//! Stage F part 2 (border height-blending): world-build
//! assignment of ground-border blend variants between *macro*
//! materials (Stage F part 1's world-space ground textures —
//! today grass ↔ dirt).
//!
//! ## Why this is not the shore/multitile variant pattern
//!
//! Phase 2d multitile and the shore slice assign a per-tile
//! *texture* per neighbor configuration, because those textures
//! are per-tile (32px, in-tile UVs). Macro textures are
//! world-anchored (128px over `span_tiles`, sampled by world
//! position in both renderers), so a pre-blended per-tile variant
//! cannot exist: the blend at a border pixel depends on both
//! materials' macro color *and* stored height at that exact world
//! position. The blend therefore runs in the fragment shaders,
//! per pixel, and this module only decides *which* tiles border
//! *which* neighbor material across *which* edge — the assignment
//! problem, which is the part the variant pattern does solve:
//!
//! * each distinct `(base, neighbor, dir)` triple that occurs is
//!   allocated a virtual material id (the Phase 2d allocation
//!   rule: sorted triples, numbered from the world's max id + 1,
//!   so the same world always produces the same assignment);
//! * the RT path rewrites border cells to the virtual ids, and
//!   the texture spec builder (`textures::
//!   build_texture_spec_variants`) gives each virtual id its base
//!   material's layers + macro entry plus a *blend entry*
//!   (`TextureSpec::material_blends`): the neighbor's macro color
//!   and height layers and the border direction. The RT shader
//!   runs the height decision from those entries.
//! * the raster path keeps the block ids; `scene_prep` stamps the
//!   same information into the top face's vertex layer word
//!   (`mesher::mesh::encode_macro_blend_layer`) from the same
//!   assignments, and the raster shader runs the identical
//!   decision.
//!
//! Both forms consume the assignments computed here from the same
//! filtered `ChunkMap` both renderers derive from, so the two
//! paths cannot disagree about which tile blends toward what.
//!
//! ## The decision (Stage D semantics)
//!
//! The shaders mirror `voxelizer::quilt::height_blend` with
//! a = the tile's own material, b = the neighbor: each side's
//! effective height is its stored height plus a weight bias,
//! `eff_a = h_a + (1 - w) * 2`, `eff_b = h_b + w * 2`, and the
//! higher wins the pixel outright (ties to a — the tile's own
//! material). The weight is spatial: `w = 0.5` at the border,
//! falling linearly to 0 at [`BLEND_HALF_BAND_TILES`] into the
//! tile, so deep in its own tile a material is exact (w = 0 makes
//! the bias gap of 2 unbeatable), and at the border the raw
//! stored heights decide — high patches of either material punch
//! across the border exactly where their height is greater. The
//! CPU mirrors of the shader math ([`blend_weight`],
//! [`neighbor_wins`]) live here so the gate tests the shipped
//! formula, not a paraphrase. Heights are compared in *sampled*
//! space: the RT macro array is raw (byte/255, exactly the CPU
//! reference), the raster array is sRGB-decoded — a monotone map,
//! so every decision agrees except exact-height ties at exactly
//! w = 0.5 (the border line itself, where each side keeping its
//! own material is the correct picture anyway).
//!
//! ## Scope limits (deliberate)
//!
//! * Macro materials only, and only pairs with the *same*
//!   `span_tiles` (both shaders sample the neighbor at the own
//!   material's world UV; a different span would need a second
//!   span in the packed data). All voxelizer macros are span 4.
//! * One border per tile: a tile bordering two different macro
//!   materials (a corner) blends across the first border in
//!   [`BLEND_DIRS`] order only; the other border stays hard.
//! * Water borders are excluded by construction: water carries
//!   no macro (its edges are owned by the shore variants), and a
//!   macro tile beside water finds no macro neighbor to blend
//!   with — no double-blending.
//! * Same-z horizontal neighbors only: the blend is a top-face
//!   phenomenon between ground tiles on one level.
//! * RT at range: the ray traversal's existing LOD shades
//!   coarser tree nodes with the node material at distance, so
//!   a far border progressively shades with the node's
//!   (unblended) material — the same approximation the
//!   renderer already applies to every texture decision at LOD
//!   nodes. Leaf-level hits blend wherever the traversal
//!   resolves them (the blend gate frames its camera
//!   accordingly).

use std::collections::BTreeMap;

use mesher::chunk::ChunkMap;
use mesher::grid::VoxelSampler as _;
use treebuild::world::VoxelWorld;

/// Half-width of the blend band, in tiles: the weight falls from
/// 0.5 at the border to 0 this far into the tile. 0.5 keeps the
/// whole band inside the two border tiles, so non-border tiles
/// are untouched by construction. Mirrored as `BLEND_HALF` in
/// both fragment shaders (raster `render.rs`, RT
/// `treebuild::render`) — keep the three in sync.
pub const BLEND_HALF_BAND_TILES: f32 = 0.5;

/// Cardinal directions a blend border can face, as
/// `(dx, dy)` grid steps. The index is the direction value both
/// shaders decode: it matches the mesher's `FACE_DIRS` cardinal
/// indices (0 +X, 1 −X, 2 +Y/south, 3 −Y/north). Scan order is
/// the corner tie-break rule (first match wins).
pub const BLEND_DIRS: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];

/// The Stage D weight at `dist` tiles inside the tile's own
/// territory from the border (0 at the border). CPU mirror of
/// the shader formula.
pub fn blend_weight(dist_into_own: f32) -> f32 {
    (0.5 - dist_into_own / (2.0 * BLEND_HALF_BAND_TILES)).clamp(0.0, 1.0)
}

/// The Stage D decision, CPU mirror of both shaders: does the
/// neighbor material win this pixel? `h_own` / `h_nbr` are the
/// stored heights (0..=1) sampled at the pixel's world position;
/// ties go to the tile's own material, exactly like
/// `voxelizer::quilt::height_blend`'s ties-to-a.
pub fn neighbor_wins(h_own: f32, h_nbr: f32, weight: f32) -> bool {
    let eff_own = h_own + (1.0 - weight) * 2.0;
    let eff_nbr = h_nbr + weight * 2.0;
    eff_nbr > eff_own
}

/// One allocated blend variant: tiles whose base material is
/// `base_id` and whose chosen border (see [`blend_neighbor_at`])
/// faces `neighbor_id` across direction `dir` blend under the
/// texture/blend entries at `virtual_id`. Mirrors
/// `crate::shore::ShoreAssignment` in shape, with the neighbor
/// material and direction in place of the mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlendAssignment {
    pub base_id: u32,
    pub neighbor_id: u32,
    pub dir: u8,
    pub virtual_id: u32,
}

/// The border choice for the cell at `pos`: the first direction
/// in [`BLEND_DIRS`] order whose same-z neighbor is a *different*
/// material with a macro of the *same* span. `get` resolves a
/// cell's material id (0 for air/absent); `macro_span` resolves a
/// material id to its macro span (`None` = no macro = never
/// blends, which is what keeps water and flat materials out).
/// Shared by both assignment forms and the raster stamper, so
/// the choice can never be computed two ways.
pub fn blend_neighbor_at(
    get: &dyn Fn(i32, i32, i32) -> u32,
    pos: [i32; 3],
    macro_span: &dyn Fn(u32) -> Option<u32>,
) -> Option<(u32, u8)> {
    let own = get(pos[0], pos[1], pos[2]);
    let span = macro_span(own)?;
    for (di, d) in BLEND_DIRS.iter().enumerate() {
        let n = get(pos[0] + d[0], pos[1] + d[1], pos[2]);
        if n != own && macro_span(n) == Some(span) {
            return Some((n, di as u8));
        }
    }
    None
}

/// Allocate virtual ids for a set of `(base, neighbor, dir)`
/// triples: sorted and numbered from `max_id + 1` upward (the
/// Phase 2d allocation rule).
fn allocate(triples: &[(u32, u32, u8)], max_id: u32) -> Vec<BlendAssignment> {
    let mut triples: Vec<(u32, u32, u8)> = triples.to_vec();
    triples.sort_unstable();
    triples.dedup();
    triples
        .iter()
        .enumerate()
        .map(|(i, &(base_id, neighbor_id, dir))| BlendAssignment {
            base_id,
            neighbor_id,
            dir,
            virtual_id: max_id + 1 + i as u32,
        })
        .collect()
}

/// The blend assignments for a `ChunkMap`: one virtual id per
/// distinct `(base, neighbor, dir)` triple among its macro
/// blocks. Tiles with no blendable border get no assignment and
/// keep their plain macro top.
pub fn blend_assignments_for_map(
    map: &ChunkMap,
    macro_span: &dyn Fn(u32) -> Option<u32>,
) -> Vec<BlendAssignment> {
    let get = |x: i32, y: i32, z: i32| map.block_at(x, y, z).id;
    let mut triples: Vec<(u32, u32, u8)> = Vec::new();
    let mut max_id = 0u32;
    for chunk in map.chunks.values() {
        let origin = chunk.origin();
        for z in 0..chunk.grid.size[2] as i32 {
            for y in 0..chunk.grid.size[1] as i32 {
                for x in 0..chunk.grid.size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    max_id = max_id.max(b.id);
                    if b.is_air() {
                        continue;
                    }
                    let pos = [origin[0] + x, origin[1] + y, origin[2] + z];
                    if let Some((nbr, dir)) = blend_neighbor_at(&get, pos, macro_span) {
                        triples.push((b.id, nbr, dir));
                    }
                }
            }
        }
    }
    allocate(&triples, max_id)
}

/// The RT form: compute blend borders over a `VoxelWorld`,
/// allocate virtual ids the same way, and rewrite the border
/// cells to them. Returns the assignments the spec builder must
/// emit blend entries for. All borders are computed before any
/// id is rewritten (rewriting first would corrupt the neighbor
/// test for later cells — the Phase 2d rule). Run this AFTER the
/// shore preparation when both are used, so blend virtual ids
/// allocate above the shore ones (shore cells carry no macro, so
/// they are never blend bases or neighbors either way).
pub fn assign_blend_variants(
    world: &mut VoxelWorld,
    macro_span: &dyn Fn(u32) -> Option<u32>,
) -> Vec<BlendAssignment> {
    let get = |x: i32, y: i32, z: i32| world.get([x, y, z]).map_or(0, |c| c.material_id);
    // Pass 1: borders for macro cells, from the unmodified world.
    let mut cells: Vec<([i32; 3], u32, u32, u8)> = Vec::new();
    let mut max_id = 0u32;
    for (pos, cell) in world.iter() {
        max_id = max_id.max(cell.material_id);
        if let Some((nbr, dir)) = blend_neighbor_at(&get, *pos, macro_span) {
            cells.push((*pos, cell.material_id, nbr, dir));
        }
    }
    let triples: Vec<(u32, u32, u8)> = cells.iter().map(|&(_, b, n, d)| (b, n, d)).collect();
    let assignments = allocate(&triples, max_id);
    let by_triple: BTreeMap<(u32, u32, u8), u32> = assignments
        .iter()
        .map(|a| ((a.base_id, a.neighbor_id, a.dir), a.virtual_id))
        .collect();
    // Pass 2: rewrite.
    for (pos, base, nbr, dir) in cells {
        if let Some(cell) = world.get(pos).copied() {
            let mut cell = cell;
            cell.material_id = by_triple[&(base, nbr, dir)];
            world.set(pos, cell);
        }
    }
    assignments
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesher::block::{Block, Pass, TILE_GROUND};
    use mesher::scale::WorldScale;

    /// Fake macro spans: grass (TILE_GROUND) and a dirt stand-in
    /// (id 10) at span 4; a third macro id (11) at span 2 (never
    /// blends — span mismatch); everything else no macro.
    fn span_of(id: u32) -> Option<u32> {
        match id {
            TILE_GROUND | 10 => Some(4),
            11 => Some(2),
            _ => None,
        }
    }

    const DIRT: u32 = 10;

    fn block(id: u32) -> Block {
        Block {
            id,
            color: [1.0, 1.0, 1.0, 1.0],
            pass: Pass::Opaque,
            tex_layers: None,
        }
    }

    /// A 6x1 strip at z 0: grass x 0..3, dirt x 3..6, in one chunk.
    fn strip_map() -> ChunkMap {
        let scale = WorldScale::default();
        let mut map = ChunkMap::new(scale);
        map.add_chunk([0, 0], 1);
        for x in 0..6 {
            let id = if x < 3 { TILE_GROUND } else { DIRT };
            map.set_global_block(x, 0, 0, block(id));
        }
        map
    }

    #[test]
    fn weight_endpoints_and_midpoint() {
        assert_eq!(blend_weight(0.0), 0.5, "border: raw heights decide");
        assert_eq!(blend_weight(BLEND_HALF_BAND_TILES), 0.0);
        assert_eq!(blend_weight(10.0), 0.0, "deep in own tile: exact");
        // Endpoints of the Stage D composite: weight 0 never lets
        // the neighbor win, however high; at 0.5 the raw height
        // decides.
        assert!(!neighbor_wins(0.0, 1.0, 0.0));
        assert!(neighbor_wins(0.0, 1.0, 0.5));
        assert!(!neighbor_wins(1.0, 0.0, 0.5));
        assert!(!neighbor_wins(0.5, 0.5, 0.5), "ties go to the own material");
    }

    #[test]
    fn strip_assignments_match_between_map_and_voxel_world() {
        let map = strip_map();
        let assignments = blend_assignments_for_map(&map, &span_of);
        // Two triples: grass facing dirt across +X (dir 0), dirt
        // facing grass across -X (dir 1). Max id is 10, so the
        // virtual ids are 11 and 12 in sorted triple order.
        assert_eq!(
            assignments,
            vec![
                BlendAssignment {
                    base_id: TILE_GROUND,
                    neighbor_id: DIRT,
                    dir: 0,
                    virtual_id: 11,
                },
                BlendAssignment {
                    base_id: DIRT,
                    neighbor_id: TILE_GROUND,
                    dir: 1,
                    virtual_id: 12,
                },
            ]
        );

        let mut vw = VoxelWorld::from_chunk_map(&map);
        let vw_assignments = assign_blend_variants(&mut vw, &span_of);
        assert_eq!(
            assignments, vw_assignments,
            "both world forms must allocate identical assignments"
        );
        // The two border cells were rewritten; interiors keep
        // their base ids.
        assert_eq!(vw.get([2, 0, 0]).unwrap().material_id, 11);
        assert_eq!(vw.get([3, 0, 0]).unwrap().material_id, 12);
        assert_eq!(vw.get([0, 0, 0]).unwrap().material_id, TILE_GROUND);
        assert_eq!(vw.get([5, 0, 0]).unwrap().material_id, DIRT);
    }

    #[test]
    fn span_mismatch_and_non_macro_never_blend() {
        let scale = WorldScale::default();
        let mut map = ChunkMap::new(scale);
        map.add_chunk([0, 0], 1);
        // grass | span-2 macro | water-ish flat id: no pair shares
        // a span, and the flat id has no macro at all.
        map.set_global_block(0, 0, 0, block(TILE_GROUND));
        map.set_global_block(1, 0, 0, block(11));
        map.set_global_block(2, 0, 0, block(8));
        assert!(blend_assignments_for_map(&map, &span_of).is_empty());
        // A uniform field has no borders.
        let mut flat = ChunkMap::new(scale);
        flat.add_chunk([0, 0], 1);
        for x in 0..3 {
            flat.set_global_block(x, 0, 0, block(TILE_GROUND));
        }
        assert!(blend_assignments_for_map(&flat, &span_of).is_empty());
    }
}
