# Fork backport patches — master APIs restored on the 0.I-1 base

Per plan.md Addendum 12 (Joey's decision, 2026-11-23), the fork base
is the latest stable release — CDDA `0.I-1` (tag `0.I-1`, commit
`7b2efa5cea38e4d4d97dd0e63b28b9148623da59`) — not the `master`
snapshot the adapter in `src/voxel_ipc/` was written against
(`master` @ `6a3e59e0`). The re-verification
(`~/workspace/cdda-stable/reverify/REPORT.md`) found five master-side
additions the stable branch never got. Rather than adapt the adapter
down and lose the behaviour, the fork backports the missing pieces
from master.

Each backport below is a **verbatim copy from `master` @ `6a3e59e0`**
unless an adaptation point is explicitly flagged. Apply alongside
the hook patches H1–H8 (`hook_patches.md`); order among B1–B4 does
not matter, but all four must be in before the real adapter compiles
on the new base. All line numbers are 0.I-1 lines unless marked
"(master)".

## B1 — def factories public (`src/mapdata.h`, `src/mapdata.cpp`)

**Why:** the adapter's TileDefs enumeration calls
`mapdata::terrain_data.get_all()` / `mapdata::furniture_data.get_all()`
(`voxel_real_adapter.cpp:233/247`). On master the factories are
public (externs at mapdata.h:43-44 (master)). In 0.I-1 they exist
only inside an anonymous namespace at mapdata.cpp:42-48 — and that
namespace contains *only* these two definitions (verified by
reading mapdata.cpp:35-50), so unwrapping it touches nothing else.

**`src/mapdata.h`** — 0.I-1's header has no `generic_factory`
forward declaration at all. Insert after `struct ter_t;`
(mapdata.h:32), before the `NUM_TERCONN` comment (mapdata.h:34) —
verbatim master mapdata.h:38-46:

```cpp
template <typename T> class generic_factory;

namespace mapdata
{

extern generic_factory<ter_t> terrain_data;
extern generic_factory<furn_t> furniture_data;

} // namespace mapdata
```

**`src/mapdata.cpp`** — replace the anonymous-namespace block
(mapdata.cpp:42-48):

```cpp
namespace
{

generic_factory<ter_t> terrain_data( "terrain" );
generic_factory<furn_t> furniture_data( "furniture" );

} // namespace
```

with master's form (master mapdata.cpp:53-61):

```cpp
namespace mapdata
{

generic_factory<ter_t> terrain_data( "terrain" );
generic_factory<furn_t> furniture_data( "furniture" );

} // namespace mapdata

using namespace mapdata;
```

The `using namespace mapdata;` is load-bearing: the other 25 uses
of `terrain_data`/`furniture_data` in mapdata.cpp (the `int_id` /
`string_id` specialisations from mapdata.cpp:54 down) are
unqualified, exactly as on master, and keep resolving through the
using-directive. `generic_factory.h` is already included
(mapdata.cpp:19).

## B2 — `map::is_roofed` (`src/map.h`, `src/map.cpp`)

**Why:** the adapter's sky-exposure rule calls `here_.is_roofed( p )`
(`voxel_real_adapter.cpp:135`). `is_roofed` does not exist anywhere
in 0.I-1 (zero hits tree-wide); master has it at map.h:1238 /
map.cpp:3794 (master). The backport copies declaration, overmap
wrapper, and implementation verbatim.

**Dependencies — each re-verified present in 0.I-1 while writing
this patch:** `level_cache` with `floor_cache_dirty`
(level_cache.h:28) and `floor_cache`
(`cata::mdarray<bool, point_bub_ms>`, level_cache.h:50 — same type
as master level_cache.h:57 (master)); `get_cache_lazy`
(map.h:2158); `has_flag_ter( ter_furn_flag, … )` (map.h:1024) and
`has_flag_furn( ter_furn_flag, … )` (map.h:1029); `veh_at`
(map.h:798) returning `optional_vpart_position`;
`vpart_position::part_with_feature( vpart_bitflags, bool, bool )`
(vpart_position.h:78); `VPFLAG_OPAQUE` (veh_type.h:72) and
`VPFLAG_ROOF` (veh_type.h:111); the flags `TFLAG_TRANSPARENT_FLOOR`,
`TFLAG_NO_FLOOR_WATER`, `TFLAG_NO_FLOOR`, `TFLAG_GOES_DOWN`,
`TFLAG_SUN_ROOF_ABOVE` (all in 0.I-1's `ter_furn_flag` enum,
mapdata.h); `OVERMAP_HEIGHT` (already used in 0.I-1's map.cpp);
`inbounds` (map.h:1761). **No adaptation points:** every identifier
in the copied code resolves in 0.I-1 (see Verification).

**`src/map.h`** — insert after `is_outside`'s declaration
(map.h:1062), verbatim master map.h:1233-1238:

```cpp
        // Returns true if precipitation cannot reach this tile. Walks upward
        // through z-levels looking for any blocking surface: solid floor,
        // TRANSPARENT_FLOOR (glass/ramps/grates), NO_FLOOR_WATER, SUN_ROOF_ABOVE
        // furniture, or vehicle with ROOF/OPAQUE. Uses floor_cache when valid,
        // falls back to direct terrain/furniture/vehicle checks otherwise.
        bool is_roofed( const tripoint_bub_ms &p ) const;
```

and the overmap-coordinate wrapper after the `is_outside`
`tripoint_omt_ms` wrapper (map.h:2490-2492), verbatim master
map.h:2832-2834:

```cpp
        bool is_roofed( const tripoint_omt_ms &p ) const {
            return map::is_roofed( rebase_bub( p ) );
        }
```

**`src/map.cpp`** — insert after `map::is_outside`'s definition
(map.cpp:3308-3316), before `map::is_last_ter_wall` (map.cpp:3318)
— the same relative position the function occupies on master.
Verbatim master map.cpp:3794-3862:

```cpp
bool map::is_roofed( const tripoint_bub_ms &p ) const
{
    if( !inbounds( p ) ) {
        return false;
    }

    // Walk upward through z-levels looking for any surface that blocks
    // precipitation. The first blocking surface found means this tile is roofed.
    for( int z = p.z() + 1; z <= OVERMAP_HEIGHT; ++z ) {
        const tripoint_bub_ms check_pos( p.x(), p.y(), z );
        const level_cache *cache = get_cache_lazy( z );

        // Fast path: use floor_cache when it exists and is clean
        if( cache && !cache->floor_cache_dirty ) {
            if( cache->floor_cache[p.x()][p.y()] ) {
                return true;
            }
            // Transparent floors (glass, ramps, grates) are marked as "no floor"
            // in floor_cache but are physical surfaces that block precipitation
            if( has_flag_ter( ter_furn_flag::TFLAG_TRANSPARENT_FLOOR, check_pos ) ) {
                return true;
            }
            // Water surfaces absorb precipitation before it reaches tiles below
            if( has_flag_ter( ter_furn_flag::TFLAG_NO_FLOOR_WATER, check_pos ) ) {
                return true;
            }
            continue;
        }

        // Fallback: direct terrain/furniture/vehicle checks when cache is
        // missing or dirty. Mirrors build_floor_cache semantics.

        // TRANSPARENT_FLOOR checked first: blocks precipitation even when
        // combined with vertical-transition flags like GOES_DOWN
        if( has_flag_ter( ter_furn_flag::TFLAG_TRANSPARENT_FLOOR, check_pos ) ) {
            return true;
        }

        const bool no_floor = has_flag_ter( ter_furn_flag::TFLAG_NO_FLOOR, check_pos );
        const bool goes_down = has_flag_ter( ter_furn_flag::TFLAG_GOES_DOWN, check_pos );

        if( !no_floor && !goes_down ) {
            return true;
        }

        if( has_flag_ter( ter_furn_flag::TFLAG_NO_FLOOR_WATER, check_pos ) ) {
            return true;
        }

        // SUN_ROOF_ABOVE on furniture at z-1 overrides no-floor at z
        // (matching build_floor_cache which checks below_submap furniture)
        const tripoint_bub_ms below_pos( p.x(), p.y(), z - 1 );
        if( inbounds( below_pos ) &&
            has_flag_furn( ter_furn_flag::TFLAG_SUN_ROOF_ABOVE, below_pos ) ) {
            return true;
        }

        // Check for vehicle roof/opaque parts at this level
        const optional_vpart_position vp = veh_at( check_pos );
        if( vp ) {
            if( vp->part_with_feature( VPFLAG_ROOF, false ) ||
                vp->part_with_feature( VPFLAG_OPAQUE, false ) ) {
                return true;
            }
        }
    }

    return false;
}
```

## B3 — moonlight (`src/calendar.h`, `src/calendar.cpp`, `src/weather.h`, `src/weather.cpp`)

**Why:** the adapter's environment export calls
`incident_moonlight( wm.weather_id, now )`
(`voxel_real_adapter.cpp:392`). 0.I-1 has no moonlight-intensity
function at all. Master has a two-piece chain: `moon_light_at` exported from
calendar (calendar.h:652 (master)) feeding `incident_moonlight`
(weather.h:190 / weather.cpp:153-157 (master)). Both pieces exist
in embryo in 0.I-1: `moon_light_at` is already implemented but
`static` (calendar.cpp:371), and the `weather_type` fields it
combines — `light_modifier` (weather_type.h:92) and
`light_multiplier` (weather_type.h:94) — match master's usage
exactly (verified).

**`src/calendar.cpp`** — one-word change at calendar.cpp:371:
remove `static`:

```cpp
float moon_light_at( const time_point &p )
```

(No other translation unit in 0.I-1 defines `moon_light_at` —
grep-verified — so exporting it cannot collide.)

**`src/calendar.h`** — declare it after `sun_light_at`'s
declaration (calendar.h:637), as on master (calendar.h:651-652
(master)):

```cpp
float moon_light_at( const time_point &p );
```

**`src/weather.h`** — insert after `incident_sunlight`'s
declaration (weather.h:187-188), verbatim master weather.h:189-191:

```cpp
// Apply weather and time of day to the moonlight.
float incident_moonlight( const weather_type_id &wtype,
                          const time_point &t = calendar::turn );
```

**`src/weather.cpp`** — insert after `incident_sunlight`'s
definition (weather.cpp:142-145), verbatim master
weather.cpp:153-157:

```cpp
float incident_moonlight( const weather_type_id &wtype, const time_point &t )
{
    return std::max<float>( 0.0f, moon_light_at( t ) * wtype->light_multiplier +
                            wtype->light_modifier );
}
```

## B4 — weather tints (`src/lightmap.h`, `src/weather_type.h`, `src/weather_type.cpp`, `data/json/weather_type.json`)

**Why:** the adapter reads `wt.tint_color` / `wt.tint_strength`
(`voxel_real_adapter.cpp:404-407`) — `tint_color.is_colored()`,
`.r/.g/.b`, and the strength — for the EnvironmentUpdate tint
(Addendum 11 light design). 0.I-1's `weather_type` has no tint
fields.

**Adaptation point (the one non-verbatim piece in this pack):**
the field *type*, `light_color_rgb`, also does not exist in
0.I-1 — it is part of master's smooth-lighting rework (defined at
master lightmap.h:61). The backport therefore carries the type
itself, verbatim, in the same relative position master keeps it:
in `lightmap.h`, immediately before the `lit_level` enum. Two
master neighbours are deliberately **not** carried, and nothing in
the tint path needs them: the `static_assert` at master
lightmap.h:90 (cosmetic) and `dawn_dusk_color_for_lightmap`'s
declaration at master lightmap.h:92-94 (its definition depends on
master's smooth-lighting machinery). The struct's
`light_color_rgb::from_hsv` *declaration* (inside the struct)
rides along verbatim; its definition (master lightmap.cpp) is not
backported — nothing in 0.I-1 calls it, and an uncalled declaration
is link-safe.

**`src/lightmap.h`** — insert before `enum class lit_level`
(lightmap.h:53), verbatim master lightmap.h:59-89:

```cpp
// Per-tile accumulated light color energy (float RGB).
// Stored as raw energy, not display-ready; the renderer converts to uint8.
struct light_color_rgb {
    light_color_rgb() = default;
    light_color_rgb( float red, float green, float blue )
        : r( red ), g( green ), b( blue ) {};
    float r;
    float g;
    float b;
    bool is_colored() const {
        return r > 0.0f || g > 0.0f || b > 0.0f;
    }
    light_color_rgb &operator+=( const light_color_rgb &rhs ) {
        r += rhs.r;
        g += rhs.g;
        b += rhs.b;
        return *this;
    }
    bool operator==( const light_color_rgb &rhs ) const {
        return r == rhs.r && g == rhs.g && b == rhs.b;
    }
    bool operator!=( const light_color_rgb &rhs ) const {
        return !( *this == rhs );
    }
    light_color_rgb operator*( float scale ) const {
        return { r * scale, g * scale, b * scale };
    }

    // Create from HSV. h in [0,360), s and v in [0,1].
    static light_color_rgb from_hsv( float h, float s, float v );
};
```

**`src/weather_type.h`** — two edits. First the include master
gets the type through (master weather_type.h:17): after
`#include "color.h"` (weather_type.h:15) add:

```cpp
#include "lightmap.h"
```

Then the fields, verbatim master weather_type.h:132-135. Master
places them at the end of the field list, just before `load()`;
0.I-1's analogous spot is after `debug_leave_eoc`
(weather_type.h:119), before the `load` declaration
(weather_type.h:120):

```cpp
        // light color tint applied when weather is active.
        // defaults to no color.
        light_color_rgb tint_color{};
        float tint_strength = 0.0f;
```

**`src/weather_type.cpp`** — insert the load code after the
`sun_multiplier` line (weather_type.cpp:112) — the same neighbours
as on master (sun_multiplier above, `sound_attn` below; master
additionally loads `temperature_modifier` between them, a field
0.I-1 does not have — irrelevant here). Verbatim master
weather_type.cpp:115-121:

```cpp
    if( jo.has_array( "tint_color" ) ) {
        JsonArray jarr = jo.get_array( "tint_color" );
        tint_color.r = jarr.get_int( 0 ) / 255.0f;
        tint_color.g = jarr.get_int( 1 ) / 255.0f;
        tint_color.b = jarr.get_int( 2 ) / 255.0f;
    }
    optional( jo, was_loaded, "tint_strength", tint_strength, 0.0f );
```

**`data/json/weather_type.json` — FINDING: nothing ports.** Every
master entry carrying `tint_color`/`tint_strength` is
scarlet-dimension weather content that **does not exist in 0.I-1
at all** (master has 28 weather types, 0.I-1 has 20, and all 20 of
0.I-1's ids exist on master; the 8 master-only ids are the 7
scarlet types below plus `rainstorm_reverberation`, which carries
no tint). The exact master values, recorded so they can travel
with the content if it is ever backported:

| id | `tint_color` | `tint_strength` | in 0.I-1? |
|---|---|---|---|
| `scarlet_day` | `[ 210, 0, 0 ]` | 1.0 | **no** |
| `scarlet_dimension_storm` | `[ 180, 15, 25 ]` | 1.0 | **no** |
| `scarlet_storm_1` | `[ 125, 100, 30 ]` | 0.8 | **no** |
| `scarlet_storm_2` | `[ 125, 60, 30 ]` | 0.9 | **no** |
| `scarlet_storm_3` | `[ 160, 15, 30 ]` | 1.0 | **no** |
| `scarlet_storm_4` | `[ 180, 15, 25 ]` | 1.0 | **no** |
| `scarlet_storm_5` | `[ 175, 25, 0 ]` | 1.0 | **no** |

Consequence for the new base: every 0.I-1 weather type loads the
defaults (no colour, strength 0.0), so the adapter compiles and
the tint path is inert — which matches 0.I-1's actual content.
The C++ half above is what makes the tint *mechanism* available
the moment tinted weather content exists.

## Verification (scratch-apply, 2026-11-23)

All four backports were applied to **copies** of the 0.I-1 files
under `~/workspace/cdda-stable/backport-apply/` (scripts:
`apply.py`, `verify.py` in the same directory; inserted text was
extracted from the master tree by line range, so the tested text
is byte-identical to the blocks quoted above). The repo and the
0.I-1 checkout were not modified. Results:

- **(a) Presence:** every inserted block is present in its copy
  (18 needle checks, all OK; `static` confirmed gone from
  calendar.cpp's `moon_light_at`).
- **(b) Brace balance:** for all 11 edited C++ files, the
  `{`/`}` count delta vs the pristine 0.I-1 file exactly equals
  the brace delta of (inserted − removed) text. Notable: B1's
  mapdata.cpp delta is (0, 0) — the anonymous-namespace braces are
  replaced one-for-one by the `namespace mapdata` braces; B2's
  map.cpp delta is (13, 13), matching the copied body.
- **(c) Identifier resolution:** all 38 identifiers referenced by
  the inserted code were grep-checked against the 0.I-1 `src/`
  tree plus the backported copies — **every one resolves, zero
  unresolved.** This includes all B2 dependencies listed above,
  the B1 factories, `moon_light_at` / `incident_moonlight`, and
  the B4 type/fields.
- **(d) JSON:** master's, 0.I-1's original, and the copied
  `weather_type.json` all parse (`json.load`). No JSON edit was
  applied — per the finding above there is nothing to insert;
  the tinted-id inventory (7 ids, values as tabulated) was
  extracted programmatically from master's file and each id
  confirmed absent from 0.I-1's.
- **Diff sizes** (changed lines, pristine → copy): mapdata.h +9,
  mapdata.cpp 6, map.h +9, map.cpp +70, calendar.h +1,
  calendar.cpp 1 line modified, weather.h +3, weather.cpp +6,
  lightmap.h +32, weather_type.h +5, weather_type.cpp +7.

What this does **not** prove: compilation. The scratch-apply
checks anchors, brace structure, and identifier existence — the
first real compile is still the PC build (gates 0.1/0.2), like
everything else in the fork.
