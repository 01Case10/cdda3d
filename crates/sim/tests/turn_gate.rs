//! S2 gates: the turn loop, driven IN-PROCESS through
//! `World::apply_input` (no socket — the same entry point the
//! server loop uses, and the one S7's scenario runner will use).
//!
//! - T1 (world/wire half): street_block — bump a closed door
//!   (blocked), `o`+dir opens it (TileChange t_door_c -> t_door_o),
//!   step out, cross a submap boundary (fresh Snapshot, epoch+1),
//!   walk back in, `c`+dir closes it. Seq advances by exactly 1
//!   per turn throughout. lit/outside values are NOT asserted —
//!   recompute is S3.
//! - T5: the full door cycle, including no-op open/close turns
//!   producing NO TileChange for the door tile.
//! - The 200-turn mirror invariant: applying every Delta to a
//!   mirror of the initial Snapshot reproduces a fresh Snapshot
//!   after every single turn.

use std::collections::HashMap;
use std::path::PathBuf;

use proto::{
    DeltaT, EntityT, Facing, InputKind, InputT, LitLevel, MessageT, SnapshotT, TileChangeT,
};
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

/// A world plus the seq/epoch bookkeeping every gate asserts on:
/// each turn's message must carry seq = previous + 1 (no
/// regression, no duplicates, no gaps), Deltas carry the current
/// epoch, a Snapshot bumps it by exactly 1.
struct Driver {
    reg: DefRegistry,
    world: World,
    seq: u64,
    epoch: u64,
}

impl Driver {
    fn new(scene_name: &str) -> (Driver, SnapshotT) {
        let reg = DefRegistry::load(&defs_path()).expect("registry loads");
        let scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
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

    /// Send one key; returns the turn's message (None for arming
    /// keys), asserting the seq/epoch contract on any message.
    fn key(&mut self, c: char) -> Option<MessageT> {
        let msg = self.world.apply_input(&self.reg, &key_input(c));
        if let Some(m) = &msg {
            match m {
                MessageT::Delta(d) => {
                    assert_eq!(
                        d.seq,
                        self.seq + 1,
                        "delta seq must advance by 1 (key {c:?})"
                    );
                    assert_eq!(d.epoch, self.epoch, "delta epoch (key {c:?})");
                    self.seq = d.seq;
                }
                MessageT::Snapshot(s) => {
                    assert_eq!(
                        s.seq,
                        self.seq + 1,
                        "snapshot seq must advance by 1 (key {c:?})"
                    );
                    assert_eq!(
                        s.epoch,
                        self.epoch + 1,
                        "snapshot must bump epoch (key {c:?})"
                    );
                    self.seq = s.seq;
                    self.epoch = s.epoch;
                }
                other => panic!("a turn must produce a Delta or Snapshot, got {other:?}"),
            }
        }
        msg
    }

    fn delta(&mut self, c: char) -> Box<DeltaT> {
        match self.key(c) {
            Some(MessageT::Delta(d)) => d,
            other => panic!("key {c:?}: expected Delta, got {other:?}"),
        }
    }

    fn snapshot(&mut self, c: char) -> Box<SnapshotT> {
        match self.key(c) {
            Some(MessageT::Snapshot(s)) => s,
            other => panic!("key {c:?}: expected Snapshot, got {other:?}"),
        }
    }

    fn player(&self) -> [i32; 3] {
        self.world.player_pos().expect("scene has a player")
    }

    fn terrain_str(&self, pos: [i32; 3]) -> String {
        let id = self.world.terrain_at(pos).expect("pos is in the scene");
        self.reg
            .string_of(Kind::Terrain, id)
            .unwrap_or("<unknown>")
            .to_string()
    }

    fn player_facing(&self) -> Facing {
        let id = self.world.player_id.expect("player id");
        self.world
            .entities
            .iter()
            .find(|e| e.id == id)
            .expect("player entity")
            .facing
    }
}

fn change_at<'a>(d: &'a DeltaT, pos: [i32; 3]) -> Option<&'a TileChangeT> {
    d.tile_changes.as_ref()?.iter().find(|tc| {
        tc.pos
            .as_ref()
            .is_some_and(|p| p.x == pos[0] && p.y == pos[1] && p.z == pos[2])
    })
}

fn entity_update_at<'a>(d: &'a DeltaT, id: u64) -> Option<&'a EntityT> {
    d.entity_updates.as_ref()?.iter().find(|e| e.id == id)
}

const DOOR: [i32; 3] = [5, 8, 0];

#[test]
fn t1_outside_then_back_inside() {
    let (mut drv, snap) = Driver::new("street_block");
    assert_eq!(
        snap.blocks.as_ref().map_or(0, |b| b.len()),
        4,
        "2x2 submaps"
    );
    assert_eq!(drv.player(), [5, 5, 0], "player starts inside the house");
    let player_id = drv.world.player_id.unwrap();

    // Walk to the door.
    let d = drv.delta('s');
    assert_eq!(drv.player(), [5, 6, 0]);
    let up = entity_update_at(&d, player_id).expect("move must update the player entity");
    assert_eq!(up.pos.as_ref().map(|p| [p.x, p.y, p.z]), Some([5, 6, 0]));
    drv.delta('s');
    assert_eq!(drv.player(), [5, 7, 0]);

    // Bump the closed door: blocked, turn consumed, door unchanged.
    let d = drv.delta('s');
    assert_eq!(drv.player(), [5, 7, 0], "closed door blocks the step");
    assert!(
        d.tile_changes.as_ref().is_none_or(|v| v.is_empty()),
        "a bump changes no tiles: {:?}",
        d.tile_changes
    );
    if let Some(up) = entity_update_at(&d, player_id) {
        assert_eq!(
            up.pos.as_ref().map(|p| [p.x, p.y, p.z]),
            Some([5, 7, 0]),
            "a bump must not move the player"
        );
    }
    assert_eq!(drv.terrain_str(DOOR), "t_door_c", "bump does not auto-open");

    // o + south opens it.
    assert!(drv.key('o').is_none(), "arming Open produces no message");
    let d = drv.delta('s');
    assert_eq!(
        drv.player(),
        [5, 7, 0],
        "the direction key applies the action, not a step"
    );
    let tc = change_at(&d, DOOR).expect("open must emit a TileChange for the door tile");
    assert_eq!(
        drv.reg.string_of(Kind::Terrain, tc.terrain),
        Some("t_door_o"),
        "door terrain follows open_result"
    );
    assert_eq!(drv.terrain_str(DOOR), "t_door_o");

    // Step out onto the grass.
    drv.delta('s');
    assert_eq!(drv.player(), [5, 8, 0], "standing in the doorway");
    drv.delta('s');
    assert_eq!(drv.player(), [5, 9, 0], "outside the house");

    // Walk east along y=9 and cross the x=12 submap boundary.
    for x in 6..12 {
        let d = drv.delta('d');
        assert_eq!(drv.player(), [x, 9, 0]);
        assert_eq!(d.epoch, 1, "no shift before the boundary");
    }
    assert_eq!(
        drv.player_facing(),
        Facing::RIGHT,
        "eastward step faces RIGHT"
    );
    let snap = drv.snapshot('d');
    assert_eq!(drv.player(), [12, 9, 0], "crossed into the next submap");
    assert_eq!(snap.epoch, 2, "map shift bumps the epoch");
    let pe = snap
        .entities
        .as_ref()
        .and_then(|es| es.iter().find(|e| e.id == player_id))
        .expect("snapshot carries the player");
    assert_eq!(pe.pos.as_ref().map(|p| [p.x, p.y, p.z]), Some([12, 9, 0]));

    // Walk back west: crossing back is another shift.
    let snap = drv.snapshot('a');
    assert_eq!(drv.player(), [11, 9, 0]);
    assert_eq!(snap.epoch, 3);
    assert_eq!(
        drv.player_facing(),
        Facing::LEFT,
        "westward step faces LEFT"
    );
    for x in (5..11).rev() {
        drv.delta('a');
        assert_eq!(drv.player(), [x, 9, 0]);
    }

    // Re-enter and close the door behind.
    drv.delta('k');
    assert_eq!(drv.player(), [5, 8, 0], "back in the (open) doorway");
    drv.delta('k');
    assert_eq!(drv.player(), [5, 7, 0], "back inside");
    assert!(drv.key('c').is_none(), "arming Close produces no message");
    let d = drv.delta('s');
    let tc = change_at(&d, DOOR).expect("close must emit a TileChange for the door tile");
    assert_eq!(
        drv.reg.string_of(Kind::Terrain, tc.terrain),
        Some("t_door_c"),
        "door terrain follows close_result"
    );
    assert_eq!(drv.terrain_str(DOOR), "t_door_c");
}

#[test]
fn t5_door_cycle_and_noops() {
    let (mut drv, _snap) = Driver::new("street_block");

    // To the door, open it, step through.
    drv.delta('s');
    drv.delta('s');
    assert_eq!(drv.player(), [5, 7, 0]);
    assert!(drv.key('o').is_none());
    let d = drv.delta('s');
    assert!(change_at(&d, DOOR).is_some(), "open emits a TileChange");
    drv.delta('s');
    drv.delta('s');
    assert_eq!(drv.player(), [5, 9, 0], "through the door and outside");

    // Close behind (north = back toward the door).
    assert!(drv.key('c').is_none());
    let d = drv.delta('k');
    let tc = change_at(&d, DOOR).expect("close emits a TileChange");
    assert_eq!(
        drv.reg.string_of(Kind::Terrain, tc.terrain),
        Some("t_door_c")
    );

    // Close AGAIN: no-op — the Delta must carry no TileChange
    // for the door tile.
    assert!(drv.key('c').is_none());
    let d = drv.delta('k');
    assert!(
        change_at(&d, DOOR).is_none(),
        "closing a closed door must not emit a TileChange: {:?}",
        d.tile_changes
    );
    assert_eq!(drv.terrain_str(DOOR), "t_door_c");

    // Re-open, then open AGAIN: same no-op rule.
    assert!(drv.key('o').is_none());
    let d = drv.delta('k');
    let tc = change_at(&d, DOOR).expect("re-open emits a TileChange");
    assert_eq!(
        drv.reg.string_of(Kind::Terrain, tc.terrain),
        Some("t_door_o")
    );
    assert!(drv.key('o').is_none());
    let d = drv.delta('k');
    assert!(
        change_at(&d, DOOR).is_none(),
        "opening an open door must not emit a TileChange: {:?}",
        d.tile_changes
    );
    assert_eq!(drv.terrain_str(DOOR), "t_door_o");
}

// ---------------------------------------------------------------------
// 200-turn mirror invariant
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
}

#[derive(Debug, Default, PartialEq)]
struct Mirror {
    tiles: HashMap<[i32; 3], TileMirror>,
    entities: HashMap<u64, EntityT>,
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
                m.tiles.insert(
                    [p.x, p.y, p.z],
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
                    },
                );
            }
            for e in d
                .entity_updates
                .as_ref()
                .map(|v| v.as_slice())
                .unwrap_or(&[])
            {
                m.entities.insert(e.id, e.clone());
            }
            for id in d
                .entity_removed
                .as_ref()
                .map(|v| v.as_slice())
                .unwrap_or(&[])
            {
                m.entities.remove(id);
            }
        }
        other => panic!("turn produced a non-turn message: {other:?}"),
    }
}

#[test]
fn mirror_invariant_200_turns() {
    let (mut drv, snap) = Driver::new("street_block");
    let mut mirror = mirror_from_snapshot(&snap);

    // A fixed patrol pattern (no RNG): walks the house, bumps
    // walls, opens the door once per cycle, waits — deterministic
    // by construction.
    const PATTERN: &[char] = &[
        's', 's', 'o', 's', 'd', 'd', 'w', 'w', 'a', 'a', 'k', 'j', 'j', 'h', 'h', 'l', '.', 'd',
        'a', 's', 'w',
    ];
    let mut turns = 0usize;
    let mut inputs = 0usize;
    while turns < 200 {
        let c = PATTERN[inputs % PATTERN.len()];
        inputs += 1;
        assert!(inputs < 10_000, "pattern stopped producing turns");
        let Some(msg) = drv.key(c) else {
            continue; // arming key: not a turn
        };
        turns += 1;
        mirror_apply(&mut mirror, &msg);
        // After EVERY turn: mirror == a fresh snapshot of the world.
        let fresh = drv.world.build_snapshot(&drv.reg, 0, 0);
        let fresh_mirror = mirror_from_snapshot(&fresh);
        assert_eq!(
            mirror, fresh_mirror,
            "mirror diverged from the world after turn {turns} (key {c:?})"
        );
    }
    assert_eq!(turns, 200);
}
