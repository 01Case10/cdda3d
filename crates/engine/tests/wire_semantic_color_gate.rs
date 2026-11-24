//! Phase 11 follow-up (Lane J) gate: the live wire world is
//! coloured by CDDA's OWN per-def colours, not by Lane G's
//! hash palette.
//!
//! Lane G shipped `wire::palette_color` — an FNV-1a hash of
//! the string id into an HSL band — as an honest stand-in:
//! distinct per def, but arbitrary (t_wall hashed to a pink,
//! t_pavement to a teal). Lane J imports the game's `color`
//! field end to end (importer -> defs.json -> additive
//! TileDef.color wire field -> sim registry -> production
//! resolver), and the resolver now maps the colour NAME to
//! RGB through the game's own colour table:
//! data/raw/colors.json's colourdef values (BLACK/RED/GREEN/
//! BROWN/... as RGB triplets) plus src/color.cpp's
//! name->colour-pair table for which base colour a name's
//! foreground is. The hash palette survives ONLY as the
//! fallback for defs that carry no colour.
//!
//! This gate pins, over the sim's REAL defs and a REAL
//! street_block snapshot (the wire_source_gate construction):
//!
//! - the pinned defs' wire `color` names are the game's own
//!   (t_grass "green", t_wall "light_gray", t_pavement
//!   "dark_gray", t_door_c "brown");
//! - the blocks the production resolver stamps for those
//!   defs carry EXACTLY the game's RGB for those names
//!   (values hardcoded here from data/raw/colors.json, so
//!   the gate pins the data, not the implementation's
//!   self-consistency);
//! - those colours are NOT the hash colours (the set
//!   demonstrably failed under the Lane G palette — the
//!   failing-first evidence);
//! - coverage is stated, not silent: every terrain def in
//!   the pinned 0.I-1 table carries a colour (1108/1108),
//!   and terrain+furniture fall back for exactly the 45
//!   furniture defs the data leaves colourless;
//! - the fallback still works: a def with no colour (or a
//!   name the game doesn't define) resolves to the hash
//!   palette colour, exercised on a synthetic table;
//! - a raster capture of the wire scene actually shows the
//!   semantic colours (grass green in bulk).
//!
//! Failing-first: with the resolver still hash-only (the
//! state after Lane J parts 1-2 landed: defs carry colour
//! names, nothing reads them), every semantic assertion
//! below fails with the hash colours printed.

use std::collections::HashMap;
use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::snapshot_world::build_snapshot_world;
use engine::wire::{palette_color, palette_resolver, WireScene};
use mesher::chunk::ChunkMap;
use mesher::{Block, WorldScale};
use proto::{SnapshotT, TileDefT, TileDefsT, TileKind};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

fn street_block_wire() -> (TileDefsT, SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path("street_block")).expect("street_block loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    (reg.tiledefs().clone(), snap)
}

/// The game's RGB for the pinned names, from the pinned
/// tree's data/raw/colors.json colourdef (BLACK [0,0,0],
/// RED [255,0,0], GREEN [0,110,0], BROWN [97,56,28], GRAY
/// [150,150,150], DGRAY [99,99,99], ...), /255. The name ->
/// base-colour leg is src/color.cpp's table: c_green is
/// pair 3 (GREEN fg), c_light_gray pair 1 (GRAY fg),
/// c_dark_gray pair 72 (DGRAY fg), c_brown pair 7 (BROWN fg).
fn rgb(r: u8, g: u8, b: u8) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

/// (string id, expected wire colour name, expected RGB).
const PINNED: [(&str, &str, [f32; 4]); 4] = [
    ("t_grass", "green", [0.0, 110.0 / 255.0, 0.0, 1.0]),
    ("t_wall", "light_gray", [150.0 / 255.0, 150.0 / 255.0, 150.0 / 255.0, 1.0]),
    ("t_pavement", "dark_gray", [99.0 / 255.0, 99.0 / 255.0, 99.0 / 255.0, 1.0]),
    ("t_door_c", "brown", [97.0 / 255.0, 56.0 / 255.0, 28.0 / 255.0, 1.0]),
];

fn terrain_def<'a>(defs: &'a TileDefsT, sid: &str) -> &'a TileDefT {
    defs.defs
        .as_ref()
        .expect("defs present")
        .iter()
        .find(|d| d.kind == TileKind::Terrain && d.string_id.as_deref() == Some(sid))
        .unwrap_or_else(|| panic!("terrain def {sid} present"))
}

fn block_at(map: &ChunkMap, x: i32, y: i32, block_z: i32) -> Block {
    match map.chunks.get(&[x.div_euclid(12), y.div_euclid(12)]) {
        Some(chunk) => chunk.grid.get(x.rem_euclid(12), y.rem_euclid(12), block_z),
        None => Block::AIR,
    }
}

#[test]
fn street_block_wire_world_uses_cdda_semantic_colours() {
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

    // Wire names + resolver colours for the pinned set.
    for (sid, name, expected) in PINNED {
        let def = terrain_def(&defs, sid);
        assert_eq!(
            def.color.as_deref(),
            Some(name),
            "{sid}: the wire TileDef must carry the game's own colour name"
        );
        assert_eq!(
            resolver(def.id).color,
            expected,
            "{sid}: the production resolver must produce the game's RGB for {name:?}"
        );
        assert_ne!(
            resolver(def.id).color,
            palette_color(sid),
            "{sid}: the resolver colour must not be the Lane G hash colour"
        );
    }

    // The built world: every street_block tile of a pinned
    // def carries the semantic colour (floor slab for
    // ground tiles, the standing fill for walls/doors).
    let strings: HashMap<u32, String> = defs
        .defs
        .as_ref()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|d| d.kind == TileKind::Terrain)
        .filter_map(|d| d.string_id.clone().map(|s| (d.id, s)))
        .collect();
    let expected_by_sid: HashMap<&str, [f32; 4]> =
        PINNED.iter().map(|(sid, _, c)| (*sid, *c)).collect();
    let mut tiles_seen: HashMap<&str, usize> = HashMap::new();
    for block in snap.blocks.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
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
            let Some(sid) = strings.get(&ter) else { continue };
            let Some(expected) = expected_by_sid.get(sid.as_str()) else {
                continue;
            };
            let (x, y) = (ox + (i % 12) as i32, oy + (i / 12) as i32);
            let base = (oz - min_z) * per;
            let b = block_at(&map, x, y, base);
            assert_eq!(b.id, ter, "{sid} at ({x},{y},{oz}) must be its def's block");
            assert_eq!(
                b.color, *expected,
                "{sid} at ({x},{y},{oz}) must carry the game's colour, got {:?}",
                b.color
            );
            *tiles_seen.entry(sid.as_str()).or_default() += 1;
        }
    }
    for (sid, _, _) in PINNED {
        assert!(
            tiles_seen.get(sid).copied().unwrap_or(0) > 0,
            "street_block must contain {sid} tiles"
        );
    }
    println!("semantic-colour pinned tiles seen: {tiles_seen:?}");

    // Coverage, stated: the pinned 0.I-1 table gives EVERY
    // terrain def a colour; furniture falls back for exactly
    // the 45 defs the data leaves colourless (importer
    // counts, Lane J part 1). The resolver's fallback count
    // over terrain+furniture is therefore exactly 45.
    let (mut ter_total, mut ter_none) = (0usize, 0usize);
    let (mut furn_total, mut furn_none) = (0usize, 0usize);
    for d in defs.defs.as_ref().map(Vec::as_slice).unwrap_or(&[]) {
        let (total, none) = match d.kind {
            TileKind::Terrain => (&mut ter_total, &mut ter_none),
            TileKind::Furniture => (&mut furn_total, &mut furn_none),
            _ => continue,
        };
        *total += 1;
        if d.color.as_deref().map_or(true, |c| c.is_empty()) {
            *none += 1;
        }
    }
    println!(
        "semantic coverage: terrain {}/{} colourless, furniture {}/{} colourless",
        ter_total - ter_none,
        ter_total,
        furn_total - furn_none,
        furn_total
    );
    assert_eq!(ter_none, 0, "every terrain def carries a colour in 0.I-1");
    assert_eq!(furn_none, 45, "furniture colourless count is a pinned data fact");
}

#[test]
fn colourless_and_unknown_defs_fall_back_to_the_hash_palette() {
    // A synthetic table: the production resolver must fall
    // back to Lane G's hash palette — never error, never a
    // hole — for a def whose colour is absent AND for a def
    // whose colour name the game does not define.
    let mut mk = |id: u32, sid: &str, color: Option<&str>| {
        let mut d = TileDefT::default();
        d.id = id;
        d.string_id = Some(sid.to_string());
        d.kind = TileKind::Terrain;
        d.color = color.map(|s| s.to_string());
        d
    };
    let mut defs = TileDefsT::default();
    defs.defs = Some(vec![
        mk(1, "t_no_colour", None),
        mk(2, "t_bogus_colour", Some("not_a_game_colour")),
        mk(3, "t_real_colour", Some("green")),
    ]);
    let resolver = palette_resolver(&defs);
    assert_eq!(
        resolver(1).color,
        palette_color("t_no_colour"),
        "absent colour falls back to the hash palette"
    );
    assert_eq!(
        resolver(2).color,
        palette_color("t_bogus_colour"),
        "an unknown colour name falls back to the hash palette"
    );
    assert_eq!(
        resolver(3).color,
        rgb(0, 110, 0),
        "a known colour name resolves semantically"
    );
    assert_eq!(resolver(0), Block::AIR, "id 0 stays AIR");
}

#[test]
fn street_block_wire_capture_shows_semantic_grass() {
    let (defs, snap) = street_block_wire();
    let scene = WireScene::from_wire(&defs, &snap, &palette_resolver(&defs));
    let out = std::env::temp_dir().join(format!(
        "cdda-wire-semantic-gate-{}.png",
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
    run_capture(&args).expect("wire capture renders");
    let img = image::open(&out).expect("capture png").to_rgba8();

    // Classify pixels by max-normalized linear colour
    // (shading-invariant, the wire_standing_gate method)
    // against the SEMANTIC grass centroid: GREEN [0,110,0]
    // normalizes to [0, 1, 0]. Under the hash palette,
    // t_grass rendered as a pink [0.578, 0.17, 0.347] whose
    // normalized form is far from [0, 1, 0] — this count is
    // the capture-level failing-first witness.
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
    let mut grass_px = 0usize;
    for p in img.pixels() {
        let lin = [
            srgb_to_linear(p.0[0] as f32 / 255.0),
            srgb_to_linear(p.0[1] as f32 / 255.0),
            srgb_to_linear(p.0[2] as f32 / 255.0),
        ];
        if lin.iter().copied().fold(0.0f32, f32::max) < 0.02 {
            continue;
        }
        let n = norm_linear(lin);
        let dist = ((n[0]).powi(2) + (n[1] - 1.0).powi(2) + (n[2]).powi(2)).sqrt();
        if dist <= 0.09 {
            grass_px += 1;
        }
    }
    println!("semantic capture: grass-green pixels {grass_px}");
    assert!(
        grass_px >= 1000,
        "the capture must show the game's green for grass in bulk, got {grass_px} px"
    );
}
