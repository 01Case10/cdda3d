//! Stage D showcase driver: for grass and dirt, generate the
//! Stage C noise texture, the Stage D quilted texture, and the
//! 128x128 macro + height layer from the real UltiCa families;
//! print their SS8 metrics; and write a contact sheet comparing
//! them, plus a height-blend strip (grass over dirt).
//!
//! Usage:
//!   cargo run -p voxelizer --example stage_d_showcase -- \
//!       <tileset_dir> <out_dir>
//!
//! Contact sheet rows (one per material), all panels 128x128:
//!   [Stage C noise tile tiled 4x4 | quilted tile tiled 4x4 |
//!    macro | macro height layer]
//! Below: the height-blend strip — height_blend(dirt, grass) at
//! weights 0 / 0.25 / 0.5 / 0.75 / 1 in five 128px bands. Where
//! grass is higher it punches through dirt before the weight
//! reaches 1; that is the shoreline mechanism Stage E builds on.

use std::path::PathBuf;
use voxelizer::*;

const FAMILIES: [(&str, RecipeKind); 2] = [
    ("t_grass", RecipeKind::Grass),
    ("t_dirt", RecipeKind::Dirt),
];

fn tile4(img: &image::RgbaImage) -> image::RgbaImage {
    let mut out = image::RgbaImage::new(128, 128);
    for dy in 0..4 {
        for dx in 0..4 {
            image::imageops::overlay(&mut out, img, (dx * 32) as i64, (dy * 32) as i64);
        }
    }
    out
}

fn gray_as_rgba(h: &image::GrayImage) -> image::RgbaImage {
    image::ImageBuffer::from_fn(h.width(), h.height(), |x, y| {
        let v = h.get_pixel(x, y)[0];
        image::Rgba([v, v, v, 255])
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: stage_d_showcase <tileset_dir> <out_dir>");
        std::process::exit(2);
    }
    let (tileset_dir, out_dir) = (PathBuf::from(&args[0]), PathBuf::from(&args[1]));
    std::fs::create_dir_all(&out_dir).expect("create out dir");
    let ts = tileset::load_decomposed(&tileset_dir).expect("tileset loads");

    let mut macros: Vec<(RecipeKind, MacroTexture)> = Vec::new();
    let mut rows: Vec<[image::RgbaImage; 4]> = Vec::new();
    for (base, kind) in FAMILIES {
        let entry = ts.entries.get(base).unwrap_or_else(|| panic!("tileset has {base}"));
        let family = family_sprites(&ts, entry);
        assert!(!family.is_empty(), "{base}: empty family");

        let noise = generate(kind, &family);
        let quilted = generate_quilted(kind, &family);
        let mac = generate_macro(kind, &family);
        let qm = metrics::evaluate(&quilted.image, &quilted.palette);
        println!(
            "{base} ({} quilted): seam {:.3}, mean dE {:.4}, max dE {:.4}, palette {}",
            kind.name(),
            qm.seam_ratio,
            qm.mean_delta_e,
            qm.max_delta_e,
            qm.palette_size
        );
        println!(
            "{base} ({} macro): seam {:.3}, mean dE {:.4} (vs family palette)",
            kind.name(),
            metrics::seam_ratio(&mac.image),
            metrics::mean_delta_e(&mac.image, &mac.palette)
        );
        quilted
            .image
            .save(out_dir.join(format!("quilt_{}.png", kind.name())))
            .expect("save quilted tile");
        mac.image
            .save(out_dir.join(format!("macro_{}.png", kind.name())))
            .expect("save macro");
        mac.height
            .save(out_dir.join(format!("height_{}.png", kind.name())))
            .expect("save height layer");
        rows.push([
            tile4(&noise.image),
            tile4(&quilted.image),
            mac.image.clone(),
            gray_as_rgba(&mac.height),
        ]);
        macros.push((kind, mac));
    }

    // --- Height-blend strip: grass over dirt ------------------
    let dirt = &macros.iter().find(|(k, _)| *k == RecipeKind::Dirt).unwrap().1;
    let grass = &macros.iter().find(|(k, _)| *k == RecipeKind::Grass).unwrap().1;
    let mut strip = image::RgbaImage::new(128 * 5, 128);
    for (i, w) in [0.0f32, 0.25, 0.5, 0.75, 1.0].iter().enumerate() {
        let band = height_blend(&dirt.image, &dirt.height, &grass.image, &grass.height, *w);
        image::imageops::overlay(&mut strip, &band, (i * 128) as i64, 0);
    }
    strip.save(out_dir.join("blend_strip.png")).expect("save blend strip");

    // --- Contact sheet ----------------------------------------
    let gap = 6u32;
    let cell = 128u32;
    let sheet_w = cell * 4 + gap * 5;
    let sheet_h = cell * 2 + gap * 3 + 128 + gap;
    let mut sheet = image::RgbaImage::from_pixel(sheet_w, sheet_h, image::Rgba([24, 24, 24, 255]));
    for (row, panels) in rows.iter().enumerate() {
        let y = gap + row as u32 * (cell + gap);
        for (col, panel) in panels.iter().enumerate() {
            let x = gap + col as u32 * (cell + gap);
            image::imageops::overlay(&mut sheet, panel, x as i64, y as i64);
        }
    }
    let strip_y = gap + 2 * (cell + gap);
    // The strip is 640 wide; the sheet is 534 — scale it down to fit.
    let strip_small = image::imageops::resize(
        &strip,
        sheet_w - gap * 2,
        ((128 * (sheet_w - gap * 2)) / strip.width()).max(1),
        image::imageops::FilterType::Nearest,
    );
    image::imageops::overlay(&mut sheet, &strip_small, gap as i64, strip_y as i64);
    let contact_path = out_dir.join("contact.png");
    sheet.save(&contact_path).expect("save contact sheet");
    println!("contact sheet: {}", contact_path.display());
    println!("blend strip: {}", out_dir.join("blend_strip.png").display());
}
