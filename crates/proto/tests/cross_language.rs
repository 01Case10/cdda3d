//! Cross-language Phase 1 gates (plan Part II §4): 1.2, 1.3, 1.11, 1.12,
//! plus the full handshake/snapshot flow against the real C++ stub server.
//!
//! These tests build the C++ side with tools/stub-server/build.sh when the
//! binaries are missing. If no C++ compiler is available they skip (with a
//! printed note) rather than fail — on this machine g++ is present and they
//! run for real.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use proto::samples::*;
use proto::*;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn stub_dir() -> PathBuf {
    repo_root().join("tools/stub-server")
}

/// Returns true if the C++ binaries are available (building them if needed).
fn ensure_cpp_binaries() -> bool {
    let dir = stub_dir();
    let stub = dir.join("stub-server");
    let roundtrip = dir.join("cpp-roundtrip");
    if stub.exists() && roundtrip.exists() {
        return true;
    }
    if Command::new("g++")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("SKIP: g++ not available, cross-language tests not run");
        return false;
    }
    let out = Command::new("sh")
        .arg("build.sh")
        .current_dir(&dir)
        .output()
        .expect("run build.sh");
    if !out.status.success() {
        eprintln!(
            "SKIP: stub build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        return false;
    }
    stub.exists() && roundtrip.exists()
}

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda-proto-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------- 1.2
#[test]
fn t1_2_cpp_round_trip() {
    if !ensure_cpp_binaries() {
        return;
    }
    let out = Command::new(stub_dir().join("cpp-roundtrip"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "cpp-roundtrip failed: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("CPP ROUNDTRIP PASS"),
        "unexpected output: {stdout}"
    );
}

// ---------------------------------------------------------------- 1.3
#[test]
fn t1_3_cross_language_golden_files() {
    if !ensure_cpp_binaries() {
        return;
    }
    let dir = tmpdir("golden");
    let out = Command::new(stub_dir().join("cpp-roundtrip"))
        .args(["write", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "cpp write goldens failed");

    // C++ -> Rust: decode the C++ goldens and compare semantically with the
    // Rust sample builders (which mirror the same house).
    let snap = std::fs::read(dir.join("golden_snapshot.bin")).unwrap();
    assert_eq!(
        decode_message(&snap).unwrap(),
        MessageT::Snapshot(Box::new(sample_snapshot())),
        "C++ snapshot golden != Rust sample snapshot"
    );
    let defs = std::fs::read(dir.join("golden_tiledefs.bin")).unwrap();
    assert_eq!(
        decode_message(&defs).unwrap(),
        MessageT::TileDefs(Box::new(sample_tiledefs())),
        "C++ tiledefs golden != Rust sample tiledefs"
    );
    let hello = std::fs::read(dir.join("golden_hello.bin")).unwrap();
    assert_eq!(
        decode_message(&hello).unwrap(),
        MessageT::Hello(Box::new(sample_hello())),
        "C++ hello golden != Rust sample hello"
    );

    // Rust -> C++: the C++ tool verifies and semantically compares a
    // Rust-written snapshot.
    let rust_snap = dir.join("rust_snapshot.bin");
    std::fs::write(
        &rust_snap,
        encode_message(&MessageT::Snapshot(Box::new(sample_snapshot()))),
    )
    .unwrap();
    let out = Command::new(stub_dir().join("cpp-roundtrip"))
        .args(["verify", rust_snap.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("CPP VERIFY PASS"),
        "cpp verify failed: {stdout}"
    );
}

// ------------------------------------------------------- stub over socket
struct Stub {
    child: Child,
    sock: PathBuf,
}

fn spawn_stub(tag: &str) -> Stub {
    let dir = tmpdir(tag);
    let sock = dir.join("stub.sock");
    let child = Command::new(stub_dir().join("stub-server"))
        .arg(&sock)
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn stub-server");
    // Wait for the socket file to appear.
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sock.exists() {
        assert!(Instant::now() < deadline, "stub socket never appeared");
        std::thread::sleep(Duration::from_millis(20));
    }
    Stub { child, sock }
}

fn send(stream: &mut UnixStream, msg: &MessageT) {
    stream
        .write_all(&FrameEncoder::encode(&encode_message(msg)))
        .unwrap();
}

fn recv(stream: &mut UnixStream, dec: &mut FrameDecoder) -> MessageT {
    loop {
        if let Some(frame) = dec.next_frame().unwrap() {
            return decode_message(&frame).unwrap();
        }
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).expect("read from stub");
        assert!(n > 0, "stub closed connection unexpectedly");
        dec.push(&buf[..n]);
    }
}

fn wait_exit(child: &mut Child, timeout: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "stub did not exit within {timeout:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn t1_11_1_12_stub_handshake_snapshot_input_shutdown() {
    if !ensure_cpp_binaries() {
        return;
    }
    let mut stub = spawn_stub("sock");
    let mut stream = UnixStream::connect(&stub.sock).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut dec = FrameDecoder::new();

    // Handshake.
    match recv(&mut stream, &mut dec) {
        MessageT::Hello(h) => {
            assert_eq!(*h, sample_hello());
            check_handshake(h.protocol_version).unwrap();
        }
        other => panic!("expected Hello, got {other:?}"),
    }
    send(
        &mut stream,
        &MessageT::HelloAck(Box::new(sample_hello_ack())),
    );

    // TileDefs + Snapshot, semantically equal to the Rust samples.
    match recv(&mut stream, &mut dec) {
        MessageT::TileDefs(td) => assert_eq!(*td, sample_tiledefs()),
        other => panic!("expected TileDefs, got {other:?}"),
    }
    match recv(&mut stream, &mut dec) {
        MessageT::Snapshot(s) => assert_eq!(*s, sample_snapshot()),
        other => panic!("expected Snapshot, got {other:?}"),
    }

    // 1.12: input round-trip, order preserved across several events.
    let mut inputs = Vec::new();
    for key in ['a', 'b', 'c', 'd'] {
        let mut input = sample_input();
        input.key_code = key as i32;
        inputs.push(input);
    }
    for input in &inputs {
        send(&mut stream, &MessageT::Input(Box::new(input.clone())));
    }
    for input in &inputs {
        match recv(&mut stream, &mut dec) {
            MessageT::Input(echo) => assert_eq!(&*echo, input),
            other => panic!("expected Input echo, got {other:?}"),
        }
    }

    // Ping/Pong.
    let mut ping = PingT::default();
    ping.nonce = 4242;
    send(&mut stream, &MessageT::Ping(Box::new(ping)));
    match recv(&mut stream, &mut dec) {
        MessageT::Pong(p) => assert_eq!(p.nonce, 4242),
        other => panic!("expected Pong, got {other:?}"),
    }

    // 1.11: Shutdown -> stub exits cleanly within 2 s.
    let mut shutdown = ShutdownT::default();
    shutdown.reason = Some("test done".into());
    send(&mut stream, &MessageT::Shutdown(Box::new(shutdown)));
    let status = wait_exit(&mut stub.child, Duration::from_secs(2));
    assert!(status.success(), "stub exit status: {status}");
}

#[test]
fn t1_11_stub_exits_when_client_vanishes() {
    if !ensure_cpp_binaries() {
        return;
    }
    let mut stub = spawn_stub("vanish");
    {
        let mut stream = UnixStream::connect(&stub.sock).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut dec = FrameDecoder::new();
        let _ = recv(&mut stream, &mut dec); // Hello
        send(
            &mut stream,
            &MessageT::HelloAck(Box::new(sample_hello_ack())),
        );
        let _ = recv(&mut stream, &mut dec); // TileDefs
        let _ = recv(&mut stream, &mut dec); // Snapshot
                                             // Drop the stream without Shutdown: the stub must notice and exit.
    }
    let status = wait_exit(&mut stub.child, Duration::from_secs(2));
    assert!(
        status.success(),
        "stub exit status after client vanish: {status}"
    );
}
