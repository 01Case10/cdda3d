//! Phase 10 gate tests 10.2a / 10.2b (docs/phase10-sky-gi.md §8),
//! CPU-only, in `crates/lighting` — the SkyState derivation from the
//! frozen `GameEnvironment` wire mirror (§2.2/§11). No GPU, no proto
//! crate: `GameEnvironment` is defined locally in `lighting::sky`.
//!
//! Vanilla weather numerics below are copied from the game's own
//! `data/json/weather_type.json` (audit checkout, 2026-11-23);
//! defaults for absent fields per `src/weather_type.h`
//! (light_multiplier 1.0, sun_multiplier 1.0, light_modifier 0).

use lighting::sky::{
    classify_weather, fallback_sun_angles, max_elev_for_season, sky_state, sun_direction,
    GameEnvironment, PrecipKind, SkyClock,
};
use lighting::WeatherKind;

const SUNRISE: f32 = 21_600.0; // 06:00
const SUNSET: f32 = 64_800.0; // 18:00
const NOON: f32 = 43_200.0;
const MIDNIGHT: f32 = 0.0;

/// A live-path env (game-supplied azimuth/altitude) at noon, clear.
fn live_noon() -> GameEnvironment {
    GameEnvironment {
        time_of_day_sec: NOON,
        sun_azimuth_deg: 0.0,
        sun_altitude_deg: 60.0,
        ..GameEnvironment::neutral()
    }
}

/// Vanilla-numerics env for one weather id, at noon on the live path.
fn vanilla(id: &str) -> GameEnvironment {
    // (precip_class, rains, light_modifier, sun_multiplier, sight_penalty)
    // precip_class: 0 none, 1 very_light, 2 light, 3 heavy (§2.2).
    let (precip, rains, light_mod, sun_mult, sight) = match id {
        "sunny" => (0, false, 2.0, 1.0, 1.0),
        "cloudy" => (0, false, -20.0, 0.3, 1.0),
        "light_drizzle" => (1, true, -10.0, 0.2, 1.01),
        "drizzle" => (2, true, -20.0, 0.1, 1.03),
        "rain" => (3, true, -30.0, 0.03, 1.1),
        "rainstorm" => (3, true, -35.0, 0.01, 1.2),
        "thunder" => (3, true, -40.0, 0.01, 1.2),
        "lightning" => (3, true, -45.0, 0.01, 1.25),
        "flurries" => (2, false, -15.0, 0.3, 1.12),
        "snowing" => (3, false, -20.0, 0.15, 1.3),
        "snowstorm" => (3, false, -30.0, 0.01, 1.2),
        "mist" => (0, false, -20.0, 0.3, 1.3),
        "fog" => (0, false, -40.0, 0.3, 1.7),
        other => panic!("no vanilla fixture for {other}"),
    };
    GameEnvironment {
        weather_id: id.to_string(),
        precip_class: precip,
        rains,
        light_modifier: light_mod,
        sun_multiplier: sun_mult,
        sight_penalty: sight,
        ..live_noon()
    }
}

fn max_step(a: &lighting::sky::SkyState, b: &lighting::sky::SkyState) -> f32 {
    let mut m = 0.0f32;
    let mut acc = |x: f32, y: f32| m = m.max((x - y).abs());
    for c in 0..3 {
        acc(a.sun_dir[c], b.sun_dir[c]);
        acc(a.sun_color[c], b.sun_color[c]);
        acc(a.moon_dir[c], b.moon_dir[c]);
        acc(a.moon_color[c], b.moon_color[c]);
        acc(a.zenith[c], b.zenith[c]);
        acc(a.horizon[c], b.horizon[c]);
        acc(a.ground[c], b.ground[c]);
        acc(a.fog_color[c], b.fog_color[c]);
    }
    acc(a.sun_intensity, b.sun_intensity);
    acc(a.moon_intensity, b.moon_intensity);
    acc(a.ambient_scale, b.ambient_scale);
    acc(a.fog_density, b.fog_density);
    acc(a.cloud_cover, b.cloud_cover);
    acc(a.precip.rate, b.precip.rate);
    acc(a.precip.fall_speed, b.precip.fall_speed);
    acc(a.exposure, b.exposure);
    m
}

// ------------------------------------------------------------- 10.2a

#[test]
fn gate_10_2a_fallback_anchors_exact() {
    // Summer day, 06:00–18:00. At both anchors the sun sits exactly
    // on the horizon, east at sunrise and west at sunset; elevation
    // peaks mid-span at the season's max_elev (decision 2A numbers).
    let rise = GameEnvironment::fake_day(SUNRISE, SUNRISE, SUNSET, 1, 4);
    let (az, alt) = fallback_sun_angles(&rise);
    assert!(alt.abs() < 1e-3, "sunrise altitude {alt}");
    assert!((az - 270.0).abs() < 1e-3, "sunrise azimuth {az} (east)");
    let s = sky_state(&rise);
    assert!(s.sun_dir[2].abs() < 1e-3, "sun on horizon: {:?}", s.sun_dir);
    assert!(s.sun_dir[0] > 0.99, "sun in the east: {:?}", s.sun_dir);
    assert!(s.sun_intensity < 1e-3, "no sun light at the anchor");

    let set = GameEnvironment::fake_day(SUNSET, SUNRISE, SUNSET, 1, 4);
    let (az, alt) = fallback_sun_angles(&set);
    assert!(alt.abs() < 1e-3, "sunset altitude {alt}");
    assert!((az - 90.0).abs() < 1e-3, "sunset azimuth {az} (west)");
    let s = sky_state(&set);
    assert!(s.sun_dir[0] < -0.99, "sun in the west: {:?}", s.sun_dir);

    // Mid-span peak = max_elev for the season: 65 summer, 25 winter.
    assert_eq!(max_elev_for_season(1), 65.0);
    assert_eq!(max_elev_for_season(3), 25.0);
    let mid = GameEnvironment::fake_day(NOON, SUNRISE, SUNSET, 1, 4);
    let (_, alt) = fallback_sun_angles(&mid);
    assert!((alt - 65.0).abs() < 1e-3, "summer peak {alt}");
    let mid_w = GameEnvironment::fake_day(NOON, SUNRISE, SUNSET, 3, 4);
    let (_, alt_w) = fallback_sun_angles(&mid_w);
    assert!((alt_w - 25.0).abs() < 1e-3, "winter peak {alt_w}");
    // Elevation is a peak, not a plateau.
    for t in [32_400.0, 54_000.0] {
        let e = GameEnvironment::fake_day(t, SUNRISE, SUNSET, 1, 4);
        assert!(fallback_sun_angles(&e).1 < alt);
    }
}

#[test]
fn gate_10_2a_live_conversion_signs() {
    // CDDA azimuth: from South, west-positive (0 = south, 90 = west,
    // 180 = north, 270 = east) — cross-checked against the game's
    // sundial (display.cpp: raw azimuth ~0 at midday) and its shadow
    // test (sun_test.cpp: sunlight offset points north all day).
    // Our world: +X east, +Y north, +Z up.
    let up = sun_direction(180.0, 90.0);
    assert!(
        (up[0]).abs() < 1e-5 && up[1].abs() < 1e-5 && (up[2] - 1.0).abs() < 1e-5,
        "altitude 90 = overhead regardless of azimuth: {up:?}"
    );
    let east = sun_direction(270.0, 0.0);
    assert!(
        (east[0] - 1.0).abs() < 1e-5 && east[1].abs() < 1e-5 && east[2].abs() < 1e-5,
        "sunrise azimuth = east horizon: {east:?}"
    );
    let west = sun_direction(90.0, 0.0);
    assert!(
        (west[0] + 1.0).abs() < 1e-5,
        "sunset azimuth = west: {west:?}"
    );
    let south_noon = sun_direction(0.0, 60.0);
    assert!(
        south_noon[1] < 0.0 && south_noon[2] > 0.8,
        "midday sun is south and high: {south_noon:?}"
    );
    let north = sun_direction(180.0, 0.0);
    assert!(
        (north[1] - 1.0).abs() < 1e-5,
        "azimuth 180 = north: {north:?}"
    );
    // Unit length everywhere on a coarse grid.
    for az in (0..360).step_by(15) {
        for alt in (-30..=90).step_by(15) {
            let d = sun_direction(az as f32, alt as f32);
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4, "unit dir at {az}/{alt}");
        }
    }
}

#[test]
fn gate_10_2a_full_day_sweep_no_steps() {
    // Fallback path, constant clear weather: stepping 1 game-minute
    // over a full day never moves any SkyState field by > 0.01
    // (the 7.6 smoothness bound, now over the full state, §3.3).
    let mut prev = sky_state(&GameEnvironment::fake_day(0.0, SUNRISE, SUNSET, 1, 4));
    for minute in 1..=(24 * 60) {
        let t = (minute as f32 * 60.0) % 86_400.0;
        let cur = sky_state(&GameEnvironment::fake_day(t, SUNRISE, SUNSET, 1, 4));
        let step = max_step(&prev, &cur);
        assert!(step <= 0.01, "step {step} at minute {minute}");
        prev = cur;
    }
}

#[test]
fn gate_10_2a_midnight_is_moon_dominant() {
    let mid = sky_state(&GameEnvironment::fake_day(MIDNIGHT, SUNRISE, SUNSET, 1, 4));
    assert!(mid.sun_intensity < 1e-3, "no sun at midnight");
    assert!(
        mid.moon_intensity > mid.sun_intensity,
        "moon dominates at midnight"
    );
    // The render gain (MOON_RENDER_GAIN in sky.rs): the
    // moon's peak maps to 0.30 render intensity, so a full
    // moon at midnight is real light, not the 0.056 the
    // linear 125-normalization produced. The fake day's
    // altitude fold keeps midnight just under the peak.
    assert!(
        mid.moon_intensity > 0.05 && mid.moon_intensity <= 0.30,
        "full-moon render intensity should sit near the gained peak: {}",
        mid.moon_intensity
    );
    assert!(
        mid.moon_dir[2] > 0.5,
        "full-ish moon high at midnight: {:?}",
        mid.moon_dir
    );
    // A new moon (phase 0) is near-dark: moon intensity collapses.
    let new_moon = sky_state(&GameEnvironment::fake_day(MIDNIGHT, SUNRISE, SUNSET, 1, 0));
    assert!(new_moon.moon_intensity < mid.moon_intensity * 0.3);
    // Noon is the opposite: sun dominates, moon is negligible.
    let noon = sky_state(&GameEnvironment::fake_day(NOON, SUNRISE, SUNSET, 1, 4));
    assert!(noon.sun_intensity > 0.9 && noon.moon_intensity < 0.05);
}

#[test]
fn gate_10_2a_moon_arc_is_phase_aware_and_twilight_scatters() {
    // Proxy v2: at full moon the direction is still the exact
    // antipode (v1 geometry preserved bit-for-bit).
    let mid = sky_state(&GameEnvironment::fake_day(MIDNIGHT, SUNRISE, SUNSET, 1, 4));
    for i in 0..3 {
        assert!(
            (mid.moon_dir[i] + mid.sun_dir[i]).abs() < 1e-5,
            "full moon must be the exact antipode: moon {:?} sun {:?}",
            mid.moon_dir,
            mid.sun_dir
        );
    }
    // A first-quarter moon (phase 2) rides high at dusk — the
    // v1 antipode put it below the horizon (intensity 0).
    let dusk_q = sky_state(&GameEnvironment::fake_day(17.5 * 3600.0, SUNRISE, SUNSET, 1, 2));
    assert!(
        dusk_q.moon_dir[2] > 0.4 && dusk_q.moon_intensity > 0.05,
        "first-quarter moon should be up and lit at dusk: dir {:?} int {}",
        dusk_q.moon_dir,
        dusk_q.moon_intensity
    );
    // A last-quarter moon (phase 6) rides high at dawn.
    let dawn_q = sky_state(&GameEnvironment::fake_day(6.5 * 3600.0, SUNRISE, SUNSET, 1, 6));
    assert!(
        dawn_q.moon_dir[2] > 0.4 && dawn_q.moon_intensity > 0.05,
        "last-quarter moon should be up and lit at dawn: dir {:?} int {}",
        dawn_q.moon_dir,
        dawn_q.moon_intensity
    );
    // Twilight scatter: the hemisphere strength at dawn
    // (full moon, so the moon term is ~0 in both) sits well
    // above its sun-driven value — measured 0.295 before the
    // scatter term existed, 0.413 with it (dawn sun alt
    // 8.5 deg, inside the +/-14 deg band); midnight gets no
    // scatter.
    let dawn = sky_state(&GameEnvironment::fake_day(6.5 * 3600.0, SUNRISE, SUNSET, 1, 4));
    assert!(
        dawn.ambient_scale > 0.38,
        "dawn ambient should carry the scatter bump: {}",
        dawn.ambient_scale
    );
    assert!(
        mid.ambient_scale < dawn.ambient_scale,
        "midnight carries no scatter: {} vs dawn {}",
        mid.ambient_scale,
        dawn.ambient_scale
    );
}

#[test]
fn gate_10_2a_clock_interpolates_and_cross_fades() {
    // Updates every 10 game-minutes (the §2.3 cadence). The clock
    // is a trailing interpolator (§3.3): after each push, evaluate
    // the minutes between the previous and current update. Stepping
    // 1 game-minute never moves any field by > 0.01.
    let mut clock = SkyClock::new(GameEnvironment::fake_day(0.0, SUNRISE, SUNSET, 1, 4));
    let mut prev_state = clock.evaluate(0.0);
    for step in 1..=144 {
        clock.push(GameEnvironment::fake_day(
            step as f32 * 600.0,
            SUNRISE,
            SUNSET,
            1,
            4,
        ));
        for minute in 1..=10 {
            let t = ((step - 1) as f32 * 600.0 + minute as f32 * 60.0) % 86_400.0;
            let cur = clock.evaluate(t);
            let step_size = max_step(&prev_state, &cur);
            assert!(
                step_size <= 0.01,
                "clock step {step_size} at t={t} (update {step}, minute {minute})"
            );
            prev_state = cur;
        }
    }

    // Weather cross-fade: clear -> thunderstorm at noon, evaluated
    // every 5 game-seconds across the 30 game-second fade (§3.3):
    // each step covers at most ~1/6 of the total field delta.
    let mut clock = SkyClock::new(live_noon());
    let mut storm = live_noon();
    storm.weather_id = "thunder".into();
    storm.precip_class = 3;
    storm.rains = true;
    storm.light_modifier = -40.0;
    storm.sun_multiplier = 0.01;
    storm.sight_penalty = 1.2;
    storm.lightning_active = true;
    clock.push(storm.clone());
    let before = clock.evaluate(NOON - 1.0);
    let after = clock.evaluate(NOON + 31.0);
    let total = max_step(&before, &after);
    assert!(total > 0.1, "storm actually changes the state ({total})");
    let mut prev = before;
    for i in 1..=16 {
        let cur = clock.evaluate(NOON - 1.0 + i as f32 * 2.0);
        let step = max_step(&prev, &cur);
        assert!(
            step <= total / 6.0 + 0.01,
            "fade step {step} of total {total} at +{}s",
            i * 2
        );
        prev = cur;
    }
    // After the fade, the state equals the direct storm state.
    let direct = sky_state(&storm);
    let settled = clock.evaluate(NOON + 60.0);
    assert!(max_step(&direct, &settled) < 1e-4, "fade settles on storm");
}

// ------------------------------------------------------------- 10.2b

#[test]
fn gate_10_2b_vanilla_classification() {
    let expected: &[(&str, WeatherKind)] = &[
        ("sunny", WeatherKind::Clear),
        ("cloudy", WeatherKind::Cloudy),
        ("light_drizzle", WeatherKind::Rain),
        ("drizzle", WeatherKind::Rain),
        ("rain", WeatherKind::HeavyRain),
        ("rainstorm", WeatherKind::HeavyRain),
        ("thunder", WeatherKind::Thunderstorm),
        ("lightning", WeatherKind::Thunderstorm),
        ("flurries", WeatherKind::Snow),
        ("snowing", WeatherKind::Snow),
        ("snowstorm", WeatherKind::Snow),
        // mist: sight_penalty 1.3 is below the fog threshold (fog is
        // 1.7); its sun_multiplier 0.3 reads as overcast. Named call.
        ("mist", WeatherKind::Cloudy),
        ("fog", WeatherKind::Fog),
    ];
    for (id, kind) in expected {
        assert_eq!(classify_weather(&vanilla(id)), *kind, "weather {id}");
    }
    // Classification is numeric, not id-based: a nonsense id with
    // sunny numerics is Clear, and "sunny" carrying fog numerics
    // is Fog.
    let mut fake = vanilla("sunny");
    fake.weather_id = "weather_bogus_xyz".into();
    assert_eq!(classify_weather(&fake), WeatherKind::Clear);
    fake.sight_penalty = 1.9;
    assert_eq!(classify_weather(&fake), WeatherKind::Fog);
}

#[test]
fn gate_10_2b_invented_rain_lights_like_vanilla_rain() {
    // An invented/modded id with vanilla drizzle numerics classifies
    // as rain and lights within 5% of vanilla drizzle.
    let mut invented = vanilla("drizzle");
    invented.weather_id = "modded_acid_drizzle_mk2".into();
    assert_eq!(classify_weather(&invented), WeatherKind::Rain);
    let a = sky_state(&invented);
    let b = sky_state(&vanilla("drizzle"));
    for (x, y, name) in [
        (a.sun_intensity, b.sun_intensity, "sun_intensity"),
        (a.ambient_scale, b.ambient_scale, "ambient_scale"),
        (a.exposure, b.exposure, "exposure"),
    ] {
        assert!(
            (x - y).abs() <= 0.05 * y.abs().max(1e-3),
            "{name}: invented {x} vs vanilla {y}"
        );
    }
    assert_eq!(a.precip.kind, PrecipKind::Rain);
    assert!(a.precip.rate > 0.0);
}

#[test]
fn gate_10_2b_extreme_numerics_are_clamped() {
    // Portal-storm-grade nonsense must not produce NaN or negative
    // light (§10 risk): every field finite and in range.
    let mut env = live_noon();
    env.weather_id = "scarlet_dimension_storm".into();
    env.light_multiplier = f32::INFINITY;
    env.light_modifier = -100_000.0;
    env.sun_multiplier = -3.0;
    env.sight_penalty = 4.0;
    env.sun_light = 1.0e9;
    env.moon_light = f32::NAN;
    env.tint_rgb = Some([2.0, -1.0, 0.5]);
    env.tint_strength = 7.0;
    env.precip_class = 250;
    let s = sky_state(&env);
    let check = |v: f32, name: &str| {
        assert!(v.is_finite(), "{name} not finite: {v}");
        assert!(v >= 0.0, "{name} negative: {v}");
    };
    check(s.sun_intensity, "sun_intensity");
    check(s.moon_intensity, "moon_intensity");
    check(s.ambient_scale, "ambient_scale");
    check(s.fog_density, "fog_density");
    check(s.cloud_cover, "cloud_cover");
    check(s.exposure, "exposure");
    for c in 0..3 {
        check(s.sun_color[c], "sun_color");
        check(s.zenith[c], "zenith");
        check(s.horizon[c], "horizon");
    }
    assert!(s.sun_intensity <= 1.0 && s.cloud_cover <= 1.0);
    // precip_class 250 clamps to heavy (3); rains=false -> snow family.
    assert_eq!(classify_weather(&env), WeatherKind::Snow);
}
