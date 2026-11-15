//! Phase 2c engine-level gates (plan Part II §5c): 2c.4 render
//! parity and 2c.6 cutaway parity, both adapter-conditional in the
//! established pattern — with an adapter (lavapipe counts) they run
//! for real; the data-level halves (tree leaf counts per cutaway)
//! run everywhere.

use engine::capture::{run_capture, CaptureArgs, RendererKind};
use engine::rt::{capture_rt_standalone, capture_visible_levels, world_for_levels};
use treebuild::build::{build_tree, BuildConfig};

fn args(renderer: RendererKind, cutaway_z: i32, out: &str) -> CaptureArgs {
    CaptureArgs {
        snapshot: "stub-house".to_string(),
        renderer,
        textures: None,
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        size: (320, 180),
        out: out.into(),
        cutaway_z,
        ..CaptureArgs::default()
    }
}

/// The raster capture's background is its clear color; geometry is
/// any pixel that differs from the image's most common color.
fn raster_silhouette(img: &image::RgbaImage) -> Vec<bool> {
    let mut counts: std::collections::HashMap<[u8; 4], usize> = std::collections::HashMap::new();
    for p in img.pixels() {
        *counts.entry(p.0).or_default() += 1;
    }
    let bg = counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .map(|(c, _)| *c)
        .unwrap_or([0, 0, 0, 0]);
    img.pixels().map(|p| p.0 != bg).collect()
}

fn iou(a: &[bool], b: &[bool]) -> f32 {
    let mut inter = 0usize;
    let mut union = 0usize;
    for (x, y) in a.iter().zip(b.iter()) {
        if *x && *y {
            inter += 1;
        }
        if *x || *y {
            union += 1;
        }
    }
    if union == 0 {
        1.0
    } else {
        inter as f32 / union as f32
    }
}

// ---------------------------------------------------------------- 2c.6
/// Cutaway parity at data level (runs everywhere): the tree built
/// for a cutaway contains exactly the levels the raster path would
/// mesh — same `filtered_map`, so this pins the wiring.
#[test]
fn gate_2c_6_cutaway_tree_levels() {
    // Full house (cutaway z=1: levels -10..=1 visible, house is 0+1).
    let full = build_tree(
        &world_for_levels(&capture_visible_levels(1)),
        BuildConfig::default(),
    )
    .unwrap();
    // Cutaway z=0 drops the roof level (game z 1).
    let cut = build_tree(
        &world_for_levels(&capture_visible_levels(0)),
        BuildConfig::default(),
    )
    .unwrap();
    assert!(
        cut.leaf_count() < full.leaf_count(),
        "cutaway must remove the roof level from the tree"
    );
    assert_eq!(
        full.leaf_count() - cut.leaf_count(),
        12 * 12,
        "the difference is exactly the 12x12 roof-slab layer"
    );
    // Extremes (gate 2.8's z=-10/+10): at -10 nothing of the house is
    // visible -> empty tree, no panic; at +10 everything is visible.
    let bottom = build_tree(
        &world_for_levels(&capture_visible_levels(-10)),
        BuildConfig::default(),
    )
    .unwrap();
    assert_eq!(bottom.leaf_count(), 0);
    assert_eq!(bottom.node_count(), 1, "empty world = root only");
    let top = build_tree(
        &world_for_levels(&capture_visible_levels(10)),
        BuildConfig::default(),
    )
    .unwrap();
    assert_eq!(top.leaf_count(), full.leaf_count());
}

// ------------------------------------------------------------- 2c.4/6
/// Render parity (adapter-conditional): RT vs raster captures of
/// the fake house from the same camera must show the same geometry
/// (silhouette IoU), with differences confined to lighting; and the
/// RT cutaway capture must lose the roof silhouette.
#[test]
fn gate_2c_4_and_2c_6_render_parity() {
    let dir = std::env::temp_dir().join("cdda-voxel-rt-gate");
    std::fs::create_dir_all(&dir).unwrap();
    let raster_path = dir.join("raster.png");
    let raster_args = args(RendererKind::Raster, 1, raster_path.to_str().unwrap());
    let raster = match run_capture(&raster_args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("2c.4: raster capture unavailable ({e}) — GPU checks NOT RUN");
            return;
        }
    };
    assert_eq!(raster.size, (320, 180));
    let raster_img = image::open(&raster_path).unwrap().to_rgba8();
    let raster_sil = raster_silhouette(&raster_img);

    let rt_args = args(RendererKind::Rt, 1, "unused.png");
    let rt = capture_rt_standalone(&rt_args).expect("RT capture");
    assert!(rt.leaves > 0 && rt.distinct_colors > 10, "RT capture must not be blank");
    let rt_sil: Vec<bool> = rt.depth.iter().map(|&d| d < rt.max_dist).collect();
    let score = iou(&raster_sil, &rt_sil);
    // Itemize the disagreement for the comparison report.
    let rt_only = raster_sil.iter().zip(rt_sil.iter()).filter(|(a, b)| !**a && **b).count();
    let raster_only = raster_sil.iter().zip(rt_sil.iter()).filter(|(a, b)| **a && !**b).count();
    // Mean color difference over pixels both call geometry: the
    // lighting delta (shadows/AO/lit) the gate allows.
    let mut diff_sum = 0f64;
    let mut both = 0usize;
    for (i, (a, b)) in raster_sil.iter().zip(rt_sil.iter()).enumerate() {
        if *a && *b {
            both += 1;
            let rp = raster_img.get_pixel((i as u32) % 320, (i as u32) / 320);
            let tp = rt.image.get_pixel((i as u32) % 320, (i as u32) / 320);
            for c in 0..3 {
                diff_sum += (rp.0[c] as f64 - tp.0[c] as f64).abs();
            }
        }
    }
    println!(
        "2c.4 parity: silhouette IoU {score:.3}; rt-only {rt_only}px, raster-only {raster_only}px; \
         mean |color diff| on shared geometry {:.1}/255 (lighting delta: sun shadow + AO + lit factor)",
        if both > 0 { diff_sum / (both * 3) as f64 } else { 0.0 }
    );
    assert!(
        score > 0.85,
        "RT and raster silhouettes must agree (same visible geometry), IoU {score:.3}"
    );

    // 2c.6 render half: cutaway z=0 removes the roof — the RT
    // silhouette must shrink, and specifically lose pixels the full
    // view has.
    let rt_cut = capture_rt_standalone(&args(RendererKind::Rt, 0, "unused.png")).unwrap();
    let cut_sil: Vec<bool> = rt_cut.depth.iter().map(|&d| d < rt_cut.max_dist).collect();
    let full_hits = rt_sil.iter().filter(|s| **s).count();
    let cut_hits = cut_sil.iter().filter(|s| **s).count();
    assert!(
        cut_hits < full_hits,
        "cutaway RT capture must show less geometry ({cut_hits} !< {full_hits})"
    );
    // And the raster path agrees on the direction of the change.
    let raster_cut_path = dir.join("raster_cut.png");
    let raster_cut = run_capture(&args(RendererKind::Raster, 0, raster_cut_path.to_str().unwrap()))
        .expect("raster cutaway capture");
    assert!(raster_cut.quads < raster.quads);
}

// ------------------------------------------------------- texture hookup
/// Phase 5 texture hookup on the RT path: with `--textures` pointing
/// at the real UltiCa voxelizer output, the fake house's materials
/// resolve, the traced geometry (depth) is bit-identical to the
/// flat render (textures shade, they don't move geometry), and the
/// colors actually change.
#[test]
fn gate_texture_hookup_ultica_fake_house() {
    let tex_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1");
    if !tex_dir.join("materials.json").exists() {
        eprintln!("texture hookup: {} not present — NOT RUN", tex_dir.display());
        return;
    }
    let flat = match capture_rt_standalone(&args(RendererKind::Rt, 1, "unused.png")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("texture hookup: RT capture unavailable ({e}) — NOT RUN");
            return;
        }
    };
    let mut tex_args = args(RendererKind::Rt, 1, "unused.png");
    tex_args.textures = Some(tex_dir);
    let tex = capture_rt_standalone(&tex_args).expect("textured RT capture");
    let stats = tex.texture_stats.expect("texture stats reported");
    println!(
        "texture hookup: {} materials textured, {} missing, {} layers uploaded",
        stats.textured_materials, stats.missing_materials, stats.layers_uploaded
    );
    assert!(
        stats.textured_materials >= 6,
        "all six fake-house materials must resolve (got {})",
        stats.textured_materials
    );
    assert!(stats.layers_uploaded >= 6);
    // Geometry is untouched by shading: depths bit-identical.
    assert_eq!(flat.depth.len(), tex.depth.len());
    assert!(
        flat.depth.iter().zip(tex.depth.iter()).all(|(a, b)| a == b),
        "textured and flat RT captures must trace identical geometry"
    );
    // And the picture changes: substantial mean color delta on
    // geometry pixels, more distinct colors than the flat palette.
    let mut diff = 0f64;
    let mut n = 0usize;
    for (i, &d) in flat.depth.iter().enumerate() {
        if d < flat.max_dist {
            n += 1;
            let fp = flat.image.get_pixel((i as u32) % 320, (i as u32) / 320);
            let tp = tex.image.get_pixel((i as u32) % 320, (i as u32) / 320);
            for c in 0..3 {
                diff += (fp.0[c] as f64 - tp.0[c] as f64).abs();
            }
        }
    }
    let mean = diff / (n * 3) as f64;
    println!("texture hookup: mean |color diff| on geometry {mean:.1}/255; distinct colors flat {} -> textured {}", flat.distinct_colors, tex.distinct_colors);
    assert!(mean > 5.0, "textures must visibly change the render (mean diff {mean:.1})");
    assert!(
        tex.distinct_colors > flat.distinct_colors,
        "textured render must have richer color variation"
    );
}
