# Fine-mesh slope normals — engine fix plan

Status: planned 2026-11-24 (Joey's instruction). Motivating case: the
vehicle Stage-G profile captures (`captures/vehicle_car_profile_*.png`,
commit 98e2a0b) — shape and colour approved, but every sloped surface
shows hard horizontal banding.

## 1. Problem

The fine mesher (`crates/mesher/src/fine.rs`, `mesh_chunk_fine`) emits
each face with a single axis normal (`normal_to_world(dir)` via
`push_quad` / `push_quad_textured`): all four corners of a quad share
the face's axis normal. The raster shader shades by Lambert against a
fixed light direction (0.3, 0.8, 0.4) (`crates/engine/src/render.rs`),
so a top face receives ~0.8 of the directional term while an x-facing
riser receives ~0.3 and a z-facing riser ~0.4.

A sloped fine surface (vehicle hood slope, windshield / rear-glass
ramps, roof crown; terrain shore and blend slopes) is a staircase of
1-unit steps. Each step's tread shades at the top-face level and its
riser at the side-face level, so the slope renders as alternating
light/dark bands at step frequency instead of one inclined plane. The
geometry is correct; the shading is a staircase because the normals
are.

Colour cannot compensate: a voxel carries one colour shared by its
tread and riser faces, while the banding is a normal difference, not
an albedo difference.

RT has the same defect by a different route: fine geometry is
intersected as a cell tree (`fine_leaves` / `fine_nodes` in
`crates/engine/src/rt.rs`) and the hit normal is the axis normal of
the face that was hit.

## 2. Goal and non-goals

Goal: fine heightfield slopes shade as continuous inclined planes in
the raster renderer (phase 1) and in RT (phase 2, §5). Banding and
the roof-crown grooves disappear; the crown and ramps read as smooth
gradients.

Non-goals: no geometry change (same voxels, same silhouette, same
meshing topology); the coarse block path is untouched; no change to
the vertex light bake (the bake is per-vertex light, independent of
normals — only the shader's diffuse term responds to the new
normals); side-face smoothing (e.g. wheel-disc silhouettes) is out of
scope — phase 1 tilts top faces only.

## 3. Design (raster)

`Mesh.normals` is already per-vertex (`crates/mesher/src/mesh.rs`);
only the emission helpers force one normal per quad. The fix is
local to `mesh_chunk_fine`:

1. When emitting a +Y face, compute a per-corner normal from the
   local heightfield gradient: for each corner, sample the column
   top heights of the fine grid at the four horizontally adjacent
   cells (central differences, in fine units) and set
   n = normalize(-dhdx, 1, -dhdz), converted to world space with the
   same convention as `normal_to_world`. Adjacent treads share
   corners, and a shared corner gets the same gradient normal from
   both sides, so the rasteriser's interpolation makes the staircase
   shade as one plane.
2. Hard-edge preservation: if any sampled neighbour height differs
   by more than a threshold T (proposed default 8 fine units — a
   quarter tile), that corner keeps the axis normal. Table tops,
   bed frames, panel edges and other deliberate cliffs stay crisp;
   only true slopes (per-step differences of 1–2 units) smooth.
   T is a named tuning constant, not buried.
3. Gradient sampling must read through the same neighbour sampler
   the mesher already uses for boundary culling, so slopes crossing
   a chunk border do not seam.
4. Both emission paths get the treatment: `push_quad` and
   `push_quad_textured` — the fine pass's flat and textured
   variants for stamped cells. (Correction, S1: shore/blend
   terrain slopes are a coarse-path texture treatment and do
   NOT pass through `mesh_chunk_fine`; the textured variant
   here covers textured stamped cells — props carrying
   `tex_layers`.)
5. A per-mesh opt-in flag, default off, so existing callers change
   only where §4 has verified the result. The fine-prop and
   fine-terrain paths opt in explicitly.

## 4. Gates

- G1 (mesher unit gate, written failing-first): a synthetic fine
  grid carrying a 1-unit-step ramp. Assert every top-face vertex
  normal tilts by atan(gradient) within tolerance; a flat plateau
  region keeps exact axis normals; a 16-unit cliff keeps axis
  normals at its edge corners. Fails on current code (all normals
  axis-aligned).
- G2 (capture diff): re-render the vehicle profile capture set.
  Assert the covered-pixel silhouette is identical to the pre-fix
  captures (geometry unchanged) while the band count across the
  windshield ramp and crown collapses. Until the vehicle engine
  build lands, the capture runs through the out-of-repo harness
  used for 98e2a0b's captures; the in-repo equivalent is a fixture
  fine ramp scene through `CaptureArgs`.
- G3 (golden review): furniture and terrain-fine goldens re-run.
  Every golden whose pixels change is listed and eyeballed before
  regen — no silent regen. Expected changes: sloped prop parts and
  shore/blend slopes only; boxy props must be pixel-identical.

## 5. RT (phase 2)

Spike: at a +Y hit, estimate the surface gradient from the fine
tree by probing occupancy one cell out in ±x/±z at the hit level
and tilt the hit normal the same way, with the same threshold rule.
If the probe cost or the tree layout makes this unreasonable, the
fallback is to document RT banding as a known RT-only artifact and
revisit when RT's frame time on target hardware is measured (the
standing open measurement that decides whether RT becomes the
default renderer at all). Decision recorded here either way.

## 6. Slices

- S1: gradient normals in `mesh_chunk_fine` behind the opt-in flag
  + G1.
- S2: opt in the fine-prop and fine-terrain paths; G2 + G3; vehicle
  profile captures re-judged.
- S3: RT spike and decision (§5).

## 7. Risks and open questions

- Threshold T interacts with chunky deliberate steps (stairs are
  1-unit steps too — smoothing their normals makes a staircase
  shade like a ramp while still stepping geometrically; G3 must
  include a stair capture for judgement).
- `mesh_bench` before/after: the gradient pass is one extra sweep
  over top faces; confirm no material regression.
- Diagonal vehicle facings stay tile-stepped (unchanged by this
  plan; a wire/render question, not a shading one).

## 8. Relation to other work

The vehicle Stage-G engine build (voxelizer per-role constructors +
stamper profile/colouring pass; design settled in the 2026-11-24
prototype thread, captures in 98e2a0b) should land after S2, so its
two-pass review judges final shading rather than the banded
interim. This plan is independent of Phase 11 (sim) work.

## 9. S1 outcome (2026-11-24)

Landed: `mesh_chunk_fine` takes `slope_normals: bool` (all
existing callers pass `false`; S2 opts the fine paths in).
With the flag on, a stamped cell's +Z face is emitted through
the new `Mesh::push_quad_per_corner` /
`push_quad_textured_per_corner` with per-corner normals from
`top_corner_normal`: the central difference of the four
neighbouring columns' surface tops (`surface_top`, walked
through the `FineSampler`, so coarse-expanded columns resolve
identically and chunk borders cannot seam — the sampler IS
the cross-chunk read path). Threshold
`SLOPE_EDGE_THRESHOLD = 8` fine cells, as proposed.
G1 (`crates/mesher/tests/fine_slope_normals_gate.rs`, written
failing-first — the ramp assertion failed on the pre-fix
mesher while the three invariant tests passed): ramp corners
tilt to normalize(-1, 1, 0) within 0.02; plateau and 16-unit
cliff keep exact axis normals; flag off is all-axis. 4/4.
Regression: full mesher suite green; engine golden_gate 3/3
with the flag off everywhere (meshes unchanged), phase9 /
puddle / raster_tex / frame / phase8 / vehicle_texture gates
all green.

## 10. S2 outcome (2026-11-24)

Opted in: `scene_prep::mesh_filtered_fine` passes `true` — the
one dispatch point for fine chunks, so the live path, the
snapshot path and captures all shade slopes.

Scope completion against §2/§3, driven by G2 evidence: with
top faces only, the vehicle profile capture still showed the
step lines — each tread shaded as a plane but every riser
(the 1-cell vertical face between treads) still shaded at
side-face level. §2's goal (banding gone) was not met by §3's
phase-1 letter, so side faces that are slope risers joined
the treatment: `riser_corner_normals` fires when the
neighbouring column's surface is 1..=SLOPE_EDGE_THRESHOLD
below this cell's top; top-edge corners take this cell's
gradient, bottom-edge corners the lower neighbour's gradient
at the same vertex. Walls, cliffs and same-height neighbours
keep axis normals. G1 gained a fifth test
(`ramp_riser_faces_carry_the_plane_normal`, verified failing
on the top-only mesher first): 5/5.

G2 (vehicle profile capture, re-rendered through the fine
prop path): quad count identical to the S1-era capture
(221,840 — geometry untouched); cover classification changed
on zero pixels against the tex6 capture (silhouette
identical); raster distinct colours 9,077 → 11,174. The
windshield and hood now shade as continuous planes; the
dense per-step banding is gone.

G3: golden_gate / phase9_render_gate / puddle_water_gate all
pass UNCHANGED — no golden in the set contains a shallow
stamped slope (fixture props are flat-topped or cliff-edged),
so there was nothing to regen or eyeball. The stair case the
plan asked for is the G1 ramp itself (1-cell steps).

Residual, recorded honestly: the roof crown's two step
grooves remain faintly visible. Their remaining cause is not
normals — it is the Stage H geometric occlusion term
(`face_occlusion`, ×0.89 per occluding neighbour, flat per
face) stepping at each 1-cell crown step, plus the steps'
geometry itself. Removing the last trace means per-corner
occlusion (a separate fine-mesher change, all props
affected) or a shallower crown (geometry; already a two-pass
tuning item for the Stage-G vehicle build). Neither is in
this plan's scope; Joey's call if either is wanted.

Regression: full mesher suite green; full engine suite green
(lib 33 + every gate target, incl. golden 3/3, phase9 set,
sky set, stage set, sim set, vehicle_texture 4/4).
Not pushed (bundle route via Joey's PC, as ever).

## 11. Wider-stencil gradient (2026-11-24, Joey's call)

S2 review found residual banding on the vehicle profile's
windshield and hood front, with squiggly band edges. Root
cause (verified against the exported car model): the S1
gradient was the 1-cell height difference across each
vertex, which oscillates between 0 and the full step on any
slope shallower than 1:1 — the integer heightfield steps in
whole cells with flat rests between (the windshield sheet's
true gradient 0.84 quantises to runs of 1-steps and 0-rests;
the hood front's 0.28 to a step every ~3.5 columns). Band
edges are iso-height contours, so cross-slope components
(crown, tumblehome, fender drop) make them wobble. The G1
ramp never showed it: at exactly 1:1 the 1-cell difference
is exact.

Fix: `top_corner_normal` now fits a least-squares plane to
the surface tops of the 6x6 columns around the vertex
(`SLOPE_FIT_COLS = 3`, centres +-0.5..+-2.5), averaging
rests and steps back out to the true gradient. Samples past
`SLOPE_EDGE_THRESHOLD` are excluded from the fit, so a
cliff still contributes only its own side and stays flat;
fewer than 12 samples (or a degenerate window) falls back
to the axis normal. Risers inherit the fit through the same
function. G1 gains a sixth test — a quantised 0.75-gradient
ramp whose corners must land on normalize(-0.75, 1, 0)
within 0.05 — verified failing on the 1-cell estimator
first (it returned flat normals at rest vertices). 6/6;
full mesher suite 34/34.

Vehicle profile re-capture: 221,840 quads identical, zero
cover-classification changes vs tex6 (silhouette
identical); the windshield shades as one continuous plane
and the hood front as one panel — the ripple is gone. The
crown grooves persist unchanged in nature (step geometry +
per-face occlusion, §10) and now read as clean straight
channels. Golden / phase9 / puddle / raster_tex / frame /
vehicle_texture / phase9_rt / fine_occlusion_rt gates all
pass unchanged — still no golden fixture with a shallow
stamped slope. Not pushed.

## 12. S3 outcome — RT spike: SUCCESS, probe ships as the implementation (2026-11-24)

The §5 spike succeeded and stays as the implementation; no
fallback to "document the banding" is needed. In the RT
render shader (`crates/treebuild/src/render.rs`), `shade()`
now takes its lighting normal from `fine_light_normal(h)`:
for a fine hit on an upward surface — a +z top face, or the
side face of a column-top cell (a slope riser) — the axis
normal is tilted by the heightfield gradient probed from
the fine tree itself: neighbour column tops (±x/±y, one
cell out) are found by bounded vertical walks through the
existing `fine_descend` point query, with the mesher's
rules — walk bound 8, any neighbour top more than 8 from
the hit top (or unmeasurable) keeps the axis normal, as do
walls, bottoms and coarse hits. Texture face selection and
cone LOD keep the axis normal; only the lighting terms
(hemisphere, sun/moon, shadow offset) take the tilt.

Evidence: the vehicle RT capture's windshield and hood go
from dense per-step striping to single continuous planes;
2.11% of pixels change, all on the car's sloped surfaces.
RT distinct colours 3,742 -> 3,997. The estimator is the
1-cell difference (S1 quality), not the §11 plane fit: at
RT's per-cell normals it shows no visible oscillation on
this scene (windshield gradient 0.84 is near-exact for it;
the hood's shallow slope is masked by RT grain). If a
future scene exposes it, the upgrade path is a wider
probe stencil in the same function — no layout changes.
Cost: bounded ~4 column walks per fine surface hit; no
slowdown measurable on this VM (full three-capture run
34.7 s vs a 54.9 s compile-free baseline — run-to-run
variance here exceeds any plausible probe cost; the real
frame-time verdict still waits on target hardware, as
with RT generally). Regression: treebuild 26/26; engine
rt_gate, phase9_rt_gate, fine_occlusion_rt_gate,
sky_rt_gate, sky_live_gate all pass (18 tests — none
pixel-compares a fine slope scene, the same golden gap
as the raster side). Raster output is untouched (the
raster and top captures are byte-identical; only the RT
capture changed). One implementation note: `wgsl_sources()`
omits EMITTER_WGSL from its render string while the
pipeline includes it — a pre-existing gap in that test
accessor, unrelated to this change; the pipeline assembly
is what the GPU gates exercise. Not pushed.

## 13. Per-corner occlusion (2026-11-24)

The §10 residual's occlusion half, fixed in the mesher.
`face_occlusion`'s counting core now takes the air position
as a parameter; the new `corner_occlusion` runs the same
rule (0.11 per opaque neighbour beside/ahead, floor 0.56)
from the air cell diagonally in front of each face corner —
the front cell that has the corner vertex on its back face.
Under the existing fine-path opt-in, `mesh_chunk_fine`
emits per-corner colours (new `push_quad_*_per_corner_colors`
in mesh.rs) so the factor interpolates across the face: at
a 1-cell step the tread's step-side corners darken and its
far corners keep 1.0, instead of one flat 0.89 notch across
the whole tread. Flag off keeps the legacy flat per-face
factor byte-identically. `face_occlusion` and
`cell_occlusion` are numerically untouched, so the RT
path's per-cell form is unchanged by construction.

Gate: `crates/mesher/tests/fine_occlusion_corner_gate.rs`,
4/4, written failing-first (pre-fix, the tread's far corners
read the flat 0.89; the invariant pins — plateau 1.0,
flag-off flat 0.89, cell_occlusion 0.89 — passed
throughout). Post-fix the step tread reads near 0.67 /
far 1.0, strictly ordered. Full mesher suite 38/38.

Engine regression: phase9_render 3/3, puddle 4/4, frame
5/5, raster_tex 3/3, vehicle_texture 4/4. golden_gate: the
raster captures that executed (house_raster_flat,
house_raster_cutaway0) show 0 mismatched pixels; the gate
then stops at its first RT capture on a shader parse error
in the RT spike's WIP crate (recursive `fine_light_normal`,
crates/treebuild/src/render.rs) — outside this slice's
scope and unreachable from mesher output. No golden
changed; none regenerated.

Vehicle check: profile capture re-rendered at 221,840
quads, zero cover-classification changes, and the files
are byte-identical to the committed captures — the crown
grooves did NOT soften in this capture, and the reason is
structural: the car renders through the TEXTURED prop
path, whose bake (`lighting::bake_pass_meshes`) replaces
every textured vertex's RGB with the pure light
multiplier, discarding mesher-side colour modulation
entirely (flat or per-corner alike — which is also why
the S1/S2 normal work showed here while colour work
cannot). §10's occlusion attribution holds for the FLAT
fine path, where the per-corner gradient now applies;
in the textured path the grooves are step geometry plus
the per-vertex bake, and softening them there is a
bake-side question, not a mesher one. Not pushed.

## 14. Outcome — bake-side fix for the textured path (2026-11-24, Joey's order)

§13's attribution was INCOMPLETE, found by doing the work:
the first cut folded the factor into the bake (mesher
records per-corner factors, `bake_pass_meshes` multiplies
its textured multiplier by them) and the vehicle capture
changed on ZERO pixels. The deeper root cause, read out of
`render.rs`: a textured fragment's diffuse term is
`texel * (sky_gi_factor + emit * bake_ceil)` — the baked
vertex RGB is read ONLY as the emitter ceiling
(`bake_ceil`), never as diffuse light. In an emitter-less
scene the bake multiplier is dead data for textured
surfaces, so no colour-channel route — mesher colour or
bake multiplier — can ever carry occlusion to a textured
diffuse term. The factor needs its own channel.

Final design, as built:

- `mesher::Mesh` gains `tex_occlusion: Vec<f32>`, parallel
  to `positions` when non-empty (empty = no factors, all
  1.0). Every `push_quad*` maintains it (1.0 appends while
  active); `Mesh::append` merges it with the same
  empty-means-none semantics; the new
  `push_quad_textured_per_corner_colors_occ` records real
  factors (backfilling 1.0 over earlier vertices on first
  use).
- `mesh_chunk_fine`, under the existing slope opt-in only,
  records the corner factors at the textured emission site
  (the same `corner_occlusion` values that shade the
  per-corner colours). Opt-in off records nothing — the
  legacy textured path is byte-identical.
- The bake deliberately does NOT fold the factor in (it
  would double-apply and pollute the emitter ceiling);
  `tex_occlusion` passes through `bake_pass_meshes`
  untouched, pinned by
  `lighting/tests/tex_occlusion_bake_gate.rs`.
- The engine uploads the array as vertex attribute 5
  (`Vertex.occ`, defensive per-vertex fallback 1.0) and
  the textured fragment branches (opaque, transparent,
  and textured water) multiply their result by the
  interpolated factor. Flat fragments always carry 1.0
  by construction (their occlusion rides in their
  colours, as before) and their branch is untouched.

Gates: `mesher/tests/fine_occlusion_textured_gate.rs`
(recording: tread factors 0.67 near / 1.0 far on a
TEXTURED step, array exactly parallel, opt-in off empty)
and the lighting gate above — both verified failing-first
(the bake-multiply cut satisfied neither the capture nor,
on reflection, the contract). Regression: mesher 40/40,
lighting 45/45, engine lib 33/33; golden 3/3 (within
tolerance, no regen), phase9_render 3/3, puddle 4/4,
raster_tex 3/3, sky_raster 7/7, vehicle_texture 4/4,
emitter 5/5, frame 4/5 — frame's one failure is the
gate_2h4 UI-latency BUDGET (median ~180 ms vs 50 ms on
llvmpipe), proven pre-existing by a stash A/B: the clean
tree fails with the same numbers (178.85/412.51 ms vs
188.09/419.27 ms) under the day's VM load; it passed
earlier the same day. Not this change; not touched.

Vehicle evidence: quads unchanged (221,840); raster
capture 30,494 pixels differ (3.31%), 30,493 of them
DARKER (mean −22.9 luma) — the occlusion signature, on
the car and on the scene's other textured props (contact
darkening now reaches every textured fine surface, not
just this car). Distinct colours 10,994 → 12,427 raster,
6,762 → 8,164 top. The crown grooves soften from flat
notches into gradients that darken toward each step; the
steps themselves are geometry and remain — the shallower
crown stays a Stage-G tuning item. RT untouched (it
never reads raster meshes); its committed capture is
unchanged. Also fixed in passing: fine.rs and the corner
gate still cited this plan's §12 for per-corner
occlusion after the renumber — now §13.
