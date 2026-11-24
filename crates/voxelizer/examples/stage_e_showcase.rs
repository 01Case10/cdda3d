//! Phase 5 rework Stage E showcase: compose a 5x5-tile pond
//! mosaic from the generated water/dirt/grass textures — 3x3
//! water tiles ringed by land, shore variants on the water ring,
//! bank variants on the adjacent land — and save it (4x nearest
//! upscale) to `target/stage-e/pond.png`. The roundness of the
//! pond is the visual counterpart of the stage_e_gate assertion.
//!
//! Run: `cargo run -p voxelizer --example stage_e_showcase`

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

fn gen(ts: &tileset::Tileset, base: &str, kind: RecipeKind) -> image::RgbaImage {
    let entry = ts.entries.get(base).unwrap_or_else(|| panic!("tileset has {base}"));
    let family = family_sprites(ts, entry);
    generate(kind, &family).image
}

fn main() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let water = gen(&ts, "t_water_dp", RecipeKind::Water);
    let dirt = gen(&ts, "t_dirt", RecipeKind::Dirt);
    let grass = gen(&ts, "t_grass", RecipeKind::Grass);

    let is_water = |tx: i32, ty: i32| (1..=3).contains(&tx) && (1..=3).contains(&ty);
    let ts32 = TEXTURE_SIZE;
    let mut mosaic = image::RgbaImage::new(5 * ts32, 5 * ts32);
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
                (tx * ts32 as i32) as i64,
                (ty * ts32 as i32) as i64,
            );
        }
    }
    let big = image::imageops::resize(&mosaic, 640, 640, image::imageops::FilterType::Nearest);
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/stage-e");
    std::fs::create_dir_all(&out_dir).expect("create target/stage-e");
    let out = out_dir.join("pond.png");
    big.save(&out).expect("save pond.png");
    println!("pond mosaic: {}", out.display());
}
