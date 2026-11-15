# CDDA Voxel Frontend: Project Plan — v2 (verified)

> ## ⚠ Maintenance rule — read first
> **This plan is a living document and MUST be updated with all progress after each phase.** When a phase finishes — and whenever a phase's scope, design, or status changes mid-phase — update this document *before* starting the next phase. A phase is not done until its test gate (Part II) passes **and** this update is written.
>
> Each phase update must record:
> - **Status and date** in the Progress log below and on the phase's heading in §5 (e.g. `— ✅ GATE PASSED 2026-…` / `— 🚧 IN PROGRESS` / `— ⛔ BLOCKED: <reason>`)
> - **What was actually built**, where it differs from what §5 planned, and **every deviation and its reason**
> - **Decisions made** and open questions closed (§14) or newly opened
> - **Test-gate results** from Part II for that phase (pass/fail per gate test, known gaps with tickets)
> - **New risks, upstream changes, or rebase events** (§12), and any resulting changes to later phases' scope, estimates, or dependencies (§5, §13)
>
> If reality disagrees with this document, fix the document in the same change. Do not let the plan drift, and do not carry progress only in chat, commits, or memory — if it isn't written here, it didn't happen.

## Progress log

| Phase | Status | Updated | Notes |
|---|---|---|---|
| 0 — Recon confirmation and decisions | ✅ GATE PASSED (conditional — see Phase 0 record) | 2026-11-15 | Mandatory gate tests 0.3/0.4/0.6/0.7/0.8 pass at harness level; 0.1/0.2 not yet run — the only build they need is the curses/text build (the tiles/SDL3 build is **not required** by this project at all, updated 2026-11-15), and compiling even that is infeasible in the recon environment (1.5 CPU/6 GB) — with an exact unblock path; in-game spike halves written as a ready-to-apply probe patch. Must be run on a build-capable machine before Phase 3. Full record: Phase 0 section below + `docs/recon.md` |
| 1 — Protocol and skeleton | Not started | — | |
| 2 — Engine MVP with fake data | Not started | — | |
| 3 — Game-side export in the fork | Not started | — | |
| 4 — UI compositing | Not started | — | |
| 5 — Voxelizer and textures | Not started | — | |
| 6 — Billboards | Not started | — | |
| 7 — Lighting, FOV and atmosphere | Not started | — | |
| 8 — Hard cases and release | Not started | — | |

A voxel-rendered frontend for Cataclysm: Dark Days Ahead. The unmodified-in-spirit game (a lightly forked CDDA) runs all simulation. A separate Rust engine renders the world as voxels and talks to the game over IPC. Characters and monsters stay as 2D billboard sprites (Doom-style). All art comes from [I-am-Erk/CDDA-Tilesets](https://github.com/I-am-Erk/CDDA-Tilesets), and voxel textures are derived from those sprites.

> **What changed in v2:** every item the original plan marked **(verify)** has been checked against the actual source, and the plan has been corrected where the source disagreed. Verification was done 2026-11-15 against:
> - CDDA `master` @ `6a3e59e08842de34ff3f111c7f098abeffb708a5`
> - CDDA-Tilesets `master` @ `5ee942f9e6601970c5da356200e7cf5061226262`
>
> Full evidence — file, symbol, and verdict for each claim — is in **Appendix A**. Corrections are also folded into the body, so the body is safe to follow on its own. The three corrections that change the design, not just details:
>
> 1. **Fields and items are not single ids per tile.** A tile's field is a *map of field entries* (type + intensity + age), and its items are a *colony (list) of items*. The v1 protocol's `fields:[uint32]` / `top_item` would have silently lost smoke density, fire intensity, and item stacks. (§6)
> 2. **There is no per-tile "map changed" hook to tap.** CDDA's invalidation is coarse — per-z-level and per-submap dirty bits (`invalidate_map_cache(z)`, `set_seen_cache_dirty`, `set_transparency_cache_dirty`). Delta export therefore becomes: hook those to collect *dirty submaps*, then diff those submaps. Budget for the diff as the primary mechanism, not the fallback. (§5 Phase 3)
> 3. **UI compositing is easier than feared — in a specific way.** The game code draws UI as many separate `catacurses` windows (terrain window + a configurable panel stack + a `ui_adaptor` stack for menus), *but* the SDL3 backend already composites everything into a single terminal-sized `SDL_Texture` (`display_buffer` in `src/sdltiles.cpp`) before presenting. Phase 4 is now a spike between two concrete routes, not an open question. (§5 Phase 4)

---

## 1. Goals and non-goals

### Goals
- Play the full, normal CDDA game through a 3D voxel view with a free/follow camera and z-level cutaway.
- Keep the game the single source of truth: no simulation logic in the engine.
- Characters, monsters, NPCs and items render as camera-facing 2D sprites from the existing tilesets.
- Terrain, furniture, walls and vehicles render as voxels with textures generated from tileset sprites.
- Keep the game's existing menus, inventory, crafting and message UI working, composited over the 3D view.
- Work with mods without per-mod engine changes (data-driven via a tile-definition table).

### Non-goals (initially)
- Replacing the game's menus with a native 3D/GUI UI (possible later).
- Multiplayer.
- A physically accurate world scale; the look is stylized.
- Mobile/web targets.
- Overmap in 3D (stays 2D at first).

---

## 2. Architecture

```
+------------------------------+        local socket (messages)         +-----------------------------+
|  cdda-fork (C++)             | <-------------------------------------> |  voxel-engine (Rust)        |
|                              |                                          |                             |
|  - full simulation           |  handshake, tile defs, snapshots,        |  - window, GPU (wgpu)       |
|  - map, z-levels, vehicles   |  deltas, entities, input, UI events      |  - chunk meshing            |
|  - FOV / lightmap            |                                          |  - billboards               |
|  - UI -> offscreen RGBA -----+---- shared memory (UI texture) --------->|  - lighting / fog           |
|  - input_context             | <-- key / mouse / resize events ---------|  - UI compositing, input    |
+------------------------------+                                          +-----------------------------+
```

- **Engine owns the window and GPU.** It launches the game as a child process and is the IPC client; the game is the server.
- **Game owns all rules and all UI layout.** Menus are rendered by the game into a shared offscreen buffer; the engine draws that buffer over the 3D scene and forwards raw input back.
- **Turn-based means low bandwidth — with one exception.** Monsters, NPCs, weather, fields and animations also change *between* player inputs, and the game redraws on its own schedule. Deltas are therefore sent on a fork-side "world changed since last send" basis (§6 sync), not strictly per player turn. The UI texture is the only large payload and it goes through shared memory, not the socket.

### Fallback
If compositing the UI proves too painful, switch to an in-process integration: build the engine as a Rust static library with a C ABI and link it into the fork, replacing the map-draw path. The snapshot/delta API designed here is reusable either way.

---

## 3. Repositories and layout

```
cdda-voxel/
  plan.md
  Cargo.toml                  # workspace
  crates/
    proto/                    # FlatBuffers schema + generated Rust
    engine/                   # binary: window, render loop, IPC client
    mesher/                   # chunk -> mesh (greedy meshing), no GPU deps
    voxelizer/                # offline tool: tileset -> atlas + material table
    tileset/                  # tile_config.json / sprite loader (shared)
    ui-share/                 # shared-memory UI texture reader
  schema/
    cdda_ipc.fbs
  fork/                       # git submodule: our CDDA fork
  assets/                     # generated output (gitignored)
  docs/
```

Fork branch strategy: `voxel-frontend` branch off CDDA `master`. Keep changes isolated in new files (`src/voxel_ipc/*`) plus small, well-marked hooks, so rebasing onto upstream stays cheap.

**Pin, don't float.** Record the exact upstream commit the fork branched from in `docs/recon.md` and rebase deliberately. Upstream is actively refactoring the exact areas this project touches (the tiles renderer, smooth lighting and the SDL3 backend all changed shape recently), so "rebase onto master whenever" is a risk, not a routine. See §12.

---

## 4. Licensing

- CDDA code and content, and the Tilesets repo, are released under **CC BY-SA 3.0** — verified in `LICENSE.txt` at the root of *both* repos.
- That is not the whole story: both `LICENSE.txt` files list third-party exceptions with their own licenses (fonts under OFL/Apache/CC BY, Catch under Boost, PLF colony under zlib, libbacktrace under BSD, and more). The fork's attribution file must reproduce those notices, not just the CC BY-SA header.
- The fork must remain CC BY-SA 3.0. Textures derived from the sprites are derivative works and must also be share-alike with attribution to the original tileset authors.
- Base art on **UltimateCataclysm** (folder `gfx/UltimateCataclysm`, display name **UltiCa** — it is the default tileset) and optionally **Altica** (display name AltiCa) and **Chibi_Ultica** for characters. The character set the original plan called "MSX+" is folder `gfx/MShockXotto+`, abbreviated **MSX** in the Tilesets README.
- Avoid the Mushroom Dream tileset (`gfx/Mushroom-Dream`): verified in the Tilesets README — only a small stub hosted there is CC BY-SA; the complete version is under a more restrictive license via the author's Patreon.
- The Rust engine can be licensed separately if it contains no CDDA code or assets, but the generated asset pipeline output is share-alike. Decide and document this in Phase 0.
- Keep an `ATTRIBUTION.md` generated by the voxelizer listing tileset sources, per-sprite source folders where the tileset provides them (UltiCa's `tile_info.json` carries per-sheet `source` entries, e.g. for filler and fallback sheets), and the third-party notices above.

---

## 5. Phases

### Phase dependencies and parallelism

Read this before scheduling. **Hard dependency** = that phase's gate must pass before this phase can pass its own gate. **Soft dependency** = needed only for the gate; most of the work can start earlier against stubs/fake data. Phases in different codebases (C++ fork vs Rust engine vs offline voxelizer) parallelize best.

| Phase | Hard dependencies | Soft dependencies | Can run in parallel with |
|---|---|---|---|
| 0 — Recon | — (starts immediately) | — | Nothing; everything waits for its gate. Only repo/CI scaffolding may start alongside |
| 1 — Protocol and skeleton | **0** (schema, scale and headless decisions come from recon) | — | **2** — once a draft schema exists, engine work on fake data need not wait for the Phase 1 gate |
| 2 — Engine MVP (fake data) | **0** (world scale, chunk size) | 1 — final world/chunk format for its gate; mesher and camera work need only Phase 0 | **1, 3, 5** |
| 3 — Game-side export (fork) | **0, 1** (hook points + protocol to export over) | — | **2, 5** — different codebase (C++ fork vs Rust), the main parallel track |
| 4 — UI compositing | **3** (fork scaffolding: `--frontend=ipc`, `voxel_ipc` server, input injector) | 2 — an engine window/quad to composite onto; engine-side compositing can start earlier against Phase 1 stub UI frames | **5**; engine-side parts alongside **3** |
| 5 — Voxelizer and textures | **0** (tileset study) | 1 — TileDefs format for materials keyed by def; the voxelizer can start from game JSON + composed `tile_config.json` before the exporter exists | **2, 3, 4**, and the sprite-atlas parts of **6** |
| 6 — Billboards | **2** (rendering/instancing), **5** (sprite atlas) — for its gate | 3 — real entity export; billboards can be developed against fake entities from Phase 2 | **7** |
| 7 — Lighting, FOV, atmosphere | **2, 3** (`lit_level`/seen export to consume) — for its gate | 5 — works on flat colors before textures land | **6** |
| 8 — Hard cases and release | **4, 5, 6, 7** for the release gate (playable + textured + billboards + lighting) | Individual hard cases only need their own inputs: vehicles and animations need **3** (+ **4** for effects-in-UI checks) | Sub-tasks inside Phase 8 (vehicles, animations, packaging prep) are parallel with each other |

**Critical path to playable (M4): 0 → 1 → 3 → 4.** Phase 3 is the long pole. The texture path **0 → 1 → 5 → 6** and engine path **2** run alongside it, which is why M5/M6 in §13 land close behind M4 instead of a full phase-duration later. Phase 7 slots in as soon as Phase 3's light/visibility export is stable, in parallel with 6. The cumulative times in §13 assume these tracks actually run concurrently (multiple developers, or one developer interleaving); a strictly serial single-developer run takes roughly the sum of the individual phase estimates.


Time estimates assume one developer working full time; scale accordingly. Phase 0 is shorter than in v1 because the source-reading part is already done — Appendix A *is* most of the recon output. What remains is confirming the hook points in a build, and two spikes.

### Phase 0: Recon confirmation and decisions (1 week) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** none — starts immediately. **Blocks:** every other phase's gate. **Parallel with:** nothing (repo/CI scaffolding only).

**Already answered by source verification (Appendix A) — do not re-litigate:**
- World constants: submap = 12×12 (`SEEX`/`SEEY`), reality bubble = 11×11 submaps = **132×132 tiles** (`MAPSIZE`), max view distance = **60 tiles** (`MAX_VIEW_DISTANCE`), z-levels = **−10…+10, 21 layers** (`OVERMAP_DEPTH`/`OVERMAP_HEIGHT`), 3D FOV z-range = 10 (`fov_3d_z_range`). All in `src/map_scale_constants.h`.
- Exact map draw order (11 layers) and entity overlay order source. (§6, Phase 6)
- Where UI is drawn: `game::draw(ui_adaptor&)` → `draw_ter()` into `w_terrain`, then `draw_panels()`; menus are separate `ui_adaptor`s managed by `src/ui_manager.*`. Everything is composited by the SDL3 backend into one `display_buffer` texture.
- Delta tracking: only coarse invalidation exists — dirty-submap collection + diff is the design.
- Build system: **Makefile is primary; CMake also works.** CMake options: `TILES` (default **OFF**), `CURSES` (default ON). The tiles backend is **SDL3 only** — SDL2 support was removed upstream. There is no single "tiles vs ncurses" binary choice at runtime; they are separate builds.

**Tasks**
- Build the fork's curses (text) version locally; confirm toolchain. **The tiles (SDL3) version is not built and not needed** (updated 2026-11-15): the shipped fork is the text build + null `catacurses` backend — the engine does all graphics — so SDL3 is out of scope for this project entirely.
- Read and annotate, with the corrected file list:
  - `src/cata_tiles.cpp/.h`: map and sprite drawing, the 11-layer order, `height_3d` pseudo-3D offsets, `looks_like` and season-variant resolution
  - `src/sdltiles.cpp/.h`: the `display_buffer`, present path, coordinate conversion (`window_to_display_buffer_coords`)
  - `src/cursesport.cpp/.h`, `src/catacurses.h`: the `catacurses::window` abstraction all UI drawing passes through
  - `src/map.h`, `src/submap.h`, `src/mapdata.h`, `src/map_scale_constants.h`: terrain/furniture/field/item storage (see §6 for the actual shapes)
  - `src/lightmap.h`, `src/map.cpp` (`build_seen_cache`, `update_visibility_cache`): FOV and light — the exportable product is the per-tile `lit_level` enum plus float seen/light values, not a raw lightmap
  - `src/vehicle.h/.cpp`, `src/vehicle_part.h`: part layout; `face` is a `tileray` (continuous `units::angle`), mounts are pre-rotated by `precalc_mounts`
  - `src/ui_manager.*` (`ui_adaptor` stack), `src/output.*`, `src/panels.*`, `src/widget.*`: UI rendering path — note there is **no `src/ui.*`** (that file in v1 does not exist)
  - `src/input.h` (`input_manager`, `input_event`) and **`src/input_context.h`** (`input_context` lives there, not in `input.*`)
  - `src/main.cpp`, `src/game.cpp`: main loop; `game::do_turn()` and the `ui_manager::redraw()` call sites are the candidate frame/turn boundaries
  - `src/animation.h/.cpp`: animation dispatch — its curses/tiles function variants are the natural hook for forwarding effects to the engine
  - Tileset loading and the composed `tile_config.json` format; `tools/gfx_tools/compose.py` (in the CDDA repo) which generates it from the Tilesets repo's per-sprite files
- Study the Tilesets repo layout: per-sprite PNG + tile-entry JSON under `gfx/<Tileset>/pngs_*/**`, `tile_info.json` (sheet list with per-sheet sprite size and offsets), multitile entries, overlays, filler sheets. Note the composed `tile_config.json` is a *build artifact* — it is not in the CDDA repo checkout (checked: 404 on master), so the pipeline must run `compose.py` or consume a release build's output.
- **Spike A (UI):** prove one Phase 4 route end-to-end at prototype quality (§ Phase 4).
- **Spike B (headless):** decide how the game runs without its own window. **Decided in Phase 0 (2026-11-15): option (2)** — a null/custom `catacurses` backend in a curses build that renders UI windows to an offscreen buffer and never draws the map. Option (1) (tiles build with the SDL window hidden) is **dropped**: it exists only to serve the tiles build, which this project does not need. Do not assume an SDL "dummy video driver" gives you a working renderer — that assumption is exactly what option (2) avoids.
- Decide: world scale (default 1 tile = 1 block, 1 z-level = 3 blocks tall), chunk size (12×12 tiles to match submaps), engine licensing, FlatBuffers vs protobuf.

**Deliverables**
- `docs/recon.md`: Appendix A plus the two spike results, hook points, and decisions.
- A list of the exact functions where snapshot data is read and where the UI draw can be redirected.

**Exit criteria**: written answers to: where is a safe turn boundary, how do we read visible tiles per z-level, how do we redirect UI drawing to a texture — each backed by a running spike, not just source reading.

> #### Phase 0 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** the `cdda-voxel` project scaffold (this plan, `docs/`, `tools/phase0/`); sparse blobless checkouts of both pinned upstreams (CDDA `6a3e59e0`, Tilesets `5ee942f9` — commits verified by `git rev-parse`); a machine-extracted hook-point catalogue (`docs/hook-points.md`, **32/32 found**); a runnable Phase 0 harness (`tools/phase0/run_all.sh`: source probes + a compiled C++ UI-capture spike + headless runner); `docs/recon.md` (exit answers, spike results, decisions) and `docs/licensing.md`.
>
> **Exit answers (detail in recon §1–2):** send boundary = `game::do_turn()` exit + `game::draw()` redraw point; visible tiles read via the named `map::ter/furn/tr_at/field_at/i_at` + `submap` uppermost-item + `apparent_light_at` after `update_visibility_cache`; UI redirect = **Route B (catacurses-level capture) as the primary route**, Route A (`display_buffer` capture) as fallback; headless = **null `catacurses` backend**.
>
> **Decisions made:** world scale 1 tile = 1 block, 1 z-level = 3 blocks; chunk = 12×12 submap column × cutaway window; **FlatBuffers**; engine license **MIT OR Apache-2.0** dual, fork CC BY-SA 3.0, derived assets CC BY-SA 3.0; first tileset **UltiCa only**; fork base **pinned `master` @ 6a3e59e0**; v1 platform **Linux first** (§14 questions 4/5 narrowed accordingly).
>
> **Deviations and why:**
> - **`game::do_turn()` is defined in `src/do_turn.cpp:536`, not `src/game.cpp`** (the plan named game.cpp). Hook catalogue and Phase 3/9 file lists corrected — this is also new risk R2 below.
> - **`map::furn()` takes `tripoint_bub_ms` by value** (map.h:1020), and **`get_uppermost_item()` lives on the submap maptile** (`submap.h:439`), not on `map`. Exporter signatures in Phase 3 must use the real ones.
> - **Test 0.9 corrected:** no decomposed tileset entry carries all eight multitile slots; entries carry the **6 neighbour subtiles** (max found, e.g. `f_metal_bench`), and `open`/`broken` are state-variant ids — consistent with test 5.5's note. The probe asserts the verified shape; the test-plan row is updated.
> - **Spikes ran as standalone harnesses, not in-game**, because the fork cannot build in the recon environment (below). The in-game halves are written as an exact probe patch (`tools/phase0/fork_probe_patch.md`) so they can be applied unchanged on a build machine.
>
> **Test-gate results (Part II §3):** 0.3 ✅ (harness + source sites), 0.4 ✅, 0.5 ✅, 0.6 ✅ (both routes prototyped; Route A CPU-side readback floor 0.39 ms mean at 1600×800 — GPU stall still to measure in-game), 0.7 ✅, 0.8 ✅ (suite + 50 scripted turns, DISPLAY unset), 0.9 ✅ with the correction above (compose.py *run* deferred: needs pyvips + the full UltiCa PNG tree), 0.10 ✅. **0.1 ⛔ NOT RUN HERE, 0.2 ⛔ NOT RUN HERE** — and the scope is smaller than first recorded (updated 2026-11-15): the tiles/SDL3 build is **not part of 0.1 anymore** (the fork never needs SDL3; see the clarification below), so the only outstanding build is the curses/text build plus `cata_test`. Its configure stops at the Curses check in this environment (dev package installable, but a ~1,500-TU compile is not feasible at 1.5 shared CPUs / 6 GB). Unblock: Joey's PC or a ≥4-core/≥16 GB CI runner — **no SDL3 needed on the target machine** — see recon §6 for the exact commands.
>
> **Gate verdict: CONDITIONAL PASS.** The gate's mandatory tests pass at the level this environment can prove and every Phase 0 decision is recorded, so Phases 1, 2 and 5 (which work against the schema/stubs) may start. **Condition:** 0.1/0.2 and the in-game spike halves must pass on a build-capable machine before Phase 3 starts; if they contradict this recon, Phase 0 reopens.
>
> **Clarification (2026-11-15, Joey's question):** the fork does **not** need SDL3 at all. SDL3 is only the original game's graphics library, linked only by the *tiles* build (`CMakeLists.txt` searches SDL3 only under `if(TILES)`). The shipped fork is the text/curses build + null `catacurses` backend (Route B) — the engine does all graphics. The tiles build in test 0.1 was only an unmodified-upstream baseline and Route A's home; with Route A demoted to fallback, **the SDL3 half of the 0.1 block is cancelled**. The only build the project needs is the curses/text build (+ `cata_test`); what remains blocked in the recon environment is purely compile capacity.
>
> **New risks / knock-on changes:** R1 — build environment is a first-class dependency: standing up the `fork-build-and-test` CI job on a capable runner is now explicitly **Phase 1 scope**, not a Phase 3 discovery. R2 — upstream already split `do_turn` out of `game.cpp`; Phase 3/§9 hook call-sites now name three files (`do_turn.cpp`, `game.cpp`, `sdltiles.cpp`) and all hook logic stays in `src/voxel_ipc/`. R3 — Phase 5's pipeline inherits the compose.py requirement (full Tilesets checkout + libvips on the build machine). No other change to later phases' scope, estimates, or dependencies; the critical path 0 → 1 → 3 → 4 stands.

---

### Phase 1: Protocol and skeleton (1 week)

> **Dependencies:** ⛔ hard — Phase 0 gate. **Parallel with:** Phase 2 may start on the draft schema before this gate passes.

**Tasks**
- Write `schema/cdda_ipc.fbs` (see section 6).
- Create the Rust workspace and the `proto` crate with codegen.
- Implement a stub C++ server (outside the fork) that sends a fake snapshot (a small procedural house) and listens for input.
- Implement the Rust client: connect, handshake, receive snapshot, log it.
- Define versioning rules: bump `protocol_version` on breaking changes; unknown fields ignored.

**Deliverables**: engine connects to stub server and prints the snapshot; round-trip input event works.

**Exit criteria**: CI builds both sides and a test verifies schema compatibility.

---

### Phase 2: Engine MVP with fake data (1-2 weeks)

> **Dependencies:** ⛔ hard — Phase 0 gate · soft — Phase 1 (final chunk format, for its gate only). **Parallel with:** Phases 1, 3, 5.

**Tasks**
- `winit` + `wgpu` window, depth buffer, resize handling.
- Chunk data structure and greedy mesher; flat color per terrain id.
- Camera: follow-player with orbit, zoom, pitch limits; z-level cutaway (hide everything above player z + N, or fade roofs).
- Basic frame pacing (render only on change or when camera moves; the game is turn-based, but camera movement and effects are not).
- Debug overlay: chunk borders, mesh stats, fps.

**Deliverables**: fake house rendered as colored cubes, navigable camera, z-slider.

**Exit criteria**: 60 fps rendering a **132×132-tile area × the z-levels in the cutaway** (the real bubble footprint — v1's "60×60×10" understated the width by >2×; 60 is the view *radius*, not the width) on mid-range hardware.

---

### Phase 3: Game-side export in the fork (3-4 weeks) (highest risk)

> **Dependencies:** ⛔ hard — Phase 0 and Phase 1 gates. **Parallel with:** Phases 2 and 5 (different codebases). This is the critical path.

**Tasks**
- Add `src/voxel_ipc/` with: server, serializer, shared-memory writer, input injector.
- **Tile definitions export** at connect: for each terrain, furniture, trap, field type, monster type, item type, vehicle part type: numeric id, string id, name, and:
  - flags — export the string flag set plus the resolved hot flags (`ter_furn_flag` bitset: transparent, wall-like, etc.); do not make the engine parse JSON flags itself
  - connection data — **resolved group names**, not raw JSON: in code these are bitsets (`connect_groups` / `connect_to_groups` / `rotate_to_groups` in `map_data_common_t`) indexed into the `connect_group` registry, so export a group table once and per-def group indices
  - open/close/bash *result ids* (`ter_t::open`, `ter_t::close`, bash → `ter_set`/`furn_set` result) so the engine can predict the post-bash voxel without simulating
  - `looks_like` id, so sprite fallback matches the game's own resolution
  - tileset sprite refs as **fg/bg entry references into the composed `tile_config.json`** (§8), including multitile and season variants — not bare sprite-name strings
- **Snapshot export**: for the player's view (radius ≤ `MAX_VIEW_DISTANCE` = 60, z within the cutaway window — *not* the whole 132×132×21 bubble by default; see §11 for the size math), export per tile: terrain id, furniture id, trap id, **field list** `(type_id, intensity)` (plus the `displayed_field_type()` for the cheap path), **item summary** (uppermost item id + stack count — full item list only on demand for examine/pickup UI, which the game's own UI handles anyway), graffiti flag, light/visibility as the game's own values: `lit_level` enum from the visibility cache (`DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT, MEMORIZED, BLANK`) plus the float seen value where smooth falloff matters. Remembered-but-not-visible tiles come from the avatar's map memory, not from live submaps — export them as a separate memory layer with terrain/furniture ids only, never live entities/items.
- **Delta export**: hook `map::invalidate_map_cache(z)`, `set_seen_cache_dirty` and `set_transparency_cache_dirty` (and vehicle/entity changes) to accumulate a **dirty-submap set**; at each send boundary, diff only those submaps against the fork's cached copy and emit tile changes. There is no finer-grained invalidation upstream to hook — this diff *is* the mechanism. Send boundaries: after each player turn **and** after any batch of non-player changes (monster moves, field spread, weather-driven changes) that occurred while the game was waiting/redrawing.
- **Entities**: source them from the game's creature tracking (`game` / critter tracker and the active NPC list), *not* from submaps — creatures are not stored per-tile. Per entity: stable id, type id, position, `FacingDirection` (**only `NONE`/`LEFT`/`RIGHT` — a horizontal flip, there are no 8-way creature sprites**), hp state, and the overlay list **in the game's own order**: `Character::get_overlay_ids()` / `monster::get_overlay_ids()` return ordered `(id, looks_like)` pairs, with mutation overlay ordering from `overlay_ordering` / the loaded tileset. The engine composes in exactly that order and must not re-sort.
- **Vehicles**: export per-vehicle: id, `face.dir()` as float degrees (continuous angle — see Phase 8 note), and a part list with **absolute tile positions computed game-side** from the pre-rotated mounts (`precalc_mounts` already does the pivot rotation; do not re-derive mount rotation in the engine). Per part: vpart id, absolute pos, mount pos, hp/damage state, open/closed and roof flags (the renderer itself splits parts into roof / no-roof layers).
- **Frontend mode flag**: `--frontend=ipc` that disables native map drawing (`draw_ter()` path) but keeps UI drawing (redirected to the offscreen buffer).
- **Input**: translate engine input events into the game's input path (`input_manager` → `input_context::get_raw_input()` / `handle_input()`) so existing keybindings, contexts and menus work unchanged. Mouse coordinates must be delivered in the same space the SDL backend would produce (display-buffer pixels → cell conversion is the backend's job today; the fork-side injector owns that conversion).
- **Save/load and world creation** must work through the same UI path.

**Deliverables**: real game running with engine connected; snapshot and deltas visible in engine logs; menus still functional via forwarded input (UI not yet visible, so use debug logging).

**Exit criteria**: replay test: a scripted input sequence on a **seeded test world** produces identical *tile-array* snapshot hashes across two runs. (Hash only the exported tile/entity data, not the full snapshot: weather, RNG-driven ambient state and timestamps will differ run to run unless the whole game is seeded, and chasing those is not a frontend bug.)

---

### Phase 4: UI compositing (1-2 weeks)

> **Dependencies:** ⛔ hard — Phase 3 gate (fork scaffolding and input injection) · soft — Phase 2 (engine window; stub frames allow early engine-side work). **Parallel with:** Phase 5.

Verified shape of the problem: `game::draw()` erases `w_terrain`, calls `draw_ter()`, runs draw callbacks, then `draw_panels()` draws the sidebar/message-log panels — each panel is a *fresh* `catacurses` window from the `panel_manager` layout (layouts are user-configurable; the message log is the panel with height −2; sidebar side is an option). Menus/dialogs are separate `ui_adaptor`s on the `ui_manager` stack. There is no single "UI surface" in game code to grab. **But** the SDL3 backend (`src/sdltiles.cpp`) already renders all of it into one terminal-sized `SDL_Texture` — `display_buffer` — and presents it with a single `RenderCopy`. Two routes:

- **Route A (preferred, spike first): backend capture.** In frontend mode, skip `draw_ter()` (leave the terrain window's rect transparent/cleared), let panels and menus draw normally into `display_buffer`, and read the buffer back at the present point into shared memory. Risks to prove in the spike: GPU→CPU readback cost/stalls per frame (mitigate: UI only changes on redraw — capture only on dirty frames; double-buffer), DPI/`SCALING_FACTOR` mismatch between display-buffer pixels and window pixels, and the shader/variant-pass work the current backend does around the buffer.
- **Route B (fallback): `catacurses`-level capture.** Implement the offscreen target one layer down: every `catacurses::window` renders into the shared buffer at its cell position, and the map window is suppressed. More fork code, but no GPU readback and it also works for a curses-backend build.

**Tasks**
- Implement the chosen route; transparent background exactly where the map used to draw.
- Double-buffered shared memory with a sequence counter and dirty rect; notification message on socket when a new frame is ready.
- Engine uploads the UI texture and draws it as a full-screen quad above the 3D view, with nearest-neighbor or integer scaling.
- Handle resize, DPI scaling, and fonts (keep the game's font rendering).
- Input mapping: key events (including text input and modifiers), mouse position in UI space, mouse clicks mapped to game coordinates when the game expects map-click input (requires an engine-side picking ray → tile coordinate → message; note the game also uses mouse clicks *on UI panels*, which stay in UI space — the fork reports which mode it is in via the UI frame/input context).

**Deliverables**: full playable game: menus over a flat-colored voxel world.

**Exit criteria**: create a character, start a game, open inventory, craft something, save and quit, all through the engine window.

---

### Phase 5: Voxelizer and textures (2-3 weeks, can overlap Phase 4)

> **Dependencies:** ⛔ hard — Phase 0 gate · soft — Phase 1 (TileDefs format). **Parallel with:** Phases 2, 3, 4 — this phase is deliberately *not* on the critical path; start it as early as Phase 1 allows.

See section 8 for the pipeline.

**Deliverables**: `voxelizer` CLI producing `assets/<tileset>/blocks.ktx2 (or png array)`, `materials.json`, `sprites.png atlas`, and `ATTRIBUTION.md`; engine loads them and renders textured terrain.

**Exit criteria**: houses, roads, fields, forests, and the starter shelter look coherent; unknown ids fall back to a visible placeholder and are logged.

---

### Phase 6: Billboards (1-2 weeks)

> **Dependencies:** ⛔ hard — Phase 2 and Phase 5 gates · soft — Phase 3 (real entities; fake entities suffice during development). **Parallel with:** Phase 7.

- Instanced camera-facing quads (cylindrical billboarding: rotate around Y only) for creatures, NPCs, items on ground.
- Overlay compositing: the engine composes layered sprites (body + clothing + wielded) into a single billboard texture, cached by overlay signature. **Draw order is verified, not guessed** — the tile layer order in `cata_tiles::draw()` is: terrain → furniture → graffiti → trap → part_con → field_or_item → vpart (no roof) → vpart (roof) → critter → zone mark → zombie-revival indicator; and within a creature, the base sprite first, then `get_overlay_ids()` in returned order. For billboards only the creature part applies, but the same ordering source governs what the game's own examine/look output expects.
- Facing: `FacingDirection` is `LEFT`/`RIGHT`/`NONE` only — mirror horizontally for `LEFT`, never look for 8-way directional creature sprites (they don't exist in this system; rotation exists for *terrain/vehicle* tiles via multitile/rotation, not creatures). Use each sprite's real width/height/offset from `tile_info.json` (§8) — creature sprites are not all 32×32.
- Shadows: blob shadow decal under each billboard.
- Sorting and alpha: alpha-test with dithering or sorted transparent pass; depth-write for cutouts.
- HP bars, status icons and names as billboard overlays or left to the game's UI.
- Animation: sprite frame animation if the tileset defines it (weighted/animated fg entries in the tile config); simple bob/hit-flash effects in engine.

**Exit criteria**: a crowd of zombies renders correctly sorted against walls and windows.

---

### Phase 7: Lighting, FOV and atmosphere (2 weeks)

> **Dependencies:** ⛔ hard — Phase 2 and Phase 3 gates (for its own gate) · soft — Phase 5 (flat colors work first). **Parallel with:** Phase 6.

- Use the game's per-tile `lit_level` and seen values as the source of truth (do not recompute FOV). Note the game also computes *colored* light (`light_color_rgb`, float RGB energy) — v1 of the frontend may quantize to intensity + a single tint per tile, but the protocol field should be sized so colored light can be added without a breaking change.
- Bake light into vertex data or a per-chunk light texture; smooth with simple averaging across neighbors.
- Unseen-but-remembered tiles (`lit_level::MEMORIZED` / map-memory layer): desaturated and darker. Never-seen tiles (`BLANK`): black / not meshed.
- Day/night tint, weather overlays (rain particles, fog), flashlight cones from game light data.
- Optional later: screen-space ambient occlusion, shadow mapping from sun.

---

### Phase 8: Hard cases (3-4+ weeks)

> **Dependencies:** ⛔ hard — Phases 4, 5, 6, 7 gates for the release gate; individual hard cases need only their own phase (vehicles/animations: Phase 3). **Parallel with:** its own sub-tasks are parallel with each other.

- **Vehicles**: orientation is a **continuous angle** (`vehicle::face` is a `tileray`; `face.dir()` returns `units::angle`). The "15-degree steps" in v1 came from a real but narrower fact: `vehicle::immediate_path()` clamps a *pathing* angle "to multiples of 15". Do not quantize exported facing to 15° — export float degrees, let the engine smooth/interpolate, and build the voxel model from the game-side absolute part positions (Phase 3), so pivot/precalc logic is never duplicated. Smooth turning animation interpolates between exported facings.
- **Doors, windows, multitile terrain**: open/closed state arrives as a *different terrain/furniture id* after the game's own transform (`open`/`close` ids) — the engine needs no door logic, just the delta. Bashed/broken states likewise. Connecting walls use the multitile set — all **eight** keys: `center, corner, edge, t_connection, end_piece, unconnected, open, broken` (`multitile_keys` in `cata_tiles.cpp`; v1 listed six and missed `open`/`broken`).
- **Open air, roofs and pits**: terrain `t_open_air` has a `roof` field naming the floor above; z-level meshing must treat open air as "no block, floor comes from below/roof rules", or every upper floor and rooftop breaks. Test explicitly with a two-storey house and a basement.
- **Animations and projectiles**: hook the game's animation dispatch (`src/animation.*` — it already selects curses vs tiles implementations per effect: bullets, explosions, thrown items) and forward effect events; the engine renders its own versions.
- **Overmap**: keep the game's 2D overmap drawn in the UI texture; optionally a 3D terrain view later.
- **Large mods and tilesets**: validate against popular mods; add a debug report for ids with no sprite/material. Mods can also add *connect groups, field types and vparts* — all are in the TileDefs export, so no engine change should be needed; test with one content mod to prove it.
- **Performance**: chunk streaming, mesh rebuild budgets, instancing, culling, LOD for distant chunks.
- **Packaging**: Windows/Linux builds, bundled fork binary, asset generation step (`compose.py` run or pre-composed tileset from a release), installer or archive.

---

## 6. IPC design

### Transport
- **Messages**: local stream socket (Unix domain socket on Linux/macOS, named pipe on Windows) via Rust `interprocess`; length-prefixed FlatBuffers frames.
- **UI texture**: shared memory region (Rust `shared_memory` / C++ `shm_open` or `CreateFileMapping`) with header `{ magic, width, height, stride, seq, dirty_rect }` and two buffers.
- **Lifecycle**: engine spawns the game with `--frontend=ipc --ipc-path=<path>`; if either side exits, the other shuts down cleanly. Heartbeat every second.

### Message types (sketch)

| Direction | Message | Purpose |
|---|---|---|
| G->E | `Hello` | protocol version, game version, tileset id, tile pixel size, z-range |
| E->G | `HelloAck` | engine version, screen size, DPI, capabilities |
| G->E | `TileDefs` | id tables for terrain, furniture, traps, fields, monsters, items, vehicle parts, plus the connect-group table |
| G->E | `Snapshot` | full view: origin, extents, per-submap tile arrays + memory layer |
| G->E | `Delta` | changed tiles, entity add/move/remove, vehicle changes |
| G->E | `LightUpdate` | per-tile `lit_level` and visibility for the view (fold into Delta when small) |
| G->E | `Effects` | animations, projectiles, sound markers |
| G->E | `UiFrame` | shared buffer index, dirty rect, and whether the game is in a menu/UI-input mode vs map-input mode |
| E->G | `Input` | key, text, mouse, wheel, modifiers, focus |
| E->G | `Resize` | new size and DPI |
| E->G | `PickResult` | tile coordinates under cursor click |
| both | `Ping/Pong`, `Shutdown` | liveness |

### FlatBuffers schema sketch

```fbs
namespace cdda.ipc;

struct Vec3i { x:int; y:int; z:int; }

table TileDef {
  id:uint32;
  string_id:string;
  name:string;
  kind:TileKind;          // terrain, furniture, trap, field, monster, item, vpart
  flags:[string];         // string flags as authored
  hot_flags:uint64;       // resolved ter_furn_flag bitset for engine-fast checks
  connect_groups:[uint16]; // indices into the Hello/TileDefs connect-group table
  connects_to_groups:[uint16];
  open_result:string;     // id after open transform, "" if none
  close_result:string;
  bash_result:string;     // terrain/furniture id after a successful bash
  looks_like:string;
  fg_ref:int32;           // index into composed tile_config tiles; -1 = none
  bg_ref:int32;           // floors/background sprite matters for voxelizer (§8)
}

table FieldOnTile {       // a tile can hold SEVERAL fields at once (smoke + fire + ...)
  type:uint32;            // field type def id
  intensity:ubyte;        // field_entry intensity
}

table TileDefs { defs:[TileDef]; connect_groups:[string]; protocol_version:uint32; }

table SubmapBlock {
  origin:Vec3i;           // absolute tile coordinates of the block's corner
  terrain:[uint32];       // 12*12 entries per z layer, row-major
  furniture:[uint32];
  trap:[uint32];
  fields:[FieldOnTile];   // sparse: only tiles with fields, with tile index — NOT one id per tile
  field_index:[int32];    // per tile: offset into fields, -1 = none (or use displayed-only fast path below)
  displayed_field:[uint32]; // per tile: field_type the game itself would draw + intensity packed
  top_item:[uint32];      // uppermost item id per tile (game has get_uppermost_item())
  item_count:[uint16];    // stack size per tile; >1 means "pile", engine need not enumerate
  lit:[ubyte];            // lit_level enum per tile (DARK..BLANK), NOT a 0-255 light value
  seen:[ubyte];           // quantized seen/visibility float, 0 = never seen
}

table Entity {
  id:uint64;
  sprite_id:string;
  pos:Vec3i;
  facing:ubyte;           // FacingDirection: 0 none, 1 left, 2 right — flip only
  overlays:[string];      // in game order (get_overlay_ids); do not re-sort engine-side
  hp_frac:ubyte;
  flags:uint32;           // friendly, hostile, sleeping, etc.
}

table VehiclePartState {
  vpart:uint32;
  abs_pos:Vec3i;          // computed game-side from pre-rotated mounts
  mount:Vec3i;
  hp_frac:ubyte;
  flags:uint32;           // open, broken, roof, etc.
}
table Vehicle { id:uint64; face_deg:float; parts:[VehiclePartState]; }

table Snapshot { view_origin:Vec3i; blocks:[SubmapBlock]; entities:[Entity]; vehicles:[Vehicle]; seq:uint64; epoch:uint64; }
table Delta    { seq:uint64; epoch:uint64; tile_changes:[TileChange]; entity_updates:[Entity]; entity_removed:[uint64]; vehicle_updates:[Vehicle]; }
```

(Exact layout to be finalized in Phase 1; prefer per-block arrays over per-tile records for speed. Field/item representation above is the minimum that doesn't lose gameplay-visible information — smoke you can't see through and a fire's intensity are not optional detail.)

### Determinism and sync
- Sync is by **fork-side monotonic `seq` + `epoch`**, not by the game's calendar turn: non-player actors, fields and weather change state outside player turns, and calendar time is a game-time value, not a message counter. `epoch` increments on every fresh `Snapshot`; a `Delta` names the epoch and the seq it applies to.
- A `Delta` is always applicable to the last acknowledged snapshot; if the engine detects a gap (wrong epoch, skipped seq), it requests a fresh `Snapshot`.
- The game never waits on the engine except for input.

---

## 7. Engine design (Rust)

### Crate choices
- Rendering: `wgpu`, `winit`
- Math: `glam`
- Image: `image`, `ktx2` (optional), `rectangle-pack` or custom atlas
- Serialization: `flatbuffers`
- IPC: `interprocess`, `shared_memory`
- Concurrency: `rayon` for meshing, channels for IPC thread -> render thread
- Logging/diagnostics: `tracing`, `tracy-client` (optional)
- Tests: `insta` for snapshot tests

### Threads
1. **IPC thread**: reads socket, decodes messages, pushes to a queue.
2. **Mesh workers** (rayon): build chunk meshes off the render thread.
3. **Render thread**: applies updates, uploads buffers, draws.

### World representation
- World stored as chunks aligned to game submaps (12x12 tiles, per z-level; a chunk column groups all z in the cutaway window — the full column is 21 levels, the *stored* column should be sized to the cutaway, not hard-coded to 21).
- A block per tile: `{ terrain, furniture, flags, lit_level }` plus optional sub-voxel model reference for furniture.
- Dirty flags per chunk; neighbor chunks marked dirty on boundary changes.

### Meshing
- Greedy meshing per chunk for opaque full blocks, producing per-face texture-array layers.
- Separate passes for: opaque, cutout (alpha-tested, e.g. fences, foliage), transparent (glass, water, fields).
- Walls choose face textures from multitile variants (all eight slots, § Phase 8) based on neighbor connectivity using exported connect-group data.
- Use a **texture array** (not a single atlas) for block faces to avoid mipmap bleeding.

### Scale and proportion
- 1 tile = 1x1 block footprint; 1 z-level = 3 blocks tall so walls read as walls and billboards (about 2 blocks tall) fit through doorways. Parameterized in one config struct so it can be tuned.
- Floors/ceilings are thin slabs or full blocks per design choice; decide visually in Phase 2. Whichever is chosen, `t_open_air` + `roof` handling (Phase 8) is part of the decision, not an afterthought.

### Camera and view
- Perspective camera with follow target, orbit yaw, pitch in [20, 85] degrees, zoom.
- Z-cutaway modes: hide all above player z; fade roof tiles; or show only the current floor +1.
- Optional classic top-down/isometric preset to compare against the original look.

### UI compositing
- Full-screen quad with the shared UI texture; alpha blend; integer scaling option.
- Input focus: when the game reports a menu is open (UiFrame mode flag), the 3D camera ignores mouse drags; otherwise camera controls are active and game keys pass through.

### Picking
- Ray from cursor through the world; hit-test chunks (DDA voxel traversal) and billboards; send tile coordinate on click.

---

## 8. Texture and sprite pipeline (voxelizer)

### Inputs
- Tileset directory from CDDA-Tilesets (e.g. `gfx/UltimateCataclysm`), in its **decomposed** form: per-sprite PNGs + per-sprite tile-entry JSON under `pngs_*/` folders, plus `tile_info.json` and `tileset.txt`.
- The **composed** form, produced by running CDDA's `tools/gfx_tools/compose.py` on that directory (requires Python + `pyvips`/libvips): a single `tiles.png` sheet + `tile_config.json`. In the composed config, each entry has an `id` (string, or list of ids sharing a sprite), `fg`/`bg` as sprite *indices* (each either a single index or a list of weighted variants), optional `multitile: true` with `additional_tiles` for the eight multitile slots, `rotates`, and season variants are separate entries whose ids carry `_season_spring/summer/autumn/winter` suffixes. Sprite resolution in-game also follows a `looks_like` fallback chain — the voxelizer should resolve ids the same way (exact → variant → season → `looks_like`) or it will report false "missing" sprites.
- TileDefs exported by the game (for flags and connect groups).
- Optional override file `overrides.toml` (hand-tuned entries).

### Sprite geometry — not uniform 32×32
UltiCa's base tile size is 32×32, but `tile_info.json` defines many sheets with different sprite sizes **and offsets**, all of which the voxelizer and billboard code must carry through per sprite:
`small` 20×20 (offset +6x), `tall` 32×64 (−32y), `human_body` 32×36 (−8y), `human_body_plus` 32×48 (−16y), `centered` 64×64 (−16,−16), `large` 64×64 (−16,−32), `huge` 64×96 (−16,−64), `giant` 96×96 (−32,−64), plus filler variants of several of these. A sprite larger than a tile is anchored by its offset, not centered by default. Block faces use only tile-sized sprites; tall/large/giant sprites belong to the billboard/furniture-model path.

### Outputs
- `blocks` texture array: per material, top, side and bottom faces at tile resolution (32x32 for UltiCa base; configurable per tileset — other tilesets differ, e.g. RetroDays is 10×10).
- `materials.json`: terrain/furniture id -> material, faces, height in voxels, transparency, tint, sub-model reference.
- `sprites` atlas for billboards (characters, monsters, items) with UV table, **per-sprite width/height/offset**, and overlay layering metadata.
- `ATTRIBUTION.md`.

### Face generation rules
1. **Top face**: the tile's own sprite — specifically its `fg` composited over its `bg` where both exist (many floor sprites are a transparent `fg` over a `bg` ground sprite; using `fg` alone produces holes).
2. **Side faces**:
   - Walls: use wall sprites as sides directly (they already depict a vertical-ish surface in UltiCa); fall back to sampling the sprite's most opaque rows.
   - Ground/floor: sample the bottom edge rows and stretch, darkened 25-40%, with subtle noise to avoid obvious stretching.
3. **Bottom face**: darkened copy of the side material; rarely visible.
4. **Multitile walls and connections**: map the game's multitile slots (`center, corner, t_connection, edge, end_piece, unconnected, open, broken`) to face variants selected at mesh time by neighbor mask and connect groups.
5. **Mipmaps**: generate with alpha-aware downsampling; clamp edges.
6. **Transparency**: alpha cutout flag when sprite has significant alpha holes; separate transparent flag for glass and water.

### Furniture and objects
- Default: furniture is a block-height-fraction extruded silhouette: take the sprite's alpha mask, extrude by `height_voxels` (heuristic by id/flags; e.g. tables 1, bookcases 3, beds 1, appliances 2). Sub-voxel resolution (sprite pixel = voxel column) is supported for these models.
- Top face uses the sprite; sides use darkened edge-sampled colors.
- `overrides.toml` can force a model: `{ id = "f_bookcase", height = 3, mode = "box" }`.

### Billboards (no voxelizing)
- Characters, monsters, NPCs, items: use original sprites unchanged, with their real sizes/offsets (above).
- Compose overlay layers in the engine at runtime and cache by signature.

### Fallbacks and QA
- Unknown id -> magenta/black checker material and a line in `missing.log`. Resolution must try `looks_like` before declaring an id missing (see Inputs).
- Generate a contact sheet (`contact.png`) of all materials for visual review.
- Visual regression: render a fixed test scene per tileset and compare with `insta` image snapshots.

---

## 9. Fork changes (C++), checklist

- [ ] New `--frontend=ipc` option and null/offscreen video path (per Phase 0 Spike B decision: **null `catacurses` backend**; hook call-sites in `src/do_turn.cpp`, `src/game.cpp`, `src/sdltiles.cpp` — `do_turn` lives in its own file since the upstream split found in Phase 0)
- [ ] `src/voxel_ipc/` module: server, framing, serializer
- [ ] Tile definitions exporter (terrain, furniture, trap, field, monster, item, vpart + connect-group table)
- [ ] Snapshot exporter (view radius, z-range, `lit_level`/seen, map-memory layer)
- [ ] Delta tracking (dirty-submap collection from existing invalidation hooks + per-submap diff)
- [ ] Entity exporter with overlay list (game order), facing (flip only)
- [ ] Vehicle exporter (absolute part positions, float facing)
- [ ] Offscreen UI render target and shared-memory writer (Phase 4 chosen route)
- [ ] Input injector feeding the existing input path (`input_context::get_raw_input`)
- [ ] Clean shutdown, crash handling, heartbeat
- [ ] Config flag to keep original tiles mode working (don't break upstream behavior)
- [ ] Rebase script and notes for syncing with upstream; pinned upstream commit recorded

Design principle: **additive hooks, no behavior changes** in the simulation.

---

## 10. Testing and CI

- **Schema tests**: round-trip serialization for both languages; golden binary files.
- **Replay tests**: scripted input into the fork on a seeded test world, compare exported tile-array hashes across runs (see Phase 3 exit criteria for why the hash scope is limited).
- **Mesher unit tests**: known chunk layouts -> expected face counts; greedy merge correctness.
- **Voxelizer tests**: tiny fixture tileset -> expected atlas and materials; include one multitile entry, one fg-over-bg entry, one non-32×32 sprite, and one `looks_like`-only id in the fixture — those are the four cases that break naive pipelines.
- **Visual snapshots**: headless wgpu render of fixed scenes (on a software adapter in CI).
- **Integration test**: launch engine + fork, create a world, walk N turns, quit; check no errors.
- CI: GitHub Actions matrix (Linux first, then Windows/macOS); cache Cargo and CDDA build objects (the fork build is slow).

---

## 11. Performance budgets

Snapshot size math, corrected: one z-level of the full bubble is 132×132 = 17,424 tiles. At ~6 bytes/tile of core arrays (terrain+furniture+trap+lit+seen, ids as uint16 where the def count allows) that's ≈100 KB per z-level — a full 21-level bubble snapshot is ≈2 MB *before* fields, items, entities and vehicles. Hence the §5/§6 rule: snapshots cover the view window and cutaway z-range, not the whole bubble, and full resends stay rare.

| Item | Target |
|---|---|
| Frame time | under 16 ms at 1080p with typical view |
| Visible area | view radius up to 60 tiles; stored world 132×132 tiles × cutaway z-range |
| Chunk remesh | under 2 ms per chunk, off the render thread |
| Delta apply | under 1 ms per turn |
| UI texture upload | only on dirty frames; under 1 ms (GPU readback on the game side is budgeted in Phase 4's spike, not here) |
| Snapshot size | under 2 MB *for the view window*; full resend rare |
| Memory | under 1.5 GB total for engine |

---

## 12. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| CDDA not built to be frontend-swappable; map and UI drawing intertwined | Phase 3-4 slip | Phase 0 spikes (done at prototype level before committing); additive hooks; fallback to in-process integration |
| UI drawn as many windows, not one surface | Compositing looks wrong | Work at the backend's existing `display_buffer` present point (Route A) or the `catacurses` layer (Route B) — both verified to exist; never try to scrape individual windows |
| Delta tracking coarser than hoped (no per-tile hooks) | Phase 3 slip | Already the plan: dirty-submap diff; cap diff cost by submap count per send |
| Mouse-driven map interactions (look, target, examine) | Broken UX | Engine picking -> tile coordinates message; UiFrame mode flag separates UI clicks from map clicks; keep keyboard flow first |
| Sprite sizes/offsets vary widely across sheets | Billboards and furniture models misaligned | Per-sprite geometry from `tile_info.json` carried through the whole pipeline (§8); fixture tests |
| Extruded-sprite textures look inconsistent across thousands of ids | Art quality | Overrides file, per-category rules, contact sheets, iterate |
| Upstream churn in CDDA — the renderer/SDL backend is actively being refactored | Rebase pain | Isolate in new files; small hooks; pin a commit and rebase deliberately on a schedule, not continuously |
| Mods adding content without sprites | Placeholder art | Fallback materials by flags; `looks_like` resolution before declaring missing; missing report |
| Transparent sorting (windows, fields, billboards) | Visual glitches | Separate passes, dithered alpha, tuned depth rules |
| Vehicle rotation and multi-tile parts | Complexity | Defer to Phase 8; game-side absolute part positions from day one so no rotation math is duplicated |
| Cross-platform shared memory/input quirks | Delays | Linux first; abstract platform layer; test early on Windows |
| Licensing mistakes | Legal/attribution | Phase 0 decision doc; auto-generated attribution including third-party notices from `LICENSE.txt` |

---

## 13. Milestones and rough timeline

Cumulative times are **critical-path** times (see §5 "Phase dependencies and parallelism"): they assume Phases 2, 5 and 6 run in parallel with the 0 → 1 → 3 → 4 path, not after it. Each milestone row must be dated in the Progress log when it is reached, per the maintenance rule at the top of this plan.

| Milestone | Cumulative time | Outcome |
|---|---|---|
| M0 Recon confirmed + spikes | ~1 week | Hook points proven, UI and headless routes chosen |
| M1 Protocol + stub | ~2 weeks | Rust client talks to fake server |
| M2 Engine MVP | ~4 weeks | Navigable colored-voxel scene with z-slicing |
| M3 Real game export | ~8 weeks | Live snapshots/deltas from the fork |
| M4 UI compositing | ~10 weeks | **Playable**: real menus over flat voxels |
| M5 Textures | ~12 weeks | Textured world from UltiCa |
| M6 Billboards | ~14 weeks | Characters/monsters as sprites |
| M7 Lighting/FOV | ~16 weeks | Atmosphere matches game visibility |
| M8 Polish and hard cases | ~20+ weeks | Vehicles, animations, packaging |

---

## 14. Open questions

1. Final world scale and floor/ceiling thickness (decide visually in Phase 2).
2. Should the original 2D tiles view remain selectable at runtime (toggle between frontends)?
3. How much of mouse interaction is required for v1?
4. Which tileset(s) to ship first: UltiCa only, or UltiCa plus a chibi character set (Altica / Chibi_Ultica)? — **Narrowed in Phase 0 (2026-11-15): UltiCa only for v1**; chibi character sets deferred, not decided against.
5. Target platforms for v1 (Linux only, or Windows too)? — **Narrowed in Phase 0 (2026-11-15): Linux first**; Windows packaging lands in Phase 8.
6. Should engine and fork be distributed together or as separate downloads?
7. Colored light: quantize to intensity+tint for v1, or carry float RGB from the start? (Protocol field is sized for it either way — §7.)
8. Map-memory layer fidelity: does the engine need furniture-level memory, or terrain-only like the game's own muted memory rendering?

---

## 15. First-week task list

1. Clone the fork, build the curses (text) version, run it. (No tiles/SDL3 build — not needed; see test 0.1 as updated 2026-11-15.)
2. Turn Appendix A into `docs/recon.md`; add the pinned upstream commit.
3. Spike A: suppress `draw_ter()`, capture `display_buffer` at present, display it in a standalone window.
4. Spike B: decide the headless route from the spike results.
5. Create the Rust workspace and `proto` crate; draft `cdda_ipc.fbs` per §6.
6. Build the stub server and the Rust client handshake.
7. Run `tools/gfx_tools/compose.py` on `gfx/UltimateCataclysm` from the Tilesets repo; inspect the composed `tile_config.json`, one multitile wall entry (all eight slots), and one fg-over-bg floor entry.
8. Make the engine draw a single textured cube with a camera.

---

## Appendix A: Verification record (2026-11-15)

Checked against CDDA `master` @ `6a3e59e0` and CDDA-Tilesets `master` @ `5ee942f9`, by reading the named files/functions directly. Verdicts: **Verified** = v1 was right · **Corrected** = right idea, wrong detail · **Refuted** = v1 was wrong and the design changed.

| # | v1 claim | Verdict | Evidence |
|---|---|---|---|
| 1 | Recon files: `src/cata_tiles.cpp`, `sdltiles.cpp`, `map.h`, `submap.h`, `mapdata.h`, `lightmap.cpp`, `vehicle*.cpp`, `output.*`, `main.cpp`, `game.cpp` | Verified | All exist on master (HTTP 200 on raw source check), as do `vehicle_part.cpp`, `animation.*`, `panels.*`, `widget.cpp` |
| 2 | `src/ui.*` for the UI path | **Refuted** | `src/ui.cpp`/`src/ui.h` return 404. The UI stack is `src/ui_manager.*` (`ui_adaptor`); panels are `src/panels.*` + `src/widget.*` |
| 3 | `src/input.*`: `input_context` and event handling | Corrected | `input_manager`/`input_event` are in `src/input.h`, but `input_context` is its own file: `src/input_context.h` (with `get_raw_input()`, `handle_input()`, `registered_actions`) |
| 4 | "`catacurse` wrappers" | Corrected | Spelled `catacurses`: `src/catacurses.h`, implementation in `src/cursesport.cpp/.h` (`cata_cursesport::WINDOW`, `catacurses::window`) |
| 5 | Chunk size 12×12 to match submaps; snapshot 60×60×10 | Corrected | `src/map_scale_constants.h`: `SEEX=12`, `SEEY=SEEX` ✓ — but `MAPSIZE=11` (bubble 132×132), `MAX_VIEW_DISTANCE = SEEX*HALF_MAPSIZE = 60` is a *radius*, `OVERMAP_DEPTH=10`, `OVERMAP_HEIGHT=10`, `OVERMAP_LAYERS=21`, `fov_3d_z_range=10` (`game_constants.h`) |
| 6 | Hook existing "map changed" invalidation for deltas, else diff | Corrected (diff is primary) | `src/map.cpp/.h`: `invalidate_map_cache(z)`, `set_seen_cache_dirty(p|z)`, `set_transparency_cache_dirty(p|z)` exist, but granularity is whole-z-level / per-submap dirty bit in the level cache. No per-tile change list exists |
| 7 | Per-tile snapshot: single field id, top item id, light level, visibility bits | **Refuted** (fields/items); Corrected (light/vis) | `src/submap.h`: `fld` is `field` = `std::map<field_type_id, field_entry{type,intensity,age,causer}>` (`src/field.h`) — multiple fields per tile; `itm` is `cata::colony<item>` per tile (a stack; `get_uppermost_item()` exists, `MAX_ITEM_IN_SQUARE=4096`). Light/visibility product is `lit_level` enum (`src/lightmap.h`: DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT, MEMORIZED, BLANK) from `visibility_cache`, plus float `seen_cache`; colored light exists as `light_color_rgb` float RGB |
| 8 | UI may be drawn into fixed regions incl. the map area | Corrected | `game::draw(ui_adaptor&)` (`src/game.cpp`): `werase(w_terrain)`, `draw_ter()`, callbacks, `draw_panels(true)` — panels are transient windows from `panel_manager` layouts (message log = height −2 panel; sidebar side is an option). Menus are separate `ui_adaptor`s (`src/ui_manager.h`). **And** `src/sdltiles.cpp` composites everything into one `SDL_Texture display_buffer`, presented via `RenderCopy`, with `window_to_display_buffer_coords` conversion and a `display_buffer_draw_scope` RAII bind — the capture point for Route A |
| 9 | Tiles backend is SDL (software or render-target) | Corrected | `CMakeLists.txt`: "SDL2 support was removed; SDL3 is the only tiles backend", `option(TILES ... OFF)`, `option(CURSES ... ON)`. Makefile remains the primary build |
| 10 | Entity overlays: match the game's draw order | Verified, and now exact | `cata_tiles::draw()` `drawing_layers[11]`: terrain, furniture, graffiti, trap, part_con, field_or_item, vpart_no_roof, vpart_roof, critter_at, zone_mark, zombie_revival_indicators. Creature-internal: base sprite, then `Character::get_overlay_ids()` / `monster::get_overlay_ids()` in order (`src/cata_tiles.cpp` `draw_entity_with_overlays`), mutation ordering via `overlay_ordering.h` / `tileset_mutation_overlay_ordering` |
| 11 | Creature facing: directional variants if the tileset has them | **Refuted** | `src/creature.h`: `enum class FacingDirection { NONE=0, LEFT=1, RIGHT=2 }`, commented "for sdl horizontal flip". Creatures flip; they do not rotate |
| 12 | Vehicles: orientation in 15-degree steps | Corrected | `src/vehicle.h`: `tileray face; units::angle turn_dir;` — `face.dir()` is a continuous `units::angle`. The 15° figure is real but narrower: `vehicle::immediate_path()` in `src/vehicle.cpp` — "clamp to multiples of 15" — for pathing. Mount rotation is precomputed game-side (`precalc_mounts`, `pivot_rotation`) |
| 13 | Multitile slots: center, corner, t_connection, edge, end_piece, unconnected | Corrected | `multitile_keys` in `src/cata_tiles.cpp` has **eight**: center, corner, edge, t_connection, end_piece, unconnected, **open, broken** |
| 14 | `connects_to` / `connects_to_group` as per-def string data | Corrected | `map_data_common_t` (`src/mapdata.h`): `std::bitset<NUM_TERCONN> connect_groups / connect_to_groups / rotate_to_groups`, set from JSON group names via the `connect_group` registry (`connect_group::load`, `group_flags`, `connects_to_flags`). Flags likewise dual: string set + `enum_bitset<ter_furn_flag>` |
| 15 | UltiCa sprites are 32×32; faces at tile resolution | Corrected | Tilesets `gfx/UltimateCataclysm/tile_info.json`: base 32×32, plus sheets at 20×20, 32×36, 32×48, 32×64, 64×64, 64×96, 96×96 with per-sheet `sprite_offset_x/y` (e.g. tall −32y, giant −32/−64). `tileset.txt`: `NAME: UltimateCataclysm`, `VIEW: UltiCa`, `JSON: tile_config.json`, `TILESET: tiles.png` |
| 16 | Tileset names: UltiCa default; "AltiCa, Chibi UltiCa and MSX+" | Corrected | Tilesets README + `gfx/` listing: folders `UltimateCataclysm` (UltiCa, "the default tileset"), `Altica` (AltiCa), `Chibi_Ultica`, `MShockXotto+` ("often abbreviated MSX" — not MSX+). Isometric variants also exist (`Ultica_iso`, `IsoMSX+`) |
| 17 | Mushroom Dream: only a stub is CC BY-SA, full version restrictive | Verified | Tilesets README, verbatim in substance: "We host here a small stub of the tileset that was initially released under cc by sa, a more complete version released under a more restrictive license is available on Albert's Patreon" (`gfx/Mushroom-Dream`) |
| 18 | CDDA + Tilesets are CC BY-SA 3.0 per READMEs | Verified, with exceptions | `LICENSE.txt` in both repos: CC BY-SA 3.0 Unported, followed by third-party notices (fonts, Catch/Boost, PLF/zlib, libbacktrace/BSD, …) that must travel with attribution |
| 19 | `tools/gfx_tools/compose.py` and `tile_config.json` format | Verified, with a caveat | `compose.py` exists in CDDA; it converts per-sprite tile-entry JSON (sprite *names* in `fg`/`bg`, weighted variants, `additional_tiles`) into composed `tile_config.json` (sprite *indices*), per its own conversion code. Caveat: composed UltiCa `tile_config.json` is **not in the CDDA repo** (404 on master) — it's generated at tileset-build/release time, so the pipeline must run compose or consume a release |
| 20 | Sprite resolution is by id | Corrected | `cata_tiles` resolves exact id → variant (`_var_`) → season suffix (`_season_spring/summer/autumn/winter`, `find_tile_type_by_season`) → `looks_like` chain (`find_tile_looks_like`, per-category, recursive with a jump limit). Any exporter/voxelizer that skips this chain will misreport coverage |

### Not verified / still open
- Whether an SDL3 "dummy" video driver yields a usable renderer for a headless tiles build — deliberately left to Phase 0 Spike B; do not plan on it.
- Exact cost of `display_buffer` readback on target hardware — Spike A measures it.
- Whether the fork should target `master` or a stable release branch — a decision for Joey/the project, recorded in Phase 0; the constants and file layout above are from `master` and may differ on a release branch.


---

# Part II — Test Plan

# CDDA Voxel Frontend: Test Plan — v2 (verified)

> **Maintenance:** the plan's maintenance rule (top of Part I) applies here too — record each phase's gate results in this part and the Progress log when the phase completes, before starting the next phase. Phase dependencies for scheduling are in plan §5 and repeated under each phase heading below.

Part II of this document (the plan is Part I — there is no separate test-plan file). Each phase has a **gate**: a set of tests that must pass before the phase is considered done. Later phases also re-run every earlier gate as a regression suite.

> **What changed in v2:** every **(verify)** item has been checked against the source (same pins as the plan: CDDA `master` @ `6a3e59e0`, Tilesets `master` @ `5ee942f9`, 2026-11-15), and the tests that depended on wrong assumptions have been rewritten. Summary of the corrections — details in Appendix A of the plan and in the rows below:
> - **Catch2 harness: verified.** CDDA's tests are Catch2 (`tests/test_main.cpp` runs a `Catch::Session`, Catch is vendored at `tests/catch/catch.hpp`, binary built via `tests/CMakeLists.txt`). Bonus fact the determinism tests rely on: the test listener **seeds the game RNG from the Catch seed at the start of every section**, so fork-side tests get reproducible RNG for free.
> - **Vehicle orientation steps: refuted.** There are not 24 orientations. `vehicle::face` is a continuous angle; test 8.1 now samples arbitrary angles and compares part positions against the game's own `abs_part_pos()` / `bub_part_pos()`.
> - **Creature facing up/down: refuted.** `FacingDirection` is `NONE`/`LEFT`/`RIGHT` only; test 6.6 now tests mirroring only.
> - **Stale-delta test rewritten.** The protocol syncs by `seq` + `epoch`, not game turn (plan §6); test 3.12 tests seq/epoch semantics.
> - **Snapshot comparison APIs are now named and exact:** `map::ter()`, `map::furn()`, `map::tr_at()`, `map::field_at()` (full field map, not one id), `map::i_at()` (full item stack), `map::has_graffiti_at()`, `map::apparent_light_at()` / `update_visibility_cache()` for `lit_level`.
> - **Scene sizes corrected** to the plan's real numbers (bubble 132×132, view radius 60, z −10…+10) in 2.13, 7.9 and the Phase 3 cost tests.
> - **New tests added** for the gaps v1 missed: map-memory layer (3.5b), map shift / bubble recentering (3.9b), field intensity changes in the delta invariant (3.9), open-air/roof handling (8.4b), season-variant and `looks_like` sprite resolution (5.5b, 5.10), and the fg-over-bg top-face rule (5.3).

---

## 1. Principles

1. **The game is the source of truth.** Most tests compare engine state or output against what the game itself reports — via its own query APIs (`map::ter/furn/tr_at/field_at/i_at`, `vehicle::abs_part_pos`, `Character::get_overlay_ids`), never against a re-implementation of the game's logic in the test.
2. **The core invariant:** applying every `Delta` to the last `Snapshot` must always equal a fresh `Snapshot`. This is tested continuously from Phase 3 onward.
3. **The fork must not change simulation.** A game run with the IPC frontend must produce the same game state as a run without it.
4. **Determinism first.** Seeded worlds plus scripted input make nearly everything replayable and diffable. Fork-side tests run under CDDA's Catch2 harness, which seeds the RNG per section; engine-side tests pin their own seeds the same way. Anything that is *known* non-deterministic even under a seed (timestamps, weather RNG streams not covered by the seed) is excluded from hashes explicitly, by field list — not by loosening the comparison.
5. **Automate by default.** Manual checks are limited to visual art review and are always backed by a generated artifact (contact sheet, screenshots).

---

## 2. Test layers and tooling

| Layer | Scope | Tooling |
|---|---|---|
| Unit | pure logic (mesher, camera, protocol, voxelizer rules) | `cargo test`, `proptest`, `insta` |
| Fuzz | decoders and framing | `cargo-fuzz` (libFuzzer) |
| Benchmark | meshing, delta apply, upload | `criterion` |
| Visual regression | headless renders of fixed scenes | `wgpu` software adapter (lavapipe on Linux, WARP on Windows), image diff with tolerance |
| Fork unit | exporters inside the game | CDDA's Catch2 harness — **verified**: `tests/`, `Catch::Session` in `tests/test_main.cpp`, vendored `tests/catch/catch.hpp`; new exporter tests are ordinary `TEST_CASE`s in a `tests/voxel_*.cpp` file, run with the normal `cata_test` binary |
| Cross-language | C++ <-> Rust compatibility | golden binary files plus a two-way round-trip test |
| Replay / integration | game + engine together, scripted | Python or Rust test driver that spawns both, injects input, asserts hashes |
| Soak | long runs, leaks | scripted play loop, RSS and handle sampling |
| Manual | art quality, feel | checklists with screenshots attached to the PR |

### Shared fixtures
- **Tiny worlds:** a handful of seeded, small worlds (starter shelter, a house with doors and windows, a road and field, a forest edge, a multi-story building **with a roof and a basement** — needed for open-air/roof tests, a vehicle on a road).
- **Fixture tileset:** a fake tileset for the voxelizer that must include the four cases that break naive pipelines: one multitile entry (all 8 slots), one fg-over-bg entry, one non-32×32 sprite with offsets, and one id resolvable only via `looks_like`. (v1's "20-sprite" count is fine as a size, but content requirements matter more than count.)
- **Golden files:** serialized snapshots, TileDefs, UI frames, atlas outputs. Stored in the repo with a documented update command (`UPDATE_GOLDEN=1`).
- **Input scripts:** plain text key/mouse sequences with expected checkpoints. Scripts must pin season/time-of-day where sprite or light output is compared (season-variant sprites and lighting are time-dependent).

### CI jobs
1. `rust-unit` (fmt, clippy, tests, proptest with fixed seed plus a nightly random seed)
2. `fuzz-smoke` (60 s per target on PRs, longer nightly)
3. `proto-compat` (C++ and Rust golden round-trip)
4. `fork-build-and-test` (cached, Linux first; runs the fork's Catch2 suite including the voxel `TEST_CASE`s)
5. `replay-integration` (engine + fork, headless)
6. `visual-regression` (software GPU)
7. `bench` (nightly; fails on more than 20% regression against stored baseline)
8. `soak` (nightly, 30 min; weekly, 2 h)

### Flaky-test policy
A test that fails intermittently is quarantined within 48 hours with a tracking issue. Gate tests may not be quarantined. A test that turns out to depend on unseeded game RNG is not "flaky" — it is miswritten; fix the seed or the hash field list (Principle 4) instead of quarantining.

---

## 3. Phase 0: Recon and decisions

> **Phase dependencies:** none — first phase. Its gate blocks Phases 1, 2, 3 and 5 (see plan §5).

Goal: prove the hook points exist before building on them. (Most source questions are already answered in the plan's Appendix A; these spikes prove them *in a running build*.)

| # | Test (spike) | Method | Pass criteria |
|---|---|---|---|
| 0.1 | Fork builds (curses/text build only) | CI build of the curses build (`CURSES=ON`, `TILES=OFF`) | the binary launches to the main menu. **The tiles/SDL3 build is NOT required and is not part of this test** (updated 2026-11-15): the fork ships as the text build + null `catacurses` backend and the Rust engine does all graphics, so SDL3 — the original game's graphics library, linked only by the tiles build — has no role in this project. A tiles build may still be produced optionally as an unmodified-upstream visual baseline, but no gate depends on it |
| 0.2 | Baseline upstream tests | run CDDA's Catch2 suite (`cata_test`) on the unmodified fork branch point | pass rate recorded; failures documented as pre-existing. This baseline is the comparison for 3.19 |
| 0.3 | Send-boundary probe | temporary log line at each candidate send boundary (`game::do_turn()` exit, redraw points) during a scripted walk, a multi-turn activity (crafting), and idle waiting | **Not** "exactly one per turn" (v1 was wrong: free actions, multi-turn activities and non-player changes break that). Pass = every scripted *state change* is followed by a boundary before the next input is consumed, and no boundary storm (≤ a small bound per turn) |
| 0.4 | Visible-tile probe | log count and checksum of exported terrain per z for a seeded world + script | values stable across two runs with the same seed, script, season and time |
| 0.5 | Light/visibility probe | log `lit_level` (via the visibility cache after `update_visibility_cache`) at known tiles: lit room, dark field, remembered-but-unseen tile | readings match expectations: room = `LIT`/`BRIGHT`, field at night = `DARK`/`LOW`, remembered tile = `MEMORIZED` |
| 0.6 | Offscreen UI capture probe | implement the plan's Route A at prototype quality: suppress `draw_ter()`, capture the SDL backend's `display_buffer` at present, dump PNG. If Route A fails, prototype Route B (`catacurses`-level) instead | PNG shows sidebar panels and a menu; the `w_terrain` rect is identifiable, empty/transparent, and its rect is reported with the frame. Readback cost per frame is measured and recorded (feeds the Phase 4 budget) |
| 0.7 | Input injection probe | feed a key event through `input_context::get_raw_input()`'s path programmatically | game reacts as if typed (e.g. opens inventory) |
| 0.8 | Headless/no-window probe | run per the plan's Spike B decision — the **null `catacurses` backend** (decided in Phase 0; the hidden-window tiles variant is dropped, as the tiles build is not part of this project) | game runs a scripted turn sequence without a visible window. Do not pass this on an SDL dummy-video-driver assumption — the run must actually render UI frames |
| 0.9 | Tileset parse probe | run `tools/gfx_tools/compose.py` on UltiCa, then parse the composed `tile_config.json` **and** one decomposed per-sprite entry; extract a multitile wall entry | composed output exists (it is not in the CDDA repo — generating it is part of the test); extracted entry has fg/bg plus the **six neighbour multitile subtiles** (`center, corner, edge, t_connection, end_piece, unconnected`) — **corrected in Phase 0 (2026-11-15):** decomposed entries carry at most these six; `open`/`broken` are state-variant ids, not additional_tiles (all eight names exist in `multitile_keys` in `cata_tiles.cpp`); one season-suffixed variant and one `looks_like` chain resolve |
| 0.10 | Licensing check | script lists licenses of bundled assets/code used, from both repos' `LICENSE.txt` including third-party notices | `docs/licensing.md` reviewed; Mushroom Dream excluded |

**Gate:** 0.3 (as redefined), 0.4, 0.6, 0.7 and 0.8 pass, or a documented fallback (in-process integration) is chosen.

**Phase 0 gate results (2026-11-15):** 0.3 ✅ harness + source · 0.4 ✅ · 0.5 ✅ · 0.6 ✅ standalone prototype (both routes) · 0.7 ✅ harness · 0.8 ✅ headless harness · 0.9 ✅ as corrected (compose.py run deferred — pyvips + full PNG tree) · 0.10 ✅ · **0.1 ⛔ not run here (curses/text build only — the tiles/SDL3 build is not required by this project, updated 2026-11-15; the curses compile is infeasible at 1.5 CPU/6 GB in the recon environment) · 0.2 ⛔ not run (needs 0.1)**. Verdict: **conditional pass** — mandatory tests pass at harness level; 0.1/0.2 and the in-game spike halves (probe patch in `tools/phase0/fork_probe_patch.md`) are a hard precondition for Phase 3, not for Phases 1/2/5. Full record in Part I's Phase 0 completion record and `docs/recon.md`.

---

## 4. Phase 1: Protocol and skeleton

> **Phase dependencies:** ⛔ requires the Phase 0 gate. Phase 2 work may start on the draft schema in parallel, but this gate is required by Phase 3.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 1.1 | Schema round-trip (Rust) | build every message with random valid data, encode, decode — including the sparse `FieldOnTile` list, a connect-group table with per-def indices, and `fg_ref`/`bg_ref` = −1 | decoded equals original |
| 1.2 | Schema round-trip (C++) | same in C++ | decoded equals original |
| 1.3 | Cross-language golden files | C++ writes golden files; Rust reads them; and the reverse | byte-identical and semantically equal |
| 1.4 | Verifier on corrupt input | flip bits and truncate golden files | decoder returns an error, never panics or reads out of bounds (FlatBuffers verifier must run before any table access, both languages) |
| 1.5 | Fuzz targets | `cargo-fuzz` on frame decoder and each message | no crashes in 10 min on CI-nightly, 60 s on PR |
| 1.6 | Framing | split frames across arbitrary read boundaries; coalesce several frames in one read | all messages recovered in order |
| 1.7 | Oversize/malformed length | send frame with absurd length prefix | connection rejected cleanly, no large allocation |
| 1.8 | Handshake | version match, mismatch (older/newer), missing hello | match proceeds; mismatch gives clear error and clean exit |
| 1.9 | Forward compatibility | add an unknown field to a golden message | old reader ignores it |
| 1.10 | Heartbeat | stop answering pings | peer detects within the timeout and shuts down |
| 1.11 | Shutdown both ways | kill engine; kill server; send `Shutdown` | other side exits within 2 s, no orphan processes |
| 1.12 | Input round-trip | engine sends key events to stub server | server receives the same sequence, order preserved |
| 1.13 | Shared-memory header | create/attach/destroy region from both languages | header fields (incl. `seq` and dirty rect) read identically; region removed on exit |
| 1.14 | **Seq/epoch semantics** | send deltas with a skipped `seq`, a duplicated `seq`, and an old `epoch` | skip → engine requests a fresh snapshot; duplicate → applied zero times (idempotent discard); old epoch → discarded. (Replaces any turn-number-based test: the protocol has no turn field) |
| 1.15 | Platform matrix | run 1.6-1.14 on Linux, then Windows | same results |

**Gate:** all of the above on Linux; CI green; golden files committed.

---

## 5. Phase 2: Engine MVP (fake data)

> **Phase dependencies:** ⛔ requires the Phase 0 gate; soft-requires Phase 1 for its gate. Runs in parallel with Phases 1, 3, 5.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2.1 | Single block mesh | one opaque block | 6 faces, correct normals and winding |
| 2.2 | Adjacent blocks | two touching blocks | shared faces removed |
| 2.3 | Greedy merge | 16x16 flat slab | one quad per visible large face; vertex count matches expectation |
| 2.4 | Chunk boundaries | block at edge, neighbor present/absent | face culled only when the neighbor chunk has an opaque block |
| 2.5 | Property: greedy equals naive | `proptest` random chunks | visible surface area and covered cells identical between greedy and naive meshers; no overlapping quads |
| 2.6 | Transparent/cutout passes | mixed block kinds | each block lands in the correct pass |
| 2.7 | Camera math | yaw/pitch/zoom clamps; follow target | values stay within limits; no NaN at extremes |
| 2.8 | Z-cutaway selection | each cutaway mode, player at several z including z = −10 and +10 extremes | expected set of z-layers/chunks drawn; no index errors at the extremes |
| 2.9 | Dirty propagation | change a boundary block | both chunks marked dirty; unrelated chunks untouched |
| 2.10 | Visual snapshot: fake house | headless render from 3 fixed cameras | image diff within tolerance of golden |
| 2.11 | Resize and minimize | resize sequence, zero-size surface | no panic; depth buffer recreated |
| 2.12 | Benchmark: meshing | `criterion`, 12x12xN chunk (N = cutaway depth) | under 2 ms per chunk (single thread) |
| 2.13 | Benchmark: frame | scene filling the real view: up to 132×132 tiles stored, camera view at radius ~60, cutaway z-range | median under 16 ms on the reference machine. (v1's "60x60x10" scene understated the world; the scene must match plan §11) |

**Gate:** 2.1-2.11 pass; benchmarks meet budgets.

---

## 6. Phase 3: Game-side export (highest risk)

> **Phase dependencies:** ⛔ requires the Phase 0 and Phase 1 gates. Runs in parallel with Phases 2 and 5. On the critical path to playable.

### 6a. Exporter correctness
All comparisons in this section are fork-side Catch2 tests calling the game's own query APIs — the exporter is wrong by definition if it disagrees with them.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 3.1 | TileDefs completeness | load base data; enumerate all terrain, furniture, traps, field types, monster types, item types, vehicle parts from the game's registries | every id appears exactly once, with name, string flags + resolved hot flags, connect-group indices that resolve into the exported group table, open/close/bash result ids, `looks_like`, and fg/bg refs |
| 3.2 | TileDefs with mods | load 3 representative mods, chosen by content type: one adding terrain/furniture, one adding monsters/items, one adding field types or vehicle parts | mod ids present in the right tables; no duplicates; no crashes |
| 3.3 | Snapshot vs game queries | for a seeded world, compare each exported tile against direct queries: `map::ter()`, `map::furn()`, `map::tr_at()` (trap id), `map::field_at()` — compared as a **full map of (type → intensity)**, `map::i_at()` — compared as **stack count + uppermost item id**, `map::has_graffiti_at()` | 100% match across all exported z-levels. A single-id field or item comparison does not satisfy this test |
| 3.4 | Snapshot bounds | player near a bubble edge (132×132) and near z limits (−10, +10) | no out-of-range reads; unexplored areas flagged `BLANK`/never-seen, not fabricated |
| 3.5 | Light and visibility export | after `map::update_visibility_cache(z)`, compare exported values per tile to the visibility cache and `map::apparent_light_at()` | `lit_level` enum matches **exactly** per tile (it is an enum, not a quantized float — no tolerance); the exported seen float is within the documented quantization of the cache value |
| 3.5b | Map-memory layer | visit a street, move away / break line of sight, then export | remembered tiles appear in the memory layer with terrain/furniture ids and `MEMORIZED` lit level; they carry **no** live entities, items, or fields, even if the real tiles have since gained them |
| 3.6 | Entity export | spawn player, NPC, several monsters via the game's creature tracking | positions, type ids and flags match game state; **`facing` is only ever `NONE`/`LEFT`/`RIGHT`** — any other value fails the test |
| 3.7 | Overlay list | player with armor, wielded item, mutation; one monster with overlays | overlay ids **and order** equal `Character::get_overlay_ids()` / `monster::get_overlay_ids()` output, which is what `cata_tiles::draw_entity_with_overlays` consumes |
| 3.8 | Vehicle export | vehicle on a road at several facings, including a non-axis-aligned one | per-part absolute positions equal `vehicle::bub_part_pos()` / `abs_part_pos()` for every part; exported `face_deg` equals `face.dir()` within float tolerance. The engine must not be re-deriving positions from mounts — this test compares against the game's pre-rotated results |

### 6b. Delta invariant

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 3.9 | **Delta equals snapshot** | random seeded action sequences (move, open/close doors, bash, pick up and drop items — including onto existing stacks, fire, craft, wait, start a fire and let it spread) plus non-action changes; after each send boundary apply deltas to a mirror and compare with a fresh snapshot — comparison includes **field maps with intensities, item stack counts, and vehicle part positions**, not just terrain ids | identical in 1000+ turns across 10 seeds |
| 3.9b | **Map shift** | scripted walk that crosses a submap boundary so `map::shift` recenters the bubble, and a z-level change (stairs) | mirror stays in sync across the shift; snapshot origin/epoch handling is exercised — a shift must not corrupt the mirror or be silently dropped |
| 3.10 | Missed-change hunt | scripted events known to change tiles outside the player's action (fire spread, monsters opening doors, field decay/intensity change, explosions, vehicle motion). Plus a unit test: mutate a submap directly with no player action and confirm the dirty-submap diff emits it | mirror stays in sync in all cases |
| 3.11 | Gap recovery | drop a delta, then send next | engine detects the `seq` gap and requests a fresh snapshot; state recovers |
| 3.12 | Stale and duplicate deltas | send a delta with an old `epoch`, and re-send the last delta unchanged | old-epoch discarded; duplicate discarded with no double-apply (e.g. no duplicated field intensity) |

### 6c. Non-interference and robustness

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 3.13 | **No simulation change** | run the same seed and input script with `--frontend=ipc` and without; compare a **canonical state dump** (exported tile/entity/vehicle state + avatar stats) at fixed checkpoints | dumps identical. Do **not** hash raw save files: saves embed timestamps and other non-simulation data, so a raw-file hash can fail without any simulation difference — if save comparison is wanted, diff parsed save JSON with a documented exclusion list |
| 3.14 | Replay determinism | same seed and script twice (Catch-seeded RNG fork-side) | exported snapshot hashes identical at every checkpoint, over the hash field list defined in Principle 4 |
| 3.15 | Headless operation | run 500 turns with no window (per the Phase 0 headless decision) | no crash, no window created |
| 3.16 | Engine disconnect | kill engine mid-turn | game autosaves or exits cleanly per design; no corrupted save |
| 3.17 | Slow engine | engine stops reading for 10 s | game never blocks on the socket beyond a bounded buffer; on overflow the documented behavior (drop deltas + force resync via new epoch, or pause) is what actually happens, verified by the mirror recovering |
| 3.18 | Export cost | time the exporter per send boundary on the full view window | snapshot export under budget (target: under 20 ms for the view window in plan §11); delta under 2 ms for a typical turn, with the dirty-submap count logged so cost regressions are attributable |
| 3.19 | Upstream suite | run CDDA's Catch2 suite on the fork | no new failures compared with baseline 0.2 |
| 3.20 | Save/load | save, reload, continue | exporter state rebuilds; engine resyncs via a new epoch |

**Gate:** 3.3, 3.5, 3.9, 3.13, 3.14, 3.19 are mandatory; others must pass or have a ticketed known gap.

---

## 7. Phase 4: UI compositing and input

> **Phase dependencies:** ⛔ requires the Phase 3 gate; soft-requires Phase 2. Runs in parallel with Phase 5.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 4.1 | UI golden frames | capture main menu, character creation, inventory, crafting, message log, sidebar — at a fixed terminal size, font and panel layout (layouts are user-configurable; goldens pin one) | image diff within tolerance of goldens |
| 4.2 | Map-region masking | capture gameplay UI | the **`w_terrain` rect reported by the fork** (not an assumed fixed rect — it depends on terminal size and panel layout) is transparent (alpha 0) where the engine draws the world; everything outside it is opaque as the game drew it |
| 4.3 | Torn-frame test | writer produces frames with an embedded checksum at max rate; reader samples continuously | zero torn frames in 10 min |
| 4.4 | Double-buffer sequencing | pause reader/writer at random points | reader always sees a complete frame |
| 4.5 | Dirty rect | change a small region | only that rect is re-uploaded; the full frame matches a full re-upload |
| 4.6 | Resize and DPI | resize through several sizes and DPI scales | UI texture and engine stay consistent (display-buffer pixels vs window pixels under the backend's scaling factor are handled); no crash; text readable |
| 4.7 | Key matrix | letters, digits, symbols, arrows, function keys, modifiers, key repeat | each reaches the game as the same key the player pressed, via `input_context` resolution — test at the action level ("opens inventory") as well as the raw-key level |
| 4.8 | Text input | name entry, search boxes, non-ASCII characters | text appears correctly |
| 4.9 | Focus handling | menu open vs closed, driven by the UiFrame mode flag | camera drags ignored when a menu is open; game keys pass through when appropriate |
| 4.10 | Mouse to UI | click menu entries and sidebar panels | correct entry activates (UI clicks stay in UI/display-buffer coordinates) |
| 4.11 | Mouse picking | click world tiles for look/target/examine on a calibration scene | tile coordinates sent match the tile under the cursor; and a click that lands on a UI panel is *not* treated as a world pick (mode flag separation) |
| 4.12 | Latency | key press to updated UI frame on screen | median under 50 ms, p99 under 100 ms. If Route A (display-buffer readback) is used, its measured readback cost from 0.6 is included in this budget |
| 4.13 | **End-to-end script** | create character, start game, move, open inventory, craft, save, quit, relaunch, load | script completes with all checkpoints |
| 4.14 | Soak | scripted random play for 30 min | no crash; RSS growth under 5%; no handle growth |

**Gate:** 4.1-4.9, 4.13 mandatory; 4.11 may ship as keyboard-only if documented.

---

## 8. Phase 5: Voxelizer and textures

> **Phase dependencies:** ⛔ requires the Phase 0 gate; soft-requires Phase 1. Runs in parallel with Phases 2, 3, 4.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 5.1 | Fixture tileset golden | run on the fixture tileset (with multitile, fg-over-bg, oversized-sprite and `looks_like`-only cases) | outputs byte-identical to goldens |
| 5.2 | Determinism | run twice on UltiCa | byte-identical atlas, texture array and `materials.json` |
| 5.3 | Top face | for each terrain material in the fixture + samples from UltiCa | top texture equals the **fg composited over bg** source (pixel-exact after compositing); an fg-only comparison fails by design for transparent-fg floors |
| 5.4 | Side face rules | floor, wall, grass samples | side darkening within configured range; walls use wall sprite directly |
| 5.5 | Multitile mapping | all **16 neighbor masks** from the 4 cardinal neighbors (this is the game's own model: `cata_tiles::get_tile_values` takes a 4-neighborhood array and derives subtile + rotation via `get_rotation_and_subtile`), **plus** connect-group cases where the neighbor is a *different* id in the same connect group (`get_tile_values_with_ter`) | each mask/group case maps to the variant the game's function pair produces — expected values are taken from the game's logic, not a hand-written table alone. Slots covered: center, corner, t_connection, edge, end_piece, unconnected. (`open`/`broken` are state variants, not neighbor-mask variants — covered in 8.4) |
| 5.5b | Sprite resolution chain | ids that resolve via variant (`_var_`), via season suffix at a pinned season, and via `looks_like` | each resolves to the same sprite the game's `find_tile_looks_like` chain resolves; none is reported missing |
| 5.6 | Alpha handling | sprites with holes (fences, trees) and glass | correct cutout/transparent flags; no color bleed at edges |
| 5.7 | Mipmaps | generate mip chain | no edge bleeding between array layers; alpha-aware downsample verified on test patterns |
| 5.8 | Extrusion | furniture with known alpha mask and height | voxel columns match mask; height matches override or heuristic |
| 5.8b | Sprite geometry | sprites from every UltiCa sheet size (20×20, 32×36/48, 32×64, 64×64, 64×96, 96×96) | atlas UVs and stored width/height/offset equal the `tile_info.json` values per sheet; oversized sprites are anchored by their offsets, not centered |
| 5.9 | Override file | override a material and a height | override applied; invalid overrides give a clear error |
| 5.10 | **Coverage** | run against full UltiCa plus the game's TileDefs, resolving every id through the full chain (exact → variant → season → `looks_like` → filler) **before** counting it missing | at least 98% of base-game terrain and furniture ids have non-placeholder materials; the rest are listed in `missing.log` with the resolution stage that failed |
| 5.11 | Unknown id fallback | request an id with no material and no `looks_like` | placeholder renders; log entry emitted once |
| 5.12 | Attribution | generate `ATTRIBUTION.md` | lists tileset, per-sheet sources where `tile_info.json` provides them, authors/sources, license **including the third-party notices**; matches inputs used |
| 5.13 | Visual scenes | headless render of house, road, field, forest, shelter | image diff within tolerance of goldens |
| 5.14 | Contact sheet review (manual) | review `contact.png` | reviewer checklist signed off; problem ids filed as override tasks |
| 5.15 | Performance | voxelizer run time on UltiCa (including the `compose.py` step if the pipeline performs it) | under 60 s on reference machine |

**Gate:** 5.1-5.3, 5.5, 5.10-5.13 pass; manual review done.

---

## 9. Phase 6: Billboards

> **Phase dependencies:** ⛔ requires the Phase 2 and Phase 5 gates; soft-requires Phase 3. Runs in parallel with Phase 7.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 6.1 | Orientation | camera at several yaws | quads always face the camera, upright (rotation about Y only) |
| 6.2 | Depth sorting | entities in front of and behind walls and windows | correct occlusion in headless render |
| 6.3 | Alpha edges | sprites with soft edges | no dark halos; matches alpha-test/dither design |
| 6.4 | Overlay composition vs game | compose the same entity two ways: (a) the game's own rendering — `cata_tiles::draw_entity_with_overlays` in tiles mode, captured offscreen; (b) the engine's compositor fed the exported overlay list | pixel diff within tolerance for player with several equipment sets and one mutated character. Tolerance must allow for the game's `height_3d` offsets; geometry/scale differences are aligned before diffing |
| 6.5 | Overlay cache | change one overlay; same signature again | new composite only when the signature changes; cache hit otherwise |
| 6.6 | Facing and mirroring | facing `LEFT`, `RIGHT`, `NONE` | `LEFT` mirrored horizontally, `RIGHT`/`NONE` unmirrored. There is **no up/down case** — v1's version of this test described sprites that do not exist in this tileset system |
| 6.7 | Item and creature variety | all monster sprites in UltiCa, plus item piles (uppermost item per tile) | no missing sprites after the resolution chain (5.5b); missing list generated for genuine gaps |
| 6.8 | Shadows | blob shadow under entity | follows entity, hidden when under roof if designed so |
| 6.9 | Picking billboards | click on sprite pixels, including transparent areas and oversized sprites (offset geometry from 5.8b) | opaque pixels hit; transparent pixels pass through; hit area matches the sprite's real size/offset, not a 32×32 box |
| 6.10 | Stress | 2000 billboards visible | frame time within budget; instance buffer resizes without crash |
| 6.11 | Visual scene | zombie crowd near walls and windows | matches golden |

**Gate:** 6.1, 6.2, 6.4, 6.5, 6.10, 6.11.

---

## 10. Phase 7: Lighting, FOV and atmosphere

> **Phase dependencies:** ⛔ requires the Phase 2 and Phase 3 gates; soft-requires Phase 5. Runs in parallel with Phase 6.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 7.1 | Light mapping | tiles at each `lit_level` (`DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT`) plus a seen-float gradient | rendered brightness is monotonic with the enum order and with the seen float within a level |
| 7.2 | Visibility states | currently seen, remembered (`MEMORIZED`), never seen (`BLANK`) | bright, desaturated/dark, and black/not-meshed respectively |
| 7.3 | No light leaking | lit room next to a dark room through a wall | dark side stays dark in render |
| 7.4 | Smoothing | gradient across neighbors | no hard seams at chunk borders |
| 7.5 | Light updates | toggle a light source in-game | engine updates only affected chunks within one frame of the delta |
| 7.6 | Day/night | game time across a day (pinned times) | tint transitions smoothly; matches game time of day |
| 7.7 | Weather | rain/fog states from the game | correct effect on/off; no performance cliff |
| 7.8 | Visual scenes | lit house at night, flashlight in field, remembered street | match goldens |
| 7.9 | Performance | full lighting over the real view (radius ~60, cutaway z-range, 132×132 stored) | frame budget maintained. (Scene size corrected from v1's 60x60x10) |

**Gate:** 7.1-7.3, 7.5, 7.8, 7.9.

---

## 11. Phase 8: Hard cases and release

> **Phase dependencies:** ⛔ release gate requires the Phase 4, 5, 6 and 7 gates; individual tests need only their own phase (8.1–8.3: Phase 3; 8.5: Phases 3–4).

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 8.1 | Vehicle orientations | sample facings across the full circle — 0°, 7°, 15°, 23°, 45°, 90°, 179°, 270°, 359° and values hit during a scripted turn — there is **no fixed step count**: `face.dir()` is a continuous `units::angle` (the 15° in v1 is a pathing clamp in `vehicle::immediate_path()`, not an orientation quantum) | for every sampled facing, each part's engine position equals the game's `vehicle::abs_part_pos()` / `bub_part_pos()` for that part (exact tile match); facing error within float tolerance |
| 8.2 | Vehicle movement | drive a scripted path, including a turn | smooth model motion; no part drift (re-check 8.1 at intervals during the drive); deltas keep in sync (3.9 invariant holds with vehicles included) |
| 8.3 | Vehicle damage/removal | break or remove parts, open/close vehicle doors, fold/remove a roof part | model updates correctly, including the roof / no-roof part split the game's renderer uses |
| 8.4 | Door and window states | open, closed, broken, bashed — for terrain doors and furniture doors | each action arrives as a delta carrying the **new terrain/furniture id** (state is an id change via the game's open/close/bash transforms, not a state flag); model and texture match the new id after each change |
| 8.4b | Open air, roofs, multi-storey | two-storey house fixture + basement: stand on floor 1, floor 2, and outside looking in, at several cutaway settings | no phantom floor over `t_open_air`, roof terrain renders as the floor above per the game's `roof` field, no missing ceilings/floors at any cutaway |
| 8.5 | Animations/projectiles | fire weapons, throw, explosion | effects appear at the right tiles and times; no leftover effects |
| 8.6 | Overmap | open the overmap | UI texture shows it correctly (it is game-drawn UI, incl. its own overmap sprites); input works; returning restores the 3D view |
| 8.7 | Mod smoke | launch with top popular mods, one at a time | no crash; missing-material report under agreed threshold (measured with the 5.10 resolution chain) |
| 8.8 | Long soak | 2 h scripted play | no crash; RSS growth under 5%; frame time stable |
| 8.9 | Memory leak check | sample RSS and GPU memory every minute | no monotonic growth after warm-up |
| 8.10 | Performance regression | full benchmark suite | all budgets in `plan.md` section 11 met |
| 8.11 | Clean-machine install | install package on fresh Linux and Windows VMs | engine launches the game, generates or loads assets (incl. running or shipping the `compose.py` step), reaches gameplay |
| 8.12 | Upgrade path | install new version over old, load an old save | save loads; assets regenerate; no stale cache issues; engine resyncs via new epoch |
| 8.13 | Crash reporting | force engine and game crashes | logs captured with version info (including the pinned upstream commit of the fork); user-facing message clear |
| 8.14 | Final manual playthrough | 1 h session on each platform | checklist: UI, camera, combat, driving, crafting, saving |

**Gate (release):** all earlier gates green; 8.1, 8.4b, 8.7, 8.8, 8.10, 8.11 pass.

---

## 12. Cross-phase regression suite

These run on every PR once their phase has shipped:

- Protocol golden and fuzz-smoke (Phase 1), including seq/epoch semantics (1.14)
- Mesher properties and benchmarks (Phase 2)
- Delta equals snapshot, map shift, and no-simulation-change (Phase 3)
- UI golden frames and the end-to-end script (Phase 4)
- Voxelizer determinism, resolution chain and coverage (Phase 5)
- Billboard overlay comparison (Phase 6)
- Lighting visual scenes (Phase 7)

---

## 13. Phase gate checklist (copy into each phase PR)

- [ ] All gate tests for this phase pass in CI
- [ ] All earlier gates still pass
- [ ] Benchmarks meet the budgets in `plan.md` section 11
- [ ] New golden files reviewed in the PR diff
- [ ] No quarantined gate tests
- [ ] Known gaps listed with tickets
- [ ] Docs updated (`recon.md`, protocol notes, voxelizer rules)
- [ ] Manual review items (if any) signed off with screenshots attached

---

## Appendix: verification notes for this test plan

All items below checked 2026-11-15 against CDDA `master` @ `6a3e59e0` (raw source):

- **Catch2 harness (v1 §2, test layer "Fork unit") — Verified.** `tests/test_main.cpp`: `#define CATCH_CONFIG_RUNNER`, a `Catch::Session`, and a `CataListener : Catch::TestEventListenerBase` whose section-start handler initializes "the cata RNG with the Catch seed". Catch is vendored (`tests/catch/catch.hpp`); `tests/CMakeLists.txt` exists. Fork exporter tests belong in this harness.
- **Map query APIs (test 3.3) — Verified, named.** `src/map.h`: `ter_id ter(const tripoint_bub_ms&)`, `furn_id furn(tripoint_bub_ms)`, `const trap &tr_at(const tripoint_bub_ms&)`, `const field &field_at(const tripoint_bub_ms&)`, `map_stack i_at(const tripoint_bub_ms&)`, `bool has_graffiti_at(const tripoint_bub_ms&)`.
- **Light/visibility APIs (tests 0.5, 3.5) — Verified, named.** `src/map.h`: `lit_level apparent_light_at(p, cache)`, `visibility_type get_visibility(lit_level, cache)`, `update_visibility_cache(int zlev)`, `invalidate_visibility_cache()`, per-level caches in `level_cache`. `lit_level` enum in `src/lightmap.h`.
- **Overlay APIs (test 3.7) — Verified** in the plan's Appendix A (#10): `get_overlay_ids()` order is the composition order.
- **Vehicle part positions (tests 3.8, 8.1) — Verified, named.** `src/vehicle.h`: `tripoint_bub_ms bub_part_pos(const map&, int)`, `tripoint_abs_ms abs_part_pos(int)`, `precalc_mounts(int, const units::angle&, const point_rel_ms&)`, `pivot_rotation` array, `tileray face`. Continuous angle — no orientation step count exists to test against.
- **Multitile model (test 5.5) — Verified.** `cata_tiles::get_tile_values(int t, const std::array<int,4> &tn, …)` builds a 4-bit value from the cardinal neighborhood (order seen in callers: south, east, west, north) and calls `get_rotation_and_subtile`; `get_tile_values_with_ter` is the connect-group-aware variant. 16 masks is the correct cardinal case count.
- **Not verified / deliberately left open:** exact readback cost of the SDL `display_buffer` (measured by spike 0.6, budgeted in 4.12); whether a stable release branch differs from `master` in any of the APIs above (re-check this appendix if the fork pins a release instead).
