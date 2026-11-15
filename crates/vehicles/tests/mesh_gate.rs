//! Render-model gates for Phase 8 vehicles: the model, stamped
//! into the mesher's world, occupies exactly the tiles the game
//! exported — no more, no fewer — and meshes to real geometry.

use std::collections::BTreeSet;

use mesher::{ChunkMap, WorldScale};
use proto::{Vec3iT, VehiclePartStateT, VehicleT};
use vehicles::fixtures::{truth_abs_pos, truth_vehicle, MOUNTS};
use vehicles::{clear_vehicle, stamp_vehicle, VehicleModel, FLAG_ROOF};

fn map_around(x: i32, y: i32) -> ChunkMap {
    let mut map = ChunkMap::new(WorldScale::default());
    // Chunks covering the fixture car's neighbourhood (tiles
    // ~14..38 in both axes), 3 z-levels.
    for cx in 1..=3 {
        for cy in 1..=3 {
            map.add_chunk([cx, cy], 3);
        }
    }
    let _ = (x, y);
    map
}

/// At a non-round facing, the stamped blocks' tile set equals the
/// truth set from the mock game — the render model cannot drift
/// from the export.
#[test]
fn stamped_tiles_equal_truth_at_23_degrees() {
    let pivot = [20, 20, 0];
    let wire = truth_vehicle(7, pivot, 23.0);
    let model = VehicleModel::from_proto(&wire);
    let mut map = map_around(20, 20);
    let (occupied, dirtied) = stamp_vehicle(&mut map, &model, 0);
    let truth: BTreeSet<[i32; 3]> = MOUNTS
        .iter()
        .map(|(mount, _, _)| truth_abs_pos(pivot, *mount, 23.0))
        .collect();
    assert_eq!(
        occupied, truth,
        "occupied tiles must equal the exported tiles exactly"
    );
    assert!(!dirtied.is_empty());
    // The chunk containing the car meshes to real geometry.
    let coord = ChunkMap::chunk_coord_of(20, 20);
    let meshes = map.mesh_chunk(coord, true).expect("chunk exists");
    assert!(
        meshes.total_quads() > 0,
        "the stamped vehicle must produce quads"
    );
    // Clearing restores air: no ghost vehicle.
    clear_vehicle(&mut map, &model, 0);
    let meshes = map.mesh_chunk(coord, true).unwrap();
    let vehicle_quads = meshes.total_quads();
    assert_eq!(vehicle_quads, 0, "after clear, the chunk is empty again");
}

/// Roof composition: a roof part sharing a tile with a body part
/// completes the column (3 blocks); alone it is the top slab only.
#[test]
fn roof_part_composes_with_body_on_shared_tile() {
    // Hand-built model (real exports put every part at the
    // vehicle's z; the roof distinction is the flag).
    let mut v = VehicleT::default();
    v.id = 9;
    v.face_deg = 0.0;
    let part = |vpart: u32, abs: [i32; 3], mount: [i32; 3], flags: u32| {
        let mut p = VehiclePartStateT::default();
        p.vpart = vpart;
        p.abs_pos = Some(Vec3iT {
            x: abs[0],
            y: abs[1],
            z: abs[2],
        });
        p.mount = Some(Vec3iT {
            x: mount[0],
            y: mount[1],
            z: mount[2],
        });
        p.hp_frac = 255;
        p.flags = flags;
        p
    };
    v.parts = Some(vec![
        part(100, [20, 20, 0], [0, 0, 0], 0),         // frame
        part(102, [20, 20, 0], [0, 0, 0], FLAG_ROOF), // roof, same tile
        part(102, [21, 20, 0], [1, 0, 0], FLAG_ROOF), // roof, alone
    ]);
    let model = VehicleModel::from_proto(&v);
    let mut map = map_around(20, 20);
    let (occupied, _) = stamp_vehicle(&mut map, &model, 0);
    assert_eq!(occupied.len(), 2);
    let chunk = &map.chunks[&ChunkMap::chunk_coord_of(20, 20)];
    // Local block coords: tile (20,20) is local (8,8) in chunk (1,1).
    let filled =
        |x: usize, y: usize, z: usize| !chunk.grid.get(x as i32, y as i32, z as i32).is_air();
    assert!(
        filled(8, 8, 0) && filled(8, 8, 1) && filled(8, 8, 2),
        "shared tile: full column"
    );
    assert!(
        !filled(9, 8, 0) && !filled(9, 8, 1) && filled(9, 8, 2),
        "lone roof: top slab only"
    );
}
