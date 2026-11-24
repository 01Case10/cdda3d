//! Fine slope normals gate (docs/fine-slope-normals.md, S1):
//! with the opt-in on, a stamped heightfield ramp's top faces
//! carry per-corner normals tilted by the local height
//! gradient, so the rasteriser interpolates the staircase as
//! one inclined plane. The gate pins:
//!
//! - a 1-unit-per-cell ramp's interior top corners all carry
//!   normalize(-1, 1, 0) in world space (gradient 1 along +x);
//! - a flat plateau keeps exact axis normals everywhere;
//! - a 16-unit cliff keeps exact axis normals everywhere
//!   (hard-edge threshold: deliberate cliffs stay crisp);
//! - with the opt-in OFF, the same ramp meshes exactly as
//!   before the fix (every normal axis-aligned).

use mesher::block::Block;
use mesher::chunk::ChunkMap;
use mesher::fine::{mesh_chunk_fine, FineLayer, FineVolume, FINE};
use mesher::mesh::Mesh;
use mesher::scale::WorldScale;
use mesher::Pass;

fn blk(id: u32) -> Block {
    Block {
        id,
        color: [0.8, 0.5, 0.25, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// A scene whose tile (4,4) fine volume is filled per column
/// by `height(lx, ly)` (cells fz in 0..height).
fn scene(height: impl Fn(i32, i32) -> i32) -> (ChunkMap, FineLayer) {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    map.clear_dirty();
    let mut vol = FineVolume::new((4, 4), 0, 96);
    for ly in 0..FINE {
        for lx in 0..FINE {
            for fz in 0..height(lx, ly) {
                vol.set(lx, ly, fz, Some(blk(77)));
            }
        }
    }
    let mut layer = FineLayer::default();
    layer.insert(vol);
    (map, layer)
}

/// All top-face quads of the opaque mesh: (centre, normals).
/// A top face is a quad whose flat normal would point up —
/// identified geometrically: its four vertices share one
/// world y and its winding normal has positive y. Tilted
/// top faces still satisfy both.
fn top_quads(mesh: &Mesh) -> Vec<([f32; 3], [[f32; 3]; 4])> {
    let mut out = Vec::new();
    for quad in mesh.indices.chunks_exact(6) {
        let idx: Vec<usize> = quad.iter().map(|i| *i as usize).collect();
        let p: Vec<[f32; 3]> = idx[..4].iter().map(|&i| mesh.positions[i]).collect();
        if p.iter().any(|v| (v[1] - p[0][1]).abs() > 1e-4) {
            continue; // not horizontal
        }
        // Geometric normal (winding) must point up.
        let ab = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let ac = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let ny = ab[2] * ac[0] - ab[0] * ac[2];
        if ny <= 0.0 {
            continue;
        }
        let centre = [
            (p[0][0] + p[2][0]) / 2.0,
            p[0][1],
            (p[0][2] + p[2][2]) / 2.0,
        ];
        let normals = [
            mesh.normals[idx[0]],
            mesh.normals[idx[1]],
            mesh.normals[idx[2]],
            mesh.normals[idx[3]],
        ];
        out.push((centre, normals));
    }
    out
}

fn mesh(scene: &(ChunkMap, FineLayer), slope_normals: bool) -> Mesh {
    let (map, layer) = scene;
    let meshes = mesh_chunk_fine(map, layer, [0, 0], WorldScale::default(), slope_normals)
        .expect("chunk meshes");
    meshes.meshes[Pass::Opaque.index()].clone()
}

#[test]
fn ramp_top_corners_tilt_with_the_gradient() {
    // Height rises 1 fine cell per cell along +x (lx).
    let scene = scene(|lx, _ly| 8 + lx);
    let mesh = mesh(&scene, true);
    let quads = top_quads(&mesh);
    // Interior: centres at least 5 cells from the volume's
    // x/y borders (border corners sample outside the volume
    // and fall back by design).
    let bs = WorldScale::default().block_size;
    let lo = (4 * FINE + 5) as f32 / FINE as f32 * bs;
    let hi = (4 * FINE + 27) as f32 / FINE as f32 * bs;
    let want = [-std::f32::consts::FRAC_1_SQRT_2, std::f32::consts::FRAC_1_SQRT_2, 0.0];
    let mut checked = 0usize;
    for (centre, normals) in &quads {
        // World z runs south = grid +y: the centre's grid
        // y is -centre.z / bs in block units.
        let gy = -centre[2] / bs;
        if centre[0] < lo || centre[0] > hi || gy < lo || gy > hi {
            continue;
        }
        checked += 1;
        for n in normals {
            for k in 0..3 {
                assert!(
                    (n[k] - want[k]).abs() < 0.02,
                    "interior ramp corner normal {n:?}, want {want:?}"
                );
            }
        }
    }
    assert!(checked > 100, "only {checked} interior top quads checked");
}

#[test]
fn plateau_keeps_axis_normals() {
    let scene = scene(|_lx, _ly| 16);
    let mesh = mesh(&scene, true);
    let quads = top_quads(&mesh);
    assert!(quads.len() > 100, "plateau top quads: {}", quads.len());
    for (_centre, normals) in &quads {
        for n in normals {
            assert_eq!(*n, [0.0, 1.0, 0.0], "plateau corner must stay axis");
        }
    }
}

#[test]
fn cliff_keeps_axis_normals() {
    // 16-unit cliff at lx = 16: over the hard-edge threshold.
    let scene = scene(|lx, _ly| if lx < 16 { 8 } else { 24 });
    let mesh = mesh(&scene, true);
    let quads = top_quads(&mesh);
    assert!(quads.len() > 100, "cliff top quads: {}", quads.len());
    for (_centre, normals) in &quads {
        for n in normals {
            assert_eq!(*n, [0.0, 1.0, 0.0], "cliff corner must stay axis");
        }
    }
}

#[test]
fn flag_off_is_the_pre_fix_mesh() {
    let scene = scene(|lx, _ly| 8 + lx);
    let mesh = mesh(&scene, false);
    for n in &mesh.normals {
        let axis = (n[0].abs() == 1.0 && n[1] == 0.0 && n[2] == 0.0)
            || (n[1].abs() == 1.0 && n[0] == 0.0 && n[2] == 0.0)
            || (n[2].abs() == 1.0 && n[0] == 0.0 && n[1] == 0.0);
        assert!(axis, "flag-off normal {n:?} is not axis-aligned");
    }
}

/// The ramp's downhill riser faces (the +X side faces of
/// ramp cells, exposed because the next column is one cell
/// lower in surface terms — here the ramp rises along +x,
/// so the risers face -X... the exposed step faces are the
/// +X faces of each column's top cell looking down the
/// staircase's upper side and the -X faces looking up it;
/// both must carry the plane normal for the staircase to
/// shade as one surface). Concretely for a ramp rising
/// along +x: the top cell of column lx has its +X face
/// culled by column lx+1's cell below the top... the
/// VISIBLE step faces are the -X faces of each top cell
/// (against the air above column lx-1's lower top) — no:
/// face_visible checks the neighbour CELL at the same fz.
/// Column lx-1 has no cell at fz = top(lx), so the -X face
/// of column lx's top cell is exposed: it is the riser.
/// All four of its corners must carry the plane normal.
#[test]
fn ramp_riser_faces_carry_the_plane_normal() {
    let scene = scene(|lx, _ly| 8 + lx);
    let mesh = mesh(&scene, true);
    let want = [-std::f32::consts::FRAC_1_SQRT_2, std::f32::consts::FRAC_1_SQRT_2, 0.0];
    // Riser quads: vertical quads (verts span two world y
    // levels) whose flat normal would be -X in world terms:
    // world x = grid x, so a grid -X face has world normal
    // (-1, 0, 0) when flat; smoothed, its corners tilt to
    // `want`. Identify candidates by position: interior
    // columns only, and by the quad's world-x extent being
    // a single plane (all four verts share one world x).
    let bs = WorldScale::default().block_size;
    let lo = (4 * FINE + 5) as f32 / FINE as f32 * bs;
    let hi = (4 * FINE + 27) as f32 / FINE as f32 * bs;
    let mut checked = 0usize;
    for quad in mesh.indices.chunks_exact(6) {
        let idx: Vec<usize> = quad.iter().map(|i| *i as usize).collect();
        let p: Vec<[f32; 3]> = idx[..4].iter().map(|&i| mesh.positions[i]).collect();
        // Vertical plane at constant world x?
        if p.iter().any(|v| (v[0] - p[0][0]).abs() > 1e-4) {
            continue;
        }
        // Spans exactly one fine cell of height?
        let (mut ymin, mut ymax) = (f32::MAX, f32::MIN);
        for v in &p {
            ymin = ymin.min(v[1]);
            ymax = ymax.max(v[1]);
        }
        if (ymax - ymin - bs / FINE as f32).abs() > 1e-3 {
            continue;
        }
        let cx = p[0][0];
        let gy = -(p[0][2] + p[2][2]) / 2.0 / bs;
        if cx < lo || cx > hi || gy < lo || gy > hi {
            continue;
        }
        // Only the downhill-facing risers: their flat
        // geometric normal (winding) points -x in world.
        let ab = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let ac = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let nx = ab[1] * ac[2] - ab[2] * ac[1];
        if nx >= 0.0 {
            continue;
        }
        checked += 1;
        for (k, &i) in idx[..4].iter().enumerate() {
            let n = mesh.normals[i];
            for a in 0..3 {
                assert!(
                    (n[a] - want[a]).abs() < 0.02,
                    "riser corner {k} normal {n:?}, want {want:?}"
                );
            }
        }
    }
    assert!(checked > 100, "only {checked} interior riser quads checked");
}

#[test]
fn shallow_ramp_corners_average_out_the_quantisation() {
    // Slope 3/4: column heights step 0,1,1,1 repeating. The
    // per-vertex 1-cell difference oscillates between 0 and
    // 1 on such a slope, flipping the shading normal between
    // flat and 45 degrees — the banding visible on the
    // vehicle profile's windshield and hood (their true
    // gradients are 0.84 and 0.28). The fitted gradient must
    // land on the true 0.75 at every interior corner:
    // world normal normalize(-0.75, 1, 0) = (-0.6, 0.8, 0).
    let scene = scene(|lx, _ly| 8 + (lx * 3) / 4);
    let mesh = mesh(&scene, true);
    let quads = top_quads(&mesh);
    let bs = WorldScale::default().block_size;
    let lo = (4 * FINE + 5) as f32 / FINE as f32 * bs;
    let hi = (4 * FINE + 27) as f32 / FINE as f32 * bs;
    let want = [-0.6, 0.8, 0.0];
    let mut checked = 0usize;
    for (centre, normals) in &quads {
        let gy = -centre[2] / bs;
        if centre[0] < lo || centre[0] > hi || gy < lo || gy > hi {
            continue;
        }
        checked += 1;
        for n in normals {
            for k in 0..3 {
                assert!(
                    (n[k] - want[k]).abs() < 0.05,
                    "shallow ramp corner normal {n:?}, want {want:?}"
                );
            }
        }
    }
    assert!(checked > 100, "only {checked} interior top quads checked");
}
