//! Stage H gate (8c-A, 2026-11-23), RT half: the geometric
//! fine-prop occlusion step-down over the RT fine world.
//! The raster half is gated in mesher's
//! `fine_occlusion_gate`.
//!
//! - a lone prop cell on an open slab (mean exposed-face
//!   factor 0.912) keeps its lit level;
//! - a prop cell walled in on all four sides by coarse
//!   full-height tiles (only its top face exposed, factor
//!   0.56) steps down two lit levels;
//! - a Memorized cell is never stepped (Phase 7 contract).

use engine::materials::PropModelData;
use engine::rt::fine_occlusion_rewrite;
use engine::snapshot_world::PropModelCells;
use mesher::chunk::ChunkMap;
use mesher::fine::{stamp_model, FineLayer};
use mesher::scale::WorldScale;
use mesher::{Block, Pass};
use treebuild::world::{VoxelWorld, LIT_BRIGHT, LIT_LIT, LIT_LOW, LIT_MEMORIZED};

fn blk(id: u32) -> Block {
    Block {
        id,
        color: [0.2, 0.2, 0.2, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

fn prop_block(rgb: [u8; 3]) -> Block {
    Block {
        id: 77,
        color: [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// One prop cell stamped on tile (4,4)'s slab at local
/// (20, 20), fine z 32. With `ring`, four two-high prop
/// cells are stamped around it (fine z 32 and 33), walling
/// it in: only its top face stays exposed, and that face's
/// air cell is boxed on all four sides by the ring's
/// second layer (factor 0.56).
fn scene(ring: bool) -> (ChunkMap, FineLayer, VoxelWorld) {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    map.set_global_block(4, 4, 0, blk(10));
    map.clear_dirty();
    let mut model = PropModelData {
        size: [32, 2, 32],
        palette: vec![[200, 100, 50]],
        voxels: vec![PropModelData::EMPTY; 32 * 2 * 32],
    };
    let set = |model: &mut PropModelData, x: u32, y: u32, z: u32| {
        model.voxels[((y * 32 + z) * 32 + x) as usize] = 0;
    };
    set(&mut model, 20, 0, 20);
    if ring {
        for (x, z) in [(19, 20), (21, 20), (20, 19), (20, 21)] {
            set(&mut model, x, 0, z);
            set(&mut model, x, 1, z);
        }
    }
    let adapter = PropModelCells::new(&model, &prop_block);
    let mut layer = FineLayer::default();
    stamp_model(&mut layer, (4, 4), 32, &adapter).unwrap();
    let world = VoxelWorld::from_fine_layer(&layer);
    (map, layer, world)
}

#[test]
fn rt_occlusion_steps_down_only_enclosed_cells() {
    let pos = [4 * 32 + 20, 4 * 32 + 20, 32];
    // Lone prop: no step.
    let (map, layer, world) = scene(false);
    assert_eq!(world.get(pos).unwrap().lit, LIT_LIT);
    let out = fine_occlusion_rewrite(&world, &map, &layer);
    assert_eq!(out.get(pos).unwrap().lit, LIT_LIT, "open prop keeps its light");
    // Walled-in prop: two steps down (LIT_LIT -> LIT_LOW).
    let (map, layer, world) = scene(true);
    let out = fine_occlusion_rewrite(&world, &map, &layer);
    assert_eq!(out.get(pos).unwrap().lit, LIT_LOW, "walled-in prop steps down two");
    // Memorized presentation is pinned: never stepped,
    // even in the walled scene.
    let (map, layer, mut world) = scene(true);
    let mut cell = *world.get(pos).unwrap();
    cell.lit = LIT_MEMORIZED;
    world.set(pos, cell);
    let out = fine_occlusion_rewrite(&world, &map, &layer);
    assert_eq!(out.get(pos).unwrap().lit, LIT_MEMORIZED);
    // Bright cells step from their own level (4 - 2 = 2).
    let (map, layer, mut world) = scene(true);
    let mut cell = *world.get(pos).unwrap();
    cell.lit = LIT_BRIGHT;
    world.set(pos, cell);
    let out = fine_occlusion_rewrite(&world, &map, &layer);
    assert_eq!(out.get(pos).unwrap().lit, 2, "bright walled cell steps 4 -> 2");
}
