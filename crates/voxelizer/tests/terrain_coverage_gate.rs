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

/// Terrain item 1 (2026-11-23, Joey's call: densify dead
/// bamboo to match the sprite). The dead sprites carry ~7
/// distinct culms (measured: 7 column clusters at a 0.30
/// opacity threshold with no gap-merging); the shared
/// builder's 0.45 threshold + gap-merge fused them into 2
/// fat culms. The dead family now clusters at 0.30 with no
/// merging and draws thin culms; the live family must not
/// move at all (whole-model FNV pins, measured pre-fix).
#[test]
fn bamboo_dead_is_dense_and_live_is_unchanged() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let load = |id: &str| {
        let entry = ts.entries.get(id).unwrap_or_else(|| panic!("tileset has {id}"));
        terrain_fg_image(&ts, entry)
            .or_else(|| composite_entry(&ts, entry))
            .expect("sprite")
    };
    let components_at = |m: &props::PropModel, y: u32| -> usize {
        let mut seen = [[false; 32]; 32];
        let mut n = 0;
        for x in 0..32u32 {
            for z in 0..32u32 {
                if seen[x as usize][z as usize] || m.get(x, y, z).is_none() {
                    continue;
                }
                n += 1;
                let mut stack = vec![(x, z)];
                seen[x as usize][z as usize] = true;
                while let Some((cx, cz)) = stack.pop() {
                    for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, nz) = (cx as i32 + dx, cz as i32 + dz);
                        if nx < 0 || nz < 0 || nx >= 32 || nz >= 32 {
                            continue;
                        }
                        if !seen[nx as usize][nz as usize]
                            && m.get(nx as u32, y, nz as u32).is_some()
                        {
                            seen[nx as usize][nz as usize] = true;
                            stack.push((nx as u32, nz as u32));
                        }
                    }
                }
            }
        }
        n
    };
    for id in ["t_bamboo_dead", "t_bamboo_dead_season_winter"] {
        let (m, _, _) = props::terrain_model_for(id, &load(id), None).expect("dead bamboo model");
        let n = components_at(&m, 20);
        assert!(
            n >= 6,
            "{id}: {n} culm components at y=20, want >= 6 (sprite has ~7; was 2 pre-fix)"
        );
    }
    let mut hash_of = |id: &str| -> u64 {
        let (m, _, _) = props::terrain_model_for(id, &load(id), None).expect("live bamboo model");
        let mut hash: u64 = 0xcbf29ce484222325;
        for v in &m.voxels {
            let bs: &[u8] = match v {
                Some(c) => {
                    hash ^= 1;
                    hash = hash.wrapping_mul(0x100000001b3);
                    &c.0
                }
                None => {
                    hash ^= 0;
                    hash = hash.wrapping_mul(0x100000001b3);
                    &[]
                }
            };
            for &b in bs {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash
    };
    assert_eq!(hash_of("t_bamboo"), 0x7d54a4c1d6623a4e, "live bamboo unchanged");
    assert_eq!(hash_of("t_bamboo_long"), 0xae58b3151f0891c9, "bamboo_long unchanged");
    assert_eq!(hash_of("t_bamboo_tall"), 0x88535d20e2f96ce0, "bamboo_tall unchanged");
}
