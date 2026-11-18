//! Terrain drive gate (2026-11-17, Joey: models for
//! every tile, all seasons): every terrain material in
//! the tileset yields a non-empty v2 terrain model via
//! `props::terrain_model_for`, built from its FG sprite
//! exactly as the v2 emission loads it. Season variants
//! are separate ids and are covered as their own
//! entries. TERRAIN_TOTAL is the measured universe size
//! (t_-prefixed tileset entries with a loadable sprite).

use std::path::PathBuf;
use voxelizer::*;

fn ultica_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest.join("../../../cdda-voxel/fork-src/tilesets/gfx/UltimateCataclysm"),
        manifest.join("../../fork-src/tilesets/gfx/UltimateCataclysm"),
    ];
    for c in &candidates {
        if c.join("tile_info.json").exists() {
            return c.clone();
        }
    }
    panic!("UltiCa tileset not found; tried {candidates:?}");
}

const TERRAIN_TOTAL: usize = 1074;

#[test]
fn terrain_coverage() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let mut ids: Vec<&String> = ts
        .entries
        .keys()
        .filter(|k| k.starts_with("t_"))
        .collect();
    ids.sort();
    let mut total = 0usize;
    let mut missing: Vec<String> = Vec::new();
    let mut no_sprite: Vec<String> = Vec::new();
    for id in &ids {
        let entry = &ts.entries[*id];
        let fg = if *id == "t_chainfence" {
            entry
                .fg
                .as_ref()
                .and_then(|r| load_sprite_image(&ts, entry, r))
        } else {
            terrain_fg_image(&ts, entry)
        };
        let Some(sprite) = fg.or_else(|| composite_entry(&ts, entry)) else {
            no_sprite.push((*id).clone());
            continue;
        };
        total += 1;
        match props::terrain_model_for(id, &sprite) {
            Some(m) if m.voxels.iter().any(|v| v.is_some()) => {}
            _ => missing.push((*id).clone()),
        }
    }
    // The open-air family is the one legitimate
    // exception: its sprite is fully transparent (it IS
    // empty space), so no model exists by design. Any
    // other id landing in `missing` has a sprite with no
    // opaque pixels or a builder bug -- fail on those.
    let unexpected: Vec<&String> = missing
        .iter()
        .filter(|id| !id.starts_with("t_open_air"))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{} terrain ids without a model: {:?}",
        unexpected.len(),
        &unexpected[..unexpected.len().min(40)]
    );
    assert_eq!(
        total, TERRAIN_TOTAL,
        "terrain universe drifted (no-sprite ids: {})",
        no_sprite.len()
    );
}
