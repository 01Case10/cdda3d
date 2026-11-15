# Fork probe patch — Phase 0 in-game spikes (apply once the fork builds)

These are the exact, minimal, additive instrumentation points the Phase 0
harness results in `docs/recon.md` are based on. They are written against
CDDA `6a3e59e0` file/symbol locations from `docs/hook-points.md`. None of
them change simulation behaviour; all logging is behind the environment
variable `CDDA_VOXEL_PROBE=1`. Apply them on the `voxel-frontend` branch,
run the scripted sequences, then remove the log lines (the hook *points*
stay as the Phase 3 design).

## 0.3 Send-boundary probe
In `src/game.cpp`, at the exit of `game::do_turn()` and at the end of
`game::draw(ui_adaptor&)` after `draw_panels()`:

```cpp
if( std::getenv( "CDDA_VOXEL_PROBE" ) ) {
    fprintf( stderr, "VOXEL_BOUNDARY turn_done seq_check\n" );
}
```
Scripted run: walk 10 turns, start a multi-turn craft, idle-wait 5 turns
while a monster moves. Pass = every state change is followed by a boundary
line before the next input prompt, and boundary lines per player turn stay
≤ a small bound (no storm). This replaces v1's "exactly one per turn",
which free actions and multi-turn activities falsify.

## 0.4 Visible-tile probe
At the same boundary, iterate the view (radius ≤ `MAX_VIEW_DISTANCE`,
z in the cutaway window) calling `map::ter/furn/tr_at` and FNV-1a hash the
id stream per z. Print `VOXEL_TILES z=<z> n=<count> hash=<hex>`. Two runs
of the same seeded world + script must print identical hashes.

## 0.5 Light/visibility probe
After `map::update_visibility_cache(z)`, sample three known tiles (lit
room interior, open field at night, a tile seen earlier but now out of
sight) via `map::apparent_light_at()` and print the `lit_level`. Expect
`LIT`/`BRIGHT`, `DARK`/`LOW`, `MEMORIZED` respectively.

## 0.6 Route A capture (in-game half)
In `src/sdltiles.cpp` at the present point, when `--frontend=ipc` is set:
skip the `draw_ter()` result by clearing the `w_terrain` rect in the
`display_buffer` before compositing, then `SDL_RenderReadPixels` the
`display_buffer` into a CPU buffer and write a PNG. Measure and log the
readback ms per frame. (The standalone spike in
`tools/phase0/spike_ui_capture.cpp` already proves the compositing shape
and the CPU-side copy cost; this step measures the real GPU stall.)

## 0.7 Input injection probe
In the fork's input path, when `CDDA_VOXEL_PROBE=1`, push one synthetic
event (`i`) into the same queue `input_context::get_raw_input()` drains,
before the first real input. Pass = the inventory opens as if `i` was
typed, with no other behavioural difference.

## 0.8 Headless run
Run the tiles build with the window hidden (Spike B route 1) and,
separately, the curses build with the Phase 0 null-backend prototype
(Spike B route 2, chosen primary — see recon §Decisions). Pass = a
scripted 50-turn sequence completes with no visible window and UI frames
still produced (route 2) or captured (route 1).
