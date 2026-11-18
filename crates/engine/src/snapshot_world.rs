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
//!
//! Phase 9 (plan §11b) adds furniture stamping on top of the
//! columns this module builds: [`stamp_furniture`] stamps
//! prop models into a `mesher::fine::FineLayer` over the built
//! map (see that module for the fine-grid terms). `build_columns`
//! itself is unchanged and never writes a furniture block —
//! that absence IS the 9.0 suppression contract (below).

use std::collections::{BTreeMap, BTreeSet};

use hardcases::{floor_at, Column, DefTable, Floor};
use mesher::block::{Block, Pass};
use mesher::chunk::ChunkMap;
use mesher::fine::{FineLayer, ModelCells, StampError, StampReport, FINE};
use mesher::scale::WorldScale;

use crate::materials::PropModelData;
use crate::textures::PropColorSet;

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

// ---------------------------------------------------------------------
// Wire ingestion: proto Snapshot -> columns -> block world
// ---------------------------------------------------------------------

/// Turn a wire `Snapshot` (the fork exporter's shape: one
/// `SubmapBlock` per submap per z-level, 12x12 row-major terrain
/// arrays) into per-column specs for [`build_columns`].
///
/// Live blocks only: `memory_blocks` are a separate presentation
/// layer (terrain/furniture memory, never live state — the same
/// split `lighting::grid_from_snapshot` enforces), so they never
/// contribute columns here.
///
/// Every column spans the snapshot's full z range (the exporter's
/// window is rectangular: every submap in it is sent at every z),
/// with id 0 ("none") filling any z a column is absent at — all
/// columns therefore share one `z_min`, as `build_columns`
/// requires. Columns come out sorted by (x, y).
pub fn columns_from_snapshot(snapshot: &proto::SnapshotT) -> Vec<ColumnSpec> {
    let empty: Vec<proto::SubmapBlockT> = Vec::new();
    let blocks = snapshot.blocks.as_ref().unwrap_or(&empty);
    if blocks.is_empty() {
        return Vec::new();
    }
    let mut z_min = i32::MAX;
    let mut z_max = i32::MIN;
    for b in blocks {
        let z = b.origin.as_ref().map(|o| o.z).unwrap_or(0);
        z_min = z_min.min(z);
        z_max = z_max.max(z);
    }
    let mut by_tile: BTreeMap<(i32, i32), BTreeMap<i32, u32>> = BTreeMap::new();
    for b in blocks {
        let o = b
            .origin
            .as_ref()
            .map(|v| (v.x, v.y, v.z))
            .unwrap_or((0, 0, 0));
        let Some(terrain) = b.terrain.as_ref() else {
            continue;
        };
        for (i, &ter) in terrain.iter().take(144).enumerate() {
            let x = o.0 + (i % 12) as i32;
            let y = o.1 + (i / 12) as i32;
            by_tile.entry((x, y)).or_default().insert(o.2, ter);
        }
    }
    by_tile
        .into_iter()
        .map(|((x, y), levels)| {
            let mut terrain = vec![0u32; (z_max - z_min + 1) as usize];
            for (z, ter) in levels {
                terrain[(z - z_min) as usize] = ter;
            }
            ColumnSpec {
                x,
                y,
                z_min,
                terrain,
            }
        })
        .collect()
}

/// Build the block world directly from the two messages the fork
/// sends at connect: `TileDefs` (which carry each terrain's
/// `roof_result`) plus a `Snapshot`. This is [`build_columns`]
/// with its def table and columns derived from the wire data
/// instead of hand-built by the caller.
pub fn build_snapshot_world(
    defs: &proto::TileDefsT,
    snapshot: &proto::SnapshotT,
    scale: WorldScale,
    block_for: &dyn Fn(u32) -> Block,
) -> ChunkMap {
    let table = DefTable::from_tiledefs(defs);
    let columns = columns_from_snapshot(snapshot);
    build_columns(&table, &columns, scale, block_for)
}

// ---------------------------------------------------------------------
// Phase 9 (plan §11b, 9.2): furniture prop-model stamping
// ---------------------------------------------------------------------

/// One furniture placement to stamp: the game tile `(x, y)` at
/// 0-based z-level index `level` (the same indexing
/// `build_columns` feeds to `WorldScale::level_base_block`)
/// carries furniture id `furn_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FurniturePlacement {
    /// Game tile x (east).
    pub x: i32,
    /// Game tile y (south).
    pub y: i32,
    /// 0-based z-level index of the tile.
    pub level: u32,
    /// Furniture def id, resolved to a prop model by the
    /// caller's resolver.
    pub furn_id: u32,
    /// Facing in quarter turns (plan §11b, 9.7): the stamped
    /// model is rotated by `quarter_turns % 4` steps inside
    /// its tile (0 = front south, 1 = west, 2 = north,
    /// 3 = east — see `mesher::fine::stamp_model_rotated`).
    /// Callers with no facing information supply 0, which is
    /// exactly the pre-9.7 unrotated stamp. Multitile
    /// furniture stays per-tile: each tile's placement
    /// carries its own model and its own facing, and a stamp
    /// never writes outside its tile.
    pub quarter_turns: u8,
}

/// The absolute fine-z origin a level's prop volume starts
/// at: the level's floor-slab top, per plan 9.0 —
/// `(level_base_block(level) + floor_slab_blocks) * FINE`.
pub fn fine_z_origin(scale: WorldScale, level: u32) -> i32 {
    (scale.level_base_block(level) + scale.floor_slab_blocks) as i32 * FINE
}

/// The `mesher::fine::ModelCells` adapter over a
/// [`PropModelData`] plus a colour->Block resolver: `cell`
/// reads the model's RGB at `(x, y, z)` and hands it to the
/// resolver (see [`prop_block_for`] for the synthetic-material
/// resolver built on `textures::PropColorSet`).
pub struct PropModelCells<'a> {
    /// The furniture material's prop model.
    pub model: &'a PropModelData,
    /// Colour -> stamped block resolver.
    pub block_for: &'a dyn Fn([u8; 3]) -> Block,
}

impl<'a> PropModelCells<'a> {
    /// An adapter for `model` resolving colours via
    /// `block_for`.
    pub fn new(model: &'a PropModelData, block_for: &'a dyn Fn([u8; 3]) -> Block) -> Self {
        Self { model, block_for }
    }
}

impl ModelCells for PropModelCells<'_> {
    fn size(&self) -> [u32; 3] {
        self.model.size
    }

    fn cell(&self, x: u32, y: u32, z: u32) -> Option<Block> {
        self.model.get(x, y, z).map(|rgb| (self.block_for)(rgb))
    }
}

/// The synthetic-material colour resolver for
/// [`PropModelCells`]: a palette RGB becomes a `Block` whose
/// id is the colour's synthetic material id in `colors`
/// (under `base_id`), whose flat colour is the RGB itself,
/// opaque pass, with all three texture layers set to the
/// colour's solid layer (`first_layer` is the value of
/// `spec.layers.len()` before
/// `textures::extend_texture_spec_with_props` appended the
/// set — see `PropColorSet::layer_of`).
///
/// A colour missing from the set (the set was not built from
/// this model — a caller bug) yields the magenta unknown-id
/// placeholder at `base_id` with no texture layers, the same
/// fallback convention as `mesher::block_for_tile`.
pub fn prop_block_for<'a>(
    colors: &'a PropColorSet,
    base_id: u32,
    first_layer: u32,
) -> impl Fn([u8; 3]) -> Block + 'a {
    move |rgb: [u8; 3]| match (
        colors.id_of(base_id, rgb),
        colors.layer_of(first_layer, rgb),
    ) {
        (Some(id), Some(layer)) => Block {
            id,
            color: [
                rgb[0] as f32 / 255.0,
                rgb[1] as f32 / 255.0,
                rgb[2] as f32 / 255.0,
                1.0,
            ],
            pass: Pass::Opaque,
            tex_layers: Some([layer, layer, layer]),
        },
        _ => Block {
            id: base_id,
            color: [1.0, 0.0, 1.0, 1.0],
            pass: Pass::Opaque,
            tex_layers: None,
        },
    }
}

/// Stamp a batch of furniture placements into `layer` over
/// the coarse world `map` (built by [`build_columns`]).
///
/// For each placement whose `furn_id` resolves to a model
/// (via `resolve`), the model is stamped at the tile's fine
/// volume with origin [`fine_z_origin`] (floor-slab top, plan
/// 9.0) through `mesher::fine::stamp_model_rotated` with the
/// placement's `quarter_turns` facing (9.7; 0 = the pre-9.7
/// unrotated stamp), and the owning chunk's `has_fine` flag
/// is recomputed from the layer.
///
/// SUPPRESSION contract (9.0): a modelled furniture gets NO
/// coarse block. `build_columns` writes none (it stamps
/// terrain floors only) and this function writes none either —
/// the coarse grid above the floor slab at a modelled tile
/// stays all AIR, and the prop exists only in the fine layer.
/// Placements with no model are skipped and counted in the
/// report's `skipped`.
///
/// RESTAMP semantics (the 9.0 Deltas paragraph, at layer
/// level): every placement first drops the tile's existing
/// volume in its level's z-range, so re-running this for a
/// tile whose furniture changed replaces the old model —
/// and drops the volume entirely when the new furniture has
/// no model. Other levels' volumes on the tile survive.
///
/// Errors: any placement resolving to an out-of-tile model
/// fails the whole batch with [`StampError::ExceedsTile`]
/// BEFORE anything is stamped (all models are validated up
/// front), leaving `layer` and `map` unchanged.
pub fn stamp_furniture<'m>(
    map: &mut ChunkMap,
    layer: &mut FineLayer,
    placements: &[FurniturePlacement],
    scale: WorldScale,
    resolve: &dyn Fn(u32) -> Option<&'m dyn ModelCells>,
) -> Result<StampReport, StampError> {
    // Validate first: a build error must not half-stamp a
    // batch (stamp_model's own unchanged-on-error contract,
    // extended to the batch).
    for p in placements {
        if let Some(model) = resolve(p.furn_id) {
            let size = model.size();
            if size[0] > FINE as u32 || size[2] > FINE as u32 || size[1] > layer.depth_z as u32 {
                return Err(StampError::ExceedsTile { size });
            }
        }
    }
    let mut report = StampReport::default();
    let mut touched: BTreeSet<[i32; 2]> = BTreeSet::new();
    for p in placements {
        let tile = (p.x, p.y);
        let z0 = fine_z_origin(scale, p.level);
        // Restamp: drop the tile's old volume at this level.
        layer.remove_tile_in_z_range(tile, z0, z0 + layer.depth_z);
        match resolve(p.furn_id) {
            Some(model) => {
                let r = mesher::fine::stamp_model_rotated(layer, tile, z0, model, p.quarter_turns)?;
                report.stamped_cells += r.stamped_cells;
                report.volumes += r.volumes;
                report.memory_bytes += r.memory_bytes;
            }
            None => report.skipped += 1,
        }
        touched.insert(ChunkMap::chunk_coord_of(p.x, p.y));
    }
    for coord in touched {
        map.refresh_chunk_fine(coord, layer);
    }
    Ok(report)
}

/// Restamp ONE tile after its furniture changed (a delta):
/// exactly [`stamp_furniture`] over a single placement — the
/// old volume at that tile+level is dropped first, and the
/// volume disappears entirely when the new furniture has no
/// model.
pub fn restamp_tile<'m>(
    map: &mut ChunkMap,
    layer: &mut FineLayer,
    placement: &FurniturePlacement,
    scale: WorldScale,
    resolve: &dyn Fn(u32) -> Option<&'m dyn ModelCells>,
) -> Result<StampReport, StampError> {
    stamp_furniture(map, layer, std::slice::from_ref(placement), scale, resolve)
}
