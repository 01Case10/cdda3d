# Phase 2h — Engine integration: the live frame loop

**Status: GATE PASSED (conditional) 2026-11-15.** Conditions: live
game data seating waits on 0.1/0.2 (the fork build, like every
in-game gate); gate 4.12's in-loop number is a llvmpipe number —
target-hardware re-measure outstanding, same class as 2c.9/6.10;
the windowed app itself has not run live on this VM (headless
lavapipe only, the standing condition). Gates:
`crates/engine/tests/frame_gate.rs` (run standalone with
`--nocapture`; GPU suites go vacuous under parallel adapter
contention).

## What landed

One composition, `engine::frame::render_frame`, renders every
frame the engine produces — windowed or headless — in the
canonical order **world → fields → billboards → post → UI**
(`frame::FRAME_ORDER`, asserted in code structure and in
execution via `FrameStats::passes`).

- **World**: raster meshes via the shared `scene_prep`
  preparation, or the Sparse-Voxel-64Tree tracer
  (`rt::render_rt_frame`). With a lighting scene, the raster
  preparation is the lit one (visibility mask in the filter step
  + per-vertex bake, Phase 7 §12's prescribed wiring) and the RT
  world is built through `rt::world_for_levels_lit` (BLANK
  voxels never enter the tree; MEMORIZED voxels carry
  `lighting::present_color` in their albedo and `LIT_MEMORIZED`
  in the side table, with the tracer's own `FLAG_MEMORIZED`
  desaturation deliberately not set, so the pinned presentation
  is applied exactly once).
- **Fields**: the fx soft-particle pass (2g.1), depth-tested
  against the world depth — for RT, against the converted
  projected depth.
- **Billboards**: `billboard::render::BillboardPass` between
  fields and post. Two supporting additions: `prepare_with_view_proj`
  in the billboard crate (the pass now draws with the *world*
  camera's view-projection instead of its harness camera — the
  harness camera's 55° FOV disagreed with the engine's 60°), and
  shadows-then-sprites draws for creatures (packed composites)
  and item piles (raw atlas) in the one frame.
- **Post**: the fx HD-2D stack, always the final world pass.
- **UI**: a resident `ui_share::UiPass` (new in ui-share; the
  Phase 4 compositor was headless-one-shot before) draws the
  store frame's masked tight pixels last, over the post output.

`capture::run_capture` now delegates to `render_frame` + readback
(it previously held its own copy of the composition). The
delegation is byte-stable: all eight 2g.3 goldens still match at
**0 mismatched pixels**, and the 2g.3 negative test still fails
the gate (one material changed → 1.730% mismatch, detected).
This also closes **2g.1 deviation 1**: the windowed app renders
through the same offscreen composition and presents it with
`frame::PresentBlit` (a plain sample — the composed texture is
sRGB, so sampling decodes and the sRGB surface re-encodes; the
pow-2.2 compensation of the old RT blit belonged to the RT
capture convention, not to presentation). §7's capture-parity
rule now covers the whole frame, not just the world pass.

Scene inputs arrive on `CaptureArgs` (the one parameter struct
for both paths): `lighting: Option<LightingScene>`,
`billboards: Option<BillboardScene>`, `ui: Option<UiPrepared>`,
all `None` = the pre-2h render. Stub constructors stand in until
the fork runs: `LightingScene::fake_house` (the Phase 7 fixture
grid + an `Environment` from hour × weather),
`BillboardScene::fixture_crowd` / `ultica_crowd` (the Phase 6
real-capture composition seated in the fake house),
`frame::stub_ui_store` + `frame::prepare_ui` (the Phase 4
per-notification path: validate the `UiFrame` notification
against the store, read the newest complete frame, mask its
terrain rect).

The windowed app (`main.rs`) is a shell over the loop: a
`CaptureArgs` template synced in place per redraw (camera,
cutaway, size, animation time — no per-frame clone; the atlas is
an image), presented via `PresentBlit`, paced by the existing
dirty-flag `FramePacer`. New flags: `--effects`,
`--billboards fixture|<atlas-dir>`, `--light-hour H`, `--ui-stub`
(also accepted by `--headless-capture`). Input: clicks route
through the Phase 4 `InputRouter` (world picks run the DDA
against the scene; panel clicks produce the router's `InputT`),
keys Z/Enter/I produce protocol inputs via `key_to_input`, and
camera drags are gated on the router's mode. Inputs land in an
in-process **stub sink** (logged, counted) — the engine has no
IPC socket client yet; seating the sink onto the fork's input
channel is 0.1/0.2 territory, and the sink's contents are
already protocol-shaped.

## Gate evidence (lavapipe, 640×360 unless noted)

- **2h.1** — `FRAME_ORDER` asserted; a fully composed frame's
  executed passes equal it exactly. Windowed route (render_frame
  + PresentBlit) vs the composed frame: **0.000% mismatch, mean
  |err| 0.000**. Headless `run_capture` of the same scene vs the
  composed frame: **0.000%**.
- **2h.2** — real UltiCa crowd through the engine (sprites from
  `assets/ultica_run1`, labeled: real atlas, fixture world).
  Skeleton outside the north wall, camera south above the house
  looking north: removing it changes the frame by **1 pixel at
  1 LSB in one channel** — packed-atlas layout shift moving the
  *window zombie's* sampled texel by a sub-texel UV step through
  the window's alpha blend (measured; the gate bounds rounding
  at ≤16 px / max delta ≤2 / zero pixels beyond the 2g.3
  tolerance, which a real leak exceeds by orders of magnitude).
  Removing the window zombie instead: **0.028% of pixels beyond
  tolerance** (~65 px, the window opening's worth, blended
  through the window's alpha-0.35 pass; gate bound 0.01%).
  Alpha picking: center ray on the hulk returns entity **8**;
  on the skeleton, entity **7** (picking is per-sprite alpha —
  world occlusion is the depth test's job, asserted above).
  Picking aims at mid-body height: a 32×48 sprite stands on its
  tile, so a ray through the tile *center* (1.5 units up)
  grazes its top edge.
- **2h.3** — day/night pair (same grid, noon vs midnight,
  clear): background silhouettes identical (>99.9% agreement —
  the pair differs only in lighting); median per-channel ratios
  **0.543 / 0.591 / 0.701** vs the Phase 7 curve's encoded
  expectation **0.572 / 0.617 / 0.733** (tempered factor
  0.4 + 0.6×0.22 = 0.532 × midnight tint (0.55, 0.65, 0.95),
  through the sRGB encode). BLANK floor patch (3×3 tiles):
  RGB distance to background **45.5 → 5.0** (raster),
  **160.4 → 6.1** (RT) when marked. MEMORIZED north-wall run:
  saturation **0.054 → 0.016** (raster, ratio **0.30** — the
  pinned saturation-keep, exact) and **0.119 → 0.033** (RT,
  ratio 0.28); raster brightness ratio **0.822** (the patch
  mixes marked-run vertices at the pinned 0.45 with
  interpolation toward the unmarked run — the gate band is
  0.5–0.95, asserting real dimming short of the raw factor).
- **2h.4** — CPU half: synthetic click round-trips to tile
  **(6, 6, 1)** (steep camera; the analytic first-hit tile —
  at pitch 45 the same click correctly picks (7, 5, 1), the
  slab entry tile, verified against the hand-computed top-plane
  crossing (7.57, 5.43)); a panel click produces no world pick;
  a key input survives the FlatBuffers envelope round-trip
  unchanged. GPU half: the sidebar pixel in the composed frame
  equals the prepared tight pixel it was stretched from (±2);
  with the store's terrain rect painted **opaque red**, the
  composed frame still shows the world there (pixel-equal to
  the no-UI frame within tolerance) — the mask comes from the
  frame's own reported rect, and the prepared tight alpha
  inside the rect is 0. **Gate 4.12 in-loop**: median
  **8.23 ms**, p99 **36.56 ms** over 30 composed frames
  (budgets: median <50 ms, p99 <100 ms).
- **2h.5** — composed golden `tests/golden/house_composed.png`
  (world + fields + fixture billboard crowd + post + stub UI
  sidebar + the lighting bake, fx time pinned at 1.25 s)
  committed and green under the 2g.3 tolerance model;
  regenerating is `UPDATE_GOLDEN=1 cargo test -p engine --test
  frame_gate`. The 2g.3 negative test still fails (above).

## Findings recorded

1. **4.12 measurement correction.** `FrameStats::ui_stage`
   drains the queue (`poll(Wait)`) *before* starting its timer.
   Without the drain, the stage's closing poll waits on the
   whole frame's queued GPU work and the "UI stage" measures
   the world/fx passes' backlog — 105.79 ms median / 273.86 ms
   p99 on llvmpipe for this scene, vs the stage's own
   8.23 / 36.56 ms. Phase 4's budgets are segment budgets; the
   drain makes the in-loop number the same segment.
2. **Per-vertex bake resolution = mesh vertex density.** The
   Phase 7 §12 wiring (mesh the unbaked map, bake per-vertex)
   expresses presentation at vertices. The fake house's floor
   is one greedy-merged quad: a MEMORIZED patch strictly inside
   it has no vertices, and its shading interpolates the quad's
   corner tiles — measured: marking a 3×3 interior floor patch
   changed **0 pixels**. §12's "identical Hidden/Memorized
   handling" therefore holds at vertices, not per tile, on the
   raster path. (Hidden is unaffected — it is a data-level mask.
   The RT path is per-voxel and exact.) If per-tile memorized
   fidelity on merged surfaces is ever required, the lever is
   the per-block baker (`bake_chunk_map`) ahead of meshing,
   whose color changes break greedy merges along light
   boundaries — a deliberate trade, not taken here.
3. **Passes are recreated per `render_frame` call** (field/post/
   billboard/UI pipelines and the texture spec are rebuilt per
   frame), following the 2g.1 capture precedent. On llvmpipe
   this is invisible next to software rasterization; on target
   hardware a resident-pipeline refactor is the known next step
   if frame cost demands it. The windowed pacer only re-renders
   on dirty state, so idle cost is zero either way.
