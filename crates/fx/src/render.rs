//! GPU rendering for Phase 2b: base box layer + instanced field
//! volumes + the HD-2D post pass, rendered offscreen headless-style
//! (the `crates/engine/src/capture.rs` pattern: `compatible_surface:
//! None`, offscreen `Rgba8UnormSrgb` target, staging-buffer readback).
//!
//! HARD RULE (Joey / gate 2b.2): the field pass binds NO texture at
//! all — no sprite texture, no sprite quad. A field volume's fragment
//! color comes only from its extracted parameters (uniform/storage
//! data) and procedural fbm noise in [`FIELD_SHADER`]. The only
//! textures in this module are the render targets themselves and the
//! post pass's input frame.

use std::fmt;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

use crate::density::effective_density;
use crate::effect::EffectDef;
use crate::post::PostConfig;
use crate::scene::{BoxPrim, PlacedField};

pub const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.10,
    g: 0.12,
    b: 0.17,
    a: 1.0,
};

#[derive(Debug)]
pub enum FxError {
    NoAdapter,
    Device(String),
    Readback(String),
    Io(std::io::Error),
}

impl fmt::Display for FxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxError::NoAdapter => write!(f, "fx-capture: no wgpu adapter available"),
            FxError::Device(e) => write!(f, "fx-capture: device error: {e}"),
            FxError::Readback(e) => write!(f, "fx-capture: readback error: {e}"),
            FxError::Io(e) => write!(f, "fx-capture: io error: {e}"),
        }
    }
}
impl std::error::Error for FxError {}
impl From<std::io::Error> for FxError {
    fn from(e: std::io::Error) -> Self {
        FxError::Io(e)
    }
}

/// Orbit camera with the engine's math (same eye/projection formulas
/// as `crates/engine/src/camera.rs`, re-implemented here because fx
/// may not depend on engine).
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub target: Vec3,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            target: Vec3::new(7.0, 0.9, -4.5),
            // From the south side: the bright backdrop wall (game
            // y = 0) sits BEHIND the field row, as intended.
            yaw_deg: 180.0,
            pitch_deg: 42.0,
            distance: 17.0,
        }
    }
}

impl Camera {
    pub fn eye(&self) -> Vec3 {
        let yaw = self.yaw_deg.to_radians();
        let pitch = self.pitch_deg.clamp(5.0, 85.0).to_radians();
        let offset = Vec3::new(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        ) * self.distance;
        self.target + offset
    }
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let proj = Mat4::perspective_rh(55f32.to_radians(), aspect.max(0.01), 0.1, 600.0);
        proj * Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }
    /// Projected (wgpu 0..1) depth of the follow target — the post
    /// stack's focal depth in GPU depth space.
    pub fn focal_projected_depth(&self) -> f32 {
        let (near, far) = (0.1f32, 600.0f32);
        let d = self.distance.max(near + 0.001);
        far * (d - near) / ((far - near) * d)
    }
}

// ---------------------------------------------------------------------
// Base box layer
// ---------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

const BASE_SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;
// Same layout as the field pass's instance buffer (bound here too):
// flame instances double as point lights — in the reference look,
// fire is the scene's light source, pooling warm light on the
// ground and washing nearby walls.
struct FieldInst {
    center: vec3<f32>, size: f32,
    base_color: vec3<f32>, density: f32,
    emissive: vec3<f32>, emissive_strength: f32,
    noise_scale: f32, noise_speed: f32, class_id: u32, seed: f32,
    drift: vec3<f32>, heat: f32,
    link: f32, _pad0: f32, _pad1: f32, _pad2: f32,
};
@group(0) @binding(1) var<storage, read> fields: array<FieldInst>;
struct VsIn { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec4<f32> };
struct VsOut { @builtin(position) clip: vec4<f32>, @location(0) normal: vec3<f32>, @location(1) color: vec4<f32>, @location(2) world: vec3<f32> };
@vertex fn vs_main(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(input.position, 1.0);
    out.normal = input.normal;
    out.color = input.color;
    out.world = input.position;
    return out;
}

@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(input.normal);
    let l = normalize(vec3<f32>(0.35, 0.8, 0.45));
    let shade = 0.45 + 0.55 * max(dot(n, l), 0.0);
    var col = input.color.rgb * shade;
    // Flame light: warm pool with distance falloff, stronger on
    // surfaces facing the flame, gently flickering.
    let count = arrayLength(&fields);
    for (var i = 0u; i < count; i++) {
        let f = fields[i];
        if (f.class_id != 1u) { continue; }
        let lp = f.center + vec3<f32>(0.0, 0.35, 0.0);
        let to_l = lp - input.world;
        let dist = length(to_l);
        let radius = 4.2 * f.size;
        let atten = pow(clamp(1.0 - dist / radius, 0.0, 1.0), 2.0);
        if (atten <= 0.0) { continue; }
        let facing = 0.45 + 0.55 * max(dot(n, to_l / max(dist, 1e-4)), 0.0);
        let flick = 0.92 + 0.13 * sin(u.time * 11.0 + f.seed * 17.0) * sin(u.time * 5.3 + f.seed * 7.0);
        col += f.emissive * atten * facing * flick * 0.85 * f.density;
    }
    return vec4<f32>(col, input.color.a);
}
"#;

/// Expand boxes into shaded cube triangles (CPU-side, shared by the
/// harness only — the real engine feeds its mesher output instead).
fn boxes_to_vertices(boxes: &[BoxPrim]) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    // (normal, four corners CCW seen from outside)
    for b in boxes {
        let (x0, y0, z0) = (b.min.x, b.min.y, b.min.z);
        let (x1, y1, z1) = (b.max.x, b.max.y, b.max.z);
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([0.0, 1.0, 0.0], [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]]), // top
            ([0.0, -1.0, 0.0], [[x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]]), // bottom
            ([0.0, 0.0, 1.0], [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]]), // south (+z)
            ([0.0, 0.0, -1.0], [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]]), // north
            ([1.0, 0.0, 0.0], [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]]), // east
            ([-1.0, 0.0, 0.0], [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]]), // west
        ];
        for (normal, corners) in faces {
            let base = verts.len() as u32;
            for c in corners {
                verts.push(Vertex {
                    position: c,
                    normal,
                    color: [b.color[0], b.color[1], b.color[2], 1.0],
                });
            }
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    (verts, idx)
}

// ---------------------------------------------------------------------
// Field volumes — procedural shader only, NO texture bindings.
// ---------------------------------------------------------------------

/// The field-volume shader. Gate 2b.2 asserts this source contains no
/// texture/sample of any sprite: volumes are pure math over the
/// per-instance extracted parameters.
pub const FIELD_SHADER: &str = r#"
struct Uniforms { view_proj: mat4x4<f32>, eye: vec3<f32>, time: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;

struct FieldInst {
    center: vec3<f32>, size: f32,
    base_color: vec3<f32>, density: f32,
    emissive: vec3<f32>, emissive_strength: f32,
    noise_scale: f32, noise_speed: f32, class_id: u32, seed: f32,
    drift: vec3<f32>, heat: f32,
    link: f32, _pad0: f32, _pad1: f32, _pad2: f32,
};
@group(0) @binding(1) var<storage, read> fields: array<FieldInst>;

// Unit-cube corners for 36 non-indexed vertices (12 triangles).
const CORNERS: array<vec3<f32>, 36> = array<vec3<f32>, 36>(
    // north (z=0)
    vec3<f32>(0.,0.,0.), vec3<f32>(1.,0.,0.), vec3<f32>(1.,1.,0.),
    vec3<f32>(0.,0.,0.), vec3<f32>(1.,1.,0.), vec3<f32>(0.,1.,0.),
    // south (z=1)
    vec3<f32>(1.,0.,1.), vec3<f32>(0.,0.,1.), vec3<f32>(0.,1.,1.),
    vec3<f32>(1.,0.,1.), vec3<f32>(0.,1.,1.), vec3<f32>(1.,1.,1.),
    // west (x=0)
    vec3<f32>(0.,0.,1.), vec3<f32>(0.,0.,0.), vec3<f32>(0.,1.,0.),
    vec3<f32>(0.,0.,1.), vec3<f32>(0.,1.,0.), vec3<f32>(0.,1.,1.),
    // east (x=1)
    vec3<f32>(1.,0.,0.), vec3<f32>(1.,0.,1.), vec3<f32>(1.,1.,1.),
    vec3<f32>(1.,0.,0.), vec3<f32>(1.,1.,1.), vec3<f32>(1.,1.,0.),
    // bottom (y=0)
    vec3<f32>(0.,0.,1.), vec3<f32>(1.,0.,1.), vec3<f32>(1.,0.,0.),
    vec3<f32>(0.,0.,1.), vec3<f32>(1.,0.,0.), vec3<f32>(0.,0.,0.),
    // top (y=1)
    vec3<f32>(0.,1.,0.), vec3<f32>(1.,1.,0.), vec3<f32>(1.,1.,1.),
    vec3<f32>(0.,1.,0.), vec3<f32>(1.,1.,1.), vec3<f32>(0.,1.,1.)
);

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec3<f32>,
    @location(1) @interpolate(flat) inst: u32,
    @location(2) world: vec3<f32>,
};

@vertex fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let f = fields[ii];
    let corner = CORNERS[vi % 36u];
    var scale = vec3<f32>(f.size, f.size, f.size);
    if (f.class_id == 2u || f.class_id == 3u) { scale.y = f.size * 0.55; } // gas/liquid stay low
    if (f.class_id == 1u) { scale = vec3<f32>(f.size * 1.3, f.size * 1.7, f.size * 1.3); } // flame: wide billowing base, tall
    if (f.class_id == 0u) { scale = vec3<f32>(f.size * 1.25, f.size * 1.35, f.size * 1.25); } // smoke billows
    let world = f.center + (corner - vec3<f32>(0.5, 0.0, 0.5)) * scale;
    var out: VsOut;
    out.clip = u.view_proj * vec4<f32>(world, 1.0);
    out.local = corner;
    out.inst = ii;
    out.world = world;
    return out;
}

fn hash3(p: vec3<i32>) -> f32 {
    var n = u32(p.x + 4096) * 374761393u ^ u32(p.y + 4096) * 668265263u ^ u32(p.z + 4096) * 1274126177u;
    n = (n ^ (n >> 13u)) * 1274126177u;
    n = n ^ (n >> 16u);
    return f32(n & 1023u) / 1023.0;
}
fn vnoise(p: vec3<f32>) -> f32 {
    let base = floor(p);
    let fr = p - base;
    let s = fr * fr * (3.0 - 2.0 * fr);
    let b = vec3<i32>(base);
    let c000 = hash3(b);
    let c100 = hash3(b + vec3<i32>(1, 0, 0));
    let c010 = hash3(b + vec3<i32>(0, 1, 0));
    let c110 = hash3(b + vec3<i32>(1, 1, 0));
    let c001 = hash3(b + vec3<i32>(0, 0, 1));
    let c101 = hash3(b + vec3<i32>(1, 0, 1));
    let c011 = hash3(b + vec3<i32>(0, 1, 1));
    let c111 = hash3(b + vec3<i32>(1, 1, 1));
    return mix(mix(mix(c000, c100, s.x), mix(c010, c110, s.x), s.y),
               mix(mix(c001, c101, s.x), mix(c011, c111, s.x), s.y), s.z);
}
fn fbm(p: vec3<f32>) -> f32 {
    var sum = 0.0; var amp = 0.5; var fr = 1.0;
    for (var i = 0; i < 4; i++) { sum += amp * vnoise(p * fr); fr *= 2.0; amp *= 0.5; }
    return sum / 0.9375;
}
fn vhash3(p: vec3<i32>) -> vec3<f32> {
    var n = u32(p.x + 4096) * 374761393u ^ u32(p.y + 4096) * 668265263u ^ u32(p.z + 4096) * 1274126177u;
    n = (n ^ (n >> 13u)) * 1274126177u;
    n = n ^ (n >> 16u);
    var m = n ^ (n >> 7u);
    m = (m ^ (m >> 11u)) * 2246822519u;
    var k = n ^ (n >> 19u);
    k = (k ^ (k >> 5u)) * 3266489917u;
    return vec3<f32>(f32(n & 1023u) / 1023.0, f32(m & 1023u) / 1023.0, f32(k & 1023u) / 1023.0);
}
// Cellular (voronoi) F1: distance to the nearest jittered cell
// point. 1-F1 gives rounded puff blobs — the core of the stylized
// smoke look (see the smoke branch in fs_main).
fn voronoi_f1(p: vec3<f32>) -> f32 {
    let base = floor(p);
    var f1 = 8.0;
    for (var dz = -1; dz <= 1; dz++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let cell = base + vec3<f32>(f32(dx), f32(dy), f32(dz));
                let pt = cell + vhash3(vec3<i32>(cell));
                f1 = min(f1, length(pt - p));
            }
        }
    }
    return f1;
}

// SMOKE shading (redone 2026-11-15, Joey's pick: technique after
// Loop-Box's "Stylized Smoke Shader", godotshaders.com, CC0 —
// voronoi noise, alpha curve, color gradient maps; here all three
// are procedural math, since the field pass binds no sprite data
// at all, gate 2b.2). Marched: 4 steps along the view chord,
// front-to-back — a single mid-chord sample read as a marbled
// glass slab with hard box edges; integration makes silhouette
// chords (short) fade and interior structure vary along the ray.
fn smoke_shade(f: FieldInst, p: vec3<f32>, dir_l: vec3<f32>, t0: f32, t1: f32) -> vec4<f32> {
    let steps = 4;
    let step_len = max((t1 - t0) / f32(steps), 0.0);
    var acc = vec3<f32>(0.0);
    var acc_a = 0.0;
    for (var i = 0; i < steps; i++) {
        let ts = mix(t0, t1, (f32(i) + 0.5) / f32(steps));
        let lp = clamp(p + dir_l * ts, vec3<f32>(0.0), vec3<f32>(1.0));
        // Voronoi puffs; cells grow as smoke rises and expands.
        let billow = 1.15 - 0.35 * lp.y;
        var sp = vec3<f32>(lp.x, lp.y * 0.8, lp.z) * (f.noise_scale * billow * 1.6) + vec3<f32>(f.seed);
        // Heat coupling (f.heat: fire on/below this tile): hot
        // smoke rises faster, churns at its base, is lit from below.
        sp.y -= u.time * f.noise_speed * 1.4 * (1.0 + f.heat * 1.1);
        sp.x += u.time * f.noise_speed * 0.22; // slight wind lean
        let v1 = 1.0 - voronoi_f1(sp);
        let v2 = 1.0 - voronoi_f1(sp * 2.13 + vec3<f32>(7.7));
        var v = clamp(0.72 * v1 + 0.28 * v2, 0.0, 1.0);
        // The fire erodes the smoke's base.
        v = clamp(v - f.heat * (1.0 - lp.y) * 0.30 * (1.0 - v2), 0.0, 1.0);
        let curve = smoothstep(0.18, 0.9, v); // the alpha-curve remap
        // Billow silhouette, LINK-AWARE (f.link: smoke below/above
        // in the column): unlinked bases pinch and unlinked tops
        // round off, linked faces stay full — a column of smoke
        // tiles shades as one plume, not stacked blobs.
        let taper = 1.0 + 0.5 * lp.y;
        let cx = (lp.x - 0.5) * 2.0 / taper;
        let cz = (lp.z - 0.5) * 2.0 / taper;
        let radial = length(vec2<f32>(cx, cz));
        let has_below = (f.link == 1.0 || f.link == 3.0);
        let has_above = (f.link == 2.0 || f.link == 3.0);
        var base_f = 0.35 + 0.65 * smoothstep(0.0, 0.3, lp.y);
        if (has_below) { base_f = 1.0; }
        var top_f = 1.0 - 0.5 * smoothstep(0.7, 1.0, lp.y);
        if (has_above) { top_f = 1.0; }
        let shape = clamp(1.2 - radial, 0.0, 1.0) * base_f * top_f;
        var sdens = f.density * (0.25 + 0.75 * curve) * shape;
        // Hot smoke is THIN at the base (still mostly flame and
        // hot gas there) and builds as it climbs.
        sdens *= mix(1.0, 0.25 + 0.75 * smoothstep(0.05, 0.6, lp.y), f.heat);
        let a = clamp(1.0 - exp(-sdens * 1.9 * step_len), 0.0, 1.0);
        if (a > 0.001) {
            // Flat color gradient, dark base to light top; heat
            // darkens fresh smoke and underlights it orange.
            let g = clamp(curve * (0.55 + 0.45 * lp.y), 0.0, 1.0);
            var dark = f.base_color * vec3<f32>(0.10, 0.095, 0.095);
            dark *= 1.0 - 0.5 * f.heat * pow(1.0 - lp.y, 1.5);
            let light = f.base_color * 0.52;
            var scol = mix(dark, light, g);
            let uflick = 0.6 + 0.4 * vnoise(vec3<f32>(f.seed * 5.0, u.time * 4.0, 3.0));
            scol += vec3<f32>(1.0, 0.42, 0.10) * (f.heat * pow(1.0 - lp.y, 2.0) * 1.15 * uflick);
            acc += (1.0 - acc_a) * a * scol;
            acc_a += (1.0 - acc_a) * a;
            if (acc_a > 0.98) { break; }
        }
    }
    if (acc_a < 0.03) { return vec4<f32>(0.0); } // stylized cutout edge
    return vec4<f32>(acc / acc_a, acc_a);
}

@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let f = fields[input.inst];
    // Chord evaluation: intersect the view ray with the unit volume,
    // sample the density field at the chord midpoint, and scale by
    // chord length — long chords through the middle are dense, short
    // silhouette chords fade out. (Evaluating a radial falloff on the
    // cube SURFACE would be zero everywhere; the volume is the chord.)
    let view_dir = normalize(input.world - u.eye);
    var scale = vec3<f32>(f.size, f.size, f.size);
    if (f.class_id == 2u || f.class_id == 3u) { scale.y = f.size * 0.55; }
    if (f.class_id == 1u) { scale = vec3<f32>(f.size * 1.3, f.size * 1.7, f.size * 1.3); }
    if (f.class_id == 0u) { scale = vec3<f32>(f.size * 1.25, f.size * 1.35, f.size * 1.25); }
    let dir_l = view_dir / scale;
    let p = input.local;
    var t_enter = -1e9;
    var t_exit = 1e9;
    for (var a = 0; a < 3; a++) {
        let d = dir_l[a];
        let o = p[a];
        if (abs(d) > 1e-6) {
            var ta = (0.0 - o) / d;
            var tb = (1.0 - o) / d;
            if (ta > tb) { let tmp = ta; ta = tb; tb = tmp; }
            t_enter = max(t_enter, ta);
            t_exit = min(t_exit, tb);
        }
    }
    let chord = clamp(t_exit - t_enter, 0.0, 1.8);
    let local = clamp(p + dir_l * ((t_enter + t_exit) * 0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    if (f.class_id == 1u) {
        // FLAME (redone 2026-11-15 against Joey's HD-2D reference
        // screenshots): a tapered column of rising tongues with a
        // heat ramp — near-white core at the base center, through
        // saturated orange, to a deep red rim — plus rising embers.
        // NOT the generic soft-volume path: fire is mostly opaque
        // and is the brightest thing in the scene.
        let taper = 1.0 - 0.38 * local.y;
        let cx = (local.x - 0.5) * 2.0 / max(taper, 0.2);
        let cz = (local.z - 0.5) * 2.0 / max(taper, 0.2);
        let radial = length(vec2<f32>(cx, cz));
        let rise = u.time * (0.9 + 0.6 * local.y) * f.noise_speed;
        let sway = 0.18 * local.y * local.y * sin(u.time * 1.7 + f.seed);
        let sp = vec3<f32>(local.x * f.noise_scale + sway, local.y * f.noise_scale * 0.8 - rise, local.z * f.noise_scale) + vec3<f32>(f.seed);
        let fnoise = fbm(sp);
        let flick = 0.85 + 0.35 * vnoise(vec3<f32>(f.seed * 3.0, u.time * 3.5, 7.0));
        let thresh = 0.24 + 0.30 * local.y;
        let body = clamp((fnoise - thresh) * 2.4, 0.0, 1.0);
        let shape = clamp(1.15 - radial, 0.0, 1.0);
        var fdens = f.density * body * shape * flick * 2.0;
        fdens *= 1.0 - smoothstep(0.75, 1.0, local.y) * 0.7; // tips fade out, never black caps
        let heat = clamp((1.0 - local.y * 1.15) * (1.25 - radial * 0.6) * (0.55 + 0.65 * fnoise) * flick * (0.7 + 0.5 * f.density), 0.0, 1.3);
        // Ramp anchored on the extracted colors (see density.rs
        // flame_color, the CPU reference of this same ramp).
        let s = max(f.emissive_strength, 0.6);
        let core = mix(f.emissive, vec3<f32>(1.0, 0.97, 0.88), 0.55) * s;
        let mid = f.emissive * (s * 0.85);
        let edge = f.base_color * vec3<f32>(1.5, 1.1, 0.85);
        let rim = f.base_color * vec3<f32>(0.85, 0.30, 0.16);
        var fcol = rim;
        fcol = mix(fcol, edge, smoothstep(0.08, 0.30, heat));
        fcol = mix(fcol, mid, smoothstep(0.42, 0.68, heat));
        fcol = mix(fcol, core, smoothstep(0.72, 0.98, heat));
        fcol *= (0.85 + 0.5 * heat);
        // Embers: sparse bright specks racing upward.
        let cell = vec3<f32>(local.x * 26.0, local.y * 9.0 - u.time * 2.6, local.z * 26.0) + vec3<f32>(f.seed * 7.0);
        let spark = vnoise(cell);
        let ember = step(0.93, spark) * clamp(heat + 0.25 - local.y * 0.4, 0.0, 1.0);
        fcol += vec3<f32>(1.0, 0.55, 0.15) * ember * 2.4;
        let falpha = clamp(fdens * 3.0 + ember * 0.8, 0.0, 1.0);
        if (falpha < 0.004) { return vec4<f32>(0.0); }
        return vec4<f32>(fcol, falpha);
    }
    // Animated fbm: time-based only, scrolled by drift/buoyancy.
    let sample = local * f.noise_scale - f.drift * (u.time * f.noise_speed) + vec3<f32>(f.seed);
    let n = fbm(sample);
    let shaped = clamp(n * 1.4 - 0.25, 0.0, 1.0);
    // Soft falloff: chord length + interior radial falloff at the
    // sample point (volume, not a quad).
    let c = (local - vec3<f32>(0.5)) * 2.0;
    let d = length(c);
    let t = clamp((1.15 - d) / 0.9, 0.0, 1.0);
    let fall = (0.25 + 0.75 * (t * t * (3.0 - 2.0 * t))) * clamp(chord, 0.0, 1.0);
    if (f.class_id == 0u) {
        return smoke_shade(f, p, dir_l, t_enter, t_exit);
    }
    var dens = f.density * fall * (0.45 + 0.55 * shaped);
    var emis = vec3<f32>(0.0);
    // (class 1, flame, returned early above — it has its own path.)
    if (f.class_id == 2u || f.class_id == 3u) { // gas/liquid stay low
        dens *= 1.15 - 0.75 * local.y;
    } else if (f.class_id == 4u) { // energy shimmer
        let shimmer = 0.9 + 0.2 * vnoise(vec3<f32>(1.0, u.time * 5.0, f.seed));
        emis = f.emissive * (f.emissive_strength * shimmer);
    }
    let alpha = clamp(1.0 - exp(-dens * 4.5), 0.0, 1.0);
    let col = f.base_color * (0.72 + 0.38 * n) + emis;
    return vec4<f32>(col, alpha);
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 3],
    time: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FieldInstGpu {
    center: [f32; 3],
    size: f32,
    base_color: [f32; 3],
    density: f32,
    emissive: [f32; 3],
    emissive_strength: f32,
    noise_scale: f32,
    noise_speed: f32,
    class_id: u32,
    seed: f32,
    drift: [f32; 3],
    /// Fire coupling: for smoke, how much fire burns on/below its
    /// tile (0..1); 0 for everything else. Computed at upload.
    heat: f32,
    /// Smoke column links: 1 = smoke directly below, 2 = smoke
    /// directly above, 3 = both, 0 = neither/other classes. Lets a
    /// vertical smoke column shade as one plume, not stacked blobs.
    link: f32,
    _pad: [f32; 3],
}

/// Heat coupling for one placed field: smoke sitting on or above
/// fire is heated by it (same tile counts full, each z-level above
/// the fire counts less). Everything that is not smoke gets 0.
fn heat_of(f: &PlacedField, all: &[PlacedField]) -> f32 {
    use crate::effect::RenderClass;
    if f.def.render_class != RenderClass::Smoke {
        return 0.0;
    }
    let mut heat = 0.0f32;
    for o in all {
        if o.def.render_class == RenderClass::Flame
            && o.tile[0] == f.tile[0]
            && o.tile[1] == f.tile[1]
            && o.tile[2] <= f.tile[2]
        {
            let dz = (f.tile[2] - o.tile[2]) as f32;
            let h = (o.intensity as f32 / 3.0) * (1.0 - 0.35 * dz).max(0.0);
            heat = heat.max(h);
        }
    }
    heat.clamp(0.0, 1.0)
}

/// Column links for one placed field (see FieldInstGpu::link).
fn link_of(f: &PlacedField, all: &[PlacedField]) -> f32 {
    use crate::effect::RenderClass;
    if f.def.render_class != RenderClass::Smoke {
        return 0.0;
    }
    let has = |dz: i32| {
        all.iter().any(|o| {
            o.def.render_class == RenderClass::Smoke
                && o.tile[0] == f.tile[0]
                && o.tile[1] == f.tile[1]
                && o.tile[2] == f.tile[2] + dz
        })
    };
    (has(-1) as u32 + 2 * has(1) as u32) as f32
}

fn field_instance(f: &PlacedField, heat: f32, link: f32) -> FieldInstGpu {
    let def: &EffectDef = &f.def;
    FieldInstGpu {
        center: f.world_center().to_array(),
        size: 1.12,
        base_color: def.base_color,
        density: effective_density(def, f.intensity),
        emissive: def.emissive_color,
        emissive_strength: def.emissive_strength,
        noise_scale: def.noise_scale,
        noise_speed: def.noise_speed,
        class_id: def.render_class.id(),
        seed: def.seed(),
        drift: def.world_drift().to_array(),
        heat,
        link,
        _pad: [0.0; 3],
    }
}

// ---------------------------------------------------------------------
// Post pass (fullscreen; mirrors post.rs CPU reference)
// ---------------------------------------------------------------------

const POST_SHADER: &str = r#"
struct PostUniforms {
    flags: vec4<u32>,          // bloom, dof, grade, chromatic
    params: vec4<f32>,         // threshold, bloom_strength, focal_depth(projected), dof_strength
    texel: vec2<f32>, _p0: f32, _p1: f32,
};
@group(0) @binding(0) var<uniform> u: PostUniforms;
@group(0) @binding(1) var t_color: texture_2d<f32>;
@group(0) @binding(2) var t_depth: texture_depth_2d;

struct VsOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi) - 1) * 2.0;
    let y = f32(i32(vi & 1u)) * 4.0 - 1.0;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}
fn load_px(c: vec2<i32>) -> vec4<f32> {
    let dims = vec2<i32>(textureDimensions(t_color));
    return textureLoad(t_color, clamp(c, vec2<i32>(0), dims - vec2<i32>(1)), 0);
}
fn lum(c: vec3<f32>) -> f32 { return dot(c, vec3<f32>(0.299, 0.587, 0.114)); }

@fragment fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let px = vec2<i32>(input.pos.xy);
    let src = load_px(px);
    if (all(u.flags == vec4<u32>(0u))) { return src; } // all-off: exact input
    var col = src.rgb;
    let dims = vec2<f32>(textureDimensions(t_color));
    if (u.flags.x == 1u) { // bloom: additive blurred bright-pass
        var glow = vec3<f32>(0.0);
        for (var dy = -2; dy <= 2; dy++) {
            for (var dx = -2; dx <= 2; dx++) {
                let s = load_px(px + vec2<i32>(dx * 2, dy * 2)).rgb;
                let over = max(lum(s) - u.params.x, 0.0);
                glow += s * (over / max(lum(s), 1e-4));
            }
        }
        col += glow / 25.0 * u.params.y;
    }
    if (u.flags.y == 1u) { // tilt-shift DoF in projected-depth space
        let depth = textureLoad(t_depth, px, 0);
        let tilt = 1.0 + 0.6 * abs(input.pos.y / max(dims.y, 1.0) - 0.5) * 2.0;
        let amt = clamp((abs(depth - u.params.z) - 0.0012) * 260.0 * u.params.w * tilt, 0.0, 1.0);
        if (amt > 0.0) {
            var blur = vec3<f32>(0.0);
            for (var dy = -1; dy <= 1; dy++) {
                for (var dx = -1; dx <= 1; dx++) {
                    blur += load_px(px + vec2<i32>(dx * 3, dy * 3)).rgb;
                }
            }
            col = mix(col, blur / 9.0, amt);
        }
    }
    if (u.flags.w == 1u) { // chromatic offset, center stays clean
        let fx = abs(input.pos.x / max(dims.x, 1.0) - 0.5) * 2.0;
        if (fx > 0.2) {
            col.r = load_px(px + vec2<i32>(1, 0)).r;
            col.b = load_px(px - vec2<i32>(1, 0)).b;
        }
    }
    if (u.flags.z == 1u) { // warm grade + vignette
        let ndc = input.pos.xy / dims * 2.0 - vec2<f32>(1.0);
        let vig = 1.0 - 0.32 * min(dot(ndc, ndc) / 2.0, 1.0);
        col = clamp(col * vec3<f32>(1.07, 1.01, 0.93) * vig, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    return vec4<f32>(col, src.a);
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PostUniforms {
    flags: [u32; 4],
    params: [f32; 4],
    texel: [f32; 2],
    _p: [f32; 2],
}

// ---------------------------------------------------------------------
// Capture entry point
// ---------------------------------------------------------------------

pub struct CaptureRequest<'a> {
    pub boxes: &'a [BoxPrim],
    pub fields: &'a [PlacedField],
    pub camera: Camera,
    pub size: (u32, u32),
    pub post: PostConfig,
    pub time: f32,
    /// Optional clear color override (e.g. near-black for the night
    /// fire showcase); `None` uses the default `CLEAR_COLOR`.
    pub clear: Option<[f32; 3]>,
}

/// Render one frame offscreen and return it as an image. Fails with
/// the named `FxError::NoAdapter` when no adapter exists — never a
/// blank-PNG pass.
pub fn render_frame(req: &CaptureRequest) -> Result<image::RgbaImage, FxError> {
    pollster::block_on(render_frame_async(req))
}

async fn render_frame_async(req: &CaptureRequest<'_>) -> Result<image::RgbaImage, FxError> {
    use wgpu::util::DeviceExt;
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or(FxError::NoAdapter)?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("fx-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .map_err(|e| FxError::Device(e.to_string()))?;

    let (width, height) = (req.size.0.max(1), req.size.1.max(1));
    let aspect = width as f32 / height as f32;
    let uniforms = Uniforms {
        view_proj: req.camera.view_projection(aspect).to_cols_array_2d(),
        eye: req.camera.eye().to_array(),
        time: req.time,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fx-uniforms"),
        contents: bytemuck::bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // Field instances are built BEFORE the base pass: the base
    // shader binds the same buffer to light the scene from flames.
    // Sort back-to-front for correct alpha compositing.
    let eye = req.camera.eye();
    let mut placed: Vec<&PlacedField> = req.fields.iter().collect();
    placed.sort_by(|a, b| {
        b.world_center()
            .distance_squared(eye)
            .partial_cmp(&a.world_center().distance_squared(eye))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut instances: Vec<FieldInstGpu> = placed
        .iter()
        .map(|f| {
            let heat = heat_of(f, req.fields);
            let link = link_of(f, req.fields);
            field_instance(f, heat, link)
        })
        .collect();
    let real_instances = instances.len();
    if instances.is_empty() {
        // wgpu rejects zero-sized buffers; bind one dummy instance
        // and simply never draw it.
        instances.push(FieldInstGpu {
            center: [0.0; 3],
            size: 0.0,
            base_color: [0.0; 3],
            density: 0.0,
            emissive: [0.0; 3],
            emissive_strength: 0.0,
            noise_scale: 1.0,
            noise_speed: 0.0,
            class_id: 5,
            seed: 0.0,
            drift: [0.0; 3],
            heat: 0.0,
            link: 0.0,
            _pad: [0.0; 3],
        });
    }
    let inst_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fx-field-instances"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // --- base pipeline ---
    let base_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fx-base-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let base_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fx-base-bg"),
        layout: &base_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: inst_buffer.as_entire_binding(),
            },
        ],
    });
    let base_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fx-base-shader"),
        source: wgpu::ShaderSource::Wgsl(BASE_SHADER.into()),
    });
    let base_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fx-base-pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fx-base-pl"),
            bind_group_layouts: &[&base_layout],
            push_constant_ranges: &[],
        })),
        vertex: wgpu::VertexState {
            module: &base_shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &base_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: CAPTURE_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });
    let (verts, indices) = boxes_to_vertices(req.boxes);
    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fx-box-verts"),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fx-box-idx"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    // --- field pipeline (storage buffer of instances; no textures) ---
    let field_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fx-field-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let field_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fx-field-bg"),
        layout: &field_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: inst_buffer.as_entire_binding(),
            },
        ],
    });
    let field_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fx-field-shader"),
        source: wgpu::ShaderSource::Wgsl(FIELD_SHADER.into()),
    });
    let field_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fx-field-pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fx-field-pl"),
            bind_group_layouts: &[&field_layout],
            push_constant_ranges: &[],
        })),
        vertex: wgpu::VertexState {
            module: &field_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &field_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: CAPTURE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });

    // --- targets ---
    let color_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fx-color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let color_view = color_tex.create_view(&Default::default());
    let depth_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fx-depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let depth_view = depth_tex.create_view(&Default::default());
    let out_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fx-post-out"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let out_view = out_tex.create_view(&Default::default());

    // --- post pipeline ---
    let post_uniforms = PostUniforms {
        flags: [
            req.post.bloom as u32,
            req.post.dof as u32,
            req.post.grade as u32,
            req.post.chromatic as u32,
        ],
        params: [
            req.post.bloom_threshold,
            req.post.bloom_strength,
            req.camera.focal_projected_depth(),
            req.post.dof_strength,
        ],
        texel: [1.0 / width as f32, 1.0 / height as f32],
        _p: [0.0; 2],
    };
    let post_ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fx-post-uniforms"),
        contents: bytemuck::bytes_of(&post_uniforms),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fx-post-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });
    let post_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fx-post-bg"),
        layout: &post_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: post_ubuf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&color_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&depth_view),
            },
        ],
    });
    let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fx-post-shader"),
        source: wgpu::ShaderSource::Wgsl(POST_SHADER.into()),
    });
    let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fx-post-pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fx-post-pl"),
            bind_group_layouts: &[&post_layout],
            push_constant_ranges: &[],
        })),
        vertex: wgpu::VertexState {
            module: &post_shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &post_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: CAPTURE_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });

    // --- encode: world pass, then post pass (UI would composite
    // after this, in the engine — never inside the stack) ---
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("fx-frame"),
    });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fx-world-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(match req.clear {
                        Some(c) => wgpu::Color {
                            r: c[0] as f64,
                            g: c[1] as f64,
                            b: c[2] as f64,
                            a: 1.0,
                        },
                        None => CLEAR_COLOR,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&base_pipeline);
        rpass.set_bind_group(0, &base_bg, &[]);
        rpass.set_vertex_buffer(0, vbuf.slice(..));
        rpass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        if real_instances > 0 {
            rpass.set_pipeline(&field_pipeline);
            rpass.set_bind_group(0, &field_bg, &[]);
            rpass.draw(0..36, 0..real_instances as u32);
        }
    }
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fx-post-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&post_pipeline);
        rpass.set_bind_group(0, &post_bg, &[]);
        rpass.draw(0..3, 0..1);
    }

    // --- readback ---
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fx-staging"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &out_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| FxError::Readback("map channel closed".into()))?
        .map_err(|e| FxError::Readback(e.to_string()))?;
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..height as usize {
            let start = row * padded as usize;
            pixels.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| FxError::Readback("pixel buffer size mismatch".into()))
}
