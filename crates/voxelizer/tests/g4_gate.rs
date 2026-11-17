//! Stage G.4 gate: jail + city archetypes — LaundryAppliance
//! (f_washer / f_dryer) and VendingMachine (f_vending_c /
//! f_vending_reinforced / f_vending_o), fitted to the sprite,
//! albedo-only.
//!
//! ALBEDO RULE: the archetypes_gate.rs form, copied verbatim
//! (see that file's header): every model cell either equals a
//! sprite pixel verbatim, or lies within OKLab 0.13 of a
//! declared material / de-shaded palette cluster, and no
//! non-verbatim cell is darker than the darkest non-shadow
//! raw cluster by more than 0.06 in L.
//!
//! Fitted expectations are measured from the sprite IN THE
//! TEST (row modals, void runs) and pinned as literals only
//! where the audit measured them directly (the console gate's
//! convention): laundry control strip 4 rows / plinth 2 rows
//! on both machines, seam rows (14, 16) washer / (13, 14)
//! dryer; vending bay rows 7..9, base 7 rows, bay back wall
//! the slot paint [1, 1, 1].

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

/// The modal color of a pixel stream (test-local, independent
/// of the crate's fitting helpers): most common exact color,
/// ties toward the smallest RGBA key.
fn modal<I: Iterator<Item = Rgba<u8>>>(it: I) -> Option<Rgba<u8>> {
    let mut counts: std::collections::BTreeMap<[u8; 4], u32> = std::collections::BTreeMap::new();
    for px in it {
        *counts.entry(px.0).or_insert(0) += 1;
    }
    let mut best: Option<([u8; 4], u32)> = None;
    for (k, n) in counts {
        if best.map_or(true, |(_, bn)| n > bn) {
            best = Some((k, n));
        }
    }
    best.map(|(k, _)| Rgba(k))
}

fn row_modal(sprite: &RgbaImage, r: u32) -> Option<Rgba<u8>> {
    modal((0..sprite.width()).filter_map(|x| {
        let px = *sprite.get_pixel(x, r);
        if structure(px) {
            Some(px)
        } else {
            None
        }
    }))
}

/// De-shaded palette clusters of a sprite region, re-anchored
/// exactly like the builders (copied from archetypes_gate.rs).
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

/// The albedo rule (file header), checked against one model
/// (copied from archetypes_gate.rs).
fn assert_albedo(sprite: &RgbaImage, seg: &Segmentation, m: &PropModel, declared: &[Rgba<u8>]) {
    let pixels: HashSet<[u8; 4]> = sprite.pixels().map(|p| p.0).collect();
    let mut materials: Vec<Oklab> = Vec::new();
    materials.extend(deshaded_clusters(sprite, seg.silhouette_rows.clone(), 8));
    materials.extend(deshaded_clusters(sprite, seg.front_rows.clone(), 6));
    materials.extend(deshaded_clusters(sprite, seg.top_rows.clone(), 6));
    materials.extend(declared.iter().map(|&c| lab(c)));
    let raw_dark = |rows: std::ops::Range<u32>, k: usize| -> f32 {
        let crop =
            image::imageops::crop_imm(sprite, 0, rows.start, sprite.width(), rows.end - rows.start)
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

/// Flank discipline for a cabinet row band: cells at the row
/// extent's edge columns, strictly between the back and front
/// planes, carry the band's fitted side material for that
/// side (within OKLab 0.03 — the builder writes the modal
/// edge color flat). `skip_top` exempts the model's top row
/// (the laundry top row carries the top un-projection).
fn assert_flanks(
    sprite: &RgbaImage,
    seg: &Segmentation,
    m: &PropModel,
    rows: std::ops::Range<u32>,
    sides: (Rgba<u8>, Rgba<u8>),
    skip_top: bool,
) {
    let d = m.size[2];
    let fh = m.size[1];
    let bot = seg.silhouette_rows.end - 1;
    let mut sampled = 0u32;
    for y in rows {
        if skip_top && y == fh - 1 {
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
        for (x, side) in [(rx0, sides.0), (rx1, sides.1)] {
            for z in 1..d - 1 {
                if let Some(c) = m.get(x, y, z) {
                    let dist = lab(c).distance(lab(side));
                    assert!(
                        dist <= 0.03,
                        "flank {x},{y},{z}: {c:?} vs side {side:?} (d={dist})"
                    );
                    sampled += 1;
                }
            }
        }
    }
    assert!(sampled > 0, "band has flank cells");
}

#[test]
fn g4_mapping_is_explicit() {
    assert_eq!(archetype_for("f_washer"), Some(Archetype::LaundryAppliance));
    assert_eq!(archetype_for("f_dryer"), Some(Archetype::LaundryAppliance));
    assert_eq!(
        archetype_for("f_autoclave_full"),
        Some(Archetype::LaundryAppliance)
    );
    assert_eq!(
        archetype_for("f_vending_c"),
        Some(Archetype::VendingMachine)
    );
    assert_eq!(
        archetype_for("f_vending_reinforced"),
        Some(Archetype::VendingMachine)
    );
    assert_eq!(
        archetype_for("f_vending_o"),
        Some(Archetype::VendingMachine)
    );
    assert_eq!(
        archetype_for("f_vending_c_off"),
        Some(Archetype::VendingMachine)
    );
    assert_eq!(
        archetype_for("f_vending_reinforced_off"),
        Some(Archetype::VendingMachine)
    );
    // Near misses: audited and dropped / not assigned (see the
    // archetypes module docs for the per-candidate evidence).
    assert_eq!(archetype_for("f_toilet"), None);
    assert_eq!(archetype_for("f_shower"), None);
    assert_eq!(archetype_for("f_armchair"), None);
    assert_eq!(archetype_for("f_dresser"), None);
    assert_eq!(archetype_for("f_filing_cabinet"), None);
    assert_eq!(archetype_for("f_cupboard"), None);
    assert_eq!(archetype_for("f_entertainment_center"), None);
    // f_autoclave has its OWN sprite (a different fit, not
    // reviewed) — only f_autoclave_full resolves to f_washer.
    assert_eq!(archetype_for("f_autoclave"), None);
    // The bare id does not exist; the served ids carry suffixes.
    assert_eq!(archetype_for("f_vending"), None);
}

#[test]
fn laundry_bands_fitted_closed_cabinet() {
    // (sprite id, fitted seam rows in model y — measured from
    // the sprite in the audit: washer seam at sprite rows
    // 47..49, dryer at 49..50, front bottom row 63).
    for (id, seam) in [("f_washer", (14u32, 16u32)), ("f_dryer", (13, 14))] {
        let (sprite, seg) = load(id);
        // Independent sprite facts: the front's dominant modal
        // is the door/body enamel; the control strip is the
        // 4 rows above the first dominant row; the plinth is
        // the 2 rows below the last dominant row.
        let front = seg.front_rows.clone();
        let dominant = modal(
            front
                .clone()
                .flat_map(|r| (0..sprite.width()).map(move |x| (x, r)))
                .filter_map(|(x, r)| {
                    let px = *sprite.get_pixel(x, r);
                    if structure(px) {
                        Some(px)
                    } else {
                        None
                    }
                }),
        )
        .expect("front has paint");
        assert_eq!(dominant, Rgba([219, 229, 231, 255]), "{id} dominant");
        for r in front.start..front.start + 4 {
            assert_ne!(
                row_modal(&sprite, r),
                Some(dominant),
                "{id} control row {r}"
            );
        }
        assert_eq!(row_modal(&sprite, front.start + 4), Some(dominant), "{id}");
        for r in front.end - 2..front.end {
            assert_ne!(row_modal(&sprite, r), Some(dominant), "{id} plinth row {r}");
        }
        assert_eq!(row_modal(&sprite, front.end - 3), Some(dominant), "{id}");
        // The seam rows are non-dominant; the rows bounding the
        // seam band (inside the middle region) are dominant.
        let bot = front.end - 1;
        for y in seam.0..=seam.1 {
            assert_ne!(
                row_modal(&sprite, bot - y),
                Some(dominant),
                "{id} seam y={y}"
            );
        }
        assert_eq!(
            row_modal(&sprite, bot - (seam.1 + 1)),
            Some(dominant),
            "{id}"
        );
        assert_eq!(
            row_modal(&sprite, bot - (seam.0 - 1)),
            Some(dominant),
            "{id}"
        );

        let fit = laundry_fit(&sprite, &seg);
        assert_eq!(fit.control_h, 4, "{id} control_h");
        assert_eq!(fit.plinth_h, 2, "{id} plinth_h");
        assert_eq!(fit.seam_rows, seam, "{id} seam_rows");
        assert_eq!(
            fit.depth,
            seg.depth_cells_for(sprite.width()),
            "{id} depth rule"
        );
        assert_eq!(fit.depth, 11, "{id} depth");

        let m = build(Archetype::LaundryAppliance, &sprite, &seg);
        assert_same_model(&m, &build(Archetype::LaundryAppliance, &sprite, &seg));
        // The carrier routes the served id to this archetype.
        let routed = model_for(id, &sprite).expect("model_for routes");
        assert_same_model(&m, &routed);
        let fh = m.size[1];
        let d = m.size[2];
        assert_eq!(fh, front.end - front.start, "{id} height");
        // Closed cabinet: every structure pixel's column is
        // filled at every depth (no hollow gaps), the front
        // plane is the sprite verbatim, the back plane closed.
        for y in 0..fh {
            let r = bot - y;
            for x in 0..sprite.width() {
                let px = *sprite.get_pixel(x, r);
                if !structure(px) {
                    continue;
                }
                assert_eq!(m.get(x, y, d - 1), Some(px), "{id} front verbatim {x},{y}");
                assert!(m.get(x, y, 0).is_some(), "{id} back closed {x},{y}");
                for z in 0..d {
                    assert!(m.get(x, y, z).is_some(), "{id} solid {x},{y},{z}");
                }
            }
        }
        // Per-band, per-side flanks (top row exempt: it carries
        // the top region's un-projection).
        let bands = [
            (fh - fit.control_h..fh, fit.control_side, "control"),
            (
                fit.seam_rows.1 + 1..fh - fit.control_h,
                fit.door_side,
                "door",
            ),
            (fit.seam_rows.0..fit.seam_rows.1 + 1, fit.seam_side, "seam"),
            (fit.plinth_h..fit.seam_rows.0, fit.body_side, "body"),
            (0..fit.plinth_h, fit.plinth_side, "plinth"),
        ];
        for (rows, sides, name) in bands {
            assert!(!rows.is_empty(), "{id} {name} band non-empty");
            assert_flanks(&sprite, &seg, &m, rows, sides, true);
        }
        let declared: Vec<Rgba<u8>> = [
            fit.control_side,
            fit.door_side,
            fit.seam_side,
            fit.body_side,
            fit.plinth_side,
        ]
        .iter()
        .flat_map(|(l, r)| [*l, *r])
        .collect();
        assert_albedo(&sprite, &seg, &m, &declared);
    }
    // The looks_like variant is served with the primary's
    // sprite, exactly as the carrier resolves it.
    let (sprite, seg) = load("f_washer");
    let via_variant = model_for("f_autoclave_full", &sprite).expect("variant routes");
    assert_same_model(
        &build(Archetype::LaundryAppliance, &sprite, &seg),
        &via_variant,
    );
}

/// The test's own void scan: the longest run of void pixels
/// (not structure, or OKLab L < 0.20) in one sprite row, as
/// (start, len) — independent of the crate's bay detector.
fn longest_void_run(sprite: &RgbaImage, r: u32) -> (u32, u32) {
    let (mut best, mut run, mut run_start, mut best_start) = (0u32, 0u32, 0u32, 0u32);
    for x in 0..sprite.width() {
        let px = *sprite.get_pixel(x, r);
        let is_void = !structure(px) || lab(px).l < 0.20;
        if is_void {
            if run == 0 {
                run_start = x;
            }
            run += 1;
            if run > best {
                best = run;
                best_start = run_start;
            }
        } else {
            run = 0;
        }
    }
    (best_start, best)
}

#[test]
fn vending_bay_open_fitted_front_verbatim() {
    for id in ["f_vending_c", "f_vending_reinforced", "f_vending_o"] {
        let (sprite, seg) = load(id);
        // Independent sprite facts (audit-measured, same layout
        // on all three sprites): the dispensing slot is sprite
        // rows 55..56 — a void run of 18 starting at column 5 —
        // and no other row in the bottom half qualifies (the
        // shelf-shadow row 52 runs only 14; the base rows below
        // the slot run 0).
        let (start, len) = longest_void_run(&sprite, 55);
        assert_eq!((start, len), (5, 18), "{id} slot run row 55");
        assert_eq!(
            longest_void_run(&sprite, 56),
            (5, 18),
            "{id} slot run row 56"
        );
        assert!(longest_void_run(&sprite, 54).1 < 16, "{id} row 54 no bay");
        assert!(longest_void_run(&sprite, 52).1 < 16, "{id} row 52 no bay");
        for r in 57..64 {
            assert!(
                longest_void_run(&sprite, r).1 < 16,
                "{id} base row {r} no bay"
            );
        }

        let fit = vending_fit(&sprite, &seg);
        assert_eq!(fit.height, 55, "{id} height = silhouette");
        assert_eq!(
            fit.depth, VENDING_DEPTH,
            "{id} depth (assumed, module docs)"
        );
        assert_eq!(fit.bay.rows, 7..9, "{id} bay rows");
        assert_eq!(fit.bay.cols, 5..23, "{id} bay cols = measured slot run");
        assert_eq!(fit.base_h, 7, "{id} base rows below the bay");
        assert_eq!(
            fit.bay_wall,
            Rgba([1, 1, 1, 255]),
            "{id} bay wall = slot paint"
        );

        let m = build(Archetype::VendingMachine, &sprite, &seg);
        assert_same_model(&m, &build(Archetype::VendingMachine, &sprite, &seg));
        let routed = model_for(id, &sprite).expect("model_for routes");
        assert_same_model(&m, &routed);
        let d = m.size[2];
        let wall_z = d - 1 - VENDING_BAY_DEPTH;
        // The bay is a genuine recess: open from the front
        // plane to the back wall, closed behind it (not a
        // through-hole), back plane intact.
        for y in fit.bay.rows.clone() {
            for x in fit.bay.cols.clone() {
                for z in wall_z + 1..d {
                    assert_eq!(m.get(x, y, z), None, "{id} bay open {x},{y},{z}");
                }
                assert_eq!(
                    m.get(x, y, wall_z),
                    Some(fit.bay_wall),
                    "{id} bay back wall {x},{y}"
                );
                assert!(m.get(x, y, 0).is_some(), "{id} bay not through {x},{y}");
            }
        }
        // Front verbatim everywhere outside the bay; the frame
        // posts flanking the bay and the base below it are
        // solid through the depth.
        let bot = seg.silhouette_rows.end - 1;
        for y in 0..fit.height {
            let r = bot - y;
            for x in 0..sprite.width() {
                let px = *sprite.get_pixel(x, r);
                if !structure(px) {
                    continue;
                }
                if fit.bay.rows.contains(&y) && fit.bay.cols.contains(&x) {
                    continue;
                }
                assert_eq!(m.get(x, y, d - 1), Some(px), "{id} front verbatim {x},{y}");
                assert!(m.get(x, y, 0).is_some(), "{id} back closed {x},{y}");
            }
        }
        // Per-band, per-side flanks at mid-height and in the
        // base (the bay rows' own flanks carry the bay band's
        // material — the frame posts beside the opening).
        assert_flanks(
            &sprite,
            &seg,
            &m,
            fit.bay.rows.end..fit.height,
            fit.upper_side,
            false,
        );
        assert_flanks(&sprite, &seg, &m, fit.bay.rows.clone(), fit.bay_side, false);
        assert_flanks(&sprite, &seg, &m, 0..fit.base_h, fit.base_side, false);
        let declared: Vec<Rgba<u8>> = [fit.upper_side, fit.bay_side, fit.base_side]
            .iter()
            .flat_map(|(l, r)| [*l, *r])
            .chain([fit.bay_wall])
            .collect();
        assert_albedo(&sprite, &seg, &m, &declared);
    }
    // The _off variants are served with the resolved sprite,
    // exactly as the carrier resolves them.
    let (sprite, seg) = load("f_vending_c");
    let via_off = model_for("f_vending_c_off", &sprite).expect("off variant routes");
    assert_same_model(&build(Archetype::VendingMachine, &sprite, &seg), &via_off);
    let (sprite, seg) = load("f_vending_reinforced");
    let via_off = model_for("f_vending_reinforced_off", &sprite).expect("off variant routes");
    assert_same_model(&build(Archetype::VendingMachine, &sprite, &seg), &via_off);
}
