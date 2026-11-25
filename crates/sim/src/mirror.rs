//! The client-side mirror (S7): a state rebuilt ONLY from the
//! wire message stream — a Snapshot seeds it, Deltas and
//! LightUpdates patch it — with no access to the World's
//! internals. It is the T12 invariant in data form: after any
//! flush, the mirror must equal a fresh Snapshot of the world.
//!
//! Factored out of the per-slice gates (turn_gate S2,
//! creature_field_gate S5, destruction_gate / vehicle_gate
//! S6 each carried a private copy) so the S7 scenario runner
//! asserts the invariant with the same code the gates proved,
//! instead of a fifth copy. The gates keep their copies; this
//! is the canonical one new code uses.
//!
//! Coverage: tiles (terrain/furniture/trap/fields/displayed
//! field/items/lit/seen/graffiti/outside/light frac+tint),
//! entities, and vehicles — the state Deltas fully determine.
//!
//! Tile memory is covered differently, by necessity: memory
//! blocks ride Snapshots only (the Delta schema has none), so
//! a delta-fed client cannot track memory-block membership
//! between snapshots — a tile that leaves the view memorized
//! is indistinguishable, mid-stream, from one still in view
//! but unlit (both arrive as lit BLANK live tiles; the
//! exporter's memory blocks exclude currently-visible tiles,
//! so a fresh Snapshot does not transmit those either). The
//! mirror therefore accumulates memory from sightings
//! (lit != BLANK in any live message) and
//! [`Mirror::assert_memory_consistent`] checks, at each
//! Snapshot, that wherever the incoming Snapshot transmits a
//! memory for a tile the accumulated mirror also remembers,
//! the remembered terrain/furniture agree — no stale memory
//! on either side. Membership equality is asserted only where
//! the wire can carry it: at the reseed itself.

use std::collections::HashMap;

use proto::{EntityT, LitLevel, MessageT, SnapshotT, VehicleT};

#[derive(Debug, Clone, PartialEq)]
pub struct TileMirror {
    pub terrain: u32,
    pub furniture: u32,
    pub trap: u32,
    pub fields: Vec<(u32, u8)>,
    pub displayed: (u32, u8),
    pub top_item: u32,
    pub item_count: u16,
    pub lit: LitLevel,
    pub seen: u8,
    pub graffiti: bool,
    pub outside: bool,
    pub frac: u8,
    pub tint: [u8; 3],
}

#[derive(Debug, Default, PartialEq)]
pub struct Mirror {
    pub tiles: HashMap<[i32; 3], TileMirror>,
    pub entities: HashMap<u64, EntityT>,
    pub vehicles: HashMap<u64, VehicleT>,
    pub memory: HashMap<[i32; 3], (u32, u32)>,
}

impl Mirror {
    /// Seed a mirror from a Snapshot (the only state a fresh
    /// client has).
    pub fn from_snapshot(snap: &SnapshotT) -> Mirror {
        let mut m = Mirror::default();
        for b in snap.blocks.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
            let o = b.origin.as_ref().expect("origin");
            for i in 0..144usize {
                let at = |arr: &Option<Vec<u32>>| {
                    arr.as_ref().and_then(|v| v.get(i)).copied().unwrap_or(0)
                };
                let fields = b
                    .fields
                    .as_ref()
                    .map(|fs| {
                        fs.iter()
                            .filter(|f| f.tile_index as usize == i)
                            .map(|f| (f.type_, f.intensity))
                            .collect()
                    })
                    .unwrap_or_default();
                let displayed = b
                    .displayed_field
                    .as_ref()
                    .and_then(|v| v.get(i))
                    .map(|d| (d.field_type, d.intensity))
                    .unwrap_or((0, 0));
                m.tiles.insert(
                    [o.x + (i % 12) as i32, o.y + (i / 12) as i32, o.z],
                    TileMirror {
                        terrain: at(&b.terrain),
                        furniture: at(&b.furniture),
                        trap: at(&b.trap),
                        fields,
                        displayed,
                        top_item: at(&b.top_item),
                        item_count: b
                            .item_count
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(0),
                        lit: b
                            .lit
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(LitLevel::DARK),
                        seen: b.seen.as_ref().and_then(|v| v.get(i)).copied().unwrap_or(0),
                        graffiti: b
                            .has_graffiti
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(false),
                        outside: b
                            .outside
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(false),
                        frac: b
                            .light_frac
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(0),
                        tint: b
                            .light_tint
                            .as_ref()
                            .and_then(|v| {
                                let j = i * 3;
                                Some([*v.get(j)?, *v.get(j + 1)?, *v.get(j + 2)?])
                            })
                            .unwrap_or([255, 255, 255]),
                    },
                );
            }
        }
        for (pos, t) in &m.tiles {
            if t.lit != LitLevel::BLANK {
                m.memory.insert(*pos, (t.terrain, t.furniture));
            }
        }
        for b in snap
            .memory_blocks
            .as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(&[])
        {
            let o = b.origin.as_ref().expect("origin");
            for i in 0..144usize {
                if b.lit.as_ref().and_then(|v| v.get(i)).copied() == Some(LitLevel::MEMORIZED) {
                    let at = |arr: &Option<Vec<u32>>| {
                        arr.as_ref().and_then(|v| v.get(i)).copied().unwrap_or(0)
                    };
                    m.memory.insert(
                        [o.x + (i % 12) as i32, o.y + (i / 12) as i32, o.z],
                        (at(&b.terrain), at(&b.furniture)),
                    );
                }
            }
        }
        for e in snap.entities.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
            m.entities.insert(e.id, e.clone());
        }
        for v in snap.vehicles.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
            m.vehicles.insert(v.id, v.clone());
        }
        m
    }

    /// Apply one wire message. State messages (Snapshot /
    /// Delta / LightUpdate) patch the mirror; companion
    /// messages (EnvironmentUpdate / Effects / UiFrame) carry
    /// no mirror state and are ignored. Anything else is a
    /// protocol surprise and is reported, not silently eaten.
    pub fn apply(&mut self, msg: &MessageT) -> Result<(), String> {
        match msg {
            MessageT::Snapshot(s) => *self = Mirror::from_snapshot(s),
            MessageT::Delta(d) => {
                for tc in d.tile_changes.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                    let p = tc.pos.as_ref().expect("tile change pos");
                    let fields = tc
                        .fields
                        .as_ref()
                        .map(|fs| fs.iter().map(|f| (f.type_, f.intensity)).collect())
                        .unwrap_or_default();
                    let displayed = tc
                        .displayed_field
                        .as_ref()
                        .map(|df| (df.field_type, df.intensity))
                        .unwrap_or((0, 0));
                    let pos = [p.x, p.y, p.z];
                    if tc.lit != LitLevel::BLANK {
                        self.memory.insert(pos, (tc.terrain, tc.furniture));
                    }
                    self.tiles.insert(
                        pos,
                        TileMirror {
                            terrain: tc.terrain,
                            furniture: tc.furniture,
                            trap: tc.trap,
                            fields,
                            displayed,
                            top_item: tc.top_item,
                            item_count: tc.item_count,
                            lit: tc.lit,
                            seen: tc.seen,
                            graffiti: tc.has_graffiti,
                            outside: tc.outside,
                            frac: tc.light_frac,
                            tint: [tc.light_tint_r, tc.light_tint_g, tc.light_tint_b],
                        },
                    );
                }
                for e in d
                    .entity_updates
                    .as_ref()
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                {
                    self.entities.insert(e.id, e.clone());
                }
                for id in d
                    .entity_removed
                    .as_ref()
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                {
                    self.entities.remove(id);
                }
                for v in d
                    .vehicle_updates
                    .as_ref()
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                {
                    self.vehicles.insert(v.id, v.clone());
                }
                for id in d
                    .vehicle_removed
                    .as_ref()
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                {
                    self.vehicles.remove(id);
                }
            }
            MessageT::LightUpdate(lu) => {
                let o = lu.origin.as_ref().expect("light update origin");
                for i in 0..144usize {
                    let pos = [o.x + (i % 12) as i32, o.y + (i / 12) as i32, o.z];
                    let lit = lu
                        .lit
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or(LitLevel::BLANK);
                    if let Some(t) = self.tiles.get_mut(&pos) {
                        t.lit = lit;
                        t.seen = lu
                            .seen
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(0);
                        t.frac = lu
                            .light_frac
                            .as_ref()
                            .and_then(|v| v.get(i))
                            .copied()
                            .unwrap_or(0);
                        if let Some(tint) = &lu.light_tint {
                            let j = i * 3;
                            if let (Some(r), Some(g), Some(b)) =
                                (tint.get(j), tint.get(j + 1), tint.get(j + 2))
                            {
                                t.tint = [*r, *g, *b];
                            }
                        }
                    }
                    if lit != LitLevel::BLANK {
                        if let Some(t) = self.tiles.get(&pos) {
                            let tf = (t.terrain, t.furniture);
                            self.memory.insert(pos, tf);
                        }
                    }
                }
            }
            MessageT::EnvironmentUpdate(_) | MessageT::Effects(_) | MessageT::UiFrame(_) => {}
            other => return Err(format!("mirror: unexpected message {other:?}")),
        }
        Ok(())
    }

    /// The invariant over the delta-determined state: tiles,
    /// entities and vehicles of this mirror must equal a
    /// mirror seeded from `snap`. (Memory is checked
    /// separately — see the module docs.) On divergence the
    /// error names the first differing tile / entity /
    /// vehicle found.
    pub fn assert_matches(&self, snap: &SnapshotT) -> Result<(), String> {
        let fresh = Mirror::from_snapshot(snap);
        if self.tiles == fresh.tiles
            && self.entities == fresh.entities
            && self.vehicles == fresh.vehicles
        {
            return Ok(());
        }
        for (pos, t) in &fresh.tiles {
            match self.tiles.get(pos) {
                None => return Err(format!("mirror: tile {pos:?} missing (fresh: {t:?})")),
                Some(mt) if mt != t => {
                    return Err(format!(
                        "mirror: tile {pos:?} diverged: mirror {mt:?} vs fresh {t:?}"
                    ))
                }
                _ => {}
            }
        }
        for pos in self.tiles.keys() {
            if !fresh.tiles.contains_key(pos) {
                return Err(format!("mirror: stale tile {pos:?} not in fresh snapshot"));
            }
        }
        for (id, e) in &fresh.entities {
            match self.entities.get(id) {
                None => return Err(format!("mirror: entity {id} missing (fresh: {e:?})")),
                Some(me) if me != e => {
                    return Err(format!(
                        "mirror: entity {id} diverged: mirror {me:?} vs fresh {e:?}"
                    ))
                }
                _ => {}
            }
        }
        for id in self.entities.keys() {
            if !fresh.entities.contains_key(id) {
                return Err(format!("mirror: stale entity {id} not in fresh snapshot"));
            }
        }
        for (id, v) in &fresh.vehicles {
            match self.vehicles.get(id) {
                None => return Err(format!("mirror: vehicle {id} missing")),
                Some(mv) if mv != v => return Err(format!("mirror: vehicle {id} diverged")),
                _ => {}
            }
        }
        for id in self.vehicles.keys() {
            if !fresh.vehicles.contains_key(id) {
                return Err(format!("mirror: stale vehicle {id} not in fresh snapshot"));
            }
        }
        Err("mirror: diverged (no single component identified)".to_string())
    }

    /// The memory consistency check, run when a Snapshot
    /// arrives (before reseeding): for every tile the incoming
    /// Snapshot transmits a memory for (its memory blocks plus
    /// its currently-lit live tiles), if this mirror also
    /// remembers the tile, the remembered terrain/furniture
    /// must agree. Membership itself is not compared — the
    /// wire cannot carry it between snapshots (module docs).
    pub fn assert_memory_consistent(&self, snap: &SnapshotT) -> Result<(), String> {
        let fresh = Mirror::from_snapshot(snap);
        for (pos, tf) in &fresh.memory {
            if let Some(mtf) = self.memory.get(pos) {
                if mtf != tf {
                    return Err(format!(
                        "mirror: memory tile {pos:?} diverged: mirror remembers {mtf:?}, \
                         the snapshot transmits {tf:?}"
                    ));
                }
            }
        }
        Ok(())
    }

    /// The player entity in the mirror (the avatar: sprite
    /// `player_male` / `player_female`, the tile-entry ids the
    /// game draws the avatar with).
    pub fn player_entity(&self) -> Option<&EntityT> {
        self.entities.values().find(|e| {
            matches!(
                e.sprite_id.as_deref(),
                Some("player_male") | Some("player_female")
            )
        })
    }

    pub fn player_pos(&self) -> Option<[i32; 3]> {
        self.player_entity()
            .and_then(|e| e.pos.as_ref())
            .map(|p| [p.x, p.y, p.z])
    }
}
