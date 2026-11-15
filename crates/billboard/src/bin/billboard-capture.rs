//! billboard-capture: headless captures for Phase 6 gates.
//!
//! Usage: billboard-capture [--out-dir captures] [--size WxH]
//! Writes billboard_crowd.png (gate 6.11 zombie crowd at the
//! wall/window scene) and billboard_stress.png (2000 billboards,
//! gate 6.10) and prints llvmpipe frame times, labeled as such.
//! Exits 2 with the named NoAdapter error when no adapter exists.

use std::path::PathBuf;

use billboard::atlas::fixture_atlas;
use billboard::render::{render_frame, wall_scene, AlphaMode, BillboardError, Camera, CaptureRequest};
use billboard::scene::{build_frame, crowd_entities, stress_camera, stress_entities};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out_dir = PathBuf::from("captures");
    let mut size = (640u32, 360u32);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => { i += 1; out_dir = PathBuf::from(args.get(i).cloned().unwrap_or_default()); }
            "--size" => {
                i += 1;
                if let Some((w, h)) = args.get(i).and_then(|s| s.split_once('x')).and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))) {
                    size = (w, h);
                }
            }
            other => { eprintln!("billboard-capture: unknown argument {other}"); std::process::exit(2); }
        }
        i += 1;
    }
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| { eprintln!("billboard-capture: {e}"); std::process::exit(1); });
    let atlas = fixture_atlas();
    let boxes = wall_scene();

    // Gate 6.11 crowd.
    let camera = Camera::default();
    let frame = build_frame(&atlas, &crowd_entities(), &camera, 0.0);
    let req = CaptureRequest { boxes: &boxes, atlas_image: &frame.packed, instances: &frame.instances, shadows: &frame.shadows, camera, size, mode: AlphaMode::Blend, time_s: 0.0 };
    match render_frame(&req) {
        Ok(out) => {
            let path = out_dir.join("billboard_crowd.png");
            out.image.save(&path).unwrap_or_else(|e| { eprintln!("billboard-capture: save: {e}"); std::process::exit(1); });
            let distinct: std::collections::HashSet<[u8; 4]> = out.image.pixels().map(|p| p.0).collect();
            println!("billboard-capture: {} ({}x{}, {} instances, {} distinct colors, {:.1} ms on llvmpipe/lavapipe — NOT target hardware)", path.display(), size.0, size.1, frame.instances.len(), distinct.len(), out.frame_ms);
        }
        Err(BillboardError::NoAdapter) => { eprintln!("billboard-capture: no wgpu adapter available"); std::process::exit(2); }
        Err(e) => { eprintln!("billboard-capture: {e}"); std::process::exit(1); }
    }

    // Gate 6.10 stress: 2000 billboards.
    let scam = stress_camera();
    let sframe = build_frame(&atlas, &stress_entities(2000), &scam, 0.0);
    let sreq = CaptureRequest { boxes: &boxes, atlas_image: &sframe.packed, instances: &sframe.instances, shadows: &sframe.shadows, camera: scam, size, mode: AlphaMode::Blend, time_s: 0.0 };
    match render_frame(&sreq) {
        Ok(out) => {
            let path = out_dir.join("billboard_stress.png");
            out.image.save(&path).unwrap_or_else(|e| { eprintln!("billboard-capture: save: {e}"); std::process::exit(1); });
            println!("billboard-capture: {} ({} instances, {:.1} ms on llvmpipe/lavapipe — NOT target hardware)", path.display(), sframe.instances.len(), out.frame_ms);
        }
        Err(BillboardError::NoAdapter) => { eprintln!("billboard-capture: no wgpu adapter available"); std::process::exit(2); }
        Err(e) => { eprintln!("billboard-capture: {e}"); std::process::exit(1); }
    }
}
