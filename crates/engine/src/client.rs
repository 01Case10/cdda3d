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
//! Seating (Phase 11 follow-up, Lane A): the session maintains
//! the **live wire world** — the connect-time Snapshot with
//! every accepted Delta folded in ([`LiveWorld`]) — and can
//! publish it as a [`WireScene`] (`CaptureArgs::wire`) that
//! `frame::render_frame` renders like any fixture scene. Three
//! consumers ride that one seating:
//! - the headless `--connect` session itself, which can
//!   capture frames from the live world at the first snapshot
//!   and at session end (`ConnectCaptures` — the gate's
//!   mid-session proof);
//! - the windowed app (`--connect-window`, see `main.rs`),
//!   which polls the published scene per frame and forwards
//!   key presses back as Inputs;
//! - any embedding caller of [`run_connect_with`].
//!
//! Input is stdin by default (each byte is one key press;
//! `Esc` byte = the Escape key, `Q` = end the session cleanly,
//! stdin EOF likewise) or an mpsc channel for embedders.

use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(windows)]
use uds_windows::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use mesher::scale::WorldScale;
use proto::samples::sample_hello_ack;
use proto::sync::{DeltaDecision, HeartbeatTracker, SeqTracker};
use proto::{
    decode_message, encode_message, FrameDecoder, FrameEncoder, InputKind, InputT, MessageT, PingT,
    ShutdownT, SnapshotT, TileDefsT,
};

use crate::capture::{CaptureArgs, CaptureError};
use crate::wire::{apply_delta_to_snapshot, WireScene};

fn now_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

/// The session's live wire state: the TileDefs from the
/// handshake and the current Snapshot — the connect-time one
/// with every accepted Delta folded in via
/// [`apply_delta_to_snapshot`]. This is the state the frame
/// loop seats: [`LiveWorld::wire_scene`] builds the renderable
/// world from it on demand, so a capture or a windowed frame
/// always shows the delta-fed world, never a stale snapshot.
#[derive(Default)]
pub struct LiveWorld {
    defs: Option<TileDefsT>,
    snapshot: Option<SnapshotT>,
    player_pos: Option<[i32; 3]>,
    /// The voxelizer furniture model table, when the session
    /// was configured with one (Lane G): wire scenes built
    /// from the live state carry the snapshot-derived prop
    /// scene.
    props_table: Option<Arc<crate::materials::MaterialTable>>,
}

impl LiveWorld {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seat (or clear) the furniture model table used for
    /// prop-scene derivation in [`LiveWorld::wire_scene`].
    pub fn set_props_table(&mut self, table: Option<Arc<crate::materials::MaterialTable>>) {
        self.props_table = table;
    }

    pub fn player_pos(&self) -> Option<[i32; 3]> {
        self.player_pos
    }

    pub fn snapshot(&self) -> Option<&SnapshotT> {
        self.snapshot.as_ref()
    }

    pub fn defs(&self) -> Option<&TileDefsT> {
        self.defs.as_ref()
    }

    fn note_defs(&mut self, defs: &TileDefsT) {
        self.defs = Some(defs.clone());
    }

    fn note_entities(&mut self, entities: &[proto::EntityT]) {
        for e in entities {
            if e.sprite_id.as_deref() == Some("player") {
                if let Some(p) = &e.pos {
                    self.player_pos = Some([p.x, p.y, p.z]);
                }
            }
        }
    }

    fn note_snapshot(&mut self, snap: &SnapshotT) {
        self.snapshot = Some(snap.clone());
        if let Some(entities) = &snap.entities {
            self.note_entities(entities);
        }
    }

    fn note_delta(&mut self, delta: &proto::DeltaT) {
        if let Some(snap) = &mut self.snapshot {
            apply_delta_to_snapshot(snap, delta);
        }
        if let Some(entities) = &delta.entity_updates {
            self.note_entities(entities);
        }
    }

    fn note_environment(&mut self, env: &proto::EnvironmentUpdateT) {
        if let Some(snap) = &mut self.snapshot {
            snap.environment = Some(Box::new(env.clone()));
        }
    }

    /// The current wire environment sample (Lane H): the
    /// held snapshot's piggybacked `EnvironmentUpdate` — the
    /// connect-time one with every accepted Delta's
    /// environment folded over it — converted by the lighting
    /// proto adapter. `None` before the first Snapshot, or
    /// when the snapshot carries no environment.
    pub fn environment(&self) -> Option<lighting::GameEnvironment> {
        self.snapshot
            .as_ref()
            .and_then(lighting::proto_adapter::game_environment_from_snapshot)
    }

    /// The renderable wire scene for the current live state
    /// (blocks coloured by [`crate::wire::palette_resolver`]
    /// — Lane G's string-id palette, semantic CDDA colours
    /// since Lane J with the hash as fallback; before Lane G,
    /// every def rendered the same grey), or `None` before
    /// the first Snapshot.
    ///
    /// Lane H: the scene also carries the GI companions
    /// derived from the same snapshot — the environment
    /// sample and the lighting scene
    /// (`LightingScene::from_snapshot`: the game's lit/outside
    /// grids + wire-derived sky exposure) — so its consumers
    /// seat Phase 10 sky + GI from the wire data itself.
    pub fn wire_scene(&self) -> Option<WireScene> {
        let (defs, snap) = (self.defs.as_ref()?, self.snapshot.as_ref()?);
        let scene = WireScene::from_wire(defs, snap, &crate::wire::palette_resolver(defs));
        let lighting = crate::frame::LightingScene::from_snapshot(snap, scene.min_game_z);
        let scene = scene.with_companions(self.environment(), Some(lighting));
        match &self.props_table {
            Some(table) => Some(scene.with_props(crate::wire::prop_scene_for_wire(defs, snap, table))),
            None => Some(scene),
        }
    }

    /// Capture args rendering the live world to `out`: the
    /// wire scene seated with its GI companions (sky +
    /// lighting, Lane H), camera on the player (or the house
    /// centre before the player is known), cutaway at the
    /// player's z.
    pub fn capture_args(&self, out: &Path, size: (u32, u32)) -> Option<CaptureArgs> {
        let wire = self.wire_scene()?;
        let mut args = CaptureArgs::default();
        args.snapshot = "wire".to_string();
        if let Some(lit) = &wire.lighting {
            args.lighting = Some(lit.clone());
        }
        if let Some(env) = &wire.environment {
            args.sky = Some(crate::frame::SkyScene::new(env.clone()));
        }
        args.wire = Some(wire);
        args.out = out.to_path_buf();
        args.size = size;
        if let Some(p) = self.player_pos {
            args.camera_target = (p[0] as f32 + 0.5, p[1] as f32 + 0.5);
            args.cutaway_z = p[2];
        }
        Some(args)
    }
}

/// The published live scene a windowed/embedder consumer
/// polls: the session replaces it after every applied flush.
pub type SharedWireScene = Arc<Mutex<Option<WireScene>>>;

/// Where the session's key bytes come from.
pub enum ConnectInput {
    /// The process's stdin, one byte per key press (the S7
    /// `--connect` behaviour).
    Stdin,
    /// An embedder's channel: each byte is one key press;
    /// dropping the sender ends the session (the stdin-EOF
    /// rule).
    Channel(mpsc::Receiver<u8>),
}

/// Frames to capture from the live wire world during the
/// session: `first` right after the first accepted Snapshot,
/// `end` after the input ends and the wire has drained, just
/// before Shutdown. Outcomes land in the [`ConnectReport`].
pub struct ConnectCaptures {
    pub first: Option<PathBuf>,
    pub end: Option<PathBuf>,
    /// Capture size (the headless default is 1280x720; gates
    /// use a smaller frame).
    pub size: (u32, u32),
}

impl Default for ConnectCaptures {
    fn default() -> Self {
        Self {
            first: None,
            end: None,
            size: (1280, 720),
        }
    }
}

/// Session configuration for [`run_connect_with`].
pub struct ConnectConfig {
    pub input: ConnectInput,
    /// Publish the live wire scene after every applied flush.
    pub share: Option<SharedWireScene>,
    pub captures: Option<ConnectCaptures>,
    /// Voxelizer output directory holding the furniture
    /// model table (Lane G): when set, the session loads it
    /// once and every published wire scene carries the
    /// snapshot-derived furniture prop scene. `None` = the
    /// coarse-only live world. A load failure is logged and
    /// the session continues without props (the protocol is
    /// the session's job; props are presentation).
    pub models_dir: Option<PathBuf>,
}

impl Default for ConnectConfig {
    fn default() -> Self {
        Self {
            input: ConnectInput::Stdin,
            share: None,
            captures: None,
            models_dir: None,
        }
    }
}

/// What a finished session did (the headless `--connect`
/// discards it; gates and embedders assert on it).
#[derive(Default)]
pub struct ConnectReport {
    pub player_pos: Option<[i32; 3]>,
    pub snapshots: usize,
    pub deltas: usize,
    pub inputs_sent: usize,
    /// Capture outcomes in order (first, then end) for the
    /// captures that were configured: `Ok(path)` = the PNG
    /// was written from the live wire world.
    pub captures: Vec<Result<PathBuf, String>>,
    /// The session's final live state (defs + delta-fed
    /// snapshot + player position) — what the last published
    /// wire scene was built from.
    pub live: LiveWorld,
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
    live: LiveWorld,
    share: Option<SharedWireScene>,
    captures: Option<ConnectCaptures>,
    capture_outcomes: Vec<Result<PathBuf, String>>,
    first_capture_done: bool,
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

    /// Publish the live wire scene to the shared slot a
    /// windowed consumer polls (no-op when not sharing).
    fn publish(&self) {
        if let Some(share) = &self.share {
            *share.lock().expect("wire share lock") = self.live.wire_scene();
        }
    }

    /// Capture one frame from the live wire world to `out`
    /// and record the outcome. A missing adapter is an
    /// outcome, not a session failure — the session's job is
    /// the protocol; the capture is evidence about it.
    fn capture_live(&mut self, out: &Path, size: (u32, u32)) {
        let outcome = match self.live.capture_args(out, size) {
            Some(args) => crate::capture::run_capture(&args)
                .map(|o| {
                    println!(
                        "connect: captured live wire world -> {} ({} distinct colors)",
                        o.out.display(),
                        o.distinct_colors
                    );
                    o.out
                })
                .map_err(|e: CaptureError| e.to_string()),
            None => Err("no live world to capture yet".to_string()),
        };
        self.capture_outcomes.push(outcome);
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
                        &crate::wire::palette_resolver(defs),
                    );
                    println!(
                        "connect: snapshot seq={} epoch={} -> world ({} chunks)",
                        s.seq,
                        s.epoch,
                        map.chunks.len()
                    );
                }
                self.live.note_snapshot(s);
                self.publish();
                if !self.first_capture_done {
                    self.first_capture_done = true;
                    if let Some(caps) = &self.captures {
                        if let Some(first) = caps.first.clone() {
                            let size = caps.size;
                            self.capture_live(&first, size);
                        }
                    }
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
                self.live.note_delta(d);
                self.publish();
                if let Some(entities) = &d.entity_updates {
                    self.note_player(entities);
                }
            }
            MessageT::LightUpdate(_) => {
                self.stats.light_updates += 1;
            }
            MessageT::EnvironmentUpdate(e) => {
                self.stats.env_updates += 1;
                self.live.note_environment(e);
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
    run_connect_with(socket_path, ConnectConfig::default()).map(|_| ())
}

/// [`run_connect`] with a full configuration: input source,
/// live-scene sharing, and live-world captures. Returns the
/// session report on a clean end.
pub fn run_connect_with(
    socket_path: &Path,
    config: ConnectConfig,
) -> Result<ConnectReport, String> {
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
        live: LiveWorld::new(),
        share: config.share,
        captures: config.captures,
        capture_outcomes: Vec::new(),
        first_capture_done: false,
    };
    // Lane G: load the furniture model table once, up front;
    // a failure leaves the session coarse-only (logged).
    if let Some(dir) = &config.models_dir {
        match crate::materials::MaterialTable::load(dir) {
            Ok(table) => {
                println!(
                    "connect: furniture model table loaded from {} (live props on)",
                    dir.display()
                );
                session.live.set_props_table(Some(Arc::new(table)));
            }
            Err(e) => {
                println!(
                    "connect: furniture model table at {} failed to load ({e}); live props off",
                    dir.display()
                );
            }
        }
    }

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
            session.live.note_defs(&td);
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

    // Input drive: stdin bytes on a thread (stdin reads
    // block), or the embedder's channel directly — both end
    // the session the same way when they close.
    let rx = match config.input {
        ConnectInput::Channel(rx) => rx,
        ConnectInput::Stdin => {
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
            rx
        }
    };

    let mut ping_nonce: u64 = 0;
    // When the user ends the session (Q / input close), the
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
                    // Input closed: end the session cleanly too.
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

    // The end-of-session capture: the wire has drained, so the
    // live world is the final state — capture it before
    // Shutdown, from the same live state the windowed app
    // would be showing.
    if let Some(caps) = &session.captures {
        if let Some(end) = caps.end.clone() {
            let size = caps.size;
            session.capture_live(&end, size);
        }
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
    Ok(ConnectReport {
        player_pos: session.player_pos,
        snapshots: session.stats.snapshots,
        deltas: session.stats.deltas,
        inputs_sent: session.stats.inputs_sent,
        captures: std::mem::take(&mut session.capture_outcomes),
        live: std::mem::take(&mut session.live),
    })
}

/// Spawn a session on its own thread (the windowed app's
/// shape: the session owns the socket; the app owns the key
/// channel's sender and polls the shared wire scene).
pub fn spawn_connect(
    socket_path: &Path,
    config: ConnectConfig,
) -> std::thread::JoinHandle<Result<ConnectReport, String>> {
    let path = socket_path.to_path_buf();
    std::thread::spawn(move || run_connect_with(&path, config))
}
