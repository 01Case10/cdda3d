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
//! Passability: CDDA's own rule, `map::move_cost_ter_furn`
//! (src/map.cpp at the pinned commit): terrain `move_cost` == 0
//! blocks; furniture `move_cost_mod` < 0 blocks; otherwise the
//! tile is passable iff terrain cost + furniture modifier > 0.
//! Plus the sim's creature rule: no stepping onto an occupied
//! tile. Bumping a closed door therefore blocks (t_door_c has
//! move_cost 0) and does NOT auto-open, per the §5 example —
//! the step is blocked but the turn is still consumed.
//!
//! History: S2 originally used the WALL hot-flag + `open_result`
//! + creature rule, which diverged from the game — `t_tree`,
//! `t_window` and `t_chainfence` carry neither a WALL flag nor
//! an open_result in the real data, so all three were walk-
//! through. The movecost rule replaced it 2026-11-23 (Joey's
//! call); the regression gate is `movecost_passability` in
//! tests/turn_gate.rs, which fails on the old rule. The cost
//! itself does not yet scale turn length (every step is one
//! turn regardless of cost) — that is a later refinement, not
//! part of passability.
//!
//! S4 routing (plan §2.7/§3.3): inputs route by the UI mode.
//! - In `UiInput` mode, Key/MouseButton/Wheel inputs are
//!   consumed by the UI state machine (`crate::ui`): a change
//!   publishes exactly one UiFrame; the world does NOT change
//!   and no turn is consumed. Movement keys move the list /
//!   look cursor — never the player (gate T2).
//! - In `MapInput` mode, the UI-open bindings (`i` Inventory,
//!   `&` Crafting, `;` LookAround) are not turns either: they
//!   publish one UiFrame and nothing else. All other pressed
//!   Keys follow the S2/S3 turn rules below.
//! - Text inputs never drive the world: the Key channel
//!   carries actions (the engine sends both for printable
//!   keys), and the sim's UI states have no text fields — Text
//!   is accepted and ignored in both modes, so a Key+Text pair
//!   can never double-drive anything.
//! - MouseButton in MapInput: the engine does not forward
//!   terrain clicks as Inputs (it picks and answers with a
//!   PickResult message instead — gate 4.11); panel clicks do
//!   arrive, and the sim's panels are display-only, so they
//!   are accepted and ignored. MouseMove/Focus likewise.

use proto::{InputKind, InputT, MessageT, UiMode};

use crate::defs::{Def, DefRegistry, Kind};
use crate::ui::UiState;
use crate::world::World;

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
    /// Returns the input's message flush: a turn's Delta /
    /// Snapshot (+ LightUpdate / EnvironmentUpdate, S3), or a
    /// single UiFrame for a UI publish (S4), or an empty Vec
    /// when the input produces nothing (key release, an
    /// Open/Close arming key, an ignored input kind, a UI key
    /// that changed nothing). The server sends the returned
    /// messages verbatim; tests assert on them directly.
    pub fn apply_input(&mut self, reg: &DefRegistry, input: &InputT) -> Vec<MessageT> {
        match input.kind {
            InputKind::Key => {
                if !input.pressed {
                    return Vec::new();
                }
                self.apply_key(reg, input.key_code)
            }
            InputKind::MouseButton => {
                if !input.pressed || self.ui.mode() != UiMode::UiInput {
                    return Vec::new();
                }
                let view = self.ui_view(reg);
                if self.ui.apply_click(input.mouse_x, input.mouse_y, &view) {
                    return self.ui_flush(reg);
                }
                Vec::new()
            }
            InputKind::Wheel => {
                if self.ui.mode() != UiMode::UiInput {
                    return Vec::new();
                }
                let view = self.ui_view(reg);
                if self.ui.apply_wheel(input.wheel_delta, &view) {
                    return self.ui_flush(reg);
                }
                Vec::new()
            }
            // Text (see module docs), MouseMove, Focus: accepted,
            // no effect, no reply.
            _ => Vec::new(),
        }
    }

    /// Publish the UI after a state-machine change: exactly one
    /// UiFrame message (or none if the store write failed —
    /// `refresh_ui` logs it).
    fn ui_flush(&mut self, reg: &DefRegistry) -> Vec<MessageT> {
        self.refresh_ui(reg)
            .map(|f| vec![MessageT::UiFrame(Box::new(f))])
            .unwrap_or_default()
    }

    /// One pressed Key, routed by the UI mode (S4).
    fn apply_key(&mut self, reg: &DefRegistry, key_code: i32) -> Vec<MessageT> {
        // UiInput: the state machine consumes the key; the
        // world does not change and no turn is consumed.
        if self.ui.mode() == UiMode::UiInput {
            let view = self.ui_view(reg);
            let bounds = self.look_bounds();
            if self.ui.apply_ui_key(key_code, &view, bounds) {
                return self.ui_flush(reg);
            }
            return Vec::new();
        }

        let key = char::from_u32(key_code as u32).unwrap_or('\0');

        // A pending Open/Close takes the next direction key.
        if let Some(action) = self.pending.take() {
            if let Some(dir) = direction_for_key(key) {
                self.apply_open_close(reg, action, dir);
                return self.flush_turn(reg);
            }
            // Any other key cancels the pending action and is
            // then processed as its own key below.
        }

        match key {
            // UI-open bindings (§3.3): not turns — one UiFrame,
            // no Delta, the clock does not advance.
            'i' => {
                self.ui.open(UiState::Inventory, self.player_pos());
                self.ui_flush(reg)
            }
            '&' => {
                self.ui.open(UiState::Crafting, self.player_pos());
                self.ui_flush(reg)
            }
            ';' => {
                self.ui.open(UiState::LookAround, self.player_pos());
                self.ui_flush(reg)
            }
            'o' => {
                self.pending = Some(PendingAction::Open);
                Vec::new()
            }
            'c' => {
                self.pending = Some(PendingAction::Close);
                Vec::new()
            }
            _ => {
                if let Some(dir) = direction_for_key(key) {
                    self.try_move(reg, dir);
                }
                // '.' / ' ' wait; every other key is a no-op turn.
                self.flush_turn(reg)
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
        // map::move_cost_ter_furn, mirrored exactly (see module
        // docs). A def with no move_cost in the table counts as
        // the game's member default, 0 = impassable.
        let ter_cost = def.move_cost.unwrap_or(0);
        if ter_cost == 0 {
            return;
        }
        let furn_mod = match self.furniture_at(target) {
            Some(0) | None => 0, // no furniture: null furniture's mod is 0
            Some(furn_id) => reg
                .def_by_id(Kind::Furniture, furn_id)
                .and_then(|d| d.move_cost)
                .unwrap_or(0),
        };
        if furn_mod < 0 || ter_cost + furn_mod <= 0 {
            return;
        }
        if self.creature_at(target) {
            return; // creature in the way
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
        // The message log (gameplay sidebar) records the action.
        self.messages.push(match action {
            PendingAction::Open => "You open the door.".to_string(),
            PendingAction::Close => "You close the door.".to_string(),
        });
    }
}
