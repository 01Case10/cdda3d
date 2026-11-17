//! Phase 9 gate 9.2 + 9.3 (plan §11b): world-build furniture
//! stamping over `snapshot_world::build_columns`, and the
//! synthetic solid materials of `textures::PropColorSet`.
//!
//! - (a) round-trip: every model voxel of two fixture prop
//!   models reads back through the world's fine sampler with
//!   its palette colour and synthetic material id; every
//!   non-model fine cell of the prop tiles is exactly the
//!   coarse expansion;
//! - (b) suppression: the coarse grid above the floor slab at
//!   prop tiles is all AIR (no coarse furniture block), the
//!   prop chunk's `has_fine` is set, the neighbouring chunk's
//!   is not;
//! - (c) dedupe: the colour set holds exactly the distinct
//!   colours across both models (they share one), and the
//!   extended texture spec's appended layers are solid with
//!   `material_layers[synthetic_id] == [l, l, l, 1]`;
//! - (d) restamp: changing a tile's furniture to a model-less
//!   id drops its volume (sampler falls back to coarse
//!   expansion); changing it to the other model restamps it;
//! - (e) an out-of-tile model fails the batch and inserts
//!   nothing.

use engine::materials::PropModelData;
use engine::snapshot_world::{
    build_columns, fine_z_origin, prop_block_for, restamp_tile, stamp_furniture, ColumnSpec,
    FurniturePlacement, PropModelCells,
};
use engine::textures::{extend_texture_spec_with_props, PropColorSet};
use mesher::fine::{FineLayer, FineSampler, ModelCells, StampError, FINE};
use mesher::grid::VoxelSampler;
use mesher::scale::WorldScale;
use mesher::Block;
use treebuild::render::TextureSpec;

const FURN_COUNTER: u32 = 100;
const FURN_TANK: u32 = 101;
const FURN_NONE: u32 = 102;
const FURN_HUGE: u32 = 103;

/// Synthetic ids live above every real id in this scene.
const BASE_ID: u32 = 1000;

const BODY: [u8; 3] = [200, 180, 160];
const TOP: [u8; 3] = [90, 60, 40];
const STEEL: [u8; 3] = [140, 150, 160];

fn def(id: u32, string_id: &str) -> proto::TileDefT {
    let mut d = proto::TileDefT::default();
    d.id = id;
    d.string_id = Some(string_id.into());
    d
}

fn block_for(id: u32) -> Block {
    Block {
        id,
        color: [id as f32 / 10.0, 0.5, 0.5, 1.0],
        pass: mesher::Pass::Opaque,
        tex_layers: None,
    }
}

/// Build a PropModelData literal from a per-voxel palette
/// index function (EMPTY = u16::MAX for no voxel).
fn make_model(
    size: [u32; 3],
    palette: Vec<[u8; 3]>,
    f: impl Fn(u32, u32, u32) -> u16,
) -> PropModelData {
    let [w, h, d] = size;
    let mut voxels = Vec::with_capacity((w * h * d) as usize);
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                voxels.push(f(x, y, z));
            }
        }
    }
    PropModelData {
        size,
        palette,
        voxels,
    }
}

/// Counter-like box: solid body, contrasting top slab.
fn counter_model() -> PropModelData {
    make_model([28, 14, 10], vec![BODY, TOP], |_x, y, _z| {
        if y == 13 {
            1 // top
        } else {
            0 // body
        }
    })
}

/// Wide tank-like model: steel barrel on short feet; the
/// feet reuse the counter's BODY colour (the shared palette
/// colour that proves scene-wide dedupe).
fn tank_model() -> PropModelData {
    make_model([32, 12, 12], vec![STEEL, BODY], |x, y, _z| {
        if y < 2 {
            // Feet only at the ends.
            if x < 3 || x >= 29 {
                1
            } else {
                PropModelData::EMPTY
            }
        } else if (3..29).contains(&x) {
            0 // barrel
        } else {
            PropModelData::EMPTY
        }
    })
}

fn huge_model() -> PropModelData {
    make_model([40, 5, 5], vec![STEEL], |_, _, _| 0)
}

struct Fixture {
    map: mesher::chunk::ChunkMap,
    layer: FineLayer,
    counter: PropModelData,
    tank: PropModelData,
    huge: PropModelData,
    colors: PropColorSet,
    spec: TextureSpec,
    first_layer: u32,
}

fn fixture() -> Fixture {
    // Terrain: wood floor under the prop tiles, grass in the
    // neighbouring chunk (tile x 13 -> chunk [1, 0]).
    let mut defs = proto::TileDefsT::default();
    defs.defs = Some(vec![def(1, "t_grass"), def(2, "t_wood_floor")]);
    let table = hardcases::DefTable::from_tiledefs(&defs);
    let columns = vec![
        ColumnSpec {
            x: 2,
            y: 2,
            z_min: 0,
            terrain: vec![2],
        },
        ColumnSpec {
            x: 3,
            y: 2,
            z_min: 0,
            terrain: vec![2],
        },
        ColumnSpec {
            x: 4,
            y: 2,
            z_min: 0,
            terrain: vec![2],
        },
        ColumnSpec {
            x: 13,
            y: 2,
            z_min: 0,
            terrain: vec![1],
        },
    ];
    let scale = WorldScale::default();
    let map = build_columns(&table, &columns, scale, &block_for);

    let counter = counter_model();
    let tank = tank_model();
    let huge = huge_model();
    let colors = PropColorSet::from_models([&counter, &tank].into_iter());
    // A minimal spec: one dummy real layer, ids 0..3 present.
    let mut spec = TextureSpec {
        tile_size: 32,
        layers: vec![vec![7u8; 32 * 32 * 4]],
        material_layers: vec![[0, 0, 0, 0]; 4],
        macro_size: 0,
        macro_layers: Vec::new(),
        material_macros: Vec::new(),
    };
    let first_layer = spec.layers.len() as u32;
    let appended = extend_texture_spec_with_props(&mut spec, &colors, BASE_ID);
    assert_eq!(appended, colors.len());
    Fixture {
        map,
        layer: FineLayer::default(),
        counter,
        tank,
        huge,
        colors,
        spec,
        first_layer,
    }
}

/// Assert EVERY fine cell of `tile` against `model` stamped
/// at `tile`: stamped cells carry the palette colour (within
/// 1/255) and the synthetic id; all other cells are exactly
/// the coarse expansion and unstamped.
fn assert_tile_matches(fx: &Fixture, tile: (i32, i32), model: &PropModelData, scale: WorldScale) {
    let sampler = FineSampler::new(&fx.map, &fx.layer);
    let z0 = fine_z_origin(scale, 0);
    let [w, h, d] = model.size;
    for ly in 0..FINE {
        for lx in 0..FINE {
            for fz in 0..96 {
                let (fx_c, fy_c) = (tile.0 * FINE + lx, tile.1 * FINE + ly);
                // Inverse stamp mapping for this cell.
                let cell =
                    if lx < w as i32 && ly >= FINE - d as i32 && fz >= z0 && fz < z0 + h as i32 {
                        model.get(lx as u32, (fz - z0) as u32, (ly - (FINE - d as i32)) as u32)
                    } else {
                        None
                    };
                let got = sampler.block_at_fine(fx_c, fy_c, fz);
                match cell {
                    Some(rgb) => {
                        assert!(
                            sampler.is_stamped(fx_c, fy_c, fz),
                            "model voxel at fine ({fx_c},{fy_c},{fz}) must be stamped"
                        );
                        assert_eq!(
                            got.id,
                            fx.colors.id_of(BASE_ID, rgb).unwrap(),
                            "synthetic id at fine ({fx_c},{fy_c},{fz})"
                        );
                        for c in 0..3 {
                            assert!(
                                (got.color[c] - rgb[c] as f32 / 255.0).abs() <= 1.0 / 255.0,
                                "colour channel {c} at fine ({fx_c},{fy_c},{fz})"
                            );
                        }
                        assert_eq!(got.pass, mesher::Pass::Opaque);
                        let layer = fx.colors.layer_of(fx.first_layer, rgb).unwrap();
                        assert_eq!(got.tex_layers, Some([layer, layer, layer]));
                    }
                    None => {
                        assert!(
                            !sampler.is_stamped(fx_c, fy_c, fz),
                            "non-model cell at fine ({fx_c},{fy_c},{fz}) must be unstamped"
                        );
                        let want = fx.map.block_at(
                            fx_c.div_euclid(FINE),
                            fy_c.div_euclid(FINE),
                            fz.div_euclid(FINE),
                        );
                        assert_eq!(got, want, "coarse expansion at fine ({fx_c},{fy_c},{fz})");
                    }
                }
            }
        }
    }
}

#[test]
fn phase9_stamp_round_trip_suppression_and_dedupe() {
    let mut fx = fixture();
    let scale = WorldScale::default();
    assert_eq!(fine_z_origin(scale, 0), 32, "origin is the floor-slab top");

    // (c) dedupe: BODY (counter body + tank feet), TOP, STEEL.
    assert_eq!(fx.colors.len(), 3, "distinct colours across both models");
    assert_eq!(
        fx.colors.colors(),
        &[BODY, TOP, STEEL],
        "first-appearance order"
    );
    // (c) extended spec: appended layers are solid, ids map
    // to [l, l, l, 1].
    for (i, rgb) in fx.colors.colors().iter().enumerate() {
        let layer = fx.first_layer + i as u32;
        let bytes = &fx.spec.layers[layer as usize];
        assert_eq!(bytes.len(), 32 * 32 * 4);
        assert!(
            bytes
                .chunks_exact(4)
                .all(|px| px == [rgb[0], rgb[1], rgb[2], 255]),
            "appended layer {layer} is solid {rgb:?}"
        );
        let id = BASE_ID + 1 + i as u32;
        assert_eq!(
            fx.spec.material_layers[id as usize],
            [layer, layer, layer, 1],
            "synthetic id {id} maps to its solid layer on all faces"
        );
        assert_eq!(fx.colors.id_of(BASE_ID, *rgb), Some(id));
        assert_eq!(fx.colors.layer_of(fx.first_layer, *rgb), Some(layer));
    }

    let block_fn = prop_block_for(&fx.colors, BASE_ID, fx.first_layer);
    let counter_adapter = PropModelCells::new(&fx.counter, &block_fn);
    let tank_adapter = PropModelCells::new(&fx.tank, &block_fn);
    let huge_adapter = PropModelCells::new(&fx.huge, &block_fn);
    let resolve = |id: u32| -> Option<&dyn ModelCells> {
        match id {
            FURN_COUNTER => Some(&counter_adapter),
            FURN_TANK => Some(&tank_adapter),
            FURN_HUGE => Some(&huge_adapter),
            _ => None,
        }
    };

    let placements = [
        FurniturePlacement {
            x: 2,
            y: 2,
            level: 0,
            furn_id: FURN_COUNTER,
        },
        FurniturePlacement {
            x: 3,
            y: 2,
            level: 0,
            furn_id: FURN_TANK,
        },
        FurniturePlacement {
            x: 4,
            y: 2,
            level: 0,
            furn_id: FURN_NONE,
        },
    ];
    let report = stamp_furniture(&mut fx.map, &mut fx.layer, &placements, scale, &resolve).unwrap();
    assert_eq!(
        report.skipped, 1,
        "the model-less placement is skipped and counted"
    );
    assert_eq!(report.volumes, 2);
    assert_eq!(
        report.stamped_cells,
        fx.counter.solid_count() + fx.tank.solid_count(),
        "every solid model voxel was stamped"
    );
    assert!(report.memory_bytes > 0);

    // (a) round-trip, both prop tiles, every fine cell.
    assert_tile_matches(&fx, (2, 2), &fx.counter, scale);
    assert_tile_matches(&fx, (3, 2), &fx.tank, scale);

    // (b) suppression: above the floor slab (coarse z 1..3)
    // at the prop tiles the coarse grid is all AIR; the slab
    // itself is the wood floor.
    let chunk = &fx.map.chunks[&[0, 0]];
    for (tx, ty) in [(2, 2), (3, 2)] {
        assert_eq!(chunk.grid.get(tx, ty, 0).id, 2, "floor slab stays");
        for z in 1..3 {
            assert_eq!(
                chunk.grid.get(tx, ty, z),
                Block::AIR,
                "no coarse furniture block above the slab at ({tx},{ty},{z})"
            );
        }
    }
    assert!(chunk.has_fine, "prop chunk flagged");
    assert!(
        !fx.map.chunks[&[1, 0]].has_fine,
        "neighbouring chunk not flagged"
    );

    // (d) restamp to model-less: volume dropped, sampler
    // falls back to coarse expansion on that tile.
    let r = restamp_tile(
        &mut fx.map,
        &mut fx.layer,
        &FurniturePlacement {
            x: 2,
            y: 2,
            level: 0,
            furn_id: FURN_NONE,
        },
        scale,
        &resolve,
    )
    .unwrap();
    assert_eq!(r.skipped, 1);
    assert_eq!(r.volumes, 0);
    assert!(
        fx.layer.volumes_for_tile((2, 2)).is_empty(),
        "volume dropped"
    );
    let sampler = FineSampler::new(&fx.map, &fx.layer);
    assert!(!sampler.is_stamped(2 * FINE, 2 * FINE + 31, 32));
    assert_eq!(
        sampler.block_at_fine(2 * FINE, 2 * FINE + 31, 32),
        fx.map.block_at(2, 2, 1),
        "falls back to coarse expansion"
    );
    assert!(
        fx.map.chunks[&[0, 0]].has_fine,
        "the tank tile in the same chunk keeps the flag set"
    );

    // (d) restamp to the OTHER model: the tile now matches
    // the tank, voxel for voxel.
    let r = restamp_tile(
        &mut fx.map,
        &mut fx.layer,
        &FurniturePlacement {
            x: 2,
            y: 2,
            level: 0,
            furn_id: FURN_TANK,
        },
        scale,
        &resolve,
    )
    .unwrap();
    assert_eq!(r.stamped_cells, fx.tank.solid_count());
    assert_tile_matches(&fx, (2, 2), &fx.tank, scale);

    // (e) out-of-tile model: Err, and the batch is atomic —
    // even the good placement in the same batch stamps
    // nothing.
    let err = stamp_furniture(
        &mut fx.map,
        &mut fx.layer,
        &[
            FurniturePlacement {
                x: 6,
                y: 6,
                level: 0,
                furn_id: FURN_COUNTER,
            },
            FurniturePlacement {
                x: 5,
                y: 5,
                level: 0,
                furn_id: FURN_HUGE,
            },
        ],
        scale,
        &resolve,
    )
    .unwrap_err();
    assert_eq!(err, StampError::ExceedsTile { size: [40, 5, 5] });
    assert!(fx.layer.volumes_for_tile((5, 5)).is_empty());
    assert!(
        fx.layer.volumes_for_tile((6, 6)).is_empty(),
        "batch validation happens before any stamping"
    );
}
