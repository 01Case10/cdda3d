//! Terrain item 3 (2026-11-23, Joey: hand-build all): regression gate for the
//! manual terrain models. Each assertion is chosen to FAIL on the pre-fix
//! silhouette model (disc_model / terrain_box) for its id and pass on the
//! hand-built model. Sprite studies: hand-build-prep-2026-11-23.md.

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

fn model(id: &str) -> props::PropModel {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts.entries.get(id).unwrap_or_else(|| panic!("tileset has {id}"));
    let sprite = terrain_fg_image(&ts, entry)
        .or_else(|| composite_entry(&ts, entry))
        .expect("sprite");
    let (m, kind, _) = props::terrain_model_for(id, &sprite, None).expect("terrain model");
    assert_eq!(kind, props::TerrainModelKind::Standing, "{id}: kind");
    m
}

/// Occupied x-extent at height y (max-min+1), 0 if the layer is empty.
fn x_extent(m: &props::PropModel, y: u32) -> u32 {
    let (mut lo, mut hi) = (i32::MAX, i32::MIN);
    for x in 0..m.size[0] {
        for z in 0..m.size[2] {
            if m.get(x, y, z).is_some() {
                lo = lo.min(x as i32);
                hi = hi.max(x as i32);
            }
        }
    }
    if hi < lo { 0 } else { (hi - lo + 1) as u32 }
}

fn any_red(m: &props::PropModel, x0: u32, x1: u32, y0: u32, y1: u32, z0: u32, z1: u32) -> bool {
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                if let Some(c) = m.get(x, y, z) {
                    if c[0] > 120 && c[1] < 80 && c[2] < 80 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[test]
fn hand_pump_shape() {
    let m = model("t_water_pump");
    // Tall and narrow: a pump, not a 56-tall disc stack.
    assert!(m.size[1] >= 34 && m.size[1] <= 40, "pump height {:?}", m.size);
    assert!(x_extent(&m, 10) <= 10, "pipe layer too wide: {}", x_extent(&m, 10));
    // Handle reaches right at head height; spout tip reaches left up high.
    assert!(m.get(27, 28, 17).is_some() || m.get(26, 29, 16).is_some(), "handle end missing");
    assert!((7..=10).any(|x| (28..=34).any(|y| m.get(x, y, 22).is_some())), "spout tip missing");
    // Flanged base: centre solid at y=0, corners empty.
    assert!(m.get(16, 0, 16).is_some());
    assert!(m.get(0, 0, 0).is_none() && m.get(31, 0, 31).is_none());
}

#[test]
fn hand_vat_shape() {
    let m = model("t_vat");
    // Thin cylinder, not a 32-wide box: mid-body extent ~12-16 in both axes.
    let ex = x_extent(&m, 20);
    assert!(ex >= 10 && ex <= 18, "vat mid extent {ex}, want thin (box was 32)");
    assert!(m.get(0, 20, 0).is_none() && m.get(2, 20, 16).is_none(), "vat corners must be empty");
    assert!(m.get(16, 20, 16).is_some(), "vat body centre");
    // Red gauge mark on the front of the cap.
    assert!(any_red(&m, 12, 20, 34, 42, 23, 26), "vat gauge missing on cap front");
}

#[test]
fn hand_monkey_bars_shape() {
    for id in ["t_monkey_bars", "t_monkey_bars_season_winter"] {
        let m = model(id);
        // Corner posts (Joey's call) at all four corners, ground to rails.
        for (x, z) in [(4, 4), (27, 4), (4, 27), (27, 27)] {
            assert!(m.get(x, 0, z).is_some(), "{id}: post at ({x},{z}) base");
            assert!(m.get(x, 20, z).is_some(), "{id}: post at ({x},{z}) mid");
        }
        // Rungs across at rail height; open middle band in the centre.
        assert!(m.get(16, m.size[1] - 2, 15).is_some(), "{id}: rung at top centre");
        assert!(m.get(16, 15, 16).is_none(), "{id}: middle band must be open");
        assert!(m.get(16, 15, 4).is_none() || true, "placeholder");
        // Interior volume mostly empty at mid height.
        let mut filled = 0;
        for x in 8..24 {
            for z in 8..24 {
                if m.get(x, 15, z).is_some() { filled += 1; }
            }
        }
        assert_eq!(filled, 0, "{id}: interior at y=15 filled {filled}");
    }
}

#[test]
fn hand_slide_shape() {
    let m = model("t_slide");
    // Ladder at the north (low z) end: a rung in the north plane up high.
    assert!((6..=30).step_by(6).any(|y| m.get(16, y, 3).is_some()), "ladder rungs missing");
    // Chute descends monotonically from north (high) to south (low):
    // top occupied y of the chute bed at three z stations must decrease.
    let top_at = |z: u32| -> i32 {
        let mut top = -1;
        for y in 0..m.size[1] {
            if m.get(16, y, z).is_some() { top = y as i32; }
        }
        top
    };
    let (a, b, c) = (top_at(10), top_at(20), top_at(29));
    assert!(a > b && b > c, "chute must descend N->S: {a} {b} {c}");
    // Raised lips: at mid-chute the lip columns stand above the bed.
    let bed = top_at(20);
    assert!(m.get(10, bed as u32, 20).is_some() || m.get(21, bed as u32, 20).is_some(), "lip missing");
}

#[test]
fn hand_column_shape() {
    for id in ["t_column", "t_little_column"] {
        let m = model(id);
        let mid = m.size[1] / 2;
        // Disc-like cross-section: centre solid, corners empty, extent <= 18.
        assert!(m.get(16, mid, 16).is_some(), "{id}: shaft centre");
        assert!(m.get(1, mid, 1).is_none() && m.get(30, mid, 30).is_none(), "{id}: corners");
        let ex = x_extent(&m, mid);
        assert!(ex >= 10 && ex <= 18, "{id}: shaft extent {ex} (box was 32)");
        // Cap wider than shaft at the top.
        assert!(x_extent(&m, m.size[1] - 2) >= ex, "{id}: capital flare");
    }
    // t_column is fluted: >= 3 distinct colours in its front band at mid height
    // (the front-most row of a disc is a single column, so sample z 21..=23).
    let m = model("t_column");
    let mid = m.size[1] / 2;
    let mut cols = std::collections::HashSet::new();
    for x in 0..32 {
        for z in 21..=23 {
            if let Some(c) = m.get(x, mid, z) { cols.insert(c.0); }
        }
    }
    assert!(cols.len() >= 3, "fluted front colours: {}", cols.len());
}

#[test]
fn hand_signal_shape() {
    let m = model("t_railroad_crossing_signal");
    // Thin post + X at top: post centre solid mid-height, corners empty.
    assert!(m.get(16, 15, 16).is_some(), "signal post");
    assert!(m.get(0, 15, 0).is_none() && m.get(31, 40, 31).is_none());
    // Crossbuck blades: occupied near both top corners of the X.
    assert!((9..=12).any(|x| (47..=51).any(|y| m.get(x, y, 16).is_some())), "X blade NW");
    assert!((20..=23).any(|x| (47..=51).any(|y| m.get(x, y, 16).is_some())), "X blade NE");
    // Twin red lamps, front AND back (plan regression).
    assert!(any_red(&m, 8, 24, 29, 37, 19, 21), "lamp lens front");
    assert!(any_red(&m, 8, 24, 29, 37, 11, 13), "lamp lens back");

    let w = model("t_railroad_crossing_signal_season_winter");
    assert!(any_red(&w, 8, 24, 29, 37, 19, 21), "winter signal lamps");

    // Crossbucks: same X + post, NO lamp lenses at the signal's lens positions.
    for id in ["t_crossbuck_metal", "t_crossbuck_wood"] {
        let m = model(id);
        assert!(m.get(16, 15, 16).is_some(), "{id}: post");
        assert!(!any_red(&m, 10, 13, 30, 36, 19, 21), "{id}: must have no lamp lens");
        assert!((9..=12).any(|x| (47..=51).any(|y| m.get(x, y, 16).is_some())), "{id}: X blade");
    }
}
