//! Decoration sprites (Procedural Texture Pipeline Stage F part 2,
//! pipeline doc §10.2): a small set of alpha sprites — grass tuft,
//! low shrub, flowering variant — generated from the grass family's
//! own pixels, for the client-side decoration scatter implemented
//! in `billboard::decor`.
//!
//! Derivation, following the Stage C/D family conventions: the
//! caller pools the family with [`crate::generate::family_sprites`]
//! (base entry + center/unconnected subtiles, transition art
//! excluded), the family's palette comes from Stage B's
//! [`crate::analyze::analyze_family`] (deterministic OKLab k-means),
//! and each sprite is drawn procedurally from that palette with a
//! [`crate::noise::Rng`] seeded by the family's content hash
//! ([`crate::generate::family_seed`]) plus the variant name
//! ([`crate::noise::seed_from_str`]). Same family in ->
//! byte-identical sprites out, always (project determinism rule).
//!
//! Sprites are 32x32 RGBA with a transparent background — the real
//! pixel size of a ground tile in UltiCa — and are packed with
//! [`decoration_atlas`] into the existing billboard atlas format
//! (one packed PNG + a [`SpriteUv`] list, the same shape the full
//! voxelizer run writes as `sprites.png` / `sprites.json`).
//! Decorations are NOT materials: nothing here touches
//! `materials.json` emission.

use image::{Rgba, RgbaImage};

use crate::analyze::analyze_family;
use crate::generate::family_seed;
use crate::noise::{seed_from_str, Rng};
use crate::SpriteUv;

/// Decoration sprite edge length in pixels (one ground tile).
pub const DECOR_SIZE: u32 = 32;

/// The decoration variant names, in variant-index order. The
/// billboard crate's `decor::DECOR_SPRITE_NAMES` carries the same
/// strings for the runtime side; the Stage F2 gate asserts the two
/// lists agree.
pub const DECOR_NAMES: [&str; 3] = ["decor_tuft", "decor_shrub", "decor_flowering"];

/// Palette roles picked out of the grass family's analysis.
struct DecorPalette {
    dark: [u8; 3],
    mid: [u8; 3],
    light: [u8; 3],
    /// Flower accent: the family's warmest color (max r+g-b), the
    /// closest thing grass pixels carry to a flower tint.
    accent: [u8; 3],
}

fn decor_palette(family: &[RgbaImage]) -> Option<DecorPalette> {
    let refs: Vec<&RgbaImage> = family.iter().collect();
    let analysis = analyze_family(&refs, 8);
    if analysis.palette.is_empty() {
        return None;
    }
    // Order by OKLab lightness; weights break ties so the order is
    // total and deterministic.
    let mut by_l: Vec<[u8; 3]> = analysis.palette.iter().map(|p| p.srgb).collect();
    let labs: Vec<f32> = analysis.palette.iter().map(|p| p.lab.l).collect();
    let mut idx: Vec<usize> = (0..by_l.len()).collect();
    idx.sort_by(|&a, &b| {
        labs[a]
            .partial_cmp(&labs[b])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(by_l[a].cmp(&by_l[b]))
    });
    by_l = idx.iter().map(|&i| by_l[i]).collect();
    let dark = by_l[0];
    let light = *by_l.last().unwrap();
    let mid = by_l[by_l.len() / 2];
    let accent = *by_l
        .iter()
        .max_by_key(|c| c[0] as i32 + c[1] as i32 - 2 * (c[2] as i32))
        .unwrap();
    Some(DecorPalette {
        dark,
        mid,
        light,
        accent,
    })
}

fn put(img: &mut RgbaImage, x: i32, y: i32, c: [u8; 3]) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, Rgba([c[0], c[1], c[2], 255]));
    }
}

/// One grass blade: a 1px column rising `h` px from the baseline,
/// leaning `lean` px over its height, optionally 2px wide at the
/// base.
fn blade(img: &mut RgbaImage, base_x: i32, base_y: i32, h: i32, lean: i32, wide: bool, c: [u8; 3]) {
    for i in 0..h {
        let x = base_x + lean * i / h.max(1);
        let y = base_y - i;
        put(img, x, y, c);
        if wide && i < h / 3 {
            put(img, x + 1, y, c);
        }
    }
}

/// A filled, noise-eroded ellipse blob (the shrub body): pixels
/// inside the ellipse whose per-pixel white noise clears the edge
/// threshold, colored in horizontal bands (dark base, mid body,
/// light top) with noise dithering between bands.
fn shrub_body(img: &mut RgbaImage, seed: u64, cx: f32, cy: f32, rx: f32, ry: f32, pal: &DecorPalette) {
    let mut rng = Rng::new(seed ^ 0x5EED);
    // Pre-roll a dither table so pixel colors do not depend on
    // scan order.
    let mut dither = [[0f32; 32]; 32];
    for row in dither.iter_mut() {
        for v in row.iter_mut() {
            *v = rng.next_f32();
        }
    }
    for y in 0..DECOR_SIZE as i32 {
        for x in 0..DECOR_SIZE as i32 {
            let dx = (x as f32 - cx) / rx;
            let dy = (y as f32 - cy) / ry;
            let d = dx * dx + dy * dy;
            let edge_noise = crate::noise::white_noise(seed, x as u32, y as u32);
            if d > 1.0 || (d > 0.72 && edge_noise < 0.45) {
                continue;
            }
            let t = (cy + ry - y as f32) / (2.0 * ry); // 0 bottom, 1 top
            let pick = t + (dither[y as usize][x as usize] - 0.5) * 0.25;
            let c = if pick > 0.72 {
                pal.light
            } else if pick > 0.38 {
                pal.mid
            } else {
                pal.dark
            };
            put(img, x, y, c);
        }
    }
}

/// Generate the three decoration sprites from a pooled grass
/// family (see [`crate::generate::family_sprites`]). Returns
/// `(name, image)` pairs in [`DECOR_NAMES`] order, or an empty
/// vec when the family carries no analyzable pixels — the caller
/// then simply has no decorations, never a placeholder sprite.
pub fn decoration_sprites(family: &[RgbaImage]) -> Vec<(String, RgbaImage)> {
    let Some(pal) = decor_palette(family) else {
        return Vec::new();
    };
    let base_seed = family_seed(family);
    let mut out = Vec::with_capacity(3);

    // -- Tuft: a fan of blades from a common root zone. ----------
    {
        let seed = base_seed ^ seed_from_str(DECOR_NAMES[0]);
        let mut rng = Rng::new(seed);
        let mut img = RgbaImage::new(DECOR_SIZE, DECOR_SIZE);
        let colors = [pal.mid, pal.dark, pal.light, pal.mid];
        for i in 0..16 {
            let base_x = 5 + rng.below(22) as i32;
            let h = 10 + rng.below(14) as i32;
            let lean = rng.below(5) as i32 - 2;
            let wide = rng.below(3) == 0;
            blade(&mut img, base_x, 31, h, lean, wide, colors[i % colors.len()]);
        }
        out.push((DECOR_NAMES[0].to_string(), img));
    }

    // -- Shrub: a low, wide blob with a few blades poking out. ---
    {
        let seed = base_seed ^ seed_from_str(DECOR_NAMES[1]);
        let mut rng = Rng::new(seed);
        let mut img = RgbaImage::new(DECOR_SIZE, DECOR_SIZE);
        shrub_body(&mut img, seed, 16.0, 22.0, 11.0, 8.0, &pal);
        for _ in 0..5 {
            let base_x = 8 + rng.below(16) as i32;
            let h = 16 + rng.below(8) as i32;
            blade(&mut img, base_x, 29, h, rng.below(3) as i32 - 1, false, pal.mid);
        }
        out.push((DECOR_NAMES[1].to_string(), img));
    }

    // -- Flowering: a smaller shrub + tuft with accent dots. -----
    {
        let seed = base_seed ^ seed_from_str(DECOR_NAMES[2]);
        let mut rng = Rng::new(seed);
        let mut img = RgbaImage::new(DECOR_SIZE, DECOR_SIZE);
        shrub_body(&mut img, seed, 16.0, 23.0, 9.0, 6.5, &pal);
        for _ in 0..7 {
            let base_x = 6 + rng.below(20) as i32;
            let h = 12 + rng.below(10) as i32;
            blade(&mut img, base_x, 31, h, rng.below(5) as i32 - 2, false, pal.mid);
        }
        // Flowers: a 2x2 accent block with a light center pixel,
        // seated on the shrub's upper body.
        for _ in 0..6 {
            let fx = 9 + rng.below(14) as i32;
            let fy = 15 + rng.below(8) as i32;
            put(&mut img, fx, fy, pal.accent);
            put(&mut img, fx + 1, fy, pal.accent);
            put(&mut img, fx, fy + 1, pal.accent);
            put(&mut img, fx + 1, fy + 1, pal.light);
        }
        out.push((DECOR_NAMES[2].to_string(), img));
    }

    out
}

/// Pack decoration sprites into the billboard atlas format: one
/// image plus the [`SpriteUv`] list that addresses it — the same
/// pair shape as the voxelizer's `sprites.png` / `sprites.json`.
/// Shelf packing in input order (already the deterministic
/// [`DECOR_NAMES`] order), one row, real pixel sizes.
pub fn decoration_atlas(sprites: &[(String, RgbaImage)]) -> (RgbaImage, Vec<SpriteUv>) {
    let width: u32 = sprites.iter().map(|(_, i)| i.width()).sum::<u32>().max(1);
    let height: u32 = sprites.iter().map(|(_, i)| i.height()).max().unwrap_or(1);
    let mut atlas = RgbaImage::new(width, height);
    let mut uvs = Vec::with_capacity(sprites.len());
    let mut x = 0u32;
    for (name, img) in sprites {
        image::imageops::replace(&mut atlas, img, x as i64, 0);
        uvs.push(SpriteUv {
            name: name.clone(),
            x,
            y: 0,
            width: img.width(),
            height: img.height(),
            offset_x: 0,
            offset_y: 0,
            sheet: "decor.png".to_string(),
        });
        x += img.width();
    }
    (atlas, uvs)
}
