//! Stage G engine-integration gates (procedural texture
//! pipeline §12, the v2 carrier half): prop models in
//! materials.json v2 — voxelizer emission and the engine loader
//! round-trip.
//!
//! * Emission + round-trip (CPU): a `--v2` run over the
//!   voxelizer's hermetic fixture tileset emits a `models` entry
//!   for f_bookcase (its fixture sprite segments); f_table's
//!   fixture sprite does not segment and gets NO entry — it
//!   keeps the extruded-sprite path. The engine loader joins the
//!   model to its material, and every voxel color matches a
//!   directly built `solid_model_for` exactly.
//! * v2 without models: a run over a grass-only tileset (nothing
//!   can segment) writes NO `models` key — the file keeps the
//!   Stage F v2 shape byte-for-byte in structure — and loads
//!   with every material model-less.
//! * v1 safety: the default run stays a byte-identical bare
//!   array across runs (the engine golden gate separately proves
//!   the renderers are untouched by the loader change).
//!
//! NOT proven here, by design: world stamping. The engine's
//! world grid is one block per tile footprint (mesher
//! `WorldScale`: 1 tile = 1 block, 3 blocks per z-level); a
//! 32-wide prop model has no sub-tile grid to land in either
//! renderer. The loader half lands complete and validated so
//! stamping can start the moment a sub-tile world
//! representation exists; the orientation contract is pinned in
//! `engine::materials` module docs.

use std::path::{Path, PathBuf};

use engine::materials::MaterialTable;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../voxelizer/tests/fixture")
}

/// Run the voxelizer over the fixture tileset into `out`
/// (created fresh) and return the output dir.
fn build_assets(out: &Path, emit_v2: bool) -> PathBuf {
    let _ = std::fs::remove_dir_all(out);
    std::fs::create_dir_all(out).unwrap();
    let ts = tileset::load_decomposed(&fixture_dir()).expect("fixture tileset loads");
    let cfg = voxelizer::VoxelizerConfig {
        emit_v2,
        ..Default::default()
    };
    voxelizer::run(&ts, &cfg, out).expect("voxelizer run");
    out.to_path_buf()
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn material_index(mats: &serde_json::Value, id: &str) -> u64 {
    mats.as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == id)
        .and_then(|m| m["material_index"].as_u64())
        .unwrap_or_else(|| panic!("fixture materials include {id}"))
}

#[test]
fn stage_g_model_round_trip_fixture() {
    let base = std::env::temp_dir().join(format!("stage_g_assets_{}", std::process::id()));
    let v2 = build_assets(&base.join("v2"), true);
    let json = read_json(&v2.join("materials.json"));
    assert_eq!(json["version"], 2);
    let models = json["models"]
        .as_array()
        .expect("v2 over the fixture carries a models table (f_bookcase segments)");

    // Every emitted model belongs to a furniture material.
    let mats = &json["materials"];
    for e in models {
        let mi = e["material_index"].as_u64().unwrap();
        let mat = mats
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["material_index"].as_u64() == Some(mi))
            .expect("model's material exists");
        assert!(
            mat["id"].as_str().unwrap().starts_with("f_"),
            "models are furniture-only, got {}",
            mat["id"]
        );
    }

    // f_bookcase has a model entry; f_table (no segmentation) does not.
    let bookcase_idx = material_index(mats, "f_bookcase");
    let table_idx = material_index(mats, "f_table");
    let entry = models
        .iter()
        .find(|e| e["material_index"].as_u64() == Some(bookcase_idx))
        .expect("f_bookcase model emitted");
    assert!(
        models
            .iter()
            .all(|e| e["material_index"].as_u64() != Some(table_idx)),
        "f_table does not segment: no model entry"
    );

    // The expected model, built directly from the fixture sprite.
    let ts = tileset::load_decomposed(&fixture_dir()).expect("fixture tileset loads");
    let f_entry = ts.entries.get("f_bookcase").expect("f_bookcase entry");
    let sprite = voxelizer::composite_entry(&ts, f_entry).expect("sprite composites");
    let seg = voxelizer::segment_for("f_bookcase", &sprite).expect("fixture bookcase segments");
    let expected = voxelizer::solid_model_for("f_bookcase", &sprite, &seg);
    assert_eq!(
        entry["size"].as_array().unwrap().len(),
        3,
        "model entry carries a 3-element size"
    );
    assert_eq!(
        entry["size"][0].as_u64(),
        Some(expected.size[0] as u64),
        "emitted size matches the direct build"
    );

    // Loader round-trip: identical size and EVERY voxel color.
    let table = MaterialTable::load(&v2).expect("v2 with models loads");
    let mat = table.get("f_bookcase").expect("f_bookcase material");
    let loaded = mat.model.as_ref().expect("loader joined the model");
    assert_eq!(loaded.size, expected.size, "round-tripped size");
    assert_eq!(
        loaded.solid_count(),
        expected.solid_count(),
        "round-tripped solid count"
    );
    let [w, h, d] = expected.size;
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                let want = expected.get(x, y, z).map(|p| [p[0], p[1], p[2]]);
                assert_eq!(
                    loaded.get(x, y, z),
                    want,
                    "voxel ({x},{y},{z}) round-trips exactly"
                );
            }
        }
    }
    // The unsegmented furniture stays model-less through the loader.
    assert!(
        table.get("f_table").unwrap().model.is_none(),
        "f_table keeps the extruded-sprite path"
    );

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn stage_g_v2_without_models_keeps_stage_f_shape() {
    // A grass-only tileset: nothing can segment, so the v2 file
    // must carry no `models` key at all.
    let base = std::env::temp_dir().join(format!("stage_g_nofurn_{}", std::process::id()));
    let mini = base.join("tileset");
    let png_dir = mini.join("pngs_normal_32x32");
    std::fs::create_dir_all(&png_dir).unwrap();
    let fx = fixture_dir();
    for f in ["tile_info.json"] {
        std::fs::copy(fx.join(f), mini.join(f)).unwrap();
    }
    for f in ["t_grass.json", "grass_fg.png", "grass_bg.png"] {
        std::fs::copy(fx.join("pngs_normal_32x32").join(f), png_dir.join(f)).unwrap();
    }
    let ts = tileset::load_decomposed(&mini).expect("mini tileset loads");
    assert_eq!(ts.entries.len(), 1, "mini tileset is grass only");

    let out = base.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let cfg = voxelizer::VoxelizerConfig {
        emit_v2: true,
        ..Default::default()
    };
    voxelizer::run(&ts, &cfg, &out).expect("voxelizer run");
    let json = read_json(&out.join("materials.json"));
    let obj = json.as_object().expect("v2 is an object");
    let mut keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    keys.sort();
    assert_eq!(
        keys,
        ["macros", "materials", "version"],
        "no models key when nothing segments"
    );
    let table = MaterialTable::load(&out).expect("model-less v2 loads");
    assert!(
        table.get("t_grass").unwrap().model.is_none(),
        "grass has no prop model"
    );

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn stage_g_v1_default_stays_byte_identical() {
    let base = std::env::temp_dir().join(format!("stage_g_v1_{}", std::process::id()));
    let a = build_assets(&base.join("a"), false);
    let b = build_assets(&base.join("b"), false);
    let bytes_a = std::fs::read(a.join("materials.json")).unwrap();
    let bytes_b = std::fs::read(b.join("materials.json")).unwrap();
    assert_eq!(bytes_a, bytes_b, "v1 output is deterministic");
    let json: serde_json::Value = serde_json::from_slice(&bytes_a).unwrap();
    assert!(json.is_array(), "v1 materials.json is a bare array");
    let _ = std::fs::remove_dir_all(&base);
}
