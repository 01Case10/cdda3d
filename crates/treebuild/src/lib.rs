//! Phase 2c: ray-traced renderer on Sparse-Voxel-64Tree (plan §5 Phase 2c).
//!
//! This crate is the tree boundary the plan requires: the
//! `Sparse-Voxel-64Tree =0.1.0` crate is used only inside [`build`]
//! (tree construction + flatten) and [`render`] (its
//! `Tree64GpuManager` owns the GPU node buffer / bind group). Everything
//! else — the world model ([`world`]), the material side-table, the CPU
//! traversal oracle ([`cpu`]) and our WGSL traversal — works on the
//! crate's documented 32-byte `GpuNode64` flatten format, so an in-house
//! flatten could replace the crate without touching callers.
//!
//! Coordinates: grid space `(x = east tile, y = south tile, z = up
//! block)`, the same space the mesher uses. The tree stores grid coords
//! minus [`BuiltTree::origin`]; rendering converts grid -> world
//! `(x, z, -y)` exactly like the mesher (see [`render::RtCamera`]).

pub mod build;
pub mod connect;
pub mod cpu;
pub mod render;
pub mod world;

pub use build::{build_tree, BuiltTree, BuildConfig, BuildError, SideEntry};
pub use connect::{assign_variants, connection_mask, same_material, VariantAssignment};
pub use cpu::{cast_ray, dda_reference, occupied_at, RayHit, TraceMode};
pub use world::{VoxelCell, VoxelWorld, FLAG_CUTOUT, FLAG_EMISSIVE, FLAG_MEMORIZED, FLAG_TRANSPARENT, LIT_BLANK, LIT_BRIGHT, LIT_BRIGHT_ONLY, LIT_DARK, LIT_LIT, LIT_LOW, LIT_MEMORIZED};
