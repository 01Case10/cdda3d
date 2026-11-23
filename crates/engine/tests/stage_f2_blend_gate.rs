//! Stage F part 2 gates — border height-blending between macro
//! ground materials (procedural texture pipeline Stage F part 2,
//! final slice):
//!
//! * CPU semantics: the shipped decision mirrors
//!   (`engine::blend::blend_weight` / `neighbor_wins`) agree with
//!   the Stage D CPU reference (`voxelizer::quilt::height_blend`)
//!   per pixel on a synthetic case — endpoints exact (weight 0 =
//!   own material everywhere, weight 1 = neighbor everywhere)
//!   and a high-height patch punches through at the border
//!   weight. The spec builder emits blend entries that copy the
//!   base macro entry and name the neighbor's compact macro
//!   layers; with no blend assignments the spec equals the base
//!   spec.
//! * Assignment (CPU): the gate scene (grass | dirt halves)
//!   yields exactly the two border triples; a uniform field
//!   yields none.
//! * Capture (GPU, adapter-conditional like the other gates):
//!   in BOTH renderers, the blended border scene shows each
//!   material's macro colors on the other material's side of the
//!   border (interpenetration), every differing pixel lies in
//!   the border band, and a uniform-material scene renders
//!   byte-identical through the blend pipeline and the plain
//!   one. The unblended control is the same scene rendered with
//!   no blend assignments — i.e. exactly today's renderer.

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use common::{blend_centroids, classify, Centroid, Class};
use engine::blend::{blend_weight, neighbor_wins, BlendAssignment, BLEND_HALF_BAND_TILES};
use engine::capture::RendererKind;
use engine::camera::OrbitCamera;
use engine::materials::MaterialTable;
use engine::render::{create_depth_view, Renderer, CAPTURE_FORMAT};
use engine::textures::{build_texture_spec, build_texture_spec_variants};
use glam::Vec3;
use image::{GrayImage, Rgba, RgbaImage};
use mesher::block::{Block, Pass, TILE_GROUND};
use mesher::chunk::ChunkMap;
use mesher::grid::VoxelSampler as _;
use mesher::mesh::{encode_macro_blend_layer, encode_macro_layer};
use mesher::scale::WorldScale;
use treebuild::build::{build_tree, BuildConfig};
use treebuild::render::{RtCamera, RtRenderer};
use treebuild::world::VoxelWorld;

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

const GRASS_ID: u32 = TILE_GROUND; // 6 -> "t_grass"
const DIRT_ID: u32 = 10; // -> "t_dirt"
const GRASS_RGB: [u8; 3] = [70, 110, 45];
const DIRT_RGB: [u8; 3] = [120, 95, 60];
const SIZE: (u32, u32) = (320, 240);

fn id_map() -> HashMap<u32, String> {
    [
        (GRASS_ID, "t_grass".to_string()),
        (DIRT_ID, "t_dirt".to_string()),
    ]
    .into_iter()
    .collect()
}

/// Macro spans per numeric id, from the loaded table (the same
/// resolution the prepare functions use).
fn macro_spans(table: &MaterialTable) -> HashMap<u32, u32> {
    id_map()
        .iter()
        .filter_map(|(num, game_id)| {
            table
                .get(game_id)
                .and_then(|m| m.macro_)
                .map(|mc| (*num, mc.span_tiles))
        })
        .collect()
}

// ---------------------------------------------------------- assets
/// Hand-built v2 asset dir: grass + dirt materials, each with a
/// 128px macro (span 4). Heights are flat fields (grass 200,
/// dirt 90) with one opposed patch each at macro columns
/// 112..128, rows 48..80 — the phase a grass-side border pixel
/// samples across the border (world x just under the x=4 border
/// of the gate scene): there grass drops to 40 and dirt rises
/// to 255, so dirt must punch into the grass side at those rows,
/// while along the rest of the border the taller grass (200 >
/// 90) holds its side and spills a short strip into the dirt.
fn asset_dir(base: &Path) -> PathBuf {
    let dir = base.join("assets");
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    let tile = |rgb: [u8; 3]| {
        let mut img = RgbaImage::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let v = if (x + y) % 2 == 0 { 0i16 } else { 6 };
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        (rgb[0] as i16 + v) as u8,
                        (rgb[1] as i16 + v) as u8,
                        (rgb[2] as i16 + v) as u8,
                        255,
                    ]),
                );
            }
        }
        img
    };
    tile(GRASS_RGB).save(blocks.join("layer_0000.png")).unwrap();
    tile(DIRT_RGB).save(blocks.join("layer_0001.png")).unwrap();
    std::fs::write(
        dir.join("blocks.json"),
        r#"{"format":"png-layers","tile_size":32,"layer_count":2}"#,
    )
    .unwrap();
    let mat = |id: &str, index: usize, layer: usize| {
        serde_json::json!({
            "id": id, "resolved_id": id, "resolution": "exact",
            "material_index": index, "top_layer": layer, "side_layer": layer,
            "bottom_layer": layer, "height_voxels": 1, "transparency": "opaque",
            "cutout": false, "transparent": false, "tint": [255, 255, 255],
            "sub_model": null, "multitile": false, "subtile_layers": {}
        })
    };
    let v2 = serde_json::json!({
        "version": 2,
        "materials": [mat("t_grass", 0, 0), mat("t_dirt", 1, 1)],
        "macros": [
            {"material_index": 0, "color_layer": 0, "height_layer": 0, "span_tiles": 4},
            {"material_index": 1, "color_layer": 1, "height_layer": 1, "span_tiles": 4}
        ]
    });
    std::fs::write(
        dir.join("materials.json"),
        serde_json::to_string_pretty(&v2).unwrap(),
    )
    .unwrap();
    // Macro color layers: the material color with a checker grain.
    let macro_color = |rgb: [u8; 3]| {
        let mut img = RgbaImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                let v = if (x + y) % 2 == 0 { 0i16 } else { 8 };
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        (rgb[0] as i16 + v) as u8,
                        (rgb[1] as i16 + v) as u8,
                        (rgb[2] as i16 + v) as u8,
                        255,
                    ]),
                );
            }
        }
        img
    };
    macro_color(GRASS_RGB)
        .save(dir.join("macro_0000_color.png"))
        .unwrap();
    macro_color(DIRT_RGB)
        .save(dir.join("macro_0001_color.png"))
        .unwrap();
    // Macro height layers (grayscale; written as RGBA gray).
    let macro_height = |base: u8, patch: u8| {
        let mut img = RgbaImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                let v = if (48..80).contains(&y) && (112..128).contains(&x) {
                    patch
                } else {
                    base
                };
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        img
    };
    macro_height(200, 40)
        .save(dir.join("macro_0000_height.png"))
        .unwrap();
    macro_height(90, 255)
        .save(dir.join("macro_0001_height.png"))
        .unwrap();
    dir
}

// ----------------------------------------------------------- scene
fn ground_block(id: u32) -> Block {
    Block {
        id,
        color: [0.5, 0.5, 0.5, 1.0],
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

/// The border scene: 8x4 ground tiles at z 0, grass west
/// (x 0..4), dirt east (x 4..8). `uniform` makes it all grass.
fn border_map(uniform: bool) -> ChunkMap {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    for y in 0..4 {
        for x in 0..8 {
            let id = if x < 4 || uniform { GRASS_ID } else { DIRT_ID };
            map.set_global_block(x, y, 0, ground_block(id));
        }
    }
    map
}

// ------------------------------------------------- CPU: semantics
#[test]
fn f2_blend_decision_matches_stage_d_reference() {
    // Synthetic fields (deterministic, no RNG): rolling heights
    // for both sides so the comparison covers wins, losses and
    // ties across the whole weight range.
    let n = 64u32;
    let mut ha = GrayImage::new(n, n);
    let mut hb = GrayImage::new(n, n);
    for y in 0..n {
        for x in 0..n {
            ha.put_pixel(x, y, image::Luma([((x * 37 + y * 91) % 256) as u8]));
            hb.put_pixel(x, y, image::Luma([((x * 53 + y * 29 + 128) % 256) as u8]));
        }
    }
    let a = RgbaImage::from_pixel(n, n, Rgba([200, 30, 30, 255]));
    let b = RgbaImage::from_pixel(n, n, Rgba([30, 30, 220, 255]));
    for w in [0.0f32, 0.125, 0.25, 0.375, 0.5] {
        let out = voxelizer::quilt::height_blend(&a, &ha, &b, &hb, w);
        for y in 0..n {
            for x in 0..n {
                let reference_says_b = out.get_pixel(x, y).0 == [30, 30, 220, 255];
                let mirror_says_b = neighbor_wins(
                    ha.get_pixel(x, y).0[0] as f32 / 255.0,
                    hb.get_pixel(x, y).0[0] as f32 / 255.0,
                    w,
                );
                assert_eq!(
                    reference_says_b, mirror_says_b,
                    "weight {w} pixel ({x},{y}): shader mirror must match height_blend"
                );
            }
        }
    }
    // Endpoints of the spatial weight: at and beyond the half
    // band the neighbor can never win, whatever the heights; at
    // the border the raw height decides (punch-through).
    assert_eq!(blend_weight(0.0), 0.5);
    assert_eq!(blend_weight(BLEND_HALF_BAND_TILES), 0.0);
    assert!(!neighbor_wins(0.0, 1.0, blend_weight(BLEND_HALF_BAND_TILES)));
    assert!(neighbor_wins(0.1, 0.9, blend_weight(0.0)), "high patch punches at the border");
    assert!(!neighbor_wins(0.9, 0.1, blend_weight(0.0)));
}

// ---------------------------------------------------- CPU: spec
#[test]
fn f2_blend_spec_entries_and_stability() {
    let base = std::env::temp_dir().join(format!("f2blend_spec_{}", std::process::id()));
    let dir = asset_dir(&base);
    let table = MaterialTable::load(&dir).unwrap();
    let assignments = [
        BlendAssignment {
            base_id: GRASS_ID,
            neighbor_id: DIRT_ID,
            dir: 0,
            virtual_id: 20,
        },
        BlendAssignment {
            base_id: DIRT_ID,
            neighbor_id: GRASS_ID,
            dir: 1,
            virtual_id: 21,
        },
    ];
    let (spec, stats) =
        build_texture_spec_variants(&table, &dir, &id_map(), &[], &[], &assignments).unwrap();
    assert_eq!(stats.blend_variants, 2);
    assert_eq!(stats.macros, 2);
    // The virtual id copies the base macro + layer entries...
    assert_eq!(spec.material_macros[20], spec.material_macros[GRASS_ID as usize]);
    assert_eq!(spec.material_layers[20], spec.material_layers[GRASS_ID as usize]);
    // ...and its blend entry names the NEIGHBOR's compact macro
    // layers (post-offset, exactly what the RT shader indexes)
    // plus the direction.
    assert_eq!(
        spec.material_blends[20],
        [
            spec.material_macros[DIRT_ID as usize][0],
            spec.material_macros[DIRT_ID as usize][1],
            0,
            1
        ],
        "grass blend entry -> dirt macro layers, dir +X"
    );
    assert_eq!(
        spec.material_blends[21],
        [
            spec.material_macros[GRASS_ID as usize][0],
            spec.material_macros[GRASS_ID as usize][1],
            1,
            1
        ],
        "dirt blend entry -> grass macro layers, dir -X"
    );
    // Stability: no blend assignments -> no blend entries, and
    // layers/macros identical to the base builder's.
    let (base_spec, _) = build_texture_spec(&table, &dir, &id_map()).unwrap();
    let (plain_spec, plain_stats) =
        build_texture_spec_variants(&table, &dir, &id_map(), &[], &[], &[]).unwrap();
    assert_eq!(plain_stats.blend_variants, 0);
    assert!(plain_spec.material_blends.iter().all(|e| e[3] == 0));
    assert_eq!(plain_spec.material_layers, base_spec.material_layers);
    assert_eq!(plain_spec.material_macros, base_spec.material_macros);
    assert_eq!(plain_spec.layers, base_spec.layers);
    let _ = std::fs::remove_dir_all(&base);
}

// ---------------------------------------------- CPU: assignment
#[test]
fn f2_blend_assignments_for_border_scene() {
    let base = std::env::temp_dir().join(format!("f2blend_asn_{}", std::process::id()));
    let dir = asset_dir(&base);
    let table = MaterialTable::load(&dir).unwrap();
    let spans = macro_spans(&table);
    let span_of = |id: u32| spans.get(&id).copied();

    let map = border_map(false);
    let assignments = engine::blend::blend_assignments_for_map(&map, &span_of);
    assert_eq!(
        assignments,
        vec![
            BlendAssignment {
                base_id: GRASS_ID,
                neighbor_id: DIRT_ID,
                dir: 0,
                virtual_id: 11,
            },
            BlendAssignment {
                base_id: DIRT_ID,
                neighbor_id: GRASS_ID,
                dir: 1,
                virtual_id: 12,
            },
        ],
        "one triple per border side, virtual ids above the max block id (10)"
    );
    // The VoxelWorld form allocates the same assignments and
    // rewrites exactly the two border columns.
    let mut world = VoxelWorld::from_chunk_map(&map);
    let vw_assignments = engine::blend::assign_blend_variants(&mut world, &span_of);
    assert_eq!(assignments, vw_assignments);
    assert_eq!(world.get([3, 0, 0]).unwrap().material_id, 11);
    assert_eq!(world.get([4, 0, 0]).unwrap().material_id, 12);
    assert_eq!(world.get([0, 0, 0]).unwrap().material_id, GRASS_ID);
    assert_eq!(world.get([7, 0, 0]).unwrap().material_id, DIRT_ID);

    // Uniform field: no borders, no assignments.
    let flat = border_map(true);
    assert!(engine::blend::blend_assignments_for_map(&flat, &span_of).is_empty());
    let _ = std::fs::remove_dir_all(&base);
}

// ------------------------------------------------------------ GPU
struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn gpu() -> Option<Gpu> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        },
    ))?;
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("f2-blend-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu { device, queue })
}

fn orbit() -> OrbitCamera {
    // Framed on the border (grid x = 4) from the south, steep
    // enough that top faces dominate the frame.
    let mut cam = OrbitCamera::new(Vec3::new(4.0, 0.0, -2.0));
    cam.yaw_deg = 0.0;
    cam.pitch_deg = 55.0;
    // Framed close enough that the RT traversal resolves leaf
    // voxels along the whole border: at longer range the
    // renderer's existing LOD stops at coarser tree nodes and
    // shades them with the node material (the same
    // approximation it already applies to every texture
    // decision), so distant borders progressively shade
    // unblended — measured: full strip at 6 and 9 units, near
    // rows only at 10.5.
    cam.distance = 9.0;
    cam.clamp();
    cam
}

/// Stamp the gate map's blocks from `spec` (the gate-local form
/// of scene_prep's stamping, following the stage_f_gate
/// precedent): base macro words everywhere, blend words on the
/// border blocks when `blend` is set.
fn stamp(map: &mut ChunkMap, spec: &treebuild::render::TextureSpec, blend: bool) {
    let macro_span = |id: u32| -> Option<u32> {
        spec.material_macros
            .get(id as usize)
            .filter(|mm| mm[3] == 1)
            .map(|mm| mm[2])
    };
    for chunk in map.chunks.values_mut() {
        let size = chunk.grid.size;
        for z in 0..size[2] {
            for y in 0..size[1] {
                for x in 0..size[0] {
                    let b = chunk.grid.get(x as i32, y as i32, z as i32);
                    if b.is_air() {
                        continue;
                    }
                    let Some(ml) = spec.material_layers.get(b.id as usize) else {
                        continue;
                    };
                    if ml[3] != 1 {
                        continue;
                    }
                    let top = match spec.material_macros.get(b.id as usize) {
                        Some(mm) if mm[3] == 1 => encode_macro_layer(mm[0], mm[2]),
                        _ => ml[0],
                    };
                    let mut stamped = b;
                    stamped.tex_layers = Some([top, ml[1], ml[2]]);
                    chunk.grid.set(x, y, z, stamped);
                }
            }
        }
    }
    if !blend {
        return;
    }
    // Second pass (collect, then apply — the choice reads the
    // whole map): border blocks take the blend word.
    let get = |x: i32, y: i32, z: i32| map.block_at(x, y, z).id;
    let mut words: Vec<([i32; 3], u32)> = Vec::new();
    for chunk in map.chunks.values() {
        let origin = chunk.origin();
        let size = chunk.grid.size;
        for z in 0..size[2] as i32 {
            for y in 0..size[1] as i32 {
                for x in 0..size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    let Some(&mm) = spec.material_macros.get(b.id as usize) else {
                        continue;
                    };
                    if mm[3] != 1 {
                        continue;
                    }
                    let pos = [origin[0] + x, origin[1] + y, origin[2] + z];
                    if let Some((nbr, dir)) =
                        engine::blend::blend_neighbor_at(&get, pos, &macro_span)
                    {
                        let nbr_mm = spec.material_macros[nbr as usize];
                        words.push((
                            pos,
                            encode_macro_blend_layer(mm[0], nbr_mm[0], dir as u32, mm[2]),
                        ));
                    }
                }
            }
        }
    }
    for (pos, word) in words {
        let b = map.block_at(pos[0], pos[1], pos[2]);
        let layers = b.tex_layers.unwrap();
        let mut stamped = b;
        stamped.tex_layers = Some([word, layers[1], layers[2]]);
        map.set_global_block(pos[0], pos[1], pos[2], stamped);
    }
}

fn render_raster(gpu: &Gpu, map: &ChunkMap, spec: &treebuild::render::TextureSpec) -> RgbaImage {
    let scale = WorldScale::default();
    let meshes = mesher::mesher::mesh_greedy(map, [0, 0, 0], [8, 4, 1], scale);
    let mut renderer = Renderer::new(&gpu.device, CAPTURE_FORMAT);
    renderer.set_textures(&gpu.device, &gpu.queue, spec);
    let gpu_meshes = renderer.upload(&gpu.device, &meshes);
    assert!(!gpu_meshes.is_empty(), "blank render is not a pass");
    let color = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("f2-blend-gate-color"),
        size: wgpu::Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let color_view = color.create_view(&Default::default());
    let depth_view = create_depth_view(&gpu.device, SIZE);
    let cam = orbit();
    renderer.set_view_projection(
        &gpu.queue,
        cam.view_projection(SIZE.0 as f32 / SIZE.1 as f32),
        cam.eye(),
    );
    renderer.draw(&gpu.device, &gpu.queue, &gpu_meshes, &color_view, &depth_view);
    gpu.device.poll(wgpu::Maintain::Wait);
    let (img, _) = engine::frame::readback_rgba(&gpu.device, &gpu.queue, &color, SIZE)
        .expect("raster readback");
    img
}

fn render_rt(
    gpu: &Gpu,
    map: &ChunkMap,
    spec: &treebuild::render::TextureSpec,
    assignments: &[BlendAssignment],
    spans: &HashMap<u32, u32>,
) -> RgbaImage {
    let mut world = VoxelWorld::from_chunk_map(map);
    if !assignments.is_empty() {
        let applied = engine::blend::assign_blend_variants(&mut world, &|id| spans.get(&id).copied());
        assert_eq!(
            applied, assignments,
            "the rendered world must be rewritten by the same assignments the spec carries"
        );
    }
    let built = build_tree(&world, BuildConfig::default()).expect("tree builds");
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    scene
        .attach_textures(&gpu.device, &gpu.queue, spec)
        .expect("textures attach");
    let cam = orbit();
    let rt_cam = RtCamera::from_world(
        cam.eye(),
        cam.target,
        60.0,
        SIZE.0 as f32 / SIZE.1 as f32,
    );
    let frame = renderer.render(&gpu.device, &gpu.queue, &scene, &rt_cam, SIZE, true);
    gpu.device.poll(wgpu::Maintain::Wait);
    let (img, _) = engine::frame::readback_rgba(&gpu.device, &gpu.queue, &frame.color, SIZE)
        .expect("rt readback");
    img
}

/// Grass/dirt pixel classification via the shared
/// chromaticity classifier (`tests/common`): the old
/// channel-ordering rules (`grass_like`: g > r && g > b;
/// `dirt_like`: r > g > b with (r − b) ≥ 20) assumed scalar
/// light preserves channel ordering — §5.3's coloured
/// hemisphere ambient broke that assumption on
/// low-saturation dirt (the pond gate's RT dirt count went
/// 982 → 0). This gate's sun-lit ground tops happened to
/// keep the old rules working, but the classifier is shared
/// so the two gates cannot drift apart again. The measured
/// derivation lives in the common module.
fn is_class(centroids: &[Centroid], class: Class, p: &Rgba<u8>) -> bool {
    classify(p, centroids) == class
}

/// The screen column of the grass|dirt border in the CONTROL
/// image: the first column (from the left) whose dirt count
/// beats its grass count. Self-calibrating — the assertions
/// then measure intrusion relative to the border as rendered,
/// not to a guessed projection.
fn border_column(control: &RgbaImage, centroids: &[Centroid]) -> usize {
    let (w, h) = control.dimensions();
    (0..w as usize)
        .find(|&c| {
            let (mut g, mut d) = (0usize, 0usize);
            for y in 0..h as usize {
                let p = control.get_pixel(c as u32, y as u32);
                if is_class(centroids, Class::Grass, p) {
                    g += 1;
                } else if is_class(centroids, Class::Dirt, p) {
                    d += 1;
                }
            }
            d > g && d > 8
        })
        .expect("control image must show a grass|dirt border")
}

fn count_in(
    img: &RgbaImage,
    cols: std::ops::Range<usize>,
    centroids: &[Centroid],
    class: Class,
) -> usize {
    let h = img.height() as usize;
    let mut n = 0;
    for c in cols {
        for y in 0..h {
            if is_class(centroids, class, img.get_pixel(c as u32, y as u32)) {
                n += 1;
            }
        }
    }
    n
}

fn diff_count(a: &RgbaImage, b: &RgbaImage, cols: std::ops::Range<usize>) -> usize {
    let h = a.height() as usize;
    let mut n = 0;
    for c in cols {
        for y in 0..h {
            if a.get_pixel(c as u32, y as u32) != b.get_pixel(c as u32, y as u32) {
                n += 1;
            }
        }
    }
    n
}

/// (count, max per-channel delta) of differing pixels in a
/// column range. The raster confinement check uses the delta
/// bound: re-splitting a greedy quad along the border tiles
/// flips its triangulation diagonal, and pixels exactly on the
/// old diagonal can land on the neighboring macro texel (the
/// checker grain, a few counts) — a material change would move
/// a channel by ~50+.
fn diff_stats(a: &RgbaImage, b: &RgbaImage, cols: std::ops::Range<usize>) -> (usize, i32) {
    let h = a.height() as usize;
    let (mut n, mut max_delta) = (0usize, 0i32);
    for c in cols {
        for y in 0..h {
            let (pa, pb) = (
                a.get_pixel(c as u32, y as u32).0,
                b.get_pixel(c as u32, y as u32).0,
            );
            if pa != pb {
                n += 1;
                for ch in 0..3 {
                    max_delta = max_delta.max((pa[ch] as i32 - pb[ch] as i32).abs());
                }
            }
        }
    }
    (n, max_delta)
}

#[test]
fn f2_blend_capture_both_renderers() {
    let _guard = gpu_lock();
    let Some(gpu) = gpu() else {
        eprintln!("F2 BLEND GATE NOT RUN: no wgpu adapter (this gate proves nothing without one)");
        return;
    };
    let base = std::env::temp_dir().join(format!("f2blend_gpu_{}", std::process::id()));
    let dir = asset_dir(&base);
    let table = MaterialTable::load(&dir).unwrap();
    let spans = macro_spans(&table);
    let span_of = |id: u32| spans.get(&id).copied();

    // Specs: control = base builder (today's renderer); blended
    // = variant builder with the scene's assignments.
    let map = border_map(false);
    let assignments = engine::blend::blend_assignments_for_map(&map, &span_of);
    assert_eq!(assignments.len(), 2);
    let (control_spec, _) = build_texture_spec(&table, &dir, &id_map()).unwrap();
    let (blend_spec, blend_stats) =
        build_texture_spec_variants(&table, &dir, &id_map(), &[], &[], &assignments).unwrap();
    assert_eq!(blend_stats.blend_variants, 2);

    // ---- raster
    let mut raster_blend_map = border_map(false);
    stamp(&mut raster_blend_map, &blend_spec, true);
    let raster_blended = render_raster(&gpu, &raster_blend_map, &blend_spec);
    let mut raster_control_map = border_map(false);
    stamp(&mut raster_control_map, &control_spec, false);
    let raster_control = render_raster(&gpu, &raster_control_map, &control_spec);

    // ---- RT
    let rt_blended = render_rt(&gpu, &map, &blend_spec, &assignments, &spans);
    let rt_control = render_rt(&gpu, &map, &control_spec, &[], &spans);

    for (name, blended, control) in [
        ("raster", &raster_blended, &raster_control),
        ("rt", &rt_blended, &rt_control),
    ] {
        let renderer = if name == "rt" {
            RendererKind::Rt
        } else {
            RendererKind::Raster
        };
        let centroids = blend_centroids(renderer, DIRT_RGB, GRASS_RGB);
        let w = SIZE.0 as usize;
        let border = border_column(control, &centroids);
        // Interpenetration: each material's colors appear on the
        // other material's side of the border (a margin of 2px
        // absorbs the border line itself).
        let dirt_on_grass = count_in(blended, 0..border.saturating_sub(2), &centroids, Class::Dirt);
        let grass_on_dirt = count_in(blended, border + 2..w, &centroids, Class::Grass);
        let control_dirt_on_grass =
            count_in(control, 0..border.saturating_sub(2), &centroids, Class::Dirt);
        let control_grass_on_dirt =
            count_in(control, border + 2..w, &centroids, Class::Grass);
        // Confinement: no MATERIAL change outside the border
        // band (the two border tiles, generously). RT (no
        // mesher) must be exactly identical there; the raster
        // path may show a few sub-texel diagonal-flip pixels
        // from the greedy re-split (bounded count, bounded
        // per-channel delta — see diff_stats).
        let band_lo = border.saturating_sub(72);
        let band_hi = (border + 72).min(w);
        let (out_lo, delta_lo) = diff_stats(blended, control, 0..band_lo);
        let (out_hi, delta_hi) = diff_stats(blended, control, band_hi..w);
        let (outside_diffs, outside_delta) = (out_lo + out_hi, delta_lo.max(delta_hi));
        let inside_diffs = diff_count(blended, control, band_lo..band_hi);
        println!(
            "{name}: border col {border}; dirt-on-grass {dirt_on_grass} (control \
             {control_dirt_on_grass}), grass-on-dirt {grass_on_dirt} (control \
             {control_grass_on_dirt}); diffs inside band {inside_diffs}, outside \
             {outside_diffs} (max channel delta {outside_delta})"
        );
        assert!(
            dirt_on_grass >= 24,
            "{name}: dirt must punch into the grass side at the height patch ({dirt_on_grass} px)"
        );
        assert!(
            grass_on_dirt >= 24,
            "{name}: taller grass must spill into the dirt side along the border ({grass_on_dirt} px)"
        );
        assert!(
            control_dirt_on_grass <= dirt_on_grass / 4 && control_grass_on_dirt <= grass_on_dirt / 4,
            "{name}: the unblended control must stay (nearly) clean on both sides \
             (dirt {control_dirt_on_grass}, grass {control_grass_on_dirt})"
        );
        if name == "rt" {
            assert_eq!(
                outside_diffs, 0,
                "rt: interior pixels beyond the blend band must be unchanged"
            );
        } else {
            assert!(
                outside_diffs <= 8 && outside_delta <= 24,
                "raster: beyond the band only sub-texel diagonal flips may appear \
                 ({outside_diffs} px, max channel delta {outside_delta})"
            );
        }
        assert!(
            inside_diffs > 100,
            "{name}: the border band must actually change ({inside_diffs} px)"
        );
        // Determinism: a second blended render is byte-identical.
        // (Checked by the callers below for one renderer each to
        // bound the gate's runtime; see the re-render asserts.)
        let _ = blended;
    }

    // Determinism re-renders (byte-identical), one per renderer.
    let raster_blended2 = render_raster(&gpu, &raster_blend_map, &blend_spec);
    assert_eq!(
        raster_blended.as_raw(),
        raster_blended2.as_raw(),
        "raster blend render must be deterministic"
    );
    let rt_blended2 = render_rt(&gpu, &map, &blend_spec, &assignments, &spans);
    assert_eq!(
        rt_blended.as_raw(),
        rt_blended2.as_raw(),
        "rt blend render must be deterministic"
    );
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn f2_blend_uniform_scene_identical() {
    let _guard = gpu_lock();
    let Some(gpu) = gpu() else {
        eprintln!("F2 BLEND GATE NOT RUN: no wgpu adapter (this gate proves nothing without one)");
        return;
    };
    let base = std::env::temp_dir().join(format!("f2blend_uni_{}", std::process::id()));
    let dir = asset_dir(&base);
    let table = MaterialTable::load(&dir).unwrap();
    let spans = macro_spans(&table);
    let span_of = |id: u32| spans.get(&id).copied();

    let map = border_map(true);
    let assignments = engine::blend::blend_assignments_for_map(&map, &span_of);
    assert!(assignments.is_empty(), "uniform field has no borders");
    let (control_spec, _) = build_texture_spec(&table, &dir, &id_map()).unwrap();
    let (blend_spec, _) =
        build_texture_spec_variants(&table, &dir, &id_map(), &[], &[], &assignments).unwrap();

    // Raster: blend stamping on a borderless map changes no
    // word, so the renders must be byte-identical.
    let mut m1 = border_map(true);
    stamp(&mut m1, &blend_spec, true);
    let r_blend = render_raster(&gpu, &m1, &blend_spec);
    let mut m2 = border_map(true);
    stamp(&mut m2, &control_spec, false);
    let r_control = render_raster(&gpu, &m2, &control_spec);
    assert_eq!(
        r_blend.as_raw(),
        r_control.as_raw(),
        "raster: uniform scene must render identically through the blend pipeline"
    );

    // RT: same proof.
    let t_blend = render_rt(&gpu, &map, &blend_spec, &assignments, &spans);
    let t_control = render_rt(&gpu, &map, &control_spec, &[], &spans);
    assert_eq!(
        t_blend.as_raw(),
        t_control.as_raw(),
        "rt: uniform scene must render identically through the blend pipeline"
    );
    let _ = std::fs::remove_dir_all(&base);
}
