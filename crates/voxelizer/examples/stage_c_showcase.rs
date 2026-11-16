//! Stage C showcase driver: generate the four Stage C materials
//! from the real UltiCa families, print their SS8 metrics, write a
//! contact sheet (source sprite | generated texture tiled 2x2 |
//! palette strip per family), and build a variant voxelizer-output
//! dir whose block top layers for the four materials are the
//! generated textures (for the engine A/B capture).
//!
//! Usage:
//!   cargo run -p voxelizer --example stage_c_showcase -- \
//!       <tileset_dir> <source_assets_dir> <out_dir>
//!
//! The variant dir is a hardlink copy (`cp -rl`) of the source
//! assets; replaced layer files are unlinked before writing so
//! the original assets are never modified through a hardlink.

use std::path::{Path, PathBuf};
use voxelizer::*;

const FAMILIES: [(&str, RecipeKind); 4] = [
    ("t_grass", RecipeKind::Grass),
    ("t_dirt", RecipeKind::Dirt),
    ("t_floor", RecipeKind::WoodFloor),
    ("t_wall", RecipeKind::Plaster),
];

/// Material id -> recipe whose generated texture replaces its
/// top layer in the variant dir. t_floor_primitive is the sample
/// scene's floor id; it takes the t_floor family's texture.
const REPLACEMENTS: [(&str, RecipeKind); 5] = [
    ("t_grass", RecipeKind::Grass),
    ("t_dirt", RecipeKind::Dirt),
    ("t_floor", RecipeKind::WoodFloor),
    ("t_floor_primitive", RecipeKind::WoodFloor),
    ("t_wall", RecipeKind::Plaster),
];

fn palette_strip(palette: &[PaletteColor], w: u32, h: u32) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(w, h);
    let mut y = 0u32;
    for (i, c) in palette.iter().enumerate() {
        let band = if i + 1 == palette.len() {
            h - y
        } else {
            (c.weight * h as f32).round() as u32
        };
        for yy in y..(y + band).min(h) {
            for xx in 0..w {
                img.put_pixel(xx, yy, image::Rgba([c.srgb[0], c.srgb[1], c.srgb[2], 255]));
            }
        }
        y += band;
        if y >= h {
            break;
        }
    }
    img
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: stage_c_showcase <tileset_dir> <source_assets_dir> <out_dir>");
        std::process::exit(2);
    }
    let (tileset_dir, assets_dir, out_dir) = (
        PathBuf::from(&args[0]),
        PathBuf::from(&args[1]),
        PathBuf::from(&args[2]),
    );
    std::fs::create_dir_all(&out_dir).expect("create out dir");
    let ts = tileset::load_decomposed(&tileset_dir).expect("tileset loads");

    let mut generated: Vec<(RecipeKind, GeneratedTexture, image::RgbaImage)> = Vec::new();
    for (base, kind) in FAMILIES {
        let entry = ts.entries.get(base).unwrap_or_else(|| panic!("tileset has {base}"));
        let family = family_sprites(&ts, entry);
        assert!(!family.is_empty(), "{base}: empty family");
        let source = family[0].clone();
        let gen = generate(kind, &family);
        let m = metrics::evaluate(&gen.image, &gen.palette);
        println!(
            "{base} ({}): family {} sprites, shading {:.2}, seam {:.3}, mean dE {:.4}, max dE {:.4}, palette {}",
            kind.name(),
            family.len(),
            gen.analysis.shading.shading_factor,
            m.seam_ratio,
            m.mean_delta_e,
            m.max_delta_e,
            m.palette_size
        );
        gen.image
            .save(out_dir.join(format!("gen_{}.png", kind.name())))
            .expect("save generated texture");
        generated.push((kind, gen, source));
    }

    // --- Contact sheet -------------------------------------------
    let cell = 64u32;
    let gap = 6u32;
    let sheet_w = cell * 3 + gap * 4;
    let sheet_h = cell * 4 + gap * 5;
    let mut sheet = image::RgbaImage::from_pixel(sheet_w, sheet_h, image::Rgba([24, 24, 24, 255]));
    for (row, (_, gen, source)) in generated.iter().enumerate() {
        let y = gap + row as u32 * (cell + gap);
        let src_up = image::imageops::resize(source, cell, cell, image::imageops::FilterType::Nearest);
        image::imageops::overlay(&mut sheet, &src_up, gap as i64, y as i64);
        // Generated texture tiled 2x2 (each tile at native 32 px).
        for dy in 0..2 {
            for dx in 0..2 {
                image::imageops::overlay(
                    &mut sheet,
                    &gen.image,
                    (gap * 2 + cell + dx * 32) as i64,
                    (y + dy * 32) as i64,
                );
            }
        }
        let strip = palette_strip(&gen.palette, cell, cell);
        image::imageops::overlay(&mut sheet, &strip, (gap * 3 + cell * 2) as i64, y as i64);
    }
    let contact_path = out_dir.join("contact.png");
    sheet.save(&contact_path).expect("save contact sheet");
    println!("contact sheet: {}", contact_path.display());

    // --- Variant assets dir --------------------------------------
    let variant = out_dir.join("textures_variant");
    if variant.exists() {
        std::fs::remove_dir_all(&variant).expect("clear old variant dir");
    }
    let status = std::process::Command::new("cp")
        .arg("-rl")
        .arg(&assets_dir)
        .arg(&variant)
        .status()
        .expect("cp -rl");
    assert!(status.success(), "hardlink copy of assets failed");
    let materials: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(assets_dir.join("materials.json")).expect("read materials.json"),
    )
    .expect("parse materials.json");
    let by_kind: std::collections::BTreeMap<&str, &GeneratedTexture> = generated
        .iter()
        .map(|(k, g, _)| (k.name(), g))
        .collect();
    for (id, kind) in REPLACEMENTS {
        let entry = materials
            .as_array()
            .expect("materials is a list")
            .iter()
            .find(|e| e["id"] == id)
            .unwrap_or_else(|| panic!("materials.json has {id}"));
        let top = entry["top_layer"].as_u64().expect("top_layer") as usize;
        let layer_path = variant.join(format!("blocks/layer_{top}.png"));
        // Break the hardlink first: saving in place would write
        // through to the original assets file.
        std::fs::remove_file(&layer_path).expect("unlink layer");
        by_kind[kind.name()]
            .image
            .save(&layer_path)
            .expect("save replaced layer");
        println!("variant: {id} top layer {top} <- generated {}", kind.name());
    }
    println!("variant assets: {}", variant.display());
    let _ = Path::new(&args[0]);
}
