//! Stage G coverage-drive showcase: contact sheet of the NEW
//! base prop models (sprites segment() declines, modeled by the
//! coverage fallback in props::coverage_model). One row per
//! representative id, columns:
//!   [original sprite | FRONT | BACK | LEFT | RIGHT]
//! Row order (fixed, explained in the drive report):
//!   1. f_toilet       seamless solid (base-treated; G.4 archetype drop stands)
//!   2. f_chest        seamless solid
//!   3. f_workbench    seamless solid
//!   4. f_wood_keg     seamless solid, short silhouette -> thin depth
//!   5. f_gunsafe_c    seamless solid, 64x64 sprite
//!   6. f_sand_castle  seamless solid (view call reviewed on this sheet)
//!   7. f_groundsheet  plan slab, thickness 2
//!   8. f_down_mattress plan slab, thickness 6
//!   9. f_firering     plan slab, ring keeps its hole
//!  10. f_canvas_door  panel slab, thickness 3
//!  11. f_grave_stone  panel slab, thickness 3
//!  12. f_bigmirror    panel slab, thickness 3
//! Renderer copied from examples/archetypes_showcase.rs.
//! Output: target/stage-g/prop_coverage.png (worktree root).

use image::{Rgba, RgbaImage};
use std::path::PathBuf;
use voxelizer::*;

fn ultica_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest.join("../../../cdda-voxel/fork-src/tilesets/gfx/UltimateCataclysm"),
        manifest.join("../../fork-src/tilesets/gfx/UltimateCataclysm"),
    ];
    for c in &candidates {
        if c.join("tile_info.json").exists() {
            return c.clone();
        }
    }
    panic!("UltiCa tileset not found; tried {candidates:?}");
}

/// Sprite ids, one row each (see header for the order).
const ROWS: [&str; 12] = [
    "f_toilet",
    "f_chest",
    "f_workbench",
    "f_wood_keg",
    "f_gunsafe_c",
    "f_sand_castle",
    "f_groundsheet",
    "f_down_mattress",
    "f_firering",
    "f_canvas_door",
    "f_grave_stone",
    "f_bigmirror",
];
const CELL_W: u32 = 340;
const CELL_H: u32 = 300;
const BACKDROP: Rgba<u8> = Rgba([24, 24, 28, 255]);

fn shade(px: Rgba<u8>, f: f32) -> Rgba<u8> {
    let m = |v: u8| (v as f32 * f).round().clamp(0.0, 255.0) as u8;
    Rgba([m(px[0]), m(px[1]), m(px[2]), 255])
}

// --- tiny vec3 helpers for the iso projection ----------------------
#[derive(Clone, Copy)]
struct V3(f32, f32, f32);
impl V3 {
    fn add(self, o: V3) -> V3 {
        V3(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
    fn scale(self, s: f32) -> V3 {
        V3(self.0 * s, self.1 * s, self.2 * s)
    }
    fn dot(self, o: V3) -> f32 {
        self.0 * o.0 + self.1 * o.1 + self.2 * o.2
    }
    fn cross(self, o: V3) -> V3 {
        V3(
            self.1 * o.2 - self.2 * o.1,
            self.2 * o.0 - self.0 * o.2,
            self.0 * o.1 - self.1 * o.0,
        )
    }
    fn norm(self) -> V3 {
        let l = self.dot(self).sqrt().max(1e-6);
        self.scale(1.0 / l)
    }
}

/// Fill a convex quad (screen-space points) into `img`, clipped to
/// the image. Pixel-center point-in-quad via consistent cross signs.
fn fill_quad(img: &mut RgbaImage, pts: [(f32, f32); 4], color: Rgba<u8>) {
    let (mut x0, mut y0) = (f32::MAX, f32::MAX);
    let (mut x1, mut y1) = (f32::MIN, f32::MIN);
    for &(x, y) in &pts {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    let (ix0, iy0) = ((x0.floor() as i32).max(0), (y0.floor() as i32).max(0));
    let (ix1, iy1) = (
        (x1.ceil() as i32).min(img.width() as i32 - 1),
        (y1.ceil() as i32).min(img.height() as i32 - 1),
    );
    for py in iy0..=iy1 {
        for px in ix0..=ix1 {
            let (cx, cy) = (px as f32 + 0.5, py as f32 + 0.5);
            let mut sign = 0.0f32;
            let mut inside = true;
            for i in 0..4 {
                let a = pts[i];
                let b = pts[(i + 1) % 4];
                let cr = (b.0 - a.0) * (cy - a.1) - (b.1 - a.1) * (cx - a.0);
                if i == 0 {
                    sign = cr.signum();
                } else if cr.signum() != sign && cr.abs() > 1e-4 {
                    inside = false;
                    break;
                }
            }
            if inside {
                img.put_pixel(px as u32, py as u32, color);
            }
        }
    }
}

/// Render a prop model into `sheet` at cell origin (ox, oy).
fn render_model(sheet: &mut RgbaImage, model: &PropModel, view: V3, ox: u32, oy: u32, cell_h: u32) {
    let [w, h, d] = model.size;
    let v = view.norm();
    let right = V3(0.0, 1.0, 0.0).cross(v).norm();
    let up2 = v.cross(right).norm();
    let center = V3(w as f32 / 2.0, h as f32 / 2.0, d as f32 / 2.0);
    let mut extent = 0.0f32;
    for &cx in &[0.0, w as f32] {
        for &cy in &[0.0, h as f32] {
            for &cz in &[0.0, d as f32] {
                let p = V3(cx, cy, cz).add(center.scale(-1.0));
                extent = extent.max(p.dot(right).abs()).max(p.dot(up2).abs());
            }
        }
    }
    // Fixed scale for every terrain strip: fit-to-cell
    // shrank tall models (a 66-tall tree rendered its
    // crown at half width) and made cross-model
    // comparison meaningless. The strip's cell height
    // is sized to the model instead (see main).
    let _ = extent;
    let s = 4.0f32;
    let project = |p: V3| -> (f32, f32) {
        let q = p.add(center.scale(-1.0));
        (
            ox as f32 + CELL_W as f32 / 2.0 + q.dot(right) * s,
            oy as f32 + cell_h as f32 / 2.0 - q.dot(up2) * s,
        )
    };
    let mut cells: Vec<(f32, u32, u32, u32, Rgba<u8>)> = Vec::new();
    let neigh = [
        (1i32, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                let Some(c) = model.get(x, y, z) else { continue };
                let surface = neigh.iter().any(|&(dx, dy, dz)| {
                    let (nx, ny, nz) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
                    nx < 0
                        || ny < 0
                        || nz < 0
                        || nx >= w as i32
                        || ny >= h as i32
                        || nz >= d as i32
                        || model.get(nx as u32, ny as u32, nz as u32).is_none()
                });
                if surface {
                    let depth = V3(x as f32, y as f32, z as f32).dot(v);
                    cells.push((depth, x, y, z, c));
                }
            }
        }
    }
    cells.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let faces: [(V3, [(f32, f32, f32); 4], f32); 6] = [
        (V3(0.0, 1.0, 0.0), [(0., 1., 0.), (1., 1., 0.), (1., 1., 1.), (0., 1., 1.)], 1.0),
        (V3(0.0, -1.0, 0.0), [(0., 0., 0.), (0., 0., 1.), (1., 0., 1.), (1., 0., 0.)], 0.45),
        (V3(0.0, 0.0, 1.0), [(0., 0., 1.), (1., 0., 1.), (1., 1., 1.), (0., 1., 1.)], 0.82),
        (V3(0.0, 0.0, -1.0), [(0., 0., 0.), (0., 1., 0.), (1., 1., 0.), (1., 0., 0.)], 0.82),
        (V3(1.0, 0.0, 0.0), [(1., 0., 0.), (1., 0., 1.), (1., 1., 1.), (1., 1., 0.)], 0.64),
        (V3(-1.0, 0.0, 0.0), [(0., 0., 0.), (0., 1., 0.), (0., 1., 1.), (0., 0., 1.)], 0.64),
    ];
    for &(_, x, y, z, c) in &cells {
        let base = V3(x as f32, y as f32, z as f32);
        for (n, corners, f) in &faces {
            if n.dot(v) <= 0.05 {
                continue;
            }
            let (nx, ny, nz) = (
                x as i32 + n.0 as i32,
                y as i32 + n.1 as i32,
                z as i32 + n.2 as i32,
            );
            if nx >= 0 && ny >= 0 && nz >= 0 && nx < w as i32 && ny < h as i32 && nz < d as i32
                && model.get(nx as u32, ny as u32, nz as u32).is_some()
            {
                continue;
            }
            let pts: [(f32, f32); 4] = corners.map(|(cx, cy, cz)| {
                project(base.add(V3(cx, cy, cz)))
            });
            fill_quad(sheet, pts, shade(c, *f));
        }
    }
}

/// Rotate a model one quarter turn clockwise seen from
/// above: (x, z) -> (z, W-1-x), footprint [W,D] -> [D,W].
fn rot90(m: &PropModel) -> PropModel {
    let [w, h, d] = m.size;
    let mut out = PropModel::new([d, h, w]);
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                if let Some(c) = m.get(x, y, z) {
                    out.set(z, y, w - 1 - x, Some(c));
                }
            }
        }
    }
    out
}

/// Blit `src` into `dst` at a cell offset.
fn blit(dst: &mut PropModel, src: &PropModel, ox: u32, oy: u32, oz: u32) {
    let [w, h, d] = src.size;
    for y in 0..h {
        for z in 0..d {
            for x in 0..w {
                if let Some(c) = src.get(x, y, z) {
                    dst.set(ox + x, oy + y, oz + z, Some(c));
                }
            }
        }
    }
}

fn main() {
    let ids: Vec<String> = std::env::args().skip(1).collect();
    assert!(!ids.is_empty(), "usage: terrain_strips <id> [id...]");
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = manifest.join("../../target/stage-g/strips-terrain");
    std::fs::create_dir_all(&out_dir).expect("create strips dir");
    for id in &ids {
        let Some(entry) = ts.entries.get(id) else {
            println!("{id}: no tileset entry");
            continue;
        };
        // FG alone, matching the emission path (the bg
        // composite would corrupt silhouette-built models).
        let fg = if id == "t_chainfence" {
            entry
                .fg
                .as_ref()
                .and_then(|r| voxelizer::load_sprite_image(&ts, entry, r))
        } else {
            voxelizer::terrain_fg_image(&ts, entry)
        };
        let Some(sprite) = fg.or_else(|| composite_entry(&ts, entry)) else {
            println!("{id}: no sprite");
            continue;
        };
        let underlay = if id.starts_with("t_puddle") {
            entry
                .bg
                .as_ref()
                .and_then(|r| voxelizer::load_sprite_image(&ts, entry, r))
        } else {
            None
        };
        let Some((base, _kind, _surface)) =
            voxelizer::props::terrain_model_for(id, &sprite, underlay.as_ref())
        else {
            println!("{id}: no terrain model");
            continue;
        };
        // Tiling compositions for the fence, built from
        // the full-frame half-segment the way the engine
        // will stamp it: a straight tile = W half + E
        // half (180 turn); a run = 3 straight tiles; a
        // corner = E half + S half (270 turn) meeting at
        // the centre post.
        let east_half = rot90(&rot90(&base));
        let model = if id == "t_chainfence" && std::env::var("TILE_RUN").is_ok() {
            let mut run = PropModel::new([96, 28, 32]);
            for i in 0..3 {
                blit(&mut run, &base, i * 32, 0, 0);
                blit(&mut run, &east_half, i * 32, 0, 0);
            }
            run
        } else if id == "t_chainfence" && std::env::var("TILE_CORNER").is_ok() {
            let mut corner = PropModel::new([32, 28, 32]);
            blit(&mut corner, &east_half, 0, 0, 0);
            let south_half = rot90(&east_half);
            blit(&mut corner, &south_half, 0, 0, 0);
            corner
        } else if id == "t_chainfence" {
            // The straight tile as stamped (W + E halves).
            let mut tile = PropModel::new([32, 28, 32]);
            blit(&mut tile, &base, 0, 0, 0);
            blit(&mut tile, &east_half, 0, 0, 0);
            tile
        } else {
            base
        };
        println!("{id}: model {:?} ({} cells)", model.size, model.voxels.iter().filter(|v| v.is_some()).count());
        sprite.save(out_dir.join(format!("{id}_fg.png"))).expect("save fg");
        // Cell height sized to the model at the fixed
        // render scale: projected vertical extent is
        // roughly (0.92 * height + 0.55 * depth) cells.
        let cell_h = ((model.size[1] as f32 * 0.92 + model.size[2] as f32 * 0.55) * 4.0 + 30.0) as u32;
        let mut strip = RgbaImage::from_pixel(CELL_W * 5, cell_h, BACKDROP);
        // Fit the sprite thumbnail inside the cell:
        // upscale x2 when it fits, shrink when the
        // sprite is taller than the model's cell (a
        // 64px sprite over a height-10 ground slab
        // used to underflow the y offset and panic).
        let max_w = CELL_W - 16;
        let max_h = cell_h.saturating_sub(16).max(16);
        let k = ((max_w as f32 / sprite.width() as f32)
            .min(max_h as f32 / sprite.height() as f32))
        .min(2.0);
        let big = image::imageops::resize(
            &sprite,
            ((sprite.width() as f32 * k) as u32).max(1),
            ((sprite.height() as f32 * k) as u32).max(1),
            image::imageops::FilterType::Nearest,
        );
        let sx = (CELL_W - big.width()) / 2;
        let sy = cell_h - big.height() - 10;
        image::imageops::overlay(&mut strip, &big, sx as i64, sy as i64);
        render_model(&mut strip, &model, V3(0.0, 0.65, 1.0), CELL_W, 0, cell_h);
        render_model(&mut strip, &model, V3(0.0, 0.65, -1.0), CELL_W * 2, 0, cell_h);
        render_model(&mut strip, &model, V3(-1.0, 0.65, 0.0), CELL_W * 3, 0, cell_h);
        render_model(&mut strip, &model, V3(1.0, 0.65, 0.0), CELL_W * 4, 0, cell_h);
        strip.save(out_dir.join(format!("{id}.png"))).expect("save strip");
        println!("{id}: strip written");
    }
}
