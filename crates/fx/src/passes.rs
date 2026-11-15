//! Reusable GPU passes for engine integration (Phase 2g.1 — the
//! follow-up recorded in docs/phase2b-effects.md §6).
//!
//! [`FieldPass`] and [`PostPass`] lift the two passes out of the
//! standalone harness in [`crate::render`] so the real engine can
//! run them over its own rendered world: field volumes after the
//! opaque world (depth-tested, no depth write), the HD-2D post
//! stack as the final world pass before any UI compositing.
//!
//! The engine's field pass is the SOFT-PARTICLE variant of the
//! field shader: it additionally binds the scene depth (a copy —
//! a pass cannot sample its own depth attachment) and fades each
//! volume's alpha where the volume surface approaches the scene
//! surface, so volumes no longer show a hard intersection line
//! against floors and walls. The base [`crate::render::FIELD_SHADER`]
//! is untouched — it still binds no texture at all (gate 2b.2);
//! the soft variant is *derived* from it by the anchored edits in
//! [`soft_field_shader`], and the derivation is test-pinned.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

use crate::density::effective_density;
use crate::effect::EffectDef;
use crate::post::PostConfig;
use crate::render::{FieldInstGpu, PostUniforms, FIELD_SHADER, POST_SHADER};
use crate::scene::PlacedField;

/// Uniform block for the soft field pass: the harness's
/// `Uniforms` plus the soft-particle fade band, in projected-depth
/// units (see [`soft_fade_z`]).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FieldUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub eye: [f32; 3],
    pub time: f32,
    /// x: fade band in projected (0..1) depth; yzw unused.
    pub soft_fade: [f32; 4],
}

impl FieldUniforms {
    pub fn new(view_proj: Mat4, eye: Vec3, time: f32, soft_fade_z: f32) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            eye: eye.to_array(),
            time,
            soft_fade: [soft_fade_z, 0.0, 0.0, 0.0],
        }
    }
}

/// Projected-depth value of a point `dist` from the camera, for
/// the engine/fx projection (near 0.1, far 600).
pub fn projected_depth(dist: f32) -> f32 {
    let (near, far) = (0.1f32, 600.0f32);
    let d = dist.max(near + 0.001);
    far * (d - near) / ((far - near) * d)
}

/// The soft-particle fade band in projected-depth units: how much
/// projected depth `world_band` world units span at `distance`.
/// Projected depth is nonlinear, so the band must be computed at
/// the working distance — a fixed band would be metres wide at
/// capture range and invisible up close.
pub fn soft_fade_z(distance: f32, world_band: f32) -> f32 {
    (projected_depth(distance + world_band) - projected_depth(distance)).max(1e-6)
}

/// Derive the soft-particle field shader from [`FIELD_SHADER`]:
/// extended uniforms, a scene-depth binding, and an `fs_main`
/// wrapper that scales the field fragment's alpha by its depth
/// separation from the scene. Panics if an anchor is missing —
/// a silent no-op derivation would ship the hard-edged shader
/// under the soft name.
pub fn soft_field_shader() -> String {
    let mut s = FIELD_SHADER.to_string();
    let anchors = [
        (
            "struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };",
            "struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32, soft_fade: vec4<f32> };",
        ),
        (
            "@group(0) @binding(1) var<storage, read> fields: array<FieldInst>;",
            "@group(0) @binding(1) var<storage, read> fields: array<FieldInst>;\n@group(0) @binding(2) var scene_depth: texture_depth_2d;",
        ),
        // fs_main stays the entry point (WGSL entry points cannot
        // be called, and non-entry functions cannot carry
        // @location returns) — instead, each of its three returns
        // is wrapped in the soften() helper.
        (
            "return vec4<f32>(fcol, falpha);",
            "return soften(vec4<f32>(fcol, falpha), input.clip);",
        ),
        (
            "return smoke_shade(f, p, dir_l, t_enter, t_exit);",
            "return soften(smoke_shade(f, p, dir_l, t_enter, t_exit), input.clip);",
        ),
        (
            "return vec4<f32>(col, alpha);",
            "return soften(vec4<f32>(col, alpha), input.clip);",
        ),
    ];
    for (from, to) in anchors {
        assert!(s.contains(from), "soft shader anchor missing: {from}");
        s = s.replacen(from, to, 1);
    }
    s.push_str(
        r#"
// Soft-particle fade (engine variant only): scale a field
// fragment's alpha by its projected-depth separation from the
// scene surface behind it, so volumes meet floors and walls
// without a hard intersection line.
fn soften(c: vec4<f32>, clip: vec4<f32>) -> vec4<f32> {
    if (c.a <= 0.0) { return c; }
    let dims = vec2<i32>(textureDimensions(scene_depth));
    let px = clamp(vec2<i32>(clip.xy), vec2<i32>(0), dims - vec2<i32>(1));
    let scene_z = textureLoad(scene_depth, px, 0);
    let soft = clamp((scene_z - clip.z) / max(u.soft_fade.x, 1e-6), 0.0, 1.0);
    return vec4<f32>(c.rgb, c.a * soft);
}
"#,
    );
    s
}

/// Build GPU field instances for `fields` with a caller-supplied
/// placement: `center` maps a placed field to its world-space
/// volume center (each renderer scales tiles its own way — the
/// harness uses [`PlacedField::world_center`], the engine uses
/// its 3-blocks-per-z-level scale), `size` is the volume size.
/// Heat/link coupling is computed here, from the full field list,
/// exactly as the harness computes it (see
/// [`crate::render::heat_of`] / [`crate::render::link_of`]).
/// Input order is preserved; callers sort back-to-front.
pub fn build_instances(
    fields: &[PlacedField],
    size: f32,
    center: impl Fn(&PlacedField) -> Vec3,
) -> Vec<FieldInstGpu> {
    fields
        .iter()
        .map(|f| {
            let def: &EffectDef = &f.def;
            FieldInstGpu {
                center: center(f).to_array(),
                size,
                base_color: def.base_color,
                density: effective_density(def, f.intensity),
                emissive: def.emissive_color,
                emissive_strength: def.emissive_strength,
                noise_scale: def.noise_scale,
                noise_speed: def.noise_speed,
                class_id: def.render_class.id(),
                seed: def.seed(),
                drift: def.world_drift().to_array(),
                heat: crate::render::heat_of(f, fields),
                link: crate::render::link_of(f, fields),
                _pad: [0.0; 3],
            }
        })
        .collect()
}

/// The instanced field-volume pass (soft-particle variant),
/// drawn into an existing color+depth pair with Load ops.
pub struct FieldPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

/// A prepared field draw: instance bind group + instance count.
pub struct PreparedFields {
    bind_group: wgpu::BindGroup,
    pub count: u32,
}

impl FieldPass {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx-field-soft-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fx-field-soft-shader"),
            source: wgpu::ShaderSource::Wgsl(soft_field_shader().into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fx-field-soft-pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fx-field-soft-pl"),
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        Self { pipeline, layout }
    }

    /// Upload instances + uniforms and bind the scene-depth copy.
    /// `scene_depth_view` must be a COPY of the depth the field
    /// pass depth-tests against (a pass cannot sample its own
    /// depth attachment).
    pub fn prepare(
        &self,
        device: &wgpu::Device,
        instances: &[FieldInstGpu],
        uniforms: &FieldUniforms,
        scene_depth_view: &wgpu::TextureView,
    ) -> PreparedFields {
        use wgpu::util::DeviceExt;
        // wgpu rejects zero-sized buffers; bind one dummy instance
        // and draw zero of them (same guard as the harness).
        let owned;
        let slice: &[FieldInstGpu] = if instances.is_empty() {
            owned = vec![FieldInstGpu {
                center: [0.0; 3],
                size: 0.0,
                base_color: [0.0; 3],
                density: 0.0,
                emissive: [0.0; 3],
                emissive_strength: 0.0,
                noise_scale: 1.0,
                noise_speed: 0.0,
                class_id: 5,
                seed: 0.0,
                drift: [0.0; 3],
                heat: 0.0,
                link: 0.0,
                _pad: [0.0; 3],
            }];
            &owned
        } else {
            instances
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fx-field-soft-uniforms"),
            contents: bytemuck::bytes_of(uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let inst_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fx-field-soft-instances"),
            contents: bytemuck::cast_slice(slice),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fx-field-soft-bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: inst_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(scene_depth_view),
                },
            ],
        });
        PreparedFields {
            bind_group,
            count: instances.len() as u32,
        }
    }

    /// Draw the prepared fields over an already-rendered world in
    /// `color_view`/`depth_view` (both loaded, neither cleared).
    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedFields,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        if prepared.count == 0 {
            return;
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fx-field-soft-encoder"),
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fx-field-soft-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &prepared.bind_group, &[]);
            rpass.draw(0..36, 0..prepared.count);
        }
        queue.submit([encoder.finish()]);
    }
}

/// The fused HD-2D post pass (bloom / tilt-shift DoF / warm
/// grade+vignette / chromatic offset, per-pass toggles in
/// [`PostConfig`]), drawn as the final world pass — UI is
/// composited by the caller AFTER this, never inside it
/// (gate 2b.7).
pub struct PostPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl PostPass {
    pub fn new(device: &wgpu::Device, out_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fx-post-layout"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fx-post-shader"),
            source: wgpu::ShaderSource::Wgsl(POST_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fx-post-pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fx-post-pl"),
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: out_format,
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
        Self { pipeline, layout }
    }

    /// Run the stack: reads `color_view` + `depth_view`, writes
    /// `out_view`. `focal_projected_depth` is the DoF sharp band
    /// in projected-depth units ([`projected_depth`] of the
    /// camera's follow distance).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        cfg: &PostConfig,
        focal_projected_depth: f32,
        size: (u32, u32),
    ) {
        use wgpu::util::DeviceExt;
        let uniforms = PostUniforms {
            flags: [
                cfg.bloom as u32,
                cfg.dof as u32,
                cfg.grade as u32,
                cfg.chromatic as u32,
            ],
            params: [
                cfg.bloom_threshold,
                cfg.bloom_strength,
                focal_projected_depth,
                cfg.dof_strength,
            ],
            texel: [1.0 / size.0.max(1) as f32, 1.0 / size.1.max(1) as f32],
            _p: [0.0; 2],
        };
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fx-post-uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fx-post-bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ubuf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fx-post-encoder"),
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fx-post-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out_view,
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
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::demo_defs;

    #[test]
    fn soft_shader_derivation_anchors_hold() {
        let soft = soft_field_shader();
        assert!(soft.contains("texture_depth_2d"));
        assert!(soft.contains("fn soften("));
        assert!(soft.contains("soft_fade"));
        // The base shader stays texture-free (gate 2b.2's object).
        assert!(!FIELD_SHADER.contains("texture"));
    }

    #[test]
    fn fade_band_shrinks_with_distance() {
        // Projected depth compresses with distance: the same
        // 0.35-world-unit band is a smaller projected delta far
        // away than near.
        assert!(soft_fade_z(8.0, 0.35) > soft_fade_z(28.0, 0.35));
        assert!(soft_fade_z(28.0, 0.35) > 0.0);
    }

    #[test]
    fn build_instances_keeps_heat_and_link_coupling() {
        let defs = demo_defs();
        let get = |id: &str| defs.iter().find(|d| d.field_id == id).unwrap().clone();
        let fields = vec![
            PlacedField { tile: [6, 6, 0], def: get("fd_fire"), intensity: 3 },
            PlacedField { tile: [6, 6, 0], def: get("fd_smoke"), intensity: 3 },
            PlacedField { tile: [6, 6, 1], def: get("fd_smoke"), intensity: 2 },
            PlacedField { tile: [9, 9, 0], def: get("fd_smoke"), intensity: 1 },
        ];
        let inst = build_instances(&fields, 1.6, |f| f.world_center());
        assert_eq!(inst.len(), 4);
        // Smoke on the fire tile is fully heated; the z+1 smoke is
        // heated less; the far standalone smoke is unheated.
        assert_eq!(inst[1].heat, 1.0);
        assert!(inst[2].heat > 0.0 && inst[2].heat < inst[1].heat);
        assert_eq!(inst[3].heat, 0.0);
        // Column links: lower smoke has smoke above (2), upper has
        // smoke below (1), standalone has neither (0).
        assert_eq!(inst[1].link, 2.0);
        assert_eq!(inst[2].link, 1.0);
        assert_eq!(inst[3].link, 0.0);
        // Caller placement/size are honored.
        assert_eq!(inst[0].size, 1.6);
        assert_eq!(inst[0].center, fields[0].world_center().to_array());
    }
}
