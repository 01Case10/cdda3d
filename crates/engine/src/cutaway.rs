//! Z-level cutaway selection (gate test 2.8).
//!
//! Game z-levels run `GAME_Z_MIN..=GAME_Z_MAX` (-10..=+10, 21 levels,
//! plan Appendix A). The cutaway decides which game z-levels are drawn for
//! a given player z. Selection is pure data — no indices into fixed-size
//! arrays — so the extremes cannot go out of range.

pub const GAME_Z_MIN: i32 = -10;
pub const GAME_Z_MAX: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutawayMode {
    /// Hide everything above `player_z + extra` (extra >= 0).
    HideAbove(i32),
    /// Show only the player's floor and the one above it.
    FloorPlusOne,
    /// Fade roofs: same layer selection as HideAbove(0) for walls/floors;
    /// roof slabs on the player's own level are rendered faded by the
    /// renderer (a material choice, not a layer choice).
    FadeRoof,
    /// Show every level (spectator / map overview).
    All,
}

impl CutawayMode {
    /// The set of visible game z-levels for `player_z`, always clamped to
    /// the valid game range and sorted ascending. `player_z` itself is
    /// clamped first, so out-of-range input is safe.
    pub fn visible_levels(&self, player_z: i32) -> Vec<i32> {
        let pz = player_z.clamp(GAME_Z_MIN, GAME_Z_MAX);
        let top = match *self {
            CutawayMode::HideAbove(extra) => pz.saturating_add(extra.max(0)),
            CutawayMode::FloorPlusOne => pz.saturating_add(1),
            CutawayMode::FadeRoof => pz,
            CutawayMode::All => GAME_Z_MAX,
        }
        .min(GAME_Z_MAX);
        (GAME_Z_MIN..=top).collect()
    }

    pub fn is_visible(&self, player_z: i32, level_z: i32) -> bool {
        level_z >= GAME_Z_MIN
            && level_z <= GAME_Z_MAX
            && self.visible_levels(player_z).contains(&level_z)
    }
}

/// Mutable cutaway state for the windowed app: current mode + player z,
/// stepped with PgUp/PgDn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CutawayState {
    pub mode: CutawayMode,
    pub player_z: i32,
}

impl Default for CutawayState {
    fn default() -> Self {
        Self {
            mode: CutawayMode::HideAbove(1),
            player_z: 0,
        }
    }
}

impl CutawayState {
    pub fn step_up(&mut self) {
        self.player_z = (self.player_z + 1).min(GAME_Z_MAX);
    }
    pub fn step_down(&mut self) {
        self.player_z = (self.player_z - 1).max(GAME_Z_MIN);
    }
    pub fn visible_levels(&self) -> Vec<i32> {
        self.mode.visible_levels(self.player_z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2_8_hide_above() {
        let vis = CutawayMode::HideAbove(1).visible_levels(0);
        assert_eq!(vis.first(), Some(&GAME_Z_MIN));
        assert_eq!(vis.last(), Some(&1));
        assert_eq!(vis.len(), 12); // -10..=1
        assert!(!CutawayMode::HideAbove(0).is_visible(0, 1));
        assert!(CutawayMode::HideAbove(0).is_visible(0, 0));
        assert!(CutawayMode::HideAbove(0).is_visible(0, -10));
    }

    #[test]
    fn test_2_8_extremes_no_index_errors() {
        for mode in [
            CutawayMode::HideAbove(0),
            CutawayMode::HideAbove(5),
            CutawayMode::FloorPlusOne,
            CutawayMode::FadeRoof,
            CutawayMode::All,
        ] {
            for pz in [GAME_Z_MIN, GAME_Z_MAX, -100, 100, 0] {
                let vis = mode.visible_levels(pz);
                assert!(!vis.is_empty(), "{mode:?} at {pz}");
                assert!(vis.iter().all(|z| (GAME_Z_MIN..=GAME_Z_MAX).contains(z)));
                let mut sorted = vis.clone();
                sorted.sort_unstable();
                assert_eq!(vis, sorted);
            }
        }
        // At the very bottom, HideAbove(0) shows exactly one level.
        assert_eq!(
            CutawayMode::HideAbove(0).visible_levels(GAME_Z_MIN),
            vec![GAME_Z_MIN]
        );
        // At the very top, everything is at or below the player.
        assert_eq!(
            CutawayMode::HideAbove(0).visible_levels(GAME_Z_MAX).len(),
            21
        );
        assert_eq!(CutawayMode::All.visible_levels(-3).len(), 21);
        assert_eq!(
            CutawayMode::FloorPlusOne.visible_levels(GAME_Z_MAX).len(),
            21
        );
        assert_eq!(CutawayMode::FloorPlusOne.visible_levels(0).last(), Some(&1));
    }

    #[test]
    fn test_2_8_state_stepping_clamps() {
        let mut s = CutawayState {
            mode: CutawayMode::HideAbove(0),
            player_z: GAME_Z_MAX,
        };
        s.step_up();
        assert_eq!(s.player_z, GAME_Z_MAX);
        s.player_z = GAME_Z_MIN;
        s.step_down();
        assert_eq!(s.player_z, GAME_Z_MIN);
        s.step_up();
        assert_eq!(s.visible_levels(), vec![GAME_Z_MIN, GAME_Z_MIN + 1]);
    }
}
