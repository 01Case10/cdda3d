# Phase 7 — Lighting, FOV and atmosphere (`crates/lighting`)

Status: implemented 2026-11-15. Gate tests 7.1–7.9 all pass at
harness/data level (`cargo test -p lighting`: **9 gate tests + 5
proto-adapter tests, 14/14**). No gate in 7.1–7.9 needs the real fork
running in-game — every gate is proven on fake, game-style lit/seen
data, exactly as the phase task specified. Nothing in this phase is
blocked on Phase 0's 0.1/0.2 build condition.

## 0. Terms (defined on first use, per project convention)

- **lit_level** — the game's per-tile visibility-cache enum
  (`LitLevel` in `schema/cdda_ipc.fbs`, from CDDA's `src/lightmap.h`):
  `DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT` for currently-visible tiles,
  `MEMORIZED` for remembered-but-not-currently-visible tiles, `BLANK`
  for never-seen tiles. It is an enum, not a 0–255 light value.
- **seen** — the game's per-tile quantized visibility float,
  `0..=255` (`0` = never seen, `255` = fully seen). Within one
  lit_level it is the fine ordering.
- **Snapshot / Delta / LightUpdate** — the protocol messages of
  §6/Phase 1: a Snapshot is the full view (plus a separate
  **memory layer**, `memory_blocks`: remembered tiles carrying
  terrain/furniture only, `lit = MEMORIZED`, never live
  entities/items/fields); a Delta carries per-tile changes including
  lit/seen; a LightUpdate is a standalone lit/seen refresh for one
  12×12 submap block.
- **tint** — an RGB multiplier carried alongside intensity in
  `LightSample`. The game computes colored light (`light_color_rgb`,
  float RGB energy); v1 quantizes it to one tint per tile, but the
  API already carries a full `[f32; 3]` per tile, so per-tile color
  later is not a breaking change (plan §14 Q7, resolved this way).
- **bake** — multiplying light factors into mesher data (block colors
  or mesh vertex colors) to produce the scene the renderers consume.
- **occluding tile** — a tile holding a full-height opaque block (a
  wall). Geometry, not light; smoothing uses it to know where walls
  are without ever propagating light through them.

## 1. Scope, per the Phase 2c split

With the ray-traced renderer (Phase 2c), *illumination* — sun
shadows, ambient occlusion, emissive feel — belongs to the tracer.
This crate owns what only the game knows:

- **Visibility truth**: the game's lit_level/seen decides what may be
  shown at all. Both renderers consume the same `Visibility` /
  `VisibilityMask` contract from this crate, so neither can reveal a
  tile the game marks unseen.
- **Remembered-vs-seen presentation** (§3).
- **Day/night and weather state** (§6): pure mappings from game state
  to the global tint/fog/precipitation parameters both renderers
  read. The full-strength day/night curve feeds the tracer's sun
  term; the baker-facing blend is documented in §6.

This crate never recomputes FOV and never propagates light
engine-side: there is no flood fill and no blur over the tile grid
anywhere in it. All "smoothing" is *sampling* of game values (§4).

## 2. Light mapping (gate 7.1) — `mapping.rs`

Each visible level owns a disjoint brightness band; within a band,
intensity lerps linearly with `seen/255`:

| lit_level | band |
|---|---|
| DARK | 0.06 – 0.14 |
| LOW | 0.18 – 0.28 |
| BRIGHT_ONLY | 0.34 – 0.46 |
| LIT | 0.54 – 0.70 |
| BRIGHT | 0.80 – 1.00 |

Because the bands are disjoint and ordered, the mapping is **strictly
monotonic in the enum order for every seen value** (the brightest
DARK tile is still darker than the dimmest LOW tile) and strictly
monotonic in seen within a level. The gate test asserts both, for
all level pairs and a seen sweep. `MEMORIZED` has no band (its
presentation is §3); `BLANK` and any unknown future enum value map
to 0 / Hidden — the mapping *fails closed*, never revealing a tile
the game has not clearly marked seen.

## 3. Visibility states (gate 7.2) — `mapping.rs`, `grid.rs`

`visibility_of(lit, seen) -> Visibility`:

- `Visible` (known visible level, seen > 0): full presentation.
- `Memorized` (`MEMORIZED`): **desaturated and darker**, with the
  exact factors chosen here and pinned by tests —
  `MEMORIZED_SATURATION_KEEP = 0.30` (the color is mixed 70% toward
  its own Rec. 709 luminance) and `MEMORIZED_BRIGHTNESS = 0.45`.
- `Hidden` (`BLANK`, seen == 0, or unknown level): black; **not
  meshed and not inserted into the RT tree**. `is_meshed()` and
  `include_in_tree()` are the same predicate by design — one
  contract, two renderers — exposed as `VisibilityMask` over any
  `LightSampler` so the mesher path and the treebuild path ask the
  identical question per tile/voxel.

The proto adapter (`proto_adapter.rs`) enforces memory semantics on
the way in: memory blocks are forced to `MEMORIZED` even if a sender
wrote something else, and only lit/seen are read from them, so live
fields/items in a real tile can never leak into its remembered
presentation through this path.

## 4. Smoothing without leaking (gates 7.3 + 7.4) — `smoothing.rs`

These gates constrain each other; the scheme is designed for both:

1. **A face takes its light from the air side.** A visible face lies
   between a solid tile `S` and the tile in front of it,
   `A = S + normal`. The face's centre light is `A`'s game light,
   full stop. A wall between a lit and a dark room therefore has a
   bright face toward the lit room and a dark face toward the dark
   room *from the same block* — neither side's sample ever reads
   across the wall plane, so nothing leaks.
2. **A vertex averages only air-side, in-plane neighbours.** For
   top/bottom faces, the mean of the up-to-4 tiles touching the
   vertex in the horizontal plane; for side faces, the mean of the
   2 tiles touching the vertex along the horizontal in-plane axis at
   the air tile's own level (light is per-tile; averaging vertically
   would mix *floors*). Tiles that are occluding, Hidden, or of the
   other visibility class (Memorized vs Visible) are **skipped** —
   a wall tile is never an air-side light source, unseen light never
   bleeds into a seen face, and the memory boundary stays a
   boundary.
3. **Sampling goes through a whole-grid `LightSampler`** — the same
   cross-chunk pattern the mesher uses via `VoxelSampler` — so a
   vertex on a chunk border averages tiles from both chunks and a
   gradient crossing the border interpolates exactly as one inside
   a chunk does. The gate test measures the border step against the
   maximum interior step of a 24-tile gradient spanning two chunks.

Known bound: at a 1-tile diagonal corner, a vertex can average
diagonally-touching air tiles — interpolation *within* the air side,
not leaking through a wall, bounded to one vertex.

## 5. Baking (task 7) — `bake.rs`

Two bakers, two precision levels, deliberately:

- **`bake_chunk_map`** (per-block): a new `ChunkMap` with light
  multiplied into block colors; Hidden tiles' blocks become air.
  Feeds the existing mesher unchanged. A `Block` has one color, so
  a wall block between two rooms gets its own tile's smoothed
  centre value on both faces — this is the fast/compatible path,
  **not** the leak-proof one.
- **`bake_pass_meshes`** (per-vertex): rewrites vertex colors of
  already-meshed `PassMeshes` through the §4 air-side scheme (a
  sample point half a block behind the face identifies the solid
  tile; side faces sample the air tile in front, top/bottom faces
  sample the tile itself — a floor slab's airspace is its own tile).
  This is the precise path, and the one gate 7.3's mesh-level check
  exercises: on the two-room fixture, the wall's east-face vertices
  bake dark and its west-face vertices bake bright.

`shade_color()` is the single reference composition both bakers use:
Visible = albedo × environment(sample); Memorized = desaturated/
dimmed albedo × environment *tint only* (memory is already dim by
definition; the environment's brightness must not double-dim it);
Hidden = black.

The working fixture is the fake house at night with a lit interior
(`fixtures::fake_house_map` + `fake_house_night_light`: exterior
`DARK`, lamp falloff `BRIGHT → LIT → LOW` inside, walls occluding) —
the same fake house as Phases 2/2c, duplicated locally because this
crate may not depend on the engine crate (treebuild's gate tests
duplicate it the same way).

## 6. Day/night and weather state (gates 7.6 + 7.7) — `atmosphere.rs`

- **Day/night**: `day_night_at(hour) -> DayNight { brightness, tint }`
  interpolates a keyframe table (midnight 0.22 blue, dawn 0.75 warm
  gold, noon 1.00 neutral, dusk 0.38 warm, nightfall by 20.5) with
  smoothstep between keyframes. The gate test pins midnight/dawn/
  noon/dusk values and asserts no step over a full day at 1-minute
  resolution exceeds 0.01 in brightness or tint. For the tracer's
  sun term this curve applies at full strength (`DayNight::apply`).
  For *baking*, `Environment::apply` tempers brightness to
  `0.4 + 0.6 × curve` (tint still applies in full): a tile's
  lit_level is already the game's time-aware truth — a lamp-lit
  interior at midnight is `BRIGHT` in the game's own terms — so the
  raw 0.22 would double-darken local light and crush interiors to
  black. The blend keeps the ordering (midnight dimmest, clear noon
  exactly 1.0) without overriding game truth.
- **Weather**: `WeatherKind { Clear, Cloudy, Rain, HeavyRain, Snow,
  Fog, Thunderstorm }` (the granularity the renderer needs; the fork
  exporter maps its weather enum onto these) →
  `weather_params(kind) -> WeatherParams { fog_density, fog_color,
  precip_rate, precip_fall_speed, brightness_scale, tint }` — a
  constant table lookup, O(1), evaluated once per frame/bake.
  `fog_density == 0` means fog is *off* (the renderer skips the term);
  precipitation is a rate for the Phase 2b-style shader emitters, not
  per-tile work. There is no per-state cost cliff because switching
  state changes constants, not algorithms.

## 7. Light updates (gate 7.5) — `updates.rs`

`apply_patch` / `apply_tile_changes` / `apply_light_update` (wire
`LightUpdateT`, one 12×12 block) / `apply_delta` (wire `DeltaT`
lit/seen) mutate the grid and return the dirty chunk set computed
from tiles whose lit/seen **actually changed** (re-sent values dirty
nothing). The set is: the changed tile's own chunk, plus the edge
neighbour(s) when the tile sits on a chunk border, plus the diagonal
at a corner — because §4's vertex sampling reads at most 1 tile
across a border. A tile one tile inside a border dirties only its
own chunk, and the tests pin that too. Sufficiency is proven, not
asserted: on the 24×24 flashlight scene, re-baking only the dirty
set after a corner-tile change reproduces the full re-bake's meshes
exactly (positions and colors, all four chunks).

## 8. Visual scenes (gate 7.8) — `render.rs`, `output/`

CPU PNG dumps (the Phase 0 precedent), labelled for what they are:
**not** renderer captures. `render_top_down` is an orthographic
plan view (one pixel block per tile, unexposed linear values — night
scenes read dark because the values are dark); `render_isometric`
is a simple dimetric projection of the baked map with fixed face
shading (×1.0 / ×0.72 / ×0.52) on top of the baked game light.
Regenerate with `cargo run -p lighting --bin render_scenes`; the
gate test also writes them and asserts the scene relationships
(lamp centre > falloff edge; cone core > mid > dark field, dark
behind the player; remembered street dimmer + desaturated vs live,
BLANK pure black).

- `crates/lighting/output/house_night_topdown.png` — lit house at
  night, plan view.
- `crates/lighting/output/house_night_iso.png` — same bake,
  isometric, roof level cut away: wall ring, windows, door and the
  warm interior falloff are all visible.
- `crates/lighting/output/flashlight_field.png` — flashlight cone
  in a dark field (game-style falloff authored as lit data).
- `crates/lighting/output/remembered_street.png` — west half
  MEMORIZED (grey, dim), live street east, BLANK black beyond.

PNG encoding is a small in-crate writer (`png.rs`, zlib stored
blocks) — the crate's dependency list is unchanged from the
scaffold (`mesher, proto, glam`); no `wgpu`, no new deps.

## 9. Performance (gate 7.9)

Scene: the full stored view — 132×132 tiles × 3 cutaway levels,
121 chunks (`fixtures::full_view_scene`, exercising visible,
memorized, blank and occluding paths). Measured on this VM
(1.5 shared CPUs / 6 GB, so treat as an upper bound, not target
hardware):

- Per-block bake of the whole view: **~35–100 ms debug, ~45 ms
  release** (single measurements on a noisy shared VM; the cost is
  dominated by allocating the output `ChunkMap`, not by the light
  math, which is O(1) per block).
- That is **~0.3–0.8 ms per chunk** — inside the §11 chunk-remesh
  budget (2 ms/chunk) even on this VM, and a full-view bake is a
  snapshot-time operation anyway: per turn, gate 7.5's dirty set
  limits re-baking to the handful of chunks a light change touches.
- Meshing the baked view (for reference; mesher's cost, not this
  crate's): ~1.5 s debug / ~0.2 s release on this VM for all 121
  chunks, 5,804 greedy quads.

The gate test asserts the bake completes under a generous 2 s
bound whose purpose is catching an accidental quadratic blow-up,
not gating on VM timing; the budget argument above is the real
7.9 evidence, and re-measuring on the RX 9070 XT machine is the
recorded follow-up (same status as 2.13/2c.9-class measurements).

## 10. Gate results

| Gate | Result | Evidence |
|---|---|---|
| 7.1 light mapping | ✅ | `gate_7_1_light_mapping_monotonic` (disjoint bands, seen sweep, tint shape) |
| 7.2 visibility states | ✅ | `gate_7_2_visibility_states` (exact MEMORIZED factors, mask contract, BLANK blocks unmeshed in a baked map) |
| 7.3 no light leaking | ✅ | `gate_7_3_no_light_leaking` (face + vertex + mesh level on the two-room fixture) |
| 7.4 smoothing | ✅ | `gate_7_4_smoothing_no_chunk_seam` (border step ≤ 1.5× interior max, monotonic gradient, border vertex interpolates) |
| 7.5 light updates | ✅ | `gate_7_5_light_updates_dirty_chunks` (exact dirty sets incl. corner/diagonal, partial == full re-bake, no-op patch dirties nothing) + wire adapters in `proto_adapter_tests` |
| 7.6 day/night | ✅ | `gate_7_6_day_night` (pinned times, warmth/blueness, 1-minute smoothness sweep, wrap) |
| 7.7 weather | ✅ | `gate_7_7_weather` (on/off per state, fog transmittance/blend, bounded finite params for all states) |
| 7.8 visual scenes | ✅ | `gate_7_8_visual_scenes` + the four PNGs in `crates/lighting/output/` (visually inspected: iso house and remembered street read clearly; top-downs are unexposed linear dumps) |
| 7.9 performance | ✅ (VM-measured, see §9) | `gate_7_9_performance_full_view` with printed timings |

Supporting: `proto_adapter_tests` (5 tests) — snapshot grid build,
memory-block MEMORIZED enforcement, default occlusion predicate,
wire LightUpdate/Delta application and dirtying.

## 11. Deviations from the plan text

1. **Two bakers instead of "bake into vertex data *or* a light
   texture"** — the plan left the choice open; both a per-block and
   a per-vertex baker are provided because only the per-vertex one
   can satisfy 7.3 for a single-color-per-block world model. No
   light *texture* was built: per-vertex colors ride the existing
   mesh format with no new GPU plumbing.
2. **Day/night brightness is tempered when baking** (§6) — the pure
   curve exists and is gate-tested; the tempering is a documented
   presentation choice so global state never overrides the game's
   per-tile truth.
3. **Weather is a state enum defined here**, mirroring what the
   game will export at renderer granularity — the schema carries no
   weather field yet, so when the fork exporter adds one it maps
   onto `WeatherKind` without a lighting-crate change.
4. **`LightUpdate` size is explicit in this crate's `LightPatch`** —
   the wire table carries no size; Phase 3's exporter shape (one
   12×12 submap block) is the adapter's documented assumption.

## 12. Engine wiring (for the parent — not done here, by ownership)

Raster (mesher) path, in `engine::world::mesh_world` /
`mesh_fake_house`'s real-data successor:

1. On Snapshot: `let grid = lighting::proto_adapter::grid_from_snapshot(&snap);`
   (z: grid z is game z; chunk level `i` maps to grid z
   `min_game_z + i` — pass that as `grid_z_of_level0` to the
   `*_with_offset` baker / `bake_pass_meshes`).
2. Keep the `ChunkMap` **unbaked** as the world of record. Per dirty
   chunk: `bake_chunk_map_with_offset(&map, &grid, env, min_game_z)`
   restricted to that chunk (or bake whole-map at snapshot time),
   then `mesh_chunk`, then
   `bake_pass_meshes(&mut meshes, &grid, env, scale, min_game_z)`
   for the precise per-face light. If only one baker is wired
   initially, wire the per-vertex one and skip the per-block bake
   (meshing the unbaked map + per-vertex bake gives identical
   Hidden/Memorized handling for Visible tiles; Hidden exclusion
   then needs step 3's mask at mesh time).
3. Hidden exclusion on the raster path: before meshing a chunk,
   drop blocks whose tile `VisibilityMask::is_meshed` is false
   (the per-block baker already does this by turning them to air;
   if it is skipped, apply the mask in the cutaway/filter step —
   `lighting::filter_levels` is the shape to extend).
4. On Delta / LightUpdate: `lighting::apply_delta` /
   `apply_light_update` → dirty chunk set → re-bake + re-mesh
   exactly those chunks (gate 7.5 proves this equals a full
   re-bake). Environment (`Environment::new(game_hour, weather)`)
   is re-evaluated per bake from game state, never per tile.

RT (treebuild) path — the visibility-mask contract:

1. When converting a `ChunkMap` to a `VoxelWorld`
   (`VoxelWorld::from_chunk_map` successor): for each block, look
   up its tile in the grid; if
   `VisibilityMask::include_in_tree(x, y, z)` is false, **skip the
   voxel entirely** (a BLANK voxel must never enter the tree —
   gate 2c.8's discipline, now owned by this crate's mask).
2. For Memorized voxels that are included: set the existing
   `FLAG_MEMORIZED` / `lit = LIT_MEMORIZED` on the cell (treebuild
   already carries both) and pre-dim its color with
   `lighting::present_color(albedo, tile)` at neutral environment,
   or pass the presented factor through the side-table — either
   way the desaturation factors are this crate's constants, not
   re-chosen shader-side.
3. For Visible voxels: the tracer's own illumination applies on
   top of the tile's `LightSample` (`tile.raw_sample()`, tempered
   by `Environment` for the sun term via `DayNight::apply` at full
   strength) — the tracer must clamp its result so a tile the game
   marks DARK cannot be sun-lit into visibility: multiply tracer
   output by the tile sample's intensity as the visibility ceiling.
