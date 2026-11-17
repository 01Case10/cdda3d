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
//!   complete drawing of the console, so the front face is the
//!   sprite's front region VERBATIM (the solid model's front
//!   rule). What the archetype adds over the solid model is the
//!   workstation's PROFILE and SIDE MATERIALS (round 2, Joey's
//!   correction — the first rework's sides read near-black
//!   top-to-bottom and its base stepped back at the rear):
//!   the rows split into three bands — BASE (the beige cabinet
//!   rows, fitted by their warm light edge color), DESK (up to
//!   the fitted desk height), TERMINAL (the console rise). All
//!   bands share one BACK plane; the desk and base fronts sit
//!   at the front plane, while the terminal's front face is
//!   RECESSED a few cells behind it (a real CRT workstation:
//!   desk ledge in front of the screen), its face still the
//!   sprite verbatim, just set back. Each band's SIDES are that
//!   band's own material — the modal color of the band region's
//!   edge columns: dark casing gray for the terminal, desk gray
//!   for the desk band, beige for the cabinet. The base's open
//!   compartment — the dark knee-hole the sprite draws in its
//!   bottom rows (opaque near-black recess rows over a
//!   transparent cutout) — is modeled OPEN through the depth;
//!   it opens at the front only in the sense that the side
//!   cabinets flanking it are full-depth, so from the side the
//!   base reads as one solid beige cabinet. FITTED: desk
//!   height, base band height, the three side materials, the
//!   opening's rows and columns (contiguous void runs from the
//!   bottom, strictly inside the frame's extent). ASSUMED: the
//!   terminal recess (4 cells — the mockup's ledge depth).
//!
//! Everything is deterministic: fixed iteration order, palettes
//! from the deterministic analyzer, grain fBm seeded from the
//! sprite's content hash.
//!
//! ## Stage G.4: the jail + city follow-on set
//!
//! The §12 sequence after the lab set: jail + city interiors.
//! Every candidate was audited against the sprite first
//! (pipeline §24/§27 — identify the real-world referent, check
//! the drawn view, check that the archetype's defining geometry
//! is MEASURABLE in the sprite). Two archetypes were built:
//!
//! * **LaundryAppliance** — f_washer / f_dryer (and
//!   f_autoclave_full, which resolves to f_washer's sprite): a
//!   front-loading laundry appliance, drawn as a clean front
//!   view (32x64 sheet, the machine in rows 27..64, segmentation
//!   confidence 0.94). A closed enamel cabinet: control strip
//!   across the top of the front, a door field, a horizontal
//!   seam, a lower body and a plinth. The construction is the
//!   ConsoleDesk discipline applied to a cabinet with no
//!   opening: front verbatim, back a plain panel in the primary
//!   material, flanks in PER-BAND, PER-SIDE materials (the
//!   modal color of each band's edge column on that side; on
//!   these sprites the bands largely share the cabinet's edge
//!   trim — a white highlight strip on the left, gray trim on
//!   the right — with the control strip and plinth differing).
//!   FITTED: the control band
//!   height, the seam rows and the plinth height — all three
//!   are runs of rows whose modal color differs from the front
//!   region's dominant (door/body) modal color, counted from
//!   the top, the middle and the bottom respectively; the depth
//!   (the pipeline's top-region rule, 11 cells); the five band
//!   side materials. ASSUMED: nothing structural — the cabinet
//!   is a closed box, so there is no recess to proportion; the
//!   interior fill is the front pixel verbatim (unseen), as in
//!   ConsoleDesk.
//! * **VendingMachine** — f_vending_c / f_vending_reinforced /
//!   f_vending_o (each with its own sprite; same cabinet layout)
//!   plus the `_off` variants, which resolve to those sprites.
//!   A glass-front vending machine: header panel, product
//!   display, and — the archetype's reason to exist — the
//!   DISPENSING BAY at the bottom of the front, drawn as a
//!   near-black slot (rows 55..56 of the 64-row sheet, an
//!   18-column void run) above a 7-row base. The bay is modeled
//!   as a genuine recess: open at the front to a fitted depth,
//!   with a back wall in the bay's own modal paint. The front
//!   is verbatim everywhere outside the bay. FITTED: the bay's
//!   rows and columns (the console opening's void-run detector,
//!   run over the silhouette's bottom half), the base height
//!   (the rows below the bay), the bay's back-wall color, the
//!   three bands' per-side materials (upper cabinet / bay row
//!   / base, from that side's edge column — the left edge is a
//!   white highlight strip, the right the dark edge trim of
//!   the green side panel the sprite draws in its front plane).
//!   ASSUMED: the cabinet depth (16
//!   cells = half the sprite width — the tall-cabinet precedent
//!   of f_fridge's 16; the scored segmentation's "top region"
//!   for these sprites is a shelf seam inside the display, so
//!   its depth reading is meaningless and the builder works
//!   from the silhouette rows instead of the front region) and
//!   the bay depth (8 cells = half the cabinet depth; a real
//!   dispensing bay is a shallow trough, not a through-hole).
//!
//! Audited and DROPPED (evidence in the G.4 report): f_toilet
//! (no segmentation — ragged silhouette; the drawing mixes a
//! plan-view bowl with a front-view cistern, so no single view
//! reading is faithful), f_shower (view ambiguous — a plan
//! tray with elevation fixtures; its segmentation is a spurious
//! 2-row "top" over a 57-row "front"), f_armchair (rotates
//! composite = the faceS front view, a whole armchair — but the
//! arm/seat junction, the archetype's defining geometry, has no
//! measurable signature: the structure extent widens smoothly
//! and arm and cushion share one modal fabric color, so arm
//! width / seat width / seat height would be assumed, not
//! fitted), f_dresser (rotates composite = faceS; the only
//! scored seam is the drawer seam at row 44, leaving the upper
//! 19 rows unreadable as either top surface or upper front),
//! f_filing_cabinet (clean front view but no segmentation —
//! the cabinet occupies half the tile width, under the
//! coverage floors), f_cupboard (multitile: the composite is
//! the `unconnected` section of a counter run, not a whole
//! object), f_entertainment_center (segments, but the upper
//! half is a plain panel that reads as neither a drawn top
//! surface nor an identifiable front component).

use crate::analyze::content_hash;
use crate::color::Oklab;
use crate::noise::Fbm;
use crate::props::{
    front_structure_extent, grain_amp, lab_of, modal_color, region_primaries, structure,
    top_row_for_z, PropModel, Segmentation, SHADOW_L,
};
use image::{Rgba, RgbaImage};
use std::ops::Range;

/// A parametric prop archetype (module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    ShelfUnit,
    ConsoleDesk,
    LaundryAppliance,
    VendingMachine,
    HollowCylinder,
    ClosedCylinder,
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
    // Stage G.4: laundry appliances (front-loading cabinet).
    ("f_washer", Archetype::LaundryAppliance),
    ("f_dryer", Archetype::LaundryAppliance),
    ("f_autoclave_full", Archetype::LaundryAppliance), // looks_like f_washer, no own sprite
    // Stage G.4: vending machines (glass front + dispensing bay).
    ("f_vending_c", Archetype::VendingMachine),
    ("f_vending_reinforced", Archetype::VendingMachine), // own sprite, same layout
    ("f_vending_o", Archetype::VendingMachine),          // own sprite (broken machine)
    ("f_vending_c_off", Archetype::VendingMachine),      // looks_like f_vending_c, no own sprite
    ("f_vending_reinforced_off", Archetype::VendingMachine), // looks_like f_vending_reinforced
    // Coverage fix round (2026-11-17): hollow cylinders — open
    // barrels and a trash can; the interior (dark, or burning
    // for the fd_fire variants) is drawn in the sprite's top
    // band and becomes a real cavity with a contents surface.
    ("f_30gal_firebarrel", Archetype::HollowCylinder),
    ("f_30gal_firebarrel_fd_fire", Archetype::HollowCylinder),
    ("f_55gal_firebarrel", Archetype::HollowCylinder),
    ("f_55gal_firebarrel_fd_fire", Archetype::HollowCylinder),
    ("f_trashcan", Archetype::HollowCylinder),
    // Closed cylinders — a lidded keg and the fermenting vats;
    // the sprite's top ellipse band becomes a flat lid disc.
    ("f_wood_keg", Archetype::ClosedCylinder),
    ("f_fvat_empty", Archetype::ClosedCylinder),
    ("f_fvat_full", Archetype::ClosedCylinder),
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
        Archetype::LaundryAppliance => build_laundry(sprite, seg),
        Archetype::VendingMachine => build_vending(sprite, seg),
        Archetype::HollowCylinder => build_hollow_cylinder(sprite, seg),
        Archetype::ClosedCylinder => build_closed_cylinder(sprite, seg),
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

/// The terminal front's recess behind the front plane, in
/// cells (module docs: assumed from the mockup's ledge depth).
pub const CONSOLE_RECESS: u32 = 4;

/// Fitted console parameters: the desk height (model rows below
/// the console rise), the base band's height, the terminal
/// recess, the three bands' side materials, and the base's
/// open compartment when the sprite draws one.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleFit {
    pub desk_h: u32,
    /// Model rows [0, base_h) are the cabinet base band; rows
    /// [base_h, desk_h) the desk band; [desk_h, fh) the
    /// terminal band.
    pub base_h: u32,
    pub recess: u32,
    /// Side (flank) material per band: the modal color of the
    /// band region's edge columns (the sprite's outermost
    /// structure pixels, both sides).
    pub term_side: Rgba<u8>,
    pub desk_side: Rgba<u8>,
    pub base_side: Rgba<u8>,
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
    let recess = CONSOLE_RECESS.min(fr.d.saturating_sub(1));
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
    // Bands and their side materials (module docs). A row's
    // edge samples are its outermost structure pixels, left and
    // right. The base band is fitted by color: the cabinet rows
    // are the warm, light (beige) rows at the bottom — the scan
    // starts at the first beige row (the dark plinth rows below
    // it belong to the base band) and stops at the first
    // non-beige row above the beige run.
    let row_edges = |y: u32| -> Option<(Rgba<u8>, Rgba<u8>)> {
        let r = fr.bot - y;
        let (mut first, mut last) = (None, None);
        for x in 0..w {
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                if first.is_none() {
                    first = Some(px);
                }
                last = Some(px);
            }
        }
        first.zip(last)
    };
    let beige = |px: Rgba<u8>| px[0] as i32 - px[2] as i32 >= 15 && lab_of(px).l >= 0.45;
    let (mut base_h, mut first_beige) = (0u32, None);
    for y in 0..desk_h {
        let is_beige = row_edges(y).map_or(false, |(a, b)| beige(a) && beige(b));
        if is_beige {
            if first_beige.is_none() {
                first_beige = Some(y);
            }
            base_h = y + 1;
        } else if first_beige.is_some() {
            break;
        }
    }
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let band_side = |rows: Range<u32>| -> Rgba<u8> {
        modal_color(
            rows.flat_map(|y| row_edges(y).into_iter().flat_map(|(a, b)| [a, b])),
        )
        .unwrap_or_else(|| rgba(panel))
    };
    let term_side = band_side(desk_h..fr.fh);
    let desk_side = band_side(base_h..desk_h);
    let base_side = match first_beige {
        Some(fb) => band_side(fb..base_h),
        None => band_side(0..base_h),
    };
    ConsoleFit {
        desk_h,
        base_h,
        recess,
        term_side,
        desk_side,
        base_side,
        opening,
    }
}

fn build_console(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = console_fit(sprite, seg);
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    // The profile (module docs): every band fills from the
    // common back plane (z = 0); desk and base fronts sit at
    // the front plane, the terminal's front face is recessed.
    let front_z = |y: u32| {
        if y >= fit.desk_h {
            d - 1 - fit.recess
        } else {
            d - 1
        }
    };
    let side_of = |y: u32| {
        if y >= fit.desk_h {
            fit.term_side
        } else if y >= fit.base_h {
            fit.desk_side
        } else {
            fit.base_side
        }
    };
    let mut m = PropModel::new([w, fh, d]);
    // Front face verbatim (at the band's front plane), back
    // plane the panel grain, flanks the band's own side
    // material, interior the front pixel (unseen).
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
        let fz = front_z(y);
        for x in rx0..=rx1 {
            let px = *sprite.get_pixel(x, r);
            if !structure(px) {
                continue;
            }
            for z in 0..=fz {
                let c = if z == fz {
                    px
                } else if z == 0 {
                    grain.back(panel, x, y)
                } else if x == rx0 || x == rx1 {
                    side_of(y)
                } else {
                    px
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Top surface: the topmost (terminal) row's cells behind
    // its recessed front face take the top region's
    // un-projection over the terminal block's own depth, as in
    // the solid model.
    let ytop = fh - 1;
    let fz = front_z(ytop);
    let td = d - fit.recess;
    for x in fr.x0..=fr.x1 {
        for z in 0..fz {
            let sr = top_row_for_z(seg, td, z);
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

// ---------------------------------------------------------------------
// Shared G.4 fitting helpers
// ---------------------------------------------------------------------

/// The modal STRUCTURE color of one sprite row (None when the
/// row carries no structure pixel).
fn row_modal(sprite: &RgbaImage, r: u32) -> Option<Rgba<u8>> {
    modal_color((0..sprite.width()).filter_map(|x| {
        let px = *sprite.get_pixel(x, r);
        if structure(px) {
            Some(px)
        } else {
            None
        }
    }))
}

/// A band's side materials: the modal color of the band rows'
/// edge samples, taken PER SIDE — each row's outermost
/// structure pixel on that side. Per side, not pooled (the
/// ConsoleDesk rule pools): the vending machine's two edges
/// genuinely differ (a white highlight strip on the left, the
/// green side panel on the right), and pooling hands both
/// flanks the more constant edge's color. Falls back to the
/// panel color when a band has no edge samples on a side.
fn band_edge_modals(
    sprite: &RgbaImage,
    rows: Range<u32>,
    bot: u32,
    fallback: Rgba<u8>,
) -> (Rgba<u8>, Rgba<u8>) {
    let (mut left, mut right) = (Vec::new(), Vec::new());
    for y in rows {
        let r = bot - y;
        let (mut first, mut last) = (None, None);
        for x in 0..sprite.width() {
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                if first.is_none() {
                    first = Some(px);
                }
                last = Some(px);
            }
        }
        if let Some(px) = first {
            left.push(px);
        }
        if let Some(px) = last {
            right.push(px);
        }
    }
    (
        modal_color(left.into_iter()).unwrap_or(fallback),
        modal_color(right.into_iter()).unwrap_or(fallback),
    )
}

/// The structure x-extent of one sprite row (None when empty).
fn row_extent(sprite: &RgbaImage, r: u32) -> Option<(u32, u32)> {
    let (mut x0, mut x1) = (sprite.width(), 0u32);
    let mut any = false;
    for x in 0..sprite.width() {
        if structure(*sprite.get_pixel(x, r)) {
            x0 = x0.min(x);
            x1 = x1.max(x);
            any = true;
        }
    }
    if any {
        Some((x0, x1))
    } else {
        None
    }
}

/// Fill one model row as a closed cabinet row (the G.4 shared
/// construction): front plane verbatim, back plane the panel
/// grain, flank cells (the row extent's edge columns, interior
/// depth) in that side's material (`sides` = (left, right)),
/// interior the front pixel verbatim. Rows whose sprite row
/// has no structure stay empty.
fn fill_cabinet_row(
    m: &mut PropModel,
    sprite: &RgbaImage,
    grain: &Grain,
    panel: Oklab,
    y: u32,
    r: u32,
    d: u32,
    sides: (Rgba<u8>, Rgba<u8>),
) {
    let Some((rx0, rx1)) = row_extent(sprite, r) else {
        return;
    };
    for x in rx0..=rx1 {
        let px = *sprite.get_pixel(x, r);
        if !structure(px) {
            continue;
        }
        for z in 0..d {
            let c = if z == d - 1 {
                px
            } else if z == 0 {
                grain.back(panel, x, y)
            } else if x == rx0 {
                sides.0
            } else if x == rx1 {
                sides.1
            } else {
                px
            };
            m.set(x, y, z, Some(c));
        }
    }
}

// ---------------------------------------------------------------------
// LaundryAppliance (Stage G.4)
// ---------------------------------------------------------------------

/// Fitted laundry-appliance parameters (module docs, "Stage
/// G.4"): the band boundaries of the front — control strip,
/// door field, seam, lower body, plinth — fitted as runs of
/// rows whose modal color differs from the front's dominant
/// modal color, plus each band's side material and the model
/// depth (the pipeline's top-region rule).
#[derive(Debug, Clone, PartialEq)]
pub struct LaundryFit {
    /// Model rows [fh - control_h, fh): the control strip.
    pub control_h: u32,
    /// Model rows [0, plinth_h): the plinth.
    pub plinth_h: u32,
    /// The seam's model rows (inclusive): the joint between
    /// the door field and the lower body.
    pub seam_rows: (u32, u32),
    /// Model depth (sub-cells).
    pub depth: u32,
    /// Side (flank) materials per band, (left, right): the
    /// modal color of the band rows' edge columns on that side.
    pub control_side: (Rgba<u8>, Rgba<u8>),
    pub door_side: (Rgba<u8>, Rgba<u8>),
    pub seam_side: (Rgba<u8>, Rgba<u8>),
    pub body_side: (Rgba<u8>, Rgba<u8>),
    pub plinth_side: (Rgba<u8>, Rgba<u8>),
}

impl LaundryFit {
    /// The side materials of the band containing model row `y`
    /// (fh = the model's front height), as (left, right).
    pub fn side_of(&self, y: u32, fh: u32) -> (Rgba<u8>, Rgba<u8>) {
        if y >= fh - self.control_h {
            self.control_side
        } else if y > self.seam_rows.1 {
            self.door_side
        } else if y >= self.seam_rows.0 {
            self.seam_side
        } else if y >= self.plinth_h {
            self.body_side
        } else {
            self.plinth_side
        }
    }
}

pub fn laundry_fit(sprite: &RgbaImage, seg: &Segmentation) -> LaundryFit {
    let fr = frame(sprite, seg);
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let fallback = rgba(panel);
    // Dominant modal color of the whole front region (the
    // door/body enamel): every band boundary is a run of rows
    // whose own modal differs from it.
    let dominant = modal_color(
        seg.front_rows
            .clone()
            .flat_map(|r| (0..sprite.width()).map(move |x| (x, r)))
            .filter_map(|(x, r)| {
                let px = *sprite.get_pixel(x, r);
                if structure(px) {
                    Some(px)
                } else {
                    None
                }
            }),
    )
    .unwrap_or(fallback);
    let rows = seg.front_rows.clone();
    let modal_at = |r: u32| row_modal(sprite, r);
    // Control band: rows from the front's top whose modal is
    // not the dominant color, up to the first dominant row.
    let mut control_h = 0u32;
    for r in rows.clone() {
        if modal_at(r) != Some(dominant) {
            control_h += 1;
        } else {
            break;
        }
    }
    // Plinth: the same run counted from the front's bottom.
    let mut plinth_h = 0u32;
    for r in rows.clone().rev() {
        if modal_at(r) != Some(dominant) {
            plinth_h += 1;
        } else {
            break;
        }
    }
    // Seam: the non-dominant run strictly between the control
    // band and the plinth, nearest the remaining band's center.
    // (Fallback, when no run reads: the single middle row —
    // assumed, like ShelfUnit's evenly-spaced fallback.)
    let mid_lo = rows.start + control_h;
    let mid_hi = rows.end - plinth_h; // exclusive
    let mut runs: Vec<(u32, u32)> = Vec::new();
    let mut r = mid_lo;
    while r < mid_hi {
        if modal_at(r) != Some(dominant) {
            let s = r;
            while r < mid_hi && modal_at(r) != Some(dominant) {
                r += 1;
            }
            runs.push((s, r - 1));
        } else {
            r += 1;
        }
    }
    let center = (mid_lo + mid_hi) as f32 / 2.0;
    let seam_sprite = runs
        .into_iter()
        .min_by(|a, b| {
            let ca = ((a.0 + a.1) as f32 / 2.0 - center).abs();
            let cb = ((b.0 + b.1) as f32 / 2.0 - center).abs();
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| {
            let m = (mid_lo + mid_hi - 1) / 2;
            (m, m)
        });
    // Sprite rows -> model rows (y = bot - row), ascending.
    let seam_rows = (fr.bot - seam_sprite.1, fr.bot - seam_sprite.0);
    let fh = fr.fh;
    let side = |rows_y: Range<u32>| band_edge_modals(sprite, rows_y, fr.bot, fallback);
    LaundryFit {
        control_h,
        plinth_h,
        seam_rows,
        depth: fr.d,
        control_side: side(fh - control_h..fh),
        door_side: side(seam_rows.1 + 1..fh - control_h),
        seam_side: side(seam_rows.0..seam_rows.1 + 1),
        body_side: side(plinth_h..seam_rows.0),
        plinth_side: side(0..plinth_h),
    }
}

fn build_laundry(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fr = frame(sprite, seg);
    let fit = laundry_fit(sprite, seg);
    let (_, panel) = region_primaries(sprite, &seg.front_rows);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fr.fh, fr.d);
    let mut m = PropModel::new([w, fh, d]);
    for y in 0..fh {
        let r = fr.bot - y;
        fill_cabinet_row(&mut m, sprite, &grain, panel, y, r, d, fit.side_of(y, fh));
    }
    // Top surface: the top region un-projected over the top
    // row's depth behind the front plane, as in ShelfUnit /
    // ConsoleDesk.
    let ytop = fh - 1;
    for x in fr.x0..=fr.x1 {
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            let px = *sprite.get_pixel(x, sr);
            if structure(px) {
                m.set(x, ytop, z, Some(px));
            }
        }
    }
    m
}

// ---------------------------------------------------------------------
// VendingMachine (Stage G.4)
// ---------------------------------------------------------------------

/// The vending cabinet's depth in sub-cells (module docs:
/// ASSUMED — half the sprite width, the tall-cabinet
/// precedent; the scored segmentation's top region is a shelf
/// seam inside the display, so the builder works from the
/// silhouette rows and fits its own depth proportion).
pub const VENDING_DEPTH: u32 = 16;
/// The dispensing bay's depth in sub-cells (module docs:
/// ASSUMED — half the cabinet depth; a shallow trough with a
/// back wall, not a through-hole).
pub const VENDING_BAY_DEPTH: u32 = 8;

/// A dispensing bay modeled OPEN (module docs): the column
/// range (model x) and row range (model y) that are empty at
/// the front, to [`VENDING_BAY_DEPTH`] cells of depth.
#[derive(Debug, Clone, PartialEq)]
pub struct VendingBay {
    pub cols: Range<u32>,
    pub rows: Range<u32>,
}

/// Fitted vending-machine parameters (module docs, "Stage
/// G.4"). The frame is the sprite's SILHOUETTE (the scored
/// front region is a shelf seam inside the display for these
/// sprites): height = silhouette height, front rows = the
/// silhouette rows.
#[derive(Debug, Clone, PartialEq)]
pub struct VendingFit {
    /// Model height (silhouette height in rows).
    pub height: u32,
    /// Model depth (sub-cells) — [`VENDING_DEPTH`], assumed.
    pub depth: u32,
    /// Model rows [0, base_h): the base below the bay.
    pub base_h: u32,
    /// The dispensing bay (fitted void run, module docs).
    pub bay: VendingBay,
    /// The bay's back-wall color: the modal paint of the bay
    /// region's dark pixels (the slot's own paint).
    pub bay_wall: Rgba<u8>,
    /// Side (flank) materials per band, (left, right): the
    /// modal color of the band rows' edge columns on that
    /// side. Bands: upper cabinet (above the bay), the bay
    /// rows themselves, the base.
    pub upper_side: (Rgba<u8>, Rgba<u8>),
    pub bay_side: (Rgba<u8>, Rgba<u8>),
    pub base_side: (Rgba<u8>, Rgba<u8>),
}

impl VendingFit {
    /// The side materials of the band containing model row
    /// `y`, as (left, right).
    pub fn side_of(&self, y: u32) -> (Rgba<u8>, Rgba<u8>) {
        if y < self.base_h {
            self.base_side
        } else if self.bay.rows.contains(&y) {
            self.bay_side
        } else {
            self.upper_side
        }
    }
}

/// A front pixel that reads as VOID for the bay detector (the
/// console opening's rule): not structure, or opaque near-dark.
fn vending_void(px: Rgba<u8>) -> bool {
    !structure(px) || lab_of(px).l < 0.20
}

pub fn vending_fit(sprite: &RgbaImage, seg: &Segmentation) -> VendingFit {
    let sil = seg.silhouette_rows.clone();
    let fh = sil.end - sil.start;
    let bot = sil.end - 1;
    let (_, panel) = region_primaries(sprite, &sil);
    let fallback = rgba(panel);
    // Global structure extent of the silhouette.
    let (mut gx0, mut gx1) = (sprite.width() - 1, 0u32);
    for r in sil.clone() {
        if let Some((a, b)) = row_extent(sprite, r) {
            gx0 = gx0.min(a);
            gx1 = gx1.max(b);
        }
    }
    let ext_w = gx1 - gx0 + 1;
    // Bay rows: scan model rows from the bottom; a row
    // qualifies when its longest interior VOID run is at least
    // half the extent width. The bay is the first contiguous
    // qualifying run in the bottom half (the base rows below
    // it do not qualify; the shelf-shadow row above the bay's
    // frame, void run 14 of 32 on f_vending_c, does not clear
    // the half-width floor either).
    let qualifies = |y: u32| -> bool {
        let r = bot - y;
        let (mut run, mut best) = (0u32, 0u32);
        for x in gx0..=gx1 {
            if vending_void(*sprite.get_pixel(x, r)) {
                run += 1;
                best = best.max(run);
            } else {
                run = 0;
            }
        }
        best * 2 >= ext_w
    };
    let mut bay_rows: Option<Range<u32>> = None;
    for y in 0..fh / 2 {
        if qualifies(y) {
            let start = y;
            let mut end = y + 1;
            while end < fh && qualifies(end) {
                end += 1;
            }
            bay_rows = Some(start..end);
            break;
        }
    }
    let bay_rows = bay_rows.unwrap_or(fh / 4..fh / 4 + 2); // assumed fallback (module docs)
    let base_h = bay_rows.start;
    // Bay columns: the console opening's column vote over the
    // bay rows — columns void in at least half the bay rows;
    // of the contiguous runs at least 6 wide, the one nearest
    // the extent's center.
    let w = sprite.width();
    let n_rows = bay_rows.end - bay_rows.start;
    let mut col_void = vec![0u32; w as usize];
    for y in bay_rows.clone() {
        let r = bot - y;
        for x in 0..w {
            if vending_void(*sprite.get_pixel(x, r)) {
                col_void[x as usize] += 1;
            }
        }
    }
    let need = n_rows.div_ceil(2);
    let center = (gx0 + gx1) as f32 / 2.0;
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
    let bay_cols = runs
        .into_iter()
        .filter(|r| r.end - r.start >= 6)
        .min_by(|a, b| {
            let ca = ((a.start + a.end - 1) as f32 / 2.0 - center).abs();
            let cb = ((b.start + b.end - 1) as f32 / 2.0 - center).abs();
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        })
        // Assumed fallback (module docs).
        .unwrap_or(gx0 + 4..gx1 - 3);
    // Bay back wall: the modal dark paint inside the bay.
    let bay_wall = modal_color(bay_rows.clone().flat_map(|y| {
        let r = bot - y;
        bay_cols.clone().filter_map(move |x| {
            let px = *sprite.get_pixel(x, r);
            if px[3] >= 128 && lab_of(px).l < 0.20 {
                Some(px)
            } else {
                None
            }
        })
    }))
    .unwrap_or(fallback);
    let side = |rows_y: Range<u32>| band_edge_modals(sprite, rows_y, bot, fallback);
    VendingFit {
        height: fh,
        depth: VENDING_DEPTH.min(sprite.width()),
        base_h,
        bay: VendingBay {
            cols: bay_cols,
            rows: bay_rows.clone(),
        },
        bay_wall,
        upper_side: side(bay_rows.end..fh),
        bay_side: side(bay_rows),
        base_side: side(0..base_h),
    }
}

fn build_vending(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fit = vending_fit(sprite, seg);
    let sil = seg.silhouette_rows.clone();
    let bot = sil.end - 1;
    let (_, panel) = region_primaries(sprite, &sil);
    let grain = Grain::new(sprite);
    let (w, fh, d) = (sprite.width(), fit.height, fit.depth);
    let mut m = PropModel::new([w, fh, d]);
    for y in 0..fh {
        let r = bot - y;
        fill_cabinet_row(&mut m, sprite, &grain, panel, y, r, d, fit.side_of(y));
    }
    // The dispensing bay: empty from the front plane back
    // VENDING_BAY_DEPTH cells, closed by a back wall in the
    // bay's own modal paint. The frame posts flanking the bay
    // and the rows above/below it keep their cabinet fill.
    let wall_z = d - 1 - VENDING_BAY_DEPTH;
    for y in fit.bay.rows.clone() {
        for x in fit.bay.cols.clone() {
            for z in wall_z + 1..d {
                m.set(x, y, z, None);
            }
            m.set(x, y, wall_z, Some(fit.bay_wall));
        }
    }
    m
}

// ---------------------------------------------------------------------
// HollowCylinder / ClosedCylinder (coverage fix round, 2026-11-17;
// pipeline §32)
//
// Joey's review of the coverage drive's cylinder treatment: the
// per-column tops followed the sprite's drawn rim ellipse, so
// open barrels carried a jagged solid cap of rim/interior paint
// where the real object has an opening. These two archetypes
// build the actual container:
//
// * HollowCylinder — a wall shell (thickness WALL_T) around a
//   genuine cavity. The cavity's floor is the CONTENTS surface:
//   what the sprite draws inside its top ellipse (dark interior
//   for an empty barrel, burning fuel for the fd_fire variants),
//   sampled as the modal paint of the top band's inner columns.
//   The surface sits INTERIOR_RATIO of the diameter below the
//   rim — the drawn ellipse's vertical diameter measured on the
//   barrel family (9 rows over a 16-wide barrel = 0.56).
// * ClosedCylinder — the same shell and disc, filled to the
//   top, with the interior disc's top face set to the LID color
//   (modal paint of the top ellipse band): a keg's lid, a vat's
//   closed top. Column paint is clamped at each column's own
//   top, so the perspective ellipse flattens into a level lid
//   instead of a stepped dome.
//
// Both: every shell voxel carries its sprite column's paint (the
// front is verbatim; the sides show the sprite's edge columns,
// the cylinder convention), the footprint is the disc inscribed
// in the front region's structure extent, depth = extent width
// capped at 32. The segmentation these ids arrive with is the
// coverage path's synthesized seamless one (front = the whole
// silhouette band) — the archetype routing in
// `props::model_for` falls back to it for archetype ids that do
// not segment.
// ---------------------------------------------------------------------

/// Wall thickness of the cylinder shell, in cells (the barrel
/// family's drawn rim measures ~2 cells at these diameters).
const CYLINDER_WALL_T: u32 = 2;
/// The drawn top ellipse's vertical diameter as a fraction of
/// the object's width (fitted on the barrel family: 9/16).
const CYLINDER_INTERIOR_RATIO: f32 = 0.55;

/// Fitted parameters of a cylinder archetype (also the
/// showcase's report line).
pub struct CylinderFit {
    /// Model depth = disc diameter (extent width, capped 32).
    pub d: u32,
    /// Model height = the WALL height: the sprite's band minus
    /// its top ellipse band. The drawn ellipse band is a
    /// HORIZONTAL surface in perspective (rim + interior, or a
    /// lid) — extruding it as vertical wall was the artifact of
    /// Joey's 2026-11-17 second cylinder review (the fire
    /// barrel's fire plug stood proud of the rim; compare the
    /// sprite, where the fire is a flat disc inside the rim).
    pub fh: u32,
    /// Sprite rows of the top ellipse band (opening or lid),
    /// measured by [`measure_opening_rows`].
    pub opening_rows: u32,
    /// How far below the rim the contents surface sits
    /// (HollowCylinder): a quarter of the opening band, at
    /// least 2 — the drawn surface sits essentially at the rim
    /// (the fire ellipse IS the barrel's top face in the
    /// sprite); a deeper drop hid it to a slit in review.
    pub drop: u32,
    /// Dominant paint family of the opening band's inner
    /// columns: the lid color (ClosedCylinder).
    pub surface_color: Rgba<u8>,
    /// Dominant family among the band's INTERIOR-classified
    /// inner pixels (dark-cluster or fire-hued, the scan's
    /// classification): the contents surface color
    /// (HollowCylinder). Falls back to `surface_color` when the
    /// band has no interior-classified pixels.
    pub interior_color: Rgba<u8>,
}

/// Measure the sprite's top ellipse band: the maximal prefix of
/// band rows that are drawn as the top surface rather than the
/// wall. A row belongs to the band when its structure extent is
/// still widening toward the full width (the ellipse curve), or
/// when the majority of its inner pixels are interior paint —
/// dark-cluster (the sprite's own two-means L split over all
/// inner pixels) or fire-hued. Measured values on the served
/// sprites: barrels 10 of 30 rows, keg 8 of 24, vats 10 of 44,
/// trash can 5 of 18. If the scan finds less than 2 rows it
/// falls back to the drawn-ellipse ratio
/// ([`CYLINDER_INTERIOR_RATIO`] of the diameter).
/// The result of the opening-band scan: the band's row count
/// and the lightness split the scan classified interiors with
/// (reused for the contents color).
struct OpeningScan {
    rows: u32,
    split: f32,
}

fn measure_opening_rows(sprite: &RgbaImage, seg: &Segmentation, width: u32) -> OpeningScan {
    let (w, _) = (sprite.width(), sprite.height());
    let band = seg.front_rows.clone();
    // Per-row stats over DRAWN pixels (fully opaque, non-black):
    // the ellipse is an opaque-paint phenomenon — the project's
    // structure() also admits semi-transparent antialiasing
    // fringe (alpha >= 128), which widens the top rows' extents
    // to full width and defeats the widening test.
    let drawn = |x: u32, r: u32| {
        let p = *sprite.get_pixel(x, r);
        p[3] >= 254 && !(p[0] == 0 && p[1] == 0 && p[2] == 0)
    };
    let mut rows: Vec<(u32, Vec<Rgba<u8>>)> = Vec::new();
    let mut all_l: Vec<f32> = Vec::new();
    for r in band.clone() {
        let xs: Vec<u32> = (0..w).filter(|&x| drawn(x, r)).collect();
        if xs.is_empty() {
            rows.push((0, Vec::new()));
            continue;
        }
        let (a, b) = (xs[0], xs[xs.len() - 1]);
        let inner: Vec<Rgba<u8>> = (a + 2..b.saturating_sub(1))
            .filter(|&x| drawn(x, r))
            .map(|x| *sprite.get_pixel(x, r))
            .collect();
        all_l.extend(inner.iter().map(|p| lab_of(*p).l));
        rows.push((b - a + 1, inner));
    }
    // Two-means split of the inner pixels' lightness.
    let split = if all_l.is_empty() {
        1.0
    } else {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for l in &all_l {
            lo = lo.min(*l);
            hi = hi.max(*l);
        }
        for _ in 0..16 {
            let (mut sl, mut sh, mut nl, mut nh) = (0f32, 0f32, 0u32, 0u32);
            for l in &all_l {
                if (l - lo).abs() <= (l - hi).abs() {
                    sl += l;
                    nl += 1;
                } else {
                    sh += l;
                    nh += 1;
                }
            }
            if nl > 0 {
                lo = sl / nl as f32;
            }
            if nh > 0 {
                hi = sh / nh as f32;
            }
        }
        (lo + hi) / 2.0
    };
    let fire = |p: Rgba<u8>| (p[0] as u32) > 100 && (p[0] as u32) > 2 * (p[1] as u32);
    let mut opening = 0u32;
    for (ext_w, inner) in &rows {
        let interior_majority = !inner.is_empty()
            && inner
                .iter()
                .filter(|p| lab_of(**p).l < split || fire(**p))
                .count()
                * 2
                > inner.len();
        if *ext_w < width || interior_majority {
            opening += 1;
        } else {
            break;
        }
    }
    let full = band.end - band.start;
    if opening < 2 {
        opening = ((width as f32 * CYLINDER_INTERIOR_RATIO).round() as u32).max(2);
    }
    OpeningScan {
        rows: opening.min(full / 2),
        split,
    }
}

/// Fit a cylinder archetype to a sprite + (seamless)
/// segmentation: footprint from the front region's structure
/// extent, opening band from [`measure_opening_rows`], surface
/// color from the band's inner columns.
pub fn cylinder_fit(sprite: &RgbaImage, seg: &Segmentation) -> Option<CylinderFit> {
    let (w, _) = (sprite.width(), sprite.height());
    let full = seg.front_rows.end - seg.front_rows.start;
    let (x0, x1) = front_structure_extent(sprite, seg);
    let width = x1 - x0 + 1;
    if width < 6 || full < 8 {
        return None;
    }
    let scan = measure_opening_rows(sprite, seg, width);
    let opening_rows = scan.rows;
    let cx = (x0 + x1) as f32 / 2.0;
    let half_inner = width as f32 / 4.0;
    // Paint families of the band's inner columns, as the mean
    // of the largest 16-step bucket. An exact modal fails here:
    // the fire interior is a gradient of distinct oranges, each
    // rarer than the repeating wall greys, so the modal picked
    // a wall grey for the burning barrel's surface.
    let family_mean = |pxs: &[Rgba<u8>]| -> Option<Rgba<u8>> {
        let mut buckets: Vec<((u8, u8, u8), Vec<Rgba<u8>>)> = Vec::new();
        for &p in pxs {
            let key = (p[0] >> 4, p[1] >> 4, p[2] >> 4);
            match buckets.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => v.push(p),
                None => buckets.push((key, vec![p])),
            }
        }
        let (_, dominant) = buckets.into_iter().max_by_key(|(_, v)| v.len())?;
        let n = dominant.len() as u32;
        let mean = |i: usize| (dominant.iter().map(|p| p[i] as u32).sum::<u32>() / n) as u8;
        Some(Rgba([mean(0), mean(1), mean(2), 255]))
    };
    let band_px: Vec<Rgba<u8>> = (seg.front_rows.start..seg.front_rows.start + opening_rows)
        .flat_map(|r| (0..w).map(move |x| (x, r)))
        .filter(|&(x, r)| {
            structure(*sprite.get_pixel(x, r)) && (x as f32 - cx).abs() <= half_inner
        })
        .map(|(x, r)| *sprite.get_pixel(x, r))
        .collect();
    let surface_color = family_mean(&band_px)?;
    let fire = |p: Rgba<u8>| (p[0] as u32) > 100 && (p[0] as u32) > 2 * (p[1] as u32);
    // Contents color: if the band contains fire paint, the
    // contents ARE the fire (the fd_fire variants) — the fire
    // family wins over the dark rim/wall pixels that also fall
    // under the lightness split. Otherwise the contents are
    // the dark interior family.
    let fire_px: Vec<Rgba<u8>> = band_px.iter().copied().filter(|p| fire(*p)).collect();
    let interior_color = if !fire_px.is_empty() {
        family_mean(&fire_px).unwrap_or(surface_color)
    } else {
        let dark_px: Vec<Rgba<u8>> = band_px
            .iter()
            .copied()
            .filter(|p| lab_of(*p).l < scan.split)
            .collect();
        family_mean(&dark_px).unwrap_or(surface_color)
    };
    Some(CylinderFit {
        d: width.min(32),
        fh: full - opening_rows,
        opening_rows,
        drop: (opening_rows / 4).max(2),
        surface_color,
        interior_color,
    })
}

/// The wall-only view of a (seamless) segmentation: the front
/// region's bottom `wall_h` rows — the rows below the opening
/// band, i.e. the actual elevation of the container's wall.
fn wall_seg(seg: &Segmentation, wall_h: u32) -> Segmentation {
    Segmentation {
        silhouette_rows: seg.silhouette_rows.clone(),
        top_rows: seg.top_rows.clone(),
        front_rows: (seg.front_rows.end - wall_h)..seg.front_rows.end,
        side: seg.side.as_ref().map(|s| crate::props::SideSliver {
            cols: s.cols.clone(),
            on_right: s.on_right,
        }),
        confidence: seg.confidence,
    }
}

/// Per-column sprite paint with vertical clamping: at model
/// heights where the column has no structure pixel (the drawn
/// ellipse curves away), the column's nearest structure pixel
/// answers — the un-projection that flattens the drawn ellipse
/// into the model's level top.
fn column_paint(sprite: &RgbaImage, seg: &Segmentation, x: u32) -> Vec<Option<Rgba<u8>>> {
    let fh = seg.front_rows.end - seg.front_rows.start;
    let bot = seg.front_rows.end - 1;
    let raw: Vec<Option<Rgba<u8>>> = (0..fh)
        .map(|y| {
            let px = *sprite.get_pixel(x, bot - y);
            structure(px).then_some(px)
        })
        .collect();
    (0..fh)
        .map(|y| {
            raw[y as usize].or_else(|| {
                (1..fh)
                    .filter_map(|dist| {
                        [y.checked_add(dist), y.checked_sub(dist)]
                            .into_iter()
                            .flatten()
                            .filter(|&yy| yy < fh)
                            .find_map(|yy| raw[yy as usize])
                    })
                    .next()
            })
        })
        .collect()
}

/// The disc geometry shared by both cylinder archetypes:
/// (x0, cx, cz, rx, rz) over the model's z range.
struct Disc {
    x0: u32,
    x1: u32,
    cx: f32,
    cz: f32,
    rx: f32,
    rz: f32,
}

impl Disc {
    fn new(sprite: &RgbaImage, seg: &Segmentation, d: u32) -> Disc {
        let (x0, x1) = front_structure_extent(sprite, seg);
        Disc {
            x0,
            x1,
            cx: (x0 + x1) as f32 / 2.0,
            cz: (d - 1) as f32 / 2.0,
            rx: (x1 - x0 + 1) as f32 / 2.0,
            rz: d as f32 / 2.0,
        }
    }
    /// Normalized radius of a cell (0 at the axis, 1 at the rim).
    fn rn(&self, x: u32, z: u32) -> f32 {
        let dx = (x as f32 - self.cx) / self.rx;
        let dz = (z as f32 - self.cz) / self.rz;
        (dx * dx + dz * dz).sqrt()
    }
    /// The shell band's inner normalized radius.
    fn inner_rn(&self) -> f32 {
        1.0 - CYLINDER_WALL_T as f32 / self.rx
    }
}

fn build_hollow_cylinder(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fit = cylinder_fit(sprite, seg).expect("cylinder fit");
    let disc = Disc::new(sprite, seg, fit.d);
    let wseg = wall_seg(seg, fit.fh);
    let surface_y = fit.fh.saturating_sub(fit.drop).max(fit.fh / 2);
    let mut m = PropModel::new([sprite.width(), fit.fh, fit.d]);
    for x in disc.x0..=disc.x1 {
        let paint = column_paint(sprite, &wseg, x);
        for z in 0..fit.d {
            let rn = disc.rn(x, z);
            if rn > 1.0 {
                continue;
            }
            if rn > disc.inner_rn() {
                // Shell: full height, the column's own paint.
                for y in 0..fit.fh {
                    if let Some(c) = paint[y as usize] {
                        m.set(x, y, z, Some(c));
                    }
                }
            } else {
                // Contents: solid up to the surface, cavity above.
                for y in 0..=surface_y {
                    m.set(x, y, z, Some(fit.interior_color));
                }
            }
        }
    }
    m
}

fn build_closed_cylinder(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let fit = cylinder_fit(sprite, seg).expect("cylinder fit");
    let disc = Disc::new(sprite, seg, fit.d);
    let wseg = wall_seg(seg, fit.fh);
    let mut m = PropModel::new([sprite.width(), fit.fh, fit.d]);
    for x in disc.x0..=disc.x1 {
        let paint = column_paint(sprite, &wseg, x);
        for z in 0..fit.d {
            let rn = disc.rn(x, z);
            if rn > 1.0 {
                continue;
            }
            for y in 0..fit.fh {
                if let Some(c) = paint[y as usize] {
                    m.set(x, y, z, Some(c));
                }
            }
            // The lid: the interior disc's top face is the one
            // fitted lid material; the rim ring keeps its paint.
            if rn <= disc.inner_rn() {
                m.set(x, fit.fh - 1, z, Some(fit.surface_color));
            }
        }
    }
    m
}
