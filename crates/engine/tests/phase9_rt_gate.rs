//! Phase 9 slice 3h gate (plan §11b, 9.5 engine wiring): fine
//! props in the RT renderer.
//!
//! - 9.10 contract, RT form: sample-scene captures at cutaway 0
//!   — fine OFF vs fine ON with an explicitly EMPTY prop scene
//!   must be pixel-identical; fine ON with the fixture props
//!   must differ and show at least 3 fixture palette colours.
//!   (The raster gate's "more quads" check has no RT analogue:
//!   the RT outcome counts coarse-tree leaves, and stamping
//!   SUPPRESSES coarse furniture blocks, so the leaf count is
//!   expected to move down, not up.)
//! - Lit contract: on the stub-house with a lighting scene,
//!   fixture props change the frame; with every placement tile
//!   marked BLANK (never seen), the props are skipped by the
//!   mask-before-stamping rule and the frame is pixel-identical
//!   to the lit fine-OFF frame.
//!
//! With no wgpu adapter the GPU test prints a loud NOT RUN
//! banner and returns (it proves nothing), like the raster gate.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::frame::{LightingScene, PropScene};

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda-voxel-phase9rt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn loud_skip(reason: &str) -> bool {
    let banner = format!(
        "================================================================\n\
         PHASE 9 RT GATE: NOT RUN — {reason}\n\
         This run proves NOTHING about RT fine-props rendering. The gate\n\
         only counts on a machine with a wgpu adapter (lavapipe counts).\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
    true
}

fn rt_args(out: &Path, snapshot: &str, fine_props: bool, props: Option<PropScene>) -> CaptureArgs {
    CaptureArgs {
        snapshot: snapshot.to_string(),
        renderer: RendererKind::Rt,
        camera_target: (6.5, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 50.0,
        distance: 15.0,
        size: (640, 360),
        out: out.to_path_buf(),
        cutaway_z: 0,
        fine_props,
        props,
        ..CaptureArgs::default()
    }
}

/// (pixels whose max channel delta exceeds 8, mean abs delta) —
/// the raster gate's tolerance model.
fn diff_stats(a: &image::RgbaImage, b: &image::RgbaImage) -> (usize, f64) {
    assert_eq!(a.dimensions(), b.dimensions());
    let mut mismatched = 0usize;
    let mut err_sum = 0f64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let mut px_max = 0u32;
        for c in 0..4 {
            let d = (pa.0[c] as i32 - pb.0[c] as i32).unsigned_abs();
            px_max = px_max.max(d);
            err_sum += d as f64;
        }
        if px_max > 8 {
            mismatched += 1;
        }
    }
    let total = (a.width() * a.height()) as usize;
    (mismatched, err_sum / (total * 4) as f64)
}

fn capture_rt(args: &CaptureArgs) -> Result<image::RgbaImage, CaptureError> {
    run_capture(args)?;
    Ok(image::open(&args.out).unwrap().to_rgba8())
}

#[test]
fn phase9_rt_empty_identical_and_fixture_visible() {
    let _guard = gpu_lock();
    let scratch = scratch_dir("gpu");

    // (a) Fine OFF.
    let off_path = scratch.join("off.png");
    let off_img = match capture_rt(&rt_args(&off_path, "sample-scene", false, None)) {
        Ok(img) => img,
        Err(CaptureError::NoAdapter) => {
            loud_skip("no wgpu adapter available (capture returned NoAdapter)");
            return;
        }
        Err(e) => panic!("phase9 RT off capture failed: {e}"),
    };

    // (a) Fine ON with an explicitly EMPTY prop scene.
    let empty_path = scratch.join("empty.png");
    let empty_img = capture_rt(&rt_args(
        &empty_path,
        "sample-scene",
        true,
        Some(PropScene {
            models: Vec::new(),
            placements: Vec::new(),
        }),
    ))
    .expect("phase9 RT empty-props capture");
    let (mismatched, mean_abs) = diff_stats(&off_img, &empty_img);
    println!(
        "phase9 RT 9.10(a): empty-props diff {mismatched} mismatched px, mean |err| {mean_abs}"
    );
    assert_eq!(
        mismatched, 0,
        "RT fine ON with an empty prop scene must be pixel-identical to fine OFF"
    );
    assert_eq!(mean_abs, 0.0, "mean abs error must be 0 for the empty scene");

    // (b) Fine ON with the fixture props (flag on, no explicit
    // scene — the fixture is resolved inside the RT path).
    let fix_path = scratch.join("fixture.png");
    let fix_img =
        capture_rt(&rt_args(&fix_path, "sample-scene", true, None)).expect("phase9 RT fixture capture");
    let (fx_mismatched, fx_mean) = diff_stats(&off_img, &fix_img);
    let total_px = (fix_img.width() * fix_img.height()) as usize;
    let frac = fx_mismatched as f64 / total_px as f64;
    println!(
        "phase9 RT 9.10(b): fixture diff {fx_mismatched} px ({:.3}%), mean |err| {fx_mean:.3}",
        frac * 100.0
    );
    assert!(
        frac > 0.001,
        "RT fixture props must change more than 0.1% of pixels (got {:.4}%)",
        frac * 100.0
    );

    // Palette colours present (same tolerance model as the
    // raster gate: palette literals are sRGB bytes, tolerance
    // 12 per channel; RT top faces shade near factor 1).
    let scene = PropScene::fixture("sample-scene").expect("fixture scene");
    let mut palette: Vec<[u8; 3]> = Vec::new();
    for (_, model) in &scene.models {
        for rgb in &model.palette {
            if !palette.contains(rgb) {
                palette.push(*rgb);
            }
        }
    }
    let frame_colors: HashSet<[u8; 3]> = fix_img.pixels().map(|p| [p.0[0], p.0[1], p.0[2]]).collect();
    let present: Vec<[u8; 3]> = palette
        .iter()
        .copied()
        .filter(|rgb| {
            frame_colors
                .iter()
                .any(|c| (0..3).all(|ch| (c[ch] as i32 - rgb[ch] as i32).abs() <= 12))
        })
        .collect();
    println!(
        "phase9 RT 9.10(b): {} of {} fixture palette colours present: {present:?}",
        present.len(),
        palette.len()
    );
    assert!(
        present.len() >= 3,
        "at least 3 fixture palette colours must be present (found {})",
        present.len()
    );
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn phase9_rt_lit_props_masked_by_blank_tiles() {
    let _guard = gpu_lock();
    let scratch = scratch_dir("lit");

    // Lit stub-house, fine OFF.
    let mut off_args = rt_args(&scratch.join("lit-off.png"), "stub-house", false, None);
    off_args.lighting = Some(LightingScene::fake_house(12.0, lighting::WeatherKind::Clear));
    let off_img = match capture_rt(&off_args) {
        Ok(img) => img,
        Err(CaptureError::NoAdapter) => {
            loud_skip("no wgpu adapter available (capture returned NoAdapter)");
            return;
        }
        Err(e) => panic!("phase9 RT lit off capture failed: {e}"),
    };

    // Lit + fixture props: the frame must change (props render
    // under the lighting contract).
    let mut fix_args = rt_args(&scratch.join("lit-fix.png"), "stub-house", true, None);
    fix_args.lighting = Some(LightingScene::fake_house(12.0, lighting::WeatherKind::Clear));
    let fix_img = capture_rt(&fix_args).expect("phase9 RT lit fixture capture");
    let (fx_mismatched, _) = diff_stats(&off_img, &fix_img);
    println!("phase9 RT lit: fixture diff {fx_mismatched} px vs lit off");
    assert!(
        fx_mismatched > 0,
        "lit RT fixture props must change the frame"
    );

    // Lit + fixture props with every placement tile BLANK: the
    // mask runs before stamping, the placements are skipped,
    // and the frame must equal the lit OFF frame exactly.
    let blank_tiles: Vec<[i32; 3]> = PropScene::fixture("stub-house")
        .expect("fixture scene")
        .placements
        .iter()
        .map(|p| [p.x, p.y, p.level as i32])
        .collect();
    assert!(!blank_tiles.is_empty());
    let mut blank_args = rt_args(&scratch.join("lit-blank.png"), "stub-house", true, None);
    blank_args.lighting = Some(LightingScene::fake_house_marked(
        12.0,
        lighting::WeatherKind::Clear,
        &blank_tiles,
        &[],
    ));
    // The OFF comparison frame must carry the SAME marked
    // lighting (the mask also removes the tiles' own blocks).
    let mut off_marked_args = rt_args(&scratch.join("lit-off-marked.png"), "stub-house", false, None);
    off_marked_args.lighting = Some(LightingScene::fake_house_marked(
        12.0,
        lighting::WeatherKind::Clear,
        &blank_tiles,
        &[],
    ));
    let blank_img = capture_rt(&blank_args).expect("phase9 RT lit blank capture");
    let off_marked_img = capture_rt(&off_marked_args).expect("phase9 RT lit off-marked capture");
    let (b_mismatched, b_mean) = diff_stats(&off_marked_img, &blank_img);
    println!(
        "phase9 RT lit: blank-masked props diff {b_mismatched} mismatched px, mean |err| {b_mean}"
    );
    assert_eq!(
        b_mismatched, 0,
        "props on BLANK tiles must be skipped entirely (frame = lit off with same mask)"
    );
    assert_eq!(b_mean, 0.0);
    let _ = std::fs::remove_dir_all(&scratch);
}

// ---------------------------------------------------------------------
// Phase 9 slice 3i (9.8): frame composition regression against a
// fine-layer world — the Phase 8 vehicle wiring (vehicles stamped
// in the world pass) composed WITH fine props, both renderers.
// The combined frame must differ from each single-input frame:
// neither pass may drop or overwrite the other.
// ---------------------------------------------------------------------

use engine::frame::VehicleScene;

#[test]
fn phase9_regression_vehicles_and_props_compose() {
    let _guard = gpu_lock();
    let scratch = scratch_dir("compose");
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let mk = |name: &str, fine: bool, car: bool| {
            let mut a = rt_args(&scratch.join(name), "stub-house", fine, None);
            a.renderer = renderer;
            a.camera_target = (5.5, 5.0);
            a.distance = 14.0;
            if car {
                a.vehicles = Some(VehicleScene::fixture_car([4, 5, 0], 23.0));
            }
            a
        };
        let plain = match capture_rt(&mk("plain.png", false, false)) {
            Ok(img) => img,
            Err(CaptureError::NoAdapter) => {
                loud_skip("no wgpu adapter available (capture returned NoAdapter)");
                return;
            }
            Err(e) => panic!("9.8 plain capture failed ({renderer:?}): {e}"),
        };
        let props_only = capture_rt(&mk("props.png", true, false))
            .unwrap_or_else(|e| panic!("9.8 props capture failed ({renderer:?}): {e}"));
        let car_only = capture_rt(&mk("car.png", false, true))
            .unwrap_or_else(|e| panic!("9.8 car capture failed ({renderer:?}): {e}"));
        let both = capture_rt(&mk("both.png", true, true))
            .unwrap_or_else(|e| panic!("9.8 combined capture failed ({renderer:?}): {e}"));

        let total = (plain.width() * plain.height()) as f64;
        let frac = |a: &image::RgbaImage, b: &image::RgbaImage| diff_stats(a, b).0 as f64 / total;
        let (d_props, d_car, d_both_props, d_both_car) = (
            frac(&plain, &props_only),
            frac(&plain, &car_only),
            frac(&props_only, &both),
            frac(&car_only, &both),
        );
        println!(
            "9.8 {renderer:?}: props {d_props:.4}, car {d_car:.4}, \
             both-vs-props {d_both_props:.4}, both-vs-car {d_both_car:.4} (frame fraction)"
        );
        assert!(d_props > 0.0005, "{renderer:?}: props alone must show");
        assert!(d_car > 0.0005, "{renderer:?}: car alone must show");
        assert!(
            d_both_props > 0.0005,
            "{renderer:?}: adding the car to the prop frame must change it (vehicle pass lost?)"
        );
        assert!(
            d_both_car > 0.0005,
            "{renderer:?}: adding props to the car frame must change it (fine pass lost?)"
        );
    }
    let _ = std::fs::remove_dir_all(&scratch);
}
