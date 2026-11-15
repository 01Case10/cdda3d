//! Phase 6: Billboards — camera-facing 2D sprites for creatures,
//! NPCs and ground items, composited over the voxel / ray-traced
//! world (plan §5 Phase 6).
//!
//! Billboards are the ONE place original tileset sprites appear
//! as-is in the 3D view. Field effects are never billboards — they
//! are Phase 2b shader volumes (`crates/fx`). Everything else
//! (terrain, furniture, vehicles) is voxel geometry.
//!
//! Modules:
//! * [`atlas`] — the voxelizer's `sprites.png`/`sprites.json`
//!   format, plus the in-crate fixture atlas.
//! * [`composite`] — overlay composition in the game's own draw
//!   order, the signature cache, facing/mirroring, and sprite
//!   animation (weighted frames, bob, hit-flash).
//! * [`instance`] — cylindrical-billboard instance generation from
//!   real sprite geometry, blob shadows, alpha picking, item piles.
//! * [`render`] — the reusable instanced wgpu pass (depth-tested
//!   against an external world/RT depth) and its headless harness.
//! * [`scene`] — fake-data scenes (zombie crowd, stress crowd)
//!   mirroring the protocol's entity/item shapes.

pub mod atlas;
pub mod composite;
pub mod instance;
pub mod render;
pub mod scene;
