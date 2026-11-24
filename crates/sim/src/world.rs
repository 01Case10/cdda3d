//! World state + Snapshot builder (slices S1-S5).
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
//!
//! S5 state: sim HP per entity behind the wire `hp_frac`, a
//! seeded xorshift RNG, per-field age/fuel bookkeeping
//! (parallel to the wire field list, never sent), a burned-out
//! tile set, and the turn's pending Effects — the creature /
//! field machinery itself lives in [`crate::rules`].
//!
//! S6 state: vehicles live in a sim-side store
//! ([`VehicleState`]: origin + continuous `face_deg` + parts
//! keyed by mount) — the wire `VehicleT` is *derived* from it,
//! with each part's absolute position recomputed by the
//! ported `precalc_mounts`/`coord_translate` math
//! ([`vehicle_precalc`]). Damage / removal / translation /
//! rotation mutate the store through the `World` vehicle
//! APIs; the flush differ diffs the derived wire form against
//! the last-sent baseline exactly like entities.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use proto::{
    DeltaT, DisplayedFieldT, EffectEventT, EffectsT, EntityT, EnvironmentUpdateT, Facing,
    FieldOnTileT, LightUpdateT, LitLevel, MessageT, PickResultT, SnapshotT, SubmapBlockT,
    TileChangeT, UiFrameT, Vec3iT, VehiclePartStateT, VehicleT,
};
use ui_share::store::UiShmStore;

use crate::defs::{DefRegistry, Kind};
use crate::environment::{self, WeatherState};
use crate::rules::PendingAction;
use crate::scenes::{LightSpec, Scene, WeatherStep, SUBMAP};
use crate::ui::{CellGrid, Ui, UiState, UiView, DEFAULT_UI_SIZE};
use crate::visibility::{self, BlockVis};

pub const TILES: usize = SUBMAP * SUBMAP;

#[derive(Debug, Clone, Default)]
pub struct TileState {
    pub terrain: u32,
    pub furniture: u32,
    pub trap: u32,
    /// (field def id, intensity) in wire order.
    pub fields: Vec<(u32, u8)>,
    /// Sim-internal per-field bookkeeping (S5), parallel to
    /// `fields`: (age in turns since the entry appeared or
    /// last changed intensity, fuel turns remaining for fire;
    /// fuel -1 = not yet initialised / not applicable). Never
    /// on the wire — the wire shape is `fields` alone.
    pub field_meta: Vec<(i32, i32)>,
    pub top_item: u32,
    pub item_count: u16,
    pub graffiti: bool,
}

impl TileState {
    /// Index of the field entry of type `type_`, if present.
    pub fn field_index(&self, type_: u32) -> Option<usize> {
        self.fields.iter().position(|&(t, _)| t == type_)
    }

    /// Intensity of the field of type `type_`, if present.
    pub fn field_intensity(&self, type_: u32) -> Option<u8> {
        self.field_index(type_).map(|i| self.fields[i].1)
    }

    /// Add a field entry (or refresh an existing one of the
    /// same type: intensity keeps the max, age restarts — the
    /// shape of the game's add_field on an existing entry).
    /// `fuel` < 0 leaves the fuel counter uninitialised (the
    /// field tick initialises fire fuel from the tile).
    pub fn add_field(&mut self, type_: u32, intensity: u8, fuel: i32) {
        match self.field_index(type_) {
            Some(i) => {
                self.fields[i].1 = self.fields[i].1.max(intensity);
                self.field_meta[i].0 = 0;
            }
            None => {
                self.fields.push((type_, intensity));
                self.field_meta.push((0, fuel));
            }
        }
    }

    /// Set an entry's intensity (+ optionally its fuel),
    /// restarting its age. Removing at intensity 0 is the
    /// caller's job (`remove_field_at`).
    pub fn set_field_at(&mut self, i: usize, intensity: u8, fuel: Option<i32>) {
        self.fields[i].1 = intensity;
        self.field_meta[i].0 = 0;
        if let Some(f) = fuel {
            self.field_meta[i].1 = f;
        }
    }

    pub fn remove_field_at(&mut self, i: usize) {
        self.fields.remove(i);
        self.field_meta.remove(i);
    }
}

#[derive(Debug, Clone)]
pub struct BlockState {
    pub origin: Vec3iT,
    pub tiles: Vec<TileState>,
}

/// Vehicle part flag bit: the part is open (vehicle door /
/// trunk). The fork adapter's bits (`part.open`,
/// voxel_real_adapter.cpp), mirrored by `vehicles::model`.
pub const VPART_FLAG_OPEN: u32 = 1 << 0;
/// Vehicle part flag bit: the part is broken
/// (`vehicle_part::is_broken()`).
pub const VPART_FLAG_BROKEN: u32 = 1 << 1;
/// Vehicle part flag bit: the part is a roof part
/// (`vpart_info` ROOF flag) — roof parts stamp on the top
/// block of their tile, body parts below.
pub const VPART_FLAG_ROOF: u32 = 1 << 2;

/// One vehicle part in the sim store (S6): identity is
/// (vpart def, mount) — the mount is the part's fixed slot in
/// the vehicle frame and never changes as the vehicle moves
/// or turns (the engine's `PartKey` notion). The wire
/// absolute position is derived, never stored.
#[derive(Debug, Clone, PartialEq)]
pub struct VehiclePartState {
    /// VehiclePart wire id (per-kind, registry).
    pub vpart: u32,
    /// Mount in the vehicle frame: x forward, y across,
    /// z up — CDDA's vehicle JSON part locations.
    pub mount: [i32; 3],
    pub hp_frac: u8,
    pub flags: u32,
}

/// One vehicle in the sim store (S6). The wire form
/// ([`VehicleT`]) is derived by [`VehicleState::to_wire`]:
/// every part's absolute position is
/// `origin + vehicle_precalc(face_deg, mount)` — the game's
/// own arrangement, where `vehicle::bub_part_pos` is the
/// vehicle position plus the mount pre-rotated by
/// `precalc_mounts` (0.I-1 src/vehicle.cpp:3757-3765).
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleState {
    pub id: u64,
    pub name: String,
    /// The pivot tile: where mount (0, 0, 0) sits. Upstream's
    /// `precalc_mounts` runs with pivot (0,0) for a placed
    /// vehicle (vehicle.cpp:2185), so the origin is the
    /// vehicle's position and the precalc is mount-relative.
    pub origin: [i32; 3],
    /// Continuous facing in degrees, stored and sent as the
    /// exact f32 — never quantised (§2.4).
    pub face_deg: f32,
    pub parts: Vec<VehiclePartState>,
}

impl VehicleState {
    /// The part's absolute tile position at the vehicle's
    /// current origin/facing.
    pub fn part_abs(&self, part: &VehiclePartState) -> [i32; 3] {
        let rel = vehicle_precalc(self.face_deg, part.mount);
        [
            self.origin[0] + rel[0],
            self.origin[1] + rel[1],
            self.origin[2] + rel[2],
        ]
    }

    /// Derive the wire form: parts in store order, absolute
    /// positions recomputed, `face_deg` verbatim.
    pub fn to_wire(&self) -> VehicleT {
        let parts: Vec<VehiclePartStateT> = self
            .parts
            .iter()
            .map(|p| {
                let abs = self.part_abs(p);
                let mut w = VehiclePartStateT::default();
                w.vpart = p.vpart;
                w.abs_pos = Some(v3(abs[0], abs[1], abs[2]));
                w.mount = Some(v3(p.mount[0], p.mount[1], p.mount[2]));
                w.hp_frac = p.hp_frac;
                w.flags = p.flags;
                w
            })
            .collect();
        let mut v = VehicleT::default();
        v.id = self.id;
        v.name = if self.name.is_empty() {
            None
        } else {
            Some(self.name.clone())
        };
        v.face_deg = self.face_deg;
        v.parts = if parts.is_empty() { None } else { Some(parts) };
        v
    }
}

/// The mount rotation at the heart of the vehicle store: a
/// faithful port of 0.I-1's `vehicle::coord_translate`
/// (src/vehicle.cpp:3598-3616) over a `tileray`
/// (src/tileray.cpp) — the exact math `precalc_mounts`
/// (vehicle.cpp:3643-3663) applies to every mount with pivot
/// (0, 0). Given the facing and a mount, returns the mount's
/// tile offset from the vehicle origin:
///
/// - the tileray's direction delta is `(100·cos θ, 100·sin θ)`
///   truncated to ints (tileray::init from an angle);
/// - `advance(mount.x)` walks the ray Bresenham-style: along
///   the major axis one tile per step, the minor axis
///   accumulating `leftover` until it crosses the major delta;
/// - the across component `mount.y` enters through the
///   orthogonal offset, which lands entirely on the minor
///   axis (ortho_dx when mostly vertical, ortho_dy when
///   mostly horizontal), signed by quadrant
///   (sx = [1,-1,-1,1], sy = [1,1,-1,-1]).
///
/// Consequences the gates pin: face 0 is the identity; face
/// 90 maps (x, y) -> (-y, x) exactly; intermediate angles
/// quantise the way the game's own ray does (at 23 degrees
/// this scene car's mounts — all within one tile of the
/// pivot — map to themselves).
pub fn vehicle_precalc(face_deg: f32, mount: [i32; 3]) -> [i32; 3] {
    // tileray::init(angle): normalise to [0, 360), delta =
    // (cos, sin) * 100 truncated toward zero (vec2d->point).
    let dir = face_deg.rem_euclid(360.0);
    let rad = dir.to_radians();
    let delta = [
        (rad.cos() * 100.0) as i32,
        (rad.sin() * 100.0) as i32,
    ];
    let abs_d = [delta[0].abs(), delta[1].abs()];
    let quadrant = ((dir / 90.0).floor() as usize) % 4;
    const SX: [i32; 4] = [1, -1, -1, 1];
    const SY: [i32; 4] = [1, 1, -1, -1];
    let mostly_vertical = abs_d[0] <= abs_d[1];

    // tileray::advance(num) from a cleared state.
    let advance = |num: i32| -> [i32; 2] {
        if num == 0 {
            return [0, 0];
        }
        let anum = num.abs();
        let mut last = [0i32; 2];
        let mut leftover = 0i32;
        if abs_d[0] != 0 && abs_d[1] != 0 {
            for _ in 0..anum {
                if mostly_vertical {
                    leftover += abs_d[0];
                    if leftover >= abs_d[1] {
                        last[0] += 1;
                        leftover -= abs_d[1];
                    }
                } else {
                    leftover += abs_d[1];
                    if leftover >= abs_d[0] {
                        last[1] += 1;
                        leftover -= abs_d[0];
                    }
                }
            }
        }
        if mostly_vertical {
            last[1] = anum;
        } else {
            last[0] = anum;
        }
        last[0] *= SX[quadrant];
        last[1] *= SY[quadrant];
        if num < 0 {
            last = [-last[0], -last[1]];
        }
        last
    };

    // coord_translate: q = advance(p.x - pivot.x) + ortho(p.y
    // - pivot.y), pivot (0,0); z passes through untouched.
    let fwd = advance(mount[0]);
    let od = mount[1];
    let ortho = if mostly_vertical {
        [od * -SY[quadrant], 0]
    } else {
        [0, od * SX[quadrant]]
    };
    [fwd[0] + ortho[0], fwd[1] + ortho[1], mount[2]]
}

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub blocks: Vec<BlockState>,
    /// (origin x, y, z) of submaps changed since the last send.
    dirty: HashSet<(i32, i32, i32)>,
    pub entities: Vec<EntityT>,
    /// The vehicle store (S6): mutated through the World
    /// vehicle APIs; the wire form is derived per snapshot /
    /// diff (`vehicles_wire`).
    pub vehicles: Vec<VehicleState>,
    pub player_pos: Option<[i32; 3]>,
    pub view_origin: Vec3iT,
    pub view_radius: u32,
    pub environment: bool,
    /// Game clock: calendar turns since turn_zero (1 s/turn).
    pub game_turn: i64,
    /// The player's move-point balance (movecost time
    /// scaling, Phase 11 follow-up; model in rules.rs module
    /// docs): upstream, a creature's moves replenish by its
    /// speed each calendar second (`moves += get_speed()`,
    /// 0.I-1 src/creature.cpp:388) and 100 points = 1 second
    /// (src/calendar.h:216-222), so the balance carries the
    /// fractional cost of expensive steps across actions.
    /// The sim's avatar is the unmodified baseline: speed
    /// 100, balance starts at a full 100. Only successful
    /// movement steps spend from it (`spend_move_points`);
    /// every other action leaves it untouched.
    pub move_balance: i32,
    /// Calendar seconds the action currently being flushed
    /// spans: set by `spend_move_points` when a movement
    /// step's cost drives the balance to <= 0 (one refill
    /// per elapsed second), 1 for every other action.
    /// Consumed (and reset to 1) by `flush_turn`, which runs
    /// its per-second processing once per elapsed second.
    turn_seconds: i32,
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
    /// Vehicle wire forms as last sent — the vehicle-diff
    /// baseline (S6).
    last_vehicles: Vec<VehicleT>,
    /// Submap index (x/12, y/12, z) the player was in at the last
    /// Snapshot; `None` until the first Snapshot is sent.
    last_snapshot_player_submap: Option<(i32, i32, i32)>,
    /// A pending Open/Close awaiting its direction key (S2 rules).
    pub pending: Option<PendingAction>,
    /// The UI state machine + shm store (S4). Owns its store
    /// (in-memory in-process; the server attaches a file store
    /// after the handshake).
    pub ui: Ui,
    /// Items the player carries: (item wire id, count), in
    /// inventory-list order. Seeded from the scene's
    /// `[[carried]]` tables; pickup/drop (S6) will mutate it.
    carried: Vec<(u32, u16)>,
    /// The most recent PickResult the engine sent (S4): a pick
    /// is a query answer, recorded for the gates — it never
    /// mutates the world.
    last_pick: Option<PickResultT>,
    /// Message-log lines for the gameplay sidebar (oldest
    /// first). Fed by the turn rules (door open/close today).
    pub messages: Vec<String>,
    /// Sim hit points per entity id (S5): the player and every
    /// monster. The wire carries only the derived `hp_frac`
    /// (0-255 of the entity's max, kept in sync by
    /// `sync_hp_frac`); max HP comes from the monster def's
    /// `hp` (the player's is `PLAYER_MAX_HP`).
    pub hp: HashMap<u64, i32>,
    /// Next entity id to allocate (runtime spawns, S5). Ids
    /// are never reused: a removed id stays removed, exactly
    /// like the game's creature tracker.
    next_entity_id: u64,
    /// Seeded RNG (xorshift64 — plan §3.1: the fork harness
    /// mock's family). Draws happen in a fixed order (fields
    /// in tile order, then creatures in id order), so seed +
    /// input log determines the whole run.
    pub rng: Rng,
    /// Effects produced since the last flush (S5): combat
    /// HitFlash/Sound from the rules. Drained into an Effects
    /// message by the flush, after the Delta/Snapshot.
    pub effects: Vec<EffectEventT>,
    /// Tiles whose fuel has burned out (S5): a burned tile no
    /// longer feeds a fire and cannot be re-lit by spread —
    /// the sim's stand-in for the game consuming the
    /// terrain/furniture itself (bash chains are S6).
    pub burned: HashSet<[i32; 3]>,
}

/// The avatar's max HP (S5 sim constant). CDDA tracks HP per
/// body part; the wire collapses it to one hp_frac, and the
/// sim's single-pool avatar uses 100 — the game's nominal
/// per-part scale — so 1 HP of damage is ~2.5 frac points.
pub const PLAYER_MAX_HP: i32 = 100;
/// Player HP floor (S5): the sim has no death/game-over
/// state, so damage floors the avatar at 1 HP. Monster
/// attacks and field burns still land (effects fire, frac
/// falls to the floor value); a game-over state is future
/// scope, named here rather than silently immortal — the
/// floor is the only lie, and it is this constant.
pub const PLAYER_HP_FLOOR: i32 = 1;
/// Fallback max HP for a monster def with no `hp` in the
/// table (the 0.I-1 table has none — all 1,123 carry `hp` —
/// but a hand-built registry must not panic the sim).
pub const DEFAULT_MONSTER_HP: i32 = 50;

/// xorshift64 (the fork harness mock's PRNG family, plan
/// §3.1; the same shifts as proto's gate Rng). Seed 0 maps to
/// a fixed nonzero constant — xorshift's zero state is a
/// fixed point.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: i64) -> Rng {
        let s = seed as u64;
        Rng(if s == 0 { 0x9E37_79B9_7F4A_7C15 } else { s })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in 0..n (n > 0).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// The game's `rng(1, 100) < chance` shape: a 1..=100 roll
    /// strictly below `chance` (so chance 25 succeeds on
    /// 1..=24 — upstream's literal comparison).
    pub fn roll_under(&mut self, chance: i32) -> bool {
        (self.below(100) as i32 + 1) < chance
    }
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
            blocks[b].tiles[i].add_field(id, f.intensity, -1);
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
        let mut entity_hp: HashMap<u64, i32> = HashMap::new();
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
            // S5: sim HP behind the wire frac — the def's max
            // scaled by the scene-authored frac.
            let max_hp = if s.type_id == "player" {
                PLAYER_MAX_HP
            } else {
                reg.def_by_id(Kind::Monster, type_def)
                    .and_then(|d| d.hp)
                    .unwrap_or(DEFAULT_MONSTER_HP)
            };
            entity_hp.insert(e.id, (max_hp * s.hp_frac as i32 + 127) / 255);
            entities.push(e);
        }
        let next_entity_id = entities.len() as u64 + 1;

        // Vehicles as store entries (S6). The origin comes
        // from the scene declaration; a scene that declares
        // none (the S1-era fixtures) seeds it from the first
        // part's declared absolute position — origin = pos -
        // precalc(face, mount) inverts the store derivation
        // exactly, so legacy scenes produce the identical
        // wire vehicles.
        let mut vehicles = Vec::new();
        for v in &scene.vehicles {
            let parts: Vec<VehiclePartState> = scene
                .vparts
                .iter()
                .filter(|p| p.vehicle == v.id)
                .map(|p| -> Result<VehiclePartState, String> {
                    let id = reg.id_of(Kind::VehiclePart, &p.type_id);
                    if id == 0 {
                        return Err(format!(
                            "scene '{}': vpart type '{}' is not a VehiclePart def",
                            scene.name, p.type_id
                        ));
                    }
                    Ok(VehiclePartState {
                        vpart: id,
                        mount: p.mount,
                        hp_frac: p.hp_frac,
                        flags: p.flags,
                    })
                })
                .collect::<Result<_, _>>()?;
            let origin = v.origin.or_else(|| {
                scene
                    .vparts
                    .iter()
                    .find(|p| p.vehicle == v.id)
                    .and_then(|first| {
                        first.pos.map(|pos| {
                            let rel = vehicle_precalc(v.face_deg, first.mount);
                            [pos[0] - rel[0], pos[1] - rel[1], pos[2] - rel[2]]
                        })
                    })
            });
            vehicles.push(VehicleState {
                id: v.id,
                name: v.name.clone(),
                origin: origin.unwrap_or([0, 0, 0]),
                face_deg: v.face_deg,
                parts,
            });
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

        // S4: the carried list (inventory contents), resolved
        // through the registry like every other scene extra.
        let mut carried = Vec::new();
        for c in &scene.carried {
            let id = reg.id_of(Kind::Item, &c.type_id);
            if id == 0 {
                return Err(format!(
                    "scene '{}': carried item '{}' is not an Item def",
                    scene.name, c.type_id
                ));
            }
            carried.push((id, c.count));
        }
        // S4: the UI, in-process form (memory store at the
        // default size; the server attaches a file store at the
        // HelloAck screen size after the handshake).
        let mut ui = Ui::new_memory(DEFAULT_UI_SIZE.0, DEFAULT_UI_SIZE.1)
            .map_err(|e| format!("scene '{}': ui store: {e}", scene.name))?;
        if let Some(start) = &scene.ui_start {
            let state = UiState::parse(start).ok_or_else(|| {
                format!(
                    "scene '{}': unknown ui_start '{start}' (use gameplay/inventory/crafting/look/main-menu)",
                    scene.name
                )
            })?;
            ui.set_state(state, scene.player);
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
            move_balance: 100,
            turn_seconds: 1,
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
            last_vehicles: Vec::new(),
            last_snapshot_player_submap: None,
            pending: None,
            ui,
            carried,
            last_pick: None,
            messages: Vec::new(),
            hp: entity_hp,
            next_entity_id,
            rng: Rng::new(scene.seed),
            effects: Vec::new(),
            burned: HashSet::new(),
        };
        world.recompute_visibility(reg);
        // Publish the initial UI frame into the store (the
        // server sends its notification after the Snapshot,
        // once the file store is attached; in-process readers
        // see it via `ui_store()`).
        world.refresh_ui(reg);
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

    // -----------------------------------------------------------------
    // UI support (S4): view assembly, publish, resize, picks.
    // -----------------------------------------------------------------

    /// An item def's display name: the def's `name` when the
    /// table carries one, else its string id.
    pub fn item_display_name(reg: &DefRegistry, id: u32) -> String {
        match reg.def_by_id(Kind::Item, id) {
            Some(d) if !d.name.is_empty() => d.name.clone(),
            Some(d) => d.string_id.clone(),
            None => format!("<item {id}>"),
        }
    }

    /// The items the player carries: (item wire id, count).
    pub fn carried_items(&self) -> &[(u32, u16)] {
        &self.carried
    }

    /// The first carried stack in list order (S6 drop rule:
    /// drop takes from here).
    pub fn carried_first(&self) -> Option<(u32, u16)> {
        self.carried.first().copied()
    }

    /// Add one unit of `id` to the carried list: merges into
    /// the existing stack of that kind if there is one, else
    /// appends a new stack (S6 pickup).
    pub fn carried_add(&mut self, id: u32) {
        if let Some(stack) = self.carried.iter_mut().find(|(s, _)| *s == id) {
            stack.1 = stack.1.saturating_add(1);
        } else {
            self.carried.push((id, 1));
        }
    }

    /// Remove one unit of `id` from the carried list; the
    /// stack leaves the list when it empties. Returns false
    /// when the player does not carry that kind (S6 drop).
    pub fn carried_remove_one(&mut self, id: u32) -> bool {
        let Some(i) = self.carried.iter().position(|(s, _)| *s == id) else {
            return false;
        };
        self.carried[i].1 -= 1;
        if self.carried[i].1 == 0 {
            self.carried.remove(i);
        }
        true
    }

    // -----------------------------------------------------------------
    // Vehicle store (S6): wire derivation + the mutation APIs
    // the gates drive (the S7 scenario DSL wraps these).
    // -----------------------------------------------------------------

    /// All vehicles in wire form, in store order.
    pub fn vehicles_wire(&self) -> Vec<VehicleT> {
        self.vehicles.iter().map(|v| v.to_wire()).collect()
    }

    /// One vehicle in wire form.
    pub fn vehicle_wire(&self, id: u64) -> Option<VehicleT> {
        self.vehicles.iter().find(|v| v.id == id).map(|v| v.to_wire())
    }

    /// A vehicle's current origin (the mount-(0,0,0) tile).
    pub fn vehicle_origin(&self, id: u64) -> Option<[i32; 3]> {
        self.vehicles.iter().find(|v| v.id == id).map(|v| v.origin)
    }

    /// A vehicle's current facing, the exact stored float.
    pub fn vehicle_face(&self, id: u64) -> Option<f32> {
        self.vehicles.iter().find(|v| v.id == id).map(|v| v.face_deg)
    }

    /// The store index of the part identified by (vpart string
    /// id, mount) on a vehicle — part identity for the
    /// mutation APIs.
    pub fn vehicle_part_index(
        &self,
        reg: &DefRegistry,
        id: u64,
        vpart: &str,
        mount: [i32; 3],
    ) -> Option<usize> {
        let vpart_id = reg.id_of(Kind::VehiclePart, vpart);
        if vpart_id == 0 {
            return None;
        }
        let v = self.vehicles.iter().find(|v| v.id == id)?;
        v.parts
            .iter()
            .position(|p| p.vpart == vpart_id && p.mount == mount)
    }

    /// The (vehicle id, part index) occupying an absolute
    /// tile, if any — bash scoping and passability read this.
    pub fn vehicle_part_at(&self, pos: [i32; 3]) -> Option<(u64, usize)> {
        for v in &self.vehicles {
            for (i, p) in v.parts.iter().enumerate() {
                if v.part_abs(p) == pos {
                    return Some((v.id, i));
                }
            }
        }
        None
    }

    /// Damage a part: `hp_frac` drops by `amount`
    /// (saturating); at 0 the broken flag sets — the sim's
    /// `vehicle_part::is_broken()` (hp == 0). The change rides
    /// the next flush's Delta as a wholesale vehicle update.
    pub fn damage_vehicle_part(&mut self, id: u64, part_index: usize, amount: u8) -> bool {
        let Some(v) = self.vehicles.iter_mut().find(|v| v.id == id) else {
            return false;
        };
        let Some(p) = v.parts.get_mut(part_index) else {
            return false;
        };
        p.hp_frac = p.hp_frac.saturating_sub(amount);
        if p.hp_frac == 0 {
            p.flags |= VPART_FLAG_BROKEN;
        }
        true
    }

    /// Remove a part from the store: it leaves the wire on the
    /// next flush (the vehicle update no longer lists it).
    pub fn remove_vehicle_part(&mut self, id: u64, part_index: usize) -> bool {
        let Some(v) = self.vehicles.iter_mut().find(|v| v.id == id) else {
            return false;
        };
        if part_index >= v.parts.len() {
            return false;
        }
        v.parts.remove(part_index);
        true
    }

    /// Move a vehicle rigidly: the origin shifts and every
    /// part's absolute position follows (derived).
    pub fn translate_vehicle(&mut self, id: u64, delta: [i32; 3]) -> bool {
        let Some(v) = self.vehicles.iter_mut().find(|v| v.id == id) else {
            return false;
        };
        v.origin = [
            v.origin[0] + delta[0],
            v.origin[1] + delta[1],
            v.origin[2] + delta[2],
        ];
        true
    }

    /// Set a vehicle's facing: the exact float is stored
    /// verbatim (never quantised); part absolutes follow
    /// through [`vehicle_precalc`] on the next derivation.
    pub fn set_vehicle_face(&mut self, id: u64, face_deg: f32) -> bool {
        let Some(v) = self.vehicles.iter_mut().find(|v| v.id == id) else {
            return false;
        };
        v.face_deg = face_deg;
        true
    }

    /// The UI store handle (readers/tests attach through this;
    /// the server swaps in a file store — fetch the handle
    /// after any attach/resize).
    pub fn ui_store(&self) -> &UiShmStore {
        self.ui.store()
    }

    /// The current UI cell grid (the frame's text source).
    pub fn ui_grid(&self) -> &CellGrid {
        self.ui.grid()
    }

    /// The current UI state.
    pub fn ui_state(&self) -> UiState {
        self.ui.state()
    }

    /// The most recent PickResult received, if any.
    pub fn last_pick(&self) -> Option<&PickResultT> {
        self.last_pick.as_ref()
    }

    /// Assemble the render view from world state (plain data —
    /// `crate::ui` never touches the world).
    pub fn ui_view(&self, reg: &DefRegistry) -> UiView {
        let secs = self.game_turn.rem_euclid(86_400);
        let clock = format!(
            "Day {}  {:02}:{:02}",
            self.game_turn.div_euclid(86_400) + 1,
            secs / 3600,
            (secs % 3600) / 60
        );
        let look_lines = if self.ui.state() == UiState::LookAround {
            self.look_lines(reg, self.ui.look_cursor())
        } else {
            Vec::new()
        };
        UiView {
            scene_name: self.name.clone(),
            player_pos: self.player_pos(),
            player_hp: self
                .player_id
                .and_then(|id| self.entities.iter().find(|e| e.id == id))
                .map(|e| e.hp_frac),
            clock,
            weather: self.weather.preset.to_string(),
            carried: self
                .carried
                .iter()
                .map(|&(id, n)| (Self::item_display_name(reg, id), n))
                .collect(),
            messages: self.messages.clone(),
            look_lines,
        }
    }

    /// Description lines for the look-around cursor tile:
    /// terrain / furniture / trap / items / fields by def name,
    /// plus the computed light + outside verdicts.
    fn look_lines(&self, reg: &DefRegistry, pos: [i32; 3]) -> Vec<String> {
        let name_of = |kind: Kind, id: u32| -> String {
            if id == 0 {
                return String::new();
            }
            match reg.def_by_id(kind, id) {
                Some(d) if !d.name.is_empty() => d.name.clone(),
                Some(d) => d.string_id.clone(),
                None => format!("<def {id}>"),
            }
        };
        let Some((b, i)) = self.tile_at(pos) else {
            return vec!["(outside the scene)".to_string()];
        };
        let t = &self.blocks[b].tiles[i];
        let mut lines = Vec::new();
        lines.push(format!("Terrain: {}", name_of(Kind::Terrain, t.terrain)));
        let furn = name_of(Kind::Furniture, t.furniture);
        if !furn.is_empty() {
            lines.push(format!("Furniture: {furn}"));
        }
        let trap = name_of(Kind::Trap, t.trap);
        if !trap.is_empty() {
            lines.push(format!("Trap: {trap}"));
        }
        if t.top_item != 0 {
            let mut line = format!("Items: {}", Self::item_display_name(reg, t.top_item));
            if t.item_count > 1 {
                line.push_str(&format!(" × {}", t.item_count));
            }
            lines.push(line);
        }
        for &(type_, intensity) in &t.fields {
            lines.push(format!("Field: {} (intensity {intensity})", name_of(Kind::Field, type_)));
        }
        let vis = &self.vis[b];
        lines.push(format!(
            "Light: {:?}   Outside: {}",
            vis.lit[i], vis.outside[i]
        ));
        lines
    }

    /// Clamp bounds for the look cursor: the scene's tile
    /// extent ([min, max] absolute pos over all blocks).
    pub fn look_bounds(&self) -> ([i32; 3], [i32; 3]) {
        let mut min = [i32::MAX; 3];
        let mut max = [i32::MIN; 3];
        for b in &self.blocks {
            min[0] = min[0].min(b.origin.x);
            min[1] = min[1].min(b.origin.y);
            min[2] = min[2].min(b.origin.z);
            max[0] = max[0].max(b.origin.x + SUBMAP as i32 - 1);
            max[1] = max[1].max(b.origin.y + SUBMAP as i32 - 1);
            max[2] = max[2].max(b.origin.z);
        }
        (min, max)
    }

    /// Render + publish the current UI state; returns the
    /// UiFrame notification to send, or `None` on a store error
    /// (logged, never a panic — a UI failure must not kill the
    /// game loop).
    pub fn refresh_ui(&mut self, reg: &DefRegistry) -> Option<UiFrameT> {
        let view = self.ui_view(reg);
        match self.ui.publish(&view) {
            Ok(frame) => Some(frame),
            Err(e) => {
                eprintln!("sim: ui publish failed: {e}");
                None
            }
        }
    }

    /// Attach the server-side file store (after the handshake
    /// told us the screen size) and publish the current state
    /// into it; returns the UiFrame notification to send.
    pub fn ui_attach_file(
        &mut self,
        reg: &DefRegistry,
        path: PathBuf,
        width: u32,
        height: u32,
    ) -> Result<UiFrameT, String> {
        self.ui
            .attach_file(path, width, height)
            .map_err(|e| format!("ui store: {e}"))?;
        self.refresh_ui(reg).ok_or_else(|| "ui publish failed".to_string())
    }

    /// Handle a Resize message: recreate the store at the new
    /// geometry and publish a full frame with the new size.
    /// A same-size resize is a no-op (no messages).
    pub fn apply_resize(&mut self, reg: &DefRegistry, width: u32, height: u32) -> Vec<MessageT> {
        if self.ui.size() == (width, height) || width == 0 || height == 0 {
            return Vec::new();
        }
        if let Err(e) = self.ui.resize(width, height) {
            eprintln!("sim: ui resize failed: {e}");
            return Vec::new();
        }
        self.refresh_ui(reg)
            .map(|f| vec![MessageT::UiFrame(Box::new(f))])
            .unwrap_or_default()
    }

    /// Record a PickResult from the engine. A pick is a query
    /// answer (the engine ran the ray; gate 4.11): it is kept
    /// for the gates and never mutates the world.
    pub fn apply_pick_result(&mut self, pick: &PickResultT) {
        self.last_pick = Some(pick.clone());
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
        let vehicles = self.vehicles_wire();
        snap.vehicles = if vehicles.is_empty() {
            None
        } else {
            Some(vehicles)
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
        self.last_vehicles = snap.vehicles.clone().unwrap_or_default();
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

    /// The computed lit level at an absolute pos, from the
    /// visibility cache (S3/S5): what the wire currently says
    /// about the tile. `None` outside the scene or before the
    /// first recompute.
    pub fn lit_at(&self, pos: [i32; 3]) -> Option<LitLevel> {
        let (b, i) = self.tile_at(pos)?;
        self.vis.get(b).and_then(|v| v.lit.get(i)).copied()
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

    /// Set the furniture at an absolute pos and mark its
    /// submap dirty (0 = no furniture). Returns false if the
    /// pos is outside the scene. (S6: the bash chain's
    /// furn_set goes through here.)
    pub fn set_furniture(&mut self, pos: [i32; 3], furniture: u32) -> bool {
        let Some((b, i)) = self.tile_at(pos) else {
            return false;
        };
        self.blocks[b].tiles[i].furniture = furniture;
        let o = &self.blocks[b].origin;
        self.mark_dirty((o.x, o.y, o.z));
        true
    }

    /// The item pile at an absolute pos: (top_item wire id,
    /// count). The wire's lossy shape — one kind per tile.
    pub fn tile_items(&self, pos: [i32; 3]) -> Option<(u32, u16)> {
        let (b, i) = self.tile_at(pos)?;
        let t = &self.blocks[b].tiles[i];
        Some((t.top_item, t.item_count))
    }

    /// Replace the item pile at an absolute pos and mark its
    /// submap dirty (S6 pickup/drop).
    pub fn set_tile_items(&mut self, pos: [i32; 3], top_item: u32, count: u16) -> bool {
        let Some((b, i)) = self.tile_at(pos) else {
            return false;
        };
        self.blocks[b].tiles[i].top_item = top_item;
        self.blocks[b].tiles[i].item_count = count;
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

    /// Move any entity to `pos`, updating facing from the
    /// horizontal step: LEFT if dx < 0, RIGHT if dx > 0,
    /// unchanged if dx == 0 (§2.4: facing is the horizontal
    /// flip only). No submap is dirtied — entities are not
    /// tile state; the entity diff carries the move.
    pub fn set_entity_pos(&mut self, id: u64, pos: [i32; 3]) {
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

    /// Move the player entity to `pos` (§3.2 facing rule, via
    /// `set_entity_pos`).
    pub fn set_player_pos(&mut self, pos: [i32; 3]) {
        if let Some(id) = self.player_id {
            self.set_entity_pos(id, pos);
        }
        self.player_pos = Some(pos);
    }

    /// True if a creature entity (not the player) occupies `pos`.
    pub fn creature_at(&self, pos: [i32; 3]) -> bool {
        self.creature_id_at(pos).is_some()
    }

    /// The creature entity (not the player) occupying `pos`.
    pub fn creature_id_at(&self, pos: [i32; 3]) -> Option<u64> {
        self.entities
            .iter()
            .find(|e| {
                Some(e.id) != self.player_id
                    && e.pos
                        .as_ref()
                        .is_some_and(|p| p.x == pos[0] && p.y == pos[1] && p.z == pos[2])
            })
            .map(|e| e.id)
    }

    /// Any entity (player included) occupying `pos`.
    pub fn entity_at(&self, pos: [i32; 3]) -> Option<u64> {
        self.entities
            .iter()
            .find(|e| {
                e.pos
                    .as_ref()
                    .is_some_and(|p| p.x == pos[0] && p.y == pos[1] && p.z == pos[2])
            })
            .map(|e| e.id)
    }

    /// An entity's absolute position.
    pub fn entity_pos(&self, id: u64) -> Option<[i32; 3]> {
        self.entities
            .iter()
            .find(|e| e.id == id)
            .and_then(|e| e.pos.as_ref())
            .map(|p| [p.x, p.y, p.z])
    }

    /// An entity's max HP: the player's pool, or its monster
    /// def's `hp` (fallback `DEFAULT_MONSTER_HP`).
    pub fn max_hp(&self, reg: &DefRegistry, id: u64) -> i32 {
        if Some(id) == self.player_id {
            return PLAYER_MAX_HP;
        }
        self.entities
            .iter()
            .find(|e| e.id == id)
            .and_then(|e| reg.def_by_id(Kind::Monster, e.type_def))
            .and_then(|d| d.hp)
            .unwrap_or(DEFAULT_MONSTER_HP)
    }

    /// An entity's current sim HP (0 if unknown/dead).
    pub fn current_hp(&self, id: u64) -> i32 {
        self.hp.get(&id).copied().unwrap_or(0)
    }

    /// Recompute an entity's wire `hp_frac` from its sim HP:
    /// round(255 * hp / max), clamped to 0..=255.
    pub fn sync_hp_frac(&mut self, reg: &DefRegistry, id: u64) {
        let max = self.max_hp(reg, id);
        let hp = self.current_hp(id).clamp(0, max.max(1));
        let frac = if max > 0 {
            ((hp * 255 + max / 2) / max).clamp(0, 255) as u8
        } else {
            0
        };
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == id) {
            e.hp_frac = frac;
        }
    }

    /// Spawn a monster at runtime (S5): the mechanism S7's
    /// scenario/script spawns will call — game-side, exactly
    /// like the real game, where a spawn is an internal event
    /// and the wire only sees the resulting entity diff (the
    /// next flush's Delta carries it in `entity_updates`; it
    /// does NOT wait for a Snapshot). Returns the new entity
    /// id, or `None` if the type is not a Monster def or the
    /// pos is outside the scene. Sprite = the monster's
    /// string id (§2.4); overlays empty (no grounded source —
    /// the def table carries no overlay data).
    pub fn spawn_monster(
        &mut self,
        reg: &DefRegistry,
        type_id: &str,
        pos: [i32; 3],
    ) -> Option<u64> {
        let type_def = reg.id_of(Kind::Monster, type_id);
        if type_def == 0 || self.tile_at(pos).is_none() {
            return None;
        }
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        let mut e = EntityT::default();
        e.id = id;
        e.type_def = type_def;
        e.sprite_id = Some(type_id.to_string());
        e.pos = Some(v3(pos[0], pos[1], pos[2]));
        e.facing = Facing::NONE;
        e.overlays = None;
        e.hp_frac = 255;
        e.flags = 0;
        self.entities.push(e);
        let max = self.max_hp(reg, id);
        self.hp.insert(id, max);
        Some(id)
    }

    /// Deal `damage` to an entity. Returns true if it died.
    /// The player's HP floors at `PLAYER_HP_FLOOR` (no
    /// game-over state in S5); a monster at HP <= 0 dies:
    /// removed from the entity list (the differ emits
    /// `entity_removed`) and its corpse item lands on its
    /// tile — the game's own outcome: `monster::die` drops a
    /// corpse item via `mdeath::normal` (0.I-1 monster.cpp:
    /// 3096-3098), whose item type is `corpse`; the wire's
    /// lossy pile (top_item + count) carries that def. If a
    /// pile already occupies the tile the corpse lands on top
    /// (top_item = corpse, count + 1) — the newest item is
    /// the uppermost.
    pub fn damage_entity(&mut self, reg: &DefRegistry, id: u64, damage: i32) -> bool {
        if damage <= 0 || !self.hp.contains_key(&id) {
            return false;
        }
        let is_player = Some(id) == self.player_id;
        let new_hp = {
            let hp = self.hp.get_mut(&id).expect("checked above");
            *hp -= damage;
            if is_player {
                *hp = (*hp).max(PLAYER_HP_FLOOR);
            }
            *hp
        };
        if !is_player && new_hp <= 0 {
            self.kill_entity(reg, id);
            return true;
        }
        self.sync_hp_frac(reg, id);
        false
    }

    /// Remove a creature and drop its corpse (see
    /// `damage_entity`). No-op for the player / unknown ids.
    pub fn kill_entity(&mut self, reg: &DefRegistry, id: u64) {
        if Some(id) == self.player_id {
            return;
        }
        let Some(pos) = self.entity_pos(id) else {
            return;
        };
        self.entities.retain(|e| e.id != id);
        self.hp.remove(&id);
        let corpse = reg.id_of(Kind::Item, "corpse");
        if corpse != 0 {
            if let Some((b, i)) = self.tile_at(pos) {
                let t = &mut self.blocks[b].tiles[i];
                if t.top_item == 0 {
                    t.top_item = corpse;
                    t.item_count = 1;
                } else {
                    t.top_item = corpse;
                    t.item_count = t.item_count.saturating_add(1);
                }
                let o = &self.blocks[b].origin;
                self.mark_dirty((o.x, o.y, o.z));
            }
        }
    }

    /// Record an effect for the current turn's flush.
    pub fn push_effect(&mut self, event: EffectEventT) {
        self.effects.push(event);
    }

    /// Can a creature stand on `pos`? CDDA's own rule,
    /// `map::move_cost_ter_furn` (see rules.rs module docs):
    /// terrain cost 0 blocks, a negative furniture modifier
    /// blocks, else the sum must be > 0. Shared by the
    /// player's step and creature movement — one rule, one
    /// implementation.
    pub fn passable(&self, reg: &DefRegistry, pos: [i32; 3]) -> bool {
        if self.tile_move_cost(reg, pos) == 0 {
            return false;
        }
        // A vehicle part's tile is occupied (S6): upstream a
        // vehicle tile blocks movement unless the part is
        // passable-through (aisle/cockpit); the sim has no
        // per-part passability data, so every part blocks —
        // one rule for player and creatures alike.
        self.vehicle_part_at(pos).is_none()
    }

    /// The tile's raw movement cost in upstream's JSON cost
    /// units (standard floor = 2), under the same rule as
    /// `passable`: terrain `move_cost` 0 blocks, a negative
    /// furniture modifier blocks, else the cost is the sum.
    /// 0 = impassable (including outside the scene and
    /// unknown defs). This is the per-tile value upstream's
    /// `map::move_cost` returns (src/map.cpp:2331-2356,
    /// `move_cost_internal`: terrain cost plus the
    /// furniture contribution, blocked cases returning 0);
    /// the step-cost formula over two tiles lives in
    /// rules.rs (`step_move_points`).
    pub fn tile_move_cost(&self, reg: &DefRegistry, pos: [i32; 3]) -> i32 {
        let Some(terrain) = self.terrain_at(pos) else {
            return 0; // outside the scene
        };
        let Some(def) = reg.def_by_id(Kind::Terrain, terrain) else {
            return 0; // unknown terrain id: blocked, never a panic
        };
        let ter_cost = def.move_cost.unwrap_or(0);
        if ter_cost == 0 {
            return 0;
        }
        let furn_mod = match self.furniture_at(pos) {
            Some(0) | None => 0, // no furniture: null furniture's mod is 0
            Some(furn_id) => reg
                .def_by_id(Kind::Furniture, furn_id)
                .and_then(|d| d.move_cost)
                .unwrap_or(0),
        };
        if furn_mod >= 0 && ter_cost + furn_mod > 0 {
            ter_cost + furn_mod
        } else {
            0
        }
    }

    /// Spend `points` move points on a successful movement
    /// step (movecost time scaling; the model and its
    /// citations are in the rules.rs module docs). The
    /// balance carries across actions; each refill needed
    /// to bring it back above 0 is one elapsed calendar
    /// second, recorded in `turn_seconds` for `flush_turn`
    /// to consume. A step the balance covers outright
    /// still spans the action's own single second (the
    /// sim's 1 Input = 1 turn contract), so the result is
    /// max(refills, 1) — upstream's average timing is
    /// preserved exactly: cost-250 steps from a full
    /// balance take 2 s, then 3 s, alternating.
    pub fn spend_move_points(&mut self, points: i32) {
        self.move_balance -= points;
        let mut seconds = 0;
        while self.move_balance <= 0 {
            self.move_balance += 100;
            seconds += 1;
        }
        self.turn_seconds = seconds.max(1);
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
    /// clock advances by the action's span in seconds —
    /// 1 s for almost every action, more when a movement
    /// step's movecost drove the move balance negative
    /// (`spend_move_points`; do_turn order: `calendar::turn
    /// += 1_turns` happens inside the turn) — due
    /// weather-script entries apply, visibility recomputes,
    /// and the diff against the last-sent baseline is
    /// composed into the turn's message flush (plan §2.7: a
    /// flush is a Delta plus optional LightUpdate /
    /// EnvironmentUpdate messages).
    ///
    /// The per-second processing below runs once per elapsed
    /// second, because upstream runs it once per do_turn and
    /// a do_turn IS one second: fields tick once per do_turn
    /// (`m.process_fields()`, src/do_turn.cpp:655), monsters
    /// act once per do_turn (`monmove()`, :666), and the
    /// calendar advances 1 turn (:475). A step that takes
    /// 3 s of game time therefore gives the world 3 field
    /// ticks and 3 creature rounds — the observable
    /// consequence of crossing expensive terrain. At a span
    /// of 1 s (every pre-scaling action) the loop body is
    /// exactly the old single pass.
    ///
    /// If the player is now in a different submap than at the
    /// last Snapshot (a map shift), the flush is instead a fresh
    /// Snapshot with epoch + 1. Seq advances by exactly 1 per
    /// message, in flush order.
    pub fn flush_turn(&mut self, reg: &DefRegistry) -> Vec<MessageT> {
        // No Snapshot sent yet: the first flush IS the Snapshot.
        if self.last_blocks.is_empty() {
            self.turn_seconds = 1;
            return vec![MessageT::Snapshot(Box::new(self.snapshot(reg)))];
        }
        let seconds = std::mem::replace(&mut self.turn_seconds, 1);
        for _ in 0..seconds {
            self.game_turn += 1;
            self.apply_script_due();
            // S5 turn processing (rules.rs), in the game's own
            // do_turn order of concerns: the world's fields tick
            // first (spread/decay/drift + field damage to whoever
            // stands in them), visibility recomputes over the new
            // field state (fire light is part of the light model),
            // then creatures act (their sight reads the fresh
            // cache). The player's own action already happened in
            // `apply_key`, before this flush.
            self.tick_fields(reg);
            self.recompute_visibility(reg);
            self.tick_creatures(reg);
        }
        // Map shift?
        if let (Some(cur), Some(last)) = (
            self.player_pos().map(submap_index),
            self.last_snapshot_player_submap,
        ) {
            if cur != last {
                let snap = self.build_snapshot(reg, self.seq + 1, self.epoch + 1);
                self.record_sent(&snap);
                let mut msgs = vec![MessageT::Snapshot(Box::new(snap))];
                self.drain_effects(&mut msgs);
                return msgs;
            }
        }
        self.compose_flush(reg, true)
    }

    /// Append the turn's pending Effects as one Effects
    /// message (its own seq, after the flush's other
    /// messages — plan §2.7: a flush is a Delta/Snapshot plus
    /// optional companions).
    fn drain_effects(&mut self, msgs: &mut Vec<MessageT>) {
        if self.effects.is_empty() {
            return;
        }
        self.seq += 1;
        let mut fx = EffectsT::default();
        fx.seq = self.seq;
        fx.events = Some(std::mem::take(&mut self.effects));
        msgs.push(MessageT::Effects(Box::new(fx)));
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

        // Vehicles diff exactly like entities (S6): the store
        // is derived to wire form and compared by id against
        // the last-sent baseline; a changed vehicle rides the
        // Delta wholesale (the protocol's update semantics).
        let cur_vehicles = self.vehicles_wire();
        let mut vehicle_updates: Vec<VehicleT> = Vec::new();
        for v in &cur_vehicles {
            match self.last_vehicles.iter().find(|old| old.id == v.id) {
                Some(old) if old == v => {}
                _ => vehicle_updates.push(v.clone()),
            }
        }
        let vehicle_removed: Vec<u64> = self
            .last_vehicles
            .iter()
            .map(|old| old.id)
            .filter(|id| !cur_vehicles.iter().any(|v| v.id == *id))
            .collect();
        let vehicle_changed = !vehicle_updates.is_empty() || !vehicle_removed.is_empty();

        self.dirty.clear();
        self.last_blocks = cur_blocks;
        self.last_entities = self.entities.clone();
        self.last_vehicles = cur_vehicles;

        let env_due = self.env_due();
        let mut msgs: Vec<MessageT> = Vec::new();
        if any_structural
            || entity_changed
            || vehicle_changed
            || (all_changes.is_empty() && emit_empty_delta)
        {
            self.seq += 1;
            let mut delta = DeltaT::default();
            delta.seq = self.seq;
            delta.epoch = self.epoch;
            delta.tile_changes = Some(all_changes);
            delta.entity_updates = Some(entity_updates);
            delta.entity_removed = Some(entity_removed);
            delta.vehicle_updates = Some(vehicle_updates);
            delta.vehicle_removed = Some(vehicle_removed);
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
        self.drain_effects(&mut msgs);
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
