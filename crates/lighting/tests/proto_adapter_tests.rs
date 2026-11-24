//! Proto-adapter tests: snapshot/memory semantics and the wire
//! `LightUpdate` / `Delta` paths into the grid + dirty sets.

use lighting::grid::LightSampler;
use lighting::mapping::{TileLight, Visibility};
use lighting::proto_adapter::{default_occludes, grid_from_block, grid_from_snapshot};
use lighting::updates::{apply_delta, apply_light_update};
use proto::samples::{sample_house_block, sample_snapshot};
use proto::{DeltaT, LightUpdateT, LitLevel, SubmapBlockT, TileChangeT, Vec3iT};

#[test]
fn snapshot_house_grid() {
    let snap = sample_snapshot();
    let g = grid_from_snapshot(&snap);
    assert_eq!(g.size, [12, 12, 1]);
    // Sample house is uniformly LIT/255.
    let t = g.tile_light(6, 6, 0);
    assert_eq!(t.lit, LitLevel::LIT);
    assert_eq!(t.seen, 255);
    assert_eq!(t.visibility(), Visibility::Visible);
    // Perimeter walls occlude (terrain id 2 == mesher TILE_WALL,
    // full + opaque); the floor centre does not.
    assert!(g.tile_light(0, 0, 0).occluding);
    assert!(!g.tile_light(6, 6, 0).occluding);
    assert!(default_occludes(2, 0));
    assert!(!default_occludes(1, 0));
}

#[test]
fn memory_blocks_do_not_force_memorized() {
    // AMENDED 2026-11-24 (Phase 11 follow-up): this test used to
    // assert that a memory block whose sender wrote LIT enters the
    // grid as MEMORIZED ("the adapter forces MEMORIZED"). That
    // pinned the bug S3 found: apply_block stamped MEMORIZED on
    // EVERY tile of a memory block, including the BLANK tiles the
    // exporter writes for never-memorized positions in a partial
    // block (voxel_exporter.cpp build_memory_blocks: "Tiles in a
    // memory block that were never memorized stay BLANK"), so
    // never-seen tiles read Memorized instead of Hidden.
    // Corrected semantics: within a memory block only a wire
    // MEMORIZED is memorized; any other value fails closed to
    // BLANK/Hidden — a memory block can never make a tile Visible
    // or Memorized. The partial-block shape itself is gated in
    // tests/memory_block_partial_gate.rs.
    let mut mem = sample_house_block();
    mem.origin = Some(Vec3iT { x: 24, y: 0, z: 0 });
    // (lit is LIT in the sample; in a memory block that is a
    // non-memorized entry and must read BLANK, not MEMORIZED.)
    let mut snap = sample_snapshot();
    snap.memory_blocks = Some(vec![mem]);
    let g = grid_from_snapshot(&snap);
    let t = g.tile_light(30, 6, 0);
    assert_eq!(t.lit, LitLevel::BLANK);
    assert_eq!(t.visibility(), Visibility::Hidden);
    // Live tiles are untouched.
    assert_eq!(g.tile_light(6, 6, 0).lit, LitLevel::LIT);
}

#[test]
fn single_block_adapter() {
    let b: SubmapBlockT = sample_house_block();
    let g = grid_from_block(&b);
    assert_eq!(g.count_visibility(Visibility::Visible), 144);
    let _ = TileLight::BLANK;
}

#[test]
fn wire_light_update_applies_and_dirties() {
    let mut g = grid_from_block(&sample_house_block());
    // Re-send the same values: no dirt.
    let mut same = LightUpdateT::default();
    same.seq = 2;
    same.epoch = 1;
    same.origin = Some(Vec3iT { x: 0, y: 0, z: 0 });
    same.lit = Some(vec![LitLevel::LIT; 144]);
    same.seen = Some(vec![255; 144]);
    assert!(apply_light_update(&mut g, &same).is_empty());
    // Darken the whole block: interior tiles dirty only chunk [0,0],
    // border tiles also dirty the neighbours they touch.
    let mut dark = same.clone();
    dark.lit = Some(vec![LitLevel::DARK; 144]);
    let dirty = apply_light_update(&mut g, &dark);
    assert!(dirty.contains(&[0, 0]));
    assert!(dirty.contains(&[-1, 0]) && dirty.contains(&[0, -1]));
    assert_eq!(g.tile_light(6, 6, 0).lit, LitLevel::DARK);
    // A malformed (short) update applies nothing.
    let mut short = same.clone();
    short.lit = Some(vec![LitLevel::BRIGHT; 3]);
    short.seen = Some(vec![255; 3]);
    assert!(apply_light_update(&mut g, &short).is_empty());
    assert_eq!(g.tile_light(0, 0, 0).lit, LitLevel::DARK);
}

#[test]
fn wire_delta_applies_lit_seen() {
    let mut g = grid_from_block(&sample_house_block());
    let mut tc = TileChangeT::default();
    tc.pos = Some(Vec3iT { x: 6, y: 6, z: 0 });
    tc.terrain = 1;
    tc.lit = LitLevel::BRIGHT;
    tc.seen = 255;
    tc.displayed_field = Some(proto::displayed_field(0, 0));
    let mut delta = DeltaT::default();
    delta.seq = 2;
    delta.epoch = 1;
    delta.tile_changes = Some(vec![tc]);
    let dirty = apply_delta(&mut g, &delta);
    assert_eq!(dirty, std::collections::BTreeSet::from([[0, 0]]));
    assert_eq!(g.tile_light(6, 6, 0).lit, LitLevel::BRIGHT);
}

// ---------------------------------------------------------------------
// Phase 10 slice 10.6 — the wire `outside[]` feed reaches the grid,
// and EnvironmentUpdate converts to GameEnvironment field-for-field.
// (Wave 1/2 left both at the adapter boundary: TileLight had no
// outside field, so grid_from_snapshot dropped the array the schema
// has carried since 10.1, and GameEnvironment had no proto source.)
// ---------------------------------------------------------------------

use lighting::proto_adapter::{
    game_environment_from_delta, game_environment_from_snapshot, game_environment_from_update,
};
use proto::samples::{sample_delta, sample_environment};

#[test]
fn snapshot_outside_reaches_grid() {
    let g = grid_from_snapshot(&sample_snapshot());
    // The stub-house rule (proto::samples, mirrored in house.h):
    // tiles strictly inside the wall perimeter are not sky-exposed;
    // walls, corners and the door/window ring are.
    assert!(!g.tile_light(6, 6, 0).outside, "interior floor is sheltered");
    assert!(!g.tile_light(1, 1, 0).outside, "interior corner is sheltered");
    assert!(g.tile_light(0, 0, 0).outside, "corner wall is exposed");
    assert!(g.tile_light(0, 6, 0).outside, "perimeter wall is exposed");
    assert!(g.tile_light(11, 11, 0).outside, "perimeter is exposed");
}

#[test]
fn snapshot_without_outside_reads_not_exposed() {
    // Schema convention (SubmapBlock.outside): old data reads as
    // absent = not exposed.
    let mut snap = sample_snapshot();
    for b in snap.blocks.as_mut().unwrap() {
        b.outside = None;
    }
    let g = grid_from_snapshot(&snap);
    assert!(!g.tile_light(0, 0, 0).outside);
    assert!(!g.tile_light(6, 6, 0).outside);
}

#[test]
fn wire_delta_applies_outside_and_dirties() {
    let mut g = grid_from_block(&sample_house_block());
    assert!(!g.tile_light(6, 6, 0).outside);
    // Flip only the outside flag (lit/seen unchanged): the tile's
    // baked GI changes, so its chunk must dirty.
    let mut tc = TileChangeT::default();
    tc.pos = Some(Vec3iT { x: 6, y: 6, z: 0 });
    tc.terrain = 1;
    tc.lit = LitLevel::LIT;
    tc.seen = 255;
    tc.outside = true;
    let mut delta = DeltaT::default();
    delta.seq = 2;
    delta.epoch = 1;
    delta.tile_changes = Some(vec![tc.clone()]);
    let dirty = apply_delta(&mut g, &delta);
    assert_eq!(dirty, std::collections::BTreeSet::from([[0, 0]]));
    assert!(g.tile_light(6, 6, 0).outside);
    // Re-sending the same change dirties nothing.
    assert!(apply_delta(&mut g, &delta).is_empty());
}

#[test]
fn environment_conversion_field_for_field() {
    let wire = sample_environment();
    let env = game_environment_from_update(&wire);
    assert_eq!(env.seq, 1);
    assert_eq!(env.epoch, 1);
    assert_eq!(env.turn, 309_600);
    assert_eq!(env.time_of_day_sec, 43_200.0);
    assert_eq!(env.sunrise_sec, 21_600.0);
    assert_eq!(env.sunset_sec, 64_800.0);
    assert_eq!(env.season, 0);
    assert_eq!(env.sun_light, 125.0);
    assert_eq!(env.moon_light, 1.0);
    assert_eq!(env.sun_azimuth_deg, 180.0);
    assert_eq!(env.sun_altitude_deg, 55.0);
    assert_eq!(env.moon_phase, 4);
    assert_eq!(env.weather_id, "weather_clear");
    assert_eq!(env.light_multiplier, 1.0);
    assert_eq!(env.light_modifier, 0.0);
    assert_eq!(env.sun_multiplier, 1.0);
    assert_eq!(env.precip_class, 0);
    assert!(!env.rains);
    // Field-for-field means field-for-field: the sample's 0.0
    // sight_penalty crosses as 0.0 (range clamping is the SkyState
    // derivation's job, not the adapter's).
    assert_eq!(env.sight_penalty, 0.0);
    assert_eq!(env.tint_rgb, None);
    assert_eq!(env.tint_strength, 0.0);
    assert_eq!(env.temperature_c, 20.0);
    assert_eq!(env.windspeed_mph, 5.0);
    assert_eq!(env.wind_dir_deg, 90.0);
    assert!(!env.lightning_active);
}

#[test]
fn environment_conversion_tint_and_piggybacks() {
    // A 3-entry tint crosses as Some; a malformed one as None.
    let mut wire = sample_environment();
    wire.tint_rgb = Some(vec![0.5, 0.25, 1.0]);
    assert_eq!(
        game_environment_from_update(&wire).tint_rgb,
        Some([0.5, 0.25, 1.0])
    );
    wire.tint_rgb = Some(vec![0.5, 0.25]);
    assert_eq!(game_environment_from_update(&wire).tint_rgb, None);
    // Missing weather id reads as empty, never panics.
    wire.weather_id = None;
    assert_eq!(game_environment_from_update(&wire).weather_id, "");

    // Snapshot piggyback: present in the canonical sample.
    let from_snap = game_environment_from_snapshot(&sample_snapshot())
        .expect("sample snapshot piggybacks the environment");
    assert_eq!(from_snap.turn, 309_600);
    let mut bare = sample_snapshot();
    bare.environment = None;
    assert!(game_environment_from_snapshot(&bare).is_none());

    // Delta piggyback: absent in the canonical sample delta,
    // carried when present.
    assert!(game_environment_from_delta(&sample_delta()).is_none());
    let mut d = sample_delta();
    d.environment = Some(Box::new(sample_environment()));
    assert_eq!(game_environment_from_delta(&d).unwrap().turn, 309_600);
}
