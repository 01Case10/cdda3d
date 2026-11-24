//! Per-corner occlusion gate (docs/fine-slope-normals.md
//! §13): under the fine-path opt-in, a stamped face's Stage H
//! occlusion factor is computed per face CORNER — the same
//! counting rule as `face_occlusion`, but run from the air
//! cell diagonally in front of each corner — and emitted as
//! per-vertex colours, so the rasteriser interpolates the
//! factor across the face instead of stamping one flat notch
//! on the whole face. The gate pins:
//!
//! - the tread beside a 1-cell step: corners at the step
//!   edge read 0.67 (3 occluders around their front cell),
//!   corners at the far edge read 1.0 exactly — a strict
//!   gradient across one face, where the flat per-face form
//!   reads 0.89 at all four corners;
//! - a flat plateau keeps factor 1.0 at every top corner;
//! - with the opt-in OFF the mesh keeps the legacy flat
//!   per-face factor exactly (0.89 at all four tread
//!   corners) and the geometry is unchanged;
//! - `cell_occlusion` (the RT path's per-cell form) still
//!   returns the flat per-face mean — RT is untouched.

use mesher::block::Block;
use mesher::chunk::ChunkMap;
use mesher::fine::{cell_occlusion, mesh_chunk_fine, FineLayer, FineSampler, FineVolume, FINE};
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

const BASE: [f32; 4] = [0.8, 0.5, 0.25, 1.0];

/// A scene whose tile (4,4) fine volume is filled per column
/// by `height(lx, ly)` (cells fz in 0..height) — the
/// fine_slope_normals_gate scene idiom.
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

fn mesh(scene: &(ChunkMap, FineLayer), slope_normals: bool) -> Mesh {
    let (map, layer) = scene;
    let meshes = mesh_chunk_fine(map, layer, [0, 0], WorldScale::default(), slope_normals)
        .expect("chunk meshes");
    meshes.meshes[Pass::Opaque.index()].clone()
}

/// Top-face quads of the opaque mesh with their corner
/// positions and colours: (centre, [(pos, colour); 4]).
/// Top faces are identified geometrically, as in the slope
/// gate (shared world y, winding normal up).
fn top_quads(mesh: &Mesh) -> Vec<([f32; 3], [([f32; 3], [f32; 4]); 4])> {
    let mut out = Vec::new();
    for quad in mesh.indices.chunks_exact(6) {
        let idx: Vec<usize> = quad.iter().map(|i| *i as usize).collect();
        let p: Vec<[f32; 3]> = idx[..4].iter().map(|&i| mesh.positions[i]).collect();
        if p.iter().any(|v| (v[1] - p[0][1]).abs() > 1e-4) {
            continue;
        }
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
        let corners = [
            (p[0], mesh.colors[idx[0]]),
            (p[1], mesh.colors[idx[1]]),
            (p[2], mesh.colors[idx[2]]),
            (p[3], mesh.colors[idx[3]]),
        ];
        out.push((centre, corners));
    }
    out
}

/// The occlusion factor a corner colour carries (the base
/// colour's red channel scaled by the factor).
fn factor(colour: [f32; 4]) -> f32 {
    colour[0] / BASE[0]
}

/// The tread top faces beside the step: top quads whose
/// centre sits over fine column `lx` of tile (4,4), in the
/// interior ly band (centres at least 5 cells from the
/// volume's y borders, as in the slope gate). Returns
/// (near factors, far factors) split by which side of the
/// quad centre each corner lies on along world x.
fn tread_factors(mesh: &Mesh, lx: i32) -> (Vec<f32>, Vec<f32>) {
    let bs = WorldScale::default().block_size;
    let want_cx = (4 * FINE + lx) as f32 / FINE as f32 * bs + 0.5 * bs / FINE as f32;
    let lo = (4 * FINE + 5) as f32 / FINE as f32 * bs;
    let hi = (4 * FINE + 27) as f32 / FINE as f32 * bs;
    let (mut near, mut far) = (Vec::new(), Vec::new());
    for (centre, corners) in top_quads(mesh) {
        if (centre[0] - want_cx).abs() > 1e-3 {
            continue;
        }
        let gy = -centre[2] / bs;
        if gy < lo || gy > hi {
            continue;
        }
        for (pos, colour) in corners {
            if pos[0] > centre[0] {
                near.push(factor(colour));
            } else {
                far.push(factor(colour));
            }
        }
    }
    (near, far)
}

fn step_scene() -> (ChunkMap, FineLayer) {
    // One-cell step at lx = 16, rising along +x.
    scene(|lx, _ly| if lx < 16 { 8 } else { 9 })
}

#[test]
fn step_tread_shows_an_occlusion_gradient() {
    let mesh = mesh(&step_scene(), true);
    // The tread beside the riser is the top of column 15.
    let (near, far) = tread_factors(&mesh, 15);
    assert!(
        near.len() >= 40 && far.len() >= 40,
        "tread corners sampled: near {}, far {}",
        near.len(),
        far.len()
    );
    for f in &far {
        assert!(
            (f - 1.0).abs() < 1e-4,
            "far corner factor {f}, want 1.0 exactly (open sky at its front cell)"
        );
    }
    for f in &near {
        assert!(
            (f - 0.67).abs() < 1e-4,
            "near corner factor {f}, want 0.67 (3 occluders around its front cell)"
        );
    }
    let near_max = near.iter().fold(f32::MIN, |a, &b| a.max(b));
    let far_min = far.iter().fold(f32::MAX, |a, &b| a.min(b));
    assert!(
        near_max < far_min,
        "gradient must be strict: darkest near {near_max} !< lightest far {far_min}"
    );
}

#[test]
fn plateau_corners_keep_full_light() {
    let mesh = mesh(&scene(|_lx, _ly| 16), true);
    let quads = top_quads(&mesh);
    assert!(quads.len() > 100, "plateau top quads: {}", quads.len());
    for (_centre, corners) in &quads {
        for (_pos, colour) in corners {
            assert_eq!(
                *colour, BASE,
                "plateau corner must keep the cell's own colour (factor 1.0)"
            );
        }
    }
}

#[test]
fn flag_off_keeps_the_flat_factor() {
    let scene = step_scene();
    let off = mesh(&scene, false);
    let (near, far) = tread_factors(&off, 15);
    assert!(near.len() >= 40 && far.len() >= 40);
    for f in near.iter().chain(far.iter()) {
        assert!(
            (f - 0.89).abs() < 1e-4,
            "flag-off tread corner factor {f}, want the legacy flat 0.89"
        );
    }
    // Geometry is the opt-in's too: same quad count either way.
    let on = mesh(&scene, true);
    assert_eq!(
        on.quad_count(),
        off.quad_count(),
        "per-corner occlusion must not change the geometry"
    );
    for n in &off.normals {
        let axis = (n[0].abs() == 1.0 && n[1] == 0.0 && n[2] == 0.0)
            || (n[1].abs() == 1.0 && n[0] == 0.0 && n[2] == 0.0)
            || (n[2].abs() == 1.0 && n[0] == 0.0 && n[1] == 0.0);
        assert!(axis, "flag-off normal {n:?} is not axis-aligned");
    }
}

#[test]
fn cell_occlusion_for_rt_is_unchanged() {
    let (map, layer) = step_scene();
    let sampler = FineSampler::new(&map, &layer);
    // The tread cell (column 15, top cell fz 7): only its
    // +Z face is exposed (the step and the plateau close
    // every other face), so the RT per-cell form is that
    // face's flat factor, 0.89 — before and after this
    // slice, by construction.
    let c = cell_occlusion(&sampler, 4 * FINE + 15, 4 * FINE + 16, 7);
    assert!(
        (c - 0.89).abs() < 1e-6,
        "cell occlusion {c}, want the flat per-face 0.89"
    );
}
