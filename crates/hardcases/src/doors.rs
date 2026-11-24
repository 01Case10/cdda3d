//! Gate 8.4 — door and window states.
//!
//! The contract under test: for terrain doors and furniture doors
//! alike, each action (open / close / bash) arrives as a Delta
//! `TileChange` carrying the **new terrain/furniture id** — the
//! result of the game's own transform — never a state flag the
//! engine interprets. The engine-side work is therefore exactly
//! two things, both proven here:
//!
//! 1. *Recognition*: given the TileDefs table, a transition
//!    old → new can be classified as the def's declared open /
//!    close / bash transform (useful for sound, animation and
//!    debugging — and a consistency check on the stream).
//! 2. *Application*: after the change is applied, the world holds
//!    the new id and the mesher produces the new id's block —
//!    model and texture follow the id, per the gate's wording.

use std::collections::HashMap;

use proto::{TileChangeT, TileDefT, TileDefsT, TileKind};

/// What a tile-id transition means, per the def table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// new == old def's `open_result`.
    Open,
    /// new == old def's `close_result`.
    Close,
    /// new == old def's `bash_result`.
    Bash,
    /// No change (old == new).
    Unchanged,
    /// Not one of the old def's declared transforms: a different
    /// edit (terrain replaced by worldgen, a trap springing, a
    /// mod's scripted change...). Perfectly legal — the engine
    /// applies it the same way; it just isn't a door action.
    Other,
}

/// The TileDefs table with string-id resolution: transforms are
/// exported as *string* ids (`open_result: "t_door_o"`), so
/// classifying a numeric transition means resolving the string
/// back to the def id it names in the same table.
///
/// **Ids are per-kind.** The game's factories (and therefore the
/// fork's real adapter, and the sim's generated table) number
/// each kind separately — terrain 345 and item 345 are different
/// defs — and 147 string ids in the real data exist in two kinds
/// at once. This table is therefore keyed by `(kind, id)` and
/// `(kind, string_id)` throughout. (Before 2026-11-23 it was
/// keyed by raw id alone, last-wins across kinds: a terrain
/// roof lookup could resolve to an item def with no
/// `roof_result`, and implicit roofs silently vanished for any
/// per-kind table. Found by the Phase 11 sim world-parity gate;
/// the hand-built fixture tables use globally unique ids, which
/// is why no earlier gate saw it.)
#[derive(Debug, Default)]
pub struct DefTable {
    by_kind_id: HashMap<(TileKind, u32), TileDefT>,
    id_by_string: HashMap<(TileKind, String), u32>,
    /// The connect-group names, by the index defs reference (a
    /// mod's new groups arrive here, gate 8.7).
    connect_groups: Vec<String>,
    /// `t_open_air`'s terrain def id, resolved by string id at
    /// build time (0 = the table has no open-air def; real
    /// TileDefs always do). Cached because [`crate::openair`]
    /// asks per tile.
    pub open_air_id: u32,
}

impl DefTable {
    pub fn from_tiledefs(defs: &TileDefsT) -> Self {
        let mut table = Self::default();
        if let Some(groups) = &defs.connect_groups {
            table.connect_groups = groups.clone();
        }
        if let Some(list) = &defs.defs {
            for d in list {
                if let Some(s) = &d.string_id {
                    table.id_by_string.insert((d.kind, s.clone()), d.id);
                    if d.kind == TileKind::Terrain && s == "t_open_air" {
                        table.open_air_id = d.id;
                    }
                }
                table.by_kind_id.insert((d.kind, d.id), d.clone());
            }
        }
        table
    }

    /// The def of the given kind with the given (per-kind) id.
    pub fn def(&self, kind: TileKind, id: u32) -> Option<&TileDefT> {
        self.by_kind_id.get(&(kind, id))
    }

    /// Every def in the table (report generation, gate 8.7).
    pub fn defs(&self) -> impl Iterator<Item = &TileDefT> {
        self.by_kind_id.values()
    }

    /// Connect-group names by index.
    pub fn connect_groups(&self) -> &[String] {
        &self.connect_groups
    }

    /// Resolve a transform's string result to its def id **in
    /// the same kind** (`None` = the def declares no such
    /// transform, or the result names a def the table doesn't
    /// carry in that kind). Transforms never cross kinds: a
    /// terrain's `roof_result` names a terrain, a furniture's
    /// `bash_result` names a furniture.
    pub fn resolve(&self, kind: TileKind, result: &Option<String>) -> Option<u32> {
        result
            .as_ref()
            .and_then(|s| self.id_by_string.get(&(kind, s.clone())).copied())
    }
}

/// Classify the transition `old_id → new_id` for one tile slot
/// of the given kind (terrain or furniture — the rule is
/// identical, the defs differ only in kind).
pub fn classify_transition(
    table: &DefTable,
    kind: TileKind,
    old_id: u32,
    new_id: u32,
) -> Transition {
    if old_id == new_id {
        return Transition::Unchanged;
    }
    let Some(def) = table.def(kind, old_id) else {
        return Transition::Other;
    };
    if table.resolve(kind, &def.open_result) == Some(new_id) {
        Transition::Open
    } else if table.resolve(kind, &def.close_result) == Some(new_id) {
        Transition::Close
    } else if table.resolve(kind, &def.bash_result) == Some(new_id) {
        Transition::Bash
    } else {
        Transition::Other
    }
}

/// A minimal tile world for gate purposes: the terrain/furniture
/// ids at absolute tile positions, with Delta application. (The
/// live engine's world store is Phase 2h's frame loop; the rule
/// proven here — a change replaces the id wholesale — is the one
/// that store implements.)
#[derive(Debug, Default)]
pub struct TileWorld {
    pub terrain: HashMap<[i32; 3], u32>,
    pub furniture: HashMap<[i32; 3], u32>,
}

impl TileWorld {
    /// Apply one Delta tile change. Returns the terrain and
    /// furniture transitions it represents, classified against
    /// the def table (a slot whose id didn't move classifies as
    /// `Unchanged`).
    pub fn apply_change(
        &mut self,
        table: &DefTable,
        change: &TileChangeT,
    ) -> (Transition, Transition) {
        let pos = change.pos.as_ref();
        let at = [
            pos.map_or(0, |p| p.x),
            pos.map_or(0, |p| p.y),
            pos.map_or(0, |p| p.z),
        ];
        let old_ter = self.terrain.get(&at).copied().unwrap_or(0);
        let old_furn = self.furniture.get(&at).copied().unwrap_or(0);
        let ter_t = classify_transition(table, TileKind::Terrain, old_ter, change.terrain);
        let furn_t = classify_transition(table, TileKind::Furniture, old_furn, change.furniture);
        self.terrain.insert(at, change.terrain);
        self.furniture.insert(at, change.furniture);
        (ter_t, furn_t)
    }

    /// The composed-tile foreground reference (`fg_ref`) the
    /// tile's current terrain id resolves to — the "model and
    /// texture match the new id" half of gate 8.4, at data level:
    /// after a state change the world holds the new id, and the
    /// new id's def carries the new sprite/material reference.
    /// (That an id change re-meshes the chunk is the mesher's own
    /// Phase 2 gate, not re-proven here.)
    pub fn terrain_fg_ref(&self, table: &DefTable, at: [i32; 3]) -> Option<i32> {
        let id = self.terrain.get(&at).copied().unwrap_or(0);
        table.def(TileKind::Terrain, id).map(|d| d.fg_ref)
    }
}
