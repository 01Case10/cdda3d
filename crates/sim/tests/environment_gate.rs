//! S3 gates, stage 2 (turn level): T1 full, T6, and the
//! LightUpdate folding rule.
//!
//! T1 note (deviation from the slice brief, grounded in
//! source): the brief's path expectation was "lit LIT -> BRIGHT
//! -> LIT". A faithful apparent_light_at port cannot produce
//! that: LIGHT_SOURCE_BRIGHT == LIGHT_AMBIENT_LIT == 10.0
//! upstream (0.I-1 lightmap.h:10,19), so the BRIGHT branch
//! subsumes LIT, and the noon interior baseline is
//! inside_light_level = 4.0, which classifies LOW. The honest
//! sequence is LOW -> BRIGHT -> LOW, asserted here.

use std::path::PathBuf;

use proto::{
    DeltaT, EnvironmentUpdateT, InputKind, InputT, LitLevel, MessageT, SnapshotT, TileChangeT,
};
use sim::scenes::WeatherStep;
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

struct Driver {
    reg: DefRegistry,
    world: World,
    seq: u64,
}

impl Driver {
    fn new(scene_name: &str, mutate: impl FnOnce(&mut Scene)) -> (Driver, SnapshotT) {
        let reg = DefRegistry::load(&defs_path()).expect("registry loads");
        let mut scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
        mutate(&mut scene);
        let mut world = World::from_scene(&reg, &scene).expect("world builds");
        let snap = world.snapshot(&reg);
        assert_eq!(snap.seq, 1);
        (Driver { reg, world, seq: 1 }, snap)
    }

    fn key(&mut self, c: char) -> Vec<MessageT> {
        let msgs = self.world.apply_input(&self.reg, &key_input(c));
        for m in &msgs {
            let s = match m {
                MessageT::Delta(d) => d.seq,
                MessageT::Snapshot(s) => s.seq,
                MessageT::LightUpdate(lu) => lu.seq,
                MessageT::EnvironmentUpdate(e) => e.seq,
                other => panic!("unexpected message {other:?}"),
            };
            assert_eq!(s, self.seq + 1, "seq must advance by 1 per message");
            self.seq = s;
        }
        msgs
    }

    fn fresh_tile(&self, pos: [i32; 3]) -> (LitLevel, u8, bool) {
        let snap = self.world.build_snapshot(&self.reg, 0, 0);
        tile_of(&snap, pos)
    }

    fn player(&self) -> [i32; 3] {
        self.world.player_pos().expect("player")
    }
}

fn tile_of(snap: &SnapshotT, pos: [i32; 3]) -> (LitLevel, u8, bool) {
    for b in snap.blocks.as_ref().expect("blocks") {
        let o = b.origin.as_ref().expect("origin");
        if o.z == pos[2] && (o.x..o.x + 12).contains(&pos[0]) && (o.y..o.y + 12).contains(&pos[1]) {
            let i = ((pos[1] - o.y) * 12 + (pos[0] - o.x)) as usize;
            return (
                b.lit.as_ref().unwrap()[i],
                b.seen.as_ref().unwrap()[i],
                b.outside.as_ref().unwrap()[i],
            );
        }
    }
    panic!("pos {pos:?} not in snapshot");
}

fn delta_of(msgs: &[MessageT]) -> &DeltaT {
    msgs.iter()
        .find_map(|m| match m {
            MessageT::Delta(d) => Some(d.as_ref()),
            _ => None,
        })
        .expect("flush contains a Delta")
}

fn change_at<'a>(d: &'a DeltaT, pos: [i32; 3]) -> Option<&'a TileChangeT> {
    d.tile_changes.as_ref()?.iter().find(|tc| {
        tc.pos
            .as_ref()
            .is_some_and(|p| p.x == pos[0] && p.y == pos[1] && p.z == pos[2])
    })
}

const DOOR: [i32; 3] = [5, 8, 0];

#[test]
fn t1_full_outside_lit_and_back() {
    let (mut drv, _snap) = Driver::new("street_block", |_| {});

    // Inside, door closed: player tile not outside, LOW.
    let (lit, seen, outside) = drv.fresh_tile([5, 5, 0]);
    assert_eq!((lit, seen, outside), (LitLevel::LOW, 255, false));

    // To the door, bump it, open it.
    drv.key('s');
    drv.key('s');
    assert_eq!(drv.player(), [5, 7, 0]);
    drv.key('s'); // bump
    assert_eq!(drv.player(), [5, 7, 0]);
    let (lit, _, outside) = drv.fresh_tile([5, 7, 0]);
    assert_eq!(
        (lit, outside),
        (LitLevel::LOW, false),
        "still inside, door closed"
    );
    assert!(drv.key('o').is_empty());
    let msgs = drv.key('s');
    let tc = change_at(delta_of(&msgs), DOOR).expect("open emits the door TileChange");
    assert_eq!(
        drv.reg.string_of(Kind::Terrain, tc.terrain),
        Some("t_door_o")
    );
    assert!(!tc.outside, "the door tile is stamped not-outside");
    assert_eq!(tc.lit, LitLevel::BRIGHT, "the door tile is sunlit");

    // Step out: the player tile flips to outside + BRIGHT.
    drv.key('s');
    assert_eq!(drv.player(), [5, 8, 0]);
    drv.key('s');
    assert_eq!(drv.player(), [5, 9, 0]);
    let (lit, seen, outside) = drv.fresh_tile([5, 9, 0]);
    assert_eq!(
        (lit, seen, outside),
        (LitLevel::BRIGHT, 255, true),
        "player tile outside in full sun"
    );

    // Walk the yard and come back in; close the door behind.
    drv.key('d');
    drv.key('a');
    drv.key('k');
    drv.key('k');
    assert_eq!(drv.player(), [5, 7, 0]);
    let (lit, _, outside) = drv.fresh_tile([5, 7, 0]);
    assert_eq!(lit, LitLevel::BRIGHT, "sun streams through the open door");
    assert!(!outside);
    assert!(drv.key('c').is_empty());
    let msgs = drv.key('s');
    let tc = change_at(delta_of(&msgs), DOOR).expect("close emits the door TileChange");
    assert_eq!(
        drv.reg.string_of(Kind::Terrain, tc.terrain),
        Some("t_door_c")
    );
    let (lit, _, outside) = drv.fresh_tile([5, 7, 0]);
    assert_eq!(
        (lit, outside),
        (LitLevel::LOW, false),
        "back inside with the door closed: LOW, not outside"
    );
}

#[test]
fn t1_memory_blocks_after_leaving() {
    let (mut drv, _snap) = Driver::new("three_rooms", |_| {});
    let mid = [10, 5, 0];
    let mid_east = [13, 5, 0];
    let east = [20, 5, 0];

    // Middle and east rooms start unseen.
    assert_eq!(drv.fresh_tile(mid), (LitLevel::BLANK, 0, false));
    assert_eq!(drv.fresh_tile(east), (LitLevel::BLANK, 0, false));

    // Visit the middle room.
    for _ in 0..4 {
        drv.key('d');
    }
    assert_eq!(drv.player(), [7, 5, 0]);
    assert!(drv.key('o').is_empty());
    let msgs = drv.key('d');
    let tc = change_at(delta_of(&msgs), mid)
        .expect("opening the door relights the middle room in the same Delta");
    assert_eq!(tc.lit, LitLevel::LOW, "middle room becomes visible lit LOW");
    assert_eq!(tc.seen, 255);
    for _ in 0..6 {
        drv.key('d');
    }
    assert_eq!(drv.player(), [13, 5, 0]);
    assert_eq!(drv.fresh_tile(mid), (LitLevel::LOW, 255, false));

    // Return and close the door.
    for _ in 0..6 {
        drv.key('a');
    }
    assert_eq!(drv.player(), [7, 5, 0]);
    assert!(drv.key('c').is_empty());
    drv.key('d');

    // Live view: the middle room is BLANK again — but NOT erased:
    // it must be in the snapshot's memory blocks as MEMORIZED,
    // while the never-visited east room has no memory at all.
    let snap = drv.world.build_snapshot(&drv.reg, 0, 0);
    assert_eq!(tile_of(&snap, mid), (LitLevel::BLANK, 0, false));
    let mem = snap.memory_blocks.as_ref().expect("memory blocks exist");
    let mem_tile = |pos: [i32; 3]| -> (LitLevel, String) {
        for b in mem {
            let o = b.origin.as_ref().expect("origin");
            if o.z == pos[2]
                && (o.x..o.x + 12).contains(&pos[0])
                && (o.y..o.y + 12).contains(&pos[1])
            {
                let i = ((pos[1] - o.y) * 12 + (pos[0] - o.x)) as usize;
                let ter = b.terrain.as_ref().unwrap()[i];
                return (
                    b.lit.as_ref().unwrap()[i],
                    drv.reg
                        .string_of(Kind::Terrain, ter)
                        .unwrap_or("<none>")
                        .to_string(),
                );
            }
        }
        panic!("no memory block covers {pos:?}");
    };
    assert_eq!(
        mem_tile(mid),
        (LitLevel::MEMORIZED, "t_floor".to_string()),
        "visited middle room is memorized"
    );
    assert_eq!(
        mem_tile(mid_east),
        (LitLevel::MEMORIZED, "t_floor".to_string()),
        "the part of the middle room in the east submap too"
    );
    assert_eq!(
        mem_tile(east),
        (LitLevel::BLANK, "<none>".to_string()),
        "never-visited east room: no memory"
    );
}

#[test]
fn t6_scripted_day_cadence_and_storm() {
    let (mut drv, snap) = Driver::new("street_block", |s| {
        s.turn = Some(0);
        s.weather_script = vec![
            WeatherStep {
                turn: 50_000,
                weather: "lightning".to_string(),
            },
            WeatherStep {
                turn: 80_000,
                weather: "clear".to_string(),
            },
        ];
    });
    assert_eq!(snap.environment.as_ref().unwrap().turn, 0);

    let msgs = drv.world.advance_clock(&drv.reg, 86_400);
    // Fold the advance's messages into the seq contract.
    let mut envs: Vec<&EnvironmentUpdateT> = Vec::new();
    for m in &msgs {
        let s = match m {
            MessageT::Delta(d) => d.seq,
            MessageT::Snapshot(s) => s.seq,
            MessageT::LightUpdate(lu) => lu.seq,
            MessageT::EnvironmentUpdate(e) => e.seq,
            other => panic!("unexpected message {other:?}"),
        };
        assert_eq!(s, drv.seq + 1, "seq advances by 1 per message");
        drv.seq = s;
        if let MessageT::EnvironmentUpdate(e) = m {
            envs.push(e);
        }
    }

    // Cadence accounting (start turn 0, last env sent at 0):
    // time-only carriers at every 600 s boundary = 83 before the
    // storm (600..=49800), the storm's on-change carrier at
    // 50000 (cadence clock resets), 49 time-only in the storm
    // (50600..=79400), the clear on-change carrier at 80000
    // (coincides with a cadence boundary — one carrier), then
    // 10 time-only to 86400 (80600..=86000). Total 144.
    assert_eq!(envs.len(), 144, "on-change (2) + time-only (142) carriers");
    assert_eq!(envs.first().unwrap().turn, 600);
    let storm_start = envs
        .iter()
        .find(|e| e.turn == 50_000)
        .expect("storm on-change carrier");
    assert_eq!(storm_start.weather_id.as_deref(), Some("weather_lightning"));
    assert_eq!(storm_start.precip_class, 3, "lightning precip is heavy");
    assert!(storm_start.lightning_active);
    assert_eq!(storm_start.sight_penalty, 1.25);
    assert_eq!(storm_start.light_modifier, -45.0);
    for e in envs.iter().filter(|e| e.turn > 50_000 && e.turn < 80_000) {
        assert_eq!(
            e.weather_id.as_deref(),
            Some("weather_lightning"),
            "carrier at turn {} is inside the storm segment",
            e.turn
        );
        assert_eq!(e.precip_class, 3);
        assert!(e.lightning_active);
    }
    let storm_end = envs
        .iter()
        .find(|e| e.turn == 80_000)
        .expect("clear on-change carrier");
    assert_eq!(storm_end.weather_id.as_deref(), Some("weather_clear"));
    assert_eq!(storm_end.precip_class, 0);
    assert!(!storm_end.lightning_active);

    // Sun altitude flips sign across dawn and dusk.
    let dawn = envs
        .windows(2)
        .find(|w| w[0].sun_altitude_deg < 0.0 && w[1].sun_altitude_deg >= 0.0)
        .expect("altitude flips up at dawn");
    // The -1 deg sunrise is ~05:58 (turn ~21484, asserted in
    // visibility_gate); altitude itself crosses 0 a few minutes
    // later, so the carrier-to-carrier flip lands at 21600 ->
    // 22200.
    assert!(
        dawn[0].turn >= 20_400 && dawn[1].turn <= 22_200,
        "dawn flip between turns {} and {}",
        dawn[0].turn,
        dawn[1].turn
    );
    let dusk = envs
        .windows(2)
        .find(|w| w[0].turn > 43_200 && w[0].sun_altitude_deg >= 0.0 && w[1].sun_altitude_deg < 0.0)
        .expect("altitude flips down at dusk");
    assert!(
        dusk[0].turn > 60_000 && dusk[1].turn < 70_000,
        "dusk flip between turns {} and {}",
        dusk[0].turn,
        dusk[1].turn
    );

    assert_eq!(drv.world.game_turn, 86_400);
    let snap = drv.world.build_snapshot(&drv.reg, 0, 0);
    let env = snap.environment.as_ref().unwrap();
    assert_eq!(env.turn, 86_400);
    assert_eq!(env.time_of_day_sec, 0.0, "midnight of day 1");
}

#[test]
fn light_update_folding_at_dawn() {
    // 05:16 of day 0: the sun is climbing through the
    // exponential dawn band; exterior fracs move every turn
    // while nothing structural changes — those flushes must
    // fold to LightUpdates, never Deltas.
    let (mut drv, _snap) = Driver::new("street_block", |s| {
        s.turn = Some(19_000);
    });
    let mut light_only_flushes = 0;
    let mut total_light_updates = 0;
    for _ in 0..400 {
        let msgs = drv.key('.');
        assert!(!msgs.is_empty(), "a turn always produces a flush");
        if msgs.iter().all(|m| matches!(m, MessageT::LightUpdate(_))) {
            light_only_flushes += 1;
            for m in &msgs {
                let MessageT::LightUpdate(lu) = m else {
                    unreachable!()
                };
                total_light_updates += 1;
                assert_eq!(lu.lit.as_ref().map(|v| v.len()), Some(144));
                assert_eq!(lu.seen.as_ref().map(|v| v.len()), Some(144));
                assert_eq!(lu.light_frac.as_ref().map(|v| v.len()), Some(144));
                assert_eq!(lu.light_tint.as_ref().map(|v| v.len()), Some(432));
            }
        }
    }
    assert!(
        light_only_flushes >= 3,
        "dawn relight folds to LightUpdate-only flushes (got {light_only_flushes})"
    );
    assert!(total_light_updates >= 3);
}
