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
use crate::world;

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

/// Phase 2f: stamp each non-air block of the (already
/// cutaway-filtered) map with its material's compact texture layers
/// from `spec`, so the meshers emit textured quads for it. Blocks
/// whose material has no texture entry (present flag 0, or id out of
/// range) keep `tex_layers: None` and stay flat.
fn stamp_tex_layers(map: &mut mesher::chunk::ChunkMap, spec: &treebuild::render::TextureSpec) {
    for chunk in map.chunks.values_mut() {
        let size = chunk.grid.size;
        for z in 0..size[2] {
            for y in 0..size[1] {
                for x in 0..size[0] {
                    let b = chunk.grid.get(x as i32, y as i32, z as i32);
                    if b.is_air() {
                        continue;
                    }
                    if let Some(ml) = spec.material_layers.get(b.id as usize) {
                        if ml[3] == 1 {
                            let mut stamped = b;
                            stamped.tex_layers = Some([ml[0], ml[1], ml[2]]);
                            chunk.grid.set(x, y, z, stamped);
                        }
                    }
                }
            }
        }
    }
}

/// Mesh an already-filtered map (the combine loop of
/// `world::mesh_world`, minus its filtering step — capture's
/// textured path filters and stamps before meshing).
fn mesh_filtered(
    map: &mesher::chunk::ChunkMap,
) -> (mesher::mesh::PassMeshes, crate::overlay::MeshStats) {
    let mut combined = mesher::mesh::PassMeshes::default();
    let mut n_chunks = 0;
    for coord in map.chunks.keys().copied().collect::<Vec<_>>() {
        if let Some(pm) = map.mesh_chunk(coord, true) {
            for (dst, src) in combined.meshes.iter_mut().zip(pm.meshes.iter()) {
                dst.append(src);
            }
            n_chunks += 1;
        }
    }
    let stats = crate::overlay::MeshStats::from_pass_meshes(n_chunks, &combined);
    (combined, stats)
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

    // Phase 2c ray-traced path: same world and cutaway selection,
    // rendered through the Sparse-Voxel-64Tree tracer instead of the
    // mesher/rasterizer.
    if args.renderer == RendererKind::Rt {
        let cap = crate::rt::capture_rt_with_device(&device, &queue, args)?;
        if let Some(parent) = args.out.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        cap.image
            .save(&args.out)
            .map_err(|e| CaptureError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        return Ok(CaptureOutcome {
            out: args.out.clone(),
            size: cap.size,
            quads: cap.leaves,
            distinct_colors: cap.distinct_colors,
        });
    }

    // Same world, same meshing, same renderer as the windowed path.
    let visible = CutawayMode::HideAbove(0).visible_levels(args.cutaway_z);
    let mut renderer = Renderer::new(&device, CAPTURE_FORMAT);
    let (meshes, stats) = match &args.textures {
        Some(dir) => {
            // Phase 2f raster textures: build the scene spec with
            // the SAME shared builder the RT path uses (gate 2f.3),
            // stamp the cutaway-filtered map's blocks with their
            // compact layers, mesh after stamping, and install the
            // spec on the renderer. Without `--textures` nothing
            // below runs and the flat capture is unchanged (2f.4).
            let table = crate::materials::MaterialTable::load(dir)
                .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
            let (spec, _tex_stats) =
                crate::textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())
                    .map_err(CaptureError::Device)?;
            let map = world::build_fake_house(mesher::scale::WorldScale::default());
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, &visible);
            stamp_tex_layers(&mut filtered, &spec);
            let (meshes, stats) = mesh_filtered(&filtered);
            renderer.set_textures(&device, &queue, &spec);
            (meshes, stats)
        }
        None => {
            let (_map, meshes, stats) = world::mesh_fake_house(&visible);
            (meshes, stats)
        }
    };
    let gpu_meshes = renderer.upload(&device, &meshes);

    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&Default::default());
    let depth_view = render::create_depth_view(&device, (width, height));

    // Camera: target tile (x east, y south) -> world (x, z_up, -y).
    let mut camera = OrbitCamera::new(Vec3::new(args.camera_target.0, 1.0, -args.camera_target.1));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    let aspect = width as f32 / height as f32;
    renderer.set_view_projection(&queue, camera.view_projection(aspect));
    renderer.draw(&device, &queue, &gpu_meshes, &color_view, &depth_view);

    // Readback: copy to a staging buffer with 256-byte row alignment.
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
            texture: &color_texture,
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
    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    img.save(&args.out)
        .map_err(|e| CaptureError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(CaptureOutcome {
        out: args.out.clone(),
        size: (width, height),
        quads: stats.quads,
        distinct_colors: distinct.len(),
    })
}
