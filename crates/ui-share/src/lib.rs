//! Phase 4: UI compositing for the CDDA Voxel Frontend.
//!
//! The game renders its entire UI (sidebar panels, message log,
//! menus — everything except the terrain window) into a CPU RGBA
//! buffer; this crate is the engine side of that handoff:
//!
//! - [`store`]: double-buffered shared-memory store over the Phase 1
//!   `proto::shm::UiShmHeader`, with per-frame checksums so a torn
//!   frame is detectable and never observed (gates 4.3/4.4).
//! - [`frame`]: `UiFrame` notification validation, terrain-rect
//!   masking (gate 4.2), and dirty-rect upload planning (gate 4.5).
//! - [`compositor`]: wgpu fullscreen-quad compositing of the UI
//!   texture over the 3D background, nearest-neighbor with
//!   stretch/integer scaling.
//! - [`scaling`]: resize/DPI conversions as pure functions (gate 4.6).
//! - [`input`]: engine events -> `proto` `Input` messages and the
//!   `UiMode` gating rules (gates 4.7–4.10).
//! - [`picking`]: camera ray -> voxel DDA -> tile `PickResult`
//!   (gate 4.11), with the panel/mode separation rule.
//! - [`synthetic`]: CPU-drawn golden-frame stand-ins + image diff
//!   (gate 4.1 at harness level).
//! - [`latency`]: harness latency accounting (gate 4.12 segment).
//!
//! Route note: Phase 0 chose Route B (`catacurses`-level capture)
//! as primary — the fork renders UI windows into this shared buffer
//! and suppresses the map window — with Route A (`display_buffer`
//! readback) as fallback. This crate is route-agnostic: both routes
//! deliver the same thing, an RGBA frame in the store plus a
//! `UiFrame` notification carrying the terrain rect.

pub mod compositor;
pub mod frame;
pub mod input;
pub mod latency;
pub mod picking;
pub mod scaling;
pub mod store;
pub mod synthetic;

pub use compositor::UiPass;
pub use frame::{TerrainRect, UploadPlan, ValidatedFrame};
pub use input::{EngineKey, InputRouter, Modifiers};
pub use picking::{PickCamera, Ray};
pub use scaling::{ScalingMode, Size2, Viewport};
pub use store::{DirtyRect, FrameSnapshot, UiShmStore};
