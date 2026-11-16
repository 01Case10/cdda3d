//! voxel-engine binary.
//!
//! Modes:
//! - (no args) windowed winit + wgpu app: the live frame loop
//!   (Phase 2h) — every frame composed by `frame::render_frame`
//!   and presented; `--billboards`, `--light-hour`, `--ui-stub`,
//!   `--effects` seat the stub scene inputs.
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
use engine::frame::BillboardScene;
use engine::overlay::{resize_action, DebugOverlay, FpsCounter, FramePacer, ResizeAction};
use engine::world;
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
             [--textures voxelizer-output-dir] [--render-scale N] [--effects effects.json] \
             [--fx-fields-off] [--fx-post-off] [--billboards fixture|sample:<atlas-dir>|<atlas-dir>] \
             [--light-hour H] [--ui-stub] \
             --size WxH --out file.png] [--scene-report] \
             [--renderer raster|rt] [--textures dir] [--effects effects.json] \
             [--billboards fixture|sample:<atlas-dir>|<atlas-dir>] [--light-hour H] [--ui-stub]"
        );
        return ExitCode::SUCCESS;
    }
    let kind = value_after(&args, "--renderer")
        .and_then(|s| RendererKind::parse(&s))
        .unwrap_or(RendererKind::Raster);
    let textures = value_after(&args, "--textures").map(std::path::PathBuf::from);
    let effects = value_after(&args, "--effects").map(std::path::PathBuf::from);
    let billboards = match value_after(&args, "--billboards") {
        Some(s) if s == "fixture" => Some(BillboardScene::fixture_crowd()),
        Some(s) if s.starts_with("sample:") => {
            match BillboardScene::sample_crowd(std::path::Path::new(&s["sample:".len()..])) {
                Ok(scene) => Some(scene),
                Err(e) => {
                    eprintln!("voxel-engine: --billboards atlas load failed: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        Some(dir) => match BillboardScene::ultica_crowd(std::path::Path::new(&dir)) {
            Ok(scene) => Some(scene),
            Err(e) => {
                eprintln!("voxel-engine: --billboards atlas load failed: {e}");
                return ExitCode::from(2);
            }
        },
        _ => None,
    };
    let light_hour = value_after(&args, "--light-hour").and_then(|s| s.parse().ok());
    windowed_main(WindowConfig {
        renderer_kind: kind,
        textures_dir: textures,
        effects,
        billboards,
        ui_stub: args.iter().any(|a| a == "--ui-stub"),
        light_hour,
        vehicle_fixture: args.iter().any(|a| a == "--vehicle-fixture"),
        transient_fixture_ms: value_after(&args, "--transient-fixture")
            .and_then(|s| s.parse().ok()),
        render_scale: value_after(&args, "--render-scale")
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&v| v >= 1)
            .unwrap_or(1),
    })
}

/// Shared by the headless and windowed entry points: build the
/// Phase 2h scene inputs from CLI flags (`--billboards`,
/// `--light-hour`, `--ui-stub` at `ui_size`).
fn apply_scene_flags(
    cap: &mut CaptureArgs,
    args: &[String],
    ui_size: (u32, u32),
) -> Result<(), String> {
    if let Some(s) = value_after(args, "--billboards") {
        cap.billboards = Some(if s == "fixture" {
            BillboardScene::fixture_crowd()
        } else if let Some(dir) = s.strip_prefix("sample:") {
            BillboardScene::sample_crowd(std::path::Path::new(dir))?
        } else {
            BillboardScene::ultica_crowd(std::path::Path::new(&s))?
        });
    }
    if let Some(h) = value_after(args, "--light-hour") {
        let hour: f32 = h.parse().map_err(|_| format!("bad --light-hour '{h}'"))?;
        cap.lighting = Some(engine::frame::LightingScene::fake_house(
            hour,
            lighting::WeatherKind::Clear,
        ));
    }
    if let Some(b) = value_after(args, "--light-bake") {
        let mode = match b.as_str() {
            "per-vertex" => engine::frame::BakeMode::PerVertex,
            "per-block" => engine::frame::BakeMode::PerBlock,
            _ => return Err(format!("bad --light-bake '{b}' (per-vertex|per-block)")),
        };
        match &mut cap.lighting {
            Some(lit) => lit.bake_mode = mode,
            None => {
                let mut lit = engine::frame::LightingScene::fake_house(
                    12.0,
                    lighting::WeatherKind::Clear,
                );
                lit.bake_mode = mode;
                cap.lighting = Some(lit);
            }
        }
    }
    if args.iter().any(|a| a == "--ui-stub") {
        let (store, note) = engine::frame::stub_ui_store(ui_size.0, ui_size.1, false)?;
        cap.ui = Some(engine::frame::prepare_ui(&store, &note)?);
    }
    // Phase 8 wiring demo scenes: the fixture car parked inside
    // the fake house, and the transient burst at a fixed clock.
    if args.iter().any(|a| a == "--vehicle-fixture") {
        cap.vehicles = Some(engine::frame::VehicleScene::fixture_car([4, 5, 0], 23.0));
    }
    if let Some(s) = value_after(args, "--transient-fixture") {
        let now_ms: u64 = s.parse().map_err(|_| format!("bad --transient-fixture '{s}'"))?;
        cap.transients = Some(engine::frame::TransientScene::fixture_burst(now_ms));
    }
    Ok(())
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
    if let Some(s) = value_after(args, "--render-scale") {
        match s.parse::<u32>() {
            Ok(v) if v >= 1 => cap.render_scale = v,
            _ => {
                eprintln!("headless-capture: --render-scale must be an integer >= 1 (got '{s}')");
                return ExitCode::from(2);
            }
        }
    }
    if let Some(e) = value_after(args, "--effects") {
        cap.effects = Some(e.into());
    }
    if args.iter().any(|a| a == "--fx-fields-off") {
        cap.fx_fields = false;
    }
    if args.iter().any(|a| a == "--fx-post-off") {
        cap.fx_post = false;
    }
    if let Some(o) = value_after(args, "--out") {
        cap.out = o.into();
    }
    let ui_size = cap.size;
    if let Err(e) = apply_scene_flags(&mut cap, args, ui_size) {
        eprintln!("headless-capture: {e}");
        return ExitCode::from(2);
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
// Windowed app (Phase 2h: a thin shell over the live frame loop —
// every frame is composed by `frame::render_frame`, the same
// function the headless capture reads back, and presented with a
// plain blit. There is no second, windowed-only composition.)
// ---------------------------------------------------------------------

/// Everything the windowed app was launched with.
struct WindowConfig {
    renderer_kind: RendererKind,
    textures_dir: Option<std::path::PathBuf>,
    effects: Option<std::path::PathBuf>,
    billboards: Option<engine::frame::BillboardScene>,
    ui_stub: bool,
    light_hour: Option<f32>,
    vehicle_fixture: bool,
    transient_fixture_ms: Option<u64>,
    render_scale: u32,
}

struct GpuState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    blit: engine::frame::PresentBlit,
    size: (u32, u32),
}

struct App {
    gpu: Option<GpuState>,
    /// The frame template `render_frame` runs each redraw; camera,
    /// cutaway, size and animation time are synced into it in
    /// place (no per-frame clone — the billboard atlas is an
    /// image).
    frame_args: CaptureArgs,
    camera: OrbitCamera,
    cutaway: CutawayState,
    overlay: DebugOverlay,
    fps_counter: FpsCounter,
    pacer: FramePacer,
    /// Stub UI store + router (Phase 2h.4 windowed half): clicks
    /// route through the Phase 4 input router and land in
    /// `input_sink` — the engine has no IPC socket client yet, so
    /// the sink is an in-process log standing in for the fork's
    /// input channel (live seating is 0.1/0.2 territory).
    ui_store: Option<ui_share::UiShmStore>,
    router: Option<ui_share::input::InputRouter>,
    input_sink: Vec<proto::InputT>,
    cursor: (f64, f64),
    drag_from: Option<(f64, f64)>,
    started: Instant,
}

impl App {
    fn new(cfg: WindowConfig) -> Self {
        // Follow target: house center tile (6,6) -> world (6, top, -6).
        let camera = OrbitCamera::new(Vec3::new(6.0, 1.0, -6.0));
        let mut frame_args = CaptureArgs {
            renderer: cfg.renderer_kind,
            render_scale: cfg.render_scale,
            textures: cfg.textures_dir,
            effects: cfg.effects,
            billboards: cfg.billboards,
            lighting: cfg
                .light_hour
                .map(|h| engine::frame::LightingScene::fake_house(h, lighting::WeatherKind::Clear)),
            vehicles: cfg
                .vehicle_fixture
                .then(|| engine::frame::VehicleScene::fixture_car([4, 5, 0], 23.0)),
            transients: cfg
                .transient_fixture_ms
                .map(engine::frame::TransientScene::fixture_burst),
            ..CaptureArgs::default()
        };
        // Stub UI: a fixed 640x360 synthetic frame in an in-memory
        // store (the compositor stretches it to the window).
        let (ui_store, router) = if cfg.ui_stub {
            match engine::frame::stub_ui_store(640, 360, false) {
                Ok((store, note)) => match engine::frame::prepare_ui(&store, &note) {
                    Ok(prepared) => {
                        let router =
                            ui_share::input::InputRouter::new(prepared.mode, prepared.terrain_rect);
                        frame_args.ui = Some(prepared);
                        (Some(store), Some(router))
                    }
                    Err(e) => {
                        eprintln!("voxel-engine: stub UI prepare failed: {e}");
                        (None, None)
                    }
                },
                Err(e) => {
                    eprintln!("voxel-engine: stub UI store failed: {e}");
                    (None, None)
                }
            }
        } else {
            (None, None)
        };
        Self {
            gpu: None,
            frame_args,
            camera,
            cutaway: CutawayState::default(),
            overlay: DebugOverlay::new(),
            fps_counter: FpsCounter::new(),
            pacer: FramePacer::new(),
            ui_store,
            router,
            input_sink: Vec::new(),
            cursor: (0.0, 0.0),
            drag_from: None,
            started: Instant::now(),
        }
    }

    /// Scene inputs (cutaway, textures, lighting) are read by
    /// `render_frame` from `frame_args` + `cutaway` at draw time,
    /// so a "remesh" is just a dirty mark now.
    fn remesh(&mut self) {
        self.pacer.mark_world_dirty();
    }

    /// Sync the live camera/cutaway/size/time into the frame
    /// template before a redraw.
    fn sync_frame_args(&mut self) {
        let a = &mut self.frame_args;
        a.camera_target = (self.camera.target.x, -self.camera.target.z);
        a.yaw_deg = self.camera.yaw_deg;
        a.pitch_deg = self.camera.pitch_deg;
        a.distance = self.camera.distance;
        a.cutaway_z = self
            .cutaway
            .visible_levels()
            .last()
            .copied()
            .unwrap_or(a.cutaway_z);
        if let Some(gpu) = &self.gpu {
            a.size = gpu.size;
        }
        let t = self.started.elapsed().as_secs_f32();
        a.fx_time = t;
        if let Some(scene) = &mut a.billboards {
            scene.time_s = t;
        }
    }

    fn render(&mut self) {
        let Some(size) = self.gpu.as_ref().map(|g| g.size) else {
            return;
        };
        if !engine::overlay::renderable(size) {
            return; // minimized: skip, never present to a zero-size surface
        }
        self.sync_frame_args();
        let Some(gpu) = &self.gpu else { return };
        let frame = match gpu.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        match engine::frame::render_frame(&gpu.device, &gpu.queue, &self.frame_args) {
            Ok(outcome) => {
                if let Some(stats) = &outcome.stats.mesh_stats {
                    self.overlay.stats = *stats;
                }
                gpu.blit
                    .draw(&gpu.device, &gpu.queue, &outcome.texture, &view);
            }
            Err(e) => eprintln!("voxel-engine: frame failed: {e}"),
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

    /// Route one left-click press through the Phase 4 input
    /// router (when the stub UI is up): a world-pick candidate runs
    /// the DDA pick against the fake house and the click lands in
    /// the input sink as a protocol `Input` message; a panel click
    /// lands as the router's own `InputT`; a press outside the UI
    /// quad starts a camera drag when the mode allows it. Without
    /// a router every press starts a camera drag.
    fn handle_click_press(&mut self) {
        let Some(gpu) = &self.gpu else { return };
        let window = ui_share::Size2 { w: gpu.size.0, h: gpu.size.1 };
        let Some(router) = self.router.clone() else {
            self.drag_from = Some(self.cursor);
            return;
        };
        // The stub UI canvas is 640x360, stretched over the window
        // (the same mode the compositor draws it with).
        let ui_size = ui_share::Size2 { w: 640, h: 360 };
        let viewport =
            ui_share::scaling::compute_viewport(window, ui_size, ui_share::ScalingMode::StretchNearest);
        match router.route_window_click(
            self.cursor.0 as f32,
            self.cursor.1 as f32,
            &viewport,
            ui_size,
            0,
            true,
            ui_share::input::Modifiers::default(),
        ) {
            ui_share::input::ClickRoute::WorldPick { ui_x, ui_y } => {
                let occupied = engine::frame::fake_house_occupied();
                let request_id = self.input_sink.len() as u64 + 1;
                let pick = ui_share::picking::pick_from_ui_click(
                    request_id,
                    &router,
                    &engine::frame::pick_camera(&self.camera),
                    ui_x,
                    ui_y,
                    occupied,
                );
                match pick {
                    Some(r) if r.hit => {
                        let pos = r.pos.unwrap_or_default();
                        eprintln!(
                            "voxel-engine: pick #{} hit tile ({}, {}, {}) [stub sink]",
                            request_id, pos.x, pos.y, pos.z
                        );
                    }
                    Some(_) => eprintln!("voxel-engine: pick #{request_id} missed [stub sink]"),
                    None => eprintln!("voxel-engine: pick #{request_id} not a world pick"),
                }
                let mut click = proto::InputT::default();
                click.kind = proto::InputKind::MouseButton;
                click.mouse_x = ui_x;
                click.mouse_y = ui_y;
                click.button = 0;
                click.pressed = true;
                click.focused = true;
                self.input_sink.push(click);
            }
            ui_share::input::ClickRoute::UiClick(input) => {
                eprintln!(
                    "voxel-engine: UI click ({}, {}) [stub sink]",
                    input.mouse_x, input.mouse_y
                );
                self.input_sink.push(input);
            }
            ui_share::input::ClickRoute::Ignored => {
                if router.camera_drag_allowed() {
                    self.drag_from = Some(self.cursor);
                }
            }
        }
    }

    /// Game keys that are not camera controls also produce
    /// protocol inputs in the stub sink (Phase 4 key mapping).
    fn handle_game_key(&mut self, key: ui_share::input::EngineKey) {
        let input = ui_share::input::key_to_input(key, ui_share::input::Modifiers::default(), true);
        eprintln!(
            "voxel-engine: key input code {} [stub sink]",
            input.key_code
        );
        self.input_sink.push(input);
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
        let blit = engine::frame::PresentBlit::new(&device, format);
        self.gpu = Some(GpuState {
            window,
            surface,
            device,
            queue,
            config,
            blit,
            size,
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
                        self.pacer.mark_world_dirty();
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::CursorMoved { position, .. } => {
                let prev = self.cursor;
                self.cursor = (position.x, position.y);
                if let Some(from) = self.drag_from {
                    let dx = (self.cursor.0 - prev.0) as f32;
                    let dy = (self.cursor.1 - prev.1) as f32;
                    let _ = from;
                    if dx != 0.0 || dy != 0.0 {
                        self.camera.orbit(dx * 0.3, -dy * 0.3, 0.0);
                        self.pacer.mark_camera_dirty();
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    match state {
                        ElementState::Pressed => self.handle_click_press(),
                        ElementState::Released => self.drag_from = None,
                    }
                }
            }
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
                    PhysicalKey::Code(KeyCode::KeyZ) => {
                        camera_moved = false;
                        self.handle_game_key(ui_share::input::EngineKey::Char('z'));
                    }
                    PhysicalKey::Code(KeyCode::Enter) => {
                        camera_moved = false;
                        self.handle_game_key(ui_share::input::EngineKey::Enter);
                    }
                    PhysicalKey::Code(KeyCode::KeyI) => {
                        camera_moved = false;
                        self.handle_game_key(ui_share::input::EngineKey::Char('i'));
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

fn windowed_main(cfg: WindowConfig) -> ExitCode {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(cfg);
    match event_loop.run_app(&mut app) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("voxel-engine: event loop error: {e}");
            ExitCode::from(2)
        }
    }
}
