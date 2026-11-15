//! The vehicle render model: a [`VehicleModel`] stamped into the
//! mesher's world as blocks.
//!
//! Placement is, again, verbatim from the export: a part occupies
//! its absolute tile. Within the tile's z-level (3 blocks at the
//! default [`WorldScale`]), body parts fill the lower blocks and
//! roof parts the top block — so a roof part sharing a tile with a
//! body part composes into a full column, and a lone roof part
//! renders as a top slab (the same shape language as the mesher's
//! own `TileShape::RoofSlab`). The split is driven by the part's
//! roof *flag*: in the real export every part of a ground vehicle
//! carries the vehicle's own z (the fork's adapter exports mount z
//! as 0), so the flag is the only roof signal there is.
//!
//! Colors are flat presentation values by part role (the textured
//! path assigns materials from the vpart defs' sprites through the
//! voxelizer, like every other material — that wiring is the
//! engine's, Phase 2h). A broken part is darkened toward black-red
//! so damage reads at a glance (gate 8.3's presentation half).

use std::collections::BTreeSet;

use mesher::{Block, ChunkMap, Pass, WorldScale, AIR_ID};

use crate::model::VehicleModel;

/// Block id for all vehicle-part blocks (the mesher's fake tile
/// ids end at 9; real material ids arrive with TileDefs).
pub const VEHICLE_BLOCK_ID: u32 = 1000;

fn part_block(model: &VehicleModel, index: usize) -> Block {
    let part = &model.parts[index];
    // Role by flags/key: roof parts are stamped from the roof
    // layer; wheels are the parts whose mount sits off the
    // centerline row of the fixture — in real data the role comes
    // from the vpart def, so here color is by (roof, broken) and a
    // per-part hue rotation keeps neighbouring parts separable in
    // captures.
    let base = if part.roof {
        [0.62, 0.30, 0.22, 1.0] // roof: rust-red sheet
    } else {
        [0.45, 0.48, 0.52, 1.0] // body: steel gray
    };
    let color = if part.broken {
        [base[0] * 0.45 + 0.18, base[1] * 0.4, base[2] * 0.4, 1.0]
    } else {
        base
    };
    Block {
        id: VEHICLE_BLOCK_ID,
        color,
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// Stamp every part of `model` into `map`. `level0_z` is the game
/// z that corresponds to chunk-column level 0 (columns store
/// levels 0-based; game z runs −10..=+10). Returns the occupied
/// game tiles (absolute, as exported) and the chunks dirtied.
///
/// Tiles whose chunk does not exist in the map are skipped and
/// not returned (the caller owns chunk streaming; a vehicle
/// outside the loaded bubble renders nothing, like any tile).
pub fn stamp_vehicle(
    map: &mut ChunkMap,
    model: &VehicleModel,
    level0_z: i32,
) -> (BTreeSet<[i32; 3]>, BTreeSet<[i32; 2]>) {
    let scale = map.scale;
    let per = scale.blocks_per_z_level as i32;
    let mut occupied = BTreeSet::new();
    let mut dirtied = BTreeSet::new();
    for (i, part) in model.parts.iter().enumerate() {
        let [tx, ty, tz] = part.abs_pos;
        let level = tz - level0_z;
        if level < 0 {
            continue;
        }
        let base = scale.level_base_block(level as u32) as i32;
        let (lo, hi) = if part.roof {
            (base + per - 1, base + per) // top block only
        } else {
            (base, base + per - 1) // lower blocks; top is the roof layer's
        };
        let block = part_block(model, i);
        let mut wrote = false;
        for bz in lo..hi {
            let d = map.set_global_block(tx, ty, bz, block);
            wrote |= !d.is_empty();
            dirtied.extend(d);
        }
        if wrote {
            occupied.insert(part.abs_pos);
        }
    }
    (occupied, dirtied)
}

/// Remove a previously stamped vehicle: write air back over the
/// blocks [`stamp_vehicle`] would occupy for `model`. Returns the
/// chunks dirtied.
pub fn clear_vehicle(
    map: &mut ChunkMap,
    model: &VehicleModel,
    level0_z: i32,
) -> BTreeSet<[i32; 2]> {
    let scale: WorldScale = map.scale;
    let per = scale.blocks_per_z_level as i32;
    let air = Block {
        id: AIR_ID,
        ..mesher::Block::AIR
    };
    let mut dirtied = BTreeSet::new();
    for part in &model.parts {
        let [tx, ty, tz] = part.abs_pos;
        let level = tz - level0_z;
        if level < 0 {
            continue;
        }
        let base = scale.level_base_block(level as u32) as i32;
        for bz in base..base + per {
            dirtied.extend(map.set_global_block(tx, ty, bz, air));
        }
    }
    dirtied
}
