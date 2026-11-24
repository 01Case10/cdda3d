//! Stage F part 1 gates (procedural texture pipeline —
//! docs/procedural-texture-pipeline.md §10.2): materials.json v2
//! emission, the engine loader, and world-space macro ground
//! sampling in both renderers.
//!
//! * Voxelizer emission (CPU): the default run stays a v1 bare
//!   array, byte-identical across runs; `--v2` (emit_v2) wraps the
//!   SAME material array with a macros table, and the fixture's
//!   t_grass (Grass class, pooled family) carries a macro whose
//!   color/height PNGs are 128x128.
//! * Loader (CPU): MaterialTable reads the v2 set with
//!   macro_size 128 and t_grass macro present; the v1 set loads
//!   with no macros. (Version rejection is unit-tested in
//!   materials.rs.)
//! * Mesher macro UVs (CPU): a top face stamped with a macro layer
//!   word gets world-based UVs — an 8-tile slab spans u in [0, 2]
//!   at span 4, and the same slab meshed 4 tiles east starts at
//!   u = 1: identical macro phase 4 tiles apart.
//! * GPU macro proof: sample-scene captures with v2 assets differ
//!   from v1 captures on the grass in BOTH renderers, and the
//!   `--macro-debug-height` capture renders the yard as height
//!   grayscale (differs from the color capture; large
//!   near-grayscale population).
//!
//! GPU halves follow the established adapter-conditional pattern:
//! with no wgpu adapter they print a named NOT-RUN banner and
//! return; a blank render is never a pass. Asset sets are built
//! from the voxelizer's hermetic fixture tileset (it carries
//! t_grass and t_dirt).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use image::GenericImageView as _;
use engine::materials::MaterialTable;
use mesher::block::{Block, Pass};
use mesher::grid::VoxelGrid;
use mesher::mesher::mesh_greedy;
use mesher::mesh::encode_macro_layer;
use mesher::scale::WorldScale;

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

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

// ------------------------------------------------------- emission
#[test]
fn stage_f_voxelizer_v1_default_and_v2_emission() {
    let base = std::env::temp_dir().join(format!("stage_f_assets_{}", std::process::id()));
    let v1a = build_assets(&base.join("v1a"), false);
    let v1b = build_assets(&base.join("v1b"), false);
    let v2 = build_assets(&base.join("v2"), true);

    // v1: bare array, no version field, byte-identical across runs.
    let v1_bytes_a = std::fs::read(v1a.join("materials.json")).unwrap();
    let v1_bytes_b = std::fs::read(v1b.join("materials.json")).unwrap();
    assert_eq!(v1_bytes_a, v1_bytes_b, "v1 output must be deterministic");
    let v1_json: serde_json::Value = serde_json::from_slice(&v1_bytes_a).unwrap();
    assert!(v1_json.is_array(), "v1 materials.json is a bare array");
    assert!(
        !v1a.join("macro_0000_color.png").exists(),
        "v1 output writes no macro layers"
    );

    // v2: version 2, the SAME materials array, plus a macros table.
    let v2_json = read_json(&v2.join("materials.json"));
    assert_eq!(v2_json["version"], 2);
    assert_eq!(
        v2_json["materials"], v1_json,
        "v2 wraps the identical material array"
    );
    let macros = v2_json["macros"].as_array().expect("macros table");
    assert!(!macros.is_empty(), "fixture has poolable ground families");

    // t_grass carries a macro with span 4 and 128px PNG layers.
    let grass_index = v2_json["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "t_grass")
        .and_then(|m| m["material_index"].as_u64())
        .expect("fixture materials include t_grass with an index");
    let entry = macros
        .iter()
        .find(|m| m["material_index"].as_u64() == Some(grass_index))
        .expect("t_grass has a macro entry");
    assert_eq!(entry["span_tiles"], 4);
    let color_layer = entry["color_layer"].as_u64().unwrap();
    let height_layer = entry["height_layer"].as_u64().unwrap();
    for (layer, kind) in [(color_layer, "color"), (height_layer, "height")] {
        let p = v2.join(format!("macro_{layer:04}_{kind}.png"));
        let img = image::open(&p).unwrap_or_else(|e| panic!("{p:?}: {e}"));
        assert_eq!(img.dimensions(), (128, 128), "macro PNG is 128px");
    }

    // Loader: v2 yields macro_size 128 + t_grass macro; v1 none.
    let table2 = MaterialTable::load(&v2).expect("v2 loads");
    assert_eq!(table2.macro_size, 128);
    let grass = table2.get("t_grass").expect("t_grass material");
    let m = grass.macro_.expect("t_grass macro joined by the loader");
    assert_eq!(m.span_tiles, 4);
    let table1 = MaterialTable::load(&v1a).expect("v1 loads");
    assert_eq!(table1.macro_size, 0);
    assert!(table1.get("t_grass").unwrap().macro_.is_none());

    let _ = std::fs::remove_dir_all(&base);
}

// ----------------------------------------------------- mesher UVs
#[test]
fn stage_f_mesher_macro_top_uvs_are_world_based() {
    let block = Block {
        id: 7,
        color: [0.4, 0.6, 0.3, 1.0],
        pass: Pass::Opaque,
        tex_layers: Some([encode_macro_layer(3, 4), 1, 2]),
    };
    // Two 8-tile slabs: one at world x 0..8 (u in [0, 2]) and one
    // at world x 4..12 (u in [1, 3]) — the second starts exactly
    // one span east, so its macro phase equals the first's.
    for (slab_x, want_min, want_max) in [(0usize, 0.0f32, 2.0f32), (4, 1.0, 3.0)] {
        let mut grid = VoxelGrid::new([12, 1, 1]);
        for x in slab_x..slab_x + 8 {
            grid.set(x, 0, 0, block);
        }
        let meshes = mesh_greedy(&grid, [0, 0, 0], grid.size, WorldScale::default());
        let mesh = &meshes.meshes[Pass::Opaque.index()];
        // World space is Y-up (grid +z maps to world +y), so the
        // top face is the one with world normal +Y.
        let mut top_uvs: Vec<[f32; 2]> = Vec::new();
        for (i, n) in mesh.normals.iter().enumerate() {
            if n[1] > 0.5 {
                top_uvs.push(mesh.uvs[i]);
            }
        }
        assert_eq!(top_uvs.len(), 4, "one merged top quad, 4 corners");
        let u_min = top_uvs.iter().map(|uv| uv[0]).fold(f32::MAX, f32::min);
        let u_max = top_uvs.iter().map(|uv| uv[0]).fold(f32::MIN, f32::max);
        assert!(
            (u_min - want_min).abs() < 1e-6 && (u_max - want_max).abs() < 1e-6,
            "slab x={slab_x}: top u in [{u_min}, {u_max}], want [{want_min}, {want_max}] \
             (world x / span 4; 4 tiles east = same macro phase)"
        );
    }
}

// ------------------------------------------------------------ GPU
fn capture(args: &CaptureArgs) -> Option<image::RgbaImage> {
    match run_capture(args) {
        Ok(outcome) => {
            assert!(outcome.quads > 0, "blank render is not a pass");
            Some(image::open(&outcome.out).unwrap().to_rgba8())
        }
        Err(CaptureError::NoAdapter) => {
            eprintln!(
                "STAGE-F GATE NOT RUN: no wgpu adapter (this gate proves nothing without one)"
            );
            None
        }
        Err(e) => panic!("capture failed: {e}"),
    }
}

fn sample_args(renderer: RendererKind, textures: &Path, out: &Path) -> CaptureArgs {
    CaptureArgs {
        snapshot: "sample-scene".to_string(),
        renderer,
        textures: Some(textures.to_path_buf()),
        size: (256, 144),
        out: out.to_path_buf(),
        ..CaptureArgs::default()
    }
}

fn diff_pixels(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    assert_eq!(a.dimensions(), b.dimensions());
    a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count()
}

fn near_grayscale_pixels(img: &image::RgbaImage) -> usize {
    img.pixels()
        .filter(|p| {
            let [r, g, b, _] = p.0;
            (r as i32 - g as i32).abs() <= 2 && (g as i32 - b as i32).abs() <= 2
        })
        .count()
}

#[test]
fn stage_f_macro_sampling_both_renderers() {
    let _guard = gpu_lock();
    let base = std::env::temp_dir().join(format!("stage_f_gpu_{}", std::process::id()));
    let v1 = build_assets(&base.join("v1"), false);
    let v2 = build_assets(&base.join("v2"), true);
    let outdir = base.join("caps");
    std::fs::create_dir_all(&outdir).unwrap();

    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let name = renderer.name();
        let img_v1 = capture(&sample_args(renderer, &v1, &outdir.join(format!("{name}_v1.png"))));
        let Some(img_v1) = img_v1 else { continue };
        let img_v2 = capture(&sample_args(renderer, &v2, &outdir.join(format!("{name}_v2.png"))))
            .expect("v1 capture ran, so the adapter exists");
        let diff = diff_pixels(&img_v1, &img_v2);
        assert!(
            diff > 500,
            "{name}: v2 macro sampling must change the ground render ({diff} px differ)"
        );

        let mut dbg_args = sample_args(renderer, &v2, &outdir.join(format!("{name}_dbg.png")));
        dbg_args.macro_debug_height = true;
        let img_dbg = capture(&dbg_args).expect("v1 capture ran, so the adapter exists");
        let ddiff = diff_pixels(&img_v2, &img_dbg);
        assert!(
            ddiff > 500,
            "{name}: height-debug capture must differ from the color capture ({ddiff} px)"
        );
        let (gray_dbg, gray_col) = (near_grayscale_pixels(&img_dbg), near_grayscale_pixels(&img_v2));
        assert!(
            gray_dbg > 2000 && gray_dbg > gray_col,
            "{name}: height view is height grayscale (gray px: debug {gray_dbg} vs color {gray_col})"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}
