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
//!
//! Terrain wiring (2026-11-18) adds the terrain counterpart:
//! [`stamp_terrain`] seats the terrain materials' own prop
//! models (the voxelizer's terrain drive) in the same fine
//! layer, resolved by game id through the TileDefs — the
//! snapshot-path counterpart of `scene_prep`'s fixture-scene
//! terrain extension, which scans a built map for fake tile
//! ids instead.

use std::collections::{BTreeMap, BTreeSet};

use hardcases::{floor_at, Column, DefTable, Floor};
use mesher::block::{Block, Pass};
use mesher::chunk::ChunkMap;
use mesher::fine::{FineLayer, FineVolume, ModelCells, StampError, StampReport, FINE};
use mesher::scale::WorldScale;

use crate::materials::{ModelKind, PropModelData};
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
/// 9.0) — or, when the tile's ground is a stamped Ground
/// model, on that model's measured top (fix 5b, 2026-11-23;
/// see the seating code) — through
/// `mesher::fine::stamp_model_rotated` with the
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
        // Fix 5b (2026-11-23, Joey's option B — measure,
        // don't publish): if the tile's ground at this level
        // is a stamped Ground model, seat the furniture on
        // that model's measured top instead of the slab-top
        // origin. A Ground model airs the coarse slab and
        // tops out at its own height (10, or 16 for a mound),
        // so slab-top seating (32) left furniture floating
        // over the modelled floor. A Ground volume is the
        // SHALLOW volume seated at the level base (depth <
        // the level depth) — a full-depth base volume is a
        // tall Standing model (fix 5a), not ground. No such
        // volume: the coarse slab is really there, and the
        // slab-top origin stands. This requires terrain to
        // be stamped before furniture on the combined path.
        let base = scale.level_base_block(p.level) as i32 * FINE;
        let ground_top = layer
            .volumes_for_tile(tile)
            .iter()
            .find(|v| v.z0 == base && v.depth_z < layer.depth_z)
            .map(|v| v.z0 + v.depth_z);
        let z0 = ground_top.unwrap_or_else(|| fine_z_origin(scale, p.level));
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

// ---------------------------------------------------------------------
// Terrain models on the snapshot path (terrain wiring,
// 2026-11-18)
// ---------------------------------------------------------------------

/// Stamp the terrain models for a set of snapshot columns
/// into `layer` over the coarse world `map` (built by
/// [`build_columns`] from the same columns).
///
/// Each tile whose def emits a block resolves through the
/// TileDefs to its game id, and `resolve` supplies the
/// material's prop model + seating kind (the voxelizer
/// records the kind in the v2 `models` table — only it
/// knows which builder produced the model). Seating:
///
/// - [`ModelKind::Ground`]: the model IS the tile's floor.
///   It stamps at the level base in a volume only as deep
///   as the model, and the coarse floor slab
///   `build_columns` stamped for the tile is aired — the
///   model replaces it. (A full-depth volume at the base
///   would shadow the tile's furniture volume, whose
///   slab-top z-range overlaps it; the shallow volume
///   leaves the band above the model to the coarse grid.)
/// - [`ModelKind::Fence`]: the model is a half-segment.
///   One composed model per same-def neighbour mask (bits
///   W=1, N=2, E=4, S=8; turns W0 N1 E2 S3 — the fixture
///   path's convention) is built from rotated copies and
///   stamped at the slab-top origin, like furniture. A
///   fence tile with no same-def neighbour (mask 0) keeps
///   its coarse block and counts as skipped.
/// - [`ModelKind::Standing`]: stamped as-is at the
///   slab-top origin, like furniture — unless the model is
///   taller than the band above the slab (level depth minus
///   the slab: 64 cells at the default scale), in which
///   case it seats at the level base instead (fix 5a,
///   2026-11-23, Joey's call). A 96-cell wall then fills
///   its own level exactly (0..96) instead of extending
///   32 cells into the level above, and an 84-cell door
///   ends at 84, not 116. The coarse slab is NOT aired:
///   stamped fine cells win over the coarse grid, and the
///   slab fills the gaps of sparse tall models (a door
///   panel) instead of leaving a pit. Shorter Standing
///   models (the fixture tree, 40) keep the slab-top
///   origin unchanged.
///
/// Open-air tiles follow `build_columns`' floor rule:
/// where `floor_at` resolves `Floor::RoofOf`, the roof
/// material's model is a floor slab by construction, so
/// it seats as Ground at that level whatever kind it
/// carries.
///
/// Restamp semantics: a Standing/Fence stamp first drops
/// the tile's volume in its seating range (the furniture
/// rule — terrain and furniture share a tile's standing
/// slot at a level; last stamp wins). The two Standing
/// seating ranges (slab-top and level-base) overlap, so a
/// restamp drops the other seating's volume too. A Ground stamp
/// replaces only the volume at its own base origin, so it
/// never drops a furniture volume.
///
/// Errors: any candidate model that exceeds the tile
/// fails the whole batch with [`StampError::ExceedsTile`]
/// BEFORE anything is stamped (all models are validated
/// up front), leaving `layer` and `map` unchanged.
pub fn stamp_terrain<'m>(
    map: &mut ChunkMap,
    layer: &mut FineLayer,
    table: &DefTable,
    columns: &[ColumnSpec],
    scale: WorldScale,
    resolve: &dyn Fn(&str) -> Option<(&'m PropModelData, ModelKind)>,
    block_for: &dyn Fn([u8; 3]) -> Block,
) -> Result<StampReport, StampError> {
    use std::collections::HashMap;
    // (x, y, level index) -> def id, for fence neighbour
    // masks (same def = same material = connects).
    let mut def_at: HashMap<(i32, i32, usize), u32> = HashMap::new();
    for c in columns {
        for (i, &ter) in c.terrain.iter().enumerate() {
            def_at.insert((c.x, c.y, i), ter);
        }
    }
    // Resolve and validate every candidate first: a build
    // error must not half-stamp a batch.
    struct Job {
        x: i32,
        y: i32,
        level: u32,
        def: u32,
        kind: ModelKind,
        /// Same-def neighbour mask (Fence only).
        mask: u8,
    }
    let mut jobs: Vec<Job> = Vec::new();
    let mut skipped = 0usize;
    for c in columns {
        let column = Column {
            z_min: c.z_min,
            terrain: c.terrain.clone(),
        };
        for (i, &ter) in c.terrain.iter().enumerate() {
            let z = c.z_min + i as i32;
            let (def, roof_floor) = if hardcases::openair::emits_block(table, ter) {
                (ter, false)
            } else {
                match floor_at(table, &column, z) {
                    Floor::RoofOf(_, roof_id) => (roof_id, true),
                    // Solid is unreachable (emits_block was
                    // false); None = no floor at this level.
                    _ => continue,
                }
            };
            let Some(string_id) = table.def(def).and_then(|d| d.string_id.as_deref()) else {
                skipped += 1;
                continue;
            };
            let Some((model, kind)) = resolve(string_id) else {
                skipped += 1;
                continue;
            };
            let kind = if roof_floor { ModelKind::Ground } else { kind };
            let size = model.size;
            if size[0] > FINE as u32 || size[2] > FINE as u32 || size[1] > layer.depth_z as u32 {
                return Err(StampError::ExceedsTile { size });
            }
            let mask = if kind == ModelKind::Fence {
                let mut m = 0u8;
                if def_at.get(&(c.x - 1, c.y, i)) == Some(&def) {
                    m |= 1; // W
                }
                if def_at.get(&(c.x, c.y - 1, i)) == Some(&def) {
                    m |= 2; // N
                }
                if def_at.get(&(c.x + 1, c.y, i)) == Some(&def) {
                    m |= 4; // E
                }
                if def_at.get(&(c.x, c.y + 1, i)) == Some(&def) {
                    m |= 8; // S
                }
                if m == 0 {
                    skipped += 1;
                    continue;
                }
                m
            } else {
                0
            };
            jobs.push(Job {
                x: c.x,
                y: c.y,
                level: i as u32,
                def,
                kind,
                mask,
            });
        }
    }
    let mut report = StampReport {
        skipped,
        ..StampReport::default()
    };
    let mut touched: BTreeSet<[i32; 2]> = BTreeSet::new();
    let mut composed: HashMap<(u32, u8), PropModelData> = HashMap::new();
    for job in &jobs {
        let tile = (job.x, job.y);
        let string_id = table
            .def(job.def)
            .and_then(|d| d.string_id.as_deref())
            .expect("job defs resolved during validation");
        let (model, _) = resolve(string_id).expect("job models resolved during validation");
        match job.kind {
            ModelKind::Ground => {
                let z0 = scale.level_base_block(job.level) as i32 * FINE;
                let [w, h, d] = model.size;
                let mut volume = FineVolume::new(tile, z0, h as i32);
                let mut cells = 0usize;
                for my in 0..h {
                    for mz in 0..d {
                        for mx in 0..w {
                            if let Some(rgb) = model.get(mx, my, mz) {
                                volume.set(
                                    mx as i32,
                                    (FINE - d as i32) + mz as i32,
                                    my as i32,
                                    Some(block_for(rgb)),
                                );
                                cells += 1;
                            }
                        }
                    }
                }
                report.stamped_cells += cells;
                report.volumes += 1;
                report.memory_bytes += volume.memory_bytes();
                layer.insert(volume);
                // The model replaces the coarse floor
                // slab build_columns stamped for the tile.
                let base = scale.level_base_block(job.level) as i32;
                let slab = (scale.floor_slab_blocks.min(scale.blocks_per_z_level)).max(1) as i32;
                for bz in base..base + slab {
                    map.set_global_block(job.x, job.y, bz, Block::AIR);
                }
            }
            ModelKind::Fence | ModelKind::Standing => {
                // Fix 5a (2026-11-23): a Standing model
                // taller than the band above the slab seats
                // at the level base, so it fills its own
                // level instead of extending into the level
                // above. Fences always seat at the slab top.
                let band_above_slab =
                    layer.depth_z - scale.floor_slab_blocks as i32 * FINE;
                let z0 = if job.kind == ModelKind::Standing
                    && model.size[1] as i32 > band_above_slab
                {
                    scale.level_base_block(job.level) as i32 * FINE
                } else {
                    fine_z_origin(scale, job.level)
                };
                // Restamp: drop the tile's old standing
                // volume at this level (shared with
                // furniture, see the doc comment).
                layer.remove_tile_in_z_range(tile, z0, z0 + layer.depth_z);
                let effective;
                let model_ref = if job.kind == ModelKind::Fence {
                    effective = composed
                        .entry((job.def, job.mask))
                        .or_insert_with(|| {
                            let mut c = PropModelData {
                                size: model.size,
                                palette: model.palette.clone(),
                                voxels: vec![PropModelData::EMPTY; model.voxels.len()],
                            };
                            for (bit, turns) in [(1u8, 0u8), (2, 1), (4, 2), (8, 3)] {
                                if job.mask & bit != 0 {
                                    c.union_from(&model.rotated(turns));
                                }
                            }
                            c
                        });
                    &*effective
                } else {
                    model
                };
                let adapter = PropModelCells::new(model_ref, block_for);
                let r = mesher::fine::stamp_model_rotated(layer, tile, z0, &adapter, 0)?;
                report.stamped_cells += r.stamped_cells;
                report.volumes += r.volumes;
                report.memory_bytes += r.memory_bytes;
            }
        }
        touched.insert(ChunkMap::chunk_coord_of(job.x, job.y));
    }
    for coord in touched {
        map.refresh_chunk_fine(coord, layer);
    }
    Ok(report)
}

/// [`stamp_terrain`] fed from the wire messages directly:
/// the TileDefs and Snapshot the fork sends at connect.
/// This is [`build_snapshot_world`]'s terrain-model half —
/// call it on the map that function built, with the same
/// scale.
pub fn stamp_snapshot_terrain<'m>(
    map: &mut ChunkMap,
    layer: &mut FineLayer,
    defs: &proto::TileDefsT,
    snapshot: &proto::SnapshotT,
    scale: WorldScale,
    resolve: &dyn Fn(&str) -> Option<(&'m PropModelData, ModelKind)>,
    block_for: &dyn Fn([u8; 3]) -> Block,
) -> Result<StampReport, StampError> {
    let table = DefTable::from_tiledefs(defs);
    let columns = columns_from_snapshot(snapshot);
    stamp_terrain(map, layer, &table, &columns, scale, resolve, block_for)
}
