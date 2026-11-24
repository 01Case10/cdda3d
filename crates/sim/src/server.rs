//! Unix-socket server for the sim test game (slices S0 + S1).
//!
//! Lifecycle: bind, build the world ONCE, then accept connections
//! one at a time, serving each until it ends. The world persists
//! across connections (S7: a client that disconnects and
//! reconnects resumes the same world — the resync behaviour T10
//! scripts). The server exits on a Shutdown message or a
//! handshake version mismatch (both clean exits, never a panic —
//! docs/protocol.md), then closes the listener, unlinks the
//! socket path, and returns `Ok(())`.
//!
//! S1 behaviour: after the HelloAck, the server sends the FULL
//! registry TileDefs (all 14,262 defs from the S1a generated
//! table) and then the configured scene's Snapshot, built by
//! [`crate::world::World`].
//!
//! S2 behaviour: the Input ECHO of S0/S1 is gone — it was stub
//! semantics. A pressed Key Input is one game turn, resolved by
//! [`crate::rules`] and answered with exactly one Delta (or a
//! fresh Snapshot on a map shift); key releases and non-Key
//! inputs get no reply. Ping/Pong, Resize/PickResult logging,
//! and Shutdown are unchanged from S0.
//!
//! S4 behaviour: after the Snapshot, the server attaches the UI
//! shared-memory file (at the HelloAck's screen size) and sends
//! the initial UiFrame; UI publishes during the run are single
//! UiFrame replies from the input routing (see [`crate::rules`]
//! module docs). Resize recreates the UI store at the new
//! geometry and is answered by one UiFrame; PickResult is
//! recorded on the world. The shm file is removed on exit.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use proto::samples::sample_hello;
use proto::{
    check_handshake, decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT, PongT,
    PROTOCOL_VERSION,
};

use crate::defs::DefRegistry;
use crate::scenes::Scene;
use crate::ui::DEFAULT_UI_SIZE;
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
    /// Weather preset override (S3); `None` = the scene's own
    /// `weather` key. Mid-run changes use the scene's
    /// `[[weather_script]]` entries or `World::set_weather`.
    pub weather: Option<String>,
    /// UI shared-memory file path (S4); `None` = derive from
    /// the socket path (`<socket>.ui-shm`, beside the socket —
    /// the phase4-ui convention: the game creates the file and
    /// the engine maps it). The file is removed on exit.
    pub ui_shm_path: Option<PathBuf>,
    /// Initial UI state override (S4; `--ui-start`); `None` =
    /// the scene's `ui_start` key, else gameplay.
    pub ui_start: Option<String>,
    /// World RNG seed override (S7; `--seed n`, the §3.1 CLI
    /// shape); `None` = the scene's own `seed` key.
    pub seed: Option<i64>,
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

/// Run the server on `config.socket_path`: build the world
/// once, then serve connections one at a time (the world
/// persists across them) until a client sends Shutdown or a
/// handshake fails the version check. The socket file is
/// removed on exit. Registry/scene load errors are returned
/// before binding, with messages naming what was tried.
pub fn run_server(config: &ServerConfig) -> proto::Result<()> {
    let reg = DefRegistry::load_resolved(config.defs_path.as_deref())
        .map_err(|e| proto::ProtoError::Malformed(format!("defs: {e}")))?;
    let mut scene = Scene::load_file(&config.scene_path)
        .map_err(|e| proto::ProtoError::Malformed(format!("scene: {e}")))?;
    if let Some(w) = &config.weather {
        scene.weather = w.clone();
    }
    if let Some(start) = &config.ui_start {
        scene.ui_start = Some(start.clone());
    }
    if let Some(seed) = config.seed {
        scene.seed = seed;
    }
    let mut world = World::from_scene(&reg, &scene)
        .map_err(|e| proto::ProtoError::Malformed(format!("world: {e}")))?;
    eprintln!(
        "sim-server: loaded {} defs, scene '{}' ({} blocks)",
        reg.len(),
        scene.name,
        world.blocks.len()
    );

    // The UI shm path (S4): explicit, or derived from the
    // socket path so it sits beside the socket.
    let ui_shm_path = config.ui_shm_path.clone().unwrap_or_else(|| {
        let mut p = config.socket_path.clone().into_os_string();
        p.push(".ui-shm");
        PathBuf::from(p)
    });

    let socket_path = &config.socket_path;
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    let listener = UnixListener::bind(socket_path)?;
    eprintln!("sim-server: listening on {}", socket_path.display());

    let mut result = Ok(());
    loop {
        match serve_one(&listener, &reg, &mut world, &ui_shm_path) {
            Ok(ServeEnd::Shutdown) | Ok(ServeEnd::Fatal) => break,
            Ok(ServeEnd::ClientGone) => {
                eprintln!("sim-server: waiting for the next connection");
                continue;
            }
            Err(e) => {
                // A client that vanishes mid-flush (broken
                // pipe on send, reset on read) ends its own
                // session, not the server: the accept loop
                // takes the next connection. Structural errors
                // (a corrupt frame, a bad handshake) stay fatal.
                if matches!(e, proto::ProtoError::Io(_)) {
                    eprintln!(
                        "sim-server: connection died ({e}); waiting for the next connection"
                    );
                    continue;
                }
                result = Err(e);
                break;
            }
        }
    }

    // Clean exit path for every outcome (shutdown, disconnect,
    // version mismatch): close the listener (drop) and unlink
    // the socket and the UI shm file.
    drop(listener);
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }
    if ui_shm_path.exists() {
        let _ = std::fs::remove_file(&ui_shm_path);
    }
    result
}

/// How one connection ended, from the server's point of view.
enum ServeEnd {
    /// The client sent Shutdown: exit the server.
    Shutdown,
    /// A fatal session error (the handshake version check):
    /// exit the server, cleanly.
    Fatal,
    /// The client went away (or sent nonsense): the world
    /// persists; accept the next connection.
    ClientGone,
}

fn serve_one(
    listener: &UnixListener,
    reg: &DefRegistry,
    world: &mut World,
    ui_shm_path: &Path,
) -> proto::Result<ServeEnd> {
    let (mut stream, _) = listener.accept()?;
    let mut dec = FrameDecoder::new();

    // Handshake: Hello out, HelloAck in, exact version match.
    send(&mut stream, &MessageT::Hello(Box::new(sample_hello())))?;
    let ack = match recv(&mut stream, &mut dec)? {
        Some(MessageT::HelloAck(ack)) => ack,
        Some(other) => {
            eprintln!("sim-server: expected HelloAck, got {other:?}; closing");
            return Ok(ServeEnd::ClientGone);
        }
        None => {
            eprintln!("sim-server: client disconnected before HelloAck");
            return Ok(ServeEnd::ClientGone);
        }
    };
    if let Err(e) = check_handshake(ack.protocol_version) {
        // Version mismatch: log both versions and exit cleanly —
        // no TileDefs, no panic (docs/protocol.md handshake rule).
        eprintln!(
            "sim-server: protocol mismatch: ours={} theirs={} ({e}); closing",
            PROTOCOL_VERSION, ack.protocol_version
        );
        return Ok(ServeEnd::Fatal);
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
    // S4: attach the UI shm file at the HelloAck's screen size
    // (the sim's default when the client reports 0) and send the
    // initial UiFrame — the real game sends a UiFrame at its
    // draw points, and the engine needs one to map the store.
    let (ui_w, ui_h) = if ack.screen_width > 0 && ack.screen_height > 0 {
        (ack.screen_width, ack.screen_height)
    } else {
        DEFAULT_UI_SIZE
    };
    let frame = world
        .ui_attach_file(reg, ui_shm_path.to_path_buf(), ui_w, ui_h)
        .map_err(proto::ProtoError::Malformed)?;
    send(&mut stream, &MessageT::UiFrame(Box::new(frame)))?;
    eprintln!(
        "sim-server: initial UiFrame sent (shm {}, {}x{})",
        ui_shm_path.display(),
        ui_w,
        ui_h
    );
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
                return Ok(ServeEnd::ClientGone);
            }
            Err(e) => {
                eprintln!("sim-server: receive error ({e}); closing");
                return Ok(ServeEnd::ClientGone);
            }
        };
        match msg {
            MessageT::Input(input) => {
                eprintln!(
                    "sim-server: input kind={:?} key={} pressed={}",
                    input.kind, input.key_code, input.pressed
                );
                // One pressed Key = one turn (S2 rules); the
                // turn's flush (Delta/Snapshot + any LightUpdate
                // / EnvironmentUpdate, S3) is the only reply.
                // Releases, non-Key inputs and Open/Close arming
                // keys produce no messages.
                for msg in world.apply_input(reg, &input) {
                    send(&mut stream, &msg)?;
                }
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
                // S4: recreate the UI store at the new geometry;
                // the reply is one UiFrame with the new
                // size/stride and a full-frame dirty rect.
                for msg in world.apply_resize(reg, r.width, r.height) {
                    send(&mut stream, &msg)?;
                }
                eprintln!("sim-server: resize {}x{} dpi={}", r.width, r.height, r.dpi);
            }
            MessageT::PickResult(p) => {
                // S4: recorded on the world (a pick is a query
                // answer — it never mutates the world).
                world.apply_pick_result(&p);
                eprintln!("sim-server: pick hit={} pos={:?} (recorded)", p.hit, p.pos);
            }
            MessageT::Shutdown(s) => {
                eprintln!("sim-server: shutdown reason={:?}", s.reason);
                return Ok(ServeEnd::Shutdown);
            }
            other => {
                eprintln!("sim-server: unexpected message {other:?}; ignoring");
            }
        }
    }
}
