//! Gate tests for Phase 2f (plan Part II §5f), mesher half:
//! 2f.1 — UV/layer output. A single-material chunk's faces must
//! carry UVs covering the material layer for their face class
//! (top/side/bottom), and greedy merging must never merge faces
//! with different layers (the merge key is the whole `Block`,
//! whose equality now includes `tex_layers`).
//!
//! Stage A amendment (procedural texture pipeline rework): UVs
//! are in TILE UNITS. A single tile's face spans [0,1]²; a
//! greedy-merged quad spanning w x h tiles spans [0,w] x [0,h],
//! and the engine's Repeat sampler wraps each integer interval to
//! one full texture per tile. The original 2f.1 expectation — a
//! merged quad's UVs span [0,1]² across its whole extent — was
//! the sprite-stretching bug Stage A fixes, so the span checks
//! below assert span == quad extent in tiles instead.

use mesher::block::{Block, Pass};
use mesher::grid::VoxelGrid;
use mesher::mesh::{covered_face_cells, Mesh, PassMeshes, FLAT_LAYER};
use mesher::mesher::{mesh_greedy, mesh_naive};
use mesher::scale::WorldScale;

fn scale() -> WorldScale {
    WorldScale::default()
}

/// One material: top layer 10, side 11, bottom 12.
fn textured_block() -> Block {
    Block {
        id: 42,
        color: [0.7, 0.68, 0.62, 1.0],
        pass: Pass::Opaque,
        tex_layers: Some([10, 11, 12]),
    }
}

/// Grid-space face class of a quad from its world normal (inverse
/// of the mesher's (x, z, -y) rotation, as in covered_face_cells).
/// Returns the layer the material assigns to that class.
fn expected_layer(mesh: &Mesh, quad: &[u32]) -> u32 {
    let n = mesh.normals[quad[0] as usize];
    let gn = [n[0], -n[2], n[1]];
    if gn[2] > 0.5 {
        10 // +Z: top
    } else if gn[2] < -0.5 {
        12 // -Z: bottom
    } else {
        11 // sides
    }
}

fn check_mesh_uvs(meshes: &PassMeshes, label: &str) {
    for mesh in meshes.meshes.iter() {
        assert_eq!(
            mesh.uvs.len(),
            mesh.positions.len(),
            "{label}: uvs parallel"
        );
        assert_eq!(
            mesh.layers.len(),
            mesh.positions.len(),
            "{label}: layers parallel"
        );
        for quad in mesh.indices.chunks_exact(6) {
            let want = expected_layer(mesh, quad);
            let verts: Vec<usize> = quad[..4].iter().map(|&i| i as usize).collect();
            // One layer per quad, and it is the material's layer
            // for this face class.
            for &v in &verts {
                assert_eq!(mesh.layers[v], want, "{label}: layer for face class");
            }
            // UV span equals the quad's extent IN TILES (Stage A
            // tile-unit UVs): one full texture repeat per tile,
            // anchored on integer boundaries.
            let (mut u_min, mut u_max) = (f32::MAX, f32::MIN);
            let (mut v_min, mut v_max) = (f32::MAX, f32::MIN);
            for &v in &verts {
                let [u, vv] = mesh.uvs[v];
                u_min = u_min.min(u);
                u_max = u_max.max(u);
                v_min = v_min.min(vv);
                v_max = v_max.max(vv);
            }
            // In-plane world extents of the quad (block_size = 1
            // at the default scale, so world units == tiles).
            let n = mesh.normals[quad[0] as usize];
            let n_axis = (0..3).max_by_key(|&a| (n[a].abs() * 1000.0) as i32).unwrap();
            let plane: Vec<usize> = (0..3).filter(|&a| a != n_axis).collect();
            let extent = |axis: usize| {
                let (mut lo, mut hi) = (f32::MAX, f32::MIN);
                for &v in &verts {
                    lo = lo.min(mesh.positions[v][axis]);
                    hi = hi.max(mesh.positions[v][axis]);
                }
                hi - lo
            };
            let (eu, ev) = (extent(plane[0]), extent(plane[1]));
            // Which plane axis feeds u depends on the face class;
            // the pair {du, dv} must equal the pair {eu, ev}.
            let (du, dv) = (u_max - u_min, v_max - v_min);
            let spans_match = ((du - eu).abs() < 1e-5 && (dv - ev).abs() < 1e-5)
                || ((du - ev).abs() < 1e-5 && (dv - eu).abs() < 1e-5);
            assert!(
                spans_match,
                "{label}: uv span ({du}, {dv}) must equal quad extent in tiles ({eu}, {ev})"
            );
            assert!(
                (u_min - u_min.round()).abs() < 1e-6,
                "{label}: u anchored on a tile boundary, got min {u_min}"
            );
            // Side faces stand upright (matches the RT mapping):
            // v decreases upward, one unit per tile, and both
            // horizontal edges land on integer v (v = 0 mod 1 is
            // the texture's top row at each tile's top).
            let gn = [n[0], -n[2], n[1]];
            if gn[2].abs() < 0.5 {
                let y_max = verts
                    .iter()
                    .map(|&v| mesh.positions[v][1])
                    .fold(f32::MIN, f32::max);
                let y_min = verts
                    .iter()
                    .map(|&v| mesh.positions[v][1])
                    .fold(f32::MAX, f32::min);
                let mut v_top = f32::MAX;
                let mut v_bot = f32::MIN;
                for &v in &verts {
                    let y = mesh.positions[v][1];
                    let vv = mesh.uvs[v][1];
                    assert!(
                        (vv - vv.round()).abs() < 1e-6,
                        "{label}: side-face horizontal edges land on integer v, got {vv}"
                    );
                    if (y - y_max).abs() < 1e-6 {
                        v_top = v_top.min(vv);
                    }
                    if (y - y_min).abs() < 1e-6 {
                        v_bot = v_bot.max(vv);
                    }
                }
                assert!(
                    v_top < v_bot,
                    "{label}: side-face v must decrease upward (top {v_top}, bottom {v_bot})"
                );
            }
        }
    }
}

#[test]
fn gate_2f_1_single_material_chunk_uvs_and_layers() {
    let mut grid = VoxelGrid::new([3, 3, 3]);
    grid.fill(textured_block());
    let naive = mesh_naive(&grid, [0, 0, 0], grid.size, scale());
    let greedy = mesh_greedy(&grid, [0, 0, 0], grid.size, scale());
    assert!(naive.total_quads() > 0 && greedy.total_quads() > 0);
    check_mesh_uvs(&naive, "naive");
    check_mesh_uvs(&greedy, "greedy");
    // Greedy merges the whole 3x3x3 cube into 6 quads, each
    // spanning 3x3 tile-unit UVs (checked above).
    assert_eq!(greedy.total_quads(), 6, "one quad per cube face");
    assert_eq!(naive.total_quads(), 6 * 9, "one quad per block face");
}

#[test]
fn gate_2f_1_greedy_never_merges_different_layers() {
    let a = textured_block();
    let mut b = textured_block();
    b.tex_layers = Some([20, 21, 22]);

    // Two blocks side by side along x, differing only in layers.
    let mut grid = VoxelGrid::new([2, 1, 1]);
    grid.set(0, 0, 0, a);
    grid.set(1, 0, 0, b);
    let naive = mesh_naive(&grid, [0, 0, 0], grid.size, scale());
    let greedy = mesh_greedy(&grid, [0, 0, 0], grid.size, scale());
    assert_eq!(naive.total_quads(), 10, "5 visible faces per block");
    assert_eq!(
        greedy.total_quads(),
        naive.total_quads(),
        "different layers must not merge: greedy degenerates to naive"
    );
    // Specifically two top quads, carrying layers 10 and 20.
    let top_layers: Vec<u32> = greedy.meshes[Pass::Opaque.index()]
        .indices
        .chunks_exact(6)
        .filter(|q| {
            let n = greedy.meshes[Pass::Opaque.index()].normals[q[0] as usize];
            n[1] > 0.5 // world +Y = grid +Z = top
        })
        .map(|q| greedy.meshes[Pass::Opaque.index()].layers[q[0] as usize])
        .collect();
    assert_eq!(top_layers.len(), 2);
    assert!(top_layers.contains(&10) && top_layers.contains(&20));

    // Control: identical layers DO merge (the test above is
    // meaningful) — 2x1x1 of one material is 6 greedy quads.
    let mut grid2 = VoxelGrid::new([2, 1, 1]);
    grid2.set(0, 0, 0, a);
    grid2.set(1, 0, 0, a);
    let greedy2 = mesh_greedy(&grid2, [0, 0, 0], grid2.size, scale());
    assert_eq!(greedy2.total_quads(), 6, "same-layer neighbors merge");
}

#[test]
fn gate_2f_1_naive_greedy_parity_with_textures_stamped() {
    // Deterministic mixed scene: textured and flat blocks of
    // several passes, some adjacent (mergeable), some not.
    let mut grid = VoxelGrid::new([6, 5, 4]);
    let palette = [
        textured_block(),
        Block {
            tex_layers: Some([1, 2, 3]),
            ..textured_block()
        },
        Block {
            tex_layers: None,
            ..textured_block()
        },
        Block {
            id: 7,
            pass: Pass::Cutout,
            tex_layers: Some([5, 5, 5]),
            ..textured_block()
        },
    ];
    for z in 0..4 {
        for y in 0..5 {
            for x in 0..6 {
                let h = (x * 7 + y * 13 + z * 29) % 5;
                if h < 4 {
                    grid.set(x, y, z, palette[h]);
                }
            }
        }
    }
    let naive = mesh_naive(&grid, [0, 0, 0], grid.size, scale());
    let greedy = mesh_greedy(&grid, [0, 0, 0], grid.size, scale());
    for pass in Pass::ALL {
        let (nset, nexp) = covered_face_cells(naive.get(pass));
        let (gset, gexp) = covered_face_cells(greedy.get(pass));
        assert_eq!(nset, gset, "{pass:?}: same covered face cells");
        assert_eq!(nexp, nset.len(), "{pass:?}: naive has no overlaps");
        assert_eq!(gexp, gset.len(), "{pass:?}: greedy has no overlaps");
    }
    // Every emitted layer is either the flat sentinel or one of the
    // stamped materials' layers — never garbage.
    let allowed = [FLAT_LAYER, 10, 11, 12, 1, 2, 3, 5];
    for meshes in [&naive, &greedy] {
        for mesh in meshes.meshes.iter() {
            for &l in &mesh.layers {
                assert!(allowed.contains(&l), "unexpected layer {l}");
            }
        }
    }
    // And the flat blocks really do take the sentinel path.
    assert!(naive
        .meshes
        .iter()
        .flat_map(|m| m.layers.iter())
        .any(|&l| l == FLAT_LAYER));
}
