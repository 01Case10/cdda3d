//! Install/upgrade bookkeeping (gates 8.11/8.12): a package
//! directory carries a **stamp** — frontend version, fork commit,
//! protocol version — and the generated-asset cache is valid only
//! while the stamp matches the running build. An upgrade changes
//! the stamp, so stale assets regenerate instead of being silently
//! reused; an unchanged install keeps its cache.
//!
//! The other half of an upgrade is the save: game state reaches
//! the frontend as snapshots, and a snapshot from the old build
//! decodes under the new one because the schema only ever grows
//! additively (gate 1.9 proves old bytes decode; the store-level
//! resync is gated in this crate's tests).

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageStamp {
    pub frontend_version: String,
    pub fork_commit: String,
    pub protocol_version: u32,
}

impl PackageStamp {
    /// The stamp of the running build.
    pub fn current() -> Self {
        Self {
            frontend_version: crate::report::FRONTEND_VERSION.to_string(),
            fork_commit: crate::report::FORK_COMMIT.to_string(),
            protocol_version: proto::PROTOCOL_VERSION,
        }
    }

    pub fn to_text(&self) -> String {
        format!(
            "frontend={}\nfork={}\nprotocol={}\n",
            self.frontend_version, self.fork_commit, self.protocol_version
        )
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut stamp = Self {
            frontend_version: String::new(),
            fork_commit: String::new(),
            protocol_version: 0,
        };
        let mut seen = 0;
        for line in text.lines() {
            let (k, v) = line.split_once('=')?;
            match k {
                "frontend" => stamp.frontend_version = v.to_string(),
                "fork" => stamp.fork_commit = v.to_string(),
                "protocol" => stamp.protocol_version = v.parse().ok()?,
                _ => return None,
            }
            seen += 1;
        }
        (seen == 3).then_some(stamp)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheVerdict {
    /// Stamp matches: generated assets may be reused.
    Fresh,
    /// Anything else: regenerate. The string says why.
    Stale(String),
}

const STAMP_FILE: &str = "STAMP";

/// Record `stamp` in the asset-cache directory (after generation).
pub fn write_stamp(cache_dir: &Path, stamp: &PackageStamp) -> std::io::Result<()> {
    std::fs::create_dir_all(cache_dir)?;
    std::fs::write(cache_dir.join(STAMP_FILE), stamp.to_text())
}

/// Decide whether the cache in `cache_dir` may be reused by a
/// build carrying `stamp`.
pub fn check_asset_cache(cache_dir: &Path, stamp: &PackageStamp) -> CacheVerdict {
    let path = cache_dir.join(STAMP_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return CacheVerdict::Stale("no stamp (first run or pre-stamp cache)".into()),
    };
    match PackageStamp::parse(&text) {
        None => CacheVerdict::Stale("unparseable stamp".into()),
        Some(old) if old == *stamp => CacheVerdict::Fresh,
        Some(old) => CacheVerdict::Stale(format!(
            "build changed (was frontend {} / fork {} / protocol {}, now frontend {} / fork {} / protocol {})",
            old.frontend_version,
            &old.fork_commit[..old.fork_commit.len().min(8)],
            old.protocol_version,
            stamp.frontend_version,
            &stamp.fork_commit[..stamp.fork_commit.len().min(8)],
            stamp.protocol_version
        )),
    }
}
