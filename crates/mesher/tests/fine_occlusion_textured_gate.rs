//! Textured-path occlusion gate (docs/fine-slope-normals.md
//! §14, re-scoped by §15): under the fine-path opt-in, a
//! stamped TEXTURED face's per-corner Stage H occlusion
//! factors are recorded in `Mesh::tex_occlusion`, parallel
//! to the vertices, because a textured vertex's RGB never
//! reaches the raster shader's diffuse term. §15's rule,
//! from the vehicle captures: a corner whose shading
//! normal is TILTED (slope-treated — a fitted top face or
//! a slope riser) records 1.0, because the fitted normal
//! already shades the step and the step occluding itself
//! on top of that draws a dark line at every step; an
//! axis-shaded corner keeps its factor, because there the
//! darkening is OTHER geometry's shadow (a tread beside a
//! cliff, a wall contact). The gate pins:
//!
//! - the tread beside a 1-cell step (its fit tilts):
//!   every recorded factor is 1.0 — where §14 recorded
//!   0.67 at the step-side corners;
//! - the tread beside a 32-cell cliff (samples beyond the
//!   fit's cliff threshold are excluded, so its fit stays
//!   flat): step-side corners record 0.56 (4 occluders —
//!   the cliff fills the front cell's side and above),
//!   far corners 1.0;
//! - the array is exactly parallel to the vertex arrays
//!   whenever anything is recorded;
//! - with the opt-in OFF nothing is recorded (empty
//!   array): the legacy textured path stays byte-identical.

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
        tex_layers: Some([7, 7, 7]),
    }
}

/// The fine_occlusion_corner_gate scene idiom: tile (4,4)'s
/// volume filled per column by `height(lx, ly)`.
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

/// (centre, [(pos, recorded factor); 4]) for top-face quads,
/// identified geometrically as in the sibling gates.
fn top_quads(mesh: &Mesh) -> Vec<([f32; 3], [([f32; 3], f32); 4])> {
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
            (p[0], mesh.tex_occlusion[idx[0]]),
            (p[1], mesh.tex_occlusion[idx[1]]),
            (p[2], mesh.tex_occlusion[idx[2]]),
            (p[3], mesh.tex_occlusion[idx[3]]),
        ];
        out.push((centre, corners));
    }
    out
}

/// (near, far) recorded factors of the tread top faces over
/// fine column `lx` of tile (4,4), interior ly band — the
/// corner gate's split: corners toward the rise are near.
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
        for (pos, f) in corners {
            if pos[0] > centre[0] {
                near.push(f);
            } else {
                far.push(f);
            }
        }
    }
    (near, far)
}

#[test]
fn slope_tread_records_no_occlusion() {
    // One-cell step at lx = 16, rising along +x: the
    // tread's plane fit tilts, so §15 records nothing.
    let mesh = mesh(&scene(|lx, _ly| if lx < 16 { 8 } else { 9 }), true);
    assert_eq!(
        mesh.tex_occlusion.len(),
        mesh.positions.len(),
        "recorded factors must be exactly parallel to the vertices"
    );
    let (near, far) = tread_factors(&mesh, 15);
    assert!(
        near.len() >= 40 && far.len() >= 40,
        "tread corners sampled: near {}, far {}",
        near.len(),
        far.len()
    );
    for f in near.iter().chain(far.iter()) {
        assert!(
            (f - 1.0).abs() < 1e-4,
            "slope-treated tread corner factor {f}, want 1.0 (§15)"
        );
    }
}

#[test]
fn cliff_tread_keeps_its_contact_occlusion() {
    // A 32-cell cliff at lx = 16: beyond the fit's cliff
    // threshold, so the tread's fit stays flat and its
    // corners keep the cliff's contact shadow.
    let mesh = mesh(&scene(|lx, _ly| if lx < 16 { 8 } else { 40 }), true);
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
            "far corner factor {f}, want 1.0 exactly"
        );
    }
    for f in &near {
        assert!(
            (f - 0.56).abs() < 1e-4,
            "near corner factor {f}, want 0.56 (4 occluders: the cliff)"
        );
    }
}

#[test]
fn flag_off_records_nothing() {
    let mesh = mesh(&scene(|lx, _ly| if lx < 16 { 8 } else { 9 }), false);
    assert!(
        mesh.tex_occlusion.is_empty(),
        "opt-in off must keep the legacy textured path untouched"
    );
}
