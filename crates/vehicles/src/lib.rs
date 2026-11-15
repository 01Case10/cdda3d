//! Phase 8 — Vehicles (hard case 8.1–8.3, model level).
//!
//! The game is the only authority on where a vehicle's parts are.
//! Phase 3 exports, per vehicle: a **continuous** facing in float
//! degrees (`vehicle::face.dir()` — never quantized; the 15° figure in
//! old notes is a *pathing* clamp in `vehicle::immediate_path()`, not
//! an orientation step) and, per part, the **absolute tile position
//! computed game-side** from the pre-rotated mounts
//! (`vehicle::bub_part_pos()` / `precalc_mounts`). This crate's one
//! inviolable rule follows from that: **the engine never recomputes
//! part positions from facing + mounts.** It stores the exported
//! absolute positions verbatim, and the only math it adds is
//! *presentation* interpolation between two exported states
//! ([`interp`]) — which is exact at both endpoints by construction,
//! so interpolation can never drift a part off the game's tile.
//!
//! What lives here (pure CPU, no `wgpu` — same precedent as the
//! Phase 7 `lighting` crate, so the gates run headless anywhere):
//!
//! - [`model`]: wire → model decoding. Part flags are the fork's bits
//!   (voxel_real_adapter.cpp): bit 0 = open, bit 1 = broken,
//!   bit 2 = roof (`vpart_info` ROOF flag). Roof parts split into
//!   their own layer, mirroring the split the game's own renderer
//!   uses (gate 8.3).
//! - [`store`]: snapshot/delta application — updates replace a
//!   vehicle's state wholesale, removals delete it, and the previous
//!   state is retained as the interpolation source.
//! - [`interp`]: smooth render states between two exported states:
//!   facing along the shortest arc (359° → 7° is +8°, not −352°),
//!   part positions lerped between the two exported absolute
//!   positions, matched by (vpart, mount) identity.
//! - [`fixtures`]: a mock game-truth generator (pivot + rotated
//!   mounts, the shape of `precalc_mounts`) used by the gate tests
//!   to prove the model reproduces the game's positions exactly at
//!   facings a quantizing engine would get wrong.
//!
//! Engine wiring (drawing the model) is Phase 2h's frame loop; this
//! crate is the data contract it will consume. Full notes:
//! `docs/phase8-hardcases.md`.

pub mod fixtures;
pub mod interp;
pub mod model;
pub mod store;

pub use interp::{lerp_angle_deg, sample_vehicle, shortest_arc_deg, RenderPart, RenderVehicle};
pub use model::{PartKey, PartModel, VehicleModel, FLAG_BROKEN, FLAG_OPEN, FLAG_ROOF};
pub use store::VehicleStore;
