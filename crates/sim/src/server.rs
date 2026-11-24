//! Unix-socket server for the sim test game (slices S0 + S1).
//!
//! Lifecycle: bind, accept ONE connection, serve it until Shutdown,
//! disconnect, or a fatal protocol error, then close the listener,
//! unlink the socket path, and return `Ok(())`. A handshake version
//! mismatch is a clean exit, never a panic (docs/protocol.md).
//!
//! S1 behaviour: after the HelloAck, the server sends the FULL
//! registry TileDefs (all 14,262 defs from the S1a generated
//! table) and then the configured scene's Snapshot, built by
//! [`crate::world::World`]. The rest of the loop is S0's: Input
//! echo (the turn loop is S2), Ping/Pong, Resize/PickResult
//! logging, Shutdown.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use proto::samples::sample_hello;
use proto::{
    check_handshake, decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT, PongT,
    PROTOCOL_VERSION,
};

use crate::defs::DefRegistry;
use crate::scenes::Scene;
use crate::world::World;

/// Everything the server needs, resolved before binding.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub socket_path: PathBuf,
    /// Explicit defs path; `None` = registry resolution order
    /// (env `CDDA_SIM_DEFS`, then CWD-relative default).
    pub defs_path: Option<PathBuf>,
    /// Scene file to load and serve.
    pub scene_path: PathBuf,
}

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

/// Run the server on `config.socket_path` until the (single)
/// client shuts it down or disconnects. The socket file is
/// removed on exit. Registry/scene load errors are returned
/// before binding, with messages naming what was tried.
pub fn run_server(config: &ServerConfig) -> proto::Result<()> {
    let reg = DefRegistry::load_resolved(config.defs_path.as_deref())
        .map_err(|e| proto::ProtoError::Malformed(format!("defs: {e}")))?;
    let scene = Scene::load_file(&config.scene_path)
        .map_err(|e| proto::ProtoError::Malformed(format!("scene: {e}")))?;
    let mut world = World::from_scene(&reg, &scene)
        .map_err(|e| proto::ProtoError::Malformed(format!("world: {e}")))?;
    eprintln!(
        "sim-server: loaded {} defs, scene '{}' ({} blocks)",
        reg.len(),
        scene.name,
        world.blocks.len()
    );

    let socket_path = &config.socket_path;
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    let listener = UnixListener::bind(socket_path)?;
    eprintln!("sim-server: listening on {}", socket_path.display());

    let result = serve_one(&listener, &reg, &mut world);

    // Clean exit path for every outcome (shutdown, disconnect,
    // version mismatch): close the listener (drop) and unlink.
    drop(listener);
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }
    result
}

fn serve_one(listener: &UnixListener, reg: &DefRegistry, world: &mut World) -> proto::Result<()> {
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

    // TileDefs (the full registry table), then the scene Snapshot.
    send(
        &mut stream,
        &MessageT::TileDefs(Box::new(reg.tiledefs().clone())),
    )?;
    send(
        &mut stream,
        &MessageT::Snapshot(Box::new(world.snapshot(reg))),
    )?;
    // DEVIATION from the C++ stub server: the stub also sends one
    // standalone EnvironmentUpdate after the Snapshot. The sim does
    // NOT: the existing proto-client expects the Input echo as the
    // next message after the Snapshot and would fail on an
    // intervening EnvironmentUpdate. The Snapshot already
    // piggybacks the current environment where the scene carries
    // one, so no information is lost. Standalone EnvironmentUpdates
    // belong to the S3 environment slice, where they will be sent
    // on change / on the 10-game-minute cadence.
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
                // Echo the input back verbatim (S0/S1: no turn loop yet).
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
