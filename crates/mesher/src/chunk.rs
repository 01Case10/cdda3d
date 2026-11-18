//! Chunk columns aligned to game submaps (12x12 tiles), plus a chunk map
//! with dirty-flag propagation across chunk boundaries.

use std::collections::{BTreeSet, HashMap};

use crate::block::{block_for_tile, tile_shape, Block, TileShape};
use crate::grid::{VoxelGrid, VoxelSampler};
use crate::mesh::PassMeshes;
use crate::mesher::{mesh_greedy, mesh_naive};
use crate::scale::WorldScale;

pub const CHUNK_W: usize = 12;
pub const CHUNK_H: usize = 12;

/// One chunk column: 12x12 tiles x `levels` z-levels
/// (x `WorldScale::blocks_per_z_level` blocks tall).
#[derive(Debug, Clone)]
pub struct Chunk {
    pub coord: [i32; 2],
    pub levels: usize,
    pub grid: VoxelGrid,
    pub dirty: bool,
    /// Phase 9 (plan §11b, 9.0): this chunk column contains at
    /// least one tile with a fine volume in the world's
    /// `fine::FineLayer`, so it takes the fine meshing path
    /// instead of the coarse fast path. Maintained from the
    /// layer via `ChunkMap::refresh_chunk_fine` /
    /// `refresh_fine_flags`; the layer is the source of truth.
    pub has_fine: bool,
}

impl Chunk {
    pub fn new(coord: [i32; 2], levels: usize, scale: WorldScale) -> Self {
        let height = scale.column_block_height(levels as u32) as usize;
        Self {
            coord,
            levels,
            grid: VoxelGrid::new([CHUNK_W, CHUNK_H, height]),
            dirty: true,
            has_fine: false,
        }
    }

    /// Block-space origin of this chunk in global grid coordinates.
    pub fn origin(&self) -> [i32; 3] {
        [
            self.coord[0] * CHUNK_W as i32,
            self.coord[1] * CHUNK_H as i32,
            0,
        ]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z_block: usize, block: Block) {
        self.grid.set(x, y, z_block, block);
        self.dirty = true;
    }

    /// Place a game tile at tile `(x, y)` of z-level `level`, expanding it
    /// to blocks per its shape and the world scale (floor = bottom slab,
    /// roof = top slab, everything else = full level height).
    pub fn set_tile(&mut self, x: usize, y: usize, level: usize, tile_id: u32, scale: WorldScale) {
        let block = block_for_tile(tile_id);
        let base = scale.level_base_block(level as u32);
        let per = scale.blocks_per_z_level;
        let (lo, hi) = match tile_shape(tile_id) {
            TileShape::Full => (base, base + per),
            TileShape::FloorSlab => (base, base + scale.floor_slab_blocks.min(per)),
            // A roof tile is authored on the level ABOVE
            // the room it closes and stamps as that
            // level's bottom slab -- a deck seated on the
            // wall plate below, exactly like the snapshot
            // path's Floor::RoofOf stamping. (It used to
            // take the level's TOP slab, which floated the
            // roof two blocks above the walls; Joey,
            // 2026-11-17.)
            TileShape::RoofSlab => (base, base + scale.roof_slab_blocks.min(per)),
        };
        for z in lo..hi {
            self.grid.set(x, y, z as usize, block);
        }
        self.dirty = true;
    }
}

/// A set of chunk columns with global block access and dirty propagation.
#[derive(Debug, Default)]
pub struct ChunkMap {
    pub chunks: HashMap<[i32; 2], Chunk>,
    pub scale: WorldScale,
}

impl ChunkMap {
    pub fn new(scale: WorldScale) -> Self {
        Self {
            chunks: HashMap::new(),
            scale,
        }
    }

    pub fn add_chunk(&mut self, coord: [i32; 2], levels: usize) -> &mut Chunk {
        self.chunks
            .entry(coord)
            .or_insert_with(|| Chunk::new(coord, levels, self.scale))
    }

    pub fn chunk_coord_of(gx: i32, gy: i32) -> [i32; 2] {
        [
            gx.div_euclid(CHUNK_W as i32),
            gy.div_euclid(CHUNK_H as i32),
        ]
    }

    /// Set a block by global grid coordinate. Marks the owning chunk dirty;
    /// if the block sits on a chunk boundary, the chunk across that
    /// boundary is marked dirty too (its meshing depends on this block).
    /// Chunks not adjacent to the edit are untouched. Returns the set of
    /// chunk coords that were marked dirty by this edit.
    pub fn set_global_block(&mut self, gx: i32, gy: i32, gz: i32, block: Block) -> BTreeSet<[i32; 2]> {
        let coord = Self::chunk_coord_of(gx, gy);
        let lx = gx.rem_euclid(CHUNK_W as i32) as usize;
        let ly = gy.rem_euclid(CHUNK_H as i32) as usize;
        let mut marked = BTreeSet::new();
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            if gz >= 0 && (gz as usize) < chunk.grid.size[2] {
                chunk.set_block(lx, ly, gz as usize, block);
                marked.insert(coord);
            }
        }
        if marked.is_empty() {
            return marked;
        }
        let mut neighbors = Vec::new();
        if lx == 0 {
            neighbors.push([coord[0] - 1, coord[1]]);
        }
        if lx == CHUNK_W - 1 {
            neighbors.push([coord[0] + 1, coord[1]]);
        }
        if ly == 0 {
            neighbors.push([coord[0], coord[1] - 1]);
        }
        if ly == CHUNK_H - 1 {
            neighbors.push([coord[0], coord[1] + 1]);
        }
        // Corner edits touch the diagonal chunk's corner-adjacent faces
        // only diagonally; the two edge neighbors above are the ones whose
        // face culling this block can change, which is what dirty means.
        for n in neighbors {
            if let Some(chunk) = self.chunks.get_mut(&n) {
                chunk.dirty = true;
                marked.insert(n);
            }
        }
        marked
    }

    pub fn dirty_coords(&self) -> BTreeSet<[i32; 2]> {
        self.chunks
            .iter()
            .filter(|(_, c)| c.dirty)
            .map(|(k, _)| *k)
            .collect()
    }

    pub fn clear_dirty(&mut self) {
        for c in self.chunks.values_mut() {
            c.dirty = false;
        }
    }

    /// Mesh one chunk, culling boundary faces against the *actual*
    /// neighbor chunks in this map (via the sampler impl below).
    pub fn mesh_chunk(&self, coord: [i32; 2], greedy: bool) -> Option<PassMeshes> {
        let chunk = self.chunks.get(&coord)?;
        let origin = chunk.origin();
        let size = chunk.grid.size;
        let out = if greedy {
            mesh_greedy(self, origin, size, self.scale)
        } else {
            mesh_naive(self, origin, size, self.scale)
        };
        Some(out)
    }
}

impl VoxelSampler for ChunkMap {
    fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        if z < 0 {
            return Block::AIR;
        }
        let coord = Self::chunk_coord_of(x, y);
        match self.chunks.get(&coord) {
            Some(chunk) => chunk.grid.get(
                x.rem_euclid(CHUNK_W as i32),
                y.rem_euclid(CHUNK_H as i32),
                z,
            ),
            None => Block::AIR,
        }
    }
}
