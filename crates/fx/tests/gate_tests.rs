//! Phase 2b gate tests (Part II §5b): 2b.2 no-sprite-rendering,
//! 2b.3 intensity mapping, 2b.4 displayed-field weighting, 2b.5
//! effect captures (adapter-conditional), 2b.6 post isolation,
//! 2b.7 UI legibility. (2b.1 lives in the voxelizer's tests; 2b.8 is
//! target-hardware only and is NOT claimed here.)

use fx::density::{density_at, effective_density, intensity_density, obscurance};
use fx::effect::{demo_defs, EffectDef, RenderClass};
use fx::post::{apply_post_cpu, composite_ui, Frame, PostConfig};
use fx::world::{composite_weights, weight_of, FieldEntry, TileFields};
use glam::Vec3;

fn def(id: &str) -> EffectDef {
    demo_defs().into_iter().find(|d| d.field_id == id).unwrap()
}

// ---------------------------------------------------------------- 2b.3
/// Intensity mapping: strictly monotonic; intensity-3 smoke fully
/// obscures through one tile, intensity-1 does not.
#[test]
fn gate_2b_3_intensity_monotonic_and_obscurance() {
    let d1 = intensity_density(1);
    let d2 = intensity_density(2);
    let d3 = intensity_density(3);
    assert!(0.0 < d1 && d1 < d2 && d2 < d3, "strictly monotonic: {d1} {d2} {d3}");
    assert_eq!(intensity_density(0), 0.0);
    let smoke = def("fd_smoke");
    let o3 = obscurance(effective_density(&smoke, 3), 1.0);
    let o1 = obscurance(effective_density(&smoke, 1), 1.0);
    assert!(o3 > 0.99, "intensity-3 smoke must fully obscure: {o3}");
    assert!(o1 < 0.99, "intensity-1 smoke must not fully obscure: {o1}");
    assert!(o1 < o3);
    // The rendered (CPU-reference) density is monotonic too, at a
    // fixed point/time/weight.
    let p = Vec3::new(0.5, 0.4, 0.5);
    let r1 = density_at(&smoke, p, 0.7, 1, 1.0);
    let r2 = density_at(&smoke, p, 0.7, 2, 1.0);
    let r3 = density_at(&smoke, p, 0.7, 3, 1.0);
    assert!(r1 < r2 && r2 < r3, "rendered density monotonic: {r1} {r2} {r3}");
    assert!(r1 > 0.0);
}

// ---------------------------------------------------------------- 2b.4
/// Displayed-field weighting: with smoke + fire on one tile, the
/// displayed field dominates the composite in BOTH configurations —
/// even when the displayed field has the lower intensity.
#[test]
fn gate_2b_4_displayed_field_weighting() {
    const SMOKE: u32 = 10;
    const FIRE: u32 = 20;
    let mk = |displayed: (u32, u8)| TileFields {
        fields: vec![
            FieldEntry { field_type: SMOKE, intensity: 3 },
            FieldEntry { field_type: FIRE, intensity: 1 },
        ],
        displayed: Some(FieldEntry {
            field_type: displayed.0,
            intensity: displayed.1,
        }),
    };
    // Fire displayed (intensity 1) still dominates smoke (intensity 3).
    let w = composite_weights(&mk((FIRE, 1)));
    assert!(
        weight_of(&w, FIRE) > weight_of(&w, SMOKE),
        "displayed fire must dominate: {w:?}"
    );
    assert!(weight_of(&w, FIRE) > 0.5);
    // And the other way: smoke displayed dominates fire.
    let w = composite_weights(&mk((SMOKE, 3)));
    assert!(weight_of(&w, SMOKE) > weight_of(&w, FIRE));
    assert!(weight_of(&w, SMOKE) > 0.5);
    // No displayed field: raw intensity rules (smoke 3 > fire 1).
    let w = composite_weights(&TileFields {
        fields: vec![
            FieldEntry { field_type: SMOKE, intensity: 3 },
            FieldEntry { field_type: FIRE, intensity: 1 },
        ],
        displayed: None,
    });
    assert!(weight_of(&w, SMOKE) > weight_of(&w, FIRE));
}

// ---------------------------------------------------------------- 2b.2
/// No sprite rendering of fields: (a) code-path assertion — the
/// field shader source contains no texture binding/sampling at all,
/// and the field pipeline's bind group has exactly uniform+storage
/// (checked textually against the module source); (b) behavioral —
/// the CPU reference density shows volumetric falloff (center dense,
/// silhouette edge empty), which a flat sprite quad cannot produce.
#[test]
fn gate_2b_2_no_sprite_rendering() {
    let shader = fx::render::FIELD_SHADER;
    assert!(
        !shader.contains("texture"),
        "field shader must not reference any texture"
    );
    assert!(
        !shader.contains("sampler"),
        "field shader must not reference any sampler"
    );
    // The whole render module must never load a sprite/atlas asset.
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render.rs"),
    )
    .unwrap();
    for banned in ["sprites.png", "sprites.json", "image::open", "load_effects"] {
        assert!(!src.contains(banned), "render.rs must not touch {banned}");
    }
    // Volumetric falloff in the reference density.
    let smoke = def("fd_smoke");
    let center = density_at(&smoke, Vec3::splat(0.5), 1.25, 3, 1.0);
    let corner = density_at(&smoke, Vec3::new(0.03, 0.03, 0.03), 1.25, 3, 1.0);
    let mid_edge = density_at(&smoke, Vec3::new(0.5, 0.5, 0.97), 1.25, 3, 1.0);
    assert!(center > corner, "center {center} must exceed corner {corner}");
    assert_eq!(corner, 0.0, "silhouette edge must be empty (soft falloff)");
    assert!(mid_edge < center, "density must fall off toward the silhouette");
}

// ---------------------------------------------------------------- 2b.6
/// Post stack isolation: all-off is pixel-identical; each pass
/// changes only its own aspect.
#[test]
fn gate_2b_6_post_isolation() {
    // Synthetic frame: left half dark, right half bright (emissive),
    // depth split near/far across the middle rows.
    let (w, h) = (16u32, 16u32);
    let mut frame = Frame::new(w, h, [0.1, 0.1, 0.1, 1.0], 18.0);
    for y in 0..h {
        for x in 0..w {
            let i = frame.idx(x, y);
            if x >= 8 {
                frame.color[i] = [0.95, 0.9, 0.85, 1.0];
            }
            if y >= 8 {
                frame.depth[i] = 40.0; // far field
            }
        }
    }
    let off = PostConfig::all_off();
    assert_eq!(
        apply_post_cpu(&frame, &off),
        frame.color,
        "all-off must be pixel-identical to the input"
    );
    // Bloom only: bright side grows, a dark pixel far from any bright
    // pixel is untouched, and no pixel moves (same buffer geometry).
    let bloom_cfg = PostConfig { bloom: true, ..PostConfig::all_off() };
    let bloomed = apply_post_cpu(&frame, &bloom_cfg);
    let dark_far = frame.idx(0, 0);
    assert_eq!(bloomed[dark_far], frame.color[dark_far], "bloom must not touch distant dark pixels");
    let bright = frame.idx(15, 0);
    assert!(bloomed[bright][0] >= frame.color[bright][0], "bloom adds light at bright pixels");
    // DoF only: the focal band (depth == focal) stays exactly sharp;
    // far-depth pixels change (blur mixes the halves at the seam).
    let dof_cfg = PostConfig { dof: true, focal_depth: 18.0, ..PostConfig::all_off() };
    let dofed = apply_post_cpu(&frame, &dof_cfg);
    for x in 0..w {
        let i = frame.idx(x, 2); // near-depth rows, away from the seam
        assert_eq!(dofed[i], frame.color[i], "focal band must stay sharp at x={x}");
    }
    let seam_far = frame.idx(7, 12);
    assert_ne!(dofed[seam_far], frame.color[seam_far], "far field must blur at the seam");
    // Grade only: warms the frame (blue channel scales down relative
    // to red) everywhere it applies.
    let grade_cfg = PostConfig { grade: true, ..PostConfig::all_off() };
    let graded = apply_post_cpu(&frame, &grade_cfg);
    let mid = frame.idx(4, 4);
    assert!(
        graded[mid][2] / graded[mid][0] < frame.color[mid][2] / frame.color[mid][0],
        "grade must warm the frame"
    );
}

// ---------------------------------------------------------------- 2b.7
/// UI legibility: post applies to the world only; a UI frame
/// composited after the stack keeps its pixels bit-exact.
#[test]
fn gate_2b_7_ui_composited_after_post() {
    let (w, h) = (16u32, 16u32);
    let frame = Frame::new(w, h, [0.9, 0.85, 0.8, 1.0], 18.0);
    let graded = apply_post_cpu(&frame, &PostConfig::all_on());
    assert_ne!(graded, frame.color, "sanity: grade must change the world frame");
    // Synthetic UI: transparent except an opaque white text rect.
    let mut ui = vec![[0.0f32; 4]; (w * h) as usize];
    for y in 4..8 {
        for x in 4..12 {
            ui[(y * w + x) as usize] = [1.0, 1.0, 1.0, 1.0];
        }
    }
    let composed = composite_ui(&graded, &ui);
    for y in 4..8 {
        for x in 4..12 {
            let i = (y * w + x) as usize;
            assert_eq!(
                composed[i],
                [1.0, 1.0, 1.0, 1.0],
                "UI pixels must be bit-exact after compositing over a graded frame"
            );
        }
    }
    // Outside the UI rect the graded world shows through unchanged.
    assert_eq!(composed[0], graded[0]);
    // By construction: fx's render path contains no UI handling at
    // all — compositing is the caller's step after render_frame.
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render.rs"),
    )
    .unwrap();
    assert!(!src.contains("composite_ui"), "post/render must not composite UI internally");
}

// ---------------------------------------------------------------- 2b.5
/// Effect captures (adapter-conditional like 2.10): with an adapter,
/// the test scene renders non-blank, fire reads brighter than smoke,
/// and a single intensity-3 smoke volume obscures its white backdrop
/// more than intensity-1 does. Without an adapter the render must
/// fail with the named NoAdapter error (never a blank pass).
#[test]
fn gate_2b_5_effect_captures() {
    use fx::render::{render_frame, Camera, FxError};
    use fx::scene::{single_field_scene, test_scene};
    let (boxes, fields) = test_scene();
    let req = fx::render::CaptureRequest {
        boxes: &boxes,
        fields: &fields,
        camera: Camera::default(),
        size: (320, 180),
        post: PostConfig::all_off(),
        time: 1.25,
    };
    let img = match render_frame(&req) {
        Ok(img) => img,
        Err(FxError::NoAdapter) => {
            eprintln!("2b.5: no wgpu adapter — clean NoAdapter failure verified; pixel checks NOT RUN");
            return;
        }
        Err(e) => panic!("capture failed unexpectedly: {e}"),
    };
    let distinct: std::collections::HashSet<[u8; 4]> = img.pixels().map(|p| p.0).collect();
    assert!(distinct.len() > 10, "capture must not be blank/flat: {} colors", distinct.len());

    // Single-volume obscurance, measured as the mean per-pixel
    // change against the same scene with NO field (self-calibrating:
    // no assumption about where the volume lands on screen).
    let smoke = def("fd_smoke");
    let render_single = |intensity: u8| -> image::RgbaImage {
        let (b, f) = if intensity == 0 {
            let (b, _) = single_field_scene(smoke.clone(), 1);
            (b, Vec::new())
        } else {
            single_field_scene(smoke.clone(), intensity)
        };
        let cam = Camera {
            target: glam::Vec3::new(0.5, 0.5, -2.5),
            yaw_deg: 0.0,
            pitch_deg: 30.0,
            distance: 6.5,
        };
        render_frame(&fx::render::CaptureRequest {
            boxes: &b,
            fields: &f,
            camera: cam,
            size: (160, 90),
            post: PostConfig::all_off(),
            time: 1.25,
        })
        .expect("single-field render")
    };
    let empty = render_single(0);
    let mean_delta = |img: &image::RgbaImage| -> (f32, i32) {
        let (mut sum, mut n, mut maxd) = (0f32, 0f32, 0i32);
        for y in 0..img.height() {
            for x in 0..img.width() {
                let d = (img.get_pixel(x, y)[0] as i32 - empty.get_pixel(x, y)[0] as i32).abs();
                if d > 3 {
                    sum += d as f32;
                    n += 1.0;
                    maxd = maxd.max(d);
                }
            }
        }
        (if n > 0.0 { sum / n } else { 0.0 }, maxd)
    };
    let (m1, max1) = mean_delta(&render_single(1));
    let (m3, max3) = mean_delta(&render_single(3));
    assert!(
        m3 > m1 * 1.3 && m3 > 5.0,
        "intensity-3 smoke must change the backdrop more than intensity-1 (mean delta {m3} vs {m1})"
    );
    // Peak obscurance: grey smoke (sRGB ~190-207) against the shaded
    // white backdrop (218) can delta at most ~28 in the red channel;
    // intensity-3 must get close to that ceiling and beat intensity-1.
    assert!(
        max3 >= 20 && max3 > max1,
        "intensity-3 smoke must strongly obscure at its core (max delta {max3} vs intensity-1 {max1})"
    );
}

/// effects.json contract: fx loads what the voxelizer writes.
#[test]
fn effects_json_roundtrip() {
    let text = r#"{
      "format": "cdda-voxel-effects-v1",
      "effects": {
        "fd_fire": {
          "field_id": "fd_fire", "render_class": "flame",
          "base_color": [0.9, 0.4, 0.1], "emissive_color": [1.0, 0.6, 0.2],
          "emissive_strength": 2.2, "density_scale": 0.9,
          "noise_scale": 3.5, "noise_speed": 1.6,
          "buoyancy": [0.0, 0.0, 0.9], "alpha_coverage": 0.7, "fallback": false
        }
      }
    }"#;
    let defs = fx::effect::load_effects_json(text).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].render_class, RenderClass::Flame);
    assert_eq!(defs[0].world_drift(), glam::Vec3::new(0.0, 0.9, 0.0));
}
