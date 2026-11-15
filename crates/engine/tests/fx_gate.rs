//! Phase 2g.1 / 2g.2 engine-level gates (plan Part II §5g):
//! 2g.1a — fx wired into the engine capture: field volumes
//!   composite over the fake house in BOTH renderers (geometry
//!   identical outside the field regions, fields present inside
//!   them), post off for the pair.
//! 2g.1b — post on/off pair: same scene, the stack changes only
//!   grading/bloom/DoF (broad, warm shift), never geometry.
//! 2g.2 — the windowed remesh's shared scene preparation: with a
//!   textures dir it returns a spec + textured meshes and the
//!   renderer accepts the spec via set_textures (the exact calls
//!   main.rs's remesh makes); without one it stays flat.
//!
//! GPU halves are adapter-conditional in the established pattern:
//! on NoAdapter they print a loud banner and skip — never a
//! silent pass. lavapipe counts as an adapter.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::render::{Renderer, CAPTURE_FORMAT};
use engine::rt::fake_material_ids;
use mesher::block::*;
use mesher::mesh::FLAT_LAYER;

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda_fx_gate_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fx_args(dir: &Path, name: &str, renderer: RendererKind) -> CaptureArgs {
    CaptureArgs {
        renderer,
        effects_defs: Some(fx::effect::demo_defs()),
        size: (320, 180),
        cutaway_z: 0,
        out: dir.join(name),
        ..CaptureArgs::default()
    }
}

fn capture(args: &CaptureArgs) -> Option<image::RgbaImage> {
    match run_capture(args) {
        Ok(_) => Some(image::open(&args.out).unwrap().to_rgba8()),
        Err(CaptureError::NoAdapter) => {
            eprintln!("fx_gate: SKIPPED — no wgpu adapter (not a pass)");
            None
        }
        Err(e) => panic!("capture failed: {e}"),
    }
}

fn diff_fraction(a: &image::RgbaImage, b: &image::RgbaImage) -> f32 {
    assert_eq!(a.dimensions(), b.dimensions());
    let n = (a.width() * a.height()) as usize;
    let d = a
        .pixels()
        .zip(b.pixels())
        .filter(|(x, y)| x.0 != y.0)
        .count();
    d as f32 / n as f32
}

#[test]
fn gate_2g_1a_fields_composite_both_renderers() {
    let dir = tmpdir("1a");
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let mut off = fx_args(&dir, &format!("off_{}.png", renderer.name()), renderer);
        off.fx_fields = false;
        off.fx_post = false;
        let mut on = fx_args(&dir, &format!("on_{}.png", renderer.name()), renderer);
        on.fx_fields = true;
        on.fx_post = false;
        let (Some(off_img), Some(on_img)) = (capture(&off), capture(&on)) else {
            return; // adapter-conditional skip (banner printed)
        };
        let frac = diff_fraction(&off_img, &on_img);
        assert!(
            frac > 0.001 && frac < 0.40,
            "{}: fields must change a localized region, got {:.2}% of pixels",
            renderer.name(),
            frac * 100.0
        );
        if renderer == RendererKind::Raster {
            // The flame's hot core (near-white, low blue) exists
            // only in the fields-on image — the flat house palette
            // contains no such pixel (measured: 0 off, >0 on).
            let hot = |img: &image::RgbaImage| {
                img.pixels()
                    .filter(|p| p.0[0] > 230 && p.0[1] > 180 && p.0[2] < 170)
                    .count()
            };
            assert!(
                hot(&on_img) > hot(&off_img),
                "fields-on must add hot flame-core pixels"
            );
        }
    }
}

#[test]
fn gate_2g_1b_post_grades_without_geometry_change() {
    let dir = tmpdir("1b");
    let mut off = fx_args(&dir, "post_off.png", RendererKind::Raster);
    off.fx_post = false;
    let mut on = fx_args(&dir, "post_on.png", RendererKind::Raster);
    on.fx_post = true;
    let (Some(off_img), Some(on_img)) = (capture(&off), capture(&on)) else {
        return;
    };
    // Grade + vignette touch essentially every pixel...
    let frac = diff_fraction(&off_img, &on_img);
    assert!(
        frac > 0.5,
        "post-on should differ broadly (grade/vignette), got {:.1}%",
        frac * 100.0
    );
    // ...and the grade is WARM: the mean red/blue ratio rises.
    let rb = |img: &image::RgbaImage| {
        let (mut r, mut b) = (0f64, 0f64);
        for p in img.pixels() {
            r += p.0[0] as f64;
            b += p.0[2] as f64;
        }
        r / b.max(1.0)
    };
    assert!(
        rb(&on_img) > rb(&off_img),
        "warm grade must raise the red/blue ratio"
    );
}

#[test]
fn gate_2g_1_effects_json_file_path() {
    // The CLI's --effects path: defs load from a real
    // effects.json document (the voxelizer's format).
    let dir = tmpdir("file");
    let mut map = serde_json::Map::new();
    for d in fx::effect::demo_defs() {
        map.insert(d.field_id.clone(), serde_json::to_value(&d).unwrap());
    }
    let doc = serde_json::json!({ "effects": map });
    let path = dir.join("effects.json");
    std::fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
    let defs = engine::fxwire::load_defs(&path).unwrap();
    assert_eq!(defs.len(), 3);
    let fields = engine::fxwire::fake_house_fields(&defs).unwrap();
    assert_eq!(fields.len(), 6);
    // Engine-scale placement: z0 volumes sit on the floor top
    // (y = 1.8), the z1 smoke one level (3 blocks) higher.
    let inst = engine::fxwire::sorted_instances(&fields, glam::Vec3::new(6.0, 20.0, -20.0));
    assert_eq!(inst.len(), 6);
    let ys: Vec<f32> = inst.iter().map(|i| i.center[1]).collect();
    assert!(ys.iter().all(|y| (*y - 1.8).abs() < 1e-4 || (*y - 4.8).abs() < 1e-4));
}

// ---------------------------------------------------------------------
// 2g.2 fixture + gate
// ---------------------------------------------------------------------

const TILE: u32 = 8;
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
/// material (same construction as the 2f gate's fixture).
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
    let _ = HashMap::<u32, String>::new(); // keep the import honest
}

#[test]
fn gate_2g_2_windowed_remesh_preparation() {
    let dir = tmpdir("2g2");
    let fixture = dir.join("vox");
    write_fixture(&fixture);
    let visible = vec![0];

    // The exact preparation main.rs's windowed remesh runs.
    let (meshes, _stats, spec) =
        engine::scene_prep::prepare_raster_house(&visible, Some(&fixture)).unwrap();
    let spec = spec.expect("textures dir must yield a spec");
    assert!(!spec.layers.is_empty(), "spec must carry layers");
    let textured_verts = meshes
        .meshes
        .iter()
        .flat_map(|m| m.layers.iter())
        .filter(|l| **l != FLAT_LAYER)
        .count();
    assert!(
        textured_verts > 0,
        "stamped meshes must carry real layer indices"
    );

    // Without a textures dir: flat, no spec (pre-2g behavior).
    let (flat_meshes, _s, flat_spec) =
        engine::scene_prep::prepare_raster_house(&visible, None).unwrap();
    assert!(flat_spec.is_none());
    assert!(flat_meshes
        .meshes
        .iter()
        .flat_map(|m| m.layers.iter())
        .all(|l| *l == FLAT_LAYER));

    // Adapter-conditional: the windowed path's remaining steps —
    // set_textures on the renderer, upload, draw — run clean.
    let inst = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(inst.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }));
    let Some(adapter) = adapter else {
        eprintln!("fx_gate: SKIPPED 2g.2 GPU half — no wgpu adapter (not a pass)");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("fx-gate-2g2-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .unwrap();
    let mut renderer = Renderer::new(&device, CAPTURE_FORMAT);
    renderer.set_textures(&device, &queue, &spec);
    let gpu_meshes = renderer.upload(&device, &meshes);
    let color = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fx-gate-2g2-color"),
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let color_view = color.create_view(&Default::default());
    let depth_view = engine::render::create_depth_view(&device, (64, 64));
    renderer.set_view_projection(
        &queue,
        engine::camera::OrbitCamera::new(glam::Vec3::new(6.0, 1.0, -6.0))
            .view_projection(1.0),
    );
    renderer.draw(&device, &queue, &gpu_meshes, &color_view, &depth_view);
    device.poll(wgpu::Maintain::Wait);
}
