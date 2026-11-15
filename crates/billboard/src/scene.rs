//! Fake-data scenes for the gates, mirroring the protocol shapes
//! (`Entity` / `SubmapBlock` top-item data) the way Phase 2's fake
//! house mirrored snapshots. Real entities arrive with Phase 3/4
//! wiring; the shapes here are field-for-field the wire shapes.

use glam::Vec3;

use crate::atlas::SpriteAtlas;
use crate::composite::{CompositeCache, Facing};
use crate::instance::{
    entity_instance, item_instances, shadow_for, sort_back_to_front, FakeEntity, FakeItemTiles,
    ShadowInstance, TILE_PIXEL_SIZE,
};
use crate::render::Camera;

/// A fully built frame's billboard data: the packed composite
/// texture image, the instances into it, and the shadows.
pub struct BuiltFrame {
    pub packed: image::RgbaImage,
    pub instances: Vec<crate::instance::BillboardInstance>,
    pub shadows: Vec<ShadowInstance>,
}

/// Compose every entity (through the cache), pack the composites,
/// and generate sorted instances + shadows for `camera`.
pub fn build_frame(
    atlas: &SpriteAtlas,
    entities: &[FakeEntity],
    camera: &Camera,
    time_s: f32,
) -> BuiltFrame {
    let eye = camera.eye();
    let mut cache = CompositeCache::new();
    let mut composites = Vec::new();
    let mut per_entity = Vec::new();
    for e in entities {
        let (comp, _missing) = cache.get_or_compose(atlas, &e.sprite_id, &e.overlays, e.facing);
        per_entity.push(comp.clone());
    }
    for c in &per_entity {
        composites.push(c.as_ref());
    }
    let (packed, rects) = crate::render::pack_composites(&composites);
    let mut instances = Vec::new();
    let mut shadows = Vec::new();
    for (e, (comp, rect)) in entities.iter().zip(per_entity.iter().zip(&rects)) {
        instances.push(entity_instance(e, comp, *rect, eye, time_s, TILE_PIXEL_SIZE));
        if let Some(s) = shadow_for(e, comp, TILE_PIXEL_SIZE) {
            shadows.push(s);
        }
    }
    sort_back_to_front(&mut instances);
    BuiltFrame { packed, instances, shadows }
}

/// The gate 6.11 zombie crowd: zombies (some with equipment
/// overlays, one facing LEFT) in front of the wall, one behind the
/// wall visible only through the window, a hulk beside the wall,
/// a ghost (soft edges), skeletons, and an item pile.
pub fn crowd_entities() -> Vec<FakeEntity> {
    let mut v = Vec::new();
    let mut z1 = FakeEntity::creature(1, "mon_zombie", [3, 4, 0]);
    z1.overlays = vec!["overlay_jacket".into(), "overlay_jeans".into()];
    v.push(z1);
    v.push(FakeEntity::creature(2, "mon_zombie", [4, 4, 0]));
    let mut z3 = FakeEntity::creature(3, "mon_zombie", [5, 5, 0]);
    z3.overlays = vec!["overlay_jacket".into(), "overlay_knife".into()];
    z3.facing = Facing::Left;
    v.push(z3);
    v.push(FakeEntity::creature(4, "mon_skeleton", [7, 4, 0]));
    v.push(FakeEntity::creature(5, "mon_skeleton", [8, 5, 0]));
    // Behind the back wall (game y = -1 row is north of the wall at
    // y=0): world z = +0.5, visible only through the window hole.
    v.push(FakeEntity::creature(6, "mon_zombie", [6, -1, 0]));
    // Fully hidden behind the wall — must NOT appear in the render.
    v.push(FakeEntity::creature(7, "mon_skeleton", [2, -1, 0]));
    v.push(FakeEntity::creature(8, "mon_hulk", [9, 4, 0]));
    v.push(FakeEntity::creature(9, "mon_ghost", [2, 5, 0]));
    let mut under = FakeEntity::creature(10, "mon_zombie", [10, 4, 0]);
    under.under_roof = true; // shadow suppressed (gate 6.8)
    v.push(under);
    v
}

/// Item piles for the crowd scene (uppermost item per tile).
pub fn crowd_items() -> FakeItemTiles {
    let mut t = FakeItemTiles::empty([2, 3, 0]);
    t.set(1, 1, "item_rock", 3);
    t.set(2, 1, "item_crowbar", 1);
    t.set(6, 2, "item_rock", 7);
    t
}

/// Item instances for the crowd scene, addressed into the raw
/// fixture atlas (items have no overlays, so no packing needed).
pub fn crowd_item_instances(atlas: &SpriteAtlas, camera: &Camera) -> Vec<crate::instance::BillboardInstance> {
    item_instances(&crowd_items(), atlas, camera.eye())
}

/// `n` zombies on a grid — the gate 6.10 stress crowd.
pub fn stress_entities(n: usize) -> Vec<FakeEntity> {
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        let x = (i % 50) as i32;
        let y = 3 + (i / 50) as i32 % 40;
        let sprite = if i % 7 == 0 { "mon_skeleton" } else { "mon_zombie" };
        v.push(FakeEntity::creature(i as u64, sprite, [x, y, 0]));
    }
    v
}

/// A camera that sees the whole stress grid.
pub fn stress_camera() -> Camera {
    Camera { target: Vec3::new(25.0, 0.0, -22.0), yaw_deg: 180.0, pitch_deg: 50.0, distance: 60.0 }
}
