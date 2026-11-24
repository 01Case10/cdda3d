//! Vehicle material wiring gates (the "(b)" path: vpart
//! materials through the voxelizer, like furniture): a named
//! vehicle scene resolves each vpart to its voxelizer material
//! (`vp_<name>`), stamps its tiles with per-vpart material
//! block ids, and the shared texture-spec builder carries those
//! ids' sprite layers — in both renderers (raster via
//! `stamp_tex_layers`, RT via the tree's material ids).
//!
//! Rules pinned here:
//! - Allocation is deterministic: distinct vparts in ascending
//!   wire-id order take consecutive block ids from
//!   `VEHICLE_MATERIAL_ID_BASE`, state allocations (open, then
//!   broken) follow their vpart's plain id.
//! - A tile stamps its DISPLAYED parts: the last body part and
//!   the last roof part in export order (the game's draw order).
//! - A state allocation exists only when the material carries
//!   that state's subtile picture; otherwise the part falls
//!   back to its plain allocation.
//! - No names / no textures = the flat path, unchanged: every
//!   vehicle block is `VEHICLE_BLOCK_ID`.
//!
//! The capture half is adapter-conditional in the established
//! pattern (lavapipe counts as an adapter).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::frame::VehicleScene;
use engine::materials::MaterialTable;
use engine::scene_prep::{
    stamp_vehicles, stamp_vehicles_mapped, vehicle_materials, VEHICLE_MATERIAL_ID_BASE,
};
use vehicles::{PartKey, PartModel, VehicleModel};

const TILE: u32 = 8;
// Fixture layer indices (source layers in the fixture dir).
const FRAME_TOP: usize = 0;
const WHEEL_TOP: usize = 3;
const ROOF_TOP: usize = 6;
const DOOR_TOP: usize = 9;
const DOOR_OPEN_SIDE: usize = 12;
const LAYER_COUNT: usize = 13;

// Wire vpart ids in the fixture scene.
const VP_FRAME: u32 = 1;
const VP_WHEEL: u32 = 2;
const VP_ROOF: u32 = 3;
const VP_DOOR: u32 = 4;
// Expected allocations (ascending vpart order from the base;
// the door's open-state allocation follows its plain one).
const ID_FRAME: u32 = VEHICLE_MATERIAL_ID_BASE;
const ID_WHEEL: u32 = VEHICLE_MATERIAL_ID_BASE + 1;
const ID_ROOF: u32 = VEHICLE_MATERIAL_ID_BASE + 2;
const ID_DOOR: u32 = VEHICLE_MATERIAL_ID_BASE + 3;
const ID_DOOR_OPEN: u32 = VEHICLE_MATERIAL_ID_BASE + 4;

/// A tiny voxelizer-shaped fixture with four vpart materials.
/// Each layer is a solid color unique to the layer, so a
/// textured render's palette identifies the material outright.
/// `vp_door` is multitile and carries an "open" subtile (layer
/// 12); the other materials carry no state subtiles.
fn write_fixture(dir: &Path) {
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    std::fs::write(
        dir.join("blocks.json"),
        format!(
            r#"{{"format":"png-layers","tile_size":{TILE},"layer_count":{LAYER_COUNT}}}"#
        ),
    )
    .unwrap();
    let mat = |id: &str, idx: usize, top: usize, multitile: bool, sub: &str| {
        format!(
            r#"{{"id":"{id}","resolved_id":"{id}","resolution":"exact","material_index":{idx},"top_layer":{top},"side_layer":{},"bottom_layer":{},"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":{multitile},"subtile_layers":{{{sub}}}}}"#,
            top + 1,
            top + 2,
        )
    };
    let mats = format!(
        "[{},{},{},{}]",
        mat("vp_frame", 0, FRAME_TOP, false, ""),
        mat("vp_wheel", 1, WHEEL_TOP, false, ""),
        mat("vp_roof", 2, ROOF_TOP, false, ""),
        mat(
            "vp_door",
            3,
            DOOR_TOP,
            true,
            &format!(r#""open":{DOOR_OPEN_SIDE}"#)
        ),
    );
    std::fs::write(dir.join("materials.json"), mats).unwrap();
    // Layer colors: frame red, wheel green, roof blue, door
    // yellow, door-open side cyan — channel-dominant so the
    // capture assertions can classify by dominant channel.
    let colors: [[u8; 3]; LAYER_COUNT] = [
        [200, 30, 30],
        [180, 30, 30],
        [160, 30, 30],
        [30, 200, 30],
        [30, 180, 30],
        [30, 160, 30],
        [30, 30, 200],
        [30, 30, 180],
        [30, 30, 160],
        [200, 200, 30],
        [180, 180, 30],
        [160, 160, 30],
        [30, 200, 200],
    ];
    for (li, c) in colors.iter().enumerate() {
        let img = image::RgbaImage::from_pixel(TILE, TILE, image::Rgba([c[0], c[1], c[2], 255]));
        img.save(blocks.join(format!("layer_{li:04}.png"))).unwrap();
    }
}

fn fixture_dir(tag: &str) -> PathBuf {
    // Unique per test: the tests run in parallel in one process
    // and each removes its dir at the end.
    let dir =
        std::env::temp_dir().join(format!("engine_vehicle_tex_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_fixture(&dir);
    dir
}

fn part(vpart: u32, mount: [i32; 3], abs: [i32; 3], roof: bool, open: bool) -> PartModel {
    PartModel {
        key: PartKey { vpart, mount },
        abs_pos: abs,
        hp_frac: 255,
        open,
        broken: false,
        roof,
    }
}

/// One vehicle, four tiles on the sample yard (y = 15, clear of
/// the house and pond):
/// - A (6,15): frame then wheel — displayed body is the wheel.
/// - B (7,15): frame + roof part — body frame, top roof.
/// - C (8,15): open door — stamps with the open-state id.
/// - D (9,15): closed door — stamps with the plain door id.
fn fixture_scene() -> VehicleScene {
    let model = VehicleModel {
        id: 1,
        name: "gate vehicle".to_string(),
        face_deg: 0.0,
        parts: vec![
            part(VP_FRAME, [-1, 0, 0], [6, 15, 0], false, false),
            part(VP_WHEEL, [0, 0, 0], [6, 15, 0], false, false),
            part(VP_FRAME, [1, 0, 0], [7, 15, 0], false, false),
            part(VP_ROOF, [1, 0, 0], [7, 15, 0], true, false),
            part(VP_DOOR, [2, 0, 0], [8, 15, 0], false, true),
            part(VP_DOOR, [3, 0, 0], [9, 15, 0], false, false),
        ],
    };
    let mut vpart_names = HashMap::new();
    vpart_names.insert(VP_FRAME, "frame".to_string());
    vpart_names.insert(VP_WHEEL, "wheel".to_string());
    vpart_names.insert(VP_ROOF, "roof".to_string());
    vpart_names.insert(VP_DOOR, "door".to_string());
    VehicleScene {
        models: vec![model],
        vpart_names,
    }
}

fn sample_map() -> mesher::chunk::ChunkMap {
    let map = engine::world::build_sample_scene(mesher::scale::WorldScale::default());
    engine::world::filtered_map(&map, engine::world::HOUSE_MIN_GAME_Z, &[0])
}

fn block_id_at(map: &mesher::chunk::ChunkMap, x: i32, y: i32, bz: i32) -> u32 {
    for (key, chunk) in &map.chunks {
        let [w, h, _] = chunk.grid.size;
        let (lx, ly) = (x - key[0] * w as i32, y - key[1] * h as i32);
        if lx >= 0 && ly >= 0 && lx < w as i32 && ly < h as i32 {
            return chunk.grid.get(lx, ly, bz).id;
        }
    }
    panic!("no chunk covers ({x},{y})");
}

// ------------------------------------------------------------ allocation

#[test]
fn vehicle_materials_allocate_deterministically() {
    let dir = fixture_dir("alloc");
    let table = MaterialTable::load(&dir).unwrap();
    let scene = fixture_scene();
    let mats = vehicle_materials(&scene, &table);
    assert!(!mats.is_empty());
    assert_eq!(mats.id_map[&ID_FRAME], "vp_frame");
    assert_eq!(mats.id_map[&ID_WHEEL], "vp_wheel");
    assert_eq!(mats.id_map[&ID_ROOF], "vp_roof");
    assert_eq!(mats.id_map[&ID_DOOR], "vp_door");
    // Per-part resolution: state id only for the open door.
    let m = &scene.models[0];
    assert_eq!(mats.block_for(&m.parts[0]), ID_FRAME);
    assert_eq!(mats.block_for(&m.parts[1]), ID_WHEEL);
    assert_eq!(mats.block_for(&m.parts[3]), ID_ROOF);
    assert_eq!(mats.block_for(&m.parts[4]), ID_DOOR_OPEN);
    assert_eq!(mats.block_for(&m.parts[5]), ID_DOOR);
    // Exactly one state assignment: the door's open state.
    assert_eq!(mats.state_assignments.len(), 1);
    let sa = mats.state_assignments[0];
    assert_eq!(sa.base_id, ID_DOOR);
    assert_eq!(sa.virtual_id, ID_DOOR_OPEN);
    assert!(sa.open && !sa.broken);
    // A wheel has no state subtile in its material: an open or
    // broken wheel still takes the plain wheel allocation.
    let mut broken_wheel = m.parts[1];
    broken_wheel.broken = true;
    assert_eq!(mats.block_for(&broken_wheel), ID_WHEEL);
    // Unnamed scene resolves nothing.
    let unnamed = VehicleScene {
        models: scene.models.clone(),
        vpart_names: HashMap::new(),
    };
    assert!(vehicle_materials(&unnamed, &table).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ------------------------------------------------------------ stamping

#[test]
fn stamped_tiles_carry_displayed_part_material_ids() {
    let dir = fixture_dir("stamp");
    let table = MaterialTable::load(&dir).unwrap();
    let scene = fixture_scene();
    let mats = vehicle_materials(&scene, &table);

    let mut map = sample_map();
    let (models, tiles) = stamp_vehicles_mapped(&mut map, &scene, Some(&mats));
    assert_eq!((models, tiles), (1, 4));
    // A: displayed body = wheel (last body part), lower blocks.
    assert_eq!(block_id_at(&map, 6, 15, 0), ID_WHEEL);
    assert_eq!(block_id_at(&map, 6, 15, 1), ID_WHEEL);
    // B: body frame below, roof part's material on top.
    assert_eq!(block_id_at(&map, 7, 15, 0), ID_FRAME);
    assert_eq!(block_id_at(&map, 7, 15, 1), ID_FRAME);
    assert_eq!(block_id_at(&map, 7, 15, 2), ID_ROOF);
    // C: open door -> state allocation. D: closed -> plain.
    assert_eq!(block_id_at(&map, 8, 15, 0), ID_DOOR_OPEN);
    assert_eq!(block_id_at(&map, 9, 15, 0), ID_DOOR);

    // The flat path is untouched: no materials, one shared id.
    let mut flat = sample_map();
    stamp_vehicles(&mut flat, &scene);
    for (x, bz) in [(6, 0), (6, 1), (7, 0), (7, 2), (8, 0), (9, 0)] {
        assert_eq!(
            block_id_at(&flat, x, 15, bz),
            vehicles::VEHICLE_BLOCK_ID,
            "flat stamp at ({x},15,{bz})"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// ------------------------------------------------------------ spec

#[test]
fn spec_carries_vehicle_material_layers() {
    let dir = fixture_dir("spec");
    let scene = fixture_scene();
    let (_meshes, _stats, spec) =
        engine::scene_prep::prepare_raster_sample_v(&[0], Some(&dir), Some(&scene)).unwrap();
    let spec = spec.expect("textured prepare returns a spec");
    // Present flags at every allocation; the compact layers are
    // the fixture's own (distinct solid layers compact in load
    // order: frame, wheel, roof, door, then the state side).
    for id in [ID_FRAME, ID_WHEEL, ID_ROOF, ID_DOOR, ID_DOOR_OPEN] {
        assert_eq!(
            spec.material_layers[id as usize][3], 1,
            "allocation {id} present in spec"
        );
    }
    assert_eq!(spec.material_layers[ID_WHEEL as usize][..3], [3, 4, 5]);
    // The open-state entry: door top/bottom, open side layer.
    assert_eq!(
        spec.material_layers[ID_DOOR_OPEN as usize][..3],
        [9, DOOR_OPEN_SIDE as u32, 11]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ------------------------------------------------------------ captures

fn capture_args(dir: Option<&Path>, out: &Path, renderer: RendererKind) -> CaptureArgs {
    let mut args = CaptureArgs {
        snapshot: "sample-scene".to_string(),
        renderer,
        size: (640, 360),
        camera_target: (7.5, 15.0),
        yaw_deg: 45.0,
        pitch_deg: 38.0,
        distance: 10.0,
        out: out.to_path_buf(),
        ..CaptureArgs::default()
    };
    args.textures = dir.map(|d| d.to_path_buf());
    args.vehicles = Some(fixture_scene());
    args
}

/// Count pixels whose dominant channel matches `ch` (0=R,1=G,
/// 2=B) by a wide margin — the fixture materials are
/// channel-dominant solids, flat vehicle colors are not.
fn dominant_count(img: &image::RgbaImage, ch: usize) -> usize {
    img.pixels()
        .filter(|p| {
            let c = [p.0[0] as u16, p.0[1] as u16, p.0[2] as u16];
            let other = (0..3).filter(|&i| i != ch).map(|i| c[i]).max().unwrap();
            c[ch] > 120 && c[ch] > other + 60
        })
        .count()
}

#[test]
fn textured_vehicle_capture_shows_vpart_materials() {
    let dir = fixture_dir("capture");
    let tmp = std::env::temp_dir().join(format!("engine_vehicle_cap_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let flat_out = tmp.join(format!("flat_{renderer:?}.png"));
        let tex_out = tmp.join(format!("tex_{renderer:?}.png"));
        let flat = run_capture(&capture_args(None, &flat_out, renderer));
        let flat = match flat {
            Ok(o) => o,
            Err(CaptureError::NoAdapter) => {
                eprintln!("vehicle texture gate: no wgpu adapter — capture half NOT RUN");
                return;
            }
            Err(e) => panic!("flat capture failed: {e}"),
        };
        let tex = run_capture(&capture_args(Some(&dir), &tex_out, renderer))
            .unwrap_or_else(|e| panic!("textured capture failed: {e}"));
        assert!(tex.distinct_colors > flat.distinct_colors);
        let flat_img = image::open(&flat_out).unwrap().to_rgba8();
        let tex_img = image::open(&tex_out).unwrap().to_rgba8();
        // The wheel's green and the roof's blue are ADDED by the
        // textured capture: the yard contributes the same
        // green/blue base (grass, pond) to both, and the flat
        // vehicles are grey/rust — never channel-dominant.
        assert!(
            dominant_count(&tex_img, 1) > dominant_count(&flat_img, 1) + 40,
            "{renderer:?}: wheel green added by textured capture (flat {}, tex {})",
            dominant_count(&flat_img, 1),
            dominant_count(&tex_img, 1)
        );
        assert!(
            dominant_count(&tex_img, 2) > dominant_count(&flat_img, 2) + 20,
            "{renderer:?}: roof blue added by textured capture (flat {}, tex {})",
            dominant_count(&flat_img, 2),
            dominant_count(&tex_img, 2)
        );
    }
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::remove_dir_all(&dir);
}
