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
   `push_quad_textured` (textured fine terrain — shore/blend slopes
   — shares `mesh_chunk_fine` and will smooth too; §4 G3 covers
   its goldens).
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
