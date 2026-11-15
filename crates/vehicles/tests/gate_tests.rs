//! Phase 8 gate tests 8.1–8.3 at model level (Part II §11).
//!
//! The in-game halves of these gates (driving a real vehicle in the
//! fork and comparing against `vehicle::abs_part_pos()` live) stay
//! blocked on 0.1/0.2 like every in-game gate; here the game's role
//! is played by `vehicles::fixtures`, a mock-truth generator with
//! the same shape as the real computation (pivot + pre-rotated
//! mounts → absolute tile positions).

use proto::{DeltaT, SnapshotT, VehiclePartStateT};
use vehicles::fixtures::{
    scripted_drive, truth_abs_pos, truth_vehicle, MOUNTS, VPART_DOOR, VPART_WHEEL,
};
use vehicles::{
    sample_vehicle, shortest_arc_deg, PartKey, VehicleModel, VehicleStore, FLAG_BROKEN, FLAG_OPEN,
};

fn model_of(v: &proto::VehicleT) -> VehicleModel {
    VehicleModel::from_proto(v)
}

/// Gate 8.1 — vehicle orientations: for every sampled facing the
/// model's part positions equal the game's absolute positions
/// exactly, and the facing is stored bit-exact (never quantized).
#[test]
fn gate_8_1_orientations_exact_at_all_sampled_facings() {
    let pivot = [20, 20, 0];
    // The gate's own sample set (Part II 8.1) plus the wrap value.
    for facing in [0.0f32, 7.0, 15.0, 23.0, 45.0, 90.0, 179.0, 270.0, 359.0] {
        let wire = truth_vehicle(7, pivot, facing);
        let model = model_of(&wire);
        assert_eq!(
            model.face_deg.to_bits(),
            facing.to_bits(),
            "facing {facing} must be stored bit-exact, never quantized"
        );
        for (mount, vpart, _) in MOUNTS {
            let key = PartKey {
                vpart: *vpart,
                mount: *mount,
            };
            let part = model.part(key).expect("part present");
            assert_eq!(
                part.abs_pos,
                truth_abs_pos(pivot, *mount, facing),
                "facing {facing}, mount {mount:?}: engine position must equal the game's"
            );
        }
    }
}

/// Gate 8.1 (counter-proof) — the reason facing is never quantized:
/// snapping 7° to the nearest 15° step changes at least one part's
/// tile. If a future change quantizes, this test's premise breaks
/// and 8.1 above starts failing with it.
#[test]
fn gate_8_1_quantizing_the_facing_would_move_parts() {
    let pivot = [20, 20, 0];
    let exact = truth_vehicle(7, pivot, 7.0);
    let snapped = truth_vehicle(7, pivot, 15.0);
    let a = model_of(&exact);
    let b = model_of(&snapped);
    let moved = a
        .parts
        .iter()
        .zip(b.parts.iter())
        .filter(|(x, y)| x.abs_pos != y.abs_pos)
        .count();
    assert!(
        moved > 0,
        "7° vs 15° must place some part on a different tile"
    );
}

/// Gate 8.2 — vehicle movement: along a scripted turning drive,
/// interpolated render states are exact at both endpoints of every
/// step (no drift — 8.1 re-checked at each interval boundary) and
/// stay on the segment in between.
#[test]
fn gate_8_2_scripted_drive_no_drift() {
    let states = scripted_drive(7);
    let models: Vec<VehicleModel> = states.iter().map(model_of).collect();
    for pair in models.windows(2) {
        let (prev, curr) = (&pair[0], &pair[1]);
        for alpha in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let render = sample_vehicle(prev, curr, alpha);
            assert_eq!(render.parts.len(), curr.parts.len());
            for rp in &render.parts {
                let p0 = prev.part(rp.key).unwrap().abs_pos;
                let p1 = curr.part(rp.key).unwrap().abs_pos;
                if alpha == 0.0 {
                    assert_eq!(rp.pos, [p0[0] as f32, p0[1] as f32, p0[2] as f32]);
                } else if alpha == 1.0 {
                    assert_eq!(
                        rp.pos,
                        [p1[0] as f32, p1[1] as f32, p1[2] as f32],
                        "endpoint must equal the export exactly (no drift)"
                    );
                } else {
                    for axis in 0..3 {
                        let (lo, hi) = if p0[axis] <= p1[axis] {
                            (p0[axis] as f32, p1[axis] as f32)
                        } else {
                            (p1[axis] as f32, p0[axis] as f32)
                        };
                        assert!(
                            rp.pos[axis] >= lo && rp.pos[axis] <= hi,
                            "mid-state must stay on the segment between exports"
                        );
                    }
                }
            }
        }
    }
}

/// Gate 8.2 — facing interpolation takes the shortest arc,
/// including across the 359° → 7° wrap (+8°, never −352°).
#[test]
fn gate_8_2_facing_shortest_arc_across_wrap() {
    assert_eq!(shortest_arc_deg(359.0, 7.0), 8.0);
    assert_eq!(shortest_arc_deg(7.0, 359.0), -8.0);
    assert_eq!(shortest_arc_deg(0.0, 179.0), 179.0);
    let states = scripted_drive(7);
    let models: Vec<VehicleModel> = states.iter().map(model_of).collect();
    let (prev, curr) = (&models[7], &models[8]); // 359° -> 7°
    let mid = sample_vehicle(prev, curr, 0.5);
    assert_eq!(
        mid.face_deg, 3.0,
        "halfway through the wrap is 3°, not 183°"
    );
    assert_eq!(sample_vehicle(prev, curr, 0.0).face_deg, 359.0);
    assert_eq!(sample_vehicle(prev, curr, 1.0).face_deg, 7.0);
}

/// Gate 8.3 — damage, removal, open state and the roof split all
/// arrive as ordinary wholesale updates and are reflected exactly.
#[test]
fn gate_8_3_damage_removal_open_and_roof_split() {
    let mut store = VehicleStore::new();
    let mut snap = SnapshotT::default();
    snap.vehicles = Some(vec![truth_vehicle(7, [20, 20, 0], 0.0)]);
    store.apply_snapshot(&snap);
    let v = store.get(7).unwrap();
    assert_eq!(
        v.roof_parts().count(),
        1,
        "the roof part is in the roof layer"
    );
    assert_eq!(v.body_parts().count(), v.parts.len() - 1);
    assert!(v.parts.iter().all(|p| !p.damaged()));

    // Damage the door (hp drop), break a wheel, open the door,
    // and remove the other front wheel — one ordinary update.
    let mut next = truth_vehicle(7, [20, 20, 0], 0.0);
    let parts = next.parts.as_mut().unwrap();
    parts.retain(|p| {
        !(p.vpart == VPART_WHEEL && p.mount.as_ref().is_some_and(|m| m.x == 0 && m.y == -1))
    });
    for p in parts.iter_mut() {
        let m = p.mount.as_ref().unwrap();
        if p.vpart == VPART_DOOR {
            p.hp_frac = 100;
            p.flags |= FLAG_OPEN;
        }
        if p.vpart == VPART_WHEEL && m.x == 0 && m.y == 1 {
            p.flags |= FLAG_BROKEN;
            p.hp_frac = 0;
        }
    }
    let mut delta = DeltaT::default();
    delta.vehicle_updates = Some(vec![next]);
    let changed = store.apply_delta(&delta);
    assert_eq!(changed, vec![7]);

    let v = store.get(7).unwrap();
    assert_eq!(
        v.parts.len(),
        MOUNTS.len() - 1,
        "removed part is gone from the model"
    );
    let door = v
        .part(PartKey {
            vpart: VPART_DOOR,
            mount: [1, 1, 0],
        })
        .unwrap();
    assert!(door.open, "open flag decoded");
    assert!(door.damaged(), "hp 100/255 reads as damaged");
    let wheel = v
        .part(PartKey {
            vpart: VPART_WHEEL,
            mount: [0, 1, 0],
        })
        .unwrap();
    assert!(
        wheel.broken && wheel.damaged(),
        "broken flag decoded and wins"
    );

    // Whole-vehicle removal.
    let mut delta = DeltaT::default();
    delta.vehicle_removed = Some(vec![7]);
    store.apply_delta(&delta);
    assert!(store.get(7).is_none());
    assert!(store.is_empty());
}

/// The Phase 1/3 sample vehicle (proto::samples) decodes into the
/// model with its wire values intact — the real protocol path, not
/// only the fixture path.
#[test]
fn sample_vehicle_decodes_verbatim() {
    let wire = proto::samples::sample_vehicle();
    let model = model_of(&wire);
    assert_eq!(model.id, 7);
    assert_eq!(model.face_deg.to_bits(), 37.5f32.to_bits());
    assert_eq!(model.parts.len(), 1);
    assert_eq!(model.parts[0].abs_pos, [20, 20, 0]);
    // The sample's flags value 4 is the ROOF bit per the fork's
    // assignment (voxel_real_adapter.cpp) — decoded as such.
    assert!(model.parts[0].roof);
    let _: &VehiclePartStateT = &wire.parts.as_ref().unwrap()[0];
}
