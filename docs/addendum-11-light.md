# Addendum 11 — Light: how CDDA handles it, and the `light_frac` + `light_tint` amendment

Date: 2026-11-23 · Status: **DESIGN RECORDED, NOT IMPLEMENTED**
Amends: Addendum 2026-11-23 (11) in `plan.md` (local light emitters —
the combined model), Phase 7 (`docs/phase7-lighting.md`), and
Phase 11 slice S3 (`docs/test-sim-engine.md` §2.3, §7).
Decision: Joey, 2026-11-23 in the light-rendering thread — carry
colour as well as falloff; colour does not break the game.

## 0. Terms

- **lightmap (`lm`)** — CDDA's per-tile, per-quadrant float light
  energy: `level_cache.lm[x][y]` is a `four_quadrants<float>`, with
  a parallel self-light cache `sm[x][y]` and a colour cache
  `light_color_cache[x][y]` of `light_color_rgb{float r,g,b}`
  (`src/lightmap.h`, `src/lightmap.cpp`).
- **`lit_level`** — the 7-value enum (`DARK, LOW, BRIGHT_ONLY, LIT,
  BRIGHT, MEMORIZED, BLANK`, `src/lightmap.h:96`) that
  `map::apparent_light_at()` collapses the float lightmap into.
  This is the only per-tile light the wire carries today.
- **`light_frac`** (new, this addendum) — a per-tile `ubyte`
  0..255 carrying the continuous position of a tile's apparent
  light between the player's vision threshold and full light,
  defined exactly as upstream's smooth lighting defines it (§2).
- **`light_tint`** (new, this addendum) — a per-tile normalised
  RGB hue (`[ubyte; 3]`), defined exactly as upstream's
  `illumination_hue()` defines it (§3). A multiplier, never an
  energy: its brightest channel is 1.0 by construction.
- **ceiling rule** — the Phase 7 / Phase 10 rule that renderer
  light (sky GI, declared emitters) is multiplied by the tile's
  game-authored lit factor last, so no renderer light can reveal
  a tile the game marks dark or unseen.

## 1. Finding: CDDA is continuous inside, discrete at the surface

CDDA computes light continuously, then throws most of it away at
the exact point a renderer reads it:

1. `build_sunlight_cache()` propagates sunlight top-down through
   per-tile transparency (`LIGHT_TRANSPARENCY_SOLID = 0`,
   `LIGHT_TRANSPARENCY_OPEN_AIR = 2.3 / MAX_VIEW_DISTANCE ≈
   0.038`, smoke ≈ 0.38; attenuation `exp(-dist * t)`), blocked
   by floors, floored at an inside baseline (`LIGHT_AMBIENT_LOW
   = 3.5`, or `LIGHT_AMBIENT_DIM * 0.8 = 4.0` in bright daylight).
2. `generate_lightmap(z)` starts from that sunlight and adds
   every artificial source — terrain/furniture/field
   `light_emitted` with its `light_color`, emissive items,
   characters, monsters, and vehicle cone/wide-cone/circle
   lights — via shadowcast rays (`apply_light_source` /
   `apply_light_arc`) through the same transparency. Coloured
   contributions accumulate additively into
   `light_color_cache`, luminance into `lm` (per-quadrant max).
3. `map::apparent_light_at(p, cache)` (`lightmap.cpp:1107`)
   computes `apparent_light = vis * lm.max()` — `vis` from
   `seen_cache`/`camera_cache`, quadrant-restricted for opaque
   tiles — and then thresholds it into `lit_level` only:
   `> LIGHT_SOURCE_BRIGHT (10)` or a self-light source →
   `BRIGHT`; `> LIGHT_AMBIENT_LIT (10)` → `LIT`;
   `>= vision_threshold` → `LOW`; obstructed or beyond
   `unimpaired_range` → `BLANK` (or `BRIGHT_ONLY` if a light
   source is sensed through the obstruction). `vision_threshold`
   is `Character::get_vision_threshold(ambient at player)`,
   capped at `LIGHT_AMBIENT_LOW`.
4. `map::get_visibility(ll)` collapses further for classic
   drawing: `LOW/LIT/BRIGHT → CLEAR` (detail), `BRIGHT_ONLY →
   LIT` (bright silhouette only, no detail), `DARK/BLANK/
   MEMORIZED → DARK/HIDDEN` (memory or black).

So the classic renderer gets a per-tile verdict, not a gradient:
shown in detail, shown as a bright blob, or not shown. All
falloff inside `LOW`/`LIT` and all per-tile colour are computed
and then discarded at this boundary. Our Phase 1 schema copied
this boundary verbatim (`lit` + `seen` only), the fork adapter
(`fork/src/voxel_ipc/voxel_real_adapter.cpp:121`) exports only
`apparent_light_at()`'s enum (and quantises `seen` coarsely to
0/64/255, having no public per-tile seen getter), and the engine
adapter (`crates/lighting/src/grid.rs`, `proto_adapter.rs`)
hardcodes `TileLight.tint = [1.0, 1.0, 1.0]`.

## 2. The sanctioned recovery: upstream smooth lighting

The pinned tree already contains upstream's own answer for its
SDL renderer, `src/smooth_lighting.{h,cpp}`, and this addendum
adopts its definitions rather than inventing new ones:

- `classify_light_cell(ll, apparent_light, vision_threshold)`:
  `BRIGHT → light = 1.0, detail = true`;
  `LOW/LIT → light = light_above_threshold(apparent_light,
  vision_threshold)`, `detail = true`;
  `DARK / BRIGHT_ONLY / MEMORIZED / BLANK → light = 0,
  detail = false`.
- `light_above_threshold(a, t) = clamp((a - t) /
  (LIGHT_AMBIENT_LIT - t), 0, 1)` — 0 at the vision threshold,
  1 at and above full light.
- The gradient is then filtered spatially (cubic B-spline,
  reach 2) under a **barrier reach-mask**: a tile only averages
  neighbours reachable without crossing an opaque tile, so
  smoothing never leaks light through a wall. Our Phase 7
  air-side sampling is the same discipline in a different form
  and stays as-is.
- Colour: `illumination_hue(light_color_rgb, scalar_light)`
  returns a hue whose brightest channel is 1.0; the coloured
  share fades in smoothly as the coloured light itself rises
  from `LIGHT_AMBIENT_LOW` to `LIGHT_AMBIENT_LIT`. Hue is
  derived from the tile's *own* light, never from what the
  avatar sees of it, so special vision does not change hue.

Key property, and the reason this is safe: gradient and hue are
recovered **only inside tiles already marked visible in
detail**. `detail = false` tiles gain nothing. Visibility —
what may be shown at all — is unchanged.

## 3. Decision: two additive wire fields

Add to `SubmapBlock`, `TileChange`, and `LightUpdate` in
`schema/cdda_ipc.fbs`, appended last in each table (the
`outside` precedent from Phase 10 slice 10.1: additive /
non-breaking, old readers ignore them, old data reads as
absent):

| Field | Type | Absent means | Source (game side) |
|---|---|---|---|
| `light_frac` | `[ubyte]` per tile in blocks / `ubyte` in `TileChange` | Derive from `lit`/`seen` exactly as today (Phase 7 bands) | `round(255 * light_above_threshold(apparent_light, vision_threshold))` for `LOW`/`LIT`; `255` for `BRIGHT`; `0` for `DARK`/`BRIGHT_ONLY`/`MEMORIZED`/`BLANK` |
| `light_tint` | `[ubyte]` ×3 per tile (interleaved RGB) in blocks / 3 `ubyte`s in `TileChange` | White `[255, 255, 255]` | `round(255 * illumination_hue(light_color_cache[p], lm[p].max()))` |

Notes on the definitions:

- **Do not send raw `light_color_rgb`.** It is unbounded
  additive energy; sending it as a multiplier would
  double-count intensity (once in `lm.max()`, once in the
  colour). The normalised hue is the only safe form, and it is
  upstream's form.
- `apparent_light` for the frac must come from
  `map::apparent_light_helper()` (the same value
  `apparent_light_at()` thresholds), read only when
  `needs_apparent_light(ll)` — `LOW`/`LIT` — exactly as
  `smooth_lighting::encode_lightmap_layer()` does. The fork
  adapter already runs `update_visibility_cache()`; the
  `visibility_variables` cache it holds carries
  `vision_threshold`.
- Colour cannot break the game, for the same reason frac
  cannot: tint is a per-channel multiplier ≤ 1.0 applied to
  light whose intensity and visibility `lit_level` already
  authorised. It cannot brighten a `DARK` tile, reveal a
  `BLANK` tile, or add detail to a `BRIGHT_ONLY` tile.
- Wire cost: +4 bytes/tile. A full 132×132 snapshot gains
  ≈ 70 KB against the measured 1.47 MB TileDefs handshake;
  Deltas and LightUpdates carry the fields only for changed
  tiles. Not a design constraint.
- The sim (Phase 11) is a second producer of these fields and
  must compute them by the same definitions from its own
  light model (S3, §5 below), so engine gates mean the same
  thing against sim data and fork data.

## 4. Engine consumption (what changes, what does not)

- **Visibility does not change.** `visibility_of(lit, seen)`
  still decides `Visible / Memorized / Hidden`; `BLANK` still
  fails closed, is not meshed, and does not enter the RT tree.
- **Intensity:** `crates/lighting` `TileLight` gains the frac;
  `base_intensity()` uses it in place of `seen/255` as the
  within-visibility ordering for `LOW`/`LIT` when the field is
  present, and falls back to the current band mapping when it
  is absent. The exact band composition is an implementation
  decision for the build slice, constrained by: gate 7.1's
  strict monotonicity in `lit_level` must be re-pinned
  failing-first (Joey's regression rule), and `DARK` /
  `BRIGHT_ONLY` keep their no-detail presentation — frac is
  0 there by definition, their bands do not move.
- **Tint:** `proto_adapter` / `grid::tile_from_proto` fill
  `TileLight.tint` from `light_tint` instead of the hardcoded
  `[1.0, 1.0, 1.0]`. Everything downstream already consumes
  tint: `LightSample::multiplier()`, smoothing's tint
  averaging, both bakers, `present_color()`. No renderer
  change is needed for colour to appear.
- **Ceiling rule unchanged.** Sky GI (Phase 10) and declared
  emitters (Addendum 11 combined model) are still multiplied
  by the tile's lit factor last; that factor now carries frac
  and tint instead of a flat per-level value.
- **Global tint is not replaced.** `EnvironmentUpdate.tint_rgb`
  (weather / twilight, global) and per-tile `light_tint`
  compose multiplicatively, as global and local light do in
  the game.

## 5. Relationship to the emitter model, and to S3

CDDA *does* bake emitter cones into `lm` (§1 step 2) — the
Addendum 11 statement "the game does not bake the cone into
the grid" describes **our renderer's shading**, not CDDA's
internals, and is clarified here: the fork's frac/tint export
will inevitably reflect CDDA's baked cone as a *visibility and
ceiling verdict* ("this tile is lit, this warm"), while the
renderer still transports declared emitters itself for
per-pixel falloff and RT shadows. Double-counting — baked
frac raising the ceiling *and* renderer transport adding the
same light — is a real risk and is **an open question for the
emitter prototype slice, not resolved here**: candidates are
(a) accept it (ceiling is a cap, transport is capped by it),
or (b) fork-side, exclude declared-emitter sources from the
frac computation. No decision is recorded until the prototype
measures it.

Phase 11 slice S3 (visibility & environment in the sim) now
includes: the sim computes `lit` by a CDDA-shaped thresholding
of its own apparent light, `light_frac` by §3's formula, and
`light_tint` by §3's hue rule from its light sources' colours;
T1 gains frac/tint assertions on the outside→inside walk and
T6 on the time/weather sweep. S3 is the first producer; the
fork adapter is the second, when 0.1/0.2 unblock it.

## 6. Status and gates for the build slice (future)

Design only — no schema, fork, sim, or engine code is changed
by this addendum. When built, the slice's gates are:

1. Schema round-trip: frac/tint survive Snapshot/Delta/
   LightUpdate in both Rust and C++ object APIs; absent fields
   decode as the §3 defaults and render byte-identical to
   today (every existing golden is the check).
2. Gate 7.1 re-pinned failing-first: monotonicity holds with
   frac as the ordering; a `LOW` tile at frac 255 is still
   darker than a `LIT` tile at frac 0.
3. A coloured-source fixture (e.g. a red light in a dark room):
   tiles in its pool render tinted toward red in both
   renderers, tiles outside it do not, and a `BLANK` tile
   inside the pool stays absent.
4. Sim parity: S3's hand-computed street_block expectations
   (noon and midnight) assert lit, frac, and tint per tile.

## 7. Emitter prototype — built and merged (2026-11-23)

The §5 prototype slice was built ahead of the frac/tint wire
work, to measure the capped baseline ("lid", option (a) shape)
before the A-vs-B decision. Branch `p11-emitter-proto`
@ 43e873d, merged to local main @ a1335f9 on Joey's word
(2026-11-23); parent re-verified (gate re-run + full
lighting/treebuild/engine suites green, no goldens regen'd).

- Shared model `crates/lighting/src/emitters.rs` (max 4
  emitters, packed uniforms, one WGSL cone+distance falloff
  snippet; visibility caller-supplied). Raster: per-fragment
  falloff, visibility 1.0 (v1 asymmetry). RT: falloff plus one
  shadow ray per emitter. Fixture-only `headlight-car` scene
  behind `--emitters`; empty set renders byte-identical.
- Measured (gate `crates/engine/tests/emitter_gate.rs`,
  lavapipe, empty render as control): raster pool luminance
  0.015 → 0.149 (per-vertex) / 0.020 → 0.190 (per-block);
  RT pool 0.061 → 0.620; RT occluded tiles behind the tree
  0.0686 → 0.0688 (shadow holds); DARK tile in the beam
  capped in both renderers; BLANK tile unchanged.
  Verdict so far: the lid is well-behaved; nothing measured
  forces option (b). A-vs-B remains Joey's call.
- Textured follow-up (2026-11-23, Joey's request): with
  `--textures assets/ultica_run1`, RT carries materials
  correctly (warm pool over grass texture, textured DARK
  patch, pond edge lit, shadow wedge intact; sampled pool
  RGB (175,167,71) vs outside grass (2,2,3)). **Raster
  textured path BUG:** the pool is effectively invisible
  (sampled pool (49,53,27) vs outside grass (51,56,59)).
  Root cause, `crates/engine/src/render.rs` textured branch:
  the emitter term is multiplied by `bake_ceil` = mean of
  the vertex colour, and the vertex colour is albedo × bake
  — so albedo and bake are applied twice to emitter light
  (the flat path has no such extra factor, which is why it
  works). The ceiling scalar must be the bake (lit factor)
  alone. Fix + a textured raster gate assertion are the
  follow-up; not yet done.
