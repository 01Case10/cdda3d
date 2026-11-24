//! Phase 9 fine grid (plan §11b, design lock 9.0): per-tile
//! fine volumes, the fine layer that holds them, model
//! stamping, and the fine sampler — the unified read path over
//! (fine layer, coarse world).
//!
//! Terms, as locked in 9.0:
//!
//! - **Fine cell**: the world's sub-tile unit. [`FINE`] = 32
//!   cells span one block (tile) edge, so one game z-level
//!   (3 blocks at the default [`crate::WorldScale`]) is 96 fine
//!   cells tall. Fine coordinates are written `(fx, fy, fz)`:
//!   `fx` east, `fy` south, `fz` up, all in fine cells.
//! - **Fine volume**: the stamped cells of ONE tile at one
//!   z-range, stored as run-length columns (see
//!   [`FineVolume`]). Only tiles whose furniture carries a
//!   prop model get a volume; everything else stays coarse.
//! - **Fine layer**: the map from tile coordinates to fine
//!   volumes ([`FineLayer`]). It sits beside the coarse
//!   `ChunkMap`, never inside it.
//! - **Fine sampler** ([`FineSampler`]): resolves a fine cell
//!   to (a) its stamped cell when its tile has a fine volume
//!   covering it, else (b) its containing coarse block,
//!   expanded — the existing [`crate::VoxelSampler`] contract
//!   honoured at fine resolution.
//! - **Model cell**: one voxel of a prop model (1 sprite
//!   pixel). A model cell is exactly 1 fine cell — stamping is
//!   a COPY, never a resample or rescale (9.0), even though
//!   sprite-derived model heights are far below the 96-cell
//!   level depth.
//!
//! This module is deliberately engine-free: models arrive
//! through the [`ModelCells`] trait, which mirrors the layout
//! of the engine's `PropModelData` (model voxel index
//! `(y * d + z) * w + x`; model `z = d - 1` is the FRONT).
//!
//! Slice 2 (9.4 + 9.10) adds [`mesh_chunk_fine`], the raster
//! fine mesher. Its boundary rule and dispatch contract:
//!
//! - **Dispatch contract**: the caller (the engine's raster
//!   scene preparation) meshes a chunk with
//!   [`mesh_chunk_fine`] iff that chunk's
//!   [`crate::chunk::Chunk::has_fine`] flag is set (refreshed
//!   from the layer after stamping), and with the ordinary
//!   coarse `ChunkMap::mesh_chunk` otherwise. A chunk with no
//!   fine volumes is therefore meshed exactly as before
//!   Phase 9, and a scene in which nothing was stamped has no
//!   flagged chunks at all — its meshes are literally the
//!   pre-Phase-9 meshes.
//! - **Boundary rule (coarse side)**: a coarse block's face
//!   whose neighbour cell is AIR but lies in a tile holding a
//!   fine volume whose z-range covers the neighbour block's
//!   whole fine-z span is culled iff ALL 32×32 stamped fine
//!   cells of that volume on the face plane (the fine cells
//!   immediately across the plane from the block) are opaque;
//!   otherwise the coarse face is drawn (the prop does not
//!   fully cover it, so culling it would leave a hole).
//! - **Boundary rule (fine side)**: a stamped fine cell's
//!   faces are decided by [`FineSampler`], so a face against
//!   a coarse block resolves to that block expanded — a prop
//!   face against a wall or a floor slab is culled exactly
//!   when [`crate::mesher::face_visible`] says the coarse
//!   block hides it.

use std::collections::BTreeMap;
use std::fmt;

use crate::block::{Block, Pass};
use crate::chunk::{ChunkMap, CHUNK_H, CHUNK_W};
use crate::grid::VoxelSampler;
use crate::mesh::PassMeshes;
use crate::mesher::{
    face_visible, layer_for_dir, normal_to_world, quad_uvs, to_world, FACE_CORNERS, FACE_DIRS,
};
use crate::scale::WorldScale;

/// Fine cells per block (tile) edge — the Phase 9 resolution
/// lock (plan §11b, 9.0). A z-level is
/// `blocks_per_z_level * FINE` fine cells tall: 96 at the
/// default scale (3 blocks per level).
pub const FINE: i32 = 32;

/// One run in a fine volume's run-length column: `len`
/// consecutive fine cells (along fine z) carrying `block`
/// (`None` = no stamped cell in this run).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FineRun {
    /// Number of fine cells this run covers (always >= 1).
    pub len: u32,
    /// The stamped block for the whole run, or `None` for an
    /// unstamped (empty) run.
    pub block: Option<Block>,
}

/// A per-tile fine volume in run-length columns (plan §11b's
/// words): for each `(lx, ly)` column of the 32x32 tile
/// footprint, a `Vec` of [`FineRun`]s along fine z covering
/// `depth_z` cells starting at the volume's absolute fine-z
/// origin `z0`.
///
/// Props are mostly solid/empty runs, so a column is typically
/// 1-3 runs; the dense equivalent (32 * 32 * `depth_z` blocks)
/// is what [`FineVolume::memory_bytes`] is measured against in
/// the 9.1 gate.
///
/// Local coordinates: `lx`/`ly` are fine offsets within the
/// tile footprint (`0..FINE`, x east, y south); `fz` in
/// [`FineVolume::get`] is an ABSOLUTE fine z, matched against
/// `z0..z0 + depth_z`.
#[derive(Debug, Clone)]
pub struct FineVolume {
    /// Tile this volume belongs to, in game tile coordinates.
    pub tile: (i32, i32),
    /// Absolute fine-z origin: the first fine z this volume
    /// covers (the floor-slab top of its z-level when stamped
    /// by the engine, see `engine::snapshot_world`).
    pub z0: i32,
    /// Fine cells of z this volume covers (one z-level's fine
    /// depth: 96 at the default scale).
    pub depth_z: i32,
    /// Columns in `(ly * FINE + lx)` order; each column's runs
    /// sum to exactly `depth_z`.
    columns: Vec<Vec<FineRun>>,
}

impl FineVolume {
    /// An empty volume (every column one unstamped run) for
    /// `tile` at absolute fine-z origin `z0`, covering
    /// `depth_z` fine cells.
    pub fn new(tile: (i32, i32), z0: i32, depth_z: i32) -> Self {
        debug_assert!(depth_z > 0, "a fine volume covers at least one cell");
        let depth = depth_z.max(1);
        Self {
            tile,
            z0,
            depth_z: depth,
            columns: vec![
                vec![FineRun {
                    len: depth as u32,
                    block: None,
                }];
                (FINE * FINE) as usize
            ],
        }
    }

    fn column_index(lx: i32, ly: i32) -> usize {
        (ly * FINE + lx) as usize
    }

    /// The stamped block at local `(lx, ly)` and ABSOLUTE fine
    /// z `fz`, or `None` when the cell is unstamped or outside
    /// this volume (wrong column or z outside `z0..z0+depth_z`).
    pub fn get(&self, lx: i32, ly: i32, fz: i32) -> Option<Block> {
        if !(0..FINE).contains(&lx) || !(0..FINE).contains(&ly) {
            return None;
        }
        let off = fz - self.z0;
        if off < 0 || off >= self.depth_z {
            return None;
        }
        let mut rest = off;
        for run in &self.columns[Self::column_index(lx, ly)] {
            if rest < run.len as i32 {
                return run.block;
            }
            rest -= run.len as i32;
        }
        None
    }

    /// Set one cell during construction: local `(lx, ly)`,
    /// z OFFSET `off` within the volume (`0..depth_z`), to
    /// `block` (`None` clears). Splits/merges runs so the
    /// column stays a minimal run list covering `depth_z`.
    pub fn set(&mut self, lx: i32, ly: i32, off: i32, block: Option<Block>) {
        debug_assert!((0..FINE).contains(&lx) && (0..FINE).contains(&ly));
        debug_assert!(off >= 0 && off < self.depth_z);
        if !(0..FINE).contains(&lx) || !(0..FINE).contains(&ly) || off < 0 || off >= self.depth_z {
            return;
        }
        let col = &mut self.columns[Self::column_index(lx, ly)];
        // Find the run containing `off`.
        let mut start = 0i32;
        let mut idx = 0usize;
        while idx < col.len() {
            let end = start + col[idx].len as i32;
            if off < end {
                break;
            }
            start = end;
            idx += 1;
        }
        let run = col[idx];
        let run_end = start + run.len as i32;
        if run.block == block {
            return; // already carries the value; runs stay minimal
        }
        // Replace the run with (before?, single, after?).
        let mut replacement: Vec<FineRun> = Vec::with_capacity(3);
        if off > start {
            replacement.push(FineRun {
                len: (off - start) as u32,
                block: run.block,
            });
        }
        replacement.push(FineRun { len: 1, block });
        if off + 1 < run_end {
            replacement.push(FineRun {
                len: (run_end - off - 1) as u32,
                block: run.block,
            });
        }
        let at = idx;
        col.splice(at..at + 1, replacement);
        // Merge the (at most) single-cell run with equal-valued
        // neighbours so repeated sets stay run-minimal.
        merge_at(col, at.saturating_sub(1));
        merge_at(col, at);
        merge_at(col, at + 1);
    }

    /// Number of stamped (non-empty, non-air) cells in the
    /// volume.
    pub fn solid_count(&self) -> usize {
        let mut n = 0usize;
        for col in &self.columns {
            for run in col {
                if let Some(b) = run.block {
                    if !b.is_air() {
                        n += run.len as usize;
                    }
                }
            }
        }
        n
    }

    /// Actual heap footprint estimate: this struct, the column
    /// vector's capacity, and every column's run-vector
    /// capacity times `size_of::<FineRun>()`. This is the
    /// number the 9.1 gate compares against the dense
    /// equivalent (`32 * 32 * depth_z * size_of::<Block>()`).
    pub fn memory_bytes(&self) -> usize {
        let mut bytes = std::mem::size_of::<Self>()
            + self.columns.capacity() * std::mem::size_of::<Vec<FineRun>>();
        for col in &self.columns {
            bytes += col.capacity() * std::mem::size_of::<FineRun>();
        }
        bytes
    }
}

/// Merge runs `i` and `i + 1` of a column when they carry the
/// same block (helper for [`FineVolume::set`]).
fn merge_at(col: &mut Vec<FineRun>, i: usize) {
    if i + 1 < col.len() && col[i].block == col[i + 1].block {
        col[i].len += col[i + 1].len;
        col.remove(i + 1);
    }
}

/// Read access to a prop model's cells, engine-free (the
/// engine adapts its `PropModelData` onto this trait).
///
/// Layout mirrors `engine::materials::PropModelData`: size is
/// `[w, h, d]` — `w` = x east, `h` = y up, `d` = z depth —
/// the voxel index is `(y * d + z) * w + x`, and model
/// `z = d - 1` is the FRONT (the face the sprite shows).
pub trait ModelCells {
    /// `[width (x, east), height (y, up), depth (z)]` in
    /// model cells.
    fn size(&self) -> [u32; 3];
    /// The block for model cell `(x, y, z)`, or `None` for an
    /// empty cell. `y = 0` is the object's bottom.
    fn cell(&self, x: u32, y: u32, z: u32) -> Option<Block>;
}

/// Why a stamp was refused. On error the target layer is
/// unchanged (validation happens before any mutation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampError {
    /// The model exceeds its tile footprint (width or depth
    /// over [`FINE`]) or the level's fine depth (height) —
    /// a build error per 9.0, never a silent clip.
    ExceedsTile {
        /// The offending model's `[w, h, d]`.
        size: [u32; 3],
    },
}

impl fmt::Display for StampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StampError::ExceedsTile { size } => write!(
                f,
                "prop model size {size:?} exceeds its tile footprint \
                 ({FINE}x{FINE} cells) or the level fine depth"
            ),
        }
    }
}

impl std::error::Error for StampError {}

/// What a stamp (or a batch of stamps) wrote.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StampReport {
    /// Model cells copied into fine volumes.
    pub stamped_cells: usize,
    /// Fine volumes inserted.
    pub volumes: usize,
    /// Sum of the inserted volumes' `memory_bytes()`.
    pub memory_bytes: usize,
    /// Placements skipped because they resolved to no model
    /// (only the engine's batch stamper fills this in;
    /// [`stamp_model`] always reports 0).
    pub skipped: usize,
}

/// The fine layer: tile coordinates -> that tile's fine
/// volumes. A tile can hold volumes at multiple z-ranges
/// (usually one — one modelled furniture per tile per level);
/// volumes are keyed additionally by their `z0` for lookup
/// and restamping.
///
/// The layer also owns the level fine depth used to bound
/// model heights at stamp time (`depth_z`: 96 at the default
/// scale — pass `scale.blocks_per_z_level * FINE` for other
/// scales; [`FineLayer::default`] is the default-scale 96).
#[derive(Debug, Clone)]
pub struct FineLayer {
    /// Fine cells of z in one z-level; stamped volumes cover
    /// exactly this depth and models taller than it are a
    /// [`StampError::ExceedsTile`].
    pub depth_z: i32,
    /// BTreeMap so every iteration (chunk scans, reports) is
    /// in deterministic tile order.
    volumes: BTreeMap<(i32, i32), Vec<FineVolume>>,
}

impl Default for FineLayer {
    fn default() -> Self {
        // Default WorldScale: 3 blocks per z-level.
        Self::new(3 * FINE)
    }
}

impl FineLayer {
    /// A layer whose z-level fine depth is `depth_z` cells.
    pub fn new(depth_z: i32) -> Self {
        Self {
            depth_z: depth_z.max(1),
            volumes: BTreeMap::new(),
        }
    }

    /// Insert a volume. A volume with the same tile AND the
    /// same `z0` as an existing one replaces it (restamping a
    /// level never leaves two volumes claiming the same
    /// cells); a different `z0` stacks beside it.
    pub fn insert(&mut self, volume: FineVolume) {
        let entry = self.volumes.entry(volume.tile).or_default();
        if let Some(slot) = entry.iter_mut().find(|v| v.z0 == volume.z0) {
            *slot = volume;
        } else {
            entry.push(volume);
        }
    }

    /// All volumes held for `tile` (empty slice when none).
    pub fn volumes_for_tile(&self, tile: (i32, i32)) -> &[FineVolume] {
        self.volumes.get(&tile).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// The volume covering absolute fine coordinate
    /// `(fx, fy, fz)`: the volume of the containing tile whose
    /// z-range covers `fz`, if any.
    pub fn volume_at(&self, fx: i32, fy: i32, fz: i32) -> Option<&FineVolume> {
        let tile = (fx.div_euclid(FINE), fy.div_euclid(FINE));
        self.volumes
            .get(&tile)?
            .iter()
            .find(|v| fz >= v.z0 && fz < v.z0 + v.depth_z)
    }

    /// The stamped block at absolute fine coordinate
    /// `(fx, fy, fz)`, or `None` when no volume covers it or
    /// the covered cell is unstamped. Exact for negative
    /// coordinates (`div_euclid` / `rem_euclid`).
    pub fn block_at_fine(&self, fx: i32, fy: i32, fz: i32) -> Option<Block> {
        let vol = self.volume_at(fx, fy, fz)?;
        vol.get(fx.rem_euclid(FINE), fy.rem_euclid(FINE), fz)
    }

    /// Drop EVERY volume of `tile` (the delta/restamp drop:
    /// the tile's furniture went away or changed). Returns how
    /// many volumes were removed.
    pub fn remove_tile(&mut self, tile: (i32, i32)) -> usize {
        self.volumes.remove(&tile).map(|v| v.len()).unwrap_or(0)
    }

    /// Drop `tile`'s volumes whose z-range OVERLAPS
    /// `z_lo..z_hi` (the level-scoped restamp drop: other
    /// levels' volumes on the same tile survive). Returns how
    /// many volumes were removed.
    pub fn remove_tile_in_z_range(&mut self, tile: (i32, i32), z_lo: i32, z_hi: i32) -> usize {
        let Some(entry) = self.volumes.get_mut(&tile) else {
            return 0;
        };
        let before = entry.len();
        entry.retain(|v| v.z0 + v.depth_z <= z_lo || v.z0 >= z_hi);
        let removed = before - entry.len();
        if entry.is_empty() {
            self.volumes.remove(&tile);
        }
        removed
    }

    /// Tiles holding at least one volume inside chunk column
    /// `chunk` (chunk coords are tile coords / 12, the
    /// `ChunkMap` convention), in ascending tile order.
    pub fn tiles_in_chunk(&self, chunk: [i32; 2]) -> Vec<(i32, i32)> {
        self.volumes
            .keys()
            .filter(|t| {
                t.0.div_euclid(CHUNK_W as i32) == chunk[0]
                    && t.1.div_euclid(CHUNK_H as i32) == chunk[1]
            })
            .copied()
            .collect()
    }

    /// Whether any volume's tile lies in chunk column `chunk`
    /// — the [`crate::chunk::Chunk::has_fine`] flag's source
    /// of truth.
    pub fn chunk_has_fine(&self, chunk: [i32; 2]) -> bool {
        self.volumes.keys().any(|t| {
            t.0.div_euclid(CHUNK_W as i32) == chunk[0] && t.1.div_euclid(CHUNK_H as i32) == chunk[1]
        })
    }

    /// Number of tiles holding at least one volume.
    pub fn tile_count(&self) -> usize {
        self.volumes.len()
    }

    /// Every tile holding at least one volume, in ascending
    /// tile order (the map is a `BTreeMap`).
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.volumes.keys().copied()
    }

    /// Whether the layer holds no volumes at all.
    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }

    /// Total `memory_bytes()` over all volumes (the fine
    /// layer's heap footprint estimate).
    pub fn memory_bytes(&self) -> usize {
        self.volumes
            .values()
            .flat_map(|vs| vs.iter())
            .map(|v| v.memory_bytes())
            .sum()
    }
}

/// Stamp `model` into `layer` as tile `tile`'s fine volume at
/// absolute fine-z origin `fine_z0`.
///
/// Mapping (the v2 contract in `engine::materials`): model
/// cell `(mx, my, mz)` lands at fine coordinate
/// `(tile_x * FINE + mx, tile_y * FINE + (FINE - d) + mz,
/// fine_z0 + my)` — the model's FRONT (`mz = d - 1`) lands
/// flush on the tile's SOUTH edge (local fine y = `FINE - 1`),
/// model `y = 0` is the bottom, and a model cell is exactly
/// one fine cell (copy, never resample — 9.0).
///
/// Errors: a model with `w > FINE`, `d > FINE`, or
/// `h > layer.depth_z` returns [`StampError::ExceedsTile`]
/// and leaves `layer` unchanged.
pub fn stamp_model(
    layer: &mut FineLayer,
    tile: (i32, i32),
    fine_z0: i32,
    model: &dyn ModelCells,
) -> Result<StampReport, StampError> {
    stamp_model_rotated(layer, tile, fine_z0, model, 0)
}

/// Stamp `model` into `layer` as tile `tile`'s fine volume at
/// absolute fine-z origin `fine_z0`, rotated within the tile
/// footprint by `quarter_turns` quarter turns (plan §11b,
/// 9.7).
///
/// The base mapping is [`stamp_model`]'s, unchanged: model
/// cell `(mx, my, mz)` first lands at tile-local
/// `(lx, ly) = (mx, (FINE - d) + mz)` with height `my`. Each
/// cell's `(lx, ly)` is then rotated by `quarter_turns % 4`
/// steps of `(lx, ly) -> (FINE - 1 - ly, lx)` inside the
/// 32x32 tile footprint; height is untouched, so one step
/// turns the model's front from the tile's SOUTH edge to its
/// WEST edge, two steps put it on the NORTH edge, and three
/// on the EAST edge. `quarter_turns = 0` (or any multiple of
/// 4) is exactly [`stamp_model`].
///
/// Errors: exactly [`stamp_model`]'s — a model with
/// `w > FINE`, `d > FINE`, or `h > layer.depth_z` returns
/// [`StampError::ExceedsTile`] and leaves `layer` unchanged,
/// at every turn count (rotation never changes which models
/// fit: a footprint that fits unrotated fits rotated, since
/// both axes are bounded by the same `FINE`).
pub fn stamp_model_rotated(
    layer: &mut FineLayer,
    tile: (i32, i32),
    fine_z0: i32,
    model: &dyn ModelCells,
    quarter_turns: u8,
) -> Result<StampReport, StampError> {
    let [w, h, d] = model.size();
    if w > FINE as u32 || d > FINE as u32 || h > layer.depth_z as u32 {
        return Err(StampError::ExceedsTile { size: [w, h, d] });
    }
    let turns = quarter_turns % 4;
    let mut volume = FineVolume::new(tile, fine_z0, layer.depth_z);
    let mut cells = 0usize;
    // y (up) outermost so each column is filled bottom-up.
    for my in 0..h {
        for mz in 0..d {
            for mx in 0..w {
                if let Some(block) = model.cell(mx, my, mz) {
                    let mut lx = mx as i32;
                    let mut ly = (FINE - d as i32) + mz as i32;
                    for _ in 0..turns {
                        (lx, ly) = (FINE - 1 - ly, lx);
                    }
                    volume.set(lx, ly, my as i32, Some(block));
                    cells += 1;
                }
            }
        }
    }
    let memory_bytes = volume.memory_bytes();
    layer.insert(volume);
    Ok(StampReport {
        stamped_cells: cells,
        volumes: 1,
        memory_bytes,
        skipped: 0,
    })
}

/// The fine sampler: the unified read path over a coarse
/// [`VoxelSampler`] (in practice the world `ChunkMap`) plus a
/// [`FineLayer`]. Slice 2's fine mesher reads through this.
#[derive(Clone, Copy)]
pub struct FineSampler<'a> {
    /// The coarse world (block-resolution sampler).
    pub coarse: &'a dyn VoxelSampler,
    /// The fine layer laid over it.
    pub layer: &'a FineLayer,
}

impl<'a> FineSampler<'a> {
    /// A sampler over `coarse` with `layer` stamped on top.
    pub fn new(coarse: &'a dyn VoxelSampler, layer: &'a FineLayer) -> Self {
        Self { coarse, layer }
    }

    /// Resolve absolute fine coordinate `(fx, fy, fz)`:
    /// (a) the stamped cell, when the containing tile has a
    /// fine volume covering `fz` with a stamped cell there;
    /// (b) otherwise the containing coarse block, expanded —
    /// `coarse.block_at(fx / FINE, fy / FINE, fz / FINE)` with
    /// Euclidean division, exact for negative coordinates.
    pub fn block_at_fine(&self, fx: i32, fy: i32, fz: i32) -> Block {
        if let Some(b) = self.layer.block_at_fine(fx, fy, fz) {
            return b;
        }
        self.coarse.block_at(
            fx.div_euclid(FINE),
            fy.div_euclid(FINE),
            fz.div_euclid(FINE),
        )
    }

    /// Whether fine cell `(fx, fy, fz)` is a STAMPED cell
    /// (case (a) of [`FineSampler::block_at_fine`]) as opposed
    /// to a coarse-expanded one — the distinction slice 2's
    /// mesher needs at fine/coarse boundaries.
    pub fn is_stamped(&self, fx: i32, fy: i32, fz: i32) -> bool {
        self.layer.block_at_fine(fx, fy, fz).is_some()
    }
}

/// Geometric occlusion for fine props (Stage H, Joey's
/// 8c-A, 2026-11-23 — see `docs/phase9-fine.md` §1, where
/// this was the named candidate). The game's light truth
/// is per-tile, so this is INVENTED lighting: a plausible
/// darkening computed from the stamped geometry itself,
/// nothing else. For a visible face of the stamped cell at
/// `(fx, fy, fz)` looking along `dir`, let `a` be the air
/// cell in front of the face. The occluders are the opaque
/// cells beside `a` in the face plane (4: for a side face
/// those are the cells above, below, and to both sides of
/// `a` — so a face under a prop overhang or on the floor
/// picks up its shelf/floor contact; for a top face, the
/// cells around `a` at its own height) plus the cell
/// straight ahead of the face at `a + dir` (a wall one
/// cell away closes the pocket). Each occluder costs 0.11
/// of light, floored at 0.56: a face in the open keeps
/// 1.0 exactly (unoccluded scenes mesh byte-identically
/// to the pre-pass mesher), a face in a pocket between
/// prop and wall drops toward 0.67, a face boxed in on
/// all sides to the 0.56 floor. Opacity here is
/// `Pass::Opaque` only — glass and cutout do not occlude.
/// The occluder count around air position `a` for a face
/// looking along `dir` (the counting core of
/// [`face_occlusion`]): the opaque cells beside `a` in the
/// face plane plus the cell straight ahead at `a + dir`.
fn occlusion_count(sampler: &FineSampler, a: [i32; 3], dir: [i32; 3]) -> u32 {
    let opaque = |p: [i32; 3]| {
        let b = sampler.block_at_fine(p[0], p[1], p[2]);
        !b.is_air() && b.pass == Pass::Opaque
    };
    let mut count = 0u32;
    for axis in 0..3 {
        if dir[axis] != 0 {
            continue;
        }
        for s in [-1, 1] {
            let mut p = a;
            p[axis] += s;
            if opaque(p) {
                count += 1;
            }
        }
    }
    if opaque([a[0] + dir[0], a[1] + dir[1], a[2] + dir[2]]) {
        count += 1;
    }
    count
}

fn occlusion_factor(count: u32) -> f32 {
    (1.0 - 0.11 * count as f32).max(0.56)
}

pub fn face_occlusion(sampler: &FineSampler, fx: i32, fy: i32, fz: i32, dir: [i32; 3]) -> f32 {
    let a = [fx + dir[0], fy + dir[1], fz + dir[2]];
    occlusion_factor(occlusion_count(sampler, a, dir))
}

/// Per-corner occlusion (docs/fine-slope-normals.md §13):
/// the [`face_occlusion`] counting rule run from the air
/// cell diagonally in front of one face corner — the front
/// cell that has the corner vertex on its back face, one
/// cell beyond the face's own front cell in each in-plane
/// axis toward the corner (`corner` is the face's
/// `FACE_CORNERS` entry for the corner, components in
/// {0, 1}; the component along `dir` is fixed by the face
/// and ignored). Where the flat form stamps one factor on
/// the whole face — a uniform dark notch on the tread
/// beside a 1-cell step — the corner form reads the step
/// only at the corners whose front cell sits against it,
/// so the rasteriser interpolates a gradient across the
/// tread. Same 0.11 step and 0.56 floor as the flat form;
/// a corner in the open keeps 1.0 exactly.
pub fn corner_occlusion(
    sampler: &FineSampler,
    fx: i32,
    fy: i32,
    fz: i32,
    dir: [i32; 3],
    corner: [f32; 3],
) -> f32 {
    let mut a = [fx + dir[0], fy + dir[1], fz + dir[2]];
    for axis in 0..3 {
        if dir[axis] != 0 {
            continue;
        }
        a[axis] += if corner[axis] > 0.5 { 1 } else { -1 };
    }
    occlusion_factor(occlusion_count(sampler, a, dir))
}

/// A stamped cell's overall occlusion: the mean of
/// [`face_occlusion`] over its exposed faces (a face is
/// exposed when the neighbouring cell is not opaque). A
/// fully enclosed cell (no exposed face) takes the floor
/// value 0.56. The RT path uses this to step a fine cell's
/// categorical `lit` down; the raster fine mesher uses
/// the per-face form directly.
pub fn cell_occlusion(sampler: &FineSampler, fx: i32, fy: i32, fz: i32) -> f32 {
    let mut sum = 0.0f32;
    let mut n = 0u32;
    for dir in FACE_DIRS {
        let b = sampler.block_at_fine(fx + dir[0], fy + dir[1], fz + dir[2]);
        if !b.is_air() && b.pass == Pass::Opaque {
            continue; // face not exposed
        }
        sum += face_occlusion(sampler, fx, fy, fz, dir);
        n += 1;
    }
    if n == 0 {
        0.56
    } else {
        sum / n as f32
    }
}

impl ChunkMap {
    /// Recompute chunk column `coord`'s `has_fine` flag from
    /// `layer` (flagged iff any volume's tile lies in it).
    /// Returns the flag's new value; a missing chunk reports
    /// `false` and is left alone.
    pub fn refresh_chunk_fine(&mut self, coord: [i32; 2], layer: &FineLayer) -> bool {
        let flag = layer.chunk_has_fine(coord);
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.has_fine = flag;
            flag
        } else {
            false
        }
    }

    /// Recompute EVERY chunk's `has_fine` flag from `layer`.
    pub fn refresh_fine_flags(&mut self, layer: &FineLayer) {
        for (coord, chunk) in self.chunks.iter_mut() {
            chunk.has_fine = layer.chunk_has_fine(*coord);
        }
    }
}

// ---------------------------------------------------------------------
// Phase 9 slice 2 (9.4): the raster fine mesher
// ---------------------------------------------------------------------

/// Whether the coarse face from a block toward its neighbour
/// cell `n` (grid/block coordinates) is culled by the fine
/// boundary rule (see the module docs). Returns:
/// - `Some(true)` — the rule applies and every one of the
///   32×32 stamped fine cells on the face plane is opaque:
///   cull the coarse face;
/// - `Some(false)` — the rule applies but the plane is not
///   fully covered by opaque stamped cells: draw the face;
/// - `None` — the rule does not apply (the neighbour cell is
///   not air, its tile holds no fine volume, or no volume's
///   z-range covers the neighbour block's whole fine-z span):
///   the caller falls back to `face_visible`.
fn coarse_face_cull_by_fine(
    coarse: &ChunkMap,
    layer: &FineLayer,
    n: [i32; 3],
    dir: [i32; 3],
) -> Option<bool> {
    if !coarse.block_at(n[0], n[1], n[2]).is_air() {
        return None;
    }
    let tile = (n[0], n[1]);
    let z_lo = n[2] * FINE;
    let z_hi = z_lo + FINE;
    let vol = layer
        .volumes_for_tile(tile)
        .iter()
        .find(|v| v.z0 <= z_lo && z_hi <= v.z0 + v.depth_z)?;
    // The face plane's fixed fine coordinate (the fine cells
    // immediately across the plane from the source block, i.e.
    // the first fine layer inside the neighbour block along
    // `dir`), and the two varying axes spanning the block face.
    let axis = (0..3).find(|&a| dir[a] != 0).unwrap();
    let others: Vec<usize> = (0..3).filter(|&a| a != axis).collect();
    let mut fixed = [0i32; 3];
    for a in 0..3 {
        fixed[a] = n[a] * FINE;
    }
    if dir[axis] < 0 {
        fixed[axis] = n[axis] * FINE + (FINE - 1);
    }
    // For +Z this is the volume's bottom fine layer when the
    // neighbour block is the volume's first block (the floor
    // slab under a prop in the same tile) — `fixed[2]` is
    // exactly `n[2] * FINE`, the neighbour block's bottom.
    for u in 0..FINE {
        for v in 0..FINE {
            let mut f = fixed;
            f[others[0]] = n[others[0]] * FINE + u;
            f[others[1]] = n[others[1]] * FINE + v;
            let lx = f[0] - tile.0 * FINE;
            let ly = f[1] - tile.1 * FINE;
            match vol.get(lx, ly, f[2]) {
                Some(b) if !b.is_air() && b.pass == Pass::Opaque => {}
                _ => return Some(false),
            }
        }
    }
    Some(true)
}

/// Hard-edge threshold for slope normals (fine cells): a
/// neighbouring column whose surface sits more than this far
/// from the face's own surface is a deliberate cliff, not a
/// slope — corners sampling it keep the axis normal, so table
/// edges, panel ends and other hard silhouettes stay crisp.
/// 8 fine cells = a quarter tile.
pub const SLOPE_EDGE_THRESHOLD: i32 = 8;

/// Columns sampled on each side of a corner vertex by the
/// slope plane fit (docs/fine-slope-normals.md S3): centres
/// at +-0.5 .. +-2.5 cells, a 6x6 window — wide enough to
/// span the step period of a quantised shallow slope
/// (gradient ~0.3 steps every ~3.5 columns), narrow enough
/// that features a few cells wide keep their shape.
const SLOPE_FIT_COLS: i32 = 3;

/// The top height (absolute fine z of the first air cell) of
/// the surface continuous with the cell at `(fx, fy, fz)` in
/// the neighbouring column `(fx, fy)`: from `fz`, walk up
/// while solid or down while air. `None` when the surface
/// steps away by more than `limit` cells in either direction
/// (a cliff or a void — not this face's sheet). Occupancy is
/// `!is_air` via the sampler, so coarse-expanded columns
/// resolve exactly like stamped ones.
fn surface_top(sampler: &FineSampler, fx: i32, fy: i32, fz: i32, limit: i32) -> Option<i32> {
    let solid = |z: i32| !sampler.block_at_fine(fx, fy, z).is_air();
    if solid(fz) {
        let mut z = fz;
        while solid(z + 1) {
            z += 1;
            if z - fz > limit {
                return None;
            }
        }
        Some(z + 1)
    } else {
        let mut z = fz;
        while !solid(z - 1) {
            z -= 1;
            if fz - z > limit {
                return None;
            }
        }
        Some(z)
    }
}

/// Per-corner normal (world space) for the +Z face of the
/// stamped cell at `(fx, fy, fz)`, at the face corner offset
/// `(dx, dy)` in {0,1}^2 (docs/fine-slope-normals.md S1, S3).
/// The gradient is a least-squares plane fit over the
/// columns around the grid vertex `(fx + dx, fy + dy)`;
/// columns whose tops are missing or further than
/// [`SLOPE_EDGE_THRESHOLD`] from the face's own top are
/// excluded from the fit, and too few samples falls the
/// corner back to the axis normal.
fn top_corner_normal(sampler: &FineSampler, fx: i32, fy: i32, fz: i32, dx: i32, dy: i32) -> [f32; 3] {
    let h0 = fz + 1;
    let vx = fx + dx;
    let vy = fy + dy;
    // Least-squares plane over the 6x6 columns around the
    // vertex (centres at +-0.5 .. +-2.5). A 1-cell difference
    // oscillates between 0 and the full step on any slope
    // shallower than 1:1, because the integer heightfield
    // steps in whole cells with flat rests between steps
    // (gradient 0.84 quantises to runs of 1s and 0s); the fit
    // averages the rests and steps back out to the true
    // gradient. Samples past the hard-edge threshold are
    // excluded, so a cliff contributes only its own side and
    // the fit stays flat there; too few samples (volume
    // border, void) falls back to the axis normal.
    let mut samples = [(0.0f64, 0.0f64, 0.0f64); 36];
    let mut n = 0usize;
    let (mut su, mut sv, mut sh) = (0.0f64, 0.0f64, 0.0f64);
    for cx in (vx - SLOPE_FIT_COLS)..=(vx + SLOPE_FIT_COLS - 1) {
        for cy in (vy - SLOPE_FIT_COLS)..=(vy + SLOPE_FIT_COLS - 1) {
            let Some(h) = surface_top(sampler, cx, cy, fz, SLOPE_EDGE_THRESHOLD) else {
                continue;
            };
            if (h - h0).abs() > SLOPE_EDGE_THRESHOLD {
                continue;
            }
            let u = (cx - vx) as f64 + 0.5;
            let v = (cy - vy) as f64 + 0.5;
            samples[n] = (u, v, h as f64);
            su += u;
            sv += v;
            sh += h as f64;
            n += 1;
        }
    }
    if n < 12 {
        return [0.0, 1.0, 0.0];
    }
    let nf = n as f64;
    let (mu, mv, mh) = (su / nf, sv / nf, sh / nf);
    let (mut sxx, mut syy, mut sxy, mut sxh, mut syh) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for &(u, v, h) in &samples[..n] {
        let (du, dv, dh) = (u - mu, v - mv, h - mh);
        sxx += du * du;
        syy += dv * dv;
        sxy += du * dv;
        sxh += du * dh;
        syh += dv * dh;
    }
    let det = sxx * syy - sxy * sxy;
    if det.abs() < 1e-6 {
        return [0.0, 1.0, 0.0];
    }
    let dhdx = ((syy * sxh - sxy * syh) / det) as f32;
    let dhdy = ((sxx * syh - sxy * sxh) / det) as f32;
    let inv_len = 1.0 / (dhdx * dhdx + dhdy * dhdy + 1.0).sqrt();
    let g = [-dhdx * inv_len, -dhdy * inv_len, inv_len];
    // Grid -> world, the normal_to_world rotation for a
    // non-axis vector: (x, y_south, z_up) -> (x, z_up, -y_south).
    [g[0], g[2], -g[1]]
}

/// Per-corner normals for a SIDE face of the stamped cell at
/// `(fx, fy, fz)` when the face is a slope RISER: the
/// neighbouring column's surface is lower than this cell's
/// top by 1..=[`SLOPE_EDGE_THRESHOLD`] cells, so the face is
/// one step of a heightfield staircase and takes the sheet's
/// plane normal — top-edge corners from this cell's gradient
/// (as [`top_corner_normal`]), bottom-edge corners from the
/// lower neighbour cell's gradient at the same vertex. Any
/// other side face (wall, cliff, same-height neighbour)
/// returns `None` and keeps the axis normal.
fn riser_corner_normals(
    sampler: &FineSampler,
    fx: i32,
    fy: i32,
    fz: i32,
    dir: [i32; 3],
    di: usize,
) -> Option<[[f32; 3]; 4]> {
    let hn = surface_top(sampler, fx + dir[0], fy + dir[1], fz, SLOPE_EDGE_THRESHOLD)?;
    let h0 = fz + 1;
    if hn >= h0 || h0 - hn > SLOPE_EDGE_THRESHOLD {
        return None;
    }
    let mut out = [[0.0f32; 3]; 4];
    for (i, c) in FACE_CORNERS[di].iter().enumerate() {
        out[i] = if c[2] > 0.5 {
            top_corner_normal(sampler, fx, fy, fz, c[0] as i32, c[1] as i32)
        } else {
            top_corner_normal(
                sampler,
                fx + dir[0],
                fy + dir[1],
                fz - 1,
                c[0] as i32 - dir[0],
                c[1] as i32 - dir[1],
            )
        };
    }
    Some(out)
}

/// Mesh one chunk column at fine resolution (Phase 9, 9.4):
/// the chunk's coarse cells naively (one quad per visible
/// face, exactly as `mesh_naive` emits them, with the fine
/// boundary rule of the module docs applied to faces against
/// fine tiles) plus every stamped fine cell of every volume
/// whose tile lies in this chunk (one quad per face the
/// [`FineSampler`] leaves visible, at fine scale — cell edge
/// `scale.block_size / FINE`).
///
/// The two passes append into one [`PassMeshes`], coarse
/// first, then fine volumes in ascending tile order (and
/// ascending `z0` within a tile), so the output order is
/// deterministic. Fine quads are textured from the stamped
/// cell's `tex_layers` with per-cell `[0, 1]` UVs (the
/// `mesh_naive` `quad_uvs` / `layer_for_dir` behaviour at
/// fine-cell scale).
///
/// Returns `None` iff the chunk is absent from `coarse`.
/// This function does NOT check `Chunk::has_fine`: the
/// dispatch contract (module docs) is the caller's — flagged
/// chunks take this path, unflagged chunks take
/// `ChunkMap::mesh_chunk`.
pub fn mesh_chunk_fine(
    coarse: &ChunkMap,
    layer: &FineLayer,
    coord: [i32; 2],
    scale: WorldScale,
    slope_normals: bool,
) -> Option<PassMeshes> {
    let chunk = coarse.chunks.get(&coord)?;
    let origin = chunk.origin();
    let size = chunk.grid.size;
    let mut out = PassMeshes::default();

    // -- Coarse pass (naive, per-cell) ---------------------------
    for z in 0..size[2] as i32 {
        for y in 0..size[1] as i32 {
            for x in 0..size[0] as i32 {
                let (gx, gy, gz) = (origin[0] + x, origin[1] + y, origin[2] + z);
                let block = coarse.block_at(gx, gy, gz);
                if block.is_air() {
                    continue;
                }
                let mesh = &mut out.meshes[block.pass.index()];
                for (di, dir) in FACE_DIRS.iter().enumerate() {
                    let n = [gx + dir[0], gy + dir[1], gz + dir[2]];
                    let visible = match coarse_face_cull_by_fine(coarse, layer, n, *dir) {
                        Some(culled) => !culled,
                        None => face_visible(block, coarse.block_at(n[0], n[1], n[2])),
                    };
                    if !visible {
                        continue;
                    }
                    let grid_corners = FACE_CORNERS[di]
                        .map(|c| [gx as f32 + c[0], gy as f32 + c[1], gz as f32 + c[2]]);
                    let corners = grid_corners.map(|g| to_world(g, scale));
                    let normal = normal_to_world(*dir);
                    match layer_for_dir(block, di) {
                        Some(tex_layer) => {
                            let quad_min = [gx as f32, gy as f32, gz as f32];
                            let uvs =
                                quad_uvs(di, &grid_corners, quad_min, [1.0, 1.0, 1.0], tex_layer);
                            mesh.push_quad_textured(corners, normal, block.color, uvs, tex_layer);
                        }
                        None => mesh.push_quad(corners, normal, block.color),
                    }
                }
            }
        }
    }

    // -- Fine pass ------------------------------------------------
    let sampler = FineSampler::new(coarse, layer);
    let inv = 1.0 / FINE as f32;
    for tile in layer.tiles_in_chunk(coord) {
        let mut volumes: Vec<&FineVolume> = layer.volumes_for_tile(tile).iter().collect();
        volumes.sort_by_key(|v| v.z0);
        for vol in volumes {
            for fz in vol.z0..vol.z0 + vol.depth_z {
                for ly in 0..FINE {
                    for lx in 0..FINE {
                        let Some(cell) = vol.get(lx, ly, fz) else {
                            continue;
                        };
                        if cell.is_air() {
                            continue;
                        }
                        let (fx, fy) = (tile.0 * FINE + lx, tile.1 * FINE + ly);
                        let mesh = &mut out.meshes[cell.pass.index()];
                        for (di, dir) in FACE_DIRS.iter().enumerate() {
                            let neighbor =
                                sampler.block_at_fine(fx + dir[0], fy + dir[1], fz + dir[2]);
                            if !face_visible(cell, neighbor) {
                                continue;
                            }
                            // Grid corners in BLOCK units (fine
                            // coordinate / FINE), so `to_world`
                            // at the coarse scale and the
                            // per-cell [0,1] UVs both fall out of
                            // the naive mesher's own helpers.
                            let grid_corners = FACE_CORNERS[di].map(|c| {
                                [
                                    (fx as f32 + c[0]) * inv,
                                    (fy as f32 + c[1]) * inv,
                                    (fz as f32 + c[2]) * inv,
                                ]
                            });
                            let corners = grid_corners.map(|g| to_world(g, scale));
                            let normal = normal_to_world(*dir);
                            // Slope normals (docs/fine-slope-
                            // normals.md S1): with the opt-in,
                            // a stamped cell's +Z face carries
                            // per-corner gradient normals so a
                            // fine staircase shades as one
                            // inclined plane. Every other face
                            // (and every face with the opt-in
                            // off) keeps the axis normal.
                            let corner_normals: Option<[[f32; 3]; 4]> =
                                if !slope_normals {
                                    None
                                } else if di == 4 {
                                    Some(FACE_CORNERS[di].map(|c| {
                                        top_corner_normal(
                                            &sampler,
                                            fx,
                                            fy,
                                            fz,
                                            c[0] as i32,
                                            c[1] as i32,
                                        )
                                    }))
                                } else if di <= 3 {
                                    // Side face: the plane
                                    // normal too when it is a
                                    // slope riser.
                                    riser_corner_normals(&sampler, fx, fy, fz, *dir, di)
                                } else {
                                    None
                                };
                            // Geometric occlusion (Stage H,
                            // 8c-A): darken the face by the
                            // stamped geometry around it. A
                            // face in the open keeps factor
                            // 1.0 — its colour is the cell's
                            // own, byte-identical to the
                            // pre-pass mesher. Under the
                            // slope opt-in the factor is
                            // PER-CORNER (docs/fine-slope-
                            // normals.md §13): each corner
                            // counts the occluders around
                            // its own front cell, so a step
                            // darkens only the tread corners
                            // beside it and the factor
                            // interpolates across the face
                            // instead of stamping one flat
                            // notch. Flag off keeps the
                            // legacy flat per-face factor
                            // exactly.
                            let shade = |occ: f32| {
                                if occ >= 1.0 {
                                    cell.color
                                } else {
                                    [
                                        cell.color[0] * occ,
                                        cell.color[1] * occ,
                                        cell.color[2] * occ,
                                        cell.color[3],
                                    ]
                                }
                            };
                            // Under the opt-in the raw corner
                            // factors are kept alongside the
                            // shaded colours: a textured face
                            // records them in the mesh's
                            // tex_occlusion (§14) because the
                            // light bake discards a textured
                            // vertex's RGB.
                            let corner_occs: Option<[f32; 4]> = if slope_normals {
                                Some(FACE_CORNERS[di].map(|c| {
                                    corner_occlusion(&sampler, fx, fy, fz, *dir, c)
                                }))
                            } else {
                                None
                            };
                            let corner_colors: [[f32; 4]; 4] = match corner_occs {
                                Some(occs) => occs.map(|o| shade(o)),
                                None => {
                                    let occ = face_occlusion(&sampler, fx, fy, fz, *dir);
                                    [shade(occ); 4]
                                }
                            };
                            let normals4 = corner_normals.unwrap_or([normal; 4]);
                            match layer_for_dir(cell, di) {
                                Some(tex_layer) => {
                                    let quad_min =
                                        [fx as f32 * inv, fy as f32 * inv, fz as f32 * inv];
                                    let uvs = quad_uvs(
                                        di,
                                        &grid_corners,
                                        quad_min,
                                        [inv, inv, inv],
                                        tex_layer,
                                    );
                                    match corner_occs {
                                        Some(occs) => mesh
                                            .push_quad_textured_per_corner_colors_occ(
                                                corners,
                                                normals4,
                                                corner_colors,
                                                occs,
                                                uvs,
                                                tex_layer,
                                            ),
                                        None => mesh.push_quad_textured_per_corner_colors(
                                            corners,
                                            normals4,
                                            corner_colors,
                                            uvs,
                                            tex_layer,
                                        ),
                                    }
                                }
                                None => mesh.push_quad_per_corner_colors(
                                    corners,
                                    normals4,
                                    corner_colors,
                                ),
                            }
                        }
                    }
                }
            }
        }
    }
    Some(out)
}
