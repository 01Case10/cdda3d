//! Phase 11 follow-up gate: §4.5's underground_lab — the
//! last starter scene. A fenced surface stairhead over a
//! sealed three-room lab at z -1, built for cave-factor /
//! no-sky tests.
//!
//! Surface expectations (noon, clear): the open ground of
//! the stairhead pad reads outside=true. THE STAIR TILE
//! ITSELF READS outside=FALSE — that is the honest value,
//! not a scene defect: t_stairs_down carries the INDOORS
//! flag in its own def, and upstream's build_outside_cache
//! (0.I-1 map.cpp:9294-9305) stamps an INDOORS tile and its
//! Chebyshev-1 neighbourhood as not outside, so the game
//! itself never reports a stair tile as outside. The sim's
//! port is faithful; this gate pins the faithful value so
//! a future def or model change surfaces loudly. (Same
//! shape as S8's basement correction: the sketch said one
//! thing, the source said another, the gate asserts what
//! the source produces.)
//!
//! Underground expectations (S8's settled semantics): the
//! round trip rides the z-move rule — `>` on t_stairs_down
//! descends, `<` on t_stairs_up ascends, and each z change
//! is a map shift (fresh Snapshot, epoch + 1). At z -1:
//! outside=false on all 144 tiles, lit in {LOW, BLANK}
//! (inside_light_level for visible tiles; apparent_light_at
//! never returns DARK), the player's tile LOW at noon.

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
    let scene = Scene::load_file(&scene_path("underground_lab")).expect("lab scene loads");
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

/// The (outside, lit) pair for one tile of one z level in a
/// Snapshot, from the block whose origin matches.
fn tile_in_snapshot(
    snap: &proto::SnapshotT,
    pos: [i32; 3],
) -> Option<(bool, LitLevel)> {
    for b in snap.blocks.as_ref()? {
        let o = b.origin.as_ref()?;
        if o.z != pos[2] {
            continue;
        }
        let lx = pos[0] - o.x;
        let ly = pos[1] - o.y;
        if !(0..12).contains(&lx) || !(0..12).contains(&ly) {
            continue;
        }
        let i = (ly * 12 + lx) as usize;
        let outside = b.outside.as_ref()?[i];
        let lit = b.lit.as_ref()?[i];
        return Some((outside, lit));
    }
    None
}

#[test]
fn lab_surface_stairhead_pad_is_outside_stair_tile_is_not() {
    let reg = registry();
    let mut w = world(&reg);
    assert_eq!(w.player_pos(), Some([4, 4, 0]), "player starts on the stair");
    let snap = w.snapshot(&reg);

    // Open ground well clear of the stair's INDOORS stamp:
    // outside in daylight.
    for pos in [[9, 9, 0], [4, 9, 0], [9, 4, 0]] {
        let (outside, lit) = tile_in_snapshot(&snap, pos).expect("surface tile present");
        assert!(outside, "open ground at {pos:?} must be outside=true at noon");
        assert_eq!(lit, LitLevel::BRIGHT, "open ground at {pos:?} is sunlit");
    }

    // The stair tile: outside=false by its own def's INDOORS
    // flag (see the module docs — upstream's verdict, pinned).
    let (outside, _) = tile_in_snapshot(&snap, [4, 4, 0]).expect("stair tile present");
    assert!(
        !outside,
        "t_stairs_down is INDOORS-flagged: outside=false even in an open field \
         (0.I-1 map.cpp build_outside_cache)"
    );
}

#[test]
fn lab_round_trip_and_sealed_basement_snapshot() {
    let reg = registry();
    let mut w = world(&reg);

    // '>' descends from t_stairs_down to the lab's
    // t_stairs_up at the same (x, y).
    let msgs = press(&mut w, &reg, '>');
    assert_eq!(w.player_pos(), Some([4, 4, -1]), "'>' must descend");
    assert_eq!(
        snapshot_epoch(&msgs),
        Some(2),
        "a z change is a map shift: fresh Snapshot, epoch + 1; got {msgs:?}"
    );

    // The z -1 block of the shift Snapshot: never outside,
    // never brighter than LOW, on all 144 tiles.
    let snap = match &msgs[0] {
        MessageT::Snapshot(s) => s,
        other => panic!("expected the shift Snapshot, got {other:?}"),
    };
    let mut lab_tiles = 0usize;
    for b in snap.blocks.as_ref().expect("blocks") {
        let o = b.origin.as_ref().expect("origin");
        if o.z != -1 {
            continue;
        }
        let outside = b.outside.as_ref().expect("outside array");
        let lit = b.lit.as_ref().expect("lit array");
        for i in 0..144usize {
            lab_tiles += 1;
            assert!(!outside[i], "lab tile {i} must be outside=false");
            assert!(
                matches!(lit[i], LitLevel::LOW | LitLevel::BLANK),
                "sealed lab tile {i} must be LOW or BLANK (no sun, no sources), got {:?}",
                lit[i]
            );
        }
    }
    assert_eq!(lab_tiles, 144, "one lab submap in the snapshot");
    assert_eq!(
        w.lit_at([4, 4, -1]),
        Some(LitLevel::LOW),
        "the tile the player stands on in the sealed lab reads LOW \
         (inside_light_level), not DARK"
    );

    // '<' on the lab's t_stairs_up climbs back to the pad.
    let msgs = press(&mut w, &reg, '<');
    assert_eq!(w.player_pos(), Some([4, 4, 0]));
    assert_eq!(snapshot_epoch(&msgs), Some(3));
}

#[test]
fn lab_rooms_connect_through_their_doors() {
    let reg = registry();
    let mut w = world(&reg);
    press(&mut w, &reg, '>');
    assert_eq!(w.player_pos(), Some([4, 4, -1]));

    // East room: south to (4,5), east to (5,5), open the
    // door at (6,5), step through to (7,5).
    press(&mut w, &reg, 'j');
    press(&mut w, &reg, 'l');
    press(&mut w, &reg, 'o');
    press(&mut w, &reg, 'l'); // open
    press(&mut w, &reg, 'l'); // onto the door tile
    press(&mut w, &reg, 'l'); // into the east room
    assert_eq!(w.player_pos(), Some([7, 5, -1]), "east room reached");

    // Back to the entry room, then south through the door
    // at (4,6) into the south-west room.
    press(&mut w, &reg, 'h');
    press(&mut w, &reg, 'h');
    press(&mut w, &reg, 'h');
    assert_eq!(w.player_pos(), Some([4, 5, -1]));
    press(&mut w, &reg, 'o');
    press(&mut w, &reg, 'j'); // open
    press(&mut w, &reg, 'j'); // onto the door tile
    press(&mut w, &reg, 'j'); // into the south-west room
    assert_eq!(w.player_pos(), Some([4, 7, -1]), "south-west room reached");
}
