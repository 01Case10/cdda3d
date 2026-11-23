//! Phase 10 slice 10.3 gates (raster half) — docs/phase10-sky-gi.md
//! §8 gates 10.3a / 10.3b, raster side, plus the §5.2 per-tile
//! sky-factor spike (decision 4A).
//!
//! - Spike (CPU): sky_factor rides the vertex colour's alpha.
//!   (a) the greedy mesher's merge key is the whole `Block`
//!   (colour included), so two adjacent blocks differing only in
//!   alpha do NOT merge; (b) the lit fake-house preparation writes
//!   alpha 0 for sheltered (interior) tiles and 1 for exposed
//!   tiles through BOTH bakers, and writes nothing (all 1.0) when
//!   the scene carries no exposure data.
//! - 10.3a (GPU): a sky-only raster render (Renderer with an empty
//!   mesh list — the fullscreen sky pass alone) at a named
//!   SkyState agrees with the freeze CPU mirror `sky_color_cpu`
//!   sampled at the same per-pixel directions, within sRGB ±2 per
//!   channel. (Raster-vs-RT parity completes in 10.4; this half
//!   proves raster == the shared function.)
//! - 10.3b (GPU): fake house, cutaway 0/1. Interior lamp-room
//!   patch at noon == the neutral-default render (sky GI does not
//!   reach sheltered tiles); exterior roof patch brightens at
//!   noon vs midnight; a BLANK region renders as sky only.
//!
//! GPU tests skip LOUDLY when no wgpu adapter exists (the
//! golden_gate convention): a skip proves nothing.

use std::sync::Mutex;

use engine::capture::{CaptureArgs, RendererKind};
use engine::frame::{self, LightingScene, SkyScene};
use engine::render::Renderer;
use glam::Vec3;
use lighting::{sky_color_cpu, GameEnvironment, SkyUniforms, WeatherKind};
use mesher::block::{block_for_tile, TILE_GROUND};
use mesher::chunk::ChunkMap;
use mesher::scale::WorldScale;

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn loud_skip(gate: &str, reason: &str) {
    let banner = format!(
        "================================================================\n\
         {gate}: NOT RUN — {reason}\n\
         This run proves NOTHING about slice 10.3. The gate only\n\
         counts on a machine with a wgpu adapter (lavapipe counts).\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
}

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
        let name = adapter.get_info().name.clone();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("sky-raster-gate-device"),
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

// ---------------------------------------------------------------------
// Spike (CPU, no GPU needed)
// ---------------------------------------------------------------------

/// Two adjacent ground blocks in one chunk, meshed greedy.
/// Returns the opaque mesh's quad count.
fn two_block_quad_count(alpha_a: f32, alpha_b: f32) -> usize {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    {
        let chunk = map.add_chunk([0, 0], 1);
        for (x, a) in [(0usize, alpha_a), (1usize, alpha_b)] {
            let mut b = block_for_tile(TILE_GROUND);
            b.color[3] = a;
            // One full block cell each (grid cells are blocks).
            chunk.grid.set(x, 0, 2, b);
        }
    }
    map.clear_dirty();
    let pm = map.mesh_chunk([0, 0], true).expect("chunk meshes");
    let opaque = &pm.meshes[mesher::block::Pass::Opaque.index()];
    opaque.vertex_count() / 4
}

#[test]
fn spike_alpha_difference_breaks_greedy_merge() {
    // Same alpha: the two top faces merge into ONE quad (plus the
    // merged sides) — strictly fewer quads than the split case.
    let merged = two_block_quad_count(1.0, 1.0);
    // Different alpha (sky_factor 1 vs 0): no face may merge
    // across the boundary.
    let split = two_block_quad_count(1.0, 0.0);
    println!("spike: merged-alpha quads {merged}, split-alpha quads {split}");
    assert!(
        split > merged,
        "alpha difference must break greedy merges (merged {merged}, split {split})"
    );
}

#[test]
fn spike_lit_prep_writes_sky_factor_alpha() {
    let visible = engine::cutaway::CutawayMode::HideAbove(0).visible_levels(1);
    // PerVertex (default fake-house bake mode).
    let lit = LightingScene::fake_house(12.0, WeatherKind::Clear);
    let (meshes, _stats, _spec) =
        engine::scene_prep::prepare_raster_house_lit_v(&visible, None, &lit, None)
            .expect("lit prep");
    let opaque = &meshes.meshes[mesher::block::Pass::Opaque.index()];
    let zeros = opaque.colors.iter().filter(|c| c[3] == 0.0).count();
    let ones = opaque.colors.iter().filter(|c| c[3] == 1.0).count();
    println!(
        "spike per-vertex: opaque verts alpha0 {zeros}, alpha1 {ones}, total {}",
        opaque.colors.len()
    );
    assert!(
        zeros > 0,
        "sheltered (interior) vertices must carry alpha 0"
    );
    assert!(ones > 0, "exposed vertices must carry alpha 1");

    // PerBlock baker path: same contract through the chunk-map bake.
    let mut lit_pb = LightingScene::fake_house(12.0, WeatherKind::Clear);
    lit_pb.bake_mode = frame::BakeMode::PerBlock;
    let (meshes, _s, _sp) =
        engine::scene_prep::prepare_raster_house_lit_v(&visible, None, &lit_pb, None)
            .expect("lit prep per-block");
    let opaque = &meshes.meshes[mesher::block::Pass::Opaque.index()];
    let zeros = opaque.colors.iter().filter(|c| c[3] == 0.0).count();
    let ones = opaque.colors.iter().filter(|c| c[3] == 1.0).count();
    println!("spike per-block: opaque verts alpha0 {zeros}, alpha1 {ones}");
    assert!(
        zeros > 0,
        "per-block: sheltered vertices must carry alpha 0"
    );
    assert!(ones > 0, "per-block: exposed vertices must carry alpha 1");

    // No exposure data -> alpha untouched (all 1.0): the global
    // (4B-shaped) fallback is a data absence, never a silent
    // rewrite.
    let mut lit_none = LightingScene::fake_house(12.0, WeatherKind::Clear);
    lit_none.sky_exposure = None;
    let (meshes, _s, _sp) =
        engine::scene_prep::prepare_raster_house_lit_v(&visible, None, &lit_none, None)
            .expect("lit prep no-exposure");
    let opaque = &meshes.meshes[mesher::block::Pass::Opaque.index()];
    assert!(
        opaque.colors.iter().all(|c| c[3] == 1.0),
        "without exposure data every opaque vertex alpha must stay 1.0"
    );
}

/// Shader-compile gate (runs everywhere, no adapter needed):
/// both composed WGSL sources — the world body with the shared
/// sky WGSL included, and the sky-pass body — parse and
/// validate under naga 24 (wgpu 24's own shader frontend, the
/// same translator a real adapter run would use). This is the
/// check that keeps a WGSL typo from hiding behind the GPU
/// gates' loud skip on adapter-less machines.
#[test]
fn wgsl_sources_validate_with_naga() {
    for (name, src) in [
        ("world", engine::render::world_shader_source()),
        ("sky", engine::render::sky_shader_source()),
    ] {
        let module = naga::front::wgsl::parse_str(&src)
            .unwrap_or_else(|e| panic!("{name} WGSL parse failed:\n{}", e.emit_to_string(&src)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|e| panic!("{name} WGSL validation failed: {e:?}"));
        println!(
            "wgsl: {name} source parses + validates ({} bytes)",
            src.len()
        );
    }
    // The world body must bind the SHARED struct (group 2), and
    // the sky body must call the shared function — inclusion,
    // not a copy.
    let world = engine::render::world_shader_source();
    assert!(
        world.contains("struct SkyUniforms"),
        "world shader lacks the shared SkyUniforms"
    );
    assert!(world.contains("@group(2) @binding(0) var<uniform> sky: SkyUniforms;"));
    let sky = engine::render::sky_shader_source();
    assert!(
        sky.contains("sky_color(rd, rd.y, sky)"),
        "sky pass must call the shared sky_color"
    );
}

/// §6 neutral calibration proof (CPU, runs everywhere): the GI
/// form of the neutral block the world pipelines bind must
/// reproduce the legacy raster shade `0.45 + 0.55·ndl` (light
/// dir normalize(0.3, 0.8, 0.4)) EXACTLY for every normal, so
/// default-frame geometry shading is byte-identical to
/// pre-Phase-10. And a non-neutral frame block passes through
/// to the GI binding verbatim (no second calibration).
#[test]
fn neutral_gi_calibration_matches_legacy_shade() {
    let l = {
        let v = [0.3f32, 0.8, 0.4];
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / len, v[1] / len, v[2] / len]
    };
    let gi = engine::render::gi_uniforms_for(&SkyUniforms::neutral());
    let normals: [[f32; 3]; 7] = [
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
        [0.2673, 0.8018, 0.5345],
    ];
    for n in normals {
        // The §5.2 sum at sky_factor 1, evaluated from the GI
        // block exactly as the world shader computes it.
        let t = n[1] * 0.5 + 0.5;
        let mut sum = [0.0f32; 3];
        for c in 0..3 {
            let hemi = (gi.ground3()[c] * (1.0 - t) + gi.zenith3()[c] * t) * gi.ambient_scale();
            let sun = gi.sun_color3()[c]
                * (n[0] * gi.sun_dir3()[0] + n[1] * gi.sun_dir3()[1] + n[2] * gi.sun_dir3()[2])
                    .max(0.0);
            let moon = gi.moon_color3()[c]
                * (n[0] * gi.moon_dir3()[0] + n[1] * gi.moon_dir3()[1] + n[2] * gi.moon_dir3()[2])
                    .max(0.0);
            sum[c] = hemi + sun + moon;
        }
        let legacy = 0.45 + 0.55 * (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
        for c in 0..3 {
            assert!(
                (sum[c] - legacy).abs() < 1e-6,
                "normal {n:?} channel {c}: GI sum {} != legacy shade {legacy}",
                sum[c]
            );
        }
    }
    // Non-neutral: verbatim pass-through.
    let frame = SkyScene::fake_day(12.0, 4).uniforms();
    assert_ne!(frame.as_rows(), SkyUniforms::neutral().as_rows());
    assert_eq!(
        engine::render::gi_uniforms_for(&frame).as_rows(),
        frame.as_rows()
    );
}

// ---------------------------------------------------------------------
// 10.3a — sky-only raster render vs the CPU mirror
// ---------------------------------------------------------------------

fn srgb_encode(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

#[test]
fn gate_10_3a_sky_pass_matches_cpu_mirror() {
    let _guard = gpu_lock();
    let Some((device, queue, adapter)) = gpu() else {
        loud_skip("10.3a", "no wgpu adapter on this machine");
        return;
    };
    println!("10.3a: adapter {adapter}");

    let size = (320u32, 180u32);
    // Named states: clear noon and clear midnight (fake-day path).
    let cases: Vec<(&str, SkyUniforms, f32)> = [12.0f32, 0.0]
        .iter()
        .map(|&hour| {
            let scene = SkyScene::fake_day(hour, 4);
            let name = if hour == 12.0 { "noon" } else { "midnight" };
            (name, scene.uniforms(), hour)
        })
        .collect();

    for (name, uniforms, _hour) in &cases {
        let mut renderer = Renderer::new(&device, engine::render::CAPTURE_FORMAT);
        let mut camera = engine::camera::OrbitCamera::new(Vec3::new(6.0, 1.0, -6.0));
        camera.yaw_deg = 45.0;
        camera.pitch_deg = 45.0;
        camera.distance = 28.0;
        camera.clamp();
        let aspect = size.0 as f32 / size.1 as f32;
        let vp = camera.view_projection(aspect);
        renderer.set_view_projection(&queue, vp);
        renderer.set_sky(&queue, uniforms, vp.inverse(), size);

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sky-gate-color"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: engine::render::CAPTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&Default::default());
        let (_dt, depth_view) = engine::render::create_depth_texture(&device, size);
        // Empty mesh list: the sky pass alone fills the frame.
        renderer.draw(&device, &queue, &[], &color_view, &depth_view);
        let (img, _distinct) =
            frame::readback_rgba(&device, &queue, &color_texture, size).expect("readback");

        // Per-pixel expected colour from the CPU mirror, with the
        // ray reconstructed exactly as the shader does (unproject
        // the pixel's NDC through the inverse view-projection).
        let inv = vp.inverse();
        let eye = camera.eye();
        let mut over = 0usize;
        let mut max_delta = 0u32;
        let mut err_sum = 0f64;
        let total = (size.0 * size.1) as usize;
        for y in 0..size.1 {
            for x in 0..size.0 {
                let ndc = glam::Vec2::new(
                    ((x as f32 + 0.5) / size.0 as f32) * 2.0 - 1.0,
                    1.0 - ((y as f32 + 0.5) / size.1 as f32) * 2.0,
                );
                let far4 = inv * glam::Vec4::new(ndc.x, ndc.y, 1.0, 1.0);
                let far = far4.truncate() / far4.w;
                let rd = (far - eye).normalize();
                let expect = sky_color_cpu(rd.y, rd.to_array(), uniforms);
                let got = img.get_pixel(x, y).0;
                for c in 0..3 {
                    let e = srgb_encode(expect[c]);
                    let d = (got[c] as i32 - e as i32).unsigned_abs();
                    max_delta = max_delta.max(d);
                    err_sum += d as f64;
                    if d > 2 {
                        over += 1;
                    }
                }
            }
        }
        let over_frac = over as f64 / (total * 3) as f64;
        let mean = err_sum / (total * 3) as f64;
        println!(
            "10.3a {name}: channel samples over ±2: {over} ({:.4}%), max delta {max_delta}, mean {mean:.3}",
            over_frac * 100.0
        );
        // Star-cell edges at midnight can flip a hash cell on a
        // sub-texel direction difference; the budget allows a
        // small fraction of channel samples out of tolerance but
        // the mean must stay tiny.
        assert!(
            over_frac <= 0.005 && mean <= 0.5,
            "10.3a {name}: raster sky diverges from the CPU mirror \
             (over {over}, max {max_delta}, mean {mean:.3})"
        );
    }
}

// ---------------------------------------------------------------------
// 10.3b — ceiling rule, raster half
// ---------------------------------------------------------------------

fn base_args() -> CaptureArgs {
    CaptureArgs {
        size: (640, 360),
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        cutaway_z: 0,
        renderer: RendererKind::Raster,
        ..CaptureArgs::default()
    }
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, args: &CaptureArgs) -> image::RgbaImage {
    let outcome = frame::render_frame(device, queue, args).expect("render_frame");
    let (img, _d) =
        frame::readback_rgba(device, queue, &outcome.texture, outcome.size).expect("readback");
    img
}

fn cam_for(args: &CaptureArgs) -> engine::camera::OrbitCamera {
    let mut camera = engine::camera::OrbitCamera::new(Vec3::new(
        args.camera_target.0,
        1.0,
        -args.camera_target.1,
    ));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    camera
}

fn project(cam: &engine::camera::OrbitCamera, p: Vec3, size: (u32, u32)) -> (f32, f32) {
    let vp = cam.view_projection(size.0 as f32 / size.1 as f32);
    let clip = vp * p.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    (
        (ndc.x * 0.5 + 0.5) * size.0 as f32,
        (0.5 - ndc.y * 0.5) * size.1 as f32,
    )
}

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

fn brightness(m: [f32; 3]) -> f32 {
    (m[0] + m[1] + m[2]) / 3.0
}

fn mean_abs_diff(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) / 3.0
}

#[test]
fn gate_10_3b_ceiling_rule_raster() {
    let _guard = gpu_lock();
    let Some((device, queue, adapter)) = gpu() else {
        loud_skip("10.3b", "no wgpu adapter on this machine");
        return;
    };
    println!("10.3b: adapter {adapter}");
    let lit = || LightingScene::fake_house(12.0, WeatherKind::Clear);

    // Interior: neutral-default vs noon sky — the lamp room must
    // not move (sky GI is scaled by the per-tile factor, which is
    // 0 inside).
    let mut args_neutral = base_args();
    args_neutral.lighting = Some(lit());
    let img_neutral = render(&device, &queue, &args_neutral);
    let mut args_noon = base_args();
    args_noon.lighting = Some(lit());
    args_noon.sky = Some(SkyScene::fake_day(12.0, 4));
    let img_noon = render(&device, &queue, &args_noon);
    let cam = cam_for(&args_noon);
    let (fx, fy) = project(
        &cam,
        ui_share::picking::tile_center_world((6, 6, 0)),
        args_noon.size,
    );
    let in_neutral = patch_mean(&img_neutral, fx, fy, 3);
    let in_noon = patch_mean(&img_noon, fx, fy, 3);
    let d_interior = mean_abs_diff(in_neutral, in_noon);
    println!(
        "10.3b interior: neutral {in_neutral:?} vs noon {in_noon:?} (mean |diff| {d_interior:.2})"
    );
    assert!(
        d_interior <= 2.0,
        "interior lamp room must not move under a noon sky (mean |diff| {d_interior:.2})"
    );

    // Exterior: roof patch (cutaway 1 so the roof is present),
    // noon vs midnight.
    let mut args_noon_r = base_args();
    args_noon_r.cutaway_z = 1;
    args_noon_r.lighting = Some(lit());
    args_noon_r.sky = Some(SkyScene::fake_day(12.0, 4));
    let img_noon_r = render(&device, &queue, &args_noon_r);
    let mut args_mid_r = base_args();
    args_mid_r.cutaway_z = 1;
    args_mid_r.lighting = Some(lit());
    args_mid_r.sky = Some(SkyScene::fake_day(0.0, 4));
    let img_mid_r = render(&device, &queue, &args_mid_r);
    let cam_r = cam_for(&args_noon_r);
    let (rx, ry) = project(
        &cam_r,
        ui_share::picking::tile_center_world((6, 6, 1)),
        args_noon_r.size,
    );
    let roof_noon = patch_mean(&img_noon_r, rx, ry, 3);
    let roof_mid = patch_mean(&img_mid_r, rx, ry, 3);
    println!(
        "10.3b roof: noon {roof_noon:?} ({:.1}) vs midnight {roof_mid:?} ({:.1})",
        brightness(roof_noon),
        brightness(roof_mid)
    );
    assert!(
        brightness(roof_noon) > brightness(roof_mid) + 25.0,
        "exterior must brighten at noon vs midnight (noon {:.1}, midnight {:.1})",
        brightness(roof_noon),
        brightness(roof_mid)
    );

    // BLANK region renders as sky only: blank the centre floor
    // tiles; the patch through the hole must move strongly toward
    // the background and away from the floor colour.
    let blank: Vec<[i32; 3]> = (5..=7)
        .flat_map(|x| (5..=7).map(move |y| [x, y, 0]))
        .collect();
    let mut args_blank = base_args();
    args_blank.lighting = Some(LightingScene::fake_house_marked(
        12.0,
        WeatherKind::Clear,
        &blank,
        &[],
    ));
    args_blank.sky = Some(SkyScene::fake_day(12.0, 4));
    let img_blank = render(&device, &queue, &args_blank);
    let blank_patch = patch_mean(&img_blank, fx, fy, 3);
    let bg_patch = patch_mean(&img_blank, 4.0, 4.0, 3);
    let dist = |a: [f32; 3], b: [f32; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let d_floor = dist(blank_patch, in_noon);
    let d_bg = dist(blank_patch, bg_patch);
    println!("10.3b BLANK: patch {blank_patch:?} (dist to floor {d_floor:.1}, to bg {d_bg:.1})");
    assert!(
        d_floor > 25.0,
        "BLANK patch must not show floor geometry (dist {d_floor:.1})"
    );
    assert!(
        d_bg < d_floor,
        "BLANK patch must read as background/sky (bg dist {d_bg:.1} vs floor dist {d_floor:.1})"
    );

    // The GameEnvironment plumbing itself: a scene built from the
    // proto-shaped environment (neutral) evaluates without panic
    // and produces finite uniforms.
    let env_scene = SkyScene::new(GameEnvironment::neutral());
    let u = env_scene.uniforms();
    assert!(u.as_rows().iter().flatten().all(|v| v.is_finite()));
}
