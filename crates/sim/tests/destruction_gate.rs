//! S6 gates (sim half): destruction + items — driven
//! IN-PROCESS through `World::apply_input` (the turn_gate /
//! creature_field_gate Driver pattern).
//!
//! - T4 (destroy objects): bash follows the defs' own
//!   `bash_result` chains — the partition door t_door_c ->
//!   t_door_b -> t_door_frame -> destroyed (chain end `t_null`
//!   = open air), the window t_window -> t_window_frame ->
//!   t_window_empty -> open air; furniture with a bash_result
//!   takes one furn_set step (f_board_map -> f_board_map_b),
//!   furniture without one is removed (f_table, plan §3.2);
//!   exactly one Effects (HitFlash + Sound) per successful
//!   bash; LOS/lit recompute through the bashed opening (the
//!   sealed south chamber's tile is BLANK before the first
//!   bash, seen after it — t_door_b is TRANSPARENT); bashing
//!   the chain end (t_open_air, no bash_result) is a no-op
//!   turn; a creature's tile is not bashable (bump-attack owns
//!   creature tiles, S5). The engine-side fine-prop / vehicle
//!   halves live in crates/engine/tests/
//!   sim_destruction_gate.rs; vehicles are in
//!   crates/sim/tests/vehicle_gate.rs (T11).
//! - T8 (items: pickup & drop): the rock pile at (3,3) goes
//!   item_count 3 -> 2 -> 1 -> cleared with top_item stable
//!   until the last pickup; the carried list grows in step
//!   and the opened inventory grid spells the rocks (ties T2
//!   shut); a drop onto a different item's pile is refused
//!   (state unchanged, turn still consumed — the sim's
//!   attempted-action rule); drops restore the pile exactly.
//! - The 200-turn mirror invariant with bash/pickup/drop in
//!   the script (turn_gate's mirror + a vehicle channel).

use std::collections::HashMap;
use std::path::PathBuf;

use proto::{
    DeltaT, EffectKind, EffectsT, EntityT, InputKind, InputT, LitLevel, MessageT, SnapshotT,
    TileChangeT, VehicleT,
};
use sim::scenes::Spawn;
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

/// The engine's Escape key code (ui-share `KEY_ESCAPE`).
const KEY_ESCAPE: i32 = 0x1001;

fn esc_input() -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = KEY_ESCAPE;
    input.pressed = true;
    input
}

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
        self.input(&key_input(c))
    }

    fn input(&mut self, input: &InputT) -> Vec<MessageT> {
        let msgs = self.world.apply_input(&self.reg, input);
        for m in &msgs {
            match m {
                MessageT::Delta(d) => {
                    assert_eq!(d.seq, self.seq + 1, "delta seq");
                    assert_eq!(d.epoch, self.epoch, "delta epoch");
                    self.seq = d.seq;
                }
                MessageT::Snapshot(s) => {
                    assert_eq!(s.seq, self.seq + 1, "snapshot seq");
                    assert_eq!(s.epoch, self.epoch + 1, "snapshot epoch");
                    self.seq = s.seq;
                    self.epoch = s.epoch;
                }
                MessageT::LightUpdate(lu) => {
                    assert_eq!(lu.seq, self.seq + 1, "light seq");
                    assert_eq!(lu.epoch, self.epoch, "light epoch");
                    self.seq = lu.seq;
                }
                MessageT::EnvironmentUpdate(e) => {
                    assert_eq!(e.seq, self.seq + 1, "env seq");
                    assert_eq!(e.epoch, self.epoch, "env epoch");
                    self.seq = e.seq;
                }
                MessageT::Effects(fx) => {
                    assert_eq!(fx.seq, self.seq + 1, "effects seq");
                    self.seq = fx.seq;
                }
                MessageT::UiFrame(_) => {}
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

    /// Arm Bash with `s` (the game's smash binding) and strike
    /// in direction `dir`. Arming produces no message.
    fn bash(&mut self, dir: char) -> Vec<MessageT> {
        let arm = self.key('s');
        assert!(arm.is_empty(), "arming Bash produces no message");
        self.key(dir)
    }

    fn player(&self) -> [i32; 3] {
        self.world.player_pos().expect("scene has a player")
    }

    fn terrain_str(&self, pos: [i32; 3]) -> String {
        let id = self.world.terrain_at(pos).expect("terrain at pos");
        self.reg.string_of(Kind::Terrain, id).expect("terrain string").to_string()
    }

    fn furniture_str(&self, pos: [i32; 3]) -> Option<String> {
        let id = self.world.furniture_at(pos).expect("furniture at pos");
        if id == 0 {
            None
        } else {
            Some(self.reg.string_of(Kind::Furniture, id).expect("furniture string").to_string())
        }
    }

    fn tile_items(&self, pos: [i32; 3]) -> (String, u16) {
        let (b, i) = self.world.tile_at(pos).expect("tile in scene");
        let t = &self.world.blocks[b].tiles[i];
        let name = if t.top_item == 0 {
            String::new()
        } else {
            self.reg.string_of(Kind::Item, t.top_item).expect("item string").to_string()
        };
        (name, t.item_count)
    }

    fn carried(&self) -> Vec<(String, u16)> {
        self.world
            .carried_items()
            .iter()
            .map(|&(id, n)| {
                (
                    self.reg.string_of(Kind::Item, id).expect("carried string").to_string(),
                    n,
                )
            })
            .collect()
    }
}

fn change_at<'a>(d: &'a DeltaT, pos: [i32; 3]) -> Option<&'a TileChangeT> {
    d.tile_changes.as_ref()?.iter().find(|tc| {
        tc.pos.as_ref().is_some_and(|p| p.x == pos[0] && p.y == pos[1] && p.z == pos[2])
    })
}

fn delta_of(msgs: &[MessageT]) -> &DeltaT {
    msgs.iter()
        .find_map(|m| match m {
            MessageT::Delta(d) => Some(&**d),
            _ => None,
        })
        .expect("flush carries a Delta")
}

fn effects_of(msgs: &[MessageT]) -> Vec<&EffectsT> {
    msgs.iter()
        .filter_map(|m| match m {
            MessageT::Effects(fx) => Some(&**fx),
            _ => None,
        })
        .collect()
}

/// The bash chain the defs themselves declare for a terrain
/// string id: successive `bash_result`s, with the chain end
/// `t_null` rendered as the destroyed state `t_open_air`
/// (upstream: a bash result of t_null is "no post-destroy
/// terrain" — the tile is destroyed to open air,
/// map::bash_ter_furn in 0.I-1 src/map.cpp).
fn bash_chain(reg: &DefRegistry, start: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = start.to_string();
    loop {
        let def = reg.def_by_string(Kind::Terrain, &cur).expect("chain def exists");
        match def.bash_result.as_deref() {
            None => break,
            Some("t_null") => {
                out.push("t_open_air".to_string());
                break;
            }
            Some(next) => {
                out.push(next.to_string());
                cur = next.to_string();
            }
        }
    }
    out
}

/// Assert a flush is exactly one successful bash's output:
/// one Effects message holding one HitFlash + one Sound,
/// both at the struck tile.
fn assert_bash_effects(msgs: &[MessageT], target: [i32; 3]) {
    let fx = effects_of(msgs);
    assert_eq!(fx.len(), 1, "exactly one Effects message per bash");
    let events = fx[0].events.as_ref().expect("effects events");
    let flashes: Vec<_> = events.iter().filter(|e| e.kind == EffectKind::HitFlash).collect();
    let sounds: Vec<_> = events.iter().filter(|e| e.kind == EffectKind::Sound).collect();
    assert_eq!(flashes.len(), 1, "exactly one HitFlash per bash");
    assert_eq!(sounds.len(), 1, "exactly one Sound per bash");
    let at = |e: &proto::EffectEventT| {
        let p = e.from_pos.as_ref().expect("effect pos");
        [p.x, p.y, p.z]
    };
    assert_eq!(at(flashes[0]), target, "HitFlash at the bashed tile");
    assert_eq!(at(sounds[0]), target, "Sound at the bashed tile");
}

// ---------------------------------------------------------------------
// T4
// ---------------------------------------------------------------------

#[test]
fn t4_bash_door_chain_and_los_through_opening() {
    let (mut drv, _snap) = Driver::new("destructible_street", |_| {});
    assert_eq!(drv.player(), [5, 4, 0]);
    // The sealed south chamber: its tile behind the partition
    // door has never been seen.
    assert_eq!(
        drv.world.lit_at([7, 7, 0]),
        Some(LitLevel::BLANK),
        "the tile behind the closed door starts BLANK"
    );
    // Walk to (7,5), adjacent to the door at (7,6).
    drv.delta('j');
    drv.delta('l');
    drv.delta('l');
    assert_eq!(drv.player(), [7, 5, 0]);
    assert_eq!(drv.terrain_str([7, 6, 0]), "t_door_c");

    // The expected chain comes from the defs, not from wishes.
    let chain = bash_chain(&drv.reg, "t_door_c");
    assert_eq!(chain, ["t_door_b", "t_door_frame", "t_open_air"]);

    for (step, want) in chain.iter().enumerate() {
        let msgs = drv.bash('j');
        let d = delta_of(&msgs);
        let tc = change_at(d, [7, 6, 0]).expect("bash emits a TileChange for the door");
        assert_eq!(
            drv.reg.string_of(Kind::Terrain, tc.terrain),
            Some(want.as_str()),
            "bash step {}: door terrain follows the def chain",
            step + 1
        );
        assert_bash_effects(&msgs, [7, 6, 0]);
        if step == 0 {
            // t_door_b is TRANSPARENT: the sealed chamber is
            // lit/seen through the bashed opening now.
            assert_ne!(
                drv.world.lit_at([7, 7, 0]),
                Some(LitLevel::BLANK),
                "after the first bash the tile behind the door is seen/lit"
            );
        }
    }
    // The chain end is destroyed: open air (assert the
    // terrain only — open air at z 0 is a hole, not a floor).
    assert_eq!(drv.terrain_str([7, 6, 0]), "t_open_air");
}

#[test]
fn t4_bash_window_chain_and_unbashable_noop() {
    let (mut drv, _snap) = Driver::new("destructible_street", |_| {});
    // To (7,3), adjacent to the window at (7,2) — routed
    // around the board-map tile (6,4), which blocks stepping.
    drv.delta('j');
    drv.delta('l');
    drv.delta('l');
    drv.delta('k');
    drv.delta('k');
    assert_eq!(drv.player(), [7, 3, 0]);
    let chain = bash_chain(&drv.reg, "t_window");
    assert_eq!(chain, ["t_window_frame", "t_window_empty", "t_open_air"]);
    for want in &chain {
        let msgs = drv.bash('k');
        let d = delta_of(&msgs);
        let tc = change_at(d, [7, 2, 0]).expect("bash emits a TileChange for the window");
        assert_eq!(drv.reg.string_of(Kind::Terrain, tc.terrain), Some(want.as_str()));
        assert_bash_effects(&msgs, [7, 2, 0]);
    }
    // Chain end: t_open_air has NO bash_result — one more bash
    // is a no-op turn: a Delta arrives (the turn happened) but
    // it carries no change for the tile and no Effects.
    let turn_before = drv.world.game_turn;
    let msgs = drv.bash('k');
    let d = delta_of(&msgs);
    assert!(change_at(d, [7, 2, 0]).is_none(), "unbashable bash changes nothing");
    assert!(effects_of(&msgs).is_empty(), "unbashable bash emits no Effects");
    assert_eq!(drv.world.game_turn, turn_before + 1, "the no-op bash still consumes the turn");
    assert_eq!(drv.terrain_str([7, 2, 0]), "t_open_air");
}

#[test]
fn t4_bash_furniture_step_and_removal() {
    let (mut drv, _snap) = Driver::new("destructible_street", |_| {});
    // f_table at (4,4), west of the spawn: NO bash_result in
    // the defs -> bashing removes it (plan §3.2).
    assert_eq!(drv.furniture_str([4, 4, 0]).as_deref(), Some("f_table"));
    let msgs = drv.bash('h');
    let d = delta_of(&msgs);
    let tc = change_at(d, [4, 4, 0]).expect("bash emits a TileChange for the table tile");
    assert_eq!(tc.furniture, 0, "table removed: furniture 0 in the TileChange");
    assert_eq!(drv.furniture_str([4, 4, 0]), None);
    assert_bash_effects(&msgs, [4, 4, 0]);

    // f_board_map at (6,4), east: bash_result f_board_map_b —
    // one furn_set step; f_board_map_b has no bash_result, so
    // the next bash removes it.
    assert_eq!(drv.furniture_str([6, 4, 0]).as_deref(), Some("f_board_map"));
    let msgs = drv.bash('l');
    let d = delta_of(&msgs);
    let tc = change_at(d, [6, 4, 0]).expect("bash emits a TileChange for the board tile");
    assert_eq!(
        drv.reg.string_of(Kind::Furniture, tc.furniture),
        Some("f_board_map_b"),
        "board takes one furn_set step along its bash_result"
    );
    assert_bash_effects(&msgs, [6, 4, 0]);
    let msgs = drv.bash('l');
    let d = delta_of(&msgs);
    let tc = change_at(d, [6, 4, 0]).expect("second bash emits a TileChange");
    assert_eq!(tc.furniture, 0, "bashed board (no bash_result) is removed");
    assert_eq!(drv.furniture_str([6, 4, 0]), None);
}

#[test]
fn t4_bash_creature_tile_is_not_bashable() {
    // A zombie standing north of the player: bashing its tile
    // does nothing to the tile or the zombie — the bump
    // attack (S5) owns creature tiles.
    let (mut drv, _snap) = Driver::new("destructible_street", |s| {
        s.spawns.push(Spawn {
            type_id: "mon_zombie".to_string(),
            pos: [5, 3, 0],
            hp_frac: 255,
            facing: "NONE".to_string(),
            overlays: Vec::new(),
            flags: 0,
            sprite: None,
        });
    });
    let zid = drv
        .world
        .entities
        .iter()
        .find(|e| e.sprite_id.as_deref() == Some("mon_zombie"))
        .map(|e| e.id)
        .expect("zombie spawned");
    let hp_before = drv.world.current_hp(zid);
    let msgs = drv.bash('k');
    let d = delta_of(&msgs);
    assert!(
        change_at(d, [5, 3, 0]).is_none(),
        "bashing a creature's tile changes no tile"
    );
    // No HitFlash lands ON the zombie's tile (the zombie's
    // own attack flashes the PLAYER's tile — a different
    // tile; its attack Sound legitimately originates at the
    // zombie, so only HitFlashes are checked here).
    for fx in effects_of(&msgs) {
        for e in fx.events.as_ref().expect("events") {
            if e.kind != EffectKind::HitFlash {
                continue;
            }
            let p = e.from_pos.as_ref().expect("pos");
            assert_ne!(
                [p.x, p.y, p.z],
                [5, 3, 0],
                "no bash HitFlash on the creature's tile"
            );
        }
    }
    assert_eq!(drv.world.current_hp(zid), hp_before, "bash does not damage the creature");
    assert_eq!(drv.terrain_str([5, 3, 0]), "t_floor");
}

// ---------------------------------------------------------------------
// T8
// ---------------------------------------------------------------------

#[test]
fn t8_pickup_and_drop_rock_pile() {
    let (mut drv, _snap) = Driver::new("destructible_street", |_| {});
    assert!(drv.carried().is_empty(), "the scene starts empty-handed");
    // Walk to the rock pile at (3,3): west onto the table tile
    // (passable), west onto the stick tile, north onto the pile.
    drv.delta('h');
    drv.delta('h');
    drv.delta('k');
    assert_eq!(drv.player(), [3, 3, 0]);
    assert_eq!(drv.tile_items([3, 3, 0]), ("rock".to_string(), 3));

    // Pickup x3: count 3 -> 2 -> 1 -> cleared; top_item stable
    // until the last unit leaves.
    for (step, want_count) in [(2u16, 1u16), (1, 2), (0, 3)] {
        let d = drv.delta('g');
        let tc = change_at(&d, [3, 3, 0]).expect("pickup emits a TileChange for the pile");
        assert_eq!(tc.item_count, step, "pile count after pickup");
        if step > 0 {
            assert_eq!(
                drv.reg.string_of(Kind::Item, tc.top_item),
                Some("rock"),
                "top_item stable while the pile lasts"
            );
        } else {
            assert_eq!(tc.top_item, 0, "last pickup clears top_item");
        }
        assert_eq!(drv.carried(), vec![("rock".to_string(), want_count)]);
    }
    // Pickup on the now-empty tile: no-op turn, nothing changes.
    let d = drv.delta('g');
    assert!(change_at(&d, [3, 3, 0]).is_none(), "empty-tile pickup changes nothing");
    assert_eq!(drv.carried(), vec![("rock".to_string(), 3)]);

    // The inventory UI lists the rocks (ties T2 shut): the
    // rendered grid spells the carried stack.
    let msgs = drv.key('i');
    assert_eq!(msgs.len(), 1, "opening the inventory is one UiFrame");
    let rows = drv.world.ui_grid().row_strings();
    assert!(
        rows.iter().any(|r| r.contains("rock") && r.contains("× 3")),
        "inventory grid must spell 'rock × 3':\n{}",
        rows.join("\n")
    );
    drv.input(&esc_input());

    // Drop refusal: stand on the stick pile (3,4) and try to
    // drop a rock — the lossy pile cannot hold two kinds.
    drv.delta('j');
    assert_eq!(drv.player(), [3, 4, 0]);
    let turn_before = drv.world.game_turn;
    let d = drv.delta('d');
    assert!(
        change_at(&d, [3, 4, 0]).is_none(),
        "refused drop changes no tile"
    );
    assert_eq!(drv.tile_items([3, 4, 0]), ("stick".to_string(), 1));
    assert_eq!(drv.carried(), vec![("rock".to_string(), 3)], "refused drop keeps the stack");
    assert_eq!(
        drv.world.game_turn,
        turn_before + 1,
        "refused drop still consumes the turn (attempted-action rule)"
    );

    // Back on the emptied pile tile, drops restore it exactly.
    drv.delta('k');
    assert_eq!(drv.player(), [3, 3, 0]);
    for want in 1..=3u16 {
        let d = drv.delta('d');
        let tc = change_at(&d, [3, 3, 0]).expect("drop emits a TileChange for the pile");
        assert_eq!(tc.item_count, want);
        assert_eq!(drv.reg.string_of(Kind::Item, tc.top_item), Some("rock"));
    }
    assert_eq!(drv.tile_items([3, 3, 0]), ("rock".to_string(), 3));
    assert!(drv.carried().is_empty(), "all rocks dropped");
    // Drop with empty hands: no-op turn.
    let d = drv.delta('d');
    assert!(change_at(&d, [3, 3, 0]).is_none(), "empty-hands drop changes nothing");
}

// ---------------------------------------------------------------------
// 200-turn mirror invariant, S6 script (bash / pickup / drop)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct TileMirror {
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

#[derive(Debug, Default, PartialEq)]
struct Mirror {
    tiles: HashMap<[i32; 3], TileMirror>,
    entities: HashMap<u64, EntityT>,
    vehicles: HashMap<u64, VehicleT>,
    memory: HashMap<[i32; 3], (u32, u32)>,
}

fn mirror_from_snapshot(snap: &SnapshotT) -> Mirror {
    let mut m = Mirror::default();
    for b in snap.blocks.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
        let o = b.origin.as_ref().expect("origin");
        for i in 0..144usize {
            let at =
                |arr: &Option<Vec<u32>>| arr.as_ref().and_then(|v| v.get(i)).copied().unwrap_or(0);
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
    for b in snap.memory_blocks.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
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

fn mirror_apply(m: &mut Mirror, msg: &MessageT) {
    match msg {
        MessageT::Snapshot(s) => *m = mirror_from_snapshot(s),
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
                    m.memory.insert(pos, (tc.terrain, tc.furniture));
                }
                m.tiles.insert(
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
            for e in d.entity_updates.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                m.entities.insert(e.id, e.clone());
            }
            for id in d.entity_removed.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                m.entities.remove(id);
            }
            for v in d.vehicle_updates.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                m.vehicles.insert(v.id, v.clone());
            }
            for id in d.vehicle_removed.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                m.vehicles.remove(id);
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
                if let Some(t) = m.tiles.get_mut(&pos) {
                    t.lit = lit;
                    t.seen = lu.seen.as_ref().and_then(|v| v.get(i)).copied().unwrap_or(0);
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
                    if let Some(t) = m.tiles.get(&pos) {
                        let tf = (t.terrain, t.furniture);
                        m.memory.insert(pos, tf);
                    }
                }
            }
        }
        MessageT::EnvironmentUpdate(_) | MessageT::Effects(_) | MessageT::UiFrame(_) => {}
        other => panic!("turn produced a non-turn message: {other:?}"),
    }
}

#[test]
fn mirror_invariant_200_turns_with_destruction() {
    let (mut drv, snap) = Driver::new("destructible_street", |_| {});
    let mut mirror = mirror_from_snapshot(&snap);

    // A fixed script (no RNG): bash the table/board/door/
    // window, pick up and drop the rocks, wander the room —
    // deterministic by construction.
    const PATTERN: &[char] = &[
        's', 'h', 's', 'l', 's', 'l', 'h', 'h', 'k', 'g', 'g', 'd', 'j', 'd', 'k', 'g', 'g', 'g',
        'd', 'd', 'd', 'l', 'l', 'j', 's', 'j', 's', 'j', 'h', 'k', 's', 'k', 's', 'k', '.', '.',
    ];
    let mut turns = 0usize;
    let mut inputs = 0usize;
    while turns < 200 {
        let c = PATTERN[inputs % PATTERN.len()];
        inputs += 1;
        assert!(inputs < 20_000, "pattern stopped producing turns");
        let msgs = drv.key(c);
        if msgs.is_empty() {
            continue; // arming key: not a turn
        }
        turns += 1;
        for msg in &msgs {
            mirror_apply(&mut mirror, msg);
        }
        let fresh = drv.world.build_snapshot(&drv.reg, 0, 0);
        let fresh_mirror = mirror_from_snapshot(&fresh);
        assert_eq!(
            mirror, fresh_mirror,
            "mirror diverged from the world after turn {turns} (key {c:?})"
        );
    }
    assert_eq!(turns, 200);
}
