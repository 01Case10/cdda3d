//! Phase 11 follow-up (Lane H) gate — global illumination in
//! the LIVE wire session: the Phase 10 sky + GI (procedural
//! sky, hemisphere/sun/moon GI under the Phase 7 ceiling rule)
//! seated from the wire data itself — the snapshot's
//! piggybacked `EnvironmentUpdate` and its lit/outside grids —
//! through the production live seating (`client::LiveWorld`,
//! the state object a real session fills over a real socket),
//! for both renderers.
//!
//! Before Lane H the live seating published the world only:
//! the windowed app forced `sky`/`lighting` to `None` for live
//! sessions and `LiveWorld::capture_args` seated neither, so a
//! live world rendered under the neutral default at every hour
//! — noon and midnight frames were byte-identical, and no sky
//! light ever reached a live tile. The proofs:
//!
//! 1. **Seating (CPU).** A real session's live state seats a
//!    sky scene and a lighting scene (with the wire-derived
//!    sky-exposure set) into its capture args — the same
//!    companions the windowed app seats from the published
//!    scene. The exposure set pins the ceiling rule's data:
//!    the house interior is sheltered (factor 0), the yard is
//!    exposed (factor 1).
//!
//! 2. **Day vs night + the ceiling, in pixels.** Production
//!    live captures of street_block at noon and at midnight
//!    (two real sessions, two scene times): noon is brighter
//!    than midnight overall on both renderers; the day/night
//!    swing lands on the sky-exposed part of the frame (the
//!    corners: yard + sky) and is much smaller over the
//!    sheltered house interior (the frame centre), and at noon
//!    the interior is dimmer than the exposed surround — the
//!    ceiling rule in pixel form (no sky light leak indoors).
//!
//! GPU tests follow the golden gate's convention: no wgpu
//! adapter = a loud skip, never a pass.

use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::client::{spawn_connect, ConnectConfig, ConnectInput, SharedWireScene};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

/// Run one real session (real sim-server on a real socket,
/// the engine client on its own thread) against the scene at
/// `scene_file` and return the session's final live state.
/// No inputs: the player stands at the scene start; dropping
/// the input channel ends the session cleanly after the first
/// Snapshot has been published.
fn run_session(
    tag: &str,
    scene_file: PathBuf,
) -> engine::client::LiveWorld {
    let dir = std::env::temp_dir().join(format!("wire-gi-gate-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let socket = dir.join("sim.sock");
    let server_cfg = sim::server::ServerConfig {
        socket_path: socket.clone(),
        defs_path: Some(defs_path()),
        scene_path: scene_file,
        weather: None,
        ui_shm_path: None,
        ui_start: None,
        seed: None,
        record_path: None,
    };
    let server = std::thread::spawn(move || sim::server::run_server(&server_cfg));
    let (tx, rx) = std::sync::mpsc::channel::<u8>();
    let share: SharedWireScene = std::sync::Arc::new(std::sync::Mutex::new(None));
    let client = spawn_connect(
        &socket,
        ConnectConfig {
            input: ConnectInput::Channel(rx),
            share: Some(share.clone()),
            captures: None,
            models_dir: None,
        },
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while share.lock().expect("share lock").is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "session never published a wire scene"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    drop(tx); // input close: drain + Shutdown.
    let report = client
        .join()
        .expect("client thread panicked")
        .expect("connect session failed");
    server
        .join()
        .expect("server thread panicked")
        .expect("sim-server exited cleanly");
    report.live
}

/// The midnight twin of street_block: the same scene file
/// with its `time` line rewritten (the sim's own scene-time
/// knob — the same one sim_visibility_gate turns in-process).
fn midnight_scene() -> PathBuf {
    let src = std::fs::read_to_string(scene_path("street_block")).expect("street_block reads");
    assert!(
        src.contains("time = \"noon\""),
        "street_block's time line moved; update the gate's rewrite"
    );
    let night = src.replacen("time = \"noon\"", "time = \"midnight\"", 1);
    let dir = std::env::temp_dir().join(format!("wire-gi-gate-{}-scenes", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("street_block_midnight.toml");
    std::fs::write(&path, night).expect("midnight scene written");
    path
}

fn loud_skip(what: &str) {
    eprintln!("SKIP {what}: no wgpu adapter available");
    println!("SKIP {what}: no wgpu adapter available");
}

/// Render one production live capture; `None` = no adapter.
fn capture(args: &CaptureArgs) -> Option<image::RgbaImage> {
    match run_capture(args) {
        Ok(_) => Some(image::open(&args.out).expect("capture png").to_rgba8()),
        Err(CaptureError::NoAdapter) => None,
        Err(e) => panic!("live capture failed: {e}"),
    }
}

fn mean_luminance(img: &image::RgbaImage) -> f32 {
    let mut sum = 0f64;
    for px in img.pixels() {
        sum += 0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64;
    }
    (sum / (img.width() * img.height()) as f64) as f32
}

/// Mean luminance over a fractional box (x0, y0, x1, y1) of
/// the frame.
fn region_luminance(img: &image::RgbaImage, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let (w, h) = (img.width() as f32, img.height() as f32);
    let (xa, xb) = ((x0 * w) as u32, (x1 * w) as u32);
    let (ya, yb) = ((y0 * h) as u32, (y1 * h) as u32);
    let mut sum = 0f64;
    let mut n = 0u64;
    for y in ya..yb {
        for x in xa..xb {
            let px = img.get_pixel(x, y);
            sum += 0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64;
            n += 1;
        }
    }
    (sum / n.max(1) as f64) as f32
}

/// The frame centre: the house interior around the player
/// (capture_args targets the player, who starts inside).
fn center_luminance(img: &image::RgbaImage) -> f32 {
    region_luminance(img, 0.42, 0.42, 0.58, 0.58)
}

/// The frame surround: four corner boxes — yard, road and
/// sky, all sky-exposed.
fn surround_luminance(img: &image::RgbaImage) -> f32 {
    let boxes = [
        (0.0, 0.0, 0.16, 0.16),
        (0.84, 0.0, 1.0, 0.16),
        (0.0, 0.84, 0.16, 1.0),
        (0.84, 0.84, 1.0, 1.0),
    ];
    boxes
        .iter()
        .map(|&(a, b, c, d)| region_luminance(img, a, b, c, d))
        .sum::<f32>()
        / boxes.len() as f32
}

// ---------------------------------------------------------------------
// 1: the production live seating carries the GI companions.
// ---------------------------------------------------------------------

#[test]
fn live_session_seats_gi_companions() {
    let live = run_session("seat-noon", scene_path("street_block"));
    let out = std::env::temp_dir().join(format!("wire-gi-gate-{}-seat.png", std::process::id()));
    let args = live
        .capture_args(&out, (640, 360))
        .expect("live state captures");
    assert!(
        args.sky.is_some(),
        "live capture seats no sky scene — the wire environment never reaches the frame"
    );
    let lit = args
        .lighting
        .as_ref()
        .expect("live capture seats no lighting scene — the wire lit grid never reaches the frame");
    let exposure = lit
        .sky_exposure
        .as_ref()
        .expect("live lighting scene carries no sky-exposure set");
    // The ceiling rule's data, from the game's own outside
    // verdicts: the roofed house interior is sheltered, the
    // yard south of the door is exposed.
    assert_eq!(
        exposure.factor([5, 5, 0]),
        0.0,
        "house interior (5,5,0) must be sheltered from the sky"
    );
    assert_eq!(
        exposure.factor([5, 9, 0]),
        1.0,
        "yard tile (5,9,0) must be sky-exposed"
    );
    // The seated sky is the wire environment's, at noon.
    let sky = args.sky.as_ref().expect("sky seated");
    let state = sky.sky_state();
    assert!(
        state.sun_intensity > 0.5,
        "noon wire environment: sun up (intensity {})",
        state.sun_intensity
    );
    println!(
        "live GI seating: sky + lighting seated; exposure interior 0.0 / yard 1.0; noon sun {}",
        state.sun_intensity
    );
}

// ---------------------------------------------------------------------
// 2: day vs night in pixels, and the ceiling over the interior.
// ---------------------------------------------------------------------

#[test]
fn live_gi_day_night_and_ceiling() {
    let noon_live = run_session("px-noon", scene_path("street_block"));
    let night_live = run_session("px-midnight", midnight_scene());
    let dir = std::env::temp_dir().join(format!("wire-gi-gate-{}-px", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");

    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let render = |live: &engine::client::LiveWorld, tag: &str| {
            let mut args = live
                .capture_args(&dir.join(format!("{tag}.png")), (640, 360))
                .expect("live state captures");
            args.renderer = renderer;
            capture(&args)
        };
        let Some(noon) = render(&noon_live, &format!("noon-{}", renderer.name())) else {
            loud_skip("live_gi_day_night_and_ceiling");
            return;
        };
        let night = render(&night_live, &format!("midnight-{}", renderer.name()))
            .expect("adapter vanished between captures");

        let (noon_lum, night_lum) = (mean_luminance(&noon), mean_luminance(&night));
        let (noon_c, noon_s) = (center_luminance(&noon), surround_luminance(&noon));
        let (night_c, night_s) = (center_luminance(&night), surround_luminance(&night));
        println!(
            "{}: noon mean {noon_lum:.1} vs midnight {night_lum:.1}; \
             noon centre {noon_c:.1} / surround {noon_s:.1}; \
             midnight centre {night_c:.1} / surround {night_s:.1}",
            renderer.name()
        );
        assert!(
            noon_lum > night_lum + 5.0,
            "{}: noon live frame ({noon_lum:.1}) must clearly outbrighten midnight ({night_lum:.1}) — \
             the wire environment is not driving the frame",
            renderer.name()
        );
        // The ceiling: at noon the sheltered interior is dimmer
        // than the sky-exposed surround, and the day/night
        // swing lands outside, not inside.
        assert!(
            noon_c < noon_s,
            "{}: noon interior ({noon_c:.1}) must stay dimmer than the exposed surround ({noon_s:.1}) — \
             sky light is leaking indoors",
            renderer.name()
        );
        assert!(
            (noon_s - night_s) > (noon_c - night_c) + 5.0,
            "{}: the day/night swing must land on the exposed surround (swing {:.1}) \
             far more than on the sheltered interior (swing {:.1})",
            renderer.name(),
            noon_s - night_s,
            noon_c - night_c
        );
    }
}
