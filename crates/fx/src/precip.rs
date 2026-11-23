//! Precipitation emitter (Phase 10.5-early, the independent half
//! of Phase 10.5 in `docs/phase10-sky-gi.md` §5.5).
//!
//! Rain and snow as field-pass instances ([`FieldInstGpu`]),
//! produced on the CPU by a **stateless, deterministic** emitter:
//! the instance set is a pure function of (seed, state, camera
//! target, time, sky-exposure sampler), so a capture that pins its
//! time gets byte-identical precipitation — the same discipline as
//! the fx harness's pinned `--time`.
//!
//! The input state is defined HERE, not imported from
//! `crates/lighting`: `lighting::WeatherParams` carries the same
//! two numbers (`precip_rate`, `precip_fall_speed`), and converting
//! a `SkyState` into a [`PrecipState`] is Phase 10.6 integration
//! work. Semantics are the lighting table's, verbatim:
//!
//! - `rate` = particles spawned **per second per tile column**
//!   (Rain 40, HeavyRain 120, Snow 30 in the vanilla table);
//! - `fall_speed` = a multiplier on [`BASE_FALL_SPEED`]
//!   (Rain 1.0, HeavyRain 1.3, Snow 0.35). One shared base speed
//!   for both kinds is deliberate: it makes the rain:snow velocity
//!   ratio exactly the params' multiplier ratio (gate-checked).
//!   The kinds differ in *shape and drift*, not in the base
//!   constant — rain is a thin fast streak, snow a slow mote that
//!   sways as it falls.
//!
//! Shape: the field-pass instance format carries a scalar `size`
//! and a `class_id`; per-class scale in the vertex shader is the
//! established way classes get non-cube shapes (flame is
//! 1.3×1.7×1.3). Rain uses class [`RAIN_CLASS_ID`] (scale
//! 0.045 × 0.62 × 0.045 of size — a thin vertical streak, NOT a
//! round blob) and snow class [`SNOW_CLASS_ID`] (a small soft
//! mote). The fragment shader shades both classes with a soft
//! radial cross-section. See `render.rs`'s `FIELD_SHADER`.
//!
//! Sky exposure (decision 4A semantics): a particle only ever
//! spawns in a column whose tile the caller's sampler reports as
//! sky-exposed at the camera target's z-level. Under a roof there
//! is no rain — the sampler is the only source of that fact here,
//! exactly as the game will be the source of it in 10.6.
//!
//! Spawning model: each exposed tile column in the cylinder runs
//! `slots_per_column` independent slots. A slot respawns on a fixed
//! period `P = slots / rate` (≥ its fall lifetime, so a slot never
//! overlaps itself), with a hash-derived phase offset and a fresh
//! hash-derived in-tile position per cycle. Spawns per second per
//! column are therefore exactly `rate` up to the one-boundary-spawn
//! error per slot that the rate gate tolerates.
//!
//! Coordinates follow the crate convention (`scene.rs`): game
//! tiles (x east, y south, z up) map to world (x, z_up, −y_south);
//! a column's world footprint is its game tile's unit square, and
//! world y is the game z level (plus the fall height).

use glam::Vec3;

use crate::render::FieldInstGpu;

/// Fall speed at multiplier 1.0, tiles/second. Rain in the vanilla
/// table (multiplier 1.0) falls at this speed; snow (0.35) at
/// 3.15 tiles/s.
pub const BASE_FALL_SPEED: f32 = 9.0;

/// Field-shader class for rain: thin elongated streak instances.
pub const RAIN_CLASS_ID: u32 = 6;
/// Field-shader class for snow: small soft mote instances.
pub const SNOW_CLASS_ID: u32 = 7;

/// Rain streak appearance (fed to the shader via the instance).
const RAIN_COLOR: [f32; 3] = [0.62, 0.70, 0.84];
const RAIN_DENSITY: f32 = 0.75;
/// Snow mote appearance.
const SNOW_COLOR: [f32; 3] = [0.93, 0.96, 1.0];
const SNOW_DENSITY: f32 = 0.9;

/// What kind of precipitation falls. (Kind comes from the weather,
/// not from the numbers: snow and rain can share a rate.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecipKind {
    Rain,
    Snow,
}

impl PrecipKind {
    pub fn class_id(&self) -> u32 {
        match self {
            PrecipKind::Rain => RAIN_CLASS_ID,
            PrecipKind::Snow => SNOW_CLASS_ID,
        }
    }
}

/// Precipitation input for one frame — the fx-local mirror of the
/// `PrecipState` in the Phase 10 plan (§3.1): kind, rate, fall
/// speed. `wind` is the horizontal drift in world space
/// (tiles/second, x east / z south-in-world); rain slants with it,
/// snow drifts and sways on top of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecipState {
    pub kind: PrecipKind,
    /// Particles per second per tile column (0 = off).
    pub rate: f32,
    /// Fall-speed multiplier on [`BASE_FALL_SPEED`].
    pub fall_speed: f32,
    /// World-space horizontal wind [x, z], tiles/second.
    pub wind: [f32; 2],
}

impl PrecipState {
    pub fn new(kind: PrecipKind, rate: f32, fall_speed: f32) -> Self {
        PrecipState {
            kind,
            rate,
            fall_speed,
            wind: [0.0, 0.0],
        }
    }
    pub fn off() -> Self {
        PrecipState::new(PrecipKind::Rain, 0.0, 1.0)
    }
    pub fn enabled(&self) -> bool {
        self.rate > 0.0 && self.fall_speed > 0.0
    }
}

/// One live particle, before conversion to a GPU instance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecipParticle {
    /// Game tile column (x east, y south) it spawned in.
    pub column: [i32; 2],
    /// Slot within the column's spawner.
    pub slot: u32,
    /// Respawn cycle of this slot (increments every period).
    pub cycle: u64,
    /// World-space center (the instance center).
    pub center: Vec3,
    /// World-space velocity (tiles/second); y is the fall speed.
    pub velocity: Vec3,
    /// Per-particle shader seed.
    pub seed: f32,
}

/// The emitter: a cylinder of falling precipitation around a
/// camera target tile. Stateless — every query re-derives the
/// particle set from `(seed, time)`, so there is no per-frame
/// state to desynchronize between the windowed app and captures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecipEmitter {
    pub seed: u64,
    /// Cylinder radius in tiles around the target tile's center.
    pub radius_tiles: f32,
    /// Fall height in tiles: particles spawn this far above the
    /// target's z-level and recycle at the target's z-level.
    pub height_tiles: f32,
}

impl Default for PrecipEmitter {
    fn default() -> Self {
        PrecipEmitter {
            seed: 0x1055_EED0_2026_1123,
            radius_tiles: 7.0,
            height_tiles: 10.0,
        }
    }
}

/// Deterministic 0..1 hash of the emitter seed and integer keys —
/// splitmix-style mixing, no RNG state anywhere in the emitter.
fn hash01(seed: u64, a: i64, b: i64, c: u64, d: u64) -> f32 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for v in [a as u64, b as u64, c, d] {
        h ^= v.wrapping_add(0x9E37_79B9_7F4A_7C15);
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
    }
    // Top 24 bits -> [0, 1).
    (h >> 40) as f32 / (1u64 << 24) as f32
}

impl PrecipEmitter {
    /// Fall speed in tiles/second for a state.
    pub fn fall_speed(state: &PrecipState) -> f32 {
        BASE_FALL_SPEED * state.fall_speed
    }

    /// Seconds a particle spends falling the emitter height.
    pub fn lifetime(&self, state: &PrecipState) -> f32 {
        if !state.enabled() {
            return 0.0;
        }
        self.height_tiles / Self::fall_speed(state)
    }

    /// Slots per column: enough that the respawn period
    /// `slots / rate` covers the fall lifetime (rounded up).
    pub fn slots_per_column(&self, state: &PrecipState) -> u32 {
        if !state.enabled() {
            return 0;
        }
        (state.rate * self.lifetime(state)).ceil().max(1.0) as u32
    }

    /// Respawn period of one slot, seconds.
    fn period(&self, state: &PrecipState) -> f32 {
        self.slots_per_column(state) as f32 / state.rate
    }

    /// All tile columns in the cylinder whose tile the sampler
    /// reports sky-exposed, in deterministic (y, x) order. The
    /// sampler is called as `exposed(game_x, game_y, game_z)` at
    /// the target's z-level.
    pub fn columns(
        &self,
        target: [i32; 3],
        exposed: &dyn Fn(i32, i32, i32) -> bool,
    ) -> Vec<[i32; 2]> {
        let r = self.radius_tiles.ceil() as i32;
        let r2 = self.radius_tiles * self.radius_tiles;
        let mut out = Vec::new();
        for dy in -r..=r {
            for dx in -r..=r {
                if (dx * dx + dy * dy) as f32 > r2 {
                    continue;
                }
                let (gx, gy) = (target[0] + dx, target[1] + dy);
                if exposed(gx, gy, target[2]) {
                    out.push([gx, gy]);
                }
            }
        }
        out
    }

    /// Phase offset of one slot in one column, in [0, period).
    fn slot_offset(&self, state: &PrecipState, column: [i32; 2], slot: u32) -> f32 {
        hash01(
            self.seed,
            column[0] as i64,
            column[1] as i64,
            slot as u64,
            0x0FF5_E700,
        ) * self.period(state)
    }

    /// Exact number of spawns in `[t0, t1)` across all exposed
    /// columns (analytic over the slot schedule — the rate gate's
    /// yardstick, and the number a profiler would check in 10.6).
    pub fn spawn_count(
        &self,
        state: &PrecipState,
        target: [i32; 3],
        t0: f32,
        t1: f32,
        exposed: &dyn Fn(i32, i32, i32) -> bool,
    ) -> u64 {
        if !state.enabled() || t1 <= t0 {
            return 0;
        }
        let period = self.period(state) as f64;
        let mut total: u64 = 0;
        for column in self.columns(target, exposed) {
            for slot in 0..self.slots_per_column(state) {
                let o = self.slot_offset(state, column, slot) as f64;
                // Spawn times: o + k*period for k = 0, 1, 2, ...
                let first = ((t0 as f64 - o) / period).ceil().max(0.0) as i64;
                let past_end = ((t1 as f64 - o) / period).ceil() as i64;
                if past_end > first {
                    total += (past_end - first) as u64;
                }
            }
        }
        total
    }

    /// The live particles at `time` (seconds, same clock the fx
    /// harness pins). Pure function of the inputs.
    pub fn particles_at(
        &self,
        state: &PrecipState,
        target: [i32; 3],
        time: f32,
        exposed: &dyn Fn(i32, i32, i32) -> bool,
    ) -> Vec<PrecipParticle> {
        if !state.enabled() || time < 0.0 {
            return Vec::new();
        }
        let period = self.period(state);
        let lifetime = self.lifetime(state);
        let v = Self::fall_speed(state);
        let floor_y = target[2] as f32;
        let mut out = Vec::new();
        for column in self.columns(target, exposed) {
            for slot in 0..self.slots_per_column(state) {
                let o = self.slot_offset(state, column, slot);
                if time < o {
                    continue; // first cycle has not started yet
                }
                let cycle = ((time - o) / period).floor();
                let age = time - o - cycle * period;
                if age >= lifetime {
                    continue; // landed; waiting out the period
                }
                let cycle = cycle as u64;
                let (cx, cy) = (column[0] as i64, column[1] as i64);
                // Fresh in-tile spot per cycle, kept off the tile
                // edges so a streak never straddles two columns.
                let fx = 0.08 + 0.84 * hash01(self.seed, cx, cy, cycle, 1);
                let fy = 0.08 + 0.84 * hash01(self.seed, cx, cy, cycle, 2);
                let phase = hash01(self.seed, cx, cy, cycle, 3) * std::f32::consts::TAU;
                // Game tile -> world: (x + fx, z_up, -(y + fy)).
                let mut center = Vec3::new(
                    column[0] as f32 + fx,
                    floor_y + self.height_tiles - v * age,
                    -(column[1] as f32 + fy),
                );
                // Wind drift accumulates over the fall.
                center.x += state.wind[0] * age;
                center.z += state.wind[1] * age;
                if state.kind == PrecipKind::Snow {
                    // Motes sway; rain streaks do not.
                    center.x += (age * 0.9 + phase).sin() * 0.22;
                    center.z += (age * 0.7 + phase * 1.7).cos() * 0.16;
                }
                out.push(PrecipParticle {
                    column,
                    slot,
                    cycle,
                    center,
                    velocity: Vec3::new(state.wind[0], -v, state.wind[1]),
                    seed: hash01(self.seed, cx, cy, cycle, 9) * 37.0,
                });
            }
        }
        out
    }

    /// Field-pass instances at `time` — the emitter's deliverable
    /// for the Fields pass (Phase 10.6 seats them alongside the
    /// field volumes; `FRAME_ORDER` is unchanged).
    pub fn instances_at(
        &self,
        state: &PrecipState,
        target: [i32; 3],
        time: f32,
        exposed: &dyn Fn(i32, i32, i32) -> bool,
    ) -> Vec<FieldInstGpu> {
        self.particles_at(state, target, time, exposed)
            .iter()
            .map(|p| particle_instance(state.kind, p))
            .collect()
    }
}

/// Convert one particle to its field-pass instance. `size` is 1.0
/// for both kinds: the shader's per-class scale turns it into a
/// 0.045×0.62×0.045 streak (rain) or a 0.085 mote (snow).
pub fn particle_instance(kind: PrecipKind, p: &PrecipParticle) -> FieldInstGpu {
    let (base_color, density) = match kind {
        PrecipKind::Rain => (RAIN_COLOR, RAIN_DENSITY),
        PrecipKind::Snow => (SNOW_COLOR, SNOW_DENSITY),
    };
    FieldInstGpu {
        center: p.center.to_array(),
        size: 1.0,
        base_color,
        density,
        emissive: base_color,
        emissive_strength: 0.0,
        noise_scale: 1.0,
        noise_speed: 0.0,
        class_id: kind.class_id(),
        seed: p.seed,
        drift: p.velocity.to_array(),
        heat: 0.0,
        link: 0.0,
        _pad: [0.0; 3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(_x: i32, _y: i32, _z: i32) -> bool {
        true
    }
    fn none(_x: i32, _y: i32, _z: i32) -> bool {
        false
    }

    #[test]
    fn off_state_emits_nothing() {
        let e = PrecipEmitter::default();
        let s = PrecipState::off();
        assert!(e.particles_at(&s, [0, 0, 0], 100.0, &all).is_empty());
        assert_eq!(e.spawn_count(&s, [0, 0, 0], 0.0, 100.0, &all), 0);
        assert!(e.instances_at(&s, [0, 0, 0], 100.0, &none).is_empty());
    }

    #[test]
    fn slots_cover_lifetime() {
        let e = PrecipEmitter::default();
        for s in [
            PrecipState::new(PrecipKind::Rain, 40.0, 1.0),
            PrecipState::new(PrecipKind::Snow, 30.0, 0.35),
            PrecipState::new(PrecipKind::Rain, 120.0, 1.3),
        ] {
            assert!(
                e.period(&s) >= e.lifetime(&s),
                "period {} must cover lifetime {}",
                e.period(&s),
                e.lifetime(&s)
            );
        }
    }

    #[test]
    fn particles_stay_inside_cylinder_and_above_floor() {
        let e = PrecipEmitter::default();
        let s = PrecipState::new(PrecipKind::Rain, 40.0, 1.0);
        let target = [3, -2, 1];
        for p in e.particles_at(&s, target, 77.0, &all) {
            let dx = (p.column[0] - target[0]) as f32;
            let dy = (p.column[1] - target[1]) as f32;
            assert!(dx * dx + dy * dy <= e.radius_tiles * e.radius_tiles + 1.0);
            assert!(p.center.y > target[2] as f32, "above the floor");
            assert!(p.center.y <= target[2] as f32 + e.height_tiles);
        }
    }

    #[test]
    fn slots_recycle_with_fresh_cycles() {
        let e = PrecipEmitter::default();
        let s = PrecipState::new(PrecipKind::Rain, 40.0, 1.0);
        let p = e.period(&s);
        let at = |t: f32| e.particles_at(&s, [0, 0, 0], t, &all);
        // Every (column, slot) live one period later is exactly one
        // cycle further along.
        let a = at(500.0);
        let b = at(500.0 + p);
        assert_eq!(a.len(), b.len());
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(pa.column, pb.column);
            assert_eq!(pa.slot, pb.slot);
            assert_eq!(pb.cycle, pa.cycle + 1, "recycled, not respawned ad hoc");
        }
    }

    #[test]
    fn hash01_is_in_range_and_varies() {
        let mut seen = std::collections::HashSet::new();
        for i in 0..200u64 {
            let v = hash01(42, i as i64, 0, 0, 0);
            assert!((0.0..1.0).contains(&v), "hash01 {v}");
            seen.insert(v.to_bits());
        }
        assert!(seen.len() > 190, "hash must actually vary");
    }
}
