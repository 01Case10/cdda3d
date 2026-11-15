//! Phase 8 — Animations and projectiles (hard case 8.5, model
//! level).
//!
//! The game dispatches animation/projectile effects through
//! `src/animation.*` (it already selects curses vs. tiles
//! implementations per effect); Phase 3's protocol forwards them as
//! `Effects` messages — a list of [`proto::EffectEventT`], each
//! carrying a kind, an effect id, from/to tiles and a duration.
//! Per Joey's Phase 2b direction, the engine renders these through
//! the **shader-effect system, never as sprite-frame animation**:
//! each event becomes a *transient emitter* — a short-lived spawn
//! in the same parameter space as the field effects (render class,
//! color, emissive) — which the engine's fx pass can draw with the
//! machinery Phase 2b/2g already landed.
//!
//! This crate owns the lifecycle, as pure data (no `wgpu`, the
//! Phase 7 precedent, so the gates run headless anywhere):
//!
//! - [`params`]: effect id + kind → emitter parameters. The render
//!   class ids mirror `fx::RenderClass::id()` exactly; like the fx
//!   crate mirroring the voxelizer's `effects.json` contract
//!   instead of depending on it, this crate mirrors the class ids
//!   instead of taking a wgpu dependency for six constants.
//! - [`system`]: ingest (idempotent per (seq, index) — a re-sent
//!   `Effects` message never double-spawns), a monotonic clock,
//!   trajectories (projectiles travel from → to over the event's
//!   duration; bursts/flashes sit at their tile), and expiry —
//!   gate 8.5's "no leftover effects" is the system's core
//!   invariant: once every duration has elapsed, the active set is
//!   empty.
//!
//! Protocol truth, stated plainly: an `EffectEvent` carries
//! endpoints and a duration, **no arc height and no curve** — a
//! thrown item's parabola is not in the data. Trajectories here are
//! therefore linear in tile space; any visual arc is a presentation
//! choice the engine may layer on, clearly separate from the
//! exported truth this crate preserves. Full notes:
//! `docs/phase8-hardcases.md`.

pub mod params;
pub mod system;

pub use params::{
    params_for, EmitterParams, CLASS_ENERGY, CLASS_FLAME, CLASS_GAS, CLASS_GENERIC, CLASS_LIQUID,
    CLASS_SMOKE,
};
pub use system::{Emitter, EmitterKey, TransientSystem};
