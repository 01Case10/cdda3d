//! Stage G.3: parametric prop archetypes — real constructed
//! geometry for known furniture types, fitted to the sprite.
//!
//! The signed-off base pipeline ([`crate::props::solid_model`])
//! extrudes the sprite's silhouette: correct in outline, but a
//! PAINTED recess stays paint — a bookcase's shelves are surface
//! texture, not open compartments. Archetypes fix that for the
//! furniture types we know how to build: each archetype is a
//! small parametric construction (frame, boards, panels) on the
//! same sub-cell grid as the solid model — footprint from the
//! front region's structure extent, depth from
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
//! resolved sprite: f_lab_bench -> f_counter,
//! f_utility_shelf -> f_rack.
//!
//! ## The G.3 rework reduced this set (pipeline §24)
//!
//! The first archetype round assumed every sprite is a front
//! view; Joey's review kept ONLY the ShelfUnit. The other
//! archetypes were deleted and their ids rerouted by VIEW
//! ([`crate::props::model_for`]): the bed chain and f_slab are
//! top-down sprites (framed-surface / terrace builders in
//! `props`), f_standing_tank / f_storage_tank are end-on (the
//! EndOn cylinder in `props`). f_surgical_table has no sprite
//! and no archetype now. What remains here:
//!
//! * **ShelfUnit** — unchanged, signed off. FITTED: shelf board
//!   heights, from the front region's recess bands (rows whose
//!   interior mean L is shadow: a shelf opening's painted
//!   back/shadow). The boards sit at the bands; the compartments
//!   between them are genuinely open. ASSUMED: side/back panels
//!   are the prop's primary material (the Panel construction),
//!   board thickness = band height, plus a base board and a top
//!   board when the bands don't reach the frame's ends. Front
//!   decal on the frame (panels + board edges) only.
//! * **ConsoleDesk** — REWORKED to be faithful instead of
//!   reconstructed. The old builder replaced the console's front
//!   with palette panels + a clean accent screen rectangle and
//!   lost the sprite's detail; the sprite's front is already a
//!   complete drawing of the console, so the front plane is now
//!   the sprite's front region VERBATIM across the full front
//!   (the solid model's front rule). What the archetype adds
//!   over the solid model is DEPTH STRUCTURE: the console rise
//!   (rows at/above the fitted desk height — the top region's
//!   height) keeps the full segmentation depth, the desk/base
//!   rows below it step back to ~55% depth, and the base's open
//!   compartment — the dark knee-hole the sprite draws in its
//!   bottom rows (opaque near-black recess rows over a
//!   transparent cutout) — is modeled OPEN: the opening's
//!   columns are empty through the depth, the side cabinets
//!   solid. FITTED: desk height, base depth (55% of full,
//!   rounded), the opening's rows and columns (contiguous void
//!   runs from the bottom, strictly inside the frame's extent).
//!
//! Everything is deterministic: fixed iteration order, palettes
//! from the deterministic analyzer, grain fBm seeded from the
//! sprite's content hash.

use crate::analyze::content_hash;
use crate::color::Oklab;
use crate::noise::Fbm;
use crate::props::{
    front_structure_extent, grain_amp, lab_of, region_primaries, structure, top_row_for_z,
    PropModel, Segmentation, SHADOW_L,
};
use image::{Rgba, RgbaImage};
use std::ops::Range;

/// A parametric prop archetype (module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    ShelfUnit,
    ConsoleDesk,
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
    // Computer consoles.
    ("f_console", Archetype::ConsoleDesk),
    ("f_console_broken", Archetype::ConsoleDesk),
    ("f_aut_gas_console", Archetype::ConsoleDesk), // looks_like f_console
    ("f_aut_gas_console_o", Archetype::ConsoleDesk),
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
        Archetype::ConsoleDesk => build_console(sprite, seg),
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
// ConsoleDesk
// ---------------------------------------------------------------------

/// A base compartment modeled OPEN (module docs): the column
/// range (model x) and row range (model y) that stay empty
/// through the model's depth.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleOpening {
    pub cols: Range<u32>,
    pub rows: Range<u32>,
}

/// Fitted console parameters: the desk height (model rows below
/// the console rise), the base rows' stepped depth, and the
/// base's open compartment when the sprite draws one.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleFit {
    pub desk_h: u32,
    pub base_depth: u32,
    pub opening: Option<ConsoleOpening>,
}

/// A front pixel that reads as VOID for the opening detector:
/// not structure (the sprite cuts the compartment out) or
/// opaque near-black (the sprite paints it as a dark recess).
fn console_void(px: Rgba<u8>) -> bool {
    !structure(px) || lab_of(px).l < 0.20
}

pub fn console_fit(sprite: &RgbaImage, seg: &Segmentation) -> ConsoleFit {
    let fr = frame(sprite, seg);
    let top_h = seg.top_rows.end - seg.top_rows.start;
    let desk_h = fr.fh.saturating_sub(top_h).max(1).min(fr.fh);
    let base_depth = ((0.55 * fr.d as f32).round() as u32).max(2).min(fr.d);
    // Opening: scan rows from the bottom; a row qualifies when
    // its longest VOID run is >= 6 columns wide and strictly
    // inside the frame's extent (a cabinet on both sides). The
    // scan stops at the first non-qualifying row. The opening's
    // columns are void in at least half of the qualifying rows;
    // of the contiguous qualifying column runs, the one nearest
    // the frame's center is the opening.
    let w = sprite.width();
    let mut rows = 0u32;
    let mut col_void = vec![0u32; w as usize];
    for y in 0..desk_h {
        let r = fr.bot - y;
        let (mut run_start, mut run_len) = (0u32, 0u32);
        let mut best: Option<(u32, u32)> = None; // (start, len)
        for x in 0..w {
            if console_void(*sprite.get_pixel(x, r)) {
                if run_len == 0 {
                    run_start = x;
                }
                run_len += 1;
                let interior = run_start > fr.x0 && x < fr.x1;
                if interior && best.map_or(true, |(_, bl)| run_len > bl) {
                    best = Some((run_start, run_len));
                }
            } else {
                run_len = 0;
            }
        }
        match best {
            Some((_, bl)) if bl >= 6 => {
                rows += 1;
                for x in 0..w {
                    if console_void(*sprite.get_pixel(x, r)) {
                        col_void[x as usize] += 1;
                    }
                }
            }
            _ => break,
        }
    }
    let opening = if rows >= 2 {
        let need = rows.div_ceil(2);
        let center = (fr.x0 + fr.x1) as f32 / 2.0;
        let mut runs: Vec<Range<u32>> = Vec::new();
        let mut x = 0u32;
        while x < w {
            if col_void[x as usize] >= need {
                let s = x;
                while x < w && col_void[x as usize] >= need {
                    x += 1;
                }
                runs.push(s..x);
            } else {
                x += 1;
            }
        }
        runs.into_iter()
            .filter(|r| r.end - r.start >= 6)
            .min_by(|a, b| {
                let ca = ((a.start + a.end - 1) as f32 / 2.0 - center).abs();
                let cb = ((b.start + b.end - 1) as f32 / 2.0 - center).abs();
                ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|cols| ConsoleOpening { cols, rows: 0..rows })
    } else {
        None
    };
    ConsoleFit { desk_h, base_depth, opening }
}

fn build_console(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = console_fit(sprite, seg);
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let depth_of = |y: u32| if y >= fit.desk_h { d } else { fit.base_depth };
    let mut m = PropModel::new([w, fh, d]);
    // The solid model's fill, with the depth stepped per row:
    // front plane verbatim, back plane the panel grain, flanks
    // the side grain, interior the front pixel (unseen).
    for y in 0..fh {
        let r = fr.bot - y;
        let (mut rx0, mut rx1) = (w, 0u32);
        let mut any = false;
        for x in 0..w {
            if structure(*sprite.get_pixel(x, r)) {
                rx0 = rx0.min(x);
                rx1 = rx1.max(x);
                any = true;
            }
        }
        if !any {
            continue;
        }
        let zd = depth_of(y);
        for x in rx0..=rx1 {
            let px = *sprite.get_pixel(x, r);
            if !structure(px) {
                continue;
            }
            for z in (d - zd)..d {
                let c = if z == d - 1 {
                    px
                } else if z == d - zd {
                    grain.back(panel, x, y)
                } else if x == rx0 || x == rx1 {
                    grain.side(panel, z, y)
                } else {
                    px
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Top surface: the topmost row's cells behind the front
    // plane take the top region's un-projection, as in the
    // solid model.
    let ytop = fh - 1;
    let zd = depth_of(ytop);
    for x in fr.x0..=fr.x1 {
        for z in (d - zd)..(d - 1) {
            let sr = top_row_for_z(seg, d, z);
            let px = *sprite.get_pixel(x, sr);
            if structure(px) {
                m.set(x, ytop, z, Some(px));
            }
        }
    }
    // The open compartment: empty through the depth.
    if let Some(op) = &fit.opening {
        for y in op.rows.clone() {
            for x in op.cols.clone() {
                for z in 0..d {
                    m.set(x, y, z, None);
                }
            }
        }
    }
    m
}
