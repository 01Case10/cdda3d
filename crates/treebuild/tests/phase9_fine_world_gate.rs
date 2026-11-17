//! Phase 9 slice 3e (9.5 adapter half): `VoxelWorld::from_fine_layer`
//! — the fine tree's world contains ONLY stamped cells, in fine
//! coordinates, with the stamped block's material/colour/flags.

use mesher::block::Block;
use mesher::fine::{stamp_model, FineLayer, ModelCells, FINE};
use mesher::Pass;
use treebuild::world::{VoxelWorld, FLAG_CUTOUT, FLAG_TRANSPARENT, LIT_LIT};

struct TestModel {
    size: [u32; 3],
    cells: Vec<Option<Block>>,
}

impl TestModel {
    fn filled(size: [u32; 3], block: Block) -> Self {
        let n = (size[0] * size[1] * size[2]) as usize;
        Self {
            size,
            cells: vec![Some(block); n],
        }
    }
}

impl ModelCells for TestModel {
    fn size(&self) -> [u32; 3] {
        self.size
    }
    fn cell(&self, x: u32, y: u32, z: u32) -> Option<Block> {
        let [w, h, d] = self.size;
        if x >= w || y >= h || z >= d {
            return None;
        }
        self.cells[((y * d + z) * w + x) as usize]
    }
}

fn blk(id: u32, pass: Pass) -> Block {
    Block {
        id,
        color: [0.25, 0.5, 0.75, 0.9],
        pass,
        tex_layers: Some([7, 7, 7]),
    }
}

#[test]
fn fine_world_contains_only_stamped_cells_at_fine_coords() {
    let mut layer = FineLayer::default();
    // Empty layer -> empty fine world (no fine tree downstream).
    assert!(VoxelWorld::from_fine_layer(&layer).is_empty());

    let opaque = blk(1001, Pass::Opaque);
    let glass = blk(1002, Pass::Transparent);
    // 4x3x2 opaque model on tile (2,3) at slab top (fine z 32),
    // 1x1x1 transparent model on the negative tile (-1,-1).
    stamp_model(&mut layer, (2, 3), 32, &TestModel::filled([4, 3, 2], opaque)).unwrap();
    stamp_model(&mut layer, (-1, -1), 32, &TestModel::filled([1, 1, 1], glass)).unwrap();

    let world = VoxelWorld::from_fine_layer(&layer);
    assert_eq!(world.len(), 24 + 1, "only stamped solid cells enter the fine world");

    // Every cell of the first model, at its fine coordinate:
    // model (mx, my, mz) -> fine (2*32+mx, 3*32 + (32-2) + mz, 32+my).
    for my in 0..3 {
        for mz in 0..2 {
            for mx in 0..4 {
                let pos = [2 * FINE + mx, 3 * FINE + 30 + mz, 32 + my];
                let cell = world.get(pos).unwrap_or_else(|| panic!("missing {pos:?}"));
                assert_eq!(cell.material_id, 1001);
                assert_eq!(cell.color, [0.25, 0.5, 0.75]);
                assert_eq!(cell.alpha, 0.9);
                assert_eq!(cell.lit, LIT_LIT);
                assert_eq!(cell.flags, 0, "opaque carries no flags");
            }
        }
    }
    // The negative tile's single cell: front flush south-west
    // corner mapping — (mx 0, mz 0) of a depth-1 model lands at
    // local (0, 31).
    let neg = world.get([-FINE, -FINE + 31, 32]).expect("negative-tile cell");
    assert_eq!(neg.material_id, 1002);
    assert!(neg.flags & FLAG_TRANSPARENT != 0, "transparent pass maps to its flag");
    assert!(neg.flags & FLAG_CUTOUT == 0);

    // Nothing else: unstamped fine cells of a stamped tile, the
    // coarse slab below the volume, and neighbouring tiles are
    // all absent (the coarse tree owns those).
    assert!(world.get([2 * FINE, 3 * FINE, 32]).is_none(), "unstamped corner");
    assert!(world.get([2 * FINE, 3 * FINE + 31, 0]).is_none(), "slab is coarse, not fine");
    assert!(world.get([4 * FINE, 3 * FINE + 31, 32]).is_none(), "neighbour tile");
}

#[test]
fn fine_world_builds_a_tree_with_one_leaf_per_stamped_cell() {
    let mut layer = FineLayer::default();
    stamp_model(
        &mut layer,
        (0, 0),
        32,
        &TestModel::filled([4, 3, 2], blk(1001, Pass::Opaque)),
    )
    .unwrap();
    let world = VoxelWorld::from_fine_layer(&layer);
    let built = treebuild::build_tree(&world, treebuild::BuildConfig::default()).unwrap();
    assert_eq!(built.voxel_count, 24);
    assert_eq!(built.leaf_count(), 24);
    // Side-table verification against the fine world passes
    // (gate 2c.2's mechanism, at fine coordinates).
    treebuild::build::verify_side_table(&built, &world).unwrap();
}
