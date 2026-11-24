//! S0/S1 gate tests: the sim server over a real Unix socket, driven
//! by a client speaking the Phase 1 wire through `proto` framing —
//! the same path `proto-client` uses. S1: the server sends the
//! FULL registry TileDefs (14,262 defs) and the stub_house scene
//! Snapshot.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use proto::samples::{sample_hello_ack, sample_input};
use proto::{
    decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT, PingT, ShutdownT,
    PROTOCOL_VERSION,
};
use sim::ServerConfig;

const TIMEOUT: Duration = Duration::from_secs(30);

fn temp_socket(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("sim-s0-{}-{name}.sock", std::process::id()));
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }
    path
}

fn spawn_server(path: PathBuf) -> JoinHandle<proto::Result<()>> {
    let config = ServerConfig {
        socket_path: path,
        defs_path: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tools/sim-defs/generated/defs.json"),
        ),
        scene_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenes/stub_house.toml"),
    };
    std::thread::spawn(move || sim::run_server(&config))
}

/// Poll-connect until the server is listening (no fixed sleeps).
fn connect(path: &PathBuf) -> UnixStream {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match UnixStream::connect(path) {
            Ok(s) => {
                s.set_read_timeout(Some(TIMEOUT)).unwrap();
                return s;
            }
            Err(e) => {
                assert!(
                    Instant::now() < deadline,
                    "server never started listening on {}: {e}",
                    path.display()
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn send(stream: &mut UnixStream, msg: &MessageT) {
    let framed = FrameEncoder::encode(&encode_message(msg));
    stream.write_all(&framed).expect("client send");
}

/// Receive one message; panics on EOF/error (tests that expect EOF
/// use `recv_err`).
fn recv(stream: &mut UnixStream, dec: &mut FrameDecoder) -> MessageT {
    recv_opt(stream, dec).expect("expected a message, connection closed")
}

fn recv_opt(stream: &mut UnixStream, dec: &mut FrameDecoder) -> Option<MessageT> {
    loop {
        if let Some(frame) = dec.next_frame().expect("frame decode") {
            return Some(decode_message(&frame).expect("message decode"));
        }
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).expect("client read");
        if n == 0 {
            return None;
        }
        dec.push(&buf[..n]);
    }
}

/// Assert the server thread finishes cleanly within the timeout.
fn join_server(handle: JoinHandle<proto::Result<()>>) {
    let deadline = Instant::now() + TIMEOUT;
    while !handle.is_finished() {
        assert!(
            Instant::now() < deadline,
            "server thread did not finish within {TIMEOUT:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    handle
        .join()
        .expect("server thread panicked")
        .expect("server returned an error");
}

#[test]
fn s0_handshake_snapshot_echo_ping_shutdown() {
    let path = temp_socket("flow");
    let server = spawn_server(path.clone());
    let mut stream = connect(&path);
    let mut dec = FrameDecoder::new();

    // Hello, with the current protocol version.
    match recv(&mut stream, &mut dec) {
        MessageT::Hello(h) => {
            assert_eq!(h.protocol_version, PROTOCOL_VERSION);
        }
        other => panic!("expected Hello, got {other:?}"),
    }
    send(
        &mut stream,
        &MessageT::HelloAck(Box::new(sample_hello_ack())),
    );

    // TileDefs (the FULL registry table), then Snapshot (seq/epoch
    // 1, one block, environment piggybacked).
    match recv(&mut stream, &mut dec) {
        MessageT::TileDefs(td) => {
            let defs = td.defs.unwrap_or_default();
            assert_eq!(defs.len(), 14_262, "full Option C def table");
            assert_eq!(
                td.connect_groups.unwrap_or_default().len(),
                40,
                "connect-group registry"
            );
        }
        other => panic!("expected TileDefs, got {other:?}"),
    }
    match recv(&mut stream, &mut dec) {
        MessageT::Snapshot(s) => {
            assert_eq!(s.seq, 1);
            assert_eq!(s.epoch, 1);
            assert_eq!(s.blocks.as_ref().map_or(0, |b| b.len()), 1);
            assert!(s.environment.is_some(), "snapshot must carry environment");
        }
        other => panic!("expected Snapshot, got {other:?}"),
    }

    // Input echo is semantically equal to what was sent.
    let input = sample_input();
    send(&mut stream, &MessageT::Input(Box::new(input.clone())));
    match recv(&mut stream, &mut dec) {
        MessageT::Input(echo) => assert_eq!(*echo, input),
        other => panic!("expected Input echo, got {other:?}"),
    }

    // Ping -> Pong with the same nonce (and sent_ms).
    let mut ping = PingT::default();
    ping.nonce = 1234;
    ping.sent_ms = 5000;
    send(&mut stream, &MessageT::Ping(Box::new(ping)));
    match recv(&mut stream, &mut dec) {
        MessageT::Pong(p) => {
            assert_eq!(p.nonce, 1234);
            assert_eq!(p.sent_ms, 5000);
        }
        other => panic!("expected Pong, got {other:?}"),
    }

    // Shutdown -> server finishes cleanly, socket file removed.
    let mut shutdown = ShutdownT::default();
    shutdown.reason = Some("test done".into());
    send(&mut stream, &MessageT::Shutdown(Box::new(shutdown)));
    drop(stream);
    join_server(server);
    assert!(!path.exists(), "socket file must be unlinked on exit");
}

#[test]
fn s0_version_mismatch_closes_without_tiledefs() {
    let path = temp_socket("mismatch");
    let server = spawn_server(path.clone());
    let mut stream = connect(&path);
    let mut dec = FrameDecoder::new();

    match recv(&mut stream, &mut dec) {
        MessageT::Hello(h) => assert_eq!(h.protocol_version, PROTOCOL_VERSION),
        other => panic!("expected Hello, got {other:?}"),
    }

    // HelloAck with a bogus version: the server must close without
    // sending TileDefs and without panicking.
    let mut ack = sample_hello_ack();
    ack.protocol_version = 999;
    send(&mut stream, &MessageT::HelloAck(Box::new(ack)));

    let after = recv_opt(&mut stream, &mut dec);
    assert!(
        after.is_none(),
        "server must close on version mismatch, got {after:?}"
    );
    drop(stream);
    join_server(server);
    assert!(!path.exists(), "socket file must be unlinked on exit");
}
