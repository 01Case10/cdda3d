//! Player billboard gate: the player character renders the
//! same way enemies do — a sprite on a billboard, with the
//! tileset's own art.
//!
//! The bug chain, in the order it was found:
//! 1. The wire -> billboard conversion existed only as a copy
//!    inside `sim_creature_gate`, so the live loop never seated
//!    ANY snapshot entity into a `BillboardScene` — the only
//!    billboard crowds were the hard-coded fixture ones.
//! 2. Creatures are named by tile-entry ID on the wire (a
//!    monster type id; the avatar's `player_male`), but the
//!    billboard atlas is keyed by sprite NAME, and an entry's
//!    fg is often a different name entirely (`player_male` ->
//!    `skin_brown_m`) or a weighted variant list (`mon_zombie`
//!    -> `mon_zombie_1..5`). With no entry table, those ids
//!    composited to a 1x1 transparent hole. The voxelizer now
//!    emits `tile_sprites.json` (entry id -> fg primary sprite)
//!    and the atlas resolves ids through it.
//!
//! Proofs:
//! - `wire::billboard_entities` converts the snapshot's
//!   entities field-for-field, player first, enemies after —
//!   one conversion, no special-casing, no filtering.
//! - Against the real UltiCa atlas, `player_male` resolves to
//!   `skin_brown_m` and `mon_zombie` to `mon_zombie_2` (the
//!   primary variant: highest weight, first on ties), and both
//!   compose to their real 32x48 sprites — no invented art.
//! - A delta-fed snapshot tracks the player to its new tile.
//! - The feet stand on the floor: UltiCa's humanoids carry
//!   transparent rows below their feet, and the composite
//!   records that margin so the instance geometry sinks the
//!   quad by it (player -5/32, zombie -7/32 tiles).
//! - Pixel level (GPU, loud-skip with no adapter or no atlas):
//!   a textured wire capture contains both sprites' own
//!   dominant colours, sampled from the atlas itself.

use std::path::PathBuf;

use billboard::atlas::{fixture_atlas, SpriteAtlas};
use billboard::instance::tile_feet;
use billboard::render::Camera;
use engine::capture::{run_capture, CaptureArgs, CaptureError, RendererKind};
use engine::wire::{apply_delta_to_snapshot, billboard_entities, palette_resolver, WireScene};
use proto::{DeltaT, EntityT};
use sim::scenes::Spawn;
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../sim/scenes/{name}.toml"))
}

fn ultica_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1")
}

fn real_atlas() -> Option<SpriteAtlas> {
    let dir = ultica_dir();
    if !dir.join("sprites.json").exists() {
        eprintln!("SKIP: real UltiCa atlas absent (assets/ultica_run1)");
        return None;
    }
    Some(SpriteAtlas::load(&dir).expect("real atlas loads"))
}

fn zombie_spawn(pos: [i32; 3]) -> Spawn {
    Spawn {
        type_id: "mon_zombie".to_string(),
        pos,
        hp_frac: 255,
        facing: "NONE".to_string(),
        overlays: Vec::new(),
        flags: 0,
        sprite: None,
    }
}

/// street_block (player inside the house at (5,5,0)) plus a
/// zombie inside with it at (7,5,0).
fn build() -> (DefRegistry, proto::SnapshotT) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let mut scene = Scene::load_file(&scene_path("street_block")).expect("scene loads");
    scene.spawns.push(zombie_spawn([7, 5, 0]));
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    let snap = world.snapshot(&reg);
    (reg, snap)
}

#[test]
fn player_and_enemy_convert_through_the_one_wire_conversion() {
    let (reg, snap) = build();
    let entities = billboard_entities(&snap);
    let player = entities
        .iter()
        .find(|e| e.sprite_id == "player_male")
        .expect("the player entity converts to a billboard entity");
    assert_eq!(player.pos, [5, 5, 0], "player at its wire position");
    let zombie = entities
        .iter()
        .find(|e| e.sprite_id == "mon_zombie")
        .expect("the zombie converts through the same conversion");
    assert_eq!(zombie.pos, [7, 5, 0]);
    // Wire order is preserved: the adapter emits the avatar first.
    assert_eq!(entities[0].sprite_id, "player_male");

    // The wire scene carries the same entities, and seats a
    // billboard scene over an atlas.
    let defs = reg.tiledefs().clone();
    let wire = WireScene::from_wire(&defs, &snap, &palette_resolver(&defs));
    assert_eq!(wire.entities.len(), entities.len());
    let scene = wire
        .billboard_scene(fixture_atlas())
        .expect("a snapshot with entities seats a billboard scene");
    assert!(scene.entities.iter().any(|e| e.sprite_id == "player_male"));
    assert!(scene.entities.iter().any(|e| e.sprite_id == "mon_zombie"));
    // The wire world's solid tiles fill their whole z-level, so
    // the rendered surface is the level top: the billboard lift
    // is a full level height (3 blocks), not the fixture
    // scenes' 1-block floor-slab lift — at the slab lift every
    // wire billboard is buried inside the opaque fill.
    assert_eq!(scene.base_lift, 3.0);
    assert_eq!(wire.billboard_base_lift(), 3.0);
}

#[test]
fn player_and_zombie_resolve_to_real_ultica_sprites() {
    let Some(atlas) = real_atlas() else { return };
    // The game's own resolution: the avatar's base entry
    // `player_male` has fg `skin_brown_m`; the `mon_zombie`
    // entry's fg is a weighted variant list whose primary
    // (weight 150, first on ties) is `mon_zombie_2`.
    assert_eq!(atlas.resolve("player_male"), Some("skin_brown_m"));
    assert_eq!(atlas.resolve("mon_zombie"), Some("mon_zombie_2"));

    let (_reg, snap) = build();
    let entities = billboard_entities(&snap);
    let cam = Camera {
        target: tile_feet([5, 5, 0]),
        yaw_deg: 180.0,
        pitch_deg: 34.0,
        distance: 18.0,
    };
    let frame = billboard::scene::build_frame(&atlas, &entities, &cam, 0.0);
    for (sprite, pos) in [("player_male", [5, 5, 0]), ("mon_zombie", [7, 5, 0])] {
        let e = entities
            .iter()
            .find(|e| e.sprite_id == sprite)
            .unwrap_or_else(|| panic!("{sprite} converted"));
        let inst = frame
            .instances
            .iter()
            .find(|i| i.entity_id == e.id)
            .unwrap_or_else(|| panic!("a billboard instance is built for the {sprite}"));
        let feet = tile_feet(pos);
        assert!(
            (inst.anchor.x - feet.x).abs() < 0.6 && (inst.anchor.z - feet.z).abs() < 0.6,
            "{sprite} instance anchored at its wire position: {:?} vs feet {feet:?}",
            inst.anchor
        );
        // The real sprites are 32x48: a full tile wide, 1.5
        // tiles tall — not a 1x1 hole, not a 32x32 stand-in.
        assert!(
            (inst.size.x - 1.0).abs() < 0.05 && (inst.size.y - 1.5).abs() < 0.05,
            "{sprite} instance carries the real sprite's geometry: {:?}",
            inst.size
        );
    }
    // The composites themselves: real dimensions, real bodies.
    // And the feet: UltiCa's 32x48 humanoids carry transparent
    // rows BELOW the feet (measured from the atlas: the player's
    // last opaque row is 42 of 48, the zombie's 40 of 48), so a
    // quad anchored by its canvas bottom leaves the feet
    // floating 5-7 px above the floor while the blob shadow sits
    // on it. The composite records that margin and the instance
    // geometry subtracts it, standing the feet on the floor.
    for (id, want_opaque, want_inset) in
        [("player_male", 200, 5), ("mon_zombie", 200, 7)]
    {
        let (comp, missing) = billboard::composite::CompositeCache::new().get_or_compose(
            &atlas,
            id,
            &[],
            billboard::composite::Facing::None,
        );
        assert!(missing.is_empty(), "{id} resolves in the atlas: {missing:?}");
        assert_eq!(
            (comp.width(), comp.height()),
            (32, 48),
            "{id} composite is the real sprite's size"
        );
        assert_eq!(
            comp.feet_inset_px, want_inset,
            "{id} feet inset is the atlas's own transparent margin"
        );
        let opaque = (0..comp.height() as i32)
            .flat_map(|y| (0..comp.width() as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| comp.alpha_at(x, y) > 127)
            .count();
        assert!(
            opaque > want_opaque,
            "{id} composite has a body: {opaque} opaque px"
        );
        // The geometry's quad bottom sits that far BELOW the
        // feet line, so the feet — not the canvas — land on the
        // floor. (The live instance adds the idle bob on top,
        // ±0.03, so the exact value is asserted on the geometry,
        // not the bobbed instance.)
        let (_, bottom, _) =
            billboard::instance::composite_geometry(&comp, 32.0);
        let want_bottom = -(want_inset as f32) / 32.0;
        assert!(
            (bottom - want_bottom).abs() < 1e-6,
            "{id} geometry bottom is the feet inset below the floor: {bottom} vs {want_bottom}"
        );
    }
}

#[test]
fn player_billboard_tracks_delta_updates() {
    let (_reg, snap0) = build();
    let mut snap = snap0.clone();
    // The player steps south: a Delta entity_update, folded the
    // way the live loop folds it.
    let mut moved: EntityT = snap
        .entities
        .as_ref()
        .unwrap()
        .iter()
        .find(|e| e.sprite_id.as_deref() == Some("player_male"))
        .unwrap()
        .clone();
    moved.pos = Some(proto::Vec3iT { x: 5, y: 6, z: 0 });
    let mut delta = DeltaT::default();
    delta.entity_updates = Some(vec![moved]);
    apply_delta_to_snapshot(&mut snap, &delta);
    let entities = billboard_entities(&snap);
    let player = entities
        .iter()
        .find(|e| e.sprite_id == "player_male")
        .expect("player still converted after the delta");
    assert_eq!(player.pos, [5, 6, 0], "player billboard tracks the delta");
}

/// The most common opaque RGB of a sprite in the atlas — the
/// colour a capture of that sprite must contain.
fn dominant_color(atlas: &SpriteAtlas, sprite: &str) -> [u8; 3] {
    let img = atlas.sprite_image(sprite).expect("sprite image");
    let mut counts: std::collections::HashMap<[u8; 3], usize> = std::collections::HashMap::new();
    for p in img.pixels() {
        if p.0[3] > 127 {
            *counts.entry([p.0[0], p.0[1], p.0[2]]).or_insert(0) += 1;
        }
    }
    *counts
        .iter()
        .max_by_key(|(_, n)| *n)
        .expect("sprite has opaque pixels")
        .0
}

#[test]
fn wire_capture_shows_player_and_enemy_sprites() {
    let Some(atlas) = real_atlas() else { return };
    let (reg, snap) = build();
    let defs = reg.tiledefs().clone();
    let wire = WireScene::from_wire(&defs, &snap, &palette_resolver(&defs));
    let out = std::env::temp_dir().join(format!(
        "wire-player-billboard-gate-{}.png",
        std::process::id()
    ));
    let mut args = CaptureArgs::default();
    args.snapshot = "wire".to_string();
    args.billboards = wire.billboard_scene(atlas.clone());
    args.wire = Some(wire);
    args.textures = Some(ultica_dir());
    args.camera_target = (5.5, 5.5);
    args.distance = 12.0;
    args.cutaway_z = 0;
    args.size = (640, 360);
    args.renderer = RendererKind::Raster;
    args.out = out.clone();
    let outcome = match run_capture(&args) {
        Ok(o) => o,
        Err(CaptureError::NoAdapter) => {
            eprintln!("wire player billboard gate: no wgpu adapter — GPU proof skipped (loud)");
            return;
        }
        Err(e) => panic!("wire capture failed: {e}"),
    };
    let img = image::open(&outcome.out).expect("capture readable").to_rgba8();
    let near = |c: [u8; 3]| {
        img.pixels()
            .filter(|p| (0..3).all(|i| (p.0[i] as i32 - c[i] as i32).abs() <= 12))
            .count()
    };
    // Each sprite's own dominant colour, sampled from the
    // atlas: the player's skin tone, the zombie's dark hide.
    let player_px = near(dominant_color(&atlas, "skin_brown_m"));
    let zombie_px = near(dominant_color(&atlas, "mon_zombie_2"));
    assert!(
        player_px > 30,
        "the player character's real sprite is in the frame ({player_px} px)"
    );
    assert!(
        zombie_px > 30,
        "the enemy's real sprite is in the same frame ({zombie_px} px)"
    );
    println!(
        "wire player billboard gate: player {player_px} px, zombie {zombie_px} px, {} distinct colours",
        outcome.distinct_colors
    );
}
