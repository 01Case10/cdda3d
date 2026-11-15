//! GPU rendering: an instanced wgpu billboard pass plus the
//! standalone headless harness (the `crates/fx` pattern: offscreen
//! `Rgba8UnormSrgb` target, `compatible_surface: None`, staging
//! readback, named `NoAdapter` error — lavapipe counts as a real
//! adapter, just a slow one).
//!
//! Composition order (the contract the engine wiring follows):
//! world/RT pass writes color + depth -> THIS pass draws shadows
//! then billboards, depth-TESTED against that depth (never
//! depth-writing in blend mode) -> field effects (Phase 2b) ->
//! post -> UI quad (Phase 4). [`BillboardPass`] is the reusable
//! piece: the engine creates it once, calls
//! [`BillboardPass::prepare`] per frame with the packed composite
//! texture + instances, and [`BillboardPass::draw`] inside a render
//! pass whose depth attachment is the world/RT depth. The headless
//! harness below uses exactly that path over its own box scene, so
//! a capture is evidence about the real pass, not a parallel one.
//!
//! Alpha (gate 6.3): the atlas texture is uploaded PREMULTIPLIED
//! (rgb *= a on the CPU at upload) and sampled with a linear
//! sampler; the fragment outputs the premultiplied color and the
//! blend state is (One, OneMinusSrcAlpha). Filtering a
//! premultiplied texture cannot invent dark fringes, because the
//! color of a transparent texel is already black in proportion to
//! its alpha — that is the standard no-dark-halo construction. A
//! cutout pipeline (discard below alpha 0.5, depth-write ON) is
//! provided for fully-opaque-bodied sprites where depth-writing
//! gives free correct occlusion; the harness crowd uses the blend
//! pipeline with back-to-front sorting.

use std::fmt;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use image::RgbaImage;

use crate::composite::Composite;
use crate::instance::{BillboardInstance, ShadowInstance};

pub const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[derive(Debug)]
pub enum BillboardError {
    NoAdapter,
    Device(String),
    Readback(String),
    Io(std::io::Error),
}
impl fmt::Display for BillboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BillboardError::NoAdapter => write!(f, "billboard-capture: no wgpu adapter available"),
            BillboardError::Device(e) => write!(f, "billboard-capture: device error: {e}"),
            BillboardError::Readback(e) => write!(f, "billboard-capture: readback error: {e}"),
            BillboardError::Io(e) => write!(f, "billboard-capture: io error: {e}"),
        }
    }
}
impl std::error::Error for BillboardError {}
impl From<std::io::Error> for BillboardError {
    fn from(e: std::io::Error) -> Self {
        BillboardError::Io(e)
    }
}

/// Orbit camera (same math as `crates/fx::render::Camera` /
/// the engine's camera; re-implemented because this crate may not
/// depend on engine).
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub target: Vec3,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub distance: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Camera { target: Vec3::new(5.5, 0.7, -4.2), yaw_deg: 180.0, pitch_deg: 34.0, distance: 10.5 }
    }
}
impl Camera {
    pub fn eye(&self) -> Vec3 {
        let yaw = self.yaw_deg.to_radians();
        let pitch = self.pitch_deg.clamp(5.0, 85.0).to_radians();
        let offset = Vec3::new(pitch.cos() * yaw.sin(), pitch.sin(), pitch.cos() * yaw.cos()) * self.distance;
        self.target + offset
    }
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(55f32.to_radians(), aspect.max(0.01), 0.1, 600.0)
            * Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }
}

// ---------------------------------------------------------------------
// Composite packing: many composites -> one GPU texture
// ---------------------------------------------------------------------

/// Shelf-pack composites into one texture image; returns the image
/// and each composite's UV rect in the same order. This packed
/// texture is what the engine uploads per frame (repacked only when
/// the cache gains a new composite).
pub fn pack_composites(composites: &[&Composite]) -> (RgbaImage, Vec<[f32; 4]>) {
    let atlas_w: u32 = 1024;
    let mut x = 0u32;
    let mut y = 0u32;
    let mut shelf_h = 0u32;
    let mut placements = Vec::new();
    for c in composites {
        if x + c.width() > atlas_w {
            x = 0;
            y += shelf_h;
            shelf_h = 0;
        }
        placements.push((x, y));
        x += c.width();
        shelf_h = shelf_h.max(c.height());
    }
    let atlas_h = (y + shelf_h).max(1);
    let mut img = RgbaImage::new(atlas_w, atlas_h);
    let mut rects = Vec::new();
    for (c, (px, py)) in composites.iter().zip(&placements) {
        image::imageops::replace(&mut img, &c.image, *px as i64, *py as i64);
        rects.push([
            *px as f32 / atlas_w as f32,
            *py as f32 / atlas_h as f32,
            (px + c.width()) as f32 / atlas_w as f32,
            (py + c.height()) as f32 / atlas_h as f32,
        ]);
    }
    (img, rects)
}

/// Premultiply an image's RGB by its alpha (upload-time, gate 6.3).
pub fn premultiply(img: &RgbaImage) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        let a = p.0[3] as u32;
        p.0[0] = (p.0[0] as u32 * a / 255) as u8;
        p.0[1] = (p.0[1] as u32 * a / 255) as u8;
        p.0[2] = (p.0[2] as u32 * a / 255) as u8;
    }
    out
}

// ---------------------------------------------------------------------
// GPU types + shaders
// ---------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 3],
    time: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct InstGpu {
    anchor: [f32; 3],
    center_offset: f32,
    size: [f32; 2],
    bottom: f32,
    mirrored: f32,
    uv_rect: [f32; 4],
    tint: [f32; 4],
}

impl From<&BillboardInstance> for InstGpu {
    fn from(i: &BillboardInstance) -> Self {
        InstGpu {
            anchor: i.anchor.to_array(),
            center_offset: i.center_offset,
            size: i.size.to_array(),
            bottom: i.bottom,
            mirrored: if i.mirrored { 1.0 } else { 0.0 },
            uv_rect: i.uv_rect,
            tint: i.tint,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ShadowGpu {
    center: [f32; 3],
    radius: f32,
    strength: f32,
    _pad: [f32; 3],
}

pub const BILLBOARD_SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var t_atlas: texture_2d<f32>;
@group(0) @binding(2) var s_atlas: sampler;
struct Inst {
    anchor: vec3<f32>, center_offset: f32,
    size: vec2<f32>, bottom: f32, mirrored: f32,
    uv_rect: vec4<f32>, tint: vec4<f32>,
};
@group(0) @binding(3) var<storage, read> insts: array<Inst>;
// Quad corners by vertex_index: 0 BL, 1 BR, 2 TR, 3 TL (two tris).
const CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.,0.), vec2<f32>(1.,0.), vec2<f32>(1.,1.),
    vec2<f32>(0.,0.), vec2<f32>(1.,1.), vec2<f32>(0.,1.));
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) tint: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let inst = insts[ii];
    let c = CORNER[vi % 6u];
    var fwd = u.eye - inst.anchor; fwd.y = 0.0;
    fwd = select(vec3<f32>(0.,0.,1.), normalize(fwd), length(fwd) > 1e-5);
    let right = normalize(vec3<f32>(fwd.z, 0.0, -fwd.x));
    var lx = (c.x - 0.5) * inst.size.x;
    let world = inst.anchor + right * (inst.center_offset + lx)
        + vec3<f32>(0.0, inst.bottom + c.y * inst.size.y, 0.0);
    var uu = select(c.x, 1.0 - c.x, inst.mirrored > 0.5);
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(world, 1.0);
    out.uv = vec2<f32>(mix(inst.uv_rect.x, inst.uv_rect.z, uu),
                       mix(inst.uv_rect.w, inst.uv_rect.y, c.y));
    out.tint = inst.tint;
    return out;
}
@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    // Texture is premultiplied at upload; output stays premultiplied.
    var col = textureSample(t_atlas, s_atlas, input.uv);
    // Hit flash: mix toward hot white by tint.a (premultiplied-safe:
    // flash adds light proportional to coverage).
    col = vec4<f32>(mix(col.rgb, vec3<f32>(1.0, 0.85, 0.8) * col.a, input.tint.a), col.a);
    col = vec4<f32>(col.rgb * input.tint.rgb, col.a);
    return col;
}
"#;

pub const BILLBOARD_CUTOUT_SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var t_atlas: texture_2d<f32>;
@group(0) @binding(2) var s_atlas: sampler;
struct Inst {
    anchor: vec3<f32>, center_offset: f32,
    size: vec2<f32>, bottom: f32, mirrored: f32,
    uv_rect: vec4<f32>, tint: vec4<f32>,
};
@group(0) @binding(3) var<storage, read> insts: array<Inst>;
const CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.,0.), vec2<f32>(1.,0.), vec2<f32>(1.,1.),
    vec2<f32>(0.,0.), vec2<f32>(1.,1.), vec2<f32>(0.,1.));
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) tint: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let inst = insts[ii];
    let c = CORNER[vi % 6u];
    var fwd = u.eye - inst.anchor; fwd.y = 0.0;
    fwd = select(vec3<f32>(0.,0.,1.), normalize(fwd), length(fwd) > 1e-5);
    let right = normalize(vec3<f32>(fwd.z, 0.0, -fwd.x));
    var lx = (c.x - 0.5) * inst.size.x;
    let world = inst.anchor + right * (inst.center_offset + lx)
        + vec3<f32>(0.0, inst.bottom + c.y * inst.size.y, 0.0);
    var uu = select(c.x, 1.0 - c.x, inst.mirrored > 0.5);
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(world, 1.0);
    out.uv = vec2<f32>(mix(inst.uv_rect.x, inst.uv_rect.z, uu),
                       mix(inst.uv_rect.w, inst.uv_rect.y, c.y));
    out.tint = inst.tint;
    return out;
}
@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let col = textureSample(t_atlas, s_atlas, input.uv);
    if (col.a < 0.5) { discard; }
    return vec4<f32>(col.rgb / max(col.a, 1e-4), 1.0);
}
"#;

const SHADOW_SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;
struct Sh { center: vec3<f32>, radius: f32, strength: f32, _p0: f32, _p1: f32, _p2: f32 };
@group(0) @binding(1) var<storage, read> shadows: array<Sh>;
const CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.,-1.), vec2<f32>(1.,-1.), vec2<f32>(1.,1.),
    vec2<f32>(-1.,-1.), vec2<f32>(1.,1.), vec2<f32>(-1.,1.));
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) local: vec2<f32>, @location(1) strength: f32 };
@vertex fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let s = shadows[ii];
    let c = CORNER[vi % 6u];
    let world = s.center + vec3<f32>(c.x * s.radius, 0.0, c.y * s.radius);
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(world, 1.0);
    out.local = c;
    out.strength = s.strength;
    return out;
}
@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let d = length(input.local);
    let a = clamp(1.0 - d, 0.0, 1.0);
    let soft = a * a * (3.0 - 2.0 * a);
    return vec4<f32>(vec3<f32>(0.0), soft * input.strength);
}
"#;

// ---------------------------------------------------------------------
// Reusable pass (engine wiring surface)
// ---------------------------------------------------------------------

/// Which alpha pipeline the billboard draw uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    /// Premultiplied blend, no depth write, back-to-front sorted.
    Blend,
    /// Alpha-test 0.5, depth write on.
    Cutout,
}

/// The reusable billboard pass. Owns pipelines + per-frame buffers;
/// the engine keeps one of these and one for shadows alongside its
/// world renderer.
pub struct BillboardPass {
    blend_pipeline: wgpu::RenderPipeline,
    cutout_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    shadow_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    shadow_bind_group: Option<wgpu::BindGroup>,
    count: u32,
    shadow_count: u32,
}

impl BillboardPass {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat, depth_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bb-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let mk = |shader_src: &str, blend: Option<wgpu::BlendState>, depth_write: bool, label: &str| {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(label), source: wgpu::ShaderSource::Wgsl(shader_src.into()) });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: &[&layout], push_constant_ranges: &[] })),
                vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
                fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: color_format, blend, write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
                primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState { format: depth_format, depth_write_enabled: depth_write, depth_compare: wgpu::CompareFunction::LessEqual, stencil: Default::default(), bias: Default::default() }),
                multisample: Default::default(), multiview: None, cache: None,
            })
        };
        let blend_pipeline = mk(BILLBOARD_SHADER, Some(wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add }, alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add } }), false, "bb-blend");
        let cutout_pipeline = mk(BILLBOARD_CUTOUT_SHADER, None, true, "bb-cutout");

        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bb-shadow-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("bb-shadow-shader"), source: wgpu::ShaderSource::Wgsl(SHADOW_SHADER.into()) });
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bb-shadow-pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("bb-shadow-pl"), bind_group_layouts: &[&shadow_layout], push_constant_ranges: &[] })),
            vertex: wgpu::VertexState { module: &shadow_shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shadow_shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: color_format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: depth_format, depth_write_enabled: false, depth_compare: wgpu::CompareFunction::LessEqual, stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(), multiview: None, cache: None,
        });
        BillboardPass { blend_pipeline, cutout_pipeline, shadow_pipeline, layout, shadow_layout, bind_group: None, shadow_bind_group: None, count: 0, shadow_count: 0 }
    }

    /// Upload this frame's uniforms, atlas texture (premultiplied
    /// here), instances and shadows. Call once per frame before
    /// `draw`. Buffer creation per call is the simple harness-grade
    /// approach; instance counts here are small (thousands).
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera,
        aspect: f32,
        time_s: f32,
        atlas_image: &RgbaImage,
        instances: &[BillboardInstance],
        shadows: &[ShadowInstance],
    ) {
        use wgpu::util::DeviceExt;
        let uniforms = Uniforms { view_proj: camera.view_projection(aspect).to_cols_array_2d(), eye: camera.eye().to_array(), time: time_s };
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("bb-uniforms"), contents: bytemuck::bytes_of(&uniforms), usage: wgpu::BufferUsages::UNIFORM });
        let premul = premultiply(atlas_image);
        let tex = device.create_texture_with_data(queue, &wgpu::TextureDescriptor {
            label: Some("bb-atlas"),
            size: wgpu::Extent3d { width: premul.width(), height: premul.height(), depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }, wgpu::util::TextureDataOrder::LayerMajor, premul.as_raw());
        let view = tex.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("bb-sampler"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        let gpu: Vec<InstGpu> = instances.iter().map(InstGpu::from).collect();
        let gpu = if gpu.is_empty() { vec![InstGpu { anchor: [0.0; 3], center_offset: 0.0, size: [0.0; 2], bottom: 0.0, mirrored: 0.0, uv_rect: [0.0; 4], tint: [0.0; 4] }] } else { gpu };
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("bb-instances"), contents: bytemuck::cast_slice(&gpu), usage: wgpu::BufferUsages::STORAGE });
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bb-bg"), layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: ibuf.as_entire_binding() },
            ],
        }));
        // Keep resources alive until submit: leak-per-frame is NOT
        // acceptable in the engine, but wgpu bind groups keep their
        // buffers/textures alive by reference internally, so storing
        // only the bind group is sufficient.
        self.count = instances.len() as u32;

        let sgpu: Vec<ShadowGpu> = shadows.iter().map(|s| ShadowGpu { center: s.center.to_array(), radius: s.radius, strength: s.strength, _pad: [0.0; 3] }).collect();
        let sgpu = if sgpu.is_empty() { vec![ShadowGpu { center: [0.0; 3], radius: 0.0, strength: 0.0, _pad: [0.0; 3] }] } else { sgpu };
        let sbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("bb-shadows"), contents: bytemuck::cast_slice(&sgpu), usage: wgpu::BufferUsages::STORAGE });
        self.shadow_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bb-shadow-bg"), layout: &self.shadow_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: sbuf.as_entire_binding() },
            ],
        }));
        self.shadow_count = shadows.len() as u32;
        // `ubuf` and `tex` are retained by the bind groups above
        // (wgpu bind groups hold their resources by handle), so
        // dropping the local handles here is safe.
    }

    /// Draw shadows then billboards into an open render pass whose
    /// depth attachment already holds the world/RT depth. This is
    /// THE engine integration point for external (traced) depth:
    /// the pass never creates or clears depth, it only tests it.
    pub fn draw(&self, rpass: &mut wgpu::RenderPass<'_>, mode: AlphaMode) {
        if self.shadow_count > 0 {
            if let Some(bg) = &self.shadow_bind_group {
                rpass.set_pipeline(&self.shadow_pipeline);
                rpass.set_bind_group(0, bg, &[]);
                rpass.draw(0..6, 0..self.shadow_count);
            }
        }
        if self.count > 0 {
            if let Some(bg) = &self.bind_group {
                rpass.set_pipeline(match mode { AlphaMode::Blend => &self.blend_pipeline, AlphaMode::Cutout => &self.cutout_pipeline });
                rpass.set_bind_group(0, bg, &[]);
                rpass.draw(0..6, 0..self.count);
            }
        }
    }
}

// ---------------------------------------------------------------------
// Headless harness scene: voxel wall with a window opening
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct BoxPrim {
    pub min: Vec3,
    pub max: Vec3,
    pub color: [f32; 3],
}

fn bx(min: [f32; 3], max: [f32; 3], color: [f32; 3]) -> BoxPrim {
    BoxPrim { min: Vec3::from_array(min), max: Vec3::from_array(max), color }
}

/// Wall/window scene for gates 6.2 / 6.11: a floor, a back wall with
/// a real window opening (four boxes around a hole, plus a pale
/// "outside" backdrop visible through it), side walls. Entities
/// stand in front of the wall, behind it (seen only through the
/// window), and beside it.
pub fn wall_scene() -> Vec<BoxPrim> {
    vec![
        bx([0.0, -0.2, -10.0], [12.0, 0.0, 0.0], [0.45, 0.35, 0.22]), // ground
        bx([0.4, 0.0, -9.6], [11.6, 0.06, -0.4], [0.60, 0.45, 0.28]), // floor slab
        // Back wall (game y=0 row, world z in [-1,0]) with window hole x in [5,7], y in [1,2.2]:
        bx([0.0, 0.0, -1.0], [5.0, 3.0, 0.0], [0.85, 0.80, 0.70]),
        bx([7.0, 0.0, -1.0], [12.0, 3.0, 0.0], [0.85, 0.80, 0.70]),
        bx([5.0, 0.0, -1.0], [7.0, 1.0, 0.0], [0.85, 0.80, 0.70]),
        bx([5.0, 2.2, -1.0], [7.0, 3.0, 0.0], [0.85, 0.80, 0.70]),
        // Window frame tint inside the hole edges.
        bx([5.0, 1.0, -0.95], [7.0, 1.08, -0.05], [0.35, 0.25, 0.18]),
        // Pale outside backdrop behind the wall (seen through window).
        bx([3.0, 0.0, 1.5], [9.0, 4.0, 2.0], [0.65, 0.75, 0.85]),
        // Side walls.
        bx([0.0, 0.0, -10.0], [0.6, 1.8, -1.0], [0.55, 0.38, 0.30]),
        bx([11.4, 0.0, -10.0], [12.0, 1.8, -1.0], [0.55, 0.38, 0.30]),
    ]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct BoxVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

const BOX_SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;
struct VsIn { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec4<f32> };
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) normal: vec3<f32>, @location(1) color: vec4<f32> };
@vertex fn vs_main(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(input.position, 1.0);
    out.normal = input.normal;
    out.color = input.color;
    return out;
}
@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(input.normal);
    let l = normalize(vec3<f32>(0.35, 0.8, 0.45));
    let shade = 0.45 + 0.55 * max(dot(n, l), 0.0);
    return vec4<f32>(input.color.rgb * shade, input.color.a);
}
"#;

fn boxes_to_vertices(boxes: &[BoxPrim]) -> (Vec<BoxVertex>, Vec<u32>) {
    let mut verts = Vec::new();
    let mut idx = Vec::new();
    for b in boxes {
        let (x0, y0, z0) = (b.min.x, b.min.y, b.min.z);
        let (x1, y1, z1) = (b.max.x, b.max.y, b.max.z);
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([0.0, 1.0, 0.0], [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]]),
            ([0.0, -1.0, 0.0], [[x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]]),
            ([0.0, 0.0, 1.0], [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]]),
            ([0.0, 0.0, -1.0], [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]]),
            ([1.0, 0.0, 0.0], [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]]),
            ([-1.0, 0.0, 0.0], [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]]),
        ];
        for (normal, corners) in faces {
            let base = verts.len() as u32;
            for c in corners {
                verts.push(BoxVertex { position: c, normal, color: [b.color[0], b.color[1], b.color[2], 1.0] });
            }
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    (verts, idx)
}

// ---------------------------------------------------------------------
// Capture entry point
// ---------------------------------------------------------------------

pub struct CaptureRequest<'a> {
    pub boxes: &'a [BoxPrim],
    pub atlas_image: &'a RgbaImage,
    pub instances: &'a [BillboardInstance],
    pub shadows: &'a [ShadowInstance],
    pub camera: Camera,
    pub size: (u32, u32),
    pub mode: AlphaMode,
    pub time_s: f32,
}

pub struct CaptureOutput {
    pub image: RgbaImage,
    /// Wall-clock render+readback time (llvmpipe numbers on this VM
    /// — see the phase doc; NOT target-hardware numbers).
    pub frame_ms: f64,
}

pub fn render_frame(req: &CaptureRequest) -> Result<CaptureOutput, BillboardError> {
    pollster::block_on(render_frame_async(req))
}

async fn render_frame_async(req: &CaptureRequest<'_>) -> Result<CaptureOutput, BillboardError> {
    use wgpu::util::DeviceExt;
    let t0 = Instant::now();
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::default(), compatible_surface: None, force_fallback_adapter: false }).await.ok_or(BillboardError::NoAdapter)?;
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { label: Some("bb-device"), required_features: wgpu::Features::empty(), required_limits: wgpu::Limits::default(), memory_hints: Default::default() }, None).await.map_err(|e| BillboardError::Device(e.to_string()))?;
    let (width, height) = (req.size.0.max(1), req.size.1.max(1));
    let aspect = width as f32 / height as f32;

    // Box (world) pipeline — shares the billboard pass's uniform layout values.
    let uniforms = Uniforms { view_proj: req.camera.view_projection(aspect).to_cols_array_2d(), eye: req.camera.eye().to_array(), time: req.time_s };
    let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("bb-box-uniforms"), contents: bytemuck::bytes_of(&uniforms), usage: wgpu::BufferUsages::UNIFORM });
    let box_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("bb-box-layout"), entries: &[wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None }] });
    let box_bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("bb-box-bg"), layout: &box_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() }] });
    let box_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("bb-box-shader"), source: wgpu::ShaderSource::Wgsl(BOX_SHADER.into()) });
    let box_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("bb-box-pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("bb-box-pl"), bind_group_layouts: &[&box_layout], push_constant_ranges: &[] })),
        vertex: wgpu::VertexState { module: &box_shader, entry_point: Some("vs_main"), buffers: &[wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<BoxVertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 }, wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 }, wgpu::VertexAttribute { offset: 24, shader_location: 2, format: wgpu::VertexFormat::Float32x4 }] }], compilation_options: Default::default() },
        fragment: Some(wgpu::FragmentState { module: &box_shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: CAPTURE_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
        primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: Some(wgpu::Face::Back), ..Default::default() },
        depth_stencil: Some(wgpu::DepthStencilState { format: DEPTH_FORMAT, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default() }),
        multisample: Default::default(), multiview: None, cache: None,
    });
    let (mut verts, indices) = boxes_to_vertices(req.boxes);
    if verts.is_empty() {
        // wgpu buffers may not be zero-sized; the draw is skipped
        // when there are no indices, this only satisfies creation.
        verts.push(BoxVertex { position: [0.0; 3], normal: [0.0, 1.0, 0.0], color: [0.0; 4] });
    }
    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("bb-box-verts"), contents: bytemuck::cast_slice(&verts), usage: wgpu::BufferUsages::VERTEX });
    let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("bb-box-idx"), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX });

    let mut pass = BillboardPass::new(&device, CAPTURE_FORMAT, DEPTH_FORMAT);
    pass.prepare(&device, &queue, &req.camera, aspect, req.time_s, req.atlas_image, req.instances, req.shadows);

    let color_tex = device.create_texture(&wgpu::TextureDescriptor { label: Some("bb-color"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: CAPTURE_FORMAT, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[] });
    let color_view = color_tex.create_view(&Default::default());
    let depth_tex = device.create_texture(&wgpu::TextureDescriptor { label: Some("bb-depth"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: DEPTH_FORMAT, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[] });
    let depth_view = depth_tex.create_view(&Default::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("bb-frame") });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("bb-world+billboard-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &color_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.10, g: 0.12, b: 0.17, a: 1.0 }), store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
            timestamp_writes: None, occlusion_query_set: None,
        });
        if !indices.is_empty() {
            rpass.set_pipeline(&box_pipeline);
            rpass.set_bind_group(0, &box_bg, &[]);
            rpass.set_vertex_buffer(0, vbuf.slice(..));
            rpass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
        // Billboards composite IN THE SAME PASS over the scene
        // depth — the harness stand-in for the RT traced depth.
        pass.draw(&mut rpass, req.mode);
    }
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("bb-staging"), size: (padded * height) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    encoder.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo { texture: &color_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, wgpu::TexelCopyBufferInfo { buffer: &staging, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) } }, wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|_| BillboardError::Readback("map channel closed".into()))?.map_err(|e| BillboardError::Readback(e.to_string()))?;
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..height as usize {
            let start = row * padded as usize;
            pixels.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    let frame_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let image = RgbaImage::from_raw(width, height, pixels).ok_or_else(|| BillboardError::Readback("pixel buffer size mismatch".into()))?;
    Ok(CaptureOutput { image, frame_ms })
}
