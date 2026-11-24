# Phase 8 — Hard cases: design and gate record (2026-11-15)

Phase 8 is the release phase: vehicles, door/window state, open
air and roofs, animations/projectiles, the overmap, mod smoke,
soak/performance, packaging and the final playthrough (plan §5
Phase 8, gates in Part II §11). **Status: GATE PASSED
(conditional)** — every gate in §11 is now proven, on dummy data
where the game itself would be the source, with the conditions
listed at the end of this document (the same standing conditions
every earlier phase carries). The phase ran in two slices on
2026-11-15: the hard-case models first (8.1–8.5), then, on
Joey's direction that dummy data should carry the phase to
completion, the release gates (8.6–8.14) plus the vehicle render
model.

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
| 8.6 overmap | ✅ data level — UiInput overmap frame validates, input routes as UI, MapInput restores world picks (release gate suite) |
| 8.7 mods | ✅ data level — mod fixture through all stores + exact missing-material report; real mods ⛔ 0.1/0.2 |
| 8.8 soak | ✅ harness level — 7,200 scripted steps, bounded state, no cost creep |
| 8.9 memory | ✅ harness level — RSS +128 KiB after warm-up over ~1.8 h of soak |
| 8.10 performance | ✅ Phase 8 surfaces (µs-scale vs ms budgets); frame times ⛔ target hardware |
| 8.11 install | ✅ Linux half — package smoke PASS from a clean dir/env; Windows half ⛔ no Windows here |
| 8.12 upgrade | ✅ — stamp-invalidated cache + wholesale state resync |
| 8.13 crash reporting | ✅ — versioned report written on a real panic |
| 8.14 playthrough | 🟡 scripted analogue ✅ (soak + package session); manual 1 h × 2 platforms outstanding |

Suites: `vehicles` 6/6, `transient` 4/4, `hardcases` 5/5,
`proto` 16/16 (goldens intact), fork harness ALL PASS +
RUST-DECODE PASS, C++ round-trip PASS; regression sample
(mesher, lighting, fx, treebuild, tileset, voxelizer) all green.

## Second slice: the release gates on dummy data

### Vehicle render model (`vehicles::mesh`)

The model now stamps into the mesher's world: body parts fill
the lower blocks of their tile's z-level, roof parts the top
block (a roof part sharing a tile with a body part completes the
column; alone it is a top slab — the mesher's own slab language).
Gate: at 23° the stamped tile set equals the mock-truth set
exactly, the chunk meshes to real quads, and clearing restores
air — no ghost vehicle. Broken parts darken toward black-red.

### `crates/release` — the release machinery

- **8.6 overmap** — proven at the data level the design
  specifies: the overmap *is* a game-drawn UI frame. A synthetic
  UiInput frame (dark field + marker pixel) is written to the
  Phase 4 shared store, validated against it (mode UiInput, no
  terrain rect — the overmap covers the UI), clicks route as UI
  input, drags are swallowed, nothing is a world pick; the next
  MapInput frame restores world picking inside its terrain rect
  and camera drags. The pixel-level composite of that frame over
  the 3D view is Phase 4's gate (4.2/4.6), and the in-loop
  presentation is 2h's.
- **8.7 mod smoke** — a mod TileDefs fixture (new connect group
  `mod_glowall`, floor/roof pair, full door transform chain,
  vehicle part, monster, shader field) flows through the base
  rules unchanged: group resolves, door classifies
  Open/Close/Bash, the mod floor roofs via `roof_result`. The
  missing-material report (`hardcases::report`) names exactly
  the defs with a sprite reference but no material — the fixture
  report is precisely the two unmaterialed defs, and the shader
  field/open air are listed as *unreferenced*, never missing.
  Cross-crate: unknown vpart ids are opaque to the vehicle
  store; unknown effect ids get a visible generic emitter.
  Loading the *popular mods themselves* needs the runnable fork
  (condition below); the data path they will take is gated.
- **8.8/8.9 soak + memory** — `release::soak` plays a
  deterministic 7,200-step session (2 h at 1 turn/s) through the
  Phase 8 stores: a turning drive (incl. the 359°→0° wrap),
  traffic spawning/despawning, effects every 7 turns, a door
  toggling 655 times, periodic snapshot resyncs. Result on this
  VM: completes; transient state bounded at 2 alive and drains
  to 0; vehicle store ends with exactly the live set; per-step
  cost 6.0 µs (first 600) → 17.4 µs (last 600, includes a
  resync block) — no creep; **RSS 3,340 → 3,520 (warm) → 3,648
  KiB, +128 KiB after warm-up** — no leak signature. The frame
  loop's own soak rides 2h.
- **8.10 performance** — Phase 8 surface costs, release build,
  this VM (`perf_probe`): vehicle delta apply, 20 vehicles:
  **11.37 µs**; transient ingest (2 events): **3.21 µs**; door
  classification: **0.05 µs**; open-air floor resolution:
  **0.02 µs**. Against the plan's turn budgets (snapshot < 20 ms,
  delta < 2 ms — fork harness 3.18) the engine-side Phase 8 work
  is three orders of magnitude inside. Frame-time budgets are
  target-hardware numbers and stay with 2c.9.
- **8.11 install (Linux half)** — `tools/package_smoke.sh`
  assembles a package dir (game binary + frontend binary +
  VERSION stamp), then from an unrelated directory with a
  scrubbed environment (`env -i`): **PACKAGE SMOKE PASS** —
  handshake, TileDefs, snapshot (gameplay), input round-trip,
  clean shutdown. The game binary in the package is the stub
  server, the project's standing stand-in for the fork.
- **8.12 upgrade** — the asset cache is keyed by a package stamp
  (frontend version + fork commit + protocol version): fresh
  install generates, same build reuses, an upgrade's stamp
  mismatch marks the cache stale with the reason, regeneration
  re-stamps. State: a store built under epoch 1 resyncs
  wholesale under epoch 2 with no residue (and byte-level
  old-save decoding is gate 1.9's).
- **8.13 crash reporting** — `release::report`: a panic hook
  writes `crash-<unix>.log` carrying frontend version, protocol
  version, the pinned fork commit, time, panic message and
  location, and prints the user-facing line naming the file.
  Gated by catching a real panic and reading the report back.
- **8.14 playthrough** — the scripted analogue is the soak plus
  the package smoke's full session; the manual checklist itself
  (1 h by hand, both platforms) is a condition, below.

**Engine wiring** of the Phase 8 crates (drawing vehicle models
and transient emitters in the live loop, `floor_at` in the tree
build, the 2d `open`/`broken` subtile follow-up) remains Phase
2h's, on its frame surface — this phase deliberately stayed out
of it, as directed.

**Update (2026-11-15, later the same day):** that wiring is
done. Vehicle models and transient emitters are scene inputs to
`engine::frame::render_frame` (world pass and fields pass
respectively), open-air columns resolve through `floor_at` in
the shared `engine::snapshot_world` builder feeding both
renderers, and the 2d `open`/`broken` follow-up is closed
(canonical `state_subtile` selection — open wins, per the
pinned `cata_tiles.cpp` draw paths — consumed by the vehicles
crate and the texture spec builder's state variants). Gates:
`crates/engine/tests/phase8_wire_gate.rs`. The blocked list
below is unchanged.

## What is actually blocked (the complete list)

Everything above is proven on dummy data. The residue is not
data-shaped and cannot be simulated:

1. **Gates 0.1/0.2** — compiling the real fork (~1,500 TUs) and
   running `cata_test`. This VM has 1.5 shared CPUs / 6 GB; it is
   a compute-capacity limit, recorded since Phase 0. Every
   in-game half of every gate (3.x, 4.13/4.14, 8.1–8.5 in-game,
   loading real mods for 8.7) inherits exactly this condition —
   the project's standing conditional-pass condition.
2. **8.11 Windows half** — no Windows OS in this environment
   (the same condition gate 1.15 carries; Linux-first per §14).
3. **Target-hardware frame times** — 8.10's frame budgets and
   the RT-default decision (2c.9) need the RX 9070 XT;
   llvmpipe numbers are recorded as proxies, never as the gate.
4. **8.14's manual session** — one hour of human play per
   platform; the scripted soak is its dummy-data analogue, not
   a substitute claim.

**Release gate** (Part II §11): all earlier gates green
(conditional, as recorded per phase) + 8.1, 8.4b, 8.7, 8.8,
8.10, 8.11 passing — met at the level above, with conditions
1–4. Phase 8 is **GATE PASSED (conditional)**.
