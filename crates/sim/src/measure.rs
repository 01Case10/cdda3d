//! S1 measurement: the Option C handshake cost, measured not
//! assumed (plan §7 S1 gate). Numbers produced:
//! - registry load time (defs.json parse + index + TileDefs build),
//! - the TileDefs frame's byte size on the wire (u32 prefix +
//!   FlatBuffers payload),
//! - connect -> TileDefs fully received + decoded time, measured
//!   over a real loopback socket against the actual server.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use proto::samples::sample_hello_ack;
use proto::{decode_message, encode_message, FrameDecoder, FrameEncoder, MessageT};

use crate::defs::DefRegistry;
use crate::server::{run_server, ServerConfig};
use crate::world::World;
use crate::Scene;

#[derive(Debug, Clone)]
pub struct Measurement {
    pub registry_load: Duration,
    pub def_count: usize,
    pub tiledefs_frame_bytes: usize,
    pub scene_snapshot_build: Duration,
    pub connect_to_tiledefs: Duration,
}

/// Run the full measurement against a real server on a temp
/// socket (the server thread serves one client — this one — and
/// exits when the client sends Shutdown).
pub fn measure(defs_path: Option<&Path>, scene_path: &Path) -> Result<Measurement, String> {
    let t0 = Instant::now();
    let reg = DefRegistry::load_resolved(defs_path)?;
    let registry_load = t0.elapsed();

    let payload = encode_message(&MessageT::TileDefs(Box::new(reg.tiledefs().clone())));
    let tiledefs_frame_bytes = FrameEncoder::encode(&payload).len();

    let scene = Scene::load_file(scene_path)?;
    let mut world = World::from_scene(&reg, &scene)?;
    let t1 = Instant::now();
    let _snap = world.snapshot(&reg);
    let scene_snapshot_build = t1.elapsed();

    // Loopback: spawn the real server, connect, handshake, and
    // time until the TileDefs message is fully decoded.
    let sock = std::env::temp_dir().join(format!("sim-measure-{}.sock", std::process::id()));
    let _ = std::fs::remove_file(&sock);
    let cfg = ServerConfig {
        socket_path: sock.clone(),
        defs_path: defs_path.map(Path::to_path_buf),
        scene_path: scene_path.to_path_buf(),
        weather: None,
    };
    let server = std::thread::spawn(move || run_server(&cfg));
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut stream = loop {
        match UnixStream::connect(&sock) {
            Ok(s) => break s,
            Err(e) => {
                if Instant::now() > deadline {
                    return Err(format!("measure: server never listened: {e}"));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| e.to_string())?;

    let mut dec = FrameDecoder::new();
    let recv = |stream: &mut UnixStream, dec: &mut FrameDecoder| -> Result<MessageT, String> {
        loop {
            if let Some(frame) = dec.next_frame().map_err(|e| e.to_string())? {
                return decode_message(&frame).map_err(|e| e.to_string());
            }
            let mut buf = [0u8; 65536];
            let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("measure: server closed early".into());
            }
            dec.push(&buf[..n]);
        }
    };

    let t_connect = Instant::now();
    match recv(&mut stream, &mut dec)? {
        MessageT::Hello(_) => {}
        other => return Err(format!("measure: expected Hello, got {other:?}")),
    }
    let ack = MessageT::HelloAck(Box::new(sample_hello_ack()));
    stream
        .write_all(&FrameEncoder::encode(&encode_message(&ack)))
        .map_err(|e| e.to_string())?;
    match recv(&mut stream, &mut dec)? {
        MessageT::TileDefs(td) => {
            let n = td.defs.map_or(0, |d| d.len());
            if n != reg.len() {
                return Err(format!(
                    "measure: server sent {n} defs, registry has {}",
                    reg.len()
                ));
            }
        }
        other => return Err(format!("measure: expected TileDefs, got {other:?}")),
    }
    let connect_to_tiledefs = t_connect.elapsed();

    // Drain the Snapshot, then shut the server down cleanly.
    match recv(&mut stream, &mut dec)? {
        MessageT::Snapshot(_) => {}
        other => return Err(format!("measure: expected Snapshot, got {other:?}")),
    }
    let mut shutdown = proto::ShutdownT::default();
    shutdown.reason = Some("measure done".into());
    stream
        .write_all(&FrameEncoder::encode(&encode_message(&MessageT::Shutdown(
            Box::new(shutdown),
        ))))
        .map_err(|e| e.to_string())?;
    drop(stream);
    match server.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(format!("measure: server error: {e}")),
        Err(_) => return Err("measure: server thread panicked".into()),
    }

    Ok(Measurement {
        registry_load,
        def_count: reg.len(),
        tiledefs_frame_bytes,
        scene_snapshot_build,
        connect_to_tiledefs,
    })
}

pub fn temp_defs_hint() -> PathBuf {
    PathBuf::from(crate::defs::DEFAULT_DEFS_REL)
}
