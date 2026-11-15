//! Shared wgpu rendering code used by BOTH the windowed app and the
//! headless capture mode (plan §7: a capture is only evidence about the
//! real renderer if both paths share pipeline, mesh upload, and draw
//! code — everything in this module is that shared code).
//!
//! Flat colors per block with a simple directional shade; one pipeline
//! per mesher pass (opaque / cutout = opaque pipeline state, transparent
//! = alpha blend, no depth write).

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use mesher::block::Pass;
use mesher::mesh::{Mesh, PassMeshes};

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.08,
    g: 0.10,
    b: 0.14,
    a: 1.0,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

const SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(input.position, 1.0);
    out.normal = input.normal;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(input.normal);
    let l = normalize(vec3<f32>(0.3, 0.8, 0.4));
    let shade = 0.45 + 0.55 * max(dot(n, l), 0.0);
    return vec4<f32>(input.color.rgb * shade, input.color.a);
}
"#;

/// One uploaded mesh for one pass.
pub struct GpuMesh {
    pub pass: Pass,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: u32,
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    blend: Option<wgpu::BlendState>,
    depth_write: bool,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("voxel-pipeline-layout"),
        bind_group_layouts: &[uniform_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("voxel-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: depth_write,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
        cache: None,
    })
}

/// The shared renderer: pipelines per pass + the camera uniform.
pub struct Renderer {
    pipelines: [wgpu::RenderPipeline; 3],
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// `format` is the surface format (windowed) or `CAPTURE_FORMAT`
    /// (headless) — the only difference between the two paths.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera-uniform-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera-uniform"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera-uniform-bg"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let opaque = create_pipeline(device, format, &uniform_layout, &shader, None, true);
        let transparent = create_pipeline(
            device,
            format,
            &uniform_layout,
            &shader,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
        );
        Self {
            // Cutout shares the opaque pipeline in the MVP (fake-data
            // cutout colors are fully opaque; alpha testing arrives
            // with real textures in Phase 5).
            pipelines: [opaque.clone(), opaque, transparent],
            uniform_buffer,
            uniform_bind_group,
        }
    }

    /// Upload mesher output to GPU buffers (shared by both paths).
    pub fn upload(&self, device: &wgpu::Device, meshes: &PassMeshes) -> Vec<GpuMesh> {
        use wgpu::util::DeviceExt;
        let mut out = Vec::new();
        for pass in Pass::ALL {
            let mesh: &Mesh = meshes.get(pass);
            if mesh.is_empty() {
                continue;
            }
            let mut verts = Vec::with_capacity(mesh.vertex_count());
            for i in 0..mesh.vertex_count() {
                verts.push(Vertex {
                    position: mesh.positions[i],
                    normal: mesh.normals[i],
                    color: mesh.colors[i],
                });
            }
            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("chunk-vertices"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("chunk-indices"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            out.push(GpuMesh {
                pass,
                vertices,
                indices,
                index_count: mesh.indices.len() as u32,
            });
        }
        out
    }

    pub fn set_view_projection(&self, queue: &wgpu::Queue, view_proj: Mat4) {
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Record and submit one frame drawing `meshes` into `color_view`
    /// with `depth_view` (shared by both paths).
    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        meshes: &[GpuMesh],
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame-encoder"),
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
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
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
            for mesh in meshes {
                rpass.set_pipeline(&self.pipelines[mesh.pass.index()]);
                rpass.set_vertex_buffer(0, mesh.vertices.slice(..));
                rpass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }
        queue.submit([encoder.finish()]);
    }
}

/// Create a depth texture view sized `size` (windowed resize and
/// headless capture both use this).
pub fn create_depth_view(device: &wgpu::Device, size: (u32, u32)) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth-texture"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&Default::default())
}
