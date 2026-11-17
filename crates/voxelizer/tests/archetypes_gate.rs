//! Stage G.3 gate: parametric archetypes — real constructed
//! geometry fitted to the sprite, albedo-only.
//!
//! ALBEDO RULE asserted here (the G.3 form of the props_gate
//! discipline): every model cell either
//!   (a) equals a sprite pixel verbatim (front decal / top
//!       un-projection — evidence, may be any paint incl. shadow), or
//!   (b) lies within OKLab 0.13 of a DECLARED material: the
//!       archetype fit's colors, or a de-shaded palette cluster of
//!       the whole sprite / front region / top region (k = 8/6/6,
//!       re-anchored to the palette's weighted-median L exactly as
//!       the builders do; 0.13 covers panel grain, <= +/-9% L).
//! AND no cell is darker than the darkest non-shadow RAW
//! cluster of those palettes by more than 0.06 in L unless it is
//! verbatim paint (raw, because de-shading lifts dark clusters
//! toward the median by design — the floor guards against
//! INVENTED darkness, not against the de-shade itself)
//! (arm a). Glass and screen accents are palette clusters by
//! construction, so they pass through arm (b) — there are no
//! other exceptions.
//!
//! Structural assertions are per archetype (compartments open,
//! liquid below the line, screens in accent, rim above channel,
//! bunk gap empty). Sprites: f_bookcase, f_standing_tank,
//! f_console, f_slab (fitted for the spriteless f_surgical_table),
//! f_bed (the sprite f_bunkbed resolves to).

use image::{Rgba, RgbaImage};
use std::collections::HashSet;
use std::path::PathBuf;
use voxelizer::color::Oklab;
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

fn load(sprite_id: &str) -> (RgbaImage, Segmentation) {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts
        .entries
        .get(sprite_id)
        .unwrap_or_else(|| panic!("tileset has {sprite_id}"));
    let sprite = composite_entry(&ts, entry).expect("sprite composites");
    let seg = segment_for(sprite_id, &sprite).expect("sprite segments");
    (sprite, seg)
}

fn structure(px: Rgba<u8>) -> bool {
    px[3] >= 8 && !(px[0] == 0 && px[1] == 0 && px[2] == 0 && px[3] < 254)
}

fn lab(px: Rgba<u8>) -> Oklab {
    Oklab::from_srgb_u8([px[0], px[1], px[2]])
}

fn assert_same_model(a: &PropModel, b: &PropModel) {
    assert_eq!(a.size, b.size);
    assert_eq!(a.solid_count(), b.solid_count());
    let [w, h, d] = a.size;
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                assert_eq!(a.get(x, y, z), b.get(x, y, z), "cell {x},{y},{z}");
            }
        }
    }
}

/// De-shaded palette clusters of a sprite region, re-anchored
/// exactly like the builders (weighted-median L over non-shadow
/// clusters, gain clamped [0.80, 1.25]).
fn deshaded_clusters(sprite: &RgbaImage, rows: std::ops::Range<u32>, k: usize) -> Vec<Oklab> {
    let crop =
        image::imageops::crop_imm(sprite, 0, rows.start, sprite.width(), rows.end - rows.start)
            .to_image();
    let a = analyze_image(&crop, k);
    let mut pairs: Vec<(f32, f32)> = a
        .palette
        .iter()
        .filter(|p| p.lab.l >= 0.15)
        .map(|p| (p.lab.l, p.weight))
        .collect();
    if pairs.is_empty() {
        return a.palette.iter().map(|p| p.lab).collect();
    }
    pairs.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
    let total: f32 = pairs.iter().map(|p| p.1).sum();
    let mut acc = 0.0f32;
    let mut median = pairs[pairs.len() - 1].0;
    for (l, w) in &pairs {
        acc += w;
        if acc >= total / 2.0 {
            median = *l;
            break;
        }
    }
    a.palette
        .iter()
        .map(|p| {
            if p.lab.l > 0.0 {
                let g = (median / p.lab.l).clamp(0.80, 1.25);
                Oklab {
                    l: p.lab.l * g,
                    a: p.lab.a,
                    b: p.lab.b,
                }
            } else {
                p.lab
            }
        })
        .collect()
}

/// The albedo rule (file header), checked against one model.
fn assert_albedo(sprite: &RgbaImage, seg: &Segmentation, m: &PropModel, declared: &[Rgba<u8>]) {
    let pixels: HashSet<[u8; 4]> = sprite.pixels().map(|p| p.0).collect();
    let mut materials: Vec<Oklab> = Vec::new();
    materials.extend(deshaded_clusters(sprite, seg.silhouette_rows.clone(), 8));
    materials.extend(deshaded_clusters(sprite, seg.front_rows.clone(), 6));
    materials.extend(deshaded_clusters(sprite, seg.top_rows.clone(), 6));
    materials.extend(declared.iter().map(|&c| lab(c)));
    // The darkness floor is the darkest non-shadow RAW cluster
    // of any palette the builders draw from (whole sprite,
    // front, top) — raw, per the header: de-shading lifts dark
    // clusters toward the median (gain up to 1.25), so a floor
    // taken from de-shaded clusters would flag the builders' own
    // panel primaries (f_slab's front primary lands at L 0.521
    // while its de-shaded darkest cluster sits at 0.588).
    let raw_dark = |rows: std::ops::Range<u32>, k: usize| -> f32 {
        let crop = image::imageops::crop_imm(
            sprite,
            0,
            rows.start,
            sprite.width(),
            rows.end - rows.start,
        )
        .to_image();
        analyze_image(&crop, k)
            .palette
            .iter()
            .filter(|p| p.lab.l >= 0.15)
            .map(|p| p.lab.l)
            .fold(f32::MAX, f32::min)
    };
    let darkest_ns = raw_dark(seg.silhouette_rows.clone(), 8)
        .min(raw_dark(seg.front_rows.clone(), 6))
        .min(raw_dark(seg.top_rows.clone(), 6));
    let [w, h, d] = m.size;
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                let Some(c) = m.get(x, y, z) else { continue };
                if pixels.contains(&c.0) {
                    continue; // verbatim evidence paint
                }
                let cl = lab(c);
                let nearest = materials
                    .iter()
                    .map(|mt| cl.distance(*mt))
                    .fold(f32::MAX, f32::min);
                assert!(
                    nearest <= 0.13,
                    "cell {x},{y},{z} color {c:?} is {nearest:.3} from every declared material"
                );
                assert!(
                    cl.l >= darkest_ns - 0.06,
                    "cell {x},{y},{z} color {c:?} darker than the darkest material"
                );
            }
        }
    }
}

#[test]
fn archetype_mapping_is_explicit() {
    assert_eq!(archetype_for("f_bookcase"), Some(Archetype::ShelfUnit));
    assert_eq!(archetype_for("f_rack"), Some(Archetype::ShelfUnit));
    assert_eq!(
        archetype_for("f_standing_tank"),
        Some(Archetype::ContainmentTank)
    );
    assert_eq!(
        archetype_for("f_storage_tank"),
        Some(Archetype::ContainmentTank)
    );
    assert_eq!(archetype_for("f_console"), Some(Archetype::ConsoleDesk));
    assert_eq!(
        archetype_for("f_surgical_table"),
        Some(Archetype::DissectionTable)
    );
    assert_eq!(
        archetype_for("f_bunkbed"),
        Some(Archetype::Bunk { levels: 2 })
    );
    assert_eq!(
        archetype_for("f_triple_bunkbed"),
        Some(Archetype::Bunk { levels: 3 })
    );
    // Not assigned: the general solid model keeps these.
    assert_eq!(archetype_for("f_table"), None);
    assert_eq!(archetype_for("f_chair"), None);
    assert_eq!(archetype_for("f_counter"), None);
    // Deliberately NOT assigned: the one true containment sprite
    // does not segment (pod on a stalk — archetypes.rs docs).
    assert_eq!(archetype_for("f_alien_pod"), None);
}

#[test]
fn shelf_unit_opens_real_compartments() {
    let (sprite, seg) = load("f_bookcase");
    let fit = shelf_fit(&sprite, &seg);
    // Five painted recess bands + the fitted base board.
    assert_eq!(fit.board_rows.len(), 6, "boards: {:?}", fit.board_rows);
    assert_eq!(fit.board_rows[0], (0, 1));
    let m = build(Archetype::ShelfUnit, &sprite, &seg);
    assert_same_model(&m, &build(Archetype::ShelfUnit, &sprite, &seg));
    let d = m.size[2];
    // Compartments are OPEN: front plane and interior empty
    // between boards (rows 2..5 above the base, 8..12 mid-case).
    for y in [3u32, 4, 10, 11] {
        assert_eq!(m.get(16, y, d - 1), None, "front of compartment y={y}");
        assert_eq!(m.get(16, y, d / 2), None, "interior y={y}");
    }
    // The back panel closes the compartments.
    assert!(m.get(16, 10, 0).is_some());
    assert!(m.get(16, 4, 0).is_some());
    // Side panels are solid through a compartment's height.
    assert!(m.get(0, 10, d / 2).is_some());
    assert!(m.get(31, 10, d / 2).is_some());
    // Board front edges carry the sprite decal verbatim.
    let bot = seg.front_rows.end - 1;
    for &(b, _) in &fit.board_rows {
        let expect = *sprite.get_pixel(16, bot - b);
        assert!(structure(expect), "board row {b} has paint at x=16");
        assert_eq!(m.get(16, b, d - 1), Some(expect), "board {b} decal");
    }
    assert_albedo(&sprite, &seg, &m, &[]);
}

#[test]
fn containment_tank_glass_and_liquid() {
    let (sprite, seg) = load("f_standing_tank");
    let fit = tank_fit(&sprite, &seg);
    let m = build(Archetype::ContainmentTank, &sprite, &seg);
    assert_same_model(&m, &build(Archetype::ContainmentTank, &sprite, &seg));
    let (fh, d) = (m.size[1], m.size[2]);
    // Glass on the front plane, inside the frame.
    assert_eq!(m.get(16, fh / 2, d - 1), Some(fit.glass));
    // Liquid below the fitted line, air above it (interior).
    assert!(fit.liquid_top >= 2 && fit.liquid_top + 2 < fh);
    assert_eq!(m.get(16, fit.liquid_top - 1, d / 2), Some(fit.liquid));
    assert_eq!(m.get(16, fit.liquid_top + 2, d / 2), None);
    // Corner post is frame material (extent starts at x=1).
    assert_eq!(m.get(1, fh / 2, d - 1), Some(fit.frame));
    assert_albedo(&sprite, &seg, &m, &[fit.frame, fit.glass, fit.liquid]);
}

#[test]
fn console_desk_screens_and_decal() {
    let (sprite, seg) = load("f_console");
    let fit = console_fit(&sprite, &seg);
    assert!(!fit.screens.is_empty());
    let m = build(Archetype::ConsoleDesk, &sprite, &seg);
    assert_same_model(&m, &build(Archetype::ConsoleDesk, &sprite, &seg));
    let d = m.size[2];
    let console_depth = (d / 2).clamp(3, d - 2);
    // Every screen rect is filled with the accent color on the
    // console's front face.
    for s in &fit.screens {
        for y in s[1]..=s[3] {
            for x in s[0]..=s[2] {
                assert_eq!(m.get(x, y, console_depth - 1), Some(fit.accent));
            }
        }
    }
    // Keyboard slab present on the desk in front of the screens.
    let kx = (fit.screens[0][0] + fit.screens[0][2]) / 2;
    assert!(m.get(kx, fit.desk_h, console_depth + 2).is_some());
    // The L profile: air above the desk, in front of the console.
    assert_eq!(m.get(16, fit.desk_h + 3, d - 2), None);
    // Desk body front = the sprite decal, exhaustive over the
    // body's structure pixels.
    let bot = seg.front_rows.end - 1;
    for y in 0..fit.desk_h {
        for x in 0..sprite.width() {
            let px = *sprite.get_pixel(x, bot - y);
            if structure(px) {
                assert_eq!(m.get(x, y, d - 1), Some(px), "body decal {x},{y}");
            }
        }
    }
    assert_albedo(&sprite, &seg, &m, &[fit.accent]);
}

#[test]
fn dissection_table_rim_above_channel() {
    // f_slab is the fitted sprite for f_surgical_table (no
    // UltiCa sprite of its own — archetypes.rs docs).
    let (sprite, seg) = load("f_slab");
    let fit = table_fit(&sprite, &seg);
    let m = build(Archetype::DissectionTable, &sprite, &seg);
    assert_same_model(&m, &build(Archetype::DissectionTable, &sprite, &seg));
    let d = m.size[2];
    assert_eq!(fit.rim_y, m.size[1] - 1);
    assert_eq!(fit.channel_y, fit.rim_y - 1);
    // Rim stands at the top row (border); the channel center is
    // open at the rim row and floored one row below.
    assert!(m.get(16, fit.rim_y, 1).is_some(), "rim cell (back edge)");
    assert!(m.get(2, fit.rim_y, d / 2).is_some(), "rim cell (left edge)");
    assert_eq!(m.get(16, fit.rim_y, d / 2), None, "channel open");
    assert!(m.get(16, fit.channel_y, d / 2).is_some(), "channel floor");
    // Pedestal under the slab center; corners underneath empty.
    assert!(m.get(16, 4, d / 2).is_some(), "pedestal");
    assert_eq!(m.get(2, 4, 2), None, "no leg at the corner");
    assert_albedo(&sprite, &seg, &m, &[]);
}

#[test]
fn bunk_slabs_posts_and_ladder() {
    // f_bed is the sprite f_bunkbed resolves to (looks_like).
    let (sprite, seg) = load("f_bed");
    let arch = archetype_for("f_bunkbed").expect("bunkbed assigned");
    let fit = bunk_fit(&sprite, &seg, 2);
    assert_eq!(fit.slab_tops.len(), 2);
    let m = build(arch, &sprite, &seg);
    assert_same_model(&m, &build(arch, &sprite, &seg));
    let d = m.size[2];
    // Mattress tops on both slabs (interior cells).
    for &top in &fit.slab_tops {
        assert_eq!(m.get(16, top, d / 2), Some(fit.mattress));
    }
    // Between the slabs: open except posts and ladder.
    let mid = (fit.slab_tops[0] + fit.slab_tops[1]) / 2;
    assert_eq!(m.get(16, mid, d / 2), None, "bunk gap is open");
    assert_eq!(m.get(2, mid, 0), Some(fit.frame_color), "corner post");
    // Ladder on the right end (x = 29, the extent's right edge):
    // a rail mid-gap and a rung (rows with y % 3 == 1).
    assert_eq!(m.get(29, mid, 3), Some(fit.frame_color), "ladder rail");
    let rung_y = (0..m.size[1]).find(|y| y % 3 == 1 && m.get(29, *y, 5).is_some());
    assert!(rung_y.is_some(), "a ladder rung exists");
    // Three levels when the id says three.
    let fit3 = bunk_fit(&sprite, &seg, 3);
    assert_eq!(fit3.slab_tops.len(), 3);
    let m3 = build(Archetype::Bunk { levels: 3 }, &sprite, &seg);
    for &top in &fit3.slab_tops {
        assert_eq!(m3.get(16, top, d / 2), Some(fit3.mattress));
    }
    assert_albedo(&sprite, &seg, &m, &[fit.mattress, fit.frame_color]);
}
