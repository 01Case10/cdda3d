//! Phase 2c engine integration: the ray-traced renderer behind
//! `--renderer rt` (raster stays the default until the 2c gate's
//! target-hardware frame-time evidence exists — see
//! docs/phase2c-rt.md).
//!
//! The RT world is built from the *same* cutaway-filtered chunk map
//! as the raster path (`world::filtered_map`), so cutaway selection
//! is identical by construction (gate 2c.6). The camera is the same
//! `OrbitCamera`: eye/target are converted world -> grid inside
//! `treebuild::render::RtCamera::from_world`, with the raster path's
//! 60-degree vertical FOV.

use std::time::Instant;

use glam::Vec3;
use mesher::grid::VoxelSampler as _;
use treebuild::build::{build_tree, BuildConfig, BuiltTree};
use treebuild::render::{RtCamera, RtRenderer};
use treebuild::world::VoxelWorld;

use crate::camera::OrbitCamera;
use crate::capture::{CaptureArgs, CaptureError};
use crate::cutaway::CutawayMode;
use crate::world;

/// Fake-data material resolution: mesher fake tile id -> UltiCa
/// game id whose voxelizer material stands in for it. Every id here
/// was verified present in the real UltiCa `materials.json`
/// (2026-11-15). Real scenes resolve through TileDefs instead —
/// this table exists so the fake house can exercise the texture
/// path end to end.
pub fn fake_material_ids() -> std::collections::HashMap<u32, String> {
    use mesher::block::*;
    [
        (TILE_FLOOR_WOOD, "t_floor_primitive"), // rough plank floor
        (TILE_WALL, "t_wall"),
        (TILE_DOOR, "t_door_c"),
        (TILE_WINDOW, "t_window"),
        (TILE_ROOF, "t_shingle_flat_roof"),
        (TILE_GROUND, "t_grass"),
        (TILE_FENCE, "t_chainfence"),
        (TILE_WATER, "t_water_dp"),
        (TILE_FOLIAGE, "t_tree"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.to_string()))
    .collect()
}

/// Phase 2d, world half: rewrite the fake house's multitile voxels
/// in `world` to virtual material ids for their connection masks
/// (same-material adjacency — the fake-data rule). Which numeric
/// ids are multitile comes from the voxelizer table in `dir`
/// through the same fake id map the spec builder uses, so the two
/// halves can never disagree about a material. Must run *before*
/// `build_tree`: the virtual ids ride into the tree's side-table.
/// Returns the assignments the spec builder must emit variants for.
///
/// Materials carrying a shore set (water) are NOT rewritten here:
/// the shore system is water's connection system — an 8-neighbor
/// superset of the cardinal multitile rule — and
/// [`prepare_fake_shore_variants`] recognizes water cells by their
/// base id. Rewriting water to multitile virtual ids first left
/// no cell carrying the base id, so the shore pass silently
/// matched nothing and every water tile kept its plain top.
pub fn prepare_fake_variants(
    world: &mut VoxelWorld,
    dir: &std::path::Path,
) -> Result<Vec<treebuild::connect::VariantAssignment>, CaptureError> {
    prepare_variants_with(world, dir, &fake_material_ids())
}

/// [`prepare_fake_variants`] keyed by a caller-supplied id
/// map (numeric block id -> game string id) — the wire
/// world's form (Lane H), where the numeric ids are wire
/// terrain ids and the map is the scene's
/// [`crate::wire::WireScene::material_ids`].
pub fn prepare_variants_with(
    world: &mut VoxelWorld,
    dir: &std::path::Path,
    id_map: &std::collections::HashMap<u32, String>,
) -> Result<Vec<treebuild::connect::VariantAssignment>, CaptureError> {
    let table = crate::materials::MaterialTable::load_cached(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let multitile_ids: std::collections::HashSet<u32> = id_map
        .iter()
        .filter(|(_, game_id)| {
            table
                .get(game_id)
                .is_some_and(|m| m.multitile && m.shore_layers.is_empty())
        })
        .map(|(num, _)| *num)
        .collect();
    Ok(treebuild::connect::assign_variants(
        world,
        |id| multitile_ids.contains(&id),
        treebuild::connect::same_material,
    ))
}

/// Stage F part 2, world half: rewrite the world's shore water
/// voxels to virtual material ids for their 8-neighbor water
/// masks. Which numeric ids are water comes from the voxelizer
/// table in `dir` (materials carrying a shore set) through the
/// same fake id map the spec builder uses. Must run *after*
/// [`prepare_fake_variants`] (shore virtual ids allocate above
/// the multitile ones) and *before* `build_tree`. Water cells
/// still carry their base ids at this point because
/// `prepare_fake_variants` excludes shore materials from its
/// rewrite. Returns the assignments the spec builder must emit
/// shore entries for.
pub fn prepare_fake_shore_variants(
    world: &mut VoxelWorld,
    dir: &std::path::Path,
) -> Result<Vec<crate::shore::ShoreAssignment>, CaptureError> {
    prepare_shore_variants_with(world, dir, &fake_material_ids())
}

/// [`prepare_fake_shore_variants`] keyed by a caller-supplied
/// id map — the wire world's form (Lane H).
pub fn prepare_shore_variants_with(
    world: &mut VoxelWorld,
    dir: &std::path::Path,
    id_map: &std::collections::HashMap<u32, String>,
) -> Result<Vec<crate::shore::ShoreAssignment>, CaptureError> {
    let table = crate::materials::MaterialTable::load_cached(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let water_ids: std::collections::HashSet<u32> = id_map
        .iter()
        .filter(|(_, game_id)| {
            table
                .get(game_id)
                .is_some_and(|m| !m.shore_layers.is_empty())
        })
        .map(|(num, _)| *num)
        .collect();
    Ok(crate::shore::assign_shore_variants(world, &|id| {
        water_ids.contains(&id)
    }))
}

/// Stage F part 2 (border height-blending), world half: rewrite
/// the world's macro-border voxels to virtual material ids for
/// their `(base, neighbor, dir)` blend triples. Which numeric
/// ids carry a macro (and its span) comes from the voxelizer
/// table in `dir` through the same fake id map the spec builder
/// uses. Must run *after* [`prepare_fake_shore_variants`] (blend
/// virtual ids allocate above the shore ones) and *before*
/// `build_tree`. Returns the assignments the spec builder must
/// emit blend entries for.
pub fn prepare_fake_blend_variants(
    world: &mut VoxelWorld,
    dir: &std::path::Path,
) -> Result<Vec<crate::blend::BlendAssignment>, CaptureError> {
    prepare_blend_variants_with(world, dir, &fake_material_ids())
}

/// [`prepare_fake_blend_variants`] keyed by a caller-supplied
/// id map — the wire world's form (Lane H).
pub fn prepare_blend_variants_with(
    world: &mut VoxelWorld,
    dir: &std::path::Path,
    id_map: &std::collections::HashMap<u32, String>,
) -> Result<Vec<crate::blend::BlendAssignment>, CaptureError> {
    let table = crate::materials::MaterialTable::load_cached(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let macro_spans: std::collections::HashMap<u32, u32> = id_map
        .iter()
        .filter_map(|(num, game_id)| {
            table
                .get(game_id)
                .and_then(|m| m.macro_)
                .map(|mc| (*num, mc.span_tiles))
        })
        .collect();
    Ok(crate::blend::assign_blend_variants(world, &|id| {
        macro_spans.get(&id).copied()
    }))
}

/// Attach voxelizer textures from `dir` to `scene`, resolving the
/// fake house's material ids. `assignments` are the Phase 2d
/// variants [`prepare_fake_variants`] rewrote the world with and
/// `shore_assignments` the Stage F part 2 shore variants
/// [`prepare_fake_shore_variants`] rewrote it with (both empty
/// for an unprepared world — the spec is then exactly the
/// pre-2d base spec). Returns the build stats.
pub fn attach_fake_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &mut treebuild::render::RtScene,
    dir: &std::path::Path,
    assignments: &[treebuild::connect::VariantAssignment],
    shore_assignments: &[crate::shore::ShoreAssignment],
    blend_assignments: &[crate::blend::BlendAssignment],
    veh_mats: Option<&crate::scene_prep::VehicleMaterials>,
) -> Result<crate::textures::TextureStats, CaptureError> {
    attach_textures_with(
        device,
        queue,
        scene,
        dir,
        &fake_material_ids(),
        assignments,
        shore_assignments,
        blend_assignments,
        veh_mats,
    )
}

/// [`attach_fake_textures`] keyed by a caller-supplied id
/// map — the wire world's form (Lane H).
#[allow(clippy::too_many_arguments)]
pub fn attach_textures_with(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &mut treebuild::render::RtScene,
    dir: &std::path::Path,
    base_id_map: &std::collections::HashMap<u32, String>,
    assignments: &[treebuild::connect::VariantAssignment],
    shore_assignments: &[crate::shore::ShoreAssignment],
    blend_assignments: &[crate::blend::BlendAssignment],
    veh_mats: Option<&crate::scene_prep::VehicleMaterials>,
) -> Result<crate::textures::TextureStats, CaptureError> {
    let table = crate::materials::MaterialTable::load_cached(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let mut id_map = base_id_map.clone();
    if let Some(vm) = veh_mats {
        id_map.extend(vm.id_map.iter().map(|(k, v)| (*k, v.clone())));
    }
    let state_assignments: &[crate::textures::StateVariantAssignment] = match veh_mats {
        Some(vm) => &vm.state_assignments,
        None => &[],
    };
    let (spec, stats) = crate::textures::build_texture_spec_all(
        &table,
        dir,
        &id_map,
        assignments,
        shore_assignments,
        blend_assignments,
        state_assignments,
    )
    .map_err(CaptureError::Device)?;
    scene
        .attach_textures(device, queue, &spec)
        .map_err(CaptureError::Device)?;
    Ok(stats)
}

/// The fake-house voxel world for a set of visible game z-levels.
pub fn world_for_levels(visible_levels: &[i32]) -> VoxelWorld {
    world_for_levels_v(visible_levels, None)
}

/// [`world_for_levels`] with a vehicle scene stamped into the
/// filtered map before conversion (Phase 8 wiring — the RT path
/// consumes the same stamped blocks the raster path meshes).
pub fn world_for_levels_v(
    visible_levels: &[i32],
    vehicles: Option<&crate::frame::VehicleScene>,
) -> VoxelWorld {
    world_for_levels_sv(visible_levels, vehicles, false)
}

/// [`world_for_levels_v`] over the sample scene instead of the
/// fake house.
pub fn world_for_levels_sv(
    visible_levels: &[i32],
    vehicles: Option<&crate::frame::VehicleScene>,
    sample: bool,
) -> VoxelWorld {
    world_for_levels_svm(visible_levels, vehicles, sample, None)
}

/// [`world_for_levels_sv`] with resolved vehicle materials: the
/// vehicles stamp with their per-vpart material block ids, so
/// the tree carries them and the attached texture spec shades
/// them with the vparts' sprites.
pub fn world_for_levels_svm(
    visible_levels: &[i32],
    vehicles: Option<&crate::frame::VehicleScene>,
    sample: bool,
    veh_mats: Option<&crate::scene_prep::VehicleMaterials>,
) -> VoxelWorld {
    let scale = mesher::scale::WorldScale::default();
    let map = if sample {
        world::build_sample_scene(scale)
    } else {
        world::build_fake_house(scale)
    };
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    if let Some(v) = vehicles {
        crate::scene_prep::stamp_vehicles_mapped(&mut filtered, v, veh_mats);
    }
    VoxelWorld::from_chunk_map(&filtered)
}

/// Phase 10 (§3.4): the RT sky-exposure feed — option (a),
/// the game's own test (`is_outside && !is_roofed`), exported
/// per tile on the wire since 10.1 and carried into the engine
/// by the light grid's `outside` field since 10.6 wired the
/// proto adapter through. A cell is sky-exposed iff its
/// containing game tile is outside.
///
/// Wave 2's interim feed was an engine-side column scan over
/// a reference world (§3.4 option (b)): exposed iff no opaque
/// cell sits above in the column. It disagreed with the game
/// exactly where geometry and exposure part ways — perimeter
/// wall tiles (a roof slab sits above the wall column, but
/// the game calls the wall tile exposed) and cover the world
/// geometry does not model (a bridge overhead). 10.6 retired
/// the scan; this feed is the wire's only. Exposure is a
/// property of the game world, not of the view, so no
/// reference world is needed: a cutaway cannot un-roof a
/// room because the grid never knew about the cutaway.
pub struct SkyExposure {
    tiles: crate::frame::SkyExposure,
    min_game_z: i32,
    per: i32,
}

impl SkyExposure {
    /// The feed from a scene's light grid (whose tiles carry
    /// the wire `outside` verdicts) at the scene's minimum
    /// game z.
    pub fn from_grid(grid: &lighting::LightGrid, min_game_z: i32) -> Self {
        Self {
            tiles: crate::frame::SkyExposure::from_grid(grid),
            min_game_z,
            per: mesher::scale::WorldScale::default()
                .blocks_per_z_level
                .max(1) as i32,
        }
    }

    /// Exposed iff the containing game tile of block coords
    /// (`x`, `y`, `z`) is outside.
    pub fn exposed(&self, x: i32, y: i32, z: i32) -> bool {
        let tile = [x, y, self.min_game_z + z.div_euclid(self.per)];
        self.tiles.factor(tile) > 0.0
    }

    /// Stamp the feed onto every cell of `world` (coarse coords).
    pub fn apply(&self, world: &mut VoxelWorld) {
        let cells: Vec<([i32; 3], treebuild::world::VoxelCell)> =
            world.iter().map(|(p, c)| (*p, *c)).collect();
        for (pos, mut cell) in cells {
            cell.sky_exposed = self.exposed(pos[0], pos[1], pos[2]);
            world.set(pos, cell);
        }
    }
}

/// Phase 2h.3: the lit form of [`world_for_levels`] — the Phase 7
/// wiring contract (docs/phase7-lighting.md §12, RT half) applied
/// before the tree build:
///
/// 1. A voxel whose tile the shared `VisibilityMask` marks
///    `Hidden` (`BLANK` / never seen) is **skipped entirely** — a
///    BLANK voxel never enters the tree, so no primary, shadow or
///    AO ray can reveal it (the same predicate the raster mask
///    uses; one contract, two renderers).
/// 2. A `MEMORIZED` voxel carries Phase 7's pinned presentation
///    in its color (`lighting::present_color`: 0.30
///    saturation-keep × 0.45 brightness) and `LIT_MEMORIZED` as
///    its side-table lit value — the tracer's scalar shading
///    (sun × `lit_factor` × AO) preserves that saturation, and
///    `FLAG_MEMORIZED` is deliberately *not* set, so the tracer
///    does not apply its own second desaturation on top.
/// 3. A visible voxel keeps its albedo; its side-table lit value
///    is the game's own `LitLevel` (the schema numbering is the
///    treebuild `LIT_*` numbering), so the tracer's per-tile
///    output stays capped by the game's visibility truth via
///    `lit_factor` (Phase 7 §12.3).
pub fn world_for_levels_lit(
    visible_levels: &[i32],
    lit: &crate::frame::LightingScene,
) -> VoxelWorld {
    world_for_levels_lit_v(visible_levels, lit, None)
}

/// [`world_for_levels_lit`] with a vehicle scene: the vehicles
/// are stamped into the base world first, so the visibility
/// contract below governs their voxels exactly like terrain
/// (a part on a BLANK tile never enters the tree).
pub fn world_for_levels_lit_v(
    visible_levels: &[i32],
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
) -> VoxelWorld {
    world_for_levels_lit_sv(visible_levels, lit, vehicles, false)
}

/// [`world_for_levels_lit_v`] over the sample scene.
pub fn world_for_levels_lit_sv(
    visible_levels: &[i32],
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
    sample: bool,
) -> VoxelWorld {
    world_for_levels_lit_svm(visible_levels, lit, vehicles, sample, None)
}

/// [`world_for_levels_lit_sv`] with resolved vehicle materials
/// (see [`world_for_levels_svm`]).
pub fn world_for_levels_lit_svm(
    visible_levels: &[i32],
    lit: &crate::frame::LightingScene,
    vehicles: Option<&crate::frame::VehicleScene>,
    sample: bool,
    veh_mats: Option<&crate::scene_prep::VehicleMaterials>,
) -> VoxelWorld {
    let base = world_for_levels_svm(visible_levels, vehicles, sample, veh_mats);
    apply_lit_contract(base, lit)
}

/// The Phase 7 §12 RT contract applied to an already-built
/// base world (sky exposure from the grid, BLANK voxels dropped,
/// MEMORIZED presentation pinned, visible voxels carrying the
/// game's lit level) — source-agnostic: the fixture worlds
/// above and the wire-fed world below both take it.
fn apply_lit_contract(mut base: VoxelWorld, lit: &crate::frame::LightingScene) -> VoxelWorld {
    // Phase 10 (10.6): sky exposure from the scene's light
    // grid — the wire outside[] feed (see SkyExposure).
    SkyExposure::from_grid(&lit.grid, lit.min_game_z).apply(&mut base);
    let mask = lighting::VisibilityMask::new(&lit.grid);
    let per = mesher::scale::WorldScale::default().blocks_per_z_level.max(1) as i32;
    let mut out = VoxelWorld::new();
    for (pos, cell) in base.iter() {
        // VoxelWorld coords are block coords: x/y are tile units
        // (1 block per tile horizontally), z is blocks.
        let tile = [pos[0], pos[1], lit.min_game_z + pos[2] / per];
        let vis = mask.visibility(tile[0], tile[1], tile[2]);
        if !lighting::include_in_tree(vis) {
            continue; // BLANK: never enters the tree
        }
        let mut cell = *cell;
        let tile_light = lit.grid.get(tile[0], tile[1], tile[2]);
        match vis {
            lighting::Visibility::Memorized => {
                cell.color = lighting::present_color(cell.color, tile_light);
                cell.lit = treebuild::world::LIT_MEMORIZED;
                cell.flags &= !treebuild::world::FLAG_MEMORIZED;
            }
            lighting::Visibility::Visible => {
                cell.lit = tile_light.lit.0;
            }
            lighting::Visibility::Hidden => unreachable!("filtered above"),
        }
        out.set(*pos, cell);
    }
    out
}

/// Wire-fed scene source (Phase 11 follow-up, Lane A): the RT
/// world for a [`crate::wire::WireScene`] — the same cutaway
/// filter + vehicle stamping the fixture worlds take, over the
/// caller's wire-built map instead of a fixture builder. The
/// lit form is this plus [`apply_lit_contract`], exactly as
/// the fixture pair (`world_for_levels_svm` /
/// `world_for_levels_lit_svm`) composes.
pub fn world_from_wire(
    visible_levels: &[i32],
    wire: &crate::wire::WireScene,
    vehicles: Option<&crate::frame::VehicleScene>,
) -> VoxelWorld {
    let mut filtered = world::filtered_map(&wire.map, wire.min_game_z, visible_levels);
    if let Some(v) = vehicles {
        crate::scene_prep::stamp_vehicles_mapped(&mut filtered, v, None);
    }
    VoxelWorld::from_chunk_map(&filtered)
}

/// Visible levels the capture path uses for a player z (same
/// selection as the raster capture: HideAbove(0)).
pub fn capture_visible_levels(cutaway_z: i32) -> Vec<i32> {
    CutawayMode::HideAbove(0).visible_levels(cutaway_z)
}

/// Build the tree for the capture at `cutaway_z` (no textures —
/// the world keeps its base material ids).
pub fn build_capture_tree(cutaway_z: i32) -> Result<BuiltTree, CaptureError> {
    let world = world_for_levels(&capture_visible_levels(cutaway_z));
    build_tree(&world, BuildConfig::default())
        .map_err(|e| CaptureError::Device(format!("treebuild: {e}")))
}

/// The RT camera framing exactly the raster capture's frame.
pub fn capture_camera(args: &CaptureArgs, aspect: f32) -> RtCamera {
    let mut camera = OrbitCamera::new(Vec3::new(args.camera_target.0, 1.0, -args.camera_target.1));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    RtCamera::from_world(camera.eye(), camera.target, 60.0, aspect)
}

/// One RT capture's full result (the PNG pixels plus the depth
/// buffer the parity gates classify silhouettes with).
pub struct RtCapture {
    pub image: image::RgbaImage,
    pub depth: Vec<f32>,
    pub max_dist: f32,
    pub size: (u32, u32),
    pub leaves: usize,
    pub nodes: usize,
    pub build_ms: f64,
    pub distinct_colors: usize,
    /// Texture hookup stats when `args.textures` was set.
    pub texture_stats: Option<crate::textures::TextureStats>,
}

/// A rendered RT frame before readback (Phase 2g.1: the fx
/// wiring composites field volumes + the post stack over these
/// textures, so it needs the frame, not just its pixels).
pub struct RtFrameOut {
    pub color: wgpu::Texture,
    pub depth: wgpu::Buffer,
    pub max_dist: f32,
    pub camera: RtCamera,
    pub size: (u32, u32),
    pub leaves: usize,
    pub nodes: usize,
    /// Phase 9 (9.5): fine-tree leaf/node counts when the frame
    /// carried props (0 on the coarse-only path).
    pub fine_leaves: usize,
    pub fine_nodes: usize,
    pub build_ms: f64,
    pub texture_stats: Option<crate::textures::TextureStats>,
}

/// A fully built RT scene: the renderer (compiled pipelines),
/// the GPU scene (trees uploaded, textures attached) and the
/// build's bookkeeping. Building one is the expensive half of
/// an RT frame — 64-tree builds for the coarse + fine worlds,
/// texture-spec construction (PNG layer decode + mip chains),
/// GPU uploads and compute-pipeline/shader compilation —
/// while tracing a frame from one is just a dispatch. The
/// one-shot capture path builds and drops one per call; the
/// live windowed loop reuses one through [`RtSceneCache`].
pub struct BuiltRtScene {
    renderer: RtRenderer,
    scene: treebuild::render::RtScene,
    leaves: usize,
    nodes: usize,
    fine_leaves: usize,
    fine_nodes: usize,
    build_ms: f64,
    texture_stats: Option<crate::textures::TextureStats>,
}

/// Trace one frame from a built scene. Sky/emitters are
/// per-frame uniforms, (re)uploaded here every call — `None`
/// restores the renderer's defaults (neutral sky / no
/// emitters), which is also what a freshly built scene starts
/// with, so a cached scene never leaks a previous frame's sky.
fn render_built_rt(
    built: &BuiltRtScene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: Option<&lighting::SkyUniforms>,
    emitters: Option<&lighting::EmitterUniforms>,
    build_ms: f64,
) -> RtFrameOut {
    match sky {
        Some(s) => built.scene.set_sky(queue, s),
        None => built.scene.set_sky(queue, &lighting::SkyUniforms::neutral()),
    }
    match emitters {
        Some(e) => built.scene.set_emitters(queue, e),
        None => built
            .scene
            .set_emitters(queue, &lighting::EmitterUniforms::empty()),
    }
    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
    let aspect = width as f32 / height as f32;
    let camera = capture_camera(args, aspect);
    let frame = if args.macro_debug_height {
        built.renderer.render_macro_debug_height(
            device,
            queue,
            &built.scene,
            &camera,
            (width, height),
        )
    } else {
        built
            .renderer
            .render(device, queue, &built.scene, &camera, (width, height), true)
    };
    device.poll(wgpu::Maintain::Wait);
    RtFrameOut {
        color: frame.color,
        depth: frame.depth,
        max_dist: frame.max_dist,
        camera,
        size: (width, height),
        leaves: built.leaves,
        nodes: built.nodes,
        fine_leaves: built.fine_leaves,
        fine_nodes: built.fine_nodes,
        build_ms,
        texture_stats: built.texture_stats.clone(),
    }
}

// ---------------------------------------------------------------------
// Live-loop scene cache (the 0 fps fix, 2026-11-24)
// ---------------------------------------------------------------------

/// Everything about `CaptureArgs` that determines the BUILT RT
/// scene (trees + attached textures). Camera, size, sky and
/// emitters are deliberately absent: they are per-frame
/// uniforms/inputs to the trace, not the build. `bake_mode` and
/// the lighting `env` are absent because the RT build never
/// reads them (the grid + `min_game_z` are what it reads; the
/// raster bake is the env's consumer). Large payloads (light
/// grids, prop model voxels) are folded in as hashes; small
/// payloads are compared exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RtCacheKey {
    snapshot: String,
    cutaway_z: i32,
    textures: Option<std::path::PathBuf>,
    fine_props: bool,
    wire_map: Option<usize>,
    wire_min_game_z: Option<i32>,
    wire_material_ids: Option<Vec<(u32, String)>>,
    wire_props: Option<u64>,
    props: Option<u64>,
    lighting: Option<(i32, u64, Option<u64>)>,
    vehicles: Option<u64>,
}

fn hash_prop_scene(scene: &crate::frame::PropScene) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    scene.models.len().hash(&mut h);
    for (id, m) in &scene.models {
        id.hash(&mut h);
        m.size.hash(&mut h);
        m.palette.hash(&mut h);
        m.voxels.len().hash(&mut h);
        for v in &m.voxels {
            v.hash(&mut h);
        }
    }
    scene.placements.len().hash(&mut h);
    for p in &scene.placements {
        p.x.hash(&mut h);
        p.y.hash(&mut h);
        p.level.hash(&mut h);
        p.furn_id.hash(&mut h);
        p.quarter_turns.hash(&mut h);
    }
    h.finish()
}

fn hash_lighting(lit: &crate::frame::LightingScene) -> (i32, u64, Option<u64>) {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    lit.grid.origin.hash(&mut h);
    lit.grid.size.hash(&mut h);
    for (pos, t) in lit.grid.iter_tiles() {
        pos.hash(&mut h);
        t.lit.0.hash(&mut h);
        t.seen.hash(&mut h);
        for c in t.tint {
            c.to_bits().hash(&mut h);
        }
        t.occluding.hash(&mut h);
        t.outside.hash(&mut h);
    }
    let exposure = lit.sky_exposure.as_ref().map(|e| {
        let mut eh = std::collections::hash_map::DefaultHasher::new();
        let mut tiles: Vec<[i32; 3]> = e.sheltered.iter().copied().collect();
        tiles.sort_unstable();
        tiles.hash(&mut eh);
        eh.finish()
    });
    (lit.min_game_z, h.finish(), exposure)
}

fn hash_vehicles(v: &crate::frame::VehicleScene) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.models.len().hash(&mut h);
    for m in &v.models {
        m.id.hash(&mut h);
        m.name.hash(&mut h);
        m.face_deg.to_bits().hash(&mut h);
        m.parts.len().hash(&mut h);
        for p in &m.parts {
            p.key.vpart.hash(&mut h);
            p.key.mount.hash(&mut h);
            p.abs_pos.hash(&mut h);
            p.hp_frac.hash(&mut h);
            p.open.hash(&mut h);
            p.broken.hash(&mut h);
            p.roof.hash(&mut h);
        }
    }
    let mut names: Vec<(&u32, &String)> = v.vpart_names.iter().collect();
    names.sort_unstable();
    names.len().hash(&mut h);
    for (k, s) in names {
        k.hash(&mut h);
        s.hash(&mut h);
    }
    h.finish()
}

impl RtCacheKey {
    fn from_args(args: &CaptureArgs) -> Self {
        Self {
            snapshot: args.snapshot.clone(),
            cutaway_z: args.cutaway_z,
            textures: args.textures.clone(),
            fine_props: args.fine_props,
            wire_map: args.wire.as_ref().map(|w| std::sync::Arc::as_ptr(&w.map) as usize),
            wire_min_game_z: args.wire.as_ref().map(|w| w.min_game_z),
            wire_material_ids: args.wire.as_ref().and_then(|w| {
                w.material_ids.as_ref().map(|ids| {
                    let mut v: Vec<(u32, String)> =
                        ids.iter().map(|(k, s)| (*k, s.clone())).collect();
                    v.sort_unstable();
                    v
                })
            }),
            wire_props: args
                .wire
                .as_ref()
                .and_then(|w| w.props.as_ref().map(hash_prop_scene)),
            props: args.props.as_ref().map(hash_prop_scene),
            lighting: args.lighting.as_ref().map(hash_lighting),
            vehicles: args.vehicles.as_ref().map(hash_vehicles),
        }
    }
}

/// A built RT scene held for reuse across live frames, with
/// the key it was built from. The windowed app owns one; a
/// frame whose key matches traces from the held scene (camera
/// moves, resizes, sky/time changes and emitter changes are
/// all per-frame uniforms), and a mismatching frame rebuilds
/// and replaces it. Without this, every live frame repaid the
/// full build — measured 2026-11-24 at ~10.5 s/frame for the
/// street_block demo (RT + textures + props + lighting) on a
/// software adapter, i.e. the title-bar 0 fps Joey hit on his
/// PC, where each build is faster but still far over a frame.
pub struct RtSceneCache {
    key: RtCacheKey,
    built: BuiltRtScene,
    builds: usize,
}

impl RtSceneCache {
    /// How many scenes this cache has built (1 = every frame
    /// since the first was a reuse).
    pub fn build_count(&self) -> usize {
        self.builds
    }
}

/// [`render_rt_frame_sky_emitters`] for a caller that renders
/// many frames of a scene (the live windowed loop): the built
/// scene is cached in `cache` and rebuilt only when the
/// scene-determining inputs change.
pub fn render_rt_frame_cached(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: Option<&lighting::SkyUniforms>,
    emitters: Option<&lighting::EmitterUniforms>,
    cache: &mut Option<RtSceneCache>,
) -> Result<RtFrameOut, CaptureError> {
    let key = RtCacheKey::from_args(args);
    if let Some(c) = cache.as_ref() {
        if c.key == key {
            return Ok(render_built_rt(&c.built, device, queue, args, sky, emitters, 0.0));
        }
    }
    let builds = cache.as_ref().map(|c| c.builds).unwrap_or(0) + 1;
    let built = build_rt_scene(device, queue, args)?;
    let out = render_built_rt(&built, device, queue, args, sky, emitters, built.build_ms);
    *cache = Some(RtSceneCache { key, built, builds });
    Ok(out)
}

/// Render one RT frame on an already-created device/queue,
/// returning the GPU frame (color texture + linear depth
/// buffer). [`capture_rt_with_device`] is this plus readback.
pub fn render_rt_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> Result<RtFrameOut, CaptureError> {
    render_rt_frame_inner(device, queue, args, None, None)
}

/// Phase 10 (10.4): [`render_rt_frame`] with the shared sky
/// uniforms driving the RT sky background + GI. The block must
/// already be in **grid space** (east, south, up): pass either
/// `lighting::SkyUniforms::neutral()` as-is (already grid-space;
/// do NOT convert it) or
/// `lighting::SkyUniforms::from_sky_state(...).to_grid_space()`
/// — `from_sky_state` emits SkyState space (east, north, up),
/// and the conversion belongs to the caller because this entry
/// point cannot distinguish a neutral block from a converted
/// one. The frame-level plumbing that evaluates a `SkyState`
/// once per frame and hands it here is §6/10.6 integration work
/// (the 10.3 slice owns `frame.rs` / `CaptureArgs`); this entry
/// point is the RT consumption surface it will call. `None` in
/// the inner path = the renderer's default
/// `SkyUniforms::neutral()`.
pub fn render_rt_frame_sky(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: &lighting::SkyUniforms,
) -> Result<RtFrameOut, CaptureError> {
    render_rt_frame_inner(device, queue, args, Some(sky), None)
}

/// Addendum 11 (emitter prototype): [`render_rt_frame`] with the
/// frame's sky and/or declared emitters. Either block may be
/// `None` (the renderer's default for that input: neutral sky /
/// empty emitter set). Both blocks must already be in **grid
/// space** — the sky per [`render_rt_frame_sky`]'s contract, the
/// emitters via `lighting::EmitterUniforms::to_grid_space`. With
/// both `None` this is exactly [`render_rt_frame`].
pub fn render_rt_frame_sky_emitters(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: Option<&lighting::SkyUniforms>,
    emitters: Option<&lighting::EmitterUniforms>,
) -> Result<RtFrameOut, CaptureError> {
    render_rt_frame_inner(device, queue, args, sky, emitters)
}

fn render_rt_frame_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: Option<&lighting::SkyUniforms>,
    emitters: Option<&lighting::EmitterUniforms>,
) -> Result<RtFrameOut, CaptureError> {
    let built = build_rt_scene(device, queue, args)?;
    Ok(render_built_rt(
        &built,
        device,
        queue,
        args,
        sky,
        emitters,
        built.build_ms,
    ))
}

/// Build the RT scene for `args` (dispatch + world builds +
/// tree builds + texture attach), without tracing. The
/// one-shot entry points build, trace once and drop it;
/// [`render_rt_frame_cached`] holds it in an [`RtSceneCache`].
fn build_rt_scene(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> Result<BuiltRtScene, CaptureError> {
    // Phase 9 (9.5): the effective prop scene — an explicit
    // `props` scene wins regardless of the flag; with only
    // `--fine-props` set, the snapshot's fixture scene is used
    // (the same rule as the raster dispatch in frame.rs). With
    // neither, the coarse-only path below runs untouched.
    let fixture_props: Option<crate::frame::PropScene> =
        if args.props.is_none() && args.fine_props {
            crate::frame::PropScene::fixture(&args.snapshot)
        } else {
            None
        };
    // Wire-fed source (Phase 11 follow-up, Lane A): the props
    // path builds its worlds from fixture builders, so a wire
    // frame takes the coarse path below regardless of the
    // FIXTURE prop inputs (the fixture-name dispatch is
    // untouched when no wire scene is seated). Since Lane G
    // a wire scene can carry its OWN prop scene (derived
    // from its snapshot), which takes the wire props path.
    if let Some(wire) = &args.wire {
        if let Some(props) = &wire.props {
            return build_rt_scene_wire_props(device, queue, args, wire, props);
        }
    }
    if args.wire.is_none() {
        if let Some(props) = args.props.as_ref().or(fixture_props.as_ref()) {
            return build_rt_scene_props(device, queue, args, props);
        }
    }
    let t0 = Instant::now();
    // Phase 2h.3: with a lighting scene, the world is built
    // through the Phase 7 visibility contract (BLANK voxels never
    // enter the tree, MEMORIZED voxels carry the pinned
    // presentation) — before any variant preparation, so hidden
    // tiles also cannot contribute connection masks.
    // Phase 2d: with textures, the world is variant-prepared
    // *before* the tree build so the virtual material ids land in
    // the side-table; without textures the world is untouched and
    // the render is exactly the pre-2d one.
    let levels = capture_visible_levels(args.cutaway_z);
    let sample = args.snapshot == "sample-scene";
    // Vehicle materials: with textures and a named vehicle
    // scene, resolve the vparts once — the world stamps with
    // the material block ids and the attach below gives those
    // ids their sprite layers. Without either, veh_mats is None
    // and the vehicles render flat, byte-identical to before.
    // The fixture texture hookups (vehicle materials, variant
    // / shore / blend preparation, texture attach below) are
    // keyed to the fake tile ids. A wire world takes the same
    // hookups keyed by its own material id map (Lane H —
    // wire terrain id -> string id) when a textures dir is
    // seated; without the map it renders its resolver's flat
    // colours, exactly as in v1.
    let wire = args.wire.as_ref();
    let veh_mats = match (wire, &args.textures, &args.vehicles) {
        (None, Some(dir), Some(v)) => {
            let table = crate::materials::MaterialTable::load_cached(dir)
                .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
            Some(crate::scene_prep::vehicle_materials(v, &table))
        }
        _ => None,
    };
    let mut voxel_world = match (wire, &args.lighting) {
        (Some(w), Some(lit)) => {
            apply_lit_contract(world_from_wire(&levels, w, args.vehicles.as_ref()), lit)
        }
        (Some(w), None) => world_from_wire(&levels, w, args.vehicles.as_ref()),
        (None, Some(lit)) => world_for_levels_lit_svm(
            &levels,
            lit,
            args.vehicles.as_ref(),
            sample,
            veh_mats.as_ref(),
        ),
        (None, None) => world_for_levels_svm(
            &levels,
            args.vehicles.as_ref(),
            sample,
            veh_mats.as_ref(),
        ),
    };
    let assignments = match (wire, &args.textures) {
        (None, Some(dir)) => prepare_fake_variants(&mut voxel_world, dir)?,
        _ => Vec::new(),
    };
    // Stage F part 2: shore preparation after the multitile
    // preparation (its virtual ids allocate above theirs); with
    // no textures the world is untouched, byte-for-byte the
    // pre-F2 render.
    let shore_assignments = match (wire, &args.textures) {
        (None, Some(dir)) => prepare_fake_shore_variants(&mut voxel_world, dir)?,
        _ => Vec::new(),
    };
    // Stage F part 2 (blend): after the shore preparation (its
    // virtual ids allocate above theirs); with no textures the
    // world is untouched, byte-for-byte the pre-blend render.
    let blend_assignments = match (wire, &args.textures) {
        (None, Some(dir)) => prepare_fake_blend_variants(&mut voxel_world, dir)?,
        (Some(w), Some(dir)) => match &w.material_ids {
            Some(ids) => prepare_blend_variants_with(&mut voxel_world, dir, ids)?,
            None => Vec::new(),
        },
        _ => Vec::new(),
    };
    let built = build_tree(&voxel_world, BuildConfig::default())
        .map_err(|e| CaptureError::Device(format!("treebuild: {e}")))?;
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let (renderer, mut scene) = RtRenderer::with_scene(device, queue, &built);
    let texture_stats = match (wire, &args.textures) {
        (None, Some(dir)) => Some(attach_fake_textures(
            device,
            queue,
            &mut scene,
            dir,
            &assignments,
            &shore_assignments,
            &blend_assignments,
            veh_mats.as_ref(),
        )?),
        (Some(w), Some(dir)) => match &w.material_ids {
            Some(ids) => Some(attach_textures_with(
                device,
                queue,
                &mut scene,
                dir,
                ids,
                &assignments,
                &shore_assignments,
                &blend_assignments,
                None,
            )?),
            None => None,
        },
        _ => None,
    };
    Ok(BuiltRtScene {
        renderer,
        scene,
        leaves: built.leaf_count(),
        nodes: built.node_count(),
        fine_leaves: 0,
        fine_nodes: 0,
        build_ms,
        texture_stats,
    })
}

/// Geometric occlusion on the RT path (Stage H, Joey's
/// 8c-A, 2026-11-23; the raster twin is the per-face factor
/// in `mesher::fine::mesh_chunk_fine`). RT `lit` is the
/// game's categorical scale, so the continuous
/// `cell_occlusion` factor becomes a step-down: a cell
/// whose exposed faces average below 0.85 loses one lit
/// level, below 0.70 two levels, never below LIT_DARK.
/// Only game-lit cells (lit <= LIT_BRIGHT) are stepped —
/// Memorized and Blank presentations are pinned by the
/// Phase 7 contract and pass through untouched. Invented
/// lighting, like the raster form: it darkens cells
/// enclosed by or tucked behind prop volumes, which the
/// per-tile game light cannot express.
pub fn fine_occlusion_rewrite(
    world: &VoxelWorld,
    coarse: &mesher::chunk::ChunkMap,
    layer: &mesher::fine::FineLayer,
) -> VoxelWorld {
    let sampler = mesher::fine::FineSampler::new(coarse, layer);
    let mut out = VoxelWorld::new();
    for (pos, cell) in world.iter() {
        let mut cell = *cell;
        if cell.lit <= treebuild::world::LIT_BRIGHT {
            let occ = mesher::fine::cell_occlusion(&sampler, pos[0], pos[1], pos[2]);
            let step = if occ < 0.70 {
                2
            } else if occ < 0.85 {
                1
            } else {
                0
            };
            cell.lit = cell.lit.saturating_sub(step);
        }
        out.set(*pos, cell);
    }
    out
}

/// The Phase 7 visibility contract as a post-pass over an
/// already-built world (the body of [`world_for_levels_lit_sv`],
/// reusable over the fine world): `Hidden` tiles' cells are
/// dropped, `Memorized` cells take the pinned presentation,
/// `Visible` cells take their tile's `LitLevel`. With `fine`,
/// the world is in fine coordinates (`mesher::fine::FINE` cells
/// per block): a fine cell takes its CONTAINING tile's light —
/// the 9.6 v1 rule (docs/phase9-fine.md §1).
fn lit_rewrite(
    world: &VoxelWorld,
    lit: &crate::frame::LightingScene,
    fine: bool,
    exposure: &SkyExposure,
) -> VoxelWorld {
    let mask = lighting::VisibilityMask::new(&lit.grid);
    let per = mesher::scale::WorldScale::default().blocks_per_z_level.max(1) as i32;
    let mut out = VoxelWorld::new();
    for (pos, cell) in world.iter() {
        let tile = if fine {
            [
                pos[0].div_euclid(mesher::fine::FINE),
                pos[1].div_euclid(mesher::fine::FINE),
                lit.min_game_z + pos[2] / (per * mesher::fine::FINE),
            ]
        } else {
            [pos[0], pos[1], lit.min_game_z + pos[2] / per]
        };
        // Phase 10 (10.4): sky exposure for this cell — fine
        // cells take their containing coarse column's exposure.
        let mut cell = *cell;
        cell.sky_exposed = if fine {
            exposure.exposed(
                pos[0].div_euclid(mesher::fine::FINE),
                pos[1].div_euclid(mesher::fine::FINE),
                pos[2].div_euclid(mesher::fine::FINE),
            )
        } else {
            exposure.exposed(pos[0], pos[1], pos[2])
        };
        let vis = mask.visibility(tile[0], tile[1], tile[2]);
        if !lighting::include_in_tree(vis) {
            continue; // BLANK: never enters the tree
        }
        let tile_light = lit.grid.get(tile[0], tile[1], tile[2]);
        match vis {
            lighting::Visibility::Memorized => {
                cell.color = lighting::present_color(cell.color, tile_light);
                cell.lit = treebuild::world::LIT_MEMORIZED;
                cell.flags &= !treebuild::world::FLAG_MEMORIZED;
            }
            lighting::Visibility::Visible => {
                cell.lit = tile_light.lit.0;
            }
            lighting::Visibility::Hidden => unreachable!("filtered above"),
        }
        out.set(*pos, cell);
    }
    out
}

/// Shift every fine-world cell's material id by
/// `to_base - from_base` (both are the synthetic base ids the
/// colour set was / will be addressed at: stamped ids are
/// `from_base + 1 + i`, the attached spec's entries are
/// `to_base + 1 + i`).
fn shift_fine_ids(world: &mut VoxelWorld, from_base: u32, to_base: u32) {
    if from_base == to_base {
        return;
    }
    let delta = to_base as i32 - from_base as i32;
    let cells: Vec<([i32; 3], treebuild::world::VoxelCell)> =
        world.iter().map(|(p, c)| (*p, *c)).collect();
    for (pos, mut cell) in cells {
        cell.material_id = (cell.material_id as i32 + delta) as u32;
        world.set(pos, cell);
    }
}

/// Phase 9 (9.5): the prop-carrying form of [`render_rt_frame`].
/// Mirrors the raster fine preparation
/// (`scene_prep::prepare_raster_fine_v` /
/// `prepare_raster_lit_fine_v`) into two trees: props are
/// stamped into a `FineLayer` over the cutaway-filtered map
/// (suppressing the coarse furniture blocks), the coarse tree
/// is built from the stamped map, and the fine tree from
/// `VoxelWorld::from_fine_layer` — composited by the shader's
/// two-tree trace. With a lighting scene the visibility mask
/// is applied before stamping and placements on `Hidden`
/// tiles are skipped, exactly as on the raster lit path; the
/// fine world then takes the same per-tile lit rewrite as the
/// coarse world (9.6 v1 rule).
///
/// Texture-spec order: stamping fixes synthetic ids against
/// the base spec (or base 1000 untextured); the ATTACHED spec
/// is the variant spec the coarse-only path builds, extended
/// with the prop colours at its own end, and the fine ids are
/// shifted onto it — variant virtual ids allocate above the
/// coarse world's max id and the shift lands the synthetic
/// ids above the whole attached spec, so the namespaces
/// cannot collide.
fn build_rt_scene_props(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    props: &crate::frame::PropScene,
) -> Result<BuiltRtScene, CaptureError> {
    let t0 = Instant::now();
    let levels = capture_visible_levels(args.cutaway_z);
    let sample = args.snapshot == "sample-scene";
    let scale = mesher::scale::WorldScale::default();
    let map = if sample {
        world::build_sample_scene(scale)
    } else {
        world::build_fake_house(scale)
    };
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, &levels);
    // Stage-G: vehicle fine models join the prop scene;
    // their covered tiles' coarse stamps are suppressed.
    let veh_fine = match (&args.vehicles, &args.textures) {
        (Some(v), Some(dir)) => crate::scene_prep::vehicle_fine_scene(v, dir)
            .map_err(CaptureError::Device)?,
        _ => None,
    };
    let combined;
    let props = match &veh_fine {
        Some((vscene, _)) => {
            combined = crate::scene_prep::merge_prop_scenes(props, vscene);
            &combined
        }
        None => props,
    };
    let covered: std::collections::HashSet<[i32; 3]> = match &veh_fine {
        Some((_, c)) => c.clone(),
        None => Default::default(),
    };
    if let Some(v) = &args.vehicles {
        crate::scene_prep::stamp_vehicles_except(&mut filtered, v, &covered);
    }
    let kept: Vec<crate::snapshot_world::FurniturePlacement>;
    let placements: &[crate::snapshot_world::FurniturePlacement] = match &args.lighting {
        Some(lit) => {
            crate::scene_prep::apply_visibility_mask(&mut filtered, lit);
            kept = props
                .placements
                .iter()
                .copied()
                .filter(|p| {
                    let slab_z = scale.level_base_block(p.level) as i32;
                    !filtered.block_at(p.x, p.y, slab_z).is_air()
                })
                .collect();
            &kept
        }
        None => &props.placements,
    };
    // Terrain models (trees, fences) join the scene;
    // their placements append to the (possibly
    // visibility-filtered) furniture placements.
    let mut all_placements: Vec<crate::snapshot_world::FurniturePlacement> =
        placements.to_vec();
    let extended;
    let props = match &args.textures {
        Some(dir) => {
            extended = crate::scene_prep::extend_prop_scene_with_terrain(
                &mut filtered,
                dir,
                scale,
                props,
            )
            .map_err(CaptureError::Device)?;
            all_placements
                .extend_from_slice(&extended.placements[props.placements.len()..]);
            &extended
        }
        None => props,
    };
    let placements: &[crate::snapshot_world::FurniturePlacement] = &all_placements;
    let mut spec_opt = match &args.textures {
        Some(dir) => {
            let table = crate::materials::MaterialTable::load_cached(dir)
                .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
            let (spec, _stats) =
                crate::textures::build_texture_spec(&table, dir, &fake_material_ids())
                    .map_err(CaptureError::Device)?;
            Some(spec)
        }
        None => None,
    };
    let prep = crate::scene_prep::prep_prop_spec(props, &mut spec_opt);
    let layer = crate::scene_prep::stamp_props_into(&mut filtered, placements, props, &prep, scale)
        .map_err(CaptureError::Device)?;
    let mut coarse_world = VoxelWorld::from_chunk_map(&filtered);
    let mut fine_world = VoxelWorld::from_fine_layer(&layer);
    if let Some(lit) = &args.lighting {
        // Phase 10 (10.6): exposure from the scene's light
        // grid (the wire outside[] feed), as in
        // world_for_levels_lit_sv.
        let exposure = SkyExposure::from_grid(&lit.grid, lit.min_game_z);
        coarse_world = lit_rewrite(&coarse_world, lit, false, &exposure);
        fine_world = lit_rewrite(&fine_world, lit, true, &exposure);
        // Geometric fine-prop occlusion (8c-A): after the
        // game's per-tile light is applied, step down fine
        // cells the stamped geometry encloses.
        fine_world = fine_occlusion_rewrite(&fine_world, &filtered, &layer);
    }
    let assignments = match &args.textures {
        Some(dir) => prepare_fake_variants(&mut coarse_world, dir)?,
        None => Vec::new(),
    };
    let shore_assignments = match &args.textures {
        Some(dir) => prepare_fake_shore_variants(&mut coarse_world, dir)?,
        None => Vec::new(),
    };
    let blend_assignments = match &args.textures {
        Some(dir) => prepare_fake_blend_variants(&mut coarse_world, dir)?,
        None => Vec::new(),
    };
    let mut texture_stats = None;
    let attach_spec: Option<treebuild::render::TextureSpec> = match &args.textures {
        Some(dir) => {
            let table = crate::materials::MaterialTable::load_cached(dir)
                .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
            let (mut spec, stats) = crate::textures::build_texture_spec_variants(
                &table,
                dir,
                &fake_material_ids(),
                &assignments,
                &shore_assignments,
                &blend_assignments,
            )
            .map_err(CaptureError::Device)?;
            texture_stats = Some(stats);
            if !prep.colors.is_empty() {
                let final_base = spec.material_layers.len() as u32;
                crate::textures::extend_texture_spec_with_props(
                    &mut spec,
                    &prep.colors,
                    final_base,
                );
                shift_fine_ids(&mut fine_world, prep.base_id, final_base);
            }
            Some(spec)
        }
        None => {
            // `prep_prop_spec` already built the prop-only spec
            // at the stamp-time base id — no shift needed.
            if prep.colors.is_empty() {
                None
            } else {
                spec_opt
            }
        }
    };
    let built = build_tree(&coarse_world, BuildConfig::default())
        .map_err(|e| CaptureError::Device(format!("treebuild: {e}")))?;
    let fine_built = if fine_world.is_empty() {
        None
    } else {
        Some(
            build_tree(&fine_world, BuildConfig::default())
                .map_err(|e| CaptureError::Device(format!("treebuild fine: {e}")))?,
        )
    };
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let (renderer, mut scene) =
        RtRenderer::with_scene_fine(device, queue, &built, fine_built.as_ref());
    if let Some(spec) = &attach_spec {
        scene
            .attach_textures(device, queue, spec)
            .map_err(CaptureError::Device)?;
    }
    Ok(BuiltRtScene {
        renderer,
        scene,
        leaves: built.leaf_count(),
        nodes: built.node_count(),
        fine_leaves: fine_built.as_ref().map(|b| b.leaf_count()).unwrap_or(0),
        fine_nodes: fine_built.as_ref().map(|b| b.node_count()).unwrap_or(0),
        build_ms,
        texture_stats,
    })
}

/// The wire form of [`build_rt_scene_props`] (Lane G): the
/// same two-tree build (coarse world + fine prop world, lit
/// rewrites, fine occlusion) over the wire scene's map and
/// its own snapshot-derived prop scene. The fixture-keyed
/// steps of the fixture form do not exist here: no terrain
/// extension (fixture-keyed to the texture pass), vehicles
/// stamp through the wire path's mapped form, exactly as
/// [`world_from_wire`] stamps them. Textures (Lane H): with
/// a textures dir and the scene's material id map, the
/// coarse world takes the full texture spec + variant
/// preparation keyed by wire ids (the fixture form's shape
/// with the fake id map swapped for the wire one); without
/// either, the wire world renders its resolver's flat
/// colours + the prop-only spec, exactly as Lane G left it.
fn build_rt_scene_wire_props(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    wire: &crate::wire::WireScene,
    props: &crate::frame::PropScene,
) -> Result<BuiltRtScene, CaptureError> {
    let t0 = Instant::now();
    let levels = capture_visible_levels(args.cutaway_z);
    let scale = mesher::scale::WorldScale::default();
    let mut filtered = world::filtered_map(&wire.map, wire.min_game_z, &levels);
    if let Some(v) = &args.vehicles {
        crate::scene_prep::stamp_vehicles_mapped(&mut filtered, v, None);
    }
    let kept: Vec<crate::snapshot_world::FurniturePlacement>;
    let placements: &[crate::snapshot_world::FurniturePlacement] = match &args.lighting {
        Some(lit) => {
            crate::scene_prep::apply_visibility_mask(&mut filtered, lit);
            kept = props
                .placements
                .iter()
                .copied()
                .filter(|p| {
                    let slab_z = scale.level_base_block(p.level) as i32;
                    !filtered.block_at(p.x, p.y, slab_z).is_air()
                })
                .collect();
            &kept
        }
        None => &props.placements,
    };
    // Lane H texture seating: both halves (dir + id map) or
    // neither, as everywhere on the wire path.
    let tex_ids = match (&args.textures, &wire.material_ids) {
        (Some(dir), Some(ids)) => Some((dir, ids)),
        _ => None,
    };
    let mut spec_opt = match tex_ids {
        Some((dir, ids)) => {
            let table = crate::materials::MaterialTable::load_cached(dir)
                .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
            let (spec, _stats) = crate::textures::build_texture_spec(&table, dir, ids)
                .map_err(CaptureError::Device)?;
            Some(spec)
        }
        None => None,
    };
    let prep = crate::scene_prep::prep_prop_spec(props, &mut spec_opt);
    let layer = crate::scene_prep::stamp_props_into(&mut filtered, placements, props, &prep, scale)
        .map_err(CaptureError::Device)?;
    let mut coarse_world = VoxelWorld::from_chunk_map(&filtered);
    let mut fine_world = VoxelWorld::from_fine_layer(&layer);
    if let Some(lit) = &args.lighting {
        let exposure = SkyExposure::from_grid(&lit.grid, lit.min_game_z);
        coarse_world = lit_rewrite(&coarse_world, lit, false, &exposure);
        fine_world = lit_rewrite(&fine_world, lit, true, &exposure);
        fine_world = fine_occlusion_rewrite(&fine_world, &filtered, &layer);
    }
    let assignments = match tex_ids {
        Some((dir, ids)) => prepare_variants_with(&mut coarse_world, dir, ids)?,
        None => Vec::new(),
    };
    let shore_assignments = match tex_ids {
        Some((dir, ids)) => prepare_shore_variants_with(&mut coarse_world, dir, ids)?,
        None => Vec::new(),
    };
    let blend_assignments = match tex_ids {
        Some((dir, ids)) => prepare_blend_variants_with(&mut coarse_world, dir, ids)?,
        None => Vec::new(),
    };
    let mut texture_stats = None;
    let attach_spec: Option<treebuild::render::TextureSpec> = match tex_ids {
        Some((dir, ids)) => {
            let table = crate::materials::MaterialTable::load_cached(dir)
                .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
            let (mut spec, stats) = crate::textures::build_texture_spec_variants(
                &table,
                dir,
                ids,
                &assignments,
                &shore_assignments,
                &blend_assignments,
            )
            .map_err(CaptureError::Device)?;
            texture_stats = Some(stats);
            if !prep.colors.is_empty() {
                let final_base = spec.material_layers.len() as u32;
                crate::textures::extend_texture_spec_with_props(
                    &mut spec,
                    &prep.colors,
                    final_base,
                );
                shift_fine_ids(&mut fine_world, prep.base_id, final_base);
            }
            Some(spec)
        }
        // Untextured: `prep_prop_spec` already built the
        // prop-only spec at the stamp-time base id — no
        // shift needed.
        None => {
            if prep.colors.is_empty() {
                None
            } else {
                spec_opt
            }
        }
    };
    let built = build_tree(&coarse_world, BuildConfig::default())
        .map_err(|e| CaptureError::Device(format!("treebuild: {e}")))?;
    let fine_built = if fine_world.is_empty() {
        None
    } else {
        Some(
            build_tree(&fine_world, BuildConfig::default())
                .map_err(|e| CaptureError::Device(format!("treebuild fine: {e}")))?,
        )
    };
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let (renderer, mut scene) =
        RtRenderer::with_scene_fine(device, queue, &built, fine_built.as_ref());
    if let Some(spec) = &attach_spec {
        scene
            .attach_textures(device, queue, spec)
            .map_err(CaptureError::Device)?;
    }
    Ok(BuiltRtScene {
        renderer,
        scene,
        leaves: built.leaf_count(),
        nodes: built.node_count(),
        fine_leaves: fine_built.as_ref().map(|b| b.leaf_count()).unwrap_or(0),
        fine_nodes: fine_built.as_ref().map(|b| b.node_count()).unwrap_or(0),
        build_ms,
        texture_stats,
    })
}

/// Render one RT capture on an already-created device/queue.
pub fn capture_rt_with_device(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> Result<RtCapture, CaptureError> {
    capture_rt_with_device_sky(device, queue, args, None)
}

/// Phase 10 (10.4): [`capture_rt_with_device`] with explicit sky
/// uniforms (`None` = the renderer's neutral default). A given
/// block must already be in grid space — see
/// [`render_rt_frame_sky`] for the conversion contract.
pub fn capture_rt_with_device_sky(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: Option<&lighting::SkyUniforms>,
) -> Result<RtCapture, CaptureError> {
    capture_rt_with_device_sky_emitters(device, queue, args, sky, None)
}

/// Addendum 11 (emitter prototype): [`capture_rt_with_device_sky`]
/// with the frame's declared emitters as well (`None` = the
/// empty set). Blocks must already be in grid space — see
/// [`render_rt_frame_sky_emitters`].
pub fn capture_rt_with_device_sky_emitters(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    sky: Option<&lighting::SkyUniforms>,
    emitters: Option<&lighting::EmitterUniforms>,
) -> Result<RtCapture, CaptureError> {
    let frame = render_rt_frame_sky_emitters(device, queue, args, sky, emitters)?;
    let (width, height) = frame.size;

    let pixels = read_texture_rgba(device, queue, &frame.color, (width, height))?;
    let depth = read_buffer_f32(device, queue, &frame.depth, (width * height) as usize)?;
    let distinct: std::collections::HashSet<[u8; 4]> = pixels
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect();
    let image = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| CaptureError::Readback("pixel buffer size mismatch".into()))?;
    Ok(RtCapture {
        image,
        depth,
        max_dist: frame.max_dist,
        size: (width, height),
        leaves: frame.leaves,
        nodes: frame.nodes,
        build_ms: frame.build_ms,
        distinct_colors: distinct.len(),
        texture_stats: frame.texture_stats,
    })
}

/// Standalone RT capture: creates its own instance/adapter/device
/// (same clean `NoAdapter` failure as the raster capture).
pub fn capture_rt_standalone(args: &CaptureArgs) -> Result<RtCapture, CaptureError> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(CaptureError::NoAdapter)?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rt-capture-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| CaptureError::Device(e.to_string()))?;
        capture_rt_with_device(&device, &queue, args)
    })
}

fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Result<Vec<u8>, CaptureError> {
    let (width, height) = size;
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rt-capture-staging"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("rt-capture-copy"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| CaptureError::Readback("map channel closed".into()))?
        .map_err(|e| CaptureError::Readback(e.to_string()))?;
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..height as usize {
            let start = row * padded as usize;
            pixels.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    Ok(pixels)
}

fn read_buffer_f32(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    count: usize,
) -> Result<Vec<f32>, CaptureError> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rt-depth-staging"),
        size: (count * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("rt-depth-copy"),
    });
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (count * 4) as u64);
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| CaptureError::Readback("map channel closed".into()))?
        .map_err(|e| CaptureError::Readback(e.to_string()))?;
    let data = slice.get_mapped_range();
    let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    Ok(out)
}

// ---------------------------------------------------------------------
// Windowed blit: present an RT storage texture on a surface.
// ---------------------------------------------------------------------

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi & 1u) * 4 - 1);
    let y = f32(i32(vi & 2u) * 2 - 1);
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, input.uv);
    // The RT texture holds display-ready (sRGB-encoded) values; the
    // surface pipeline treats fragment output as linear and encodes
    // on write, so pre-apply the approximate inverse transfer.
    return vec4<f32>(pow(c.rgb, vec3<f32>(2.2, 2.2, 2.2)), 1.0);
}
"#;

/// Fullscreen-triangle blit of the RT color texture onto a surface.
pub struct RtBlit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl RtBlit {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rt-blit-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rt-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rt-blit-pipeline-layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rt-blit-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rt-blit-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &wgpu::Texture,
        dst_view: &wgpu::TextureView,
    ) {
        let src_view = src.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rt-blit-bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rt-blit-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rt-blit-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}
