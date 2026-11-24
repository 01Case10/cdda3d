//! Gate 8.4b — open air, roofs and multi-storey buildings.
//!
//! The game's rule (verified against the pinned source): a
//! terrain def carries `roof` — "what will be the floor above this
//! terrain" (`ter_t::roof`, src/mapdata.h). The tile above a floor
//! is typically `t_open_air`, which is *empty space* (flags
//! `NO_FLOOR`, `EMPTY_SPACE`): it contributes no block and no
//! floor of its own. What you stand on — and what the renderer
//! must draw as the floor at that level — is the **below tile's
//! roof material**.
//!
//! So the resolution is single-step and per-column:
//!
//! - solid terrain at z → its own material is the floor;
//! - `t_open_air` at z → the floor is `roof_result` of the terrain
//!   at z−1, resolved through the def table; if the tile below
//!   declares no roof (grass, dirt, water...), there is **no
//!   floor** — a hole, not a fabricated slab;
//! - `t_open_air` above `t_open_air` → no floor at this level
//!   either (a shaft/pit: the floor, if any, belongs to the level
//!   whose below-tile is solid — you see down the shaft to it).
//!
//! The rule is a pure function of the column and the def table,
//! so it is independent of the cutaway: the cutaway chooses which
//! z-levels get *drawn*, it never changes what a level's floor
//! *is* — the gate's "no missing ceilings/floors at any cutaway"
//! is that independence, tested below.

use crate::doors::DefTable;

/// What the floor at one level of a column is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Floor {
    /// The tile itself is solid terrain; its own def id is the
    /// floor material.
    Solid(u32),
    /// The tile is open air; the floor material is the `roof`
    /// (second field) of the solid terrain below it (first field).
    RoofOf(u32, u32),
    /// No floor at this level: open air over a roof-less terrain,
    /// over more open air, or at the bottom of the world.
    None,
}

/// One vertical column of terrain ids, bottom first.
#[derive(Debug, Clone)]
pub struct Column {
    pub z_min: i32,
    pub terrain: Vec<u32>,
}

impl Column {
    pub fn terrain_at(&self, z: i32) -> Option<u32> {
        let i = (z - self.z_min) as usize;
        self.terrain.get(i).copied()
    }
}

/// Does this terrain id emit a block at all? Open air (and the
/// reserved id 0, "none") emit nothing — every phantom-floor bug
/// in this area starts with giving open air geometry.
pub fn emits_block(table: &DefTable, terrain_id: u32) -> bool {
    terrain_id != 0 && terrain_id != table.open_air_id
}

/// The floor at level `z` of `column`, per the game's roof rule.
pub fn floor_at(table: &DefTable, column: &Column, z: i32) -> Floor {
    let Some(here) = column.terrain_at(z) else {
        return Floor::None;
    };
    if emits_block(table, here) {
        return Floor::Solid(here);
    }
    // Open air (or none): the floor comes from below, or there
    // is none. Single step — if below is itself open air, this
    // level has no floor (the shaft continues down).
    let Some(below) = column.terrain_at(z - 1) else {
        return Floor::None;
    };
    if !emits_block(table, below) {
        return Floor::None;
    }
    let Some(def) = table.def(proto::TileKind::Terrain, below) else {
        return Floor::None;
    };
    match table.resolve(proto::TileKind::Terrain, &def.roof_result) {
        Some(roof_id) => Floor::RoofOf(below, roof_id),
        None => Floor::None,
    }
}
