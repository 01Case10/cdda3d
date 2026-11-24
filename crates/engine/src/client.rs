//! Live IPC client session (Phase 11 S7): `voxel-engine
//! --connect <unix-socket>` runs a session against a running
//! game server (the sim-server, or the real fork with
//! `--frontend=ipc`): connect, handshake, ingest
//! TileDefs/Snapshot/UiFrame + the UI shm store, then a live
//! loop — keyboard input becomes Input messages, Deltas and
//! Snapshots are ingested as they arrive (seq/epoch discipline
//! via `proto::sync`), Ping/Pong heartbeat per the protocol
//! (1 s interval, 3 s silence = dead), clean Shutdown at the
//! end.
//!
//! This is the session/ingest half of the live loop, headless:
//! it builds the voxel world from each Snapshot through the
//! same `snapshot_world` path the gates use and validates UI
//! frames through `frame::prepare_ui`, and it prints the
//! session state (player position, UI mode, environment) as it
//! changes. It does NOT open a window: seating this session in
//! the windowed frame loop needs the capture/frame plumbing
//! to accept a wire-fed world (the standing CaptureArgs
//! limitation — captures take fixture names), which is the
//! named remaining step for the windowed run on a desktop
//! machine. Input here is the process's stdin: each byte is
//! one key press (`Esc` byte = the Escape key, `Q` = end the
//! session cleanly, stdin EOF likewise).

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use mesher::scale::WorldScale;
use mesher::{Block, Pass};
use proto::samples::sample_hello_ack;
use proto::sync::{DeltaDecision, HeartbeatTracker, SeqTracker};
use proto::{
    decode_message, encode_message, FrameDecoder, FrameEncoder, InputKind, InputT, MessageT, PingT,
    ShutdownT, TileDefsT,
};

/// One coarse block per def id (the gates' stand-in mapping;
/// the windowed app resolves real models/textures instead).
fn coarse_block(id: u32) -> Block {
    Block {
        id,
        color: [0.5, 0.5, 0.5, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

fn now_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

/// Session counters, printed as the summary at the end.
#[derive(Default)]
struct Stats {
    snapshots: usize,
    deltas: usize,
    light_updates: usize,
    env_updates: usize,
    effects: usize,
    ui_frames: usize,
    pongs: usize,
    inputs_sent: usize,
}

struct Session {
    stream: UnixStream,
    dec: FrameDecoder,
    seqs: SeqTracker,
    heartbeat: HeartbeatTracker,
    start: Instant,
    defs: Option<TileDefsT>,
    ui_store: Option<ui_share::UiShmStore>,
    player_pos: Option<[i32; 3]>,
    stats: Stats,
}

impl Session {
    fn send(&mut self, msg: &MessageT) -> Result<(), String> {
        let framed = FrameEncoder::encode(&encode_message(msg));
        self.stream
            .write_all(&framed)
            .map_err(|e| format!("client send: {e}"))
    }

    /// Read one message if a complete frame is buffered or
    /// arrives within `wait`. Ok(None) = nothing complete yet.
    fn poll_message(&mut self, wait: Duration) -> Result<Option<MessageT>, String> {
        if let Some(frame) = self.dec.next_frame().map_err(|e| format!("frame: {e}"))? {
            return Ok(Some(
                decode_message(&frame).map_err(|e| format!("decode: {e}"))?,
            ));
        }
        self.stream
            .set_read_timeout(Some(wait))
            .map_err(|e| format!("set timeout: {e}"))?;
        let mut buf = [0u8; 16384];
        match self.stream.read(&mut buf) {
            Ok(0) => Err("server closed the connection".to_string()),
            Ok(n) => {
                self.dec.push(&buf[..n]);
                match self.dec.next_frame().map_err(|e| format!("frame: {e}"))? {
                    Some(frame) => Ok(Some(
                        decode_message(&frame).map_err(|e| format!("decode: {e}"))?,
                    )),
                    None => Ok(None),
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                Ok(None)
            }
            Err(e) => Err(format!("client read: {e}")),
        }
    }

    fn expect(&mut self, what: &str) -> Result<MessageT, String> {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(msg) = self.poll_message(Duration::from_millis(100))? {
                return Ok(msg);
            }
            if Instant::now() >= deadline {
                return Err(format!("timed out waiting for {what}"));
            }
        }
    }

    fn note_player(&mut self, entities: &[proto::EntityT]) {
        for e in entities {
            if e.sprite_id.as_deref() == Some("player") {
                if let Some(p) = &e.pos {
                    let pos = [p.x, p.y, p.z];
                    if self.player_pos != Some(pos) {
                        self.player_pos = Some(pos);
                        println!("connect: player at {pos:?}");
                    }
                }
            }
        }
    }

    fn ingest(&mut self, msg: &MessageT) -> Result<(), String> {
        self.heartbeat.note_seen(now_ms(self.start));
        match msg {
            MessageT::Snapshot(s) => {
                self.stats.snapshots += 1;
                if !self.seqs.accept_snapshot(s.seq, s.epoch) {
                    println!(
                        "connect: snapshot seq={} epoch={} rejected by the seq tracker (stale)",
                        s.seq, s.epoch
                    );
                    return Ok(());
                }
                if let Some(defs) = &self.defs {
                    let map = crate::snapshot_world::build_snapshot_world(
                        defs,
                        s,
                        WorldScale::default(),
                        &coarse_block,
                    );
                    println!(
                        "connect: snapshot seq={} epoch={} -> world ({} chunks)",
                        s.seq,
                        s.epoch,
                        map.chunks.len()
                    );
                }
                if let Some(entities) = &s.entities {
                    self.note_player(entities);
                }
                if let Some(env) = &s.environment {
                    println!(
                        "connect: environment turn={} weather={:?}",
                        env.turn,
                        env.weather_id.as_deref().unwrap_or("?")
                    );
                }
            }
            MessageT::Delta(d) => {
                self.stats.deltas += 1;
                match self.seqs.observe_delta(d.seq, d.epoch) {
                    DeltaDecision::Apply => {}
                    DeltaDecision::Discard => {
                        println!("connect: delta seq={} discarded (duplicate/stale)", d.seq);
                        return Ok(());
                    }
                    DeltaDecision::NeedsSnapshot => {
                        // The protocol has no client resync
                        // request; the server sends a fresh
                        // Snapshot on the next map shift. The
                        // session logs the gap and carries on —
                        // the same posture the protocol doc
                        // specifies for a dropped connection.
                        println!(
                            "connect: delta seq={} epoch={} skips a seq — awaiting a fresh snapshot",
                            d.seq, d.epoch
                        );
                        return Ok(());
                    }
                }
                if let Some(entities) = &d.entity_updates {
                    self.note_player(entities);
                }
            }
            MessageT::LightUpdate(_) => {
                self.stats.light_updates += 1;
            }
            MessageT::EnvironmentUpdate(e) => {
                self.stats.env_updates += 1;
                println!(
                    "connect: environment turn={} weather={:?} sun_alt={:.1}",
                    e.turn,
                    e.weather_id.as_deref().unwrap_or("?"),
                    e.sun_altitude_deg
                );
            }
            MessageT::Effects(fx) => {
                self.stats.effects += fx.events.as_ref().map_or(0, |v| v.len());
            }
            MessageT::UiFrame(note) => {
                self.stats.ui_frames += 1;
                if let Some(store) = &self.ui_store {
                    match crate::frame::prepare_ui(store, note) {
                        Ok(prepared) => println!(
                            "connect: ui frame seq={} mode={:?} ({} tight pixels)",
                            note.seq,
                            note.mode,
                            prepared.tight.len()
                        ),
                        Err(e) => {
                            println!("connect: ui frame seq={} failed validation: {e}", note.seq)
                        }
                    }
                }
            }
            MessageT::Ping(p) => {
                let mut pong = proto::PongT::default();
                pong.nonce = p.nonce;
                pong.sent_ms = p.sent_ms;
                self.send(&MessageT::Pong(Box::new(pong)))?;
            }
            MessageT::Pong(_) => {
                self.stats.pongs += 1;
            }
            MessageT::Shutdown(_) => {
                return Err("server sent Shutdown".to_string());
            }
            other => {
                println!("connect: ignoring {other:?}");
            }
        }
        Ok(())
    }
}

/// Run the `--connect` session until `Q`, stdin EOF, a dead
/// heartbeat, or a protocol failure. Returns Ok on a clean
/// session end (Shutdown sent and acknowledged by exit).
pub fn run_connect(socket_path: &Path) -> Result<(), String> {
    let start = Instant::now();
    // Poll-connect: the server may still be starting.
    let stream = {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match UnixStream::connect(socket_path) {
                Ok(s) => break s,
                Err(e) => {
                    if Instant::now() >= deadline {
                        return Err(format!("cannot connect to {}: {e}", socket_path.display()));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    };
    let mut session = Session {
        stream,
        dec: FrameDecoder::new(),
        seqs: SeqTracker::new(),
        heartbeat: HeartbeatTracker::new(0),
        start,
        defs: None,
        ui_store: None,
        player_pos: None,
        stats: Stats::default(),
    };

    // Handshake: Hello, HelloAck, TileDefs, Snapshot, UiFrame.
    match session.expect("Hello")? {
        MessageT::Hello(h) => {
            if h.protocol_version != proto::PROTOCOL_VERSION {
                return Err(format!(
                    "server speaks protocol {}, engine speaks {}",
                    h.protocol_version,
                    proto::PROTOCOL_VERSION
                ));
            }
            println!(
                "connect: hello from game {:?} tileset {:?} (protocol {})",
                h.game_version.as_deref().unwrap_or("?"),
                h.tileset_id.as_deref().unwrap_or("?"),
                h.protocol_version
            );
        }
        other => return Err(format!("expected Hello, got {other:?}")),
    }
    session.send(&MessageT::HelloAck(Box::new(sample_hello_ack())))?;
    match session.expect("TileDefs")? {
        MessageT::TileDefs(td) => {
            println!(
                "connect: tiledefs ({} defs)",
                td.defs.as_ref().map_or(0, |d| d.len())
            );
            session.defs = Some((*td).clone());
        }
        other => return Err(format!("expected TileDefs, got {other:?}")),
    }
    let snap = session.expect("Snapshot")?;
    session.ingest(&snap)?;
    match session.expect("UiFrame")? {
        MessageT::UiFrame(note) => {
            // The shm path is agreed alongside the handshake
            // by convention (docs/phase4-ui.md): the server's
            // store sits beside its socket at
            // `<socket>.ui-shm` unless configured otherwise;
            // the UiFrame itself carries geometry/seq/mode.
            let mut shm_path = socket_path.as_os_str().to_os_string();
            shm_path.push(".ui-shm");
            match ui_share::UiShmStore::open_file(Path::new(&shm_path)) {
                Ok(store) => {
                    session.ui_store = Some(store);
                    println!("connect: ui shm attached ({})", shm_path.to_string_lossy());
                }
                Err(e) => {
                    println!(
                        "connect: ui shm open failed ({}): {e}",
                        shm_path.to_string_lossy()
                    );
                }
            }
            session.ingest(&MessageT::UiFrame(note))?;
        }
        other => return Err(format!("expected UiFrame, got {other:?}")),
    }

    // Stdin drive: bytes on a thread (stdin reads block).
    let (tx, rx) = mpsc::channel::<u8>();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let mut buf = [0u8; 1];
        loop {
            match lock.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if tx.send(buf[0]).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut ping_nonce: u64 = 0;
    // When the user ends the session (Q / stdin EOF), the
    // client first drains: turn replies to the inputs already
    // sent are still in flight, and a Shutdown that overtakes
    // them would cut the server's flush off mid-write. Drain
    // until the wire has been quiet for 300 ms (5 s cap), then
    // shut down.
    let mut draining_since: Option<Instant> = None;
    let mut last_msg_at = Instant::now();
    loop {
        // Socket first: ingest everything that has arrived.
        while let Some(msg) = session.poll_message(Duration::from_millis(2))? {
            last_msg_at = Instant::now();
            session.ingest(&msg)?;
        }
        if let Some(since) = draining_since {
            let quiet_since = last_msg_at.max(since);
            if quiet_since.elapsed() > Duration::from_millis(400)
                || since.elapsed() > Duration::from_secs(5)
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        // Then keyboard bytes.
        let mut end_session = false;
        loop {
            match rx.try_recv() {
                Ok(b'Q') => {
                    end_session = true;
                    break;
                }
                Ok(byte) => {
                    let mut input = InputT::default();
                    input.kind = InputKind::Key;
                    input.key_code = if byte == 0x1b {
                        ui_share::input::KEY_ESCAPE
                    } else {
                        byte as i32
                    };
                    input.pressed = true;
                    session.send(&MessageT::Input(Box::new(input)))?;
                    session.stats.inputs_sent += 1;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Stdin EOF: end the session cleanly too.
                    end_session = true;
                    break;
                }
            }
        }
        if end_session {
            draining_since = Some(Instant::now());
        }
        // Heartbeat: ping on the protocol's cadence; silence
        // past the timeout means a dead server.
        let now = now_ms(session.start);
        if session.heartbeat.is_timed_out(now) {
            return Err(format!(
                "server silent for over {} ms — treating the session as dead",
                3000
            ));
        }
        if session.heartbeat.should_ping(now) {
            ping_nonce += 1;
            let mut ping = PingT::default();
            ping.nonce = ping_nonce;
            ping.sent_ms = now;
            session.send(&MessageT::Ping(Box::new(ping)))?;
            session.heartbeat.note_ping_sent(now);
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    // Clean shutdown: tell the server, give its final messages
    // a moment, and report the session.
    let mut shutdown = ShutdownT::default();
    shutdown.reason = Some("engine --connect session ended".to_string());
    session.send(&MessageT::Shutdown(Box::new(shutdown)))?;
    println!(
        "connect: session ended — {} snapshots, {} deltas, {} light updates, {} env updates, \
         {} effects, {} ui frames, {} pongs, {} inputs sent; player at {:?}",
        session.stats.snapshots,
        session.stats.deltas,
        session.stats.light_updates,
        session.stats.env_updates,
        session.stats.effects,
        session.stats.ui_frames,
        session.stats.pongs,
        session.stats.inputs_sent,
        session.player_pos
    );
    Ok(())
}
