# CDDA Voxel — Test Game Engine (Sim Server) Plan
Date: 2026-11-23 · Status: S0 + S1a DONE 2026-11-23 (see plan.md Phase 11 record); S1 next
Repo home: `crates/sim/` (library) + `tools/sim-server/` (binary)
in `~/workspace/cdda-voxel/`; this doc also lives in the repo as `docs/test-sim-engine.md`.

**Decisions made 2026-11-23 (Joey):** (1) **Rust** — yes. (2) Name
**sim-server** — yes. (4) UI text = **real font rendering** — yes
(see revised §3.3). (5) **Phase 11** in `plan.md` — yes, added.
(3) Def universe — **C, full real import, everything** (Joey's
pick 2026-11-23, §8.3). S1a built the full importer (Python stdlib
tool `tools/sim-defs`, output language-neutral JSON — done 2026-11-23).

## 0. The problem, in one paragraph

Every engine gate today runs on *static* fake data. There is no stateful
stand-in for the game that holds a mutable world, reacts to input, and
emits the next Snapshot/Delta the way the real fork would. That is why
live behaviour — going outside then back inside, opening the UI,
spawning/destroying things, a door changing the light — can only be
tested either (a) in pieces, against fixtures that never change, or
(b) in the real fork, which still cannot be compiled here (0.1/0.2
blocked on a build-capable machine since 2026-11-15). A small,
deterministic, protocol-faithful *test game* closes that gap: the
engine can be driven end-to-end, headless and in CI, before the fork
ever runs.

## 1. What exists today — three partial doubles, none of them a game

| Double | Where | What it does | What it cannot do |
|---|---|---|---|
| Stub server | `tools/stub-server/` (C++, `house.h`, `stub_server.cpp`) | Handshake, TileDefs + **one** fake-house Snapshot, echoes `Input` verbatim, Ping→Pong, Shutdown. `--env-script`: each Input cycles noon-clear → storm → midnight `EnvironmentUpdate` | No state. Input changes nothing in the world. No Delta, no LightUpdate, no Effects, no UI shared memory, no movement |
| Fork harness mock | `tools/fork-harness/mock_world.{h,cpp}` | 5 static worlds (shelter, house, road+field, multi-storey+basement, vehicle-on-road) behind the fork's real `IGameAdapter` (`fork/src/voxel_ipc/voxel_adapter.h`); proves the *exporter's* diff/serialiser (gate 3.9: mirror == fresh snapshot, 1,100 turns × 10 seeds) | World does not evolve in response to input — tests mutate it directly in C++. Tests the C++ exporter, not the Rust engine's live loop |
| Rust samples + fixtures | `crates/proto/src/samples.rs`, `crates/engine/src/world.rs` (`build_fake_house`), `frame.rs` (`PropScene/BillboardScene/VehicleScene::fixture*`, `SkyScene::named`, `LightingScene::fake_house`), `crates/ui-share/src/synthetic.rs` | Canonical house, roofed-house (explicit + implicit roof), terrain snapshot; synthetic UI frames for main menu / character creation / inventory / crafting / gameplay sidebar | All frozen. UI frames are pictures, not a UI state machine; scenes are selected by CLI flag, not played |

**Conclusion:** do not extend the C++ stub into a game (it must stay
the minimal cross-language golden source, and `house.h` already has to
be kept in sync with `samples.rs` by hand). Do not extend the fork
harness either (its job is exporter evidence). Build a **new Rust
simulator** that reuses `crates/proto` — the same object API, framing,
verifier, `SeqTracker` and shm header the engine already tests against.

## 2. What "what CDDA would give us" actually is — source-derived contract

Verified against pinned CDDA `master @ 6a3e59e0` in
`fork-src/cdda/`, `docs/hook-points.md`, `docs/recon.md`,
[Base note, 2026-11-23: the fork base moved to CDDA **0.I-1**
(release tag `0.I-1`, `7b2efa5c`) per plan.md Addendum 12; the
contract facts below were re-verified against the 0.I-1 tree in the
re-verification pass and hold unchanged. The generated def table
was regenerated from 0.I-1 data the same day: **13,532 defs**
(Terrain 1,108, Furniture 659, Trap 124, Field 144, Monster 1,123,
Item 9,912, VehiclePart 462), 36 connect groups, validate PASS
with 604 dangling refs investigated + allowlisted fresh for this
base; sim suite 12/12 and the engine world/mesh parity gate 3/3
re-run green on the new table. The 14,262-def census cited in the
Phase 11 record is the master-pin figure.]
`fork/src/voxel_ipc/voxel_real_adapter.cpp`, and
`schema/cdda_ipc.fbs`. The sim must mimic *this*, not invent a
friendlier protocol:

### 2.1 Geometry / world shape
- Submap = 12×12 (`SEEX/SEEY`, `src/map_scale_constants.h`); reality
  bubble = 11×11 submaps = **132×132 tiles** (`MAPSIZE`); max view
  distance **60**; z-levels **−10…+10**; 3D FOV z-range 10.
- Snapshot window (fork exporter, `voxel_exporter.h` ViewSpec):
  centre = player tile, z = player z −1 / +1 by default. One
  `SubmapBlock` per (submap, z), 144-entry row-major arrays.
- World scale already decided: 1 tile = 1 block, 1 z-level = 3 blocks.

### 2.2 Per-tile state (`map.h` / `submap.h` → `SubmapBlock` / `TileChange`)
- Exactly 1 each of: terrain (`map::ter`, map.h:1045), furniture
  (`map::furn`, by value, map.h:1020), trap (`map::tr_at`, map.h:1614).
- **Fields are a sparse list, not an id:** a tile can hold smoke + fire
  at once, each with intensity (`map::field_at`, full field map,
  map.h:1660). Plus `displayed_field` — the one field the game itself
  would draw, with its intensity. A Delta's `TileChange` replaces the
  tile's whole field list *and* carries the new `displayed_field`.
- **Items are lossy by design on the wire:** `top_item` =
  `maptile::get_uppermost_item()` (submap.h:439) + `item_count` =
  stack size from `map::i_at` (map.h:1424). Nothing else about items
  crosses.
- `has_graffiti`, `lit`, `seen`, `outside[]` (see 2.3/2.5).

### 2.3 Light, visibility, memory
- `lit` is the game's **`lit_level` enum, verbatim** (`src/lightmap.h`):
  `DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT, MEMORIZED, BLANK` — not a
  0–255 brightness. Read via `map::apparent_light_at(p, cache)`
  (map.h:585) after `map::update_visibility_cache(z)` (map.h:2520).
- `seen` = quantised 0–255 visibility. No public seen-cache getter
  exists in pinned source — the real adapter quantises from
  `lit_level` (BLANK=0, MEMORIZED=64, visible=255); the sim should
  follow the same convention so engine thresholds mean the same thing.
- **`light_frac` / `light_tint` (Addendum 11 light amendment,
  2026-11-23, Joey's decision — full design in
  `docs/addendum-11-light.md`):** CDDA computes continuous light
  (float `lm` + `light_color_cache`) and collapses it into
  `lit_level` at `apparent_light_at()`; the amendment adds two
  additive wire fields alongside `lit`/`seen` in `SubmapBlock` /
  `TileChange` / `LightUpdate` (absent = today's behaviour):
  `light_frac` (`ubyte`) = upstream smooth lighting's
  `light_above_threshold(apparent_light, vision_threshold)`
  (meaningful only for `LOW`/`LIT`; `BRIGHT`=255, else 0), and
  `light_tint` (`[ubyte;3]`) = upstream's normalised
  `illumination_hue(light_color_cache, lm.max())` (brightest
  channel 1.0; absent = white; raw colour *energy* is never
  sent). The sim is the fields' first producer (S3): it must
  compute frac/tint by these same definitions from its own
  apparent-light model and light-source colours, so sim gates
  and fork gates mean the same thing. Visibility is unchanged —
  `lit_level` still decides what may be shown; frac/tint only
  order and colour light inside visible tiles.
- **Memory blocks are a separate layer:** avatar map memory
  (`avatar::get_memorized_tile`) carries terrain/furniture only,
  `lit = MEMORIZED`, never live entities/items/fields.
- Phase 7 ceiling rule the engine applies on top: `lit_level` caps
  how much sky/ambient light a tile can take.

### 2.4 Entities and vehicles — not stored in submaps
- Entities come from creature tracking (`creature_tracker`,
  `game::all_npcs()`, avatar first, id 1, `sprite_id="player"` in the
  real adapter), **not** from tiles. Fields per entity: type def,
  sprite id, absolute pos, `facing` (**NONE/LEFT/RIGHT only** —
  `creature.h` FacingDirection, horizontal flip, no 8-way), overlays
  in `get_overlay_ids()` order (character.h:2553 / monster.h:630 —
  never re-sorted), `hp_frac` 0–255, flags.
- Vehicles: parts carry **game-computed absolute positions**
  (`vehicle::bub_part_pos(map, idx)`, vehicle.h:1555 — mounts are
  pre-rotated game-side) and `face_deg` is a **continuous float**
  (`vehicle::face.dir()`), never quantised. Part flags: open/broken/roof.

### 2.5 Time, weather, sky exposure (Phase 10)
- `EnvironmentUpdate` is the game's resolved state, not raw inputs:
  `calendar` turn, time-of-day / sunrise / sunset (sec), season,
  `sun_azimuth_altitude(turn)`, `incident_sunlight / incident_moonlight
  (wtype, turn)`, moon phase, plus weather as **string id + resolved
  numerics** (`light_multiplier/modifier`, `sun_multiplier`,
  `precip_class`, `rains`, `sight_penalty`, tint, temperature, wind,
  `lightning_active`) — weather types are JSON/mod-defined, so no
  closed enum.
- Cadence ratified in Phase 10: environment sent on change, time-only
  updates every **10 game-minutes**. Snapshot always piggybacks the
  current environment.
- `outside[]` / `TileChange.outside` = the game's own test,
  resolved game-side: `map::is_outside(p) && !map::is_roofed(p)`;
  z < 0 is never outside.
- **S3 implementation notes:** the sim's calendar is a port of the
  0.I-1 formulas (`sun_azimuth_altitude`, `sun_light_at`,
  moon phase/light, incident sunlight/moonlight,
  `natural_light_level`, `get_vision_threshold`) at the game's
  fixed location (42.36, −71.06) with the default 91-day seasons;
  1 turn = 1 s (see the §3.2 correction). Sunrise/sunset are
  solved numerically from the same −1° altitude equation the
  game's iterative solver answers. Weather changes mid-run via
  scene `[[weather_script]]` entries (absolute calendar turn →
  preset), `World::advance_clock` (fast-forward emitting the
  cadence carriers), `World::set_weather`, or the sim-server
  `--weather` startup override. A turn's flush is a message list
  (Delta/Snapshot + optional LightUpdate(s) + optional
  EnvironmentUpdate); a purely-light diff folds to one
  LightUpdate per affected submap instead of a Delta.

### 2.6 TileDefs and transforms
- Sent once at connect: string id, kind (Terrain/Furniture/Trap/
  Field/Monster/Item/VehiclePart), authored flag strings + resolved
  `hot_flags` bitset, connect-group *indices* into a registry table
  (never per-def group name strings), `fg_ref/bg_ref` composed-tile
  indices (−1 = none), and the transform string ids the engine must
  honour: `open_result` (`ter_t::open`), `close_result`,
  `bash_result` (bash → `ter_set`/`furn_set`), `looks_like`, and
  `roof_result` (`ter_t::roof` — the floor above an open-air tile).
- Real values already in fixtures, e.g. `t_floor → t_wood_treated_roof`,
  `t_wall/t_window → t_flat_roof`, `t_door_c → t_wood_roof`,
  `t_door_c.open_result = t_door_o`, `t_grass →` no roof.

### 2.7 Sync, lifecycle, input, UI — the rules of the conversation
- Transport: local stream socket, **u32 LE length + one FlatBuffers
  `Envelope`**, 32 MiB cap, `CDDA` file identifier, verifier before
  any table access. Game is server; engine is client and spawns it
  (`--frontend=ipc --ipc-path=<path>`).
- Handshake: `Hello`/`HelloAck`, `PROTOCOL_VERSION` must match
  exactly (currently 1) or both sides exit cleanly.
- Sync by monotonic **`seq` + `epoch`**, never by game turn. Fresh
  Snapshot bumps `epoch`; a skipped `seq` ⇒ engine requests a fresh
  snapshot; duplicate `seq` discarded; old-`epoch` delta discarded
  (`proto::sync::SeqTracker`).
- **There is no per-tile "map changed" hook in CDDA.** Real delta
  export is: coarse invalidation (`map::invalidate_map_cache(z)`,
  `set_seen_cache_dirty`, `set_transparency_cache_dirty`) collects
  dirty submaps, then the exporter *diffs* them. The sim should
  generate its Deltas the same way (mutate world → mark submap dirty
  → diff → Delta), so the engine is tested against diff-shaped
  deltas, not hand-written ones. `map::shift` (bubble recentring) ⇒
  new epoch + fresh Snapshot.
- Send boundary in the real game: `game::do_turn()` exit
  (do_turn.cpp:536) + `game::draw()` redraw point (game.cpp:3364).
  The sim's equivalent: **one input = one turn boundary = one flush**
  (Delta + optional LightUpdate/Effects/EnvironmentUpdate/UiFrame).
- Heartbeat: Ping/Pong, 1 s cadence, 3 s silence = dead. Shutdown:
  either side, clean exit < 2 s. Dropped connection = Shutdown.
- Input (engine → game): `Input` (Key/Text/MouseButton/MouseMove/
  Wheel/Focus, platform-independent key code, modifier bitmask
  1/2/4/8 = shift/ctrl/alt/meta), `Resize` (physical px + DPI),
  `PickResult` (request id, hit, tile pos). In the real game these
  are injected at `input_context::get_raw_input()`
  (input_context.h:427).
- UI: pixels **never** travel on the socket. The game renders all
  `catacurses` windows except the terrain window into shared memory
  (48-byte `UiShmHeader`, magic `0x43444155`, double-buffered,
  dirty rect, seq + checksum — `proto::shm`, `crates/ui-share`);
  the socket only carries the `UiFrame` notification: buffer index,
  size/stride, seq, dirty rect, **`mode` (`MapInput`/`UiInput`)**,
  and the **terrain rect** in UI pixels (reported per frame, never
  assumed — it depends on terminal size/panel layout). In `UiInput`,
  clicks stay in UI space and camera drags are ignored; in
  `MapInput`, a click inside the terrain rect is a world pick.
- Effects: `Effects` message (Animation/Projectile/Sound/HitFlash/
  Explosion/Other, from/to, duration) — the fork hook is
  `src/animation.cpp` dispatch.

## 3. Design of the test game

### 3.1 Shape
```
crates/sim/            # library: world, rules, FOV/light, UI state, scenes, scenarios
  src/world.rs         # sparse tile world, def registry, entity/vehicle stores
  src/rules.rs         # turn resolution: move, open/close, bash, pickup, fields, combat
  src/visibility.rs    # lit/seen/memory/outside recompute (LOS + roof scan)
  src/environment.rs   # calendar advance, sun/moon, weather presets + scripts
  src/ui.rs            # UI state machine -> RGBA frames into ui-share store format
  src/scenes.rs        # ASCII-map scene loader + built-in scenes (§4)
  src/scenario.rs      # scripted test DSL (§5)
  src/server.rs        # socket server: framing, handshake, diff->Delta, heartbeat
tools/sim-server/      # binary: sim-server <socket> --scene <name> [--scenario <file>] [--seed n]
crates/engine/tests/sim_gate.rs   # engine-side gates driving a live sim (§6)
```
- **Language: Rust** (recommendation — needs Joey's nod, §8). One
  implementation of the wire, no second `samples.rs`↔`house.h`
  hand-sync, deterministic tests run under `cargo test`, and it can
  be linked *in-process* for fast gates as well as run as a separate
  process over a real socket for honest IPC gates.
- **Deterministic:** seeded RNG (same xorshift family as the fork
  harness mock), fixed turn step, no wall-clock anywhere except
  heartbeat. Same seed + same inputs ⇒ byte-identical message log.
- **Two driving modes, same world code:**
  1. *Process mode* — `sim-server` on a Unix socket; the real engine
     binary / `proto-client` connects exactly as it will to the fork.
  2. *In-process mode* — tests construct the world, feed `InputT`s,
     and assert on the emitted `MessageT`s and on the engine's
     ingested state, with no socket.
- **The engine is not modified to accommodate the sim.** If the sim
  needs an engine change, that is a finding about the live-loop
  wiring (Phase 2h/10.6), reported as such — the sim's whole value
  is that it speaks only the documented protocol.

### 3.2 The minimal rule set (simulate the *contract*, not CDDA)
Deliberately small, each rule chosen because an engine system
consumes its output:

| Rule | Behaviour in the sim | Engine system it exercises |
|---|---|---|
| Movement | 8-way step per movement key; blocked by wall / closed door / closed window / vehicle part / creature; costs 1 turn, **plus movecost time scaling (Phase 11 follow-up, settled 2026-11-24):** a successful step spends `combined_movecost` points — `(cost_from + cost_to) × 25` cardinal, `× 71 / 2` diagonal, over the tiles' raw JSON costs (0.I-1 src/map.cpp:2466-2488; deducted via `run_cost`, src/game.cpp:10804-10814; the baseline avatar's `run_cost` is the identity) — from a move balance carried across actions (100 points = 1 s: src/calendar.h:216-222; refills `moves += get_speed()` per second, src/creature.cpp:388). Each refill a spend drives is an elapsed second the flush lives through, so fields/creatures tick once per elapsed second (src/do_turn.cpp:655/666), not once per action. Stair z-moves spend `move_vertical`'s base 100 points (src/game.cpp:12331, deducted :12621). Blocked steps, bump attacks, waits, open/close, bash, pickup/drop stay 1 s and never touch the balance | Delta ingest, billboard move, camera follow |
| Open / close | Stepping into / activating a tile with `open_result` swaps terrain to that def (and back via `close_result`); recomputes transparency | TileChange restamp, LOS/light change through a doorway |
| Bash / destroy | Bash action on adjacent tile: terrain/furniture follows `bash_result` chain (door → bashed door → rubble/air per def table); furniture with no bash result is removed; emits HitFlash + Sound effects. **S6 settled:** the key is the game's own — `s` arms, a direction key executes (0.I-1 `data/raw/keybindings.json`: `smash` = `s`, like `open`/`close` = `o`/`c`); resolution mirrors `map::bash_ter_furn` (0.I-1 src/map.cpp:3932+) — furniture on the tile is smashed first (its `bash_result` furn_set applies; `"f_null"` removes it), terrain otherwise (ter_set; a `"t_null"` result destroys the tile to `t_open_air`, map.cpp:4203). Two named simplifications: **one bash = one chain step** (no strength stat; upstream rolls `strength + damage >= str_max` with damage accruing), and furniture with NO `bash_result` is **removed outright** (this plan's rule — upstream's missing bash spec falls through to the terrain). Terrain with no `bash_result` is unbashable (no-op, turn consumed). Success emits exactly one HitFlash + one Sound (`"smash_success"`, map.cpp:4108) at the target. Creature tiles (bump attack's) and vehicle-part tiles (the vehicle damage API's) are out of bash scope — no-ops. **Keymap settlement:** real bindings win over movement aliases — the S2 WASD aliases `s` (south) / `d` (east) are removed (`w`/`a` stay; south = `j`, east = `l`), in the UI cursor keys too; pre-S6 gates amended with the reason recorded in place | Fine-prop restamp removal (Phase 9), hardcases state=id-change (8.4), transient effects |
| Stairs / z-move | `>`/`<` on a stair tile moves player z ±1, snapshot window recentres. **S8 settled:** `>` descends, `<` ascends (CDDA's convention — 0.I-1 src/action.cpp:498-501, src/mapdata.h:184-185); the move requires the matching flag at the player's tile — GOES_DOWN for `>`, GOES_UP for `<`, on the terrain OR furniture def (upstream's gate is `game::move_vertical`'s `has_flag(TFLAG_GOES_DOWN/GOES_UP, pos)`, game.cpp:12430 / 12336). Destination = same (x, y) at z ±1; the sim's landing rule: it must exist, be passable (the movecost rule), be unoccupied, and not be NO_FLOOR terrain (you land on a floor/stair, not into open air). Otherwise a no-op turn. Upstream's climbing fallback (no-stair climb checks, wall cling, swimming) is NOT modelled. A z change crosses a submap boundary (the submap index includes z), so the flush rides the map-shift path: fresh Snapshot, epoch + 1 — the "window recentres" of the contract in the sim's no-culling form (a Snapshot carries all scene submaps) | Multi-storey seating, cutaway, cave factor |
| Items | Pickup removes top item / decrements `item_count`; drop reverses it. Piles only — no inventory simulation beyond a list for the UI. **S6 settled:** keys are the game's (`pickup` = `g`, `drop` = `d`, keybindings.json), acting on the player's OWN tile only (CDDA's adjacent-pickup UI is out of scope). Pickup takes ONE unit: `item_count` −1, `top_item` stable until the last unit clears it; the unit joins the S4 carried list (the inventory UI shows it — T8 ties T2 shut). Drop takes one unit from the FIRST carried stack (list order): an empty tile takes it as top_item × 1, a matching pile grows by one, a pile of a DIFFERENT kind **refuses** the drop (the lossy wire holds one kind per tile) — state unchanged but the turn is still consumed (the attempted-action rule, as for a blocked step). Empty-tile pickup / empty-hands drop are no-op turns | top_item billboard/item layer, UI inventory contents |
| Fields | Fire spreads to adjacent flammable tiles, intensity ±, burns out to smoke; smoke drifts with wind and decays. `displayed_field` recomputed by the game's rule (highest intensity, fire over smoke on tie — as in the sample house). **S5 settled:** the tick ports `field_processor_fd_fire`'s shape (0.I-1 map_field.cpp) — spread chance `25 × (intensity − 1)` as rng(1,100) < chance, targets gated FLAMMABLE/FLAMMABLE_ASH (power ≥ 2) / FLAMMABLE_HARD (power ≥ 3), growth cap 1 + flammable-tile + burning-neighbour rule (≥3 → 2, ≥7 → 3), smoke created on the fire tile at the fire's intensity; smoke drift follows `gas_spread_to` (source −1, downwind dest +1, intensity > 1 gate). Wind = the S3 fixture constant (5 mph at 90° → (+1, 0) by `convert_wind_to_coord`, weather.cpp). Timescale is compressed (upstream fuel is field-age in tens of minutes; sim budgets fuel in turns by class) and fire damage is the expectation of upstream's flat 3 + rng by intensity = [7, 12, 18]/turn. Note: `t_grass` is NOT flammable in 0.I-1 — grass does not carry fire; floors/furniture do | fx field volumes, heat/smoke coupling, displayed-field correctness from Delta alone |
| Creatures | Spawn (script or scenario), simple AI per turn: idle → wander (seeded) → chase player if visible (LOS + lit != BLANK) → attack in range (hp_frac drops, HitFlash). Death ⇒ `entity_removed` + corpse item. **S5 settled:** there is NO EntityAdded on the wire — Delta carries `entity_updates` + `entity_removed` only, so spawns ride `entity_updates`; runtime spawn entry point `World::spawn_monster` (S7's scripts drive it). Damage = the def's melee-dice **expected value** (importer now carries hp + melee dice from the monster defs); player bump-attack = fixed sim constant 8; corpse = item type `corpse` (`mdeath::normal`'s drop) as top_item. Player death is NOT simulated: player HP floors at 1. Greedy-chase limit (documented): Chebyshev greedy stalls in local minima behind obstacles — upstream pathfinds, the sim keeps the greedy rule per the non-goal below. Overlays empty (no grounded source in def data); FIREPROOF field exemption not modelled (monster flags are not in defs.json) | Billboard spawn/move/despawn, occlusion, picking, entity delta handling |
| Vehicles | Static by default; scenario can translate/rotate (parts' absolute positions recomputed sim-side, `face_deg` continuous), damage a part (hp_frac, broken flag), remove it. **S6 settled:** vehicles live in a sim-side STORE (`VehicleState`: id, name, origin = the mount-(0,0,0) pivot tile, `face_deg` f32 stored/sent verbatim, parts keyed by (vpart, mount) with hp_frac + flags); the wire `VehicleT` is derived from it and diffed by id like entities. Absolute positions are recomputed by a faithful port of 0.I-1's `precalc_mounts` / `coord_translate` (src/vehicle.cpp:3643/3598) over its `tileray` (src/tileray.cpp): the ray delta is (100·cos θ, 100·sin θ) truncated, advance() walks Bresenham-style, the across-mount enters via the quadrant-signed orthogonal offset — face 0 = identity, face 90 maps (x, y) → (−y, x) exactly, intermediate angles quantise the way the game's own ray does. World APIs (S7's DSL wraps them): `damage_vehicle_part` (hp_frac saturating; 0 sets the broken flag), `remove_vehicle_part`, `translate_vehicle`, `set_vehicle_face`. Scene format: `[[vehicle]]` gains `origin`; `[[vpart]]` declares (type, mount, hp_frac, flags) — a declared `pos` is now optional and only seeds a missing origin (origin = pos − precalc(mount)), so the S1-era fixtures produce identical wire vehicles. A vehicle part's tile blocks movement (one passability rule for player + creatures; no per-part passability data exists in the defs) | Vehicle stamping, roof-part split, restamp on damage/removal |
| Time / weather | Every turn advances the calendar (**1 s/turn** — S3 grounded this in the 0.I-1 source: `calendar::turn += 1_turns` per do_turn and `time_duration`'s unit IS the turn; the earlier "6 s/turn" wording came from the old fixture convention and is superseded); a movement turn onto expensive terrain spans several seconds (movecost scaling — see the Movement row), and weather-script entries keyed on absolute calendar turns fire inside the span at their due second; sun/moon recomputed; weather is a named preset or script (clear/sunny/cloudy/rain/storm/lightning, resolved from 0.I-1 `weather_type.json`; "clear" is the game's built-in default type — it has no JSON entry) | SkyState, Phase 10 GI, fog/precip, EnvironmentUpdate cadence (10-min rule) |
| Visibility | After any world/player change: LOS flood from player (radius ≤ 60, blocked by opaque defs, 3D z-range rule), lit from sun/moon + scene light sources + field light (S5: fd_fire emits `light_emitted` **20 / 60 / 160** per intensity into the same lm + colour cache — 0.I-1 lightmap.cpp:535-540 + field_type.json; colour from master field_type.json `light_color` [255,120,40] / [255,180,80] / [255,255,210], 0.I-1 having no field light colour), **`light_frac`/`light_tint` derived from the same apparent-light values (§2.3)**, `outside` = `is_outside && !is_roofed` resolved sim-side (S3: `build_outside_cache`'s INDOORS Chebyshev-1 stamp + the backported `is_roofed` upward walk — NOT the S1 enclosure-flood heuristic, and the scene `emit_outside` key is vestigial; the array is always emitted), previously-seen → MEMORIZED memory tiles | Lighting bake, VisibilityMask (BLANK excluded in both renderers), sky-exposure channel (wire, not column scan) |
| Map shift | Player crossing a submap edge recentres the bubble: epoch++, fresh Snapshot | SeqTracker epoch handling, resync (8.12) |
| UI state | See §3.3 | UI compositing, input routing, 4.12 latency path |

Explicit **non-goals**: no save/load, no JSON mod loading (except
the def-universe import in §8.3, if Joey picks B or C), no NPC
dialogue, no crafting simulation (the crafting *screen* opens; its
contents are a list), no hunger/health simulation beyond hp_frac,
no roof collapse when a wall is destroyed, no pathfinding beyond
greedy chase, no SDL/curses — the sim's UI is drawn CPU-side into
the shm store with real font rendering (§3.3, Joey's decision).

### 3.3 UI simulation — real font rendering + a state machine (Joey's decision, 2026-11-23)
Layout starts from the five existing synthetic frames (main menu,
character creation, inventory, crafting, gameplay sidebar) — same
panel geometry and terrain rect — but the text is **real rendered
text, not the synthetic bar-glyph stand-ins**: S4 now includes a
font rasteriser in the sim (a Rust glyph crate, e.g. `fontdue` /
`ab_glyph`), a bundled monospace font matching CDDA's own choice
(CDDA's font loader always ensures Unifont as fallback,
`src/font_loader.cpp`; a bundled OFL monospace is the fallback if
Unifont's licence/embedding is awkward — font choice is an S4
implementation detail, recorded when made), and a catacurses-style
character-cell grid (cell size from the font, foreground/background
colour pairs per cell, dirty rect computed per changed cell row).
That is what makes the inventory test honest: the item names are
readable text in the frame, and a capture can be checked for the
actual string "rock × 3", not for a bar of the right length.
Cost accepted with the decision: S4 grows (glyph cache, text
layout/wrapping, wide chars deferred) and UI goldens become
font-version-sensitive — goldens must pin the bundled font file's
hash. Add a state machine on top:

- States: `Gameplay` (mode=MapInput, sidebar frame, terrain rect =
  left portion as in `synthetic_terrain_rect`), `Inventory`,
  `Crafting`, `LookAround`, `MainMenu` (all mode=UiInput, full-frame).
- Transitions on the real key bindings the engine will send:
  `i` → Inventory, `&`/craft key → Crafting, `;` → LookAround,
  `Esc`/`q` → back to Gameplay. While in UiInput, movement keys are
  consumed by the UI (list cursor moves, world does **not** change) —
  this is the single most valuable UI assertion the sim enables.
- Every transition (and cursor move) writes the inactive shm buffer,
  publishes with bumped seq + dirty rect covering the changed panel,
  and sends one `UiFrame`. Inventory frame lists the sim's actual
  carried-item list, so pickup → open inventory is an end-to-end check.
- Resize: on `Resize`, redraw at the new size, terrain rect scales
  with the layout rule, next `UiFrame` carries the new geometry.

**Settled in S4 (2026-11-24):**

- **Font choice (recorded per the paragraph above):** rasteriser =
  **`ab_glyph` 0.2.32** (named above as an acceptable crate;
  `fontdue` was not fetchable in this environment — crates.io
  returns 403 through the proxy and it is not in the local
  registry cache, while `ab_glyph` was already in the workspace
  lock). Font = **Liberation Mono Regular** (the documented OFL
  fallback: SIL Open Font License 1.1; licence text bundled at
  `crates/sim/assets/LICENSE-LiberationMono.txt`), embedded from
  `crates/sim/assets/LiberationMono-Regular.ttf`, SHA-256
  `395fa5ab8d40c8eba390ced528744ea75a7f69aabf3e68b6f925ca0e39a27370`.
  Goldens pin the bytes by FNV-1a (`0x0b0d852dc58d8cc0`, asserted
  in a unit test). Cell metrics come from the font at 16 px:
  cell = the `M` advance × the ascent−descent+line-gap height.
  Bold is a double-strike (coverage OR-ed with itself shifted
  1 px), the catacurses convention.
- **MainMenu entry (was unspecified above):** the menu is an
  *entry* state, selected by the scene key `ui_start =
  "main-menu"` or the server flag `--ui-start <state>` (the
  same override shape as `--weather`). It is deliberately NOT
  on Esc-from-Gameplay (T2 pins Esc = back to Gameplay from the
  three sub-screens); inside MainMenu, Esc/`q` are no-ops (no
  game exists behind it yet) and the entries are display-only.
- **Shm path convention:** the server creates the UI file store
  at `--ui-shm <path>`, defaulting to `<socket>.ui-shm` beside
  the socket (the phase4-ui convention: the game creates the
  file, the engine maps it), and removes it on exit. Over the
  socket, the server sends one initial `UiFrame` right after
  the Snapshot (mirroring the real game's draw-point send);
  `measure` and the server gate consume it as part of the
  handshake.
- **Store generations:** the ui-share store's geometry is fixed
  at creation, so both the post-handshake attach and every
  Resize create a NEW store generation at the same path/role;
  the frame seq restarts within the new store and the next
  publish is a full-frame dirty rect. State, cursor and look
  cursor survive a resize.
- **Input routing details:** `Key` is the action channel; `Text`
  inputs are accepted and ignored in both modes (the engine
  sends Key+Text pairs for printable keys — routing Text would
  double-drive turns and UI). In UiInput the full vi-key set
  plus arrows/PgUp/PgDn drive the list/look cursor (`h`/`l`
  included — no movement key may fall through to the world).
  The UI-open bindings (`i`/`&`/`;`) in MapInput are not turns:
  no Delta, no clock advance, exactly one UiFrame. Mouse: in
  UiInput, clicks hit-test the list/menu rows (cursor moves,
  one UiFrame, never a pick); in MapInput the sim emits nothing
  for clicks — the engine routes terrain-rect clicks through
  its own picking and answers with a `PickResult` message,
  which the sim records verbatim (`World::last_pick`) without
  mutating anything.
- **Carried list:** scenes gain `[[carried]]` tables (`type` =
  Item string id, `count`) seeding the inventory; display name
  = the def's `name` when present, else its string id. Pickup
  and drop remain S6 (T8).

### 3.4 Scene format
One text file per scene, loadable by both the sim and (for fixture
parity) the engine gates:

```toml
# scenes/roofed_house.toml
name = "roofed_house"
seed = 1
time = "noon"            # preset (midnight/dawn/noon/dusk), or turn = 43200
weather = "clear"
player = [6, 6, 0]
[z0]
legend = { "#" = "t_wall", "." = "t_floor", "D" = "t_door_c",
           "W" = "t_window", "," = "t_grass", "T" = "f_table" }
map = [ "............", ".##########.", ... ]
[z1]                     # explicit roof level; omit for implicit-roof variant
...
[[spawn]]  type = "mon_zombie", pos = [9, 6, 0], hp_frac = 200
[[vehicle]] name = "car", face_deg = 37.5, parts = [...]
[[light]]  pos = [12, 9, 0], color = [1.0, 0.6, 0.2], intensity = 6.0
[[weather_script]]  turn = 50000, weather = "lightning"
```
S3 scene additions: `turn = <int>` (absolute calendar start turn,
wins over the `time` preset); `[[light]]` — a scene-declared
light source (position, float-RGB colour energy, luminance at
the source tile) feeding the visibility recompute's lm +
colour caches, the sim's analogue of a def's `light_emitted`;
`[[weather_script]]` — at the entry's absolute calendar turn the
weather preset changes (drives T6's storm segment).
ASCII layers per z keep scenes reviewable in a diff and close to
CDDA's own mapgen mental model. Furniture/fields/items are legend
characters on an overlay layer or explicit `[[place]]` entries.

## 4. Starter scene set (each one is a port or a named gap)

1. **stub_house** — the canonical Phase 1 house. Acceptance: the
   sim's first Snapshot is semantically equal to
   `proto::samples::sample_snapshot()` (this pins the sim to the
   existing goldens instead of drifting from them).
2. **roofed_house_explicit / roofed_house_implicit** — ports of
   `sample_roofed_house_snapshot` / `sample_implicit_roof_snapshot`
   (bungalow footprint, `roof_result` values from real game data).
3. **shelter, road_field, multistorey, vehicle_road** — ports of the
   fork harness's five mock worlds, so exporter fixtures and engine
   fixtures finally describe the same places.
4. **street_block** *(new — the outside/inside workhorse)*: two
   roofed houses on a road, trees, chain-link fence run, parked car,
   street items. Big enough that walking out a door crosses submaps.
5. **underground_lab** *(built, Phase 11 follow-up 2026-11-24)*:
   a fenced surface stairhead over a sealed lab at z −1 — for
   cave-factor / no-sky tests. As built (12×12 sections): z 0 is
   open grass with a chain-link fence ring (t_chainfence, gap on
   the east side) at Chebyshev distance 2 around the t_stairs_down
   tile at (4,4), nothing modelled above, so the pad's open
   ground reads outside=true in daylight. The stair tile itself
   reads outside=FALSE — its own def carries INDOORS, and
   build_outside_cache (0.I-1 map.cpp:9294-9305) stamps an
   INDOORS tile and its Chebyshev-1 neighbourhood as not
   outside; the game itself never reports a stair tile as
   outside, and the gate pins the faithful value. z −1 is three
   enclosed rooms cut into t_rock (entry room NW holding the
   t_stairs_up, east room through the door at (6,5), south-west
   room through the door at (4,6)), NO light sources, NO
   openings besides the stair shaft: outside=false on all 144
   tiles, visible lit LOW / unseen BLANK (the §5.2 settlement —
   this sketch's original "DARK lit" was wrong the same way
   T9's was). Gates: `crates/sim/tests/lab_gate.rs` (round trip
   on the z-move rule, sealed-level snapshot, both door walks,
   surface outside values) and
   `crates/engine/tests/sim_lab_gate.rs` (every z −1 tile's
   sky-exposure factor is 0 from the wire grid;
   cave_factor_for engages at the lab tile and not at the
   surface stairhead).
6. **field_fire** *(built, S5)*: grass + a cabin, one lit fire tile —
   field spread, smoke drift, night light pool. The fire's fuel is
   the cabin (FLAMMABLE_HARD floor + FLAMMABLE tables): `t_grass`
   carries no FLAMMABLE flag in 0.I-1, so the grass itself does not
   burn — the scene header records the finding.
7. **destructible_street** *(built, S6)*: a walled garage (24×12) —
   a window (7,2) and a partition door (7,6) whose sealed south
   chamber proves LOS/lit through a bashed opening, f_table
   (no bash_result → removed) + f_board_map (one furn_set step,
   then removed), a rock pile ×3 + a stick for T8's pickup/drop
   and mixed-pile refusal, and a 13-part store car at 23.0°
   for T11. Note: street_block (§4.4's "parked car") never
   actually got one in the S2 scene reduction — the T11 car
   lives here instead.

## 5. Scenario DSL — how tests are written

A scenario is an ordered step list (TOML, same file style as scenes).
The runner executes it in either driving mode and fails on the first
unmet assertion, printing the message log up to that step.

```toml
scene = "street_block"
steps = [
  { do = "key", key = "s" },                    # walk to the door
  { do = "key", key = "s" },
  { assert_tile = [6, 11, 0], terrain = "t_door_o" },   # auto-opened? no —
  { assert_player = [6, 10, 0] },               # closed door blocked the step
  { do = "key", key = "o" }, { do = "key", key = "s" },
  { assert_tile = [6, 11, 0], terrain = "t_door_o" },
  { do = "key", key = "s" },
  { assert_outside = [6, 12, 0], value = true },        # stepped outside
  { assert_lit = [6, 12, 0], value = "BRIGHT" },
  { do = "spawn", type = "mon_zombie", pos = [8, 12, 0] },
  { assert_entity_visible = "mon_zombie" },
  { do = "key", key = "i" },
  { assert_ui_mode = "UiInput" }, { assert_ui_screen = "inventory" },
  { do = "key", key = "s" },                    # cursor move, NOT a step south
  { assert_player = [6, 12, 0] },
  { do = "key", key = "Esc" }, { assert_ui_mode = "MapInput" },
  { do = "bash", pos = [6, 11, 0] },
  { assert_tile = [6, 11, 0], terrain = "t_door_bashed" },
  { do = "advance_time", minutes = 720 },       # to midnight
  { assert_env = { weather = "clear", sun_altitude_lt = 0.0 } },
  { do = "capture", name = "street_midnight" }, # headless frame + golden diff
]
```

Step vocabulary (v1): `key`, `text`, `click` (UI px or tile),
`resize`, `advance_turns`, `advance_time`, `set_weather`,
`spawn`, `damage_tile`, `damage_entity`, `remove_vehicle_part`,
`snapshot` (force resync), `disconnect`/`reconnect`, `ping`.
Assertion vocabulary (v1): **wire** (`assert_message` type/seq/epoch,
`assert_delta_contains`, mirror==snapshot invariant), **world**
(`assert_tile`, `assert_player`, `assert_lit/seen/outside`,
`assert_entity_*`, `assert_ui_mode/screen`), **engine** (ingested
grid cell, fine-layer volume present/absent, sky exposure at tile),
**render** (`capture` + region/golden checks, reusing the 2g.3
image-diff harness).

### 5.1 S7 settlements (2026-11-24, as built)

The runner lives in `crates/sim/src/scenario.rs`; scenario files
live in `crates/sim/scenarios/`; the mirror the invariant uses is
`crates/sim/src/mirror.rs` (factored out of the per-slice gates'
private copies — the gates keep theirs). `sim-server --scenario
<file>` runs a scenario in-process and exits 0/1; the cargo gate
`crates/sim/tests/scenario_gate.rs` runs the green set in-process
and a subset over a real socket. Settled points:

- **File shape:** top-level `scene` (required), `name`, `seed`
  (overrides the scene's), `status` (`"s8"` = written but excluded
  from the green set), `steps = [...]`. A step is one inline table
  with a `do` key (action, fields flat beside it) or exactly one
  `assert_*` key (fields inside its table, or the bare payload
  form `{ assert_player = [x, y, z] }`). The worked example above
  predates two settlements and is left as written: its `s` keys
  meant "south" under the old WASD aliases (S6's keymap
  settlement removed them — `s` is now smash) and its positions
  are the old street_block sketch, not the built scene.
- **`damage_tile` ≡ the `bash` step:** tiles have no HP; the step
  advances the tile's bash chain one step via the S6 machinery
  (`World::bash_tile`), no player involved. `damage_entity` deals
  N damage to the first live entity of the named type through the
  S5 combat path (death + corpse included). Script-mutation steps
  (spawn / bash / damage_* / vehicle mutations) flush a turn after
  applying, so the change rides that step's Delta.
- **`capture` is rejected at parse time**, with an error naming
  the CaptureArgs limitation (pixel capture takes fixture names,
  not wire snapshots). Green-set scenarios must not use it.
  *(The limitation itself was resolved after S8 by the Lane A
  wire-source change — §5.3: `CaptureArgs.wire` now seats a
  wire-built world in the frame loop. The runner still rejects
  the step: capturing is an engine-side act, and the engine
  gates assert it; the rejection's wording names the historical
  reason.)* The
  **engine** and **render** assertion classes of the v1 vocabulary
  are likewise not implemented by the sim runner — they belong to
  the engine-side gates, which already assert them against the
  same scenes.
- **The mirror invariant** (mirror == a fresh Snapshot of the
  world) is checked after every action step in-process, over the
  state Deltas fully determine: tiles (incl. frac/tint), entities,
  vehicles. **Memory is scoped out of the per-step equality, by
  wire necessity:** memory blocks ride Snapshots only, so a
  delta-fed client cannot track memory membership between
  snapshots (a tile that leaves view memorized and one still in
  view but unlit both arrive as lit-BLANK live tiles, and the
  exporter's memory blocks exclude currently-visible tiles). In
  its place, at each Snapshot the runner checks memory
  *consistency*: wherever the incoming Snapshot transmits a memory
  for a tile the accumulated mirror also remembers, the remembered
  terrain/furniture must agree.
- **Per-mode step matrix.** In-process implements every step.
  Process mode (a real client over the socket) refuses the
  world-event steps by name — `spawn`, `bash`/`damage_tile`,
  `damage_entity`, `damage_vehicle_part`, `rotate_vehicle`,
  `translate_vehicle`, `remove_vehicle_part`, `set_weather`,
  `advance_time` — because the protocol has no client message
  that injects them; author the event into the scene instead
  (T3's process twin does exactly this: `zombie_yard` is
  street_block with the zombie authored in). Equivalents:
  `advance_turns` = that many `.` wait keys; `snapshot` =
  disconnect + reconnect (the fresh Snapshot must satisfy the
  memory-consistency check against the pre-disconnect mirror);
  `disconnect`/`reconnect` in-process = drop the mirror / reseed
  from a fresh world Snapshot (the world persists); `ping`
  in-process = log-only no-op, in process mode a real Ping→Pong
  nonce check. Assertions refused in process mode, by name:
  `assert_ui_screen` and `assert_carried` (the wire carries the
  UI *mode*, never the screen's identity or the item list) and
  `assert_env`'s `weather` preset name (the wire carries the
  weather *id*; `weather_id` and the sun-altitude checks work in
  both modes, as do all tile/entity/vehicle/message assertions
  against the stream-fed mirror).
- **Server lifecycle change (S7):** the sim-server now builds
  the world once and accepts connections one at a time — a
  client disconnect no longer ends the server (the world
  persists across connections; a dead connection's I/O error
  just ends that session). Shutdown and the handshake version
  mismatch still exit cleanly, as S0 pinned. This is what makes
  T10's reconnect half and `engine --connect` reconnections
  meaningful against the real binary.
- **Scenario file inventory:** green in-process — `t1_go_outside`,
  `t2_open_ui`, `t3_spawn_chase_kill`, `t4_bash`, `t5_door_cycle`,
  `t6_time_weather`, `t7_fields`, `t8_items`, `t9_stairs` (S8),
  `t10_resync`, `t11_vehicles`, `t12_soak` (S8, a scripted slice —
  see below); green over a real socket — `t1_go_outside`,
  `t3_process` (scene `crates/sim/scenes/zombie_yard.toml`),
  `t5_door_cycle`, `t9_stairs` (S8), `t10_resync`. T10's green half is the
  disconnect/reconnect + shift resync; its abuse half is S8's
  server fuzz gate (§5.2). T7's smoke-drift half stays gate-only (it needs a
  mid-run scene mutation scenario steps cannot express).
  No scenario file carries `status = "s8"` since S8: T9's rule
  and scene exist (§5.2), and T12's full form is a gate, not a
  scenario — `t12_soak.toml` runs a 200-turn scripted slice in
  the green set and names `soak_gate.rs` as the real thing.
- **Interactive mode:** `voxel-engine --connect <socket>`
  (crates/engine/src/client.rs) — a headless live session:
  handshake, Snapshot → world via `snapshot_world`, UiFrame +
  shm validated via `frame::prepare_ui`, stdin bytes → Key
  inputs (0x1b = Esc, `Q` = end), SeqTracker/HeartbeatTracker
  discipline, drain-then-Shutdown on exit. The **windowed**
  seating of this session was NOT built at S7: `render_frame` /
  CaptureArgs accepted fixture names only, so the windowed frame
  loop could not take a wire-fed world without the engine change
  already named as the pixel-capture-parity follow-up. The
  windowed human run is a desktop-machine step; the headless
  session is the proof available here (street_block: player
  walked (5,5) → (5,9), out of the house; clean exits both
  ends). *(Both halves were built after S8 by the Lane A
  wire-source change — §5.3: the session now seats its live
  world in the frame loop, headless captures prove it, and
  `--connect-window` runs the same seating in the windowed
  app; the human windowed run remains a desktop step.)*

### 5.2 S8 settlements (soak & fuzz; T9 / T10-abuse / T12)

- **Multistorey scene** (`crates/sim/scenes/multistorey.toml`):
  a port of the fork-harness mock `make_multistorey`
  (tools/fork-harness/mock_world.cpp:101) — its 9×9 walled
  stack (basement z −1, ground z 0, first floor z 1, roof
  z 2) kept, offset into a 12×12 section per level. The
  mock's bare `t_open_air` stairwell holes become real stair
  terrain (the mock predates the z-move rule): stair A at
  (3,3) ground `t_stairs_down` → basement `t_stairs_up`
  (the z 1 tile above it stays `t_open_air`, the mock's
  opening carried up); stair B at (7,7) ground
  `t_stairs_up` → first-floor `t_stairs_down`. The
  basement's non-building tiles are `t_rock`, the roof
  level is `t_flat_roof` over the footprint.
- **Basement light — the T9 sketch's "lit DARK" was wrong,
  corrected here and in the §6 row.** The fork exporter
  takes lit from `map::apparent_light_at` (0.I-1
  lightmap.cpp:801), which never returns DARK (DARK comes
  from `map::light_at`, which the exporter does not use).
  `build_sunlight_cache` fills a fully-inside level at
  `inside_light_level` = LIGHT_AMBIENT_LOW (3.5)
  (lightmap.cpp:340-345), and 3.5 ≥ the vision threshold at
  that ambient (≈3.22), so a *visible* sealed-basement tile
  reads **LOW**; tiles the player cannot see read BLANK.
  The ported S3 model already produces exactly this (z < 0
  gets no direct sun and no sun cast; the inside floor
  applies) — no model change and no scene change was
  needed; the gates assert the honest output:
  outside=false on all 144 basement tiles, every basement
  lit ∈ {LOW, BLANK}, the player's tile LOW
  (`stairs_gate.rs`, `t9_stairs.toml`).
- **Soak as-run (T12, `crates/sim/tests/soak_gate.rs`):**
  the plan's matrix verbatim — 1,100 turns × 10 seeds,
  seeded xorshift generator over the full key vocabulary
  (movement incl. diagonals, arm + direction keys, g, d,
  the UI opens, Esc ×2 + q so runs return to MapInput,
  `>`/`<`), scenes alternating destructible_street (even
  seeds) and zombie_yard (odd). After every flush: mirror
  == fresh Snapshot structurally AND by FNV-1a hash over a
  canonical mirror encoding; seq advances by exactly 1 per
  seq-bearing message; epoch never regresses. Measured on
  this VM (debug profile): 11,000 inputs, mean step
  17.5 ms, p95 70.9 ms, gate wall time ≈ 4.1 min — the
  assertions on step cost are generous named bounds (mean
  < 250 ms, p95 < 1 s) that trip on a pathological blow-up,
  not a product spec. **Zero map shifts occurred** in the
  10 runs (random walks stayed inside their starting
  submaps — both scenes start the player indoors); the
  shift path itself is pinned by T1/T10/T9, and the soak
  asserts the invariant whether or not a shift happens.
  CI: the soak runs in the nightly workflow (new
  `sim-soak` job in ci-nightly.yml); the PR job carries
  the fast S8 gates (stairs / fuzz / record).
- **Server fuzz (T10 abuse half,
  `crates/sim/tests/server_fuzz_gate.rs`):** a deterministic
  corpus driven at a real server — garbage bytes, a
  0xFFFFFFFF length prefix, a prefix one byte over the
  32 MiB cap, a valid prefix wrapping an invalid
  FlatBuffers payload, a truncated prefix, a well-formed
  wrong first message, empty reads, 20 rapid
  connect/disconnects — before AND after the handshake.
  Assertions: no panic, no hang (all client ops
  timeout-bounded), each bad connection closed cleanly,
  and a full normal session completes afterwards. **One
  real bug found and fixed:** a malformed FIRST frame
  (pre-HelloAck) propagated its decode/framing error out
  of `serve_one` and killed the accept loop — one hostile
  connection took the server down. The fix treats
  pre-handshake receive errors as ClientGone, exactly like
  the post-handshake path already did; the fuzz gate is
  its regression test (it fails on the pre-fix code at the
  second connection). Wrong protocol version is asserted
  separately as a clean server EXIT (the S0 rule), not a
  rejected connection. The wire-level T10 assertions
  (duplicate delta applied zero times, skipped seq ⇒
  snapshot request, replayed epoch discarded) were already
  covered engine/proto-side by proto's gate
  `t1_14_seq_epoch_semantics` (crates/proto/tests/
  gate_tests.rs, over `proto::sync::SeqTracker`) — recorded
  here, not duplicated.
- **Record/replay (`crates/sim/src/record.rs`,
  `sim-server --record <path>`):** the log format IS the
  §5 scenario format — scene name + effective seed +
  one step per recorded Input — so replay is the S7
  runner. Recorded: pressed Keys, Text, clicks, Resizes.
  Not recorded, by design: Key releases, MouseMove, Focus
  (none can change state) and Wheel inputs (the DSL has
  no wheel step; they are counted in the file's header
  comment when they occur, so the gap is on the record).
  The gate (`record_gate.rs`) drives a scripted live
  session, replays the recorded keys against a fresh
  server, and compares the full message streams
  frame-by-frame byte-for-byte — no class excluded (the
  sim has no wall-clock input; the gate's Ping barriers
  are deterministic): 31 frames, 1,409,476 bytes,
  byte-identical.

### 5.3 Lane A settlements (post-S8 follow-up: the wire-source engine change)

- **The source variant:** `CaptureArgs` gains
  `wire: Option<engine::wire::WireScene>` (new module
  `crates/engine/src/wire.rs`). A `WireScene` is a built wire
  world — `TileDefs` + `Snapshot` fed through
  `snapshot_world::build_snapshot_world` with a caller-chosen
  block resolver (gates: the S1 canonical palette; the client
  session: the coarse stand-in; a desktop run: the voxelizer
  materials) — carried as an `Arc<ChunkMap>` + the window's
  `min_game_z`. When seated, both renderer paths prepare from
  the wire map (`scene_prep::prepare_raster_wire`; the RT
  path's `world_from_wire` + the shared lit contract) and the
  fixture-name dispatch is untouched when it is `None` — the
  golden gates pass without regen, which is the regression
  fence. Companions seat as usual: `lighting` / `vehicles`
  apply to a wire world exactly as to a fixture world.
  Fixture-keyed inputs are not consulted for a wire world in
  v1: `textures` / `effects` (fake-id material tables) and the
  Phase 9 fixture prop scenes (name-keyed) — the wire world's
  blocks carry their resolver's colours already, and wire
  fine-prop seating remains the snapshot path's own stamping
  (`snapshot_world::stamp_furniture` / `stamp_terrain`) into a
  caller-owned layer the frame loop does not yet carry.
- **Live state:** the client session folds every accepted
  Delta into its stored Snapshot
  (`wire::apply_delta_to_snapshot`: per-tile changes land in
  their submap block's arrays, the sparse field list is
  replaced per tile with `field_index` rebuilt, entities /
  vehicles replace-by-id, removals drop, a piggybacked
  environment replaces) and republishes the `WireScene`, so
  the seated world is the delta-fed state, not the
  connect-time snapshot.
- **Pixel parity (gate `wire_source_gate.rs`):** the sim's
  stub_house Snapshot and the fixture `sample_snapshot()`,
  both through the wire path with the canonical palette,
  render **byte-identical PNGs on both renderers** (raster
  14,496 bytes; RT 11,679 bytes, lavapipe) — S1's world parity
  now holds at pixel level. The comparison against the legacy
  capture fixture named "stub-house"
  (`world::build_fake_house`) is a **named finding, not a
  parity target**: the two houses were authored separately
  (door (6,11) vs (5,11); windows (3,0)/(0,6)/(11,6) vs (2,0))
  and are stamped by different models (fixture `set_tile`
  shapes — walls/doors/windows a full 3-block level, roof
  slabs at z 3 — vs the wire builder's every-terrain-a-slab
  rule). The gate enumerates the ENTIRE world-level diff and
  pins it to exactly those causes: **238 cells** (144 roof
  band + 88 perimeter height + 6 authoring), and the captures
  differ in 17,840 of 230,400 pixels accordingly. Nothing in
  the seating path owns that diff; closing it would mean
  re-authoring one of the two houses or changing the wire
  builder's stamping model, both outside this follow-up.
- **Windowed seating:** `voxel-engine --connect-window
  <socket>` — the session runs on its own thread
  (`client::spawn_connect`), publishes its live scene to a
  shared slot the windowed app polls per frame into
  `frame_args.wire`, and receives the app's key presses over
  a channel as Inputs (camera keys WASD/E/Q/arrows/Escape
  stay local; closing the window ends the session via the
  client's input-close rule, drain + Shutdown, joined at
  exit). Fixture companions (fake-house lighting/sky,
  vehicle/transient fixtures) are NOT seated over a wire
  world; the windowed session's lighting is unseated in v1
  (the capture path seats wire + grid today). The windowed
  run itself is a desktop step — the headless proof is the
  gate: a real sim-server (street_block) + the client over a
  real socket, first frame captured from the live world at
  the first Snapshot, `jjojjj` walked the player (5,5) →
  (5,9), end frame captured from the delta-fed world after
  the drain — 20,341 of 230,400 pixels differ (door opened,
  camera follows the player), both frames non-blank, the
  final live state shows the door at (5,8) as `t_door_o`,
  clean Shutdown both ends.

## 6. Test matrix — the user's named tests, plus the ones they imply

| # | Test | Scenario sketch | Key assertions |
|---|---|---|---|
| T1 | **Go outside, then go back inside** | roofed house, noon; exit via door, walk the yard, re-enter, close door | `outside[]` flips false→true→false on the player tile and via Delta; lit LIT→BRIGHT→LIT; sky factor / cave factor follow the *wire* grid in both renderers; interior capture dims vs exterior; tiles left behind become MEMORIZED, not BLANK |
| T2 | **Open the UI** | gameplay → `i` → navigate → Esc; also crafting + look-around | UiMode MapInput→UiInput→MapInput; shm frame actually changes (seq++, dirty rect inside the panel); movement key in UiInput moves cursor, not player; terrain-rect masking still correct; click in UiInput produces no PickResult; click in MapInput terrain rect does |
| T3 | **Spawn enemies** | script spawn in view / behind a wall / in the dark; hostile chase over N turns; kill it | Entity appears in Delta (not just next Snapshot); billboard renders in view, is occluded behind wall, absent in BLANK; chase path updates pos per turn; death ⇒ `entity_removed` + corpse item on tile |
| T4 | **Destroy objects** | bash door, window, table; damage vehicle door/wheel | Terrain follows `bash_result` chain per TileChange; destroyed furniture's fine prop volume is *removed* by restamp (no ghost prop); LOS + lit recompute through the new opening; Effects HitFlash/Sound emitted once; vehicle part broken flag + restamp |
| T5 | Door open/close cycle | open, step through, close behind | `t_door_c → t_door_o → t_door_c`; transparency/LOS blocked again after close; no duplicate delta on a no-op close |
| T6 | Time & weather sweep | advance 24 h; storm script (mirrors stub `--env-script`) | EnvironmentUpdate cadence = on-change + 10-min time-only; sun altitude sign flips at dawn/dusk; storm: precip_class 3, `lightning_active`, fog/sight penalty reaches the frame; captures at dawn/noon/dusk/midnight × clear/storm differ in the Phase 10 §8 pattern |
| T7 | Fields: fire & smoke | light the cabin floor / grass | Field list grows/shrinks per tile; `displayed_field` correct from Delta alone; smoke drifts downwind; fire lights its tiles at night (lit bump) and damages a standing creature |
| T8 | Items: pickup & drop | rock pile (count 3) | `item_count` 3→2→… with `top_item` stable, last pickup clears `top_item`; drop restores; inventory UI lists the rocks (ties T2 shut) |
| T9 | Stairs & multi-storey | multistorey scene, go up, then to basement | Player z changes; snapshot window recentres (z −1/+1 around new z — in the sim's no-culling form: z change ⇒ fresh Snapshot, epoch + 1, the new level's tiles present); basement: outside=false everywhere, lit LOW on visible tiles / BLANK beyond LOS (**S8 correction:** the sketch's "lit DARK" is not what the exporter produces — `apparent_light_at` never returns DARK and a sealed level sits at inside_light_level 3.5 ≥ the vision threshold; see §5.2), cave gradient in capture |
| T10 | Resync & lifecycle | move across submap edge; kill/restart sim; send garbage seq | Shift ⇒ epoch++ + fresh Snapshot; duplicate delta applied zero times; skipped seq ⇒ engine requests snapshot; Ping/Pong; Shutdown < 2 s; reconnect gets TileDefs + Snapshot + piggybacked environment. **S8: the abuse half passes** — wire-level seq/epoch abuse was already pinned by proto's `t1_14_seq_epoch_semantics`; malformed-frame abuse of the sim's own server is `server_fuzz_gate` (one real bug found + fixed, §5.2) |
| T11 | Vehicles | rotate/translate the fixture car 23°→90°; remove a part | Part absolute positions move rigidly; `face_deg` stays float-exact; roof parts stay on the top block; removed part leaves no stamped tile |
| T12 | Soak / invariant | seeded random-input run, 1,100 turns × 10 seeds (fork-harness 3.9 parity) | After every Delta, engine mirror == a fresh Snapshot from the sim (hash compare); no seq regression; RSS/step cost flat (8.8-style). **S8 as-run: PASS** — 11,000 inputs, mean step 17.5 ms / p95 70.9 ms on the dev VM, 0 map shifts (see §5.2) |

T1–T4 are the acceptance set for v1; T5–T12 can land slice by slice.

## 7. Build slices, gates, and order

Each slice: failing test first (Joey's rule — the test must fail on
the pre-slice code), then the sim work, gate re-run, `plan.md`
updated in the same change.

| Slice | Scope | Gate |
|---|---|---|
| **S0 — Decisions + skeleton** | §8 decisions recorded; `crates/sim` + `tools/sim-server` scaffold; framing/handshake/heartbeat/shutdown over the real socket, reusing `proto` | Engine `proto-client` connects, handshakes, disconnects cleanly; version-mismatch exits cleanly |
| **S1a — Full def importer (Option C, Joey's pick)** | `tools/sim-defs`: census of `fork-src/cdda/data/json`, `copy-from`/abstract resolution, connect-group indices from load order, nested bash/open/close/roof extraction, `hot_flags` from the `ter_furn_flag` order, fg/bg join from the tileset crate; generated table checked in + census/closure validation | Generated counts match the source census per kind; zero dangling transform strings; zero abstracts emitted; spot-parity vs the fork harness's mock defs and vs hand-checked real entries (glass door bash chain, `t_door_c` open/roof) |
| **S1 — Static scenes over the wire** | Def registry from S1a's generated table, scene loader, Snapshot builder with diff infrastructure in place (all-dirty first send) | stub_house Snapshot ≡ `sample_snapshot()`; roofed-house pair matches the existing fixtures; engine capture of a sim scene ≡ capture of the same scene as a fixture (both renderers); **TileDefs connect time + message size measured and recorded** |
| **S2 — Turn loop: movement, doors, Delta** | Input→turn→dirty-submap diff→Delta; movement/collision; open/close transforms | **T1 (world/wire half) + T5 pass**; mirror invariant holds for a scripted 200-turn walk |
| **S3 — Visibility & environment** ✅ DONE 2026-11-23 | LOS/lit/seen/memory/outside recompute; **`light_frac` + `light_tint` recompute and wire export (Addendum 11 light amendment, §2.3)**; calendar + weather presets/scripts; EnvironmentUpdate cadence; LightUpdate folding | **T1 (full, incl. render) + T6 pass**; outside/lit grids match a hand-computed expectation for street_block at noon and midnight, **including per-tile frac/tint (a coloured source's pool tints, tiles outside it stay white)** — record in plan.md Phase 11 (incl. the LOW→BRIGHT→LOW correction to T1's expected lit sequence and the grid-level render-half deviation) |
| **S4 — UI** | shm writer (double-buffer, header, dirty rect), UI state machine (§3.3), Resize handling | **T2 pass**; ui-share gates re-run against sim-written frames (not just synthetic-written); torn-frame reader test against the sim writer |
| **S5 — Creatures, fields, effects** | Spawn/AI/combat/death; field spread/decay + displayed-field rule; Effects emission | **T3 + T7 pass** — DONE 2026-11-24 |
| **S6 — Destruction, items, vehicles** | Bash chains, furniture removal, item piles + inventory list, vehicle move/damage/remove | **T4 + T8 + T11 pass**; fine-layer restamp assertions in engine gates — DONE 2026-11-24 (v1 gate T1–T4 complete) |
| **S7 — Scenario DSL + interactive mode** | §5 runner (process + in-process), scenario files for T1–T12; `engine --connect` live session against sim-server you can play with the keyboard; CI job (headless, lavapipe) | All scenarios green in CI; a human can run the windowed engine against the sim and walk out of the house — **DONE 2026-11-24 with the windowed half named, not built**: scenarios green in-process + process subset over real sockets (locally validated CI commands; CI itself runs on push via Joey's PC route), `engine --connect` proven headless (walk out of the house, clean shutdown); the windowed seating waits on the CaptureArgs wire-source engine change (see §5.1) — **that change landed as the post-S8 Lane A follow-up (§5.3): pixel parity byte-identical on both renderers, windowed seating built as `--connect-window`** |
| **S8 — Soak & fuzz (optional, after v1)** ✅ DONE 2026-11-24 | Random-input soak, malformed-frame fuzz on the sim's own server side, record/replay of a session log | **T9/T10/T12 pass**; decode fuzz clean (parity with the existing proto fuzz gates) — z-move rule + multistorey scene (T9, with the basement-lit correction in §5.2), soak 1,100 × 10 as-run (T12), server fuzz with one real bug found + fixed (T10 abuse half), `--record` session logs replaying byte-identical (§5.2) |

Rough size: S0–S2 is the critical path to first value (a walkable,
door-opening world the engine renders live). S3–S6 are each
independent once S2's diff/turn loop exists and can be parallelised
in worktrees, one merge at a time, per the usual rule.

## 8. Decisions

### 8.1 Made 2026-11-23 (Joey)
1. **Language / location: Rust**, `crates/sim` + `tools/sim-server`. ✅
2. **Name: sim-server** (crate `sim`). ✅
4. **UI fidelity: real font rendering** (see §3.3). ✅
5. **Phase 11 in `plan.md`, T1–T4 as its v1 gate.** ✅ Added
   2026-11-23 (progress-log row, dependency row, Phase 11 section).

### 8.2 Def universe — DECIDED 2026-11-23 (Joey): Option C
**Full real import, everything** (§8.3, Option C). The sim's
TileDefs are generated from the real CDDA JSON at the pinned
commit — all terrain/furniture/traps/fields/monsters/vehicle
parts and all items — by an importer that resolves
`copy-from`/abstract inheritance, connect-group load order,
nested bash objects, and `hot_flags`, and joins fg/bg refs.
Consequences accepted with the decision and now in scope:
(1) the importer is a pre-S1 slice of its own (S1a in §7,
+2–3 slices, open tail on item JSON); (2) TileDefs is a
multi-MB handshake message — connect-time cost is measured in
the S1 gate, not assumed away; (3) every upstream rebase is a
regen + golden review; (4) validation gates are load-bearing:
every transform string must resolve to a sent def, every
abstract must be excluded, and the generated table's counts
must match a census of the source JSON. No other decision
remains open before S0.

### 8.3 Decision 3 in detail — where the sim's TileDefs come from

First, what a "def" actually is here, because the options differ
field by field. A def is one `TileDef` the sim sends once at
connect (§2.6): numeric id (sim-assigned, per-kind ranges like the
stub's), `string_id` (`"t_door_c"`), name, kind, authored flag
strings, the resolved `hot_flags` bitset (the game's
`ter_furn_flag` enum resolved to bits — this is what the engine
actually tests for TRANSPARENT etc., it never parses flag strings),
connect-group indices, `fg_ref/bg_ref` (indices into the composed
UltiCa tile config; −1 = no sprite), and the transform strings
`open_result / close_result / bash_result / looks_like /
roof_result`. **Every transform string must name another def in
the same table** or the chain dangles — open a door whose
`t_door_o` was never sent and the engine gets a tile id it cannot
resolve. Likewise a def with no voxelizer material renders as a
fallback block: a def is only *visible* if `assets/` has a model
for its string id (terrain/furniture/monster coverage is 100%
from the zero-drive, but only for ids the voxelizer has run on).

What the real game's JSON looks like (measured in
`fork-src/cdda/data/json` at the pinned commit): terrain 1,189 +
furniture 686 entries in `furniture_and_terrain/` alone, ~1,237
monsters, ~106 field types, ~93 vehicle parts there plus the
`vehicleparts/` tree, and **10,289 items**. And it is not flat
JSON you can just read: **401** of the terrain/furniture entries
use `copy-from` (inherit every field from another entry, override
some), 24 are `abstract` (templates, never instantiated), and
items use `copy-from` 4,991 times. Resolving that inheritance —
plus nested objects like `"bash": { "str_min":…, "ter_set":…,
"items":[…] }`, `"open"/"close"` as plain string fields, connect
groups given as *names* (`"connect_groups": "WALL"`) that become
bitset indices only through the load order of
`data/json/connect_groups.json`, and `hot_flags` which requires
reproducing the C++ `ter_furn_flag` enum order — is exactly what
CDDA's C++ `generic_factory` loader does, and what the fork's
real adapter (`voxel_real_adapter.cpp`, 418 lines) gets *for free*
by calling `get_all()` on the already-loaded game data. **Any
Rust importer is a second, partial reimplementation of that
loader.** That asymmetry is the whole decision.

**Option A — Hand-built subset (~40 defs).**
Write the defs in Rust code, like `samples.rs` / `house.h` do
today, expanding the sample set to cover the starter scenes:
grass/dirt/road/floor, wall/window/door_c/door_o (+ one bashed
variant each), tree, chain fence, stairs up/down, the roof set
(`t_flat_roof`, `t_wood_roof`, `t_wood_treated_roof`,
`t_shingle_flat_roof`), open air; table/bed/chair/bookcase;
spike trap; smoke/fire fields; rock/can items; zombie/dog;
car frame/wheel/door/roof parts. Every field is set explicitly:
flags chosen to match the real def, `hot_flags` computed from the
same flag list, transforms wired by hand, `fg_ref/bg_ref` copied
from the existing fixtures where they exist, else −1.
- *Fidelity:* exact for the ~40 ids, and the values are checked
  against the real JSON by eye when written. Zero coverage beyond
  them — a scene cannot use a def nobody wrote.
- *Cost:* smallest. Part of S1, no new tool, no JSON parsing.
- *Risk:* hand-drift — if upstream changes `t_door_c`'s bash
  result, the sim silently keeps the old one. The S1 parity gate
  (stub_house ≡ samples) does not catch this; a spot-check test
  against a handful of real JSON entries would, cheaply.
- *Tests it can run:* all of T1–T12 as designed — they were
  scoped to these defs.

**Option B — Curated real import (~150–300 defs).**
A small offline tool (`tools/sim-defs`, run manually, output
checked in as a generated Rust/JSON table — same pattern as the
fg_refs TSV) that reads the real JSON, resolves `copy-from` *for
a named allowlist only* (the starter-scene ids + their transform
closure: if `t_door_c` is in, its open/bash/roof targets are
pulled in automatically, recursively, so no chain dangles), maps
connect-group names to indices from `connect_groups.json`,
computes `hot_flags` from the flag strings via a Rust copy of the
`ter_furn_flag` order, and joins `fg_ref/bg_ref` from the
tileset crate's existing fg_refs output. The allowlist grows
when a new scene needs a new def — one line plus a regen.
- *Fidelity:* real values for everything imported — real bash
  chains (a laminated glass door really does bash to
  `t_mdoor_frame` and drop glass shards), real roofs, real flags.
  Still zero coverage outside the allowlist, by design.
- *Cost:* the importer itself is the work — `copy-from`
  resolution (chains, abstract bases, field override semantics),
  the nested bash object, connect-group ordering, and a
  validation gate (every transform resolves, every id has a
  material or is reported). Estimate: one extra slice before S1
  proper, and the tool lives forever (regen on rebase).
- *Risk:* partial-loader bugs — a mis-resolved inheritance looks
  like a game bug in a test failure. Mitigated by the closure +
  resolution gates and by keeping the allowlist small enough to
  eyeball the generated output.
- *Tests it can run:* T1–T12, plus scenes built from real
  mapgen-adjacent def sets (e.g. a real house's worth of
  furniture) without hand-writing each def.

**Option C — Full real import (everything).**
Same tool as B, no allowlist: all terrain/furniture/traps/
fields/monsters/vehicle parts — and, if it is to match what the
real adapter sends, **all 10,289 items too**, since the real
`TileDefs` includes every itype. Wire defs would number in the
~14,000s.
- *Fidelity:* maximal on paper. In practice the engine consumes
  almost none of it: an item def beyond its id is never read
  (the wire only ever carries `top_item` + count), a monster def
  beyond sprite/flags is never read, and 95% of terrain defs
  would never appear in any scene. The fidelity that *matters*
  (transforms/flags of defs actually placed) is identical to B.
- *Cost:* B's importer plus the long tail: item JSON is the
  messiest (4,991 `copy-from`s, migrations, item groups),
  `hot_flags` for furniture/terrain only exists for ter/furn —
  other kinds need a defined default, trap defs have no public
  registry in the pinned source (the fork adds a 3-line hook,
  H7 — a Rust importer has no equivalent problem but must find
  traps in `data/json/traps.json` + mod folders), and the
  handshake now ships a multi-MB TileDefs message, slowing every
  test's connect and making goldens churn on *any* upstream data
  change, even in content no scene uses.
- *Risk:* highest drift *surface* — every upstream rebase is a
  regen + a golden review, forever, for coverage no test uses.
  And a subtle one: it makes the sim look authoritative about
  the full def universe, when the authority is the fork's C++
  loader, which will produce the real table when it runs.
- *Tests it can run:* the same T1–T12. No planned test needs a
  def outside B's closure.

**Option D — Staged hybrid (A now, B later, C never).**
S1 ships on A so the turn loop, UI, and T1–T4 land first; the
B importer is scheduled as its own slice (say S6b) when a scene
first needs a def set too big to hand-write, migrating the
hand-built table to generated output with the same ids. C is
explicitly not planned: when the real fork runs, its adapter
supplies the full table and the sim's job is done.
- This is the original plan's recommendation, now with B's
  trigger named instead of "S8+": the first scene whose def
  count exceeds what fits comfortably in one reviewed file
  (~60–80 defs) triggers the importer slice.

**Summary table**

| | A hand-built | B curated import | C full import | D staged A→B |
|---|---|---|---|---|
| Defs | ~40 | ~150–300, closure-complete | ~14,000 | 40 → 150–300 |
| Values | hand-checked copies | real JSON, resolved | real JSON, resolved | copies → real |
| New tooling | none | importer + gates | importer + long-tail fixes | importer, deferred |
| Extra cost before S1 | none | +1 slice | +2–3 slices, open tail | none |
| Upstream rebase pain | silent drift on ~40 ids | regen + review allowlist | regen + review everything | drift, then regen |
| Unlocks a test A can't run | — | real-def scenes at scale | none over B | same as B, later |


## 9. Risks and honest limits

- **A sim proves the engine honours the contract; it does not prove
  the fork *produces* the contract.** 0.1/0.2 and the in-game halves
  of Phases 3/4 remain blocked exactly as before. The sim de-risks
  the engine side so that when the fork runs, a failure localises
  to the fork instead of "somewhere in the loop".
- **Drift risk:** the sim is a second implementation of game rules
  (LOS, displayed-field, roof/outside). Mitigation: every rule cites
  its CDDA source (§2), fixtures are shared with the existing
  samples/mock worlds (S1 gates pin byte/semantic parity), and any
  rule the pinned source contradicts gets fixed in the sim, with
  the finding recorded — same discipline as Phase 0's corrections.
- **Scope creep risk:** it is tempting to grow the sim toward a
  real game. The §3.2 table is the ceiling for v1; anything not
  consumed by an engine system stays out.
- **Windows:** socket transport is Unix-first like the stub; the
  named-pipe transport is already a known fork TODO and the sim
  inherits it unchanged — in-process mode covers Windows-side
  engine testing until then.
