//! sim-server: run the Phase 11 test game on a Unix socket.
//!
//! Usage:
//!   sim-server <unix-socket-path> [--scene <name>] [--defs <path>]
//!   sim-server --measure [--scene <name>] [--defs <path>]
//!
//! The voxel engine (or `proto-client`) connects as the IPC client,
//! exactly as it will to the real fork with
//! `--frontend=ipc --ipc-path=<path>`.
//!
//! Defaults: scene `stub_house` (see `sim::scenes::resolve_scene`
//! for the search order); defs via the registry resolution order
//! (explicit `--defs`, then `$CDDA_SIM_DEFS`, then
//! `tools/sim-defs/generated/defs.json` relative to CWD).
//!
//! `--measure` prints the S1 handshake-cost numbers (registry load
//! time, TileDefs frame bytes, connect -> TileDefs decoded time)
//! measured over a loopback socket, and exits.

use std::path::PathBuf;
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!("usage: sim-server <unix-socket-path> [--scene <name>] [--defs <path>]");
    eprintln!("       sim-server --measure [--scene <name>] [--defs <path>]");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut socket: Option<PathBuf> = None;
    let mut scene = "stub_house".to_string();
    let mut defs: Option<PathBuf> = None;
    let mut measure = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scene" => match args.next() {
                Some(v) => scene = v,
                None => return usage(),
            },
            "--defs" => match args.next() {
                Some(v) => defs = Some(PathBuf::from(v)),
                None => return usage(),
            },
            "--measure" => measure = true,
            _ if arg.starts_with("--") => return usage(),
            _ => {
                if socket.is_some() {
                    return usage();
                }
                socket = Some(PathBuf::from(arg));
            }
        }
    }

    let scene_path = match sim::scenes::resolve_scene(&scene) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("sim-server: {e}");
            return ExitCode::FAILURE;
        }
    };

    if measure {
        return match sim::measure::measure(defs.as_deref(), &scene_path) {
            Ok(m) => {
                println!("defs: {}", m.def_count);
                println!("registry load: {:?}", m.registry_load);
                println!("tiledefs frame bytes: {}", m.tiledefs_frame_bytes);
                println!("scene snapshot build: {:?}", m.scene_snapshot_build);
                println!("connect -> tiledefs decoded: {:?}", m.connect_to_tiledefs);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("sim-server --measure: {e}");
                ExitCode::FAILURE
            }
        };
    }

    let socket = match socket {
        Some(s) => s,
        None => return usage(),
    };
    let config = sim::ServerConfig {
        socket_path: socket,
        defs_path: defs,
        scene_path,
    };
    match sim::run_server(&config) {
        Ok(()) => {
            eprintln!("sim-server: clean exit");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("sim-server: error: {e}");
            ExitCode::FAILURE
        }
    }
}
