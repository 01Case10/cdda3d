//! Stage C (first half) gate: the noise foundation's contract with
//! the material recipes — exact tileability, determinism, and
//! documented output ranges.
//!
//! Tileability verdict, asserted as two separate facts:
//! - **Edges are bit-exact.** `f(0, y) == f(period, y)` and
//!   `f(x, 0) == f(x, period)` hold to the bit (`to_bits` equality),
//!   because sampling wraps the input into `[0, period)` first, so
//!   both sides run the identical computation. This is the property
//!   texture tiling actually relies on.
//! - **Interior period shifts hold to ~1 ulp, not to the bit.**
//!   `f(x + period, y)` starts from a different `f32` — `x + period`
//!   itself rounds — so the wrapped coordinate can differ by an ulp
//!   and the result by a few ulps. Asserted with epsilon 1e-5, far
//!   below one texel step of output value, and stated here so no
//!   recipe author assumes bitwise period-shift invariance.

use voxelizer::{seed_from_str, white_noise, Fbm, Rng, ValueNoise, Worley};

const SEED: u64 = 0xCDDA_0001;

/// Sample points strictly inside one period, reused across tests.
fn interior_points(period: u32) -> Vec<(f32, f32)> {
    let p = period as f32;
    let mut pts = Vec::new();
    let mut x = 0.13;
    while x < p {
        let mut y = 0.29;
        while y < p {
            pts.push((x, y));
            y += 0.77;
        }
        x += 0.61;
    }
    pts
}

#[test]
fn value_noise_edges_bit_exact() {
    for period in [1u32, 4, 7] {
        let n = ValueNoise::new(SEED, period);
        let p = period as f32;
        for &t in &[0.0f32, 0.25, 1.5, 3.7, 6.99] {
            assert_eq!(
                n.sample(0.0, t).to_bits(),
                n.sample(p, t).to_bits(),
                "value noise period {period}: left/right edge mismatch at y={t}"
            );
            assert_eq!(
                n.sample(t, 0.0).to_bits(),
                n.sample(t, p).to_bits(),
                "value noise period {period}: top/bottom edge mismatch at x={t}"
            );
        }
        // Corner: all four corners are the same computation.
        assert_eq!(n.sample(0.0, 0.0).to_bits(), n.sample(p, p).to_bits());
    }
}

#[test]
fn worley_edges_bit_exact() {
    for period in [1u32, 4, 7] {
        let w = Worley::new(SEED, period);
        let p = period as f32;
        for &t in &[0.0f32, 0.25, 1.5, 3.7, 6.99] {
            let a = w.sample(0.0, t);
            let b = w.sample(p, t);
            assert_eq!(a.f1.to_bits(), b.f1.to_bits(), "worley f1 edge, period {period}, y={t}");
            assert_eq!(a.f2.to_bits(), b.f2.to_bits(), "worley f2 edge, period {period}, y={t}");
            let c = w.sample(t, 0.0);
            let d = w.sample(t, p);
            assert_eq!(c.f1.to_bits(), d.f1.to_bits(), "worley f1 edge, period {period}, x={t}");
            assert_eq!(c.f2.to_bits(), d.f2.to_bits(), "worley f2 edge, period {period}, x={t}");
        }
    }
}

#[test]
fn fbm_edges_bit_exact_for_every_octave_count() {
    // Periodicity must survive adding octaves: each octave wraps
    // with its own period * lacunarity^i, so the sum stays periodic
    // with the base period. Check 1..=5 octaves explicitly.
    for octaves in 1..=5u32 {
        let f = Fbm::new(SEED, 4).octaves(octaves);
        for &t in &[0.0f32, 0.25, 1.5, 3.7] {
            assert_eq!(
                f.sample(0.0, t).to_bits(),
                f.sample(4.0, t).to_bits(),
                "fbm octaves {octaves}: left/right edge mismatch at y={t}"
            );
            assert_eq!(
                f.sample(t, 0.0).to_bits(),
                f.sample(t, 4.0).to_bits(),
                "fbm octaves {octaves}: top/bottom edge mismatch at x={t}"
            );
        }
    }
    // Non-default lacunarity/gain keep the same property.
    let f = Fbm::new(SEED, 3).octaves(3).lacunarity(3).gain(0.7);
    assert_eq!(f.sample(0.0, 1.25).to_bits(), f.sample(3.0, 1.25).to_bits());
    assert_eq!(f.sample(1.25, 0.0).to_bits(), f.sample(1.25, 3.0).to_bits());
}

#[test]
fn interior_period_shift_within_epsilon() {
    // See the module header: bitwise equality is NOT promised here,
    // because (x + period) rounds in f32 before wrapping. 1e-5 is
    // ~100x the observed ulp noise and ~1/3000 of the output range.
    const EPS: f32 = 1e-5;
    let period = 4u32;
    let p = period as f32;
    let vn = ValueNoise::new(SEED, period);
    let wo = Worley::new(SEED, period);
    let fb = Fbm::new(SEED, period);
    for (x, y) in interior_points(period) {
        assert!((vn.sample(x + p, y) - vn.sample(x, y)).abs() <= EPS, "value noise x-shift at ({x}, {y})");
        assert!((vn.sample(x, y + p) - vn.sample(x, y)).abs() <= EPS, "value noise y-shift at ({x}, {y})");
        let a = wo.sample(x, y);
        let b = wo.sample(x + p, y);
        assert!((a.f1 - b.f1).abs() <= EPS && (a.f2 - b.f2).abs() <= EPS, "worley x-shift at ({x}, {y})");
        let c = wo.sample(x, y + p);
        assert!((a.f1 - c.f1).abs() <= EPS && (a.f2 - c.f2).abs() <= EPS, "worley y-shift at ({x}, {y})");
        assert!((fb.sample(x + p, y) - fb.sample(x, y)).abs() <= EPS, "fbm x-shift at ({x}, {y})");
        assert!((fb.sample(x, y + p) - fb.sample(x, y)).abs() <= EPS, "fbm y-shift at ({x}, {y})");
    }
}

#[test]
fn sample_uv_wraps_at_tile_edges_bit_exact() {
    // The recipe-facing form: u = 1.0 must be u = 0.0, exactly.
    let vn = ValueNoise::new(SEED, 4);
    let wo = Worley::new(SEED, 4);
    let fb = Fbm::new(SEED, 4);
    for &t in &[0.0f32, 0.31, 0.5, 0.97] {
        assert_eq!(vn.sample_uv(0.0, t).to_bits(), vn.sample_uv(1.0, t).to_bits());
        assert_eq!(vn.sample_uv(t, 0.0).to_bits(), vn.sample_uv(t, 1.0).to_bits());
        assert_eq!(fb.sample_uv(0.0, t).to_bits(), fb.sample_uv(1.0, t).to_bits());
        let a = wo.sample_uv(0.0, t);
        let b = wo.sample_uv(1.0, t);
        assert_eq!(a.f1.to_bits(), b.f1.to_bits());
        assert_eq!(a.f2.to_bits(), b.f2.to_bits());
    }
}

#[test]
fn determinism_same_seed_identical_different_seed_differs() {
    let period = 4u32;
    let grid = |f: &dyn Fn(f32, f32) -> f32| -> Vec<u32> {
        let mut out = Vec::new();
        for i in 0..16 {
            for j in 0..16 {
                let x = i as f32 / 16.0 * period as f32;
                let y = j as f32 / 16.0 * period as f32;
                out.push(f(x, y).to_bits());
            }
        }
        out
    };

    let a = ValueNoise::new(SEED, period);
    let b = ValueNoise::new(SEED, period);
    let c = ValueNoise::new(SEED + 1, period);
    assert_eq!(grid(&|x, y| a.sample(x, y)), grid(&|x, y| b.sample(x, y)), "value noise: same seed must be identical");
    assert_ne!(grid(&|x, y| a.sample(x, y)), grid(&|x, y| c.sample(x, y)), "value noise: different seeds must differ");

    let a = Worley::new(SEED, period);
    let b = Worley::new(SEED, period);
    let c = Worley::new(SEED + 1, period);
    assert_eq!(grid(&|x, y| a.sample(x, y).f1), grid(&|x, y| b.sample(x, y).f1), "worley: same seed identical");
    assert_ne!(grid(&|x, y| a.sample(x, y).f1), grid(&|x, y| c.sample(x, y).f1), "worley: different seeds differ");

    let a = Fbm::new(SEED, period);
    let b = Fbm::new(SEED, period);
    let c = Fbm::new(SEED + 1, period);
    assert_eq!(grid(&|x, y| a.sample(x, y)), grid(&|x, y| b.sample(x, y)), "fbm: same seed identical");
    assert_ne!(grid(&|x, y| a.sample(x, y)), grid(&|x, y| c.sample(x, y)), "fbm: different seeds differ");
}

#[test]
fn seed_plumbing_deterministic() {
    assert_eq!(seed_from_str("t_grass"), seed_from_str("t_grass"));
    assert_ne!(seed_from_str("t_grass"), seed_from_str("t_dirt"));
    assert_ne!(seed_from_str("t_grass"), seed_from_str("t_grass2"));

    let mut r1 = Rng::new(42);
    let mut r2 = Rng::new(42);
    let mut r3 = Rng::new(43);
    let s1: Vec<u64> = (0..64).map(|_| r1.next_u64()).collect();
    let s2: Vec<u64> = (0..64).map(|_| r2.next_u64()).collect();
    let s3: Vec<u64> = (0..64).map(|_| r3.next_u64()).collect();
    assert_eq!(s1, s2, "rng: same seed, same stream");
    assert_ne!(s1, s3, "rng: different seed, different stream");

    let mut r = Rng::new(7);
    for _ in 0..1000 {
        let v = r.next_f32();
        assert!((0.0..1.0).contains(&v), "rng next_f32 out of range: {v}");
        assert!(r.below(6) < 6, "rng below out of range");
    }
}

#[test]
fn output_ranges_hold_on_dense_grid() {
    let period = 4u32;
    let vn = ValueNoise::new(SEED, period);
    let wo = Worley::new(SEED, period);
    let fb = Fbm::new(SEED, period).octaves(5);
    // Two periods across, so wrapped sampling is exercised too.
    for i in 0..64 {
        for j in 0..64 {
            let x = i as f32 / 64.0 * 2.0 * period as f32;
            let y = j as f32 / 64.0 * 2.0 * period as f32;
            let v = vn.sample(x, y);
            assert!((0.0..=1.0).contains(&v), "value noise out of [0,1]: {v} at ({x}, {y})");
            let f = fb.sample(x, y);
            assert!((0.0..=1.0).contains(&f), "fbm out of [0,1]: {f} at ({x}, {y})");
            let s = wo.sample(x, y);
            assert!(s.f1 >= 0.0 && s.f1 < std::f32::consts::SQRT_2, "worley f1 out of [0, sqrt2): {} at ({x}, {y})", s.f1);
            assert!(s.f2 >= s.f1, "worley f2 < f1 at ({x}, {y}): {} < {}", s.f2, s.f1);
            let d = s.f2_minus_f1();
            assert!((0.0..1.5).contains(&d), "worley f2-f1 out of [0, 1.5): {d} at ({x}, {y})");
        }
    }
    // white_noise is a hash value: [0, 1) everywhere.
    for x in 0..32 {
        for y in 0..32 {
            let v = white_noise(SEED, x, y);
            assert!((0.0..1.0).contains(&v), "white noise out of [0,1): {v}");
        }
    }
}
