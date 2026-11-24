//! Vehicle gallery (one-off showcase, 2026-11-24, Joey's
//! request): stamp three real CDDA vehicles into the engine's
//! sample scene and capture them, so the current vehicle render
//! model can be judged visually.
//!
//! The models carry the REAL part lists of the definitions in
//! `fork-src/cdda/data/json/vehicles/` (`car`, `pickup`,
//! `bicycle`), `copy-from` resolved, in definition order:
//! (mount x, mount y, vpart wire id, roof flag) per part. The
//! roof flag is the game's own — the vpart defs carrying the
//! ROOF flag are hdroof/roof/roof_cloth/roof_wood. Wire ids are
//! gallery-local: index+1 into `NAMES`, and the scene's
//! `vpart_names` maps them back to the base string ids, which
//! is exactly the TileDefs contract the fork export follows
//! (base id only — the `#variant` is not on the wire, so
//! variant sprites stay unreachable; the engine resolves the
//! base material `vp_<name>`).
//!
//! Captures render WITH the voxelizer materials (the "(b)"
//! wiring): `target/gallery_assets` from the prop gallery run
//! (the voxelizer runs first when it is missing). Facing is 0
//! degrees (front/+x east), so absolute positions are
//! pivot + mount verbatim.
//!
//! Not a gate; a visual check. Output PNGs land in
//! `captures/vehicle_*_textured.png`.

use std::collections::HashMap;
use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::frame::VehicleScene;
use vehicles::{PartKey, PartModel, VehicleModel};

// wire ids: index+1 into NAMES
const NAMES: &[&str] = &["alternator_car", "alternator_truck", "basketsm", "battery_car", "cargo_space", "controls", "dashboard", "door", "door_trunk", "engine_inline4", "engine_v6", "foot_pedals", "frame", "halfboard", "headlight", "horn_bicycle", "horn_car", "integrated_heater_small", "muffler", "reclining_seat", "roof", "saddle", "seat", "seat_back", "seatbelt", "stereo", "tank", "trunk", "vehicle_alarm", "wheel", "wheel_bicycle", "wheel_mount_light", "wheel_mount_light_steerable", "wheel_mount_medium", "wheel_mount_medium_steerable", "windshield", "xlframe"];
const CAR: &[(i32, i32, u32, bool)] = &[(0, 0, 13, false), (0, 0, 20, false), (0, 0, 25, false), (0, 0, 6, false), (0, 0, 18, false), (0, 0, 7, false), (0, 0, 29, false), (0, 0, 26, false), (0, 0, 17, false), (0, 0, 21, true), (0, -1, 13, false), (0, -1, 8, false), (0, 1, 13, false), (0, 1, 20, false), (0, 1, 25, false), (0, 1, 21, true), (0, 2, 13, false), (0, 2, 8, false), (1, -1, 13, false), (1, -1, 36, false), (1, -1, 35, false), (1, -1, 30, false), (1, 0, 13, false), (1, 0, 36, false), (1, 1, 13, false), (1, 1, 36, false), (1, 2, 13, false), (1, 2, 36, false), (1, 2, 35, false), (1, 2, 30, false), (2, -1, 13, false), (2, -1, 14, false), (2, -1, 15, false), (2, 0, 13, false), (2, 0, 14, false), (2, 0, 4, false), (2, 1, 13, false), (2, 1, 14, false), (2, 2, 13, false), (2, 2, 14, false), (2, 2, 15, false), (-1, -1, 13, false), (-1, -1, 8, false), (-1, 0, 13, false), (-1, 0, 24, false), (-1, 0, 25, false), (-1, 0, 21, true), (-1, 1, 13, false), (-1, 1, 24, false), (-1, 1, 25, false), (-1, 1, 21, true), (-1, 2, 13, false), (-1, 2, 8, false), (-2, -1, 13, false), (-2, -1, 14, false), (-2, -1, 34, false), (-2, 0, 13, false), (-2, 0, 28, false), (-2, 0, 19, false), (-2, 0, 21, true), (-2, 1, 13, false), (-2, 1, 28, false), (-2, 1, 21, true), (-2, 2, 13, false), (-2, 2, 14, false), (-2, 2, 34, false), (-2, 2, 30, false), (-3, -1, 13, false), (-3, -1, 14, false), (-3, 0, 13, false), (-3, 0, 9, false), (-3, 1, 13, false), (-3, 1, 9, false), (-3, 2, 13, false), (-3, 2, 14, false), (-2, -1, 30, false), (2, 0, 10, false), (2, 0, 1, false), (-2, -1, 27, false)];
const PICKUP: &[(i32, i32, u32, bool)] = &[(0, 0, 13, false), (0, 0, 23, false), (0, 0, 25, false), (0, 0, 6, false), (0, 0, 26, false), (0, 0, 7, false), (0, 0, 29, false), (0, 0, 17, false), (0, 0, 21, true), (0, -1, 13, false), (0, -1, 8, false), (0, 1, 13, false), (0, 1, 23, false), (0, 1, 25, false), (0, 1, 21, true), (0, 2, 13, false), (0, 2, 8, false), (1, -1, 13, false), (1, -1, 36, false), (1, -1, 35, false), (1, -1, 30, false), (1, 0, 13, false), (1, 0, 36, false), (1, 1, 13, false), (1, 1, 36, false), (1, 2, 13, false), (1, 2, 36, false), (1, 2, 35, false), (1, 2, 30, false), (2, -1, 13, false), (2, -1, 14, false), (2, -1, 15, false), (2, 0, 13, false), (2, 0, 14, false), (2, 0, 11, false), (2, 0, 2, false), (2, 0, 4, false), (2, 1, 13, false), (2, 1, 14, false), (2, 2, 13, false), (2, 2, 14, false), (2, 2, 15, false), (-1, -1, 13, false), (-1, -1, 36, false), (-1, 0, 13, false), (-1, 0, 36, false), (-1, 0, 21, true), (-1, 1, 13, false), (-1, 1, 36, false), (-1, 1, 21, true), (-1, 2, 13, false), (-1, 2, 36, false), (-2, -1, 13, false), (-2, -1, 14, false), (-2, -1, 27, false), (-2, 0, 13, false), (-2, 0, 5, false), (-2, 0, 19, false), (-2, 1, 13, false), (-2, 1, 5, false), (-2, 2, 13, false), (-2, 2, 14, false), (-3, -1, 13, false), (-3, -1, 14, false), (-3, -1, 34, false), (-3, -1, 30, false), (-3, 0, 13, false), (-3, 0, 28, false), (-3, 1, 13, false), (-3, 1, 28, false), (-3, 2, 13, false), (-3, 2, 14, false), (-3, 2, 34, false), (-3, 2, 30, false), (-4, -1, 13, false), (-4, -1, 14, false), (-4, 0, 13, false), (-4, 0, 28, false), (-4, 1, 13, false), (-4, 1, 28, false), (-4, 2, 13, false), (-4, 2, 14, false)];
const BICYCLE: &[(i32, i32, u32, bool)] = &[(0, 0, 37, false), (0, 0, 22, false), (0, 0, 16, false), (0, 0, 12, false), (1, 0, 37, false), (1, 0, 33, false), (-1, 0, 37, false), (-1, 0, 32, false), (1, 0, 31, false), (-1, 0, 31, false), (-1, 0, 3, false)];

fn build_model(id: u64, name: &str, pivot: [i32; 3], parts: &[(i32, i32, u32, bool)]) -> VehicleModel {
    VehicleModel {
        id,
        name: name.to_string(),
        face_deg: 0.0,
        parts: parts
            .iter()
            .map(|&(dx, dy, vpart, roof)| PartModel {
                key: PartKey {
                    vpart,
                    mount: [dx, dy, 0],
                },
                abs_pos: [pivot[0] + dx, pivot[1] + dy, pivot[2]],
                hp_frac: 255,
                open: false,
                broken: false,
                roof,
            })
            .collect(),
    }
}

fn scene(models: Vec<VehicleModel>) -> VehicleScene {
    VehicleScene {
        models,
        vpart_names: NAMES
            .iter()
            .enumerate()
            .map(|(i, n)| (i as u32 + 1, n.to_string()))
            .collect::<HashMap<_, _>>(),
    }
}

fn capture(
    root: &PathBuf,
    assets: &PathBuf,
    name: &str,
    renderer: RendererKind,
    size: (u32, u32),
    scene: VehicleScene,
    target: (f32, f32),
    distance: f32,
) {
    let out = root.join("captures").join(name);
    let args = CaptureArgs {
        snapshot: "sample-scene".to_string(),
        renderer,
        textures: Some(assets.clone()),
        size,
        camera_target: target,
        yaw_deg: 45.0,
        pitch_deg: 38.0,
        distance,
        out,
        vehicles: Some(scene),
        ..CaptureArgs::default()
    };
    match run_capture(&args) {
        Ok(o) => println!(
            "wrote {} ({}x{}, {} quads, {} distinct colors)",
            o.out.display(),
            o.size.0,
            o.size.1,
            o.quads,
            o.distinct_colors
        ),
        Err(e) => println!("capture {name} failed: {e}"),
    }
}

fn main() {
    let root = PathBuf::from("/home/hatch/workspace/cdda-voxel");
    let assets = root.join("target/gallery_assets");
    if !assets.join("materials.json").exists() {
        // Same run the prop gallery makes (v2 output includes
        // the v1 materials vehicle texturing consumes).
        let tileset_dir = root.join("fork-src/cdda/gfx/UltimateCataclysm");
        let ts = tileset::load_decomposed(&tileset_dir).expect("tileset loads");
        let cfg = voxelizer::VoxelizerConfig {
            emit_v2: true,
            ..Default::default()
        };
        voxelizer::run(&ts, &cfg, &assets).expect("voxelizer run");
    }

    capture(
        &root,
        &assets,
        "vehicle_car_textured.png",
        RendererKind::Raster,
        (1280, 720),
        scene(vec![build_model(1, "car", [14, 6, 0], CAR)]),
        (13.5, 6.5),
        15.0,
    );
    capture(
        &root,
        &assets,
        "vehicle_pickup_textured.png",
        RendererKind::Raster,
        (1280, 720),
        scene(vec![build_model(2, "pickup", [15, 6, 0], PICKUP)]),
        (14.0, 6.5),
        17.0,
    );
    capture(
        &root,
        &assets,
        "vehicle_bicycle_textured.png",
        RendererKind::Raster,
        (1280, 720),
        scene(vec![build_model(3, "bicycle", [12, 12, 0], BICYCLE)]),
        (12.0, 12.0),
        8.0,
    );
    let all = scene(vec![
        build_model(1, "car", [14, 6, 0], CAR),
        build_model(2, "pickup", [13, 11, 0], PICKUP),
        build_model(3, "bicycle", [11, 20, 0], BICYCLE),
    ]);
    capture(
        &root,
        &assets,
        "vehicle_all_textured_raster.png",
        RendererKind::Raster,
        (1280, 720),
        all.clone(),
        (12.5, 12.0),
        30.0,
    );
    capture(
        &root,
        &assets,
        "vehicle_all_textured_rt.png",
        RendererKind::Rt,
        (960, 540),
        all,
        (12.5, 12.0),
        30.0,
    );
}
