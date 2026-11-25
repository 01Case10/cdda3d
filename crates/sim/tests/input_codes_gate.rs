//! Input key-code gate (Launch-Sim symptoms 4/5, sim half):
//! the special key codes the window sends for non-character
//! keys must work on the map.
//!
//! - Arrow codes (KEY_LEFT/RIGHT/UP/DOWN) are directions:
//!   they move the player and they satisfy an armed pending
//!   action (o + Down opens the door). Before the fix the
//!   codes had no char form, so MapInput dropped them into
//!   the no-op arm: arrows never moved the player, and an
//!   arrow after `o` cancelled the pending action.
//! - KEY_ESCAPE on the map is not a turn: it cancels an
//!   armed pending action and returns no flush. Before the
//!   fix it fell through to the no-op-turn arm and advanced
//!   the clock (and in UiInput mode it already closed the
//!   menu — that path is ui_gate's).
//!
//! Driven in-process through `World::apply_input`, the same
//! entry point the server loop uses.

use std::path::PathBuf;

use proto::{InputKind, InputT, MessageT};
use sim::{DefRegistry, Scene, World};
use ui_share::input::{KEY_DOWN, KEY_ESCAPE, KEY_LEFT};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}
fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn code_input(key_code: i32) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = key_code;
    input.pressed = true;
    input
}

fn street_block() -> (DefRegistry, World) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = Scene::load_file(&scene_path("street_block")).expect("scene loads");
    let world = World::from_scene(&reg, &scene).expect("world builds");
    (reg, world)
}

#[test]
fn arrow_keys_move_the_player() {
    let (reg, mut world) = street_block();
    assert_eq!(world.player_pos(), Some([5, 5, 0]), "street_block start");
    let flush = world.apply_input(&reg, &code_input(KEY_DOWN));
    assert!(!flush.is_empty(), "an arrow move is a turn (flush expected)");
    assert_eq!(world.player_pos(), Some([5, 6, 0]), "Down steps south");
    let flush = world.apply_input(&reg, &code_input(KEY_LEFT));
    assert!(!flush.is_empty());
    assert_eq!(world.player_pos(), Some([4, 6, 0]), "Left steps west");
}

#[test]
fn arrow_key_satisfies_a_pending_open() {
    let (reg, mut world) = street_block();
    // The proven walk, in arrows: Down Down to the door at
    // (5,8), arm Open with 'o', aim it with the Down arrow,
    // then step through onto the door tile.
    world.apply_input(&reg, &code_input(KEY_DOWN));
    world.apply_input(&reg, &code_input(KEY_DOWN));
    assert_eq!(world.player_pos(), Some([5, 7, 0]), "at the door");
    let flush = world.apply_input(&reg, &code_input('o' as i32));
    assert!(flush.is_empty(), "arming Open is not a turn");
    world.apply_input(&reg, &code_input(KEY_DOWN));
    world.apply_input(&reg, &code_input(KEY_DOWN));
    assert_eq!(
        world.player_pos(),
        Some([5, 8, 0]),
        "the player stands in the opened doorway (the arrow aimed the Open)"
    );
}

#[test]
fn escape_on_the_map_is_not_a_turn() {
    let (reg, mut world) = street_block();
    let flush = world.apply_input(&reg, &code_input(KEY_ESCAPE));
    assert!(
        flush.is_empty(),
        "Esc on the map produces no flush (no turn consumed), got {} messages",
        flush.len()
    );
    assert_eq!(world.player_pos(), Some([5, 5, 0]));
    // Esc also cancels an armed pending action: o, Esc, j
    // must MOVE (not open), and the door stays shut — the
    // next j bumps it instead of stepping through.
    world.apply_input(&reg, &code_input('o' as i32));
    let flush = world.apply_input(&reg, &code_input(KEY_ESCAPE));
    assert!(flush.is_empty(), "Esc cancelling a pending action is not a turn");
    world.apply_input(&reg, &code_input('j' as i32));
    world.apply_input(&reg, &code_input('j' as i32));
    let flush = world.apply_input(&reg, &code_input('j' as i32));
    assert!(
        flush.iter().any(|m| matches!(m, MessageT::Delta(_))),
        "the bump turn still flushes"
    );
    assert_eq!(
        world.player_pos(),
        Some([5, 7, 0]),
        "player walked to the (still closed) door and bumped it"
    );
}
