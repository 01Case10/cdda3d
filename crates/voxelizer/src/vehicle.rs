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
//!
//! Colouring ([`Colorizer`]) ports the prototype's tex6 rules
//! plus one fix the prototype computed but never applied: its
//! `repaint()` vehicle-paint unification now covers panel
//! TOPS, not just sides (the hood squiggle root cause,
//! docs/vehicle-stage-g.md §4).

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
/// Stage-G S4: pickup bed floor height (mockup units;
/// assistant-chosen, tuned at the S5 review).
const BED_FLOOR: f64 = 14.0;
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
    resolve_part_sprite(base, sprites).map(|(_, img)| img)
}

/// Resolve a part's sprite, returning the winning
/// material name with the image. Order: the part's own
/// vp_ entry; for wheels the multitile front/rear entries
/// (the bicycle wheel exists only as
/// vp_wheel_bicycle_front/_rear — the bare name is an
/// unrelated item icon, which this ordering must never
/// reach first); the hand-pinned fallback map; and only
/// then the bare part name.
pub fn resolve_part_sprite(
    base: &str,
    sprites: &dyn Fn(&str) -> Option<RgbaImage>,
) -> Option<(String, RgbaImage)> {
    if INTERNAL_PREFIXES.iter().any(|p| base.starts_with(p)) {
        return None;
    }
    let mut candidates = vec![format!("vp_{base}")];
    if base.starts_with("wheel") {
        candidates.push(format!("vp_{base}_front"));
        candidates.push(format!("vp_{base}_rear"));
    }
    if let Some(fb) = fallback_material(base) {
        candidates.push(fb.to_string());
    }
    // The bare name comes LAST: some tileset entries are
    // genuinely keyed bare (frame, seat), but others
    // collide with item icons (wheel_bicycle is a 20x20
    // small.png item sprite) — the vp_ and suffixed
    // candidates above must win first.
    candidates.push(base.to_string());
    for name in candidates {
        if let Some(s) = sprites(&name) {
            return Some((name, s));
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

/// Draw a sprite as a side elevation in the tile's centre
/// plane (rows 15..18): sprite columns map to tile x,
/// sprite rows map to height (30 units over the tile).
/// Thin vehicles' tileset art is drawn this way — the
/// bicycle's wheel, saddle, pedals and rear basket are all
/// side elevations, unlike the cars' top-down sprites.
fn draw_profile(grid: &mut Grid, tx: i32, ty: i32, spr: &RgbaImage, part: &str) {
    let (w, h) = (spr.width() as i32, spr.height() as i32);
    for sy in 0..h {
        for sx in 0..w {
            let px = spr.get_pixel(sx as u32, sy as u32);
            if px[3] < 128 {
                continue;
            }
            let lx = sx * FINE / w;
            let zt = (h - 1 - sy) as f64 * 30.0 / h as f64;
            for ly in 15..18 {
                put(grid, tx * FINE + lx, ty * FINE + ly,
                    zt, zt + 30.0 / h as f64,
                    [px[0], px[1], px[2]], part);
            }
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
    /// Stage-G S4: the pickup bed — cargo_space tiles with
    /// no roof part, flooded over adjacent trunk tiles.
    bed_tiles: std::collections::HashSet<(i32, i32)>,
    /// Stage-G S4: a single-row vehicle (a bicycle) is a
    /// thin profile object, not a footprint extrusion.
    thin: bool,
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
    // Bed: seed on cargo tiles with no roof part, then
    // flood over 4-connected trunk tiles (the bed floor
    // continues under the tailgate in the pickup layout).
    let mut bed_tiles: std::collections::HashSet<(i32, i32)> = tiles
        .iter()
        .filter(|(t, ps)| {
            !roof_tiles.contains(t)
                && ps.iter().any(|p| p.base == "cargo_space")
        })
        .map(|(t, _)| *t)
        .collect();
    loop {
        let grown: Vec<(i32, i32)> = tiles
            .keys()
            .filter(|t| {
                !bed_tiles.contains(t)
                    && tiles[*t].iter().any(|p| p.base == "trunk")
                    && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .any(|(dx, dy)| bed_tiles.contains(&(t.0 + dx, t.1 + dy)))
            })
            .copied()
            .collect();
        if grown.is_empty() {
            break;
        }
        bed_tiles.extend(grown);
    }
    let row_lo = tiles.keys().map(|t| t.1).min().unwrap();
    let row_hi = tiles.keys().map(|t| t.1).max().unwrap();
    // Thin means a profile-art vehicle (the bicycle): a
    // single row AND no bed — a one-row cargo cart or
    // trailer keeps the footprint treatment, its sprites
    // are top-down like the cars'.
    let thin = row_lo == row_hi && bed_tiles.is_empty();
    Some(Zones {
        roof_tiles,
        glass_tiles,
        cabin_x0,
        cabin_x1,
        roof_x0,
        roof_x1,
        front_x: tiles.keys().map(|t| t.0).max().unwrap(),
        rear_x: tiles.keys().map(|t| t.0).min().unwrap(),
        row_lo,
        row_hi,
        bed_tiles,
        thin,
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
        // Thin-vehicle profile wheels are drawn after the
        // tile's other parts: in the shared profile plane
        // the wheel must win over the parts behind it (a
        // rear basket otherwise swallows the rear wheel).
        let mut thin_wheels: Vec<RgbaImage> = Vec::new();
        for spot in spots {
            let base = spot.base.as_str();
            if z.thin {
                // Thin vehicle: every part is a side
                // elevation in the centre plane. The
                // tileset's bike-specific variant of a
                // part (vp_<base>_bike_rear) wins over the
                // generic sprite when it exists. Wheels
                // defer so they draw over the parts behind
                // them in the plane.
                let Some(generic) = sprite_for(base, sprites) else {
                    continue;
                };
                let spr = sprites(&format!("vp_{base}_bike_rear")).unwrap_or(generic);
                if base.starts_with("wheel") {
                    thin_wheels.push(spr);
                } else {
                    draw_profile(&mut grid, tx, ty, &spr, base);
                }
                continue;
            }
            if base == "wheel" {
                continue; // the tile-level constructor below
            }
            let Some(spr) = sprite_for(base, sprites) else {
                continue;
            };
            // Thin vehicles: footprint parts extrude only
            // in the centre band (the object's profile
            // plane); the band keep composes with each
            // branch's own keep.
            let band = |_lx: i32, ly: i32| !z.thin || (13..19).contains(&ly);
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
                let keep2 = |lx: i32, ly: i32| keep(lx, ly) && band(lx, ly);
                extrude(&mut grid, tx, ty, &spr, ROOF_Z.0, &top, Some(&keep2), "roof");
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
                extrude(&mut grid, tx, ty, &spr, FRAME_Z.0, &|_, _, _| FRAME_Z.1, Some(&band), "frame");
            } else if matches!(base, "seat" | "reclining_seat" | "seat_back" | "saddle") {
                extrude(&mut grid, tx, ty, &spr, SEAT_Z.0, &|_, _, _| SEAT_Z.1, Some(&band), base);
            } else if matches!(base, "cargo_space" | "trunk") && z.bed_tiles.contains(&(tx, ty)) {
                // The bed: interior columns stop at the
                // floor; a 4-wide rim on any edge facing a
                // non-bed tile keeps the panel height (bed
                // walls and tailgate).
                let bed = &z.bed_tiles;
                let top = |gx: i32, lx: i32, ly: i32| {
                    let rim = (lx < 4 && !bed.contains(&(tx - 1, ty)))
                        || (lx >= FINE - 4 && !bed.contains(&(tx + 1, ty)))
                        || (ly < 4 && !bed.contains(&(tx, ty - 1)))
                        || (ly >= FINE - 4 && !bed.contains(&(tx, ty + 1)));
                    if rim {
                        deck_top_at(&z, gx, tx)
                    } else {
                        BED_FLOOR
                    }
                };
                extrude(&mut grid, tx, ty, &spr, CLEAR, &top, Some(&band), base);
            } else {
                let top = |gx: i32, _lx: i32, _ly: i32| deck_top_at(&z, gx, tx);
                extrude(&mut grid, tx, ty, &spr, CLEAR, &top, Some(&band), base);
            }
        }
        // Rear glass: a sloped sheet over the rear tile's
        // front half, from the deck rise up to the roof's
        // rear edge.
        if tx == z.rear_x && !z.roof_tiles.is_empty() && z.rear_x >= z.roof_x0 - 1 {
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
        for spr in &thin_wheels {
            draw_profile(&mut grid, tx, ty, spr, "wheel");
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
/// later segments win a shared voxel; each row's colour is
/// the Colorizer's, from the segment's part and depth).
fn fill(grid: &Grid, colorizer: &Colorizer) -> Vec<((i32, i32), PropModel)> {
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
                    let c = colorizer.voxel_color(s.z0, s.z1, s.color, &s.part, y);
                    model.set(lx, y as u32, ly, Some(Rgba([c[0], c[1], c[2], 255])));
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
    let colorizer = Colorizer::new(sprites);
    let mut out = Vec::new();
    for (zl, tiles) in levels {
        let (mut grid, z) = build_level(&tiles, sprites);
        crown_and_enclosure(&mut grid, &z, &tiles);
        for ((tx, ty), model) in fill(&grid, &colorizer) {
            out.push(((tx, ty, zl), model));
        }
    }
    out
}

// ----------------------------------------------------------------
// Colouring: the prototype's tex6 rules (export_models.py),
// plus the repaint() top unification it computed but never
// applied. All float/int semantics mirror the Python: round()
// is half-even, int() truncates, // floors.
// ----------------------------------------------------------------

/// Parts whose sprites are separate icons of one painted
/// surface: their colours unify onto the vehicle paint.
const PAINT_PARTS: [&str; 6] = ["door", "door_trunk", "halfboard", "trunk", "cargo_space", "windshield"];
/// Parts that carry a trim row under their top.
const PANELS: [&str; 9] = [
    "halfboard", "door", "door_trunk", "trunk", "cargo_space", "roof", "frame", "windshield",
    "xlframe",
];

fn lum(c: [u8; 3]) -> f64 {
    0.3 * c[0] as f64 + 0.5 * c[1] as f64 + 0.2 * c[2] as f64
}

fn shade(c: [u8; 3], f: f64) -> [u8; 3] {
    let m = |v: u8| py_round(v as f64 * f).clamp(0, 255) as u8;
    [m(c[0]), m(c[1]), m(c[2])]
}

/// One part's colour table from its sprite.
#[derive(Debug, Clone)]
struct PartTable {
    /// The 12 most frequent opaque colours (count order,
    /// first-seen wins ties — Python `Counter.most_common`).
    common: Vec<[u8; 3]>,
    dominant: [u8; 3],
    trim: [u8; 3],
}

fn part_table(spr: &RgbaImage) -> Option<PartTable> {
    let mut counts: Vec<([u8; 3], u32)> = Vec::new();
    for sy in 0..spr.height() {
        for sx in 0..spr.width() {
            let px = spr.get_pixel(sx, sy);
            if px[3] < 128 {
                continue;
            }
            let c = [px[0], px[1], px[2]];
            match counts.iter_mut().find(|(k, _)| *k == c) {
                Some((_, n)) => *n += 1,
                None => counts.push((c, 1)),
            }
        }
    }
    if counts.is_empty() {
        return None;
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1)); // stable: first-seen wins ties
    let common: Vec<[u8; 3]> = counts.iter().take(12).map(|(c, _)| *c).collect();
    let dominant = common[0];
    let trim = counts
        .iter()
        .take(40)
        .map(|(c, _)| *c)
        .find(|c| lum(*c) < 95.0)
        .unwrap_or_else(|| {
            let t = |v: u8| (v as f64 * 0.55) as u8; // int() truncates
            [t(dominant[0]), t(dominant[1]), t(dominant[2])]
        });
    Some(PartTable { common, dominant, trim })
}

/// The tex6 colorizer: per-part tables, the unified side
/// paint, and the vehicle paint for top unification.
pub struct Colorizer<'a> {
    sprites: &'a dyn Fn(&str) -> Option<RgbaImage>,
    cache: std::cell::RefCell<std::collections::HashMap<String, Option<PartTable>>>,
    /// Median achromatic dominant among PAINT_PARTS.
    paint: Option<[u8; 3]>,
    side_paint: [u8; 3],
    side_trim: [u8; 3],
}

impl<'a> Colorizer<'a> {
    pub fn new(sprites: &'a dyn Fn(&str) -> Option<RgbaImage>) -> Self {
        let mut cz = Colorizer {
            sprites,
            cache: Default::default(),
            paint: None,
            side_paint: [115, 115, 115],
            side_trim: [38, 38, 38],
        };
        // Vehicle paint: median of the achromatic PAINT_PARTS
        // dominants, by luminance (prototype's selection).
        let mut doms: Vec<[u8; 3]> = PAINT_PARTS
            .iter()
            .filter_map(|p| cz.table_for(p))
            .map(|t| t.dominant)
            .filter(|d| d.iter().max().unwrap() - d.iter().min().unwrap() <= 12)
            .collect();
        doms.sort_by(|a, b| lum(*a).partial_cmp(&lum(*b)).unwrap());
        cz.paint = doms.get(doms.len() / 2).copied();
        // Side paint: mean of the door and halfboard
        // dominants; side trim: the door's most common
        // non-dominant colour.
        if let (Some(door), Some(half)) = (cz.table_for("door"), cz.table_for("halfboard")) {
            cz.side_paint = [
                ((door.dominant[0] as u16 + half.dominant[0] as u16) / 2) as u8,
                ((door.dominant[1] as u16 + half.dominant[1] as u16) / 2) as u8,
                ((door.dominant[2] as u16 + half.dominant[2] as u16) / 2) as u8,
            ];
            cz.side_trim = door
                .common
                .iter()
                .find(|c| **c != door.dominant)
                .copied()
                .unwrap_or([38, 38, 38]);
        }
        cz
    }

    fn table_for(&self, part: &str) -> Option<PartTable> {
        if let Some(hit) = self.cache.borrow().get(part) {
            return hit.clone();
        }
        let tab = sprite_for(part, self.sprites).and_then(|spr| part_table(&spr));
        self.cache.borrow_mut().insert(part.to_string(), tab.clone());
        tab
    }

    /// The vehicle paint (None when no PAINT_PART is
    /// achromatic).
    pub fn paint(&self) -> Option<[u8; 3]> {
        self.paint
    }

    /// Shift an achromatic paint part's colour onto the
    /// vehicle paint, preserving its delta from the part's
    /// own dominant (the prototype's `repaint()`).
    fn repaint(&self, part: &str, c: [u8; 3]) -> [u8; 3] {
        let Some(paint) = self.paint else { return c };
        if !PAINT_PARTS.contains(&part) {
            return c;
        }
        let Some(tab) = self.table_for(part) else { return c };
        let dom = tab.dominant;
        if dom.iter().max().unwrap() - dom.iter().min().unwrap() > 12 {
            return c; // chromatic part: keeps its own colour
        }
        let f = |ch: usize| (paint[ch] as i32 + c[ch] as i32 - dom[ch] as i32).clamp(0, 255) as u8;
        [f(0), f(1), f(2)]
    }

    /// Colour of fine row `y` of one segment (prototype
    /// `voxel_color`): depth is in mockup units below the
    /// segment's top.
    pub fn voxel_color(&self, z0: f64, z1: f64, top_color: [u8; 3], part: &str, y: i32) -> [u8; 3] {
        let depth = z1 - (y + 1) as f64 / 2.0;
        let mut col;
        if part == "glass" {
            return GLASS;
        } else if part == "wheel" || part == "pillar" {
            return top_color;
        }
        let tab = self.table_for(part);
        if depth < 1.0 {
            col = match &tab {
                // Nearest table colour; first wins ties
                // (Python `min` semantics — Rust's
                // `min_by_key` would take the last).
                Some(t) => {
                    let mut best = t.common[0];
                    let mut best_d = i32::MAX;
                    for k in &t.common {
                        let d = (k[0] as i32 - top_color[0] as i32).pow(2)
                            + (k[1] as i32 - top_color[1] as i32).pow(2)
                            + (k[2] as i32 - top_color[2] as i32).pow(2);
                        if d < best_d {
                            best_d = d;
                            best = *k;
                        }
                    }
                    best
                }
                None => top_color,
            };
            // THE FIX: panel tops unify onto the vehicle
            // paint too, not just the sides.
            col = self.repaint(part, col);
        } else if tab.is_none() {
            col = shade(top_color, 0.8);
        } else {
            let t = tab.unwrap();
            let (mut dominant, mut trim) = (t.dominant, t.trim);
            if PAINT_PARTS.contains(&part) {
                dominant = self.side_paint;
                trim = self.side_trim;
            }
            if depth < 2.0 && PANELS.contains(&part) {
                col = trim;
            } else {
                let span = ((z1 - z0) - 2.0).max(1.0);
                let mut f = 1.0 - 0.3 * ((depth - 2.0) / span).clamp(0.0, 1.0);
                if depth >= (z1 - z0) - 1.0 {
                    f *= 0.85;
                }
                col = shade(dominant, f);
            }
        }
        // Underbody: below the body floor, non-wheel mass
        // reads as shadow, not grey boxes.
        if part != "wheel" && (y as f64) / 2.0 < 8.5 {
            col = shade(col, 0.3);
        }
        col
    }
}
