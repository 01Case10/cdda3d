//! Wire-fed scene source (Phase 11 follow-up, Lane A): a world
//! built from the two messages a game server sends at connect
//! — `TileDefs` + `Snapshot` — carried as a renderable scene
//! source for `frame::render_frame` / `capture::run_capture`.
//!
//! Before this module, every public capture/render entry took
//! `CaptureArgs::snapshot` as a fixture *name* ("stub-house" /
//! "sample-scene"), so a world that arrived over the wire could
//! be ingested (`snapshot_world::build_snapshot_world`, the
//! client session's world build) but never seated into the
//! frame loop: no pixel captures of wire data, and no windowed
//! session against a live server. `WireScene` is that seating:
//! the caller builds the block world once (choosing the block
//! resolver — the gates use a canonical palette, the client
//! session the coarse stand-in, a real desktop run the
//! voxelizer materials) and hands the built map to the frame;
//! both renderer paths consume it through the same filtered-map
//! preparation the fixtures use.
//!
//! Live state: a session folds each accepted `Delta` into its
//! stored `Snapshot` with [`apply_delta_to_snapshot`] and
//! rebuilds the `WireScene`, so the seated world is the
//! delta-fed state, not just the connect-time snapshot.

use std::sync::Arc;

use mesher::block::Block;
use mesher::chunk::ChunkMap;
use mesher::scale::WorldScale;

use crate::snapshot_world::build_snapshot_world;

/// A built wire world: the block map derived from a
/// `TileDefs` + `Snapshot` pair (at the frame path's default
/// world scale — every scene source in the frame loop is), and
/// the snapshot window's minimum game z (chunk level 0's game
/// z, the offset the lighting bake and the cutaway key off).
///
/// The map is shared (`Arc`): `CaptureArgs` is cloned per
/// frame in the windowed loop, and rebuilding wire columns per
/// frame would be waste. Rebuild on state change instead —
/// a session publishes a fresh `WireScene` per applied flush.
#[derive(Debug, Clone)]
pub struct WireScene {
    pub map: Arc<ChunkMap>,
    pub min_game_z: i32,
}

impl WireScene {
    /// Build from the connect-time wire pair. `block_for`
    /// resolves a terrain def id (in `defs`' per-kind id space)
    /// to the block stamped for it — the same resolver argument
    /// [`build_snapshot_world`] takes.
    pub fn from_wire(
        defs: &proto::TileDefsT,
        snapshot: &proto::SnapshotT,
        block_for: &dyn Fn(u32) -> Block,
    ) -> Self {
        let map = build_snapshot_world(defs, snapshot, WorldScale::default(), block_for);
        Self::from_map(map, snapshot)
    }

    /// Wrap an already-built snapshot world (built by
    /// [`build_snapshot_world`] from `snapshot` at the default
    /// scale), deriving `min_game_z` from the snapshot's block
    /// origins.
    pub fn from_map(map: ChunkMap, snapshot: &proto::SnapshotT) -> Self {
        let min_game_z = snapshot
            .blocks
            .as_ref()
            .map(|bs| {
                bs.iter()
                    .map(|b| b.origin.as_ref().map(|o| o.z).unwrap_or(0))
                    .min()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        Self {
            map: Arc::new(map),
            min_game_z,
        }
    }
}

/// Fold one `Delta` into a stored `Snapshot`, producing the
/// live state the next [`WireScene::from_wire`] build renders.
/// This is the snapshot-level counterpart of the sim's mirror
/// (which keeps its own tile map): per-tile changes land in
/// their submap block's arrays, entity/vehicle updates replace
/// by id, removals drop, and a piggybacked environment replaces
/// the snapshot's.
///
/// Field entries are sparse (a block's `fields` list plus the
/// per-tile `field_index`): a tile change replaces that tile's
/// entries wholesale and `field_index` is rebuilt for the
/// block, so the stored snapshot stays in the exact shape
/// `columns_from_snapshot` / `grid_from_snapshot` consume.
///
/// Light changes are NOT folded: `LightUpdate` carries no
/// structural state, and the world build reads terrain only —
/// a caller seating lighting re-derives its grid from the
/// snapshot at snapshot boundaries (the client's posture).
/// Changes addressed outside every stored block (a submap the
/// snapshot window does not cover) are dropped, matching the
/// mirror's coverage rule.
pub fn apply_delta_to_snapshot(snapshot: &mut proto::SnapshotT, delta: &proto::DeltaT) {
    if let Some(changes) = &delta.tile_changes {
        if let Some(blocks) = snapshot.blocks.as_mut() {
            for tc in changes {
                let Some(pos) = &tc.pos else { continue };
                let Some(block) = blocks.iter_mut().find(|b| {
                    let o = b.origin.as_ref();
                    o.map(|o| o.z) == Some(pos.z)
                        && pos.x >= o.map(|o| o.x).unwrap_or(0)
                        && pos.x < o.map(|o| o.x).unwrap_or(0) + 12
                        && pos.y >= o.map(|o| o.y).unwrap_or(0)
                        && pos.y < o.map(|o| o.y).unwrap_or(0) + 12
                }) else {
                    continue;
                };
                let o = block.origin.as_ref().expect("block origin");
                let idx = ((pos.y - o.y) * 12 + (pos.x - o.x)) as usize;
                let set = |arr: &mut Option<Vec<u32>>, v: u32| {
                    if let Some(a) = arr {
                        if let Some(slot) = a.get_mut(idx) {
                            *slot = v;
                        }
                    }
                };
                set(&mut block.terrain, tc.terrain);
                set(&mut block.furniture, tc.furniture);
                set(&mut block.trap, tc.trap);
                set(&mut block.top_item, tc.top_item);
                if let Some(a) = &mut block.item_count {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.item_count;
                    }
                }
                if let Some(a) = &mut block.lit {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.lit;
                    }
                }
                if let Some(a) = &mut block.seen {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.seen;
                    }
                }
                if let Some(a) = &mut block.has_graffiti {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.has_graffiti;
                    }
                }
                if let Some(a) = &mut block.outside {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.outside;
                    }
                }
                if let Some(a) = &mut block.light_frac {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.light_frac;
                    }
                }
                if let Some(a) = &mut block.light_tint {
                    let base = idx * 3;
                    if a.len() >= base + 3 {
                        a[base] = tc.light_tint_r;
                        a[base + 1] = tc.light_tint_g;
                        a[base + 2] = tc.light_tint_b;
                    }
                }
                if let Some(a) = &mut block.displayed_field {
                    if let Some(slot) = a.get_mut(idx) {
                        *slot = tc.displayed_field.clone().unwrap_or_default();
                    }
                }
                // Fields: replace this tile's sparse entries,
                // then rebuild the block's field_index (first
                // entry per tile, -1 for none).
                {
                    let fields = block.fields.get_or_insert_with(Vec::new);
                    fields.retain(|f| f.tile_index as usize != idx);
                    if let Some(new_fields) = &tc.fields {
                        for mut f in new_fields.iter().cloned() {
                            f.tile_index = idx as u16;
                            fields.push(f);
                        }
                    }
                    let mut index = vec![-1i32; 144];
                    for (i, f) in fields.iter().enumerate() {
                        let t = f.tile_index as usize;
                        if t < 144 && index[t] < 0 {
                            index[t] = i as i32;
                        }
                    }
                    block.field_index = Some(index);
                }
            }
        }
    }
    if let Some(updates) = &delta.entity_updates {
        let entities = snapshot.entities.get_or_insert_with(Vec::new);
        for e in updates {
            if let Some(slot) = entities.iter_mut().find(|x| x.id == e.id) {
                *slot = e.clone();
            } else {
                entities.push(e.clone());
            }
        }
    }
    if let Some(removed) = &delta.entity_removed {
        if let Some(entities) = &mut snapshot.entities {
            entities.retain(|e| !removed.contains(&e.id));
        }
    }
    if let Some(updates) = &delta.vehicle_updates {
        let vehicles = snapshot.vehicles.get_or_insert_with(Vec::new);
        for v in updates {
            if let Some(slot) = vehicles.iter_mut().find(|x| x.id == v.id) {
                *slot = v.clone();
            } else {
                vehicles.push(v.clone());
            }
        }
    }
    if let Some(removed) = &delta.vehicle_removed {
        if let Some(vehicles) = &mut snapshot.vehicles {
            vehicles.retain(|v| !removed.contains(&v.id));
        }
    }
    if let Some(env) = &delta.environment {
        snapshot.environment = Some(env.clone());
    }
}
