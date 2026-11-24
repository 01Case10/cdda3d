//! sim-server: run the Phase 11 test game on a Unix socket.
//!
//! Usage:
//!   sim-server <unix-socket-path> [--scene <name>] [--defs <path>]
//!              [--weather <preset>] [--ui-shm <path>]
//!              [--ui-start <state>] [--seed <n>]
//!   sim-server --measure [--scene <name>] [--defs <path>]
//!   sim-server --scenario <file> [--defs <path>]
//!
//! `--weather` overrides the scene's starting weather preset
//! (S3: clear/sunny/cloudy/rain/storm/lightning). Mid-run weather
//! changes are the scene's `[[weather_script]]` entries.
//!
//! `--ui-shm` sets the UI shared-memory file path (S4; default:
//! `<socket>.ui-shm` beside the socket). `--ui-start` sets the
//! initial UI state (gameplay|inventory|crafting|look|
//! main-menu), overriding the scene's `ui_start` key. `--seed`
//! overrides the scene's world RNG seed (S7; the §3.1 CLI shape).
//!
//! `--scenario <file>` (S7) runs a §5 scenario file in-process
//! against a fresh world built from the scenario's own scene,
//! prints the step log, and exits 0 (all assertions held) or 1.
//! No socket is bound in this mode.
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
    eprintln!("usage: sim-server <unix-socket-path> [--scene <name>] [--defs <path>] [--weather <preset>] [--ui-shm <path>] [--ui-start <state>] [--seed <n>]");
    eprintln!("       sim-server --measure [--scene <name>] [--defs <path>]");
    eprintln!("       sim-server --scenario <file> [--defs <path>]");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut socket: Option<PathBuf> = None;
    let mut scene = "stub_house".to_string();
    let mut defs: Option<PathBuf> = None;
    let mut weather: Option<String> = None;
    let mut ui_shm: Option<PathBuf> = None;
    let mut ui_start: Option<String> = None;
    let mut seed: Option<i64> = None;
    let mut scenario_file: Option<PathBuf> = None;
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
            "--weather" => match args.next() {
                Some(v) => weather = Some(v),
                None => return usage(),
            },
            "--ui-shm" => match args.next() {
                Some(v) => ui_shm = Some(PathBuf::from(v)),
                None => return usage(),
            },
            "--ui-start" => match args.next() {
                Some(v) => ui_start = Some(v),
                None => return usage(),
            },
            "--seed" => match args.next() {
                Some(v) => match v.parse::<i64>() {
                    Ok(n) => seed = Some(n),
                    Err(_) => return usage(),
                },
                None => return usage(),
            },
            "--scenario" => match args.next() {
                Some(v) => scenario_file = Some(PathBuf::from(v)),
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

    // S7 scenario mode: no socket, no scene flag — the file
    // names its own scene. Run in-process and report.
    if let Some(file) = &scenario_file {
        return run_scenario_mode(file, defs.as_deref());
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
        weather,
        ui_shm_path: ui_shm,
        ui_start,
        seed,
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

/// `sim-server --scenario <file>`: parse the scenario, build
/// its scene's world, run it in-process, print the step log.
fn run_scenario_mode(file: &PathBuf, defs: Option<&std::path::Path>) -> ExitCode {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("sim-server --scenario: cannot read {}: {e}", file.display());
            return ExitCode::FAILURE;
        }
    };
    let scenario = match sim::scenario::Scenario::parse(&text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sim-server --scenario: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !scenario.is_green() {
        eprintln!(
            "sim-server --scenario: '{}' carries status {:?} — it is not in the green set and does not run",
            scenario.name.as_deref().unwrap_or("<unnamed>"),
            scenario.status.as_deref().unwrap_or("")
        );
        return ExitCode::FAILURE;
    }
    let reg = match sim::DefRegistry::load_resolved(defs) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("sim-server --scenario: defs: {e}");
            return ExitCode::FAILURE;
        }
    };
    let scene_path = match sim::scenes::resolve_scene(&scenario.scene) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("sim-server --scenario: {e}");
            return ExitCode::FAILURE;
        }
    };
    let scene = match sim::Scene::load_file(&scene_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sim-server --scenario: scene: {e}");
            return ExitCode::FAILURE;
        }
    };
    match sim::scenario::run_in_process(&reg, scene, &scenario) {
        Ok(outcome) => {
            for line in &outcome.log {
                println!("{line}");
            }
            println!(
                "scenario '{}': PASS ({} steps)",
                scenario.name.as_deref().unwrap_or("<unnamed>"),
                outcome.steps_executed
            );
            ExitCode::SUCCESS
        }
        Err(failure) => {
            eprintln!("{failure}");
            ExitCode::FAILURE
        }
    }
}
