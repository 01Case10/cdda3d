//! Phase 10, slice 10.2 — `SkyState`: the one derived environment
//! state both renderers will read (docs/phase10-sky-gi.md §3).
//!
//! Inputs are [`GameEnvironment`], the engine-side mirror of the
//! frozen `EnvironmentUpdate` wire table (§2.2, freeze record §11).
//! It is defined *here*, not imported from the `proto` crate: the
//! proto adapter conversion is slice 10.6 integration work, and this
//! module must not drift with generated types.
//!
//! Division of labour, per the plan: the game supplies *truth*
//! (sun position via `sun_azimuth_altitude`, weather-folded sun/moon
//! strength via `incident_sunlight`/`incident_moonlight`, resolved
//! weather numerics). This module supplies *presentation*: sky
//! gradient stops, hemisphere GI strength, fog, cloud cover, precip
//! parameters — derived from the Phase 7 [`weather_params`] table,
//! keyed by a [`WeatherKind`] *classified from the numeric fields*,
//! never from the weather id string (§2.1: weather types are
//! JSON-defined and mod-extensible, so an unknown id with sane
//! numerics must still light correctly).
//!
//! The one invented piece is the **moon direction**: CDDA exposes
//! moon phase and moonlight strength but no moon position function
//! (§11 Q2). The moon's direction is the antipode of the sun's —
//! a proxy, labelled as such wherever it is used. Its *strength*
//! is game truth (`moon_light`).

use crate::atmosphere::{day_night_at, weather_params, WeatherKind, WeatherParams};

/// Seconds in a game day.
pub const DAY_LENGTH_SEC: f32 = 86_400.0;
/// `sun_light_at` ceiling in CDDA light units (`src/calendar.cpp`:
/// the linear segment caps at 125). Wire `sun_light`/`moon_light`
/// are normalized by this.
const MAX_SUN_LIGHT: f32 = 125.0;
/// Weather cross-fade duration in game-seconds (§3.3).
pub const WEATHER_FADE_SEC: f32 = 30.0;

// ---------------------------------------------------------------------------
// GameEnvironment — mirror of the frozen EnvironmentUpdate (§2.2/§11)
// ---------------------------------------------------------------------------

/// Engine-side mirror of the `EnvironmentUpdate` FlatBuffers table.
/// Field names and units follow §2.2 exactly; the proto adapter
/// (slice 10.6) converts field-for-field.
#[derive(Debug, Clone, PartialEq)]
pub struct GameEnvironment {
    pub seq: u64,
    pub epoch: u64,
    /// Total turns since `calendar::turn_zero`.
    pub turn: i64,
    /// 0..86400, fractional.
    pub time_of_day_sec: f32,
    /// Today's sunrise/sunset, game-local seconds.
    pub sunrise_sec: f32,
    pub sunset_sec: f32,
    /// `season_type`: 0 spring, 1 summer, 2 autumn, 3 winter.
    pub season: u8,
    /// `incident_sunlight(wtype, turn)` — already weather-folded
    /// game-side (`sun_light_at * light_multiplier + light_modifier`,
    /// clamped ≥ 0). Do NOT re-apply the multipliers here.
    pub sun_light: f32,
    /// `incident_moonlight(wtype, turn)` — same folding.
    pub moon_light: f32,
    /// `sun_azimuth_altitude(turn)`: azimuth from South,
    /// west-positive (0 = south, 90 = west, 180 = north, 270 = east);
    /// altitude negative below the horizon. NaN in fake-data scenes
    /// that have no game astronomy — selects the fallback path (§3.2).
    pub sun_azimuth_deg: f32,
    pub sun_altitude_deg: f32,
    /// `get_moon_phase(turn)`, 0..8 (8 = `MOON_PHASE_MAX` sentinel).
    pub moon_phase: u8,
    /// Weather type id, e.g. `"weather_rain"`. Debugging and
    /// special cases only — classification never reads it (§2.1).
    pub weather_id: String,
    pub light_multiplier: f32,
    pub light_modifier: f32,
    pub sun_multiplier: f32,
    /// 0 none, 1 very_light, 2 light, 3 heavy (`precip_class`).
    pub precip_class: u8,
    pub rains: bool,
    /// Fog-density input (§5.5).
    pub sight_penalty: f32,
    /// Weather tint colour (0..1 RGB), when the type defines one.
    pub tint_rgb: Option<[f32; 3]>,
    pub tint_strength: f32,
    pub temperature_c: f32,
    pub windspeed_mph: f32,
    pub wind_dir_deg: f32,
    pub lightning_active: bool,
}

impl GameEnvironment {
    /// Clear noon — the neutral environment, live-path form (real
    /// azimuth/altitude filled in as a game at noon in summer would
    /// report them, approximately). Matches `Environment::neutral()`
    /// semantics for the bake.
    pub fn neutral() -> Self {
        Self {
            seq: 0,
            epoch: 0,
            turn: 0,
            time_of_day_sec: 43_200.0,
            sunrise_sec: 21_600.0,
            sunset_sec: 64_800.0,
            season: 1,
            sun_light: MAX_SUN_LIGHT,
            moon_light: 1.0,
            sun_azimuth_deg: 0.0,
            sun_altitude_deg: 65.0,
            moon_phase: 0,
            weather_id: "sunny".to_string(),
            light_multiplier: 1.0,
            light_modifier: 2.0,
            sun_multiplier: 1.0,
            precip_class: 0,
            rains: false,
            sight_penalty: 1.0,
            tint_rgb: None,
            tint_strength: 0.0,
            temperature_c: 20.0,
            windspeed_mph: 0.0,
            wind_dir_deg: 0.0,
            lightning_active: false,
        }
    }

    /// Fake-data scene constructor (stub scenes, captures without a
    /// game): clear "sunny" numerics, **no** game astronomy — the
    /// azimuth/altitude are NaN, which selects the §3.2 fallback
    /// sine path. `sun_light`/`moon_light` are precomputed from the
    /// fallback angles so the values are what a game *would* have
    /// exported, in its own units.
    pub fn fake_day(
        time_of_day_sec: f32,
        sunrise_sec: f32,
        sunset_sec: f32,
        season: u8,
        moon_phase: u8,
    ) -> Self {
        let mut env = Self {
            sun_azimuth_deg: f32::NAN,
            sun_altitude_deg: f32::NAN,
            time_of_day_sec: time_of_day_sec.rem_euclid(DAY_LENGTH_SEC),
            sunrise_sec,
            sunset_sec,
            season,
            moon_phase,
            ..Self::neutral()
        };
        let (_, alt) = fallback_sun_angles(&env);
        // Shape follows the game's own curve (calendar.cpp
        // `sun_light_at`): rising with altitude, capped at the
        // 125 maximum once the sun is above ~60 degrees.
        env.sun_light =
            MAX_SUN_LIGHT * (alt.to_radians().sin() / 60f32.to_radians().sin()).clamp(0.0, 1.0);
        // Moon: phase strength (the game's own curve) scaled by how
        // high the antipode moon rides — fake scenes have no better
        // source, and the shape matches the live path's range.
        let moon_alt = -alt;
        env.moon_light = moon_light_for_phase(moon_phase) * moon_alt.to_radians().sin().max(0.0);
        env
    }

    /// Fake-data scene constructor for the §8 capture matrix
    /// (slice 10.6): [`Self::fake_day`]'s clear numerics with the
    /// weather fields folded the way the game exports them —
    /// `sun_light` re-derived through the type's own
    /// `light_multiplier`/`light_modifier` folding, so the
    /// numbers are self-consistent wire values, and each kind's
    /// numerics round-trip through [`classify_weather`] back to
    /// the kind (gate: `fake_weather_classifies_to_intended_kind`).
    /// Kinds outside the §8 matrix (Cloudy, HeavyRain, Snow)
    /// fall back to the clear numerics.
    pub fn fake_weather(
        time_of_day_sec: f32,
        sunrise_sec: f32,
        sunset_sec: f32,
        season: u8,
        moon_phase: u8,
        kind: WeatherKind,
    ) -> Self {
        let mut env = Self::fake_day(
            time_of_day_sec,
            sunrise_sec,
            sunset_sec,
            season,
            moon_phase,
        );
        let clear_sun = env.sun_light;
        match kind {
            WeatherKind::Rain => {
                env.weather_id = "weather_rain".to_string();
                env.light_multiplier = 0.75;
                env.light_modifier = -10.0;
                env.sun_multiplier = 0.7;
                env.precip_class = 2;
                env.rains = true;
                env.sight_penalty = 1.1;
            }
            WeatherKind::Fog => {
                env.weather_id = "weather_fog".to_string();
                env.light_multiplier = 0.9;
                env.light_modifier = -5.0;
                env.sun_multiplier = 0.8;
                // Vanilla fog's own value (the classifier's
                // pinned threshold comment, gate 10.2b).
                env.sight_penalty = 1.7;
            }
            WeatherKind::Thunderstorm => {
                env.weather_id = "weather_thunderstorm".to_string();
                env.light_multiplier = 0.6;
                // The thunder band (classifier comment: -40/-45
                // vs rainstorm's -35).
                env.light_modifier = -40.0;
                env.sun_multiplier = 0.3;
                env.precip_class = 3;
                env.rains = true;
                env.sight_penalty = 1.2;
                env.lightning_active = true;
            }
            _ => {}
        }
        if kind != WeatherKind::Clear {
            // The game's folding (the `sun_light` field doc):
            // what a game would have exported for this type.
            env.sun_light =
                (clear_sun * env.light_multiplier + env.light_modifier).max(0.0);
        }
        env
    }

    /// Clamp every numeric field into the vanilla range (§10 risk:
    /// a modded weather type with extreme numerics can look
    /// different, it cannot produce NaN or negative light). Angle
    /// fields are the exception: non-finite azimuth/altitude are
    /// the fallback-path signal and are preserved as NaN.
    pub fn sanitized(&self) -> Self {
        let mut e = self.clone();
        e.time_of_day_sec = finite_or(e.time_of_day_sec, 0.0).rem_euclid(DAY_LENGTH_SEC);
        e.sunrise_sec = finite_or(e.sunrise_sec, 21_600.0).clamp(0.0, DAY_LENGTH_SEC);
        e.sunset_sec = finite_or(e.sunset_sec, 64_800.0).clamp(0.0, DAY_LENGTH_SEC);
        if e.sunset_sec <= e.sunrise_sec {
            e.sunrise_sec = 21_600.0;
            e.sunset_sec = 64_800.0;
        }
        e.season = e.season.min(3);
        e.sun_light = finite_or(e.sun_light, 0.0).clamp(0.0, MAX_SUN_LIGHT);
        e.moon_light = finite_or(e.moon_light, 0.0).clamp(0.0, MAX_SUN_LIGHT);
        if !e.sun_azimuth_deg.is_finite() || !e.sun_altitude_deg.is_finite() {
            e.sun_azimuth_deg = f32::NAN;
            e.sun_altitude_deg = f32::NAN;
        } else {
            e.sun_azimuth_deg = e.sun_azimuth_deg.rem_euclid(360.0);
            e.sun_altitude_deg = e.sun_altitude_deg.clamp(-90.0, 90.0);
        }
        e.moon_phase = e.moon_phase.min(8);
        // Vanilla ranges (weather_type.json extremes, 2026-11-23):
        // light_multiplier is 1.0 unless authored; light_modifier
        // spans -100..2; sun_multiplier 0.01..1; sight_penalty 1..4.
        e.light_multiplier = finite_or(e.light_multiplier, 1.0).clamp(0.0, 2.0);
        e.light_modifier = finite_or(e.light_modifier, 0.0).clamp(-100.0, 10.0);
        e.sun_multiplier = finite_or(e.sun_multiplier, 1.0).clamp(0.0, 1.0);
        e.precip_class = e.precip_class.min(3);
        e.sight_penalty = finite_or(e.sight_penalty, 1.0).clamp(1.0, 4.0);
        e.tint_rgb = e.tint_rgb.map(|t| {
            [
                finite_or(t[0], 0.0).clamp(0.0, 1.0),
                finite_or(t[1], 0.0).clamp(0.0, 1.0),
                finite_or(t[2], 0.0).clamp(0.0, 1.0),
            ]
        });
        e.tint_strength = finite_or(e.tint_strength, 0.0).clamp(0.0, 1.0);
        e.temperature_c = finite_or(e.temperature_c, 20.0).clamp(-60.0, 60.0);
        e.windspeed_mph = finite_or(e.windspeed_mph, 0.0).clamp(0.0, 200.0);
        e.wind_dir_deg = finite_or(e.wind_dir_deg, 0.0).rem_euclid(360.0);
        e
    }
}

fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// CDDA's own moonlight curve (`calendar.cpp` `moon_light_at`):
/// phase folds at full (4), light = 1 + 1.5 per quarter step,
/// in game light units (so 1..7 against the sun's 125).
pub fn moon_light_for_phase(phase: u8) -> f32 {
    let p = (phase.min(8)) as i32;
    let folded = if p > 4 { 8 - p } else { p };
    1.0 + 1.5 * folded as f32
}

// ---------------------------------------------------------------------------
// Sun / moon direction
// ---------------------------------------------------------------------------

/// Convert the game's sun azimuth/altitude to a direction *toward
/// the sun* in engine world space (+X east, +Y north, +Z up).
///
/// CDDA azimuth is measured **from South, turning positive to the
/// west** (0 = south, 90 = west, 180 = north, 270 = east) — verified
/// against the game's own sundial (`display.cpp`: raw azimuth ≈ 0
/// at midday, when the sun is in the south) and its shadow test
/// (`sun_test.cpp`: `sunlight_angle` offsets point north all day).
/// CDDA's horizontal toward-sun vector is `(-sin az, cos az)` in
/// (east, south) map coordinates; our +Y is north, so the south
/// component negates:
///
/// ```text
/// dir = (-sin(az)·cos(alt),  -cos(az)·cos(alt),  sin(alt))
/// ```
pub fn sun_direction(azimuth_deg: f32, altitude_deg: f32) -> [f32; 3] {
    let az = azimuth_deg.to_radians();
    let alt = altitude_deg.to_radians();
    let c = alt.cos();
    [-az.sin() * c, -az.cos() * c, alt.sin()]
}

/// Peak sun elevation for the fake-data fallback path, by season
/// (decision 2A's ratified numbers: 65° summer, 25° winter;
/// spring/autumn are the linear midpoint, 45°).
pub fn max_elev_for_season(season: u8) -> f32 {
    match season {
        1 => 65.0,
        3 => 25.0,
        _ => 45.0,
    }
}

/// The §3.2 fake-data fallback sun path: a sine arc between the
/// game-supplied sunrise/sunset anchors by day, mirrored below the
/// horizon by night (so the antipode moon rides a continuous arc).
/// Returns `(azimuth_deg, altitude_deg)` in the game's convention.
/// Used only when the wire carries no real angles (fake scenes);
/// live data always wins (§3.2).
pub fn fallback_sun_angles(env: &GameEnvironment) -> (f32, f32) {
    let t = env.time_of_day_sec.rem_euclid(DAY_LENGTH_SEC);
    let rise = env.sunrise_sec;
    let set = env.sunset_sec;
    let max_elev = max_elev_for_season(env.season);
    if (rise..set).contains(&t) {
        let frac = (t - rise) / (set - rise);
        let alt = (std::f32::consts::PI * frac).sin() * max_elev;
        // East (270) through south (0) to west (90).
        let az = -90.0 + 180.0 * frac;
        (az.rem_euclid(360.0), alt)
    } else {
        let night_len = DAY_LENGTH_SEC - (set - rise);
        let since_set = (t - set).rem_euclid(DAY_LENGTH_SEC);
        let frac = if night_len > 0.0 {
            since_set / night_len
        } else {
            0.0
        };
        let alt = -(std::f32::consts::PI * frac).sin() * max_elev;
        // West (90) through north (180) to east (270).
        let az = 90.0 + 180.0 * frac;
        (az.rem_euclid(360.0), alt)
    }
}

// ---------------------------------------------------------------------------
// Weather classification (§2.1: from numerics, never from the id)
// ---------------------------------------------------------------------------

/// Derive the Phase 7 presentation class from the *numeric* wire
/// fields. Thresholds are pinned against the vanilla weather table
/// (`data/json/weather_type.json`, gate 10.2b):
///
/// - no precip + `sight_penalty ≥ 1.5` → Fog (vanilla fog is 1.7;
///   mist at 1.3 deliberately falls through to Cloudy);
/// - no precip + `sun_multiplier ≤ 0.45` → Cloudy, else Clear;
/// - precip + `rains == false` → Snow (all snow family);
/// - heavy rain + (`lightning_active` or `light_modifier ≤ -38`,
///   the thunder/lightning band −40/−45 vs rainstorm's −35) →
///   Thunderstorm, else HeavyRain;
/// - lighter rain → Rain (lightning with light rain still storms).
pub fn classify_weather(env: &GameEnvironment) -> WeatherKind {
    let e = env.sanitized();
    if e.precip_class == 0 {
        if e.sight_penalty >= 1.5 {
            WeatherKind::Fog
        } else if e.sun_multiplier <= 0.45 {
            WeatherKind::Cloudy
        } else {
            WeatherKind::Clear
        }
    } else if !e.rains {
        WeatherKind::Snow
    } else if e.precip_class >= 3 {
        if e.lightning_active || e.light_modifier <= -38.0 {
            WeatherKind::Thunderstorm
        } else {
            WeatherKind::HeavyRain
        }
    } else if e.lightning_active {
        WeatherKind::Thunderstorm
    } else {
        WeatherKind::Rain
    }
}

fn cloud_cover_for(kind: WeatherKind) -> f32 {
    match kind {
        WeatherKind::Clear => 0.05,
        WeatherKind::Cloudy => 0.55,
        WeatherKind::Rain => 0.80,
        WeatherKind::HeavyRain => 0.95,
        WeatherKind::Snow => 0.85,
        WeatherKind::Fog => 0.70,
        WeatherKind::Thunderstorm => 1.0,
    }
}

// ---------------------------------------------------------------------------
// SkyState (§3.1)
// ---------------------------------------------------------------------------

/// Precipitation presentation state: kind, rate, fall speed —
/// the existing [`WeatherParams`] precip fields, repackaged for
/// the fx emitter (slice 10.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecipKind {
    None,
    Rain,
    Snow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecipState {
    pub kind: PrecipKind,
    pub rate: f32,
    pub fall_speed: f32,
}

/// The derived per-frame sky state (§3.1, field-for-field).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyState {
    /// Toward the sun, world space.
    pub sun_dir: [f32; 3],
    /// Sun colour (weather tint applied; intensity is separate).
    pub sun_color: [f32; 3],
    pub sun_intensity: f32,
    /// Toward the moon — the antipode of the sun's direction.
    /// PROXY: CDDA has no moon position function (§11 Q2); the
    /// moon's *strength* (`moon_intensity`) is game truth.
    pub moon_dir: [f32; 3],
    pub moon_color: [f32; 3],
    pub moon_intensity: f32,
    /// Sky gradient stops.
    pub zenith: [f32; 3],
    pub horizon: [f32; 3],
    pub ground: [f32; 3],
    /// Hemisphere GI strength.
    pub ambient_scale: f32,
    pub fog_density: f32,
    /// = horizon colour, by construction.
    pub fog_color: [f32; 3],
    /// 0 clear .. 1 overcast.
    pub cloud_cover: f32,
    pub precip: PrecipState,
    /// The tempered Phase 7 bake factor (see `sky_state` docs).
    pub exposure: f32,
}

fn lerp3(a: [f32; 3], b: [f32; 3], f: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Apply the weather-type tint (wire `tint_rgb`/`tint_strength`)
/// on top of the Phase 7 params tint. No tint on the wire = params
/// tint only.
fn apply_tints(base: [f32; 3], params: &WeatherParams, env: &GameEnvironment) -> [f32; 3] {
    let mut c = [
        base[0] * params.tint[0],
        base[1] * params.tint[1],
        base[2] * params.tint[2],
    ];
    if let Some(tint) = env.tint_rgb {
        if env.tint_strength > 0.0 {
            c = lerp3(c, tint, env.tint_strength);
        }
    }
    c
}

/// Derive the full sky state from one environment sample.
///
/// `exposure` reuses the Phase 7 tempered factor verbatim — the
/// same `0.4 + 0.6 * (day_brightness * weather_scale)` formula as
/// [`crate::atmosphere::Environment::apply`], with the day
/// brightness from the existing `day_night_at` keyframe curve and
/// the weather scale from the derived kind's params. The bake's
/// contract (a tile's `lit_level` is already time-aware; never
/// double-darken) is unchanged.
pub fn sky_state(env: &GameEnvironment) -> SkyState {
    let env = env.sanitized();
    let kind = classify_weather(&env);
    let params = weather_params(kind);
    let cloud_cover = cloud_cover_for(kind);

    // Sun direction: game angles when the wire carries them, the
    // fallback sine path only for fake-data scenes (§3.2).
    let (az, alt) = if env.sun_azimuth_deg.is_finite() {
        (env.sun_azimuth_deg, env.sun_altitude_deg)
    } else {
        fallback_sun_angles(&env)
    };
    let sun_dir = sun_direction(az, alt);
    // Moon direction is the antipode proxy (see SkyState docs).
    let moon_dir = [-sun_dir[0], -sun_dir[1], -sun_dir[2]];

    // Intensities: the wire values are already weather-folded
    // game-side; normalize from CDDA light units. Cloud cover
    // collapses the *directional* term into the hemisphere term
    // (§5.3's overcast rule, applied at the state level).
    let sun_raw = (env.sun_light / MAX_SUN_LIGHT).clamp(0.0, 1.0);
    let sun_intensity = sun_raw * (1.0 - 0.85 * cloud_cover);
    // Moonlight render gain (Joey's call, 2026-11-23): CDDA's
    // moonlight scale (max 7 units at full moon against the
    // sun's 125) is a visibility scale, not linear luminance —
    // in the game those 7 units are a clearly visible "can
    // see, can't read" level. Normalized linearly by 125, a
    // full moon became 0.056 and vanished under the §5.3
    // collapse threshold; midnight rendered pitch black. The
    // render normalization instead maps the moon's peak
    // (7 units: phase 4, moon high) to 0.30. Phase and
    // altitude still modulate from there — a new moon
    // (1 unit) lands at ~0.043, near-dark as it should be.
    const MOON_RENDER_GAIN: f32 = 0.30 * MAX_SUN_LIGHT / 7.0;
    let moon_intensity = (env.moon_light * MOON_RENDER_GAIN / MAX_SUN_LIGHT).clamp(0.0, 1.0);

    // Sun colour warms as the sun drops toward the horizon.
    let warm = smoothstep(alt / 35.0);
    let sun_base = lerp3([1.0, 0.50, 0.26], [1.0, 0.97, 0.92], warm);
    let sun_color = apply_tints(sun_base, &params, &env);
    let moon_color = apply_tints([0.62, 0.72, 0.95], &params, &env);

    // Gradient stops: night <-> day by altitude (civil-twilight
    // band), twilight warmth around the horizon crossing, then the
    // weather blend toward the params fog colour under cloud.
    let day_factor = smoothstep((alt + 6.0) / 42.0);
    let mut zenith = lerp3([0.015, 0.025, 0.060], [0.30, 0.48, 0.78], day_factor);
    let mut horizon = lerp3([0.050, 0.070, 0.120], [0.65, 0.75, 0.85], day_factor);
    let mut ground = lerp3([0.020, 0.020, 0.025], [0.35, 0.38, 0.35], day_factor);
    let twilight = (1.0 - (alt.abs() / 14.0)).clamp(0.0, 1.0) * day_factor.max(0.15);
    horizon = lerp3(horizon, [0.98, 0.52, 0.30], twilight * 0.55);
    zenith = lerp3(zenith, [0.45, 0.35, 0.55], twilight * 0.25);
    let grey = cloud_cover * 0.55;
    zenith = lerp3(zenith, params.fog_color, grey);
    horizon = lerp3(horizon, params.fog_color, grey);
    ground = lerp3(ground, params.fog_color, grey * 0.5);
    let stop_scale = 0.45 + 0.55 * params.brightness_scale;
    zenith = scale3(zenith, stop_scale);
    horizon = scale3(horizon, stop_scale);
    ground = scale3(ground, stop_scale);
    if let Some(tint) = env.tint_rgb {
        if env.tint_strength > 0.0 {
            let f = env.tint_strength * 0.5;
            zenith = lerp3(zenith, tint, f);
            horizon = lerp3(horizon, tint, f);
        }
    }

    // Fog (§5.5): density from the derived kind's table entry,
    // scaled by the wire sight penalty when it exceeds 1.
    let sight_scale = if env.sight_penalty > 1.0 {
        env.sight_penalty
    } else {
        1.0
    };
    let fog_density = params.fog_density * sight_scale;
    let fog_color = horizon;

    let ambient_scale = (0.15 + 0.85 * (sun_raw + moon_intensity).clamp(0.0, 1.0))
        * (0.5 + 0.5 * params.brightness_scale);

    let precip = PrecipState {
        kind: match kind {
            WeatherKind::Rain | WeatherKind::HeavyRain | WeatherKind::Thunderstorm => {
                PrecipKind::Rain
            }
            WeatherKind::Snow => PrecipKind::Snow,
            _ => PrecipKind::None,
        },
        rate: params.precip_rate,
        fall_speed: params.precip_fall_speed,
    };

    let hour = env.time_of_day_sec / 3600.0;
    let exposure = 0.4 + 0.6 * (day_night_at(hour).brightness * params.brightness_scale);

    SkyState {
        sun_dir,
        sun_color,
        sun_intensity,
        moon_dir,
        moon_color,
        moon_intensity,
        zenith,
        horizon,
        ground,
        ambient_scale,
        fog_density,
        fog_color,
        cloud_cover,
        precip,
        exposure,
    }
}

// ---------------------------------------------------------------------------
// SkyClock — per-frame evaluation between game updates (§3.3)
// ---------------------------------------------------------------------------

fn weather_signature(env: &GameEnvironment) -> (WeatherKind, u8, bool) {
    (
        classify_weather(env),
        env.precip_class.min(3),
        env.lightning_active,
    )
}

/// Seconds from `from` to `to` on the day clock, in `[0, 86400)`.
fn wrap_diff(from: f32, to: f32) -> f32 {
    (to - from).rem_euclid(DAY_LENGTH_SEC)
}

fn lerp_angle_deg(a: f32, b: f32, f: f32) -> f32 {
    // Shortest arc.
    let mut d = (b - a).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    (a + d * f).rem_euclid(360.0)
}

fn interpolate_env(a: &GameEnvironment, b: &GameEnvironment, at_sec: f32) -> GameEnvironment {
    let span = wrap_diff(a.time_of_day_sec, b.time_of_day_sec);
    if !(1e-3..=DAY_LENGTH_SEC * 0.5).contains(&span) {
        // Same instant (or updates too far apart to interpolate
        // honestly): snap to the newest sample.
        let mut e = b.clone();
        e.time_of_day_sec = at_sec.rem_euclid(DAY_LENGTH_SEC);
        return e;
    }
    let f = (wrap_diff(a.time_of_day_sec, at_sec) / span).clamp(0.0, 1.0);
    let mut e = b.clone();
    e.time_of_day_sec = at_sec.rem_euclid(DAY_LENGTH_SEC);
    e.sun_light = a.sun_light + (b.sun_light - a.sun_light) * f;
    e.moon_light = a.moon_light + (b.moon_light - a.moon_light) * f;
    if a.sun_azimuth_deg.is_finite() && b.sun_azimuth_deg.is_finite() {
        e.sun_azimuth_deg = lerp_angle_deg(a.sun_azimuth_deg, b.sun_azimuth_deg, f);
        e.sun_altitude_deg = a.sun_altitude_deg + (b.sun_altitude_deg - a.sun_altitude_deg) * f;
    }
    e
}

fn copy_weather_fields(dst: &mut GameEnvironment, src: &GameEnvironment) {
    dst.weather_id = src.weather_id.clone();
    dst.light_multiplier = src.light_multiplier;
    dst.light_modifier = src.light_modifier;
    dst.sun_multiplier = src.sun_multiplier;
    dst.precip_class = src.precip_class;
    dst.rains = src.rains;
    dst.sight_penalty = src.sight_penalty;
    dst.tint_rgb = src.tint_rgb;
    dst.tint_strength = src.tint_strength;
    dst.lightning_active = src.lightning_active;
}

fn lerp_state(a: &SkyState, b: &SkyState, f: f32) -> SkyState {
    let l = |x: f32, y: f32| x + (y - x) * f;
    SkyState {
        sun_dir: lerp3(a.sun_dir, b.sun_dir, f),
        sun_color: lerp3(a.sun_color, b.sun_color, f),
        sun_intensity: l(a.sun_intensity, b.sun_intensity),
        moon_dir: lerp3(a.moon_dir, b.moon_dir, f),
        moon_color: lerp3(a.moon_color, b.moon_color, f),
        moon_intensity: l(a.moon_intensity, b.moon_intensity),
        zenith: lerp3(a.zenith, b.zenith, f),
        horizon: lerp3(a.horizon, b.horizon, f),
        ground: lerp3(a.ground, b.ground, f),
        ambient_scale: l(a.ambient_scale, b.ambient_scale),
        fog_density: l(a.fog_density, b.fog_density),
        fog_color: lerp3(a.fog_color, b.fog_color, f),
        cloud_cover: l(a.cloud_cover, b.cloud_cover),
        precip: PrecipState {
            kind: if f >= 0.5 {
                b.precip.kind
            } else {
                a.precip.kind
            },
            rate: l(a.precip.rate, b.precip.rate),
            fall_speed: l(a.precip.fall_speed, b.precip.fall_speed),
        },
        exposure: l(a.exposure, b.exposure),
    }
}

/// Per-frame sky evaluation (§3.3): the game is turn-based, the
/// camera is not. The clock interpolates time-of-day fields between
/// the last two environment updates, and cross-fades all
/// weather-derived fields over [`WEATHER_FADE_SEC`] game-seconds
/// when the weather signature changes — no brightness pop on a turn
/// boundary, and the sun crawls instead of stepping.
#[derive(Debug, Clone)]
pub struct SkyClock {
    prev: Option<GameEnvironment>,
    current: GameEnvironment,
    /// The environment whose weather was in effect before the most
    /// recent weather change (for the cross-fade's source side).
    prev_weather: Option<GameEnvironment>,
    /// `time_of_day_sec` at which the current weather took effect.
    weather_since_sec: f32,
}

impl SkyClock {
    pub fn new(env: GameEnvironment) -> Self {
        let weather_since_sec = env.time_of_day_sec;
        Self {
            prev: None,
            current: env,
            prev_weather: None,
            weather_since_sec,
        }
    }

    /// Record a new environment update (the §2.3 cadence: on
    /// weather change, or after ≥ 1 game-minute of time advance).
    pub fn push(&mut self, env: GameEnvironment) {
        let old = std::mem::replace(&mut self.current, env);
        if weather_signature(&old) != weather_signature(&self.current) {
            self.prev_weather = Some(old.clone());
            self.weather_since_sec = self.current.time_of_day_sec;
        }
        self.prev = Some(old);
    }

    /// Evaluate the sky state at a game time-of-day (seconds,
    /// fractional; wraps). Pure function of the recorded updates.
    pub fn evaluate(&self, at_sec: f32) -> SkyState {
        let at = at_sec.rem_euclid(DAY_LENGTH_SEC);
        let env_at = match &self.prev {
            Some(p) => interpolate_env(p, &self.current, at),
            None => {
                let mut e = self.current.clone();
                e.time_of_day_sec = at;
                e
            }
        };
        let current_state = sky_state(&env_at);
        if let Some(prev_weather) = &self.prev_weather {
            let since = wrap_diff(self.weather_since_sec, at);
            if since < WEATHER_FADE_SEC {
                // Inside the fade window: blend the previous
                // weather's state into the current one.
                let f = smoothstep(since / WEATHER_FADE_SEC);
                let mut old_env = env_at.clone();
                copy_weather_fields(&mut old_env, prev_weather);
                let old_state = sky_state(&old_env);
                return lerp_state(&old_state, &current_state, f);
            } else if since > DAY_LENGTH_SEC * 0.5 {
                // `at` sits just *before* the weather change on the
                // day clock: the previous weather is still in
                // effect there.
                let mut old_env = env_at;
                copy_weather_fields(&mut old_env, prev_weather);
                return sky_state(&old_env);
            }
        }
        current_state
    }
}
