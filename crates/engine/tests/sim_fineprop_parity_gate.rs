//! Phase 11 follow-up gate — furniture FINE-PROP parity
//! between the sim's wire data and the Phase 1 fixtures.
//!
//! The S1 gate (`sim_world_parity_gate`) proved the sim's
//! TileDefs + Snapshot build the same COARSE world as the
//! fixtures (ChunkMap cells, meshes, RT form). It never ran
//! the Phase 9 fine-prop path: whether a furniture tile fed
//! from the sim's wire stamps the same fine volume as the
//! same tile fed from the fixture's wire was left open.
//!
//! This gate closes that. For each shared scene (the three
//! scenes the S1 gate pairs — stub_house and both roofed
//! houses; street_block has no fixture counterpart in
//! `proto::samples`, so it cannot be paired):
//!
//! 1. placements are derived from EACH side's own Snapshot
//!    furniture arrays, the way the live loop derives them
//!    (the `sim_destruction_gate` shape): every non-zero
//!    furniture cell of every SubmapBlock becomes a
//!    `FurniturePlacement` at its tile, at the block's
//!    0-based level within the snapshot's z window;
//! 2. each placement's wire furniture id is resolved to a
//!    string id through THAT side's own TileDefs (per-kind
//!    ids collide across kinds; the string id is the only
//!    honest join key — the same convention
//!    `stamp_snapshot_terrain` uses for terrain), and the
//!    string id is resolved to a prop model through the
//!    voxelizer's real model table — `MaterialTable` loaded
//!    from `assets/ultica_run1`, the v2 output whose `models`
//!    table the Stage G drive built (351 furniture models);
//! 3. both sides stamp through
//!    `snapshot_world::stamp_furniture` into a fresh
//!    `FineLayer`, over the coarse world
//!    `build_snapshot_world` builds from the same wire pair,
//!    resolving model colours through ONE shared
//!    `PropColorSet` + `prop_block_for` resolver, so a
//!    stamped `Block` is a pure function of the model colour
//!    on both sides;
//! 4. the comparison: placement sets as (tile, level,
//!    string id) must be identical; the stamp reports must
//!    agree (stamped cells, volumes, skipped); and for every
//!    tile, the two layers' volumes (matched by z0) must be
//!    cell-for-cell equal — every `FineVolume::get` over the
//!    full 32x32 footprint and the volume's whole z depth.
//!    A per-volume FNV-1a hash over the same cells is
//!    printed as the compact fingerprint in the gate log.
//!
//! Non-vacuity: each scene's furniture (an `f_table`) has a
//! real model in the table, and the gate asserts both sides
//! actually stamped volumes with solid cells — a resolver
//! that silently resolved nothing on both sides would fail
//! here, not pass.
//!
//! Seating note: this stamps furniture only, on the coarse
//! world, with no terrain models in the layer — so
//! `stamp_furniture`'s Ground-model seating (fix 5b) finds
//! no ground volume on either side and both seat at the
//! slab-top origin (`fine_z_origin`). Terrain-model parity
//! is the terrain wiring gates' ground, not this one's.
//!
//! The model table is machine-local generated data
//! (`assets/*` is gitignored), so a machine without it loud-
//! skips, per the `frame_gate` convention: the run then
//! proves NOTHING about fine-prop parity.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::OnceLock;

use engine::materials::{MaterialTable, PropModelData};
use engine::snapshot_world::{
    build_snapshot_world, prop_block_for, stamp_furniture, FurniturePlacement, PropModelCells,
};
use engine::textures::PropColorSet;
use mesher::fine::{FineLayer, FineVolume, FINE};
use mesher::{Block, Pass, WorldScale};
use proto::samples::{
    sample_implicit_roof_snapshot, sample_roofed_house_snapshot, sample_roofed_tiledefs,
    sample_snapshot, sample_tiledefs,
};
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

/// The voxelizer's real model table, loaded once for the
/// whole gate binary (12k materials / 1.4k models is not a
/// per-test cost worth paying three times).
fn model_table() -> &'static MaterialTable {
    static TABLE: OnceLock<MaterialTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        MaterialTable::load(&ultica_dir()).expect("assets/ultica_run1 model table loads")
    })
}

fn loud_skip(gate: &str, reason: &str) -> bool {
    if ultica_dir().join("materials.json").exists() {
        return false;
    }
    eprintln!(
        "================================================================\n\
         {gate}: NOT RUN — {reason}\n\
         This run proves NOTHING about fine-prop parity. The gate only\n\
         counts on a machine with the voxelizer's real output present.\n\
         ================================================================"
    );
    true
}

// ---------------------------------------------------------------------
// Sides (the S1 parity gate's construction, plus furniture strings)
// ---------------------------------------------------------------------

struct Side {
    label: &'static str,
    defs: TileDefsT,
    snap: SnapshotT,
    terrain_strings: HashMap<u32, String>,
    furniture_strings: HashMap<u32, String>,
}

fn strings_of_kind(defs: &TileDefsT, kind: TileKind) -> HashMap<u32, String> {
    let mut m = HashMap::new();
    for d in defs.defs.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        if d.kind == kind {
            if let Some(s) = &d.string_id {
                m.insert(d.id, s.clone());
            }
        }
    }
    m
}

fn merge_defs(tables: &[TileDefsT]) -> TileDefsT {
    // The sample tables share one hand-assigned id space (the
    // S1 parity gate documents the same merge): later tables
    // only fill ids earlier ones lack.
    let mut out = tables[0].clone();
    let acc = out.defs.get_or_insert_with(Vec::new);
    let mut have: std::collections::HashSet<(TileKind, u32)> =
        acc.iter().map(|d| (d.kind, d.id)).collect();
    for t in &tables[1..] {
        for d in t.defs.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
            if have.insert((d.kind, d.id)) {
                acc.push(d.clone());
            }
        }
    }
    out
}

fn make_side(label: &'static str, defs: TileDefsT, snap: SnapshotT) -> Side {
    let terrain_strings = strings_of_kind(&defs, TileKind::Terrain);
    let furniture_strings = strings_of_kind(&defs, TileKind::Furniture);
    Side {
        label,
        defs,
        snap,
        terrain_strings,
        furniture_strings,
    }
}

fn sim_side(scene_name: &str) -> Side {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("sim scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    make_side("sim", reg.tiledefs().clone(), snap)
}

/// Canonical palette for the coarse build's `block_for`
/// (verbatim convention from the S1 parity gate): union of
/// both sides' terrain string ids, sorted, numbered from 1.
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
            .unwrap_or_else(|| panic!("{}: terrain id {id} has no string id", side.label));
        let canonical = palette[string];
        Block {
            id: canonical,
            color: [canonical as f32 / 64.0, 0.5, 0.5, 1.0],
            pass: Pass::Opaque,
            tex_layers: None,
        }
    }
}

// ---------------------------------------------------------------------
// Placement derivation + stamping
// ---------------------------------------------------------------------

fn z_range(snap: &SnapshotT) -> (i32, i32) {
    let mut lo = i32::MAX;
    let mut hi = i32::MIN;
    for b in snap.blocks.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        let z = b.origin.as_ref().map(|o| o.z).unwrap_or(0);
        lo = lo.min(z);
        hi = hi.max(z);
    }
    (lo, hi)
}

/// One derived placement plus the string id its wire id
/// resolved to through the side's own TileDefs.
struct Derived {
    placement: FurniturePlacement,
    string_id: String,
}

/// Derive placements from a snapshot's furniture arrays:
/// every non-zero furniture cell of every block, at the
/// block's 0-based level within the snapshot's z window.
fn derive_placements(side: &Side) -> Vec<Derived> {
    let (min_z, _) = z_range(&side.snap);
    let mut out = Vec::new();
    for b in side.snap.blocks.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        let o = b.origin.as_ref().expect("snapshot block origin");
        let level = (o.z - min_z) as u32;
        for (i, &f) in b
            .furniture
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if f != 0 {
                let string_id = side
                    .furniture_strings
                    .get(&f)
                    .unwrap_or_else(|| {
                        panic!("{}: furniture id {f} has no string id", side.label)
                    })
                    .clone();
                out.push(Derived {
                    placement: FurniturePlacement {
                        x: o.x + (i % 12) as i32,
                        y: o.y + (i / 12) as i32,
                        level,
                        furn_id: f,
                        quarter_turns: 0,
                    },
                    string_id,
                });
            }
        }
    }
    out
}

/// Stamp one side: build its coarse world, derive its
/// placements, resolve them through the shared model table
/// + colour resolver, stamp into a fresh layer. Returns the
/// layer, the stamp report, and the derived placements.
fn stamp_side(
    side: &Side,
    palette: &HashMap<String, u32>,
    table: &MaterialTable,
    color_block: &dyn Fn([u8; 3]) -> Block,
    scale: WorldScale,
) -> (FineLayer, mesher::fine::StampReport, Vec<Derived>) {
    let derived = derive_placements(side);
    let mut map = build_snapshot_world(&side.defs, &side.snap, scale, &block_for_side(side, palette));
    // One adapter per distinct wire id that resolves to a
    // model; ids whose string id has no model in the table
    // resolve to None (stamp_furniture counts them skipped).
    let mut adapters: Vec<(u32, PropModelCells<'_>)> = Vec::new();
    for d in &derived {
        if adapters.iter().any(|(id, _)| *id == d.placement.furn_id) {
            continue;
        }
        if let Some(model) = table
            .get(&d.string_id)
            .and_then(|m| m.model.as_ref())
        {
            adapters.push((
                d.placement.furn_id,
                PropModelCells::new(model, color_block),
            ));
        }
    }
    let resolve = |id: u32| -> Option<&dyn mesher::fine::ModelCells> {
        adapters
            .iter()
            .find(|(fid, _)| *fid == id)
            .map(|(_, a)| a as &dyn mesher::fine::ModelCells)
    };
    let placements: Vec<FurniturePlacement> = derived.iter().map(|d| d.placement).collect();
    let mut layer = FineLayer::new(scale.blocks_per_z_level as i32 * FINE);
    let report = stamp_furniture(&mut map, &mut layer, &placements, scale, &resolve)
        .expect("stamp succeeds");
    (layer, report, derived)
}

// ---------------------------------------------------------------------
// Volume comparison
// ---------------------------------------------------------------------

/// FNV-1a over every cell of a volume in (ly, lx, fz)
/// order, mixing each cell's Debug rendering (which covers
/// id, colour bits, pass and texture layers) — the compact
/// fingerprint printed in the gate log. The assertion
/// itself is the cell loop below, not this hash.
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

/// Compare two layers tile by tile, volume by volume, cell
/// by cell. Returns (diff count, rendered first diffs,
/// per-tile summary lines for the gate log).
fn compare_layers(
    sim: &FineLayer,
    fixture: &FineLayer,
) -> (usize, Vec<String>, Vec<String>) {
    let mut total = 0usize;
    let mut diffs: Vec<String> = Vec::new();
    let mut summary: Vec<String> = Vec::new();
    let sim_tiles: BTreeSet<(i32, i32)> = sim.tiles().collect();
    let fix_tiles: BTreeSet<(i32, i32)> = fixture.tiles().collect();
    if sim_tiles != fix_tiles {
        total += 1;
        diffs.push(format!(
            "stamped tile sets differ: sim {sim_tiles:?} vs fixture {fix_tiles:?}"
        ));
    }
    for tile in sim_tiles.union(&fix_tiles) {
        let mut sv: Vec<&FineVolume> = sim.volumes_for_tile(*tile).iter().collect();
        let mut fv: Vec<&FineVolume> = fixture.volumes_for_tile(*tile).iter().collect();
        sv.sort_by_key(|v| v.z0);
        fv.sort_by_key(|v| v.z0);
        if sv.len() != fv.len() {
            total += 1;
            diffs.push(format!(
                "tile {tile:?}: volume count sim {} vs fixture {}",
                sv.len(),
                fv.len()
            ));
            continue;
        }
        for (s, f) in sv.iter().zip(fv.iter()) {
            let (sh, fh) = (volume_hash(s), volume_hash(f));
            summary.push(format!(
                "tile {tile:?} z0 {} depth {}: solid sim {} fixture {}, hash sim {sh:#018x} fixture {fh:#018x}",
                s.z0,
                s.depth_z,
                s.solid_count(),
                f.solid_count(),
            ));
            if s.z0 != f.z0 || s.depth_z != f.depth_z {
                total += 1;
                diffs.push(format!(
                    "tile {tile:?}: volume z-range sim (z0 {}, depth {}) vs fixture (z0 {}, depth {})",
                    s.z0, s.depth_z, f.z0, f.depth_z
                ));
                continue;
            }
            if s.solid_count() != f.solid_count() {
                total += 1;
                diffs.push(format!(
                    "tile {tile:?} z0 {}: solid cells sim {} vs fixture {}",
                    s.z0,
                    s.solid_count(),
                    f.solid_count()
                ));
            }
            if sh != fh {
                // The hash disagrees: find the cells (the
                // hash alone doesn't say where).
                let mut cell_diffs = 0usize;
                for ly in 0..FINE {
                    for lx in 0..FINE {
                        for fz in s.z0..s.z0 + s.depth_z {
                            let (sb, fb) = (s.get(lx, ly, fz), f.get(lx, ly, fz));
                            if sb != fb {
                                cell_diffs += 1;
                                if diffs.len() < 50 {
                                    diffs.push(format!(
                                        "tile {tile:?} cell (lx {lx}, ly {ly}, fz {fz}): sim {sb:?} vs fixture {fb:?}"
                                    ));
                                }
                            }
                        }
                    }
                }
                total += cell_diffs;
                if cell_diffs == 0 {
                    total += 1;
                    diffs.push(format!(
                        "tile {tile:?} z0 {}: volume hashes differ ({sh:#x} vs {fh:#x}) with no differing cell — hash collision or nondeterminism",
                        s.z0
                    ));
                }
            }
        }
    }
    (total, diffs, summary)
}

// ---------------------------------------------------------------------
// The gate
// ---------------------------------------------------------------------

fn assert_scene_fineprop_parity(scene: &str, fixture_defs: TileDefsT, fixture_snap: SnapshotT) {
    if loud_skip(
        "sim_fineprop_parity_gate",
        "real UltiCa model table absent (assets/ultica_run1)",
    ) {
        return;
    }
    let table = model_table();
    let sim = sim_side(scene);
    let fixture = make_side("fixture", fixture_defs, fixture_snap);
    let palette = canonical_palette(&sim, &fixture);
    let scale = WorldScale::default();

    // One shared colour resolver for both sides: the colour
    // set is built from the union of the models either side
    // resolves, so a stamped Block is a pure function of the
    // model colour, identically on both sides.
    let mut models: BTreeMap<String, &PropModelData> = BTreeMap::new();
    for side in [&sim, &fixture] {
        for d in derive_placements(side) {
            if let Some(model) = table.get(&d.string_id).and_then(|m| m.model.as_ref()) {
                models.entry(d.string_id.clone()).or_insert(model);
            }
        }
    }
    let colors = PropColorSet::from_models(models.values().copied());
    let color_block = prop_block_for(&colors, 40_000, 0);

    let (sim_layer, sim_report, sim_derived) =
        stamp_side(&sim, &palette, table, &color_block, scale);
    let (fix_layer, fix_report, fix_derived) =
        stamp_side(&fixture, &palette, table, &color_block, scale);

    let mut all_diffs: Vec<String> = Vec::new();
    let mut total = 0usize;
    let note = |cond: bool, msg: String, total: &mut usize, diffs: &mut Vec<String>| {
        if !cond {
            *total += 1;
            diffs.push(msg);
        }
    };

    // The level derivation is only sound if both snapshots
    // span the same z window.
    note(
        z_range(&sim.snap) == z_range(&fixture.snap),
        format!(
            "{scene}: snapshot z ranges differ: sim {:?} vs fixture {:?}",
            z_range(&sim.snap),
            z_range(&fixture.snap)
        ),
        &mut total,
        &mut all_diffs,
    );

    // Placement sets as (tile, level, string id): the wire
    // furniture itself, joined through each side's own defs.
    let placement_set = |derived: &[Derived]| -> BTreeSet<(i32, i32, u32, String)> {
        derived
            .iter()
            .map(|d| {
                (
                    d.placement.x,
                    d.placement.y,
                    d.placement.level,
                    d.string_id.clone(),
                )
            })
            .collect()
    };
    let (sp, fp) = (placement_set(&sim_derived), placement_set(&fix_derived));
    note(
        sp == fp,
        format!("{scene}: placement sets differ: sim {sp:?} vs fixture {fp:?}"),
        &mut total,
        &mut all_diffs,
    );

    // Stamp reports must agree field for field (except
    // memory_bytes, whose Vec capacities are not a parity
    // claim).
    note(
        sim_report.stamped_cells == fix_report.stamped_cells
            && sim_report.volumes == fix_report.volumes
            && sim_report.skipped == fix_report.skipped,
        format!(
            "{scene}: stamp reports differ: sim (cells {}, volumes {}, skipped {}) vs fixture (cells {}, volumes {}, skipped {})",
            sim_report.stamped_cells,
            sim_report.volumes,
            sim_report.skipped,
            fix_report.stamped_cells,
            fix_report.volumes,
            fix_report.skipped
        ),
        &mut total,
        &mut all_diffs,
    );

    // Non-vacuity: the scene's furniture has a real model,
    // and both sides really stamped it.
    for (label, report) in [("sim", &sim_report), ("fixture", &fix_report)] {
        note(
            report.volumes >= 1 && report.stamped_cells > 0,
            format!(
                "{scene}: {label} side stamped nothing (volumes {}, cells {}) — the gate would be vacuous",
                report.volumes, report.stamped_cells
            ),
            &mut total,
            &mut all_diffs,
        );
    }

    // The volumes themselves, cell for cell.
    let (n, d, summary) = compare_layers(&sim_layer, &fix_layer);
    total += n;
    all_diffs.extend(d);

    assert_eq!(
        total,
        0,
        "{scene}: {} fine-prop diffs:\n{}",
        total,
        all_diffs.join("\n")
    );
    println!(
        "{scene}: sim/fixture furniture fine-prop parity — 0 diffs; placements {:?}",
        sp.iter()
            .map(|(x, y, l, s)| format!("({x},{y},lvl{l},{s})"))
            .collect::<Vec<_>>()
    );
    for line in &summary {
        println!("{scene}: {line}");
    }
}

#[test]
fn sim_stub_house_fineprop_parity() {
    assert_scene_fineprop_parity("stub_house", sample_tiledefs(), sample_snapshot());
}

#[test]
fn sim_roofed_house_explicit_fineprop_parity() {
    let defs = merge_defs(&[sample_roofed_tiledefs(), sample_tiledefs()]);
    assert_scene_fineprop_parity(
        "roofed_house_explicit",
        defs,
        sample_roofed_house_snapshot(),
    );
}

#[test]
fn sim_roofed_house_implicit_fineprop_parity() {
    let defs = merge_defs(&[sample_roofed_tiledefs(), sample_tiledefs()]);
    assert_scene_fineprop_parity(
        "roofed_house_implicit",
        defs,
        sample_implicit_roof_snapshot(),
    );
}
