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

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use Sparse_Voxel_64Tree::Tree64GpuManager;
use wgpu::util::DeviceExt;

use crate::build::BuiltTree;
use crate::cpu::TraceMode;

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
}
@group(0) @binding(0) var<storage, read> nodes: array<GpuNode>;
@group(1) @binding(0) var<storage, read> side: array<SideEntry>;
@group(1) @binding(1) var<uniform> u: Uniforms;

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

fn is_leaf(idx: u32) -> bool {
    return (nodes[idx].child_ptr_and_leaf & 0x80000000u) != 0u;
}

fn child_ptr(idx: u32) -> u32 {
    return nodes[idx].child_ptr_and_leaf & 0x7fffffffu;
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
}

fn trace(ro: vec3<f32>, rd: vec3<f32>, skip_transparent: bool) -> Hit {
    var result = Hit(false, 0u, vec3<f32>(0.0, 0.0, 0.0), 0.0, vec3<f32>(0.0, 0.0, 0.0));
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
            result = Hit(true, d.node, d.cell_min, dist, entry_normal(ro, rd, d.cell_min, 1.0, hs.x));
            return result;
        }
        t = advance(ro, rd, d.cell_min, d.cell_size, t);
    }
    return result;
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
@group(1) @binding(2) var color_out: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(3) var<storage, read_write> depth_out: array<f32>;

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
    let node = nodes[h.node];
    let e = side[h.node];
    var albedo = node.color;
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
    let hitp = ro + rd * h.dist;
    var light = 0.35;
    let ndl = max(dot(n, u.sun_dir), 0.0);
    if ndl > 0.0 {
        var shadowed = false;
        if u.shadows != 0u {
            let sh = trace(hitp + n * 0.01, u.sun_dir, true);
            shadowed = sh.hit;
        }
        if !shadowed {
            light = light + 0.85 * ndl;
        }
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
    return albedo * (light * lit_factor(e.lit) * ao);
}

fn sky(rd: vec3<f32>) -> vec3<f32> {
    let horizon = vec3<f32>(0.08, 0.10, 0.14);
    let zenith = vec3<f32>(0.30, 0.48, 0.78);
    let ground = vec3<f32>(0.05, 0.06, 0.08);
    if rd.z > 0.0 {
        return mix(horizon, zenith, pow(clamp(rd.z, 0.0, 1.0), 0.6));
    }
    return mix(horizon, ground, clamp(-rd.z * 3.0, 0.0, 1.0));
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
    let first = trace(ro, rd, false);
    let opq = trace(ro, rd, true);
    var col = sky(rd);
    var depth = u.max_dist;
    if opq.hit {
        col = shade(ro, rd, opq);
        depth = opq.dist;
    }
    if first.hit && (side[first.node].flags & 1u) != 0u && (!opq.hit || first.dist < opq.dist) {
        let gcol = shade(ro, rd, first);
        let a = f32(side[first.node].alpha) / 255.0;
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
    let h = trace(r.ro, r.rd, u.mode != 0u);
    var out = RayOut(0u, 0u, 0.0, 0u, vec3<i32>(0, 0, 0), 0u, vec3<i32>(0, 0, 0), 0u);
    if h.hit {
        out.hit = 1u;
        out.node = h.node;
        out.dist = h.dist;
        out.voxel = vec3<i32>(h.voxel_min);
        out.normal = vec3<i32>(h.normal);
    }
    results[i] = out;
}
"#;

/// The full WGSL sources (traversal + entry points), exposed for
/// diagnostics and source-level test assertions.
pub fn wgsl_sources() -> (String, String) {
    (
        format!("{TRAVERSAL_WGSL}\n{RENDER_WGSL}"),
        format!("{TRAVERSAL_WGSL}\n{QUERY_WGSL}"),
    )
}

/// An uploaded tree: the crate manager (nodes) + our side-table.
pub struct RtScene {
    pub manager: Tree64GpuManager,
    side_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pub node_count: usize,
    pub depth: usize,
    pub size: u32,
    pub origin: [i32; 3],
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
    pub fn new(device: &wgpu::Device, manager_layout: &wgpu::BindGroupLayout) -> Self {
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
            ],
        });
        let render_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-render-shader"),
            source: wgpu::ShaderSource::Wgsl(format!("{TRAVERSAL_WGSL}\n{RENDER_WGSL}").into()),
        });
        let query_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-query-shader"),
            source: wgpu::ShaderSource::Wgsl(format!("{TRAVERSAL_WGSL}\n{QUERY_WGSL}").into()),
        });
        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt-render-layout"),
            bind_group_layouts: &[manager_layout, &render_group1_layout],
            push_constant_ranges: &[],
        });
        let query_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt-query-layout"),
            bind_group_layouts: &[manager_layout, &query_group1_layout],
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
    pub fn with_scene(device: &wgpu::Device, built: &BuiltTree) -> (Self, RtScene) {
        let manager = Tree64GpuManager::new(device, &built.tree);
        let renderer = Self::new(device, &manager.contree_bind_group_layout);
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
            node_count: built.node_count(),
            depth: built.depth,
            size: built.size,
            origin: built.origin,
            manager,
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
