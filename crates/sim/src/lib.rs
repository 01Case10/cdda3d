//! CDDA Voxel Frontend Phase 11 — test game engine (`sim-server`).
//!
//! A deterministic stand-in for the forked game, speaking the exact
//! Phase 1 IPC wire (`schema/cdda_ipc.fbs`, `docs/protocol.md`):
//! u32 little-endian length prefix + one FlatBuffers `Envelope` per
//! frame, verifier-checked decode, exact-match handshake.
//!
//! Slices landed: S0 (socket skeleton: handshake, TileDefs,
//! Snapshot, Input echo, Ping/Pong, clean shutdown) and S1 (the
//! full S1a def registry — 14,262 real defs — plus the §3.4 scene
//! loader and the Snapshot builder; stub_house and the roofed-house
//! pair reproduce the `proto::samples` fixtures semantically).
//! The stateful turn loop (one Input = one turn, diff-shaped
//! Deltas) is slice S2; visibility/environment simulation is S3.

pub mod defs;
pub mod measure;
pub mod scenes;
pub mod server;
pub mod world;

pub use defs::{DefRegistry, Kind};
pub use scenes::Scene;
pub use server::{run_server, ServerConfig};
pub use world::World;
