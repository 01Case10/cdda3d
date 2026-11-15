//! fx-capture: headless captures of the Phase 2b effect test scene.
//!
//! Usage: fx-capture [--out-dir captures] [--size WxH] [--time SECONDS]
//! Writes effects_post_off.png, effects_post_on.png and one A/B per
//! post pass (bloom / dof / grade). On a machine with no wgpu adapter
//! it exits 2 with the named NoAdapter error and writes nothing.

use std::path::PathBuf;

use fx::post::PostConfig;
use fx::render::{render_frame, Camera, CaptureRequest, FxError};
use fx::scene::{fire_scene, test_scene};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out_dir = PathBuf::from("captures");
    let mut size = (640u32, 360u32);
    let mut time = 1.25f32;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => {
                i += 1;
                out_dir = PathBuf::from(args.get(i).cloned().unwrap_or_default());
            }
            "--size" => {
                i += 1;
                if let Some((w, h)) = args
                    .get(i)
                    .and_then(|s| s.split_once('x'))
                    .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                {
                    size = (w, h);
                }
            }
            "--time" => {
                i += 1;
                time = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(time);
            }
            other => {
                eprintln!("fx-capture: unknown argument {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    let (boxes, fields) = test_scene();
    let camera = Camera::default();
    let variants: Vec<(&str, PostConfig)> = vec![
        ("effects_post_off", PostConfig::all_off()),
        ("effects_post_on", PostConfig::all_on()),
        (
            "effects_bloom",
            PostConfig {
                bloom: true,
                ..PostConfig::all_off()
            },
        ),
        (
            "effects_dof",
            PostConfig {
                dof: true,
                ..PostConfig::all_off()
            },
        ),
        (
            "effects_grade",
            PostConfig {
                grade: true,
                ..PostConfig::all_off()
            },
        ),
    ];
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| {
        eprintln!("fx-capture: cannot create out dir: {e}");
        std::process::exit(1);
    });
    for (name, post) in &variants {
        let req = CaptureRequest {
            boxes: &boxes,
            fields: &fields,
            camera,
            size,
            post: *post,
            time,
            clear: None,
        };
        match render_frame(&req) {
            Ok(img) => {
                let path = out_dir.join(format!("{name}.png"));
                if let Err(e) = img.save(&path) {
                    eprintln!("fx-capture: save failed: {e}");
                    std::process::exit(1);
                }
                let distinct: std::collections::HashSet<[u8; 4]> =
                    img.pixels().map(|p| p.0).collect();
                println!(
                    "fx-capture: {} ({}x{}, {} distinct colors)",
                    path.display(),
                    img.width(),
                    img.height(),
                    distinct.len()
                );
            }
            Err(FxError::NoAdapter) => {
                eprintln!("fx-capture: no wgpu adapter available");
                std::process::exit(2);
            }
            Err(e) => {
                eprintln!("fx-capture: {e}");
                std::process::exit(1);
            }
        }
    }

    // Night fire showcase (flame redo, 2026-11-15): dark scene,
    // fire as the light source, hotter bloom than the neutral set.
    let (fboxes, ffields) = fire_scene();
    let fire_cam = Camera {
        target: glam::Vec3::new(7.0, 0.8, -4.5),
        yaw_deg: 180.0,
        pitch_deg: 26.0,
        distance: 12.0,
    };
    let fire_post = PostConfig {
        bloom_threshold: 0.5,
        bloom_strength: 1.0,
        // Chromatic offset off for this shot: its R/B channel split
        // leaves a green fringe on the flames.
        chromatic: false,
        ..PostConfig::all_on()
    };
    let req = CaptureRequest {
        boxes: &fboxes,
        fields: &ffields,
        camera: fire_cam,
        size,
        post: fire_post,
        time,
        clear: Some([0.015, 0.015, 0.025]),
    };
    match render_frame(&req) {
        Ok(img) => {
            let path = out_dir.join("effects_fire.png");
            if let Err(e) = img.save(&path) {
                eprintln!("fx-capture: save failed: {e}");
                std::process::exit(1);
            }
            let distinct: std::collections::HashSet<[u8; 4]> =
                img.pixels().map(|p| p.0).collect();
            println!(
                "fx-capture: {} ({}x{}, {} distinct colors)",
                path.display(),
                img.width(),
                img.height(),
                distinct.len()
            );
        }
        Err(FxError::NoAdapter) => {
            eprintln!("fx-capture: no wgpu adapter available");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("fx-capture: {e}");
            std::process::exit(1);
        }
    }
}
