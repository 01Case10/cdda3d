# Hook-point catalogue (Phase 0)

Extracted mechanically from CDDA `master` @ `6a3e59e08842de34ff3f111c7f098abeffb708a5`

> **Base note (2026-11-23):** the fork base moved to CDDA **0.I-1**
> (release tag `0.I-1`, `7b2efa5c`) per plan.md Addendum 12. This
> catalogue was re-verified against the 0.I-1 tree (31/32 hooks
> found, 28 moved-only; full report
> `~/workspace/cdda-stable/reverify/REPORT.md`). The file/line
> entries below remain the master-pin extraction; H3's target
> changed shape (free `do_turn()` on 0.I-1 — see
> `fork/hooks/hook_patches.md`).
by `tools/phase0/extract_hooks.py`. A MISS means the pattern did not
match at that file/line shape — see docs/recon.md for the resolution.

## Send boundary

- `src/do_turn.cpp:536` — primary send boundary candidate: fires once per completed player turn (definition lives in do_turn.cpp, NOT game.cpp — Phase 0 finding)
  - `bool game::do_turn()`
- `src/game.cpp:3364` — game::draw(ui_adaptor&) — redraw point; panels+terrain drawn here
  - `void game::draw( ui_adaptor &ui )`
- `src/game.h:221` — do_turn declaration
  - `bool do_turn();`
## Snapshot read

- `src/map.h:1045` — terrain id per tile
  - `ter_id ter( const tripoint_bub_ms &p ) const;`
- `src/map.h:1020` — furniture id per tile (takes tripoint by value)
  - `furn_id furn( tripoint_bub_ms p ) const;`
- `src/map.h:1614` — trap per tile
  - `const trap &tr_at( const tripoint_bub_ms &p ) const;`
- `src/map.h:1660` — FULL field map per tile (type->entry)
  - `const field &field_at( const tripoint_bub_ms &p ) const;`
- `src/map.h:1424` — item stack per tile
  - `map_stack i_at( const tripoint_bub_ms &p );`
- `src/map.h:1811` — graffiti flag
  - `bool has_graffiti_at( const tripoint_bub_ms &p ) const;`
- `src/submap.h:439` — uppermost item helper — lives on the submap maptile (submap.h), not map
  - `const item &get_uppermost_item() const {`
## Light / FOV

- `src/map.h:585` — per-tile lit_level from visibility cache
  - `lit_level apparent_light_at( const tripoint_bub_ms &p, const visibility_variables &cache ) const;`
- `src/map.h:2520` — recompute visibility cache per z
  - `void update_visibility_cache( int zlev );`
- `src/lightmap.h:96` — lit_level enum definition
  - `enum class lit_level : uint8_t {`
## Delta dirty set

- `src/map.cpp:550` — coarse invalidation hook (per z)
  - `void map::invalidate_map_cache( const int zlev )`
- `src/map.h:526` — seen-cache dirty hook
  - `void set_seen_cache_dirty( const tripoint_bub_ms &change_location );`
- `src/map.h:517` — transparency-cache dirty hook
  - `void set_transparency_cache_dirty( int zlev );`
- `src/map.cpp:9170` — bubble recentering — snapshot epoch event
  - `void map::shift( const point_rel_sm &sp )`
## UI capture

- `src/sdltiles.cpp:151` — SDL backend's single composited texture (Route A capture point)
  - `static SDL_Texture_Ptr display_buffer;`
- `src/game.cpp:3387` — terrain draw — suppressed in frontend mode (Route A)
  - `draw_ter();`
- `src/game.cpp:3404` — sidebar/message panels drawn after terrain
  - `draw_panels( true );`
- `src/cursesport.cpp:21` — catacurses window impl (Route B capture layer)
  - `* The struct WINDOW is the base. It acts as the normal curses window, having`
## Input inject

- `src/input_context.h:427` — raw input entry the injector feeds
  - `input_event get_raw_input();`
- `src/input_context.h:227` — action dispatch after raw input
  - `* Only registered actions will be returned by `handle_input()`, it's`
## Entities

- `src/character.h:2553` — Character overlay list in game order
  - `std::vector<std::pair<std::string, std::string>> get_overlay_ids() const;`
- `src/monster.h:630` — monster overlay list in game order
  - `std::vector<std::pair<std::string, std::string>> get_overlay_ids() const;`
- `src/creature.h:217` — facing: NONE/LEFT/RIGHT only
  - `enum class FacingDirection : int {`
## Vehicles

- `src/vehicle.h:1555` — game-side absolute part position (pre-rotated mounts)
  - `tripoint_bub_ms bub_part_pos( const map &here, int index ) const;`
- `src/vehicle.h:2560` — continuous facing angle
  - `tileray face;`
- `src/vehicle.cpp:734` — the 15-degree pathing clamp (NOT an orientation quantum)
  - `// clamp to multiples of 15.`
## Tiles draw order

- `src/cata_tiles.cpp:112` — 11 draw layers / 8 multitile keys
  - `static const std::array<std::string, 8> multitile_keys = {{`
## Animation

- `src/animation.cpp:293` — animation dispatch (Phase 8 effects hook)
  - `// avoid segfault from null tilecontext in tests`
## Main loop

- `src/main.cpp:900` — process entry; --frontend=ipc arg parsing lands near here
  - `int main( int argc, const char *argv[] )`
