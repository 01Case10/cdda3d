//! Terrain-model gate on WIRE-shaped data (terrain wiring,
//! 2026-11-18).
//!
//! The fixture-scene terrain extension
//! (`scene_prep::extend_prop_scene_with_terrain`) finds
//! modelled tiles by scanning a built map for fake tile
//! ids. This gate proves the snapshot path's counterpart:
//! `snapshot_world::build_snapshot_world` +
//! `stamp_snapshot_terrain` resolve each wire tile's def id
//! through the TileDefs to its game id, and seat the
//! material's prop model by the kind the voxelizer recorded
//! for it — Ground models replace the coarse floor slab at
//! the level base, Fence half-segments compose per
//! same-def neighbour mask on the slab top, Standing models
//! stamp as-is on the slab top, and an open-air tile whose
//! floor resolves through `roof_result` seats the roof
//! material's model as ground. Synthetic models and defs:
//! the wiring is what is under test, not the builders
//! (those are the voxelizer coverage gate's business).

use engine::materials::{ModelKind, PropModelData};
use engine::snapshot_world::{build_snapshot_world, stamp_snapshot_terrain};
use mesher::chunk::ChunkMap;
use mesher::fine::FineLayer;
use mesher::scale::WorldScale;
use mesher::{Block, Pass};
use proto::samples::{
    sample_terrain_snapshot, sample_terrain_tiledefs, TER_CHAINFENCE, TER_FLOOR, TER_GRASS,
    TER_ROOF_SHINGLE, TER_TREE, TER_WALL,
};

/// A solid [32, h, 32] model of one colour.
fn slab_model(h: u32, rgb: [u8; 3]) -> PropModelData {
    PropModelData {
        size: [32, h, 32],
        palette: vec![rgb],
        voxels: vec![0; (32 * h * 32) as usize],
    }
}

fn tree_model() -> PropModelData {
    let mut m = PropModelData {
        size: [32, 40, 32],
        palette: vec![[90, 60, 30], [40, 160, 40]],
        voxels: vec![PropModelData::EMPTY; 32 * 40 * 32],
    };
    let set = |m: &mut PropModelData, x: u32, y: u32, z: u32, v: u16| {
        m.voxels[((y * 32 + z) * 32 + x) as usize] = v;
    };
    for y in 0..20 {
        set(&mut m, 16, y, 16, 0); // trunk
    }
    for x in 15..=17 {
        for z in 15..=17 {
            set(&mut m, x, 35, z, 1); // canopy card
        }
    }
    m
}

fn fence_model() -> PropModelData {
    // The half-segment: panel on the centre plane
    // (z = 15..16), full width, 28 high.
    let mut m = PropModelData {
        size: [32, 28, 32],
        palette: vec![[180, 180, 180]],
        voxels: vec![PropModelData::EMPTY; 32 * 28 * 32],
    };
    for y in 0..28 {
        for z in 15..=16 {
            for x in 0..32 {
                m.voxels[((y * 32 + z) * 32 + x) as usize] = 0;
            }
        }
    }
    m
}

fn wall_model() -> PropModelData {
    // Sparse markers are enough: bottom and top corners.
    let mut m = PropModelData {
        size: [32, 96, 32],
        palette: vec![[200, 200, 200]],
        voxels: vec![PropModelData::EMPTY; 32 * 96 * 32],
    };
    m.voxels[0] = 0; // (0, 0, 0)
    m.voxels[((95 * 32 + 0) * 32 + 0) as usize] = 0; // (0, 95, 0)
    m
}

struct Fixture {
    grass: PropModelData,
    floor: PropModelData,
    roof: PropModelData,
    tree: PropModelData,
    fence: PropModelData,
    wall: PropModelData,
}

impl Fixture {
    fn new() -> Self {
        Self {
            grass: slab_model(10, [10, 200, 10]),
            floor: slab_model(10, [150, 120, 80]),
            roof: slab_model(10, [160, 60, 50]),
            tree: tree_model(),
            fence: fence_model(),
            wall: wall_model(),
        }
    }

    fn resolve(&self, id: &str) -> Option<(&PropModelData, ModelKind)> {
        match id {
            "t_grass" => Some((&self.grass, ModelKind::Ground)),
            "t_floor" => Some((&self.floor, ModelKind::Ground)),
            // Deliberately Standing: a RoofOf floor seats
            // as Ground whatever kind the material carries
            // (the roof model is a floor slab by
            // construction) — the gate proves the rule.
            "t_shingle_flat_roof" => Some((&self.roof, ModelKind::Standing)),
            "t_tree" => Some((&self.tree, ModelKind::Standing)),
            "t_chainfence" => Some((&self.fence, ModelKind::Fence)),
            "t_wall" => Some((&self.wall, ModelKind::Standing)),
            _ => None,
        }
    }
}

/// Colour -> stamped block: one tag id per fixture colour.
fn colour_block(rgb: [u8; 3]) -> Block {
    let id = match rgb {
        [10, 200, 10] => 101,
        [150, 120, 80] => 102,
        [160, 60, 50] => 103,
        [90, 60, 30] => 104,
        [40, 160, 40] => 105,
        [180, 180, 180] => 106,
        [200, 200, 200] => 107,
        _ => 999,
    };
    Block {
        id,
        color: [
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
            1.0,
        ],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// Coarse def block: the def id itself, like the roof gate.
fn coarse_block(id: u32) -> Block {
    Block {
        id,
        color: [id as f32 / 16.0, 0.5, 0.5, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

fn slab_id(map: &ChunkMap, x: i32, y: i32, block_z: i32) -> u32 {
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

/// The fine block at tile (tx, ty), tile-local (lx, ly),
/// absolute fine z `fz`.
fn fine_at(layer: &FineLayer, tx: i32, ty: i32, lx: i32, ly: i32, fz: i32) -> Option<u32> {
    layer
        .block_at_fine(tx * 32 + lx, ty * 32 + ly, fz)
        .map(|b| b.id)
}

#[test]
fn snapshot_terrain_models_seat_by_kind() {
    let defs = sample_terrain_tiledefs();
    let snap = sample_terrain_snapshot();
    let fx = Fixture::new();
    let scale = WorldScale::default();
    let mut map = build_snapshot_world(&defs, &snap, scale, &coarse_block);
    // Pre-stamp coarse state: grass slab present, the roof
    // deck over the floor patch resolved by build_columns,
    // no roof over grass.
    assert_eq!(slab_id(&map, 0, 0, 0), TER_GRASS);
    assert_eq!(slab_id(&map, 3, 8, 0), TER_FLOOR);
    assert_eq!(slab_id(&map, 3, 8, 3), TER_ROOF_SHINGLE);
    assert_eq!(slab_id(&map, 0, 0, 3), 0, "open air stamps no slab");

    let mut layer = FineLayer::new(96);
    let resolve = |id: &str| fx.resolve(id);
    let report = stamp_snapshot_terrain(
        &mut map,
        &mut layer,
        &defs,
        &snap,
        scale,
        &resolve,
        &colour_block,
    )
    .unwrap();

    // Volumes: 133 grass + 4 floor + 4 roof (Ground),
    // 4 composed fences + tree + wall (standing slot).
    assert_eq!(report.volumes, 147);
    // Only the isolated fence tile is skipped.
    assert_eq!(report.skipped, 1);
    assert!(report.stamped_cells > 0);
    assert!(map.chunks[&[0, 0]].has_fine);

    // Ground: the model replaces the coarse slab at the
    // level base, in a volume only as deep as the model.
    assert_eq!(slab_id(&map, 0, 0, 0), 0, "grass slab aired");
    assert_eq!(count_id(&map, TER_GRASS), 0);
    assert_eq!(fine_at(&layer, 0, 0, 3, 3, 0), Some(101));
    assert_eq!(fine_at(&layer, 0, 0, 3, 3, 9), Some(101));
    assert_eq!(fine_at(&layer, 0, 0, 3, 3, 10), None, "shallow volume ends");
    // Floor tiles seat their own ground model the same way.
    assert_eq!(slab_id(&map, 3, 8, 0), 0);
    assert_eq!(fine_at(&layer, 3, 8, 3, 3, 0), Some(102));

    // RoofOf: the open-air level over the floor patch seats
    // the roof material's model as ground at that level's
    // base, replacing the roof slab build_columns stamped.
    assert_eq!(slab_id(&map, 3, 8, 3), 0, "roof slab aired");
    assert_eq!(count_id(&map, TER_ROOF_SHINGLE), 0);
    assert_eq!(fine_at(&layer, 3, 8, 3, 3, 96), Some(103));
    assert_eq!(fine_at(&layer, 3, 8, 3, 3, 105), Some(103));
    assert_eq!(fine_at(&layer, 3, 8, 3, 3, 106), None);
    // Grass declares no roof: nothing at level 1 there.
    assert_eq!(fine_at(&layer, 0, 0, 3, 3, 96), None);

    // Standing: the tree keeps its coarse slab and stamps
    // at the slab-top origin (fine z 32 at the default
    // scale).
    assert_eq!(slab_id(&map, 2, 2, 0), TER_TREE, "tree slab kept");
    assert_eq!(fine_at(&layer, 2, 2, 16, 16, 31), None);
    assert_eq!(fine_at(&layer, 2, 2, 16, 16, 32), Some(104), "trunk base");
    assert_eq!(fine_at(&layer, 2, 2, 16, 16, 32 + 35), Some(105), "canopy");
    // The wall (96 high) seats at the slab top too, so its
    // top marker lands in the level above's band — the
    // recorded slab-top convention.
    assert_eq!(slab_id(&map, 8, 2, 0), TER_WALL);
    assert_eq!(fine_at(&layer, 8, 2, 0, 0, 32), Some(107), "wall base");
    assert_eq!(fine_at(&layer, 8, 2, 0, 0, 32 + 95), Some(107), "wall top");

    // Fences: composed per same-def neighbour mask.
    // Middle of the run (mask W|E): the centre-plane panel.
    assert_eq!(fine_at(&layer, 5, 5, 16, 15, 42), Some(106));
    assert_eq!(fine_at(&layer, 5, 5, 16, 16, 42), Some(106));
    // Run end (mask E only): same panel (180-degree copy).
    assert_eq!(fine_at(&layer, 4, 5, 16, 15, 42), Some(106));
    // Corner (mask W|S): both planes present.
    assert_eq!(fine_at(&layer, 6, 5, 20, 15, 42), Some(106), "W-E panel");
    assert_eq!(fine_at(&layer, 6, 5, 15, 20, 42), Some(106), "N-S panel");
    // South leg (mask N only): panel across the tile.
    assert_eq!(fine_at(&layer, 6, 6, 16, 20, 42), Some(106));
    // Isolated fence: no volume, coarse block kept.
    assert!(layer.volumes_for_tile((9, 9)).is_empty());
    assert_eq!(slab_id(&map, 9, 9, 0), TER_CHAINFENCE);
    println!("terrain wire gate: ground/fence/standing/roof seating by material id");
}

#[test]
fn oversize_terrain_model_fails_before_any_stamp() {
    let defs = sample_terrain_tiledefs();
    let snap = sample_terrain_snapshot();
    let fx = Fixture::new();
    let scale = WorldScale::default();
    let mut map = build_snapshot_world(&defs, &snap, scale, &coarse_block);
    let mut layer = FineLayer::new(96);
    let wide = PropModelData {
        size: [33, 10, 32],
        palette: vec![[10, 200, 10]],
        voxels: vec![0; 33 * 10 * 32],
    };
    let resolve = |id: &str| match id {
        "t_grass" => Some((&wide, ModelKind::Ground)),
        other => fx.resolve(other),
    };
    let err = stamp_snapshot_terrain(
        &mut map,
        &mut layer,
        &defs,
        &snap,
        scale,
        &resolve,
        &colour_block,
    )
    .unwrap_err();
    assert_eq!(
        err,
        mesher::fine::StampError::ExceedsTile { size: [33, 10, 32] }
    );
    // Nothing stamped, nothing aired.
    assert!(layer.is_empty());
    assert_eq!(slab_id(&map, 0, 0, 0), TER_GRASS);
    assert_eq!(slab_id(&map, 3, 8, 0), TER_FLOOR);
    println!("terrain wire gate: oversize model rejected atomically");
}
