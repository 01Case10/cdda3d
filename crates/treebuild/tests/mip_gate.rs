//! Phase 2e gate tests (plan Part II §5e): mip chain correctness,
//! ray-cone LOD selection (CPU analytic + GPU debug-view readback),
//! and two-pose aliasing reduction. GPU tests are adapter-conditional
//! in the established pattern: with no adapter they print that the
//! GPU checks were not run and return.

use glam::Vec3;
use treebuild::build::{build_tree, BuildConfig};
use treebuild::lod::{downsample2x, mip_chain, ray_cone_lod, spread_angle};
use treebuild::world::{VoxelCell, VoxelWorld};

// ---------------------------------------------------------------------
// Patterns + an independent reference downsample (gate 2e.1)
// ---------------------------------------------------------------------

fn checkerboard(size: u32, check: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let white = ((x / check) + (y / check)) % 2 == 0;
            let c = if white { 255u8 } else { 0u8 };
            v.extend_from_slice(&[c, c, c, 255]);
        }
    }
    v
}

fn gradient(size: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((size * size * 4) as usize);
    let m = (size - 1).max(1) as f32;
    for y in 0..size {
        for x in 0..size {
            v.extend_from_slice(&[
                (x as f32 / m * 255.0).round() as u8,
                (y as f32 / m * 255.0).round() as u8,
                ((x + y) as f32 / (2.0 * m) * 255.0).round() as u8,
                255,
            ]);
        }
    }
    v
}

/// Cutout: ~80% opaque red, ~20% fully transparent texels whose RGB
/// is garbage green — any color bleed from transparent neighbors
/// shows up as green in the mips.
fn cutout(size: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            if (x * 7 + y * 13) % 5 == 0 {
                v.extend_from_slice(&[0, 255, 0, 0]);
            } else {
                v.extend_from_slice(&[200, 30, 20, 255]);
            }
        }
    }
    v
}

/// Half-plane cutout: left half opaque pure red, right half
/// transparent garbage green. Texels far from the seam have an
/// unambiguous expected value at every level.
fn half_cutout(size: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((size * size * 4) as usize);
    for _y in 0..size {
        for x in 0..size {
            if x < size / 2 {
                v.extend_from_slice(&[255, 0, 0, 255]);
            } else {
                v.extend_from_slice(&[0, 255, 0, 0]);
            }
        }
    }
    v
}

/// Independent reference: premultiplied alpha-aware 2x2 average with
/// edge clamping, in f64, written separately from `crate::lod`.
fn ref_downsample(layer: &[u8], size: u32) -> Vec<u8> {
    let out_size = (size / 2).max(1);
    let mut out = Vec::with_capacity((out_size * out_size * 4) as usize);
    for oy in 0..out_size {
        for ox in 0..out_size {
            let mut acc = [0f64; 4];
            for dy in 0..2u32 {
                for dx in 0..2u32 {
                    let sx = (ox * 2 + dx).min(size - 1) as usize;
                    let sy = (oy * 2 + dy).min(size - 1) as usize;
                    let i = (sy * size as usize + sx) * 4;
                    let a = layer[i + 3] as f64 / 255.0;
                    acc[0] += layer[i] as f64 * a;
                    acc[1] += layer[i + 1] as f64 * a;
                    acc[2] += layer[i + 2] as f64 * a;
                    acc[3] += a;
                }
            }
            let mean_a = acc[3] / 4.0;
            let mut px = [0u8; 4];
            for c in 0..3 {
                px[c] = if mean_a > 0.001 {
                    ((acc[c] / 4.0) / mean_a).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                };
            }
            px[3] = (mean_a * 255.0).round() as u8;
            out.extend_from_slice(&px);
        }
    }
    out
}

fn mean_alpha(layer: &[u8]) -> f64 {
    layer.iter().skip(3).step_by(4).map(|&a| a as f64).sum::<f64>() / (layer.len() / 4) as f64
}

// ---------------------------------------------------------------- 2e.1
#[test]
fn gate_2e_1_chain_correctness() {
    let size = 32u32;
    for (name, layer) in [
        ("checkerboard", checkerboard(size, 1)),
        ("gradient", gradient(size)),
        ("cutout", cutout(size)),
    ] {
        // Single step matches the independent reference exactly-ish.
        let step = downsample2x(&layer, size);
        let reference = ref_downsample(&layer, size);
        assert_eq!(step.len(), reference.len(), "{name}: level-1 size");
        for (i, (a, b)) in step.iter().zip(reference.iter()).enumerate() {
            assert!(
                (*a as i32 - *b as i32).abs() <= 2,
                "{name}: downsample2x byte {i}: {a} vs reference {b}"
            );
        }

        // Full chain: 32 -> 1 is 6 levels, sizes halve each time.
        let chain = mip_chain(&layer, size);
        assert_eq!(chain.len(), 6, "{name}: chain length");
        assert_eq!(chain[0], layer, "{name}: level 0 is the input");
        let mut ref_chain = vec![layer.clone()];
        let mut s = size;
        while s > 1 {
            ref_chain.push(ref_downsample(ref_chain.last().unwrap(), s));
            s = (s / 2).max(1);
        }
        let level0_mean_alpha = mean_alpha(&layer);
        let mut s = size;
        for (lvl, (got, want)) in chain.iter().zip(ref_chain.iter()).enumerate() {
            assert_eq!(got.len(), (s * s * 4) as usize, "{name} level {lvl} size");
            for (i, (a, b)) in got.iter().zip(want.iter()).enumerate() {
                let chan = i % 4;
                // Alpha may move a few LSB under the coverage
                // rescale; RGB is only meaningful where the
                // reference texel has real coverage (un-premultiply
                // is unstable near alpha 0).
                let tol = if chan == 3 {
                    4
                } else if want[(i / 4) * 4 + 3] >= 32 {
                    3
                } else {
                    continue;
                };
                assert!(
                    (*a as i32 - *b as i32).abs() <= tol,
                    "{name} level {lvl} byte {i}: {a} vs reference {b} (tol {tol})"
                );
            }
            // Coverage preservation: mean alpha tracks level 0.
            let drift = (mean_alpha(got) - level0_mean_alpha).abs();
            assert!(
                drift <= 1.5,
                "{name} level {lvl}: mean alpha {} drifts {drift:.2} from level 0's {level0_mean_alpha:.2}",
                mean_alpha(got)
            );
            s = (s / 2).max(1);
        }
    }

    // Opaque layers: the coverage rescale is a no-op, alpha stays
    // pinned at 255 on every level.
    for lvl in mip_chain(&gradient(size), size) {
        assert!(lvl.iter().skip(3).step_by(4).all(|&a| a == 255));
    }

    // No color bleed from transparent neighbors: texels fully inside
    // the opaque half stay pure red; texels fully inside the
    // transparent half are empty (alpha 0, no green).
    let chain = mip_chain(&half_cutout(size), size);
    // Generated levels only: level 0 is the input itself, whose
    // transparent texels carry the garbage color by construction.
    for (lvl, level) in chain.iter().enumerate().skip(1) {
        let s = size >> lvl;
        for y in 0..s {
            for x in 0..s {
                let i = ((y * s + x) * 4) as usize;
                let px = &level[i..i + 4];
                // This texel's source footprint in level-0 coords.
                let span = size / s;
                let (x0, x1) = (x * span, x * span + span);
                if x1 <= size / 2 {
                    assert_eq!(px, [255, 0, 0, 255], "level {lvl} texel ({x},{y}) inside opaque half");
                } else if x0 >= size / 2 {
                    assert_eq!(px[3], 0, "level {lvl} texel ({x},{y}) inside transparent half: alpha");
                    assert_eq!(px[1], 0, "level {lvl} texel ({x},{y}): green bleed from transparent texels");
                }
            }
        }
    }
}

// ---------------------------------------------------------------- 2e.2
#[test]
fn gate_2e_2_cpu_analytic_lod() {
    let spread = spread_angle((60.0f32.to_radians() * 0.5).tan(), 512);
    let max_mip = 10.0;
    let lod = |t: f32, ndv: f32| ray_cone_lod(32, 32, spread, t, ndv, max_mip);

    // Distance doubling raises LOD by exactly 1 (fronto-parallel).
    for t in [100.0f32, 200.0, 400.0] {
        let d = lod(t * 2.0, 1.0) - lod(t, 1.0);
        assert!((d - 1.0).abs() <= 0.15, "doubling at t={t}: LOD delta {d}");
    }
    // Slant adds -NDOTV_WEIGHT * log2(cos theta) (Stage A: the
    // weight halves the pre-Stage-A slant penalty of -log2(cos)).
    for deg in [30.0f32, 45.0, 60.0] {
        let cos = deg.to_radians().cos();
        let d = lod(200.0, cos) - lod(200.0, 1.0);
        let want = -treebuild::NDOTV_WEIGHT * cos.log2();
        assert!(
            (d - want).abs() <= 0.15,
            "slant {deg} deg: LOD delta {d}, want {want}"
        );
    }
    // Delta for a 32x32 texture is 5: at w_hit == 1 the LOD is
    // 5 + LOD_BIAS (Stage A bias −0.5 → 4.5).
    let t_unit = 1.0 / spread;
    assert!((lod(t_unit, 1.0) - (5.0 + treebuild::LOD_BIAS)).abs() <= 0.01);
    // Hygiene: clamps hold and NaN inputs cannot produce NaN out.
    assert_eq!(lod(1e-6, 1.0), 0.0, "close-up clamps to level 0");
    assert_eq!(lod(1e6, 0.0), max_mip, "grazing + far clamps to max mip");
    for bad in [f32::NAN, f32::INFINITY, -1.0] {
        let v = ray_cone_lod(32, 32, spread, bad, 1.0, max_mip);
        assert!(v.is_finite() && (0.0..=max_mip).contains(&v), "t={bad}: lod {v}");
        let v = ray_cone_lod(32, 32, spread, 200.0, bad, max_mip);
        assert!(v.is_finite() && (0.0..=max_mip).contains(&v), "ndv={bad}: lod {v}");
    }
}

// ---------------------------------------------------------------------
// GPU helpers (duplicated from gate_tests.rs — test files are
// independent crates).
// ---------------------------------------------------------------------

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn gpu() -> Option<Gpu> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("treebuild-mip-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu { device, queue })
}

fn read_rgba_texture(gpu: &Gpu, texture: &wgpu::Texture, size: (u32, u32)) -> Vec<u8> {
    let (w, h) = size;
    let unpadded = w * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mip-gate-tex-readback"),
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..h as usize {
            let start = row * padded as usize;
            out.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    out
}

fn read_f32_buffer(gpu: &Gpu, buffer: &wgpu::Buffer, count: usize) -> Vec<f32> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mip-gate-readback"),
        size: (count * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (count * 4) as u64);
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let data = slice.get_mapped_range();
    let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    out
}

/// A 16x16 voxel wall in the plane y=8 (front face toward -y),
/// every voxel carrying material 3.
fn wall_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    for z in 0..16 {
        for x in 0..16 {
            w.set([x, 8, z], VoxelCell::new([0.5, 0.5, 0.5], 3));
        }
    }
    w
}

fn wall_scene(gpu: &Gpu, layer: Vec<u8>, tile: u32) -> (treebuild::render::RtRenderer, treebuild::render::RtScene) {
    use treebuild::render::{RtRenderer, TextureSpec};
    let built = build_tree(&wall_world(), BuildConfig::default()).unwrap();
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    let mut material_layers = vec![[0u32, 0, 0, 0]; 4];
    material_layers[3] = [0, 0, 0, 1];
    scene
        .attach_textures(
            &gpu.device,
            &gpu.queue,
            &TextureSpec {
                tile_size: tile,
                layers: vec![layer],
                material_layers,
            },
        )
        .unwrap();
    (renderer, scene)
}

/// Camera looking straight at the wall's front face from distance
/// `dist` (eye on the wall's axis), fov 60, square aspect.
fn wall_camera(dist: f32, x_shift: f32) -> treebuild::render::RtCamera {
    treebuild::render::RtCamera {
        eye: Vec3::new(8.0 + x_shift, 8.0 - dist, 8.0),
        target: Vec3::new(8.0 + x_shift, 8.0, 8.0),
        fov_y_deg: 60.0,
        aspect: 1.0,
    }
}

/// The renderer's own ray for a pixel of a 128x128 wall_camera
/// frame, replicated for CPU-side expectations: returns (t to the
/// wall plane, |N·V|) for the fronto-parallel wall.
fn pixel_ray(px: u32, py: u32, dist: f32) -> (f32, f32) {
    let tan = (60.0f32.to_radians() * 0.5).tan();
    let ndc = Vec3::new(
        ((px as f32 + 0.5) / 128.0) * 2.0 - 1.0,
        0.0,
        1.0 - ((py as f32 + 0.5) / 128.0) * 2.0,
    );
    // fwd = +y, right = +x, up = +z (see wall_camera basis).
    let rd = (Vec3::Y + ndc.x * tan * Vec3::X + ndc.z * tan * Vec3::Z).normalize();
    (dist / rd.y, rd.y)
}

/// The WGSL `lod_color` palette, mirrored for readback decoding.
const LOD_PALETTE: [[u8; 3]; 9] = [
    [0, 0, 255],
    [0, 255, 0],
    [255, 255, 0],
    [255, 128, 0],
    [255, 0, 0],
    [255, 0, 255],
    [0, 255, 255],
    [255, 255, 255],
    [128, 0, 128],
];

fn decode_lod_level(px: &[u8]) -> i32 {
    let mut best = 0i32;
    let mut best_d = i64::MAX;
    for (i, c) in LOD_PALETTE.iter().enumerate() {
        let d: i64 = (0..3).map(|k| (px[k] as i64 - c[k] as i64).pow(2)).sum();
        if d < best_d {
            best_d = d;
            best = i as i32;
        }
    }
    best
}

// ------------------------------------------------- 2e.2 GPU + chain
#[test]
fn gate_2e_2_gpu_debug_lod_matches_cpu() {
    let Some(gpu) = gpu() else {
        eprintln!("2e.2 GPU: no wgpu adapter — GPU debug-LOD checks NOT RUN (CPU half passes above)");
        return;
    };
    let (renderer, scene) = wall_scene(&gpu, gradient(32), 32);
    assert_eq!(scene.textures.mip_levels, 6, "32px layers carry a full 6-level chain");
    // The dummy set stays a single level.
    let dummy = treebuild::render::TextureSet::dummy(&gpu.device, &gpu.queue);
    assert_eq!(dummy.mip_levels, 1);

    let spread = spread_angle((60.0f32.to_radians() * 0.5).tan(), 128);
    let max_mip = 5.0;
    let mut center_levels = Vec::new();
    for dist in [32.0f32, 64.0] {
        let camera = wall_camera(dist, 0.0);
        let frame = renderer.render_debug_lod(&gpu.device, &gpu.queue, &scene, &camera, (128, 128));
        let pixels = read_rgba_texture(&gpu, &frame.color, frame.size);
        let depth = read_f32_buffer(&gpu, &frame.depth, 128 * 128);
        let at = |px: u32, py: u32| -> (i32, f32) {
            let i = ((py * 128 + px) * 4) as usize;
            (decode_lod_level(&pixels[i..i + 3]), depth[(py * 128 + px) as usize])
        };
        // Probed points around the center (silhouette excluded: the
        // wall fills this whole neighborhood at both distances).
        for (px, py) in [(64u32, 64u32), (56, 64), (72, 64), (64, 56), (64, 72)] {
            let (level, d) = at(px, py);
            assert!(d < frame.max_dist, "probe ({px},{py}) at dist {dist} must hit the wall");
            let (t, ndv) = pixel_ray(px, py, dist);
            let want = ray_cone_lod(32, 32, spread, t, ndv, max_mip);
            // Skip probes whose expected LOD sits on a level
            // boundary — floor() is discontinuous there by design.
            if (want.fract() - 0.5).abs() < 0.1 {
                continue;
            }
            assert_eq!(
                level,
                want.floor() as i32,
                "dist {dist} probe ({px},{py}): GPU floor(lod) {level}, CPU lod {want:.3}"
            );
            if (px, py) == (64, 64) {
                center_levels.push(level);
            }
        }
    }
    assert_eq!(center_levels.len(), 2);
    assert_eq!(
        center_levels[1] - center_levels[0],
        1,
        "doubling the distance must raise the debug LOD level by 1: {center_levels:?}"
    );
}

// ---------------------------------------------------------------- 2e.3
#[test]
fn gate_2e_3_two_pose_variance() {
    let Some(gpu) = gpu() else {
        eprintln!("2e.3: no wgpu adapter — GPU two-pose checks NOT RUN");
        return;
    };
    // 1px checkerboard: the worst aliasing case there is.
    let (renderer, scene) = wall_scene(&gpu, checkerboard(32, 1), 32);
    let dist = 48.0f32;
    let render_pose = |x_shift: f32| -> (Vec<u8>, Vec<f32>, f32) {
        let camera = wall_camera(dist, x_shift);
        let frame = renderer.render(&gpu.device, &gpu.queue, &scene, &camera, (128, 128), false);
        (
            read_rgba_texture(&gpu, &frame.color, frame.size),
            read_f32_buffer(&gpu, &frame.depth, 128 * 128),
            frame.max_dist,
        )
    };
    // A quarter-footprint lateral shift: invisible to a mipmapped
    // render (the footprint is ~14 texels wide here), devastating
    // to level-0 nearest sampling.
    let footprint = spread_angle((60.0f32.to_radians() * 0.5).tan(), 128) * dist;
    let (a, da, max_dist) = render_pose(0.0);
    let (b, db, _) = render_pose(footprint * 0.25);

    // Central wall region, silhouettes excluded, hits in both poses.
    let mut gpu_diff_sum = 0f64;
    let mut sim_diff_sum = 0f64;
    let mut count = 0usize;
    for py in 48u32..80 {
        for px in 48u32..80 {
            let idx = (py * 128 + px) as usize;
            if da[idx] >= max_dist || db[idx] >= max_dist {
                continue;
            }
            count += 1;
            for c in 0..3 {
                gpu_diff_sum += (a[idx * 4 + c] as f64 - b[idx * 4 + c] as f64).abs();
            }
            // Simulated level-0-only render of the same two poses:
            // nearest texel of the 1px checkerboard at the exact
            // hit UV (side face: u = fract(x), v = 1 - fract(z)).
            let sim = |x_shift: f32| -> f64 {
                let (t, _) = pixel_ray(px, py, dist);
                let tan = (60.0f32.to_radians() * 0.5).tan();
                let ndc_x = ((px as f32 + 0.5) / 128.0) * 2.0 - 1.0;
                let ndc_y = 1.0 - ((py as f32 + 0.5) / 128.0) * 2.0;
                let rd = (Vec3::Y + ndc_x * tan * Vec3::X + ndc_y * tan * Vec3::Z).normalize();
                let hit = Vec3::new(8.0 + x_shift, 8.0 - dist, 8.0) + rd * t;
                let tx = (hit.x.fract() * 32.0).floor() as i32;
                let tz = (hit.z.fract() * 32.0).floor() as i32;
                if (tx + tz) % 2 == 0 { 255.0 } else { 0.0 }
            };
            sim_diff_sum += (sim(0.0) - sim(footprint * 0.25)).abs();
        }
    }
    assert!(count > 500, "wall region must be well covered, got {count} px");
    let gpu_mad = gpu_diff_sum / (count * 3) as f64;
    let sim_mad = sim_diff_sum / count as f64;
    println!(
        "2e.3 two-pose mean |diff|: mipmapped GPU {gpu_mad:.2}, simulated level-0-only {sim_mad:.2} ({} px)",
        count
    );
    assert!(sim_mad > 40.0, "level-0 baseline must actually alias (sim MAD {sim_mad:.1})");
    assert!(
        gpu_mad < sim_mad * 0.5,
        "mipmapped two-pose variance {gpu_mad:.2} must drop by at least half vs level-0's {sim_mad:.2}"
    );
    assert!(gpu_mad < 12.0, "mipmapped wall must be near-stable between poses ({gpu_mad:.2})");

    // Close-up identity: at LOD-0 range the checkerboard is still
    // sampled sharp — both extremes present in the wall region.
    let camera = wall_camera(2.0, 0.0);
    let frame = renderer.render(&gpu.device, &gpu.queue, &scene, &camera, (128, 128), false);
    let pixels = read_rgba_texture(&gpu, &frame.color, frame.size);
    let depth = read_f32_buffer(&gpu, &frame.depth, 128 * 128);
    let reds: Vec<u8> = depth
        .iter()
        .zip(pixels.chunks_exact(4))
        .filter(|(&d, _)| d < frame.max_dist)
        .map(|(_, px)| px[0])
        .collect();
    assert!(reds.len() > 1000, "close-up wall must fill the frame");
    // The -y face's shading factor is ~0.60 (sun dot 0.42 -> light
    // 0.71, times lit_factor 0.85), so a white texel lands at ~154
    // and a black one at 0; a mip-averaged render would sit at a
    // uniform ~77 instead. Sharp extremes = level 0 sampled.
    let (min, max) = (*reds.iter().min().unwrap(), *reds.iter().max().unwrap());
    assert!(min < 40, "close-up: black texels must survive (min {min})");
    assert!(max > 130, "close-up: white texels must survive (max {max})");
    assert!(max - min > 100, "close-up contrast must stay sharp ({min}..{max})");
}
