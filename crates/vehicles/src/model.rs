//! Wire → model decoding for one vehicle. No geometry math happens
//! here: absolute part positions are copied verbatim from the wire,
//! and facing is stored as the exact `f32` the game sent.

use proto::{VehiclePartStateT, VehicleT};

/// Part flag bit: the part is open (vehicle door / trunk).
/// Fork source: `part.open` (voxel_real_adapter.cpp).
pub const FLAG_OPEN: u32 = 1 << 0;
/// Part flag bit: the part is broken (`vehicle_part::is_broken()`).
pub const FLAG_BROKEN: u32 = 1 << 1;
/// Part flag bit: the part is a roof part (`vpart_info` ROOF flag).
/// Roof parts render in the roof layer, not the body layer — the
/// same split the game's renderer makes.
pub const FLAG_ROOF: u32 = 1 << 2;

/// Identity of one part across updates. A vehicle routinely carries
/// several parts with the same vpart id (four wheels), so identity
/// is (vpart id, mount) — the mount is the part's fixed slot in the
/// vehicle frame and does not change as the vehicle moves/turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartKey {
    pub vpart: u32,
    pub mount: [i32; 3],
}

/// One vehicle part, exactly as exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartModel {
    pub key: PartKey,
    /// Absolute tile position, computed game-side from the
    /// pre-rotated mounts. Copied verbatim — never re-derived.
    pub abs_pos: [i32; 3],
    /// Hit points as a 0–255 fraction of max (wire `hp_frac`).
    pub hp_frac: u8,
    pub open: bool,
    pub broken: bool,
    pub roof: bool,
}

impl PartModel {
    pub fn from_proto(p: &VehiclePartStateT) -> Self {
        let abs = p.abs_pos.as_ref();
        let mount = p.mount.as_ref();
        Self {
            key: PartKey {
                vpart: p.vpart,
                mount: [
                    mount.map_or(0, |m| m.x),
                    mount.map_or(0, |m| m.y),
                    mount.map_or(0, |m| m.z),
                ],
            },
            abs_pos: [
                abs.map_or(0, |a| a.x),
                abs.map_or(0, |a| a.y),
                abs.map_or(0, |a| a.z),
            ],
            hp_frac: p.hp_frac,
            open: p.flags & FLAG_OPEN != 0,
            broken: p.flags & FLAG_BROKEN != 0,
            roof: p.flags & FLAG_ROOF != 0,
        }
    }

    /// Damaged presentation state (gate 8.3): a part reads as damaged
    /// when the game marks it broken, or its hp fraction has fallen
    /// to half or below. Broken is the game's own verdict and wins
    /// over the hp heuristic.
    pub fn damaged(&self) -> bool {
        self.broken || self.hp_frac <= 127
    }
}

/// One vehicle, exactly as exported.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleModel {
    pub id: u64,
    pub name: String,
    /// Continuous facing in degrees, stored bit-exact as sent.
    /// Never quantized to any step (see crate docs).
    pub face_deg: f32,
    pub parts: Vec<PartModel>,
}

impl VehicleModel {
    pub fn from_proto(v: &VehicleT) -> Self {
        Self {
            id: v.id,
            name: v.name.clone().unwrap_or_default(),
            face_deg: v.face_deg,
            parts: v
                .parts
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(PartModel::from_proto)
                .collect(),
        }
    }

    pub fn part(&self, key: PartKey) -> Option<&PartModel> {
        self.parts.iter().find(|p| p.key == key)
    }

    /// Body-layer parts (everything that is not a roof part).
    pub fn body_parts(&self) -> impl Iterator<Item = &PartModel> {
        self.parts.iter().filter(|p| !p.roof)
    }

    /// Roof-layer parts (gate 8.3's roof / no-roof split).
    pub fn roof_parts(&self) -> impl Iterator<Item = &PartModel> {
        self.parts.iter().filter(|p| p.roof)
    }
}
