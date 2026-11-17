//! Stage G.3: parametric prop archetypes — real constructed
//! geometry for known furniture types, fitted to the sprite.
//!
//! The signed-off base pipeline ([`crate::props::solid_model`])
//! extrudes the sprite's silhouette: correct in outline, but a
//! PAINTED recess stays paint — a bookcase's shelves are surface
//! texture, not open compartments. Archetypes fix that for the
//! furniture types we know how to build: each archetype is a small
//! parametric construction (frame, boards, panels, glass, liquid)
//! on the same sub-cell grid as the solid model — footprint from
//! the front region's structure extent, depth from
//! [`Segmentation::depth_cells_for`], height from the front
//! region — textured from the sprite's own DE-SHADED palette
//! materials (albedo only, exactly as in `props`: the engine
//! provides the lighting), with the sprite's front pixels applied
//! as a decal on the front faces the archetype shares with the
//! sprite.
//!
//! Assignment is case-by-case, like the other Stage G overrides:
//! [`archetype_for`] consults the explicit [`ARCHETYPE_IDS`]
//! table. An id not in the table gets no archetype (the caller
//! keeps the solid model / extrusion). Served ids whose game
//! sprite resolves through a `looks_like` chain (the same
//! resolution the material pipeline uses) are fitted to the
//! resolved sprite: f_bunkbed -> f_bed, f_lab_bench -> f_counter,
//! f_storage_tank -> f_standing_tank, f_utility_shelf -> f_rack.
//!
//! ## What is fitted vs assumed, per archetype
//!
//! * **ShelfUnit** — FITTED: shelf board heights, from the front
//!   region's recess bands (rows whose interior mean L is shadow:
//!   a shelf opening's painted back/shadow). The boards sit at the
//!   bands; the compartments between them are genuinely open.
//!   ASSUMED: side/back panels are the prop's primary material
//!   (the Panel construction), board thickness = band height,
//!   plus a base board and a top board when the bands don't reach
//!   the frame's ends. Front decal on the frame (panels + board
//!   edges) only.
//! * **ContainmentTank** — FITTED: the liquid level, from the
//!   strongest horizontal band boundary in the front region's
//!   row-mean color profile (a real fill line when the sprite
//!   draws one); the glass color (palest blue-green palette
//!   cluster) and the frame color (darkest non-shadow cluster).
//!   ASSUMED: when no band boundary clears the step floor, the
//!   level is 55% of the height (a sealed metal tank like
//!   f_standing_tank draws no fill line); the frame is corner
//!   posts + top/bottom rings; the back is a plain panel; there
//!   is NO specimen — no gate sprite lets one be read cleanly
//!   (f_alien_pod, the one true containment sprite, does not
//!   segment: a pod on a stalk has no box-like reading, so it is
//!   deliberately NOT in the table).
//! * **ConsoleDesk** — FITTED: desk height (the top region's
//!   height becomes the console rise), the screen rectangles
//!   (from the bounding box of the sprite's accent-colored
//!   pixels — the drawn screen), the accent color itself (the
//!   front palette's green/cyan high-chroma cluster). ASSUMED:
//!   the console block occupies the back half of the depth, its
//!   front face is archetype material (the sprite's console face
//!   is panel + screen; the screen becomes a clean accent rect),
//!   a keyboard slab sits on the desk in front of the screens.
//!   The desk body's front keeps the sprite decal.
//! * **DissectionTable** — FITTED: materials (top region's stone
//!   primaries), the pedestal profile scales with the frame.
//!   ASSUMED: slab thickness 3, rim 2 cells wide standing 1 cell
//!   above a central channel, pedestal + wider plinth. The served
//!   id f_surgical_table has NO UltiCa sprite and no looks_like —
//!   the archetype is fitted/showcased on f_slab (a stone slab on
//!   a stepped base, the nearest real sprite).
//! * **Bunk** — FITTED: mattress color (brightest cluster) and
//!   frame wood (the front palette's wood-hue cluster), slab
//!   heights divided evenly across the height. ASSUMED: 2x2
//!   corner posts, slab thickness 2, ladder on the right end
//!   (rails + rungs every 3 rows). Levels come from the id
//!   (f_triple_bunkbed = 3) since the resolved sprite (f_bed)
//!   only ever shows one bed.
//!
//! Everything is deterministic: fixed iteration order, palettes
//! from the deterministic analyzer, grain fBm seeded from the
//! sprite's content hash.

use crate::analyze::{analyze_image, content_hash};
use crate::color::Oklab;
use crate::noise::Fbm;
use crate::props::{
    deshade_lab, front_structure_extent, grain_amp, lab_of, region_primaries, structure,
    top_row_for_z, weighted_median_l, PropModel, Segmentation, SHADOW_L,
};
use image::{Rgba, RgbaImage};
use std::ops::Range;

/// A parametric prop archetype (module docs). `Bunk` carries its
/// level count because the id knows it and the sprite cannot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    ShelfUnit,
    ContainmentTank,
    ConsoleDesk,
    DissectionTable,
    Bunk { levels: u8 },
}

/// The archetype id table: furniture id -> archetype. The
/// case-by-case layer, mirroring `props::BOUNDARY_OVERRIDES` —
/// an entry exists because the furniture type was reviewed.
/// (Ids whose sprite resolves via `looks_like` are fitted to the
/// resolved sprite; see module docs.)
pub const ARCHETYPE_IDS: &[(&str, Archetype)] = &[
    // Shelf units: bookcase + the rack family.
    ("f_bookcase", Archetype::ShelfUnit),
    ("f_rack", Archetype::ShelfUnit),
    ("f_utility_shelf", Archetype::ShelfUnit), // looks_like f_rack
    ("f_warehouse_shelf", Archetype::ShelfUnit), // looks_like f_utility_shelf
    ("f_vitrified_bookcase", Archetype::ShelfUnit), // no sprite yet
    ("f_vitrified_bookcase_talker", Archetype::ShelfUnit), // no sprite yet
    // Containment / liquid tanks.
    ("f_standing_tank", Archetype::ContainmentTank),
    ("f_storage_tank", Archetype::ContainmentTank), // looks_like f_standing_tank
    // Computer consoles.
    ("f_console", Archetype::ConsoleDesk),
    ("f_console_broken", Archetype::ConsoleDesk),
    ("f_aut_gas_console", Archetype::ConsoleDesk), // looks_like f_console
    ("f_aut_gas_console_o", Archetype::ConsoleDesk),
    // Dissection / surgical tables. f_surgical_table has no
    // sprite; f_slab is its fitted stand-in (module docs).
    ("f_surgical_table", Archetype::DissectionTable),
    ("f_slab", Archetype::DissectionTable),
    // Bunks (sprite resolves to f_bed / f_bed_down).
    ("f_bunkbed", Archetype::Bunk { levels: 2 }),
    ("f_bunkbed_down", Archetype::Bunk { levels: 2 }),
    ("f_triple_bunkbed", Archetype::Bunk { levels: 3 }),
];

/// The archetype serving a furniture id, if one has been
/// assigned (module docs). Case-by-case: no table entry, no
/// archetype.
pub fn archetype_for(id: &str) -> Option<Archetype> {
    ARCHETYPE_IDS
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, a)| *a)
}

/// Build the archetype model for a segmented sprite. Same grid
/// convention as [`crate::props::solid_model`]: size
/// `[sprite width, front height, depth_cells_for]`, y = 0 at the
/// object's bottom, z = depth-1 at the front.
pub fn build(archetype: Archetype, sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    match archetype {
        Archetype::ShelfUnit => build_shelf(sprite, seg),
        Archetype::ContainmentTank => build_tank(sprite, seg),
        Archetype::ConsoleDesk => build_console(sprite, seg),
        Archetype::DissectionTable => build_table(sprite, seg),
        Archetype::Bunk { levels } => build_bunk(sprite, seg, levels.max(1)),
    }
}

// ---------------------------------------------------------------------
// Shared fitting machinery
// ---------------------------------------------------------------------

/// The frame every archetype builds in: the solid model's grid.
struct Frame {
    fh: u32,
    d: u32,
    /// Front region's structure x-extent (inclusive).
    x0: u32,
    x1: u32,
    /// Sprite row of the front region's bottom (model y = 0).
    bot: u32,
}

fn frame(sprite: &RgbaImage, seg: &Segmentation) -> Frame {
    let (x0, x1) = front_structure_extent(sprite, seg);
    Frame {
        fh: seg.front_rows.end - seg.front_rows.start,
        d: seg.depth_cells_for(sprite.width()),
        x0,
        x1,
        bot: seg.front_rows.end - 1,
    }
}

fn rgba(lab: Oklab) -> Rgba<u8> {
    let [r, g, b] = lab.to_srgb_u8();
    Rgba([r, g, b, 255])
}

/// A region's palette with every cluster DE-SHADED the way
/// `props` de-shades the panel primary: each cluster's L is
/// re-anchored to the palette's weighted median L over the
/// non-shadow clusters, gain clamped to [0.80, 1.25]. Sorted by
/// weight descending (the analyzer's order).
fn deshaded_palette(sprite: &RgbaImage, rows: &Range<u32>, k: usize) -> Vec<(Oklab, f32)> {
    let crop = image::imageops::crop_imm(
        sprite,
        0,
        rows.start,
        sprite.width(),
        rows.end - rows.start,
    )
    .to_image();
    let a = analyze_image(&crop, k);
    let non_shadow: Vec<(f32, f32)> = a
        .palette
        .iter()
        .filter(|p| p.lab.l >= SHADOW_L)
        .map(|p| (p.lab.l, p.weight))
        .collect();
    let median = if non_shadow.is_empty() {
        None
    } else {
        Some(weighted_median_l(&non_shadow))
    };
    a.palette
        .iter()
        .map(|p| {
            let lab = match median {
                Some(m) if p.lab.l > 0.0 => {
                    deshade_lab(p.lab, (m / p.lab.l).clamp(0.80, 1.25))
                }
                _ => p.lab,
            };
            (lab, p.weight)
        })
        .collect()
}

/// The palette's largest non-shadow cluster (the material), or
/// the largest cluster overall when all are shadow.
fn largest_non_shadow(pal: &[(Oklab, f32)]) -> Oklab {
    pal.iter()
        .find(|(l, _)| l.l >= SHADOW_L)
        .or_else(|| pal.first())
        .map(|(l, _)| *l)
        .unwrap_or(Oklab { l: 0.5, a: 0.0, b: 0.0 })
}

/// The palette's darkest non-shadow cluster (fittings, frames).
fn darkest_non_shadow(pal: &[(Oklab, f32)]) -> Oklab {
    pal.iter()
        .filter(|(l, _)| l.l >= SHADOW_L)
        .min_by(|a, b| a.0.l.partial_cmp(&b.0.l).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(l, _)| *l)
        .unwrap_or_else(|| largest_non_shadow(pal))
}

/// The palette's brightest cluster (mattresses, glass fallback).
fn brightest(pal: &[(Oklab, f32)]) -> Oklab {
    pal.iter()
        .max_by(|a, b| a.0.l.partial_cmp(&b.0.l).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(l, _)| *l)
        .unwrap_or(Oklab { l: 0.9, a: 0.0, b: 0.0 })
}

/// The palette's second-largest non-shadow cluster (secondary
/// material: console panel, channel stone), falling back to the
/// largest.
fn second_non_shadow(pal: &[(Oklab, f32)]) -> Oklab {
    let mut it = pal.iter().filter(|(l, _)| l.l >= SHADOW_L);
    match (it.next(), it.next()) {
        (_, Some((l, _))) => *l,
        (Some((l, _)), None) => *l,
        _ => largest_non_shadow(pal),
    }
}

/// Deterministic panel grain, the `props::BackPanels` recipe:
/// the base color's L modulated by stretched fBm (seeded from
/// the sprite's content hash), amplitude by material
/// ([`grain_amp`]).
struct Grain {
    fbm: Fbm,
}

impl Grain {
    fn new(sprite: &RgbaImage) -> Self {
        Grain {
            fbm: Fbm::new(content_hash(sprite.as_raw()), 16).octaves(2),
        }
    }
    fn color(&self, base: Oklab, u: f32, v: f32) -> Rgba<u8> {
        let n = self.fbm.sample(u, v);
        let t = ((n - 0.5) * 2.0).clamp(-1.0, 1.0);
        let mut lab = base;
        lab.l = (lab.l * (1.0 + grain_amp(base) * t)).clamp(0.02, 0.98);
        rgba(lab)
    }
    /// Back/panel grain: streaks run horizontally, sample (x/6, y).
    fn back(&self, base: Oklab, x: u32, y: u32) -> Rgba<u8> {
        self.color(base, x as f32 / 6.0, y as f32)
    }
    /// Flank grain: streaks run vertically, sample (z, y/6).
    fn side(&self, base: Oklab, z: u32, y: u32) -> Rgba<u8> {
        self.color(base, z as f32, y as f32 / 6.0)
    }
}

/// The sprite's front pixel at model (x, y) when it is structure
/// (real paint) — the decal source.
fn decal(sprite: &RgbaImage, fr: &Frame, x: u32, y: u32) -> Option<Rgba<u8>> {
    let px = *sprite.get_pixel(x, fr.bot - y);
    if structure(px) {
        Some(px)
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// ShelfUnit
// ---------------------------------------------------------------------

/// Fitted shelf geometry: the board rows (inclusive model-y
/// ranges, ascending) — boards sit at the front's recess bands,
/// plus a base board and a top board when the bands don't reach
/// the frame's ends (module docs).
#[derive(Debug, Clone, PartialEq)]
pub struct ShelfFit {
    pub board_rows: Vec<(u32, u32)>,
}

pub fn shelf_fit(sprite: &RgbaImage, seg: &Segmentation) -> ShelfFit {
    let fr = frame(sprite, seg);
    // Interior mean L per front row (structure pixels, extent
    // trimmed by 2 — the frame shouldn't define the recess).
    let (tx0, tx1) = if fr.x1 >= fr.x0 + 4 {
        (fr.x0 + 2, fr.x1 - 2)
    } else {
        (fr.x0, fr.x1)
    };
    let mut shadow_row = vec![false; fr.fh as usize];
    for y in 0..fr.fh {
        let r = fr.bot - y;
        let (mut s, mut n) = (0.0f32, 0u32);
        for x in tx0..=tx1 {
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                s += lab_of(px).l;
                n += 1;
            }
        }
        shadow_row[y as usize] = n > 0 && (s / n as f32) < SHADOW_L;
    }
    // Maximal consecutive shadow runs = recess bands = boards.
    let mut boards: Vec<(u32, u32)> = Vec::new();
    let mut y = 0u32;
    while y < fr.fh {
        if shadow_row[y as usize] {
            let start = y;
            while y < fr.fh && shadow_row[y as usize] {
                y += 1;
            }
            boards.push((start, y - 1));
        } else {
            y += 1;
        }
    }
    if boards.is_empty() {
        // No readable recesses: evenly spaced boards every ~7
        // rows (assumed fallback, module docs).
        let mut yy = 0u32;
        while yy < fr.fh {
            boards.push((yy, (yy + 1).min(fr.fh - 1)));
            yy += 7;
        }
    }
    // A base board and a top board when the bands don't reach.
    if boards.first().map_or(true, |(b, _)| *b > 1) {
        boards.insert(0, (0, 1.min(fr.fh - 1)));
    }
    if boards.last().map_or(true, |(_, t)| *t < fr.fh - 1) {
        boards.push((fr.fh.saturating_sub(2), fr.fh - 1));
    }
    ShelfFit { board_rows: boards }
}

fn build_shelf(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = shelf_fit(sprite, seg);
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let mut m = PropModel::new([w, fh, d]);
    let is_board = |y: u32| fit.board_rows.iter().any(|(b, t)| y >= *b && y <= *t);
    // Side panels: 2 cells thick at the frame's x extremes, full
    // height and depth (Panel construction).
    for y in 0..fh {
        for z in 0..d {
            for x in [fr.x0, fr.x0 + 1, fr.x1 - 1, fr.x1] {
                m.set(x, y, z, Some(grain.side(panel, z, y)));
            }
        }
    }
    // Back panel.
    for y in 0..fh {
        for x in fr.x0..=fr.x1 {
            m.set(x, y, 0, Some(grain.back(panel, x, y)));
        }
    }
    // Shelf boards: full frame width and depth at the fitted rows.
    for &(b, t) in &fit.board_rows {
        for y in b..=t {
            for z in 0..d {
                for x in fr.x0..=fr.x1 {
                    m.set(x, y, z, Some(grain.back(panel, x, y)));
                }
            }
        }
    }
    // Top surface: the top board's top face takes the top
    // region's un-projection (evidence), as in the solid model.
    let ytop = fit.board_rows.last().map(|(_, t)| *t).unwrap_or(fh - 1);
    for x in fr.x0..=fr.x1 {
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            let px = *sprite.get_pixel(x, sr);
            if structure(px) {
                m.set(x, ytop, z, Some(px));
            }
        }
    }
    // Front decal: the frame's front plane (side panels + board
    // edges) takes the sprite's front pixels; compartment
    // openings stay empty — that is the archetype's point.
    for y in 0..fh {
        for x in fr.x0..=fr.x1 {
            let frame_cell = is_board(y) || x <= fr.x0 + 1 || x >= fr.x1 - 1;
            if frame_cell {
                if let Some(px) = decal(sprite, &fr, x, y) {
                    m.set(x, y, d - 1, Some(px));
                }
            }
        }
    }
    m
}

// ---------------------------------------------------------------------
// ContainmentTank
// ---------------------------------------------------------------------

/// Fitted tank parameters: the liquid's top row (model y), and
/// the frame / glass / liquid colors (de-shaded palette picks).
#[derive(Debug, Clone, PartialEq)]
pub struct TankFit {
    pub liquid_top: u32,
    pub frame: Rgba<u8>,
    pub glass: Rgba<u8>,
    pub liquid: Rgba<u8>,
}

/// Minimum row-mean color step that reads as a drawn fill line.
const LIQUID_STEP_FLOOR: f32 = 0.05;

pub fn tank_fit(sprite: &RgbaImage, seg: &Segmentation) -> TankFit {
    let fr = frame(sprite, seg);
    let pal_front = deshaded_palette(sprite, &seg.front_rows, 6);
    let pal_all = deshaded_palette(sprite, &seg.silhouette_rows, 6);
    let frame_c = darkest_non_shadow(&pal_front);
    // Glass: the palest blue-green cluster (OKLab blue-green
    // quadrant: a <= 0.02 and b <= 0.02 — neutral counts, these
    // sprites are mostly achromatic); fallback the palest overall.
    let glass_c = pal_all
        .iter()
        .filter(|(l, _)| l.a <= 0.02 && l.b <= 0.02)
        .max_by(|a, b| a.0.l.partial_cmp(&b.0.l).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(l, _)| *l)
        .unwrap_or_else(|| brightest(&pal_all));
    // Liquid level: strongest row-mean color boundary in the
    // front region's middle band (model y in [fh/5, 4fh/5]).
    let (tx0, tx1) = if fr.x1 >= fr.x0 + 4 {
        (fr.x0 + 2, fr.x1 - 2)
    } else {
        (fr.x0, fr.x1)
    };
    let row_mean = |r: u32| -> Option<Oklab> {
        let (mut s, mut n) = (Oklab { l: 0.0, a: 0.0, b: 0.0 }, 0u32);
        for x in tx0..=tx1 {
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                let l = lab_of(px);
                s.l += l.l;
                s.a += l.a;
                s.b += l.b;
                n += 1;
            }
        }
        if n > 0 {
            Some(Oklab {
                l: s.l / n as f32,
                a: s.a / n as f32,
                b: s.b / n as f32,
            })
        } else {
            None
        }
    };
    let mut best: Option<(f32, u32)> = None; // (step, sprite boundary row)
    let lo_y = fr.fh / 5;
    let hi_y = (fr.fh * 4 / 5).max(lo_y);
    for y in lo_y..=hi_y {
        // Boundary between sprite rows bot-y (above) and
        // bot-y+1 (below): liquid occupies rows >= that row.
        let (ra, rb) = (fr.bot - y, fr.bot - y + 1);
        if rb >= seg.front_rows.end {
            continue;
        }
        if let (Some(a), Some(b)) = (row_mean(ra), row_mean(rb)) {
            let step = a.distance(b);
            if best.map_or(true, |(bs, _)| step > bs) {
                best = Some((step, rb));
            }
        }
    }
    let (liquid_top, liquid_rows) = match best {
        Some((step, rb)) if step >= LIQUID_STEP_FLOOR => {
            (fr.bot - rb, rb..seg.front_rows.end)
        }
        _ => {
            let top = ((fr.fh as f32 * 0.55).round() as u32).clamp(1, fr.fh - 1);
            (top, (fr.bot - top)..seg.front_rows.end)
        }
    };
    // Liquid color: the dominant material of the front below the
    // line (what the sprite shows there), de-shaded.
    let liquid_c = if liquid_rows.end > liquid_rows.start {
        largest_non_shadow(&deshaded_palette(sprite, &liquid_rows, 5))
    } else {
        largest_non_shadow(&pal_front)
    };
    TankFit {
        liquid_top,
        frame: rgba(frame_c),
        glass: rgba(glass_c),
        liquid: rgba(liquid_c),
    }
}

fn build_tank(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = tank_fit(sprite, seg);
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let mut m = PropModel::new([w, fh, d]);
    // Liquid fill: the interior below the fitted level.
    for y in 0..=fit.liquid_top.min(fh - 1) {
        for z in 1..d.saturating_sub(1) {
            for x in fr.x0 + 2..fr.x1 - 1 {
                m.set(x, y, z, Some(fit.liquid));
            }
        }
    }
    // Back panel (a tank's back is plain material, not glass).
    for y in 0..fh {
        for x in fr.x0..=fr.x1 {
            m.set(x, y, 0, Some(grain.back(panel, x, y)));
        }
    }
    // Top lid: the top region un-projected across the frame.
    for x in fr.x0..=fr.x1 {
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            let px = *sprite.get_pixel(x, sr);
            if structure(px) {
                m.set(x, fh - 1, z, Some(px));
            }
        }
    }
    // Glass: front plane and both side planes, inside the frame.
    if fh > 4 && d > 4 {
        for y in 2..fh - 2 {
            for x in fr.x0 + 2..fr.x1 - 1 {
                m.set(x, y, d - 1, Some(fit.glass));
            }
            for z in 2..d - 2 {
                m.set(fr.x0, y, z, Some(fit.glass));
                m.set(fr.x1, y, z, Some(fit.glass));
            }
        }
    }
    // Frame: corner posts + top/bottom rings.
    for y in 0..fh {
        for z in [0, 1, d - 2, d - 1] {
            for x in [fr.x0, fr.x0 + 1, fr.x1 - 1, fr.x1] {
                m.set(x, y, z, Some(fit.frame));
            }
        }
    }
    for y in [0, 1, fh - 2, fh - 1] {
        for z in 0..d {
            for x in fr.x0..=fr.x1 {
                if z < 2 || z >= d - 2 || x <= fr.x0 + 1 || x >= fr.x1 - 1 {
                    m.set(x, y, z, Some(fit.frame));
                }
            }
        }
    }
    m
}

// ---------------------------------------------------------------------
// ConsoleDesk
// ---------------------------------------------------------------------

/// Fitted console parameters: the desk height (model rows below
/// the console rise), the screen rectangles on the console's
/// front face as inclusive `[x0, y0, x1, y1]` model coords, and
/// the accent (screen) color.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleFit {
    pub desk_h: u32,
    pub screens: Vec<[u32; 4]>,
    pub accent: Rgba<u8>,
}

pub fn console_fit(sprite: &RgbaImage, seg: &Segmentation) -> ConsoleFit {
    let fr = frame(sprite, seg);
    let top_h = seg.top_rows.end - seg.top_rows.start;
    // The top region's height becomes the console rise.
    let rise = top_h.clamp(4, (fr.fh / 2).max(4));
    let desk_h = fr.fh.saturating_sub(rise).max(4).min(fr.fh - 4);
    // Accent: the front palette's green/cyan high-chroma cluster
    // (raw palette — the bounding-box match below runs against
    // raw sprite pixels; the placed color is the de-shaded pick).
    let crop = image::imageops::crop_imm(
        sprite,
        0,
        seg.front_rows.start,
        sprite.width(),
        seg.front_rows.end - seg.front_rows.start,
    )
    .to_image();
    let raw = analyze_image(&crop, 6);
    let chroma = |l: &Oklab| (l.a * l.a + l.b * l.b).sqrt();
    let raw_accent = raw
        .palette
        .iter()
        .filter(|p| p.lab.a < -0.03)
        .max_by(|a, b| {
            chroma(&a.lab)
                .partial_cmp(&chroma(&b.lab))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .or_else(|| {
            raw.palette.iter().max_by(|a, b| {
                chroma(&a.lab)
                    .partial_cmp(&chroma(&b.lab))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
        .map(|p| p.lab);
    let pal_front = deshaded_palette(sprite, &seg.front_rows, 6);
    let accent_c = pal_front
        .iter()
        .filter(|(l, _)| l.a < -0.03)
        .max_by(|a, b| {
            chroma(&a.0)
                .partial_cmp(&chroma(&b.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(l, _)| *l)
        .unwrap_or_else(|| brightest(&pal_front));
    // Screen bounding box: sprite pixels matching the raw accent.
    let mut bbox: Option<(u32, u32, u32, u32)> = None; // x0, top row, x1, bottom row
    if let Some(ac) = raw_accent {
        for tol in [0.10f32, 0.14] {
            let mut bb = (u32::MAX, u32::MAX, 0u32, 0u32);
            let mut n = 0u32;
            for r in seg.silhouette_rows.clone() {
                for x in 0..sprite.width() {
                    let px = *sprite.get_pixel(x, r);
                    if structure(px) && lab_of(px).distance(ac) <= tol {
                        bb.0 = bb.0.min(x);
                        bb.1 = bb.1.min(r);
                        bb.2 = bb.2.max(x);
                        bb.3 = bb.3.max(r);
                        n += 1;
                    }
                }
            }
            if n >= 8 {
                bbox = Some(bb);
                break;
            }
        }
    }
    let (mut sx0, mut sx1, mut sy0, mut sy1) = match bbox {
        Some((bx0, rtop, bx1, rbot)) => (
            bx0.max(fr.x0 + 2).min(fr.x1.saturating_sub(2)),
            bx1.min(fr.x1.saturating_sub(2)).max(fr.x0 + 2),
            (fr.bot - rbot).max(desk_h + 1).min(fr.fh - 2),
            (fr.bot - rtop).min(fr.fh - 2).max(desk_h + 1),
        ),
        // No readable accent: one centered screen on the console.
        None => (
            fr.x0 + (fr.x1 - fr.x0) / 4,
            fr.x1 - (fr.x1 - fr.x0) / 4,
            desk_h + 2,
            fr.fh - 3,
        ),
    };
    if sx1 < sx0 {
        std::mem::swap(&mut sx0, &mut sx1);
    }
    if sy1 < sy0 {
        std::mem::swap(&mut sy0, &mut sy1);
    }
    let ext_w = fr.x1 - fr.x0 + 1;
    let screens = if sx1 - sx0 + 1 > (ext_w as f32 * 0.55) as u32 && sx1 - sx0 >= 9 {
        let mid = (sx0 + sx1) / 2;
        vec![
            [sx0, sy0, mid - 2, sy1],
            [mid + 2, sy0, sx1, sy1],
        ]
    } else {
        vec![[sx0, sy0, sx1, sy1]]
    };
    ConsoleFit {
        desk_h,
        screens,
        accent: rgba(accent_c),
    }
}

fn build_console(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = console_fit(sprite, seg);
    let pal_front = deshaded_palette(sprite, &seg.front_rows, 6);
    let (_, body_panel) = region_primaries(sprite, &seg.front_rows);
    let panel2 = second_non_shadow(&pal_front);
    let dark = darkest_non_shadow(&pal_front);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let desk_h = fit.desk_h.min(fh - 1);
    let console_depth = (d / 2).clamp(3, d.saturating_sub(2).max(3));
    let mut m = PropModel::new([w, fh, d]);
    // Desk body: solid block, panel construction; the front
    // plane is the sprite decal (the archetype shares that face
    // with the sprite).
    for y in 0..desk_h {
        for z in 0..d {
            for x in fr.x0..=fr.x1 {
                let c = if z == d - 1 {
                    decal(sprite, &fr, x, y)
                        .unwrap_or_else(|| grain.back(body_panel, x, y))
                } else if z == 0 {
                    grain.back(body_panel, x, y)
                } else if x == fr.x0 || x == fr.x1 {
                    grain.side(body_panel, z, y)
                } else if y == desk_h - 1 && z >= console_depth {
                    rgba(panel2) // the desk work surface
                } else {
                    rgba(body_panel)
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Console block: back half of the depth, rising to the full
    // height. Its front face is archetype material + screens.
    for y in desk_h..fh {
        for z in 0..console_depth {
            for x in fr.x0..=fr.x1 {
                let c = if z == 0 {
                    grain.back(body_panel, x, y)
                } else if x == fr.x0 || x == fr.x1 {
                    grain.side(body_panel, z, y)
                } else {
                    rgba(panel2)
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Console top: the top region un-projected (evidence).
    for x in fr.x0..=fr.x1 {
        for z in 0..console_depth - 1 {
            let sr = top_row_for_z(seg, d, z);
            let px = *sprite.get_pixel(x, sr);
            if structure(px) {
                m.set(x, fh - 1, z, Some(px));
            }
        }
    }
    // Screens on the console front face (z = console_depth - 1).
    let face_z = console_depth - 1;
    for s in &fit.screens {
        for y in s[1]..=s[3] {
            for x in s[0]..=s[2] {
                m.set(x, y, face_z, Some(fit.accent));
            }
        }
    }
    // Keyboard slab: on the desk, in front of the screens.
    let (kx0, kx1) = fit
        .screens
        .iter()
        .fold((u32::MAX, 0u32), |(a, b), s| (a.min(s[0]), b.max(s[2])));
    if kx0 <= kx1 && desk_h < fh {
        for z in (console_depth + 1)..=(console_depth + 3).min(d - 1) {
            for x in kx0..=kx1 {
                m.set(x, desk_h, z, Some(rgba(dark)));
            }
        }
    }
    m
}

// ---------------------------------------------------------------------
// DissectionTable
// ---------------------------------------------------------------------

/// Fitted table parameters: the slab's top row (= model height
/// - 1) and the channel floor row (one below the rim).
#[derive(Debug, Clone, PartialEq)]
pub struct TableFit {
    pub rim_y: u32,
    pub channel_y: u32,
}

pub fn table_fit(sprite: &RgbaImage, seg: &Segmentation) -> TableFit {
    let fr = frame(sprite, seg);
    TableFit {
        rim_y: fr.fh - 1,
        channel_y: fr.fh.saturating_sub(2),
    }
}

fn build_table(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let pal_top = deshaded_palette(sprite, &seg.top_rows, 6);
    let stone = largest_non_shadow(&pal_top);
    let channel_c = second_non_shadow(&pal_top);
    let (_, base_panel) = region_primaries(sprite, &seg.front_rows);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let slab_thick = if fh >= 9 { 3 } else { (fh / 3).max(1) };
    let slab_bot = fh - slab_thick;
    let mut m = PropModel::new([w, fh, d]);
    // Pedestal + plinth (centered, stepped scales — assumed
    // profile, module docs).
    let cx = (fr.x0 + fr.x1) as f32 / 2.0;
    let cz = (d - 1) as f32 / 2.0;
    let ext_w = (fr.x1 - fr.x0 + 1) as f32;
    for y in 0..slab_bot {
        let scale = if y < 2 { 0.75 } else { 0.55 };
        let hw = ext_w * scale / 2.0;
        let hd = d as f32 * scale / 2.0;
        for z in 0..d {
            for x in fr.x0..=fr.x1 {
                if (x as f32 - cx).abs() <= hw && (z as f32 - cz).abs() <= hd {
                    let c = if z == 0 || x == fr.x0 || x == fr.x1 {
                        grain.back(base_panel, x, y)
                    } else {
                        rgba(base_panel)
                    };
                    m.set(x, y, z, Some(c));
                }
            }
        }
    }
    // Slab body (below the rim row): full footprint. Interior
    // top cells are the channel floor.
    for y in slab_bot..fh - 1 {
        for z in 0..d {
            for x in fr.x0..=fr.x1 {
                let interior_top =
                    y == fh - 2 && x > fr.x0 + 1 && x < fr.x1 - 1 && z > 1 && z < d - 2;
                let c = if interior_top {
                    rgba(channel_c)
                } else {
                    rgba(stone)
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Rim: the top row's border cells, standing 1 above the
    // channel.
    for z in 0..d {
        for x in fr.x0..=fr.x1 {
            if x <= fr.x0 + 1 || x >= fr.x1 - 1 || z <= 1 || z >= d - 2 {
                m.set(x, fh - 1, z, Some(rgba(stone)));
            }
        }
    }
    // Slab front edge: the sprite's slab-edge rows as decal.
    for y in slab_bot..fh {
        for x in fr.x0..=fr.x1 {
            if m.get(x, y, d - 1).is_some() {
                if let Some(px) = decal(sprite, &fr, x, y) {
                    m.set(x, y, d - 1, Some(px));
                }
            }
        }
    }
    m
}

// ---------------------------------------------------------------------
// Bunk
// ---------------------------------------------------------------------

/// Fitted bunk parameters: the slab top rows (ascending), the
/// mattress color and the frame (wood) color.
#[derive(Debug, Clone, PartialEq)]
pub struct BunkFit {
    pub slab_tops: Vec<u32>,
    pub mattress: Rgba<u8>,
    pub frame_color: Rgba<u8>,
}

pub fn bunk_fit(sprite: &RgbaImage, seg: &Segmentation, levels: u8) -> BunkFit {
    let fr = frame(sprite, seg);
    let pal_front = deshaded_palette(sprite, &seg.front_rows, 6);
    let pal_all = deshaded_palette(sprite, &seg.silhouette_rows, 6);
    // Frame wood: the wood-hue cluster (the grain hue window)
    // with the most weight; fallback the front primary.
    let wood = pal_front
        .iter()
        .filter(|(l, _)| {
            let chroma = (l.a * l.a + l.b * l.b).sqrt();
            let hue = l.b.atan2(l.a).to_degrees();
            chroma >= 0.03 && (35.0..=95.0).contains(&hue) && l.l >= SHADOW_L
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(l, _)| *l)
        .unwrap_or_else(|| region_primaries(sprite, &seg.front_rows).1);
    let mattress = brightest(&pal_all);
    let levels = levels.max(1) as u32;
    let mut slab_tops = Vec::new();
    for i in 0..levels {
        let top = if i == levels - 1 {
            fr.fh - 1
        } else {
            ((fr.fh * (i + 1)) / (levels + 1)).max(2)
        };
        slab_tops.push(top);
    }
    BunkFit {
        slab_tops,
        mattress: rgba(mattress),
        frame_color: rgba(wood),
    }
}

fn build_bunk(sprite: &RgbaImage, seg: &Segmentation, levels: u8) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = bunk_fit(sprite, seg, levels);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let mut m = PropModel::new([w, fh, d]);
    let wood = fit.frame_color;
    // Corner posts (2x2), full height.
    for y in 0..fh {
        for z in [0, 1, d - 2, d - 1] {
            for x in [fr.x0, fr.x0 + 1, fr.x1 - 1, fr.x1] {
                m.set(x, y, z, Some(wood));
            }
        }
    }
    // Slabs: frame border + mattress top, 2 thick.
    for &top in &fit.slab_tops {
        for y in top.saturating_sub(1)..=top {
            for z in 0..d {
                for x in fr.x0..=fr.x1 {
                    let border =
                        x <= fr.x0 + 1 || x >= fr.x1 - 1 || z <= 1 || z >= d - 2;
                    let c = if y == top && !border {
                        fit.mattress
                    } else {
                        wood
                    };
                    m.set(x, y, z, Some(c));
                }
            }
        }
    }
    // Ladder on the right end: rails + rungs every 3 rows,
    // between the corner posts.
    if d >= 8 {
        let (z0, z1) = (3, d - 4);
        for y in 0..fh {
            m.set(fr.x1, y, z0, Some(wood));
            m.set(fr.x1, y, z1, Some(wood));
            if y % 3 == 1 {
                for z in z0..=z1 {
                    m.set(fr.x1, y, z, Some(wood));
                }
            }
        }
    }
    m
}
