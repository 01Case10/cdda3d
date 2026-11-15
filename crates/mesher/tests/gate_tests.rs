//! Phase 2 gate tests 2.1-2.6 and 2.9 (Part II §5) for the mesher crate.

use mesher::block::{block_for_tile, Pass};
use mesher::chunk::{ChunkMap, CHUNK_W};
use mesher::grid::VoxelGrid;
use mesher::mesh::covered_face_cells;
use mesher::mesher::{mesh_greedy, mesh_naive};
use mesher::scale::WorldScale;
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

fn scale() -> WorldScale {
    WorldScale::default()
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ---------- 2.1: single block mesh ----------
#[test]
fn test_2_1_single_block_six_faces_correct_normals_and_winding() {
    let mut grid = VoxelGrid::new([1, 1, 1]);
    grid.set(0, 0, 0, block_for_tile(2)); // wall, opaque
    for meshes in [
        mesh_naive(&grid, [0, 0, 0], grid.size, scale()),
        mesh_greedy(&grid, [0, 0, 0], grid.size, scale()),
    ] {
        let m = meshes.get(Pass::Opaque);
        assert_eq!(m.quad_count(), 6, "one block = 6 faces");
        assert_eq!(m.vertex_count(), 24);
        assert_eq!(m.indices.len(), 36);
        // Every quad: stored normal is a unit axis, and the geometric
        // normal from the winding agrees with it (CCW front face).
        let mut normals_seen = std::collections::HashSet::new();
        for quad in m.indices.chunks_exact(6) {
            let p = |i: u32| m.positions[i as usize];
            let (a, b, c) = (p(quad[0]), p(quad[1]), p(quad[2]));
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
            let geo = cross(ab, bc);
            let n = m.normals[quad[0] as usize];
            let len = (geo[0] * geo[0] + geo[1] * geo[1] + geo[2] * geo[2]).sqrt();
            assert!(len > 0.99 && len < 1.01, "unit quad, |cross|={len}");
            let dot = geo[0] * n[0] + geo[1] * n[1] + geo[2] * n[2];
            assert!(dot > 0.99, "winding must face along normal, dot={dot}");
            normals_seen.insert([n[0] as i32, n[1] as i32, n[2] as i32]);
        }
        assert_eq!(normals_seen.len(), 6, "all six axis directions present");
        assert!((m.surface_area() - 6.0).abs() < 1e-5);
    }
}

// ---------- 2.2: adjacent blocks cull shared faces ----------
#[test]
fn test_2_2_adjacent_blocks_remove_shared_faces() {
    let mut grid = VoxelGrid::new([2, 1, 1]);
    grid.set(0, 0, 0, block_for_tile(2));
    grid.set(1, 0, 0, block_for_tile(2));
    let naive = mesh_naive(&grid, [0, 0, 0], grid.size, scale());
    // Two separate blocks would have 12 faces; the 2 shared faces go away.
    assert_eq!(naive.get(Pass::Opaque).quad_count(), 10);
    assert!((naive.get(Pass::Opaque).surface_area() - 10.0).abs() < 1e-5);
    let greedy = mesh_greedy(&grid, [0, 0, 0], grid.size, scale());
    assert!((greedy.get(Pass::Opaque).surface_area() - 10.0).abs() < 1e-5);
}

// ---------- 2.3: greedy merge of a 16x16 slab ----------
#[test]
fn test_2_3_greedy_merges_16x16_slab_to_one_quad_per_large_face() {
    let mut grid = VoxelGrid::new([16, 16, 1]);
    grid.fill(block_for_tile(2));
    let greedy = mesh_greedy(&grid, [0, 0, 0], grid.size, scale());
    let m = greedy.get(Pass::Opaque);
    // Top, bottom, and 4 sides: each is a single merged quad.
    assert_eq!(m.quad_count(), 6, "one quad per visible large face");
    assert_eq!(m.vertex_count(), 24, "6 quads * 4 vertices");
    assert_eq!(m.indices.len(), 36);
    assert!((m.surface_area() - (16.0 * 16.0 * 2.0 + 16.0 * 4.0)).abs() < 1e-4);
    // Contrast: the naive mesher emits every unit face.
    let naive = mesh_naive(&grid, [0, 0, 0], grid.size, scale());
    assert_eq!(
        naive.get(Pass::Opaque).quad_count(),
        16 * 16 * 2 + 16 * 4,
        "naive keeps all unit faces"
    );
}

// ---------- 2.4: chunk boundaries ----------
#[test]
fn test_2_4_boundary_face_culled_only_when_neighbor_chunk_has_opaque_block() {
    let s = scale();
    // Case A: neighbor chunk exists but the adjacent cell is air -> the
    // boundary face must be emitted.
    let mut map = ChunkMap::new(s);
    map.add_chunk([0, 0], 1);
    map.add_chunk([1, 0], 1);
    map.set_global_block(11, 5, 0, block_for_tile(2));
    map.clear_dirty();
    let meshes = map.mesh_chunk([0, 0], true).unwrap();
    let (cells, _) = covered_face_cells(meshes.get(Pass::Opaque));
    assert!(
        cells.contains(&([11, 5, 0], 0)),
        "+X boundary face must be present when neighbor cell is air"
    );

    // Case B: neighbor chunk has an opaque block in the adjacent cell ->
    // the boundary face (and the neighbor's facing face) must be culled.
    map.set_global_block(12, 5, 0, block_for_tile(2));
    let meshes = map.mesh_chunk([0, 0], true).unwrap();
    let (cells, _) = covered_face_cells(meshes.get(Pass::Opaque));
    assert!(
        !cells.contains(&([11, 5, 0], 0)),
        "+X boundary face must be culled by the neighbor chunk's opaque block"
    );
    let meshes_e = map.mesh_chunk([1, 0], true).unwrap();
    let (cells_e, _) = covered_face_cells(meshes_e.get(Pass::Opaque));
    assert!(
        !cells_e.contains(&([12, 5, 0], 1)),
        "-X face of the neighbor block must also be culled"
    );

    // Case C: neighbor chunk absent entirely -> face present.
    let mut map2 = ChunkMap::new(s);
    map2.add_chunk([0, 0], 1);
    map2.set_global_block(11, 5, 0, block_for_tile(2));
    let meshes2 = map2.mesh_chunk([0, 0], true).unwrap();
    let (cells2, _) = covered_face_cells(meshes2.get(Pass::Opaque));
    assert!(cells2.contains(&([11, 5, 0], 0)));

    // Case D: neighbor cell holds glass (transparent, not opaque) -> the
    // opaque block's boundary face is still visible through it.
    let mut map3 = ChunkMap::new(s);
    map3.add_chunk([0, 0], 1);
    map3.add_chunk([1, 0], 1);
    map3.set_global_block(11, 5, 0, block_for_tile(2));
    map3.set_global_block(12, 5, 0, block_for_tile(4)); // window glass
    let meshes3 = map3.mesh_chunk([0, 0], true).unwrap();
    let (cells3, _) = covered_face_cells(meshes3.get(Pass::Opaque));
    assert!(
        cells3.contains(&([11, 5, 0], 0)),
        "transparent neighbor must not cull an opaque face"
    );
}

// ---------- 2.5: property — greedy equals naive ----------
proptest! {
    #![proptest_config(ProptestConfig { cases: 48, rng_seed: RngSeed::Fixed(0xCDDA_0002), ..ProptestConfig::default() })]

    #[test]
    fn test_2_5_greedy_equals_naive_random_chunks(
        cells in prop::collection::vec(0u32..10, 6 * 6 * 4),
    ) {
        let mut grid = VoxelGrid::new([6, 6, 4]);
        for (i, id) in cells.iter().enumerate() {
            let (x, y, z) = (i % 6, (i / 6) % 6, i / 36);
            // Map the random id range onto air + a mix of passes.
            let block = match id {
                0..=4 => mesher::block::Block::AIR,
                5 => block_for_tile(2), // wall opaque
                6 => block_for_tile(1), // floor opaque (different color/id)
                7 => block_for_tile(4), // glass transparent
                8 => block_for_tile(7), // fence cutout
                _ => block_for_tile(8), // water transparent
            };
            grid.set(x, y, z, block);
        }
        let naive = mesh_naive(&grid, [0, 0, 0], grid.size, scale());
        let greedy = mesh_greedy(&grid, [0, 0, 0], grid.size, scale());
        for pass in Pass::ALL {
            let (n, g) = (naive.get(pass), greedy.get(pass));
            // Same visible surface area.
            prop_assert!((n.surface_area() - g.surface_area()).abs() < 1e-3,
                "area mismatch in {:?}: naive {} greedy {}", pass, n.surface_area(), g.surface_area());
            // Same covered face-cells.
            let (n_cells, _) = covered_face_cells(n);
            let (g_cells, g_expanded) = covered_face_cells(g);
            prop_assert_eq!(&n_cells, &g_cells, "covered cells differ in {:?}", pass);
            // No overlapping quads in the greedy output: the expanded
            // unit-cell count equals the distinct-cell count.
            prop_assert_eq!(g_expanded, g_cells.len(), "greedy quads overlap in {:?}", pass);
            // Greedy never uses more quads than naive.
            prop_assert!(g.quad_count() <= n.quad_count());
        }
    }
}

// ---------- 2.6: transparent / cutout passes ----------
#[test]
fn test_2_6_pass_classification() {
    let mut grid = VoxelGrid::new([5, 1, 1]);
    grid.set(0, 0, 0, block_for_tile(2)); // wall -> opaque
    grid.set(1, 0, 0, block_for_tile(7)); // fence -> cutout
    grid.set(2, 0, 0, block_for_tile(9)); // foliage -> cutout
    grid.set(3, 0, 0, block_for_tile(4)); // glass -> transparent
    grid.set(4, 0, 0, block_for_tile(8)); // water -> transparent
    for meshes in [
        mesh_naive(&grid, [0, 0, 0], grid.size, scale()),
        mesh_greedy(&grid, [0, 0, 0], grid.size, scale()),
    ] {
        assert!(meshes.get(Pass::Opaque).quad_count() > 0);
        // Fence (5 visible faces: its face against the opaque wall is
        // culled) + foliage (6) = 11 cutout faces.
        assert_eq!(meshes.get(Pass::Cutout).quad_count(), 11);
        // Glass and water are different ids, so their shared face shows
        // on both: 6 + 6 transparent faces.
        assert_eq!(meshes.get(Pass::Transparent).quad_count(), 12);
        // No block leaks into another pass: total faces accounted for.
        assert_eq!(meshes.total_quads(), 6 + 11 + 12);
    }
    // Two adjacent glass blocks (same id) share no internal face.
    let mut grid2 = VoxelGrid::new([2, 1, 1]);
    grid2.set(0, 0, 0, block_for_tile(4));
    grid2.set(1, 0, 0, block_for_tile(4));
    let m = mesh_naive(&grid2, [0, 0, 0], grid2.size, scale());
    assert_eq!(m.get(Pass::Transparent).quad_count(), 10);
}

// ---------- 2.9: dirty propagation ----------
#[test]
fn test_2_9_dirty_propagation_boundary_and_interior() {
    let s = scale();
    let mut map = ChunkMap::new(s);
    for cx in 0..3 {
        for cy in 0..3 {
            map.add_chunk([cx, cy], 1);
        }
    }
    map.clear_dirty();
    assert!(map.dirty_coords().is_empty());

    // Interior edit of the center chunk: only that chunk goes dirty.
    let marked = map.set_global_block(12 + 5, 12 + 5, 0, block_for_tile(2));
    assert_eq!(marked.len(), 1);
    assert!(marked.contains(&[1, 1]));
    assert_eq!(map.dirty_coords().len(), 1);

    map.clear_dirty();

    // Boundary edit on the west edge of chunk (1,1): chunks (1,1) and
    // (0,1) go dirty; all others stay clean.
    let marked = map.set_global_block(12, 12 + 5, 0, block_for_tile(2));
    assert_eq!(marked.len(), 2, "owner + west neighbor: {marked:?}");
    assert!(marked.contains(&[1, 1]) && marked.contains(&[0, 1]));
    let dirty = map.dirty_coords();
    assert!(!dirty.contains(&[2, 1]), "unrelated chunk must stay clean");
    assert!(!dirty.contains(&[1, 0]));
    assert!(!dirty.contains(&[1, 2]));
    assert!(!dirty.contains(&[2, 2]));

    map.clear_dirty();

    // Corner edit: owner + both edge-adjacent neighbors.
    let marked = map.set_global_block(12, 12, 0, block_for_tile(2));
    assert_eq!(marked.len(), 3, "owner + west + north: {marked:?}");
    assert!(marked.contains(&[1, 1]) && marked.contains(&[0, 1]) && marked.contains(&[1, 0]));
    assert_eq!(CHUNK_W, 12, "chunk width is the submap size");
}
