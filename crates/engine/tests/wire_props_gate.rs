//! Phase 11 follow-up (Lane G, Part 2) gate: furniture fine
//! models in the LIVE wire path.
//!
//! The builder under test is `engine::wire::prop_scene_for_wire`
//! — it derives a `PropScene` from a wire snapshot's furniture
//! arrays through the one real voxelizer model table, the
//! derivation Lane F proved at gate level for the fixture
//! path. This gate proves the LIVE path stamps the same
//! thing, two ways:
//!
//! 1. Layer equality (the Lane F method): stamp the builder's
//!    PropScene through `stamp_furniture` with the shared
//!    colour resolver, and stamp an independent in-gate Lane F
//!    derivation (placements from the furniture arrays, models
//!    by string id) the same way — the two FineLayers must be
//!    cell-for-cell identical.
//! 2. Render: a wire scene WITH the prop scene renders
//!    (raster and RT) differently from the same scene without
//!    it, and non-blank — the frame paths really stamp it.
//!
//! The model table is a machine-local artifact
//! (`assets/*` is gitignored), so a machine without it
//! loud-skips, per the Lane F / frame_gate convention.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::OnceLock;

use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::materials::{MaterialTable, PropModelData};
use engine::snapshot_world::{
    build_snapshot_world, prop_block_for, stamp_furniture, FurniturePlacement, PropModelCells,
};
use engine::textures::PropColorSet;
use engine::wire::{palette_resolver, prop_scene_for_wire, WireScene};
use mesher::fine::{FineLayer, FineVolume, FINE};
use mesher::{Block, WorldScale};
use proto::{SnapshotT, TileDefsT, TileKind};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

fn ultica_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1")
}

fn model_table() -> &'static MaterialTable {
    static TABLE: OnceLock<MaterialTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        MaterialTable::load(&ultica_dir()).expect("assets/ultica_run1 model table loads")
    })
}

fn loud_skip(gate: &str) -> bool {
    if ultica_dir().join("materials.json").exists() {
        return false;
    }
    eprintln!(
        "================================================================\n\
         {gate}: NOT RUN — assets/ultica_run1 (the voxelizer's real model table) is absent.\n\
         This run proves NOTHING about live wire props. The gate only\n\
         counts on a machine with the voxelizer's real output present.\n\
         ================================================================"
    );
    true
}

fn street_block_wire() -> (TileDefsT, SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path("street_block")).expect("street_block loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    (reg.tiledefs().clone(), snap)
}

fn furniture_strings(defs: &TileDefsT) -> HashMap<u32, String> {
    let mut m = HashMap::new();
    for d in defs.defs.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        if d.kind == TileKind::Furniture {
            if let Some(s) = &d.string_id {
                m.insert(d.id, s.clone());
            }
        }
    }
    m
}

/// The independent Lane F derivation: placements from the
/// furniture arrays (wire furn ids), string ids alongside.
fn derive_placements(defs: &TileDefsT, snap: &SnapshotT) -> Vec<(FurniturePlacement, String)> {
    let strings = furniture_strings(defs);
    let min_z = snap
        .blocks
        .as_ref()
        .and_then(|b| b.iter().filter_map(|b| b.origin.as_ref().map(|o| o.z)).min())
        .unwrap_or(0);
    let mut out = Vec::new();
    for b in snap.blocks.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        let o = b.origin.as_ref().expect("snapshot block origin");
        for (i, &f) in b
            .furniture
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if f != 0 {
                out.push((
                    FurniturePlacement {
                        x: o.x + (i % 12) as i32,
                        y: o.y + (i / 12) as i32,
                        level: (o.z - min_z) as u32,
                        furn_id: f,
                        quarter_turns: 0,
                    },
                    strings.get(&f).cloned().unwrap_or_else(|| format!("#{f}")),
                ));
            }
        }
    }
    out
}

fn volume_hash(v: &FineVolume) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(&v.z0.to_le_bytes());
    mix(&v.depth_z.to_le_bytes());
    for ly in 0..FINE {
        for lx in 0..FINE {
            for fz in v.z0..v.z0 + v.depth_z {
                match v.get(lx, ly, fz) {
                    None => mix(&[0]),
                    Some(b) => {
                        mix(&[1]);
                        mix(format!("{b:?}").as_bytes());
                    }
                }
            }
        }
    }
    h
}

/// Cell-for-cell layer comparison (Lane F's method): tile
/// sets, volume z-ranges, solid counts, and per-cell blocks
/// (located via the volume hash). Returns the diff count.
fn compare_layers(a: &FineLayer, b: &FineLayer) -> (usize, Vec<String>) {
    let mut total = 0usize;
    let mut diffs: Vec<String> = Vec::new();
    let at: BTreeSet<(i32, i32)> = a.tiles().collect();
    let bt: BTreeSet<(i32, i32)> = b.tiles().collect();
    if at != bt {
        total += 1;
        diffs.push(format!("stamped tile sets differ: {at:?} vs {bt:?}"));
    }
    for tile in at.union(&bt) {
        let mut av: Vec<&FineVolume> = a.volumes_for_tile(*tile).iter().collect();
        let mut bv: Vec<&FineVolume> = b.volumes_for_tile(*tile).iter().collect();
        av.sort_by_key(|v| v.z0);
        bv.sort_by_key(|v| v.z0);
        if av.len() != bv.len() {
            total += 1;
            diffs.push(format!(
                "tile {tile:?}: volume count {} vs {}",
                av.len(),
                bv.len()
            ));
            continue;
        }
        for (x, y) in av.iter().zip(bv.iter()) {
            if x.z0 != y.z0 || x.depth_z != y.depth_z || volume_hash(x) != volume_hash(y) {
                total += 1;
                diffs.push(format!(
                    "tile {tile:?}: volume differs (z0 {} vs {}, solids {} vs {})",
                    x.z0,
                    y.z0,
                    x.solid_count(),
                    y.solid_count()
                ));
            }
        }
    }
    (total, diffs)
}

#[test]
fn live_wire_props_match_the_lane_f_derivation() {
    if loud_skip("wire_props_gate") {
        return;
    }
    let (defs, snap) = street_block_wire();
    let table = model_table();
    let scale = WorldScale::default();

    // The builder under test.
    let props = prop_scene_for_wire(&defs, &snap, table)
        .expect("street_block has a modelled furniture (f_table)");
    assert_eq!(
        props.placements.len(),
        1,
        "street_block carries exactly one furniture: {props:?}"
    );
    let p = props.placements[0];
    assert_eq!(
        (p.x, p.y, p.level),
        (4, 4, 0),
        "the f_table placement sits at its scene tile"
    );

    // The independent Lane F derivation of the same snapshot.
    let derived = derive_placements(&defs, &snap);
    assert_eq!(derived.len(), 1, "one furniture cell in the snapshot");

    // One shared colour resolver over both sides' models
    // (Lane F's rule: same colours -> same synthetic ids).
    let mut model_refs: Vec<&PropModelData> = Vec::new();
    for (_, m) in &props.models {
        model_refs.push(m);
    }
    for (_, sid) in &derived {
        if let Some(m) = table.get(sid).and_then(|mt| mt.model.as_ref()) {
            if !model_refs.iter().any(|r| std::ptr::eq(*r, m)) {
                model_refs.push(m);
            }
        }
    }
    let colors = PropColorSet::from_models(model_refs.iter().copied());
    let color_block = prop_block_for(&colors, 40_000, 0);

    // Fixture-derivation side: adapters keyed by WIRE id.
    let mut fix_adapters: Vec<(u32, PropModelCells<'_>)> = Vec::new();
    for (pl, sid) in &derived {
        if let Some(model) = table.get(sid).and_then(|m| m.model.as_ref()) {
            fix_adapters.push((pl.furn_id, PropModelCells::new(model, &color_block)));
        }
    }
    let fix_resolve = |id: u32| -> Option<&dyn mesher::fine::ModelCells> {
        fix_adapters
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, a)| a as &dyn mesher::fine::ModelCells)
    };
    // Live side: adapters keyed by the PropScene's synthetic
    // ids, models taken from the PropScene itself.
    let live_adapters: Vec<(u32, PropModelCells<'_>)> = props
        .models
        .iter()
        .map(|(id, m)| (*id, PropModelCells::new(m, &color_block)))
        .collect();
    let live_resolve = |id: u32| -> Option<&dyn mesher::fine::ModelCells> {
        live_adapters
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, a)| a as &dyn mesher::fine::ModelCells)
    };

    let fix_placements: Vec<FurniturePlacement> = derived.iter().map(|(pl, _)| *pl).collect();
    let mut fix_map = build_snapshot_world(&defs, &snap, scale, &palette_resolver(&defs));
    let mut fix_layer = FineLayer::new(scale.blocks_per_z_level as i32 * FINE);
    let fix_report = stamp_furniture(
        &mut fix_map,
        &mut fix_layer,
        &fix_placements,
        scale,
        &fix_resolve,
    )
    .expect("fixture-derivation stamp succeeds");

    let mut live_map = build_snapshot_world(&defs, &snap, scale, &palette_resolver(&defs));
    let mut live_layer = FineLayer::new(scale.blocks_per_z_level as i32 * FINE);
    let live_report = stamp_furniture(
        &mut live_map,
        &mut live_layer,
        &props.placements,
        scale,
        &live_resolve,
    )
    .expect("live-builder stamp succeeds");

    assert!(fix_report.stamped_cells > 0, "the table model stamps cells");
    assert_eq!(
        live_report.stamped_cells, fix_report.stamped_cells,
        "live and derivation stamp the same number of cells"
    );
    let (diffs, msgs) = compare_layers(&live_layer, &fix_layer);
    assert_eq!(
        diffs, 0,
        "live wire props must equal the Lane F derivation cell-for-cell: {msgs:?}"
    );
    println!(
        "wire props: live builder == Lane F derivation ({} cells, {} volumes)",
        live_report.stamped_cells, live_report.volumes
    );
}

#[test]
fn live_wire_renders_include_the_furniture() {
    if loud_skip("wire_props_gate renders") {
        return;
    }
    let (defs, snap) = street_block_wire();
    let table = model_table();
    let props = prop_scene_for_wire(&defs, &snap, table).expect("props derive");
    let bare = WireScene::from_wire(&defs, &snap, &palette_resolver(&defs));
    let with = bare.clone().with_props(Some(props));

    let capture = |scene: WireScene, renderer: RendererKind, tag: &str| -> (Vec<u8>, usize) {
        let out = std::env::temp_dir().join(format!(
            "cdda-wire-props-gate-{}-{tag}.png",
            std::process::id()
        ));
        let mut args = CaptureArgs::default();
        args.snapshot = "wire".to_string();
        args.wire = Some(scene);
        args.renderer = renderer;
        args.camera_target = (4.5, 4.5);
        args.cutaway_z = 0;
        args.distance = 20.0;
        args.size = (480, 270);
        args.out = out.clone();
        let outcome = run_capture(&args).expect("wire capture renders");
        (std::fs::read(&out).expect("png written"), outcome.distinct_colors)
    };

    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let (bare_png, _) = capture(bare.clone(), renderer, "bare");
        let (props_png, distinct) = capture(with.clone(), renderer, "props");
        assert!(
            distinct > 10,
            "{renderer:?} wire+props capture must not be blank ({distinct} distinct colours)"
        );
        assert_ne!(
            bare_png, props_png,
            "{renderer:?}: the furniture prop scene must change the live wire render"
        );
    }
    println!("wire props: raster and RT live renders include the furniture");
}
