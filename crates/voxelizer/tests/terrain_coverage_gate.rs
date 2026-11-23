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
        // Mirror the emission: puddles also get
        // their bg underlay (terrain item 2).
        let underlay = if id.starts_with("t_puddle") {
            entry
                .bg
                .as_ref()
                .and_then(|r| load_sprite_image(&ts, entry, r))
        } else {
            None
        };
        match props::terrain_model_for(id, &sprite, underlay.as_ref()) {
            Some((m, kind, _surface)) if m.voxels.iter().any(|v| v.is_some()) => {
                // Terrain wiring (2026-11-18): the kind the
                // engine seats by is part of the contract --
                // pin the three families' representatives.
                let want = match id.as_str() {
                    "t_grass" | "t_floor" | "t_water_dp" | "t_shingle_flat_roof" => {
                        Some(props::TerrainModelKind::Ground)
                    }
                    "t_chainfence" | "t_chickenwire_fence" => Some(props::TerrainModelKind::Fence),
                    "t_tree" | "t_wall" | "t_door_c" => {
                        Some(props::TerrainModelKind::Standing)
                    }
                    _ => None,
                };
                if let Some(want) = want {
                    assert_eq!(kind, want, "{id}: wrong terrain model kind");
                }
            }
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
