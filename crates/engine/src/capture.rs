//! Headless capture mode (Phase 2 deliverable, plan §5/§7, test 2.10).
//!
//! Offscreen wgpu rendering with no window, no surface, and no event
//! loop: `Instance`/`Adapter`/`Device` with `compatible_surface: None`,
//! render the same fake-house chunk meshes through the same
//! `render::Renderer` as the windowed path into an offscreen
//! `Rgba8UnormSrgb` texture, read it back, and write a PNG.
//!
//! On a machine with no wgpu adapter this returns
//! `CaptureError::NoAdapter` — a named, clean failure. It never panics
//! and never writes a blank PNG presented as a pass.

use std::fmt;
use std::path::PathBuf;

use glam::Vec3;

use crate::camera::OrbitCamera;
use crate::cutaway::CutawayMode;
use crate::render::{self, Renderer, CAPTURE_FORMAT};

/// Which renderer a capture uses (Phase 2c added the ray-traced
/// path; raster remains the default — see docs/phase2c-rt.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Raster,
    Rt,
}

impl RendererKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "raster" => Some(RendererKind::Raster),
            "rt" => Some(RendererKind::Rt),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            RendererKind::Raster => "raster",
            RendererKind::Rt => "rt",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureArgs {
    /// Scene source. Phase 2 supports only "stub-house".
    pub snapshot: String,
    /// Renderer selection (Phase 2c).
    pub renderer: RendererKind,
    /// Voxelizer output directory for RT textures (Phase 5
    /// hookup). `None` = flat-color RT rendering.
    pub textures: Option<std::path::PathBuf>,
    /// Voxelizer `effects.json` for the Phase 2g.1 fx wiring:
    /// field volumes + the HD-2D post stack over the fake-house
    /// field scene. `None` (and no `effects_defs`) = the pre-2g
    /// render, byte-for-byte.
    pub effects: Option<std::path::PathBuf>,
    /// Effect defs supplied directly (tests); takes precedence
    /// over `effects` when both are set.
    pub effects_defs: Option<Vec<fx::effect::EffectDef>>,
    /// Draw the field volumes (only meaningful with effects).
    pub fx_fields: bool,
    /// Run the post stack (only meaningful with effects).
    pub fx_post: bool,
    /// Animation time for the field shader, pinned so captures
    /// are deterministic (the fx harness pins 1.25 s too).
    pub fx_time: f32,
    /// Follow-target tile coords (x = east, y = south).
    pub camera_target: (f32, f32),
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub distance: f32,
    pub size: (u32, u32),
    pub out: PathBuf,
    /// Player z for the cutaway (levels <= this z are drawn).
    pub cutaway_z: i32,
}

impl Default for CaptureArgs {
    fn default() -> Self {
        Self {
            snapshot: "stub-house".to_string(),
            renderer: RendererKind::Raster,
            textures: None,
            effects: None,
            effects_defs: None,
            fx_fields: true,
            fx_post: true,
            fx_time: 1.25,
            camera_target: (6.0, 6.0),
            yaw_deg: 45.0,
            pitch_deg: 45.0,
            distance: 28.0,
            size: (1280, 720),
            out: PathBuf::from("captures/house.png"),
            cutaway_z: 1,
        }
    }
}

#[derive(Debug)]
pub enum CaptureError {
    NoAdapter,
    UnsupportedSnapshot(String),
    Device(String),
    Readback(String),
    Io(std::io::Error),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::NoAdapter => {
                write!(f, "headless-capture: no wgpu adapter available")
            }
            CaptureError::UnsupportedSnapshot(s) => write!(
                f,
                "headless-capture: unsupported --snapshot source '{s}' (Phase 2 supports 'stub-house')"
            ),
            CaptureError::Device(e) => write!(f, "headless-capture: device error: {e}"),
            CaptureError::Readback(e) => {
                write!(f, "headless-capture: readback error: {e}")
            }
            CaptureError::Io(e) => write!(f, "headless-capture: io error: {e}"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<std::io::Error> for CaptureError {
    fn from(e: std::io::Error) -> Self {
        CaptureError::Io(e)
    }
}

/// Outcome of a successful capture.
#[derive(Debug, Clone)]
pub struct CaptureOutcome {
    pub out: PathBuf,
    pub size: (u32, u32),
    /// Raster: mesh quads drawn. RT: tree leaves (voxels) traced.
    pub quads: usize,
    pub distinct_colors: usize,
}

/// The effect defs a capture runs with, if fx is enabled:
/// direct defs win over the `effects.json` path.
fn fx_defs(args: &CaptureArgs) -> Result<Option<Vec<fx::effect::EffectDef>>, CaptureError> {
    if let Some(defs) = &args.effects_defs {
        return Ok(Some(defs.clone()));
    }
    match &args.effects {
        Some(path) => Ok(Some(
            crate::fxwire::load_defs(path).map_err(CaptureError::Device)?,
        )),
        None => Ok(None),
    }
}

/// The capture's orbit camera (shared framing for both
/// renderers — the RT camera is built from the same values).
fn capture_camera(args: &CaptureArgs) -> OrbitCamera {
    let mut camera = OrbitCamera::new(Vec3::new(args.camera_target.0, 1.0, -args.camera_target.1));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    camera
}

/// Composite the fx layers over a rendered world (Phase 2g.1):
/// field volumes (soft-particle pass) into `color_view`, then
/// the post stack into a fresh texture. Returns the post output
/// texture when the post stack ran — the caller reads back from
/// it — or `None` to read back from the color texture itself.
/// Frame order is fixed here: fields, then post; UI (Phase 4)
/// composites after everything, in its own step.
#[allow(clippy::too_many_arguments)]
fn apply_fx(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    defs: &[fx::effect::EffectDef],
    camera: &OrbitCamera,
    color_view: &wgpu::TextureView,
    depth_texture: &wgpu::Texture,
    depth_view: &wgpu::TextureView,
    size: (u32, u32),
) -> Result<Option<wgpu::Texture>, CaptureError> {
    let (width, height) = size;
    let aspect = width as f32 / height as f32;
    if args.fx_fields {
        let fields = crate::fxwire::fake_house_fields(defs).map_err(CaptureError::Device)?;
        let instances = crate::fxwire::sorted_instances(&fields, camera.eye());
        if !instances.is_empty() {
            // The soft-particle pass samples a COPY of the scene
            // depth (a pass cannot sample its own depth
            // attachment).
            let depth_copy = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("capture-depth-copy"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: render::DEPTH_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let depth_copy_view = depth_copy.create_view(&Default::default());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("capture-depth-copy-encoder"),
            });
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: depth_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &depth_copy,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let pass = fx::passes::FieldPass::new(device, CAPTURE_FORMAT, render::DEPTH_FORMAT);
            let uniforms = fx::passes::FieldUniforms::new(
                camera.view_projection(aspect),
                camera.eye(),
                args.fx_time,
                fx::passes::soft_fade_z(camera.distance, 0.35),
            );
            let prepared = pass.prepare(device, &instances, &uniforms, &depth_copy_view);
            pass.draw(device, queue, &prepared, color_view, depth_view);
        }
    }
    if args.fx_post {
        let out_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("capture-post-out"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: CAPTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let out_view = out_tex.create_view(&Default::default());
        let post = fx::passes::PostPass::new(device, CAPTURE_FORMAT);
        post.draw(
            device,
            queue,
            color_view,
            depth_view,
            &out_view,
            &fx::post::PostConfig::default(),
            fx::passes::projected_depth(camera.distance),
            size,
        );
        return Ok(Some(out_tex));
    }
    Ok(None)
}

/// Read an offscreen texture back into an image (+ its distinct
/// color count).
fn readback_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Result<(image::RgbaImage, usize), CaptureError> {
    let (width, height) = size;
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("capture-staging"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("capture-copy"),
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

    let distinct: std::collections::HashSet<[u8; 4]> = pixels
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect();
    let img = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| CaptureError::Readback("pixel buffer size mismatch".into()))?;
    Ok((img, distinct.len()))
}

/// Save a capture image to `args.out` (creating parent dirs).
fn save_capture(args: &CaptureArgs, img: &image::RgbaImage) -> Result<(), CaptureError> {
    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    img.save(&args.out)
        .map_err(|e| CaptureError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}

/// Run one headless capture. Synchronous (pollster drives the futures).
pub fn run_capture(args: &CaptureArgs) -> Result<CaptureOutcome, CaptureError> {
    if args.snapshot != "stub-house" {
        return Err(CaptureError::UnsupportedSnapshot(args.snapshot.clone()));
    }
    pollster::block_on(run_capture_async(args))
}

async fn run_capture_async(args: &CaptureArgs) -> Result<CaptureOutcome, CaptureError> {
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
                label: Some("headless-capture-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .map_err(|e| CaptureError::Device(e.to_string()))?;

    let defs = fx_defs(args)?;

    // Phase 2c ray-traced path: same world and cutaway selection,
    // rendered through the Sparse-Voxel-64Tree tracer instead of the
    // mesher/rasterizer.
    if args.renderer == RendererKind::Rt {
        if let Some(defs) = &defs {
            // Phase 2g.1 RT fx: trace the frame, copy its color
            // into an engine-format target, convert its linear
            // depth buffer into a projected-depth texture, then
            // run the same fx compositing as the raster path.
            let frame = crate::rt::render_rt_frame(&device, &queue, args)?;
            let (width, height) = frame.size;
            let color_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("capture-rt-color"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: CAPTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let color_view = color_texture.create_view(&Default::default());
            let (depth_texture, depth_view) =
                render::create_depth_texture(&device, (width, height));
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("capture-rt-copy"),
            });
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &frame.color,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &color_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);
            crate::fxwire::RtDepthConvert::new(&device).draw(
                &device,
                &queue,
                &frame.depth,
                frame.max_dist,
                &frame.camera,
                (width, height),
                &depth_view,
            );
            let camera = capture_camera(args);
            let post_out = apply_fx(
                &device,
                &queue,
                args,
                defs,
                &camera,
                &color_view,
                &depth_texture,
                &depth_view,
                (width, height),
            )?;
            let source = post_out.as_ref().unwrap_or(&color_texture);
            let (img, distinct) = readback_rgba(&device, &queue, source, (width, height))?;
            save_capture(args, &img)?;
            return Ok(CaptureOutcome {
                out: args.out.clone(),
                size: (width, height),
                quads: frame.leaves,
                distinct_colors: distinct,
            });
        }
        let cap = crate::rt::capture_rt_with_device(&device, &queue, args)?;
        save_capture(args, &cap.image)?;
        return Ok(CaptureOutcome {
            out: args.out.clone(),
            size: cap.size,
            quads: cap.leaves,
            distinct_colors: cap.distinct_colors,
        });
    }

    // Same world, same meshing, same renderer as the windowed path.
    // Phase 2g.2: scene preparation is shared with the windowed
    // remesh (`scene_prep::prepare_raster_house`) — with a
    // textures dir the spec is built by the shared builder and
    // installed on the renderer; without one the flat capture is
    // unchanged (2f.4).
    let visible = CutawayMode::HideAbove(0).visible_levels(args.cutaway_z);
    let mut renderer = Renderer::new(&device, CAPTURE_FORMAT);
    let (meshes, stats, spec) = crate::scene_prep::prepare_raster_house(
        &visible,
        args.textures.as_deref(),
    )
    .map_err(CaptureError::Device)?;
    if let Some(spec) = &spec {
        renderer.set_textures(&device, &queue, spec);
    }
    let gpu_meshes = renderer.upload(&device, &meshes);

    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
    let fx_on = defs.is_some();
    let color_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("capture-color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | if fx_on {
                wgpu::TextureUsages::TEXTURE_BINDING
            } else {
                wgpu::TextureUsages::empty()
            },
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&Default::default());
    let (depth_texture, depth_view) = render::create_depth_texture(&device, (width, height));

    // Camera: target tile (x east, y south) -> world (x, z_up, -y).
    let camera = capture_camera(args);
    let aspect = width as f32 / height as f32;
    renderer.set_view_projection(&queue, camera.view_projection(aspect));
    renderer.draw(&device, &queue, &gpu_meshes, &color_view, &depth_view);

    // Phase 2g.1: field volumes + post stack over the rendered
    // world, when effects are enabled.
    let post_out = match &defs {
        Some(defs) => apply_fx(
            &device,
            &queue,
            args,
            defs,
            &camera,
            &color_view,
            &depth_texture,
            &depth_view,
            (width, height),
        )?,
        None => None,
    };
    let source = post_out.as_ref().unwrap_or(&color_texture);
    let (img, distinct) = readback_rgba(&device, &queue, source, (width, height))?;
    save_capture(args, &img)?;

    Ok(CaptureOutcome {
        out: args.out.clone(),
        size: (width, height),
        quads: stats.quads,
        distinct_colors: distinct,
    })
}
