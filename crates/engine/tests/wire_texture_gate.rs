//! Phase 11 follow-up (Lane H) gate — textures for the live
//! wire world: with the voxelizer material table seated
//! (`CaptureArgs::textures`), the wire world's coarse blocks
//! (floor + standing) resolve texture layers/macros by STRING
//! id from the one real table, in both renderers — the
//! fixture textured path's shading, keyed by wire terrain ids
//! through the scene's material id map
//! (`WireScene::material_ids`, derived at world build from
//! the snapshot's presented defs + their roof targets).
//!
//! Proofs:
//!
//! 1. **Coverage (CPU).** For street_block's presented
//!    terrain defs, the production spec builder resolves a
//!    texture entry for exactly the defs the table carries —
//!    a def the table HAS but the spec leaves flat fails the
//!    gate by name (the root-cause tripwire), and the named
//!    representatives (grass, pavement, floor, wall) must all
//!    resolve. The full resolved/flat list is printed for the
//!    record.
//!
//! 2. **Pixels.** Textured renders differ from the Lane G
//!    flat renders on both renderers, with strictly more
//!    distinct colours and strictly more per-pixel detail
//!    (mean horizontal-neighbour difference — texture grain
//!    the flat palette cannot produce), with and without the
//!    furniture prop scene attached.
//!
//! 3. **The fallback is exact.** A textures dir seated on a
//!    scene WITHOUT a material id map renders byte-identical
//!    to no textures at all — the Lane G render — on both
//!    renderers. (The no-textures path itself is pinned
//!    byte-for-byte by wire_source_gate's parity.)
//!
//! The table is `assets/ultica_run1` (the real voxelizer
//! output); its absence is a LOUD skip, per the Lane F
//! convention — a skip proves nothing.

use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::wire::WireScene;
use proto::{SnapshotT, TileDefsT};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../crates/sim/scenes/{name}.toml"))
}

fn ultica_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1")
}

fn table_present() -> bool {
    ultica_dir().join("materials.json").exists()
}

fn loud_skip(what: &str) {
    eprintln!("SKIP {what}: real UltiCa table absent (assets/ultica_run1)");
    println!("SKIP {what}: real UltiCa table absent (assets/ultica_run1)");
}

fn sim_side(scene_name: &str) -> (TileDefsT, SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("sim registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("sim scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("sim world builds");
    let snap = world.snapshot(&reg);
    (reg.tiledefs().clone(), snap)
}

fn wire_args(
    scene: WireScene,
    renderer: RendererKind,
    textures: Option<PathBuf>,
    out: PathBuf,
) -> CaptureArgs {
    let mut args = CaptureArgs::default();
    args.snapshot = "wire".to_string();
    args.wire = Some(scene);
    args.renderer = renderer;
    args.textures = textures;
    args.size = (640, 360);
    args.cutaway_z = 0;
    args.out = out;
    args
}

fn capture(args: &CaptureArgs) -> Option<Vec<u8>> {
    match run_capture(args) {
        Ok(_) => Some(std::fs::read(&args.out).expect("capture png written")),
        Err(CaptureError::NoAdapter) => None,
        Err(e) => panic!("capture failed: {e}"),
    }
}

fn distinct_colors(img: &image::RgbaImage) -> usize {
    img.pixels()
        .map(|p| [p[0], p[1], p[2]])
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// Mean absolute difference between horizontally adjacent
/// pixels — a texture-grain measure: flat palette regions
/// score ~0 except at geometry edges; a textured surface
/// scores throughout its area.
fn adjacency_detail(img: &image::RgbaImage) -> f32 {
    let (w, h) = (img.width(), img.height());
    let mut sum = 0f64;
    let mut n = 0u64;
    for y in 0..h {
        for x in 0..w - 1 {
            let a = img.get_pixel(x, y);
            let b = img.get_pixel(x + 1, y);
            sum += (a[0] as i32 - b[0] as i32).abs() as f64
                + (a[1] as i32 - b[1] as i32).abs() as f64
                + (a[2] as i32 - b[2] as i32).abs() as f64;
            n += 1;
        }
    }
    (sum / n.max(1) as f64) as f32
}

// ---------------------------------------------------------------------
// 1: coverage — the spec resolves exactly what the table carries.
// ---------------------------------------------------------------------

#[test]
fn wire_texture_coverage() {
    if !table_present() {
        loud_skip("wire_texture_coverage");
        return;
    }
    let (defs, snap) = sim_side("street_block");
    let scene = WireScene::from_wire(&defs, &snap, &engine::wire::palette_resolver(&defs));
    let ids = scene
        .material_ids
        .as_ref()
        .expect("from_wire derives the material id map");
    assert!(!ids.is_empty(), "street_block presents terrain defs");
    let table = engine::materials::MaterialTable::load(&ultica_dir())
        .expect("assets/ultica_run1 material table loads");
    let (spec, stats) =
        engine::textures::build_texture_spec(&table, &ultica_dir(), ids).expect("spec builds");

    // The named representatives must all resolve.
    for named in ["t_grass", "t_pavement", "t_floor", "t_wall"] {
        let (wid, _) = ids
            .iter()
            .find(|(_, s)| s.as_str() == named)
            .unwrap_or_else(|| panic!("{named} is presented by street_block"));
        let entry = spec.material_layers[*wid as usize];
        assert_eq!(
            entry[3], 1,
            "{named} (wire id {wid}) must resolve texture layers, got {entry:?}"
        );
    }
    // The tripwire: resolution == table membership, per def.
    let mut resolved = Vec::new();
    let mut flat = Vec::new();
    for (wid, sid) in ids {
        let in_table = table.get(sid).is_some();
        let entry = spec.material_layers[*wid as usize];
        assert_eq!(
            entry[3] == 1,
            in_table,
            "{sid} (wire id {wid}): table membership {in_table} but spec present flag {}",
            entry[3]
        );
        if entry[3] == 1 {
            resolved.push(sid.as_str());
        } else {
            flat.push(sid.as_str());
        }
    }
    println!(
        "street_block texture coverage: {}/{} presented defs textured (stats: {} textured, {} missing); flat: {flat:?}",
        resolved.len(),
        resolved.len() + flat.len(),
        stats.textured_materials,
        stats.missing_materials,
    );
    assert_eq!(
        stats.textured_materials, resolved.len(),
        "spec stats agree with the per-def enumeration"
    );
    assert!(
        resolved.len() >= 4,
        "at least the named representatives resolve"
    );
}

// ---------------------------------------------------------------------
// 2 + 3: pixels, and the exact fallback.
// ---------------------------------------------------------------------

#[test]
fn wire_textured_renders() {
    if !table_present() {
        loud_skip("wire_textured_renders");
        return;
    }
    let (defs, snap) = sim_side("street_block");
    let dir = std::env::temp_dir().join(format!("wire-tex-gate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let table = engine::materials::MaterialTable::load(&ultica_dir())
        .expect("assets/ultica_run1 material table loads");

    let base_scene = WireScene::from_wire(&defs, &snap, &engine::wire::palette_resolver(&defs));
    let props_scene = WireScene::from_wire(&defs, &snap, &engine::wire::palette_resolver(&defs))
        .with_props(engine::wire::prop_scene_for_wire(&defs, &snap, &table));
    assert!(
        props_scene.props.is_some(),
        "street_block furniture derives a prop scene"
    );

    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        for (label, scene) in [("coarse", &base_scene), ("props", &props_scene)] {
            let flat_png = dir.join(format!("{}-{}-flat.png", renderer.name(), label));
            let tex_png = dir.join(format!("{}-{}-tex.png", renderer.name(), label));
            let Some(flat_bytes) = capture(&wire_args(
                scene.clone(),
                renderer,
                None,
                flat_png.clone(),
            )) else {
                loud_skip("wire_textured_renders");
                return;
            };
            let tex_bytes = capture(&wire_args(
                scene.clone(),
                renderer,
                Some(ultica_dir()),
                tex_png.clone(),
            ))
            .expect("adapter vanished between captures");
            assert_ne!(
                flat_bytes, tex_bytes,
                "{} {label}: textured render is byte-identical to the flat render — \
                 the table is not reaching the wire world",
                renderer.name()
            );
            let flat_img = image::open(&flat_png).expect("flat png").to_rgba8();
            let tex_img = image::open(&tex_png).expect("tex png").to_rgba8();
            let (fd, td) = (distinct_colors(&flat_img), distinct_colors(&tex_img));
            let (fg, tg) = (adjacency_detail(&flat_img), adjacency_detail(&tex_img));
            println!(
                "{} {label}: distinct colours flat {fd} -> textured {td}; \
                 adjacency detail flat {fg:.3} -> textured {tg:.3}",
                renderer.name()
            );
            assert!(
                td > fd,
                "{} {label}: textured render must add colours ({td} <= {fd})",
                renderer.name()
            );
            assert!(
                tg > fg,
                "{} {label}: textured render must add per-pixel detail ({tg:.3} <= {fg:.3})",
                renderer.name()
            );
        }

        // The exact fallback: textures seated, no id map.
        let mut no_ids = base_scene.clone();
        no_ids.material_ids = None;
        let fb_png = dir.join(format!("{}-fallback.png", renderer.name()));
        let flat_png = dir.join(format!("{}-coarse-flat.png", renderer.name()));
        let fb_bytes = capture(&wire_args(
            no_ids,
            renderer,
            Some(ultica_dir()),
            fb_png,
        ))
        .expect("adapter vanished between captures");
        let flat_bytes = std::fs::read(&flat_png).expect("flat capture exists");
        assert_eq!(
            fb_bytes, flat_bytes,
            "{}: textures dir without a material id map must render exactly the Lane G flat world",
            renderer.name()
        );
        println!(
            "{}: fallback exact (textures seated, no id map == no textures)",
            renderer.name()
        );
    }
}
