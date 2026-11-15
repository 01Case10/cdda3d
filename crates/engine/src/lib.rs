//! CDDA Voxel Frontend engine (Phase 2 MVP).
//!
//! All testable logic lives in the lib: camera, z-cutaway, fake-data
//! world building/meshing, debug-overlay state, frame pacing, resize
//! decisions, the shared wgpu renderer, and the headless capture mode.
//! The bin is a thin winit/wgpu shell over this lib, and the headless
//! capture path shares `render::Renderer` with the windowed path.

pub mod camera;
pub mod capture;
pub mod cutaway;
pub mod frame;
pub mod fxwire;
pub mod materials;
pub mod overlay;
pub mod render;
pub mod rt;
pub mod scene_prep;
pub mod textures;
pub mod world;
