//! Phase 10 Wave 2 FREEZE — the shared sky contract both renderers
//! consume (docs/phase10-sky-gi.md §4, §5.1, §6). Slices 10.3
//! (raster) and 10.4 (RT) build in parallel on top of this module;
//! its layout and WGSL are **frozen**: changing a field, its order,
//! or the WGSL math is a contract change that must be re-frozen
//! with both slices, not edited unilaterally.
//!
//! Contents:
//! - [`SkyUniforms`]: the uniform block, an explicitly-padded
//!   `#[repr(C)]` struct of eight `[f32; 4]` rows (128 bytes), each
//!   row one `vec4<f32>` in WGSL. No implicit padding anywhere.
//! - [`SKY_WGSL`]: the single source of truth for the shader side —
//!   the WGSL `SkyUniforms` struct (field-for-field the Rust layout)
//!   plus `sky_color(rd, rd_up, sky)`.
//! - [`sky_color_cpu`]: a pure-Rust mirror of the WGSL function, for
//!   CPU gates. The star hash is integer-based (u32 ops on the
//!   quantized direction) so Rust and WGSL agree exactly.
//!
//! ## Coordinate-space contract
//!
//! [`SkyState`](crate::sky::SkyState) directions are **engine world
//! space**: +X east, +Y north, +Z up. [`SkyUniforms::from_sky_state`]
//! emits those directions **unchanged** — it does not convert
//! spaces. Each renderer consumes the uniforms in its own space, so
//! the *caller* converts the sun/moon directions when uploading:
//! the raster renderer is already world-space (no conversion); the
//! RT renderer is grid space (+X east, +Y south, +Z up) and must run
//! the dirs through `treebuild::world_to_grid` — the same transform
//! that produced the legacy RT sun dir: world (0.3, 0.8, 0.4) → grid
//! (0.3, −0.4, 0.8). [`SkyUniforms::neutral`] carries the *grid-space*
//! legacy dir because neutral exists to reproduce the legacy RT
//! `sky()` byte-for-byte; world-space callers at neutral should use
//! the world legacy dir (0.3, 0.8, 0.4) normalized — the gradient is
//! identical either way since only `rd_up` and the stops enter it,
//! and at neutral the discs are gated off.
//!
//! `sky_color` takes the ray direction `rd` **plus** its up
//! component `rd_up` explicitly (grid callers pass `rd.z`, world
//! callers pass `rd.y`), so the one function serves both spaces
//! without a space flag.
//!
//! ## Cloud dimming lives CPU-side
//!
//! §4's disc term is `× sun_color × (1 − cloud_cover·0.9)`. The
//! cloud factor is folded into the disc *scales* by
//! [`SkyUniforms::from_sky_state`] (and the star scale carries
//! night × (1 − cloud) × fog fade), so the WGSL stays dumb: it
//! multiplies by the scales it is given and never re-derives them.
//!
//! ## Deviation from §4's tentative text (recorded per the freeze)
//!
//! §4 imagined the WGSL string living in `treebuild` with the engine
//! including it and a gate diffing two copies. Instead the string
//! and struct live **here, in `lighting`**: it is the only crate
//! both renderers can depend on acyclically (engine already depends
//! on lighting; treebuild adding lighting is acyclic — lighting
//! depends only on mesher + proto). One literal string consumed by
//! both renderers is strictly stronger than two diffed copies.

use crate::sky::SkyState;

/// Legacy gradient stops (§6 neutral identity): the raster
/// `CLEAR_COLOR` / RT `sky()` horizon, and the RT zenith/ground.
pub const LEGACY_HORIZON: [f32; 3] = [0.08, 0.10, 0.14];
pub const LEGACY_ZENITH: [f32; 3] = [0.30, 0.48, 0.78];
pub const LEGACY_GROUND: [f32; 3] = [0.05, 0.06, 0.08];
/// Legacy RT sun direction, grid space, *unnormalized* source
/// vector: (0.3, −0.4, 0.8). `treebuild::sun_dir_grid` normalizes
/// it; it is the world-space raster light dir (0.3, 0.8, 0.4) run
/// through `treebuild::world_to_grid` (grid = (wx, −wz, wy)).
pub const LEGACY_SUN_DIR_GRID: [f32; 3] = [0.3, -0.4, 0.8];

/// Cave gradient stops (decision 3C): very dark blue/brown, so
/// underground silhouettes still separate from the background.
/// Mirrored as constants in [`SKY_WGSL`] — the gate tests pin the
/// CPU side; keep the two in sync when editing (the WGSL-match gate
/// covers the struct, these values are covered by review).
pub const CAVE_ZENITH: [f32; 3] = [0.012, 0.016, 0.030];
pub const CAVE_HORIZON: [f32; 3] = [0.030, 0.026, 0.020];
pub const CAVE_GROUND: [f32; 3] = [0.008, 0.007, 0.006];

// ---------------------------------------------------------------------------
// SkyUniforms — frozen layout (8 rows × vec4 = 128 bytes)
// ---------------------------------------------------------------------------

/// The shared sky uniform block (§5.1, frozen). Each field is one
/// WGSL `vec4<f32>`: `.xyz` carries the §5.1 vector/scalar's vector
/// part and `.w` carries a scalar gate, packed as follows:
///
/// | row | field       | .xyz                    | .w              |
/// |-----|-------------|-------------------------|-----------------|
/// | 0   | sun_dir     | toward the sun          | sun disc scale  |
/// | 1   | sun_color   | sun colour × intensity  | moon disc scale |
/// | 2   | moon_dir    | toward the moon         | star scale      |
/// | 3   | moon_color  | moon colour × intensity | cave factor     |
/// | 4   | zenith      | zenith stop             | ambient_scale   |
/// | 5   | horizon     | horizon stop            | fog_density     |
/// | 6   | ground_col  | ground stop             | cloud_cover     |
/// | 7   | fog_color   | fog colour              | time_sec        |
///
/// The disc/star scales are explicit gates, not derived in-shader:
/// at [`SkyUniforms::neutral`] they are exactly 0, which is what
/// makes the neutral sky reproduce the legacy gradient bit-for-bit
/// (§6) regardless of the colours packed beside them.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyUniforms {
    pub sun_dir: [f32; 4],
    pub sun_color: [f32; 4],
    pub moon_dir: [f32; 4],
    pub moon_color: [f32; 4],
    pub zenith: [f32; 4],
    pub horizon: [f32; 4],
    pub ground_col: [f32; 4],
    pub fog_color: [f32; 4],
}

// Frozen-layout assertions (compile time).
const _: () = {
    assert!(std::mem::size_of::<SkyUniforms>() == 128);
    assert!(std::mem::offset_of!(SkyUniforms, sun_dir) == 0);
    assert!(std::mem::offset_of!(SkyUniforms, sun_color) == 16);
    assert!(std::mem::offset_of!(SkyUniforms, moon_dir) == 32);
    assert!(std::mem::offset_of!(SkyUniforms, moon_color) == 48);
    assert!(std::mem::offset_of!(SkyUniforms, zenith) == 64);
    assert!(std::mem::offset_of!(SkyUniforms, horizon) == 80);
    assert!(std::mem::offset_of!(SkyUniforms, ground_col) == 96);
    assert!(std::mem::offset_of!(SkyUniforms, fog_color) == 112);
};

impl SkyUniforms {
    /// The eight rows in layout order (upload / test helper).
    pub fn as_rows(&self) -> [[f32; 4]; 8] {
        [
            self.sun_dir,
            self.sun_color,
            self.moon_dir,
            self.moon_color,
            self.zenith,
            self.horizon,
            self.ground_col,
            self.fog_color,
        ]
    }

    pub fn sun_dir3(&self) -> [f32; 3] {
        [self.sun_dir[0], self.sun_dir[1], self.sun_dir[2]]
    }
    pub fn sun_disc_scale(&self) -> f32 {
        self.sun_dir[3]
    }
    pub fn sun_color3(&self) -> [f32; 3] {
        [self.sun_color[0], self.sun_color[1], self.sun_color[2]]
    }
    pub fn moon_disc_scale(&self) -> f32 {
        self.sun_color[3]
    }
    pub fn moon_dir3(&self) -> [f32; 3] {
        [self.moon_dir[0], self.moon_dir[1], self.moon_dir[2]]
    }
    pub fn star_scale(&self) -> f32 {
        self.moon_dir[3]
    }
    pub fn moon_color3(&self) -> [f32; 3] {
        [self.moon_color[0], self.moon_color[1], self.moon_color[2]]
    }
    pub fn cave_factor(&self) -> f32 {
        self.moon_color[3]
    }
    pub fn set_cave_factor(&mut self, f: f32) {
        self.moon_color[3] = f.clamp(0.0, 1.0);
    }
    pub fn zenith3(&self) -> [f32; 3] {
        [self.zenith[0], self.zenith[1], self.zenith[2]]
    }
    pub fn ambient_scale(&self) -> f32 {
        self.zenith[3]
    }
    pub fn horizon3(&self) -> [f32; 3] {
        [self.horizon[0], self.horizon[1], self.horizon[2]]
    }
    pub fn fog_density(&self) -> f32 {
        self.horizon[3]
    }
    pub fn ground3(&self) -> [f32; 3] {
        [self.ground_col[0], self.ground_col[1], self.ground_col[2]]
    }
    pub fn cloud_cover(&self) -> f32 {
        self.ground_col[3]
    }
    pub fn fog_color3(&self) -> [f32; 3] {
        [self.fog_color[0], self.fog_color[1], self.fog_color[2]]
    }
    pub fn time_sec(&self) -> f32 {
        self.fog_color[3]
    }

    /// The §6 neutral block: today's hardcoded legacy constants,
    /// exactly. Gradient stops are the legacy RT `sky()` stops (the
    /// horizon is also the raster `CLEAR_COLOR`); the sun dir is the
    /// legacy **grid-space** dir (see the module doc for the space
    /// contract); discs, stars, cave and fog are all gated to
    /// exactly zero, so `sky_color(rd, neutral)` == legacy `sky(rd)`
    /// for every `rd`.
    pub fn neutral() -> Self {
        let sun = normalize3(LEGACY_SUN_DIR_GRID);
        Self {
            sun_dir: [sun[0], sun[1], sun[2], 0.0],
            // Colour values are inert at neutral (disc scale 0);
            // white keeps the block honest if a caller re-opens the
            // gate deliberately.
            sun_color: [1.0, 1.0, 1.0, 0.0],
            moon_dir: [-sun[0], -sun[1], -sun[2], 0.0],
            moon_color: [0.62, 0.72, 0.95, 0.0],
            zenith: [LEGACY_ZENITH[0], LEGACY_ZENITH[1], LEGACY_ZENITH[2], 1.0],
            horizon: [LEGACY_HORIZON[0], LEGACY_HORIZON[1], LEGACY_HORIZON[2], 0.0],
            ground_col: [LEGACY_GROUND[0], LEGACY_GROUND[1], LEGACY_GROUND[2], 0.0],
            fog_color: [LEGACY_HORIZON[0], LEGACY_HORIZON[1], LEGACY_HORIZON[2], 0.0],
        }
    }

    /// The one conversion both renderers use. Sun/moon colours are
    /// pre-multiplied by their intensities here (§5.1). All
    /// presentation gates are derived here from the state so the
    /// WGSL stays dumb:
    ///
    /// - disc scales: `1 − cloud_cover·0.9` (§4's cloud dimming,
    ///   applied CPU-side — see module doc);
    /// - star scale: night (sun below the horizon, full by −0.25 of
    ///   sun height) × `(1 − cloud_cover)` × fog fade
    ///   `exp(−fog_density·60)`;
    /// - cave factor: 0 — later slices set it from the camera
    ///   target via [`SkyUniforms::set_cave_factor`].
    ///
    /// Directions are emitted in SkyState's world space, unchanged
    /// (module doc: callers convert for their own space).
    pub fn from_sky_state(state: &SkyState, time_sec: f32) -> Self {
        let cloud_dim = 1.0 - state.cloud_cover.clamp(0.0, 1.0) * 0.9;
        let night = (-state.sun_dir[2] * 4.0).clamp(0.0, 1.0);
        let fog_fade = (-state.fog_density.max(0.0) * 60.0).exp();
        let star_scale = night * (1.0 - state.cloud_cover.clamp(0.0, 1.0)) * fog_fade;
        Self {
            sun_dir: [
                state.sun_dir[0],
                state.sun_dir[1],
                state.sun_dir[2],
                cloud_dim,
            ],
            sun_color: [
                state.sun_color[0] * state.sun_intensity,
                state.sun_color[1] * state.sun_intensity,
                state.sun_color[2] * state.sun_intensity,
                cloud_dim,
            ],
            moon_dir: [
                state.moon_dir[0],
                state.moon_dir[1],
                state.moon_dir[2],
                star_scale,
            ],
            moon_color: [
                state.moon_color[0] * state.moon_intensity,
                state.moon_color[1] * state.moon_intensity,
                state.moon_color[2] * state.moon_intensity,
                0.0,
            ],
            zenith: [
                state.zenith[0],
                state.zenith[1],
                state.zenith[2],
                state.ambient_scale,
            ],
            horizon: [
                state.horizon[0],
                state.horizon[1],
                state.horizon[2],
                state.fog_density,
            ],
            ground_col: [
                state.ground[0],
                state.ground[1],
                state.ground[2],
                state.cloud_cover,
            ],
            fog_color: [
                state.fog_color[0],
                state.fog_color[1],
                state.fog_color[2],
                time_sec,
            ],
        }
    }

    /// Convert the sun/moon directions FROM [`SkyUniforms::from_sky_state`]
    /// output space (SkyState space: +X east, +Y north, +Z up) TO grid
    /// space (+X east, +Y south, +Z up) — the RT renderer's space:
    /// negate the north component of `sun_dir`/`moon_dir` (`.xyz` only;
    /// the `.w` scalar gates are untouched).
    ///
    /// [`SkyUniforms::neutral`] is already in grid space and must NOT
    /// be converted — it carries the legacy grid-space sun dir by
    /// construction.
    pub fn to_grid_space(mut self) -> Self {
        self.sun_dir[1] = -self.sun_dir[1];
        self.moon_dir[1] = -self.moon_dir[1];
        self
    }

    /// Convert the sun/moon directions FROM [`SkyUniforms::from_sky_state`]
    /// output space (SkyState space: +X east, +Y north, +Z up) TO
    /// mesher world space (+X east, +Y up, +Z north) — the raster
    /// renderer's space: swap the north and up components of
    /// `sun_dir`/`moon_dir` (`.xyz` only; the `.w` scalar gates are
    /// untouched).
    ///
    /// [`SkyUniforms::neutral`] is already in grid space and must NOT
    /// be converted — it carries the legacy grid-space sun dir by
    /// construction.
    pub fn to_mesher_world_space(mut self) -> Self {
        self.sun_dir.swap(1, 2);
        self.moon_dir.swap(1, 2);
        self
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
// Shared WGSL — single source of truth (see module doc for the
// deviation from §4's tentative treebuild placement)
// ---------------------------------------------------------------------------

/// The shared sky shader source: the WGSL `SkyUniforms` struct
/// (field-for-field the frozen Rust layout above) and
/// `sky_color(rd, rd_up, sky)`. Both renderers embed this string
/// verbatim; there is no second copy.
pub const SKY_WGSL: &str = r#"
// Phase 10 shared sky (single source of truth:
// crates/lighting/src/sky_uniforms.rs, SKY_WGSL). Do not copy-edit
// elsewhere — include this string.
struct SkyUniforms {
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    moon_dir: vec4<f32>,
    moon_color: vec4<f32>,
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    ground_col: vec4<f32>,
    fog_color: vec4<f32>,
}

// Cave gradient stops (decision 3C): very dark blue/brown.
const SKY_CAVE_ZENITH: vec3<f32> = vec3<f32>(0.012, 0.016, 0.030);
const SKY_CAVE_HORIZON: vec3<f32> = vec3<f32>(0.030, 0.026, 0.020);
const SKY_CAVE_GROUND: vec3<f32> = vec3<f32>(0.008, 0.007, 0.006);

// Sun disc: cos-angle edges (inner ~2.1 deg, outer ~2.7 deg) and
// glow power/strength. The moon disc is smaller (inner ~1.5 deg).
const SKY_SUN_DISC_INNER: f32 = 0.99935;
const SKY_SUN_DISC_OUTER: f32 = 0.99890;
const SKY_MOON_DISC_INNER: f32 = 0.99965;
const SKY_MOON_DISC_OUTER: f32 = 0.99935;

fn sky_disc(rd: vec3<f32>, dir: vec3<f32>, inner: f32, outer: f32, glow_pow: f32, glow_str: f32) -> f32 {
    let c = dot(rd, dir);
    let disc = smoothstep(outer, inner, c);
    let glow = pow(clamp(c, 0.0, 1.0), glow_pow) * glow_str;
    return disc + glow;
}

// Deterministic integer star hash on the quantized direction:
// u32 ops only, so the CPU mirror agrees exactly.
fn sky_star(rd: vec3<f32>) -> f32 {
    let q = vec3<i32>(floor(rd * 300.0));
    var h = u32(q.x) * 374761393u + u32(q.y) * 668265263u + u32(q.z) * 974634211u;
    h = (h ^ (h >> 13u)) * 1274126177u;
    h = h ^ (h >> 16u);
    let v = f32(h) / 4294967295.0;
    return step(0.998, v) * v;
}

// rd: ray direction in the caller's space. rd_up: rd's up
// component in that space (grid/Z-up callers pass rd.z, world/Y-up
// callers pass rd.y). Sun/moon dirs in the uniforms must already be
// in the caller's space (the caller converts; see module doc).
fn sky_color(rd: vec3<f32>, rd_up: f32, sky: SkyUniforms) -> vec3<f32> {
    let cave = sky.moon_color.w;
    let zenith = mix(sky.zenith.xyz, SKY_CAVE_ZENITH, cave);
    let horizon = mix(sky.horizon.xyz, SKY_CAVE_HORIZON, cave);
    let ground = mix(sky.ground_col.xyz, SKY_CAVE_GROUND, cave);
    var col: vec3<f32>;
    if rd_up > 0.0 {
        col = mix(horizon, zenith, pow(clamp(rd_up, 0.0, 1.0), 0.6));
    } else {
        col = mix(horizon, ground, clamp(-rd_up * 3.0, 0.0, 1.0));
    }
    col += sky_disc(rd, sky.sun_dir.xyz, SKY_SUN_DISC_INNER, SKY_SUN_DISC_OUTER, 700.0, 0.30)
        * sky.sun_dir.w * sky.sun_color.xyz;
    col += sky_disc(rd, sky.moon_dir.xyz, SKY_MOON_DISC_INNER, SKY_MOON_DISC_OUTER, 900.0, 0.12)
        * sky.sun_color.w * sky.moon_color.xyz;
    let star = sky_star(rd) * sky.moon_dir.w * clamp(rd_up * 8.0, 0.0, 1.0);
    col += vec3<f32>(0.9, 0.95, 1.0) * star;
    // The sky's own horizon fog blend (fog on distant geometry is
    // a separate term, slice 10.5). Zero when fog_density is zero.
    let horizon_weight = 1.0 - abs(clamp(rd_up, -1.0, 1.0));
    let fog_amount = 1.0 - exp(-sky.horizon.w * 30.0 * horizon_weight * horizon_weight);
    col = mix(col, sky.fog_color.xyz, clamp(fog_amount, 0.0, 1.0));
    return col;
}
"#;

// ---------------------------------------------------------------------------
// CPU mirror of the WGSL above
// ---------------------------------------------------------------------------

fn mix3(a: [f32; 3], b: [f32; 3], f: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ]
}

fn smoothstep_f(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn disc_cpu(
    rd: [f32; 3],
    dir: [f32; 3],
    inner: f32,
    outer: f32,
    glow_pow: f32,
    glow_str: f32,
) -> f32 {
    let c = rd[0] * dir[0] + rd[1] * dir[1] + rd[2] * dir[2];
    smoothstep_f(outer, inner, c) + c.clamp(0.0, 1.0).powf(glow_pow) * glow_str
}

fn star_cpu(rd: [f32; 3]) -> f32 {
    let q = [
        (rd[0] * 300.0).floor() as i32,
        (rd[1] * 300.0).floor() as i32,
        (rd[2] * 300.0).floor() as i32,
    ];
    let mut h = (q[0] as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((q[1] as u32).wrapping_mul(668_265_263))
        .wrapping_add((q[2] as u32).wrapping_mul(974_634_211));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    let v = h as f32 / 4_294_967_295.0;
    if v >= 0.998 {
        v
    } else {
        0.0
    }
}

/// Pure-Rust mirror of the WGSL `sky_color` in [`SKY_WGSL`], for
/// CPU gates. `rd` is the ray direction in the caller's space and
/// `rd_up` its up component in that space (grid: `rd[2]`, world:
/// `rd[1]`) — the same contract as the WGSL.
pub fn sky_color_cpu(rd_up: f32, rd: [f32; 3], sky: &SkyUniforms) -> [f32; 3] {
    let cave = sky.cave_factor();
    let zenith = mix3(sky.zenith3(), CAVE_ZENITH, cave);
    let horizon = mix3(sky.horizon3(), CAVE_HORIZON, cave);
    let ground = mix3(sky.ground3(), CAVE_GROUND, cave);
    let mut col = if rd_up > 0.0 {
        mix3(horizon, zenith, rd_up.clamp(0.0, 1.0).powf(0.6))
    } else {
        mix3(horizon, ground, (-rd_up * 3.0).clamp(0.0, 1.0))
    };
    let sun = disc_cpu(rd, sky.sun_dir3(), 0.99935, 0.99890, 700.0, 0.30) * sky.sun_disc_scale();
    let sun_c = sky.sun_color3();
    col = [
        col[0] + sun * sun_c[0],
        col[1] + sun * sun_c[1],
        col[2] + sun * sun_c[2],
    ];
    let moon = disc_cpu(rd, sky.moon_dir3(), 0.99965, 0.99935, 900.0, 0.12) * sky.moon_disc_scale();
    let moon_c = sky.moon_color3();
    col = [
        col[0] + moon * moon_c[0],
        col[1] + moon * moon_c[1],
        col[2] + moon * moon_c[2],
    ];
    let star = star_cpu(rd) * sky.star_scale() * (rd_up * 8.0).clamp(0.0, 1.0);
    col = [col[0] + 0.9 * star, col[1] + 0.95 * star, col[2] + star];
    let horizon_weight = 1.0 - rd_up.clamp(-1.0, 1.0).abs();
    let fog_amount = 1.0 - (-sky.fog_density() * 30.0 * horizon_weight * horizon_weight).exp();
    mix3(col, sky.fog_color3(), fog_amount.clamp(0.0, 1.0))
}
