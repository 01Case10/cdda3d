# PC build for gates 0.1 / 0.2 + the Phase 3 fork tests

One command on Joey's PC:

```powershell
cd <your cdda-voxel clone>\tools\pc-build
.\Build-CddaVoxel.ps1
```

That is the whole interface. Everything below is what it does and what
to expect.

## Playing the fake engine (Phase 11 sim-server)

One command builds the Rust side, starts the sim-server (the fake
engine), and opens the renderer window connected to it:

```powershell
cd <your cdda-voxel clone>
powershell -ExecutionPolicy Bypass -File tools\pc-build\Launch-Sim.ps1
```

Options: `-Scene <name>` (default `street_block`), `-Seed <n>`,
`-Record <path>` (writes a replayable session log), `-SkipBuild`,
`-DebugBuild`, `-Headless` (smoke mode: scripted keys, no window).
The bash twin `launch-sim.sh` takes the same options with `--` flags.
Play with the game keys (hjkl / yubn / arrows, o/c, s, g, d, i, &,
;, ., >/<, Esc); WASD also move (w/a are movement aliases, s = smash,
d = drop — they are game keys, not camera keys). On the map, Q/E yaw
the camera and PgUp/PgDn step the cutaway level — but while a menu
is open, EVERY key belongs to the game: q backs out of the menu (it
does not touch the camera), PgUp/PgDn page the menu's list, and Esc
closes the menu. Mouse-drag orbits; close the window to end the
session. The camera follows the player. (Key trouble? Run with the
environment variable CDDA_VOXEL_KEY_DEBUG=1 to log every key event,
the sim mode it routed under, and the decision, to stderr.) With the
UltiCa export in assets/ultica_run1 the live world renders textured
with furniture props, voxel trees, entity billboards (the player
included) and the sim's own menus (i opens the inventory in-window);
without it, it renders the coarse-colour world. (The 2026-11-25
fix round seated all of these in the windowed path — before it,
the window showed the textured coarse world with no player, no
menus, and dead movement keys.)

## Why this exists

Gates 0.1 (the fork's curses/text build compiles and runs) and 0.2 (the
baseline `cata_test` run is recorded) are the master blocker: they need
a build-capable machine, and the recon VM (1.5 CPU / 6 GB) cannot
compile CDDA's ~1,500 translation units. Everything in-game inherits
them — Phase 3's first real compile of `fork/src/voxel_ipc/`, gates
3.2/3.13/3.15/3.16/3.19/3.20, 4.13/4.14, 5.10's TileDefs form, 6.4's
game-side half, Phase 8's in-game halves, and Phase 2h's live-data
seating.

## What it does, in order

1. **Deps** — finds MSYS2 (installs nothing itself; if MSYS2 is absent
   it stops and says so), then inside a UCRT64 shell pacman-installs
   only what a curses + tests build needs: gcc, cmake, ninja, ccache,
   ncurses, zlib, gettext, pkgconf, git, python. **No SDL3** — the
   tiles build is out of scope for this project (Phase 0 clarification);
   `TILES=OFF` never links it.
2. **Clone** — a *full* checkout of `CleverRaven/Cataclysm-DDA` at the
   pinned commit `7b2efa5cea38e4d4d97dd0e63b28b9148623da59` into
   `Documents\cdda-voxel-build\Cataclysm-DDA`. The repo's
   `fork-src/cdda` is a sparse recon checkout and cannot build or run
   tests (no `data/`, `lang/`, full `tests/`), so it is not used. If
   `git clone` fails, it falls back to the commit tarball over HTTPS.
   The tree must be pristine for the baseline; the script resets only
   the paths it owns and refuses to build a "baseline" on other local
   modifications.
3. **Baseline (0.1/0.2)** — CMake + Ninja, `RelWithDebInfo`,
   `-DTILES=OFF -DCURSES=ON -DSOUND=OFF -DTESTS=ON`, builds the
   `cataclysm` and `cata_test` targets only, in `build-baseline/`.
   - 0.1: runs the binary (`--help`, falling back to `--jsonverify`)
     from the source dir and records the result in `results/gate-0.1.txt`.
   - 0.2: runs the **full** `cata_test` suite (fixed seed, lexical
     order, JUnit reporter) and records `results/baseline-record.txt`,
     `baseline-summary.txt` and `baseline-failed.txt`. Upstream
     failures, if any, are part of the baseline — 0.2 passes by being
     recorded, not by being green.
4. **Fork (Phase 3)** — stages `fork/src/voxel_ipc/` and
   `fork/tests/voxel_exporter_tests.cpp` into the checkout, patches
   `src/CMakeLists.txt` (see "Two build-system facts" below), builds in
   a **separate** `build-fork/` dir, then:
   - `cata_test "[voxel]"` — the in-game form of gates 3.1 / 3.3 / 3.5 /
     3.5b / 3.6 / 3.7 / 3.8 (this is also the first compile ever of
     `voxel_real_adapter.cpp` against the real game headers).
   - Full `cata_test` again, and the failed-test set is diffed against
     the baseline: **gate 3.19 = no new failures** (`results/comparison.txt`).

Results and per-run logs land in `Documents\cdda-voxel-build\results`
and `\logs`. Exit codes: 0 = all gates passed, 2 = environment,
3 = build failure, 4 = test gate failure.

## Options

| Switch | Effect |
|---|---|
| `-Stage Baseline` / `-Stage Fork` | run one stage only |
| `-Jobs 8` | cap parallel compile jobs (default: cores, capped at 12 for RAM) |
| `-SkipDeps` | don't run pacman (toolchain already installed) |
| `-SkipFullSuite` | quick mode: builds + 0.1 + `[voxel]` only; no 0.2 record, no 3.19 diff |
| `-Clean` | delete the build dirs first (full recompile) |
| `-DryRun` | print the plan, touch nothing |
| `-WorkDir <path>` | move the clone/builds/results (default `Documents\cdda-voxel-build`) |
| `-Msys2Root <path>` | MSYS2 install location (default: `C:\msys64`) |

## Two build-system facts the script works around

Both were found by reading the pinned sources, not upstream docs:

1. **The source glob is non-recursive.** `src/CMakeLists.txt` collects
   `src/*.cpp` only, and `tests/CMakeLists.txt` globs `tests/*.cpp`
   (that one works in our favour — the voxel cases are picked up
   automatically). `patch_cmake.py` appends `src/voxel_ipc/*.cpp`
   after the glob, idempotently, and fails loudly if the anchor text
   ever changes upstream.
2. **FlatBuffers version split.** The generated IPC header asserts
   FlatBuffers **24.3.25**, but upstream vendors **1.12.0** under
   `src/third-party` for its own TUs (flexbuffers etc.). The script
   stages the 24.3.25 headers from `tools/stub-server/third_party`
   into `src/voxel_ipc/flatbuffers/` and gives only the voxel sources
   `src/voxel_ipc` as an extra include dir (per-source
   `INCLUDE_DIRECTORIES` in the patch), so the generated header's
   quoted include resolves to 24.3.25 and no game TU ever sees it.
   If the first fork compile dies with a FlatBuffers version
   `static_assert` or a `flatbuffers/flatbuffers.h not found` error,
   this isolation is the first thing to check.

## What it deliberately does NOT do

- **Hooks H1–H7 are not applied.** They are prose in
  `fork/hooks/hook_patches.md`, not applyable patches, and the
  `[voxel]` Catch2 cases do not need them (they link the exporter
  directly). The hooks are only needed for *live* IPC — a running game
  serving the engine — which is the step after this one. Note H6 as
  written calls `voxel_ipc::to_input_event(...)`, a function that does
  not exist yet in `fork/src/voxel_ipc/`; writing it is first-compile
  development work, not scripting. H7 (trap accessor) is likewise
  optional for these gates: the real adapter currently emits no trap
  defs and no `[voxel]` case checks them.
- **The IPC server is POSIX-only.** `voxel_server.cpp`'s socket code
  is guarded to Linux/Unix and returns `false` on Windows (named-pipe
  transport is Phase 8 packaging scope). It *compiles* on Windows —
  which is all these gates need — but `--frontend=ipc` cannot serve
  the engine from a Windows build yet. Live-data seating (2h) still
  needs either that transport or a Linux build machine.
- **No SDL3, no tiles build, ever** (Phase 0 decision).

## Expectations and known risks

- First baseline build: tens of minutes on the 9800X3D (the full suite
  run is of the same order). ccache is on, so rebuilds after a fix are
  much faster. Disk: plan on ~25 GB for clone + two build trees.
- Upstream compiles with `-Werror`. If a newer UCRT gcc than upstream
  CI's turns a new warning into an error in *upstream* files, the
  build stops there — that is a toolchain-drift finding to record,
  not something this script papers over.
- First fork compile will very likely need small fixes in
  `voxel_real_adapter.cpp` — `docs/phase3-fork.md` §4 lists the known
  candidates (string-id conversion spelling, `vpart_info::has_flag`
  form, accessor names). Fix them in `fork/src/voxel_ipc/` in this
  repo (not in the build checkout — the fork stage re-copies from
  here), then re-run `-Stage Fork`.

## After it passes

Send back `results/baseline-record.txt`, `results/voxel-summary.txt`
and `results/comparison.txt` (or just say it passed) — that closes
0.1/0.2, converts Phase 3's conditional gate, and unblocks the
in-game halves listed at the top. The plan's Phase 0/3 records get
updated from those files.
