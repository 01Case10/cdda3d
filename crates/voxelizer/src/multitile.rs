//! Multitile mapping (plan §8 rule 4, test 5.5). The game's model
//! (`cata_tiles::get_tile_values` + `get_rotation_and_subtile`) takes
//! the 4 cardinal neighbors and derives a subtile + rotation. The six
//! neighbour subtiles are: center, corner, edge, t_connection,
//! end_piece, unconnected. `open`/`broken` are state ids, NOT
//! neighbor variants, and are deliberately absent here.

/// Neighbor bits: N=1, E=2, S=4, W=8 (mask over the 4 cardinal dirs).
pub fn subtile_for_mask(mask: u8) -> (&'static str, u32) {
    let n = mask & 1 != 0;
    let e = mask & 2 != 0;
    let s = mask & 4 != 0;
    let w = mask & 8 != 0;
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
/// and by the engine-side variant selection.
pub fn all_mask_mappings() -> Vec<(u8, &'static str, u32)> {
    (0..16u8)
        .map(|m| {
            let (s, r) = subtile_for_mask(m);
            (m, s, r)
        })
        .collect()
}

/// Connect-group test (test 5.5 group case): a neighbor connects to
/// `self` when it is the same id, or when any of the neighbor's
/// connect groups intersects the groups `self` connects to
/// (`get_tile_values_with_ter` semantics over exported group indices).
pub fn neighbors_connect(
    self_id: &str,
    self_connects_to: &[u16],
    neighbor_id: &str,
    neighbor_groups: &[u16],
) -> bool {
    if self_id == neighbor_id {
        return true;
    }
    neighbor_groups.iter().any(|g| self_connects_to.contains(g))
}

/// Build a neighbor mask from four connect tests (N, E, S, W order).
pub fn mask_from_connects(n: bool, e: bool, s: bool, w: bool) -> u8 {
    (n as u8) | ((e as u8) << 1) | ((s as u8) << 2) | ((w as u8) << 3)
}
