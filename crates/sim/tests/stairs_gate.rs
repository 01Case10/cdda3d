//! S8 gate: T9 — stairs & multi-storey.
//!
//! The z-move rule (plan §3.2, settled in S8): `>` descends
//! and `<` ascends — CDDA's own convention (0.I-1 src/
//! action.cpp:498-501 maps the `move_down` / `move_up`
//! actions; src/game.cpp `move_vertical` requires
//! TFLAG_GOES_DOWN at the player's tile for movez -1
//! (game.cpp:12430) and TFLAG_GOES_UP for movez +1
//! (game.cpp:12336); src/mapdata.h:184-185 documents the
//! flags with exactly these keys). The destination is the
//! same (x, y) one level over; it must be passable and must
//! not be NO_FLOOR terrain (you land on a floor/stair, not
//! into open air). Anything else is a no-op turn (the S2
//! attempted-action convention). A z change crosses a submap
//! boundary (submap index includes z), so the flush rides
//! the existing map-shift path: fresh Snapshot, epoch + 1 —
//! the "snapshot window recentres" of the contract, in the
//! sim's no-culling form (a Snapshot carries all scene
//! submaps; see world.rs `build_snapshot`).
//!
//! Basement light (the T9 sketch said "lit DARK" — checked
//! against the 0.I-1 source in S8 and corrected): the fork
//! exporter takes lit from `map::apparent_light_at`
//! (lightmap.cpp:801), which NEVER returns DARK (DARK comes
//! from `map::light_at`, which the exporter does not use —
//! see visibility.rs module docs). A sealed basement is
//! filled at `inside_light_level` = LIGHT_AMBIENT_LOW (3.5)
//! by `build_sunlight_cache` (lightmap.cpp:340-345, the
//! fully-inside fill), and 3.5 >= the vision threshold at
//! that ambient (~3.22), so a VISIBLE basement tile reads
//! LOW, not DARK. The honest T9 assertions are therefore:
//! outside=false on every basement tile, no tile brighter
//! than LOW (no sunlight reaches the sealed level — the
//! ported model gives z < 0 no direct sun and no sun cast),
//! and the player's own tile LOW once standing in it. The
//! plan's T9 row was amended in S8 to say exactly this.
//!
//! Failing-first: pre-S8 the keymap has no `>` / `<` handling
//! at all, so every z assertion below fails (the player
//! stays at z 0 and the flush is a plain Delta).

use std::path::PathBuf;

use proto::{InputKind, InputT, LitLevel, MessageT};
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

fn world(reg: &DefRegistry) -> World {
    let scene = Scene::load_file(&scene_path("multistorey")).expect("multistorey scene loads");
    let mut w = World::from_scene(reg, &scene).expect("world builds");
    // The session's opening Snapshot (what the server sends).
    let _ = w.snapshot(reg);
    w
}

fn press(w: &mut World, reg: &DefRegistry, c: char) -> Vec<MessageT> {
    w.apply_input(reg, &key_input(c))
}

fn snapshot_epoch(msgs: &[MessageT]) -> Option<u64> {
    msgs.iter().find_map(|m| match m {
        MessageT::Snapshot(s) => Some(s.epoch),
        _ => None,
    })
}

#[test]
fn t9_up_then_down_stair_b() {
    let reg = registry();
    let mut w = world(&reg);
    assert_eq!(w.player_pos(), Some([7, 7, 0]), "player starts on stair B");

    // '<' ascends from t_stairs_up (GOES_UP) to the first
    // floor's t_stairs_down at the same (x, y).
    let msgs = press(&mut w, &reg, '<');
    assert_eq!(w.player_pos(), Some([7, 7, 1]), "'<' must move the player up");
    assert_eq!(
        snapshot_epoch(&msgs),
        Some(2),
        "a z change is a map shift: fresh Snapshot, epoch + 1; got {msgs:?}"
    );

    // '>' descends back from t_stairs_down (GOES_DOWN).
    let msgs = press(&mut w, &reg, '>');
    assert_eq!(w.player_pos(), Some([7, 7, 0]));
    assert_eq!(snapshot_epoch(&msgs), Some(3));
}

#[test]
fn t9_down_to_basement_and_back() {
    let reg = registry();
    let mut w = world(&reg);
    // Walk to stair A at (3,3): west x4, north x4.
    for c in ['h', 'h', 'h', 'h', 'k', 'k', 'k', 'k'] {
        press(&mut w, &reg, c);
    }
    assert_eq!(w.player_pos(), Some([3, 3, 0]));
    let msgs = press(&mut w, &reg, '>');
    assert_eq!(w.player_pos(), Some([3, 3, -1]), "'>' must descend");
    assert_eq!(snapshot_epoch(&msgs), Some(2));

    // Basement assertions over the fresh Snapshot the shift
    // produced: outside=false everywhere on z -1, nothing
    // brighter than LOW, the player's tile LOW.
    let snap = match &msgs[0] {
        MessageT::Snapshot(s) => s,
        other => panic!("expected the shift Snapshot, got {other:?}"),
    };
    let mut basement_tiles = 0usize;
    for b in snap.blocks.as_ref().expect("blocks") {
        let o = b.origin.as_ref().expect("origin");
        if o.z != -1 {
            continue;
        }
        let outside = b.outside.as_ref().expect("outside array");
        let lit = b.lit.as_ref().expect("lit array");
        for i in 0..144usize {
            basement_tiles += 1;
            assert!(!outside[i], "basement tile {i} must be outside=false");
            assert!(
                matches!(lit[i], LitLevel::LOW | LitLevel::BLANK),
                "sealed basement tile {i} must be LOW or BLANK (no sun), got {:?}",
                lit[i]
            );
        }
    }
    assert_eq!(basement_tiles, 144, "one basement submap in the snapshot");
    assert_eq!(
        w.lit_at([3, 3, -1]),
        Some(LitLevel::LOW),
        "the tile the player stands on in the sealed basement reads LOW \
         (inside_light_level 3.5 >= vision threshold), not DARK"
    );

    // '<' on the basement's t_stairs_up climbs back.
    let msgs = press(&mut w, &reg, '<');
    assert_eq!(w.player_pos(), Some([3, 3, 0]));
    assert_eq!(snapshot_epoch(&msgs), Some(3));
}

#[test]
fn t9_z_move_off_stairs_is_a_noop_turn() {
    let reg = registry();
    let mut w = world(&reg);
    // Step off the stair onto plain floor, then try both keys.
    press(&mut w, &reg, 'h');
    assert_eq!(w.player_pos(), Some([6, 7, 0]));
    let msgs = press(&mut w, &reg, '>');
    assert_eq!(w.player_pos(), Some([6, 7, 0]), "no GOES_DOWN here: no-op");
    assert!(
        snapshot_epoch(&msgs).is_none(),
        "a no-op z attempt is a plain turn (Delta), not a shift"
    );
    let msgs = press(&mut w, &reg, '<');
    assert_eq!(w.player_pos(), Some([6, 7, 0]), "no GOES_UP here: no-op");
    assert!(snapshot_epoch(&msgs).is_none());
}

#[test]
fn t9_wrong_direction_on_a_stair_is_a_noop() {
    let reg = registry();
    let mut w = world(&reg);
    // The player starts on t_stairs_up: '>' (down) must NOT
    // descend — the tile carries GOES_UP only.
    let msgs = press(&mut w, &reg, '>');
    assert_eq!(w.player_pos(), Some([7, 7, 0]));
    assert!(snapshot_epoch(&msgs).is_none());
}
