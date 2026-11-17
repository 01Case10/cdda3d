//! Contact sheet of source sprites for a list of ids (the
//! rest-of-props phase starts from looking at the sprites).
//! Usage: sprite_contact <out.png> <ids.txt>
use image::{Rgba, RgbaImage};
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
    panic!("UltiCa tileset not found");
}

fn main() {
    let out = std::env::args().nth(1).expect("arg1: out.png");
    let ids_path = std::env::args().nth(2).expect("arg2: ids.txt");
    let ids: Vec<String> = std::fs::read_to_string(&ids_path)
        .expect("read ids")
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let (cell, cols, scale) = (128u32, 6u32, 3u32);
    let rows = (ids.len() as u32 + cols - 1) / cols;
    let mut sheet = RgbaImage::from_pixel(cols * cell, rows * cell, Rgba([24, 24, 28, 255]));
    for (i, id) in ids.iter().enumerate() {
        let Some(entry) = ts.entries.get(id) else { continue };
        let Some(sprite) = composite_entry(&ts, entry) else { continue };
        let big = image::imageops::resize(
            &sprite,
            sprite.width() * scale,
            sprite.height() * scale,
            image::imageops::FilterType::Nearest,
        );
        let (cx, cy) = ((i as u32 % cols) * cell, (i as u32 / cols) * cell);
        let x = cx + (cell - big.width()) / 2;
        let y = cy + cell - big.height() - 4;
        image::imageops::overlay(&mut sheet, &big, x as i64, y as i64);
    }
    sheet.save(&out).expect("save sheet");
    println!("sheet: {} ids -> {}", ids.len(), out);
}
