//! Wire tree gate: TREE-flagged terrain in a live (wire-fed)
//! world seats its voxelizer Standing model — standing on
//! the tile's level TOP, since a wire world's solid tiles
//! fill their whole z-level — instead of rendering as the
//! coarse textured column that read as a flat stump in the
//! 2026-11-25 Launch-Sim screenshot.
//!
//! street_block carries three `t_tree` tiles at (13, 4),
//! (10, 13) and (15, 18). Proofs:
//! 1. The tree derivation (the live session's own step)
//!    resolves one placement per tree tile against the real
//!    model table; `from_wire` alone carries no trees.
//! 2. The level-top stamper seats the model at fine z 96
//!    (the top of street_block's single level) and replaces
//!    the coarse column with the scene's grass block; a tile
//!    whose top block is air (cutaway/mask) is skipped.
//! 3. Behavioural, through the pre-existing public prep API:
//!    meshing the wire world WITH its tree scene produces
//!    the tree geometry (far more quads than the same world
//!    meshed without it). Before the fix the two meshings
//!    had identical quad counts — the wire world never
//!    seated terrain models at all.
//! 4. Pixel level (GPU, loud-skip with no adapter or no
//!    atlas): an RT close-up on a tree shows the voxel
//!    canopy's colour variety, not the stump's handful.
//!
//! The model table is machine-local generated data
//! (`assets/*` is gitignored), so a machine without it
//! loud-skips, per the frame_gate convention.

use std::path::PathBuf;

use mesher::VoxelSampler as _;

use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::scene_prep::prepare_raster_wire;
use engine::wire::{prop_scene_for_wire, tree_scene_for_wire, WireScene};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}
fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../sim/scenes/{name}.toml"))
}
fn ultica_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1")
}
fn have_table() -> bool {
    ultica_dir().join("materials.json").exists()
}

/// street_block over the wire, with the furniture prop scene
/// derived exactly as the live session derives it (trees are
/// attached per-test, so the no-trees world stays available).
fn street_block_wire() -> (proto::TileDefsT, proto::SnapshotT, WireScene) {
    let reg = sim::DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = sim::Scene::load_file(&scene_path("street_block")).expect("scene loads");
    let mut world = sim::World::from_scene(&reg, &scene).expect("world builds");
    let snap = world.snapshot(&reg);
    let defs = reg.tiledefs().clone();
    let table = engine::materials::MaterialTable::load_cached(&ultica_dir()).expect("table loads");
    let wire = WireScene::from_wire(&defs, &snap, &engine::wire::palette_resolver(&defs))
        .with_props(prop_scene_for_wire(&defs, &snap, &table));
    (defs, snap, wire)
}

fn with_trees(
    defs: &proto::TileDefsT,
    snap: &proto::SnapshotT,
    wire: WireScene,
) -> WireScene {
    let table = engine::materials::MaterialTable::load_cached(&ultica_dir()).expect("table loads");
    wire.with_trees(tree_scene_for_wire(defs, snap, &table))
}

const TREES: [(i32, i32); 3] = [(13, 4), (10, 13), (15, 18)];

#[test]
fn tree_scene_derives_the_street_block_trees() {
    if !have_table() {
        eprintln!("SKIP wire tree gate: model table absent (assets/ultica_run1)");
        return;
    }
    let (defs, snap, wire) = street_block_wire();
    assert!(wire.trees.is_none(), "from_wire alone derives no trees");
    let table = engine::materials::MaterialTable::load_cached(&ultica_dir()).expect("table loads");
    let trees = tree_scene_for_wire(&defs, &snap, &table).expect("street_block has trees");
    assert_eq!(trees.placements.len(), TREES.len(), "one placement per tree tile");
    for (x, y) in TREES {
        assert!(
            trees.placements.iter().any(|p| p.x == x && p.y == y && p.level == 0),
            "a tree placement at ({x}, {y})"
        );
    }
    // The placed model is the real t_tree voxel model.
    let (_id, model) = &trees.models[0];
    assert_eq!(model.size, [32, 80, 32], "t_tree model dimensions");
}

#[test]
fn level_top_stamp_seats_the_tree_on_the_surface() {
    if !have_table() {
        eprintln!("SKIP wire tree gate: model table absent (assets/ultica_run1)");
        return;
    }
    let (defs, snap, wire) = street_block_wire();
    let wire = with_trees(&defs, &snap, wire);
    let trees = wire.trees.clone().expect("trees attached");
    let visible: Vec<i32> = (-10..=0).collect();
    let mut filtered = engine::world::filtered_map(&wire.map, wire.min_game_z, &visible);
    let scale = mesher::scale::WorldScale::default();
    let mut layer = mesher::fine::FineLayer::new(scale.blocks_per_z_level as i32 * mesher::fine::FINE);
    // Adapters over the tree scene with an empty colour set
    // (ids/layers are irrelevant to the seating assertions).
    let colors = engine::textures::PropColorSet::default();
    let block_fn = engine::snapshot_world::prop_block_for(&colors, 1000, 0);
    let adapters: Vec<(u32, engine::snapshot_world::PropModelCells<'_>)> = trees
        .models
        .iter()
        .map(|(id, m)| (*id, engine::snapshot_world::PropModelCells::new(m, &block_fn)))
        .collect();
    let resolve = |id: u32| -> Option<&dyn mesher::fine::ModelCells> {
        adapters
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, a)| a as &dyn mesher::fine::ModelCells)
    };
    let grass_id = wire
        .material_ids
        .as_ref()
        .expect("material ids")
        .iter()
        .find(|(_, sid)| sid.as_str() == "t_grass")
        .map(|(id, _)| *id)
        .expect("street_block presents t_grass");
    let grass_block = filtered.block_at(0, 0, 0);
    assert_eq!(grass_block.id, grass_id, "probe tile (0,0) is grass");
    let report = engine::snapshot_world::stamp_level_top_standing(
        &mut filtered,
        &mut layer,
        &trees.placements,
        scale,
        &resolve,
        Some(grass_block),
    )
    .expect("stamp runs");
    assert_eq!(report.skipped, 0, "no tree tile is skipped on the full map");
    for (x, y) in TREES {
        // The model volume sits at the level top: fine z 96
        // for street_block's single 3-block level.
        let vols = layer.volumes_for_tile((x, y));
        assert_eq!(vols.len(), 1, "one volume at tree tile ({x}, {y})");
        assert_eq!(vols[0].z0, 96, "the tree stands on the level top");
        // The coarse column became grass, top to bottom.
        for bz in 0..3 {
            assert_eq!(
                filtered.block_at(x, y, bz).id,
                grass_id,
                "tree column block bz{bz} is grass"
            );
        }
    }
    // A tile whose top block is air is skipped, not floated.
    let mut filtered2 = engine::world::filtered_map(&wire.map, wire.min_game_z, &visible);
    filtered2.set_global_block(13, 4, 2, mesher::block::Block::AIR);
    let mut layer2 = mesher::fine::FineLayer::new(scale.blocks_per_z_level as i32 * mesher::fine::FINE);
    let report2 = engine::snapshot_world::stamp_level_top_standing(
        &mut filtered2,
        &mut layer2,
        &trees.placements,
        scale,
        &resolve,
        Some(grass_block),
    )
    .expect("stamp runs");
    assert_eq!(report2.skipped, 1, "the aired-top tree is skipped");
    assert!(
        layer2.volumes_for_tile((13, 4)).is_empty(),
        "no volume floats over the aired tile"
    );
}

#[test]
fn textured_wire_mesh_gains_the_tree_geometry() {
    if !have_table() {
        eprintln!("SKIP wire tree gate: model table absent (assets/ultica_run1)");
        return;
    }
    let (defs, snap, wire) = street_block_wire();
    let visible: Vec<i32> = (-10..=0).collect();
    let (_m, stats_plain, _) =
        prepare_raster_wire(&visible, &wire, None, None, Some(&ultica_dir()))
            .expect("plain prep");
    let wire = with_trees(&defs, &snap, wire);
    let (_m, stats_trees, _) =
        prepare_raster_wire(&visible, &wire, None, None, Some(&ultica_dir()))
            .expect("tree prep");
    assert!(
        stats_trees.quads > stats_plain.quads + 500,
        "seating the tree scene adds the trees' fine geometry: \
         with trees {} quads vs without {} quads",
        stats_trees.quads,
        stats_plain.quads
    );
}

#[test]
fn rt_close_up_shows_the_voxel_canopy() {
    if !have_table() {
        eprintln!("SKIP wire tree gate: model table absent (assets/ultica_run1)");
        return;
    }
    let (defs, snap, wire) = street_block_wire();
    let wire = with_trees(&defs, &snap, wire);
    let out = std::env::temp_dir().join(format!("wire-tree-gate-{}.png", std::process::id()));
    let mut args = CaptureArgs::default();
    args.renderer = RendererKind::Rt;
    args.textures = Some(ultica_dir());
    args.wire = Some(wire);
    args.cutaway_z = 0;
    args.camera_target = (13.5, 4.5);
    args.yaw_deg = 0.0;
    args.pitch_deg = 35.0;
    args.distance = 10.0;
    args.size = (800, 600);
    args.out = out.clone();
    match run_capture(&args) {
        Ok(o) => {
            // The stump-only close-up of the pre-fix world
            // measured ~200 distinct colours; a voxel canopy
            // (per-cell palette colours under the tracer's
            // lighting) is far richer.
            assert!(
                o.distinct_colors > 350,
                "tree close-up shows the voxel canopy ({} distinct colours)",
                o.distinct_colors
            );
        }
        Err(e) => {
            eprintln!("SKIP wire tree pixel gate: capture unavailable: {e}");
        }
    }
}
