//! Gate 7.5 — light updates: given new lit/seen data, compute the
//! exact set of chunks whose baked output can change, and nothing
//! else.
//!
//! Why the set is larger than "chunks containing a changed tile":
//! smoothing (see `smoothing.rs`) reads, for a face/vertex in chunk
//! C, air-side tiles up to 1 tile across C's border. So a changed
//! tile at a chunk edge invalidates the neighbouring chunk, and a
//! changed tile *one tile inside* a neighbouring chunk can still be
//! read by vertices on C's border faces — wait, precisely: a vertex
//! on C's border reads tiles at offsets `0 / -1` in-plane from its
//! air tile, i.e. at most 1 tile across the border. A changed tile
//! therefore dirties (a) its own chunk, (b) any chunk it sits on the
//! border of (edge neighbours; both edge neighbours at a corner,
//! plus the diagonal chunk, because a corner vertex is touched by
//! the diagonal tile), and (c) the chunk across a border when the
//! tile is the *first tile inside* the neighbouring side from the
//! perspective of border vertices — case (c) is the same statement
//! as (b) seen from the other chunk: if the tile is at local
//! coordinate 0 or 11, the chunk beyond that edge reads it. Tiles
//! with local coordinate 1 or 10 are *not* read across a border
//! (vertices read at most 1 tile across), so they dirty only their
//! own chunk plus, via (b), nothing else. The functions below encode
//! exactly (a)+(b) and the gate test proves sufficiency by equality
//! with a full re-bake — if the radius were under-claimed, the
//! partial re-bake would differ from the full one and fail.

use std::collections::BTreeSet;

use mesher::chunk::{CHUNK_H, CHUNK_W};
use mesher::ChunkMap;
use proto::LitLevel;

use crate::grid::LightGrid;

/// Chunk coords dirtied by a single changed tile at `(x, y)`
/// (z does not affect chunk columns — a column groups all z).
pub fn chunks_dirtied_by_tile(x: i32, y: i32) -> BTreeSet<[i32; 2]> {
    let c = ChunkMap::chunk_coord_of(x, y);
    let lx = x.rem_euclid(CHUNK_W as i32);
    let ly = y.rem_euclid(CHUNK_H as i32);
    let mut out = BTreeSet::new();
    out.insert(c);
    let west = lx == 0;
    let east = lx == CHUNK_W as i32 - 1;
    let north = ly == 0;
    let south = ly == CHUNK_H as i32 - 1;
    if west {
        out.insert([c[0] - 1, c[1]]);
    }
    if east {
        out.insert([c[0] + 1, c[1]]);
    }
    if north {
        out.insert([c[0], c[1] - 1]);
    }
    if south {
        out.insert([c[0], c[1] + 1]);
    }
    if west && north {
        out.insert([c[0] - 1, c[1] - 1]);
    }
    if west && south {
        out.insert([c[0] - 1, c[1] + 1]);
    }
    if east && north {
        out.insert([c[0] + 1, c[1] - 1]);
    }
    if east && south {
        out.insert([c[0] + 1, c[1] + 1]);
    }
    out
}

/// Dirty set for a batch of changed tiles.
pub fn dirty_chunks_for_tiles(changed: &[[i32; 3]]) -> BTreeSet<[i32; 2]> {
    let mut out = BTreeSet::new();
    for t in changed {
        out.extend(chunks_dirtied_by_tile(t[0], t[1]));
    }
    out
}

/// A rectangular light patch in the shape of a protocol
/// `LightUpdate` / one `SubmapBlock`'s lit+seen arrays: `lit`/`seen`
/// are row-major over `size_xy` at a single z-level `origin.z`.
/// (The wire `LightUpdate` carries no explicit size; in Phase 3's
/// exporter it covers one submap block, 12x12 — callers pass that
/// size explicitly here rather than this crate guessing.)
#[derive(Debug, Clone)]
pub struct LightPatch {
    pub origin: [i32; 3],
    pub size_xy: [usize; 2],
    pub lit: Vec<LitLevel>,
    pub seen: Vec<u8>,
}

/// Apply a patch to `grid`, returning the dirty chunk set computed
/// from the tiles whose lit/seen *actually changed* (a patch that
/// re-sends unchanged values dirties nothing).
pub fn apply_patch(grid: &mut LightGrid, patch: &LightPatch) -> BTreeSet<[i32; 2]> {
    let mut changed: Vec<[i32; 3]> = Vec::new();
    let [w, h] = patch.size_xy;
    for ly in 0..h {
        for lx in 0..w {
            let i = ly * w + lx;
            let (lit, seen) = match (patch.lit.get(i), patch.seen.get(i)) {
                (Some(&l), Some(&s)) => (l, s),
                _ => continue,
            };
            let pos = [
                patch.origin[0] + lx as i32,
                patch.origin[1] + ly as i32,
                patch.origin[2],
            ];
            let old = grid.get(pos[0], pos[1], pos[2]);
            if old.lit != lit || old.seen != seen {
                let mut t = old;
                t.lit = lit;
                t.seen = seen;
                grid.set(pos[0], pos[1], pos[2], t);
                changed.push(pos);
            }
        }
    }
    dirty_chunks_for_tiles(&changed)
}

/// Apply per-tile changes in the shape of a protocol `Delta`'s
/// `TileChange` lit/seen fields: `(pos, lit, seen)` triples.
/// Returns the dirty set from actually-changed tiles.
pub fn apply_tile_changes(
    grid: &mut LightGrid,
    changes: &[([i32; 3], LitLevel, u8)],
) -> BTreeSet<[i32; 2]> {
    let mut changed = Vec::new();
    for &(pos, lit, seen) in changes {
        let old = grid.get(pos[0], pos[1], pos[2]);
        if old.lit != lit || old.seen != seen {
            let mut t = old;
            t.lit = lit;
            t.seen = seen;
            grid.set(pos[0], pos[1], pos[2], t);
            changed.push(pos);
        }
    }
    dirty_chunks_for_tiles(&changed)
}

/// Apply a wire `LightUpdateT` whose arrays cover one 12x12 submap
/// block at `origin` (the Phase 3 exporter's shape). Tint/occlusion
/// already in the grid are preserved — a light update changes light,
/// not geometry.
pub fn apply_light_update(
    grid: &mut LightGrid,
    update: &proto::LightUpdateT,
) -> BTreeSet<[i32; 2]> {
    let origin = update
        .origin
        .as_ref()
        .map(|v| [v.x, v.y, v.z])
        .unwrap_or([0, 0, 0]);
    let patch = LightPatch {
        origin,
        size_xy: [CHUNK_W, CHUNK_H],
        lit: update.lit.clone().unwrap_or_default(),
        seen: update.seen.clone().unwrap_or_default(),
    };
    // Only apply if the arrays are actually block-sized; a malformed
    // short update dirties nothing rather than half-applying.
    if patch.lit.len() != CHUNK_W * CHUNK_H || patch.seen.len() != CHUNK_W * CHUNK_H {
        return BTreeSet::new();
    }
    apply_patch(grid, &patch)
}

/// Apply a wire `DeltaT`'s tile-change lit/seen fields, plus the
/// Phase 10 `outside` flag (slice 10.6): a change to any of the
/// three marks the tile changed — an outside flip changes the
/// tile's sky-GI bake exactly as a lit change does, so it dirties
/// the same chunk set.
pub fn apply_delta(grid: &mut LightGrid, delta: &proto::DeltaT) -> BTreeSet<[i32; 2]> {
    let mut changed = Vec::new();
    if let Some(tcs) = &delta.tile_changes {
        for tc in tcs {
            if let Some(p) = &tc.pos {
                let pos = [p.x, p.y, p.z];
                let old = grid.get(pos[0], pos[1], pos[2]);
                if old.lit != tc.lit || old.seen != tc.seen || old.outside != tc.outside {
                    let mut t = old;
                    t.lit = tc.lit;
                    t.seen = tc.seen;
                    t.outside = tc.outside;
                    grid.set(pos[0], pos[1], pos[2], t);
                    changed.push(pos);
                }
            }
        }
    }
    dirty_chunks_for_tiles(&changed)
}
