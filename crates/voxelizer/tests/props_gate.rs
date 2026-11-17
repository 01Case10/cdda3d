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
        if sprite.get_pixel(x, bot - y)[3] < 128 {
            return false; // not in the front silhouette
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
