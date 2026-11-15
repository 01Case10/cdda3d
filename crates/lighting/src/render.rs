//! CPU PNG dumps for gate 7.8 — the Phase 0 precedent: visual
//! evidence produced CPU-side, honestly labelled. These are *not*
//! renderer captures. Two projections are provided:
//!
//! - [`render_top_down`]: one pixel block per tile, coloured by the
//!   baked level-0 presentation. Orthographic, looking straight
//!   down; roofs are a cutaway concern, so the caller passes the
//!   level to draw.
//! - [`render_isometric`]: a simple dimetric projection of the baked
//!   chunk map's blocks (top faces + two side faces with fixed
//!   directional shading), drawn back-to-front. It exists so a
//!   human can see the house *shape* with its light, not just a
//!   plan view.
//!
//! Both consume *baked* data (post-`bake_chunk_map` / presented
//! colours), so what they show is exactly what the renderers would
//! consume, minus GPU shading.

use mesher::chunk::ChunkMap;

use crate::atmosphere::Environment;
use crate::grid::LightSampler;

/// An RGB image buffer.
#[derive(Debug, Clone)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGB, row-major, top row first
}
impl RgbImage {
    pub fn new(width: u32, height: u32, fill: [u8; 3]) -> Self {
        let mut pixels = vec![0u8; (width * height * 3) as usize];
        for i in 0..(width * height) as usize {
            pixels[i * 3..i * 3 + 3].copy_from_slice(&fill);
        }
        Self {
            width,
            height,
            pixels,
        }
    }
    pub fn set(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let i = ((y as u32 * self.width + x as u32) * 3) as usize;
        self.pixels[i..i + 3].copy_from_slice(&c);
    }
    pub fn get(&self, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * self.width + x) * 3) as usize;
        [self.pixels[i], self.pixels[i + 1], self.pixels[i + 2]]
    }
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
    pub fn distinct_colors(&self) -> usize {
        let mut set = std::collections::HashSet::new();
        for px in self.pixels.chunks_exact(3) {
            set.insert([px[0], px[1], px[2]]);
        }
        set.len()
    }
    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        crate::png::write_png_rgb(path, self.width, self.height, &self.pixels)
    }
}

fn to_u8(c: [f32; 3]) -> [u8; 3] {
    c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Top-down dump of one z-level of a scene given as (map, sampler):
/// each tile is drawn with the *presented* colour of the topmost
/// non-air block in that tile at that level (its baked albedo would
/// be identical — this function recomputes the presentation from the
/// unbaked map + sampler so it can be used before or after baking).
/// Hidden tiles draw pure black; empty tiles draw the background.
pub fn render_top_down<S: LightSampler + ?Sized>(
    map: &ChunkMap,
    sampler: &S,
    level: i32,
    env: Environment,
    px_per_tile: u32,
) -> RgbImage {
    let scale = map.scale;
    let per = scale.blocks_per_z_level as i32;
    // Bounds from the chunk map.
    let (mut max_x, mut max_y) = (0i32, 0i32);
    for chunk in map.chunks.values() {
        let o = chunk.origin();
        max_x = max_x.max(o[0] + chunk.grid.size[0] as i32);
        max_y = max_y.max(o[1] + chunk.grid.size[1] as i32);
    }
    let mut img = RgbImage::new(
        (max_x as u32) * px_per_tile,
        (max_y as u32) * px_per_tile,
        [10, 10, 14],
    );
    for gy in 0..max_y {
        for gx in 0..max_x {
            // Topmost block in this tile at this level.
            let base = level * per;
            let mut albedo: Option<[f32; 3]> = None;
            for bz in (base..base + per).rev() {
                let b = map_block(map, gx, gy, bz);
                if let Some(b) = b {
                    if !b.is_air() {
                        albedo = Some([b.color[0], b.color[1], b.color[2]]);
                        break;
                    }
                }
            }
            let t = sampler.tile_light(gx, gy, level);
            let color = match albedo {
                None => {
                    if t.visibility() == crate::mapping::Visibility::Hidden {
                        [0u8, 0, 0]
                    } else {
                        continue; // background
                    }
                }
                Some(a) => {
                    // shade_color is the single reference composition:
                    // visible = albedo x env(raw sample); memorized =
                    // desaturated/dimmed albedo x env tint; hidden =
                    // black. (present_color is the no-environment form
                    // of the same composition, exercised in tests.)
                    let c = crate::bake::shade_color(a, t.raw_sample(), t.visibility(), env);
                    to_u8(c)
                }
            };
            img.fill_rect(
                gx * px_per_tile as i32,
                gy * px_per_tile as i32,
                px_per_tile as i32,
                px_per_tile as i32,
                color,
            );
        }
    }
    img
}

fn map_block(map: &ChunkMap, gx: i32, gy: i32, gz: i32) -> Option<mesher::block::Block> {
    use mesher::grid::VoxelSampler;
    Some(map.block_at(gx, gy, gz))
}

/// Simple isometric (dimetric) dump of a *baked* chunk map: blocks
/// are drawn as top + left + right faces with fixed face shading
/// (top ×1.0, left ×0.75, right ×0.55 — a presentation constant of
/// the dump, not game light), back-to-front by (x + y, then z).
/// The baked block colours already carry game light, so the dump
/// shows lit shape, honestly labelled as a CPU projection.
pub fn render_isometric(baked: &ChunkMap, px: u32) -> RgbImage {
    let scale = baked.scale;
    let bs = px as i32; // pixels per block edge (top face is bs x bs/2)
    let (mut max_x, mut max_y, mut max_z) = (0i32, 0i32, 0i32);
    for chunk in baked.chunks.values() {
        let o = chunk.origin();
        max_x = max_x.max(o[0] + chunk.grid.size[0] as i32);
        max_y = max_y.max(o[1] + chunk.grid.size[1] as i32);
        max_z = max_z.max(chunk.grid.size[2] as i32);
    }
    let _ = scale;
    let w = ((max_x + max_y) * bs) as u32;
    let h = (((max_x + max_y) * bs) / 2 + max_z * bs) as u32;
    let mut img = RgbImage::new(w.max(1), h.max(1), [8, 8, 12]);
    // Collect blocks, sort painter-style: increasing (x+y), then z.
    let mut blocks: Vec<([i32; 3], [f32; 3])> = Vec::new();
    for chunk in baked.chunks.values() {
        let o = chunk.origin();
        for z in 0..chunk.grid.size[2] as i32 {
            for y in 0..chunk.grid.size[1] as i32 {
                for x in 0..chunk.grid.size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    if b.is_air() {
                        continue;
                    }
                    blocks.push((
                        [o[0] + x, o[1] + y, z],
                        [b.color[0], b.color[1], b.color[2]],
                    ));
                }
            }
        }
    }
    blocks.sort_by_key(|(p, _)| (p[0] + p[1], p[2]));
    for (p, c) in blocks {
        // Screen position of the block's top-face centre-ish origin.
        let sx = (p[0] - p[1] + max_y) * bs;
        let sy = ((p[0] + p[1]) * bs) / 2 - p[2] * bs + max_z * bs / 2;
        let top = to_u8(c);
        let left = to_u8([c[0] * 0.72, c[1] * 0.72, c[2] * 0.72]);
        let right = to_u8([c[0] * 0.52, c[1] * 0.52, c[2] * 0.52]);
        // Top diamond.
        for dy in 0..bs / 2 {
            let half = dy * 2 + 2;
            for dx in -half..half {
                img.set(sx + dx, sy + dy, top);
                img.set(sx + dx, sy + bs - 1 - dy, top);
            }
        }
        // Side faces: simple rectangles below the diamond halves.
        for dy in 0..bs {
            for dx in -bs..0 {
                img.set(sx + dx, sy + bs / 2 + dy, left);
            }
            for dx in 0..bs {
                img.set(sx + dx, sy + bs / 2 + dy, right);
            }
        }
        // Redraw the top diamond over the side tops.
        for dy in 0..bs / 2 {
            let half = dy * 2 + 2;
            for dx in -half..half {
                img.set(sx + dx, sy + dy, top);
                img.set(sx + dx, sy + bs - 1 - dy, top);
            }
        }
    }
    img
}
