//! Stage-G S2 colour gate (docs/vehicle-stage-g.md §4):
//! the tex6 colour rules + the top unification, pinned with
//! synthetic sprites whose colour counts are known exactly.
//!
//! Fixture sprites (32×32):
//! - door:      600×(76,76,76), 300×(100,100,100), 124×(30,30,30)
//! - halfboard: 600×(153,153,153), 424×(120,120,120)
//! - trunk:     1024×(41,41,41)
//! - windshield: 1024×(24,123,180) (chromatic)
//!
//! Derived by the spec's rules: dominants 76 / 153 / 41 /
//! (24,123,180); achromatic PAINT_PARTS dominants sorted by
//! luminance = [41, 76, 153], median = (76,76,76) = PAINT;
//! SIDE_PAINT = mean(76, 153) = (114,114,114); SIDE_TRIM =
//! door's most common non-dominant = (100,100,100).

use image::{Rgba, RgbaImage};
use voxelizer::vehicle::{Colorizer, GLASS};

fn sprite(pixels: &[([u8; 3], u32)]) -> RgbaImage {
    let mut img = RgbaImage::new(32, 32);
    let mut i = 0u32;
    for (c, n) in pixels {
        for _ in 0..*n {
            img.put_pixel(i % 32, i / 32, Rgba([c[0], c[1], c[2], 255]));
            i += 1;
        }
    }
    assert_eq!(i, 32 * 32);
    img
}

fn resolver(id: &str) -> Option<RgbaImage> {
    match id {
        "vp_door" => Some(sprite(&[([76, 76, 76], 600), ([100, 100, 100], 300), ([30, 30, 30], 124)])),
        "vp_halfboard" => Some(sprite(&[([153, 153, 153], 600), ([120, 120, 120], 424)])),
        "vp_trunk" => Some(sprite(&[([41, 41, 41], 1024)])),
        "vp_windshield" => Some(sprite(&[([24, 123, 180], 1024)])),
        _ => None,
    }
}

#[test]
fn vehicle_paint_is_the_median_achromatic_dominant() {
    let cz = Colorizer::new(&resolver);
    assert_eq!(cz.paint(), Some([76, 76, 76]));
}

#[test]
fn panel_tops_unify_onto_the_vehicle_paint() {
    let cz = Colorizer::new(&resolver);
    // Segment 9→26 (a body panel); its top fine row is y=51
    // (depth 0). A halfboard top pixel (160) cleans to its
    // dominant 153, then repaints: 76 + 153 − 153 = 76.
    assert_eq!(cz.voxel_color(9.0, 26.0, [160, 160, 160], "halfboard", 51), [76, 76, 76]);
    // A door top pixel at its own dominant stays 76 — the
    // same surface colour from a different part's sprite.
    assert_eq!(cz.voxel_color(9.0, 26.0, [76, 76, 76], "door", 51), [76, 76, 76]);
    // A door detail pixel keeps its delta from the dominant:
    // 76 + 100 − 76 = 100.
    assert_eq!(cz.voxel_color(9.0, 26.0, [100, 100, 100], "door", 51), [100, 100, 100]);
    // A trunk detail: 76 + 41 − 41 = 76 at the dominant.
    assert_eq!(cz.voxel_color(9.0, 26.0, [41, 41, 41], "trunk", 51), [76, 76, 76]);
}

#[test]
fn chromatic_parts_keep_their_colour() {
    let cz = Colorizer::new(&resolver);
    // Windshield is a PAINT_PART but its dominant is
    // chromatic: no repaint, no paint-median membership.
    assert_eq!(cz.voxel_color(9.0, 26.0, [24, 123, 180], "windshield", 51), [24, 123, 180]);
}

#[test]
fn panel_sides_use_the_unified_side_paint() {
    let cz = Colorizer::new(&resolver);
    // Trim row (depth 1, y=49): SIDE_TRIM.
    assert_eq!(cz.voxel_color(9.0, 26.0, [76, 76, 76], "door", 49), [100, 100, 100]);
    // Mid-depth (depth 5, y=41): SIDE_PAINT under the
    // gradient: span 15, f = 1 − 0.3·(3/15) = 0.94 →
    // round(114·0.94) = 107.
    assert_eq!(cz.voxel_color(9.0, 26.0, [76, 76, 76], "door", 41), [107, 107, 107]);
    // The same depth on a halfboard segment reads the same
    // side paint, not halfboard's own dominant.
    assert_eq!(cz.voxel_color(9.0, 26.0, [153, 153, 153], "halfboard", 41), [107, 107, 107]);
}

#[test]
fn glass_and_wheel_bypass_the_tables() {
    let cz = Colorizer::new(&resolver);
    assert_eq!(cz.voxel_color(20.0, 40.0, [1, 2, 3], "glass", 70), GLASS);
    assert_eq!(cz.voxel_color(0.0, 32.0, [9, 9, 9], "wheel", 3), [9, 9, 9]);
}

#[test]
fn underbody_reads_as_shadow() {
    let cz = Colorizer::new(&resolver);
    // A frame segment (3→11) at y=5: below mockup z 8.5, so
    // whatever colour the rules give is shaded ×0.3. Frame
    // has no sprite here → shade(top, 0.8) then ×0.3:
    // round(round(200·0.8)·0.3) = 48.
    assert_eq!(cz.voxel_color(3.0, 11.0, [200, 200, 200], "frame", 5), [48, 48, 48]);
}
