//! Phase 8 gate tests 8.4 and 8.4b at model level (Part II §11).
//! The in-game halves (actually opening a door in the fork, standing
//! in a real two-storey house) stay blocked on 0.1/0.2.

use hardcases::{classify_transition, floor_at, Column, DefTable, Floor, TileWorld, Transition};
use proto::samples::{
    sample_delta, sample_house_block, sample_tiledefs, TER_DOOR, TER_DOOR_OPEN, TER_WALL,
};
use proto::{TileChangeT, TileDefT, TileDefsT, TileKind, Vec3iT, PROTOCOL_VERSION};

// ---------------------------------------------------------------------------
// Gate 8.4 — doors/windows: state arrives as an id change
// ---------------------------------------------------------------------------

fn house_world() -> (DefTable, TileWorld) {
    let table = DefTable::from_tiledefs(&sample_tiledefs());
    let block = sample_house_block();
    let mut world = TileWorld::default();
    let terrain = block.terrain.as_ref().unwrap();
    for y in 0..12 {
        for x in 0..12 {
            world
                .terrain
                .insert([x as i32, y as i32, 0], terrain[y * 12 + x]);
        }
    }
    (table, world)
}

/// The protocol's own sample flow: the sample house has a closed
/// door at (5,11); the sample Delta opens it. Classification must
/// say Open (via the def's declared transform), the world must
/// hold the new id afterwards, and the new id's material reference
/// must differ from the closed door's.
#[test]
fn gate_8_4_sample_door_opens_by_id_change() {
    let (table, mut world) = house_world();
    let door_at = [5, 11, 0];
    assert_eq!(world.terrain[&door_at], TER_DOOR);
    let closed_fg = world.terrain_fg_ref(&table, door_at);

    let delta = sample_delta();
    let change = &delta.tile_changes.as_ref().unwrap()[0];
    assert_eq!(change.terrain, TER_DOOR_OPEN);
    let (ter_t, _) = world.apply_change(&table, change);

    assert_eq!(
        ter_t,
        Transition::Open,
        "3 -> 5 is t_door_c's declared open transform"
    );
    assert_eq!(
        world.terrain[&door_at], TER_DOOR_OPEN,
        "the world holds the new id"
    );
    let open_fg = world.terrain_fg_ref(&table, door_at);
    assert_ne!(
        closed_fg, open_fg,
        "model/texture follow the new id (fg_ref 102 -> 104)"
    );
    assert_eq!(closed_fg, Some(102));
    assert_eq!(open_fg, Some(104));
}

/// Close and bash classify through the same table (fixture defs
/// carrying the full transform set, as real TileDefs do), an
/// unrelated replacement classifies as Other, and a no-op as
/// Unchanged. Furniture doors follow the identical rule — the
/// classifier never looks at the slot, only at the defs.
#[test]
fn gate_8_4_close_bash_other_unchanged() {
    // Generated wire structs are non-exhaustive outside the proto
    // crate: build via Default + assignment.
    let def = |id: u32, sid: &str, kind: TileKind| {
        let mut d = TileDefT::default();
        d.id = id;
        d.string_id = Some(sid.into());
        d.name = Some(sid.into());
        d.kind = kind;
        d.fg_ref = -1;
        d.bg_ref = -1;
        d
    };
    let mut closed = def(1, "f_door_c", TileKind::Furniture);
    closed.open_result = Some("f_door_o".into());
    closed.bash_result = Some("f_door_broken".into());
    let mut open = def(2, "f_door_o", TileKind::Furniture);
    open.close_result = Some("f_door_c".into());
    let broken = def(3, "f_door_broken", TileKind::Furniture);
    let mut defs = TileDefsT::default();
    defs.defs = Some(vec![closed, open, broken]);
    defs.protocol_version = PROTOCOL_VERSION;
    let table = DefTable::from_tiledefs(&defs);
    assert_eq!(
        classify_transition(&table, proto::TileKind::Furniture, 1, 2),
        Transition::Open
    );
    assert_eq!(
        classify_transition(&table, proto::TileKind::Furniture, 2, 1),
        Transition::Close
    );
    assert_eq!(
        classify_transition(&table, proto::TileKind::Furniture, 1, 3),
        Transition::Bash
    );
    assert_eq!(
        classify_transition(&table, proto::TileKind::Furniture, 1, 1),
        Transition::Unchanged
    );
    assert_eq!(
        classify_transition(&table, proto::TileKind::Furniture, 1, 99),
        Transition::Other
    );
    // A wall smashing into a door id is not a door action.
    assert_eq!(
        classify_transition(&table, proto::TileKind::Terrain, TER_WALL, TER_DOOR),
        Transition::Other
    );
}

// ---------------------------------------------------------------------------
// Gate 8.4b — open air, roofs, multi-storey
// ---------------------------------------------------------------------------

const FLR: u32 = 10; // wood floor, roofs as FLAT_ROOF
const FLAT_ROOF: u32 = 11;
const WALL: u32 = 12;
const AIR: u32 = 13; // t_open_air
const GRASS: u32 = 14; // declares no roof

fn roof_table() -> DefTable {
    let def = |id: u32, sid: &str, roof: Option<&str>| {
        let mut d = TileDefT::default();
        d.id = id;
        d.string_id = Some(sid.into());
        d.name = Some(sid.into());
        d.kind = TileKind::Terrain;
        d.fg_ref = -1;
        d.bg_ref = -1;
        d.roof_result = roof.map(str::to_string);
        d
    };
    let mut defs = TileDefsT::default();
    defs.defs = Some(vec![
        def(FLR, "t_wood_floor", Some("t_flat_roof")),
        def(FLAT_ROOF, "t_flat_roof", None),
        def(WALL, "t_wall", Some("t_flat_roof")),
        def(AIR, "t_open_air", None),
        def(GRASS, "t_grass", None),
    ]);
    defs.protocol_version = PROTOCOL_VERSION;
    DefTable::from_tiledefs(&defs)
}

/// Two-storey house column + basement, per the gate: stand on
/// floor 1, floor 2, and in the basement. No phantom floor over
/// open air; the floor above a solid tile is its roof material.
#[test]
fn gate_8_4b_two_storey_and_basement() {
    let table = roof_table();
    // z=-1 basement floor, z=0 ground floor, z=1 open air above it
    // (the second storey's floor is z=0's roof), z=2 open air
    // (attic shaft — no floor of its own).
    let column = Column {
        z_min: -1,
        terrain: vec![FLR, FLR, AIR, AIR],
    };
    assert!(
        !hardcases::openair::emits_block(&table, AIR),
        "open air emits no block"
    );
    assert_eq!(
        floor_at(&table, &column, -1),
        Floor::Solid(FLR),
        "basement floor is itself"
    );
    assert_eq!(floor_at(&table, &column, 0), Floor::Solid(FLR));
    assert_eq!(
        floor_at(&table, &column, 1),
        Floor::RoofOf(FLR, FLAT_ROOF),
        "floor 2 is floor 1's roof material — not a phantom slab, not air's material"
    );
    assert_eq!(
        floor_at(&table, &column, 2),
        Floor::None,
        "open air over open air: no floor at this level (the shaft continues)"
    );
}

/// Outside looking in + pit: open air over a roof-less terrain is a
/// hole (no fabricated floor), and the resolution is identical
/// whatever subset of levels a cutaway asks about.
#[test]
fn gate_8_4b_no_fabricated_floor_and_cutaway_independence() {
    let table = roof_table();
    let outside = Column {
        z_min: 0,
        terrain: vec![GRASS, AIR],
    };
    assert_eq!(
        floor_at(&table, &outside, 1),
        Floor::None,
        "grass declares no roof: open air above it has no floor"
    );
    // Cutaway independence: resolving only the levels a cutaway
    // would draw (z <= player_z) changes nothing for those levels.
    let house = Column {
        z_min: -1,
        terrain: vec![FLR, FLR, AIR, AIR],
    };
    for z in -1..=2 {
        let full = floor_at(&table, &house, z);
        let cut = Column {
            z_min: house.z_min,
            terrain: house.terrain[..=(z - house.z_min) as usize].to_vec(),
        };
        assert_eq!(
            floor_at(&table, &cut, z),
            full,
            "cutaway must not change a level's floor"
        );
    }
}

/// The Phase 8 schema addition round-trips on the wire: a TileDefs
/// carrying `roof_result` encodes and decodes with the value
/// intact (FlatBuffers framing via the proto crate), and the def
/// table resolves it to the roof def's id.
#[test]
fn gate_8_4b_roof_result_survives_the_wire() {
    use proto::{FrameDecoder, FrameEncoder, MessageT};
    let mut defs = TileDefsT::default();
    defs.protocol_version = PROTOCOL_VERSION;
    let mut floor = TileDefT::default();
    floor.id = FLR;
    floor.string_id = Some("t_wood_floor".into());
    floor.name = Some("wood floor".into());
    floor.kind = TileKind::Terrain;
    floor.fg_ref = 0;
    floor.bg_ref = -1;
    floor.roof_result = Some("t_flat_roof".into());
    let mut roof = floor.clone();
    roof.id = FLAT_ROOF;
    roof.string_id = Some("t_flat_roof".into());
    roof.roof_result = None;
    defs.defs = Some(vec![floor, roof]);

    let payload = proto::encode_message(&MessageT::TileDefs(Box::new(defs)));
    let wire = FrameEncoder::encode(&payload);
    let mut dec = FrameDecoder::new();
    dec.push(&wire);
    let frame = dec.next_frame().unwrap().expect("one complete frame");
    let msg = proto::decode_message(&frame).unwrap();
    let MessageT::TileDefs(back) = &msg else {
        panic!("expected TileDefs back");
    };
    let table = DefTable::from_tiledefs(back);
    let f = table.def(TileKind::Terrain, FLR).unwrap();
    assert_eq!(f.roof_result.as_deref(), Some("t_flat_roof"));
    assert_eq!(
        table.resolve(TileKind::Terrain, &f.roof_result),
        Some(FLAT_ROOF)
    );
    // Sanity: TileChange positions still decode as before (the
    // change type itself is untouched by the schema addition).
    let _: &TileChangeT = &sample_delta().tile_changes.unwrap()[0];
    let _ = Vec3iT { x: 0, y: 0, z: 0 };
}
