//! Phase 10 Wave 2 FREEZE gates — the shared sky contract
//! (`SkyUniforms` + `SKY_WGSL` + `sky_color_cpu`) that slices 10.3
//! (raster) and 10.4 (RT) build on in parallel. See
//! `crates/lighting/src/sky_uniforms.rs` for the contract itself and
//! docs/phase10-sky-gi.md §4/§5.1/§6.

use lighting::sky::{sky_state, GameEnvironment};
use lighting::sky_uniforms::{
    sky_color_cpu, SkyUniforms, CAVE_GROUND, CAVE_HORIZON, CAVE_ZENITH, SKY_WGSL,
};

fn approx(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= tol)
}

/// The legacy RT `sky()` from treebuild, reimplemented from its
/// hardcoded constants (grid space: `.z` is up).
fn legacy_sky(rd: [f32; 3]) -> [f32; 3] {
    let horizon = [0.08, 0.10, 0.14];
    let zenith = [0.30, 0.48, 0.78];
    let ground = [0.05, 0.06, 0.08];
    let mix = |a: [f32; 3], b: [f32; 3], f: f32| {
        [
            a[0] + (b[0] - a[0]) * f,
            a[1] + (b[1] - a[1]) * f,
            a[2] + (b[2] - a[2]) * f,
        ]
    };
    if rd[2] > 0.0 {
        mix(horizon, zenith, rd[2].clamp(0.0, 1.0).powf(0.6))
    } else {
        mix(horizon, ground, (-rd[2] * 3.0).clamp(0.0, 1.0))
    }
}

fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

// ---------------------------------------------------------------------------
// a) Layout freeze
// ---------------------------------------------------------------------------

#[test]
fn gate_freeze_layout_size_and_offsets() {
    assert_eq!(std::mem::size_of::<SkyUniforms>(), 128);
    assert_eq!(std::mem::offset_of!(SkyUniforms, sun_dir), 0);
    assert_eq!(std::mem::offset_of!(SkyUniforms, sun_color), 16);
    assert_eq!(std::mem::offset_of!(SkyUniforms, moon_dir), 32);
    assert_eq!(std::mem::offset_of!(SkyUniforms, moon_color), 48);
    assert_eq!(std::mem::offset_of!(SkyUniforms, zenith), 64);
    assert_eq!(std::mem::offset_of!(SkyUniforms, horizon), 80);
    assert_eq!(std::mem::offset_of!(SkyUniforms, ground_col), 96);
    assert_eq!(std::mem::offset_of!(SkyUniforms, fog_color), 112);
}

#[test]
fn gate_freeze_wgsl_struct_matches_rust_layout() {
    let start = SKY_WGSL
        .find("struct SkyUniforms")
        .expect("SKY_WGSL must declare struct SkyUniforms");
    let body = &SKY_WGSL[start..];
    let end = body.find('}').expect("struct must close");
    let names: Vec<&str> = body[..end]
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            l.split(':').next().filter(|n| {
                !n.is_empty()
                    && n.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                    && l.contains("vec4<f32>")
            })
        })
        .collect();
    assert_eq!(
        names,
        vec![
            "sun_dir",
            "sun_color",
            "moon_dir",
            "moon_color",
            "zenith",
            "horizon",
            "ground_col",
            "fog_color"
        ],
        "WGSL SkyUniforms field order drifted from the frozen Rust layout"
    );
    assert!(
        SKY_WGSL.contains("fn sky_color("),
        "SKY_WGSL must define sky_color"
    );
}

// ---------------------------------------------------------------------------
// b) Neutral identity (§6: pre-Phase-10 goldens byte-identical)
// ---------------------------------------------------------------------------

#[test]
fn gate_freeze_neutral_reproduces_legacy_sky() {
    let u = SkyUniforms::neutral();
    // The legacy gradient stops, exactly.
    assert!(approx(u.zenith3(), [0.30, 0.48, 0.78], 0.0));
    assert!(approx(u.horizon3(), [0.08, 0.10, 0.14], 0.0));
    assert!(approx(u.ground3(), [0.05, 0.06, 0.08], 0.0));
    // Legacy RT sun direction in grid space: (0.3, -0.4, 0.8) normalized.
    let legacy = norm([0.3, -0.4, 0.8]);
    assert!(approx(u.sun_dir3(), legacy, 1e-7));
    // Every additive/fog term gated off.
    assert_eq!(u.sun_disc_scale(), 0.0);
    assert_eq!(u.moon_disc_scale(), 0.0);
    assert_eq!(u.star_scale(), 0.0);
    assert_eq!(u.cave_factor(), 0.0);
    assert_eq!(u.fog_density(), 0.0);

    let dirs = [
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        norm([0.3, -0.4, 0.8]),
        norm([-0.7, 0.2, 0.35]),
        norm([0.1, 0.9, -0.4]),
        norm([0.5, 0.5, -0.05]),
        norm([-0.2, -0.3, 0.001]),
    ];
    for rd in dirs {
        let got = sky_color_cpu(rd[2], rd, &u);
        let want = legacy_sky(rd);
        assert!(
            approx(got, want, 1e-6),
            "neutral sky mismatch at {rd:?}: got {got:?}, legacy {want:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// c) from_sky_state derivations
// ---------------------------------------------------------------------------

#[test]
fn gate_freeze_from_state_neutral_env_is_daytime() {
    let state = sky_state(&GameEnvironment::neutral());
    let u = SkyUniforms::from_sky_state(&state, 0.0);
    // Daytime stops: blue-dominant zenith, horizon brighter than zenith.
    assert!(
        u.zenith3()[2] > u.zenith3()[0],
        "zenith not blue: {:?}",
        u.zenith3()
    );
    assert!(u.horizon3()[0] > u.zenith3()[0]);
    for c in [u.zenith3(), u.horizon3(), u.ground3()].concat() {
        assert!((0.0..=1.0).contains(&c), "stop component out of range: {c}");
    }
    // Sun colour is pre-multiplied by intensity.
    let premult = [
        state.sun_color[0] * state.sun_intensity,
        state.sun_color[1] * state.sun_intensity,
        state.sun_color[2] * state.sun_intensity,
    ];
    assert!(approx(u.sun_color3(), premult, 1e-7));
    // Directions pass through in SkyState's world space, unconverted.
    assert!(approx(u.sun_dir3(), state.sun_dir, 0.0));
    assert!(approx(u.moon_dir3(), state.moon_dir, 0.0));
    assert_eq!(u.cave_factor(), 0.0);
}

#[test]
fn gate_freeze_from_state_midnight_moon_and_stars() {
    // Midnight, summer, full moon (phase 4) on the fake-data path.
    let env = GameEnvironment::fake_day(0.0, 21_600.0, 64_800.0, 1, 4);
    let state = sky_state(&env);
    assert!(
        state.sun_intensity < 1e-6,
        "sun up at midnight: {}",
        state.sun_intensity
    );
    let u = SkyUniforms::from_sky_state(&state, 0.0);
    assert!(approx(u.sun_color3(), [0.0, 0.0, 0.0], 1e-6));
    let moon_mag = u.moon_color3().iter().map(|c| c * c).sum::<f32>().sqrt();
    assert!(
        moon_mag > 0.01,
        "moon should dominate at midnight: {moon_mag}"
    );
    assert!(
        u.star_scale() > 0.0,
        "stars must be visible on a clear midnight"
    );
    assert!(u.moon_disc_scale() > 0.0);
}

#[test]
fn gate_freeze_cloud_cover_collapses_disc() {
    let clear = SkyUniforms::from_sky_state(&sky_state(&GameEnvironment::neutral()), 0.0);
    let mut storm_env = GameEnvironment::neutral();
    storm_env.precip_class = 3;
    storm_env.rains = true;
    storm_env.lightning_active = true;
    storm_env.light_modifier = -45.0;
    let storm_state = sky_state(&storm_env);
    assert_eq!(storm_state.cloud_cover, 1.0);
    let storm = SkyUniforms::from_sky_state(&storm_state, 0.0);
    assert!(
        storm.sun_disc_scale() < clear.sun_disc_scale() * 0.2,
        "overcast disc scale {} not collapsed vs clear {}",
        storm.sun_disc_scale(),
        clear.sun_disc_scale()
    );
    // And the rendered sky toward the sun is far dimmer under the storm.
    let rd = clear.sun_dir3();
    let up = rd[2];
    let clear_col = sky_color_cpu(up, rd, &clear);
    let storm_col = sky_color_cpu(up, rd, &storm);
    assert!(
        storm_col[0] < clear_col[0] - 0.3,
        "storm sun {storm_col:?} not collapsed vs clear {clear_col:?}"
    );
}

// ---------------------------------------------------------------------------
// c2) Direction space conversions (10.3/10.4 fix: from_sky_state emits
// SkyState space (east, north, up); each renderer converts)
// ---------------------------------------------------------------------------

#[test]
fn gate_space_conversions_pin_noon_values() {
    // GameEnvironment::neutral(): azimuth 0 (sun in the south),
    // altitude 65°. SkyState space sun dir = (0, -cos65, +sin65).
    let state = sky_state(&GameEnvironment::neutral());
    let cos = 65f32.to_radians().cos();
    let sin = 65f32.to_radians().sin();
    assert!(
        approx(state.sun_dir, [0.0, -cos, sin], 1e-6),
        "SkyState noon sun dir: {:?}",
        state.sun_dir
    );
    let u = SkyUniforms::from_sky_state(&state, 0.0);
    // Grid space (east, south, up): negate the north component.
    let g = u.to_grid_space();
    assert!(
        approx(g.sun_dir3(), [0.0, cos, sin], 1e-6),
        "grid-space noon sun dir: {:?}",
        g.sun_dir3()
    );
    // Moon is the antipode proxy: (-0, -cos, -sin) in grid space.
    assert!(
        approx(g.moon_dir3(), [0.0, -cos, -sin], 1e-6),
        "grid-space noon moon dir: {:?}",
        g.moon_dir3()
    );
    // Mesher world space (east, up, north): swap north/up.
    let m = u.to_mesher_world_space();
    assert!(
        approx(m.sun_dir3(), [0.0, sin, -cos], 1e-6),
        "mesher-space noon sun dir: {:?}",
        m.sun_dir3()
    );
    assert!(
        approx(m.moon_dir3(), [0.0, -sin, cos], 1e-6),
        "mesher-space noon moon dir: {:?}",
        m.moon_dir3()
    );
    // Conversions touch only .xyz of the two dir rows: every .w
    // scalar gate and every other row is untouched.
    for conv in [g, m] {
        assert_eq!(conv.sun_dir[3], u.sun_dir[3]);
        assert_eq!(conv.moon_dir[3], u.moon_dir[3]);
        assert_eq!(conv.sun_color, u.sun_color);
        assert_eq!(conv.moon_color, u.moon_color);
        assert_eq!(conv.zenith, u.zenith);
        assert_eq!(conv.horizon, u.horizon);
        assert_eq!(conv.ground_col, u.ground_col);
        assert_eq!(conv.fog_color, u.fog_color);
    }
}

// ---------------------------------------------------------------------------
// d) Cave blend (decision 3C)
// ---------------------------------------------------------------------------

#[test]
fn gate_freeze_cave_factor_blends_to_cave_stops() {
    let base = SkyUniforms::neutral();
    let mut caved = base;
    caved.set_cave_factor(1.0);
    // Zenith ray: pure zenith stop (fog/disc/star terms are zero or
    // horizon-weighted at rd_up = 1).
    let got = sky_color_cpu(1.0, [0.0, 0.0, 1.0], &caved);
    assert!(approx(got, CAVE_ZENITH, 1e-6), "cave zenith: {got:?}");
    let got = sky_color_cpu(0.0, [1.0, 0.0, 0.0], &caved);
    assert!(approx(got, CAVE_HORIZON, 1e-6), "cave horizon: {got:?}");
    let got = sky_color_cpu(-1.0, [0.0, 0.0, -1.0], &caved);
    assert!(approx(got, CAVE_GROUND, 1e-6), "cave ground: {got:?}");
    // Factor 0 is identity.
    let mut zero = base;
    zero.set_cave_factor(0.0);
    for rd in [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]] {
        assert!(approx(
            sky_color_cpu(rd[2], rd, &zero),
            sky_color_cpu(rd[2], rd, &base),
            0.0
        ));
    }
}

// ---------------------------------------------------------------------------
// e) Finiteness across the day (§10 mod-weather clamp spirit)
// ---------------------------------------------------------------------------

#[test]
fn gate_freeze_no_nan_across_day() {
    let rds = [
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        norm([0.3, -0.4, 0.8]),
        norm([-0.6, 0.75, -0.28]),
    ];
    for hour in 0..48 {
        let t = hour as f32 * 1800.0;
        let env = GameEnvironment::fake_day(t, 21_600.0, 64_800.0, 1, 4);
        let u = SkyUniforms::from_sky_state(&sky_state(&env), t);
        for f in u.as_rows().concat() {
            assert!(f.is_finite(), "non-finite uniform at t={t}: {f}");
        }
        for rd in rds {
            let c = sky_color_cpu(rd[2], rd, &u);
            assert!(
                c.iter().all(|x| x.is_finite()),
                "non-finite sky colour at t={t} rd={rd:?}: {c:?}"
            );
        }
    }
}
