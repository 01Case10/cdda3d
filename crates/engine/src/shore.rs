//! Stage F part 2 (§10.1 engine half): world-build assignment of
//! shoreline top-face variants.
//!
//! The voxelizer emits, for the primary water material, one shore
//! texture per 8-neighbor water mask (`materials::Material::
//! shore_layers`). At world build, a water tile whose mask is not
//! all-water (0xFF — an interior tile, plain water by rule) is
//! assigned the shore variant for its mask. Assignment follows the
//! Phase 2d virtual-material design (`treebuild::connect`): each
//! distinct `(base material, mask)` pair that actually occurs is
//! allocated a virtual material id, and the texture spec builder
//! (`textures::build_texture_spec_shore`) fills that id's entry
//! with the shore top layer and the base side/bottom layers. The
//! RT path rewrites its `VoxelWorld` cells to the virtual ids;
//! the raster path keeps the block ids and stamps the virtual
//! entry's compact top layer per block (`scene_prep`) — both
//! consume the SAME assignments computed here, from the same
//! filtered `ChunkMap` both renderers derive from, so the two
//! paths cannot disagree about which tile gets which variant.
//!
//! Masks are computed through `mesher::connect::shore_mask_for`
//! — the canonical bit order the voxelizer's generator also uses.
//! Land tiles are never assigned anything: the dirt bank ring
//! lives inside the shore texture itself (§10.1), so there is no
//! land-side mechanism here.

use std::collections::BTreeMap;

use mesher::chunk::ChunkMap;
use mesher::grid::VoxelSampler as _;
use treebuild::world::VoxelWorld;

/// One allocated shore variant: voxels/blocks whose base material
/// is `base_id` and whose 8-neighbor water mask is `mask` use the
/// texture entry at `virtual_id`. Mirrors
/// `treebuild::connect::VariantAssignment` in shape, but `mask`
/// here is the 8-neighbor shore mask, not the cardinal
/// connection mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShoreAssignment {
    pub base_id: u32,
    pub mask: u8,
    pub virtual_id: u32,
}

/// Allocate virtual ids for a set of `(base, mask)` pairs:
/// pairs are sorted and numbered from `max_id + 1` upward, so
/// the same world always produces the same assignment (the
/// Phase 2d allocation rule).
fn allocate(pairs: &[(u32, u8)], max_id: u32) -> Vec<ShoreAssignment> {
    let mut pairs: Vec<(u32, u8)> = pairs.to_vec();
    pairs.sort_unstable();
    pairs.dedup();
    pairs
        .iter()
        .enumerate()
        .map(|(i, &(base_id, mask))| ShoreAssignment {
            base_id,
            mask,
            virtual_id: max_id + 1 + i as u32,
        })
        .collect()
}

/// Shore masks for every water block of a `ChunkMap`, keyed by
/// global block position. `is_water` decides which block ids are
/// water (the fake-data path: `TILE_WATER`; the material-driven
/// path: ids whose material carries a shore set). Interior
/// blocks (mask 0xFF) are included — callers filter them through
/// `Material::shore_top_layer` / the assignment rule below.
pub fn shore_masks_for_map(
    map: &ChunkMap,
    is_water: &dyn Fn(u32) -> bool,
) -> BTreeMap<[i32; 3], u8> {
    let mut out = BTreeMap::new();
    for chunk in map.chunks.values() {
        let origin = chunk.origin();
        for z in 0..chunk.grid.size[2] as i32 {
            for y in 0..chunk.grid.size[1] as i32 {
                for x in 0..chunk.grid.size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    if b.is_air() || !is_water(b.id) {
                        continue;
                    }
                    let (gx, gy) = (origin[0] + x, origin[1] + y);
                    let mask = mesher::connect::shore_mask_for(
                        |nx, ny| {
                            let n = map.block_at(nx, ny, z);
                            !n.is_air() && is_water(n.id)
                        },
                        gx,
                        gy,
                    );
                    out.insert([gx, gy, origin[2] + z], mask);
                }
            }
        }
    }
    out
}

/// The shore assignments for a `ChunkMap`: one virtual id per
/// distinct `(water base id, mask)` pair among its non-interior
/// water blocks. Interior blocks (mask 0xFF) get no assignment
/// and keep their plain water entry.
pub fn shore_assignments_for_map(
    map: &ChunkMap,
    is_water: &dyn Fn(u32) -> bool,
) -> Vec<ShoreAssignment> {
    let masks = shore_masks_for_map(map, is_water);
    let mut max_id = 0u32;
    for chunk in map.chunks.values() {
        for z in 0..chunk.grid.size[2] {
            for y in 0..chunk.grid.size[1] {
                for x in 0..chunk.grid.size[0] {
                    max_id = max_id.max(chunk.grid.get(x as i32, y as i32, z as i32).id);
                }
            }
        }
    }
    let pairs: Vec<(u32, u8)> = masks
        .iter()
        .filter(|(_, &mask)| mask != mesher::connect::SHORE_MASK_ALL_WATER)
        .map(|(pos, &mask)| (map.block_at(pos[0], pos[1], pos[2]).id, mask))
        .collect();
    allocate(&pairs, max_id)
}

/// The RT form: compute shore masks over a `VoxelWorld`, allocate
/// virtual ids the same way, and rewrite the non-interior water
/// cells to them. Returns the assignments the spec builder must
/// emit variant entries for. All masks are computed before any
/// id is rewritten (rewriting first would corrupt the water test
/// for later cells — the Phase 2d rule). Run this AFTER the
/// Phase 2d multitile preparation when both are used, so shore
/// virtual ids allocate above the multitile ones.
pub fn assign_shore_variants(
    world: &mut VoxelWorld,
    is_water: &dyn Fn(u32) -> bool,
) -> Vec<ShoreAssignment> {
    // Pass 1: masks for water cells, from the unmodified world.
    let mut masks: Vec<([i32; 3], u32, u8)> = Vec::new();
    let mut max_id = 0u32;
    for (pos, cell) in world.iter() {
        max_id = max_id.max(cell.material_id);
        if is_water(cell.material_id) {
            let mask = mesher::connect::shore_mask_for(
                |nx, ny| {
                    world
                        .get([nx, ny, pos[2]])
                        .is_some_and(|n| is_water(n.material_id))
                },
                pos[0],
                pos[1],
            );
            masks.push((*pos, cell.material_id, mask));
        }
    }
    let pairs: Vec<(u32, u8)> = masks
        .iter()
        .filter(|(_, _, mask)| *mask != mesher::connect::SHORE_MASK_ALL_WATER)
        .map(|(_, base, mask)| (*base, *mask))
        .collect();
    let assignments = allocate(&pairs, max_id);
    let by_pair: BTreeMap<(u32, u8), u32> = assignments
        .iter()
        .map(|a| ((a.base_id, a.mask), a.virtual_id))
        .collect();
    // Pass 2: rewrite.
    for (pos, base, mask) in masks {
        if mask == mesher::connect::SHORE_MASK_ALL_WATER {
            continue;
        }
        if let Some(cell) = world.get(pos).copied() {
            let mut cell = cell;
            cell.material_id = by_pair[&(base, mask)];
            world.set(pos, cell);
        }
    }
    assignments
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesher::block::TILE_WATER;
    use mesher::scale::WorldScale;

    fn is_water(id: u32) -> bool {
        id == TILE_WATER
    }

    #[test]
    fn pond_assignments_match_between_map_and_voxel_world() {
        // The shared-builder assertion: the sample scene's pond
        // (4x4 water at x 16..=19, y 14..=17) yields the same
        // (base, mask) pairs and the same per-position masks from
        // the ChunkMap form and the VoxelWorld form.
        let map = crate::world::build_sample_scene(WorldScale::default());
        let filtered = crate::world::filtered_map(&map, crate::world::HOUSE_MIN_GAME_Z, &[0, 1]);
        let map_masks = shore_masks_for_map(&filtered, &is_water);
        // 4x4 pond, one block per tile (water is a
        // floor slab, mesher::tile_shape).
        assert_eq!(map_masks.len(), 16);
        // Corner tile of the pond at (16, 14): water neighbors at
        // E, SE, S only (bits 2, 3, 4).
        assert_eq!(map_masks[&[16, 14, 0]], 0b0001_1100);
        // Center tiles are interior.
        assert_eq!(map_masks[&[17, 15, 0]], 0xFF);
        assert_eq!(map_masks[&[18, 16, 0]], 0xFF);

        let assignments = shore_assignments_for_map(&filtered, &is_water);
        // 12 edge tiles (16 minus 4 interior), each a distinct
        // mask for this square pond? Corners share by symmetry
        // but masks differ by orientation: 4 corners + 4 edge
        // orientations... every edge tile's mask is distinct
        // except symmetric pairs — just assert the set covers
        // corners and straights and excludes the interior.
        assert!(assignments.iter().all(|a| a.base_id == TILE_WATER));
        assert!(assignments.iter().all(|a| a.mask != 0xFF));
        assert!(assignments.len() >= 8, "got {assignments:?}");

        let mut vw = VoxelWorld::from_chunk_map(&filtered);
        let vw_assignments = assign_shore_variants(&mut vw, &is_water);
        assert_eq!(
            assignments, vw_assignments,
            "both world forms must allocate identical assignments"
        );
        // Interior cells keep the base water id; a corner cell
        // was rewritten to its assignment's virtual id.
        let corner_mask = 0b0001_1100u8;
        let corner_virtual = assignments
            .iter()
            .find(|a| a.mask == corner_mask)
            .unwrap()
            .virtual_id;
        assert_eq!(vw.get([16, 14, 0]).unwrap().material_id, corner_virtual);
        assert_eq!(vw.get([17, 15, 0]).unwrap().material_id, TILE_WATER);
    }
}
