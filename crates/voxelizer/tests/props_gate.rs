//! Phase 5 rework Stage G gate (generation side): sprite
//! segmentation + box/column prop models on NAMED real UltiCa
//! furniture sprites.
//!
//! The four chosen sprites were probed before this gate was written
//! (row coverage / mean-L / seam-edge profiles): each has one clean
//! top/front seam. Expected boundary bands below are the probed seam
//! rows, widened by a couple of rows for the apron/trim ambiguity —
//! they are sanity bands, not fitted values: the gate's real teeth
//! are the exactness assertions (box front face == sprite front
//! pixels, column front projection == front silhouette) and the
//! `None` cases.
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

/// (id, expected boundary band) — probed seams:
/// f_counter 20, f_bookcase 27, f_table 19-20, f_fridge 17-19.
const PROPS: [(&str, std::ops::RangeInclusive<u32>); 4] = [
    ("f_counter", 18..=22),
    ("f_bookcase", 25..=29),
    ("f_table", 18..=21),
    ("f_fridge", 15..=20),
];

fn load(ts: &tileset::Tileset, id: &str) -> image::RgbaImage {
    let entry = ts.entries.get(id).unwrap_or_else(|| panic!("tileset has {id}"));
    composite_entry(ts, entry).unwrap_or_else(|| panic!("{id} composites"))
}

#[test]
fn stage_g_props_segment_and_project_exactly() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (id, band) in &PROPS {
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
            seg.confidence >= 0.45,
            "{id}: confidence {:.2} below the reporting floor",
            seg.confidence
        );
        // Determinism: segmentation is a pure function of the sprite.
        assert_eq!(seg, segment(&sprite).unwrap(), "{id}: segmentation must be deterministic");

        let bm = box_model(&sprite, &seg);
        let cm = column_model(&sprite, &seg);
        assert_eq!(bm, box_model(&sprite, &seg), "{id}: box model deterministic");
        assert_eq!(cm, column_model(&sprite, &seg), "{id}: column model deterministic");

        let (w, fh, d) = (bm.size[0], bm.size[1], bm.size[2]);
        assert_eq!(fh, seg.front_rows.end - seg.front_rows.start, "{id}: model height = front height");
        assert_eq!(cm.size, bm.size, "{id}: box and column share the grid");
        let bot = seg.front_rows.end - 1;
        // Box front face: every opaque sprite front pixel inside the
        // box's x-extent appears verbatim on the front face. (Sampled
        // on a stride to keep the gate fast; the column check below
        // is exhaustive.)
        for y in 0..fh {
            let r = bot - y;
            for x in (0..w).step_by(2) {
                let px = *sprite.get_pixel(x, r);
                if px[3] >= 128 {
                    // x is inside the extent by construction (it is
                    // itself an opaque front pixel).
                    assert_eq!(
                        bm.get(x, y, d - 1),
                        Some(px),
                        "{id}: box front face mismatch at sprite ({x},{r})"
                    );
                }
            }
        }
        // Column model front projection == front region silhouette,
        // exactly, for every cell.
        for x in 0..w {
            for y in 0..fh {
                let r = bot - y;
                let want = sprite.get_pixel(x, r)[3] >= 128;
                assert_eq!(
                    cm.get(x, y, d - 1).is_some(),
                    want,
                    "{id}: column front projection mismatch at ({x},{y})"
                );
                if want {
                    assert_eq!(
                        cm.get(x, y, d - 1),
                        Some(*sprite.get_pixel(x, r)),
                        "{id}: column front color mismatch at ({x},{y})"
                    );
                }
            }
        }
        // The box is a solid cuboid over the front x-extent; the
        // column shell is strictly sparser.
        assert!(
            bm.solid_count() > cm.solid_count(),
            "{id}: box {} should exceed column shell {}",
            bm.solid_count(),
            cm.solid_count()
        );
    }
}

#[test]
fn stage_g_unsegmentable_sprites_return_none() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    // f_chair: 23% opaque cutout illustration — no box-like reading.
    let chair = load(&ts, "f_chair");
    assert!(segment(&chair).is_none(), "f_chair must not segment");
    // t_grass: flat top-down ground texture — no front face at all.
    let grass = load(&ts, "t_grass");
    assert!(segment(&grass).is_none(), "t_grass must not segment");
}
