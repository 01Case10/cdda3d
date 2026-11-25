//! Phase 11 S1 gate — sim scene vs fixture, through the engine's
//! real wire-ingest path.
//!
//! SCOPE (honest): this is **world + mesh parity**, not pixel-
//! capture parity. Every public capture entry
//! (`frame::render_frame`, `rt::render_rt_frame`) takes
//! `CaptureArgs` whose snapshot is a fixture *name* — there is no
//! public entry that renders an arbitrary wire `SnapshotT`, so
//! capture parity would need an engine source change and remains
//! open. What this gate proves instead: the sim's wire data
//! (full S1a TileDefs + a scene Snapshot, built in-process by
//! `crates/sim`) fed through `snapshot_world::build_snapshot_world`
//! — the same wire -> columns -> blocks path the live loop uses —
//! produces exactly the fixture's world, in both world forms:
//! the raster form (ChunkMap equality cell-by-cell, and
//! `mesh_world` quad counts + a hash of every mesh array) and
//! the RT form (`treebuild::VoxelWorld::from_chunk_map` cell
//! materials probed over every occupied cell). Identical
//! ChunkMaps through identical deterministic meshers are the two
//! renderers' inputs, so world+mesh equality is the strongest
//! parity statement available without an engine change.
//!
//! Ids are NOT compared raw: the fixture tables use globally
//! unique hand-assigned ids, the sim's registry uses per-kind
//! ids (the real adapter's convention). Both sides are mapped
//! through a canonical palette — terrain string id -> canonical
//! id, built from the union of both tables' terrain defs — and
//! the `block_for` colour is a pure function of the canonical
//! id, so any remaining difference is a real world difference,
//! never an id-numbering artefact.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use engine::cutaway::CutawayMode;
use engine::snapshot_world::build_snapshot_world;
use engine::world::{filtered_map, mesh_world};
use mesher::chunk::ChunkMap;
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

/// One side of the comparison: a TileDefs table, a Snapshot, and
/// that table's terrain id -> string id map.
struct Side {
    label: &'static str,
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

fn merge_defs(tables: &[TileDefsT]) -> TileDefsT {
    // The sample tables share one hand-assigned id space (the
    // sim parity gate documents the same merge): later tables
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

fn sim_side(scene_name: &str) -> Side {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("sim scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    let defs = reg.tiledefs().clone();
    let terrain_strings = terrain_strings(&defs);
    Side {
        label: "sim",
        defs,
        snap,
        terrain_strings,
    }
}

fn fixture_side(defs: TileDefsT, snap: SnapshotT) -> Side {
    let terrain_strings = terrain_strings(&defs);
    Side {
        label: "fixture",
        defs,
        snap,
        terrain_strings,
    }
}

/// Canonical palette: union of both sides' terrain string ids,
/// sorted, numbered from 1 (0 = air/none).
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
            // build_columns never calls block_for with 0
            // (emits_block excludes it); AIR is the safe value.
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

fn min_game_z(snap: &SnapshotT) -> i32 {
    snap.blocks
        .as_ref()
        .map(|bs| {
            bs.iter()
                .map(|b| b.origin.as_ref().map(|o| o.z).unwrap_or(0))
                .min()
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn player_z(snap: &SnapshotT) -> i32 {
    if let Some(entities) = &snap.entities {
        if let Some(p) = entities
            .iter()
            .find(|e| e.sprite_id.as_deref() == Some("player_male"))
        {
            if let Some(pos) = &p.pos {
                return pos.z;
            }
        }
    }
    snap.view_origin.as_ref().map(|o| o.z).unwrap_or(0)
}

/// Compare two ChunkMaps cell by cell. Returns (diff count,
/// first diffs rendered with global positions + terrain names).
fn compare_maps(
    sim: &ChunkMap,
    fixture: &ChunkMap,
    palette: &HashMap<String, u32>,
    ctx: &str,
) -> (usize, Vec<String>) {
    let mut rev: HashMap<u32, &str> = HashMap::new();
    for (s, &id) in palette {
        rev.insert(id, s.as_str());
    }
    let name = |b: &Block| -> String {
        if b.id == 0 {
            "<air>".to_string()
        } else {
            rev.get(&b.id).copied().unwrap_or("<?>").to_string()
        }
    };
    let mut diffs: Vec<String> = Vec::new();
    let mut total = 0usize;
    let sim_keys: BTreeSet<[i32; 2]> = sim.chunks.keys().copied().collect();
    let fix_keys: BTreeSet<[i32; 2]> = fixture.chunks.keys().copied().collect();
    if sim_keys != fix_keys {
        total += 1;
        diffs.push(format!(
            "{ctx}: chunk key sets differ: sim {sim_keys:?} vs fixture {fix_keys:?}"
        ));
    }
    for coord in sim_keys.intersection(&fix_keys) {
        let (sc, fc) = (&sim.chunks[coord], &fixture.chunks[coord]);
        if sc.grid.size != fc.grid.size {
            total += 1;
            diffs.push(format!(
                "{ctx}: chunk {coord:?} grid size {:?} vs {:?}",
                sc.grid.size, fc.grid.size
            ));
            continue;
        }
        for z in 0..sc.grid.size[2] as i32 {
            for y in 0..sc.grid.size[1] as i32 {
                for x in 0..sc.grid.size[0] as i32 {
                    let (sb, fb) = (sc.grid.get(x, y, z), fc.grid.get(x, y, z));
                    if sb != fb {
                        total += 1;
                        if diffs.len() < 50 {
                            diffs.push(format!(
                                "{ctx}: cell ({}, {}, {}) sim {} (id {}) vs fixture {} (id {})",
                                coord[0] * 12 + x,
                                coord[1] * 12 + y,
                                z,
                                name(&sb),
                                sb.id,
                                name(&fb),
                                fb.id
                            ));
                        }
                    }
                }
            }
        }
    }
    (total, diffs)
}

/// Compare two RT worlds by probing every cell position either
/// ChunkMap occupies (VoxelWorld has get/len but no iterator).
fn compare_rt(sim_map: &ChunkMap, fixture_map: &ChunkMap, ctx: &str) -> (usize, Vec<String>) {
    let sim_rt = treebuild::VoxelWorld::from_chunk_map(sim_map);
    let fix_rt = treebuild::VoxelWorld::from_chunk_map(fixture_map);
    let mut diffs: Vec<String> = Vec::new();
    let mut total = 0usize;
    if sim_rt.len() != fix_rt.len() {
        total += 1;
        diffs.push(format!(
            "{ctx}: RT cell counts differ: sim {} vs fixture {}",
            sim_rt.len(),
            fix_rt.len()
        ));
    }
    let mut positions: BTreeSet<[i32; 3]> = BTreeSet::new();
    for map in [sim_map, fixture_map] {
        for (coord, chunk) in &map.chunks {
            for z in 0..chunk.grid.size[2] as i32 {
                for y in 0..chunk.grid.size[1] as i32 {
                    for x in 0..chunk.grid.size[0] as i32 {
                        if chunk.grid.get(x, y, z).id != 0 {
                            positions.insert([coord[0] * 12 + x, coord[1] * 12 + y, z]);
                        }
                    }
                }
            }
        }
    }
    for pos in positions {
        // Full-cell equality (VoxelCell: material_id, color,
        // alpha, lit, flags, sky_exposed — all derived from the
        // ChunkMap block by the same conversion on both sides).
        let s = sim_rt.get(pos);
        let f = fix_rt.get(pos);
        if s != f {
            total += 1;
            if diffs.len() < 50 {
                diffs.push(format!("{ctx}: RT cell {pos:?} sim {s:?} vs fixture {f:?}"));
            }
        }
    }
    (total, diffs)
}

/// FNV-1a hash over every array of a mesh (positions/normals/
/// colors/uvs as f32 bits, layers + indices verbatim).
fn mesh_hash(meshes: &mesher::PassMeshes) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for m in &meshes.meshes {
        for v in &m.positions {
            for f in v {
                mix(&f.to_bits().to_le_bytes());
            }
        }
        for v in &m.normals {
            for f in v {
                mix(&f.to_bits().to_le_bytes());
            }
        }
        for v in &m.colors {
            for f in v {
                mix(&f.to_bits().to_le_bytes());
            }
        }
        for v in &m.uvs {
            for f in v {
                mix(&f.to_bits().to_le_bytes());
            }
        }
        for v in &m.layers {
            mix(&v.to_le_bytes());
        }
        for v in &m.indices {
            mix(&v.to_le_bytes());
        }
    }
    h
}

/// Mesh both sides at one cutaway; compare per-pass quad counts,
/// totals, and the full-array hash. Returns rendered diffs.
fn compare_meshes(
    sim_map: &ChunkMap,
    fixture_map: &ChunkMap,
    sim_min_z: i32,
    fixture_min_z: i32,
    visible: &[i32],
    ctx: &str,
) -> Vec<String> {
    let (sm, _) = mesh_world(sim_map, sim_min_z, visible);
    let (fm, _) = mesh_world(fixture_map, fixture_min_z, visible);
    let mut diffs = Vec::new();
    for (i, (s, f)) in sm.meshes.iter().zip(fm.meshes.iter()).enumerate() {
        if s.quad_count() != f.quad_count() {
            diffs.push(format!(
                "{ctx}: pass {i} quad count sim {} vs fixture {}",
                s.quad_count(),
                f.quad_count()
            ));
        }
    }
    if sm.total_quads() != fm.total_quads() {
        diffs.push(format!(
            "{ctx}: total quads sim {} vs fixture {}",
            sm.total_quads(),
            fm.total_quads()
        ));
    }
    let (sh, fh) = (mesh_hash(&sm), mesh_hash(&fm));
    if sh != fh {
        diffs.push(format!(
            "{ctx}: mesh array hash sim {sh:#x} vs fixture {fh:#x}"
        ));
    }
    println!(
        "{ctx}: quads sim {} (passes {:?}) fixture {} (passes {:?})",
        sm.total_quads(),
        sm.meshes.iter().map(|m| m.quad_count()).collect::<Vec<_>>(),
        fm.total_quads(),
        fm.meshes.iter().map(|m| m.quad_count()).collect::<Vec<_>>()
    );
    diffs
}

fn assert_scene_parity(scene: &str, fixture_defs: TileDefsT, fixture_snap: SnapshotT) {
    let sim = sim_side(scene);
    let fixture = fixture_side(fixture_defs, fixture_snap);
    let palette = canonical_palette(&sim, &fixture);
    let scale = WorldScale::default();

    let sim_map =
        build_snapshot_world(&sim.defs, &sim.snap, scale, &block_for_side(&sim, &palette));
    let fix_map = build_snapshot_world(
        &fixture.defs,
        &fixture.snap,
        scale,
        &block_for_side(&fixture, &palette),
    );

    let mut all_diffs: Vec<String> = Vec::new();
    let mut total = 0usize;

    // Full maps: ChunkMap cells + RT form.
    let (n, d) = compare_maps(&sim_map, &fix_map, &palette, "full");
    total += n;
    all_diffs.extend(d);
    let (n, d) = compare_rt(&sim_map, &fix_map, "full");
    total += n;
    all_diffs.extend(d);

    // Cutaway at the player z (HideAbove(0), the roof gate's
    // inside view): filtered maps' cells + RT form.
    let sim_pz = player_z(&sim.snap);
    let fix_pz = player_z(&fixture.snap);
    assert_eq!(sim_pz, fix_pz, "{scene}: player z differs between sides");
    let sim_min_z = min_game_z(&sim.snap);
    let fix_min_z = min_game_z(&fixture.snap);
    let inside = CutawayMode::HideAbove(0).visible_levels(sim_pz);
    let sim_cut = filtered_map(&sim_map, sim_min_z, &inside);
    let fix_cut = filtered_map(&fix_map, fix_min_z, &inside);
    let (n, d) = compare_maps(&sim_cut, &fix_cut, &palette, "cutaway");
    total += n;
    all_diffs.extend(d);
    let (n, d) = compare_rt(&sim_cut, &fix_cut, "cutaway");
    total += n;
    all_diffs.extend(d);

    // Meshes (raster form) at both of the roof gate's cutaways.
    for (label, levels) in [
        (
            "mesh-inside",
            CutawayMode::HideAbove(0).visible_levels(sim_pz),
        ),
        (
            "mesh-outside",
            CutawayMode::HideAbove(1).visible_levels(sim_pz),
        ),
    ] {
        let d = compare_meshes(&sim_map, &fix_map, sim_min_z, fix_min_z, &levels, label);
        total += d.len();
        all_diffs.extend(d);
    }

    assert_eq!(
        total,
        0,
        "{scene}: {} world/mesh diffs (first {}):\n{}",
        total,
        all_diffs.len(),
        all_diffs.join("\n")
    );
    println!("{scene}: sim/fixture world + mesh parity — 0 diffs");
}

#[test]
fn sim_stub_house_world_parity() {
    assert_scene_parity("stub_house", sample_tiledefs(), sample_snapshot());
}

#[test]
fn sim_roofed_house_explicit_world_parity() {
    let defs = merge_defs(&[sample_roofed_tiledefs(), sample_tiledefs()]);
    assert_scene_parity(
        "roofed_house_explicit",
        defs,
        sample_roofed_house_snapshot(),
    );
}

#[test]
fn sim_roofed_house_implicit_world_parity() {
    let defs = merge_defs(&[sample_roofed_tiledefs(), sample_tiledefs()]);
    assert_scene_parity(
        "roofed_house_implicit",
        defs,
        sample_implicit_roof_snapshot(),
    );
}
