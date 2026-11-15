//! The game's sprite resolution chain (plan Appendix A #20,
//! `cata_tiles::find_tile_looks_like` / `find_tile_with_season`):
//! exact → variant (`<id>_var_<variant>`) → season suffix
//! (`<id>_season_<season>`) → `looks_like` (recursive, jump-limited)
//! → filler. An id is missing only when every stage fails.

use crate::{TileEntry, Tileset};
use std::collections::BTreeMap;

pub const LOOKS_LIKE_JUMP_LIMIT: usize = 10;
pub const SEASONS: [&str; 4] = ["spring", "summer", "autumn", "winter"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Exact,
    Variant,
    Season,
    LooksLike,
    Filler,
    Missing,
}

impl Stage {
    pub fn name(&self) -> &'static str {
        match self {
            Stage::Exact => "exact",
            Stage::Variant => "variant",
            Stage::Season => "season",
            Stage::LooksLike => "looks_like",
            Stage::Filler => "filler",
            Stage::Missing => "missing",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Resolution<'a> {
    pub stage: Stage,
    /// The entry that satisfied the request (None only for Missing).
    pub entry: Option<&'a TileEntry>,
    /// The id the entry was found under.
    pub resolved_id: String,
    /// looks_like hops taken (0 unless stage == LooksLike).
    pub jumps: usize,
}

fn try_stages<'a>(
    ts: &'a Tileset,
    id: &str,
    variant: Option<&str>,
    season: &str,
) -> Option<Resolution<'a>> {
    if let Some(e) = ts.entries.get(id) {
        return Some(Resolution {
            stage: Stage::Exact,
            entry: Some(e),
            resolved_id: id.to_string(),
            jumps: 0,
        });
    }
    if let Some(v) = variant {
        if !v.is_empty() {
            let vid = format!("{id}_var_{v}");
            if let Some(e) = ts.entries.get(&vid) {
                return Some(Resolution {
                    stage: Stage::Variant,
                    entry: Some(e),
                    resolved_id: vid,
                    jumps: 0,
                });
            }
        }
    }
    // Any `<id>_var_*` entry also counts as a variant hit (the game
    // picks among defined variants); choose the first sorted name for
    // determinism.
    let prefix = format!("{id}_var_");
    if let Some((vid, e)) = ts.entries.range(prefix.clone()..).next() {
        if vid.starts_with(&prefix) {
            return Some(Resolution {
                stage: Stage::Variant,
                entry: Some(e),
                resolved_id: vid.clone(),
                jumps: 0,
            });
        }
    }
    let sid = format!("{id}_season_{season}");
    if let Some(e) = ts.entries.get(&sid) {
        return Some(Resolution {
            stage: Stage::Season,
            entry: Some(e),
            resolved_id: sid,
            jumps: 0,
        });
    }
    None
}

/// Resolve `id` through the full chain. `looks_like` maps game ids to
/// their `looks_like` string id (from game JSON / TileDefs).
pub fn resolve<'a>(
    ts: &'a Tileset,
    id: &str,
    looks_like: &BTreeMap<String, String>,
    variant: Option<&str>,
    season: &str,
) -> Resolution<'a> {
    if let Some(r) = try_stages(ts, id, variant, season) {
        return r;
    }
    // looks_like chain, recursive with the game's jump-limit spirit.
    let mut current = id.to_string();
    let mut jumps = 0usize;
    while jumps < LOOKS_LIKE_JUMP_LIMIT {
        let next = match looks_like.get(&current) {
            Some(n) if !n.is_empty() => n.clone(),
            _ => break,
        };
        jumps += 1;
        if let Some(mut r) = try_stages(ts, &next, variant, season) {
            r.stage = Stage::LooksLike;
            r.jumps = jumps;
            return r;
        }
        if next == current {
            break;
        }
        current = next;
    }
    // Filler: an entry literally named "filler", as the tileset's
    // filler sheets provide.
    if let Some(e) = ts.entries.get("filler") {
        return Resolution {
            stage: Stage::Filler,
            entry: Some(e),
            resolved_id: "filler".to_string(),
            jumps,
        };
    }
    Resolution {
        stage: Stage::Missing,
        entry: None,
        resolved_id: id.to_string(),
        jumps,
    }
}
