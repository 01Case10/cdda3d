//! Phase 9 slice 2 gate (plan §11b, 9.4 + 9.10): raster fine
//! meshing and the `--fine-props` rollout.
//!
//! - CPU boundary fixture: a full-footprint prop against a
//!   wall culls the wall's face toward it (and the floor
//!   slab's top under it); a narrower prop in a doorway gap
//!   culls nothing it does not fully cover. The fine pass is
//!   proven complete and duplicate-free by set equality
//!   against a brute-force `FineSampler` reference.
//! - Face-count budget: the sample scene with the fixture
//!   prop scene, prepared unlit — coarse-only vs fine totals
//!   printed, fine-pass quads > 0, total < 250,000.
//! - 9.10 GPU: raster captures of the sample scene at cutaway
//!   0 — fine OFF vs fine ON with an explicitly EMPTY prop
//!   scene must be pixel-identical; fine ON with the fixture
//!   props must differ, draw more quads, and show at least 3
//!   fixture palette colours. With no wgpu adapter this test
//!   prints a loud NOT RUN banner and returns (it proves
//!   nothing), exactly like the golden gate.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::frame::PropScene;
use mesher::block::{Block, Pass};
use mesher::chunk::ChunkMap;
use mesher::fine::{mesh_chunk_fine, stamp_model, FineLayer, FineSampler, ModelCells, FINE};
use mesher::mesh::{covered_face_cells, Mesh, PassMeshes};
use mesher::mesher::face_visible;
use mesher::scale::WorldScale;

// ---------------------------------------------------------------------
// CPU boundary fixture
// ---------------------------------------------------------------------

/// A `ModelCells` fixture with the PropModelData layout
/// (voxel index `(y * d + z) * w + x`), filled with one block.
struct FilledModel {
    size: [u32; 3],
    block: Block,
}

impl ModelCells for FilledModel {
    fn size(&self) -> [u32; 3] {
        self.size
    }
    fn cell(&self, x: u32, y: u32, z: u32) -> Option<Block> {
        let [w, h, d] = self.size;
        if x >= w || y >= h || z >= d {
            None
        } else {
            Some(self.block)
        }
    }
}

fn blk(id: u32, color: [f32; 4]) -> Block {
    Block {
        id,
        color,
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// Split a mesh's quads into (coarse, fine) by edge length:
/// coarse quads have unit edges, fine quads have 1/FINE edges
/// (at the default block size 1.0). Corner order, normals,
/// colours, UVs and layers are preserved per quad.
fn split_by_edge(mesh: &Mesh) -> (Mesh, Mesh) {
    let mut coarse = Mesh::default();
    let mut fine = Mesh::default();
    for quad in mesh.indices.chunks_exact(6) {
        // Corner order in the index pattern [b, b+1, b+2, b, b+2, b+3].
        let vi = [quad[0], quad[1], quad[2], quad[5]];
        let corners = vi.map(|i| mesh.positions[i as usize]);
        let edge = ((corners[1][0] - corners[0][0]).powi(2)
            + (corners[1][1] - corners[0][1]).powi(2)
            + (corners[1][2] - corners[0][2]).powi(2))
        .sqrt();
        let target = if edge > 0.5 { &mut coarse } else { &mut fine };
        let uvs = vi.map(|i| mesh.uvs[i as usize]);
        target.push_quad_textured(
            corners,
            mesh.normals[quad[0] as usize],
            mesh.colors[quad[0] as usize],
            uvs,
            mesh.layers[quad[0] as usize],
        );
    }
    (coarse, fine)
}

fn count_fine_quads(meshes: &PassMeshes) -> usize {
    meshes
        .meshes
        .iter()
        .map(|m| split_by_edge(m).1.quad_count())
        .sum()
}

/// The fixture world (see the test below for the layout).
fn boundary_fixture() -> (ChunkMap, FineLayer, WorldScale) {
    let scale = WorldScale::default();
    let floor = blk(10, [0.5, 0.4, 0.3, 1.0]);
    let wall = blk(11, [0.7, 0.7, 0.6, 1.0]);
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    // Floor slabs under the props and the doorway.
    for (x, y) in [(1, 3), (2, 3), (2, 2)] {
        map.set_global_block(x, y, 0, floor);
    }
    // Wall row at y = 2: blocks at x = 1 and x = 3 (full
    // height), with the doorway gap at x = 2.
    for x in [1, 3] {
        for z in 0..3 {
            map.set_global_block(x, 2, z, wall);
        }
    }
    map.clear_dirty();

    let mut layer = FineLayer::default();
    // (i) Full-footprint prop on tile (1,3), south of the wall
    // at (1,2): a solid 32x32x32 block of fine cells.
    let full = FilledModel {
        size: [32, 32, 32],
        block: blk(99, [0.9, 0.2, 0.2, 1.0]),
    };
    stamp_model(&mut layer, (1, 3), 32, &full).unwrap();
    // (ii) Partial prop in the doorway tile (2,2): 16 wide,
    // 20 high, 16 deep — narrower than the tile on every axis
    // that matters to the neighbouring walls.
    let partial = FilledModel {
        size: [16, 20, 16],
        block: blk(98, [0.2, 0.2, 0.9, 1.0]),
    };
    stamp_model(&mut layer, (2, 2), 32, &partial).unwrap();
    map.refresh_fine_flags(&layer);
    (map, layer, scale)
}

#[test]
fn phase9_boundary_fixture_culling_and_fine_coverage() {
    let (map, layer, scale) = boundary_fixture();
    assert!(map.chunks[&[0, 0]].has_fine, "fixture chunk is flagged");

    let fine_meshes = mesh_chunk_fine(&map, &layer, [0, 0], scale).expect("chunk present");
    // The coarse-only reference: the same map meshed naively
    // with no fine knowledge.
    let coarse_only = map.mesh_chunk([0, 0], false).expect("chunk present");
    let (coarse_only_set, _) = covered_face_cells(&coarse_only.meshes[Pass::Opaque.index()]);

    // Method: split the fine mesher's output by quad edge
    // length (unit = coarse pass, 1/32 = fine pass) and run
    // `covered_face_cells` on the coarse part — exact face-cell
    // sets, not quad-count comparisons.
    let (coarse_part, fine_part) = split_by_edge(&fine_meshes.meshes[Pass::Opaque.index()]);
    let (coarse_set, coarse_expanded) = covered_face_cells(&coarse_part);
    assert_eq!(
        coarse_expanded,
        coarse_set.len(),
        "coarse part has no duplicate coverage"
    );

    // Sanity: without fine knowledge the wall's south face at
    // (1,2,1) IS drawn (its neighbour is coarse air).
    assert!(
        coarse_only_set.contains(&([1, 2, 1], 2)),
        "naive coarse mesher draws the wall face toward the prop tile"
    );
    // (i) The full-footprint prop covers the whole face plane:
    // the wall's +Y (south, dir 2) face at z-block 1 is culled,
    // and so is the floor slab's +Z (up, dir 4) top under it
    // (the volume's bottom fine layer is fully opaque).
    assert!(
        !coarse_set.contains(&([1, 2, 1], 2)),
        "wall face toward the fully-covered tile must be culled"
    );
    assert!(
        !coarse_set.contains(&([1, 3, 0], 4)),
        "floor slab top under the full prop must be culled"
    );
    // (ii) The doorway prop covers neither neighbouring wall
    // face fully: both are still drawn (no holes), and the
    // doorway slab's top is drawn too.
    assert!(
        coarse_set.contains(&([1, 2, 1], 0)),
        "west wall's +X face into the doorway must be drawn (partial cover)"
    );
    assert!(
        coarse_set.contains(&([3, 2, 1], 1)),
        "east wall's -X face into the doorway must be drawn (no cover on that plane)"
    );
    assert!(
        coarse_set.contains(&([2, 2, 0], 4)),
        "doorway floor slab top must be drawn (partial cover)"
    );

    // Fine-pass completeness: brute-force reference over every
    // STAMPED fine cell of both prop tiles — every direction
    // `face_visible` leaves open is expected, nothing else.
    let sampler = FineSampler::new(&map, &layer);
    let mut expected: HashSet<([i32; 3], usize)> = HashSet::new();
    for tile in [(1, 3), (2, 2)] {
        for ly in 0..FINE {
            for lx in 0..FINE {
                for fz in 32..32 + 96 {
                    let (fx, fy) = (tile.0 * FINE + lx, tile.1 * FINE + ly);
                    if !sampler.is_stamped(fx, fy, fz) {
                        continue;
                    }
                    let cell = sampler.block_at_fine(fx, fy, fz);
                    for (di, dir) in mesher::mesher::FACE_DIRS.iter().enumerate() {
                        let n = sampler.block_at_fine(fx + dir[0], fy + dir[1], fz + dir[2]);
                        if face_visible(cell, n) {
                            expected.insert(([fx, fy, fz], di));
                        }
                    }
                }
            }
        }
    }
    assert!(!expected.is_empty(), "reference must not be vacuous");
    // Scale the fine part's world positions x32 into fine grid
    // units and expand to face cells.
    let mut scaled = fine_part.clone();
    for p in scaled.positions.iter_mut() {
        *p = [p[0] * 32.0, p[1] * 32.0, p[2] * 32.0];
    }
    let (fine_set, fine_expanded) = covered_face_cells(&scaled);
    assert_eq!(
        fine_expanded,
        fine_set.len(),
        "fine pass has no duplicate coverage (no z-fighting duplicates)"
    );
    assert_eq!(
        fine_set, expected,
        "fine-pass coverage must equal the brute-force FineSampler reference exactly"
    );
}

// ---------------------------------------------------------------------
// Face-count budget (CPU preparation)
// ---------------------------------------------------------------------

#[test]
fn phase9_face_count_budget_sample_scene() {
    let visible = [0];
    let scene = PropScene::fixture("sample-scene").expect("fixture scene");
    let (_cm, coarse_stats, _) =
        engine::scene_prep::prepare_raster_sample_v(&visible, None, None).unwrap();
    let (fine_meshes, fine_stats, spec) =
        engine::scene_prep::prepare_raster_sample_vp(&visible, None, None, Some(&scene)).unwrap();
    let fine_quads = count_fine_quads(&fine_meshes);
    println!(
        "phase9 face budget: coarse-only total {} quads; with fixture props total {} quads \
         (fine-pass {} quads)",
        coarse_stats.quads, fine_stats.quads, fine_quads
    );
    assert!(
        spec.is_some(),
        "the untextured fine path returns the prop-only texture spec"
    );
    assert!(fine_quads > 0, "fine pass must contribute quads");
    assert!(
        fine_stats.quads < 250_000,
        "total quads {} must stay under the 250,000 budget",
        fine_stats.quads
    );
}

// ---------------------------------------------------------------------
// 9.10 GPU gate
// ---------------------------------------------------------------------

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda-voxel-phase9-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn loud_skip(reason: &str) -> bool {
    let banner = format!(
        "================================================================\n\
         PHASE 9 RENDER GATE: NOT RUN — {reason}\n\
         This run proves NOTHING about fine-props rendering. The gate\n\
         only counts on a machine with a wgpu adapter (lavapipe counts).\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
    true
}

fn sample_args(out: &Path, fine_props: bool, props: Option<PropScene>) -> CaptureArgs {
    CaptureArgs {
        snapshot: "sample-scene".to_string(),
        renderer: RendererKind::Raster,
        camera_target: (6.5, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 50.0,
        distance: 15.0,
        size: (640, 360),
        out: out.to_path_buf(),
        cutaway_z: 0,
        fine_props,
        props,
        ..CaptureArgs::default()
    }
}

/// Per-pixel diff at the golden gate's tolerance model:
/// (pixels whose max channel delta exceeds 8, mean abs delta).
fn diff_stats(a: &image::RgbaImage, b: &image::RgbaImage) -> (usize, f64) {
    assert_eq!(a.dimensions(), b.dimensions());
    let mut mismatched = 0usize;
    let mut err_sum = 0f64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let mut px_max = 0u32;
        for c in 0..4 {
            let d = (pa.0[c] as i32 - pb.0[c] as i32).unsigned_abs();
            px_max = px_max.max(d);
            err_sum += d as f64;
        }
        if px_max > 8 {
            mismatched += 1;
        }
    }
    let total = (a.width() * a.height()) as usize;
    (mismatched, err_sum / (total * 4) as f64)
}

#[test]
fn phase9_gpu_empty_identical_and_fixture_visible() {
    let _guard = gpu_lock();
    let scratch = scratch_dir("gpu");

    // (a) Fine OFF.
    let off_path = scratch.join("off.png");
    let t0 = Instant::now();
    let off = match run_capture(&sample_args(&off_path, false, None)) {
        Ok(o) => o,
        Err(CaptureError::NoAdapter) => {
            loud_skip("no wgpu adapter available (capture returned NoAdapter)");
            return;
        }
        Err(e) => panic!("phase9 off capture failed: {e}"),
    };
    let off_elapsed = t0.elapsed();
    assert!(
        off.quads > 0 && off.distinct_colors > 5,
        "off capture looks blank ({} quads, {} colors)",
        off.quads,
        off.distinct_colors
    );

    // (a) Fine ON with an explicitly EMPTY prop scene.
    let empty_path = scratch.join("empty.png");
    let t1 = Instant::now();
    let empty = run_capture(&sample_args(
        &empty_path,
        true,
        Some(PropScene {
            models: Vec::new(),
            placements: Vec::new(),
        }),
    ))
    .expect("phase9 empty-props capture");
    let empty_elapsed = t1.elapsed();
    assert!(empty.quads > 0 && empty.distinct_colors > 5);

    let off_img = image::open(&off_path).unwrap().to_rgba8();
    let empty_img = image::open(&empty_path).unwrap().to_rgba8();
    let (mismatched, mean_abs) = diff_stats(&off_img, &empty_img);
    println!(
        "phase9 9.10(a): off {} quads in {:?}; empty-props {} quads in {:?}; \
         diff {mismatched} mismatched px, mean |err| {mean_abs}",
        off.quads, off_elapsed, empty.quads, empty_elapsed
    );
    assert_eq!(
        mismatched, 0,
        "fine ON with an empty prop scene must be pixel-identical to fine OFF \
         (0 mismatched px at tol 8)"
    );
    assert_eq!(
        mean_abs, 0.0,
        "mean abs error must be 0 for the empty scene"
    );

    // (b) Fine ON with the fixture props (via --fine-props
    // semantics: flag on, no explicit scene).
    let fix_path = scratch.join("fixture.png");
    let t2 = Instant::now();
    let fix = run_capture(&sample_args(&fix_path, true, None)).expect("phase9 fixture capture");
    let fix_elapsed = t2.elapsed();
    assert!(fix.quads > 0 && fix.distinct_colors > 5);
    let fix_img = image::open(&fix_path).unwrap().to_rgba8();
    let (fx_mismatched, fx_mean) = diff_stats(&off_img, &fix_img);
    let total_px = (fix_img.width() * fix_img.height()) as usize;
    let frac = fx_mismatched as f64 / total_px as f64;
    println!(
        "phase9 9.10(b): fixture {} quads in {:?} (off {} quads); diff {fx_mismatched} px \
         ({:.3}%), mean |err| {fx_mean:.3}",
        fix.quads,
        fix_elapsed,
        off.quads,
        frac * 100.0
    );
    assert!(
        frac > 0.001,
        "fixture props must change more than 0.1% of pixels (got {:.4}%)",
        frac * 100.0
    );
    assert!(
        fix.quads > off.quads,
        "fixture capture must draw more quads than off ({} vs {})",
        fix.quads,
        off.quads
    );

    // Palette colours present in the frame. Conversion used:
    // NONE beyond comparing in the capture's own encoding —
    // the PNG bytes are sRGB-encoded values and the palette
    // literals are sRGB byte triples; the renderer samples the
    // sRGB texture array (decoding to linear), shades, and the
    // sRGB target re-encodes, so a face whose shade factor is
    // near 1 (the props' top faces) lands within a few levels
    // of the palette byte. Tolerance is 12 per channel.
    let scene = PropScene::fixture("sample-scene").expect("fixture scene");
    let mut palette: Vec<[u8; 3]> = Vec::new();
    for (_, model) in &scene.models {
        for rgb in &model.palette {
            if !palette.contains(rgb) {
                palette.push(*rgb);
            }
        }
    }
    let frame_colors: HashSet<[u8; 3]> =
        fix_img.pixels().map(|p| [p.0[0], p.0[1], p.0[2]]).collect();
    let mut present: Vec<[u8; 3]> = Vec::new();
    for rgb in &palette {
        let found = frame_colors
            .iter()
            .any(|c| (0..3).all(|ch| (c[ch] as i32 - rgb[ch] as i32).abs() <= 12));
        if found {
            present.push(*rgb);
        }
    }
    println!(
        "phase9 9.10(b): {} of {} fixture palette colours present in frame: {present:?}",
        present.len(),
        palette.len()
    );
    assert!(
        present.len() >= 3,
        "at least 3 fixture palette colours must be present (found {}: {present:?})",
        present.len()
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
