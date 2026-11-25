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
//! Colors are flat presentation values by part role. The textured
//! path assigns materials from the vpart defs' sprites through
//! the voxelizer, like every other material: the engine resolves
//! each part to a per-vpart block id ([`stamp_vehicle_mapped`])
//! and the shared texture-spec builder carries that id's layers —
//! that wiring is the engine's (scene_prep's `VehicleMaterials`).
//! A broken part is darkened toward black-red so damage reads at
//! a glance (gate 8.3's presentation half).

use std::collections::{BTreeMap, BTreeSet};

use mesher::{Block, ChunkMap, Pass, WorldScale, AIR_ID};

use crate::model::{PartModel, VehicleModel};

/// Block id for all vehicle-part blocks (the mesher's fake tile
/// ids end at 9; real material ids arrive with TileDefs).
pub const VEHICLE_BLOCK_ID: u32 = 1000;

/// The state subtile a part's texture selection uses, from its
/// exported flags — the canonical table in `mesher::connect`
/// (the game's vpart renderer: open wins over broken; a part
/// with neither flag has no state subtile and selects by its
/// base entry). The engine's spec builder turns this name into
/// the variant's side layer
/// (`build_texture_spec_state_variants`).
pub fn part_state_subtile(part: &crate::model::PartModel) -> Option<&'static str> {
    mesher::connect::state_subtile(part.open, part.broken)
}

fn part_block(model: &VehicleModel, index: usize, block_id: u32) -> Block {
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
        id: block_id,
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
///
/// Every block carries [`VEHICLE_BLOCK_ID`] — the flat path,
/// byte-for-byte the pre-materials stamping.
pub fn stamp_vehicle(
    map: &mut ChunkMap,
    model: &VehicleModel,
    level0_z: i32,
) -> (BTreeSet<[i32; 3]>, BTreeSet<[i32; 2]>) {
    stamp_vehicle_mapped(map, model, level0_z, &|_| VEHICLE_BLOCK_ID)
}

/// [`stamp_vehicle`] with a per-part block id, for the textured
/// path: the engine resolves each part's vpart to a material
/// block id and the stamped blocks carry it, so the shared
/// texture stamping assigns the vpart's sprite layers.
///
/// A tile's blocks are written once, from the tile's *displayed*
/// parts — the last body part and the last roof part in export
/// order. That is the game's own draw order (parts draw in
/// vector order, later parts on top, so the last part on a tile
/// is the visible one), and it makes the flat path's result
/// identical to the old per-part overwriting: the last writer
/// won there too.
pub fn stamp_vehicle_mapped(
    map: &mut ChunkMap,
    model: &VehicleModel,
    level0_z: i32,
    block_id: &dyn Fn(&PartModel) -> u32,
) -> (BTreeSet<[i32; 3]>, BTreeSet<[i32; 2]>) {
    stamp_vehicle_mapped_except(map, model, level0_z, block_id, &BTreeSet::new())
}

/// [`stamp_vehicle_mapped`] skipping the given absolute tiles
/// entirely (Stage-G: tiles a fine vehicle model covers must
/// not also stamp coarse — the fine model replaces them).
pub fn stamp_vehicle_mapped_except(
    map: &mut ChunkMap,
    model: &VehicleModel,
    level0_z: i32,
    block_id: &dyn Fn(&PartModel) -> u32,
    except: &BTreeSet<[i32; 3]>,
) -> (BTreeSet<[i32; 3]>, BTreeSet<[i32; 2]>) {
    let scale = map.scale;
    let per = scale.blocks_per_z_level as i32;
    // Per tile: (last body part index, last roof part index).
    let mut tiles: BTreeMap<[i32; 3], (Option<usize>, Option<usize>)> = BTreeMap::new();
    for (i, part) in model.parts.iter().enumerate() {
        let slot = tiles.entry(part.abs_pos).or_insert((None, None));
        if part.roof {
            slot.1 = Some(i);
        } else {
            slot.0 = Some(i);
        }
    }
    let mut occupied = BTreeSet::new();
    let mut dirtied = BTreeSet::new();
    for (pos, (body, roof)) in tiles {
        if except.contains(&pos) {
            continue;
        }
        let [tx, ty, tz] = pos;
        let level = tz - level0_z;
        if level < 0 {
            continue;
        }
        let base = scale.level_base_block(level as u32) as i32;
        let mut wrote = false;
        if let Some(i) = body {
            let block = part_block(model, i, block_id(&model.parts[i]));
            for bz in base..base + per - 1 {
                // lower blocks; top is the roof layer's
                let d = map.set_global_block(tx, ty, bz, block);
                wrote |= !d.is_empty();
                dirtied.extend(d);
            }
        }
        if let Some(i) = roof {
            let block = part_block(model, i, block_id(&model.parts[i]));
            let d = map.set_global_block(tx, ty, base + per - 1, block);
            wrote |= !d.is_empty();
            dirtied.extend(d);
        }
        if wrote {
            occupied.insert(pos);
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
