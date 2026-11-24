//! Turn rules (slice S2): one pressed Key Input = one turn =
//! one flush (a Delta, or a fresh Snapshot on a map shift).
//!
//! Key map (key_code is the key's Unicode char, as in the
//! samples where 105 = 'i'):
//! - Movement: vi-keys `h` west, `j` south, `k` north, `l` east,
//!   `y` northwest, `u` northeast, `b` southwest, `n` southeast;
//!   WASD cardinals as aliases (`w` north, `a` west, `s` south,
//!   `d` east).
//! - `.` or ` ` waits.
//! - `o` / `c` arm a pending Open / Close: the NEXT movement-
//!   direction key applies the action to the adjacent tile in
//!   that direction instead of moving (CDDA's `o`/<dir> pattern).
//!   Arming does not consume a turn and produces no message; any
//!   non-direction key cancels the pending action (and is then
//!   processed as its own key). A direction key that applies a
//!   pending action still consumes the turn.
//! - Any other pressed key consumes a turn with no effect. Key
//!   releases and non-Key inputs are not turns at all: no state
//!   change, no reply.
//!
//! Passability (§3.2): a step succeeds iff the target terrain
//! (a) does NOT carry the WALL flag, (b) has no `open_result`
//! (a closed openable — door/window — blocks; bumping does NOT
//! auto-open, per the §5 example: the step is blocked but the
//! turn is still consumed), and (c) no creature occupies the
//! tile. Furniture never blocks in S2.
//!
//! The WALL test uses the def's resolved `hot_flags` bitset —
//! the engine-facing form (§2.6: the engine never parses flag
//! strings). `WALL` is `ter_furn_flag::TFLAG_WALL`, index 25 in
//! the enum order declared in `src/mapdata.h` at the pinned
//! commit (TRANSPARENT = 0 ... UNSTABLE = 24, WALL = 25), and
//! 25 < 64 so the S1a importer keeps it in `hot_flags` (verified
//! against the generated table: `t_wall` has bit 25 set;
//! `t_floor`/`t_door_c`/`t_window`/`t_grass` do not).

use proto::{InputKind, InputT, MessageT};

use crate::defs::{Def, DefRegistry, Kind};
use crate::world::World;

/// `ter_furn_flag` index of WALL (see module docs).
pub const WALL_HOT_BIT: u32 = 25;
const WALL_HOT_MASK: u64 = 1 << WALL_HOT_BIT;

/// An armed Open/Close awaiting its direction key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    Open,
    Close,
}

/// Movement direction (dx, dy) for a key char, if it is one.
pub fn direction_for_key(key: char) -> Option<(i32, i32)> {
    Some(match key {
        'h' | 'a' => (-1, 0),
        'j' | 's' => (0, 1),
        'k' | 'w' => (0, -1),
        'l' | 'd' => (1, 0),
        'y' => (-1, -1),
        'u' => (1, -1),
        'b' => (-1, 1),
        'n' => (1, 1),
        _ => return None,
    })
}

/// A def's transform string, with "" normalised to none (the
/// importer stores JSON null as None, but be liberal).
fn transform<'a>(def: &'a Def, which: fn(&Def) -> &Option<String>) -> Option<&'a str> {
    which(def).as_deref().filter(|s| !s.is_empty())
}

impl World {
    /// Drive one Input (in-process mode: no socket involved).
    /// Returns the message the turn produced, or `None` when the
    /// input is not a turn (key release, non-Key input, or an
    /// Open/Close arming key). The server sends the returned
    /// message verbatim; tests assert on it directly.
    pub fn apply_input(&mut self, reg: &DefRegistry, input: &InputT) -> Option<MessageT> {
        if input.kind != InputKind::Key || !input.pressed {
            return None;
        }
        let key = char::from_u32(input.key_code as u32).unwrap_or('\0');

        // A pending Open/Close takes the next direction key.
        if let Some(action) = self.pending.take() {
            if let Some(dir) = direction_for_key(key) {
                self.apply_open_close(reg, action, dir);
                return Some(self.flush_turn(reg));
            }
            // Any other key cancels the pending action and is
            // then processed as its own key below.
        }

        match key {
            'o' => {
                self.pending = Some(PendingAction::Open);
                None
            }
            'c' => {
                self.pending = Some(PendingAction::Close);
                None
            }
            _ => {
                if let Some(dir) = direction_for_key(key) {
                    self.try_move(reg, dir);
                }
                // '.' / ' ' wait; every other key is a no-op turn.
                Some(self.flush_turn(reg))
            }
        }
    }

    /// Attempt to step the player by `dir`. A blocked step
    /// changes nothing (the caller still flushes the turn).
    fn try_move(&mut self, reg: &DefRegistry, dir: (i32, i32)) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let target = [pos[0] + dir.0, pos[1] + dir.1, pos[2]];
        let Some(terrain) = self.terrain_at(target) else {
            return; // outside the scene: blocked
        };
        let Some(def) = reg.def_by_id(Kind::Terrain, terrain) else {
            return; // unknown terrain id: blocked, never a panic
        };
        if def.hot_flags & WALL_HOT_MASK != 0 {
            return; // (a) WALL flag
        }
        if transform(def, |d| &d.open_result).is_some() {
            return; // (b) closed openable: bump does not open
        }
        if self.creature_at(target) {
            return; // (c) creature in the way
        }
        self.set_player_pos(target);
    }

    /// Apply an armed Open/Close to the tile adjacent to the
    /// player in `dir`, following the terrain def's
    /// `open_result` / `close_result` through the registry. A
    /// tile without the transform is a no-op and produces NO
    /// TileChange (nothing mutates, nothing is dirtied). A
    /// transform naming a def the registry does not have is
    /// logged and treated as a no-op — scene/registry problems
    /// must never panic mid-game.
    fn apply_open_close(&mut self, reg: &DefRegistry, action: PendingAction, dir: (i32, i32)) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let target = [pos[0] + dir.0, pos[1] + dir.1, pos[2]];
        let Some(terrain) = self.terrain_at(target) else {
            return;
        };
        let Some(def) = reg.def_by_id(Kind::Terrain, terrain) else {
            return;
        };
        let sid = match action {
            PendingAction::Open => transform(def, |d| &d.open_result),
            PendingAction::Close => transform(def, |d| &d.close_result),
        };
        let Some(sid) = sid else {
            return; // no such transform: no-op
        };
        let sid = sid.to_string();
        let new_id = reg.id_of(Kind::Terrain, &sid);
        if new_id == 0 {
            eprintln!("sim: {action:?} transform target '{sid}' is not in the registry; no-op");
            return;
        }
        self.set_terrain(target, new_id);
    }
}
