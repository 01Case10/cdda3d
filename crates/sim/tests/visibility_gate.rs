//! S3 gates, stage 1 (snapshot level): the computed visibility
//! model — LOS flood, apparent light, outside, frac/tint, and the
//! environment numerics — checked against expectations derived
//! BY HAND from the scene layouts and the upstream formulas
//! (0.I-1 lightmap.cpp/calendar.cpp, master smooth_lighting).
//!
//! These gates were written against the S2 placeholder behaviour
//! (lit/seen by z-level rule, sample_environment() verbatim,
//! enclosure-flood outside, no frac/tint at all) and FAILED there;
//! the S3 implementation makes them pass. The turn-level half of
//! T1/T6 lives in environment_gate.rs; the mirror invariant in
//! turn_gate.rs.
//!
//! Hand-derivation notes (street_block, noon, clear, turn 43200,
//! day 0 = spring equinox at lat 42.36):
//! - sun altitude at noon = 90 - 42.36 = 47.6 deg, so
//!   sun_light_at = (65/61)*47.6 + 65/61 + 60 = ~111.8;
//!   moon phase 0 (new) -> moon_light 1.0; natural light ~112.8.
//! - Exterior (sky-exposed) tiles: lm = natural -> BRIGHT.
//! - Interior tiles: the sun cast south is stopped by the south
//!   wall/door, so lm = inside_light_level = LIGHT_AMBIENT_DIM*0.8
//!   = 4.0 (build_sunlight_cache: outside light > 10). With the
//!   vision threshold at the player's tile (ambient 4.0):
//!   vt = min(3.5, (3.7/exp((8/3)*0.03833) - 0.01) * (1 + 0.3/6.3))
//!      = 3.489, so interior = LOW (4.0 >= vt, < 10) and
//!   frac = (4.0 - 3.489)/(10 - 3.489) = 0.0785 -> 20.
//!   NOTE: upstream apparent_light_at can never return LIT here —
//!   LIGHT_SOURCE_BRIGHT == LIGHT_AMBIENT_LIT == 10.0, so the
//!   BRIGHT branch subsumes LIT; interiors read LOW, not LIT.
//! - The closed door tile (5,8): not sky-exposed (INDOORS stamp),
//!   but its own sun cast south exits over the yard, attenuated
//!   exp(-0.0383 * ~15) = 0.56 -> lm ~63 -> BRIGHT (sunlit face).
//! - outside follows map::build_outside_cache: a tile is NOT
//!   outside when any INDOORS-flagged tile is within Chebyshev
//!   distance 1 on its z-level — so the house's wall/door/window
//!   ring reads not-outside (the Phase 1 stub's ring=true was a
//!   stub simplification), the yard reads outside.

use std::path::PathBuf;

use proto::{LitLevel, SnapshotT};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn build(scene_name: &str, mutate: impl FnOnce(&mut Scene)) -> (DefRegistry, World, SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let mut scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
    mutate(&mut scene);
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    let snap = world.snapshot(&reg);
    (reg, world, snap)
}

/// (lit, seen, outside, frac, tint) of one tile from a snapshot.
fn tile(snap: &SnapshotT, pos: [i32; 3]) -> (LitLevel, u8, bool, Option<u8>, Option<[u8; 3]>) {
    for b in snap.blocks.as_ref().expect("blocks") {
        let o = b.origin.as_ref().expect("origin");
        if o.z == pos[2] && (o.x..o.x + 12).contains(&pos[0]) && (o.y..o.y + 12).contains(&pos[1]) {
            let i = ((pos[1] - o.y) * 12 + (pos[0] - o.x)) as usize;
            let lit = b.lit.as_ref().expect("lit array")[i];
            let seen = b.seen.as_ref().expect("seen array")[i];
            let outside = b.outside.as_ref().expect("S3: blocks always carry outside")[i];
            let frac = b
                .light_frac
                .as_ref()
                .expect("S3: blocks carry light_frac")
                .get(i)
                .copied();
            let tint = b.light_tint.as_ref().and_then(|t| {
                let j = i * 3;
                Some([*t.get(j)?, *t.get(j + 1)?, *t.get(j + 2)?])
            });
            return (lit, seen, outside, frac, tint);
        }
    }
    panic!("pos {pos:?} not in snapshot");
}

#[test]
fn street_block_noon_grids() {
    let (_reg, _world, snap) = build("street_block", |_| {});

    // --- outside grid (build_outside_cache stamp + is_roofed) ---
    for pos in [[5, 5, 0], [4, 4, 0], [3, 3, 0]] {
        assert!(!tile(&snap, pos).2, "interior floor {pos:?} is not outside");
    }
    for pos in [[2, 2, 0], [5, 8, 0], [4, 2, 0], [9, 5, 0]] {
        assert!(
            !tile(&snap, pos).2,
            "wall/door/window ring {pos:?} is stamped not-outside by the adjacent INDOORS floor"
        );
    }
    for pos in [[5, 9, 0], [0, 0, 0], [20, 5, 0], [13, 4, 0], [23, 23, 0]] {
        assert!(tile(&snap, pos).2, "yard/road {pos:?} is outside");
    }

    // --- lit grid at noon ---
    // Exterior: full natural light.
    for pos in [[5, 9, 0], [10, 5, 0], [0, 0, 0], [20, 5, 0], [23, 23, 0]] {
        let (lit, seen, _, frac, _) = tile(&snap, pos);
        assert_eq!(lit, LitLevel::BRIGHT, "exterior {pos:?} at noon");
        assert_eq!(seen, 255, "exterior {pos:?} is in the LOS flood");
        assert_eq!(frac, Some(255), "BRIGHT tiles carry frac 255");
    }
    // Interior: inside_light_level 4.0 -> LOW. The lamp-free
    // frac would be 20 (derivation above); the streetlamp adds
    // ~3 through the window (its flood enters by the only
    // opening, exactly as apply_light_source would), lifting
    // the interior frac into the ~130-160 range — still LOW.
    for pos in [[5, 5, 0], [3, 3, 0], [8, 7, 0], [5, 7, 0]] {
        let (lit, seen, _, frac, _) = tile(&snap, pos);
        assert_eq!(lit, LitLevel::LOW, "interior {pos:?} at noon");
        assert_eq!(seen, 255);
        let f = frac.expect("frac");
        assert!(
            (100..=180).contains(&f),
            "interior frac {f} = inside floor + lamp-through-window"
        );
    }
    // The closed door's sunlit outer face.
    let (lit, seen, ..) = tile(&snap, [5, 8, 0]);
    assert_eq!(lit, LitLevel::BRIGHT, "door tile catches the sun");
    assert_eq!(seen, 255);
    // Tree: opaque endpoint of the flood, sky-exposed.
    let (lit, seen, ..) = tile(&snap, [13, 4, 0]);
    assert_eq!(lit, LitLevel::BRIGHT);
    assert_eq!(seen, 255);

    // --- tint at noon: sunlight is uncoloured; far from the
    // streetlamp the accumulated colour cache is ~0 -> white. ---
    for pos in [[0, 0, 0], [23, 23, 0], [5, 5, 0]] {
        assert_eq!(
            tile(&snap, pos).4,
            Some([255, 255, 255]),
            "noon tint at {pos:?} is white"
        );
    }
}

#[test]
fn street_block_midnight_lamp_pool() {
    // Midnight, day 0: moon phase 0 (new) -> moon_light 1.0,
    // natural light 1.0. Vision threshold at the player (indoors,
    // ambient = inside_light_level 3.5):
    //   vt = min(3.5, 3.330 * (1 + (3.5-3.7)/6.3)) = 3.225.
    // Streetlamp (12,9), colour [1.0, 0.6, 0.2], intensity 6:
    // contribution at distance d = 6 * exp(-0.0383 * d).
    let (_reg, _world, snap) = build("street_block", |s| {
        s.time = "midnight".to_string();
    });

    // The yard away from the lamp: at the game's own OPEN_AIR
    // coefficient a luminance-6 source still contributes ~3 at
    // the far corners (6 * exp(-0.0383 * 18) = 2.9), so the
    // whole yard is dimly LOW — but far below the pool's frac.
    for pos in [[5, 20, 0], [0, 0, 0], [22, 3, 0]] {
        let (lit, seen, _, frac, _) = tile(&snap, pos);
        assert_eq!(lit, LitLevel::LOW, "lamp-lit yard {pos:?} at midnight");
        assert_eq!(seen, 255);
        let f = frac.expect("frac");
        // Closest of the three is (22,3) at ~11.7 tiles:
        // contribution 6*exp(-0.0383*11.7) = 3.8 -> frac ~61.
        // The pool's dimmest asserted tile is ~89.
        assert!(f <= 64, "far yard frac {f} is well below the pool's");
    }
    // Interior floor: the 3.5 inside floor still clears the
    // dark-adapted threshold -> LOW.
    let (lit, seen, ..) = tile(&snap, [5, 5, 0]);
    assert_eq!(lit, LitLevel::LOW, "interior at midnight");
    assert_eq!(seen, 255);

    // The lamp tile itself: sm = 6 > 0 -> BRIGHT, frac 255, and
    // the tint is the lamp colour run through compute_tile_tint
    // (hand-derived: cc = [6, 3.6, 1.2], sat_mag 4.8, alpha 54,
    // fade 0.330 -> hue = [1.0, 0.888, 0.777] -> [255, 226, 198]).
    let (lit, seen, _, frac, tint) = tile(&snap, [12, 9, 0]);
    assert_eq!(lit, LitLevel::BRIGHT, "lamp tile self-lights");
    assert_eq!(seen, 255);
    assert_eq!(frac, Some(255));
    let tint = tint.expect("tint present");
    assert_eq!(tint[0], 255, "brightest channel normalised to 255");
    assert!(
        (tint[1] as i32 - 226).abs() <= 2 && (tint[2] as i32 - 198).abs() <= 2,
        "lamp tint is its normalised colour, got {tint:?}"
    );

    // Pool gradient east along y=9: lit LOW, frac strictly
    // decreasing with distance from the lamp.
    let mut prev = 256i32;
    for x in [13, 15, 17, 19] {
        let (lit, seen, _, frac, tint) = tile(&snap, [x, 9, 0]);
        assert_eq!(lit, LitLevel::LOW, "pool tile x={x}");
        assert_eq!(seen, 255);
        let f = frac.expect("frac") as i32;
        assert!(
            f < prev,
            "frac decreases with distance (x={x}: {f} !< {prev})"
        );
        assert!(f > 0, "pool tiles are above the vision threshold");
        prev = f;
        let t = tint.expect("tint");
        assert_eq!(t[0], 255);
        assert!(t[1] < 250 && t[2] < t[1], "pool tint is warm: {t:?}");
    }
    // Far corners: colour cache max <= 3.5 -> the tint fade is 0
    // -> exactly white even though a trace of lamp light arrives.
    for pos in [[0, 23, 0], [23, 0, 0]] {
        assert_eq!(
            tile(&snap, pos).4,
            Some([255, 255, 255]),
            "far corner {pos:?} tint stays white"
        );
    }
}

#[test]
fn stub_house_noon_spectator() {
    // No player spawn: the fixture snapshot is fully observed
    // (no LOS culling), classified by light alone.
    let (_reg, _world, snap) = build("stub_house", |_| {});
    let (lit, seen, outside, frac, _) = tile(&snap, [6, 6, 0]);
    assert_eq!(lit, LitLevel::LOW, "interior floor: inside_light_level 4.0");
    assert_eq!(seen, 255);
    assert!(!outside, "interior floor is not outside");
    assert_eq!(frac, Some(20));
    let (lit, seen, outside, ..) = tile(&snap, [0, 0, 0]);
    // The stub house is a sealed 12x12 fixture — it has NO
    // exterior tiles at all, so the corner wall's sun cast is
    // blocked by the wall below it and no lit face spills onto
    // it: it reads the inside floor, LOW. (In the real bubble
    // the surroundings would light its outer face.)
    assert_eq!(lit, LitLevel::LOW, "sealed-fixture corner wall");
    assert_eq!(seen, 255);
    assert!(
        !outside,
        "the wall corner is stamped not-outside by the adjacent INDOORS floor \
         (the stub's ring=true was its own simplification)"
    );
}

#[test]
fn street_block_midnight_no_lamp_is_dark() {
    // Same scene with the streetlamp removed: nothing breaks
    // the darkness, so the yard's lm = natural = 1.0 (new moon)
    // falls below the dark-adapted threshold (3.22) -> BLANK,
    // while the interior's 3.5 inside floor still reads LOW.
    let (_reg, _world, snap) = build("street_block", |s| {
        s.time = "midnight".to_string();
        s.lights.clear();
    });
    for pos in [[5, 20, 0], [0, 0, 0], [22, 3, 0], [12, 9, 0]] {
        let (lit, seen, _, frac, _) = tile(&snap, pos);
        assert_eq!(lit, LitLevel::BLANK, "unlit yard {pos:?} at midnight");
        assert_eq!(seen, 0);
        assert_eq!(frac, Some(0));
    }
    let (lit, seen, ..) = tile(&snap, [5, 5, 0]);
    assert_eq!(lit, LitLevel::LOW, "interior floor at midnight");
    assert_eq!(seen, 255);
}

#[test]
fn roofed_house_levels() {
    let (_reg, _world, snap) = build("roofed_house_explicit", |_| {});
    // Roof over the interior: visible from below (the tile under
    // it is flood-reached), sky-exposed, full sun.
    let (lit, seen, outside, ..) = tile(&snap, [6, 6, 1]);
    assert_eq!(lit, LitLevel::BRIGHT, "roof tile at noon");
    assert_eq!(seen, 255);
    assert!(outside, "roof tile is sky-exposed");
    // Basement: no opening in the floor above -> never visible.
    // (The fixture's DARK comes from map::light_at, which the
    // exporter does not use; apparent_light_at yields BLANK.)
    let (lit, seen, outside, ..) = tile(&snap, [6, 6, -1]);
    assert_eq!(lit, LitLevel::BLANK, "basement is unseen");
    assert_eq!(seen, 0);
    assert!(!outside, "z<0 is never outside");
    // Interior floor.
    let (lit, seen, ..) = tile(&snap, [6, 6, 0]);
    assert_eq!(lit, LitLevel::LOW);
    assert_eq!(seen, 255);
}

#[test]
fn environment_numerics_noon() {
    let (_reg, _world, snap) = build("street_block", |_| {});
    let env = snap.environment.as_ref().expect("environment piggybacked");
    assert_eq!(env.turn, 43200, "noon of day 0 = 43200 turns (1 s/turn)");
    assert_eq!(env.time_of_day_sec, 43200.0);
    assert_eq!(env.season, 0, "spring");
    assert_eq!(env.weather_id.as_deref(), Some("weather_clear"));
    assert_eq!(env.precip_class, 0);
    assert_eq!(env.sight_penalty, 0.0);
    assert!(!env.lightning_active);
    // Equinox noon at lat 42.36: altitude ~47.6 deg.
    assert!(
        env.sun_altitude_deg > 40.0 && env.sun_altitude_deg < 55.0,
        "noon altitude {}, expected ~47.6",
        env.sun_altitude_deg
    );
    // sun_light_at(47.6) = ~111.8, clear weather folds 1.0/0.
    assert!(
        env.sun_light > 100.0 && env.sun_light < 120.0,
        "incident sunlight {}, expected ~111.8",
        env.sun_light
    );
    // Day 0 noon: moon phase 0 (new) -> moon_light = 1 + 1.5*0.
    assert_eq!(env.moon_phase, 0);
    assert_eq!(env.moon_light, 1.0);
    // Sunrise/sunset solved from the -1 deg crossing: ~05:58/18:02.
    assert!(
        env.sunrise_sec > 20_000.0 && env.sunrise_sec < 22_000.0,
        "sunrise {}",
        env.sunrise_sec
    );
    assert!(
        env.sunset_sec > 64_000.0 && env.sunset_sec < 66_000.0,
        "sunset {}",
        env.sunset_sec
    );
}
