//! Stage-G vehicle composition (docs/vehicle-stage-g.md).
//!
//! The closed catalog is the parts, not the vehicles: per-role
//! constructors turn a part's sprite into fine geometry, and a
//! vehicle-level analysis over the part layout supplies the
//! zones the constructors need (cabin, deck line, front/rear).
//! The composition is a faithful port of the 2026-11-24 Python
//! prototype (`mockup3.py` + `export_models.py`, out of repo),
//! which is the spec: S1's gate pins per-tile cell occupancy
//! against the prototype's exported car models exactly.
//!
//! Coordinates: vehicle frame, x = forward (+x is the front,
//! where the headlights are), y = south, z = up. Models are
//! per tile, size [32, 96, 32] — the prototype worked in
//! "mockup units" (48 per z-level); engine fine rows are
//! units × 2. Python semantics are mirrored exactly where they
//! differ from Rust's defaults: `round()` is round-half-even
//! ([`py_round`]), `int()` truncates, division is float.
//!
//! Part state (`open`, `broken`) is carried on the layout but
//! not yet acted on — the prototype composed the closed,
//! undamaged vehicle, and state presentation is later work.

use crate::props::PropModel;
use image::{Rgba, RgbaImage};
use std::collections::BTreeMap;

/// Fine cells per tile edge.
pub const FINE: i32 = 32;
/// Fine rows per z-level in a vehicle model.
pub const LEVEL_ROWS: u32 = 96;

// Profile constants, in mockup units (48 per z-level).
const CLEAR: f64 = 9.0;
const FRAME_Z: (f64, f64) = (3.0, 11.0);
const DECK: f64 = 26.0;
const GLASS_TOP: f64 = 40.0;
const ROOF_Z: (f64, f64) = (40.0, 45.0);
const SEAT_Z: (f64, f64) = (11.0, 20.0);
const WHEEL_R: i32 = 16;
/// The glass colour (prototype GLASS).
pub const GLASS: [u8; 3] = [70, 92, 110];
const PILLAR: [u8; 3] = [24, 26, 30];
const ROOF_PAINT: [u8; 3] = [24, 123, 180];

/// Base ids with no sprite and no constructor (internal parts).
const INTERNAL_PREFIXES: [&str; 7] = [
    "engine_",
    "tank",
    "dashboard",
    "horn_",
    "wheel_mount_",
    "vehicle_alarm",
    "integrated_heater_small",
];

/// Sprite fallback materials for base ids whose plain material
/// is a variant (the tileset lookup the prototype used).
fn fallback_material(base: &str) -> Option<&'static str> {
    match base {
        "halfboard" => Some("vp_halfboard_horizontal"),
        "door_trunk" => Some("vp_door_trunk_front"),
        "reclining_seat" => Some("vp_reclining_seat_front"),
        "seat_back" => Some("vp_seat_back_front"),
        _ => None,
    }
}

/// One part on a tile, in vehicle frame. `base` is the vpart
/// base id (no `#variant` — the wire carries base ids only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSpot {
    pub base: String,
    /// The wire's ROOF flag for this part instance.
    pub roof: bool,
    pub open: bool,
    pub broken: bool,
}

/// A vehicle's part layout in its own frame: tile (x, y, z) →
/// the parts mounted there, in definition order. x/y are the
/// parts' mounts; the engine stamps the composed models at
/// the parts' exported absolute positions, never at positions
/// recomputed from this layout.
#[derive(Debug, Clone, Default)]
pub struct VehicleLayout {
    pub tiles: BTreeMap<(i32, i32, i32), Vec<PartSpot>>,
}

/// Python `round()`: round half to even.
fn py_round(x: f64) -> i32 {
    x.round_ties_even() as i32
}

fn lerp(a: f64, b: f64, f: f64) -> f64 {
    a + (b - a) * f
}

/// Resolve a part's sprite through the prototype's lookup
/// order: `vp_<base>`, `<base>`, the variant fallback.
fn sprite_for(base: &str, sprites: &dyn Fn(&str) -> Option<RgbaImage>) -> Option<RgbaImage> {
    if INTERNAL_PREFIXES.iter().any(|p| base.starts_with(p)) {
        return None;
    }
    let vp = format!("vp_{base}");
    if let Some(s) = sprites(&vp) {
        return Some(s);
    }
    if let Some(s) = sprites(base) {
        return Some(s);
    }
    if let Some(fb) = fallback_material(base) {
        if let Some(s) = sprites(fb) {
            return Some(s);
        }
    }
    None
}

/// One vertical run of one part in one fine column.
#[derive(Debug, Clone)]
struct Seg {
    z0: f64,
    z1: f64,
    color: [u8; 3],
    part: String,
}

/// Column grid in vehicle fine coordinates.
type Grid = std::collections::HashMap<(i32, i32), Vec<Seg>>;

fn put(grid: &mut Grid, gx: i32, gy: i32, z0: f64, z1: f64, color: [u8; 3], part: &str) {
    if z1 > z0 {
        grid.entry((gx, gy)).or_default().push(Seg {
            z0,
            z1,
            color,
            part: part.to_string(),
        });
    }
}

/// Extrude a sprite's alpha mask from `z0` to a per-column top.
fn extrude(
    grid: &mut Grid,
    tx: i32,
    ty: i32,
    spr: &RgbaImage,
    z0: f64,
    top: &dyn Fn(i32, i32, i32) -> f64,
    keep: Option<&dyn Fn(i32, i32) -> bool>,
    part: &str,
) {
    let (w, h) = (spr.width() as i32, spr.height() as i32);
    for sy in 0..h {
        for sx in 0..w {
            let px = spr.get_pixel(sx as u32, sy as u32);
            if px[3] < 128 {
                continue;
            }
            let (lx, ly) = (sx * FINE / w, sy * FINE / h);
            if let Some(k) = keep {
                if !k(lx, ly) {
                    continue;
                }
            }
            let (gx, gy) = (tx * FINE + lx, ty * FINE + ly);
            put(grid, gx, gy, z0, top(gx, lx, ly), [px[0], px[1], px[2]], part);
        }
    }
}

/// Zones derived from one z-level's tile map.
struct Zones {
    roof_tiles: std::collections::HashSet<(i32, i32)>,
    glass_tiles: std::collections::HashSet<(i32, i32)>,
    cabin_x0: i32,
    cabin_x1: i32,
    roof_x0: i32,
    roof_x1: i32,
    front_x: i32,
    rear_x: i32,
    row_lo: i32,
    row_hi: i32,
}

fn zones(tiles: &BTreeMap<(i32, i32), Vec<PartSpot>>) -> Option<Zones> {
    if tiles.is_empty() {
        return None;
    }
    let roof_tiles: std::collections::HashSet<(i32, i32)> = tiles
        .iter()
        .filter(|(_, ps)| ps.iter().any(|p| p.roof))
        .map(|(t, _)| *t)
        .collect();
    let glass_tiles: std::collections::HashSet<(i32, i32)> = tiles
        .iter()
        .filter(|(_, ps)| ps.iter().any(|p| p.base == "windshield"))
        .map(|(t, _)| *t)
        .collect();
    let cabin: Vec<i32> = roof_tiles
        .iter()
        .chain(glass_tiles.iter())
        .map(|t| t.0)
        .collect();
    let (cabin_x0, cabin_x1) = if cabin.is_empty() {
        (0, -1) // no cabin: the range checks below never fire
    } else {
        (*cabin.iter().min().unwrap(), *cabin.iter().max().unwrap())
    };
    let roof_x0 = roof_tiles.iter().map(|t| t.0).min().unwrap_or(0);
    let roof_x1 = roof_tiles.iter().map(|t| t.0).max().unwrap_or(-1);
    Some(Zones {
        roof_tiles,
        glass_tiles,
        cabin_x0,
        cabin_x1,
        roof_x0,
        roof_x1,
        front_x: tiles.keys().map(|t| t.0).max().unwrap(),
        rear_x: tiles.keys().map(|t| t.0).min().unwrap(),
        row_lo: tiles.keys().map(|t| t.1).min().unwrap(),
        row_hi: tiles.keys().map(|t| t.1).max().unwrap(),
    })
}

/// The deck heightfield (prototype `deck_top_at`).
fn deck_top_at(z: &Zones, gx: i32, tx: i32) -> f64 {
    let lx = gx - tx * FINE;
    let f = lx as f64 / FINE as f64;
    if tx == z.front_x {
        lerp(DECK, 17.0, f) // hood leading edge slopes down
    } else if tx == z.rear_x {
        lerp(21.0, 30.0, f) // rises toward the rear glass
    } else if tx == z.roof_x0 - 1 {
        lerp(DECK, 33.0, f) // rear-glass ramp zone
    } else {
        DECK
    }
}

/// Compose one z-level's tiles into the segment grid
/// (prototype mockup3's build, before the exporter's passes).
fn build_level(
    tiles: &BTreeMap<(i32, i32), Vec<PartSpot>>,
    sprites: &dyn Fn(&str) -> Option<RgbaImage>,
) -> (Grid, Zones) {
    let z = zones(tiles).expect("non-empty level");
    let mut grid: Grid = Grid::new();
    for ((tx, ty), spots) in tiles {
        let (tx, ty) = (*tx, *ty);
        for spot in spots {
            let base = spot.base.as_str();
            if base == "wheel" {
                continue; // the tile-level constructor below
            }
            let Some(spr) = sprite_for(base, sprites) else {
                continue;
            };
            if spot.roof {
                let keep = |lx: i32, ly: i32| {
                    !(lx < 4 && !z.roof_tiles.contains(&(tx - 1, ty)))
                        && !(lx >= FINE - 4 && !z.roof_tiles.contains(&(tx + 1, ty)))
                        && !(ly < 4 && !z.roof_tiles.contains(&(tx, ty - 1)))
                        && !(ly >= FINE - 4 && !z.roof_tiles.contains(&(tx, ty + 1)))
                };
                let top = |_gx: i32, lx: i32, _ly: i32| {
                    if tx == z.roof_x0 && lx < 8 {
                        lerp(41.0, ROOF_Z.1, lx as f64 / 8.0) // rear edge slopes down
                    } else {
                        ROOF_Z.1
                    }
                };
                extrude(&mut grid, tx, ty, &spr, ROOF_Z.0, &top, Some(&keep), "roof");
            } else if base == "windshield" {
                // Sloped glass sheet: GLASS_TOP at the cabin
                // edge down to DECK at the front edge, over a
                // body-coloured panel in the sprite's colours.
                let (w, h) = (spr.width() as i32, spr.height() as i32);
                for sy in 0..h {
                    for sx in 0..w {
                        let px = spr.get_pixel(sx as u32, sy as u32);
                        if px[3] < 128 {
                            continue;
                        }
                        let (lx, ly) = (sx * FINE / w, sy * FINE / h);
                        let (gx, gy) = (tx * FINE + lx, ty * FINE + ly);
                        let z1 = lerp(GLASS_TOP, DECK, lx as f64 / FINE as f64);
                        put(&mut grid, gx, gy, CLEAR, z1 - 3.0, [px[0], px[1], px[2]], "windshield");
                        put(&mut grid, gx, gy, z1 - 3.0, z1, GLASS, "glass");
                    }
                }
            } else if base == "frame" || base == "xlframe" {
                extrude(&mut grid, tx, ty, &spr, FRAME_Z.0, &|_, _, _| FRAME_Z.1, None, "frame");
            } else if matches!(base, "seat" | "reclining_seat" | "seat_back") {
                extrude(&mut grid, tx, ty, &spr, SEAT_Z.0, &|_, _, _| SEAT_Z.1, None, base);
            } else {
                let top = |gx: i32, _lx: i32, _ly: i32| deck_top_at(&z, gx, tx);
                extrude(&mut grid, tx, ty, &spr, CLEAR, &top, None, base);
            }
        }
        // Rear glass: a sloped sheet over the rear tile's
        // front half, from the deck rise up to the roof's
        // rear edge.
        if tx == z.rear_x {
            for lx in FINE / 2..FINE {
                for ly in 0..FINE {
                    let z1 = lerp(30.0, 41.0, (lx - FINE / 2) as f64 / (FINE / 2) as f64);
                    put(&mut grid, tx * FINE + lx, ty * FINE + ly, z1 - 3.0, z1, GLASS, "glass");
                }
            }
        }
        // Side glass with tumblehome on the cabin's edge
        // rows, solid pillars at the cabin's rear end.
        if z.cabin_x0 <= tx && tx <= z.cabin_x1 && !z.glass_tiles.contains(&(tx, ty))
            && (ty == z.row_lo || ty == z.row_hi)
        {
            for lx in 0..FINE {
                for ly in 0..FINE {
                    let edge = if ty == z.row_lo { ly } else { FINE - 1 - ly };
                    if lx < 4 && tx != z.cabin_x1 {
                        if edge < 3 {
                            put(&mut grid, tx * FINE + lx, ty * FINE + ly,
                                DECK, GLASS_TOP, PILLAR, "pillar");
                        }
                        continue;
                    }
                    if edge < 1 {
                        put(&mut grid, tx * FINE + lx, ty * FINE + ly, DECK, 32.0, GLASS, "glass");
                    } else if edge < 3 {
                        put(&mut grid, tx * FINE + lx, ty * FINE + ly,
                            DECK, GLASS_TOP, GLASS, "glass");
                    }
                }
            }
        }
        // Wheel constructor: a disc at the tile's centre
        // column, on the vehicle's outer side.
        if spots.iter().any(|p| p.base == "wheel") {
            if let Some(spr) = sprite_for("wheel", sprites) {
                let (w, h) = (spr.width() as i32, spr.height() as i32);
                let tyre = spr.get_pixel((w / 2) as u32, 2);
                let hub = spr.get_pixel((w / 2) as u32, (h / 2) as u32);
                let rows: Vec<i32> = if ty == z.row_lo {
                    (0..3).collect()
                } else {
                    (FINE - 3..FINE).collect()
                };
                let cx = tx * FINE + FINE / 2;
                for dx in -WHEEL_R..=WHEEL_R {
                    let half = ((WHEEL_R * WHEEL_R - dx * dx) as f64).sqrt() as i32;
                    let col = if dx.abs() <= 5 { hub } else { tyre };
                    for ly in &rows {
                        put(
                            &mut grid,
                            cx + dx,
                            ty * FINE + ly,
                            (WHEEL_R - half) as f64,
                            (WHEEL_R + half) as f64,
                            [col[0], col[1], col[2]],
                            "wheel",
                        );
                    }
                }
            }
        }
    }
    (grid, z)
}

/// The exporter's crown + cabin-enclosure passes.
fn crown_and_enclosure(grid: &mut Grid, z: &Zones, tiles: &BTreeMap<(i32, i32), Vec<PartSpot>>) {
    // Lateral extent from the vehicle's own footprint (the
    // prototype's grid extent, made deterministic).
    let gy_lo = z.row_lo * FINE;
    let gy_hi = z.row_hi * FINE + (FINE - 1);
    let centre = (gy_lo + gy_hi) as f64 / 2.0;
    let half_width = (gy_hi - gy_lo + 1) as f64 / 2.0;
    let crown_top = |gy: i32| -> f64 {
        let d = ((gy as f64 - centre).abs()) / half_width;
        ROOF_Z.1 - py_round(3.0 * d * d) as f64
    };
    for ((_, gy), segs) in grid.iter_mut() {
        for s in segs.iter_mut() {
            if s.z0 == ROOF_Z.0 && s.z1 > 44.5 {
                s.z1 = crown_top(*gy);
            }
        }
    }
    // Roof across the cabin's two outer rows (the prototype
    // iterates exactly the row_lo/row_hi pair) at every roof
    // tile's x, plus a 4-column strip past the front roof
    // edge on the roof tiles' own rows.
    let mut roof_xs: Vec<i32> = z.roof_tiles.iter().map(|t| t.0).collect();
    roof_xs.sort_unstable();
    roof_xs.dedup();
    let roof_rows: Vec<i32> = {
        let mut r: Vec<i32> = z.roof_tiles.iter().map(|t| t.1).collect();
        r.sort_unstable();
        r.dedup();
        r
    };
    let _ = tiles;
    for tx in &roof_xs {
        for ty in [z.row_lo, z.row_hi] {
            for lx in 0..FINE {
                for ly in 0..FINE {
                    let gy = ty * FINE + ly;
                    put(grid, tx * FINE + lx, gy, ROOF_Z.0, crown_top(gy), ROOF_PAINT, "roof");
                }
            }
        }
        if *tx == z.roof_x1 {
            for ty in &roof_rows {
                for lx in FINE - 4..FINE {
                    for ly in 0..FINE {
                        let (gx, gy) = (tx * FINE + lx, ty * FINE + ly);
                        let has_roof = grid
                            .get(&(gx, gy))
                            .is_some_and(|segs| segs.iter().any(|s| s.z0 == ROOF_Z.0));
                        if !has_roof {
                            put(grid, gx, gy, ROOF_Z.0, crown_top(gy), ROOF_PAINT, "roof");
                        }
                    }
                }
            }
        }
    }
}

/// Fill the segment grid into per-tile dense models
/// (prototype exporter's fill: rows `round(z0*2)..round(z1*2)`,
/// later segments win a shared voxel).
fn fill(grid: &Grid) -> Vec<((i32, i32), PropModel)> {
    let mut models: BTreeMap<(i32, i32), PropModel> = BTreeMap::new();
    for ((gx, gy), segs) in grid {
        let (tx, ty) = (gx.div_euclid(FINE), gy.div_euclid(FINE));
        let (lx, ly) = ((gx - tx * FINE) as u32, (gy - ty * FINE) as u32);
        let model = models
            .entry((tx, ty))
            .or_insert_with(|| PropModel::new([FINE as u32, LEVEL_ROWS, FINE as u32]));
        for s in segs {
            for y in py_round(s.z0 * 2.0)..py_round(s.z1 * 2.0) {
                if (0..LEVEL_ROWS as i32).contains(&y) {
                    model.set(lx, y as u32, ly, Some(Rgba([s.color[0], s.color[1], s.color[2], 255])));
                }
            }
        }
    }
    models.into_iter().collect()
}

/// Compose a vehicle's fine models, one per occupied tile,
/// keyed by vehicle-frame tile (x, y, z-level).
pub fn compose_vehicle(
    layout: &VehicleLayout,
    sprites: &dyn Fn(&str) -> Option<RgbaImage>,
) -> Vec<((i32, i32, i32), PropModel)> {
    let mut levels: BTreeMap<i32, BTreeMap<(i32, i32), Vec<PartSpot>>> = BTreeMap::new();
    for ((x, y, zl), spots) in &layout.tiles {
        levels.entry(*zl).or_default().insert((*x, *y), spots.clone());
    }
    let mut out = Vec::new();
    for (zl, tiles) in levels {
        let (mut grid, z) = build_level(&tiles, sprites);
        crown_and_enclosure(&mut grid, &z, &tiles);
        for ((tx, ty), model) in fill(&grid) {
            out.push(((tx, ty, zl), model));
        }
    }
    out
}
