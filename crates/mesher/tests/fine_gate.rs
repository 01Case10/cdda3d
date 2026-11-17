//! Phase 9 gate 9.1 (plan §11b): fine volumes + fine sampler.
//!
//! - a stamped cell wins over its containing coarse block;
//! - coarse expansion is EXACT: every fine cell of an
//!   unstamped tile resolves to its containing coarse block,
//!   including across a floor-slab boundary in z;
//! - cross-boundary reads are correct, including negative
//!   tile coordinates (Euclidean division);
//! - the v2 front-flush-south mapping: a model with depth
//!   d < 32 puts its FRONT (mz = d - 1) at local fine y 31
//!   and its back (mz = 0) at local fine y 32 - d;
//! - an out-of-tile model is a build error and leaves the
//!   layer unchanged;
//! - the restamp drop removes a tile's volume and clears
//!   the chunk's `has_fine` flag;
//! - memory: a bookcase-like volume's run-length footprint
//!   is under 1/4 of the dense equivalent (printed).

use mesher::block::Block;
use mesher::chunk::ChunkMap;
use mesher::fine::{stamp_model, FineLayer, FineSampler, ModelCells, StampError, FINE};
use mesher::grid::VoxelSampler;
use mesher::scale::WorldScale;
use mesher::Pass;

/// A `ModelCells` fixture with the PropModelData layout:
/// voxel index `(y * d + z) * w + x`.
struct TestModel {
    size: [u32; 3],
    cells: Vec<Option<Block>>,
}

impl TestModel {
    fn new(size: [u32; 3]) -> Self {
        let n = (size[0] * size[1] * size[2]) as usize;
        Self {
            size,
            cells: vec![None; n],
        }
    }
    fn set(&mut self, x: u32, y: u32, z: u32, block: Block) {
        let [w, _h, d] = self.size;
        let i = ((y * d + z) * w + x) as usize;
        self.cells[i] = Some(block);
    }
    fn filled(size: [u32; 3], block: Block) -> Self {
        let mut m = Self::new(size);
        m.cells.fill(Some(block));
        m
    }
}

impl ModelCells for TestModel {
    fn size(&self) -> [u32; 3] {
        self.size
    }
    fn cell(&self, x: u32, y: u32, z: u32) -> Option<Block> {
        let [w, h, d] = self.size;
        if x >= w || y >= h || z >= d {
            return None;
        }
        self.cells[((y * d + z) * w + x) as usize]
    }
}

fn blk(id: u32, r: f32) -> Block {
    Block {
        id,
        color: [r, 0.5, 0.25, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// A coarse world: chunk [0,0] and [-1,-1], one level.
/// Tile (4,4) has a floor slab only (block z 0); tile (2,3)
/// is full-height; tile (-1,-1) has a slab.
fn coarse_world() -> ChunkMap {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    map.add_chunk([-1, -1], 1);
    map.set_global_block(4, 4, 0, blk(10, 0.1));
    for z in 0..3 {
        map.set_global_block(2, 3, z, blk(11, 0.2));
    }
    map.set_global_block(-1, -1, 0, blk(12, 0.3));
    map.clear_dirty();
    map
}

#[test]
fn coarse_expansion_exact_everywhere_unstamped() {
    let map = coarse_world();
    let layer = FineLayer::default();
    let sampler = FineSampler::new(&map, &layer);
    // Every fine cell over a patch of tiles (positive and
    // negative) and the full level height in fine z —
    // including fz 0..32 vs 32..64 across the slab boundary
    // (coarse block z 0 vs 1) — equals the containing coarse
    // block, and nothing is stamped.
    for tx in -1..6 {
        for ty in -1..6 {
            for lx in 0..FINE {
                for ly in 0..FINE {
                    for fz in 0..96i32 {
                        let (fx, fy) = (tx * FINE + lx, ty * FINE + ly);
                        let want = map.block_at(
                            fx.div_euclid(FINE),
                            fy.div_euclid(FINE),
                            fz.div_euclid(FINE),
                        );
                        assert_eq!(
                            sampler.block_at_fine(fx, fy, fz),
                            want,
                            "fine ({fx},{fy},{fz}) must expand to its coarse block"
                        );
                        assert!(!sampler.is_stamped(fx, fy, fz));
                    }
                }
            }
        }
    }
    // Below the world (negative fine z) expands to air.
    assert_eq!(sampler.block_at_fine(0, 0, -1), Block::AIR);
}

#[test]
fn stamped_cell_wins_over_coarse() {
    let map = coarse_world();
    let mut layer = FineLayer::default();
    // A 2x2x2 model stamped on the full-height tile (2,3) at
    // the slab top (fine z 32): its cells sit inside coarse
    // block z 1, which is solid — the stamp must WIN there,
    // while unstamped cells of the same tile keep the coarse
    // block.
    let prop = blk(99, 0.9);
    let model = TestModel::filled([2, 2, 2], prop);
    let report = stamp_model(&mut layer, (2, 3), 32, &model).unwrap();
    assert_eq!(report.stamped_cells, 8);
    assert_eq!(report.volumes, 1);
    let sampler = FineSampler::new(&map, &layer);
    for my in 0..2 {
        for mz in 0..2 {
            for mx in 0..2 {
                let fx = 2 * FINE + mx;
                let fy = 3 * FINE + (FINE - 2) + mz;
                let fz = 32 + my;
                assert_eq!(sampler.block_at_fine(fx, fy, fz), prop, "stamped cell wins");
                assert!(sampler.is_stamped(fx, fy, fz));
            }
        }
    }
    // Neighbouring unstamped fine cell in the same coarse
    // block: the coarse wall block, not the prop, not air.
    let fx = 2 * FINE + 5;
    let fy = 3 * FINE + 5;
    assert_eq!(sampler.block_at_fine(fx, fy, 32), blk(11, 0.2));
    assert!(!sampler.is_stamped(fx, fy, 32));
    // Below the volume's z-range the tile is coarse again.
    assert_eq!(
        sampler.block_at_fine(2 * FINE, 3 * FINE + 31, 0),
        blk(11, 0.2)
    );
}

#[test]
fn cross_boundary_and_negative_tiles() {
    let map = coarse_world();
    let mut layer = FineLayer::default();
    // Stamp the negative tile (-1,-1) and tile (0,0)'s
    // neighbour (-1, 0)... tile (-1,0) has no chunk in the
    // coarse world; the fine layer is independent of chunks,
    // and unstamped reads there expand to AIR coarse.
    let edge = blk(77, 0.7);
    let mut model = TestModel::new([32, 1, 1]);
    model.set(31, 0, 0, edge); // east-most cell of the tile
    model.set(0, 0, 0, blk(78, 0.8)); // west-most cell
    stamp_model(&mut layer, (-1, -1), 32, &model).unwrap();
    let sampler = FineSampler::new(&map, &layer);
    // fx = -1 is tile -1's local x 31; fx = -32 is local 0.
    assert_eq!(sampler.block_at_fine(-1, -1, 32), edge);
    assert_eq!(sampler.block_at_fine(-32, -1, 32), blk(78, 0.8));
    // One step east across the boundary: tile 0, unstamped,
    // coarse air (no chunk content there at block z 1).
    assert_eq!(sampler.block_at_fine(0, -1, 32), Block::AIR);
    assert!(!sampler.is_stamped(0, -1, 32));
    // The stamped tile's slab still expands below the volume.
    assert_eq!(sampler.block_at_fine(-1, -1, 0), blk(12, 0.3));
}

#[test]
fn front_flush_south_mapping() {
    let mut layer = FineLayer::default();
    // Depth 5 < 32: FRONT (mz = 4) must land at local fine
    // y = 31 (flush on the south edge), back (mz = 0) at
    // local fine y = 32 - 5 = 27.
    let front = blk(50, 0.5);
    let back = blk(51, 0.6);
    let mut model = TestModel::new([3, 2, 5]);
    model.set(1, 0, 4, front);
    model.set(1, 0, 0, back);
    model.set(1, 1, 4, front); // one up: fine z = z0 + 1
    stamp_model(&mut layer, (1, 1), 0, &model).unwrap();
    let vol = &layer.volumes_for_tile((1, 1))[0];
    assert_eq!(
        vol.get(1, 31, 0),
        Some(front),
        "front flush on the south edge"
    );
    assert_eq!(vol.get(1, 27, 0), Some(back), "back at 32 - d");
    assert_eq!(
        vol.get(1, 31, 1),
        Some(front),
        "model y is fine z, bottom up"
    );
    assert_eq!(
        vol.get(1, 30, 0),
        None,
        "the cell behind the front is empty"
    );
    assert_eq!(
        vol.get(1, 26, 0),
        None,
        "north of the back is outside the model"
    );
    assert_eq!(vol.solid_count(), 3);
}

#[test]
fn out_of_tile_model_errors_and_layer_unchanged() {
    let mut layer = FineLayer::default();
    let good = TestModel::filled([4, 4, 4], blk(60, 0.4));
    stamp_model(&mut layer, (0, 0), 0, &good).unwrap();
    let before = layer.volumes_for_tile((0, 0))[0].solid_count();
    for size in [[33, 1, 1], [1, 1, 33], [1, 97, 1]] {
        let bad = TestModel::new(size);
        let err = stamp_model(&mut layer, (0, 0), 0, &bad).unwrap_err();
        assert_eq!(err, StampError::ExceedsTile { size });
        assert_eq!(
            layer.volumes_for_tile((0, 0))[0].solid_count(),
            before,
            "a refused stamp leaves the existing volume untouched"
        );
    }
    // And on a fresh tile, nothing is inserted at all.
    let bad = TestModel::new([40, 1, 1]);
    assert!(stamp_model(&mut layer, (5, 5), 0, &bad).is_err());
    assert!(layer.volumes_for_tile((5, 5)).is_empty());
    assert_eq!(layer.tile_count(), 1);
}

#[test]
fn restamp_drop_removes_volume_and_clears_chunk_flag() {
    let mut map = coarse_world();
    let mut layer = FineLayer::default();
    let model = TestModel::filled([8, 8, 8], blk(61, 0.4));
    stamp_model(&mut layer, (3, 3), 32, &model).unwrap();
    assert_eq!(layer.tiles_in_chunk([0, 0]), vec![(3, 3)]);
    assert!(layer.tiles_in_chunk([-1, -1]).is_empty());
    map.refresh_fine_flags(&layer);
    assert!(
        map.chunks[&[0, 0]].has_fine,
        "chunk with a stamped tile is flagged"
    );
    assert!(
        !map.chunks[&[-1, -1]].has_fine,
        "neighbour chunk stays unflagged"
    );

    // The delta drop: furniture changed to something
    // model-less — the volume goes away and the flag clears.
    assert_eq!(layer.remove_tile((3, 3)), 1);
    assert!(layer.is_empty());
    map.refresh_fine_flags(&layer);
    assert!(
        !map.chunks[&[0, 0]].has_fine,
        "flag clears with the last volume"
    );

    // Level-scoped drop: two volumes at different z0 on one
    // tile; removing the level-0 range keeps level 1's.
    stamp_model(&mut layer, (3, 3), 32, &model).unwrap();
    stamp_model(&mut layer, (3, 3), 128, &model).unwrap();
    assert_eq!(layer.volumes_for_tile((3, 3)).len(), 2);
    assert_eq!(layer.remove_tile_in_z_range((3, 3), 32, 128), 1);
    let rest = layer.volumes_for_tile((3, 3));
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].z0, 128, "the other level's volume survives");
}

#[test]
fn bookcase_volume_memory_vs_dense() {
    // Bookcase-like fixture: 32 wide, 22 high, 20 deep,
    // mostly solid (solid shelves to y=18, sparse above).
    let mut model = TestModel::new([32, 22, 20]);
    let wood = blk(70, 0.33);
    for y in 0..18 {
        for z in 0..20 {
            for x in 0..32 {
                model.set(x, y, z, wood);
            }
        }
    }
    for x in 0..32 {
        model.set(x, 21, 10, wood);
    }
    let mut layer = FineLayer::default();
    let report = stamp_model(&mut layer, (0, 0), 32, &model).unwrap();
    let dense = (32 * 32 * 96) * std::mem::size_of::<Block>();
    println!(
        "bookcase-like FineVolume: {} bytes vs dense {} bytes ({:.1}x smaller); {} cells stamped",
        report.memory_bytes,
        dense,
        dense as f64 / report.memory_bytes as f64,
        report.stamped_cells
    );
    assert!(
        report.memory_bytes < dense / 4,
        "run-length volume {} must be under 1/4 of dense {}",
        report.memory_bytes,
        dense
    );
    assert_eq!(report.stamped_cells, 32 * 20 * 18 + 32);
}

// ---------------------------------------------------------------------
// Phase 9 slice 3 (9.7): rotated stamping — stamp_model_rotated
// ---------------------------------------------------------------------

use mesher::fine::stamp_model_rotated;

/// An asymmetric fixture for the rotation tests: an L in plan
/// (a full back row + a full west column at `my = 0`), a
/// distinct block in each of the four plan corners, and one
/// tall cell at the front-west corner so height preservation
/// is checked too. No symmetry of any kind: any wrong
/// rotation (mirror, transpose, off-by-one) moves a marker.
fn l_model() -> TestModel {
    let mut m = TestModel::new([6, 3, 5]);
    // L arms at my = 0: back row (mz = 0) and west column
    // (mx = 0).
    for mx in 0..6 {
        m.set(mx, 0, 0, blk(94, 0.94));
    }
    for mz in 0..5 {
        m.set(0, 0, mz, blk(95, 0.95));
    }
    // Distinct corner markers (overwriting the arm cells).
    m.set(0, 0, 0, blk(90, 0.90)); // back-west
    m.set(5, 0, 0, blk(91, 0.91)); // back-east
    m.set(0, 0, 4, blk(92, 0.92)); // front-west
    m.set(5, 0, 4, blk(93, 0.93)); // front-east
    // One tall cell: front-west corner, top height.
    m.set(0, 2, 4, blk(96, 0.96));
    m
}

/// Snapshot of one tile's whole fine volume through the
/// layer: index `(fz * FINE + ly) * FINE + lx` over
/// `z0..z0 + depth` (depth = the layer's level depth, 96).
fn tile_snapshot(layer: &FineLayer, tile: (i32, i32), z0: i32) -> Vec<Option<Block>> {
    let mut out = Vec::with_capacity((FINE * FINE * 96) as usize);
    for fz in z0..z0 + 96 {
        for ly in 0..FINE {
            for lx in 0..FINE {
                out.push(layer.block_at_fine(tile.0 * FINE + lx, tile.1 * FINE + ly, fz));
            }
        }
    }
    out
}

/// One rotation step of the spec transform, applied to a
/// tile-local coordinate: `(lx, ly) -> (FINE - 1 - ly, lx)`.
fn rot_step(lx: i32, ly: i32) -> (i32, i32) {
    (FINE - 1 - ly, lx)
}

#[test]
fn rotated_stamp_turn0_equals_stamp_model() {
    let model = l_model();
    let tile = (4, 4);
    let z0 = 32;
    let mut plain = FineLayer::default();
    let want_report = stamp_model(&mut plain, tile, z0, &model).unwrap();
    // Turns 0 and every multiple of 4 must reproduce
    // stamp_model exactly, cell-for-cell over the whole tile
    // volume (layer reads AND sampler reads against the
    // coarse world).
    let map = coarse_world();
    let plain_sampler = FineSampler::new(&map, &plain);
    for turns in [0u8, 4, 8] {
        let mut rotated = FineLayer::default();
        let report = stamp_model_rotated(&mut rotated, tile, z0, &model, turns).unwrap();
        assert_eq!(report.stamped_cells, want_report.stamped_cells);
        assert_eq!(
            tile_snapshot(&rotated, tile, z0),
            tile_snapshot(&plain, tile, z0),
            "turns {turns}: layer volume must equal stamp_model's"
        );
        let sampler = FineSampler::new(&map, &rotated);
        for fz in z0..z0 + 96 {
            for ly in 0..FINE {
                for lx in 0..FINE {
                    let (fx, fy) = (tile.0 * FINE + lx, tile.1 * FINE + ly);
                    assert_eq!(
                        sampler.block_at_fine(fx, fy, fz),
                        plain_sampler.block_at_fine(fx, fy, fz),
                        "turns {turns}: sampler read at fine ({fx},{fy},{fz})"
                    );
                }
            }
        }
    }
}

#[test]
fn rotated_stamp_quarter_turns_match_independent_rotation() {
    let model = l_model();
    let tile = (1, 2);
    let z0 = 32;
    let mut base = FineLayer::default();
    let base_report = stamp_model_rotated(&mut base, tile, z0, &model, 0).unwrap();
    let base_snap = tile_snapshot(&base, tile, z0);
    let idx = |lx: i32, ly: i32, fz: i32| ((fz * FINE + ly) * FINE + lx) as usize;
    for turns in 1..=3u8 {
        // Independently rotate the turn-0 snapshot: every
        // stamped source cell (lx, ly) lands at rot^turns.
        let mut expected: Vec<Option<Block>> = vec![None; base_snap.len()];
        for fz in 0..96 {
            for ly in 0..FINE {
                for lx in 0..FINE {
                    let cell = base_snap[idx(lx, ly, fz)];
                    if cell.is_none() {
                        continue;
                    }
                    let (mut rx, mut ry) = (lx, ly);
                    for _ in 0..turns {
                        (rx, ry) = rot_step(rx, ry);
                    }
                    expected[idx(rx, ry, fz)] = cell;
                }
            }
        }
        let mut layer = FineLayer::default();
        let report = stamp_model_rotated(&mut layer, tile, z0, &model, turns).unwrap();
        assert_eq!(
            report.stamped_cells, base_report.stamped_cells,
            "turns {turns}: rotation copies every cell, none lost or duplicated"
        );
        assert_eq!(
            tile_snapshot(&layer, tile, z0),
            expected,
            "turns {turns}: stamped volume must equal the independently \
             rotated turn-0 volume, voxel-for-voxel"
        );
    }
    // Spot-check the front marker's facing per turn count:
    // turn 0 front-west corner sits flush SOUTH (ly = 31);
    // one step puts the front on the WEST edge (lx = 0), two
    // on the NORTH edge (ly = 0), three on the EAST (lx = 31).
    let front = blk(92, 0.92);
    let front_at = |turns: u8| {
        let mut layer = FineLayer::default();
        stamp_model_rotated(&mut layer, tile, z0, &model, turns).unwrap();
        let vol = &layer.volumes_for_tile(tile)[0];
        (0..FINE)
            .flat_map(|ly| (0..FINE).map(move |lx| (lx, ly)))
            .find(|&(lx, ly)| vol.get(lx, ly, z0) == Some(front))
            .expect("front marker is stamped")
    };
    assert_eq!(front_at(0), (0, 31), "turn 0: front flush south");
    assert_eq!(front_at(1), (0, 0), "turn 1: front on the west edge");
    assert_eq!(front_at(2), (31, 0), "turn 2: front on the north edge");
    assert_eq!(front_at(3), (31, 31), "turn 3: front on the east edge");
}

#[test]
fn rotated_stamp_oversize_errors_at_every_turn() {
    let model = l_model();
    // A good stamp already on the target tile: every refused
    // rotated stamp must leave it (and the layer) untouched.
    let mut layer = FineLayer::default();
    stamp_model(&mut layer, (0, 0), 0, &model).unwrap();
    let before = tile_snapshot(&layer, (0, 0), 0);
    for size in [[33, 1, 1], [1, 1, 33], [1, 97, 1]] {
        let bad = TestModel::new(size);
        for turns in 0..=3u8 {
            let err = stamp_model_rotated(&mut layer, (0, 0), 0, &bad, turns).unwrap_err();
            assert_eq!(err, StampError::ExceedsTile { size });
            assert_eq!(
                tile_snapshot(&layer, (0, 0), 0),
                before,
                "size {size:?} turns {turns}: a refused stamp leaves the layer unchanged"
            );
        }
        // And on a fresh tile, nothing is inserted at any turn.
        for turns in 0..=3u8 {
            assert!(stamp_model_rotated(&mut layer, (5, 5), 0, &bad, turns).is_err());
        }
        assert!(layer.volumes_for_tile((5, 5)).is_empty());
    }
    assert_eq!(layer.tile_count(), 1);
}
