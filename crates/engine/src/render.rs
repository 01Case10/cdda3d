//! Shared wgpu rendering code used by BOTH the windowed app and the
//! headless capture mode (plan §7: a capture is only evidence about the
//! real renderer if both paths share pipeline, mesh upload, and draw
//! code — everything in this module is that shared code).
//!
//! Flat colors per block with a simple directional shade; one pipeline
//! per mesher pass (opaque, cutout = alpha-tested variant, transparent
//! = alpha blend, no depth write). Phase 2f: vertices also carry a UV
//! + texture-array layer; textured fragments sample the scene texture
//! array (group 1, bound via `Renderer::set_textures`; a dummy 1-layer
//! white set is bound until then), while vertices with the flat
//! sentinel layer take exactly the pre-2f flat shading path.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use mesher::block::Pass;
use mesher::mesh::{Mesh, PassMeshes, FLAT_LAYER};

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
    /// Phase 2f: texture UV (meaningless when `layer == FLAT_LAYER`).
    pub uv: [f32; 2],
    /// Phase 2f: texture-array layer, or `FLAT_LAYER` (u32::MAX).
    pub layer: u32,
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
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
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
@group(1) @binding(0) var tex_array: texture_2d_array<f32>;
@group(1) @binding(1) var tex_samp: sampler;
@group(1) @binding(2) var macro_array: texture_2d_array<f32>;
struct MacroParams { color_count: u32, debug_height: u32 };
@group(1) @binding(3) var<uniform> macro_params: MacroParams;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) layer: u32,
};
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) layer: u32,
};

@vertex
fn vs_main(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(input.position, 1.0);
    out.normal = input.normal;
    out.color = input.color;
    out.uv = input.uv;
    out.layer = input.layer;
    return out;
}

fn shade_fragment(input: VsOut, cutout: bool) -> vec4<f32> {
    let n = normalize(input.normal);
    let l = normalize(vec3<f32>(0.3, 0.8, 0.4));
    let shade = 0.45 + 0.55 * max(dot(n, l), 0.0);
    let flat = input.layer == 4294967295u;
    // Sample unconditionally (flat vertices read layer 0 and ignore
    // the result) so textureSample's implicit derivatives stay in
    // uniform control flow; the mip chain makes auto-LOD work.
    let sample_layer = select(i32(input.layer), 0, flat);
    let texel_tile = textureSample(tex_array, tex_samp, input.uv, sample_layer);
    // Stage F: macro vertices carry the mesher's packed layer word
    // (bit 31 set; FLAT_LAYER also has it set, hence the !flat
    // term). Their UVs are world-based (mesher-side), and the
    // sample comes from the separate macro array — its color half
    // normally, its height half (offset by color_count) in the
    // height debug view. Sampled unconditionally like the tile
    // array, selected after.
    let is_macro = !flat && (input.layer & 2147483648u) != 0u;
    let macro_idx = i32(input.layer & 16777215u);
    let macro_layer = select(
        macro_idx,
        macro_idx + i32(macro_params.color_count),
        macro_params.debug_height != 0u,
    );
    let texel_macro = textureSample(
        macro_array,
        tex_samp,
        input.uv,
        select(macro_layer, 0, !is_macro),
    );
    let texel = select(texel_tile, texel_macro, is_macro);
    if flat {
        // Exactly the pre-2f flat shading math (gate 2f.4).
        return vec4<f32>(input.color.rgb * shade, input.color.a);
    }
    if cutout && texel.a < 0.5 {
        discard;
    }
    return vec4<f32>(texel.rgb * shade, texel.a * input.color.a);
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    return shade_fragment(input, false);
}

@fragment
fn fs_cutout(input: VsOut) -> @location(0) vec4<f32> {
    return shade_fragment(input, true);
}
"#;

/// One uploaded mesh for one pass.
pub struct GpuMesh {
    pub pass: Pass,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: u32,
}

#[allow(clippy::too_many_arguments)]
fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_layout: &wgpu::BindGroupLayout,
    tex_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    blend: Option<wgpu::BlendState>,
    depth_write: bool,
    frag_entry: &'static str,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("voxel-pipeline-layout"),
        bind_group_layouts: &[uniform_layout, tex_layout],
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
            entry_point: Some(frag_entry),
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

/// Number of mip levels for a square texture of edge `size`
/// (full chain down to 1x1).
fn mip_level_count(size: u32) -> u32 {
    32 - size.max(1).leading_zeros()
}

/// One premultiplied-alpha 2x2 box downsample step (Phase 2f raster
/// mip chain; the RT path's Phase 2e chain is its own). Averaging in
/// premultiplied space keeps transparent texels from bleeding their
/// RGB into opaque neighbors. Odd edges clamp (tile sizes are
/// normally powers of two, so this is just belt-and-braces).
fn downsample_premul(src: &[u8], w: usize, h: usize) -> (Vec<u8>, usize, usize) {
    let nw = (w / 2).max(1);
    let nh = (h / 2).max(1);
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..nh {
        for x in 0..nw {
            let mut acc = [0f32; 4];
            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = (2 * x + dx).min(w - 1);
                    let sy = (2 * y + dy).min(h - 1);
                    let i = (sy * w + sx) * 4;
                    let a = src[i + 3] as f32 / 255.0;
                    acc[0] += src[i] as f32 * a;
                    acc[1] += src[i + 1] as f32 * a;
                    acc[2] += src[i + 2] as f32 * a;
                    acc[3] += src[i + 3] as f32;
                }
            }
            let o = (y * nw + x) * 4;
            let a_avg = acc[3] / 4.0; // 0..=255 scale
            out[o + 3] = a_avg.round().clamp(0.0, 255.0) as u8;
            if a_avg > 0.0 {
                // Un-premultiply: value = (premul average) / (alpha
                // fraction) = (acc[c] / 4) * 255 / a_avg.
                for c in 0..3 {
                    let premul_avg = acc[c] / 4.0; // value * alpha01, 0..=255 scale
                    let v = premul_avg * 255.0 / a_avg;
                    out[o + c] = v.round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    (out, nw, nh)
}

/// Full mip chain for one texture layer: level 0 is the layer itself.
fn build_mip_chain(layer: &[u8], tile_size: u32) -> Vec<(Vec<u8>, u32, u32)> {
    let mut chain = vec![(layer.to_vec(), tile_size, tile_size)];
    while chain.last().map(|(_, w, h)| (*w, *h)) != Some((1, 1)) {
        let (prev, w, h) = chain.last().unwrap();
        let (next, nw, nh) = downsample_premul(prev, *w as usize, *h as usize);
        chain.push((next, nw as u32, nh as u32));
    }
    chain
}

/// The shared renderer: pipelines per pass + the camera uniform +
/// the scene texture array (group 1).
/// Stage F macro sampling parameters (group 1, binding 3):
/// how many color layers the macro array's first half holds (the
/// height half starts at that offset) and whether the height
/// debug view is on.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MacroParams {
    color_count: u32,
    debug_height: u32,
    _pad: [u32; 2],
}

pub struct Renderer {
    pipelines: [wgpu::RenderPipeline; 3],
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    tex_layout: wgpu::BindGroupLayout,
    tex_bind_group: wgpu::BindGroup,
    /// Keeps the texture array alive behind `tex_bind_group`'s view.
    _tex_texture: wgpu::Texture,
    /// Stage F: keeps the macro array alive behind the same bind
    /// group, plus the macro params buffer + current debug flag.
    _macro_texture: wgpu::Texture,
    macro_params_buffer: wgpu::Buffer,
    macro_debug_height: bool,
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
        let tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raster-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
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
                // Stage F: the macro array + its params (see the
                // shader's MacroParams).
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let opaque = create_pipeline(
            device,
            format,
            &uniform_layout,
            &tex_layout,
            &shader,
            None,
            true,
            "fs_main",
        );
        // Phase 2f: cutout gets its own pipeline whose fragment
        // entry alpha-tests the sampled texel (pre-2f it shared the
        // opaque pipeline, when cutout colors were fully opaque).
        let cutout = create_pipeline(
            device,
            format,
            &uniform_layout,
            &tex_layout,
            &shader,
            None,
            true,
            "fs_cutout",
        );
        let transparent = create_pipeline(
            device,
            format,
            &uniform_layout,
            &tex_layout,
            &shader,
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
            "fs_main",
        );
        // Dummy 1-layer texture set, bound until set_textures
        // installs a real scene set — the pipeline layout never
        // changes. `new` has no queue to upload with, so the dummy
        // layer is left at wgpu's zero-initialized contents; flat
        // vertices sample it and ignore the result, and no textured
        // vertex can exist before set_textures runs.
        let macro_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("raster-macro-params"),
            size: std::mem::size_of::<MacroParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let (_tex_texture, _macro_texture, tex_bind_group) =
            Self::make_tex_bind_group(device, &tex_layout, 1, 1, 1, 1, &macro_params_buffer);
        Self {
            pipelines: [opaque, cutout, transparent],
            uniform_buffer,
            uniform_bind_group,
            tex_layout,
            tex_bind_group,
            _tex_texture,
            _macro_texture,
            macro_params_buffer,
            macro_debug_height: false,
        }
    }

    /// Create a texture array plus its group-1 bind group (no data
    /// uploaded — see `upload_layers`).
    fn make_tex_bind_group(
        device: &wgpu::Device,
        tex_layout: &wgpu::BindGroupLayout,
        tile_size: u32,
        layer_count: usize,
        macro_size: u32,
        macro_layer_count: usize,
        macro_params: &wgpu::Buffer,
    ) -> (wgpu::Texture, wgpu::Texture, wgpu::BindGroup) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("raster-texture-array"),
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: layer_count.max(1) as u32,
            },
            mip_level_count: mip_level_count(tile_size),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        // Stage A (procedural texture pipeline rework): Nearest
        // magnification keeps pixel-art texels crisp up close (the
        // pre-fix Linear mag was the raster half of the "everything
        // is soft/blurry" symptom); Linear min + mip stay for
        // minification. `Repeat` address mode is required by the
        // mesher's tile-unit UVs (a merged quad spans [0, w] x
        // [0, h] and wraps once per tile).
        //
        // Deviation from the Stage A draft, forced by wgpu
        // validation (hit on the first Stage A capture run):
        // anisotropy > 1 requires ALL filter modes to be Linear,
        // so the draft's "Nearest mag + anisotropy 8x" cannot
        // exist in one sampler. Crisp magnification is the
        // headline fix, so anisotropy stays at 1 here; oblique
        // minification is carried by the Linear mip chain alone.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("raster-texture-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        // Stage F: the macro array shares the tile array's sampler
        // (Repeat wrap is what makes world-space macro UVs tile;
        // Nearest mag keeps the quilt crisp like the tiles).
        let macro_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("raster-macro-array"),
            size: wgpu::Extent3d {
                width: macro_size.max(1),
                height: macro_size.max(1),
                depth_or_array_layers: macro_layer_count.max(1) as u32,
            },
            mip_level_count: mip_level_count(macro_size.max(1)),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let macro_view = macro_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("raster-texture-bg"),
            layout: tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&macro_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: macro_params.as_entire_binding(),
                },
            ],
        });
        (texture, macro_texture, bind_group)
    }

    /// Upload `layers` (level 0) plus a CPU-generated mip chain
    /// into `texture`.
    fn upload_layers(
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        tile_size: u32,
        layers: &[Vec<u8>],
    ) {
        for (li, layer) in layers.iter().enumerate() {
            for (mi, (data, w, h)) in build_mip_chain(layer, tile_size).into_iter().enumerate() {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: mi as u32,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: li as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * w),
                        rows_per_image: Some(h),
                    },
                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    /// Phase 2f: install a scene texture set built by the shared
    /// spec builder (`textures::build_texture_spec`, the same one
    /// the RT path uses — gate 2f.3). Uploads the spec's compact
    /// layers with a CPU mip chain and rebuilds the group-1 bind
    /// group; block `tex_layers` indices address this array.
    pub fn set_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spec: &treebuild::render::TextureSpec,
    ) {
        let macro_count = spec.macro_layers.len();
        let (texture, macro_texture, bind_group) = Self::make_tex_bind_group(
            device,
            &self.tex_layout,
            spec.tile_size,
            spec.layers.len(),
            spec.macro_size.max(1),
            macro_count,
            &self.macro_params_buffer,
        );
        Self::upload_layers(queue, &texture, spec.tile_size, &spec.layers);
        if macro_count > 0 {
            Self::upload_layers(queue, &macro_texture, spec.macro_size, &spec.macro_layers);
        }
        let params = MacroParams {
            color_count: (macro_count / 2) as u32,
            debug_height: u32::from(self.macro_debug_height),
            _pad: [0; 2],
        };
        queue.write_buffer(&self.macro_params_buffer, 0, bytemuck::bytes_of(&params));
        self._tex_texture = texture;
        self._macro_texture = macro_texture;
        self.tex_bind_group = bind_group;
    }

    /// Stage F proof hook (`--macro-debug-height`): render macro
    /// materials' top faces as their height-layer grayscale
    /// instead of color. Takes effect on the next `set_textures`.
    pub fn set_macro_debug_height(&mut self, on: bool) {
        self.macro_debug_height = on;
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
                    // Defensive fallback for meshes built without
                    // UV/layer data: flat sentinel, never sampled.
                    uv: mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                    layer: mesh.layers.get(i).copied().unwrap_or(FLAT_LAYER),
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
            rpass.set_bind_group(1, &self.tex_bind_group, &[]);
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

/// Create a depth texture + view sized `size`. The texture
/// carries the usages the Phase 2g fx wiring needs (sampled by
/// the post pass, copied for the soft-particle field pass) on top
/// of render-attachment; the extra usages do not change what the
/// world pass renders into it.
pub fn create_depth_texture(
    device: &wgpu::Device,
    size: (u32, u32),
) -> (wgpu::Texture, wgpu::TextureView) {
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

/// Create a depth texture view sized `size` (windowed resize and
/// headless capture both use this).
pub fn create_depth_view(device: &wgpu::Device, size: (u32, u32)) -> wgpu::TextureView {
    create_depth_texture(device, size).1
}
