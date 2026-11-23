//! Phase 10 slice 10.6 gates — live seating.
//!
//! What this file pins:
//! - The wire `outside[]` feed (the game's `is_outside &&
//!   !is_roofed`, §3.4 option (a)) is the sky-exposure source
//!   for BOTH renderers' live paths: the raster factor via
//!   `frame::SkyExposure::from_grid`, and the RT per-cell
//!   `sky_exposed` via the grid the `LightingScene` already
//!   carries. Wave 2's RT interim feed — an engine-side column
//!   scan over the geometry (§3.4 option (b)) — disagrees with
//!   the game on perimeter wall tiles (scan: sheltered by the
//!   roof above; game: exposed), and 10.6 retires it.
//! - The proto `EnvironmentUpdate` conversion seats a
//!   `SkyScene` (the joint the live loop consumes).
//! - The cave factor seating rule for the live loop (§4).
//! - The gate-10.6 visual capture set (§8): both scenes x
//!   {midnight, dawn, noon, dusk} x {Clear, Rain, Fog,
//!   Thunderstorm} x both renderers, written out as contact
//!   sheets for Joey's sign-off (a green run here is NOT the
//!   sign-off — it proves the set exists and is sane).

use engine::frame::{LightingScene, SkyExposure, SkyScene};
use lighting::WeatherKind;

// ---------------------------------------------------------------------
// Raster exposure from the wire grid
// ---------------------------------------------------------------------

#[test]
fn raster_exposure_from_wire_grid() {
    let snap = proto::samples::sample_snapshot();
    let grid = lighting::proto_adapter::grid_from_snapshot(&snap);
    let exposure = SkyExposure::from_grid(&grid);
    // Stub-house rule: interior sheltered, perimeter exposed.
    assert_eq!(exposure.factor([6, 6, 0]), 0.0, "interior floor");
    assert_eq!(exposure.factor([1, 1, 0]), 0.0, "interior corner");
    assert_eq!(exposure.factor([0, 0, 0]), 1.0, "corner wall");
    assert_eq!(exposure.factor([0, 6, 0]), 1.0, "perimeter wall");
    // Tiles the grid does not cover keep the no-data shape:
    // factor 1 (the GI applies globally, as with `sky_exposure:
    // None` — absence never shelters).
    assert_eq!(exposure.factor([40, 40, 0]), 1.0, "outside the grid");
}

// ---------------------------------------------------------------------
// RT exposure follows the wire grid, not the geometry scan
// ---------------------------------------------------------------------

/// Any cell of `world` whose containing game tile is `tile`,
/// as (found, sky_exposed). Tile math mirrors rt.rs's own
/// (block coords, `blocks_per_z_level` blocks per game z).
fn cell_exposure(
    world: &treebuild::world::VoxelWorld,
    tile: [i32; 3],
    min_game_z: i32,
) -> Option<bool> {
    let per = mesher::scale::WorldScale::default().blocks_per_z_level.max(1) as i32;
    world
        .iter()
        .find(|(pos, _)| [pos[0], pos[1], min_game_z + pos[2] / per] == tile)
        .map(|(_, cell)| cell.sky_exposed)
}

#[test]
fn rt_exposure_follows_wire_grid() {
    let lit = LightingScene::fake_house(12.0, WeatherKind::Clear);
    // Both fake-house levels (floor + roof), so the roof
    // level's cells exist to carry the verdict.
    let world = engine::rt::world_for_levels_lit(&[0, 1], &lit);
    // Perimeter wall tile: the game's test says exposed (the
    // wire rule the fixture grid now carries). The retired
    // column scan said sheltered — the roof slab sits above
    // the wall column — so this assertion fails on the scan.
    assert_eq!(
        cell_exposure(&world, [0, 6, 0], lit.min_game_z),
        Some(true),
        "perimeter wall cell takes the wire's exposed verdict"
    );
    // Interior floor tile: both sources agree — sheltered.
    assert_eq!(
        cell_exposure(&world, [6, 6, 0], lit.min_game_z),
        Some(false),
        "interior floor cell is sheltered"
    );
    // Roof tile (z 1 over the interior): exposed.
    assert_eq!(
        cell_exposure(&world, [6, 6, 1], lit.min_game_z),
        Some(true),
        "roof cell is exposed"
    );
}

#[test]
fn rt_exposure_honours_wire_over_geometry() {
    // The converse flip: the game marks the roof tile
    // sheltered (a higher roof the capture's world does not
    // model, a bridge overhead — the game is the truth, the
    // geometry scan cannot know). The wire must win.
    let mut lit = LightingScene::fake_house(12.0, WeatherKind::Clear);
    let mut t = lit.grid.get(6, 6, 1);
    t.outside = false;
    lit.grid.set(6, 6, 1, t);
    let world = engine::rt::world_for_levels_lit(&[0, 1], &lit);
    assert_eq!(
        cell_exposure(&world, [6, 6, 1], lit.min_game_z),
        Some(false),
        "roof cell follows the wire's sheltered verdict"
    );
}

// ---------------------------------------------------------------------
// SkyScene seating from the wire
// ---------------------------------------------------------------------

#[test]
fn sky_scene_seats_from_wire_environment() {
    let wire = proto::samples::sample_environment();
    let scene = SkyScene::from_wire(&wire);
    assert_eq!(scene.game_env.turn, 309_600);
    assert_eq!(scene.game_env.weather_id, "weather_clear");
    assert_eq!(scene.game_env.time_of_day_sec, 43_200.0);
    // The seated scene evaluates: noon in spring, sun up.
    let state = scene.sky_state();
    assert!(state.sun_intensity > 0.0, "noon sun is up: {state:?}");
    // Snapshot piggyback seats the same scene.
    let snap = proto::samples::sample_snapshot();
    let scene2 = SkyScene::from_snapshot(&snap).expect("piggybacked environment");
    assert_eq!(scene2.game_env.turn, 309_600);
}
