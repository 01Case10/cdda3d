//! Turn rules (slice S2): one pressed Key Input = one turn =
//! one flush (a Delta, or a fresh Snapshot on a map shift).
//!
//! Key map (key_code is the key's Unicode char, as in the
//! samples where 105 = 'i'):
//! - Movement: vi-keys `h` west, `j` south, `k` north, `l` east,
//!   `y` northwest, `u` northeast, `b` southwest, `n` southeast;
//!   `w` north and `a` west remain as aliases.
//! - `.` or ` ` waits.
//! - `o` / `c` arm a pending Open / Close, `s` arms a pending
//!   Bash (S6): the NEXT movement-direction key applies the
//!   action to the adjacent tile in that direction instead of
//!   moving (CDDA's `o`/<dir> pattern). Arming does not
//!   consume a turn and produces no message; any non-direction
//!   key cancels the pending action (and is then processed as
//!   its own key). A direction key that applies a pending
//!   action still consumes the turn.
//! - `g` picks up one unit from the player's own tile, `d`
//!   drops one unit onto it (S6; see below).
//! - `>` descends and `<` ascends (S8; see below).
//! - Any other pressed key consumes a turn with no effect. Key
//!   releases and non-Key inputs are not turns at all: no state
//!   change, no reply.
//!
//! **Keymap settlement (S6):** the action keys are the game's
//! own defaults from 0.I-1 `data/raw/keybindings.json` —
//! `smash` = `s`, `pickup` = `g`, `drop` = `d`, matching
//! `open` = `o` / `close` = `c` from S2. Where a real binding
//! collides with a movement alias, the real binding wins:
//! S2's WASD aliases `s` (south) and `d` (east) are REMOVED
//! (south is `j`, east is `l`); `w`/`a` stay because they
//! collide only with actions the sim does not model (wear /
//! apply). Gates written against the old aliases were amended
//! with the reason recorded in place.
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
//! also scales turn length (see the Phase 11 follow-up
//! section below).
//!
//! S6 additions:
//! - **Bash (`s` + direction):** the adjacent tile advances
//!   ONE step along its bash chain. Resolution mirrors
//!   upstream `map::bash_ter_furn` (0.I-1 src/map.cpp:3932+):
//!   furniture is smashed first when the tile has any — a
//!   furniture def's `bash_result` (furn_set) becomes the new
//!   furniture, `"f_null"` removes it; terrain otherwise —
//!   its `bash_result` (ter_set) becomes the new terrain, and
//!   `"t_null"` destroys the tile to `t_open_air` (upstream
//!   `ter_set(p, ter_t_open_air)` when the result id is null,
//!   map.cpp:4203). Two named simplifications: (1) the sim
//!   has no strength stat, so one bash = one chain step,
//!   deterministic — upstream rolls `strength + damage >=
//!   str_max` with damage accruing across attempts; (2) a
//!   furniture def with NO `bash_result` is REMOVED outright
//!   (plan §3.2's rule) where upstream's missing bash spec
//!   falls through to the terrain — the plan's rule is the
//!   one the gates pin. Terrain with no `bash_result` is
//!   unbashable: a no-op, turn still consumed. A successful
//!   bash queues exactly one HitFlash + one Sound
//!   (`"smash_success"`, upstream's success sound id,
//!   map.cpp:4108) at the target tile. Bashing a creature's
//!   tile is out of scope (the bump attack owns creature
//!   tiles — the bash is a no-op), and so is bashing a tile
//!   holding a vehicle part (vehicle damage has its own
//!   entry point, `World::damage_vehicle_part`).
//! - **Pickup (`g`) / drop (`d`)**, own tile only (CDDA's
//!   adjacent-pickup UI is out of scope): pickup takes ONE
//!   unit from the tile's pile — `item_count` −1, `top_item`
//!   stable until the last unit clears it — and the unit
//!   joins the carried list (so the S4 inventory UI shows
//!   it). Drop removes one unit from the FIRST carried stack
//!   (list order) onto the player's tile: an empty tile takes
//!   it as `top_item` × 1, a matching pile grows by one, and
//!   a tile whose pile is a DIFFERENT kind refuses the drop
//!   (the lossy wire cannot hold two kinds on one tile) —
//!   the state does not change but the turn is still
//!   consumed, the sim's attempted-action rule, same as a
//!   blocked step. Empty-tile pickup and empty-hands drop
//!   are likewise no-op turns.
//!
//! S8 additions:
//! - **Stairs / z-move (`>` / `<`):** `>` descends, `<`
//!   ascends — CDDA's own convention (0.I-1 src/action.cpp:
//!   498-501 maps the `move_down` / `move_up` actions;
//!   src/mapdata.h:184-185 documents the stair flags with
//!   exactly these keys). The move is legal only while
//!   standing on a tile whose terrain OR furniture def
//!   carries the matching flag — GOES_DOWN for `>`, GOES_UP
//!   for `<` (upstream's gate is `game::move_vertical`'s
//!   `has_flag(TFLAG_GOES_DOWN/GOES_UP, pos)` test,
//!   game.cpp:12430 / 12336, over ter_furn_flag, so both
//!   layers count). The destination is the same (x, y) one
//!   level over (upstream's `stairs` is `pos.xy()` at
//!   `pos.z() + movez`); the sim's landing rule: the
//!   destination must exist, be passable by the usual
//!   movecost rule, be unoccupied, and its terrain must not
//!   be NO_FLOOR (you land on a floor/stair, not into open
//!   air — the stair tiles themselves are floors). Anything
//!   else is a no-op turn (the attempted-action convention).
//!   Upstream's climbing fallback (no stair flag: climb
//!   checks, wall cling, swimming) is NOT modelled — the
//!   sim's rule is the stair path only, named here. A z
//!   change crosses a submap boundary (the submap index
//!   includes z), so the flush rides the existing map-shift
//!   path: fresh Snapshot, epoch + 1 — the contract's
//!   "snapshot window recentres", in the sim's no-culling
//!   form (a Snapshot carries all scene submaps).
//!
//! Movecost time scaling (Phase 11 follow-up, 2026-11-24):
//! - **Model.** Upstream runs on move points: 1 calendar
//!   turn = 1 second = 100 points (`time_duration::
//!   from_moves(t) = t/100` turns, `from_seconds(t) = t`
//!   turns — src/calendar.h:216-222); `do_turn()` advances
//!   `calendar::turn += 1_turns` once per call (src/
//!   do_turn.cpp:475) and each creature's points replenish
//!   by its speed per second (`moves += get_speed()`,
//!   src/creature.cpp:388). A walking step deducts
//!   `run_cost(combined_movecost(from, to))` points
//!   (src/game.cpp:10804-10814); `combined_movecost`
//!   (src/map.cpp:2466-2488) is `(cost_from + cost_to) *
//!   mult / 2` over the tiles' raw JSON costs, mult 50
//!   cardinal / 71 diagonal. The sim's avatar is the
//!   unmodified baseline (speed 100, `run_cost` the
//!   identity), so a floor->floor step costs exactly 100
//!   points. The player acts while their balance is > 0
//!   (src/do_turn.cpp:564-605); when a step drives it to
//!   <= 0, more do_turns elapse before the next action —
//!   the sim carries the balance across actions
//!   (`World::move_balance`, `spend_move_points`) and
//!   `flush_turn` lives through one second per refill, so
//!   upstream's aggregate timing is exact: cost-250 steps
//!   alternate 2 s / 3 s from a full balance.
//! - **Per-second systems.** Fields and creatures tick
//!   once per do_turn upstream (`m.process_fields()`,
//!   src/do_turn.cpp:655; `monmove()`, :666) — i.e. once
//!   per elapsed SECOND, not once per player action — so
//!   a 3-second step gives the world 3 field ticks and 3
//!   creature rounds, and weather-script entries fire at
//!   their absolute calendar turn inside the span. (At
//!   the 1-second span of every other action this is
//!   indistinguishable from the old per-action tick.)
//!   Creature MOVEMENT within a round still uses the
//!   sim's per-round rule (one tile, S5) — creature move
//!   points/speeds are not modelled, named here.
//! - **Scope.** ONLY successful movement steps spend
//!   points: horizontal steps by the formula above, and
//!   stair z-moves at `move_vertical`'s base 100 points
//!   (`int move_cost = 100`, src/game.cpp:12331, deducted
//!   at :12621; the +500 climbing surcharges belong to
//!   the unmodelled climbing fallback). Everything else
//!   stays exactly 1 s and never touches the balance:
//!   waits, Open/Close, Bash, pickup/drop, bump attacks,
//!   blocked steps, no-op keys. Upstream gives several of
//!   those their own point costs (a wait deducts
//!   `get_speed()` = 100, src/avatar_action.cpp:529;
//!   pickup deducts the item's obtain cost, :1192) — at
//!   the baseline avatar those are all 100-point actions
//!   whose timing equals the sim's flat 1 s anyway; the
//!   follow-up's scope was the terrain-driven scaling,
//!   and melee/weapon costs are outside the sim's
//!   fixed-damage abstraction. Gate:
//!   tests/movecost_time_gate.rs.
//!
//! S5 additions:
//! - **Bump attack:** stepping into a creature's tile is a
//!   melee attack (CDDA's rule), not a step: the creature
//!   takes `PLAYER_MELEE_DAMAGE`, a HitFlash rides the flush,
//!   the player does not move, the turn is consumed.
//! - **Creature phase** (after the player's action, once per
//!   turn, creatures in id order): a creature adjacent to the
//!   player attacks (its def's expected melee damage; player
//!   hp_frac falls; HitFlash + Sound effects); else if it can
//!   see the player (flood LOS + the player's tile lit !=
//!   BLANK in the computed cache) it takes one greedy 8-way
//!   chase step minimising Chebyshev distance, under the same
//!   passability as the player (it cannot open doors and
//!   cannot enter occupied tiles); else it wanders one seeded
//!   step or stays. Death (HP <= 0, from any source) removes
//!   the entity and drops a corpse item — see
//!   `World::damage_entity`.
//! - **Field phase** (before the creature phase): fire and
//!   smoke tick — see `tick_fields` for the rule set and its
//!   upstream grounding. Field damage lands here, so a
//!   creature killed by its fire does not act that turn.
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

use proto::{EffectEventT, EffectKind, InputKind, InputT, LitLevel, MessageT, UiMode, Vec3iT};

use crate::defs::{Def, DefRegistry, Kind};
use crate::ui::UiState;
use crate::visibility;
use crate::world::World;

/// The player's bump-attack damage (S5 sim constant): a
/// fixed melee value, roughly an unarmed hit — CDDA derives
/// the avatar's melee damage from stats/skills/equipment,
/// none of which the sim models; the constant is named here
/// so gates can do exact HP arithmetic (mon_zombie: 80 HP =
/// 10 bumps).
pub const PLAYER_MELEE_DAMAGE: i32 = 8;

/// Fire damage per turn to a creature (or the player)
/// standing in fd_fire, by intensity 1..=3 (S5): upstream
/// rolls rng(2,6) / rng(6,12) / rng(10,20) plus a flat 3 for
/// flesh creatures (0.I-1 map_field.cpp, the creature
/// field-effects section of `field_processor_fd_fire`'s
/// file — `cur_field_type == fd_fire` in the damage loop);
/// the sim deals the expectations 3+4 / 3+9 / 3+15. No
/// material/armour model (the registry carries neither), and
/// the FIREPROOF/FIREY exemption is not applied — monster
/// flag strings are not in the def table (the importer
/// carries flags for terrain/furniture only).
pub const FIRE_DAMAGE: [i32; 3] = [7, 12, 18];
/// Turns of burning between intensity growth steps (S5).
pub const FIRE_GROW_EVERY: i32 = 8;
/// Turns between intensity decay steps once fuel is gone.
pub const FIRE_DECAY_EVERY: i32 = 6;
/// Fuel (turns) a fire draws from its tile, by flammability
/// class: FLAMMABLE / FLAMMABLE_ASH tiles feed a fire for
/// FUEL_FLAMMABLE turns; FLAMMABLE_HARD-only tiles burn
/// slower and longer (upstream consumes their fuel at
/// one_in(3) per turn vs every turn); a non-flammable tile
/// gives FUEL_NONE — "nothing to burn = fire should be dying
/// out faster" (map_field.cpp, fd_fire processor). Upstream
/// fuel is measured in field age (tens of minutes); the sim
/// compresses the timescale so a cabin fire lives and dies
/// inside a gate's horizon — the shape (grow while fuelled,
/// decay when spent, intensity caps from burning neighbours)
/// is upstream's, the constants are the sim's.
pub const FUEL_FLAMMABLE: i32 = 90;
/// See `FUEL_FLAMMABLE`.
pub const FUEL_HARD: i32 = 150;
/// See `FUEL_FLAMMABLE`.
pub const FUEL_NONE: i32 = 8;
/// Turns before an undrifted smoke entry loses an intensity
/// level (the sim's stand-in for fd_smoke's 2-minute
/// half-life at gate timescale).
pub const SMOKE_DECAY_EVERY: i32 = 12;
/// HitFlash / melee Sound duration on the wire (ms).
pub const HIT_EFFECT_MS: u32 = 100;

/// The 8 movement directions in fixed (row-major) order —
/// every neighbour scan in the turn rules uses this order,
/// so seeded-RNG draws and tie-breaks are deterministic.
pub const DIRS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// The sim's wind: a single downwind step, +x. The sim has
/// no weather wind model — the EnvironmentUpdate carries the
/// fixture constants (5 mph, wind_dir_deg 90; see
/// environment.rs) — and 90 degrees maps to the vector (1, 0)
/// by the game's own `convert_wind_to_coord` (0.I-1
/// weather.cpp:852-863: the 60..120 band yields (1, 0)).
pub const WIND_STEP: (i32, i32) = (1, 0);

fn v3(pos: [i32; 3]) -> Vec3iT {
    Vec3iT {
        x: pos[0],
        y: pos[1],
        z: pos[2],
    }
}

/// A HitFlash at a tile: effect id "hit" (the transient
/// system's recognised id — crates/transient gate
/// convention), from == to == the struck tile.
fn hit_flash(pos: [i32; 3]) -> EffectEventT {
    let mut e = EffectEventT::default();
    e.kind = EffectKind::HitFlash;
    e.effect_id = Some("hit".to_string());
    e.from_pos = Some(v3(pos));
    e.to_pos = Some(v3(pos));
    e.duration_ms = HIT_EFFECT_MS;
    e
}

/// A melee Sound: from the attacker to the struck tile.
/// Sound ids are soundpack strings upstream (no closed set);
/// the sim's is "melee_hit".
fn melee_sound(from: [i32; 3], to: [i32; 3]) -> EffectEventT {
    let mut e = EffectEventT::default();
    e.kind = EffectKind::Sound;
    e.effect_id = Some("melee_hit".to_string());
    e.from_pos = Some(v3(from));
    e.to_pos = Some(v3(to));
    e.duration_ms = HIT_EFFECT_MS;
    e
}

/// A bash Sound at the smashed tile: upstream plays the
/// success sound AT the bashed position
/// (`sounds::sound(p, ..., "smash_success")`, map.cpp:4108),
/// so from == to == the target.
fn bash_sound(pos: [i32; 3]) -> EffectEventT {
    let mut e = EffectEventT::default();
    e.kind = EffectKind::Sound;
    e.effect_id = Some("smash_success".to_string());
    e.from_pos = Some(v3(pos));
    e.to_pos = Some(v3(pos));
    e.duration_ms = HIT_EFFECT_MS;
    e
}

/// Chebyshev distance (8-way movement's natural metric).
fn chebyshev(a: [i32; 3], b: [i32; 3]) -> i32 {
    (a[0] - b[0]).abs().max((a[1] - b[1]).abs()).max((a[2] - b[2]).abs())
}

/// An armed Open/Close/Bash awaiting its direction key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    Open,
    Close,
    Bash,
}

/// Movement direction (dx, dy) for a key char, if it is one.
/// The vi-keys are the full set; `w`/`a` are the surviving
/// WASD aliases — `s` and `d` are the game's smash/drop keys
/// since S6 (see the module docs' keymap settlement).
/// Move points a step from one tile to the next costs
/// (movecost time scaling): upstream's `map::
/// combined_movecost` (0.I-1 src/map.cpp:2466-2488) over
/// the two tiles' raw costs — `(cost_from + cost_to) *
/// mult / 2` with mult = 50 for a cardinal step and 71
/// for a diagonal (the trigdist table {0, 50, 71, 100}:
/// 100% / 141% of the two-tile average). The walking code
/// deducts `run_cost(that value)` (src/game.cpp:10804-
/// 10814); for the sim's unmodified baseline avatar
/// `run_cost` (src/character.cpp:10363) returns it
/// unchanged — its diagonal divide/re-multiply by sqrt(2)
/// cancels against the mult table and it has no cost
/// effects to apply. Floor (raw 2) to floor is therefore
/// (2+2)*50/2 = 100 points = exactly 1 second.
fn step_move_points(cost_from: i32, cost_to: i32, dir: (i32, i32)) -> i32 {
    let mult = if dir.0 != 0 && dir.1 != 0 { 71 } else { 50 };
    (cost_from + cost_to) * mult / 2
}

pub fn direction_for_key(key: char) -> Option<(i32, i32)> {
    Some(match key {
        'h' | 'a' => (-1, 0),
        'j' => (0, 1),
        'k' | 'w' => (0, -1),
        'l' => (1, 0),
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

        // A pending Open/Close/Bash takes the next direction key.
        if let Some(action) = self.pending.take() {
            if let Some(dir) = direction_for_key(key) {
                match action {
                    PendingAction::Bash => self.apply_bash(reg, dir),
                    PendingAction::Open | PendingAction::Close => {
                        self.apply_open_close(reg, action, dir)
                    }
                }
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
            // Smash (0.I-1 keybindings: "smash" = 's'): arm a
            // pending Bash, exactly like Open/Close.
            's' => {
                self.pending = Some(PendingAction::Bash);
                Vec::new()
            }
            // Pickup ("pickup" = 'g'): one unit from the
            // player's own tile into the carried list.
            'g' => {
                self.pickup(reg);
                self.flush_turn(reg)
            }
            // Drop ("drop" = 'd'): one unit from the first
            // carried stack onto the player's own tile.
            'd' => {
                self.drop_item(reg);
                self.flush_turn(reg)
            }
            // Stairs (S8): '>' descends, '<' ascends, only
            // from a tile carrying the matching GOES_* flag.
            '>' => {
                self.try_z_move(reg, -1);
                self.flush_turn(reg)
            }
            '<' => {
                self.try_z_move(reg, 1);
                self.flush_turn(reg)
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
    /// Stepping into a creature's tile is a bump attack
    /// (S5): the creature takes `PLAYER_MELEE_DAMAGE` and a
    /// HitFlash is queued; the player does not move.
    fn try_move(&mut self, reg: &DefRegistry, dir: (i32, i32)) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let target = [pos[0] + dir.0, pos[1] + dir.1, pos[2]];
        if let Some(zid) = self.creature_id_at(target) {
            self.damage_entity(reg, zid, PLAYER_MELEE_DAMAGE);
            self.push_effect(hit_flash(target));
            return;
        }
        // map::move_cost_ter_furn, mirrored exactly (see
        // module docs) — `World::passable`.
        if !self.passable(reg, target) {
            return;
        }
        // Movecost time scaling (Phase 11 follow-up; model
        // in the module docs): the step spends its combined
        // move points from the player's balance, and any
        // refills that drives are elapsed seconds the flush
        // will live through.
        let points = step_move_points(
            self.tile_move_cost(reg, pos),
            self.tile_move_cost(reg, target),
            dir,
        );
        self.set_player_pos(target);
        self.spend_move_points(points);
    }

    /// Attempt a z-move (S8): `dz` -1 descends (`>`), +1
    /// ascends (`<`). See the module docs for the rule and
    /// its upstream grounding. A failed attempt changes
    /// nothing (the caller still flushes the turn).
    fn try_z_move(&mut self, reg: &DefRegistry, dz: i32) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let flag = if dz < 0 { "GOES_DOWN" } else { "GOES_UP" };
        // Upstream tests ter_furn_flag at the player's tile:
        // either layer (terrain or furniture) can carry it.
        let has = |kind: Kind, id: u32| {
            id != 0
                && reg
                    .def_by_id(kind, id)
                    .is_some_and(|d| d.flags.iter().any(|f| f == flag))
        };
        let terrain = self.terrain_at(pos).unwrap_or(0);
        let furniture = self.furniture_at(pos).unwrap_or(0);
        if !has(Kind::Terrain, terrain) && !has(Kind::Furniture, furniture) {
            return;
        }
        let dest = [pos[0], pos[1], pos[2] + dz];
        // Landing rule: the destination must be a real,
        // passable, unoccupied floor — not open air.
        let Some(dest_terrain) = self.terrain_at(dest) else {
            return;
        };
        let Some(dest_def) = reg.def_by_id(Kind::Terrain, dest_terrain) else {
            return;
        };
        if dest_def.flags.iter().any(|f| f == "NO_FLOOR") {
            return;
        }
        if self.creature_id_at(dest).is_some() || !self.passable(reg, dest) {
            return;
        }
        self.set_player_pos(dest);
        // A stair move costs upstream's base 100 move points
        // (`game::move_vertical`: `int move_cost = 100`,
        // game.cpp:12331, deducted via `u.mod_moves(
        // -move_cost)` at :12621; the +500 climbing cases
        // are the fallback the sim does not model). From a
        // full balance that is the same 1 s as a floor step.
        self.spend_move_points(100);
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
            // Bash never reaches here (apply_key routes it to
            // apply_bash); no transform.
            PendingAction::Bash => None,
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
            PendingAction::Bash => "You smash it.".to_string(),
        });
    }

    /// Apply an armed Bash to the tile adjacent to the player
    /// in `dir` (S6; the rule set and its upstream grounding
    /// are in the module docs). One chain step on success,
    /// exactly one HitFlash + Sound; every failure mode is a
    /// silent no-op (the caller still flushes the turn).
    fn apply_bash(&mut self, reg: &DefRegistry, dir: (i32, i32)) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let target = [pos[0] + dir.0, pos[1] + dir.1, pos[2]];
        self.bash_tile(reg, target);
    }

    /// Smash the tile at `target`: one step along its bash
    /// chain (furniture first, then terrain — `apply_bash`'s
    /// resolution, shared verbatim). Returns true when a smash
    /// landed (chain step or furniture removal; the HitFlash +
    /// Sound effects are queued for the caller's flush).
    ///
    /// This is the position-based entry point the S7 scenario
    /// DSL's `bash` / `damage_tile` steps wrap: tiles have no
    /// HP in the sim, so "damage a tile" IS one bash-chain
    /// step, with no player involved. The scoping is the
    /// player's own: creature tiles belong to the bump attack
    /// and vehicle-part tiles to the vehicle damage API, so
    /// both are no-ops here too.
    pub fn bash_tile(&mut self, reg: &DefRegistry, target: [i32; 3]) -> bool {
        // Scope: creature tiles belong to the bump attack, and
        // vehicle-part tiles to the vehicle damage API.
        if self.creature_id_at(target).is_some() {
            return false;
        }
        if self.vehicle_part_at(target).is_some() {
            return false;
        }
        // Furniture first (upstream's priority in
        // bash_ter_furn): any furniture on the tile is the
        // bash target.
        let furniture = self.furniture_at(target).unwrap_or(0);
        if furniture != 0 {
            let result = reg
                .def_by_id(Kind::Furniture, furniture)
                .and_then(|d| transform(d, |d| &d.bash_result).map(str::to_string));
            match result {
                // A furn_set chain step; "f_null" is the
                // removal sentinel (id 0).
                Some(sid) => {
                    let new_id = reg.id_of(Kind::Furniture, &sid);
                    if new_id == 0 && sid != "f_null" {
                        eprintln!(
                            "sim: bash result '{sid}' is not in the registry; no-op"
                        );
                        return false;
                    }
                    self.set_furniture(target, new_id);
                }
                // No bash_result: the furniture is destroyed
                // outright (plan §3.2's rule — see module
                // docs for the upstream divergence).
                None => {
                    self.set_furniture(target, 0);
                }
            }
            self.push_effect(hit_flash(target));
            self.push_effect(bash_sound(target));
            self.messages.push("You smash it.".to_string());
            return true;
        }
        // Terrain: no bash_result = unbashable.
        let Some(terrain) = self.terrain_at(target) else {
            return false;
        };
        let Some(def) = reg.def_by_id(Kind::Terrain, terrain) else {
            return false;
        };
        let Some(sid) = transform(def, |d| &d.bash_result).map(str::to_string) else {
            return false;
        };
        // "t_null" destroys the tile: upstream's ter_set with
        // a null result id lands on t_open_air (map.cpp:4203).
        let new_id = if sid == "t_null" {
            reg.id_of(Kind::Terrain, "t_open_air")
        } else {
            reg.id_of(Kind::Terrain, &sid)
        };
        if new_id == 0 {
            eprintln!("sim: bash result '{sid}' is not in the registry; no-op");
            return false;
        }
        self.set_terrain(target, new_id);
        self.push_effect(hit_flash(target));
        self.push_effect(bash_sound(target));
        self.messages.push("You smash it.".to_string());
        true
    }

    /// Pickup (S6): take one unit from the pile on the
    /// player's own tile into the carried list. An empty tile
    /// is a no-op (the caller still flushes the turn).
    fn pickup(&mut self, reg: &DefRegistry) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let Some((top, count)) = self.tile_items(pos) else {
            return;
        };
        if top == 0 || count == 0 {
            return;
        }
        // top_item stays stable until the last unit, which
        // clears the pile entirely.
        let (new_top, new_count) = if count > 1 {
            (top, count - 1)
        } else {
            (0, 0)
        };
        self.set_tile_items(pos, new_top, new_count);
        self.carried_add(top);
        let name = Self::item_display_name(reg, top);
        self.messages.push(format!("You pick up the {name}."));
    }

    /// Drop (S6): place one unit from the FIRST carried stack
    /// onto the player's own tile. A pile of a different kind
    /// refuses the drop (state unchanged; the caller still
    /// flushes the turn — see the module docs).
    fn drop_item(&mut self, reg: &DefRegistry) {
        let Some(pos) = self.player_pos() else {
            return;
        };
        let Some((id, _)) = self.carried_first() else {
            return; // empty hands: no-op turn
        };
        let Some((top, count)) = self.tile_items(pos) else {
            return;
        };
        if top != 0 && top != id {
            return; // mixed pile: refused (documented rule)
        }
        if !self.carried_remove_one(id) {
            return;
        }
        self.set_tile_items(pos, id, count + 1);
        let name = Self::item_display_name(reg, id);
        self.messages.push(format!("You drop the {name}."));
    }
}

// ---------------------------------------------------------------------
// S5 turn processing: fields, then creatures. Called from
// `World::flush_turn` (the player's own action has already
// resolved). NOT called by `advance_clock` — the fast-forward
// is a harness time skip, not a per-second simulation, so
// fields and creatures do not tick during it.
// ---------------------------------------------------------------------

/// A tile's flammability class (S5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FuelClass {
    /// No FLAMMABLE* flag on either layer.
    None,
    /// FLAMMABLE or FLAMMABLE_ASH on terrain or furniture.
    Standard,
    /// FLAMMABLE_HARD only.
    Hard,
}

impl FuelClass {
    fn fuel_turns(self) -> i32 {
        match self {
            FuelClass::None => FUEL_NONE,
            FuelClass::Standard => FUEL_FLAMMABLE,
            FuelClass::Hard => FUEL_HARD,
        }
    }
}

impl World {
    /// Flammability of a tile: the game's fuel flags across
    /// the terrain + furniture layers. Upstream's tests are a
    /// disjunction (ter_furn_has_flag over FLAMMABLE /
    /// FLAMMABLE_ASH at the low power gate, FLAMMABLE_HARD at
    /// the high one — 0.I-1 map_field.cpp:908-910 for spread
    /// targets, 1051-1099 for fuel), so the easiest applicable
    /// class wins: FLAMMABLE furniture on a FLAMMABLE_HARD
    /// floor burns by the FLAMMABLE rule.
    fn fuel_class(&self, reg: &DefRegistry, pos: [i32; 3]) -> FuelClass {
        let has = |kind: Kind, id: u32, flag: &str| {
            id != 0
                && reg
                    .def_by_id(kind, id)
                    .is_some_and(|d| d.flags.iter().any(|f| f == flag))
        };
        let ter = self.terrain_at(pos).unwrap_or(0);
        let furn = self.furniture_at(pos).unwrap_or(0);
        let any = |flag: &str| {
            has(Kind::Terrain, ter, flag) || has(Kind::Furniture, furn, flag)
        };
        if any("FLAMMABLE") || any("FLAMMABLE_ASH") {
            FuelClass::Standard
        } else if any("FLAMMABLE_HARD") {
            FuelClass::Hard
        } else {
            FuelClass::None
        }
    }

    /// One turn of field processing (S5): fire lifecycle +
    /// spread, smoke production/drift/decay, and field damage
    /// to whoever stands in fire. Deterministic: tiles are
    /// processed in block/row-major order, neighbours in
    /// `DIRS` order, and the only randomness is the world's
    /// seeded RNG (spread rolls).
    ///
    /// The rule set compresses upstream's age-based fire
    /// processor (`field_processor_fd_fire`, 0.I-1
    /// map_field.cpp:945+) to gate timescale while keeping
    /// its structure:
    /// - A fire draws fuel from its tile (class-based budget;
    ///   see the FUEL_* constants). While fuel remains it can
    ///   GROW: +1 intensity per `FIRE_GROW_EVERY` turns, up to
    ///   a cap of 1 + (its tile is flammable) + the neighbour
    ///   rule — upstream's maximum intensity is 1 for a lone
    ///   fire, 2 with fuel or >= 3 burning neighbours, 3 with
    ///   >= 7 (map_field.cpp:1243-1261; the fuel path to 2 is
    ///   its age < -50 min branch).
    /// - When the fuel is spent, intensity decays
    ///   (`FIRE_DECAY_EVERY`) and the fire goes out, leaving
    ///   its smoke; the tile is marked burned (fuel gone for
    ///   good — the sim's stand-in for terrain consumption).
    /// - Spread: per neighbour per turn, upstream's chance
    ///   `25 * (intensity - 1)` rolled as rng(1,100) < chance
    ///   (map_field.cpp:1310-1330); the target must be
    ///   flammable and unburned; FLAMMABLE_HARD-only targets
    ///   need a raging (intensity 3) source, upstream's
    ///   power >= 3 gate. New fires start at intensity 1.
    /// - Smoke: a burning tile carries fd_smoke at the fire's
    ///   intensity (upstream creates it on the fire tile,
    ///   map_field.cpp:1417), pinned while the fire lives.
    ///   Free smoke follows `gas_spread_to`'s shape
    ///   (map_field.cpp:250-274): one intensity unit moves to
    ///   the downwind neighbour per turn when the source is
    ///   at intensity >= 2 and the destination is passable
    ///   with weaker/no smoke (upstream's percent-spread roll
    ///   and intensity > 1 gate, made deterministic); what
    ///   cannot drift decays (`SMOKE_DECAY_EVERY`).
    /// - Damage: standing in fd_fire costs `FIRE_DAMAGE` per
    ///   turn (creatures and the player alike).
    pub fn tick_fields(&mut self, reg: &DefRegistry) {
        let fire_id = reg.id_of(Kind::Field, "fd_fire");
        let smoke_id = reg.id_of(Kind::Field, "fd_smoke");
        if fire_id == 0 || smoke_id == 0 {
            return;
        }

        // Every field entry ages one turn (an entry's age
        // resets whenever its intensity changes).
        for b in &mut self.blocks {
            for t in &mut b.tiles {
                for meta in &mut t.field_meta {
                    meta.0 += 1;
                }
            }
        }

        // Field positions, in deterministic (block, row-major)
        // order, snapshotted before any mutation: entries
        // created this tick do not process until the next.
        let mut fire_tiles: Vec<[i32; 3]> = Vec::new();
        let mut smoke_tiles: Vec<[i32; 3]> = Vec::new();
        for b in &self.blocks {
            for (i, t) in b.tiles.iter().enumerate() {
                let pos = [
                    b.origin.x + (i % crate::scenes::SUBMAP) as i32,
                    b.origin.y + (i / crate::scenes::SUBMAP) as i32,
                    b.origin.z,
                ];
                if t.field_intensity(fire_id).is_some() {
                    fire_tiles.push(pos);
                }
                if t.field_intensity(smoke_id).is_some() {
                    smoke_tiles.push(pos);
                }
            }
        }
        let fire_set: std::collections::HashSet<[i32; 3]> =
            fire_tiles.iter().copied().collect();

        // --- Fire: fuel, growth, decay, smoke, spread. ---
        for &pos in &fire_tiles {
            let (b, i) = self.tile_at(pos).expect("field tile is in the scene");
            let class = self.fuel_class(reg, pos);
            let burning_neighbours = DIRS
                .iter()
                .filter(|&&(dx, dy)| fire_set.contains(&[pos[0] + dx, pos[1] + dy, pos[2]]))
                .count();
            let cap = if burning_neighbours >= 7 {
                3
            } else if burning_neighbours >= 3 || class != FuelClass::None {
                2
            } else {
                1
            };
            let mut died = false;
            {
                let t = &mut self.blocks[b].tiles[i];
                let fi = t.field_index(fire_id).expect("fire present");
                if t.field_meta[fi].1 < 0 {
                    t.field_meta[fi].1 = class.fuel_turns();
                }
                let (age, fuel) = t.field_meta[fi];
                let intensity = t.fields[fi].1;
                if fuel > 0 {
                    t.field_meta[fi].1 = fuel - 1;
                    if age >= FIRE_GROW_EVERY && intensity < cap {
                        t.set_field_at(fi, intensity + 1, None);
                    }
                } else if age >= FIRE_DECAY_EVERY {
                    if intensity <= 1 {
                        t.remove_field_at(fi);
                        died = true;
                    } else {
                        t.set_field_at(fi, intensity - 1, None);
                    }
                }
            }
            if died {
                self.burned.insert(pos);
                continue;
            }
            let intensity = self.blocks[b].tiles[i]
                .field_intensity(fire_id)
                .unwrap_or(0);
            // Smoke on the burning tile, at the fire's
            // intensity, pinned while the fire lives.
            {
                let t = &mut self.blocks[b].tiles[i];
                match t.field_index(smoke_id) {
                    Some(si) => t.set_field_at(si, intensity, None),
                    None => t.add_field(smoke_id, intensity, -1),
                }
            }
            // Spread.
            if intensity >= 2 {
                let chance = 25 * (intensity as i32 - 1);
                for (dx, dy) in DIRS {
                    let dst = [pos[0] + dx, pos[1] + dy, pos[2]];
                    if self.burned.contains(&dst) {
                        continue;
                    }
                    let Some((db, di)) = self.tile_at(dst) else {
                        continue;
                    };
                    if self.blocks[db].tiles[di].field_intensity(fire_id).is_some() {
                        continue;
                    }
                    let dclass = self.fuel_class(reg, dst);
                    if dclass == FuelClass::None {
                        continue;
                    }
                    if dclass == FuelClass::Hard && intensity < 3 {
                        continue;
                    }
                    if self.rng.roll_under(chance) {
                        self.blocks[db].tiles[di].add_field(fire_id, 1, -1);
                    }
                }
            }
        }

        // --- Smoke: downwind drift, else decay. ---
        for &pos in &smoke_tiles {
            let Some((b, i)) = self.tile_at(pos) else {
                continue;
            };
            // Pinned under a live fire (handled above).
            if self.blocks[b].tiles[i].field_intensity(fire_id).is_some() {
                continue;
            }
            let Some(si) = self.blocks[b].tiles[i].field_index(smoke_id) else {
                continue;
            };
            let intensity = self.blocks[b].tiles[i].fields[si].1;
            let age = self.blocks[b].tiles[i].field_meta[si].0;
            let mut moved = false;
            if intensity >= 2 {
                let dst = [pos[0] + WIND_STEP.0, pos[1] + WIND_STEP.1, pos[2]];
                if let Some((db, di)) = self.tile_at(dst) {
                    let dst_ok = self.passable(reg, dst)
                        && self.blocks[db].tiles[di].field_intensity(fire_id).is_none()
                        && self.blocks[db].tiles[di]
                            .field_intensity(smoke_id)
                            .is_none_or(|s| s < intensity);
                    if dst_ok {
                        match self.blocks[db].tiles[di].field_index(smoke_id) {
                            Some(dsi) => {
                                let cur = self.blocks[db].tiles[di].fields[dsi].1;
                                self.blocks[db].tiles[di]
                                    .set_field_at(dsi, (cur + 1).min(3), None);
                            }
                            None => {
                                self.blocks[db].tiles[di].add_field(smoke_id, 1, -1);
                            }
                        }
                        self.blocks[b].tiles[i].set_field_at(si, intensity - 1, None);
                        moved = true;
                    }
                }
            }
            if !moved && age >= SMOKE_DECAY_EVERY {
                let t = &mut self.blocks[b].tiles[i];
                if let Some(si) = t.field_index(smoke_id) {
                    let cur = t.fields[si].1;
                    if cur <= 1 {
                        t.remove_field_at(si);
                    } else {
                        t.set_field_at(si, cur - 1, None);
                    }
                }
            }
        }

        // --- Field damage: whoever stands in fire burns. ---
        let mut victims: Vec<(u64, i32)> = Vec::new();
        for e in &self.entities {
            let Some(p) = e.pos.as_ref() else { continue };
            let pos = [p.x, p.y, p.z];
            if let Some((b, i)) = self.tile_at(pos) {
                if let Some(intensity) = self.blocks[b].tiles[i].field_intensity(fire_id) {
                    let dmg = FIRE_DAMAGE[((intensity.max(1) - 1) as usize).min(2)];
                    victims.push((e.id, dmg));
                }
            }
        }
        for (id, dmg) in victims {
            self.damage_entity(reg, id, dmg);
        }
    }

    /// One turn of creature AI (S5), creatures in ascending
    /// id order (deterministic RNG draw order). Per creature:
    /// attack if adjacent to the player; else chase if it can
    /// see the player (flood LOS from the creature, and the
    /// player's tile lit != BLANK in the computed cache —
    /// §3.2's visibility test); else wander. See the module
    /// docs for the rule details.
    pub fn tick_creatures(&mut self, reg: &DefRegistry) {
        let Some(player_pos) = self.player_pos() else {
            return;
        };
        let Some(player_id) = self.player_id else {
            return;
        };
        let mut ids: Vec<u64> = self
            .entities
            .iter()
            .map(|e| e.id)
            .filter(|&id| id != player_id)
            .collect();
        ids.sort_unstable();
        for id in ids {
            let Some(pos) = self.entity_pos(id) else {
                continue; // died earlier this turn (field damage)
            };
            let dist = chebyshev(pos, player_pos);
            if dist <= 1 {
                // Attack: the def's expected melee damage,
                // HitFlash at the player + a Sound.
                let dmg = self
                    .entities
                    .iter()
                    .find(|e| e.id == id)
                    .and_then(|e| reg.def_by_id(Kind::Monster, e.type_def))
                    .map_or(0, |d| d.melee_expected());
                self.damage_entity(reg, player_id, dmg);
                self.push_effect(hit_flash(player_pos));
                self.push_effect(melee_sound(pos, player_pos));
            } else if visibility::has_line_of_sight(reg, &self.blocks, pos, player_pos)
                && self
                    .lit_at(player_pos)
                    .is_some_and(|l| l != LitLevel::BLANK)
            {
                // Greedy chase: the step minimising Chebyshev
                // distance (first in DIRS order on ties),
                // under the player's passability rule, into an
                // unoccupied tile. No door opening: a closed
                // door's tile is impassable, full stop.
                let mut best: Option<[i32; 3]> = None;
                let mut best_dist = dist;
                for (dx, dy) in DIRS {
                    let dst = [pos[0] + dx, pos[1] + dy, pos[2]];
                    let d = chebyshev(dst, player_pos);
                    if d >= best_dist {
                        continue;
                    }
                    if !self.passable(reg, dst) || self.entity_at(dst).is_some() {
                        continue;
                    }
                    best = Some(dst);
                    best_dist = d;
                }
                if let Some(dst) = best {
                    self.set_entity_pos(id, dst);
                }
            } else {
                // Wander: one seeded step (a roll of 8 = stay).
                let roll = self.rng.below(9);
                if roll < 8 {
                    let (dx, dy) = DIRS[roll as usize];
                    let dst = [pos[0] + dx, pos[1] + dy, pos[2]];
                    if self.passable(reg, dst) && self.entity_at(dst).is_none() {
                        self.set_entity_pos(id, dst);
                    }
                }
            }
        }
    }
}
