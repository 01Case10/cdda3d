//! Phase 11 follow-up gate: movecost scales turn length.
//!
//! Upstream model (0.I-1, grounded in the source):
//! - 1 calendar turn = 1 second = 100 move points
//!   (`time_duration::from_moves(t) = t/100` turns and
//!   `from_seconds(t) = t` turns, src/calendar.h:216-222).
//! - `do_turn()` advances `calendar::turn += 1_turns` once
//!   per call (src/do_turn.cpp:475) and every creature's
//!   move points replenish by its speed (`moves +=
//!   get_speed()`, src/creature.cpp:388); the sim's avatar
//!   is the unmodified baseline, speed 100.
//! - A walking step deducts `run_cost(combined_movecost
//!   (from, to))` points (src/game.cpp:10804-10814);
//!   `combined_movecost` (src/map.cpp:2466-2488) is
//!   `(move_cost(from) + move_cost(to)) * mult / 2` in raw
//!   JSON cost units, mult = 50 per differing axis (71 for
//!   a diagonal under trigdist). For the baseline avatar
//!   `run_cost` is the identity on that value (its diag
//!   divide/re-multiply by sqrt(2) cancels against the
//!   mult table), so a cardinal floor->floor step costs
//!   exactly (2+2)*25 = 100 points = 1 second.
//! - The player acts while moves > 0 (src/do_turn.cpp:
//!   564-605); when a step's cost drives the balance to
//!   <= 0, further do_turns elapse — one calendar second,
//!   one field tick (src/do_turn.cpp:655) and one monster
//!   round (:666) each — until the balance is positive
//!   again. The sim carries the balance across actions, so
//!   cost-250 steps alternate 2 s / 3 s from a partial
//!   balance exactly as upstream's aggregate timing does.
//!
//! This gate pins the observable consequence in calendar
//! seconds (`world.game_turn` deltas):
//! - floor->shrub (250 pts): 2 s; shrub->shrub (400 pts):
//!   4 s; shrub->floor (250 pts from balance 50): 3 s —
//!   9 s for the 3-step corridor vs 3 s on pure floor.
//! - Pure-floor walking is a steady state: 1 s per step,
//!   no drift over 10 steps (balance returns to 100).
//! - Diagonal floor steps cost (2+2)*71/2 = 142 pts: the
//!   first two take 1 s each (balance 100 -> 58 -> 16),
//!   the third takes 2 s (16 - 142 < 0 needs two refills).
//! - A blocked step and a wait stay 1 s and do not touch
//!   the move balance (scope pins: only successful
//!   movement steps scale — see rules.rs module docs).
//! - A stair z-move costs move_vertical's base 100 points
//!   (src/game.cpp:12331, deducted at :12621): 1 s from a
//!   full balance, like a floor step.
//!
//! Failing-first: pre-change every action advances the
//! clock exactly 1 s, so the corridor totals 3 s (not 9)
//! and the diagonals total 3 s (not 4). The floor-control
//! and z-move tests pass pre-change — they are pins
//! against over-scaling, not failing-first evidence.

use std::path::PathBuf;

use proto::{InputKind, InputT, MessageT};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn registry() -> DefRegistry {
    DefRegistry::load(&defs_path()).expect("registry loads")
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

fn world(reg: &DefRegistry, scene: &str) -> World {
    let scene = Scene::load_file(&scene_path(scene)).expect("scene loads");
    let mut w = World::from_scene(reg, &scene).expect("world builds");
    let _ = w.snapshot(reg);
    w
}

/// Press a key, return the calendar seconds the action took.
fn press_seconds(w: &mut World, reg: &DefRegistry, c: char) -> i64 {
    let before = w.game_turn;
    let _: Vec<MessageT> = w.apply_input(reg, &key_input(c));
    w.game_turn - before
}

#[test]
fn expensive_terrain_scales_step_time() {
    let reg = registry();
    let mut w = world(&reg, "shrub_walk");
    assert_eq!(w.player_pos(), Some([2, 5, 0]));

    // Blocked step north into the tree: 1 s (the standing
    // attempted-action convention), balance untouched —
    // proven by the first corridor step still taking 2 s.
    assert_eq!(press_seconds(&mut w, &reg, 'k'), 1, "blocked step = 1 s");
    assert_eq!(w.player_pos(), Some([2, 5, 0]));

    // floor -> shrub: (2+8)*25 = 250 points from balance
    // 100: -150 needs two refills -> 2 s, balance 50.
    assert_eq!(press_seconds(&mut w, &reg, 'l'), 2, "floor->shrub = 2 s");
    assert_eq!(w.player_pos(), Some([3, 5, 0]));

    // A wait stays 1 s and does not touch the balance
    // (proven by the next step's cost below).
    assert_eq!(press_seconds(&mut w, &reg, '.'), 1, "wait = 1 s");

    // shrub -> shrub: (8+8)*25 = 400 points from balance
    // 50: -350 needs four refills -> 4 s, balance 50.
    assert_eq!(press_seconds(&mut w, &reg, 'l'), 4, "shrub->shrub = 4 s");
    assert_eq!(w.player_pos(), Some([4, 5, 0]));

    // shrub -> floor: 250 points from balance 50: -200
    // needs three refills (0 is not > 0) -> 3 s, balance
    // back to exactly 100.
    assert_eq!(press_seconds(&mut w, &reg, 'l'), 3, "shrub->floor = 3 s");
    assert_eq!(w.player_pos(), Some([5, 5, 0]));

    // Steady state restored: the next floor step is 1 s.
    assert_eq!(press_seconds(&mut w, &reg, 'l'), 1, "floor again = 1 s");
}

#[test]
fn floor_walking_is_one_second_per_step_steady_state() {
    let reg = registry();
    let mut w = world(&reg, "shrub_walk");
    // Walk the all-floor row south of the corridor.
    assert_eq!(press_seconds(&mut w, &reg, 'j'), 1);
    for _ in 0..9 {
        assert_eq!(press_seconds(&mut w, &reg, 'l'), 1, "floor step = 1 s");
    }
    assert_eq!(w.player_pos(), Some([11, 6, 0]));
}

#[test]
fn diagonal_steps_cost_sqrt2_points() {
    let reg = registry();
    let mut w = world(&reg, "shrub_walk");
    // Southeast diagonals on pure floor: (2+2)*71/2 = 142
    // points each. Balances: 100 -> 58 -> 16 -> (2 s) 74.
    assert_eq!(press_seconds(&mut w, &reg, 'n'), 1, "diag 1 = 1 s");
    assert_eq!(press_seconds(&mut w, &reg, 'n'), 1, "diag 2 = 1 s");
    assert_eq!(press_seconds(&mut w, &reg, 'n'), 2, "diag 3 = 2 s");
    assert_eq!(w.player_pos(), Some([5, 8, 0]));
}

#[test]
fn z_move_costs_one_second_from_full_balance() {
    let reg = registry();
    let mut w = world(&reg, "multistorey");
    assert_eq!(w.player_pos(), Some([7, 7, 0]));
    // move_vertical's base cost is 100 points (game.cpp:
    // 12331, deducted at :12621) — a stair move from a
    // full balance takes the same 1 s as a floor step.
    assert_eq!(press_seconds(&mut w, &reg, '<'), 1, "z-move = 1 s");
    assert_eq!(w.player_pos(), Some([7, 7, 1]));
}
