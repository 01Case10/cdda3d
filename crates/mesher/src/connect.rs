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
