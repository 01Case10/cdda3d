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
    let table = crate::materials::MaterialTable::load(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let id_map = fake_material_ids();
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
    let table = crate::materials::MaterialTable::load(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let id_map = fake_material_ids();
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
    let table = crate::materials::MaterialTable::load(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let id_map = fake_material_ids();
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
) -> Result<crate::textures::TextureStats, CaptureError> {
    let table = crate::materials::MaterialTable::load(dir)
        .map_err(|e| CaptureError::Device(format!("materials: {e}")))?;
    let (spec, stats) = crate::textures::build_texture_spec_variants(
        &table,
        dir,
        &fake_material_ids(),
        assignments,
        shore_assignments,
        blend_assignments,
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
    let scale = mesher::scale::WorldScale::default();
    let map = if sample {
        world::build_sample_scene(scale)
    } else {
        world::build_fake_house(scale)
    };
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, visible_levels);
    if let Some(v) = vehicles {
        crate::scene_prep::stamp_vehicles(&mut filtered, v);
    }
    VoxelWorld::from_chunk_map(&filtered)
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
    let base = world_for_levels_sv(visible_levels, vehicles, sample);
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

/// Render one RT frame on an already-created device/queue,
/// returning the GPU frame (color texture + linear depth
/// buffer). [`capture_rt_with_device`] is this plus readback.
pub fn render_rt_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> Result<RtFrameOut, CaptureError> {
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
    if let Some(props) = args.props.as_ref().or(fixture_props.as_ref()) {
        return render_rt_frame_props(device, queue, args, props);
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
    let mut voxel_world = match &args.lighting {
        Some(lit) => world_for_levels_lit_sv(&levels, lit, args.vehicles.as_ref(), sample),
        None => world_for_levels_sv(&levels, args.vehicles.as_ref(), sample),
    };
    let assignments = match &args.textures {
        Some(dir) => prepare_fake_variants(&mut voxel_world, dir)?,
        None => Vec::new(),
    };
    // Stage F part 2: shore preparation after the multitile
    // preparation (its virtual ids allocate above theirs); with
    // no textures the world is untouched, byte-for-byte the
    // pre-F2 render.
    let shore_assignments = match &args.textures {
        Some(dir) => prepare_fake_shore_variants(&mut voxel_world, dir)?,
        None => Vec::new(),
    };
    // Stage F part 2 (blend): after the shore preparation (its
    // virtual ids allocate above theirs); with no textures the
    // world is untouched, byte-for-byte the pre-blend render.
    let blend_assignments = match &args.textures {
        Some(dir) => prepare_fake_blend_variants(&mut voxel_world, dir)?,
        None => Vec::new(),
    };
    let built = build_tree(&voxel_world, BuildConfig::default())
        .map_err(|e| CaptureError::Device(format!("treebuild: {e}")))?;
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let (renderer, mut scene) = RtRenderer::with_scene(device, queue, &built);
    let texture_stats = match &args.textures {
        Some(dir) => Some(attach_fake_textures(
            device,
            queue,
            &mut scene,
            dir,
            &assignments,
            &shore_assignments,
            &blend_assignments,
        )?),
        None => None,
    };
    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
    let aspect = width as f32 / height as f32;
    let camera = capture_camera(args, aspect);
    let frame = if args.macro_debug_height {
        renderer.render_macro_debug_height(device, queue, &scene, &camera, (width, height))
    } else {
        renderer.render(device, queue, &scene, &camera, (width, height), true)
    };
    device.poll(wgpu::Maintain::Wait);
    Ok(RtFrameOut {
        color: frame.color,
        depth: frame.depth,
        max_dist: frame.max_dist,
        camera,
        size: (width, height),
        leaves: built.leaf_count(),
        nodes: built.node_count(),
        fine_leaves: 0,
        fine_nodes: 0,
        build_ms,
        texture_stats,
    })
}

/// The Phase 7 visibility contract as a post-pass over an
/// already-built world (the body of [`world_for_levels_lit_sv`],
/// reusable over the fine world): `Hidden` tiles' cells are
/// dropped, `Memorized` cells take the pinned presentation,
/// `Visible` cells take their tile's `LitLevel`. With `fine`,
/// the world is in fine coordinates (`mesher::fine::FINE` cells
/// per block): a fine cell takes its CONTAINING tile's light —
/// the 9.6 v1 rule (docs/phase9-fine.md §1).
fn lit_rewrite(world: &VoxelWorld, lit: &crate::frame::LightingScene, fine: bool) -> VoxelWorld {
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
fn render_rt_frame_props(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    props: &crate::frame::PropScene,
) -> Result<RtFrameOut, CaptureError> {
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
    if let Some(v) = &args.vehicles {
        crate::scene_prep::stamp_vehicles(&mut filtered, v);
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
            let table = crate::materials::MaterialTable::load(dir)
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
        coarse_world = lit_rewrite(&coarse_world, lit, false);
        fine_world = lit_rewrite(&fine_world, lit, true);
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
            let table = crate::materials::MaterialTable::load(dir)
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
    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
    let aspect = width as f32 / height as f32;
    let camera = capture_camera(args, aspect);
    let frame = if args.macro_debug_height {
        renderer.render_macro_debug_height(device, queue, &scene, &camera, (width, height))
    } else {
        renderer.render(device, queue, &scene, &camera, (width, height), true)
    };
    device.poll(wgpu::Maintain::Wait);
    Ok(RtFrameOut {
        color: frame.color,
        depth: frame.depth,
        max_dist: frame.max_dist,
        camera,
        size: (width, height),
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
    let frame = render_rt_frame(device, queue, args)?;
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
