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
| 5. How much original sprite look stays for furniture/decals | **Joey's call at the Stage E review.** Default: decals keep the sprite's alpha shape (draft §4.8); furniture stays extruded-sprite models (plan.md §8). Flag it in the Stage G sign-off, don't pre-decide. |
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
Total estimate stays 8–10 weeks to Stage G; Stage A is a day or two and
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
  autotiled curve is expected to carry it; revisit only on Stage G
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
