//! real-capture: the gate 6.11 crowd scene rendered with the REAL
//! UltiCa atlas (the voxelizer's `sprites.png`/`sprites.json` from
//! `assets/ultica_run1`) instead of the in-crate fixture atlas.
//!
//! Same wall/window scene, same entity layout as
//! `billboard-capture`'s crowd, so the two captures are directly
//! comparable: fixture rectangles prove the math, this proves the
//! machinery carries actual tileset art (real sprite geometry:
//! zombies are 32x48 at offset (0,-16); the hulk is 64x96 at
//! (-16,-64)).
//!
//! Usage: real-capture [--atlas-dir assets/ultica_run1]
//!                     [--out captures/billboard_real_crowd.png]
//!                     [--size WxH]
//! Exits 2 with the named NoAdapter error when no adapter exists;
//! exits 1 listing any sprite id the atlas does not contain.

use std::path::PathBuf;

use billboard::atlas::SpriteAtlas;
use billboard::composite::Facing;
use billboard::instance::FakeEntity;
use billboard::render::{render_frame, wall_scene, AlphaMode, BillboardError, Camera, CaptureRequest};
use billboard::scene::build_frame;

fn real_crowd() -> Vec<FakeEntity> {
    let mut v = Vec::new();
    let mut z1 = FakeEntity::creature(1, "mon_zombie_1", [3, 4, 0]);
    z1.overlays = vec!["overlay_effect_bleed".into()];
    v.push(z1);
    v.push(FakeEntity::creature(2, "mon_zombie_2", [4, 4, 0]));
    let mut z3 = FakeEntity::creature(3, "mon_zombie_3", [5, 5, 0]);
    z3.facing = Facing::Left;
    v.push(z3);
    v.push(FakeEntity::creature(4, "mon_skeleton", [7, 4, 0]));
    v.push(FakeEntity::creature(5, "mon_zombie_cop", [8, 5, 0]));
    // Behind the back wall: visible only through the window hole.
    v.push(FakeEntity::creature(6, "mon_zombie_1", [6, -1, 0]));
    // Fully hidden behind the wall — must NOT appear.
    v.push(FakeEntity::creature(7, "mon_skeleton", [2, -1, 0]));
    v.push(FakeEntity::creature(8, "mon_zombie_hulk", [9, 4, 0]));
    v.push(FakeEntity::creature(9, "mon_shadow", [2, 5, 0]));
    let mut dog = FakeEntity::creature(10, "mon_dog", [10, 4, 0]);
    dog.under_roof = true; // blob shadow suppressed (gate 6.8)
    v.push(dog);
    v
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut atlas_dir = PathBuf::from("assets/ultica_run1");
    let mut out = PathBuf::from("captures/billboard_real_crowd.png");
    let mut size = (1280u32, 720u32);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--atlas-dir" => { i += 1; atlas_dir = PathBuf::from(args.get(i).cloned().unwrap_or_default()); }
            "--out" => { i += 1; out = PathBuf::from(args.get(i).cloned().unwrap_or_default()); }
            "--size" => {
                i += 1;
                if let Some((w, h)) = args.get(i).and_then(|s| s.split_once('x')).and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))) {
                    size = (w, h);
                }
            }
            other => { eprintln!("real-capture: unknown argument {other}"); std::process::exit(2); }
        }
        i += 1;
    }

    let atlas = SpriteAtlas::load(&atlas_dir).unwrap_or_else(|e| {
        eprintln!("real-capture: loading atlas {}: {e}", atlas_dir.display());
        std::process::exit(1);
    });

    let entities = real_crowd();
    // Fail loudly if any sprite id is not real art in this atlas.
    let mut missing = Vec::new();
    for e in &entities {
        for id in std::iter::once(&e.sprite_id).chain(e.overlays.iter()) {
            match atlas.get(id) {
                Some(uv) => println!("real-capture: {id}: {}x{} @ offset ({},{})", uv.width, uv.height, uv.offset_x, uv.offset_y),
                None => missing.push(id.clone()),
            }
        }
    }
    if !missing.is_empty() {
        eprintln!("real-capture: sprite ids missing from atlas: {missing:?}");
        std::process::exit(1);
    }

    let camera = Camera::default();
    let frame = build_frame(&atlas, &entities, &camera, 0.0);
    let req = CaptureRequest {
        boxes: &wall_scene(),
        atlas_image: &frame.packed,
        instances: &frame.instances,
        shadows: &frame.shadows,
        camera,
        size,
        mode: AlphaMode::Blend,
        time_s: 0.0,
    };
    match render_frame(&req) {
        Ok(o) => {
            if let Some(parent) = out.parent() { std::fs::create_dir_all(parent).ok(); }
            o.image.save(&out).unwrap_or_else(|e| { eprintln!("real-capture: save: {e}"); std::process::exit(1); });
            let distinct: std::collections::HashSet<[u8; 4]> = o.image.pixels().map(|p| p.0).collect();
            println!("real-capture: {} ({}x{}, {} instances, {} distinct colors, {:.1} ms on llvmpipe/lavapipe — NOT target hardware)", out.display(), size.0, size.1, frame.instances.len(), distinct.len(), o.frame_ms);
        }
        Err(BillboardError::NoAdapter) => { eprintln!("real-capture: no wgpu adapter available"); std::process::exit(2); }
        Err(e) => { eprintln!("real-capture: {e}"); std::process::exit(1); }
    }
}
