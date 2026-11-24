//! Phase 5 rework Stage G gate (generation side): sprite
//! segmentation + box/column prop models on NAMED real UltiCa
//! furniture sprites.
//!
//! The chosen sprites were probed before this gate was written
//! (row coverage / mean-L / seam-edge profiles): each has one clean
//! top/front seam. Expected boundary bands below are the probed seam
//! rows, widened by a couple of rows for the apron/trim ambiguity —
//! they are sanity bands, not fitted values: the gate's real teeth
//! are the exactness assertions (box front face == sprite front
//! pixels, column front projection == front silhouette, column back
//! panel == mirrored silhouette) and the `None` cases.
//!
//! The canonical model is now `solid_model` (the front silhouette
//! extruded and filled at every depth cell): `check_solid` asserts
//! its front projection == front silhouette exactly (colors
//! included) and SOLIDITY — every silhouette cell filled at every
//! z, no gaps — for all five props. The box/column assertions below
//! still run (both models are kept, deprecated).
//!
//! `stage_g_solid_model_inferred_faces` covers the inferred unseen
//! faces. Side inference is PER-CONSTRUCTION (2026-11-16, fourth
//! review round): a bookcase's flank is a flat wooden slab — the
//! shelves do not continue through it — so enclosed furniture
//! (Construction::Panel, the default) gets primary-material panel
//! flanks, asserted here as panel-uniformity (every non-rim cell
//! within OKLab 0.06 of the DE-SHADED primary, row-mean L
//! variance < 0.001 — no banding) for f_fridge, f_bookcase and
//! f_locker. This REPLACES the previous fridge assertion (side
//! cells == the de-shaded row field): the field rule no longer
//! colors enclosed flanks. The field rule survives for
//! open-base/structural props (f_table) and as the sliver
//! re-anchor reference; the bookcase's sliver flank is still
//! asserted cell-by-cell against the recomputed sliver
//! un-projection. f_counter's flank carries its two-panel split
//! (see `stage_g_counter_back_is_two_panels`). The back is a
//! plain panel in the primary material, not the row fields.
//! `stage_g_construction_profiles` pins the assignment itself.
//!
//! Albedo-only cleanup covered here (2026-11-16, third review
//! round): the model carries NO baked light — every directional
//! multiplier is gone (sides x0.85, interior x0.7 in the earlier
//! solid model) and derived colors are de-shaded: the sprite's
//! baked top-to-bottom L gradient is divided out of row-derived
//! colors by a per-row gain (median row L / row L over a smoothed
//! mean-L profile, clamped to [0.80, 1.25], a/b preserved), and
//! the back panel's primary by re-anchoring its L to the palette's weighted median. The test
//! recomputes both gains itself. `stage_g_solid_model_albedo_only`
//! is the headline gate: a synthetic sprite of ONE uniform
//! material with a fake baked gradient + darker sliver painted
//! over it must yield side/back cells at the original material
//! color, while its front plane keeps the gradient verbatim (the
//! front IS the texture).
//!
//! Back-panel + artifact fixes covered here (2026-11-16, second
//! review round):
//! * Backs are plain panels: f_fridge/f_bookcase back cells sit
//!   within OKLab 0.06 of the prop's DE-SHADED primary material
//!   (largest non-shadow palette cluster, recomputed here via the
//!   analyzer, then re-anchored to the palette weighted median), the
//!   bookcase back's row-mean L variance is tiny
//!   (no banding), and f_counter's back is two-material (rim band
//!   in the countertop material, body panel in the front primary).
//! * Black artifacts, root-caused: (1) f_fridge's right edge
//!   column is transparent black ([0,0,0,128] fringe — no color
//!   data), which the old alpha>=128 silhouette copied verbatim
//!   onto the front plane and the sliver un-projection — the
//!   solid model's silhouette is now STRUCTURE (opaque, not
//!   fringe), asserted in `check_solid`; (2) f_bookcase's shelf
//!   rows are fully-opaque pure black, so their row field was
//!   black and painted black bands on sides/back — shadow fields
//!   now borrow the nearest non-shadow field for side coloring,
//!   and backs are panels. `stage_g_no_black_inferred_cells`
//!   asserts no side/back cell (rim excluded — the rim belongs to
//!   the top surface) in any gate prop has L < 0.10.
//! * Minimum depth: `stage_g_minimum_depth` pins the floored
//!   depths (f_locker 4 -> 11; the rest unchanged).
//!
//! Weak-case fixes covered here (2026-11-16):
//! * f_table's box is slab-and-legs: its front base is painted
//!   shadow (alpha 128) between two leg runs, so the under-space
//!   must be EMPTY in the box model (the solid-box front-face
//!   assertion therefore does not apply to it).
//! * Column models are closed shells: back panel + side strips are
//!   asserted present for every prop.
//! * f_locker enters via `segment_for` (per-id boundary override,
//!   28); bare `segment` still picks the mid door-seam — asserted,
//!   so the override is proven to be what fixes it.
//! * f_fridge carries a 0.65 confidence floor: with the recalibrated
//!   formula (seam support instead of runner-up margin) its correct
//!   shallow-top segmentation measures in the same band as the other
//!   clean props, so 0.65 is a floor with headroom, not a fit.
//!
//! The tileset is read from the MAIN tree checkout (fork-src is
//! gitignored and exists only there), same as stage_c_gate.

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

/// (id, expected boundary band, confidence floor) — probed seams:
/// f_counter 20, f_bookcase 27, f_table 19-20, f_fridge 17-19.
const PROPS: [(&str, std::ops::RangeInclusive<u32>, f32); 4] = [
    ("f_counter", 18..=22, 0.45),
    ("f_bookcase", 25..=29, 0.45),
    ("f_table", 18..=21, 0.45),
    ("f_fridge", 15..=20, 0.65),
];

fn load(ts: &tileset::Tileset, id: &str) -> image::RgbaImage {
    let entry = ts.entries.get(id).unwrap_or_else(|| panic!("tileset has {id}"));
    composite_entry(ts, entry).unwrap_or_else(|| panic!("{id} composites"))
}

/// Structure, the solid model's silhouette test (recomputed from
/// the documented convention): opaque and not transparent-black
/// fringe (RGB exactly 0 below alpha 254 carries no color data).
fn structure(px: image::Rgba<u8>) -> bool {
    px[3] >= 128 && !(px[3] < 254 && px[0] == 0 && px[1] == 0 && px[2] == 0)
}

/// A sprite region's primary material, recomputed via the public
/// analyzer: the largest palette cluster (k=5) that is not shadow
/// (L >= 0.15).
fn primary_of(sprite: &image::RgbaImage, rows: &std::ops::Range<u32>) -> Oklab {
    let crop = image::imageops::crop_imm(
        sprite,
        0,
        rows.start,
        sprite.width(),
        rows.end - rows.start,
    )
    .to_image();
    let a = analyze_image(&crop, 5);
    a.palette
        .iter()
        .find(|p| p.lab.l >= 0.15)
        .or_else(|| a.palette.first())
        .map(|p| p.lab)
        .expect("front region has a palette")
}

fn lab_of(px: image::Rgba<u8>) -> Oklab {
    Oklab::from_srgb_u8([px[0], px[1], px[2]])
}

/// Shared model assertions. `solid_box` selects the box variant:
/// true = solid cuboid (verbatim front face over the whole front),
/// false = slab-and-legs (f_table: under-space empty, legs filled).
fn check_models(id: &str, sprite: &image::RgbaImage, seg: &Segmentation, solid_box: bool) {
    let bm = box_model(sprite, seg);
    let cm = column_model(sprite, seg);
    assert_eq!(bm, box_model(sprite, seg), "{id}: box model deterministic");
    assert_eq!(cm, column_model(sprite, seg), "{id}: column model deterministic");

    let (w, fh, d) = (bm.size[0], bm.size[1], bm.size[2]);
    assert_eq!(fh, seg.front_rows.end - seg.front_rows.start, "{id}: model height = front height");
    assert_eq!(cm.size, bm.size, "{id}: box and column share the grid");
    let bot = seg.front_rows.end - 1;
    if solid_box {
        // Box front face: every opaque sprite front pixel inside
        // the box's x-extent appears verbatim on the front face.
        // (Sampled on a stride to keep the gate fast; the column
        // check below is exhaustive.)
        for y in 0..fh {
            let r = bot - y;
            for x in (0..w).step_by(2) {
                let px = *sprite.get_pixel(x, r);
                if px[3] >= 128 {
                    assert_eq!(
                        bm.get(x, y, d - 1),
                        Some(px),
                        "{id}: box front face mismatch at sprite ({x},{r})"
                    );
                }
            }
        }
        // The solid box is a filled cuboid; the column shell is
        // strictly sparser.
        assert!(
            bm.solid_count() > cm.solid_count(),
            "{id}: box {} should exceed column shell {}",
            bm.solid_count(),
            cm.solid_count()
        );
    } else {
        // Slab-and-legs (f_table): the slab is the 1-row apron
        // (probed header band); everything below it between the leg
        // runs (x 1..=3 and 28..=30) is open under-space.
        for x in [8u32, 16, 24] {
            for y in 0..fh - 1 {
                for z in 0..d {
                    assert_eq!(
                        bm.get(x, y, z),
                        None,
                        "{id}: under-space must be empty at ({x},{y},{z})"
                    );
                }
            }
        }
        // Legs are filled at their columns, front and back planes.
        for x in [2u32, 29] {
            assert!(bm.get(x, 0, d - 1).is_some(), "{id}: leg front at x={x}");
            assert!(bm.get(x, 0, 0).is_some(), "{id}: leg back at x={x}");
            assert_eq!(
                bm.get(x, 0, d - 1),
                Some(*sprite.get_pixel(x, bot)),
                "{id}: leg front face is the sprite's bottom pixel at x={x}"
            );
        }
        // The slab's front face is the apron row, verbatim.
        assert_eq!(
            bm.get(16, fh - 1, d - 1),
            Some(*sprite.get_pixel(16, seg.front_rows.start)),
            "{id}: slab front face is the apron row"
        );
    }
    // Column model front projection == front region silhouette,
    // exactly, for every cell — and the back panel mirrors that
    // silhouette at z = 0 (closed shell). Exception: the cap row
    // (y = fh-1) at z = 0 is owned by the top cap, whose back edge
    // un-projects the top region's first row — present iff either
    // surface has a pixel there.
    for x in 0..w {
        for y in 0..fh {
            let r = bot - y;
            let want = sprite.get_pixel(x, r)[3] >= 128;
            assert_eq!(
                cm.get(x, y, d - 1).is_some(),
                want,
                "{id}: column front projection mismatch at ({x},{y})"
            );
            if y < fh - 1 {
                assert_eq!(
                    cm.get(x, y, 0).is_some(),
                    want,
                    "{id}: column back panel mismatch at ({x},{y})"
                );
            } else {
                let cap = sprite.get_pixel(x, seg.top_rows.start)[3] >= 128;
                assert_eq!(
                    cm.get(x, y, 0).is_some(),
                    want || cap,
                    "{id}: column back/cap corner mismatch at ({x},{y})"
                );
            }
            if want {
                assert_eq!(
                    cm.get(x, y, d - 1),
                    Some(*sprite.get_pixel(x, r)),
                    "{id}: column front color mismatch at ({x},{y})"
                );
            }
        }
    }
    // Side closure strips: at each front row's opaque extremes,
    // every depth cell is filled.
    for r in seg.front_rows.clone() {
        let y = bot - r;
        let (mut rx0, mut rx1) = (w, 0u32);
        for x in 0..w {
            if sprite.get_pixel(x, r)[3] >= 128 {
                rx0 = rx0.min(x);
                rx1 = rx1.max(x);
            }
        }
        if rx0 > rx1 {
            continue;
        }
        for sx in [rx0, rx1] {
            for z in 0..d {
                assert!(
                    cm.get(sx, y, z).is_some(),
                    "{id}: side strip gap at ({sx},{y},{z})"
                );
            }
        }
    }
}

/// The canonical solid model. `open_base` is f_table only (probed,
/// same facts as the slab-and-legs box assertions): a 1-row apron
/// slab at the top of the front, leg runs at x 1..=3 and 28..=30;
/// the under-space between the legs is painted shadow and must be
/// empty at every z — front plane included.
fn check_solid(id: &str, sprite: &image::RgbaImage, seg: &Segmentation, open_base: bool) {
    let sm = solid_model(sprite, seg);
    assert_eq!(sm, solid_model(sprite, seg), "{id}: solid model deterministic");
    let (w, fh, d) = (sm.size[0], sm.size[1], sm.size[2]);
    assert_eq!(
        fh,
        seg.front_rows.end - seg.front_rows.start,
        "{id}: solid model height = front height"
    );
    let bot = seg.front_rows.end - 1;
    let expected = |x: u32, y: u32| -> bool {
        if !structure(*sprite.get_pixel(x, bot - y)) {
            return false; // not in the (structure) front silhouette
        }
        if !open_base {
            return true;
        }
        y >= fh - 1 || (1..=3).contains(&x) || (28..=30).contains(&x)
    };
    let mut solid_voxels = 0usize;
    for x in 0..w {
        for y in 0..fh {
            let want = expected(x, y);
            // (b) SOLIDITY: a silhouette cell is filled at EVERY z;
            // a non-silhouette cell at none. No gaps, anywhere.
            for z in 0..d {
                assert_eq!(
                    sm.get(x, y, z).is_some(),
                    want,
                    "{id}: solid model solidity mismatch at ({x},{y},{z})"
                );
            }
            if want {
                solid_voxels += d as usize;
                // (a) Front projection == front silhouette exactly,
                // colors included.
                assert_eq!(
                    sm.get(x, y, d - 1),
                    Some(*sprite.get_pixel(x, bot - y)),
                    "{id}: solid front color mismatch at ({x},{y})"
                );
            }
        }
    }
    println!("{id}: solid model {w}x{fh}x{d} — {solid_voxels} solid voxels, all asserted filled at every z");
}

#[test]
fn stage_g_props_segment_and_project_exactly() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (id, band, min_conf) in &PROPS {
        let sprite = load(&ts, id);
        let seg = segment(&sprite)
            .unwrap_or_else(|| panic!("{id}: must segment (caller would keep extrusion)"));
        println!(
            "{id}: sprite {}x{}, silhouette {:?}, boundary {} (top {:?}, front {:?}), side {:?}, confidence {:.2}",
            sprite.width(),
            sprite.height(),
            seg.silhouette_rows,
            seg.boundary(),
            seg.top_rows,
            seg.front_rows,
            seg.side,
            seg.confidence
        );
        assert!(
            band.contains(&seg.boundary()),
            "{id}: boundary {} outside expected band {band:?}",
            seg.boundary()
        );
        assert!(
            seg.confidence >= *min_conf,
            "{id}: confidence {:.2} below its floor {min_conf}",
            seg.confidence
        );
        // Determinism: segmentation is a pure function of the sprite.
        assert_eq!(seg, segment(&sprite).unwrap(), "{id}: segmentation must be deterministic");
        // Only f_table has an open base; the rest are solid boxes.
        check_models(id, &sprite, &seg, *id != "f_table");
        check_solid(id, &sprite, &seg, *id == "f_table");
    }
}

#[test]
fn stage_g_locker_segments_via_override() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let sprite = load(&ts, "f_locker");
    // Without the override, the scored rule still picks the mid
    // door-seam (38) — the override, not the calibration, fixes it.
    let raw = segment(&sprite).expect("f_locker segments (at the wrong seam)");
    assert_eq!(
        raw.boundary(),
        38,
        "f_locker without override: expected the documented stacked-seam miss at 38, got {}",
        raw.boundary()
    );
    let seg = segment_for("f_locker", &sprite)
        .unwrap_or_else(|| panic!("f_locker: override segmentation must succeed"));
    println!(
        "f_locker: override boundary {} (top {:?}, front {:?}), side {:?}, confidence {:.2} (floor bypassed)",
        seg.boundary(),
        seg.top_rows,
        seg.front_rows,
        seg.side,
        seg.confidence
    );
    assert_eq!(seg.boundary(), 28, "f_locker: override boundary is the cap seam");
    assert_eq!(seg.top_rows, 26..28, "f_locker: top is the 2-row cap");
    assert_eq!(
        seg,
        segment_for("f_locker", &sprite).unwrap(),
        "f_locker: override segmentation must be deterministic"
    );
    // Solid box; its front face matches the sprite front region
    // (asserted inside check_models).
    check_models("f_locker", &sprite, &seg, true);
    check_solid("f_locker", &sprite, &seg, false);
}

/// Independent de-shading recompute (the documented rule in
/// props.rs, re-derived here so the checks below don't call the
/// code under test): per-row mean L over structure pixels in the
/// region's trimmed structure extent (outermost 2 columns), the
/// given columns excluded, empty rows nearest-filled, radius-2
/// moving-average smoothing. Returns (smoothed row L in region
/// row order, median over rows with data, pixel-weighted mean).
fn l_profile(
    sprite: &image::RgbaImage,
    rows: &std::ops::Range<u32>,
    exclude: Option<&std::ops::Range<u32>>,
) -> Option<(Vec<f32>, f32, f32)> {
    let (mut x0, mut x1) = (sprite.width() - 1, 0u32);
    let mut any = false;
    for r in rows.clone() {
        for x in 0..sprite.width() {
            if structure(*sprite.get_pixel(x, r)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    let (tx0, tx1) = if x1 >= x0 + 4 { (x0 + 2, x1 - 2) } else { (x0, x1) };
    let n_rows = (rows.end - rows.start) as usize;
    let mut sum = vec![0.0f32; n_rows];
    let mut cnt = vec![0u32; n_rows];
    for (i, r) in rows.clone().enumerate() {
        for x in tx0..=tx1 {
            if exclude.map_or(false, |e| e.contains(&x)) {
                continue;
            }
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                sum[i] += lab_of(px).l;
                cnt[i] += 1;
            }
        }
    }
    let total: u32 = cnt.iter().sum();
    if total == 0 {
        return None;
    }
    let mean = sum.iter().sum::<f32>() / total as f32;
    let raw: Vec<Option<f32>> = (0..n_rows)
        .map(|i| {
            if cnt[i] > 0 {
                Some(sum[i] / cnt[i] as f32)
            } else {
                None
            }
        })
        .collect();
    let filled: Vec<f32> = (0..n_rows)
        .map(|i| {
            raw[i]
                .or_else(|| {
                    (1..n_rows).find_map(|dist| {
                        if i + dist < n_rows && raw[i + dist].is_some() {
                            raw[i + dist]
                        } else if dist <= i && raw[i - dist].is_some() {
                            raw[i - dist]
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or(mean)
        })
        .collect();
    let smoothed: Vec<f32> = (0..n_rows)
        .map(|i| {
            let lo = i.saturating_sub(2);
            let hi = (i + 2).min(n_rows - 1);
            filled[lo..=hi].iter().sum::<f32>() / (hi - lo + 1) as f32
        })
        .collect();
    let mut vals: Vec<f32> = (0..n_rows)
        .filter(|&i| raw[i].is_some())
        .map(|i| smoothed[i])
        .collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if vals.len() % 2 == 1 {
        vals[vals.len() / 2]
    } else {
        0.5 * (vals[vals.len() / 2 - 1] + vals[vals.len() / 2])
    };
    Some((smoothed, median, mean))
}

/// Per-row de-shading gains for the front region, indexed by
/// MODEL y: gain = median row L / smoothed row L, clamped to
/// [0.80, 1.25]; the segmentation's sliver columns are excluded
/// from the profile.
fn row_gains(sprite: &image::RgbaImage, seg: &Segmentation) -> Vec<f32> {
    let fh = (seg.front_rows.end - seg.front_rows.start) as usize;
    let exclude = seg.side.as_ref().map(|sl| &sl.cols);
    match l_profile(sprite, &seg.front_rows, exclude) {
        Some((smoothed, median, _)) => (0..fh)
            .map(|y| {
                let s = smoothed[fh - 1 - y];
                if s > 0.0 {
                    (median / s).clamp(0.80, 1.25)
                } else {
                    1.0
                }
            })
            .collect(),
        None => vec![1.0; fh],
    }
}

/// Scale a color's OKLab L by `gain`, a/b preserved.
fn deshade(px: image::Rgba<u8>, gain: f32) -> image::Rgba<u8> {
    let mut lab = lab_of(px);
    lab.l = (lab.l * gain).clamp(0.0, 1.0);
    let [r, g, b] = lab.to_srgb_u8();
    image::Rgba([r, g, b, 255])
}

/// A region's primary material, DE-SHADED by re-anchoring its L
/// to the palette's weighted median L over the non-shadow
/// clusters (cumulative-weight crossing rule, as in
/// `generate::weighted_median_l`), gain clamped to [0.80, 1.25]
/// — the back panel's expected base color, recomputed from the
/// analyzer's palette directly.
fn deshaded_primary(sprite: &image::RgbaImage, rows: &std::ops::Range<u32>) -> Oklab {
    let crop = image::imageops::crop_imm(
        sprite,
        0,
        rows.start,
        sprite.width(),
        rows.end - rows.start,
    )
    .to_image();
    let a = analyze_image(&crop, 5);
    let mut lab = a
        .palette
        .iter()
        .find(|p| p.lab.l >= 0.15)
        .or_else(|| a.palette.first())
        .map(|p| p.lab)
        .expect("region has a palette");
    let mut ls: Vec<(f32, f32)> = a
        .palette
        .iter()
        .filter(|p| p.lab.l >= 0.15)
        .map(|p| (p.lab.l, p.weight))
        .collect();
    if !ls.is_empty() && lab.l > 0.0 {
        ls.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        let total: f32 = ls.iter().map(|x| x.1).sum();
        let mut acc = 0.0f32;
        let mut median = ls[ls.len() - 1].0;
        for (l, w) in &ls {
            acc += w;
            if acc >= total * 0.5 {
                median = *l;
                break;
            }
        }
        lab.l = (lab.l * (median / lab.l).clamp(0.80, 1.25)).clamp(0.0, 1.0);
    }
    lab
}

/// The front region's opaque x-extent, recomputed from the sprite.
fn front_extent(sprite: &image::RgbaImage, seg: &Segmentation) -> (u32, u32) {
    let (mut x0, mut x1) = (sprite.width() - 1, 0u32);
    for r in seg.front_rows.clone() {
        for x in 0..sprite.width() {
            if sprite.get_pixel(x, r)[3] >= 128 {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    (x0, x1)
}

/// Independent recompute of a front row's FIELD color: the modal
/// (most common) solid-paint (alpha >= 254) color over the front
/// extent with the outermost 2 columns on each side ignored;
/// fallbacks: untrimmed solid paint, then untrimmed opaque paint.
/// Ties break toward the smallest RGBA key.
fn field_of(sprite: &image::RgbaImage, seg: &Segmentation, r: u32) -> image::Rgba<u8> {
    let (x0, x1) = front_extent(sprite, seg);
    let modal = |lo: u32, hi: u32, min_alpha: u8| -> Option<image::Rgba<u8>> {
        let mut counts: std::collections::BTreeMap<[u8; 4], u32> =
            std::collections::BTreeMap::new();
        for x in lo..=hi {
            let px = *sprite.get_pixel(x, r);
            if px[3] >= min_alpha {
                *counts.entry(px.0).or_insert(0) += 1;
            }
        }
        let mut best: Option<([u8; 4], u32)> = None;
        for (k, n) in counts {
            if best.map_or(true, |(_, bn)| n > bn) {
                best = Some((k, n));
            }
        }
        best.map(|(k, _)| image::Rgba(k))
    };
    let (tx0, tx1) = if x1 >= x0 + 4 { (x0 + 2, x1 - 2) } else { (x0, x1) };
    modal(tx0, tx1, 254)
        .or_else(|| modal(x0, x1, 254))
        .or_else(|| modal(x0, x1, 128))
        .unwrap_or_else(|| panic!("front row {r} has a field color"))
}

/// Panel-flank assertion (Construction::Panel): every non-rim
/// side cell on the given flank sits within OKLab 0.06 of the
/// DE-SHADED primary (a flat slab — only the panel's noise grain
/// varies), and the flank's row-mean L variance is < 0.001 (no
/// banding: no front structure wraps a panel flank). The edge
/// column's rim row is excluded — it belongs to the top surface.
fn assert_panel_flank(
    id: &str,
    sprite: &image::RgbaImage,
    seg: &Segmentation,
    sm: &PropModel,
    on_right: bool,
    primary: Oklab,
) -> usize {
    let (w, fh, d) = (sm.size[0], sm.size[1], sm.size[2]);
    let bot = seg.front_rows.end - 1;
    let mut row_means: Vec<f32> = Vec::new();
    let mut cells = 0usize;
    for y in 0..fh {
        let r = bot - y;
        let cols: Vec<u32> = (0..w).filter(|&x| structure(*sprite.get_pixel(x, r))).collect();
        if cols.is_empty() {
            continue;
        }
        let edge = if on_right {
            *cols.last().unwrap()
        } else {
            *cols.first().unwrap()
        };
        let rim = (0..fh)
            .rev()
            .find(|&yy| structure(*sprite.get_pixel(edge, bot - yy)));
        if rim == Some(y) {
            continue; // top surface row, not flank inference
        }
        let (mut lsum, mut n) = (0.0f32, 0u32);
        for z in 1..d - 1 {
            let c = sm.get(edge, y, z).expect("panel side cell exists");
            let dist = lab_of(c).distance(primary);
            assert!(
                dist <= 0.06,
                "{id}: panel side cell at ({edge},{y},{z}) is {dist:.3} from the de-shaded primary (rgb {:?})",
                c.0
            );
            lsum += lab_of(c).l;
            n += 1;
            cells += 1;
        }
        if n > 0 {
            row_means.push(lsum / n as f32);
        }
    }
    assert!(
        cells > 50,
        "{id}: panel flank assertion must be non-vacuous ({cells} cells)"
    );
    let mean = row_means.iter().sum::<f32>() / row_means.len() as f32;
    let var = row_means.iter().map(|m| (m - mean) * (m - mean)).sum::<f32>()
        / row_means.len() as f32;
    assert!(
        var < 0.001,
        "{id}: panel flank row-mean L variance {var:.5} must show no banding (< 0.001)"
    );
    println!(
        "{id}: panel flank (on_right={on_right}): {cells} cells within 0.06 of the primary, row-mean L variance {var:.5}"
    );
    cells
}

/// Inferred unseen faces of the solid model (panel flanks for
/// enclosed furniture, plain panel backs, the bookcase's sliver
/// flank). The top rim is excluded throughout: a column's
/// topmost cell belongs to the top surface (top-region
/// un-projection), not to the side/back inference.
#[test]
fn stage_g_solid_model_inferred_faces() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");

    // ---- f_fridge: (a) the non-sliver flank is a flat PANEL
    // (Construction::Panel — enclosed furniture; this replaces
    // the old assertion that side cells equal the de-shaded row
    // field: the field rule no longer colors fridge flanks),
    // (b) back plane is a plain panel in the de-shaded primary
    // material.
    let sprite = load(&ts, "f_fridge");
    let seg = segment(&sprite).expect("f_fridge segments");
    let sm = solid_model_for("f_fridge", &sprite, &seg);
    let (w, fh, _d) = (sm.size[0], sm.size[1], sm.size[2]);
    let bot = seg.front_rows.end - 1;
    // Per-column topmost STRUCTURE cell (model y); None for the
    // fringe-only column x=31, which has no cells at all now.
    let col_top = |x: u32| -> Option<u32> {
        (0..fh).rev().find(|&y| structure(*sprite.get_pixel(x, bot - y)))
    };
    let primary = deshaded_primary(&sprite, &seg.front_rows);
    // The flank WITHOUT the sliver is the panel flank (the
    // sliver flank keeps the sprite evidence, as before).
    let panel_on_right = seg.side.as_ref().map_or(true, |sl| !sl.on_right);
    let side_cells = assert_panel_flank("f_fridge", &sprite, &seg, &sm, panel_on_right, primary);
    // (b) Back plane: every non-rim back cell within OKLab 0.06 of
    // the DE-SHADED primary material — a single plain panel with
    // only its noise grain. Non-vacuous: the sample must include
    // cells whose FRONT pixel is far from the primary, so a
    // mirrored or field-based back would fail here.
    let mut back_cells = 0usize;
    let mut non_vacuous = 0usize;
    for y in 0..fh {
        let r = bot - y;
        for x in 0..w {
            if !structure(*sprite.get_pixel(x, r)) || col_top(x) == Some(y) {
                continue;
            }
            let c = sm.get(x, y, 0).expect("f_fridge: back cell exists");
            let dist = lab_of(c).distance(primary);
            assert!(
                dist <= 0.06,
                "f_fridge: back cell at ({x},{y},0) is {dist:.3} from the primary material"
            );
            back_cells += 1;
            if lab_of(*sprite.get_pixel(x, r)).distance(primary) > 0.06 {
                non_vacuous += 1;
            }
        }
    }
    assert!(back_cells > 500, "f_fridge: back assertion must be wide ({back_cells} cells)");
    assert!(
        non_vacuous > 100,
        "f_fridge: back sample must include much front detail the panel discards ({non_vacuous})"
    );
    println!("f_fridge: {side_cells} panel side cells, {back_cells} back cells within 0.06 of the de-shaded primary ({non_vacuous} over discarded front detail)");

    // ---- f_bookcase: (c) BOTH flanks are flat wooden slabs —
    // the non-sliver flank is a primary-material panel (the old
    // shelf-band wrap assertion is gone: bands must NOT wrap a
    // bookcase), and the sliver flank keeps the sliver
    // un-projection, asserted cell-by-cell below; (d) the back
    // is ONE plain wood panel: every non-rim cell within 0.06
    // of the primary and the row-mean L variance across rows is
    // tiny (no banding — the old field-based back had whole
    // pure-black rows).
    let sprite = load(&ts, "f_bookcase");
    let seg = segment(&sprite).expect("f_bookcase segments");
    let sm = solid_model_for("f_bookcase", &sprite, &seg);
    let (w, fh, d) = (sm.size[0], sm.size[1], sm.size[2]);
    let bot = seg.front_rows.end - 1;
    let col_top = |x: u32| -> Option<u32> {
        (0..fh).rev().find(|&y| structure(*sprite.get_pixel(x, bot - y)))
    };
    let primary = deshaded_primary(&sprite, &seg.front_rows);
    // (c1) panel flank: the side WITHOUT the sliver.
    let panel_on_right = seg.side.as_ref().map_or(true, |sl| !sl.on_right);
    assert_panel_flank("f_bookcase", &sprite, &seg, &sm, panel_on_right, primary);
    // (c2) sliver flank: still the sliver un-projection. On rows
    // whose own field is non-shadow (no borrowing involved), each
    // interior side cell must equal the sliver sample DE-SHADED
    // by the re-anchor (de-shaded field L / sliver band mean L)
    // — recomputed here from the documented rule.
    let sl = seg.side.as_ref().expect("f_bookcase: a sliver was detected");
    let sw = sl.cols.end - sl.cols.start;
    let gains = row_gains(&sprite, &seg);
    let mut sliver_cells = 0usize;
    for y in 0..fh {
        let r = bot - y;
        let field = field_of(&sprite, &seg, r);
        if lab_of(field).l < 0.15 {
            continue; // shadow-field row: the module borrows; not exact-recomputable here
        }
        let (mut bs, mut bn) = (0.0f32, 0u32);
        for x in sl.cols.clone() {
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                bs += lab_of(px).l;
                bn += 1;
            }
        }
        if bn == 0 {
            continue;
        }
        let anchor = lab_of(deshade(field, gains[y as usize])).l / (bs / bn as f32);
        let cols: Vec<u32> = (0..w).filter(|&x| structure(*sprite.get_pixel(x, r))).collect();
        let edge = if sl.on_right {
            *cols.last().unwrap()
        } else {
            *cols.first().unwrap()
        };
        if col_top(edge) == Some(y) {
            continue;
        }
        for z in 1..d - 1 {
            let back_steps = d - 1 - z;
            let off = (back_steps * sw / d).min(sw - 1);
            let sx = if sl.on_right {
                sl.cols.end - 1 - off
            } else {
                sl.cols.start + off
            };
            let px = *sprite.get_pixel(sx, r);
            if !structure(px) || lab_of(px).l < 0.15 {
                continue; // shadow/missing sample: the flank falls back to the panel there
            }
            assert_eq!(
                sm.get(edge, y, z),
                Some(deshade(px, anchor)),
                "f_bookcase: sliver side cell at ({edge},{y},{z}) must be the re-anchored sliver sample"
            );
            sliver_cells += 1;
        }
    }
    assert!(
        sliver_cells > 30,
        "f_bookcase: sliver assertion must be non-vacuous ({sliver_cells} cells)"
    );
    println!("f_bookcase: {sliver_cells} sliver flank cells == re-anchored sliver samples");
    // (d) plain wood panel back.
    let mut row_means: Vec<f32> = Vec::new();
    let mut back_cells = 0usize;
    for y in 0..fh {
        let r = bot - y;
        let (mut lsum, mut n) = (0.0f32, 0u32);
        for x in 0..w {
            if !structure(*sprite.get_pixel(x, r)) || col_top(x) == Some(y) {
                continue;
            }
            let c = sm.get(x, y, 0).expect("f_bookcase: back cell exists");
            let dist = lab_of(c).distance(primary);
            assert!(
                dist <= 0.06,
                "f_bookcase: back cell at ({x},{y},0) is {dist:.3} from the primary material"
            );
            lsum += lab_of(c).l;
            n += 1;
            back_cells += 1;
        }
        if n > 0 {
            row_means.push(lsum / n as f32);
        }
    }
    assert!(back_cells > 500, "f_bookcase: back assertion must be wide ({back_cells} cells)");
    let mean = row_means.iter().sum::<f32>() / row_means.len() as f32;
    let var = row_means.iter().map(|m| (m - mean) * (m - mean)).sum::<f32>()
        / row_means.len() as f32;
    assert!(
        var < 0.001,
        "f_bookcase: back row-mean L variance {var:.5} must show no banding (< 0.001)"
    );
    println!("f_bookcase: {back_cells} back cells within 0.06 of the primary, row-mean L variance {var:.5}");

    // ---- f_locker: the non-sliver flank is a flat dark panel —
    // in particular it must NOT show the front's louver banding
    // (the old field wrap carried the vent rows around the
    // flank). Panel-uniformity + the variance bound assert
    // exactly that.
    let sprite = load(&ts, "f_locker");
    let seg = segment_for("f_locker", &sprite).expect("f_locker segments via override");
    let sm = solid_model_for("f_locker", &sprite, &seg);
    let primary = deshaded_primary(&sprite, &seg.front_rows);
    let panel_on_right = seg.side.as_ref().map_or(true, |sl| !sl.on_right);
    assert_panel_flank("f_locker", &sprite, &seg, &sm, panel_on_right, primary);
}

/// f_counter's back is TWO plain panels, and its Panel flank
/// carries the same split: the countertop is a
/// distinct slab material (top/front means 0.155 apart, top = 20
/// of 32 silhouette rows), so the back's rim band carries the
/// countertop material and everything below the rim is the front
/// primary panel. The rim row's back cells belong to the top
/// surface pass (the countertop's back edge row, achromatic light
/// gray [163,163,163]); per-cell OKLab distance to the beige body
/// primary is a coin flip there (0.058-0.065 — both materials are
/// light), so the rim is asserted on its band MEAN (> 0.06 from
/// the body primary) and its achromaticity (chroma < 0.025, vs
/// the warm body primary's ~0.05), while every non-rim cell must
/// be within 0.06 of the DE-SHADED front primary — 2 base colors
/// in total.
#[test]
fn stage_g_counter_back_is_two_panels() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let sprite = load(&ts, "f_counter");
    let seg = segment(&sprite).expect("f_counter segments");
    let sm = solid_model_for("f_counter", &sprite, &seg);
    let (w, fh, d) = (sm.size[0], sm.size[1], sm.size[2]);
    let bot = seg.front_rows.end - 1;
    let col_top = |x: u32| -> Option<u32> {
        (0..fh).rev().find(|&y| structure(*sprite.get_pixel(x, bot - y)))
    };
    let front_primary = primary_of(&sprite, &seg.front_rows);
    let top_primary = primary_of(&sprite, &seg.top_rows);
    assert!(
        front_primary.distance(top_primary) > 0.10,
        "f_counter: the two panel materials must actually differ ({:.3})",
        front_primary.distance(top_primary)
    );
    // Body cells are colored with the DE-SHADED front primary.
    let front_primary_ds = deshaded_primary(&sprite, &seg.front_rows);
    let mut body_cells = 0usize;
    let mut rim_cells = 0usize;
    let mut rim_lab_sum = (0.0f32, 0.0f32, 0.0f32);
    for y in 0..fh {
        let r = bot - y;
        for x in 0..w {
            if !structure(*sprite.get_pixel(x, r)) {
                continue;
            }
            let c = sm.get(x, y, 0).expect("f_counter: back cell exists");
            if col_top(x) == Some(y) {
                // Rim: countertop territory — achromatic stone,
                // not the warm body material.
                let l = lab_of(c);
                let chroma = (l.a * l.a + l.b * l.b).sqrt();
                assert!(
                    chroma < 0.025,
                    "f_counter: rim back cell at ({x},{y},0) must be achromatic stone (chroma {chroma:.3}, rgb {:?})",
                    c.0
                );
                rim_lab_sum.0 += l.l;
                rim_lab_sum.1 += l.a;
                rim_lab_sum.2 += l.b;
                rim_cells += 1;
            } else {
                let dist = lab_of(c).distance(front_primary_ds);
                assert!(
                    dist <= 0.06,
                    "f_counter: body back cell at ({x},{y},0) is {dist:.3} from the de-shaded front primary"
                );
                body_cells += 1;
            }
        }
    }
    assert!(body_cells > 100 && rim_cells > 10, "f_counter: both panels sampled (body {body_cells}, rim {rim_cells})");
    let rim_mean = Oklab {
        l: rim_lab_sum.0 / rim_cells as f32,
        a: rim_lab_sum.1 / rim_cells as f32,
        b: rim_lab_sum.2 / rim_cells as f32,
    };
    assert!(
        rim_mean.distance(front_primary) > 0.06,
        "f_counter: rim band mean must differ from the body primary ({:.3})",
        rim_mean.distance(front_primary)
    );
    println!("f_counter: back = achromatic rim band (mean {:.3} from body) + {body_cells} body cells within 0.06 of the front primary", rim_mean.distance(front_primary));

    // Side: the two-panel split applies to the Panel flank too.
    // The counter's sliver is on its right, so the LEFT flank is
    // the panel flank: its rim row is the countertop's edge
    // (achromatic stone, via the top surface — the same material
    // split as the back's rim band) and every row below it is
    // the front-primary panel at full value — two plain panels,
    // no field banding anywhere on the flank.
    let mut side_body = 0usize;
    let mut side_rim = 0usize;
    for y in 0..fh {
        let r = bot - y;
        let cols: Vec<u32> = (0..w).filter(|&x| structure(*sprite.get_pixel(x, r))).collect();
        if cols.is_empty() {
            continue;
        }
        let edge = *cols.first().unwrap(); // left flank (sliver is on the right)
        let is_rim = col_top(edge) == Some(y);
        for z in 1..d - 1 {
            let c = sm.get(edge, y, z).expect("f_counter: side cell exists");
            if is_rim {
                let l = lab_of(c);
                let chroma = (l.a * l.a + l.b * l.b).sqrt();
                assert!(
                    chroma < 0.03,
                    "f_counter: side rim cell at ({edge},{y},{z}) must be achromatic stone (chroma {chroma:.3}, rgb {:?})",
                    c.0
                );
                side_rim += 1;
            } else {
                let dist = lab_of(c).distance(front_primary_ds);
                assert!(
                    dist <= 0.06,
                    "f_counter: side body cell at ({edge},{y},{z}) is {dist:.3} from the de-shaded front primary"
                );
                side_body += 1;
            }
        }
    }
    assert!(
        side_body > 100 && side_rim > 5,
        "f_counter: side split must sample both panels (body {side_body}, rim {side_rim})"
    );
    println!("f_counter: side = stone rim ({side_rim} cells) over {side_body} body panel cells");
}

/// Construction profiles (fourth review round): side inference
/// is assigned per case. The defaults pin the reviewed gate set —
/// enclosed furniture is Panel, the open-base table is
/// Structural, and NOTHING defaults to FieldWrap (it is the
/// opt-in profile for slatted construction, pinned per id via
/// CONSTRUCTION_OVERRIDES — which starts empty, so for these ids
/// the id-threaded and default model entry points must agree
/// exactly).
#[test]
fn stage_g_construction_profiles() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (id, want) in [
        ("f_counter", Construction::Panel),
        ("f_bookcase", Construction::Panel),
        ("f_table", Construction::Structural),
        ("f_fridge", Construction::Panel),
        ("f_locker", Construction::Panel),
    ] {
        let sprite = load(&ts, id);
        let seg = segment_for(id, &sprite).unwrap_or_else(|| panic!("{id} segments"));
        assert_eq!(
            construction_for(Some(id), &sprite, &seg),
            want,
            "{id}: id-threaded construction"
        );
        assert_eq!(
            construction_for(None, &sprite, &seg),
            want,
            "{id}: default construction (no pin in CONSTRUCTION_OVERRIDES)"
        );
        assert_eq!(
            solid_model_for(id, &sprite, &seg),
            solid_model(&sprite, &seg),
            "{id}: solid_model_for == solid_model under the default assignment"
        );
        println!("{id}: construction {want:?}");
    }
}

/// Minimum depth (deliverable A): floor-standing props (silhouette
/// >= 24 rows) get depth >= ceil(width/3). Pinned depths: only
/// f_locker changes (4 -> 11); the rest were already deeper.
#[test]
fn stage_g_minimum_depth() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (id, want) in [
        ("f_counter", 32),
        ("f_bookcase", 14),
        ("f_table", 32),
        ("f_fridge", 16),
        ("f_locker", 11),
    ] {
        let sprite = load(&ts, id);
        let seg = segment_for(id, &sprite).unwrap_or_else(|| panic!("{id} segments"));
        let sm = solid_model(&sprite, &seg);
        assert_eq!(
            sm.size[2], want,
            "{id}: solid model depth {} != pinned {want} (raw rule {})",
            sm.size[2],
            seg.depth_cells()
        );
        println!("{id}: depth {} (raw {})", sm.size[2], seg.depth_cells());
    }
}

/// No black inferred cells (deliverable C gate): for every gate
/// prop, every BACK cell and every SIDE surface cell (interior z
/// at a row's structure extremes) — top rim excluded, since the
/// rim belongs to the evidence-based top surface — has OKLab
/// L >= 0.10. (Front-plane cells are exempt by construction: they
/// are the sprite verbatim, shadow paint included.)
#[test]
fn stage_g_no_black_inferred_cells() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for id in ["f_counter", "f_bookcase", "f_table", "f_fridge", "f_locker"] {
        let sprite = load(&ts, id);
        let seg = segment_for(id, &sprite).unwrap_or_else(|| panic!("{id} segments"));
        let sm = solid_model(&sprite, &seg);
        let (w, fh, d) = (sm.size[0], sm.size[1], sm.size[2]);
        let bot = seg.front_rows.end - 1;
        let col_top = |x: u32| -> Option<u32> {
            (0..fh).rev().find(|&y| structure(*sprite.get_pixel(x, bot - y)))
        };
        let mut checked = 0usize;
        for y in 0..fh {
            let r = bot - y;
            let cols: Vec<u32> =
                (0..w).filter(|&x| structure(*sprite.get_pixel(x, r))).collect();
            if cols.is_empty() {
                continue;
            }
            let (lo, hi) = (*cols.first().unwrap(), *cols.last().unwrap());
            for x in cols.iter().cloned() {
                if col_top(x) == Some(y) {
                    continue;
                }
                let c = sm.get(x, y, 0).expect("back cell exists");
                assert!(
                    lab_of(c).l >= 0.10,
                    "{id}: back cell at ({x},{y},0) has L {:.3} (rgb {:?})",
                    lab_of(c).l,
                    c.0
                );
                checked += 1;
            }
            for edge in [lo, hi] {
                if col_top(edge) == Some(y) {
                    continue;
                }
                for z in 1..d - 1 {
                    let c = sm.get(edge, y, z).expect("side cell exists");
                    assert!(
                        lab_of(c).l >= 0.10,
                        "{id}: side cell at ({edge},{y},{z}) has L {:.3} (rgb {:?})",
                        lab_of(c).l,
                        c.0
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 100, "{id}: no-black assertion must be wide ({checked} cells)");
        println!("{id}: {checked} inferred side/back cells all L >= 0.10");
    }
}

/// Albedo-only headline gate: a synthetic sprite built from ONE
/// uniform material color with only LIGHT painted over it — a
/// brighter lit top, a smooth top-to-bottom L ramp on the front
/// (factor 1.15 -> 0.85), and a right sliver under a further
/// x0.78 face shade — must produce a solid model whose inferred
/// side and back cells are the ORIGINAL material color: the
/// de-shading divides the baked light back out. The front plane
/// keeps the gradient verbatim (the front IS the texture; the
/// engine lights it). Painting is done in OKLab (L scaled, a/b
/// untouched) — the space the correction works in.
///
/// Tolerances (OKLab distance to the material): the RIGHT
/// (sliver) flank's side cells 0.025 — the recovery there is
/// near-exact (sliver re-anchor + analytic gains; the budget is
/// sRGB quantization plus the profile smoothing's bias at the
/// ramp's end rows; measured max 0.013). The LEFT flank is
/// Construction::Panel under the default assignment (closed
/// base), so its cells carry the panel budget — the same 0.05
/// as the back (palette anchor + grain), with the row-mean L
/// spread bound (0.03) standing in for the per-row ramp check.
/// Back cells 0.05 — the panel base is the palette
/// primary re-anchored to the palette's weighted median L
/// (residual ~0.01 on this ramp; measured max 0.023) plus the
/// panel's grain (<= 3% of L for this flat tone, ~0.02), with a
/// separate flatness bound on the back's row-mean L spread
/// (0.03: grain only, no ramp; measured 0.012).
#[test]
fn stage_g_solid_model_albedo_only() {
    let base = image::Rgba([150, 142, 132, 255]);
    let base_lab = lab_of(base);
    let paint = |l_factor: f32| -> image::Rgba<u8> {
        let lab = Oklab {
            l: (base_lab.l * l_factor).clamp(0.0, 1.0),
            a: base_lab.a,
            b: base_lab.b,
        };
        let [r, g, b] = lab.to_srgb_u8();
        image::Rgba([r, g, b, 255])
    };
    let mut img = image::RgbaImage::new(32, 32);
    // Lit top surface (rows 2..8).
    for y in 2..8 {
        for x in 0..32 {
            img.put_pixel(x, y, paint(1.45));
        }
    }
    // Front (rows 8..28): the baked ramp; right 5 columns are
    // the same material as a darker side sliver.
    for y in 8..28 {
        let f = 1.15 - 0.30 * (y - 8) as f32 / 19.0;
        for x in 0..32 {
            img.put_pixel(x, y, paint(if x >= 27 { f * 0.78 } else { f }));
        }
    }
    let seg = segment(&img).expect("synthetic albedo sprite must segment");
    assert_eq!(seg.boundary(), 8, "the lit top must seam at row 8");
    let sl = seg.side.as_ref().expect("the darker right band must read as a sliver");
    assert!(
        sl.on_right && sl.cols.end == 32 && (26..=28).contains(&sl.cols.start),
        "sliver must be the right band, got {sl:?}"
    );
    let sm = solid_model(&img, &seg);
    let (w, fh, d) = (sm.size[0], sm.size[1], sm.size[2]);
    // Side cells on BOTH flanks, rim row excluded (it belongs to
    // the top surface): all at the material color, no
    // top-to-bottom ramp — but the flanks exercise different
    // construction paths now. The synthetic's base is closed, so
    // the default assignment is Panel: the LEFT flank is the
    // primary-material panel (palette anchor + grain budget,
    // like the back), the RIGHT flank is the sliver path
    // (near-exact recovery budget).
    assert_eq!(
        construction_for(None, &img, &seg),
        Construction::Panel,
        "the synthetic's closed base must default to Panel"
    );
    let mut side_max = 0.0f32;
    // Right flank: sliver path, per-cell 0.025, per-row L spread
    // 0.025 (no ramp survives the re-anchor).
    {
        let edge = w - 1;
        let mut ls: Vec<f32> = Vec::new();
        for y in 0..fh - 1 {
            let mut row_l = None;
            for z in 1..d - 1 {
                let c = sm.get(edge, y, z).expect("side cell exists");
                let dist = lab_of(c).distance(base_lab);
                side_max = side_max.max(dist);
                assert!(
                    dist <= 0.025,
                    "sliver side cell at ({edge},{y},{z}) is {dist:.3} from the material (rgb {:?})",
                    c.0
                );
                row_l = Some(lab_of(c).l);
            }
            ls.push(row_l.expect("a side row was sampled"));
        }
        let spread = ls.iter().cloned().fold(0.0f32, f32::max)
            - ls.iter().cloned().fold(1.0f32, f32::min);
        assert!(
            spread <= 0.025,
            "sliver side at x={edge} keeps a top-to-bottom ramp: L spread {spread:.3}"
        );
    }
    // Left flank: panel path, per-cell 0.05 (palette anchor +
    // grain, the back's budget), row-mean L spread 0.03 (grain
    // only, no ramp in the panel).
    {
        let edge = 0u32;
        let mut row_means: Vec<f32> = Vec::new();
        for y in 0..fh - 1 {
            let (mut s, mut n) = (0.0f32, 0u32);
            for z in 1..d - 1 {
                let c = sm.get(edge, y, z).expect("side cell exists");
                let dist = lab_of(c).distance(base_lab);
                side_max = side_max.max(dist);
                assert!(
                    dist <= 0.05,
                    "panel side cell at ({edge},{y},{z}) is {dist:.3} from the material (rgb {:?})",
                    c.0
                );
                s += lab_of(c).l;
                n += 1;
            }
            row_means.push(s / n as f32);
        }
        let spread = row_means.iter().cloned().fold(0.0f32, f32::max)
            - row_means.iter().cloned().fold(1.0f32, f32::min);
        assert!(
            spread <= 0.03,
            "panel side row-mean L spread {spread:.3} must be grain only, no ramp"
        );
    }
    // Back cells (rim row excluded): at the material color,
    // row-mean L flat (grain only).
    let mut back_max = 0.0f32;
    let mut back_row_means: Vec<f32> = Vec::new();
    for y in 0..fh - 1 {
        let (mut s, mut n) = (0.0f32, 0u32);
        for x in 0..w {
            let c = sm.get(x, y, 0).expect("back cell exists");
            back_max = back_max.max(lab_of(c).distance(base_lab));
            s += lab_of(c).l;
            n += 1;
        }
        back_row_means.push(s / n as f32);
    }
    assert!(
        back_max <= 0.05,
        "back cell is {back_max:.3} from the material (budget: weighted-median anchor + grain)"
    );
    let back_spread = back_row_means.iter().cloned().fold(0.0f32, f32::max)
        - back_row_means.iter().cloned().fold(1.0f32, f32::min);
    assert!(
        back_spread <= 0.03,
        "back row-mean L spread {back_spread:.3} must be grain only, no ramp"
    );
    // The front plane keeps the baked gradient VERBATIM — it is
    // the texture, and the engine will light it.
    let bot_px = *img.get_pixel(16, 27);
    assert_eq!(
        sm.get(16, 0, d - 1),
        Some(bot_px),
        "front bottom cell is the sprite pixel verbatim"
    );
    assert!(
        lab_of(bot_px).l < base_lab.l - 0.05,
        "the front's baked gradient must still be in the texture (bottom L {:.3} vs material {:.3})",
        lab_of(bot_px).l,
        base_lab.l
    );
    assert_eq!(
        sm.get(16, fh - 1, d - 1),
        Some(*img.get_pixel(16, 8)),
        "front top cell is the sprite pixel verbatim"
    );
    println!(
        "albedo synthetic: side max distance {side_max:.3}, back max distance {back_max:.3}, back L spread {back_spread:.3}"
    );
}

#[test]
fn stage_g_unsegmentable_sprites_return_none() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    // f_chair: 23% opaque cutout illustration — no box-like reading.
    let chair = load(&ts, "f_chair");
    assert!(segment(&chair).is_none(), "f_chair must not segment");
    assert!(segment_for("f_chair", &chair).is_none(), "f_chair must not segment via segment_for");
    // t_grass: flat top-down ground texture — no front face at all.
    let grass = load(&ts, "t_grass");
    assert!(segment(&grass).is_none(), "t_grass must not segment");
}
