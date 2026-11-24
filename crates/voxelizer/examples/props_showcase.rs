//! Stage G showcase: contact sheet of inferred 3D prop models for
//! sign-off. One row per furniture id, ONE model per prop (the
//! solid model), columns:
//!   [original sprite | FRONT | BACK | LEFT | RIGHT]
//! The four views are orthographic azimuths around the model
//! (front = the sprite's face, +z; back = -z; left = the sprite's
//! left flank, -x; right = +x), each with a slight elevation
//! (~33 degrees) so the top surface stays visible. The sheet has
//! no text labels (the pipeline has no font renderer) — the column
//! order above is fixed and explained in chat.
//! Models are drawn with a tiny software isometric cube renderer
//! (surface voxels only, painter-sorted, three shaded faces).
//! Output: target/stage-g/props.png (relative to the worktree root).

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

const IDS: [&str; 5] = ["f_counter", "f_bookcase", "f_table", "f_fridge", "f_locker"];
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
/// `view` points from the model toward the camera, so the face it
/// names is the one whose outward normal aligns with it: FRONT
/// (0,e,1), BACK (0,e,-1), LEFT (-1,e,0), RIGHT (1,e,0), with e the
/// elevation component.
fn render_model(sheet: &mut RgbaImage, model: &PropModel, view: V3, ox: u32, oy: u32) {
    let [w, h, d] = model.size;
    let v = view.norm();
    // Screen basis: right = up x view, up2 = view x right (a true
    // right-handed camera basis — an earlier version used
    // view x up for `right`, which mirrored every render
    // horizontally against the sprite column).
    let right = V3(0.0, 1.0, 0.0).cross(v).norm();
    let up2 = v.cross(right).norm();
    let center = V3(w as f32 / 2.0, h as f32 / 2.0, d as f32 / 2.0);
    // Fit: projected extent of the bounding box, sub-cell = 3px max.
    let mut extent = 0.0f32;
    for &cx in &[0.0, w as f32] {
        for &cy in &[0.0, h as f32] {
            for &cz in &[0.0, d as f32] {
                let p = V3(cx, cy, cz).add(center.scale(-1.0));
                extent = extent.max(p.dot(right).abs()).max(p.dot(up2).abs());
            }
        }
    }
    let s = ((CELL_W.min(CELL_H) as f32 / 2.0 - 12.0) / extent.max(1.0)).min(3.0);
    let project = |p: V3| -> (f32, f32) {
        let q = p.add(center.scale(-1.0));
        (
            ox as f32 + CELL_W as f32 / 2.0 + q.dot(right) * s,
            oy as f32 + CELL_H as f32 / 2.0 - q.dot(up2) * s,
        )
    };
    // Surface voxels, far-to-near by depth along the view vector.
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
    // Face templates: (normal, corner offsets, shade factor).
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
                continue; // back-facing
            }
            // Skip faces buried against a solid neighbor.
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

fn main() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let mut sheet = RgbaImage::from_pixel(CELL_W * 5, CELL_H * IDS.len() as u32, BACKDROP);
    for (row, id) in IDS.iter().enumerate() {
        let entry = ts.entries.get(*id).unwrap_or_else(|| panic!("tileset has {id}"));
        let sprite = composite_entry(&ts, entry).unwrap_or_else(|| panic!("{id} composites"));
        let seg = segment_for(id, &sprite)
            .unwrap_or_else(|| panic!("{id} must segment for the showcase"));
        println!(
            "{id}: boundary {}, top {:?}, front {:?}, side {:?}, confidence {:.2}",
            seg.boundary(),
            seg.top_rows,
            seg.front_rows,
            seg.side,
            seg.confidence
        );
        let oy = row as u32 * CELL_H;
        // Column 0: the original sprite, 2x nearest, bottom-anchored.
        let big = image::imageops::resize(
            &sprite,
            sprite.width() * 2,
            sprite.height() * 2,
            image::imageops::FilterType::Nearest,
        );
        let sx = (CELL_W - big.width()) / 2;
        let sy = oy + CELL_H - big.height() - 10;
        image::imageops::overlay(&mut sheet, &big, sx as i64, sy as i64);
        let sm = solid_model_for(id, &sprite, &seg);
        // Azimuth views at ~33 degrees elevation (0.65 over a unit
        // horizontal), matching the old 3/4 views' tilt.
        render_model(&mut sheet, &sm, V3(0.0, 0.65, 1.0), CELL_W, oy); // FRONT
        render_model(&mut sheet, &sm, V3(0.0, 0.65, -1.0), CELL_W * 2, oy); // BACK
        render_model(&mut sheet, &sm, V3(-1.0, 0.65, 0.0), CELL_W * 3, oy); // LEFT
        render_model(&mut sheet, &sm, V3(1.0, 0.65, 0.0), CELL_W * 4, oy); // RIGHT
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("../../target/stage-g/props.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target/stage-g");
    }
    sheet.save(&out).expect("save sheet");
    println!("wrote {}", out.display());
}
