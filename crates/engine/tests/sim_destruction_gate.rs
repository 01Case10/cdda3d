//! S6 gate, T4/T11 engine half: the engine consumes the
//! sim's destruction + vehicle wire through its own ingest
//! paths — `snapshot_world::build_snapshot_world` +
//! `stamp_furniture` / `restamp_tile` over a `FineLayer` (the
//! Phase 9 prop path), and `vehicles::store::VehicleStore` +
//! `vehicles::mesh::{stamp_vehicle, clear_vehicle}` (the
//! live loop's vehicle path).
//!
//! - T4 fine-layer half: the destructible_street table
//!   (f_table at (4,4)) is stamped as a fine prop volume from
//!   the sim Snapshot; after the sim's bash Delta removes the
//!   furniture, restamping the tile through the Delta's
//!   TileChange leaves NO volume behind — no ghost prop.
//! - T4 vehicle half / T11: the fixture car stamps its 9
//!   tiles; the roof part stamps the top block, a wheel-only
//!   tile leaves the top block AIR; damaging the door to 0 hp
//!   through the sim's World API flips the engine model's
//!   `broken` flag via the Delta; rotating 23 -> 90 degrees
//!   and restamping (clear previous + stamp current, the
//!   live loop's shape) moves the parts rigidly with the roof
//!   still on top; removing parts unstamps their tiles.
//!
//! State level, not pixels — the CaptureArgs limitation
//! stands (S1/S3/S5 precedent).

use std::collections::BTreeSet;
use std::path::PathBuf;

use engine::materials::PropModelData;
use engine::snapshot_world::{
    build_snapshot_world, prop_block_for, restamp_tile, stamp_furniture, FurniturePlacement,
    PropModelCells,
};
use engine::textures::PropColorSet;
use mesher::chunk::ChunkMap;
use mesher::fine::FineLayer;
use mesher::{Block, Pass, WorldScale};
use proto::{DeltaT, InputKind, InputT, MessageT};
use sim::{DefRegistry, Kind, Scene, World};
use vehicles::mesh::{clear_vehicle, stamp_vehicle, VEHICLE_BLOCK_ID};
use vehicles::model::PartKey;
use vehicles::store::VehicleStore;

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

fn block_for(id: u32) -> Block {
    Block {
        id,
        color: [id as f32 / 16.0, 0.5, 0.5, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

fn sim_world() -> (DefRegistry, World, proto::SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = Scene::load_file(&scene_path("destructible_street")).expect("scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    let snap = world.snapshot(&reg);
    (reg, world, snap)
}

/// The single Delta in a key's flush.
fn delta_of(msgs: &[MessageT]) -> &DeltaT {
    let deltas: Vec<&DeltaT> = msgs
        .iter()
        .filter_map(|m| match m {
            MessageT::Delta(d) => Some(&**d),
            _ => None,
        })
        .collect();
    assert_eq!(deltas.len(), 1, "expected exactly one Delta in the flush");
    deltas[0]
}

/// Block id at (x, y, block_z) in a snapshot-built ChunkMap
/// (the roof gate's reader).
fn id_at(map: &ChunkMap, x: i32, y: i32, block_z: i32) -> u32 {
    let chunk = &map.chunks[&[x.div_euclid(12), y.div_euclid(12)]];
    chunk.grid.get(x.rem_euclid(12), y.rem_euclid(12), block_z).id
}

const AIR: u32 = 0;
const CAR: u64 = 1;

// ---------------------------------------------------------------------
// T4: fine-prop ghost — a bashed-away table leaves no volume
// ---------------------------------------------------------------------

#[test]
fn t4_fine_prop_volume_dies_with_the_furniture() {
    let (reg, mut world, snap) = sim_world();
    let scale = WorldScale::default();
    let mut map = build_snapshot_world(reg.tiledefs(), &snap, scale, &block_for);
    let mut layer = FineLayer::default();

    // A synthetic table model: a solid box, resolved ONLY for
    // the f_table wire id (the scene's other furniture, the
    // board map, has no model here and must stamp nothing).
    let model = PropModelData {
        size: [16, 12, 16],
        palette: vec![[150, 110, 70]],
        voxels: vec![0u16; 16 * 12 * 16],
    };
    let colors = PropColorSet::from_models([&model].into_iter());
    let color_block = prop_block_for(&colors, 40_000, 0);
    let cells = PropModelCells::new(&model, &color_block);
    let table_id = reg.id_of(Kind::Furniture, "f_table");
    assert_ne!(table_id, 0);
    let resolve = |id: u32| -> Option<&dyn mesher::fine::ModelCells> {
        if id == table_id {
            Some(&cells as &dyn mesher::fine::ModelCells)
        } else {
            None
        }
    };

    // Placements from the snapshot's furniture arrays, the
    // way the live loop derives them.
    let mut placements = Vec::new();
    for b in snap.blocks.as_ref().expect("blocks") {
        let o = b.origin.as_ref().expect("origin");
        for (i, &f) in b.furniture.as_ref().expect("furniture").iter().enumerate() {
            if f != 0 {
                placements.push(FurniturePlacement {
                    x: o.x + (i % 12) as i32,
                    y: o.y + (i / 12) as i32,
                    level: 0,
                    furn_id: f,
                    quarter_turns: 0,
                });
            }
        }
    }
    stamp_furniture(&mut map, &mut layer, &placements, scale, &resolve)
        .expect("stamp succeeds");
    assert!(
        !layer.volumes_for_tile((4, 4)).is_empty(),
        "the table's fine volume is stamped"
    );
    assert!(
        layer.volumes_for_tile((6, 4)).is_empty(),
        "the model-less board map stamps no volume"
    );

    // Bash the table away in the sim (arm `s`, direction west)
    // and restamp the tile from the Delta's TileChange.
    assert!(world.apply_input(&reg, &key_input('s')).is_empty(), "arming is silent");
    let msgs = world.apply_input(&reg, &key_input('h'));
    let d = delta_of(&msgs);
    let tc = d
        .tile_changes
        .as_ref()
        .expect("tile changes")
        .iter()
        .find(|tc| {
            let p = tc.pos.as_ref().expect("pos");
            [p.x, p.y, p.z] == [4, 4, 0]
        })
        .expect("the bash Delta carries the table tile's change");
    assert_eq!(tc.furniture, 0, "the wire says the furniture is gone");
    restamp_tile(
        &mut map,
        &mut layer,
        &FurniturePlacement {
            x: 4,
            y: 4,
            level: 0,
            furn_id: tc.furniture,
            quarter_turns: 0,
        },
        scale,
        &resolve,
    )
    .expect("restamp succeeds");
    assert!(
        layer.volumes_for_tile((4, 4)).is_empty(),
        "no ghost prop: the destroyed table's volume is gone"
    );
}

// ---------------------------------------------------------------------
// T4 vehicle half + T11: store, stamps, broken, rotate, remove
// ---------------------------------------------------------------------

#[test]
fn t11_vehicle_stamps_broken_rotation_and_removal() {
    let (reg, mut world, snap) = sim_world();
    let scale = WorldScale::default();
    let mut map = build_snapshot_world(reg.tiledefs(), &snap, scale, &block_for);
    let mut store = VehicleStore::new();
    store.apply_snapshot(&snap);
    let model = store.get(CAR).expect("car in the store").clone();
    assert_eq!(model.face_deg, 23.0, "the store keeps the float facing verbatim");
    assert_eq!(model.parts.len(), 13, "13 parts in the engine model");

    // Stamp: the 9 distinct part tiles are occupied; the roof
    // part sits on the top block of its tile (17,5); the
    // wheel-only tile (16,4) fills the body band and leaves
    // the top block to whatever the world build put there
    // (AIR under the pre-Lane-G slab model; the terrain's
    // standing-fill block since Lane G — either way the
    // wheel stamp must not touch it).
    let wheel_top_before = id_at(&map, 16, 4, 2);
    let (occupied, _) = stamp_vehicle(&mut map, &model, 0);
    let want: BTreeSet<[i32; 3]> = [
        [16, 4, 0],
        [16, 5, 0],
        [16, 6, 0],
        [17, 4, 0],
        [17, 5, 0],
        [17, 6, 0],
        [18, 4, 0],
        [18, 5, 0],
        [18, 6, 0],
    ]
    .into_iter()
    .collect();
    assert_eq!(occupied, want, "the car's 9 tiles stamp");
    assert_eq!(id_at(&map, 17, 5, 2), VEHICLE_BLOCK_ID, "roof part on the top block");
    assert_eq!(id_at(&map, 17, 5, 0), VEHICLE_BLOCK_ID, "body under the roof tile");
    assert_eq!(id_at(&map, 16, 4, 0), VEHICLE_BLOCK_ID, "wheel tile body band");
    assert_eq!(
        id_at(&map, 16, 4, 2),
        wheel_top_before,
        "wheel-only tile: the stamp leaves the top block alone"
    );
    assert_ne!(
        id_at(&map, 16, 4, 2),
        VEHICLE_BLOCK_ID,
        "wheel-only tile: no vehicle block on the top block"
    );

    // Damage the door (mount (0,-1)) to 0 through the sim's
    // World API: the next Delta flips the engine model's
    // broken flag for exactly that part.
    let door = PartKey {
        vpart: reg.id_of(Kind::VehiclePart, "door"),
        mount: [0, -1, 0],
    };
    let door_index = world
        .vehicle_part_index(&reg, CAR, "door", [0, -1, 0])
        .expect("door part index");
    assert!(world.damage_vehicle_part(CAR, door_index, 255));
    let msgs = world.apply_input(&reg, &key_input('.'));
    store.apply_delta(delta_of(&msgs));
    let model = store.get(CAR).expect("car still stored").clone();
    assert!(
        model.part(door).expect("door still present").broken,
        "hp 0 sets the engine-side broken flag"
    );
    assert_eq!(model.face_deg, 23.0, "damage does not move the car");

    // Rotate to 90 degrees: restamp the live-loop way (clear
    // the store's previous model, stamp the current one). The
    // wheel from mount (1,-1) moves (18,4) -> (18,6); the
    // roof part is still a roof part, still on top at (17,5).
    assert!(world.set_vehicle_face(CAR, 90.0));
    let msgs = world.apply_input(&reg, &key_input('.'));
    store.apply_delta(delta_of(&msgs));
    let previous = store.previous(CAR).expect("previous model kept").clone();
    let model = store.get(CAR).expect("car still stored").clone();
    assert_eq!(model.face_deg, 90.0, "the rotated facing is the exact float");
    clear_vehicle(&mut map, &previous, 0);
    let (occupied, _) = stamp_vehicle(&mut map, &model, 0);
    assert_eq!(occupied, want, "rotation is rigid: the same 3x3 tile block");
    assert_eq!(id_at(&map, 18, 6, 0), VEHICLE_BLOCK_ID, "wheel (1,-1) rotated to (18,6)");
    assert_eq!(id_at(&map, 17, 5, 2), VEHICLE_BLOCK_ID, "roof still on top after rotation");
    let roof = model
        .parts
        .iter()
        .find(|p| p.key.vpart == reg.id_of(Kind::VehiclePart, "roof"))
        .expect("roof part");
    assert!(roof.roof && roof.abs_pos == [17, 5, 0], "roof part stays roof at the pivot tile");

    // Remove the roof part: its top block unstamps; the body
    // band of the pivot tile (seat/frame) remains.
    let roof_index = world
        .vehicle_part_index(&reg, CAR, "roof", [0, 0, 0])
        .expect("roof part index");
    assert!(world.remove_vehicle_part(CAR, roof_index));
    let msgs = world.apply_input(&reg, &key_input('.'));
    store.apply_delta(delta_of(&msgs));
    let previous = store.previous(CAR).expect("previous model kept").clone();
    let model = store.get(CAR).expect("car still stored").clone();
    assert_eq!(model.parts.len(), 12, "the wire dropped the roof part");
    clear_vehicle(&mut map, &previous, 0);
    stamp_vehicle(&mut map, &model, 0);
    assert_eq!(id_at(&map, 17, 5, 2), AIR, "removed roof part leaves no top stamp");
    assert_eq!(id_at(&map, 17, 5, 0), VEHICLE_BLOCK_ID, "body parts still stamp the pivot tile");

    // Remove the wheel at mount (-1,-1) — at 90 degrees it
    // sits at (18,4): that tile unstamps entirely.
    let wheel_index = world
        .vehicle_part_index(&reg, CAR, "wheel", [-1, -1, 0])
        .expect("wheel part index");
    assert!(world.remove_vehicle_part(CAR, wheel_index));
    let msgs = world.apply_input(&reg, &key_input('.'));
    store.apply_delta(delta_of(&msgs));
    let previous = store.previous(CAR).expect("previous model kept").clone();
    let model = store.get(CAR).expect("car still stored").clone();
    assert_eq!(model.parts.len(), 11, "the wire dropped the wheel too");
    clear_vehicle(&mut map, &previous, 0);
    let (occupied, _) = stamp_vehicle(&mut map, &model, 0);
    assert!(!occupied.contains(&[18, 4, 0]), "the removed wheel's tile no longer stamps");
    assert_eq!(id_at(&map, 18, 4, 0), AIR, "the removed wheel's tile is AIR");
}
