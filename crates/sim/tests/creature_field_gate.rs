//! S5 gates: creatures, fields, effects — driven IN-PROCESS
//! through `World::apply_input` (the turn_gate Driver pattern,
//! extended to accept the Effects message a combat flush now
//! carries).
//!
//! - T3 (spawn enemies): a hostile chases the player across
//!   open ground (pos updates per Delta, facing follows the
//!   step), attacks when adjacent (player hp_frac drops,
//!   HitFlash + Sound in the flush's Effects), and dies to
//!   bump attacks (entity_removed + a corpse item on its
//!   tile). Occlusion/dark contract: a spawned entity is on
//!   the wire even when its tile is BLANK (behind a wall /
//!   unvisited dark room) — the engine-side half lives in
//!   crates/engine/tests/sim_creature_gate.rs.
//! - T7 (fields: fire & smoke): from Delta alone (a mirror fed
//!   only the message stream) the field list grows (spread)
//!   and shrinks (burnout), `displayed_field` follows the
//!   §3.2 rule (highest intensity, fire over smoke on tie),
//!   smoke drifts downwind, fire lights its tiles at night
//!   (lit/frac bump + fire-coloured tint vs the same scene
//!   unlit) and damages whoever stands in it.
//! - The 200-turn mirror invariant, extended: creatures
//!   chasing + a fire burning/spreading while the mirror ==
//!   fresh Snapshot invariant is checked after every turn.

use std::collections::HashMap;
use std::path::PathBuf;

use proto::{
    DeltaT, EffectKind, EffectsT, EntityT, Facing, InputKind, InputT, LitLevel, MessageT,
    SnapshotT, TileChangeT,
};
use sim::scenes::{FieldPlace, Spawn};
use sim::{DefRegistry, Kind, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

fn zombie_spawn(pos: [i32; 3]) -> Spawn {
    Spawn {
        type_id: "mon_zombie".to_string(),
        pos,
        hp_frac: 255,
        facing: "NONE".to_string(),
        overlays: Vec::new(),
        flags: 0,
        sprite: None,
    }
}

/// A world plus the seq/epoch bookkeeping every gate asserts
/// on (turn_gate's Driver, plus the S5 Effects arm).
struct Driver {
    reg: DefRegistry,
    world: World,
    seq: u64,
    epoch: u64,
}

impl Driver {
    fn new(scene_name: &str, mutate: impl FnOnce(&mut Scene)) -> (Driver, SnapshotT) {
        let reg = DefRegistry::load(&defs_path()).expect("registry loads");
        let mut scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
        mutate(&mut scene);
        let mut world = World::from_scene(&reg, &scene).expect("world builds");
        let snap = world.snapshot(&reg);
        assert_eq!(snap.seq, 1, "initial snapshot seq");
        assert_eq!(snap.epoch, 1, "initial snapshot epoch");
        (
            Driver {
                reg,
                world,
                seq: 1,
                epoch: 1,
            },
            snap,
        )
    }

    fn key(&mut self, c: char) -> Vec<MessageT> {
        let msgs = self.world.apply_input(&self.reg, &key_input(c));
        for m in &msgs {
            match m {
                MessageT::Delta(d) => {
                    assert_eq!(d.seq, self.seq + 1, "delta seq (key {c:?})");
                    assert_eq!(d.epoch, self.epoch, "delta epoch (key {c:?})");
                    self.seq = d.seq;
                }
                MessageT::Snapshot(s) => {
                    assert_eq!(s.seq, self.seq + 1, "snapshot seq (key {c:?})");
                    assert_eq!(s.epoch, self.epoch + 1, "snapshot epoch (key {c:?})");
                    self.seq = s.seq;
                    self.epoch = s.epoch;
                }
                MessageT::LightUpdate(lu) => {
                    assert_eq!(lu.seq, self.seq + 1, "light seq (key {c:?})");
                    assert_eq!(lu.epoch, self.epoch, "light epoch (key {c:?})");
                    self.seq = lu.seq;
                }
                MessageT::EnvironmentUpdate(e) => {
                    assert_eq!(e.seq, self.seq + 1, "env seq (key {c:?})");
                    assert_eq!(e.epoch, self.epoch, "env epoch (key {c:?})");
                    self.seq = e.seq;
                }
                MessageT::Effects(fx) => {
                    assert_eq!(fx.seq, self.seq + 1, "effects seq (key {c:?})");
                    self.seq = fx.seq;
                }
                other => panic!("a turn produced an unexpected message: {other:?}"),
            }
        }
        msgs
    }

    fn delta(&mut self, c: char) -> Box<DeltaT> {
        let msgs = self.key(c);
        let mut deltas: Vec<Box<DeltaT>> = msgs
            .into_iter()
            .filter_map(|m| match m {
                MessageT::Delta(d) => Some(d),
                _ => None,
            })
            .collect();
        assert_eq!(deltas.len(), 1, "key {c:?}: expected exactly one Delta");
        deltas.pop().expect("one delta")
    }

    fn player(&self) -> [i32; 3] {
        self.world.player_pos().expect("scene has a player")
    }

    fn player_hp_frac(&self) -> u8 {
        let id = self.world.player_id.expect("player id");
        self.world
            .entities
            .iter()
            .find(|e| e.id == id)
            .expect("player entity")
            .hp_frac
    }

    fn entity(&self, id: u64) -> Option<&EntityT> {
        self.world.entities.iter().find(|e| e.id == id)
    }

    fn entity_pos(&self, id: u64) -> Option<[i32; 3]> {
        self.entity(id)
            .and_then(|e| e.pos.as_ref())
            .map(|p| [p.x, p.y, p.z])
    }
}

fn entity_update_at<'a>(d: &'a DeltaT, id: u64) -> Option<&'a EntityT> {
    d.entity_updates.as_ref()?.iter().find(|e| e.id == id)
}

fn effects_of(msgs: &[MessageT]) -> Vec<&EffectsT> {
    msgs.iter()
        .filter_map(|m| match m {
            MessageT::Effects(fx) => Some(&**fx),
            _ => None,
        })
        .collect()
}

fn chebyshev(a: [i32; 3], b: [i32; 3]) -> i32 {
    (a[0] - b[0]).abs().max((a[1] - b[1]).abs()).max((a[2] - b[2]).abs())
}

/// Walk the street_block player out of the house to (5,9).
fn walk_out(drv: &mut Driver) {
    drv.delta('s');
    drv.delta('s');
    assert!(drv.key('o').is_empty(), "arming Open produces no message");
    drv.delta('s'); // open the door
    drv.delta('s'); // (5,8) doorway
    drv.delta('s'); // (5,9) outside
    assert_eq!(drv.player(), [5, 9, 0]);
}

// ---------------------------------------------------------------------
// T3
// ---------------------------------------------------------------------

#[test]
fn t3_chase_and_attack() {
    // The zombie is runtime-spawned into the open yard AFTER
    // the player steps out (the S5 spawn entry point — the
    // mechanism S7's scripts will use). Open ground matters:
    // the sim's chase is greedy (§3.2: no pathfinding beyond
    // greedy), and a greedy chaser cornered behind the house
    // wall stalls in a Chebyshev local minimum — upstream
    // would path around; the sim documents the limit instead.
    let (mut drv, _snap) = Driver::new("street_block", |_| {});
    walk_out(&mut drv);
    let zid = drv
        .world
        .spawn_monster(&drv.reg, "mon_zombie", [12, 11, 0])
        .expect("runtime spawn succeeds");
    assert_eq!(
        drv.reg.string_of(Kind::Monster, drv.entity(zid).unwrap().type_def),
        Some("mon_zombie"),
        "type_def resolves through the Monster registry"
    );

    // Wait: the zombie closes one tile per turn until it is
    // adjacent, every step carried by an EntityUpdated in that
    // turn's Delta, facing LEFT (westward).
    let mut prev = drv.entity_pos(zid).expect("zombie alive");
    let mut guard = 0;
    while chebyshev(prev, drv.player()) > 1 {
        guard += 1;
        assert!(guard <= 12, "chase must converge");
        let d = drv.delta('.');
        let up = entity_update_at(&d, zid).expect("a chase step rides the turn's Delta");
        let pos = up.pos.as_ref().map(|p| [p.x, p.y, p.z]).unwrap();
        assert_eq!(
            chebyshev(pos, drv.player()),
            chebyshev(prev, drv.player()) - 1,
            "each chase turn closes the distance by one"
        );
        assert_eq!(up.facing, Facing::LEFT, "westward chase faces LEFT");
        prev = pos;
    }
    assert_eq!(drv.player_hp_frac(), 255, "no attack before adjacency");

    // Adjacent: the next wait is an attack — the zombie stays,
    // the player's hp_frac drops, and the flush carries
    // HitFlash (at the player) + Sound Effects.
    let msgs = drv.key('.');
    let d = msgs
        .iter()
        .find_map(|m| match m {
            MessageT::Delta(d) => Some(d),
            _ => None,
        })
        .expect("attack turn still produces a Delta");
    assert!(
        entity_update_at(d, zid).is_none_or(|e| e.pos.as_ref().map(|p| [p.x, p.y, p.z])
            == Some(prev)),
        "an attacking creature does not move"
    );
    let player_up = entity_update_at(d, drv.world.player_id.unwrap())
        .expect("the attack updates the player entity");
    assert!(player_up.hp_frac < 255, "player hp_frac drops on a hit");
    assert_eq!(drv.player_hp_frac(), player_up.hp_frac);
    let fx = effects_of(&msgs);
    assert_eq!(fx.len(), 1, "one Effects message in the attack flush");
    let events = fx[0].events.as_ref().expect("events");
    let player_pos = drv.player();
    assert!(
        events.iter().any(|e| e.kind == EffectKind::HitFlash
            && e.to_pos.as_ref().map(|p| [p.x, p.y, p.z]) == Some(player_pos)),
        "HitFlash at the player's tile: {events:?}"
    );
    assert!(
        events.iter().any(|e| e.kind == EffectKind::Sound),
        "a Sound effect rides the attack: {events:?}"
    );
}

#[test]
fn t3_bump_attack_kill_and_corpse() {
    // Runtime spawn into the open yard, as in
    // t3_chase_and_attack (greedy chase needs open ground).
    let (mut drv, _snap) = Driver::new("street_block", |_| {});
    let corpse_id = drv.reg.id_of(Kind::Item, "corpse");
    assert_ne!(corpse_id, 0, "the corpse item def exists");

    walk_out(&mut drv);
    let zid = drv
        .world
        .spawn_monster(&drv.reg, "mon_zombie", [12, 11, 0])
        .expect("runtime spawn succeeds");
    drv.delta('.'); // the spawn's Delta (it also steps closer)
    // Let it come to us: it ends adjacent to the player (it
    // approaches from x=12 along the yard; the greedy chase
    // may cut the corner diagonally).
    let mut guard = 0;
    while chebyshev(drv.entity_pos(zid).unwrap(), drv.player()) > 1 {
        guard += 1;
        assert!(guard <= 12, "chase must converge");
        drv.delta('.');
    }
    let zpos = drv.entity_pos(zid).unwrap();
    assert_eq!(chebyshev(zpos, drv.player()), 1, "the zombie closes to adjacency");
    // The bump key is the direction from the player to the
    // zombie, whichever of the 8 it is.
    let ppos = drv.player();
    let bump_key = match (zpos[0] - ppos[0], zpos[1] - ppos[1]) {
        (-1, -1) => 'y',
        (0, -1) => 'k',
        (1, -1) => 'u',
        (-1, 0) => 'a',
        (1, 0) => 'd',
        (-1, 1) => 'b',
        (0, 1) => 's',
        (1, 1) => 'n',
        other => panic!("zombie not adjacent: {other:?}"),
    };

    // Bump into it: a melee attack, not a step. The zombie's
    // hp_frac falls every bump; the player never moves; the
    // zombie hits back while it lives.
    let mut prev_frac = drv.entity(zid).unwrap().hp_frac;
    assert_eq!(prev_frac, 255);
    let mut bumps = 0;
    let killed = loop {
        bumps += 1;
        assert!(bumps <= 20, "the zombie must die to bump attacks");
        let msgs = drv.key(bump_key);
        assert_eq!(drv.player(), [5, 9, 0], "a bump attack never moves the player");
        let d = msgs
            .iter()
            .find_map(|m| match m {
                MessageT::Delta(d) => Some(d),
                _ => None,
            })
            .expect("bump turn produces a Delta");
        // A HitFlash at the zombie's tile rides every bump.
        let fx = effects_of(&msgs);
        assert!(
            fx.iter().any(|f| f.events.as_ref().is_some_and(|evs| evs
                .iter()
                .any(|e| e.kind == EffectKind::HitFlash
                    && e.to_pos.as_ref().map(|p| [p.x, p.y, p.z]) == Some(zpos)))),
            "bump {bumps}: HitFlash at the zombie's tile"
        );
        if d.entity_removed
            .as_ref()
            .is_some_and(|r| r.contains(&zid))
        {
            // Death: the corpse item is on the zombie's tile.
            let tc = d
                .tile_changes
                .as_ref()
                .and_then(|tcs| {
                    tcs.iter().find(|tc| {
                        tc.pos.as_ref().map(|p| [p.x, p.y, p.z]) == Some(zpos)
                    })
                })
                .expect("death emits a TileChange for the corpse tile");
            assert_eq!(tc.top_item, corpse_id, "corpse item on the tile");
            assert_eq!(tc.item_count, 1);
            break true;
        }
        let frac = drv.entity(zid).expect("zombie alive").hp_frac;
        assert!(frac < prev_frac, "bump {bumps}: hp_frac falls ({prev_frac} -> {frac})");
        prev_frac = frac;
    };
    assert!(killed);
    assert!(drv.entity(zid).is_none(), "the zombie is gone from the world");
    let (b, i) = drv.world.tile_at(zpos).expect("tile exists");
    assert_eq!(drv.world.blocks[b].tiles[i].top_item, corpse_id);
    assert_eq!(drv.world.blocks[b].tiles[i].item_count, 1);
}

#[test]
fn t3_occlusion_and_dark_contract() {
    // Behind a wall (three_rooms' east room, door never
    // opened): the entity is on the wire in the Snapshot, and
    // its tile is BLANK — the engine must not show it.
    let (_drv, snap) = Driver::new("three_rooms", |s| {
        s.spawns.push(zombie_spawn([19, 5, 0]));
    });
    assert!(
        snap.entities
            .as_ref()
            .is_some_and(|es| es.iter().any(|e| e.sprite_id.as_deref() == Some("mon_zombie"))),
        "the occluded zombie is on the wire"
    );
    let lit = snap_tile_lit(&snap, [19, 5, 0]);
    assert_eq!(lit, LitLevel::BLANK, "behind the wall: BLANK");

    // In the dark (three_rooms at midnight, the unvisited
    // middle room): same contract.
    let (_drv, snap) = Driver::new("three_rooms", |s| {
        s.time = "midnight".to_string();
        s.spawns.push(zombie_spawn([12, 5, 0]));
    });
    assert_eq!(
        snap_tile_lit(&snap, [12, 5, 0]),
        LitLevel::BLANK,
        "in the dark: BLANK"
    );
}

fn snap_tile_lit(snap: &SnapshotT, pos: [i32; 3]) -> LitLevel {
    for b in snap.blocks.as_ref().expect("blocks") {
        let o = b.origin.as_ref().expect("origin");
        if o.z == pos[2]
            && (o.x..o.x + 12).contains(&pos[0])
            && (o.y..o.y + 12).contains(&pos[1])
        {
            let i = ((pos[1] - o.y) * 12 + (pos[0] - o.x)) as usize;
            return b.lit.as_ref().expect("lit")[i];
        }
    }
    panic!("tile {pos:?} not in snapshot");
}

// ---------------------------------------------------------------------
// Mirror (fed ONLY the message stream — the T7 "from Delta
// alone" instrument; turn_gate's mirror, tiles + entities).
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct TileMirror {
    fields: Vec<(u32, u8)>,
    displayed: (u32, u8),
    top_item: u32,
    item_count: u16,
    lit: LitLevel,
}

#[derive(Debug, Default)]
struct Mirror {
    tiles: HashMap<[i32; 3], TileMirror>,
    entities: HashMap<u64, EntityT>,
}

fn mirror_from_snapshot(snap: &SnapshotT) -> Mirror {
    let mut m = Mirror::default();
    for b in snap.blocks.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
        let o = b.origin.as_ref().expect("origin");
        for i in 0..144usize {
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
                    fields,
                    displayed,
                    top_item: b
                        .top_item
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or(0),
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
                },
            );
        }
    }
    for e in snap.entities.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
        m.entities.insert(e.id, e.clone());
    }
    m
}

fn mirror_apply(m: &mut Mirror, msg: &MessageT) {
    match msg {
        MessageT::Snapshot(s) => {
            let fresh = mirror_from_snapshot(s);
            m.tiles = fresh.tiles;
            m.entities = fresh.entities;
        }
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
                m.tiles.insert(
                    [p.x, p.y, p.z],
                    TileMirror {
                        fields,
                        displayed,
                        top_item: tc.top_item,
                        item_count: tc.item_count,
                        lit: tc.lit,
                    },
                );
            }
            for e in d.entity_updates.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                m.entities.insert(e.id, e.clone());
            }
            for id in d.entity_removed.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                m.entities.remove(id);
            }
        }
        MessageT::LightUpdate(lu) => {
            let o = lu.origin.as_ref().expect("origin");
            for i in 0..144usize {
                let pos = [o.x + (i % 12) as i32, o.y + (i / 12) as i32, o.z];
                if let Some(t) = m.tiles.get_mut(&pos) {
                    t.lit = lu
                        .lit
                        .as_ref()
                        .and_then(|v| v.get(i))
                        .copied()
                        .unwrap_or(LitLevel::BLANK);
                }
            }
        }
        MessageT::EnvironmentUpdate(_) | MessageT::Effects(_) => {}
        other => panic!("unexpected message in a turn flush: {other:?}"),
    }
}

// ---------------------------------------------------------------------
// T7
// ---------------------------------------------------------------------

#[test]
fn t7_fire_spread_burnout_displayed() {
    let (mut drv, snap) = Driver::new("field_fire", |_| {});
    let fire_id = drv.reg.id_of(Kind::Field, "fd_fire");
    let smoke_id = drv.reg.id_of(Kind::Field, "fd_smoke");
    assert_ne!(fire_id, 0);
    assert_ne!(smoke_id, 0);
    let mut mirror = mirror_from_snapshot(&snap);
    let burning = |m: &Mirror| -> usize {
        m.tiles
            .values()
            .filter(|t| t.fields.iter().any(|&(ty, _)| ty == fire_id))
            .count()
    };
    assert_eq!(burning(&mirror), 1, "the scene lights exactly one fire tile");

    let mut max_burning = 1usize;
    let mut saw_smoke = false;
    let mut saw_tie_display = false;
    let mut turn = 0usize;
    let mut burned_out_at = None;
    while turn < 1200 {
        turn += 1;
        let msgs = drv.key('.');
        for msg in &msgs {
            mirror_apply(&mut mirror, msg);
        }
        max_burning = max_burning.max(burning(&mirror));
        for t in mirror.tiles.values() {
            let fire = t.fields.iter().find(|&&(ty, _)| ty == fire_id);
            let smoke = t.fields.iter().find(|&&(ty, _)| ty == smoke_id);
            if smoke.is_some() {
                saw_smoke = true;
            }
            if let (Some(&(_, fi)), Some(&(_, _))) = (fire, smoke) {
                // Both present: the display must be the fire
                // (highest intensity wins; fire takes ties).
                if t.displayed.0 == fire_id && t.displayed.1 == fi {
                    saw_tie_display = true;
                }
            }
        }
        if burning(&mirror) == 0 {
            burned_out_at = Some(turn);
            break;
        }
    }
    assert!(
        max_burning >= 2,
        "the fire spreads beyond its starting tile (max burning {max_burning})"
    );
    assert!(saw_smoke, "burning produces smoke");
    assert!(
        saw_tie_display,
        "a tile holding fire + smoke displays the fire (§3.2 rule)"
    );
    assert!(
        burned_out_at.is_some(),
        "the fire burns out within 1200 turns (fuel is finite)"
    );
}

#[test]
fn t7_smoke_drifts_downwind() {
    let (mut drv, snap) = Driver::new("field_fire", |s| {
        s.fields.clear();
        s.fields.push(FieldPlace {
            pos: [5, 6, 0],
            type_id: "fd_smoke".to_string(),
            intensity: 3,
        });
    });
    let smoke_id = drv.reg.id_of(Kind::Field, "fd_smoke");
    let mut mirror = mirror_from_snapshot(&snap);
    let centroid_x = |m: &Mirror| -> f32 {
        let (mut num, mut den) = (0f32, 0f32);
        for (pos, t) in &m.tiles {
            for &(ty, inten) in &t.fields {
                if ty == smoke_id {
                    num += pos[0] as f32 * inten as f32;
                    den += inten as f32;
                }
            }
        }
        num / den.max(1.0)
    };
    let start = centroid_x(&mirror);
    assert_eq!(start, 5.0, "smoke starts on its placed tile");
    for _ in 0..8 {
        let msgs = drv.key('.');
        for msg in &msgs {
            mirror_apply(&mut mirror, msg);
        }
    }
    let end = centroid_x(&mirror);
    // Upstream gas cannot spread at intensity 1 (the
    // intensity > 1 gate in gas_spread_to), so a lone puff
    // thins into a short trail and stalls rather than
    // marching indefinitely; the gate asserts the drift that
    // does happen: centroid east by more than half a tile and
    // the leading edge two tiles downwind.
    assert!(
        end > start + 0.5,
        "smoke centroid drifts downwind (+x): {start} -> {end}"
    );
    let lead = mirror
        .tiles
        .iter()
        .filter(|(_, t)| t.fields.iter().any(|&(ty, _)| ty == smoke_id))
        .map(|(pos, _)| pos[0])
        .max()
        .unwrap();
    assert!(lead >= 7, "the smoke's leading edge advances east (x={lead})");
    let src = mirror.tiles[&[5, 6, 0]]
        .fields
        .iter()
        .find(|&&(ty, _)| ty == smoke_id)
        .map(|&(_, i)| i)
        .unwrap_or(0);
    assert!(src < 3, "the source tile thins as smoke leaves ({src})");
}

#[test]
fn t7_fire_light_and_player_damage() {
    // Midnight, fire lit: the fire tile is BRIGHT, its
    // neighbour's frac beats the unlit variant's, and the tint
    // at the fire tile is fire-coloured — all from the wire.
    let (mut drv, snap) = Driver::new("field_fire", |s| {
        s.time = "midnight".to_string();
    });
    let fire_pos = [18, 5, 0];
    let neighbour = [17, 5, 0];
    let block = snap
        .blocks
        .as_ref()
        .unwrap()
        .iter()
        .find(|b| b.origin.as_ref().unwrap().x == 12)
        .expect("east block");
    let at = |pos: [i32; 3]| ((pos[1]) * 12 + (pos[0] - 12)) as usize;
    assert_eq!(
        block.lit.as_ref().unwrap()[at(fire_pos)],
        LitLevel::BRIGHT,
        "the fire tile is BRIGHT at midnight"
    );
    let lit_frac = block.light_frac.as_ref().unwrap()[at(neighbour)];
    let tint = &block.light_tint.as_ref().unwrap()[at(fire_pos) * 3..at(fire_pos) * 3 + 3];
    assert_eq!(tint[0], 255, "fire tint: red channel max");
    assert!(
        tint[2] < tint[1] && tint[1] < 255,
        "fire tint is fire-coloured (orange), not white: {tint:?}"
    );

    // The same scene with no fire: strictly dimmer.
    let (_drv2, snap2) = Driver::new("field_fire", |s| {
        s.time = "midnight".to_string();
        s.fields.clear();
    });
    let block2 = snap2
        .blocks
        .as_ref()
        .unwrap()
        .iter()
        .find(|b| b.origin.as_ref().unwrap().x == 12)
        .expect("east block");
    assert_ne!(
        block2.lit.as_ref().unwrap()[at(fire_pos)],
        LitLevel::BRIGHT,
        "unlit: the fire tile is not BRIGHT"
    );
    let unlit_frac = block2.light_frac.as_ref().unwrap()[at(neighbour)];
    assert!(
        lit_frac > unlit_frac,
        "fire light raises the neighbour's frac ({unlit_frac} -> {lit_frac})"
    );

    // Standing in the fire burns the player: the step onto
    // the fire tile already costs one turn of burn, the two
    // waits cost two more (3 x 7 damage at intensity 1).
    let before = drv.player_hp_frac();
    assert_eq!(before, 255);
    drv.delta('d');
    drv.delta('d');
    assert_eq!(drv.player(), fire_pos, "the player steps onto the fire tile");
    drv.delta('.');
    drv.delta('.');
    let after = drv.player_hp_frac();
    assert!(
        after <= before - 20,
        "three turns in intensity-1 fire costs hp (frac {before} -> {after})"
    );
}

// ---------------------------------------------------------------------
// Mirror invariant with creatures + fire (200 turns)
// ---------------------------------------------------------------------

#[test]
fn mirror_invariant_creatures_and_fire_200_turns() {
    let (mut drv, snap) = Driver::new("field_fire", |s| {
        s.spawns.push(zombie_spawn([20, 5, 0]));
    });
    let zid = snap
        .entities
        .as_ref()
        .and_then(|es| es.iter().find(|e| e.sprite_id.as_deref() == Some("mon_zombie")))
        .map(|e| e.id)
        .expect("zombie in the snapshot");
    let fire_id = drv.reg.id_of(Kind::Field, "fd_fire");
    let mut mirror = mirror_from_snapshot(&snap);
    let start_zpos = drv.entity_pos(zid);
    let mut zombie_moved = false;
    let mut fire_spread = false;

    // A small patrol inside the cabin + waits; the zombie
    // chases through the cabin (across the fire row), the fire
    // grows and spreads meanwhile.
    const PATTERN: &[char] = &['.', '.', 's', '.', 'd', '.', 'k', '.', 'a', '.', '.', '.'];
    let mut turns = 0usize;
    let mut inputs = 0usize;
    while turns < 200 {
        let c = PATTERN[inputs % PATTERN.len()];
        inputs += 1;
        assert!(inputs < 10_000, "pattern stopped producing turns");
        let msgs = drv.key(c);
        if msgs.is_empty() {
            continue;
        }
        turns += 1;
        for msg in &msgs {
            mirror_apply(&mut mirror, msg);
        }
        if drv.entity_pos(zid) != start_zpos {
            zombie_moved = true;
        }
        if mirror
            .tiles
            .values()
            .filter(|t| t.fields.iter().any(|&(ty, _)| ty == fire_id))
            .count()
            > 1
        {
            fire_spread = true;
        }
        // The invariant: mirror (Delta-fed) == fresh Snapshot,
        // over the fields/entities this gate tracks, EVERY turn.
        let fresh = drv.world.build_snapshot(&drv.reg, 0, 0);
        let fresh_mirror = mirror_from_snapshot(&fresh);
        assert_eq!(
            mirror.tiles.len(),
            fresh_mirror.tiles.len(),
            "tile count diverged after turn {turns}"
        );
        for (pos, t) in &fresh_mirror.tiles {
            let mt = mirror.tiles.get(pos).expect("mirror tile present");
            assert_eq!(
                (mt.fields.clone(), mt.displayed, mt.top_item, mt.item_count),
                (t.fields.clone(), t.displayed, t.top_item, t.item_count),
                "tile {pos:?} diverged after turn {turns} (key {c:?})"
            );
        }
        assert_eq!(
            mirror.entities, fresh_mirror.entities,
            "entities diverged after turn {turns} (key {c:?})"
        );
    }
    assert_eq!(turns, 200);
    assert!(zombie_moved, "the zombie acted during the 200 turns");
    assert!(fire_spread, "the fire spread during the 200 turns");
}

// ---------------------------------------------------------------------
// Runtime spawn entry point (the mechanism S7's scenario
// scripts will drive) + creature death by field damage.
// ---------------------------------------------------------------------

#[test]
fn t3_runtime_spawn_arrives_in_delta() {
    let (mut drv, _snap) = Driver::new("street_block", |_| {});
    // The entry point's contract: unknown types and
    // off-scene positions are refused, not panicked on.
    assert_eq!(drv.world.spawn_monster(&drv.reg, "mon_nope", [12, 11, 0]), None);
    assert_eq!(drv.world.spawn_monster(&drv.reg, "mon_zombie", [99, 99, 0]), None);

    let zid = drv
        .world
        .spawn_monster(&drv.reg, "mon_zombie", [12, 11, 0])
        .expect("runtime spawn succeeds");
    // One wait turn: the spawn rides that turn's Delta (the
    // schema has no EntityAdded field — a spawn is an
    // entity_updates entry for an id the client has never
    // seen; see world.rs). `delta()` fails unless the flush
    // is exactly one Delta, so a Snapshot-carried spawn would
    // fail here.
    let d = drv.delta('.');
    let e = entity_update_at(&d, zid).expect("the spawn rides the turn's Delta");
    assert_eq!(e.sprite_id.as_deref(), Some("mon_zombie"));
    assert_eq!(e.type_def, drv.reg.id_of(Kind::Monster, "mon_zombie"));
    assert_eq!(e.hp_frac, 255, "a fresh spawn is at full HP");
}

#[test]
fn t7_creature_fire_damage_and_death() {
    // The player waits beside the cabin fire; a zombie
    // spawned ON the fire tile attacks instead of moving off
    // (adjacency wins) and burns for FIRE_DAMAGE[0] = 7 HP
    // per turn: 80 HP -> hp_frac ~188 after 3 turns, dead on
    // the 12th field tick.
    let (mut drv, snap) = Driver::new("field_fire", |s| {
        s.player = Some([17, 5, 0]);
        for sp in &mut s.spawns {
            if sp.type_id == "player" {
                sp.pos = [17, 5, 0];
            }
        }
    });
    let corpse_id = drv.reg.id_of(Kind::Item, "corpse");
    assert_ne!(corpse_id, 0, "the corpse item def exists");
    let mut mirror = mirror_from_snapshot(&snap);
    let zid = drv
        .world
        .spawn_monster(&drv.reg, "mon_zombie", [18, 5, 0])
        .expect("spawn on the fire tile");

    for turn in 1..=3 {
        let msgs = drv.key('.');
        for msg in &msgs {
            mirror_apply(&mut mirror, msg);
        }
        let e = mirror
            .entities
            .get(&zid)
            .unwrap_or_else(|| panic!("the zombie lives after turn {turn}"));
        assert_eq!(
            drv.entity_pos(zid),
            Some([18, 5, 0]),
            "an adjacent creature attacks; it does not step off the fire"
        );
        assert!(e.hp_frac < 255, "fire damage lands from turn 1");
        if turn == 3 {
            assert!(
                (150..=210).contains(&e.hp_frac),
                "3 turns x 7 damage on 80 HP -> hp_frac ~188 (got {})",
                e.hp_frac
            );
        }
    }

    // Keep waiting: the fire kills it. The killing flush
    // carries entity_removed, and the corpse lands on the
    // fire tile — both visible from the message stream alone.
    let mut died = false;
    for _ in 0..20 {
        let msgs = drv.key('.');
        for msg in &msgs {
            if let MessageT::Delta(d) = msg {
                if d.entity_removed
                    .as_ref()
                    .is_some_and(|v| v.contains(&zid))
                {
                    died = true;
                }
            }
            mirror_apply(&mut mirror, msg);
        }
        if died {
            break;
        }
    }
    assert!(died, "the zombie burns to death within 20 more turns");
    assert!(!mirror.entities.contains_key(&zid), "mirror dropped the zombie");
    let tile = mirror.tiles.get(&[18, 5, 0]).expect("fire tile in mirror");
    assert_eq!(tile.top_item, corpse_id, "the corpse is the fire tile's top item");
    assert!(tile.item_count >= 1, "corpse count on the tile");
}
