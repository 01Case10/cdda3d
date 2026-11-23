//! Phase 10.5-early gates: the precipitation emitter (fx crate).
//!
//! - spawn rate: spawns/sec over a fixed window matches
//!   `PrecipState::rate` (particles/sec per tile column, the
//!   `lighting::WeatherParams::precip_rate` semantics) times the
//!   number of sky-exposed columns, within the per-slot boundary
//!   tolerance of the deterministic slot schedule;
//! - sky-exposure gating: a fully roofed volume spawns nothing, a
//!   partially roofed volume spawns only in exposed columns;
//! - fall speed: rain vs snow velocity ratio equals the ratio of
//!   the states' `precip_fall_speed` multipliers;
//! - determinism: same seed + time + state = byte-identical
//!   instance set (the capture harness pins time; the emitter is a
//!   pure function of its inputs).

use fx::precip::{PrecipEmitter, PrecipKind, PrecipState};
use fx::render::FieldInstGpu;

const TARGET: [i32; 3] = [10, 20, 0];

fn all_exposed(_x: i32, _y: i32, _z: i32) -> bool {
    true
}

fn rain() -> PrecipState {
    // lighting::WeatherParams for WeatherKind::Rain.
    PrecipState::new(PrecipKind::Rain, 40.0, 1.0)
}

fn snow() -> PrecipState {
    // lighting::WeatherParams for WeatherKind::Snow.
    PrecipState::new(PrecipKind::Snow, 30.0, 0.35)
}

fn bytes(instances: &[FieldInstGpu]) -> &[u8] {
    bytemuck::cast_slice(instances)
}

// ------------------------------------------------------------- rate
#[test]
fn gate_10_5_precip_spawn_rate_matches_params() {
    let emitter = PrecipEmitter::default();
    let state = rain();
    let cols = emitter.columns(TARGET, &all_exposed).len() as f64;
    assert!(cols > 100.0, "cylinder should cover many columns: {cols}");
    let (t0, t1) = (600.0f32, 610.0f32);
    let spawned = emitter.spawn_count(&state, TARGET, t0, t1, &all_exposed) as f64;
    let expected = state.rate as f64 * cols * (t1 - t0) as f64;
    // Each slot contributes at most one boundary spawn of error.
    let slots = emitter.slots_per_column(&state) as f64;
    let tol = cols * slots;
    assert!(
        (spawned - expected).abs() <= tol,
        "spawned {spawned} vs expected {expected} (tol {tol})"
    );
    assert!(
        (spawned - expected).abs() / expected < 0.03,
        "spawn rate within 3%: spawned {spawned}, expected {expected}"
    );
    // Steady-state active count tracks rate x lifetime per column.
    let active = emitter.particles_at(&state, TARGET, 605.0, &all_exposed).len() as f64;
    let lifetime = emitter.height_tiles / PrecipEmitter::fall_speed(&state);
    let expected_active = state.rate as f64 * cols * lifetime as f64;
    assert!(
        (active - expected_active).abs() / expected_active < 0.15,
        "active {active} vs expected {expected_active}"
    );
}

// ---------------------------------------------------- sky exposure
#[test]
fn gate_10_5_precip_no_spawn_under_roof() {
    let emitter = PrecipEmitter::default();
    let state = rain();
    let roofed = |_x: i32, _y: i32, _z: i32| false;
    for t in [0.0f32, 3.7, 61.25, 600.5] {
        assert!(
            emitter.instances_at(&state, TARGET, t, &roofed).is_empty(),
            "roofed volume must spawn nothing at t={t}"
        );
    }
    assert_eq!(emitter.spawn_count(&state, TARGET, 0.0, 60.0, &roofed), 0);
    // Partial roof: only columns west of the target are exposed.
    let west_only = |x: i32, _y: i32, _z: i32| x < TARGET[0];
    let parts = emitter.particles_at(&state, TARGET, 605.0, &west_only);
    assert!(!parts.is_empty(), "exposed half must still rain");
    assert!(
        parts.iter().all(|p| p.column[0] < TARGET[0]),
        "no particle may spawn in a roofed column"
    );
}

// ------------------------------------------------------- fall speed
#[test]
fn gate_10_5_precip_fall_speed_ratio_matches_params() {
    let emitter = PrecipEmitter::default();
    let (r, s) = (rain(), snow());
    let vr = PrecipEmitter::fall_speed(&r);
    let vs = PrecipEmitter::fall_speed(&s);
    assert!(vr > vs, "rain falls faster than snow: {vr} vs {vs}");
    let ratio = vr / vs;
    let param_ratio = r.fall_speed / s.fall_speed;
    assert!(
        (ratio - param_ratio).abs() / param_ratio < 1e-4,
        "velocity ratio {ratio} must equal params ratio {param_ratio}"
    );
    // And the emitted particles actually carry that velocity.
    for p in emitter.particles_at(&r, TARGET, 42.0, &all_exposed) {
        assert!((p.velocity.y + vr).abs() < 1e-5, "rain vy {}", p.velocity.y);
    }
    for p in emitter.particles_at(&s, TARGET, 42.0, &all_exposed) {
        assert!((p.velocity.y + vs).abs() < 1e-5, "snow vy {}", p.velocity.y);
    }
}

// ------------------------------------------------------- determinism
#[test]
fn gate_10_5_precip_deterministic_byte_equal() {
    let emitter = PrecipEmitter::default();
    let state = snow();
    let a = emitter.instances_at(&state, TARGET, 123.5, &all_exposed);
    let b = emitter.instances_at(&state, TARGET, 123.5, &all_exposed);
    assert!(!a.is_empty());
    assert_eq!(bytes(&a), bytes(&b), "same inputs = byte-identical instances");
    // A different seed or a different time must change the set.
    let other_seed = PrecipEmitter { seed: 7, ..emitter };
    let c = other_seed.instances_at(&state, TARGET, 123.5, &all_exposed);
    assert_ne!(bytes(&a), bytes(&c), "seed must matter");
    let d = emitter.instances_at(&state, TARGET, 124.5, &all_exposed);
    assert_ne!(bytes(&a), bytes(&d), "time must matter");
}

// ----------------------------------------------------- streak shape
#[test]
fn gate_10_5_precip_rain_uses_streak_class() {
    let emitter = PrecipEmitter::default();
    let inst = emitter.instances_at(&rain(), TARGET, 10.0, &all_exposed);
    assert!(!inst.is_empty());
    assert!(
        inst.iter().all(|i| i.class_id == fx::precip::RAIN_CLASS_ID),
        "rain instances must use the streak class"
    );
    let inst = emitter.instances_at(&snow(), TARGET, 10.0, &all_exposed);
    assert!(
        inst.iter().all(|i| i.class_id == fx::precip::SNOW_CLASS_ID),
        "snow instances must use the mote class"
    );
    // The field shader must give the streak class an elongated,
    // thin scale (the instance format carries a scalar size; the
    // per-class scale in the vertex shader is what makes it a
    // streak rather than a cube) — and keep the no-texture rule.
    let shader = fx::render::FIELD_SHADER;
    assert!(shader.contains("class_id == 6u"), "rain class branch missing");
    assert!(shader.contains("class_id == 7u"), "snow class branch missing");
    assert!(!shader.contains("texture"), "field shader binds no texture");
}
