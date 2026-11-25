//! RT live-loop cache gate (2026-11-24, the 0 fps fix).
//!
//! The windowed RT demo repaid the full scene build on every
//! presented frame: 64-tree builds, texture-spec construction
//! (PNG decode + mip chains), GPU uploads, compute-pipeline
//! compilation — and, worst, `MaterialTable::load` re-reading
//! and re-parsing the ~246 MB UltiCa `materials.json` several
//! times per frame. Measured on a software adapter at
//! 1280x720: ~10.5 s/frame for the street_block demo scene.
//!
//! Proofs:
//! 1. `MaterialTable::load_cached` returns the SAME table
//!    (Arc identity) on a second call for the same directory.
//! 2. Through `frame::render_frame_with_rt_cache`, two frames
//!    of the demo scene (camera moved between them) build the
//!    scene exactly once (`RtSceneCache::build_count` == 1),
//!    and a cached frame at the uncached frame's camera is
//!    pixel-identical to the uncached build-per-call frame.
//! 3. Changing a scene-determining input (cutaway) rebuilds.

use std::path::PathBuf;
use std::sync::Mutex;

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

fn demo_args() -> CaptureArgs {
    let root = root();
    let textures = root.join("assets/ultica_run1");
    let reg =
        DefRegistry::load(&root.join("tools/sim-defs/generated/defs.json")).expect("defs load");
    let scene =
        Scene::load_file(&root.join("crates/sim/scenes/street_block.toml")).expect("scene load");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    let snap = world.snapshot(&reg);
    let defs = reg.tiledefs().clone();
    let table = engine::materials::MaterialTable::load_cached(&textures).expect("table loads");
    let wire = WireScene::from_wire(&defs, &snap, &engine::wire::palette_resolver(&defs))
        .with_props(engine::wire::prop_scene_for_wire(&defs, &snap, &table));
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

#[test]
fn material_table_load_cached_is_identity() {
    let dir = root().join("assets/ultica_run1");
    if !dir.join("materials.json").exists() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let a = engine::materials::MaterialTable::load_cached(&dir).expect("first load");
    let b = engine::materials::MaterialTable::load_cached(&dir).expect("second load");
    assert!(
        std::sync::Arc::ptr_eq(&a, &b),
        "load_cached must return the cached table, not a re-parse"
    );
}

#[test]
fn rt_scene_cache_builds_once_and_matches_uncached() {
    let _guard = GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if !root().join("assets/ultica_run1/materials.json").exists() {
        eprintln!("SKIP rt_cache_gate: real UltiCa table absent (assets/ultica_run1)");
        return;
    }
    let Some((device, queue)) = gpu() else {
        eprintln!("SKIP rt_cache_gate: no wgpu adapter available");
        return;
    };
    let args = demo_args();

    // Uncached reference frame (build-per-call path).
    let reference = render_frame(&device, &queue, &args).expect("uncached frame");
    let (ref_img, _) =
        readback_rgba(&device, &queue, &reference.texture, reference.size).expect("readback");

    // Cached path: same camera, then a moved camera.
    let mut cache: Option<engine::rt::RtSceneCache> = None;
    let first = render_frame_with_rt_cache(&device, &queue, &args, &mut cache).expect("cached 1");
    let (first_img, _) =
        readback_rgba(&device, &queue, &first.texture, first.size).expect("readback");
    assert_eq!(
        first_img.as_raw(),
        ref_img.as_raw(),
        "cached frame must be pixel-identical to the uncached frame"
    );
    let mut moved = args.clone();
    moved.yaw_deg += 25.0;
    let _ = render_frame_with_rt_cache(&device, &queue, &moved, &mut cache).expect("cached 2");
    assert_eq!(
        cache.as_ref().map(|c| c.build_count()),
        Some(1),
        "a camera move must trace from the held scene, not rebuild it"
    );

    // A scene-determining change rebuilds.
    let mut cut = args.clone();
    cut.cutaway_z = -1;
    let _ = render_frame_with_rt_cache(&device, &queue, &cut, &mut cache).expect("cached 3");
    assert_eq!(
        cache.as_ref().map(|c| c.build_count()),
        Some(2),
        "changing the cutaway must rebuild the scene"
    );
}
