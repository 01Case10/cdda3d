//! CDDA Voxel Frontend Phase 11 — test game engine (`sim-server`).
//!
//! A deterministic stand-in for the forked game, speaking the exact
//! Phase 1 IPC wire (`schema/cdda_ipc.fbs`, `docs/protocol.md`):
//! u32 little-endian length prefix + one FlatBuffers `Envelope` per
//! frame, verifier-checked decode, exact-match handshake.
//!
//! Slices landed: S0 (socket skeleton: handshake, TileDefs,
//! Snapshot, Ping/Pong, clean shutdown), S1 (the full S1a def
//! registry — 13,532 real defs on the 0.I-1 base — plus the §3.4
//! scene loader and the Snapshot builder; stub_house and the
//! roofed-house pair reproduce the `proto::samples` fixtures
//! semantically), S2 (the turn loop: one pressed Key Input = one
//! turn, movement / door open-close rules, diff-shaped Deltas,
//! fresh Snapshot with epoch + 1 on a map shift), and S3
//! (computed visibility + environment: LOS/apparent-light/
//! outside/frac/tint recompute, tile memory, the real calendar
//! and weather presets, EnvironmentUpdate cadence, LightUpdate
//! folding — see `environment` and `visibility`), S4 (the
//! UI: shm writer in the ui-share store format, a five-state
//! UI state machine with real-font rendering — see `ui`), and
//! S5 (creatures, fields, effects: runtime spawns, chase /
//! wander / attack AI, bump attacks, corpse drops, fire
//! spread / smoke drift field ticks, fd_fire light in the
//! visibility model, Effects emission — see `rules`).

pub mod defs;
pub mod environment;
pub mod measure;
pub mod rules;
pub mod scenes;
pub mod server;
pub mod ui;
pub mod visibility;
pub mod world;

pub use defs::{DefRegistry, Kind};
pub use scenes::Scene;
pub use server::{run_server, ServerConfig};
pub use world::World;
