//! Fake-data scenes shared by the gate tests, the visual dumps and
//! the perf measurement. All scenes are built from the mesher's
//! fake tile ids and hand-authored game-style lit/seen data — the
//! same "fake house first" pattern Phases 2/2b/2c used, because none
//! of gates 7.1–7.9 needs the real fork running (see the Phase 7
//! doc for that statement in full).

use mesher::block::*;
use mesher::chunk::{ChunkMap, CHUNK_H, CHUNK_W};
use mesher::scale::WorldScale;
use proto::LitLevel;

use crate::grid::LightGrid;
use crate::mapping::TileLight;

fn lit(l: LitLevel, seen: u8) -> TileLight {
    TileLight::new(l, seen)
}
fn occ(l: LitLevel, seen: u8) -> TileLight {
    TileLight::new(l, seen).occluding(true)
}

/// The fake house as a `ChunkMap` — a local replica of
/// `engine::world::build_fake_house` (this crate may not depend on
/// the engine crate, so the 30-line builder is duplicated here, as
/// treebuild's gate tests also duplicate it).
pub fn fake_house_map() -> ChunkMap {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    {
        let chunk = map.add_chunk([0, 0], 2);
        for y in 0..CHUNK_H {
            for x in 0..CHUNK_W {
                chunk.set_tile(x, y, 0, TILE_GROUND, scale);
            }
        }
        for y in 1..CHUNK_H - 1 {
            for x in 1..CHUNK_W - 1 {
                chunk.set_tile(x, y, 0, TILE_FLOOR_WOOD, scale);
            }
        }
        for i in 0..CHUNK_W {
            chunk.set_tile(i, 0, 0, TILE_WALL, scale);
            chunk.set_tile(i, CHUNK_H - 1, 0, TILE_WALL, scale);
            chunk.set_tile(0, i, 0, TILE_WALL, scale);
            chunk.set_tile(CHUNK_W - 1, i, 0, TILE_WALL, scale);
        }
        chunk.set_tile(6, CHUNK_H - 1, 0, TILE_DOOR, scale);
        chunk.set_tile(3, 0, 0, TILE_WINDOW, scale);
        chunk.set_tile(0, 6, 0, TILE_WINDOW, scale);
        chunk.set_tile(CHUNK_W - 1, 6, 0, TILE_WINDOW, scale);
        for y in 0..CHUNK_H {
            for x in 0..CHUNK_W {
                chunk.set_tile(x, y, 1, TILE_ROOF, scale);
            }
        }
    }
    map.clear_dirty();
    map
}

/// Light for the fake house *at night with a lit interior*
/// (gate 7.8 scene 1): exterior ground `DARK`, interior floor
/// `LIT`/`BRIGHT` near the centre (a lamp), walls occluding with
/// their own `LIT` value, roof level `DARK` (it is only rendered
/// when the cutaway shows it; the dumps use the level-0 cutaway).
pub fn fake_house_night_light() -> LightGrid {
    let mut g = LightGrid::filled([0, 0, 0], [12, 12, 2], lit(LitLevel::DARK, 255));
    for y in 0..12 {
        for x in 0..12 {
            let perimeter = x == 0 || y == 0 || x == 11 || y == 11;
            if perimeter {
                g.set(x, y, 0, occ(LitLevel::LIT, 255));
            } else {
                // Lamp falloff from the centre, game-style: the game
                // would export these lit_levels; we author them.
                let d = ((x as i32 - 6).abs() + (y as i32 - 6).abs()) as u32;
                let l = match d {
                    0..=1 => LitLevel::BRIGHT,
                    2..=3 => LitLevel::LIT,
                    _ => LitLevel::LOW,
                };
                g.set(x, y, 0, lit(l, 255));
            }
        }
    }
    g
}

/// Gate 7.3/7.4 fixture: two 5-wide rooms side by side along x,
/// separated by a wall column at x=5, in a single 11x5 tile grid at
/// one z-level. Left room `BRIGHT`, right room `DARK`, wall tiles
/// occluding. Authored in *one* grid so the same fixture also spans
/// a chunk border when tiled — see `two_room_grid_wide`.
pub fn two_room_grid() -> LightGrid {
    let mut g = LightGrid::filled([0, 0, 0], [11, 5, 1], lit(LitLevel::DARK, 255));
    for y in 0..5 {
        for x in 0..5 {
            g.set(x, y, 0, lit(LitLevel::BRIGHT, 255));
        }
        g.set(5, y, 0, occ(LitLevel::LIT, 255));
        // x = 6..11 stay DARK.
    }
    g
}

/// Gate 7.4 fixture: a smooth horizontal gradient `DARK -> BRIGHT`
/// across 24 tiles (two chunks: x 0..12 and 12..24) at one z-level.
/// Seen decreases/increases in step with lit so both orderings are
/// exercised; the *rendered* (smoothed) gradient is what the test
/// checks for border seams.
pub fn gradient_grid() -> LightGrid {
    let levels = [
        LitLevel::DARK,
        LitLevel::LOW,
        LitLevel::BRIGHT_ONLY,
        LitLevel::LIT,
        LitLevel::BRIGHT,
    ];
    let mut g = LightGrid::new([0, 0, 0], [24, 4, 1]);
    for y in 0..4 {
        for x in 0..24 {
            // 24 tiles mapped onto a continuous t in [0,1], then
            // quantized into (level, seen) pairs the way the game
            // would export a smooth falloff.
            let t = x as f32 / 23.0;
            let scaled = t * 4.999;
            let li = (scaled as usize).min(4);
            let frac = scaled - li as f32;
            // seen doubles as the within-level fine value here, but
            // 0 means "never seen" in the protocol — a visible tile
            // must carry at least 1, so sweep [1, 255] within a band.
            let seen = (1.0 + frac * 254.0).round() as u8;
            // For the top level, seen sweeps the BRIGHT band.
            g.set(x, y, 0, lit(levels[li], seen));
        }
    }
    g
}

/// Gate 7.8 scene 2: flashlight in a field. A 24x24 `DARK` field at
/// night; from the player tile at (4, 12) a cone toward +x carries
/// game-style falloff: `BRIGHT` at the core, then `LIT`, `LOW`,
/// `DARK` with distance — authored as lit data (the game computes
/// the cone from its lightmap; the engine only presents it).
pub fn flashlight_scene() -> (ChunkMap, LightGrid) {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    for cy in 0..2 {
        for cx in 0..2 {
            let chunk = map.add_chunk([cx, cy], 1);
            for y in 0..CHUNK_H {
                for x in 0..CHUNK_W {
                    chunk.set_tile(x, y, 0, TILE_GROUND, scale);
                }
            }
        }
    }
    map.clear_dirty();
    let mut g = LightGrid::filled([0, 0, 0], [24, 24, 1], lit(LitLevel::DARK, 200));
    let (px, py) = (4i32, 12i32);
    for y in 0..24i32 {
        for x in 0..24i32 {
            let dx = x - px;
            let dy = (y - py).abs();
            if dx < 0 {
                continue;
            }
            // Cone: half-width grows with distance; outside = DARK.
            if dy as f32 > 1.0 + dx as f32 * 0.45 {
                continue;
            }
            let dist = ((dx * dx + (y - py) * (y - py)) as f32).sqrt();
            let (l, seen) = match dist {
                d if d <= 2.5 => (LitLevel::BRIGHT, 255),
                d if d <= 5.5 => (LitLevel::LIT, 230),
                d if d <= 9.5 => (LitLevel::LOW, 200),
                _ => (LitLevel::DARK, 200),
            };
            if l != LitLevel::DARK {
                g.set(x, y, 0, lit(l, seen));
            }
        }
    }
    g.set(px, py, 0, lit(LitLevel::BRIGHT, 255));
    (map, g)
}

/// Gate 7.8 scene 3: a remembered street. Layout (24x12, one level):
/// a horizontal road (y = 5..7) with house walls/floors north of it.
/// The west half was visited and is now out of view: every tile
/// there is `MEMORIZED` (terrain/furniture only, per the snapshot
/// memory-layer semantics). The east half around the player is
/// currently seen (`LIT` street, `DARK` night). Beyond the street's
/// east end, tiles are `BLANK` (never seen) and must not render.
pub fn remembered_street_scene() -> (ChunkMap, LightGrid) {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    for cx in 0..2 {
        let chunk = map.add_chunk([cx, 0], 1);
        for y in 0..CHUNK_H {
            for x in 0..CHUNK_W {
                let gx = cx as usize * CHUNK_W + x;
                let tile = if (5..7).contains(&y) {
                    TILE_GROUND // road strip (ground tile as road stand-in)
                } else if y < 5 {
                    if gx % 6 == 0 || y == 4 {
                        TILE_WALL
                    } else {
                        TILE_FLOOR_WOOD
                    }
                } else {
                    TILE_GROUND // field south of the road
                };
                chunk.set_tile(x, y, 0, tile, scale);
            }
        }
    }
    map.clear_dirty();
    let mut g = LightGrid::new([0, 0, 0], [24, 12, 1]);
    for y in 0..12i32 {
        for x in 0..24i32 {
            let t = if x < 12 {
                TileLight::new(LitLevel::MEMORIZED, 128)
            } else if x < 20 {
                if (5..7).contains(&y) {
                    lit(LitLevel::LIT, 255)
                } else {
                    lit(LitLevel::DARK, 255)
                }
            } else {
                TileLight::BLANK
            };
            // House walls occlude wherever they stand.
            let is_wall = y < 5 && (x % 6 == 0 || y == 4);
            g.set(x, y, 0, if is_wall { t.occluding(true) } else { t });
        }
    }
    (map, g)
}

/// Gate 7.9 scene: the full stored view — 132x132 tiles (11x11
/// chunks) x 3 z-levels (the cutaway window), with a deterministic
/// lit pattern (a lit town centre fading to dark, one memorized
/// quadrant, one blank corner) so the bake exercises every path.
pub fn full_view_scene() -> (ChunkMap, LightGrid) {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    for cy in 0..11 {
        for cx in 0..11 {
            let chunk = map.add_chunk([cx, cy], 3);
            for y in 0..CHUNK_H {
                for x in 0..CHUNK_W {
                    chunk.set_tile(x, y, 0, TILE_GROUND, scale);
                }
            }
            for i in 2..10 {
                chunk.set_tile(i, 2, 1, TILE_WALL, scale);
                chunk.set_tile(i, 9, 1, TILE_WALL, scale);
            }
        }
    }
    map.clear_dirty();
    let mut g = LightGrid::new([0, 0, 0], [132, 132, 3]);
    for z in 0..3i32 {
        for y in 0..132i32 {
            for x in 0..132i32 {
                let d = ((x - 66).abs() + (y - 66).abs()) as u32;
                let mut t = match d {
                    0..=10 => lit(LitLevel::BRIGHT, 255),
                    11..=30 => lit(LitLevel::LIT, 220),
                    31..=55 => lit(LitLevel::LOW, 200),
                    _ => lit(LitLevel::DARK, 180),
                };
                if x < 33 && y < 33 && z == 0 {
                    t = TileLight::new(LitLevel::MEMORIZED, 128);
                }
                if x >= 120 && y >= 120 {
                    t = TileLight::BLANK;
                }
                if z == 1 && (x % 12 == 2 || y % 12 == 2) {
                    t = t.occluding(true);
                }
                g.set(x, y, z, t);
            }
        }
    }
    (map, g)
}
