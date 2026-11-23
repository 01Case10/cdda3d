//! Phase 10 slice 10.4 gates (RT sky GI, docs/phase10-sky-gi.md
//! §5.3 + §8): 10.3a RT half (empty-scene sky pixels vs the
//! `sky_color_cpu` mirror), 10.4b (shadow direction/length measured
//! on captures at dawn vs noon; overcast collapse), and the
//! per-tile sky-exposure channel (§3.4, decision 4A) reaching the
//! side table. GPU halves are adapter-conditional in the
//! established pattern, but print the adapter name and fail loudly
//! on blank renders — a silent skip proves nothing.

use glam::Vec3;
use lighting::{sky_color_cpu, sky_state, GameEnvironment, SkyUniforms};
use treebuild::build::{build_tree, BuildConfig};
use treebuild::render::{RtCamera, RtRenderer};
use treebuild::world::{VoxelCell, VoxelWorld, FLAG_SKY_EXPOSED};

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    name: String,
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
    let info = adapter.get_info();
    let name = format!("{} ({:?}, {:?})", info.name, info.device_type, info.backend);
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("sky-gi-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu {
        device,
        queue,
        name,
    })
}

fn read_rgba(gpu: &Gpu, texture: &wgpu::Texture, size: (u32, u32)) -> Vec<u8> {
    let (w, h) = size;
    let unpadded = w * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sky-gate-readback"),
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
            let s = row * padded as usize;
            out.extend_from_slice(&data[s..s + unpadded as usize]);
        }
    }
    staging.unmap();
    out
}

fn read_depth(gpu: &Gpu, buffer: &wgpu::Buffer, count: usize) -> Vec<f32> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sky-gate-depth"),
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

/// Per-pixel ray direction exactly as `render_main` builds it
/// (camera basis formulas mirrored from `RtCamera::basis`).
fn pixel_rd(camera: &RtCamera, size: (u32, u32), x: u32, y: u32) -> Vec3 {
    let fwd = (camera.target - camera.eye).normalize_or_zero();
    let right = fwd.cross(Vec3::Z).normalize_or_zero();
    let up = right.cross(fwd);
    let tan = (camera.fov_y_deg.to_radians() * 0.5).tan();
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let ndc = glam::Vec2::new(
        (px / size.0 as f32) * 2.0 - 1.0,
        1.0 - (py / size.1 as f32) * 2.0,
    );
    (fwd + ndc.x * tan * camera.aspect * right + ndc.y * tan * up).normalize()
}

/// SkyState -> uniforms at a fake-data time of day (summer,
/// sunrise 06:00, sunset 18:00 — the §3.2 fallback path).
fn uniforms_at(time_of_day_sec: f32, moon_phase: u8) -> SkyUniforms {
    let env = GameEnvironment::fake_day(time_of_day_sec, 21_600.0, 64_800.0, 1, moon_phase);
    SkyUniforms::from_sky_state(&sky_state(&env), time_of_day_sec)
}

// ---------------------------------------------------------- channel
/// §3.4 channel (CPU, runs everywhere): a cell's sky exposure
/// rides `VoxelCell` into the side table as FLAG_SKY_EXPOSED;
/// the default (fake/unlit data) is exposed.
#[test]
fn sky_exposure_channel_reaches_side_table() {
    let mut w = VoxelWorld::new();
    let mut open = VoxelCell::new([0.5, 0.5, 0.5], 1);
    open.sky_exposed = true;
    w.set([0, 0, 0], open);
    let mut roofed = VoxelCell::new([0.4, 0.4, 0.4], 1);
    roofed.sky_exposed = false;
    w.set([1, 0, 0], roofed);
    w.set([2, 0, 0], VoxelCell::new([0.3, 0.3, 0.3], 1)); // default
    let built = build_tree(&w, BuildConfig::default()).unwrap();
    let flag_at = |pos: [i32; 3]| {
        let idx = (0..built.node_count())
            .find(|&i| built.leaf_voxel(i) == Some(pos))
            .unwrap_or_else(|| panic!("no leaf for {pos:?}"));
        built.side[idx].flags
    };
    assert!(flag_at([0, 0, 0]) & FLAG_SKY_EXPOSED != 0, "exposed cell");
    assert!(flag_at([1, 0, 0]) & FLAG_SKY_EXPOSED == 0, "roofed cell");
    assert!(
        flag_at([2, 0, 0]) & FLAG_SKY_EXPOSED != 0,
        "default VoxelCell must be sky-exposed (legacy directional reach)"
    );
}

// ------------------------------------------------------------- 10.3a
/// Gate 10.3a (RT half): an empty scene renders pure sky; every
/// pixel must match the CPU mirror of the shared WGSL at the same
/// uniforms within sRGB ±2 per channel — at neutral, at noon, and
/// at midnight (stars/discs live).
#[test]
fn gate_10_3a_rt_sky_matches_cpu_mirror() {
    let Some(gpu) = gpu() else {
        eprintln!("10.3a RT: no wgpu adapter — NOT RUN");
        return;
    };
    println!("10.3a RT adapter: {}", gpu.name);
    let world = VoxelWorld::new();
    let built = build_tree(&world, BuildConfig::default()).unwrap();
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    let camera = RtCamera {
        eye: Vec3::new(6.0, -14.0, 10.0),
        target: Vec3::new(6.0, 6.0, 1.0),
        fov_y_deg: 60.0,
        aspect: 4.0 / 3.0,
    };
    let size = (160u32, 120u32);
    let cases: [(&str, SkyUniforms); 3] = [
        ("neutral", SkyUniforms::neutral()),
        ("noon", uniforms_at(43_200.0, 0)),
        ("midnight", uniforms_at(0.0, 4)),
    ];
    for (name, uniforms) in cases {
        scene.set_sky(&gpu.queue, &uniforms);
        let frame = renderer.render(&gpu.device, &gpu.queue, &scene, &camera, size, true);
        gpu.device.poll(wgpu::Maintain::Wait);
        let px = read_rgba(&gpu, &frame.color, size);
        let depth = read_depth(&gpu, &frame.depth, (size.0 * size.1) as usize);
        assert!(
            depth.iter().all(|&d| d >= frame.max_dist),
            "10.3a {name}: empty scene must miss everywhere"
        );
        let mut worst = 0i32;
        for y in 0..size.1 {
            for x in 0..size.0 {
                let rd = pixel_rd(&camera, size, x, y);
                let want = sky_color_cpu(rd.z, rd.to_array(), &uniforms);
                let i = ((y * size.0 + x) * 4) as usize;
                for c in 0..3 {
                    let want_byte = (want[c].clamp(0.0, 1.0) * 255.0).round() as i32;
                    let got = px[i + c] as i32;
                    worst = worst.max((want_byte - got).abs());
                }
            }
        }
        println!("10.3a {name}: worst channel delta {worst}/255 over {} px", size.0 * size.1);
        assert!(worst <= 2, "10.3a {name}: sky pixels drifted from the CPU mirror (worst {worst})");
    }
}

// ------------------------------------------------------------- 10.4b
/// Wall + ground scene for the shadow gate (grid coords).
fn shadow_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    for y in 0..32 {
        for x in 0..32 {
            w.set([x, y, 0], VoxelCell::new([0.55, 0.50, 0.45], 1));
        }
    }
    for z in 1..=5 {
        for y in 14..18 {
            for x in 14..18 {
                w.set([x, y, z], VoxelCell::new([0.75, 0.30, 0.20], 2));
            }
        }
    }
    w
}

struct ShadowMeasure {
    ground_px: usize,
    shadow_px: usize,
    /// Shadow centroid in GRID coords (the wall footprint is
    /// x/y 14..18, centre (16, 16)).
    grid_centroid: (f32, f32),
    /// Grid-space extent of the shadow along x (min, max).
    grid_x_extent: (f32, f32),
    median_ground_lum: f32,
    min_ground_ratio: f32,
}

fn measure_shadow(
    px: &[u8],
    depth: &[f32],
    max_dist: f32,
    size: (u32, u32),
    camera: &RtCamera,
) -> ShadowMeasure {
    let (w, _h) = size;
    let lum = |i: usize| -> f32 {
        (0.299 * px[i] as f32 + 0.587 * px[i + 1] as f32 + 0.114 * px[i + 2] as f32) / 255.0
    };
    // Wall vs ground by hue: wall albedo is strongly red-dominant.
    let is_wall = |i: usize| -> bool {
        let (r, b) = (px[i] as f32, px[i + 2] as f32);
        (r - b) / (r + b + 1.0) > 0.25
    };
    // Ground TOP faces only: unproject the hit — the slab's side
    // faces are ground-albedo too and legitimately darker (they
    // face sideways), so counting them pollutes the shadow measure
    // (a fixed 1894-px band in the first run of this gate).
    let is_ground_top = |x: u32, y: u32| -> bool {
        let p = (y * w + x) as usize;
        if depth[p] >= max_dist {
            return false;
        }
        let rd = pixel_rd(camera, size, x, y);
        let hitp = camera.eye + rd * depth[p];
        (hitp.z - 1.0).abs() < 0.05
    };
    let mut ground_lums: Vec<f32> = Vec::new();
    let (mut wx, mut wy, mut wn) = (0f32, 0f32, 0usize);
    for y in 0..size.1 {
        for x in 0..size.0 {
            let p = (y * w + x) as usize;
            if depth[p] >= max_dist {
                continue;
            }
            let i = p * 4;
            if is_wall(i) {
                wx += x as f32;
                wy += y as f32;
                wn += 1;
            } else if is_ground_top(x, y) {
                ground_lums.push(lum(i));
            }
        }
    }
    assert!(wn > 100, "wall must be visible ({wn} px)");
    let _ = (wx, wy);
    ground_lums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = ground_lums[ground_lums.len() / 2];
    let (mut sx, mut sy, mut sn) = (0f32, 0f32, 0usize);
    let (mut xmin, mut xmax) = (f32::MAX, f32::MIN);
    let mut min_ratio = 1.0f32;
    for y in 0..size.1 {
        for x in 0..size.0 {
            let p = (y * w + x) as usize;
            if depth[p] >= max_dist {
                continue;
            }
            let i = p * 4;
            if is_wall(i) || !is_ground_top(x, y) {
                continue;
            }
            let l = lum(i);
            min_ratio = min_ratio.min(l / median);
            if l < 0.72 * median {
                let rd = pixel_rd(camera, size, x, y);
                let hitp = camera.eye + rd * depth[p];
                sx += hitp.x;
                sy += hitp.y;
                xmin = xmin.min(hitp.x);
                xmax = xmax.max(hitp.x);
                sn += 1;
            }
        }
    }
    let grid_centroid = if sn > 0 {
        (sx / sn as f32, sy / sn as f32)
    } else {
        (0.0, 0.0)
    };
    ShadowMeasure {
        ground_px: ground_lums.len(),
        shadow_px: sn,
        grid_centroid,
        grid_x_extent: (xmin, xmax),
        median_ground_lum: median,
        min_ground_ratio: min_ratio,
    }
}

/// Gate 10.4b: measured on the captures — dawn shadow is long and
/// falls opposite the low eastern sun; noon shadow is short and
/// falls opposite the high southern sun; an overcast
/// (Thunderstorm) noon casts no hard shadow.
#[test]
fn gate_10_4b_shadow_direction_length_and_overcast() {
    let Some(gpu) = gpu() else {
        eprintln!("10.4b: no wgpu adapter — NOT RUN");
        return;
    };
    println!("10.4b adapter: {}", gpu.name);
    let built = build_tree(&shadow_world(), BuildConfig::default()).unwrap();
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    let camera = RtCamera {
        eye: Vec3::new(16.0, 38.0, 14.0),
        target: Vec3::new(16.0, 14.0, 0.0),
        fov_y_deg: 60.0,
        aspect: 4.0 / 3.0,
    };
    let size = (320u32, 240u32);
    let render = |scene: &mut treebuild::render::RtScene, u: &SkyUniforms| {
        scene.set_sky(&gpu.queue, u);
        let frame = renderer.render(&gpu.device, &gpu.queue, scene, &camera, size, true);
        gpu.device.poll(wgpu::Maintain::Wait);
        let px = read_rgba(&gpu, &frame.color, size);
        let depth = read_depth(&gpu, &frame.depth, (size.0 * size.1) as usize);
        measure_shadow(&px, &depth, frame.max_dist, size, &camera)
    };
    // 07:30 summer (sun alt ~25 deg, azimuth ~292 deg: east).
    let dawn = render(&mut scene, &uniforms_at(27_000.0, 0));
    // 12:00 summer (alt 65 deg, azimuth 0: south).
    let noon = render(&mut scene, &uniforms_at(43_200.0, 0));
    // Overcast noon: thunderstorm numerics at noon.
    let mut storm_env = GameEnvironment::fake_day(43_200.0, 21_600.0, 64_800.0, 1, 0);
    storm_env.precip_class = 3;
    storm_env.rains = true;
    storm_env.lightning_active = true;
    storm_env.light_modifier = -40.0;
    storm_env.sun_light = 30.0;
    let storm_u = SkyUniforms::from_sky_state(&sky_state(&storm_env), 43_200.0);
    let storm = render(&mut scene, &storm_u);

    println!(
        "10.4b dawn: shadow {} px, grid centroid ({:.1}, {:.1}), x extent ({:.1}, {:.1}), median ground lum {:.3}",
        dawn.shadow_px, dawn.grid_centroid.0, dawn.grid_centroid.1, dawn.grid_x_extent.0,
        dawn.grid_x_extent.1, dawn.median_ground_lum
    );
    println!(
        "10.4b noon: shadow {} px, grid centroid ({:.1}, {:.1}), x extent ({:.1}, {:.1}), median ground lum {:.3}",
        noon.shadow_px, noon.grid_centroid.0, noon.grid_centroid.1, noon.grid_x_extent.0,
        noon.grid_x_extent.1, noon.median_ground_lum
    );
    println!(
        "10.4b storm: shadow {} px of {} ground px, min ground ratio {:.3}",
        storm.shadow_px, storm.ground_px, storm.min_ground_ratio
    );
    // Dawn (sun alt ~25 deg at azimuth ~292 deg — east): a real,
    // LONG shadow falling -x in grid space, opposite the sun.
    // Wall footprint centre is (16, 16); its west edge is x = 14.
    assert!(dawn.shadow_px > 300, "dawn must cast a substantial shadow ({} px)", dawn.shadow_px);
    assert!(
        dawn.grid_centroid.0 < 12.0,
        "dawn shadow centroid must sit west of the wall (grid x {:.1})",
        dawn.grid_centroid.0
    );
    assert!(
        dawn.grid_x_extent.1 - dawn.grid_x_extent.0 > 6.0,
        "dawn shadow must be long in grid x (extent {:.1}..{:.1})",
        dawn.grid_x_extent.0,
        dawn.grid_x_extent.1
    );
    // Noon (alt 65 deg, azimuth 0): the shadow is SHORT and falls
    // +y, opposite the high southern sun — a different direction
    // AND a different length than dawn's, in the sun's favour.
    assert!(noon.shadow_px > 50, "noon must still cast a shadow ({} px)", noon.shadow_px);
    assert!(
        noon.grid_centroid.1 > 17.0,
        "noon shadow centroid must sit +y of the wall centre (grid y {:.1})",
        noon.grid_centroid.1
    );
    assert!(
        noon.grid_centroid.0 > dawn.grid_centroid.0 + 3.0,
        "the shadow must swing from dawn's -x fall to noon's +y fall (centroid x {:.1} -> {:.1})",
        dawn.grid_centroid.0,
        noon.grid_centroid.0
    );
    // Overcast: the directional term collapsed — no hard shadow.
    assert!(
        storm.shadow_px * 20 < dawn.shadow_px || storm.min_ground_ratio > 0.72,
        "overcast must not cast a hard shadow ({} px, min ratio {:.3})",
        storm.shadow_px,
        storm.min_ground_ratio
    );
}
