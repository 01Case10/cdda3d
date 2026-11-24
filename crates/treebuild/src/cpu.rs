//! CPU traversal oracle over the flattened tree.
//!
//! The algorithm is the one the WGSL tracer mirrors line-for-line
//! (gate 2c.3 compares them ray-for-ray): hierarchical empty-space
//! skipping by *descent* — from the ray's current point, descend the
//! tree; the descent stops either at an occupied leaf or at the
//! largest empty child cell containing the point, and the ray then
//! advances to that cell's exit plane. No stack, no recursion, and
//! the only float ops are slab tests, so CPU/GPU parity is exact
//! except for final rounding.
//!
//! All rays here are in *tree space* (grid coords minus
//! `BuiltTree::origin`) with a normalized direction; `dist` is in
//! voxel units along the ray.

use glam::Vec3;

use crate::build::{node_child_ptr, node_mask, BuiltTree};
use crate::world::{VoxelWorld, FLAG_TRANSPARENT};

/// Step used to move the sample point just inside the current cell.
pub const EPS: f32 = 1e-4;
const MAX_STEPS: usize = 8192;

/// Which voxels count as hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceMode {
    /// The first voxel of any pass (picking, primary rays before
    /// glass blending).
    FirstHit,
    /// Transparent voxels are marched through (shadow rays, the
    /// opaque layer under glass).
    OpaqueOnly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// World (grid) coord of the hit voxel.
    pub voxel: [i32; 3],
    /// Flattened node index of the hit leaf.
    pub node_index: u32,
    /// Face normal in grid space (unit axis).
    pub normal: [i32; 3],
    /// Distance along the (normalized) ray.
    pub dist: f32,
    pub material_id: u32,
    pub lit: u32,
    pub flags: u32,
    pub alpha: f32,
}

enum Descent {
    Occupied { node: usize, min: [u32; 3] },
    Empty { min: [u32; 3], size: u32 },
}

/// Descend the tree at point `p` (tree space, inside the root span).
fn descend(built: &BuiltTree, p: Vec3) -> Descent {
    let mut node_idx = 0usize;
    let mut min = [0u32; 3];
    let mut size = built.size;
    for _level in 0..built.depth {
        let child_size = size / 4;
        let node = &built.flat[node_idx];
        let mask = node_mask(node);
        let mut part = [0u32; 3];
        for a in 0..3 {
            let rel = (p[a] - min[a] as f32) / child_size as f32;
            part[a] = (rel.floor() as i32).clamp(0, 3) as u32;
        }
        let cidx = (part[2] << 4) | (part[1] << 2) | part[0];
        let child_min = [
            min[0] + part[0] * child_size,
            min[1] + part[1] * child_size,
            min[2] + part[2] * child_size,
        ];
        if mask & (1u64 << cidx) == 0 {
            return Descent::Empty {
                min: child_min,
                size: child_size,
            };
        }
        let rank = (mask & ((1u64 << cidx) - 1)).count_ones() as usize;
        let child_idx = node_child_ptr(node) + rank;
        if built.is_leaf(child_idx) {
            return Descent::Occupied {
                node: child_idx,
                min: child_min,
            };
        }
        node_idx = child_idx;
        min = child_min;
        size = child_size;
    }
    // Unreachable for trees built by `build_tree` (leaves sit exactly
    // at full depth); treat as empty rather than panicking.
    Descent::Empty { min, size }
}

/// Slab test: (entry, exit) parameters of `ro`/`rd` against the AABB
/// `[min, min+size)`. Entry may be negative when the origin is inside.
fn slab(ro: Vec3, rd: Vec3, min: [u32; 3], size: u32) -> (f32, f32) {
    let mut t0 = f32::NEG_INFINITY;
    let mut t1 = f32::INFINITY;
    for a in 0..3 {
        let lo = min[a] as f32;
        let hi = lo + size as f32;
        if rd[a].abs() < 1e-12 {
            if ro[a] < lo || ro[a] >= hi {
                return (f32::INFINITY, f32::NEG_INFINITY);
            }
        } else {
            let inv = 1.0 / rd[a];
            let mut ta = (lo - ro[a]) * inv;
            let mut tb = (hi - ro[a]) * inv;
            if ta > tb {
                std::mem::swap(&mut ta, &mut tb);
            }
            t0 = t0.max(ta);
            t1 = t1.min(tb);
        }
    }
    (t0, t1)
}

fn slab_exit(ro: Vec3, rd: Vec3, min: [u32; 3], size: u32) -> f32 {
    let (_, t1) = slab(ro, rd, min, size);
    t1
}

/// Face normal for a ray entering the AABB at parameter `t_enter`
/// (which came from `slab`). Ties break toward the lowest axis index,
/// matching the WGSL.
fn entry_normal(ro: Vec3, rd: Vec3, min: [u32; 3], size: u32, t_enter: f32) -> [i32; 3] {
    if t_enter < 0.0 {
        // Origin inside the voxel: face against the dominant axis.
        let mut axis = 0;
        for a in 1..3 {
            if rd[a].abs() > rd[axis].abs() {
                axis = a;
            }
        }
        let mut n = [0i32; 3];
        n[axis] = if rd[axis] > 0.0 { -1 } else { 1 };
        return n;
    }
    let mut best_axis = 0usize;
    let mut best_t = f32::NEG_INFINITY;
    for a in 0..3 {
        let ta = if rd[a].abs() < 1e-12 {
            f32::NEG_INFINITY
        } else {
            let lo = min[a] as f32;
            let hi = lo + size as f32;
            let inv = 1.0 / rd[a];
            ((lo - ro[a]) * inv).min((hi - ro[a]) * inv)
        };
        if ta > best_t {
            best_t = ta;
            best_axis = a;
        }
    }
    let mut n = [0i32; 3];
    n[best_axis] = if rd[best_axis] > 0.0 { -1 } else { 1 };
    n
}

/// Cast a ray through the tree. `ro`/`rd` are in tree space
/// (grid coords minus `built.origin`), `rd` normalized.
pub fn cast_ray(
    built: &BuiltTree,
    ro: Vec3,
    rd: Vec3,
    max_dist: f32,
    mode: TraceMode,
) -> Option<RayHit> {
    if built.flat.len() <= 1 {
        return None; // empty tree (root only)
    }
    let (root_t0, root_t1) = slab(ro, rd, [0, 0, 0], built.size);
    if root_t1 < root_t0.max(0.0) || root_t1 < 0.0 {
        return None;
    }
    let limit = root_t1.min(max_dist);
    let mut t = root_t0.max(0.0);
    for _ in 0..MAX_STEPS {
        if t > limit {
            return None;
        }
        let p = ro + rd * (t + EPS);
        if p.x < 0.0 || p.y < 0.0 || p.z < 0.0
            || p.x >= built.size as f32
            || p.y >= built.size as f32
            || p.z >= built.size as f32
        {
            return None;
        }
        match descend(built, p) {
            Descent::Occupied { node, min } => {
                let entry = &built.side[node];
                if mode == TraceMode::OpaqueOnly && entry.flags & FLAG_TRANSPARENT != 0 {
                    t = advance(ro, rd, min, 1, t);
                    continue;
                }
                let (ht0, _) = slab(ro, rd, min, 1);
                let dist = ht0.max(0.0);
                if dist > max_dist {
                    return None;
                }
                return Some(RayHit {
                    voxel: [
                        min[0] as i32 + built.origin[0],
                        min[1] as i32 + built.origin[1],
                        min[2] as i32 + built.origin[2],
                    ],
                    node_index: node as u32,
                    normal: entry_normal(ro, rd, min, 1, ht0),
                    dist,
                    material_id: entry.material_id,
                    lit: entry.lit,
                    flags: entry.flags,
                    alpha: entry.alpha as f32 / 255.0,
                });
            }
            Descent::Empty { min, size } => {
                t = advance(ro, rd, min, size, t);
            }
        }
    }
    None
}

/// Phase 9 (9.5, two-tree design): cast a ray through the
/// coarse tree and the fine tree together. `ro`/`rd` are in
/// the COARSE tree's space (block units, `rd` normalized), as
/// for [`cast_ray`]; the fine tree (built from
/// `VoxelWorld::from_fine_layer`, fine coordinates) is entered
/// by scaling the ray into fine space — fine origin offset in
/// fine units, distances divided back by `FINE` (32) — so both
/// hits are compared in block units and the NEARER hit wins.
/// The fine hit wins an exact tie: a stamped cell is the more
/// specific geometry for its space. The returned hit's
/// `voxel` is in the winning tree's world coordinates (block
/// coords for a coarse hit, fine coords for a fine hit), and
/// the `bool` reports which tree won (`true` = fine).
///
/// With `fine = None` (or an empty fine tree) the result is
/// exactly `cast_ray(coarse, …)` with `false` — the
/// byte-identical no-props contract of plan 9.10, at trace
/// level.
pub fn cast_ray_two(
    coarse: &BuiltTree,
    fine: Option<&BuiltTree>,
    ro: Vec3,
    rd: Vec3,
    max_dist: f32,
    mode: TraceMode,
) -> Option<(RayHit, bool)> {
    let coarse_hit = cast_ray(coarse, ro, rd, max_dist, mode);
    let Some(fine_tree) = fine else {
        return coarse_hit.map(|h| (h, false));
    };
    if fine_tree.flat.len() <= 1 {
        return coarse_hit.map(|h| (h, false));
    }
    let scale = mesher::fine::FINE as f32;
    let grid_ro = ro
        + Vec3::new(
            coarse.origin[0] as f32,
            coarse.origin[1] as f32,
            coarse.origin[2] as f32,
        );
    let fine_ro = grid_ro * scale
        - Vec3::new(
            fine_tree.origin[0] as f32,
            fine_tree.origin[1] as f32,
            fine_tree.origin[2] as f32,
        );
    let Some(mut fine_hit) = cast_ray(fine_tree, fine_ro, rd, max_dist * scale, mode) else {
        return coarse_hit.map(|h| (h, false));
    };
    fine_hit.dist /= scale;
    match coarse_hit {
        Some(ch) if ch.dist < fine_hit.dist => Some((ch, false)),
        _ => Some((fine_hit, true)),
    }
}

fn advance(ro: Vec3, rd: Vec3, min: [u32; 3], size: u32, t: f32) -> f32 {
    let exit = slab_exit(ro, rd, min, size);
    if exit > t {
        exit
    } else {
        t + EPS
    }
}

/// Point occupancy query (the descent half of the traversal): the
/// flattened node index of the leaf at a world (grid) coord, if any.
pub fn point_node(built: &BuiltTree, world_pos: [i32; 3]) -> Option<usize> {
    let t = [
        world_pos[0] - built.origin[0],
        world_pos[1] - built.origin[1],
        world_pos[2] - built.origin[2],
    ];
    if t.iter().any(|&v| v < 0 || v as u32 >= built.size) {
        return None;
    }
    let p = Vec3::new(t[0] as f32 + 0.5, t[1] as f32 + 0.5, t[2] as f32 + 0.5);
    match descend(built, p) {
        Descent::Occupied { node, .. } => Some(node),
        Descent::Empty { .. } => None,
    }
}

pub fn occupied_at(built: &BuiltTree, world_pos: [i32; 3]) -> bool {
    point_node(built, world_pos).is_some()
}

/// Reference DDA over the source world (Amanatides & Woo), in tree
/// space like `cast_ray`. This is the "mesher-era" traversal the tree
/// path must agree with (gates 2c.1/2c.7): same world, dumb grid
/// walk, no tree involved.
pub fn dda_reference(
    world: &VoxelWorld,
    origin: [i32; 3],
    ro: Vec3,
    rd: Vec3,
    max_dist: f32,
    mode: TraceMode,
) -> Option<RayHit> {
    let mut cell = [
        ro.x.floor() as i32,
        ro.y.floor() as i32,
        ro.z.floor() as i32,
    ];
    let step = [
        if rd.x > 0.0 { 1 } else { -1 },
        if rd.y > 0.0 { 1 } else { -1 },
        if rd.z > 0.0 { 1 } else { -1 },
    ];
    let mut t_max = [0f32; 3];
    let mut t_delta = [0f32; 3];
    for a in 0..3 {
        if rd[a].abs() < 1e-12 {
            t_max[a] = f32::INFINITY;
            t_delta[a] = f32::INFINITY;
        } else {
            let boundary = if step[a] > 0 {
                cell[a] as f32 + 1.0
            } else {
                cell[a] as f32
            };
            t_max[a] = (boundary - ro[a]) / rd[a];
            t_delta[a] = (1.0 / rd[a]).abs();
        }
    }
    let mut t = 0.0f32;
    let mut normal = [0i32; 3];
    // Check the starting cell first (dist 0, dominant-axis normal).
    for _ in 0..MAX_STEPS {
        let world_pos = [cell[0] + origin[0], cell[1] + origin[1], cell[2] + origin[2]];
        if let Some(c) = world.get(world_pos) {
            if c.lit != crate::world::LIT_BLANK
                && !(mode == TraceMode::OpaqueOnly && c.flags & FLAG_TRANSPARENT != 0)
            {
                let n = if t == 0.0 {
                    let mut axis = 0;
                    for a in 1..3 {
                        if rd[a].abs() > rd[axis].abs() {
                            axis = a;
                        }
                    }
                    let mut n = [0i32; 3];
                    n[axis] = if rd[axis] > 0.0 { -1 } else { 1 };
                    n
                } else {
                    normal
                };
                return Some(RayHit {
                    voxel: world_pos,
                    node_index: 0,
                    normal: n,
                    dist: t,
                    material_id: c.material_id,
                    lit: c.lit as u32,
                    flags: c.flags,
                    alpha: c.alpha,
                });
            }
        }
        // Advance to the next cell boundary (lowest axis wins ties,
        // matching entry_normal's tie-break).
        let mut axis = 0;
        for a in 1..3 {
            if t_max[a] < t_max[axis] {
                axis = a;
            }
        }
        t = t_max[axis];
        if t > max_dist || !t.is_finite() {
            return None;
        }
        cell[axis] += step[axis];
        t_max[axis] += t_delta[axis];
        normal = [0, 0, 0];
        normal[axis] = -step[axis];
    }
    None
}
