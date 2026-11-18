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
    solid_model_impl_d(sprite, seg, construction, force_closed, fitted_top, None)
}

fn solid_model_impl_d(
    sprite: &RgbaImage,
    seg: &Segmentation,
    construction: Construction,
    force_closed: bool,
    fitted_top: Option<Rgba<u8>>,
    depth_override: Option<u32>,
) -> PropModel {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let d = depth_override.unwrap_or_else(|| seg.depth_cells_for(w));
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
                // The cylinder archetypes serve ids whose sprites
                // do not segment (open barrels, vats): they fit on
                // the coverage path's synthesized seamless
                // segmentation instead. Every other archetype id
                // segments, so the fallback changes nothing for
                // them (and an id that fits neither still declines).
                let seg = segment_for(id, sprite).or_else(|| seamless_segment(sprite))?;
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
    // (The barrels, trash can, keg and vats left this list for
    // the HollowCylinder / ClosedCylinder archetypes — an open
    // or lidded container needs a real cavity/lid, not a capped
    // column extrusion; see archetypes.rs.)
    "f_water_heater",
    "f_logstool",
    "f_punching_bag",
    "f_sand_castle",
    // (Boulders and the fungal clump moved to LATHE_PINS: their
    // profiles vary per row, which a constant-diameter cylinder
    // cannot follow.)
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

/// Irregular round shapes: bodies of revolution whose radius
/// VARIES with height — a twisted alien gasper, a resin pod, egg
/// sacks, beehive kilns, boulders, a fungal mass. The seamless
/// solid gave them a flat fitted back and flat striped sides
/// (Joey's review, 2026-11-17, on f_alien_gasper: "the front
/// should match the back; the back should not be flat and the
/// sides should not be flat either"). Each pin was verified
/// against its sprite: the depicted object is round in plan at
/// every height, with a per-height radius.
pub const LATHE_PINS: &[&str] = &[
    "f_alien_gasper",
    "f_alien_pod_resin",
    "f_clay_kiln",
    "f_kiln_empty",
    "f_kiln_full",
];

/// Irregular lump pins: boulders and the fungal clump
/// (Joey's reviews, 2026-11-17: "Literally a boulder" — the
/// lathe made them surfaces of revolution, i.e. cylinders with
/// a profile; these objects are NOT round in plan, they are
/// irregular masses). Routed to [`blob_model`].
pub const BLOB_PINS: &[&str] = &[
    "f_boulder_large",
    "f_boulder_medium",
    "f_boulder_small",
    "f_glow_boulder",
    "f_fungal_clump",
];

/// Egg-pile pins: the egg sacks are mounds of individual
/// eggs (Joey, 2026-11-17: "These are mounds of things and
/// eggs. The back shouldn't be flat"). Routed to
/// [`egg_pile_model`]: a blob mass whose back and sides are
/// formed by packed egg spheres, so every non-front face
/// reads as individual round eggs. The front stays the
/// sprite verbatim (Joey approved the fronts).
/// Mockup: target/stage-g/mockups/f_egg_sacke_sides.png.
pub const EGG_PINS: &[&str] = &["f_egg_sacke", "f_egg_sackws"];

/// An irregular lump fitted to the sprite: the front face is
/// the sprite verbatim (every structure pixel is a column at
/// the front plane), and each column's DEPTH is fitted from
/// the silhouette itself — an elliptical falloff over the
/// pixel's position in its row and its column (edge pixels are
/// shallow, central pixels deep) modulated by low-frequency
/// value noise, so the back and side silhouettes are lumpy and
/// asymmetric, never a disc profile. Every voxel carries its
/// column's sprite paint (mottled rock/fungal texture wraps
/// the mass). Depth budget = the widest row (<= 32).
fn blob_base(sprite: &RgbaImage, seg: &Segmentation) -> Option<PropModel> {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let bot = front.end - 1;
    // Per-row and per-column structure extents.
    let mut row_ext: Vec<Option<(u32, u32)>> = vec![None; fh as usize];
    let mut col_ext: Vec<Option<(u32, u32)>> = vec![None; w as usize];
    let mut d_max = 0u32;
    for y in 0..fh {
        let r = bot - y;
        for x in 0..w {
            if structure(*sprite.get_pixel(x, r)) {
                row_ext[y as usize] = Some(match row_ext[y as usize] {
                    None => (x, x),
                    Some((a, b)) => (a.min(x), b.max(x)),
                });
                col_ext[x as usize] = Some(match col_ext[x as usize] {
                    None => (y, y),
                    Some((a, b)) => (a.min(y), b.max(y)),
                });
            }
        }
        if let Some((a, b)) = row_ext[y as usize] {
            d_max = d_max.max(b - a + 1);
        }
    }
    if d_max < 4 {
        return None;
    }
    let d = d_max.min(32);
    // Depth from the silhouette's DISTANCE TRANSFORM (mockup
    // round, 2026-11-17): a column's depth follows how deep
    // inside the silhouette it sits — the mass is an inflated
    // ("puffy") version of the 2D shape, so the back relief
    // and the side profile are smooth lumps. The first blob
    // version's per-column taper x noise left a spiky relief
    // (the back read as a heart/dart with drips) — the
    // distance field cannot spike: it is smooth by
    // construction. A low-frequency lump field keeps it
    // irregular.
    let dist = {
        let mut dm = vec![0u32; (w * fh) as usize];
        let mut maxd = 0u32;
        for y in 0..fh {
            for x in 0..w {
                if !structure(*sprite.get_pixel(x, bot - y)) {
                    continue;
                }
                // Brute-force distance to the nearest
                // non-structure cell (grid is at most 32x64).
                let mut best = u32::MAX;
                for yy in 0..fh {
                    for xx in 0..w {
                        if !structure(*sprite.get_pixel(xx, bot - yy)) {
                            best = best.min(x.abs_diff(xx) + y.abs_diff(yy));
                        }
                    }
                }
                dm[(y * w + x) as usize] = best;
                maxd = maxd.max(best);
            }
        }
        (dm, maxd.max(1))
    };
    let noise = crate::noise::Fbm::new(0xB10B, 8).octaves(2);
    let mut m = PropModel::new([w, fh, d]);
    for y in 0..fh {
        let Some((rx0, rx1)) = row_ext[y as usize] else {
            continue;
        };
        let r = bot - y;
        let rcx = (rx0 + rx1) as f32 / 2.0;
        let rhalf = ((rx1 - rx0 + 1) as f32 / 2.0).max(1.0);
        for x in rx0..=rx1 {
            let mut px = *sprite.get_pixel(x, r);
            if !structure(px) {
                continue;
            }
            // De-void (as column_paint): a pure-black pixel is
            // outline — the column takes its nearest non-black
            // paint instead of extruding a black streak.
            if px[0] == 0 && px[1] == 0 && px[2] == 0 {
                let (cy0, cy1) = col_ext[x as usize].unwrap_or((y, y));
                for dist in 1..=(cy1 - cy0) {
                    let mut found = None;
                    for yy in [y.checked_add(dist), y.checked_sub(dist)]
                        .into_iter()
                        .flatten()
                    {
                        if yy >= cy0 && yy <= cy1 {
                            let c = *sprite.get_pixel(x, bot - yy);
                            if structure(c) && !(c[0] == 0 && c[1] == 0 && c[2] == 0) {
                                found = Some(c);
                                break;
                            }
                        }
                    }
                    if let Some(c) = found {
                        px = c;
                        break;
                    }
                }
            }
            let dfrac = dist.0[(y * w + x) as usize] as f32 / dist.1 as f32;
            let lump = 0.85 + 0.30 * noise.sample(x as f32 * 0.09, y as f32 * 0.09);
            // Dome toward the column's top (the top rows must
            // not slab backwards at full depth) and a ground
            // boost at the base (the back face reaches the
            // ground across the width — a grounded mass, per
            // the mockup's side profile).
            let (cy0, cy1) = col_ext[x as usize].unwrap_or((y, y));
            let v = (y - cy0) as f32 / ((cy1 - cy0) as f32).max(1.0);
            let dome = 1.0 - 0.38 * v * v;
            let mut depth = d as f32 * dfrac.powf(0.65) * lump * dome;
            let gfrac = ((0.4 * fh as f32 - y as f32) / (0.4 * fh as f32)).clamp(0.0, 1.0);
            depth = depth.max(d as f32 * 0.45 * gfrac);
            let depth = (depth.round() as u32).clamp(2, d);
            for z in (d - depth)..d {
                m.set(x, y, z, Some(px));
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// The blob mass with the pattern wrap (boulders, fungal
/// clump).
fn blob_model(sprite: &RgbaImage, seg: &Segmentation) -> Option<PropModel> {
    let mut m = blob_base(sprite, seg)?;
    let fh = seg.front_rows.end - seg.front_rows.start;
    let bot = seg.front_rows.end - 1;
    let rows: Vec<Option<(u32, u32)>> = (0..fh)
        .map(|y| {
            let r = bot - y;
            let xs: Vec<u32> = (0..sprite.width())
                .filter(|&x| structure(*sprite.get_pixel(x, r)))
                .collect();
            if xs.is_empty() {
                None
            } else {
                Some((xs[0], xs[xs.len() - 1]))
            }
        })
        .collect();
    wrap_row_pattern(&mut m, sprite, seg, &rows);
    Some(m)
}

/// Deterministic 2D hash -> [0, 1) (xorshift-multiply), for
/// the egg scatter (rebuilds must be identical).
fn hash01(a: u32, b: u32) -> f32 {
    let mut h = (a.wrapping_mul(0x9E3779B1) ^ b.wrapping_mul(0x85EBCA6B)) as u64;
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58476D1CE4E5B9);
    h ^= h >> 32;
    (h % 10000) as f32 / 10000.0
}

/// A mound of individual eggs (the egg sacks): the blob
/// mass, with packed egg spheres seated on its back surface
/// so the back and sides are formed by round egg caps —
/// individually tinted — instead of a smooth relief. Egg
/// discs may bulge past the silhouette only in the back
/// half (z <= d-3), so the front face and its silhouette
/// stay exactly the sprite.
fn egg_pile_model(sprite: &RgbaImage, seg: &Segmentation) -> Option<PropModel> {
    let mut m = blob_base(sprite, seg)?;
    let (w, fh, d) = (sprite.width(), m.size[1], m.size[2]);
    let bot = seg.front_rows.end - 1;
    // Back endpoint (z-min) of the base mass per column.
    let back_z = |m: &PropModel, x: u32, y: u32| -> Option<u32> {
        (0..d).find(|&z| m.get(x, y, z).is_some())
    };
    // Silhouette dilated by 2 (where egg bulges may live).
    let mut dilated = vec![false; (w * fh) as usize];
    for y in 0..fh {
        for x in 0..w {
            if structure(*sprite.get_pixel(x, bot - y)) {
                for dy in -2i32..=2 {
                    for dx in -2i32..=2 {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < fh as i32 {
                            dilated[(ny * w as i32 + nx) as usize] = true;
                        }
                    }
                }
            }
        }
    }
    // Egg seats: a jittered grid over the silhouette.
    let mut eggs: Vec<(i32, i32, i32, f32, f32)> = Vec::new(); // x, y, z, r, tint
    let mut gy = 1i32;
    while gy < fh as i32 - 1 {
        let mut gx = 2i32;
        while gx < w as i32 - 1 {
            let cx = (gx + ((hash01(gx as u32, gy as u32) - 0.5) * 2.4) as i32)
                .clamp(0, w as i32 - 1);
            let cy = (gy + ((hash01(gy as u32, gx as u32 ^ 0x5EED) - 0.5) * 2.4) as i32)
                .clamp(0, fh as i32 - 1);
            if let Some(bz) = back_z(&m, cx as u32, cy as u32) {
                let r = 2.1 + hash01(cx as u32 ^ 0xE66, cy as u32) * 1.1;
                let cz = bz as i32 + (r * 0.72) as i32;
                let tint = 0.86 + hash01(cx as u32, cy as u32 ^ 0x7177) * 0.26;
                eggs.push((cx, cy, cz, r, tint));
            }
            gx += 4;
        }
        gy += 3;
    }
    // Side seats: eggs along both side walls (Joey,
    // 2026-11-17: in the side views one side was flat —
    // the back-seated eggs never reached the flanks, so
    // the front half of each side stayed the base mass's
    // flat extruded wall).
    for y in (1..fh.saturating_sub(1)).step_by(3) {
        let xs: Vec<u32> = (0..w)
            .filter(|&x| structure(*sprite.get_pixel(x, bot - y)))
            .collect();
        if xs.is_empty() {
            continue;
        }
        for xe in [xs[0], xs[xs.len() - 1]] {
            let Some(bz) = back_z(&m, xe, y) else { continue };
            let mut zc = bz as i32 + 1;
            while zc <= d as i32 - 3 {
                let r = 2.1 + hash01(xe ^ 0x51DE, (zc as u32) ^ y) * 1.1;
                let tint = 0.86 + hash01((zc as u32) ^ 0x51D3, xe ^ y) * 0.26;
                eggs.push((xe as i32, y as i32, zc, r, tint));
                zc += 4;
            }
        }
    }
    // Lay the eggs: sphere voxels, tinted per egg.
    for &(ex, ey, ez, r, tint) in &eggs {
        let ri = r.ceil() as i32;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                for dz in -ri..=ri {
                    let dist2 = (dx * dx + dy * dy + dz * dz) as f32;
                    if dist2 > r * r {
                        continue;
                    }
                    let (x, y, z) = (ex + dx, ey + dy, ez + dz);
                    if x < 0 || y < 0 || z < 0 || x >= w as i32 || y >= fh as i32 || z >= d as i32 {
                        continue;
                    }
                    let (x, y, z) = (x as u32, y as u32, z as u32);
                    if z == d - 1 {
                        continue; // the front face stays the sprite
                    }
                    let in_sil = structure(*sprite.get_pixel(x, bot - y));
                    if !in_sil && !(z <= d.saturating_sub(2) && dilated[(y * w + x) as usize]) {
                        continue;
                    }
                    // Paint: the egg seat's column paint (or
                    // the voxel's own column when it has one),
                    // tinted per egg.
                    let src = if in_sil {
                        *sprite.get_pixel(x, bot - y)
                    } else {
                        *sprite.get_pixel(ex as u32, bot - ey as u32)
                    };
                    let t = tint * (1.0 - 0.25 * (dist2.sqrt() / r));
                    let c = Rgba([
                        (src[0] as f32 * t).min(255.0) as u8,
                        (src[1] as f32 * t).min(255.0) as u8,
                        (src[2] as f32 * t).min(255.0) as u8,
                        255,
                    ]);
                    m.set(x, y, z, Some(c));
                }
            }
        }
    }
    Some(m)
}

/// A body of revolution fitted to the sprite's profile: at
/// every height, the row's structure extent is that height's
/// diameter, and the cross-section is the disc of that diameter
/// centered on the row's own center (so a leaning or twisting
/// profile is followed, not straightened). Every voxel carries
/// its column's sprite paint at that height — the front-most
/// AND back-most voxel of each column are both the sprite pixel,
/// so the back matches the front, and the side silhouette is
/// the depth profile (the same profile as the front), never a
/// flat panel. Model depth = the widest row's width (<= 32).
fn lathe_model(sprite: &RgbaImage, seg: &Segmentation) -> Option<PropModel> {
    let w = sprite.width();
    let front = &seg.front_rows;
    let fh = front.end - front.start;
    let bot = front.end - 1;
    let mut rows: Vec<Option<(u32, u32)>> = Vec::with_capacity(fh as usize);
    let mut d_max = 0u32;
    for y in 0..fh {
        let r = bot - y;
        let mut ext: Option<(u32, u32)> = None;
        for x in 0..w {
            if structure(*sprite.get_pixel(x, r)) {
                ext = Some(match ext {
                    None => (x, x),
                    Some((a, b)) => (a.min(x), b.max(x)),
                });
            }
        }
        if let Some((a, b)) = ext {
            d_max = d_max.max(b - a + 1);
        }
        rows.push(ext);
    }
    if d_max < 4 {
        return None;
    }
    let d = d_max.min(32);
    let cz = (d - 1) as f32 / 2.0;
    // Hollow top (the gasper's mouth): if the sprite's top rows
    // are wide (a flare, not a shrinking dome) and their inner
    // pixels are majority dark-classified, those rows are an
    // OPENING — they become rings over a dark interior surface
    // instead of a solid disc whose top face was the far rim's
    // pale paint (a flat slab artifact in review).
    let split = {
        let mut ls: Vec<f32> = Vec::new();
        for y in 0..fh {
            let r = bot - y;
            if let Some((a, b)) = rows[y as usize] {
                for x in a + 2..b.saturating_sub(1) {
                    let p = *sprite.get_pixel(x, r);
                    if structure(p) {
                        ls.push(lab_of(p).l);
                    }
                }
            }
        }
        two_means_split(&ls)
    };
    let inner_dark_majority = |y: u32| -> bool {
        let r = bot - y;
        let Some((a, b)) = rows[y as usize] else {
            return false;
        };
        let inner: Vec<Rgba<u8>> = (a + 2..b.saturating_sub(1))
            .filter(|&x| structure(*sprite.get_pixel(x, r)))
            .map(|x| *sprite.get_pixel(x, r))
            .collect();
        !inner.is_empty()
            && inner.iter().filter(|p| lab_of(**p).l < split).count() * 2 > inner.len()
    };
    let mut mouth_rows = 0u32;
    for y in (0..fh).rev() {
        let wide = rows[y as usize].map(|(a, b)| (b - a + 1) as f32 >= 0.6 * d_max as f32);
        if wide == Some(true) && inner_dark_majority(y) {
            mouth_rows += 1;
        } else {
            break;
        }
    }
    let mouth = if mouth_rows >= 2 {
        let dark: Vec<Rgba<u8>> = ((fh - mouth_rows)..fh)
            .flat_map(|y| {
                let r = bot - y;
                let (a, b) = rows[y as usize].unwrap();
                (a + 2..b.saturating_sub(1))
                    .filter(move |&x| structure(*sprite.get_pixel(x, r)))
                    .map(move |x| *sprite.get_pixel(x, r))
            })
            .filter(|p| lab_of(*p).l < split)
            .collect();
        modal_color(dark.into_iter()).map(|c| (mouth_rows, c))
    } else {
        None
    };
    let mut m = PropModel::new([w, fh, d]);
    for y in 0..fh {
        let Some((rx0, rx1)) = rows[y as usize] else {
            continue;
        };
        let r = bot - y;
        let wr = rx1 - rx0 + 1;
        let cxr = (rx0 + rx1) as f32 / 2.0;
        let (rxr, rzr) = (wr as f32 / 2.0, wr.min(d) as f32 / 2.0);
        let inner_rn = 1.0 - 4.0 / wr as f32;
        let in_mouth = mouth.map(|(mr, _)| y >= fh - mr).unwrap_or(false);
        let mouth_floor = mouth.map(|(mr, _)| y + 1 == fh - mr).unwrap_or(false);
        // The mouth's rim is one continuous lip: ring voxels of
        // a mouth row take the row's modal NON-dark paint, not
        // the per-column pixels — the center columns' dark
        // interior pixels otherwise stripe the rim (the
        // "dashed crown" artifact of Joey's gasper review,
        // 2026-11-17). The interior those dark pixels depict
        // is the cavity + surface, already modeled.
        let rim_color = if in_mouth {
            modal_color(
                (rx0..=rx1)
                    .map(|x| *sprite.get_pixel(x, r))
                    .filter(|p| structure(*p) && lab_of(*p).l >= split),
            )
        } else {
            None
        };
        for x in rx0..=rx1 {
            let px = row_paint(sprite, r, x, rx0, rx1);
            let dx = (x as f32 - cxr) / rxr;
            for z in 0..d {
                let dz = (z as f32 - cz) / rzr;
                let rn2 = dx * dx + dz * dz;
                if rn2 > 1.0 {
                    continue;
                }
                if in_mouth && rn2.sqrt() <= inner_rn {
                    continue; // the opening: genuinely hollow
                }
                let c = if mouth_floor && rn2.sqrt() <= inner_rn {
                    mouth.unwrap().1
                } else if in_mouth {
                    rim_color.unwrap_or(px)
                } else {
                    px
                };
                m.set(x, y, z, Some(c));
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    // The back face must not inherit FRONT features: the
    // kiln's firebox is drawn near-black, and the lathe's
    // back-most voxel is the column's sprite pixel — so the
    // kiln's back grew a black firebox arch (mockup round,
    // 2026-11-17). A back-most voxel whose sprite pixel is
    // near-black feature paint takes the row's nearest
    // non-black structure paint instead. The front keeps the
    // feature (it IS the opening).
    for y in 0..fh {
        let r = bot - y;
        let Some((rx0, rx1)) = rows[y as usize] else {
            continue;
        };
        for x in rx0..=rx1 {
            let px = *sprite.get_pixel(x, r);
            if !structure(px) || lab_of(px).l >= 0.10 {
                continue;
            }
            let Some(zb) = (0..d).find(|&z| m.get(x, y, z).is_some()) else {
                continue;
            };
            for dist in 1..=(rx1 - rx0) {
                let mut done = false;
                for cand in [x.checked_sub(dist), x.checked_add(dist)]
                    .into_iter()
                    .flatten()
                {
                    if (rx0..=rx1).contains(&cand) {
                        let c = *sprite.get_pixel(cand, r);
                        if structure(c) && lab_of(c).l >= 0.10 {
                            m.set(x, y, zb, Some(c));
                            done = true;
                            break;
                        }
                    }
                }
                if done {
                    break;
                }
            }
        }
    }
    wrap_row_pattern(&mut m, sprite, seg, &rows);
    Some(m)
}

/// Side faces: wrap the front pattern around (the kiln's
/// cobbles must show on all sides — Joey, 2026-11-17; the
/// same wrap gives the blob's sides its rock texture instead
/// of one column pixel smeared through the depth). The
/// pattern coordinate across a side face is the depth,
/// mapped onto the sprite row's extent.
fn wrap_row_pattern(
    m: &mut PropModel,
    sprite: &RgbaImage,
    seg: &Segmentation,
    rows: &[Option<(u32, u32)>],
) {
    let (fh, d) = (m.size[1], m.size[2]);
    let bot = seg.front_rows.end - 1;
    let zext: Vec<Option<(u32, u32)>> = (0..fh)
        .map(|y| {
            let mut r: Option<(u32, u32)> = None;
            for z in 0..d {
                if (0..m.size[0]).any(|x| m.get(x, y, z).is_some()) {
                    r = Some(match r {
                        None => (z, z),
                        Some((a, b)) => (a.min(z), b.max(z)),
                    });
                }
            }
            r
        })
        .collect();
    // The pattern column wobbles smoothly as it wraps
    // (mockup round, 2026-11-17): a straight linear map
    // carries a wide dark patch on the gasper's rows around
    // the side as a hard vertical stripe; a low-frequency
    // wobble breaks the stripes into organic blotches while
    // leaving fine texture (cobbles, rock) essentially
    // unshifted in character.
    let wobble = crate::noise::Fbm::new(0xB0B1E, 8).octaves(2);
    wrap_side_faces(m, |y, z| {
        let (zlo, zhi) = zext[y as usize]?;
        if zhi == zlo {
            return None;
        }
        // The ROW wobbles too: a column-only wobble cannot
        // break a marking that runs vertically through the
        // sprite's own rows (the gasper's streaks) — sampling
        // a few rows up or down, smoothly varying with depth,
        // interrupts the streak into blotches.
        let rwob = (wobble.sample(z as f32 * 0.13 + 40.0, y as f32 * 0.13) - 0.5) * 8.0;
        let yy = (y as i32 + rwob.round() as i32).clamp(0, fh as i32 - 1) as u32;
        let (rx0, rx1) = rows[yy as usize]?;
        let r = bot - yy;
        let base = rx0 + (z - zlo) * (rx1 - rx0) / (zhi - zlo);
        let wob = (wobble.sample(z as f32 * 0.16, y as f32 * 0.16) - 0.5) * 7.0;
        let col = (base as i32 + wob.round() as i32).clamp(rx0 as i32, rx1 as i32) as u32;
        let c = row_paint(sprite, r, col, rx0, rx1);
        // Pure black is a front-face feature (a firebox
        // opening) or outline — never wrap it onto a side.
        if c[0] == 0 && c[1] == 0 && c[2] == 0 {
            return None;
        }
        Some(c)
    });
}

/// Repaint a model's side faces with the front pattern
/// wrapped around (Joey's reviews, 2026-11-17: the kiln's
/// "cobblestone pattern should show on all sides"; the fire
/// barrel's "pattern on the front should extend to the back").
/// For every (y, z) row, the side-most solid voxels (min/max x)
/// that are NOT the front-most or back-most voxel of their
/// (x, y) column take `paint(y, z)` — the front/back faces keep
/// their exact paint, and the pattern runs continuously
/// front -> side -> back. Builders supply the paint lookup
/// (the sprite pattern mapped by depth: at the front corner
/// the lookup returns the pattern's edge, at the back corner
/// its opposite edge).
pub fn wrap_side_faces(m: &mut PropModel, paint: impl Fn(u32, u32) -> Option<Rgba<u8>>) {
    let [sx, sy, sz] = m.size;
    // Column z-extents (front-most / back-most solid per (x, y)).
    let mut col_range: Vec<Option<(u32, u32)>> = vec![None; (sx * sy) as usize];
    for x in 0..sx {
        for y in 0..sy {
            let mut r: Option<(u32, u32)> = None;
            for z in 0..sz {
                if m.get(x, y, z).is_some() {
                    r = Some(match r {
                        None => (z, z),
                        Some((a, b)) => (a.min(z), b.max(z)),
                    });
                }
            }
            col_range[(y * sx + x) as usize] = r;
        }
    }
    for y in 0..sy {
        for z in 0..sz {
            let xs: Vec<u32> = (0..sx).filter(|&x| m.get(x, y, z).is_some()).collect();
            if xs.is_empty() {
                continue;
            }
            for &x in [xs[0], xs[xs.len() - 1]].iter() {
                let Some((zlo, zhi)) = col_range[(y * sx + x) as usize] else {
                    continue;
                };
                if z == zlo || z == zhi {
                    continue; // front/back face voxel: exact paint stays
                }
                if let Some(c) = paint(y, z) {
                    m.set(x, y, z, Some(c));
                }
            }
        }
    }
}

/// Two-means lightness split of a sample set (the dark-cluster
/// boundary), midpoint of the two cluster means after 16
/// assignment iterations. 1.0 (nothing is dark) for an empty or
/// single-valued sample.
fn two_means_split(ls: &[f32]) -> f32 {
    if ls.is_empty() {
        return 1.0;
    }
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for l in ls {
        lo = lo.min(*l);
        hi = hi.max(*l);
    }
    if (hi - lo).abs() < f32::EPSILON {
        return 1.0;
    }
    for _ in 0..16 {
        let (mut sl, mut sh, mut nl, mut nh) = (0f32, 0f32, 0u32, 0u32);
        for l in ls {
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
}

/// The paint for column `x` of sprite row `r`: the pixel itself
/// when it is structure, else the nearest structure pixel of
/// the same row (an interior gap in the silhouette keeps the
/// row's own paint family instead of dropping the voxel).
fn row_paint(sprite: &RgbaImage, r: u32, x: u32, rx0: u32, rx1: u32) -> Rgba<u8> {
    let px = *sprite.get_pixel(x, r);
    if structure(px) {
        return px;
    }
    for dist in 1..=(rx1 - rx0) {
        for cand in [x.checked_sub(dist), x.checked_add(dist)]
            .into_iter()
            .flatten()
        {
            if (rx0..=rx1).contains(&cand) {
                let c = *sprite.get_pixel(cand, r);
                if structure(c) {
                    return c;
                }
            }
        }
    }
    px
}

/// The coverage-drive fallback (see the section header):
/// reached only when [`segment_for`] declines and the id is not
/// view-pinned. Pin lists first (plan, then panel), then the
/// seamless solid via [`seamless_segment`] +
/// [`solid_model_for`], minus [`SEAMLESS_EXCLUSIONS`].
/// Round-plan pins ([`CYLINDER_PINS`]) route to
/// [`cylinder_model`] instead of the rectangular solid.
/// Rest-of-props pins (Joey, 2026-11-17: model the
/// remaining 129). These ids fail the seamless criteria
/// (thin, sparse, or low-fill sprites), so they route
/// through [`raw_segment`] — the opaque band with no
/// criteria — into the established builders. Case-by-case
/// pins only, each reviewed from all angles before
/// landing (plan.md standing workflow).
pub const REST_SLAB_PINS: &[(&str, u32)] = &[
    // A ritual circle is a flat ground marking; the sprite
    // is its perspective drawing, used as the plan.
    ("f_magic_circle", 1),
    // f_absence is a shadow on the ground.
    ("f_absence", 1),
];

/// Egg sacks bw/cs: the same draped egg piles as e/ws
/// (their sprites are flatter, failing the deep-fraction
/// criterion the e/ws pair passes).
pub const REST_EGG_PINS: &[&str] = &["f_egg_sackbw", "f_egg_sackcs"];

/// Lathe fits on a raw segment: the birdbath is a basin on
/// a pedestal (rotationally symmetric); the alien pods are
/// small bulbs on stems.
/// Lathe fits on a raw segment: the alien pods are small
/// bulbs on stems; the orrery is a brass instrument around
/// a central axis — its arms sweep a disc, so discs around
/// the axis are the faithful read (a solid extrusion made
/// its sides flat gold planks, Joey's review 2026-11-17).
pub const REST_LATHE_PINS: &[&str] = &["f_alien_pod", "f_alien_pod_organ", "f_roof_turbine_vent"];

/// Closed solids on a raw segment with a fitted depth
/// override (the depth rule misreads both: the anvil's
/// short silhouette gets paper depth 2 although an anvil
/// body runs ~1/3 as deep as long; the orrery's tall
/// silhouette trips the floor-standing floor and gets a
/// plank ~10 deep although the mechanism is narrow).
pub const REST_SOLID_DEPTH_PINS: &[(&str, u32)] = &[("f_anvil", 7)];

/// Frame pins (class D1: poles, signs, lamps, chains,
/// conduits): the sprite is the front view of a thin
/// upright whose members are genuinely thin -- a post,
/// a sign panel, a chain. The build is a structure
/// extrusion at the pinned depth: front and back faces
/// carry the sprite, structure pixels fill between.
/// Depths: 2 for chains/ropes/cables, 3 default, 4 for
/// chunky machines.
/// Member pins (class D1: poles, signs, lamps, chains,
/// conduits): built by member_model -- every sprite row
/// decomposes into contiguous clusters; a cluster up to
/// 6 wide is a MEMBER and gets a round plan
/// cross-section (a pipe, post, or chain run), a wider
/// cluster is a PANEL/BODY and gets a box of real
/// thickness. Joey's group rule (2026-11-17): front and
/// back look good, but NOTHING should be flat -- no
/// planar extrusions.
pub const REST_MEMBER_PINS: &[&str] = &[
    "f_flagpole",
    "f_wooden_flagpole",
    "f_utility_pole",
    "f_exodii_lamp",
    "f_nl_exodii_lamp",
    "f_machinery_light",
    "f_sign_stop",
    "f_sign_warning",
    "f_wind_vane",
    "f_TV_antenna",
    "f_cellphone_booster",
    "f_ventilator",
    "f_battlefield_cross",
    "f_rope_up",
    "f_hoist_chain",
    "f_hoist_rope",
    "f_hoist_pulley",
    "f_hanging_meathook",
    "f_ground_cable",
    "f_lane",
];

/// Stand pins (class D1 fix round): uprights whose
/// sprite carries ROUND parts -- a caster-star base, a
/// lamp shade, a dome head. A flat extrusion keeps the
/// front but the sides collapse to a line (Joey on the
/// IV stand, 2026-11-17: front and back look good, it
/// should not be flat). The value is the head treatment:
/// Cross = the top bar also runs in z (the IV stand's
/// hooks), Discs = the wide top rows are lathe discs
/// (shade / lamp head), None = pole top.
pub const STAND_PINS: &[(&str, u8)] = &[
    ("f_IV_pole", 1),
    ("f_street_light", 2),
    ("f_floor_lamp", 2),
    ("f_dancing_pole", 0),
];

/// Uniform-material pins: the whole model repaints in
/// the sprite's modal structure tone (the anvil's black
/// iron — Joey's instruction, 2026-11-17).
pub const REST_UNIFORM_PINS: &[&str] = &["f_anvil"];

/// Plant pins: fern-like organisms whose fronds spread
/// in depth as well as width (Joey, 2026-11-17, on the
/// zapper's flat extrusion: "It shouldn't be flat. It is
/// a plant"). Routed to [`plant_model`].
pub const REST_PLANT_PINS: &[&str] = &["f_alien_tendril", "f_alien_zapper"];

/// A plant, built as a cross: the sprite silhouette is
/// extruded thin in z (the front slab) AND thin in x
/// along the stalk axis (the side slab, the same
/// silhouette mapped onto the z axis). From the front
/// you see the sprite; from the side you see the same
/// plant profile; in plan the two slabs cross. The
/// standard voxel-plant construction — an earlier
/// per-row z-spread read from the side as a stack of
/// horizontal plates (Joey's zapper review, 2026-11-17:
/// a plant is not flat, and it is not a ribbon either).
fn plant_model(sprite: &RgbaImage) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let fh = band.end - band.start;
    let zc = (w / 2) as i32;
    let mut m = PropModel::new([w, fh, w]);
    // Side slab first (the front slab overwrites the
    // crossing column).
    for y in 0..fh {
        let r = band.end - 1 - y;
        let xs: Vec<u32> = (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            continue;
        }
        let med = xs[xs.len() / 2] as i32;
        for &x in &xs {
            let px = *sprite.get_pixel(x, r);
            for dx in [med - 1, med] {
                if dx >= 0 && (dx as u32) < w {
                    m.set(dx as u32, y, x, Some(px));
                }
            }
        }
    }
    // Front slab: the silhouette extruded 2 deep at the
    // center plane.
    for y in 0..fh {
        let r = band.end - 1 - y;
        for x in 0..w {
            let px = *sprite.get_pixel(x, r);
            if opaque(px) {
                for dz in [zc - 1, zc] {
                    m.set(x, y, dz as u32, Some(px));
                }
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// The birdbath (mockup: target/stage-g/mockups/
/// f_birdbath_sides.png). The sprite is a 3/4 view INTO
/// the basin: the big dark oval is the basin's interior
/// face, not a ball — a lathe of the profile made a
/// lollipop. The model: a shallow basin (4 layers:
/// underside, rim side, dished interior floor, rim lip)
/// on a pedestal of per-row discs from the sprite's stem
/// rows. The sprite's drop shadow (near-black, right of
/// the stem) is excluded from every measurement.
fn birdbath_model(sprite: &RgbaImage) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let solid = |x: u32, y: u32| -> bool {
        let p = *sprite.get_pixel(x, y);
        p[3] >= 128 && !(p[0] < 40 && p[1] < 40 && p[2] < 40)
    };
    let extent = |r: u32| -> Option<(u32, u32)> {
        let xs: Vec<u32> = (0..w).filter(|&x| solid(x, r)).collect();
        if xs.is_empty() {
            None
        } else {
            Some((xs[0], xs[xs.len() - 1]))
        }
    };
    let band = opaque_band(sprite)?;
    let rows: Vec<u32> = band.clone().filter(|&r| extent(r).is_some()).collect();
    let basin_rows: Vec<u32> = rows
        .iter()
        .copied()
        .filter(|&r| {
            let (a, b) = extent(r).unwrap();
            b - a + 1 >= 12
        })
        .collect();
    let last_basin = *basin_rows.last()?;
    let stem_rows: Vec<u32> = rows
        .iter()
        .copied()
        .filter(|&r| r > last_basin)
        .collect();
    if basin_rows.is_empty() || stem_rows.is_empty() {
        return None;
    }
    let (bx0, bx1) = extent(*basin_rows.iter().max_by_key(|&&r| {
        let (a, b) = extent(r).unwrap();
        b - a
    })?)?;
    let radius = (bx1 - bx0 + 1) / 2;
    let cx = (bx0 + bx1) as f32 / 2.0;
    let depth = radius * 2;
    let zc = radius as f32 - 0.5;
    let row_modal = |r: u32| -> Option<Rgba<u8>> {
        modal_color(
            (0..w)
                .filter(|&x| solid(x, r))
                .map(|x| *sprite.get_pixel(x, r)),
        )
    };
    let rows_modal = |rs: &[u32]| -> Option<Rgba<u8>> {
        modal_color(
            rs.iter()
                .flat_map(|&r| (0..w).map(move |x| (x, r)))
                .filter(|&(x, r)| solid(x, r))
                .map(|(x, r)| *sprite.get_pixel(x, r)),
        )
    };
    let rim = rows_modal(&basin_rows[..basin_rows.len().min(4)])?;
    let interior = rows_modal(&basin_rows[basin_rows.len() / 3..(basin_rows.len() * 2 / 3).max(basin_rows.len() / 3 + 1)])?;
    let underside = row_modal(*basin_rows.last()?)?;
    let stem_h = stem_rows.len() as u32;
    let mut m = PropModel::new([w, stem_h + 4, depth]);
    let disc = |m: &mut PropModel, y: u32, r: f32, paint: Rgba<u8>, ring_inner: Option<f32>| {
        for x in 0..w {
            for z in 0..depth {
                let dd = ((x as f32 - cx).powi(2) + (z as f32 - zc).powi(2)).sqrt();
                if dd <= r && ring_inner.map_or(true, |ri| dd > ri) {
                    m.set(x, y, z, Some(paint));
                }
            }
        }
    };
    // Pedestal: per-row discs, bottom row first.
    for (i, &r) in stem_rows.iter().rev().enumerate() {
        let (a, b) = extent(r)?;
        disc(&mut m, i as u32, (b - a) as f32 / 2.0 + 0.5, row_modal(r)?, None);
    }
    // Basin: underside, rim side, interior floor (dished:
    // the inner disc sits at this layer, the rim ring
    // rises one more as the lip).
    disc(&mut m, stem_h, radius as f32 - 2.0, underside, None);
    disc(&mut m, stem_h + 1, radius as f32, rim, None);
    disc(&mut m, stem_h + 2, radius as f32, interior, None);
    disc(&mut m, stem_h + 2, radius as f32, rim, Some(radius as f32 - 3.0));
    disc(&mut m, stem_h + 3, radius as f32, rim, Some(radius as f32 - 2.0));
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// The orrery (mockup: target/stage-g/mockups/
/// f_orrery_sides.png). The sprite is a candelabrum-like
/// mechanism: a flared base and decorated column, then
/// thin rods with colored finial tips rising from an arm
/// assembly. A solid extrusion made the sides gold
/// planks; a lathe made the head a solid trophy drum
/// (both rejected by Joey, 2026-11-17). The build: the
/// lower half (base + column + gear body) is a narrow
/// structure extrusion 4 deep; the head (top half of the
/// band: arms + rods) is a cross like the plants — the
/// rod profile reads from front and side alike.
fn orrery_model(sprite: &RgbaImage) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let fh = band.end - band.start;
    // Rod rows: the top contiguous run of sparse rows
    // (opaque count <= 9 — thin rods and the arm bar),
    // counted down from the band's top. Everything
    // below the first dense row is body: housing, gear
    // body, column, base.
    let mut rod_rows = 0u32;
    for y in (0..fh).rev() {
        let r = band.end - 1 - y;
        let n = (0..w).filter(|&x| opaque(*sprite.get_pixel(x, r))).count();
        if n > 9 {
            break;
        }
        rod_rows += 1;
    }
    let rod_rows = rod_rows.clamp(4, fh.saturating_sub(8).max(4));
    let mut m = PropModel::new([w, fh, w]);
    let zc = (w / 2) as i32;
    for y in 0..fh {
        let r = band.end - 1 - y;
        let xs: Vec<u32> = (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            continue;
        }
        if y >= fh - rod_rows {
            // Rods: cross slabs (front + side), as built
            // before — the rod profile reads from both
            // axes and the finial tips keep their colors.
            let med = xs[xs.len() / 2] as i32;
            for &x in &xs {
                let px = *sprite.get_pixel(x, r);
                for dx in [med - 1, med] {
                    if dx >= 0 && (dx as u32) < w {
                        m.set(dx as u32, y, x, Some(px));
                    }
                }
            }
            for &x in &xs {
                let px = *sprite.get_pixel(x, r);
                for dz in [zc - 1, zc] {
                    m.set(x, y, dz as u32, Some(px));
                }
            }
        } else {
            // Body: a disc per row (the row's extent is
            // the diameter, the row's own center is the
            // axis — the body leans as the sprite leans).
            // Paint wraps the row around the disc: front
            // and back quadrants sample the row by x, the
            // side quadrants by z from the center.
            let (a, b) = (xs[0], xs[xs.len() - 1]);
            let cx = (a + b) as f32 / 2.0;
            let rad = (b - a) as f32 / 2.0 + 0.5;
            // The housing is shallower than it is wide:
            // elliptical discs (z radius 0.7x), or the
            // body reads as a drum. And the sprite's
            // near-black gap pixels (the artist's
            // background showing through the open frame)
            // are not surface paint: de-void them to the
            // row's modal solid tone, or they smear
            // around the disc as black patches.
            let zrad = rad * 0.7;
            let row_modal = modal_color(xs.iter().filter_map(|&x| {
                let px = *sprite.get_pixel(x, r);
                let [pr, pg, pb, _] = px.0;
                if pr < 40 && pg < 40 && pb < 40 {
                    None
                } else {
                    Some(px)
                }
            }));
            for x in 0..w {
                for z in 0..w {
                    let (dx, dz) = (x as f32 - cx, z as f32 - zc as f32);
                    if dx * dx / (rad * rad) + dz * dz / (zrad * zrad) > 1.0 {
                        continue;
                    }
                    let sx = if dz.abs() / zrad >= dx.abs() / rad {
                        x.clamp(a, b)
                    } else {
                        (cx + dz.signum() * (dz.abs() / 0.7))
                            .round()
                            .clamp(a as f32, b as f32) as u32
                    };
                    let mut px = *sprite.get_pixel(sx, r);
                    let [pr, pg, pb, _] = px.0;
                    if pr < 40 && pg < 40 && pb < 40 {
                        if let Some(c) = row_modal {
                            px = c;
                        }
                    }
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

/// A hammock (mockup: target/stage-g/mockups/
/// f_hammock_sides.png). Both hammock sprites are PLAN
/// views: a leaf-shaped sling (pointed gathered ends,
/// wide middle) with gold suspension ropes running off
/// both ends. The model is that plan, hung: the sprite
/// supplies the plan shape and the paint (light rim,
/// dark trough center; the net's checkerboard for
/// f_hammock_net), the sheet sags along its length
/// (ends at rope height 12, middle at 7) and curls up
/// across its width (a trough section, +3 at the
/// edges). The sprite's 48-row length scales onto the
/// tile's 32 in z; the ropes run to the tile edges.
fn hammock_model(sprite: &RgbaImage, net: bool) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let extent = |r: u32| -> Option<(u32, u32)> {
        let xs: Vec<u32> = (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            None
        } else {
            Some((xs[0], xs[xs.len() - 1]))
        }
    };
    // Fabric rows: extent at least 4 wide (the ropes are
    // 2 wide).
    let fabric: Vec<u32> = (0..h)
        .filter(|&r| extent(r).map_or(false, |(a, b)| b - a + 1 >= 4))
        .collect();
    let (f0, f1) = (fabric.first()?, fabric.last()?);
    // Cloth paint is smoothed over the source 3x3 (the
    // sprite's per-pixel weave noise read as lumps on a
    // 1-cell sheet -- Joey: "these look really bad",
    // 2026-11-17). The net keeps its raw checkerboard:
    // the checker IS the pattern.
    let paint = |x: u32, r: u32| -> Rgba<u8> {
        let px = *sprite.get_pixel(x, r);
        if net {
            return px;
        }
        let (mut sr, mut sg, mut sb, mut n) = (0u32, 0u32, 0u32, 0u32);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let (nx, ny) = (x as i32 + dx, r as i32 + dy);
                if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
                    let q = *sprite.get_pixel(nx as u32, ny as u32);
                    if opaque(q) {
                        sr += q.0[0] as u32;
                        sg += q.0[1] as u32;
                        sb += q.0[2] as u32;
                        n += 1;
                    }
                }
            }
        }
        if n == 0 {
            px
        } else {
            Rgba([(sr / n) as u8, (sg / n) as u8, (sb / n) as u8, 255])
        }
    };
    // Heightfield first, then a skirt fill: the deep
    // canoe slopes step by up to 2 between neighbours,
    // and a 1-cell sheet with steps > 1 has see-through
    // gaps at angle views. Each column fills down to its
    // lowest neighbour's surface, so the sheet is
    // continuous from every angle.
    let mut top: std::collections::HashMap<(u32, u32), (u32, Rgba<u8>)> =
        std::collections::HashMap::new();
    for z in 0..32u32 {
        let r = (z * (h - 1)) / 31;
        let (a, b) = extent(r)?;
        if fabric.contains(&r) {
            let t = (r - f0) as f32 / (f1 - f0) as f32;
            // Deep shapes, second pass: the first build's
            // +3 curl and 5 sag read as a flat lumpy
            // plank. A hammock bed is a canoe: center
            // sags 9 below the ends, edges curl 5 up
            // (flattening toward the gathered ends).
            let cy = 14.0 - 9.0 * (std::f32::consts::PI * t).sin();
            let cx = (a + b) as f32 / 2.0;
            let hw = ((b - a) as f32 / 2.0).max(1.0);
            let curl_scale = (((b - a + 1) as f32) / 10.0).min(1.0);
            for x in a..=b {
                let u = ((x as f32 - cx) / hw).abs();
                let curl = 5.0 * u.powf(2.5) * curl_scale;
                let y = (cy + curl).round() as u32;
                top.insert((x, z), (y, paint(x, r)));
            }
        } else {
            // Suspension rope: the row's own pixels at
            // rope height (meets the fabric ends at 14).
            for x in a..=b {
                top.insert((x, z), (14, *sprite.get_pixel(x, r)));
            }
        }
    }
    let mut m = PropModel::new([w, 16, 32]);
    let keys: Vec<(u32, u32)> = top.keys().copied().collect();
    for (x, z) in keys {
        let (y, c) = top[&(x, z)];
        let mut lo = y;
        for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let n = (x as i32 + dx, z as i32 + dz);
            if n.0 >= 0 && n.1 >= 0 {
                if let Some((ny, _)) = top.get(&(n.0 as u32, n.1 as u32)) {
                    lo = lo.min(*ny);
                }
            }
        }
        for yy in lo..=y {
            m.set(x, yy, z, Some(c));
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// A thin upright (poles, signs, chains -- class D1):
/// the sprite's structure extruded `depth` cells at the
/// model's z center; opaque fringe pixels paint the
/// front and back faces only. The band is any-opaque
/// rows (a chain's links and a lamp's finial are sparse
/// rows the seamless band test would trim).
fn member_model(sprite: &RgbaImage) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let fh = band.end - band.start;
    let zc = 16.0f32;
    let mut m = PropModel::new([w, fh, 32]);
    for y in 0..fh {
        let r = band.end - 1 - y;
        // Contiguous opaque clusters in this row.
        let mut clusters: Vec<(u32, u32)> = Vec::new();
        let mut x = 0;
        while x < w {
            if opaque(*sprite.get_pixel(x, r)) {
                let a = x;
                while x < w && opaque(*sprite.get_pixel(x, r)) {
                    x += 1;
                }
                clusters.push((a, x - 1));
            } else {
                x += 1;
            }
        }
        for (a, b) in clusters {
            let wdt = b - a + 1;
            let cx = (a + b) as f32 / 2.0;
            if wdt <= 6 {
                // Member: a disc in plan (a tube). Paint
                // wraps the row by quadrant, like the
                // lathe discs.
                let rad = wdt as f32 / 2.0 + 0.3;
                for vx in 0..w {
                    for vz in 0..32u32 {
                        let (dx, dz) = (vx as f32 - cx, vz as f32 - zc);
                        if dx * dx + dz * dz > rad * rad {
                            continue;
                        }
                        let sx = if dz.abs() >= dx.abs() {
                            vx.clamp(a, b)
                        } else {
                            (cx + dz.signum() * dz.abs())
                                .round()
                                .clamp(a as f32, b as f32) as u32
                        };
                        m.set(vx, y, vz, Some(*sprite.get_pixel(sx, r)));
                    }
                }
            } else {
                // Panel/body: a box of real thickness
                // (width / 3, clamped 3..6). Front and
                // back faces carry the row's paint by x;
                // the end faces carry the edge pixels.
                let d = ((wdt as f32 / 3.0).round() as u32).clamp(3, 6);
                let z0 = 16 - d / 2;
                let z1 = z0 + d - 1;
                for vx in a..=b {
                    for vz in z0..=z1 {
                        let px = if vz == z0 || vz == z1 {
                            *sprite.get_pixel(vx, r)
                        } else if vx == a {
                            *sprite.get_pixel(a, r)
                        } else if vx == b {
                            *sprite.get_pixel(b, r)
                        } else {
                            *sprite.get_pixel(vx, r)
                        };
                        m.set(vx, y, vz, Some(px));
                    }
                }
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// A stand (IV pole, lamps, dancing pole): the pole
/// extrudes as a post, but the flared base rows (bottom
/// rows at least 5 wide, up to 6) become per-row discs
/// -- a caster star or round base reads from every
/// angle -- and the head follows the pin: a cross for
/// the IV stand's hook bar, discs for shades and dome
/// heads (top rows at least 6 wide, up to 7).
fn stand_model(sprite: &RgbaImage, head_mode: u8) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let fh = band.end - band.start;
    let width = |y: u32| -> u32 {
        let r = band.end - 1 - y;
        let xs: Vec<u32> = (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            0
        } else {
            xs[xs.len() - 1] - xs[0] + 1
        }
    };
    let mut base_rows = 0u32;
    while base_rows < 6 && base_rows < fh && width(base_rows) >= 5 {
        base_rows += 1;
    }
    let mut head_rows = 0u32;
    if head_mode > 0 {
        while head_rows < 7 && head_rows + base_rows < fh {
            let y = fh - 1 - head_rows;
            if width(y) >= 6 {
                head_rows += 1;
            } else {
                break;
            }
        }
    }
    let zc = 16i32;
    let mut m = PropModel::new([w, fh, 32]);
    // Disc rows (base, and head when mode 2): per-row
    // discs with the row's paint wrapped by quadrant.
    let disc = |m: &mut PropModel, y: u32| {
        let r = band.end - 1 - y;
        let xs: Vec<u32> = (0..w)
            .filter(|&x| opaque(*sprite.get_pixel(x, r)))
            .collect();
        if xs.is_empty() {
            return;
        }
        let (a, b) = (xs[0], xs[xs.len() - 1]);
        let cx = (a + b) as f32 / 2.0;
        let rad = (b - a) as f32 / 2.0 + 0.5;
        for x in 0..w {
            for z in 0..32u32 {
                let (dx, dz) = (x as f32 - cx, z as f32 - zc as f32);
                if dx * dx + dz * dz > rad * rad {
                    continue;
                }
                let sx = if dz.abs() >= dx.abs() {
                    x.clamp(a, b)
                } else {
                    (cx + dz.signum() * dz.abs()).round().clamp(a as f32, b as f32) as u32
                };
                m.set(x, y, z, Some(*sprite.get_pixel(sx, r)));
            }
        }
    };
    for y in 0..fh {
        let r = band.end - 1 - y;
        if y < base_rows || (head_mode == 2 && y >= fh - head_rows) {
            disc(&mut m, y);
            continue;
        }
        // Post rows: structure extrusion 3 deep.
        for x in 0..w {
            let px = *sprite.get_pixel(x, r);
            if !opaque(px) {
                continue;
            }
            if structure(px) {
                for z in (zc - 1)..=(zc + 1) {
                    m.set(x, y, z as u32, Some(px));
                }
            } else {
                m.set(x, y, (zc - 1) as u32, Some(px));
                m.set(x, y, (zc + 1) as u32, Some(px));
            }
        }
        // IV stand head: the hook bar also runs in z
        // (cross slab over the head rows).
        if head_mode == 1 && y >= fh - head_rows {
            let xs: Vec<u32> = (0..w)
                .filter(|&x| opaque(*sprite.get_pixel(x, r)))
                .collect();
            if !xs.is_empty() {
                let med = xs[xs.len() / 2] as i32;
                for &x in &xs {
                    let px = *sprite.get_pixel(x, r);
                    for dx in [med - 1, med, med + 1] {
                        if dx >= 0 && (dx as u32) < w {
                            m.set(dx as u32, y, x, Some(px));
                        }
                    }
                }
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

/// The small satellite dish. The sprite's rows 8..20
/// are a reflector disc (center ~(18.5, 14), radii
/// ~7 x 6.5) with a feed arm at its right edge; rows
/// 21..24 are the mount post. A flat extrusion made the
/// reflector a plate edge-on; here the disc's pixels
/// sit on a paraboloid (rim forward at z 19, center
/// recessed to z 15, shell 2 thick) and everything else
/// (arm, mount) extrudes flat behind the bowl.
fn dish_model(sprite: &RgbaImage) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let fh = band.end - band.start;
    let mut m = PropModel::new([w, fh, 32]);
    let (cx, cy) = (18.5f32, 14.0f32);
    for y in 0..fh {
        let r = band.end - 1 - y;
        for x in 0..w {
            let px = *sprite.get_pixel(x, r);
            if !opaque(px) {
                continue;
            }
            let e = ((x as f32 - cx) / 7.0).powi(2) + ((r as f32 - cy) / 6.5).powi(2);
            if e <= 1.0 {
                let z = 19 - (4.0 * (1.0 - e)).round() as u32;
                m.set(x, y, z, Some(px));
                m.set(x, y, z - 1, Some(px));
            } else {
                for z in 13..=15 {
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

/// A thin upright as a flat structure extrusion
/// (front/back faces carry the sprite, structure pixels
/// fill between) -- kept for the two conduit bundles,
/// the one flat build Joey approved in the D1 group.
fn frame_model(sprite: &RgbaImage, depth: u32) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let fh = band.end - band.start;
    let zc = 16i32;
    let z0 = zc - (depth as i32) / 2;
    let z1 = z0 + depth as i32 - 1;
    let mut m = PropModel::new([w, fh, 32]);
    for y in 0..fh {
        let r = band.end - 1 - y;
        for x in 0..w {
            let px = *sprite.get_pixel(x, r);
            if !opaque(px) {
                continue;
            }
            if structure(px) {
                for z in z0..=z1 {
                    m.set(x, y, z as u32, Some(px));
                }
            } else {
                m.set(x, y, z0 as u32, Some(px));
                m.set(x, y, z1 as u32, Some(px));
            }
        }
    }
    if m.solid_count() == 0 {
        return None;
    }
    Some(m)
}

// ----------------------------------------------------------------
// Hand-built models (Joey, 2026-11-17: "Edit the models
// manually"). Each is composed from primitives on the
// 32x32 plan, designed from the sprite and the real
// object; colors are the sprites' measured modal tones
// (named per builder). These override every procedural
// route for their ids.
// ----------------------------------------------------------------

fn rgb(r: u8, g: u8, b: u8) -> Rgba<u8> {
    Rgba([r, g, b, 255])
}

fn box_fill(
    m: &mut PropModel,
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
    z0: u32,
    z1: u32,
    c: Rgba<u8>,
) {
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                m.set(x, y, z, Some(c));
            }
        }
    }
}

/// Vertical cylinder (disc stack) of radius r at (cx, cz).
fn cyl_y(m: &mut PropModel, cx: i32, cz: i32, r: f32, y0: u32, y1: u32, c: Rgba<u8>) {
    let ri = r.ceil() as i32;
    for x in (cx - ri)..=(cx + ri) {
        for z in (cz - ri)..=(cz + ri) {
            let (dx, dz) = (x - cx, z - cz);
            if (dx * dx + dz * dz) as f32 <= r * r {
                for y in y0..=y1 {
                    if x >= 0 && z >= 0 && (x as u32) < 32 && (z as u32) < 32 {
                        m.set(x as u32, y, z as u32, Some(c));
                    }
                }
            }
        }
    }
}

/// Horizontal tube along x at (y, cz), radius r.
fn tube_x(m: &mut PropModel, x0: u32, x1: u32, cy: i32, cz: i32, r: f32, c: Rgba<u8>) {
    let ri = r.ceil() as i32;
    for y in (cy - ri)..=(cy + ri) {
        for z in (cz - ri)..=(cz + ri) {
            let (dy, dz) = (y - cy, z - cz);
            if (dy * dy + dz * dz) as f32 <= r * r {
                for x in x0..=x1 {
                    if y >= 0 && z >= 0 {
                        m.set(x, y as u32, z as u32, Some(c));
                    }
                }
            }
        }
    }
}

/// Horizontal tube along z at (cx, y), radius r.
fn tube_z(m: &mut PropModel, z0: u32, z1: u32, cx: i32, cy: i32, r: f32, c: Rgba<u8>) {
    let ri = r.ceil() as i32;
    for x in (cx - ri)..=(cx + ri) {
        for y in (cy - ri)..=(cy + ri) {
            let (dx, dy) = (x - cx, y - cy);
            if (dx * dx + dy * dy) as f32 <= r * r {
                for z in z0..=z1 {
                    if x >= 0 && y >= 0 {
                        m.set(x as u32, y as u32, z, Some(c));
                    }
                }
            }
        }
    }
}

fn manual_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    match id {
        "f_floor_lamp" => Some(manual_floor_lamp()),
        "f_wind_vane" => Some(manual_wind_vane(sprite)),
        "f_TV_antenna" => Some(manual_tv_antenna()),
        "f_cellphone_booster" => Some(manual_cellphone_booster()),
        "f_ventilator" => Some(manual_ventilator()),
        "f_machinery_light" => Some(manual_machinery_light(sprite)),
        "f_ground_cable" => Some(manual_ground_cable()),
        _ => None,
    }
}

/// Floor lamp: round base, slim pole, a cylindrical
/// paper shade with a dark grille cap and the warm bulb
/// showing at its top (sprite tones: shade (183,183,183),
/// base (124,129,109), pole brown).
fn manual_floor_lamp() -> PropModel {
    let mut m = PropModel::new([32, 44, 32]);
    let shade = rgb(196, 196, 190);
    let base = rgb(124, 129, 109);
    let pole = rgb(96, 74, 52);
    cyl_y(&mut m, 16, 16, 4.2, 0, 1, base);
    cyl_y(&mut m, 16, 16, 1.1, 2, 29, pole);
    // Shade: a shell cylinder (hollow core) y 30..41.
    for y in 30..=41 {
        for x in 11..=21 {
            for z in 11..=21 {
                let d = ((x as i32 - 16).pow(2) + (z as i32 - 16).pow(2)) as f32;
                if d <= 20.0 && d >= 9.0 {
                    m.set(x, y, z, Some(shade));
                }
            }
        }
    }
    // Bulb glow disc inside the top, grille cap ring.
    cyl_y(&mut m, 16, 16, 3.0, 40, 40, rgb(232, 196, 92));
    for x in 11..=21 {
        for z in 11..=21 {
            let d = ((x as i32 - 16).pow(2) + (z as i32 - 16).pow(2)) as f32;
            if d <= 20.0 && d >= 12.0 {
                m.set(x, 42, z, Some(rgb(58, 56, 52)));
            }
        }
    }
    m
}

/// Wind vane: post, a directional cross (bars on both
/// axes with end plates), a ball, and an arrow whose
/// tail carries the sprite's rooster figure as a flat
/// plate. Greys from the sprite ((148,148,148) modal).
fn manual_wind_vane(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 20, 32]);
    let metal = rgb(150, 150, 150);
    let dark = rgb(100, 100, 100);
    cyl_y(&mut m, 16, 16, 1.1, 0, 10, dark);
    // Directional cross at y 8: bars along x and z with
    // end plates.
    box_fill(&mut m, 10, 22, 8, 8, 15, 17, metal);
    box_fill(&mut m, 15, 17, 8, 8, 10, 22, metal);
    box_fill(&mut m, 9, 10, 7, 9, 15, 17, metal);
    box_fill(&mut m, 22, 23, 7, 9, 15, 17, metal);
    box_fill(&mut m, 15, 17, 7, 9, 9, 10, metal);
    box_fill(&mut m, 15, 17, 7, 9, 22, 23, metal);
    // Ball at y 11..12.
    box_fill(&mut m, 15, 17, 11, 12, 15, 17, metal);
    // Arrow at y 14 along x: shaft, head at +x, tail fin
    // at -x.
    box_fill(&mut m, 8, 24, 14, 14, 15, 17, metal);
    box_fill(&mut m, 25, 27, 13, 15, 15, 17, metal);
    box_fill(&mut m, 7, 9, 12, 16, 15, 17, dark);
    // Rooster plate: the sprite's top figure (rows
    // 4..12, cols 11..17 of the 32x32 sprite), stamped
    // 2 thick at the tail, standing on the arrow.
    for r in 4..12u32 {
        for x in 11..17u32 {
            let px = *sprite.get_pixel(x, r);
            if opaque(px) {
                let y = 15 + (11 - r);
                m.set(x + 1, y, 15, Some(px));
                m.set(x + 1, y, 16, Some(px));
            }
        }
    }
    m
}

/// TV antenna (Yagi-Uda): a mast, a boom along z, and
/// seven element rods along x tapering toward the
/// front. Grey (102,102,102) modal.
fn manual_tv_antenna() -> PropModel {
    let mut m = PropModel::new([32, 20, 32]);
    let metal = rgb(120, 120, 120);
    let dark = rgb(70, 70, 70);
    cyl_y(&mut m, 16, 16, 1.1, 0, 11, dark);
    box_fill(&mut m, 14, 18, 11, 12, 14, 18, dark);
    tube_z(&mut m, 5, 27, 16, 13, 0.9, metal);
    // Elements: z from back (long) to front (short).
    let elems = [
        (6u32, 21u32),
        (9, 20),
        (12, 18),
        (15, 17),
        (18, 15),
        (21, 14),
        (24, 12),
    ];
    for (z, half) in elems {
        let x0 = 16 - half / 2;
        tube_x(&mut m, x0, x0 + half, 13, z as i32, 0.7, metal);
    }
    m
}

/// Cellphone booster: two vertical panels (a tall main
/// panel forward-right, a smaller one back-left, as the
/// sprite composes them) joined by two bracket arms.
/// Whites/greys from the sprite ((255,255,255),
/// (155,155,155)).
/// Cellphone booster, hand-built to the tile's
/// measured layout (Joey: the section-solid version
/// looked worse than this composition): the main panel
/// (6 wide, x 17..22, grey top section over a white
/// face), the thin second panel (x 6..8), a diagonal
/// brace lattice between them, a base bar, and the
/// stand foot the sprite's lower diagonal runs to.
fn manual_cellphone_booster() -> PropModel {
    let mut m = PropModel::new([32, 30, 32]);
    let white = rgb(233, 233, 230);
    let grey = rgb(158, 158, 158);
    let back = rgb(128, 128, 128);
    let dark = rgb(88, 88, 88);
    // Depth layout: the main panel stands forward
    // (z 17..19), the second panel and the lattice sit
    // behind it (z 10..13) -- the front view is the
    // tile's composition unchanged, but the side
    // profile shows the assembly's structure (panel
    // edge, brace plane, panel edge) instead of one
    // featureless line.
    // Main panel: y 3..29; grey top (the sprite's
    // upper section), white below.
    box_fill(&mut m, 17, 22, 22, 29, 17, 19, grey);
    box_fill(&mut m, 17, 22, 3, 21, 17, 19, white);
    box_fill(&mut m, 17, 22, 3, 29, 16, 16, back);
    // Second panel: x 6..8, y 7..25, z 11..12.
    box_fill(&mut m, 6, 8, 7, 25, 11, 12, white);
    box_fill(&mut m, 6, 8, 7, 25, 10, 10, back);
    // Diagonal lattice between the panels (z 11..12):
    // arm A descends from the left panel's top, arm B
    // ascends from its base -- the sprite's X brace.
    for i in 0..=7u32 {
        box_fill(&mut m, 9 + i, 10 + i, 20 - i, 21 - i, 11, 12, grey);
        box_fill(&mut m, 9 + i, 10 + i, 11 + i, 12 + i, 11, 12, grey);
    }
    // Two standoff arms reach forward from the lattice
    // plane to the main panel's back.
    box_fill(&mut m, 15, 17, 19, 20, 11, 16, dark);
    box_fill(&mut m, 15, 17, 10, 11, 11, 16, dark);
    // Base bar (the sprite's wide lower rows) ties
    // both depth planes; the stand post and diagonal
    // leg run to the foot.
    box_fill(&mut m, 6, 22, 6, 7, 10, 19, dark);
    box_fill(&mut m, 10, 11, 2, 6, 11, 12, dark);
    for i in 0..=5u32 {
        box_fill(&mut m, 11 + i, 12 + i, 5 - i, 6 - i, 11, 18, dark);
    }
    box_fill(&mut m, 9, 13, 0, 1, 11, 18, dark);
    m
}

/// Machinery light, built from the approved
/// all-views mockup (Joey, 2026-11-17, after a
/// pixel-level study of the sprite): a twin-lamp head
/// on three posts (brown column left, two steel rods)
/// over a bell housing and a solid body; the body
/// raised on four legs with open space beneath,
/// standing in a shallow floor pan. Front faces are
/// stamped with the sprite's own pixels; back and
/// sides carry the mockup's surfaces (plain head
/// back + conduit, seamed body access panel with
/// vents, lamp pods in profile, plain flanks).

/// A stepped beam of 2x2 cells from (x0, y0, z0) to
/// (x1, y1, z1): the D2 frames' slanted legs, arms
/// and braces.
#[allow(clippy::too_many_arguments)]
fn beam(
    m: &mut PropModel,
    x0: i32,
    y0: i32,
    z0: i32,
    x1: i32,
    y1: i32,
    z1: i32,
    color: Rgba<u8>,
) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max((z1 - z0).abs()).max(1);
    for i in 0..=steps {
        let x = x0 + (x1 - x0) * i / steps;
        let y = y0 + (y1 - y0) * i / steps;
        let z = z0 + (z1 - z0) * i / steps;
        for dx in 0..2 {
            for dz in 0..2 {
                m.set(
                    (x + dx) as u32,
                    y as u32,
                    (z + dz) as u32,
                    Some(color),
                );
            }
        }
    }
}

/// Class D2 (frames and racks), hand-built from each
/// sprite's real-world referent. Sprites wider than
/// 32 (the 64px racks, tripod, clothesline, stooks)
/// are built at tile scale from their proportions.

/// Class D3 (machines and bodies), hand-built from
/// each sprite's real-world referent.

/// Class C (vegetation): upright flowers and herbs
/// take the cross construction (two perpendicular
/// silhouette slabs, the plant_model the alien
/// zapper established); flat mats and water plants
/// take a plan slab; the lotus adds its bud; the
/// indoor plants get a real pot under the cross.

/// Class A (the former exclusion pins), modeled on
/// Joey's instruction: every furniture id gets a
/// model.
fn excl_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    match id {
        "f_alien_anemone" => Some(excl_anemone()),
        "f_bed_frame" => excl_bed_frame(sprite),
        "f_bullettrailer_table_bottom" => plan_slab(sprite, 5),
        "f_bullettrailer_table_middle" => plan_slab(sprite, 4),
        "f_bullettrailer_table_top" => plan_slab(sprite, 4),
        "f_chair" => Some(excl_chair()),
        "f_deckchair" => Some(excl_deckchair()),
        "f_scrap_antenna" => Some(excl_scrap_antenna()),
        "f_small_satelitte_dish" => Some(excl_small_dish(sprite)),
        "f_traffic_light" => Some(excl_traffic_light()),
        "f_winter_wreath" => Some(excl_wreath()),
        _ => None,
    }
}

/// Alien anemone: a columnar grey body with a
/// broad crown of teal tentacles -- the sprite's
/// tentacles spread wider than the body and are
/// its dominant feature.
fn excl_anemone() -> PropModel {
    let mut m = PropModel::new([32, 22, 32]);
    let greys = [
        rgb(110, 108, 106),
        rgb(126, 124, 122),
        rgb(140, 138, 134),
        rgb(96, 94, 92),
    ];
    // Irregular column: per-layer radius with hash
    // jitter and speckle.
    for y in 0..13u32 {
        let r = match y {
            0..=2 => 6.4,
            3..=6 => 5.4,
            7..=10 => 4.4,
            _ => 3.6,
        };
        let cx = if y > 8 { 17 } else { 16 };
        for dx in -8i32..=8 {
            for dz in -8i32..=8 {
                let d = ((dx * dx + dz * dz) as f32).sqrt();
                let jitter = hash01((dx + 20) as u32 * 7 + y, (dz + 20) as u32 * 13) * 1.2;
                if d <= r + jitter - 0.6 {
                    let c = greys[(hash01((dx + 20) as u32 * 3 + y * 31, (dz + 20) as u32 * 5) * 4.0)
                        as usize
                        % 4];
                    m.set((cx + dx) as u32, y, (16 + dz) as u32, Some(c));
                }
            }
        }
    }
    // The crown: a dozen-plus tentacles leaning
    // outward, two cells thick at the base, with
    // bulb tips.
    let teal = rgb(96, 196, 168);
    let teal_dk = rgb(64, 158, 132);
    let tip = rgb(160, 240, 210);
    let stalks: [(i32, i32, u32); 13] = [
        (0, 0, 8),
        (2, 0, 7),
        (-2, 0, 6),
        (0, 2, 7),
        (0, -2, 6),
        (3, 2, 5),
        (-3, 2, 6),
        (3, -2, 6),
        (-3, -2, 5),
        (1, 3, 5),
        (-1, -3, 6),
        (3, 0, 4),
        (-2, 3, 4),
    ];
    for (ox, oz, tall) in stalks {
        let bx = 16 + ox;
        let bz = 16 + oz;
        for dy in 0..tall {
            let lean = (dy / 2) as i32;
            let lx = bx + lean * ox.signum() + if ox == 0 && dy > 2 { 0 } else { 0 };
            let lz = bz + lean * oz.signum();
            m.set(lx as u32, 13 + dy, lz as u32, Some(teal));
            if dy < tall / 2 {
                m.set((lx - ox.signum()) as u32, 13 + dy, (lz - oz.signum()) as u32, Some(teal_dk));
            }
            if dy == tall - 1 {
                m.set(lx as u32, 13 + dy + 1, lz as u32, Some(tip));
                m.set((lx + 1) as u32, 13 + dy, lz as u32, Some(tip));
            }
        }
    }
    m
}

/// Bed frame: the sprite is a plan view -- a plan
/// slab with the headboard band raised.
fn excl_bed_frame(sprite: &RgbaImage) -> Option<PropModel> {
    let mut m = plan_slab(sprite, 3)?;
    let (w, h) = (sprite.width(), sprite.height());
    for z in 0..h {
        for x in 0..w {
            if !opaque(*sprite.get_pixel(x, z)) {
                continue;
            }
            let top = if z <= 7 {
                7
            } else if z >= h - 4 {
                5
            } else {
                continue;
            };
            let px = *sprite.get_pixel(x, z);
            for y in 3..top {
                m.set(x, y, z, Some(px));
            }
        }
    }
    Some(m)
}

/// A simple chair: grey frame, tan seat and back
/// panels (the sprite's south face).
fn excl_chair() -> PropModel {
    let mut m = PropModel::new([32, 26, 32]);
    let frame = rgb(128, 130, 133);
    let tan = rgb(206, 188, 152);
    for x in [11u32, 20] {
        for z in [11u32, 19] {
            box_fill(&mut m, x, x + 1, 0, 11, z, z + 1, frame);
        }
        box_fill(&mut m, x, x + 1, 11, 25, 11, 12, frame);
    }
    box_fill(&mut m, 10, 22, 11, 13, 11, 20, frame);
    box_fill(&mut m, 12, 19, 12, 12, 13, 18, tan);
    box_fill(&mut m, 10, 22, 19, 25, 11, 12, frame);
    box_fill(&mut m, 12, 19, 20, 24, 12, 12, tan);
    m
}

/// Deckchair: wooden side frames and a long reclined
/// sling.
fn excl_deckchair() -> PropModel {
    let mut m = PropModel::new([32, 18, 32]);
    let wood = rgb(150, 108, 58);
    let fabric = rgb(226, 228, 230);
    let stripe = rgb(96, 124, 192);
    for x in [10i32, 21] {
        beam(&mut m, x, 0, 24, x, 9, 12, wood);
        beam(&mut m, x, 0, 6, x, 16, 11, wood);
        box_fill(&mut m, x as u32, x as u32 + 1, 0, 1, 6, 25, wood);
    }
    // The sling: overlapping stepped bands from the
    // backrest top to the seat front, in the sprite's
    // blue-and-white stripes (stripes run along the
    // sling, so they vary along x).
    let steps = [
        (15u32, 10i32),
        (13, 13),
        (11, 16),
        (9, 19),
        (8, 22),
    ];
    for (y, z) in steps {
        for x in 11..=21u32 {
            let c = if x % 4 == 1 || x % 4 == 2 {
                stripe
            } else {
                fabric
            };
            for dy in 0..=1u32 {
                for dz in 0..=3u32 {
                    m.set(x, y - dy, (z + dz as i32) as u32, Some(c));
                }
            }
        }
    }
    // The sling's back drape: it wraps over the top
    // bar and hangs down the back (without it the
    // stepped sling reads as a single band from
    // behind, the steps grazing out of view).
    for x in 11..=21u32 {
        let c = if x % 4 == 1 || x % 4 == 2 {
            stripe
        } else {
            fabric
        };
        for y in 9..=13u32 {
            m.set(x, y, 10, Some(c));
        }
    }
    m
}

/// Scrap antenna station: a brown equipment
/// cabinet with a control box and indicator lights,
/// a tall mast with five gold elements, a curved
/// dish arm, and a second shorter mast.
fn excl_scrap_antenna() -> PropModel {
    let mut m = PropModel::new([32, 45, 32]);
    let brown = rgb(106, 88, 74);
    let brown_dk = rgb(80, 65, 54);
    let green_dk = rgb(74, 96, 74);
    let gold = rgb(216, 180, 32);
    let grey = rgb(170, 172, 175);
    // Cabinet with panel lines.
    box_fill(&mut m, 8, 24, 0, 16, 10, 22, brown);
    for y in [5u32, 11] {
        for x in 8..=24u32 {
            m.set(x, y, 22, Some(brown_dk));
        }
    }
    box_fill(&mut m, 8, 24, 0, 1, 10, 22, brown_dk);
    // Control box with gold buttons and a drooping
    // magenta cable.
    box_fill(&mut m, 12, 19, 3, 7, 22, 23, rgb(150, 152, 155));
    m.set(13, 5, 23, Some(gold));
    m.set(17, 4, 23, Some(gold));
    for (i, (x, y)) in [(20u32, 5u32), (21, 4), (22, 3), (23, 2)]
        .iter()
        .enumerate()
    {
        let _ = i;
        m.set(*x, *y, 23, Some(rgb(196, 44, 110)));
    }
    // Indicator row: purple, green, pink.
    m.set(11, 14, 22, Some(rgb(150, 60, 220)));
    m.set(12, 14, 22, Some(rgb(150, 60, 220)));
    m.set(15, 14, 22, Some(rgb(52, 196, 84)));
    m.set(16, 14, 22, Some(rgb(52, 196, 84)));
    m.set(19, 14, 22, Some(rgb(226, 90, 160)));
    m.set(20, 14, 22, Some(rgb(226, 90, 160)));
    // Display unit on the cabinet top.
    box_fill(&mut m, 14, 18, 17, 20, 11, 14, rgb(52, 54, 56));
    for x in 15..=17u32 {
        for y in 18..=19u32 {
            m.set(x, y, 14, Some(rgb(150, 60, 220)));
        }
    }
    // Tall mast with five gold elements.
    box_fill(&mut m, 10, 11, 16, 44, 13, 14, green_dk);
    for y in [22u32, 27, 32, 37, 42] {
        box_fill(&mut m, 9, 12, y, y + 1, 13, 14, gold);
    }
    // Curved dish arm sweeping right and down.
    for (x, y) in [
        (13u32, 41u32),
        (15, 39),
        (17, 36),
        (19, 33),
        (21, 30),
        (23, 27),
        (24, 25),
    ] {
        box_fill(&mut m, x, x + 1, y, y + 1, 13, 14, grey);
    }
    // Second, shorter mast with a feed box.
    box_fill(&mut m, 23, 24, 16, 30, 13, 14, green_dk);
    box_fill(&mut m, 22, 25, 30, 32, 12, 15, grey);
    m
}

/// Small satellite dish: the sprite's disc on a
/// dished surface, feed arm, and mount post.
fn excl_small_dish(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 27, 32]);
    let (cx, cy) = (16i32, 13i32);
    for y in 0..27u32 {
        for x in 0..32u32 {
            let px = *sprite.get_pixel(x, 26 - y);
            if !opaque(px) {
                continue;
            }
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            let z = 13 + (3.0 * (1.0 - (d / 14.0).powi(2))).round() as u32;
            m.set(x, y, z.min(16), Some(px));
        }
    }
    // Feed arm and feed box.
    beam(&mut m, 15, 1, 13, 15, 9, 18, rgb(90, 92, 95));
    box_fill(&mut m, 15, 17, 9, 11, 18, 19, rgb(70, 72, 75));
    // Mount post and base.
    box_fill(&mut m, 15, 17, 0, 5, 11, 13, rgb(110, 112, 115));
    box_fill(&mut m, 12, 20, 0, 0, 9, 15, rgb(110, 112, 115));
    m
}

/// Portable traffic light: a green pole with a
/// three-lens head on a round concrete base.
fn excl_traffic_light() -> PropModel {
    let mut m = PropModel::new([32, 47, 32]);
    let green = rgb(38, 128, 84);
    let green_dk = rgb(28, 96, 62);
    cyl_y(&mut m, 16, 16, 8.0, 0, 2, rgb(128, 128, 128));
    cyl_y(&mut m, 16, 16, 1.3, 2, 44, green);
    // Head with the three lenses on the front.
    box_fill(&mut m, 13, 19, 35, 45, 14, 17, rgb(40, 42, 44));
    for (ly, c) in [
        (42u32, rgb(216, 48, 40)),
        (39, rgb(232, 186, 44)),
        (36, rgb(52, 196, 84)),
    ] {
        box_fill(&mut m, 15, 16, ly, ly + 1, 17, 17, c);
    }
    box_fill(&mut m, 13, 19, 45, 46, 14, 17, green_dk);
    m
}

/// Winter wreath: a green ring with brown bow
/// accents, in the front plane.
fn excl_wreath() -> PropModel {
    let mut m = PropModel::new([32, 20, 32]);
    let green = rgb(52, 102, 44);
    let green_lt = rgb(84, 142, 62);
    let bow = rgb(128, 84, 48);
    let (cx, cy) = (16i32, 10i32);
    for dx in -9i32..=9 {
        for dy in -9i32..=9 {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if (5.2..=9.2).contains(&d) {
                let c = if hash01((cx + dx) as u32, (cy + dy) as u32) < 0.4 {
                    green_lt
                } else {
                    green
                };
                for z in 14..=15u32 {
                    m.set((cx + dx) as u32, (cy + dy) as u32, z, Some(c));
                }
            }
        }
    }
    // Bow accents at the sides and bottom, hanger
    // at the top.
    box_fill(&mut m, 6, 8, 9, 11, 14, 15, bow);
    box_fill(&mut m, 24, 26, 9, 11, 14, 15, bow);
    box_fill(&mut m, 14, 18, 0, 2, 14, 15, bow);
    m.set(16, 19, 14, Some(bow));
    m.set(16, 19, 15, Some(bow));
    m
}

fn veg_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    match id {
        "f_lilypad" | "f_flower_spurge" => plan_slab(sprite, 2),
        "f_fungal_mass" | "f_fungal_tangle" => plan_slab(sprite, 3),
        "f_lotus" => {
            let mut m = plan_slab(sprite, 2)?;
            // The bud: the pinkest cluster in the
            // sprite, raised as a small lathe bud.
            let (mut bx, mut bz, mut best) = (16u32, 16u32, 0i32);
            for y in 0..sprite.height() {
                for x in 0..sprite.width() {
                    let px = sprite.get_pixel(x, y);
                    if opaque(*px) {
                        let score = px[0] as i32 - px[1] as i32;
                        if score > best {
                            best = score;
                            bx = x;
                            bz = y;
                        }
                    }
                }
            }
            let pink = rgb(233, 151, 183);
            cyl_y(&mut m, bx as i32, bz as i32, 1.9, 2, 3, pink);
            cyl_y(&mut m, bx as i32, bz as i32, 1.2, 4, 5, pink);
            Some(m)
        }
        "f_indoor_plant"
        | "f_indoor_plant_y"
        | "f_indoor_plant_y_season_autumn"
        | "f_indoor_plant_y_season_winter" => veg_potted(sprite),
        "f_black_eyed_susan"
        | "f_bluebell"
        | "f_burdock"
        | "f_carrot_wild"
        | "f_carrot_wild_season_autumn"
        | "f_carrot_wild_season_summer"
        | "f_carrot_wild_season_winter"
        | "f_cattails_season_autumn"
        | "f_cattails_season_spring"
        | "f_cattails_season_summer"
        | "f_cattails_season_winter"
        | "f_chamomile"
        | "f_chicory"
        | "f_dahlia"
        | "f_dandelion_season_autumn"
        | "f_dandelion_season_spring"
        | "f_dandelion_season_summer"
        | "f_dandelion_season_winter"
        | "f_datura"
        | "f_flower_fungal"
        | "f_flower_tulip"
        | "f_lily"
        | "f_mustard_season_autumn"
        | "f_mustard_season_spring"
        | "f_mustard_season_summer"
        | "f_mustard_season_winter"
        | "f_mutpoppy"
        | "f_plant_harvest"
        | "f_plant_mature"
        | "f_plant_seed"
        | "f_plant_seedling"
        | "f_sunflower_season_autumn"
        | "f_sunflower_season_spring"
        | "f_sunflower_season_summer"
        | "f_sunflower_season_winter" => plant_model(sprite),
        _ => None,
    }
}

/// A potted plant: terracotta pot (lathe) with the
/// sprite's foliage as a cross above it. The pot is
/// the sprite's bottom 7 band rows.
fn veg_potted(sprite: &RgbaImage) -> Option<PropModel> {
    let (w, h) = (sprite.width(), sprite.height());
    let mut rows = (0..h).filter(|&y| (0..w).any(|x| opaque(*sprite.get_pixel(x, y))));
    let band = rows.next()?..rows.last()? + 1;
    let pot_h = 7u32.min(band.end - band.start);
    let fol_h = band.end - band.start - pot_h;
    if fol_h == 0 {
        return None;
    }
    // Crop the foliage rows into their own image.
    let mut fol = RgbaImage::new(w, fol_h);
    for y in 0..fol_h {
        for x in 0..w {
            fol.put_pixel(x, y, *sprite.get_pixel(x, band.start + y));
        }
    }
    let cross = plant_model(&fol)?;
    let terra = rgb(158, 82, 50);
    let terra_dk = rgb(122, 60, 36);
    let mut m = PropModel::new([w, pot_h + cross.size[1], w]);
    let cx = (w / 2) as i32;
    cyl_y(&mut m, cx, cx, 3.1, 0, 1, terra_dk);
    cyl_y(&mut m, cx, cx, 3.9, 2, 4, terra);
    cyl_y(&mut m, cx, cx, 4.4, 5, 6, terra);
    cyl_y(&mut m, cx, cx, 3.6, 6, 6, rgb(74, 56, 40));
    for y in 0..cross.size[1] {
        for z in 0..cross.size[2] {
            for x in 0..cross.size[0] {
                if let Some(c) = cross.get(x, y, z) {
                    m.set(x, y + pot_h - 1, z, Some(c));
                }
            }
        }
    }
    Some(m)
}

fn d3_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    match id {
        "f_alembic" => Some(d3_alembic()),
        "f_beverly_shear" => Some(d3_beverly_shear()),
        "f_drill_press" => Some(d3_drill_press()),
        "f_hydraulic_press" => Some(d3_hydraulic_press()),
        "f_manual_tire_changer" => Some(d3_tire_changer()),
        "f_ergometer" => Some(d3_ergometer()),
        "f_exercise" => Some(d3_exercise_bench()),
        "f_mannequin" => Some(d3_mannequin()),
        "f_training_dummy_heavy" => Some(d3_dummy(true)),
        "f_training_dummy_light" => Some(d3_dummy(false)),
        "f_pinball_machine" => Some(d3_pinball(sprite)),
        "f_mailbox" => Some(d3_mailbox()),
        "f_recycle_bin" => Some(d3_recycle_bin(sprite)),
        "f_water_pump" => Some(d3_water_pump()),
        "f_wind_mill" => Some(d3_windmill()),
        "f_exodii_portal_tower" => Some(d3_portal_tower()),
        "f_exodii_lamp_string_warped" => Some(d3_lamp_string()),
        "f_eyewash" => Some(d3_eyewash()),
        _ => None,
    }
}

/// Alembic still: copper boiler, a swooping glass
/// arm, two receiver flasks.
fn d3_alembic() -> PropModel {
    let mut m = PropModel::new([32, 30, 32]);
    let copper = rgb(196, 116, 52);
    let copper_dk = rgb(150, 84, 36);
    let glass = rgb(198, 204, 198);
    let flask = rgb(148, 150, 148);
    // Boiler: squat pot with a domed lid.
    cyl_y(&mut m, 9, 13, 5.0, 0, 7, copper);
    cyl_y(&mut m, 9, 13, 4.2, 8, 10, copper_dk);
    cyl_y(&mut m, 9, 13, 2.4, 11, 13, copper);
    for a in 0..24 {
        let t = a as f32 / 24.0 * std::f32::consts::TAU;
        let x = (9.0 + 5.0 * t.cos()) as u32;
        let z = (13.0 + 5.0 * t.sin()) as u32;
        m.set(x, 4, z, Some(copper_dk));
    }
    // The arm: three beam segments arcing right.
    beam(&mut m, 9, 13, 13, 12, 23, 14, glass);
    beam(&mut m, 12, 23, 14, 18, 26, 15, glass);
    beam(&mut m, 18, 26, 15, 23, 24, 16, glass);
    beam(&mut m, 23, 24, 16, 26, 19, 16, glass);
    // Receiver flasks: neck + bell.
    for (cx, cz, top) in [(23i32, 16i32, 20u32), (26, 16, 15)] {
        cyl_y(&mut m, cx, cz, 1.2, top - 3, top, flask);
        cyl_y(&mut m, cx, cz, 3.4, top - 9, top - 4, flask);
        cyl_y(&mut m, cx, cz, 3.9, top - 10, top - 9, flask);
    }
    m
}

/// Beverly shear: bench body, jaw, and the long
/// curved blade arm.
fn d3_beverly_shear() -> PropModel {
    let mut m = PropModel::new([32, 23, 32]);
    let steel = rgb(150, 152, 155);
    let steel_dk = rgb(96, 98, 102);
    box_fill(&mut m, 13, 18, 0, 6, 13, 18, steel_dk);
    box_fill(&mut m, 14, 17, 6, 10, 14, 17, steel);
    // The workpiece in the jaw.
    box_fill(&mut m, 15, 17, 8, 8, 17, 20, rgb(196, 164, 110));
    // Curved arm in the front plane.
    for (x, y) in [
        (16u32, 10u32),
        (16, 13),
        (15, 16),
        (15, 19),
        (16, 21),
        (17, 22),
    ] {
        box_fill(&mut m, x, x + 1, y, y + 1, 14, 15, steel);
    }
    m
}

/// Drill press, built in profile: base, column,
/// table, head extending forward, spindle at its tip.
fn d3_drill_press() -> PropModel {
    let mut m = PropModel::new([32, 35, 32]);
    let green = rgb(84, 168, 92);
    let green_dk = rgb(56, 120, 64);
    let grey = rgb(150, 152, 150);
    let grey_dk = rgb(96, 98, 96);
    box_fill(&mut m, 11, 21, 0, 3, 7, 25, green_dk);
    cyl_y(&mut m, 16, 10, 2.2, 3, 30, grey);
    // Table on its bracket.
    box_fill(&mut m, 14, 18, 12, 15, 10, 14, green_dk);
    box_fill(&mut m, 11, 21, 15, 17, 14, 25, grey);
    m.set(16, 17, 20, Some(grey_dk));
    // Head and motor housing.
    box_fill(&mut m, 12, 20, 28, 34, 8, 25, green);
    box_fill(&mut m, 12, 20, 32, 34, 8, 13, green_dk);
    // Spindle and chuck at the head's tip.
    cyl_y(&mut m, 16, 23, 1.1, 23, 28, grey_dk);
    cyl_y(&mut m, 16, 23, 1.7, 24, 26, grey);
    // Red switch on the head's front face.
    m.set(16, 31, 25, Some(rgb(206, 44, 40)));
    m.set(13, 30, 25, Some(green_dk));
    m
}

/// Hydraulic press: red H-frame, ram, bolster table.
fn d3_hydraulic_press() -> PropModel {
    let mut m = PropModel::new([32, 37, 32]);
    let red = rgb(198, 52, 44);
    let red_dk = rgb(150, 38, 34);
    let steel_dk = rgb(84, 86, 90);
    for x in [9u32, 20] {
        box_fill(&mut m, x, x + 3, 0, 34, 13, 18, red);
        box_fill(&mut m, x - 1, x + 4, 0, 1, 10, 21, red_dk);
    }
    box_fill(&mut m, 9, 23, 31, 36, 13, 18, red);
    box_fill(&mut m, 9, 23, 0, 3, 13, 18, red_dk);
    // Ram cylinder and piston.
    cyl_y(&mut m, 16, 15, 2.6, 25, 31, steel_dk);
    cyl_y(&mut m, 16, 15, 1.6, 20, 25, rgb(170, 172, 175));
    // Bolster table.
    box_fill(&mut m, 10, 22, 16, 19, 11, 20, steel_dk);
    m
}

/// Manual tire changer: base, a wheel rim on its
/// hub, and the red swing-arm post.
fn d3_tire_changer() -> PropModel {
    let mut m = PropModel::new([32, 26, 32]);
    let dark = rgb(52, 52, 56);
    let grey = rgb(152, 154, 156);
    let grey_dk = rgb(100, 102, 105);
    let red = rgb(204, 56, 44);
    box_fill(&mut m, 8, 24, 0, 2, 8, 24, dark);
    // Rim: a ring in the front plane with spokes.
    for dx in -8i32..=8 {
        for dy in -8i32..=8 {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if (6.0..=7.6).contains(&d) {
                for z in 14..=16u32 {
                    m.set((16 + dx) as u32, (13 + dy) as u32, z, Some(grey));
                }
            }
        }
    }
    box_fill(&mut m, 15, 17, 12, 14, 13, 17, grey_dk);
    for (x, y) in [(16u32, 19u32), (16, 7), (10, 13), (22, 13)] {
        box_fill(&mut m, x - 1, x + 1, y - 1, y + 1, 14, 16, grey_dk);
    }
    // Red post and swing arm.
    box_fill(&mut m, 8, 10, 0, 24, 14, 16, red);
    box_fill(&mut m, 8, 20, 22, 24, 14, 16, red);
    box_fill(&mut m, 19, 20, 18, 22, 14, 16, grey_dk);
    m
}

/// Ergometer (stationary bike): base rails, flywheel
/// body at the front, saddle and handlebar posts.
fn d3_ergometer() -> PropModel {
    let mut m = PropModel::new([32, 40, 32]);
    let grey = rgb(168, 170, 172);
    let grey_dk = rgb(104, 106, 110);
    let cyan = rgb(64, 148, 178);
    let white = rgb(226, 228, 228);
    for x in [11u32, 19] {
        box_fill(&mut m, x, x + 2, 0, 2, 6, 26, grey_dk);
    }
    // Flywheel body at the front.
    box_fill(&mut m, 10, 22, 2, 14, 18, 26, grey);
    for x in 12..=20u32 {
        for y in 4..=12u32 {
            m.set(x, y, 26, Some(white));
        }
    }
    // Seat post and saddle.
    beam(&mut m, 15, 10, 9, 15, 24, 7, grey_dk);
    box_fill(&mut m, 12, 20, 24, 26, 5, 11, cyan);
    // Handlebar post, bar, grips, console.
    beam(&mut m, 15, 12, 21, 15, 30, 23, grey_dk);
    box_fill(&mut m, 8, 24, 30, 31, 23, 25, grey);
    box_fill(&mut m, 8, 10, 30, 31, 23, 25, cyan);
    box_fill(&mut m, 22, 24, 30, 31, 23, 25, cyan);
    box_fill(&mut m, 13, 19, 32, 34, 23, 25, grey_dk);
    m.set(16, 33, 25, Some(white));
    // Pedal cranks.
    box_fill(&mut m, 8, 10, 7, 9, 20, 22, cyan);
    box_fill(&mut m, 22, 24, 7, 9, 20, 22, cyan);
    m
}

/// Bench press station, read from the sprite's
/// pixel map: the bench runs TOWARD the viewer on a
/// central pedestal (pad face foreshortened, raised
/// head section at the far end), and the rack stands
/// behind it -- two uprights rising past the bar,
/// plates hanging on the bar just outside them.
fn d3_exercise_bench() -> PropModel {
    let mut m = PropModel::new([32, 39, 32]);
    let steel = rgb(158, 160, 163);
    let steel_dk = rgb(102, 104, 108);
    let red = rgb(172, 34, 36);
    let red_dk = rgb(128, 24, 28);
    let red_lt = rgb(196, 96, 62);
    // Rack uprights at the back, rising past the bar.
    for x in [5u32, 25] {
        box_fill(&mut m, x, x + 1, 0, 38, 12, 14, steel);
        box_fill(&mut m, x - 1, x + 2, 0, 1, 8, 18, steel_dk);
    }
    // Bar at y 31, resting in hooks below the tops.
    tube_x(&mut m, 1, 30, 31, 13, 0.9, steel);
    for x in [7u32, 24] {
        m.set(x, 30, 13, Some(steel_dk));
    }
    // Plates: pairs of discs just outside the
    // uprights, centered on the bar.
    for xc in [1i32, 3, 27, 29] {
        for dy in -6i32..=6 {
            for dz in -6i32..=6 {
                let d = ((dy * dy + dz * dz) as f32).sqrt();
                if d <= 6.0 {
                    m.set(xc as u32, (31 + dy) as u32, (13 + dz) as u32, Some(steel_dk));
                }
            }
        }
    }
    // Bench pad along z on a central pedestal: the
    // wide near end, a narrower raised head section
    // at the far end by the rack.
    box_fill(&mut m, 9, 22, 11, 13, 20, 30, red);
    box_fill(&mut m, 9, 22, 11, 11, 20, 30, red_dk);
    box_fill(&mut m, 11, 20, 11, 13, 15, 20, red);
    box_fill(&mut m, 12, 19, 14, 14, 15, 19, red_lt);
    // The pad's lit left edge (the sprite's stripe).
    for z in 20..=30u32 {
        m.set(9, 13, z, Some(red_lt));
    }
    // Pedestal column and T foot; far support leg.
    box_fill(&mut m, 14, 17, 2, 11, 22, 25, steel_dk);
    box_fill(&mut m, 10, 21, 0, 1, 20, 27, steel_dk);
    box_fill(&mut m, 14, 17, 0, 11, 16, 17, steel_dk);
    m
}

/// Mannequin: a pale humanoid figure.
fn d3_mannequin() -> PropModel {
    let mut m = PropModel::new([32, 33, 32]);
    let body = rgb(206, 204, 199);
    let body_dk = rgb(168, 166, 160);
    for x in [13u32, 17] {
        box_fill(&mut m, x, x + 2, 0, 14, 14, 17, body);
        box_fill(&mut m, x, x + 2, 0, 1, 17, 20, body_dk);
    }
    box_fill(&mut m, 12, 20, 14, 16, 13, 18, body);
    box_fill(&mut m, 11, 21, 16, 26, 13, 18, body);
    box_fill(&mut m, 12, 20, 24, 26, 13, 18, body_dk);
    beam(&mut m, 10, 25, 14, 7, 16, 14, body);
    beam(&mut m, 21, 25, 14, 24, 16, 14, body);
    cyl_y(&mut m, 16, 15, 1.2, 26, 27, body_dk);
    box_fill(&mut m, 13, 19, 28, 32, 13, 18, body);
    m
}

/// Training dummy (pell): a padded torso and helmet
/// head on a post with a weighted base. The heavy
/// version is steel-grey and broader; the light one
/// is wood and tan.
fn d3_dummy(heavy: bool) -> PropModel {
    let mut m = PropModel::new([32, 30, 32]);
    let (pad, strap, helm, face) = if heavy {
        (rgb(150, 148, 145), rgb(88, 86, 84), rgb(140, 142, 145), rgb(210, 180, 120))
    } else {
        (rgb(190, 150, 90), rgb(110, 80, 44), rgb(150, 100, 55), rgb(226, 196, 140))
    };
    cyl_y(&mut m, 16, 16, 6.0, 0, 1, rgb(160, 100, 40));
    cyl_y(&mut m, 16, 16, 1.5, 1, 10, rgb(96, 74, 44));
    let r = if heavy { 5.0 } else { 4.2 };
    cyl_y(&mut m, 16, 16, r - 0.6, 10, 13, pad);
    cyl_y(&mut m, 16, 16, r, 14, 20, pad);
    cyl_y(&mut m, 16, 16, r - 0.8, 21, 23, pad);
    for y in [13u32, 20] {
        for a in 0..28 {
            let t = a as f32 / 28.0 * std::f32::consts::TAU;
            let x = (16.0 + r * t.cos()) as u32;
            let z = (16.0 + r * t.sin()) as u32;
            m.set(x, y, z, Some(strap));
        }
    }
    for x in [9u32, 21] {
        box_fill(&mut m, x, x + 2, 17, 20, 14, 17, pad);
    }
    cyl_y(&mut m, 16, 16, 3.0, 24, 28, helm);
    for x in 13..=19u32 {
        m.set(x, 25, 19, Some(face));
        m.set(x, 26, 19, Some(face));
    }
    m.set(16, 29, 16, Some(helm));
    m
}

/// Pinball machine: legs, a stepped cabinet with
/// the playfield art on its steps, and the backbox
/// with the sprite's art on its face.
fn d3_pinball(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 27, 32]);
    let navy = rgb(36, 40, 92);
    let grey = rgb(150, 152, 155);
    for x in [12u32, 27] {
        for z in [8u32, 23] {
            box_fill(&mut m, x, x + 1, 0, 8, z, z + 1, grey);
        }
    }
    // Cabinet in three steps, art sampled per step.
    for (z0, z1, y) in [(6u32, 13u32, 16u32), (14, 19, 14), (20, 26, 12)] {
        box_fill(&mut m, 11, 29, 8, y, z0, z1, navy);
        for x in 12..=28u32 {
            let row = 6 + (26 - z1) / 2;
            let px = *sprite.get_pixel(x, row);
            if opaque(px) {
                m.set(x, y, z1 - 1, Some(px));
                m.set(x, y, z0, Some(px));
            }
        }
    }
    // Coin door on the front.
    box_fill(&mut m, 17, 22, 8, 11, 26, 26, grey);
    // Backbox with its art stamped on the face.
    box_fill(&mut m, 11, 29, 16, 26, 6, 10, navy);
    for x in 11..=29u32 {
        for y in 16..=26u32 {
            let px = *sprite.get_pixel(x, 26 - y);
            if opaque(px) {
                m.set(x, y, 10, Some(px));
            }
        }
    }
    m
}

/// Mailbox: post, and the tunnel-profile box with
/// its door and red flag.
fn d3_mailbox() -> PropModel {
    let mut m = PropModel::new([32, 25, 32]);
    let blue = rgb(128, 138, 198);
    let blue_dk = rgb(88, 96, 152);
    let wood = rgb(146, 110, 62);
    box_fill(&mut m, 15, 17, 0, 12, 14, 16, wood);
    box_fill(&mut m, 12, 20, 11, 12, 14, 16, wood);
    // Tunnel shell extruded along z.
    for z in 10..=20u32 {
        for x in 12..=20u32 {
            m.set(x, 13, z, Some(blue));
        }
        for y in 14..=19u32 {
            m.set(12, y, z, Some(blue));
            m.set(20, y, z, Some(blue));
        }
        for x in 13..=19u32 {
            m.set(x, 20, z, Some(blue));
        }
        for x in 15..=17u32 {
            m.set(x, 21, z, Some(blue));
        }
    }
    // Door on the front face.
    for x in 13..=19u32 {
        for y in 14..=19u32 {
            m.set(x, y, 20, Some(blue_dk));
        }
    }
    m.set(16, 17, 20, Some(rgb(60, 62, 66)));
    // Flag, up, on the right side.
    box_fill(&mut m, 21, 21, 18, 22, 17, 18, rgb(206, 52, 44));
    m
}

/// Recycle bin: tapered green body, lid, and the
/// sprite stamped on the front.
fn d3_recycle_bin(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 17, 32]);
    let green = rgb(72, 130, 82);
    let green_lt = rgb(96, 156, 102);
    box_fill(&mut m, 12, 20, 0, 11, 12, 19, green);
    box_fill(&mut m, 11, 21, 11, 15, 11, 20, green);
    box_fill(&mut m, 10, 22, 15, 16, 10, 21, green_lt);
    for x in 11..=21u32 {
        for y in 0..=16u32 {
            let px = *sprite.get_pixel(x - 1, 27 - y);
            if opaque(px) {
                let z = if y <= 11 { 19 } else { 20 };
                m.set(x, y, z, Some(px));
            }
        }
    }
    m
}

/// Hand water pump: flanged base, pipe, head, spout.
fn d3_water_pump() -> PropModel {
    let mut m = PropModel::new([32, 37, 32]);
    let green = rgb(86, 150, 106);
    let green_dk = rgb(58, 108, 76);
    cyl_y(&mut m, 16, 16, 4.2, 0, 2, green_dk);
    cyl_y(&mut m, 16, 16, 1.8, 2, 24, green);
    box_fill(&mut m, 13, 19, 24, 28, 13, 19, green);
    // Spout toward the front with a downturned tip.
    tube_z(&mut m, 19, 26, 16, 26, 1.2, green);
    box_fill(&mut m, 15, 17, 23, 25, 25, 27, green_dk);
    // Handle arcing up and back.
    beam(&mut m, 15, 28, 13, 14, 36, 7, green_dk);
    box_fill(&mut m, 13, 15, 35, 36, 5, 7, green_dk);
    m
}

/// Farm windmill: lattice tower and a big fan wheel.
fn d3_windmill() -> PropModel {
    let mut m = PropModel::new([32, 51, 32]);
    let rust = rgb(142, 96, 60);
    let rust_lt = rgb(172, 118, 74);
    let dark = rgb(96, 66, 40);
    for (bx, bz) in [(7i32, 10i32), (25, 10), (7, 22), (25, 22)] {
        let tx = if bx < 16 { 12 } else { 20 };
        let tz = if bz < 16 { 13 } else { 19 };
        beam(&mut m, bx, 0, bz, tx, 33, tz, rust);
    }
    for y in [12u32, 22] {
        let t = y as f32 / 33.0;
        let xl = (7.0 + 5.0 * t) as u32;
        let xh = (25.0 - 5.0 * t) as u32;
        let zl = (10.0 + 3.0 * t) as u32;
        let zh = (22.0 - 3.0 * t) as u32;
        box_fill(&mut m, xl, xh, y, y, zl, zl, dark);
        box_fill(&mut m, xl, xh, y, y, zh, zh, dark);
        box_fill(&mut m, xl, xl, y, y, zl, zh, dark);
        box_fill(&mut m, xh, xh, y, y, zl, zh, dark);
    }
    box_fill(&mut m, 11, 21, 33, 34, 12, 20, dark);
    // Fan wheel in the front plane.
    let (cx, cy) = (16i32, 41i32);
    for dx in -10i32..=10 {
        for dy in -10i32..=10 {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if d > 10.2 || d < 1.6 {
                continue;
            }
            let ang = (dy as f32).atan2(dx as f32);
            let sector = ((ang / std::f32::consts::TAU * 9.0).rem_euclid(9.0)) as u32;
            let frac = (ang / std::f32::consts::TAU * 9.0).rem_euclid(1.0);
            if d <= 4.0 || frac < 0.62 || sector % 3 == 0 && frac < 0.8 {
                let c = if hash01((cx + dx) as u32, (cy + dy) as u32) < 0.35 {
                    rust
                } else {
                    rust_lt
                };
                for z in 10..=11u32 {
                    m.set((cx + dx) as u32, (cy + dy) as u32, z, Some(c));
                }
            }
        }
    }
    box_fill(&mut m, 14, 18, 39, 43, 11, 14, dark);
    m
}

/// Exodii portal tower: four banded legs, a central
/// column, and the cradle with its two prongs.
fn d3_portal_tower() -> PropModel {
    let mut m = PropModel::new([32, 48, 32]);
    let dark = rgb(70, 78, 70);
    let darker = rgb(48, 54, 48);
    let gold = rgb(178, 138, 52);
    let grey = rgb(150, 152, 150);
    for (bx, bz) in [(10i32, 12i32), (22, 12), (10, 20), (22, 20)] {
        let tx = if bx < 16 { 12 } else { 20 };
        let tz = if bz < 16 { 13 } else { 19 };
        beam(&mut m, bx, 0, bz, tx, 36, tz, dark);
        for y in [6i32, 24] {
            let t = y as f32 / 36.0;
            let lx = (bx as f32 + (tx - bx) as f32 * t) as u32;
            let lz = (bz as f32 + (tz - bz) as f32 * t) as u32;
            box_fill(&mut m, lx - 1, lx + 1, y as u32, y as u32 + 1, lz - 1, lz + 1, gold);
        }
    }
    cyl_y(&mut m, 16, 16, 2.0, 8, 38, darker);
    // Cradle: shallow wide bowl with raised ends.
    box_fill(&mut m, 6, 26, 38, 40, 10, 22, grey);
    box_fill(&mut m, 6, 8, 41, 43, 10, 22, grey);
    box_fill(&mut m, 24, 26, 41, 43, 10, 22, grey);
    box_fill(&mut m, 6, 26, 41, 42, 10, 11, grey);
    box_fill(&mut m, 6, 26, 41, 42, 21, 22, grey);
    // Prongs: gold left, green right.
    box_fill(&mut m, 7, 8, 44, 47, 12, 13, gold);
    box_fill(&mut m, 24, 25, 44, 47, 19, 20, rgb(140, 190, 110));
    // Two small lights on the cradle's front.
    m.set(13, 39, 22, Some(rgb(230, 150, 170)));
    m.set(19, 39, 22, Some(rgb(230, 150, 170)));
    m
}

/// Warped lamp string: a leaning post with a cloth
/// wrap, a glowing green lamp blob, and a rock base.
fn d3_lamp_string() -> PropModel {
    let mut m = PropModel::new([32, 43, 32]);
    let rock = rgb(172, 166, 150);
    let post = rgb(62, 67, 86);
    let cloth = rgb(182, 156, 114);
    cyl_y(&mut m, 16, 16, 5.0, 0, 1, rock);
    cyl_y(&mut m, 16, 16, 3.8, 2, 2, rock);
    beam(&mut m, 15, 2, 15, 13, 34, 14, post);
    for y in 16..=22u32 {
        let t = (y - 2) as f32 / 32.0;
        let px = (15.0 - 2.0 * t) as u32;
        box_fill(&mut m, px - 1, px + 1, y, y, 13, 17, cloth);
    }
    // Lamp blob: a rough sphere of greens.
    let greens = [rgb(118, 236, 130), rgb(80, 200, 96), rgb(192, 250, 182)];
    for dx in -4i32..=4 {
        for dy in -4i32..=4 {
            for dz in -4i32..=4 {
                let d = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                if d <= 3.6 && hash01((dx * 31 + dz) as u32, (dy + 40) as u32) > 0.18 {
                    let c = greens[(hash01(dx as u32, dy as u32) * 3.0) as usize % 3];
                    m.set(
                        (13 + dx) as u32,
                        (38 + dy) as u32,
                        (14 + dz) as u32,
                        Some(c),
                    );
                }
            }
        }
    }
    m.set(13, 33, 14, Some(greens[1]));
    m.set(14, 34, 15, Some(greens[0]));
    m
}

/// Eyewash station: pipe post, sign panel, bowl,
/// and the angled shower pipe.
fn d3_eyewash() -> PropModel {
    let mut m = PropModel::new([32, 39, 32]);
    let green = rgb(96, 176, 92);
    let green_dk = rgb(66, 132, 64);
    cyl_y(&mut m, 19, 16, 3.4, 0, 1, green_dk);
    cyl_y(&mut m, 19, 16, 1.3, 1, 36, green);
    // Sign panel at the top, extending left.
    box_fill(&mut m, 6, 18, 33, 36, 15, 16, rgb(232, 234, 230));
    box_fill(&mut m, 6, 9, 33, 36, 15, 16, green);
    m.set(7, 35, 16, Some(rgb(206, 44, 40)));
    m.set(12, 34, 16, Some(green_dk));
    m.set(15, 34, 16, Some(green_dk));
    // Bowl: a ring on a short arm at mid height.
    beam(&mut m, 18, 24, 15, 11, 24, 15, green_dk);
    for dx in -4i32..=4 {
        for dz in -4i32..=4 {
            let d = ((dx * dx + dz * dz) as f32).sqrt();
            if (3.0..=4.2).contains(&d) {
                m.set((10 + dx) as u32, 25, (16 + dz) as u32, Some(green));
            } else if d < 3.0 {
                m.set((10 + dx) as u32, 24, (16 + dz) as u32, Some(green_dk));
            }
        }
    }
    m.set(10, 26, 16, Some(green_dk));
    // Angled shower pipe down to the right.
    beam(&mut m, 19, 22, 15, 25, 12, 15, rgb(140, 142, 145));
    m
}

fn d2_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    match id {
        "f_aluminum_stepladder" => Some(d2_stepladder()),
        "f_ladder" => Some(d2_ladder()),
        "f_butcher_rack" => Some(d2_butcher_rack()),
        "f_smoking_rack" => Some(d2_smoking_rack(false)),
        "f_smoking_rack_active" => Some(d2_smoking_rack(true)),
        "f_rack_coat" => Some(d2_coat_rack()),
        "f_rotary_clothesline" => Some(d2_clothesline()),
        "f_stook_empty" => Some(d2_stook_empty()),
        "f_stook_full" => Some(d2_stook_full()),
        "f_archery_target_box" => Some(d2_target_box(sprite)),
        "f_barricade_road" => Some(d2_barricade(sprite)),
        "f_counter_gate_o" => Some(d2_counter_gate()),
        "f_bullettrailer_bed_head" => Some(d2_bed_head(sprite)),
        "f_camp_chair" => Some(d2_camp_chair()),
        "f_stool" => Some(d2_stool()),
        _ => None,
    }
}

/// Stepladder: stepped front section, runged rear
/// section slanting back to the hinge, top cap.
fn d2_stepladder() -> PropModel {
    let mut m = PropModel::new([32, 28, 32]);
    let alu = rgb(168, 173, 178);
    let alu_lt = rgb(203, 208, 213);
    let dark = rgb(90, 93, 97);
    for x in [9u32, 22] {
        box_fill(&mut m, x, x + 1, 0, 25, 12, 14, alu);
        m.set(x, 0, 13, Some(dark));
        m.set(x + 1, 0, 13, Some(dark));
    }
    for (i, y) in [4u32, 10, 16, 22].iter().enumerate() {
        let _ = i;
        box_fill(&mut m, 9, 23, *y, *y + 1, 12, 16, alu_lt);
    }
    box_fill(&mut m, 8, 24, 26, 27, 11, 17, alu_lt);
    // Rear section: rails slant from the hinge (z 17)
    // down-back to z 25, with three rungs.
    for x in [9i32, 22] {
        beam(&mut m, x, 0, 25, x, 25, 17, alu);
    }
    for (y, z) in [(6u32, 23i32), (12, 21), (18, 19)] {
        box_fill(&mut m, 9, 23, y, y, z as u32, z as u32, alu);
    }
    m
}

/// A plain wooden ladder: rails, round rungs, and
/// the sprite's wide top board.
fn d2_ladder() -> PropModel {
    let mut m = PropModel::new([32, 28, 32]);
    let wood = rgb(176, 120, 40);
    let wood_dk = rgb(140, 92, 30);
    for x in [9u32, 22] {
        box_fill(&mut m, x, x + 1, 0, 24, 14, 16, wood_dk);
    }
    for y in [5u32, 10, 15, 20] {
        tube_x(&mut m, 10, 22, y as i32, 15, 0.9, wood);
    }
    box_fill(&mut m, 8, 24, 25, 27, 12, 18, wood);
    m
}

/// Butcher rack: two A-frame ends, a top rail with
/// pegs, and two hanging hook blocks.
fn d2_butcher_rack() -> PropModel {
    let mut m = PropModel::new([32, 24, 32]);
    let wood = rgb(165, 120, 50);
    let wood_dk = rgb(120, 84, 34);
    let lump = rgb(190, 188, 180);
    let lump_dk = rgb(120, 118, 110);
    for x in [5i32, 25] {
        beam(&mut m, x, 0, 7, x, 21, 13, wood);
        beam(&mut m, x, 0, 25, x, 21, 19, wood);
        box_fill(
            &mut m,
            x as u32,
            (x + 1) as u32,
            9,
            10,
            11,
            21,
            wood_dk,
        );
    }
    tube_x(&mut m, 5, 27, 22, 16, 1.1, wood);
    for x in [8u32, 12, 16, 20, 24] {
        m.set(x, 23, 16, Some(wood_dk));
    }
    for x0 in [10u32, 19] {
        box_fill(&mut m, x0, x0 + 3, 13, 18, 14, 18, lump);
        m.set(x0 + 1, 15, 16, Some(lump_dk));
        m.set(x0 + 2, 17, 15, Some(lump_dk));
        m.set(x0 + 1, 20, 16, Some(wood_dk));
        m.set(x0 + 1, 19, 16, Some(wood_dk));
    }
    m
}

/// Smoking rack: a tripod over a stone fire ring.
/// The active variant adds the fire and two meat
/// chunks on the crossbar.
fn d2_smoking_rack(active: bool) -> PropModel {
    let mut m = PropModel::new([32, 22, 32]);
    let wood = rgb(140, 100, 45);
    let wood_dk = rgb(105, 74, 32);
    // Tripod legs to the apex (16, 21, 16).
    beam(&mut m, 15, 0, 5, 15, 21, 15, wood);
    beam(&mut m, 6, 0, 23, 15, 21, 15, wood);
    beam(&mut m, 24, 0, 23, 15, 21, 15, wood);
    box_fill(&mut m, 15, 17, 21, 21, 15, 17, wood_dk);
    // Crossbar between the two back legs.
    tube_x(&mut m, 10, 22, 11, 21, 0.9, wood);
    // Stone ring and ash bed.
    for a in 0..48 {
        let t = a as f32 / 48.0 * std::f32::consts::TAU;
        let x = (16.0 + 8.0 * t.cos()) as u32;
        let z = (16.0 + 8.0 * t.sin()) as u32;
        let g = 118 + ((a * 37) % 40) as u8;
        m.set(x, 0, z, Some(rgb(g, g, g - 4)));
        if a % 2 == 0 {
            m.set(x, 1, z, Some(rgb(g - 14, g - 14, g - 18)));
        }
    }
    for dx in -5i32..=5 {
        for dz in -5i32..=5 {
            if dx * dx + dz * dz <= 25 {
                m.set((16 + dx) as u32, 0, (16 + dz) as u32, Some(rgb(64, 59, 54)));
            }
        }
    }
    if active {
        // Fire in the pit.
        let fire = [
            rgb(232, 120, 26),
            rgb(245, 178, 44),
            rgb(250, 214, 96),
        ];
        for (i, (dx, dz)) in [(0i32, 0i32), (1, 0), (-1, 1), (0, -1), (1, 1)]
            .iter()
            .enumerate()
        {
            m.set((16 + dx) as u32, 1, (16 + dz) as u32, Some(fire[i % 3]));
        }
        m.set(16, 2, 16, Some(fire[2]));
        m.set(16, 3, 16, Some(fire[1]));
        // Two meat chunks on the crossbar.
        for x0 in [12u32, 18] {
            box_fill(&mut m, x0, x0 + 2, 7, 10, 20, 22, rgb(168, 42, 36));
            m.set(x0 + 1, 10, 21, Some(rgb(238, 230, 220)));
            m.set(x0, 9, 20, Some(rgb(238, 230, 220)));
        }
    } else {
        // Cold ash pile.
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                m.set((16 + dx) as u32, 1, (16 + dz) as u32, Some(rgb(150, 138, 116)));
            }
        }
    }
    m
}

/// Coat rack: turned post, four upturned hooks,
/// four splayed feet.
fn d2_coat_rack() -> PropModel {
    let mut m = PropModel::new([32, 44, 32]);
    let wood = rgb(146, 99, 47);
    let wood_dk = rgb(110, 72, 33);
    cyl_y(&mut m, 16, 16, 1.2, 2, 40, wood);
    cyl_y(&mut m, 16, 16, 3.0, 0, 3, wood_dk);
    // Feet: radiating bars stepping down to the floor.
    for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
        for d in 1..=6i32 {
            let y = if d <= 2 { 1 } else { 0 };
            m.set((16 + dx * d) as u32, y, (16 + dz * d) as u32, Some(wood));
            m.set(
                (16 + dx * d) as u32,
                y,
                (16 + dz * d + if dx != 0 { 0 } else { 0 }) as u32,
                Some(wood),
            );
        }
    }
    // Hooks: arms stepping up and out, with a tip.
    for (dx, dz) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
        for d in 1..=4i32 {
            let y = 36 + (d / 2) as u32;
            m.set((16 + dx * d) as u32, y, (16 + dz * d) as u32, Some(wood));
        }
        m.set((16 + dx * 5) as u32, 39, (16 + dz * 5) as u32, Some(wood_dk));
    }
    // Top knob.
    box_fill(&mut m, 15, 17, 41, 42, 15, 17, wood_dk);
    m.set(16, 43, 16, Some(wood));
    m
}

/// Rotary clothesline: pole, four diagonal arms,
/// three concentric line squares on the arms' plane.
fn d2_clothesline() -> PropModel {
    let mut m = PropModel::new([32, 27, 32]);
    let steel = rgb(168, 172, 176);
    let steel_dk = rgb(110, 114, 118);
    let line = rgb(203, 198, 172);
    cyl_y(&mut m, 16, 16, 1.3, 0, 20, steel_dk);
    cyl_y(&mut m, 16, 16, 3.0, 0, 1, steel_dk);
    box_fill(&mut m, 14, 18, 20, 21, 14, 18, steel_dk);
    for (cx, cz) in [(3i32, 3i32), (27, 3), (3, 27), (27, 27)] {
        beam(&mut m, 15, 21, 15, cx, 25, cz, steel);
    }
    for (r, y) in [(5i32, 22u32), (9, 24), (13, 25)] {
        let lo = 16 - r;
        let hi = 16 + r;
        for t in lo..=hi {
            m.set(t as u32, y, lo as u32, Some(line));
            m.set(t as u32, y, hi as u32, Some(line));
            m.set(lo as u32, y, t as u32, Some(line));
            m.set(hi as u32, y, t as u32, Some(line));
        }
    }
    m
}

/// The empty stook: a single pole on a cross foot.
fn d2_stook_empty() -> PropModel {
    let mut m = PropModel::new([32, 13, 32]);
    let wood = rgb(150, 112, 52);
    box_fill(&mut m, 11, 21, 0, 0, 15, 17, wood);
    box_fill(&mut m, 15, 17, 0, 0, 11, 21, wood);
    cyl_y(&mut m, 16, 16, 1.0, 0, 12, wood);
    m
}

/// The full stook: a bound sheaf -- stalk bundle
/// flaring into a ragged grain head.
fn d2_stook_full() -> PropModel {
    let mut m = PropModel::new([32, 21, 32]);
    let wood = rgb(150, 112, 52);
    let stalk = rgb(190, 160, 100);
    let grain = rgb(205, 175, 105);
    let grain_dk = rgb(178, 144, 78);
    box_fill(&mut m, 11, 21, 0, 0, 15, 17, wood);
    box_fill(&mut m, 15, 17, 0, 0, 11, 21, wood);
    cyl_y(&mut m, 16, 16, 1.0, 0, 5, wood);
    for y in 1..=9u32 {
        let r = if y <= 5 { 2.2 } else { 1.8 };
        cyl_y(&mut m, 16, 16, r, y, y, stalk);
    }
    box_fill(&mut m, 14, 18, 8, 9, 14, 18, rgb(150, 115, 60));
    for y in 10..=19u32 {
        let r = 2.5 + (y - 10) as f32 * 0.45;
        for dx in -7i32..=7 {
            for dz in -7i32..=7 {
                let d = ((dx * dx + dz * dz) as f32).sqrt();
                if d <= r {
                    let c = if hash01(((16 + dx) * 57 + (16 + dz)) as u32, y) < 0.3 {
                        grain_dk
                    } else {
                        grain
                    };
                    m.set((16 + dx) as u32, y, (16 + dz) as u32, Some(c));
                }
            }
        }
    }
    // Ragged ear tips above the head.
    for dx in -6i32..=6 {
        for dz in -6i32..=6 {
            let d = ((dx * dx + dz * dz) as f32).sqrt();
            if (4.0..=6.6).contains(&d) && hash01((dx * 57 + dz) as u32, 77) < 0.4 {
                m.set((16 + dx) as u32, 20, (16 + dz) as u32, Some(grain));
            }
        }
    }
    m
}

/// Archery target box: a plank crate with the
/// sprite's target face stamped on the front.
fn d2_target_box(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 12, 32]);
    let plank = rgb(186, 152, 96);
    let plank_dk = rgb(150, 118, 70);
    box_fill(&mut m, 10, 23, 0, 11, 11, 20, plank);
    for y in [3u32, 7] {
        for z in 11..=20u32 {
            m.set(10, y, z, Some(plank_dk));
            m.set(23, y, z, Some(plank_dk));
        }
    }
    for x in 10..=21u32 {
        for y in 0..=11u32 {
            let px = *sprite.get_pixel(x, 22 - y);
            if opaque(px) {
                m.set(x, y, 20, Some(px));
            }
        }
    }
    m
}

/// Police barricade: a long lettered board on two
/// A-frame ends.
fn d2_barricade(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 25, 32]);
    let blue = rgb(96, 130, 180);
    let blue_dk = rgb(70, 100, 148);
    box_fill(&mut m, 1, 30, 16, 20, 14, 17, blue);
    for x in 1..=30u32 {
        for y in 16..=20u32 {
            let px = *sprite.get_pixel(x, 25 - y);
            if opaque(px) {
                m.set(x, y, 17, Some(px));
            }
        }
    }
    for x in [4i32, 26] {
        beam(&mut m, x, 0, 8, x, 16, 13, blue_dk);
        beam(&mut m, x, 0, 24, x, 16, 18, blue_dk);
        box_fill(
            &mut m,
            x as u32,
            (x + 1) as u32,
            0,
            1,
            7,
            25,
            blue_dk,
        );
        box_fill(
            &mut m,
            x as u32,
            (x + 1) as u32,
            7,
            8,
            11,
            21,
            blue_dk,
        );
    }
    box_fill(&mut m, 5, 27, 6, 7, 15, 16, blue_dk);
    m
}

/// Counter gate (open): a grey slab swung out from
/// the counter end on its hinge post.
fn d2_counter_gate() -> PropModel {
    let mut m = PropModel::new([32, 22, 32]);
    let counter = rgb(190, 165, 120);
    let grey = rgb(140, 142, 145);
    let grey_dk = rgb(104, 107, 110);
    box_fill(&mut m, 12, 19, 0, 9, 5, 10, counter);
    box_fill(&mut m, 14, 16, 0, 20, 9, 11, grey_dk);
    for t in 0..=15i32 {
        let x = 16 + t / 3;
        let z = 11 + t;
        for y in 2..=20i32 {
            let border = y == 2 || y == 20 || t == 0 || t == 15;
            let c = if border { grey_dk } else { grey };
            m.set(x as u32, y as u32, z as u32, Some(c));
            m.set((x + 1) as u32, y as u32, z as u32, Some(c));
        }
    }
    m
}

/// Trailer bed head: a ribbed metal panel fragment.
fn d2_bed_head(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 18, 32]);
    let metal = rgb(186, 189, 191);
    let metal_dk = rgb(140, 143, 146);
    box_fill(&mut m, 0, 10, 0, 17, 14, 15, metal);
    for x in [2u32, 5, 8] {
        box_fill(&mut m, x, x, 0, 17, 16, 16, metal_dk);
    }
    box_fill(&mut m, 0, 10, 4, 5, 16, 16, metal_dk);
    for x in 0..=10u32 {
        for y in 0..=16u32 {
            let px = *sprite.get_pixel(x, 17 - y);
            if opaque(px) {
                m.set(x, y, 16, Some(px));
            }
        }
    }
    m
}

/// Camp chair: X-frame sides, a fabric seat and
/// backrest.
fn d2_camp_chair() -> PropModel {
    let mut m = PropModel::new([32, 17, 32]);
    let frame = rgb(219, 219, 216);
    let fabric = rgb(206, 201, 195);
    for x in [11i32, 20] {
        beam(&mut m, x, 0, 10, x, 15, 19, frame);
        beam(&mut m, x, 0, 20, x, 9, 11, frame);
    }
    box_fill(&mut m, 11, 21, 8, 9, 11, 19, fabric);
    box_fill(&mut m, 11, 21, 13, 16, 18, 19, fabric);
    box_fill(&mut m, 11, 21, 8, 8, 11, 12, rgb(215, 180, 175));
    m
}

/// Stool: a round padded seat on four splayed legs
/// with a stretcher frame.
fn d2_stool() -> PropModel {
    let mut m = PropModel::new([32, 16, 32]);
    let wood = rgb(150, 105, 60);
    let wood_dk = rgb(112, 76, 40);
    for (sx, sz) in [(11i32, 10i32), (19, 10), (11, 20), (19, 20)] {
        let fx = if sx < 16 { sx - 2 } else { sx + 2 };
        let fz = if sz < 16 { sz - 2 } else { sz + 2 };
        beam(&mut m, fx, 0, fz, sx, 12, sz, wood);
    }
    box_fill(&mut m, 10, 21, 5, 5, 11, 11, wood_dk);
    box_fill(&mut m, 10, 21, 5, 5, 20, 20, wood_dk);
    box_fill(&mut m, 10, 10, 5, 5, 11, 20, wood_dk);
    box_fill(&mut m, 21, 21, 5, 5, 11, 20, wood_dk);
    cyl_y(&mut m, 16, 15, 6.2, 13, 14, wood_dk);
    cyl_y(&mut m, 16, 15, 6.2, 15, 15, rgb(96, 96, 99));
    m
}

fn manual_machinery_light(sprite: &RgbaImage) -> PropModel {
    let olive = rgb(140, 152, 66);
    let olive_dk = rgb(96, 106, 44);
    let pan_c = rgb(78, 88, 36);
    let brown = rgb(128, 98, 66);
    let steel = rgb(172, 174, 166);
    let steel_dk = rgb(110, 112, 104);
    let dark = rgb(38, 40, 30);
    let lens = rgb(238, 234, 196);
    let conduit = rgb(52, 54, 44);
    let mut m = PropModel::new([32, 62, 32]);
    // Stamp helper: paint a component's front layer
    // with the sprite's pixels (model y -> sprite row
    // 63 - y).
    let stamp = |m: &mut PropModel, x0: u32, x1: u32, y0: u32, y1: u32, z: u32| {
        for x in x0..=x1 {
            for y in y0..=y1 {
                let px = *sprite.get_pixel(x, 63 - y);
                if opaque(px) {
                    m.set(x, y, z, Some(px));
                }
            }
        }
    };
    // --- floor pan: slab + rim walls ----------------
    box_fill(&mut m, 6, 25, 0, 1, 6, 26, pan_c);
    for x in 6..=25 {
        for z in 6..=26 {
            if x == 6 || x == 25 || z == 6 || z == 26 {
                box_fill(&mut m, x, x, 2, 4, z, z, pan_c);
                m.set(x, 5, z, Some(olive_dk));
            }
        }
    }
    // --- legs (feet inside the pan) -----------------
    box_fill(&mut m, 8, 10, 2, 17, 20, 23, brown); // front-left
    stamp(&mut m, 8, 10, 5, 17, 23);
    box_fill(&mut m, 6, 8, 2, 17, 9, 12, rgb(96, 72, 46)); // back-left
    box_fill(&mut m, 22, 24, 2, 17, 20, 23, steel); // front-right
    stamp(&mut m, 22, 24, 5, 17, 23);
    box_fill(&mut m, 18, 20, 2, 17, 9, 12, steel_dk); // back-right
    // --- body block ---------------------------------
    box_fill(&mut m, 4, 27, 18, 34, 9, 23, olive);
    stamp(&mut m, 4, 27, 18, 34, 23);
    // Back face (z = 9): access panel seam, fasteners,
    // vent slots.
    for x in 6..=25 {
        m.set(x, 20, 9, Some(olive_dk));
        m.set(x, 32, 9, Some(olive_dk));
    }
    for y in 20..=32 {
        m.set(6, y, 9, Some(olive_dk));
        m.set(25, y, 9, Some(olive_dk));
    }
    for (x, y) in [(6, 20), (25, 20), (6, 32), (25, 32)] {
        m.set(x, y, 9, Some(dark));
    }
    for x in (8..=22).step_by(2) {
        m.set(x, 30, 9, Some(dark));
        m.set(x, 31, 9, Some(dark));
    }
    // Side details from the mockup: trim strip end on
    // the right flank's front-top, bright patch edge
    // on the left flank's front-top.
    box_fill(&mut m, 27, 27, 31, 34, 21, 23, steel);
    box_fill(&mut m, 4, 4, 30, 34, 21, 23, rgb(172, 184, 88));
    m.set(4, 32, 22, Some(lens));
    // --- bell housing (dome levels on the body) -----
    box_fill(&mut m, 16, 23, 35, 37, 10, 21, olive);
    stamp(&mut m, 16, 23, 35, 37, 21);
    box_fill(&mut m, 17, 22, 38, 39, 11, 20, olive);
    stamp(&mut m, 17, 22, 38, 39, 20);
    m.set(23, 35, 16, Some(rgb(66, 74, 30))); // side boss
    box_fill(&mut m, 18, 18, 40, 41, 15, 16, lens); // top peg
    // --- posts ---------------------------------------
    box_fill(&mut m, 4, 6, 34, 50, 13, 16, brown); // column
    stamp(&mut m, 4, 6, 34, 50, 16);
    box_fill(&mut m, 10, 10, 34, 50, 15, 16, steel); // rod 1
    stamp(&mut m, 10, 10, 34, 50, 16);
    box_fill(&mut m, 12, 13, 34, 50, 15, 16, steel); // rod 2
    stamp(&mut m, 12, 13, 34, 50, 16);
    // The bright fitting at the column's top, flush
    // with the posts' front plane.
    box_fill(&mut m, 5, 9, 47, 48, 13, 16, steel_dk);
    stamp(&mut m, 5, 9, 47, 48, 16);
    // Conduit: clipped behind the rods, landing on
    // the head's back and the body's back top.
    box_fill(&mut m, 11, 11, 35, 50, 13, 13, conduit);
    box_fill(&mut m, 11, 11, 51, 51, 9, 13, conduit);
    m.set(11, 34, 9, Some(conduit));
    // --- head ----------------------------------------
    box_fill(&mut m, 1, 15, 51, 58, 14, 24, olive);
    stamp(&mut m, 1, 15, 51, 58, 24);
    // Top posts as small clean knobs (their sprite
    // stamps include outline-black pixels that read
    // as a plaque at the front plane): a brown knob
    // with a pale cap at the left, a shorter steel
    // knob right of center.
    box_fill(&mut m, 2, 4, 59, 61, 20, 21, brown);
    box_fill(&mut m, 2, 4, 61, 61, 20, 21, rgb(216, 210, 186));
    box_fill(&mut m, 9, 12, 59, 60, 20, 21, steel_dk);
    m.set(10, 60, 21, Some(rgb(216, 210, 186)));
    // Lamp pods in profile: just the head's front
    // corner cells on each side face.
    for xf in [1u32, 15] {
        box_fill(&mut m, xf, xf, 53, 55, 23, 24, dark);
        m.set(xf, 54, 24, Some(lens));
    }
    // Ledge paint: every upward face that the tilted
    // front view can see takes the sprite row just
    // above its component (body top <- row 28, head
    // top <- row 4, housing steps <- olive_dk where
    // the row above is empty air).
    for x in 4..=27u32 {
        let px = *sprite.get_pixel(x, 28);
        for z in 9..=22u32 {
            if m.get(x, 34, z).is_some() {
                m.set(x, 34, z, Some(if opaque(px) { px } else { olive_dk }));
            }
        }
    }
    for x in 1..=15u32 {
        for z in 14..=23u32 {
            if m.get(x, 58, z).is_some() {
                m.set(x, 58, z, Some(olive_dk));
            }
        }
    }
    m
}

/// Ventilator (medical): caster base, column, bellows
/// box, and the main cabinet with its cyan screen on
/// the front face and a side arm. Tones: cabinet
/// (203..227 greys), teal base, screen cyan.
fn manual_ventilator() -> PropModel {
    let mut m = PropModel::new([32, 32, 32]);
    let cab = rgb(208, 208, 208);
    let cab_top = rgb(168, 168, 168);
    let teal = rgb(133, 178, 172);
    let dark = rgb(52, 54, 58);
    // Casters.
    for (x, z) in [(10u32, 12u32), (20, 12), (10, 19), (20, 19)] {
        box_fill(&mut m, x, x + 1, 0, 1, z, z + 1, dark);
    }
    // Base slab + column + bellows.
    box_fill(&mut m, 9, 23, 2, 4, 11, 21, teal);
    box_fill(&mut m, 14, 18, 5, 10, 14, 18, cab);
    box_fill(&mut m, 11, 21, 11, 15, 12, 20, rgb(118, 156, 150));
    box_fill(&mut m, 11, 21, 11, 12, 20, 20, dark);
    // Main cabinet y 16..29, top cap.
    box_fill(&mut m, 10, 23, 16, 28, 11, 20, cab);
    box_fill(&mut m, 10, 23, 29, 29, 11, 20, cab_top);
    // Screen on the front face (z = 20): cyan panel with
    // a dark surround, plus two button dots.
    box_fill(&mut m, 12, 19, 22, 26, 20, 20, rgb(24, 30, 48));
    box_fill(&mut m, 13, 18, 23, 25, 20, 20, rgb(16, 178, 198));
    box_fill(&mut m, 20, 21, 23, 24, 20, 20, rgb(240, 240, 240));
    box_fill(&mut m, 20, 21, 26, 26, 20, 20, rgb(240, 240, 240));
    // Side arm on the left with a knob.
    tube_x(&mut m, 5, 10, 24, 15, 0.8, dark);
    box_fill(&mut m, 4, 5, 23, 25, 14, 16, cab_top);
    m
}

/// Machinery light: an olive industrial machine -- base
/// cabinet on feet, a table slab, a motor block, two
/// posts, and the lamp head with two pale lamps. Greens
/// from the sprite ((157,169,73), (123,132,61),
/// (79,86,38)).
/// Ground cable (unconnected): two loose coils lying
/// flat -- a red ring and a green ring -- with two
/// white/yellow plugs at the red coil's edge. Reds
/// (122,15,15), green (26,85,0) from the sprite.
fn manual_ground_cable() -> PropModel {
    let mut m = PropModel::new([32, 4, 32]);
    let ring = |m: &mut PropModel, cx: i32, cz: i32, rad: f32, c: Rgba<u8>| {
        for x in 0..32 {
            for z in 0..32 {
                let d = (((x - cx).pow(2) + (z - cz).pow(2)) as f32).sqrt();
                if (d - rad).abs() <= 1.2 {
                    m.set(x as u32, 0, z as u32, Some(c));
                    m.set(x as u32, 1, z as u32, Some(c));
                }
            }
        }
    };
    ring(&mut m, 11, 12, 6.0, rgb(140, 26, 22));
    ring(&mut m, 23, 14, 4.6, rgb(32, 92, 8));
    // A red run joining the coils along the ground.
    box_fill(&mut m, 16, 19, 0, 1, 12, 13, rgb(140, 26, 22));
    // Two plugs at the red coil's lower edge.
    box_fill(&mut m, 5, 7, 0, 1, 19, 20, rgb(216, 216, 208));
    box_fill(&mut m, 6, 6, 0, 1, 21, 21, rgb(232, 192, 64));
    box_fill(&mut m, 12, 14, 0, 1, 20, 21, rgb(216, 216, 208));
    box_fill(&mut m, 13, 13, 0, 1, 22, 22, rgb(232, 192, 64));
    m
}

/// The opaque band as a segmentation, with none of
/// [`seamless_segment`]'s firing criteria — for rest-of-
/// props pins whose sprites are too thin, sparse, or
/// low-fill to pass them.
pub fn raw_segment(sprite: &RgbaImage) -> Option<Segmentation> {
    let (w, h) = (sprite.width(), sprite.height());
    if w < 4 || h < 4 {
        return None;
    }
    let band = opaque_band(sprite)?;
    if band.end - band.start < 3 {
        return None;
    }
    Some(Segmentation {
        silhouette_rows: band.clone(),
        top_rows: band.start..band.start + 1,
        front_rows: band,
        side: None,
        confidence: 0.5,
    })
}

pub fn coverage_model(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    // Hand-built models first (Joey's instruction,
    // 2026-11-17: the procedural builds for these still
    // looked wrong; edit the models manually).
    if let Some(m) = manual_model(id, sprite) {
        return Some(m);
    }
    if let Some(m) = d2_model(id, sprite) {
        return Some(m);
    }
    if let Some(m) = d3_model(id, sprite) {
        return Some(m);
    }
    if let Some(m) = veg_model(id, sprite) {
        return Some(m);
    }
    if let Some(m) = excl_model(id, sprite) {
        return Some(m);
    }
    if let Some((_, t)) = PLAN_SLAB_PINS.iter().find(|(k, _)| *k == id) {
        return plan_slab(sprite, *t);
    }
    if let Some((_, t)) = PANEL_SLAB_PINS.iter().find(|(k, _)| *k == id) {
        return panel_slab(sprite, *t);
    }
    if let Some((_, t)) = REST_SLAB_PINS.iter().find(|(k, _)| *k == id) {
        return plan_slab(sprite, *t);
    }
    if REST_EGG_PINS.contains(&id) {
        if let Some(seg) = raw_segment(sprite) {
            if let Some(m) = egg_pile_model(sprite, &seg) {
                return Some(m);
            }
        }
    }
    if REST_LATHE_PINS.contains(&id) {
        if let Some(seg) = raw_segment(sprite) {
            if let Some(m) = lathe_model(sprite, &seg) {
                return Some(m);
            }
        }
    }
    if id == "f_electrical_conduit" || id == "f_LIXA_electrical_conduit" {
        // The two conduit bundles keep the flat
        // extrusion: Joey reviewed the whole D1 group
        // (2026-11-17) and these were the only side
        // views he called OK -- a wall-run pipe bundle
        // reads correctly as a flat row.
        if let Some(m) = frame_model(sprite, 2) {
            return Some(m);
        }
    }
    if REST_MEMBER_PINS.contains(&id) {
        if let Some(m) = member_model(sprite) {
            return Some(m);
        }
    }
    if let Some((_, depth)) = REST_SOLID_DEPTH_PINS.iter().find(|(k, _)| *k == id) {
        if let Some(seg) = raw_segment(sprite) {
            let top = seamless_top_color(sprite, &seg);
            let construction = construction_for(Some(id), sprite, &seg);
            let mut m = solid_model_impl_d(
                sprite,
                &seg,
                construction,
                true,
                top,
                Some(*depth),
            );
            // Uniform-material pins (Joey, 2026-11-17, the
            // anvil: "a uniform shiny black metal material
            // all the way around"): repaint every voxel in
            // the sprite's modal structure tone — one
            // material, no mottling, no highlight stripe
            // carried around the sides.
            if REST_UNIFORM_PINS.contains(&id) {
                if let Some(c) = modal_color(
                    sprite
                        .pixels()
                        .copied()
                        .filter(|&px| structure(px)),
                ) {
                    for cell in m.voxels.iter_mut().flatten() {
                        *cell = c;
                    }
                }
            }
            return Some(m);
        }
    }
    if REST_PLANT_PINS.contains(&id) {
        if let Some(m) = plant_model(sprite) {
            return Some(m);
        }
    }
    if id == "f_birdbath" {
        if let Some(m) = birdbath_model(sprite) {
            return Some(m);
        }
    }
    if id == "f_orrery" {
        if let Some(m) = orrery_model(sprite) {
            return Some(m);
        }
    }
    if let Some(mode) = STAND_PINS.iter().find(|(k, _)| *k == id).map(|(_, m)| *m) {
        if let Some(m) = stand_model(sprite, mode) {
            return Some(m);
        }
    }
    if id == "f_small_satellite_dish" {
        if let Some(m) = dish_model(sprite) {
            return Some(m);
        }
    }
    if id == "f_hammock" || id == "f_hammock_net" {
        if let Some(m) = hammock_model(sprite, id == "f_hammock_net") {
            return Some(m);
        }
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
    if LATHE_PINS.contains(&id) {
        if let Some(m) = lathe_model(sprite, &seg) {
            return Some(m);
        }
    }
    if BLOB_PINS.contains(&id) {
        if let Some(m) = blob_model(sprite, &seg) {
            return Some(m);
        }
    }
    if EGG_PINS.contains(&id) {
        if let Some(m) = egg_pile_model(sprite, &seg) {
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

// ---------------------------------------------------------------------
// Terrain models (2026-11-17, Joey: "The fence and trees
// should also be models"). Terrain ids whose coarse rendering
// is a textured cube get real prop models, emitted through
// the same v2 models table as furniture; the engine stamps
// them at terrain tiles in place of the block.

pub fn terrain_model_for(id: &str, sprite: &RgbaImage) -> Option<PropModel> {
    if !id.starts_with("t_") {
        return None;
    }
    match id {
        "t_tree" => return Some(terrain_tree(sprite)),
        "t_chainfence" => return Some(terrain_chainfence(sprite)),
        _ if id.starts_with("t_chainfence_season") => {
            return Some(terrain_chainfence(sprite))
        }
        _ => {}
    }
    // Terrain drive (2026-11-17, Joey: models for every
    // tile, all seasons): family routing by id pattern;
    // season variants are separate ids and route with
    // their family, each built from its own sprite. The
    // silhouette builder is the generic fallback, so
    // every terrain id with a sprite gets a model; the
    // per-family batches then refine from the strips.
    let s = &id[2..];
    if s.contains("fencegate") || s.contains("door") {
        if s.contains("fencegate") && !s.contains("privacy") {
            // A gate in a fence line is a fence segment,
            // not a door: panel builder at fence height.
            return Some(terrain_fence_panel(sprite));
        }
        let open = s.contains("open") || s.ends_with("_o") || s.contains("_o_");
        let style = if s.ends_with("frame") {
            DoorStyle::Frame
        } else if s.contains("curtain")
            || s.contains("fencegate")
            || s.split('_').any(|seg| seg == "b")
        {
            // Broken doors, curtains and open privacy
            // gates: the sprite's own silhouette IS the
            // panel (holes, parted curtains, gate halves
            // at the frame edges) -- a filled modal slab
            // invents door that is not there.
            DoorStyle::Silhouette
        } else if open {
            DoorStyle::Open
        } else {
            DoorStyle::Closed
        };
        return Some(terrain_door(sprite, style));
    }
    if s.contains("tree")
        || s.contains("shrub")
        || s.starts_with("bamboo")
        || s.contains("fern")
        || s.contains("sapling")
        || s.contains("underbrush")
        || s.contains("azalea")
        || s.contains("lilac")
        || s.contains("forsythia")
        || s.contains("pine")
    {
        if s.starts_with("bamboo") {
            return Some(bamboo_model(sprite));
        }
        if s.contains("fungal") {
            // A fungal young tree is a solid pale
            // spire (bulbous head, narrow waist, flared
            // base) -- per-layer discs, not branches.
            return Some(disc_model(sprite, 76, false));
        }
        if s.contains("fern") {
            return Some(fern_model(sprite));
        }
        let base = if s.contains("tree") || s.contains("pine") {
            76
        } else if s.contains("underbrush") {
            26
        } else if s.contains("shrub") {
            40
        } else {
            52
        };
        return Some(vegetation_model(sprite, base));
    }
    if s.contains("fence") || s.contains("splitrail") || s.contains("barbed") || s.contains("railing") {
        return Some(terrain_fence_panel(sprite));
    }
    if s.contains("window") {
        // Empty windows and bare frames have no pane:
        // the builder's glass plane would fill the
        // opening with the sprite's interior colour.
        let pane = !(s.contains("empty") || s.ends_with("frame"));
        return Some(terrain_window(sprite, pane));
    }
    if s.contains("wall") {
        let h = if s.contains("half") { 48 } else { 96 };
        return Some(terrain_box(sprite, h));
    }
    if s.contains("stairs") || s.contains("ladder") || s.starts_with("ramp") {
        return Some(terrain_stairs(sprite, s.contains("down")));
    }
    // "water" objects are not liquid terrain: a water
    // cooler, a hand pump and a solid water cube are
    // objects that happen to contain the word.
    let water_object = s.contains("water_cooler")
        || s.contains("water_pump")
        || s.contains("water_cube");
    let liquid = (s.contains("water") && !water_object)
        || s.contains("lava")
        || s.contains("sewage")
        || s.starts_with("pool")
        || s.contains("acid")
        || s.contains("ice");
    let ground = liquid
        || s.starts_with("grass")
        || s.starts_with("dirt")
        || s.starts_with("sand")
        || s.starts_with("pavement")
        || s.starts_with("floor")
        || s.starts_with("soil")
        || s.starts_with("snow")
        || s.starts_with("road")
        || s.starts_with("sidewalk")
        || s.starts_with("bridge")
        || s.starts_with("gravel")
        || s.starts_with("rubble")
        || s.starts_with("moss")
        || s.starts_with("puddle")
        || s.contains("_floor")
        || s.contains("linoleum")
        || s.contains("carpet")
        || s.starts_with("rock_floor")
        || s == "rock"
        || s.starts_with("slime")
        || s == "clay"
        || s == "ash";
    if ground {
        let mound = s.contains("mound");
        return Some(terrain_ground(sprite, liquid, mound));
    }
    terrain_silhouette(sprite)
}

/// Sparse-stroke vegetation (bamboo and loose
/// thickets): one airy tuft per leaf-stroke cluster.
/// Clusters are 8-connected components of the opaque
/// mask after a 1px dilation (a spray's strokes sit
/// 1-2px apart). Each tuft is a thin irregular ellipse
/// at the cluster's sprite height, textured from the
/// cluster's own pixels, rim cells dropped by hash so
/// it stays see-through; a radius-1 stem in darkened
/// cluster colour grounds it (the sprite crops the
/// thicket's stems out of frame; a floating tuft
/// reads worse than the implied stem).
fn thicket_model(sprite: &RgbaImage, h: i32) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut m = PropModel::new([32, h as u32, 32]);
    let opaque = |x: i32, y: i32| -> bool {
        x >= 0
            && y >= 0
            && x < sw
            && y < sh
            && sprite.get_pixel(x as u32, y as u32)[3] >= 128
    };
    // Dilated mask for clustering.
    let mut near = vec![false; (sw * sh) as usize];
    for y in 0..sh {
        for x in 0..sw {
            if opaque(x, y) {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx >= 0 && ny >= 0 && nx < sw && ny < sh {
                            near[(ny * sw + nx) as usize] = true;
                        }
                    }
                }
            }
        }
    }
    let mut seen = vec![false; (sw * sh) as usize];
    for sy in 0..sh {
        for sx in 0..sw {
            let idx = (sy * sw + sx) as usize;
            if !near[idx] || seen[idx] {
                continue;
            }
            // Flood-fill one cluster over the dilated
            // mask, collecting its true opaque pixels.
            let mut stack = vec![(sx, sy)];
            seen[idx] = true;
            let mut pixels: Vec<(i32, i32)> = Vec::new();
            while let Some((x, y)) = stack.pop() {
                if opaque(x, y) {
                    pixels.push((x, y));
                }
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= sw || ny >= sh {
                            continue;
                        }
                        let ni = (ny * sw + nx) as usize;
                        if near[ni] && !seen[ni] {
                            seen[ni] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
            if pixels.len() < 2 {
                continue;
            }
            let (mut x0, mut x1, mut r0, mut r1) = (sw, -1, sh, -1);
            for &(x, r) in &pixels {
                x0 = x0.min(x);
                x1 = x1.max(x);
                r0 = r0.min(r);
                r1 = r1.max(r);
            }
            let cx_px = (x0 + x1) as f32 / 2.0;
            let rc = (r0 + r1) as f32 / 2.0;
            let cx = (cx_px * 32.0 / sw as f32) as i32;
            let cz = 16 + ((cx_px - sw as f32 / 2.0) * 0.3) as i32;
            let yc = (((sh as f32 - 1.0 - rc) * h as f32 / sh as f32) as i32).clamp(1, h - 1);
            let rx = (((x1 - x0 + 1) as f32 * 32.0 / sw as f32) / 2.0).max(2.0);
            let rz = (rx * 0.62).max(2.0);
            let modal = modal_color(pixels.iter().map(|&(x, r)| *sprite.get_pixel(x as u32, r as u32)))
                .unwrap_or(rgb(70, 130, 60));
            // Stem: ground to tuft, darkened cluster
            // colour.
            if yc > 4 {
                let stem = shade(modal, 0.62);
                for y in 0..yc {
                    for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (vx, vz) = (cx + dx, cz + dz);
                        if (0..32).contains(&vx) && (0..32).contains(&vz) {
                            m.set(vx as u32, y as u32, vz as u32, Some(stem));
                        }
                    }
                }
            }
            // Tuft: two thin layers, airy rim.
            for layer in 0..2i32 {
                let y = yc - layer;
                if y < 0 {
                    continue;
                }
                let (lx, lz) = (rx.ceil() as i32, rz.ceil() as i32);
                for dz in -lz..=lz {
                    for dx in -lx..=lx {
                        let d = ((dx as f32 / rx).powi(2) + (dz as f32 / rz).powi(2)).sqrt();
                        if d > 1.0 {
                            continue;
                        }
                        if d > 0.72 && hash01((cx + dx) as u32, (y * 7 + dz) as u32) > 0.62 {
                            continue; // airy rim
                        }
                        let (vx, vz) = (cx + dx, cz + dz);
                        if !(0..32).contains(&vx) || !(0..32).contains(&vz) {
                            continue;
                        }
                        // Texture from the cluster's own
                        // pixels at the mapped position.
                        let spx = x0 + ((vx as f32 / 32.0 * sw as f32) as i32 - x0).clamp(0, x1 - x0);
                        let spr = r0 + ((dz as f32 / rz.max(1.0) * (r1 - r0) as f32 / 2.0) as i32).clamp(0, r1 - r0);
                        let p = *sprite.get_pixel(spx as u32, spr as u32);
                        let c = if p[3] >= 128 { p } else { modal };
                        m.set(vx as u32, y as u32, vz as u32, Some(c));
                    }
                }
            }
        }
    }
    m
}

/// A bare tree / dormant shrub: just branches. Each
/// sprite row's opaque RUNS are individual branches
/// (a row crossing three branches has three runs);
/// each run becomes a thin disc (radius ~1.3) at the
/// run's centre, fanned in depth proportional to its
/// distance from the trunk column, so the branch fan
/// reads in 3D from every side with air between the
/// branches. The run containing the trunk column in
/// the lower half is the trunk: thicker, tapering.
/// Bamboo: a stand of culms (vertical banded stalks)
/// with leaf tufts. Culm columns are sprite columns
/// opaque over most of the plant's height; adjacent
/// ones cluster into stalks. Leaf pixels away from
/// the culms become thicket tufts. Sprites with no
/// culm columns at all (the base bamboo is pure leaf
/// strokes over grass) fall back to the thicket.
fn bamboo_model(sprite: &RgbaImage) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let (mut top, mut bottom) = (sh, -1);
    for r in 0..sh {
        if (0..sw).any(|x| sprite.get_pixel(x as u32, r as u32)[3] >= 128) {
            top = top.min(r);
            bottom = bottom.max(r);
        }
    }
    if bottom < 0 {
        return PropModel::new([32, 10, 32]);
    }
    let frac = (bottom - top + 1) as f32 / sh as f32;
    let h = ((48.0 * frac) as i32).max(10);
    let rows = (bottom - top + 1).max(1);
    // Culm columns: opaque over >= 55% of the height.
    let mut culm_col = vec![false; sw as usize];
    for x in 0..sw {
        let n = (top..=bottom)
            .filter(|&r| sprite.get_pixel(x as u32, r as u32)[3] >= 128)
            .count();
        culm_col[x as usize] = (n as f32) >= 0.45 * (rows as f32);
    }
    // Cluster culm columns (gaps of <= 2 merged).
    let mut clusters: Vec<Vec<i32>> = Vec::new();
    let mut x = 0;
    while x < sw {
        if culm_col[x as usize] {
            let mut cl = vec![x];
            let mut xx = x + 1;
            let mut gap = 0;
            while xx < sw && gap <= 2 {
                if culm_col[xx as usize] {
                    cl.push(xx);
                    gap = 0;
                } else {
                    gap += 1;
                }
                xx += 1;
            }
            clusters.push(cl);
            x = xx;
        } else {
            x += 1;
        }
    }
    if clusters.is_empty() {
        return thicket_model(sprite, h);
    }
    // Leaf tufts from a sprite copy with the culm
    // columns (dilated by 1) blanked out.
    let mut masked = sprite.clone();
    for cl in &clusters {
        for &cx in cl {
            for dx in -1..=1 {
                let xx = cx + dx;
                if xx >= 0 && xx < sw {
                    for r in 0..sh {
                        masked.put_pixel(xx as u32, r as u32, Rgba([0, 0, 0, 0]));
                    }
                }
            }
        }
    }
    let mut m = thicket_model(&masked, h);
    // Culms drawn over the tufts.
    let culm_pal = palette_of(
        (0..sw)
            .filter(|&x| culm_col[x as usize])
            .flat_map(|x| (top..=bottom).map(move |r| (x, r)))
            .map(|(x, r)| *sprite.get_pixel(x as u32, r as u32))
            .filter(|p| p[3] >= 128),
        10,
    );
    for cl in &clusters {
        let cx_px = cl[cl.len() / 2];
        let cx = cx_px * 32 / sw;
        let z_off = ((cx_px - sw / 2) as f32 * 0.3) as i32;
        // Culm top: the tallest column of the
        // cluster (the centre column may be a
        // broken stalk -- the winter dead bamboo
        // rendered as ground stubs).
        let ctop = cl
            .iter()
            .filter_map(|&cx| {
                (top..=bottom).find(|&r| sprite.get_pixel(cx as u32, r as u32)[3] >= 128)
            })
            .min()
            .unwrap_or(top);
        let top_y = (sh - 1 - ctop) * h / sh;
        for y in 0..=top_y.min(h - 1) {
            let r = (h - 1 - y) * sh / h;
            let mut c = *sprite.get_pixel(cx_px as u32, r as u32);
            if c[3] < 128 {
                c = pick(&culm_pal, hash01(cx as u32, y as u32));
            }
            for dz in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx * dx + dz * dz > 2 {
                        continue;
                    }
                    let (vx, vz) = (cx + dx, 16 + z_off + dz);
                    if vx < 0 || vx >= 32 || vz < 0 || vz >= 32 {
                        continue;
                    }
                    m.set(vx as u32, y as u32, vz as u32, Some(c));
                }
            }
        }
    }
    m
}

/// Fern: a fan of fronds from a ground point -- the
/// sprite's silhouette as a two-plane cross (the
/// construction Joey accepted for furniture plants).
/// Discs filled the fan into a lumpy blob with black
/// gaps.
fn fern_model(sprite: &RgbaImage) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let (mut top, mut bottom) = (sh, -1);
    for r in 0..sh {
        if (0..sw).any(|x| sprite.get_pixel(x as u32, r as u32)[3] >= 128) {
            top = top.min(r);
            bottom = bottom.max(r);
        }
    }
    if bottom < 0 {
        return PropModel::new([32, 8, 32]);
    }
    let frac = (bottom - top + 1) as f32 / sh as f32;
    let h = ((40.0 * frac) as i32).clamp(8, 36);
    let mut m = PropModel::new([32, h as u32, 32]);
    for r in 0..sh {
        let y = (sh - 1 - r) * h / sh;
        for x in 0..sw {
            let p = *sprite.get_pixel(x as u32, r as u32);
            if p[3] < 128 {
                continue;
            }
            let c = x * 32 / sw;
            for dz in 15..=16i32 {
                m.set(c as u32, y as u32, dz as u32, Some(p));
            }
            for dx in 15..=16i32 {
                m.set(dx as u32, y as u32, c as u32, Some(p));
            }
        }
    }
    m
}

fn branch_model(sprite: &RgbaImage, h: i32, spans: &[Option<(i32, i32)>]) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut m = PropModel::new([32, h as u32, 32]);
    // Trunk column: median span centre of the bottom
    // third of the sprite.
    let mut centres: Vec<i32> = (0..sh)
        .filter(|&r| r >= sh * 2 / 3)
        .filter_map(|r| spans[r as usize].map(|(a, b)| (a + b) / 2))
        .collect();
    centres.sort();
    let trunk_cx_px = if centres.is_empty() {
        sw / 2
    } else {
        centres[centres.len() / 2]
    };
    let barks = palette_of(sprite.pixels().copied().filter(|&p| p[3] >= 128), 12);
    for y in 0..h {
        let r = (h - 1 - y) * sh / h;
        // Opaque runs in this row (1px gaps merged:
        // anti-aliased branch strokes break otherwise).
        let mut runs: Vec<(i32, i32)> = Vec::new();
        let mut x = 0;
        while x < sw {
            if sprite.get_pixel(x as u32, r as u32)[3] >= 128 {
                let a = x;
                let mut b = x;
                let mut gap = 0;
                let mut xx = x + 1;
                while xx < sw {
                    if sprite.get_pixel(xx as u32, r as u32)[3] >= 128 {
                        b = xx;
                        gap = 0;
                    } else {
                        gap += 1;
                        if gap > 1 {
                            break;
                        }
                    }
                    xx += 1;
                }
                runs.push((a, b));
                x = b + 1;
            } else {
                x += 1;
            }
        }
        for (a, b) in runs {
            let is_trunk = (a <= trunk_cx_px && trunk_cx_px <= b) && r >= sh / 2;
            // A wide run is a horizontal limb (or a
            // fungal shelf): step discs along it as a
            // tube instead of one disc at its centre
            // (stacks of centred discs read as totems).
            let run_cells = ((b - a + 1) * 32 / sw).max(1);
            let steps = if is_trunk { 1 } else { (run_cells / 3).max(1) };
            for i in 0..steps {
                let cx_px = if steps == 1 {
                    (a + b) / 2
                } else {
                    a + (b - a) * (2 * i + 1) / (2 * steps)
                };
                let rad = if is_trunk {
                    // Taper: 3.2 at the ground to 1.5 at
                    // the trunk's top.
                    3.2 - 1.7 * (y as f32 / h as f32)
                } else {
                    1.3
                };
                let cx = cx_px * 32 / sw;
                let z_off = ((cx_px - trunk_cx_px) as f32 * 32.0 / sw as f32 * 0.55) as i32;
                let cz = 16 + z_off;
                let ri = rad.ceil() as i32;
                for dz in -ri..=ri {
                    for dx in -ri..=ri {
                        let d = ((dx * dx + dz * dz) as f32).sqrt();
                        if d > rad {
                            continue;
                        }
                        let (vx, vz) = (cx + dx, cz + dz);
                        if vx < 0 || vx >= 32 || vz < 0 || vz >= 32 {
                            continue;
                        }
                        let p = *sprite.get_pixel(cx_px as u32, r as u32);
                        let c = if p[3] >= 128 {
                            p
                        } else {
                            pick(&barks, hash01(vx as u32, y as u32))
                        };
                        m.set(vx as u32, y as u32, vz as u32, Some(c));
                    }
                }
            }
        }
    }
    m
}

/// Vegetation (trees, shrubs, plants) by STRUCTURE,
/// not colour: autumn foliage and bark share a palette,
/// so colour classification collapses the crown (the
/// v1 autumn tree came out a narrow column). Instead:
/// rows wider than half the widest row are CROWN --
/// discs follow the full opaque span, painted from the
/// sprite's own pixels; rows below the crown are TRUNK
/// -- narrow discs in the trunk-zone palette. Crown
/// depth is squashed by leafiness (non-bark-coloured
/// fraction): a leafy crown is nearly round, a bare
/// winter tree is a thin branch silhouette. A thin core
/// disc at the trunk column runs through the crown so
/// limbs read inside the foliage.
/// Crop an oversize canvas (96x96 renders of a
/// tile-filling subject) to its opaque bbox and
/// resample to tile width, so spans and heights
/// measure the SUBJECT, not the canvas margins (the
/// 96px pine measured 13 cells wide and rendered as
/// a pencil).
fn normalize_oversize(sprite: &RgbaImage) -> RgbaImage {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let (mut x0, mut y0, mut x1, mut y1) = (sw, sh, -1, -1);
    for r in 0..sh {
        for x in 0..sw {
            if sprite.get_pixel(x as u32, r as u32)[3] >= 128 {
                x0 = x0.min(x);
                y0 = y0.min(r);
                x1 = x1.max(x);
                y1 = y1.max(r);
            }
        }
    }
    if x1 < 0 {
        return sprite.clone();
    }
    let (bw, bh) = ((x1 - x0 + 1).max(1), (y1 - y0 + 1).max(1));
    let crop = image::imageops::crop_imm(sprite, x0 as u32, y0 as u32, bw as u32, bh as u32)
        .to_image();
    let nh = ((32 * bh / bw).clamp(8, 96)) as u32;
    image::imageops::resize(&crop, 32, nh, image::imageops::FilterType::Nearest)
}

fn vegetation_model(sprite: &RgbaImage, base_h: i32) -> PropModel {
    let normalized;
    let sprite = if sprite.width() > 48 || sprite.height() > 96 {
        normalized = normalize_oversize(sprite);
        &normalized
    } else {
        sprite
    };
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    // Per-row opaque spans + bbox.
    let mut spans: Vec<Option<(i32, i32)>> = Vec::with_capacity(sh as usize);
    let (mut top, mut bottom) = (sh, -1);
    let mut max_w = 0i32;
    for r in 0..sh {
        let mut span: Option<(i32, i32)> = None;
        for x in 0..sw {
            let p = sprite.get_pixel(x as u32, r as u32);
            // Drop shadows are semi-transparent black:
            // they are not part of the plant.
            let shadow = p[3] < 250 && p[0] < 40 && p[1] < 40 && p[2] < 40;
            if p[3] >= 128 && !shadow {
                span = Some(match span {
                    None => (x, x),
                    Some((a, b)) => (a.min(x), b.max(x)),
                });
            }
        }
        if let Some((a, b)) = span {
            max_w = max_w.max(b - a + 1);
            top = top.min(r);
            bottom = bottom.max(r);
        }
        spans.push(span);
    }
    let frac = if bottom >= 0 {
        (bottom - top + 1) as f32 / sh as f32
    } else {
        1.0
    };
    let mut h = ((base_h as f32 * frac) as i32).max(10);
    if base_h <= 44 && max_w > 0 {
        // Shrub class: cap the height near the crown
        // width -- several berry shrubs rendered as
        // tall cans.
        let w_cells = max_w * 32 / sw;
        h = h.min((w_cells as f32 * 1.15) as i32).max(10);
    }
    let mut m = PropModel::new([32, h as u32, 32]);
    if bottom < 0 || max_w == 0 {
        return m;
    }
    // Bare case (winter / dead): almost no leaf-coloured
    // pixels -- the sprite IS a branch drawing, so build
    // branches, not a filled silhouette (Joey's catch on
    // the winter tree: "It should just be branches").
    {
        let opaque_n = sprite.pixels().filter(|p| p[3] >= 128).count();
        let leafy_n = sprite
            .pixels()
            .filter(|p| looks_like_foliage(**p))
            .count();
        if opaque_n > 0 && (leafy_n as f32) < 0.15 * (opaque_n as f32) {
            return branch_model(sprite, h, &spans);
        }
        // Tree-class with a minority of leaf colour:
        // branch structure carrying sparse leaves
        // (harvested orchard trees, dead trees, winter
        // fruit trees all rendered as puffballs or
        // grey clouds before this route).
        if base_h >= 60
            && opaque_n > 0
            && (leafy_n as f32) < 0.45 * (opaque_n as f32)
        {
            return branch_model(sprite, h, &spans);
        }
        // Sparse-stroke case (bamboo, loose thickets):
        // the sprite is separate leaf strokes with air
        // between them, so filled crown discs read as
        // stacked lollipops (Joey on the bamboo sample:
        // "I can't even tell what row 5 is supposed to
        // be"). Build one airy tuft per stroke cluster.
        let bbox_rows = (bottom - top + 1).max(1);
        let coverage = opaque_n as f32 / (bbox_rows * max_w.max(1)) as f32;
        if opaque_n > 0 && coverage < 0.22 {
            return thicket_model(sprite, h);
        }
    }
    // Crown = rows wider than half the widest row;
    // trunk zone = rows below the lowest crown row.
    let crown_bottom = (0..sh)
        .rev()
        .find(|&r| spans[r as usize].map_or(false, |(a, b)| b - a + 1 > max_w / 2))
        .unwrap_or(-1);
    // Trunk column: median span centre of the trunk zone
    // (bbox centre when there is no trunk zone: a shrub).
    let mut centres: Vec<i32> = (0..sh)
        .filter(|&r| r > crown_bottom)
        .filter_map(|r| spans[r as usize].map(|(a, b)| (a + b) / 2))
        .collect();
    centres.sort();
    let trunk_cx_px = if centres.is_empty() {
        sw / 2
    } else {
        centres[centres.len() / 2]
    };
    let trunk_cx = trunk_cx_px * 32 / sw;
    // Leafiness from the loose colour test (paint/shape
    // hint only -- never a shape gate).
    let opaque_n = sprite.pixels().filter(|p| p[3] >= 128).count();
    let leafy_n = sprite
        .pixels()
        .filter(|p| looks_like_foliage(**p))
        .count();
    let leafiness = if opaque_n > 0 {
        leafy_n as f32 / opaque_n as f32
    } else {
        1.0
    };
    let depth_factor = 0.35 + 0.65 * leafiness;
    // Palettes: crown from ALL opaque pixels (leaves and
    // twigs together); trunk from bark-coloured pixels
    // in the trunk zone.
    let full_pal = palette_of(sprite.pixels().copied().filter(|&p| p[3] >= 128), 24);
    let dark = pick(&full_pal, 0.85);
    let trunk_pal = palette_of(
        (0..sh)
            .filter(|&r| r > crown_bottom)
            .flat_map(|r| (0..sw).map(move |x| (x, r)))
            .map(|(x, r)| *sprite.get_pixel(x as u32, r as u32))
            .filter(|&p| is_bark(p)),
        8,
    );
    let trunk_col = if trunk_pal.is_empty() {
        dark
    } else {
        pick(&trunk_pal, 0.3)
    };
    for y in 0..h {
        let r = (h - 1 - y) * sh / h;
        let Some((x0, x1)) = spans[r as usize] else {
            continue;
        };
        let cx = ((x0 + x1) / 2) * 32 / sw;
        let span_rad = ((x1 - x0 + 1) * 32 / sw) as f32 / 2.0;
        if r > crown_bottom {
            // Trunk zone: one narrow disc.
            let rad = span_rad.clamp(1.2, 4.5);
            let ri = rad.ceil() as i32;
            for dz in -ri..=ri {
                for dx in -ri..=ri {
                    let d = ((dx * dx + dz * dz) as f32).sqrt();
                    if d > rad + hash01((dx + ri) as u32, (y * 7) as u32) * 0.4 {
                        continue;
                    }
                    let (x, z) = (cx + dx, 16 + dz);
                    if x < 0 || x >= 32 || z < 0 || z >= 32 {
                        continue;
                    }
                    let c = if trunk_pal.is_empty() {
                        trunk_col
                    } else {
                        pick(&trunk_pal, hash01(x as u32, (y * 3 + z) as u32))
                    };
                    m.set(x as u32, y as u32, z as u32, Some(c));
                }
            }
        } else {
            // Crown: elliptical disc (depth squashed by
            // leafiness), front shell = the sprite's own
            // pixels snapped to the full palette. The
            // top fifth tapers in, so the crown rounds
            // off instead of ending in a flat full-width
            // "lid" disc (every berry shrub wore one).
            let taper = if y > h * 4 / 5 {
                let t = (y - h * 4 / 5) as f32 / ((h - h * 4 / 5).max(1)) as f32;
                1.0 - 0.55 * t
            } else {
                1.0
            };
            let rad_x = (span_rad * taper).max(1.2);
            let rad_z = (span_rad * depth_factor * taper).max(1.0);
            let ri_x = rad_x.ceil() as i32;
            let ri_z = rad_z.ceil() as i32;
            for dz in -ri_z..=ri_z {
                for dx in -ri_x..=ri_x {
                    let e = ((dx as f32 / rad_x).powi(2) + (dz as f32 / rad_z).powi(2)).sqrt();
                    let edge = 1.0 + (hash01((dx + ri_x) as u32, (y * 31 + dz) as u32) - 0.5) * 0.14;
                    if e > edge {
                        continue;
                    }
                    let (x, z) = (cx + dx, 16 + dz);
                    if x < 0 || x >= 32 || z < 0 || z >= 32 {
                        continue;
                    }
                    let c = if dz > 0 && e > 0.82 {
                        let sx = (x * sw / 32).clamp(0, sw - 1);
                        let p = *sprite.get_pixel(sx as u32, r as u32);
                        if p[3] >= 128 {
                            *full_pal
                                .iter()
                                .min_by_key(|g| {
                                    (g[0] as i32 - p[0] as i32).abs()
                                        + (g[1] as i32 - p[1] as i32).abs()
                                        + (g[2] as i32 - p[2] as i32).abs()
                                })
                                .unwrap_or(&dark)
                        } else {
                            pick(&full_pal, hash01(x as u32, y as u32))
                        }
                    } else if e > 0.82 {
                        pick(&full_pal, hash01((x * 5 + z) as u32, (y * 11) as u32))
                    } else {
                        dark
                    };
                    m.set(x as u32, y as u32, z as u32, Some(c));
                }
            }
            // Limb core through the crown at the trunk
            // column.
            for dz in -2..=2i32 {
                for dx in -2..=2i32 {
                    if dx * dx + dz * dz > 4 {
                        continue;
                    }
                    let (x, z) = (trunk_cx + dx, 16 + dz);
                    if x < 0 || x >= 32 || z < 0 || z >= 32 {
                        continue;
                    }
                    m.set(x as u32, y as u32, z as u32, Some(trunk_col));
                }
            }
        }
    }
    m
}

/// Per-layer discs from the sprite's row spans: the
/// unified-solid vegetation / blob form. `base_h` is
/// scaled by the sprite's opaque bbox fraction, so
/// harvested and winter variants shrink with their
/// drawings. With `bark_pass`, bark-classified spans
/// draw over the canopy as trunk / limbs / branches
/// (a dead tree is then all bark: bare branches).
fn disc_model(sprite: &RgbaImage, base_h: i32, bark_pass: bool) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut top = sh;
    let mut bottom = -1;
    for r in 0..sh {
        if (0..sw).any(|x| sprite.get_pixel(x as u32, r as u32)[3] >= 128) {
            top = top.min(r);
            bottom = bottom.max(r);
        }
    }
    let frac = if bottom >= 0 {
        (bottom - top + 1) as f32 / sh as f32
    } else {
        1.0
    };
    let h = ((base_h as f32 * frac) as i32).max(10);
    let mut m = PropModel::new([32, h as u32, 32]);
    let canopy_pal = palette_of(
        sprite
            .pixels()
            .copied()
            .filter(|&p| p[3] >= 128 && !(bark_pass && is_bark(p))),
        20,
    );
    let dark = pick(&canopy_pal, 0.85);
    let row_span = |r: i32, test: &dyn Fn(Rgba<u8>) -> bool| -> Option<(i32, i32)> {
        let mut span: Option<(i32, i32)> = None;
        for x in 0..sw {
            if test(*sprite.get_pixel(x as u32, r as u32)) {
                span = Some(match span {
                    None => (x, x),
                    Some((a, b)) => (a.min(x), b.max(x)),
                });
            }
        }
        span
    };
    for y in 0..h {
        let r = (h - 1 - y) * sh / h;
        if let Some((x0, x1)) = row_span(r, &|p| p[3] >= 128 && !(bark_pass && is_bark(p))) {
            let cx = ((x0 + x1) / 2) * 32 / sw;
            let rad = ((x1 - x0 + 1) * 32 / sw) as f32 / 2.0;
            let ri = rad.ceil() as i32;
            for dz in -ri..=ri {
                for dx in -ri..=ri {
                    let d = ((dx * dx + dz * dz) as f32).sqrt();
                    let edge = rad + (hash01((dx + ri) as u32, (y * 31 + dz) as u32) - 0.5) * 1.6;
                    if d > edge {
                        continue;
                    }
                    let (x, z) = (cx + dx, 16 + dz);
                    if x < 0 || x >= 32 || z < 0 || z >= 32 {
                        continue;
                    }
                    let c = if dz > 0 && d > rad - 1.2 {
                        let sx = (x * sw / 32).clamp(0, sw - 1);
                        let p = *sprite.get_pixel(sx as u32, r as u32);
                        if p[3] >= 128 && !(bark_pass && is_bark(p)) {
                            *canopy_pal
                                .iter()
                                .min_by_key(|g| {
                                    (g[0] as i32 - p[0] as i32).abs()
                                        + (g[1] as i32 - p[1] as i32).abs()
                                        + (g[2] as i32 - p[2] as i32).abs()
                                })
                                .unwrap_or(&dark)
                        } else {
                            pick(&canopy_pal, hash01(x as u32, y as u32))
                        }
                    } else if d > rad - 1.2 {
                        pick(&canopy_pal, hash01((x * 5 + z) as u32, (y * 11) as u32))
                    } else {
                        dark
                    };
                    m.set(x as u32, y as u32, z as u32, Some(c));
                }
            }
        }
    }
    if bark_pass {
        let barks = palette_of(sprite.pixels().copied().filter(|&p| is_bark(p)), 10);
        // Bare-tree case (winter / dead): if canopy-class
        // pixels are a small minority of the opaque ones,
        // the branches are the whole silhouette -- the
        // bark pass then follows the span radius instead
        // of the thin limb cap.
        let opaque_n = sprite.pixels().filter(|p| p[3] >= 128).count();
        let canopy_n = sprite
            .pixels()
            .filter(|p| p[3] >= 128 && !is_bark(**p))
            .count();
        let bare = opaque_n > 0 && (canopy_n as f32) < 0.15 * (opaque_n as f32);
        if !barks.is_empty() {
            for y in 0..h {
                let r = (h - 1 - y) * sh / h;
                let Some((x0, x1)) = row_span(r, &|p| is_bark(p)) else {
                    continue;
                };
                let cx = ((x0 + x1) / 2) * 32 / sw;
                let span_rad = ((x1 - x0 + 1) * 32 / sw) as f32 / 2.0;
                let cap = if y <= h * 2 / 5 { 3.2 } else { 1.8 };
                let rad = if bare {
                    span_rad.max(1.2)
                } else {
                    span_rad.clamp(1.2, cap)
                };
                let ri = rad.ceil() as i32;
                for dz in -ri..=ri {
                    for dx in -ri..=ri {
                        let d = ((dx * dx + dz * dz) as f32).sqrt();
                        if d > rad + hash01((x0 + dx) as u32, (y * 7) as u32) * 0.4 {
                            continue;
                        }
                        let (x, z) = (cx + dx, 16 + dz);
                        if x < 0 || x >= 32 || z < 0 || z >= 32 {
                            continue;
                        }
                        let c = pick(&barks, hash01(x as u32, (y * 3 + z) as u32));
                        m.set(x as u32, y as u32, z as u32, Some(c));
                    }
                }
            }
        }
    }
    m
}

/// A fence half-segment from the sprite's own left
/// half: the FG silhouette extruded on the tile's
/// centre planes, with full-height posts at the edge
/// and centre (in the sprite's modal colour). Picket,
/// rail and solid fences all come from their drawings;
/// the engine composes directions like t_chainfence.
fn terrain_fence_panel(sprite: &RgbaImage) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let h = if sh >= 64 { 56 } else { 28 };
    let mut m = PropModel::new([32, h as u32, 32]);
    let post = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128))
        .unwrap_or(rgb(120, 120, 120));
    for x in 0..17i32 {
        for y in 0..h {
            let sx = (x * sw / 32).clamp(0, sw - 1);
            // Block-max over the sprite rows this model
            // row covers: thin rails (1px diagonals)
            // vanish under point sampling at 32 -> 28.
            let r0 = sh - 1 - ((y + 1) * sh / h);
            let r1 = sh - 1 - (y * sh / h);
            let mut found: Option<Rgba<u8>> = None;
            for sy in r0..=r1 {
                let p = *sprite.get_pixel(sx as u32, sy.clamp(0, sh - 1) as u32);
                if p[3] >= 128 {
                    found = Some(p);
                    break;
                }
            }
            if let Some(p) = found {
                for z in [15, 16] {
                    m.set(x as u32, y as u32, z as u32, Some(p));
                }
            }
        }
    }
    for x in [0, 1, 15, 16] {
        for z in [15, 16] {
            for y in 0..h {
                m.set(x as u32, y as u32, z as u32, Some(post));
            }
        }
    }
    m
}

/// A sprite-faced shell box (walls; boxy fallback):
/// front and back carry the sprite, sides carry its
/// edge columns, the top is capped with the face's
/// top row. Shell thickness 2 -- the fine mesher only
/// ever draws the surface.
fn terrain_box(sprite: &RgbaImage, h: i32) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut m = PropModel::new([32, h as u32, 32]);
    let modal = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128))
        .unwrap_or(rgb(120, 120, 120));
    // Side/top fallback colour: the modal over NON-
    // near-black opaque pixels. A sprite drawn on a
    // dark backdrop (the ATM) has a near-black modal;
    // its sides must not inherit the backdrop.
    let side_modal = modal_color(
        sprite
            .pixels()
            .copied()
            .filter(|p| p[3] >= 128 && (p[0] >= 28 || p[1] >= 28 || p[2] >= 28)),
    )
    .unwrap_or(modal);
    // The face samples the sprite's OPAQUE BBOX, not
    // the whole sprite: sprites whose subject sits in
    // part of the frame (the ATM occupies the lower
    // 56 of its 64 rows) otherwise stretch a band of
    // transparent-row fallback colour across the box's
    // upper face -- the "large black box" over the ATM.
    let (mut bx0, mut by0, mut bx1, mut by1) = (sw, sh, -1, -1);
    for yy in 0..sh {
        for xx in 0..sw {
            if sprite.get_pixel(xx as u32, yy as u32)[3] >= 128 {
                bx0 = bx0.min(xx);
                bx1 = bx1.max(xx);
                by0 = by0.min(yy);
                by1 = by1.max(yy);
            }
        }
    }
    if bx1 < 0 {
        (bx0, by0, bx1, by1) = (0, 0, sw - 1, sh - 1);
    }
    let (bw, bh) = (bx1 - bx0 + 1, by1 - by0 + 1);
    let face = |mx: i32, y: i32| -> Rgba<u8> {
        let sx = bx0 + (mx * bw / 32).min(bw - 1);
        let sy = by1 - (y * bh / h).min(bh - 1);
        let p = *sprite.get_pixel(sx as u32, sy as u32);
        if p[3] >= 128 {
            p
        } else {
            modal
        }
    };
    for y in 0..h {
        for x in 0..32i32 {
            let c = face(x, y);
            for z in [30, 31] {
                m.set(x as u32, y as u32, z as u32, Some(c));
            }
            for z in [0, 1] {
                m.set(x as u32, y as u32, z as u32, Some(c));
            }
            // Side faces sample the sprite's edge
            // columns; a near-black sample there is the
            // sprite's backdrop margin (e.g. the ATM's
            // dark surround), not a surface colour --
            // fall back to the modal colour.
            let side = |sx: i32, y: i32| -> Rgba<u8> {
                let c = face(sx, y);
                if c[0] < 28 && c[1] < 28 && c[2] < 28 {
                    return side_modal;
                }
                // A sample far brighter or darker than
                // the body's modal colour is a trim /
                // highlight segment of the edge column
                // (the ATM's chrome strip): extruded
                // across the whole side it becomes a
                // glaring band, so the side takes the
                // modal colour for that row instead.
                let lum = |p: Rgba<u8>| p[0] as i32 + p[1] as i32 + p[2] as i32;
                let (lc, lm) = (lum(c), lum(side_modal).max(1));
                if lc > lm * 2 || lc * 2 < lm {
                    side_modal
                } else {
                    c
                }
            };
            let cl = side(0, y);
            let cr = side(31, y);
            for xx in [0, 1] {
                m.set(xx as u32, y as u32, x as u32, Some(cl));
            }
            for xx in [30, 31] {
                m.set(xx as u32, y as u32, x as u32, Some(cr));
            }
        }
    }
    for x in 0..32i32 {
        for z in 0..32i32 {
            // Cap shaded slightly: a flat smear of the
            // face's top row otherwise reads as a solid
            // slab floating over the body (the apiary's
            // "large orange square").
            let c = shade(face(x, h - 1), 0.88);
            m.set(x as u32, (h - 1) as u32, z as u32, Some(c));
        }
    }
    m
}

/// A door: frame posts + header, and a panel textured
/// from the sprite. Open doors swing the panel 55
/// degrees toward the front on its hinge at x=2.
#[derive(Clone, Copy, PartialEq)]
enum DoorStyle {
    Closed,
    Open,
    Frame,
    Silhouette,
}

fn terrain_door(sprite: &RgbaImage, style: DoorStyle) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let h = 84i32;
    let mut m = PropModel::new([32, h as u32, 32]);
    let modal = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128))
        .unwrap_or(rgb(120, 100, 80));
    let face = |sx: i32, y: i32| -> Rgba<u8> {
        let sy = (sh - 1 - (y * sh / h)).clamp(0, sh - 1);
        let p = *sprite.get_pixel(sx.clamp(0, sw - 1) as u32, sy as u32);
        if p[3] >= 128 {
            p
        } else {
            modal
        }
    };
    // Frame: posts + header in the modal colour.
    for y in 0..h {
        for x in [0, 1, 30, 31] {
            for z in 13..=18 {
                m.set(x as u32, y as u32, z as u32, Some(modal));
            }
        }
    }
    for x in 0..32i32 {
        for y in (h - 3)..h {
            for z in 13..=18 {
                m.set(x as u32, y as u32, z as u32, Some(modal));
            }
        }
    }
    if style == DoorStyle::Frame {
        // Bare frame: no panel at all.
    } else if style == DoorStyle::Silhouette {
        for x in 2..30i32 {
            for y in 0..(h - 3) {
                let sy = (sh - 1 - (y * sh / h)).clamp(0, sh - 1);
                let p = *sprite.get_pixel((x * sw / 32).clamp(0, sw - 1) as u32, sy as u32);
                if p[3] < 128 {
                    continue;
                }
                m.set(x as u32, y as u32, 14, Some(p));
                m.set(x as u32, y as u32, 17, Some(p));
                let inner = shade(p, 0.8);
                m.set(x as u32, y as u32, 15, Some(inner));
                m.set(x as u32, y as u32, 16, Some(inner));
            }
        }
    } else if style == DoorStyle::Closed {
        for x in 2..30i32 {
            for y in 0..(h - 3) {
                let c = face(x * sw / 32, y);
                m.set(x as u32, y as u32, 14, Some(c));
                m.set(x as u32, y as u32, 17, Some(c));
                let inner = shade(c, 0.8);
                m.set(x as u32, y as u32, 15, Some(inner));
                m.set(x as u32, y as u32, 16, Some(inner));
            }
        }
    } else {
        // Panel swung toward +z on the hinge at (2, 15).
        let (cos_t, sin_t) = (0.574f32, 0.819f32); // 55 deg
        for t in 0..27i32 {
            for n in -2..2i32 {
                let wx = 2 + (t as f32 * cos_t + n as f32 * sin_t).round() as i32;
                let wz = 15 + (t as f32 * sin_t - n as f32 * cos_t).round() as i32;
                if !(0..32).contains(&wx) || !(0..32).contains(&wz) {
                    continue;
                }
                for y in 0..(h - 3) {
                    let c = face((t + 2) * sw / 32, y);
                    m.set(wx as u32, y as u32, wz as u32, Some(c));
                }
            }
        }
    }
    m
}

/// A window: frame border in the sprite's modal edge
/// colour, glass plane carrying the sprite itself
/// (drawn bars and glare come with it).
fn terrain_window(sprite: &RgbaImage, pane: bool) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let h = 64i32;
    let mut m = PropModel::new([32, h as u32, 32]);
    let modal = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128))
        .unwrap_or(rgb(140, 140, 140));
    let face = |sx: i32, y: i32| -> Rgba<u8> {
        let sy = (sh - 1 - (y * sh / h)).clamp(0, sh - 1);
        let p = *sprite.get_pixel(sx.clamp(0, sw - 1) as u32, sy as u32);
        if p[3] >= 128 {
            p
        } else {
            modal
        }
    };
    for y in 0..h {
        for x in 0..32i32 {
            let border = x < 3 || x > 28 || y < 3 || y > h - 4;
            if border {
                for z in 14..=17 {
                    m.set(x as u32, y as u32, z as u32, Some(modal));
                }
            } else if pane {
                let c = face(x * sw / 32, y);
                for z in [15, 16] {
                    m.set(x as u32, y as u32, z as u32, Some(c));
                }
            }
        }
    }
    m
}

/// A ground / liquid tile: a low relief block whose
/// top carries the sprite as its texture (the sprite
/// IS the top-down drawing of the ground). Liquids
/// are flat; solids get +-2 of hash relief.
fn terrain_ground(sprite: &RgbaImage, liquid: bool, mound: bool) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut m = PropModel::new([32, if mound { 16 } else { 10 }, 32]);
    let modal = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128))
        .unwrap_or(rgb(100, 100, 100));
    for x in 0..32i32 {
        for z in 0..32i32 {
            let p = *sprite.get_pixel(
                (x * sw / 32).clamp(0, sw - 1) as u32,
                (z * sh / 32).clamp(0, sh - 1) as u32,
            );
            let c = if p[3] >= 128 { p } else { modal };
            let relief = if liquid {
                0
            } else {
                (hash01(x as u32, z as u32) * 2.4) as i32
            };
            // A mound heaps toward the tile centre.
            let bump = if mound {
                let dx = x as f32 - 15.5;
                let dz = z as f32 - 15.5;
                let d = (dx * dx + dz * dz).sqrt();
                ((1.0 - d / 22.0).max(0.0) * 5.0) as i32
            } else {
                0
            };
            let top = 6 + relief.min(2) + bump;
            for y in 0..=top {
                let cc = if y == top { c } else { shade(c, 0.72) };
                m.set(x as u32, y as u32, z as u32, Some(cc));
            }
        }
    }
    m
}

/// Stairs: six solid steps ascending toward the far
/// side (z=0), tops textured from the sprite. Down
/// stairs use the same structure darkened -- the
/// stairwell reads as descending into shadow from
/// most angles; orientation pins come with review.
fn terrain_stairs(sprite: &RgbaImage, down: bool) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut m = PropModel::new([32, 96, 32]);
    let modal = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128))
        .unwrap_or(rgb(110, 110, 110));
    for k in 0..6i32 {
        let top = (6 - k) * 16;
        for z in (k * 5)..(k * 5 + 5) {
            for x in 0..32i32 {
                let p = *sprite.get_pixel(
                    (x * sw / 32).clamp(0, sw - 1) as u32,
                    (z * sh / 32).clamp(0, sh - 1) as u32,
                );
                let mut c = if p[3] >= 128 { p } else { modal };
                if down {
                    c = shade(c, 0.55);
                }
                for y in 0..top {
                    let cc = if y >= top - 2 { c } else { shade(c, 0.6) };
                    m.set(x as u32, y as u32, z as u32, Some(cc));
                }
            }
        }
    }
    m
}

/// The generic fallback: a fully-opaque, frame-filling
/// sprite becomes a sprite-faced box; anything else
/// becomes the disc-solid of its silhouette.
fn terrain_silhouette(sprite: &RgbaImage) -> Option<PropModel> {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let mut opaque = 0usize;
    let (mut x0, mut x1, mut y0, mut y1) = (sw, -1, sh, -1);
    for yy in 0..sh {
        for xx in 0..sw {
            if sprite.get_pixel(xx as u32, yy as u32)[3] >= 128 {
                opaque += 1;
                x0 = x0.min(xx);
                x1 = x1.max(xx);
                y0 = y0.min(yy);
                y1 = y1.max(yy);
            }
        }
    }
    if opaque == 0 {
        return None;
    }
    let coverage = opaque as f32 / (sw * sh) as f32;
    let fills = (x1 - x0 + 1) as f32 >= 0.9 * sw as f32 && (y1 - y0 + 1) as f32 >= 0.9 * sh as f32;
    if coverage >= 0.78 && fills {
        let h = ((y1 - y0 + 1) * 48 / sh).clamp(16, 96);
        return Some(terrain_box(sprite, h));
    }
    // Boxy silhouettes (median row mostly filled across
    // its span) get the box form too, at their drawn
    // height -- a hive or an ATM is not a cylinder.
    let mut full_rows = 0usize;
    let mut rows = 0usize;
    for yy in y0..=y1 {
        let mut lo = sw;
        let mut hi = -1;
        let mut n = 0usize;
        for xx in 0..sw {
            if sprite.get_pixel(xx as u32, yy as u32)[3] >= 128 {
                lo = lo.min(xx);
                hi = hi.max(xx);
                n += 1;
            }
        }
        if hi >= 0 {
            rows += 1;
            if n as f32 >= 0.8 * (hi - lo + 1) as f32 {
                full_rows += 1;
            }
        }
    }
    if rows > 0 && full_rows as f32 >= 0.7 * rows as f32 {
        let h = ((y1 - y0 + 1) * 48 / sh).clamp(16, 96);
        return Some(terrain_box(sprite, h));
    }
    let base = if sh >= 96 {
        72
    } else if sh >= 64 {
        56
    } else {
        56
    };
    Some(disc_model(sprite, base, false))
}

/// Top-`n` colors of an iterator, most frequent first.
fn palette_of<I: Iterator<Item = Rgba<u8>>>(it: I, n: usize) -> Vec<Rgba<u8>> {
    let mut counts: std::collections::BTreeMap<[u8; 4], u32> =
        std::collections::BTreeMap::new();
    for px in it {
        if px[3] >= 128 {
            *counts.entry(px.0).or_insert(0) += 1;
        }
    }
    let mut v: Vec<([u8; 4], u32)> = counts.into_iter().collect();
    v.sort_by_key(|&(_, c)| std::cmp::Reverse(c));
    v.truncate(n);
    v.into_iter().map(|(c, _)| Rgba(c)).collect()
}

fn pick(palette: &[Rgba<u8>], h: f32) -> Rgba<u8> {
    if palette.is_empty() {
        return rgb(128, 128, 128);
    }
    palette[((h * palette.len() as f32) as usize).min(palette.len() - 1)]
}

fn is_foliage(px: Rgba<u8>) -> bool {
    px[3] >= 128 && px[1] > px[0] && px[1] >= px[2]
}

/// Foliage-coloured: green-dominant, or saturated
/// warm (autumn reds/oranges, r far above b). Snow,
/// bark browns and dormant tan are NOT foliage --
/// this is the leafiness measure for the bare-tree
/// decision and the crown depth factor, where a
/// "not bark" test fails on snow-covered winter
/// sprites (snow counts as leafy there).
fn looks_like_foliage(px: Rgba<u8>) -> bool {
    if px[3] < 128 {
        return false;
    }
    let (r, g, b) = (px[0] as i32, px[1] as i32, px[2] as i32);
    (g > r && g >= b)
        || (r - b > 40 && r >= 2 * g)
        || (g - b > 55 && r - b > 60)
        || (r > 150 && b > g && r - b > 30)
}

fn is_bark(px: Rgba<u8>) -> bool {
    // Brown trunk colours: r > g >= b with CLOSE channels
    // (low saturation). Saturated reds/oranges (autumn
    // foliage) have r far above b and are NOT bark --
    // without the guard an autumn crown classifies as
    // trunk and the canopy collapses to a column.
    px[3] >= 128
        && px[0] > px[1]
        && px[1] >= px[2]
        && px[0] < 175
        && (px[0] as i32 - px[2] as i32) <= 70
}

/// A tree: bark trunk and limbs under a solid crown whose
/// per-layer radius follows the sprite's foliage silhouette.
/// The sprite is a giant (96px) drawing of one tree on one
/// tile, so the model is built at tile scale from the
/// sprite's proportions. The caller passes the FG sprite
/// alone (no bg composite): the bg's grass would otherwise
/// read as foliage and grow the crown to the ground.
fn terrain_tree(sprite: &RgbaImage) -> PropModel {
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    let (mw, mh, md) = (32i32, 80i32, 32i32);
    let mut m = PropModel::new([mw as u32, mh as u32, md as u32]);
    let greens = palette_of(sprite.pixels().copied().filter(|&p| is_foliage(p)), 24);
    let barks = palette_of(sprite.pixels().copied().filter(|&p| is_bark(p)), 10);
    let dark_green = pick(&greens, 0.85);
    // Per-sprite-row spans of foliage and bark (FG alpha).
    let row_span = |r: i32, test: fn(Rgba<u8>) -> bool| -> Option<(i32, i32)> {
        let mut span: Option<(i32, i32)> = None;
        for x in 0..sw {
            if test(*sprite.get_pixel(x as u32, r as u32)) {
                span = Some(match span {
                    None => (x, x),
                    Some((a, b)) => (a.min(x), b.max(x)),
                });
            }
        }
        span
    };
    let row_of_y = |y: i32| -> i32 { (mh - 1 - y) * sh / mh };
    // Crown first: per-layer discs from the foliage
    // silhouette, solid to the core (no hollow shell).
    for y in 0..mh {
        let r = row_of_y(y);
        let Some((x0, x1)) = row_span(r, is_foliage) else { continue };
        let cx = ((x0 + x1) / 2) * mw / sw;
        let rad = ((x1 - x0 + 1) * mw / sw) as f32 / 2.0;
        let ri = rad.ceil() as i32;
        for dz in -ri..=ri {
            for dx in -ri..=ri {
                let d = ((dx * dx + dz * dz) as f32).sqrt();
                let edge = rad + (hash01((dx + ri) as u32, (y * 31 + dz) as u32) - 0.5) * 1.6;
                if d > edge {
                    continue;
                }
                let (x, z) = (cx + dx, 16 + dz);
                if x < 0 || x >= mw || z < 0 || z >= md {
                    continue;
                }
                let c = if dz > 0 && d > rad - 1.2 {
                    // Front shell: the sprite's own leaf
                    // pixel, snapped to the crown palette
                    // (top-24 foliage colours) -- bounded
                    // colour count for the texture-array
                    // budget, no stray edge colours.
                    let sx = (x * sw / mw).clamp(0, sw - 1);
                    let p = *sprite.get_pixel(sx as u32, r as u32);
                    if is_foliage(p) {
                        *greens.iter().min_by_key(|g| {
                            (g[0] as i32 - p[0] as i32).abs()
                                + (g[1] as i32 - p[1] as i32).abs()
                                + (g[2] as i32 - p[2] as i32).abs()
                        }).unwrap_or(&dark_green)
                    } else { pick(&greens, hash01(x as u32, y as u32)) }
                } else if d > rad - 1.2 {
                    pick(&greens, hash01((x * 5 + z) as u32, (y * 11) as u32))
                } else {
                    dark_green
                };
                m.set(x as u32, y as u32, z as u32, Some(c));
            }
        }
    }
    // Bark second, so the trunk and the limbs the sprite
    // shows inside the crown draw over the foliage: a disc
    // per layer at the bark span, thick for the trunk,
    // capped thin for limbs.
    for y in 0..mh {
        let r = row_of_y(y);
        let Some((x0, x1)) = row_span(r, is_bark) else { continue };
        let cx = ((x0 + x1) / 2) * mw / sw;
        let cap = if y <= mh * 2 / 5 { 3.4 } else { 2.0 };
        let rad = (((x1 - x0 + 1) * mw / sw) as f32 / 2.0).clamp(1.2, cap);
        let ri = rad.ceil() as i32;
        for dz in -ri..=ri {
            for dx in -ri..=ri {
                let d = ((dx * dx + dz * dz) as f32).sqrt();
                if d > rad + hash01((x0 + dx) as u32, (y * 7) as u32) * 0.4 {
                    continue;
                }
                let (x, z) = (cx + dx, 16 + dz);
                if x < 0 || x >= mw || z < 0 || z >= md {
                    continue;
                }
                let c = if dz > 0 && (d - rad).abs() < 1.0 {
                    let sx = (x * sw / mw).clamp(0, sw - 1);
                    let p = *sprite.get_pixel(sx as u32, r as u32);
                    if is_bark(p) { p } else { pick(&barks, hash01(dx as u32, y as u32)) }
                } else {
                    pick(&barks, hash01((cx + dx) as u32, (y * 3 + dz) as u32))
                };
                m.set(x as u32, y as u32, z as u32, Some(c));
            }
        }
    }
    // Root flare: bark reaching out at the base in the
    // diagonals, as the sprite's roots splay.
    for (dx, dz) in [(2, 0), (-2, 0), (0, 2), (0, -2), (2, 2), (-2, -2), (2, -2), (-2, 2)] {
        for y in 0..2 {
            m.set((16 + dx) as u32, y as u32, (16 + dz) as u32, Some(pick(&barks, hash01(dx as u32, dz as u32))));
        }
    }
    m
}

/// A chain-link fence HALF-segment in the full tile
/// frame (32x32): from the tile's centre post to the
/// WEST edge post, top rail, and a see-through diamond
/// mesh, the panel centred at z 15..=16 (symmetric
/// about the tile centre, so the engine's quarter turns
/// map it onto itself). The engine stamps one rotated
/// copy per connected direction and unions them, so
/// straights, corners, tees, crosses and ends all
/// compose from this one model. An isolated tile (no
/// connected direction) gets no placement and keeps
/// its coarse block rendering.
fn terrain_chainfence(sprite: &RgbaImage) -> PropModel {
    let mut m = PropModel::new([32, 28, 32]);
    let steel = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128 && p[0] > 120 && (p[0] as i32 - p[2] as i32).abs() < 40))
        .unwrap_or(rgb(168, 175, 182));
    let wire = modal_color(sprite.pixels().copied().filter(|p| p[3] >= 128)).unwrap_or(steel);
    // Posts: west edge (x 0..=1) and centre (x 15..=16),
    // z 15..=16, full height.
    for x in [0, 1, 15, 16] {
        for z in [15, 16] {
            for y in 0..28 {
                m.set(x, y, z, Some(steel));
            }
        }
    }
    // Top rail.
    for x in 0..=16 {
        for z in [15, 16] {
            for y in 25..=26 {
                m.set(x, y, z, Some(steel));
            }
        }
    }
    // Diamond mesh on both centre planes + bottom
    // tension wire.
    for x in 2..=14 {
        for y in 2..=24 {
            let wire_here = (x + y) % 6 == 0 || (x as i32 - y as i32).rem_euclid(6) == 0 || y == 2;
            if wire_here {
                for z in [15, 16] {
                    m.set(x, y, z, Some(wire));
                }
            }
        }
    }
    m
}
