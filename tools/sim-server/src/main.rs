//! sim-server: run the Phase 11 test game on a Unix socket.
//!
//! Usage: sim-server <unix-socket-path>
//!
//! The voxel engine (or `proto-client`) connects as the IPC client,
//! exactly as it will to the real fork with
//! `--frontend=ipc --ipc-path=<path>`.

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: sim-server <unix-socket-path>");
            return ExitCode::from(2);
        }
    };
    match sim::run_server(Path::new(&path)) {
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
