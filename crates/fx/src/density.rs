//! Density math (Phase 2b): the intensity→density mapping (gate
//! 2b.3) and the CPU reference implementation of the field-volume
//! noise/density model that the WGSL in `render.rs` mirrors.
//!
//! The CPU reference exists so gates can assert behavior (monotonic
//! density, volumetric falloff, displayed weighting) without a GPU.
//! It implements the same formulas as the shader — value-noise fbm,
//! radial silhouette falloff, class shaping — with its own integer
//! hash; it is a behavioral reference, not a bit-exact twin.

use glam::Vec3;

use crate::effect::{EffectDef, RenderClass};

/// Game intensity (1..=3) -> base density. Strictly monotonic by
/// construction; 0 (no field) maps to 0, values above 3 clamp to 3.
pub fn intensity_density(intensity: u8) -> f32 {
    match intensity {
        0 => 0.0,
        1 => 0.35,
        2 => 0.70,
        _ => 1.0,
    }
}

/// Density for a concrete effect: base curve × the type's extracted
/// density scale.
pub fn effective_density(def: &EffectDef, intensity: u8) -> f32 {
    intensity_density(intensity) * def.density_scale
}

/// Fraction of a background obscured by looking through `path` tiles
/// of volume at `density` (Beer–Lambert with the harness extinction
/// coefficient 4.0). Intensity-3 smoke (density 1.0 × scale 1.25)
/// obscures >99% through one tile; intensity-1 does not.
pub fn obscurance(density: f32, path_tiles: f32) -> f32 {
    1.0 - (-density * path_tiles * 4.0).exp()
}

// ---------------------------------------------------------------------
// Value noise + fbm (CPU reference; WGSL mirrors this structure)
// ---------------------------------------------------------------------

fn hash3(x: i32, y: i32, z: i32) -> f32 {
    let mut n: u32 = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (z as u32).wrapping_mul(1274126177);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^= n >> 16;
    (n & 1023) as f32 / 1023.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Trilinear value noise at `p` (one lattice cell per unit).
pub fn value_noise(p: Vec3) -> f32 {
    let base = p.floor();
    let f = p - base;
    let (x0, y0, z0) = (base.x as i32, base.y as i32, base.z as i32);
    let (fx, fy, fz) = (smooth(f.x), smooth(f.y), smooth(f.z));
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let c000 = hash3(x0, y0, z0);
    let c100 = hash3(x0 + 1, y0, z0);
    let c010 = hash3(x0, y0 + 1, z0);
    let c110 = hash3(x0 + 1, y0 + 1, z0);
    let c001 = hash3(x0, y0, z0 + 1);
    let c101 = hash3(x0 + 1, y0, z0 + 1);
    let c011 = hash3(x0, y0 + 1, z0 + 1);
    let c111 = hash3(x0 + 1, y0 + 1, z0 + 1);
    lerp(
        lerp(lerp(c000, c100, fx), lerp(c010, c110, fx), fy),
        lerp(lerp(c001, c101, fx), lerp(c011, c111, fx), fy),
        fz,
    )
}

/// Fractal Brownian motion, 4 octaves, lacunarity 2, gain 0.5.
/// Output roughly in [0, 1).
pub fn fbm(p: Vec3) -> f32 {
    let mut sum = 0.0f32;
    let mut amp = 0.5f32;
    let mut freq = 1.0f32;
    for _ in 0..4 {
        sum += amp * value_noise(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    sum / 0.9375 // normalize by total amplitude
}

/// Soft silhouette falloff inside a unit volume: `local` in [0,1]^3,
/// 1 at the center, 0 at and beyond the inscribed sphere's shell.
/// This is what makes a field read as a volume, not a flat quad
/// (gate 2b.2).
pub fn falloff(local: Vec3) -> f32 {
    let c = (local - Vec3::splat(0.5)) * 2.0;
    let d = c.length();
    let t = ((1.15 - d) / 0.9).clamp(0.0, 1.0);
    smooth(t)
}

/// Density at a point inside a field volume (CPU reference).
///
/// `local` is the point in unit-volume space ([0,1]^3, y up),
/// `time` is seconds (animation is time-based only — nothing per-turn
/// crosses the IPC), `weight` is the displayed-field composite weight
/// from `world::composite_weights` (1.0 for a lone field).
pub fn density_at(def: &EffectDef, local: Vec3, time: f32, intensity: u8, weight: f32) -> f32 {
    let drift = def.world_drift();
    // The noise pattern scrolls opposite the drift velocity so the
    // volume appears to move with its buoyancy/drift.
    let sample = local * def.noise_scale - drift * (time * def.noise_speed)
        + Vec3::splat(def.seed());
    let n = fbm(sample);
    // Patchiness: keep some structure even at low noise values.
    let shaped = (n * 1.4 - 0.25).clamp(0.0, 1.0);
    let mut d = effective_density(def, intensity) * weight * falloff(local) * (0.45 + 0.55 * shaped);
    // Class shaping (mirrors the WGSL):
    match def.render_class {
        // Flame burns denser at the base and flickers.
        RenderClass::Flame => {
            let flicker = 0.85 + 0.3 * value_noise(Vec3::new(def.seed(), time * 2.5, 0.0));
            d *= (1.25 - 0.55 * local.y) * flicker;
        }
        // Gas stays low: density falls off with height in the volume.
        RenderClass::Gas | RenderClass::Liquid => {
            d *= 1.15 - 0.75 * local.y;
        }
        _ => {}
    }
    d.max(0.0)
}

/// Emissive output at a point (for bloom): flame/energy flicker,
/// everything else returns its (usually zero) base emissive.
pub fn emissive_at(def: &EffectDef, time: f32) -> [f32; 3] {
    if def.emissive_strength <= 0.0 {
        return [0.0, 0.0, 0.0];
    }
    let flicker = match def.render_class {
        RenderClass::Flame => {
            0.8 + 0.45 * value_noise(Vec3::new(def.seed() * 3.0, time * 3.5, 7.0))
        }
        RenderClass::Energy => {
            0.9 + 0.2 * value_noise(Vec3::new(1.0, time * 5.0, def.seed()))
        }
        _ => 1.0,
    };
    let s = def.emissive_strength * flicker;
    [
        def.emissive_color[0] * s,
        def.emissive_color[1] * s,
        def.emissive_color[2] * s,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::demo_defs;

    #[test]
    fn fbm_is_bounded_and_deterministic() {
        for i in 0..50 {
            let p = Vec3::new(i as f32 * 0.37, i as f32 * 0.11, 1.0);
            let v = fbm(p);
            assert!((0.0..1.05).contains(&v), "fbm {v}");
            assert_eq!(v, fbm(p), "fbm must be deterministic");
        }
    }

    #[test]
    fn falloff_center_beats_corner() {
        assert_eq!(falloff(Vec3::splat(0.5)), 1.0);
        assert_eq!(falloff(Vec3::new(0.02, 0.02, 0.02)), 0.0);
    }

    #[test]
    fn density_center_beats_edge_for_smoke() {
        let smoke = demo_defs().into_iter().find(|d| d.field_id == "fd_smoke").unwrap();
        let center = density_at(&smoke, Vec3::splat(0.5), 1.25, 2, 1.0);
        let edge = density_at(&smoke, Vec3::new(0.02, 0.02, 0.02), 1.25, 2, 1.0);
        assert!(center > 0.0);
        assert_eq!(edge, 0.0);
        assert!(center > edge);
    }
}
