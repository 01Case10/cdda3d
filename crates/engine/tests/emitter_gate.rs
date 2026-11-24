//! Addendum 11 emitter prototype gate: the "headlight-car"
//! fixture scene (sample-scene south yard at midnight, the
//! Phase 8 fixture car facing east, two headlight emitters, the
//! yard tree at (13, 20) standing in the beam) rendered with the
//! emitter set vs the SAME scene with the empty set — the empty
//! render is the control, so the brightness/shadow assertions
//! below fail on any code path where the emitter transport does
//! nothing (they did exactly that during development, when a
//! space-conversion sign error put the raster beam on the wrong
//! side of the world: every raster delta measured 0.0000).
//!
//! What is proven:
//! - Raster + RT: the beam pool (road/grass tiles in the cone,
//!   unoccluded) is measurably brighter with emitters.
//! - RT only: the occluded region (tiles in the cone behind the
//!   tree) stays at its empty-render luminance — the per-emitter
//!   shadow ray works — and far below the pool. (Raster v1 has
//!   no emitter shadows; the recorded asymmetry. The raster
//!   occluded region DOES rise, and this gate does not assert
//!   otherwise.)
//! - Ceiling (both renderers): the DARK-patch centre tile
//!   (11, 18) — inside the beam, capped by the game's DARK lit
//!   level — takes only a capped fraction of the pool's rise;
//!   the BLANK tile (11, 16) produces no geometry (RT: its hit
//!   bucket is byte-identical between renders; raster: no mesh
//!   vertex exists inside its column).
//!
//! Measured on lavapipe (llvmpipe, Mesa 25.2.8) at 480x360,
//! linear luminance (raster readbacks are sRGB-decoded first;
//! RT values are stored linear):
//! - raster pool (per-vertex bake): 0.0154 -> 0.1491 (rise 0.134)
//! - raster pool (per-block bake):  0.0195 -> 0.1898 (rise 0.170)
//! - raster DARK centre (per-block): 0.0041 -> 0.0556 (rise 0.052,
//!   0.30x the pool's rise; per-vertex it leaks to 0.103 through
//!   vertex smoothing/greedy merging — still below the pool)
//! - raster textured pool (per-vertex): 0.0162 -> 0.0431
//!   (rise 0.027; DARK centre rise 0.71x the pool's — see the
//!   textured test's own doc for detail and pre-fix numbers)
//! - RT pool: 0.0611 -> 0.6199 (rise 0.559)
//! - RT occluded {(16,21),(18,21)}: 0.0686 -> 0.0688 (rise 0.0002)
//! - RT DARK centre (11, 18): 0.0203 -> 0.2924 (rise 0.272,
//!   0.49x the pool's rise — the pool's near tiles saturate the
//!   8-bit store, compressing its measured rise; an UNCAPPED
//!   dark tile would rise ~1.0x, being closer than the pool mean)
//! - RT BLANK (11, 16): 40 hit px, mean 0.0038 -> 0.0038 (identical)
//!
//! The raster ceiling assertion runs under the per-block bake
//! (`BakeMode::PerBlock`): that bake's contract is per-tile
//! light ("the greedy mesher cannot merge across a light
//! boundary"), while the default per-vertex bake smooths light
//! across tile borders by design (Phase 7) and so cannot express
//! a one-tile ceiling step exactly — the same reason the RT
//! path, per-tile by construction, needs no such choice. The
//! pool assertion runs under BOTH bakes.
//!
//! GPU note: like every GPU gate here, a missing adapter prints
//! NOT RUN and passes vacuously — on this VM the adapter is
//! lavapipe side-loaded via VK_ICD_FILENAMES (see plan.md,
//! Stage A environment note); the recorded run had the adapter.

use engine::capture::{CaptureArgs, RendererKind};
use engine::frame::{readback_rgba, render_frame, BakeMode, EmitterScene};
use engine::rt::capture_rt_with_device_sky_emitters;
use std::collections::HashMap;

/// Beam-pool tiles: grass in the cone, unoccluded, all `LIT` in
/// the fixture grid, none adjacent to the DARK patch.
const POOL: &[(i32, i32)] = &[
    (7, 17),
    (8, 17),
    (7, 19),
    (8, 19),
    (13, 17),
    (13, 19),
    (14, 18),
    (15, 18),
];
/// In the cone but behind the tree at (13, 20) from BOTH
/// headlights — the RT shadow proof's region. (Neighbours like
/// (15, 22) are only half-shadowed or clear of the tree and are
/// deliberately not in this set.)
const OCCLUDED: &[(i32, i32)] = &[(16, 21), (18, 21)];
/// Centre of the fixture's 3x3 DARK patch (10..=12, 17..=19).
const DARK_TILE: (i32, i32) = (11, 18);
/// The fixture's BLANK tile, inside the cone.
const BLANK_TILE: (i32, i32) = (11, 16);

fn gpu() -> Option<(wgpu::Device, wgpu::Queue, String)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;
    let info = adapter.get_info();
    let name = format!("{} ({:?}, {:?})", info.name, info.device_type, info.backend);
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("emitter-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some((device, queue, name))
}

/// The headlight-car frame: companions (midnight sky, fixture
/// car, night grid) always seated; only the emitter set differs.
fn scene_args(with_emitters: bool, bake_mode: BakeMode) -> CaptureArgs {
    let scene = EmitterScene::headlight_car();
    let mut args = CaptureArgs {
        snapshot: "sample-scene".to_string(),
        camera_target: (11.0, 18.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        size: (480, 360),
        ..CaptureArgs::default()
    };
    scene.seat_companions(&mut args);
    if let Some(lit) = &mut args.lighting {
        lit.bake_mode = bake_mode;
    }
    if with_emitters {
        args.emitters = Some(scene);
    }
    args
}

fn srgb_decode(c: u8) -> f64 {
    let v = c as f64 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear luminance of an sRGB-encoded raster readback pixel.
fn lum_srgb(px: [u8; 4]) -> f64 {
    0.299 * srgb_decode(px[0]) + 0.587 * srgb_decode(px[1]) + 0.114 * srgb_decode(px[2])
}

/// Luminance of an RT pixel (the tracer stores linear values in
/// an unorm texture — no decode, the sky gates' convention).
fn lum_linear(px: [u8; 4]) -> f64 {
    (0.299 * px[0] as f64 + 0.587 * px[1] as f64 + 0.114 * px[2] as f64) / 255.0
}

/// Mean linear luminance over a small disc at the projection of
/// a tile's ground-top centre (raster region sampler).
fn raster_tile_lum(args: &CaptureArgs, img: &image::RgbaImage, tile: (i32, i32), radius: i32) -> f64 {
    let mut camera = engine::camera::OrbitCamera::new(glam::Vec3::new(
        args.camera_target.0,
        1.0,
        -args.camera_target.1,
    ));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    let (w, h) = (args.size.0 as f32, args.size.1 as f32);
    let vp = camera.view_projection(w / h);
    let p = glam::Vec3::new(tile.0 as f32 + 0.5, 1.0, -(tile.1 as f32 + 0.5));
    let clip = vp * p.extend(1.0);
    assert!(clip.w > 0.0, "tile {tile:?} behind the camera");
    let ndc = clip.truncate() / clip.w;
    let cx = (ndc.x * 0.5 + 0.5) * w;
    let cy = (1.0 - (ndc.y * 0.5 + 0.5)) * h;
    let mut sum = 0.0;
    let mut n = 0usize;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            let (x, y) = (cx as i32 + dx, cy as i32 + dy);
            if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                continue;
            }
            sum += lum_srgb(img.get_pixel(x as u32, y as u32).0);
            n += 1;
        }
    }
    assert!(n >= 8, "tile {tile:?} disc too small: {n}px");
    sum / n as f64
}

fn raster_region(args: &CaptureArgs, img: &image::RgbaImage, tiles: &[(i32, i32)]) -> f64 {
    let mut sum = 0.0;
    for &t in tiles {
        sum += raster_tile_lum(args, img, t, 3);
    }
    sum / tiles.len() as f64
}

fn render_raster(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    with: bool,
    bake_mode: BakeMode,
) -> image::RgbaImage {
    let args = scene_args(with, bake_mode);
    let frame = render_frame(device, queue, &args).expect("raster frame");
    let (img, _) = readback_rgba(device, queue, &frame.texture, frame.size).expect("readback");
    img
}

/// Single-pass RT bucketing: tile (x, y at z-level 0) -> (lum sum, n).
fn rt_buckets(args: &CaptureArgs, cap: &engine::rt::RtCapture) -> HashMap<(i32, i32), (f64, usize)> {
    let size = cap.size;
    let camera = engine::rt::capture_camera(args, size.0 as f32 / size.1 as f32);
    let eye = camera.eye;
    let fwd = (camera.target - eye).normalize_or_zero();
    let right = fwd.cross(glam::Vec3::Z).normalize_or_zero();
    let up = right.cross(fwd);
    let tan = (camera.fov_y_deg.to_radians() * 0.5).tan();
    let mut out: HashMap<(i32, i32), (f64, usize)> = HashMap::new();
    for y in 0..size.1 {
        for x in 0..size.0 {
            let d = cap.depth[(y * size.0 + x) as usize];
            if d >= cap.max_dist {
                continue;
            }
            let ndc = glam::Vec2::new(
                ((x as f32 + 0.5) / size.0 as f32) * 2.0 - 1.0,
                1.0 - ((y as f32 + 0.5) / size.1 as f32) * 2.0,
            );
            let rd = (fwd + ndc.x * tan * camera.aspect * right + ndc.y * tan * up).normalize();
            let p = eye + rd * d;
            if (p.z.floor() as i32) / 3 != 0 {
                continue;
            }
            let e = out.entry((p.x.floor() as i32, p.y.floor() as i32)).or_default();
            e.0 += lum_linear(cap.image.get_pixel(x, y).0);
            e.1 += 1;
        }
    }
    out
}

fn rt_region(b: &HashMap<(i32, i32), (f64, usize)>, tiles: &[(i32, i32)]) -> (f64, usize) {
    let mut sum = 0.0;
    let mut n = 0usize;
    for &t in tiles {
        let (s, c) = b.get(&t).copied().unwrap_or((0.0, 0));
        sum += s;
        n += c;
    }
    (if n == 0 { 0.0 } else { sum / n as f64 }, n)
}

fn rt_tile(b: &HashMap<(i32, i32), (f64, usize)>, t: (i32, i32)) -> (f64, usize) {
    b.get(&t)
        .map(|(s, n)| (s / *n as f64, *n))
        .unwrap_or((0.0, 0))
}

#[test]
fn raster_beam_pool_brightens_and_dark_tile_is_capped() {
    let Some((device, queue, name)) = gpu() else {
        println!("NOT RUN: no adapter");
        return;
    };
    println!("adapter: {name}");
    let args0 = scene_args(false, BakeMode::PerVertex);

    // Pool under BOTH bakes (the shipped default per-vertex, and
    // per-block): measured rises 0.126 / 0.158.
    for mode in [BakeMode::PerVertex, BakeMode::PerBlock] {
        let empty = render_raster(&device, &queue, false, mode);
        let with = render_raster(&device, &queue, true, mode);
        let pe = raster_region(&args0, &empty, POOL);
        let pw = raster_region(&args0, &with, POOL);
        println!("raster pool ({mode:?}): {pe:.4} -> {pw:.4}");
        assert!(
            pw > pe + 0.05 && pw > 2.5 * pe,
            "beam pool must brighten measurably ({mode:?}): {pe:.4} -> {pw:.4}"
        );
    }

    // Ceiling under the per-block bake (its contract is per-tile
    // light): the DARK centre takes a capped fraction of the
    // pool's rise. Measured: dark 0.0039 -> 0.055, pool rise 0.158.
    let empty = render_raster(&device, &queue, false, BakeMode::PerBlock);
    let with = render_raster(&device, &queue, true, BakeMode::PerBlock);
    let pe = raster_region(&args0, &empty, POOL);
    let pw = raster_region(&args0, &with, POOL);
    let de = raster_tile_lum(&args0, &empty, DARK_TILE, 2);
    let dw = raster_tile_lum(&args0, &with, DARK_TILE, 2);
    println!("raster DARK centre (per-block): {de:.4} -> {dw:.4} (pool {pe:.4} -> {pw:.4})");
    assert!(
        dw < pw * 0.5,
        "DARK tile must stay capped below half the pool: {dw:.4} vs pool {pw:.4}"
    );
    assert!(
        dw - de < (pw - pe) * 0.5,
        "DARK tile's rise must be a capped fraction of the pool's: rise {:.4} vs pool rise {:.4}",
        dw - de,
        pw - pe
    );

    // The shipped default (per-vertex) leaks through smoothing,
    // but even it must keep the DARK centre below the pool.
    let empty = render_raster(&device, &queue, false, BakeMode::PerVertex);
    let with = render_raster(&device, &queue, true, BakeMode::PerVertex);
    let pw = raster_region(&args0, &with, POOL);
    let dw = raster_tile_lum(&args0, &with, DARK_TILE, 2);
    println!("raster DARK centre (per-vertex): {dw:.4} (pool {pw:.4})");
    assert!(
        dw < pw,
        "DARK tile must stay below the pool even per-vertex: {dw:.4} vs {pw:.4}"
    );
}

/// Textured raster (the `--textures assets/ultica_run1` path):
/// the same pool/ceiling assertions over textured materials.
/// This is the regression gate for the textured-ceiling bug:
/// the shader's textured branch capped the emitter term by the
/// MEAN of the vertex colour, but the Phase 7 per-vertex bake
/// fuses albedo x bake into that colour, so the cap applied
/// albedo and bake a second time and the pool was invisible
/// over textures (CLI sample, pre-fix: pool RGB (49,53,27) vs
/// outside grass (51,56,59)). The fix's contract: textured
/// vertices carry the PURE bake multiplier in colour.rgb
/// (bake_pass_meshes), and the shader caps the emitter term by
/// that multiplier per channel.
///
/// Measured on lavapipe at 480x360 (per-vertex bake):
/// - pool: 0.0162 -> 0.0431 (rise 0.027, 2.66x; the buggy fused
///   cap measured 0.0162 -> 0.0271, rise 0.011 / 1.67x, when
///   this test was run against the unfixed code)
/// - DARK centre: 0.0163 -> 0.0355 (rise 0.019, 0.71x the
///   pool's rise — per-vertex smoothing leaks across the patch
///   border as in the flat gate; an uncapped tile at that spot,
///   closer to the headlights than the pool mean, would rise
///   ~1.0x)
#[test]
fn raster_textured_beam_pool_brightens_and_dark_tile_is_capped() {
    let Some((device, queue, name)) = gpu() else {
        println!("NOT RUN: no adapter");
        return;
    };
    println!("adapter: {name}");
    let tex_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1");
    if !tex_dir.join("materials.json").exists() {
        // assets/ is gitignored voxelizer output (the 2f.2
        // gate's convention): absent on a bare checkout.
        println!("NOT RUN: assets/ultica_run1 not present");
        return;
    }
    let render_tex = |with: bool| -> image::RgbaImage {
        let mut args = scene_args(with, BakeMode::PerVertex);
        args.textures = Some(tex_dir.clone());
        let frame = render_frame(&device, &queue, &args).expect("textured raster frame");
        let (img, _) = readback_rgba(&device, &queue, &frame.texture, frame.size).expect("readback");
        img
    };
    let args0 = scene_args(false, BakeMode::PerVertex);
    let empty = render_tex(false);
    let with = render_tex(true);
    let pe = raster_region(&args0, &empty, POOL);
    let pw = raster_region(&args0, &with, POOL);
    println!("raster textured pool (per-vertex): {pe:.4} -> {pw:.4}");
    // Thresholds from the measured numbers: the buggy fused
    // cap rose 0.011 (1.67x); the fixed multiplier cap rises
    // 0.027 (2.66x). The rise is smaller than the flat path's
    // 0.134 because the UltiCa grass texel is much darker than
    // the scene's flat grass colour — the emitter term is
    // multiplied by the texel albedo in both paths.
    assert!(
        pw > pe + 0.02 && pw > 2.2 * pe,
        "textured beam pool must brighten measurably: {pe:.4} -> {pw:.4}"
    );

    // Ceiling over textures (per-vertex bake — the textured
    // path's baker; as in the flat gate, vertex smoothing leaks
    // some light across the patch border, but the DARK centre
    // must stay below the pool and its rise must be a capped
    // fraction of the pool's rise).
    let de = raster_tile_lum(&args0, &empty, DARK_TILE, 2);
    let dw = raster_tile_lum(&args0, &with, DARK_TILE, 2);
    println!("raster textured DARK centre (per-vertex): {de:.4} -> {dw:.4} (pool {pe:.4} -> {pw:.4})");
    assert!(
        dw < pw,
        "textured DARK tile must stay below the pool: {dw:.4} vs {pw:.4}"
    );
    assert!(
        dw - de < (pw - pe) * 0.85,
        "textured DARK tile's rise must be a capped fraction of the pool's: rise {:.4} vs pool rise {:.4}",
        dw - de,
        pw - pe
    );
}

#[test]
fn raster_blank_tile_has_no_geometry() {
    // The raster ceiling contract for BLANK is the Phase 7 one:
    // a BLANK tile is never meshed. Assert it at the mesh the
    // frame itself is drawn from: no vertex strictly inside the
    // BLANK tile's column (neighbour tiles' side faces lie ON
    // the boundary planes and are excluded by the margins).
    let args = scene_args(false, BakeMode::PerVertex);
    let lit = args.lighting.as_ref().expect("fixture lighting");
    let visible = engine::cutaway::CutawayMode::HideAbove(0).visible_levels(args.cutaway_z);
    let (meshes, _stats, _spec) = engine::scene_prep::prepare_raster_sample_lit_v(
        &visible,
        None,
        lit,
        args.vehicles.as_ref(),
    )
    .expect("scene prep");
    let (bx, by) = BLANK_TILE;
    let mut inside = 0usize;
    for pass in mesher::block::Pass::ALL {
        let mesh = meshes.get(pass);
        for p in &mesh.positions {
            // Mesher world: x east, y up, z = -south.
            if p[0] > bx as f32 + 0.02
                && p[0] < bx as f32 + 0.98
                && p[2] < -(by as f32) - 0.02
                && p[2] > -(by as f32) - 0.98
                && p[1] > 0.02
                && p[1] < 2.98
            {
                inside += 1;
            }
        }
    }
    assert_eq!(
        inside, 0,
        "BLANK tile {BLANK_TILE:?} must contribute no mesh vertices"
    );
}

/// The committed capture set (`emitter-gate/` at the repo
/// root): the gate scene with/without emitters in both
/// renderers at 1280x720. Regenerated only under
/// `UPDATE_EMITTER_CAPTURES=1` (the golden gate's UPDATE_GOLDEN
/// convention); a normal suite run skips it, so the committed
/// PNGs are stable artifacts, not test output.
#[test]
fn emitter_capture_set() {
    if std::env::var_os("UPDATE_EMITTER_CAPTURES").is_none() {
        println!("capture set regeneration skipped (set UPDATE_EMITTER_CAPTURES=1)");
        return;
    }
    let Some((device, queue, name)) = gpu() else {
        println!("NOT RUN: no adapter");
        return;
    };
    println!("adapter: {name}");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../emitter-gate");
    std::fs::create_dir_all(&dir).expect("create emitter-gate dir");
    for (renderer, rname) in [(RendererKind::Raster, "raster"), (RendererKind::Rt, "rt")] {
        for with in [false, true] {
            let scene = EmitterScene::headlight_car();
            let mut args = CaptureArgs {
                snapshot: "sample-scene".to_string(),
                renderer,
                camera_target: (11.0, 18.0),
                yaw_deg: 45.0,
                pitch_deg: 45.0,
                distance: 28.0,
                size: (1280, 720),
                ..CaptureArgs::default()
            };
            scene.seat_companions(&mut args);
            if with {
                args.emitters = Some(scene);
            }
            let frame = render_frame(&device, &queue, &args).expect("capture frame");
            let (img, _) =
                readback_rgba(&device, &queue, &frame.texture, frame.size).expect("readback");
            let label = if with { "emitters" } else { "empty" };
            let path = dir.join(format!("headlight-car-{rname}-{label}.png"));
            img.save(&path).expect("save capture");
            println!("wrote {}", path.display());
        }
    }
}

#[test]
fn rt_beam_pool_occlusion_and_ceiling() {
    let Some((device, queue, name)) = gpu() else {
        println!("NOT RUN: no adapter");
        return;
    };
    println!("adapter: {name}");
    let scene = EmitterScene::headlight_car();
    let sky = scene.sky.uniforms_grid_space();
    let emitters = scene.uniforms_grid_space();
    let mut buckets = Vec::new();
    for with in [false, true] {
        let mut args = scene_args(with, BakeMode::PerVertex);
        args.renderer = RendererKind::Rt;
        let cap = capture_rt_with_device_sky_emitters(
            &device,
            &queue,
            &args,
            Some(&sky),
            if with { Some(&emitters) } else { None },
        )
        .expect("rt capture");
        buckets.push(rt_buckets(&args, &cap));
    }
    let (pe, pn) = rt_region(&buckets[0], POOL);
    let (pw, _) = rt_region(&buckets[1], POOL);
    println!("RT pool: n={pn} {pe:.4} -> {pw:.4}");
    assert!(pn > 200, "pool must have real coverage: {pn}px");
    assert!(
        pw > pe + 0.25 && pw > 4.0 * pe,
        "RT beam pool must brighten measurably: {pe:.4} -> {pw:.4}"
    );

    // Shadow proof: the occluded region does not move (measured
    // rise 0.0002) and sits far below the lit pool.
    let (oe, on) = rt_region(&buckets[0], OCCLUDED);
    let (ow, _) = rt_region(&buckets[1], OCCLUDED);
    println!("RT occluded: n={on} {oe:.4} -> {ow:.4}");
    assert!(on > 40, "occluded region must have real coverage: {on}px");
    assert!(
        ow <= oe + 0.01,
        "occluded region must stay dark under the shadow ray: {oe:.4} -> {ow:.4}"
    );
    assert!(
        ow < pw * 0.25,
        "occluded region must sit far below the pool: {ow:.4} vs pool {pw:.4}"
    );

    // Ceiling: the DARK centre's rise is capped (measured 0.49x
    // the pool's rise — see the module doc for why it is not the
    // raw 0.25/0.85 lit-factor ratio).
    let (de, _) = rt_tile(&buckets[0], DARK_TILE);
    let (dw, _) = rt_tile(&buckets[1], DARK_TILE);
    println!("RT DARK centre: {de:.4} -> {dw:.4}");
    assert!(
        dw < pw * 0.55 && dw - de < (pw - pe) * 0.6,
        "DARK tile must be capped: {de:.4} -> {dw:.4} vs pool {pe:.4} -> {pw:.4}"
    );

    // BLANK: no geometry in the tree — the few bucket pixels are
    // rim/boundary hits, identical between the renders.
    let (be, bn) = rt_tile(&buckets[0], BLANK_TILE);
    let (bw, wn) = rt_tile(&buckets[1], BLANK_TILE);
    println!("RT BLANK tile: n={bn}/{wn} {be:.4} -> {bw:.4}");
    assert_eq!(bn, wn, "BLANK tile hit count must not change with emitters");
    assert!(
        (bw - be).abs() <= 0.002,
        "BLANK tile must not light up: {be:.4} -> {bw:.4}"
    );
}
