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
//! - `--connect <unix-socket>` (Phase 11 S7): a headless live
//!   session against a running game server (sim-server or the
//!   fork's IPC frontend) — handshake, wire-fed world ingest,
//!   stdin keys -> Input messages, heartbeat, clean shutdown.
//!   See `engine::client`.
//! - `--connect-window <unix-socket>` (Phase 11 follow-up,
//!   Lane A): the same session, seated in the windowed app —
//!   the session runs on its own thread and publishes its live
//!   wire world (`CaptureArgs::wire`, the wire-source change);
//!   the window renders that world through the one frame loop
//!   and forwards key presses back as Input messages (camera
//!   keys stay local; closing the window ends the session via
//!   the client's input-close rule). The windowed run itself
//!   needs a desktop — on this VM only the headless seating is
//!   proven (the wire-source gate's connect test).

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
use winit::keyboard::{Key, KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--headless-capture") {
        return headless_main(&args);
    }
    if let Some(socket) = value_after(&args, "--connect") {
        return match engine::client::run_connect(std::path::Path::new(&socket)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("voxel-engine --connect: {e}");
                ExitCode::FAILURE
            }
        };
    }
    // --connect-window: spawn the live session on its thread
    // (it owns the socket), then run the windowed app with the
    // session's published wire scene + key channel linked in.
    // The app holds the channel sender; when the window closes
    // the app drops, the channel closes, and the session ends
    // cleanly (drain + Shutdown) — joined below, bounded by
    // the client's own drain cap.
    let mut live_session: Option<std::thread::JoinHandle<Result<engine::client::ConnectReport, String>>> =
        None;
    let live_link = match value_after(&args, "--connect-window") {
        Some(socket) => {
            let (tx, rx) = std::sync::mpsc::channel::<i32>();
            let share: engine::client::SharedWireScene =
                std::sync::Arc::new(std::sync::Mutex::new(None));
            let ui_share: engine::client::SharedUiScene =
                std::sync::Arc::new(std::sync::Mutex::new(None));
            live_session = Some(engine::client::spawn_connect(
                std::path::Path::new(&socket),
                engine::client::ConnectConfig {
                    input: engine::client::ConnectInput::Channel(rx),
                    share: Some(share.clone()),
                    ui_share: Some(ui_share.clone()),
                    captures: None,
                    // Lane G: `--textures <voxelizer-output-dir>`
                    // doubles as the furniture model table
                    // source for the live world (the same dir
                    // the fixture paths load materials from).
                    models_dir: value_after(&args, "--textures")
                        .map(std::path::PathBuf::from),
                    // The same voxelizer dir holds the sprite
                    // atlas (`sprites.json`/`sprites.png` +
                    // `tile_sprites.json`): seat it for the
                    // session's captures too, so the windowed
                    // session and a headless capture of the same
                    // world draw the same entity billboards. The
                    // App seats its own copy for the live frame
                    // (see `App::new`).
                    billboards_dir: value_after(&args, "--textures")
                        .map(std::path::PathBuf::from),
                },
            ));
            Some(LiveWindowLink {
                share,
                ui_share,
                input: tx,
            })
        }
        None => None,
    };
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
             [--light-hour H] [--ui-stub] [--fine-props] \
             --size WxH --out file.png] [--scene-report] \
             [--renderer raster|rt] [--textures dir] [--effects effects.json] \
             [--billboards fixture|sample:<atlas-dir>|<atlas-dir>] [--light-hour H] [--ui-stub] \
             [--fine-props] [--connect <unix-socket>] [--connect-window <unix-socket>]"
        );
        return ExitCode::SUCCESS;
    }
    // Renderer default (Lane H, Joey's call 2026-11-24): a live
    // `--connect-window` session defaults to RT — ray tracing is
    // the default renderer for the live launch, and `--renderer
    // raster` is the opt-out. The fixture app keeps its raster
    // default (its gates and scripts name their renderer
    // explicitly), and an explicit `--renderer` always wins in
    // both. (This supersedes the pre-Lane-H stance that raster
    // stays default until RT frame time is measured on Joey's
    // 9070 XT; the measurement is still outstanding, now
    // post-decision.)
    let kind = match value_after(&args, "--renderer") {
        Some(s) => RendererKind::parse(&s).unwrap_or(RendererKind::Raster),
        None if live_link.is_some() => RendererKind::Rt,
        None => RendererKind::Raster,
    };
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
    let code = windowed_main(WindowConfig {
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
        fine_props: args.iter().any(|a| a == "--fine-props"),
        live: live_link,
    });
    // The window is closed and the app (with the key channel's
    // sender) is dropped: the session is draining to its clean
    // Shutdown. Join it so its report/exit is not lost.
    if let Some(handle) = live_session {
        match handle.join() {
            Ok(Ok(report)) => println!(
                "voxel-engine --connect-window: session ended — {} snapshots, {} deltas, {} inputs sent; player at {:?}",
                report.snapshots, report.deltas, report.inputs_sent, report.player_pos
            ),
            Ok(Err(e)) => eprintln!("voxel-engine --connect-window: session error: {e}"),
            Err(_) => eprintln!("voxel-engine --connect-window: session thread panicked"),
        }
    }
    code
}

/// The windowed app's half of a `--connect-window` session:
/// the shared slot the session thread publishes its live wire
/// scene into, and the channel key presses are forwarded to.
struct LiveWindowLink {
    share: engine::client::SharedWireScene,
    ui_share: engine::client::SharedUiScene,
    input: std::sync::mpsc::Sender<i32>,
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
                let mut lit =
                    engine::frame::LightingScene::fake_house(12.0, lighting::WeatherKind::Clear);
                lit.bake_mode = mode;
                cap.lighting = Some(lit);
            }
        }
    }
    // Phase 10 (§6): named sky environments for captures, so the
    // gate capture set is deterministic. The names resolve
    // through `SkyScene::named` — the same resolver the gate
    // matrix uses (slice 10.6), so the CLI and the gates cannot
    // drift. (The live windowed path takes its environment from
    // the client state; these names are the fake-data scripts.)
    if let Some(s) = value_after(args, "--sky") {
        cap.sky = Some(engine::frame::SkyScene::named(&s).ok_or_else(|| {
            format!(
                "bad --sky '{s}' (neutral|{{midnight,dawn,noon,dusk}}-{{clear,rain,fog,thunderstorm}})"
            )
        })?);
    }
    // Addendum 11 (emitter prototype): named emitter fixture
    // scenes, resolved through `EmitterScene::named` — the same
    // resolver the gate uses. The scene seats its companions
    // (sky / vehicles / lighting) wherever the args don't
    // already carry one, so the prototype capture is one flag.
    if let Some(s) = value_after(args, "--emitters") {
        let scene = engine::frame::EmitterScene::named(&s)
            .ok_or_else(|| format!("bad --emitters '{s}' (headlight-car)"))?;
        scene.seat_companions(cap);
        cap.emitters = Some(scene);
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
        let now_ms: u64 = s
            .parse()
            .map_err(|_| format!("bad --transient-fixture '{s}'"))?;
        cap.transients = Some(engine::frame::TransientScene::fixture_burst(now_ms));
    }
    // Phase 9 slice 2 rollout (9.10): fine prop stamping,
    // default OFF; parsed here so headless and windowed both
    // get it.
    if args.iter().any(|a| a == "--fine-props") {
        cap.fine_props = true;
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
    if args.iter().any(|a| a == "--macro-debug-height") {
        cap.macro_debug_height = true;
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
    fine_props: bool,
    /// `--connect-window`: seat the live session's wire world
    /// (see `LiveWindowLink`). `None` = the fixture app.
    live: Option<LiveWindowLink>,
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
    /// RT scene cache for the live loop: the built RT scene
    /// (trees + textures + pipelines) is reused across frames
    /// until the scene-determining inputs change — without it
    /// every presented frame repaid the full build and the RT
    /// demo ran at 0 fps.
    rt_cache: Option<engine::rt::RtSceneCache>,
    /// Stub UI store + router (Phase 2h.4 windowed half): clicks
    /// route through the Phase 4 input router and land in
    /// `input_sink` — the engine has no IPC socket client yet, so
    /// the sink is an in-process log standing in for the fork's
    /// input channel (live seating is 0.1/0.2 territory).
    ui_store: Option<ui_share::UiShmStore>,
    router: Option<ui_share::input::InputRouter>,
    input_sink: Vec<proto::InputT>,
    /// `--connect-window` seating: the session thread's
    /// published live wire scene (polled per frame into
    /// `frame_args.wire`) and the channel key presses are
    /// forwarded to as session Inputs.
    live_share: Option<engine::client::SharedWireScene>,
    /// The session thread's published UI frames, polled per
    /// frame into `frame_args.ui` (the live menu source —
    /// without it the window composites only the `--ui-stub`
    /// fixture and no sim menu can appear).
    live_ui_share: Option<engine::client::SharedUiScene>,
    /// Seq of the last published UI frame seated into
    /// `frame_args.ui` (re-seat only on a newer frame).
    live_ui_seq: u64,
    /// The sim's current UI mode, as last published on the
    /// UI share (MapInput = map focus; UiInput = a menu is
    /// open). Key routing is gated on it: while a menu is
    /// open, every key belongs to the sim — Q/E yaw and
    /// PgUp/PgDn cutaway are MapInput-only. Updated by
    /// `poll_live` straight from the share, not only when a
    /// frame seats, so the first key pressed after a menu
    /// opens already routes by the new mode.
    live_mode: proto::UiMode,
    live_input: Option<std::sync::mpsc::Sender<i32>>,
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
            fine_props: cfg.fine_props,
            textures: cfg.textures_dir,
            effects: cfg.effects,
            billboards: cfg.billboards,
            // Phase 10 (10.6): the windowed app runs under a
            // seated sky — `--light-hour`'s fake day (the same
            // hour the bake below uses), else the §6 neutral
            // environment. Before 10.6 the sky stayed None (raw
            // neutral uniforms) while `--light-hour` moved only
            // the bake, so the window showed a noon sky over a
            // midnight bake.
            sky: Some(match cfg.light_hour {
                Some(h) => engine::frame::SkyScene::fake_day(h, 4),
                None => engine::frame::SkyScene::neutral_env(),
            }),
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
        // Live session seating (`--connect-window`): the frame
        // renders the session's wire world, so the fixture
        // companions come off — a fake-house light grid or sky
        // over a wire world would mask/shade tiles the game
        // never reported. The wire world's OWN companions take
        // their place per publish in `sync_frame_args` (Lane
        // H: the published scene carries the snapshot-derived
        // lighting scene + environment sample). Vehicles and
        // transients stay unseated for live sessions. Until
        // the first Snapshot publishes, `wire` stays None and
        // the frame is the default fixture.
        let (live_share, live_ui_share, live_input) = match cfg.live {
            Some(link) => {
                frame_args.sky = None;
                frame_args.lighting = None;
                frame_args.vehicles = None;
                frame_args.transients = None;
                // Entity billboards (the player included): seat
                // an empty scene over the sprite atlas from the
                // textures dir — the same dir the session loads
                // its model table from; it holds `sprites.json`
                // + `tile_sprites.json`. `sync_frame_args` fills
                // its entities + lift from every published wire
                // scene. Without a seated scene that sync is a
                // no-op and NO wire entity — above all the
                // player — ever renders in the window (the
                // 2026-11-25 Launch-Sim screenshot). An explicit
                // `--billboards` scene wins; a failed atlas load
                // is logged and the session continues without
                // billboards (the client's posture for its own
                // atlas load).
                if frame_args.billboards.is_none() {
                    if let Some(dir) = frame_args.textures.clone() {
                        match billboard::atlas::SpriteAtlas::load(&dir) {
                            Ok(atlas) => {
                                frame_args.billboards =
                                    Some(BillboardScene::empty_with_atlas(atlas));
                            }
                            Err(e) => {
                                eprintln!(
                                    "voxel-engine: live billboard atlas at {} failed to load ({e}); entity billboards off",
                                    dir.display()
                                );
                            }
                        }
                    }
                }
                (Some(link.share), Some(link.ui_share), Some(link.input))
            }
            None => (None, None, None),
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
            rt_cache: None,
            ui_store,
            router,
            input_sink: Vec::new(),
            live_share,
            live_ui_share,
            live_ui_seq: 0,
            live_mode: proto::UiMode::MapInput,
            live_input,
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

    /// Poll the session's shared slots WITHOUT seating or
    /// rendering: returns true when a fresher wire scene or
    /// UI frame is waiting, and refreshes `live_mode` from
    /// the latest published UI frame either way.
    ///
    /// This poll is the live window's only wake-up path, and
    /// its absence was the root cause of the second
    /// Launch-Sim input round (2026-11-25, merged build):
    /// the session thread publishes into these slots but has
    /// no channel to the winit event loop, the loop sleeps
    /// under `ControlFlow::Wait`, and the frame pacer only
    /// renders when something marks it dirty — forwarded
    /// keys never did. So the sim advanced (the player
    /// moved, menus opened, Esc closed them sim-side) while
    /// the screen sat on the pre-key frame until some
    /// unrelated OS event forced a redraw: movement looked
    /// dead, menus "barely worked" with large, random
    /// latency, and Esc-in-menu looked broken. The
    /// app-wiring tests could not see it — they call
    /// `sync_frame_args` directly, bypassing the pacer and
    /// the event loop entirely.
    fn poll_live(&mut self) -> bool {
        let mut changed = false;
        if let Some(share) = &self.live_ui_share {
            let published = {
                let guard = share.lock().expect("ui share lock");
                guard.as_ref().map(|p| (p.seq, p.mode))
            };
            if let Some((seq, mode)) = published {
                if mode != self.live_mode {
                    if key_debug_enabled() {
                        eprintln!(
                            "key-debug: sim mode {:?} -> {:?} (ui seq {seq})",
                            self.live_mode, mode
                        );
                    }
                    self.live_mode = mode;
                }
                if seq != self.live_ui_seq {
                    changed = true;
                }
            }
        }
        if let Some(share) = &self.live_share {
            let guard = share.lock().expect("wire share lock");
            let fresh = match (&*guard, &self.frame_args.wire) {
                (Some(w), Some(old)) => !Arc::ptr_eq(&old.map, &w.map),
                (Some(_), None) => true,
                _ => false,
            };
            if fresh {
                changed = true;
            }
        }
        changed
    }

    /// Sync the live camera/cutaway/size/time into the frame
    /// template before a redraw.
    fn sync_frame_args(&mut self) {
        // Live UI: seat the session's latest published UI
        // frame (menus included) whenever a newer one lands.
        // UI frames arrive on their own cadence (per sim
        // action), independent of wire-scene publishes.
        if let Some(share) = &self.live_ui_share {
            let published = share.lock().expect("ui share lock").clone();
            if let Some(prepared) = published {
                if prepared.seq != self.live_ui_seq {
                    self.live_ui_seq = prepared.seq;
                    self.live_mode = prepared.mode;
                    self.frame_args.ui = Some(prepared);
                }
            }
        }
        // Live session: seat the session's latest published
        // wire world (replaced by the session thread after
        // every applied flush; an unchanged scene is the same
        // Arc, so this is a pointer swap per frame).
        if let Some(share) = &self.live_share {
            if let Some(w) = share.lock().expect("wire share lock").clone() {
                // Lane H: on a fresh publish, seat the wire
                // world's GI companions with it — the lighting
                // scene wholesale (it is data: the game's
                // current lit/outside grids), and the
                // environment sample into the running sky
                // scene (seat fresh if none; `push` only on a
                // newer sample — the §2.3 cadence joint — so
                // the clock cross-fades instead of restarting
                // on every flush). Vehicles/transients stay
                // None for live sessions.
                let fresh = match &self.frame_args.wire {
                    Some(old) => !Arc::ptr_eq(&old.map, &w.map),
                    None => true,
                };
                if fresh {
                    self.frame_args.wire = Some(w.clone());
                    // Follow the player + track their level:
                    // the camera recentres on the player
                    // entity and the cutaway's player z follows
                    // them, so movement is visible the moment
                    // it happens (the pre-fix camera sat on the
                    // house centre forever and the cutaway
                    // stayed at its default level, which —
                    // with the player invisible — made even
                    // working movement look dead).
                    if let Some(player) = w.entities.iter().find(|e| {
                        e.sprite_id == "player_male" || e.sprite_id == "player_female"
                    }) {
                        let p = player.pos;
                        self.camera.follow(Vec3::new(
                            p[0] as f32 + 0.5,
                            1.0,
                            -(p[1] as f32 + 0.5),
                        ));
                        self.cutaway.player_z = p[2];
                    }
                    // Entity billboards: the seated billboard
                    // scene's ATLAS comes from `--billboards`
                    // (fixture / sample / a tileset dir), but its
                    // ENTITIES are the wire world's — the player
                    // character first, then creatures — never the
                    // flag's hard-coded crowd. The player thus
                    // renders as a sprite on a billboard through
                    // the same pass as every enemy.
                    if let Some(scene) = &mut self.frame_args.billboards {
                        scene.entities = w.entities.clone();
                        scene.base_lift = w.billboard_base_lift();
                    }
                    if let Some(lit) = &w.lighting {
                        self.frame_args.lighting = Some(lit.clone());
                    }
                    if let Some(env) = &w.environment {
                        match &mut self.frame_args.sky {
                            Some(sky) => {
                                if sky.game_env.turn != env.turn
                                    || sky.game_env.time_of_day_sec != env.time_of_day_sec
                                {
                                    sky.push(env.clone());
                                }
                            }
                            None => {
                                self.frame_args.sky =
                                    Some(engine::frame::SkyScene::new(env.clone()));
                            }
                        }
                    }
                }
            }
        }
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
        // Phase 10 (10.6, §4): reseat the sky's cave factor
        // from the camera target tile — x/y from the camera,
        // z from the cutaway's player level, exposure from the
        // lighting scene's wire-derived set.
        if let (Some(sky), Some(lit)) = (&mut a.sky, &a.lighting) {
            if let Some(exposure) = &lit.sky_exposure {
                let target = [
                    a.camera_target.0.floor() as i32,
                    a.camera_target.1.floor() as i32,
                    self.cutaway.player_z,
                ];
                sky.cave_factor =
                    engine::frame::SkyScene::cave_factor_for(exposure, target);
            }
        }
        let t = self.started.elapsed().as_secs_f32();
        a.fx_time = t;
        if let Some(scene) = &mut a.billboards {
            scene.time_s = t;
        }
    }

    /// Render one world frame through the app's RT scene cache
    /// — the exact call `render` presents. Extracted so the
    /// rt_cache_gate's app-wiring test can drive the windowed
    /// path headlessly: if this ever falls back to the uncached
    /// `render_frame`, the live demo is back to rebuilding the
    /// scene every frame (the 2026-11-24 0 fps bug) while the
    /// frame-level gates stay green.
    fn render_world_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<engine::frame::FrameOutcome, engine::capture::CaptureError> {
        let outcome = engine::frame::render_frame_with_rt_cache(
            device,
            queue,
            &self.frame_args,
            &mut self.rt_cache,
        )?;
        if let Some(stats) = &outcome.stats.mesh_stats {
            self.overlay.stats = *stats;
            self.overlay.rt_voxels = None;
        } else {
            // RT reports no mesh stats: without this the
            // title keeps the zero defaults and claims an
            // empty world. Report the traced voxel count and
            // the live map's chunk count instead.
            self.overlay.rt_voxels = Some(outcome.quads);
            if let Some(w) = &self.frame_args.wire {
                self.overlay.stats.chunks = w.map.chunks.len();
            }
        }
        Ok(outcome)
    }

    fn render(&mut self) {
        let Some(size) = self.gpu.as_ref().map(|g| g.size) else {
            return;
        };
        if !engine::overlay::renderable(size) {
            return; // minimized: skip, never present to a zero-size surface
        }
        self.sync_frame_args();
        // Render the world BEFORE acquiring the surface frame:
        // `render_world_frame` borrows `self` mutably for the
        // RT cache, so it cannot run under a `&self.gpu` borrow.
        // The world texture is independent of the surface, so
        // the order is free.
        let (device, queue) = {
            let Some(gpu) = &self.gpu else { return };
            (gpu.device.clone(), gpu.queue.clone())
        };
        let world = self.render_world_frame(&device, &queue);
        let Some(gpu) = &self.gpu else { return };
        let frame = match gpu.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        match world {
            Ok(outcome) => {
                gpu.blit
                    .draw(&gpu.device, &gpu.queue, &outcome.texture, &view);
            }
            Err(e) => eprintln!("voxel-engine: frame failed: {e}"),
        }
        frame.present();
        self.overlay.fps = self.fps_counter.tick(Instant::now());
        let scene_label = if self.live_share.is_some() {
            "live session"
        } else {
            "fake house"
        };
        gpu.window.set_title(&format!(
            "CDDA Voxel — {scene_label} | {} | cutaway z<={}",
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
        let window = ui_share::Size2 {
            w: gpu.size.0,
            h: gpu.size.1,
        };
        let Some(router) = self.router.clone() else {
            self.drag_from = Some(self.cursor);
            return;
        };
        // The stub UI canvas is 640x360, stretched over the window
        // (the same mode the compositor draws it with).
        let ui_size = ui_share::Size2 { w: 640, h: 360 };
        let viewport = ui_share::scaling::compute_viewport(
            window,
            ui_size,
            ui_share::ScalingMode::StretchNearest,
        );
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Live session: poll the session's shared slots and
        // dirty the pacer when the sim published anything —
        // and keep a short wake deadline while a session is
        // attached, because `ControlFlow::Wait` otherwise
        // sleeps until the next OS event and a publish that
        // lands while the loop sleeps would wait for one
        // (see `poll_live`: the presentation-latency root
        // cause). With the poll, a sim publish is presented
        // within one poll interval + one frame.
        if self.poll_live() {
            self.pacer.mark_world_dirty();
        }
        if self.pacer.should_render() {
            if let Some(gpu) = &self.gpu {
                gpu.window.request_redraw();
            }
        }
        if self.live_share.is_some() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + LIVE_POLL_INTERVAL,
            ));
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
                // Live session key routing: ONE decision
                // function (`live_key_action`) decides, gated
                // on the sim's current UI mode — in MapInput
                // the game's keys go to the sim and only Q/E
                // (camera yaw) and PgUp/PgDn (cutaway) stay
                // local; in UiInput (a menu is open) EVERY key
                // is the sim's, Q/E and PgUp/PgDn included, so
                // `q` backs out of the menu instead of yawing
                // the camera behind it. The character comes
                // from the event's LOGICAL key (`key_char`),
                // not `event.text` alone: winit only
                // guarantees text for character-producing keys
                // and it can be absent where the logical key
                // still names the character. The session ends
                // by closing the window (the app drop closes
                // the channel — the client's input-close rule).
                if let Some(tx) = &self.live_input {
                    let tx = tx.clone();
                    // Fresh mode first: a menu may have opened
                    // since the last rendered frame (publishes
                    // do not wait for renders — `poll_live`).
                    self.poll_live();
                    let physical = match event.physical_key {
                        PhysicalKey::Code(c) => Some(c),
                        _ => None,
                    };
                    let ch = key_char(&event.logical_key, event.text.as_deref());
                    let action = physical.map_or(LiveKeyAction::Ignore, |c| {
                        live_key_action(self.live_mode, c, ch)
                    });
                    if key_debug_enabled() {
                        eprintln!(
                            "key-debug: physical={physical:?} logical={:?} text={:?} \
                             mode={:?} -> {action:?}",
                            event.logical_key, event.text, self.live_mode
                        );
                    }
                    match action {
                        LiveKeyAction::Forward(code) => {
                            if tx.send(code).is_err() && key_debug_enabled() {
                                eprintln!("key-debug: forward of {code} failed: channel closed");
                            }
                        }
                        LiveKeyAction::CameraYaw(d) => {
                            self.camera.orbit(0.0, 0.0, d);
                            self.pacer.mark_camera_dirty();
                        }
                        LiveKeyAction::CutawayUp => {
                            self.cutaway.step_up();
                            self.remesh();
                        }
                        LiveKeyAction::CutawayDown => {
                            self.cutaway.step_down();
                            self.remesh();
                        }
                        LiveKeyAction::Ignore => {}
                    }
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

/// How often the live window wakes to poll the session's
/// shared slots while a session is attached (see
/// `App::poll_live`): the bound on publish -> present
/// latency for sim-driven changes (movement results, menu
/// frames) is one interval + one rendered frame.
const LIVE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);

/// Key-input debug logging (Joey's debug-logging rule):
/// `CDDA_VOXEL_KEY_DEBUG=1` logs every live-window key
/// event, the sim mode it was routed under, the routing
/// decision, and mode changes as publishes land. Off by
/// default; no behaviour change when off.
fn key_debug_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| {
        std::env::var_os("CDDA_VOXEL_KEY_DEBUG").is_some_and(|v| v != "0")
    })
}

/// The character a key event names, from its LOGICAL key
/// first (winit's layout-aware character), falling back to
/// the event text. The pre-fix routing read `event.text`
/// only: winit guarantees text just for
/// character-producing keys, and where text is absent the
/// old code silently forwarded nothing for a key whose
/// logical key still named it.
fn key_char(logical: &Key, text: Option<&str>) -> Option<char> {
    if let Key::Character(s) = logical {
        if let Some(c) = s.chars().next() {
            return Some(c);
        }
    }
    text.and_then(|t| t.chars().next())
}

/// What one live-window key press should do — the REAL
/// window handler's decision, extracted as a pure function
/// so the gates exercise the exact code the window runs
/// (the first fix round tested a parallel routing helper
/// while the handler kept its own match; they diverged in
/// the ways Joey's PC run reported).
#[derive(Debug, Clone, Copy, PartialEq)]
enum LiveKeyAction {
    /// Send this `ui_share::input` key code to the sim.
    Forward(i32),
    /// Yaw the camera by this many degrees (Q/E, MapInput
    /// only).
    CameraYaw(f32),
    /// Step the cutaway up/down (PgUp/PgDn, MapInput only).
    CutawayUp,
    CutawayDown,
    /// Neither the sim's nor the window's: drop it.
    Ignore,
}

/// The live window's key routing, gated on the sim's UI
/// mode (Launch-Sim input rounds 1+2):
/// - **MapInput** (no menu): the game's keys forward —
///   Escape (pending-action cancel), the arrows, Enter /
///   Tab / Backspace, and every printable character, WASD
///   included (`w`/`a` are movement aliases in the sim and
///   `s`/`d` are its smash/drop keys). Only Q/E (camera
///   yaw) and PgUp/PgDn (cutaway) stay local.
/// - **UiInput** (a menu is open): EVERY key is the sim's.
///   `q` backs out of the menu (it must not also yaw the
///   camera), `e` is the menu's own key, PgUp/PgDn are the
///   sim's list-page keys, and Escape closes the menu.
///   Camera and cutaway keys are suspended until the sim
///   returns to MapInput.
fn live_key_action(mode: proto::UiMode, physical: KeyCode, ch: Option<char>) -> LiveKeyAction {
    use ui_share::input::{
        KEY_BACKSPACE, KEY_DOWN, KEY_ENTER, KEY_ESCAPE, KEY_LEFT, KEY_PAGE_DOWN, KEY_PAGE_UP,
        KEY_RIGHT, KEY_TAB, KEY_UP,
    };
    let in_menu = mode == proto::UiMode::UiInput;
    match physical {
        KeyCode::Escape => return LiveKeyAction::Forward(KEY_ESCAPE),
        KeyCode::ArrowLeft => return LiveKeyAction::Forward(KEY_LEFT),
        KeyCode::ArrowRight => return LiveKeyAction::Forward(KEY_RIGHT),
        KeyCode::ArrowUp => return LiveKeyAction::Forward(KEY_UP),
        KeyCode::ArrowDown => return LiveKeyAction::Forward(KEY_DOWN),
        KeyCode::Enter | KeyCode::NumpadEnter => return LiveKeyAction::Forward(KEY_ENTER),
        KeyCode::Tab => return LiveKeyAction::Forward(KEY_TAB),
        KeyCode::Backspace => return LiveKeyAction::Forward(KEY_BACKSPACE),
        KeyCode::PageUp => {
            return if in_menu {
                LiveKeyAction::Forward(KEY_PAGE_UP)
            } else {
                LiveKeyAction::CutawayUp
            };
        }
        KeyCode::PageDown => {
            return if in_menu {
                LiveKeyAction::Forward(KEY_PAGE_DOWN)
            } else {
                LiveKeyAction::CutawayDown
            };
        }
        KeyCode::KeyQ | KeyCode::KeyE => {
            if in_menu {
                let c = ch.unwrap_or(if physical == KeyCode::KeyQ { 'q' } else { 'e' });
                return LiveKeyAction::Forward(c as i32);
            }
            return LiveKeyAction::CameraYaw(if physical == KeyCode::KeyQ {
                -2.0
            } else {
                2.0
            });
        }
        _ => {}
    }
    match ch {
        Some(c) if (' '..='~').contains(&c) => {
            if !in_menu && matches!(c, 'q' | 'Q' | 'e' | 'E') {
                LiveKeyAction::Ignore
            } else {
                LiveKeyAction::Forward(c as i32)
            }
        }
        _ => LiveKeyAction::Ignore,
    }
}

/// The first round's routing helper, kept as a thin
/// MapInput-mode shim over [`live_key_action`] (its original
/// callers/tests predate mode-aware routing).
#[cfg(test)]
fn live_forward_code(physical: KeyCode, text: Option<&str>) -> Option<i32> {
    match live_key_action(
        proto::UiMode::MapInput,
        physical,
        text.and_then(|t| t.chars().next()),
    ) {
        LiveKeyAction::Forward(code) => Some(code),
        _ => None,
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

#[cfg(test)]
mod app_wiring_tests {
    //! App-wiring half of the rt_cache_gate (test #3): the
    //! frame-level gates exercise `frame::render_frame*`, but
    //! the live demo runs through THIS binary's render path.
    //! Drive `App::render_world_frame` — the call `render`
    //! presents — twice and require the app's own cache to
    //! show exactly one build. A fallback to the uncached
    //! entry here would restore the 0 fps bug with every
    //! frame-level gate still green.
    use super::*;

    #[test]
    fn windowed_render_path_builds_the_rt_scene_once() {
        let gpu = pollster::block_on(async {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await?;
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("app-wiring-test"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .await
                .ok()
        });
        let Some((device, queue)) = gpu else {
            eprintln!("SKIP app wiring test: no wgpu adapter available");
            return;
        };
        let mut app = App::new(WindowConfig {
            renderer_kind: RendererKind::Rt,
            textures_dir: None,
            effects: None,
            billboards: None,
            ui_stub: false,
            light_hour: None,
            vehicle_fixture: false,
            transient_fixture_ms: None,
            render_scale: 1,
            fine_props: false,
            live: None,
        });
        app.frame_args.size = (320, 180);
        app.render_world_frame(&device, &queue).expect("frame 1");
        app.frame_args.yaw_deg += 20.0;
        app.render_world_frame(&device, &queue).expect("frame 2");
        assert_eq!(
            app.rt_cache.as_ref().map(|c| c.build_count()),
            Some(1),
            "the windowed render path must trace frame 2 from the held scene"
        );
    }

    /// Regression (2026-11-25 Launch-Sim screenshot, symptom
    /// 1): a `--connect-window` App launched with a textures
    /// dir must seat a billboard scene, and a published wire
    /// scene must land its entities — the player first — in
    /// it. Before the fix the App seated NO billboard scene
    /// for live sessions (the session config hardcoded
    /// `billboards_dir: None` and nothing else seated one), so
    /// `sync_frame_args`'s entity copy had no scene to copy
    /// into and the player never rendered.
    #[test]
    fn live_app_seats_the_wire_player_billboard() {
        let ultica = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/ultica_run1");
        if !ultica.join("sprites.json").exists() {
            eprintln!("SKIP live billboard seating test: real UltiCa atlas absent");
            return;
        }
        let (tx, _rx) = std::sync::mpsc::channel::<i32>();
        let share: engine::client::SharedWireScene =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let mut app = App::new(WindowConfig {
            renderer_kind: RendererKind::Rt,
            textures_dir: Some(ultica),
            effects: None,
            billboards: None,
            ui_stub: false,
            light_hour: None,
            vehicle_fixture: false,
            transient_fixture_ms: None,
            render_scale: 1,
            fine_props: false,
            live: Some(LiveWindowLink {
                share: share.clone(),
                ui_share: std::sync::Arc::new(std::sync::Mutex::new(None)),
                input: tx,
            }),
        });
        // A published wire scene whose only entity is the
        // player at (5, 5, 0).
        let mut snap = proto::SnapshotT::default();
        let mut player = proto::EntityT::default();
        player.id = 1;
        player.sprite_id = Some("player_male".to_string());
        player.pos = Some(proto::Vec3iT { x: 5, y: 5, z: 0 });
        snap.entities = Some(vec![player]);
        let wire = engine::wire::WireScene::from_map(
            mesher::chunk::ChunkMap::new(mesher::scale::WorldScale::default()),
            &snap,
        );
        let want_lift = wire.billboard_base_lift();
        *share.lock().expect("share lock") = Some(wire);
        app.sync_frame_args();
        let scene = app
            .frame_args
            .billboards
            .as_ref()
            .expect("a live App with a textures dir seats a billboard scene");
        assert!(
            scene.entities.iter().any(|e| e.sprite_id == "player_male"),
            "the published wire player lands in the seated billboard scene"
        );
        assert_eq!(scene.base_lift, want_lift, "the wire lift is applied");
    }

    /// Regression (Launch-Sim symptoms 4/5, window half):
    /// the live key routing forwards the game's keys —
    /// including the WASD the old filter swallowed and the
    /// arrow/Escape codes the old byte channel could not
    /// carry — and keeps only Q/E + PgUp/PgDn local.
    /// Round 2: these assertions run against
    /// `live_key_action`, the decision function the REAL
    /// window handler calls (round 1 tested a parallel
    /// helper while the handler kept its own match — the
    /// divergence Joey's PC run exposed).
    #[test]
    fn live_key_routing_forwards_the_game_keys() {
        use ui_share::input::{KEY_DOWN, KEY_ESCAPE, KEY_LEFT};
        let map = proto::UiMode::MapInput;
        let fwd = |c: KeyCode, t: Option<char>| live_key_action(map, c, t);
        // Movement + verbs that the pre-fix filter ate.
        assert_eq!(fwd(KeyCode::KeyW, Some('w')), LiveKeyAction::Forward('w' as i32));
        assert_eq!(fwd(KeyCode::KeyA, Some('a')), LiveKeyAction::Forward('a' as i32));
        assert_eq!(fwd(KeyCode::KeyS, Some('s')), LiveKeyAction::Forward('s' as i32));
        assert_eq!(fwd(KeyCode::KeyD, Some('d')), LiveKeyAction::Forward('d' as i32));
        assert_eq!(fwd(KeyCode::KeyB, Some('b')), LiveKeyAction::Forward('b' as i32));
        // Menus.
        assert_eq!(fwd(KeyCode::KeyI, Some('i')), LiveKeyAction::Forward('i' as i32));
        assert_eq!(fwd(KeyCode::Escape, None), LiveKeyAction::Forward(KEY_ESCAPE));
        // Arrows: no character, the code carries them.
        assert_eq!(fwd(KeyCode::ArrowDown, None), LiveKeyAction::Forward(KEY_DOWN));
        assert_eq!(fwd(KeyCode::ArrowLeft, None), LiveKeyAction::Forward(KEY_LEFT));
        // ';' (look) forwards — round 1 keyed it on
        // `event.text`, which winit does not guarantee.
        assert_eq!(fwd(KeyCode::Semicolon, Some(';')), LiveKeyAction::Forward(';' as i32));
        // Local-only keys in MapInput: Q/E yaw, PgUp/PgDn cutaway.
        assert_eq!(fwd(KeyCode::KeyQ, Some('q')), LiveKeyAction::CameraYaw(-2.0));
        assert_eq!(fwd(KeyCode::KeyE, Some('e')), LiveKeyAction::CameraYaw(2.0));
        assert_eq!(fwd(KeyCode::PageUp, None), LiveKeyAction::CutawayUp);
        assert_eq!(fwd(KeyCode::PageDown, None), LiveKeyAction::CutawayDown);
        // The round-1 shim still agrees in MapInput mode.
        assert_eq!(live_forward_code(KeyCode::KeyW, Some("w")), Some('w' as i32));
        assert_eq!(live_forward_code(KeyCode::KeyQ, Some("q")), None);
        assert_eq!(live_forward_code(KeyCode::PageUp, None), None);
    }

    /// Regression (input round 2, defect 2): while a menu is
    /// open (UiInput), routing must give EVERY key to the
    /// sim — above all `q`, which round 1 yawed the camera
    /// with even as it (didn't) back out of the menu, and
    /// PgUp/PgDn, which are the sim's list-page keys there.
    #[test]
    fn live_key_routing_in_a_menu_gives_every_key_to_the_sim() {
        use ui_share::input::{KEY_ESCAPE, KEY_PAGE_DOWN, KEY_PAGE_UP};
        let ui = proto::UiMode::UiInput;
        assert_eq!(
            live_key_action(ui, KeyCode::KeyQ, Some('q')),
            LiveKeyAction::Forward('q' as i32),
            "q backs out of the menu; it must not yaw the camera"
        );
        assert_eq!(
            live_key_action(ui, KeyCode::KeyE, Some('e')),
            LiveKeyAction::Forward('e' as i32)
        );
        assert_eq!(
            live_key_action(ui, KeyCode::PageUp, None),
            LiveKeyAction::Forward(KEY_PAGE_UP),
            "PgUp pages the sim's list in a menu, not the cutaway"
        );
        assert_eq!(
            live_key_action(ui, KeyCode::PageDown, None),
            LiveKeyAction::Forward(KEY_PAGE_DOWN)
        );
        assert_eq!(
            live_key_action(ui, KeyCode::Escape, None),
            LiveKeyAction::Forward(KEY_ESCAPE),
            "Esc closes the menu (defect 3)"
        );
        assert_eq!(
            live_key_action(ui, KeyCode::KeyJ, Some('j')),
            LiveKeyAction::Forward('j' as i32),
            "menu cursor keys still forward"
        );
    }

    /// Regression (input round 2, defect 1 half): the
    /// character for a printable key comes from the LOGICAL
    /// key, with text only as a fallback — round 1 read
    /// `event.text` alone, which winit does not guarantee
    /// for every key/platform.
    #[test]
    fn key_char_prefers_the_logical_key_over_text() {
        assert_eq!(
            key_char(&Key::Character("j".into()), None),
            Some('j'),
            "a logical character with NO text still names the key"
        );
        assert_eq!(
            key_char(&Key::Character(";".into()), Some(";")),
            Some(';')
        );
        assert_eq!(
            key_char(
                &Key::Named(winit::keyboard::NamedKey::ArrowDown),
                None
            ),
            None,
            "named keys have no character (the code path carries them)"
        );
        assert_eq!(
            key_char(&Key::Named(winit::keyboard::NamedKey::Enter), Some("\r")),
            Some('\r'),
            "text is the fallback when the logical key names none"
        );
    }

    /// Regression (input round 2, defects 1+4): a session
    /// publish must be VISIBLE to the App's wake path
    /// without any render happening first. Round 1's App
    /// only ever learned about publishes inside
    /// `sync_frame_args` (called from `render`), and renders
    /// only happened when the pacer was dirty — forwarded
    /// keys never dirtied it — so publishes sat unpresented
    /// until an unrelated OS event forced a redraw. This
    /// test drives `poll_live` (the `about_to_wait` path)
    /// exactly as the event loop does: publish -> poll ->
    /// changed, and the UI mode tracks the publish so the
    /// NEXT key routes by it.
    #[test]
    fn live_poll_surfaces_publishes_and_tracks_the_sim_mode() {
        let (tx, _rx) = std::sync::mpsc::channel::<i32>();
        let share: engine::client::SharedWireScene =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let ui_share: engine::client::SharedUiScene =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let mut app = App::new(WindowConfig {
            renderer_kind: RendererKind::Rt,
            textures_dir: None,
            effects: None,
            billboards: None,
            ui_stub: false,
            light_hour: None,
            vehicle_fixture: false,
            transient_fixture_ms: None,
            render_scale: 1,
            fine_props: false,
            live: Some(LiveWindowLink {
                share: share.clone(),
                ui_share: ui_share.clone(),
                input: tx,
            }),
        });
        assert!(!app.poll_live(), "nothing published: no wake");
        assert_eq!(app.live_mode, proto::UiMode::MapInput);

        // A UI frame ALONE (no world publish at all) must
        // surface: this is the menu-latency property — the
        // UI lane is seatable independent of the world path.
        *ui_share.lock().expect("ui share lock") = Some(engine::frame::UiPrepared {
            tight: vec![7, 7, 7, 255],
            size: (1, 1),
            mode: proto::UiMode::UiInput,
            terrain_rect: None,
            seq: 7,
        });
        assert!(app.poll_live(), "a lone UI publish wakes the window");
        assert_eq!(
            app.live_mode,
            proto::UiMode::UiInput,
            "mode tracks the publish before any render seats the frame"
        );
        // And it seats through the ordinary render sync.
        app.sync_frame_args();
        assert_eq!(app.frame_args.ui.as_ref().map(|u| u.seq), Some(7));
        assert!(!app.poll_live(), "seated: no further wake");

        // A fresh wire publish alone also wakes the window
        // (movement results land as wire publishes).
        let snap = proto::SnapshotT::default();
        let wire = engine::wire::WireScene::from_map(
            mesher::chunk::ChunkMap::new(mesher::scale::WorldScale::default()),
            &snap,
        );
        *share.lock().expect("share lock") = Some(wire);
        assert!(app.poll_live(), "a wire publish wakes the window");
        app.sync_frame_args();
        assert!(!app.poll_live(), "seated wire scene: no further wake");
    }

    /// Regression (Launch-Sim symptom 5, visibility half +
    /// the title stats): a fresh live publish must recentre
    /// the camera on the player and sync the cutaway to
    /// their level — the pre-fix camera sat on the house
    /// centre forever, so even a moving player left the
    /// frame (or was never in it).
    #[test]
    fn live_app_follows_the_player_and_syncs_cutaway() {
        let (tx, _rx) = std::sync::mpsc::channel::<i32>();
        let share: engine::client::SharedWireScene =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let mut app = App::new(WindowConfig {
            renderer_kind: RendererKind::Rt,
            textures_dir: None,
            effects: None,
            billboards: None,
            ui_stub: false,
            light_hour: None,
            vehicle_fixture: false,
            transient_fixture_ms: None,
            render_scale: 1,
            fine_props: false,
            live: Some(LiveWindowLink {
                share: share.clone(),
                ui_share: std::sync::Arc::new(std::sync::Mutex::new(None)),
                input: tx,
            }),
        });
        let publish = |app: &mut App, x: i32, y: i32| {
            let mut snap = proto::SnapshotT::default();
            let mut player = proto::EntityT::default();
            player.id = 1;
            player.sprite_id = Some("player_male".to_string());
            player.pos = Some(proto::Vec3iT { x, y, z: 0 });
            snap.entities = Some(vec![player]);
            let wire = engine::wire::WireScene::from_map(
                mesher::chunk::ChunkMap::new(mesher::scale::WorldScale::default()),
                &snap,
            );
            *share.lock().expect("share lock") = Some(wire);
            app.sync_frame_args();
        };
        publish(&mut app, 5, 5);
        assert_eq!(app.camera.target, Vec3::new(5.5, 1.0, -5.5));
        assert_eq!(app.cutaway.player_z, 0);
        publish(&mut app, 5, 9);
        assert_eq!(
            app.camera.target,
            Vec3::new(5.5, 1.0, -9.5),
            "the camera follows the player out of the house"
        );
    }

    /// Regression (Launch-Sim symptom 4, display half): a UI
    /// frame the session publishes into the shared UI slot
    /// must land in the App's frame args — that slot is the
    /// ONLY route by which a live sim menu (inventory &c.)
    /// can reach the compositor. Before the fix the slot did
    /// not exist: the session validated each UiFrame and
    /// dropped it, and the App composited only its --ui-stub
    /// fixture.
    #[test]
    fn live_app_seats_the_published_ui_frame() {
        let (tx, _rx) = std::sync::mpsc::channel::<i32>();
        let share: engine::client::SharedWireScene =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let ui_share: engine::client::SharedUiScene =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let mut app = App::new(WindowConfig {
            renderer_kind: RendererKind::Rt,
            textures_dir: None,
            effects: None,
            billboards: None,
            ui_stub: false,
            light_hour: None,
            vehicle_fixture: false,
            transient_fixture_ms: None,
            render_scale: 1,
            fine_props: false,
            live: Some(LiveWindowLink {
                share,
                ui_share: ui_share.clone(),
                input: tx,
            }),
        });
        assert!(app.frame_args.ui.is_none(), "no UI before any publish");
        *ui_share.lock().expect("ui share lock") = Some(engine::frame::UiPrepared {
            tight: vec![7, 7, 7, 255],
            size: (1, 1),
            mode: proto::UiMode::UiInput,
            terrain_rect: None,
            seq: 42,
        });
        app.sync_frame_args();
        let ui = app
            .frame_args
            .ui
            .as_ref()
            .expect("the published UI frame is seated for compositing");
        assert_eq!(ui.seq, 42);
        assert_eq!(ui.mode, proto::UiMode::UiInput, "the inventory-mode frame lands");
    }
}
