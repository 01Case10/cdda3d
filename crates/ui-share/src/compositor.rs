//! GPU compositing (Phase 4): a fullscreen-quad pipeline that draws
//! the UI texture over a background frame — the engine's 3D view in
//! the real app, a synthetic background in this harness — with
//! nearest-neighbor sampling and the scaling viewport from
//! [`crate::scaling`]. Follows the `crates/fx` offscreen harness
//! pattern (`compatible_surface: None`, offscreen target, staging
//! readback). Texel access is `textureLoad` by pixel coordinate, so
//! orientation is exact by construction.

use std::fmt;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};

use crate::frame::UploadPlan;
use crate::latency::LatencyStats;
use crate::scaling::{compute_viewport, ScalingMode, Size2};

pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[derive(Debug)]
pub enum UiError {
    NoAdapter,
    Device(String),
    Readback(String),
}
impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiError::NoAdapter => write!(f, "ui-capture: no wgpu adapter available"),
            UiError::Device(e) => write!(f, "ui-capture: device error: {e}"),
            UiError::Readback(e) => write!(f, "ui-capture: readback error: {e}"),
        }
    }
}
impl std::error::Error for UiError {}

/// CPU reference for the GPU blend (straight-alpha src-over an
/// opaque background), at UI resolution. The GPU path must match it
/// within sRGB quantization (±1 per channel in tests).
pub fn composite_cpu(background_rgb: [u8; 3], ui_tight: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for i in 0..(width * height) as usize {
        let s = &ui_tight[i * 4..i * 4 + 4];
        let a = s[3] as f32 / 255.0;
        for ch in 0..3 {
            let v = s[ch] as f32 * a + background_rgb[ch] as f32 * (1.0 - a);
            out.push(v.round().clamp(0.0, 255.0) as u8);
        }
        out.push(255);
    }
    out
}

/// CPU composite at output resolution honoring the scaling viewport
/// (nearest-neighbor). Used as the oracle for GPU captures.
pub fn composite_cpu_scaled(background: &[u8], bg_w: u32, bg_h: u32, ui_tight: &[u8], ui_w: u32, ui_h: u32, out_w: u32, out_h: u32, mode: ScalingMode) -> Vec<u8> {
    let vp = compute_viewport(Size2 { w: out_w, h: out_h }, Size2 { w: ui_w, h: ui_h }, mode);
    let mut out = background.to_vec();
    if out.len() < (out_w * out_h * 4) as usize { out.resize((out_w * out_h * 4) as usize, 0); }
    let _ = (bg_w, bg_h);
    for oy in 0..out_h {
        for ox in 0..out_w {
            let (fx, fy) = (ox as f32, oy as f32);
            if fx < vp.x || fy < vp.y || fx >= vp.x + vp.w || fy >= vp.y + vp.h { continue; }
            // Sample at the output pixel's center — the standard
            // nearest-neighbor convention, and what the GPU path
            // (fragment position = pixel center) computes.
            let ux = (((fx + 0.5 - vp.x) / vp.scale_x).floor() as u32).min(ui_w - 1);
            let uy = (((fy + 0.5 - vp.y) / vp.scale_y).floor() as u32).min(ui_h - 1);
            let s = ((uy * ui_w + ux) * 4) as usize;
            let a = ui_tight[s + 3] as f32 / 255.0;
            if a == 0.0 { continue; }
            let d = ((oy * out_w + ox) * 4) as usize;
            for ch in 0..3 {
                out[d + ch] = (ui_tight[s + ch] as f32 * a + out[d + ch] as f32 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
            }
            out[d + 3] = 255;
        }
    }
    out
}

const SHADER: &str = r#"
struct Uniforms { src_size: vec2<f32>, origin: vec2<f32>, scale: vec2<f32>, _pad: vec2<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var t_src: texture_2d<f32>;
struct VsOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi) - 1) * 2.0;
    let y = f32(i32(vi & 1u)) * 4.0 - 1.0;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}
@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let local = input.pos.xy - u.origin;
    let texel = vec2<i32>(floor(local / u.scale));
    let dims = vec2<i32>(u.src_size);
    return textureLoad(t_src, clamp(texel, vec2<i32>(0), dims - vec2<i32>(1)), 0);
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    src_size: [f32; 2],
    origin: [f32; 2],
    scale: [f32; 2],
    _pad: [f32; 2],
}

pub struct CompositeRequest<'a> {
    /// Background frame, tight RGBA at output size (the "3D view").
    pub background_tight: &'a [u8],
    pub ui_tight: &'a [u8],
    pub ui_size: (u32, u32),
    pub out_size: (u32, u32),
    pub scaling: ScalingMode,
    /// When set, the UI texture is first filled with `stale_ui_tight`
    /// and then updated with only this plan's rect from `ui_tight`
    /// (gate 4.5's GPU form). Requires `stale_ui_tight`.
    pub dirty_plan: Option<UploadPlan>,
    pub stale_ui_tight: Option<&'a [u8]>,
}

pub struct CompositeOutput {
    pub image: image::RgbaImage,
    pub adapter_name: String,
    pub stats: LatencyStats,
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_name: String,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

async fn make_gpu() -> Result<Gpu, UiError> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }).await.ok_or(UiError::NoAdapter)?;
    let adapter_name = adapter.get_info().name;
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ui-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: Default::default(),
    }, None).await.map_err(|e| UiError::Device(e.to_string()))?;
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ui-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ui-shader"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ui-pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ui-pl"), bind_group_layouts: &[&layout], push_constant_ranges: &[] })),
        vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
        fragment: Some(wgpu::FragmentState {
            module: &shader, entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState { format: FORMAT, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
        depth_stencil: None, multisample: Default::default(), multiview: None, cache: None,
    });
    Ok(Gpu { device, queue, adapter_name, pipeline, layout })
}

fn make_texture(gpu: &Gpu, w: u32, h: u32, label: &str) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn write_full(gpu: &Gpu, tex: &wgpu::Texture, tight: &[u8], w: u32, h: u32) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        tight,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
}

/// Upload only `plan.rect` of `src_tight` into `tex` (which already
/// holds the previous frame). Sub-rect writes carry just the rect's
/// rows, tightly packed per row.
fn write_dirty_rect(gpu: &Gpu, tex: &wgpu::Texture, src_tight: &[u8], ui_w: u32, plan: &UploadPlan) {
    let r = plan.rect;
    if r.is_empty() { return; }
    let row = (ui_w * 4) as usize;
    let mut data = Vec::with_capacity((r.w * r.h * 4) as usize);
    for y in r.y..r.y + r.h {
        let s = y as usize * row + r.x as usize * 4;
        data.extend_from_slice(&src_tight[s..s + (r.w as usize) * 4]);
    }
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: r.x as u32, y: r.y as u32, z: 0 }, aspect: wgpu::TextureAspect::All },
        &data,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some((r.w as u32) * 4), rows_per_image: Some(r.h as u32) },
        wgpu::Extent3d { width: r.w as u32, height: r.h as u32, depth_or_array_layers: 1 },
    );
}

fn bind_group(gpu: &Gpu, tex: &wgpu::Texture, uniforms: &Uniforms) -> wgpu::BindGroup {
    use wgpu::util::DeviceExt;
    let ubuf = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ui-uniforms"), contents: bytemuck::bytes_of(uniforms), usage: wgpu::BufferUsages::UNIFORM,
    });
    let view = tex.create_view(&Default::default());
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ui-bg"), layout: &gpu.layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
        ],
    })
}

pub fn composite_headless(req: &CompositeRequest) -> Result<CompositeOutput, UiError> {
    pollster::block_on(composite_headless_async(req))
}

async fn composite_headless_async(req: &CompositeRequest<'_>) -> Result<CompositeOutput, UiError> {
    let gpu = make_gpu().await?;
    let (ui_w, ui_h) = req.ui_size;
    let (out_w, out_h) = req.out_size;

    let t_upload = Instant::now();
    let bg_tex = make_texture(&gpu, out_w, out_h, "ui-bg-tex");
    write_full(&gpu, &bg_tex, req.background_tight, out_w, out_h);
    let ui_tex = make_texture(&gpu, ui_w, ui_h, "ui-tex");
    match (&req.dirty_plan, req.stale_ui_tight) {
        (Some(plan), Some(stale)) => {
            write_full(&gpu, &ui_tex, stale, ui_w, ui_h);
            write_dirty_rect(&gpu, &ui_tex, req.ui_tight, ui_w, plan);
        }
        _ => write_full(&gpu, &ui_tex, req.ui_tight, ui_w, ui_h),
    }
    gpu.queue.submit([]);
    let upload = t_upload.elapsed();

    let t_comp = Instant::now();
    let out_tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ui-out"),
        size: wgpu::Extent3d { width: out_w, height: out_h, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let out_view = out_tex.create_view(&Default::default());
    let vp = compute_viewport(Size2 { w: out_w, h: out_h }, Size2 { w: ui_w, h: ui_h }, req.scaling);
    let bg_bg = bind_group(&gpu, &bg_tex, &Uniforms { src_size: [out_w as f32, out_h as f32], origin: [0.0, 0.0], scale: [1.0, 1.0], _pad: [0.0; 2] });
    let ui_bg = bind_group(&gpu, &ui_tex, &Uniforms { src_size: [ui_w as f32, ui_h as f32], origin: [vp.x, vp.y], scale: [vp.scale_x, vp.scale_y], _pad: [0.0; 2] });
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ui-frame") });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_view, resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });
        // Background quad (fills the output; its alpha is 1 so the
        // blend result is the background itself).
        pass.set_pipeline(&gpu.pipeline);
        pass.set_bind_group(0, &bg_bg, &[]);
        pass.draw(0..3, 0..1);
        // UI quad inside the scaling viewport.
        if vp.w > 0.0 && vp.h > 0.0 {
            pass.set_viewport(vp.x, vp.y, vp.w, vp.h, 0.0, 1.0);
            pass.set_bind_group(0, &ui_bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    let unpadded = out_w * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ui-staging"), size: (padded * out_h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &out_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &staging, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(out_h) } },
        wgpu::Extent3d { width: out_w, height: out_h, depth_or_array_layers: 1 },
    );
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|_| UiError::Readback("map channel closed".into()))?.map_err(|e| UiError::Readback(e.to_string()))?;
    let mut pixels = Vec::with_capacity((unpadded * out_h) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..out_h as usize {
            let s = row * padded as usize;
            pixels.extend_from_slice(&data[s..s + unpadded as usize]);
        }
    }
    staging.unmap();
    let composite_and_readback = t_comp.elapsed();
    let image = image::RgbaImage::from_raw(out_w, out_h, pixels).ok_or_else(|| UiError::Readback("pixel buffer size mismatch".into()))?;
    Ok(CompositeOutput {
        image,
        adapter_name: gpu.adapter_name.clone(),
        stats: LatencyStats { store_read: std::time::Duration::ZERO, upload, composite_and_readback },
    })
}

/// Resident UI pass (Phase 2h engine wiring): the same fullscreen
/// quad + shader as the headless harness above, but as a reusable
/// pass over a caller-owned device/queue and an existing color
/// target — the engine keeps one of these beside its world
/// renderer and draws it as the LAST pass of the frame, after
/// world → fields → billboards → post (docs/phase4-ui.md wiring
/// step 4). The UI texture is uploaded once per new UI frame (full
/// upload, or only an [`UploadPlan`]'s dirty rect when the resident
/// texture already holds the previous frame at the same size —
/// gate 4.5's equivalence), not once per rendered frame.
pub struct UiPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    texture: Option<wgpu::Texture>,
    tex_size: (u32, u32),
}

impl UiPass {
    pub fn new(device: &wgpu::Device, out_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui-resident-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ui-resident-shader"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui-resident-pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("ui-resident-pl"), bind_group_layouts: &[&layout], push_constant_ranges: &[] })),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader, entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { format: out_format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: Default::default(), multiview: None, cache: None,
        });
        Self { pipeline, layout, texture: None, tex_size: (0, 0) }
    }

    /// Upload a UI frame's tight RGBA pixels. With `plan` set and a
    /// same-size resident texture, only the plan's rect is written
    /// (the resident texture must already hold the previous frame);
    /// otherwise the texture is (re)created and fully written.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, tight: &[u8], size: (u32, u32), plan: Option<&UploadPlan>) {
        let (w, h) = (size.0.max(1), size.1.max(1));
        if let (Some(plan), Some(tex)) = (plan, &self.texture) {
            if self.tex_size == (w, h) && !plan.rect.is_empty() {
                let r = plan.rect;
                let row = (w * 4) as usize;
                let mut data = Vec::with_capacity((r.w * r.h * 4) as usize);
                for y in r.y..r.y + r.h {
                    let s = y as usize * row + r.x as usize * 4;
                    data.extend_from_slice(&tight[s..s + (r.w as usize) * 4]);
                }
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: r.x as u32, y: r.y as u32, z: 0 }, aspect: wgpu::TextureAspect::All },
                    &data,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some((r.w * 4) as u32), rows_per_image: Some(r.h as u32) },
                    wgpu::Extent3d { width: r.w as u32, height: r.h as u32, depth_or_array_layers: 1 },
                );
                return;
            }
        }
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ui-resident-tex"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            tight,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.texture = Some(tex);
        self.tex_size = (w, h);
    }

    /// Draw the uploaded UI over `out_view` (loaded, never cleared)
    /// inside the scaling viewport for `out_size`. No-op before the
    /// first upload.
    pub fn draw(&self, device: &wgpu::Device, queue: &wgpu::Queue, out_view: &wgpu::TextureView, out_size: (u32, u32), mode: ScalingMode) {
        use wgpu::util::DeviceExt;
        let Some(tex) = &self.texture else { return };
        let (ui_w, ui_h) = self.tex_size;
        let vp = compute_viewport(Size2 { w: out_size.0, h: out_size.1 }, Size2 { w: ui_w, h: ui_h }, mode);
        if vp.w <= 0.0 || vp.h <= 0.0 { return; }
        let view = tex.create_view(&Default::default());
        let uniforms = Uniforms { src_size: [ui_w as f32, ui_h as f32], origin: [vp.x, vp.y], scale: [vp.scale_x, vp.scale_y], _pad: [0.0; 2] };
        let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui-resident-uniforms"), contents: bytemuck::bytes_of(&uniforms), usage: wgpu::BufferUsages::UNIFORM,
        });
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui-resident-bg"), layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ui-resident-encoder") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui-resident-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out_view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_viewport(vp.x, vp.y, vp.w, vp.h, 0.0, 1.0);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}

/// Probe for an adapter without rendering (tests use it to decide
/// whether GPU assertions are live or vacuous-skipped).
pub fn adapter_available() -> bool {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });
        instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::default(), compatible_surface: None, force_fallback_adapter: false }).await.is_some()
    })
}
