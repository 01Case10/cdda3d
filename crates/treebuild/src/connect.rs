//! Phase 2d: per-voxel cardinal connection masks over a
//! [`VoxelWorld`], and the virtual-material assignment that carries
//! multitile layer selection into the RT renderer.
//!
//! Selection design — virtual materials, not a shader change. The
//! RT shader already resolves a hit's texture layers through
//! `mat_layers[material_id]` (top, side, bottom). A connected wall
//! voxel needs a different *side* layer than an isolated one, so
//! instead of growing the side-table with a per-node override (and
//! touching the WGSL + the 2c parity machinery), each
//! `(base material, mask)` pair that actually occurs in a world is
//! allocated a fresh *virtual* material id whose `mat_layers` entry
//! the engine's spec builder fills with the variant side layer (see
//! `engine::textures::build_texture_spec_multitile`). The voxel's
//! side-table entry simply carries the virtual id; the shader,
//! the CPU oracle and the tree are untouched, and the same virtual
//! ids can feed the mesher path (Phase 2f) later.
//!
//! Semantics: a neighbor connects under a caller-supplied predicate
//! over the two cells. The fake-data rule is [`same_material`]
//! (Phase 2d plan: fake data uses same-material adjacency and says
//! so). Real scenes will pass a connect-group predicate built from
//! TileDefs — the game's own rule (`connects_to` groups, Phase 3
//! export; the group test itself already exists as
//! `voxelizer::multitile::neighbors_connect`) — without changing
//! anything here.
//!
//! Masks are computed in the XY plane at the voxel's own z: grid
//! north is -y. A full-height wall stack therefore gets the same
//! mask at every block of the stack, because its neighbors are
//! full-height too.

use std::collections::BTreeMap;

use mesher::connect::mask_from_connects;

use crate::world::{VoxelCell, VoxelWorld};

/// The fake-data connection rule: a neighbor connects when it
/// carries the same material id as the voxel itself.
pub fn same_material(a: &VoxelCell, b: &VoxelCell) -> bool {
    a.material_id == b.material_id
}

/// Cardinal connection mask of the voxel at `pos`: bit N/E/S/W set
/// when the neighbor at the same z exists and `connects(self,
/// neighbor)` holds. An absent voxel has mask 0.
pub fn connection_mask(
    world: &VoxelWorld,
    pos: [i32; 3],
    connects: impl Fn(&VoxelCell, &VoxelCell) -> bool,
) -> u8 {
    let Some(cell) = world.get(pos) else {
        return 0;
    };
    let test = |d: [i32; 3]| {
        world
            .get([pos[0] + d[0], pos[1] + d[1], pos[2]])
            .is_some_and(|n| connects(cell, n))
    };
    mask_from_connects(
        test([0, -1, 0]),
        test([1, 0, 0]),
        test([0, 1, 0]),
        test([-1, 0, 0]),
    )
}

/// One allocated virtual material: voxels whose base material is
/// `base_id` and whose connection mask is `mask` are rewritten to
/// `virtual_id`, and the texture spec must carry a variant entry
/// (base top/bottom, mask-selected side) at `virtual_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariantAssignment {
    pub base_id: u32,
    pub mask: u8,
    pub virtual_id: u32,
}

/// Rewrite every multitile voxel in `world` to a virtual material
/// id for its (base, mask) pair, allocating ids deterministically:
/// pairs are sorted by (base, mask) and numbered from
/// `max_material_id + 1` upward, so the same world always produces
/// the same assignment. Returns the assignments in allocation
/// order. Non-multitile voxels keep their base id untouched.
///
/// All masks are computed *before* any id is rewritten — rewriting
/// first would corrupt the same-material test for later voxels.
pub fn assign_variants(
    world: &mut VoxelWorld,
    is_multitile: impl Fn(u32) -> bool,
    connects: impl Fn(&VoxelCell, &VoxelCell) -> bool,
) -> Vec<VariantAssignment> {
    // Pass 1: masks for multitile voxels, from the unmodified world.
    let mut masks: Vec<([i32; 3], u32, u8)> = Vec::new();
    let mut max_id = 0u32;
    for (pos, cell) in world.iter() {
        max_id = max_id.max(cell.material_id);
        if is_multitile(cell.material_id) {
            masks.push((
                *pos,
                cell.material_id,
                connection_mask(world, *pos, &connects),
            ));
        }
    }
    // Allocate one virtual id per distinct (base, mask) pair.
    let mut pairs: Vec<(u32, u8)> = masks.iter().map(|(_, b, m)| (*b, *m)).collect();
    pairs.sort_unstable();
    pairs.dedup();
    let assignments: Vec<VariantAssignment> = pairs
        .iter()
        .enumerate()
        .map(|(i, &(base_id, mask))| VariantAssignment {
            base_id,
            mask,
            virtual_id: max_id + 1 + i as u32,
        })
        .collect();
    let by_pair: BTreeMap<(u32, u8), u32> = assignments
        .iter()
        .map(|a| ((a.base_id, a.mask), a.virtual_id))
        .collect();
    // Pass 2: rewrite.
    for (pos, base, mask) in masks {
        let virtual_id = by_pair[&(base, mask)];
        if let Some(cell) = world.get(pos).copied() {
            let mut cell = cell;
            cell.material_id = virtual_id;
            world.set(pos, cell);
        }
    }
    assignments
}
