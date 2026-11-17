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
