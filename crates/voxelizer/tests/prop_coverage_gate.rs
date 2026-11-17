//! Stage G coverage-drive gate: base prop models for the
//! sprites segment() declines (props::coverage_model).
//!
//! Three claims, in order:
//! 1. NO REGRESSION: the 119 furniture ids that already emitted
//!    a v2 model before the drive still emit one with exactly
//!    the same solid-cell count (the parent's audit TSV,
//!    embedded as REGRESSION below). The drive's code paths
//!    fire only where segment_for() returns None.
//! 2. COVERAGE: the full furniture universe now yields exactly
//!    COVERAGE_TOTAL models (119 before the drive; measured by
//!    the parent's audit example after it).
//! 3. PER-CLASS STRUCTURE: the seamless solid keeps the signed-
//!    off standard (front verbatim, back closed, albedo rule);
//!    plan slabs are low with the sprite as the top face; panel
//!    slabs are thin with front verbatim; named vegetation /
//!    skeleton / exclusion ids still decline.

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

fn load_sprite(id: &str) -> RgbaImage {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts.entries.get(id).unwrap_or_else(|| panic!("tileset has {id}"));
    composite_entry(&ts, entry).unwrap_or_else(|| panic!("{id} composites"))
}

fn structure(px: Rgba<u8>) -> bool {
    px[3] >= 254 && !(px[0] == 0 && px[1] == 0 && px[2] == 0 && px[3] < 254)
}

fn lab(px: Rgba<u8>) -> Oklab {
    Oklab::from_srgb_u8([px[0], px[1], px[2]])
}

fn assert_same_model(a: &PropModel, b: &PropModel) {
    assert_eq!(a.size, b.size);
    assert_eq!(a.voxels, b.voxels);
}

/// The 119 pre-drive models: (id, solid-cell count) from the
/// parent's coverage audit (target/prop_coverage.tsv, before).
const REGRESSION: &[(&str, usize)] = &[
    ("f_CTscan", 10464),
    ("f_LIXA_machinery_electronic", 22848),
    ("f_MRI", 44022),
    ("f_active_backup_generator", 9828),
    ("f_air_conditioner", 16260),
    ("f_alien_table", 14528),
    ("f_arcade_machine", 11648),
    ("f_archery_target_bale", 6380),
    ("f_autoclave", 4624),
    ("f_autodoc_couch", 15488),
    ("f_bandsaw", 6360),
    ("f_bathtub", 5040),
    ("f_bed", 6031),
    ("f_bookcase", 7392),
    ("f_bullettrailer_bed_foot", 1920),
    ("f_bullettrailer_cabinet", 10816),
    ("f_bullettrailer_desk", 9408),
    ("f_bullettrailer_minisofa", 4608),
    ("f_bullettrailer_sign_danabakery", 3712),
    ("f_bullettrailer_sink", 12480),
    ("f_bullettrailer_toilet", 14336),
    ("f_capacitor", 32896),
    ("f_cardboard_box", 9800),
    ("f_compact_ASRG_containment", 14756),
    ("f_console", 17784),
    ("f_console_broken", 12148),
    ("f_counter", 11392),
    ("f_crate_c", 19456),
    ("f_crate_o", 15488),
    ("f_cupboard", 11392),
    ("f_curtain", 9350),
    ("f_curtain_open", 5511),
    ("f_desk", 5148),
    ("f_dishwasher", 13500),
    ("f_displaycase", 4368),
    ("f_displaycase_b", 4888),
    ("f_displaycase_o", 4368),
    ("f_dresser", 19456),
    ("f_dryer", 12320),
    ("f_dumpster", 14208),
    ("f_earthbag_half", 16128),
    ("f_entertainment_center", 14448),
    ("f_exodii_charger", 21100),
    ("f_exodii_charger_cheap", 19900),
    ("f_exodii_generator_3", 32758),
    ("f_exodii_printer_large", 2992),
    ("f_exodii_printer_small", 2640),
    ("f_exodii_pump", 8514),
    ("f_fake_bench_hands", 2048),
    ("f_fireman_cabinet", 6292),
    ("f_fireman_cabinet_b", 6567),
    ("f_fireplace", 13376),
    ("f_foot_locker", 5184),
    ("f_freezer", 12960),
    ("f_fridge", 18656),
    ("f_generator_broken", 10944),
    ("f_glass_cabinet", 25344),
    ("f_glass_fridge", 25948),
    ("f_gunsafe_o", 31746),
    ("f_hay", 6380),
    ("f_hd_compressor_unit", 7799),
    ("f_home_furnace", 22496),
    ("f_intermodal_crate_open", 9240),
    ("f_jointer", 5082),
    ("f_locker", 12672),
    ("f_machinery_electronic", 11040),
    ("f_machinery_heavy", 8184),
    ("f_machinery_old", 14080),
    ("f_magic_bench", 20480),
    ("f_magiclysm_translocator_gate", 51678),
    ("f_makeshift_bed", 7392),
    ("f_metal_crate_c", 7590),
    ("f_metal_crate_o", 7590),
    ("f_metal_crate_r", 7766),
    ("f_mitresaw", 2160),
    ("f_oven", 12320),
    ("f_pallet_belted", 13312),
    ("f_pallet_belted_open", 6144),
    ("f_pallet_cement", 14912),
    ("f_piano", 6216),
    ("f_pillow_fort", 7264),
    ("f_planter", 8910),
    ("f_planter_harvest", 11520),
    ("f_planter_mature", 11520),
    ("f_planter_seedling", 8910),
    ("f_pool_table", 7936),
    ("f_rack", 5760),
    ("f_rack_wood", 14448),
    ("f_rubble", 2896),
    ("f_rubble_landfill", 16320),
    ("f_rubble_rock", 2896),
    ("f_safe_o", 2244),
    ("f_sandbag_half", 8932),
    ("f_sandbag_wall", 4675),
    ("f_screenmirror", 7326),
    ("f_sealed_intermodal_crate", 9570),
    ("f_shower", 20064),
    ("f_sink", 12416),
    ("f_slab", 5472),
    ("f_sofa", 5874),
    ("f_spinwheel", 28512),
    ("f_spinwheel_cordage", 28512),
    ("f_spinwheel_leather", 28512),
    ("f_standing_tank", 23224),
    ("f_straw_bed", 7392),
    ("f_table", 2944),
    ("f_tablesaw", 9688),
    ("f_tourist_table", 2144),
    ("f_treadmill", 13446),
    ("f_vending_c", 27872),
    ("f_vending_o", 27872),
    ("f_vending_reinforced", 27872),
    ("f_vent_pipe", 3712),
    ("f_wardrobe", 27648),
    ("f_washer", 12320),
    ("f_water_mill", 7650),
    ("f_water_purifier", 5060),
    ("f_woodchips", 2896),
    ("f_wreckage", 4100)
];

/// Models over the full furniture universe after the drive
/// (audit example, parent-measured). 233 after rest-of-
/// props batch 1 (2026-11-17: 11 ambiguous-class ids —
/// 2 slabs, 2 egg piles, 2 lathe pods, 2 depth solids,
/// 2 thin extrusions, the birdbath).
const COVERAGE_TOTAL: usize = 296;

#[test]
fn pre_drive_models_unchanged() {
    assert_eq!(REGRESSION.len(), 119);
    for &(id, cells) in REGRESSION {
        let sprite = load_sprite(id);
        let m = model_for(id, &sprite)
            .unwrap_or_else(|| panic!("{id} modeled before the drive, declines now"));
        assert_eq!(m.solid_count(), cells, "{id} cell count changed");
    }
}

#[test]
fn coverage_total_is_exact() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let mut ids: Vec<&String> = ts.entries.keys().filter(|k| k.starts_with("f_")).collect();
    ids.sort();
    assert_eq!(ids.len(), 351);
    let mut n = 0usize;
    for id in ids {
        let entry = ts.entries.get(id).unwrap();
        if let Some(sprite) = composite_entry(&ts, entry) {
            if model_for(id, &sprite).is_some() {
                n += 1;
            }
        }
    }
    assert_eq!(n, COVERAGE_TOTAL, "coverage total drifted");
}

// --- albedo rule (copied from archetypes_gate.rs) --------------
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
                Oklab { l: p.lab.l * g, a: p.lab.a, b: p.lab.b }
            } else {
                p.lab
            }
        })
        .collect()
}

fn assert_albedo(sprite: &RgbaImage, seg: &Segmentation, m: &PropModel) {
    let pixels: HashSet<[u8; 4]> = sprite.pixels().map(|p| p.0).collect();
    let mut materials: Vec<Oklab> = Vec::new();
    materials.extend(deshaded_clusters(sprite, seg.silhouette_rows.clone(), 8));
    materials.extend(deshaded_clusters(sprite, seg.front_rows.clone(), 6));
    materials.extend(deshaded_clusters(sprite, seg.top_rows.clone(), 6));
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
                    continue;
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
fn seamless_solid_front_verbatim_back_closed() {
    // f_toilet: the G.4 archetype drop stands (no tank/bowl
    // split is fitted); the drive treats it as a base silhouette
    // solid. Premise: segment() declines it, seamless fires.
    let sprite = load_sprite("f_toilet");
    assert!(segment(&sprite).is_none(), "premise: segment declines");
    let seg = seamless_segment(&sprite).expect("seamless segment fires");
    let m = model_for("f_toilet", &sprite).expect("toilet models");
    assert_same_model(&m, &model_for("f_toilet", &sprite).expect("rebuild"));
    assert_eq!(m.size, [32, 28, 11], "toilet size (w, band, depth floor)");
    let d = m.size[2];
    let fh = seg.front_rows.end - seg.front_rows.start;
    assert_eq!(m.size[1], fh);
    let bot = seg.front_rows.end - 1;
    for y in 0..fh {
        for x in 0..32 {
            let px = *sprite.get_pixel(x, bot - y);
            if structure(px) {
                assert_eq!(m.get(x, y, d - 1), Some(px), "front verbatim {x},{y}");
                assert!(m.get(x, y, 0).is_some(), "back closed {x},{y}");
            } else {
                assert_eq!(m.get(x, y, d - 1), None, "front hole {x},{y}");
            }
        }
    }
    assert_albedo(&sprite, &seg, &m);
}

#[test]
fn seamless_top_is_one_fitted_material() {
    // Joey's review (2026-11-17): coloring the seamless top from
    // the sprite's single top edge row streaked every top face.
    // The top (each column's topmost solid voxel, z < d-1) must
    // now be ONE fitted material per model.
    for id in ["f_chest", "f_toilet", "f_workbench"] {
        let sprite = load_sprite(id);
        let m = model_for(id, &sprite).expect("models");
        let (w, fh, d) = (m.size[0], m.size[1], m.size[2]);
        let mut tops = std::collections::HashSet::new();
        for x in 0..w {
            let ytop = (0..fh)
                .rev()
                .find(|&y| (0..d).any(|z| m.get(x, y, z).is_some()));
            if let Some(ytop) = ytop {
                for z in 0..d - 1 {
                    if let Some(c) = m.get(x, ytop, z) {
                        tops.insert(c.0);
                    }
                }
            }
        }
        assert_eq!(
            tops.len(),
            1,
            "{id}: top face is one fitted material, got {tops:?}"
        );
    }
}

#[test]
fn cylinder_footprint_is_round() {
    // Joey's review (2026-11-17): round-plan objects (barrels,
    // vats, heaters) came out square. A pinned cylinder's plan
    // is the inscribed disc: full depth at the center column,
    // shallow at the silhouette edges, and the front-most voxel
    // of every column is still the sprite pixel verbatim.
    let sprite = load_sprite("f_30gal_firebarrel");
    let seg = seamless_segment(&sprite).expect("seamless fires");
    let m = model_for("f_30gal_firebarrel", &sprite).expect("barrel models");
    assert_same_model(&m, &model_for("f_30gal_firebarrel", &sprite).expect("rebuild"));
    let d = m.size[2];
    let bot = seg.front_rows.end - 1;
    let (mut x0, mut x1) = (32u32, 0u32);
    for y in seg.front_rows.clone() {
        for x in 0..32 {
            if structure(*sprite.get_pixel(x, y)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    assert_eq!(d, (x1 - x0 + 1).min(32), "depth = silhouette diameter");
    let y = m.size[1] / 2;
    let depth_at = |x: u32| (0..d).filter(|&z| m.get(x, y, z).is_some()).count();
    let cx = (x0 + x1) / 2;
    assert!(
        depth_at(cx) >= (d - 2) as usize,
        "center column near-full depth: {} of {d}",
        depth_at(cx)
    );
    assert!(
        depth_at(x0) * 3 <= depth_at(cx) * 2 && depth_at(x0) < depth_at(cx),
        "edge column shallow (disc, not block): {} vs center {}",
        depth_at(x0),
        depth_at(cx)
    );
    // Front verbatim at WALL rows. Above them (round 4: the
    // model is full height) the front is the rim section —
    // shell painted with the wall's topmost row, the rim roll
    // — never the ellipse band's interior paint extruded.
    let fit = voxelizer::archetypes::cylinder_fit(&sprite, &seg).expect("fit");
    for yy in 0..m.size[1] {
        for x in x0..=x1 {
            let px = *sprite.get_pixel(x, bot - yy);
            if structure(px) {
                let front = (0..d)
                    .rev()
                    .find(|&z| m.get(x, yy, z).is_some())
                    .map(|z| m.get(x, yy, z).expect("found"));
                if yy < fit.fh && px[0] == 0 && px[1] == 0 && px[2] == 0 {
                    // De-void (round 4): a pure-black sprite
                    // pixel is outline, not paint — the front
                    // takes the column's nearest non-black wall
                    // paint instead of a void black.
                    let wall: Vec<image::Rgba<u8>> = (0..fit.fh)
                        .map(|wy| *sprite.get_pixel(x, bot - wy))
                        .filter(|p| structure(*p) && !(p[0] == 0 && p[1] == 0 && p[2] == 0))
                        .collect();
                    assert!(
                        front.is_some_and(|c| wall.contains(&c)),
                        "de-voided front comes from the column's wall paint {x},{yy}: {front:?}"
                    );
                } else if yy < fit.fh {
                    assert_eq!(front, Some(px), "front-most is sprite paint {x},{yy}");
                } else {
                    let rim = *sprite.get_pixel(x, bot - (fit.fh - 1));
                    assert_eq!(
                        front,
                        Some(rim),
                        "rim section is wall paint, not band paint {x},{yy}"
                    );
                }
            }
        }
    }
}

#[test]
fn lathe_front_matches_back_and_sides_follow_profile() {
    // Joey's review (2026-11-17) on f_alien_gasper: an irregular
    // round shape must not get a flat fitted back or flat
    // striped sides. The lathe's front-most AND back-most voxel
    // of every column are both the sprite pixel, and the depth
    // profile follows the sprite's width profile.
    let sprite = load_sprite("f_alien_gasper");
    let seg = seamless_segment(&sprite).expect("seamless fires");
    let m = model_for("f_alien_gasper", &sprite).expect("models");
    assert_same_model(&m, &model_for("f_alien_gasper", &sprite).expect("rebuild"));
    let (fh, d) = (m.size[1], m.size[2]);
    let bot = seg.front_rows.end - 1;
    let mut widest = (0u32, 0u32, 0u32);
    let mut narrowest = (u32::MAX, 0u32, 0u32);
    for y in 0..fh {
        let r = bot - y;
        let xs: Vec<u32> = (0..32)
            .filter(|&x| structure(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            continue;
        }
        let (a, b) = (xs[0], xs[xs.len() - 1]);
        // The widest BODY row (the flared mouth rows at the top
        // are rings, not discs — depth there is the wall, by
        // design of the hollow top).
        if y < 3 * fh / 4 && b - a + 1 > widest.0 {
            widest = (b - a + 1, y, (a + b) / 2);
        }
        if y > fh / 4 && y < 3 * fh / 4 && b - a + 1 < narrowest.0 {
            narrowest = (b - a + 1, y, (a + b) / 2);
        }
        // Mouth rows (rings: the disc center is hollow) are
        // the exception to verbatim: their ring takes the
        // row's rim paint — the sprite's dark center pixels
        // there depict the opening's interior, which the model
        // renders as the actual cavity. Front still == back.
        let mouth_row = m.get((a + b) / 2, y, d / 2).is_none();
        for &x in &xs {
            let px = *sprite.get_pixel(x, r);
            let front = (0..d)
                .rev()
                .find(|&z| m.get(x, y, z).is_some())
                .map(|z| m.get(x, y, z).expect("found"));
            let back = (0..d)
                .find(|&z| m.get(x, y, z).is_some())
                .map(|z| m.get(x, y, z).expect("found"));
            if mouth_row {
                assert_eq!(front, back, "mouth rim symmetric {x},{y}");
            } else {
                assert_eq!(front, Some(px), "front is sprite paint {x},{y}");
                assert_eq!(back, Some(px), "back matches front {x},{y}");
            }
        }
    }
    let depth_at = |y: u32, x: u32| (0..d).filter(|&z| m.get(x, y, z).is_some()).count() as u32;
    assert!(
        depth_at(widest.1, widest.2) >= widest.0 - 2,
        "widest row is as deep as it is wide: {} vs {}",
        depth_at(widest.1, widest.2),
        widest.0
    );
    assert!(
        narrowest.0 < widest.0 && depth_at(narrowest.1, narrowest.2) < depth_at(widest.1, widest.2),
        "side profile follows the sprite (not flat): narrow {} vs wide {}",
        depth_at(narrowest.1, narrowest.2),
        depth_at(widest.1, widest.2)
    );
}

#[test]
fn blob_is_irregular_not_a_revolution() {
    // Boulders, the fungal clump, egg sacks (Joey's reviews,
    // 2026-11-17: "Literally a boulder"; the fungal clump is
    // "an irregular shape with lumps"). The lathe made them
    // surfaces of revolution — cylinders with a profile. The
    // blob's front is the sprite verbatim, its per-column
    // depth varies irregularly, and some row's depth profile
    // is asymmetric about the row's center (a revolution's
    // never is).
    let sprite = load_sprite("f_boulder_medium");
    let seg = seamless_segment(&sprite).expect("seamless fires");
    let m = model_for("f_boulder_medium", &sprite).expect("models");
    assert_same_model(&m, &model_for("f_boulder_medium", &sprite).expect("rebuild"));
    let (fh, d) = (m.size[1], m.size[2]);
    let bot = seg.front_rows.end - 1;
    let depth_at = |x: u32, y: u32| (0..d).filter(|&z| m.get(x, y, z).is_some()).count() as u32;
    let mut depths: Vec<u32> = Vec::new();
    let mut asymmetric = false;
    for y in 0..fh {
        let r = bot - y;
        let xs: Vec<u32> = (0..32)
            .filter(|&x| structure(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            continue;
        }
        let (a, b) = (xs[0], xs[xs.len() - 1]);
        for &x in &xs {
            let px = *sprite.get_pixel(x, r);
            let front = (0..d)
                .rev()
                .find(|&z| m.get(x, y, z).is_some())
                .map(|z| m.get(x, y, z).expect("found"));
            assert_eq!(front, Some(px), "blob front verbatim {x},{y}");
            depths.push(depth_at(x, y));
            let mirror = a + b - x;
            if mirror != x && xs.contains(&mirror) && depth_at(mirror, y) != depth_at(x, y) {
                asymmetric = true;
            }
        }
    }
    let (mn, mx) = (
        *depths.iter().min().expect("depths"),
        *depths.iter().max().expect("depths"),
    );
    assert!(mx - mn >= 6, "depth varies irregularly: {mn}..{mx}");
    assert!(asymmetric, "no row is asymmetric — still a revolution?");
}

#[test]
fn plan_slab_is_low_and_top_is_the_sprite() {
    let sprite = load_sprite("f_groundsheet");
    let m = model_for("f_groundsheet", &sprite).expect("groundsheet models");
    assert_eq!(m.size, [32, 2, 32], "groundsheet slab size");
    assert_same_model(&m, &model_for("f_groundsheet", &sprite).expect("rebuild"));
    let pixels: HashSet<[u8; 4]> = sprite.pixels().map(|p| p.0).collect();
    for z in 0..32 {
        for x in 0..32 {
            let px = *sprite.get_pixel(x, z);
            if structure(px) {
                assert_eq!(m.get(x, 1, z), Some(px), "top verbatim {x},{z}");
                assert!(m.get(x, 0, z).is_some(), "slab body {x},{z}");
            } else {
                assert_eq!(m.get(x, 1, z), None, "no invented mass {x},{z}");
            }
        }
    }
    for c in m.voxels.iter().flatten() {
        assert!(pixels.contains(&c.0), "plan colors are sprite paint: {c:?}");
    }
    // Mattress pin: thicker, still a low pad.
    let mat = model_for("f_down_mattress", &load_sprite("f_down_mattress"))
        .expect("mattress models");
    assert_eq!(mat.size[1], 6, "mattress thickness");
    // The firering keeps its hole: some interior cell of the
    // sprite's bounding box is not structure and stays empty.
    let ring_sprite = load_sprite("f_firering");
    let ring = model_for("f_firering", &ring_sprite).expect("firering models");
    let mut hole_seen = false;
    for z in 0..32 {
        for x in 0..32 {
            if !structure(*ring_sprite.get_pixel(x, z)) && ring.get(x, 2, z).is_none() {
                hole_seen = true;
            }
        }
    }
    assert!(hole_seen, "firering has empty cells (its hole)");
}

#[test]
fn panel_slab_is_thin_and_front_verbatim() {
    let sprite = load_sprite("f_canvas_door");
    let m = model_for("f_canvas_door", &sprite).expect("canvas door models");
    assert_eq!(m.size, [32, 32, 3], "door panel size");
    assert_same_model(&m, &model_for("f_canvas_door", &sprite).expect("rebuild"));
    let pixels: HashSet<[u8; 4]> = sprite.pixels().map(|p| p.0).collect();
    for c in m.voxels.iter().flatten() {
        assert!(pixels.contains(&c.0), "panel colors are sprite paint: {c:?}");
    }
    // Front plane verbatim over the silhouette band's structure.
    let grave = load_sprite("f_grave_stone");
    let g = model_for("f_grave_stone", &grave).expect("grave stone models");
    assert_eq!(g.size[2], 3, "grave stone thickness");
    let grave_pixels: HashSet<[u8; 4]> = grave.pixels().map(|p| p.0).collect();
    let mut any = false;
    for y in 0..g.size[1] {
        for x in 0..32 {
            if let Some(c) = g.get(x, y, 2) {
                any = true;
                assert!(grave_pixels.contains(&c.0), "grave front is sprite paint");
                assert!(g.get(x, y, 0).is_some(), "grave back closed {x},{y}");
            }
        }
    }
    assert!(any, "grave stone has a front");
}

#[test]
fn exclusions_and_skeletons_still_decline() {
    for id in [
        "f_dandelion_season_spring", // vegetation
        "f_traffic_light",            // exclusion pin (pole)
        "f_bed_frame",                // exclusion pin (slatted frame)
        "f_deckchair",                // class A exclusion pin
    ] {
        let sprite = load_sprite(id);
        assert!(model_for(id, &sprite).is_none(), "{id} must stay unmodeled");
    }
}
