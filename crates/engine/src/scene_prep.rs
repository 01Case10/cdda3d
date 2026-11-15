//! Shared fake-house scene preparation for the raster path
//! (Phase 2g.2): ONE implementation of "mesh the fake house,
//! optionally textured" used by both the headless capture and
//! the windowed app's remesh, so the two paths cannot drift.
//!
//! With a textures dir, the scene spec comes from the shared
//! builder (`textures::build_texture_spec` — the same one the RT
//! path uses, gate 2f.3), the cutaway-filtered map's blocks are
//! stamped with their compact texture layers, and the caller
//! installs the returned spec on its renderer
//! (`Renderer::set_textures`). Without one, the flat meshes are
//! exactly the pre-2f ones.

use std::path::Path;

use mesher::mesh::PassMeshes;
use treebuild::render::TextureSpec;

use crate::overlay::MeshStats;
use crate::{materials, textures, world};

/// Phase 2f: stamp each non-air block of the (already
/// cutaway-filtered) map with its material's compact texture
/// layers from `spec`, so the meshers emit textured quads for
/// it. Blocks whose material has no texture entry (present flag
/// 0, or id out of range) keep `tex_layers: None` and stay flat.
fn stamp_tex_layers(map: &mut mesher::chunk::ChunkMap, spec: &TextureSpec) {
    for chunk in map.chunks.values_mut() {
        let size = chunk.grid.size;
        for z in 0..size[2] {
            for y in 0..size[1] {
                for x in 0..size[0] {
                    let b = chunk.grid.get(x as i32, y as i32, z as i32);
                    if b.is_air() {
                        continue;
                    }
                    if let Some(ml) = spec.material_layers.get(b.id as usize) {
                        if ml[3] == 1 {
                            let mut stamped = b;
                            stamped.tex_layers = Some([ml[0], ml[1], ml[2]]);
                            chunk.grid.set(x, y, z, stamped);
                        }
                    }
                }
            }
        }
    }
}

/// Mesh an already-filtered map (the combine loop of
/// `world::mesh_world`, minus its filtering step — the textured
/// path filters and stamps before meshing).
fn mesh_filtered(map: &mesher::chunk::ChunkMap) -> (PassMeshes, MeshStats) {
    let mut combined = PassMeshes::default();
    let mut n_chunks = 0;
    for coord in map.chunks.keys().copied().collect::<Vec<_>>() {
        if let Some(pm) = map.mesh_chunk(coord, true) {
            for (dst, src) in combined.meshes.iter_mut().zip(pm.meshes.iter()) {
                dst.append(src);
            }
            n_chunks += 1;
        }
    }
    let stats = MeshStats::from_pass_meshes(n_chunks, &combined);
    (combined, stats)
}

/// Prepare the fake house for a raster render at the given
/// visible z-levels. Returns the meshes, their stats, and the
/// texture spec to install on the renderer (`None` = flat).
pub fn prepare_raster_house(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    match textures_dir {
        Some(dir) => {
            let table = materials::MaterialTable::load(dir)
                .map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            let map = world::build_fake_house(mesher::scale::WorldScale::default());
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            stamp_tex_layers(&mut filtered, &spec);
            let (meshes, stats) = mesh_filtered(&filtered);
            Ok((meshes, stats, Some(spec)))
        }
        None => {
            let (_map, meshes, stats) = world::mesh_fake_house(visible_levels);
            Ok((meshes, stats, None))
        }
    }
}

/// Phase 2h.3: the lit form of [`prepare_raster_house`] — the
/// Phase 7 wiring contract (docs/phase7-lighting.md §12, raster
/// half) applied to the fake house:
///
/// 1. The cutaway-filtered map is masked through the shared
///    `lighting::VisibilityMask`: blocks whose tile is `Hidden`
///    (`BLANK` / never seen) are dropped before meshing, so they
///    produce no geometry at all (the per-vertex baker alone
///    cannot remove geometry).
/// 2. The meshed result is light-baked per-vertex with
///    `lighting::bake_pass_meshes` (the precise, leak-proof
///    baker: air-side sampling, MEMORIZED desaturation at the
///    pinned 0.30 / 0.45 factors) under `lit.env` (day/night ×
///    weather state).
///
/// `grid_z_of_level0` for the baker is the scene's minimum game
/// z (`world::HOUSE_MIN_GAME_Z`): the grid is indexed in game
/// tile coordinates, chunk level `i` is grid z `min_game_z + i`.
pub fn prepare_raster_house_lit(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    let scale = mesher::scale::WorldScale::default();
    let (spec, mut filtered) = match textures_dir {
        Some(dir) => {
            let table = materials::MaterialTable::load(dir)
                .map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            let map = world::build_fake_house(scale);
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            stamp_tex_layers(&mut filtered, &spec);
            (Some(spec), filtered)
        }
        None => {
            let map = world::build_fake_house(scale);
            let filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            (None, filtered)
        }
    };
    apply_visibility_mask(&mut filtered, lit);
    let (mut meshes, stats) = mesh_filtered(&filtered);
    lighting::bake_pass_meshes(
        &mut meshes,
        &lit.grid,
        lit.env,
        scale,
        world::HOUSE_MIN_GAME_Z,
    );
    Ok((meshes, stats, spec))
}

/// Drop every block whose tile the shared visibility mask marks
/// `Hidden` (Phase 7 §12 step 3: the mask applied in the filter
/// step). Tiles are addressed in game coordinates: chunk origin
/// (block space) + block offset, z from the block's z-level.
pub fn apply_visibility_mask(
    map: &mut mesher::chunk::ChunkMap,
    lit: &crate::frame::LightingScene,
) {
    let mask = lighting::VisibilityMask::new(&lit.grid);
    let per = map.scale.blocks_per_z_level.max(1) as i32;
    for chunk in map.chunks.values_mut() {
        let origin = chunk.origin();
        for z in 0..chunk.grid.size[2] {
            let game_z = lit.min_game_z + (z as i32) / per;
            for y in 0..chunk.grid.size[1] {
                for x in 0..chunk.grid.size[0] {
                    let b = chunk.grid.get(x as i32, y as i32, z as i32);
                    if b.is_air() {
                        continue;
                    }
                    let tile = [origin[0] + x as i32, origin[1] + y as i32, game_z];
                    if !mask.is_meshed(tile[0], tile[1], tile[2]) {
                        chunk.grid.set(x, y, z, mesher::block::Block::AIR);
                    }
                }
            }
        }
        chunk.dirty = true;
    }
}
