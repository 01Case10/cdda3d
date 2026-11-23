//! Puddle water gate (terrain item 2 follow-up,
//! 2026-11-23, Joey's water-shader call): the v2
//! `surface: "water"` field must reach both renderers.
//!
//! CPU half (always runs):
//! - `stamp_snapshot_terrain` marks the water model's
//!   column-TOP cells (and only those) with
//!   `mesher::block::WATER_ALPHA`; an identical model
//!   with surface None marks nothing.
//! - `treebuild::world::VoxelWorld::from_fine_layer`
//!   turns the marker into `FLAG_WATER` on exactly the
//!   marked cells, with alpha restored to 1.0.
//! - `mesher::fine::mesh_chunk_fine` emits the puddle
//!   top faces with the marker alpha in their vertex
//!   colour (what the raster shader keys on).
//!
//! GPU half (loud NOT RUN banner without an adapter):
//! the same scene rendered by the raster path with and
//! without the water surface -- the water frame's
//! puddle area must be brighter (specular + fresnel
//! sheen), and the two frames must otherwise be the
//! same scene. Captures are written to
//! `target/puddle-water-gate/` for the sign-off sheet.

use std::path::PathBuf;

use engine::materials::{ModelKind, ModelSurface, PropModelData};
use engine::snapshot_world::{build_snapshot_world, stamp_snapshot_terrain};
use mesher::block::WATER_ALPHA;
use mesher::chunk::ChunkMap;
use mesher::fine::{mesh_chunk_fine, FineLayer};
use mesher::scale::WorldScale;
use mesher::{Block, Pass};
use proto::samples::{sample_terrain_snapshot, sample_terrain_tiledefs, TER_GRASS};

const TER_PUDDLE: u32 = 20;
/// Snapshot tile index of (x, y) in the 12x12 submap
/// (mirrors proto::samples' layout).
fn idx(x: usize, y: usize) -> usize {
    y * 12 + x
}

fn slab_model(h: u32, rgb: [u8; 3]) -> PropModelData {
    PropModelData {
        size: [32, h, 32],
        palette: vec![rgb],
        voxels: vec![0; (32 * h * 32) as usize],
    }
}

fn colour_block(rgb: [u8; 3]) -> Block {
    Block {
        id: 42,
        color: [
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
            1.0,
        ],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

fn coarse_block(id: u32) -> Block {
    Block {
        id,
        color: [0.5, 0.5, 0.5, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// One grass field with a 2x1 puddle patch at tiles
/// (5, 2) and (6, 2), stamped through the snapshot
/// path. The puddle model is a flat slab one cell
/// shorter than the grass slab; its surface is the
/// parameter under test.
fn scene(surface: ModelSurface) -> (ChunkMap, FineLayer) {
    let mut defs = sample_terrain_tiledefs();
    let grass_def = defs
        .defs
        .as_ref()
        .unwrap()
        .iter()
        .find(|d| d.id == TER_GRASS)
        .unwrap()
        .clone();
    let mut puddle_def = grass_def;
    puddle_def.id = TER_PUDDLE;
    puddle_def.string_id = Some("t_puddle".into());
    puddle_def.name = Some("t_puddle".into());
    defs.defs.as_mut().unwrap().push(puddle_def);

    let mut snap = sample_terrain_snapshot();
    {
        let terrain = snap.blocks.as_mut().unwrap()[0].terrain.as_mut().unwrap();
        terrain[idx(5, 2)] = TER_PUDDLE;
        terrain[idx(6, 2)] = TER_PUDDLE;
    }

    let grass = slab_model(10, [80, 140, 60]);
    let puddle = slab_model(9, [90, 74, 55]);
    let resolve = |id: &str| match id {
        "t_grass" => Some((&grass, ModelKind::Ground, ModelSurface::None)),
        "t_puddle" => Some((&puddle, ModelKind::Ground, surface)),
        _ => None,
    };
    let scale = WorldScale::default();
    let mut map = build_snapshot_world(&defs, &snap, scale, &coarse_block);
    let mut layer = FineLayer::new(96);
    stamp_snapshot_terrain(&mut map, &mut layer, &defs, &snap, scale, &resolve, &colour_block)
        .expect("stamp");
    (map, layer)
}

/// (marked, unmarked) stamped cells of the tile's
/// volume at tile (tx, ty).
fn marked_counts(layer: &FineLayer, tx: i32, ty: i32) -> (usize, usize) {
    let (mut marked, mut plain) = (0usize, 0usize);
    for fz in 0..96 {
        for ly in 0..32 {
            for lx in 0..32 {
                if let Some(b) = layer.block_at_fine(tx * 32 + lx, ty * 32 + ly, fz) {
                    if b.is_water_surface() {
                        marked += 1;
                    } else {
                        plain += 1;
                    }
                }
            }
        }
    }
    (marked, plain)
}

#[test]
fn water_surface_marks_only_model_tops() {
    let (_map, layer) = scene(ModelSurface::Water);
    // Puddle tiles: exactly the 32x32 top cells are
    // marked, the 8 fill layers beneath are not.
    for tx in [5, 6] {
        let (marked, plain) = marked_counts(&layer, tx, 2);
        assert_eq!(marked, 32 * 32, "puddle tile {tx}: top cells marked");
        assert_eq!(plain, 8 * 32 * 32, "puddle tile {tx}: fill unmarked");
    }
    // A grass tile: nothing marked anywhere.
    let (marked, _) = marked_counts(&layer, 0, 0);
    assert_eq!(marked, 0, "grass carries no water marker");

    // The same scene with surface None marks nothing.
    let (_map2, layer2) = scene(ModelSurface::None);
    let (marked2, _) = marked_counts(&layer2, 5, 2);
    assert_eq!(marked2, 0, "no surface, no marker");
}

#[test]
fn water_marker_becomes_rt_flag() {
    let (_map, layer) = scene(ModelSurface::Water);
    let world = treebuild::world::VoxelWorld::from_fine_layer(&layer);
    let mut water = 0usize;
    let mut other = 0usize;
    for (pos, cell) in world.iter() {
        if cell.flags & treebuild::world::FLAG_WATER != 0 {
            water += 1;
            assert_eq!(cell.alpha, 1.0, "water cell alpha restored at {pos:?}");
        } else {
            other += 1;
        }
    }
    assert_eq!(water, 2 * 32 * 32, "both puddle tops flagged");
    assert!(other > 0, "grass cells present and unflagged");
}

#[test]
fn water_marker_reaches_raster_vertices() {
    let (map, layer) = scene(ModelSurface::Water);
    let scale = WorldScale::default();
    let meshes = mesh_chunk_fine(&map, &layer, [0, 0], scale).expect("chunk meshes");
    let mut water_up_verts = 0usize;
    let mut plain_up_verts = 0usize;
    for pass in &meshes.meshes {
        for (normal, color) in pass.normals.iter().zip(pass.colors.iter()) {
            if normal[1] > 0.5 {
                if (color[3] - WATER_ALPHA).abs() < 0.01 {
                    water_up_verts += 1;
                } else {
                    plain_up_verts += 1;
                }
            }
        }
    }
    assert!(
        water_up_verts >= 4 * 2 * 32,
        "puddle top faces carry the marker into the mesh ({water_up_verts} verts)"
    );
    assert!(plain_up_verts > 0, "grass tops stay unmarked");
}

// ---------------------------------------------------------------------
// GPU half
// ---------------------------------------------------------------------

fn loud_skip(reason: &str) -> bool {
    let banner = format!(
        "================================================================\n\
         PUDDLE WATER GATE (GPU): NOT RUN — {reason}\n\
         This run proves NOTHING about the raster water shading. The\n\
         gate only counts on a machine with a wgpu adapter.\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
    true
}

#[test]
fn water_shading_brightens_puddle_raster() {
    use engine::render::Renderer;
    use glam::Vec3;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }));
    let Some(adapter) = adapter else {
        loud_skip("no wgpu adapter available");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("puddle-water-gate"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .expect("device");

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/puddle-water-gate");
    let _ = std::fs::create_dir_all(&out_dir);

    let size = (480u32, 270u32);
    let render = |surface: ModelSurface, name: &str| -> image::RgbaImage {
        let (map, layer) = scene(surface);
        let scale = WorldScale::default();
        let meshes = mesh_chunk_fine(&map, &layer, [0, 0], scale).expect("chunk meshes");
        let renderer = Renderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);
        let gpu_meshes = renderer.upload(&device, &meshes);
        let color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("puddle-water-color"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color_view = color.create_view(&Default::default());
        let (depth_texture, depth_view) =
            engine::render::create_depth_texture(&device, size);
        // Camera over the puddle patch (tiles 5-6, 2),
        // placed on the fixed light's reflection
        // direction about +Y (light (0.3, 0.8, 0.4) ->
        // reflection (-0.32, 0.85, -0.42)): yaw 217,
        // pitch 58 puts the whole flat puddle top inside
        // the specular highlight.
        let mut cam = engine::camera::OrbitCamera::new(Vec3::new(5.5, 0.0, -2.0));
        cam.yaw_deg = 217.0;
        cam.pitch_deg = 58.0;
        cam.distance = 8.0;
        renderer.set_view_projection(
            &queue,
            cam.view_projection(size.0 as f32 / size.1 as f32),
            cam.eye(),
        );
        renderer.draw(&device, &queue, &gpu_meshes, &color_view, &depth_view);
        device.poll(wgpu::Maintain::Wait);
        let _ = depth_texture;
        let (img, _) = engine::frame::readback_rgba(&device, &queue, &color, size)
            .expect("readback");
        img.save(out_dir.join(name)).expect("save capture");
        img
    };

    let water = render(ModelSurface::Water, "raster_water.png");
    let plain = render(ModelSurface::None, "raster_plain.png");

    let mean = |img: &image::RgbaImage| -> f64 {
        let mut sum = 0f64;
        let mut n = 0usize;
        for p in img.pixels() {
            sum += 0.299 * p.0[0] as f64 + 0.587 * p.0[1] as f64 + 0.114 * p.0[2] as f64;
            n += 1;
        }
        sum / n as f64
    };
    let (mw, mp) = (mean(&water), mean(&plain));
    assert!(
        mw > mp + 3.0,
        "water shading must brighten the frame (water {mw:.1} vs plain {mp:.1})"
    );
    // The geometry is identical: the silhouette (pixels
    // differing from the corner background) must match
    // almost exactly -- only the shading changed.
    let bg = plain.get_pixel(0, 0).0;
    let mut sil_diff = 0usize;
    for (a, b) in water.pixels().zip(plain.pixels()) {
        if (a.0 == bg) != (b.0 == bg) {
            sil_diff += 1;
        }
    }
    assert!(
        sil_diff < (size.0 * size.1) as usize / 100,
        "silhouette changed with shading ({sil_diff} px)"
    );
}
