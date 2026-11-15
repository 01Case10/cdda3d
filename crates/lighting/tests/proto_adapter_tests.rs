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
fn memory_blocks_forced_memorized() {
    // A memory block whose sender (incorrectly) wrote LIT must still
    // enter the grid as MEMORIZED — memory semantics are enforced
    // adapter-side, per the schema's Snapshot.memory_blocks comment.
    let mut mem = sample_house_block();
    mem.origin = Some(Vec3iT { x: 24, y: 0, z: 0 });
    // (lit already LIT in the sample; the adapter must override.)
    let mut snap = sample_snapshot();
    snap.memory_blocks = Some(vec![mem]);
    let g = grid_from_snapshot(&snap);
    let t = g.tile_light(30, 6, 0);
    assert_eq!(t.lit, LitLevel::MEMORIZED);
    assert_eq!(t.visibility(), Visibility::Memorized);
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
