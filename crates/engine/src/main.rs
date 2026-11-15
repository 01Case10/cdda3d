//! voxel-engine binary.
//!
//! Modes:
//! - (no args) windowed winit + wgpu app rendering the fake house.
//! - `--headless-capture --snapshot stub-house --camera x,y,yaw,pitch
//!   [--distance d] [--cutaway z] --size WxH --out file.png`:
//!   offscreen render to PNG, no window / surface / event loop.
//!   Exits non-zero with a named error when no wgpu adapter exists.
//! - `--scene-report`: build + mesh the full 132x132 bubble scene on
//!   CPU and print counts/timing (the 2.13 data-level proxy). No GPU.

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use engine::camera::OrbitCamera;
use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::cutaway::CutawayState;
use engine::overlay::{resize_action, DebugOverlay, FpsCounter, FramePacer, ResizeAction};
use engine::render::{create_depth_view, Renderer};
use engine::rt::RtBlit;
use engine::world;
use treebuild::render::{RtCamera, RtRenderer, RtScene};
use glam::Vec3;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--headless-capture") {
        return headless_main(&args);
    }
    if args.iter().any(|a| a == "--scene-report") {
        let r = world::bubble_report();
        println!(
            "bubble scene (2.13 data proxy): {} chunks, {} tiles stored, {} z-levels, {} non-air blocks",
            r.chunks, r.tiles_stored, r.levels, r.non_air_blocks
        );
        println!(
            "meshed (greedy, all levels visible): {} quads, {} vertices in {:.1} ms total ({:.3} ms/chunk)",
            r.quads,
            r.vertices,
            r.mesh_ms,
            r.mesh_ms / r.chunks as f64
        );
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "usage: voxel-engine [--headless-capture --snapshot stub-house \
             --camera x,y,yaw,pitch [--distance d] [--cutaway z] [--renderer raster|rt] \
             [--textures voxelizer-output-dir] \
             --size WxH --out file.png] [--scene-report] [--renderer raster|rt] [--textures dir]"
        );
        return ExitCode::SUCCESS;
    }
    let kind = value_after(&args, "--renderer")
        .and_then(|s| RendererKind::parse(&s))
        .unwrap_or(RendererKind::Raster);
    let textures = value_after(&args, "--textures").map(std::path::PathBuf::from);
    windowed_main(kind, textures)
}

fn value_after(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn headless_main(args: &[String]) -> ExitCode {
    let mut cap = CaptureArgs::default();
    if let Some(s) = value_after(args, "--snapshot") {
        cap.snapshot = s;
    }
    if let Some(c) = value_after(args, "--camera") {
        let parts: Vec<f32> = c.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if parts.len() == 4 {
            cap.camera_target = (parts[0], parts[1]);
            cap.yaw_deg = parts[2];
            cap.pitch_deg = parts[3];
        } else {
            eprintln!("headless-capture: --camera must be x,y,yaw,pitch (got '{c}')");
            return ExitCode::from(2);
        }
    }
    if let Some(d) = value_after(args, "--distance") {
        match d.parse() {
            Ok(v) => cap.distance = v,
            Err(_) => {
                eprintln!("headless-capture: bad --distance '{d}'");
                return ExitCode::from(2);
            }
        }
    }
    if let Some(z) = value_after(args, "--cutaway") {
        match z.parse() {
            Ok(v) => cap.cutaway_z = v,
            Err(_) => {
                eprintln!("headless-capture: bad --cutaway '{z}'");
                return ExitCode::from(2);
            }
        }
    }
    if let Some(s) = value_after(args, "--size") {
        let mut it = s.split(['x', 'X']);
        let (w, h) = (
            it.next().and_then(|v| v.parse().ok()),
            it.next().and_then(|v| v.parse().ok()),
        );
        match (w, h) {
            (Some(w), Some(h)) => cap.size = (w, h),
            _ => {
                eprintln!("headless-capture: --size must be WxH (got '{s}')");
                return ExitCode::from(2);
            }
        }
    }
    if let Some(r) = value_after(args, "--renderer") {
        match RendererKind::parse(&r) {
            Some(k) => cap.renderer = k,
            None => {
                eprintln!("headless-capture: --renderer must be raster|rt (got '{r}')");
                return ExitCode::from(2);
            }
        }
    }
    if let Some(t) = value_after(args, "--textures") {
        cap.textures = Some(t.into());
    }
    if let Some(o) = value_after(args, "--out") {
        cap.out = o.into();
    }
    match run_capture(&cap) {
        Ok(outcome) => {
            println!(
                "headless-capture: wrote {} ({}x{}, renderer {}, {} {}, {} distinct colors)",
                outcome.out.display(),
                outcome.size.0,
                outcome.size.1,
                cap.renderer.name(),
                outcome.quads,
                if cap.renderer == RendererKind::Rt {
                    "voxels"
                } else {
                    "quads"
                },
                outcome.distinct_colors
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

// ---------------------------------------------------------------------
// Windowed app (thin shell over the lib; shared Renderer with capture)
// ---------------------------------------------------------------------

struct GpuState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    gpu_meshes: Vec<engine::render::GpuMesh>,
    depth_view: wgpu::TextureView,
    size: (u32, u32),
    // Phase 2c: present only when --renderer rt was requested.
    rt_renderer: Option<RtRenderer>,
    rt_scene: Option<RtScene>,
    rt_blit: Option<RtBlit>,
}

struct App {
    gpu: Option<GpuState>,
    camera: OrbitCamera,
    cutaway: CutawayState,
    overlay: DebugOverlay,
    fps_counter: FpsCounter,
    pacer: FramePacer,
    renderer_kind: RendererKind,
    textures_dir: Option<std::path::PathBuf>,
}

impl App {
    fn new(renderer_kind: RendererKind, textures_dir: Option<std::path::PathBuf>) -> Self {
        // Follow target: house center tile (6,6) -> world (6, top, -6).
        let camera = OrbitCamera::new(Vec3::new(6.0, 1.0, -6.0));
        Self {
            gpu: None,
            camera,
            cutaway: CutawayState::default(),
            overlay: DebugOverlay::new(),
            fps_counter: FpsCounter::new(),
            pacer: FramePacer::new(),
            renderer_kind,
            textures_dir,
        }
    }

    fn remesh(&mut self) {
        let Some(gpu) = &self.gpu else { return };
        let visible = self.cutaway.visible_levels();
        if self.renderer_kind == RendererKind::Rt {
            // Rebuild the tree for the new cutaway selection (the
            // windowed RT path rebuilds renderer+scene per remesh;
            // cutaway changes are rare, per-frame rendering reuses
            // the scene).
            let mut voxel_world = engine::rt::world_for_levels(&visible);
            // Phase 2d: variant-prepare the world before the tree
            // build when textures are on (same flow as the capture
            // path); a prepare failure leaves the base world, and
            // the attach below reports the underlying error.
            let assignments = match &self.textures_dir {
                Some(dir) => engine::rt::prepare_fake_variants(&mut voxel_world, dir)
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            match treebuild::build::build_tree(&voxel_world, treebuild::build::BuildConfig::default()) {
                Ok(built) => {
                    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
                    if let Some(dir) = &self.textures_dir {
                        match engine::rt::attach_fake_textures(&gpu.device, &gpu.queue, &mut scene, dir, &assignments) {
                            Ok(stats) => eprintln!(
                                "voxel-engine: RT textures: {} materials, {} layers",
                                stats.textured_materials, stats.layers_uploaded
                            ),
                            Err(e) => eprintln!("voxel-engine: RT textures unavailable: {e}"),
                        }
                    }
                    let gpu = self.gpu.as_mut().unwrap();
                    gpu.rt_renderer = Some(renderer);
                    gpu.rt_scene = Some(scene);
                }
                Err(e) => eprintln!("voxel-engine: RT tree build failed: {e}"),
            }
            self.pacer.mark_world_dirty();
            return;
        }
        let (_map, meshes, stats) = world::mesh_fake_house(&visible);
        self.overlay.stats = stats;
        let gpu_meshes = gpu.renderer.upload(&gpu.device, &meshes);
        self.gpu.as_mut().unwrap().gpu_meshes = gpu_meshes;
        self.pacer.mark_world_dirty();
    }

    fn render(&mut self) {
        let Some(gpu) = &self.gpu else { return };
        if !engine::overlay::renderable(gpu.size) {
            return; // minimized: skip, never present to a zero-size surface
        }
        let aspect = gpu.size.0 as f32 / gpu.size.1 as f32;
        let frame = match gpu.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        if self.renderer_kind == RendererKind::Rt {
            if let (Some(rt_renderer), Some(rt_scene), Some(rt_blit)) =
                (&gpu.rt_renderer, &gpu.rt_scene, &gpu.rt_blit)
            {
                let rt_camera = RtCamera::from_world(
                    self.camera.eye(),
                    self.camera.target,
                    60.0,
                    aspect,
                );
                let rt_frame =
                    rt_renderer.render(&gpu.device, &gpu.queue, rt_scene, &rt_camera, gpu.size, true);
                rt_blit.draw(&gpu.device, &gpu.queue, &rt_frame.color, &view);
            }
        } else {
            gpu.renderer
                .set_view_projection(&gpu.queue, self.camera.view_projection(aspect));
            gpu.renderer.draw(
                &gpu.device,
                &gpu.queue,
                &gpu.gpu_meshes,
                &view,
                &gpu.depth_view,
            );
        }
        frame.present();
        self.overlay.fps = self.fps_counter.tick(Instant::now());
        gpu.window.set_title(&format!(
            "CDDA Voxel — fake house | {} | cutaway z<={}",
            self.overlay.summary(),
            self.cutaway.visible_levels().last().copied().unwrap_or(0)
        ));
        self.pacer.rendered();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("CDDA Voxel"))
                .expect("create window"),
        );
        let size = window.inner_size();
        let size = (size.width.max(1), size.height.max(1));
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("no wgpu adapter available for windowed mode");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("windowed-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .expect("request device");
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let renderer = Renderer::new(&device, format);
        let depth_view = create_depth_view(&device, size);
        let rt_blit = if self.renderer_kind == RendererKind::Rt {
            Some(RtBlit::new(&device, format))
        } else {
            None
        };
        self.gpu = Some(GpuState {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            gpu_meshes: Vec::new(),
            depth_view,
            size,
            rt_renderer: None,
            rt_scene: None,
            rt_blit,
        });
        self.remesh();
        self.gpu.as_ref().unwrap().window.request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.pacer.should_render() {
            if let Some(gpu) = &self.gpu {
                gpu.window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                let Some(gpu) = &mut self.gpu else { return };
                match resize_action(gpu.size, (new_size.width, new_size.height)) {
                    ResizeAction::Ignore => {}
                    ResizeAction::Reconfigure { width, height } => {
                        gpu.config.width = width;
                        gpu.config.height = height;
                        gpu.size = (width, height);
                        gpu.surface.configure(&gpu.device, &gpu.config);
                        gpu.depth_view = create_depth_view(&gpu.device, gpu.size);
                        self.pacer.mark_world_dirty();
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                let mut camera_moved = true;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                        return;
                    }
                    PhysicalKey::Code(KeyCode::ArrowLeft) => self.camera.orbit(-5.0, 0.0, 0.0),
                    PhysicalKey::Code(KeyCode::ArrowRight) => self.camera.orbit(5.0, 0.0, 0.0),
                    PhysicalKey::Code(KeyCode::ArrowUp) => self.camera.orbit(0.0, 3.0, 0.0),
                    PhysicalKey::Code(KeyCode::ArrowDown) => self.camera.orbit(0.0, -3.0, 0.0),
                    PhysicalKey::Code(KeyCode::KeyQ) => self.camera.orbit(0.0, 0.0, -2.0),
                    PhysicalKey::Code(KeyCode::KeyE) => self.camera.orbit(0.0, 0.0, 2.0),
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        let t = self.camera.target + Vec3::new(0.0, 0.0, 1.0);
                        self.camera.follow(t);
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        let t = self.camera.target + Vec3::new(0.0, 0.0, -1.0);
                        self.camera.follow(t);
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        let t = self.camera.target + Vec3::new(-1.0, 0.0, 0.0);
                        self.camera.follow(t);
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        let t = self.camera.target + Vec3::new(1.0, 0.0, 0.0);
                        self.camera.follow(t);
                    }
                    PhysicalKey::Code(KeyCode::PageUp) => {
                        self.cutaway.step_up();
                        camera_moved = false;
                        self.remesh();
                    }
                    PhysicalKey::Code(KeyCode::PageDown) => {
                        self.cutaway.step_down();
                        camera_moved = false;
                        self.remesh();
                    }
                    PhysicalKey::Code(KeyCode::KeyB) => {
                        self.overlay.show_chunk_borders = !self.overlay.show_chunk_borders;
                        camera_moved = false;
                        self.pacer.mark_world_dirty();
                    }
                    _ => camera_moved = false,
                }
                if camera_moved {
                    self.pacer.mark_camera_dirty();
                }
            }
            _ => {}
        }
    }
}

fn windowed_main(
    renderer_kind: RendererKind,
    textures_dir: Option<std::path::PathBuf>,
) -> ExitCode {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(renderer_kind, textures_dir);
    match event_loop.run_app(&mut app) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("voxel-engine: event loop error: {e}");
            ExitCode::from(2)
        }
    }
}
