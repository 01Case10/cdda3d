//! The light grid: per-tile game light over a tile rectangle, and the
//! sampler trait smoothing reads through.
//!
//! Coordinates are *tile* coordinates: `(x = east tile, y = south tile,
//! z = game z-level index)`. This is deliberately not block space —
//! light is per-tile game data; the bakers translate block <-> tile
//! via `mesher::WorldScale::blocks_per_z_level`.

use std::collections::BTreeSet;

use proto::LitLevel;

use crate::mapping::{TileLight, Visibility};

/// Read-only whole-grid access to tile light. This is the analogue of
/// `mesher::VoxelSampler` for light: smoothing samples through this
/// trait over the *whole* stored world, never per-chunk in isolation,
/// which is exactly why gradients have no seam at chunk borders
/// (gate 7.4) — a chunk-local sampler would clamp at the border.
pub trait LightSampler {
    /// Light at a tile; out-of-bounds reads as `TileLight::BLANK`
    /// (never seen), mirroring `VoxelGrid::get` reading air.
    fn tile_light(&self, x: i32, y: i32, z: i32) -> TileLight;
    fn is_occluding(&self, x: i32, y: i32, z: i32) -> bool {
        self.tile_light(x, y, z).occluding
    }
    fn visibility_at(&self, x: i32, y: i32, z: i32) -> Visibility {
        self.tile_light(x, y, z).visibility()
    }
}

/// A dense light grid with an origin (the tile coordinate of index
/// `(0,0,0)`), so snapshot windows whose origin is not `(0,0,0)` work
/// unchanged.
#[derive(Debug, Clone)]
pub struct LightGrid {
    pub origin: [i32; 3],
    pub size: [usize; 3], // [x, y, z-levels]
    tiles: Vec<TileLight>,
}

impl LightGrid {
    pub fn new(origin: [i32; 3], size: [usize; 3]) -> Self {
        Self {
            origin,
            size,
            tiles: vec![TileLight::BLANK; size[0] * size[1] * size[2]],
        }
    }
    /// A grid filled with one light value (fixtures / uniform scenes).
    pub fn filled(origin: [i32; 3], size: [usize; 3], t: TileLight) -> Self {
        let mut g = Self::new(origin, size);
        g.tiles.fill(t);
        g
    }
    fn idx(&self, lx: usize, ly: usize, lz: usize) -> usize {
        (lz * self.size[1] + ly) * self.size[0] + lx
    }
    pub fn in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        let (lx, ly, lz) = (x - self.origin[0], y - self.origin[1], z - self.origin[2]);
        lx >= 0
            && ly >= 0
            && lz >= 0
            && (lx as usize) < self.size[0]
            && (ly as usize) < self.size[1]
            && (lz as usize) < self.size[2]
    }
    pub fn get(&self, x: i32, y: i32, z: i32) -> TileLight {
        if !self.in_bounds(x, y, z) {
            return TileLight::BLANK;
        }
        self.tiles[self.idx(
            (x - self.origin[0]) as usize,
            (y - self.origin[1]) as usize,
            (z - self.origin[2]) as usize,
        )]
    }
    pub fn set(&mut self, x: i32, y: i32, z: i32, t: TileLight) {
        if !self.in_bounds(x, y, z) {
            return;
        }
        let i = self.idx(
            (x - self.origin[0]) as usize,
            (y - self.origin[1]) as usize,
            (z - self.origin[2]) as usize,
        );
        self.tiles[i] = t;
    }
    pub fn iter_tiles(&self) -> impl Iterator<Item = ([i32; 3], TileLight)> + '_ {
        self.tiles.iter().enumerate().map(move |(i, t)| {
            let lz = i / (self.size[0] * self.size[1]);
            let rem = i % (self.size[0] * self.size[1]);
            let ly = rem / self.size[0];
            let lx = rem % self.size[0];
            (
                [
                    self.origin[0] + lx as i32,
                    self.origin[1] + ly as i32,
                    self.origin[2] + lz as i32,
                ],
                *t,
            )
        })
    }
    pub fn count_visibility(&self, v: Visibility) -> usize {
        self.tiles.iter().filter(|t| t.visibility() == v).count()
    }
}

impl LightSampler for LightGrid {
    fn tile_light(&self, x: i32, y: i32, z: i32) -> TileLight {
        self.get(x, y, z)
    }
}

/// Sampler backed by a closure — the light analogue of
/// `mesher::ClosureSampler`, handy for adapters (e.g. occlusion read
/// from a `ChunkMap` while light comes from a `LightGrid`).
pub struct ClosureLightSampler<F: Fn(i32, i32, i32) -> TileLight>(pub F);
impl<F: Fn(i32, i32, i32) -> TileLight> LightSampler for ClosureLightSampler<F> {
    fn tile_light(&self, x: i32, y: i32, z: i32) -> TileLight {
        (self.0)(x, y, z)
    }
}

/// The visibility mask both renderers consume (gate 7.2 contract as a
/// reusable object): wraps any sampler and answers the two questions
/// renderers ask — *may this tile be shown at all?* and *how is it
/// presented?* The mesher path consults it per block (via the block's
/// tile); the treebuild path consults it per voxel before inserting
/// into the tree, so a `BLANK` voxel never enters the tree and can
/// never be revealed by a shadow/visibility ray.
#[derive(Clone, Copy)]
pub struct VisibilityMask<'a> {
    pub sampler: &'a dyn LightSampler,
}
impl<'a> VisibilityMask<'a> {
    pub fn new(sampler: &'a dyn LightSampler) -> Self {
        Self { sampler }
    }
    pub fn visibility(&self, x: i32, y: i32, z: i32) -> Visibility {
        self.sampler.visibility_at(x, y, z)
    }
    pub fn is_meshed(&self, x: i32, y: i32, z: i32) -> bool {
        crate::mapping::is_meshed(self.visibility(x, y, z))
    }
    pub fn include_in_tree(&self, x: i32, y: i32, z: i32) -> bool {
        crate::mapping::include_in_tree(self.visibility(x, y, z))
    }
    /// All chunk coords (mesher chunk = 12x12 tiles) that contain at
    /// least one tile of the given visibility — used by tests and by
    /// the parent's wiring to sanity-check a baked scene.
    pub fn chunks_with(
        &self,
        grid_size: [usize; 3],
        origin: [i32; 3],
        v: Visibility,
    ) -> BTreeSet<[i32; 2]> {
        let mut out = BTreeSet::new();
        for lz in 0..grid_size[2] as i32 {
            for ly in 0..grid_size[1] as i32 {
                for lx in 0..grid_size[0] as i32 {
                    let (x, y, z) = (origin[0] + lx, origin[1] + ly, origin[2] + lz);
                    if self.visibility(x, y, z) == v {
                        out.insert(mesher::ChunkMap::chunk_coord_of(x, y));
                    }
                }
            }
        }
        out
    }
}

/// Convenience constructor from raw protocol values, used by the
/// proto adapter and fixtures.
pub fn tile_from_proto(lit: LitLevel, seen: u8, occluding: bool) -> TileLight {
    TileLight {
        lit,
        seen,
        tint: [1.0, 1.0, 1.0],
        occluding,
        outside: false,
    }
}
