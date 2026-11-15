//! Phase 2b: shader field effects + HD-2D post stack.
//!
//! Fields are NEVER rendered from sprites (Joey's direction): the
//! voxelizer extracts per-field-type parameters into `effects.json`
//! ([`effect`]), and this crate renders each field tile as a
//! procedural shader volume ([`render`], [`density`]) weighted by the
//! game's own `displayed_field` choice ([`world`]). The HD-2D post
//! stack ([`post`]) is renderer-agnostic — fullscreen passes over
//! color+depth that apply to the world only; UI composites after.
//!
//! [`scene`] is the standalone fake-data test harness (the crate may
//! not depend on `crates/engine`); engine integration is documented
//! in `docs/phase2b-effects.md`.

pub mod density;
pub mod effect;
pub mod passes;
pub mod post;
pub mod render;
pub mod scene;
pub mod world;
