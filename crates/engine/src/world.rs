//! Fake-data worlds for Phase 2: the procedural stub house and the
//! full-bubble data scene used as the 2.13 data-level proxy.
//!
//! Tile ids come from `mesher::block` (the Phase 2 fake-data set).
//! Game-z mapping: a chunk column's level `i` is game z `min_game_z + i`.

use std::time::Instant;

use mesher::block::*;
use mesher::chunk::{Chunk, ChunkMap, CHUNK_H, CHUNK_W};
use mesher::mesh::PassMeshes;
use mesher::scale::WorldScale;

use crate::overlay::MeshStats;

/// The fake house occupies one chunk column at game z 0 (floor/walls)
/// and z 1 (roof slabs).
pub const HOUSE_MIN_GAME_Z: i32 = 0;
pub const HOUSE_LEVELS: usize = 2;

/// Build the stub house: one 12x12 chunk — ground + wood floor, wall
/// perimeter, a door in the south wall, windows in the north/east/west
/// walls, and a full roof-slab layer one z-level up.
pub fn build_fake_house(scale: WorldScale) -> ChunkMap {
    let mut map = ChunkMap::new(scale);
    {
        let chunk = map.add_chunk([0, 0], HOUSE_LEVELS);
        // Level 0: ground everywhere, wood floor inside.
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
        // Level 0: wall perimeter (full height).
        for i in 0..CHUNK_W {
            chunk.set_tile(i, 0, 0, TILE_WALL, scale);
            chunk.set_tile(i, CHUNK_H - 1, 0, TILE_WALL, scale);
            chunk.set_tile(0, i, 0, TILE_WALL, scale);
            chunk.set_tile(CHUNK_W - 1, i, 0, TILE_WALL, scale);
        }
        // Door in the south wall, windows elsewhere.
        chunk.set_tile(6, CHUNK_H - 1, 0, TILE_DOOR, scale);
        chunk.set_tile(3, 0, 0, TILE_WINDOW, scale);
        chunk.set_tile(0, 6, 0, TILE_WINDOW, scale);
        chunk.set_tile(CHUNK_W - 1, 6, 0, TILE_WINDOW, scale);
        // Level 1: roof slabs across the whole footprint.
        for y in 0..CHUNK_H {
            for x in 0..CHUNK_W {
                chunk.set_tile(x, y, 1, TILE_ROOF, scale);
            }
        }
    }
    map.clear_dirty();
    map
}

/// The sample scene (Joey's request, 2026-11-15): a 24x24-tile
/// yard — 2x2 chunk columns, same z layout as the house — with a
/// house in the north-west, and open ground outside it: a pond,
/// scattered trees, and a chain-link fence run. Built for looking
/// at, not gating: monsters inside the room and outside on the
/// grass come from `BillboardScene::sample_crowd`, whose
/// placements are written against these coordinates:
///
/// - house footprint x 3..=10, y 3..=9 (walls), wood floor
///   x 4..=9, y 4..=8, door in the south wall at (6, 9), windows
///   at (5, 3), (3, 6), (10, 6); roof slabs at level 1 over the
///   footprint only (a cutaway at z 0 opens the room).
/// - pond x 16..=19, y 14..=17; fence along y = 13, x 2..=8;
///   trees at (14, 4), (19, 7), (5, 15), (21, 17), (13, 20),
///   (18, 20).
pub fn build_sample_scene(scale: WorldScale) -> ChunkMap {
    let mut map = ChunkMap::new(scale);
    for cy in 0..2 {
        for cx in 0..2 {
            map.add_chunk([cx, cy], HOUSE_LEVELS);
        }
    }
    let set = |map: &mut ChunkMap, x: usize, y: usize, z: usize, tile: u32| {
        let coord = [
            (x / CHUNK_W) as i32,
            (y / CHUNK_H) as i32,
        ];
        let chunk = map.chunks.get_mut(&coord).expect("sample chunk");
        chunk.set_tile(x % CHUNK_W, y % CHUNK_H, z, tile, scale);
    };
    // Grass everywhere at level 0.
    for y in 0..24 {
        for x in 0..24 {
            set(&mut map, x, y, 0, TILE_GROUND);
        }
    }
    // House: wood floor, wall perimeter, door + windows, roof.
    for y in 4..=8 {
        for x in 4..=9 {
            set(&mut map, x, y, 0, TILE_FLOOR_WOOD);
        }
    }
    for x in 3..=10 {
        set(&mut map, x, 3, 0, TILE_WALL);
        set(&mut map, x, 9, 0, TILE_WALL);
    }
    for y in 3..=9 {
        set(&mut map, 3, y, 0, TILE_WALL);
        set(&mut map, 10, y, 0, TILE_WALL);
    }
    set(&mut map, 6, 9, 0, TILE_DOOR);
    set(&mut map, 5, 3, 0, TILE_WINDOW);
    set(&mut map, 3, 6, 0, TILE_WINDOW);
    set(&mut map, 10, 6, 0, TILE_WINDOW);
    for y in 3..=9 {
        for x in 3..=10 {
            set(&mut map, x, y, 1, TILE_ROOF);
        }
    }
    // The yard: pond, fence run, trees.
    for y in 14..=17 {
        for x in 16..=19 {
            set(&mut map, x, y, 0, TILE_WATER);
        }
    }
    for x in 2..=8 {
        set(&mut map, x, 13, 0, TILE_FENCE);
    }
    for (x, y) in [(14, 4), (19, 7), (5, 15), (21, 17), (13, 20), (18, 20)] {
        set(&mut map, x, y, 0, TILE_FOLIAGE);
    }
    map.clear_dirty();
    map
}

/// The cutaway applied at data level: a copy of `map` keeping only
/// blocks whose game z-level is in `visible_levels`. Shared by the
/// raster mesher and the Phase 2c tree builder so both renderers see
/// exactly the same world for a given cutaway (gate 2c.6).
pub fn filtered_map(map: &ChunkMap, min_game_z: i32, visible_levels: &[i32]) -> ChunkMap {
    let scale = map.scale;
    let per = scale.blocks_per_z_level as i32;
    let mut filtered = ChunkMap::new(scale);
    for (coord, chunk) in &map.chunks {
        let mut fc = Chunk::new(*coord, chunk.levels, scale);
        for z in 0..chunk.grid.size[2] {
            let game_z = min_game_z + (z as i32) / per;
            if !visible_levels.contains(&game_z) {
                continue;
            }
            for y in 0..chunk.grid.size[1] {
                for x in 0..chunk.grid.size[0] {
                    let b = chunk.grid.get(x as i32, y as i32, z as i32);
                    if !b.is_air() {
                        fc.grid.set(x, y, z, b);
                    }
                }
            }
        }
        fc.dirty = false;
        filtered.chunks.insert(*coord, fc);
    }
    filtered
}

/// Mesh every chunk in `map`, keeping only blocks whose game z-level is
/// in `visible_levels` (the cutaway applied at data level, so windowed
/// and headless paths render exactly the selected layers). Returns the
/// combined meshes plus stats.
pub fn mesh_world(
    map: &ChunkMap,
    min_game_z: i32,
    visible_levels: &[i32],
) -> (PassMeshes, MeshStats) {
    let filtered = filtered_map(map, min_game_z, visible_levels);
    let mut combined = PassMeshes::default();
    let mut n_chunks = 0;
    for coord in filtered.chunks.keys().copied().collect::<Vec<_>>() {
        if let Some(pm) = filtered.mesh_chunk(coord, true) {
            for (dst, src) in combined.meshes.iter_mut().zip(pm.meshes.iter()) {
                dst.append(src);
            }
            n_chunks += 1;
        }
    }
    let stats = MeshStats::from_pass_meshes(n_chunks, &combined);
    (combined, stats)
}

/// Convenience: the fake house meshed with the given visible levels.
pub fn mesh_fake_house(visible_levels: &[i32]) -> (ChunkMap, PassMeshes, MeshStats) {
    let map = build_fake_house(WorldScale::default());
    let (meshes, stats) = mesh_world(&map, HOUSE_MIN_GAME_Z, visible_levels);
    (map, meshes, stats)
}

/// Data-level report for the full-bubble scene (2.13 proxy): the real
/// stored footprint — 132x132 tiles (11x11 chunk columns) x 3 z-levels —
/// built and fully meshed on CPU. This is NOT a frame-time measurement;
/// it proves the data pipeline handles the real scene size and reports
/// what a frame would have to draw.
#[derive(Debug, Clone)]
pub struct BubbleReport {
    pub chunks: usize,
    pub tiles_stored: usize,
    pub levels: usize,
    pub non_air_blocks: usize,
    pub quads: usize,
    pub vertices: usize,
    pub mesh_ms: f64,
}

pub const BUBBLE_MIN_GAME_Z: i32 = -1;
pub const BUBBLE_LEVELS: usize = 3;

pub fn build_bubble_scene(scale: WorldScale) -> ChunkMap {
    let mut map = ChunkMap::new(scale);
    for cy in 0..11 {
        for cx in 0..11 {
            let chunk = map.add_chunk([cx, cy], BUBBLE_LEVELS);
            // Bottom level: ground everywhere.
            for y in 0..CHUNK_H {
                for x in 0..CHUNK_W {
                    chunk.set_tile(x, y, 0, TILE_GROUND, scale);
                }
            }
            // Middle level: a small house outline per chunk plus trees,
            // deterministic so reports are reproducible.
            for i in 2..10 {
                chunk.set_tile(i, 2, 1, TILE_WALL, scale);
                chunk.set_tile(i, 9, 1, TILE_WALL, scale);
                chunk.set_tile(2, i, 1, TILE_WALL, scale);
                chunk.set_tile(9, i, 1, TILE_WALL, scale);
            }
            chunk.set_tile(5, 9, 1, TILE_DOOR, scale);
            chunk.set_tile(6, 2, 1, TILE_WINDOW, scale);
            if (cx + cy) % 2 == 0 {
                chunk.set_tile(0, 0, 1, TILE_FOLIAGE, scale);
                chunk.set_tile(11, 11, 1, TILE_FENCE, scale);
            }
        }
    }
    map.clear_dirty();
    map
}

pub fn bubble_report() -> BubbleReport {
    let scale = WorldScale::default();
    let map = build_bubble_scene(scale);
    let non_air: usize = map.chunks.values().map(|c| c.grid.count_non_air()).sum();
    let visible: Vec<i32> = (BUBBLE_MIN_GAME_Z..BUBBLE_MIN_GAME_Z + BUBBLE_LEVELS as i32).collect();
    let start = Instant::now();
    let (meshes, stats) = mesh_world(&map, BUBBLE_MIN_GAME_Z, &visible);
    let mesh_ms = start.elapsed().as_secs_f64() * 1000.0;
    BubbleReport {
        chunks: map.chunks.len(),
        tiles_stored: 132 * 132,
        levels: BUBBLE_LEVELS,
        non_air_blocks: non_air,
        quads: meshes.total_quads(),
        vertices: stats.vertices,
        mesh_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesher::block::block_for_tile;

    fn count_block(map: &ChunkMap, tile_id: u32) -> usize {
        let want = block_for_tile(tile_id);
        let mut n = 0;
        for c in map.chunks.values() {
            for z in 0..c.grid.size[2] {
                for y in 0..c.grid.size[1] {
                    for x in 0..c.grid.size[0] {
                        if c.grid.get(x as i32, y as i32, z as i32) == want {
                            n += 1;
                        }
                    }
                }
            }
        }
        n
    }

    #[test]
    fn fake_house_has_door_windows_walls_and_roof() {
        let map = build_fake_house(WorldScale::default());
        let per = WorldScale::default().blocks_per_z_level as usize;
        // Door/window tiles are full-height: one tile = `per` blocks.
        assert_eq!(count_block(&map, TILE_DOOR), per);
        assert_eq!(count_block(&map, TILE_WINDOW), 3 * per);
        assert!(count_block(&map, TILE_WALL) > 30 * per);
        // Roof: 12x12 tiles x 1 slab block.
        assert_eq!(count_block(&map, TILE_ROOF), 12 * 12);
        assert!(count_block(&map, TILE_FLOOR_WOOD) > 0);
    }

    #[test]
    fn fake_house_meshes_non_empty_and_cutaway_reduces_it() {
        let (_m, full, full_stats) = mesh_fake_house(&[0, 1]);
        assert!(full.total_quads() > 0);
        assert!(full_stats.chunks == 1);
        // Hiding the roof level must remove geometry.
        let (_m2, no_roof, _) = mesh_fake_house(&[0]);
        assert!(no_roof.total_quads() < full.total_quads());
        // Showing only the roof level keeps some geometry, less than full.
        let (_m3, roof_only, _) = mesh_fake_house(&[1]);
        assert!(roof_only.total_quads() > 0);
        assert!(roof_only.total_quads() < full.total_quads());
    }

    #[test]
    fn sample_scene_has_house_pond_fence_and_trees() {
        let map = build_sample_scene(WorldScale::default());
        assert_eq!(map.chunks.len(), 4);
        let per = WorldScale::default().blocks_per_z_level as usize;
        assert_eq!(count_block(&map, TILE_DOOR), per);
        assert_eq!(count_block(&map, TILE_WINDOW), 3 * per);
        // Pond: 4x4 tiles of water (floor-slab shape, 1 block each
        // at the tile's base like ground).
        assert!(count_block(&map, TILE_WATER) >= 16);
        assert_eq!(count_block(&map, TILE_FOLIAGE), 6 * per);
        // Roof only over the 8x7 house footprint.
        assert_eq!(count_block(&map, TILE_ROOF), 8 * 7);
        let (meshes, stats) = mesh_world(&map, HOUSE_MIN_GAME_Z, &[0, 1]);
        assert!(meshes.total_quads() > 0 && stats.chunks == 4);
    }

    #[test]
    fn bubble_scene_is_real_footprint() {
        let r = bubble_report();
        assert_eq!(r.chunks, 121);
        assert_eq!(r.tiles_stored, 132 * 132);
        assert_eq!(r.levels, 3);
        assert!(
            r.non_air_blocks > 132 * 132,
            "ground alone is one block/tile"
        );
        assert!(r.quads > 0 && r.vertices > 0);
    }
}
