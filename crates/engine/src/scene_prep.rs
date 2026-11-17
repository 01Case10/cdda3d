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

use mesher::fine::{FineLayer, ModelCells, FINE};
use mesher::grid::VoxelSampler;
use mesher::mesh::PassMeshes;
use treebuild::render::TextureSpec;

use crate::overlay::MeshStats;
use crate::snapshot_world::FurniturePlacement;
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
                            // Stage F: a material with a macro
                            // gets the macro-encoded layer word as
                            // its top layer — the mesher turns it
                            // into world-based UVs and the raster
                            // shader samples the macro array.
                            let top = match spec.material_macros.get(b.id as usize) {
                                Some(mm) if mm[3] == 1 => {
                                    mesher::mesh::encode_macro_layer(mm[0], mm[2])
                                }
                                _ => ml[0],
                            };
                            stamped.tex_layers = Some([top, ml[1], ml[2]]);
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

/// Phase 9 slice 2 (9.4 dispatch contract): mesh an
/// already-filtered (and already prop-stamped) map chunk by
/// chunk — chunks flagged `has_fine` take
/// `mesher::fine::mesh_chunk_fine`, every other chunk takes
/// the existing greedy `mesh_chunk`. Chunk order is sorted
/// for determinism. With no chunk flagged (nothing stamped),
/// this produces exactly the meshes [`mesh_filtered`] would.
fn mesh_filtered_fine(map: &mesher::chunk::ChunkMap, layer: &FineLayer) -> (PassMeshes, MeshStats) {
    let mut combined = PassMeshes::default();
    let mut n_chunks = 0;
    let mut coords: Vec<[i32; 2]> = map.chunks.keys().copied().collect();
    coords.sort_unstable();
    for coord in coords {
        let chunk = &map.chunks[&coord];
        let pm = if chunk.has_fine {
            mesher::fine::mesh_chunk_fine(map, layer, coord, map.scale)
        } else {
            map.mesh_chunk(coord, true)
        };
        if let Some(pm) = pm {
            for (dst, src) in combined.meshes.iter_mut().zip(pm.meshes.iter()) {
                dst.append(src);
            }
            n_chunks += 1;
        }
    }
    let stats = MeshStats::from_pass_meshes(n_chunks, &combined);
    (combined, stats)
}

/// Phase 9 slice 2: the synthetic-material preparation for a
/// prop scene against the scene's (optional) texture spec.
/// Holds the scene-wide colour set plus the synthetic base id
/// and first appended layer the stamped blocks are resolved
/// against (`snapshot_world::prop_block_for`).
struct PropPrep {
    colors: textures::PropColorSet,
    base_id: u32,
    first_layer: u32,
}

/// Build the [`PropPrep`] for `props`, mutating `spec_opt` as
/// the task's spec order requires:
/// - a textures-dir spec exists: the synthetic base id is
///   `spec.material_layers.len()` and the spec is extended via
///   `textures::extend_texture_spec_with_props` NOW — before
///   any `stamp_tex_layers` run — with
///   `first_layer = spec.layers.len()` captured before the
///   extension, so the layers `prop_block_for` computes for
///   stamped cells match the appended layers;
/// - no textures dir: the base id is 1000 and a prop-only spec
///   (`textures::prop_only_spec`, whose first layer is 0) is
///   built and returned through `spec_opt` for `set_textures`.
///
/// An empty colour set (an empty prop scene) extends/creates
/// nothing: `spec_opt` is left exactly as it was.
fn prep_prop_spec(props: &crate::frame::PropScene, spec_opt: &mut Option<TextureSpec>) -> PropPrep {
    let colors = textures::PropColorSet::from_models(props.models.iter().map(|(_, m)| m));
    match spec_opt {
        Some(spec) => {
            let base_id = spec.material_layers.len() as u32;
            let first_layer = spec.layers.len() as u32;
            if !colors.is_empty() {
                textures::extend_texture_spec_with_props(spec, &colors, base_id);
            }
            PropPrep {
                colors,
                base_id,
                first_layer,
            }
        }
        None => {
            let base_id = 1000;
            if !colors.is_empty() {
                *spec_opt = Some(textures::prop_only_spec(&colors, base_id));
            }
            PropPrep {
                colors,
                base_id,
                first_layer: 0,
            }
        }
    }
}

/// Stamp `placements` of `props` into a fresh [`FineLayer`]
/// over `map` (which the caller has already filtered — and,
/// on the lit path, already visibility-masked), resolving
/// colours through `prep`. Refreshes every chunk's `has_fine`
/// flag from the layer afterwards (the 9.4 dispatch reads the
/// flags, so they must reflect the final layer, not only the
/// chunks `stamp_furniture` touched).
fn stamp_props_into(
    map: &mut mesher::chunk::ChunkMap,
    placements: &[FurniturePlacement],
    props: &crate::frame::PropScene,
    prep: &PropPrep,
    scale: mesher::scale::WorldScale,
) -> Result<FineLayer, String> {
    let mut layer = FineLayer::new(scale.blocks_per_z_level as i32 * FINE);
    if placements.is_empty() || props.models.is_empty() {
        map.refresh_fine_flags(&layer);
        return Ok(layer);
    }
    let block_fn =
        crate::snapshot_world::prop_block_for(&prep.colors, prep.base_id, prep.first_layer);
    let adapters: Vec<(u32, crate::snapshot_world::PropModelCells<'_>)> = props
        .models
        .iter()
        .map(|(id, model)| {
            (
                *id,
                crate::snapshot_world::PropModelCells::new(model, &block_fn),
            )
        })
        .collect();
    let resolve = |id: u32| -> Option<&dyn ModelCells> {
        adapters
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, a)| a as &dyn ModelCells)
    };
    crate::snapshot_world::stamp_furniture(map, &mut layer, placements, scale, &resolve)
        .map_err(|e| e.to_string())?;
    map.refresh_fine_flags(&layer);
    Ok(layer)
}

/// Phase 8 wiring: stamp a vehicle scene's models into a
/// filtered map (their exported absolute part positions,
/// verbatim — `vehicles::stamp_vehicle`). Returns how many
/// models were stamped and how many tiles they occupy. Tiles
/// outside the map's chunks render nothing (the stamp's own
/// contract).
pub fn stamp_vehicles(
    map: &mut mesher::chunk::ChunkMap,
    scene: &crate::frame::VehicleScene,
) -> (usize, usize) {
    let mut tiles = 0usize;
    for model in &scene.models {
        let (occupied, _dirtied) =
            vehicles::stamp_vehicle(map, model, world::HOUSE_MIN_GAME_Z);
        tiles += occupied.len();
    }
    (scene.models.len(), tiles)
}

/// Prepare the fake house for a raster render at the given
/// visible z-levels. Returns the meshes, their stats, and the
/// texture spec to install on the renderer (`None` = flat).
pub fn prepare_raster_house(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    prepare_raster_house_v(visible_levels, textures_dir, None)
}

/// [`prepare_raster_house`] with a vehicle scene stamped into
/// the world before meshing (Phase 8 wiring).
pub fn prepare_raster_house_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    vehicles: Option<&crate::frame::VehicleScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    prepare_raster_v(visible_levels, textures_dir, vehicles, false)
}

/// The sample scene's form of [`prepare_raster_house_v`]
/// (24x24 yard + house; `world::build_sample_scene`).
pub fn prepare_raster_sample_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    vehicles: Option<&crate::frame::VehicleScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    prepare_raster_v(visible_levels, textures_dir, vehicles, true)
}

/// [`prepare_raster_house_v`] with a Phase 9 prop scene
/// (slice 2): props are stamped into a fine layer over the
/// filtered map and flagged chunks mesh at fine resolution.
/// `None` props = the pre-Phase-9 path, exactly.
pub fn prepare_raster_house_vp(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    vehicles: Option<&crate::frame::VehicleScene>,
    props: Option<&crate::frame::PropScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    match props {
        Some(p) => prepare_raster_fine_v(visible_levels, textures_dir, vehicles, p, false),
        None => prepare_raster_v(visible_levels, textures_dir, vehicles, false),
    }
}

/// The sample scene's form of [`prepare_raster_house_vp`].
pub fn prepare_raster_sample_vp(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    vehicles: Option<&crate::frame::VehicleScene>,
    props: Option<&crate::frame::PropScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    match props {
        Some(p) => prepare_raster_fine_v(visible_levels, textures_dir, vehicles, p, true),
        None => prepare_raster_v(visible_levels, textures_dir, vehicles, true),
    }
}

/// The prop-stamping form of the unlit raster preparation
/// (Phase 9 slice 2). Order: build the texture spec (extending
/// it with the synthetic prop materials before any coarse
/// tex-layer stamping), filter the map, stamp coarse texture
/// layers, stamp vehicles, then stamp the props into the fine
/// layer and mesh with the fine dispatch.
fn prepare_raster_fine_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    vehicles: Option<&crate::frame::VehicleScene>,
    props: &crate::frame::PropScene,
    sample: bool,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    let scale = mesher::scale::WorldScale::default();
    let build_map = |scale: mesher::scale::WorldScale| {
        if sample {
            world::build_sample_scene(scale)
        } else {
            world::build_fake_house(scale)
        }
    };
    let mut spec_opt = match textures_dir {
        Some(dir) => {
            let table = materials::MaterialTable::load(dir)
                .map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            Some(spec)
        }
        None => None,
    };
    let prep = prep_prop_spec(props, &mut spec_opt);
    let map = build_map(scale);
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    if let Some(spec) = &spec_opt {
        // Coarse blocks only: stamped fine blocks already carry
        // their layers from `prop_block_for` at stamp time.
        stamp_tex_layers(&mut filtered, spec);
    }
    if let Some(v) = vehicles {
        stamp_vehicles(&mut filtered, v);
    }
    let layer = stamp_props_into(&mut filtered, &props.placements, props, &prep, scale)?;
    let (meshes, stats) = mesh_filtered_fine(&filtered, &layer);
    Ok((meshes, stats, spec_opt))
}

fn prepare_raster_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    vehicles: Option<&crate::frame::VehicleScene>,
    sample: bool,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    let build_map = |scale: mesher::scale::WorldScale| {
        if sample {
            world::build_sample_scene(scale)
        } else {
            world::build_fake_house(scale)
        }
    };
    match textures_dir {
        Some(dir) => {
            let table = materials::MaterialTable::load(dir)
                .map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            let map = build_map(mesher::scale::WorldScale::default());
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            stamp_tex_layers(&mut filtered, &spec);
            if let Some(v) = vehicles {
                stamp_vehicles(&mut filtered, v);
            }
            let (meshes, stats) = mesh_filtered(&filtered);
            Ok((meshes, stats, Some(spec)))
        }
        None => match vehicles {
            Some(v) => {
                let map = build_map(mesher::scale::WorldScale::default());
                let mut filtered =
                    world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
                stamp_vehicles(&mut filtered, v);
                let (meshes, stats) = mesh_filtered(&filtered);
                Ok((meshes, stats, None))
            }
            None if sample => {
                let map = build_map(mesher::scale::WorldScale::default());
                let (meshes, stats) =
                    world::mesh_world(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
                Ok((meshes, stats, None))
            }
            None => {
                let (_map, meshes, stats) = world::mesh_fake_house(visible_levels);
                Ok((meshes, stats, None))
            }
        },
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
    prepare_raster_house_lit_v(visible_levels, textures_dir, lit, None)
}

/// [`prepare_raster_house_lit`] with a vehicle scene stamped
/// into the world *before* the visibility mask, so a vehicle
/// part on a `BLANK` tile is masked out with the world's own
/// blocks — the shared mask governs vehicle geometry exactly
/// like terrain.
pub fn prepare_raster_house_lit_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    prepare_raster_lit_v(visible_levels, textures_dir, lit, vehicles, false)
}

/// The sample scene's form of [`prepare_raster_house_lit_v`].
pub fn prepare_raster_sample_lit_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    prepare_raster_lit_v(visible_levels, textures_dir, lit, vehicles, true)
}

/// [`prepare_raster_house_lit_v`] with a Phase 9 prop scene
/// (slice 2). `None` props = the pre-Phase-9 lit path, exactly.
pub fn prepare_raster_house_lit_vp(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
    props: Option<&crate::frame::PropScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    match props {
        Some(p) => prepare_raster_lit_fine_v(visible_levels, textures_dir, lit, vehicles, p, false),
        None => prepare_raster_lit_v(visible_levels, textures_dir, lit, vehicles, false),
    }
}

/// The sample scene's form of [`prepare_raster_house_lit_vp`].
pub fn prepare_raster_sample_lit_vp(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
    props: Option<&crate::frame::PropScene>,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    match props {
        Some(p) => prepare_raster_lit_fine_v(visible_levels, textures_dir, lit, vehicles, p, true),
        None => prepare_raster_lit_v(visible_levels, textures_dir, lit, vehicles, true),
    }
}

/// The prop-stamping form of the lit raster preparation
/// (Phase 9 slice 2). Differences from the unlit fine path:
/// - the visibility mask is applied BEFORE stamping, and a
///   placement whose tile lost its floor-slab block to the
///   mask is skipped (its prop must not float over a `Hidden`
///   tile);
/// - lighting is the 9.6 v1 rule: the per-vertex bake runs
///   over the combined (coarse + fine) meshes exactly as
///   today — it is position-based, so a fine quad takes the
///   light of the tile containing its position. Props do not
///   occlude the coarse bake; that limit is 9.6's named
///   caveat, not a silent acceptance;
/// - slice-2 deviation: with props present, a `PerBlock` bake
///   mode takes the PerVertex route for that frame instead
///   (the per-block baker runs on the coarse chunk map and
///   cannot carry fine cells; the frame is NOT double-baked).
fn prepare_raster_lit_fine_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
    props: &crate::frame::PropScene,
    sample: bool,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    let scale = mesher::scale::WorldScale::default();
    let build_map = |scale: mesher::scale::WorldScale| {
        if sample {
            world::build_sample_scene(scale)
        } else {
            world::build_fake_house(scale)
        }
    };
    let mut spec_opt = match textures_dir {
        Some(dir) => {
            let table =
                materials::MaterialTable::load(dir).map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            Some(spec)
        }
        None => None,
    };
    let prep = prep_prop_spec(props, &mut spec_opt);
    let map = build_map(scale);
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    if let Some(spec) = &spec_opt {
        stamp_tex_layers(&mut filtered, spec);
    }
    if let Some(v) = vehicles {
        stamp_vehicles(&mut filtered, v);
    }
    apply_visibility_mask(&mut filtered, lit);
    // Skip placements whose tile lost its floor slab to the
    // mask (or to the cutaway filter): the slab block is the
    // level's bottom block at the placement tile.
    let kept: Vec<FurniturePlacement> = props
        .placements
        .iter()
        .copied()
        .filter(|p| {
            let slab_z = scale.level_base_block(p.level) as i32;
            !filtered.block_at(p.x, p.y, slab_z).is_air()
        })
        .collect();
    let layer = stamp_props_into(&mut filtered, &kept, props, &prep, scale)?;
    let (mut meshes, stats) = mesh_filtered_fine(&filtered, &layer);
    // PerVertex route for every bake mode with props (see the
    // deviation note above): bake the combined meshes.
    lighting::bake_pass_meshes(
        &mut meshes,
        &lit.grid,
        lit.env,
        scale,
        world::HOUSE_MIN_GAME_Z,
    );
    Ok((meshes, stats, spec_opt))
}

fn prepare_raster_lit_v(
    visible_levels: &[i32],
    textures_dir: Option<&Path>,
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
    sample: bool,
) -> Result<(PassMeshes, MeshStats, Option<TextureSpec>), String> {
    let scale = mesher::scale::WorldScale::default();
    let build_map = |scale: mesher::scale::WorldScale| {
        if sample {
            world::build_sample_scene(scale)
        } else {
            world::build_fake_house(scale)
        }
    };
    let (spec, mut filtered) = match textures_dir {
        Some(dir) => {
            let table = materials::MaterialTable::load(dir)
                .map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            let map = build_map(scale);
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            stamp_tex_layers(&mut filtered, &spec);
            (Some(spec), filtered)
        }
        None => {
            let map = build_map(scale);
            let filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            (None, filtered)
        }
    };
    if let Some(v) = vehicles {
        stamp_vehicles(&mut filtered, v);
    }
    apply_visibility_mask(&mut filtered, lit);
    if lit.bake_mode == crate::frame::BakeMode::PerBlock {
        // Per-block bake (Phase 7's second baker): every block
        // takes its own tile's presented light BEFORE meshing,
        // so light boundaries are block-color boundaries and the
        // greedy mesher cannot merge across them — a MEMORIZED
        // patch inside a floor mega-quad renders at tile
        // resolution. (The per-vertex path below remains the
        // default: it is the leak-proof air-side baker.)
        let baked = lighting::bake_chunk_map_with_offset(
            &filtered,
            &lit.grid,
            lit.env,
            world::HOUSE_MIN_GAME_Z,
        );
        let (meshes, stats) = mesh_filtered(&baked);
        return Ok((meshes, stats, spec));
    }
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
