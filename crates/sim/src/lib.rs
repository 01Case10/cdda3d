//! CDDA Voxel Frontend Phase 11 — test game engine (`sim-server`), slice S0.
//!
//! A deterministic stand-in for the forked game, speaking the exact
//! Phase 1 IPC wire (`schema/cdda_ipc.fbs`, `docs/protocol.md`):
//! u32 little-endian length prefix + one FlatBuffers `Envelope` per
//! frame, verifier-checked decode, exact-match handshake.
//!
//! S0 scope is the skeleton only, mirroring the C++ stub server
//! (`tools/stub-server/stub_server.cpp`): handshake, TileDefs, one
//! static sample Snapshot, Input echo, Ping/Pong, clean shutdown.
//! The stateful turn loop (one Input = one turn, diff-shaped Deltas)
//! is slice S2; the full real def table (Option C) is slices S1/S1a.

pub mod server;

pub use server::run_server;
