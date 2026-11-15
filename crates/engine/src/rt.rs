//! Phase 2c engine integration: the ray-traced renderer behind
//! `--renderer rt` (raster stays the default until the 2c gate's
//! target-hardware frame-time evidence exists — see
//! docs/phase2c-rt.md).
//!
//! The RT world is built from the *same* cutaway-filtered chunk map
//! as the raster path (`world::filtered_map`), so cutaway selection
//! is identical by construction (gate 2c.6). The camera is the same
//! `OrbitCamera`: eye/target are converted world -> grid inside
//! `treebuild::render::RtCamera::from_world`, with the raster path's
//! 60-degree vertical FOV.

use std::time::Instant;

use glam::Vec3;
use treebuild::build::{build_tree, BuildConfig, BuiltTree};
use treebuild::render::{RtCamera, RtRenderer};
use treebuild::world::VoxelWorld;

use crate::camera::OrbitCamera;
use crate::capture::{CaptureArgs, CaptureError};
use crate::cutaway::CutawayMode;
use crate::world;

/// Fake-data material resolution: mesher fake tile id -> UltiCa
/// game id whose voxelizer material stands in for it. Every id here
/// was verified present in the real UltiCa `materials.json`
/// (2026-11-15). Real scenes resolve through TileDefs instead —
/// this table exists so the fake house can exercise the texture
/// path end to end.
pub fn fake_material_ids() -> std::collections::HashMap<u32, String> {
    use mesher::block::*;
    [
        (TILE_FLOOR_WOOD, "t_floor_primitive"), // rough plank floor
        (TILE_WALL, "t_wall"),
        (TILE_DOOR, "t_door_c"),
        (TILE_WINDOW, "t_window"),
        (TILE_ROOF, "t_shingle_flat_roof"),
        (TILE_GROUND, "t_grass"),
        (TILE_FENCE, "t_chainfence"),
        (TILE_WATER, "t_water_dp"),
        (TILE_FOLIAGE, "t_tree"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.to_string()))
    .collect()
}

/// Attach voxelizer textures from `dir` to `scene`, resolving the
/// fake house's material ids. Returns the build stats.
pub fn attach_fake_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &mut treebuild::render::RtScene,
    dir: &std::path::Path,
) -> Result<crate::textures::TextureStats, CaptureError> {
    let table = crate::materials::MaterialTable::load(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let (spec, stats) = crate::textures::build_texture_spec(&table, dir, &fake_material_ids())
        .map_err(CaptureError::Device)?;
    scene
        .attach_textures(device, queue, &spec)
        .map_err(CaptureError::Device)?;
    Ok(stats)
}

/// The fake-house voxel world for a set of visible game z-levels.
pub fn world_for_levels(visible_levels: &[i32]) -> VoxelWorld {
    let map = world::build_fake_house(mesher::scale::WorldScale::default());
    let filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    VoxelWorld::from_chunk_map(&filtered)
}

/// Visible levels the capture path uses for a player z (same
/// selection as the raster capture: HideAbove(0)).
pub fn capture_visible_levels(cutaway_z: i32) -> Vec<i32> {
    CutawayMode::HideAbove(0).visible_levels(cutaway_z)
}

/// Build the tree for the capture at `cutaway_z`.
pub fn build_capture_tree(cutaway_z: i32) -> Result<BuiltTree, CaptureError> {
    let world = world_for_levels(&capture_visible_levels(cutaway_z));
    build_tree(&world, BuildConfig::default())
        .map_err(|e| CaptureError::Device(format!("treebuild: {e}")))
}

/// The RT camera framing exactly the raster capture's frame.
pub fn capture_camera(args: &CaptureArgs, aspect: f32) -> RtCamera {
    let mut camera = OrbitCamera::new(Vec3::new(args.camera_target.0, 1.0, -args.camera_target.1));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    RtCamera::from_world(camera.eye(), camera.target, 60.0, aspect)
}

/// One RT capture's full result (the PNG pixels plus the depth
/// buffer the parity gates classify silhouettes with).
pub struct RtCapture {
    pub image: image::RgbaImage,
    pub depth: Vec<f32>,
    pub max_dist: f32,
    pub size: (u32, u32),
    pub leaves: usize,
    pub nodes: usize,
    pub build_ms: f64,
    pub distinct_colors: usize,
    /// Texture hookup stats when `args.textures` was set.
    pub texture_stats: Option<crate::textures::TextureStats>,
}

/// Render one RT capture on an already-created device/queue.
pub fn capture_rt_with_device(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> Result<RtCapture, CaptureError> {
    let t0 = Instant::now();
    let built = build_capture_tree(args.cutaway_z)?;
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let (renderer, mut scene) = RtRenderer::with_scene(device, queue, &built);
    let texture_stats = match &args.textures {
        Some(dir) => Some(attach_fake_textures(device, queue, &mut scene, dir)?),
        None => None,
    };
    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
    let aspect = width as f32 / height as f32;
    let camera = capture_camera(args, aspect);
    let frame = renderer.render(device, queue, &scene, &camera, (width, height), true);
    device.poll(wgpu::Maintain::Wait);

    let pixels = read_texture_rgba(device, queue, &frame.color, (width, height))?;
    let depth = read_buffer_f32(device, queue, &frame.depth, (width * height) as usize)?;
    let distinct: std::collections::HashSet<[u8; 4]> =
        pixels.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect();
    let image = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| CaptureError::Readback("pixel buffer size mismatch".into()))?;
    Ok(RtCapture {
        image,
        depth,
        max_dist: frame.max_dist,
        size: (width, height),
        leaves: built.leaf_count(),
        nodes: built.node_count(),
        build_ms,
        distinct_colors: distinct.len(),
        texture_stats,
    })
}

/// Standalone RT capture: creates its own instance/adapter/device
/// (same clean `NoAdapter` failure as the raster capture).
pub fn capture_rt_standalone(args: &CaptureArgs) -> Result<RtCapture, CaptureError> {
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
            .await
            .ok_or(CaptureError::NoAdapter)?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rt-capture-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| CaptureError::Device(e.to_string()))?;
        capture_rt_with_device(&device, &queue, args)
    })
}

fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Result<Vec<u8>, CaptureError> {
    let (width, height) = size;
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rt-capture-staging"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("rt-capture-copy"),
    });
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
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| CaptureError::Readback("map channel closed".into()))?
        .map_err(|e| CaptureError::Readback(e.to_string()))?;
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..height as usize {
            let start = row * padded as usize;
            pixels.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    Ok(pixels)
}

fn read_buffer_f32(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    count: usize,
) -> Result<Vec<f32>, CaptureError> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rt-depth-staging"),
        size: (count * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("rt-depth-copy"),
    });
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (count * 4) as u64);
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| CaptureError::Readback("map channel closed".into()))?
        .map_err(|e| CaptureError::Readback(e.to_string()))?;
    let data = slice.get_mapped_range();
    let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    Ok(out)
}

// ---------------------------------------------------------------------
// Windowed blit: present an RT storage texture on a surface.
// ---------------------------------------------------------------------

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi & 1u) * 4 - 1);
    let y = f32(i32(vi & 2u) * 2 - 1);
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, input.uv);
    // The RT texture holds display-ready (sRGB-encoded) values; the
    // surface pipeline treats fragment output as linear and encodes
    // on write, so pre-apply the approximate inverse transfer.
    return vec4<f32>(pow(c.rgb, vec3<f32>(2.2, 2.2, 2.2)), 1.0);
}
"#;

/// Fullscreen-triangle blit of the RT color texture onto a surface.
pub struct RtBlit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl RtBlit {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rt-blit-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt-blit-pipeline-layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rt-blit-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rt-blit-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &wgpu::Texture,
        dst_view: &wgpu::TextureView,
    ) {
        let src_view = src.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rt-blit-bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rt-blit-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rt-blit-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}
