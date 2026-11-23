//! Gates 7.6 + 7.7 — day/night and weather *state*.
//!
//! Both are pure functions / table lookups over game-supplied state.
//! Neither does per-tile work: the engine evaluates them once per
//! frame (or once per bake) and applies the result as a global
//! multiplier / fog uniform. That is what "no per-state cost cliff"
//! (gate 7.7) means here — switching weather changes constants, not
//! algorithms.
//!
//! Scope note (plan §5 Phase 7 scope split): the *sun's* illumination
//! and shadows belong to the ray tracer; what this module owns is the
//! game-time / game-weather *state mapping* — the curve the tracer
//! and the raster baker both read. The game's `lit_level` already
//! encodes how dark a tile is; the day/night factor below is a global
//! presentation tint/exposure on top of it (it preserves the
//! interior-vs-exterior ratio the game data carries, it does not
//! re-derive it).

use crate::mapping::LightSample;

// Phase 10 (§3): the derived sky state lives in `sky`; this module
// stays the bake-facing layer and re-exports it so existing
// `atmosphere::` paths keep working.
pub use crate::sky::{sky_state, GameEnvironment, SkyClock, SkyState};

/// Global day/night presentation: a brightness (exposure) factor and
/// an RGB tint for the sun/ambient term.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayNight {
    pub brightness: f32,
    pub tint: [f32; 3],
}

impl DayNight {
    /// Multiply a tile sample by the global factor.
    pub fn apply(self, s: LightSample) -> LightSample {
        LightSample::new(
            (s.intensity * self.brightness).clamp(0.0, 1.0),
            [
                s.tint[0] * self.tint[0],
                s.tint[1] * self.tint[1],
                s.tint[2] * self.tint[2],
            ],
        )
    }
}

// Keyframes: (hour, brightness, tint). Sorted, covering [0, 24]; the
// last entry wraps to the first at hour 0 of the next day.
const KEYFRAMES: &[(f32, f32, [f32; 3])] = &[
    (0.0, 0.22, [0.55, 0.65, 0.95]),  // midnight: dim, blue
    (4.5, 0.22, [0.55, 0.65, 0.95]),  // deep night, held
    (6.0, 0.35, [0.80, 0.65, 0.75]),  // pre-dawn
    (7.5, 0.75, [1.00, 0.80, 0.60]),  // dawn: warm gold
    (9.0, 0.95, [1.00, 0.95, 0.90]),  // morning
    (12.0, 1.00, [1.00, 1.00, 1.00]), // noon: neutral full
    (15.0, 0.98, [1.00, 0.97, 0.92]), // afternoon
    (17.5, 0.80, [1.00, 0.82, 0.62]), // late afternoon, warming
    (19.0, 0.38, [0.85, 0.60, 0.70]), // dusk
    (20.5, 0.22, [0.55, 0.65, 0.95]), // nightfall complete
    (24.0, 0.22, [0.55, 0.65, 0.95]), // wraps to midnight
];

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Day/night state at a game time-of-day. `hour` is in `[0, 24)`
/// (fractional; values outside are wrapped). Interpolation between
/// keyframes uses smoothstep, so the curve is C1-continuous in
/// practice: brightness and tint change with zero slope at each
/// keyframe and never jump (gate 7.6's "transitions smoothly").
pub fn day_night_at(hour: f32) -> DayNight {
    let h = hour.rem_euclid(24.0);
    for w in KEYFRAMES.windows(2) {
        let (h0, b0, t0) = w[0];
        let (h1, b1, t1) = w[1];
        if h >= h0 && h <= h1 {
            let f = if h1 > h0 {
                smoothstep((h - h0) / (h1 - h0))
            } else {
                0.0
            };
            return DayNight {
                brightness: b0 + (b1 - b0) * f,
                tint: [
                    t0[0] + (t1[0] - t0[0]) * f,
                    t0[1] + (t1[1] - t0[1]) * f,
                    t0[2] + (t1[2] - t0[2]) * f,
                ],
            };
        }
    }
    // Unreachable given the table covers [0,24], but never panic.
    DayNight {
        brightness: 1.0,
        tint: [1.0, 1.0, 1.0],
    }
}

// ---------------------------------------------------------------------------
// Weather
// ---------------------------------------------------------------------------

/// Weather state as the game will export it (mirrors CDDA's weather
/// types at the granularity the renderer needs; the fork exporter
/// maps its weather enum onto these variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherKind {
    Clear,
    Cloudy,
    Rain,
    HeavyRain,
    Snow,
    Fog,
    Thunderstorm,
}

/// Everything the engine consumes for one weather state: fog
/// (exponential density + color) and precipitation parameters for
/// the Phase 2b-style shader emitters, plus a global light scale.
/// `fog_density == 0.0` means fog is *off* — the renderer skips the
/// fog term entirely rather than evaluating it at zero strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherParams {
    pub fog_density: f32,
    pub fog_color: [f32; 3],
    /// Particles per second per tile column the emitter should aim
    /// for (0 = precipitation off).
    pub precip_rate: f32,
    /// Fall speed multiplier for precipitation (snow drifts slower).
    pub precip_fall_speed: f32,
    /// Global brightness scale (storms darken the sun term).
    pub brightness_scale: f32,
    pub tint: [f32; 3],
}

impl WeatherParams {
    pub fn fog_enabled(&self) -> bool {
        self.fog_density > 0.0
    }
    pub fn precip_enabled(&self) -> bool {
        self.precip_rate > 0.0
    }
    /// Transmittance over `dist` tiles under exponential fog:
    /// `exp(-density * dist)`. Only called when fog is enabled.
    pub fn fog_transmittance(&self, dist: f32) -> f32 {
        if !self.fog_enabled() {
            return 1.0;
        }
        (-self.fog_density * dist.max(0.0)).exp()
    }
    /// Blend a color toward fog color over distance.
    pub fn apply_fog(&self, color: [f32; 3], dist: f32) -> [f32; 3] {
        let t = self.fog_transmittance(dist);
        [
            self.fog_color[0] + (color[0] - self.fog_color[0]) * t,
            self.fog_color[1] + (color[1] - self.fog_color[1]) * t,
            self.fog_color[2] + (color[2] - self.fog_color[2]) * t,
        ]
    }
}

/// The weather table: a constant lookup, O(1) per state change.
pub fn weather_params(kind: WeatherKind) -> WeatherParams {
    match kind {
        WeatherKind::Clear => WeatherParams {
            fog_density: 0.0,
            fog_color: [0.70, 0.80, 0.90],
            precip_rate: 0.0,
            precip_fall_speed: 1.0,
            brightness_scale: 1.0,
            tint: [1.0, 1.0, 1.0],
        },
        WeatherKind::Cloudy => WeatherParams {
            fog_density: 0.0,
            fog_color: [0.65, 0.70, 0.78],
            precip_rate: 0.0,
            precip_fall_speed: 1.0,
            brightness_scale: 0.85,
            tint: [0.95, 0.97, 1.0],
        },
        WeatherKind::Rain => WeatherParams {
            fog_density: 0.008,
            fog_color: [0.55, 0.62, 0.70],
            precip_rate: 40.0,
            precip_fall_speed: 1.0,
            brightness_scale: 0.80,
            tint: [0.92, 0.96, 1.0],
        },
        WeatherKind::HeavyRain => WeatherParams {
            fog_density: 0.020,
            fog_color: [0.48, 0.55, 0.64],
            precip_rate: 120.0,
            precip_fall_speed: 1.3,
            brightness_scale: 0.65,
            tint: [0.88, 0.93, 1.0],
        },
        WeatherKind::Snow => WeatherParams {
            fog_density: 0.012,
            fog_color: [0.85, 0.88, 0.92],
            precip_rate: 30.0,
            precip_fall_speed: 0.35,
            brightness_scale: 0.90,
            tint: [0.97, 0.99, 1.0],
        },
        WeatherKind::Fog => WeatherParams {
            fog_density: 0.045,
            fog_color: [0.75, 0.78, 0.82],
            precip_rate: 0.0,
            precip_fall_speed: 1.0,
            brightness_scale: 0.75,
            tint: [0.95, 0.97, 1.0],
        },
        WeatherKind::Thunderstorm => WeatherParams {
            fog_density: 0.015,
            fog_color: [0.42, 0.48, 0.58],
            precip_rate: 100.0,
            precip_fall_speed: 1.25,
            brightness_scale: 0.55,
            tint: [0.85, 0.90, 1.0],
        },
    }
}

/// Combined per-frame environment: day/night × weather. Evaluated
/// once, applied everywhere — the bakers take it as a value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    pub day_night: DayNight,
    pub weather: WeatherParams,
    pub weather_kind: WeatherKind,
}

impl Environment {
    pub fn new(hour: f32, kind: WeatherKind) -> Self {
        Self {
            day_night: day_night_at(hour),
            weather: weather_params(kind),
            weather_kind: kind,
        }
    }
    /// Clear noon — the neutral environment (identity on samples).
    pub fn neutral() -> Self {
        Self::new(12.0, WeatherKind::Clear)
    }
    /// Apply the environment to a tile sample *for baking*.
    ///
    /// Tint applies in full. Brightness applies *tempered*:
    /// `factor = 0.4 + 0.6 * (day_brightness * weather_scale)`.
    /// Rationale: a tile's `lit_level` is already the game's
    /// time-of-day-aware truth — a lamp-lit interior at midnight is
    /// `BRIGHT` *in the game's own terms*. Multiplying it by the
    /// raw midnight curve (0.22) would double-darken local light
    /// and crush every interior to black, while the curve's real
    /// consumer for full-strength application is the ray tracer's
    /// sun/ambient term (via [`DayNight::apply`], which stays pure).
    /// The 0.4 floor keeps the day/night *ordering* (midnight is
    /// still the dimmest presentation, noon is exactly 1.0 in clear
    /// weather) without overriding game truth.
    pub fn apply(self, s: LightSample) -> LightSample {
        let combined = self.day_night.brightness * self.weather.brightness_scale;
        let factor = 0.4 + 0.6 * combined;
        LightSample::new(
            (s.intensity * factor).clamp(0.0, 1.0),
            [
                s.tint[0] * self.day_night.tint[0] * self.weather.tint[0],
                s.tint[1] * self.day_night.tint[1] * self.weather.tint[1],
                s.tint[2] * self.day_night.tint[2] * self.weather.tint[2],
            ],
        )
    }
}
