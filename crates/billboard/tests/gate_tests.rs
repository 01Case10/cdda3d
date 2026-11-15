//! Phase 6 gate tests (Part II §9): 6.1 orientation, 6.2 depth,
//! 6.3 alpha edges, 6.4 engine-half overlay composition, 6.5 cache,
//! 6.6 facing, 6.7 variety, 6.8 shadows, 6.9 picking, 6.10 stress,
//! 6.11 visual scene. 6.4's game-side half is BLOCKED on Phase 0
//! tests 0.1/0.2 (needs the real tiles build) and is not faked
//! here — see docs/phase6-billboards.md.

use billboard::atlas::{fixture_atlas, SpriteAtlas};
use billboard::composite::{
    compose, game_draw_order, hit_flash, CompositeCache, Facing, WeightedFrames,
};
use billboard::instance::{
    billboard_axes, entity_instance, instance_corners, item_instances, pick_any, pick_billboard,
    shadow_for, tile_feet, variety_pass, FakeEntity, FakeItemTiles, TILE_PIXEL_SIZE,
};
use billboard::render::{render_frame, wall_scene, AlphaMode, Camera, CaptureRequest};
use billboard::scene::{build_frame, crowd_entities, stress_entities};
use glam::Vec3;

fn atlas() -> SpriteAtlas {
    fixture_atlas()
}

fn zombie_comp(cache: &mut CompositeCache, a: &SpriteAtlas) -> std::rc::Rc<billboard::composite::Composite> {
    cache.get_or_compose(a, "mon_zombie", &[], Facing::None).0
}

// ---------------------------------------------------------------- atlas format
#[test]
fn atlas_roundtrip_voxelizer_format() {
    let a = atlas();
    let dir = std::env::temp_dir().join(format!("bb-atlas-{}", std::process::id()));
    a.save(&dir).unwrap();
    let b = SpriteAtlas::load(&dir).unwrap();
    assert_eq!(a.uvs, b.uvs, "sprites.json must round-trip exactly");
    assert_eq!(a.image.dimensions(), b.image.dimensions());
    let hulk = b.get("mon_hulk").unwrap();
    assert_eq!((hulk.width, hulk.height, hulk.offset_x, hulk.offset_y), (64, 64, -16, -32));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------- 6.1
#[test]
fn gate_6_1_cylindrical_orientation_all_yaws() {
    let a = atlas();
    let mut cache = CompositeCache::new();
    let comp = zombie_comp(&mut cache, &a);
    let e = FakeEntity::creature(1, "mon_zombie", [5, 5, 0]);
    for yaw_deg in [0.0f32, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] {
        let yaw = yaw_deg.to_radians();
        let eye = tile_feet(e.pos) + Vec3::new(yaw.sin() * 10.0, 4.0, yaw.cos() * 10.0);
        let (right, fwd) = billboard_axes(tile_feet(e.pos), eye);
        assert_eq!(right.y, 0.0, "right axis must stay horizontal at yaw {yaw_deg}");
        assert_eq!(fwd.y, 0.0, "forward must be horizontal (Y-only rotation) at yaw {yaw_deg}");
        let inst = entity_instance(&e, &comp, [0.0, 0.0, 1.0, 1.0], eye, 0.0, TILE_PIXEL_SIZE);
        let c = instance_corners(&inst, eye);
        // Upright: bottom edge horizontal, top edge directly above bottom.
        assert!((c[0].y - c[1].y).abs() < 1e-5, "bottom edge not level at yaw {yaw_deg}");
        assert!((c[3].y - c[2].y).abs() < 1e-5, "top edge not level at yaw {yaw_deg}");
        assert!((c[3].x - c[0].x).abs() < 1e-5 && (c[3].z - c[0].z).abs() < 1e-5, "left edge not vertical at yaw {yaw_deg}");
        assert!((c[2].y - c[1].y - inst.size.y).abs() < 1e-4, "height changed at yaw {yaw_deg}");
        let width = c[0].distance(c[1]);
        assert!((width - inst.size.x).abs() < 1e-4, "width changed at yaw {yaw_deg}: {width}");
        // Faces the camera: quad normal (right x up) points toward eye.
        let normal = (c[1] - c[0]).cross(c[3] - c[0]);
        let to_eye = eye - inst.anchor;
        assert!(normal.dot(to_eye) > 0.0, "quad must face camera at yaw {yaw_deg}");
    }
}

// ---------------------------------------------------------------- 6.4 engine half + 6.5 + 6.6
#[test]
fn gate_6_4_engine_compositor_game_order() {
    let a = atlas();
    // Reference fixture: the game's documented order — base first,
    // then get_overlay_ids() order (never re-sorted).
    let order = game_draw_order("mon_zombie", &["overlay_jacket".to_string(), "overlay_jeans".to_string(), "overlay_knife".to_string()]);
    assert_eq!(order, vec!["mon_zombie", "overlay_jacket", "overlay_jeans", "overlay_knife"]);
    let (comp, missing) = compose(&a, &order, Facing::None);
    assert!(missing.is_empty());
    // Jacket covers the torso (sprite-local 16,18), jeans the legs
    // (16,28), knife the hand (28,18), head stays base (16,6).
    assert_eq!(comp.image.get_pixel(16, 18).0, [40, 60, 160, 255], "jacket must draw over the torso");
    assert_eq!(comp.image.get_pixel(16, 28).0, [30, 30, 90, 255], "jeans must draw over the legs");
    assert_eq!(comp.image.get_pixel(28, 18).0, [200, 200, 210, 255], "knife (last overlay) must be visible");
    assert_eq!(comp.image.get_pixel(16, 6).0, [90, 160, 80, 255], "head stays the base sprite");
    // Order is semantic: semi-transparent ghost OVER the zombie
    // blends; the zombie OVER the ghost replaces its center.
    let (over, _) = compose(&a, &["mon_zombie".into(), "mon_ghost".into()], Facing::None);
    let (under, _) = compose(&a, &["mon_ghost".into(), "mon_zombie".into()], Facing::None);
    assert_ne!(over.image.get_pixel(16, 16).0, under.image.get_pixel(16, 16).0, "overlay order must change the result");
}

#[test]
fn gate_6_5_overlay_cache_signature() {
    let a = atlas();
    let mut cache = CompositeCache::new();
    let ov = vec!["overlay_jacket".to_string()];
    let (c1, _) = cache.get_or_compose(&a, "mon_zombie", &ov, Facing::None);
    assert_eq!((cache.created, cache.hits), (1, 0));
    let (c2, _) = cache.get_or_compose(&a, "mon_zombie", &ov, Facing::None);
    assert_eq!((cache.created, cache.hits), (1, 1), "same signature must be a hit");
    assert!(std::rc::Rc::ptr_eq(&c1, &c2));
    // One overlay changed -> exactly one new composite.
    let ov2 = vec!["overlay_jeans".to_string()];
    cache.get_or_compose(&a, "mon_zombie", &ov2, Facing::None);
    assert_eq!(cache.created, 2);
    // Reordering the same overlays is a different signature.
    let ov3 = vec!["overlay_jacket".to_string(), "overlay_jeans".to_string()];
    let ov4 = vec!["overlay_jeans".to_string(), "overlay_jacket".to_string()];
    cache.get_or_compose(&a, "mon_zombie", &ov3, Facing::None);
    cache.get_or_compose(&a, "mon_zombie", &ov4, Facing::None);
    assert_eq!(cache.created, 4, "order is part of the signature");
    // Facing is part of the signature too.
    cache.get_or_compose(&a, "mon_zombie", &ov, Facing::Left);
    assert_eq!(cache.created, 5);
}

#[test]
fn gate_6_6_facing_mirror_only_left() {
    let a = atlas();
    let ids = vec!["mon_zombie".to_string(), "overlay_knife".to_string()];
    let (right, _) = compose(&a, &ids, Facing::Right);
    let (none, _) = compose(&a, &ids, Facing::None);
    assert_eq!(right.image, none.image, "RIGHT and NONE must be identical (unmirrored)");
    let (left, _) = compose(&a, &ids, Facing::Left);
    // Knife sits at x 26..30 unmirrored; mirrored it must sit at 2..6.
    assert_eq!(left.image.get_pixel(3, 18).0, [200, 200, 210, 255], "LEFT must mirror horizontally");
    assert_eq!(left.image.get_pixel(28, 18).0[3], 0, "unmirrored knife position must be empty when mirrored");
    assert!(!Facing::Right.mirrored() && !Facing::None.mirrored() && Facing::Left.mirrored());
    assert_eq!(Facing::from_wire(1), Facing::Left);
    assert_eq!(Facing::from_wire(2), Facing::Right);
    assert_eq!(Facing::from_wire(0), Facing::None);
}

// ---------------------------------------------------------------- 6.7
#[test]
fn gate_6_7_variety_and_items() {
    let a = atlas();
    let ids = ["mon_zombie", "mon_skeleton", "mon_hulk", "mon_ghost", "item_rock", "item_crowbar", "mon_missing_xyz"];
    let (ok, missing) = variety_pass(&a, ids.iter().copied());
    assert_eq!(ok.len(), 6);
    assert_eq!(missing, vec!["mon_missing_xyz"], "genuine gaps must be reported, not hidden");
    // Item piles: one billboard per tile with an uppermost item.
    let mut tiles = FakeItemTiles::empty([0, 0, 0]);
    tiles.set(1, 1, "item_rock", 3);
    tiles.set(2, 1, "item_crowbar", 1);
    let eye = Vec3::new(5.0, 4.0, -15.0);
    let insts = item_instances(&tiles, &a, eye);
    assert_eq!(insts.len(), 2, "exactly the two item tiles produce billboards");
}

// ---------------------------------------------------------------- 6.8
#[test]
fn gate_6_8_blob_shadow_follows_and_suppressed_under_roof() {
    let a = atlas();
    let mut cache = CompositeCache::new();
    let comp = zombie_comp(&mut cache, &a);
    let e = FakeEntity::creature(1, "mon_zombie", [5, 5, 0]);
    let s = shadow_for(&e, &comp, TILE_PIXEL_SIZE).expect("outdoor entity gets a shadow");
    let feet = tile_feet(e.pos);
    assert!((s.center.x - feet.x).abs() < 1e-6 && (s.center.z - feet.z).abs() < 1e-6, "shadow must follow the entity's tile");
    assert!(s.center.y > feet.y && s.center.y - feet.y < 0.05, "shadow sits just above the floor");
    let mut roofed = e.clone();
    roofed.under_roof = true;
    assert!(shadow_for(&roofed, &comp, TILE_PIXEL_SIZE).is_none(), "under-roof shadow must be suppressed");
    // And it moves when the entity moves.
    let e2 = FakeEntity::creature(1, "mon_zombie", [8, 5, 0]);
    let s2 = shadow_for(&e2, &comp, TILE_PIXEL_SIZE).unwrap();
    assert!((s2.center.x - s.center.x - 3.0).abs() < 1e-6);
}

// ---------------------------------------------------------------- 6.9
#[test]
fn gate_6_9_picking_alpha_and_oversized() {
    let a = atlas();
    let mut cache = CompositeCache::new();
    let comp = zombie_comp(&mut cache, &a);
    let e = FakeEntity::creature(42, "mon_zombie", [5, 5, 0]);
    let eye = tile_feet(e.pos) + Vec3::new(0.0, 0.5, 10.0);
    let inst = entity_instance(&e, &comp, [0.0, 0.0, 1.0, 1.0], eye, 0.0, TILE_PIXEL_SIZE);
    let corners = instance_corners(&inst, eye);
    let ray_to = |u: f32, v: f32| {
        // Bilinear point on the quad (v from bottom).
        let p = corners[0] * (1.0 - u) * (1.0 - v) + corners[1] * u * (1.0 - v) + corners[3] * (1.0 - u) * v + corners[2] * u * v;
        (p - eye).normalize()
    };
    // Center of the zombie body: opaque -> hit.
    assert!(pick_billboard(eye, ray_to(0.5, 0.5), &inst, &comp, eye).is_some(), "opaque pixel must hit");
    assert_eq!(pick_any(eye, ray_to(0.5, 0.5), &[(&inst, &comp)], eye), Some(42));
    // Top-left corner of the sprite is transparent -> pass through.
    assert!(pick_billboard(eye, ray_to(0.02, 0.98), &inst, &comp, eye).is_none(), "transparent pixel must pass through");

    // Oversized hulk (64x64, offset -16,-32): a point well outside
    // the 32x32 tile box but inside the real sprite must hit.
    let (hulk, _) = cache.get_or_compose(&a, "mon_hulk", &[], Facing::None);
    let he = FakeEntity::creature(43, "mon_hulk", [5, 5, 0]);
    let hinst = entity_instance(&he, &hulk, [0.0, 0.0, 1.0, 1.0], eye, 0.0, TILE_PIXEL_SIZE);
    assert!(hinst.size.x > 1.9, "hulk must use its real 64px width, got {}", hinst.size.x);
    let hcorners = instance_corners(&hinst, eye);
    let hp = hcorners[0] * 0.8 + hcorners[1] * 0.2; // near the left edge, beyond the tile box
    let hpoint = Vec3::new(hp.x, hcorners[0].y + 1.0, hp.z);
    let hdir = (hpoint - eye).normalize();
    assert!(pick_billboard(eye, hdir, &hinst, &hulk, eye).is_some(), "oversized sprite must hit outside the 32x32 box");
}

// ---------------------------------------------------------------- animation
#[test]
fn animation_weighted_frames_bob_flash() {
    let wf = WeightedFrames { frames: vec![("a".into(), 3), ("b".into(), 1), ("c".into(), 2)], unit_ms: 100 };
    // Cycle = 6 units = 600 ms: a owns 0-299, b 300-399, c 400-599.
    assert_eq!(wf.frame_at(0), Some("a"));
    assert_eq!(wf.frame_at(299), Some("a"));
    assert_eq!(wf.frame_at(300), Some("b"));
    assert_eq!(wf.frame_at(450), Some("c"));
    assert_eq!(wf.frame_at(600), Some("a"), "cycle must wrap");
    assert_eq!(billboard::composite::bob_offset(0.0, 0.0), 0.0);
    assert!(hit_flash(1.0, 1.0) > 0.99);
    assert!(hit_flash(2.0, 1.0) < hit_flash(1.2, 1.0), "flash decays");
    assert_eq!(hit_flash(0.5, 1.0), 0.0, "no flash before the hit");
}

// ---------------------------------------------------------------- 6.3 CPU half
#[test]
fn gate_6_3_premultiply_no_dark_halo_math() {
    // Premultiplied blend of the ghost's semi-transparent bright
    // border over a dark background must stay proportionally
    // bright — never darker than the background (the halo defect).
    let img = billboard::render::premultiply(&atlas().sprite_image("mon_ghost").unwrap());
    let border = img.get_pixel(0, 0).0; // [255,255,255,110] straight
    assert_eq!(border[3], 110);
    let bg = [26u8, 31, 43];
    let a = border[3] as f32 / 255.0;
    for c in 0..3 {
        let out = border[c] as f32 + bg[c] as f32 * (1.0 - a);
        assert!(out >= bg[c] as f32, "premultiplied composite darkened channel {c}: {out} < {}", bg[c]);
        assert!(out > 100.0, "bright sprite over dark bg must read bright, got {out}");
    }
}

// ---------------------------------------------------------------- GPU gates
fn render_crowd(entities: &[FakeEntity]) -> Option<image::RgbaImage> {
    let a = atlas();
    let camera = Camera::default();
    let frame = build_frame(&a, entities, &camera, 0.0);
    let boxes = wall_scene();
    let req = CaptureRequest { boxes: &boxes, atlas_image: &frame.packed, instances: &frame.instances, shadows: &frame.shadows, camera, size: (640, 360), mode: AlphaMode::Blend, time_s: 0.0 };
    match render_frame(&req) {
        Ok(out) => Some(out.image),
        Err(billboard::render::BillboardError::NoAdapter) => None,
        Err(e) => panic!("render failed: {e}"),
    }
}

fn count_color(img: &image::RgbaImage, pred: impl Fn([u8; 4]) -> bool) -> usize {
    img.pixels().filter(|p| pred(p.0)).count()
}

#[test]
fn gate_6_2_depth_occlusion_walls_and_window() {
    let Some(base) = render_crowd(&[]) else { eprintln!("no adapter — skipped"); return; };
    // Entity fully behind the wall: render must be identical to
    // the empty scene (fully occluded by scene depth).
    let hidden = vec![FakeEntity::creature(7, "mon_skeleton", [2, -1, 0])];
    let Some(hidden_img) = render_crowd(&hidden) else { return; };
    assert_eq!(hidden_img.as_raw(), base.as_raw(), "entity behind the wall must be fully occluded");
    // Entity behind the wall but aligned with the window: visible.
    let windowed = vec![FakeEntity::creature(6, "mon_zombie", [6, -1, 0])];
    let Some(win_img) = render_crowd(&windowed) else { return; };
    assert_ne!(win_img.as_raw(), base.as_raw(), "entity behind the window opening must be visible");
    // Entity in front of the wall: its green must appear.
    let front = vec![FakeEntity::creature(1, "mon_zombie", [3, 4, 0])];
    let Some(front_img) = render_crowd(&front) else { return; };
    let greens = count_color(&front_img, |p| p[1] > 100 && p[0] < 110 && p[2] < 110);
    assert!(greens > 50, "front zombie's green must render, found {greens} px");
}

#[test]
fn gate_6_3_gpu_soft_edges_no_dark_halo() {
    // Ghost close-up over the dark clear color.
    let a = atlas();
    let camera = Camera { target: tile_feet([5, 5, 0]) + Vec3::new(0.0, 0.5, 0.0), yaw_deg: 180.0, pitch_deg: 20.0, distance: 4.0 };
    let entities = vec![FakeEntity::creature(9, "mon_ghost", [5, 5, 0])];
    let frame = build_frame(&a, &entities, &camera, 0.0);
    let req = CaptureRequest { boxes: &[], atlas_image: &frame.packed, instances: &frame.instances, shadows: &[], camera, size: (320, 240), mode: AlphaMode::Blend, time_s: 0.0 };
    let out = match render_frame(&req) { Ok(o) => o, Err(billboard::render::BillboardError::NoAdapter) => { eprintln!("no adapter — skipped"); return; } Err(e) => panic!("{e}") };
    // The ghost's bright core must be present...
    let bright = count_color(&out.image, |p| p[0] > 200 && p[1] > 200);
    assert!(bright > 100, "ghost core must render bright, found {bright}");
    // ...and its semi-transparent border must blend UP from the
    // clear color, never below it (a dark halo is below-clear).
    let clear = [26u8, 31, 43];
    let below = count_color(&out.image, |p| p[0] < clear[0] - 4 && p[1] < clear[1] - 4 && p[2] < clear[2] - 4);
    assert_eq!(below, 0, "no pixel may darken below the clear color around the ghost (halo)");
}

#[test]
fn gate_6_10_stress_2000_billboards() {
    let a = atlas();
    let t0 = std::time::Instant::now();
    let entities = stress_entities(2000);
    let camera = billboard::scene::stress_camera();
    let frame = build_frame(&a, &entities, &camera, 0.0);
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(frame.instances.len(), 2000);
    eprintln!("stress: instance build (compose+pack+generate) {build_ms:.1} ms for 2000 billboards");
    let boxes = wall_scene();
    let req = CaptureRequest { boxes: &boxes, atlas_image: &frame.packed, instances: &frame.instances, shadows: &frame.shadows, camera, size: (640, 360), mode: AlphaMode::Blend, time_s: 0.0 };
    match render_frame(&req) {
        Ok(out) => {
            eprintln!("stress: render+readback {:.1} ms on llvmpipe/lavapipe — NOT target hardware", out.frame_ms);
            let distinct: std::collections::HashSet<[u8; 4]> = out.image.pixels().map(|p| p.0).collect();
            assert!(distinct.len() > 10, "stress render must contain the crowd, {} colors", distinct.len());
        }
        Err(billboard::render::BillboardError::NoAdapter) => eprintln!("no adapter — GPU half skipped"),
        Err(e) => panic!("stress render failed: {e}"),
    }
}

#[test]
fn gate_6_11_visual_crowd_scene() {
    let Some(img) = render_crowd(&crowd_entities()) else { eprintln!("no adapter — skipped"); return; };
    let distinct: std::collections::HashSet<[u8; 4]> = img.pixels().map(|p| p.0).collect();
    assert!(distinct.len() > 25, "crowd scene must be visually rich, {} colors", distinct.len());
    let greens = count_color(&img, |p| p[1] > 100 && p[0] < 110 && p[2] < 110);
    assert!(greens > 100, "zombies must be visible in the crowd, {greens} green px");
    let purples = count_color(&img, |p| p[2] > 100 && p[0] > 60 && p[0] < 150 && p[1] < 110);
    assert!(purples > 50, "the oversized hulk must be visible, {purples} purple px");
    // Shadows: the crowd frame builds one blob shadow per outdoor
    // entity (10 entities, 1 under-roof suppressed -> 9).
    let a = atlas();
    let frame = build_frame(&a, &crowd_entities(), &Camera::default(), 0.0);
    assert_eq!(frame.shadows.len(), 9, "one shadow per outdoor entity, under-roof suppressed");
}
