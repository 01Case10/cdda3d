//! Phase 2d gates (plan Part II §5d), treebuild half:
//! 2d.1 connection masks over known wall layouts, and the
//! virtual-material assignment that carries the selection into the
//! tree's side-table. Layer selection itself (2d.2) and render
//! evidence (2d.3) are engine gates — the engine owns materials.

use treebuild::build::{build_tree, verify_side_table, BuildConfig};
use treebuild::connect::{assign_variants, connection_mask, same_material};
use treebuild::world::{VoxelCell, VoxelWorld};

const WALL: u32 = 7;
const OTHER: u32 = 9;

fn cell(mat: u32) -> VoxelCell {
    VoxelCell::new([0.5, 0.5, 0.5], mat)
}

/// All six layouts in one world, spaced apart so no layout's voxels
/// neighbor another's. Coordinates are (x, y) at z = 0.
fn layouts_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    let put = |w: &mut VoxelWorld, pts: &[(i32, i32)]| {
        for &(x, y) in pts {
            w.set([x, y, 0], cell(WALL));
        }
    };
    put(&mut w, &[(0, 0)]); // isolated
    put(&mut w, &[(10, 0), (11, 0), (12, 0)]); // straight E-W run
    put(&mut w, &[(20, 0), (20, 1), (20, 2)]); // straight N-S run
    put(&mut w, &[(30, 0), (31, 0), (31, 1)]); // corner
    put(&mut w, &[(41, 1), (40, 1), (42, 1), (41, 0)]); // T
    put(&mut w, &[(51, 1), (50, 1), (52, 1), (51, 0), (51, 2)]); // cross
    w
}

#[test]
fn gate_2d_1_connection_masks_match_hand_computed_layouts() {
    let w = layouts_world();
    // (pos, expected mask) — bits N=1, E=2, S=4, W=8; north is -y.
    let expected: &[([i32; 3], u8)] = &[
        // Isolated pillar: nothing connects.
        ([0, 0, 0], 0),
        // E-W run: ends are end caps, middle connects E+W.
        ([10, 0, 0], 2),
        ([11, 0, 0], 10),
        ([12, 0, 0], 8),
        // N-S run.
        ([20, 0, 0], 4),
        ([20, 1, 0], 5),
        ([20, 2, 0], 1),
        // Corner at (31,0): connects W and S.
        ([30, 0, 0], 2),
        ([31, 0, 0], 12),
        ([31, 1, 0], 1),
        // T center (41,1): N+E+W, open to the south.
        ([41, 1, 0], 11),
        ([40, 1, 0], 2),
        ([42, 1, 0], 8),
        ([41, 0, 0], 4),
        // Cross center: all four.
        ([51, 1, 0], 15),
        ([50, 1, 0], 2),
        ([52, 1, 0], 8),
        ([51, 0, 0], 4),
        ([51, 2, 0], 1),
    ];
    for (pos, want) in expected {
        assert_eq!(
            connection_mask(&w, *pos, same_material),
            *want,
            "mask at {pos:?}"
        );
    }
    // Absent voxel: mask 0 by definition.
    assert_eq!(connection_mask(&w, [99, 99, 0], same_material), 0);
}

#[test]
fn gate_2d_1_connection_needs_same_material_same_z_cardinal_only() {
    let mut w = VoxelWorld::new();
    // Different material next door does not connect (fake rule).
    w.set([60, 0, 0], cell(WALL));
    w.set([61, 0, 0], cell(OTHER));
    assert_eq!(connection_mask(&w, [60, 0, 0], same_material), 0);
    assert_eq!(connection_mask(&w, [61, 0, 0], same_material), 0);
    // Diagonals do not connect.
    w.set([70, 0, 0], cell(WALL));
    w.set([71, 1, 0], cell(WALL));
    assert_eq!(connection_mask(&w, [70, 0, 0], same_material), 0);
    assert_eq!(connection_mask(&w, [71, 1, 0], same_material), 0);
    // A neighbor one z up does not connect sideways.
    w.set([80, 0, 0], cell(WALL));
    w.set([81, 0, 1], cell(WALL));
    assert_eq!(connection_mask(&w, [80, 0, 0], same_material), 0);
    assert_eq!(connection_mask(&w, [81, 0, 1], same_material), 0);
    // A full-height stack gets the same mask at every block: two
    // adjacent 3-high wall columns connect E/W at z = 0, 1, 2.
    for z in 0..3 {
        w.set([90, 0, z], cell(WALL));
        w.set([91, 0, z], cell(WALL));
    }
    for z in 0..3 {
        assert_eq!(connection_mask(&w, [90, 0, z], same_material), 2, "z {z}");
        assert_eq!(connection_mask(&w, [91, 0, z], same_material), 8, "z {z}");
    }
    // A custom predicate is honored (here: everything connects) —
    // this is the seam the TileDefs connect-group rule plugs into.
    let all = |_: &VoxelCell, _: &VoxelCell| true;
    assert_eq!(connection_mask(&w, [60, 0, 0], all), 2);
}

#[test]
fn gate_2d_assignment_allocates_virtual_ids_deterministically() {
    let build = || {
        let mut w = VoxelWorld::new();
        for x in 10..=12 {
            w.set([x, 0, 0], cell(WALL));
        }
        w.set([0, 0, 0], cell(WALL)); // isolated, mask 0
        w.set([5, 5, 0], cell(OTHER)); // non-multitile bystander
        w
    };
    let is_multi = |id: u32| id == WALL;

    let mut w1 = build();
    let a1 = assign_variants(&mut w1, is_multi, same_material);
    // Distinct pairs sorted by (base, mask); ids from max_id+1 = 10.
    assert_eq!(
        a1.iter()
            .map(|a| (a.base_id, a.mask, a.virtual_id))
            .collect::<Vec<_>>(),
        vec![(7, 0, 10), (7, 2, 11), (7, 8, 12), (7, 10, 13)]
    );
    // Voxels rewritten to their pair's virtual id.
    assert_eq!(
        w1.get([0, 0, 0]).unwrap().material_id,
        10,
        "isolated -> mask 0"
    );
    assert_eq!(
        w1.get([10, 0, 0]).unwrap().material_id,
        11,
        "west end -> mask E"
    );
    assert_eq!(
        w1.get([11, 0, 0]).unwrap().material_id,
        13,
        "middle -> mask E+W"
    );
    assert_eq!(
        w1.get([12, 0, 0]).unwrap().material_id,
        12,
        "east end -> mask W"
    );
    // Non-multitile voxel untouched.
    assert_eq!(w1.get([5, 5, 0]).unwrap().material_id, OTHER);

    // Same world, same assignment (ids are not HashMap-order luck).
    let mut w2 = build();
    let a2 = assign_variants(&mut w2, is_multi, same_material);
    assert_eq!(a1, a2);

    // The virtual ids ride into the tree's side-table: the built
    // tree verifies against the rewritten world, and the middle
    // voxel's leaf carries the mask-10 virtual id.
    let built = build_tree(&w1, BuildConfig::default()).unwrap();
    verify_side_table(&built, &w1).unwrap();
    let mut found = false;
    for idx in 0..built.node_count() {
        if let Some(voxel) = built.leaf_voxel(idx) {
            if voxel == [11, 0, 0] {
                assert_eq!(built.side[idx].material_id, 13);
                found = true;
            }
        }
    }
    assert!(found, "run-middle leaf must exist in the tree");
}
