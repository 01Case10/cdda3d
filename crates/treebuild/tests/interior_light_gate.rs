//! Option B regression gate (Joey's call, 2026-11-25): the RT
//! interior-lighting restructure.
//!
//! The pre-fix `shade()` composed three independent darkening
//! factors multiplicatively — `albedo * light * (lit_factor * ao)`
//! — so a roofed interior (no directional term, hemisphere-only
//! light) netted ~0.07–0.15× and rendered near-black even when
//! the game reported the tile lit (LOW at noon: raster renders
//! the same data as a warm wood floor, RT netted ~(17,22,24)).
//!
//! Option B is the industry shape (probes carry interiors):
//! 1. the indirect (hemisphere) term carries a floor — the cheap
//!    stand-in for an interior probe — and the floor is the low
//!    end of the game's own lit band for the tile
//!    (`lighting::mapping::band` lows: DARK .06 / LOW .18 /
//!    BRIGHT_ONLY .34 / LIT .54 / BRIGHT .80), tinted by the
//!    hemisphere colour, applying only where the tile takes no
//!    sky (not sky-exposed);
//! 2. occupancy AO scopes to the indirect term only — it no
//!    longer darkens direct light;
//! 3. the game's lit band caps the *sum* (direct + indirect)
//!    — a luminance cap, ratio-applied so hue survives —
//!    instead of multiplying the parts.
//!
//! Every assertion below was checked against the pre-fix shader
//! arithmetic (recorded in the test comments) and fails on it.

use glam::Vec3;
use lighting::{sky_state, GameEnvironment, SkyUniforms};
use treebuild::build::{build_tree, BuildConfig};
use treebuild::render::{RtCamera, RtRenderer};
use treebuild::world::{VoxelCell, VoxelWorld, LIT_BRIGHT, LIT_LOW};

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
            label: Some("interior-light-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu { device, queue, name })
}

fn read_rgba(gpu: &Gpu, texture: &wgpu::Texture, size: (u32, u32)) -> Vec<u8> {
    let (w, h) = size;
    let unpadded = w * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("interior-gate-readback"),
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
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
        label: Some("interior-gate-depth"),
        size: (count * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (count * 4) as u64);
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let data = slice.get_mapped_range();
    let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    out
}

fn pixel_rd(camera: &RtCamera, size: (u32, u32), x: u32, y: u32) -> Vec3 {
    let fwd = (camera.target - camera.eye).normalize_or_zero();
    let right = fwd.cross(Vec3::Z).normalize_or_zero();
    let up = right.cross(fwd);
    let tan = (camera.fov_y_deg.to_radians() * 0.5).tan();
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let ndc = glam::Vec2::new((px / size.0 as f32) * 2.0 - 1.0, 1.0 - (py / size.1 as f32) * 2.0);
    (fwd + ndc.x * tan * camera.aspect * right + ndc.y * tan * up).normalize()
}

fn uniforms_at(time_of_day_sec: f32, moon_phase: u8) -> SkyUniforms {
    let env = GameEnvironment::fake_day(time_of_day_sec, 21_600.0, 64_800.0, 1, moon_phase);
    SkyUniforms::from_sky_state(&sky_state(&env), time_of_day_sec).to_grid_space()
}

fn render_median_top_lum(
    gpu: &Gpu,
    world: &VoxelWorld,
    uniforms: &SkyUniforms,
    camera: &RtCamera,
    size: (u32, u32),
    top_tile: Option<(i32, i32)>,
) -> f32 {
    let built = build_tree(world, BuildConfig::default()).unwrap();
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    scene.set_sky(&gpu.queue, uniforms);
    let frame = renderer.render(&gpu.device, &gpu.queue, &scene, camera, size, true);
    gpu.device.poll(wgpu::Maintain::Wait);
    let px = read_rgba(gpu, &frame.color, size);
    let depth = read_depth(gpu, &frame.depth, (size.0 * size.1) as usize);
    let mut lums: Vec<f32> = Vec::new();
    for y in 0..size.1 {
        for x in 0..size.0 {
            let p = (y * size.0 + x) as usize;
            if depth[p] >= frame.max_dist {
                continue;
            }
            let rd = pixel_rd(camera, size, x, y);
            let hitp = camera.eye + rd * depth[p];
            if (hitp.z - 1.0).abs() >= 0.05 {
                continue; // top faces of z=0 cells only
            }
            if let Some((tx, ty)) = top_tile {
                if hitp.x.floor() as i32 != tx || hitp.y.floor() as i32 != ty {
                    continue;
                }
            }
            let i = p * 4;
            lums.push((0.299 * px[i] as f32 + 0.587 * px[i + 1] as f32 + 0.114 * px[i + 2] as f32) / 255.0);
        }
    }
    assert!(lums.len() > 100, "gate scene must show >100 top-face px, got {}", lums.len());
    lums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    lums[lums.len() / 2]
}

/// A roofed floor: every cell not sky-exposed, game-lit LOW,
/// white albedo (so the measured luminance IS the light term).
fn interior_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    for y in 0..24 {
        for x in 0..24 {
            let mut c = VoxelCell::new([1.0, 1.0, 1.0], 1);
            c.lit = LIT_LOW;
            c.sky_exposed = false;
            w.set([x, y, 0], c);
        }
    }
    w
}

#[test]
fn option_b_interior_floor_and_cap() {
    let Some(gpu) = gpu() else {
        eprintln!("interior gate: no wgpu adapter — NOT RUN");
        return;
    };
    println!("interior gate adapter: {}", gpu.name);
    let camera = RtCamera {
        eye: Vec3::new(12.0, -10.0, 14.0),
        target: Vec3::new(12.0, 12.0, 0.0),
        fov_y_deg: 60.0,
        aspect: 4.0 / 3.0,
    };
    let size = (320u32, 240u32);
    let world = interior_world();

    // Noon, LOW, roofed. Pre-fix: albedo * (zenith hemi * 1.0)
    // * lit_factor(LOW)=0.45 * ao=1 -> luminance ~0.19.
    // Option B: indirect = hemi (lum ~0.46), luminance-capped
    // at the LOW ceiling 0.45 -> luminance ~0.45.
    let noon = render_median_top_lum(&gpu, &world, &uniforms_at(43_200.0, 0), &camera, size, None);
    println!("interior noon LOW median lum {noon:.3}");
    assert!(noon >= 0.32, "noon LOW interior must read as a lit room, not a crush (median lum {noon:.3}; pre-fix ~0.19)");

    // Midnight, LOW, roofed (the lamp-lit room): the sky's
    // hemisphere is ~0.01 here, so only the probe floor — the
    // game's own LOW band low, 0.18 — can carry it.
    // Pre-fix: ~0.005. Option B: ~0.18 by construction.
    let midnight = render_median_top_lum(&gpu, &world, &uniforms_at(0.0, 4), &camera, size, None);
    println!("interior midnight LOW median lum {midnight:.3}");
    assert!(midnight >= 0.10, "midnight LOW interior must be carried by the probe floor (median lum {midnight:.3}; pre-fix ~0.005)");
}

#[test]
fn option_b_ao_scopes_to_indirect_only() {
    let Some(gpu) = gpu() else {
        eprintln!("interior gate: no wgpu adapter — NOT RUN");
        return;
    };
    // One BRIGHT, sky-exposed floor tile in a pit: four cardinal
    // neighbours at z=1 make its top-face occupancy AO read
    // occ=4 -> ao=0.6. Neutral uniforms with the sun moved
    // straight overhead (a cardinal sun would be physically
    // shadowed by the pit's own walls): direct on the top face
    // 1.0, indirect (zenith) luminance ~0.46.
    // Pre-fix: (direct + indirect) * 1.0 * 0.6 -> lum ~0.87.
    // Option B: direct untouched by AO; indirect * 0.6; the sum
    // caps at BRIGHT 1.0 per channel -> lum 1.0.
    let mut w = VoxelWorld::new();
    let mut floor = VoxelCell::new([1.0, 1.0, 1.0], 1);
    floor.lit = LIT_BRIGHT;
    w.set([16, 16, 0], floor);
    for (x, y) in [(17, 16), (15, 16), (16, 17), (16, 15)] {
        let mut wall = VoxelCell::new([0.4, 0.4, 0.4], 2);
        wall.lit = LIT_BRIGHT;
        w.set([x, y, 1], wall);
    }
    let camera = RtCamera {
        eye: Vec3::new(16.5, 13.0, 11.0),
        target: Vec3::new(16.5, 16.5, 0.5),
        fov_y_deg: 60.0,
        aspect: 4.0 / 3.0,
    };
    let mut overhead = SkyUniforms::neutral();
    overhead.sun_dir = [0.0, 0.0, 1.0, 0.0];
    let lum = render_median_top_lum(&gpu, &w, &overhead, &camera, (320, 240), Some((16, 16)));
    println!("pit tile (ao=0.6, direct sun) median lum {lum:.3}");
    assert!(lum >= 0.96, "AO must not darken the direct term (median lum {lum:.3}; pre-fix ~0.87)");
}
