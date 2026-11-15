//! Chunk data and voxel meshing for the CDDA voxel frontend.
//!
//! Pure Rust, no GPU dependencies. Two meshers are provided over the same
//! data — a naive per-face mesher (reference implementation) and a greedy
//! mesher — so tests can prove they produce the same visible surface.
//!
//! Coordinates: grid space is `(x = east tile, y = south tile, z = up block)`.
//! Mesh output is Y-up world space: grid `(x, y, z)` maps to world
//! `(x, z, -y)` (a rotation, so winding is preserved), scaled by
//! [`WorldScale::block_size`]. World +Z therefore points north.

pub mod block;
pub mod chunk;
pub mod grid;
pub mod mesh;
pub mod mesher;
pub mod scale;

pub use block::{block_for_tile, tile_shape, Block, Pass, TileShape, AIR_ID};
pub use chunk::{Chunk, ChunkMap, CHUNK_H, CHUNK_W};
pub use grid::{ClosureSampler, VoxelGrid, VoxelSampler};
pub use mesh::{covered_face_cells, Mesh, PassMeshes};
pub use mesher::{face_visible, mesh_greedy, mesh_naive};
pub use scale::WorldScale;
