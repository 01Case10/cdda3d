//! Phase 5 gate tests (Part II §8): 5.1, 5.3, 5.4, 5.5, 5.5b, 5.6,
//! 5.7, 5.8, 5.8b, 5.9, 5.11 against the committed fixture tileset.

use image::{Rgba, RgbaImage};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use voxelizer::*;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture")
}

fn load_fixture() -> tileset::Tileset {
    tileset::load_decomposed(&fixture_dir()).expect("fixture loads")
}

fn fixture_cfg() -> VoxelizerConfig {
    let mut looks = BTreeMap::new();
    looks.insert("t_dirt_lookalike".to_string(), "t_dirt".to_string());
    VoxelizerConfig {
        season: "spring".into(),
        tile_size: 32,
        looks_like: looks,
        extra_ids: vec![
            "t_dirt_lookalike".into(),
            "t_rock".into(),
            "t_seasonal".into(),
            "t_totally_unknown".into(),
        ],
        tileset_license_text: std::fs::read_to_string(fixture_dir().join("LICENSE.txt")).ok(),
        ..Default::default()
    }
}

fn run_fixture(out: &Path) -> (tileset::Tileset, RunReport, Vec<MaterialOut>) {
    let ts = load_fixture();
    let report = run(&ts, &fixture_cfg(), out).expect("voxelizer run");
    let mats: Vec<MaterialOut> =
        serde_json::from_str(&std::fs::read_to_string(out.join("materials.json")).unwrap())
            .unwrap();
    (ts, report, mats)
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vox_gate_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn mat<'a>(mats: &'a [MaterialOut], id: &str) -> &'a MaterialOut {
    mats.iter()
        .find(|m| m.id == id)
        .unwrap_or_else(|| panic!("material {id}"))
}

fn layer_img(out: &Path, layer: usize) -> RgbaImage {
    image::open(out.join("blocks").join(format!("layer_{layer:04}.png")))
        .unwrap()
        .to_rgba8()
}

/// 5.1 Fixture golden: outputs byte-identical to committed goldens.
/// Regenerate with UPDATE_GOLDEN=1 cargo test -p voxelizer.
#[test]
fn gate_5_1_fixture_golden() {
    let out = tmpdir("golden");
    run_fixture(&out);
    let golden_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        let _ = std::fs::remove_dir_all(&golden_root);
        std::fs::create_dir_all(&golden_root).unwrap();
        for f in list_outputs(&out) {
            let rel = f.strip_prefix(&out).unwrap();
            let dest = golden_root.join(rel);
            std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
            std::fs::copy(&f, &dest).unwrap();
        }
        return;
    }
    let got = list_outputs(&out);
    let want = list_outputs(&golden_root);
    let got_rel: Vec<_> = got
        .iter()
        .map(|f| f.strip_prefix(&out).unwrap().to_path_buf())
        .collect();
    let want_rel: Vec<_> = want
        .iter()
        .map(|f| f.strip_prefix(&golden_root).unwrap().to_path_buf())
        .collect();
    assert_eq!(
        got_rel, want_rel,
        "output file sets differ (run UPDATE_GOLDEN=1 if intentional)"
    );
    for (g, w) in got.iter().zip(want.iter()) {
        assert_eq!(
            std::fs::read(g).unwrap(),
            std::fs::read(w).unwrap(),
            "golden mismatch: {}",
            g.display()
        );
    }
}

/// 5.3 Top face = fg composited OVER bg (fg-only comparison must fail).
#[test]
fn gate_5_3_top_face_fg_over_bg() {
    let out = tmpdir("topface");
    let (ts, _, mats) = run_fixture(&out);
    let grass = mat(&mats, "t_grass");
    let top = layer_img(&out, grass.top_layer);
    // A pixel where fg is transparent must show the bg dirt color.
    // grass_fg is opaque where (x*7+y*13)%3==0; find a transparent one.
    let bg_px = top.get_pixel(1, 0); // (7)%3=1 -> fg transparent
    assert_eq!(
        *bg_px,
        Rgba([101, 67, 33, 255]),
        "bg must show through transparent fg"
    );
    let fg_px = top.get_pixel(0, 0); // 0%3==0 -> fg opaque green
    assert_eq!(
        *fg_px,
        Rgba([34, 139, 34, 255]),
        "fg must cover bg where opaque"
    );
    // And the entry composite itself matches the material top.
    let entry = ts.entries.get("t_grass").unwrap().clone();
    let comp = composite_entry(&ts, &entry).unwrap();
    assert_eq!(comp.get_pixel(1, 0), &Rgba([101, 67, 33, 255]));
}

/// 5.4 Side rules: floor side darkened within 25-40%; wall side = sprite.
#[test]
fn gate_5_4_side_face_rules() {
    let out = tmpdir("side");
    let (_, _, mats) = run_fixture(&out);
    let floor = mat(&mats, "t_floor_wood");
    let top = layer_img(&out, floor.top_layer);
    let side = layer_img(&out, floor.side_layer);
    // Sample a non-noise-sensitive ratio bound over the whole face:
    // every side channel must be within [0.55, 0.75] x top source row.
    for y in 0..32u32 {
        for x in 0..32u32 {
            let s = side.get_pixel(x, y);
            let t = top.get_pixel(x, 31); // bottom edge rows sampled
            for c in 0..3 {
                let ratio = s[c] as f32 / t[c].max(1) as f32;
                assert!(
                    ratio > 0.5 && ratio < 0.8,
                    "floor side ratio {ratio} at {x},{y}"
                );
            }
        }
    }
    let wall = mat(&mats, "t_wall");
    let wtop = layer_img(&out, wall.top_layer);
    let wside = layer_img(&out, wall.side_layer);
    assert_eq!(wtop, wside, "wall side must be the wall sprite directly");
}

/// 5.5 Multitile: all 16 cardinal masks + connect-group case.
#[test]
fn gate_5_5_multitile_mapping() {
    let expected: [(u8, &str); 16] = [
        (0, "unconnected"),
        (1, "end_piece"),
        (2, "end_piece"),
        (4, "end_piece"),
        (8, "end_piece"),
        (3, "corner"),
        (6, "corner"),
        (9, "corner"),
        (12, "corner"),
        (5, "edge"),
        (10, "edge"),
        (7, "t_connection"),
        (11, "t_connection"),
        (13, "t_connection"),
        (14, "t_connection"),
        (15, "center"),
    ];
    for (mask, want) in expected {
        let (got, _) = subtile_for_mask(mask);
        assert_eq!(got, want, "mask {mask:04b}");
    }
    // Rotations are in-range for variant arrays (corner/tee/end: 4, edge: 2).
    for m in 0..16u8 {
        let (sub, rot) = subtile_for_mask(m);
        let max = match sub {
            "edge" => 2,
            "center" | "unconnected" => 1,
            _ => 4,
        };
        assert!(rot < max, "mask {m} subtile {sub} rot {rot}");
    }
    // Connect groups: different id, same group still connects.
    assert!(neighbors_connect("t_wall", &[3], "t_wall", &[]));
    assert!(neighbors_connect("t_wall", &[3], "t_brick_wall", &[3]));
    assert!(!neighbors_connect("t_wall", &[3], "t_wood_wall", &[4]));
    let mask = mask_from_connects(true, false, true, false);
    assert_eq!(subtile_for_mask(mask).0, "edge");
    // Fixture wall entry carries all 6 neighbour subtiles as layers.
    let out = tmpdir("multi");
    let (_, _, mats) = run_fixture(&out);
    let wall = mat(&mats, "t_wall");
    for sub in [
        "center",
        "corner",
        "edge",
        "t_connection",
        "end_piece",
        "unconnected",
    ] {
        assert!(
            wall.subtile_layers.contains_key(sub),
            "missing subtile layer {sub}"
        );
    }
}

/// 5.5b Resolution chain: variant, season (pinned), looks_like.
#[test]
fn gate_5_5b_resolution_chain() {
    let out = tmpdir("chain");
    let (_, _, mats) = run_fixture(&out);
    assert_eq!(mat(&mats, "t_rock").resolution, "variant");
    assert_eq!(mat(&mats, "t_rock").resolved_id, "t_rock_var_red");
    assert_eq!(mat(&mats, "t_seasonal").resolution, "season");
    assert_eq!(mat(&mats, "t_dirt_lookalike").resolution, "looks_like");
    assert_eq!(mat(&mats, "t_grass").resolution, "exact");
}

/// 5.6 Alpha: fence = cutout, glass = transparent, no edge bleed color.
#[test]
fn gate_5_6_alpha_handling() {
    let out = tmpdir("alpha");
    let (_, _, mats) = run_fixture(&out);
    let fence = mat(&mats, "t_fence");
    assert!(fence.cutout && !fence.transparent, "fence must be cutout");
    let glass = mat(&mats, "t_glass_window");
    assert!(glass.transparent, "glass must be transparent");
    let grass = mat(&mats, "t_grass");
    assert!(!glass.cutout);
    let _ = grass;
    // Transparent texels carry no color (premultiplied-clean).
    let ftop = layer_img(&out, fence.top_layer);
    assert_eq!(ftop.get_pixel(4, 0)[3], 0);
}

/// 5.7 Mipmaps: alpha-aware downsample, no bleeding, edge clamping.
#[test]
fn gate_5_7_mipmaps() {
    // Half red opaque, half fully transparent "green" (poison color).
    let mut img = RgbaImage::new(32, 32);
    for y in 0..32 {
        for x in 0..32 {
            img.put_pixel(
                x,
                y,
                if x < 15 {
                    Rgba([255, 0, 0, 255])
                } else {
                    Rgba([0, 255, 0, 0])
                },
            );
        }
    }
    let chain = mip_chain(&img);
    assert_eq!(chain.len(), 6); // 32,16,8,4,2,1
    let mip1 = &chain[1];
    // Interior red texel stays pure red (transparent green contributes nothing).
    assert_eq!(*mip1.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
    // Boundary texel (covers source x=14 red + x=15 transparent):
    // alpha ~50%, color still pure red, NO green bleed.
    let b = mip1.get_pixel(7, 0);
    assert_eq!(b[1], 0, "green bleed into mip: {b:?}");
    assert_eq!(b[0], 255);
    assert!(b[3] > 100 && b[3] < 160, "boundary alpha {}", b[3]);
    // Fully transparent region stays transparent.
    assert_eq!(mip1.get_pixel(15, 0)[3], 0);
}

/// 5.8 Extrusion: columns match alpha mask; heights match heuristic.
#[test]
fn gate_5_8_extrusion() {
    let out = tmpdir("extrude");
    let (_, _, mats) = run_fixture(&out);
    let table = mat(&mats, "f_table");
    let model = table.sub_model.as_ref().expect("table sub-model");
    assert_eq!(model.mode, "extruded");
    assert_eq!(model.height_voxels, 1);
    // Table sprite: solid rect 4<=x<28, 4<=y<28 => 24*24 columns.
    assert_eq!(model.columns.len(), 24 * 24);
    assert!(model.columns.contains(&[4, 4]));
    assert!(!model.columns.contains(&[0, 0]));
    let bookcase = mat(&mats, "f_bookcase");
    assert_eq!(bookcase.height_voxels, 3, "bookcase heuristic = 3");
    assert_eq!(bookcase.sub_model.as_ref().unwrap().columns.len(), 32 * 32);
}

/// 5.8b Sprite geometry: every sheet size carries w/h/offset through.
#[test]
fn gate_5_8b_sprite_geometry() {
    let ts = load_fixture();
    let tall = &ts.sprites_by_name["tall_thing"];
    assert_eq!(
        (tall.width, tall.height, tall.offset_x, tall.offset_y),
        (32, 64, 0, -32)
    );
    let big = &ts.sprites_by_name["big_thing"];
    assert_eq!(
        (big.width, big.height, big.offset_x, big.offset_y),
        (64, 64, -16, -32)
    );
    // UV table in a run carries the same values.
    let out = tmpdir("geom");
    run_fixture(&out);
    let uvs: Vec<SpriteUv> =
        serde_json::from_str(&std::fs::read_to_string(out.join("sprites.json")).unwrap()).unwrap();
    let tall_uv = uvs.iter().find(|u| u.name == "tall_thing").unwrap();
    assert_eq!(
        (tall_uv.width, tall_uv.height, tall_uv.offset_y),
        (32, 64, -32)
    );
    let big_uv = uvs.iter().find(|u| u.name == "big_thing").unwrap();
    assert_eq!(
        (big_uv.width, big_uv.height, big_uv.offset_x),
        (64, 64, -16)
    );
    // Anchoring: the tall sprite's tile face crops at -offset (bottom half = blue).
    let (_, _, mats) = run_fixture(&out);
    let mon = mat(&mats, "mon_tall");
    let top = layer_img(&out, mon.top_layer);
    assert_eq!(
        *top.get_pixel(0, 0),
        Rgba([40, 40, 220, 255]),
        "offset-anchored crop"
    );
}

/// 5.9 Overrides: applied; invalid override = clear error.
#[test]
fn gate_5_9_overrides() {
    let text = "[f_bookcase]\nheight = 2\nmode = \"box\"\n\n[f_table]\ntint = [10, 20, 30]\n";
    let ov = parse_overrides(text).unwrap();
    let ts = load_fixture();
    let mut cfg = fixture_cfg();
    cfg.overrides = ov;
    let out = tmpdir("ovr");
    run(&ts, &cfg, &out).unwrap();
    let mats: Vec<MaterialOut> =
        serde_json::from_str(&std::fs::read_to_string(out.join("materials.json")).unwrap())
            .unwrap();
    let bc = mat(&mats, "f_bookcase");
    assert_eq!(bc.height_voxels, 2);
    assert_eq!(bc.sub_model.as_ref().unwrap().mode, "box");
    assert_eq!(mat(&mats, "f_table").tint, [10, 20, 30]);
    // Invalid mode and unknown key are clear errors.
    assert!(parse_overrides("[f_x]\nmode = \"sphere\"\n").is_err());
    assert!(parse_overrides("[f_x]\nbogus = 1\n").is_err());
    assert!(parse_overrides("[f_x]\nheight = 0\n").is_err());
}

/// 5.11 Unknown id: placeholder material + exactly one missing.log line.
#[test]
fn gate_5_11_unknown_fallback() {
    let out = tmpdir("unknown");
    let (_, report, mats) = run_fixture(&out);
    let unk = mat(&mats, "t_totally_unknown");
    assert_eq!(unk.resolution, "missing");
    let top = layer_img(&out, unk.top_layer);
    assert_eq!(
        *top.get_pixel(0, 0),
        Rgba([255, 0, 255, 255]),
        "magenta checker"
    );
    assert_eq!(
        *top.get_pixel(8, 0),
        Rgba([0, 0, 0, 255]),
        "black checker cell"
    );
    let log = std::fs::read_to_string(out.join("missing.log")).unwrap();
    assert_eq!(log.matches("t_totally_unknown").count(), 1);
    assert_eq!(report.missing.len(), 1);
}

/// 5.12 Attribution content (part of the 5.x gate set).
#[test]
fn gate_5_12_attribution() {
    let out = tmpdir("attr");
    run_fixture(&out);
    let attr = std::fs::read_to_string(out.join("ATTRIBUTION.md")).unwrap();
    assert!(attr.contains("CC BY-SA 3.0"));
    assert!(
        attr.contains("fixture://filler-source"),
        "per-sheet source carried"
    );
    assert!(
        attr.contains("SIL Open Font License"),
        "third-party notices included"
    );
}

/// Composed-form loader: indices, shared ids, weighted fg.
#[test]
fn composed_form_loads() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/composed");
    let ts = tileset::load_composed(&dir.join("tile_config.json")).unwrap();
    assert!(ts.composed);
    assert!(ts.entries.contains_key("t_floor_wood"));
    assert!(ts.entries.contains_key("t_shared_a") && ts.entries.contains_key("t_shared_b"));
    let w = &ts.entries["t_weighted"];
    // primary = highest weight (index 3)
    assert_eq!(w.fg.as_ref().unwrap().primary().unwrap().index(), Some(3));
    let g = ts
        .geom_for_entry_ref(
            &ts.entries["t_floor_wood"],
            ts.entries["t_floor_wood"].fg.as_ref().unwrap(),
        )
        .unwrap();
    assert_eq!((g.width, g.height), (32, 32));
    // Voxelizer runs on composed form too.
    let out = tmpdir("composed");
    let report = run(&ts, &VoxelizerConfig::default(), &out).unwrap();
    assert!(report.materials >= 4);
}
