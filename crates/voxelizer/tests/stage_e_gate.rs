//! Phase 5 rework Stage E gate: the eight new recipes (concrete,
//! brick, stone, metal, glass, water, road, shingle) generated
//! from the real UltiCa families must meet the SAME plan SS8
//! targets as Stages C/D — seam ratio < 1.15, mean deltaE_OK vs
//! the family palette < 0.04, palette <= 24 — byte-identical
//! across two generations. Plus the shoreline autotiling checks
//! (plan SS10.1, voxelizer half): curved waterlines on corner and
//! straight-edge masks, all-water identity, determinism, and a
//! composed 5x5 pond mosaic whose water region is round-ish.
//!
//! Tileset path resolution mirrors stage_c_gate (fork-src lives
//! only in the main tree checkout).

use std::path::PathBuf;
use image::{Rgba, RgbaImage};
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

const FAMILIES: [(&str, RecipeKind); 8] = [
    ("t_concrete", RecipeKind::Concrete),
    ("t_brick_wall", RecipeKind::Brick),
    ("t_rock_wall", RecipeKind::Stone),
    ("t_wall_metal", RecipeKind::Metal),
    ("t_wall_glass", RecipeKind::Glass),
    ("t_water_dp", RecipeKind::Water),
    ("t_pavement", RecipeKind::Road),
    ("t_shingle_flat_roof", RecipeKind::Shingle),
];

fn gen(ts: &tileset::Tileset, base: &str, kind: RecipeKind) -> GeneratedTexture {
    let entry = ts
        .entries
        .get(base)
        .unwrap_or_else(|| panic!("tileset has {base}"));
    let family = family_sprites(ts, entry);
    assert!(!family.is_empty(), "{base}: family pool must not be empty");
    generate(kind, &family)
}

#[test]
fn stage_e_recipes_meet_plan_metrics() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (base, kind) in FAMILIES {
        let g = gen(&ts, base, kind);
        assert_eq!(
            (g.image.width(), g.image.height()),
            (TEXTURE_SIZE, TEXTURE_SIZE),
            "{base}: generated texture must be 32x32"
        );
        let m = metrics::evaluate(&g.image, &g.palette);
        println!(
            "{} ({}): seam {:.3}, mean dE {:.4}, max dE {:.4}, palette {}",
            base,
            kind.name(),
            m.seam_ratio,
            m.mean_delta_e,
            m.max_delta_e,
            m.palette_size
        );
        assert!(
            m.seam_ratio < 1.15,
            "{base} ({}): seam ratio {:.3} must be < 1.15",
            kind.name(),
            m.seam_ratio
        );
        assert!(
            m.mean_delta_e < 0.04,
            "{base} ({}): mean deltaE {:.4} must be < 0.04",
            kind.name(),
            m.mean_delta_e
        );
        assert!(
            m.palette_size <= 24,
            "{base} ({}): palette {} must be <= 24",
            kind.name(),
            m.palette_size
        );
        let again = gen(&ts, base, kind);
        assert_eq!(
            g.image.as_raw(),
            again.image.as_raw(),
            "{base} ({}): two generations must be byte-identical",
            kind.name()
        );
        assert_eq!(g.palette, again.palette);
    }
}

fn solid(rgb: [u8; 3]) -> RgbaImage {
    RgbaImage::from_pixel(32, 32, Rgba([rgb[0], rgb[1], rgb[2], 255]))
}

const WATER_RGB: [u8; 3] = [30, 80, 160];
const DIRT_RGB: [u8; 3] = [120, 95, 60];
const GRASS_RGB: [u8; 3] = [70, 110, 45];

#[test]
fn shore_waterlines_curve_and_stay_deterministic() {
    let (w, d, g) = (solid(WATER_RGB), solid(DIRT_RGB), solid(GRASS_RGB));

    // Convex corner: land at N, NE, E (mask bits set = water
    // neighbors: SE, S, SW, W, NW).
    let corner_mask = 0b1111_1000u8;
    let out = shore_texture(corner_mask, &w, &d, &g);
    // The NE corner texels are on the land side of the arc:
    // dirt or grass source color, never the water color.
    for (x, y) in [(31, 0), (30, 1), (31, 2), (29, 0)] {
        let p = out.get_pixel(x, y);
        assert!(
            p[0] == DIRT_RGB[0] && p[1] == DIRT_RGB[1] && p[2] == DIRT_RGB[2]
                || p[0] == GRASS_RGB[0] && p[1] == GRASS_RGB[1] && p[2] == GRASS_RGB[2],
            "corner texel ({x},{y}) must be dirt/grass, got {p:?}"
        );
        assert_ne!(
            material_at(corner_mask, true, x, y),
            ShoreMaterial::Water,
            "corner texel ({x},{y}) must not classify as water"
        );
    }
    // Tile center is water.
    assert_eq!(material_at(corner_mask, true, 16, 16), ShoreMaterial::Water);
    assert_eq!(out.get_pixel(16, 16), &Rgba([WATER_RGB[0], WATER_RGB[1], WATER_RGB[2], 255]));

    // Straight edge: land only at N. The waterline hugs the top
    // edge: top texel is dirt, water starts within the top
    // quarter, center is water.
    let straight_mask = 0b1111_1110u8;
    let out = shore_texture(straight_mask, &w, &d, &g);
    assert_eq!(
        material_at(straight_mask, true, 16, 0),
        ShoreMaterial::Dirt,
        "top texel on a straight shore must be dirt bank"
    );
    let first_water = (0..32)
        .find(|&y| material_at(straight_mask, true, 16, y) == ShoreMaterial::Water)
        .expect("water must appear down the center column");
    assert!(
        (6..=18).contains(&first_water),
        "waterline must cross near mid-edge (dirt bank above it), first water at y={first_water}"
    );
    assert_eq!(out.get_pixel(16, 0), &Rgba([DIRT_RGB[0], DIRT_RGB[1], DIRT_RGB[2], 255]));

    // All-water neighborhood: output is exactly the water input.
    let out = shore_texture(0xFF, &w, &d, &g);
    assert_eq!(out.as_raw(), w.as_raw(), "all-water mask must return the water input");

    // Determinism.
    let again = shore_texture(corner_mask, &w, &d, &g);
    let first = shore_texture(corner_mask, &w, &d, &g);
    assert_eq!(first.as_raw(), again.as_raw(), "shore output must be byte-identical");
}

#[test]
fn pond_mosaic_water_region_is_roundish() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let water = gen(&ts, "t_water_dp", RecipeKind::Water).image;
    let dirt = gen(&ts, "t_dirt", RecipeKind::Dirt).image;
    let grass = gen(&ts, "t_grass", RecipeKind::Grass).image;

    // Same composition as examples/stage_e_showcase.rs.
    let is_water = |tx: i32, ty: i32| (1..=3).contains(&tx) && (1..=3).contains(&ty);
    let mut mosaic = RgbaImage::new(5 * TEXTURE_SIZE, 5 * TEXTURE_SIZE);
    for ty in 0..5i32 {
        for tx in 0..5i32 {
            let mask = mask_for(is_water, tx, ty);
            let tile = if is_water(tx, ty) {
                shore_texture(mask, &water, &dirt, &grass)
            } else if mask != 0 {
                bank_texture(mask, &water, &dirt, &grass)
            } else {
                grass.clone()
            };
            image::imageops::overlay(
                &mut mosaic,
                &tile,
                (tx * TEXTURE_SIZE as i32) as i64,
                (ty * TEXTURE_SIZE as i32) as i64,
            );
        }
    }
    assert_eq!(mosaic.dimensions(), (160, 160));

    // Water coverage of the four corner tiles of the 3x3 pond
    // block: a straight square pond would leave them ~100%
    // water; the rounded shoreline must cut them below 60%.
    let mut water_px = 0u32;
    for (tx, ty) in [(1, 1), (3, 1), (1, 3), (3, 3)] {
        let mask = mask_for(is_water, tx, ty);
        for y in 0..TEXTURE_SIZE {
            for x in 0..TEXTURE_SIZE {
                if material_at(mask, true, x, y) == ShoreMaterial::Water {
                    water_px += 1;
                }
            }
        }
    }
    let total = 4 * TEXTURE_SIZE * TEXTURE_SIZE;
    println!(
        "pond mosaic: corner-tile water coverage {water_px}/{total} = {:.1}%",
        100.0 * water_px as f32 / total as f32
    );
    assert!(
        water_px * 5 < total * 3,
        "corner tiles must be < 60% water (round pond), got {water_px}/{total}"
    );
    // And the pond center is fully water.
    let center_mask = mask_for(is_water, 2, 2);
    assert_eq!(center_mask, 0xFF);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            assert_eq!(material_at(center_mask, true, x, y), ShoreMaterial::Water);
        }
    }
}
