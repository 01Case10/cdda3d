//! Stage F part 2 gates — decoration scatter ONLY (pipeline doc
//! §10.2). Border height-blending is explicitly NOT here: it is
//! held until Phase 9 items 9.4/9.5 land (Joey's decision).
//!
//! * Generation (voxelizer): three decoration sprites (tuft,
//!   shrub, flowering) derived from the fixture t_grass family's
//!   palette, byte-identical across runs, packaged in the
//!   billboard atlas format; sprite colors come from the family's
//!   analyzed palette.
//! * Placement (billboard::decor): deterministic and input-order
//!   independent; 0 decorations on floor / water / under-roof /
//!   other tiles; 0–2 per eligible tile; density measurably
//!   modulated by macro height, with the documented mid-height
//!   fallback between the low and high extremes.
//! * Hash parity: billboard's re-implemented seed hash equals
//!   voxelizer::noise::seed_from_str (the two must not drift).
//! * Not pickable: a ray that provably hits a decoration's opaque
//!   pixels (checked through billboard's own picker) returns no
//!   hit from the engine's entity picker, whose iteration set is
//!   entities only.
//! * Engine wiring (GPU, adapter-conditional like the other
//!   gates): decorations seated in a BillboardScene draw in the
//!   billboard pass and are counted in FrameStats.
//! * Perf record: the sample scene's decoration count is printed
//!   (frame-time verdict stays deferred to the RX 9070 XT).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use billboard::decor::{GroundKind, GroundTile};
use engine::capture::CaptureArgs;
use engine::frame::{self, BillboardScene};
use glam::Vec3;

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../voxelizer/tests/fixture")
}

/// The fixture tileset's pooled t_grass family (Stage C/D
/// pooling conventions).
fn grass_family() -> Vec<image::RgbaImage> {
    let ts = tileset::load_decomposed(&fixture_dir()).expect("fixture tileset loads");
    let entry = ts.entries.get("t_grass").expect("fixture has t_grass");
    voxelizer::family_sprites(&ts, entry)
}

/// The voxelizer decoration atlas, converted to the billboard
/// crate's atlas type (the two SpriteUv shapes are
/// field-for-field identical — that is the shared format).
fn decor_billboard_atlas() -> billboard::atlas::SpriteAtlas {
    let sprites = voxelizer::decoration_sprites(&grass_family());
    let (img, uvs) = voxelizer::decoration_atlas(&sprites);
    let uvs = uvs
        .into_iter()
        .map(|u| billboard::atlas::SpriteUv {
            name: u.name,
            x: u.x,
            y: u.y,
            width: u.width,
            height: u.height,
            offset_x: u.offset_x,
            offset_y: u.offset_y,
            sheet: u.sheet,
        })
        .collect();
    billboard::atlas::SpriteAtlas::from_parts(img, uvs)
}

// ---------------------------------------------------------- generation
#[test]
fn generation_deterministic_atlas_packaged_from_family_palette() {
    let family = grass_family();
    assert!(!family.is_empty(), "fixture t_grass family pools");

    let a = voxelizer::decoration_sprites(&family);
    let b = voxelizer::decoration_sprites(&family);
    assert_eq!(a.len(), 3, "tuft + shrub + flowering");
    let names: Vec<&str> = a.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, voxelizer::DECOR_NAMES);
    assert_eq!(
        names, billboard::decor::DECOR_SPRITE_NAMES,
        "generation and runtime sprite name lists must agree"
    );
    for ((na, ia), (nb, ib)) in a.iter().zip(&b) {
        assert_eq!(na, nb);
        assert_eq!(ia.as_raw(), ib.as_raw(), "{na}: byte-identical on re-run");
        assert_eq!(ia.dimensions(), (32, 32));
        let opaque = ia.pixels().filter(|p| p.0[3] == 255).count();
        let clear = ia.pixels().filter(|p| p.0[3] == 0).count();
        assert!(opaque > 40, "{na}: a real alpha shape ({opaque} opaque px)");
        assert!(clear > 200, "{na}: transparent background ({clear} clear px)");
    }

    // Derivation: every color used is one of the family's
    // analyzed palette colors (Stage B k-means over the pooled
    // family) — no invented colors.
    let refs: Vec<&image::RgbaImage> = family.iter().collect();
    let palette: Vec<[u8; 3]> = voxelizer::analyze_family(&refs, 8)
        .palette
        .iter()
        .map(|p| p.srgb)
        .collect();
    for (name, img) in &a {
        for p in img.pixels() {
            if p.0[3] > 0 {
                assert!(
                    palette.contains(&[p.0[0], p.0[1], p.0[2]]),
                    "{name}: color {:?} is not a grass-family palette color",
                    [p.0[0], p.0[1], p.0[2]]
                );
            }
        }
    }

    // Atlas packaging: one row in name order, round-trips through
    // the billboard atlas type back to the exact sprite images.
    let (img, uvs) = voxelizer::decoration_atlas(&a);
    assert_eq!(img.dimensions(), (96, 32));
    assert_eq!(
        uvs.iter().map(|u| u.x).collect::<Vec<_>>(),
        vec![0, 32, 64]
    );
    let atlas = decor_billboard_atlas();
    for (name, sprite) in &a {
        let cropped = atlas.sprite_image(name).expect("decor sprite in atlas");
        assert_eq!(cropped.as_raw(), sprite.as_raw(), "{name} atlas round-trip");
    }
}

// ---------------------------------------------------------- hash parity
#[test]
fn billboard_seed_hash_matches_voxelizer_noise() {
    for s in ["decor-scatter-v1", "t_grass", "decor_tuft", ""] {
        assert_eq!(
            billboard::decor::seed_from_str(s),
            voxelizer::noise::seed_from_str(s),
            "seed_from_str({s:?}) drifted between billboard::decor and voxelizer::noise"
        );
    }
}

// ---------------------------------------------------------- placement
fn grid(kind: GroundKind, n: i32, h: Option<f32>) -> Vec<GroundTile> {
    let mut v = Vec::new();
    for y in 0..n {
        for x in 0..n {
            v.push(GroundTile::new([x, y, 0], kind).macro_height(h));
        }
    }
    v
}

#[test]
fn scatter_deterministic_and_order_independent() {
    let mut tiles = grid(GroundKind::Grass, 24, None);
    tiles.extend(grid(GroundKind::Dirt, 8, Some(0.8)));
    let a = billboard::decor::scatter(&tiles);
    let b = billboard::decor::scatter(&tiles);
    assert_eq!(a, b, "same input -> identical decoration list");
    assert!(!a.is_empty());
    let mut reversed = tiles.clone();
    reversed.reverse();
    assert_eq!(
        a,
        billboard::decor::scatter(&reversed),
        "input order must not change the scatter"
    );
}

#[test]
fn scatter_exclusions_and_per_tile_bounds() {
    // One combined field: grass (eligible), dirt (eligible),
    // floor, water, under-roof grass, other — 20x20 each.
    let mut tiles = Vec::new();
    let mut eligible: BTreeMap<[i32; 3], ()> = BTreeMap::new();
    for (region, kind, roof) in [
        (0, GroundKind::Grass, false),
        (1, GroundKind::Dirt, false),
        (2, GroundKind::Floor, false),
        (3, GroundKind::Water, false),
        (4, GroundKind::Grass, true),
        (5, GroundKind::Other, false),
    ] {
        for y in 0..20 {
            for x in 0..20 {
                let pos = [region * 100 + x, y, 0];
                tiles.push(GroundTile::new(pos, kind).under_roof(roof));
                if matches!(kind, GroundKind::Grass | GroundKind::Dirt) && !roof {
                    eligible.insert(pos, ());
                }
            }
        }
    }
    let decors = billboard::decor::scatter(&tiles);
    assert!(!decors.is_empty(), "eligible regions decorate");
    let mut per_tile: BTreeMap<[i32; 3], usize> = BTreeMap::new();
    for d in &decors {
        assert!(
            eligible.contains_key(&d.tile),
            "decoration on excluded tile {:?}",
            d.tile
        );
        assert!((d.variant as usize) < billboard::decor::DECOR_SPRITE_NAMES.len());
        assert!(d.jitter[0].abs() <= 0.3 && d.jitter[1].abs() <= 0.3);
        *per_tile.entry(d.tile).or_insert(0) += 1;
    }
    assert!(
        per_tile.values().all(|&n| (1..=2).contains(&n)),
        "0-2 decorations per eligible tile (and >=1 wherever one was placed)"
    );
}

#[test]
fn scatter_density_follows_macro_height_with_mid_fallback() {
    let n = 60;
    let low = billboard::decor::scatter(&grid(GroundKind::Grass, n, Some(0.1))).len();
    let high = billboard::decor::scatter(&grid(GroundKind::Grass, n, Some(0.9))).len();
    let fallback = billboard::decor::scatter(&grid(GroundKind::Grass, n, None)).len();
    println!("density over {} tiles: low(h=0.1)={low} fallback(None)={fallback} high(h=0.9)={high}", n * n);
    assert!(
        high > low * 2,
        "high macro ground must decorate measurably denser ({high} vs {low})"
    );
    assert!(
        fallback > low && fallback < high,
        "fallback density sits between the extremes ({low} < {fallback} < {high})"
    );
}

// ---------------------------------------------------------- sample scene
#[test]
fn sample_scene_scatter_count_and_exclusions() {
    let tiles = frame::sample_scene_ground_tiles();
    assert_eq!(tiles.len(), 24 * 24, "sample scene is 24x24 tiles");
    let a = frame::sample_scene_decorations();
    let b = frame::sample_scene_decorations();
    assert_eq!(a, b, "sample scene scatter is deterministic");
    println!(
        "SAMPLE SCENE DECORATIONS: {} instances over {} eligible tiles",
        a.len(),
        tiles.iter().filter(|t| t.eligible()).count()
    );
    assert!(!a.is_empty(), "sample scene yard decorates");
    for d in &a {
        let (x, y) = (d.tile[0], d.tile[1]);
        assert!(
            !(3..=10).contains(&x) || !(3..=9).contains(&y),
            "no decorations in the house footprint (floor/walls/roof): {:?}",
            d.tile
        );
        assert!(
            !(16..=19).contains(&x) || !(14..=17).contains(&y),
            "no decorations on the pond: {:?}",
            d.tile
        );
        assert!(
            !(y == 13 && (2..=8).contains(&x)),
            "no decorations on the fence run: {:?}",
            d.tile
        );
        assert!(
            ![(14, 4), (19, 7), (5, 15), (21, 17), (13, 20), (18, 20)].contains(&(x, y)),
            "no decorations on tree tiles: {:?}",
            d.tile
        );
    }
}

// ---------------------------------------------------------- not pickable
#[test]
fn decorations_are_not_pickable() {
    // A decoration standing alone on a tile, with a real decor
    // atlas. First prove (through billboard's own picker) that a
    // ray through one of its opaque pixels WOULD hit it if it
    // were registered — then prove the engine picker, which
    // iterates entities only, returns nothing for the same ray.
    let atlas = decor_billboard_atlas();
    let decor = billboard::decor::Decoration {
        tile: [6, 5, 0],
        variant: 1,
        jitter: [0.0, 0.0],
        slot: 0,
    };
    let camera = engine::camera::OrbitCamera::new(Vec3::new(6.0, 1.0, -6.0));
    let eye = camera.eye();
    let insts = billboard::decor::decoration_instances(&[decor], &atlas, eye);
    assert_eq!(insts.len(), 1);
    let inst = &insts[0];
    assert_eq!(inst.entity_id, billboard::decor::DECORATION_ENTITY_ID);

    let sprite = atlas.sprite_image("decor_shrub").unwrap();
    let comp = billboard::composite::Composite {
        image: sprite,
        origin_x: 0,
        origin_y: 0,
        feet_inset_px: 0,
        signature: billboard::composite::Signature {
            sprite_ids: vec!["decor_shrub".into()],
            facing: billboard::composite::Facing::None,
        },
    };
    // Find an opaque pixel and build the ray through it.
    let (right, _) = billboard::instance::billboard_axes(inst.anchor, eye);
    let corners = billboard::instance::instance_corners(inst, eye);
    let mut hit_ray = None;
    'outer: for py in 0..comp.height() as i32 {
        for px in 0..comp.width() as i32 {
            if comp.alpha_at(px, py) > 127 {
                let u = (px as f32 + 0.5) / comp.width() as f32;
                let vfb = 1.0 - (py as f32 + 0.5) / comp.height() as f32;
                let world = corners[0] + right * (u * inst.size.x) + Vec3::Y * (vfb * inst.size.y);
                hit_ray = Some((world - eye).normalize());
                break 'outer;
            }
        }
    }
    let dir = hit_ray.expect("shrub has an opaque pixel");
    let registered = billboard::instance::pick_any(eye, dir, &[(&inst.clone(), &comp)], eye);
    assert_eq!(
        registered,
        Some(billboard::decor::DECORATION_ENTITY_ID),
        "sanity: the ray really covers the decoration (it hits when registered)"
    );

    // The engine scene: the decoration is present for drawing,
    // with NO entities. The engine picker must return nothing.
    let scene = BillboardScene {
        atlas: billboard::atlas::fixture_atlas(),
        entities: Vec::new(),
        items: None,
        time_s: 0.0,
        decorations: vec![decor],
        decor_atlas: Some(atlas),
        // Fixture world: the floor-slab lift (frame::floor_surface_lift).
        base_lift: 1.0,
    };
    let picked = frame::pick_billboard_entity(&scene, &camera, eye, dir);
    assert_eq!(picked, None, "decorations are never pickable");
}

// ---------------------------------------------------------- engine wiring (GPU)
fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("stage-f2-gate-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .ok()?;
        Some((device, queue))
    })
}

#[test]
fn engine_frame_draws_decorations_in_billboard_pass() {
    let _guard = gpu_lock();
    let Some((device, queue)) = gpu() else {
        eprintln!(
            "STAGE-F2 GATE NOT RUN: no wgpu adapter (engine wiring unproven without one)"
        );
        return;
    };
    let mut scene = BillboardScene::fixture_crowd();
    let base_count = scene.entities.len();
    scene.decorations = vec![
        billboard::decor::Decoration { tile: [1, 1, 0], variant: 0, jitter: [0.0, 0.0], slot: 0 },
        billboard::decor::Decoration { tile: [2, 2, 0], variant: 1, jitter: [0.1, -0.1], slot: 0 },
    ];
    scene.decor_atlas = Some(decor_billboard_atlas());
    let mut args = CaptureArgs {
        size: (320, 180),
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        cutaway_z: 0,
        ..CaptureArgs::default()
    };
    args.billboards = Some(scene);
    let outcome = frame::render_frame(&device, &queue, &args).expect("render_frame");
    assert_eq!(
        outcome.stats.passes,
        vec![frame::FramePass::World, frame::FramePass::Billboards],
        "decorations ride the existing billboard pass in frame order"
    );
    // entities (5) + item piles (3) + decorations (2).
    assert_eq!(
        outcome.stats.billboards,
        base_count + 3 + 2,
        "decoration instances are drawn and counted"
    );
}
