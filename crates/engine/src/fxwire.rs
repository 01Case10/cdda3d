//! Engine wiring for the fx crate (Phase 2g.1): fake-house field
//! placement in the engine's world scale, effect-def loading,
//! and the RT depth conversion the field/post passes need.
//!
//! Frame order (asserted by structure in `capture.rs`): opaque
//! world → field volumes (soft-particle pass, depth-tested, no
//! depth write) → post stack → readback/UI. The post stack is
//! always the final world pass; UI compositing (Phase 4) lands
//! after it, never inside it (gate 2b.7).
//!
//! The raster path's depth is a real `Depth32Float` texture. The
//! RT path's tracer instead produces a buffer of linear ray
//! distances; [`RtDepthConvert`] turns that buffer into the same
//! projected-depth texture the passes consume (view depth =
//! ray distance × cos(angle to the view axis), then the engine
//! projection's depth mapping, near 0.1 / far 600).

use std::path::Path;

use glam::Vec3;

use fx::effect::{EffectDef, RenderClass};
use fx::render::FieldInstGpu;
use fx::scene::PlacedField;
use treebuild::render::RtCamera;

/// Volume size for a field on one engine tile. The engine's
/// z-level is 3 blocks tall with the floor slab in the bottom
/// block, so a ground field sits in the 2-block air column above
/// the floor; 1.6 keeps the volume inside its tile's column
/// while reading at the fx harness's visual scale.
pub const FIELD_SIZE: f32 = 1.6;

/// World-space center of a field volume on game tile
/// `(x east, y south, z-level)`, engine scale: floor top at
/// y = 1 + 3z, volume center 0.8 above it.
pub fn engine_center(f: &PlacedField) -> Vec3 {
    Vec3::new(
        f.tile[0] as f32 + 0.5,
        1.8 + 3.0 * f.tile[2] as f32,
        -(f.tile[1] as f32 + 0.5),
    )
}

/// Load effect definitions from a voxelizer `effects.json`.
pub fn load_defs(path: &Path) -> Result<Vec<EffectDef>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("reading {}: {e}", path.display()))?;
    fx::effect::load_effects_json(&text)
}

/// The fake-house field scene, in the Phase 3 exporter's shape
/// (per-tile field lists, one displayed field per tile): a fire
/// inside the house with its coupled smoke column (same tile +
/// the z-level above), a gas tile, a standalone thin smoke, and
/// a small fire. Defs are selected by render class, so any
/// `effects.json` whose extraction covers the three headline
/// classes works (the voxelizer's real-UltiCa run covers all).
pub fn fake_house_fields(defs: &[EffectDef]) -> Result<Vec<PlacedField>, String> {
    // Preferred ids first (the real extraction splits fire into
    // per-intensity defs — fd_fire_intN is the game's fd_fire at
    // intensity N, so the scene pairs def and intensity), then
    // any def of the class.
    let pick = |class: RenderClass, prefs: &[&str]| -> Result<EffectDef, String> {
        for id in prefs {
            if let Some(d) = defs.iter().find(|d| &d.field_id == id) {
                return Ok(d.clone());
            }
        }
        defs.iter()
            .find(|d| d.render_class == class)
            .cloned()
            .ok_or_else(|| format!("effects contain no {class:?} def"))
    };
    let fire_big = pick(RenderClass::Flame, &["fd_fire_int3", "fd_fire"])?;
    let fire_small = pick(RenderClass::Flame, &["fd_fire_int1", "fd_fire"])?;
    let smoke = pick(RenderClass::Smoke, &["fd_smoke", "fd_smoke_overlay"])?;
    let gas = pick(RenderClass::Gas, &["fd_gas", "fd_fungal_haze_int2"])?;
    let at = |tile: [i32; 3], def: EffectDef, intensity: u8| PlacedField {
        tile,
        def,
        intensity,
    };
    Ok(vec![
        at([6, 6, 0], fire_big, 3),
        at([6, 6, 0], smoke.clone(), 3),
        at([6, 6, 1], smoke.clone(), 2),
        at([3, 3, 0], gas, 2),
        at([8, 8, 0], smoke, 1),
        at([4, 8, 0], fire_small, 1),
    ])
}

/// GPU instances for `fields`, sorted back-to-front from `eye`
/// (alpha compositing order), with heat/link coupling computed
/// by fx from the full field list.
pub fn sorted_instances(fields: &[PlacedField], eye: Vec3) -> Vec<FieldInstGpu> {
    let mut sorted: Vec<PlacedField> = fields.to_vec();
    sorted.sort_by(|a, b| {
        engine_center(b)
            .distance_squared(eye)
            .partial_cmp(&engine_center(a).distance_squared(eye))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fx::passes::build_instances(&sorted, FIELD_SIZE, engine_center)
}

/// Converts the RT tracer's linear depth buffer (ray distance
/// per pixel, `max_dist` on a miss) into a projected-depth
/// texture via a depth-only fullscreen pass writing
/// `@builtin(frag_depth)`.
pub struct RtDepthConvert {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

const RT_DEPTH_SHADER: &str = r#"
struct U {
    tan_aspect: vec2<f32>,   // tan(half fov y), aspect
    viewport: vec2<f32>,
    max_dist: f32, _p: f32,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read> dist: array<f32>;

@vertex fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(vi) - 1) * 2.0;
    let y = f32(i32(vi & 1u)) * 4.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

struct FsOut { @builtin(frag_depth) z: f32 };

@fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> FsOut {
    var out: FsOut;
    let px = vec2<i32>(pos.xy);
    let idx = px.y * i32(u.viewport.x) + px.x;
    let t = dist[idx];
    if (t >= u.max_dist) { out.z = 1.0; return out; }
    // Same ray construction as the tracer: view depth is the
    // ray distance projected onto the view axis.
    let ndc = vec2<f32>(
        (pos.x / u.viewport.x) * 2.0 - 1.0,
        1.0 - (pos.y / u.viewport.y) * 2.0
    );
    let ax = ndc.x * u.tan_aspect.x * u.tan_aspect.y;
    let ay = ndc.y * u.tan_aspect.x;
    let zv = t / sqrt(1.0 + ax * ax + ay * ay);
    let near = 0.1;
    let far = 600.0;
    out.z = clamp(far * (zv - near) / ((far - near) * zv), 0.0, 1.0);
    return out;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RtDepthUniforms {
    tan_aspect: [f32; 2],
    viewport: [f32; 2],
    max_dist: f32,
    _p: f32,
}

impl RtDepthConvert {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rt-depth-convert-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-depth-convert-shader"),
            source: wgpu::ShaderSource::Wgsl(RT_DEPTH_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rt-depth-convert-pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("rt-depth-convert-pl"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        Self { pipeline, layout }
    }

    /// Write the projected-depth texture for one RT frame into
    /// `depth_view` (cleared first).
    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        depth_buffer: &wgpu::Buffer,
        max_dist: f32,
        camera: &RtCamera,
        size: (u32, u32),
        depth_view: &wgpu::TextureView,
    ) {
        use wgpu::util::DeviceExt;
        let uniforms = RtDepthUniforms {
            tan_aspect: [
                (camera.fov_y_deg.to_radians() * 0.5).tan(),
                camera.aspect,
            ],
            viewport: [size.0 as f32, size.1 as f32],
            max_dist,
            _p: 0.0,
        };
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rt-depth-convert-uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rt-depth-convert-bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ubuf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: depth_buffer.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rt-depth-convert-encoder"),
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rt-depth-convert-pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}
