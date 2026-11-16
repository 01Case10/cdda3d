//! Shoreline autotiling (Phase 5 rework, Stage E; plan SS10.1,
//! voxelizer half): round-looking ponds from square tiles. The
//! terrain data stays square; the rounding lives in the texels of
//! per-neighborhood shore variants generated here.
//!
//! Each function renders the CENTER tile of a 3x3 neighborhood.
//! `mask` is the 8-neighbor water-adjacency bitmask of that
//! neighborhood, bit order:
//!
//! ```text
//! bit 0: N  ( 0,-1)    bit 4: S  ( 0, 1)
//! bit 1: NE ( 1,-1)    bit 5: SW (-1, 1)
//! bit 2: E  ( 1, 0)    bit 6: W  (-1, 0)
//! bit 3: SE ( 1, 1)    bit 7: NW (-1,-1)
//! ```
//!
//! A set bit means "that neighbor tile is water". For
//! [`shore_texture`] the center tile is water; for
//! [`bank_texture`] the center tile is land and the mask says
//! which neighbors are water.
//!
//! Method: the mask implies a water region on the 96x96
//! neighborhood frame. Its signed distance field — positive
//! inside water, in pixels — is computed exactly (two-pass
//! chamfer distance transform to the nearest opposite-class
//! pixel; outside the frame counts as land) and then box-blurred
//! (radius 4), which rounds convex corners into arcs; a small
//! periodic fBm wobble (+-2.5 px, sampled in the neighborhood
//! frame) bows the line organically. The field maps to a terrain
//! height through the fixed material heights — water 0.0, dirt
//! 0.45, grass 1.0: height 0 well inside water, rising through
//! the dirt shelf across the bank band to full grass height
//! inland. The waterline (the water/dirt height-band crossover)
//! sits ~12 px inside a shore tile, so a shore tile reads as
//! water lapping onto a dirt bank, not as a full water tile with
//! a fringe. Each material's height layer for the blend is a
//! tent around its fixed height, and the three source textures
//! are composited with [`crate::quilt::height_blend`] (two
//! steps, running height = per-pixel max, exactly the winner's
//! height), so the winner at each pixel is the material whose
//! height band the terrain height falls in. Water pixels within
//! 14 px of the shoreline are lightened (shallow tint); deeper
//! water is the source texture untouched, so an all-water
//! neighborhood returns the water input byte-for-byte.
//!
//! Known v1 limitation: the wobble field is tile-local (sampled
//! in the neighborhood frame), so two adjacent shore tiles can
//! disagree by a pixel or two where their waterlines meet. The
//! distance field — the dominant term — is computed from the
//! shared neighborhood geometry and agrees across the border.

use image::{GrayImage, Luma, RgbaImage};

use crate::color::Oklab;
use crate::generate::TEXTURE_SIZE;
use crate::noise::{seed_from_str, Fbm};
use crate::quilt::height_blend;

/// Fixed material heights (plan SS10.1: the blend order is forced
/// by height, never hand-tuned per tile).
pub const WATER_HEIGHT: f32 = 0.0;
pub const DIRT_HEIGHT: f32 = 0.45;
pub const GRASS_HEIGHT: f32 = 1.0;

/// Neighborhood frame size (3x3 tiles at 32 px).
const FRAME: usize = 96;
/// Box-blur radius (px) applied to the distance field: how round
/// the shoreline corners get.
const BLUR_RADIUS: i32 = 4;
/// Wobble amplitude (px) on the distance field.
const WOBBLE_AMP: f32 = 2.5;
/// Water pixels closer than this to the shoreline get the
/// shallow lighten; beyond it the source texture is untouched.
const SHALLOW_RANGE: f32 = 14.0;
/// Field clamp (px): deeper than any real shoreline influence.
const FIELD_MAX: f32 = 64.0;

/// Which material wins a shore pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShoreMaterial {
    Water,
    Dirt,
    Grass,
}

/// Neighbor offsets in bit order (see module docs).
const NEIGHBORS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Water-adjacency mask for tile `(tx, ty)` under a layout
/// predicate in tile coordinates (arbitrary origin): bit `i`
/// is set when the neighbor at offset `NEIGHBORS[i]` is water.
pub fn mask_for(is_water: impl Fn(i32, i32) -> bool, tx: i32, ty: i32) -> u8 {
    let mut mask = 0u8;
    for (bit, (dx, dy)) in NEIGHBORS.iter().enumerate() {
        if is_water(tx + dx, ty + dy) {
            mask |= 1 << bit;
        }
    }
    mask
}

/// Is frame tile `(dx, dy)` (center = (0,0)) water under this
/// mask / center class?
fn tile_is_water(mask: u8, center_water: bool, dx: i32, dy: i32) -> bool {
    if dx == 0 && dy == 0 {
        return center_water;
    }
    for (bit, (nx, ny)) in NEIGHBORS.iter().enumerate() {
        if (*nx, *ny) == (dx, dy) {
            return mask & (1 << bit) != 0;
        }
    }
    false
}

/// Two-pass chamfer distance transform: distance from every
/// cell to the nearest cell for which `target` holds.
fn distance_to(occ: &[bool], target: bool) -> Vec<f32> {
    const INF: f32 = 1.0e6;
    const DIAG: f32 = std::f32::consts::SQRT_2;
    let mut d: Vec<f32> = occ
        .iter()
        .map(|&o| if o == target { 0.0 } else { INF })
        .collect();
    let idx = |x: usize, y: usize| y * FRAME + x;
    // Forward pass (top-left -> bottom-right).
    for y in 0..FRAME {
        for x in 0..FRAME {
            let mut v = d[idx(x, y)];
            if x > 0 {
                v = v.min(d[idx(x - 1, y)] + 1.0);
            }
            if y > 0 {
                v = v.min(d[idx(x, y - 1)] + 1.0);
                if x > 0 {
                    v = v.min(d[idx(x - 1, y - 1)] + DIAG);
                }
                if x + 1 < FRAME {
                    v = v.min(d[idx(x + 1, y - 1)] + DIAG);
                }
            }
            d[idx(x, y)] = v;
        }
    }
    // Backward pass (bottom-right -> top-left).
    for y in (0..FRAME).rev() {
        for x in (0..FRAME).rev() {
            let mut v = d[idx(x, y)];
            if x + 1 < FRAME {
                v = v.min(d[idx(x + 1, y)] + 1.0);
            }
            if y + 1 < FRAME {
                v = v.min(d[idx(x, y + 1)] + 1.0);
                if x + 1 < FRAME {
                    v = v.min(d[idx(x + 1, y + 1)] + DIAG);
                }
                if x > 0 {
                    v = v.min(d[idx(x - 1, y + 1)] + DIAG);
                }
            }
            d[idx(x, y)] = v;
        }
    }
    d
}

/// Separable box blur (clamped edges) over the frame field.
fn box_blur(src: &[f32], radius: i32) -> Vec<f32> {
    let mut tmp = vec![0.0f32; FRAME * FRAME];
    let mut out = vec![0.0f32; FRAME * FRAME];
    let norm = (2 * radius + 1) as f32;
    for y in 0..FRAME as i32 {
        for x in 0..FRAME as i32 {
            let mut sum = 0.0;
            for k in -radius..=radius {
                let cx = (x + k).clamp(0, FRAME as i32 - 1) as usize;
                sum += src[y as usize * FRAME + cx];
            }
            tmp[y as usize * FRAME + x as usize] = sum / norm;
        }
    }
    for y in 0..FRAME as i32 {
        for x in 0..FRAME as i32 {
            let mut sum = 0.0;
            for k in -radius..=radius {
                let cy = (y + k).clamp(0, FRAME as i32 - 1) as usize;
                sum += tmp[cy * FRAME + x as usize];
            }
            out[y as usize * FRAME + x as usize] = sum / norm;
        }
    }
    out
}

/// Build the full 96x96 shore distance field for a neighborhood:
/// signed distance to the water region (positive inside water),
/// corner-rounded by blur, plus the organic wobble.
fn build_field(mask: u8, center_water: bool) -> Vec<f32> {
    let mut occ = vec![false; FRAME * FRAME];
    for fy in 0..FRAME {
        for fx in 0..FRAME {
            let dx = (fx / 32) as i32 - 1;
            let dy = (fy / 32) as i32 - 1;
            occ[fy * FRAME + fx] = tile_is_water(mask, center_water, dx, dy);
        }
    }
    let to_land = distance_to(&occ, false);
    let to_water = distance_to(&occ, true);
    let mut field: Vec<f32> = occ
        .iter()
        .enumerate()
        .map(|(i, &o)| {
            if o {
                to_land[i].min(FIELD_MAX)
            } else {
                -to_water[i].min(FIELD_MAX)
            }
        })
        .collect();
    field = box_blur(&field, BLUR_RADIUS);
    let wobble = Fbm::new(seed_from_str("shore-wobble") ^ mask as u64, 96).octaves(2);
    for fy in 0..FRAME {
        for fx in 0..FRAME {
            let w = wobble.sample(fx as f32 + 0.5, fy as f32 + 0.5);
            field[fy * FRAME + fx] += (w - 0.5) * 2.0 * WOBBLE_AMP;
        }
    }
    field
}

/// Field value at center-tile pixel `(x, y)` from a built field.
fn field_sample(field: &[f32], x: u32, y: u32) -> f32 {
    field[(y as usize + 32) * FRAME + (x as usize + 32)]
}

/// The built neighborhood field, cached per (mask, center
/// class) on the calling thread — there are only 512
/// neighborhoods — so per-pixel lookups and repeated texture
/// generation stay cheap.
fn field_for(mask: u8, center_water: bool) -> std::rc::Rc<Vec<f32>> {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    thread_local! {
        static CACHE: RefCell<HashMap<(u8, bool), Rc<Vec<f32>>>> =
            RefCell::new(HashMap::new());
    }
    CACHE.with(|c| {
        c.borrow_mut()
            .entry((mask, center_water))
            .or_insert_with(|| Rc::new(build_field(mask, center_water)))
            .clone()
    })
}

/// The shore distance field at center-tile pixel `(x, y)`:
/// positive inside the implied water region, in pixels, wobble
/// included.
pub fn field_at(mask: u8, center_water: bool, x: u32, y: u32) -> f32 {
    let field = field_for(mask, center_water);
    field_sample(&field, x, y)
}

/// Terrain height from the shore field: 0.0 (water level) well
/// inside water, rising through the dirt shelf (0.45) across the
/// bank band to 1.0 (grass height) inland. The rises are linear
/// in field pixels; the water/dirt band crossover lands ~12 px
/// inside a shore tile.
fn terrain_height(f: f32) -> f32 {
    if f >= 17.0 {
        WATER_HEIGHT
    } else if f >= 7.0 {
        DIRT_HEIGHT * (17.0 - f) / 10.0
    } else if f >= -13.0 {
        DIRT_HEIGHT + (GRASS_HEIGHT - DIRT_HEIGHT) * (7.0 - f) / 20.0
    } else {
        GRASS_HEIGHT
    }
}

fn tent(h: f32, peak: f32, width: f32) -> u8 {
    let v = (1.0 - (h - peak).abs() / width).clamp(0.0, 1.0);
    (v * 255.0).round() as u8
}

/// The three height layers (grass, dirt, water) for one field
/// value, as fed to [`height_blend`]. Tent peaks sit at the
/// fixed material heights, so the blend winner is the material
/// whose height band the terrain height falls in.
fn heights_for(f: f32) -> (u8, u8, u8) {
    let h = terrain_height(f);
    (
        tent(h, GRASS_HEIGHT, 0.40),
        tent(h, DIRT_HEIGHT, 0.34),
        tent(h, WATER_HEIGHT, 0.28),
    )
}

/// The winning material at center-tile pixel `(x, y)` — the same
/// decision [`height_blend`] makes on the layers from
/// [`heights_for`] (dirt over grass on strict `>`, water over
/// the running winner on strict `>`; ties keep the earlier
/// layer).
pub fn material_at(mask: u8, center_water: bool, x: u32, y: u32) -> ShoreMaterial {
    let (hg, hd, hw) = heights_for(field_at(mask, center_water, x, y));
    pick(hg, hd, hw)
}

fn pick(hg: u8, hd: u8, hw: u8) -> ShoreMaterial {
    let first = if hd > hg {
        ShoreMaterial::Dirt
    } else {
        ShoreMaterial::Grass
    };
    if hw > hg.max(hd) {
        ShoreMaterial::Water
    } else {
        first
    }
}

/// Tile-level water depth for shallow/deep tinting: the shore
/// field evaluated at the tile center IS an erosion depth — the
/// pixel distance from the tile center to the implied shoreline
/// — normalized so 20 px of clearance reads as fully deep (1.0)
/// and a tile center on the waterline reads 0.0. (One erosion
/// step of the neighbor mask ≈ one tile of clearance ≈ depth
/// 1.0; partial neighborhoods land in between by construction.)
pub fn depth_shade(mask: u8) -> f32 {
    (field_at(mask, true, TEXTURE_SIZE / 2, TEXTURE_SIZE / 2) / 20.0).clamp(0.0, 1.0)
}

fn height_image(field: &[f32], which: usize) -> GrayImage {
    let mut img = GrayImage::new(TEXTURE_SIZE, TEXTURE_SIZE);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let hs = heights_for(field_sample(field, x, y));
            let v = match which {
                0 => hs.0,
                1 => hs.1,
                _ => hs.2,
            };
            img.put_pixel(x, y, Luma([v]));
        }
    }
    img
}

fn max_height(a: &GrayImage, b: &GrayImage) -> GrayImage {
    let mut img = GrayImage::new(TEXTURE_SIZE, TEXTURE_SIZE);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            img.put_pixel(
                x,
                y,
                Luma([a.get_pixel(x, y)[0].max(b.get_pixel(x, y)[0])]),
            );
        }
    }
    img
}

fn compose(
    mask: u8,
    center_water: bool,
    water: &RgbaImage,
    dirt: &RgbaImage,
    grass: &RgbaImage,
) -> RgbaImage {
    for (name, img) in [("water", water), ("dirt", dirt), ("grass", grass)] {
        assert_eq!(
            img.dimensions(),
            (TEXTURE_SIZE, TEXTURE_SIZE),
            "shore: {name} source must be 32x32"
        );
    }
    let field = field_for(mask, center_water);
    let hg = height_image(&field, 0);
    let hd = height_image(&field, 1);
    let hw = height_image(&field, 2);
    // Dirt over grass, then water over the running composite.
    let step1 = height_blend(grass, &hg, dirt, &hd, 0.5);
    let h1 = max_height(&hg, &hd);
    let mut out = height_blend(&step1, &h1, water, &hw, 0.5);
    // Shallow lighten: water winners near the shoreline only.
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let hs = heights_for(field_sample(&field, x, y));
            if pick(hs.0, hs.1, hs.2) != ShoreMaterial::Water {
                continue;
            }
            let f = field_sample(&field, x, y);
            if f < SHALLOW_RANGE {
                let p = out.get_pixel(x, y);
                let mut lab = Oklab::from_srgb_u8([p[0], p[1], p[2]]);
                lab.l = (lab.l + 0.05 * (1.0 - f.max(0.0) / SHALLOW_RANGE)).min(1.0);
                let [r, g, b] = lab.to_srgb_u8();
                out.put_pixel(x, y, image::Rgba([r, g, b, p[3]]));
            }
        }
    }
    out
}

/// Shore variant for a WATER tile: curved waterline over a dirt
/// bank, grass fringe where the neighborhood's land corners cut
/// across the tile. `mask` = water adjacency of the 8 neighbors.
pub fn shore_texture(
    mask: u8,
    water: &RgbaImage,
    dirt: &RgbaImage,
    grass: &RgbaImage,
) -> RgbaImage {
    compose(mask, true, water, dirt, grass)
}

/// Bank variant for a LAND tile adjacent to water: the matching
/// dirt ring on the land side of the border, grass beyond it.
/// `mask` = which of the 8 neighbors are water.
pub fn bank_texture(
    mask: u8,
    water: &RgbaImage,
    dirt: &RgbaImage,
    grass: &RgbaImage,
) -> RgbaImage {
    compose(mask, false, water, dirt, grass)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(rgb: [u8; 3]) -> RgbaImage {
        RgbaImage::from_pixel(32, 32, Rgba([rgb[0], rgb[1], rgb[2], 255]))
    }

    #[test]
    fn all_water_neighborhood_returns_water_input() {
        let (w, d, g) = (solid([30, 80, 160]), solid([120, 95, 60]), solid([70, 110, 45]));
        let out = shore_texture(0xFF, &w, &d, &g);
        assert_eq!(out.as_raw(), w.as_raw());
    }

    #[test]
    fn no_water_neighborhood_returns_grass_input() {
        let (w, d, g) = (solid([30, 80, 160]), solid([120, 95, 60]), solid([70, 110, 45]));
        let out = bank_texture(0, &w, &d, &g);
        assert_eq!(out.as_raw(), g.as_raw());
    }

    #[test]
    fn depth_shade_orders_by_clearance() {
        assert_eq!(depth_shade(0xFF), 1.0, "surrounded tile center is fully deep");
        let corner = depth_shade(0xF8);
        assert!(corner > 0.3 && corner < 1.0, "corner depth {corner}");
        let isolated = depth_shade(0);
        assert!(
            isolated > 0.5 && isolated < 1.0,
            "isolated tile center is 16 px from shore: depth {isolated}"
        );
    }
}
