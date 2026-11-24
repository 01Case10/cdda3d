//! Image quilting (Phase 5 rework, Stage D): Efros–Freeman
//! texture synthesis from exemplar sprites, plus the macro-ground
//! height layer and the CPU reference height blend.
//!
//! Stage C's recipes render a material *from its palette* through
//! periodic noise — clean and tileable, but the result has none of
//! the source sprite's actual pixel character (individual grass
//! blades, dirt clods). Quilting is the complementary path: output
//! pixels are copied from the exemplars themselves, patch by patch,
//! so the texture carries the real exemplar structure while the
//! construction below keeps it exactly tileable.
//!
//! # Torus construction (why the output tiles exactly)
//!
//! Synthesis happens on a torus, in three places at once:
//!
//! * **Patch candidates wrap around the exemplar.** A candidate
//!   patch whose origin sits near an exemplar edge reads its pixels
//!   modulo the exemplar size, so edge origins are as usable as
//!   interior ones (no border shrink of the candidate pool).
//! * **Blocks are placed on a toroidal canvas.** Block origins sit
//!   on a grid with `step = patch - overlap` dividing the output
//!   size, so the last block column/row wraps around and overlaps
//!   the first column/row by exactly `overlap` pixels — the wrap
//!   seam is an ordinary overlap, not a special case.
//! * **Overlap error is computed against the wrapped canvas**, and
//!   each overlap strip is composited through a minimum-error
//!   boundary cut (dynamic programming over the squared-error
//!   surface), exactly as in Efros–Freeman: the seam between old
//!   and new content runs where the two already agree best.
//!
//! * **The first column/row is re-placed last** (wrap
//!   relaxation). In raster order the first blocks are placed
//!   with the fewest constraints — block (0, 0) with none — so a
//!   second pass re-places the first column and row with strips
//!   on all four sides against the finished canvas.
//!
//! Every wrap adjacency in the finished texture is therefore
//! either patch-interior (real exemplar adjacency) or a min-error
//! cut scored against the actual neighbour pixels — there is no
//! untreated border anywhere.
//!
//! # Determinism
//!
//! Same exemplars + parameters + seed -> byte-identical output.
//! Candidates are enumerated in a fixed order (exemplar index,
//! then y, then x), scored over the full overlap, ties break
//! toward the lowest candidate index, and the final pick is
//! drawn by the seeded [`Rng`] from the per-exemplar near-best
//! set (see [`pick_candidate`]), which is the paper's stochastic
//! step made reproducible. Cut paths use the
//! same discipline: DP predecessors are scanned in a fixed order
//! with strict improvement, so ties resolve identically every run.

use image::{GrayImage, Luma, Rgba, RgbaImage};

use crate::color::Oklab;
use crate::noise::Rng;

/// Quilting geometry. `step = patch - overlap` must divide the
/// output size (see [`quilt`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuiltParams {
    /// Patch edge length in pixels.
    pub patch: u32,
    /// Overlap between adjacent blocks, per side. Must satisfy
    /// `2 * overlap <= patch` so the strips on opposite sides of a
    /// wrapping block never merge.
    pub overlap: u32,
}

impl QuiltParams {
    /// Distance between adjacent block origins.
    pub fn step(&self) -> u32 {
        self.patch - self.overlap
    }
}

/// One exemplar prepared for sampling: pixels in OKLab (for error
/// and cut costs) alongside the original RGBA (for output), with
/// transparent pixels replaced by the exemplar's mean opaque color
/// so a cutout hole can never leak stored-but-invisible RGB into
/// the synthesis.
struct Exemplar {
    w: u32,
    h: u32,
    lab: Vec<Oklab>,
    rgba: Vec<Rgba<u8>>,
}

impl Exemplar {
    fn new(img: &RgbaImage) -> Exemplar {
        let (w, h) = (img.width(), img.height());
        let mut sum = Oklab::BLACK;
        let mut n = 0u32;
        for p in img.pixels() {
            if p[3] >= 128 {
                let lab = Oklab::from_srgb_u8([p[0], p[1], p[2]]);
                sum.l += lab.l;
                sum.a += lab.a;
                sum.b += lab.b;
                n += 1;
            }
        }
        let fill = if n > 0 {
            let mean = Oklab {
                l: sum.l / n as f32,
                a: sum.a / n as f32,
                b: sum.b / n as f32,
            };
            let [r, g, b] = mean.to_srgb_u8();
            Rgba([r, g, b, 255])
        } else {
            Rgba([0, 0, 0, 255])
        };
        let mut lab = Vec::with_capacity((w * h) as usize);
        let mut rgba = Vec::with_capacity((w * h) as usize);
        for p in img.pixels() {
            let px = if p[3] >= 128 { *p } else { fill };
            rgba.push(px);
            lab.push(Oklab::from_srgb_u8([px[0], px[1], px[2]]));
        }
        Exemplar { w, h, lab, rgba }
    }

    /// Pixel at a patch-local offset from origin `(ox, oy)`,
    /// wrapping around the exemplar (torus sampling).
    fn lab_at(&self, ox: u32, oy: u32, dx: u32, dy: u32) -> Oklab {
        let x = (ox + dx) % self.w;
        let y = (oy + dy) % self.h;
        self.lab[(y * self.w + x) as usize]
    }

    fn rgba_at(&self, ox: u32, oy: u32, dx: u32, dy: u32) -> Rgba<u8> {
        let x = (ox + dx) % self.w;
        let y = (oy + dy) % self.h;
        self.rgba[(y * self.w + x) as usize]
    }
}

/// A patch candidate: exemplar index + origin inside it.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    ex: usize,
    x: u32,
    y: u32,
}

/// Squared OKLab distance (the quilting error surface; squared so
/// large disagreements dominate candidate choice and cut cost,
/// as in the paper's SSD overlap error).
fn err2(a: Oklab, b: Oklab) -> f32 {
    let (dl, da, db) = (a.l - b.l, a.a - b.a, a.b - b.b);
    dl * dl + da * da + db * db
}

/// Minimum-error boundary cut through one overlap strip.
///
/// `cost` is the squared-error surface of the strip, `w` pixels
/// wide (indexed across the strip) by `h` long (indexed along
/// it). Returns the seam column `s(y) in [0, w)` for every row:
/// the minimum-cost top-to-bottom path moving at most one column
/// per row. Ties resolve deterministically (straight predecessor
/// preferred, then -1, then +1; the final argmin takes the
/// lowest column).
fn min_error_cut(cost: &[f32], w: usize, h: usize) -> Vec<u32> {
    debug_assert_eq!(cost.len(), w * h);
    let mut dp = vec![0.0f32; w * h];
    for x in 0..w {
        dp[x] = cost[x];
    }
    for y in 1..h {
        for x in 0..w {
            // Predecessor scan order is part of determinism:
            // straight, then left, then right; strict < keeps the
            // earliest-scanned predecessor on ties.
            let mut best = dp[(y - 1) * w + x];
            if x > 0 {
                let v = dp[(y - 1) * w + x - 1];
                if v < best {
                    best = v;
                }
            }
            if x + 1 < w {
                let v = dp[(y - 1) * w + x + 1];
                if v < best {
                    best = v;
                }
            }
            dp[y * w + x] = cost[y * w + x] + best;
        }
    }
    // Backtrack from the cheapest bottom cell.
    let mut seam = vec![0u32; h];
    let mut x = 0usize;
    for cx in 1..w {
        if dp[(h - 1) * w + cx] < dp[(h - 1) * w + x] {
            x = cx;
        }
    }
    seam[h - 1] = x as u32;
    for y in (1..h).rev() {
        let mut bx = x;
        let mut best = dp[(y - 1) * w + x];
        if x > 0 {
            let v = dp[(y - 1) * w + x - 1];
            if v < best {
                best = v;
                bx = x - 1;
            }
        }
        if x + 1 < w {
            let v = dp[(y - 1) * w + x + 1];
            if v < best {
                bx = x + 1;
            }
        }
        x = bx;
        seam[y - 1] = x as u32;
    }
    seam
}

/// Synthesize a `size` x `size` exactly-tileable texture from
/// `exemplars` by torus quilting (see module docs).
///
/// `size % (patch - overlap)` must be 0 and the block grid must
/// have at least 2 blocks per axis (a single block per axis has
/// no neighbour to cut against on the wrap). Output is fully
/// opaque. Deterministic in `(exemplars, size, params, seed)`.
pub fn quilt(exemplars: &[RgbaImage], size: u32, params: QuiltParams, seed: u64) -> RgbaImage {
    assert!(!exemplars.is_empty(), "quilt: no exemplars");
    let patch = params.patch;
    let overlap = params.overlap;
    let step = params.step();
    assert!(patch >= 2, "quilt: patch must be >= 2");
    assert!(overlap >= 1, "quilt: overlap must be >= 1");
    assert!(
        2 * overlap <= patch,
        "quilt: overlap {overlap} too wide for patch {patch} (strips would merge)"
    );
    assert!(
        size % step == 0,
        "quilt: step {step} must divide output size {size}"
    );
    let nb = size / step;
    assert!(nb >= 2, "quilt: need >= 2 blocks per axis, got {nb}");

    let exs: Vec<Exemplar> = exemplars.iter().map(Exemplar::new).collect();

    // Candidate pool, fixed enumeration order: every exemplar
    // origin at stride 1. (A stride-2 pool was tried for large
    // outputs and measurably hurt the macro wrap seam — the
    // wrapping blocks starved for candidates that match the
    // first column/row on the wrap strips.)
    let stride = 1;
    let mut candidates: Vec<Candidate> = Vec::new();
    for (ei, ex) in exs.iter().enumerate() {
        let mut y = 0;
        while y < ex.h {
            let mut x = 0;
            while x < ex.w {
                candidates.push(Candidate { ex: ei, x, y });
                x += stride;
            }
            y += stride;
        }
    }
    assert!(!candidates.is_empty(), "quilt: empty candidate pool");

    let mut canvas = Canvas {
        size,
        patch,
        overlap,
        lab: vec![Oklab::BLACK; (size * size) as usize],
        rgba: vec![Rgba([0, 0, 0, 255]); (size * size) as usize],
        written: vec![false; (size * size) as usize],
        owner: vec![u32::MAX; (size * size) as usize],
    };
    let mut rng = Rng::new(seed);

    // Pass 1: raster order. A block's strips are the sides where
    // a neighbour already exists: left/top from earlier blocks,
    // right/bottom only for the wrapping last column/row (against
    // the first column/row across the torus seam).
    for by in 0..nb {
        for bx in 0..nb {
            canvas.place(
                &exs,
                &candidates,
                bx,
                by,
                Strips {
                    left: bx > 0,
                    right: bx == nb - 1,
                    top: by > 0,
                    bottom: by == nb - 1,
                },
                &mut rng,
            );
        }
    }

    // Pass 2 (wrap relaxation): the first column and row were
    // placed in pass 1 with the fewest constraints — block (0, 0)
    // with none at all — so the torus seam adjacencies were never
    // scored when their content was chosen. Re-place them now
    // that their wrap neighbours exist, with strips on all four
    // sides against the current canvas. This is the standard
    // finish for tileable quilting: the seam blocks are re-fit
    // with both neighbours present, so every wrap adjacency is
    // either patch-interior or protected by a cut that was scored
    // against the actual neighbour pixels.
    let all = Strips {
        left: true,
        right: true,
        top: true,
        bottom: true,
    };
    for by in 0..nb {
        canvas.place(&exs, &candidates, 0, by, all, &mut rng);
    }
    for bx in 1..nb {
        canvas.place(&exs, &candidates, bx, 0, all, &mut rng);
    }

    let mut out = RgbaImage::new(size, size);
    for (i, px) in canvas.rgba.iter().enumerate() {
        out.put_pixel((i as u32) % size, (i as u32) / size, *px);
    }
    out
}

/// Which sides of a block footprint carry an overlap strip
/// against already-placed content.
#[derive(Debug, Clone, Copy)]
struct Strips {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

/// The toroidal synthesis canvas plus the block-placement step.
struct Canvas {
    size: u32,
    patch: u32,
    overlap: u32,
    lab: Vec<Oklab>,
    rgba: Vec<Rgba<u8>>,
    written: Vec<bool>,
    /// Which block (by * nb + bx) wrote each pixel last. Pass 2
    /// scores a re-placed block only against pixels owned by
    /// OTHER blocks; without this, the incumbent candidate would
    /// trivially win by matching its own previous output and the
    /// relaxation would be a no-op.
    owner: Vec<u32>,
}

impl Canvas {
    /// Place one block at grid position `(bx, by)`: choose the
    /// candidate that best matches the current canvas under the
    /// footprint, compute min-error cuts for the strips it has,
    /// and composite. See [`quilt`] for the pass structure.
    fn place(
        &mut self,
        exs: &[Exemplar],
        candidates: &[Candidate],
        bx: u32,
        by: u32,
        strips: Strips,
        rng: &mut Rng,
    ) {
        let (size, patch, overlap) = (self.size, self.patch, self.overlap);
        let step = patch - overlap;
        let nb = size / step;
        let self_id = by * nb + bx;
        let (ox, oy) = (bx * step, by * step);
        // Overlap set: footprint pixels written by OTHER blocks
        // (toroidal coordinates). In pass 1 a block's footprint
        // only ever contains other blocks' pixels, so these are
        // exactly the strips against placed blocks; in pass 2 the
        // owner filter is what makes re-placement a real re-fit
        // instead of the incumbent matching itself.
        let mut overlap_set: Vec<(u32, u32, usize)> = Vec::new();
        for ly in 0..patch {
            for lx in 0..patch {
                let cx = (ox + lx) % size;
                let cy = (oy + ly) % size;
                let idx = (cy * size + cx) as usize;
                if self.written[idx] && self.owner[idx] != self_id {
                    overlap_set.push((lx, ly, idx));
                }
            }
        }
        let chosen = if overlap_set.is_empty() {
            candidates[rng.below(candidates.len() as u32) as usize]
        } else {
            pick_candidate(exs, candidates, &self.lab, &overlap_set, rng)
        };
        let ex = &exs[chosen.ex];

        // Boundary cuts. Ownership (the seam pixel itself stays
        // with the old content): left/top strips — new content
        // wins on the interior side of the seam (lx > s(ly) /
        // ly > s(lx)); right/bottom strips — new content wins on
        // the interior side from the other direction
        // ((lx - (patch - overlap)) < s(ly) / mirrored).
        let ov = overlap as usize;
        let p = patch as usize;
        let left_cut = strips.left.then(|| {
            let cost = strip_cost(ex, chosen, &self.lab, &self.written, size, ox, oy, patch, overlap, 0, true);
            min_error_cut(&cost, ov, p)
        });
        let right_cut = strips.right.then(|| {
            let cost = strip_cost(ex, chosen, &self.lab, &self.written, size, ox, oy, patch, overlap, patch - overlap, true);
            min_error_cut(&cost, ov, p)
        });
        let top_cut = strips.top.then(|| {
            let cost = strip_cost(ex, chosen, &self.lab, &self.written, size, ox, oy, patch, overlap, 0, false);
            min_error_cut(&cost, ov, p)
        });
        let bottom_cut = strips.bottom.then(|| {
            let cost = strip_cost(ex, chosen, &self.lab, &self.written, size, ox, oy, patch, overlap, patch - overlap, false);
            min_error_cut(&cost, ov, p)
        });

        // Composite: a pixel takes the new patch only if every
        // strip it belongs to votes new (old content wins corners
        // where two cuts disagree). A pixel not yet written always
        // takes the new patch: strips can reach into
        // not-yet-placed territory (the wrapping right strip's
        // lower rows belong to the next block row's first block),
        // and "keep the old content" is meaningless where no old
        // content exists.
        for ly in 0..patch {
            for lx in 0..patch {
                let cx = (ox + lx) % size;
                let cy = (oy + ly) % size;
                let idx = (cy * size + cx) as usize;
                let mut take_new = true;
                if let (true, Some(cut)) = (lx < overlap, &left_cut) {
                    take_new &= lx > cut[ly as usize];
                }
                if let (true, Some(cut)) = (lx >= patch - overlap, &right_cut) {
                    take_new &= (lx - (patch - overlap)) < cut[ly as usize];
                }
                if let (true, Some(cut)) = (ly < overlap, &top_cut) {
                    take_new &= ly > cut[lx as usize];
                }
                if let (true, Some(cut)) = (ly >= patch - overlap, &bottom_cut) {
                    take_new &= (ly - (patch - overlap)) < cut[lx as usize];
                }
                if !self.written[idx] {
                    take_new = true;
                }
                if take_new {
                    self.rgba[idx] = ex.rgba_at(chosen.x, chosen.y, lx, ly);
                    self.lab[idx] = ex.lab_at(chosen.x, chosen.y, lx, ly);
                    self.written[idx] = true;
                    self.owner[idx] = self_id;
                }
            }
        }
    }
}

/// Squared-error surface of one overlap strip of the chosen
/// candidate against the canvas. For a vertical strip
/// (`vertical == true`), the strip starts at local x = `start`,
/// is `overlap` wide and `patch` tall, and the returned grid is
/// row-major [row][column across strip] — the layout
/// [`min_error_cut`] consumes top-to-bottom. For a horizontal
/// strip the grid is transposed: [column][row across strip], so
/// the same DP finds the left-to-right seam.
#[allow(clippy::too_many_arguments)]
fn strip_cost(
    ex: &Exemplar,
    cand: Candidate,
    canvas_lab: &[Oklab],
    written: &[bool],
    size: u32,
    ox: u32,
    oy: u32,
    patch: u32,
    overlap: u32,
    start: u32,
    vertical: bool,
) -> Vec<f32> {
    let mut cost = Vec::with_capacity((patch * overlap) as usize);
    if vertical {
        for ly in 0..patch {
            for j in 0..overlap {
                let lx = start + j;
                let cx = (ox + lx) % size;
                let cy = (oy + ly) % size;
                let idx = (cy * size + cx) as usize;
                // Unwritten canvas (strip reaches into
                // not-yet-placed blocks) constrains nothing.
                cost.push(if written[idx] {
                    err2(ex.lab_at(cand.x, cand.y, lx, ly), canvas_lab[idx])
                } else {
                    0.0
                });
            }
        }
    } else {
        for lx in 0..patch {
            for j in 0..overlap {
                let ly = start + j;
                let cx = (ox + lx) % size;
                let cy = (oy + ly) % size;
                let idx = (cy * size + cx) as usize;
                cost.push(if written[idx] {
                    err2(ex.lab_at(cand.x, cand.y, lx, ly), canvas_lab[idx])
                } else {
                    0.0
                });
            }
        }
    }
    cost
}

/// Score all candidates against the current overlap set and pick
/// one: full-overlap scoring of the whole pool, then a seeded
/// pick among the best candidate of each exemplar whose error is
/// within a band of the global best.
///
/// The selection rule is doing real work here; two simpler rules
/// were measured and rejected on the dirt family:
///
/// * *Band around the best error* (the paper's rule): patches
///   wrap around the exemplar, so an exact self-continuation of
///   the neighbouring blocks usually exists with error exactly
///   0.0. A relative band around 0.0 admits nothing else, and
///   the synthesis collapses into tiling a single exemplar
///   (period == exemplar size, 16384/16384 pixels period-16 in
///   the dirt macro), importing the exemplar's own non-tileable
///   edges as the output seam (seam 1.53 vs the 1.15 target).
/// * *Top-K by error*: the family's flat sprites (near-uniform
///   dirt fields) beat detailed sprites on any flat canvas
///   region, flat is self-reinforcing, and the macro's detail
///   collapses (height-layer std 1.7 gray levels).
///
/// Taking each exemplar's best candidate keeps the whole family
/// in play -- the same pooling Stage C's palette extraction
/// uses -- and the band (4x the global best + a small absolute
/// floor) drops an exemplar only where its best patch genuinely
/// does not fit the current neighbourhood. Where the canvas is
/// detailed, flat exemplars fall out of the band; where it is
/// flat, detailed exemplars stay in and their flattest patches
/// seed detail back in.
fn pick_candidate(
    exs: &[Exemplar],
    candidates: &[Candidate],
    canvas_lab: &[Oklab],
    overlap_set: &[(u32, u32, usize)],
    rng: &mut Rng,
) -> Candidate {
    let mut scored: Vec<(f32, usize)> = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let ex = &exs[c.ex];
            let mut sum = 0.0f32;
            for &(lx, ly, idx) in overlap_set {
                sum += err2(ex.lab_at(c.x, c.y, lx, ly), canvas_lab[idx]);
            }
            (sum / overlap_set.len().max(1) as f32, i)
        })
        .collect();
    scored.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });
    let best_global = scored[0].0;
    // Best candidate per exemplar, in exemplar order (the sort
    // above is by (error, index), so the first occurrence of each
    // exemplar is its best, ties to the lowest candidate index).
    let mut per_exemplar: Vec<(f32, usize)> = Vec::with_capacity(exs.len());
    let mut seen = vec![false; exs.len()];
    for &(err, i) in &scored {
        let e = candidates[i].ex;
        if !seen[e] {
            seen[e] = true;
            per_exemplar.push((err, i));
        }
    }
    per_exemplar.sort_by_key(|&(_, i)| candidates[i].ex);
    let band = best_global * 4.0 + 1e-3;
    let eligible: Vec<usize> = per_exemplar
        .iter()
        .filter(|(err, _)| *err <= band)
        .map(|&(_, i)| i)
        .collect();
    debug_assert!(!eligible.is_empty(), "the global best is always within band");
    let pick = eligible[rng.below(eligible.len() as u32) as usize];
    candidates[pick]
}

/// Height layer for a macro ground texture: each pixel's OKLab
/// lightness stored explicitly as an 8-bit gray value
/// (`round(255 * L)`). This is a *stored* layer, not a luminance
/// proxy recomputed at blend time — later stages can author or
/// filter it independently of the albedo.
pub fn height_layer(img: &RgbaImage) -> GrayImage {
    let mut out = GrayImage::new(img.width(), img.height());
    for (x, y, p) in img.enumerate_pixels() {
        let l = Oklab::from_srgb_u8([p[0], p[1], p[2]]).l;
        out.put_pixel(x, y, Luma([(l.clamp(0.0, 1.0) * 255.0).round() as u8]));
    }
    out
}

/// CPU reference height blend (the semantics the Stage F engine
/// blend will mirror): composite `b` over `a` pixel-by-pixel,
/// where each side's *effective* height is its stored height
/// plus a weight bias —
///
/// ```text
/// eff_a = h_a + (1 - weight) * 2
/// eff_b = h_b + weight * 2        (heights normalized to 0..=1)
/// ```
///
/// and the higher effective height wins the pixel outright.
/// Consequences, all load-bearing for the gate: at `weight == 0`
/// the output is exactly `a` and at `weight == 1` exactly `b`
/// (the bias gap of 2 exceeds any height difference); at
/// `weight == 0.5` the biases cancel and the raw stored height
/// decides, so a high patch of `b` punches through `a` exactly
/// where its height is greater. Ties go to `a`.
pub fn height_blend(
    a: &RgbaImage,
    ha: &GrayImage,
    b: &RgbaImage,
    hb: &GrayImage,
    weight: f32,
) -> RgbaImage {
    assert_eq!(a.dimensions(), b.dimensions(), "height_blend: size mismatch");
    assert_eq!(a.dimensions(), ha.dimensions(), "height_blend: height A size mismatch");
    assert_eq!(a.dimensions(), hb.dimensions(), "height_blend: height B size mismatch");
    let w = weight.clamp(0.0, 1.0);
    let (width, height) = a.dimensions();
    let mut out = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let ea = ha.get_pixel(x, y)[0] as f32 / 255.0 + (1.0 - w) * 2.0;
            let eb = hb.get_pixel(x, y)[0] as f32 / 255.0 + w * 2.0;
            let px = if eb > ea {
                *b.get_pixel(x, y)
            } else {
                *a.get_pixel(x, y)
            };
            out.put_pixel(x, y, px);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic synthetic exemplar: two-tone checker grain
    /// with a diagonal feature, no RNG involved in building it.
    fn synthetic_exemplar() -> RgbaImage {
        let mut img = RgbaImage::new(16, 16);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let base = if (x + y) % 7 < 3 { [96, 120, 56] } else { [70, 96, 44] };
                let lift = ((x * 31 + y * 17) % 23) as u8;
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        base[0] + lift,
                        base[1] + lift,
                        base[2] + lift / 2,
                        255,
                    ]),
                );
            }
        }
        img
    }

    #[test]
    fn quilt_is_deterministic_sized_and_opaque() {
        let ex = synthetic_exemplar();
        let params = QuiltParams { patch: 16, overlap: 8 };
        let a = quilt(&[ex.clone()], 32, params, 42);
        let b = quilt(&[ex], 32, params, 42);
        assert_eq!(a.dimensions(), (32, 32));
        assert_eq!(a.as_raw(), b.as_raw(), "same inputs + seed must be byte-identical");
        assert!(a.pixels().all(|p| p[3] == 255), "output must be opaque");
        // A different seed is a different synthesis (not asserted
        // unequal pixel-for-pixel everywhere — just not the same
        // byte stream for this fixture).
        let c = quilt(&[synthetic_exemplar()], 32, params, 7);
        assert_ne!(a.as_raw(), c.as_raw());
    }

    #[test]
    fn height_layer_tracks_lightness_and_varies() {
        let mut img = RgbaImage::new(8, 8);
        for y in 0..8u32 {
            for x in 0..8u32 {
                let v = (x * 32) as u8;
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        let h = height_layer(&img);
        assert_eq!(h.dimensions(), (8, 8));
        assert_eq!(h.get_pixel(0, 0)[0], 0);
        assert!(h.get_pixel(7, 0)[0] > 200, "bright end maps high: {}", h.get_pixel(7, 0)[0]);
        let (mut mn, mut mx) = (255u8, 0u8);
        for p in h.pixels() {
            mn = mn.min(p[0]);
            mx = mx.max(p[0]);
        }
        assert!(mx - mn > 100, "height layer must have real variance");
    }

    #[test]
    fn height_blend_endpoints_and_punch_through() {
        let a = RgbaImage::from_pixel(8, 8, Rgba([200, 40, 40, 255]));
        let b = RgbaImage::from_pixel(8, 8, Rgba([40, 40, 220, 255]));
        let ha = GrayImage::from_pixel(8, 8, Luma([0]));
        let mut hb = GrayImage::from_pixel(8, 8, Luma([0]));
        for y in 2..6 {
            for x in 2..6 {
                hb.put_pixel(x, y, Luma([255]));
            }
        }
        let at0 = height_blend(&a, &ha, &b, &hb, 0.0);
        assert_eq!(at0.as_raw(), a.as_raw(), "weight 0 must be exactly A");
        let at1 = height_blend(&a, &ha, &b, &hb, 1.0);
        assert_eq!(at1.as_raw(), b.as_raw(), "weight 1 must be exactly B");
        let mid = height_blend(&a, &ha, &b, &hb, 0.5);
        assert_eq!(mid.get_pixel(4, 4), &Rgba([40, 40, 220, 255]), "high B patch punches through");
        assert_eq!(mid.get_pixel(0, 0), &Rgba([200, 40, 40, 255]), "equal heights: A holds");
    }
}
