//! World -> `Sparse64Tree` builder + the material side-table.
//!
//! The crate's `GpuNode64` carries only an RGB color, so materials ride
//! in a parallel buffer (plan §5 Phase 2c): [`SideEntry`] per flattened
//! node, indexed by the crate's deterministic breadth-first flatten
//! order. The indexing is *proven at build time*, not assumed:
//! `flatten()` emits nodes breadth-first with each node's children
//! contiguous starting at its `child_ptr`, so walking the flat array in
//! order while tracking the next unassigned child index reconstructs
//! every node's voxel bounds. For leaf nodes the reconstructed voxel
//! is looked up in the source world and the flat node's color (which
//! the crate set from the inserted voxel, independently of this walk)
//! must match that voxel's color — a single mismatch fails the build
//! with [`BuildError::SideTableMismatch`]. Gate 2c.2 exercises exactly
//! this check, including a negative (shuffled table) case.

use std::fmt;

use bytemuck::{Pod, Zeroable};
use Sparse_Voxel_64Tree::{GpuNode64, Sparse64Tree};

use crate::world::{VoxelCell, VoxelWorld, FLAG_MEMORIZED, LIT_BLANK, LIT_MEMORIZED};

/// Per-flattened-node material data, parallel to `BuiltTree::flat`.
/// 16 bytes, GPU storage-buffer friendly.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct SideEntry {
    pub material_id: u32,
    pub lit: u32,
    pub flags: u32,
    /// Opacity * 255.
    pub alpha: u32,
}

impl SideEntry {
    pub const EMPTY: SideEntry = SideEntry {
        material_id: 0,
        lit: 0,
        flags: 0,
        alpha: 0,
    };

    fn from_cell(cell: &VoxelCell) -> Self {
        let mut flags = cell.flags;
        if cell.lit == LIT_MEMORIZED {
            flags |= FLAG_MEMORIZED;
        }
        SideEntry {
            material_id: cell.material_id,
            lit: cell.lit as u32,
            flags,
            alpha: (cell.alpha.clamp(0.0, 1.0) * 255.0).round() as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BuildConfig {
    /// Tree depth (edges root -> leaf). `None` = smallest depth whose
    /// `4^depth` span covers the world extent (clamped to 1..=6).
    pub depth: Option<usize>,
    /// Grid coord mapped to tree coord (0,0,0). `None` = per-axis
    /// minimum over the included cells.
    pub origin: Option<[i32; 3]>,
}

#[derive(Debug)]
pub enum BuildError {
    /// A cell falls outside the tree's span at the chosen depth/origin.
    OutOfBounds([i32; 3]),
    /// The flatten-order / side-table reconstruction disagreed with
    /// the crate's flattened output or the source world.
    SideTableMismatch(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::OutOfBounds(p) => {
                write!(f, "treebuild: cell {p:?} outside the tree span")
            }
            BuildError::SideTableMismatch(m) => {
                write!(f, "treebuild: side-table mismatch: {m}")
            }
        }
    }
}

impl std::error::Error for BuildError {}

/// A built tree plus everything derived from it, in flatten order.
pub struct BuiltTree {
    /// The crate tree (kept so `render` can hand it to
    /// `Tree64GpuManager`, which flattens it itself — same code, same
    /// tree, so the GPU buffer order is this `flat` order).
    pub tree: Sparse64Tree,
    /// Flattened GPU nodes parsed from `tree.flatten()`.
    pub flat: Vec<GpuNode64>,
    /// Material side-table, parallel to `flat`.
    pub side: Vec<SideEntry>,
    /// Tree depth (edges root -> leaf); span per axis = `4^depth`.
    pub depth: usize,
    /// Span per axis in voxels.
    pub size: u32,
    /// Grid coord of tree coord (0,0,0).
    pub origin: [i32; 3],
    /// Voxels inserted (== leaf count).
    pub voxel_count: usize,
    /// Never-seen (`LIT_BLANK`) cells excluded at build time (2c.8).
    pub excluded_never_seen: usize,
    /// Per-flat-node voxel bounds `(min tree coord, size)` from the
    /// reconstruction walk — test/debug evidence for gate 2c.2.
    pub node_bounds: Vec<([u32; 3], u32)>,
}

impl BuiltTree {
    pub fn node_count(&self) -> usize {
        self.flat.len()
    }

    pub fn leaf_count(&self) -> usize {
        self.flat
            .iter()
            .filter(|n| n.child_ptr_and_leaf & 0x8000_0000 != 0)
            .count()
    }

    pub fn is_leaf(&self, node_index: usize) -> bool {
        self.flat[node_index].child_ptr_and_leaf & 0x8000_0000 != 0
    }

    /// World (grid) coord of the voxel a leaf node represents.
    pub fn leaf_voxel(&self, node_index: usize) -> Option<[i32; 3]> {
        if !self.is_leaf(node_index) {
            return None;
        }
        let (min, _) = self.node_bounds[node_index];
        Some([
            min[0] as i32 + self.origin[0],
            min[1] as i32 + self.origin[1],
            min[2] as i32 + self.origin[2],
        ])
    }
}

pub fn node_mask(node: &GpuNode64) -> u64 {
    (node.child_mask_low as u64) | ((node.child_mask_high as u64) << 32)
}

pub fn node_child_ptr(node: &GpuNode64) -> usize {
    (node.child_ptr_and_leaf & 0x7fff_ffff) as usize
}

/// Parse the crate's flattened byte vector into nodes (manual LE
/// decode: the byte vector is not 32-byte aligned, so a bytemuck cast
/// of the slice is not available).
pub fn parse_flat(bytes: &[u8]) -> Vec<GpuNode64> {
    assert_eq!(bytes.len() % 32, 0, "GpuNode64 records are 32 bytes");
    bytes
        .chunks_exact(32)
        .map(|c| {
            let u = |o: usize| u32::from_le_bytes(c[o..o + 4].try_into().unwrap());
            let f = |o: usize| f32::from_le_bytes(c[o..o + 4].try_into().unwrap());
            GpuNode64 {
                child_mask_low: u(0),
                child_mask_high: u(4),
                child_ptr_and_leaf: u(8),
                _padding: u(12),
                color: [f(16), f(20), f(24)],
                _padding2: u(28),
            }
        })
        .collect()
}

fn smallest_depth(extent: u32) -> usize {
    let mut d = 1usize;
    while d < 6 && (4u32.pow(d as u32)) < extent.max(1) {
        d += 1;
    }
    d
}

/// Build a tree from `world`. Never-seen cells are excluded; every
/// other non-air cell becomes one leaf at full depth.
pub fn build_tree(world: &VoxelWorld, config: BuildConfig) -> Result<BuiltTree, BuildError> {
    // Collect included cells in a deterministic (sorted) order.
    let mut included: Vec<([i32; 3], VoxelCell)> = Vec::new();
    let mut excluded_never_seen = 0usize;
    for (pos, cell) in world.iter() {
        if cell.lit == LIT_BLANK {
            excluded_never_seen += 1;
        } else {
            included.push((*pos, *cell));
        }
    }
    included.sort_unstable_by_key(|(pos, _)| *pos);

    let origin = config.origin.unwrap_or_else(|| {
        let mut o = [0i32; 3];
        for a in 0..3 {
            o[a] = included.iter().map(|(p, _)| p[a]).min().unwrap_or(0);
        }
        o
    });
    let extent = (0..3)
        .map(|a| {
            included
                .iter()
                .map(|(p, _)| (p[a] - origin[a] + 1).max(0) as u32)
                .max()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    let depth = config.depth.unwrap_or_else(|| smallest_depth(extent));
    let size = 4u32.pow(depth as u32);

    let mut tree = Sparse64Tree::new();
    for (pos, cell) in &included {
        let t = [
            pos[0] - origin[0],
            pos[1] - origin[1],
            pos[2] - origin[2],
        ];
        if t.iter().any(|&v| v < 0 || v as u32 >= size) {
            return Err(BuildError::OutOfBounds(*pos));
        }
        tree.insert(t[0] as u32, t[1] as u32, t[2] as u32, depth, cell.color_u8());
    }

    let flat = parse_flat(&tree.flatten());
    if flat.len() != tree.nodes.len() {
        return Err(BuildError::SideTableMismatch(format!(
            "flatten produced {} nodes, slotmap holds {}",
            flat.len(),
            tree.nodes.len()
        )));
    }
    let (side, node_bounds) = reconstruct_side_table(&flat, world, origin, size)?;

    Ok(BuiltTree {
        voxel_count: included.len(),
        tree,
        flat,
        side,
        depth,
        size,
        origin,
        excluded_never_seen,
        node_bounds,
    })
}

/// Walk the flat array breadth-first, reconstruct each node's voxel
/// bounds, and fill the side-table from the source world. This is the
/// gate 2c.2 mechanism; it is also a public function so tests can run
/// it against a tampered table/world and watch it fail.
pub fn reconstruct_side_table(
    flat: &[GpuNode64],
    world: &VoxelWorld,
    origin: [i32; 3],
    size: u32,
) -> Result<(Vec<SideEntry>, Vec<([u32; 3], u32)>), BuildError> {
    let mut side = vec![SideEntry::EMPTY; flat.len()];
    let mut bounds: Vec<([u32; 3], u32)> = vec![([0; 3], 0); flat.len()];
    if flat.is_empty() {
        return Err(BuildError::SideTableMismatch("empty flatten".into()));
    }
    bounds[0] = ([0, 0, 0], size);
    let mut next_child = 1usize;
    for idx in 0..flat.len() {
        let node = &flat[idx];
        let (min, sz) = bounds[idx];
        if sz == 0 {
            return Err(BuildError::SideTableMismatch(format!(
                "node {idx} was never assigned bounds (flatten order is not the expected BFS)"
            )));
        }
        let mask = node_mask(node);
        let leaf = node.child_ptr_and_leaf & 0x8000_0000 != 0;
        if leaf {
            if mask != 0 {
                return Err(BuildError::SideTableMismatch(format!(
                    "node {idx} is both leaf and internal"
                )));
            }
            if sz != 1 {
                return Err(BuildError::SideTableMismatch(format!(
                    "leaf node {idx} has cell size {sz}, expected 1"
                )));
            }
            let world_pos = [
                min[0] as i32 + origin[0],
                min[1] as i32 + origin[1],
                min[2] as i32 + origin[2],
            ];
            let cell = world.get(world_pos).ok_or_else(|| {
                BuildError::SideTableMismatch(format!(
                    "leaf node {idx} reconstructs to {world_pos:?}, absent from the world"
                ))
            })?;
            if cell.lit == LIT_BLANK {
                return Err(BuildError::SideTableMismatch(format!(
                    "leaf node {idx} reconstructs to never-seen cell {world_pos:?}"
                )));
            }
            // Independent ground truth: the color the crate stored in
            // the flat node must be this voxel's inserted color.
            let expect = cell.color_u8();
            for c in 0..3 {
                let got = (node.color[c] * 255.0).round() as i32;
                if (got - expect[c] as i32).abs() > 1 {
                    return Err(BuildError::SideTableMismatch(format!(
                        "leaf node {idx} at {world_pos:?}: flat color {:?} != voxel color {:?}",
                        node.color, expect
                    )));
                }
            }
            side[idx] = SideEntry::from_cell(cell);
            continue;
        }
        if mask != 0 {
            let cp = node_child_ptr(node);
            if cp != next_child {
                return Err(BuildError::SideTableMismatch(format!(
                    "node {idx}: child_ptr {cp} != next BFS index {next_child}"
                )));
            }
            let child_size = sz / 4;
            let mut rank = 0usize;
            for cidx in 0..64usize {
                if mask & (1u64 << cidx) == 0 {
                    continue;
                }
                let part = [
                    (cidx & 3) as u32,
                    ((cidx >> 2) & 3) as u32,
                    ((cidx >> 4) & 3) as u32,
                ];
                bounds[cp + rank] = (
                    [
                        min[0] + part[0] * child_size,
                        min[1] + part[1] * child_size,
                        min[2] + part[2] * child_size,
                    ],
                    child_size,
                );
                rank += 1;
            }
            next_child += rank;
        }
    }
    if next_child != flat.len() {
        return Err(BuildError::SideTableMismatch(format!(
            "BFS walk assigned {next_child} children but the tree has {} nodes",
            flat.len()
        )));
    }
    Ok((side, bounds))
}

/// Verify an existing built tree's side-table against a world (the
/// check gate 2c.2 runs, factored out so a tampered copy can be fed
/// in and must fail).
pub fn verify_side_table(built: &BuiltTree, world: &VoxelWorld) -> Result<(), BuildError> {
    let (side, _) = reconstruct_side_table(&built.flat, world, built.origin, built.size)?;
    if side != built.side {
        let first = side
            .iter()
            .zip(built.side.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(0);
        return Err(BuildError::SideTableMismatch(format!(
            "side-table entry {first} differs: expected {:?}, found {:?}",
            side[first], built.side[first]
        )));
    }
    Ok(())
}
