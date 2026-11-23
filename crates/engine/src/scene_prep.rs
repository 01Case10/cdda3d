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

/// Stage F part 2: the textured scene's spec + shore + blend
/// assignments, built from the (already cutaway-filtered) map
/// both renderers derive from. Water ids are the fake ids whose
/// voxelizer material carries a shore set; with no shore set in
/// the table the assignment list is empty and the spec is
/// exactly the base spec (the pre-F2 build). Blend assignments
/// cover the map's macro-material borders (`crate::blend`); in
/// the fake scenes no two macro materials are ever adjacent (the
/// yard is one grass material), so the list is empty there and
/// the spec is again the pre-blend one.
fn textured_spec(
    dir: &Path,
    map: &mesher::chunk::ChunkMap,
) -> Result<
    (
        TextureSpec,
        Vec<crate::shore::ShoreAssignment>,
        Vec<crate::blend::BlendAssignment>,
    ),
    String,
> {
    let table = materials::MaterialTable::load(dir).map_err(|e| format!("materials: {e}"))?;
    let id_map = crate::rt::fake_material_ids();
    let water_ids: std::collections::HashSet<u32> = id_map
        .iter()
        .filter(|(_, game_id)| {
            table
                .get(game_id)
                .is_some_and(|m| !m.shore_layers.is_empty())
        })
        .map(|(num, _)| *num)
        .collect();
    let assignments = crate::shore::shore_assignments_for_map(map, &|id| water_ids.contains(&id));
    let macro_spans: std::collections::HashMap<u32, u32> = id_map
        .iter()
        .filter_map(|(num, game_id)| {
            table
                .get(game_id)
                .and_then(|m| m.macro_)
                .map(|mc| (*num, mc.span_tiles))
        })
        .collect();
    let blend_assignments =
        crate::blend::blend_assignments_for_map(map, &|id| macro_spans.get(&id).copied());
    let (spec, _stats) = textures::build_texture_spec_variants(
        &table,
        dir,
        &id_map,
        &[],
        &assignments,
        &blend_assignments,
    )?;
    Ok((spec, assignments, blend_assignments))
}

/// Stage F part 2: stamp shore variant tops onto a filtered
/// map's water blocks. Each water block takes the compact
/// `[top, side, bottom]` of its assignment's virtual entry in
/// `spec` (top = shore texture, side/bottom = base water) —
/// the raster form of the RT path's virtual-id rewrite, driven
/// by the same assignments (`crate::shore`).
fn stamp_shore_tops(
    map: &mut mesher::chunk::ChunkMap,
    spec: &TextureSpec,
    assignments: &[crate::shore::ShoreAssignment],
) {
    if assignments.is_empty() {
        return;
    }
    let masks =
        crate::shore::shore_masks_for_map(map, &|id| assignments.iter().any(|a| a.base_id == id));
    for (pos, mask) in &masks {
        let b = map.block_at(pos[0], pos[1], pos[2]);
        if b.is_air() {
            continue;
        }
        let Some(a) = assignments
            .iter()
            .find(|a| a.base_id == b.id && a.mask == *mask)
        else {
            continue; // interior (0xFF) or unassigned: plain top
        };
        let Some(entry) = spec.material_layers.get(a.virtual_id as usize) else {
            continue;
        };
        if entry[3] != 1 {
            continue;
        }
        let mut stamped = b;
        stamped.tex_layers = Some([entry[0], entry[1], entry[2]]);
        map.set_global_block(pos[0], pos[1], pos[2], stamped);
    }
}

/// Stage F part 2 (border height-blending): stamp blend words
/// onto a filtered map's macro-border blocks. Each block whose
/// chosen border (`crate::blend::blend_neighbor_at`, the same
/// choice the assignments were allocated from) faces a different
/// same-span macro material takes the blend-encoded layer word
/// as its top: own + neighbor compact macro color layers,
/// direction, span — everything the raster shader needs for the
/// Stage D height decision. Side/bottom layers are untouched
/// (the blend is a top-face phenomenon). Blocks whose compact
/// macro indices exceed the blend word's 11-bit fields keep the
/// plain macro word (documented fallback in
/// `mesher::mesh::MACRO_BLEND_BIT`).
fn stamp_blend_tops(
    map: &mut mesher::chunk::ChunkMap,
    spec: &TextureSpec,
    assignments: &[crate::blend::BlendAssignment],
) {
    if assignments.is_empty() {
        return;
    }
    let macro_span = |id: u32| -> Option<u32> {
        spec.material_macros
            .get(id as usize)
            .filter(|mm| mm[3] == 1)
            .map(|mm| mm[2])
    };
    // Collect first (the neighbor choice reads the whole map),
    // then apply — the two phases must not alias.
    let get = |x: i32, y: i32, z: i32| map.block_at(x, y, z).id;
    let mut stamps: Vec<([i32; 3], u32)> = Vec::new();
    for chunk in map.chunks.values() {
        let origin = chunk.origin();
        let size = chunk.grid.size;
        for z in 0..size[2] as i32 {
            for y in 0..size[1] as i32 {
                for x in 0..size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    if b.tex_layers.is_none() {
                        continue;
                    }
                    let Some(&mm) = spec.material_macros.get(b.id as usize) else {
                        continue;
                    };
                    if mm[3] != 1 {
                        continue;
                    }
                    let pos = [origin[0] + x, origin[1] + y, origin[2] + z];
                    let Some((nbr, dir)) = crate::blend::blend_neighbor_at(&get, pos, &macro_span)
                    else {
                        continue;
                    };
                    let nbr_mm = spec.material_macros[nbr as usize];
                    if mm[0] > mesher::mesh::MACRO_BLEND_INDEX_MASK
                        || nbr_mm[0] > mesher::mesh::MACRO_BLEND_INDEX_MASK
                    {
                        continue; // documented fallback: plain macro word
                    }
                    stamps.push((
                        pos,
                        mesher::mesh::encode_macro_blend_layer(mm[0], nbr_mm[0], dir as u32, mm[2]),
                    ));
                }
            }
        }
    }
    for (pos, word) in stamps {
        let b = map.block_at(pos[0], pos[1], pos[2]);
        let Some(layers) = b.tex_layers else {
            continue;
        };
        let mut stamped = b;
        stamped.tex_layers = Some([word, layers[1], layers[2]]);
        map.set_global_block(pos[0], pos[1], pos[2], stamped);
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
pub(crate) struct PropPrep {
    pub(crate) colors: textures::PropColorSet,
    pub(crate) base_id: u32,
    pub(crate) first_layer: u32,
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
pub(crate) fn prep_prop_spec(
    props: &crate::frame::PropScene,
    spec_opt: &mut Option<TextureSpec>,
) -> PropPrep {
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
pub(crate) fn stamp_props_into(
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

/// Terrain models: extend a prop scene with placements
/// for the map's modelled TERRAIN tiles -- trees (the
/// foliage tile's material model, stamped as-is) and
/// chain-link fences (the material's half-segment model,
/// one rotated copy per connected direction, unioned
/// into one composed model per neighbour mask). Modelled
/// tiles are suppressed in `map` (replaced by the ground
/// slab the model stands on) so the block and the model
/// never double up. A fence tile with no fence neighbour keeps its
/// block. Returns the extended scene (a clone; `props`
/// is untouched). Model ids are allocated above the
/// highest id already in `props`.
pub(crate) fn extend_prop_scene_with_terrain(
    map: &mut mesher::chunk::ChunkMap,
    textures_dir: &Path,
    scale: mesher::scale::WorldScale,
    props: &crate::frame::PropScene,
) -> Result<crate::frame::PropScene, String> {
    use std::collections::BTreeSet;
    let table =
        materials::MaterialTable::load(textures_dir).map_err(|e| format!("materials: {e}"))?;
    let fake = crate::rt::fake_material_ids();
    let per = scale.blocks_per_z_level as i32;
    let mut fence: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    let mut trees: Vec<(i32, i32, i32)> = Vec::new();
    for (coord, chunk) in &map.chunks {
        for level in 0..chunk.levels as i32 {
            let bz = level * per;
            for ly in 0..mesher::chunk::CHUNK_H as i32 {
                for lx in 0..mesher::chunk::CHUNK_W as i32 {
                    let id = chunk.grid.get(lx, ly, bz).id;
                    let x = coord[0] * mesher::chunk::CHUNK_W as i32 + lx;
                    let y = coord[1] * mesher::chunk::CHUNK_H as i32 + ly;
                    if id == mesher::block::TILE_FENCE {
                        fence.insert((x, y, level));
                    } else if id == mesher::block::TILE_FOLIAGE {
                        trees.push((x, y, level));
                    }
                }
            }
        }
    }
    if fence.is_empty() && trees.is_empty() {
        return Ok(props.clone());
    }
    let mut out = props.clone();
    let mut next_id = props
        .models
        .iter()
        .map(|(id, _)| *id)
        .max()
        .map_or(0, |m| m + 1);
    let mut suppress: Vec<(i32, i32, i32)> = Vec::new();
    if !trees.is_empty() {
        if let Some(model) = fake
            .get(&mesher::block::TILE_FOLIAGE)
            .and_then(|gid| table.get(gid))
            .and_then(|m| m.model.clone())
        {
            let id = next_id;
            next_id += 1;
            out.models.push((id, model));
            for &(x, y, level) in &trees {
                out.placements.push(FurniturePlacement {
                    x,
                    y,
                    level: level as u32,
                    furn_id: id,
                    quarter_turns: 0,
                });
                suppress.push((x, y, level));
            }
        }
    }
    if !fence.is_empty() {
        if let Some(half) = fake
            .get(&mesher::block::TILE_FENCE)
            .and_then(|gid| table.get(gid))
            .and_then(|m| m.model.clone())
        {
            // Direction bits: W=1 (turns 0), N=2 (1),
            // E=4 (2), S=8 (3).
            let mut by_mask: std::collections::HashMap<u8, u32> = Default::default();
            for &(x, y, level) in &fence {
                let mut mask = 0u8;
                if fence.contains(&(x - 1, y, level)) {
                    mask |= 1;
                }
                if fence.contains(&(x, y - 1, level)) {
                    mask |= 2;
                }
                if fence.contains(&(x + 1, y, level)) {
                    mask |= 4;
                }
                if fence.contains(&(x, y + 1, level)) {
                    mask |= 8;
                }
                if mask == 0 {
                    continue;
                }
                let id = match by_mask.get(&mask) {
                    Some(id) => *id,
                    None => {
                        let mut composed = materials::PropModelData {
                            size: half.size,
                            palette: half.palette.clone(),
                            voxels: vec![materials::PropModelData::EMPTY; half.voxels.len()],
                        };
                        for (bit, turns) in [(1u8, 0u8), (2, 1), (4, 2), (8, 3)] {
                            if mask & bit != 0 {
                                composed.union_from(&half.rotated(turns));
                            }
                        }
                        let id = next_id;
                        next_id += 1;
                        out.models.push((id, composed));
                        by_mask.insert(mask, id);
                        id
                    }
                };
                out.placements.push(FurniturePlacement {
                    x,
                    y,
                    level: level as u32,
                    furn_id: id,
                    quarter_turns: 0,
                });
                suppress.push((x, y, level));
            }
        }
    }
    // Suppression replaces the tile with the ground slab:
    // the model stands ON the ground (its fine origin is
    // the slab top), so airing the whole tile would leave
    // a pit under every tree.
    let ground = mesher::block::block_for_tile(mesher::block::TILE_GROUND);
    for (x, y, level) in suppress {
        for b in 0..per {
            let block = if b == 0 {
                ground
            } else {
                mesher::block::Block::AIR
            };
            map.set_global_block(x, y, level * per + b, block);
        }
    }
    Ok(out)
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
        let (occupied, _dirtied) = vehicles::stamp_vehicle(map, model, world::HOUSE_MIN_GAME_Z);
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
            let table =
                materials::MaterialTable::load(dir).map_err(|e| format!("materials: {e}"))?;
            let (spec, _tex_stats) =
                textures::build_texture_spec(&table, dir, &crate::rt::fake_material_ids())?;
            Some(spec)
        }
        None => None,
    };
    let map = build_map(scale);
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    // Terrain models (trees, fences) join the prop scene
    // before colours/layers are allocated; their tiles
    // are suppressed in the filtered map.
    let extended;
    let props = match textures_dir {
        Some(dir) => {
            extended = extend_prop_scene_with_terrain(&mut filtered, dir, scale, props)?;
            &extended
        }
        None => props,
    };
    let prep = prep_prop_spec(props, &mut spec_opt);
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
            let map = build_map(mesher::scale::WorldScale::default());
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            let (spec, shore_assignments, blend_assignments) = textured_spec(dir, &filtered)?;
            stamp_tex_layers(&mut filtered, &spec);
            stamp_shore_tops(&mut filtered, &spec, &shore_assignments);
            stamp_blend_tops(&mut filtered, &spec, &blend_assignments);
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
    let map = build_map(scale);
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    // Terrain models (trees, fences) join the prop scene
    // before colours/layers are allocated; their tiles
    // are suppressed in the filtered map.
    let extended;
    let props = match textures_dir {
        Some(dir) => {
            extended = extend_prop_scene_with_terrain(&mut filtered, dir, scale, props)?;
            &extended
        }
        None => props,
    };
    let prep = prep_prop_spec(props, &mut spec_opt);
    if let Some(spec) = &spec_opt {
        stamp_tex_layers(&mut filtered, spec);
    }
    if let Some(v) = vehicles {
        stamp_vehicles(&mut filtered, v);
    }
    apply_visibility_mask(&mut filtered, lit);
    apply_sky_exposure_alpha(&mut filtered, lit);
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
    refine_sky_factor_alpha(&mut meshes, lit, scale);
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
            let map = build_map(scale);
            let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
            let (spec, shore_assignments, blend_assignments) = textured_spec(dir, &filtered)?;
            stamp_tex_layers(&mut filtered, &spec);
            stamp_shore_tops(&mut filtered, &spec, &shore_assignments);
            stamp_blend_tops(&mut filtered, &spec, &blend_assignments);
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
    apply_sky_exposure_alpha(&mut filtered, lit);
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
        let (mut meshes, stats) = mesh_filtered(&baked);
        refine_sky_factor_alpha(&mut meshes, lit, scale);
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
    refine_sky_factor_alpha(&mut meshes, lit, scale);
    Ok((meshes, stats, spec))
}

/// Phase 10 slice 10.3 (§5.2 spike, decision 4A): stamp the
/// per-tile sky factor into the block colour's alpha, BEFORE
/// meshing. Alpha is unused for shading in the opaque/cutout
/// passes (it is 1.0 there today); the transparent pass keeps
/// its material alpha and is NOT stamped (its sky factor is the
/// global one in v1 — the §5.2 route is scoped to the opaque
/// pass in the plan's own text). Because the greedy mesher's
/// merge key is the whole `Block` (colour included), a factor
/// boundary breaks merges exactly like a colour boundary —
/// the spike gate proves this at mesher level. Both bakers
/// preserve alpha (`shade_rgba` keeps `albedo[3]`), so the
/// factor survives the bake into the vertex colour, where the
/// raster shader reads it. No exposure data on the scene =
/// no stamping (alpha stays 1.0, GI applies globally).
pub fn apply_sky_exposure_alpha(
    map: &mut mesher::chunk::ChunkMap,
    lit: &crate::frame::LightingScene,
) {
    let Some(exposure) = &lit.sky_exposure else {
        return;
    };
    let per = map.scale.blocks_per_z_level.max(1) as i32;
    for chunk in map.chunks.values_mut() {
        let origin = chunk.origin();
        for z in 0..chunk.grid.size[2] {
            let game_z = lit.min_game_z + (z as i32) / per;
            for y in 0..chunk.grid.size[1] {
                for x in 0..chunk.grid.size[0] {
                    let b = chunk.grid.get(x as i32, y as i32, z as i32);
                    if b.is_air() || b.pass == mesher::block::Pass::Transparent {
                        continue;
                    }
                    let tile = [origin[0] + x as i32, origin[1] + y as i32, game_z];
                    let mut nb = b;
                    nb.color[3] = exposure.factor(tile);
                    chunk.grid.set(x, y, z, nb);
                }
            }
        }
        chunk.dirty = true;
    }
}

/// Phase 10 slice 10.3: per-quad refinement of the stamped
/// sky factor, run AFTER the bake (which preserves alpha).
/// The block stamp keys the factor to the block's OWN tile; a
/// wall block on the perimeter is exposed, but its INNER face
/// fronts a sheltered air tile and must not take sky GI. This
/// pass mirrors `lighting::bake_pass_meshes`' own air-side
/// sampling rule: side faces take the factor of the air tile
/// in front of the face, top/bottom faces of their own tile.
/// The transparent mesh is skipped (material alpha owns that
/// channel). Vertices of fine prop meshes are included — the
/// rule is position-based, like the bake itself (the 9.6
/// caveat applies unchanged).
///
/// Sampling is per QUAD, at the quad's face centre — NOT per
/// vertex at the vertex position (10.3 GPU-gate fix). A greedy-
/// merged quad's corner vertices sit exactly on tile
/// boundaries, and `floor()` of a boundary coordinate lands in
/// the NEIGHBOURING tile: the fake house's merged interior
/// floor quad picked up factor 1 at three of its four corners
/// (from the wall/exterior tiles past its edges) and
/// interpolated noon sky GI across the whole lamp room,
/// failing gate 10.3b on a real adapter (interior mean |diff|
/// 10.55 vs the ≤2 bound). The face centre is strictly inside
/// the quad's own tiles, and every block of a merged quad
/// shares one stamped factor (the merge key is the whole
/// `Block`), so one centre sample per quad is unambiguous.
/// Mesh vertices are per-quad (4 fresh verts per `push_quad`),
/// so the assignment never fights another quad over a vertex.
pub fn refine_sky_factor_alpha(
    meshes: &mut PassMeshes,
    lit: &crate::frame::LightingScene,
    scale: mesher::scale::WorldScale,
) {
    let Some(exposure) = &lit.sky_exposure else {
        return;
    };
    let bs = scale.block_size;
    let per = scale.blocks_per_z_level.max(1) as i32;
    for (pass, mesh) in meshes.meshes.iter_mut().enumerate() {
        if pass == mesher::block::Pass::Transparent.index() {
            continue;
        }
        // Collect first (immutable borrows), then assign.
        let mut assignments: Vec<(usize, f32)> = Vec::new();
        for quad in mesh.indices.chunks_exact(6) {
            let verts: Vec<usize> = quad[..4].iter().map(|&i| i as usize).collect();
            let n = mesh.normals[verts[0]];
            // Face centre, world -> grid (the bake's conversion).
            let mut c = [0.0f32; 3];
            for &i in &verts {
                let p = mesh.positions[i];
                c[0] += p[0] / 4.0;
                c[1] += p[1] / 4.0;
                c[2] += p[2] / 4.0;
            }
            let gc = [c[0] / bs, -c[2] / bs, c[1] / bs];
            let gn = [n[0], -n[2], n[1]];
            let di = lighting::FACE_DIRS
                .iter()
                .position(|d| d[0] as f32 == gn[0] && d[1] as f32 == gn[1] && d[2] as f32 == gn[2])
                .unwrap_or(4);
            let d = lighting::FACE_DIRS[di];
            let axis = (0..3).find(|&a| d[a] != 0).unwrap();
            let sp = [
                gc[0] - gn[0] * 0.5,
                gc[1] - gn[1] * 0.5,
                gc[2] - gn[2] * 0.5,
            ];
            let solid_tile = [
                sp[0].floor() as i32,
                sp[1].floor() as i32,
                (sp[2].floor() as i32).div_euclid(per) + lit.min_game_z,
            ];
            let sample_tile = if axis == 2 {
                solid_tile
            } else {
                [solid_tile[0] + d[0], solid_tile[1] + d[1], solid_tile[2]]
            };
            let f = exposure.factor(sample_tile);
            for &i in &verts {
                assignments.push((i, f));
            }
        }
        for (i, f) in assignments {
            mesh.colors[i][3] = f;
        }
    }
}

/// Drop every block whose tile the shared visibility mask marks
/// `Hidden` (Phase 7 §12 step 3: the mask applied in the filter
/// step). Tiles are addressed in game coordinates: chunk origin
/// (block space) + block offset, z from the block's z-level.
pub fn apply_visibility_mask(map: &mut mesher::chunk::ChunkMap, lit: &crate::frame::LightingScene) {
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
