//! S6 gates (sim half): vehicles — T11 plus the World
//! vehicle APIs the S7 scenario DSL will wrap.
//!
//! The destructible_street car is a sim vehicle STORE entry
//! (origin + face_deg + (vpart, mount) parts), not authored
//! absolute positions: the wire's absolute part positions are
//! recomputed sim-side by the ported `precalc_mounts` /
//! `coord_translate` math (0.I-1 src/vehicle.cpp:3643/3598 +
//! src/tileray.cpp). At the scene's 23 degrees the tileray
//! precalc is the identity for this car's mounts, so each
//! part starts at origin + mount; at 90 degrees the mount map
//! is exactly (x, y) -> (-y, x).
//!
//! - T11: rotate the fixture car 23 -> 90 degrees and
//!   translate it: every part's absolute position moves
//!   rigidly (relative geometry preserved exactly), face_deg
//!   on the wire is the float 23.0 then 90.0 (never snapped),
//!   roof-flagged parts stay roof-flagged. (The engine-side
//!   stamp half lives in crates/engine/tests/
//!   sim_destruction_gate.rs.)
//! - damage_part: hp_frac drops; at 0 the broken flag sets.
//! - remove_part: the part leaves the store and the wire.
//! - The mirror invariant with vehicle mutations interleaved
//!   into the turn script: a mirror fed only the message
//!   stream (vehicle_updates / vehicle_removed applied
//!   wholesale, the protocol's own semantics) equals a fresh
//!   Snapshot after every flush.

use std::collections::HashMap;
use std::path::PathBuf;

use proto::{DeltaT, InputKind, InputT, MessageT, SnapshotT, VehicleT};
use sim::{DefRegistry, Kind, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

struct Driver {
    reg: DefRegistry,
    world: World,
    seq: u64,
    epoch: u64,
}

impl Driver {
    fn new(scene_name: &str) -> (Driver, SnapshotT) {
        let reg = DefRegistry::load(&defs_path()).expect("registry loads");
        let scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
        let mut world = World::from_scene(&reg, &scene).expect("world builds");
        let snap = world.snapshot(&reg);
        assert_eq!(snap.seq, 1, "initial snapshot seq");
        assert_eq!(snap.epoch, 1, "initial snapshot epoch");
        (
            Driver {
                reg,
                world,
                seq: 1,
                epoch: 1,
            },
            snap,
        )
    }

    fn key(&mut self, c: char) -> Vec<MessageT> {
        let msgs = self.world.apply_input(&self.reg, &key_input(c));
        for m in &msgs {
            match m {
                MessageT::Delta(d) => {
                    assert_eq!(d.seq, self.seq + 1, "delta seq");
                    assert_eq!(d.epoch, self.epoch, "delta epoch");
                    self.seq = d.seq;
                }
                MessageT::Snapshot(s) => {
                    assert_eq!(s.seq, self.seq + 1, "snapshot seq");
                    assert_eq!(s.epoch, self.epoch + 1, "snapshot epoch");
                    self.seq = s.seq;
                    self.epoch = s.epoch;
                }
                MessageT::LightUpdate(lu) => {
                    assert_eq!(lu.seq, self.seq + 1, "light seq");
                    self.seq = lu.seq;
                }
                MessageT::EnvironmentUpdate(e) => {
                    assert_eq!(e.seq, self.seq + 1, "env seq");
                    self.seq = e.seq;
                }
                MessageT::Effects(fx) => {
                    assert_eq!(fx.seq, self.seq + 1, "effects seq");
                    self.seq = fx.seq;
                }
                other => panic!("a turn produced an unexpected message: {other:?}"),
            }
        }
        msgs
    }

    fn delta(&mut self, c: char) -> Box<DeltaT> {
        let msgs = self.key(c);
        let mut deltas: Vec<Box<DeltaT>> = msgs
            .into_iter()
            .filter_map(|m| match m {
                MessageT::Delta(d) => Some(d),
                _ => None,
            })
            .collect();
        assert_eq!(deltas.len(), 1, "key {c:?}: expected exactly one Delta");
        deltas.pop().expect("one delta")
    }
}

/// Parts of a wire vehicle keyed by (vpart string, mount):
/// (abs_pos, hp_frac, flags).
fn parts_by_key(
    reg: &DefRegistry,
    v: &VehicleT,
) -> HashMap<(String, [i32; 3]), ([i32; 3], u8, u32)> {
    let mut out = HashMap::new();
    for p in v.parts.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
        let a = p.abs_pos.as_ref().expect("abs pos");
        let m = p.mount.as_ref().expect("mount");
        out.insert(
            (
                reg.string_of(Kind::VehiclePart, p.vpart).expect("vpart string").to_string(),
                [m.x, m.y, m.z],
            ),
            ([a.x, a.y, a.z], p.hp_frac, p.flags),
        );
    }
    out
}

fn vehicle_update<'a>(d: &'a DeltaT, id: u64) -> Option<&'a VehicleT> {
    d.vehicle_updates.as_ref()?.iter().find(|v| v.id == id)
}

const CAR: u64 = 1;
const ORIGIN: [i32; 3] = [17, 5, 0];

#[test]
fn t11_rotate_and_translate_fixture_car() {
    let (mut drv, snap) = Driver::new("destructible_street");
    let car0 = snap
        .vehicles
        .as_ref()
        .expect("snapshot carries vehicles")
        .iter()
        .find(|v| v.id == CAR)
        .expect("the fixture car")
        .clone();
    // face_deg crosses the wire as the exact float the scene
    // declared — 23.0, not a snapped/quantised neighbour.
    assert_eq!(car0.face_deg, 23.0, "face_deg is float-exact 23.0");
    let before = parts_by_key(&drv.reg, &car0);
    assert_eq!(before.len(), 13, "the car has 13 parts");
    // At 23 degrees the ported tileray precalc is the identity
    // for these mounts: abs == origin + mount.
    for ((_, mount), (abs, _, _)) in &before {
        assert_eq!(
            *abs,
            [ORIGIN[0] + mount[0], ORIGIN[1] + mount[1], ORIGIN[2] + mount[2]],
            "part at mount {mount:?} starts at origin + mount"
        );
    }
    // The roof part is roof-flagged (bit 2, vehicles::model
    // FLAG_ROOF) from the scene.
    let roof_key = ("roof".to_string(), [0, 0, 0]);
    assert_eq!(before[&roof_key].2 & 4, 4, "roof part carries FLAG_ROOF");

    // Rotate to 90 degrees through the World API; the change
    // rides the next turn's Delta as a wholesale vehicle
    // update.
    assert!(drv.world.set_vehicle_face(CAR, 90.0));
    let d = drv.delta('.');
    let car1 = vehicle_update(&d, CAR).expect("rotation rides a vehicle_update").clone();
    assert_eq!(car1.face_deg, 90.0, "face_deg is float-exact 90.0 after rotation");
    let after = parts_by_key(&drv.reg, &car1);
    assert_eq!(after.len(), before.len(), "rotation loses no parts");
    for ((name, mount), (abs, hp, flags)) in &after {
        // The 90-degree mount map is exactly (x, y) -> (-y, x).
        let want = [
            ORIGIN[0] - mount[1],
            ORIGIN[1] + mount[0],
            ORIGIN[2] + mount[2],
        ];
        assert_eq!(*abs, want, "part {name} at mount {mount:?} rotated rigidly");
        let (old_abs, old_hp, old_flags) = before[&(name.clone(), *mount)];
        assert_eq!((*hp, *flags), (old_hp, old_flags), "rotation keeps hp/flags");
        let _ = old_abs;
    }
    // Rigidity, stated pairwise: every part pair's relative
    // offset after rotation is the rotated before-offset —
    // relative geometry is preserved exactly.
    let keys: Vec<&(String, [i32; 3])> = before.keys().collect();
    for a in &keys {
        for b in &keys {
            let rel_before = [
                before[*a].0[0] - before[*b].0[0],
                before[*a].0[1] - before[*b].0[1],
            ];
            let rel_after = [
                after[*a].0[0] - after[*b].0[0],
                after[*a].0[1] - after[*b].0[1],
            ];
            assert_eq!(
                rel_after,
                [-rel_before[1], rel_before[0]],
                "relative geometry preserved for {a:?} vs {b:?}"
            );
        }
    }
    assert_eq!(after[&roof_key].2 & 4, 4, "roof part stays roof-flagged after rotation");
    assert_eq!(drv.world.vehicle_face(CAR), Some(90.0));

    // Translate by (3, -2): every absolute position shifts by
    // exactly that; facing untouched.
    assert!(drv.world.translate_vehicle(CAR, [3, -2, 0]));
    let d = drv.delta('.');
    let car2 = vehicle_update(&d, CAR).expect("translation rides a vehicle_update").clone();
    assert_eq!(car2.face_deg, 90.0, "translation does not change facing");
    let moved = parts_by_key(&drv.reg, &car2);
    for (key, (abs, _, _)) in &moved {
        let prev = after[key].0;
        assert_eq!(
            *abs,
            [prev[0] + 3, prev[1] - 2, prev[2]],
            "part {key:?} translated rigidly"
        );
    }
    assert_eq!(drv.world.vehicle_origin(CAR), Some([20, 3, 0]));
}

#[test]
fn vehicle_damage_breaks_and_remove_part() {
    let (mut drv, _snap) = Driver::new("destructible_street");
    let wire = drv.world.vehicle_wire(CAR).expect("car on the wire");
    let parts = parts_by_key(&drv.reg, &wire);
    let door_key = ("door".to_string(), [0, -1, 0]);
    let door_index = drv
        .world
        .vehicle_part_index(&drv.reg, CAR, door_key.0.as_str(), door_key.1)
        .expect("door part index");
    assert_eq!(parts[&door_key].1, 255, "door starts undamaged");

    // Damage 100: hp_frac drops, broken flag stays clear.
    assert!(drv.world.damage_vehicle_part(CAR, door_index, 100));
    let d = drv.delta('.');
    let v = vehicle_update(&d, CAR).expect("damage rides a vehicle_update");
    let p = parts_by_key(&drv.reg, v);
    assert_eq!(p[&door_key].1, 155, "hp_frac drops by the damage amount");
    assert_eq!(p[&door_key].2 & 2, 0, "not broken above 0 hp (FLAG_BROKEN bit 1)");

    // Damage past 0: hp_frac floors at 0 and the broken flag
    // sets.
    assert!(drv.world.damage_vehicle_part(CAR, door_index, 200));
    let d = drv.delta('.');
    let v = vehicle_update(&d, CAR).expect("breaking rides a vehicle_update");
    let p = parts_by_key(&drv.reg, v);
    assert_eq!(p[&door_key].1, 0, "hp_frac floors at 0");
    assert_ne!(p[&door_key].2 & 2, 0, "broken flag set at 0 hp");

    // Remove a wheel: the part leaves the store and the wire.
    let wheel_key = ("wheel".to_string(), [1, -1, 0]);
    let wheel_index = drv
        .world
        .vehicle_part_index(&drv.reg, CAR, wheel_key.0.as_str(), wheel_key.1)
        .expect("wheel part index");
    assert!(drv.world.remove_vehicle_part(CAR, wheel_index));
    let d = drv.delta('.');
    let v = vehicle_update(&d, CAR).expect("removal rides a vehicle_update");
    let p = parts_by_key(&drv.reg, v);
    assert_eq!(p.len(), 12, "one part fewer on the wire");
    assert!(!p.contains_key(&wheel_key), "the removed wheel is gone from the wire");
    assert!(
        drv.world.vehicle_part_index(&drv.reg, CAR, "wheel", [1, -1, 0]).is_none(),
        "the removed wheel is gone from the store"
    );
    // Unknown vehicle / part: clean refusal, no panic.
    assert!(!drv.world.damage_vehicle_part(999, 0, 10));
    assert!(!drv.world.remove_vehicle_part(CAR, 999));
    assert!(!drv.world.set_vehicle_face(999, 0.0));
}

// ---------------------------------------------------------------------
// Mirror with interleaved vehicle mutations
// ---------------------------------------------------------------------

#[derive(Debug, Default, PartialEq)]
struct Mirror {
    vehicles: HashMap<u64, VehicleT>,
    entities: HashMap<u64, proto::EntityT>,
}

fn mirror_from_snapshot(snap: &SnapshotT) -> Mirror {
    let mut m = Mirror::default();
    for v in snap.vehicles.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
        m.vehicles.insert(v.id, v.clone());
    }
    for e in snap.entities.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
        m.entities.insert(e.id, e.clone());
    }
    m
}

fn mirror_apply(m: &mut Mirror, msg: &MessageT) {
    match msg {
        MessageT::Snapshot(s) => *m = mirror_from_snapshot(s),
        MessageT::Delta(d) => {
            for v in d.vehicle_updates.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
                m.vehicles.insert(v.id, v.clone());
            }
            for id in d.vehicle_removed.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
                m.vehicles.remove(id);
            }
            for e in d.entity_updates.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
                m.entities.insert(e.id, e.clone());
            }
            for id in d.entity_removed.as_ref().map(|x| x.as_slice()).unwrap_or(&[]) {
                m.entities.remove(id);
            }
        }
        _ => {}
    }
}

#[test]
fn mirror_invariant_with_vehicle_mutations() {
    let (mut drv, snap) = Driver::new("destructible_street");
    let mut mirror = mirror_from_snapshot(&snap);
    let door_index = drv
        .world
        .vehicle_part_index(&drv.reg, CAR, "door", [0, -1, 0])
        .expect("door part");

    // Interleave turns and API mutations; after every flush
    // the mirror (fed only messages) must equal a fresh
    // Snapshot's vehicle/entity state.
    let mut flushes = 0usize;
    let mut check = |drv: &mut Driver, mirror: &mut Mirror, msgs: &[MessageT]| {
        for msg in msgs {
            mirror_apply(mirror, msg);
        }
        flushes += 1;
        let fresh = drv.world.build_snapshot(&drv.reg, 0, 0);
        assert_eq!(
            *mirror,
            mirror_from_snapshot(&fresh),
            "mirror diverged after flush {flushes}"
        );
    };
    for round in 0..20 {
        let msgs = drv.key('.');
        check(&mut drv, &mut mirror, &msgs);
        if round == 3 {
            assert!(drv.world.damage_vehicle_part(CAR, door_index, 30));
        }
        if round == 7 {
            assert!(drv.world.set_vehicle_face(CAR, 45.0));
        }
        if round == 11 {
            assert!(drv.world.translate_vehicle(CAR, [-1, 1, 0]));
        }
        if round == 15 {
            let wheel = drv
                .world
                .vehicle_part_index(&drv.reg, CAR, "wheel", [1, 1, 0])
                .expect("wheel part");
            assert!(drv.world.remove_vehicle_part(CAR, wheel));
        }
        let msgs = drv.key('.');
        check(&mut drv, &mut mirror, &msgs);
    }
    assert_eq!(flushes, 40);
}
