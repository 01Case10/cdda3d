//! Material heuristics: wall detection, furniture heights, extrusion.

use image::RgbaImage;
use tileset::TileEntry;

pub fn is_wall_like(id: &str, entry: &TileEntry) -> bool {
    entry.multitile
        && (id.contains("wall")
            || id.contains("fence")
            || id.contains("bench")
            || id.contains("conduit"))
        || id.contains("_wall")
        || id.ends_with("wall")
}

/// Furniture height heuristic (plan §8): tables 1, bookcases 3,
/// beds 1, appliances 2 (in voxels; one z-level = 3 voxels).
pub fn furniture_height(id: &str, ov: Option<&crate::OverrideEntry>) -> u32 {
    if let Some(o) = ov {
        if let Some(h) = o.height {
            return h;
        }
    }
    let id = id.to_ascii_lowercase();
    if id.contains("bookcase")
        || id.contains("bookshelf")
        || id.contains("locker")
        || id.contains("wardrobe")
        || id.contains("cabinet")
    {
        3
    } else if id.contains("fridge")
        || id.contains("oven")
        || id.contains("washer")
        || id.contains("dryer")
        || id.contains("appliance")
        || id.contains("freezer")
    {
        2
    } else {
        // tables, beds, chairs, desks, counters, everything else
        1
    }
}

/// Extruded-silhouette columns: every sprite pixel whose alpha exceeds
/// the cutout threshold is a solid sub-voxel column. Sorted by (y, x)
/// for determinism (pixels() iterates row-major already).
pub fn extrusion_columns(img: &RgbaImage) -> Vec<[u32; 2]> {
    let mut cols = Vec::new();
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 127 {
            cols.push([x, y]);
        }
    }
    cols
}
