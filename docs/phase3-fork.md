# Phase 3 — game-side export in the fork: how `fork/` maps into a real CDDA checkout

**Status (2026-11-15): CONDITIONAL gate at harness level.** The exporter
core is built and gate-tested against a mock world through the *real*
Phase 1 FlatBuffers schema. The fork has **not** been compiled anywhere:
tests 0.1/0.2 (curses build + baseline `cata_test`) have not run on any
machine yet, and per the Phase 0 record they are a hard precondition for
Phase 3's full gate. Nothing in this document claims an in-game pass.

## 1. File map (repo `fork/` → CDDA checkout)

| Repo path | Lands at | Role |
|---|---|---|
| `fork/src/voxel_ipc/voxel_types.h` | `src/voxel_ipc/voxel_types.h` | Wire value types; `lit_level`/`FacingDirection` mirrors (7 / 3 values, exact) |
| `fork/src/voxel_ipc/voxel_adapter.h` | `src/voxel_ipc/voxel_adapter.h` | `IGameAdapter` — the only seam between core and game |
| `fork/src/voxel_ipc/voxel_exporter.{h,cpp}` | same | TileDefs/Snapshot/Delta serializers, dirty-submap diff, seq/epoch, snapshot hash |
| `fork/src/voxel_ipc/voxel_server.{h,cpp}` | same | Framing (u32 LE + Envelope, 32 MiB cap, CDDA id check), bounded outbox, Unix-socket server, SeqTracker, heartbeat |
| `fork/src/voxel_ipc/voxel_input.{h,cpp}` | same | Engine `Input` → queued game events |
| `fork/src/voxel_ipc/voxel_frontend.{h,cpp}` | same | Singleton the hooks call; handshake, boundary flush, slow-consumer policy, clean shutdown |
| `fork/src/voxel_ipc/voxel_real_adapter.{h,cpp}` | same | **Fork-only TU**: reads the real game via the verified APIs (below). Never compiled in the harness |
| `fork/src/voxel_ipc/cdda_ipc_generated.h` | same | Checked-in flatc 24.3.25 output for `schema/cdda_ipc.fbs` (`--cpp --gen-object-api`), same policy as the Phase 1 stub |
| `fork/hooks/hook_patches.md` | (applied, not copied) | H1–H7: flag parse, build lines, send boundaries, dirty hooks, shift hook, input injection, trap accessor |
| `fork/tests/voxel_exporter_tests.cpp` | `tests/voxel_exporter_tests.cpp` | Catch2 `TEST_CASE`s for 3.1/3.3/3.5/3.5b/3.6/3.7/3.8, run via `cata_test "[voxel]"` |

Vendored FlatBuffers headers: reuse `tools/stub-server/third_party/`
(flatbuffers 24.3.25 headers) on the include path, or the fork's own
vendoring when it lands; the exporter includes only
`cdda_ipc_generated.h`, which includes `flatbuffers/flatbuffers.h`.

## 2. Build-system lines to add

Makefile (primary upstream build): add the module objects next to the
other `src/` objects —

```make
SOURCES += $(wildcard src/voxel_ipc/*.cpp)
CXXFLAGS += -Isrc/voxel_ipc -Itools/stub-server/third_party  # adjust to fork vendoring
```

CMake: append `src/voxel_ipc/*.cpp` glob to the cataclysm sources and
the same include directories. No new link dependencies (POSIX sockets
only; FlatBuffers is header-only as vendored).

## 3. Real APIs the adapter uses (all grep-verified 2026-11-15 @ 6a3e59e0)

> **Base note (2026-11-23):** the fork base moved to CDDA **0.I-1**
> (release tag `0.I-1`, `7b2efa5c`) per plan.md Addendum 12. Every
> API below was re-verified against the 0.I-1 tree
> (`~/workspace/cdda-stable/reverify/REPORT.md`); the five
> master-only APIs are restored by `fork/hooks/backport_patches.md`
> (B1–B4). File:line numbers below remain the master-pin numbers.

- Tiles: `map::ter(const tripoint_bub_ms&)` map.h:1045 ·
  `map::furn(tripoint_bub_ms)` **by value** map.h:1020 ·
  `map::tr_at` map.h:1614 · `map::field_at` (full field map) map.h:1660 ·
  `map::i_at` map.h:1424 · `map::maptile_at` map.h:662 →
  `const_maptile::get_uppermost_item()` submap.h:439 ·
  `map::has_graffiti_at` map.h:1811
- Light: `map::update_visibility_cache(int)` map.h:2520 ·
  `map::apparent_light_at(p, cache)` map.h:585 ·
  `map::get_visibility_variables_cache()` map.h:2524 ·
  `lit_level` lightmap.h:96 (exactly DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT,
  MEMORIZED, BLANK)
- Defs: `mapdata::terrain_data/furniture_data.get_all()` (generic_factory,
  used the same way in ammo.cpp:152/170) · `get_all_field_types()`
  field_type.h:58 · `MonsterGenerator::generator().get_all_mtypes()`
  monstergenerator.h:59/82 · `item_controller->get_generic_factory().get_all()`
  item_factory.h:100 · `vehicles::parts::get_all()` veh_type.h ·
  transforms: `ter_t::open/close` (ter_str_id), bash → `map_ter_bash_info::ter_set` /
  `map_furn_bash_info::furn_set` (mapdata.h) · connect bitsets
  `connect_groups/connect_to_groups/rotate_to_groups` (mapdata.h:637-641,
  `NUM_TERCONN=256`)
- Entities: `creature_tracker::get_monsters_list()` + `temporary_id()`,
  `game::all_npcs()`, avatar — from creature tracking, **not** submaps ·
  `Character::get_overlay_ids()` character.h:2553 /
  `monster::get_overlay_ids()` monster.h:630 (order preserved verbatim) ·
  `Creature::facing` (creature.h:310; NONE/LEFT/RIGHT only, creature.h:217) ·
  `Creature::hp_percentage()` / `pos_bub(map)`
- Vehicles: `map::get_vehicles()` → `vehicle::bub_part_pos(map, index)`
  vehicle.h:1555 (game-side pre-rotated absolute positions) ·
  `vehicle::face.dir()` → `units::to_degrees` (float; never quantized) ·
  `vehicle_part::mount`, `health_percent()`, `open`, `is_broken()`
- Memory: `avatar::get_memorized_tile(tripoint_abs_ms)` avatar.h:165 →
  `memorized_tile::get_ter_id()/get_dec_id()` (map_memory.h) — terrain/
  furniture string ids only; the exporter emits them as a separate layer
- Input: injection at `input_context::get_raw_input()` input_context.h:427,
  events shaped as `input_event` (input_enums.h:63)
- Hooks: `game::do_turn()` do_turn.cpp:536 · `game::draw()` game.cpp:3364
  (`draw_ter()` :3387, `draw_panels(true)` :3404) ·
  `map::invalidate_map_cache(z)` map.cpp:550 ·
  `set_seen_cache_dirty` map.h:526 · `set_transparency_cache_dirty`
  map.h:517/525 · `map::shift` map.cpp:9170 · `main()` arg handlers
  main.cpp:286 (`cli_opts` :276)

## 4. Known gaps / first-compile fix candidates (flagged, not hidden)

1. **Trap registry**: upstream exposes no public `get_all()` for traps
   (`trap_factory` is file-local to trap.cpp). Hook H7 adds a 3-line
   accessor; until it is applied, per-tile trap ids export exactly but
   trap *defs* are emitted from that accessor's TU. Test 3.1 in-game
   depends on H7.
2. **Connect-group names**: bitset indices are exact, but the pinned
   source has no index→name reverse lookup (`get_connect_group` maps
   name→group). **Mirror wired (2026-11-15 follow-up):** hook H8
   records each name at `connect_group::load` into the registry in
   `voxel_ipc/voxel_refs.*`, and the adapter's
   `connect_group_table()` reads it back; an unrecorded index keeps
   the stable `group_<index>` placeholder. Harness gate 3.22.
3. **Seen float**: no public per-tile `seen_cache` getter exists in the
   pinned source. The adapter quantizes from `lit_level` (BLANK=0,
   MEMORIZED=64, visible=255). Test 3.5's seen-tolerance check in-game
   will show whether that suffices or a cache accessor hook is needed.
4. **fg/bg sprite refs**: **wired (2026-11-15 follow-up).** The
   tileset crate's `fg_refs` tool derives the table from the composed
   `tile_config.json` (primary sprite per entry: highest weight,
   ties by order; absent slot = -1) and writes a TSV; the adapter
   loads it once from the `CDDA_VOXEL_FG_REFS` env path into
   `voxel_ipc::FgBgRefs` and fills every def's refs by string id in
   a post-pass. No table = every def keeps -1 (nothing invented).
   Refs are section-local composed indices, matching the schema's
   single-int fields. Harness gate 3.21; tool gate in the tileset
   crate's `fg_refs_tool` test.
5. **`TileChange` carries no `displayed_field`** (schema observation,
   reported to the parent — schema/ is read-only for Phase 3): after a
   delta, the engine cannot know the game's newly displayed field from
   the delta alone; it can recompute from the full field list only by
   guessing the game's rule. Adding `displayed_field` to `TileChange`
   is an additive, non-breaking schema change and is recommended.
6. Real-adapter details most likely to need adjustment at first compile
   (none verifiable without building): `ter_t/furn_t` string-id →
   int-id conversion spelling (`d.id.id()`), `vpart_info::has_flag`
   string form for the ROOF flag, `MAPSIZE_X/Y` loop bounds in
   `memory_tiles()`, and `game`/`avatar` accessor names
   (`get_game()`/`get_avatar()`).

## 5. Exact unblock steps (build-capable machine: Joey's PC or ≥4-core/≥16 GB CI)

**Scripted (2026-11-15): `tools/pc-build/Build-CddaVoxel.ps1`** — one
command on Joey's PC does everything below for the baseline + the
`[voxel]` cases (deps via MSYS2 UCRT64, full clone at the pinned
commit, separate baseline/fork build dirs, JUnit-recorded baseline,
3.19 failure-set diff). It applies the §2 build lines via
`tools/pc-build/patch_cmake.py` (the pinned `src/CMakeLists.txt` glob
is non-recursive, so the module must be appended explicitly) and
stages the FlatBuffers 24.3.25 headers inside `src/voxel_ipc/` —
upstream vendors 1.12.0, which the generated header's static_assert
rejects. Hooks H1–H7 are NOT applied by the script: the Catch2 cases
link the exporter directly and do not need them; the hooks are for
live IPC, the step after. See `tools/pc-build/README.md`. The manual
steps below remain the reference for what the script does.

```sh
# 0. Precondition — Phase 0 tests 0.1/0.2 (curses build only; no SDL3)
cmake -DTILES=OFF -DCURSES=ON -B build && cmake --build build -j
./build/cataclysm --help            # 0.1: launches
./build/tests/cata_test             # 0.2: record baseline pass rate
# 1. Apply the fork tree
cp -r fork/src/voxel_ipc src/voxel_ipc
cp fork/tests/voxel_exporter_tests.cpp tests/
# apply hook_patches.md H1-H7; add the build lines from §2
# 2. Build + run the voxel cases
cmake --build build -j
./build/tests/cata_test "[voxel]"  # 3.1/3.3/3.5/3.5b/3.6/3.7/3.8 in-game
./build/tests/cata_test             # 3.19: no new failures vs the 0.2 baseline
# 3. Replay test (Phase 3 exit criteria): seeded world + scripted input
#    via the engine's proto-client, twice; compare Exporter::hash_snapshot
#    at each checkpoint (3.14 in-game form).
```

## 6. Harness evidence (this environment)

`tools/fork-harness/run.sh` builds the exporter core + mock worlds and
runs the gate checks; `tools/fork-harness/rust-decode` decodes the
harness snapshot with the real Rust `proto` crate. Results are recorded
in the Phase 3 section of plan.md (by the parent) and in the Phase 3
agent's report: 3.1 (harness form), 3.3, 3.4, 3.5, 3.5b, 3.6, 3.7, 3.8,
3.9 (1,100 turns × 10 seeds), 3.9b, 3.10, 3.11, 3.12, 3.14, 3.17, 3.18
(measured), plus schema round-trip and input-order checks.
