//! Presentation interpolation between two exported vehicle states.
//!
//! The game exports discrete states (one per sync point); the
//! renderer draws at display rate. Interpolation is presentation
//! only and is pinned to the exports at both ends: at alpha 0 the
//! render state equals the previous export exactly, at alpha 1 it
//! equals the current export exactly — gate 8.2's "no part drift"
//! is that endpoint property, re-checked at every step of a
//! scripted drive by the gate tests.
//!
//! Facing interpolates along the shortest arc. Angles are degrees
//! on the circle: 359° → 7° is a +8° turn, not a −352° spin.

use crate::model::{PartKey, VehicleModel};

/// Signed shortest arc from `from` to `to`, in degrees, in
/// (−180, 180]. This is the turn the vehicle actually makes.
pub fn shortest_arc_deg(from: f32, to: f32) -> f32 {
    let mut d = (to - from).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

/// Facing at `alpha` ∈ [0, 1] between two exported facings,
/// normalized to [0, 360). At alpha 0/1 the result equals the
/// respective export (up to normalization of the export itself,
/// which the game already sends in range).
pub fn lerp_angle_deg(from: f32, to: f32, alpha: f32) -> f32 {
    let a = alpha.clamp(0.0, 1.0);
    (from + shortest_arc_deg(from, to) * a).rem_euclid(360.0)
}

/// One part in render space: game tile coordinates as floats
/// (x east, y south, z up — the game's own axes; the renderer's
/// world-space conversion is the engine's one shared mapping,
/// applied at draw time, not here).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderPart {
    pub key: PartKey,
    pub pos: [f32; 3],
    pub hp_frac: u8,
    pub open: bool,
    pub broken: bool,
    pub roof: bool,
}

/// One vehicle in render space at one instant.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderVehicle {
    pub id: u64,
    pub face_deg: f32,
    pub parts: Vec<RenderPart>,
}

fn exact(p: [i32; 3]) -> [f32; 3] {
    [p[0] as f32, p[1] as f32, p[2] as f32]
}

/// Sample the render state of a vehicle between its previous and
/// current exports. Parts are matched by [`PartKey`]:
///
/// - present in both → position lerps between the two exports;
/// - only in current (fitted / appeared) → renders at its exported
///   position for the whole interval (no slide-in);
/// - only in previous (removed / broken off) → absent immediately:
///   gate 8.3 requires the model to update when the game removes a
///   part, and a fading wreck part is a presentation choice the
///   game never asked for.
///
/// State flags (open/broken/roof/hp) always come from the current
/// export — damage and door state are facts, not motion.
pub fn sample_vehicle(prev: &VehicleModel, curr: &VehicleModel, alpha: f32) -> RenderVehicle {
    assert_eq!(prev.id, curr.id, "cannot interpolate across vehicles");
    let a = alpha.clamp(0.0, 1.0);
    let parts = curr
        .parts
        .iter()
        .map(|cp| {
            let pos = match prev.part(cp.key) {
                Some(pp) if a <= 0.0 => exact(pp.abs_pos),
                Some(_pp) if a >= 1.0 => exact(cp.abs_pos),
                Some(pp) => {
                    let p0 = exact(pp.abs_pos);
                    let p1 = exact(cp.abs_pos);
                    [
                        p0[0] + (p1[0] - p0[0]) * a,
                        p0[1] + (p1[1] - p0[1]) * a,
                        p0[2] + (p1[2] - p0[2]) * a,
                    ]
                }
                None => exact(cp.abs_pos),
            };
            RenderPart {
                key: cp.key,
                pos,
                hp_frac: cp.hp_frac,
                open: cp.open,
                broken: cp.broken,
                roof: cp.roof,
            }
        })
        .collect();
    RenderVehicle {
        id: curr.id,
        face_deg: if a <= 0.0 {
            prev.face_deg.rem_euclid(360.0)
        } else if a >= 1.0 {
            curr.face_deg.rem_euclid(360.0)
        } else {
            lerp_angle_deg(prev.face_deg, curr.face_deg, a)
        },
        parts,
    }
}
