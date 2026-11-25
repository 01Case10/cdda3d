//! Phase 10 slice 10.4, gate 10.4a (the RT half of 10.3b — the
//! ceiling rule, docs/phase10-sky-gi.md §3.4/§8): on the lit fake
//! house, sky GI must brighten the exposed exterior (the roof, in
//! the full view) at noon and darken it at midnight, while the
//! roofed interior lamp room (cutaway view) stays within noise of
//! the neutral-environment render; BLANK (never-seen) tiles must
//! never produce geometry. Plus the engine-side sky-exposure
//! feed: interior cells roofed, roof cells exposed
//! (world_for_levels_lit).

use engine::capture::CaptureArgs;
use engine::frame::LightingScene;
use engine::rt::{capture_rt_with_device_sky, capture_visible_levels, world_for_levels_lit};
use lighting::{sky_state, GameEnvironment, SkyUniforms, WeatherKind};

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
            label: Some("sky-rt-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some((device, queue, name))
}

fn uniforms_at(time_of_day_sec: f32, moon_phase: u8) -> SkyUniforms {
    let env = GameEnvironment::fake_day(time_of_day_sec, 21_600.0, 64_800.0, 1, moon_phase);
    // RT consumes grid space (east, south, up); from_sky_state
    // emits SkyState space (east, north, up) — convert.
    SkyUniforms::from_sky_state(&sky_state(&env), time_of_day_sec).to_grid_space()
}

fn house_args(lit: LightingScene, cutaway_z: i32) -> CaptureArgs {
    CaptureArgs {
        snapshot: "stub-house".to_string(),
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        size: (320, 240),
        cutaway_z,
        lighting: Some(lit),
        ..CaptureArgs::default()
    }
}

/// Unproject a hit pixel to its game tile (grid coords = tile
/// x/y; game z from block z at 3 blocks per level, min z 0).
fn hit_tile(args: &CaptureArgs, depth: &[f32], max_dist: f32, size: (u32, u32), x: u32, y: u32) -> Option<[i32; 3]> {
    let d = depth[(y * size.0 + x) as usize];
    if d >= max_dist {
        return None;
    }
    let camera = engine::rt::capture_camera(args, size.0 as f32 / size.1 as f32);
    let eye = camera.eye;
    let fwd = (camera.target - eye).normalize_or_zero();
    let right = fwd.cross(glam::Vec3::Z).normalize_or_zero();
    let up = right.cross(fwd);
    let tan = (camera.fov_y_deg.to_radians() * 0.5).tan();
    let ndc = glam::Vec2::new(
        ((x as f32 + 0.5) / size.0 as f32) * 2.0 - 1.0,
        1.0 - ((y as f32 + 0.5) / size.1 as f32) * 2.0,
    );
    let rd = (fwd + ndc.x * tan * camera.aspect * right + ndc.y * tan * up).normalize();
    let p = eye + rd * d;
    Some([p.x.floor() as i32, p.y.floor() as i32, (p.z.floor() as i32) / 3])
}

/// [`hit_tile`] for the BLANK check: a hit counts only if the
/// unprojected point sits at least 0.02 tiles inside the tile —
/// hits on a boundary plane between a blank tile and its
/// neighbour are float-ambiguous (both signs of the error were
/// observed: 14 false hits un-nudged, 116 with a ray pull-back)
/// and prove nothing about geometry leaking.
fn hit_tile_strict(args: &CaptureArgs, depth: &[f32], max_dist: f32, size: (u32, u32), x: u32, y: u32) -> Option<[i32; 3]> {
    let d = depth[(y * size.0 + x) as usize];
    if d >= max_dist {
        return None;
    }
    let camera = engine::rt::capture_camera(args, size.0 as f32 / size.1 as f32);
    let eye = camera.eye;
    let fwd = (camera.target - eye).normalize_or_zero();
    let right = fwd.cross(glam::Vec3::Z).normalize_or_zero();
    let up = right.cross(fwd);
    let tan = (camera.fov_y_deg.to_radians() * 0.5).tan();
    let ndc = glam::Vec2::new(
        ((x as f32 + 0.5) / size.0 as f32) * 2.0 - 1.0,
        1.0 - ((y as f32 + 0.5) / size.1 as f32) * 2.0,
    );
    let rd = (fwd + ndc.x * tan * camera.aspect * right + ndc.y * tan * up).normalize();
    let p = eye + rd * d;
    let frac = [p.x.fract(), p.y.fract(), (p.z / 3.0).fract()];
    if frac.iter().any(|f| *f < 0.02 || *f > 0.98) {
        return None; // boundary-ambiguous
    }
    Some([p.x.floor() as i32, p.y.floor() as i32, (p.z.floor() as i32) / 3])
}

#[derive(Default)]
struct Bucket {
    n: usize,
    lum: f64,
    diff: f64,
}

impl Bucket {
    fn mean_lum(&self) -> f64 {
        if self.n > 0 { self.lum / self.n as f64 } else { 0.0 }
    }
    fn mean_diff(&self) -> f64 {
        if self.n > 0 { self.diff / self.n as f64 } else { 0.0 }
    }
}

/// Split hit pixels into interior (footprint 1..=10 x 1..=10 at
/// game z 0), roof (game z 1), and other; report mean luminance
/// and mean abs diff vs a reference capture.
fn region_stats(
    args: &CaptureArgs,
    cap: &engine::rt::RtCapture,
    reference: Option<&engine::rt::RtCapture>,
) -> (Bucket, Bucket, Bucket) {
    let size = cap.size;
    let (mut interior, mut roof, mut other) = (Bucket::default(), Bucket::default(), Bucket::default());
    for y in 0..size.1 {
        for x in 0..size.0 {
            let Some(tile) = hit_tile(args, &cap.depth, cap.max_dist, size, x, y) else {
                continue;
            };
            let p = cap.image.get_pixel(x, y).0;
            let lum = (0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64) / 255.0;
            let diff = match reference {
                Some(r) => {
                    let q = r.image.get_pixel(x, y).0;
                    (0..3).map(|c| (p[c] as f64 - q[c] as f64).abs()).sum::<f64>() / (3.0 * 255.0)
                }
                None => 0.0,
            };
            // Roof bucket = the roof LEVEL's cells: side faces
            // unproject to block z in [3, 6) (tile z 1) and the
            // top face to exactly 6 (tile z 2) — both are the
            // sky-exposed roof level.
            let b = if tile[2] == 0 && (1..=10).contains(&tile[0]) && (1..=10).contains(&tile[1]) {
                &mut interior
            } else if tile[2] >= 1 {
                &mut roof
            } else {
                &mut other
            };
            b.n += 1;
            b.lum += lum;
            b.diff += diff;
        }
    }
    (interior, roof, other)
}

/// The exposure feed (CPU, runs everywhere): in the lit fake
/// house world, roofed interior cells are not sky-exposed; the
/// roof itself is.
#[test]
fn sky_exposure_feed_marks_roofed_interior() {
    let lit = LightingScene::fake_house(12.0, WeatherKind::Clear);
    // Cutaway world (level 0 only): every cell in the interior
    // column (6, 6) is roofed in the uncut scene, so none of them
    // may be sky-exposed — even though the roof itself is cut
    // away in this world.
    let cut = world_for_levels_lit(&capture_visible_levels(0), &lit);
    let column: Vec<&treebuild::world::VoxelCell> = cut
        .iter()
        .filter(|(p, _)| p[0] == 6 && p[1] == 6)
        .map(|(_, c)| c)
        .collect();
    assert!(!column.is_empty(), "interior column must have cells");
    assert!(
        column.iter().all(|c| !c.sky_exposed),
        "roofed interior cells must not be sky-exposed"
    );
    // Full world: the roof slab at (6, 6) is the column top —
    // nothing above it, so it is exposed.
    let full = world_for_levels_lit(&capture_visible_levels(1), &lit);
    let roof_z = full
        .iter()
        .filter(|(p, _)| p[0] == 6 && p[1] == 6)
        .map(|(p, _)| p[2])
        .max()
        .expect("roof column has cells");
    assert!(
        full.get([6, 6, roof_z]).unwrap().sky_exposed,
        "roof top must be sky-exposed"
    );
}

#[test]
fn gate_10_4a_ceiling_rule_rt() {
    let Some((device, queue, name)) = gpu() else {
        eprintln!("10.4a: no wgpu adapter — NOT RUN");
        return;
    };
    println!("10.4a adapter: {name}");
    let lit = || LightingScene::fake_house(12.0, WeatherKind::Clear);
    let noon_u = uniforms_at(43_200.0, 0);
    let midnight_u = uniforms_at(0.0, 4);

    // --- Full view (cutaway 1): the exposed roof responds.
    let args_full = house_args(lit(), 1);
    let f_neutral = capture_rt_with_device_sky(&device, &queue, &args_full, None)
        .expect("neutral full RT capture");
    let f_noon = capture_rt_with_device_sky(&device, &queue, &args_full, Some(&noon_u))
        .expect("noon full RT capture");
    let f_mid = capture_rt_with_device_sky(&device, &queue, &args_full, Some(&midnight_u))
        .expect("midnight full RT capture");
    assert!(f_neutral.leaves > 0, "house must render");
    let (_, roof_neut, _) = region_stats(&args_full, &f_neutral, None);
    let (_, roof_noon, _) = region_stats(&args_full, &f_noon, Some(&f_neutral));
    let (_, roof_mid, _) = region_stats(&args_full, &f_mid, Some(&f_neutral));
    println!(
        "10.4a roof: {} px, lum neutral {:.3} -> noon {:.3} -> midnight {:.3}",
        roof_neut.n,
        roof_neut.mean_lum(),
        roof_noon.mean_lum(),
        roof_mid.mean_lum()
    );
    assert!(roof_neut.n > 200, "roof must be visible in the full view ({} px)", roof_neut.n);
    // Neutral IS a noon-class legacy sun (that is its purpose,
    // §6), so the honest noon assertion is stability, not
    // brightening: the SkyState noon must land within a few
    // percent of it on exposed exterior. The brightening the
    // gate exists to prove shows against midnight, below.
    assert!(
        (roof_noon.mean_lum() / roof_neut.mean_lum()) > 0.90
            && (roof_noon.mean_lum() / roof_neut.mean_lum()) < 1.15,
        "exposed roof at SkyState noon must match the neutral noon-class render          ({:.3} -> {:.3})",
        roof_neut.mean_lum(),
        roof_noon.mean_lum()
    );
    // Midnight darkening threshold re-derived for Option B
    // (2026-11-25): the roof level of this night fixture is lit
    // DARK (cap 0.25). Under the pre-B product, midnight moon
    // (~0.21 light) was multiplied by the cap and by AO and read
    // 0.011 vs noon 0.084 (0.13x). Under Option B the cap is a
    // ceiling on the sum, not a multiplier: noon saturates the
    // DARK ceiling and midnight's ~0.21 of moonlight nearly
    // does too, so the rendered ratio compresses to ~0.67
    // (measured 0.048 vs 0.068). The gate's claim — the exposed
    // roof still darkens at midnight while the interior below
    // does not move — holds at <0.8x; the >2x figure was a
    // product-composition artifact.
    assert!(
        roof_mid.mean_lum() < roof_noon.mean_lum() * 0.8,
        "exposed roof must darken at midnight vs noon ({:.3} -> {:.3})",
        roof_noon.mean_lum(),
        roof_mid.mean_lum()
    );

    // --- Cutaway view (cutaway 0): the lamp room ignores noon.
    let args_cut = house_args(lit(), 0);
    let c_neutral = capture_rt_with_device_sky(&device, &queue, &args_cut, None)
        .expect("neutral cutaway RT capture");
    let c_noon = capture_rt_with_device_sky(&device, &queue, &args_cut, Some(&noon_u))
        .expect("noon cutaway RT capture");
    let (int_neut, _, _) = region_stats(&args_cut, &c_neutral, None);
    let (int_noon, _, other_noon) = region_stats(&args_cut, &c_noon, Some(&c_neutral));
    println!(
        "10.4a interior: {} px, mean|diff| noon-vs-neutral {:.4} (other regions {:.4})",
        int_neut.n,
        int_noon.mean_diff(),
        other_noon.mean_diff()
    );
    assert!(int_neut.n > 200, "interior must be visible in the cutaway capture ({} px)", int_neut.n);
    assert!(
        int_noon.mean_diff() <= 0.05,
        "interior must stay within noise of neutral at noon (mean|diff| {:.4})",
        int_noon.mean_diff()
    );

    // --- BLANK: marked tiles never produce geometry.
    let blank: Vec<[i32; 3]> = (0..4)
        .flat_map(|x| (0..4).map(move |y| [x, y, 0]))
        .collect();
    let marked = LightingScene::fake_house_marked(12.0, WeatherKind::Clear, &blank, &[]);
    let args_b = house_args(marked, 0);
    let cap_b = capture_rt_with_device_sky(&device, &queue, &args_b, Some(&noon_u))
        .expect("blank-marked RT capture");
    let size = cap_b.size;
    let mut hits_in_blank = 0usize;
    let mut misses = 0usize;
    for y in 0..size.1 {
        for x in 0..size.0 {
            match hit_tile_strict(&args_b, &cap_b.depth, cap_b.max_dist, size, x, y) {
                Some(t) if blank.contains(&t) => hits_in_blank += 1,
                _ => {}
            }
            if cap_b.depth[(y * size.0 + x) as usize] >= cap_b.max_dist {
                misses += 1;
            }
        }
    }
    println!("10.4a BLANK: {hits_in_blank} hit px in blank tiles, {misses} sky px");
    assert_eq!(hits_in_blank, 0, "BLANK tiles must never render as geometry");
    assert!(misses > 0, "the frame must contain sky/background pixels");
}
