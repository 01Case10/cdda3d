//! Phase 2d engine gates (plan Part II §5d):
//! 2d.2 — layer selection: for a multitile material with known
//! `subtile_layers`, every one of the 16 masks resolves to the
//! voxelizer's layer for that mask; non-multitile materials and
//! missing subtiles fall back to the base side layer. Checked both
//! directly (`Material::side_layer_for_mask`) and through the
//! scene spec builder, including against the real UltiCa output.
//! 2d.3 — render evidence (adapter-conditional): wall layouts
//! rendered through the RT path sample their variant layers — the
//! center pixel of a face-on render is the variant's color, the
//! base-layer render of the same scene is the base color, geometry
//! (depth) is identical, sky is identical, and the no-textures
//! render is unchanged by variant assignment.
//! 2d.1 (masks) lives in treebuild's multitile gate; 2d.4 is the
//! workspace suite + the 2c.4 parity test in rt_gate.rs.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use engine::materials::{Material, MaterialTable};
use engine::textures::{build_texture_spec, build_texture_spec_multitile};
use treebuild::connect::{assign_variants, same_material, VariantAssignment};
use treebuild::world::{VoxelCell, VoxelWorld};

// ------------------------------------------------------------- 2d.2

fn material(multitile: bool, subtile_layers: &[(&str, usize)]) -> Material {
    Material {
        id: "t_test".into(),
        resolved_id: "t_test".into(),
        resolution: "exact".into(),
        top_layer: 0,
        side_layer: 7,
        bottom_layer: 1,
        height_voxels: 3,
        transparency: "opaque".into(),
        cutout: false,
        transparent: false,
        multitile,
        subtile_layers: subtile_layers
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect::<BTreeMap<_, _>>(),
        macro_: None,
    }
}

#[test]
fn gate_2d_2_layer_selection_all_16_masks() {
    let mat = material(
        true,
        &[
            ("center", 100),
            ("corner", 101),
            ("edge", 102),
            ("end_piece", 103),
            ("t_connection", 104),
            ("unconnected", 105),
        ],
    );
    // Hand table (the game's rule, same as gate 5.5 / mesher's
    // connect gate): mask -> expected global side layer.
    let expected: [(u8, usize); 16] = [
        (0, 105),
        (1, 103),
        (2, 103),
        (3, 101),
        (4, 103),
        (5, 102),
        (6, 101),
        (7, 104),
        (8, 103),
        (9, 101),
        (10, 102),
        (11, 104),
        (12, 101),
        (13, 104),
        (14, 104),
        (15, 100),
    ];
    for (mask, want) in expected {
        assert_eq!(mat.side_layer_for_mask(mask), want, "mask {mask:04b}");
    }
    // Non-multitile materials resolve to base layers for every mask.
    let plain = material(false, &[("edge", 102)]);
    for mask in 0..16u8 {
        assert_eq!(plain.side_layer_for_mask(mask), 7, "mask {mask}");
    }
    // A multitile material missing a subtile falls back per-mask:
    // only the subtiles it carries vary.
    let partial = material(true, &[("edge", 102)]);
    assert_eq!(partial.side_layer_for_mask(10), 102, "edge present");
    assert_eq!(partial.side_layer_for_mask(15), 7, "center absent -> base");
    assert_eq!(
        partial.side_layer_for_mask(0),
        7,
        "unconnected absent -> base"
    );
}

#[test]
fn gate_2d_2_real_ultica_wall_variants_differ_from_base() {
    let tex_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1");
    if !tex_dir.join("materials.json").exists() {
        eprintln!("2d.2: {} not present — NOT RUN", tex_dir.display());
        return;
    }
    let table = MaterialTable::load(&tex_dir).unwrap();
    let wall = table.get("t_wall").expect("t_wall in UltiCa output");
    assert!(wall.multitile, "t_wall is multitile in the real data");
    assert_eq!(wall.subtile_layers.len(), 6, "all six neighbour subtiles");
    // One assignment per subtile class.
    let masks = [0u8, 3, 5, 7, 10, 15];
    let assignments: Vec<VariantAssignment> = masks
        .iter()
        .enumerate()
        .map(|(i, &mask)| VariantAssignment {
            base_id: 2,
            mask,
            virtual_id: 100 + i as u32,
        })
        .collect();
    let id_map: HashMap<u32, String> = [(2, "t_wall".to_string())].into_iter().collect();
    let (spec, stats) =
        build_texture_spec_multitile(&table, &tex_dir, &id_map, &assignments).unwrap();
    assert_eq!(stats.variants, 6);
    let base = spec.material_layers[2];
    assert_eq!(base[3], 1, "base entry present");
    // The criterion is index fidelity: the variant's side layer in
    // the compact spec must be byte-identical to the voxelizer's
    // own layer file for the mask's subtile. (For t_wall the
    // unconnected subtile's picture happens to be the same sprite
    // the base side was derived from, so "different bytes than
    // base" is NOT a valid expectation on real data — different
    // *layer index*, same pixels, is a correct selection there.)
    let layer_bytes = |global: usize| -> Vec<u8> {
        image::open(
            tex_dir
                .join("blocks")
                .join(format!("layer_{global:04}.png")),
        )
        .unwrap()
        .to_rgba8()
        .into_raw()
    };
    let mut distinct_from_base = 0usize;
    for a in &assignments {
        let v = spec.material_layers[a.virtual_id as usize];
        assert_eq!(v[3], 1, "variant {} present", a.virtual_id);
        assert_eq!(v[0], base[0], "top layer never varies (mask {})", a.mask);
        assert_eq!(v[2], base[2], "bottom layer never varies (mask {})", a.mask);
        let want_global = wall.side_layer_for_mask(a.mask);
        assert_eq!(
            spec.layers[v[1] as usize],
            layer_bytes(want_global),
            "mask {} must select the voxelizer's layer {want_global} for its subtile",
            a.mask
        );
        if want_global != wall.side_layer {
            distinct_from_base += 1;
        }
    }
    assert!(
        distinct_from_base >= 5,
        "t_wall's subtile layers are distinct global layers (got {distinct_from_base})"
    );
}

// ------------------------------------------------------------- 2d.3

const TILE: u32 = 8;
/// Fixture layer colors (solid fills): base side is red; each
/// subtile is a distinct hue so a face-on render's center pixel
/// identifies the sampled layer by its dominant channel.
const COL_TOP: [u8; 4] = [90, 90, 90, 255];
const COL_BASE_SIDE: [u8; 4] = [220, 30, 30, 255];
const COL_BOTTOM: [u8; 4] = [40, 40, 40, 255];
const COL_CENTER: [u8; 4] = [230, 210, 30, 255];
const COL_CORNER: [u8; 4] = [210, 30, 200, 255];
const COL_EDGE: [u8; 4] = [30, 200, 30, 255];
const COL_END: [u8; 4] = [30, 190, 200, 255];
const COL_TEE: [u8; 4] = [235, 235, 235, 255];
const COL_UNCONN: [u8; 4] = [30, 30, 220, 255];

fn write_fixture(dir: &Path) {
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    std::fs::write(
        dir.join("blocks.json"),
        format!(r#"{{"format":"png-layers","tile_size":{TILE},"layer_count":9}}"#),
    )
    .unwrap();
    std::fs::write(
        dir.join("materials.json"),
        r#"[{"id":"t_wall_mt","resolved_id":"t_wall_mt","resolution":"exact","material_index":0,"top_layer":0,"side_layer":1,"bottom_layer":2,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":true,"subtile_layers":{"center":3,"corner":4,"edge":5,"end_piece":6,"t_connection":7,"unconnected":8}}]"#,
    )
    .unwrap();
    let colors = [
        COL_TOP,
        COL_BASE_SIDE,
        COL_BOTTOM,
        COL_CENTER,
        COL_CORNER,
        COL_EDGE,
        COL_END,
        COL_TEE,
        COL_UNCONN,
    ];
    for (i, c) in colors.iter().enumerate() {
        image::RgbaImage::from_pixel(TILE, TILE, image::Rgba(*c))
            .save(blocks.join(format!("layer_{i:04}.png")))
            .unwrap();
    }
}

fn fixture_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("engine_mt_gate_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_fixture(&dir);
    dir
}

const WALL_ID: u32 = 7;

fn wall_world(pts: &[[i32; 3]]) -> VoxelWorld {
    let mut w = VoxelWorld::new();
    for &p in pts {
        w.set(p, VoxelCell::new([0.7, 0.68, 0.62], WALL_ID));
    }
    w
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn gpu() -> Option<Gpu> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("mt-gate-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .ok()?;
    Some(Gpu { device, queue })
}

/// Render `world` face-on at 128x128 from `eye` toward `target`
/// and read back (pixels, depth, max_dist). `spec` = None renders
/// flat (dummy textures), exactly the no-textures path.
fn render_face_on(
    gpu: &Gpu,
    world: &VoxelWorld,
    spec: Option<&treebuild::render::TextureSpec>,
    eye: (f32, f32, f32),
    target: (f32, f32, f32),
) -> (Vec<u8>, Vec<f32>, f32) {
    use treebuild::build::{build_tree, BuildConfig};
    use treebuild::render::{RtCamera, RtRenderer};
    let built = build_tree(world, BuildConfig::default()).unwrap();
    let (renderer, mut scene) = RtRenderer::with_scene(&gpu.device, &gpu.queue, &built);
    if let Some(spec) = spec {
        scene
            .attach_textures(&gpu.device, &gpu.queue, spec)
            .unwrap();
    }
    let camera = RtCamera {
        eye: glam::Vec3::new(eye.0, eye.1, eye.2),
        target: glam::Vec3::new(target.0, target.1, target.2),
        fov_y_deg: 60.0,
        aspect: 1.0,
    };
    let frame = renderer.render(&gpu.device, &gpu.queue, &scene, &camera, (128, 128), false);
    let pixels = read_rgba(&gpu, &frame.color, frame.size);
    let depth = read_f32(&gpu, &frame.depth, 128 * 128);
    (pixels, depth, frame.max_dist)
}

fn read_rgba(gpu: &Gpu, texture: &wgpu::Texture, size: (u32, u32)) -> Vec<u8> {
    let (w, h) = size;
    let unpadded = w * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mt-gate-readback"),
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..h as usize {
            let start = row * padded as usize;
            out.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();
    out
}

fn read_f32(gpu: &Gpu, buffer: &wgpu::Buffer, count: usize) -> Vec<f32> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mt-gate-depth"),
        size: (count * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (count * 4) as u64);
    gpu.queue.submit([encoder.finish()]);
    gpu.device.poll(wgpu::Maintain::Wait);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let data = slice.get_mapped_range();
    let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();
    out
}

fn center_px(pixels: &[u8]) -> [u8; 4] {
    let i = (64 * 128 + 64) * 4;
    [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
}

#[test]
fn gate_2d_3_wall_layouts_sample_their_variant_layers() {
    let Some(gpu) = gpu() else {
        eprintln!("2d.3: no wgpu adapter — GPU checks NOT RUN (data halves pass)");
        return;
    };
    let dir = fixture_dir();
    let table = MaterialTable::load(&dir).unwrap();
    let id_map: HashMap<u32, String> = [(WALL_ID, "t_wall_mt".to_string())].into_iter().collect();
    let (base_spec, base_stats) = build_texture_spec(&table, &dir, &id_map).unwrap();
    assert_eq!(base_stats.variants, 0);

    // Scene -> the subtile the CENTER PIXEL's face must sample,
    // named by the dominant-channel signature of the fixture
    // color. The center ray meets the first wall face along the
    // view direction, which is not always the layout's middle
    // voxel: in the cross, the north arm (mask S only) occludes
    // the mask-15 center voxel — a fully surrounded voxel's side
    // faces are interior and can never be rendered from outside,
    // so the center subtile is pinned at selection level by
    // gates 2d.1/2d.2 and the cross asserts the arm's end_piece.
    // The tee is viewed from its open (south) side, where the
    // junction voxel's own face IS the first hit.
    const NORTH_CAM: ((f32, f32, f32), (f32, f32, f32)) =
        ((8.5, 1.5, 4.5), (8.5, 8.5, 4.5));
    const SOUTH_CAM: ((f32, f32, f32), (f32, f32, f32)) =
        ((8.5, 15.5, 4.5), (8.5, 8.5, 4.5));
    let scenes: [(&str, Vec<[i32; 3]>, &str, ((f32, f32, f32), (f32, f32, f32))); 4] = [
        ("isolated", vec![[8, 8, 4]], "blue", NORTH_CAM), // mask 0 -> unconnected
        (
            "ew-run",
            vec![[7, 8, 4], [8, 8, 4], [9, 8, 4]],
            "green", // middle voxel mask E+W -> edge
            NORTH_CAM,
        ),
        (
            "cross",
            vec![[8, 8, 4], [7, 8, 4], [9, 8, 4], [8, 7, 4], [8, 9, 4]],
            "cyan", // north arm mask S -> end_piece (center occluded)
            NORTH_CAM,
        ),
        (
            "tee",
            vec![[8, 8, 4], [8, 7, 4], [7, 8, 4], [9, 8, 4]],
            "white", // junction mask N+E+W -> t_connection
            SOUTH_CAM,
        ),
    ];
    for (name, pts, want, (eye, target)) in &scenes {
        let mut world = wall_world(pts);
        let assignments = assign_variants(&mut world, |id| id == WALL_ID, same_material);
        assert!(!assignments.is_empty(), "{name}: variants assigned");
        let (spec, stats) =
            build_texture_spec_multitile(&table, &dir, &id_map, &assignments).unwrap();
        assert_eq!(stats.variants, assignments.len(), "{name}");

        let (px, depth, max_dist) = render_face_on(&gpu, &world, Some(&spec), *eye, *target);
        assert!(
            depth[64 * 128 + 64] < max_dist,
            "{name}: center ray must hit the wall"
        );
        let c = center_px(&px);
        let (r, g, b) = (c[0] as i32, c[1] as i32, c[2] as i32);
        match *want {
            "blue" => assert!(
                b > r + 40 && b > g + 40,
                "{name}: center pixel {c:?} must be the blue unconnected variant"
            ),
            "green" => assert!(
                g > r + 40 && g > b + 40,
                "{name}: center pixel {c:?} must be the green edge variant"
            ),
            "cyan" => assert!(
                g > r + 40 && b > r + 40,
                "{name}: center pixel {c:?} must be the cyan end_piece variant"
            ),
            // The tee is viewed from the south, whose face light
            // is the dimmest (~0.30 vs ~0.55 north): white reads
            // as a balanced mid-grey there. Balance + separation
            // from the red base control is the signature.
            "white" => assert!(
                r > 45 && (r - g).abs() < 20 && (g - b).abs() < 20,
                "{name}: center pixel {c:?} must be the white t_connection variant"
            ),
            _ => unreachable!(),
        }

        // Base-layer control: the same scene without assignment
        // samples the red base side at the center pixel, over
        // identical geometry.
        let base_world = wall_world(pts);
        let (bpx, bdepth, _) = render_face_on(&gpu, &base_world, Some(&base_spec), *eye, *target);
        assert_eq!(depth, bdepth, "{name}: variants must not move geometry");
        let bc = center_px(&bpx);
        assert!(
            bc[0] as i32 > bc[1] as i32 + 40 && bc[0] as i32 > bc[2] as i32 + 40,
            "{name}: base control center pixel {bc:?} must be the red base layer"
        );
        // Sky (miss) pixels are identical between the two renders;
        // the difference lives exactly on the wall faces.
        let mut sky_diff = 0usize;
        let mut geo_diff = 0usize;
        let mut geo = 0usize;
        for i in 0..depth.len() {
            if depth[i] >= max_dist {
                if px[i * 4..i * 4 + 4] != bpx[i * 4..i * 4 + 4] {
                    sky_diff += 1;
                }
            } else {
                geo += 1;
                if px[i * 4..i * 4 + 4] != bpx[i * 4..i * 4 + 4] {
                    geo_diff += 1;
                }
            }
        }
        assert_eq!(sky_diff, 0, "{name}: sky must be untouched by selection");
        assert!(
            geo > 100 && geo_diff * 2 > geo,
            "{name}: most wall pixels must change ({geo_diff}/{geo})"
        );

        // No-textures path unchanged: the variant-assigned world
        // rendered flat is pixel-identical to the base world flat.
        let (flat_v, _, _) = render_face_on(&gpu, &world, None, *eye, *target);
        let (flat_b, _, _) = render_face_on(&gpu, &base_world, None, *eye, *target);
        assert_eq!(
            flat_v, flat_b,
            "{name}: flat render must not see material ids"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
