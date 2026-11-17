//! Stage G.3 rework showcase: contact sheet of the
//! view-faithful prop models (pipeline §24), built through
//! props::model_for. One row per prop, columns:
//!   [original sprite | FRONT | BACK | LEFT | RIGHT]
//! The four views are orthographic azimuths around the model
//! (front = +z, the end cap for the tank / the south elevation
//! for the top-down props; back = -z; left = -x; right = +x),
//! each with a slight elevation (~33 degrees). No text labels
//! (the pipeline has no font renderer) — the row order is fixed
//! and explained in chat:
//!   1. f_standing_tank (EndOn: horizontal cylinder)
//!   2. f_console       (Front: reworked ConsoleDesk)
//!   3. f_slab          (TopDown: terrace)
//!   4. f_bunkbed       (TopDown: framed surface; sprite f_bed)
//! Renderer identical to examples/archetypes_showcase.rs.
//! Output: target/stage-g/rework.png (worktree root).

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

/// (sprite id, served furniture id) per row.
const ROWS: [(&str, &str); 4] = [
    ("f_standing_tank", "f_standing_tank"),
    ("f_console", "f_console"),
    ("f_slab", "f_slab"),
    ("f_bed", "f_bunkbed"),
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
fn render_model(sheet: &mut RgbaImage, model: &PropModel, view: V3, ox: u32, oy: u32) {
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
    let s = ((CELL_W.min(CELL_H) as f32 / 2.0 - 12.0) / extent.max(1.0)).min(3.0);
    let project = |p: V3| -> (f32, f32) {
        let q = p.add(center.scale(-1.0));
        (
            ox as f32 + CELL_W as f32 / 2.0 + q.dot(right) * s,
            oy as f32 + CELL_H as f32 / 2.0 - q.dot(up2) * s,
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

fn main() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let mut sheet = RgbaImage::from_pixel(CELL_W * 5, CELL_H * ROWS.len() as u32, BACKDROP);
    for (row, (sprite_id, served_id)) in ROWS.iter().enumerate() {
        let entry = ts
            .entries
            .get(*sprite_id)
            .unwrap_or_else(|| panic!("tileset has {sprite_id}"));
        let sprite = composite_entry(&ts, entry)
            .unwrap_or_else(|| panic!("{sprite_id} composites"));
        let view = view_for(Some(served_id), &sprite);
        let fit_note = match *served_id {
            "f_standing_tank" => {
                let f = endon_tank_fit(served_id, &sprite).expect("tank fits");
                format!(
                    "diameter {} length {} leg_h {} neck {:?} fitting {:?}",
                    f.diameter, f.length, f.leg_h, f.neck, f.fitting.0
                )
            }
            "f_console" => {
                let seg = segment_for(sprite_id, &sprite).expect("console segments");
                let f = console_fit(&sprite, &seg);
                format!(
                    "desk_h {} base_h {} recess {} sides {:?}/{:?}/{:?} opening {:?}",
                    f.desk_h, f.base_h, f.recess, f.term_side.0, f.desk_side.0, f.base_side.0,
                    f.opening
                )
            }
            "f_slab" => {
                let f = terrace_fit(&sprite).expect("slab fits");
                format!("levels {} step {} edge {:?}", f.levels, f.step, f.edge.0)
            }
            _ => {
                let f = bed_fit(served_id, &sprite).expect("bed fits");
                format!(
                    "frame {:?} mattress {:?} headboard_top {} head_rows {:?}",
                    f.frame.0, f.mattress.0, f.headboard_top, f.head_rows
                )
            }
        };
        println!("{served_id} <- {sprite_id}: {view:?}; {fit_note}");
        let oy = row as u32 * CELL_H;
        let big = image::imageops::resize(
            &sprite,
            sprite.width() * 2,
            sprite.height() * 2,
            image::imageops::FilterType::Nearest,
        );
        let sx = (CELL_W - big.width()) / 2;
        let sy = oy + CELL_H - big.height() - 10;
        image::imageops::overlay(&mut sheet, &big, sx as i64, sy as i64);
        let model = model_for(served_id, &sprite)
            .unwrap_or_else(|| panic!("{served_id} has a prop model"));
        render_model(&mut sheet, &model, V3(0.0, 0.65, 1.0), CELL_W, oy); // FRONT
        render_model(&mut sheet, &model, V3(0.0, 0.65, -1.0), CELL_W * 2, oy); // BACK
        render_model(&mut sheet, &model, V3(-1.0, 0.65, 0.0), CELL_W * 3, oy); // LEFT
        render_model(&mut sheet, &model, V3(1.0, 0.65, 0.0), CELL_W * 4, oy); // RIGHT
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("../../target/stage-g/rework.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target/stage-g");
    }
    sheet.save(&out).expect("save sheet");
    println!("wrote {}", out.display());
}
