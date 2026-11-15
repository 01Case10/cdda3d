//! Phase 2h gates: the live frame loop (plan.md Phase 2h).
//!
//! One composition — `engine::frame::render_frame`, order
//! world → fields → billboards → post → UI — exercised end to end:
//!
//! - 2h.1: the pass order is asserted in code structure
//!   (`FRAME_ORDER`) and in execution (`FrameStats::passes`); the
//!   windowed route (render_frame + `PresentBlit`) and the
//!   headless capture of the same scene agree within the 2g.3
//!   tolerance model.
//! - 2h.2: the billboard pass inside the engine frame, with the
//!   real UltiCa atlas: an entity behind the fake house's north
//!   wall is fully occluded, one behind its window is visible, and
//!   alpha picking the hulk returns its entity id.
//! - 2h.3: the lighting contract in both renderers: a day/night
//!   pair differs per the Phase 7 environment curve and only in
//!   lighting; `BLANK` tiles render in neither renderer;
//!   `MEMORIZED` tiles show the pinned desaturation.
//! - 2h.4: the UI quad composites last, over the post-processed
//!   world, with the terrain rect masked per the frame itself; a
//!   synthetic click round-trips through the Phase 4 router + DDA
//!   picking to the expected tile and through the FlatBuffers
//!   envelope; gate 4.12 is measured in-loop.
//! - 2h.5: the fully composed frame (world + fields + billboards
//!   + post + stub UI) is a golden in the 2g.3 harness model.
//!
//! GPU tests serialize on a lock and skip LOUDLY (banner, never a
//! silent pass) when no wgpu adapter exists, mirroring
//! golden_gate.rs. Run standalone with --nocapture: GPU suites go
//! vacuous under parallel adapter contention.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use engine::camera::OrbitCamera;
use engine::capture::{CaptureArgs, CaptureError, RendererKind};
use engine::frame::{
    self, BillboardScene, FramePass, LightingScene, FRAME_ORDER,
};
use glam::Vec3;

static GPU_LOCK: Mutex<()> = Mutex::new(());

const SIZE: (u32, u32) = (640, 360);
// The 2g.3 tolerance model, reused verbatim.
const CHANNEL_TOL: u32 = 8;
const MISMATCH_BUDGET: f64 = 0.01;
const MEAN_ABS_BUDGET: f64 = 2.0;

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn loud_skip(gate: &str, reason: &str) {
    let banner = format!(
        "================================================================\n\
         {gate}: NOT RUN — {reason}\n\
         This run proves NOTHING about the frame loop. The gate only\n\
         counts on a machine with a wgpu adapter (lavapipe counts).\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
}

/// Create a headless device, or None when no adapter exists.
fn gpu() -> Option<(wgpu::Device, wgpu::Queue, String)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        let name = adapter.get_info().name;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("frame-gate-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .ok()?;
        Some((device, queue, name))
    })
}

/// Render one composed frame and read it back.
fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> (image::RgbaImage, frame::FrameOutcome) {
    let outcome = frame::render_frame(device, queue, args).expect("render_frame");
    let (img, _distinct) =
        frame::readback_rgba(device, queue, &outcome.texture, outcome.size).expect("readback");
    (img, outcome)
}

/// The engine camera `render_frame` will use for these args
/// (mirrors the frame's own construction).
fn cam_for(args: &CaptureArgs) -> OrbitCamera {
    let mut camera = OrbitCamera::new(Vec3::new(args.camera_target.0, 1.0, -args.camera_target.1));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    camera
}

/// Project a world point to frame pixels.
fn project(cam: &OrbitCamera, p: Vec3, size: (u32, u32)) -> (f32, f32) {
    let vp = cam.view_projection(size.0 as f32 / size.1 as f32);
    let clip = vp * p.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    (
        (ndc.x * 0.5 + 0.5) * size.0 as f32,
        (0.5 - ndc.y * 0.5) * size.1 as f32,
    )
}

/// Mean RGB over a square patch (radius r) around a pixel.
fn patch_mean(img: &image::RgbaImage, cx: f32, cy: f32, r: i32) -> [f32; 3] {
    let (cx, cy) = (cx.round() as i32, cy.round() as i32);
    let (mut acc, mut n) = ([0f64; 3], 0u32);
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
                continue;
            }
            let p = img.get_pixel(x as u32, y as u32).0;
            for c in 0..3 {
                acc[c] += p[c] as f64;
            }
            n += 1;
        }
    }
    assert!(n > 0, "patch at ({cx}, {cy}) is empty");
    [
        (acc[0] / n as f64) as f32,
        (acc[1] / n as f64) as f32,
        (acc[2] / n as f64) as f32,
    ]
}

fn saturation(m: [f32; 3]) -> f32 {
    let max = m[0].max(m[1]).max(m[2]);
    let min = m[0].min(m[1]).min(m[2]);
    if max <= 0.0 {
        0.0
    } else {
        (max - min) / max
    }
}

fn brightness(m: [f32; 3]) -> f32 {
    (m[0] + m[1] + m[2]) / 3.0
}

/// The 2g.3 diff model (per-pixel channel tolerance + budgets).
fn diff_stats(a: &image::RgbaImage, b: &image::RgbaImage) -> (f64, f64, u32) {
    assert_eq!(a.dimensions(), b.dimensions(), "frame sizes differ");
    let mut mismatched = 0usize;
    let mut err_sum = 0f64;
    let mut max_delta = 0u32;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let mut px_max = 0u32;
        for c in 0..4 {
            let d = (pa.0[c] as i32 - pb.0[c] as i32).unsigned_abs();
            px_max = px_max.max(d);
            err_sum += d as f64;
        }
        max_delta = max_delta.max(px_max);
        if px_max > CHANNEL_TOL {
            mismatched += 1;
        }
    }
    let total = (a.width() * a.height()) as usize;
    (
        mismatched as f64 / total.max(1) as f64,
        err_sum / (total * 4) as f64,
        max_delta,
    )
}

/// Occlusion identity with a counted failure report (never an
/// 8-megabyte assert_eq dump of both frames). Two frames of the
/// same scene ± a fully occluded entity must agree everywhere
/// except blend rounding: the window zombie renders through the
/// window's alpha-0.35 pass, and removing another entity shifts
/// the packed-atlas layout, which moves one sampled texel by a
/// sub-texel UV step — measured on lavapipe as exactly 1 pixel
/// differing by 1 LSB in one channel. A real visibility leak
/// changes hundreds of pixels by large deltas, so the bounds
/// below (<= 16 rounding pixels, max delta <= 2, zero pixels
/// beyond the 2g.3 channel tolerance) still prove occlusion.
fn assert_frames_identical(a: &image::RgbaImage, b: &image::RgbaImage, what: &str) {
    let (frac, mean_abs, max_delta) = diff_stats(a, b);
    let raw_diffs = a
        .pixels()
        .zip(b.pixels())
        .filter(|(p, q)| p.0 != q.0)
        .count();
    println!(
        "{what}: {raw_diffs} raw differing px, max delta {max_delta},          beyond-tolerance {:.4}%",
        frac * 100.0
    );
    assert!(
        frac == 0.0 && raw_diffs <= 16 && max_delta <= 2,
        "{what}: frames differ beyond blend rounding: {raw_diffs} px,          max delta {max_delta}, mean |err| {mean_abs:.3}"
    );
}

fn assert_within_tol(what: &str, a: &image::RgbaImage, b: &image::RgbaImage) {
    let (frac, mean_abs, max_delta) = diff_stats(a, b);
    println!(
        "{what}: mismatch {:.3}% px, mean |err| {mean_abs:.3}, max delta {max_delta}",
        frac * 100.0
    );
    assert!(
        frac <= MISMATCH_BUDGET && mean_abs <= MEAN_ABS_BUDGET,
        "{what}: outside the 2g.3 tolerance (budget {:.1}% px / mean {MEAN_ABS_BUDGET}): \
         {:.3}% px, mean |err| {mean_abs:.3}",
        MISMATCH_BUDGET * 100.0,
        frac * 100.0
    );
}

fn base_args() -> CaptureArgs {
    CaptureArgs {
        size: SIZE,
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        cutaway_z: 0,
        ..CaptureArgs::default()
    }
}

/// The fully composed stub frame: fx demo defs (fields + post),
/// fixture billboard crowd, stub UI sidebar.
fn composed_args() -> CaptureArgs {
    let mut args = base_args();
    args.effects_defs = Some(fx::effect::demo_defs());
    args.billboards = Some(BillboardScene::fixture_crowd());
    let (store, note) = frame::stub_ui_store(320, 200, false).expect("stub ui store");
    args.ui = Some(frame::prepare_ui(&store, &note).expect("prepare ui"));
    args
}

fn ultica_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1")
}

// ---------------------------------------------------------------------
// 2h.1 — frame order + windowed/headless parity
// ---------------------------------------------------------------------

#[test]
fn gate_2h1_frame_order_and_windowed_parity() {
    // The order is a code-structure fact first: the canonical
    // sequence the composition is written in.
    assert_eq!(
        FRAME_ORDER,
        [
            FramePass::World,
            FramePass::Fields,
            FramePass::Billboards,
            FramePass::Post,
            FramePass::Ui
        ],
        "canonical frame order changed — update plan.md Phase 2h with it"
    );

    let _guard = gpu_lock();
    let Some((device, queue, _name)) = gpu() else {
        loud_skip("2h.1", "no wgpu adapter available");
        return;
    };
    let args = composed_args();
    let (capture_img, outcome) = render(&device, &queue, &args);

    // Execution matches structure: every layer on → every pass
    // ran, in canonical order.
    assert_eq!(
        outcome.stats.passes, FRAME_ORDER,
        "executed passes must be the canonical order when all layers are on"
    );
    assert!(outcome.stats.billboards > 0, "billboard pass drew nothing");

    // The windowed route: the same composition presented through
    // PresentBlit into a surface-format target.
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("2h1-present-target"),
        size: wgpu::Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: engine::render::CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());
    let blit = frame::PresentBlit::new(&device, engine::render::CAPTURE_FORMAT);
    blit.draw(&device, &queue, &outcome.texture, &target_view);
    let (windowed_img, _) =
        frame::readback_rgba(&device, &queue, &target, SIZE).expect("present readback");
    assert_within_tol("2h.1 windowed route vs composed frame", &capture_img, &windowed_img);

    // The headless capture of the same scene (its own device, the
    // run_capture path) agrees with the in-test composition.
    let scratch = std::env::temp_dir().join(format!("cdda-voxel-2h1-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut cap_args = args.clone();
    cap_args.out = scratch.join("composed.png");
    match engine::capture::run_capture(&cap_args) {
        Ok(_) => {
            let png = image::open(&cap_args.out).expect("capture png").to_rgba8();
            assert_within_tol("2h.1 headless capture vs composed frame", &capture_img, &png);
        }
        Err(CaptureError::NoAdapter) => loud_skip("2h.1 (capture half)", "no wgpu adapter"),
        Err(e) => panic!("run_capture failed: {e}"),
    }
    println!("2h.1: order asserted; windowed and capture routes agree within 2g.3 tolerance");
}

// ---------------------------------------------------------------------
// 2h.2 — billboards in the engine frame (real UltiCa sprites)
// ---------------------------------------------------------------------

#[test]
fn gate_2h2_billboard_occlusion_and_picking() {
    let dir = ultica_dir();
    if !dir.join("sprites.json").exists() {
        loud_skip(
            "2h.2",
            "real UltiCa atlas absent (assets/ultica_run1) — the gate needs the \
             voxelizer's real output, not the fixture atlas",
        );
        return;
    }
    let _guard = gpu_lock();
    let Some((device, queue, _name)) = gpu() else {
        loud_skip("2h.2", "no wgpu adapter available");
        return;
    };
    let scene = BillboardScene::ultica_crowd(&dir).expect("ultica crowd");
    // Camera south of the house, above it, looking north
    // (yaw 180): the north wall + window are on the far side.
    let mut args = base_args();
    args.yaw_deg = 180.0;
    args.pitch_deg = 60.0;
    args.distance = 20.0;

    let mut with_all = args.clone();
    with_all.billboards = Some(scene.clone());
    let (img_all, outcome) = render(&device, &queue, &with_all);
    assert_eq!(outcome.stats.passes, vec![FramePass::World, FramePass::Billboards]);

    // Without the behind-wall skeleton (id 7): its sprite AND its
    // shadow are behind the house from this camera, so the frame
    // must not change at all.
    let mut no_skeleton = scene.clone();
    no_skeleton.entities.retain(|e| e.id != 7);
    let mut args_ns = args.clone();
    args_ns.billboards = Some(no_skeleton);
    let (img_ns, _) = render(&device, &queue, &args_ns);
    assert_frames_identical(
        &img_all,
        &img_ns,
        "entity behind the wall must be fully occluded by the world depth",
    );

    // Without the window zombie (id 6): it IS visible through the
    // north window's transparent pass, so the frame must change.
    let mut no_window = scene.clone();
    no_window.entities.retain(|e| e.id != 6);
    let mut args_nw = args.clone();
    args_nw.billboards = Some(no_window);
    let (img_nw, _) = render(&device, &queue, &args_nw);
    // Measured on lavapipe: 0.028% of pixels beyond tolerance
    // (~65 px — the window opening's worth of zombie, blended
    // through the window's alpha-0.35 pass). The bound sits at
    // 0.01%: an order above the rounding floor (0 px), below signal.
    let (wfrac, _, _) = diff_stats(&img_all, &img_nw);
    assert!(
        wfrac > 0.0001,
        "entity behind the window must be visible in the composed frame          (frame delta {:.4}%)",
        wfrac * 100.0
    );

    // Alpha picking: a ray through the hulk's center returns the
    // hulk's entity id; a ray through the skeleton returns its id
    // (picking is per-sprite alpha — world occlusion is the depth
    // test's job, asserted above).
    let cam_hulk = {
        let mut c = OrbitCamera::new(ui_share::picking::tile_center_world((9, 4, 0)));
        c.yaw_deg = 0.0;
        c.pitch_deg = 45.0;
        c.distance = 28.0;
        c.clamp();
        c
    };
    let eye = cam_hulk.eye();
    let dir_h = (cam_hulk.target - eye).normalize();
    assert_eq!(
        frame::pick_billboard_entity(&scene, &cam_hulk, eye, dir_h),
        Some(8),
        "picking the hulk must return its entity id"
    );
    // Aim at mid-body height (0.75 world units): the skeleton's
    // 32x48 sprite stands on its tile, so the tile *center*
    // (1.5 units up) grazes the sprite's top edge — the hulk's
    // 64x96 sprite covers its tile center, a 48px sprite does
    // not. Picking is alpha-exact; the ray must cross the body.
    let cam_skel = {
        let mut c = OrbitCamera::new(Vec3::new(1.5, 0.75, 0.5));
        c.yaw_deg = 0.0;
        c.pitch_deg = 45.0;
        c.distance = 28.0;
        c.clamp();
        c
    };
    let eye_s = cam_skel.eye();
    let dir_s = (cam_skel.target - eye_s).normalize();
    assert_eq!(
        frame::pick_billboard_entity(&scene, &cam_skel, eye_s, dir_s),
        Some(7),
        "picking the skeleton must return its entity id"
    );
    println!("2h.2: wall occludes, window reveals, hulk picks as entity 8 (real UltiCa sprites)");
}

// ---------------------------------------------------------------------
// 2h.3 — lighting bake + shared VisibilityMask, both renderers
// ---------------------------------------------------------------------

fn median(xs: &mut [f32]) -> f32 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs[xs.len() / 2]
}

#[test]
fn gate_2h3_lighting_visibility_contract() {
    let _guard = gpu_lock();
    let Some((device, queue, _name)) = gpu() else {
        loud_skip("2h.3", "no wgpu adapter available");
        return;
    };

    // -- Day/night pair (raster): the same scene at noon and at
    // midnight differs per the Phase 7 environment curve — the
    // tempered bake factor 0.4 + 0.6 x (0.22 x 1.0) = 0.532 with
    // the midnight tint (0.55, 0.65, 0.95) — and only in lighting
    // (the silhouettes are identical: same pixels are background).
    let mut noon_args = base_args();
    noon_args.lighting = Some(LightingScene::fake_house(12.0, lighting::WeatherKind::Clear));
    let (noon, _) = render(&device, &queue, &noon_args);
    let mut mid_args = base_args();
    mid_args.lighting = Some(LightingScene::fake_house(0.0, lighting::WeatherKind::Clear));
    let (mid, _) = render(&device, &queue, &mid_args);

    let bg = noon.get_pixel(2, 2).0;
    let is_bg = |p: [u8; 4]| (0..3).all(|c| (p[c] as i32 - bg[c] as i32).abs() <= 4);
    let mut ratios: [Vec<f32>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let (mut bg_agree, mut bg_total) = (0usize, 0usize);
    for (pn, pm) in noon.pixels().zip(mid.pixels()) {
        bg_total += 1;
        if is_bg(pn.0) == is_bg(pm.0) {
            bg_agree += 1;
        }
        if is_bg(pn.0) || is_bg(pm.0) {
            continue;
        }
        for c in 0..3 {
            if pn.0[c] >= 40 {
                ratios[c].push(pm.0[c] as f32 / pn.0[c] as f32);
            }
        }
    }
    assert!(
        bg_agree as f64 / bg_total as f64 > 0.999,
        "day/night silhouettes differ — the pair must differ only in lighting"
    );
    // Encoded-space expectation: linear ratio (0.532 x tint)
    // through the sRGB encode, ^(1/2.2).
    let expected = [
        (0.532f32 * 0.55).powf(1.0 / 2.2),
        (0.532f32 * 0.65).powf(1.0 / 2.2),
        (0.532f32 * 0.95).powf(1.0 / 2.2),
    ];
    for c in 0..3 {
        assert!(ratios[c].len() > 500, "too few lit pixels in channel {c}");
        let m = median(&mut ratios[c]);
        println!(
            "2h.3 day/night channel {c}: median ratio {m:.3} (expected ~{:.3})",
            expected[c]
        );
        assert!(
            (m - expected[c]).abs() <= 0.13,
            "day/night ratio off the Phase 7 curve in channel {c}: {m:.3} vs {:.3}",
            expected[c]
        );
    }

    // -- BLANK + MEMORIZED, both renderers.
    let blank_tiles: Vec<[i32; 3]> = (5..=7)
        .flat_map(|x| (5..=7).map(move |y| [x, y, 0]))
        .collect();
    // Memorized tiles: a run of the north wall. The per-vertex
    // bake expresses presentation at mesh vertices (Phase 7 §12's
    // prescribed wiring), so the marked region must cover the
    // vertices that shade the sampled patch: the wall run's
    // vertices sample the wall tiles themselves. (A patch in the
    // middle of the greedy-merged floor mega-quad has no
    // vertices of its own — its shading interpolates the quad
    // corners. That resolution limit of the prescribed wiring is
    // recorded in docs/phase2h-integration.md.)
    let memo_tiles: Vec<[i32; 3]> = (4..=7).map(|x| [x, 0, 0]).collect();
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let mut plain = base_args();
        plain.renderer = renderer;
        plain.lighting = Some(LightingScene::fake_house(12.0, lighting::WeatherKind::Clear));
        let (plain_img, _) = render(&device, &queue, &plain);
        let cam = cam_for(&plain);

        // BLANK floor patch: present in the unmarked frame, gone
        // (background shows through) in the marked frame — the
        // shared VisibilityMask excludes it from meshing AND from
        // the ray tree.
        let mut marked = plain.clone();
        marked.lighting = Some(LightingScene::fake_house_marked(
            12.0,
            lighting::WeatherKind::Clear,
            &blank_tiles,
            &[],
        ));
        let (marked_img, _) = render(&device, &queue, &marked);
        let (px, py) = project(
            &cam,
            ui_share::picking::tile_center_world((6, 6, 0)),
            SIZE,
        );
        let plain_patch = patch_mean(&plain_img, px, py, 3);
        let marked_patch = patch_mean(&marked_img, px, py, 3);
        let bg_mean = patch_mean(&marked_img, 4.0, 4.0, 3);
        // Distance in RGB space, not brightness: the warm wood
        // floor and the sky clear color have similar luminance
        // but very different hues (measured distance ~45 vs the
        // marked patch's ~15, which is the hole's rim faces).
        let dist = |a: [f32; 3], b: [f32; 3]| {
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };
        let d_plain = dist(plain_patch, bg_mean);
        let d_marked = dist(marked_patch, bg_mean);
        println!(
            "2h.3 {renderer:?} BLANK patch: plain {plain_patch:?} (dist {d_plain:.1}), marked {marked_patch:?} (dist {d_marked:.1}), bg {bg_mean:?}"
        );
        assert!(
            d_plain > 25.0,
            "{renderer:?}: floor patch should be clearly present unmarked (dist {d_plain:.1})"
        );
        assert!(
            d_marked < d_plain * 0.5,
            "{renderer:?}: BLANK tiles must not render (marked dist {d_marked:.1} vs plain {d_plain:.1})"
        );

        // MEMORIZED wall tiles: geometry stays, presentation is
        // the pinned desaturation (saturation collapses toward
        // 0.30 x the visible saturation).
        let mut memo = plain.clone();
        memo.lighting = Some(LightingScene::fake_house_marked(
            12.0,
            lighting::WeatherKind::Clear,
            &[],
            &memo_tiles,
        ));
        let (memo_img, _) = render(&device, &queue, &memo);
        let (wx, wy) = project(
            &cam,
            ui_share::picking::tile_center_world((5, 0, 0)),
            SIZE,
        );
        let vis_patch = patch_mean(&plain_img, wx, wy, 4);
        let memo_patch = patch_mean(&memo_img, wx, wy, 4);
        let (sv, sm) = (saturation(vis_patch), saturation(memo_patch));
        println!(
            "2h.3 {renderer:?} MEMORIZED patch: visible {vis_patch:?} (sat {sv:.3}), \
             memorized {memo_patch:?} (sat {sm:.3})"
        );
        assert!(sv > 0.03, "{renderer:?}: wall patch too grey to judge desaturation (sat {sv:.3})");
        assert!(
            sm < sv * 0.55,
            "{renderer:?}: MEMORIZED saturation must collapse (pinned 0.30 keep): {sm:.3} vs visible {sv:.3}"
        );
        if renderer == RendererKind::Raster {
            // The bake also dims: the patch mixes the marked
            // run's vertices (pinned 0.45 presentation) with
            // interpolation toward the unmarked run, so the
            // measured ratio (0.81 on lavapipe) sits between the
            // pinned factor and 1.0 — the band asserts a real
            // dimming that has not collapsed to the raw factor.
            let ratio = brightness(memo_patch) / brightness(vis_patch).max(1.0);
            println!("2h.3 raster MEMORIZED brightness ratio {ratio:.3}");
            assert!(
                (0.5..=0.95).contains(&ratio),
                "MEMORIZED brightness ratio {ratio:.3} outside the expected dimming band"
            );
        }
    }
    println!("2h.3: day/night on the Phase 7 curve; BLANK absent and MEMORIZED desaturated in both renderers");
}

// ---------------------------------------------------------------------
// 2h.4 — UI quad last + input round-trip + gate 4.12 in-loop
// ---------------------------------------------------------------------

#[test]
fn gate_2h4_ui_composite_input_roundtrip_and_latency() {
    // -- CPU half: the click round-trip (no GPU needed).
    // A router over a fullscreen terrain rect, a camera centered
    // on the roof tile (6, 6, 1): the center click picks exactly
    // that tile through the DDA.
    let router = ui_share::input::InputRouter::new(
        proto::UiMode::MapInput,
        Some(ui_share::TerrainRect {
            x: 0,
            y: 0,
            w: 320,
            h: 200,
        }),
    );
    // Steep, close camera: at pitch 45 / distance 28 the center
    // ray enters the roof slab through tile (7, 5, 1) first
    // (it crosses the slab's top plane at tile (7.57, 5.43)) and
    // the DDA correctly reports that tile. At pitch 85 /
    // distance 12 the ray crosses the top plane at (6.59, 6.41)
    // — inside the target tile — so (6, 6, 1) is the analytic
    // first hit, not just the implementation's answer.
    let cam = ui_share::picking::PickCamera {
        target: ui_share::picking::tile_center_world((6, 6, 1)),
        yaw_deg: 45.0,
        pitch_deg: 85.0,
        distance: 12.0,
        fov_y_deg: 60.0,
    };
    let occupied = frame::fake_house_occupied();
    let pick = ui_share::picking::pick_from_ui_click(1, &router, &cam, 160, 100, &occupied)
        .expect("center click is a world pick");
    assert!(pick.hit, "center click must hit");
    let pos = pick.pos.expect("hit carries a tile");
    assert_eq!(
        (pos.x, pos.y, pos.z),
        (6, 6, 1),
        "synthetic click must round-trip to the expected tile"
    );
    // A panel click (outside the terrain rect) is not a world pick.
    let panel_router = ui_share::input::InputRouter::new(
        proto::UiMode::MapInput,
        Some(ui_share::TerrainRect {
            x: 0,
            y: 0,
            w: 213,
            h: 200,
        }),
    );
    assert!(
        ui_share::picking::pick_from_ui_click(2, &panel_router, &cam, 300, 100, &occupied)
            .is_none(),
        "panel click must not produce a world pick"
    );
    // The input the click produces survives the FlatBuffers
    // envelope round-trip unchanged.
    let input = ui_share::input::key_to_input(
        ui_share::input::EngineKey::Char('i'),
        ui_share::input::Modifiers::default(),
        true,
    );
    let bytes = proto::encode_message(&proto::MessageT::Input(Box::new(input.clone())));
    match proto::decode_message(&bytes).expect("decode input envelope") {
        proto::MessageT::Input(back) => {
            assert_eq!(back.kind, input.kind);
            assert_eq!(back.key_code, input.key_code);
        }
        other => panic!("envelope round-trip changed the message: {other:?}"),
    }
    println!("2h.4 CPU: click -> tile (6, 6, 1); panel click separated; envelope round-trips");

    // -- GPU half: compositing + the 4.12 in-loop measurement.
    let _guard = gpu_lock();
    let Some((device, queue, adapter_name)) = gpu() else {
        loud_skip("2h.4 (GPU half)", "no wgpu adapter available");
        return;
    };
    let mut args = composed_args();
    args.lighting = None;
    let (img, outcome) = render(&device, &queue, &args);
    assert_eq!(outcome.stats.passes.last(), Some(&FramePass::Ui), "UI must be the last pass");

    // The sidebar pixel equals the prepared tight pixel it was
    // stretched from (UI x 300, y 100 -> frame x 600, y 180 at
    // exactly 2x / 1.8x stretch).
    let ui = args.ui.as_ref().unwrap();
    let tight_px = {
        let i = ((100 * ui.size.0 + 300) * 4) as usize;
        [ui.tight[i], ui.tight[i + 1], ui.tight[i + 2], ui.tight[i + 3]]
    };
    assert_eq!(tight_px[3], 255, "sidebar sample must be opaque UI");
    let frame_px = img.get_pixel(600, 180).0;
    for c in 0..3 {
        assert!(
            (frame_px[c] as i32 - tight_px[c] as i32).abs() <= 2,
            "UI must composite verbatim over the post-processed world: frame {frame_px:?} vs tight {tight_px:?}"
        );
    }

    // Masking comes from the frame itself: paint the store's
    // terrain rect opaque red — the composed frame must still
    // show the world there, matching the no-UI frame.
    let (store2, note2) = frame::stub_ui_store(320, 200, true).expect("opaque-terrain store");
    let mut args2 = args.clone();
    args2.ui = Some(frame::prepare_ui(&store2, &note2).expect("prepare opaque ui"));
    let prepared2 = args2.ui.as_ref().unwrap();
    let rect = prepared2.terrain_rect.expect("stub frame reports a terrain rect");
    let ci = (((rect.y + 5) as u32 * prepared2.size.0 + (rect.x + 5) as u32) * 4 + 3) as usize;
    assert_eq!(prepared2.tight[ci], 0, "terrain rect must be masked in the prepared frame");
    let (img2, _) = render(&device, &queue, &args2);
    let mut args3 = args.clone();
    args3.ui = None;
    let (img_world, _) = render(&device, &queue, &args3);
    // Terrain center in frame pixels (UI rect center stretched).
    let (fx, fy) = (200u32, 180u32);
    let p2 = img2.get_pixel(fx, fy).0;
    let pw = img_world.get_pixel(fx, fy).0;
    assert!(
        !(p2[0] > 150 && p2[1] < 80 && p2[2] < 80),
        "the store's red terrain paint must NOT appear (masking failed): {p2:?}"
    );
    for c in 0..3 {
        assert!(
            (p2[c] as i32 - pw[c] as i32).abs() <= CHANNEL_TOL as i32,
            "masked terrain must show the world itself: ui {p2:?} vs world {pw:?}"
        );
    }

    // Gate 4.12, measured in-loop: the UI stage (upload +
    // composite + GPU wait) across 30 composed frames, against
    // the Phase 4 harness-segment budgets (median < 50 ms,
    // p99 < 100 ms).
    let mut samples_ms = Vec::new();
    for _ in 0..30 {
        let outcome = frame::render_frame(&device, &queue, &args).expect("latency frame");
        samples_ms.push(outcome.stats.ui_stage.as_secs_f64() * 1000.0);
    }
    let report = ui_share::latency::LatencyReport::from_samples(samples_ms, &adapter_name);
    println!(
        "2h.4 gate 4.12 in-loop ({adapter_name}): median {:.2} ms, p99 {:.2} ms over {} frames",
        report.median_ms,
        report.p99_ms,
        report.samples_ms.len()
    );
    assert!(
        report.median_ms < 50.0 && report.p99_ms < 100.0,
        "gate 4.12 budgets exceeded in-loop: median {:.2} ms (budget 50), p99 {:.2} ms (budget 100)",
        report.median_ms, report.p99_ms
    );
    println!("2h.4: UI last + masked per the frame; 4.12 within budget in-loop");
}

// ---------------------------------------------------------------------
// 2h.5 — the composed golden
// ---------------------------------------------------------------------

#[test]
fn gate_2h5_composed_golden() {
    let _guard = gpu_lock();
    let Some((device, queue, _name)) = gpu() else {
        loud_skip("2h.5", "no wgpu adapter available");
        return;
    };
    // The composed scene: world + fields + billboards + post +
    // stub UI, under the Phase 7 night-interior grid evaluated at
    // noon (the lighting bake is part of the composition).
    let mut args = composed_args();
    args.lighting = Some(LightingScene::fake_house(12.0, lighting::WeatherKind::Clear));
    let (img, outcome) = render(&device, &queue, &args);
    assert_eq!(outcome.stats.passes, FRAME_ORDER);

    let golden_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/house_composed.png");
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        img.save(&golden_path).expect("write composed golden");
        println!("2h.5 UPDATE_GOLDEN: wrote {}", golden_path.display());
        return;
    }
    let golden = image::open(&golden_path)
        .unwrap_or_else(|e| {
            panic!(
                "composed golden missing ({e}) — run UPDATE_GOLDEN=1 cargo test -p engine \
                 --test frame_gate if this scene is new"
            )
        })
        .to_rgba8();
    assert_within_tol("2h.5 composed golden", &golden, &img);
    println!("2h.5: composed frame matches the golden within the 2g.3 tolerance");
}


