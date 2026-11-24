//! Phase 8 release gates over dummy data: 8.6 (overmap flow),
//! 8.7 (mod content across crates), 8.12 (upgrade), 8.13 (crash
//! reporting). The in-game/Windows/hardware halves are the
//! project's standing conditions, recorded in docs/phase8.

use std::collections::HashSet;
use std::path::PathBuf;

use hardcases::{classify_transition, DefTable, Transition};
use proto::{TileDefT, TileDefsT, TileKind, UiMode, VehicleT};
use release::{check_asset_cache, write_stamp, CacheVerdict, PackageStamp};
use ui_share::frame::{frame_notification, validate_frame};
use ui_share::input::{ClickRoute, DragRoute, InputRouter, Modifiers};
use ui_share::store::DirtyRect;
use ui_share::{TerrainRect, UiShmStore};
use vehicles::VehicleStore;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda-release-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------- 8.6

/// The overmap flow, end to end at the data level: the game sends
/// a UiInput frame whose pixels are the overmap; the engine
/// validates it against the store, shows it (the UI texture IS the
/// overmap — there is no terrain rect), input routes as UI (never
/// a world pick, never a camera drag); the next MapInput frame
/// restores world picking inside the terrain rect.
#[test]
fn gate_8_6_overmap_open_input_and_return() {
    let (w, h) = (64u32, 48u32);
    let store = UiShmStore::create_memory(w, h).unwrap();
    // Overmap pixels: dark blue field, bright marker at center.
    let mut px = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        px[i * 4] = 16;
        px[i * 4 + 1] = 16;
        px[i * 4 + 2] = 48;
        px[i * 4 + 3] = 255;
    }
    let marker = ((24 * w + 32) * 4) as usize;
    px[marker] = 255;
    px[marker + 1] = 255;
    px[marker + 2] = 0;
    store.write_frame(&px, DirtyRect::full(w, h)).unwrap();

    let snap = store.read_latest().unwrap();
    assert_eq!(
        snap.pixel(32, 24),
        [255, 255, 0, 255],
        "the UI texture carries the overmap"
    );
    let note = frame_notification(&snap, UiMode::UiInput, None);
    let validated = validate_frame(&store, &note).unwrap();
    assert_eq!(validated.mode, UiMode::UiInput);
    assert_eq!(
        validated.terrain_rect, None,
        "overmap covers the whole UI: no 3D window"
    );

    let router = InputRouter::new(UiMode::UiInput, None);
    assert!(
        !router.is_world_pick(32, 24),
        "no world picking while the overmap is up"
    );
    assert_eq!(router.route_drag(), DragRoute::Ignored);
    let mods = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    };
    match router.route_ui_click(32, 24, 0, true, mods) {
        ClickRoute::UiClick(_) => {}
        other => panic!("overmap click must route as UI input, got {other:?}"),
    }

    // Return to gameplay: a MapInput frame with a terrain rect.
    store.write_frame(&px, DirtyRect::full(w, h)).unwrap();
    let snap = store.read_latest().unwrap();
    let rect = TerrainRect {
        x: 16,
        y: 12,
        w: 32,
        h: 24,
    };
    let note = frame_notification(&snap, UiMode::MapInput, Some(rect));
    let validated = validate_frame(&store, &note).unwrap();
    assert_eq!(validated.mode, UiMode::MapInput);
    assert_eq!(validated.terrain_rect, Some(rect));
    let router = InputRouter::new(UiMode::MapInput, Some(rect));
    assert!(
        router.is_world_pick(32, 24),
        "inside the terrain rect: world picks again"
    );
    assert!(!router.is_world_pick(2, 2), "outside: UI territory");
    assert_eq!(router.route_drag(), DragRoute::Camera);
}

// ---------------------------------------------------------------- 8.7

/// A mod's content driven across all three Phase 8 stores at
/// once: unknown vpart ids are opaque to the vehicle store, an
/// unknown effect id still gets a visible emitter, the mod door
/// classifies. Nothing panics; nothing is dropped silently.
#[test]
fn gate_8_7_mod_content_across_stores() {
    let mut door = TileDefT::default();
    door.id = 300;
    door.string_id = Some("t_mod_door_c".into());
    door.kind = TileKind::Terrain;
    door.fg_ref = 950;
    door.open_result = Some("t_mod_door_o".into());
    let mut door_o = TileDefT::default();
    door_o.id = 301;
    door_o.string_id = Some("t_mod_door_o".into());
    door_o.kind = TileKind::Terrain;
    door_o.fg_ref = 951;
    let mut defs = TileDefsT::default();
    defs.defs = Some(vec![door, door_o]);
    let table = DefTable::from_tiledefs(&defs);
    assert_eq!(
        classify_transition(&table, proto::TileKind::Terrain, 300, 301),
        Transition::Open
    );

    let mut store = VehicleStore::new();
    let mut snap = proto::SnapshotT::default();
    let mut v = VehicleT::default();
    v.id = 5;
    let mut part = proto::VehiclePartStateT::default();
    part.vpart = 9999; // a mod vehicle part the base game lacks
    part.abs_pos = Some(proto::Vec3iT { x: 1, y: 2, z: 0 });
    part.mount = Some(proto::Vec3iT { x: 0, y: 0, z: 0 });
    part.hp_frac = 255;
    v.parts = Some(vec![part]);
    snap.vehicles = Some(vec![v]);
    store.apply_snapshot(&snap);
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(5).unwrap().parts[0].key.vpart, 9999);

    // Unknown mod effect ids resolve to a visible generic emitter,
    // never a silent drop (transient params contract).
    let p = transient::params_for(proto::EffectKind::Other, "mod_death_ray_mk2")
        .expect("unknown effects still render");
    assert_eq!(p.render_class, transient::CLASS_GENERIC);
    let _ = HashSet::<u32>::new(); // (report exactness is gated in hardcases)
}

// ---------------------------------------------------------------- 8.12

#[test]
fn gate_8_12_upgrade_invalidates_cache_and_resyncs_state() {
    let dir = temp_dir("upgrade");
    let cache = dir.join("assets");
    let v1 = PackageStamp {
        frontend_version: "0.1.0".into(),
        fork_commit: release::FORK_COMMIT.into(),
        protocol_version: proto::PROTOCOL_VERSION,
    };
    // Fresh install: no stamp → stale → generate → stamp written.
    assert!(matches!(
        check_asset_cache(&cache, &v1),
        CacheVerdict::Stale(_)
    ));
    write_stamp(&cache, &v1).unwrap();
    assert_eq!(check_asset_cache(&cache, &v1), CacheVerdict::Fresh);
    // Upgrade: the stamp no longer matches → regenerate.
    let v2 = PackageStamp {
        frontend_version: "0.2.0".into(),
        ..v1.clone()
    };
    match check_asset_cache(&cache, &v2) {
        CacheVerdict::Stale(why) => assert!(why.contains("build changed"), "{why}"),
        other => panic!("expected Stale after upgrade, got {other:?}"),
    }
    write_stamp(&cache, &v2).unwrap();
    assert_eq!(check_asset_cache(&cache, &v2), CacheVerdict::Fresh);

    // The old save: state built under v1 resyncs cleanly under v2
    // — a snapshot replaces wholesale, leaving no v1 residue.
    let mut store = VehicleStore::new();
    let mut snap1 = proto::SnapshotT::default();
    snap1.epoch = 1;
    let mut v = VehicleT::default();
    v.id = 77;
    snap1.vehicles = Some(vec![v]);
    store.apply_snapshot(&snap1);
    assert_eq!(store.len(), 1);
    let mut snap2 = proto::SnapshotT::default();
    snap2.epoch = 2; // the upgraded game re-bases on connect
    store.apply_snapshot(&snap2);
    assert_eq!(store.len(), 0, "no residue from the old version's state");
}

// ---------------------------------------------------------------- 8.13

#[test]
fn gate_8_13_crash_writes_a_versioned_report() {
    let dir = temp_dir("crash");
    let previous = release::install_panic_hook(dir.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        panic!("gate 8.13 probe");
    }));
    std::panic::set_hook(previous);
    assert!(result.is_err());
    let logs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("crash-")
        })
        .collect();
    assert_eq!(logs.len(), 1, "exactly one crash report written");
    let text = std::fs::read_to_string(&logs[0]).unwrap();
    assert!(text.contains("frontend: 0.1.0"), "{text}");
    assert!(
        text.contains(&format!("protocol: {}", proto::PROTOCOL_VERSION)),
        "{text}"
    );
    assert!(
        text.contains(release::FORK_COMMIT),
        "report names the pinned fork commit: {text}"
    );
    assert!(text.contains("gate 8.13 probe"), "{text}");
}
