//! Phase 11 follow-up (Lane G) gate: the wire world carries
//! STANDING geometry and per-def resolver colour.
//!
//! Before Lane G, `build_snapshot_world` stamped floor slabs
//! only and the live client painted every def the same grey:
//! Joey's first windowed run on his PC showed a single
//! uniform slab where the street_block house should be. This
//! gate pins the fix at the data level and through a real
//! raster capture of the sim's street_block world, built from
//! the sim's REAL defs and a REAL sim `World` snapshot (the
//! same construction `wire_source_gate` uses):
//!
//! - tiles whose terrain is standing (t_wall / t_window in
//!   the scene) fill their level's blocks above the floor
//!   slab with that def's block;
//! - the blocks' colours are the production resolver's
//!   per-def colours (`engine::wire::palette_resolver` —
//!   Lane G's string-id hash palette at this gate's birth,
//!   the defs' CDDA semantic colours since Lane J; this
//!   gate pins that the built world carries the resolver's
//!   colours, wire_semantic_color_gate pins what those
//!   colours ARE), and grass / pavement / floor carry three
//!   pairwise-distinct colours;
//! - the filled-block count and exposed-face count jump
//!   vs the floors-only baseline (both numbers recorded
//!   at the assertions);
//! - a raster capture of the wire scene is non-blank and
//!   actually contains the resolver colours of the scene's
//!   defs (the windowed app renders exactly this scene).
//!
//! Failing-first: on the pre-Lane-G builder the standing
//! assertions fail (every above-slab cell is AIR) and the
//! quad count is the floors-only baseline.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::snapshot_world::build_snapshot_world;
use engine::wire::{palette_resolver, WireScene};
use mesher::chunk::ChunkMap;
use mesher::{Block, WorldScale};
use proto::{SnapshotT, TileDefsT, TileKind};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

/// The sim's street_block world: real defs, real snapshot,
/// exactly as the live server would send them.
fn street_block_wire() -> (TileDefsT, SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path("street_block")).expect("street_block loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    (reg.tiledefs().clone(), snap)
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

/// Every (world x, world y, game z, terrain wire id, string
/// id) in the snapshot, in snapshot order.
fn tiles(defs: &TileDefsT, snap: &SnapshotT) -> Vec<(i32, i32, i32, u32, String)> {
    let strings = terrain_strings(defs);
    let mut out = Vec::new();
    for block in snap.blocks.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        // Origins are TILE coords of the submap's corner
        // (columns_from_snapshot's own convention).
        let (ox, oy, oz) = block
            .origin
            .as_ref()
            .map(|o| (o.x, o.y, o.z))
            .unwrap_or((0, 0, 0));
        for (i, &ter) in block
            .terrain
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            let sid = strings
                .get(&ter)
                .cloned()
                .unwrap_or_else(|| format!("#{ter}"));
            out.push((ox + (i % 12) as i32, oy + (i / 12) as i32, oz, ter, sid));
        }
    }
    out
}

fn block_at(map: &ChunkMap, x: i32, y: i32, block_z: i32) -> Block {
    match map.chunks.get(&[x.div_euclid(12), y.div_euclid(12)]) {
        Some(chunk) => chunk.grid.get(x.rem_euclid(12), y.rem_euclid(12), block_z),
        None => Block::AIR,
    }
}

#[test]
fn street_block_wire_world_has_standing_geometry_and_palette() {
    let (defs, snap) = street_block_wire();
    let scale = WorldScale::default();
    let per = scale.blocks_per_z_level as i32;
    let resolver = palette_resolver(&defs);
    let map = build_snapshot_world(&defs, &snap, scale, &resolver);
    let min_z = snap
        .blocks
        .as_ref()
        .and_then(|b| b.iter().filter_map(|b| b.origin.as_ref().map(|o| o.z)).min())
        .unwrap_or(0);
    let all = tiles(&defs, &snap);

    // -- Geometry volume vs the floors-only baseline
    // (pre-Lane-G builder, same scene/defs/scale, measured
    // on this gate's first runs, 2026-11-24): non-air blocks
    // 576 (one slab per tile) and 1,248 exposed faces. The
    // mesher's own quad stat does not track this change
    // (it reports 76 for both worlds — it counts merged
    // runs, not faces), so the gate measures the geometry
    // directly: filled blocks and naive exposed faces.
    // Printed BEFORE the standing assertions so a failing
    // run still reports the numbers.
    const BLOCKS_BASELINE: usize = 576;
    const FACES_BASELINE: usize = 1248;
    let (mut blocks, mut faces) = (0usize, 0usize);
    {
        let solid = |x: i32, y: i32, z: i32| -> bool {
            z >= 0
                && map
                    .chunks
                    .get(&[x.div_euclid(12), y.div_euclid(12)])
                    .map(|c| c.grid.get(x.rem_euclid(12), y.rem_euclid(12), z).id != 0)
                    .unwrap_or(false)
        };
        let (mut x0, mut x1, mut y0, mut y1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for (x, y, ..) in &all {
            x0 = x0.min(*x);
            x1 = x1.max(*x);
            y0 = y0.min(*y);
            y1 = y1.max(*y);
        }
        let top = (levels_span(&snap, min_z)) * per;
        for y in y0..=y1 {
            for x in x0..=x1 {
                for z in 0..top {
                    if solid(x, y, z) {
                        blocks += 1;
                        for (dx, dy, dz) in
                            [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]
                        {
                            if !solid(x + dx, y + dy, z + dz) {
                                faces += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "wire-standing: filled blocks {blocks} (baseline {BLOCKS_BASELINE}), exposed faces {faces} (baseline {FACES_BASELINE})"
    );

    // -- Standing geometry: every t_wall / t_window tile
    // fills the blocks above its floor slab with its own
    // def's block (id AND resolver colour).
    let mut standing = 0usize;
    for (x, y, z, ter, sid) in &all {
        if sid != "t_wall" && sid != "t_window" {
            continue;
        }
        standing += 1;
        let base = (z - min_z) * per;
        for dz in 1..per {
            let b = block_at(&map, *x, *y, base + dz);
            assert_eq!(
                b.id, *ter,
                "standing tile {sid} at ({x},{y},{z}): block z {} must be the def's block, got id {}",
                base + dz,
                b.id
            );
            assert_eq!(
                b.color,
                resolver(*ter).color,
                "standing tile {sid} at ({x},{y},{z}): block z {} must carry the resolver colour",
                base + dz
            );
        }
    }
    assert!(
        standing >= 20,
        "street_block must exercise standing tiles (found {standing})"
    );

    // -- Resolver colours: grass / pavement / floor appear,
    // carry their production-resolver colours in the built
    // world, and the three colours are pairwise distinct.
    let mut seen: BTreeMap<&str, [f32; 4]> = BTreeMap::new();
    for (x, y, z, ter, sid) in &all {
        if !matches!(sid.as_str(), "t_grass" | "t_pavement" | "t_floor") {
            continue;
        }
        let base = (z - min_z) * per;
        let b = block_at(&map, *x, *y, base);
        assert_eq!(
            b.color,
            resolver(*ter).color,
            "tile {sid} at ({x},{y},{z}) must carry its resolver colour"
        );
        seen.insert(sid.as_str(), b.color);
    }
    assert_eq!(
        seen.len(),
        3,
        "street_block must contain grass, pavement and floor tiles: {seen:?}"
    );
    let colors: Vec<[f32; 4]> = seen.values().copied().collect();
    for i in 0..colors.len() {
        for j in i + 1..colors.len() {
            assert_ne!(
                colors[i], colors[j],
                "resolver colours must be distinct per def: {seen:?}"
            );
        }
    }

    // -- The geometry-volume assertions (numbers at the
    // measurement above): every solid tile now fills its
    // level, so the block count triples at the default
    // scale, and the taller perimeter adds exposed faces.
    assert!(
        blocks >= BLOCKS_BASELINE * 2,
        "standing fill must at least double the filled blocks: got {blocks}, baseline {BLOCKS_BASELINE}"
    );
    assert!(
        faces > FACES_BASELINE,
        "standing fill must add exposed faces: got {faces}, baseline {FACES_BASELINE}"
    );
}

/// Number of z-levels in the snapshot (block bands in the
/// built map = levels * blocks_per_z_level).
fn levels_span(snap: &SnapshotT, min_z: i32) -> i32 {
    let max_z = snap
        .blocks
        .as_ref()
        .and_then(|b| b.iter().filter_map(|b| b.origin.as_ref().map(|o| o.z)).max())
        .unwrap_or(min_z);
    max_z - min_z + 1
}

#[test]
fn street_block_wire_capture_shows_resolver_colours() {
    let (defs, snap) = street_block_wire();
    let scene = WireScene::from_wire(&defs, &snap, &palette_resolver(&defs));
    let out = std::env::temp_dir().join(format!(
        "cdda-wire-standing-gate-{}.png",
        std::process::id()
    ));
    let mut args = CaptureArgs::default();
    args.snapshot = "wire".to_string();
    args.wire = Some(scene);
    args.renderer = RendererKind::Raster;
    args.camera_target = (12.0, 12.0);
    args.cutaway_z = 0;
    args.distance = 34.0;
    args.size = (640, 360);
    args.out = out.clone();
    let outcome = run_capture(&args).expect("wire capture renders");
    assert!(
        outcome.distinct_colors > 10,
        "the wire capture must not be a flat slab ({} distinct colours)",
        outcome.distinct_colors
    );
    let img = image::open(&out).expect("capture png").to_rgba8();
    // The resolver colours of the scene's defs must actually
    // appear. The renderer treats block colours as LINEAR
    // and sRGB-encodes on output, with a per-face scalar
    // shade — so a def's rendered pixels are scalar
    // multiples of its resolver colour in linear space.
    // Classify each pixel by its max-normalized linear
    // colour (shading-invariant) against the defs'
    // normalized resolver colours.
    //
    // Lane J amendment: the classes are grass / floor / GREY.
    // In the game's own data t_wall is "light_gray" and
    // t_pavement is "dark_gray" — both achromatic, so the
    // shading-invariant normalized classifier cannot tell
    // them apart BY CONSTRUCTION (they differ only in value,
    // which normalization discards). They form one grey class
    // here; their distinct exact RGBs are pinned at the data
    // level by wire_semantic_color_gate. (Under Lane G's hash
    // palette the two hashed to unrelated hues and classified
    // separately — a property of the hash, not of the game.)
    fn norm_linear(rgb: [f32; 3]) -> [f32; 3] {
        let m = rgb.iter().copied().fold(0.0f32, f32::max).max(1e-6);
        [rgb[0] / m, rgb[1] / m, rgb[2] / m]
    }
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    let resolver = palette_resolver(&defs);
    let terrain_id = |sid: &str| {
        defs.defs
            .as_ref()
            .expect("defs present")
            .iter()
            .find(|d| d.kind == TileKind::Terrain && d.string_id.as_deref() == Some(sid))
            .unwrap_or_else(|| panic!("terrain def {sid} present"))
            .id
    };
    // Class representatives: grass, floor, and t_wall for
    // the shared grey class (t_pavement lands on it too).
    let classes = ["t_grass", "t_floor", "t_wall"];
    let centroids: Vec<[f32; 3]> = classes
        .iter()
        .map(|sid| {
            let c = resolver(terrain_id(sid)).color;
            norm_linear([c[0], c[1], c[2]])
        })
        .collect();
    let mut counts = [0usize; 3];
    for p in img.pixels() {
        let lin = [
            srgb_to_linear(p.0[0] as f32 / 255.0),
            srgb_to_linear(p.0[1] as f32 / 255.0),
            srgb_to_linear(p.0[2] as f32 / 255.0),
        ];
        if lin.iter().copied().fold(0.0f32, f32::max) < 0.02 {
            continue; // near-black: shade crushed the hue away
        }
        let n = norm_linear(lin);
        let (best, dist) = centroids
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let d = (0..3).map(|k| (n[k] - c[k]).powi(2)).sum::<f32>().sqrt();
                (i, d)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        if dist <= 0.09 {
            counts[best] += 1;
        }
    }
    println!("wire-standing capture colour counts: {:?}", counts);
    let present = counts.iter().filter(|&&n| n >= 200).count();
    assert!(
        present >= 3,
        "all three def colour classes must appear in the capture: {counts:?} for {classes:?}"
    );
}
