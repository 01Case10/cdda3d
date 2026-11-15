# Recon — CDDA Voxel Frontend, Phase 0

**Date:** 2026-11-15 · **Status:** Phase 0 complete at harness level; in-game
build spikes blocked in this environment (see §6 — blockers are concrete,
with the exact unblock path).

**Pinned sources (verified by `git rev-parse HEAD` in `fork-src/`):**
- CDDA `master` @ `6a3e59e08842de34ff3f111c7f098abeffb708a5` — sparse,
  blobless clone: `src/`, `tools/gfx_tools/`, `tests/`, `LICENSE.txt`,
  `CMakeLists.txt`, `Makefile`
- CDDA-Tilesets `master` @ `5ee942f9e6601970c5da356200e7cf5061226262` —
  sparse clone: `gfx/UltimateCataclysm` JSON + metadata, `LICENSE.txt`,
  `readme.md`

Everything below marked *(source)* was extracted mechanically from those
checkouts by `tools/phase0/extract_hooks.py` (32/32 hook points found —
full catalogue in `docs/hook-points.md`) and `tools/phase0/probes.py`.
Re-run everything with `tools/phase0/run_all.sh`.

---

## 1. The three exit questions, answered

### Q1. Where is a safe turn/send boundary?
**`game::do_turn()` exit — which lives in `src/do_turn.cpp:536`, not
`src/game.cpp`** *(source; new finding — the plan named game.cpp)* — plus
the redraw point at the end of `game::draw(ui_adaptor&)`
(`src/game.cpp:3364`, after `draw_panels(true)` at :3404).

`do_turn()` is declared at `src/game.h:221`. The boundary policy the fork
implements (Phase 3): accumulate dirty submaps continuously; flush a
`Delta` at `do_turn()` exit **and** at any redraw that follows a
non-player state change, always before the next input is consumed.
Test 0.3's harness proves this policy on a scripted trace (walk,
multi-turn craft, idle wait, monster-only move, fire spread): 8 state
changes → 7 boundaries, every change flushed before the next input, no
boundary storm. The v1 criterion "exactly one boundary per turn" stays
refuted — free actions and multi-turn activities break it.

### Q2. How do we read visible tiles per z-level?
Per tile, from `src/map.h` *(source, line numbers in hook-points.md)*:
- `ter_id ter(const tripoint_bub_ms&)` :1045
- `furn_id furn(tripoint_bub_ms)` :1020 — **by value, not const&**
  (Phase 0 correction to Appendix A's implied signature)
- `const trap &tr_at(const tripoint_bub_ms&)` :1614
- `const field &field_at(const tripoint_bub_ms&)` :1660 — the **full**
  field map (type → entry{intensity, age}), never a single id
- `map_stack i_at(const tripoint_bub_ms&)` :1424 — item stack; the
  uppermost item comes from the submap maptile:
  `submap.h:439 get_uppermost_item()` *(source; it is not a `map` method)*
- `bool has_graffiti_at(const tripoint_bub_ms&)` :1811

View geometry, from `src/map_scale_constants.h` *(source)*: `SEEX=12`,
`SEEY=SEEX`, `MAPSIZE=11` → bubble 132×132, `MAX_VIEW_DISTANCE=60`
(radius). Export covers the view window (radius ≤ 60) × the cutaway
z-window, not the whole 132×132×21 bubble (plan §11 math stands).

Light/visibility: call `map::update_visibility_cache(zlev)`
(`map.h:2520`), then read `lit_level apparent_light_at(p, cache)`
(`map.h:585`). `lit_level` (`src/lightmap.h:96`) is
`enum class lit_level : uint8_t { DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT,
MEMORIZED, BLANK }` — compared exactly, no quantization tolerance.
Test 0.5 verified all 7 members in source and a monotonic engine
brightness mapping. Remembered tiles come from the avatar's map memory
as a separate layer (terrain/furniture only), per plan Phase 3.

Delta mechanism (no per-tile hook exists — confirmed again): hook
`map::invalidate_map_cache(zlev)` (`src/map.cpp:550`),
`map::set_seen_cache_dirty` (`map.h:526`),
`map::set_transparency_cache_dirty` (`map.h:517`) to accumulate a
dirty-submap set, diff those submaps at the send boundary.
`map::shift(const point_rel_sm&)` (`src/map.cpp:9170`) is an epoch event:
on shift, the fork bumps `epoch` and the engine re-bases.

### Q3. How do we redirect UI drawing to a texture?
The game draws UI as separate `catacurses` windows — `game::draw()`
erases `w_terrain`, calls `draw_ter()` (`game.cpp:3387`), runs callbacks,
then `draw_panels(true)` (:3404); menus are `ui_adaptor`s on the
`ui_manager` stack — **but** the SDL3 backend composites everything into
one texture first: `static SDL_Texture_Ptr display_buffer`
(`src/sdltiles.cpp:151`), with `display_buffer_draw_scope`,
`compute_display_buffer_dims()` (:263) and
`window_to_display_buffer_coords` handling the pixel mapping *(source)*.

**Spike A result (test 0.6) — both routes prototyped standalone**
(`tools/phase0/spike_ui_capture.cpp`, output
`tools/phase0/spike_out/spike_ui.png`):
- **Route B (catacurses-level) works end-to-end at prototype quality:**
  terrain window suppressed (rect stays alpha 0), sidebar panels,
  message log and a menu adaptor rendered into one offscreen RGBA
  buffer; the PNG shows exactly that — transparent (magenta) terrain
  rect, opaque panels/log, menu drawn *over* the terrain area, matching
  the real draw order.
- **Route A cost model:** compositing + full-buffer readback of a
  1600×800 (100×50 cells × 16 px) frame costs **0.39 ms mean / 0.65 ms
  worst** CPU-side over 200 iterations. This is a *floor*: a real
  `SDL_RenderReadPixels` also pays the GPU pipeline stall, which only
  the in-game spike (§6) can measure. Budget implication for Phase 4:
  capture on dirty frames only, double-buffered — as the plan already
  specifies.
- **Decision: Route B is the primary route, Route A the fallback.**
  Reason: Route B needs no GPU readback at all, works for the curses
  build, and is the same code path as the chosen headless route (§2).
  Route A remains attractive if Route B's per-window plumbing proves
  invasive; its capture point is verified to exist.

## 2. Spike B — headless decision
Options from the plan, resolved:
1. *Tiles build, hidden SDL window* — **dropped** (it needs SDL3 and a
   renderer, and the tiles build is out of scope entirely — §6
   clarification); the SDL "dummy video driver" yielding a usable
   renderer remains **unverified** and is not planned on.
2. ***Null/custom `catacurses` backend* — CHOSEN.** The game already
   routes every UI draw through the `catacurses::window` abstraction
   (`src/catacurses.h`, impl `src/cursesport.cpp`); a backend that
   renders windows into the shared offscreen buffer and never creates a
   window is a small, additive fork component. Test 0.8's harness ran
   the full probe suite plus a 50-turn scripted sequence with
   `DISPLAY` unset and no window created — the design needs no display
   at any point.

## 3. Decisions (Phase 0)
| Decision | Choice | Basis |
|---|---|---|
| World scale | 1 tile = 1 block footprint; **1 z-level = 3 blocks tall** | plan default; billboards ~2 blocks fit doorways; revisit visually in Phase 2 (open question §14.1 stays open) |
| Chunk size | **12×12 tiles (one submap) × cutaway z-window** per chunk column | matches `SEEX/SEEY=12` and the dirty-submap delta unit |
| Protocol serialization | **FlatBuffers** | zero-copy snapshot arrays, C++/Rust codegen from one schema, built-in verifier (test 1.4 depends on it); protobuf's advantage (schema evolution tooling) doesn't outweigh per-tile array cost |
| Engine license | **MIT OR Apache-2.0 (dual)** for the Rust engine; fork stays **CC BY-SA 3.0**; derived assets CC BY-SA 3.0 | `docs/licensing.md` |
| UI route | **Route B primary** (catacurses-level), Route A fallback | §1 Q3 |
| Headless route | **Null `catacurses` backend** | §2 |
| First tileset | **UltiCa only** | default tileset; Altica/Chibi_Ultica characters deferred (§14.4 narrowed, not closed) |
| Fork base | **`master` @ 6a3e59e0, pinned** — not a release branch | all Appendix A evidence is from master; re-check Appendix A if this ever changes |
| Target platform v1 | **Linux first** (Windows packaging in Phase 8) | plan §10/§12 already Linux-first; §14.5 narrowed accordingly |

## 4. Annotated source map (what Phase 0 actually read)
- `src/do_turn.cpp` (884 lines) — `game::do_turn()` :536; the turn
  pipeline the send boundary hooks. **Not in game.cpp.**
- `src/game.cpp` (11,659 lines) — `game::draw(ui_adaptor&)` :3364,
  `draw_ter()` call :3387, `draw_panels(true)` :3404.
- `src/sdltiles.cpp` (7,587 lines) — `display_buffer` :151 and the
  draw-scope/dims/coordinate conversion around it (:193–:380).
- `src/cata_tiles.cpp` (6,078 lines) — `multitile_keys` :112 (all 8:
  center, corner, edge, t_connection, end_piece, unconnected, open,
  broken); `drawing_layers` :1041 — the exact 11-layer order:
  terrain, furniture, graffiti, trap, part_con, field_or_item,
  vpart_no_roof, vpart_roof, critter_at, zone_mark,
  zombie_revival_indicators.
- `src/map.h` / `src/map.cpp` (12,349) / `src/submap.h` — snapshot
  readers and dirty hooks per §1 Q2; `map::shift` :9170.
- `src/lightmap.h` — `lit_level` :96.
- `src/input_context.h` — `get_raw_input()` :427, `handle_input()`
  contract :227. Test 0.7's harness injected `[i, x, .]` through the
  same queue→action resolution and the stub reacted
  `[inventory, unbound:x, wait]` — inventory first, as if typed.
- `src/creature.h` :217 — `FacingDirection { NONE=0, LEFT=1, RIGHT=2 }`.
  Flip only; there is no other facing to export.
- `src/character.h` :2553 / `src/monster.h` :630 — `get_overlay_ids()`
  returning ordered `(id, looks_like)` pairs; the engine must not
  re-sort.
- `src/vehicle.h` — `bub_part_pos(map, index)` :1555 (game-side
  pre-rotated absolute positions), `tileray face` :2560 (continuous
  angle); `src/vehicle.cpp:734` is the "clamp to multiples of 15"
  pathing comment — not an orientation quantum.
- `src/animation.cpp` :293 — animation dispatch exists with a tiles
  context; Phase 8 effects hook point.
- `src/main.cpp` :900 — `main()`; `--frontend=ipc` parsing lands here.
- `tests/test_main.cpp` — Catch2 `Catch::Session` with the RNG-seeding
  listener (re-verified in the sparse checkout's `tests/` tree);
  fork exporter tests will be `tests/voxel_*.cpp` `TEST_CASE`s.
- `tools/gfx_tools/compose.py` — present; converts per-sprite entries
  (names) to composed `tile_config.json` (indices). Requires
  Python + `pyvips`/libvips.

## 5. Tileset findings (test 0.9)
- `gfx/UltimateCataclysm/tile_info.json` is a **list of 22 entries**:
  entry 0 is the base `{pixelscale:1, width:32, height:32}`; each later
  entry maps a sheet filename to `{sprite_width, sprite_height,
  sprite_offset_x, sprite_offset_y}`. Distinct (w,h,ox,oy) tuples
  measured: (20,20,+6,0), (32,32,0,0), (32,36,0,−8), (32,48,0,−16),
  (32,64,0,−32), (64,64,−16,−16), (64,64,−16,−27), (64,64,−16,−32),
  (64,96,−16,−64), (96,96,−32,−64) — matches plan §8, plus a second
  64×64 offset variant (−27 y) the plan didn't list.
- 2,027 decomposed JSON files under UltiCa; 853 multitile entries.
- **Correction to test 0.9's wording:** no decomposed entry carries all
  eight multitile slots. The maximum is the **6 neighbour subtiles**
  (e.g. `f_metal_bench`: center, corner, edge, end_piece,
  t_connection, unconnected). `open`/`broken` are state variants
  resolved as separate ids — consistent with test 5.5's own note. The
  probe now asserts the verified shape (6 in entries + all 8 keys in
  `multitile_keys`).
- fg-over-bg entries exist as expected (e.g. `f_electrical_conduit`
  carries both `fg` and `bg`).
- **Compose run deferred:** `compose.py` needs `pyvips`; libvips is
  installable here (apt candidate 8.15.1) but the sparse checkout
  intentionally excludes UltiCa's PNGs (thousands of files), which
  compose consumes. Run compose on a full Tilesets checkout on the
  build machine (§6) — the parse half of 0.9 passes.

## 6. Build status (tests 0.1 / 0.2) — ⛔ NOT RUN in this environment (compile capacity only; the tiles/SDL3 build is not required at all — see the clarification at the end of this section)
What was attempted and measured here (Ubuntu 24.04 VM, 1.5 shared CPUs,
6 GB shared RAM):
- Toolchain present: gcc/g++ 13.3, CMake 3.28, Make 4.3, Python 3.12.
  **Rust is not installed** (needed from Phase 1, not Phase 0's gate).
- **Tiles build (SDL3): not required — removed from scope
  (2026-11-15).** This build was in the original test 0.1 only as an
  upstream baseline / Route A host; the project ships the text build
  (see the clarification below), so its configure failure here —
  `cmake -DTILES=ON` → `Could NOT find SDL3` (no `libsdl3-dev`
  candidate exists in Ubuntu 24.04) — is recorded as evidence about
  the *environment*, not as a project block.
- **Curses build: configure reaches the Curses check and stops there.**
  Actual attempt: `cmake -DTILES=OFF -DCURSES=ON` →
  `Could NOT find Curses (missing: CURSES_LIBRARY CURSES_INCLUDE_PATH)`
  (the compiler/ZLIB/Threads checks before it pass). `libncurses-dev`
  6.4 and `gettext` 0.21 are apt-installable, so configure *can* be
  satisfied — but a full CDDA compile is ~1,500 translation units
  (map.cpp alone is 12k lines); on 1.5 shared CPUs with 6 GB RAM that
  is a multi-hour, OOM-prone build — not a credible in-session spike.
- Therefore **0.1 (curses fork build) and 0.2 (baseline `cata_test`
  run) are recorded as NOT RUN here, not passed** — blocked solely by
  this environment's compile capacity. The in-game halves of spikes
  0.3–0.8 are written up as an exact, minimal probe patch in
  `tools/phase0/fork_probe_patch.md`, ready to apply on the
  `voxel-frontend` branch.
- **Unblock path (pick one):** (a) run the build on Joey's PC
  (9800X3D / 32 GB-class machine; MSYS2 route per upstream docs), or
  (b) a CI runner with ≥4 cores / ≥16 GB. **No SDL3 is needed on the
  target machine.** Then: build the curses version (`make` or CMake
  `-DCURSES=ON -DTILES=OFF`), launch it to the
  main menu (0.1), run `cata_test` unmodified and record the baseline
  pass rate (0.2), apply the probe patch and run the scripted
  sequences (in-game 0.3–0.8).

**Clarification (2026-11-15, Joey's question — "we do the graphics now,
why do we need SDL3?"): we don't.** SDL3 is only the *original game's*
graphics library, and only the fork's *tiles* build would ever link it
(verified: `CMakeLists.txt` searches for SDL3 only under `if(TILES)`;
the text/curses build never touches it). The shipped fork is the
**text build + null `catacurses` backend** (Route B, §1–2): the game
hands us its menus as text cells and all world data over IPC, and the
Rust engine does 100% of the graphics. The tiles build appeared in
test 0.1 only as (a) an unmodified-upstream baseline to compare
against, and (b) the home of Route A, the original UI-capture plan
that Phase 0 demoted to fallback. **Consequence: the SDL3 half of the
0.1 block is cancelled, not just deferred** — the only build the
project actually needs is the curses/text one (plus `cata_test`). What
remains blocked here is solely "this VM is too weak to compile the
game at all", and the unblock above no longer needs SDL3 on the
target machine either.

## 7. Gate results (Part II, Phase 0)
| # | Test | Result |
|---|---|---|
| 0.1 | Fork builds (curses/text only — tiles/SDL3 build not required, removed from scope 2026-11-15) | ⛔ NOT RUN (env compile capacity, §6) |
| 0.2 | Baseline upstream tests | ⛔ BLOCKED (needs 0.1) |
| 0.3 | Send-boundary probe | ✅ PASS (harness + source sites; in-game half in probe patch) |
| 0.4 | Visible-tile probe | ✅ PASS (deterministic checksums, real constants) |
| 0.5 | Light/visibility probe | ✅ PASS (enum + APIs verified in source; mapping monotonic) |
| 0.6 | Offscreen UI capture | ✅ PASS (standalone prototype, both routes; PNG produced) |
| 0.7 | Input injection probe | ✅ PASS (harness against real `input_context` API) |
| 0.8 | Headless probe | ✅ PASS (full suite + 50 scripted turns, DISPLAY unset, null-backend design) |
| 0.9 | Tileset parse probe | ✅ PASS with correction (§5); compose.py run deferred (pyvips + full PNG tree) |
| 0.10 | Licensing check | ✅ PASS (`docs/licensing.md`; Mushroom Dream excluded) |

**Gate verdict: CONDITIONAL PASS.** The gate's mandatory tests
(0.3, 0.4, 0.6, 0.7, 0.8) pass at the level this environment can prove,
every hook point is verified in the pinned source (32/32), and all
Phase 0 decisions are recorded. The condition: 0.1/0.2 and the in-game
halves of 0.3–0.8 must be run on a build-capable machine before Phase 3
starts (they are *not* on Phase 1/2/5's critical path — those phases
work against the schema and stubs). If the in-game spikes contradict
this recon, Phase 0 reopens — that is what the probe patch is for.

## 8. New risks / changes to later phases
- **R1 (new): build environment is a first-class dependency.** Phase 1's
  CI job `fork-build-and-test` must be stood up on a capable runner
  *during* Phase 1, not discovered in Phase 3. Added to Phase 1 scope
  in the plan update.
- **R2: `do_turn.cpp` split.** Upstream has already split `game.cpp`
  once; hook code must live in `src/voxel_ipc/` with one-line call
  sites in `do_turn.cpp`/`game.cpp`/`sdltiles.cpp` so rebases stay
  cheap (plan §9 principle, now with the third file named).
- **R3: 0.9's compose step** needs a full Tilesets checkout + libvips
  on the build machine; Phase 5's pipeline inherits this requirement.
- No change to Phase 1–8 scope, estimates, or the critical path
  (0 → 1 → 3 → 4) beyond R1's CI note.
