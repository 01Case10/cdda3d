//! Procedural noise foundation (Procedural Texture Pipeline, Stage C
//! first half): the deterministic, tileable noise primitives the
//! Stage C material recipes sample from. No recipes live here — this
//! module is only the substrate they compose.
//!
//! # What a recipe author calls, and in what units
//!
//! All sampling is in **lattice-cell units**: one unit of `x`/`y` is
//! one noise cell, and every sampler is periodic with an integer
//! `period` counted in cells. A recipe generating one 32 px tile
//! texture with, say, 4 features across the tile constructs
//! `ValueNoise::new(seed, 4)` and either samples in cell units
//! directly (`sample(px / 8.0, py / 8.0)`) or uses the per-tile
//! convenience form `sample_uv(u, v)`, where `u`/`v` are in **tiles**
//! (`0.0..1.0` across one tile) and are scaled by `period`
//! internally. Larger spans compose the same way: a macro texture
//! covering 4x4 tiles samples in tile units times the per-tile
//! period, and stays tileable over the macro span when the period is
//! multiplied by 4 as well. World/voxel scale is a call-site concern:
//! multiply world coordinates by the cells-per-voxel density before
//! sampling; nothing in here knows about voxels.
//!
//! - [`ValueNoise`] — smooth lattice noise in `[0, 1]`, the base
//!   signal for mottling, erosion masks and height fields.
//! - [`Worley`] — cellular noise returning `(F1, F2)` distances in
//!   cell units; `F2 - F1` gives cell borders (cracks, paving,
//!   scales), `F1` gives blobs.
//! - [`Fbm`] — fractal sum of [`ValueNoise`] octaves, normalized to
//!   `[0, 1]`; periodic with the base period (see its docs for why
//!   lacunarity is an integer).
//! - [`Rng`] / [`seed_from_str`] — deterministic seed plumbing.
//!   Content seeds come from strings (material ids, recipe names)
//!   via [`seed_from_str`]; never seed from `std` hashing, whose
//!   output is randomized per process.
//! - [`white_noise`] — per-cell hash value in `[0, 1]`, the u64
//!   analogue of `faces::hash_noise` for per-cell random picks.
//!
//! # Determinism
//!
//! Everything is pure: same seed and coordinates give bit-identical
//! output across runs and platforms. All lattice/feature hashing is
//! `u64` integer arithmetic (splitmix64 finalizer, in the same
//! wrapping-multiply/xor spirit as the existing `faces::hash_noise`);
//! interpolation uses only `+`, `*`, `floor` and `sqrt`, which are
//! exactly rounded under IEEE-754, and no transcendentals appear
//! anywhere. One caveat: exactness is a property of the wrapped
//! computation, so the *edges* are bit-exact — `f(0, y)` and
//! `f(period, y)` are the identical computation after wrapping —
//! while a period-shifted *interior* point `f(x + period, y)` can
//! differ by ~1 ulp because `x + period` itself rounds in `f32`.
//! The gate tests assert both facts separately.

/// Golden-ratio constant used by splitmix64 and the lattice hash.
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// splitmix64 finalizer: strong avalanche for a single u64.
fn splitmix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hash one lattice/feature cell for `seed`. Pure integer math.
fn lattice_hash(seed: u64, x: u32, y: u32) -> u64 {
    let z = seed
        ^ (x as u64).wrapping_mul(GOLDEN)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    splitmix64(z)
}

/// Top 24 bits of a hash as a float in `[0, 1)`. Exact in `f32`:
/// the integer is below 2^24 and the scale is a power of two.
fn unit_from_hash(h: u64) -> f32 {
    ((h >> 40) as f32) * (1.0 / 16_777_216.0)
}

/// Derive a stable seed from a content string (material id, recipe
/// name, ...). FNV-1a 64 over the UTF-8 bytes, then a splitmix64
/// avalanche so similar strings (`t_grass`, `t_grass2`) decorrelate.
/// Stable across runs and platforms by construction — unlike
/// `std::collections::hash_map::DefaultHasher`, which is randomized.
pub fn seed_from_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    splitmix64(h)
}

/// Small deterministic RNG (splitmix64 stream) for recipe-side
/// random picks that must reproduce exactly: palette jitter,
/// variant selection, scatter placement. Not cryptographic.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        splitmix64(self.state)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform float in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        unit_from_hash(self.next_u64())
    }

    /// Uniform integer in `[0, n)` (multiply-high, no modulo bias
    /// worth measuring at recipe scales). `n` must be nonzero.
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "Rng::below(0)");
        (((self.next_u64() >> 32) * (n as u64)) >> 32) as u32
    }
}

/// Per-cell white noise in `[0, 1)`: the hash of one cell, with no
/// interpolation. For per-cell random picks (which variant, which
/// tint step) where smoothness would be wrong.
pub fn white_noise(seed: u64, x: u32, y: u32) -> f32 {
    unit_from_hash(lattice_hash(seed, x, y))
}

/// Periodic value noise: a random value per lattice point,
/// smoothstep-interpolated between them. Output in `[0, 1]`.
///
/// Sampling coordinates are in cell units and wrap with
/// [`period`](Self::period): lattice coordinates are taken modulo
/// the period, so the field tiles exactly over `period` cells.
pub struct ValueNoise {
    seed: u64,
    period: u32,
}

impl ValueNoise {
    /// `period` is the tile size in cells and must be at least 1.
    pub fn new(seed: u64, period: u32) -> Self {
        assert!(period >= 1, "ValueNoise period must be >= 1");
        ValueNoise { seed, period }
    }

    pub fn period(&self) -> u32 {
        self.period
    }

    fn lattice_value(&self, x: u32, y: u32) -> f32 {
        unit_from_hash(lattice_hash(self.seed, x, y))
    }

    /// Sample at `(x, y)` in cell units. Inputs may be any finite
    /// floats; they wrap into `[0, period)` before sampling.
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let p = self.period as f32;
        let xw = x.rem_euclid(p);
        let yw = y.rem_euclid(p);
        let xi = xw.floor();
        let yi = yw.floor();
        let xf = xw - xi;
        let yf = yw - yi;
        let x0 = xi as u32;
        let y0 = yi as u32;
        let x1 = (x0 + 1) % self.period;
        let y1 = (y0 + 1) % self.period;
        // Smoothstep weights: zero slope at lattice points, so the
        // field is C1 across cell borders.
        let u = xf * xf * (3.0 - 2.0 * xf);
        let v = yf * yf * (3.0 - 2.0 * yf);
        let a = self.lattice_value(x0, y0);
        let b = self.lattice_value(x1, y0);
        let c = self.lattice_value(x0, y1);
        let d = self.lattice_value(x1, y1);
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    /// Sample in tile units: `(0, 0)` is one tile corner and
    /// `(1, 1)` the opposite; equivalent to
    /// `sample(u * period, v * period)`. Tiles exactly.
    pub fn sample_uv(&self, u: f32, v: f32) -> f32 {
        self.sample(u * self.period as f32, v * self.period as f32)
    }
}

/// One Worley sample: distances to the nearest (`f1`) and
/// second-nearest (`f2`) feature points, in cell units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorleySample {
    pub f1: f32,
    pub f2: f32,
}

impl WorleySample {
    /// `F2 - F1`: near zero along cell borders, up to ~1 inside
    /// cells. The crack/paving signal.
    pub fn f2_minus_f1(&self) -> f32 {
        self.f2 - self.f1
    }
}

/// Periodic Worley (cellular) noise: one jittered feature point per
/// cell, cell coordinates wrapped modulo the period. Distances are
/// in cell units; `f1` lies in `[0, sqrt(2))` because the sample's
/// own cell always contributes a candidate point.
pub struct Worley {
    seed: u64,
    period: u32,
}

impl Worley {
    /// `period` is the tile size in cells and must be at least 1.
    pub fn new(seed: u64, period: u32) -> Self {
        assert!(period >= 1, "Worley period must be >= 1");
        Worley { seed, period }
    }

    pub fn period(&self) -> u32 {
        self.period
    }

    /// Jittered feature point position within cell `(cx, cy)`
    /// (already wrapped), as offsets in `[0, 1)` from the cell
    /// corner. Two domain-salted hashes so x/y are independent.
    fn feature_offset(&self, cx: u32, cy: u32) -> (f32, f32) {
        let hx = lattice_hash(self.seed ^ 0xA076_1D64_78BD_642F, cx, cy);
        let hy = lattice_hash(self.seed ^ 0xE703_7ED1_A0B4_28DB, cx, cy);
        (unit_from_hash(hx), unit_from_hash(hy))
    }

    /// Sample at `(x, y)` in cell units; inputs wrap into
    /// `[0, period)` before sampling.
    pub fn sample(&self, x: f32, y: f32) -> WorleySample {
        let p = self.period as f32;
        let xw = x.rem_euclid(p);
        let yw = y.rem_euclid(p);
        let cx = xw.floor() as i64;
        let cy = yw.floor() as i64;
        let period = self.period as i64;
        let mut f1 = f32::INFINITY;
        let mut f2 = f32::INFINITY;
        // 3x3 neighbourhood: with one feature point per cell, the
        // two nearest points to any sample are always in the
        // adjacent cells (the own-cell point alone bounds F1 by
        // sqrt(2), and any point two cells away is farther than
        // the second-nearest candidate found here for unit jitter).
        for dy in -1..=1i64 {
            for dx in -1..=1i64 {
                let cell_x = cx + dx;
                let cell_y = cy + dy;
                // Hash the wrapped cell, but position its point
                // relative to the *unwrapped* cell so distances
                // across the seam measure the short way.
                let wrapped_x = cell_x.rem_euclid(period) as u32;
                let wrapped_y = cell_y.rem_euclid(period) as u32;
                let (jx, jy) = self.feature_offset(wrapped_x, wrapped_y);
                let px = cell_x as f32 + jx;
                let py = cell_y as f32 + jy;
                let ddx = px - xw;
                let ddy = py - yw;
                let d = (ddx * ddx + ddy * ddy).sqrt();
                if d < f1 {
                    f2 = f1;
                    f1 = d;
                } else if d < f2 {
                    f2 = d;
                }
            }
        }
        WorleySample { f1, f2 }
    }

    /// Sample in tile units; see [`ValueNoise::sample_uv`].
    pub fn sample_uv(&self, u: f32, v: f32) -> WorleySample {
        self.sample(u * self.period as f32, v * self.period as f32)
    }
}

/// Fractal Brownian motion over [`ValueNoise`]: a sum of octaves at
/// doubling frequency and falling amplitude, normalized to `[0, 1]`
/// by the amplitude sum.
///
/// Lacunarity is an integer (2 in practice) on purpose: octave `i`
/// is periodic with `period * lacunarity^i` cells and is sampled at
/// `lacunarity^i` times the base frequency, so every octave — and
/// therefore the sum — is exactly periodic with the base `period`.
/// A fractional lacunarity would break that identity and with it
/// tileability, which is a hard requirement of the pipeline.
pub struct Fbm {
    seed: u64,
    period: u32,
    octaves: u32,
    lacunarity: u32,
    gain: f32,
}

impl Fbm {
    /// Defaults: 4 octaves, lacunarity 2, gain 0.5.
    pub fn new(seed: u64, period: u32) -> Self {
        assert!(period >= 1, "Fbm period must be >= 1");
        Fbm {
            seed,
            period,
            octaves: 4,
            lacunarity: 2,
            gain: 0.5,
        }
    }

    pub fn octaves(mut self, octaves: u32) -> Self {
        assert!(octaves >= 1, "Fbm octaves must be >= 1");
        self.octaves = octaves;
        self
    }

    pub fn lacunarity(mut self, lacunarity: u32) -> Self {
        assert!(lacunarity >= 1, "Fbm lacunarity must be >= 1");
        self.lacunarity = lacunarity;
        self
    }

    /// Per-octave amplitude multiplier, in `(0, 1]`.
    pub fn gain(mut self, gain: f32) -> Self {
        assert!(gain > 0.0 && gain <= 1.0, "Fbm gain must be in (0, 1]");
        self.gain = gain;
        self
    }

    pub fn period(&self) -> u32 {
        self.period
    }

    /// Sample at `(x, y)` in base-cell units; inputs wrap with the
    /// base period (each octave wraps with its own).
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let mut amp = 1.0f32;
        let mut sum = 0.0f32;
        let mut norm = 0.0f32;
        let mut freq: u32 = 1;
        for i in 0..self.octaves {
            // Per-octave seed: avalanched offset so octaves are
            // independent fields, not shifted copies of each other.
            let octave_seed = splitmix64(self.seed.wrapping_add(i as u64));
            let octave = ValueNoise::new(octave_seed, self.period * freq);
            sum += amp * octave.sample(x * freq as f32, y * freq as f32);
            norm += amp;
            amp *= self.gain;
            freq *= self.lacunarity;
        }
        sum / norm
    }

    /// Sample in tile units; see [`ValueNoise::sample_uv`].
    pub fn sample_uv(&self, u: f32, v: f32) -> f32 {
        self.sample(u * self.period as f32, v * self.period as f32)
    }
}
