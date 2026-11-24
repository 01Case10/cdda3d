//! Stage-G S7 gates: synthesized frame tubes on thin
//! vehicles (Joey approved the invention 2026-11-24 —
//! the bicycle's xlframe art never resolves, so without
//! tubes the wheels and saddle are disconnected slices).
//! Synthetic sprites with known alpha bboxes; the tube
//! assertions target columns only a tube can fill.

use voxelizer::vehicle::{compose_vehicle, PartSpot, VehicleLayout};

fn spot(base: &str) -> PartSpot {
    PartSpot {
        base: base.to_string(),
        roof: false,
        open: false,
        broken: false,
    }
}

/// Wheel sprite: opaque block at sprite cols 4..28,
/// rows 8..24 (from top) — bbox centre (16, 16), so the
/// hub lands at tile lx 16, height ~14.1 units.
/// Pedals sprite: opaque at cols 10..22, rows 12..28 —
/// bbox centre (16, 20), bracket at lx 16, ~10.3 units.
fn sprites(name: &str) -> Option<image::RgbaImage> {
    let mut img = image::RgbaImage::new(32, 32);
    let (x0, x1, y0, y1) = if name.contains("wheel") {
        (4, 28, 8, 24)
    } else if name.contains("pedals") {
        (10, 22, 12, 28)
    } else if name.contains("saddle") || name.contains("xlframe") {
        (12, 20, 4, 28)
    } else {
        return None;
    };
    for y in y0..y1 {
        for x in x0..x1 {
            img.put_pixel(x, y, image::Rgba([90, 90, 90, 255]));
        }
    }
    Some(img)
}

fn bike_layout() -> VehicleLayout {
    let mut layout = VehicleLayout::default();
    layout
        .tiles
        .insert((0, 0, 0), vec![spot("xlframe"), spot("saddle"), spot("foot_pedals")]);
    layout.tiles.insert((1, 0, 0), vec![spot("wheel_bicycle")]);
    layout.tiles.insert((-1, 0, 0), vec![spot("wheel_bicycle")]);
    layout
}

fn voxel(
    models: &[((i32, i32, i32), voxelizer::PropModel)],
    tile: (i32, i32),
    x: u32,
    y: u32,
    z: u32,
) -> bool {
    let m = &models
        .iter()
        .find(|(k, _)| *k == (tile.0, tile.1, 0))
        .unwrap_or_else(|| panic!("no model for tile {tile:?}"))
        .1;
    m.get(x, y, z).is_some()
}

#[test]
fn chainstay_tube_fills_the_gap_column() {
    let models = compose_vehicle(&bike_layout(), &sprites);
    // Rear hub at global gx -16 (tile -1, lx 16), height
    // ~14.1; bracket at gx 16, ~10.3. The line's midpoint
    // is tile 0's lx 0 at ~12.2 units = engine rows
    // ~23..26 — a column the pedals sprite (lx 10..22)
    // and saddle sprite (lx 12..20) never touch, at a
    // height above both. Only the tube fills it.
    let hit = (20..30).any(|y| voxel(&models, (0, 0), 0, y, 16));
    assert!(hit, "no tube voxel on the rear-hub-to-bracket line");
}

#[test]
fn top_tube_fills_the_high_midpoint() {
    let models = compose_vehicle(&bike_layout(), &sprites);
    // Top tube at 25 units = engine rows ~48..52,
    // probed at tile 0's lx 24 — outside every sprite on
    // the tile (saddle/xlframe cols 12..20, pedals
    // 10..22) but under the tube, which spans from
    // bracket-7 to bracket+9 (lx 9..25).
    let hit = (46..55).any(|y| voxel(&models, (0, 0), 24, y, 16));
    assert!(hit, "no top-tube voxel over the bracket");
}

#[test]
fn tubes_stay_in_the_centre_plane() {
    let models = compose_vehicle(&bike_layout(), &sprites);
    // Nothing the tubes add may leave rows 15..18: check
    // the gap column at every other depth row.
    for z in 0..32u32 {
        if (15..18).contains(&z) {
            continue;
        }
        let any = (0..96u32).any(|y| voxel(&models, (0, 0), 0, y, z));
        assert!(!any, "tube leaked out of the centre plane at row {z}");
    }
}
