//! Mock game truth for the gate tests.
//!
//! This module plays the role of the game: it computes absolute
//! part positions the way the game does — pivot plus the mount
//! rotated by the facing, rounded to the tile grid (the shape of
//! `vehicle::precalc_mounts` + `bub_part_pos`). The rotation
//! convention here is the *fixture's own*; the engine under test
//! never sees the mounts-and-facing formula, only the resulting
//! absolute positions — which is exactly the protocol contract.
//! If the engine re-derived positions (or quantized the facing),
//! the gate tests comparing model vs. this truth would fail at
//! non-round facings like 7° and 23°.

use proto::{Vec3iT, VehiclePartStateT, VehicleT};

use crate::model::FLAG_ROOF;

pub const VPART_FRAME: u32 = 100;
pub const VPART_WHEEL: u32 = 101;
pub const VPART_ROOF_PART: u32 = 102;
pub const VPART_DOOR: u32 = 103;

/// Mounts of the fixture car, in the vehicle frame.
pub const MOUNTS: &[([i32; 3], u32, u32)] = &[
    // (mount, vpart, flags)
    ([0, 0, 0], VPART_FRAME, 0),
    ([1, 0, 0], VPART_FRAME, 0),
    ([2, 0, 0], VPART_FRAME, 0),
    ([0, 1, 0], VPART_WHEEL, 0),
    ([2, 1, 0], VPART_WHEEL, 0),
    ([0, -1, 0], VPART_WHEEL, 0),
    ([2, -1, 0], VPART_WHEEL, 0),
    ([1, 0, 1], VPART_ROOF_PART, FLAG_ROOF),
    ([1, 1, 0], VPART_DOOR, 0),
];

/// The game's answer for one part: absolute tile position of
/// `mount` for a vehicle whose pivot sits at `pivot` facing
/// `face_deg`. Rotation in the game plane (x east, y south);
/// z mounts ride above their rotated x/y.
pub fn truth_abs_pos(pivot: [i32; 3], mount: [i32; 3], face_deg: f32) -> [i32; 3] {
    let rad = (face_deg as f64).to_radians();
    let (c, s) = (rad.cos(), rad.sin());
    let x = mount[0] as f64 * c - mount[1] as f64 * s;
    let y = mount[0] as f64 * s + mount[1] as f64 * c;
    [
        pivot[0] + x.round() as i32,
        pivot[1] + y.round() as i32,
        pivot[2] + mount[2],
    ]
}

fn part_state(vpart: u32, abs: [i32; 3], mount: [i32; 3], flags: u32) -> VehiclePartStateT {
    // Generated wire structs are non-exhaustive outside the proto
    // crate: build via Default + assignment (the lighting crate's
    // test pattern).
    let mut p = VehiclePartStateT::default();
    p.vpart = vpart;
    p.abs_pos = Some(Vec3iT {
        x: abs[0],
        y: abs[1],
        z: abs[2],
    });
    p.mount = Some(Vec3iT {
        x: mount[0],
        y: mount[1],
        z: mount[2],
    });
    p.hp_frac = 255;
    p.flags = flags;
    p
}

/// One exported vehicle state, built from truth — this is what the
/// fork would put on the wire for the fixture car.
pub fn truth_vehicle(id: u64, pivot: [i32; 3], face_deg: f32) -> VehicleT {
    let mut v = VehicleT::default();
    v.id = id;
    v.name = Some("fixture car".into());
    v.face_deg = face_deg;
    v.parts = Some(
        MOUNTS
            .iter()
            .map(|(mount, vpart, flags)| {
                part_state(
                    *vpart,
                    truth_abs_pos(pivot, *mount, face_deg),
                    *mount,
                    *flags,
                )
            })
            .collect(),
    );
    v
}

/// A scripted drive: the pivot steps east then south while the
/// facing turns through a full circle in uneven steps — including
/// 7° and 23° (values a 15°-quantizing engine cannot represent)
/// and the 359° → 7° wrap. Returns successive exported states.
pub fn scripted_drive(id: u64) -> Vec<VehicleT> {
    let facings = [0.0f32, 7.0, 23.0, 45.0, 90.0, 179.0, 270.0, 359.0, 7.0];
    facings
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let pivot = [20 + i as i32, 20 + (i as i32) / 3, 0];
            truth_vehicle(id, pivot, *f)
        })
        .collect()
}
