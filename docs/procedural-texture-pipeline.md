# Procedural Texture Pipeline — finished plan (Phase 5 rework)

Status: **plan finished, verified against code at HEAD `393df0a`** (2026-11-16).
Source plan: Joey's draft, sent 2026-11-16T18:57:35Z in the "Revise CDDA voxel
texture pipeline" chat (sections 1–12 below refer to that draft). This file does
not re-paste the draft; it records what finishing it meant: the `(verify)` items
checked against the real code, the corrections the code forced, the open
questions resolved or still Joey's call, and the concrete Stage A work order.
Nothing here is committed — file is untracked until Joey says so.

## 0. What this is

A rework of the `voxelizer` (Phase 5 of `plan.md`) after the first
sprite-to-voxel output looked bad. Phase 5 passed conditionally, with Joey's
provisional sign-off on the contact sheet ("looks fine for now"), and the
coverage audit later showed its "0 missing" was circular — so this is filed as
a **Phase 5 rework, not a new phase**. It replaces "use the sprite as the
texture" with "use the sprite as the source of palette and exemplars, and
generate clean tileable textures", for **both** renderers (RT and raster).

Non-goal reminder: items stay as original sprites (billboards). The item drive
(ITEM42 done, items 3,742 → 2,084; ITEM43 seeds drawn, unverified, uncommitted)
is unaffected by this plan.

## 1. Verification record — the draft's claims checked against the code

Checked 2026-11-16 against `~/workspace/cdda-voxel` at HEAD `393df0a`.
All paths are relative to the repo root.

| Draft claim | Verdict | Evidence |
|---|---|---|
| Raster: greedy-meshed quads stretch the sprite (UVs span 0..1 over the merged quad) | **Confirmed, in code with a comment saying so** | `crates/mesher/src/mesher.rs`, `face_uv`: "Fractions are relative to the (possibly greedy-merged) quad, so a quad's UVs span [0,1]² across its whole extent." `emit_greedy_quad` passes the merged quad's min/extent. This is the §1 smearing root cause on the raster path. |
| RT: in-voxel UVs repeat once per tile; smearing there is sampling, not UV stretch | **Confirmed** | `crates/treebuild/src/render.rs`, `tex_albedo`: `local = hitp - voxel_min`, top `(local.x, local.y)`, sides `(local.y / local.x, 1.0 - local.z)`. One full texture per 1×1 face. |
| RT sampler is Linear mag/min/mip today | **Confirmed — for the texture sampler** | `crates/treebuild/src/render.rs:724-731` (`rt-texture-sampler`): Linear/Linear/Linear, ClampToEdge. **Do not confuse with** `crates/engine/src/rt.rs:589` (`rt-blit-sampler`, Nearest/Nearest) — that one upscales the already-traced image, not textures. Stage A touches the treebuild sampler only. |
| Raster sampler is Linear, no anisotropy | **Confirmed** | `crates/engine/src/render.rs:410-412` (`raster-texture-sampler`): Linear/Linear/Linear, default address mode. `grep -r anisotropy crates/` finds nothing — no anisotropy exists anywhere yet. |
| Ray-cone LOD: `lod = delta + log2(w_hit) - log2(|N·V|)`, delta = 5 for 32 px | **Confirmed, both implementations** | `crates/treebuild/src/lod.rs:43` (`ray_cone_lod`, CPU mirror) and `crates/treebuild/src/render.rs:355` (`cone_lod`, WGSL). NaN → 0, clamped to `[0, max_mip]`. A mip-colorized debug view already exists (`render_debug_lod` / `lod_color`). |
| Side faces are made by stretching an edge row | **Confirmed — it is the current shipped rule** | `crates/voxelizer/src/faces.rs` header + `plan.md` §8 face rules: ground/floor sides sample the bottom edge rows and stretch, darkened 25–40% (`make_side_face`, factor 0.65 + ±3% hash noise); walls use the wall sprite directly as the side. The §1 tree-block streaks follow directly. |
| Mip chain generated at load, level 0 only from voxelizer | **Confirmed** | `crates/treebuild/src/render.rs` `from_spec` / `lod::mip_chain` (premultiplied-alpha averaging, alpha-coverage preservation, averaged in stored/display space). |
| Cutout texel < 0.5 alpha falls back to flat color, not a hole | **Confirmed** | `tex_albedo`: `if (e.flags & 2u) != 0u && texel.a < 0.5 { return flat; }`, with the comment that traversal already decided the voxel solid. |
| `voxelizer` has no color/analysis/noise/quilting code yet | **Confirmed** | `crates/voxelizer/src/` holds only `lib.rs`, `effects.rs`, `faces.rs`, `material.rs`, `multitile.rs`, `overrides.rs` (+ `bin/`). No `color.rs`, `analyze.rs`, `noise.rs`, `quilt.rs`, `classify.rs` — Stages B–D are greenfield, as the draft assumes. |
| `docs/sprite-anatomy.md` still to write (Stage B) | **Confirmed open** | `docs/` holds `phase2b/2c/2d/2e/2g/2h`, `phase3/4/6/7/8`, `hook-points.md`, `licensing.md`, `protocol.md`, `recon.md` — no sprite-anatomy doc. |
| UltiCa id patterns (`^t_grass`, `^t_dirt`, `^t_floor`, `_wall`, …) | **Partly confirmed** | `t_grass` is a real id (appears in `coverage-audit/REPORT.md`); `t_wall` appears there too. The full prefix table (§4.2) still needs enumerating from the last real-UltiCa voxelizer run (12,206 materials) — see Stage B first step below. Real TileDefs enumeration stays blocked on 0.1/0.2, as the draft notes. |
| A/B scene for Stage A exists | **Confirmed** | `world::build_sample_scene` (`--snapshot sample-scene`): 24×24 grass yard, house with wood floor / door / three windows / roof, pond, trees, fence; plus the 2h.5 composed golden `house_composed.png`. Stage A re-captures this scene from the same camera in both renderers — no new scene needed. |

## 2. Corrections to the draft (code forced these)

1. **§5.1 output layout — blocks ship as PNG layers, not KTX2.** Phase 5
   deviation 1 (recorded in `plan.md`): no dependency-free KTX2 encoder
   exists; each PNG is one texture-array layer and the indices in
   `blocks.json` are identical either way. Read "blocks.ktx2" as
   "blocks/ PNG layers (KTX2 if an encoder lands)". Same for §6.1 "second
   texture array" / `ground.ktx2` — PNG layers until then.
2. **§5.3 `materials.json` v2 needs a loader migration note.** The current
   loader (`crates/engine/src/materials.rs`) loads v1. V2 must land with a
   version check and a v1 fallback (or a one-shot migration of the
   committed `materials.json`), otherwise Stage F breaks every existing
   golden. Add to Stage F, before the engine work.
3. **§10 risk row "(RT) Hardware ray query support in wgpu is limited" is
   stale.** The shipped RT is already a software compute tracer over the
   64-tree — there is no hardware-ray-query path to fall back from. Rewrite
   the row as resolved; the live portability risk is llvmpipe-vs-GPU float
   divergence, already covered by the golden tolerances in §9.
4. **§10 risk row "(RT) No derivatives leads to wrong or blurry LOD" is
   mitigated, not open.** Ray-cone LOD shipped in Phase 2e with a CPU
   mirror and a mip-colorized debug view. Stage A *verifies* it (render the
   debug view, check transitions land at the expected distances); it does
   not build it.
5. **§6.1 "Per-tile ground variants chosen in the shader by hashing" vs
   §4.11 world-space ground mapping.** These are two different mechanisms
   for the same problem; the draft resolves it (world-space wins for v1,
   per-tile stays as the quality-flag fallback), but the `mat_layers`
   vec4-full note means the fallback path needs its variant count packed
   somewhere new. Record the decision explicitly in Stage F: fallback
   variant count goes in a second small buffer, not by repacking `mat_layers`.
6. **Stage A, first step — determine the screenshot's renderer from the
   capture, don't guess.** The draft says "confirm which path produced the
   bad screenshot". Concretely: the sample-scene captures carry the
   renderer in their `CaptureArgs`; re-render both and compare against the
   bad screenshot before touching code. If the bad shot is raster, the
   greedy-UV fix (§6 table) is the whole of Stage A; if RT, it is the
   sampler + LOD-bias work. Do not fix both blindly.

## 3. Open questions — resolved, and still Joey's call

| # (draft §11) | Resolution |
|---|---|
| 1. UltiCa wall/terrain sprite layout | **Stage B work, not a pre-decision.** `plan.md` §8 already documents sprite geometry (sheets, offsets, sizes) and the current face rules; Stage B annotates real examples into `docs/sprite-anatomy.md`. |
| 2. 32 px vs 64 px generated textures | **Recommendation: stay 32 px for v1 (Joey's call).** The LOD constant (delta = 5), the 6-level mip chain, and texel density all assume 32; 64 px doubles layer memory and changes delta everywhere. Revisit only if Stage C contact sheets show 32 px is the binding constraint. |
| 3. Macro texture size 128 vs 256; height layer vs luminance | **128 px, stored height layer (draft's default).** 128 = 4×4 tiles at the same texel density, so delta is unchanged; a stored height layer is what §4.11's height-based blending needs, luminance is a lossy proxy. 256 px only if the 128 px repetition metric (§8) fails. |
| 4. Shareable materials between tilesets | **Yes, document as a feature.** The design already supports it (recipes + per-tileset palettes); Stage B writes one paragraph in the voxelizer README. No code change. |
| 5. How much original sprite look stays for furniture/decals | **Joey's call at the Stage E review.** Default: decals keep the sprite's alpha shape (draft §4.8); furniture stays extruded-sprite models (plan.md §8). Flag it in the Stage H sign-off, don't pre-decide. **Superseded in part 2026-11-16:** furniture no longer stays extruded-sprite by default — Joey directed props to become real 3D (Stage G, §12); the open call narrows to decals and how far the archetype set goes. |
| 6–8. RT is software compute; raster stays fallback; one voxel per tile for now | **Resolved — already true in code.** No action. |

## 4. Decisions needed from Joey (nothing else blocks Stage A)

1. **Go on Stage A as written (1–3 days)?** It is diagnostic by design:
   fix rendering first, re-screenshot, then decide whether Stages B+ keep
   their scope. Saying yes authorizes only Stage A.
2. **32 px for v1** (recommendation above) — or 64 px, accepting the LOD /
   memory rework it implies.
3. **File this as `docs/procedural-texture-pipeline.md`** (this file) and
   link it from `plan.md` Phase 5 as the rework plan — or keep it out of
   the repo until Stage A reports.

## 5. Stage A work order (concrete, in file order)

1. Re-render the sample scene in both renderers from the same camera;
   identify which renderer produced the bad screenshot.
2. Raster: `crates/mesher/src/mesher.rs` — `face_uv` / `emit_greedy_quad`:
   scale UVs by merged-quad size in tiles; sampler `Repeat`.
   `crates/engine/src/render.rs` — `raster-texture-sampler`: mag Nearest,
   min/mip Linear, anisotropy 8x (new — none exists today).
3. RT: `crates/treebuild/src/render.rs` — `rt-texture-sampler`: mag
   Nearest, keep min/mip Linear; add LOD bias and N·V-weight tunables
   (start bias −0.5, weight 0.5, per the draft). Verify with
   `render_debug_lod` before/after.
4. Add the UV checker texture + debug view; assert one checker cell per
   tile on every face (draft §9 UV checker test).
5. Internal-resolution target + integer upscale (both paths; RT already
   has the blit sampler — reuse it, don't build a second one).
6. Re-capture the sample scene; save the A/B to `docs/ab/stage-a.png`;
   decide the Stage B+ scope on the evidence.

## 6. What "finished" means for the rest of the draft

Sections 1–12 of the draft stand as written, with the corrections in §2
applied and the open questions in §3 resolved as tabled. Stages B–H,
the §8 metrics, and the §9 test plan need no further finishing — they
were complete in the draft; the verification above is what was missing.
Total estimate stays 8–10 weeks to Stage H (Stage G Props 3D inserted 2026-11-16, §12; tuning/sign-off moved to Stage H); Stage A is a day or two and
may shrink the rest.

## 7. Stage A outcome (2026-11-16, Joey's go; 32 px stays for v1)

Stage A is DONE in the working tree (uncommitted; `plan.md` Phase 5
record carries the addendum). Per §5's work order:

1. **Renderer identified:** the bad screenshot was the RASTER path.
   Before captures were rendered from a pristine worktree at HEAD
   `393df0a`; the grass yard, roof and walls are one stretched sprite
   each before, and tile per tile after. A/B sheet:
   `docs/ab/stage-a.png` (raster + RT, before/after, same camera,
   sample scene, UltiCa textures, 1280x720, llvmpipe).
2. **Raster:** `face_uv` emits tile-unit UVs; sampler Repeat +
   Nearest mag / Linear min+mip. **Deviation:** anisotropy 8x is
   impossible with Nearest mag in wgpu (validation requires
   all-Linear filters when anisotropy > 1 — it panicked the first
   capture run). Nearest mag kept; anisotropy stays 1.
3. **RT:** sampler mag Nearest + Repeat; LOD bias −0.5 and N·V
   weight 0.5 are uniforms (`lod_bias`, `ndotv_weight`) fed from
   `lod::LOD_BIAS` / `NDOTV_WEIGHT`; `mip_gate` cross-checks the
   GPU against the updated CPU mirror (2e.2 slant expectation is
   now the weighted term).
4. **UV checker:** `stage_a_gate` — mesher-level exact assertion
   (merged-quad UV span == tile extent, integer-anchored) plus the
   amended `uv_gate` (2f.1's old [0,1]²-span expectation encoded
   the bug and was rewritten with the reason recorded).
5. **Internal resolution:** `render_scale` on `CaptureArgs` +
   `--render-scale N` in both entry points; composition at size/N,
   integer upscale through the existing PresentBlit. Gate: at
   scale 2 every 2x2 output block is exactly uniform, both
   renderers.
6. **Verification (own runs, lavapipe):** engine 51/51 incl.
   golden gate — flat/effects goldens 0 mismatched px; the four
   textured goldens changed by the deliberate fix (2.7–10.0%
   mismatch vs old) and were regenerated via `UPDATE_GOLDEN=1`
   after visual inspection of the actuals. treebuild + mesher
   suites pass; workspace green. Sample-scene geometry unchanged
   (216 quads raster / 742 voxels RT, before and after).

**Remaining Stage A-adjacent call for Joey:** whether Stages B+
proceed at original scope. The A/B evidence says the worst
symptoms (stretch, smear, close-up blur) were rendering bugs and
are gone; what remains visibly "spritey" at close range — baked
3/4-view wall tops, edge-row side faces — is exactly what Stages
B–F address. (Joey's call, 2026-11-16: proceed — B and the noise
foundation ran in parallel; see §8.)

## 9. Stage C outcome (2026-11-16)

Stage C is DONE (plan.md Phase 5 record carries the full
addendum). `generate.rs` ships the recipe framework (family
pooling with transition subtiles excluded, `deshade_palette`,
content-hash seeding) and the first four recipes — grass, dirt,
wood floor, plaster — all passing the §8 targets in
`stage_c_gate` (seam 0.26–1.09 vs < 1.15; mean ΔE_OK
0.006–0.028 vs < 0.04; 10 palette colors vs ≤ 24; byte-identical
determinism). The sample scene renders with generated grass in
both renderers. Next: Stage D quilting + macro ground, Stage E
remaining materials (shingle included — it has its own UltiCa
entry; other roof terrains inherit floor families via the game's
`looks_like`, so no roof recipe is needed).

## 8. Stage B + noise outcome (2026-11-16)

Stage B is DONE (plan.md Phase 5 record carries the full
addendum). Classifier coverage, measured on the live UltiCa
tileset (12,206 entries): block scope 100.0% classified / 0.0%
unknown; whole universe 93.5% with 37 fully unknown (fd_* and
facial-hair overlays). The ≥90% exit is met on the scope the
pipeline actually consumes. `docs/sprite-anatomy.md` records the
measured anatomy: no top-face pixels exist in wall sprites,
shading direction is per-sprite, ground sprites bake transition
fringes. The noise foundation for Stage C is in (`noise.rs`,
bit-exact edge tileability). One open input for Stage C: UltiCa
has no `t_roof` terrain entry — roof palettes can only come from
mapgen scene illustrations (shading factor 1.00) unless Joey
picks another source.

## 10. Stage E/F additions — mockup-driven scope (2026-11-16, Joey's direction)

Source: the finished-look mockup of the sample scene
(`~/workspace/imagine_media/`, concept image — NOT an engine render).
Joey's call: the engine output should get close to it, specifically
(a) shrubs/tufts standing on the grass and (b) round-looking ponds.
Both are now plan items, not chat-only ideas.

### 10.1 Stage E addition — circular ponds via shoreline autotiling

Terrain data stays square tiles; the rounding lives in the texels.

- Water recipe (Stage E) ships with shore variants selected by the
  8-neighbor mask, reusing the Phase 2d canonical mask→subtile
  machinery (`voxelizer/src/multitile.rs`, `neighbors_connect`).
- Interior water tiles: plain water texture. Edge tiles: generated
  shore texture — waterline drawn as a curve (quarter-circle arcs on
  corners, a smooth bow on straights) over a dirt bank, composed with
  the Stage D height blend (height order water < dirt < grass forces
  the blend, no per-tile hand tuning).
- Land tiles adjacent to water get the matching dirt ring on their
  side of the border.
- Depth cue: erode the neighbor mask once; edge distance selects
  shallow vs deep water color (interior reads darker).
- No engine/shader changes: both renderers already sample one
  texture per voxel face (Stage A UV rules), so the curve appears in
  RT and raster identically. The SDF-in-shader alternative (perfectly
  smooth waterline) is deferred — at 32px texel density the
  autotiled curve is expected to carry it; revisit only on Stage H
  A/B evidence.
- Gate: sample-scene pond capture in both renderers; corner tiles
  show the arc (pixel check: corner texels on the land side of the
  arc are dirt/grass, not water), interior unchanged, determinism
  byte-identical.

### 10.2 Stage F addition — decoration scatter (shrubs/tufts on grass)

Purely visual; no simulation or protocol change.

- Voxelizer generates a small set of alpha sprites (grass tuft, low
  shrub, flowering variant) from the grass family's own pixels,
  packed in the existing billboard atlas format.
- Placement is derived client-side from world tile coordinates with
  the noise module's integer hashing: for each exposed grass/dirt
  ground tile, the hash picks 0–2 decorations, in-tile jitter, and
  variant. Identical on every client and in both renderers from the
  same snapshot; nothing new crosses IPC.
- Density is modulated by the Stage D macro height layer so
  decorations clump in patches rather than sprinkling uniformly.
  Excluded: floors, water, under-roof tiles.
- Rendering reuses the billboard pass (cylindrical Y-only instances,
  frame order world → fields → billboards, depth-tested against
  raster geometry and RT traced depth, floor-surface lift already
  applied). Decorations are NOT registered for picking — clicks pass
  through.
- Perf gate: decoration instance count for the sample scene
  reported; frame-time verdict waits for the RX 9070 XT (existing
  billboard stress numbers are llvmpipe-only and do not settle it).

## 11. Stage D outcome (2026-11-16)

Stage D is DONE (plan.md Phase 5 record carries the full addendum).
`quilt.rs` ships Efros–Freeman quilting on a torus (patch candidates
wrap the exemplar; min-error boundary cut by DP over the overlap
error surface; deterministic seeded candidate choice), wired into
`generate.rs` as `generate_quilted` / `generate_macro` for grass and
dirt alongside the untouched Stage C noise path. Macros are 128px
with a stored height layer (per-pixel OKLab L); `height_blend` is
the CPU reference composite (stored height + weight bias, higher
wins). Parent-verified in the main tree (`stage_d_gate` 2/2, targets
NOT relaxed): quilted grass seam 0.952 / ΔE 0.0034, dirt 0.782 /
0.0051; macro grass seam 0.986 (height std 12.74), dirt 0.960 (std
4.55); byte-identical determinism for tiles, macros, and height
layers; blend endpoints exact and a synthetic high-height patch
punches through at weight 0.5. Contact sheet visually checked —
quilted tiles carry real sprite grain vs Stage C's mottled noise.
One finding worth keeping: naive candidate rules collapse into
tiling a single exemplar (an exact self-continuation always exists
on a torus); the shipped rule — best candidate per exemplar, seeded
pick among those within 4x global best — keeps the whole family in
play (documented at `pick_candidate` in quilt.rs). Next: Stage E
remaining materials + §10.1 shoreline autotiling; Stage F end-caps,
side faces, world-space ground mapping, §10.2 decoration scatter,
materials.json v2 migration.

## 12. Stage G — Props 3D (2026-11-16, Joey's direction)

Source: the lab mockup review. Joey's call: props (furniture) are to
be rendered as real 3D objects based on their sprites, eventually —
not extruded-sprite models. This inserts Stage G between Stage F and
the tuning/sign-off stage, which moves to **Stage H**.

Premise, stated plainly: a sprite is one flat projection; the depth
is not in the data. Prop geometry is therefore *inference*, from
three sources the pipeline already has:

1. **Drawing convention.** UltiCa furniture sprites are 3/4-view:
   top rows = top surface, the band below = front face, sometimes a
   side sliver. `analyze.rs` (structure-tensor orientation, per-sprite
   shading direction; anatomy in `docs/sprite-anatomy.md`) segments a
   sprite into top / front / side regions.
2. **Baked shading.** The sprite's light gradient gives surface
   orientation hints — enough for bas-relief depth, not true shape.
3. **Category.** The Stage B classifier already labels the archetype
   (tank, table, console, bunk, shelf) — a strong shape prior.

### Work items, in build order

- **G.1 Box decomposition.** Segment the sprite (top/front/side),
  un-project each region onto the matching face of a cuboid whose
  proportions come from the segment sizes. Props gain real top faces
  and depth. Shares segmentation machinery with Stage F's wall
  end-caps; target: the furniture classes where a box is honest
  (counters, lockers, shelves, crates, appliances).
- **G.2 Column voxelization.** Each sprite pixel becomes a column
  footprint on a sub-tile grid (32x32, matching the sprite 1:1);
  column height derives from position within the front segment plus
  shading relief. Output is plain voxels, so both renderers consume
  it unchanged (raster mesher + RT tree) and lighting/shadows/
  occlusion come for free. Carrier: the Stage F `materials.json` v2
  migration gains an optional per-id voxel-model blob; the engine
  stamps models into the world at build time; picking still resolves
  to the owning tile. Target: irregular props (machinery, rubble,
  tanks approximately) and everything G.1 can't box honestly.
- **G.3 Parametric archetypes.** Hand-built procedural geometry
  templates for the hero archetypes (containment tank, console desk,
  dissection/autopsy table, bunk, shelf unit), dimensions fit to the
  sprite silhouette, textured with the generated textures from the
  sprite's palette, sprite front detail kept as a decal. Per-archetype
  effort, so the set is prioritized by location: **lab set first**
  (the mockup that prompted this stage), then jail, then city
  interiors.

### Rules

- Deterministic generation only — same rules-and-gates discipline as
  the texture stages (byte-identical re-runs). ML depth estimation /
  image-to-3D is explicitly out: non-deterministic, untestable
  against the gates.
- Inferred geometry must never change gameplay meaning: prop voxels
  are visual; collision, movement, and LOS (line of sight) stay with
  the game's tile data, exactly as decoration scatter (§10.2).
- Fallback is the current extruded-sprite model: any id the
  segmentation can't read confidently keeps today's rendering rather
  than getting a guessed shape.

### Gates

- Segmentation accuracy spot-checked on a cited sprite set (the
  anatomy doc's method: measured, named sprites).
- Lab room capture in both renderers with G.1/G.2 props vs the
  extruded-sprite baseline: geometry counts reported, goldens
  regenerated only after visual inspection.
- Determinism: model blobs byte-identical on re-run.
- Frame-time verdict on the RX 9070 XT (llvmpipe numbers do not
  settle sub-tile voxel counts).

## 13. Stage G.1/G.2 outcome — generation side (2026-11-16)

DONE in the voxelizer (engine stamping waits on the Stage F v2
carrier). `props.rs`: `segment()` splits a furniture sprite into
top/front regions (+ optional side sliver) — silhouette gating,
boundary candidates scored on horizontal seam edge + local L-step,
first-strong-edge choice (furniture fronts carry repeated internal
seams below the true one); orientation corroboration is a bonus term
only, because clean tops are flat isotropic fields with no
orientation to flip. Low confidence returns None (caller keeps the
extruded-sprite fallback, per §12 rules). `box_model` (G.1) builds a
cuboid with faces un-projected from their sprite regions;
`column_model` (G.2) builds a front wall + top cap shell on the
32x32 sub-grid whose front projection reproduces the sprite
silhouette exactly. Parent re-verified in the main tree
(`props_gate` 2/2; lib 27/27): f_counter boundary 20 (conf 0.81),
f_bookcase 27 (0.76), f_table 19 (0.86), f_fridge 17 (0.55 — weakest
pass, top only 8 rows on a 47-row front). Known failure: f_locker
segments at its mid door-seam (the stacked-seam mode) — excluded
from the gate set; per §12 that's a G.3 per-id override case, not a
cleverer global rule. Depth uses a documented 2:1 foreshortening
heuristic, clamped to the tile. Showcase:
`examples/props_showcase.rs` -> target/stage-g/props.png, presented
to Joey 2026-11-16 for the §12 quality sign-off before G.3.

## 14. Stage E outcome (2026-11-16)

Stage E is DONE in the voxelizer (parent re-verified in the main
tree: `stage_e_gate` 3/3, lib 30/30, all prior gates green).
Eight new recipes on the Stage C/D framework, all passing the §8
targets unrelaxed (seam / mean ΔE_OK): concrete 1.045 / 0.0006,
brick 0.695 / 0.0130, stone 1.052 / 0.0069, metal 0.339 / 0.0051,
glass 0.901 / 0.0121, water 0.437 / 0.0138, road 0.968 / 0.0064,
shingle 0.816 / 0.0109 (family fetched by id from
t_shingle_flat_roof). Structured recipes keep features off the wrap
edge by phase shift (the Stage C wood-floor lesson). `shore.rs`
(§10.1, voxelizer half): shoreline autotiling from the 8-neighbor
mask — chamfer distance field over the 96x96 neighborhood frame,
radius-4 blur for corner rounding, ±2.5px fBm wobble, fixed material
heights (water 0.0 / dirt 0.45 / grass 1.0) composited via
height_blend, shallow tint within 14px of the waterline,
`depth_shade` from field erosion. Gate: convex-corner texels are
dirt (never water), all-water mask returns the water input
byte-for-byte, pond mosaic corner-tile water coverage 38.6% vs the
<60% target. HONEST VISUAL VERDICT (parent, target/stage-e/pond.png):
the mosaic reads as a rounded-square pond — corners genuinely round
and the dirt bank ring is continuous, but the straights stay
straight; the mockup's fully circular pond is NOT yet matched at
3x3-tile pond size, because the rounding radius is capped near one
tile. Closing that gap means a larger straight-bow (pull the
waterline further in mid-edge) — a tuning decision recorded here
for Stage H, not silently baked in. Engine-side mask assignment +
the §10.1 engine pond capture belong to the Stage F integration
round, as planned.

## 15. Stage F part 1 outcome — v2 format + world-space macros (2026-11-16)

DONE, parent re-verified in the main tree (full engine + mesher +
treebuild + voxelizer suites green on lavapipe, including the v1
golden gate at 0 mismatched px and the new `stage_f_gate` 3/3).
The v2 contract as implemented: v1 stays a bare JSON array and the
voxelizer default output is byte-identical to before (gate-asserted);
`--v2` wraps the identical materials array as
`{"version": 2, "materials": [...], "macros": [...]}` and writes
128px `macro_<layer>_color.png` / `macro_<layer>_height.png` layers
beside it. Grass/Dirt-class materials with a poolable family get a
macro (span_tiles 4); unpoolable families are omitted and keep v1
behavior. The engine loader accepts v1 and v2 and rejects other
versions with a clear error (unit-tested). Sampling: raster top
faces of macro materials carry world-position UVs from the mesher
(encoded in the layer word: flag bit 31, span bits 24-30, layer
bits 0-23) into a second texture array; RT `tex_albedo` branches on
the material's macro entry for +Z faces with uv from the world hit
position. The height layer is uploaded in the same array and proven
sampled via `--macro-debug-height` in both renderers; it does not
yet shade normal rendering. GPU proof: v2 vs v1 sample-scene
captures differ by >500 px in both renderers; mesher macro UV phase
repeats exactly every 4 tiles. NOT in this part (by design): border
height-blending between materials, generated end-caps/decals,
decoration scatter (§10.2) — Stage F part 2. This v2 carrier is
also what Stage G engine stamping will ride on.

## 16. Stage G weak-case fixes (2026-11-16, Joey's sign-off condition)

Joey's verdict on the §13 showcase: right direction, fix the weak
cases first. All four fixed, parent re-verified in the main tree
(`props_gate` 3/3, lib 32/32, all voxelizer suites green):
(1) Open-base rule in the box path — a front column is "grounded"
iff its bottom pixel is solid paint (alpha >= 254; the under-table
shadow in UltiCa is painted at exactly alpha 128); open fraction
>= 0.25 with a solid header band switches the box to slab-and-legs
(header + top slab, each grounded column run extruded through the
depth, under-space empty). Measured separation: table 0.81 open vs
counter 0.10 / fridge 0.16 / bookcase+locker 0.00, which stay solid.
(2) Column models are closed shells now: back panel at z=0
(silhouette mirrored, colors x0.7), side closure strips at each
row's opaque extremes, bottom strip — interior still hollow, front
projection still exactly the sprite silhouette (gate-asserted for
all five props). (3) f_locker: per-id boundary override
(`BOUNDARY_OVERRIDES` + `segment_for(id, sprite)`); boundary 28
chosen from the row profile (2-row cap: dark rim row 26, brightest
row 27, body widens at 28). The gate proves the override is the
fix — bare `segment` still picks the wrong seam at 38. The override
bypasses the confidence floor (honest score 0.33: the cap seam is
genuinely weak) but keeps silhouette gating and side detection.
(4) Confidence recalibrated: the margin-over-runner-up term
penalized exactly the tall furniture the first-strong-edge rule
exists for (door seams stronger than the top seam); replaced with a
seam-support term (fraction of seam columns with a real per-column
break; fridge measures 1.00) + edge rescale. Fridge 0.55 -> 0.75;
counter 0.85, bookcase 0.84, table 0.92; negatives (f_chair,
t_grass) still None. Updated showcase (locker added as fifth row)
re-presented to Joey 2026-11-16 for the §12 sign-off.

## 17. Stage G unified solid model (2026-11-16, Joey's direction)

Joey's ruling on the §16 sheet: one model, a REAL 3D model — the
two variants confused the comparison, and the column shell showed
an open wedge on the fridge's side. `solid_model` replaces both as
THE model (box/column kept, deprecated): cross-section = the front
silhouette exactly, extruded through the full depth and filled at
every z (gate-asserted exhaustively per prop: counter 11,840 /
bookcase 16,128 / table 2,944 / fridge 19,232 / locker 4,608 voxels
checked); front plane = sprite pixels verbatim; open-base rule
unchanged (table under-space empty at every z). Top surface is
stepped — it follows each column's own silhouette top, colored from
the top region by the existing un-projection. Parent re-verified
in the main tree (props_gate 3/3, lib 34/34). Joey's follow-up
direction, same day: the next showcase must show front / back /
left / right views, and the faces the sprite never shows (sides,
back) should be EDUCATED DECISIONS, not edge-stretch or mirrored
front detail — that inference pass is the next G work item.

## 18. Stage G inferred unseen faces + 4-angle showcase (2026-11-16)

Joey's requirement: unseen faces are educated decisions, and the
showcase shows front / back / left / right. Implemented in
`solid_model` (geometry unchanged): `row_field()` computes each
front row's modal solid-paint color (edge 2 columns trimmed;
deterministic tie-break). Side interior cells = field x0.85; back
plane = field x0.7 — horizontal bands wrap at their heights,
vertical front detail (door splits, handles) never dominates a
row's mode so it never wraps. A detected side sliver (real sprite
evidence) still wins over inference on its side. Parent re-verified
(props_gate 4/4, lib 34/34): fridge — 532 side cells == independently
recomputed field x0.85, 195 back cells == field x0.7, 45 of them
over front detail the old mirrored back would have ghosted;
bookcase shelf band wraps on the non-sliver side. The showcase
renderer also had a handedness bug (screen basis mirrored every
render); fixed to a right-handed basis, so FRONT now matches the
sprite's handedness — earlier sheets' model renders were mirrored
(the sprite column was not). Showcase is now sprite | FRONT | BACK
| LEFT | RIGHT at ~33 deg elevation. Remaining honest weakness
(parent, from the sheet): f_locker's depth is 4 sub-cells (its cap
is 2 rows at the 2:1 rule), so side views read paper-thin — a
category minimum-depth decision, recorded for Joey's call.

## 19. Stage G fixes round 3 — depth floor, plain-panel backs, black artifacts (2026-11-16)

Joey approved the minimum-depth rule, rejected rule-derived backs
("make educated guesses"), and flagged a black column on the
fridge's right view + a black spot on the bookcase. All fixed,
parent re-verified (props_gate 7/7, lib 36/36). ROOT CAUSES (both
in color data, not the renderer — its face factors are
1.0/0.82/0.64/0.45 and the offending cells were literally
[0,0,0]): (1) fridge black column = transparent-black fringe —
the sprite's right edge column x=31 rows 23-55 is [0,0,0,128]
(no color data) but the alpha>=128 silhouette test admitted it as
structure, so it extended the extent and was copied/slivered as
pure black (measured L=0.00 over the front 6 depth cells). Fix:
a STRUCTURE test (opaque AND not fringe) — fringe never extends
the silhouette and is never a color source. (2) bookcase black =
shelf-shadow rows used as material: shelf interiors are fully
opaque pure black (280 px, 24% of the front), so their row field
was [0,0,0] and got painted across sides/back. Fix: shadow sources
(L < 0.15) are recesses, not material — side sources borrow the
nearest non-shadow row's field. MINIMUM DEPTH: floor-standing
props (silhouette >= 24 rows) get d >= ceil(width/3); locker
4 -> 11, others unchanged (gate-pinned). BACKS are now plain
panels by construction: one panel in the front region's primary
material (largest non-shadow palette cluster, k=5 — bookcase ->
wood [69,48,10], not its 24% black cluster), L modulated by
stretched fBm grain (amplitude by material: wood ±9%, flat/metal
±3%, else ±5%); two panels only when the top is a distinct slab
(top/front mean distance >= 0.15 AND top >= 1/3 of silhouette —
counter and table fire; the fridge's teal cap at 8/47 rows
correctly does not). A no-black gate now covers all five props
(986-2,200 inferred cells each, all L >= 0.10).

## 20. Stage G albedo-only models (2026-11-16, Joey's ruling)

Joey: the models carry baked light; the engine provides lighting,
the voxelizer provides textures. All directional multipliers are
removed from `solid_model` (side x0.85, sliver x0.85, interior
x0.7, panel offsets): sides = de-shaded field at full value, back
panel = de-shaded primary at full value (fBm grain stays — texture,
not light), interior = front pixel verbatim. Derived colors are
DE-SHADED so the front's baked top-to-bottom falloff doesn't leak:
per-row OKLab-L gain (median row L / row L, clamped [0.80, 1.25],
a/b preserved), aligned with Stage C's deshade_palette — the panel
primary's L is re-anchored to the palette's weighted median L
(measurement showed the largest cluster can be a band edge of the
ramp, which no profile gain can correct). Sliver pixels get the
row gain plus a re-anchor onto the row's de-shaded field (removes
the sliver's baked face shading, keeps its pattern; assumes sliver
and front are the same material — stated limit). Front plane and
top face stay verbatim (they ARE the texture). New synthetic gate:
one uniform material with a painted light ramp + darker sliver
must yield sides/back at the original uniform color (measured:
side max Δ 0.013 vs 0.025 tol, back max Δ 0.023 vs 0.05, back
row-mean L spread 0.012 vs 0.03). Parent re-verified: props_gate
8/8, lib 36/36. The deprecated box/column models keep their
multipliers (reference-only, noted).

## 21. Stage G case-by-case side construction (2026-11-16, Joey's ruling)

Joey: the bookcase's sides should be flat wooden slabs; missing
detail can't be filled by a global rule — it goes case by case.
The row-field side wrap is replaced by CONSTRUCTION PROFILES:
`Construction { Panel, Structural, FieldWrap }` via
`construction_for(id, sprite, seg)` with a per-id
CONSTRUCTION_OVERRIDES pin table (empty today; pins accrete as
cases are reviewed, mirroring BOUNDARY_OVERRIDES). Defaults:
open-base -> Structural (table: sides are legs + apron geometry);
everything else that segments -> Panel (flanks = the back-panel
treatment: de-shaded primary + grain, grain running vertically on
flanks; two-panel split applies). FieldWrap (the old row wrap) is
assigned to NOTHING by default — it exists for per-case pinning
(e.g. slatted crates). A detected sliver still wins on its flank
(real evidence over inference); a shadow/missing sliver sample on
a Panel flank now falls back to the panel, not the field. Parent
re-verified (props_gate 9/9, lib 36/36): bookcase right flank 420
cells panel-uniform (variance 0.00002), left flank = sliver
un-projection asserted cell-exact (312 cells); fridge flank 532
cells panel (variance 0.00001) — its old side==field test was
REPLACED by the panel assertions; locker flank 315 cells panel, no
louver banding; counter flank carries the two-panel split (30
stone rim cells over 330 body cells). The albedo synthetic's left
flank is Panel by default and now uses the panel budget (measured
side max 0.023 overall).

## 22. Stage G.3 archetypes — lab set (2026-11-16)

DONE (generation side), parent re-verified in the main tree
(archetypes_gate 6/6, lib 36/36, props_gate 9/9). archetypes.rs:
`archetype_for(id)` (explicit table + pins) and parametric builders
on the signed-off base (albedo-only, de-shaded palettes,
deterministic). Mapping: ShelfUnit <- f_bookcase, f_rack,
f_utility_shelf (+ warehouse/vitrified via looks_like; vitrified
has NO sprite — assigned, unrenderable today); ContainmentTank <-
f_standing_tank, f_storage_tank (CDDA has no true containment
furniture: f_alien_pod doesn't segment, f_fvat_* are barrels —
deliberately unassigned; specimen skipped, nothing reads one
cleanly); ConsoleDesk <- f_console, f_console_broken, gas consoles;
DissectionTable <- f_surgical_table (NO sprite exists — fitted and
shown on f_slab); Bunk <- f_bunkbed/_down (2 levels), f_triple_
bunkbed (3), sprite via looks_like to f_bed. Fitted vs assumed is
documented per archetype in the module: shelf boards from the
front's recess bands (bookcase: 5 bands -> 6 boards, compartments
genuinely OPEN), tank liquid line from the strongest row-mean step
(fired at 11/39), console screen rect from the accent-pixel
bounding box ([0,196,0] green), slab/pedestal proportions and bunk
posts/ladder assumed. Gate-design note: the albedo darkness floor
is taken from RAW palette clusters because de-shading lifts dark
clusters toward the median by design. Engine stamping of models
(base + archetypes) rides the v2 models section (next commit).

## 23. Stage G v2 models carrier + the stamping blocker (2026-11-16)

Carrier DONE, parent re-verified (stage_g_stamp_gate round-trip
3/3, loader units 6/6, golden gate 0 px, engine+voxelizer suites
green; release-profile full-UltiCa --v2 run emits 119 models).
Format: v2 "models" table — palette (distinct RGB, first-appearance
order) + voxels as hex string, 4 digits per voxel = palette index,
0xFFFF = EMPTY, PropModel's (y*d+z)*w+x layout; key omitted when
no model exists (v2-without-models byte-identical to Stage F).
Emitter builds solid_model_for for segmenting f_* materials;
loader validates (hex length, palette range, dims) with named
errors. Orientation contract pinned in materials.rs docs (x east,
y up, z=d-1 front to the tile's south edge).

STAMPING BLOCKED ON AN ARCHITECTURE FACT THE ORIGINAL §12 PLAN
MISSED: the engine has NO sub-tile world grid. WorldScale (Phase 0)
is 1 tile = 1 block footprint, 3 blocks per z-level; both renderers
consume that block grid, and nothing consumes even the voxelizer's
existing sub_model extrusion data — furniture renders today as
textured BLOCKS. A 32-wide prop model has nowhere to land without
either (a) a fine-grid world representation (major rework of the
mesher/tree/RT core) or (b) a model-instancing render path (props
drawn as per-instance voxel models at tile positions, parallel to
the billboard pass; block grid keeps terrain/walls/floors). (b) is
the recommended route; it is Joey's call. Picking/LOS/collision
unchanged under either (game tile data governs).

## 24. Stage G.3 rework spec — view-faithful generation (2026-11-16)

Joey's verdict on the §22 archetypes: only ShelfUnit correct. Root
cause (his correction, with a photo of the real object): a prop
model must be a 3D model OF WHAT THE SPRITE DEPICTS — the sprite
is a 3/4-view illustration of a real object; identify the referent
first. The generator's hidden assumption was that every sprite is
a FRONT view (top band / front face / depth). The failures are
sprites in other views: f_bed and f_slab are TOP-DOWN (the sprite
is the top face; the bunk and the pedestal table were inventions),
f_standing_tank is END-ON (the sprite is the end cap of a
HORIZONTAL plastic leg tank — round cap to the viewer, barrel
curving over, hatch-neck fitting on top, molded leg stands), and
f_console is a front view whose archetype wrongly replaced the
front with palette blocks. Approved mockup reference sheets (the
acceptance target) live in target/stage-g/mockups/: tank.webp,
console.webp, slab.webp, bed.png (bed BACK = one plain wood slab
occluding the mattress completely — Joey's ruling).

THE FIX (Joey approved the approach 2026-11-16):
1. VIEW CLASSIFICATION before segmentation: view_for(id, sprite)
   -> Front | TopDown | EndOn. Heuristics propose, per-id pins
   (VIEW_OVERRIDES, beside the existing override tables) decide;
   default Front.
2. ONE FAITHFUL BUILDER PER VIEW:
   - Front -> the signed-off pipeline, code untouched (solid_model
     + Construction profiles + ShelfUnit, which survives because
     the bookcase front depicts open shelves).
   - TopDown -> the sprite IS the top face, verbatim. Structure
     from the sprite's border: framed-surface construction (bed:
     border = frame extruded down, interior = mattress surface,
     head-end strip rises as a plain-slab headboard that occludes
     the mattress from behind) vs terrace construction (slab: the
     concentric rings are a heightmap — ring footprints at
     increasing height via distance-from-edge; identical from all
     sides, flat level top).
   - EndOn -> the sprite is the END face; extrude its cross-section
     along the depth axis (tank: disc -> horizontal cylinder),
     fittings placed as depicted (hatch neck + brass fitting on
     top near the front end, leg stands under the barrel). Barrel
     length is unobserved by the sprite; default ~= cap diameter,
     pinnable per id.
   - Console stays Front but its archetype is rebuilt: full sprite
     front as the front plane (no palette replacement), depth
     stepped (terminal block deep, desk/base shallower), the
     base's depicted open compartment modeled open.
3. ARCHETYPES DEMOTED: Bunk and DissectionTable DELETED (their
   sprites depict a single bed and a terraced slab). What remains
   is a fittings layer keyed by view class + id pins.
4. APPROVED-MODEL PROTECTION: the Front path is byte-identical
   code; new builders fire only for TopDown/EndOn-classified ids
   (initially exactly f_standing_tank/f_storage_tank, the f_bed
   chain incl. bunkbed ids, f_slab). A regression gate pins a hash
   of every approved model's voxel data (f_counter, f_bookcase
   solid + ShelfUnit, f_fridge, f_locker, f_table), computed on
   the pre-rework tree; the rework must not move a single voxel.
5. ACCEPTANCE: structural gates per builder (tank cross-section
   is a disc at every depth step; bed back face 100% wood; slab
   invariant under 90-degree rotation, heights monotonic inward;
   console front plane == sprite front exhaustively) + the
   4-angle showcase beside the mockup sheets for Joey's sign-off.
