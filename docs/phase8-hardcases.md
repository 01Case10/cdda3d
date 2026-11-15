# Phase 8 — Hard cases: design and gate record (start, 2026-11-15)

Phase 8 is the release phase: vehicles, door/window state, open
air and roofs, animations/projectiles, the overmap, mod smoke,
soak/performance, packaging and the final playthrough (plan §5
Phase 8, gates in Part II §11). This document records the first
slice — the hard cases that are provable at model level without
the live frame loop — and scopes the rest honestly.

**Isolation note.** This slice ran in parallel with Phase 2h
(engine frame loop, `crates/engine`) and Phase 2g.4 (tile
coverage, `coverage-audit/` + tileset outputs). It therefore
touches **none** of: `crates/engine`, `crates/billboard`,
`crates/ui-share`, `crates/tileset`, `crates/voxelizer`,
`coverage-audit/`. Everything below lives in three new crates,
one additive schema field, and the fork exporter. `Cargo.lock`
is deliberately left uncommitted by this slice: the lockfile in
the working tree also carries Phase 2h's in-flight engine
dependency edges, and the lock regenerates on build (CI does not
run `--locked`).

## What was built

### `crates/vehicles` — gates 8.1, 8.2, 8.3 (model level)

The rule the whole crate hangs on: **the engine never recomputes
part positions from facing + mounts.** Phase 3 exports absolute
tile positions computed game-side from the pre-rotated mounts
(`bub_part_pos` / `precalc_mounts`) and a continuous float facing
(`face.dir()`); the model stores both verbatim.

- `model` — wire → model decoding. Part flags are the fork's bits
  (voxel_real_adapter.cpp): bit 0 open (`part.open`), bit 1 broken
  (`is_broken()`), bit 2 roof (`vpart_info` ROOF flag). Roof parts
  split into a roof layer, mirroring the game's own renderer
  split. A part reads *damaged* when broken, or at ≤ ½ hp.
- `store` — Snapshot replaces wholesale (prev := current: nothing
  to interpolate from after a resync); Delta applies full
  replacement vehicle states plus `vehicle_removed`. Part damage
  and removal need no special case — a broken part arrives with
  its flag set, a removed part stops appearing.
- `interp` — presentation sampling between the last two exports.
  Facing follows the shortest arc (359° → 7° is +8°). Parts match
  by (vpart, mount) identity; positions lerp; flags always come
  from the current export. Endpoints are exact by construction —
  that *is* gate 8.2's no-drift property.
- `fixtures` — mock game truth (pivot + rotated mounts → rounded
  absolute positions, the shape of `precalc_mounts`). The gate
  compares the model against truth at 0°, 7°, 15°, 23°, 45°, 90°,
  179°, 270°, 359°; a counter-proof test shows quantizing 7° to
  the nearest 15° step moves at least one part to a different
  tile — the concrete reason facing is never quantized (the 15°
  in old notes is `vehicle::immediate_path()`'s *pathing* clamp).

### `crates/transient` — gate 8.5 (model level)

`Effects` messages (the game's `src/animation.*` dispatch,
forwarded by the protocol) become **transient emitters** in the
Phase 2b shader-effect parameter space — never sprite-frame
animation (Joey's 2b direction, which Phase 8 inherits).

- `params` — effect id + kind → (render class, color, emissive,
  size). Render class ids mirror `fx::RenderClass::id()`; like fx
  mirroring the voxelizer's JSON contract instead of depending on
  it, this crate mirrors six constants instead of taking a wgpu
  dependency. Id families win over kind defaults (an
  `explosion` id is a flame burst whatever the kind says); sound
  markers get no visual emitter; unknown ids get a visible
  generic flash rather than a silent drop.
- `system` — ingest is idempotent per (seq, index) and stale-seq
  messages drop whole (replaying them would put effects on the
  wrong tiles at the wrong times). The clock is explicit and
  monotonic. Projectiles travel from → to over the duration;
  bursts/flashes sit at their tile. An emitter occupies
  [start, start+duration) — at the end instant it is gone.
  "No leftover effects" is the system's core invariant, gated
  directly.

**Protocol truth, plainly:** an `EffectEvent` carries endpoints
and a duration — no arc height, no curve. A thrown item's
parabola is not in the data. Trajectories here are linear in
tile space; any visual arc is a presentation choice layered on
top, separate from the exported truth.

### `crates/hardcases` — gates 8.4, 8.4b (model level)

- `doors` — state is an id change, never a flag. `DefTable`
  resolves the TileDefs transform strings (`open_result` etc.)
  back to def ids; `classify_transition` names a transition
  Open / Close / Bash / Unchanged / Other; `TileWorld` applies a
  Delta `TileChange` wholesale. The gate runs the protocol's own
  sample flow: the sample house's closed door at (5,11) plus the
  sample Delta → classified Open, world holds `t_door_o`, and its
  material reference moves (fg_ref 102 → 104).
- `openair` — `t_open_air` emits no block. The floor at an
  open-air level is the **`roof` of the terrain below**
  (`ter_t::roof`: "what will be the floor above this terrain"),
  single-step: below declares no roof → no floor (a hole, not a
  fabricated slab); below is itself open air → no floor at this
  level (a shaft). Resolution is a pure function of the column,
  so cutaway choices cannot change what a level's floor is —
  gated by resolving truncated columns and comparing.

### Schema: `TileDef.roof_result` (additive)

Gate 8.4b was **not implementable from the Phase 3 export**: no
field carried `ter_t::roof`. Phase 8 therefore appends
`roof_result: string` as TileDef's **last** field (vtable id 34,
after bg_ref's 32 — every existing id unchanged; old readers
ignore it, the displayed_field precedent of 2026-11-15). Filled
by the fork for terrain defs only
(`voxel_real_adapter.cpp`, mirroring the `open` fill);
`DefVal::roof_result` + the exporter's copy line complete the
path. Both checked-in generated C++ headers were regenerated
with flatc 24.3.25 (`--cpp --gen-object-api`); the regen diff is
exactly the new field. Proto samples keep `roof_result: None`
(absent on the wire), so the committed Phase 1 goldens stay
byte-identical — verified by the proto suite and a rebuilt
`cpp-roundtrip` (PASS). A present value's wire round-trip is
gated in `hardcases` (framing encode → decode → resolve).

## Gate results (parent-run, this machine)

| Gate | Result |
|---|---|
| 8.1 orientations | ✅ model level — exact positions + bit-exact facing at all sampled facings; quantization counter-proof ✅. In-game half ⛔ 0.1/0.2 (fork harness 3.8 already pins the export side at 0/37.5/179.9/359.99°) |
| 8.2 movement | ✅ model level — scripted turning drive, endpoints exact at every step, mid-states on-segment, wrap interpolation ✅ |
| 8.3 damage/removal | ✅ model level — broken/open/hp states, part removal, vehicle removal, roof split ✅ |
| 8.4 doors/windows | ✅ model level — sample-flow Open + close/bash/other classification, furniture parity ✅ |
| 8.4b open air/roofs | ✅ model level — two-storey + basement fixture, no phantom/fabricated floors, cutaway independence, wire round-trip ✅. In-game half ⛔ 0.1/0.2 |
| 8.5 animations/projectiles | ✅ model level — right tiles/times, expiry with no leftovers, resend/stale handling ✅. In-game half ⛔ 0.1/0.2 |

Suites: `vehicles` 6/6, `transient` 4/4, `hardcases` 5/5,
`proto` 16/16 (goldens intact), fork harness ALL PASS +
RUST-DECODE PASS, C++ round-trip PASS; regression sample
(mesher, lighting, fx, treebuild, tileset, voxelizer) all green.

## What remains in Phase 8 (scoped, not started here)

- **8.6 overmap** — by design the overmap stays game-drawn UI in
  the shared texture; the gate is a Phase 2h frame-loop check
  (UI quad shows it, input round-trips, returning restores 3D).
- **8.7 mod smoke** — needs a runnable fork + popular mods;
  blocked on 0.1/0.2 like every in-game gate. The missing-material
  report it thresholds against exists (Phase 5 resolution chain +
  2g.4 coverage audit).
- **8.8 / 8.9 soak + memory** — need the live loop (2h) and a
  runnable game; 2 h scripted play cannot be faked at model
  level.
- **8.10 performance regression** — the budgets live in plan
  §11; several are target-hardware numbers (RX 9070 XT), and the
  RT default decision (2c.9) is still open.
- **8.11 / 8.12 install + upgrade** — packaging: Windows/Linux
  builds, bundled fork binary, the `compose.py` asset step (or a
  pre-composed tileset from a release). Linux-first per §14;
  Windows packaging is Phase 8 scope proper and needs Joey's PC
  for the clean-VM checks.
- **8.13 crash reporting** — logs with version info incl. the
  pinned fork commit; engine half can start with 2h, game half
  needs the fork running.
- **8.14 final playthrough** — manual, both platforms, last.
- **Engine wiring of this slice** — drawing `vehicles` models and
  `transient` emitters, and applying `hardcases` floor resolution
  in the mesher/RT tree build, are Phase 2h follow-ups (the frame
  loop owns the render surface; this slice deliberately stayed
  out of it). The multitile `open`/`broken` *subtile* variants
  (Phase 2d follow-up) ride the same wiring: state arrives as an
  id change (8.4), and the new id's def selects its own subtile
  set — no extra engine state.

**Release gate** (Part II §11) still requires all earlier gates
green plus 8.1, 8.4b, 8.7, 8.8, 8.10, 8.11 passing in full —
i.e. the in-game halves on a build-capable machine. Phase 8 is
therefore **in progress**, not passed.
