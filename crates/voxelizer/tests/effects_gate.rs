//! Phase 2b gate tests (Part II §5b): 2b.1 effect-parameter extraction
//! against the committed fixture tileset, plus the render-class
//! mapping and the effects side of the overrides mechanism.

use std::path::{Path, PathBuf};
use voxelizer::*;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture")
}

fn load_fixture() -> tileset::Tileset {
    tileset::load_decomposed(&fixture_dir()).expect("fixture loads")
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vox_fx_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Independent direct sprite sample (does NOT call voxelizer code):
/// alpha-weighted mean computed straight from the composited pixels.
fn direct_mean(img: &image::RgbaImage) -> [f32; 3] {
    let mut sum = [0f64; 3];
    let mut asum = 0f64;
    for px in img.pixels() {
        let a = px[3] as f64;
        if a == 0.0 {
            continue;
        }
        for c in 0..3 {
            sum[c] += px[c] as f64 * a;
        }
        asum += a;
    }
    [
        (sum[0] / asum / 255.0) as f32,
        (sum[1] / asum / 255.0) as f32,
        (sum[2] / asum / 255.0) as f32,
    ]
}

/// 2b.1: every requested field type yields an entry or a logged
/// fallback; palette matches a direct sample of the source sprite;
/// extraction is deterministic (two runs byte-identical).
#[test]
fn gate_2b_1_extraction_fixture() {
    let ts = load_fixture();
    let cfg = VoxelizerConfig::default();
    // t_grass / t_dirt stand in as sprite-bearing "field" ids in the
    // fixture (the fixture ships no fd_* entries); fd_smoke_fixture is
    // deliberately unresolvable and must take the logged fallback.
    let ids: Vec<String> = vec!["t_grass".into(), "t_dirt".into(), "fd_smoke_fixture".into()];
    let (entries, log) = extract_effects(&ts, &cfg, &ids);
    assert_eq!(entries.len(), 3, "every field type yields an entry");
    let grass = entries.iter().find(|e| e.field_id == "t_grass").unwrap();
    assert!(!grass.fallback);
    // Palette == direct sample of the composited sprite.
    let entry = ts.entries.get("t_grass").unwrap().clone();
    let comp = composite_entry(&ts, &entry).unwrap();
    let want = direct_mean(&comp);
    for c in 0..3 {
        assert!(
            (grass.base_color[c] - want[c]).abs() < 0.005,
            "channel {c}: extracted {} vs direct sample {}",
            grass.base_color[c],
            want[c]
        );
    }
    // Fallback: explicit, logged, generic palette — never a sprite.
    let fb = entries
        .iter()
        .find(|e| e.field_id == "fd_smoke_fixture")
        .unwrap();
    assert!(fb.fallback, "unresolvable id must be a marked fallback");
    assert_eq!(log.len(), 1);
    assert!(log[0].contains("fd_smoke_fixture"));
    // fd_smoke_fixture still classifies as smoke from its id.
    assert_eq!(fb.render_class, RenderClass::Smoke);

    // Determinism: a second extraction serializes byte-identically,
    // and so does the file written by run_effects.
    let (entries2, _) = extract_effects(&ts, &cfg, &ids);
    assert_eq!(effects_json(&entries), effects_json(&entries2));
    let out1 = tmpdir("fx1");
    let out2 = tmpdir("fx2");
    run_effects(&ts, &cfg, &ids, &out1).unwrap();
    run_effects(&ts, &cfg, &ids, &out2).unwrap();
    assert_eq!(
        std::fs::read(out1.join("effects.json")).unwrap(),
        std::fs::read(out2.join("effects.json")).unwrap(),
        "effects.json must be byte-identical across runs"
    );
    let fallback_log = std::fs::read_to_string(out1.join("effects_fallback.log")).unwrap();
    assert!(fallback_log.contains("fd_smoke_fixture"));
}

/// Render-class mapping covers the real UltiCa field id set.
#[test]
fn render_class_mapping_ultica_ids() {
    let cases: &[(&str, RenderClass)] = &[
        ("fd_fire_int1", RenderClass::Flame),
        ("fd_fire_int2", RenderClass::Flame),
        ("fd_smoke_overlay", RenderClass::Smoke),
        ("fd_cigsmoke", RenderClass::Smoke),
        ("fd_cracksmoke", RenderClass::Smoke),
        ("fd_methsmoke", RenderClass::Smoke),
        ("fd_weedsmoke", RenderClass::Smoke),
        ("fd_tindalos_gas", RenderClass::Gas),
        ("fd_fungal_haze_int1", RenderClass::Gas),
        ("fd_dust_int1", RenderClass::Gas),
        ("fd_blood", RenderClass::Liquid),
        ("fd_blood_int3", RenderClass::Liquid),
        ("fd_acid", RenderClass::Liquid),
        ("fd_bile", RenderClass::Liquid),
        ("fd_sludge", RenderClass::Liquid),
        ("fd_mechanical_fluid", RenderClass::Liquid),
        ("fd_electricity", RenderClass::Energy),
        ("fd_spotlight", RenderClass::Energy),
        ("fd_congealed_light", RenderClass::Energy),
        ("fd_web", RenderClass::Generic),
        ("fd_splinters", RenderClass::Generic),
    ];
    for (id, want) in cases {
        assert_eq!(classify_field(id), *want, "classify {id}");
    }
}

/// Effects honor the overrides.toml mechanism (5.9's file, extended).
#[test]
fn effects_overrides_apply() {
    let text = "[fd_smoke]\neffect_density = 2.5\neffect_class = \"gas\"\n\n[fd_fire]\neffect_emissive = 3.5\n";
    let ov = parse_overrides(text).unwrap();
    let mut cfg = VoxelizerConfig::default();
    cfg.overrides = ov;
    let smoke = effect_from_stats("fd_smoke", None, &cfg);
    assert_eq!(smoke.density_scale, 2.5);
    assert_eq!(smoke.render_class, RenderClass::Gas);
    let fire = effect_from_stats("fd_fire", None, &cfg);
    assert_eq!(fire.emissive_strength, 3.5);
    // Invalid effect overrides are clear errors, like 5.9's.
    assert!(parse_overrides("[fd_x]\neffect_class = \"plasma\"\n").is_err());
    assert!(parse_overrides("[fd_x]\neffect_density = 99\n").is_err());
}

/// sample_sprite ignores poison colors under fully transparent texels.
#[test]
fn sample_sprite_premultiplied_clean() {
    let mut img = image::RgbaImage::new(2, 1);
    img.put_pixel(0, 0, image::Rgba([200, 100, 50, 255]));
    img.put_pixel(1, 0, image::Rgba([0, 255, 0, 0])); // poison green
    let s = sample_sprite(&img);
    assert_eq!(s.mean_color, [200.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0]);
    assert_eq!(s.coverage, 0.5);
}
