//! Prop-model coverage audit (Stage G coverage drive): for every
//! furniture (`f_*`) entry in the decomposed UltiCa tileset, run
//! the exact chain the v2 emitter runs (`composite_entry` ->
//! `model_for`) plus the diagnostics that explain a decline
//! (`segment`, `view_for`, `archetype_for`, sprite size, alpha
//! coverage, multitile/rotates flags), and write a TSV.
//!
//! Usage:
//!   cargo run -p voxelizer --example prop_coverage_audit -- \
//!       <tileset_dir> [audit_tsv_out]
//!
//! TSV columns: id, w, h, multitile, rotates, alpha_frac,
//! seg_conf (or "-"), view, archetype (or "-"), model_cells
//! (or "-"). Deterministic: same tileset, byte-identical TSV.

use std::collections::BTreeMap;
use std::io::Write;
use voxelizer::*;

fn ultica_arg() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first() {
        Some(d) => d.clone(),
        None => {
            eprintln!("usage: prop_coverage_audit <tileset_dir> [audit_tsv_out]");
            std::process::exit(2);
        }
    }
}

fn main() {
    let dir = ultica_arg();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let audit_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "target/prop_coverage.tsv".to_string());
    let ts = tileset::load_decomposed(std::path::Path::new(&dir)).unwrap_or_else(|e| {
        eprintln!("tileset load failed: {e:?}");
        std::process::exit(1);
    });
    let mut ids: Vec<String> = ts
        .entries
        .keys()
        .filter(|k| k.starts_with("f_"))
        .cloned()
        .collect();
    ids.sort();
    println!("furniture entries: {}", ids.len());

    let mut out = String::new();
    out.push_str(
        "id\tw\th\tmultitile\trotates\talpha_frac\tseg_conf\tview\tarchetype\tmodel_cells\n",
    );
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for id in &ids {
        let entry = ts.entries.get(id).unwrap();
        let sprite = composite_entry(&ts, entry);
        let (w, h, alpha_frac, seg_s, view_s, arch_s, model_s) = match &sprite {
            None => (0, 0, 0.0f32, "-".to_string(), "-".into(), "-".into(), "-".into()),
            Some(img) => {
                let total = (img.width() * img.height()).max(1) as f32;
                let opaque = img.pixels().filter(|p| p[3] >= 8).count() as f32;
                let seg = segment(img);
                let seg_s = match &seg {
                    Some(s) => format!("{:.3}", s.confidence),
                    None => "-".to_string(),
                };
                let view_s = format!("{:?}", view_for(Some(id), img));
                let arch_s = match archetype_for(id) {
                    Some(a) => format!("{a:?}"),
                    None => "-".to_string(),
                };
                let model_s = match model_for(id, img) {
                    Some(m) => m.solid_count().to_string(),
                    None => "-".to_string(),
                };
                (
                    img.width(),
                    img.height(),
                    opaque / total,
                    seg_s,
                    view_s,
                    arch_s,
                    model_s,
                )
            }
        };
        let bucket = if model_s != "-" {
            "model"
        } else if sprite.is_none() {
            "no-composite"
        } else if seg_s == "-" {
            "no-segment"
        } else {
            "segment-but-no-model"
        };
        *counts.entry(bucket.to_string()).or_insert(0) += 1;
        out.push_str(&format!(
            "{id}\t{w}\t{h}\t{}\t{}\t{alpha_frac:.3}\t{seg_s}\t{view_s}\t{arch_s}\t{model_s}\n",
            entry.multitile as u8, entry.rotates as u8
        ));
    }
    for (k, v) in &counts {
        println!("{k}: {v}");
    }
    let mut f = std::fs::File::create(&audit_path).expect("create audit tsv");
    f.write_all(out.as_bytes()).expect("write audit tsv");
    println!("wrote {audit_path}");
}
