# Phase 10 — Skybox and sky-driven global illumination

Status: **PLANNED 2026-11-23. All six §9 decisions made 2026-11-23
by Joey: 1A procedural, 2A seasonal sun path, 3C cave gradient,
4A per-tile both renderers, 5A lightning flash only, 6A stars +
moon disc. Slice 10.0 spike DONE 2026-11-23 (§11) — wire shape
frozen with one upgrade: the game exports its own real sun
azimuth/altitude, so the invented sine path survives only as the
fake-data fallback. Wave 1 (10.1 / 10.2 / 10.5-precip-early) built
2026-11-23 by three parallel subagents in separate worktrees and
MERGED to main the same day on Joey's word (merges 4d7fb92 /
4bbecd2 / dbbc059), each suite re-verified in the main tree after
its merge; cargo check --workspace clean.** This document is the implementation plan; when the
phase starts, `plan.md` gets the §5 section, progress-log row updates, and
Part II gate rows in the usual form, and this doc becomes the design record
(like `docs/phase7-lighting.md`).

## 0. Terms

- **Skybox** — what a ray that hits no geometry shows: the sky itself.
  Procedural here (§4), not a texture cubemap — UltiCa ships no sky art.
- **Global illumination (GI), as scoped here** — the sky's contribution to
  lighting every surface: a directional sun/moon term plus a hemisphere
  ambient term (up-facing surfaces take zenith colour, down-facing take
  ground colour, sides blend). It is *not* multi-bounce path tracing.
  The RT renderer already does one shadow ray; §5 retargets it. A single
  sky-bounce extension for RT is an optional stretch (§5.4), not v1.
- **Environment state** — the one per-frame value everything reads:
  game time, sun/moon position and strength, sky colours, fog, weather.
  Today this is `lighting::Environment` (`DayNight × WeatherParams`),
  evaluated from *fake* inputs. Phase 10 makes its inputs real and
  extends it into a full `SkyState` (§3).
- **Game truth / ceiling rule** — the Phase 7 contract that still governs:
  a tile's `lit_level`/`seen` (the game's per-tile visibility enum and
  quantized visibility value, exported in Snapshot/Delta) is the truth
  about what may be shown. Sky GI is presentation *under* that ceiling —
  it must never sun-light a tile the game marks `DARK` into visibility,
  and never reveal a `BLANK` (never-seen) tile. Phase 7 §12.3 already
  states this for the tracer; Phase 10 applies it to both renderers.
- **Snapshot / Delta / LightUpdate** — the existing FlatBuffers IPC
  messages (`schema/cdda_ipc.fbs`): full view, per-tile changes, and a
  standalone light refresh. Environment data rides the same channel (§2).

## 1. Where things stand today (measured from the code, 2026-11-23)

| Piece | Today | File |
|---|---|---|
| Raster background | Fixed clear colour `(0.08, 0.10, 0.14)` — dark blue-grey at every hour, every weather | `crates/engine/src/render.rs` `CLEAR_COLOR`, used as the world pass `LoadOp::Clear` |
| RT background | Hardcoded `sky(rd)` gradient: fixed horizon `(0.08,0.10,0.14)`, zenith `(0.30,0.48,0.78)`, ground `(0.05,0.06,0.08)`; no uniforms, no sun disc, no night | `crates/treebuild/src/render.rs` `sky()` |
| Raster light | Hardcoded direction `(0.3, 0.8, 0.4)`, `shade = 0.45 + 0.55·ndl`; no uniform, no colour, no time | `crates/engine/src/render.rs` WGSL |
| RT light | Hardcoded `sun_dir_grid() = (0.3, −0.4, 0.8)`; ambient `0.35 + 0.85·ndl`, one shadow ray, occupancy AO; result multiplied by `lit_factor(lit_level)` | `crates/treebuild/src/render.rs` `shade()` |
| Day/night + weather model | Exists and is gated (7.6/7.7): keyframe curve `day_night_at(hour)`, `WeatherKind` → `WeatherParams {fog, precip, brightness_scale, tint}` — but it only *tempers the CPU bake* (`0.4 + 0.6·curve`). Fog is computed and never applied in any shader; precip rates feed nothing | `crates/lighting/src/atmosphere.rs` |
| Environment inputs | Fake: CLI `--hour` / `LightingScene::fake_house(hour, WeatherKind::Clear)`; windowed app hardcodes `Clear` | `crates/engine/src/main.rs`, `frame.rs` |
| Protocol | **No time field, no weather field anywhere.** Phase 7 deviation 3 recorded this: "the schema carries no weather field yet" | `schema/cdda_ipc.fbs` |
| Fork exporter | `IGameAdapter` has tile/entity/vehicle/memory queries only — no environment query | `fork/src/voxel_ipc/voxel_adapter.h` |

So the work splits cleanly: **(a)** get real time + weather across the
wire, **(b)** one `SkyState` derived from them, **(c)** a sky background
and GI terms in both renderers driven by that state, **(d)** fog/precip
actually reaching the frame. The Phase 7 model is the foundation, not a
rewrite target.

## 2. Protocol: export time and weather from the game (fork + schema)

### 2.1 What the game actually has (verified against upstream source)

- Time: `calendar::turn` (`time_point`); `hour_of_day(p)`,
  `minute_of_hour(p)`, `second_of_minute(p)`; `sunrise(p)` / `sunset(p)`
  (season-dependent); `is_night / is_dawn / is_dusk(p)`;
  `season_of_year(p)`. Sun/moon strength: `sun_light_at(t)`,
  `moon_light_at(t)` in `src/calendar.cpp`, wrapped by
  `incident_sunlight(wtype, t)` / `incident_moonlight(wtype, t)` in
  `src/weather.cpp` — these already fold weather in.
- Weather: `get_weather()` → `weather_manager { weather_id:
  weather_type_id, temperature, windspeed, winddirection,
  lightning_active, nextweather }`. The per-type data
  (`weather_type` in `src/weather_type.h`) is **JSON-defined and
  mod-extensible**: `light_multiplier`, `light_modifier`,
  `sun_multiplier`, `precip: precip_class`, `rains`, `sight_penalty`,
  `tint_color: light_color_rgb`, `tint_strength`.
- **Consequence:** weather must cross the wire as *string id + resolved
  numeric fields*, never as a closed enum alone. A modded weather type
  the engine has never heard of must still light correctly. The existing
  `WeatherKind` (7 variants) becomes a *presentation classification*
  the engine derives from the numeric fields (precip class + rains +
  fog-from-sight-penalty), with the string id kept for precip/fog
  special cases (e.g. portal storm) and for debugging.

### 2.2 Wire design (additive, non-breaking — per `docs/protocol.md`)

New table, appended to the `Message` union (old readers ignore it):

```fbs
table EnvironmentUpdate {
  seq: uint64;
  epoch: uint64;
  /// Game time: total turns since calendar::turn_zero, plus the
  /// derived values the engine should not have to re-derive.
  turn: int64;
  time_of_day_sec: float;      // 0..86400, fractional
  sunrise_sec: float;          // today, game-local
  sunset_sec: float;
  season: ubyte;               // season_type: 0 spring .. 3 winter
  sun_light: float;            // incident_sunlight(wtype, turn)
  moon_light: float;           // incident_moonlight(wtype, turn)
  sun_azimuth_deg: float;      // sun_azimuth_altitude(turn).first — from South, west-positive
  sun_altitude_deg: float;     // sun_azimuth_altitude(turn).second — negative below horizon
  moon_phase: ubyte;           // get_moon_phase(turn), 0..MOON_PHASE_MAX
  /// Weather at the player (weather_manager state).
  weather_id: string;          // e.g. "weather_rain"
  light_multiplier: float;     // weather_type fields, resolved game-side
  light_modifier: float;
  sun_multiplier: float;
  precip_class: ubyte;         // 0 none, 1 very_light, 2 light, 3 heavy
  rains: bool;
  sight_penalty: float;        // -> fog density input
  tint_rgb: [float];           // weather_type tint_color, 3 entries or absent
  tint_strength: float;
  temperature_c: float;
  windspeed_mph: float;
  wind_dir_deg: float;
  lightning_active: bool;
}
```

Plus two piggyback fields so a fresh client is never environment-blind
before the first update: `environment: EnvironmentUpdate` appended to
`Snapshot`, and the same appended to `Delta` (usually absent; present
when it changed — see §2.3). All additive: v1 readers ignore them, the
golden round-trip tests (gate 1.3 pattern) get a new golden pair.

Why game-side resolution instead of sending only `turn` + `weather_id`
and re-implementing the astronomy in Rust: CDDA's sunrise/sunset and
sun/moon curves depend on season length and world options; duplicating
them engine-side guarantees drift on rebase. The 10.0 spike (§11)
found the strongest form of this: the game exports its own computed
sun *position* (`sun_azimuth_altitude`), so the engine computes no
astronomy at all for live data — it converts two game-supplied angles
to a direction vector (§3.2). Sunrise/sunset stay on the wire for
gates, debugging, and the fake-data fallback path.

### 2.3 When it is sent

At the existing send boundary (`game::do_turn()` exit, the Phase 3
hook), the exporter compares against the last-sent environment and
emits an `EnvironmentUpdate` when any of these holds:

- `weather_id` changed, or `lightning_active` flipped;
- game time advanced ≥ 10 minutes since the last send (time-only
  cadence; the engine interpolates between updates, §3.3, so this is
  not a visible step. **Ratified by Joey 2026-11-23:** this section
  originally said ≥ 1 minute, contradicting gate 10.1b's
  9-quiet-minutes/10th-emits wording — the gate won, and the
  implementation's `kEnvIntervalTurns = 100` stands as built);
- a Snapshot is being sent anyway (piggyback, §2.2).

No new hook points: the exporter polls at the boundary it already owns.
`IGameAdapter` gains one method, `environment() -> EnvVal`, implemented
in `voxel_real_adapter.cpp` against `calendar::turn` + `get_weather()`;
the harness mock implements the same method so the serializer/diff
logic is tested byte-for-byte as in Phase 3.

### 2.4 Blocked-half note

The fork half compiles only on a build-capable machine (the standing
0.1/0.2 condition). Everything engine-side is built and gated against
the harness/stub server first — the same conditional-pass pattern as
Phases 1/3/7. The stub server gains scripted environment sequences
(noon → storm → midnight) for the gates.

## 3. `SkyState`: one derived state in `crates/lighting`

New module `crates/lighting/src/sky.rs`; `atmosphere.rs` stays as the
bake-facing layer and re-exports from it (no breaking change to the
Phase 7 gates).

### 3.1 Contents

```rust
pub struct SkyState {
    pub sun_dir: [f32; 3],        // toward the sun, world space; below-horizon = moon takes over
    pub sun_color: [f32; 3],      // incident_sunlight x weather tint
    pub sun_intensity: f32,
    pub moon_dir: [f32; 3],       // antipode-ish: sun path offset by moon phase proxy (see 3.2)
    pub moon_color: [f32; 3],
    pub moon_intensity: f32,
    pub zenith: [f32; 3],         // sky gradient stops, weather-blended
    pub horizon: [f32; 3],
    pub ground: [f32; 3],         // below-horizon + hemisphere ground term
    pub ambient_scale: f32,       // hemisphere GI strength
    pub fog_density: f32,         // from WeatherParams, now actually consumed
    pub fog_color: [f32; 3],      // = horizon colour, by construction
    pub cloud_cover: f32,         // 0 clear .. 1 overcast (from weather class + precip)
    pub precip: PrecipState,      // kind, rate, fall speed (existing WeatherParams fields)
    pub exposure: f32,            // = existing tempered Environment factor, for the bake
}
pub fn sky_state(env: &GameEnvironment, /* last: for transitions */) -> SkyState
```

`GameEnvironment` is the proto-adapter form of `EnvironmentUpdate`
(mirror of `proto_adapter.rs` for lit/seen). The existing
`day_night_at` keyframes become the *fallback/cross-check* curve:
when the wire supplies `sun_light`, it wins; the keyframes remain for
fake-data scenes and as the gate's smoothness reference.

### 3.2 Sun path — game truth for live data (revised by the 10.0 spike)

**Live data:** the sun direction is the game's own.
`sun_azimuth_altitude(turn)` (public, `src/calendar.h`) computes real
horizontal coordinates — declination and sidereal time from the world
location's latitude/longitude and season-effective time, with
eternal-day/night handled game-side. The engine converts the wire
`sun_azimuth_deg` (from South, turning positive to the west) +
`sun_altitude_deg` into a direction vector; one documented conversion,
gated for sign errors against the game's `sunlight_angle()` semantics
(the per-z ray offset it returns for shadow direction). Seasonal
variation — decision 2A's intent — is inherent in the game's model;
no `max_elev` table exists on the live path.

**Moon:** CDDA exposes phase (`get_moon_phase`) and strength
(`moon_light_at`) but *no* moon position function. The moon's
direction is the one remaining invented piece: mirrored sun path
through the night span (antipode of the sun's daytime arc), strength
and disc size from the real phase/light values.

**Fake-data fallback only** (stub scenes, captures without a game):
the originally planned sine path — elevation =
`sin(π · (t − sunrise)/(sunset − sunrise)) · max_elev(season)`,
`max_elev` 65° summer → 25° winter (decision 2A's numbers, kept for
this path). Gates pin continuity on both paths; only the live path
pins position, against game-supplied anchors.

### 3.3 Smoothing between game updates

The game is turn-based; the camera is not. `SkyState` is therefore
evaluated **per rendered frame** from an engine-side clock that
interpolates `time_of_day_sec` between the last two environment
updates at the game's rate, and weather transitions (any change of
`weather_id` or precip class) cross-fade all weather-derived fields
over 30 game-seconds. Result: no brightness pop on a turn boundary,
sun crawls instead of stepping. Gate: 1-minute sweep asserting the
same ≤0.01 max step the 7.6 gate pins, now over the full `SkyState`.

### 3.4 Underground, indoors, and the ceiling rule

Sky GI applies only where the sky is reachable:

- **Per-tile sky factor** `s ∈ [0,1]`: 1 outdoors under open sky, 0
  underground / under a roof. **Source decided by the 10.0 spike
  (§11): option (a), the game's own test** — the exporter computes
  `sky_exposed = map::is_outside(p) && !map::is_roofed(p)` per tile
  (the same conjunction the game itself uses for weather exposure,
  e.g. `character.cpp`) and exports it as one bool per tile appended
  to `SubmapBlock` plus `outside: bool` on `TileChange` (additive).
  Known fringe: `build_outside_cache` spreads INDOORS flags over a
  3×3, so an outdoor tile adjacent to an indoor one can read
  not-outside — a 1-tile fringe at doorways, accepted and named.
  The engine-derived fallback (b) (column scan of the unbaked world)
  is *not* built; it remains the documented fallback if (a) ever
  proves wrong in-game.
- Final per-tile/per-voxel presentation stays
  `min(sky-lit result, ceiling from lit_level)` in the RT path
  (existing `lit_factor` multiply already does the capping shape) and
  `albedo × Environment-tempered sample` in the raster bake, with the
  directional/hemisphere terms scaled by `s`. Interiors lit by lamps
  at noon look exactly as they do today; a field at midnight stays
  dark even though the sky shader "knows" the moon.

## 4. The sky background (both renderers, one function)

One WGSL function, shared by construction — the raster copy and the
RT copy are generated from a single source string in a new
`crates/engine/src/sky_wgsl.rs` (the treebuild crate cannot depend on
the engine crate; the string lives in `treebuild` and the engine
includes it — same pattern as the duplicated fake-house fixtures,
but for a shader the gate diffs the two copies so they cannot drift).

```
fn sky_color(rd, sky: SkyUniforms) -> vec3 {
    gradient: mix(horizon, zenith, pow(rd.z, 0.6)) above horizon,
              mix(horizon, ground, ...) below          // today's shape, uniform-driven
    + sun disc: smoothstep(cos angle) disc + wide glow, x sun_color x (1 - cloud_cover*0.9)
    + moon disc: smaller, cooler, same construction
    + stars: hash grid on rd, only when sun below horizon,
             faded by cloud_cover and fog               // v1: cheap hash stars, no texture
    all x fog blend toward fog_color at the horizon
}
```

- **Raster:** the world pass stops clearing to a fixed colour. The
  clear colour becomes the frame's horizon colour (so letterboxed /
  uncovered pixels are still right), and a fullscreen sky pass —
  one triangle, depth-write off, drawn first, reconstructing `rd`
  from the inverse view-projection — fills the background. Cost: one
  fullscreen pass, no geometry.
- **RT:** `sky(rd)` is replaced by the same function; a miss returns
  `sky_color`, and a primary ray's *first* bounce context (§5.4) uses
  it too. The existing golden rule "sky pixels identical between
  variant renders" (multitile gate) keeps holding because the sky is
  a pure function of uniforms + ray direction.
- **Cutaway honesty:** when the cutaway hides upper z-levels, the sky
  is still drawn behind the world — it is background, not a hidden
  tile, so this does not violate the visibility contract. Underground
  (player z < 0 with a ceiling above the whole view): `fog_color` and
  the gradient switch to the dark cave gradient (decision 3C, §9)
  via the §3.4 factor evaluated at the *camera target tile*.

## 5. Sky GI in the renderers

### 5.1 Shared uniform block

```wgsl
struct SkyUniforms {
    sun_dir: vec3, sun_color: vec3,   // colour pre-multiplied by intensity
    moon_dir: vec3, moon_color: vec3,
    zenith: vec3, horizon: vec3, ground_col: vec3,
    ambient_scale: f32, fog_density: f32, fog_color: vec3,
    cloud_cover: f32, time_sec: f32,  // time_sec: cloud drift / star twinkle phase
}
```

One buffer, written once per frame from `SkyState`, bound by the
raster world pipeline (new group), the RT compute pipeline (extends
its existing `Uniforms` — `sun_dir` already exists there and stops
being a constant), the billboard pass (sprites take hemisphere +
sun so creatures stop being lit by a different sun than the ground),
and the fx field pass tint. **One sun everywhere** is the point.

### 5.2 Raster path

Replace the hardcoded shade with:

```
fn shade(albedo, n, sky_factor) =
    albedo * ( hemisphere(n) * ambient_scale
             + sun_color * max(dot(n, sun_dir), 0) * sky_factor
             + moon_color * max(dot(n, moon_dir), 0) * sky_factor )
             * baked_light        // existing Phase 7 per-vertex bake = the ceiling
```

where `hemisphere(n) = mix(ground, zenith, n.up*0.5+0.5)`.
No shadow maps in v1 (named limit): raster shadows remain the baked
game light, exactly as today; the directional term is modulated by
the bake so a wall's dark side stays dark. The vertex format gains
nothing — `sky_factor` is derived in-shader from the existing baked
colour's outdoor-ness proxy… **no** — cleaner: `sky_factor` rides the
currently-unused vertex alpha high bits? Rejected as hacky during
planning: v1 raster takes `sky_factor` from a per-tile uniform-free
route — the bake writes it into the vertex colour's alpha (alpha is
1.0 for opaque pass today and unused for shading). Slice 10.3 spikes
this and falls back to "raster GI is global-only, per-tile factor
applies in RT only" if it fights the greedy mesher's merge key
(colour is part of the merge key, so per-tile alpha differences
already break merges correctly — the spike verifies).

### 5.3 RT path

`shade()` changes: ambient `0.35` → hemisphere term from the sky
uniforms; `0.85·ndl` → `sun_color` term; the shadow ray already traces
toward `u.sun_dir` — it now traces toward the *real* sun (or moon,
when the moon is the stronger source), so shadows lengthen at dawn
and vanish under overcast (`cloud_cover → 1` collapses the directional
term into the hemisphere term; no shadow ray is cast when the
directional intensity is below a threshold — a small perf win on
storm frames). Emissive and MEMORIZED handling unchanged; the
`lit_factor` ceiling multiply stays last.

### 5.4 Stretch (not v1): one sky bounce in RT

A miss on a *shadow/secondary* ray already returns sky; letting a
primary hit spawn one diffuse bounce ray that samples `sky_color`
would give true sky GI (colour bleeding from the sky dome) in RT
only. Cost is a second trace per pixel — this waits on the 2c.9
RX 9070 XT frame-time number, same as the raster-vs-RT default call.
Listed here so the uniform design doesn't preclude it.

### 5.5 Fog and precipitation — finally consumed

- **Fog:** exponential heightless fog, `exp(−density · dist)`, toward
  `fog_color` (= horizon colour), applied in the raster fragment and
  the RT `shade`/sky return, using the per-pixel depth both already
  have. `WeatherParams::apply_fog` is the CPU reference the gates
  check the WGSL against (same discipline as the fx post stack).
  Density source: existing `WeatherParams` table, now keyed by the
  *derived* `WeatherKind` (§2.1) and scaled by wire `sight_penalty`
  when present.
- **Precipitation:** the fx crate's soft-particle system gains a
  precip emitter fed by `PrecipState` (rate/fall-speed already in
  `WeatherParams`): rain = short streak instances, snow = slow
  drifting motes, spawned in a cylinder around the camera target,
  only on sky-exposed tiles (§3.4 factor at spawn). Drawn in the
  existing Fields pass — no new frame pass, `FRAME_ORDER` unchanged.
- **Lightning:** `lightning_active` + a fork-sent flash event (reuse
  `EffectEvent` kind `Other`, id `"lightning"`) spikes
  `sun_intensity`/exposure for ~150 ms with a random re-strike —
  v1 is the flash only, no bolt geometry. Thunderstorm sky colours
  already darken via the weather table.

## 6. Frame loop integration

- `frame.rs`: `CaptureArgs` gains `sky: Option<SkyScene>` beside
  `lighting` — where `SkyScene { game_env: GameEnvironment,
  clock: EngineClock }`. `LightingScene::fake_house(hour, weather)`
  is re-expressed as a constructor of a fake `GameEnvironment` so all
  existing gates keep their call shapes.
- `render_frame` evaluates `SkyState` once, writes the uniform block
  once, and passes it to world/fields/billboards/post. The HD-2D post
  grade stops hardcoding "warm": its grade tint is multiplied by the
  sky tint (dawn warms, midnight cools) — `PostConfig` gains a
  `grade_tint: [f32;3]` defaulting to today's warm values at neutral
  environment, so pre-Phase-10 goldens are byte-identical at neutral.
- **Golden migration:** every existing capture golden is pinned to
  `Environment::neutral()` = Clear noon. At neutral, the plan requires
  the new pipeline to reproduce today's *geometry* pixels; background
  pixels change by design (fixed clear → noon gradient). The 2g.3
  harness gets a one-time golden regeneration for background-only
  diffs, reviewed as a set (the Phase 9 OFF-vs-ON discipline: diff
  must be exactly the sky region + GI shading, nothing else moves).
- CLI/capture: `--sky <script>` for captures (a named environment
  sequence: `noon-clear`, `midnight-clear`, `dawn-rain`, …) so the
  gate captures are deterministic; windowed app takes its environment
  from the live client state, falling back to neutral when no game
  is connected.

## 7. Slices (each gated before the next starts)

| Slice | Work | Depends on |
|---|---|---|
| 10.0 Spike | Verify the per-tile outside flag is readable in the fork cheaply (§3.4a vs 3.4b); verify `sun_light_at/moon_light_at` linkage from `voxel_real_adapter`; pin `max_elev` defaults against CDDA's own daylight spans | — |
| 10.1 Protocol | `EnvironmentUpdate` in schema + proto crate + C++ stub + goldens; `IGameAdapter::environment()` + harness mock; exporter send rules (§2.3) in the harness | 10.0 |
| 10.2 SkyState | `lighting::sky` module: derivation from `GameEnvironment`, sun path, weather classification from numeric fields (unknown/modded weather → sane defaults), interpolation + weather cross-fade; CPU gates only, no GPU | 10.1 |
| 10.3 Sky background + raster GI | Shared WGSL sky function; raster sky pass + `SkyUniforms`; hemisphere/sun/moon shading replacing the hardcoded shade; per-tile sky factor spike (§5.2) resolved | 10.2 |
| 10.4 RT GI | RT uniforms extended; hemisphere + real sun/moon + shadow retarget + overcast collapse; ceiling rule re-verified; sky-bounce stretch evaluated against perf | 10.3 (shared WGSL) |
| 10.5 Fog + precip + billboard/fx sync | Fog in both renderers vs CPU reference; precip emitter in Fields pass; billboards + field volumes take the same uniforms; post grade tint wiring | 10.3, 10.4 |
| 10.6 Live seating + captures | Windowed app consumes live environment from the client (replacing hardcoded `Clear`); capture `--sky` scripts; the visual capture set (§8) presented for Joey's sign-off; plan.md close-out | 10.5; fork half still conditional on 0.1/0.2 |

Parallelism: 10.1's fork/harness half and 10.2's pure-Rust half can
run concurrently once the wire shape is frozen in 10.0. 10.3/10.4
share the sky WGSL — one owner, sequential merges (the props.rs
lesson: same-file work does not parallelize across worktrees).

## 8. Gates (Part II style — each fails without its slice)

- **10.1a** Round-trip: `EnvironmentUpdate` C++ golden ↔ Rust decode,
  semantic equality (gate 1.3 pattern); old-reader test: a v1 decoder
  reading a stream with environment messages ignores them.
- **10.1b** Exporter: harness mock — weather change emits exactly one
  update; 9 game-minutes of quiet time emits none; the 10th emits one;
  Snapshot always carries the current environment.
- **10.2a** Sun path: sunrise/sunset anchors exact (sun on horizon at
  both), elevation peaks mid-span, full-day 1-minute sweep has no
  step > 0.01 in any `SkyState` field; midnight state = moon-dominant.
- **10.2b** Weather classification: every vanilla weather id classifies
  to its expected `WeatherKind`; an *invented* id with rain numerics
  classifies as rain and lights within 5% of vanilla rain.
- **10.3a** Sky parity: raster vs RT, same `SkyState`, empty scene —
  sky pixels agree within sRGB ±2 per channel over the full frame
  (the shared-WGSL gate; a drift in either copy fails it).
- **10.3b / 10.4a** Ceiling rule: fake house at noon — interior lamp
  room's pixels change ≤ noise vs neutral-environment render; the
  exterior field brightens. At midnight the field darkens. A `BLANK`
  region renders as background/sky only, never geometry.
- **10.4b** Shadows: RT capture at dawn vs noon — a wall's shadow
  direction/length differs in the sun's favour (measured on the
  capture, not asserted from uniforms); overcast capture casts no
  hard shadow (directional term collapsed).
- **10.5a** Fog: capture pair at two distances under `Fog` — measured
  transmittance matches `WeatherParams::fog_transmittance` within 5%.
- **10.5b** One-sun check: the same frame's ground, billboard sprite,
  and field volume sample the same sun colour/intensity (probe pixels
  in one capture).
- **10.6** Visual set for sign-off: fake house + sample scene at
  {midnight, dawn, noon, dusk} × {Clear, Rain, Fog, Thunderstorm},
  both renderers, presented as contact sheets — **Joey's sign-off is
  acceptance**, per the standing rule; a green gate set is not.
- Perf: sky pass + GI terms add no per-tile CPU work; frame cost is
  re-measured in the 2c.9 slot on the RX 9070 XT (lavapipe numbers
  are shape only, as ever).

## 9. Decisions (all made by Joey 2026-11-23)

1. **Sky art — DECIDED 2026-11-23 by Joey: 1A, procedural gradient +
   sun/moon discs + hash stars.** (Rejected: painted cubemap — no
   source art exists and six faces per weather/time is a combinatorial
   problem.) This also settles the substance of decision 6 (stars +
   moon disc are in v1); decision 6 stays on the list only for a
   contrary explicit call.
2. **Sun path — DECIDED 2026-11-23 by Joey: 2A, seasonal.**
   Originally ratified as an invented sine path (`max_elev` 65°
   summer / 25° winter). **Superseded in mechanism by the 10.0 spike
   (§11), intent preserved:** the game computes real sun
   azimuth/altitude and the live path consumes it — seasonal
   variation comes from the game's own astronomy, strictly better
   than the ratified approximation. The 2A numbers survive only as
   the fake-data fallback (§3.2). Flagged to Joey 2026-11-23; if he
   wants the literal sine path instead, that is a one-line reversal
   in §3.2 before 10.2 builds on it.
3. **Underground sky — DECIDED 2026-11-23 by Joey: 3C, dark cave
   gradient, not pure black.** (Rejected: near-black collapse, always
   showing the surface sky.) §4's underground bullet reads with this
   gradient: very dark blue/brown stops so silhouettes still separate
   from the background.
4. **Raster per-tile sky factor — DECIDED 2026-11-23 by Joey: 4A,
   per-tile in both renderers**, via the §5.2 vertex-alpha spike.
   Pre-agreed fallback stands: if the spike shows the alpha route
   fights the greedy mesher, fall back to 4B (RT per-tile, raster
   global-only) with the failure recorded — that fallback needs no
   new decision.
5. **Lightning — DECIDED 2026-11-23 by Joey: 5A, flash only in v1.**
   No bolt geometry until a bolt visual is designed separately.
6. **Stars/moon disc — DECIDED 2026-11-23 by Joey: 6A, both in v1**
   (confirming the substance already implied by decision 1A).

## 10. Risks

- **Rebase drift:** `calendar`/`weather` are stable upstream APIs, but
  the exporter's send boundary is the Phase 3 hook — environment
  polling adds no new hooks, so rebase cost stays at Phase 3's level.
- **Double-darkening, again:** the Phase 7 tempering exists because
  `lit_level` is already time-aware. Sky GI multiplying *on top of*
  the bake can reintroduce exactly that bug at the directional term —
  gate 10.3b is written to catch it, and the temper applies to the
  GI sum, not just the ambient.
- **Golden churn:** every capture in the repo shows sky. The neutral-
  environment byte-identity requirement (§6) is what keeps the churn
  to one reviewed regeneration instead of a slow drift.
- **Mod weather ids** with extreme numerics (portal storms):
  classification clamps all derived fields to the vanilla ranges —
  a mod can look different, it cannot produce NaN/negative light.

## 11. Slice 10.0 spike record (2026-11-23, source-verified)

Verified against the upstream audit checkout (`~/workspace/cdda-audit/cdda`,
same pinned lineage as the fork) and the fork adapter sources. Source
reading only — the fork still cannot compile here (standing 0.1/0.2
condition); every claim below is a named public API at a named site,
ready for the harness adapter to mirror.

**Q1 — per-tile outside flag: YES, cheap, option (a) confirmed.**
- `map::is_outside(const tripoint_bub_ms&)` — public, `src/map.h:1232`;
  implementation (`src/map.cpp:3784`) is one `mdarray<bool>` lookup in
  `level_cache.outside_cache` after an inbounds check. O(1) per tile.
- The cache is maintained by `map::build_scene_caches()` →
  `build_outside_cache(z)` (`src/map.cpp:10627`), which early-outs when
  the level's cache is not dirty and is part of the game's normal
  per-turn cache build — the exporter adds no cache work, it reads at
  the `do_turn` boundary after the game has built them.
- Semantics: `z < 0` fills all-false (underground is never outside —
  exactly the §3.4 requirement); a tile whose terrain/furniture carries
  `TFLAG_INDOORS` marks itself and its 8 neighbours not-outside in the
  padded build — the named 1-tile doorway fringe in §3.4.
- `is_outside` alone is not sky exposure (a roofed, non-INDOORS porch
  reads outside). The game's own exposure conjunction is
  `is_outside(p) && !is_roofed(p)` (`src/character.cpp:6334`;
  `map::is_roofed`, `src/map.h`, walks upward through z-levels — bounded
  by the levels above the tile, and the exporter already pays the
  heavier `apparent_light_at` per tile in the same function).
- **Wire consequence (frozen):** `TileVal` gains `bool sky_exposed`,
  filled in `RealAdapter::query_tile` beside the existing lit/seen
  reads; schema gains `outside: [bool]` on `SubmapBlock` and
  `outside: bool` on `TileChange`, both additive. The harness mock
  fills the same field. Slice 10.1 owns this.

**Q2 — sun/moon linkage from `voxel_real_adapter.cpp`: YES, and the
sun's position is game truth.**
- All needed functions are public free functions taking
  `calendar::turn`: `sun_azimuth_altitude(t)` → `(azimuth, altitude)`
  (`src/calendar.h:664`; real equatorial→horizontal conversion using
  the world location's latitude/longitude and season-effective time;
  azimuth from South, west-positive; eternal day/night forced
  game-side), `sun_light_at(t)`, `moon_light_at(t)`,
  `sun_irradiance(t)`, `get_moon_phase(t)`, `sunrise(t)`/`sunset(t)`,
  `is_night/is_day/is_twilight(t)`.
- Weather side: `get_weather()` (`src/weather.h`) → `weather_manager`;
  `incident_sunlight(wtype, t)` / `incident_moonlight(wtype, t)` fold
  the weather type's `light_multiplier`/`light_modifier` in already.
- The adapter TU (`voxel_real_adapter.cpp`) already includes `game.h`,
  `map.h`, `lightmap.h`, `units.h` (it uses `units::to_degrees` for
  vehicle facing — the same conversion the sun angles need). It needs
  two new includes: `calendar.h`, `weather.h` (+ `weather_type.h` for
  the resolved fields). No linkage risk beyond the standing
  compile-on-a-real-machine condition.
- **Gap (named):** there is *no* moon azimuth/altitude function —
  phase and strength only. Moon direction stays the §3.2 proxy.
- **Wire consequence (frozen):** `EnvironmentUpdate` as in §2.2,
  including `sun_azimuth_deg` / `sun_altitude_deg` / `moon_phase`;
  sunrise/sunset retained for gates/debug/fallback. Slice 10.1 owns
  the schema; slice 10.2 owns the angle→direction conversion and its
  sign gate.

**Q3 — daylight spans / decision 2A:** resolved by Q2 — no engine-side
elevation table on the live path (§3.2, §9.2). The 65°/25° sine path
is retained verbatim for fake-data scenes only.

**Freeze statement:** with Q1–Q3 answered, the §2.2 wire shape and the
§3 `SkyState` inputs are frozen for Wave 1. Any change after this
point is a protocol-version event, not an edit.
