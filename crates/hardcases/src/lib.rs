//! Phase 8 — Tile-state hard cases (8.4 doors/windows, 8.4b open
//! air and roofs).
//!
//! Two rules of the game's world model that a voxel renderer gets
//! wrong if it invents its own state:
//!
//! - **State is an id change, never a flag** ([`doors`]). Opening,
//!   closing or bashing a door/window makes the game replace the
//!   tile's terrain/furniture id with the transform's result id
//!   (`ter_t::open` / `close` / `bash`, exported in TileDefs as
//!   `open_result` / `close_result` / `bash_result`). The engine
//!   needs no door logic: a Delta carrying the new id *is* the
//!   state change. This module proves the rule end to end —
//!   classify a transition against the def table, apply it to a
//!   tile world, and show the world then holds the new id with
//!   the new id's material reference.
//! - **Open air has no floor of its own** ([`openair`]). A tile of
//!   `t_open_air` emits no block; the floor seen at that level is
//!   the `roof` of the terrain below it (`ter_t::roof` — "what
//!   will be the floor above this terrain"), exported by the new
//!   additive TileDef field `roof_result` (Phase 8). Resolving
//!   anything else — a phantom slab, or the air tile's own
//!   material — puts a fake floor over every shaft, pit and
//!   two-storey room.
//!
//! Pure CPU (no `wgpu`), the Phase 7 precedent. Full notes:
//! `docs/phase8-hardcases.md`.

pub mod doors;
pub mod openair;
pub mod report;

pub use doors::{classify_transition, DefTable, TileWorld, Transition};
pub use openair::{floor_at, Column, Floor};
pub use report::{missing_materials, missing_report_lines, unreferenced_defs, MissingDef};
