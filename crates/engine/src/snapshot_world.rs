//! Snapshot-column world building (Phase 8 wiring): turn
//! per-tile terrain ids — the shape a Phase 3 Snapshot carries —
//! into the mesher's block world, applying the game's open-air
//! rule through `hardcases::floor_at` (gate 8.4b's engine half,
//! previously only proven at model level).
//!
//! The rule, per column:
//! - a terrain id that emits a block stamps its own block as the
//!   level's floor slab (bottom block of the z-level);
//! - `t_open_air` (and the reserved id 0) emit **nothing** — the
//!   floor at that level, if any, is the `roof` of the terrain
//!   below (`Floor::RoofOf`), stamped as that level's floor slab
//!   in the roof material;
//! - open air over a roof-less terrain, or over more open air,
//!   stamps nothing: a hole / shaft, never a fabricated slab.
//!
//! One builder feeds both renderers: the returned `ChunkMap`
//! meshes on the raster path and converts to the RT tree's
//! `VoxelWorld` unchanged, so a phantom floor (or a missing one)
//! would show in both. Block choice per def id is the caller's
//! (`block_for`): real scenes resolve ids through TileDefs +
//! the voxelizer materials; the gates use the fake-data blocks.

use hardcases::{floor_at, Column, DefTable, Floor};
use mesher::block::Block;
use mesher::chunk::ChunkMap;
use mesher::scale::WorldScale;

/// One vertical column of terrain def ids, bottom first:
/// `terrain[i]` is the tile at game z `z_min + i`.
#[derive(Debug, Clone)]
pub struct ColumnSpec {
    pub x: i32,
    pub y: i32,
    pub z_min: i32,
    pub terrain: Vec<u32>,
}

/// Build the block world for a set of terrain columns. Chunk
/// columns are created on demand for every column position;
/// levels run 0-based from each column's `z_min` (all columns in
/// one build must share a `z_min` — a Snapshot's columns do).
pub fn build_columns(
    table: &DefTable,
    columns: &[ColumnSpec],
    scale: WorldScale,
    block_for: &dyn Fn(u32) -> Block,
) -> ChunkMap {
    let mut map = ChunkMap::new(scale);
    if columns.is_empty() {
        return map;
    }
    let z_min = columns[0].z_min;
    let levels = columns.iter().map(|c| c.terrain.len()).max().unwrap_or(0);
    // Create every needed chunk column up front (stamp_vehicle's
    // contract is the same: chunks outside the map render
    // nothing, so the builder owns chunk creation).
    let mut coords: Vec<[i32; 2]> = columns
        .iter()
        .map(|c| {
            [
                c.x.div_euclid(mesher::chunk::CHUNK_W as i32),
                c.y.div_euclid(mesher::chunk::CHUNK_H as i32),
            ]
        })
        .collect();
    coords.sort_unstable();
    coords.dedup();
    for coord in coords {
        if !map.chunks.contains_key(&coord) {
            map.add_chunk(coord, levels);
        }
    }
    let per = scale.blocks_per_z_level as i32;
    for c in columns {
        debug_assert_eq!(c.z_min, z_min, "snapshot columns share one z_min");
        let column = Column {
            z_min: c.z_min,
            terrain: c.terrain.clone(),
        };
        for (i, &ter) in c.terrain.iter().enumerate() {
            let z = c.z_min + i as i32;
            let base = scale.level_base_block(i as u32) as i32;
            let floor_block = if hardcases::openair::emits_block(table, ter) {
                Some(block_for(ter))
            } else {
                match floor_at(table, &column, z) {
                    Floor::RoofOf(_, roof_id) => Some(block_for(roof_id)),
                    // Solid is unreachable here (emits_block was
                    // false); None = no floor at this level.
                    _ => None,
                }
            };
            if let Some(block) = floor_block {
                // Floor slab: the bottom block(s) of the level.
                let slab = scale.floor_slab_blocks.min(per as u32) as i32;
                for bz in base..base + slab.max(1) {
                    map.set_global_block(c.x, c.y, bz, block);
                }
            }
        }
    }
    map.clear_dirty();
    map
}
