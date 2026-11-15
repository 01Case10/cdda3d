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
