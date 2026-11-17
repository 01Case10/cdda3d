//! Stage G.3 rework gate: view-faithful prop models (pipeline
//! §24). The sprite's VIEW decides the construction:
//!
//! * view pins are the ground truth (the heuristic alone gets
//!   every pinned case wrong — that is why the pins exist);
//! * f_standing_tank (EndOn): a horizontal cylinder — disc
//!   cross-section at every depth step, the sprite's cap region
//!   mapped verbatim onto the front disc, the hatch fitting on
//!   top at the front end, saddle stands at both ends;
//! * the bed chain (TopDown, framed): back plane 100% frame
//!   material (the headboard slab occludes the mattress), top
//!   face verbatim, headboard above the mattress;
//! * f_slab (TopDown, terrace): column heights are a function
//!   of chamfer distance only (the sprite is NOT 90°-rotationally
//!   symmetric, so the ring-function assertion replaces the
//!   rotation invariant), top face verbatim;
//! * every builder byte-identical across two runs.

use image::{Rgba, RgbaImage};
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

fn load(sprite_id: &str) -> RgbaImage {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts
        .entries
        .get(sprite_id)
        .unwrap_or_else(|| panic!("tileset has {sprite_id}"));
    composite_entry(&ts, entry).expect("sprite composites")
}

fn structure(px: Rgba<u8>) -> bool {
    px[3] >= 8 && !(px[0] == 0 && px[1] == 0 && px[2] == 0 && px[3] < 254)
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

/// Independent chamfer distance (BFS) over the sprite's
/// structure mask, for the terrace assertions.
fn chamfer(sprite: &RgbaImage) -> Vec<u32> {
    let (w, h) = (sprite.width(), sprite.height());
    let sil: Vec<bool> = sprite.pixels().map(|p| structure(*p)).collect();
    let idx = |x: u32, y: u32| (y * w + x) as usize;
    let mut dist = vec![0u32; (w * h) as usize];
    let mut q: std::collections::VecDeque<(u32, u32)> = Default::default();
    for y in 0..h {
        for x in 0..w {
            if !sil[idx(x, y)] {
                q.push_back((x, y));
            }
        }
    }
    while let Some((x, y)) = q.pop_front() {
        let d = dist[idx(x, y)];
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx >= w || ny >= h {
                continue;
            }
            let i = idx(nx, ny);
            if sil[i] && dist[i] == 0 {
                dist[i] = d + 1;
                q.push_back((nx, ny));
            }
        }
    }
    dist
}

#[test]
fn view_pins_are_the_ground_truth() {
    let tank = load("f_standing_tank");
    let bed = load("f_bed");
    let slab = load("f_slab");
    let counter = load("f_counter");
    // Pins.
    assert_eq!(view_for(Some("f_standing_tank"), &tank), SpriteView::EndOn);
    assert_eq!(view_for(Some("f_storage_tank"), &tank), SpriteView::EndOn);
    for id in ["f_bed", "f_bunkbed", "f_bunkbed_down", "f_triple_bunkbed"] {
        assert_eq!(view_for(Some(id), &bed), SpriteView::TopDown, "{id}");
    }
    assert_eq!(view_for(Some("f_slab"), &slab), SpriteView::TopDown);
    // Heuristic alone: the tank and the bed both segment with
    // shallow top regions, so it proposes Front for both — the
    // documented reason the pins exist.
    assert_eq!(view_for(None, &tank), SpriteView::Front);
    assert_eq!(view_for(None, &bed), SpriteView::Front);
    // Ordinary furniture stays Front, pinned or not.
    assert_eq!(view_for(Some("f_counter"), &counter), SpriteView::Front);
    assert_eq!(view_for(None, &counter), SpriteView::Front);
    // The router still serves the Front path.
    assert!(model_for("f_counter", &counter).is_some());
    assert!(model_for("f_table", &load("f_table")).is_some());
}

#[test]
fn tank_is_a_horizontal_cylinder() {
    let sprite = load("f_standing_tank");
    let fit = endon_tank_fit("f_standing_tank", &sprite).expect("tank fits");
    // Cap diameter = the silhouette width under the pipeline's
    // structure test (recomputed here): 30 for this sprite.
    let (mut ex0, mut ex1) = (sprite.width() - 1, 0u32);
    for (x, _y, px) in sprite.enumerate_pixels() {
        if structure(*px) {
            ex0 = ex0.min(x);
            ex1 = ex1.max(x);
        }
    }
    assert_eq!(fit.diameter, ex1 - ex0 + 1, "cap diameter = silhouette width");
    assert_eq!(fit.length, fit.diameter, "unpinned length = diameter");
    let m = model_for("f_standing_tank", &sprite).expect("tank model");
    assert_same_model(&m, &model_for("f_standing_tank", &sprite).unwrap());
    let (mw, mh, ml) = (m.size[0], m.size[1], m.size[2]);
    assert_eq!(ml, fit.length);
    assert_eq!(mh, fit.neck[3] + 1, "model height reaches the neck top");
    // Geometry recomputed from the fit: disc predicate, neck
    // box, saddle stands (fill below the disc's underside).
    let cy = fit.leg_h as f32 + (fit.diameter - 1) as f32 / 2.0;
    let r = fit.diameter as f32 / 2.0;
    let in_disc = |x: u32, y: u32| {
        let dx = x as f32 - fit.center_x;
        let dy = y as f32 - cy;
        dx * dx + dy * dy <= r * r
    };
    let disc_bottom = |x: u32| (0..mh).find(|&y| in_disc(x, y));
    for z in 0..ml {
        for y in 0..mh {
            for x in 0..mw {
                let in_neck = fit.neck_z.contains(&z)
                    && x >= fit.neck[0]
                    && x <= fit.neck[1]
                    && y >= fit.neck[2]
                    && y <= fit.neck[3];
                let in_stand = fit.stand_z.iter().any(|zr| zr.contains(&z))
                    && fit.stand_x.contains(&x)
                    && disc_bottom(x).map_or(false, |yb| y < yb);
                let expect = in_disc(x, y) || in_neck || in_stand;
                assert_eq!(
                    m.get(x, y, z).is_some(),
                    expect,
                    "tank shape at {x},{y},{z} (disc {} neck {} stand {})",
                    in_disc(x, y),
                    in_neck,
                    in_stand
                );
            }
        }
    }
    // The hatch fitting crowns the neck at the front end.
    for x in fit.fitting_x.clone() {
        assert_eq!(
            m.get(x, fit.neck[3], ml - 1),
            Some(fit.fitting),
            "fitting color at neck top x={x}"
        );
    }
    // Stands touch the ground at both ends.
    let mid_x = fit.center_x as u32;
    assert_eq!(m.get(mid_x, 0, 0), Some(fit.base), "back stand");
    assert_eq!(m.get(mid_x, 0, ml - 1), Some(fit.base), "front stand");
    // Front disc == the sprite's cap region mapped onto the
    // disc (exhaustive over disc cells).
    let cap_h = fit.cap_rows.end - fit.cap_rows.start;
    let cap_bot = fit.cap_rows.end - 1;
    for y in 0..mh {
        for x in 0..mw {
            if !in_disc(x, y) {
                continue;
            }
            let sy = cap_bot
                - (((y - fit.leg_h) as f32 * (cap_h - 1) as f32
                    / (fit.diameter - 1) as f32)
                    .round() as u32)
                    .min(cap_h - 1);
            let px = *sprite.get_pixel(x, sy);
            let expect = if structure(px) { px } else { fit.barrel };
            assert_eq!(m.get(x, y, ml - 1), Some(expect), "front disc {x},{y}");
        }
    }
    // Barrel interior is the cap material; the back face too.
    assert_eq!(m.get(mid_x, cy as u32, ml / 2), Some(fit.barrel));
    assert_eq!(m.get(mid_x, cy as u32, 0), Some(fit.barrel));
}

#[test]
fn bed_back_is_all_frame_and_top_is_verbatim() {
    let sprite = load("f_bed");
    // f_bed is the sprite the whole bed chain resolves to.
    let fit = bed_fit("f_bunkbed", &sprite).expect("bed fits");
    assert!(
        fit.headboard_top > fit.mattress_top,
        "headboard rises above the mattress"
    );
    let m = model_for("f_bunkbed", &sprite).expect("bed model");
    assert_same_model(&m, &model_for("f_bunkbed", &sprite).unwrap());
    let (mw, mh, md) = (m.size[0], m.size[1], m.size[2]);
    assert_eq!(mh, fit.headboard_top + 1);
    // The headboard slab is solid frame material over its whole
    // extent — in particular the BACK plane (z = 0) shows zero
    // mattress: every solid cell on it is the frame color.
    for y in 0..mh {
        for x in 0..mw {
            if let Some(c) = m.get(x, y, 0) {
                assert_eq!(c, fit.frame, "back plane cell {x},{y} is frame");
            }
        }
    }
    for z in fit.head_rows.clone() {
        for x in fit.head_x.0..=fit.head_x.1 {
            for y in 0..=fit.headboard_top {
                assert_eq!(m.get(x, y, z), Some(fit.frame), "slab {x},{y},{z}");
            }
        }
    }
    assert_ne!(fit.frame, fit.mattress, "frame and mattress differ");
    // Top face == the sprite verbatim, exhaustive over the
    // silhouette outside the headboard strip: the topmost solid
    // cell of every plan column carries the sprite's pixel.
    for z in 0..md {
        if fit.head_rows.contains(&z) {
            continue;
        }
        for x in 0..mw {
            let px = *sprite.get_pixel(x, z);
            if !structure(px) {
                continue;
            }
            let top = (0..mh).rev().find(|&y| m.get(x, y, z).is_some());
            let top = top.unwrap_or_else(|| panic!("column {x},{z} is filled"));
            assert_eq!(m.get(x, top, z), Some(px), "top verbatim {x},{z}");
        }
    }
    // The mattress top sits one cell proud of the frame rails.
    assert_eq!(fit.mattress_top, fit.frame_top + 1);
}

#[test]
fn slab_heights_follow_chamfer_rings() {
    let sprite = load("f_slab");
    let fit = terrace_fit(&sprite).expect("slab fits");
    assert!(fit.levels >= 2 && fit.levels <= 4, "levels {:?}", fit.levels);
    let m = model_for("f_slab", &sprite).expect("slab model");
    assert_same_model(&m, &model_for("f_slab", &sprite).unwrap());
    let (mw, mh, md) = (m.size[0], m.size[1], m.size[2]);
    assert_eq!(mh, fit.levels * fit.step, "max height = levels x step");
    assert!(
        mh <= mw / 4 + fit.step,
        "terrace height ~1/4 width: {mh} vs width {mw}"
    );
    // Heights are a function of chamfer distance only; the top
    // cell is the sprite pixel verbatim, step sides the edge
    // material. (The f_slab sprite is not 90°-rotationally
    // symmetric, so this replaces the rotation invariant.)
    let dist = chamfer(&sprite);
    for z in 0..md {
        for x in 0..mw {
            let px = *sprite.get_pixel(x, z);
            if !structure(px) {
                for y in 0..mh {
                    assert_eq!(m.get(x, y, z), None, "empty column {x},{z}");
                }
                continue;
            }
            let col_h = dist[(z * mw + x) as usize].min(fit.levels) * fit.step;
            assert!(col_h >= 1);
            for y in 0..mh {
                let expect = if y < col_h - 1 {
                    Some(fit.edge)
                } else if y == col_h - 1 {
                    Some(px)
                } else {
                    None
                };
                assert_eq!(m.get(x, y, z), expect, "terrace cell {x},{y},{z}");
            }
        }
    }
    // Monotonic non-decreasing toward the center: adjacent
    // columns differ by at most one step.
    for z in 0..md {
        for x in 0..mw {
            let h_here = (0..mh).filter(|&y| m.get(x, y, z).is_some()).count() as u32;
            for (nx, nz) in [(x + 1, z), (x, z + 1)] {
                if nx >= mw || nz >= md {
                    continue;
                }
                let h_there =
                    (0..mh).filter(|&y| m.get(nx, y, nz).is_some()).count() as i32;
                assert!(
                    (h_there - h_here as i32).abs() <= fit.step as i32,
                    "step between {x},{z} and {nx},{nz}"
                );
            }
        }
    }
}
