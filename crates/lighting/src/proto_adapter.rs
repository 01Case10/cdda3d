//! Adapters from the wire protocol (`crates/proto` object types) to
//! the light grid. This is where snapshot memory-layer semantics are
//! enforced on the way in:
//!
//! - `Snapshot.blocks` are live tiles with their exported lit/seen.
//! - `Snapshot.memory_blocks` are remembered tiles: *terrain /
//!   furniture only, `lit = MEMORIZED`* (schema comment on
//!   `Snapshot.memory_blocks`). The adapter forces `MEMORIZED` for
//!   them even if a sender wrote something else, and — because it
//!   only reads lit/seen — live entities/items/fields in a memory
//!   block can never leak through this path (they are not light).
//!
//! Occlusion: a tile occludes when its terrain (or furniture, if it
//! fills the tile) expands to a full-height opaque block under the
//! mesher's fake/real mapping. The default predicate uses the
//! mesher's `block_for_tile` + `tile_shape` (full + opaque); real
//! TileDefs-driven worlds can pass a custom predicate later without
//! changing this module's shape.

use mesher::block::{block_for_tile, tile_shape, Pass, TileShape};
use proto::{DeltaT, EnvironmentUpdateT, SnapshotT, SubmapBlockT};

use crate::grid::LightGrid;
use crate::mapping::TileLight;
use crate::sky::GameEnvironment;

/// Default occlusion predicate for a terrain/furniture id pair.
pub fn default_occludes(terrain: u32, furniture: u32) -> bool {
    let id = if furniture != 0 { furniture } else { terrain };
    if id == 0 {
        return false;
    }
    tile_shape(id) == TileShape::Full && block_for_tile(id).pass == Pass::Opaque
}

/// Build a light grid covering every block (live + memory) of a
/// snapshot. The grid origin is the minimum block origin; each block
/// occupies 12x12 tiles at its origin's z.
pub fn grid_from_snapshot(snap: &SnapshotT) -> LightGrid {
    let empty: Vec<SubmapBlockT> = Vec::new();
    let live = snap.blocks.as_ref().unwrap_or(&empty);
    let mem = snap.memory_blocks.as_ref().unwrap_or(&empty);
    let all: Vec<&SubmapBlockT> = live.iter().chain(mem.iter()).collect();
    if all.is_empty() {
        return LightGrid::new([0, 0, 0], [0, 0, 0]);
    }
    let (mut min_x, mut min_y, mut min_z) = (i32::MAX, i32::MAX, i32::MAX);
    let (mut max_x, mut max_y, mut max_z) = (i32::MIN, i32::MIN, i32::MIN);
    for b in &all {
        let o = b
            .origin
            .as_ref()
            .map(|v| [v.x, v.y, v.z])
            .unwrap_or([0, 0, 0]);
        min_x = min_x.min(o[0]);
        min_y = min_y.min(o[1]);
        min_z = min_z.min(o[2]);
        max_x = max_x.max(o[0] + 12);
        max_y = max_y.max(o[1] + 12);
        max_z = max_z.max(o[2] + 1);
    }
    let mut grid = LightGrid::new(
        [min_x, min_y, min_z],
        [
            (max_x - min_x) as usize,
            (max_y - min_y) as usize,
            (max_z - min_z) as usize,
        ],
    );
    // Live blocks first, memory blocks second so memory never
    // overwrites a live tile at the same position (a position is in
    // exactly one of the two in a well-formed snapshot; ordering is
    // belt-and-braces).
    for b in live {
        apply_block(&mut grid, b, false);
    }
    for b in mem {
        apply_block(&mut grid, b, true);
    }
    grid
}

fn apply_block(grid: &mut LightGrid, b: &SubmapBlockT, is_memory: bool) {
    let o = b
        .origin
        .as_ref()
        .map(|v| [v.x, v.y, v.z])
        .unwrap_or([0, 0, 0]);
    let lit = b.lit.as_ref();
    let seen = b.seen.as_ref();
    let terrain = b.terrain.as_ref();
    let furniture = b.furniture.as_ref();
    let outside = b.outside.as_ref();
    for ly in 0..12i32 {
        for lx in 0..12i32 {
            let i = (ly * 12 + lx) as usize;
            let mut l = lit
                .and_then(|v| v.get(i).copied())
                .unwrap_or(proto::LitLevel::BLANK);
            let s = seen.and_then(|v| v.get(i).copied()).unwrap_or(0);
            if is_memory {
                l = proto::LitLevel::MEMORIZED;
            }
            let ter = terrain.and_then(|v| v.get(i).copied()).unwrap_or(0);
            let furn = furniture.and_then(|v| v.get(i).copied()).unwrap_or(0);
            // Memory tiles never occlude for smoothing across the
            // memory boundary (smoothing already refuses to mix
            // memorized with visible); live tiles use the predicate.
            let occ = if is_memory {
                false
            } else {
                default_occludes(ter, furn)
            };
            // Sky exposure (Phase 10 §3.4): the schema's own
            // convention is that absent data reads as not exposed,
            // so a pre-10.1 block simply carries `false` here.
            let out = outside.and_then(|v| v.get(i).copied()).unwrap_or(false);
            grid.set(
                o[0] + lx,
                o[1] + ly,
                o[2],
                TileLight::new(l, s).occluding(occ).outside(out),
            );
        }
    }
}

/// Build a grid from a single submap block (fixtures / stub house).
pub fn grid_from_block(b: &SubmapBlockT) -> LightGrid {
    let mut snap = SnapshotT::default();
    snap.blocks = Some(vec![b.clone()]);
    grid_from_snapshot(&snap)
}

// ---------------------------------------------------------------------
// Environment conversion (Phase 10 slice 10.6): the proto-adapter
// form of EnvironmentUpdate that sky.rs's GameEnvironment doc has
// pointed at since 10.2. Field-for-field, deliberately WITHOUT the
// §10 range clamping — sanitizing is the SkyState derivation's job
// (classify_weather / sky_state sanitize their input), and an
// adapter that quietly rewrote values would break the gate-10.1a
// semantic-equality discipline this conversion is checked under.
// ---------------------------------------------------------------------

/// Convert one wire `EnvironmentUpdate` to its engine-side mirror.
pub fn game_environment_from_update(u: &EnvironmentUpdateT) -> GameEnvironment {
    GameEnvironment {
        seq: u.seq,
        epoch: u.epoch,
        turn: u.turn,
        time_of_day_sec: u.time_of_day_sec,
        sunrise_sec: u.sunrise_sec,
        sunset_sec: u.sunset_sec,
        season: u.season,
        sun_light: u.sun_light,
        moon_light: u.moon_light,
        sun_azimuth_deg: u.sun_azimuth_deg,
        sun_altitude_deg: u.sun_altitude_deg,
        moon_phase: u.moon_phase,
        weather_id: u.weather_id.clone().unwrap_or_default(),
        light_multiplier: u.light_multiplier,
        light_modifier: u.light_modifier,
        sun_multiplier: u.sun_multiplier,
        precip_class: u.precip_class,
        rains: u.rains,
        sight_penalty: u.sight_penalty,
        // The wire tint is a flat [float] vector: exactly 3 entries
        // is a colour; anything else (absent, malformed) is none.
        tint_rgb: match u.tint_rgb.as_deref() {
            Some([r, g, b]) => Some([*r, *g, *b]),
            _ => None,
        },
        tint_strength: u.tint_strength,
        temperature_c: u.temperature_c,
        windspeed_mph: u.windspeed_mph,
        wind_dir_deg: u.wind_dir_deg,
        lightning_active: u.lightning_active,
    }
}

/// The environment piggybacked on a `Snapshot` (§2.2: a fresh
/// client is never environment-blind), when present.
pub fn game_environment_from_snapshot(snap: &SnapshotT) -> Option<GameEnvironment> {
    snap.environment
        .as_deref()
        .map(game_environment_from_update)
}

/// The environment piggybacked on a `Delta` (§2.3: present when
/// it changed at that send boundary), when present.
pub fn game_environment_from_delta(delta: &DeltaT) -> Option<GameEnvironment> {
    delta
        .environment
        .as_deref()
        .map(game_environment_from_update)
}
