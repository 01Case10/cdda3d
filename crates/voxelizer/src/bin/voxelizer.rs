//! voxelizer CLI (plan §5 Phase 5 deliverable).
//! Usage:
//!   voxelizer --tileset <dir> --out <dir> [--composed <tile_config.json>]
//!             [--overrides <file>] [--season spring|summer|autumn|winter]
//!             [--ids <file: one game id per line>]
//!             [--looks-like <file: TSV id<TAB>looks_like_id>]
//!             [--game-json <dir-or-file>...]  (extract looks_like from CDDA game JSON)

use std::collections::BTreeMap;
use std::path::PathBuf;
use voxelizer::{parse_overrides, run, VoxelizerConfig};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut tileset_dir: Option<PathBuf> = None;
    let mut composed: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut overrides_path: Option<PathBuf> = None;
    let mut season = "spring".to_string();
    let mut ids_path: Option<PathBuf> = None;
    let mut ll_path: Option<PathBuf> = None;
    let mut game_json: Vec<PathBuf> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let mut take = |i: &mut usize| -> PathBuf {
            *i += 1;
            PathBuf::from(args.get(*i).cloned().unwrap_or_default())
        };
        match args[i].as_str() {
            "--tileset" => tileset_dir = Some(take(&mut i)),
            "--composed" => composed = Some(take(&mut i)),
            "--out" => out = Some(take(&mut i)),
            "--overrides" => overrides_path = Some(take(&mut i)),
            "--season" => {
                i += 1;
                season = args.get(i).cloned().unwrap_or(season);
            }
            "--ids" => ids_path = Some(take(&mut i)),
            "--looks-like" => ll_path = Some(take(&mut i)),
            "--game-json" => game_json.push(take(&mut i)),
            "-h" | "--help" => {
                println!("voxelizer --tileset <dir> --out <dir> [--composed <tile_config.json>] [--overrides f] [--season s] [--ids f] [--looks-like f] [--game-json path...]");
                return;
            }
            other => {
                eprintln!("voxelizer: unknown argument {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let out = match out {
        Some(o) => o,
        None => {
            eprintln!("voxelizer: --out is required");
            std::process::exit(2);
        }
    };
    let ts = if let Some(cfg) = &composed {
        match tileset::load_composed(cfg) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("voxelizer: composed load failed: {e}");
                std::process::exit(1);
            }
        }
    } else if let Some(dir) = &tileset_dir {
        match tileset::load_decomposed(dir) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("voxelizer: tileset load failed: {e}");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("voxelizer: --tileset or --composed is required");
        std::process::exit(2);
    };

    let mut cfg = VoxelizerConfig {
        season,
        tile_size: ts.base_width.max(1),
        ..Default::default()
    };
    if let Some(p) = &overrides_path {
        let text = std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("voxelizer: cannot read overrides: {e}");
            std::process::exit(1);
        });
        cfg.overrides = parse_overrides(&text).unwrap_or_else(|e| {
            eprintln!("voxelizer: invalid overrides: {e}");
            std::process::exit(1);
        });
    }
    if let Some(p) = &ids_path {
        if let Ok(text) = std::fs::read_to_string(p) {
            cfg.extra_ids = text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }
    }
    if let Some(p) = &ll_path {
        if let Ok(text) = std::fs::read_to_string(p) {
            for line in text.lines() {
                if let Some((a, b)) = line.split_once('\t') {
                    cfg.looks_like
                        .insert(a.trim().to_string(), b.trim().to_string());
                }
            }
        }
    }
    for gj in &game_json {
        extract_looks_like(gj, &mut cfg.looks_like);
    }
    // License text for attribution: <tileset root>/../LICENSE.txt or root/LICENSE.txt
    let lic_candidates = [
        ts.root.join("LICENSE.txt"),
        ts.root.join("../LICENSE.txt"),
        ts.root.join("../../LICENSE.txt"),
    ];
    for c in &lic_candidates {
        if let Ok(t) = std::fs::read_to_string(c) {
            cfg.tileset_license_text = Some(t);
            break;
        }
    }
    let t0 = std::time::Instant::now();
    match run(&ts, &cfg, &out) {
        Ok(report) => {
            println!(
                "voxelizer: {} materials, {} layers, {} sprites, {} missing in {:?}",
                report.materials,
                report.layers,
                report.sprites,
                report.missing.len(),
                t0.elapsed()
            );
            for (stage, count) in &report.stage_counts {
                println!("  resolution {stage}: {count}");
            }
        }
        Err(e) => {
            eprintln!("voxelizer: run failed: {e}");
            std::process::exit(1);
        }
    }
}

/// Extract id -> looks_like from CDDA game JSON files (objects with
/// "id" and "looks_like" string fields), recursively over directories.
fn extract_looks_like(path: &PathBuf, map: &mut BTreeMap<String, String>) {
    let mut stack = vec![path.clone()];
    while let Some(p) = stack.pop() {
        if p.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&p) {
                for e in rd.filter_map(|e| e.ok()) {
                    stack.push(e.path());
                }
            }
        } else if p.extension().map(|x| x == "json").unwrap_or(false) {
            if let Ok(text) = std::fs::read_to_string(&p) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    collect_ll(&v, map);
                }
            }
        }
    }
}

fn collect_ll(v: &serde_json::Value, map: &mut BTreeMap<String, String>) {
    match v {
        serde_json::Value::Object(o) => {
            if let (Some(id), Some(ll)) = (
                o.get("id").and_then(|x| x.as_str()),
                o.get("looks_like").and_then(|x| x.as_str()),
            ) {
                if !ll.is_empty() {
                    map.entry(id.to_string()).or_insert_with(|| ll.to_string());
                }
            }
            for child in o.values() {
                collect_ll(child, map);
            }
        }
        serde_json::Value::Array(a) => {
            for child in a {
                collect_ll(child, map);
            }
        }
        _ => {}
    }
}
