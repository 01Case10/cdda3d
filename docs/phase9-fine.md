# Phase 9 — fine-grid world: slice 3 notes

Companion to plan.md §11b. Slice 3 is built in micro-slices
(3a, 3b, …); each lands gated and committed on its own. This
document collects the slice-3 design notes and the named
limits that the plan's gate requires to be written down, not
silently accepted.

## 1. Lighting for fine props — the v1 rule and its limit (9.6)

**The v1 rule (in force since slice 2, raster path).** A fine
cell takes the baked light of the coarse block that contains
it. Concretely, on the raster path the per-vertex baker
(`lighting::bake_pass_meshes`) is position-based, so a fine
quad is baked with the light of the game tile containing its
position — the same `lit_level`-derived light its containing
coarse block would have received. With props present, a
`PerBlock` bake request is routed through the per-vertex
baker for that frame (the per-block baker runs on the coarse
chunk map and cannot carry fine cells; the frame is never
double-baked). On the lit path the visibility mask is applied
*before* stamping, and a placement whose tile lost its floor
slab to the mask is skipped — a prop never floats over a
`Hidden` (BLANK / never-seen) tile. The RT path follows the
same rule when 9.5 lands: a fine cell's side-table `lit` is
its containing tile's `lit` (see the fine adapter, slice 3e).

**The known limit.** Props do not occlude the coarse bake.
The bake is computed from the coarse world and the game's
per-tile light grid; fine geometry is not an occluder in it.
Two visible consequences:

- Light can leak *through* a large prop: the floor and wall
  behind a fridge are lit as if the fridge were not there,
  because at coarse resolution that tile's furniture block is
  suppressed (plan 9.0) and nothing else marks it occupied.
- A prop's own faces all take their containing tile's light:
  the back of a fridge against a wall is no darker than its
  front, beyond what face shading (normal · sun) provides.

**Why v1 accepts this.** The game's lighting truth is
per-tile (`lit_level` per tile, Phase 7): there is no
sub-tile light information to recover, so the error is
bounded by one tile's light value — the same granularity at
which the coarse world itself is lit. Inventing sub-tile
occlusion would be a visual effect, not a correction of
game truth.

**Stage H candidate (named, not silent).** A fine-aware
bake: rasterize fine volumes into the baker's occlusion
sampling so a full-footprint prop shadows its own tile's
floor slab and its back faces, and (RT) let fine-tree hits
participate in shadow/AO rays with their containing tile's
`lit` as the cap. This is Stage H tuning work; it changes
no protocol, no game truth, and no v1 gate. Until it lands,
captures that show a prop are understood to carry this
limit — in particular the slice-2 lab capture
(`captures/phase9_lab.png`) and any RT lab capture from 9.5.

## 2. The two-tree RT design (9.5)

The RT renderer keeps the coarse tree exactly as before and
adds a second, independent tree built only from stamped fine
cells (`treebuild::world::VoxelWorld::from_fine_layer`, fine
coordinates: 1 unit = 1 fine cell = 1/32 block). Both trees
are traced per ray and composited by hit distance
(`cpu::cast_ray_two` on the CPU, `trace_two` in WGSL — the
ray is scaled into fine space, the fine hit's distance is
divided back by 32, the nearer hit wins, the fine hit wins
an exact tie). Nothing stamped → no fine tree
(`fine_present = 0`) → the render is the coarse-only render,
unchanged; this is the trace-level form of 9.10's
byte-identical contract, and it is gated at CPU, GPU-query
and full-capture level.

Per-face LOD for fine leaves adjusts the ray-cone Δ by
−log2(32) = **−5**, applied *before* the clamp
(`lod::ray_cone_lod_fine` / WGSL `cone_lod_adj` with
`fu.fine_lod_adjust`): a fine face maps its texture over
1/32 of a block per axis, so its texel density per unit
distance is 32× the coarse face's. The prop colours are
synthetic solids — every mip of a solid layer is the same
colour — so for props themselves the adjustment is
unobservable; it exists so a fine cell carrying a *coarse*
material's texture (the adapter's general case) samples at
the right level.

Shading notes: a fine hit shades from the fine tree's own
node/side entries; shadow rays use the two-tree trace (props
occlude the sun); the occupancy-AO probe stays coarse-only
in v1 (it samples block-scale neighbours; a prop's own cells
do not AO each other — same family as the 9.6 limit, and
bounded the same way). Lit: a fine cell's `lit` is rewritten
per containing tile after the adapter build, the 9.6 rule.

## 3. Perf + memory numbers (9.9)

Measured by `crates/engine/tests/phase9_numbers_gate.rs`
(main-tree re-run, 2026-11-17; this VM: 1.5 shared CPUs,
lavapipe — build times varied ~±40% between runs under load,
so ranges are given where two runs exist). Sample scene,
cutaway 0, 640×360 frames.

World + trees:

| scene | cells | tree nodes / leaves | build (median of 3) |
|---|---|---|---|
| coarse world (pre-Phase-9) | 686 | 727 / 686 | ~2.3 ms |
| fixture props, fine tree (5 placements, 5 tiles) | 19,576 | 19,998 / 19,576 | ~130–200 ms |
| stress, fine tree (200 placements, 100 tiles) | 420,528 | 429,363 / 420,528 | ~3.1–5.0 s |

Fine-layer memory (`FineLayer::memory_bytes`): fixture
668,856 B (~131 KB per prop tile); stress 15,014,976 B
(~147 KB per prop tile). The fine tree is ~1 leaf per solid
cell (the 64-tree does not merge same-material neighbours
into larger leaves at build time), so node count tracks
cell count almost 1:1 — a prop-dense scene is ~21k fine
cells per placement. On the sample scene the fixture tiles
carry no coarse furniture blocks, so stamping suppresses
nothing there (suppression is gate-proven in
`phase9_stamp_gate` where a furniture block exists).

Frame times (lavapipe, median of 3 after warmup):

| renderer | coarse-only | fixture props | stress (200 props) |
|---|---|---|---|
| raster | 105 ms | 401 ms | 3,406 ms |
| RT | 328 ms | 504 ms | 8,747 ms |

**Reading these numbers.** Lavapipe is a CPU renderer on a
throttled shared VM; none of these are predictions for real
hardware. They establish the *shape*: the fine tree build
dominates world-build cost (~60–90× the coarse build for
the fixture scene), frame cost grows with stamped-cell
count in both renderers, and RT pays more per fine cell
than raster. The RX 9070 XT frame-time verdict (gate 2c.9)
stays the standing deferred measurement — raster remains
the default renderer until it exists, and `--fine-props`
stays default-OFF pending Joey's sign-off on the lab
capture (9.10).

**Stage H landing (2026-11-23, Joey's 8c-A).** The candidate
above is now built, as a geometric pass over the stamped
geometry (still invented lighting — no game truth changes):
`mesher::fine::face_occlusion` darkens a fine face by the
opaque cells around the air cell in front of it (4 in-plane
neighbours + the cell straight ahead; 0.11 per occluder,
floor 0.56; a face in the open is exactly 1.0, so unoccluded
scenes mesh byte-identically). The raster fine mesher applies
it per face; the RT path (`engine::rt::fine_occlusion_rewrite`,
applied after the lit rewrite in the capture path) averages
it per cell (`cell_occlusion`) and steps the categorical
`lit` down 1 level below 0.85 mean, 2 below 0.70, never
touching Memorized/Blank cells. What it fixes from the list
above: a prop's faces tucked against walls, under overhangs,
or inside prop clusters are now darker than its open faces,
and cells enclosed by other prop geometry darken too. What
it still does NOT do: coarse floor/wall cells behind a prop
keep their tile light (the leak-through direction) — the
baker's occlusion sampling remains coarse, and that stays a
named limit. Gates: mesher `fine_occlusion_gate`, engine
`fine_occlusion_rt_gate`. The tuning constants (0.11, 0.56,
0.85/0.70) are first-pass values chosen by the assistant;
Joey accepted the tuning risk when he picked this option.
