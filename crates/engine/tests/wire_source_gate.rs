//! Phase 11 follow-up (Lane A) gate — the wire-source engine
//! change: `CaptureArgs::wire` seats a world built from wire
//! data (`TileDefs` + `Snapshot`) into the one frame loop, for
//! both renderers, and the `--connect` session seats its live
//! (delta-fed) world the same way.
//!
//! Three proofs:
//!
//! 1. **Pixel parity, wire vs wire.** The sim's `stub_house`
//!    Snapshot and the fixture `proto::samples::sample_snapshot()`
//!    are rendered through the NEW wire path with the S1 parity
//!    gate's canonical-palette resolver. S1 proved the two
//!    snapshots build identical worlds; identical worlds through
//!    one deterministic frame composition must give identical
//!    pixels — asserted byte-for-byte, raster and RT.
//!
//! 2. **The legacy fixture comparison is a named finding, not
//!    a silent relaxation.** The pre-Lane-A capture fixture
//!    named "stub-house" (`world::build_fake_house`) is NOT the
//!    same scene as the wire sample house, and does not become
//!    one through this change: the two were authored separately
//!    (door at (6,11) vs (5,11); windows at (3,0)/(0,6)/(11,6)
//!    vs (2,0) only) and are stamped by different models (the
//!    fixture's `set_tile` shapes — walls/doors/windows a full
//!    3-block level, a roof slab band at z 3 — vs the wire
//!    builder's rule that every terrain stamps its level's
//!    1-block floor slab, standing height on the wire path
//!    being the fine layer's job). The world-level test below
//!    enumerates the ENTIRE cell diff between the two maps and
//!    pins it to exactly those three causes; the pixel test
//!    then asserts the two captures differ (the comparison is
//!    meaningful) without pretending they should match.
//!
//! 3. **Connect seating.** A real sim-server (street_block)
//!    plus the engine client over a real socket: the session
//!    captures its first frame from the live wire world at the
//!    first Snapshot, walks the player out of the house
//!    (`jjojjj` — S7's proven walk), and captures the end frame
//!    from the delta-fed live world after the drain. The end
//!    frame must differ from the first (the door opened, the
//!    camera follows the player), both must be non-blank, and
//!    the final live state must show the door open at (5,8)
//!    and the player at (5,9,0).
//!
//! GPU tests follow the golden gate's convention: no wgpu
//! adapter = a loud skip, never a pass.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::client::{
    spawn_connect, ConnectCaptures, ConnectConfig, ConnectInput, SharedWireScene,
};
use engine::wire::WireScene;
use engine::world::build_fake_house;
use mesher::block::{block_for_tile, TILE_DOOR, TILE_FLOOR_WOOD, TILE_WALL, TILE_WINDOW};
use mesher::grid::VoxelSampler;
use mesher::{Block, Pass, WorldScale};
use proto::samples::{sample_snapshot, sample_tiledefs};
use proto::{SnapshotT, TileDefsT, TileKind};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

fn tmp_out(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wire-gate-{}-{tag}.png", std::process::id()))
}

// ---------------------------------------------------------------------
// S1 parity helpers (mirrored from sim_world_parity_gate: ids are
// per-kind on the sim side, hand-assigned on the fixture side, so
// both sides resolve through a canonical palette keyed by the
// terrain string id — any remaining difference is a real world
// difference, never an id-numbering artefact).
// ---------------------------------------------------------------------

struct Side {
    defs: TileDefsT,
    snap: SnapshotT,
    terrain_strings: HashMap<u32, String>,
}

fn terrain_strings(defs: &TileDefsT) -> HashMap<u32, String> {
    let mut m = HashMap::new();
    for d in defs.defs.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        if d.kind == TileKind::Terrain {
            if let Some(s) = &d.string_id {
                m.insert(d.id, s.clone());
            }
        }
    }
    m
}

fn sim_side(scene_name: &str) -> Side {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("sim scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    let defs = reg.tiledefs().clone();
    let terrain_strings = terrain_strings(&defs);
    Side {
        defs,
        snap,
        terrain_strings,
    }
}

fn fixture_side() -> Side {
    let defs = sample_tiledefs();
    let terrain_strings = terrain_strings(&defs);
    Side {
        defs,
        snap: sample_snapshot(),
        terrain_strings,
    }
}

fn canonical_palette(a: &Side, b: &Side) -> HashMap<String, u32> {
    let mut names: BTreeSet<&str> = BTreeSet::new();
    for side in [a, b] {
        for s in side.terrain_strings.values() {
            names.insert(s.as_str());
        }
    }
    names
        .into_iter()
        .enumerate()
        .map(|(i, s)| (s.to_string(), i as u32 + 1))
        .collect()
}

fn block_for_side<'a>(
    side: &'a Side,
    palette: &'a HashMap<String, u32>,
) -> impl Fn(u32) -> Block + 'a {
    move |id: u32| {
        if id == 0 {
            return Block::AIR;
        }
        let string = side
            .terrain_strings
            .get(&id)
            .unwrap_or_else(|| panic!("terrain id {id} has no string id"));
        let canonical = palette[string];
        Block {
            id: canonical,
            color: [canonical as f32 / 64.0, 0.5, 0.5, 1.0],
            pass: Pass::Opaque,
            tex_layers: None,
        }
    }
}

fn wire_args(
    side: &Side,
    palette: &HashMap<String, u32>,
    renderer: RendererKind,
    out: PathBuf,
) -> CaptureArgs {
    let mut args = CaptureArgs::default();
    args.snapshot = "wire".to_string();
    args.wire = Some(WireScene::from_wire(
        &side.defs,
        &side.snap,
        &block_for_side(side, palette),
    ));
    args.renderer = renderer;
    args.size = (640, 360);
    args.cutaway_z = 0;
    args.out = out;
    args
}

/// Run one capture; `None` = no adapter (the caller loud-skips).
fn capture(args: &CaptureArgs) -> Option<Vec<u8>> {
    match run_capture(args) {
        Ok(_) => Some(std::fs::read(&args.out).expect("capture png written")),
        Err(CaptureError::NoAdapter) => None,
        Err(e) => panic!("capture failed: {e}"),
    }
}

fn loud_skip(what: &str) {
    eprintln!("SKIP {what}: no wgpu adapter available");
    println!("SKIP {what}: no wgpu adapter available");
}

// ---------------------------------------------------------------------
// 1 + 2 (pixel half): wire-vs-wire parity; legacy differs.
// ---------------------------------------------------------------------

#[test]
fn wire_pixel_parity_sim_vs_fixture() {
    let sim = sim_side("stub_house");
    let fixture = fixture_side();
    let palette = canonical_palette(&sim, &fixture);

    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let sim_png = tmp_out(&format!("sim-{}", renderer.name()));
        let fix_png = tmp_out(&format!("fix-{}", renderer.name()));
        let Some(sim_bytes) = capture(&wire_args(&sim, &palette, renderer, sim_png)) else {
            loud_skip("wire_pixel_parity_sim_vs_fixture");
            return;
        };
        let fix_bytes = capture(&wire_args(&fixture, &palette, renderer, fix_png))
            .expect("adapter vanished between captures");
        assert_eq!(
            sim_bytes,
            fix_bytes,
            "{}: sim stub_house and fixture sample_snapshot differ through the wire path",
            renderer.name()
        );
        println!(
            "{}: wire pixel parity — sim == fixture ({} png bytes)",
            renderer.name(),
            sim_bytes.len()
        );
    }

    // The legacy fixture path renders a DIFFERENT house (see
    // the header finding): assert the captures differ, so the
    // parity above is a meaningful comparison, and report the
    // pixel diff count for the record.
    let legacy_png = tmp_out("legacy");
    let mut legacy_args = CaptureArgs::default();
    legacy_args.snapshot = "stub-house".to_string();
    legacy_args.size = (640, 360);
    legacy_args.cutaway_z = 0;
    legacy_args.out = legacy_png.clone();
    let legacy_bytes = capture(&legacy_args).expect("adapter vanished between captures");
    let fix_bytes = std::fs::read(tmp_out("fix-raster")).expect("wire fixture capture exists");
    assert_ne!(
        legacy_bytes, fix_bytes,
        "legacy stub-house fixture unexpectedly matches the wire sample house"
    );
    let a = image::open(&legacy_png).expect("legacy png").to_rgba8();
    let b = image::open(tmp_out("fix-raster"))
        .expect("wire png")
        .to_rgba8();
    let diff = a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count();
    println!(
        "legacy fixture vs wire fixture: {diff} of {} pixels differ (named finding — different authoring + stamping model)",
        a.pixels().len()
    );
    assert!(diff > 0);
}

// ---------------------------------------------------------------------
// 2 (world half): the ENTIRE fixture-vs-wire cell diff, enumerated.
// ---------------------------------------------------------------------

#[test]
fn wire_fixture_world_diff_is_the_named_finding() {
    let fixture_map = build_fake_house(WorldScale::default());

    // The wire sample house built with a resolver that maps
    // each terrain string to its fixture twin block, so block
    // identity is apples-to-apples and every remaining diff is
    // geometry or authoring, never resolver colour.
    let defs = sample_tiledefs();
    let strings = terrain_strings(&defs);
    let twin = |s: &str| -> u32 {
        match s {
            "t_floor" => TILE_FLOOR_WOOD,
            "t_wall" => TILE_WALL,
            "t_door_c" | "t_door_o" => TILE_DOOR,
            "t_window" => TILE_WINDOW,
            other => panic!("sample terrain {other} has no fixture twin"),
        }
    };
    let wire_map = engine::snapshot_world::build_snapshot_world(
        &defs,
        &sample_snapshot(),
        WorldScale::default(),
        &|id: u32| {
            if id == 0 {
                Block::AIR
            } else {
                block_for_tile(twin(&strings[&id]))
            }
        },
    );

    // The six z=0 authoring diffs: the door sits one tile west
    // in the wire house, and the window placements share no
    // tile at all.
    let authoring: BTreeSet<(i32, i32)> = [(5, 11), (6, 11), (2, 0), (3, 0), (0, 6), (11, 6)]
        .into_iter()
        .collect();
    let mut diffs = 0usize;
    let mut unclassified: Vec<(i32, i32, i32)> = Vec::new();
    for z in 0..6 {
        for y in 0..12 {
            for x in 0..12 {
                let f = fixture_map.block_at(x, y, z);
                let w = wire_map.block_at(x, y, z);
                if f == w {
                    continue;
                }
                diffs += 1;
                let perimeter = x == 0 || x == 11 || y == 0 || y == 11;
                let classified = (z == 3) // roof band: fixture slabs, wire window has no z+1
                    || (perimeter && (z == 1 || z == 2)) // authoring diffs extended upward:
                    // both sides fill standing tiles since Lane G, so pure wall
                    // tiles now MATCH; what remains here is the door/window
                    // placement divergence rising through z 1..=2
                    || (!perimeter && (z == 1 || z == 2)) // Lane G interior fill: the
                    // wire builder fills a floor tile's whole level (the coarse
                    // column is the tile's full occupancy); the hand-authored
                    // fixture keeps interior headroom (air) above its floors
                    || (z == 0 && authoring.contains(&(x, y)));
                if !classified {
                    unclassified.push((x, y, z));
                }
            }
        }
    }
    assert!(
        unclassified.is_empty(),
        "fixture/wire diff has unclassified cells (the named finding no longer covers it): {unclassified:?}"
    );
    // 144 roof-band cells (z 3) + 100 interior tiles x 2
    // (Lane G fill, z 1..=2) + 6 authoring tiles x 2
    // (perimeter, z 1..=2) + 6 authoring cells (z 0) = 362.
    // Was 238 before Lane G: the interior-fill class added
    // 200, and the old 88-cell perimeter-height class
    // collapsed to 12 (the authoring tiles only) because
    // both sides now fill standing tiles — pure wall tiles
    // match for the first time.
    assert_eq!(diffs, 362, "fixture/wire world diff count changed");
    println!(
        "fixture vs wire world diff: {diffs} cells, all inside the named finding \
         (roof band 144, interior fill 200, authoring 6 + 12 upward)"
    );
}

// ---------------------------------------------------------------------
// 3: connect seating against a real sim-server.
// ---------------------------------------------------------------------

#[test]
fn connect_session_captures_live_wire_world() {
    let dir = std::env::temp_dir().join(format!("wire-gate-{}-session", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let socket = dir.join("sim.sock");
    let first_png = dir.join("first.png");
    let end_png = dir.join("end.png");

    let server_cfg = sim::server::ServerConfig {
        socket_path: socket.clone(),
        defs_path: Some(defs_path()),
        scene_path: scene_path("street_block"),
        weather: None,
        ui_shm_path: None,
        ui_start: None,
        seed: None,
        record_path: None,
    };
    let server = std::thread::spawn(move || sim::server::run_server(&server_cfg));

    let (tx, rx) = std::sync::mpsc::channel::<i32>();
    let share: SharedWireScene = std::sync::Arc::new(std::sync::Mutex::new(None));
    let client = spawn_connect(
        &socket,
        ConnectConfig {
            input: ConnectInput::Channel(rx),
            share: Some(share.clone()),
            ui_share: None,
            captures: Some(ConnectCaptures {
                first: Some(first_png.clone()),
                end: Some(end_png.clone()),
                size: (640, 360),
            }),
            models_dir: None,
            billboards_dir: None,
        },
    );

    // Wait for the first published wire scene (the first
    // accepted Snapshot), then walk S7's proven route out of
    // the street_block house: j j (to the door), o (arm open),
    // j j j (open, step through, step out).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while share.lock().expect("share lock").is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "session never published a wire scene"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    for b in b"jjojjj" {
        tx.send(*b as i32).expect("client input channel open");
    }
    drop(tx); // input close: drain, end capture, Shutdown.

    let report = client
        .join()
        .expect("client thread panicked")
        .expect("connect session failed");
    server
        .join()
        .expect("server thread panicked")
        .expect("sim-server exited cleanly");

    assert_eq!(
        report.player_pos,
        Some([5, 9, 0]),
        "player walk out of the house"
    );
    assert!(report.deltas >= 5, "walk produced {} deltas", report.deltas);
    assert_eq!(report.inputs_sent, 6);
    assert!(
        share.lock().expect("share lock").is_some(),
        "live wire scene still published at session end"
    );

    // The final live state shows the door open at (5,8): the
    // delta-fed snapshot, not the connect-time one.
    let live = &report.live;
    let snap = live.snapshot().expect("live snapshot");
    let defs = live.defs().expect("live defs");
    let strings = terrain_strings(defs);
    let mut door_string: Option<&str> = None;
    for b in snap.blocks.as_ref().expect("snapshot blocks") {
        let o = b.origin.as_ref().expect("block origin");
        if o.z == 0 && 5 >= o.x && 5 < o.x + 12 && 8 >= o.y && 8 < o.y + 12 {
            let idx = ((8 - o.y) * 12 + (5 - o.x)) as usize;
            let ter = b.terrain.as_ref().expect("terrain")[idx];
            door_string = strings.get(&ter).map(|s| s.as_str());
        }
    }
    assert_eq!(
        door_string,
        Some("t_door_o"),
        "door at (5,8) open in the live state"
    );

    // Captures: both from the live wire world.
    assert_eq!(report.captures.len(), 2, "first + end captures recorded");
    for outcome in &report.captures {
        if let Err(e) = outcome {
            if e.contains("no wgpu adapter") {
                loud_skip("connect_session_captures_live_wire_world (captures)");
                return;
            }
            panic!("live capture failed: {e}");
        }
    }
    let first = image::open(&first_png).expect("first png").to_rgba8();
    let end = image::open(&end_png).expect("end png").to_rgba8();
    let distinct = |img: &image::RgbaImage| -> usize {
        img.pixels().map(|p| p.0).collect::<BTreeSet<_>>().len()
    };
    assert!(
        distinct(&first) > 10,
        "first live capture is blank ({} distinct colors)",
        distinct(&first)
    );
    assert!(
        distinct(&end) > 10,
        "end live capture is blank ({} distinct colors)",
        distinct(&end)
    );
    let diff = first
        .pixels()
        .zip(end.pixels())
        .filter(|(x, y)| x != y)
        .count();
    println!(
        "connect seating: first vs end live capture — {diff} of {} pixels differ (door opened, camera followed the player)",
        first.pixels().len()
    );
    assert!(
        diff > 0,
        "end capture identical to first — the live world did not move"
    );
}
