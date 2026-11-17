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
//! (arm a).
//!
//! The G.3 rework (pipeline §24) reduced the archetype set to
//! ShelfUnit (signed off, unchanged) and ConsoleDesk (reworked:
//! front verbatim, knee-hole open). Round 2 gave the console
//! its workstation profile: one common back plane, the
//! terminal front recessed behind the desk ledge, and per-band
//! side materials (terminal casing gray / desk gray / cabinet
//! beige, each the modal color of its region's edge columns).
//! The deleted archetypes' ids (tank, bunk, dissection table)
//! route by sprite VIEW now — views_gate.rs asserts those
//! models. Sprites: f_bookcase, f_console.

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
    // panel primaries.
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
    assert_eq!(archetype_for("f_console"), Some(Archetype::ConsoleDesk));
    // Deleted in the G.3 rework: these ids route by sprite view
    // (props::model_for) — the tank is EndOn, the bed chain and
    // f_slab are TopDown, f_surgical_table has no sprite at all.
    assert_eq!(archetype_for("f_standing_tank"), None);
    assert_eq!(archetype_for("f_storage_tank"), None);
    assert_eq!(archetype_for("f_surgical_table"), None);
    assert_eq!(archetype_for("f_slab"), None);
    assert_eq!(archetype_for("f_bunkbed"), None);
    assert_eq!(archetype_for("f_bunkbed_down"), None);
    assert_eq!(archetype_for("f_triple_bunkbed"), None);
    // Not assigned: the general solid model keeps these.
    assert_eq!(archetype_for("f_table"), None);
    assert_eq!(archetype_for("f_chair"), None);
    assert_eq!(archetype_for("f_counter"), None);
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
fn console_front_verbatim_recessed_terminal_open_base() {
    let (sprite, seg) = load("f_console");
    let fit = console_fit(&sprite, &seg);
    let fh = seg.front_rows.end - seg.front_rows.start;
    let top_h = seg.top_rows.end - seg.top_rows.start;
    assert_eq!(fit.desk_h, fh - top_h, "desk height = front minus rise");
    let m = build(Archetype::ConsoleDesk, &sprite, &seg);
    assert_same_model(&m, &build(Archetype::ConsoleDesk, &sprite, &seg));
    let d = m.size[2];
    assert!(
        fit.recess >= 1 && fit.recess < d,
        "terminal recess fitted: {}",
        fit.recess
    );
    assert!(
        fit.base_h >= 1 && fit.base_h < fit.desk_h,
        "base band inside the desk rows: {}",
        fit.base_h
    );
    // The knee-hole is fitted where the sprite draws it: the
    // bottom 8 model rows, columns 12..28 (dark recess rows over
    // a transparent cutout — measured from the sprite).
    let op = fit.opening.clone().expect("console opening fitted");
    assert_eq!(op.rows, 0..8, "opening rows");
    assert_eq!(op.cols, 12..28, "opening cols");
    // The opening is empty through the whole depth.
    for y in op.rows.clone() {
        for x in op.cols.clone() {
            for z in 0..d {
                assert_eq!(m.get(x, y, z), None, "opening cell {x},{y},{z}");
            }
        }
    }
    // Front faces == the sprite's front region verbatim,
    // exhaustive over structure pixels outside the opening:
    // desk/base rows at the front plane (z = d - 1), terminal
    // rows at the recessed front (z = d - 1 - recess) and EMPTY
    // at the front plane — the recess is real air.
    let bot = seg.front_rows.end - 1;
    let front_z = |y: u32| if y >= fit.desk_h { d - 1 - fit.recess } else { d - 1 };
    for y in 0..fh {
        for x in 0..sprite.width() {
            if op.cols.contains(&x) && op.rows.contains(&y) {
                continue;
            }
            let px = *sprite.get_pixel(x, bot - y);
            if structure(px) {
                assert_eq!(m.get(x, y, front_z(y)), Some(px), "front verbatim {x},{y}");
                if y >= fit.desk_h {
                    assert_eq!(m.get(x, y, d - 1), None, "recess void {x},{y}");
                }
            }
        }
    }
    // Silhouette: ONE common back plane — every filled cell row
    // reaches z = 0 in every band (the round-1 base step notched
    // the rear; it is gone) — and each row's frontmost solid
    // cell sits exactly at its band's fitted front plane.
    for y in 0..fh {
        let mut frontmost: Option<u32> = None;
        for x in 0..sprite.width() {
            if op.cols.contains(&x) && op.rows.contains(&y) {
                continue;
            }
            if !structure(*sprite.get_pixel(x, bot - y)) {
                continue;
            }
            assert!(m.get(x, y, 0).is_some(), "back plane {x},{y}");
            frontmost = frontmost.max((0..d).rev().find(|&z| m.get(x, y, z).is_some()));
        }
        assert_eq!(frontmost, Some(front_z(y)), "row {y} front plane");
    }
    // Side band colors: flank cells strictly between the back
    // and front planes carry the band's fitted side material —
    // asserted within OKLab 0.03 (the builder writes the modal
    // edge color flat, so this is near-exact; the tolerance
    // only covers the comparison, not a material spread). The
    // top row is exempt: it carries the un-projected top
    // surface, not flank material.
    let bands = [
        (0..fit.base_h, fit.base_side, "base"),
        (fit.base_h..fit.desk_h, fit.desk_side, "desk"),
        (fit.desk_h..fh, fit.term_side, "terminal"),
    ];
    for (rows, side, name) in bands {
        let mut sampled = 0u32;
        for y in rows {
            if y == fh - 1 {
                continue;
            }
            let r = bot - y;
            let (mut rx0, mut rx1) = (sprite.width(), 0u32);
            let mut any = false;
            for x in 0..sprite.width() {
                if structure(*sprite.get_pixel(x, r)) {
                    rx0 = rx0.min(x);
                    rx1 = rx1.max(x);
                    any = true;
                }
            }
            if !any || rx0 == rx1 {
                continue;
            }
            for x in [rx0, rx1] {
                for z in 1..front_z(y) {
                    if let Some(c) = m.get(x, y, z) {
                        let dist = lab(c).distance(lab(side));
                        assert!(
                            dist <= 0.03,
                            "{name} flank {x},{y},{z}: {c:?} vs side {side:?} (d={dist})"
                        );
                        sampled += 1;
                    }
                }
            }
        }
        assert!(sampled > 0, "{name} band has flank cells");
    }
    assert_albedo(&sprite, &seg, &m, &[]);
}

// ---------------------------------------------------------------------
// HollowCylinder / ClosedCylinder (coverage fix round, pipeline
// §32): open barrels get a real cavity with a contents surface;
// the keg gets a flat fitted lid.
// ---------------------------------------------------------------------

/// The cylinder ids' sprites do not segment, so `load` (which
/// requires a segmentation) cannot serve them; load raw and use
/// the seamless segmentation where a seg is needed.
fn load_raw(sprite_id: &str) -> RgbaImage {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts
        .entries
        .get(sprite_id)
        .unwrap_or_else(|| panic!("tileset has {sprite_id}"));
    composite_entry(&ts, entry).expect("sprite composites")
}

#[test]
fn hollow_cylinder_has_a_real_cavity() {
    let sprite = load_raw("f_30gal_firebarrel");
    let m = model_for("f_30gal_firebarrel", &sprite).expect("barrel models");
    assert_same_model(
        &m,
        &model_for("f_30gal_firebarrel", &sprite).expect("rebuild"),
    );
    let (fh, d) = (m.size[1], m.size[2]);
    assert_eq!(d, 16, "diameter = silhouette width");
    // Extent 8..=23 (measured), axis at x=15/16, z=7/8.
    // Cavity: the axis just under the rim is EMPTY (the first
    // treatment filled it with a stepped cap of rim paint).
    assert!(m.get(15, fh - 2, 7).is_none(), "cavity under the rim");
    assert!(m.get(16, fh - 2, 8).is_none(), "cavity under the rim");
    // Contents surface: interior_depth = round(16 * 0.55) = 9,
    // so the surface is at y = fh - 9 and reads dark (the empty
    // barrel's drawn interior).
    let s = m.get(15, fh - 9, 7).expect("contents surface");
    assert!(lab(s).l < 0.35, "dark interior surface: {:?}", s.0);
    // Shell front is the sprite verbatim at body rows.
    let seg = seamless_segment(&sprite).expect("seamless seg");
    let bot = seg.front_rows.end - 1;
    for y in [4u32, 10, 15] {
        let px = *sprite.get_pixel(15, bot - y);
        let front = (0..d)
            .rev()
            .find(|&z| m.get(15, y, z).is_some())
            .map(|z| m.get(15, y, z).expect("found"));
        assert_eq!(front, Some(px), "front verbatim at y={y}");
    }
    // The burning variant's surface is fire paint, not dark.
    let fire = load_raw("f_30gal_firebarrel_fd_fire");
    let fm = model_for("f_30gal_firebarrel_fd_fire", &fire).expect("fire barrel models");
    let fs = fm.get(15, fm.size[1] - 9, 7).expect("fire surface");
    assert!(
        fs[0] > 100 && fs[0] > fs[1] * 2,
        "contents surface is fire paint: {:?}",
        fs.0
    );
}

#[test]
fn closed_cylinder_has_a_flat_fitted_lid() {
    let sprite = load_raw("f_wood_keg");
    let m = model_for("f_wood_keg", &sprite).expect("keg models");
    assert_same_model(&m, &model_for("f_wood_keg", &sprite).expect("rebuild"));
    let (w, fh, d) = (m.size[0], m.size[1], m.size[2]);
    // The whole top face is level: every disc column reaches fh.
    let mut lid_colors = std::collections::HashSet::new();
    let (cx, cz) = (15u32, d / 2);
    assert!(m.get(cx, fh - 1, cz).is_some(), "lid at the axis");
    for x in 0..w {
        for z in 0..d {
            if let Some(c) = m.get(x, fh - 1, z) {
                let dx = (x as f32 - 15.0) / 9.5;
                let dz = (z as f32 - (d - 1) as f32 / 2.0) / (d as f32 / 2.0);
                if dx * dx + dz * dz <= 0.36 {
                    lid_colors.insert(c.0);
                }
            }
        }
    }
    assert_eq!(
        lid_colors.len(),
        1,
        "interior lid is one fitted material: {lid_colors:?}"
    );
}
