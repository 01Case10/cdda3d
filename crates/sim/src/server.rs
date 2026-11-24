//! Unix-socket server for the sim test game (slice S0).
//!
//! Behaviour mirrors the C++ stub server
//! (`tools/stub-server/stub_server.cpp`) message for message, with
//! ONE deliberate deviation, noted at the Snapshot send below.
//!
//! Lifecycle: bind, accept ONE connection, serve it until Shutdown,
//! disconnect, or a fatal protocol error, then close the listener,
//! unlink the socket path, and return `Ok(())`. A handshake version
//! mismatch is a clean exit, never a panic (docs/protocol.md).

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

use proto::samples::{sample_hello, sample_snapshot, sample_tiledefs};
use proto::{
    check_handshake, decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT, PongT,
    PROTOCOL_VERSION,
};

fn send(stream: &mut UnixStream, msg: &MessageT) -> proto::Result<()> {
    let framed = FrameEncoder::encode(&encode_message(msg));
    stream.write_all(&framed)?;
    Ok(())
}

/// Receive the next message, or `Ok(None)` on EOF / clean disconnect.
fn recv(stream: &mut UnixStream, dec: &mut FrameDecoder) -> proto::Result<Option<MessageT>> {
    loop {
        if let Some(frame) = dec.next_frame()? {
            return Ok(Some(decode_message(&frame)?));
        }
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        dec.push(&buf[..n]);
    }
}

/// Run the server on `socket_path` until the (single) client shuts
/// it down or disconnects. The socket file is removed on exit.
pub fn run_server(socket_path: &Path) -> proto::Result<()> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    let listener = UnixListener::bind(socket_path)?;
    eprintln!("sim-server: listening on {}", socket_path.display());

    let result = serve_one(&listener);

    // Clean exit path for every outcome (shutdown, disconnect,
    // version mismatch): close the listener (drop) and unlink.
    drop(listener);
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }
    result
}

fn serve_one(listener: &UnixListener) -> proto::Result<()> {
    let (mut stream, _) = listener.accept()?;
    let mut dec = FrameDecoder::new();

    // Handshake: Hello out, HelloAck in, exact version match.
    send(&mut stream, &MessageT::Hello(Box::new(sample_hello())))?;
    let ack = match recv(&mut stream, &mut dec)? {
        Some(MessageT::HelloAck(ack)) => ack,
        Some(other) => {
            eprintln!("sim-server: expected HelloAck, got {other:?}; closing");
            return Ok(());
        }
        None => {
            eprintln!("sim-server: client disconnected before HelloAck");
            return Ok(());
        }
    };
    if let Err(e) = check_handshake(ack.protocol_version) {
        // Version mismatch: log both versions and exit cleanly —
        // no TileDefs, no panic (docs/protocol.md handshake rule).
        eprintln!(
            "sim-server: protocol mismatch: ours={} theirs={} ({e}); closing",
            PROTOCOL_VERSION, ack.protocol_version
        );
        return Ok(());
    }
    eprintln!(
        "sim-server: helloack engine={:?} screen={}x{}",
        ack.engine_version, ack.screen_width, ack.screen_height
    );

    // TileDefs, then the (static, S0) Snapshot.
    send(
        &mut stream,
        &MessageT::TileDefs(Box::new(sample_tiledefs())),
    )?;
    send(
        &mut stream,
        &MessageT::Snapshot(Box::new(sample_snapshot())),
    )?;
    // DEVIATION from the C++ stub server: the stub also sends one
    // standalone EnvironmentUpdate after the Snapshot. S0 does NOT:
    // the existing proto-client expects the Input echo as the next
    // message after the Snapshot and would fail on an intervening
    // EnvironmentUpdate. The Snapshot already piggybacks the current
    // environment, so no information is lost. Standalone
    // EnvironmentUpdates belong to the S3 environment slice, where
    // they will be sent on change / on the 10-game-minute cadence.
    eprintln!("sim-server: sent tiledefs + snapshot");

    // Main loop.
    loop {
        let msg = match recv(&mut stream, &mut dec) {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                eprintln!("sim-server: client disconnected");
                return Ok(());
            }
            Err(e) => {
                eprintln!("sim-server: receive error ({e}); closing");
                return Ok(());
            }
        };
        match msg {
            MessageT::Input(input) => {
                eprintln!(
                    "sim-server: input kind={:?} key={} pressed={}",
                    input.kind, input.key_code, input.pressed
                );
                // Echo the input back verbatim (S0: no turn loop yet).
                send(&mut stream, &MessageT::Input(input))?;
            }
            MessageT::Ping(ping) => {
                // Generated types are #[non_exhaustive] outside the
                // proto crate: build via Default + field assignment.
                let mut pong = PongT::default();
                pong.nonce = ping.nonce;
                pong.sent_ms = ping.sent_ms;
                send(&mut stream, &MessageT::Pong(Box::new(pong)))?;
            }
            MessageT::Resize(r) => {
                eprintln!("sim-server: resize {}x{} dpi={}", r.width, r.height, r.dpi);
            }
            MessageT::PickResult(p) => {
                eprintln!("sim-server: pick hit={} pos={:?}", p.hit, p.pos);
            }
            MessageT::Shutdown(s) => {
                eprintln!("sim-server: shutdown reason={:?}", s.reason);
                return Ok(());
            }
            other => {
                eprintln!("sim-server: unexpected message {other:?}; ignoring");
            }
        }
    }
}
