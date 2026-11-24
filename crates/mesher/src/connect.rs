//! Cardinal connection masks and the mask -> subtile mapping
//! (Phase 2d). This is the one canonical copy of the game's
//! multitile rule (`cata_tiles::get_tile_values` +
//! `get_rotation_and_subtile`, verified in Phase 5): a tile's four
//! cardinal neighbors decide which of the six neighbour subtiles —
//! center, corner, edge, t_connection, end_piece, unconnected —
//! its sprite comes from, plus a rotation variant index.
//!
//! The mesher crate hosts the table because it is the workspace's
//! leaf crate: the voxelizer (Phase 5 data), the tree builder
//! (Phase 2d RT selection) and the engine all build on it, so no
//! second copy of the rule can drift. `voxelizer::multitile`
//! delegates here; its group-membership helper
//! (`neighbors_connect`) stays there because it speaks in game ids
//! and exported connect-group indices, not masks.
//!
//! Phase 2d selects the subtile *layer* only. The rotation half of
//! the mapping is carried in the return value for completeness (and
//! pinned by tests) but not applied to UVs — rotation variants are
//! Phase 8 polish per the Phase 2d plan section.

/// Neighbor bits in a connection mask: N=1, E=2, S=4, W=8.
/// Grid space: north is -y, south is +y, east is +x, west is -x.
pub const MASK_N: u8 = 1;
pub const MASK_E: u8 = 2;
pub const MASK_S: u8 = 4;
pub const MASK_W: u8 = 8;

/// Build a neighbor mask from four connect tests (N, E, S, W order).
pub fn mask_from_connects(n: bool, e: bool, s: bool, w: bool) -> u8 {
    (n as u8) | ((e as u8) << 1) | ((s as u8) << 2) | ((w as u8) << 3)
}

/// The subtile + rotation for a cardinal connection mask.
/// `open`/`broken` are state-variant ids in the game, NOT neighbor
/// variants, and are deliberately absent here.
pub fn subtile_for_mask(mask: u8) -> (&'static str, u32) {
    let n = mask & MASK_N != 0;
    let e = mask & MASK_E != 0;
    let s = mask & MASK_S != 0;
    let w = mask & MASK_W != 0;
    match mask.count_ones() {
        0 => ("unconnected", 0),
        4 => ("center", 0),
        3 => {
            // t_connection: rotation points at the missing direction.
            // Variant order in tileset entries: [n, e, s, w] = the
            // direction the T opens toward (the missing neighbor).
            let rot = if !n {
                0
            } else if !e {
                1
            } else if !s {
                2
            } else {
                3
            };
            ("t_connection", rot)
        }
        2 => {
            if (n && s) || (e && w) {
                // edge: variants [ns, ew]
                ("edge", if n { 0 } else { 1 })
            } else {
                // corner: variants [nw, ne, se, sw]
                let rot = match (n, e, s, w) {
                    (true, false, false, true) => 0, // NW
                    (true, true, false, false) => 1, // NE
                    (false, true, true, false) => 2, // SE
                    (false, false, true, true) => 3, // SW
                    _ => 0,
                };
                ("corner", rot)
            }
        }
        _ => {
            // end_piece: variants [n, e, s, w] = direction of the
            // single connection.
            let rot = if n {
                0
            } else if e {
                1
            } else if s {
                2
            } else {
                3
            };
            ("end_piece", rot)
        }
    }
}

/// The *state* subtile for a tile/part that carries one: the
/// game's vehicle-part renderer selects it instead of any
/// neighbor-mask subtile (pinned source, `cata_tiles.cpp`
/// `draw_vpart`-family: `vd.is_open ? open_ : vd.is_broken ?
/// broken : 0`, and the override path's identical
/// `part_mod == 1 ? open_ : part_mod == 2 ? broken : 0`).
/// **Open wins over broken** — that is the game's ternary order,
/// not a severity ranking. `None` = no state variant; the caller
/// falls back to mask selection (or the base layer).
pub fn state_subtile(is_open: bool, is_broken: bool) -> Option<&'static str> {
    if is_open {
        Some("open")
    } else if is_broken {
        Some("broken")
    } else {
        None
    }
}

/// Neighbor offsets of the 8-neighbor *shore* mask (Stage F part 2,
/// §10.1 shoreline autotiling), in bit order: bit 0 = N, 1 = NE,
/// 2 = E, 3 = SE, 4 = S, 5 = SW, 6 = W, 7 = NW. Grid space: north
/// is -y, south is +y. This is the one canonical copy of the shore
/// mask's bit order — `voxelizer::shore` (which generates the shore
/// textures) and the engine (which assigns them at world build)
/// both resolve through here, exactly as the cardinal table above
/// is canonical for Phase 2d.
pub const SHORE_NEIGHBORS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// The all-water shore mask: every neighbor is water (an interior
/// water tile — it keeps its plain water texture, no shore variant
/// is ever emitted or assigned for it).
pub const SHORE_MASK_ALL_WATER: u8 = 0xFF;

/// 8-neighbor water-adjacency mask for tile `(tx, ty)` under a
/// layout predicate in tile coordinates (arbitrary origin): bit
/// `i` is set when the neighbor at `SHORE_NEIGHBORS[i]` is water.
pub fn shore_mask_for(is_water: impl Fn(i32, i32) -> bool, tx: i32, ty: i32) -> u8 {
    let mut mask = 0u8;
    for (bit, (dx, dy)) in SHORE_NEIGHBORS.iter().enumerate() {
        if is_water(tx + dx, ty + dy) {
            mask |= 1 << bit;
        }
    }
    mask
}

/// All 16 masks -> (subtile, rotation), in mask order. Used by tests
/// and by engine-side variant selection.
pub fn all_mask_mappings() -> Vec<(u8, &'static str, u32)> {
    (0..16u8)
        .map(|m| {
            let (s, r) = subtile_for_mask(m);
            (m, s, r)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shore_mask_uses_the_voxelizer_bit_order() {
        // 3x3 pond at (1..=3, 1..=3): the center is all-water, the
        // NW corner sees water only at E, SE, S (bits 2, 3, 4).
        let is_water = |x: i32, y: i32| (1..=3).contains(&x) && (1..=3).contains(&y);
        assert_eq!(shore_mask_for(is_water, 2, 2), SHORE_MASK_ALL_WATER);
        assert_eq!(shore_mask_for(is_water, 1, 1), 0b0001_1100);
        // A land tile NW of the pond sees water only at SE (bit 3).
        assert_eq!(shore_mask_for(is_water, 0, 0), 0b0000_1000);
        // An isolated water tile has no water neighbors.
        assert_eq!(shore_mask_for(|x, y| x == 5 && y == 5, 5, 5), 0);
    }

    #[test]
    fn state_subtile_follows_the_games_ternary_order() {
        // Pinned source (cata_tiles.cpp, both vpart draw paths):
        // is_open ? open : is_broken ? broken : none.
        assert_eq!(
            state_subtile(true, true),
            Some("open"),
            "open wins over broken"
        );
        assert_eq!(state_subtile(true, false), Some("open"));
        assert_eq!(state_subtile(false, true), Some("broken"));
        assert_eq!(state_subtile(false, false), None);
    }
}
