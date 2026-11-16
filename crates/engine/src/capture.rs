//! Headless capture mode (Phase 2 deliverable, plan §5/§7, test 2.10).
//!
//! Offscreen wgpu rendering with no window, no surface, and no event
//! loop: `Instance`/`Adapter`/`Device` with `compatible_surface: None`,
//! then the shared live frame (`frame::render_frame` — the same
//! composition the windowed app presents) into an offscreen
//! `Rgba8UnormSrgb` texture, read back, and written as a PNG.
//!
//! On a machine with no wgpu adapter this returns
//! `CaptureError::NoAdapter` — a named, clean failure. It never panics
//! and never writes a blank PNG presented as a pass.

use std::fmt;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct CaptureArgs {
    /// Scene source: "stub-house" (the Phase 2 fixture) or
    /// "sample-scene" (the 24x24 yard + house sample world).
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
    /// Stage A internal resolution: when > 1, the frame is
    /// composed at `size / render_scale` and integer-upscaled with
    /// nearest filtering into the full-size output (the
    /// PresentBlit pass), for a consistent texel density and the
    /// deliberate pixel look. 1 = render at full size (the
    /// pre-Stage-A behavior, byte-for-byte). Sizes should be
    /// multiples of the scale for an exact integer upscale.
    pub render_scale: u32,
    /// Stage F proof mode (`--macro-debug-height`): render macro
    /// materials' top faces as their height-layer grayscale
    /// instead of color, in both renderers. Off = normal render.
    pub macro_debug_height: bool,
    pub out: PathBuf,
    /// Player z for the cutaway (levels <= this z are drawn).
    pub cutaway_z: i32,
    /// Phase 2h.3: lighting scene (game light grid + environment)
    /// for the visibility mask + bake. `None` = the pre-2h
    /// unlit render, byte-for-byte.
    pub lighting: Option<crate::frame::LightingScene>,
    /// Phase 2h.2: billboard scene drawn between the fields and
    /// the post. `None` = no billboard pass.
    pub billboards: Option<crate::frame::BillboardScene>,
    /// Phase 2h.4: prepared UI frame composited last, over the
    /// post-processed world. `None` = no UI pass.
    pub ui: Option<crate::frame::UiPrepared>,
    /// Phase 8 wiring: vehicles stamped into the world pass as
    /// blocks (their exported absolute part positions, verbatim).
    /// `None` = the pre-wiring render, byte-for-byte.
    pub vehicles: Option<crate::frame::VehicleScene>,
    /// Phase 8 wiring: transient emitters (animations /
    /// projectiles from `Effects` messages) drawn in the fields
    /// pass through the fx shader classes. `None` = no emitters.
    pub transients: Option<crate::frame::TransientScene>,
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
            render_scale: 1,
            macro_debug_height: false,
            out: PathBuf::from("captures/house.png"),
            cutaway_z: 1,
            lighting: None,
            billboards: None,
            ui: None,
            vehicles: None,
            transients: None,
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
                "headless-capture: unsupported --snapshot source '{s}' (supports 'stub-house' and 'sample-scene')"
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
///
/// Phase 2h: the capture is the live frame loop read back —
/// `frame::render_frame` composes world → fields → billboards →
/// post → UI into one offscreen texture (the same function the
/// windowed app presents), and the PNG is that texture. Before
/// 2h this function held its own copy of the composition; the
/// golden harness (2g.3) pins that the delegation is
/// byte-stable for the pre-2h scenes.
pub fn run_capture(args: &CaptureArgs) -> Result<CaptureOutcome, CaptureError> {
    if args.snapshot != "stub-house" && args.snapshot != "sample-scene" {
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

    let frame = crate::frame::render_frame(&device, &queue, args)?;
    let (img, distinct) =
        crate::frame::readback_rgba(&device, &queue, &frame.texture, frame.size)?;
    save_capture(args, &img)?;
    Ok(CaptureOutcome {
        out: args.out.clone(),
        size: frame.size,
        quads: frame.quads,
        distinct_colors: distinct,
    })
}
