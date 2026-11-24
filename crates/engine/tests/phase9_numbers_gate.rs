//! Phase 9 slice 3j (plan §11b, 9.9): perf + memory numbers
//! for the fine-grid world, recorded in docs/phase9-fine.md.
//!
//! Measured here (all on this machine — lavapipe for frames):
//! - world-build: coarse world cells, coarse tree build time
//!   and node/leaf counts for the sample scene at cutaway 0;
//! - the fixture prop scene's fine layer: stamped tiles,
//!   `FineLayer::memory_bytes`, fine world cells, fine tree
//!   build time and node/leaf counts;
//! - a prop-dense lab stress scene (200 placements cycling
//!   the fixture models over the sample yard): the same fine
//!   numbers, scaled;
//! - frame times (median of 3 after a warmup) at 640x360 for
//!   raster and RT, fixture and stress scenes.
//!
//! The numbers are RECORDED, not gated tight: assertions are
//! sanity-only (things exist, stress > fixture). The RX 9070
//! XT frame-time verdict stays the standing deferred
//! measurement — raster remains the default renderer.

use std::time::Instant;

use engine::capture::{CaptureArgs, CaptureError, RendererKind};
use engine::frame::{self, PropScene};
use engine::snapshot_world::{FurniturePlacement, PropModelCells};
use engine::{textures, world};
use mesher::fine::{FineLayer, ModelCells, FINE};
use mesher::scale::WorldScale;
use treebuild::build::{build_tree, BuildConfig};
use treebuild::world::VoxelWorld;

fn median_ms(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Stamp `scene` over the sample scene's filtered map exactly
/// as the engine paths do (base id 1000, untextured colours),
/// returning the map and the fine layer.
fn stamp_sample(
    scene: &PropScene,
) -> (mesher::chunk::ChunkMap, FineLayer) {
    let scale = WorldScale::default();
    let map = world::build_sample_scene(scale);
    let mut filtered = world::filtered_map(&map, world::HOUSE_MIN_GAME_Z, &[0]);
    let colors = textures::PropColorSet::from_models(scene.models.iter().map(|(_, m)| m));
    let block_fn = engine::snapshot_world::prop_block_for(&colors, 1000, 0);
    let adapters: Vec<(u32, PropModelCells<'_>)> = scene
        .models
        .iter()
        .map(|(id, model)| (*id, PropModelCells::new(model, &block_fn)))
        .collect();
    let resolve = |id: u32| -> Option<&dyn ModelCells> {
        adapters
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, a)| a as &dyn ModelCells)
    };
    let mut layer = FineLayer::new(scale.blocks_per_z_level as i32 * FINE);
    engine::snapshot_world::stamp_furniture(
        &mut filtered,
        &mut layer,
        &scene.placements,
        scale,
        &resolve,
    )
    .expect("stamp fixture/stress scene");
    (filtered, layer)
}

struct TreeNums {
    cells: usize,
    nodes: usize,
    leaves: usize,
    build_ms: f64,
}

fn tree_nums(world: &VoxelWorld) -> TreeNums {
    let mut times = Vec::new();
    let mut built = None;
    for _ in 0..3 {
        let t = Instant::now();
        built = Some(build_tree(world, BuildConfig::default()).expect("build tree"));
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let built = built.unwrap();
    TreeNums {
        cells: world.len(),
        nodes: built.node_count(),
        leaves: built.leaf_count(),
        build_ms: median_ms(times),
    }
}

/// The prop-dense lab stress scene: the fixture models cycled
/// over a 10x10 grid of yard tiles (200 placements — two full
/// cycles of the grid with offset facings), all level 0.
fn stress_scene() -> PropScene {
    let fixture = PropScene::fixture("sample-scene").expect("fixture scene");
    let ids: Vec<u32> = fixture.models.iter().map(|(id, _)| *id).collect();
    let mut placements = Vec::new();
    let mut n = 0usize;
    for rep in 0..2 {
        for gy in 0..10 {
            for gx in 0..10 {
                placements.push(FurniturePlacement {
                    x: 2 + gx * 2,
                    y: 2 + gy * 2,
                    level: 0,
                    furn_id: ids[n % ids.len()],
                    quarter_turns: ((n + rep) % 4) as u8,
                });
                n += 1;
            }
        }
    }
    PropScene {
        models: fixture.models,
        placements,
    }
}

#[test]
fn phase9_numbers_world_and_trees() {
    let levels = [0];

    // Coarse-only baseline (the pre-Phase-9 world).
    let coarse_world = engine::rt::world_for_levels_sv(&levels, None, true);
    let coarse = tree_nums(&coarse_world);

    // Fixture props.
    let fixture = PropScene::fixture("sample-scene").expect("fixture scene");
    let (filtered, layer) = stamp_sample(&fixture);
    let fine_world = VoxelWorld::from_fine_layer(&layer);
    let fine = tree_nums(&fine_world);
    let stamped_coarse = tree_nums(&VoxelWorld::from_chunk_map(&filtered));

    // Stress scene.
    let stress = stress_scene();
    let (_sfiltered, slayer) = stamp_sample(&stress);
    let sfine_world = VoxelWorld::from_fine_layer(&slayer);
    let sfine = tree_nums(&sfine_world);

    println!("=== Phase 9 numbers (9.9) — sample scene, cutaway 0 ===");
    println!(
        "coarse world: {} cells; tree {} nodes / {} leaves; build {:.2} ms (median of 3)",
        coarse.cells, coarse.nodes, coarse.leaves, coarse.build_ms
    );
    println!(
        "fixture props: {} placements on {} tiles; fine layer {} bytes; \
         fine world {} cells; fine tree {} nodes / {} leaves; build {:.2} ms",
        fixture.placements.len(),
        layer.tile_count(),
        layer.memory_bytes(),
        fine.cells,
        fine.nodes,
        fine.leaves,
        fine.build_ms
    );
    println!(
        "stamped coarse world: {} cells; tree {} nodes / {} leaves; build {:.2} ms \
         (furniture blocks suppressed vs {} cells unstamped)",
        stamped_coarse.cells, stamped_coarse.nodes, stamped_coarse.leaves, stamped_coarse.build_ms,
        coarse.cells
    );
    println!(
        "stress props: {} placements on {} tiles; fine layer {} bytes; \
         fine world {} cells; fine tree {} nodes / {} leaves; build {:.2} ms",
        stress.placements.len(),
        slayer.tile_count(),
        slayer.memory_bytes(),
        sfine.cells,
        sfine.nodes,
        sfine.leaves,
        sfine.build_ms
    );

    assert!(fine.cells > 0 && fine.nodes > 0, "fixture fine tree exists");
    assert!(
        sfine.cells > fine.cells && sfine.nodes > fine.nodes,
        "stress scene must exceed the fixture scene"
    );
    // Suppression: on the sample scene the fixture tiles carry
    // no coarse furniture blocks (the placements stand on bare
    // floor), so the counts are EQUAL here — suppression itself
    // is gate-proven where a furniture block exists
    // (phase9_stamp_gate). The invariant that must hold on
    // every scene is: stamping never ADDS coarse cells.
    assert!(
        stamped_coarse.cells <= coarse.cells,
        "stamping must never add coarse cells ({} vs {})",
        stamped_coarse.cells,
        coarse.cells
    );
}

// ---------------------------------------------------------------------
// Frame times (adapter-conditional, loud skip)
// ---------------------------------------------------------------------

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn gpu() -> Option<Gpu> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("phase9-numbers-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu { device, queue })
}

fn frame_args(renderer: RendererKind, props: Option<PropScene>) -> CaptureArgs {
    CaptureArgs {
        snapshot: "sample-scene".to_string(),
        renderer,
        camera_target: (6.5, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 50.0,
        distance: 15.0,
        size: (640, 360),
        cutaway_z: 0,
        fine_props: props.is_some(),
        props,
        ..CaptureArgs::default()
    }
}

fn median_frame_ms(gpu: &Gpu, args: &CaptureArgs) -> Result<f64, CaptureError> {
    // Warmup (device-side caches, shader compiles), then the
    // median of 3 timed frames.
    let _ = frame::render_frame(&gpu.device, &gpu.queue, args)?;
    let mut times = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        let _ = frame::render_frame(&gpu.device, &gpu.queue, args)?;
        gpu.device.poll(wgpu::Maintain::Wait);
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    Ok(median_ms(times))
}

#[test]
fn phase9_numbers_frame_times_lavapipe() {
    let Some(gpu) = gpu() else {
        eprintln!(
            "================================================================\n\
             PHASE 9 NUMBERS: frame times NOT RUN — no wgpu adapter.\n\
             The world/tree/memory numbers above still count; frame\n\
             times only count on a machine with an adapter (lavapipe counts).\n\
             ================================================================"
        );
        return;
    };
    let fixture = PropScene::fixture("sample-scene").expect("fixture scene");
    let stress = stress_scene();
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let off = median_frame_ms(&gpu, &frame_args(renderer, None)).expect("off frame");
        let fix = median_frame_ms(&gpu, &frame_args(renderer, Some(fixture.clone()))).expect("fixture frame");
        let stress_ms = median_frame_ms(&gpu, &frame_args(renderer, Some(stress.clone()))).expect("stress frame");
        println!(
            "frame times 640x360 lavapipe {renderer:?}: coarse-only {off:.2} ms, \
             fixture props {fix:.2} ms, stress (200 props) {stress_ms:.2} ms (median of 3)"
        );
        assert!(off > 0.0 && fix > 0.0 && stress_ms > 0.0);
    }
}
