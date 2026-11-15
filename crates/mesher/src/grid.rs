//! Voxel grids and the neighbor sampler used for cross-boundary culling.

use crate::block::Block;

/// Read-only access to blocks by global grid coordinate
/// `(x = east, y = south, z = up block)`. This is the callback the meshers
/// use for every neighbor lookup, so faces at a chunk edge are culled by
/// whatever lives in the *neighbor* chunk, not by an assumption.
pub trait VoxelSampler {
    fn block_at(&self, x: i32, y: i32, z: i32) -> Block;
}

/// Sampler backed by a closure — handy for tests and adapters.
pub struct ClosureSampler<F: Fn(i32, i32, i32) -> Block>(pub F);

impl<F: Fn(i32, i32, i32) -> Block> VoxelSampler for ClosureSampler<F> {
    fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        (self.0)(x, y, z)
    }
}

/// A dense 3D grid of blocks with origin at grid coordinate (0,0,0).
#[derive(Debug, Clone)]
pub struct VoxelGrid {
    pub size: [usize; 3],
    blocks: Vec<Block>,
}

impl VoxelGrid {
    pub fn new(size: [usize; 3]) -> Self {
        Self {
            size,
            blocks: vec![Block::AIR; size[0] * size[1] * size[2]],
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && y >= 0
            && z >= 0
            && (x as usize) < self.size[0]
            && (y as usize) < self.size[1]
            && (z as usize) < self.size[2]
    }

    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.size[1] + y) * self.size[0] + x
    }

    /// Block at a local coordinate; out of bounds reads as air. (Chunk-edge
    /// culling must go through a `VoxelSampler` that knows the neighbors —
    /// a lone grid honestly reports air outside itself.)
    pub fn get(&self, x: i32, y: i32, z: i32) -> Block {
        if self.in_bounds(x, y, z) {
            self.blocks[self.idx(x as usize, y as usize, z as usize)]
        } else {
            Block::AIR
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let i = self.idx(x, y, z);
        self.blocks[i] = block;
    }

    pub fn fill(&mut self, block: Block) {
        self.blocks.fill(block);
    }

    pub fn count_non_air(&self) -> usize {
        self.blocks.iter().filter(|b| !b.is_air()).count()
    }
}

impl VoxelSampler for VoxelGrid {
    fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        self.get(x, y, z)
    }
}
