//! The live vehicle set: snapshot/delta application.
//!
//! Semantics mirror the protocol's sync rules (plan §6): a Snapshot
//! replaces the world wholesale (new epoch), a Delta carries full
//! replacement states for the vehicles it updates (a `Vehicle` on
//! the wire is always the vehicle's complete current part list —
//! there is no per-part delta), and `vehicle_removed` deletes.
//! Because updates are wholesale, part-level damage/removal (gate
//! 8.3) needs no special case: a part the game broke arrives with
//! its broken flag set, and a part the game removed simply stops
//! appearing in the list.
//!
//! The store keeps the previous state of each vehicle alongside the
//! current one so [`crate::interp`] can sample between the last two
//! exported states. A vehicle that appears for the first time has
//! prev == current (it renders at its exported position, no slide
//! in from nowhere).

use std::collections::BTreeMap;

use proto::{DeltaT, SnapshotT};

use crate::model::VehicleModel;

#[derive(Debug, Default)]
pub struct VehicleStore {
    current: BTreeMap<u64, VehicleModel>,
    previous: BTreeMap<u64, VehicleModel>,
}

impl VehicleStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Full replacement (new epoch). Previous states are discarded:
    /// after a resync there is nothing meaningful to interpolate
    /// from, so prev == current for every vehicle.
    pub fn apply_snapshot(&mut self, snap: &SnapshotT) {
        self.current.clear();
        self.previous.clear();
        if let Some(vehicles) = &snap.vehicles {
            for v in vehicles {
                let m = VehicleModel::from_proto(v);
                self.previous.insert(m.id, m.clone());
                self.current.insert(m.id, m);
            }
        }
    }

    /// Incremental update. Returns the ids whose state changed
    /// (updated or removed), in sorted order.
    pub fn apply_delta(&mut self, delta: &DeltaT) -> Vec<u64> {
        let mut changed = Vec::new();
        if let Some(updates) = &delta.vehicle_updates {
            for v in updates {
                let m = VehicleModel::from_proto(v);
                if let Some(old) = self.current.insert(m.id, m.clone()) {
                    self.previous.insert(m.id, old);
                } else {
                    self.previous.insert(m.id, m);
                }
                changed.push(v.id);
            }
        }
        if let Some(removed) = &delta.vehicle_removed {
            for id in removed {
                if self.current.remove(id).is_some() {
                    self.previous.remove(id);
                    changed.push(*id);
                }
            }
        }
        changed.sort_unstable();
        changed.dedup();
        changed
    }

    pub fn get(&self, id: u64) -> Option<&VehicleModel> {
        self.current.get(&id)
    }

    /// The previous exported state of a vehicle (== current when the
    /// vehicle has only ever been seen once).
    pub fn previous(&self, id: u64) -> Option<&VehicleModel> {
        self.previous.get(&id)
    }

    pub fn ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.current.keys().copied()
    }

    pub fn len(&self) -> usize {
        self.current.len()
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }
}
