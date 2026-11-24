//! World state + Snapshot builder (slices S1-S3).
//!
//! The `World` holds tile state per submap block in *registry*
//! wire ids (per-kind, from [`crate::defs::DefRegistry`]), the
//! entity/vehicle lists, and a dirty-submap set — the shape the
//! S2 turn loop needs: mutate world -> mark submap dirty -> diff
//! -> Delta.
//!
//! S3 state: a game clock (calendar turn, 1 s/turn — see
//! [`crate::environment`]), the current weather preset, the
//! scene's weather script and light sources, a visibility cache
//! (per-block lit/seen/frac/tint/outside, recomputed by
//! [`crate::visibility`] after any world/player/time change),
//! and the tile memory (terrain/furniture of every tile ever
//! visible, exported as MEMORIZED memory blocks once a tile
//! leaves visibility — the fork exporter's shape).
//!
//! The S1 placeholder rules are gone: lit/seen are no longer a
//! z-level rule, `outside` is always computed and always
//! emitted (the scene `emit_outside` key is vestigial — the real
//! exporter always writes the array), and the Snapshot's
//! environment is the computed EnvironmentUpdate, not
//! `sample_environment()`.
//!
//! `displayed_field` follows the sample rule (§3.2): fire is
//! displayed over other fields; otherwise the highest intensity
//! wins; ties keep the first entry.

use std::collections::{HashMap, HashSet};

use proto::{
    DeltaT, DisplayedFieldT, EntityT, EnvironmentUpdateT, Facing, FieldOnTileT, LightUpdateT,
    LitLevel, MessageT, SnapshotT, SubmapBlockT, TileChangeT, Vec3iT, VehiclePartStateT, VehicleT,
};

use crate::defs::{DefRegistry, Kind};
use crate::environment::{self, WeatherState};
use crate::rules::PendingAction;
use crate::scenes::{LightSpec, Scene, WeatherStep, SUBMAP};
use crate::visibility::{self, BlockVis};

pub const TILES: usize = SUBMAP * SUBMAP;

#[derive(Debug, Clone, Default)]
pub struct TileState {
    pub terrain: u32,
    pub furniture: u32,
    pub trap: u32,
    /// (field def id, intensity) in wire order.
    pub fields: Vec<(u32, u8)>,
    pub top_item: u32,
    pub item_count: u16,
    pub graffiti: bool,
}

#[derive(Debug, Clone)]
pub struct BlockState {
    pub origin: Vec3iT,
    pub tiles: Vec<TileState>,
}

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub blocks: Vec<BlockState>,
    /// (origin x, y, z) of submaps changed since the last send.
    dirty: HashSet<(i32, i32, i32)>,
    pub entities: Vec<EntityT>,
    pub vehicles: Vec<VehicleT>,
    pub player_pos: Option<[i32; 3]>,
    pub view_origin: Vec3iT,
    pub view_radius: u32,
    pub environment: bool,
    /// Game clock: calendar turns since turn_zero (1 s/turn).
    pub game_turn: i64,
    /// Current weather preset (resolved numerics).
    pub weather: WeatherState,
    /// Scene weather script, in file order.
    weather_script: Vec<WeatherStep>,
    /// Scene-declared light sources.
    lights: Vec<LightSpec>,
    /// Visibility cache, parallel to `blocks` (S3 recompute).
    vis: Vec<BlockVis>,
    /// Tiles visible at the last recompute.
    visible: HashSet<[i32; 3]>,
    /// Tile memory: pos -> (terrain, furniture) as last seen.
    memory: HashMap<[i32; 3], (u32, u32)>,
    /// Calendar turn at which an EnvironmentUpdate was last
    /// sent (piggybacked or standalone).
    last_env_turn: i64,
    /// Weather changed since the last EnvironmentUpdate send.
    env_dirty: bool,
    /// Entity id of the player spawn (`type = "player"`), if the
    /// scene has one. The player entity is the movement actor.
    pub player_id: Option<u64>,
    /// Seq of the last message sent (0 = nothing sent yet). Every
    /// Snapshot/Delta send advances it by exactly 1 (proto::sync).
    pub seq: u64,
    /// Current epoch. Starts at 1; a map shift (the player crosses
    /// into a different submap than the one they were in at the
    /// last Snapshot) bumps it and forces a fresh Snapshot.
    pub epoch: u64,
    /// Wire form of the blocks as last sent — the diff baseline,
    /// exactly like the fork exporter diffing dirty submaps
    /// against the client's known state.
    last_blocks: Vec<SubmapBlockT>,
    /// Entities as last sent — the entity-diff baseline.
    last_entities: Vec<EntityT>,
    /// Submap index (x/12, y/12, z) the player was in at the last
    /// Snapshot; `None` until the first Snapshot is sent.
    last_snapshot_player_submap: Option<(i32, i32, i32)>,
    /// A pending Open/Close awaiting its direction key (S2 rules).
    pub pending: Option<PendingAction>,
}

fn v3(x: i32, y: i32, z: i32) -> Vec3iT {
    Vec3iT { x, y, z }
}

fn idx(x: usize, y: usize) -> usize {
    y * SUBMAP + x
}

impl World {
    pub fn from_scene(reg: &DefRegistry, scene: &Scene) -> Result<World, String> {
        // One BlockState per submap: a z-section may span several
        // submaps (S2), split here into 12x12 blocks with absolute
        // origins, row-major over the section.
        let mut blocks: Vec<BlockState> = Vec::new();
        for zs in &scene.zsections {
            let layer = |rows: &[String], x: usize, y: usize| -> char {
                rows[y]
                    .chars()
                    .nth(x)
                    .expect("layer rows are width-checked")
            };
            for by in 0..zs.height / SUBMAP {
                for bx in 0..zs.width / SUBMAP {
                    let mut tiles = vec![TileState::default(); TILES];
                    for ly in 0..SUBMAP {
                        for lx in 0..SUBMAP {
                            let (gx, gy) = (bx * SUBMAP + lx, by * SUBMAP + ly);
                            let t = &mut tiles[idx(lx, ly)];
                            let c = layer(&zs.map, gx, gy);
                            let sid = zs.legend.get(&c).expect("map chars are legend-checked");
                            t.terrain = reg.id_of(Kind::Terrain, sid);
                            if t.terrain == 0 {
                                return Err(format!(
                                    "scene '{}': legend char '{c}' -> '{sid}' is not a Terrain def",
                                    scene.name
                                ));
                            }
                            if let Some(furn) = &zs.furn {
                                let fc = layer(furn, gx, gy);
                                if fc != '.' {
                                    let fsid =
                                        zs.legend.get(&fc).expect("furn chars are legend-checked");
                                    t.furniture = reg.id_of(Kind::Furniture, fsid);
                                    if t.furniture == 0 {
                                        return Err(format!(
                                            "scene '{}': furn char '{fc}' -> '{fsid}' is not a Furniture def",
                                            scene.name
                                        ));
                                    }
                                }
                            }
                            if let Some(trap) = &zs.trap {
                                let tc = layer(trap, gx, gy);
                                if tc != '.' {
                                    let tsid =
                                        zs.legend.get(&tc).expect("trap chars are legend-checked");
                                    t.trap = reg.id_of(Kind::Trap, tsid);
                                    if t.trap == 0 {
                                        return Err(format!(
                                            "scene '{}': trap char '{tc}' -> '{tsid}' is not a Trap def",
                                            scene.name
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    blocks.push(BlockState {
                        origin: v3((bx * SUBMAP) as i32, (by * SUBMAP) as i32, zs.z),
                        tiles,
                    });
                }
            }
        }

        // Sparse per-tile extras, addressed by absolute pos: the
        // block whose 12x12 footprint contains the pos.
        fn place(
            blocks: &[BlockState],
            scene_name: &str,
            pos: [i32; 3],
            what: &str,
        ) -> Result<(usize, usize), String> {
            let b = blocks
                .iter()
                .position(|b| {
                    b.origin.z == pos[2]
                        && (b.origin.x..b.origin.x + SUBMAP as i32).contains(&pos[0])
                        && (b.origin.y..b.origin.y + SUBMAP as i32).contains(&pos[1])
                })
                .ok_or_else(|| {
                    format!(
                        "scene '{scene_name}': {what} at {pos:?} is outside the scene's submaps"
                    )
                })?;
            let o = &blocks[b].origin;
            Ok((b, idx((pos[0] - o.x) as usize, (pos[1] - o.y) as usize)))
        }
        for f in &scene.fields {
            let (b, i) = place(&blocks, &scene.name, f.pos, "field")?;
            let id = reg.id_of(Kind::Field, &f.type_id);
            if id == 0 {
                return Err(format!(
                    "scene '{}': field type '{}' is not a Field def",
                    scene.name, f.type_id
                ));
            }
            blocks[b].tiles[i].fields.push((id, f.intensity));
        }
        for it in &scene.items {
            let (b, i) = place(&blocks, &scene.name, it.pos, "items")?;
            let id = reg.id_of(Kind::Item, &it.top);
            if id == 0 {
                return Err(format!(
                    "scene '{}': item '{}' is not an Item def",
                    scene.name, it.top
                ));
            }
            blocks[b].tiles[i].top_item = id;
            blocks[b].tiles[i].item_count = it.count;
        }
        for g in &scene.graffiti {
            let (b, i) = place(&blocks, &scene.name, *g, "graffiti")?;
            blocks[b].tiles[i].graffiti = true;
        }

        // Entities in file order, ids 1.. — author the player
        // first (the real adapter emits the avatar first).
        let mut entities = Vec::new();
        let mut player_id: Option<u64> = None;
        for (n, s) in scene.spawns.iter().enumerate() {
            let (type_def, sprite) = if s.type_id == "player" {
                (0, s.sprite.clone().unwrap_or_else(|| "player".to_string()))
            } else {
                let id = reg.id_of(Kind::Monster, &s.type_id);
                if id == 0 {
                    return Err(format!(
                        "scene '{}': spawn type '{}' is not a Monster def",
                        scene.name, s.type_id
                    ));
                }
                (id, s.sprite.clone().unwrap_or_else(|| s.type_id.clone()))
            };
            let facing = match s.facing.as_str() {
                "NONE" => Facing::NONE,
                "LEFT" => Facing::LEFT,
                "RIGHT" => Facing::RIGHT,
                other => {
                    return Err(format!(
                        "scene '{}': spawn facing '{other}' is not NONE/LEFT/RIGHT",
                        scene.name
                    ))
                }
            };
            if s.type_id == "player" {
                player_id = Some((n + 1) as u64);
            }
            // Generated table types are #[non_exhaustive] outside
            // the proto crate: Default + field assignment.
            let mut e = EntityT::default();
            e.id = (n + 1) as u64;
            e.type_def = type_def;
            e.sprite_id = Some(sprite);
            e.pos = Some(v3(s.pos[0], s.pos[1], s.pos[2]));
            e.facing = facing;
            e.overlays = if s.overlays.is_empty() {
                None
            } else {
                Some(s.overlays.clone())
            };
            e.hp_frac = s.hp_frac;
            e.flags = s.flags;
            entities.push(e);
        }

        let mut vehicles = Vec::new();
        for v in &scene.vehicles {
            let parts: Vec<VehiclePartStateT> = scene
                .vparts
                .iter()
                .filter(|p| p.vehicle == v.id)
                .map(|p| -> Result<VehiclePartStateT, String> {
                    let id = reg.id_of(Kind::VehiclePart, &p.type_id);
                    if id == 0 {
                        return Err(format!(
                            "scene '{}': vpart type '{}' is not a VehiclePart def",
                            scene.name, p.type_id
                        ));
                    }
                    let mut part = VehiclePartStateT::default();
                    part.vpart = id;
                    part.abs_pos = Some(v3(p.pos[0], p.pos[1], p.pos[2]));
                    part.mount = Some(v3(p.mount[0], p.mount[1], p.mount[2]));
                    part.hp_frac = p.hp_frac;
                    part.flags = p.flags;
                    Ok(part)
                })
                .collect::<Result<_, _>>()?;
            let mut veh = VehicleT::default();
            veh.id = v.id;
            veh.name = if v.name.is_empty() {
                None
            } else {
                Some(v.name.clone())
            };
            veh.face_deg = v.face_deg;
            veh.parts = if parts.is_empty() { None } else { Some(parts) };
            vehicles.push(veh);
        }

        // S3 clock: the scene's `turn` key wins; else the
        // `time` preset (time-of-day on day 0); an empty time
        // means noon, matching the old placeholder's assumption.
        let game_turn = if let Some(t) = scene.turn {
            t
        } else if scene.time.is_empty() {
            environment::time_preset_turn("noon").expect("noon preset exists")
        } else {
            environment::time_preset_turn(&scene.time).ok_or_else(|| {
                format!(
                    "scene '{}': unknown time preset '{}' (use midnight/dawn/noon/dusk or turn = <int>)",
                    scene.name, scene.time
                )
            })?
        };
        let weather_name = if scene.weather.is_empty() {
            "clear"
        } else {
            scene.weather.as_str()
        };
        let weather = environment::weather_preset(weather_name).ok_or_else(|| {
            format!(
                "scene '{}': unknown weather preset '{weather_name}'",
                scene.name
            )
        })?;
        for step in &scene.weather_script {
            if environment::weather_preset(&step.weather).is_none() {
                return Err(format!(
                    "scene '{}': weather_script names unknown preset '{}'",
                    scene.name, step.weather
                ));
            }
        }

        let dirty = blocks
            .iter()
            .map(|b| (b.origin.x, b.origin.y, b.origin.z))
            .collect();
        let mut world = World {
            name: scene.name.clone(),
            blocks,
            dirty,
            entities,
            vehicles,
            player_pos: scene.player,
            view_origin: v3(
                scene.view_origin[0],
                scene.view_origin[1],
                scene.view_origin[2],
            ),
            view_radius: scene.view_radius,
            environment: scene.environment,
            game_turn,
            weather,
            weather_script: scene.weather_script.clone(),
            lights: scene.lights.clone(),
            vis: Vec::new(),
            visible: HashSet::new(),
            memory: HashMap::new(),
            last_env_turn: game_turn,
            env_dirty: false,
            player_id,
            seq: 0,
            epoch: 1,
            last_blocks: Vec::new(),
            last_entities: Vec::new(),
            last_snapshot_player_submap: None,
            pending: None,
        };
        world.recompute_visibility(reg);
        Ok(world)
    }

    /// Recompute the visibility cache (lit/seen/frac/tint/
    /// outside for every block) and fold newly-visible tiles
    /// into tile memory. Called after any world, player, clock
    /// or weather change.
    pub fn recompute_visibility(&mut self, reg: &DefRegistry) {
        let viewpoint = self.player_pos();
        let spectator = [self.view_origin.x, self.view_origin.y, self.view_origin.z];
        let result = visibility::compute(
            reg,
            &self.blocks,
            &self.lights,
            viewpoint,
            spectator,
            self.game_turn,
            &self.weather,
        );
        self.vis = result.blocks;
        self.visible = result.visible;
        for pos in &self.visible {
            if let Some((b, i)) = self.tile_at(*pos) {
                let t = &self.blocks[b].tiles[i];
                self.memory.insert(*pos, (t.terrain, t.furniture));
            }
        }
    }

    /// The current EnvironmentUpdate (this world's seq/epoch are
    /// filled in by the caller-visible builders below; this is
    /// the pure value for a given seq/epoch).
    pub fn environment_update(&self, seq: u64, epoch: u64) -> EnvironmentUpdateT {
        environment::build_environment(seq, epoch, self.game_turn, &self.weather)
    }

    /// Is an EnvironmentUpdate due at the current turn? (Only
    /// scenes with `environment = true` ever send one.)
    fn env_due(&self) -> bool {
        self.environment
            && (self.env_dirty
                || self.game_turn - self.last_env_turn >= environment::ENV_CADENCE_SECONDS)
    }

    /// Submaps changed since the last Snapshot/Delta send. S1:
    /// every submap is dirty until the first Snapshot.
    pub fn dirty_submaps(&self) -> &HashSet<(i32, i32, i32)> {
        &self.dirty
    }

    pub fn mark_dirty(&mut self, origin: (i32, i32, i32)) {
        self.dirty.insert(origin);
    }

    /// Build a Snapshot of the current state with the given
    /// seq/epoch. Pure: does not touch the send baseline (the
    /// mirror-invariant gate uses it to ask "what would a fresh
    /// Snapshot say right now?"). Reads the visibility cache —
    /// refreshed by `snapshot()`, `flush_turn()`,
    /// `advance_clock()` and `set_weather()`. Note the S2
    /// simplification: a Snapshot contains ALL scene submaps —
    /// the scene extent is the bubble; there is no view-window
    /// culling yet.
    pub fn build_snapshot(&self, reg: &DefRegistry, seq: u64, epoch: u64) -> SnapshotT {
        let blocks: Vec<SubmapBlockT> = self
            .blocks
            .iter()
            .enumerate()
            .map(|(bi, b)| self.build_block(reg, b, &self.vis[bi]))
            .collect();
        let mut snap = SnapshotT::default();
        snap.view_origin = Some(self.view_origin.clone());
        snap.view_radius = self.view_radius;
        snap.blocks = Some(blocks);
        snap.memory_blocks = self.build_memory_blocks();
        snap.entities = if self.entities.is_empty() {
            None
        } else {
            Some(self.entities.clone())
        };
        snap.vehicles = if self.vehicles.is_empty() {
            None
        } else {
            Some(self.vehicles.clone())
        };
        snap.seq = seq;
        snap.epoch = epoch;
        snap.environment = if self.environment {
            Some(Box::new(self.environment_update(seq, epoch)))
        } else {
            None
        };
        snap
    }

    /// Memory blocks in the fork exporter's shape
    /// (voxel_exporter.cpp build_memory_blocks): one block per
    /// submap holding at least one memorized-but-not-currently-
    /// visible tile; terrain/furniture only, lit MEMORIZED on
    /// those tiles and BLANK elsewhere in the block, seen 0,
    /// outside all false, no frac/tint. `None` when nothing is
    /// memorized out of sight.
    fn build_memory_blocks(&self) -> Option<Vec<SubmapBlockT>> {
        let mut by_origin: HashMap<(i32, i32, i32), Vec<([i32; 3], (u32, u32))>> = HashMap::new();
        for (pos, tf) in &self.memory {
            if self.visible.contains(pos) {
                continue;
            }
            by_origin
                .entry(submap_origin(*pos))
                .or_default()
                .push((*pos, *tf));
        }
        if by_origin.is_empty() {
            return None;
        }
        let mut origins: Vec<(i32, i32, i32)> = by_origin.keys().copied().collect();
        origins.sort();
        let mut blocks = Vec::with_capacity(origins.len());
        for origin in origins {
            let mut block = SubmapBlockT::default();
            block.origin = Some(v3(origin.0, origin.1, origin.2));
            let mut terrain = vec![0u32; TILES];
            let mut furniture = vec![0u32; TILES];
            let mut lit = vec![LitLevel::BLANK; TILES];
            for (pos, (ter, furn)) in &by_origin[&origin] {
                let i = idx((pos[0] - origin.0) as usize, (pos[1] - origin.1) as usize);
                terrain[i] = *ter;
                furniture[i] = *furn;
                lit[i] = LitLevel::MEMORIZED;
            }
            block.terrain = Some(terrain);
            block.furniture = Some(furniture);
            block.trap = Some(vec![0; TILES]);
            block.fields = Some(Vec::new());
            block.field_index = Some(vec![-1; TILES]);
            block.displayed_field = Some(vec![
                DisplayedFieldT {
                    field_type: 0,
                    intensity: 0
                };
                TILES
            ]);
            block.top_item = Some(vec![0; TILES]);
            block.item_count = Some(vec![0; TILES]);
            block.lit = Some(lit);
            block.seen = Some(vec![0; TILES]);
            block.has_graffiti = Some(vec![false; TILES]);
            block.outside = Some(vec![false; TILES]);
            block.light_frac = None;
            block.light_tint = None;
            blocks.push(block);
        }
        Some(blocks)
    }

    /// Record a Snapshot as sent: it becomes the diff baseline,
    /// the dirty set clears, and the player's submap at this
    /// Snapshot is remembered for shift detection. A piggybacked
    /// environment counts as sent at the current turn.
    fn record_sent(&mut self, snap: &SnapshotT) {
        self.seq = snap.seq;
        self.epoch = snap.epoch;
        self.last_blocks = snap.blocks.clone().unwrap_or_default();
        self.last_entities = snap.entities.clone().unwrap_or_default();
        self.last_snapshot_player_submap = self.player_pos().map(submap_index);
        self.dirty.clear();
        if snap.environment.is_some() {
            self.last_env_turn = self.game_turn;
            self.env_dirty = false;
        }
    }

    /// The initial Snapshot send (seq 1, epoch 1 on a fresh
    /// world) — the all-dirty first send.
    pub fn snapshot(&mut self, reg: &DefRegistry) -> SnapshotT {
        self.recompute_visibility(reg);
        let snap = self.build_snapshot(reg, self.seq + 1, self.epoch);
        self.record_sent(&snap);
        snap
    }

    // -----------------------------------------------------------------
    // Turn-loop support (S2): tile/entity access for the rules,
    // and the diff-shaped flush.
    // -----------------------------------------------------------------

    /// (block index, tile index) for an absolute tile pos, if the
    /// scene has a submap containing it.
    pub fn tile_at(&self, pos: [i32; 3]) -> Option<(usize, usize)> {
        let b = self.blocks.iter().position(|b| {
            b.origin.z == pos[2]
                && (b.origin.x..b.origin.x + SUBMAP as i32).contains(&pos[0])
                && (b.origin.y..b.origin.y + SUBMAP as i32).contains(&pos[1])
        })?;
        let o = &self.blocks[b].origin;
        Some((b, idx((pos[0] - o.x) as usize, (pos[1] - o.y) as usize)))
    }

    /// Terrain wire id at an absolute pos.
    pub fn terrain_at(&self, pos: [i32; 3]) -> Option<u32> {
        let (b, i) = self.tile_at(pos)?;
        Some(self.blocks[b].tiles[i].terrain)
    }

    /// Furniture wire id at an absolute pos (0 = none).
    pub fn furniture_at(&self, pos: [i32; 3]) -> Option<u32> {
        let (b, i) = self.tile_at(pos)?;
        Some(self.blocks[b].tiles[i].furniture)
    }

    /// Set the terrain at an absolute pos and mark its submap
    /// dirty. Returns false if the pos is outside the scene.
    pub fn set_terrain(&mut self, pos: [i32; 3], terrain: u32) -> bool {
        let Some((b, i)) = self.tile_at(pos) else {
            return false;
        };
        self.blocks[b].tiles[i].terrain = terrain;
        let o = &self.blocks[b].origin;
        self.mark_dirty((o.x, o.y, o.z));
        true
    }

    /// The player's current position: the player entity's pos
    /// (the entity list is the source of truth once spawned),
    /// falling back to the scene's `player` key.
    pub fn player_pos(&self) -> Option<[i32; 3]> {
        if let Some(id) = self.player_id {
            if let Some(e) = self.entities.iter().find(|e| e.id == id) {
                if let Some(p) = &e.pos {
                    return Some([p.x, p.y, p.z]);
                }
            }
        }
        self.player_pos
    }

    /// Move the player entity to `pos`, updating facing from the
    /// horizontal step: LEFT if dx < 0, RIGHT if dx > 0, unchanged
    /// if dx == 0 (§3.2). No submap is dirtied — entities are not
    /// tile state; the entity diff carries the move.
    pub fn set_player_pos(&mut self, pos: [i32; 3]) {
        if let Some(id) = self.player_id {
            if let Some(e) = self.entities.iter_mut().find(|e| e.id == id) {
                let dx = e.pos.as_ref().map_or(0, |p| pos[0] - p.x);
                if dx < 0 {
                    e.facing = Facing::LEFT;
                } else if dx > 0 {
                    e.facing = Facing::RIGHT;
                }
                e.pos = Some(v3(pos[0], pos[1], pos[2]));
            }
        }
        self.player_pos = Some(pos);
    }

    /// True if a creature entity (not the player) occupies `pos`.
    pub fn creature_at(&self, pos: [i32; 3]) -> bool {
        self.entities.iter().any(|e| {
            Some(e.id) != self.player_id
                && e.pos
                    .as_ref()
                    .is_some_and(|p| p.x == pos[0] && p.y == pos[1] && p.z == pos[2])
        })
    }

    /// Apply any weather-script entries scheduled at or before
    /// the current turn. A change marks the environment dirty
    /// (an on-change EnvironmentUpdate is due).
    fn apply_script_due(&mut self) {
        let due: Vec<String> = self
            .weather_script
            .iter()
            .filter(|s| s.turn <= self.game_turn)
            .map(|s| s.weather.clone())
            .collect();
        if due.is_empty() {
            return;
        }
        self.weather_script.retain(|s| s.turn > self.game_turn);
        for name in due {
            if let Some(w) = environment::weather_preset(&name) {
                if w.preset != self.weather.preset {
                    self.weather = w;
                    self.env_dirty = true;
                }
            }
        }
    }

    /// Flush one resolved turn (S2 shape, S3 contents): the
    /// clock advances 1 s (do_turn order: `calendar::turn +=
    /// 1_turns` happens inside the turn), due weather-script
    /// entries apply, visibility recomputes, and the diff
    /// against the last-sent baseline is composed into the
    /// turn's message flush (plan §2.7: a flush is a Delta plus
    /// optional LightUpdate / EnvironmentUpdate messages).
    ///
    /// If the player is now in a different submap than at the
    /// last Snapshot (a map shift), the flush is instead a fresh
    /// Snapshot with epoch + 1. Seq advances by exactly 1 per
    /// message, in flush order.
    pub fn flush_turn(&mut self, reg: &DefRegistry) -> Vec<MessageT> {
        // No Snapshot sent yet: the first flush IS the Snapshot.
        if self.last_blocks.is_empty() {
            return vec![MessageT::Snapshot(Box::new(self.snapshot(reg)))];
        }
        self.game_turn += 1;
        self.apply_script_due();
        self.recompute_visibility(reg);
        // Map shift?
        if let (Some(cur), Some(last)) = (
            self.player_pos().map(submap_index),
            self.last_snapshot_player_submap,
        ) {
            if cur != last {
                let snap = self.build_snapshot(reg, self.seq + 1, self.epoch + 1);
                self.record_sent(&snap);
                return vec![MessageT::Snapshot(Box::new(snap))];
            }
        }
        self.compose_flush(reg, true)
    }

    /// Fast-forward the clock by `seconds` (the T6 mechanism):
    /// step through every environment boundary in the interval
    /// — weather-script entries and the 10-minute time-only
    /// cadence — emitting one standalone EnvironmentUpdate per
    /// boundary (on-change sends reset the cadence clock, as in
    /// the per-turn path), then recompute visibility once at the
    /// end and flush the accumulated light diff. Intermediate
    /// light states are skipped by design: this is a harness
    /// fast-forward, not a per-turn simulation.
    pub fn advance_clock(&mut self, reg: &DefRegistry, seconds: i64) -> Vec<MessageT> {
        let mut msgs = Vec::new();
        if self.last_blocks.is_empty() {
            msgs.push(MessageT::Snapshot(Box::new(self.snapshot(reg))));
        }
        let target = self.game_turn + seconds.max(0);
        loop {
            let next_script = self
                .weather_script
                .iter()
                .map(|s| s.turn)
                .filter(|t| *t > self.game_turn && *t <= target)
                .min();
            let next_cadence = if self.environment {
                let t = self.last_env_turn + environment::ENV_CADENCE_SECONDS;
                (t > self.game_turn && t <= target).then_some(t)
            } else {
                None
            };
            let Some(boundary) = [next_script, next_cadence].into_iter().flatten().min() else {
                break;
            };
            self.game_turn = boundary;
            self.apply_script_due();
            if self.env_due() {
                self.seq += 1;
                let env = self.environment_update(self.seq, self.epoch);
                self.last_env_turn = self.game_turn;
                self.env_dirty = false;
                msgs.push(MessageT::EnvironmentUpdate(Box::new(env)));
            }
        }
        self.game_turn = target;
        self.apply_script_due();
        self.recompute_visibility(reg);
        msgs.extend(self.compose_flush(reg, false));
        msgs
    }

    /// Change the weather preset now (the sim-server/test
    /// counterpart of a script entry): marks the environment
    /// dirty and flushes the relight + the on-change
    /// EnvironmentUpdate.
    pub fn set_weather(&mut self, reg: &DefRegistry, name: &str) -> Result<Vec<MessageT>, String> {
        let w = environment::weather_preset(name)
            .ok_or_else(|| format!("unknown weather preset '{name}'"))?;
        if w.preset != self.weather.preset {
            self.weather = w;
            self.env_dirty = true;
        }
        self.recompute_visibility(reg);
        Ok(self.compose_flush(reg, false))
    }

    /// The shared diff/compose step: diff every block against
    /// the last-sent baseline (S3 recomputes visibility for the
    /// whole scene each flush, so the diff — not the dirty set —
    /// is authoritative; dawn/dusk relights in untouched submaps
    /// are caught by construction), diff entities by id, and
    /// compose the flush:
    ///
    /// - Any structural tile change (terrain/furniture/trap/
    ///   fields/displayed/items/graffiti/outside) or any entity
    ///   change -> one Delta carrying ALL tile changes (light
    ///   changes fold in as TileChanges).
    /// - Otherwise, light-only tile changes -> one LightUpdate
    ///   per affected submap (full 144-tile arrays) instead of a
    ///   Delta.
    /// - A due EnvironmentUpdate rides the Delta when one is
    ///   emitted; otherwise it is appended standalone.
    /// - `emit_empty_delta` (the per-turn path): a turn with no
    ///   changes at all still yields one empty Delta (the S2
    ///   contract — the client knows the turn happened).
    fn compose_flush(&mut self, reg: &DefRegistry, emit_empty_delta: bool) -> Vec<MessageT> {
        let cur_blocks: Vec<SubmapBlockT> = self
            .blocks
            .iter()
            .enumerate()
            .map(|(bi, b)| self.build_block(reg, b, &self.vis[bi]))
            .collect();
        let last_by_origin: HashMap<(i32, i32, i32), &SubmapBlockT> = self
            .last_blocks
            .iter()
            .map(|b| {
                let o = b.origin.as_ref().expect("blocks always carry an origin");
                ((o.x, o.y, o.z), b)
            })
            .collect();

        let mut all_changes: Vec<TileChangeT> = Vec::new();
        let mut any_structural = false;
        let mut light_submaps: Vec<(i32, i32, i32)> = Vec::new();
        for cb in &cur_blocks {
            let o = cb.origin.as_ref().expect("blocks always carry an origin");
            let key = (o.x, o.y, o.z);
            let Some(lb) = last_by_origin.get(&key) else {
                continue;
            };
            let mut submap_light_only = false;
            for i in 0..TILES {
                if let Some((tc, light_only)) = diff_tile(lb, cb, i) {
                    if light_only {
                        submap_light_only = true;
                    } else {
                        any_structural = true;
                    }
                    all_changes.push(tc);
                }
            }
            if submap_light_only {
                light_submaps.push(key);
            }
        }

        let mut entity_updates: Vec<EntityT> = Vec::new();
        for e in &self.entities {
            match self.last_entities.iter().find(|old| old.id == e.id) {
                Some(old) if old == e => {}
                _ => entity_updates.push(e.clone()),
            }
        }
        let entity_removed: Vec<u64> = self
            .last_entities
            .iter()
            .map(|old| old.id)
            .filter(|id| !self.entities.iter().any(|e| e.id == *id))
            .collect();
        let entity_changed = !entity_updates.is_empty() || !entity_removed.is_empty();

        self.dirty.clear();
        self.last_blocks = cur_blocks;
        self.last_entities = self.entities.clone();

        let env_due = self.env_due();
        let mut msgs: Vec<MessageT> = Vec::new();
        if any_structural || entity_changed || (all_changes.is_empty() && emit_empty_delta) {
            self.seq += 1;
            let mut delta = DeltaT::default();
            delta.seq = self.seq;
            delta.epoch = self.epoch;
            delta.tile_changes = Some(all_changes);
            delta.entity_updates = Some(entity_updates);
            delta.entity_removed = Some(entity_removed);
            delta.vehicle_updates = None;
            delta.vehicle_removed = None;
            if env_due {
                delta.environment = Some(Box::new(self.environment_update(self.seq, self.epoch)));
                self.last_env_turn = self.game_turn;
                self.env_dirty = false;
            }
            msgs.push(MessageT::Delta(Box::new(delta)));
        } else if !all_changes.is_empty() {
            // Light-only flush: one LightUpdate per affected
            // submap, full arrays from the current blocks.
            for key in &light_submaps {
                let cb = self
                    .last_blocks
                    .iter()
                    .find(|b| {
                        let o = b.origin.as_ref().expect("origin");
                        (o.x, o.y, o.z) == *key
                    })
                    .expect("light submap is in the baseline");
                self.seq += 1;
                let mut lu = LightUpdateT::default();
                lu.seq = self.seq;
                lu.epoch = self.epoch;
                lu.origin = cb.origin.clone();
                lu.lit = cb.lit.clone();
                lu.seen = cb.seen.clone();
                lu.light_frac = cb.light_frac.clone();
                lu.light_tint = cb.light_tint.clone();
                msgs.push(MessageT::LightUpdate(Box::new(lu)));
            }
            if env_due {
                self.seq += 1;
                let env = self.environment_update(self.seq, self.epoch);
                self.last_env_turn = self.game_turn;
                self.env_dirty = false;
                msgs.push(MessageT::EnvironmentUpdate(Box::new(env)));
            }
        } else if env_due {
            self.seq += 1;
            let env = self.environment_update(self.seq, self.epoch);
            self.last_env_turn = self.game_turn;
            self.env_dirty = false;
            msgs.push(MessageT::EnvironmentUpdate(Box::new(env)));
        }
        msgs
    }

    fn build_block(&self, reg: &DefRegistry, b: &BlockState, vis: &BlockVis) -> SubmapBlockT {
        let mut terrain = Vec::with_capacity(TILES);
        let mut furniture = Vec::with_capacity(TILES);
        let mut trap = Vec::with_capacity(TILES);
        let mut fields: Vec<FieldOnTileT> = Vec::new();
        let mut field_index = vec![-1i32; TILES];
        let mut displayed = vec![
            DisplayedFieldT {
                field_type: 0,
                intensity: 0
            };
            TILES
        ];
        let mut top_item = Vec::with_capacity(TILES);
        let mut item_count = Vec::with_capacity(TILES);
        let mut graffiti = Vec::with_capacity(TILES);
        for (i, t) in b.tiles.iter().enumerate() {
            terrain.push(t.terrain);
            furniture.push(t.furniture);
            trap.push(t.trap);
            top_item.push(t.top_item);
            item_count.push(t.item_count);
            graffiti.push(t.graffiti);
            if !t.fields.is_empty() {
                field_index[i] = fields.len() as i32;
                for &(type_, intensity) in &t.fields {
                    let mut f = FieldOnTileT::default();
                    f.tile_index = i as u16;
                    f.type_ = type_;
                    f.intensity = intensity;
                    fields.push(f);
                }
                let (dt, di) = displayed_field(reg, &t.fields);
                displayed[i] = DisplayedFieldT {
                    field_type: dt,
                    intensity: di,
                };
            }
        }
        let mut block = SubmapBlockT::default();
        block.origin = Some(b.origin.clone());
        block.terrain = Some(terrain);
        block.furniture = Some(furniture);
        block.trap = Some(trap);
        block.fields = Some(fields);
        block.field_index = Some(field_index);
        block.displayed_field = Some(displayed);
        block.top_item = Some(top_item);
        block.item_count = Some(item_count);
        // S3: the computed visibility cache (crate::visibility).
        block.lit = Some(vis.lit.clone());
        block.seen = Some(vis.seen.clone());
        block.has_graffiti = Some(graffiti);
        block.outside = Some(vis.outside.clone());
        block.light_frac = Some(vis.frac.clone());
        block.light_tint = Some(vis.tint.clone());
        block
    }
}

/// Submap index of a tile pos: (x/12, y/12, z), euclidean so
/// negative coordinates index correctly.
pub fn submap_index(pos: [i32; 3]) -> (i32, i32, i32) {
    (
        pos[0].div_euclid(SUBMAP as i32),
        pos[1].div_euclid(SUBMAP as i32),
        pos[2],
    )
}

/// Absolute origin of the submap containing a tile pos.
pub fn submap_origin(pos: [i32; 3]) -> (i32, i32, i32) {
    let s = SUBMAP as i32;
    (pos[0].div_euclid(s) * s, pos[1].div_euclid(s) * s, pos[2])
}

/// One tile's full wire state, extracted from a `SubmapBlockT` —
/// the unit the Delta diff compares. Missing arrays read as
/// their zero values (our own blocks always carry every array;
/// `outside` is the exception, absent unless the scene emits it).
#[derive(Debug, Clone, PartialEq)]
struct TileView {
    terrain: u32,
    furniture: u32,
    trap: u32,
    fields: Vec<(u32, u8)>,
    displayed: (u32, u8),
    top_item: u32,
    item_count: u16,
    lit: LitLevel,
    seen: u8,
    graffiti: bool,
    outside: bool,
    frac: u8,
    tint: [u8; 3],
}

impl TileView {
    /// True when two views differ ONLY in the light channels
    /// (lit/seen/frac/tint) — the LightUpdate folding test.
    fn light_only_differs(&self, other: &TileView) -> bool {
        self.terrain == other.terrain
            && self.furniture == other.furniture
            && self.trap == other.trap
            && self.fields == other.fields
            && self.displayed == other.displayed
            && self.top_item == other.top_item
            && self.item_count == other.item_count
            && self.graffiti == other.graffiti
            && self.outside == other.outside
    }
}

fn tile_view(b: &SubmapBlockT, i: usize) -> TileView {
    fn at<T: Copy>(arr: &Option<Vec<T>>, i: usize, zero: T) -> T {
        arr.as_ref().and_then(|v| v.get(i)).copied().unwrap_or(zero)
    }
    let fields: Vec<(u32, u8)> = b
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
    let tint = b
        .light_tint
        .as_ref()
        .and_then(|t| {
            let j = i * 3;
            Some([*t.get(j)?, *t.get(j + 1)?, *t.get(j + 2)?])
        })
        .unwrap_or([255, 255, 255]);
    TileView {
        terrain: at(&b.terrain, i, 0),
        furniture: at(&b.furniture, i, 0),
        trap: at(&b.trap, i, 0),
        fields,
        displayed,
        top_item: at(&b.top_item, i, 0),
        item_count: at(&b.item_count, i, 0),
        lit: at(&b.lit, i, LitLevel::DARK),
        seen: at(&b.seen, i, 0),
        graffiti: at(&b.has_graffiti, i, false),
        outside: at(&b.outside, i, false),
        frac: at(&b.light_frac, i, 0),
        tint,
    }
}

/// Diff one tile between the last-sent block and the current
/// block; `Some((TileChange, light_only))` with the CURRENT
/// state if anything differs — `light_only` marks diffs the
/// LightUpdate folding may carry (lit/seen/frac/tint only).
fn diff_tile(last: &SubmapBlockT, cur: &SubmapBlockT, i: usize) -> Option<(TileChangeT, bool)> {
    let (lv, cv) = (tile_view(last, i), tile_view(cur, i));
    if lv == cv {
        return None;
    }
    let light_only = lv.light_only_differs(&cv);
    let o = cur.origin.as_ref().expect("blocks always carry an origin");
    let mut tc = TileChangeT::default();
    tc.pos = Some(v3(
        o.x + (i % SUBMAP) as i32,
        o.y + (i / SUBMAP) as i32,
        o.z,
    ));
    tc.terrain = cv.terrain;
    tc.furniture = cv.furniture;
    tc.trap = cv.trap;
    tc.fields = if cv.fields.is_empty() {
        None
    } else {
        Some(
            cv.fields
                .iter()
                .map(|&(type_, intensity)| {
                    let mut f = FieldOnTileT::default();
                    f.tile_index = i as u16;
                    f.type_ = type_;
                    f.intensity = intensity;
                    f
                })
                .collect(),
        )
    };
    tc.top_item = cv.top_item;
    tc.item_count = cv.item_count;
    tc.lit = cv.lit;
    tc.seen = cv.seen;
    tc.has_graffiti = cv.graffiti;
    tc.displayed_field = Some(DisplayedFieldT {
        field_type: cv.displayed.0,
        intensity: cv.displayed.1,
    });
    tc.outside = cv.outside;
    tc.light_frac = cv.frac;
    tc.light_tint_r = cv.tint[0];
    tc.light_tint_g = cv.tint[1];
    tc.light_tint_b = cv.tint[2];
    Some((tc, light_only))
}

/// The displayed-field rule (§3.2, as in the sample house): fire
/// is displayed over other fields; otherwise the highest
/// intensity; ties keep the first entry. (0, 0) = none.
fn displayed_field(reg: &DefRegistry, fields: &[(u32, u8)]) -> (u32, u8) {
    let fire_id = reg.id_of(Kind::Field, "fd_fire");
    let mut best: Option<(u32, u8)> = None;
    for &(type_, intensity) in fields {
        best = match best {
            None => Some((type_, intensity)),
            Some((bt, bi)) => {
                let cur_is_fire = type_ == fire_id && fire_id != 0;
                let best_is_fire = bt == fire_id && fire_id != 0;
                if (cur_is_fire && !best_is_fire) || (cur_is_fire == best_is_fire && intensity > bi)
                {
                    Some((type_, intensity))
                } else {
                    best
                }
            }
        };
    }
    best.unwrap_or((0, 0))
}
