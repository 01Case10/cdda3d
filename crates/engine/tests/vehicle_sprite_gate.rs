//! Stage-G sprite-resolution gate (2026-11-24): a vehicle
//! part must never silently resolve to an unrelated item
//! icon. The bicycle's wheel existed only as
//! vp_wheel_bicycle_front/_rear (rotation-list fg); the
//! composer's chain missed them and landed on the bare
//! name `wheel_bicycle` — a 20x20 item icon from
//! small.png — whose margins made the composed wheel
//! small and floating. These assertions fail on that
//! resolution and pass on the real art.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use voxelizer::vehicle::{compose_vehicle, resolve_part_sprite, PartSpot, VehicleLayout};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ensure_assets(root: &Path) -> PathBuf {
    let assets = root.join("target/gallery_assets");
    if !assets.join("materials.json").exists() {
        let tileset_dir = root.join("fork-src/tilesets/gfx/UltimateCataclysm");
        let ts = tileset::load_decomposed(&tileset_dir).expect("tileset loads");
        let cfg = voxelizer::VoxelizerConfig {
            emit_v2: true,
            ..Default::default()
        };
        voxelizer::run(&ts, &cfg, &assets).expect("voxelizer run");
    }
    assets
}

/// The engine's resolver, mirrored: material name -> its
/// sprite layer image (sprite_layer when the build
/// emitted one, else top_layer).
fn sprite_resolver(assets: &Path) -> impl Fn(&str) -> Option<image::RgbaImage> {
    let text = std::fs::read_to_string(assets.join("materials.json")).expect("materials.json");
    let v: serde_json::Value = serde_json::from_str(&text).expect("materials parse");
    let mut layers: HashMap<String, u32> = HashMap::new();
    for m in v["materials"].as_array().expect("materials array") {
        let id = m["id"].as_str().expect("material id");
        let layer = m
            .get("sprite_layer")
            .and_then(|l| l.as_u64())
            .or_else(|| m.get("top_layer").and_then(|l| l.as_u64()))
            .expect("a layer index");
        layers.insert(id.to_string(), layer as u32);
    }
    let dir = assets.to_path_buf();
    move |id: &str| {
        layers.get(id).and_then(|layer| {
            image::open(dir.join(format!("blocks/layer_{layer:04}.png")))
                .ok()
                .map(|i| i.to_rgba8())
        })
    }
}

const NAMES: &[&str] = &["alternator_car", "alternator_truck", "basketsm", "battery_car", "cargo_space", "controls", "dashboard", "door", "door_trunk", "engine_inline4", "engine_v6", "foot_pedals", "frame", "halfboard", "headlight", "horn_bicycle", "horn_car", "integrated_heater_small", "muffler", "reclining_seat", "roof", "saddle", "seat", "seat_back", "seatbelt", "stereo", "tank", "trunk", "vehicle_alarm", "wheel", "wheel_bicycle", "wheel_mount_light", "wheel_mount_light_steerable", "wheel_mount_medium", "wheel_mount_medium_steerable", "windshield", "xlframe"];

#[test]
fn no_wheel_resolves_to_a_bare_item_icon() {
    // Scoped to wheels: they are the visible, load-bearing
    // case (the bicycle defect). Some non-wheel parts are
    // genuinely keyed bare in this tileset (frame, seat),
    // and a few under-hood parts resolve to their item
    // sprites — prototype-parity behaviour, out of scope.
    let assets = ensure_assets(&root());
    let resolver = sprite_resolver(&assets);
    for base in NAMES.iter().filter(|b| b.contains("wheel")) {
        if let Some((winner, _)) = resolve_part_sprite(base, &resolver) {
            assert!(
                winner.starts_with("vp_"),
                "wheel part {base} resolved to bare/icon name {winner}"
            );
        }
    }
}

#[test]
fn bicycle_wheel_is_the_real_grounded_disc() {
    let assets = ensure_assets(&root());
    let resolver = sprite_resolver(&assets);
    let (winner, img) = resolve_part_sprite("wheel_bicycle", &resolver)
        .expect("wheel_bicycle resolves");
    assert!(
        winner == "vp_wheel_bicycle_front" || winner == "vp_wheel_bicycle_rear",
        "wheel_bicycle resolved to {winner}"
    );
    // The real art is a grounded disc: alpha touches the
    // frame's bottom edge and stands at least 20px tall.
    let (w, h) = (img.width(), img.height());
    let (mut min_y, mut max_y) = (h, 0);
    let (mut min_x, mut max_x) = (w, 0);
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] >= 128 {
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
        }
    }
    assert_eq!(max_y, h - 1, "wheel art bottom margin (winner {winner})");
    assert!(max_y - min_y >= 20, "wheel art height {}px", max_y - min_y + 1);
    // The disc is wide; the edge-on rotation variants and
    // the item icon are a few px across.
    assert!(max_x - min_x >= 20, "wheel art width {}px", max_x - min_x + 1);

    // Composed: the front wheel's model touches the
    // ground and stands at least 40 engine rows tall.
    let mut layout = VehicleLayout::default();
    let spot = |base: &str| PartSpot {
        base: base.to_string(),
        roof: false,
        open: false,
        broken: false,
    };
    layout.tiles.insert((0, 0, 0), vec![spot("xlframe"), spot("saddle")]);
    layout.tiles.insert((1, 0, 0), vec![spot("wheel_bicycle")]);
    layout.tiles.insert((-1, 0, 0), vec![spot("wheel_bicycle")]);
    let models = compose_vehicle(&layout, &resolver);
    let wheel = &models
        .iter()
        .find(|(k, _)| *k == (1, 0, 0))
        .expect("front wheel model")
        .1;
    let mut lo = 96;
    let mut hi = 0;
    for x in 0..32u32 {
        for y in 0..96u32 {
            for z in 0..32u32 {
                if wheel.get(x, y, z).is_some() {
                    lo = lo.min(y);
                    hi = hi.max(y);
                }
            }
        }
    }
    assert_eq!(lo, 0, "composed wheel floats: lowest voxel row {lo}");
    assert!(hi >= 40, "composed wheel height {} rows", hi + 1);
    let (mut xlo, mut xhi) = (32u32, 0u32);
    for x in 0..32u32 {
        for y in 0..96u32 {
            for z in 0..32u32 {
                if wheel.get(x, y, z).is_some() {
                    xlo = xlo.min(x);
                    xhi = xhi.max(x);
                }
            }
        }
    }
    assert!(xhi - xlo >= 20, "composed wheel width {} cols", xhi - xlo + 1);
}
