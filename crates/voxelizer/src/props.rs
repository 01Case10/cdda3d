//! Stage G (generation side): inferred 3D prop models from furniture
//! sprites.
//!
//! Today furniture renders as a flat extruded silhouette
//! (`material::extrusion_columns`). UltiCa furniture sprites are
//! drawn in 3/4 view — top rows are the object's top surface, the
//! band below is its front face, sometimes with a side sliver — so a
//! sprite carries more shape than a silhouette. This module recovers
//! that shape in one canonical form, plus the two earlier forms kept
//! for reference:
//!
//! * [`solid_model`] (THE model): the front silhouette extruded
//!   through the full model depth and filled at every depth cell —
//!   a real solid whose cross-section is the silhouette itself, so
//!   no viewing angle can see into a hollow interior or a gap.
//!   Fronts whose base is open (legs + painted shadow, e.g. tables)
//!   keep only their grounded columns under the slab — see "Base
//!   structure".
//! * [`box_model`] (G.1, deprecated): one cuboid whose depth comes
//!   from the top region's height and whose height comes from the
//!   front region's height, each face textured by un-projecting its
//!   sprite region.
//! * [`column_model`] (G.2, deprecated): a closed shell — a front
//!   wall of per-pixel columns whose front projection reproduces
//!   the sprite's front region exactly, a top cap extending
//!   backward, a mirrored back panel, and side/bottom closure
//!   strips.
//!
//! All consume a [`Segmentation`] produced by [`segment`]. The
//! engine-side stamping of these models is a later round; everything
//! here is voxelizer-side and deterministic (fixed iteration order,
//! no RNG, no wall clock).
//!
//! ## Segmentation method
//!
//! 1. **Silhouette.** Rows with opaque coverage >= 0.30 form the
//!    silhouette band (first to last such row). Sprites whose overall
//!    opaque fraction < 0.35, whose mean in-band coverage < 0.60, or
//!    whose band is ragged (>10% of in-band rows below 0.30 coverage)
//!    are rejected: spindly cutouts (chairs, creatures) and partial
//!    illustrations have no box-like reading.
//! 2. **Boundary candidates.** A boundary row `b` splits the band
//!    into top region `[top, b)` and front region `[b, bot]`. It must
//!    leave >= 2 top rows and a front of >= max(4 rows, 15% of the
//!    band). A candidate is *valid* when two signals agree:
//!    - the **horizontal edge** at `b`: mean OKLab distance between
//!      vertically adjacent pixels across the boundary, >= 0.12. The
//!      top/front seam is the object's front-top edge, drawn as a
//!      near-full-width color break;
//!    - the **local L step**: |mean L of the 4 rows below `b` minus
//!      mean L of the 4 rows above|, >= 0.03. The top surface and the
//!      front face are lit differently, so mean lightness steps at
//!      the seam even where the edge is soft.
//! 3. **Choice.** Among valid candidates, take the *first* (topmost)
//!    whose edge is >= 60% of the strongest valid edge. Furniture
//!    fronts are full of repeated internal seams (bookcase shelves,
//!    locker vents) that rival the true seam's strength, but they
//!    all lie *below* it; the top surface is the first strong seam
//!    under the sprite's top.
//! 4. **Orientation corroboration.** The structure-tensor
//!    orientation of the top band vs the front band (via
//!    `analyze::analyze_image` on the two crops) adjusts confidence:
//!    a >= 20-degree feature-angle difference, or exactly one band
//!    being coherent, corroborates; both bands coherent and near-
//!    parallel argues against. This is a *bonus/penalty only*: on
//!    the cleanest real sprites (f_counter, f_table) the top surface
//!    is a flat uniform field — isotropic, no orientation to flip —
//!    while the edge + L step are present in every clean case we
//!    measured, so orientation cannot be the primary signal.
//! 5. **Confidence.** A fixed weighted sum of edge strength, local
//!    L step, seam *support* (the fraction of seam columns whose own
//!    OKLab break clears a per-column floor — a true top/front edge
//!    is a near-full-width break), silhouette coverage, a small
//!    region-L term (top region mean L vs the front's first rows),
//!    and the orientation term. Below 0.45 the sprite is reported
//!    unsegmentable (`None`). An earlier formula also rewarded
//!    *margin over the runner-up edge*; that term was removed
//!    because it contradicts the first-strong-edge premise — tall
//!    furniture (fridge, bookcase, locker) legitimately has internal
//!    seams stronger than its top seam, so the margin term taxed
//!    exactly the sprites the rule exists for (it held the correctly
//!    segmented f_fridge at 0.55).
//!
//! ## Per-id boundary overrides
//!
//! Stacked-seam furniture defeats the scored rule: f_locker's mid
//! door-seam (edge 0.382 at row 38) beats its true cap seam (edge
//! 0.076 at row 28 — the cap is a 2-row rim whose lower row is the
//! body's own gray, so neither the edge nor the L-step floor can see
//! it). Rather than complicate the global rule, [`segment_for`]
//! consults [`BOUNDARY_OVERRIDES`]: a static id -> boundary-row
//! table. The forced boundary still goes through silhouette gating
//! and side detection, and its confidence is computed from the
//! forced seam's own (weak) signals for honesty, but the confidence
//! floor is bypassed — a human verified the boundary. The table
//! lives here, not in `overrides.toml`, because `segment` is a pure
//! sprite function with no id-carrying caller yet, and the TOML
//! mechanism is consumed at material-build time for mode/height;
//! when engine stamping lands, the table can move there.
//! f_locker's 28 was verified against the sprite's row profile:
//! rows 26..27 are the cap (row 26 darker rim, L 0.375; row 27 the
//! sprite's brightest row, L 0.588), the body widens from x-extent
//! 0..30 to 0..31 at row 28, and L settles to the door field's flat
//! ~0.49 from row 28 down.
//!
//! ## Base structure (box model)
//!
//! A solid cuboid lies for legged furniture: f_table's front is a
//! 1-row apron over legs at x 1..=3 and 28..=30, with the under-
//! space painted as translucent shadow (RGBA alpha 128, pure
//! black). The box therefore classifies each front-extent column by
//! its bottom-row pixel: *grounded* when that pixel is fully opaque
//! (alpha >= 254 — solid paint; the shadow sits at exactly 128),
//! *open* otherwise. If the open fraction reaches
//! [`OPEN_BASE_FRACTION`] and the front opens with a solid header
//! band (>= [`HEADER_SOLID_FRACTION`] fully-opaque columns for at
//! least one row), the model is slab-and-legs: the header band plus
//! the top face form the slab, each maximal grounded run becomes a
//! leg extruded through the full model depth (front plane verbatim,
//! interior side-shaded, back plane darkened), and the under-space
//! stays empty. Measured separation on the gate set: f_table open
//! fraction 0.81 / header 1 row; the solid fronts are f_counter
//! 0.10, f_bookcase 0.00, f_fridge 0.16, f_locker 0.00. If no column
//! is grounded (a wall-mounted shelf would qualify), the solid box
//! is kept — a floating slab is worse than a solid block.
//!
//! ## Failure modes (all return `None`, caller keeps the extrusion)
//!
//! * Low coverage / ragged silhouette: chairs, creatures, plants.
//! * Top-down sprites (f_table is borderline: it segments, with its
//!   large top region read as depth) and flat textures (t_grass):
//!   no seam passes the edge + L-step floors.
//! * Face-on sprites with no drawn top surface (some doors, signs):
//!   no valid candidate.
//! * Stacked-seam furniture (a locker is two doors with a dark mid
//!   seam): the *first-strong-edge* rule can pick the mid seam,
//!   reading the upper door as a very deep "top". `segment` still
//!   does this for f_locker; the fix is the per-id override in
//!   [`BOUNDARY_OVERRIDES`] (via [`segment_for`]), not a cleverer
//!   global rule.
//! * Sprites taller than the tile (32x64 lockers/fridges): handled —
//!   the model grid follows the sprite, height is not clamped to 32.

use crate::analyze::analyze_image;
use crate::color::Oklab;
use image::{Rgba, RgbaImage};
use std::ops::Range;

/// Alpha at or above this counts as opaque (same cutout convention
/// as the analyzer).
const ALPHA_OPAQUE: u8 = 128;
/// A row belongs to the silhouette at this opaque coverage.
const MIN_ROW_COVERAGE: f32 = 0.30;
/// Mean coverage over silhouette rows required to attempt a split.
const MIN_MEAN_COVERAGE: f32 = 0.60;
/// Overall opaque fraction required to attempt a split.
const MIN_OPAQUE_FRACTION: f32 = 0.35;
/// The top region must be at least this many sprite rows.
const MIN_TOP_ROWS: u32 = 2;
/// The front region must be at least this many sprite rows...
const MIN_FRONT_ROWS: u32 = 4;
/// ...and at least this fraction of the silhouette band.
const MIN_FRONT_FRACTION: f32 = 0.15;
/// Boundary edge (mean OKLab distance across the seam) floor.
const EDGE_FLOOR: f32 = 0.12;
/// Local mean-L step across the boundary floor.
const LSTEP_FLOOR: f32 = 0.03;
/// The chosen boundary is the first valid candidate whose edge is
/// at least this fraction of the strongest valid edge.
const FIRST_EDGE_BAND: f32 = 0.6;
/// Segmentation is only reported at or above this confidence.
const CONFIDENCE_FLOOR: f32 = 0.45;
/// Per-column seam break that counts toward seam support.
const SUPPORT_FLOOR: f32 = 0.08;
/// Alpha at or above this is *solid paint* (structure). Painted
/// contact shadow in UltiCa furniture sprites sits at alpha 128;
/// the object itself is painted fully opaque.
const SOLID_ALPHA: u8 = 254;
/// Open-base box variant fires at or above this fraction of
/// front-extent columns not grounded at the base.
const OPEN_BASE_FRACTION: f32 = 0.25;
/// A front row belongs to the solid header band when at least this
/// fraction of its extent columns are solid paint.
const HEADER_SOLID_FRACTION: f32 = 0.90;

/// Per-id top/front boundary overrides (sprite row of the first
/// front row), consulted by [`segment_for`] before the scored
/// choice. See the module docs ("Per-id boundary overrides") for
/// why this is a static table and how each entry was verified.
///
/// * `f_locker` -> 28: cap is rows 26..27 (dark rim + brightest
///   row); the scored rule picks the mid door-seam at 38 instead.
pub const BOUNDARY_OVERRIDES: &[(&str, u32)] = &[("f_locker", 28)];
/// Sub-cells of depth per sprite row of top region. UltiCa's 3/4
/// view foreshortens the top surface roughly 2:1 vertically, so one
/// drawn top row corresponds to ~2 sub-cells of real depth.
pub const DEPTH_PER_TOP_ROW: f32 = 2.0;
/// Depth is clamped to the tile footprint.
const MAX_DEPTH: u32 = 32;

/// A detected side sliver: a vertical band of sprite columns at one
/// edge of the front region, shaded distinctly from the front field
/// (the object's side face caught by the 3/4 view).
#[derive(Debug, Clone, PartialEq)]
pub struct SideSliver {
    /// Sprite column range of the sliver.
    pub cols: Range<u32>,
    /// True if the sliver is at the sprite's right edge.
    pub on_right: bool,
}

/// The result of segmenting a furniture sprite into the regions the
/// 3/4 drawing convention encodes. All ranges are sprite pixel
/// coordinates; `front_rows.end` is exclusive like `top_rows`.
#[derive(Debug, Clone, PartialEq)]
pub struct Segmentation {
    /// Sprite rows of the silhouette band (opaque object rows).
    pub silhouette_rows: Range<u32>,
    /// Sprite rows of the top surface region.
    pub top_rows: Range<u32>,
    /// Sprite rows of the front face region.
    pub front_rows: Range<u32>,
    /// Side sliver within the front region, if one was detected.
    pub side: Option<SideSliver>,
    /// 0..=1 confidence in the split (see module docs).
    pub confidence: f32,
}

impl Segmentation {
    /// The boundary row: first row of the front region.
    pub fn boundary(&self) -> u32 {
        self.front_rows.start
    }
    /// Depth in sub-cells implied by the top region's height.
    pub fn depth_cells(&self) -> u32 {
        let th = self.top_rows.end - self.top_rows.start;
        ((th as f32 * DEPTH_PER_TOP_ROW).round() as u32).clamp(2, MAX_DEPTH)
    }
}

/// A dense voxel prop model on a sub-cell grid. `size` is
/// `[width(x), height(y), depth(z)]`; the footprint follows the
/// sprite (width = sprite width, depth from the segmentation).
/// Voxel (x, y, z): x is the sprite column, y = 0 is the object's
/// bottom, z = depth-1 is the front (the face the sprite shows).
#[derive(Debug, Clone, PartialEq)]
pub struct PropModel {
    pub size: [u32; 3],
    /// Row-major voxels, index `(y * depth + z) * width + x`.
    pub voxels: Vec<Option<Rgba<u8>>>,
}

impl PropModel {
    pub fn new(size: [u32; 3]) -> Self {
        let n = (size[0] * size[1] * size[2]) as usize;
        PropModel {
            size,
            voxels: vec![None; n],
        }
    }
    fn idx(&self, x: u32, y: u32, z: u32) -> usize {
        ((y * self.size[2] + z) * self.size[0] + x) as usize
    }
    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<Rgba<u8>> {
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return None;
        }
        self.voxels[self.idx(x, y, z)]
    }
    pub fn set(&mut self, x: u32, y: u32, z: u32, c: Option<Rgba<u8>>) {
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return;
        }
        let i = self.idx(x, y, z);
        self.voxels[i] = c;
    }
    pub fn solid_count(&self) -> usize {
        self.voxels.iter().filter(|v| v.is_some()).count()
    }
}

fn opaque(px: Rgba<u8>) -> bool {
    px[3] >= ALPHA_OPAQUE
}

fn lab_of(px: Rgba<u8>) -> Oklab {
    Oklab::from_srgb_u8([px[0], px[1], px[2]])
}

/// Scale a color's RGB by `f` (alpha forced opaque).
fn shade(px: Rgba<u8>, f: f32) -> Rgba<u8> {
    let m = |v: u8| (v as f32 * f).round().clamp(0.0, 255.0) as u8;
    Rgba([m(px[0]), m(px[1]), m(px[2]), 255])
}

/// Segment a furniture sprite into top / front ( / side) regions.
/// Returns `None` when the sprite has no confident box-like reading
/// — the caller then keeps the flat extrusion (see module docs).
pub fn segment(sprite: &RgbaImage) -> Option<Segmentation> {
    segment_inner(sprite, None)
}

/// [`segment`] with per-id boundary overrides: when `id` is in
/// [`BOUNDARY_OVERRIDES`], the segmentation is built at the forced
/// boundary (silhouette gating and side detection still apply; the
/// confidence floor is bypassed, see module docs).
pub fn segment_for(id: &str, sprite: &RgbaImage) -> Option<Segmentation> {
    let forced = BOUNDARY_OVERRIDES
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, b)| *b);
    segment_inner(sprite, forced)
}

fn segment_inner(sprite: &RgbaImage, forced: Option<u32>) -> Option<Segmentation> {
    let (w, h) = (sprite.width(), sprite.height());
    if w < 8 || h < 8 {
        return None;
    }
    // Per-row opaque coverage and mean L; overall opaque fraction.
    let mut row_cov = vec![0.0f32; h as usize];
    let mut row_mean_l = vec![0.0f32; h as usize];
    let mut total_opaque = 0u32;
    for y in 0..h {
        let mut n = 0u32;
        let mut lsum = 0.0f32;
        for x in 0..w {
            let px = *sprite.get_pixel(x, y);
            if opaque(px) {
                n += 1;
                lsum += lab_of(px).l;
            }
        }
        row_cov[y as usize] = n as f32 / w as f32;
        row_mean_l[y as usize] = if n > 0 { lsum / n as f32 } else { 0.0 };
        total_opaque += n;
    }
    if (total_opaque as f32) / ((w * h) as f32) < MIN_OPAQUE_FRACTION {
        return None;
    }
    // Silhouette band: first..=last row with enough coverage.
    let first = (0..h).find(|&y| row_cov[y as usize] >= MIN_ROW_COVERAGE)?;
    let last = (0..h)
        .rev()
        .find(|&y| row_cov[y as usize] >= MIN_ROW_COVERAGE)?;
    if last <= first {
        return None;
    }
    let band_h = last - first + 1;
    let in_band = &row_cov[first as usize..=last as usize];
    let thin_rows = in_band.iter().filter(|&&c| c < MIN_ROW_COVERAGE).count();
    if thin_rows as f32 > 0.10 * band_h as f32 {
        return None; // ragged silhouette: cutout art, not a box
    }
    let mean_cov = in_band.iter().sum::<f32>() / band_h as f32;
    if mean_cov < MIN_MEAN_COVERAGE {
        return None;
    }
    // Horizontal seam stats at each row boundary b (between b-1 and
    // b): mean OKLab distance over columns opaque on both sides,
    // plus *support* — the fraction of those columns whose own
    // break clears SUPPORT_FLOOR.
    let seam_at = |b: u32| -> (f32, f32) {
        let mut sum = 0.0f32;
        let mut n = 0u32;
        let mut strong = 0u32;
        for x in 0..w {
            let a = *sprite.get_pixel(x, b - 1);
            let c = *sprite.get_pixel(x, b);
            if opaque(a) && opaque(c) {
                let d = lab_of(a).distance(lab_of(c));
                sum += d;
                n += 1;
                if d >= SUPPORT_FLOOR {
                    strong += 1;
                }
            }
        }
        if n as f32 >= 0.25 * w as f32 {
            (sum / n as f32, strong as f32 / n as f32)
        } else {
            (0.0, 0.0)
        }
    };
    // Local L step at b: 4-row windows either side of the seam.
    let lstep_at = |b: u32| -> f32 {
        let above: Vec<f32> = (b.saturating_sub(4).max(first)..b)
            .map(|y| row_mean_l[y as usize])
            .collect();
        let below: Vec<f32> = (b..(b + 4).min(last + 1))
            .map(|y| row_mean_l[y as usize])
            .collect();
        if above.is_empty() || below.is_empty() {
            return 0.0;
        }
        let ma = above.iter().sum::<f32>() / above.len() as f32;
        let mb = below.iter().sum::<f32>() / below.len() as f32;
        (mb - ma).abs()
    };
    let min_front = MIN_FRONT_ROWS.max((MIN_FRONT_FRACTION * band_h as f32).ceil() as u32);
    // Region L step: top region mean L vs the front's first rows.
    // A secondary surface-difference signal for seams whose local
    // step is diluted by a bright trim row at the boundary.
    let region_step = |b: u32| -> f32 {
        let mean_rows = |r: std::ops::Range<u32>| -> f32 {
            let v: Vec<f32> = r.map(|y| row_mean_l[y as usize]).collect();
            if v.is_empty() {
                0.0
            } else {
                v.iter().sum::<f32>() / v.len() as f32
            }
        };
        let top = mean_rows(first..b);
        let front = mean_rows(b..(b + 8).min(last + 1));
        (front - top).abs()
    };
    // Confidence assembly (module docs, step 5).
    let confidence_of = |b: u32, edge: f32, lstep: f32, support: f32| -> f32 {
        let orient_term = orientation_term(sprite, first, b, last);
        (0.35 * (edge / 0.25).min(1.0)
            + 0.20 * (lstep / 0.15).min(1.0)
            + 0.15 * support
            + 0.15 * mean_cov
            + 0.05 * (region_step(b) / 0.15).min(1.0)
            + orient_term)
            .clamp(0.0, 1.0)
    };
    let b = if let Some(fb) = forced {
        // Per-id override: the boundary is human-verified, so the
        // edge/L-step floors and the confidence floor do not apply;
        // the range sanity checks still do.
        if fb < first + MIN_TOP_ROWS || fb > last + 1 - min_front {
            return None;
        }
        fb
    } else {
        let mut candidates: Vec<(u32, f32, f32)> = Vec::new(); // (b, edge, lstep)
        for b in (first + MIN_TOP_ROWS)..=(last + 1 - min_front) {
            let (e, _) = seam_at(b);
            let s = lstep_at(b);
            if e >= EDGE_FLOOR && s >= LSTEP_FLOOR {
                candidates.push((b, e, s));
            }
        }
        if candidates.is_empty() {
            return None;
        }
        let best_edge = candidates.iter().map(|c| c.1).fold(0.0f32, f32::max);
        let &(b, edge, lstep) = candidates
            .iter()
            .find(|c| c.1 >= FIRST_EDGE_BAND * best_edge)
            .unwrap();
        let (_, support) = seam_at(b);
        if confidence_of(b, edge, lstep, support) < CONFIDENCE_FLOOR {
            return None;
        }
        b
    };
    let (edge, support) = seam_at(b);
    let confidence = confidence_of(b, edge, lstep_at(b), support);
    let side = detect_side(sprite, b, last);
    Some(Segmentation {
        silhouette_rows: first..last + 1,
        top_rows: first..b,
        front_rows: b..last + 1,
        side,
        confidence,
    })
}

/// Orientation corroboration term in [-0.05, +0.10] (see module
/// docs, step 4).
fn orientation_term(sprite: &RgbaImage, top: u32, b: u32, last: u32) -> f32 {
    let (w, _) = (sprite.width(), sprite.height());
    let crop = |y0: u32, y1: u32| {
        image::imageops::crop_imm(sprite, 0, y0, w, y1 - y0).to_image()
    };
    let top_a = analyze_image(&crop(top, b), 1).orientation;
    let front_a = analyze_image(&crop(b, last + 1), 1).orientation;
    let top_coh = top_a.coherence >= 0.15;
    let front_coh = front_a.coherence >= 0.15;
    match (top_coh, front_coh) {
        (true, true) => {
            let d = (top_a.feature_angle_deg - front_a.feature_angle_deg)
                .abs()
                .min(180.0 - (top_a.feature_angle_deg - front_a.feature_angle_deg).abs());
            if d >= 20.0 {
                0.10
            } else if d < 10.0 {
                -0.05
            } else {
                0.0
            }
        }
        (true, false) | (false, true) => 0.05,
        (false, false) => 0.0,
    }
}

/// Detect a side sliver in the front region: an edge band
/// (3..=w/3 columns, either side) whose mean L differs from the rest
/// of the front by >= 0.06 with a consistent sign on >= 80% of front
/// rows. Among all passing bands, the one with the *largest*
/// contrast wins — the true sliver boundary is where the band/rest
/// contrast peaks; a wider band that bleeds into the front field
/// dilutes its own contrast. See module docs.
fn detect_side(sprite: &RgbaImage, front_top: u32, front_bot: u32) -> Option<SideSliver> {
    let w = sprite.width();
    let rows: Vec<u32> = (front_top..=front_bot).collect();
    if rows.len() < 4 || w < 12 {
        return None;
    }
    let row_mean_l = |y: u32, xs: &Range<u32>| -> Option<f32> {
        let mut s = 0.0f32;
        let mut n = 0u32;
        for x in xs.clone() {
            let px = *sprite.get_pixel(x, y);
            if opaque(px) {
                s += lab_of(px).l;
                n += 1;
            }
        }
        if n >= 2 {
            Some(s / n as f32)
        } else {
            None
        }
    };
    let mut best: Option<(f32, SideSliver)> = None;
    for on_right in [true, false] {
        for k in 3..=(w / 3) {
            let (band, rest) = if on_right {
                ((w - k)..w, 0..(w - k))
            } else {
                (0..k, k..w)
            };
            let mut diffs = Vec::new();
            for &y in &rows {
                if let (Some(lb), Some(lr)) = (row_mean_l(y, &band), row_mean_l(y, &rest)) {
                    diffs.push(lb - lr);
                }
            }
            if diffs.len() < rows.len() * 3 / 4 {
                continue;
            }
            let mean_diff = diffs.iter().sum::<f32>() / diffs.len() as f32;
            if mean_diff.abs() < 0.06 {
                continue;
            }
            let consistent = diffs
                .iter()
                .filter(|d| d.signum() == mean_diff.signum() && d.abs() >= 0.03)
                .count();
            if consistent as f32 >= 0.8 * diffs.len() as f32
                && best.as_ref().map_or(true, |(bd, _)| mean_diff.abs() > *bd + 1e-4)
            {
                best = Some((
                    mean_diff.abs(),
                    SideSliver {
                        cols: band,
                        on_right,
                    },
                ));
            }
        }
    }
    best.map(|(_, sl)| sl)
}

/// The sprite pixel at (x, r), or the nearest opaque pixel in row r
/// within [x0, x1] (left neighbor first). None only if the whole row
/// band is transparent.
fn row_pixel(sprite: &RgbaImage, x: u32, r: u32, x0: u32, x1: u32) -> Option<Rgba<u8>> {
    let px = *sprite.get_pixel(x, r);
    if opaque(px) {
        return Some(px);
    }
    for d in 1..=(x1 - x0) {
        if x >= d && x - d >= x0 {
            let p = *sprite.get_pixel(x - d, r);
            if opaque(p) {
                return Some(p);
            }
        }
        if x + d <= x1 {
            let p = *sprite.get_pixel(x + d, r);
            if opaque(p) {
                return Some(p);
            }
        }
    }
    None
}

/// Mean color of a sprite region's opaque pixels (alpha forced 255).
fn region_mean(sprite: &RgbaImage, rows: &Range<u32>) -> Rgba<u8> {
    let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
    for y in rows.clone() {
        for x in 0..sprite.width() {
            let px = *sprite.get_pixel(x, y);
            if opaque(px) {
                r += px[0] as u64;
                g += px[1] as u64;
                b += px[2] as u64;
                n += 1;
            }
        }
    }
    if n == 0 {
        return Rgba([128, 128, 128, 255]);
    }
    Rgba([
        (r / n) as u8,
        (g / n) as u8,
        (b / n) as u8,
        255,
    ])
}

/// Map a depth cell z (0 = back, d-1 = front) to the top-region
/// sprite row it un-projects from: z = d-1 is the top region's last
/// row (the front-top edge), rows repeat/stretch by nearest sample.
fn top_row_for_z(seg: &Segmentation, d: u32, z: u32) -> u32 {
    let th = seg.top_rows.end - seg.top_rows.start;
    let back_steps = d - 1 - z;
    let row_back = (back_steps * th / d).min(th - 1);
    seg.top_rows.end - 1 - row_back
}

/// Solid paint: fully opaque (see [`SOLID_ALPHA`]).
fn solid(px: Rgba<u8>) -> bool {
    px[3] >= SOLID_ALPHA
}

/// Base structure of a front region (module docs, "Base
/// structure"): whether the box is solid or slab-and-legs, the
/// header band height, and each extent column's groundedness.
struct BasePlan {
    open: bool,
    header_rows: u32,
    /// Per extent column (x0..=x1): bottom-row pixel is solid paint.
    grounded: Vec<bool>,
}

fn base_plan(sprite: &RgbaImage, seg: &Segmentation, x0: u32, x1: u32) -> BasePlan {
    let front = &seg.front_rows;
    let bot = front.end - 1;
    let ext = (x1 - x0 + 1) as f32;
    let grounded: Vec<bool> = (x0..=x1)
        .map(|x| solid(*sprite.get_pixel(x, bot)))
        .collect();
    let open_frac = grounded.iter().filter(|g| !**g).count() as f32 / ext;
    // Header band: consecutive front rows, from the boundary down,
    // that are solid paint across (nearly) the whole extent.
    let mut header_rows = 0u32;
    for r in front.clone() {
        let s = (x0..=x1)
            .filter(|&x| solid(*sprite.get_pixel(x, r)))
            .count();
        if s as f32 >= HEADER_SOLID_FRACTION * ext {
            header_rows += 1;
        } else {
            break;
        }
    }
    let fh = front.end - front.start;
    let any_grounded = grounded.iter().any(|g| *g);
    let open = open_frac >= OPEN_BASE_FRACTION
        && header_rows >= 1
        && header_rows < fh
        && any_grounded;
    BasePlan {
        open,
        header_rows,
        grounded,
    }
}

/// G.1: the box model. Deprecated — superseded by [`solid_model`],
/// which is the canonical prop model; kept for the gate and as a
/// reference implementation of the face un-projections.
///
/// A cuboid spanning the front region's opaque
/// x-extent, height = front region height, depth from the top
/// region. Faces are textured by un-projecting their sprite region;
/// the interior is filled with the front region's mean color (it is
/// never visible in a surface render, but a solid model keeps later
/// engine stamping trivially correct). When the front's base is
/// open (see [`base_plan`]), the model is slab-and-legs instead:
/// the header band + top face form a slab and the under-space
/// between the legs stays empty.
pub fn box_model(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let d = seg.depth_cells();
    // Front region's opaque x-extent.
    let (mut x0, mut x1) = (w - 1, 0u32);
    for y in front.clone() {
        for x in 0..w {
            if opaque(*sprite.get_pixel(x, y)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    let plan = base_plan(sprite, seg, x0, x1);
    if plan.open {
        return box_model_open(sprite, seg, x0, x1, &plan);
    }
    let interior = region_mean(sprite, front);
    let mut m = PropModel::new([w, fh, d]);
    for y in 0..fh {
        for z in 0..d {
            for x in x0..=x1 {
                m.set(x, y, z, Some(interior));
            }
        }
    }
    let bot = front.end - 1;
    // Front (z = d-1) and back (z = 0, darkened) faces.
    for x in x0..=x1 {
        for r in front.clone() {
            let y = bot - r;
            if let Some(px) = row_pixel(sprite, x, r, x0, x1) {
                m.set(x, y, d - 1, Some(px));
                m.set(x, y, 0, Some(shade(px, 0.55)));
            }
        }
    }
    // Top face (y = fh-1): un-project the top region across depth.
    // The front plane (z = d-1) is owned by the front face, so the
    // top stops one cell short of it; same for the bottom face.
    for x in x0..=x1 {
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            if let Some(px) = row_pixel(sprite, x, sr, x0, x1) {
                m.set(x, fh - 1, z, Some(px));
            }
        }
    }
    // Bottom face (y = 0): front's bottom row, darkened.
    for x in x0..=x1 {
        for z in 0..d - 1 {
            if let Some(px) = row_pixel(sprite, x, bot, x0, x1) {
                m.set(x, 0, z, Some(shade(px, 0.5)));
            }
        }
    }
    // Side faces (x = x0 and x = x1), interior cells only: edge
    // rows/columns belong to the front/back/top/bottom faces.
    for (sx, on_right) in [(x0, false), (x1, true)] {
        for y in 1..fh - 1 {
            let r = bot - y;
            for z in 1..d - 1 {
                let c = side_color(sprite, seg, sx, r, z, d, on_right);
                if let Some(px) = c {
                    m.set(sx, y, z, Some(px));
                }
            }
        }
    }
    m
}

/// The slab-and-legs box variant for open-base fronts (see
/// [`base_plan`]). The slab is the header band across the full
/// extent and depth, capped by the un-projected top face; each
/// maximal grounded column run becomes a leg extruded through the
/// full depth below the slab; everything else under the slab is
/// empty.
fn box_model_open(
    sprite: &RgbaImage,
    seg: &Segmentation,
    x0: u32,
    x1: u32,
    plan: &BasePlan,
) -> PropModel {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let d = seg.depth_cells();
    let bot = front.end - 1;
    let slab_bottom = fh - plan.header_rows; // lowest slab row (model y)
    let interior = region_mean(sprite, front);
    let mut m = PropModel::new([w, fh, d]);
    // Slab body + front/back faces per slab row.
    for y in slab_bottom..fh {
        let r = bot - y;
        for z in 0..d {
            for x in x0..=x1 {
                m.set(x, y, z, Some(interior));
            }
        }
        for x in x0..=x1 {
            if let Some(px) = row_pixel(sprite, x, r, x0, x1) {
                m.set(x, y, d - 1, Some(px));
                m.set(x, y, 0, Some(shade(px, 0.55)));
            }
        }
    }
    // Slab underside: the lowest slab row's interior cells take the
    // header row's color darkened like the solid box's bottom face
    // (they are what a from-below view sees between the legs).
    let r_under = bot - slab_bottom;
    for x in x0..=x1 {
        if let Some(px) = row_pixel(sprite, x, r_under, x0, x1) {
            for z in 1..d - 1 {
                m.set(x, slab_bottom, z, Some(shade(px, 0.5)));
            }
        }
    }
    // Top face: same un-projection as the solid box.
    for x in x0..=x1 {
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            if let Some(px) = row_pixel(sprite, x, sr, x0, x1) {
                m.set(x, fh - 1, z, Some(px));
            }
        }
    }
    // Slab side faces at the extent edges.
    for (sx, on_right) in [(x0, false), (x1, true)] {
        for y in slab_bottom..fh - 1 {
            let r = bot - y;
            for z in 1..d - 1 {
                if let Some(px) = side_color(sprite, seg, sx, r, z, d, on_right) {
                    m.set(sx, y, z, Some(px));
                }
            }
        }
    }
    // Legs: maximal grounded runs extruded through the full depth.
    let mut x = x0;
    while x <= x1 {
        if !plan.grounded[(x - x0) as usize] {
            x += 1;
            continue;
        }
        let run_start = x;
        while x <= x1 && plan.grounded[(x - x0) as usize] {
            x += 1;
        }
        for lx in run_start..x {
            for y in 0..slab_bottom {
                let r = bot - y;
                if let Some(px) = row_pixel(sprite, lx, r, x0, x1) {
                    m.set(lx, y, d - 1, Some(px));
                    m.set(lx, y, 0, Some(shade(px, 0.55)));
                    for z in 1..d - 1 {
                        m.set(lx, y, z, Some(shade(px, 0.7)));
                    }
                }
            }
        }
    }
    m
}

/// Color for a box side-face cell: the detected sliver un-projected
/// when this side has one, else the front's edge column darkened
/// (the faces.rs convention for synthesized sides).
fn side_color(
    sprite: &RgbaImage,
    seg: &Segmentation,
    edge_x: u32,
    r: u32,
    z: u32,
    d: u32,
    on_right: bool,
) -> Option<Rgba<u8>> {
    if let Some(sl) = &seg.side {
        if sl.on_right == on_right {
            let sw = sl.cols.end - sl.cols.start;
            // Front of the model (z = d-1) meets the sliver column
            // nearest the front face; walk back across the sliver.
            let back_steps = d - 1 - z;
            let off = (back_steps * sw / d).min(sw - 1);
            let sx = if on_right {
                sl.cols.end - 1 - off
            } else {
                sl.cols.start + off
            };
            let px = *sprite.get_pixel(sx, r);
            if opaque(px) {
                return Some(shade(px, 0.85));
            }
        }
    }
    let px = *sprite.get_pixel(edge_x, r);
    if opaque(px) {
        Some(shade(px, 0.7))
    } else {
        None
    }
}

/// G.2: the column model. Deprecated — superseded by
/// [`solid_model`], which is the canonical prop model; kept for the
/// gate and as a reference. (Its shell construction is what left a
/// visible open wedge on deep props' flanks: closure strips only
/// join the walls at each row's silhouette extremes.)
///
/// A closed shell of surfaces:
///
/// * the **front wall** at z = depth-1: for every opaque front-region
///   pixel (x, r), one voxel at height `bot - r` with that pixel's
///   color — the model's front projection is the sprite's front
///   region, exactly;
/// * the **top cap** at y = front_height-1: the top region
///   un-projected across the depth (same mapping as the box),
///   extending backward from the front wall's top;
/// * the **back panel** at z = 0: the front silhouette mirrored in
///   depth, per-pixel colors darkened by the side factor (0.7);
/// * **side closure strips**: at each front row's opaque x-extremes,
///   a run of cells across the full depth, side-shaded — they join
///   the front wall to the back panel along both flanks;
/// * a **bottom strip** at y = 0 joining front wall to back panel
///   under every column that reaches the base.
///
/// The interior stays hollow (invisible); from any outside angle
/// the model reads as a closed object. Closure never writes the
/// front plane (z = depth-1), so the exact front projection is
/// preserved.
pub fn column_model(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let d = seg.depth_cells();
    let bot = front.end - 1;
    let mut m = PropModel::new([w, fh, d]);
    // Top cap: extends backward from just behind the front wall.
    // The front plane (z = d-1) is owned by the front wall alone,
    // so the front projection stays the sprite's exactly.
    for x in 0..w {
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            let px = *sprite.get_pixel(x, sr);
            if opaque(px) {
                m.set(x, fh - 1, z, Some(px));
            }
        }
    }
    // Front wall.
    for x in 0..w {
        for r in front.clone() {
            let px = *sprite.get_pixel(x, r);
            if opaque(px) {
                m.set(x, bot - r, d - 1, Some(px));
            }
        }
    }
    // Bottom closure strip (z between the walls; the back panel
    // below owns z = 0).
    for x in 0..w {
        let px = *sprite.get_pixel(x, bot);
        if opaque(px) {
            for z in 1..d - 1 {
                m.set(x, 0, z, Some(shade(px, 0.5)));
            }
        }
    }
    // Back panel: the front silhouette mirrored in depth, darkened
    // by the side factor.
    for x in 0..w {
        for r in front.clone() {
            let px = *sprite.get_pixel(x, r);
            if opaque(px) {
                m.set(x, bot - r, 0, Some(shade(px, 0.7)));
            }
        }
    }
    // Side closure strips at each front row's opaque extremes.
    for r in front.clone() {
        let y = bot - r;
        let (mut rx0, mut rx1) = (w, 0u32);
        for x in 0..w {
            if opaque(*sprite.get_pixel(x, r)) {
                rx0 = rx0.min(x);
                rx1 = rx1.max(x);
            }
        }
        if rx0 > rx1 {
            continue; // empty row
        }
        for sx in [rx0, rx1] {
            let px = *sprite.get_pixel(sx, r);
            for z in 1..d - 1 {
                m.set(sx, y, z, Some(shade(px, 0.7)));
            }
        }
    }
    m
}

/// THE prop model: the front silhouette extruded through the full
/// model depth and filled at *every* depth cell — a real solid, so
/// no viewing angle can see into a hollow interior or a side gap
/// (the failure that retired the G.2 shell).
///
/// * Cross-section: voxel (x, y, z) is solid iff the front-region
///   pixel at (x, front_bottom - y) is opaque and z < d, with
///   d = [`Segmentation::depth_cells`] uniform for every silhouette
///   column. (Depth does not vary per column: the top region sets
///   the one depth, exactly as in the earlier models.)
/// * Front plane (z = d-1): the sprite pixel verbatim — the front
///   projection is the front silhouette exactly, colors included.
///   Nothing below overwrites the front plane.
/// * Back plane (z = 0): the front pixel darkened x0.55. Interior
///   cells: x0.7 (never visible in a surface render, but correct if
///   the model is sliced).
/// * Side surfaces: each row's leftmost/rightmost solid columns are
///   real surface (the model is filled behind them), colored on the
///   interior depth cells by [`side_color`] — the sliver
///   un-projection where a sliver was detected, else the front
///   pixel x0.7. The front/back planes keep their own colors, so
///   side shading never touches the exact front projection.
/// * Top surface: stepped, following the silhouette — each column
///   x's top surface is its own topmost solid voxel, colored for
///   z in 0..d-1 from the top region via [`top_row_for_z`] (the
///   box's un-projection). The front plane of that voxel keeps the
///   front pixel, so the top reads as a rim behind the front-top
///   edge, as in the box.
/// * Open base ([`base_plan`]): where the front's base is open
///   (tables), the slab rows (header band + top) stay solid across
///   the full silhouette, below the slab only grounded columns are
///   solid through the depth, and the under-space is empty at every
///   z — including the front plane, whose "silhouette" there is
///   painted shadow, not structure.
pub fn solid_model(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let d = seg.depth_cells();
    let bot = front.end - 1;
    // Front region's opaque x-extent (base_plan + row_pixel ranges).
    let (mut x0, mut x1) = (w - 1, 0u32);
    for y in front.clone() {
        for x in 0..w {
            if opaque(*sprite.get_pixel(x, y)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    let plan = base_plan(sprite, seg, x0, x1);
    let slab_bottom = fh - plan.header_rows; // model y; only if plan.open
    // The solidity mask: the whole model is this mask extruded
    // through every depth cell.
    let is_solid = |x: u32, y: u32| -> bool {
        if !opaque(*sprite.get_pixel(x, bot - y)) {
            return false;
        }
        if !plan.open || y >= slab_bottom {
            return true;
        }
        // Below the slab of an open base: grounded columns only.
        // (x is within the extent: its front pixel is opaque.)
        plan.grounded[(x - x0) as usize]
    };
    let mut m = PropModel::new([w, fh, d]);
    // Base fill: front verbatim, back x0.55, interior x0.7.
    for y in 0..fh {
        for x in 0..w {
            if !is_solid(x, y) {
                continue;
            }
            let px = *sprite.get_pixel(x, bot - y);
            for z in 0..d {
                let c = if z == d - 1 {
                    px
                } else if z == 0 {
                    shade(px, 0.55)
                } else {
                    shade(px, 0.7)
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Side surfaces: each row's silhouette extremes, interior depth
    // cells only (front/back planes keep their colors above).
    for y in 0..fh {
        let mut extremes: Option<(u32, u32)> = None;
        for x in 0..w {
            if is_solid(x, y) {
                extremes = Some(match extremes {
                    None => (x, x),
                    Some((lo, _)) => (lo, x),
                });
            }
        }
        let Some((lo, hi)) = extremes else { continue };
        let r = bot - y;
        for (edge_x, on_right) in [(lo, false), (hi, true)] {
            for z in 1..d - 1 {
                if let Some(c) = side_color(sprite, seg, edge_x, r, z, d, on_right) {
                    m.set(edge_x, y, z, Some(c));
                }
            }
        }
    }
    // Top surface: per column, its topmost solid voxel, colored
    // from the top region across the depth behind the front plane.
    for x in 0..w {
        let Some(ytop) = (0..fh).rev().find(|&y| is_solid(x, y)) else {
            continue;
        };
        for z in 0..d - 1 {
            let sr = top_row_for_z(seg, d, z);
            if let Some(px) = row_pixel(sprite, x, sr, x0, x1) {
                m.set(x, ytop, z, Some(px));
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic 3/4 furniture: bright top band rows 4..10, front
    /// rows 10..28 with vertical striping, right 6 columns of the
    /// front darker (side sliver), on transparency.
    fn synthetic() -> RgbaImage {
        let mut img = RgbaImage::new(32, 32);
        for y in 4..10 {
            for x in 0..32 {
                img.put_pixel(x, y, Rgba([210, 190, 150, 255]));
            }
        }
        for y in 10..28 {
            for x in 0..32 {
                let base = if x >= 26 {
                    [90, 70, 50, 255]
                } else if (x / 4) % 2 == 0 {
                    [150, 115, 80, 255]
                } else {
                    [140, 108, 76, 255]
                };
                img.put_pixel(x, y, Rgba(base));
            }
        }
        img
    }

    #[test]
    fn synthetic_segments_with_sliver() {
        let img = synthetic();
        let seg = segment(&img).expect("synthetic sprite must segment");
        assert_eq!(seg.top_rows, 4..10);
        assert_eq!(seg.front_rows, 10..28);
        assert_eq!(seg.boundary(), 10);
        let sl = seg.side.expect("right sliver must be detected");
        assert!(sl.on_right);
        assert_eq!(sl.cols, 26..32);
        assert!(seg.confidence >= 0.45, "confidence {}", seg.confidence);
    }

    /// Open-base synthetic: bright top band, a 2-row solid apron,
    /// then legs (fully opaque) at x 2..=4 and 27..=29 with the
    /// under-space painted as alpha-128 black shadow — the f_table
    /// structure. The Segmentation is built by hand so the test
    /// targets the base rule, not the seam choice.
    fn synthetic_open_base() -> (RgbaImage, Segmentation) {
        let mut img = RgbaImage::new(32, 32);
        for y in 2..8 {
            for x in 0..32 {
                img.put_pixel(x, y, Rgba([200, 180, 140, 255]));
            }
        }
        for y in 8..29 {
            for x in 0..32 {
                let px = if y < 10 || (2..=4).contains(&x) || (27..=29).contains(&x) {
                    Rgba([120, 90, 60, 255]) // apron + legs: solid paint
                } else {
                    Rgba([0, 0, 0, 128]) // painted shadow, not structure
                };
                img.put_pixel(x, y, px);
            }
        }
        let seg = Segmentation {
            silhouette_rows: 2..29,
            top_rows: 2..8,
            front_rows: 8..29,
            side: None,
            confidence: 1.0,
        };
        (img, seg)
    }

    #[test]
    fn open_base_box_is_slab_and_legs() {
        let (img, seg) = synthetic_open_base();
        let plan = base_plan(&img, &seg, 0, 31);
        assert!(plan.open, "shadow base must read as open");
        assert_eq!(plan.header_rows, 2, "apron is the header band");
        let bm = box_model(&img, &seg);
        let (fh, d) = (bm.size[1], bm.size[2]);
        // Under-space between the legs is empty at every depth.
        for y in 0..fh - plan.header_rows {
            for z in 0..d {
                assert_eq!(bm.get(16, y, z), None, "under-space at (16,{y},{z})");
            }
        }
        // Legs are filled through the depth at their columns.
        assert!(bm.get(3, 0, d - 1).is_some(), "front leg cell");
        assert!(bm.get(28, 0, 0).is_some(), "back leg cell");
        // The slab's front face is the apron, verbatim.
        assert_eq!(
            bm.get(16, fh - 1, d - 1),
            Some(Rgba([120, 90, 60, 255])),
            "slab front face"
        );
        assert_eq!(bm, box_model(&img, &seg), "deterministic");
    }

    #[test]
    fn column_model_is_a_closed_shell() {
        let img = synthetic();
        let seg = segment(&img).unwrap();
        let cm = column_model(&img, &seg);
        let (fh, d) = (cm.size[1], cm.size[2]);
        let bot = seg.front_rows.end - 1;
        // Back panel mirrors the front silhouette, side-shaded.
        for x in 0..32 {
            for y in 0..fh {
                let px = *img.get_pixel(x, bot - y);
                assert_eq!(
                    cm.get(x, y, 0),
                    Some(shade(px, 0.7)),
                    "back panel at ({x},{y})"
                );
            }
        }
        // Interior stays hollow.
        assert_eq!(cm.get(16, fh / 2, d / 2), None, "hollow interior");
        // Side strips close the flanks at the row extremes.
        assert!(cm.get(0, fh / 2, d / 2).is_some(), "left strip");
        assert!(cm.get(31, fh / 2, d / 2).is_some(), "right strip");
    }

    #[test]
    fn flat_and_sparse_do_not_segment() {
        let flat = RgbaImage::from_pixel(32, 32, Rgba([120, 120, 120, 255]));
        assert!(segment(&flat).is_none());
        let mut sparse = RgbaImage::new(32, 32);
        for i in 0..10 {
            sparse.put_pixel(i * 3, i * 2, Rgba([200, 50, 50, 255]));
        }
        assert!(segment(&sparse).is_none());
    }

    #[test]
    fn synthetic_models_project_exactly() {
        let img = synthetic();
        let seg = segment(&img).unwrap();
        let cm = column_model(&img, &seg);
        let d = cm.size[2];
        // Front projection == front region silhouette, exactly.
        for x in 0..32 {
            for y in 0..cm.size[1] {
                let want = opaque(*img.get_pixel(x, seg.front_rows.end - 1 - y));
                assert_eq!(cm.get(x, y, d - 1).is_some(), want, "at ({x},{y})");
            }
        }
        let bm = box_model(&img, &seg);
        assert_eq!(bm.size[1], 18, "box height = front height");
        assert!(bm.solid_count() > cm.solid_count());
        // Determinism.
        assert_eq!(bm, box_model(&img, &seg));
        assert_eq!(cm, column_model(&img, &seg));
    }

    #[test]
    fn solid_model_is_filled_and_projects_exactly() {
        let img = synthetic();
        let seg = segment(&img).unwrap();
        let sm = solid_model(&img, &seg);
        let (fh, d) = (sm.size[1], sm.size[2]);
        let bot = seg.front_rows.end - 1;
        // Cross-section == front silhouette, filled at EVERY z.
        for x in 0..32 {
            for y in 0..fh {
                let px = *img.get_pixel(x, bot - y);
                let want = opaque(px);
                for z in 0..d {
                    assert_eq!(sm.get(x, y, z).is_some(), want, "at ({x},{y},{z})");
                }
                if want {
                    assert_eq!(sm.get(x, y, d - 1), Some(px), "front verbatim at ({x},{y})");
                }
            }
        }
        assert_eq!(sm, solid_model(&img, &seg), "deterministic");
    }

    #[test]
    fn solid_model_open_base_keeps_only_legs_under_the_slab() {
        let (img, seg) = synthetic_open_base();
        let plan = base_plan(&img, &seg, 0, 31);
        assert!(plan.open);
        let sm = solid_model(&img, &seg);
        let (fh, d) = (sm.size[1], sm.size[2]);
        let slab_bottom = fh - plan.header_rows;
        // Under-space: empty at every depth, front plane included.
        for y in 0..slab_bottom {
            for z in 0..d {
                assert_eq!(sm.get(16, y, z), None, "under-space at (16,{y},{z})");
            }
        }
        // Legs: solid at every depth where the front pixel is paint.
        for x in [3u32, 28] {
            for y in 0..slab_bottom {
                for z in 0..d {
                    assert!(sm.get(x, y, z).is_some(), "leg at ({x},{y},{z})");
                }
            }
        }
        // Slab: solid across the width at every depth.
        for y in slab_bottom..fh {
            for z in 0..d {
                assert!(sm.get(16, y, z).is_some(), "slab at (16,{y},{z})");
            }
        }
    }
}
