//! RT live-loop cache gate (2026-11-24, the 0 fps fix + its
//! regression armour).
//!
//! The windowed RT demo repaid the full scene build on every
//! presented frame: 64-tree builds, texture-spec construction
//! (PNG decode + mip chains), GPU uploads, compute-pipeline
//! compilation — and, worst, `MaterialTable::load` re-reading
//! and re-parsing the ~246 MB UltiCa `materials.json` several
//! times per frame. Measured on a software adapter at
//! 1280x720: ~10.5 s/frame for the street_block demo scene.
//!
//! Proofs (numbered as proposed when the armour was added):
//! 1. The material table parses at most once: `load_cached`
//!    returns the same Arc, the parse counter works, and
//!    rendering textured frames on BOTH renderer paths never
//!    re-parses.
//! 2. Relative timing: steady-state frames together cost less
//!    than the one build frame (hardware-independent ratio).
//! 3. App wiring lives in the binary's own test module
//!    (`main.rs::app_wiring_tests`) — it drives the windowed
//!    render path, which this file cannot reach.
//! 4. Invalidation matrix: every scene-determining mutation
//!    (snapshot, textures, lighting grid, props, vehicles)
//!    rebuilds, and the rebuilt frame matches an uncached
//!    render of the mutated scene.
//! 5. No stale-uniform leak: a frame with sky + emitters
//!    seated, followed by a frame with neither, matches the
//!    uncached no-sky/no-emitter frame.
//! 6. Per-frame uniforms never rebuild: resize and a sky
//!    time change trace from the held scene and match their
//!    uncached renders.
//! Plus the original core proof: a cached frame is
//! pixel-identical to the uncached frame, and a camera move
//! does not rebuild.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use engine::capture::{CaptureArgs, RendererKind};
use engine::frame::{readback_rgba, render_frame, render_frame_with_rt_cache};
use engine::wire::WireScene;
use sim::{DefRegistry, Scene, World};

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rt-cache-gate"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .ok()
    })
}

fn textures_dir() -> PathBuf {
    root().join("assets/ultica_run1")
}

fn have_table() -> bool {
    textures_dir().join("materials.json").exists()
}

/// The demo scene (street_block over the wire, textures +
/// props + snapshot lighting), optionally without props.
fn demo_args(with_props: bool) -> CaptureArgs {
    let root = root();
    let textures = textures_dir();
    let reg =
        DefRegistry::load(&root.join("tools/sim-defs/generated/defs.json")).expect("defs load");
    let scene =
        Scene::load_file(&root.join("crates/sim/scenes/street_block.toml")).expect("scene load");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    let snap = world.snapshot(&reg);
    let defs = reg.tiledefs().clone();
    let table = engine::materials::MaterialTable::load_cached(&textures).expect("table loads");
    let mut wire = WireScene::from_wire(&defs, &snap, &engine::wire::palette_resolver(&defs));
    if with_props {
        wire = wire.with_props(engine::wire::prop_scene_for_wire(&defs, &snap, &table));
    }
    let lit = engine::frame::LightingScene::from_snapshot(&snap, wire.min_game_z);
    let wire = wire.with_companions(None, Some(lit));
    let mut args = CaptureArgs::default();
    args.snapshot = "wire".into();
    args.renderer = RendererKind::Rt;
    args.size = (320, 180);
    args.cutaway_z = 0;
    args.textures = Some(textures);
    args.lighting = wire.lighting.clone();
    args.wire = Some(wire);
    args
}

fn pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    outcome: &engine::frame::FrameOutcome,
) -> Vec<u8> {
    let (img, _) = readback_rgba(device, queue, &outcome.texture, outcome.size).expect("readback");
    img.as_raw().clone()
}

fn builds(cache: &Option<engine::rt::RtSceneCache>) -> usize {
    cache.as_ref().map(|c| c.build_count()).unwrap_or(0)
}

// ----------------------------------------------------------------
// Core proof (the original gate).

#[test]
fn rt_scene_cache_builds_once_and_matches_uncached() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    let args = demo_args(true);

    let reference = render_frame(&device, &queue, &args).expect("uncached frame");
    let ref_px = pixels(&device, &queue, &reference);

    let mut cache: Option<engine::rt::RtSceneCache> = None;
    let first = render_frame_with_rt_cache(&device, &queue, &args, &mut cache).expect("cached 1");
    assert_eq!(
        pixels(&device, &queue, &first),
        ref_px,
        "cached frame must be pixel-identical to the uncached frame"
    );
    let mut moved = args.clone();
    moved.yaw_deg += 25.0;
    let _ = render_frame_with_rt_cache(&device, &queue, &moved, &mut cache).expect("cached 2");
    assert_eq!(
        builds(&cache),
        1,
        "a camera move must trace from the held scene, not rebuild it"
    );

    let mut cut = args.clone();
    cut.cutaway_z = -1;
    let _ = render_frame_with_rt_cache(&device, &queue, &cut, &mut cache).expect("cached 3");
    assert_eq!(builds(&cache), 2, "changing the cutaway must rebuild the scene");
}

// ----------------------------------------------------------------
// #1: the material table parses at most once, on any frame path.

#[test]
fn material_table_load_cached_is_identity() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let dir = textures_dir();
    let a = engine::materials::MaterialTable::load_cached(&dir).expect("first load");
    let b = engine::materials::MaterialTable::load_cached(&dir).expect("second load");
    assert!(
        std::sync::Arc::ptr_eq(&a, &b),
        "load_cached must return the cached table, not a re-parse"
    );
}

#[test]
fn frame_paths_never_reparse_the_material_table() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    use engine::materials::MaterialTable;

    // The counter itself works: one explicit uncached load is
    // exactly one parse.
    let c0 = MaterialTable::parse_count();
    let _fresh = MaterialTable::load(&textures_dir()).expect("explicit load");
    assert_eq!(
        MaterialTable::parse_count(),
        c0 + 1,
        "parse counter must count a real load"
    );

    // First frame may parse once (cold cache); after that, no
    // frame on either renderer path may parse again.
    let args = demo_args(true);
    let _ = render_frame(&device, &queue, &args).expect("rt frame");
    let settled = MaterialTable::parse_count();
    assert!(
        settled <= c0 + 2,
        "the first frame may add at most the one cached parse (c0={c0}, settled={settled})"
    );
    let _ = render_frame(&device, &queue, &args).expect("rt frame 2");
    let mut raster = args.clone();
    raster.renderer = RendererKind::Raster;
    let _ = render_frame(&device, &queue, &raster).expect("raster frame");
    let _ = render_frame(&device, &queue, &args).expect("rt frame 3");
    assert_eq!(
        MaterialTable::parse_count(),
        settled,
        "textured frames must never re-parse the material table"
    );
}

// ----------------------------------------------------------------
// #2: relative timing — steady frames are cheaper than the build.

#[test]
fn steady_state_frames_are_cheaper_than_the_build_frame() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    let args = demo_args(true);
    let mut cache: Option<engine::rt::RtSceneCache> = None;
    let mut times = Vec::new();
    for i in 0..5 {
        let mut a = args.clone();
        a.yaw_deg += i as f32 * 3.0;
        let t = Instant::now();
        let _ = render_frame_with_rt_cache(&device, &queue, &a, &mut cache).expect("frame");
        device.poll(wgpu::Maintain::Wait);
        times.push(t.elapsed().as_secs_f64());
    }
    let steady: f64 = times[1..].iter().sum();
    assert!(
        steady < times[0],
        "steady-state frames ({steady:.2}s total) must cost less than the build frame ({:.2}s) — a per-frame rebuild regression makes them cost ~4x the build frame",
        times[0]
    );
}

// ----------------------------------------------------------------
// #4: invalidation matrix.

#[test]
fn cache_invalidation_matrix() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    let base = demo_args(true);
    let mut cache: Option<engine::rt::RtSceneCache> = None;
    let _ = render_frame_with_rt_cache(&device, &queue, &base, &mut cache).expect("base frame");
    assert_eq!(builds(&cache), 1);
    let mut expected = 1usize;

    // Each mutation: rebuild (+1) AND pixels equal to an
    // uncached render of the mutated args.
    let check = |name: &str,
                 mutated: CaptureArgs,
                 cache: &mut Option<engine::rt::RtSceneCache>,
                 expected: &mut usize| {
        let got = render_frame_with_rt_cache(&device, &queue, &mutated, cache)
            .unwrap_or_else(|e| panic!("{name}: cached render failed: {e:?}"));
        *expected += 1;
        assert_eq!(
            builds(cache),
            *expected,
            "{name}: changing this input must rebuild the scene"
        );
        let reference = render_frame(&device, &queue, &mutated)
            .unwrap_or_else(|e| panic!("{name}: uncached render failed: {e:?}"));
        assert_eq!(
            pixels(&device, &queue, &got),
            pixels(&device, &queue, &reference),
            "{name}: rebuilt frame must match the uncached render"
        );
    };

    let mut m = base.clone();
    m.snapshot = "wire-renamed".into();
    check("snapshot", m, &mut cache, &mut expected);

    let mut m = base.clone();
    m.textures = None;
    check("textures removed", m, &mut cache, &mut expected);

    let mut m = base.clone();
    let mut lit = m.lighting.clone().expect("demo args carry lighting");
    let mut tile = lit.grid.get(5, 5, 0);
    tile.seen = if tile.seen == 0 { 1 } else { 0 };
    lit.grid.set(5, 5, 0, tile);
    m.lighting = Some(lit);
    check("lighting grid tile", m, &mut cache, &mut expected);

    let mut m = demo_args(false);
    m.lighting = m.wire.as_ref().and_then(|w| w.lighting.clone());
    check("props removed", m, &mut cache, &mut expected);

    let mut m = base.clone();
    m.vehicles = Some(engine::frame::VehicleScene::fixture_car([4, 5, 0], 23.0));
    check("vehicles seated", m, &mut cache, &mut expected);
}

// ----------------------------------------------------------------
// #5: no stale sky/emitter leak across cached frames.

#[test]
fn cached_frames_do_not_leak_sky_or_emitters() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    let base = demo_args(true);
    let mut cache: Option<engine::rt::RtSceneCache> = None;

    // Frame 1: sky + emitters seated.
    let mut dressed = base.clone();
    dressed.sky = Some(engine::frame::SkyScene::fake_day(12.0, 4));
    dressed.transients = Some(engine::frame::TransientScene::fixture_burst(120));
    let _ = render_frame_with_rt_cache(&device, &queue, &dressed, &mut cache).expect("dressed");

    // Frame 2: neither — must equal the uncached plain frame,
    // bit for bit (a cached renderer that kept frame 1's sky
    // or emitter uniforms would glow here).
    let plain = render_frame_with_rt_cache(&device, &queue, &base, &mut cache).expect("plain");
    let reference = render_frame(&device, &queue, &base).expect("uncached plain");
    assert_eq!(
        pixels(&device, &queue, &plain),
        pixels(&device, &queue, &reference),
        "a cached frame must not leak the previous frame's sky/emitters"
    );
    assert_eq!(builds(&cache), 1, "sky/emitters are uniforms, never a rebuild");
}

// ----------------------------------------------------------------
// #6: per-frame uniforms (resize, sky time) never rebuild.

#[test]
fn resize_and_sky_change_trace_without_rebuild() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !have_table() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    let mut base = demo_args(true);
    base.sky = Some(engine::frame::SkyScene::fake_day(12.0, 4));
    let mut cache: Option<engine::rt::RtSceneCache> = None;
    let noon = render_frame_with_rt_cache(&device, &queue, &base, &mut cache).expect("noon");
    assert_eq!(builds(&cache), 1);

    // Resize: new size, same build, pixels match uncached.
    let mut small = base.clone();
    small.size = (256, 144);
    let resized = render_frame_with_rt_cache(&device, &queue, &small, &mut cache).expect("resize");
    assert_eq!(resized.size, (256, 144));
    assert_eq!(builds(&cache), 1, "a resize must not rebuild the scene");
    let reference = render_frame(&device, &queue, &small).expect("uncached resize");
    assert_eq!(
        pixels(&device, &queue, &resized),
        pixels(&device, &queue, &reference),
        "resized cached frame must match the uncached render"
    );

    // Sky time change: different picture, same build, matches
    // the uncached render of the same sky.
    let mut night = base.clone();
    night.sky = Some(engine::frame::SkyScene::fake_day(0.0, 4));
    let night_frame =
        render_frame_with_rt_cache(&device, &queue, &night, &mut cache).expect("night");
    assert_eq!(builds(&cache), 1, "a sky change must not rebuild the scene");
    assert_ne!(
        pixels(&device, &queue, &night_frame),
        pixels(&device, &queue, &noon),
        "midnight must not render as noon"
    );
    let reference = render_frame(&device, &queue, &night).expect("uncached night");
    assert_eq!(
        pixels(&device, &queue, &night_frame),
        pixels(&device, &queue, &reference),
        "night cached frame must match the uncached render"
    );
}
