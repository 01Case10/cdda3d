//! Stage B coverage measurement (Phase 5 rework): enumerate the real
//! id universe of a tileset, run the material classifier over every
//! id (with its sprite family's extracted palette), and report
//! coverage by rule bucket and by class.
//!
//! Usage:
//!   cargo run -p voxelizer --example classify_coverage -- \
//!       <tileset_dir> [audit_tsv_out]
//!
//! The audit TSV (default `target/classify_coverage.tsv`) has one
//! line per id: `id <TAB> class <TAB> rule`. Everything is
//! deterministic: two runs over the same tileset produce a
//! byte-identical audit file.

use std::collections::BTreeMap;
use voxelizer::*;

/// Strip the tileset resolution suffixes (`_var_<variant>`,
/// `_season_<season>`) to get the base id a prefix statistic should
/// count (mirrors crates/tileset resolve order: variant, then season).
fn base_id(id: &str) -> &str {
    let mut end = id.len();
    if let Some(p) = id.find("_var_") {
        end = end.min(p);
    }
    if let Some(p) = id.find("_season_") {
        end = end.min(p);
    }
    &id[..end]
}

/// Prefix for grouping: first two `_`-separated tokens for the
/// prefixed kinds (`t_grass`, `vp_frame`, ...), first token otherwise
/// (`overlay`, `seed`, ...).
fn prefix_of(base: &str) -> String {
    let mut parts = base.split('_');
    let first = parts.next().unwrap_or("");
    if matches!(first, "t" | "f" | "fd" | "vp" | "mon") {
        match parts.next() {
            Some(second) => format!("{first}_{second}"),
            None => first.to_string(),
        }
    } else {
        first.to_string()
    }
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 * 100.0 / d as f64
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = match args.first() {
        Some(d) => d.clone(),
        None => {
            eprintln!("usage: classify_coverage <tileset_dir> [audit_tsv_out]");
            std::process::exit(2);
        }
    };
    let audit_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "target/classify_coverage.tsv".to_string());

    let ts = tileset::load_decomposed(std::path::Path::new(&dir)).unwrap_or_else(|e| {
        eprintln!("tileset load failed: {e:?}");
        std::process::exit(1);
    });
    let ids: Vec<String> = ts.entries.keys().cloned().collect();
    println!("id universe: {} entries", ids.len());

    // --- Prefix enumeration over base ids --------------------------
    let mut base_counts: BTreeMap<String, usize> = BTreeMap::new();
    for id in &ids {
        *base_counts.entry(base_id(id).to_string()).or_insert(0) += 1;
    }
    let mut prefix_counts: BTreeMap<String, usize> = BTreeMap::new();
    for b in base_counts.keys() {
        *prefix_counts.entry(prefix_of(b)).or_insert(0) += 1;
    }
    let mut prefixes: Vec<(&String, &usize)> = prefix_counts.iter().collect();
    prefixes.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!(
        "base ids (variant/season stripped): {}",
        base_counts.len()
    );
    println!("top prefixes by distinct base id count:");
    for (p, n) in prefixes.iter().take(40) {
        println!("  {p:<28} {n}");
    }

    // --- Classification over the full universe ---------------------
    let overrides = Overrides::default();
    let mut audit = String::new();
    // scope -> bucket -> count
    let mut scopes: BTreeMap<&'static str, BTreeMap<&'static str, usize>> = BTreeMap::new();
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut unknown_ids: Vec<String> = Vec::new();
    for id in &ids {
        let entry = &ts.entries[id];
        let palette = composite_entry(&ts, entry)
            .map(|img| analyze_image(&img, 5).palette)
            .unwrap_or_default();
        let kind = TileKind::from_id_prefix(id);
        let c = classify(id, Some(kind), Some(&palette), &overrides);
        audit.push_str(&format!("{id}\t{}\t{}\n", c.class.name(), c.rule.name()));
        *class_counts.entry(c.class.name().to_string()).or_insert(0) += 1;
        let bucket = c.rule.bucket();
        let scope_names: &[&'static str] = match kind {
            TileKind::Terrain | TileKind::Furniture => &["all", "block (t_/f_/vp_)", "terrain+furniture"],
            TileKind::VehiclePart => &["all", "block (t_/f_/vp_)"],
            _ => &["all"],
        };
        for s in scope_names {
            *scopes.entry(s).or_default().entry(bucket).or_insert(0) += 1;
        }
        if c.class == MaterialClass::Unknown && c.rule.bucket() == "unknown" {
            unknown_ids.push(id.clone());
        }
    }
    if let Some(parent) = std::path::Path::new(&audit_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&audit_path, &audit).expect("write audit tsv");
    println!("audit file: {audit_path}");

    for (scope, buckets) in &scopes {
        let total: usize = buckets.values().sum();
        let ruled = buckets.get("override").copied().unwrap_or(0)
            + buckets.get("pattern").copied().unwrap_or(0)
            + buckets.get("keyword").copied().unwrap_or(0);
        let fallback = buckets.get("color-fallback").copied().unwrap_or(0);
        let unknown = buckets.get("unknown").copied().unwrap_or(0);
        let classified = total - unknown - buckets.get("kind").copied().unwrap_or(0);
        println!(
            "scope {scope}: {total} ids | ruled (override+pattern+keyword) {:.1}% | color-fallback {:.1}% | unknown {:.1}% | any class {:.1}%",
            pct(ruled, total),
            pct(fallback, total),
            pct(unknown, total),
            pct(classified, total)
        );
        let mut bs: Vec<(&&str, &usize)> = buckets.iter().collect();
        bs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (b, n) in bs {
            println!("    {b:<16} {n}");
        }
    }
    println!("classes:");
    let mut cs: Vec<(&String, &usize)> = class_counts.iter().collect();
    cs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (c, n) in cs {
        println!("  {c:<12} {n}");
    }
    println!("fully-unknown ids (no rule, no palette): {}", unknown_ids.len());
    for id in unknown_ids.iter().take(30) {
        println!("  unknown: {id}");
    }
}
