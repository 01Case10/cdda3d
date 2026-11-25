//! Wire UI gate (Launch-Sim symptom 4, display half): the
//! live session must PUBLISH the sim's prepared UI frames to
//! the shared slot the windowed app composites — before the
//! fix, the session validated each `UiFrame`, logged it, and
//! discarded it, so no menu could ever appear in the live
//! window even though the sim opened it.
//!
//! Proof, against a real sim-server (street_block) over a
//! real socket: the handshake publishes a MapInput UI frame;
//! sending `i` (inventory) publishes a UiInput frame with a
//! newer seq; KEY_ESCAPE closes it back to MapInput; and a
//! KEY_DOWN code — which the pre-fix byte channel could not
//! even represent — moves the player. The gate fails on
//! pre-fix code by construction (the `ui_share` slot and the
//! key-code channel did not exist), and the App-side seating
//! of the slot is covered by
//! `app_wiring_tests::live_app_seats_the_published_ui_frame`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use engine::client::{spawn_connect, ConnectConfig, ConnectInput, SharedUiScene};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}
fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../sim/scenes/{name}.toml"))
}

#[test]
fn session_publishes_ui_frames_and_the_inventory_opens() {
    let dir = std::env::temp_dir().join(format!("wire-ui-gate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let socket = dir.join("sim.sock");

    let server_cfg = sim::server::ServerConfig {
        socket_path: socket.clone(),
        defs_path: Some(defs_path()),
        scene_path: scene_path("street_block"),
        weather: None,
        ui_shm_path: None,
        ui_start: None,
        seed: None,
        record_path: None,
    };
    let server = std::thread::spawn(move || sim::server::run_server(&server_cfg));

    let (tx, rx) = std::sync::mpsc::channel::<i32>();
    let ui_share: SharedUiScene = Arc::new(Mutex::new(None));
    let client = spawn_connect(
        &socket,
        ConnectConfig {
            input: ConnectInput::Channel(rx),
            share: None,
            ui_share: Some(ui_share.clone()),
            captures: None,
            models_dir: None,
            billboards_dir: None,
        },
    );

    let wait_mode = |want: proto::UiMode, what: &str| -> u64 {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(ui) = ui_share.lock().expect("ui share lock").as_ref() {
                if ui.mode == want {
                    return ui.seq;
                }
            }
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(25));
        }
    };

    // The handshake's UiFrame publishes in MapInput mode.
    let seq0 = wait_mode(proto::UiMode::MapInput, "the initial MapInput UI frame");
    // Open the inventory: the sim switches to UiInput and the
    // session must publish THAT frame too.
    tx.send(b'i' as i32).expect("client input channel open");
    let seq1 = wait_mode(proto::UiMode::UiInput, "the inventory (UiInput) UI frame");
    assert!(seq1 > seq0, "inventory frame is newer ({seq1} > {seq0})");

    // Escape closes the menu (a special key code through the
    // channel), and the Down arrow then moves the player —
    // the two key classes the pre-fix window never delivered.
    tx.send(ui_share::input::KEY_ESCAPE)
        .expect("client input channel open");
    let seq2 = wait_mode(proto::UiMode::MapInput, "the menu closed back to MapInput");
    assert!(seq2 > seq1, "close frame is newer ({seq2} > {seq1})");
    tx.send(ui_share::input::KEY_DOWN)
        .expect("client input channel open");

    drop(tx); // input close: drain + Shutdown.
    let report = client
        .join()
        .expect("client thread panicked")
        .expect("connect session failed");
    server
        .join()
        .expect("server thread panicked")
        .expect("sim-server exited cleanly");
    assert_eq!(report.inputs_sent, 3, "i + Esc + Down were forwarded");
    assert_eq!(
        report.player_pos,
        Some([5, 6, 0]),
        "the Down arrow moved the player south"
    );
}
