//! The naive (reference) and greedy meshers.
//!
//! Both take a [`VoxelSampler`] plus an origin/size window, so the exact
//! same code meshes a lone grid, a chunk inside a chunk map (neighbor
//! lookups cross chunk boundaries through the sampler), or any sub-region.

use crate::block::{Block, Pass};
use crate::grid::VoxelSampler;
use crate::mesh::{Mesh, PassMeshes};
use crate::scale::WorldScale;

/// Face definitions in grid space: `(direction, corners)` with corners
/// counter-clockwise seen from outside the cube `[0,1]^3`.
/// Direction order: 0:+X 1:-X 2:+Y(south) 3:-Y(north) 4:+Z(up) 5:-Z(down).
pub const FACE_DIRS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

pub(crate) const FACE_CORNERS: [[[f32; 3]; 4]; 6] = [
    // +X
    [[1., 0., 0.], [1., 1., 0.], [1., 1., 1.], [1., 0., 1.]],
    // -X
    [[0., 1., 0.], [0., 0., 0.], [0., 0., 1.], [0., 1., 1.]],
    // +Y (south)
    [[1., 1., 0.], [0., 1., 0.], [0., 1., 1.], [1., 1., 1.]],
    // -Y (north)
    [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
    // +Z (up)
    [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
    // -Z (down)
    [[0., 1., 0.], [1., 1., 0.], [1., 0., 0.], [0., 0., 0.]],
];

/// Is the face of `src` against `neighbor` visible?
///
/// Rules: a face against air is always visible. An opaque face is hidden
/// only by another opaque block. A cutout face is hidden by an opaque
/// block or by an identical cutout block. A transparent face is hidden by
/// an opaque block or by an identical transparent block (so two adjacent
/// glass blocks share no internal face, but glass against water shows).
pub fn face_visible(src: Block, neighbor: Block) -> bool {
    if neighbor.is_air() {
        return true;
    }
    match src.pass {
        Pass::Opaque => neighbor.pass != Pass::Opaque,
        Pass::Cutout => neighbor.pass != Pass::Opaque && neighbor.id != src.id,
        Pass::Transparent => neighbor.pass != Pass::Opaque && neighbor.id != src.id,
    }
}

/// Texture-array layer for face direction `di` of `block` (Phase 2f):
/// +Z samples the top layer, -Z the bottom layer, every side face the
/// side layer — the same selection the RT shader's `tex_albedo` makes.
/// `None` when the block is untextured (flat shading).
pub(crate) fn layer_for_dir(block: Block, di: usize) -> Option<u32> {
    let layers = block.tex_layers?;
    Some(match di {
        4 => layers[0], // +Z (up): top
        5 => layers[2], // -Z (down): bottom
        _ => layers[1], // sides
    })
}

/// UV for a grid-space point `p` on face `di`, given the quad's
/// per-axis grid minimum (Phase 2f; Stage A of the procedural
/// texture pipeline rework). Mirrors the RT shader's `tex_albedo`
/// mapping: top and bottom faces use `(x, y)` in tile units; side
/// faces use `(horizontal, 1 - z)` in tile units so v = 0 is the
/// texture's top row at the top of each voxel.
///
/// Stage A fix: UVs are in TILE UNITS, not fractions of the merged
/// quad. The pre-fix version divided by the quad extent, so a
/// greedy-merged quad spanned [0,1]² across its whole extent and
/// stretched one sprite over the merged area (the smearing in the
/// first textured screenshots). In tile units a merged quad's UVs
/// span [0, w] x [0, h], and the engine's `Repeat` sampler wraps
/// each integer interval to one full texture per tile. Quad minima
/// sit on tile boundaries, so the repeat phase is anchored
/// correctly; `quad_ext` is no longer needed and kept only so the
/// two call sites read the same.
fn face_uv(di: usize, p: [f32; 3], quad_min: [f32; 3], _quad_ext: [f32; 3]) -> [f32; 2] {
    let dir = FACE_DIRS[di];
    let axis = (0..3).find(|&a| dir[a] != 0).unwrap();
    if axis == 2 {
        [p[0] - quad_min[0], p[1] - quad_min[1]]
    } else {
        // The horizontal axis is the one that is neither the face
        // axis nor z.
        let h = (0..3).find(|&a| a != axis && a != 2).unwrap();
        [p[h] - quad_min[h], 1.0 - (p[2] - quad_min[2])]
    }
}

/// Per-corner UVs for one textured quad whose face layer is
/// `layer` (Stage F): a +Z (top) face carrying a macro-encoded
/// layer samples the world-space macro texture, so its UVs are the
/// corners' *absolute* grid coordinates divided by the macro span —
/// two world points exactly `span` tiles apart share a macro phase
/// (the macro array's Repeat sampler wraps the rest). Every other
/// face keeps the Stage A tile-unit mapping ([`face_uv`]).
pub(crate) fn quad_uvs(
    di: usize,
    grid_corners: &[[f32; 3]; 4],
    quad_min: [f32; 3],
    quad_ext: [f32; 3],
    layer: u32,
) -> [[f32; 2]; 4] {
    if di == 4 {
        if let Some((_, span)) = crate::mesh::decode_macro_layer(layer) {
            let span = span.max(1) as f32;
            return grid_corners.map(|g| [g[0] / span, g[1] / span]);
        }
    }
    grid_corners.map(|g| face_uv(di, g, quad_min, quad_ext))
}

/// Grid-space point -> Y-up world point. This is a rotation about X
/// (grid `(x, y_south, z_up)` -> world `(x, z_up, -y_south)`), NOT a plain
/// axis swap: a swap is a reflection and would flip triangle winding.
pub(crate) fn to_world(p: [f32; 3], scale: WorldScale) -> [f32; 3] {
    [
        p[0] * scale.block_size,
        p[2] * scale.block_size,
        -p[1] * scale.block_size,
    ]
}

/// Grid-space direction -> Y-up world direction (same rotation).
pub(crate) fn normal_to_world(d: [i32; 3]) -> [f32; 3] {
    [d[0] as f32, d[2] as f32, -d[1] as f32]
}

/// Naive mesher: one quad per visible face. Reference implementation.
pub fn mesh_naive(
    sampler: &dyn VoxelSampler,
    origin: [i32; 3],
    size: [usize; 3],
    scale: WorldScale,
) -> PassMeshes {
    let mut out = PassMeshes::default();
    for z in 0..size[2] as i32 {
        for y in 0..size[1] as i32 {
            for x in 0..size[0] as i32 {
                let (gx, gy, gz) = (origin[0] + x, origin[1] + y, origin[2] + z);
                let block = sampler.block_at(gx, gy, gz);
                if block.is_air() {
                    continue;
                }
                let mesh = &mut out.meshes[block.pass.index()];
                for (di, dir) in FACE_DIRS.iter().enumerate() {
                    let neighbor = sampler.block_at(gx + dir[0], gy + dir[1], gz + dir[2]);
                    if !face_visible(block, neighbor) {
                        continue;
                    }
                    let grid_corners = FACE_CORNERS[di]
                        .map(|c| [gx as f32 + c[0], gy as f32 + c[1], gz as f32 + c[2]]);
                    let corners = grid_corners.map(|g| to_world(g, scale));
                    let normal = normal_to_world(*dir);
                    match layer_for_dir(block, di) {
                        Some(layer) => {
                            let quad_min = [gx as f32, gy as f32, gz as f32];
                            let uvs = quad_uvs(di, &grid_corners, quad_min, [1.0, 1.0, 1.0], layer);
                            mesh.push_quad_textured(corners, normal, block.color, uvs, layer);
                        }
                        None => mesh.push_quad(corners, normal, block.color),
                    }
                }
            }
        }
    }
    out
}

/// Greedy mesher: merges coplanar adjacent faces of identical blocks into
/// maximal rectangles, per pass, per direction, per slice.
pub fn mesh_greedy(
    sampler: &dyn VoxelSampler,
    origin: [i32; 3],
    size: [usize; 3],
    scale: WorldScale,
) -> PassMeshes {
    let mut out = PassMeshes::default();
    // Axis indices in grid space: 0=x, 1=y, 2=z.
    for pass in Pass::ALL {
        let mesh = &mut out.meshes[pass.index()];
        for (di, dir) in FACE_DIRS.iter().enumerate() {
            let axis = (0..3).find(|&a| dir[a] != 0).unwrap();
            let others: Vec<usize> = (0..3).filter(|&a| a != axis).collect();
            let (u_ax, v_ax) = (others[0], others[1]);
            let (u_len, v_len) = (size[u_ax], size[v_ax]);
            for slice in 0..size[axis] {
                // Mask of merge keys over (u, v) for this slice.
                let mut mask: Vec<Option<Block>> = vec![None; u_len * v_len];
                for v in 0..v_len {
                    for u in 0..u_len {
                        let mut cell = [0i32; 3];
                        cell[axis] = slice as i32;
                        cell[u_ax] = u as i32;
                        cell[v_ax] = v as i32;
                        let (gx, gy, gz) = (
                            origin[0] + cell[0],
                            origin[1] + cell[1],
                            origin[2] + cell[2],
                        );
                        let block = sampler.block_at(gx, gy, gz);
                        if block.is_air() || block.pass != pass {
                            continue;
                        }
                        let neighbor = sampler.block_at(gx + dir[0], gy + dir[1], gz + dir[2]);
                        if face_visible(block, neighbor) {
                            mask[v * u_len + u] = Some(block);
                        }
                    }
                }
                // Greedy rectangle extraction.
                let mut used = vec![false; u_len * v_len];
                for v in 0..v_len {
                    for u in 0..u_len {
                        let key = match mask[v * u_len + u] {
                            Some(b) if !used[v * u_len + u] => b,
                            _ => continue,
                        };
                        // Expand along u.
                        let mut w = 1;
                        while u + w < u_len
                            && !used[v * u_len + u + w]
                            && mask[v * u_len + u + w] == Some(key)
                        {
                            w += 1;
                        }
                        // Expand along v while the whole row strip matches.
                        let mut h = 1;
                        'grow: while v + h < v_len {
                            for uu in u..u + w {
                                if used[(v + h) * u_len + uu]
                                    || mask[(v + h) * u_len + uu] != Some(key)
                                {
                                    break 'grow;
                                }
                            }
                            h += 1;
                        }
                        for vv in v..v + h {
                            for uu in u..u + w {
                                used[vv * u_len + uu] = true;
                            }
                        }
                        emit_greedy_quad(
                            mesh,
                            origin,
                            axis,
                            u_ax,
                            v_ax,
                            slice as i32,
                            u as i32,
                            v as i32,
                            w as i32,
                            h as i32,
                            *dir,
                            di,
                            key,
                            scale,
                        );
                    }
                }
            }
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn emit_greedy_quad(
    mesh: &mut Mesh,
    origin: [i32; 3],
    axis: usize,
    u_ax: usize,
    v_ax: usize,
    slice: i32,
    u0: i32,
    v0: i32,
    w: i32,
    h: i32,
    dir: [i32; 3],
    di: usize,
    block: Block,
    scale: WorldScale,
) {
    // Plane coordinate along the axis: top of the cell for positive faces.
    let plane = if dir[axis] > 0 { slice + 1 } else { slice };
    let grid_corner = |u: i32, v: i32| -> [f32; 3] {
        let mut g = [0f32; 3];
        g[axis] = (origin[axis] + plane) as f32;
        g[u_ax] = (origin[u_ax] + u) as f32;
        g[v_ax] = (origin[v_ax] + v) as f32;
        g
    };
    let mut grid_corners = [
        grid_corner(u0, v0),
        grid_corner(u0 + w, v0),
        grid_corner(u0 + w, v0 + h),
        grid_corner(u0, v0 + h),
    ];
    let mut corners = grid_corners.map(|g| to_world(g, scale));
    // Guarantee outward winding: flip the order if the geometric normal
    // disagrees with the face normal.
    let n = normal_to_world(dir);
    let ab = [
        corners[1][0] - corners[0][0],
        corners[1][1] - corners[0][1],
        corners[1][2] - corners[0][2],
    ];
    let ac = [
        corners[2][0] - corners[0][0],
        corners[2][1] - corners[0][1],
        corners[2][2] - corners[0][2],
    ];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    if cross[0] * n[0] + cross[1] * n[1] + cross[2] * n[2] < 0.0 {
        corners.swap(1, 3);
        grid_corners.swap(1, 3);
    }
    match layer_for_dir(block, di) {
        Some(layer) => {
            // UV fractions are relative to the merged quad's minimum
            // and extent along its two in-plane axes.
            let mut quad_min = [0f32; 3];
            let mut quad_ext = [1f32; 3];
            quad_min[u_ax] = (origin[u_ax] + u0) as f32;
            quad_ext[u_ax] = w as f32;
            quad_min[v_ax] = (origin[v_ax] + v0) as f32;
            quad_ext[v_ax] = h as f32;
            let uvs = quad_uvs(di, &grid_corners, quad_min, quad_ext, layer);
            mesh.push_quad_textured(corners, n, block.color, uvs, layer);
        }
        None => mesh.push_quad(corners, n, block.color),
    }
}
