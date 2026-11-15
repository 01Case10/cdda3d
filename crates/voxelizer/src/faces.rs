//! Face generation rules (plan §8): top = fg composited OVER bg;
//! walls use the wall sprite directly as side; floors/ground sample
//! bottom edge rows, darkened 25–40% with subtle deterministic noise;
//! bottom = darkened side. Alpha-aware mip downsampling.

use image::{Rgba, RgbaImage};
use tileset::{TileEntry, Tileset};

/// Alpha-over composite `fg` over `bg`. The two sprites may differ in
/// size: both are anchored by their sheet offsets onto a canvas of the
/// larger size (offset = where the sprite's top-left sits relative to
/// the tile's top-left, so canvas origin shifts by -min(offset)).
pub fn composite_over(bg: &RgbaImage, fg: &RgbaImage) -> RgbaImage {
    let w = bg.width().max(fg.width());
    let h = bg.height().max(fg.height());
    let mut out = RgbaImage::new(w, h);
    for y in 0..bg.height() {
        for x in 0..bg.width() {
            out.put_pixel(x, y, bg.get_pixel(x, y).clone());
        }
    }
    for y in 0..fg.height() {
        for x in 0..fg.width() {
            let f = fg.get_pixel(x, y);
            let fa = f[3] as f32 / 255.0;
            if fa <= 0.0 {
                continue;
            }
            let b = out.get_pixel(x, y);
            let ba = b[3] as f32 / 255.0;
            let oa = fa + ba * (1.0 - fa);
            let mut px = [0u8; 4];
            for c in 0..3 {
                let v = if oa > 0.0 {
                    (f[c] as f32 * fa + b[c] as f32 * ba * (1.0 - fa)) / oa
                } else {
                    0.0
                };
                px[c] = v.round().clamp(0.0, 255.0) as u8;
            }
            px[3] = (oa * 255.0).round() as u8;
            out.put_pixel(x, y, Rgba(px));
        }
    }
    out
}

/// Crop/anchor a sprite to a tile-sized face. Sprites larger than the
/// tile are anchored by their sheet offset: the tile window's origin
/// inside the sprite is (-offset_x, -offset_y), clamped into bounds —
/// the sprite is NOT centered and NOT scaled (plan §8 geometry rule).
/// Smaller sprites are placed at their offset on a transparent canvas
/// of tile size (then the caller's bg shows through via compositing
/// order — faces are generated from the already-composited sprite).
pub fn make_top_face(img: &RgbaImage, ts: &Tileset, entry: &TileEntry, tile: u32) -> RgbaImage {
    let (ox, oy) = entry_offset(ts, entry);
    crop_anchored(img, ox, oy, tile)
}

pub fn entry_offset(ts: &Tileset, entry: &TileEntry) -> (i32, i32) {
    if let Some(r) = entry.fg.as_ref().or(entry.bg.as_ref()) {
        if let Some(g) = ts.geom_for_entry_ref(entry, r) {
            return (g.offset_x, g.offset_y);
        }
    }
    (0, 0)
}

pub fn crop_anchored(img: &RgbaImage, offset_x: i32, offset_y: i32, tile: u32) -> RgbaImage {
    let mut out = RgbaImage::new(tile, tile);
    let origin_x = (-offset_x).clamp(0, img.width().saturating_sub(1) as i32);
    let origin_y = (-offset_y).clamp(0, img.height().saturating_sub(1) as i32);
    for y in 0..tile {
        for x in 0..tile {
            let sx = origin_x + x as i32;
            let sy = origin_y + y as i32;
            if sx >= 0 && sy >= 0 && (sx as u32) < img.width() && (sy as u32) < img.height() {
                out.put_pixel(x, y, img.get_pixel(sx as u32, sy as u32).clone());
            }
        }
    }
    out
}

pub fn darken(img: &RgbaImage, factor: f32) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        for c in 0..3 {
            p[c] = (p[c] as f32 * factor).round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

/// Deterministic hash noise in [-1, 1] for (x, y, seed).
pub fn hash_noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = seed
        .wrapping_mul(0x9E3779B1)
        .wrapping_add(x.wrapping_mul(0x85EBCA6B))
        .wrapping_add(y.wrapping_mul(0xC2B2AE35));
    h ^= h >> 13;
    h = h.wrapping_mul(0x27D4EB2F);
    h ^= h >> 15;
    (h % 2000) as f32 / 1000.0 - 1.0
}

/// Side face per plan §8 rule 2. Darkening for floors is 35%
/// (factor 0.65, inside the 25–40% band) plus ±3% deterministic noise.
pub const FLOOR_SIDE_DARKEN: f32 = 0.65;

pub fn make_side_face(top: &RgbaImage, wall_like: bool, seed: u32) -> RgbaImage {
    if wall_like {
        return top.clone();
    }
    let (w, h) = (top.width(), top.height());
    let rows = 4.min(h);
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        let src_y = h - rows + (y * rows / h.max(1));
        for x in 0..w {
            let p = top.get_pixel(x, src_y.min(h - 1));
            let noise = 1.0 + hash_noise(x, y, seed) * 0.03;
            let f = FLOOR_SIDE_DARKEN * noise;
            out.put_pixel(
                x,
                y,
                Rgba([
                    (p[0] as f32 * f).round().clamp(0.0, 255.0) as u8,
                    (p[1] as f32 * f).round().clamp(0.0, 255.0) as u8,
                    (p[2] as f32 * f).round().clamp(0.0, 255.0) as u8,
                    p[3],
                ]),
            );
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlphaInfo {
    pub transparent_fraction: f32,
    pub semi_fraction: f32,
    pub has_holes: bool,
}

pub fn analyze_alpha(img: &RgbaImage) -> AlphaInfo {
    let total = (img.width() * img.height()).max(1) as f32;
    let mut transparent = 0u32;
    let mut semi = 0u32;
    for p in img.pixels() {
        if p[3] == 0 {
            transparent += 1;
        } else if p[3] < 250 {
            semi += 1;
        }
    }
    AlphaInfo {
        transparent_fraction: transparent as f32 / total,
        semi_fraction: semi as f32 / total,
        // "significant alpha holes": some fully transparent pixels but
        // the sprite is mostly opaque (fences, foliage).
        has_holes: transparent > 0 && (transparent as f32 / total) < 0.9,
    }
}

/// Alpha flags per plan §8 rule 6.
pub fn alpha_flags(id: &str, alpha: &AlphaInfo, ov: Option<&crate::OverrideEntry>) -> (bool, bool) {
    if let Some(o) = ov {
        if let Some(t) = &o.transparency {
            return match t.as_str() {
                "cutout" => (true, false),
                "transparent" => (false, true),
                _ => (false, false),
            };
        }
    }
    let glass_or_water = id.contains("glass") || id.contains("water");
    let transparent = glass_or_water || alpha.semi_fraction > 0.25;
    let cutout = !transparent && alpha.has_holes && alpha.transparent_fraction > 0.02;
    (cutout, transparent)
}

/// Alpha-aware mip downsample by 2: premultiplied averaging so fully
/// transparent pixels contribute no color (no edge bleed), with the
/// result un-premultiplied. Edge texels clamp (repeat the edge pixel).
pub fn downsample2x(img: &RgbaImage) -> RgbaImage {
    let w = (img.width() / 2).max(1);
    let h = (img.height() / 2).max(1);
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0f32; 4];
            let mut count = 0f32;
            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = (x * 2 + dx).min(img.width() - 1);
                    let sy = (y * 2 + dy).min(img.height() - 1);
                    let p = img.get_pixel(sx, sy);
                    let a = p[3] as f32 / 255.0;
                    sum[0] += p[0] as f32 * a;
                    sum[1] += p[1] as f32 * a;
                    sum[2] += p[2] as f32 * a;
                    sum[3] += a;
                    count += 1.0;
                }
            }
            let mean_a = sum[3] / count;
            let mut px = [0u8; 4];
            for c in 0..3 {
                let premul_mean = sum[c] / count;
                px[c] = if mean_a > 0.001 {
                    (premul_mean / mean_a).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                };
            }
            px[3] = (mean_a * 255.0).round() as u8;
            out.put_pixel(x, y, Rgba(px));
        }
    }
    out
}

/// Full mip chain down to 1x1.
pub fn mip_chain(img: &RgbaImage) -> Vec<RgbaImage> {
    let mut chain = vec![img.clone()];
    while chain
        .last()
        .map(|i| i.width() > 1 || i.height() > 1)
        .unwrap_or(false)
    {
        let next = downsample2x(chain.last().unwrap());
        chain.push(next);
    }
    chain
}
