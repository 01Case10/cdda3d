//! Phase 11 follow-up gate, engine half: §4.5's
//! underground_lab through the engine's live sky paths —
//! `lighting::proto_adapter::grid_from_snapshot` (wire ->
//! LightGrid), `engine::frame::SkyExposure::from_grid` (the
//! §3.4 sky-exposure channel both renderers consume), and
//! `SkyScene::cave_factor_for` (the decision-3C cave
//! gradient seating rule).
//!
//! What this pins: from the wire grid of a real sim session
//! (player descended into the lab), EVERY z -1 tile the
//! grid stores reads sky factor 0 — the sky does not reach
//! the lab at all — and the cave-factor inputs hold exactly
//! as the live loop's rule states them: below ground AND
//! sheltered engages the gradient (factor 1 at the lab
//! tile), while the surface never engages it, not even at
//! the sheltered stair tile.
//!
//! Grid level, not pixels — the standing CaptureArgs
//! deviation (see sim_visibility_gate.rs): captures take
//! fixture names only, so the engine-side assertion lives
//! on the grids the renderers sample.

use std::path::PathBuf;

use engine::frame::{SkyExposure, SkyScene};
use lighting::proto_adapter::grid_from_snapshot;
use proto::{InputKind, InputT};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../sim/scenes/{name}.toml"))
}

fn key(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

#[test]
fn lab_tiles_have_zero_sky_exposure_and_engage_the_cave_factor() {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = Scene::load_file(&scene_path("underground_lab")).expect("lab scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    let _ = world.snapshot(&reg);

    // Descend; the z change produces the shift Snapshot the
    // live loop would seat its sky from.
    let msgs = world.apply_input(&reg, &key('>'));
    assert_eq!(world.player_pos(), Some([4, 4, -1]));
    let snap = match &msgs[0] {
        proto::MessageT::Snapshot(s) => s,
        other => panic!("expected the shift Snapshot, got {other:?}"),
    };
    let grid = grid_from_snapshot(snap);
    let exposure = SkyExposure::from_grid(&grid);

    // Every z -1 tile the grid stores: wire outside=false,
    // sky factor exactly 0.
    let mut lab_tiles = 0usize;
    for (pos, t) in grid.iter_tiles() {
        if pos[2] != -1 {
            continue;
        }
        lab_tiles += 1;
        assert!(!t.outside, "lab tile {pos:?}: wire outside must be false");
        assert_eq!(
            exposure.factor(pos),
            0.0,
            "lab tile {pos:?}: sky exposure must be 0"
        );
    }
    assert_eq!(lab_tiles, 144, "the grid stores the whole lab submap");

    // The cave-factor inputs (§4 seating rule): z < 0 AND
    // sheltered engages the dark cave gradient...
    assert_eq!(
        SkyScene::cave_factor_for(&exposure, [4, 4, -1]),
        1.0,
        "standing in the lab: cave gradient engaged"
    );
    // ...the surface never engages it — not even at the
    // stair tile, which is sheltered (its def is
    // INDOORS-flagged) but at z 0: the gradient is a cave
    // treatment, not a roof treatment.
    assert_eq!(
        SkyScene::cave_factor_for(&exposure, [4, 4, 0]),
        0.0,
        "the surface stairhead keeps the surface sky"
    );

    // And the open pad around the stairhead is genuinely
    // sky-exposed in the same grid — the lab's zero is a
    // property of the lab, not of the feed.
    assert_eq!(
        exposure.factor([9, 9, 0]),
        1.0,
        "open surface ground: sky exposure 1"
    );
}
