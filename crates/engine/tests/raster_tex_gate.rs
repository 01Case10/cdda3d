//! Phase 2f engine-level gates (plan Part II §5f):
//! 2f.3 — shared selection: both renderer paths resolve materials
//! through the ONE builder (`textures::build_texture_spec`); this
//! test pins that the builder is deterministic and resolves the
//! fake house's materials to the fixture's layers.
//! 2f.2 — raster textured capture: same geometry as the flat
//! capture (silhouette IoU >= 0.99), materially more colors.
//! 2f.4 — no regression: the flat capture is deterministic
//! (byte-identical across runs) and unstamped meshes carry only
//! the flat layer sentinel. GPU halves are adapter-conditional in
//! the established pattern (lavapipe counts as an adapter).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::materials::MaterialTable;
use engine::rt::fake_material_ids;
use engine::textures::build_texture_spec;
use mesher::block::*;
use mesher::mesh::FLAT_LAYER;

const TILE: u32 = 8;
/// Fake tile ids in sorted order — the builder iterates ids sorted,
/// loading top/side/bottom in order, so with distinct source layers
/// per material the compact indices equal the source indices.
const FAKE_IDS_SORTED: [u32; 9] = [
    TILE_FLOOR_WOOD,
    TILE_WALL,
    TILE_DOOR,
    TILE_WINDOW,
    TILE_ROOF,
    TILE_GROUND,
    TILE_FENCE,
    TILE_WATER,
    TILE_FOLIAGE,
];

/// A tiny voxelizer-shaped fixture covering every fake-house
/// material: material k (in sorted id order) owns source layers
/// 3k (top), 3k+1 (side), 3k+2 (bottom); each layer is an 8x8
/// two-color checkerboard whose colors are unique to the layer, so
/// a textured render must show many more distinct colors than the
/// flat palette.
fn write_fixture(dir: &Path) {
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    let id_map = fake_material_ids();
    let layer_count = FAKE_IDS_SORTED.len() * 3;
    std::fs::write(
        dir.join("blocks.json"),
        format!(r#"{{"format":"png-layers","tile_size":{TILE},"layer_count":{layer_count}}}"#),
    )
    .unwrap();
    let mut mats = String::from("[");
    for (k, num_id) in FAKE_IDS_SORTED.iter().enumerate() {
        if k > 0 {
            mats.push(',');
        }
        mats.push_str(&format!(
            r#"{{"id":"{}","resolved_id":"{}","resolution":"exact","material_index":{k},"top_layer":{},"side_layer":{},"bottom_layer":{},"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":false,"subtile_layers":{{}}}}"#,
            id_map[num_id],
            id_map[num_id],
            3 * k,
            3 * k + 1,
            3 * k + 2,
        ));
    }
    mats.push(']');
    std::fs::write(dir.join("materials.json"), mats).unwrap();
    for li in 0..layer_count {
        let c1 = image::Rgba([
            (li as u32 * 37 % 256) as u8,
            (li as u32 * 91 % 256) as u8,
            (li as u32 * 53 % 256) as u8,
            255,
        ]);
        let c2 = image::Rgba([255 - c1.0[0], 255 - c1.0[1], 255 - c1.0[2], 255]);
        let img =
            image::RgbaImage::from_fn(TILE, TILE, |x, y| if (x + y) % 2 == 0 { c1 } else { c2 });
        img.save(blocks.join(format!("layer_{li:04}.png"))).unwrap();
    }
}

fn fixture_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("engine_raster_tex_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_fixture(&dir);
    dir
}

// ---------------------------------------------------------------- 2f.3
#[test]
fn gate_2f_3_shared_spec_builder_is_deterministic() {
    let dir = fixture_dir();
    let table = MaterialTable::load(&dir).unwrap();
    let id_map: HashMap<u32, String> = fake_material_ids();
    let (spec_a, stats_a) = build_texture_spec(&table, &dir, &id_map).unwrap();
    let (spec_b, stats_b) = build_texture_spec(&table, &dir, &id_map).unwrap();
    assert_eq!(stats_a, stats_b);
    assert_eq!(stats_a.missing_materials, 0, "all fake materials resolve");
    assert_eq!(stats_a.textured_materials, FAKE_IDS_SORTED.len());
    // The one builder both paths call (RT: rt::attach_fake_textures,
    // raster: capture's textured branch) resolves identically.
    assert_eq!(spec_a.material_layers, spec_b.material_layers);
    assert_eq!(spec_a.layers, spec_b.layers);
    assert_eq!(spec_a.tile_size, TILE);
    // Material -> layers resolution matches the fixture assignment:
    // wall (id 2, second in sorted order) owns layers 3/4/5.
    assert_eq!(spec_a.material_layers[TILE_WALL as usize], [3, 4, 5, 1]);
    assert_eq!(
        spec_a.material_layers[TILE_FLOOR_WOOD as usize],
        [0, 1, 2, 1]
    );
    assert_eq!(spec_a.material_layers[TILE_ROOF as usize], [12, 13, 14, 1]);
    // Unmapped ids stay flat.
    assert_eq!(spec_a.material_layers[0], [0, 0, 0, 0]);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------- 2f.4 (data half)
#[test]
fn gate_2f_4_unstamped_meshes_are_all_flat() {
    let (_map, meshes, _stats) = engine::world::mesh_fake_house(&[0, 1]);
    assert!(meshes.total_quads() > 0);
    for mesh in meshes.meshes.iter() {
        assert_eq!(mesh.layers.len(), mesh.positions.len());
        assert!(
            mesh.layers.iter().all(|&l| l == FLAT_LAYER),
            "without --textures every vertex must carry the flat sentinel"
        );
    }
}

// ------------------------------------------------------- 2f.2 + 2f.4 (GPU)
fn capture_args(textures: Option<PathBuf>, out: &Path) -> CaptureArgs {
    CaptureArgs {
        snapshot: "stub-house".to_string(),
        renderer: RendererKind::Raster,
        textures,
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        size: (320, 180),
        out: out.to_path_buf(),
        cutaway_z: 1,
        ..CaptureArgs::default()
    }
}

/// Geometry mask: any pixel that differs from the image's most
/// common color (the clear color) — same method as rt_gate.
fn silhouette(img: &image::RgbaImage) -> Vec<bool> {
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for p in img.pixels() {
        *counts.entry(p.0).or_default() += 1;
    }
    let bg = counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .map(|(c, _)| *c)
        .unwrap_or([0, 0, 0, 0]);
    img.pixels().map(|p| p.0 != bg).collect()
}

fn iou(a: &[bool], b: &[bool]) -> f32 {
    let (mut inter, mut union) = (0usize, 0usize);
    for (x, y) in a.iter().zip(b.iter()) {
        inter += (*x && *y) as usize;
        union += (*x || *y) as usize;
    }
    if union == 0 {
        1.0
    } else {
        inter as f32 / union as f32
    }
}

#[test]
fn gate_2f_2_and_2f_4_raster_textured_capture() {
    let dir = std::env::temp_dir().join(format!("engine_raster_cap_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let flat_path = dir.join("flat.png");
    let flat = match run_capture(&capture_args(None, &flat_path)) {
        Ok(o) => o,
        Err(CaptureError::NoAdapter) => {
            eprintln!("2f.2/2f.4: no wgpu adapter on this machine — GPU checks NOT RUN");
            return;
        }
        Err(e) => panic!("flat raster capture failed: {e}"),
    };

    // 2f.4: the flat capture is deterministic — a second run is
    // byte-identical (the strongest in-test proxy for "unchanged
    // vs the pre-2f golden": the flat shader path is untouched by
    // construction, flat sentinel -> pre-2f math).
    let flat2_path = dir.join("flat2.png");
    let _flat2 = run_capture(&capture_args(None, &flat2_path)).expect("flat capture 2");
    assert_eq!(
        std::fs::read(&flat_path).unwrap(),
        std::fs::read(&flat2_path).unwrap(),
        "flat captures must be byte-identical across runs"
    );

    // 2f.2 with the hermetic fixture.
    let tex_dir = fixture_dir();
    let tex_path = dir.join("textured.png");
    let tex = run_capture(&capture_args(Some(tex_dir.clone()), &tex_path))
        .expect("textured raster capture");
    assert_eq!(
        tex.quads, flat.quads,
        "textures shade, they must not change geometry"
    );
    let flat_img = image::open(&flat_path).unwrap().to_rgba8();
    let tex_img = image::open(&tex_path).unwrap().to_rgba8();
    let score = iou(&silhouette(&flat_img), &silhouette(&tex_img));
    println!(
        "2f.2 fixture: silhouette IoU {score:.3}; distinct colors flat {} -> textured {}",
        flat.distinct_colors, tex.distinct_colors
    );
    assert!(score >= 0.99, "silhouette IoU {score:.3} < 0.99");
    assert!(
        tex.distinct_colors > flat.distinct_colors + 20,
        "textured capture must be materially richer: flat {} -> textured {}",
        flat.distinct_colors,
        tex.distinct_colors
    );

    // 2f.2 with the real UltiCa voxelizer output, when present
    // (same conditional pattern as rt_gate's texture test): the
    // raster path must show the same materials the RT textured
    // capture shows — same builder, same fake-id resolution.
    let ultica = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1");
    if ultica.join("materials.json").exists() {
        let u_path = dir.join("textured_ultica.png");
        let u = run_capture(&capture_args(Some(ultica), &u_path))
            .expect("UltiCa textured raster capture");
        let u_img = image::open(&u_path).unwrap().to_rgba8();
        let u_score = iou(&silhouette(&flat_img), &silhouette(&u_img));
        println!(
            "2f.2 ultica: silhouette IoU {u_score:.3}; distinct colors flat {} -> textured {}",
            flat.distinct_colors, u.distinct_colors
        );
        assert!(u_score >= 0.99, "UltiCa silhouette IoU {u_score:.3} < 0.99");
        assert_eq!(u.quads, flat.quads);
        assert!(
            u.distinct_colors > flat.distinct_colors,
            "UltiCa textures must enrich the raster capture"
        );
    } else {
        eprintln!("2f.2 ultica: assets/ultica_run1 not present — fixture half only");
    }
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&tex_dir);
}
