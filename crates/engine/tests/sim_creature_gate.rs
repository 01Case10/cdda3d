//! S5 gate, T3 engine half: the engine consumes the sim's
//! creature wire through its own paths — `billboard`'s
//! FakeEntity mirror + `billboard::scene::build_frame` (the
//! composed-billboard path `engine::frame` draws) for the
//! in-view spawn, and `lighting::proto_adapter::
//! grid_from_snapshot` + `visibility_of` (the renderers'
//! visibility verdict) for the occluded and dark spawns.
//!
//! Grid/state level, not pixels — the CaptureArgs limitation
//! stands (S1/S3 precedent): a billboard instance is built
//! for the visible zombie at its wire position, and the
//! behind-wall / in-dark zombies' tiles read Hidden in the
//! engine's LightGrid, which is what keeps their billboards
//! off screen in the live loop.
//!
//! The chase is followed through the Delta stream the way
//! the live loop ingests it: entity_updates upsert, and the
//! converted FakeEntity tracks the sim's positions.

use std::collections::HashMap;
use std::path::PathBuf;

use billboard::atlas::fixture_atlas;
use billboard::composite::Facing as BillboardFacing;
use billboard::instance::{tile_feet, FakeEntity};
use billboard::render::Camera;
use lighting::mapping::{visibility_of, Visibility};
use lighting::proto_adapter::grid_from_snapshot;
use proto::{EntityT, Facing, InputKind, InputT, MessageT};
use sim::scenes::Spawn;
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
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    world.snapshot(&reg);
    (world, reg)
}

fn zombie_spawn(pos: [i32; 3]) -> Spawn {
    Spawn {
        type_id: "mon_zombie".to_string(),
        pos,
        hp_frac: 255,
        facing: "NONE".to_string(),
        overlays: Vec::new(),
        flags: 0,
        sprite: None,
    }
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

/// The wire Entity -> billboard mirror conversion (field for
/// field; the shape the live wiring performs).
fn to_fake(e: &EntityT) -> FakeEntity {
    let p = e.pos.as_ref().expect("entity pos");
    FakeEntity {
        id: e.id,
        sprite_id: e.sprite_id.clone().unwrap_or_default(),
        pos: [p.x, p.y, p.z],
        facing: match e.facing {
            Facing::LEFT => BillboardFacing::Left,
            Facing::RIGHT => BillboardFacing::Right,
            _ => BillboardFacing::None,
        },
        overlays: e.overlays.clone().unwrap_or_default(),
        hp_frac: e.hp_frac,
        flags: e.flags,
        under_roof: false,
        bob_phase: e.id as f32 * 0.7,
        last_hit_s: -100.0,
    }
}

#[test]
fn t3_engine_billboard_for_in_view_spawn() {
    let (world, _reg) = build("street_block", |s| {
        s.spawns.push(zombie_spawn([12, 9, 0]));
    });
    let snap = world.build_snapshot(&_reg, 0, 0);
    let entities: Vec<FakeEntity> = snap
        .entities
        .as_ref()
        .expect("entities")
        .iter()
        .map(to_fake)
        .collect();
    let zombie = entities
        .iter()
        .find(|e| e.sprite_id == "mon_zombie")
        .expect("zombie converts to a FakeEntity");
    assert_eq!(zombie.pos, [12, 9, 0]);

    // Its tile is Visible in the engine's LightGrid (the
    // player is inside the house; the yard row is lit and
    // seen at noon) — wait: visibility is from the player's
    // viewpoint; (12,9) is outside, across the yard, visible
    // through the window/door line at noon per the sim's LOS.
    let grid = grid_from_snapshot(&snap);
    let tile = grid.get(12, 9, 0);
    assert_eq!(
        visibility_of(tile.lit, tile.seen),
        Visibility::Visible,
        "in-view spawn tile is Visible in the engine grid"
    );

    // The composed billboard path builds an instance for it,
    // anchored at the wire position.
    let atlas = fixture_atlas();
    let cam = Camera {
        target: tile_feet([9, 9, 0]),
        yaw_deg: 180.0,
        pitch_deg: 34.0,
        distance: 18.0,
    };
    let frame = billboard::scene::build_frame(&atlas, &entities, &cam, 0.0);
    let inst = frame
        .instances
        .iter()
        .find(|i| i.entity_id == zombie.id)
        .expect("a billboard instance is built for the in-view zombie");
    let feet = tile_feet([12, 9, 0]);
    assert!(
        (inst.anchor.x - feet.x).abs() < 0.6 && (inst.anchor.z - feet.z).abs() < 0.6,
        "instance anchored at the wire position: {:?} vs feet {feet:?}",
        inst.anchor
    );
}

#[test]
fn t3_engine_occluded_and_dark_spawns_are_hidden() {
    // Behind a wall: three_rooms east room, door never opened.
    let (world, reg) = build("three_rooms", |s| {
        s.spawns.push(zombie_spawn([19, 5, 0]));
    });
    let snap = world.build_snapshot(&reg, 0, 0);
    assert!(
        snap.entities
            .as_ref()
            .is_some_and(|es| es.iter().any(|e| e.sprite_id.as_deref() == Some("mon_zombie"))),
        "the occluded zombie is on the wire"
    );
    let grid = grid_from_snapshot(&snap);
    let tile = grid.get(19, 5, 0);
    assert_eq!(
        visibility_of(tile.lit, tile.seen),
        Visibility::Hidden,
        "behind-wall spawn tile is Hidden in the engine grid"
    );

    // In the dark: three_rooms at midnight, middle room.
    let (world, reg) = build("three_rooms", |s| {
        s.time = "midnight".to_string();
        s.spawns.push(zombie_spawn([12, 5, 0]));
    });
    let snap = world.build_snapshot(&reg, 0, 0);
    let grid = grid_from_snapshot(&snap);
    let tile = grid.get(12, 5, 0);
    assert_eq!(
        visibility_of(tile.lit, tile.seen),
        Visibility::Hidden,
        "dark spawn tile is Hidden in the engine grid"
    );
}

#[test]
fn t3_engine_chase_through_delta_ingest() {
    let (mut world, reg) = build("street_block", |s| {
        s.spawns.push(zombie_spawn([12, 9, 0]));
    });
    // Live-loop style ingest: a gate-side entity store fed
    // only by Snapshot/Delta entity fields.
    let snap = world.build_snapshot(&reg, 0, 0);
    let mut store: HashMap<u64, FakeEntity> = snap
        .entities
        .as_ref()
        .expect("entities")
        .iter()
        .map(|e| (e.id, to_fake(e)))
        .collect();
    let zid = store
        .values()
        .find(|e| e.sprite_id == "mon_zombie")
        .map(|e| e.id)
        .expect("zombie in store");
    let start_x = store[&zid].pos[0];

    // Walk out and wait; apply each flush's entity fields.
    for c in ['s', 's', 'o', 's', 's', 's', '.', '.', '.'] {
        let msgs = world.apply_input(&reg, &key_input(c));
        for m in &msgs {
            match m {
                MessageT::Delta(d) => {
                    for e in d.entity_updates.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                        store.insert(e.id, to_fake(e));
                    }
                    for id in d.entity_removed.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                        store.remove(id);
                    }
                }
                MessageT::Snapshot(s) => {
                    store = s
                        .entities
                        .as_ref()
                        .map(|es| es.iter().map(|e| (e.id, to_fake(e))).collect())
                        .unwrap_or_default();
                }
                _ => {}
            }
        }
    }
    let end = store[&zid].pos;
    assert!(
        end[0] < start_x,
        "the ingested zombie chased west: {start_x} -> {}",
        end[0]
    );
    assert_eq!(store[&zid].facing, BillboardFacing::Left, "chase faces LEFT");
}
