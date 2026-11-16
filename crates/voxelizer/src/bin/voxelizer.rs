//! voxelizer CLI (plan §5 Phase 5 deliverable).
//! Usage:
//!   voxelizer --tileset <dir> --out <dir> [--composed <tile_config.json>]
//!             [--overrides <file>] [--season spring|summer|autumn|winter]
//!             [--ids <file: one game id per line>]
//!             [--looks-like <file: TSV id<TAB>looks_like_id>]
//!             [--game-json <dir-or-file>...]  (extract looks_like from CDDA game JSON)
//!
//!   voxelizer preview --tileset <dir> --out <sheet.png> --ids a,b,c
//!             [--ids-file <file>] [--overrides <file>] [--season <s>]
//!             (Phase 5 rework Stage B: contact sheet of source sprite +
//!              extracted palette + classification band per id; the
//!              per-id class/rule/analysis table goes to stdout — the
//!              pipeline has no font renderer, so the PNG carries no
//!              text; cell order matches the stdout table.)

use std::collections::BTreeMap;
use std::path::PathBuf;
use voxelizer::{field_ids_in_tileset, parse_overrides, run, run_effects, VoxelizerConfig};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|a| a == "preview").unwrap_or(false) {
        std::process::exit(run_preview(&args[1..]));
    }
    let mut tileset_dir: Option<PathBuf> = None;
    let mut composed: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut overrides_path: Option<PathBuf> = None;
    let mut season = "spring".to_string();
    let mut ids_path: Option<PathBuf> = None;
    let mut ll_path: Option<PathBuf> = None;
    let mut game_json: Vec<PathBuf> = Vec::new();
    let mut want_effects = false;
    let mut field_ids_path: Option<PathBuf> = None;
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
            "--effects" => want_effects = true,
            "--field-ids" => field_ids_path = Some(take(&mut i)),
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
    if want_effects {
        // Phase 2b: effect-parameter extraction. Field ids default to
        // every fd_* entry in the tileset; --field-ids adds explicit ids.
        let mut field_ids = field_ids_in_tileset(&ts);
        if let Some(p) = &field_ids_path {
            if let Ok(text) = std::fs::read_to_string(p) {
                for l in text.lines() {
                    let l = l.trim();
                    if !l.is_empty() {
                        field_ids.push(l.to_string());
                    }
                }
            }
        }
        match run_effects(&ts, &cfg, &field_ids, &out) {
            Ok((entries, log)) => {
                let fallbacks = entries.iter().filter(|e| e.fallback).count();
                println!(
                    "voxelizer effects: {} field types, {} with sprites, {} fallbacks (see effects_fallback.log)",
                    entries.len(),
                    entries.len() - fallbacks,
                    fallbacks
                );
                for line in &log {
                    println!("  fallback: {line}");
                }
            }
            Err(e) => {
                eprintln!("voxelizer: effects run failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Stage B contact sheet: for each id, one cell with the source
/// sprite (scaled to fit, nearest), its k-means palette as a
/// weight-proportional swatch strip, and a bottom band filled with
/// the classified material's anchor color. Prints the audit table
/// (index, id, class, rule, shading, orientation) to stdout.
fn run_preview(args: &[String]) -> i32 {
    use image::{Rgba, RgbaImage};
    use voxelizer::{
        analyze_image, classify, composite_entry, TileKind,
    };

    let mut tileset_dir: Option<PathBuf> = None;
    let mut composed: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut overrides_path: Option<PathBuf> = None;
    let mut season = "spring".to_string();
    let mut ids: Vec<String> = Vec::new();
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
            "--ids" => {
                let v = take(&mut i);
                for part in v.to_string_lossy().split(',') {
                    let p = part.trim();
                    if !p.is_empty() {
                        ids.push(p.to_string());
                    }
                }
            }
            "--ids-file" => {
                let p = take(&mut i);
                match std::fs::read_to_string(&p) {
                    Ok(text) => {
                        for l in text.lines() {
                            let l = l.trim();
                            if !l.is_empty() {
                                ids.push(l.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("voxelizer preview: cannot read ids file: {e}");
                        return 1;
                    }
                }
            }
            "-h" | "--help" => {
                println!("voxelizer preview --tileset <dir> --out <sheet.png> --ids a,b,c [--ids-file f] [--overrides f] [--season s]");
                return 0;
            }
            other => {
                eprintln!("voxelizer preview: unknown argument {other}");
                return 2;
            }
        }
        i += 1;
    }
    let out = match out {
        Some(o) => o,
        None => {
            eprintln!("voxelizer preview: --out is required");
            return 2;
        }
    };
    if ids.is_empty() {
        eprintln!("voxelizer preview: no ids given (--ids or --ids-file)");
        return 2;
    }
    let ts = if let Some(cfg) = &composed {
        match tileset::load_composed(cfg) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("voxelizer preview: composed load failed: {e}");
                return 1;
            }
        }
    } else if let Some(dir) = &tileset_dir {
        match tileset::load_decomposed(dir) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("voxelizer preview: tileset load failed: {e}");
                return 1;
            }
        }
    } else {
        eprintln!("voxelizer preview: --tileset or --composed is required");
        return 2;
    };
    let overrides = match &overrides_path {
        Some(p) => match std::fs::read_to_string(p)
            .map_err(|e| e.to_string())
            .and_then(|t| parse_overrides(&t))
        {
            Ok(o) => o,
            Err(e) => {
                eprintln!("voxelizer preview: invalid overrides: {e}");
                return 1;
            }
        },
        None => voxelizer::Overrides::default(),
    };

    const CELL_W: u32 = 184;
    const CELL_H: u32 = 184;
    const SPRITE_BOX: u32 = 144;
    let cols = (ids.len() as f64).sqrt().ceil() as u32;
    let cols = cols.max(1);
    let rows = (ids.len() as u32 + cols - 1) / cols;
    let mut sheet = RgbaImage::from_pixel(cols * CELL_W, rows * CELL_H, Rgba([24, 24, 28, 255]));

    println!("idx\tid\tclass\trule\tshading\torientation\tpalette (top 3)");
    for (idx, id) in ids.iter().enumerate() {
        let cx = (idx as u32 % cols) * CELL_W;
        let cy = (idx as u32 / cols) * CELL_H;
        let res = tileset::resolve(&ts, id, &BTreeMap::new(), None, &season);
        let sprite = res
            .entry
            .as_ref()
            .and_then(|e| composite_entry(&ts, e));
        let analysis = sprite.as_ref().map(|img| analyze_image(img, 5));
        let palette: Vec<voxelizer::PaletteColor> =
            analysis.as_ref().map(|a| a.palette.clone()).unwrap_or_default();
        let class = classify(
            id,
            Some(TileKind::from_id_prefix(id)),
            Some(&palette),
            &overrides,
        );
        // Sprite, scaled to fit the box (nearest), centered.
        if let Some(img) = &sprite {
            let scale = (SPRITE_BOX as f32 / img.width().max(img.height()) as f32).min(4.0);
            let nw = ((img.width() as f32 * scale) as u32).max(1);
            let nh = ((img.height() as f32 * scale) as u32).max(1);
            let scaled = image::imageops::resize(
                img,
                nw,
                nh,
                image::imageops::FilterType::Nearest,
            );
            let ox = cx + 8 + (SPRITE_BOX - nw) / 2;
            let oy = cy + 8 + (SPRITE_BOX - nh) / 2;
            image::imageops::replace(&mut sheet, &scaled, ox as i64, oy as i64);
        }
        // Palette strip: swatches stacked by weight at the cell's
        // right edge of the sprite box.
        let strip_x = cx + 8 + SPRITE_BOX + 4;
        let mut sy = cy + 8;
        for p in &palette {
            let h = ((p.weight * SPRITE_BOX as f32) as u32).max(2);
            let h = h.min(cy + 8 + SPRITE_BOX - sy);
            for y in sy..sy + h {
                for x in strip_x..strip_x + 16 {
                    sheet.put_pixel(x, y, Rgba([p.srgb[0], p.srgb[1], p.srgb[2], 255]));
                }
            }
            sy += h;
            if sy >= cy + 8 + SPRITE_BOX {
                break;
            }
        }
        // Classification band: the class anchor color.
        let anchor = class.class.anchor_srgb();
        for y in cy + CELL_H - 20..cy + CELL_H - 8 {
            for x in cx + 8..cx + CELL_W - 8 {
                sheet.put_pixel(x, y, Rgba([anchor[0], anchor[1], anchor[2], 255]));
            }
        }
        let (shading, orient) = match &analysis {
            Some(a) => (
                format!("{:.2}", a.shading.shading_factor),
                format!("{:?}", a.orientation.kind),
            ),
            None => ("-".to_string(), "-".to_string()),
        };
        let top3: Vec<String> = palette
            .iter()
            .take(3)
            .map(|p| format!("#{:02x}{:02x}{:02x}:{:.0}%", p.srgb[0], p.srgb[1], p.srgb[2], p.weight * 100.0))
            .collect();
        println!(
            "{idx}\t{id}\t{}\t{}\t{shading}\t{orient}\t{}",
            class.class.name(),
            class.rule.name(),
            top3.join(" ")
        );
    }
    match sheet.save(&out) {
        Ok(()) => {
            println!("voxelizer preview: wrote {}", out.display());
            0
        }
        Err(e) => {
            eprintln!("voxelizer preview: save failed: {e}");
            1
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
