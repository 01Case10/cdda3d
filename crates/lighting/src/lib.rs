//! Phase 7 — Lighting, FOV and atmosphere (visibility truth).
//!
//! What this crate owns (plan §5 Phase 7, including the Phase 2c
//! scope split): everything that only the *game* knows about
//! visibility — per-tile `lit_level` (`LitLevel`) and `seen` as the
//! source of truth for what may be shown at all, remembered-vs-seen
//! presentation, and day/night + weather *state*. What it does not
//! own: illumination in the physical sense (sun shadows, AO, bounce
//! feel) — that belongs to the Phase 2c ray tracer, and raster
//! shading consumes this crate's output as its visibility/light
//! factor. The tracer/renderer must never reveal a tile the game
//! marks unseen; the [`VisibilityMask`] / [`Visibility`] contract in
//! `mapping`/`grid` is how both renderers enforce that, as pure data.
//!
//! This crate is deliberately pure CPU (no `wgpu`): gates 7.1–7.9
//! are proven at data level on light-baked meshes plus CPU PNG
//! dumps, following the Phase 0 spike precedent. Full design notes:
//! `docs/phase7-lighting.md`.
//!
//! Pipeline at a glance:
//! `proto Snapshot/Delta/LightUpdate` → [`LightGrid`] (via
//! `proto_adapter` / `updates`) → mapping + smoothing (`mapping`,
//! `smoothing`) → baking (`bake`) under an [`Environment`]
//! (`atmosphere`) → mesher `ChunkMap`/`PassMeshes` the renderers
//! already consume, or a `treebuild` voxel set filtered by the
//! visibility mask.

pub mod atmosphere;
pub mod bake;
pub mod fixtures;
pub mod grid;
pub mod mapping;
pub mod png;
pub mod proto_adapter;
pub mod render;
pub mod sky;
pub mod smoothing;
pub mod updates;

pub use atmosphere::{
    day_night_at, weather_params, DayNight, Environment, WeatherKind, WeatherParams,
};
pub use bake::{
    bake_chunk_map, bake_chunk_map_with_offset, bake_pass_meshes, filter_levels, shade_color,
};
pub use grid::{ClosureLightSampler, LightGrid, LightSampler, VisibilityMask};
pub use mapping::{
    apply_visibility_rgb, base_intensity, include_in_tree, is_meshed, luminance, present_color,
    presented_sample, visibility_of, LightSample, TileLight, Visibility, MEMORIZED_BRIGHTNESS,
    MEMORIZED_SATURATION_KEEP,
};
pub use sky::{
    classify_weather, fallback_sun_angles, max_elev_for_season, moon_light_for_phase, sky_state,
    sun_direction, GameEnvironment, PrecipKind, PrecipState, SkyClock, SkyState,
};
pub use smoothing::{
    air_side_sample, face_light, smoothed_tile_sample, top_face_vertex_light, vertex_light,
    vertex_light_at_grid_corner, vertex_light_from_air, FACE_DIRS,
};
pub use updates::{
    apply_delta, apply_light_update, apply_patch, apply_tile_changes, chunks_dirtied_by_tile,
    dirty_chunks_for_tiles, LightPatch,
};
