//! S8 gate: record / replay — plan §3.1's determinism claim
//! ("same seed + same inputs ⇒ byte-identical message log")
//! made executable.
//!
//! A scripted live session is driven against a real
//! `sim-server --record <file>` over its Unix socket (the
//! process-mode client shape: after each Input, a Ping acts
//! as a flush barrier — everything before its Pong is that
//! input's flush). The server writes the session as a §5
//! scenario file. The gate then parses that file with the
//! S7 runner's own parser, extracts the recorded key steps,
//! drives a SECOND fresh server (same socket path, same
//! seed) with exactly those keys, and compares the two
//! sessions' full message streams frame by frame,
//! byte-for-byte — Hello, TileDefs, Snapshot, UiFrame,
//! every Delta / companion, every Pong. No message class is
//! excluded: the sim has no wall-clock inputs (the only
//! timer is the client's own Ping cadence, which this gate
//! fixes deterministically), so the streams must be
//! identical, not merely equivalent.
//!
//! Failing-first: pre-S8 the server had no record mode at
//! all (no `--record`, no `record` module), so this gate
//! fails against pre-S8 code by construction.

use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(windows)]
use uds_windows::UnixStream;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use proto::samples::sample_hello_ack;
use proto::{
    decode_message, encode_message, FrameDecoder, FrameEncoder, InputKind, InputT, MessageT,
    PingT, ShutdownT,
};
use sim::scenario::{Action, Scenario, StepKind};

const TIMEOUT: Duration = Duration::from_secs(30);
const ESC: i32 = ui_share::input::KEY_ESCAPE;

/// The scripted session's keys (wire key codes): out of the
/// house, a UI open/close, a wait, a pickup attempt, and the
/// S8 z-move keys (no-ops on this scene — still recorded).
const KEYS: &[i32] = &[
    'j' as i32, 'j' as i32, 'o' as i32, 'j' as i32, 'j' as i32, 'j' as i32, 'l' as i32,
    'l' as i32, 'i' as i32, ESC, '.' as i32, 'g' as i32, '>' as i32, '<' as i32,
];

fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("sim-s8rec-{}-{name}", std::process::id()));
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }
    path
}

fn spawn_server(
    socket: PathBuf,
    record_path: Option<PathBuf>,
) -> JoinHandle<proto::Result<()>> {
    let config = sim::ServerConfig {
        socket_path: socket,
        defs_path: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tools/sim-defs/generated/defs.json"),
        ),
        scene_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenes/street_block.toml"),
        weather: None,
        ui_shm_path: None,
        ui_start: None,
        seed: Some(42),
        record_path,
    };
    std::thread::spawn(move || sim::run_server(&config))
}

fn connect(path: &PathBuf) -> UnixStream {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match UnixStream::connect(path) {
            Ok(s) => {
                s.set_read_timeout(Some(TIMEOUT)).unwrap();
                return s;
            }
            Err(e) => {
                assert!(Instant::now() < deadline, "server never listened: {e}");
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn join_server(handle: JoinHandle<proto::Result<()>>) {
    let deadline = Instant::now() + TIMEOUT;
    while !handle.is_finished() {
        assert!(Instant::now() < deadline, "server did not finish (hang?)");
        std::thread::sleep(Duration::from_millis(10));
    }
    handle
        .join()
        .expect("server thread panicked")
        .expect("server returned an error");
}

/// Read one frame's raw payload bytes (the exact bytes the
/// determinism claim is about).
fn recv_frame(stream: &mut UnixStream, dec: &mut FrameDecoder) -> Vec<u8> {
    loop {
        if let Some(frame) = dec.next_frame().expect("frame decode") {
            return frame;
        }
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).expect("client read");
        assert!(n > 0, "server closed mid-session");
        dec.push(&buf[..n]);
    }
}

fn send_msg(stream: &mut UnixStream, msg: &MessageT) {
    stream
        .write_all(&FrameEncoder::encode(&encode_message(msg)))
        .expect("client send");
}

/// Drive one full session with `keys`, collecting every
/// frame the server sends, in order. Returns the stream.
fn drive_session(socket: &PathBuf, record_path: Option<PathBuf>, keys: &[i32]) -> Vec<Vec<u8>> {
    let server = spawn_server(socket.clone(), record_path);
    let mut stream = connect(socket);
    let mut dec = FrameDecoder::new();
    let mut stream_bytes: Vec<Vec<u8>> = Vec::new();

    // Hello / HelloAck / TileDefs / Snapshot / UiFrame.
    stream_bytes.push(recv_frame(&mut stream, &mut dec));
    send_msg(
        &mut stream,
        &MessageT::HelloAck(Box::new(sample_hello_ack())),
    );
    for _ in 0..3 {
        stream_bytes.push(recv_frame(&mut stream, &mut dec));
    }

    for (i, &code) in keys.iter().enumerate() {
        let mut input = InputT::default();
        input.kind = InputKind::Key;
        input.key_code = code;
        input.pressed = true;
        send_msg(&mut stream, &MessageT::Input(Box::new(input)));
        // Ping barrier: read until this Ping's Pong; every
        // frame before it belongs to the input's flush.
        let mut ping = PingT::default();
        ping.nonce = 10_000 + i as u64;
        ping.sent_ms = 5_000 + i as u64;
        send_msg(&mut stream, &MessageT::Ping(Box::new(ping)));
        loop {
            let frame = recv_frame(&mut stream, &mut dec);
            let is_pong = matches!(decode_message(&frame), Ok(MessageT::Pong(_)));
            stream_bytes.push(frame);
            if is_pong {
                break;
            }
        }
    }

    let mut shutdown = ShutdownT::default();
    shutdown.reason = Some("record gate done".into());
    send_msg(&mut stream, &MessageT::Shutdown(Box::new(shutdown)));
    drop(stream);
    join_server(server);
    stream_bytes
}

#[test]
fn s8_recorded_session_replays_byte_identical() {
    let socket = temp_path("session.sock");
    let record_file = temp_path("session.toml");

    // The recorded live session.
    let live = drive_session(&socket, Some(record_file.clone()), KEYS);
    assert!(
        record_file.exists(),
        "the server must write the session log on exit"
    );
    let text = std::fs::read_to_string(&record_file).expect("record file reads");

    // The log is a scenario file: parse it with the runner's
    // own parser and check its header + steps.
    let scenario = Scenario::parse(&text).expect("recorded log parses as a scenario");
    assert!(scenario.is_green(), "recorded log must be runnable");
    assert_eq!(scenario.scene, "street_block");
    assert_eq!(scenario.seed, Some(42), "the effective seed is recorded");
    let recorded_keys: Vec<i32> = scenario
        .steps
        .iter()
        .map(|s| match &s.kind {
            StepKind::Action(Action::Key(code)) => *code,
            other => panic!("recorded step is not a key step: {other:?}"),
        })
        .collect();
    assert_eq!(
        recorded_keys, KEYS,
        "every pressed key of the session is recorded, in order"
    );

    // Replay: a fresh server, same socket path + seed, fed
    // exactly the recorded keys. (Also proves the recorded
    // file's steps run green through the runner's in-process
    // mode — the same world code the server drives.)
    let reg = sim::DefRegistry::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tools/sim-defs/generated/defs.json"),
    )
    .expect("registry loads");
    let scene = sim::Scene::load_file(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenes/street_block.toml"),
    )
    .expect("scene loads");
    sim::scenario::run_in_process(&reg, scene, &scenario)
        .unwrap_or_else(|f| panic!("recorded scenario must run green in-process:\n{f}"));

    let replay = drive_session(&socket, None, &recorded_keys);

    assert_eq!(
        live.len(),
        replay.len(),
        "stream lengths differ: live {} frames vs replay {} frames",
        live.len(),
        replay.len()
    );
    for (i, (a, b)) in live.iter().zip(replay.iter()).enumerate() {
        assert_eq!(
            a, b,
            "frame {i} differs between the live session and its replay \
             ({} vs {} bytes) — §3.1 determinism broken",
            a.len(),
            b.len()
        );
    }
    println!(
        "record/replay: {} frames, {} bytes, byte-identical",
        live.len(),
        live.iter().map(|f| f.len()).sum::<usize>()
    );
}
