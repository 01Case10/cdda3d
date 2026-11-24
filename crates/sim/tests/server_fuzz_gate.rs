//! S8 gate: T10 abuse half — malformed-frame fuzz against
//! the sim's own server side.
//!
//! In the spirit of proto's fuzz gates (gate_tests.rs
//! t1_4_and_1_5: a deterministic seeded corruption sweep, no
//! external fuzzer): a fixed corpus of hostile connections
//! is driven at a real `sim-server` over its Unix socket —
//! garbage bytes, truncated frames, absurd length prefixes
//! (including > the 32 MiB cap), valid-prefix / invalid
//! FlatBuffers payloads, empty reads, rapid connect /
//! disconnect — before AND after the handshake. Assertions:
//! no panic, no hang (every client op is timeout-bounded),
//! each bad connection is rejected / closed cleanly, and the
//! server still completes a full normal session afterwards
//! (S7's sequential-accept change makes that meaningful).
//!
//! The wrong-protocol-version case is asserted separately,
//! on its own server instance: by the S0 handshake rule
//! (docs/protocol.md) a version mismatch is a clean server
//! EXIT, not a rejected connection — the fuzz assertion is
//! that it exits cleanly (no panic, socket unlinked), which
//! server_gate's s0_version_mismatch test also pins.
//!
//! Failing-first: pre-S8, a malformed FIRST frame (before the
//! HelloAck) propagated its decode/framing error out of
//! `serve_one` and killed the accept loop — the server died
//! on the first hostile connection and the "normal session
//! afterwards" half of this gate could never run. The fix
//! (server.rs) treats per-connection decode/framing errors
//! like the post-handshake path already did: the connection
//! ends (ClientGone), the server lives.

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
    decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT, PingT, ShutdownT,
};

const TIMEOUT: Duration = Duration::from_secs(15);

fn temp_socket(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("sim-s8fuzz-{}-{name}.sock", std::process::id()));
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }
    path
}

fn spawn_server(path: PathBuf) -> JoinHandle<proto::Result<()>> {
    let config = sim::ServerConfig {
        socket_path: path,
        defs_path: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tools/sim-defs/generated/defs.json"),
        ),
        scene_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenes/stub_house.toml"),
        weather: None,
        ui_shm_path: None,
        ui_start: None,
        seed: None,
        record_path: None,
    };
    std::thread::spawn(move || sim::run_server(&config))
}

fn connect(path: &PathBuf) -> UnixStream {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match UnixStream::connect(path) {
            Ok(s) => {
                s.set_read_timeout(Some(TIMEOUT)).unwrap();
                s.set_write_timeout(Some(TIMEOUT)).unwrap();
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
        assert!(
            Instant::now() < deadline,
            "server thread did not finish within {TIMEOUT:?} (hang?)"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    handle
        .join()
        .expect("server thread panicked")
        .expect("server returned an error");
}

/// Read the server's Hello (every connection starts with one).
fn read_hello(stream: &mut UnixStream, dec: &mut FrameDecoder) {
    loop {
        if let Some(frame) = dec.next_frame().expect("frame decode") {
            match decode_message(&frame).expect("message decode") {
                MessageT::Hello(_) => return,
                other => panic!("expected Hello, got {other:?}"),
            }
        }
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).expect("client read");
        assert!(n > 0, "server closed before Hello");
        dec.push(&buf[..n]);
    }
}

fn recv_msg(stream: &mut UnixStream, dec: &mut FrameDecoder) -> MessageT {
    loop {
        if let Some(frame) = dec.next_frame().expect("frame decode") {
            return decode_message(&frame).expect("message decode");
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

/// One hostile connection: connect, read the Hello, fire the
/// payload, close. The server must survive every one of these.
fn hostile(path: &PathBuf, payload: &[u8]) {
    let mut stream = connect(path);
    let mut dec = FrameDecoder::new();
    read_hello(&mut stream, &mut dec);
    if !payload.is_empty() {
        // The write itself may fail if the server closes
        // early — that IS a clean rejection, not a failure.
        let _ = stream.write_all(payload);
    }
    drop(stream);
}

/// A complete, well-behaved session: handshake, TileDefs,
/// Snapshot, UiFrame, one turn, Ping/Pong, Shutdown.
fn normal_session(path: &PathBuf) {
    let mut stream = connect(path);
    let mut dec = FrameDecoder::new();
    read_hello(&mut stream, &mut dec);
    send_msg(
        &mut stream,
        &MessageT::HelloAck(Box::new(sample_hello_ack())),
    );
    match recv_msg(&mut stream, &mut dec) {
        MessageT::TileDefs(_) => {}
        other => panic!("expected TileDefs, got {other:?}"),
    }
    match recv_msg(&mut stream, &mut dec) {
        MessageT::Snapshot(_) => {}
        other => panic!("expected Snapshot, got {other:?}"),
    }
    match recv_msg(&mut stream, &mut dec) {
        MessageT::UiFrame(_) => {}
        other => panic!("expected UiFrame, got {other:?}"),
    }
    let mut ping = PingT::default();
    ping.nonce = 0x5eed;
    send_msg(&mut stream, &MessageT::Ping(Box::new(ping)));
    match recv_msg(&mut stream, &mut dec) {
        MessageT::Pong(p) => assert_eq!(p.nonce, 0x5eed),
        other => panic!("expected Pong, got {other:?}"),
    }
    let mut shutdown = ShutdownT::default();
    shutdown.reason = Some("fuzz gate done".into());
    send_msg(&mut stream, &MessageT::Shutdown(Box::new(shutdown)));
    drop(stream);
}

/// Deterministic garbage: xorshift64 with a fixed seed (the
/// fork harness mock's PRNG family, like the sim's own RNG).
fn garbage(seed: u64, n: usize) -> Vec<u8> {
    let mut s = if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed };
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        out.extend_from_slice(&s.to_le_bytes());
    }
    out.truncate(n);
    out
}

#[test]
fn s8_server_survives_the_malformed_corpus() {
    let path = temp_socket("corpus");
    let server = spawn_server(path.clone());

    // 1. Pure garbage as the first frame (its length prefix
    //    is whatever the bytes say — usually absurd).
    hostile(&path, &garbage(1, 256));
    // 2. An explicit absurd length prefix: 0xFFFFFFFF
    //    (>> the 32 MiB cap), rejected from the prefix alone.
    hostile(&path, &[0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 3, 4]);
    // 3. Just over the cap: 32 MiB + 1.
    hostile(&path, &33_554_433u32.to_le_bytes());
    // 4. A valid length prefix wrapping an invalid
    //    FlatBuffers payload (no CDDA identifier).
    let mut framed = Vec::new();
    framed.extend_from_slice(&64u32.to_le_bytes());
    framed.extend_from_slice(&[0xAB; 64]);
    hostile(&path, &framed);
    // 5. A truncated frame: two bytes of a length prefix,
    //    then EOF.
    hostile(&path, &[0x40, 0x00]);
    // 6. A well-formed frame carrying the WRONG first
    //    message (a Ping instead of the HelloAck).
    let mut ping = PingT::default();
    ping.nonce = 7;
    hostile(
        &path,
        &FrameEncoder::encode(&encode_message(&MessageT::Ping(Box::new(ping)))),
    );
    // 7. Empty read: connect, read Hello, close at once.
    hostile(&path, &[]);
    // 8. Rapid connect/disconnect, no bytes at all.
    for _ in 0..20 {
        drop(connect(&path));
    }

    // Post-handshake hostility: complete a handshake, THEN
    // send garbage / an oversize prefix. (Each of these ends
    // that session; the server must take the next one.)
    for payload in [
        garbage(2, 128),
        vec![0xFF, 0xFF, 0xFF, 0xFF],
        {
            let mut f = Vec::new();
            f.extend_from_slice(&32u32.to_le_bytes());
            f.extend_from_slice(&[0x00; 32]);
            f
        },
    ] {
        let mut stream = connect(&path);
        let mut dec = FrameDecoder::new();
        read_hello(&mut stream, &mut dec);
        send_msg(
            &mut stream,
            &MessageT::HelloAck(Box::new(sample_hello_ack())),
        );
        // Drain TileDefs + Snapshot + UiFrame.
        for _ in 0..3 {
            let _ = recv_msg(&mut stream, &mut dec);
        }
        let _ = stream.write_all(&payload);
        drop(stream);
    }

    // The point of the exercise: after all of that, a normal
    // session still completes end to end, and Shutdown exits
    // the server cleanly.
    normal_session(&path);
    join_server(server);
    assert!(!path.exists(), "socket file must be unlinked on exit");
}

#[test]
fn s8_wrong_version_is_a_clean_exit_not_a_panic() {
    let path = temp_socket("version");
    let server = spawn_server(path.clone());
    let mut stream = connect(&path);
    let mut dec = FrameDecoder::new();
    read_hello(&mut stream, &mut dec);
    let mut ack = sample_hello_ack();
    ack.protocol_version = 999;
    send_msg(&mut stream, &MessageT::HelloAck(Box::new(ack)));
    drop(stream);
    // The S0 rule: mismatch = clean server exit.
    join_server(server);
    assert!(!path.exists(), "socket file must be unlinked on exit");
}
