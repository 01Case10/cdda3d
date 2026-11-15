//! Phase 2c gate tests (plan Part II §5c). GPU tests are
//! adapter-conditional in the established pattern (2.10/2b.5): with
//! an adapter they run for real; without one they verify the clean
//! named failure path is what *would* happen by simply not running —
//! here there is no fallible constructor, so they return early after
//! printing that pixel/GPU checks were not run.

use std::collections::HashSet;
use std::time::Instant;

use glam::Vec3;
use proptest::prelude::*;
use treebuild::build::{build_tree, reconstruct_side_table, verify_side_table, BuildConfig, SideEntry};
use treebuild::cpu::{cast_ray, dda_reference, occupied_at, point_node, TraceMode};
use treebuild::world::{
    VoxelCell, VoxelWorld, FLAG_EMISSIVE, FLAG_MEMORIZED, FLAG_TRANSPARENT, LIT_BLANK, LIT_BRIGHT,
    LIT_LIT, LIT_MEMORIZED,
};

// ---------------------------------------------------------------------
// Test worlds
// ---------------------------------------------------------------------

/// A house-like structure on a 12x12 footprint built directly as
/// voxels: ground slab, wall perimeter with a door gap and a glass
/// window, roof slab. Coordinates in blocks (z up).
fn house_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    let ground = VoxelCell::new([0.35, 0.50, 0.25], 6);
    let floor = VoxelCell::new([0.55, 0.38, 0.22], 1);
    let wall = VoxelCell::new([0.70, 0.68, 0.62], 2);
    let roof = VoxelCell::new([0.50, 0.20, 0.15], 5);
    let mut glass = VoxelCell::new([0.60, 0.80, 0.90], 4);
    glass.flags |= FLAG_TRANSPARENT;
    glass.alpha = 0.35;
    for y in 0..12 {
        for x in 0..12 {
            w.set([x, y, 0], ground);
            if (1..11).contains(&x) && (1..11).contains(&y) {
                w.set([x, y, 1], floor);
            }
            w.set([x, y, 5], roof);
        }
    }
    for i in 0..12 {
        for z in 1..4 {
            w.set([i, 0, z], wall);
            w.set([i, 11, z], wall);
            w.set([0, i, z], wall);
            w.set([11, i, z], wall);
        }
    }
    // Door gap (south wall) and window (north wall).
    for z in 1..4 {
        w.remove([6, 11, z]);
    }
    for z in 2..4 {
        w.set([3, 0, z], glass);
    }
    w
}

/// Unique color per coord (for ranges x,y < 16, z < 8): the side-table
/// ground-truth check ties a flat leaf to exactly one voxel.
fn unique_cell(x: i32, y: i32, z: i32) -> VoxelCell {
    VoxelCell::new(
        [
            (x * 16) as f32 / 255.0,
            (y * 16) as f32 / 255.0,
            (z * 32) as f32 / 255.0,
        ],
        (x * 100 + y * 10 + z) as u32,
    )
}

fn random_world(cells: &[(i32, i32, i32)]) -> VoxelWorld {
    let mut w = VoxelWorld::new();
    let mut seen = HashSet::new();
    for &(x, y, z) in cells {
        let p = (x.rem_euclid(12), y.rem_euclid(12), z.rem_euclid(8));
        if seen.insert(p) {
            w.set([p.0, p.1, p.2], unique_cell(p.0, p.1, p.2));
        }
    }
    w
}

/// The stored bubble footprint: 132x132 tiles x 3 z-levels, built the
/// same way the engine builds its bubble scene (ground everywhere,
/// house outline per 12x12 chunk), at block resolution via mesher.
fn bubble_world() -> VoxelWorld {
    use mesher::block::*;
    use mesher::chunk::ChunkMap;
    use mesher::scale::WorldScale;
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    for cy in 0..11 {
        for cx in 0..11 {
            let chunk = map.add_chunk([cx, cy], 3);
            for y in 0..12 {
                for x in 0..12 {
                    chunk.set_tile(x, y, 0, TILE_GROUND, scale);
                }
            }
            for i in 2..10 {
                chunk.set_tile(i, 2, 1, TILE_WALL, scale);
                chunk.set_tile(i, 9, 1, TILE_WALL, scale);
                chunk.set_tile(2, i, 1, TILE_WALL, scale);
                chunk.set_tile(9, i, 1, TILE_WALL, scale);
            }
            chunk.set_tile(5, 9, 1, TILE_DOOR, scale);
            chunk.set_tile(6, 2, 1, TILE_WINDOW, scale);
        }
    }
    VoxelWorld::from_chunk_map(&map)
}

// ---------------------------------------------------------------- 2c.1
#[test]
fn gate_2c_1_tree_build_correctness() {
    for world in [house_world(), bubble_world()] {
        let built = build_tree(&world, BuildConfig::default()).unwrap();
        // Every voxel in the world is a leaf; leaf count matches.
        assert_eq!(built.leaf_count(), built.voxel_count);
        assert_eq!(built.voxel_count, world.len());
        // Node count within a sane bound of the occupied count: at
        // least the leaves, at most leaves * depth + 1 (no sharing).
        assert!(built.node_count() >= built.voxel_count);
        assert!(built.node_count() <= built.voxel_count * built.depth + 1);
        // Exhaustive point queries over the bounding box: occupied
        // exactly where the world is occupied.
        let mut max = [0i32; 3];
        for (p, _) in world.iter() {
            for a in 0..3 {
                max[a] = max[a].max(p[a]);
            }
        }
        for z in 0..=max[2] + 1 {
            for y in 0..=max[1] + 1 {
                for x in 0..=max[0] + 1 {
                    let p = [x, y, z];
                    assert_eq!(
                        occupied_at(&built, p),
                        world.get(p).is_some(),
                        "occupancy mismatch at {p:?}"
                    );
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn gate_2c_1_random_worlds_point_queries(cells in prop::collection::vec((0i32..12, 0i32..12, 0i32..8), 0..300)) {
        let world = random_world(&cells);
        let built = build_tree(&world, BuildConfig::default()).unwrap();
        prop_assert_eq!(built.leaf_count(), world.len());
        for z in 0..9 {
            for y in 0..13 {
                for x in 0..13 {
                    let p = [x, y, z];
                    prop_assert_eq!(occupied_at(&built, p), world.get(p).is_some());
                }
            }
        }
    }

    /// Tree traversal vs the DDA reference over the same world:
    /// identical hit voxel + normal, distance within tolerance, for
    /// a fan of rays per random world (the ray half of 2c.1 and the
    /// CPU half of 2c.3/2c.7).
    #[test]
    fn gate_2c_1_rays_match_dda(cells in prop::collection::vec((0i32..12, 0i32..12, 0i32..8), 1..300)) {
        let world = random_world(&cells);
        let built = build_tree(&world, BuildConfig::default()).unwrap();
        let origin = built.origin;
        let dirs = [
            Vec3::new(1.0, 0.13, 0.21),
            Vec3::new(-0.71, 1.0, -0.35),
            Vec3::new(0.27, -1.0, 0.53),
            Vec3::new(-1.0, -0.41, -0.17),
            Vec3::new(0.11, 0.37, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        for oy in [0.5f32, 3.5, 7.5, 11.5] {
            for ox in [0.5f32, 5.5, 11.5] {
                let ro = Vec3::new(ox - 6.0, oy - 6.0, 10.5);
                for d in dirs {
                    let rd = d.normalize();
                    for mode in [TraceMode::FirstHit, TraceMode::OpaqueOnly] {
                        let a = cast_ray(&built, ro, rd, 500.0, mode);
                        let b = dda_reference(&world, origin, ro, rd, 500.0, mode);
                        match (a, b) {
                            (None, None) => {}
                            (Some(x), Some(y)) => {
                                prop_assert_eq!(x.voxel, y.voxel, "voxel mismatch ro={:?} rd={:?} mode={:?}", ro, rd, mode);
                                prop_assert_eq!(x.normal, y.normal, "normal mismatch ro={:?} rd={:?} mode={:?}", ro, rd, mode);
                                prop_assert!((x.dist - y.dist).abs() < 1e-3, "dist {} vs {}", x.dist, y.dist);
                            }
                            _ => prop_assert!(false, "hit/miss mismatch ro={:?} rd={:?} mode={:?}: tree={:?} dda={:?}", ro, rd, mode, a, b),
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------- 2c.2
#[test]
fn gate_2c_2_side_table_indexing() {
    let world = house_world();
    let built = build_tree(&world, BuildConfig::default()).unwrap();
    verify_side_table(&built, &world).unwrap();
    // Every leaf's side entry is exactly its voxel's data.
    let mut leaves = 0;
    for idx in 0..built.node_count() {
        if let Some(voxel) = built.leaf_voxel(idx) {
            leaves += 1;
            let cell = world.get(voxel).unwrap();
            let e = built.side[idx];
            assert_eq!(e.material_id, cell.material_id, "leaf {idx} at {voxel:?}");
            assert_eq!(e.lit, cell.lit as u32);
            assert_eq!(e.flags, cell.flags);
        } else {
            assert_eq!(built.side[idx], SideEntry::EMPTY, "internal node {idx} must carry no material");
        }
    }
    assert_eq!(leaves, world.len());

    // Negative case: a shuffled side-table must fail verification.
    let mut tampered = build_tree(&world, BuildConfig::default()).unwrap();
    let leaf_indices: Vec<usize> = (0..tampered.node_count())
        .filter(|&i| tampered.is_leaf(i))
        .collect();
    assert!(leaf_indices.len() > 2);
    tampered.side.swap(leaf_indices[0], leaf_indices[leaf_indices.len() - 1]);
    assert!(
        verify_side_table(&tampered, &world).is_err(),
        "shuffled side-table must be detected"
    );
    // And reconstruction against a *different* world must fail too.
    let (side2, _) = reconstruct_side_table(&built.flat, &world, built.origin, built.size).unwrap();
    assert_eq!(side2, built.side);
    let mut other = house_world();
    other.set([1, 1, 1], VoxelCell::new([0.01, 0.02, 0.03], 999));
    assert!(reconstruct_side_table(&built.flat, &other, built.origin, built.size).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    #[test]
    fn gate_2c_2_random_worlds(cells in prop::collection::vec((0i32..12, 0i32..12, 0i32..8), 0..300)) {
        let world = random_world(&cells);
        let built = build_tree(&world, BuildConfig::default()).unwrap();
        verify_side_table(&built, &world).unwrap();
        for idx in 0..built.node_count() {
            if let Some(voxel) = built.leaf_voxel(idx) {
                let cell = world.get(voxel).unwrap();
                prop_assert_eq!(built.side[idx].material_id, cell.material_id);
            }
        }
    }
}

// ---------------------------------------------------------------- 2c.5
#[test]
fn gate_2c_5_rebuild_cost() {
    let world = bubble_world();
    let t0 = Instant::now();
    let built = build_tree(&world, BuildConfig::default()).unwrap();
    let full_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "2c.5 full stored-world build: {:.2} ms ({} voxels, {} nodes, depth {}) — budget 20 ms on target hardware; this VM is 1.5 shared CPUs, debug profile",
        full_ms,
        built.voxel_count,
        built.node_count(),
        built.depth
    );
    // Per-turn delta path, single-tree design: apply a 1-tile change
    // and rebuild everything.
    let mut changed = world.clone();
    changed.set([60, 60, 3], VoxelCell::new([0.7, 0.68, 0.62], 2));
    let t1 = Instant::now();
    let _rebuilt = build_tree(&changed, BuildConfig::default()).unwrap();
    let delta_single_ms = t1.elapsed().as_secs_f64() * 1000.0;
    println!("2c.5 delta via full rebuild (single tree): {delta_single_ms:.2} ms — budget 2 ms");
    // Per-column design: rebuild only the dirty 12x12 column.
    let column = changed.subset_box([60, 60, 0], [72, 72, 9]);
    let t2 = Instant::now();
    let col_built = build_tree(&column, BuildConfig::default()).unwrap();
    let delta_column_ms = t2.elapsed().as_secs_f64() * 1000.0;
    println!(
        "2c.5 delta via dirty-column rebuild: {delta_column_ms:.3} ms ({} voxels, {} nodes)",
        col_built.voxel_count,
        col_built.node_count()
    );
    // Sanity bounds only (real budgets are target-hardware numbers,
    // recorded in docs/phase2c-rt.md): nothing here may be pathological.
    assert!(full_ms < 30_000.0, "full build took {full_ms} ms");
    assert!(delta_column_ms < full_ms, "column rebuild must beat full rebuild");
    assert!(col_built.voxel_count > 0);
}

// ---------------------------------------------------------------- 2c.7
#[test]
fn gate_2c_7_picking_parity() {
    // Calibration scene: the house. Picking rays from a camera-like
    // orbit; the tree path and the grid DDA must return the same
    // *tile* (block / (1, 1, 3)) — the mesher-era result.
    let world = house_world();
    let built = build_tree(&world, BuildConfig::default()).unwrap();
    let origin = built.origin;
    let target = Vec3::new(6.0, 6.0, 2.0);
    let mut hits = 0;
    for i in 0..24 {
        let ang = (i as f32) * 15.0f32.to_radians();
        let eye = target + Vec3::new(ang.cos() * 20.0, ang.sin() * 20.0, 14.0);
        let rd = (target - eye).normalize();
        let ro = eye - origin_vec(origin);
        let a = cast_ray(&built, ro, rd, 500.0, TraceMode::FirstHit);
        let b = dda_reference(&world, origin, ro, rd, 500.0, TraceMode::FirstHit);
        match (a, b) {
            (Some(x), Some(y)) => {
                hits += 1;
                let tile = |v: [i32; 3]| [v[0], v[1], v[2].div_euclid(3)];
                assert_eq!(tile(x.voxel), tile(y.voxel), "pick mismatch at ray {i}");
                assert_eq!(x.voxel, y.voxel);
            }
            (None, None) => {}
            _ => panic!("pick hit/miss mismatch at ray {i}"),
        }
    }
    assert!(hits >= 12, "calibration rays should mostly hit the house, got {hits}");
}

fn origin_vec(o: [i32; 3]) -> Vec3 {
    Vec3::new(o[0] as f32, o[1] as f32, o[2] as f32)
}

// ---------------------------------------------------------------- 2c.8
#[test]
fn gate_2c_8_visibility_discipline() {
    // A lit floor, a never-seen (BLANK) wall and a memorized wall
    // between the floor and the sun side (+x).
    let mut world = VoxelWorld::new();
    let floor = VoxelCell::new([0.5, 0.5, 0.5], 10);
    world.set([5, 5, 0], floor);
    let mut blank = VoxelCell::new([0.9, 0.1, 0.1], 11);
    blank.lit = LIT_BLANK;
    world.set([6, 5, 0], blank);
    let mut memo = VoxelCell::new([0.1, 0.9, 0.1], 12);
    memo.lit = LIT_MEMORIZED;
    world.set([7, 5, 0], memo);
    let mut bright = VoxelCell::new([0.2, 0.2, 0.9], 13);
    bright.lit = LIT_BRIGHT;
    world.set([8, 5, 0], bright);

    let built = build_tree(&world, BuildConfig::default()).unwrap();
    assert_eq!(built.excluded_never_seen, 1);
    assert_eq!(built.voxel_count, 3, "BLANK cell must not be inserted");
    assert!(!occupied_at(&built, [6, 5, 0]), "never-seen voxel must be absent from the tree");
    assert!(occupied_at(&built, [7, 5, 0]), "memorized voxel stays in the tree");
    // Memorized leaf carries the flag + its lit value.
    let memo_node = point_node(&built, [7, 5, 0]).unwrap();
    assert!(built.side[memo_node].flags & FLAG_MEMORIZED != 0);
    assert_eq!(built.side[memo_node].lit, LIT_MEMORIZED as u32);
    // Shadow discipline: a ray from the floor's +x face toward +x
    // (the sun side) passes through the BLANK wall's cell as if it
    // were air — unseen geometry cannot cast an identifiable
    // shadow — and is stopped by the memorized wall, which *is* in
    // the tree.
    let ro = Vec3::new(6.0, 5.5, 0.5) - origin_vec(built.origin);
    let hit = cast_ray(&built, ro, Vec3::X, 100.0, TraceMode::OpaqueOnly)
        .expect("memorized wall must block the shadow ray");
    assert_eq!(hit.voxel, [7, 5, 0]);
    // Emissive flag survives into the side-table.
    let mut w2 = VoxelWorld::new();
    let mut em = VoxelCell::new([1.0, 0.5, 0.1], 20);
    em.flags |= FLAG_EMISSIVE;
    w2.set([0, 0, 0], em);
    let b2 = build_tree(&w2, BuildConfig::default()).unwrap();
    let n = point_node(&b2, [0, 0, 0]).unwrap();
    assert!(b2.side[n].flags & FLAG_EMISSIVE != 0);
    assert_eq!(b2.side[n].lit, LIT_LIT as u32);
}

// ---------------------------------------------------------------------
// GPU gates (adapter-conditional): 2c.3, 2c.10 + an R0 render smoke.
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
            label: Some("treebuild-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu { device, queue })
}

#[test]
fn gate_2c_3_and_2c_10_gpu_traversal_parity_and_crate_smoke() {
    let Some(gpu) = gpu() else {
        eprintln!("2c.3/2c.10: no wgpu adapter — GPU checks NOT RUN (CPU halves pass above)");
        return;
    };
    use treebuild::render::{RtCamera, RtRenderer};
    let world = house_world();
    let built = build_tree(&world, BuildConfig::default()).unwrap();
    let (renderer, scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    // 2c.10 smoke: the crate manager's buffer is exactly the flat
    // tree's bytes, and its bind group is the one the pipelines use.
    assert_eq!(scene.node_count, built.node_count());
    assert_eq!(
        scene.manager.get_buffer().size(),
        (built.node_count() * 32) as u64,
        "Tree64GpuManager buffer must hold exactly the flattened tree"
    );

    // Fixed ray set: origins on a ring + above, directions at the
    // house and along axes; both trace modes.
    let origin = origin_vec(built.origin);
    let mut rays: Vec<(Vec3, Vec3)> = Vec::new();
    let target = Vec3::new(6.0, 6.0, 2.0);
    for i in 0..16 {
        let ang = (i as f32) * 22.5f32.to_radians();
        let eye = target + Vec3::new(ang.cos() * 18.0, ang.sin() * 18.0, 12.0);
        rays.push((eye - origin, (target - eye).normalize()));
    }
    for d in [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::NEG_Z,
        Vec3::new(1.0, 1.0, -0.5).normalize(),
    ] {
        rays.push((Vec3::new(-4.5, 5.5, 2.5) - origin + Vec3::ZERO, d));
        rays.push((Vec3::new(16.5, 5.5, 2.5), d));
    }
    for mode in [TraceMode::FirstHit, TraceMode::OpaqueOnly] {
        let gpu_hits = renderer.trace_rays(&gpu.device, &gpu.queue, &scene, &rays, mode, 500.0);
        assert_eq!(gpu_hits.len(), rays.len());
        for (i, ((ro, rd), g)) in rays.iter().zip(gpu_hits.iter()).enumerate() {
            let cpu = cast_ray(&built, *ro, *rd, 500.0, mode);
            match cpu {
                None => assert_eq!(g.hit, 0, "ray {i} mode {mode:?}: GPU hit, CPU miss"),
                Some(h) => {
                    assert_eq!(g.hit, 1, "ray {i} mode {mode:?}: CPU hit {:?}, GPU miss", h.voxel);
                    let gpu_voxel = [
                        g.voxel[0] + built.origin[0],
                        g.voxel[1] + built.origin[1],
                        g.voxel[2] + built.origin[2],
                    ];
                    assert_eq!(gpu_voxel, h.voxel, "ray {i} mode {mode:?}: voxel mismatch");
                    assert_eq!(g.normal, h.normal, "ray {i} mode {mode:?}: normal mismatch");
                    assert!(
                        (g.dist - h.dist).abs() < 1e-3,
                        "ray {i} mode {mode:?}: dist {} (GPU) vs {} (CPU)",
                        g.dist,
                        h.dist
                    );
                    assert_eq!(g.node, h.node_index, "ray {i} mode {mode:?}: node mismatch");
                }
            }
        }
    }

    // R0/render smoke: the house renders non-blank through the tree.
    let camera = RtCamera {
        eye: Vec3::new(6.0, -14.0, 16.0),
        target: Vec3::new(6.0, 6.0, 1.0),
        fov_y_deg: 60.0,
        aspect: 16.0 / 9.0,
    };
    let frame = renderer.render(&gpu.device, &gpu.queue, &scene, &camera, (320, 180), true);
    let (w, h) = frame.size;
    let depth = read_f32_buffer(&gpu, &frame.depth, (w * h) as usize);
    let hit_pixels = depth.iter().filter(|&&d| d < frame.max_dist).count();
    assert!(
        hit_pixels > (w * h) as usize / 10,
        "RT render should cover a good part of the frame with house hits, got {hit_pixels}/{}",
        w * h
    );
}

fn read_f32_buffer(gpu: &Gpu, buffer: &wgpu::Buffer, count: usize) -> Vec<f32> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gate-readback"),
        size: (count * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (count * 4) as u64);
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let data = slice.get_mapped_range();
    let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    out
}

// ---------------------------------------------------------------------
// Texture hookup (Phase 5 follow-up, closed on the RT path 2026-11-15)
// ---------------------------------------------------------------------

fn read_rgba_texture(gpu: &Gpu, texture: &wgpu::Texture, size: (u32, u32)) -> Vec<u8> {
    let (w, h) = size;
    let unpadded = w * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gate-tex-readback"),
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
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
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..h as usize {
            let start = row * padded as usize;
            out.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    out
}

/// Textured materials sample the texture array by material + face
/// class; untextured materials keep flat shading. Two grey voxels,
/// one mapped to a black/white checkerboard layer: the textured
/// render must contain both near-black and near-white hit pixels,
/// the flat control render neither.
#[test]
fn gate_texture_hookup_samples_array_and_falls_back_flat() {
    let Some(gpu) = gpu() else {
        eprintln!("texture hookup: no wgpu adapter — GPU checks NOT RUN");
        return;
    };
    use treebuild::render::{RtCamera, RtRenderer, TextureSpec};

    let mut world = VoxelWorld::new();
    // Both voxels flat mid-grey; only material 7 gets a texture.
    world.set([4, 4, 4], VoxelCell::new([0.5, 0.5, 0.5], 7));
    world.set([6, 4, 4], VoxelCell::new([0.5, 0.5, 0.5], 9));
    let built = build_tree(&world, BuildConfig::default()).unwrap();
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);

    // 8x8 checkerboard layer: 4px checks, pure black / pure white.
    let tile = 8u32;
    let mut layer = Vec::with_capacity((tile * tile * 4) as usize);
    for y in 0..tile {
        for x in 0..tile {
            let white = ((x / 4) + (y / 4)) % 2 == 0;
            let v = if white { 255u8 } else { 0u8 };
            layer.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let mut material_layers = vec![[0u32, 0, 0, 0]; 10];
    material_layers[7] = [0, 0, 0, 1];
    scene
        .attach_textures(
            &gpu.device,
            &gpu.queue,
            &TextureSpec {
                tile_size: tile,
                layers: vec![layer],
                material_layers,
            },
        )
        .unwrap();
    assert_eq!(scene.textures.layer_count, 1);

    // Tilted camera (a perfectly vertical look direction is
    // degenerate for the +z-up basis — same reason the engine's
    // OrbitCamera clamps pitch below 90 degrees).
    let camera = RtCamera {
        eye: Vec3::new(5.0, 0.0, 11.0),
        target: Vec3::new(5.0, 5.0, 4.0),
        fov_y_deg: 60.0,
        aspect: 1.0,
    };
    let hit_luminance = |scene: &treebuild::render::RtScene| -> Vec<u8> {
        let frame = renderer.render(&gpu.device, &gpu.queue, scene, &camera, (128, 128), false);
        let pixels = read_rgba_texture(&gpu, &frame.color, frame.size);
        let depth = read_f32_buffer(&gpu, &frame.depth, 128 * 128);
        depth
            .iter()
            .zip(pixels.chunks_exact(4))
            .filter(|(&d, _)| d < frame.max_dist)
            .map(|(_, px)| px[0])
            .collect()
    };

    let textured = hit_luminance(&scene);
    assert!(textured.len() > 100, "both voxels must be visible");
    assert!(
        textured.iter().any(|&v| v < 40),
        "checkerboard black texels must appear (min {})",
        textured.iter().min().unwrap()
    );
    assert!(
        textured.iter().any(|&v| v > 180),
        "checkerboard white texels must appear (max {})",
        textured.iter().max().unwrap()
    );

    // Control: a fresh scene (dummy textures) renders both voxels
    // flat — no dark or bright outliers.
    let (renderer2, scene2) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    let frame = renderer2.render(&gpu.device, &gpu.queue, &scene2, &camera, (128, 128), false);
    let pixels = read_rgba_texture(&gpu, &frame.color, frame.size);
    let depth = read_f32_buffer(&gpu, &frame.depth, 128 * 128);
    let flat: Vec<u8> = depth
        .iter()
        .zip(pixels.chunks_exact(4))
        .filter(|(&d, _)| d < frame.max_dist)
        .map(|(_, px)| px[0])
        .collect();
    assert!(flat.len() > 100);
    // Flat shading legitimately spans ~38 (face turned from the sun:
    // ambient only) to ~117 (lit top face) for 0.5 grey — but it
    // takes only a handful of distinct values (one per face/AO
    // combination), unlike the textured render's checkerboard.
    let distinct: HashSet<u8> = flat.iter().copied().collect();
    assert!(
        flat.iter().all(|&v| v >= 30 && v <= 180),
        "flat control must stay mid-grey (min {}, max {})",
        flat.iter().min().unwrap(),
        flat.iter().max().unwrap()
    );
    assert!(
        distinct.len() <= 16,
        "flat control must be near-uniform per face ({} distinct values)",
        distinct.len()
    );
    let _ = renderer;
}
