//! World scale configuration — the single place the plan (§7) allows the
//! proportions to be tuned.

/// Scale and proportion config.
///
/// Defaults (Phase 0 decision): 1 tile = 1 block footprint, 1 z-level =
/// 3 blocks tall.
///
/// Phase 2 decision on plan §14 Q1 (floor/ceiling thickness): **thin slabs**.
/// A floor tile occupies only the bottom `floor_slab_blocks` (1 of 3) blocks
/// of its z-level. A roof tile is authored on the z-level ABOVE the room it
/// closes and occupies the bottom `roof_slab_blocks` (1 of 3) blocks of that
/// level -- a deck seated on the wall plate, matching the snapshot path's
/// `Floor::RoofOf` stamping. (The original reading put the roof at the TOP
/// of its level, floating it two blocks above the walls; corrected
/// 2026-11-17.) Full-height tiles (walls, doors, windows) fill all
/// `blocks_per_z_level`. This keeps interiors 2 blocks of clear headroom per
/// level and makes floors/ceilings read as slabs rather than solid storeys.
/// It is a config value, not a constant baked into the mesher: set
/// `floor_slab_blocks == blocks_per_z_level` to get full-block floors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldScale {
    /// World units per block edge.
    pub block_size: f32,
    /// Blocks of vertical space per game z-level.
    pub blocks_per_z_level: u32,
    /// Thickness (in blocks) of a floor slab within its z-level.
    pub floor_slab_blocks: u32,
    /// Thickness (in blocks) of a roof slab within its z-level.
    pub roof_slab_blocks: u32,
}

impl Default for WorldScale {
    fn default() -> Self {
        Self {
            block_size: 1.0,
            blocks_per_z_level: 3,
            floor_slab_blocks: 1,
            roof_slab_blocks: 1,
        }
    }
}

impl WorldScale {
    /// First block-z of game z-level `level` (levels may be negative:
    /// game z runs -10..=+10; chunk columns store levels 0-based, the
    /// mapping to game z is the caller's offset).
    pub fn level_base_block(self, level: u32) -> u32 {
        level * self.blocks_per_z_level
    }

    /// Total block height of a chunk column with `levels` z-levels.
    pub fn column_block_height(self, levels: u32) -> u32 {
        levels * self.blocks_per_z_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_phase0_decision() {
        let s = WorldScale::default();
        assert_eq!(s.block_size, 1.0);
        assert_eq!(s.blocks_per_z_level, 3);
        assert_eq!(s.column_block_height(21), 63);
        assert_eq!(s.level_base_block(2), 6);
    }
}
