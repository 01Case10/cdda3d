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
//!
//! GI companions (Lane H): a scene built by the live session
//! (`client::LiveWorld::wire_scene`) also carries the wire
//! environment sample and the lighting scene derived from the
//! same snapshot, so a consumer (the windowed app, a live
//! capture) seats Phase 10 sky + GI from the wire data itself.
//! [`WireScene::from_wire`] leaves both `None` — it stays the
//! pure world build the parity gates pin.

use std::sync::Arc;

use mesher::block::{Block, Pass};
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
    /// Furniture fine models for this world (Lane G),
    /// derived from the same snapshot by
    /// [`prop_scene_for_wire`] when the caller has the
    /// voxelizer model table. `None` = the coarse-only wire
    /// world (the pre-Lane-G seating). The frame paths stamp
    /// it through the same `stamp_props_into` pipeline the
    /// fixture prop scenes take.
    pub props: Option<crate::frame::PropScene>,
    /// The wire environment sample this world was built from
    /// (Lane H): the snapshot's piggybacked
    /// `EnvironmentUpdate`, converted by the lighting proto
    /// adapter. A consumer seats its sky scene from it
    /// (`SkyScene::new` / `SkyScene::push` on newer samples).
    /// `None` = the snapshot carried no environment (or the
    /// scene was built by [`WireScene::from_wire`] directly,
    /// which leaves the companions unseated by design).
    pub environment: Option<lighting::GameEnvironment>,
    /// The lighting scene derived from the same snapshot
    /// (Lane H): the game's lit/outside grids via
    /// `LightingScene::from_snapshot`, sky exposure included.
    /// Seated into `CaptureArgs::lighting` by the live
    /// consumers, so both renderers apply the Phase 7 ceiling
    /// rule + Phase 10 GI to the wire world. `None` as for
    /// [`WireScene::environment`].
    pub lighting: Option<crate::frame::LightingScene>,
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
            props: None,
            environment: None,
            lighting: None,
        }
    }

    /// Attach the furniture prop scene derived from this
    /// scene's own snapshot (see [`prop_scene_for_wire`]).
    pub fn with_props(mut self, props: Option<crate::frame::PropScene>) -> Self {
        self.props = props;
        self
    }

    /// Attach the GI companions derived from this scene's
    /// own snapshot (Lane H — see the field docs): the wire
    /// environment sample and the snapshot's lighting scene.
    pub fn with_companions(
        mut self,
        environment: Option<lighting::GameEnvironment>,
        lighting: Option<crate::frame::LightingScene>,
    ) -> Self {
        self.environment = environment;
        self.lighting = lighting;
        self
    }
}

/// Derive the furniture prop scene for a wire world from its
/// snapshot (Lane G): every non-zero furniture cell of every
/// `SubmapBlock` becomes a placement at its world tile
/// (block origins are tile coords of the submap's corner),
/// at level `block.z - snapshot min z`; the furniture def's
/// STRING id resolves the model in the one real voxelizer
/// table (`table`, 351 furniture models) — the Lane F
/// derivation, shared now by the live path instead of being
/// re-implemented per gate.
///
/// PropScene model ids are scene-local: the modelled defs'
/// string ids are sorted and numbered from 1, and placements
/// carry those ids. Furniture with no model in the table is
/// NOT placed (it stays coarse-suppressed per the 9.0
/// contract's other half — no fabricated geometry; it is
/// also not stamped as a coarse cube). Returns `None` when
/// no placement resolves to a model.
pub fn prop_scene_for_wire(
    defs: &proto::TileDefsT,
    snapshot: &proto::SnapshotT,
    table: &crate::materials::MaterialTable,
) -> Option<crate::frame::PropScene> {
    use std::collections::BTreeMap;
    let strings: std::collections::HashMap<u32, String> = defs
        .defs
        .as_ref()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|d| d.kind == proto::TileKind::Furniture)
        .filter_map(|d| d.string_id.clone().map(|s| (d.id, s)))
        .collect();
    let blocks = snapshot.blocks.as_ref().map(Vec::as_slice).unwrap_or(&[]);
    let min_z = blocks
        .iter()
        .map(|b| b.origin.as_ref().map(|o| o.z).unwrap_or(0))
        .min()
        .unwrap_or(0);
    // Modelled furniture defs present in the snapshot:
    // string id -> model (BTreeMap = sorted = deterministic
    // synthetic ids).
    let mut modelled: BTreeMap<String, crate::materials::PropModelData> = BTreeMap::new();
    let mut cells: Vec<(i32, i32, u32, String)> = Vec::new();
    for block in blocks {
        let (ox, oy, oz) = block
            .origin
            .as_ref()
            .map(|o| (o.x, o.y, o.z))
            .unwrap_or((0, 0, 0));
        for (i, &f) in block
            .furniture
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if f == 0 {
                continue;
            }
            let Some(sid) = strings.get(&f) else {
                continue;
            };
            let Some(model) = table.get(sid).and_then(|m| m.model.as_ref()) else {
                continue;
            };
            modelled
                .entry(sid.clone())
                .or_insert_with(|| model.clone());
            cells.push((
                ox + (i % 12) as i32,
                oy + (i / 12) as i32,
                (oz - min_z) as u32,
                sid.clone(),
            ));
        }
    }
    if cells.is_empty() {
        return None;
    }
    let ids: std::collections::HashMap<String, u32> = modelled
        .keys()
        .enumerate()
        .map(|(i, s)| (s.clone(), (i + 1) as u32))
        .collect();
    let models: Vec<(u32, crate::materials::PropModelData)> = modelled
        .into_iter()
        .enumerate()
        .map(|(i, (_s, m))| ((i + 1) as u32, m))
        .collect();
    let placements = cells
        .into_iter()
        .map(|(x, y, level, sid)| crate::snapshot_world::FurniturePlacement {
            x,
            y,
            level,
            furn_id: ids[&sid],
            quarter_turns: 0,
        })
        .collect();
    Some(crate::frame::PropScene { models, placements })
}

/// The live wire palette (Phase 11 follow-up, Lane G): one
/// deterministic colour per terrain def, keyed by the def's
/// STRING id — the only join key that means the same thing on
/// every side of the wire (wire ids are per-kind and renumber
/// between tables). The colour is an FNV-1a hash of the string
/// id mapped into a constrained HSL band (saturation
/// 0.38..0.68, lightness 0.36..0.60), so colours are stable
/// across runs and machines, distinct per def, and never
/// garish or near-black/near-white.
///
/// This is honest colour-CODING, not the game's colours: it
/// exists so a live wire world is readable (grass vs road vs
/// floor vs wall are different hues) until semantic colours
/// land — CDDA's own per-def `color` field is not currently
/// imported into the wire def table, so there is nothing
/// truer to show yet. When that import lands, this palette
/// is the thing it replaces.
pub fn palette_color(string_id: &str) -> [f32; 4] {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in string_id.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let hue = (hash % 360) as f32;
    let sat = 0.38 + ((hash >> 9) % 100) as f32 / 100.0 * 0.30;
    let light = 0.36 + ((hash >> 17) % 100) as f32 / 100.0 * 0.24;
    let [r, g, b] = hsl_to_rgb(hue, sat, light);
    [r, g, b, 1.0]
}

fn hsl_to_rgb(hue: f32, sat: f32, light: f32) -> [f32; 3] {
    let c = (1.0 - (2.0 * light - 1.0).abs()) * sat;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = light - c / 2.0;
    let (r, g, b) = match hue {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

/// The `block_for` resolver form of [`palette_color`] for a
/// whole def table: terrain wire id -> that def's palette
/// block (the block keeps the wire id; only the colour is
/// derived). Id 0 ("none") resolves to AIR, matching the
/// gates' resolver convention; a terrain id with no string
/// id (a malformed table) hashes its `#<id>` placeholder, so
/// it still gets a stable colour instead of a hole.
pub fn palette_resolver(defs: &proto::TileDefsT) -> impl Fn(u32) -> Block + '_ {
    let mut colors: std::collections::HashMap<u32, [f32; 4]> =
        std::collections::HashMap::new();
    if let Some(list) = &defs.defs {
        for d in list {
            if d.kind == proto::TileKind::Terrain {
                let key = d
                    .string_id
                    .clone()
                    .unwrap_or_else(|| format!("#{}", d.id));
                colors.insert(d.id, palette_color(&key));
            }
        }
    }
    move |id: u32| {
        if id == 0 {
            return Block::AIR;
        }
        let color = colors
            .get(&id)
            .copied()
            .unwrap_or_else(|| palette_color(&format!("#{id}")));
        Block {
            id,
            color,
            pass: Pass::Opaque,
            tex_layers: None,
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
