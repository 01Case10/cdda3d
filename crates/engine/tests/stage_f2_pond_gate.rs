//! Stage F part 2 gates — pond/shoreline engine integration
//! (procedural texture pipeline §10.1 engine half, §14/§15):
//!
//! * Emission (voxelizer, CPU): a v2 run over a tileset with a
//!   water material gains a `shores` table — 255 entries (every
//!   mask except the all-water interior 0xFF) on the primary
//!   water material `t_water_dp`, one `shore_<layer>.png` per
//!   entry — while a v1 run and a v2 run over the water-less
//!   fixture stay in the pre-F2 form (no `shores` key, no shore
//!   files). Two v2 runs are byte-identical.
//! * Round-trip (CPU): the emitted shore PNG for a known mask,
//!   loaded through the engine loader + spec builder, equals
//!   `voxelizer::shore::shore_texture` for that mask byte-for-
//!   byte, composed from the canonical generated trio
//!   (t_water_dp / t_dirt / t_grass families).
//! * Assignment (CPU): a fixture pond (the sample scene's 4x4
//!   pond) — corner/edge water tiles get shore variants, the
//!   center keeps plain water, and the ChunkMap and VoxelWorld
//!   assignment forms agree (also unit-tested in engine::shore).
//! * Spec stability (CPU): with no shore assignments, the
//!   combined variant builder's spec equals the base builder's.
//! * Capture (GPU, adapter-conditional like the other gates):
//!   the sample-scene pond in BOTH renderers with shore assets
//!   shows dirt-bank pixels the plain-assets capture does not
//!   have, framed on the pond. The pond still reads
//!   rounded-square (Stage H tuning) — this gate proves wiring,
//!   not shape.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::materials::MaterialTable;
use engine::textures::{build_texture_spec, build_texture_spec_variants};
use image::{Rgba, RgbaImage};

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

const WATER_RGB: [u8; 3] = [30, 80, 160];
const DIRT_RGB: [u8; 3] = [120, 95, 60];
const GRASS_RGB: [u8; 3] = [70, 110, 45];
/// The sample pond's NW-corner mask (water at E, SE, S).
const CORNER_MASK: u8 = 0b0001_1100;

fn solid(rgb: [u8; 3]) -> RgbaImage {
    RgbaImage::from_pixel(32, 32, Rgba([rgb[0], rgb[1], rgb[2], 255]))
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../voxelizer/tests/fixture")
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), &to).unwrap();
        }
    }
}

/// The fixture tileset plus a `t_water_dp` entry (a blue sprite
/// named `water`), in a fresh temp dir.
fn tileset_with_water(base: &Path) -> PathBuf {
    let dir = base.join("tileset");
    copy_dir(&fixture_dir(), &dir);
    let pngs = dir.join("pngs_normal_32x32");
    // A blue checker so the generated water texture has grain.
    let mut img = RgbaImage::new(32, 32);
    for y in 0..32 {
        for x in 0..32 {
            let v = if (x + y) % 2 == 0 { 0 } else { 12 };
            img.put_pixel(
                x,
                y,
                Rgba([WATER_RGB[0], WATER_RGB[1] + v, WATER_RGB[2], 255]),
            );
        }
    }
    img.save(pngs.join("water.png")).unwrap();
    std::fs::write(
        pngs.join("t_water_dp.json"),
        "{\n \"fg\": \"water\",\n \"id\": \"t_water_dp\"\n}\n",
    )
    .unwrap();
    dir
}

/// The fixture tileset plus a MULTITILE `t_water_dp` entry in
/// the real UltiCa shape: the base fg is the `unconnected`
/// variant (a blue blob, transparent outside the middle, over a
/// grass bg — a lone puddle), and the `center` additional tile
/// is full-tile water. This is the shape that made a body of
/// water render as one puddle per tile.
fn tileset_with_multitile_water(base: &Path) -> PathBuf {
    let dir = base.join("tileset");
    copy_dir(&fixture_dir(), &dir);
    let pngs = dir.join("pngs_normal_32x32");
    let checker = |img: &mut RgbaImage, x: u32, y: u32| {
        let v = if (x + y) % 2 == 0 { 0 } else { 12 };
        img.put_pixel(
            x,
            y,
            Rgba([WATER_RGB[0], WATER_RGB[1] + v, WATER_RGB[2], 255]),
        );
    };
    // Center variant: full-tile water.
    let mut center = RgbaImage::new(32, 32);
    for y in 0..32 {
        for x in 0..32 {
            checker(&mut center, x, y);
        }
    }
    center.save(pngs.join("water_center.png")).unwrap();
    // Unconnected variant: a centered blob, transparent around
    // it (the grass bg shows through when composited).
    let mut blob = RgbaImage::new(32, 32);
    for y in 8..24 {
        for x in 8..24 {
            checker(&mut blob, x, y);
        }
    }
    blob.save(pngs.join("water_unconnected.png")).unwrap();
    std::fs::write(
        pngs.join("t_water_dp.json"),
        "{\n \"additional_tiles\": [\n  {\n   \"bg\": \"grass_bg\",\n   \"fg\": \"water_center\",\n   \"id\": \"center\"\n  },\n  {\n   \"bg\": \"grass_bg\",\n   \"fg\": \"water_unconnected\",\n   \"id\": \"unconnected\"\n  }\n ],\n \"bg\": \"grass_bg\",\n \"fg\": \"water_unconnected\",\n \"id\": \"t_water_dp\",\n \"multitile\": true\n}\n",
    )
    .unwrap();
    dir
}

fn run_voxelizer(tileset: &Path, out: &Path, emit_v2: bool) {
    let _ = std::fs::remove_dir_all(out);
    std::fs::create_dir_all(out).unwrap();
    let ts = tileset::load_decomposed(tileset).expect("tileset loads");
    let cfg = voxelizer::VoxelizerConfig {
        emit_v2,
        ..Default::default()
    };
    voxelizer::run(&ts, &cfg, out).expect("voxelizer run");
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

// ------------------------------------------------------- emission
#[test]
fn f2_shore_emission_and_forms() {
    let base = std::env::temp_dir().join(format!("f2pond_emit_{}", std::process::id()));
    let ts = tileset_with_water(&base);

    // v1: no shores anywhere.
    let v1 = base.join("v1");
    run_voxelizer(&ts, &v1, false);
    let v1_json = read_json(&v1.join("materials.json"));
    assert!(v1_json.is_array(), "v1 stays a bare array");
    assert!(
        !v1.join("shore_0000.png").exists(),
        "v1 writes no shore files"
    );

    // v2 with water: shores table, 255 entries on t_water_dp.
    let v2a = base.join("v2a");
    run_voxelizer(&ts, &v2a, true);
    let v2_json = read_json(&v2a.join("materials.json"));
    let water_index = v2_json["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "t_water_dp")
        .and_then(|m| m["material_index"].as_u64())
        .expect("materials include t_water_dp");
    let shores = v2_json["shores"]
        .as_array()
        .expect("v2 gains a shores table");
    assert_eq!(shores.len(), 255, "every mask except all-water 0xFF");
    for (i, e) in shores.iter().enumerate() {
        assert_eq!(e["material_index"].as_u64(), Some(water_index));
        assert_eq!(
            e["mask"].as_u64(),
            Some(i as u64),
            "masks in ascending order"
        );
        assert_eq!(e["layer"].as_u64(), Some(i as u64));
        assert!(
            v2a.join(format!("shore_{i:04}.png")).exists(),
            "shore file for mask {i}"
        );
    }

    // Determinism: a second v2 run is byte-identical.
    let v2b = base.join("v2b");
    run_voxelizer(&ts, &v2b, true);
    assert_eq!(
        std::fs::read(v2a.join("materials.json")).unwrap(),
        std::fs::read(v2b.join("materials.json")).unwrap(),
        "v2 materials.json byte-identical across runs"
    );
    for i in [0usize, CORNER_MASK as usize, 254] {
        assert_eq!(
            std::fs::read(v2a.join(format!("shore_{i:04}.png"))).unwrap(),
            std::fs::read(v2b.join(format!("shore_{i:04}.png"))).unwrap(),
            "shore PNG {i} byte-identical across runs"
        );
    }

    // v2 over the water-less fixture: pre-F2 form, no shores key.
    let v2_plain = base.join("v2plain");
    run_voxelizer(&fixture_dir(), &v2_plain, true);
    let plain_json = read_json(&v2_plain.join("materials.json"));
    assert!(
        plain_json.get("shores").is_none(),
        "no water -> no shores table"
    );
    assert!(!v2_plain.join("shore_0000.png").exists());

    let _ = std::fs::remove_dir_all(&base);
}

// --------------------------------------- plain top of multitile
/// Regression (2026-11-17): a body of water rendered as multiple
/// cumulative puddles — one per tile. Root cause, voxelizer half:
/// a multitile entry's base sprite is its UNCONNECTED variant
/// (the lone-tile drawing), and that sprite became the material's
/// plain top layer, which is exactly the layer interior water
/// tiles keep. The plain top of a multitile ground material is
/// its CENTER (fully-connected) variant.
#[test]
fn f2_multitile_water_plain_top_is_center() {
    let base = std::env::temp_dir().join(format!("f2pond_mt_{}", std::process::id()));
    let ts = tileset_with_multitile_water(&base);
    let v2 = base.join("v2");
    run_voxelizer(&ts, &v2, true);
    let v2_json = read_json(&v2.join("materials.json"));
    let water = v2_json["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "t_water_dp")
        .expect("materials include t_water_dp")
        .clone();
    assert_eq!(water["multitile"], true);
    let layer_bytes = |l: u64| {
        std::fs::read(v2.join("blocks").join(format!("layer_{l:04}.png"))).unwrap()
    };
    let top = layer_bytes(water["top_layer"].as_u64().unwrap());
    let center = layer_bytes(water["subtile_layers"]["center"].as_u64().unwrap());
    let unconnected = layer_bytes(water["subtile_layers"]["unconnected"].as_u64().unwrap());
    assert_eq!(
        top, center,
        "plain top of multitile water must be the center variant"
    );
    assert_ne!(
        top, unconnected,
        "plain top must not be the unconnected (lone puddle) variant"
    );
    // The shore set still attaches to the multitile water.
    assert_eq!(
        v2_json["shores"].as_array().map(|s| s.len()),
        Some(255),
        "multitile water still carries the shore set"
    );
    let _ = std::fs::remove_dir_all(&base);
}

// ----------------------------------------------------- round-trip
#[test]
fn f2_shore_round_trip_through_loader_and_spec() {
    let base = std::env::temp_dir().join(format!("f2pond_rt_{}", std::process::id()));
    let ts_dir = tileset_with_water(&base);
    let v2 = base.join("v2");
    run_voxelizer(&ts_dir, &v2, true);

    // Expected: shore_texture over the canonical generated trio.
    let ts = tileset::load_decomposed(&ts_dir).expect("tileset loads");
    let gen_for = |id: &str, kind: voxelizer::RecipeKind| {
        let entry = ts.entries.get(id).expect("entry");
        voxelizer::generate(kind, &voxelizer::family_sprites(&ts, entry)).image
    };
    let expected = voxelizer::shore_texture(
        CORNER_MASK,
        &gen_for("t_water_dp", voxelizer::RecipeKind::Water),
        &gen_for("t_dirt", voxelizer::RecipeKind::Dirt),
        &gen_for("t_grass", voxelizer::RecipeKind::Grass),
    );

    // Loader: the water material carries the corner variant.
    let table = MaterialTable::load(&v2).expect("v2 loads");
    assert_eq!(table.shore_size, 32);
    let water = table.get("t_water_dp").expect("water material");
    let shore_layer = water
        .shore_top_layer(CORNER_MASK)
        .expect("corner mask has a shore variant");
    assert_eq!(
        water.shore_top_layer(0xFF),
        None,
        "interior keeps plain top"
    );

    // Spec: the variant entry's top layer bytes are the emitted
    // shore PNG's bytes, which equal shore_texture's output.
    let id_map: HashMap<u32, String> = [(8u32, "t_water_dp".to_string())].into_iter().collect();
    let assignments = [engine::shore::ShoreAssignment {
        base_id: 8,
        mask: CORNER_MASK,
        virtual_id: 40,
    }];
    let (spec, stats) =
        build_texture_spec_variants(&table, &v2, &id_map, &[], &assignments, &[]).unwrap();
    assert_eq!(stats.shore_variants, 1);
    let entry = spec.material_layers[40];
    assert_eq!(entry[3], 1, "variant entry present");
    let top_bytes = &spec.layers[entry[0] as usize];
    let emitted = image::open(v2.join(format!("shore_{shore_layer:04}.png")))
        .unwrap()
        .to_rgba8();
    assert_eq!(top_bytes, emitted.as_raw(), "spec top = emitted shore PNG");
    assert_eq!(
        top_bytes,
        expected.as_raw(),
        "emitted shore = shore_texture(corner mask) byte-for-byte"
    );
    // Side/bottom are the base water material's.
    let base_entry = spec.material_layers[8];
    assert_eq!(entry[1], base_entry[1], "variant side = base side");
    assert_eq!(entry[2], base_entry[2], "variant bottom = base bottom");

    let _ = std::fs::remove_dir_all(&base);
}

// ----------------------------------------------------- assignment
#[test]
fn f2_spec_without_shore_assignments_equals_base_spec() {
    let base = std::env::temp_dir().join(format!("f2pond_spec_{}", std::process::id()));
    let ts_dir = tileset_with_water(&base);
    let v2 = base.join("v2");
    run_voxelizer(&ts_dir, &v2, true);
    let table = MaterialTable::load(&v2).unwrap();
    let id_map = engine::rt::fake_material_ids();
    let (base_spec, _) = build_texture_spec(&table, &v2, &id_map).unwrap();
    let (variant_spec, stats) =
        build_texture_spec_variants(&table, &v2, &id_map, &[], &[], &[]).unwrap();
    assert_eq!(stats.shore_variants, 0);
    assert_eq!(variant_spec.material_layers, base_spec.material_layers);
    assert_eq!(variant_spec.layers, base_spec.layers);
    let _ = std::fs::remove_dir_all(&base);
}

// ------------------------------------------- RT preparation order
/// Regression (2026-11-17), engine half of the puddles bug: the
/// RT path ran the Phase 2d multitile rewrite before the shore
/// preparation, and the shore preparation recognizes water by
/// its BASE material id — so with a (real, multitile) water
/// material every water cell had already been rewritten to a
/// multitile virtual id, the shore pass matched nothing, and
/// every water tile kept its plain top. Shore-carrying
/// materials are excluded from the multitile rewrite; this test
/// runs the exact preparation sequence `render_rt_frame` runs.
#[test]
fn f2_rt_shore_survives_multitile_water() {
    use mesher::block::TILE_WATER;
    let base = std::env::temp_dir().join(format!("f2pond_rtord_{}", std::process::id()));
    let dir = capture_assets(&base, true);

    // The sample scene's 4x4 pond, prepared as the RT path does.
    let levels = engine::rt::capture_visible_levels(1);
    let mut world = engine::rt::world_for_levels_sv(&levels, None, true);
    let mt = engine::rt::prepare_fake_variants(&mut world, &dir).unwrap();
    assert!(
        mt.iter().all(|a| a.base_id != TILE_WATER),
        "shore water must not be multitile-rewritten: {mt:?}"
    );
    let shore = engine::rt::prepare_fake_shore_variants(&mut world, &dir).unwrap();
    assert!(
        !shore.is_empty(),
        "shore assignments must exist for the pond"
    );
    assert!(shore.iter().all(|a| a.base_id == TILE_WATER));
    // A pond corner cell was rewritten to its shore variant; an
    // interior cell keeps the base water id (plain top = the
    // center variant, per the voxelizer rule above).
    let corner_mask = 0b0001_1100u8;
    let corner_virtual = shore
        .iter()
        .find(|a| a.mask == corner_mask)
        .expect("corner mask assigned")
        .virtual_id;
    assert_eq!(
        world.get([16, 14, 0]).unwrap().material_id,
        corner_virtual,
        "pond corner takes its shore variant"
    );
    assert_eq!(
        world.get([17, 15, 0]).unwrap().material_id,
        TILE_WATER,
        "pond interior keeps the base water entry"
    );

    // A large body of water (12x12): every border tile takes a
    // shore variant, every interior tile keeps the base entry —
    // no tile is left carrying a per-tile puddle top.
    let mut big = treebuild::world::VoxelWorld::new();
    for y in 0..12 {
        for x in 0..12 {
            big.set(
                [x, y, 0],
                treebuild::world::VoxelCell::new([0.2, 0.4, 0.8], TILE_WATER),
            );
        }
    }
    let mt = engine::rt::prepare_fake_variants(&mut big, &dir).unwrap();
    assert!(mt.iter().all(|a| a.base_id != TILE_WATER));
    let shore = engine::rt::prepare_fake_shore_variants(&mut big, &dir).unwrap();
    let interior = big
        .iter()
        .filter(|(_, c)| c.material_id == TILE_WATER)
        .count();
    assert_eq!(interior, 100, "10x10 interior keeps the base entry");
    let edged = big.iter().filter(|(_, c)| c.material_id != TILE_WATER).count();
    assert_eq!(edged, 44, "the 12x12 border takes shore variants");
    assert!(
        shore.len() >= 8,
        "border masks cover corners and straights: {} assignments",
        shore.len()
    );
    let _ = std::fs::remove_dir_all(&base);
}

// ------------------------------------------------------------ GPU
/// Hand-built asset dir for the capture: solid-color layers for
/// the fake scene's materials, with the water material carrying
/// shore variants generated from the solid trio (so dirt-bank
/// pixels are exactly DIRT_RGB before renderer shading).
/// `with_shores` false builds the same dir without the shores
/// table — the plain-water control.
fn capture_assets(base: &Path, with_shores: bool) -> PathBuf {
    let dir = base.join(if with_shores { "shore" } else { "plain" });
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    let mut mats = Vec::new();
    let mut push = |id: &str, rgb: [u8; 3], mats: &mut Vec<serde_json::Value>| {
        let layer = mats.len();
        solid(rgb)
            .save(blocks.join(format!("layer_{layer:04}.png")))
            .unwrap();
        mats.push(serde_json::json!({
            "id": id, "resolved_id": id, "resolution": "exact",
            "material_index": layer, "top_layer": layer, "side_layer": layer,
            "bottom_layer": layer, "height_voxels": 1, "transparency": "opaque",
            "cutout": false, "transparent": false, "tint": [255, 255, 255],
            "sub_model": null, "multitile": false, "subtile_layers": {}
        }));
    };
    // Order follows fake_material_ids insertion needs only by id.
    push("t_floor_primitive", [150, 110, 70], &mut mats);
    push("t_wall", [170, 165, 150], &mut mats);
    push("t_door_c", [100, 60, 30], &mut mats);
    push("t_window", [150, 190, 210], &mut mats);
    push("t_shingle_flat_roof", [120, 50, 40], &mut mats);
    push("t_grass", GRASS_RGB, &mut mats);
    push("t_chainfence", [110, 110, 110], &mut mats);
    let water_index = mats.len();
    push("t_water_dp", WATER_RGB, &mut mats);
    push("t_tree", [40, 90, 35], &mut mats);
    // The real UltiCa t_water_dp is multitile — the fixture
    // water must be too, or the RT multitile/shore interaction
    // (the shore preparation running after the multitile
    // rewrite) is invisible to the capture gate below.
    for m in mats.iter_mut() {
        if m["id"] == "t_water_dp" {
            m["multitile"] = serde_json::json!(true);
            m["subtile_layers"] = serde_json::json!({ "center": water_index });
        }
    }
    std::fs::write(
        dir.join("blocks.json"),
        format!(
            "{{\"format\":\"png-layers\",\"tile_size\":32,\"layer_count\":{}}}",
            mats.len()
        ),
    )
    .unwrap();
    let mut v2 = serde_json::json!({
        "version": 2,
        "materials": mats,
        "macros": [],
    });
    if with_shores {
        let (w, d, g) = (solid(WATER_RGB), solid(DIRT_RGB), solid(GRASS_RGB));
        let mut entries = Vec::new();
        for mask in 0..=254u8 {
            voxelizer::shore_texture(mask, &w, &d, &g)
                .save(dir.join(format!("shore_{mask:04}.png")))
                .unwrap();
            entries.push(serde_json::json!({
                "material_index": water_index, "mask": mask, "layer": mask
            }));
        }
        v2["shores"] = serde_json::Value::Array(entries);
    }
    std::fs::write(
        dir.join("materials.json"),
        serde_json::to_string_pretty(&v2).unwrap(),
    )
    .unwrap();
    dir
}

fn capture(args: &CaptureArgs) -> Option<RgbaImage> {
    match run_capture(args) {
        Ok(outcome) => {
            assert!(outcome.quads > 0, "blank render is not a pass");
            Some(image::open(&outcome.out).unwrap().to_rgba8())
        }
        Err(CaptureError::NoAdapter) => {
            eprintln!(
                "F2 POND GATE NOT RUN: no wgpu adapter (this gate proves nothing without one)"
            );
            None
        }
        Err(e) => panic!("capture failed: {e}"),
    }
}

/// Dirt-like pixels: r > g > b with a clear red-blue spread —
/// multiplicative face shading preserves channel ordering, so
/// this survives both renderers' lighting where an exact RGB
/// match would not.
fn dirt_like(img: &RgbaImage) -> usize {
    img.pixels()
        .filter(|p| {
            let [r, g, b, _] = p.0;
            r > g && g > b && (r as i32 - b as i32) >= 20
        })
        .count()
}

fn water_like(img: &RgbaImage) -> usize {
    img.pixels()
        .filter(|p| {
            let [r, g, b, _] = p.0;
            b > g && g > r
        })
        .count()
}

#[test]
fn f2_pond_capture_both_renderers() {
    let _guard = gpu_lock();
    let base = std::env::temp_dir().join(format!("f2pond_gpu_{}", std::process::id()));
    let shore_dir = capture_assets(&base, true);
    let plain_dir = capture_assets(&base, false);
    let outdir = base.join("caps");
    std::fs::create_dir_all(&outdir).unwrap();

    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let name = renderer.name();
        // Framed on the pond (tiles x 16..=19, y 14..=17).
        let args = |textures: &Path, out: &Path| CaptureArgs {
            snapshot: "sample-scene".to_string(),
            renderer,
            textures: Some(textures.to_path_buf()),
            size: (320, 240),
            camera_target: (17.5, 15.5),
            distance: 13.0,
            pitch_deg: 55.0,
            yaw_deg: 45.0,
            out: out.to_path_buf(),
            ..CaptureArgs::default()
        };
        let Some(shore_img) = capture(&args(&shore_dir, &outdir.join(format!("{name}_shore.png"))))
        else {
            continue;
        };
        let plain_img = capture(&args(&plain_dir, &outdir.join(format!("{name}_plain.png"))))
            .expect("shore capture ran, so the adapter exists");
        let (shore_dirt, plain_dirt) = (dirt_like(&shore_img), dirt_like(&plain_img));
        let shore_water = water_like(&shore_img);
        println!(
            "{name}: shore capture dirt-like {shore_dirt}, water-like {shore_water}; \
             plain capture dirt-like {plain_dirt}"
        );
        assert!(
            shore_dirt > 200,
            "{name}: shore pond must show its dirt bank ({shore_dirt} dirt-like px)"
        );
        assert!(
            plain_dirt < shore_dirt / 4,
            "{name}: plain pond has no bank (dirt-like {plain_dirt} vs shore {shore_dirt})"
        );
        assert!(
            shore_water > shore_dirt,
            "{name}: pond interior stays water ({shore_water} water-like vs {shore_dirt} dirt-like)"
        );
        assert_ne!(
            shore_img.as_raw(),
            plain_img.as_raw(),
            "{name}: shore integration must change the pond render"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}
