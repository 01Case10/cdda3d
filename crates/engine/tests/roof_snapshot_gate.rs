//! Roof + cutaway gate on WIRE-shaped fake data.
//!
//! The stub house (`world::build_fake_house`) proves the cutaway
//! on a hand-built ChunkMap whose roof is a mesher tile id. This
//! gate proves the same behaviour on the data the fork would
//! actually send — `proto::samples::sample_roofed_tiledefs()` +
//! `sample_roofed_house_snapshot()` /
//! `sample_implicit_roof_snapshot()`, modelled on the exporter's
//! ViewSpec window (z -1..=+1 around the player), bungalow-style
//! roof mapgen, and the game data's real `roof` values — ingested
//! through `snapshot_world::build_snapshot_world`, the wire ->
//! columns -> blocks path. Data-level, like gate 2c.6: the
//! cutaway is a level filter, so block counts by material in the
//! filtered map ARE the behaviour, in both world forms (the
//! filtered ChunkMap meshes on the raster path and converts to
//! the RT tree's VoxelWorld unchanged).

use engine::cutaway::CutawayMode;
use engine::snapshot_world::{build_snapshot_world, columns_from_snapshot};
use engine::world::{filtered_map, mesh_world};
use mesher::chunk::ChunkMap;
use mesher::{Block, Pass, WorldScale};
use proto::samples::*;
use proto::MessageT;

fn block_for(id: u32) -> Block {
    Block {
        id,
        color: [id as f32 / 16.0, 0.5, 0.5, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// Block-z of the floor slab of game level `z` in a snapshot
/// whose z_min is -1: level index (z + 1) * 3 blocks per level.
fn slab_z(game_z: i32) -> i32 {
    (game_z + 1) * 3
}

fn id_at(map: &ChunkMap, x: i32, y: i32, block_z: i32) -> u32 {
    let chunk = &map.chunks[&[x.div_euclid(12), y.div_euclid(12)]];
    chunk
        .grid
        .get(x.rem_euclid(12), y.rem_euclid(12), block_z)
        .id
}

fn count_id(map: &ChunkMap, want: u32) -> usize {
    let mut n = 0;
    for chunk in map.chunks.values() {
        for z in 0..chunk.grid.size[2] as i32 {
            for y in 0..chunk.grid.size[1] as i32 {
                for x in 0..chunk.grid.size[0] as i32 {
                    if chunk.grid.get(x, y, z).id == want {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

/// The player z the snapshot itself reports: the avatar entity's
/// position, which must agree with the view origin (the
/// exporter's ViewSpec centre is the player tile).
fn player_z(snap: &proto::SnapshotT) -> i32 {
    let player = snap
        .entities
        .as_ref()
        .unwrap()
        .iter()
        .find(|e| e.sprite_id.as_deref() == Some("player"))
        .expect("snapshot carries the player entity");
    let pz = player.pos.as_ref().unwrap().z;
    assert_eq!(snap.view_origin.as_ref().unwrap().z, pz);
    pz
}

#[test]
fn roofed_fake_data_survives_the_wire() {
    // TileDefs and Snapshot round-trip through the real
    // FlatBuffers envelope codec, unchanged in the parts the
    // roof path reads.
    let defs_msg = MessageT::TileDefs(Box::new(sample_roofed_tiledefs()));
    let MessageT::TileDefs(defs) =
        proto::decode_message(&proto::encode_message(&defs_msg)).unwrap()
    else {
        panic!("decoded message is not TileDefs");
    };
    let floor = defs
        .defs
        .as_ref()
        .unwrap()
        .iter()
        .find(|d| d.string_id.as_deref() == Some("t_floor"))
        .unwrap();
    assert_eq!(floor.roof_result.as_deref(), Some("t_wood_treated_roof"));

    let snap_msg = MessageT::Snapshot(Box::new(sample_roofed_house_snapshot()));
    let MessageT::Snapshot(snap) =
        proto::decode_message(&proto::encode_message(&snap_msg)).unwrap()
    else {
        panic!("decoded message is not Snapshot");
    };
    let blocks = snap.blocks.as_ref().unwrap();
    assert_eq!(blocks.len(), 3, "ViewSpec window z_below=1, z_above=1");
    let zs: Vec<i32> = blocks
        .iter()
        .map(|b| b.origin.as_ref().unwrap().z)
        .collect();
    assert_eq!(zs, vec![-1, 0, 1]);
    let roof_block = &blocks[2];
    assert_eq!(
        roof_block.terrain.as_ref().unwrap()[idx(5, 5)],
        TER_ROOF_SHINGLE,
        "explicit roof terrain over the room at z +1"
    );
    assert_eq!(
        roof_block.terrain.as_ref().unwrap()[idx(0, 0)],
        TER_OPEN_AIR,
        "open air outside the footprint at z +1"
    );
    assert_eq!(player_z(&snap), 0, "player stands inside, at z 0");

    // Column derivation: 144 columns, one shared z_min, the
    // room column reads [open air, floor, roof] bottom-first.
    let columns = columns_from_snapshot(&snap);
    assert_eq!(columns.len(), 144);
    assert!(columns
        .iter()
        .all(|c| c.z_min == -1 && c.terrain.len() == 3));
    let room = columns.iter().find(|c| c.x == 5 && c.y == 5).unwrap();
    assert_eq!(
        room.terrain,
        vec![TER_OPEN_AIR, TER_FLOOR, TER_ROOF_SHINGLE]
    );
    println!("roof wire gate: fake TileDefs + Snapshot round-trip and columnise");
}

#[test]
fn explicit_roof_cutaway_at_player_z_shows_the_interior() {
    let defs = sample_roofed_tiledefs();
    let snap = sample_roofed_house_snapshot();
    let pz = player_z(&snap);
    let scale = WorldScale::default();
    let map = build_snapshot_world(&defs, &snap, scale, &block_for);

    // Full build (no cutaway): the roof deck exists at z +1
    // over the whole 6x6 footprint, walls included; the room
    // floor is at z 0; open air over grass produces no slab
    // at z +1. Counts: since Lane G a solid tile fills its
    // whole level, so a material's block count is its tile
    // count x blocks_per_z_level (the explicit roof is
    // authored terrain at z +1, so it fills like any solid
    // tile; the per-tile facts these counts pin are
    // unchanged from the slab-era numbers / PER).
    let per = scale.blocks_per_z_level as usize;
    assert_eq!(id_at(&map, 5, 5, slab_z(1)), TER_ROOF_SHINGLE);
    assert_eq!(id_at(&map, 3, 3, slab_z(1)), TER_ROOF_SHINGLE);
    assert_eq!(id_at(&map, 5, 5, slab_z(0)), TER_FLOOR);
    assert_eq!(id_at(&map, 0, 0, slab_z(0)), TER_GRASS);
    assert_eq!(
        id_at(&map, 0, 0, slab_z(1)),
        0,
        "no fabricated roof over the yard"
    );
    assert_eq!(count_id(&map, TER_ROOF_SHINGLE), 36 * per);

    // Player inside (z 0), HideAbove(0): the roof level is
    // filtered out at data level; the interior floor remains.
    let inside = CutawayMode::HideAbove(0).visible_levels(pz);
    let cut = filtered_map(&map, -1, &inside);
    assert_eq!(count_id(&cut, TER_ROOF_SHINGLE), 0, "roof gone from inside");
    assert_eq!(
        count_id(&cut, TER_FLOOR),
        16 * per,
        "4x4 interior floor still drawn"
    );
    assert_eq!(count_id(&cut, TER_GRASS), (144 - 36) * per, "yard still drawn");

    // Exterior view (the windowed default HideAbove(1) at
    // player z 0): the roof level is kept whole.
    let outside = CutawayMode::HideAbove(1).visible_levels(pz);
    let full = filtered_map(&map, -1, &outside);
    assert_eq!(count_id(&full, TER_ROOF_SHINGLE), 36 * per);

    // Both world forms agree: the RT tree built from the cut
    // map has no roof cell over the room; from the full map it
    // carries the roof material at the roof slab.
    let rt_cut = treebuild::VoxelWorld::from_chunk_map(&cut);
    assert!(
        rt_cut.get([5, 5, slab_z(1)]).is_none(),
        "RT: no roof over the room"
    );
    let rt_full = treebuild::VoxelWorld::from_chunk_map(&full);
    assert_eq!(
        rt_full.get([5, 5, slab_z(1)]).map(|c| c.material_id),
        Some(TER_ROOF_SHINGLE),
        "RT: roof material present in the exterior view"
    );

    // And the cut map meshes (raster form) with real quads —
    // the interior is a drawn scene, not an empty world.
    let (cut_meshes, _) = mesh_world(&map, -1, &inside);
    assert!(cut_meshes.total_quads() > 0);
    let (full_meshes, _) = mesh_world(&map, -1, &outside);
    assert_ne!(cut_meshes.total_quads(), full_meshes.total_quads());
    println!("roof wire gate: explicit roof removed by the player-z cutaway in both world forms");
}

#[test]
fn implicit_roof_resolves_from_tiledef_roof_results() {
    let defs = sample_roofed_tiledefs();
    let snap = sample_implicit_roof_snapshot();
    let pz = player_z(&snap);
    let scale = WorldScale::default();
    let map = build_snapshot_world(&defs, &snap, scale, &block_for);

    // z +1 is all open air in the snapshot; the slabs stamped
    // there are each column's roof_result material: treated
    // wood over the floor, flat roof over walls/windows, wood
    // roof over the door — and nothing over the yard.
    assert_eq!(id_at(&map, 5, 5, slab_z(1)), TER_ROOF_WOOD_TREATED);
    assert_eq!(id_at(&map, 3, 3, slab_z(1)), TER_ROOF_FLAT);
    assert_eq!(id_at(&map, 5, 3, slab_z(1)), TER_ROOF_FLAT, "window column");
    assert_eq!(id_at(&map, 6, 8, slab_z(1)), TER_ROOF_WOOD, "door column");
    assert_eq!(id_at(&map, 0, 0, slab_z(1)), 0, "grass declares no roof");
    let roof_slabs = count_id(&map, TER_ROOF_WOOD_TREATED)
        + count_id(&map, TER_ROOF_FLAT)
        + count_id(&map, TER_ROOF_WOOD);
    assert_eq!(roof_slabs, 36, "one roof slab per footprint column");

    // Same cutaway behaviour as the explicit form. (The
    // implicit roof slabs above stay one-per-column —
    // RoofOf never fills; the FLOOR count is tiles x
    // blocks_per_z_level since Lane G's standing fill.)
    let inside = CutawayMode::HideAbove(0).visible_levels(pz);
    let cut = filtered_map(&map, -1, &inside);
    assert_eq!(count_id(&cut, TER_ROOF_WOOD_TREATED), 0);
    assert_eq!(count_id(&cut, TER_ROOF_FLAT), 0);
    assert_eq!(count_id(&cut, TER_ROOF_WOOD), 0);
    assert_eq!(count_id(&cut, TER_FLOOR), 16 * scale.blocks_per_z_level as usize);
    let outside = CutawayMode::HideAbove(1).visible_levels(pz);
    let full = filtered_map(&map, -1, &outside);
    assert_eq!(
        count_id(&full, TER_ROOF_WOOD_TREATED)
            + count_id(&full, TER_ROOF_FLAT)
            + count_id(&full, TER_ROOF_WOOD),
        36
    );
    println!("roof wire gate: implicit roof resolved from roof_result and cut away identically");
}
