//! Crash reporting (gate 8.13): when the frontend dies, the user
//! gets a plain message and a log file carrying the version facts
//! needed to reproduce — frontend version, protocol version, and
//! the pinned fork commit the build was made against (plan §1:
//! the fork is pinned to `7b2efa5c…`; a crash report that doesn't
//! name it can't be matched to game-side behavior).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The pinned CDDA fork commit (plan §1 / docs/recon.md §6).
pub const FORK_COMMIT: &str = "7b2efa5cea38e4d4d97dd0e63b28b9148623da59";

/// This frontend build's version (the release crate's version is
/// the workspace's release version).
pub const FRONTEND_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct CrashReport {
    pub when_unix: u64,
    pub message: String,
    pub location: String,
}

impl CrashReport {
    pub fn now(message: String, location: String) -> Self {
        let when_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            when_unix,
            message,
            location,
        }
    }
}

/// The report body. The first lines are the version facts, in a
/// fixed order, so a report can be eyeballed and grepped alike.
pub fn format_report(report: &CrashReport) -> String {
    format!(
        "CDDA Voxel Frontend crash report\n\
         frontend: {FRONTEND_VERSION}\n\
         protocol: {}\n\
         fork: {FORK_COMMIT}\n\
         time: {} (unix)\n\
         panic: {} at {}\n",
        proto::PROTOCOL_VERSION,
        report.when_unix,
        report.message,
        report.location
    )
}

/// Write the report into `dir` as `crash-<unix>.log`; returns the
/// path. Creates the directory if needed.
pub fn write_report(dir: &Path, report: &CrashReport) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("crash-{}.log", report.when_unix));
    let mut f = std::fs::File::create(&path)?;
    f.write_all(format_report(report).as_bytes())?;
    Ok(path)
}

/// Install a panic hook that writes a crash report into `dir` and
/// prints the user-facing message (what happened + where the
/// report went). Returns the previous hook so tests can restore
/// it.
pub fn install_panic_hook(
    dir: PathBuf,
) -> Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let report = CrashReport::now(message, location);
        match write_report(&dir, &report) {
            Ok(path) => eprintln!(
                "The voxel frontend crashed. A report was written to {} — \
                 please include it if you report the problem.",
                path.display()
            ),
            Err(e) => {
                eprintln!("The voxel frontend crashed (and writing the crash report failed: {e}).")
            }
        }
    }));
    previous
}
