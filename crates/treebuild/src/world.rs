//! The tree builder's world model: a sparse set of voxel cells in grid
//! space `(x = east, y = south, z = up block)` carrying everything the
//! ray tracer needs that the tree's `GpuNode64.color` cannot: a
//! material id, the game's `lit_level`, an alpha, and flags.
//!
//! `lit_level` values are the protocol's `LitLevel` enum verbatim
//! (schema/cdda_ipc.fbs): the game is the source of truth for
//! visibility. `LIT_BLANK` (never seen) cells are *excluded* from the
//! tree at build time (gate 2c.8); `LIT_MEMORIZED` cells are included
//! and flagged so the shader can desaturate them.

use std::collections::HashMap;

use mesher::block::Pass;
use mesher::chunk::ChunkMap;

pub const LIT_DARK: u8 = 0;
pub const LIT_LOW: u8 = 1;
pub const LIT_BRIGHT_ONLY: u8 = 2;
pub const LIT_LIT: u8 = 3;
pub const LIT_BRIGHT: u8 = 4;
pub const LIT_MEMORIZED: u8 = 5;
pub const LIT_BLANK: u8 = 6;

/// Side-table / cell flag bits.
pub const FLAG_TRANSPARENT: u32 = 1 << 0;
pub const FLAG_CUTOUT: u32 = 1 << 1;
pub const FLAG_EMISSIVE: u32 = 1 << 2;
pub const FLAG_MEMORIZED: u32 = 1 << 3;
/// Water surface (terrain item 2 follow-up, 2026-11-23):
/// the cell is a marked water-surface cell (its block
/// carries `mesher::block::WATER_ALPHA`); the RT shader
/// gives its up faces the water treatment. The WGSL
/// tests bit 4 (16u) -- keep in sync.
pub const FLAG_WATER: u32 = 1 << 4;

/// One occupied voxel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelCell {
    /// Albedo (linear-ish, matches the mesher's flat block colors).
    pub color: [f32; 3],
    /// Opacity 0..=1 (1 for opaque/cutout; <1 for the transparent pass).
    pub alpha: f32,
    /// Material key for the side-table. On the fake-data path this is
    /// the mesher tile id; with real TileDefs it is the material index
    /// the Phase 5 texture story resolves (see docs/phase2c-rt.md).
    pub material_id: u32,
    /// Protocol `LitLevel` (0..=6).
    pub lit: u8,
    /// FLAG_* bits.
    pub flags: u32,
}

impl VoxelCell {
    pub fn new(color: [f32; 3], material_id: u32) -> Self {
        Self {
            color,
            alpha: 1.0,
            material_id,
            lit: LIT_LIT,
            flags: 0,
        }
    }

    /// The color as the tree stores it: `[u8; 3]`, round-to-nearest.
    pub fn color_u8(&self) -> [u8; 3] {
        self.color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
    }

    pub fn is_transparent(&self) -> bool {
        self.flags & FLAG_TRANSPARENT != 0
    }
}

/// A sparse voxel world: occupied cells only, keyed by grid coord.
#[derive(Debug, Clone, Default)]
pub struct VoxelWorld {
    cells: HashMap<[i32; 3], VoxelCell>,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, pos: [i32; 3], cell: VoxelCell) {
        self.cells.insert(pos, cell);
    }

    pub fn get(&self, pos: [i32; 3]) -> Option<&VoxelCell> {
        self.cells.get(&pos)
    }

    pub fn remove(&mut self, pos: [i32; 3]) -> Option<VoxelCell> {
        self.cells.remove(&pos)
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[i32; 3], &VoxelCell)> {
        self.cells.iter()
    }

    /// Axis-aligned subset `[min, max)` per axis — the per-chunk-column
    /// rebuild unit for gate 2c.5's delta path: rebuilding one column
    /// is this subset fed through the same builder, not a second code
    /// path.
    pub fn subset_box(&self, min: [i32; 3], max: [i32; 3]) -> VoxelWorld {
        let mut out = VoxelWorld::new();
        for (pos, cell) in &self.cells {
            if (0..3).all(|a| pos[a] >= min[a] && pos[a] < max[a]) {
                out.set(*pos, *cell);
            }
        }
        out
    }

    /// Convert a mesher `ChunkMap` (already cutaway-filtered by the
    /// caller, exactly as `engine::world::mesh_world` filters for the
    /// raster path) into a voxel world. Material id = the block's tile
    /// id; lit defaults to `LIT_LIT` (fake data carries no lightmap);
    /// flags come from the block's render pass.
    pub fn from_chunk_map(map: &ChunkMap) -> VoxelWorld {
        let mut out = VoxelWorld::new();
        for chunk in map.chunks.values() {
            let origin = chunk.origin();
            for z in 0..chunk.grid.size[2] as i32 {
                for y in 0..chunk.grid.size[1] as i32 {
                    for x in 0..chunk.grid.size[0] as i32 {
                        let b = chunk.grid.get(x, y, z);
                        if b.is_air() {
                            continue;
                        }
                        let mut flags = 0;
                        match b.pass {
                            Pass::Transparent => flags |= FLAG_TRANSPARENT,
                            Pass::Cutout => flags |= FLAG_CUTOUT,
                            Pass::Opaque => {}
                        }
                        if b.is_water_surface() {
                            flags |= FLAG_WATER;
                        }
                        out.set(
                            [origin[0] + x, origin[1] + y, origin[2] + z],
                            VoxelCell {
                                color: [b.color[0], b.color[1], b.color[2]],
                                alpha: if flags & FLAG_WATER != 0 { 1.0 } else { b.color[3] },
                                material_id: b.id,
                                lit: LIT_LIT,
                                flags,
                            },
                        );
                    }
                }
            }
        }
        out
    }

    /// Phase 9 (9.5, two-tree design): the FINE tree's world —
    /// only the stamped cells of `layer`, in FINE coordinates
    /// `(fx, fy, fz)` (1 unit = 1 fine cell = 1/32 block; see
    /// `mesher::fine`). The coarse tree is built from
    /// [`VoxelWorld::from_chunk_map`] exactly as before and the
    /// two trees are composited by hit distance, so a scene
    /// with nothing stamped yields an empty fine world and no
    /// fine tree at all. Material id, colour and flags come
    /// from the stamped block (its id is the prop's synthetic
    /// material id on the engine path); `lit` defaults to
    /// `LIT_LIT` exactly as in `from_chunk_map` — a lit caller
    /// rewrites it per containing tile afterwards, the same
    /// post-pass `engine::rt::world_for_levels_lit` applies to
    /// the coarse world (the 9.6 v1 rule, docs/phase9-fine.md).
    pub fn from_fine_layer(layer: &mesher::fine::FineLayer) -> VoxelWorld {
        use mesher::fine::FINE;
        let mut out = VoxelWorld::new();
        for tile in layer.tiles() {
            for vol in layer.volumes_for_tile(tile) {
                for fz in vol.z0..vol.z0 + vol.depth_z {
                    for ly in 0..FINE {
                        for lx in 0..FINE {
                            let Some(b) = vol.get(lx, ly, fz) else {
                                continue;
                            };
                            if b.is_air() {
                                continue;
                            }
                            let mut flags = 0;
                            match b.pass {
                                Pass::Transparent => flags |= FLAG_TRANSPARENT,
                                Pass::Cutout => flags |= FLAG_CUTOUT,
                                Pass::Opaque => {}
                            }
                            if b.is_water_surface() {
                                flags |= FLAG_WATER;
                            }
                            out.set(
                                [tile.0 * FINE + lx, tile.1 * FINE + ly, fz],
                                VoxelCell {
                                    color: [b.color[0], b.color[1], b.color[2]],
                                    alpha: if flags & FLAG_WATER != 0 { 1.0 } else { b.color[3] },
                                    material_id: b.id,
                                    lit: LIT_LIT,
                                    flags,
                                },
                            );
                        }
                    }
                }
            }
        }
        out
    }
}
