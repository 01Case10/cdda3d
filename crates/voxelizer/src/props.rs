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
//! ## Construction profiles (side inference is per-case)
//!
//! How an unseen flank is filled is a property of how the object
//! is BUILT, not a law derivable from the sprite: a bookcase's
//! side is a solid panel (its shelves do not continue through
//! it), a table's side is its legs and apron, a crate's side
//! really is its slats continued around. One global rule got the
//! bookcase wrong (the row-field wrap carried shelf bands onto
//! its flanks), so [`solid_model`] colors sides by a
//! [`Construction`] profile, assigned per case by
//! [`construction_for`]:
//!
//! * [`Construction::Panel`] — enclosed furniture: the flank is a
//!   flat panel in the prop's primary material, built exactly
//!   like the back panel (de-shaded primary + fBm grain, the
//!   two-panel slab split included), only with the grain
//!   stretched vertically. Default for everything that segments
//!   with a closed base.
//! * [`Construction::Structural`] — open-base props (the table):
//!   the flank IS the base geometry (legs + apron), colored by
//!   the row-field rule that predates profiles. Default when
//!   [`base_plan`] reads the base as open.
//! * [`Construction::FieldWrap`] — the row-field wrap itself,
//!   kept for slatted/open construction (crates and the like).
//!   Assigned to NOTHING by default; it exists so a reviewed case
//!   can be pinned to it.
//!
//! [`CONSTRUCTION_OVERRIDES`] is the per-id pin table, consulted
//! before the default assignment — the case-by-case layer. It
//! starts empty: the five reviewed gate props are all served by
//! the defaults, and entries are added as individual cases are
//! reviewed, exactly like [`BOUNDARY_OVERRIDES`]. Whichever
//! profile a flank gets, a detected side sliver on that flank
//! still wins — it is real sprite evidence, not inference.
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
//! ## Inferred unseen faces (solid model)
//!
//! A sprite shows only the front (plus the top, and sometimes a
//! side sliver). The solid model's sides and back are therefore
//! *inferred* — educated decisions from the sprite's own materials
//! and structure, not smears or mirrors:
//!
//! * **Structure vs fringe.** A pixel is STRUCTURE (object paint)
//!   when it is opaque and not *fringe*: fringe pixels are
//!   transparent black — RGB exactly (0,0,0) at alpha < 254. They
//!   carry no color data at all (un-premultiplied edge/shadow
//!   residue in the source PNGs); the `alpha >= 128` test alone
//!   admits them, and copying one verbatim paints a pure-black
//!   cell. Measured: f_fridge's whole right edge column (x=31) is
//!   fringe, and it produced a black column down the model's right
//!   flank (front plane verbatim + sliver un-projection sampling
//!   the same column). Fringe never extends the solid model's
//!   silhouette or extent and is never a color source; in the gate
//!   sprites it occurs only at row edges, never interior (the one
//!   interior case, f_table's under-shadow, is already
//!   non-structure under the base rule). Segmentation and the
//!   deprecated models keep the plain opaque test — only the solid
//!   model applies the structure rule.
//! * **Row field.** Each front row has a FIELD color: the modal
//!   (most common) solid-paint color among the row's pixels,
//!   ignoring the outermost 2 columns of the front extent on each
//!   side, so edge trim and corner posts don't define the field.
//! * **Sides.** Colored by the prop's [`Construction`] profile
//!   (see "Construction profiles" above). Under
//!   [`Construction::Panel`] (the enclosed-furniture default) a
//!   side cell is the primary-material panel — the same
//!   construction as the back, grain stretched vertically — so
//!   the flank is one flat slab: no shelf lines, no door detail,
//!   nothing wraps. Under [`Construction::Structural`] and
//!   [`Construction::FieldWrap`] a side cell takes its row's
//!   field at FULL value, so horizontal structure wraps around
//!   the side at the same heights (shelf lines, aprons and seams
//!   continue around the flank) while vertical front detail
//!   (door splits, handles, vents) does NOT wrap: it never
//!   dominates a row's mode, so it never enters the field. In
//!   every profile a side sliver detected on that flank wins
//!   over the profile's fill (real sprite evidence). One
//!   correction for the field-based profiles: SHADOW rows are
//!   recesses, not material — f_bookcase's shelf interiors are
//!   fully-opaque pure black (24% of its front), so their field
//!   IS black and the raw rule painted black bands down the
//!   flanks and across the back. A field-based side source
//!   (field or sliver pixel) whose L < [`SHADOW_L`] is therefore
//!   replaced by the nearest non-shadow row's field (ties prefer
//!   the row above). (Under Panel the same shadow-sliver case
//!   falls back to the panel itself.)
//! * **Back.** The back is its own construction, NOT a symmetry
//!   of the front: a plain panel (or two) in the prop's primary
//!   material, where the primary material is the largest
//!   NON-SHADOW palette cluster of the front region
//!   (`analyze_image`, k=5) — the non-shadow guard matters, since
//!   a naive largest-cluster pick hands the bookcase's back to its
//!   shelf shadow. The panel carries no row banding and no front
//!   detail; its only variation is a subtle deterministic grain:
//!   the primary's OKLab L modulated by stretched fBm noise
//!   (seeded from the sprite's content hash; sampled at (x/6, y)
//!   so the grain streaks horizontally), amplitude by material —
//!   ±9% for wood tones (OKLab hue 35..=95 deg, chroma >= 0.03),
//!   ±3% for flat tones (chroma < 0.03: metal, plastic), ±5%
//!   otherwise. TWO panels when the top region is a distinct slab
//!   component rather than the same box's top face: the top and
//!   front region means differ by >= 0.15 OKLab AND the top region
//!   is at least a third of the silhouette height (a deep slab —
//!   countertop, tabletop — vs a shallow cap like a fridge's,
//!   which is the same material however it is painted). The upper
//!   panel (top region's primary) covers the back rows whose front
//!   field belongs to the top material (field closer to the top
//!   primary than to the front primary, scanned from the boundary
//!   down); if no front row matches, the split falls back to the
//!   top region's proportional share of the silhouette. Limits of
//!   the judgment: the palette cluster is a centroid, so a front
//!   whose largest non-shadow cluster is a trim color would
//!   mislead it; and a two-material front split VERTICALLY (not by
//!   rows) is read as its dominant material only.
//! * **Albedo only — de-shading.** The model carries TEXTURE
//!   (albedo); the engine's renderers shade faces by orientation,
//!   so any light baked into the model would double-shade. The
//!   solid model therefore applies NO directional multiplier
//!   anywhere (an earlier iteration darkened sides x0.85 and the
//!   interior x0.7 — both removed): side cells are the row field
//!   at full value, interior cells are the front pixel verbatim,
//!   the back panel is the primary at full value (its fBm grain
//!   is texture variation, not light). The front plane and the
//!   top face stay verbatim sprite — they ARE the texture.
//!   Derived colors additionally have the sprite's own baked
//!   vertical light gradient removed: UltiCa fronts are painted
//!   with a top-to-bottom L falloff (Stage B measures it per
//!   family as a shading factor), so a field color taken from a
//!   low row would otherwise carry that row's shade onto the
//!   flank. The correction fits the front region's mean-L
//!   profile per row (structure pixels, detected sliver columns
//!   excluded — a differently-lit face would bias the fit —
//!   smoothed by a 5-row moving average) and scales each derived
//!   color's OKLab L by gain(row) = (median row L) / (row L),
//!   clamped to [0.80, 1.25], a/b preserved. This is the same
//!   principle as [`deshade_palette`](crate::generate::deshade_palette)
//!   — remove baked L variation toward a median-L anchor,
//!   keeping hue/chroma — applied per row because prop inference
//!   derives colors per row, where a palette-level compression
//!   has nothing to act on. The back panel's primary (a palette
//!   cluster spanning all rows) is re-anchored to the palette's
//!   own weighted median L over the non-shadow clusters —
//!   deshade_palette's anchor at full strength: a gradient
//!   front's palette splits into near-equal L bands, so the
//!   largest cluster alone is a band edge, not the material's
//!   center, and no row-profile gain can fix a wrong anchor.
//!   Same clamp. A sliver pixel carries
//!   a second baked component — its face is drawn darker than
//!   the front at the same row — so after the row gain it is
//!   re-anchored: its L is scaled by (de-shaded field L at its
//!   row) / (de-shaded sliver-band mean L at its row), pinning
//!   the band's average to the material while preserving the
//!   sliver's own pattern (vents, seams). Limit: the re-anchor
//!   assumes the sliver is the SAME material as the front field
//!   at that row — the 3/4 convention for these sprites; a side
//!   genuinely painted in a darker albedo would be lightened
//!   toward the front's material.
//! * **Top** stays evidence-based: the top region un-projected
//!   (see [`solid_model`]).
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
//!
//! ## View classification (Stage G.3 rework)
//!
//! The pipeline above assumes every sprite is a FRONT view (top
//! band / front face / depth). That assumption is false in
//! general: some UltiCa furniture sprites are TOP-DOWN (the sprite
//! IS the object's top face — f_bed, f_slab) and some are END-ON
//! (the sprite is the object's end face — f_standing_tank is the
//! end cap of a horizontal leg tank). Forcing those through the
//! front-view reading invents geometry the sprite never depicted
//! (a bunk bed from a single-bed plan, a pedestal under a flat
//! slab). The rework therefore classifies the sprite's view
//! BEFORE segmenting ([`view_for`]): per-id pins in
//! [`VIEW_OVERRIDES`] are the ground truth; a conservative
//! heuristic proposes (a sprite that does not segment at all, or
//! whose "top region" covers >= 85% of its silhouette, reads as
//! top-down). [`model_for`] is the routed entry point: Front keeps
//! the signed-off pipeline (archetype if one is assigned, else
//! the solid model); TopDown and EndOn go to dedicated faithful
//! builders ([`topdown_model`], [`endon_model`]). Per pipeline
//! §24.4 the new builders fire initially EXACTLY for the pinned
//! ids — a heuristic proposal alone does not reroute an unpinned
//! id away from the Front path in `model_for`.
//!
//! * **Terrace** (f_slab): the plan's concentric rings are a
//!   heightmap — a cell's height is its chamfer distance to the
//!   silhouette edge (ring index), capped and scaled so the
//!   maximum height is ~1/4 of the sprite width. The top face is
//!   the sprite verbatim; step sides are the sprite's edge
//!   material (the modal boundary-ring color).
//! * **Framed surface** (the bed chain): the plan's border is a
//!   wooden frame (walls extruded down from the top face), the
//!   interior is the mattress (top face verbatim, a few cells
//!   thick, sitting one cell proud of the frame rails), and the
//!   head end's border strip rises as a PLAIN SLAB headboard in
//!   the frame material, extending to the ground — from behind,
//!   the slab occludes the mattress completely (Joey's ruling on
//!   the approved bed mockup). The head end is detected (widest /
//!   darkest end strip) but pinned per id in
//!   [`HEAD_END_OVERRIDES`]: on f_bed the detector alone picks
//!   the foot (its band + posts are wider), while the depicted
//!   headboard is the plain band at the sprite's top rows.
//!   Round 2 (Joey's corrections on the built model): the foot
//!   end carries a real FOOTBOARD — a plain frame panel spanning
//!   the full width, ground to the mattress top — and the long
//!   side walls are built from the silhouette PERIMETER, not
//!   from color classification: every perimeter cell gets a
//!   frame wall from the ground to the mattress underside
//!   whatever its top color reads as, because the sprite's edge
//!   cells alternate between frame shades and mattress tones
//!   and a color-classified wall gaps wherever the edge reads
//!   as mattress.
//! * **EndOn** (the tank): the sprite's round face is the end
//!   cap; the model is a horizontal cylinder (disc cross-section,
//!   length = cap diameter unless pinned in
//!   [`TANK_LENGTH_OVERRIDES`]) whose FRONT plane is the sprite's
//!   cap region mapped onto the disc verbatim. The fitting
//!   cluster the sprite draws in the band above the cap (the
//!   brass/olive hatch fitting) becomes a short neck block on
//!   top of the barrel at the front end, topped with the fitting
//!   color; two stands under the barrel (front and back) carry
//!   the sprite's base material.

use crate::analyze::{analyze_image, content_hash};
use crate::color::Oklab;
use crate::noise::Fbm;
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
/// A field/sliver color below this OKLab L is shadow (a recess or
/// painted shade), not a material color; unseen-face inference
/// never uses it as one (module docs, "Inferred unseen faces").
pub(crate) const SHADOW_L: f32 = 0.15;
/// De-shading gain clamp (module docs, "Albedo only — de-shading"):
/// a derived color's OKLab L is scaled by its row's gain, never by
/// more than this either way — the correction removes a smooth
/// baked gradient, it does not re-light the sprite.
const DESHADE_GAIN_MIN: f32 = 0.80;
const DESHADE_GAIN_MAX: f32 = 1.25;
/// Radius (in rows) of the moving-average smoothing applied to a
/// region's mean-L profile before de-shading gains are built
/// from it.
const PROFILE_SMOOTH_RADIUS: usize = 2;
/// Floor-standing props (silhouette at least this many rows) get
/// a minimum model depth — see [`Segmentation::depth_cells_for`].
const MIN_DEPTH_SILHOUETTE_ROWS: u32 = 24;
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

/// How a prop's unseen side surfaces are constructed (module
/// docs, "Construction profiles"). Side inference is per-case:
/// the profile says what a flank IS, and the color rule follows
/// from that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Construction {
    /// Enclosed furniture: the flank is a flat panel in the
    /// prop's primary material — the back panel's construction,
    /// grain stretched vertically. Nothing from the front wraps.
    Panel,
    /// Open-base props: the flank is the base structure itself
    /// (legs + apron), colored by the row-field rule.
    Structural,
    /// Slatted/open construction: the row-field wrap — horizontal
    /// front structure continues around the flank. Assigned to
    /// nothing by default; pin it per id in
    /// [`CONSTRUCTION_OVERRIDES`] when a reviewed case calls for
    /// it.
    FieldWrap,
}

/// Per-id construction pins, consulted by [`construction_for`]
/// before the default assignment. The case-by-case layer: an
/// entry is added when a specific prop is reviewed and the
/// default profile is wrong for how that object is built.
/// Starts empty — the reviewed gate props (counter, bookcase,
/// table, fridge, locker) are all served by the defaults.
pub const CONSTRUCTION_OVERRIDES: &[(&str, Construction)] = &[];

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
    /// Depth for the solid model: [`Self::depth_cells`] with a
    /// minimum-depth floor for floor-standing props. The raw rule
    /// reads depth off the drawn top region, so a tall piece whose
    /// cap is drawn thin (f_locker: a 2-row cap) comes out
    /// paper-thin (4 sub-cells) although the real object is a deep
    /// cabinet. When the silhouette is at least
    /// [`MIN_DEPTH_SILHOUETTE_ROWS`] rows tall — a floor-standing
    /// piece, not a countertop item — the depth is at least a
    /// third of the sprite width, the educated proportion for
    /// lockers/wardrobes (real units run ~1/3 as deep as wide).
    /// Measured on the gate set: only f_locker changes (4 -> 11);
    /// counter 32, bookcase 14, table 32, fridge 16 already clear
    /// the floor. The deprecated box/column models keep the raw
    /// rule.
    pub fn depth_cells_for(&self, sprite_width: u32) -> u32 {
        let d = self.depth_cells();
        let silh = self.silhouette_rows.end - self.silhouette_rows.start;
        if silh >= MIN_DEPTH_SILHOUETTE_ROWS {
            let floor = (sprite_width + 2) / 3; // ceil(width / 3)
            d.max(floor)
        } else {
            d
        }
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

pub(crate) fn lab_of(px: Rgba<u8>) -> Oklab {
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
pub(crate) fn top_row_for_z(seg: &Segmentation, d: u32, z: u32) -> u32 {
    let th = seg.top_rows.end - seg.top_rows.start;
    let back_steps = d - 1 - z;
    let row_back = (back_steps * th / d).min(th - 1);
    seg.top_rows.end - 1 - row_back
}

/// Solid paint: fully opaque (see [`SOLID_ALPHA`]).
fn solid(px: Rgba<u8>) -> bool {
    px[3] >= SOLID_ALPHA
}

/// Fringe: transparent black — RGB exactly (0,0,0) below
/// [`SOLID_ALPHA`]. Such pixels carry no color data (edge/shadow
/// residue in the source PNGs); treating one as paint produces a
/// pure-black cell (module docs, "Inferred unseen faces").
fn is_fringe(px: Rgba<u8>) -> bool {
    px[3] < SOLID_ALPHA && px[0] == 0 && px[1] == 0 && px[2] == 0
}

/// Structure: a pixel that is real object paint — opaque and not
/// fringe. The solid model's silhouette/extent test.
pub(crate) fn structure(px: Rgba<u8>) -> bool {
    opaque(px) && !is_fringe(px)
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
            if structure(px) {
                return Some(shade(px, 0.85));
            }
        }
    }
    let px = *sprite.get_pixel(edge_x, r);
    if structure(px) {
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

/// Columns of the front extent ignored on each side when computing
/// a row's field color (edge trim / corner posts shouldn't define
/// the field).
const FIELD_TRIM: u32 = 2;

/// The FIELD color of one front row (module docs, "Inferred unseen
/// faces"): the modal (most common) solid-paint color among the
/// row's pixels in the front extent, ignoring the outermost
/// [`FIELD_TRIM`] columns on each side. Fallbacks, in order: solid
/// paint over the untrimmed extent, then merely-opaque paint over
/// the untrimmed extent (a row of pure painted shadow). `None` only
/// when the row carries no usable pixel at all. Ties break toward
/// the smallest RGBA key (BTreeMap order + strict >), so the result
/// is deterministic.
fn row_field(sprite: &RgbaImage, r: u32, x0: u32, x1: u32) -> Option<Rgba<u8>> {
    let modal = |lo: u32, hi: u32, min_alpha: u8| -> Option<Rgba<u8>> {
        let mut counts: std::collections::BTreeMap<[u8; 4], u32> =
            std::collections::BTreeMap::new();
        for x in lo..=hi {
            let px = *sprite.get_pixel(x, r);
            if px[3] >= min_alpha {
                *counts.entry(px.0).or_insert(0) += 1;
            }
        }
        let mut best: Option<([u8; 4], u32)> = None;
        for (k, n) in counts {
            if best.map_or(true, |(_, bn)| n > bn) {
                best = Some((k, n));
            }
        }
        best.map(|(k, _)| Rgba(k))
    };
    let (tx0, tx1) = if x1 >= x0 + 2 * FIELD_TRIM {
        (x0 + FIELD_TRIM, x1 - FIELD_TRIM)
    } else {
        (x0, x1)
    };
    modal(tx0, tx1, SOLID_ALPHA)
        .or_else(|| modal(x0, x1, SOLID_ALPHA))
        .or_else(|| modal(x0, x1, ALPHA_OPAQUE).filter(|c| !is_fringe(*c)))
}

/// Scale a color's OKLab L by `gain`, a/b preserved — the
/// de-shading primitive (module docs, "Albedo only — de-shading").
/// Alpha forced opaque, like the rest of the inference path.
fn deshade(px: Rgba<u8>, gain: f32) -> Rgba<u8> {
    let lab = deshade_lab(lab_of(px), gain);
    let [r, g, b] = lab.to_srgb_u8();
    Rgba([r, g, b, 255])
}

/// [`deshade`] on an OKLab color.
pub(crate) fn deshade_lab(lab: Oklab, gain: f32) -> Oklab {
    Oklab {
        l: (lab.l * gain).clamp(0.0, 1.0),
        a: lab.a,
        b: lab.b,
    }
}

/// A region's mean-L profile (module docs, "Albedo only —
/// de-shading"): per-row mean OKLab L over STRUCTURE pixels
/// within the region's trimmed structure extent ([`FIELD_TRIM`]
/// columns, as for [`row_field`]), optionally excluding a column
/// range (the detected sliver's columns — a differently-lit
/// face would bias the front's fit). Rows with no usable pixel
/// are filled from the nearest usable row before the profile is
/// smoothed by a moving average of radius
/// [`PROFILE_SMOOTH_RADIUS`].
struct LProfile {
    /// Smoothed per-row mean L, in region row order.
    smoothed: Vec<f32>,
    /// Median of the smoothed values over rows that had data.
    median: f32,
}

fn l_profile(
    sprite: &RgbaImage,
    rows: &Range<u32>,
    exclude: Option<&Range<u32>>,
) -> Option<LProfile> {
    // Structure extent of the region.
    let (mut x0, mut x1) = (sprite.width() - 1, 0u32);
    let mut any = false;
    for r in rows.clone() {
        for x in 0..sprite.width() {
            if structure(*sprite.get_pixel(x, r)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    let (tx0, tx1) = if x1 >= x0 + 2 * FIELD_TRIM {
        (x0 + FIELD_TRIM, x1 - FIELD_TRIM)
    } else {
        (x0, x1)
    };
    let n_rows = (rows.end - rows.start) as usize;
    let mut sum = vec![0.0f32; n_rows];
    let mut cnt = vec![0u32; n_rows];
    for (i, r) in rows.clone().enumerate() {
        for x in tx0..=tx1 {
            if exclude.map_or(false, |e| e.contains(&x)) {
                continue;
            }
            let px = *sprite.get_pixel(x, r);
            if structure(px) {
                sum[i] += lab_of(px).l;
                cnt[i] += 1;
            }
        }
    }
    let total: u32 = cnt.iter().sum();
    if total == 0 {
        return None;
    }
    let mean = sum.iter().sum::<f32>() / total as f32;
    let raw: Vec<Option<f32>> = (0..n_rows)
        .map(|i| {
            if cnt[i] > 0 {
                Some(sum[i] / cnt[i] as f32)
            } else {
                None
            }
        })
        .collect();
    // Fill empty rows from the nearest usable row (above wins ties,
    // matching the side-field borrowing convention).
    let filled: Vec<f32> = (0..n_rows)
        .map(|i| {
            raw[i]
                .or_else(|| {
                    (1..n_rows).find_map(|dist| {
                        if i + dist < n_rows && raw[i + dist].is_some() {
                            raw[i + dist]
                        } else if dist <= i && raw[i - dist].is_some() {
                            raw[i - dist]
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or(mean)
        })
        .collect();
    let rad = PROFILE_SMOOTH_RADIUS;
    let smoothed: Vec<f32> = (0..n_rows)
        .map(|i| {
            let lo = i.saturating_sub(rad);
            let hi = (i + rad).min(n_rows - 1);
            filled[lo..=hi].iter().sum::<f32>() / (hi - lo + 1) as f32
        })
        .collect();
    let mut vals: Vec<f32> = (0..n_rows)
        .filter(|&i| raw[i].is_some())
        .map(|i| smoothed[i])
        .collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if vals.len() % 2 == 1 {
        vals[vals.len() / 2]
    } else {
        0.5 * (vals[vals.len() / 2 - 1] + vals[vals.len() / 2])
    };
    Some(LProfile { smoothed, median })
}

/// Per-row de-shading gains for a region, in region row order:
/// gain(row) = median row L / smoothed row L, clamped to
/// [DESHADE_GAIN_MIN, DESHADE_GAIN_MAX]. All 1.0 when the region
/// has no usable profile.
fn row_gains(sprite: &RgbaImage, rows: &Range<u32>, exclude: Option<&Range<u32>>) -> Vec<f32> {
    let n = (rows.end - rows.start) as usize;
    match l_profile(sprite, rows, exclude) {
        Some(p) => (0..n)
            .map(|i| {
                if p.smoothed[i] > 0.0 {
                    (p.median / p.smoothed[i]).clamp(DESHADE_GAIN_MIN, DESHADE_GAIN_MAX)
                } else {
                    1.0
                }
            })
            .collect(),
        None => vec![1.0; n],
    }
}

/// The sliver pixel a solid-model side cell at depth `z` samples:
/// the same un-projection mapping as [`side_color`], but RAW —
/// no darkening (the solid model is albedo only; the caller
/// de-shades). None when the sampled pixel is not structure.
fn sliver_sample(
    sprite: &RgbaImage,
    seg: &Segmentation,
    r: u32,
    z: u32,
    d: u32,
    on_right: bool,
) -> Option<Rgba<u8>> {
    let sl = seg.side.as_ref()?;
    if sl.on_right != on_right {
        return None;
    }
    let sw = sl.cols.end - sl.cols.start;
    // Front of the model (z = d-1) meets the sliver column nearest
    // the front face; walk back across the sliver.
    let back_steps = d - 1 - z;
    let off = (back_steps * sw / d).min(sw - 1);
    let sx = if on_right {
        sl.cols.end - 1 - off
    } else {
        sl.cols.start + off
    };
    let px = *sprite.get_pixel(sx, r);
    if structure(px) {
        Some(px)
    } else {
        None
    }
}

/// Mean L of a sliver band's structure pixels in one row — the
/// band's own lightness, for the de-shading re-anchor (module
/// docs, "Albedo only — de-shading").
fn sliver_band_mean_l(
    sprite: &RgbaImage,
    seg: &Segmentation,
    r: u32,
    on_right: bool,
) -> Option<f32> {
    let sl = seg.side.as_ref()?;
    if sl.on_right != on_right {
        return None;
    }
    let (mut s, mut n) = (0.0f32, 0u32);
    for x in sl.cols.clone() {
        let px = *sprite.get_pixel(x, r);
        if structure(px) {
            s += lab_of(px).l;
            n += 1;
        }
    }
    if n > 0 {
        Some(s / n as f32)
    } else {
        None
    }
}

/// Weight-weighted median of cluster lightness (the same rule
/// as `generate::weighted_median_l`): sort by L, accumulate
/// weights, return the L where the cumulative weight reaches
/// half the total.
pub(crate) fn weighted_median_l(clusters: &[(f32, f32)]) -> f32 {
    let mut ls: Vec<(f32, f32)> = clusters.to_vec();
    ls.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let total: f32 = ls.iter().map(|x| x.1).sum();
    let mut acc = 0.0f32;
    for (l, w) in &ls {
        acc += w;
        if acc >= total * 0.5 {
            return *l;
        }
    }
    ls.last().map(|x| x.0).unwrap_or(0.0)
}

/// A region's primaries for the back panel: the RAW primary
/// (material identity — the two-panel scan compares fields
/// against it in the sprite's own color space) and the PANEL
/// primary (the color the back is actually painted with): the
/// raw primary DE-SHADED by re-anchoring its L to the palette's
/// weighted median L — [`deshade_palette`](crate::generate::deshade_palette)'s
/// anchor, applied at full strength to this one color. The
/// re-anchor matters because a gradient front's palette splits
/// into near-equal L bands and the largest cluster is then a
/// band edge, not the material's center; the weighted median
/// over the non-shadow clusters is the material's reference
/// lightness. Gain clamped to [DESHADE_GAIN_MIN, DESHADE_GAIN_MAX]
/// like the row gains.
pub(crate) fn region_primaries(sprite: &RgbaImage, rows: &Range<u32>) -> (Oklab, Oklab) {
    let crop = image::imageops::crop_imm(
        sprite,
        0,
        rows.start,
        sprite.width(),
        rows.end - rows.start,
    )
    .to_image();
    let a = analyze_image(&crop, 5);
    let raw = a
        .palette
        .iter()
        .find(|p| p.lab.l >= SHADOW_L)
        .or_else(|| a.palette.first())
        .map(|p| p.lab)
        .unwrap_or_else(|| lab_of(region_mean(sprite, rows)));
    let non_shadow: Vec<(f32, f32)> = a
        .palette
        .iter()
        .filter(|p| p.lab.l >= SHADOW_L)
        .map(|p| (p.lab.l, p.weight))
        .collect();
    let median = if non_shadow.is_empty() {
        raw.l
    } else {
        weighted_median_l(&non_shadow)
    };
    let panel = if raw.l > 0.0 {
        deshade_lab(raw, (median / raw.l).clamp(DESHADE_GAIN_MIN, DESHADE_GAIN_MAX))
    } else {
        raw
    };
    (raw, panel)
}

/// Grain amplitude (fraction of L) for a back panel of the given
/// material: wood tones (OKLab hue 35..=95 deg with real chroma)
/// get visible grain, flat tones (metal/plastic, chroma < 0.03)
/// stay near-flat, everything else is in between.
pub(crate) fn grain_amp(lab: Oklab) -> f32 {
    let chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();
    if chroma < 0.03 {
        return 0.03;
    }
    let hue = lab.b.atan2(lab.a).to_degrees();
    if (35.0..=95.0).contains(&hue) {
        0.09
    } else {
        0.05
    }
}

/// The solid model's plain panels: the back — and, under
/// [`Construction::Panel`], the side flanks — in the front
/// region's primary material: one panel, or two when the top
/// region is a distinct slab component (module docs, "Inferred
/// unseen faces"). Panels are colored per cell by
/// [`BackPanels::color`] (back) / [`BackPanels::side_color`]
/// (flank): the panel's primary with its L grain-modulated by
/// stretched fBm — horizontally on the back, vertically on a
/// flank.
struct BackPanels {
    fbm: Fbm,
    front: Oklab,
    /// (top primary, number of back rows it covers from the top).
    upper: Option<(Oklab, u32)>,
}

impl BackPanels {
    /// The panel material covering model row `y` (the upper
    /// panel, when the two-panel split fired, covers the top
    /// `count` rows).
    fn base(&self, y: u32, fh: u32) -> Oklab {
        match self.upper {
            Some((top, count)) if y >= fh - count => top,
            _ => self.front,
        }
    }

    /// A panel cell's color: the base primary with its L
    /// grain-modulated by the fBm sampled at (u, v).
    fn grain_color(&self, base: Oklab, u: f32, v: f32) -> Rgba<u8> {
        let n = self.fbm.sample(u, v);
        let t = ((n - 0.5) * 2.0).clamp(-1.0, 1.0);
        let mut lab = base;
        lab.l = (lab.l * (1.0 + grain_amp(base) * t)).clamp(0.02, 0.98);
        let [r, g, b] = lab.to_srgb_u8();
        Rgba([r, g, b, 255])
    }

    /// Back-plane color at (x, y): grain sampled at (x/6, y), so
    /// the streaks run horizontally.
    fn color(&self, x: u32, y: u32, fh: u32) -> Rgba<u8> {
        self.grain_color(self.base(y, fh), x as f32 / 6.0, y as f32)
    }

    /// Side-flank color at depth cell `z`, model row `y`
    /// ([`Construction::Panel`] flanks): the same panels as the
    /// back, but the grain is stretched VERTICALLY — sampled at
    /// (z, y/6), since a flank's horizontal axis is depth, so the
    /// streaks run along y.
    fn side_color(&self, z: u32, y: u32, fh: u32) -> Rgba<u8> {
        self.grain_color(self.base(y, fh), z as f32, y as f32 / 6.0)
    }
}

fn back_panels(
    sprite: &RgbaImage,
    seg: &Segmentation,
    fields: &[Option<Rgba<u8>>],
    fh: u32,
) -> BackPanels {
    // Raw primaries drive the two-panel scan below (a material-
    // identity test, run in the sprite's own color space); the
    // panels themselves are colored with the DE-SHADED panel
    // primaries (module docs, "Albedo only").
    let (front, front_panel) = region_primaries(sprite, &seg.front_rows);
    let fbm = Fbm::new(content_hash(sprite.as_raw()), 16).octaves(2);
    let top_h = seg.top_rows.end - seg.top_rows.start;
    let silh = seg.silhouette_rows.end - seg.silhouette_rows.start;
    // Two-panel test: distinct material AND a deep slab share of
    // the silhouette (a shallow cap is the same box's top face,
    // however it is painted — the fridge's teal cap over a gray
    // body must not split the back).
    let region_mean_lab = |rows: &Range<u32>| -> Oklab {
        let crop = image::imageops::crop_imm(
            sprite,
            0,
            rows.start,
            sprite.width(),
            rows.end - rows.start,
        )
        .to_image();
        analyze_image(&crop, 1).mean_color
    };
    let two_panel = region_mean_lab(&seg.top_rows).distance(region_mean_lab(&seg.front_rows))
        >= 0.15
        && top_h * 3 >= silh;
    let upper = if two_panel {
        let (top_primary, top_panel) = region_primaries(sprite, &seg.top_rows);
        // Upper rows: front fields that belong to the top material,
        // scanned from the boundary down.
        let mut count = 0u32;
        for i in 0..fh {
            let f = fields[(fh - 1 - i) as usize];
            let belongs = f.map_or(false, |c| {
                let l = lab_of(c);
                l.distance(top_primary) < l.distance(front)
            });
            if belongs {
                count += 1;
            } else {
                break;
            }
        }
        if count == 0 || count >= fh {
            // No front row reads as top material (or all do):
            // fall back to the top's proportional silhouette share.
            count = ((fh as f32 * top_h as f32 / silh as f32).round() as u32).clamp(1, fh - 1);
        }
        Some((top_panel, count))
    } else {
        None
    };
    BackPanels {
        fbm,
        front: front_panel,
        upper,
    }
}

/// The front region's STRUCTURE x-extent: fringe pixels never
/// extend it (module docs, "Inferred unseen faces").
pub(crate) fn front_structure_extent(sprite: &RgbaImage, seg: &Segmentation) -> (u32, u32) {
    let (mut x0, mut x1) = (sprite.width() - 1, 0u32);
    for y in seg.front_rows.clone() {
        for x in 0..sprite.width() {
            if structure(*sprite.get_pixel(x, y)) {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
        }
    }
    (x0, x1)
}

/// The construction profile for a prop (module docs,
/// "Construction profiles"): [`CONSTRUCTION_OVERRIDES`] first
/// (the per-case layer), then the default assignment — open-base
/// props are [`Construction::Structural`] (their flanks are their
/// legs and apron), everything else that segments is enclosed
/// furniture and gets [`Construction::Panel`]. The defaults are a
/// starting assignment, not a law: a case that builds differently
/// gets pinned in the table.
pub fn construction_for(
    id: Option<&str>,
    sprite: &RgbaImage,
    seg: &Segmentation,
) -> Construction {
    if let Some(id) = id {
        if let Some((_, c)) = CONSTRUCTION_OVERRIDES.iter().find(|(k, _)| *k == id) {
            return *c;
        }
    }
    let (x0, x1) = front_structure_extent(sprite, seg);
    if base_plan(sprite, seg, x0, x1).open {
        Construction::Structural
    } else {
        Construction::Panel
    }
}

/// THE prop model: the front silhouette extruded through the full
/// model depth and filled at *every* depth cell — a real solid, so
/// no viewing angle can see into a hollow interior or a side gap
/// (the failure that retired the G.2 shell).
///
/// * Cross-section: voxel (x, y, z) is solid iff the front-region
///   pixel at (x, front_bottom - y) is STRUCTURE (opaque, not
///   transparent-black fringe — see [`structure`]) and z < d, with
///   d = [`Segmentation::depth_cells_for`] uniform for every
///   silhouette column. (Depth does not vary per column: the top
///   region sets the one depth, exactly as in the earlier models.)
/// * Front plane (z = d-1): the sprite pixel verbatim — the front
///   projection is the front silhouette exactly, colors included.
///   Nothing below overwrites the front plane.
/// * Back plane (z = 0): a plain panel (or two) in the prop's
///   primary material with a subtle noise grain — the rear
///   construction inferred as its own object (module docs,
///   "Inferred unseen faces"), NOT a darkened mirror of the front
///   and NOT the row fields. Interior cells: the front pixel
///   verbatim (never visible in a surface render, but correct if
///   the model is sliced). NO directional multipliers anywhere —
///   the model is albedo only; the engine provides the lighting
///   (module docs, "Albedo only — de-shading").
/// * Side surfaces: each row's leftmost/rightmost solid columns are
///   real surface (the model is filled behind them), colored on the
///   interior depth cells by the prop's [`Construction`] profile
///   (module docs, "Construction profiles"): the detected sliver
///   un-projected where one exists on that side ([`sliver_sample`]
///   — real sprite evidence wins in every profile); otherwise
///   [`Construction::Panel`] flanks take the primary-material
///   panel color (the back's construction, grain stretched
///   vertically), while [`Construction::Structural`] /
///   [`Construction::FieldWrap`] flanks take the row's field color
///   at full value, so horizontal bands wrap at their heights and
///   vertical front detail does not. All inferred colors are
///   DE-SHADED (per-row gain; sliver pixels additionally
///   re-anchored to the row's field — module docs). Shadow-colored
///   sources (field or sliver, L < [`SHADOW_L`]) are not material:
///   field-based profiles replace them with the nearest
///   non-shadow row's field, Panel flanks with the panel itself.
///   The front/back planes keep their own colors, so side
///   inference never touches the exact front projection.
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
/// [`solid_model`] with the prop's id, so per-id construction
/// pins ([`CONSTRUCTION_OVERRIDES`]) apply. Prefer this entry
/// point whenever the caller knows the id.
pub fn solid_model_for(id: &str, sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let construction = construction_for(Some(id), sprite, seg);
    solid_model_impl(sprite, seg, construction, false, None)
}

/// The solid model with the default construction assignment
/// ([`construction_for`] with no id — no per-id pins).
pub fn solid_model(sprite: &RgbaImage, seg: &Segmentation) -> PropModel {
    let construction = construction_for(None, sprite, seg);
    solid_model_impl(sprite, seg, construction, false, None)
}

fn solid_model_impl(
    sprite: &RgbaImage,
    seg: &Segmentation,
    construction: Construction,
    force_closed: bool,
    fitted_top: Option<Rgba<u8>>,
) -> PropModel {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let d = seg.depth_cells_for(w);
    let bot = front.end - 1;
    // Front region's STRUCTURE x-extent (base_plan + row_pixel
    // ranges): fringe pixels never extend it (module docs).
    let (x0, x1) = front_structure_extent(sprite, seg);
    let plan = base_plan(sprite, seg, x0, x1);
    // Coverage-drive seamless path only: a pedestal/cantilever
    // body whose open-base reading would drop real silhouette
    // mass is built closed instead (see coverage_model). The
    // segmenting callers pass false and are unchanged.
    let plan = if force_closed {
        BasePlan {
            open: false,
            header_rows: plan.header_rows,
            grounded: plan.grounded,
        }
    } else {
        plan
    };
    let slab_bottom = fh - plan.header_rows; // model y; only if plan.open
    // The solidity mask: the whole model is this mask extruded
    // through every depth cell.
    let is_solid = |x: u32, y: u32| -> bool {
        if !structure(*sprite.get_pixel(x, bot - y)) {
            return false;
        }
        if !plan.open || y >= slab_bottom {
            return true;
        }
        // Below the slab of an open base: grounded columns only.
        // (x is within the extent: its front pixel is opaque.)
        plan.grounded[(x - x0) as usize]
    };
    // Per-row field colors for the inferred side faces, indexed
    // by model y.
    let fields: Vec<Option<Rgba<u8>>> = (0..fh)
        .map(|y| row_field(sprite, bot - y, x0, x1))
        .collect();
    // De-shading gains per front row, indexed by model y (module
    // docs, "Albedo only — de-shading"). The detected sliver's
    // columns are excluded from the profile fit.
    let gains = row_gains(sprite, front, seg.side.as_ref().map(|sl| &sl.cols));
    let gain_at = |y: u32| -> f32 { gains[(fh - 1 - y) as usize] };
    // Side colors with SHADOW BORROWING + DE-SHADING: a shadow
    // field (a recess row — f_bookcase's shelf interiors are pure
    // black) borrows the nearest non-shadow row's field; ties
    // prefer the row above (larger model y = smaller sprite row).
    // Every field is de-shaded by its OWN row's gain — a borrowed
    // field carries its source row's correction with it. A front
    // that is shadow throughout keeps its own (de-shaded) fields.
    let side_colors: Vec<Option<Rgba<u8>>> = (0..fh)
        .map(|y| {
            let own = fields[y as usize];
            if own.map_or(false, |f| lab_of(f).l >= SHADOW_L) {
                return own.map(|f| deshade(f, gain_at(y)));
            }
            for dist in 1..fh {
                if y + dist < fh {
                    if let Some(f) = fields[(y + dist) as usize] {
                        if lab_of(f).l >= SHADOW_L {
                            return Some(deshade(f, gain_at(y + dist)));
                        }
                    }
                }
                if dist <= y {
                    if let Some(f) = fields[(y - dist) as usize] {
                        if lab_of(f).l >= SHADOW_L {
                            return Some(deshade(f, gain_at(y - dist)));
                        }
                    }
                }
            }
            own.map(|f| deshade(f, gain_at(y)))
        })
        .collect();
    // The back: plain panel(s) in the primary material(s).
    let back = back_panels(sprite, seg, &fields, fh);
    let mut m = PropModel::new([w, fh, d]);
    // Base fill: front verbatim, back = the panel color, interior
    // = the front pixel verbatim. No directional multipliers —
    // albedo only (module docs).
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
                    back.color(x, y, fh)
                } else {
                    px
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Side surfaces: each row's silhouette extremes, interior depth
    // cells only (front/back planes keep their colors above). A
    // detected sliver on that side is real evidence and wins in
    // every profile; otherwise the construction profile fills the
    // flank: Panel takes the primary-material panel color (the
    // back's construction), Structural/FieldWrap take the row's
    // de-shaded field color at full value — horizontal bands wrap,
    // vertical detail doesn't (module docs).
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
            let sliver_here = seg
                .side
                .as_ref()
                .map_or(false, |sl| sl.on_right == on_right);
            // De-shading scale for sliver evidence (module docs):
            // the total L scale that lands the sliver band's
            // average on this row's de-shaded field color is
            // (de-shaded field L) / (raw band mean L) — the row
            // gain is already inside the field's de-shading, so
            // it is NOT applied again on top. Without a field to
            // anchor to, the row gain alone applies.
            let anchor = if sliver_here {
                match (
                    side_colors[y as usize],
                    sliver_band_mean_l(sprite, seg, r, on_right),
                ) {
                    (Some(f), Some(band)) if band > 0.0 => Some(lab_of(f).l / band),
                    _ => None,
                }
            } else {
                None
            };
            for z in 1..d - 1 {
                let mut c = if sliver_here {
                    sliver_sample(sprite, seg, r, z, d, on_right).and_then(|px| {
                        // A shadow-colored source (a sliver pixel
                        // sampled from a recess row) is not
                        // material evidence: fall back to the
                        // profile's fill, as does missing evidence
                        // (a non-structure sliver sample).
                        if lab_of(px).l < SHADOW_L {
                            None
                        } else {
                            Some(deshade(px, anchor.unwrap_or_else(|| gain_at(y))))
                        }
                    })
                } else {
                    None
                };
                if c.is_none() {
                    c = match construction {
                        // A Panel flank falls back to the panel
                        // itself; the field-based profiles fall
                        // back to the row's field, as before.
                        Construction::Panel => Some(back.side_color(z, y, fh)),
                        _ => side_colors[y as usize],
                    };
                }
                if let Some(c) = c {
                    m.set(edge_x, y, z, Some(c));
                }
            }
        }
    }
    // Top surface: per column, its topmost solid voxel, colored
    // from the top region across the depth behind the front plane.
    // Coverage-drive seamless models pass `fitted_top`: their
    // segmentation's "top region" is a single edge row, and
    // stretching that row across the depth produced streak
    // artifacts on every top (Joey's review, 2026-11-17) — a
    // seamless body has no top-face evidence, so its top is one
    // fitted material instead, the back-panel treatment.
    for x in 0..w {
        let Some(ytop) = (0..fh).rev().find(|&y| is_solid(x, y)) else {
            continue;
        };
        for z in 0..d - 1 {
            if let Some(c) = fitted_top {
                m.set(x, ytop, z, Some(c));
            } else {
                let sr = top_row_for_z(seg, d, z);
                if let Some(px) = row_pixel(sprite, x, sr, x0, x1) {
                    m.set(x, ytop, z, Some(px));
                }
            }
        }
    }
    m
}

// ---------------------------------------------------------------------
// View classification + view-faithful builders (Stage G.3 rework;
// see the module docs, "View classification").
// ---------------------------------------------------------------------

/// The view a furniture sprite is drawn in (module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteView {
    /// The signed-off reading: top band / front face / depth.
    Front,
    /// The sprite is the object's top face (a plan view).
    TopDown,
    /// The sprite is the object's end face; the object extends
    /// away from the viewer along the depth axis.
    EndOn,
}

/// Per-id view pins: the ground truth for [`view_for`], beside
/// the other Stage G override tables. An entry exists because
/// the sprite's view was verified against the depicted object
/// (pipeline §24): f_standing_tank's sprite is the end cap of a
/// horizontal plastic leg tank (photo-verified), the f_bed chain
/// and f_slab are plan views.
pub const VIEW_OVERRIDES: &[(&str, SpriteView)] = &[
    ("f_standing_tank", SpriteView::EndOn),
    ("f_storage_tank", SpriteView::EndOn), // looks_like f_standing_tank
    ("f_bed", SpriteView::TopDown),
    ("f_bunkbed", SpriteView::TopDown), // sprite resolves to f_bed
    ("f_bunkbed_down", SpriteView::TopDown),
    ("f_triple_bunkbed", SpriteView::TopDown),
    ("f_slab", SpriteView::TopDown),
];

/// The view a sprite is drawn in: [`VIEW_OVERRIDES`] pins first
/// (ground truth), then a conservative heuristic — a sprite that
/// does not segment at all, or whose top region covers >= 85% of
/// its silhouette height, is proposed TopDown; everything else is
/// Front. The heuristic only proposes: on the pinned set it is
/// wrong for every entry (the tank and the bed both "segment"
/// with shallow top regions), which is exactly why the pins are
/// the ground truth.
pub fn view_for(id: Option<&str>, sprite: &RgbaImage) -> SpriteView {
    if let Some(id) = id {
        if let Some((_, v)) = VIEW_OVERRIDES.iter().find(|(k, _)| *k == id) {
            return *v;
        }
    }
    match segment(sprite) {
        None => SpriteView::TopDown,
        Some(seg) => {
            let silh = seg.silhouette_rows.end - seg.silhouette_rows.start;
            let top_h = seg.top_rows.end - seg.top_rows.start;
            if silh > 0 && top_h * 100 >= 85 * silh {
                SpriteView::TopDown
            } else {
                SpriteView::Front
            }
        }
    }
}

/// The routed prop-model entry point (Stage G.3 rework): build
/// the model the sprite's VIEW calls for. Front is the signed-off
/// path — the assigned archetype when [`crate::archetypes`]
/// has one, else [`solid_model_for`]. TopDown / EndOn go to the
/// view-faithful builders, but ONLY for ids pinned in
/// [`VIEW_OVERRIDES`] (pipeline §24.4: the new builders fire
/// initially exactly for the pinned set); an unpinned id whose
/// heuristic merely proposes TopDown keeps the Front path.
pub fn model_for(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    let pinned = VIEW_OVERRIDES.iter().any(|(k, _)| *k == id);
    match view_for(Some(id), sprite) {
        SpriteView::TopDown if pinned => topdown_model(id, sprite),
        SpriteView::EndOn if pinned => endon_model(id, sprite),
        _ => {
            if let Some(arch) = crate::archetypes::archetype_for(id) {
                let seg = segment_for(id, sprite)?;
                Some(crate::archetypes::build(arch, sprite, &seg))
            } else if let Some(seg) = segment_for(id, sprite) {
                Some(solid_model_for(id, sprite, &seg))
            } else {
                coverage_model(id, sprite)
            }
        }
    }
}

// ---------------------------------------------------------------------
// Coverage drive: base models for the sprites segment() declines.
// ---------------------------------------------------------------------

/// Plan-view flat objects, pinned per id after visual review
/// (the faithfulness rule: a groundsheet must not be stood up
/// as a tall box). The value is the slab thickness in cells —
/// ASSUMED (a groundsheet/tarp/mat is a few cells thick; a
/// mattress is a low pad, 6). The builder is [`plan_slab`]:
/// the sprite IS the top face, verbatim; the sides/bottom are
/// the silhouette's modal border paint.
pub const PLAN_SLAB_PINS: &[(&str, u32)] = &[
    ("f_groundsheet", 2),
    ("f_center_groundsheet", 2),
    ("f_fema_groundsheet", 2),
    ("f_large_groundsheet", 2),
    ("f_plastic_groundsheet", 2),
    ("f_skin_groundsheet", 2),
    ("f_leather_tarp", 2),
    ("f_tatami", 2),
    ("f_fiber_mat", 2),
    ("f_canvas_floor", 2),
    ("f_down_mattress", 6),
    ("f_floor_mattress", 6),
    ("f_ash", 3),
    ("f_leaves_pile", 3),
    ("f_firering", 3),
    ("f_scrap_bridge", 3),
];

/// Thin panel referents, pinned per id after visual review:
/// doors, a mirror, a notice board, grave stones, a sign, a
/// target. The value is the panel thickness in cells (3,
/// ASSUMED — a door slab / stone tablet at prop scale). The
/// builder is [`panel_slab`]: front verbatim, back the sprite's
/// modal structure paint as a plain panel.
pub const PANEL_SLAB_PINS: &[(&str, u32)] = &[
    ("f_canvas_door", 3),
    ("f_canvas_door_o", 3),
    ("f_large_canvas_door", 3),
    ("f_large_canvas_door_o", 3),
    ("f_skin_door", 3),
    ("f_skin_door_o", 3),
    ("f_beaded_door", 3),
    ("f_beaded_door_o", 3),
    ("f_bigmirror", 3),
    ("f_bigmirror_b", 3),
    ("f_bulletin", 3),
    ("f_grave_stone", 3),
    ("f_grave_stone_old", 3),
    ("f_grave_head", 3),
    ("f_sign", 3),
    ("f_target", 3),
];

/// Ids the seamless criteria (below) would otherwise take, held
/// out after visual review of the actual sprites: each depicts
/// something a silhouette solid would misrepresent — a pole
/// (f_traffic_light: the metrics read its narrow extent, the
/// sprite is a thin pole), a slatted frame whose gaps are
/// shadow paint (f_bed_frame), folding/sling chairs
/// (f_deckchair, f_chair — the latter also pictogram-scale at
/// 11 cells wide), flat trailer-interior sections
/// (f_bullettrailer_table_*), a dish/antenna skeleton
/// (f_scrap_antenna, f_small_satelitte_dish), an alien
/// organism whose mass is tendrils on a rock (f_alien_anemone),
/// and a wall-hung ring (f_winter_wreath).
pub const SEAMLESS_EXCLUSIONS: &[&str] = &[
    "f_traffic_light",
    "f_bed_frame",
    "f_deckchair",
    "f_chair",
    "f_bullettrailer_table_bottom",
    "f_bullettrailer_table_middle",
    "f_bullettrailer_table_top",
    "f_scrap_antenna",
    "f_small_satelitte_dish",
    "f_alien_anemone",
    "f_winter_wreath",
];

/// Seamless-solid firing criteria (measured on the coverage
/// audit, 232 unmodeled ids): the silhouette band must be at
/// least this many rows tall (a substantial body, not a
/// countertop pictogram)...
const SEAMLESS_MIN_BAND_ROWS: u32 = 12;
/// ...with at most this fraction of band rows under the
/// silhouette coverage floor (segment()'s own ragged bound is
/// 0.10 for the seam split; the silhouette mask itself tolerates
/// more dips — a dip is shape, preserved by the mask)...
const SEAMLESS_MAX_THIN_FRACTION: f32 = 0.15;
/// ...a mean band coverage of at least this (the coverage floor
/// is band-relative, not sprite-relative: the sprite-relative
/// opaque floor segment() uses punishes 64px sprites whose
/// object is centered, e.g. the gun safes at 0.305)...
const SEAMLESS_MIN_BAND_COVERAGE: f32 = 0.42;
/// ...a structure-mask fill of the silhouette bounding box of
/// at least this (solid bodies vs lattice/vegetation)...
const SEAMLESS_MIN_BBOX_FILL: f32 = 0.65;
/// ...a mean per-row extent coverage of at least this (rows
/// are filled across their own extent, not just in aggregate)...
const SEAMLESS_MIN_EXTENT_COVERAGE: f32 = 0.75;
/// ...and at least this fraction of mask pixels at chamfer
/// distance >= 3 from the background (real interior mass;
/// poles, rails, plant strokes and ladders measure < 0.10).
const SEAMLESS_MIN_DEEP_FRACTION: f32 = 0.20;

/// The silhouette band (first..last+1 sprite rows at or above
/// [`MIN_ROW_COVERAGE`] opaque coverage), computed exactly as
/// segment() computes it.
fn opaque_band(sprite: &RgbaImage) -> Option<Range<u32>> {
    let (w, h) = (sprite.width(), sprite.height());
    let cov = |y: u32| -> f32 {
        (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, y)))
            .count() as f32
            / w as f32
    };
    let first = (0..h).find(|&y| cov(y) >= MIN_ROW_COVERAGE)?;
    let last = (0..h).rev().find(|&y| cov(y) >= MIN_ROW_COVERAGE)?;
    if last <= first {
        return None;
    }
    Some(first..last + 1)
}

/// A synthesized segmentation for a solid-body sprite that has
/// no readable top/front seam: the whole silhouette band is the
/// front, and the top region is the front's top edge row only —
/// it exists to drive [`Segmentation::depth_cells_for`] (the
/// depth follows it as usual: floor-standing silhouettes get at
/// least width/3, short ones stay thin). The coverage path does
/// NOT color the top face from that row (the streak artifacts of
/// Joey's 2026-11-17 review); it passes a fitted top material
/// ([`seamless_top_color`]) to the solid model instead. Fires only on the measured criteria above
/// ([`SEAMLESS_MIN_BAND_ROWS`] etc.); `confidence` carries the
/// band coverage, as there is no seam to be confident about.
pub fn seamless_segment(sprite: &RgbaImage) -> Option<Segmentation> {
    let (w, h) = (sprite.width(), sprite.height());
    if w < 8 || h < 8 {
        return None;
    }
    let band = opaque_band(sprite)?;
    let band_h = band.end - band.start;
    if band_h < SEAMLESS_MIN_BAND_ROWS {
        return None;
    }
    let cov = |y: u32| -> f32 {
        (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, y)))
            .count() as f32
            / w as f32
    };
    let in_band: Vec<f32> = band.clone().map(cov).collect();
    let thin = in_band.iter().filter(|&&c| c < MIN_ROW_COVERAGE).count() as f32 / band_h as f32;
    if thin > SEAMLESS_MAX_THIN_FRACTION {
        return None;
    }
    let wcov = in_band.iter().sum::<f32>() / band_h as f32;
    if wcov < SEAMLESS_MIN_BAND_COVERAGE {
        return None;
    }
    // Structure-mask metrics over the band.
    let mask = structure_mask(sprite);
    let at = |x: u32, y: u32| mask[(y * w + x) as usize];
    let (mut x0, mut x1, mut n) = (w, 0u32, 0u32);
    for y in band.clone() {
        for x in 0..w {
            if at(x, y) {
                x0 = x0.min(x);
                x1 = x1.max(x);
                n += 1;
            }
        }
    }
    if n == 0 {
        return None;
    }
    let fill = n as f32 / ((x1 - x0 + 1) * band_h) as f32;
    if fill < SEAMLESS_MIN_BBOX_FILL {
        return None;
    }
    let mut xcov_sum = 0.0f32;
    for y in band.clone() {
        let xs: Vec<u32> = (0..w).filter(|&x| at(x, y)).collect();
        if xs.is_empty() {
            continue;
        }
        xcov_sum += xs.len() as f32 / (xs[xs.len() - 1] - xs[0] + 1) as f32;
    }
    if xcov_sum / band_h as f32 <= SEAMLESS_MIN_EXTENT_COVERAGE {
        return None;
    }
    let dist = chamfer_distances(&mask, w, h);
    let deep = (band.clone())
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| at(x, y) && dist[(y * w + x) as usize] >= 3)
        .count();
    if deep as f32 / n as f32 <= SEAMLESS_MIN_DEEP_FRACTION {
        return None;
    }
    Some(Segmentation {
        silhouette_rows: band.clone(),
        top_rows: band.start..band.start + 1,
        front_rows: band,
        side: None,
        confidence: wcov,
    })
}

/// A plan-view flat object as a low slab: model x = sprite x,
/// model z = sprite row, the top cell of every silhouette
/// column is the sprite pixel verbatim (the sprite IS the top
/// face), and the cells below are the silhouette's modal border
/// paint, so the slab's edge reads as the object's own edge.
/// Only silhouette columns are filled — a ring (f_firering)
/// keeps its hole.
fn plan_slab(sprite: &RgbaImage, thickness: u32) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mask = structure_mask(sprite);
    if !mask.iter().any(|m| *m) {
        return None;
    }
    let dist = chamfer_distances(&mask, w, h);
    let edge = modal_color(
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = (y * w + x) as usize;
                mask[i] && dist[i] == 1
            })
            .map(|(x, y)| *sprite.get_pixel(x, y)),
    )
    .or_else(|| modal_color(sprite.pixels().copied()))?;
    let mut m = PropModel::new([w, thickness, h]);
    for z in 0..h {
        for x in 0..w {
            if !mask[(z * w + x) as usize] {
                continue;
            }
            let top = *sprite.get_pixel(x, z);
            for y in 0..thickness {
                m.set(x, y, z, Some(if y == thickness - 1 { top } else { edge }));
            }
        }
    }
    Some(m)
}

/// A thin panel referent as a slab of the stated thickness:
/// the front plane is the sprite's silhouette-band pixels
/// verbatim, the back plane is the sprite's modal structure
/// paint (a plain panel), interior cells carry the front pixel
/// (the solid model's convention).
fn panel_slab(sprite: &RgbaImage, thickness: u32) -> Option<PropModel> {
    let (w, _) = (sprite.width(), sprite.height());
    let band = opaque_band(sprite)?;
    let fh = band.end - band.start;
    let back = modal_color(
        band.clone()
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| structure(*sprite.get_pixel(x, y)))
            .map(|(x, y)| *sprite.get_pixel(x, y)),
    )?;
    let mut m = PropModel::new([w, fh, thickness]);
    let bot = band.end - 1;
    for y in 0..fh {
        let r = bot - y;
        for x in 0..w {
            let px = *sprite.get_pixel(x, r);
            if !structure(px) {
                continue;
            }
            for z in 0..thickness {
                let c = if z == thickness - 1 {
                    px
                } else if z == 0 {
                    back
                } else {
                    px
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    Some(m)
}

/// The fitted top material for a seamless model: the modal
/// structure paint of the front's top quarter band, de-shaded by
/// the top row's gain. A seamless sprite carries no top-face
/// evidence (that is what "seamless" means), so — like the back
/// — the top is one fitted material, not a texture.
fn seamless_top_color(sprite: &RgbaImage, seg: &Segmentation) -> Option<Rgba<u8>> {
    let (w, _) = (sprite.width(), sprite.height());
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let band = (fh / 4).max(1);
    let modal = modal_color(
        (front.start..front.start + band)
            .flat_map(|r| (0..w).map(move |x| (x, r)))
            .filter(|&(x, r)| structure(*sprite.get_pixel(x, r)))
            .map(|(x, r)| *sprite.get_pixel(x, r)),
    )
    .or_else(|| {
        modal_color(
            front
                .clone()
                .flat_map(|r| (0..w).map(move |x| (x, r)))
                .filter(|&(x, r)| structure(*sprite.get_pixel(x, r)))
                .map(|(x, r)| *sprite.get_pixel(x, r)),
        )
    })?;
    let gains = row_gains(sprite, front, None);
    Some(deshade(modal, gains.first().copied().unwrap_or(1.0)))
}

/// Round-plan referents among the seamless ids: barrels, vats,
/// a water heater, a keg, a trash can, a stump stool, a hanging
/// bag, boulders, a sand pile, a fungal mass. Their sprite is
/// the elevation of a body whose plan is a disc, so the
/// rectangular seamless footprint made them square blocks
/// (Joey's review, 2026-11-17). Each pin was verified against
/// its sprite: the depicted object is round in plan.
pub const CYLINDER_PINS: &[&str] = &[
    "f_30gal_firebarrel",
    "f_30gal_firebarrel_fd_fire",
    "f_55gal_firebarrel",
    "f_55gal_firebarrel_fd_fire",
    "f_fvat_empty",
    "f_fvat_full",
    "f_water_heater",
    "f_wood_keg",
    "f_trashcan",
    "f_logstool",
    "f_punching_bag",
    "f_boulder_large",
    "f_boulder_medium",
    "f_boulder_small",
    "f_glow_boulder",
    "f_sand_castle",
    "f_fungal_clump",
];

/// A round-plan body from its elevation sprite: every sprite
/// column becomes a voxel column carrying that column's sprite
/// paint at every height (the front-most voxel of each column
/// is the sprite pixel verbatim, and the column's top voxel is
/// the sprite's own top pixel in that column — real data, no
/// fitted top needed), and the set of columns is clipped to the
/// ellipse inscribed in the silhouette extent, with depth =
/// extent width (a disc), capped at 32 (one tile). Per-column
/// vertical ranges follow the sprite's structure mask exactly:
/// solid where the sprite says object in that column, empty
/// where not (a hanging bag does not grow to the floor).
fn cylinder_model(sprite: &RgbaImage, seg: &Segmentation) -> Option<PropModel> {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let bot = front.end - 1;
    let (x0, x1) = front_structure_extent(sprite, seg);
    let width = x1 - x0 + 1;
    if width < 6 {
        return None;
    }
    let d = width.min(32);
    let cx = (x0 + x1) as f32 / 2.0;
    let cz = (d - 1) as f32 / 2.0;
    let (rx, rz) = (width as f32 / 2.0, d as f32 / 2.0);
    let inside = |x: u32, z: u32| {
        let dx = (x as f32 - cx) / rx;
        let dz = (z as f32 - cz) / rz;
        dx * dx + dz * dz <= 1.0
    };
    let mut m = PropModel::new([w, fh, d]);
    for x in x0..=x1 {
        for y in 0..fh {
            let px = *sprite.get_pixel(x, bot - y);
            if !structure(px) {
                continue;
            }
            for z in 0..d {
                if inside(x, z) {
                    m.set(x, y, z, Some(px));
                }
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// The coverage-drive fallback (see the section header):
/// reached only when [`segment_for`] declines and the id is not
/// view-pinned. Pin lists first (plan, then panel), then the
/// seamless solid via [`seamless_segment`] +
/// [`solid_model_for`], minus [`SEAMLESS_EXCLUSIONS`].
/// Round-plan pins ([`CYLINDER_PINS`]) route to
/// [`cylinder_model`] instead of the rectangular solid.
pub fn coverage_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    if let Some((_, t)) = PLAN_SLAB_PINS.iter().find(|(k, _)| *k == id) {
        return plan_slab(sprite, *t);
    }
    if let Some((_, t)) = PANEL_SLAB_PINS.iter().find(|(k, _)| *k == id) {
        return panel_slab(sprite, *t);
    }
    if SEAMLESS_EXCLUSIONS.contains(&id) {
        return None;
    }
    let seg = seamless_segment(sprite)?;
    if CYLINDER_PINS.contains(&id) {
        if let Some(m) = cylinder_model(sprite, &seg) {
            return Some(m);
        }
    }
    let top = seamless_top_color(sprite, &seg);
    // The open-base reading (base_plan) is a table reading: it
    // keeps the header slab + grounded columns and drops the
    // rest. That is faithful only when nothing real is dropped.
    // A pedestal / cantilever body (f_toilet's bowl overhanging
    // its pedestal, a statue's figure) carries STRUCTURE below
    // the slab in ungrounded columns — dropping it would hollow
    // the object — so when the dropped structure count is
    // nonzero the seamless model is built CLOSED (the whole
    // silhouette, the base standard). Bench sections and other
    // true open bases drop nothing and keep the open reading.
    let (x0, x1) = front_structure_extent(sprite, &seg);
    let plan = base_plan(sprite, &seg, x0, x1);
    if plan.open {
        let fh = seg.front_rows.end - seg.front_rows.start;
        let slab_bottom = fh - plan.header_rows;
        let bot = seg.front_rows.end - 1;
        let mut dropped = 0u32;
        for y in 0..slab_bottom {
            for x in x0..=x1 {
                if !plan.grounded[(x - x0) as usize]
                    && structure(*sprite.get_pixel(x, bot - y))
                {
                    dropped += 1;
                }
            }
        }
        if dropped > 0 {
            let construction = construction_for(Some(id), sprite, &seg);
            return Some(solid_model_impl(sprite, &seg, construction, true, top));
        }
    }
    let construction = construction_for(Some(id), sprite, &seg);
    Some(solid_model_impl(sprite, &seg, construction, false, top))
}

/// The sprite's structure mask in plan order (index y * w + x).
fn structure_mask(sprite: &RgbaImage) -> Vec<bool> {
    sprite.pixels().map(|p| structure(*p)).collect()
}

/// Chamfer distance (4-neighborhood BFS steps) from each plan
/// cell to the nearest non-silhouette cell; non-silhouette cells
/// are 0, silhouette edge cells are 1. When the silhouette fills
/// the whole grid, the border counts as outside.
fn chamfer_distances(sil: &[bool], w: u32, h: u32) -> Vec<u32> {
    let idx = |x: u32, y: u32| (y * w + x) as usize;
    let mut dist = vec![0u32; (w * h) as usize];
    let mut q: std::collections::VecDeque<(u32, u32)> = std::collections::VecDeque::new();
    for y in 0..h {
        for x in 0..w {
            if !sil[idx(x, y)] {
                q.push_back((x, y));
            }
        }
    }
    if q.is_empty() {
        for x in 0..w {
            for y in [0, h - 1] {
                if sil[idx(x, y)] && dist[idx(x, y)] == 0 {
                    dist[idx(x, y)] = 1;
                    q.push_back((x, y));
                }
            }
        }
        for y in 0..h {
            for x in [0, w - 1] {
                if sil[idx(x, y)] && dist[idx(x, y)] == 0 {
                    dist[idx(x, y)] = 1;
                    q.push_back((x, y));
                }
            }
        }
    }
    while let Some((x, y)) = q.pop_front() {
        let d = dist[idx(x, y)];
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx >= w || ny >= h {
                continue;
            }
            let i = idx(nx, ny);
            if sil[i] && dist[i] == 0 {
                dist[i] = d + 1;
                q.push_back((nx, ny));
            }
        }
    }
    dist
}

/// The modal (most common) color of a pixel stream, ties toward
/// the smallest RGBA key (BTreeMap order + strict >), as in
/// [`row_field`].
pub(crate) fn modal_color<I: Iterator<Item = Rgba<u8>>>(it: I) -> Option<Rgba<u8>> {
    let mut counts: std::collections::BTreeMap<[u8; 4], u32> =
        std::collections::BTreeMap::new();
    for px in it {
        *counts.entry(px.0).or_insert(0) += 1;
    }
    let mut best: Option<([u8; 4], u32)> = None;
    for (k, n) in counts {
        if best.map_or(true, |(_, bn)| n > bn) {
            best = Some((k, n));
        }
    }
    best.map(|(k, _)| Rgba(k))
}

// ---------------------------------------------------------------------
// TopDown builders
// ---------------------------------------------------------------------

/// Which top-down construction a plan sprite gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopDownKind {
    /// Concentric-ring heightmap (a stepped slab).
    Terrace,
    /// Border frame + interior surface (a bed).
    Framed,
}

/// Per-id top-down construction pins (the case-by-case layer).
pub const TOPDOWN_KIND_OVERRIDES: &[(&str, TopDownKind)] = &[
    ("f_slab", TopDownKind::Terrace),
    ("f_bed", TopDownKind::Framed),
    ("f_bunkbed", TopDownKind::Framed),
    ("f_bunkbed_down", TopDownKind::Framed),
    ("f_triple_bunkbed", TopDownKind::Framed),
];

/// The top-down construction for an id: pin first, then a
/// heuristic proposal — a plan whose rim band and interior share
/// one material reads as a terrace; a plan with a distinct
/// border material around a different interior reads as framed.
fn topdown_kind(id: &str, sprite: &RgbaImage) -> TopDownKind {
    if let Some((_, k)) = TOPDOWN_KIND_OVERRIDES.iter().find(|(k, _)| *k == id) {
        return *k;
    }
    let (w, h) = (sprite.width(), sprite.height());
    let sil = structure_mask(sprite);
    let dist = chamfer_distances(&sil, w, h);
    let at = |x: u32, y: u32| *sprite.get_pixel(x, y);
    let interior = modal_color(
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| sil[(y * w + x) as usize] && dist[(y * w + x) as usize] > 4)
            .map(|(x, y)| at(x, y)),
    );
    let rim = modal_color(
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = (y * w + x) as usize;
                sil[i] && dist[i] >= 1 && dist[i] <= 4
            })
            .map(|(x, y)| at(x, y)),
    );
    match (interior, rim) {
        (Some(a), Some(b)) if lab_of(a).distance(lab_of(b)) >= 0.08 => TopDownKind::Framed,
        _ => TopDownKind::Terrace,
    }
}

/// A top-down prop model (plan sprite: model x = sprite x,
/// model z = sprite row, sprite top rows at the model's back,
/// z = 0). Dispatches on [`topdown_kind`].
pub fn topdown_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    match topdown_kind(id, sprite) {
        TopDownKind::Terrace => terrace_model(sprite),
        TopDownKind::Framed => framed_model(id, sprite),
    }
}

/// Fitted terrace parameters (module docs, "Terrace").
#[derive(Debug, Clone, PartialEq)]
pub struct TerraceFit {
    /// Ring levels used (distinct chamfer distances, capped).
    pub levels: u32,
    /// Model cells of height per ring level.
    pub step: u32,
    /// The step-side material: modal color of the boundary ring.
    pub edge: Rgba<u8>,
}

/// Terrace fit: ring count from the sprite's own chamfer
/// distances (cap 4 levels), step scaled so the maximum height
/// is ~1/4 of the sprite width (minimum height 2 for one level).
pub fn terrace_fit(sprite: &RgbaImage) -> Option<TerraceFit> {
    let (w, h) = (sprite.width(), sprite.height());
    let sil = structure_mask(sprite);
    if !sil.iter().any(|s| *s) {
        return None;
    }
    let dist = chamfer_distances(&sil, w, h);
    let mut distinct: Vec<u32> = dist
        .iter()
        .zip(sil.iter())
        .filter(|(_, s)| **s)
        .map(|(d, _)| *d)
        .collect();
    distinct.sort_unstable();
    distinct.dedup();
    let levels = (distinct.len() as u32).min(4).max(1);
    let step = (((w / 4) as f32 / levels as f32).round() as u32).max(1);
    let edge = modal_color(
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = (y * w + x) as usize;
                sil[i] && dist[i] == 1
            })
            .map(|(x, y)| *sprite.get_pixel(x, y)),
    )?;
    Some(TerraceFit { levels, step, edge })
}

/// The height of a terrace column: its chamfer ring, capped at
/// the fit's levels, times the fit's step.
fn terrace_height(fit: &TerraceFit, dist: u32) -> u32 {
    dist.min(fit.levels) * fit.step
}

fn terrace_model(sprite: &RgbaImage) -> Option<PropModel> {
    let fit = terrace_fit(sprite)?;
    let (w, h) = (sprite.width(), sprite.height());
    let sil = structure_mask(sprite);
    let dist = chamfer_distances(&sil, w, h);
    let max_h = fit.levels * fit.step;
    let mut m = PropModel::new([w, max_h, h]);
    for z in 0..h {
        for x in 0..w {
            let i = (z * w + x) as usize;
            if !sil[i] {
                continue;
            }
            let col_h = terrace_height(&fit, dist[i]);
            let top_px = *sprite.get_pixel(x, z);
            for y in 0..col_h {
                let c = if y == col_h - 1 { top_px } else { fit.edge };
                m.set(x, y, z, Some(c));
            }
        }
    }
    Some(m)
}

/// Per-id head-end pins for the framed construction: `true` =
/// the head end is the sprite's TOP rows (plan z = 0). Detection
/// alone picks f_bed's foot (its band + posts are wider than the
/// headboard band), so the bed chain is pinned to the depicted
/// headboard — the plain band at the sprite's top rows.
pub const HEAD_END_OVERRIDES: &[(&str, bool)] = &[
    ("f_bed", true),
    ("f_bunkbed", true),
    ("f_bunkbed_down", true),
    ("f_triple_bunkbed", true),
];

/// Fitted framed-surface (bed) parameters (module docs,
/// "Framed surface").
#[derive(Debug, Clone, PartialEq)]
pub struct BedFit {
    /// The frame material: modal color of the border's frame
    /// cells (the wood).
    pub frame: Rgba<u8>,
    /// The interior material: modal color of the interior cells
    /// (the mattress).
    pub mattress: Rgba<u8>,
    /// Model y of the frame rails' top face.
    pub frame_top: u32,
    /// Model y of the mattress top face (one cell proud of the
    /// rails).
    pub mattress_top: u32,
    /// Model y of the headboard slab's top.
    pub headboard_top: u32,
    /// Plan rows (z) the headboard slab occupies.
    pub head_rows: Range<u32>,
    /// Plan x extent (inclusive) the headboard slab spans.
    pub head_x: (u32, u32),
    /// Model y of the footboard panel's top face: the mattress
    /// top itself (the spec band is mattress_top ± 2 — a
    /// footboard rises TO mattress height, not above it).
    pub footboard_top: u32,
    /// Plan rows (z) the footboard panel occupies.
    pub foot_rows: Range<u32>,
    /// Plan x extent (inclusive) the footboard panel spans.
    pub foot_x: (u32, u32),
}

/// Rim band width (plan cells) within which border material is
/// read as frame rather than interior surface.
const FRAME_RIM: u32 = 4;
/// OKLab distance from the interior's modal color beyond which
/// a rim cell is border material, not interior.
const FRAME_COLOR_SPLIT: f32 = 0.10;

/// The framed-surface fit. Returns None when the plan has no
/// readable interior (no cells deeper than the rim band).
pub fn bed_fit(id: &str, sprite: &RgbaImage) -> Option<BedFit> {
    let (w, h) = (sprite.width(), sprite.height());
    let sil = structure_mask(sprite);
    if !sil.iter().any(|s| *s) {
        return None;
    }
    let dist = chamfer_distances(&sil, w, h);
    let at = |x: u32, z: u32| *sprite.get_pixel(x, z);
    let cell = |x: u32, z: u32| (z * w + x) as usize;
    // Interior material: the modal color of the deep cells.
    let mattress = modal_color(
        (0..h)
            .flat_map(|z| (0..w).map(move |x| (x, z)))
            .filter(|&(x, z)| sil[cell(x, z)] && dist[cell(x, z)] > FRAME_RIM)
            .map(|(x, z)| at(x, z)),
    )?;
    // Frame material: the modal color of the rim cells that are
    // NOT the interior material (the border's own material).
    let frame = modal_color(
        (0..h)
            .flat_map(|z| (0..w).map(move |x| (x, z)))
            .filter(|&(x, z)| {
                let i = cell(x, z);
                sil[i] && dist[i] <= FRAME_RIM
                    && lab_of(at(x, z)).distance(lab_of(mattress)) > FRAME_COLOR_SPLIT
            })
            .map(|(x, z)| at(x, z)),
    )?;
    let is_frame = |x: u32, z: u32| -> bool {
        let i = cell(x, z);
        sil[i] && dist[i] <= FRAME_RIM
            && lab_of(at(x, z)).distance(lab_of(frame)) <= FRAME_COLOR_SPLIT
    };
    // End strips: consecutive rows from an end whose silhouette
    // cells are >= 80% frame. Scored by (depth, then darkness of
    // the strip's frame cells); the pin table decides for the
    // bed chain (module docs).
    let strip = |from_top: bool| -> (u32, f32) {
        let mut depth = 0u32;
        let mut lsum = 0.0f32;
        let mut n = 0u32;
        for k in 0..h {
            let z = if from_top { k } else { h - 1 - k };
            let row: Vec<u32> = (0..w).filter(|&x| sil[cell(x, z)]).collect();
            if row.is_empty() {
                break;
            }
            let total = row.len();
            let fr: Vec<u32> = row.into_iter().filter(|&x| is_frame(x, z)).collect();
            if (fr.len() as f32) < 0.8 * total as f32 {
                break;
            }
            for x in fr {
                lsum += lab_of(at(x, z)).l;
                n += 1;
            }
            depth += 1;
        }
        (depth, if n > 0 { lsum / n as f32 } else { 1.0 })
    };
    let (top_strip, top_dark) = strip(true);
    let (bot_strip, bot_dark) = strip(false);
    let detected_top = top_strip > bot_strip
        || (top_strip == bot_strip && top_dark <= bot_dark);
    let head_at_top = HEAD_END_OVERRIDES
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
        .unwrap_or(detected_top);
    let head_depth = if head_at_top { top_strip } else { bot_strip }.max(1);
    let head_rows = if head_at_top {
        0..head_depth
    } else {
        (h - head_depth)..h
    };
    // The slab spans the strip's structure x extent.
    let (mut hx0, mut hx1) = (w - 1, 0u32);
    for z in head_rows.clone() {
        for x in 0..w {
            if sil[cell(x, z)] {
                hx0 = hx0.min(x);
                hx1 = hx1.max(x);
            }
        }
    }
    // The footboard: the OTHER end's strip, capped at the
    // headboard's plan thickness (the frame's end panels share
    // a thickness; the foot strip as drawn also carries the
    // band's painted shadow, which is not panel). At least one
    // row: the foot-end plane is always a panel.
    let foot_depth_raw = if head_at_top { bot_strip } else { top_strip };
    let foot_depth = foot_depth_raw.clamp(1, head_depth.max(1));
    let foot_rows = if head_at_top {
        (h - foot_depth)..h
    } else {
        0..foot_depth
    };
    let (mut fx0, mut fx1) = (w - 1, 0u32);
    for z in foot_rows.clone() {
        for x in 0..w {
            if sil[cell(x, z)] {
                fx0 = fx0.min(x);
                fx1 = fx1.max(x);
            }
        }
    }
    let mattress_top = (w / 4).max(4);
    let headboard_top = mattress_top + (w * 5 / 16).max(6);
    Some(BedFit {
        frame,
        mattress,
        frame_top: mattress_top - 1,
        mattress_top,
        headboard_top,
        head_rows,
        head_x: (hx0, hx1),
        footboard_top: mattress_top,
        foot_rows,
        foot_x: (fx0, fx1),
    })
}

fn framed_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    let fit = bed_fit(id, sprite)?;
    let (w, h) = (sprite.width(), sprite.height());
    let sil = structure_mask(sprite);
    let dist = chamfer_distances(&sil, w, h);
    let cell = |x: u32, z: u32| (z * w + x) as usize;
    let is_frame = |x: u32, z: u32| -> bool {
        let i = cell(x, z);
        sil[i] && dist[i] <= FRAME_RIM
            && lab_of(*sprite.get_pixel(x, z)).distance(lab_of(fit.frame)) <= FRAME_COLOR_SPLIT
    };
    let mut m = PropModel::new([w, fit.headboard_top + 1, h]);
    for z in 0..h {
        for x in 0..w {
            if fit.head_rows.contains(&z) && x >= fit.head_x.0 && x <= fit.head_x.1 {
                // The headboard: a plain slab in the frame
                // material, ground to above the mattress — its
                // back face (the plan's head-end plane) is 100%
                // frame material, occluding the mattress.
                for y in 0..=fit.headboard_top {
                    m.set(x, y, z, Some(fit.frame));
                }
                continue;
            }
            if fit.foot_rows.contains(&z) && x >= fit.foot_x.0 && x <= fit.foot_x.1 {
                // The footboard: a plain frame panel spanning
                // the full width, ground to the mattress top.
                // Its top face is the sprite verbatim (the
                // top-face rule), the body below it frame.
                for y in 0..fit.footboard_top {
                    m.set(x, y, z, Some(fit.frame));
                }
                let top_px = *sprite.get_pixel(x, z);
                let top_c = if structure(top_px) { top_px } else { fit.frame };
                m.set(x, fit.footboard_top, z, Some(top_c));
                continue;
            }
            if !sil[cell(x, z)] {
                continue;
            }
            let top_px = *sprite.get_pixel(x, z);
            if is_frame(x, z) {
                // Frame wall: extruded down from the top face.
                for y in 0..=fit.frame_top {
                    let c = if y == fit.frame_top { top_px } else { fit.frame };
                    m.set(x, y, z, Some(c));
                }
            } else {
                // Mattress block: ~3 cells thick at the top,
                // its top face the sprite verbatim. A PERIMETER
                // mattress cell first gets the side frame wall
                // under it — ground to the mattress underside,
                // in frame material regardless of the cell's
                // top color — so the long sides never gap where
                // the sprite's edge reads as mattress.
                if dist[cell(x, z)] == 1 {
                    for y in 0..(fit.mattress_top - 2) {
                        m.set(x, y, z, Some(fit.frame));
                    }
                }
                for y in (fit.mattress_top - 2)..=fit.mattress_top {
                    let c = if y == fit.mattress_top {
                        top_px
                    } else {
                        fit.mattress
                    };
                    m.set(x, y, z, Some(c));
                }
            }
        }
    }
    Some(m)
}

// ---------------------------------------------------------------------
// EndOn builder (the standing tank)
// ---------------------------------------------------------------------

/// Per-id barrel-length pins for the EndOn tank (the one
/// dimension the end-on sprite cannot show). Empty: the default
/// length = cap diameter serves the reviewed case.
pub const TANK_LENGTH_OVERRIDES: &[(&str, u32)] = &[];

/// How far above the stand height the barrel's underside may
/// sit at the stands' outer columns (the saddle's rise).
const STAND_REACH: u32 = 3;

/// Fitted EndOn tank parameters (module docs, "EndOn").
#[derive(Debug, Clone, PartialEq)]
pub struct TankEndFit {
    /// Cap diameter in cells (= the sprite's silhouette width).
    pub diameter: u32,
    /// Barrel length in cells (depth).
    pub length: u32,
    /// Stand height under the barrel.
    pub leg_h: u32,
    /// Cap center x (sprite/model coords).
    pub center_x: f32,
    /// Sprite rows of the cap region (the front-face source).
    pub cap_rows: Range<u32>,
    /// Neck block extent in model coords: [x0, x1, y0, y1].
    pub neck: [u32; 4],
    /// Depth range the neck occupies (the front end).
    pub neck_z: Range<u32>,
    /// Depth ranges of the two stands: [back, front].
    pub stand_z: [Range<u32>; 2],
    /// Column range the stands span: the columns whose disc
    /// bottom sits within [`STAND_REACH`] cells of the stand
    /// height (the saddle the barrel rests in — wider than the
    /// disc's bottom chord, narrower than the full diameter).
    pub stand_x: Range<u32>,
    /// The hatch fitting color (the sprite's fitting cluster).
    pub fitting: Rgba<u8>,
    /// X range of the fitting cluster in the sprite.
    pub fitting_x: Range<u32>,
    /// Barrel material: the cap region's de-shaded primary.
    pub barrel: Rgba<u8>,
    /// Neck material: modal color of the sprite's neck band.
    pub neck_color: Rgba<u8>,
    /// Stand material: modal color of the sprite's base rows.
    pub base: Rgba<u8>,
}

/// The EndOn tank fit. The cap is the large round face in the
/// sprite's lower portion: its top row is the first silhouette
/// row at >= 95% of the silhouette width, its bottom is the
/// silhouette's last row. The fitting cluster is the non-gray
/// (chromatic) paint in the band above the cap — the brass/olive
/// hatch fitting on the neck.
pub fn endon_tank_fit(id: &str, sprite: &RgbaImage) -> Option<TankEndFit> {
    let (w, h) = (sprite.width(), sprite.height());
    let sil = structure_mask(sprite);
    if !sil.iter().any(|s| *s) {
        return None;
    }
    let cell = |x: u32, y: u32| (y * w + x) as usize;
    let (mut x0, mut x1) = (w - 1, 0u32);
    let (mut top, mut bot) = (h - 1, 0u32);
    for y in 0..h {
        for x in 0..w {
            if sil[cell(x, y)] {
                x0 = x0.min(x);
                x1 = x1.max(x);
                top = top.min(y);
                bot = bot.max(y);
            }
        }
    }
    let diameter = x1 - x0 + 1;
    let center_x = (x0 + x1) as f32 / 2.0;
    let row_count = |y: u32| (0..w).filter(|&x| sil[cell(x, y)]).count() as u32;
    let disc_top = (top..=bot).find(|&y| (row_count(y) as f32) >= 0.95 * diameter as f32)?;
    let cap_rows = disc_top..bot + 1;
    // Fitting cluster: chromatic (non-gray) structure pixels in
    // the neck band, reaching a couple of rows into the cap (the
    // fitting is drawn straddling the cap's top edge).
    let chromatic = |px: Rgba<u8>| -> bool {
        let mx = px[0].max(px[1]).max(px[2]);
        let mn = px[0].min(px[1]).min(px[2]);
        mx - mn >= 20
    };
    let mut fit_px: Vec<(u32, u32, Rgba<u8>)> = Vec::new();
    for y in top..(disc_top + 2).min(h) {
        for x in 0..w {
            let px = *sprite.get_pixel(x, y);
            if structure(px) && chromatic(px) {
                fit_px.push((x, y, px));
            }
        }
    }
    if fit_px.is_empty() {
        return None;
    }
    let (mut fx0, mut fx1) = (w - 1, 0u32);
    let (mut fy0, mut fy1) = (h - 1, 0u32);
    for &(x, y, _) in &fit_px {
        fx0 = fx0.min(x);
        fx1 = fx1.max(x);
        fy0 = fy0.min(y);
        fy1 = fy1.max(y);
    }
    let fitting = modal_color(fit_px.iter().map(|&(_, _, px)| px))?;
    let fitting_h = fy1 - fy0 + 1;
    // Neck: ~the fitting's width + margin (at least a third of
    // the cap), centered on the fitting; it protrudes above the
    // barrel by the fitting's height, embedded 2 cells.
    let leg_h = ((diameter as f32 / 8.0).round() as u32).max(3);
    let neck_w = ((fx1 - fx0 + 1) + 4).max(diameter / 3);
    let fit_cx = (fx0 + fx1) as f32 / 2.0;
    let nx0 = (fit_cx - neck_w as f32 / 2.0).round().max(0.0) as u32;
    let nx1 = (nx0 + neck_w - 1).min(w - 1);
    let barrel_top = leg_h + diameter - 1;
    let neck = [nx0, nx1, barrel_top - 2, barrel_top - 2 + fitting_h + 1];
    let length = TANK_LENGTH_OVERRIDES
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, l)| *l)
        .unwrap_or(diameter);
    let neck_color = modal_color(
        (top..disc_top)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| sil[cell(x, y)])
            .map(|(x, y)| *sprite.get_pixel(x, y)),
    )?;
    let base = modal_color(
        ((bot.saturating_sub(3))..=bot)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| sil[cell(x, y)])
            .map(|(x, y)| *sprite.get_pixel(x, y)),
    )?;
    // Stand columns: the disc's bottom row per column, via the
    // same disc predicate the builder fills with.
    let cy = leg_h as f32 + (diameter - 1) as f32 / 2.0;
    let rad = diameter as f32 / 2.0;
    let disc_bottom = |x: u32| -> Option<u32> {
        let dx = x as f32 - center_x;
        (0..leg_h + diameter).find(|&y| {
            let dy = y as f32 - cy;
            dx * dx + dy * dy <= rad * rad
        })
    };
    let (mut sx0, mut sx1) = (w, 0u32);
    for x in 0..w {
        if let Some(yb) = disc_bottom(x) {
            if yb <= leg_h + STAND_REACH {
                sx0 = sx0.min(x);
                sx1 = sx1.max(x);
            }
        }
    }
    let stand_x = if sx1 >= sx0 { sx0..sx1 + 1 } else { 0..0 };
    let (_, barrel_lab) = region_primaries(sprite, &cap_rows);
    let [r, g, b] = barrel_lab.to_srgb_u8();
    Some(TankEndFit {
        diameter,
        length,
        leg_h,
        center_x,
        cap_rows,
        neck,
        neck_z: (length - neck_w.min(length))..length,
        stand_z: [0..6.min(length), length.saturating_sub(6)..length],
        stand_x,
        fitting,
        fitting_x: fx0..fx1 + 1,
        barrel: Rgba([r, g, b, 255]),
        neck_color,
        base,
    })
}

fn endon_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    let fit = endon_tank_fit(id, sprite)?;
    let (w, _) = (sprite.width(), sprite.height());
    let d = fit.diameter;
    let l = fit.length;
    let cy = fit.leg_h as f32 + (d - 1) as f32 / 2.0;
    let r = d as f32 / 2.0;
    let height = fit.neck[3] + 1;
    let mut m = PropModel::new([w, height, l]);
    let in_disc = |x: u32, y: u32| -> bool {
        let dx = x as f32 - fit.center_x;
        let dy = y as f32 - cy;
        dx * dx + dy * dy <= r * r
    };
    // Stands: under the barrel at both ends — each stand fills
    // its columns from the ground up to the disc's underside
    // (a saddle), in the sprite's base material.
    for zr in &fit.stand_z {
        for z in zr.clone() {
            for x in fit.stand_x.clone() {
                let yb = (0..height).find(|&y| in_disc(x, y)).unwrap_or(0);
                for y in 0..yb {
                    m.set(x, y, z, Some(fit.base));
                }
            }
        }
    }
    // Barrel: the disc extruded along the length. The FRONT
    // plane (z = l - 1) is the sprite's cap region mapped onto
    // the disc (the sprite IS the front view of the end); every
    // other plane is the cap material.
    let cap_h = fit.cap_rows.end - fit.cap_rows.start;
    let cap_bot = fit.cap_rows.end - 1;
    for z in 0..l {
        for y in 0..height {
            for x in 0..w {
                if !in_disc(x, y) {
                    continue;
                }
                let c = if z == l - 1 {
                    let sy = cap_bot
                        - (((y - fit.leg_h) as f32 * (cap_h - 1) as f32
                            / (d - 1) as f32)
                            .round() as u32)
                            .min(cap_h - 1);
                    let px = *sprite.get_pixel(x, sy);
                    if structure(px) {
                        px
                    } else {
                        fit.barrel
                    }
                } else {
                    fit.barrel
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    // Neck + fitting on top of the barrel at the front end.
    // Where the neck overlaps the disc on the FRONT plane, the
    // cap mapping already placed the sprite's paint (the sprite
    // IS the front view of the end) — the neck's own material
    // shows on its protruding cells and its other faces.
    for z in fit.neck_z.clone() {
        for y in fit.neck[2]..=fit.neck[3] {
            for x in fit.neck[0]..=fit.neck[1] {
                if z == l - 1 && in_disc(x, y) {
                    continue;
                }
                let c = if y == fit.neck[3] && fit.fitting_x.contains(&x) {
                    fit.fitting
                } else {
                    fit.neck_color
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    Some(m)
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

    #[test]
    fn depth_floor_only_for_floor_standing_props() {
        // Tall silhouette (30 rows) with a 2-row cap: the raw rule
        // gives 4 (the f_locker failure); the floor gives ceil(32/3).
        let tall = Segmentation {
            silhouette_rows: 0..30,
            top_rows: 0..2,
            front_rows: 2..30,
            side: None,
            confidence: 1.0,
        };
        assert_eq!(tall.depth_cells(), 4, "raw rule stays as documented");
        assert_eq!(tall.depth_cells_for(32), 11, "floor-standing minimum depth");
        // Short silhouette (10 rows): no floor applies.
        let short = Segmentation {
            silhouette_rows: 0..10,
            top_rows: 0..2,
            front_rows: 2..10,
            side: None,
            confidence: 1.0,
        };
        assert_eq!(short.depth_cells_for(32), 4, "no floor for short props");
    }

    #[test]
    fn solid_model_excludes_transparent_black_fringe() {
        // A solid block whose right edge column is transparent
        // black (the f_fridge x=31 case): the fringe column is not
        // structure — no cells there, at any depth.
        let mut img = RgbaImage::new(32, 32);
        for y in 2..6 {
            for x in 0..32 {
                img.put_pixel(x, y, Rgba([210, 190, 150, 255]));
            }
        }
        for y in 6..20 {
            for x in 0..=30 {
                img.put_pixel(x, y, Rgba([150, 115, 80, 255]));
            }
            img.put_pixel(31, y, Rgba([0, 0, 0, 128]));
        }
        let seg = Segmentation {
            silhouette_rows: 2..20,
            top_rows: 2..6,
            front_rows: 6..20,
            side: None,
            confidence: 1.0,
        };
        let sm = solid_model(&img, &seg);
        let (fh, d) = (sm.size[1], sm.size[2]);
        for y in 0..fh {
            for z in 0..d {
                assert_eq!(sm.get(31, y, z), None, "fringe cell at (31,{y},{z})");
                assert!(sm.get(30, y, z).is_some(), "structure at (30,{y},{z})");
            }
            assert_eq!(
                sm.get(30, y, d - 1),
                Some(Rgba([150, 115, 80, 255])),
                "front verbatim next to the fringe"
            );
        }
    }
}
