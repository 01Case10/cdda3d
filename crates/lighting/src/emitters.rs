//! Addendum 11 (2026-11-23): local light emitters — the shared
//! emitter contract both renderers consume, mirroring the
//! [`crate::sky_uniforms`] pattern (one Rust uniform block + one
//! WGSL string, single source of truth, consumed verbatim by the
//! raster and RT renderers).
//!
//! The model (plan.md, "Addendum 2026-11-23 (11): local light
//! emitters — the combined model"): local sources — vehicle
//! headlights are the motivating case; lamps and flashlights have
//! the same shape — are NOT baked into the per-tile light grid
//! and NOT free renderer inventions. The game side *declares*
//! emitters as data (position, direction, cone angle, range,
//! intensity, colour); the renderer *transports* the light
//! (per-fragment/per-hit cone + distance falloff). The light grid
//! keeps its Phase 7 job — visibility truth and the ceiling rule:
//! emitter light is added INSIDE the renderers' existing ceiling
//! multiply, so it can never reveal a tile the game marks dark or
//! unseen, and MEMORIZED presentation is not bypassed.
//!
//! This module is the PROTOTYPE slice of that model: the capped
//! baseline (the amendment's "lid" option) is what gets measured.
//! The alternative — the fork excluding declared emitters from
//! the game's own light computation so the renderer does them
//! alone — is deliberately NOT implemented here; the prototype's
//! measurements decide that question (docs/addendum-11-light.md).
//!
//! ## Contents
//!
//! - [`Emitter`]: one declared source, authored in game space.
//! - [`EmitterUniforms`]: the packed uniform block (13 `vec4`
//!   rows: one header row + 3 rows per emitter, [`MAX_EMITTERS`]
//!   emitters), with the per-renderer space conversions.
//! - [`EMITTER_WGSL`]: the shared cone + distance falloff math.
//!   It contains NO shadow/visibility logic — visibility is
//!   supplied by the caller per emitter (the raster passes 1.0,
//!   v1's recorded asymmetry; the RT renderer passes a per-emitter
//!   shadow-ray result).
//! - [`emitter_light_cpu`]: a pure-Rust mirror of the WGSL sum,
//!   for CPU gates.
//!
//! ## Coordinate-space contract
//!
//! [`Emitter`] positions and directions are authored in **game
//! space**: +X east, +Y north, +Z up; positions in tiles
//! (x/y) and z-levels (z), range in tiles. Each renderer consumes
//! the uniforms in its own space, so the *caller* converts with
//! the helpers on [`EmitterUniforms`], exactly as with
//! [`crate::SkyUniforms`]:
//!
//! - the RT renderer is grid space (+X east, +Y south, +Z up,
//!   z in BLOCKS — `mesher::WorldScale::blocks_per_z_level` blocks
//!   per z-level) and takes [`EmitterUniforms::to_grid_space`];
//! - the raster renderer is mesher world space (+X east, +Y up,
//!   +Z north, y in world units) and takes
//!   [`EmitterUniforms::to_mesher_world_space`].
//!
//! Directions are converted with the same axis scale as positions
//! and re-normalized, so a beam's tilt survives the anisotropic
//! z scale. Range is carried unchanged: it is measured in the
//! target space's own units (1 tile = 1 block = 1 world unit
//! horizontally in both), which makes the falloff volume very
//! slightly ellipsoidal in game terms — vertical distances count
//! `blocks_per_z_level`× more. Headlight beams are near-horizontal,
//! so the prototype accepts this rather than complicating the
//! shared WGSL with a per-space range scale; recorded here as a
//! known prototype simplification.

use mesher::scale::WorldScale;

/// Maximum declared emitters per frame (the uniform block is
/// fixed-size; [`EmitterUniforms::from_emitters`] keeps the first
/// [`MAX_EMITTERS`] of a longer list).
pub const MAX_EMITTERS: usize = 4;

/// Rows in the [`EmitterUniforms`] block: 1 header row + 3 rows
/// per emitter. 13 rows × 16 B = 208 B.
pub const EMITTER_UNIFORM_ROWS: usize = 1 + 3 * MAX_EMITTERS;

/// Cone edge softness: the cone falloff is a smoothstep from
/// `cos_half_angle` (0 at the authored cone edge) up to an inner
/// cos this fraction of the way from the edge cos toward the axis
/// (cos = 1). 0.35 = the inner cone sits at 35% of the cos gap —
/// a soft penumbra, not a hard disc edge. Mirrored as a literal
/// in [`EMITTER_WGSL`]; keep the two in sync.
pub const CONE_INNER_FRACTION: f32 = 0.35;

/// One declared local light emitter, authored in game space
/// (module doc: +X east, +Y north, +Z up; position in tiles /
/// z-levels, range in tiles). `direction` is a unit vector
/// (toward where the light points); `cos_half_angle` is the
/// cosine of the cone's half-angle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Emitter {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub range: f32,
    pub cos_half_angle: f32,
    pub intensity: f32,
    pub color_rgb: [f32; 3],
}

impl Emitter {
    /// Build an emitter from a position, a (not necessarily unit)
    /// direction, and the cone half-angle in degrees.
    /// The direction is normalized here so fixtures can author
    /// tilts like `[1.0, 0.0, -0.12]` directly.
    pub fn new(
        position: [f32; 3],
        direction: [f32; 3],
        half_angle_deg: f32,
        range: f32,
        intensity: f32,
        color_rgb: [f32; 3],
    ) -> Self {
        Self {
            position,
            direction: normalize3(direction),
            range,
            cos_half_angle: half_angle_deg.to_radians().cos(),
            intensity,
            color_rgb,
        }
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        v
    }
}

// ---------------------------------------------------------------------------
// EmitterUniforms — packed layout (13 rows × vec4 = 208 bytes)
// ---------------------------------------------------------------------------

/// The shared emitter uniform block. Each row is one WGSL
/// `vec4<f32>`:
///
/// | row        | content                                  |
/// |------------|------------------------------------------|
/// | 0          | header: `.x` = emitter count, rest 0     |
/// | 1 + 3i     | emitter i: position `.xyz`, range `.w`   |
/// | 2 + 3i     | emitter i: direction `.xyz` (unit), `cos_half_angle` `.w` |
/// | 3 + 3i     | emitter i: colour `.xyz`, intensity `.w` |
///
/// The block is space-agnostic: it holds whatever space the last
/// conversion put it in (game space as built by
/// [`EmitterUniforms::from_emitters`]; grid or mesher-world after
/// the matching `to_*_space`). Like [`crate::SkyUniforms`], the
/// block cannot tell the caller which space it is in — the
/// conversion belongs to the caller, at the upload site.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterUniforms {
    pub rows: [[f32; 4]; EMITTER_UNIFORM_ROWS],
}

const _: () = {
    assert!(std::mem::size_of::<EmitterUniforms>() == EMITTER_UNIFORM_ROWS * 16);
};

impl EmitterUniforms {
    /// The empty set: count 0. Both renderers initialize their
    /// emitter buffer to this, and the shared WGSL returns exactly
    /// zero for it, so a frame with no emitters renders
    /// byte-identically to the pre-Addendum-11 frame (the
    /// prototype's empty-set invariant).
    pub fn empty() -> Self {
        Self {
            rows: [[0.0; 4]; EMITTER_UNIFORM_ROWS],
        }
    }

    /// Pack emitters (authored in game space — see the module
    /// doc) into the block. Keeps the first [`MAX_EMITTERS`];
    /// extra emitters are dropped (the prototype cap; the game
    /// side owns prioritization when this becomes wire data).
    pub fn from_emitters(emitters: &[Emitter]) -> Self {
        let mut u = Self::empty();
        let n = emitters.len().min(MAX_EMITTERS);
        u.rows[0] = [n as f32, 0.0, 0.0, 0.0];
        for (i, e) in emitters.iter().take(n).enumerate() {
            u.rows[1 + 3 * i] = [e.position[0], e.position[1], e.position[2], e.range];
            u.rows[2 + 3 * i] = [
                e.direction[0],
                e.direction[1],
                e.direction[2],
                e.cos_half_angle,
            ];
            u.rows[3 + 3 * i] = [e.color_rgb[0], e.color_rgb[1], e.color_rgb[2], e.intensity];
        }
        u
    }

    /// The rows in layout order (upload helper).
    pub fn as_rows(&self) -> [[f32; 4]; EMITTER_UNIFORM_ROWS] {
        self.rows
    }

    /// Number of live emitters in the block.
    pub fn count(&self) -> usize {
        self.rows[0][0] as usize
    }

    /// Unpack emitter `i` (in the block's current space).
    pub fn emitter(&self, i: usize) -> Option<Emitter> {
        if i >= self.count() || i >= MAX_EMITTERS {
            return None;
        }
        let p = self.rows[1 + 3 * i];
        let d = self.rows[2 + 3 * i];
        let c = self.rows[3 + 3 * i];
        Some(Emitter {
            position: [p[0], p[1], p[2]],
            range: p[3],
            direction: [d[0], d[1], d[2]],
            cos_half_angle: d[3],
            color_rgb: [c[0], c[1], c[2]],
            intensity: c[3],
        })
    }

    fn map_emitters(self, f: impl Fn(Emitter) -> Emitter) -> Self {
        let n = self.count();
        let mapped: Vec<Emitter> = (0..n).filter_map(|i| self.emitter(i).map(&f)).collect();
        // from_emitters rewrites the header + live rows; the dead
        // rows beyond the count were zero and stay zero.
        Self::from_emitters(&mapped)
    }

    /// Convert the block **from game space** (the
    /// [`EmitterUniforms::from_emitters`] output space) to
    /// treebuild grid space (+X east, +Y south, +Z up, z in
    /// blocks): negate Y (north→south) and scale Z by
    /// `blocks_per_z_level`, for positions and (re-normalized)
    /// directions. Range and colour/intensity pass through
    /// (module doc: range is in the target space's units).
    ///
    /// This converts FROM game space only; applying it to an
    /// already-converted block double-converts.
    pub fn to_grid_space(self) -> Self {
        let per = WorldScale::default().blocks_per_z_level as f32;
        self.map_emitters(|mut e| {
            e.position = [e.position[0], -e.position[1], e.position[2] * per];
            e.direction = normalize3([e.direction[0], -e.direction[1], e.direction[2] * per]);
            e
        })
    }

    /// Convert the block **from game space** to mesher world
    /// space (+X east, +Y up, +Z north, y in world units): the
    /// position becomes (east, up × blocks_per_z_level ×
    /// block_size, north) — a pure axis swap like
    /// [`crate::SkyUniforms::to_mesher_world_space`], with the
    /// up axis scaled to world units; the direction gets the
    /// same map, re-normalized. Range and colour/intensity pass
    /// through. (No negation anywhere here: world Z IS north.
    /// The negation belongs to the grid conversion, whose Y is
    /// south.)
    ///
    /// This converts FROM game space only; applying it to an
    /// already-converted block double-converts.
    pub fn to_mesher_world_space(self) -> Self {
        let s = WorldScale::default();
        let z_scale = s.blocks_per_z_level as f32 * s.block_size;
        self.map_emitters(|mut e| {
            e.position = [e.position[0], e.position[2] * z_scale, e.position[1]];
            e.direction = normalize3([e.direction[0], e.direction[2] * z_scale, e.direction[1]]);
            e
        })
    }
}

// ---------------------------------------------------------------------------
// Shared WGSL — single source of truth (mirrors the SKY_WGSL pattern)
// ---------------------------------------------------------------------------

/// The shared emitter shader source: the WGSL `EmitterUniforms`
/// struct (row-for-row the Rust layout above) plus the cone +
/// distance falloff math. Both renderers embed this string
/// verbatim; there is no second copy. The binding declaration is
/// each renderer's own (group/binding differ); the functions take
/// the block as a parameter.
///
/// Falloff shape (the prototype's transport model):
/// - distance: `x = clamp(1 − d/range)` shaped by the smoothstep
///   polynomial `x²(3 − 2x)` — 1 at the emitter, smoothly 0 at
///   `range`, exactly 0 beyond;
/// - cone: smoothstep from `cos_half_angle` (0 at the authored
///   edge) to an inner cos [`CONE_INNER_FRACTION`] of the way
///   toward the axis (1 inside the inner cone);
/// - contribution = colour × intensity × distance × cone ×
///   visibility.
///
/// NO shadow logic lives here: `emitter_term` takes the caller's
/// per-emitter `visibility` (0 or 1 in v1; the RT renderer
/// supplies a shadow-ray result, the raster supplies 1.0).
pub const EMITTER_WGSL: &str = r#"
// Addendum 11 shared emitters (single source of truth:
// crates/lighting/src/emitters.rs, EMITTER_WGSL). Do not copy-edit
// elsewhere — include this string.
struct EmitterUniforms {
    rows: array<vec4<f32>, 13>,
}

// Mirror of lighting::emitters::CONE_INNER_FRACTION — keep in sync.
const EMITTER_CONE_INNER_FRACTION: f32 = 0.35;

fn emitter_count(u: EmitterUniforms) -> i32 {
    return i32(u.rows[0].x + 0.5);
}

fn emitter_pos(u: EmitterUniforms, i: i32) -> vec3<f32> {
    return u.rows[1 + i * 3].xyz;
}

fn emitter_range(u: EmitterUniforms, i: i32) -> f32 {
    return u.rows[1 + i * 3].w;
}

fn emitter_dir(u: EmitterUniforms, i: i32) -> vec3<f32> {
    return u.rows[2 + i * 3].xyz;
}

fn emitter_cos_half(u: EmitterUniforms, i: i32) -> f32 {
    return u.rows[2 + i * 3].w;
}

fn emitter_color(u: EmitterUniforms, i: i32) -> vec3<f32> {
    return u.rows[3 + i * 3].xyz;
}

fn emitter_intensity(u: EmitterUniforms, i: i32) -> f32 {
    return u.rows[3 + i * 3].w;
}

// One emitter's contribution at point `p` (same space as the
// block's positions), scaled by the caller-supplied visibility.
// Zero at and beyond range, zero outside the cone.
fn emitter_term(u: EmitterUniforms, i: i32, p: vec3<f32>, visibility: f32) -> vec3<f32> {
    let to_p = p - emitter_pos(u, i);
    let d = length(to_p);
    let range = emitter_range(u, i);
    if d >= range || range <= 0.0 {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let x = clamp(1.0 - d / range, 0.0, 1.0);
    let dist_f = x * x * (3.0 - 2.0 * x);
    var cone_f = 1.0;
    if d > 1e-5 {
        let cosang = dot(to_p / d, emitter_dir(u, i));
        let ch = emitter_cos_half(u, i);
        let cos_inner = ch + (1.0 - ch) * EMITTER_CONE_INNER_FRACTION;
        cone_f = smoothstep(ch, cos_inner, cosang);
    }
    return emitter_color(u, i) * (emitter_intensity(u, i) * dist_f * cone_f * visibility);
}

// The summed contribution of every emitter at `p`, all at full
// visibility — the raster form (v1 raster has no traced emitter
// shadows; the RT renderer calls emitter_term per emitter with
// its shadow-ray visibility instead).
fn emitter_light(u: EmitterUniforms, p: vec3<f32>) -> vec3<f32> {
    var sum = vec3<f32>(0.0, 0.0, 0.0);
    let count = emitter_count(u);
    for (var i = 0; i < count; i = i + 1) {
        sum = sum + emitter_term(u, i, p, 1.0);
    }
    return sum;
}
"#;

// ---------------------------------------------------------------------------
// CPU mirror of the WGSL above
// ---------------------------------------------------------------------------

fn smoothstep_f(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Pure-Rust mirror of the WGSL `emitter_term`, for CPU gates.
/// `u` is a block in any one space; `p` must be in the same space.
/// `visibility` is the caller's per-emitter visibility.
pub fn emitter_term_cpu(u: &EmitterUniforms, i: usize, p: [f32; 3], visibility: f32) -> [f32; 3] {
    let Some(e) = u.emitter(i) else {
        return [0.0; 3];
    };
    let to_p = [
        p[0] - e.position[0],
        p[1] - e.position[1],
        p[2] - e.position[2],
    ];
    let d = (to_p[0] * to_p[0] + to_p[1] * to_p[1] + to_p[2] * to_p[2]).sqrt();
    if d >= e.range || e.range <= 0.0 {
        return [0.0; 3];
    }
    let x = (1.0 - d / e.range).clamp(0.0, 1.0);
    let dist_f = x * x * (3.0 - 2.0 * x);
    let mut cone_f = 1.0;
    if d > 1e-5 {
        let cosang = (to_p[0] * e.direction[0] + to_p[1] * e.direction[1] + to_p[2] * e.direction[2]) / d;
        let cos_inner = e.cos_half_angle + (1.0 - e.cos_half_angle) * CONE_INNER_FRACTION;
        cone_f = smoothstep_f(e.cos_half_angle, cos_inner, cosang);
    }
    let s = e.intensity * dist_f * cone_f * visibility;
    [e.color_rgb[0] * s, e.color_rgb[1] * s, e.color_rgb[2] * s]
}

/// Pure-Rust mirror of the WGSL `emitter_light` (all emitters at
/// full visibility).
pub fn emitter_light_cpu(u: &EmitterUniforms, p: [f32; 3]) -> [f32; 3] {
    let mut sum = [0.0; 3];
    for i in 0..u.count() {
        let t = emitter_term_cpu(u, i, p, 1.0);
        sum = [sum[0] + t[0], sum[1] + t[1], sum[2] + t[2]];
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headlight() -> Emitter {
        Emitter::new([6.15, -17.4, 0.5], [1.0, 0.0, -0.12], 35.0, 14.0, 2.2, [1.0, 0.85, 0.62])
    }

    #[test]
    fn empty_block_has_zero_count_and_zero_light() {
        let u = EmitterUniforms::empty();
        assert_eq!(u.count(), 0);
        assert_eq!(emitter_light_cpu(&u, [0.0, 0.0, 0.0]), [0.0; 3]);
        assert_eq!(u.as_rows().len(), EMITTER_UNIFORM_ROWS);
        assert_eq!(
            std::mem::size_of::<EmitterUniforms>(),
            EMITTER_UNIFORM_ROWS * 16
        );
    }

    #[test]
    fn pack_round_trips_and_caps_at_max() {
        let e = headlight();
        let u = EmitterUniforms::from_emitters(&[e]);
        assert_eq!(u.count(), 1);
        let back = u.emitter(0).expect("emitter 0");
        assert_eq!(back.position, e.position);
        assert_eq!(back.range, e.range);
        assert_eq!(back.cos_half_angle, e.cos_half_angle);
        assert_eq!(back.intensity, e.intensity);
        assert_eq!(back.color_rgb, e.color_rgb);
        // Direction was normalized at construction; the packed
        // copy is the same unit vector.
        for c in 0..3 {
            assert!((back.direction[c] - e.direction[c]).abs() < 1e-6);
        }
        let many: Vec<Emitter> = (0..MAX_EMITTERS + 3).map(|_| e).collect();
        assert_eq!(EmitterUniforms::from_emitters(&many).count(), MAX_EMITTERS);
    }

    #[test]
    fn falloff_shape_axis_edge_and_range() {
        let u = EmitterUniforms::from_emitters(&[headlight()]);
        let pos = [6.15, -17.4, 0.5];
        // On the emitter: full intensity x colour (cone guard).
        let at = emitter_term_cpu(&u, 0, pos, 1.0);
        assert!((at[0] - 2.2).abs() < 1e-5, "at emitter: {at:?}");
        // Dead ahead, halfway: distance factor x=0.5 -> 0.5,
        // cone 1 (inside the inner cone dead ahead — the tilt
        // keeps a level-ahead point slightly off-axis, so use a
        // point exactly on the axis instead).
        let dir = headlight().direction;
        let mid = [
            pos[0] + dir[0] * 7.0,
            pos[1] + dir[1] * 7.0,
            pos[2] + dir[2] * 7.0,
        ];
        let m = emitter_term_cpu(&u, 0, mid, 1.0);
        assert!((m[0] - 2.2 * 0.5).abs() < 1e-4, "mid-axis: {m:?}");
        // At range on the axis: exactly zero.
        let far = [
            pos[0] + dir[0] * 14.0,
            pos[1] + dir[1] * 14.0,
            pos[2] + dir[2] * 14.0,
        ];
        assert_eq!(emitter_term_cpu(&u, 0, far, 1.0), [0.0; 3]);
        // Behind the emitter: outside the cone, zero.
        let behind = [pos[0] - 2.0, pos[1], pos[2]];
        assert_eq!(emitter_term_cpu(&u, 0, behind, 1.0), [0.0; 3]);
        // Visibility scales linearly; 0 kills the term.
        assert_eq!(emitter_term_cpu(&u, 0, mid, 0.0), [0.0; 3]);
        let half = emitter_term_cpu(&u, 0, mid, 0.5);
        assert!((half[0] - m[0] * 0.5).abs() < 1e-5);
    }

    #[test]
    fn space_conversions_move_position_and_tilt() {
        let u = EmitterUniforms::from_emitters(&[headlight()]);
        // Grid space: (east, -north, up x 3 blocks/level).
        let g = u.to_grid_space().emitter(0).expect("grid emitter");
        assert_eq!(g.position, [6.15, 17.4, 1.5]);
        assert!(g.direction[2] < 0.0, "downward tilt survives: {:?}", g.direction);
        let gl = (g.direction[0].powi(2) + g.direction[1].powi(2) + g.direction[2].powi(2)).sqrt();
        assert!((gl - 1.0).abs() < 1e-6, "direction stays unit: {gl}");
        // Mesher world space: (east, up x 3 world units, north) —
        // north −17.4 stays −17.4 (world Z is north, unsigned).
        let w = u.to_mesher_world_space().emitter(0).expect("world emitter");
        assert_eq!(w.position, [6.15, 1.5, -17.4]);
        assert!(w.direction[1] < 0.0, "tilt becomes -Y in world space: {:?}", w.direction);
        // Range / colour / intensity are space-invariant.
        assert_eq!(g.range, 14.0);
        assert_eq!(w.range, 14.0);
        assert_eq!(g.color_rgb, [1.0, 0.85, 0.62]);
    }
}
