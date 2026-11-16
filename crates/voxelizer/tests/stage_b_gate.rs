//! Phase 5 rework Stage B gate tests: classifier behavior on the
//! committed fixture tileset + end-to-end determinism (analyze +
//! classify every fixture id twice; the serialized results must be
//! byte-identical).

use std::path::{Path, PathBuf};
use voxelizer::*;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture")
}

fn load_fixture() -> tileset::Tileset {
    tileset::load_decomposed(&fixture_dir()).expect("fixture loads")
}

/// Run analyze+classify over every fixture entry and serialize the
/// results in id order (the audit format used by the coverage tool).
fn audit_pass(ts: &tileset::Tileset) -> String {
    let overrides = Overrides::default();
    let mut out = String::new();
    for (id, entry) in &ts.entries {
        let palette = composite_entry(ts, entry)
            .map(|img| analyze_image(&img, 5).palette)
            .unwrap_or_default();
        let c = classify(id, Some(TileKind::from_id_prefix(id)), Some(&palette), &overrides);
        out.push_str(&format!("{id}\t{}\t{}\n", c.class.name(), c.rule.name()));
        for p in &palette {
            out.push_str(&format!(
                "  #{:02x}{:02x}{:02x} {:.4}\n",
                p.srgb[0], p.srgb[1], p.srgb[2], p.weight
            ));
        }
    }
    out
}

#[test]
fn stage_b_fixture_classifications() {
    let ts = load_fixture();
    let ov = Overrides::default();
    let class_of = |id: &str| {
        let entry = ts.entries.get(id).unwrap_or_else(|| panic!("fixture has {id}"));
        let palette = composite_entry(&ts, entry)
            .map(|img| analyze_image(&img, 5).palette)
            .unwrap_or_default();
        classify(id, Some(TileKind::from_id_prefix(id)), Some(&palette), &ov)
    };
    assert_eq!(class_of("t_grass").class, MaterialClass::Grass);
    assert_eq!(class_of("t_dirt").class, MaterialClass::Dirt);
    assert_eq!(class_of("t_wall").class, MaterialClass::Plaster);
    assert_eq!(class_of("t_floor_wood").class, MaterialClass::WoodFloor);
    assert_eq!(class_of("t_fence").class, MaterialClass::Wood);
    assert_eq!(class_of("t_glass_window").class, MaterialClass::Glass);
    // Monsters are gated by kind even though a palette exists.
    assert_eq!(class_of("mon_tall").rule, Rule::KindNotBlock);
}

#[test]
fn stage_b_determinism_fixture_audit() {
    let ts = load_fixture();
    let a = audit_pass(&ts);
    let b = audit_pass(&ts);
    assert_eq!(a, b, "two analyze+classify passes must be byte-identical");
    assert!(a.lines().count() > 10, "audit should cover the fixture ids");
}
