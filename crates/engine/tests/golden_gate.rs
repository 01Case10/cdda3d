//! Gate 2g.3 — formal image-diff golden gate (closes conditions
//! 2.10 and 5.13). See docs/phase2g-imagediff.md.
//!
//! A fixed set of headless captures of the fake house (both
//! renderers, flat + textured + effects, full view + cutaway) is compared
//! pixel-by-pixel against committed golden PNGs in
//! `crates/engine/tests/golden/`.
//!
//! Regenerate the goldens (mirrors the voxelizer's 5.1 flow):
//!   UPDATE_GOLDEN=1 cargo test -p engine --test golden_gate
//! or `tools/golden.sh --update`.
//!
//! Adapter rule (the plan's vacuous-skip caveat, honored loudly):
//! with no wgpu adapter this gate proves NOTHING. The probe below
//! detects that case, prints a named NOT-RUN banner, drops a
//! SKIPPED.txt marker in the failure-artifact directory, and
//! returns — it never reports a silent ok, and a blank render is
//! never a pass (every capture asserts quads > 0 and a minimum
//! distinct-color count).

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::rt::fake_material_ids;
use mesher::block::*;

/// Serialize the GPU tests in this binary: parallel captures on a
/// shared software adapter have historically produced vacuous GPU
/// behaviour in this project; the golden gate runs sequentially.
static GPU_LOCK: Mutex<()> = Mutex::new(());

/// Lock recovering from poisoning: if one GPU test panics, the
/// other must still run (and report its own result), not die on
/// a PoisonError.
fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------------
// Tolerance model (justified in docs/phase2g-imagediff.md):
// * A pixel "matches" when every channel is within CHANNEL_TOL of
//   the golden. 8/255 (~3%) absorbs cross-driver interpolation,
//   filtering and sRGB-quantization differences (lavapipe here vs
//   a real GPU on the target machine) while staying far below the
//   shift any real material/geometry change produces (tens to
//   hundreds of levels — see the negative test).
// * At most MISMATCH_BUDGET (1%) of pixels may exceed the tolerance:
//   rasterization edge pixels are the legitimate cross-driver
//   variance and cover well under 1% of these mostly flat-shaded
//   frames.
// * Mean absolute channel error over the whole frame must stay
//   under MEAN_ABS_BUDGET, which catches a global tint shift that
//   is everywhere just under the per-pixel tolerance.
const CHANNEL_TOL: u32 = 8;
const MISMATCH_BUDGET: f64 = 0.01;
const MEAN_ABS_BUDGET: f64 = 2.0;

const SIZE: (u32, u32) = (640, 360);

// ------------------------------------------------------------------
// Scene set

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tex {
    Flat,
    /// Real UltiCa voxelizer output in assets/ultica_run1
    /// (gitignored — scene skips loudly when the dir is absent).
    Ultica,
    /// Hermetic checkerboard fixture built by this harness
    /// (same construction as raster_tex_gate's fixture).
    Fixture,
}

struct Scene {
    name: &'static str,
    renderer: RendererKind,
    tex: Tex,
    yaw: f32,
    pitch: f32,
    cutaway_z: i32,
    /// Effects variant (2g.1): field volumes + HD-2D post composited
    /// over the frame, defs injected from fx's hermetic demo set
    /// (the same injection fx_gate uses — deterministic, no file IO).
    effects: bool,
}

const SCENES: &[Scene] = &[
    Scene { name: "house_raster_flat", renderer: RendererKind::Raster, tex: Tex::Flat, yaw: 45.0, pitch: 45.0, cutaway_z: 1, effects: false },
    Scene { name: "house_raster_cutaway0", renderer: RendererKind::Raster, tex: Tex::Flat, yaw: 135.0, pitch: 50.0, cutaway_z: 0, effects: false },
    Scene { name: "house_rt_flat", renderer: RendererKind::Rt, tex: Tex::Flat, yaw: 45.0, pitch: 45.0, cutaway_z: 1, effects: false },
    Scene { name: "house_raster_textured_ultica", renderer: RendererKind::Raster, tex: Tex::Ultica, yaw: 45.0, pitch: 45.0, cutaway_z: 1, effects: false },
    Scene { name: "house_rt_textured_ultica", renderer: RendererKind::Rt, tex: Tex::Ultica, yaw: 45.0, pitch: 45.0, cutaway_z: 1, effects: false },
    Scene { name: "house_raster_textured_fixture", renderer: RendererKind::Raster, tex: Tex::Fixture, yaw: 45.0, pitch: 45.0, cutaway_z: 1, effects: false },
    Scene { name: "house_rt_textured_fixture", renderer: RendererKind::Rt, tex: Tex::Fixture, yaw: 45.0, pitch: 45.0, cutaway_z: 1, effects: false },
    // Effects scene (added with 2g.1's flags): interior view
    // (cutaway 0, the fx_gate camera) so the fire/smoke/gas volumes
    // inside the house are in frame; fields + post both on.
    Scene { name: "house_raster_effects", renderer: RendererKind::Raster, tex: Tex::Flat, yaw: 135.0, pitch: 50.0, cutaway_z: 0, effects: true },
];

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn failures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/golden-failures")
}

fn ultica_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1")
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda-voxel-golden-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ------------------------------------------------------------------
// Hermetic texture fixture (mirrors raster_tex_gate::write_fixture)

const FIX_TILE: u32 = 8;
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

fn write_fixture(dir: &Path) {
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    let id_map = fake_material_ids();
    let layer_count = FAKE_IDS_SORTED.len() * 3;
    std::fs::write(
        dir.join("blocks.json"),
        format!(
            r#"{{"format":"png-layers","tile_size":{FIX_TILE},"layer_count":{layer_count}}}"#
        ),
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
    write_fixture_layers(dir, None);
}

/// Write all fixture layer PNGs. `recolor: Some((layer, color))`
/// rewrites exactly one layer as a solid color — the negative
/// test's one-material change.
fn write_fixture_layers(dir: &Path, recolor: Option<(usize, [u8; 4])>) {
    let blocks = dir.join("blocks");
    let layer_count = FAKE_IDS_SORTED.len() * 3;
    for li in 0..layer_count {
        if let Some((target, color)) = recolor {
            if li == target {
                let img = image::RgbaImage::from_pixel(FIX_TILE, FIX_TILE, image::Rgba(color));
                img.save(blocks.join(format!("layer_{li:04}.png"))).unwrap();
                continue;
            }
        }
        let c1 = image::Rgba([
            (li as u32 * 37 % 256) as u8,
            (li as u32 * 91 % 256) as u8,
            (li as u32 * 53 % 256) as u8,
            255,
        ]);
        let c2 = image::Rgba([255 - c1.0[0], 255 - c1.0[1], 255 - c1.0[2], 255]);
        let img =
            image::RgbaImage::from_fn(FIX_TILE, FIX_TILE, |x, y| if (x + y) % 2 == 0 { c1 } else { c2 });
        img.save(blocks.join(format!("layer_{li:04}.png"))).unwrap();
    }
}

// ------------------------------------------------------------------
// Capture + diff

fn scene_args(scene: &Scene, textures: Option<PathBuf>, out: &Path) -> CaptureArgs {
    let mut args = CaptureArgs {
        snapshot: "stub-house".to_string(),
        renderer: scene.renderer,
        textures,
        camera_target: (6.0, 6.0),
        yaw_deg: scene.yaw,
        pitch_deg: scene.pitch,
        distance: 28.0,
        size: SIZE,
        out: out.to_path_buf(),
        cutaway_z: scene.cutaway_z,
        ..CaptureArgs::default()
    };
    if scene.effects {
        // fx_fields / fx_post default to on and fx_time is pinned
        // in CaptureArgs::default, so the effects frame is
        // deterministic; defs come from fx's demo set (hermetic).
        args.effects_defs = Some(fx::effect::demo_defs());
    }
    args
}

struct Diff {
    mismatched: usize,
    total: usize,
    mean_abs: f64,
    max_delta: u32,
}

impl Diff {
    fn mismatch_frac(&self) -> f64 {
        self.mismatched as f64 / self.total.max(1) as f64
    }
    fn passes(&self) -> bool {
        self.mismatch_frac() <= MISMATCH_BUDGET && self.mean_abs <= MEAN_ABS_BUDGET
    }
    fn describe(&self) -> String {
        format!(
            "{} mismatched px of {} ({:.3}%), mean |err| {:.3}, max delta {}",
            self.mismatched,
            self.total,
            self.mismatch_frac() * 100.0,
            self.mean_abs,
            self.max_delta
        )
    }
}

/// Per-pixel diff vs the tolerance model. Also renders a diff
/// image: mismatched pixels in magenta over the actual frame
/// dimmed by half, so a failure is inspectable at a glance.
fn diff_images(golden: &image::RgbaImage, actual: &image::RgbaImage) -> (Diff, image::RgbaImage) {
    assert_eq!(
        golden.dimensions(),
        actual.dimensions(),
        "golden and actual sizes differ: {:?} vs {:?}",
        golden.dimensions(),
        actual.dimensions()
    );
    let mut mismatched = 0usize;
    let mut err_sum = 0f64;
    let mut max_delta = 0u32;
    let mut diff = actual.clone();
    for (i, (g, a)) in golden.pixels().zip(actual.pixels()).enumerate() {
        let mut px_max = 0u32;
        for c in 0..4 {
            let d = (g.0[c] as i32 - a.0[c] as i32).unsigned_abs();
            px_max = px_max.max(d);
            err_sum += d as f64;
        }
        max_delta = max_delta.max(px_max);
        if px_max > CHANNEL_TOL {
            mismatched += 1;
            let x = (i as u32) % actual.width();
            let y = (i as u32) / actual.width();
            diff.put_pixel(x, y, image::Rgba([255, 0, 255, 255]));
        } else {
            let x = (i as u32) % actual.width();
            let y = (i as u32) / actual.width();
            let p = diff.get_pixel_mut(x, y);
            *p = image::Rgba([
                (p.0[0] as u32 / 2) as u8,
                (p.0[1] as u32 / 2) as u8,
                (p.0[2] as u32 / 2) as u8,
                255,
            ]);
        }
    }
    let total = (golden.width() * golden.height()) as usize;
    (
        Diff {
            mismatched,
            total,
            mean_abs: err_sum / (total * 4) as f64,
            max_delta,
        },
        diff,
    )
}

/// Loud no-adapter handling: banner on stderr AND stdout, plus a
/// SKIPPED.txt marker file. Returns true when the gate must stop.
fn loud_skip(reason: &str) -> bool {
    let banner = format!(
        "================================================================\n\
         2g.3 GOLDEN GATE: NOT RUN — {reason}\n\
         This run proves NOTHING about rendering. The golden image-diff\n\
         gate only counts on a machine with a wgpu adapter (lavapipe\n\
         counts). Do not record 2.10 / 5.13 as passed from this run.\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
    let dir = failures_dir();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("SKIPPED.txt"), format!("{banner}\n"));
    true
}

/// Probe for an adapter with a tiny capture before running the
/// set. Ok(true) = adapter present and probe non-blank.
fn probe_adapter(scratch: &Path) -> Result<bool, CaptureError> {
    let probe = Scene {
        name: "probe",
        renderer: RendererKind::Raster,
        tex: Tex::Flat,
        yaw: 45.0,
        pitch: 45.0,
        cutaway_z: 1,
        effects: false,
    };
    let mut args = scene_args(&probe, None, &scratch.join("probe.png"));
    args.size = (64, 36);
    match run_capture(&args) {
        Ok(outcome) => {
            assert!(
                outcome.quads > 0 && outcome.distinct_colors > 1,
                "adapter probe rendered a blank frame ({} quads, {} colors) — \
                 treating as vacuous, not a pass",
                outcome.quads,
                outcome.distinct_colors
            );
            Ok(true)
        }
        Err(CaptureError::NoAdapter) => Ok(false),
        Err(e) => Err(e),
    }
}

// ------------------------------------------------------------------
// The golden gate

#[test]
fn gate_2g3_golden_image_diff() {
    let _guard = gpu_lock();
    let scratch = scratch_dir("gate");
    if !probe_adapter(&scratch).expect("adapter probe") {
        loud_skip("no wgpu adapter available (probe capture returned NoAdapter)");
        return;
    }
    let update = std::env::var("UPDATE_GOLDEN").is_ok();
    let golden_root = golden_dir();
    if update {
        std::fs::create_dir_all(&golden_root).unwrap();
    }
    let fixture = scratch_dir("fixture");
    write_fixture(&fixture);
    let ultica = ultica_dir();
    let ultica_present = ultica.join("materials.json").exists();

    let mut failures: Vec<String> = Vec::new();
    let mut compared = 0usize;
    let mut regenerated = 0usize;
    for scene in SCENES {
        let textures = match scene.tex {
            Tex::Flat => None,
            Tex::Fixture => Some(fixture.clone()),
            Tex::Ultica => {
                if !ultica_present {
                    eprintln!(
                        "2g.3 scene {}: SKIPPED — {} not present (assets/ is gitignored; \
                         the textured-UltiCa scene runs only where the voxelizer output exists). \
                         This scene was NOT compared.",
                        scene.name,
                        ultica.display()
                    );
                    continue;
                }
                Some(ultica.clone())
            }
        };
        let out = scratch.join(format!("{}.png", scene.name));
        let t0 = Instant::now();
        let outcome = match run_capture(&scene_args(scene, textures, &out)) {
            Ok(o) => o,
            Err(CaptureError::NoAdapter) => {
                loud_skip(&format!(
                    "adapter vanished mid-run at scene {}",
                    scene.name
                ));
                return;
            }
            Err(e) => panic!("2g.3 scene {}: capture failed: {e}", scene.name),
        };
        let elapsed = t0.elapsed();
        assert!(
            outcome.quads > 0 && outcome.distinct_colors > 5,
            "2g.3 scene {}: capture looks blank ({} quads/voxels, {} distinct colors) — \
             a blank render is never a pass",
            scene.name,
            outcome.quads,
            outcome.distinct_colors
        );
        let actual = image::open(&out).unwrap().to_rgba8();
        assert_eq!(actual.dimensions(), SIZE, "2g.3 scene {}: wrong size", scene.name);

        if update {
            let dest = golden_root.join(format!("{}.png", scene.name));
            std::fs::copy(&out, &dest).unwrap();
            regenerated += 1;
            println!(
                "2g.3 UPDATE_GOLDEN: {} <- {} ({}x{}, {:?})",
                dest.display(),
                scene.name,
                SIZE.0,
                SIZE.1,
                elapsed
            );
            continue;
        }

        let golden_path = golden_root.join(format!("{}.png", scene.name));
        if !golden_path.exists() {
            failures.push(format!(
                "{}: golden missing ({}) — run UPDATE_GOLDEN=1 cargo test -p engine --test golden_gate (or tools/golden.sh --update) if this scene is new",
                scene.name,
                golden_path.display()
            ));
            continue;
        }
        let golden = image::open(&golden_path).unwrap().to_rgba8();
        let (diff, diff_img) = diff_images(&golden, &actual);
        compared += 1;
        println!(
            "2g.3 {}: {} [capture {:?}, {} quads/voxels, {} colors]",
            scene.name,
            diff.describe(),
            elapsed,
            outcome.quads,
            outcome.distinct_colors
        );
        if !diff.passes() {
            let fdir = failures_dir();
            std::fs::create_dir_all(&fdir).unwrap();
            let actual_path = fdir.join(format!("{}.actual.png", scene.name));
            let diff_path = fdir.join(format!("{}.diff.png", scene.name));
            actual.save(&actual_path).unwrap();
            diff_img.save(&diff_path).unwrap();
            failures.push(format!(
                "{}: GOLDEN MISMATCH — {} (budget: <= {:.1}% px over tol {}, mean |err| <= {}). \
                 actual: {}; diff (magenta = mismatch): {}",
                scene.name,
                diff.describe(),
                MISMATCH_BUDGET * 100.0,
                CHANNEL_TOL,
                MEAN_ABS_BUDGET,
                actual_path.display(),
                diff_path.display()
            ));
        }
    }
    if update {
        println!(
            "2g.3 UPDATE_GOLDEN: regenerated {regenerated} goldens in {}",
            golden_root.display()
        );
        return;
    }
    assert!(
        compared > 0 || !failures.is_empty(),
        "2g.3: no scenes were compared and none failed — the gate proves nothing (all scenes skipped?)"
    );
    assert!(
        failures.is_empty(),
        "2g.3 golden gate FAILED ({} of {} compared scenes):\n{}",
        failures.len(),
        compared,
        failures.join("\n")
    );
    println!(
        "2g.3 golden gate: {compared} scenes within tolerance (tol {CHANNEL_TOL}/channel, \
         budget {:.1}% px, mean |err| <= {MEAN_ABS_BUDGET})",
        MISMATCH_BUDGET * 100.0
    );
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The loud-skip machinery itself: the banner names the gate as
/// NOT RUN and the SKIPPED.txt marker is written. (The NoAdapter
/// detection feeding it is the same CaptureError::NoAdapter path
/// gate 2.10's capture test exercises; on this VM the branch
/// cannot be forced live because the engine pins Backends::all()
/// and both mesa Vulkan and GL adapters are present — see
/// docs/phase2g-imagediff.md.)
#[test]
fn gate_2g3_skip_notice_is_loud() {
    let marker = failures_dir().join("SKIPPED.txt");
    let _ = std::fs::remove_file(&marker);
    assert!(loud_skip("unit-test probe of the skip path"));
    let text = std::fs::read_to_string(&marker).expect("SKIPPED.txt must be written");
    assert!(text.contains("2g.3 GOLDEN GATE: NOT RUN"), "{text}");
    assert!(text.contains("proves NOTHING"), "{text}");
    let _ = std::fs::remove_file(&marker);
}

// ------------------------------------------------------------------
// Negative test: a deliberate one-material change must FAIL the
// gate, with an inspectable diff artifact.

#[test]
fn gate_2g3_negative_one_material_change_fails() {
    let _guard = gpu_lock();
    let scratch = scratch_dir("negative");
    if !probe_adapter(&scratch).expect("adapter probe") {
        loud_skip("no wgpu adapter available (negative test probe returned NoAdapter)");
        return;
    }
    let fixture = scratch_dir("neg-fixture");
    write_fixture(&fixture);
    let scene = SCENES
        .iter()
        .find(|s| s.name == "house_raster_textured_fixture")
        .unwrap();

    // Baseline capture with the untouched fixture.
    let base_path = scratch.join("base.png");
    run_capture(&scene_args(scene, Some(fixture.clone()), &base_path))
        .expect("negative-test baseline capture");
    let base = image::open(&base_path).unwrap().to_rgba8();

    // Control: a frame diffed against itself passes the budget.
    let (control, _) = diff_images(&base, &base);
    assert!(
        control.passes(),
        "control diff (frame vs itself) must pass: {}",
        control.describe()
    );

    // The one-material change: wall is FAKE_IDS_SORTED[1], owning
    // layers 3 (top), 4 (side), 5 (bottom). Recolor ONLY the side
    // layer (layer 4) solid red in the scratch fixture copy — no
    // committed data is touched.
    let wall_side_layer = 3 * 1 + 1;
    write_fixture_layers(&fixture, Some((wall_side_layer, [255, 0, 0, 255])));

    let changed_path = scratch.join("changed.png");
    run_capture(&scene_args(scene, Some(fixture.clone()), &changed_path))
        .expect("negative-test changed capture");
    let changed = image::open(&changed_path).unwrap().to_rgba8();

    let (diff, diff_img) = diff_images(&base, &changed);
    // The diff artifact is inspectable even though this test
    // *expects* the mismatch: same location a real failure uses.
    let fdir = failures_dir();
    std::fs::create_dir_all(&fdir).unwrap();
    let diff_path = fdir.join("negative_one_material.diff.png");
    let changed_copy = fdir.join("negative_one_material.actual.png");
    diff_img.save(&diff_path).unwrap();
    changed.save(&changed_copy).unwrap();
    println!(
        "2g.3 NEGATIVE TEST: one material changed (fixture layer {wall_side_layer} -> solid red): {}; \
         diff image: {}; changed frame: {}",
        diff.describe(),
        diff_path.display(),
        changed_copy.display()
    );
    assert!(
        !diff.passes(),
        "one-material change must FAIL the golden budget, but the diff passed: {}",
        diff.describe()
    );
    // Margin check, calibrated to the measured signal: the wall
    // side layer covers ~1.7% of this frame, so one material
    // lands at 1.73% mismatched vs the 1.0% budget — over budget
    // (the gate fails, above) with thousands of pixels, not a
    // borderline handful.
    assert!(
        diff.mismatch_frac() > MISMATCH_BUDGET && diff.mismatched > 1_000,
        "one-material change should exceed the budget with a substantial pixel count          (got {:.3}% / {} px vs {:.1}% budget)",
        diff.mismatch_frac() * 100.0,
        diff.mismatched,
        MISMATCH_BUDGET * 100.0
    );
    // The diff image itself must actually mark the changed pixels.
    let marked = diff_img
        .pixels()
        .filter(|p| p.0 == [255, 0, 255, 255])
        .count();
    assert_eq!(
        marked, diff.mismatched,
        "diff image must mark exactly the mismatched pixels"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
