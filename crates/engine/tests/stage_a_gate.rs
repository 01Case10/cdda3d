//! Stage A gates (procedural texture pipeline rework —
//! docs/procedural-texture-pipeline.md §5 work order, draft §9):
//!
//! * UV checker, mesher level (CPU, exact): a greedy-merged quad
//!   of w x h tiles carries UVs spanning exactly [0,w] x [0,h]
//!   tile units — one full checker repeat per tile. This is the
//!   literal "one checker cell per tile on every face" assertion;
//!   UVs are decided in the mesher (raster) and by the in-voxel
//!   formula in the RT shader (already per-tile, cross-checked by
//!   treebuild's mip_gate CPU mirror).
//! * Internal resolution (GPU): a capture with render_scale = 2
//!   is composed at half size and integer-upscaled — the output
//!   PNG is full size and every 2x2 pixel block is uniform, in
//!   both renderers (the upscale wrapper is shared).
//! * Textured smoke (GPU): with the Stage A samplers (Nearest
//!   magnification; Repeat on raster, Repeat + ray-cone LOD
//!   tunables on RT), textured fixture captures in both renderers
//!   still render the scene richly (quads > 0, many distinct
//!   colors) — the sampler changes must not break texturing.
//!
//! GPU halves follow the established adapter-conditional pattern:
//! with no wgpu adapter they print a named NOT-RUN banner and
//! return; a blank render is never a pass.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use mesher::block::{Block, Pass};
use mesher::grid::VoxelGrid;
use mesher::mesher::mesh_greedy;
use mesher::scale::WorldScale;

static GPU_LOCK: Mutex<()> = Mutex::new(());

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------ UV checker
#[test]
fn stage_a_checker_merged_quad_uvs_are_tile_units() {
    let block = Block {
        id: 42,
        color: [0.7, 0.68, 0.62, 1.0],
        pass: Pass::Opaque,
        tex_layers: Some([10, 11, 12]),
    };
    // A 4x3 slab of one material: greedy merges each face into a
    // single quad. Every face's UV span must equal its extent in
    // tiles (4x3 top, 4x1 / 3x1 sides), anchored on integers.
    let mut grid = VoxelGrid::new([4, 3, 1]);
    grid.fill(block);
    let meshes = mesh_greedy(&grid, [0, 0, 0], grid.size, WorldScale::default());
    let mesh = &meshes.meshes[Pass::Opaque.index()];
    assert_eq!(mesh.indices.len() / 6, 6, "one quad per slab face");
    for quad in mesh.indices.chunks_exact(6) {
        let verts: Vec<usize> = quad[..4].iter().map(|&i| i as usize).collect();
        let (mut u_min, mut u_max) = (f32::MAX, f32::MIN);
        let (mut v_min, mut v_max) = (f32::MAX, f32::MIN);
        let mut p_min = [f32::MAX; 3];
        let mut p_max = [f32::MIN; 3];
        for &v in &verts {
            let [u, vv] = mesh.uvs[v];
            u_min = u_min.min(u);
            u_max = u_max.max(u);
            v_min = v_min.min(vv);
            v_max = v_max.max(vv);
            for a in 0..3 {
                p_min[a] = p_min[a].min(mesh.positions[v][a]);
                p_max[a] = p_max[a].max(mesh.positions[v][a]);
            }
        }
        let ext: Vec<f32> = (0..3)
            .map(|a| p_max[a] - p_min[a])
            .filter(|&e| e > 1e-6)
            .collect();
        assert_eq!(ext.len(), 2, "a quad face has two in-plane extents");
        let (du, dv) = (u_max - u_min, v_max - v_min);
        let matches = ((du - ext[0]).abs() < 1e-5 && (dv - ext[1]).abs() < 1e-5)
            || ((du - ext[1]).abs() < 1e-5 && (dv - ext[0]).abs() < 1e-5);
        assert!(
            matches,
            "checker must repeat once per tile: uv span ({du}, {dv}) vs tile extent {ext:?}"
        );
        assert!(
            (u_min - u_min.round()).abs() < 1e-6 && (v_min - v_min.round()).abs() < 1e-6,
            "checker anchored on tile boundaries: uv min ({u_min}, {v_min})"
        );
    }
}

// ------------------------------------------------------- GPU helpers
fn capture(args: &CaptureArgs) -> Option<image::RgbaImage> {
    match run_capture(args) {
        Ok(outcome) => {
            assert!(outcome.quads > 0, "blank render is not a pass");
            Some(image::open(&outcome.out).unwrap().to_rgba8())
        }
        Err(CaptureError::NoAdapter) => {
            eprintln!(
                "STAGE-A GATE NOT RUN: no wgpu adapter (this gate proves nothing without one)"
            );
            None
        }
        Err(e) => panic!("capture failed: {e}"),
    }
}

fn base_args(renderer: RendererKind, out: &Path) -> CaptureArgs {
    CaptureArgs {
        snapshot: "stub-house".to_string(),
        renderer,
        size: (256, 144),
        out: out.to_path_buf(),
        ..CaptureArgs::default()
    }
}

// ------------------------------------------------- internal resolution
#[test]
fn stage_a_render_scale_2_integer_upscale() {
    let _guard = gpu_lock();
    let dir = std::env::temp_dir().join(format!("stage_a_scale_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let mut args = base_args(renderer, &dir.join(format!("scale2_{}.png", renderer.name())));
        args.render_scale = 2;
        let Some(img) = capture(&args) else { continue };
        assert_eq!(img.dimensions(), (256, 144), "output stays full size");
        // Integer upscale: every 2x2 block of the output is one
        // texel of the half-size composition, hence uniform.
        let mut nonuniform = 0usize;
        for by in 0..144 / 2 {
            for bx in 0..256 / 2 {
                let p = img.get_pixel(bx * 2, by * 2);
                for dy in 0..2 {
                    for dx in 0..2 {
                        if img.get_pixel(bx * 2 + dx, by * 2 + dy) != p {
                            nonuniform += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(
            nonuniform, 0,
            "{}: render_scale=2 must integer-upscale (no filtered texels)",
            renderer.name()
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// ----------------------------------------------------- textured smoke
/// Hermetic checkerboard fixture (same construction as
/// raster_tex_gate's): every fake-house material owns three
/// 8x8 two-color checker layers with per-layer unique colors.
fn write_fixture(dir: &Path) {
    use engine::rt::fake_material_ids;
    const TILE: u32 = 8;
    const IDS: [u32; 9] = [
        mesher::block::TILE_FLOOR_WOOD,
        mesher::block::TILE_WALL,
        mesher::block::TILE_DOOR,
        mesher::block::TILE_WINDOW,
        mesher::block::TILE_ROOF,
        mesher::block::TILE_GROUND,
        mesher::block::TILE_FENCE,
        mesher::block::TILE_WATER,
        mesher::block::TILE_FOLIAGE,
    ];
    let blocks = dir.join("blocks");
    std::fs::create_dir_all(&blocks).unwrap();
    let id_map = fake_material_ids();
    let layer_count = IDS.len() * 3;
    std::fs::write(
        dir.join("blocks.json"),
        format!(r#"{{"format":"png-layers","tile_size":{TILE},"layer_count":{layer_count}}}"#),
    )
    .unwrap();
    let mut mats = String::from("[");
    for (k, num_id) in IDS.iter().enumerate() {
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
    for li in 0..layer_count {
        let c1 = image::Rgba([
            (li as u32 * 37 % 256) as u8,
            (li as u32 * 91 % 256) as u8,
            (li as u32 * 53 % 256) as u8,
            255,
        ]);
        let c2 = image::Rgba([255 - c1.0[0], 255 - c1.0[1], 255 - c1.0[2], 255]);
        let img =
            image::RgbaImage::from_fn(TILE, TILE, |x, y| if (x + y) % 2 == 0 { c1 } else { c2 });
        img.save(blocks.join(format!("layer_{li:04}.png"))).unwrap();
    }
}

#[test]
fn stage_a_textured_captures_stay_rich_both_renderers() {
    let _guard = gpu_lock();
    let dir = std::env::temp_dir().join(format!("stage_a_tex_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_fixture(&dir);
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let mut args = base_args(renderer, &dir.join(format!("tex_{}.png", renderer.name())));
        args.textures = Some(dir.clone());
        let Some(img) = capture(&args) else { continue };
        let distinct: std::collections::HashSet<[u8; 4]> =
            img.pixels().map(|p| p.0).collect();
        // The guard band: a flat-shaded fallback of this
        // scene has ~35 distinct colors, so >64 proves
        // textures are really being sampled with a wide
        // margin. (The threshold was 100 until 2026-11-17:
        // seating the fixture house's roof as a deck on
        // its wall plate hid the roof slab's underside
        // and lower sides from the camera, costing the
        // frame a few fixture colors: 98 on raster.)
        assert!(
            distinct.len() > 64,
            "{}: textured capture must stay rich, got {} distinct colors",
            renderer.name(),
            distinct.len()
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
