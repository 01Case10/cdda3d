//! Per-sprite (and per-sprite-family) analysis for the procedural
//! texture pipeline (Phase 5 rework, Stage B):
//!
//! * **Palette extraction** — k-means clustering in OKLab over the
//!   sprite's opaque pixels. Seeding is deterministic: the first
//!   center is chosen by an FNV-1a content hash of the pixel data,
//!   later centers by deterministic farthest-point selection. Same
//!   input -> byte-identical palette, always (project determinism
//!   rule: no wall clock, no RNG device).
//! * **Baked-shading detection** — UltiCa sprites are drawn as 3/4-view
//!   illustrations, so many carry a painted vertical light gradient
//!   (and specular highlights) that must not leak into generated
//!   tileable textures. We measure the top-vs-bottom lightness
//!   gradient and the highlight fraction and report a 0..=1 shading
//!   factor.
//! * **Dominant orientation** — structure tensor over the lightness
//!   gradient field: reports whether the sprite's features run
//!   horizontally, vertically, diagonally, or isotropically, plus the
//!   coherence (how strongly oriented it is). Planks, siding and
//!   brick courses are strongly oriented; grass and dirt are not.
//!
//! A "family" is the set of sprites sharing one material (e.g. all
//! `t_grass*` variants). [`analyze_family`] pools the pixels of every
//! sprite in the family (weighted by pixel count) and runs the same
//! analysis once, which is what the classifier's color fallback and
//! the Stage C recipes consume.

use crate::color::Oklab;
use image::RgbaImage;

/// One palette cluster: centroid color plus its share of the sprite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaletteColor {
    pub lab: Oklab,
    /// The centroid converted back to 8-bit sRGB (for display/swatch).
    pub srgb: [u8; 3],
    /// Fraction of analyzed (opaque) pixels in this cluster, 0..=1.
    pub weight: f32,
    pub count: u32,
}

/// Baked-shading measurements for one sprite/family.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadingInfo {
    /// Mean OKLab L of the bottom quartile of rows minus the top
    /// quartile (negative = darker toward the bottom, the usual
    /// painted-wall gradient).
    pub vertical_gradient: f32,
    /// 0..=1 strength of the vertical gradient: |gradient| mapped
    /// through a 0.35 L-unit saturation. >~0.25 reads as deliberate
    /// baked shading rather than texture noise.
    pub shading_factor: f32,
    /// Fraction of opaque pixels at least 1.5 standard deviations
    /// lighter than the sprite's mean lightness (painted highlights /
    /// specular spots).
    pub highlight_fraction: f32,
}

/// Dominant feature orientation from the structure tensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    /// Coherence too low to call: no dominant direction.
    Isotropic,
    /// Features run left-right (e.g. horizontal planks, brick courses).
    Horizontal,
    /// Features run top-bottom (e.g. vertical siding, tree trunks).
    Vertical,
    /// Features run at ~45 degrees.
    Diagonal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrientationInfo {
    pub kind: Orientation,
    /// Direction the features run, degrees in image space
    /// (0 = horizontal, 90 = vertical), 0..180.
    pub feature_angle_deg: f32,
    /// 0..=1: how strongly the gradient field agrees on one direction.
    pub coherence: f32,
}

/// Full analysis of one sprite or sprite family.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteAnalysis {
    /// Palette sorted by weight (descending); ties broken by
    /// (l, a, b) so the order is fully deterministic.
    pub palette: Vec<PaletteColor>,
    /// Weight-weighted mean color of all analyzed pixels.
    pub mean_color: Oklab,
    /// Fraction of all pixels (incl. transparent) that were opaque
    /// enough to analyze (alpha >= 128).
    pub opaque_fraction: f32,
    pub shading: ShadingInfo,
    pub orientation: OrientationInfo,
    /// Opaque pixels that went into the analysis.
    pub pixel_count: u32,
}

/// FNV-1a 64-bit hash — the project-standard content hash for
/// deterministic seeding (same family as the faces.rs hash noise).
pub fn content_hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn hash_image(img: &RgbaImage) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mix = |v: u32, h: &mut u64| {
        for byte in v.to_le_bytes() {
            *h ^= byte as u64;
            *h = h.wrapping_mul(0x100000001b3);
        }
    };
    mix(img.width(), &mut h);
    mix(img.height(), &mut h);
    content_hash(img.as_raw()) ^ h
}

/// Collect the OKLab colors of opaque pixels (alpha >= 128), in
/// row-major order. Alpha is premultiplied away: a half-transparent
/// pixel contributes its stored RGB (sprites are authored opaque-ish;
/// cutout art uses full alpha).
fn collect_points(images: &[&RgbaImage]) -> Vec<Oklab> {
    let mut pts = Vec::new();
    for img in images {
        for px in img.pixels() {
            if px[3] >= 128 {
                pts.push(Oklab::from_srgb_u8([px[0], px[1], px[2]]));
            }
        }
    }
    pts
}

/// Deterministic k-means in OKLab.
///
/// * Center 0: the point at index `seed % n` (seed = content hash).
/// * Centers 1..k: farthest-point selection (max min-distance; ties
///   go to the lowest index), which spreads centers over the gamut
///   without any randomness.
/// * Lloyd iterations until assignments stop changing (max 64).
/// * An empty cluster is reseeded to the point farthest from its own
///   assigned center (ties: lowest index).
fn kmeans(points: &[Oklab], k: usize, seed: u64) -> Vec<PaletteColor> {
    if points.is_empty() || k == 0 {
        return Vec::new();
    }
    let k = k.min(points.len());
    // Deterministic subsample cap: stride-sample to at most 8192
    // points so a 96x96 sprite family stays fast; the stride is
    // fixed, so the sample is identical on every run.
    let pts: &[Oklab] = points;
    let sampled: Vec<Oklab> = if pts.len() > 8192 {
        let stride = pts.len() as f64 / 8192.0;
        (0..8192)
            .map(|i| pts[(i as f64 * stride) as usize])
            .collect()
    } else {
        pts.to_vec()
    };
    let n = sampled.len();

    let mut centers: Vec<Oklab> = Vec::with_capacity(k);
    centers.push(sampled[(seed % (n as u64)) as usize]);
    while centers.len() < k {
        let mut best = 0usize;
        let mut best_d = -1.0f32;
        for (i, p) in sampled.iter().enumerate() {
            let d = centers
                .iter()
                .map(|c| p.distance(*c))
                .fold(f32::INFINITY, f32::min);
            if d > best_d {
                best_d = d;
                best = i;
            }
        }
        centers.push(sampled[best]);
    }

    let mut assign = vec![0usize; n];
    for _ in 0..64 {
        // Assignment step.
        let mut changed = false;
        for (i, p) in sampled.iter().enumerate() {
            let mut bi = 0usize;
            let mut bd = f32::INFINITY;
            for (ci, c) in centers.iter().enumerate() {
                let d = p.distance(*c);
                if d < bd {
                    bd = d;
                    bi = ci;
                }
            }
            if assign[i] != bi {
                assign[i] = bi;
                changed = true;
            }
        }
        // Update step.
        let mut sums = vec![(0.0f32, 0.0f32, 0.0f32, 0u32); k];
        for (i, p) in sampled.iter().enumerate() {
            let s = &mut sums[assign[i]];
            s.0 += p.l;
            s.1 += p.a;
            s.2 += p.b;
            s.3 += 1;
        }
        for ci in 0..k {
            let s = sums[ci];
            if s.3 > 0 {
                centers[ci] = Oklab {
                    l: s.0 / s.3 as f32,
                    a: s.1 / s.3 as f32,
                    b: s.2 / s.3 as f32,
                };
            } else {
                // Empty cluster: reseed to the point worst served by
                // its current center (deterministic).
                let mut best = 0usize;
                let mut best_d = -1.0f32;
                for (i, p) in sampled.iter().enumerate() {
                    let d = p.distance(centers[assign[i]]);
                    if d > best_d {
                        best_d = d;
                        best = i;
                    }
                }
                centers[ci] = sampled[best];
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut counts = vec![0u32; k];
    for a in &assign {
        counts[*a] += 1;
    }
    let mut out: Vec<PaletteColor> = centers
        .iter()
        .enumerate()
        .map(|(ci, c)| PaletteColor {
            lab: *c,
            srgb: c.to_srgb_u8(),
            weight: counts[ci] as f32 / n as f32,
            count: counts[ci],
        })
        .collect();
    out.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.lab.l.partial_cmp(&b.lab.l).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.lab.a.partial_cmp(&b.lab.a).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.lab.b.partial_cmp(&b.lab.b).unwrap_or(std::cmp::Ordering::Equal))
    });
    out
}

/// Vertical shading profile: per-row mean L over opaque pixels.
fn shading_info(images: &[&RgbaImage]) -> ShadingInfo {
    // Pool rows across the family by relative row position: use the
    // tallest image's height as the row count and map each image's
    // rows onto it by proportion. For a single sprite this is exact.
    let height = images.iter().map(|i| i.height()).max().unwrap_or(0);
    if height == 0 {
        return ShadingInfo {
            vertical_gradient: 0.0,
            shading_factor: 0.0,
            highlight_fraction: 0.0,
        };
    }
    let mut row_sum = vec![0.0f32; height as usize];
    let mut row_n = vec![0u32; height as usize];
    let mut all_l: Vec<f32> = Vec::new();
    for img in images {
        let h = img.height();
        for y in 0..h {
            let ry = if h == height {
                y
            } else {
                (y as u64 * height as u64 / h as u64) as u32
            };
            for x in 0..img.width() {
                let px = img.get_pixel(x, y);
                if px[3] >= 128 {
                    let l = Oklab::from_srgb_u8([px[0], px[1], px[2]]).l;
                    row_sum[ry as usize] += l;
                    row_n[ry as usize] += 1;
                    all_l.push(l);
                }
            }
        }
    }
    let quartile_mean = |rows: &[usize]| -> f32 {
        let mut s = 0.0f32;
        let mut n = 0u32;
        for &r in rows {
            s += row_sum[r];
            n += row_n[r];
        }
        if n > 0 {
            s / n as f32
        } else {
            0.0
        }
    };
    let q = (height / 4).max(1) as usize;
    let top: Vec<usize> = (0..q).collect();
    let bottom: Vec<usize> = (height as usize - q..height as usize).collect();
    let gradient = quartile_mean(&bottom) - quartile_mean(&top);
    let shading_factor = (gradient.abs() / 0.35).clamp(0.0, 1.0);

    let highlight_fraction = if all_l.is_empty() {
        0.0
    } else {
        let mean = all_l.iter().sum::<f32>() / all_l.len() as f32;
        let var = all_l.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / all_l.len() as f32;
        let thresh = mean + 1.5 * var.sqrt();
        all_l.iter().filter(|l| **l > thresh).count() as f32 / all_l.len() as f32
    };
    ShadingInfo {
        vertical_gradient: gradient,
        shading_factor,
        highlight_fraction,
    }
}

/// Structure-tensor orientation over the pooled lightness field.
/// For families, tensors are summed across sprites (each sprite's own
/// gradients), so mixed sizes are fine.
fn orientation_info(images: &[&RgbaImage]) -> OrientationInfo {
    let (mut jxx, mut jxy, mut jyy) = (0.0f64, 0.0f64, 0.0f64);
    for img in images {
        let (w, h) = (img.width(), img.height());
        if w < 3 || h < 3 {
            continue;
        }
        // Lightness grid (transparent pixels borrow the image mean so
        // they don't manufacture edges at the alpha boundary... they
        // still do at cutout silhouettes; acceptable for v1 and noted
        // in docs/sprite-anatomy.md).
        let mut lgrid = vec![0.0f32; (w * h) as usize];
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for y in 0..h {
            for x in 0..w {
                let px = img.get_pixel(x, y);
                if px[3] >= 128 {
                    let l = Oklab::from_srgb_u8([px[0], px[1], px[2]]).l;
                    lgrid[(y * w + x) as usize] = l;
                    sum += l;
                    n += 1;
                }
            }
        }
        if n == 0 {
            continue;
        }
        let mean = sum / n as f32;
        for y in 0..h {
            for x in 0..w {
                let px = img.get_pixel(x, y);
                if px[3] < 128 {
                    lgrid[(y * w + x) as usize] = mean;
                }
            }
        }
        let at = |x: u32, y: u32| lgrid[(y * w + x) as usize] as f64;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                // Sobel gradients.
                let gx = (at(x + 1, y - 1) + 2.0 * at(x + 1, y) + at(x + 1, y + 1))
                    - (at(x - 1, y - 1) + 2.0 * at(x - 1, y) + at(x - 1, y + 1));
                let gy = (at(x - 1, y + 1) + 2.0 * at(x, y + 1) + at(x + 1, y + 1))
                    - (at(x - 1, y - 1) + 2.0 * at(x, y - 1) + at(x + 1, y - 1));
                jxx += gx * gx;
                jxy += gx * gy;
                jyy += gy * gy;
            }
        }
    }
    let trace = jxx + jyy;
    if trace <= 0.0 {
        return OrientationInfo {
            kind: Orientation::Isotropic,
            feature_angle_deg: 0.0,
            coherence: 0.0,
        };
    }
    let disc = ((jxx - jyy) * (jxx - jyy) + 4.0 * jxy * jxy).sqrt();
    let coherence = (disc / trace).clamp(0.0, 1.0) as f32;
    // Gradient direction (direction of fastest change); features run
    // perpendicular to it.
    let grad_deg = (0.5 * (2.0 * jxy).atan2(jxx - jyy)).to_degrees();
    let mut feature = grad_deg + 90.0;
    while feature < 0.0 {
        feature += 180.0;
    }
    while feature >= 180.0 {
        feature -= 180.0;
    }
    let kind = if coherence < 0.15 {
        Orientation::Isotropic
    } else if feature < 22.5 || feature >= 157.5 {
        Orientation::Horizontal
    } else if feature >= 67.5 && feature < 112.5 {
        Orientation::Vertical
    } else {
        Orientation::Diagonal
    };
    OrientationInfo {
        kind,
        feature_angle_deg: feature as f32,
        coherence,
    }
}

/// Analyze a single sprite. `k` = palette size (Stage B default 5).
pub fn analyze_image(img: &RgbaImage, k: usize) -> SpriteAnalysis {
    analyze_family(&[img], k)
}

/// Analyze a sprite family (all variants of one material pooled).
/// Pixel weighting is by pixel count: a 64x64 variant contributes 4x
/// the points of a 32x32 one. Deterministic for identical input.
pub fn analyze_family(images: &[&RgbaImage], k: usize) -> SpriteAnalysis {
    let total_px: u64 = images.iter().map(|i| i.width() as u64 * i.height() as u64).sum();
    let points = collect_points(images);
    let opaque_fraction = if total_px > 0 {
        points.len() as f32 / total_px as f32
    } else {
        0.0
    };
    let mean_color = if points.is_empty() {
        Oklab::BLACK
    } else {
        let weights = vec![1.0f32; points.len()];
        Oklab::weighted_mean(&points, &weights)
    };
    let seed = images.iter().fold(0x243f6a8885a308d3u64, |acc, img| {
        acc ^ hash_image(img)
    });
    SpriteAnalysis {
        palette: kmeans(&points, k, seed),
        mean_color,
        opaque_fraction,
        shading: shading_info(images),
        orientation: orientation_info(images),
        pixel_count: points.len() as u32,
    }
}

/// Convenience for the classifier: the palette-weighted centroid of an
/// analysis (== mean color, but computed from the palette alone so
/// callers holding only a palette can use it too).
pub fn palette_centroid(palette: &[PaletteColor]) -> Option<Oklab> {
    if palette.is_empty() {
        return None;
    }
    let colors: Vec<Oklab> = palette.iter().map(|p| p.lab).collect();
    let weights: Vec<f32> = palette.iter().map(|p| p.weight).collect();
    Some(Oklab::weighted_mean(&colors, &weights))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([rgb[0], rgb[1], rgb[2], 255]))
    }

    fn two_color() -> RgbaImage {
        let mut img = RgbaImage::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let c = if x < 16 { [200, 60, 40, 255] } else { [40, 80, 200, 255] };
                img.put_pixel(x, y, Rgba(c));
            }
        }
        img
    }

    #[test]
    fn palette_two_colors() {
        let a = analyze_image(&two_color(), 2);
        assert_eq!(a.palette.len(), 2);
        let total: f32 = a.palette.iter().map(|p| p.weight).sum();
        assert!((total - 1.0).abs() < 0.01, "weights sum {total}");
        for p in &a.palette {
            assert!((p.weight - 0.5).abs() < 0.02, "half weight {}", p.weight);
        }
        let reds = a.palette.iter().filter(|p| p.srgb[0] > 150).count();
        assert_eq!(reds, 1, "one red cluster: {:?}", a.palette);
    }

    #[test]
    fn palette_solid_color_single_cluster_dominates() {
        let a = analyze_image(&solid(32, 32, [94, 128, 54]), 5);
        assert!(!a.palette.is_empty());
        assert!(a.palette[0].weight > 0.99);
        assert_eq!(a.palette[0].srgb, [94, 128, 54]);
        assert_eq!(a.opaque_fraction, 1.0);
        assert_eq!(a.pixel_count, 1024);
    }

    #[test]
    fn determinism_same_input_same_palette() {
        let img = two_color();
        let a = analyze_image(&img, 5);
        let b = analyze_image(&img, 5);
        assert_eq!(a, b, "analysis must be byte-identical across runs");
        let fam1 = analyze_family(&[&img, &img], 5);
        let fam2 = analyze_family(&[&img, &img], 5);
        assert_eq!(fam1, fam2);
    }

    #[test]
    fn shading_gradient_detected() {
        // Top rows bright, bottom rows dark: classic baked wall shading.
        let mut img = RgbaImage::new(32, 32);
        for y in 0..32 {
            let v = 220 - (y as u32 * 5) as u8;
            for x in 0..32 {
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        let a = analyze_image(&img, 3);
        assert!(a.shading.vertical_gradient < -0.1, "gradient {}", a.shading.vertical_gradient);
        assert!(a.shading.shading_factor > 0.25, "factor {}", a.shading.shading_factor);
        // Flat sprite: no shading.
        let flat = analyze_image(&solid(32, 32, [120, 120, 120]), 3);
        assert!(flat.shading.shading_factor < 0.05, "flat factor {}", flat.shading.shading_factor);
    }

    #[test]
    fn orientation_horizontal_planks() {
        // Horizontal bands -> features run horizontally.
        let mut img = RgbaImage::new(32, 32);
        for y in 0..32 {
            let v = if (y / 4) % 2 == 0 { 200 } else { 80 };
            for x in 0..32 {
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        let a = analyze_image(&img, 2);
        assert_eq!(a.orientation.kind, Orientation::Horizontal, "{:?}", a.orientation);
        assert!(a.orientation.coherence > 0.5);
        // Vertical bands.
        let mut img2 = RgbaImage::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let v = if (x / 4) % 2 == 0 { 200 } else { 80 };
                img2.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        let a2 = analyze_image(&img2, 2);
        assert_eq!(a2.orientation.kind, Orientation::Vertical, "{:?}", a2.orientation);
        // Flat: isotropic.
        let a3 = analyze_image(&solid(32, 32, [90, 90, 90]), 2);
        assert_eq!(a3.orientation.kind, Orientation::Isotropic);
    }

    #[test]
    fn transparent_pixels_ignored() {
        let mut img = RgbaImage::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                let px = if x < 8 { [255, 0, 0, 255] } else { [0, 0, 255, 0] };
                img.put_pixel(x, y, Rgba(px));
            }
        }
        let a = analyze_image(&img, 3);
        assert_eq!(a.opaque_fraction, 0.5);
        assert_eq!(a.pixel_count, 128);
        assert!(a.palette[0].srgb[0] > 200, "red dominates: {:?}", a.palette[0]);
    }
}
