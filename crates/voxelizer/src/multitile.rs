//! Multitile mapping (plan §8 rule 4, test 5.5). The game's model
//! (`cata_tiles::get_tile_values` + `get_rotation_and_subtile`) takes
//! the 4 cardinal neighbors and derives a subtile + rotation. The six
//! neighbour subtiles are: center, corner, edge, t_connection,
//! end_piece, unconnected. `open`/`broken` are state ids, NOT
//! neighbor variants, and are deliberately absent here.
//!
//! The mask table itself lives in `mesher::connect` (Phase 2d made
//! it the workspace's one canonical copy — the mesher is the leaf
//! crate every consumer already builds on) and is re-exported here
//! so Phase 5 call sites and gates keep their original paths.

pub use mesher::connect::{all_mask_mappings, mask_from_connects, subtile_for_mask};

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
