# Fork hook patches — Phase 3 (ready to apply on the `voxel-frontend` branch)

All logic lives in `src/voxel_ipc/` (see `docs/phase3-fork.md` for the
file map). Every hook below is additive and a no-op unless
`--frontend=ipc` was passed: one or two lines at a verified call site
(CDDA `master` @ `6a3e59e0`; line numbers from `docs/hook-points.md`).
Apply order: H1 (flag) → H2 (adapter construction) → H3–H6 (hooks) →
H7 (trap accessor the real adapter needs) → H8 (connect-group name
mirror the real adapter reads).

**Base change (2026-11-23, plan.md Addendum 12):** the fork base is
now CDDA `0.I-1` (tag `0.I-1`, commit
`7b2efa5cea38e4d4d97dd0e63b28b9148623da59`), not the master snapshot
above. The line numbers in H1–H8 are still the master-pin numbers —
the re-verification (`~/workspace/cdda-stable/reverify/REPORT.md`)
found H1, H2, H4–H8 unchanged in substance on the new base, with
targets moved only — **except H3, which is updated below** (on
0.I-1 the send boundary is a free function, not a `game` member).
Additionally, the backports in `backport_patches.md` (B1–B4:
public def factories, `map::is_roofed`, moonlight, weather tints —
master APIs the stable branch never got) **must be applied for the
real adapter to compile on the new base**.

## H1 — `src/main.cpp`: `--frontend=ipc` flag

In `cli_opts` (main.cpp:276) add two fields:

```cpp
bool frontend_ipc = false;
std::string ipc_path;
```

In `first_pass_arguments` (near `--seed`, main.cpp:294) add:

```cpp
{
    "--frontend", "<ipc>",
    "Run with a frontend over IPC: --frontend=ipc --ipc-path=<path>",
    section_default, 1,
    [&result]( int, const char **params ) -> int {
        if( std::string( params[0] ) == "ipc" ) {
            result.frontend_ipc = true;
            voxel_ipc::set_frontend_requested( "" );
        }
        return 1;
    }
},
{
    "--ipc-path", "<path>",
    "Unix socket path for --frontend=ipc",
    section_default, 1,
    [&result]( int, const char **params ) -> int {
        result.ipc_path = params[0];
        voxel_ipc::set_frontend_requested( params[0] );
        return 1;
    }
},
```

And where the game object is initialised (after options load), once:

```cpp
if( cli.frontend_ipc ) {
    static voxel_ipc::RealAdapter voxel_adapter( get_map() );
    voxel_ipc::Frontend::instance().enable(
        voxel_adapter,
        cli.ipc_path.empty() ? voxel_ipc::frontend_ipc_path() : cli.ipc_path );
}
```

Frontend mode keeps UI drawing (Phase 4 redirects it); native map
drawing is suppressed in H4. The original tiles/curses behaviour is
untouched when the flag is absent.

## H2 — build system

Makefile/CMake: add `src/voxel_ipc/*.cpp` to the game sources and the
vendored FlatBuffers headers + `cdda_ipc_generated.h` to the include
path (same vendoring as `tools/stub-server/third_party`). Exact lines
are in `docs/phase3-fork.md`.

## H3 — `src/do_turn.cpp`: send boundary (PRIMARY)

**Updated for the 0.I-1 base (2026-11-23).** On 0.I-1 the send
boundary is the **free function** `bool do_turn()` — defined at
do_turn.cpp:455, declared in do_turn.h:6, a `friend` of `game`
(game.h:192), and called from the main loop at main.cpp:864
(`while( !do_turn() ) {}`). It is NOT a `game` member on this base.
(For the record: on the old master pin it was `game::do_turn()`,
defined at do_turn.cpp:536 (master) — NOT game.cpp, the original
Phase 0 finding.)

At every `return` point of `do_turn()` — in practice: wrap the
body end and the early `turn_handler::cleanup_at_end()` returns
(do_turn.cpp:458 and do_turn.cpp:594 on 0.I-1; the final
`return false;` is at do_turn.cpp:779) — call:

```cpp
voxel_ipc_hook_send_boundary(); // -> Frontend::on_send_boundary(view of u)
```

Implemented as a tiny free function in `voxel_frontend.cpp`'s fork glue
(it builds the `ViewSpec` from `u.pos_bub()` and the cutaway setting) so
the hook site stays one line. This is the boundary test 0.3 verified:
every state change is flushed before the next input is consumed.

## H4 — `src/game.cpp`: redraw-point flush + map-draw suppression

At the end of `game::draw(ui_adaptor&)` (game.cpp:3364), after
`draw_panels(true)` (:3404):

```cpp
voxel_ipc_hook_send_boundary();
```

And in frontend mode only, skip the terrain draw (the engine renders
the world; UI panels/menus still draw — Phase 4 captures them):

```cpp
if( !voxel_ipc::frontend_requested() ) {
    draw_ter(); // game.cpp:3387, unchanged otherwise
}
```

## H5 — `src/map.cpp` / `src/map.h`: dirty-submap collection

These hooks only ADD the submap/z to the exporter's dirty set; the diff
at the send boundary is the mechanism (no per-tile hook exists).

- `map::invalidate_map_cache(const int zlev)` (map.cpp:550), at entry:
  ```cpp
  voxel_ipc::Frontend::instance().on_invalidate_map_cache( zlev );
  ```
- `map::set_seen_cache_dirty(const tripoint_bub_ms&)` (map.h:526):
  ```cpp
  voxel_ipc::Frontend::instance().on_seen_dirty( change_location.x(), change_location.y(), change_location.z() );
  ```
- `map::set_transparency_cache_dirty(int zlev)` (map.h:517) and the
  `tripoint_bub_ms` overload (map.h:525):
  ```cpp
  voxel_ipc::Frontend::instance().on_transparency_dirty( zlev );
  ```
- `map::shift(const point_rel_sm&)` (map.cpp:9170), at exit:
  ```cpp
  voxel_ipc::Frontend::instance().on_map_shift(); // epoch bump + resync
  ```
- Creature/vehicle change points: `creature_tracker::add/remove/
  update_pos` and `vehicle` part install/remove call
  `on_creatures_changed()` / `on_vehicles_changed()` respectively.
  (Entity/vehicle diffs are also cheap enough to run unconditionally at
  a boundary; the flags exist so the common no-change turn skips them.)

## H6 — `src/input_context` path: input injection

`input_context::get_raw_input()` (input_context.h:427): before draining
the platform queue, drain the frontend queue first:

```cpp
if( voxel_ipc::QueuedInput q; voxel_ipc::Frontend::instance().input().next( q ) ) {
    return voxel_ipc::to_input_event( q ); // in voxel_real_adapter glue
}
```

`to_input_event` builds an `input_event` (input_enums.h:63): KeyCode →
`input_event(keycode, input_event_t::keyboard_code)` with modifiers
mapped to `keymod_t`; Text → `keyboard_char` events with `text` set;
mouse kinds → `input_event_t::mouse` with `mouse_pos` converted from
display-buffer pixels to cells (cell = pixel / font size, the same
conversion the SDL backend performs). Because injection happens at
`get_raw_input()`, all `input_context` action resolution, keybindings
and menus work unchanged (recon test 0.7 shape).

## H7 — `src/trap.cpp`: trap registry accessor (3 lines)

`trap_factory` is file-local to trap.cpp and upstream exposes no public
`get_all()` for traps in the pinned source (verified 2026-11-15). The
real adapter's trap TileDefs need one, so the fork adds, next to
`trap_factory` in trap.cpp + a declaration in trap.h:

```cpp
const std::vector<trap> &get_all_traps_for_voxel() { return trap_factory.get_all(); }
```

This is the ONLY upstream-file logic addition beyond one-line hooks,
and it exists because the alternative (inventing an accessor that
already exists) was checked and does not.

## H8 — `src/mapdata.cpp`: connect-group name mirror (1 line + include)

`connect_group::load` (mapdata.cpp:355 in the pinned source) is the
one place where a group's name and its bitset index are both known:
`get_connect_group` maps name -> group, and upstream has no
index -> name lookup (verified 2026-11-15). The real adapter's
`connect_group_table()` needs the names, so the fork mirrors them
at load. Add the include at the top of mapdata.cpp:

```cpp
#include "voxel_ipc/voxel_refs.h"
```

and one line in `connect_group::load`, after the NUM_TERCONN
overflow check, before `ter_connects_map[...] = result;`:

```cpp
voxel_ipc::record_connect_group_name( static_cast<int>( result.index ), result.id.str() );
```

Unlike H3–H6 this line is NOT gated on `--frontend=ipc`: group
loading happens during data init, and the mirror is inert data
(a name table nothing reads unless the frontend runs). An index
the mirror never recorded still exports as the stable placeholder
`group_<index>` — the indices themselves come from the game's own
bitsets and are exact with or without H8.
