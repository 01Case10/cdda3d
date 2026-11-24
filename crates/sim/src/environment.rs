//! Calendar, weather and the EnvironmentUpdate (slice S3).
//!
//! A faithful port of the game's own formulas, so sim gates and
//! fork gates mean the same thing. Sources are the fork base
//! tree, CDDA 0.I-1 (`~/workspace/cdda-stable/cdda-0.I-1/src`):
//!
//! - **Turn duration: 1 turn = 1 second.** `calendar::turn`
//!   advances by `1_turns` once per `do_turn` (do_turn.cpp:475),
//!   and `time_duration::from_seconds(t) = time_duration(t)` —
//!   the duration's unit IS the turn (calendar.h:220-257). A day
//!   is therefore 86,400 turns. (This supersedes the plan §3.2
//!   note "6 s/turn default", which was written against the old
//!   hand-authored fixture convention — `sample_environment`'s
//!   "day 21 noon" turn 309600 assumes 14,400 turns/day.)
//! - Location: `static lat_long location = { 42.36, -71.06 }`
//!   (calendar.cpp:31). Season length default 91 days
//!   (options.cpp "SEASON_LENGTH" default), year = 4 seasons =
//!   364 days; turn_zero is the vernal equinox (Spring day 1);
//!   eternal_season off, so `season_effective_time` is identity.
//! - `sun_azimuth_altitude` (calendar.cpp:186): ported below with
//!   f64 angles in degrees; azimuth is from South, west-positive
//!   (the schema's documented convention).
//! - `sun_light_at` (calendar.cpp, "Sunlight rises exponentially
//!   ..."): piecewise on solar altitude with the dawn constants
//!   astronomical -18, nautical -12, civil -6, sunrise -1 deg.
//! - `moon_light_at` (calendar.cpp:371, static upstream — the
//!   fork's backport B3 un-statics it): phase folded at
//!   MOON_PHASE_MAX/2, `1 + 1.5 * folded` (moonlight_per_quarter
//!   = 1.5, calendar.cpp:23). `get_moon_phase`: phase switches at
//!   noon (`p - turn_zero + 12h`, floored to days), phase =
//!   round(days / 29.530588853 * 8) % 8 (lunar_month, calendar.h).
//! - `incident_sunlight` / `incident_moonlight` (weather.cpp:142
//!   and the master-tree twin): max(0, light * light_multiplier +
//!   light_modifier). `sun_multiplier` does NOT enter these — it
//!   is carried on the wire for the engine's sky, not for lm.
//! - `game::natural_light_level(z)` (game.cpp:4414): z < 0 (or
//!   above the overmap) -> LIGHT_AMBIENT_MINIMAL (3.7); otherwise
//!   (sun_light_at + moon_light_at) * light_multiplier +
//!   light_modifier. Artifact/DIM-event modifiers are not
//!   modelled (no artifacts in the sim).
//! - `Character::get_vision_threshold` (character.cpp): the sim's
//!   avatar is the default character — PER 8, no night-vision
//!   modes, no debug vision. range = PER/3;
//!   threshold_for_range(r) = 3.7 / exp(r * (2.3/60)) - 0.01;
//!   dimming = 1 + (ambient - 3.7) / (10 - 3.7);
//!   threshold = min(LIGHT_AMBIENT_LOW = 3.5, tfr * dimming).
//!
//! Weather presets resolve `data/json/weather_type.json` (0.I-1)
//! numerics by hand into the table below. There is NO "clear"
//! entry in that file: clear is the game's built-in default
//! weather, a default-constructed `weather_type` (weather_type.h:
//! light_multiplier 1.0, light_modifier 0, sun_multiplier 1.0,
//! sight_penalty 0.0, precip none, rains false). Wire ids use the
//! fixture vocabulary established by `proto::samples`
//! ("weather_clear", "weather_rain", "weather_thunderstorm").
//! `lightning_active` is a weather_manager state flag, not a type
//! field: the sim sets it deterministically while a thunder-class
//! preset is active (in-game it flickers per strike — documented
//! simplification). Temperature/wind have no sim model: the
//! sample fixture's constants (20 C, 5 mph, 90 deg) are carried,
//! weather-independent, exactly as the fixture does.
//!
//! Sunrise/sunset: upstream computes them with an iterative
//! `sun_at_altitude(-1 deg, ...)` solver (calendar.cpp ~296-323).
//! The sim solves the same equation numerically — bisection on
//! the ported altitude function over the day's two halves — and
//! reports seconds-of-day, matching the adapter's
//! `(sunrise(now) - turn_zero) % 1 day` shape
//! (voxel_real_adapter.cpp `environment()`).

use proto::EnvironmentUpdateT;

/// Seconds in a game day: 86,400 turns at 1 s/turn.
pub const DAY_SECONDS: i64 = 86_400;
/// EnvironmentUpdate cadence: on change + every 10 game-minutes
/// of game time (Phase 10 §2.3, ratified over the per-turn
/// alternative).
pub const ENV_CADENCE_SECONDS: i64 = 600;

// Light constants (0.I-1 lightmap.h).
pub const LIGHT_SOURCE_BRIGHT: f32 = 10.0;
pub const LIGHT_AMBIENT_LOW: f32 = 3.5;
pub const LIGHT_AMBIENT_DIM: f32 = 5.0;
pub const LIGHT_AMBIENT_LIT: f32 = 10.0;
pub const LIGHT_AMBIENT_MINIMAL: f32 = 3.7;
/// LIGHT_TRANSPARENCY_OPEN_AIR = 2.3 / MAX_VIEW_DISTANCE(60)
/// (lightmap.h): the per-tile attenuation coefficient of open
/// air, for both light propagation and the LOS flood.
pub const LIGHT_TRANSPARENCY_OPEN_AIR: f32 = 2.3 / 60.0;

const LATITUDE_DEG: f64 = 42.36; // calendar.cpp:31
const LONGITUDE_DEG: f64 = -71.06; // calendar.cpp:31
const SEASON_LENGTH_DAYS: i64 = 91; // options.cpp SEASON_LENGTH default
const YEAR_DAYS: i64 = 4 * SEASON_LENGTH_DAYS;
const LUNAR_MONTH_DAYS: f64 = 29.530_588_853; // calendar.h lunar_month()
const MOONLIGHT_PER_QUARTER: f32 = 1.5; // calendar.cpp:23

/// One resolved weather preset: the `weather_type` numerics the
/// EnvironmentUpdate carries, resolved from 0.I-1's
/// weather_type.json (see module docs for "clear").
#[derive(Debug, Clone, PartialEq)]
pub struct WeatherState {
    /// Scene vocabulary name ("clear", "sunny", ...).
    pub preset: String,
    /// Wire id, fixture vocabulary ("weather_clear", ...).
    pub wire_id: String,
    pub light_multiplier: f32,
    pub light_modifier: f32,
    pub sun_multiplier: f32,
    /// 0 none, 1 very_light, 2 light, 3 heavy (schema).
    pub precip_class: u8,
    pub rains: bool,
    pub sight_penalty: f32,
    pub lightning_active: bool,
}

impl WeatherState {
    fn new(
        preset: &str,
        wire_id: &str,
        light_multiplier: f32,
        light_modifier: f32,
        sun_multiplier: f32,
        precip_class: u8,
        rains: bool,
        sight_penalty: f32,
        lightning_active: bool,
    ) -> WeatherState {
        WeatherState {
            preset: preset.to_string(),
            wire_id: wire_id.to_string(),
            light_multiplier,
            light_modifier,
            sun_multiplier,
            precip_class,
            rains,
            sight_penalty,
            lightning_active,
        }
    }
}

/// Resolve a scene weather name to its preset. Names are the
/// weather_type.json ids (plus "storm" as an alias for the
/// thunder type, matching the stub's env-script vocabulary).
pub fn weather_preset(name: &str) -> Option<WeatherState> {
    Some(match name {
        // Built-in default weather (no JSON entry): all defaults.
        "clear" => WeatherState::new(
            "clear",
            "weather_clear",
            1.0,
            0.0,
            1.0,
            0,
            false,
            0.0,
            false,
        ),
        // weather_type.json "sunny": light_modifier +2.
        "sunny" => WeatherState::new(
            "sunny",
            "weather_sunny",
            1.0,
            2.0,
            1.0,
            0,
            false,
            0.0,
            false,
        ),
        // "cloudy": light_modifier -20, sun_multiplier 0.3.
        "cloudy" => WeatherState::new(
            "cloudy",
            "weather_cloudy",
            1.0,
            -20.0,
            0.3,
            0,
            false,
            0.0,
            false,
        ),
        // "rain": modifier -30, sun 0.03, precip heavy, rains,
        // sight_penalty 1.1.
        "rain" => WeatherState::new(
            "rain",
            "weather_rain",
            1.0,
            -30.0,
            0.03,
            3,
            true,
            1.1,
            false,
        ),
        // "thunder": modifier -40, sun 0.01, precip heavy, rains,
        // sight_penalty 1.2; the manager's lightning flag is
        // active while a thunder-class weather runs.
        "storm" | "thunder" => WeatherState::new(
            "storm",
            "weather_thunderstorm",
            1.0,
            -40.0,
            0.01,
            3,
            true,
            1.2,
            true,
        ),
        // "lightning": modifier -45, sun 0.01, precip heavy,
        // sight_penalty 1.25, lightning active.
        "lightning" => WeatherState::new(
            "lightning",
            "weather_lightning",
            1.0,
            -45.0,
            0.01,
            3,
            true,
            1.25,
            true,
        ),
        _ => return None,
    })
}

/// The scene `time` presets: time-of-day on day 0 (the vernal
/// equinox), as an absolute calendar turn. `turn = <int>` in the
/// scene wins over these (handled by the caller).
pub fn time_preset_turn(time: &str) -> Option<i64> {
    Some(match time {
        "midnight" => 0,
        "dawn" => 21_600,
        "noon" => 43_200,
        "dusk" => 64_800,
        _ => return None,
    })
}

/// (right ascension, declination) in degrees — sun_ra_declination
/// (calendar.cpp). `days_since_epoch` counts from the vernal
/// equinox (turn_zero), timezone-shifted by longitude.
fn sun_ra_declination(turn: i64) -> (f64, f64) {
    let timezone_sec = LONGITUDE_DEG / 15.0 * 3600.0; // angle_to_time
    let days_since_epoch = (turn as f64 - timezone_sec) / DAY_SECONDS as f64;
    let angle_per_day = 360.0 / YEAR_DAYS as f64;
    let mean_long = angle_per_day * days_since_epoch;
    let mean_anomaly = 77.0 + mean_long;
    let ecliptic_longitude = mean_long
        + 1.915 * mean_anomaly.to_radians().sin()
        + 0.020 * (2.0 * mean_anomaly).to_radians().sin();
    const OBLIQUITY: f64 = 23.439_279;
    let eclip = (
        ecliptic_longitude.to_radians().cos(),
        ecliptic_longitude.to_radians().sin(),
    );
    let rot = (
        eclip.0,
        eclip.1 * OBLIQUITY.to_radians().cos(),
        eclip.1 * OBLIQUITY.to_radians().sin(),
    );
    let right_ascension = rot.1.atan2(rot.0).to_degrees();
    let declination = rot.2.asin().to_degrees();
    (right_ascension, declination)
}

/// Sidereal time in degrees — sidereal_time_at (calendar.cpp).
fn sidereal_time(turn: i64) -> f64 {
    let timezone_sec = LONGITUDE_DEG / 15.0 * 3600.0;
    let days_since_epoch = (turn as f64 - timezone_sec) / DAY_SECONDS as f64;
    let angle_per_day = 360.0 / YEAR_DAYS as f64;
    180.0 + (360.0 + angle_per_day) * days_since_epoch + LONGITUDE_DEG
}

/// `sun_azimuth_altitude(turn)` (calendar.cpp:186): (azimuth,
/// altitude) in degrees. Azimuth from South, west-positive,
/// normalised to 0..360; altitude negative below the horizon.
pub fn sun_azimuth_altitude(turn: i64) -> (f64, f64) {
    let (ra, declination) = sun_ra_declination(turn);
    let hour_angle = sidereal_time(turn) - ra;
    let intermediate = (
        hour_angle.to_radians().cos() * declination.to_radians().cos(),
        hour_angle.to_radians().sin() * declination.to_radians().cos(),
        declination.to_radians().sin(),
    );
    let horizontal = (
        -intermediate.0 * LATITUDE_DEG.to_radians().sin()
            + intermediate.2 * LATITUDE_DEG.to_radians().cos(),
        intermediate.1,
        intermediate.0 * LATITUDE_DEG.to_radians().cos()
            + intermediate.2 * LATITUDE_DEG.to_radians().sin(),
    );
    let azimuth = (-horizontal.1.atan2(horizontal.0).to_degrees() + 180.0).rem_euclid(360.0);
    let altitude = horizontal.2.asin().to_degrees();
    (azimuth, altitude)
}

/// `sun_light_at` (calendar.cpp): incident sunlight strength
/// from solar altitude alone (weather not yet folded in).
pub fn sun_light_at(turn: i64) -> f32 {
    let alt = sun_azimuth_altitude(turn).1;
    let v = if alt < -18.0 {
        0.0
    } else if alt <= -12.0 {
        3.7 * (((alt + 18.0) / 6.0).exp2() - 1.0)
    } else if alt <= -6.0 {
        1.3 * (((alt + 12.0) / 6.0).exp2() - 1.0) + 3.7
    } else if alt <= -1.0 {
        55.0 * (((alt + 6.0) / 5.0).exp2() - 1.0) + 5.0
    } else if alt <= 60.0 {
        (65.0 / 61.0) * alt + 65.0 / 61.0 + 60.0
    } else {
        125.0
    };
    v as f32
}

/// `get_moon_phase(turn)` (calendar.cpp): 0 NEW .. 4 FULL .. 7.
pub fn moon_phase(turn: i64) -> u8 {
    // Phase switches at noon: middays since turn_zero.
    let num_middays = (turn + DAY_SECONDS / 2).div_euclid(DAY_SECONDS);
    let phase_change = num_middays as f64 / LUNAR_MONTH_DAYS;
    ((phase_change * 8.0).round() as i64).rem_euclid(8) as u8
}

/// `moon_light_at` (calendar.cpp:371; fork backport B3).
pub fn moon_light_at(turn: i64) -> f32 {
    let mut phase = moon_phase(turn) as i32;
    if phase > 4 {
        phase = 8 - phase;
    }
    1.0 + MOONLIGHT_PER_QUARTER * phase as f32
}

/// `incident_sunlight(wtype, t)` (weather.cpp:142).
pub fn incident_sunlight(w: &WeatherState, turn: i64) -> f32 {
    (sun_light_at(turn) * w.light_multiplier + w.light_modifier).max(0.0)
}

/// `incident_moonlight(wtype, t)` (master weather.cpp:153,
/// backport B3).
pub fn incident_moonlight(w: &WeatherState, turn: i64) -> f32 {
    (moon_light_at(turn) * w.light_multiplier + w.light_modifier).max(0.0)
}

/// `game::natural_light_level(z)` (game.cpp:4414), artifact
/// modifiers excluded. NOT clamped below: with a heavy weather
/// modifier the game value can go negative and the light code
/// treats it as darkness; callers clamp for lm use.
pub fn natural_light_level(w: &WeatherState, turn: i64, z: i32) -> f32 {
    if z < 0 {
        return LIGHT_AMBIENT_MINIMAL;
    }
    (sun_light_at(turn) + moon_light_at(turn)) * w.light_multiplier + w.light_modifier
}

/// `season_of_year(turn)` (calendar.h): 0 spring .. 3 winter.
pub fn season_of_year(turn: i64) -> u8 {
    ((turn.div_euclid(DAY_SECONDS) / SEASON_LENGTH_DAYS).rem_euclid(4)) as u8
}

/// `light_above_threshold(apparent, threshold)` (master
/// smooth_lighting.cpp:30): the fraction of the way from the
/// vision threshold to full light, clamped 0..1 — the
/// smooth-lighting value the wire `light_frac` carries for
/// LOW/LIT tiles (Addendum 11).
pub fn light_above_threshold(apparent: f32, vision_threshold: f32) -> f32 {
    ((apparent - vision_threshold) / (LIGHT_AMBIENT_LIT - vision_threshold)).clamp(0.0, 1.0)
}

/// `Character::get_vision_threshold(ambient)` (character.cpp)
/// for the sim's default avatar (PER 8, no vision modes).
pub fn vision_threshold(ambient_at_player: f32) -> f32 {
    let range = 8.0f32 / 3.0;
    let threshold_for_range =
        LIGHT_AMBIENT_MINIMAL / (range * LIGHT_TRANSPARENCY_OPEN_AIR).exp() - 0.01;
    let dimming = 1.0
        + (ambient_at_player - LIGHT_AMBIENT_MINIMAL) / (LIGHT_AMBIENT_LIT - LIGHT_AMBIENT_MINIMAL);
    LIGHT_AMBIENT_LOW.min(threshold_for_range * dimming)
}

/// Seconds-of-day of today's sunrise and sunset, solving
/// altitude = -1 deg (the upstream `sunrise_angle`) by bisection
/// on the ported altitude function over each half-day — the same
/// equation the game's iterative `sun_at_altitude` solver
/// answers. At the sim's fixed location/season lengths the sun
/// always crosses -1 deg twice a day.
pub fn sunrise_sunset_sec(turn: i64) -> (f32, f32) {
    let day_start = turn.div_euclid(DAY_SECONDS) * DAY_SECONDS;
    let solve = |lo: i64, hi: i64| -> f32 {
        // Invariant: altitude(lo) and altitude(hi) straddle -1.
        let (mut lo, mut hi) = (lo as f64, hi as f64);
        for _ in 0..48 {
            let mid = (lo + hi) / 2.0;
            let alt_lo = sun_azimuth_altitude(day_start + lo as i64).1;
            let alt_mid = sun_azimuth_altitude(day_start + mid as i64).1;
            if (alt_lo + 1.0) * (alt_mid + 1.0) <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        ((lo + hi) / 2.0) as f32
    };
    let sunrise = solve(0, DAY_SECONDS / 2);
    let sunset = solve(DAY_SECONDS / 2, DAY_SECONDS);
    (sunrise, sunset)
}

/// Build the wire EnvironmentUpdate for a game state, in the
/// adapter's shape (voxel_real_adapter.cpp `environment()`):
/// turn/time-of-day in seconds, sunrise/sunset as
/// seconds-of-day, weather numerics resolved from the type.
pub fn build_environment(seq: u64, epoch: u64, turn: i64, w: &WeatherState) -> EnvironmentUpdateT {
    let (azimuth, altitude) = sun_azimuth_altitude(turn);
    let (sunrise, sunset) = sunrise_sunset_sec(turn);
    let mut env = EnvironmentUpdateT::default();
    env.seq = seq;
    env.epoch = epoch;
    env.turn = turn;
    env.time_of_day_sec = turn.rem_euclid(DAY_SECONDS) as f32;
    env.sunrise_sec = sunrise;
    env.sunset_sec = sunset;
    env.season = season_of_year(turn);
    env.sun_light = incident_sunlight(w, turn);
    env.moon_light = incident_moonlight(w, turn);
    env.sun_azimuth_deg = azimuth as f32;
    env.sun_altitude_deg = altitude as f32;
    env.moon_phase = moon_phase(turn);
    env.weather_id = Some(w.wire_id.clone());
    env.light_multiplier = w.light_multiplier;
    env.light_modifier = w.light_modifier;
    env.sun_multiplier = w.sun_multiplier;
    env.precip_class = w.precip_class;
    env.rains = w.rains;
    env.sight_penalty = w.sight_penalty;
    // No weather_type on the 0.I-1 base carries a tint (backport
    // pack finding: only the master-only scarlet-dimension types
    // do), so tint is absent / strength 0.
    env.tint_rgb = None;
    env.tint_strength = 0.0;
    // No temperature/wind model in the sim: the sample fixture's
    // constants, documented in the module header.
    env.temperature_c = 20.0;
    env.windspeed_mph = 5.0;
    env.wind_dir_deg = 90.0;
    env.lightning_active = w.lightning_active;
    env
}
