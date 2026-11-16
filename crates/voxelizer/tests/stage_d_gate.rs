//! Phase 5 rework Stage D gate: quilting + macro ground.
//!
//! For grass and dirt, from the real UltiCa families:
//! * the quilted 32x32 texture meets the SAME plan SS8 targets as
//!   Stage C (seam ratio < 1.15, mean deltaE_OK vs the family
//!   palette < 0.04) — targets are not relaxed for the new path;
//! * the 128x128 macro texture is exactly 128x128, tiles over its
//!   4x4-tile span (seam < 1.15), and is deterministic;
//! * the stored height layer is 128x128 with real variance;
//! * the height-blend reference behaves at its endpoints and a
//!   high-height patch of B punches through A at weight 0.5.
//!
//! Tileset path resolution mirrors stage_c_gate (fork-src lives
//! only in the main tree checkout).

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

const FAMILIES: [(&str, RecipeKind); 2] = [
    ("t_grass", RecipeKind::Grass),
    ("t_dirt", RecipeKind::Dirt),
];

fn height_stats(h: &image::GrayImage) -> (f32, f32) {
    let n = (h.width() * h.height()) as f32;
    let mean = h.pixels().map(|p| p[0] as f32).sum::<f32>() / n;
    let var = h
        .pixels()
        .map(|p| (p[0] as f32 - mean) * (p[0] as f32 - mean))
        .sum::<f32>()
        / n;
    (mean, var.sqrt())
}

#[test]
fn stage_d_quilted_and_macro_meet_targets() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (base, kind) in FAMILIES {
        let entry = ts
            .entries
            .get(base)
            .unwrap_or_else(|| panic!("tileset has {base}"));
        let family = family_sprites(&ts, entry);
        assert!(!family.is_empty(), "{base}: family pool must not be empty");

        // --- Quilted 32x32 ------------------------------------
        let gen = generate_quilted(kind, &family);
        assert_eq!(
            (gen.image.width(), gen.image.height()),
            (TEXTURE_SIZE, TEXTURE_SIZE),
            "{base}: quilted texture must be 32x32"
        );
        let m = metrics::evaluate(&gen.image, &gen.palette);
        println!(
            "{} ({} quilted): seam {:.3}, mean dE {:.4}, max dE {:.4}, palette {}",
            base,
            kind.name(),
            m.seam_ratio,
            m.mean_delta_e,
            m.max_delta_e,
            m.palette_size
        );
        assert!(
            m.seam_ratio < 1.15,
            "{base} ({} quilted): seam ratio {:.3} must be < 1.15",
            kind.name(),
            m.seam_ratio
        );
        assert!(
            m.mean_delta_e < 0.04,
            "{base} ({} quilted): mean deltaE {:.4} must be < 0.04",
            kind.name(),
            m.mean_delta_e
        );
        assert!(m.palette_size <= 24, "{base}: palette {} must be <= 24", m.palette_size);
        let again = generate_quilted(kind, &family);
        assert_eq!(
            gen.image.as_raw(),
            again.image.as_raw(),
            "{base} ({} quilted): two generations must be byte-identical",
            kind.name()
        );
        assert_eq!(gen.palette, again.palette);

        // --- Macro 128x128 + height layer ----------------------
        let mac = generate_macro(kind, &family);
        assert_eq!(
            (mac.image.width(), mac.image.height()),
            (MACRO_SIZE, MACRO_SIZE),
            "{base}: macro must be 128x128"
        );
        assert_eq!(
            (mac.height.width(), mac.height.height()),
            (MACRO_SIZE, MACRO_SIZE),
            "{base}: height layer must be 128x128"
        );
        let macro_seam = metrics::seam_ratio(&mac.image);
        let macro_de = metrics::mean_delta_e(&mac.image, &mac.palette);
        let (h_mean, h_std) = height_stats(&mac.height);
        println!(
            "{} ({} macro): seam {:.3}, mean dE {:.4} (diagnostic), height mean {:.1}, std {:.2}",
            base,
            kind.name(),
            macro_seam,
            macro_de,
            h_mean,
            h_std
        );
        assert!(
            macro_seam < 1.15,
            "{base} ({} macro): seam ratio {:.3} must be < 1.15",
            kind.name(),
            macro_seam
        );
        assert!(
            h_std > 2.0,
            "{base} ({} macro): height layer std {:.2} must show real variance",
            kind.name(),
            h_std
        );
        let mac2 = generate_macro(kind, &family);
        assert_eq!(
            mac.image.as_raw(),
            mac2.image.as_raw(),
            "{base} ({} macro): two generations must be byte-identical",
            kind.name()
        );
        assert_eq!(
            mac.height.as_raw(),
            mac2.height.as_raw(),
            "{base} ({} macro): height layers must be byte-identical",
            kind.name()
        );
    }
}

#[test]
fn height_blend_reference_behavior() {
    use image::{GrayImage, Luma, Rgba, RgbaImage};
    let a = RgbaImage::from_pixel(16, 16, Rgba([210, 60, 50, 255]));
    let b = RgbaImage::from_pixel(16, 16, Rgba([50, 60, 220, 255]));
    let ha = GrayImage::from_pixel(16, 16, Luma([0]));
    let mut hb = GrayImage::from_pixel(16, 16, Luma([0]));
    for y in 4..12 {
        for x in 4..12 {
            hb.put_pixel(x, y, Luma([255]));
        }
    }
    let at0 = height_blend(&a, &ha, &b, &hb, 0.0);
    assert_eq!(at0.as_raw(), a.as_raw(), "weight 0: output must be exactly A");
    let at1 = height_blend(&a, &ha, &b, &hb, 1.0);
    assert_eq!(at1.as_raw(), b.as_raw(), "weight 1: output must be exactly B");
    let mid = height_blend(&a, &ha, &b, &hb, 0.5);
    assert_eq!(
        mid.get_pixel(8, 8),
        &Rgba([50, 60, 220, 255]),
        "weight 0.5: high-height B patch must punch through A"
    );
    assert_eq!(
        mid.get_pixel(0, 0),
        &Rgba([210, 60, 50, 255]),
        "weight 0.5: equal heights outside the patch keep A"
    );
}
