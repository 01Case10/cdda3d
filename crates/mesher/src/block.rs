//! Block / tile types, flat colors and render-pass classification.

/// Render pass a block belongs to. Meshes are produced separately per pass
/// so the renderer can use different pipeline state (opaque, alpha-tested
/// cutout, blended transparent) per pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pass {
    Opaque = 0,
    Cutout = 1,
    Transparent = 2,
}

impl Pass {
    pub const ALL: [Pass; 3] = [Pass::Opaque, Pass::Cutout, Pass::Transparent];
    pub fn index(self) -> usize {
        self as usize
    }
}

/// One voxel block. `id == 0` is air (no geometry).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub id: u32,
    pub color: [f32; 4],
    pub pass: Pass,
    /// Phase 2f: compact texture-array layer indices
    /// `[top, side, bottom]` for this block's material, stamped by
    /// the engine from the shared texture spec builder when a scene
    /// is textured. `None` = flat color only (the pre-2f behavior).
    /// Because this field is part of `Block` equality, the greedy
    /// mesher's merge key never merges faces across different
    /// layers (gate 2f.1).
    pub tex_layers: Option<[u32; 3]>,
}

pub const AIR_ID: u32 = 0;

impl Block {
    pub const AIR: Block = Block {
        id: AIR_ID,
        color: [0.0, 0.0, 0.0, 0.0],
        pass: Pass::Transparent,
        tex_layers: None,
    };

    pub fn is_air(self) -> bool {
        self.id == AIR_ID
    }
}

/// How a game tile fills the blocks of its z-level (see `WorldScale`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileShape {
    /// Fills every block of the z-level (walls, doors, windows).
    Full,
    /// Fills only the bottom slab of the z-level (floors, ground).
    FloorSlab,
    /// Fills only the top slab of the z-level (roofs / ceilings).
    RoofSlab,
}

// Fake-data tile ids used by Phase 2 (the engine's fake house and the
// mesher tests). Real ids arrive with the Phase 1/3 tile-def table; the
// proto adapter will map them onto this same Block representation.
pub const TILE_FLOOR_WOOD: u32 = 1;
pub const TILE_WALL: u32 = 2;
pub const TILE_DOOR: u32 = 3;
pub const TILE_WINDOW: u32 = 4;
pub const TILE_ROOF: u32 = 5;
pub const TILE_GROUND: u32 = 6;
pub const TILE_FENCE: u32 = 7;
pub const TILE_WATER: u32 = 8;
pub const TILE_FOLIAGE: u32 = 9;

/// Flat color + pass for a fake-data tile id. Unknown ids get the classic
/// magenta placeholder (plan §8 fallback) in the opaque pass.
pub fn block_for_tile(id: u32) -> Block {
    let (color, pass) = match id {
        AIR_ID => return Block::AIR,
        TILE_FLOOR_WOOD => ([0.55, 0.38, 0.22, 1.0], Pass::Opaque),
        TILE_WALL => ([0.70, 0.68, 0.62, 1.0], Pass::Opaque),
        TILE_DOOR => ([0.40, 0.25, 0.12, 1.0], Pass::Opaque),
        TILE_WINDOW => ([0.60, 0.80, 0.90, 0.35], Pass::Transparent),
        TILE_ROOF => ([0.50, 0.20, 0.15, 1.0], Pass::Opaque),
        TILE_GROUND => ([0.35, 0.50, 0.25, 1.0], Pass::Opaque),
        TILE_FENCE => ([0.45, 0.33, 0.20, 1.0], Pass::Cutout),
        TILE_WATER => ([0.20, 0.40, 0.80, 0.60], Pass::Transparent),
        TILE_FOLIAGE => ([0.20, 0.45, 0.20, 1.0], Pass::Cutout),
        _ => ([1.0, 0.0, 1.0, 1.0], Pass::Opaque), // unknown id placeholder
    };
    Block {
        id,
        color,
        pass,
        tex_layers: None,
    }
}

/// Shape for a fake-data tile id.
pub fn tile_shape(id: u32) -> TileShape {
    match id {
        // Water is a floor slab: its surface sits at the
        // floor line of its level (matching the real
        // snapshot path, where t_water_dp's voxelizer
        // height is 1). As a full-height tile it stood
        // two blocks proud of the grass -- a fish tank,
        // not a pond (Joey, 2026-11-17).
        TILE_FLOOR_WOOD | TILE_GROUND | TILE_WATER => TileShape::FloorSlab,
        TILE_ROOF => TileShape::RoofSlab,
        _ => TileShape::Full,
    }
}
