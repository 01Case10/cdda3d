//! Stage-G S4 gates: the pickup bed profile and the
//! thin-vehicle (bicycle) treatment. Synthetic opaque
//! sprites; the assertions are on composed geometry.

use voxelizer::vehicle::{compose_vehicle, PartSpot, VehicleLayout};

fn spot(base: &str) -> PartSpot {
    PartSpot {
        base: base.to_string(),
        roof: false,
        open: false,
        broken: false,
    }
}

fn sprites(_: &str) -> Option<image::RgbaImage> {
    Some(image::RgbaImage::from_pixel(
        32,
        32,
        image::Rgba([120, 120, 120, 255]),
    ))
}

fn top_at(models: &[((i32, i32, i32), voxelizer::PropModel)], tile: (i32, i32), lx: u32, ly: u32) -> i32 {
    let m = &models
        .iter()
        .find(|(k, _)| *k == (tile.0, tile.1, 0))
        .unwrap_or_else(|| panic!("no model for tile {tile:?}"))
        .1;
    (0..96i32)
        .rev()
        .find(|&y| m.get(lx, y as u32, ly).is_some())
        .unwrap_or(-1)
}

#[test]
fn bed_interior_is_floor_rims_are_walls() {
    // Two cargo tiles in a row, a frame tile ahead of
    // them: both cargo tiles form the bed.
    let mut layout = VehicleLayout::default();
    layout.tiles.insert((0, 0, 0), vec![spot("cargo_space")]);
    layout.tiles.insert((1, 0, 0), vec![spot("cargo_space")]);
    layout.tiles.insert((2, 0, 0), vec![spot("frame")]);
    let models = compose_vehicle(&layout, &sprites);
    // Interior of the rear cargo tile stops at the bed
    // floor (14 mockup units = engine row 28).
    assert_eq!(top_at(&models, (0, 0), 16, 16), 27, "bed interior top");
    // Its rear edge faces outside the vehicle: rim at
    // panel height (rear-tile deck lerp starts at 21 ->
    // engine row 42).
    assert!(top_at(&models, (0, 0), 1, 16) >= 41, "rear rim top");
    // The front cargo tile's rear edge faces another bed
    // tile: no rim there.
    assert_eq!(top_at(&models, (1, 0), 1, 16), 27, "shared edge has no rim");
    // Its front edge faces the frame tile: rim.
    assert!(top_at(&models, (1, 0), 30, 16) >= 41, "front rim top");
}

#[test]
fn thin_vehicle_parts_stay_in_the_centre_band() {
    let mut layout = VehicleLayout::default();
    layout.tiles.insert((0, 0, 0), vec![spot("xlframe")]);
    layout.tiles.insert((1, 0, 0), vec![spot("wheel_bicycle")]);
    layout.tiles.insert((-1, 0, 0), vec![spot("wheel_bicycle")]);
    let models = compose_vehicle(&layout, &sprites);
    // Frame: a side elevation like every other part on a
    // thin vehicle — centre plane rows 15..18 only.
    let frame = &models
        .iter()
        .find(|(k, _)| *k == (0, 0, 0))
        .expect("frame model")
        .1;
    for z in 0..32u32 {
        let any = (0..32u32).any(|x| (0..96u32).any(|y| frame.get(x, y, z).is_some()));
        assert_eq!(any, (15..18).contains(&z), "frame row {z} occupancy");
    }
    // Wheel: a profile disc — centre rows only, spanning
    // most of the tile's height, touching near the ground.
    let wheel = &models
        .iter()
        .find(|(k, _)| *k == (1, 0, 0))
        .expect("wheel model")
        .1;
    for z in 0..32u32 {
        let any = (0..32u32).any(|x| (0..96u32).any(|y| wheel.get(x, y, z).is_some()));
        assert_eq!(any, (15..18).contains(&z), "wheel row {z} occupancy");
    }
    let top = (0..96).rev().find(|&y| {
        (0..32u32).any(|x| (0..32u32).any(|z| wheel.get(x, y, z).is_some()))
    });
    assert!(top.expect("wheel has voxels") >= 55, "wheel reaches near full height");
    let bottom = (0..96).find(|&y| {
        (0..32u32).any(|x| (0..32u32).any(|z| wheel.get(x, y, z).is_some()))
    });
    assert!(bottom.expect("wheel has voxels") <= 3, "wheel reaches the ground");
}
