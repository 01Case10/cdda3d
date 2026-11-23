//! GPU ray tracer: WGSL traversal over the crate's flattened tree.
//!
//! The node buffer and its bind group are the crate's own
//! (`Tree64GpuManager`, gate 2c.10's primary arm) — group 0 in both
//! pipelines is the manager's layout, untouched. Group 1 is ours:
//! the material side-table, camera uniforms, and the outputs.
//!
//! Two entry points share one traversal implementation
//! (`TRAVERSAL_WGSL`, a line-for-line mirror of [`crate::cpu`]):
//! - `render_main`: per-pixel primary rays with sun + shadow ray,
//!   occupancy AO, `lit_level` shading, memorized desaturation,
//!   emissive passthrough, and one layer of transparent blending;
//!   writes color (rgba8unorm storage texture) + depth (f32 buffer).
//! - `query_main`: casts caller-supplied rays and writes structured
//!   hits to a buffer — the GPU half of gate 2c.3's parity check and
//!   the engine's picking path.
//!
//! Phase 10 (10.4, docs/phase10-sky-gi.md §4/§5.3): the sky
//! background and the sky GI terms come from the frozen shared
//! contract in `lighting` — [`SkyUniforms`] (128 B, uploaded to a
//! dedicated uniform buffer at group 1 binding 12, render
//! pipeline only; the query pipeline's layout is untouched) and
//! the [`SKY_WGSL`] string, embedded verbatim between the
//! traversal and render WGSL. There is no second copy of the sky
//! function in this crate.
//!
//! Coordinate-space note (corrected 10.4 fix — the original
//! slice got this wrong): `SkyState` directions are (east,
//! north, up), but this renderer's grid space is (east, *south*,
//! up) — the two differ in the sign of Y. A `SkyUniforms` built
//! by `SkyUniforms::from_sky_state` must therefore be converted
//! with `SkyUniforms::to_grid_space` (negate sun/moon Y) before
//! upload; uploaded verbatim, the noon sun (SkyState Y negative,
//! toward the south) is read as sitting toward the north.
//! [`world_to_grid`] is NOT the right conversion either: it
//! converts *mesher* Y-up world vectors (east, up, north) and
//! applied to a SkyState-space dir it puts the noon sun below
//! the horizon. [`SkyUniforms::neutral`] is already grid-space
//! (it carries the legacy grid sun dir, cf. [`sun_dir_grid`])
//! and must NOT be converted. `sky_color`'s up component in
//! grid space is `rd.z`.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use lighting::{SkyUniforms, SKY_WGSL};
use Sparse_Voxel_64Tree::Tree64GpuManager;
use wgpu::util::DeviceExt;

use crate::build::BuiltTree;
use crate::cpu::TraceMode;
use crate::lod;

/// Phase 10 (§5.3 overcast collapse): when the dominant
/// directional source's premultiplied luminance is below this,
/// the directional term collapses into the hemisphere term and
/// no shadow ray is cast. Mirrored as a literal in the render
/// WGSL (`SKY_COLLAPSE_LUMINANCE` there); keep the two in sync.
pub const SKY_COLLAPSE_LUMINANCE: f32 = 0.18;

/// Direction *toward* the sun in grid space: the raster shader's
/// world-space light dir (0.3, 0.8, 0.4) run through `world_to_grid`.
pub fn sun_dir_grid() -> Vec3 {
    Vec3::new(0.3, -0.4, 0.8).normalize()
}

/// World (mesher Y-up) -> grid (x east, y south, z up) for positions
/// and directions alike: grid = (wx, -wz, wy). A proper rotation, so
/// lengths and cross products are preserved.
pub fn world_to_grid(v: Vec3) -> Vec3 {
    Vec3::new(v.x, -v.z, v.y)
}

/// Grid -> world, the inverse of [`world_to_grid`].
pub fn grid_to_world(v: Vec3) -> Vec3 {
    Vec3::new(v.x, v.z, -v.y)
}

/// Camera for the RT renderer, in *grid* space (world-model coords;
/// the renderer subtracts the tree origin internally).
#[derive(Debug, Clone, Copy)]
pub struct RtCamera {
    pub eye: Vec3,
    pub target: Vec3,
    pub fov_y_deg: f32,
    pub aspect: f32,
}

impl RtCamera {
    /// Build from world-space eye/target (the engine's `OrbitCamera`
    /// outputs), converting into grid space.
    pub fn from_world(eye_world: Vec3, target_world: Vec3, fov_y_deg: f32, aspect: f32) -> Self {
        Self {
            eye: world_to_grid(eye_world),
            target: world_to_grid(target_world),
            fov_y_deg,
            aspect,
        }
    }

    fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let fwd = (self.target - self.eye).normalize_or_zero();
        // Grid-space up is +z.
        let right = fwd.cross(Vec3::Z).normalize_or_zero();
        let up = right.cross(fwd);
        (fwd, right, up)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    cam_pos: [f32; 3],
    max_dist: f32,
    cam_fwd: [f32; 3],
    tan_half_fov: f32,
    cam_right: [f32; 3],
    aspect: f32,
    cam_up: [f32; 3],
    tree_size: f32,
    sun_dir: [f32; 3],
    depth: u32,
    viewport: [u32; 2],
    mode: u32,
    shadows: u32,
    /// Phase 2e debug LOD view: nonzero = `shade` returns a false
    /// color by `floor(lod)` instead of albedo shading.
    debug_lod: u32,
    /// Stage A LOD tunables (see `crate::lod::LOD_BIAS` /
    /// `NDOTV_WEIGHT` — the values written here come from those
    /// constants). They occupy what was the trailing pad, keeping
    /// the buffer at the shader's expected 112 bytes.
    lod_bias: f32,
    ndotv_weight: f32,
    /// Stage F debug view: nonzero = top faces of macro materials
    /// shade with their macro *height* layer (grayscale) instead
    /// of the macro color. Proves the height layer is uploaded
    /// and sampled; border blending is a later round.
    macro_debug_height: u32,
    /// Stage F: the tree's origin in world (global grid) coords.
    /// Hit points are tree-local; macro UVs are world-based (the
    /// raster mesher's grid coords are global), so `tex_albedo`
    /// adds this back before dividing by the macro span.
    tree_origin: [f32; 3],
    _pad_origin: u32,
}

/// Phase 9 (9.5) fine-tree uniforms: per-scene constants for
/// the second tree (see the WGSL `FineUniforms`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FineUniforms {
    fine_origin: [f32; 3],
    fine_size: f32,
    fine_depth: u32,
    fine_present: u32,
    fine_lod_adjust: f32,
    _pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct RayIn {
    ro: [f32; 3],
    _p0: f32,
    rd: [f32; 3],
    _p1: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct RayOut {
    pub hit: u32,
    pub node: u32,
    pub dist: f32,
    pub _p: u32,
    /// Hit voxel in *tree* coords.
    pub voxel: [i32; 3],
    pub _p2: u32,
    pub normal: [i32; 3],
    pub _p3: u32,
}

const TRAVERSAL_WGSL: &str = r#"
struct GpuNode {
    mask_lo: u32,
    mask_hi: u32,
    child_ptr_and_leaf: u32,
    _pad: u32,
    color: vec3<f32>,
    _pad2: u32,
}
struct SideEntry {
    material_id: u32,
    lit: u32,
    flags: u32,
    alpha: u32,
}
struct Uniforms {
    cam_pos: vec3<f32>,
    max_dist: f32,
    cam_fwd: vec3<f32>,
    tan_half_fov: f32,
    cam_right: vec3<f32>,
    aspect: f32,
    cam_up: vec3<f32>,
    tree_size: f32,
    sun_dir: vec3<f32>,
    depth: u32,
    viewport: vec2<u32>,
    mode: u32,
    shadows: u32,
    debug_lod: u32,
    lod_bias: f32,
    ndotv_weight: f32,
    macro_debug_height: u32,
    tree_origin: vec3<f32>,
    _pad_o: u32,
}
@group(0) @binding(0) var<storage, read> nodes: array<GpuNode>;
@group(1) @binding(0) var<storage, read> side: array<SideEntry>;
@group(1) @binding(1) var<uniform> u: Uniforms;

// Phase 9 (9.5, two-tree design): the fine tree — a second
// manager bind group at group 2 (same layout as group 0), its
// side table and uniforms in group 1. `fine_present == 0` means
// no props are stamped: trace_two never touches the fine tree
// and the render is the coarse-only render, unchanged.
@group(2) @binding(0) var<storage, read> fine_nodes: array<GpuNode>;
@group(1) @binding(9) var<storage, read> fine_side: array<SideEntry>;
struct FineUniforms {
    // The fine tree's origin, in fine (world grid) units.
    fine_origin: vec3<f32>,
    fine_size: f32,
    fine_depth: u32,
    fine_present: u32,
    // Per-face LOD adjustment for fine leaves: -log2(32) = -5
    // (crate::lod::FINE_LOD_ADJUST), applied before the clamp.
    fine_lod_adjust: f32,
    _pad: u32,
}
@group(1) @binding(10) var<uniform> fu: FineUniforms;

fn mask_bit(n: GpuNode, idx: u32) -> bool {
    if idx < 32u {
        return ((n.mask_lo >> idx) & 1u) == 1u;
    }
    return ((n.mask_hi >> (idx - 32u)) & 1u) == 1u;
}

fn mask_rank(n: GpuNode, idx: u32) -> u32 {
    if idx == 0u {
        return 0u;
    }
    if idx < 32u {
        return countOneBits(n.mask_lo & ((1u << idx) - 1u));
    }
    var below_hi = 0u;
    if idx > 32u {
        below_hi = (1u << (idx - 32u)) - 1u;
    }
    return countOneBits(n.mask_lo) + countOneBits(n.mask_hi & below_hi);
}

fn node_is_leaf(n: GpuNode) -> bool {
    return (n.child_ptr_and_leaf & 0x80000000u) != 0u;
}

fn node_child_ptr(n: GpuNode) -> u32 {
    return n.child_ptr_and_leaf & 0x7fffffffu;
}

fn is_leaf(idx: u32) -> bool {
    return node_is_leaf(nodes[idx]);
}

fn child_ptr(idx: u32) -> u32 {
    return node_child_ptr(nodes[idx]);
}

struct Desc {
    occupied: bool,
    node: u32,
    cell_min: vec3<f32>,
    cell_size: f32,
}

fn descend(p: vec3<f32>) -> Desc {
    var node_idx = 0u;
    var cell_min = vec3<f32>(0.0, 0.0, 0.0);
    var cell_size = u.tree_size;
    for (var level = 0u; level < u.depth; level = level + 1u) {
        let child_size = cell_size * 0.25;
        let n = nodes[node_idx];
        let px = u32(clamp(floor((p.x - cell_min.x) / child_size), 0.0, 3.0));
        let py = u32(clamp(floor((p.y - cell_min.y) / child_size), 0.0, 3.0));
        let pz = u32(clamp(floor((p.z - cell_min.z) / child_size), 0.0, 3.0));
        let cidx = (pz << 4u) | (py << 2u) | px;
        let child_min = cell_min + vec3<f32>(f32(px), f32(py), f32(pz)) * child_size;
        if !mask_bit(n, cidx) {
            return Desc(false, 0u, child_min, child_size);
        }
        let child_idx = child_ptr(node_idx) + mask_rank(n, cidx);
        if is_leaf(child_idx) {
            return Desc(true, child_idx, child_min, child_size);
        }
        node_idx = child_idx;
        cell_min = child_min;
        cell_size = child_size;
    }
    return Desc(false, 0u, cell_min, cell_size);
}

fn safe_rd(rd: vec3<f32>) -> vec3<f32> {
    // Zero direction components would make the slab division produce
    // NaN when the origin sits exactly on a boundary; substitute a
    // tiny epsilon with the same sign convention as the CPU oracle
    // (parallel axis: entry -inf / exit +inf when inside, a miss when
    // outside — the huge magnitudes reproduce both).
    return vec3<f32>(
        select(rd.x, 1e-12, rd.x == 0.0),
        select(rd.y, 1e-12, rd.y == 0.0),
        select(rd.z, 1e-12, rd.z == 0.0)
    );
}

fn slab(ro: vec3<f32>, rd_in: vec3<f32>, bmin: vec3<f32>, size: f32) -> vec2<f32> {
    let rd = safe_rd(rd_in);
    let t0s = (bmin - ro) / rd;
    let t1s = (bmin + vec3<f32>(size, size, size) - ro) / rd;
    let tmin = min(t0s, t1s);
    let tmax = max(t0s, t1s);
    let t0 = max(max(tmin.x, tmin.y), tmin.z);
    let t1 = min(min(tmax.x, tmax.y), tmax.z);
    return vec2<f32>(t0, t1);
}

fn entry_normal(ro: vec3<f32>, rd_in: vec3<f32>, bmin: vec3<f32>, size: f32, t_enter: f32) -> vec3<f32> {
    let rd = safe_rd(rd_in);
    var n = vec3<f32>(0.0, 0.0, 0.0);
    if t_enter < 0.0 {
        // Origin inside the voxel: face against the dominant axis.
        var axis = 0;
        var best = abs(rd.x);
        if abs(rd.y) > best { axis = 1; best = abs(rd.y); }
        if abs(rd.z) > best { axis = 2; best = abs(rd.z); }
        var s = 1.0;
        if axis == 0 { if rd.x > 0.0 { s = -1.0; } n.x = s; }
        else if axis == 1 { if rd.y > 0.0 { s = -1.0; } n.y = s; }
        else { if rd.z > 0.0 { s = -1.0; } n.z = s; }
        return n;
    }
    let t0s = (bmin - ro) / rd;
    let t1s = (bmin + vec3<f32>(size, size, size) - ro) / rd;
    let tmin = min(t0s, t1s);
    var axis = 0;
    var best = tmin.x;
    if tmin.y > best { axis = 1; best = tmin.y; }
    if tmin.z > best { axis = 2; best = tmin.z; }
    var s = 1.0;
    if axis == 0 { if rd.x > 0.0 { s = -1.0; } n.x = s; }
    else if axis == 1 { if rd.y > 0.0 { s = -1.0; } n.y = s; }
    else { if rd.z > 0.0 { s = -1.0; } n.z = s; }
    return n;
}

fn advance(ro: vec3<f32>, rd: vec3<f32>, bmin: vec3<f32>, size: f32, t: f32) -> f32 {
    let e = slab(ro, rd, bmin, size).y;
    if e > t {
        return e;
    }
    return t + 1e-4;
}

struct Hit {
    hit: bool,
    node: u32,
    voxel_min: vec3<f32>,
    dist: f32,
    normal: vec3<f32>,
    // Phase 9 two-tree: a fine-tree hit carries `fine = true`;
    // `voxel_min`/`dist` are already converted to coarse (block)
    // units for shading, and `fvoxel_min` keeps the fine
    // tree-space cell for query output.
    fine: bool,
    fvoxel_min: vec3<f32>,
}

fn trace(ro: vec3<f32>, rd: vec3<f32>, skip_transparent: bool) -> Hit {
    var result = Hit(false, 0u, vec3<f32>(0.0, 0.0, 0.0), 0.0, vec3<f32>(0.0, 0.0, 0.0), false, vec3<f32>(0.0, 0.0, 0.0));
    if arrayLength(&nodes) <= 1u {
        return result;
    }
    let root = slab(ro, rd, vec3<f32>(0.0, 0.0, 0.0), u.tree_size);
    if root.y < max(root.x, 0.0) || root.y < 0.0 {
        return result;
    }
    let limit = min(root.y, u.max_dist);
    var t = max(root.x, 0.0);
    for (var i = 0; i < 8192; i = i + 1) {
        if t > limit {
            return result;
        }
        let p = ro + rd * (t + 1e-4);
        if any(p < vec3<f32>(0.0, 0.0, 0.0)) || any(p >= vec3<f32>(u.tree_size, u.tree_size, u.tree_size)) {
            return result;
        }
        let d = descend(p);
        if d.occupied {
            let e = side[d.node];
            if skip_transparent && (e.flags & 1u) != 0u {
                t = advance(ro, rd, d.cell_min, 1.0, t);
                continue;
            }
            let hs = slab(ro, rd, d.cell_min, 1.0);
            let dist = max(hs.x, 0.0);
            if dist > u.max_dist {
                return result;
            }
            result = Hit(true, d.node, d.cell_min, dist, entry_normal(ro, rd, d.cell_min, 1.0, hs.x), false, vec3<f32>(0.0, 0.0, 0.0));
            return result;
        }
        t = advance(ro, rd, d.cell_min, d.cell_size, t);
    }
    return result;
}

// The fine tree's traversal: `descend`/`trace` line-for-line
// over the fine globals, in fine units (1 unit = 1 fine cell).
// Kept as a separate copy rather than a parameterized shared
// function — the same reason the CPU oracle and the WGSL mirror
// each other by duplication: each tree's trace stays auditable
// against `crate::cpu::cast_ray` on its own.
fn fine_descend(p: vec3<f32>) -> Desc {
    var node_idx = 0u;
    var cell_min = vec3<f32>(0.0, 0.0, 0.0);
    var cell_size = fu.fine_size;
    for (var level = 0u; level < fu.fine_depth; level = level + 1u) {
        let child_size = cell_size * 0.25;
        let n = fine_nodes[node_idx];
        let px = u32(clamp(floor((p.x - cell_min.x) / child_size), 0.0, 3.0));
        let py = u32(clamp(floor((p.y - cell_min.y) / child_size), 0.0, 3.0));
        let pz = u32(clamp(floor((p.z - cell_min.z) / child_size), 0.0, 3.0));
        let cidx = (pz << 4u) | (py << 2u) | px;
        let child_min = cell_min + vec3<f32>(f32(px), f32(py), f32(pz)) * child_size;
        if !mask_bit(n, cidx) {
            return Desc(false, 0u, child_min, child_size);
        }
        let child_idx = node_child_ptr(n) + mask_rank(n, cidx);
        if node_is_leaf(fine_nodes[child_idx]) {
            return Desc(true, child_idx, child_min, child_size);
        }
        node_idx = child_idx;
        cell_min = child_min;
        cell_size = child_size;
    }
    return Desc(false, 0u, cell_min, cell_size);
}

fn fine_trace(ro: vec3<f32>, rd: vec3<f32>, skip_transparent: bool, max_dist_f: f32) -> Hit {
    var result = Hit(false, 0u, vec3<f32>(0.0, 0.0, 0.0), 0.0, vec3<f32>(0.0, 0.0, 0.0), true, vec3<f32>(0.0, 0.0, 0.0));
    if arrayLength(&fine_nodes) <= 1u {
        return result;
    }
    let root = slab(ro, rd, vec3<f32>(0.0, 0.0, 0.0), fu.fine_size);
    if root.y < max(root.x, 0.0) || root.y < 0.0 {
        return result;
    }
    let limit = min(root.y, max_dist_f);
    var t = max(root.x, 0.0);
    for (var i = 0; i < 8192; i = i + 1) {
        if t > limit {
            return result;
        }
        let p = ro + rd * (t + 1e-4);
        if any(p < vec3<f32>(0.0, 0.0, 0.0)) || any(p >= vec3<f32>(fu.fine_size, fu.fine_size, fu.fine_size)) {
            return result;
        }
        let d = fine_descend(p);
        if d.occupied {
            let e = fine_side[d.node];
            if skip_transparent && (e.flags & 1u) != 0u {
                t = advance(ro, rd, d.cell_min, 1.0, t);
                continue;
            }
            let hs = slab(ro, rd, d.cell_min, 1.0);
            let dist = max(hs.x, 0.0);
            if dist > max_dist_f {
                return result;
            }
            result = Hit(true, d.node, d.cell_min, dist, entry_normal(ro, rd, d.cell_min, 1.0, hs.x), true, d.cell_min);
            return result;
        }
        t = advance(ro, rd, d.cell_min, d.cell_size, t);
    }
    return result;
}

// Two-tree trace (Phase 9, 9.5): the coarse trace and the fine
// trace composited by distance — the GPU mirror of
// `crate::cpu::cast_ray_two`. The ray (coarse tree space) is
// scaled into fine space, the fine hit's distance is divided
// back by 32, and the nearer hit wins; the fine hit wins an
// exact tie. A fine hit's `voxel_min` is converted back to
// coarse tree space so shading (hit point, UVs, macro world
// position) is uniform; `fvoxel_min` keeps the fine cell.
fn trace_two(ro: vec3<f32>, rd: vec3<f32>, skip_transparent: bool) -> Hit {
    let coarse = trace(ro, rd, skip_transparent);
    if fu.fine_present == 0u {
        return coarse;
    }
    let fro = (ro + u.tree_origin) * 32.0 - fu.fine_origin;
    var fh = fine_trace(fro, rd, skip_transparent, u.max_dist * 32.0);
    if !fh.hit {
        return coarse;
    }
    fh.dist = fh.dist / 32.0;
    fh.fvoxel_min = fh.voxel_min;
    fh.voxel_min = (fh.voxel_min + fu.fine_origin) / 32.0 - u.tree_origin;
    if coarse.hit && coarse.dist < fh.dist {
        return coarse;
    }
    return fh;
}

fn point_occluded(p: vec3<f32>) -> bool {
    if any(p < vec3<f32>(0.0, 0.0, 0.0)) || any(p >= vec3<f32>(u.tree_size, u.tree_size, u.tree_size)) {
        return false;
    }
    let d = descend(p);
    if !d.occupied {
        return false;
    }
    return (side[d.node].flags & 1u) == 0u;
}
"#;

const RENDER_WGSL: &str = r#"
// Phase 10 (10.4): the shared sky uniforms (lighting::SkyUniforms,
// 128 bytes) — a dedicated buffer so the traversal Uniforms layout
// (shared with the query pipeline) stays untouched. The
// SkyUniforms struct itself comes from lighting::SKY_WGSL,
// embedded verbatim ahead of this string.
@group(1) @binding(12) var<uniform> sky_u: SkyUniforms;

@group(1) @binding(2) var color_out: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(3) var<storage, read_write> depth_out: array<f32>;
@group(1) @binding(4) var tex_array: texture_2d_array<f32>;
@group(1) @binding(5) var tex_samp: sampler;
@group(1) @binding(6) var<storage, read> mat_layers: array<vec4<u32>>;
@group(1) @binding(7) var macro_array: texture_2d_array<f32>;
@group(1) @binding(8) var<storage, read> mat_macros: array<vec4<u32>>;
@group(1) @binding(11) var<storage, read> mat_blends: array<vec4<u32>>;

// Stage F part 2 (border height-blending): half-width of the
// blend band in tiles — the Stage D weight is 0.5 at the border
// and 0 this far into the tile. Mirror of
// engine::blend::BLEND_HALF_BAND_TILES (and of the raster
// shader's BLEND_HALF); keep the three in sync.
const BLEND_HALF: f32 = 0.5;

// Ray-cone LOD (Phase 2e, docs/phase2e-mipmaps.md §2 — mirror of
// crate::lod::ray_cone_lod): the compute shader has no implicit
// derivatives, so the mip level is estimated from the ray cone.
// spread is the cone's full-angle width per unit distance, from the
// existing camera uniforms; every voxel face maps the full texture
// over 1x1 world units, so the per-face term is the constant
// Δ = 0.5·log2(texW·texH) of the array's level-0 dimensions.
fn cone_lod_adj(t: f32, n_dot_v: f32, adj: f32) -> f32 {
    let dims = textureDimensions(tex_array, 0u);
    let delta = 0.5 * log2(f32(dims.x * dims.y));
    let spread = atan(2.0 * u.tan_half_fov / f32(u.viewport.y));
    let w_hit = max(spread * t, 1e-4);
    let cosang = max(abs(n_dot_v), 1e-4);
    // Stage A tunables: u.lod_bias and u.ndotv_weight (uniform-fed
    // mirrors of crate::lod::LOD_BIAS / NDOTV_WEIGHT). Phase 9:
    // `adj` is the caller's per-face adjustment (the fine tree's
    // -5), applied BEFORE the clamp — the mirror of
    // crate::lod::ray_cone_lod_fine.
    var lod = delta + log2(w_hit) - u.ndotv_weight * log2(cosang) + u.lod_bias + adj;
    if lod != lod {
        lod = 0.0;
    }
    let max_mip = f32(textureNumLevels(tex_array) - 1u);
    return clamp(lod, 0.0, max_mip);
}

fn cone_lod(t: f32, n_dot_v: f32) -> f32 {
    return cone_lod_adj(t, n_dot_v, 0.0);
}

// Debug LOD view palette: one saturated color per mip level
// (floor(lod)). An if-chain, not an indexed array — dynamically
// indexed local arrays lose the llvmpipe device (Phase 2c finding).
fn lod_color(lod: f32) -> vec3<f32> {
    let l = i32(floor(lod));
    if l <= 0 { return vec3<f32>(0.0, 0.0, 1.0); }
    if l == 1 { return vec3<f32>(0.0, 1.0, 0.0); }
    if l == 2 { return vec3<f32>(1.0, 1.0, 0.0); }
    if l == 3 { return vec3<f32>(1.0, 0.5, 0.0); }
    if l == 4 { return vec3<f32>(1.0, 0.0, 0.0); }
    if l == 5 { return vec3<f32>(1.0, 0.0, 1.0); }
    if l == 6 { return vec3<f32>(0.0, 1.0, 1.0); }
    if l == 7 { return vec3<f32>(1.0, 1.0, 1.0); }
    return vec3<f32>(0.5, 0.0, 0.5);
}

// Face-class texture lookup (Phase 5 texture hookup): the material's
// (top, side, bottom) layers in the scene texture array, selected by
// hit normal — +z is top, -z bottom, anything else side (grid space,
// z is up). UVs come from the hit point's position inside the voxel;
// side faces map +z to v=0 so textures stand upright. Materials with
// no entry (present flag 0) keep the flat node color. Cutout
// materials whose texel is mostly transparent also fall back to the
// flat color — traversal already decided the voxel is solid, so a
// hole here would be a shading lie; true alpha-tested traversal is
// a later refinement. Phase 2e: the sample is taken at the ray-cone
// `lod` computed by the caller (see cone_lod).
fn tex_albedo(e: SideEntry, n: vec3<f32>, hitp: vec3<f32>, voxel_min: vec3<f32>, flat: vec3<f32>, lod: f32) -> vec3<f32> {
    if e.material_id >= arrayLength(&mat_layers) {
        return flat;
    }
    let ml = mat_layers[e.material_id];
    if ml.w == 0u {
        return flat;
    }
    let local = hitp - voxel_min;
    var uv = vec2<f32>(0.0, 0.0);
    var layer = ml.y;
    if abs(n.z) > 0.5 {
        uv = vec2<f32>(local.x, local.y);
        layer = select(ml.z, ml.x, n.z > 0.0);
        // Stage F: a macro material's top (+z) face samples the
        // macro array by WORLD position — uv = world_xy / span,
        // Repeat-wrapped by the sampler (the macro texel density
        // equals the tile density, so this is a pure replacement
        // for the per-tile sample, at the cone LOD shifted by the
        // span's log2). The debug view swaps in the height layer.
        if n.z > 0.0 && e.material_id < arrayLength(&mat_macros) {
            let mm = mat_macros[e.material_id];
            if mm.w != 0u {
                let world = hitp + u.tree_origin;
                let muv = vec2<f32>(world.x, world.y) / f32(mm.z);
                let mlod = max(lod - log2(f32(mm.z)), 0.0);
                if u.macro_debug_height != 0u {
                    return textureSampleLevel(macro_array, tex_samp, muv, i32(mm.y), mlod).rgb;
                }
                let mt = textureSampleLevel(macro_array, tex_samp, muv, i32(mm.x), mlod);
                var top = mt;
                // Stage F part 2 (border height-blending): a
                // blend entry names the neighbor material's macro
                // layers + the border direction. The Stage D
                // height decision (mirror of
                // engine::blend::neighbor_wins / blend_weight):
                // each side's effective height is its stored
                // height plus a weight bias, the weight falling
                // from 0.5 at the border to 0 at BLEND_HALF into
                // the tile (distance from the hit point to the
                // border edge of its own voxel, along the blend
                // direction). The higher effective height supplies
                // the albedo; ties keep the tile's own material.
                if e.material_id < arrayLength(&mat_blends) {
                    let mb = mat_blends[e.material_id];
                    if mb.w != 0u {
                        let nmt = textureSampleLevel(macro_array, tex_samp, muv, i32(mb.x), mlod);
                        let own_h = textureSampleLevel(macro_array, tex_samp, muv, i32(mm.y), mlod).r;
                        let nbr_h = textureSampleLevel(macro_array, tex_samp, muv, i32(mb.y), mlod).r;
                        var dist = 0.0;
                        if mb.z == 0u {
                            dist = (voxel_min.x + 1.0) - hitp.x;
                        } else if mb.z == 1u {
                            dist = hitp.x - voxel_min.x;
                        } else if mb.z == 2u {
                            dist = (voxel_min.y + 1.0) - hitp.y;
                        } else {
                            dist = hitp.y - voxel_min.y;
                        }
                        let wgt = clamp(0.5 - dist / (2.0 * BLEND_HALF), 0.0, 1.0);
                        if nbr_h + wgt * 2.0 > own_h + (1.0 - wgt) * 2.0 {
                            top = nmt;
                        }
                    }
                }
                if (e.flags & 2u) != 0u && top.a < 0.5 {
                    return flat;
                }
                return top.rgb;
            }
        }
    } else if abs(n.x) > 0.5 {
        uv = vec2<f32>(local.y, 1.0 - local.z);
    } else {
        uv = vec2<f32>(local.x, 1.0 - local.z);
    }
    let texel = textureSampleLevel(tex_array, tex_samp, uv, i32(layer), lod);
    if (e.flags & 2u) != 0u && texel.a < 0.5 {
        return flat;
    }
    return texel.rgb;
}

fn lit_factor(lit: u32) -> f32 {
    switch lit {
        case 0u: { return 0.25; }
        case 1u: { return 0.45; }
        case 2u: { return 0.65; }
        case 3u: { return 0.85; }
        case 4u: { return 1.0; }
        case 5u: { return 0.5; }
        default: { return 0.0; }
    }
}

fn shade(ro: vec3<f32>, rd: vec3<f32>, h: Hit) -> vec3<f32> {
    // Phase 9 two-tree: a fine hit shades from the FINE tree's
    // node/side entries (same node index space, other arrays).
    var node: GpuNode;
    var e: SideEntry;
    if h.fine {
        node = fine_nodes[h.node];
        e = fine_side[h.node];
    } else {
        node = nodes[h.node];
        e = side[h.node];
    }
    let hitp = ro + rd * h.dist;
    // One cone LOD per hit, from this hit's own distance — the
    // transparent overlay hit is the same primary ray continued, so
    // it re-enters here with its own (farther) distance. Fine hits
    // take the -5 per-face adjustment (fu.fine_lod_adjust).
    let lod = cone_lod_adj(h.dist, dot(h.normal, -rd), select(0.0, fu.fine_lod_adjust, h.fine));
    if u.debug_lod != 0u {
        return lod_color(lod);
    }
    var albedo = tex_albedo(e, h.normal, hitp, h.voxel_min, node.color, lod);
    if u.macro_debug_height != 0u {
        // Stage F debug view: tex_albedo already swapped macro
        // top faces to the height layer; return it unlit, like
        // the debug_lod view above, so the view stays the
        // grayscale height data it exists to inspect. The §5.3
        // hemisphere light is coloured and would tint it (the
        // legacy scalar light could not); gate stage_f measures
        // the debug view's grayscale-ness.
        return albedo;
    }
    if (e.flags & 8u) != 0u {
        // Memorized: remembered, not currently seen — desaturate.
        let lum = dot(albedo, vec3<f32>(0.299, 0.587, 0.114));
        albedo = mix(albedo, vec3<f32>(lum, lum, lum), 0.6);
    }
    if (e.flags & 4u) != 0u {
        // Emissive: unlit, feeds the Phase 2b bloom pass downstream.
        return albedo * 1.6;
    }
    let n = h.normal;
    // Phase 10 (10.4, §5.3): sky GI from the shared SkyUniforms,
    // replacing the hardcoded `0.35 + 0.85 * ndl` model.
    //
    // Hemisphere ambient: mix(ground, zenith, n.z * 0.5 + 0.5)
    // (grid space: +z is up) * ambient_scale (packed in zenith.w).
    // The hemisphere term is NOT scaled by the per-tile sky
    // factor: it is the ambient floor lamp-lit interiors rely on
    // (§3.4 — interiors lit by lamps look as they do today; the
    // lit_factor ceiling below still caps the total). The
    // directional terms ARE scaled by it (§3.4, decision 4A;
    // side-table flag bit 32, FLAG_SKY_EXPOSED).
    let sky_factor = select(0.0, 1.0, (e.flags & 32u) != 0u);
    let hemi = mix(sky_u.ground_col.xyz, sky_u.zenith.xyz, n.z * 0.5 + 0.5);
    var light = hemi * sky_u.zenith.w;
    // Directional: sun + moon, colours pre-multiplied by
    // intensity in the uniforms. ONE shadow ray, toward the
    // dominant source (greater premultiplied luminance); the
    // weaker source is unshadowed (v1 limit). Two-tree: props
    // occlude too (same coarse-bias note as Phase 9).
    let sun_c = sky_u.sun_color.xyz;
    let moon_c = sky_u.moon_color.xyz;
    let sun_i = dot(sun_c, vec3<f32>(0.299, 0.587, 0.114));
    let moon_i = dot(moon_c, vec3<f32>(0.299, 0.587, 0.114));
    let dom_i = max(sun_i, moon_i);
    // Overcast collapse threshold: mirror of
    // crate::render::SKY_COLLAPSE_LUMINANCE (0.18).
    if dom_i < 0.18 {
        // Overcast collapse (§5.3): the directional energy folds
        // into the hemisphere term; no shadow ray is cast.
        light = light + hemi * dom_i;
    } else {
        let moon_dom = moon_i > sun_i;
        var dom_vis = 1.0;
        if u.shadows != 0u {
            let dom_dir = select(sky_u.sun_dir.xyz, sky_u.moon_dir.xyz, moon_dom);
            let sh = trace_two(hitp + n * 0.01, dom_dir, true);
            dom_vis = select(1.0, 0.0, sh.hit);
        }
        let sun_vis = select(dom_vis, 1.0, moon_dom);
        let moon_vis = select(1.0, dom_vis, moon_dom);
        light = light + (sun_c * max(dot(n, sky_u.sun_dir.xyz), 0.0) * sun_vis
            + moon_c * max(dot(n, sky_u.moon_dir.xyz), 0.0) * moon_vis) * sky_factor;
    }
    // Cheap occupancy AO: the four cells just outside the face,
    // diagonally adjacent along its tangents.
    var t1 = vec3<f32>(1.0, 0.0, 0.0);
    var t2 = vec3<f32>(0.0, 1.0, 0.0);
    if abs(n.x) > 0.5 {
        t1 = vec3<f32>(0.0, 1.0, 0.0);
        t2 = vec3<f32>(0.0, 0.0, 1.0);
    } else if abs(n.y) > 0.5 {
        t1 = vec3<f32>(1.0, 0.0, 0.0);
        t2 = vec3<f32>(0.0, 0.0, 1.0);
    }
    let center = h.voxel_min + n + vec3<f32>(0.5, 0.5, 0.5);
    var occ = 0.0;
    if point_occluded(center + t1) { occ = occ + 1.0; }
    if point_occluded(center - t1) { occ = occ + 1.0; }
    if point_occluded(center + t2) { occ = occ + 1.0; }
    if point_occluded(center - t2) { occ = occ + 1.0; }
    let ao = 1.0 - 0.10 * occ;
    // Water surface (terrain item 2 follow-up,
    // 2026-11-23): FLAG_WATER cells (bit 4, from the
    // block's WATER_ALPHA marker) with an up face shade
    // as water -- a fresnel sheen toward sky colour is
    // mixed into the albedo before lighting, and a
    // Blinn specular from the sun is added after.
    // Mirrors the raster treatment in engine::render;
    // constants are shared by hand, keep in sync.
    var spec_add = 0.0;
    if (e.flags & 16u) != 0u && n.z > 0.5 {
        let v = -rd;
        let fres = pow(1.0 - max(dot(n, v), 0.0), 3.0);
        albedo = mix(albedo, vec3<f32>(0.55, 0.65, 0.75), fres * 0.45);
        let hv = normalize(u.sun_dir + v);
        spec_add = pow(max(dot(n, hv), 0.0), 64.0) * 0.35;
    }
    // The lit_factor ceiling multiply stays LAST (§3.4 ceiling
    // rule: sky GI never reveals what the game marks dark/unseen).
    var col = albedo * light * (lit_factor(e.lit) * ao);
    if (e.flags & 8u) != 0u {
        // Memorized, second stage: the albedo desaturation above
        // sufficed when the light was a scalar, but the §5.3
        // hemisphere light is coloured and its chroma
        // re-saturates the product (gate 2h.3 measured memorized
        // saturation 0.592 vs visible 0.542 — no collapse). Like
        // the raster path, desaturate the shaded colour too
        // (0.30 keep, the pinned presentation).
        let clum = dot(col, vec3<f32>(0.299, 0.587, 0.114));
        col = mix(vec3<f32>(clum), col, 0.30);
    }
    return col + vec3<f32>(spec_add, spec_add, spec_add) * lit_factor(e.lit);
}

fn sky(rd: vec3<f32>) -> vec3<f32> {
    // Phase 10 (10.4, §4): the shared sky function from
    // lighting::SKY_WGSL — one source for both renderers. Grid
    // space: the up component is rd.z. At SkyUniforms::neutral()
    // this returns exactly the pre-10.4 hardcoded gradient (the
    // freeze gate proves it for the CPU mirror; gate 10.3a's RT
    // half proves it for this GPU path).
    return sky_color(rd, rd.z, sky_u);
}

@compute @workgroup_size(8, 8)
fn render_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.viewport.x || gid.y >= u.viewport.y {
        return;
    }
    let px = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5);
    let ndc = vec2<f32>(
        (px.x / f32(u.viewport.x)) * 2.0 - 1.0,
        1.0 - (px.y / f32(u.viewport.y)) * 2.0
    );
    let rd = normalize(u.cam_fwd
        + ndc.x * u.tan_half_fov * u.aspect * u.cam_right
        + ndc.y * u.tan_half_fov * u.cam_up);
    let ro = u.cam_pos;
    let first = trace_two(ro, rd, false);
    let opq = trace_two(ro, rd, true);
    var col = sky(rd);
    var depth = u.max_dist;
    if opq.hit {
        col = shade(ro, rd, opq);
        depth = opq.dist;
    }
    // The overlay's flags/alpha come from whichever tree the
    // first hit belongs to.
    var fe: SideEntry;
    if first.fine {
        fe = fine_side[first.node];
    } else {
        fe = side[first.node];
    }
    if first.hit && (fe.flags & 1u) != 0u && (!opq.hit || first.dist < opq.dist) {
        let gcol = shade(ro, rd, first);
        let a = f32(fe.alpha) / 255.0;
        col = mix(col, gcol, a);
        depth = first.dist;
    }
    textureStore(color_out, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(col, 1.0));
    depth_out[gid.y * u.viewport.x + gid.x] = depth;
}
"#;

const QUERY_WGSL: &str = r#"
struct RayIn {
    ro: vec3<f32>,
    _p0: f32,
    rd: vec3<f32>,
    _p1: f32,
}
struct RayOut {
    hit: u32,
    node: u32,
    dist: f32,
    _p: u32,
    voxel: vec3<i32>,
    _p2: u32,
    normal: vec3<i32>,
    _p3: u32,
}
@group(1) @binding(2) var<storage, read> rays: array<RayIn>;
@group(1) @binding(3) var<storage, read_write> results: array<RayOut>;

@compute @workgroup_size(64)
fn query_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&rays) {
        return;
    }
    let r = rays[i];
    let h = trace_two(r.ro, r.rd, u.mode != 0u);
    var out = RayOut(0u, 0u, 0.0, 0u, vec3<i32>(0, 0, 0), 0u, vec3<i32>(0, 0, 0), 0u);
    if h.hit {
        out.hit = 1u;
        out.node = h.node;
        out.dist = h.dist;
        // A fine hit reports its cell in FINE tree coords,
        // mirroring the coarse convention (tree coords, not
        // world) — cpu::cast_ray_two's fine voxel minus the
        // fine origin.
        out.voxel = select(vec3<i32>(h.voxel_min), vec3<i32>(h.fvoxel_min), h.fine);
        out.normal = vec3<i32>(h.normal);
    }
    results[i] = out;
}
"#;

/// The full WGSL sources (traversal + entry points), exposed for
/// diagnostics and source-level test assertions.
pub fn wgsl_sources() -> (String, String) {
    (
        format!("{TRAVERSAL_WGSL}\n{SKY_WGSL}\n{RENDER_WGSL}"),
        format!("{TRAVERSAL_WGSL}\n{QUERY_WGSL}"),
    )
}

/// CPU-side description of a scene texture set: a stack of square
/// RGBA layers (the scene-local texture array — only the layers the
/// scene's materials actually use, already remapped by the caller)
/// plus, per material id, the `(top, side, bottom, present)` layer
/// indices into that stack. Materials absent from `material_layers`
/// (or with `present == 0`) shade with their flat node color.
#[derive(Debug, Clone)]
pub struct TextureSpec {
    pub tile_size: u32,
    /// One RGBA byte buffer per layer, `tile_size * tile_size * 4`
    /// bytes each, row-major, first row = texture v=0.
    pub layers: Vec<Vec<u8>>,
    /// Indexed by material id.
    pub material_layers: Vec<[u32; 4]>,
    /// Stage F: edge length of the macro layers (128 for the
    /// voxelizer's v2 macros); 0 = the spec carries no macro set.
    pub macro_size: u32,
    /// Stage F: the macro layer stack, `macro_size * macro_size *
    /// 4` bytes per layer: color layers first (`0..C`), then
    /// height layers (`C..2C`) in the same material order.
    pub macro_layers: Vec<Vec<u8>>,
    /// Stage F: per material id, `(color_layer, height_layer,
    /// span_tiles, present)` into `macro_layers`. Materials with
    /// `present == 0` (or no entry) keep the per-tile top face.
    pub material_macros: Vec<[u32; 4]>,
    /// Stage F part 2 (border height-blending): per material id,
    /// `(neighbor_color_layer, neighbor_height_layer, dir,
    /// present)` into `macro_layers` — for a blend *virtual* id,
    /// the neighbor macro material its top face height-blends
    /// against (the virtual id's own macro entry in
    /// `material_macros` is its base material's). `dir` is a
    /// cardinal index (0 +X, 1 −X, 2 +Y, 3 −Y): the neighbor lies
    /// across that edge of the tile. Empty (or all-zero) = no
    /// blends; only the RT path consumes this — the raster path
    /// packs the same fields into its vertex layer words.
    pub material_blends: Vec<[u32; 4]>,
}

/// GPU texture set bound to the render pipeline: texture array +
/// sampler + the material-layer lookup buffer. Texels are sampled
/// raw (Rgba8Unorm, no sRGB decode) — the same display-value
/// convention as node colors in this renderer.
pub struct TextureSet {
    /// Kept alive for `view`; not read directly.
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    layers_buffer: wgpu::Buffer,
    pub layer_count: usize,
    pub material_count: usize,
    /// Mip levels per layer, level 0 included (Phase 2e).
    pub mip_levels: u32,
    /// Stage F: the macro texture array (kept alive for
    /// `macro_view`) and the per-material macro lookup buffer.
    /// Always present (a 1x1 dummy when the spec has no macros)
    /// so the pipeline layout never changes.
    _macro_texture: wgpu::Texture,
    macro_view: wgpu::TextureView,
    macro_buffer: wgpu::Buffer,
    pub macro_layer_count: usize,
    /// Stage F part 2: the per-material blend lookup buffer
    /// (`TextureSpec::material_blends`). Always present (a single
    /// all-zero entry when the spec carries no blends) so the
    /// pipeline layout never changes.
    blends_buffer: wgpu::Buffer,
}

impl TextureSet {
    /// The no-textures set: a 1x1 white layer and a lookup buffer
    /// whose only entry says "not present", so every material takes
    /// the flat-color path. Bound unconditionally so the pipeline
    /// layout never changes.
    pub fn dummy(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_spec(
            device,
            queue,
            &TextureSpec {
                tile_size: 1,
                layers: vec![vec![255, 255, 255, 255]],
                material_layers: vec![[0, 0, 0, 0]],
                macro_size: 0,
                macro_layers: Vec::new(),
                material_macros: Vec::new(),
                material_blends: Vec::new(),
            },
        )
        .expect("dummy texture spec is valid")
    }

    pub fn from_spec(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spec: &TextureSpec,
    ) -> Result<Self, String> {
        if spec.tile_size == 0 {
            return Err("texture tile_size is 0".into());
        }
        if spec.layers.is_empty() {
            return Err("texture set has no layers".into());
        }
        let expect = (spec.tile_size * spec.tile_size * 4) as usize;
        for (i, l) in spec.layers.iter().enumerate() {
            if l.len() != expect {
                return Err(format!(
                    "texture layer {i} has {} bytes, expected {expect}",
                    l.len()
                ));
            }
        }
        let material_layers: Vec<[u32; 4]> = if spec.material_layers.is_empty() {
            vec![[0, 0, 0, 0]]
        } else {
            spec.material_layers.clone()
        };
        for (id, ml) in material_layers.iter().enumerate() {
            if ml[3] != 0 && ml[..3].iter().any(|&l| l as usize >= spec.layers.len()) {
                return Err(format!(
                    "material {id} references layer out of range ({} layers)",
                    spec.layers.len()
                ));
            }
        }
        // Phase 2e: the spec stays level-0 layers only; the mip
        // chain is generated here, per layer, with the CPU
        // premultiplied alpha-aware downsample (crate::lod).
        let chains: Vec<Vec<Vec<u8>>> = spec
            .layers
            .iter()
            .map(|l| lod::mip_chain(l, spec.tile_size))
            .collect();
        let mip_levels = chains.first().map(|c| c.len() as u32).unwrap_or(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rt-texture-array"),
            size: wgpu::Extent3d {
                width: spec.tile_size,
                height: spec.tile_size,
                depth_or_array_layers: spec.layers.len() as u32,
            },
            mip_level_count: mip_levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (i, chain) in chains.iter().enumerate() {
            for (level, data) in chain.iter().enumerate() {
                let level_size = (spec.tile_size >> level).max(1);
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: level as u32,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(level_size * 4),
                        rows_per_image: Some(level_size),
                    },
                    wgpu::Extent3d {
                        width: level_size,
                        height: level_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        // Trilinear sampler. The LOD clamps must span the whole
        // chain: a too-low lod_max_clamp silently degrades mips to
        // level 0 (docs/phase2e-mipmaps.md §3 W2 warning).
        // Stage A: magnification is Nearest — the pre-fix Linear
        // mag was the RT half of the "everything is soft/blurry"
        // symptom (close-up faces were always bilinear-blurred).
        // Min/mip stay Linear (trilinear minification via the
        // ray-cone LOD). Address mode is Repeat so filtering wraps
        // across voxel borders for tileable textures instead of
        // clamping to the edge texel.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rt-texture-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: (mip_levels - 1) as f32,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let layers_data: Vec<[u32; 4]> = material_layers;
        let layers_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rt-material-layers"),
            contents: bytemuck::cast_slice(&layers_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        // Stage F: the macro set. Same construction as the tile
        // array (CPU mip chain per layer, one texture array), at
        // macro_size; a 1x1 dummy when the spec carries no macros.
        let (macro_size, macro_layers, material_macros): (u32, Vec<Vec<u8>>, Vec<[u32; 4]>) =
            if spec.macro_size > 0 && !spec.macro_layers.is_empty() {
                let expect = (spec.macro_size * spec.macro_size * 4) as usize;
                for (i, l) in spec.macro_layers.iter().enumerate() {
                    if l.len() != expect {
                        return Err(format!(
                            "macro layer {i} has {} bytes, expected {expect}",
                            l.len()
                        ));
                    }
                }
                for (id, mm) in spec.material_macros.iter().enumerate() {
                    if mm[3] != 0
                        && (mm[0] as usize >= spec.macro_layers.len()
                            || mm[1] as usize >= spec.macro_layers.len())
                    {
                        return Err(format!(
                            "material {id} references macro layer out of range ({} layers)",
                            spec.macro_layers.len()
                        ));
                    }
                }
                (
                    spec.macro_size,
                    spec.macro_layers.clone(),
                    if spec.material_macros.is_empty() {
                        vec![[0, 0, 0, 0]]
                    } else {
                        spec.material_macros.clone()
                    },
                )
            } else {
                (1, vec![vec![255, 255, 255, 255]], vec![[0, 0, 0, 0]])
            };
        let macro_chains: Vec<Vec<Vec<u8>>> = macro_layers
            .iter()
            .map(|l| lod::mip_chain(l, macro_size))
            .collect();
        let macro_mips = macro_chains.first().map(|c| c.len() as u32).unwrap_or(1);
        let macro_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rt-macro-array"),
            size: wgpu::Extent3d {
                width: macro_size,
                height: macro_size,
                depth_or_array_layers: macro_layers.len() as u32,
            },
            mip_level_count: macro_mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (i, chain) in macro_chains.iter().enumerate() {
            for (level, data) in chain.iter().enumerate() {
                let level_size = (macro_size >> level).max(1);
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &macro_texture,
                        mip_level: level as u32,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(level_size * 4),
                        rows_per_image: Some(level_size),
                    },
                    wgpu::Extent3d {
                        width: level_size,
                        height: level_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        let macro_view = macro_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let macro_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rt-material-macros"),
            contents: bytemuck::cast_slice(&material_macros),
            usage: wgpu::BufferUsages::STORAGE,
        });
        // Stage F part 2: the blend lookup buffer, validated
        // against the macro layer stack like the macro entries.
        let material_blends: Vec<[u32; 4]> = if spec.material_blends.is_empty() {
            vec![[0, 0, 0, 0]]
        } else {
            spec.material_blends.clone()
        };
        for (id, mb) in material_blends.iter().enumerate() {
            if mb[3] != 0
                && (mb[0] as usize >= macro_layers.len() || mb[1] as usize >= macro_layers.len())
            {
                return Err(format!(
                    "material {id} references blend macro layer out of range ({} layers)",
                    macro_layers.len()
                ));
            }
        }
        let blends_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rt-material-blends"),
            contents: bytemuck::cast_slice(&material_blends),
            usage: wgpu::BufferUsages::STORAGE,
        });
        Ok(Self {
            _texture: texture,
            view,
            sampler,
            layers_buffer,
            layer_count: spec.layers.len(),
            material_count: layers_data.len(),
            mip_levels,
            _macro_texture: macro_texture,
            macro_view,
            macro_buffer,
            macro_layer_count: macro_layers.len(),
            blends_buffer,
        })
    }
}

/// An uploaded tree: the crate manager (nodes) + our side-table.
pub struct RtScene {
    pub manager: Tree64GpuManager,
    side_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    /// Phase 10 (10.4): the shared sky uniforms buffer (128 B),
    /// bound at group 1 binding 12 in the render pipeline.
    /// Starts at [`SkyUniforms::neutral`] — the §6 neutral block,
    /// under which the sky background reproduces the pre-10.4
    /// gradient exactly.
    sky_uniform_buffer: wgpu::Buffer,
    /// Bound textures; starts as the dummy set (all flat color)
    /// until `attach_textures` replaces it.
    pub textures: TextureSet,
    pub node_count: usize,
    pub depth: usize,
    pub size: u32,
    pub origin: [i32; 3],
    /// Phase 9 (9.5) fine tree: a second manager (bound at group
    /// 2) + its side table and uniforms. When no fine tree was
    /// supplied it mirrors the coarse tree with
    /// `fine_present = false`, and the shader never traces it.
    pub fine_manager: Tree64GpuManager,
    fine_side_buffer: wgpu::Buffer,
    fine_uniform_buffer: wgpu::Buffer,
    pub fine_present: bool,
    pub fine_node_count: usize,
    pub fine_depth: usize,
    pub fine_size: u32,
    pub fine_origin: [i32; 3],
}

impl RtScene {
    /// Phase 10 (10.4): upload the shared sky uniforms for
    /// subsequent renders. This takes the block **as given, in
    /// grid space** — it cannot distinguish a neutral block from
    /// a converted one, so the conversion belongs to the caller:
    /// pass either [`SkyUniforms::neutral`] as-is (already
    /// grid-space; do NOT convert it) or
    /// `SkyUniforms::from_sky_state(...).to_grid_space()`. See
    /// the module doc's coordinate note. Scenes start at
    /// [`SkyUniforms::neutral`].
    pub fn set_sky(&self, queue: &wgpu::Queue, sky: &SkyUniforms) {
        let rows = sky.as_rows();
        queue.write_buffer(
            &self.sky_uniform_buffer,
            0,
            bytemuck::cast_slice(rows.as_slice()),
        );
    }

    /// Replace the scene's texture set with a real one built from
    /// `spec` (see `TextureSpec`). Materials the spec doesn't cover
    /// keep flat-color shading.
    pub fn attach_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spec: &TextureSpec,
    ) -> Result<(), String> {
        self.textures = TextureSet::from_spec(device, queue, spec)?;
        Ok(())
    }
}

/// A rendered frame: color texture (rgba8unorm) + depth buffer
/// (one f32 per pixel, row-major; `max_dist` where the ray missed).
pub struct RtFrame {
    pub color: wgpu::Texture,
    pub depth: wgpu::Buffer,
    pub size: (u32, u32),
    pub max_dist: f32,
}

pub struct RtRenderer {
    render_pipeline: wgpu::ComputePipeline,
    query_pipeline: wgpu::ComputePipeline,
    render_group1_layout: wgpu::BindGroupLayout,
    query_group1_layout: wgpu::BindGroupLayout,
}

fn side_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl RtRenderer {
    pub fn new(
        device: &wgpu::Device,
        manager_layout: &wgpu::BindGroupLayout,
        fine_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let render_group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rt-render-group1"),
            entries: &[
                side_layout_entry(0),
                uniform_layout_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                side_layout_entry(6),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                side_layout_entry(8),
                // Phase 9 fine tree: side table + uniforms.
                side_layout_entry(9),
                uniform_layout_entry(10),
                // Stage F part 2: per-material blend entries.
                side_layout_entry(11),
                // Phase 10 (10.4): shared sky uniforms — render
                // pipeline only (the query layout below is
                // unchanged, so its buffer layout stays stable).
                uniform_layout_entry(12),
            ],
        });
        let query_group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rt-query-group1"),
            entries: &[
                side_layout_entry(0),
                uniform_layout_entry(1),
                side_layout_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Phase 9 fine tree: side table + uniforms.
                side_layout_entry(9),
                uniform_layout_entry(10),
            ],
        });
        let render_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-render-shader"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{TRAVERSAL_WGSL}\n{SKY_WGSL}\n{RENDER_WGSL}").into(),
            ),
        });
        let query_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-query-shader"),
            source: wgpu::ShaderSource::Wgsl(format!("{TRAVERSAL_WGSL}\n{QUERY_WGSL}").into()),
        });
        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt-render-layout"),
            bind_group_layouts: &[manager_layout, &render_group1_layout, fine_layout],
            push_constant_ranges: &[],
        });
        let query_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt-query-layout"),
            bind_group_layouts: &[manager_layout, &query_group1_layout, fine_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rt-render-pipeline"),
            layout: Some(&render_layout),
            module: &render_module,
            entry_point: Some("render_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let query_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("rt-query-pipeline"),
            layout: Some(&query_layout),
            module: &query_module,
            entry_point: Some("query_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            render_pipeline,
            query_pipeline,
            render_group1_layout,
            query_group1_layout,
        }
    }

    /// Create a renderer + scene in one step (the manager's layout is
    /// needed to build the pipelines, and the manager lives in the
    /// scene — this constructor keeps that ordering in one place).
    pub fn with_scene(device: &wgpu::Device, queue: &wgpu::Queue, built: &BuiltTree) -> (Self, RtScene) {
        Self::with_scene_fine(device, queue, built, None)
    }

    /// Phase 9 (9.5): as `with_scene`, plus the fine tree built
    /// from `VoxelWorld::from_fine_layer`. The fine tree gets its
    /// own manager (bound at group 2 with ITS layout — each
    /// manager creates its own layout object, so the pipeline
    /// layout is built from both), side table and uniforms.
    /// `fine = None` mirrors the coarse tree into the fine slots
    /// with `fine_present = false`: the shader's two-tree trace
    /// early-outs and the render is the coarse-only render.
    pub fn with_scene_fine(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        built: &BuiltTree,
        fine: Option<&BuiltTree>,
    ) -> (Self, RtScene) {
        let manager = Tree64GpuManager::new(device, &built.tree);
        let fine_built = fine.unwrap_or(built);
        let fine_manager = Tree64GpuManager::new(device, &fine_built.tree);
        let renderer = Self::new(
            device,
            &manager.contree_bind_group_layout,
            &fine_manager.contree_bind_group_layout,
        );
        let fine_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rt-fine-uniforms"),
            size: std::mem::size_of::<FineUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &fine_uniform_buffer,
            0,
            bytemuck::bytes_of(&FineUniforms {
                fine_origin: [
                    fine_built.origin[0] as f32,
                    fine_built.origin[1] as f32,
                    fine_built.origin[2] as f32,
                ],
                fine_size: fine_built.size as f32,
                fine_depth: fine_built.depth as u32,
                fine_present: fine.is_some() as u32,
                fine_lod_adjust: crate::lod::FINE_LOD_ADJUST,
                _pad: 0,
            }),
        );
        let sky_rows = SkyUniforms::neutral().as_rows();
        let scene = RtScene {
            side_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rt-side-table"),
                contents: bytemuck::cast_slice(&built.side),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            uniform_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("rt-uniforms"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            sky_uniform_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rt-sky-uniforms"),
                contents: bytemuck::cast_slice(sky_rows.as_slice()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            node_count: built.node_count(),
            depth: built.depth,
            size: built.size,
            origin: built.origin,
            textures: TextureSet::dummy(device, queue),
            manager,
            fine_side_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rt-fine-side-table"),
                contents: bytemuck::cast_slice(&fine_built.side),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            fine_uniform_buffer,
            fine_present: fine.is_some(),
            fine_node_count: fine_built.node_count(),
            fine_depth: fine_built.depth,
            fine_size: fine_built.size,
            fine_origin: fine_built.origin,
            fine_manager,
        };
        (renderer, scene)
    }

    fn write_uniforms(&self, queue: &wgpu::Queue, scene: &RtScene, u: &Uniforms) {
        queue.write_buffer(&scene.uniform_buffer, 0, bytemuck::bytes_of(u));
    }

    fn base_uniforms(&self, scene: &RtScene) -> Uniforms {
        Uniforms {
            cam_pos: [0.0; 3],
            max_dist: 1000.0,
            cam_fwd: [0.0, 0.0, -1.0],
            tan_half_fov: 0.5,
            cam_right: [1.0, 0.0, 0.0],
            aspect: 1.0,
            cam_up: [0.0, 1.0, 0.0],
            tree_size: scene.size as f32,
            sun_dir: sun_dir_grid().to_array(),
            depth: scene.depth as u32,
            viewport: [1, 1],
            mode: 0,
            shadows: 1,
            debug_lod: 0,
            lod_bias: crate::lod::LOD_BIAS,
            ndotv_weight: crate::lod::NDOTV_WEIGHT,
            macro_debug_height: 0,
            tree_origin: [
                scene.origin[0] as f32,
                scene.origin[1] as f32,
                scene.origin[2] as f32,
            ],
            _pad_origin: 0,
        }
    }

    /// Render one frame from `camera` (grid space) at `size` pixels.
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RtScene,
        camera: &RtCamera,
        size: (u32, u32),
        shadows: bool,
    ) -> RtFrame {
        self.render_impl(device, queue, scene, camera, size, shadows, 0, 0)
    }

    /// Render with the Phase 2e debug LOD view: every hit shades as
    /// a false color by `floor(lod)` (see `lod_color` in the WGSL)
    /// instead of its albedo shading. Validation/tuning only.
    pub fn render_debug_lod(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RtScene,
        camera: &RtCamera,
        size: (u32, u32),
    ) -> RtFrame {
        self.render_impl(device, queue, scene, camera, size, false, 1, 0)
    }

    /// Render with the Stage F macro height debug view: top faces
    /// of macro materials shade with their macro height layer
    /// (grayscale) instead of the macro color. Validation only —
    /// it proves the height layer is uploaded and sampled.
    pub fn render_macro_debug_height(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RtScene,
        camera: &RtCamera,
        size: (u32, u32),
    ) -> RtFrame {
        self.render_impl(device, queue, scene, camera, size, true, 0, 1)
    }

    #[allow(clippy::too_many_arguments)]
    fn render_impl(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RtScene,
        camera: &RtCamera,
        size: (u32, u32),
        shadows: bool,
        debug_lod: u32,
        macro_debug_height: u32,
    ) -> RtFrame {
        let (width, height) = (size.0.max(1), size.1.max(1));
        let origin = Vec3::new(
            scene.origin[0] as f32,
            scene.origin[1] as f32,
            scene.origin[2] as f32,
        );
        let (fwd, right, up) = camera.basis();
        let mut uniforms = self.base_uniforms(scene);
        uniforms.cam_pos = (camera.eye - origin).to_array();
        uniforms.cam_fwd = fwd.to_array();
        uniforms.cam_right = right.to_array();
        uniforms.cam_up = up.to_array();
        uniforms.tan_half_fov = (camera.fov_y_deg.to_radians() * 0.5).tan();
        uniforms.aspect = camera.aspect;
        uniforms.viewport = [width, height];
        uniforms.shadows = shadows as u32;
        uniforms.debug_lod = debug_lod;
        uniforms.macro_debug_height = macro_debug_height;
        self.write_uniforms(queue, scene, &uniforms);

        let color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rt-color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rt-depth"),
            size: (width * height * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let color_view = color.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rt-render-bg"),
            layout: &self.render_group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: scene.side_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: depth.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&scene.textures.view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&scene.textures.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: scene.textures.layers_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&scene.textures.macro_view),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: scene.textures.macro_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: scene.fine_side_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: scene.fine_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: scene.textures.blends_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: scene.sky_uniform_buffer.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rt-render-encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("rt-render-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &scene.manager.contree_bind_group, &[]);
            pass.set_bind_group(1, &bind_group, &[]);
            pass.set_bind_group(2, &scene.fine_manager.contree_bind_group, &[]);
            pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
        }
        queue.submit([encoder.finish()]);
        RtFrame {
            color,
            depth,
            size: (width, height),
            max_dist: uniforms.max_dist,
        }
    }

    /// Cast `rays` (tree space: grid coords minus `scene.origin`,
    /// normalized directions) on the GPU. Returns one entry per ray.
    pub fn trace_rays(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RtScene,
        rays: &[(Vec3, Vec3)],
        mode: TraceMode,
        max_dist: f32,
    ) -> Vec<RayOut> {
        if rays.is_empty() {
            return Vec::new();
        }
        let mut uniforms = self.base_uniforms(scene);
        uniforms.max_dist = max_dist;
        uniforms.mode = match mode {
            TraceMode::FirstHit => 0,
            TraceMode::OpaqueOnly => 1,
        };
        self.write_uniforms(queue, scene, &uniforms);

        let ray_data: Vec<RayIn> = rays
            .iter()
            .map(|(ro, rd)| RayIn {
                ro: ro.to_array(),
                _p0: 0.0,
                rd: rd.to_array(),
                _p1: 0.0,
            })
            .collect();
        let rays_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rt-rays"),
            contents: bytemuck::cast_slice(&ray_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let results_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rt-ray-results"),
            size: (rays.len() * std::mem::size_of::<RayOut>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rt-query-bg"),
            layout: &self.query_group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: scene.side_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: rays_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: results_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: scene.fine_side_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: scene.fine_uniform_buffer.as_entire_binding(),
                },
            ],
        });
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rt-ray-staging"),
            size: (rays.len() * std::mem::size_of::<RayOut>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rt-query-encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("rt-query-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.query_pipeline);
            pass.set_bind_group(0, &scene.manager.contree_bind_group, &[]);
            pass.set_bind_group(1, &bind_group, &[]);
            pass.set_bind_group(2, &scene.fine_manager.contree_bind_group, &[]);
            pass.dispatch_workgroups((rays.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &results_buffer,
            0,
            &staging,
            0,
            (rays.len() * std::mem::size_of::<RayOut>()) as u64,
        );
        queue.submit([encoder.finish()]);
        device.poll(wgpu::Maintain::Wait);
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv().expect("map channel").expect("map results");
        let data = slice.get_mapped_range();
        let out: Vec<RayOut> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();
        out
    }
}
