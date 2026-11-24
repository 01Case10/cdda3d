//! S3 gate, T1 render half: the engine consumes the sim's
//! computed wire grids through the same paths it uses for the
//! real game — `lighting::proto_adapter::grid_from_snapshot`
//! (the wire -> LightGrid conversion) and
//! `game_environment_from_snapshot` + `sky_state` (the wire
//! environment -> sky/GI state).
//!
//! DEVIATION (recorded, S1 precedent): full pixel captures of a
//! sim-fed snapshot remain impossible without an engine change
//! — CaptureArgs takes fixture names only, there is no
//! wire-snapshot source variant. This gate is the strongest
//! feasible engine-side assertion instead: the LightGrid the
//! renderers sample carries the sim's outside/lit verbatim, the
//! presented intensity of an interior tile is strictly below
//! the exterior's at noon, visibility classes follow the wire,
//! and the SkyState derived from the sim's EnvironmentUpdate
//! has the sun up at noon and down at midnight.

use std::path::PathBuf;

use lighting::mapping::{presented_sample, visibility_of, Visibility};
use lighting::proto_adapter::{game_environment_from_snapshot, grid_from_snapshot};
use lighting::sky::sky_state;
use proto::LitLevel;
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../sim/scenes/{name}.toml"))
}

fn build(scene_name: &str, mutate: impl FnOnce(&mut Scene)) -> (World, DefRegistry) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let mut scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
    mutate(&mut scene);
    let world = World::from_scene(&reg, &scene).expect("world builds");
    (world, reg)
}

#[test]
fn t1_render_half_noon() {
    let (mut world, reg) = build("street_block", |_| {});
    let snap = world.snapshot(&reg);
    let grid = grid_from_snapshot(&snap);

    // The wire outside grid reaches TileLight verbatim.
    let interior = grid.get(5, 5, 0);
    assert!(!interior.outside, "interior tile: not sky-exposed");
    assert_eq!(interior.lit, LitLevel::LOW);
    let exterior = grid.get(5, 9, 0);
    assert!(exterior.outside, "yard tile: sky-exposed");
    assert_eq!(exterior.lit, LitLevel::BRIGHT);

    // Interior dims vs exterior through the presentation mapping.
    let in_i = presented_sample(interior).intensity;
    let out_i = presented_sample(exterior).intensity;
    assert!(
        in_i < out_i,
        "interior presented intensity {in_i} < exterior {out_i} at noon"
    );
    assert_eq!(
        visibility_of(interior.lit, interior.seen),
        Visibility::Visible
    );
    assert_eq!(
        visibility_of(exterior.lit, exterior.seen),
        Visibility::Visible
    );

    // The sim's EnvironmentUpdate drives a noon SkyState.
    let env = game_environment_from_snapshot(&snap).expect("environment present");
    let sky = sky_state(&env);
    assert!(
        sky.sun_intensity > 0.5,
        "noon sun is up (intensity {})",
        sky.sun_intensity
    );
    assert!(
        sky.cloud_cover < 0.1,
        "clear weather: ~no cloud cover (got {})",
        sky.cloud_cover
    );
}

#[test]
fn t1_render_half_midnight_and_memory() {
    // Midnight street_block: the lamp pool is Visible in the
    // grid; the roofed house's basement is Hidden; a memorized
    // room (three_rooms, after the visit) reads Memorized.
    let (mut world, reg) = build("street_block", |s| {
        s.time = "midnight".to_string();
    });
    let snap = world.snapshot(&reg);
    let grid = grid_from_snapshot(&snap);
    let pool = grid.get(13, 9, 0);
    assert_eq!(pool.lit, LitLevel::LOW);
    assert_eq!(visibility_of(pool.lit, pool.seen), Visibility::Visible);
    let lamp = grid.get(12, 9, 0);
    assert_eq!(lamp.lit, LitLevel::BRIGHT);

    let (mut world, reg) = build("roofed_house_explicit", |_| {});
    let snap = world.snapshot(&reg);
    let grid = grid_from_snapshot(&snap);
    let basement = grid.get(6, 6, -1);
    assert_eq!(basement.lit, LitLevel::BLANK);
    assert_eq!(
        visibility_of(basement.lit, basement.seen),
        Visibility::Hidden,
        "unseen basement is Hidden in the grid"
    );

    // three_rooms: visit the middle room and leave; the fresh
    // snapshot's memory blocks must surface in the grid as
    // Memorized tiles.
    let (mut world, reg) = build("three_rooms", |_| {});
    let _ = world.snapshot(&reg);
    let key = |world: &mut World, c: char| {
        let mut input = proto::InputT::default();
        input.kind = proto::InputKind::Key;
        input.key_code = c as i32;
        input.pressed = true;
        world.apply_input(&reg, &input);
    };
    // (S6 amendment: east steps use the vi-key `l` — `d` is
    // the drop key since S6. Same walk.)
    for _ in 0..4 {
        key(&mut world, 'l');
    }
    key(&mut world, 'o');
    key(&mut world, 'l'); // open
    for _ in 0..6 {
        key(&mut world, 'l');
    }
    for _ in 0..6 {
        key(&mut world, 'a');
    }
    key(&mut world, 'c');
    key(&mut world, 'l'); // close
    let snap = world.build_snapshot(&reg, 0, 0);
    assert!(snap.memory_blocks.is_some(), "the visit left memory blocks");
    let grid = grid_from_snapshot(&snap);
    let mem = grid.get(10, 5, 0);
    assert_eq!(
        visibility_of(mem.lit, mem.seen),
        Visibility::Memorized,
        "the left-behind room reads Memorized through the engine's grid"
    );
    // The never-visited room: on the WIRE it is BLANK in both
    // the live block and the memory block (the exporter's own
    // contract, voxel_exporter.cpp:164-167 — never-memorized
    // tiles in a partial memory block stay BLANK).
    //
    // FOUND DIVERGENCE (engine side, not fixed here — the
    // lighting crate is another thread's territory):
    // grid_from_snapshot's apply_block stamps MEMORIZED on
    // EVERY tile of a memory block, ignoring the block's lit
    // array, so through the grid this tile currently reads
    // Memorized instead of Hidden. The fixtures never exposed
    // it (the stub house has no memory blocks at all). The
    // wire-level assertion below is the contract the sim is
    // responsible for.
    let mb = snap
        .memory_blocks
        .as_ref()
        .unwrap()
        .iter()
        .find(|b| {
            let o = b.origin.as_ref().unwrap();
            o.x == 12 && o.y == 0 && o.z == 0
        })
        .expect("memory block for the east submap");
    let i = (5 * 12 + (20 - 12)) as usize;
    assert_eq!(
        mb.lit.as_ref().unwrap()[i],
        LitLevel::BLANK,
        "never-visited room: BLANK in the memory block (wire contract)"
    );
    let live_block = snap
        .blocks
        .as_ref()
        .unwrap()
        .iter()
        .find(|b| {
            let o = b.origin.as_ref().unwrap();
            o.x == 12 && o.y == 0 && o.z == 0
        })
        .expect("live block for the east submap");
    assert_eq!(
        live_block.lit.as_ref().unwrap()[i],
        LitLevel::BLANK,
        "never-visited room: BLANK in the live block (wire contract)"
    );
}
