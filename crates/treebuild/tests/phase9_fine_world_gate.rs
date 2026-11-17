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

// ---------------------------------------------------------------------
// Phase 9 slice 3f (9.5 CPU two-tree trace): nearer hit wins
// ---------------------------------------------------------------------

use glam::Vec3;
use treebuild::build::BuildConfig;
use treebuild::cpu::{cast_ray, cast_ray_two, TraceMode};
use treebuild::world::VoxelCell;

fn coarse_wall_world() -> VoxelWorld {
    // One wall block at block (4, 0, 1), material 7.
    let mut w = VoxelWorld::new();
    w.set([4, 0, 1], VoxelCell::new([0.5, 0.5, 0.5], 7));
    w
}

fn fine_prop_layer() -> FineLayer {
    // A 2x1x2 prop (material 1001) on tile (1, 0) at fine z 32:
    // fine x 32..34, fine y 30..32, fine z 32 — i.e. block
    // x 1.0..1.0625, y 0.9375..1.0, z 1.0..1.03125.
    let mut layer = FineLayer::default();
    stamp_model(&mut layer, (1, 0), 32, &TestModel::filled([2, 1, 2], blk(1001, Pass::Opaque))).unwrap();
    layer
}

#[test]
fn two_tree_nearer_fine_hit_wins_and_miss_falls_through() {
    let coarse = treebuild::build_tree(
        &coarse_wall_world(),
        BuildConfig { depth: None, origin: Some([0, 0, 0]) },
    )
    .unwrap();
    let fine_world = VoxelWorld::from_fine_layer(&fine_prop_layer());
    let fine = treebuild::build_tree(
        &fine_world,
        BuildConfig { depth: None, origin: Some([0, 0, 0]) },
    )
    .unwrap();
    let rd = Vec3::new(1.0, 0.0, 0.0);

    // Ray through the prop (y 0.97, z 1.01 in block units):
    // the fine hit at block x = 1.0 beats the wall at x = 4.
    let (hit, is_fine) = cast_ray_two(&coarse, Some(&fine), Vec3::new(0.0, 0.97, 1.01), rd, 100.0, TraceMode::FirstHit)
        .expect("prop hit");
    assert!(is_fine, "the nearer fine hit wins");
    assert_eq!(hit.material_id, 1001);
    assert!((hit.dist - 1.0).abs() < 1e-3, "fine dist in block units: {}", hit.dist);
    assert_eq!(hit.voxel, [32, 31, 32], "fine hit voxel is in fine coords");
    assert_eq!(hit.normal, [-1, 0, 0]);

    // Ray above the prop (z 1.5): falls through to the wall.
    let (hit, is_fine) = cast_ray_two(&coarse, Some(&fine), Vec3::new(0.0, 0.5, 1.5), rd, 100.0, TraceMode::FirstHit)
        .expect("wall hit");
    assert!(!is_fine);
    assert_eq!(hit.material_id, 7);
    assert!((hit.dist - 4.0).abs() < 1e-3);
    assert_eq!(hit.voxel, [4, 0, 1]);

    // Ray that misses everything (pointing -x, or above both).
    assert!(cast_ray_two(&coarse, Some(&fine), Vec3::new(0.0, 0.5, 1.5), Vec3::new(-1.0, 0.0, 0.0), 100.0, TraceMode::FirstHit).is_none());
    assert!(cast_ray_two(&coarse, Some(&fine), Vec3::new(0.0, 5.5, 1.5), rd, 100.0, TraceMode::FirstHit).is_none());

    // max_dist in block units applies to the fine tree too:
    // the prop at dist 1.0 is beyond a 0.5 limit.
    assert!(cast_ray_two(&coarse, Some(&fine), Vec3::new(0.0, 0.97, 1.01), rd, 0.5, TraceMode::FirstHit).is_none());
}

#[test]
fn two_tree_without_fine_is_exactly_cast_ray() {
    let coarse = treebuild::build_tree(
        &coarse_wall_world(),
        BuildConfig { depth: None, origin: Some([0, 0, 0]) },
    )
    .unwrap();
    let empty_fine = treebuild::build_tree(
        &VoxelWorld::from_fine_layer(&FineLayer::default()),
        BuildConfig { depth: None, origin: Some([0, 0, 0]) },
    );
    for ro in [Vec3::new(0.0, 0.5, 1.5), Vec3::new(0.0, 0.97, 1.01), Vec3::new(9.0, 0.5, 1.5)] {
        let want = cast_ray(&coarse, ro, Vec3::new(1.0, 0.0, 0.0), 100.0, TraceMode::FirstHit);
        for fine in [None, empty_fine.as_ref().ok()] {
            let got = cast_ray_two(&coarse, fine, ro, Vec3::new(1.0, 0.0, 0.0), 100.0, TraceMode::FirstHit);
            match (want, got) {
                (None, None) => {}
                (Some(w), Some((g, is_fine))) => {
                    assert!(!is_fine);
                    assert_eq!(g, w, "no-fine two-tree hit must equal cast_ray exactly");
                }
                (w, g) => panic!("mismatch: cast_ray {w:?} vs two-tree {g:?}"),
            }
        }
    }
}
