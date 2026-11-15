//! Billboard instance generation, blob shadows, picking, and the
//! fake-data shapes the crate develops against.
//!
//! World convention (shared with `crates/mesher` / `crates/fx`):
//! game tile (x east, y south, z-level) -> world
//! `(x + 0.5, z * 3.0, -(y + 0.5))` — one tile is one world unit,
//! one z-level is 3 units (the Phase 0 world scale), world +Z
//! points north. A *billboard* is a quad that rotates about the
//! world Y axis only (*cylindrical billboarding*): it always faces
//! the camera horizontally and never tilts, so creatures stay
//! upright at any pitch.
//!
//! Sprite anchoring: world size comes from real pixel size with
//! `tile_pixel_size` (32 for UltiCa) pixels per world unit. A
//! sprite's rect in tile-pixel space is
//! `(offset_x, offset_y)..(offset_x + w, offset_y + h)`; its
//! horizontal center relative to the tile center and its bottom
//! relative to the tile's floor line give the quad's anchor —
//! oversized sprites extend past the tile instead of being centered
//! or clamped to 32x32.

use glam::{Vec2, Vec3};

use crate::composite::{Composite, Facing};

/// Baseline tile pixel size (UltiCa). The protocol's `Hello` carries
/// the real value per tileset; 32 is the fixture/default.
pub const TILE_PIXEL_SIZE: f32 = 32.0;
/// World units per game z-level (Phase 0 world scale).
pub const Z_LEVEL_HEIGHT: f32 = 3.0;

/// Fake entity mirroring the protocol `Entity` table
/// (`schema/cdda_ipc.fbs`): `sprite_id` is the base sprite,
/// `overlays` are in game order, `facing` is the wire byte.
#[derive(Debug, Clone)]
pub struct FakeEntity {
    pub id: u64,
    pub sprite_id: String,
    pub pos: [i32; 3],
    pub facing: Facing,
    pub overlays: Vec<String>,
    pub hp_frac: u8,
    pub flags: u32,
    /// Designed suppression input for blob shadows (gate 6.8): an
    /// entity the game reports as indoors/under a roof casts no
    /// blob shadow. The engine derives this from the tile's roof /
    /// lit state at wiring time; here it is an explicit flag.
    pub under_roof: bool,
    pub bob_phase: f32,
    pub last_hit_s: f32,
}

impl FakeEntity {
    pub fn creature(id: u64, sprite: &str, pos: [i32; 3]) -> Self {
        FakeEntity {
            id,
            sprite_id: sprite.into(),
            pos,
            facing: Facing::None,
            overlays: Vec::new(),
            hp_frac: 255,
            flags: 0,
            under_roof: false,
            bob_phase: id as f32 * 0.7,
            last_hit_s: -100.0,
        }
    }
}

/// Fake per-tile item data mirroring `SubmapBlock.top_item` /
/// `item_count` (uppermost item def id per tile, 0 = none; stack
/// size per tile). Item-pile billboards need only the uppermost
/// item — the engine never enumerates the pile (plan §6).
#[derive(Debug, Clone)]
pub struct FakeItemTiles {
    pub origin: [i32; 3],
    /// Row-major 12x12, def-id -> sprite name resolved by the caller.
    pub top_item_sprite: Vec<Option<String>>,
    pub item_count: Vec<u16>,
}

impl FakeItemTiles {
    pub fn empty(origin: [i32; 3]) -> Self {
        FakeItemTiles {
            origin,
            top_item_sprite: vec![None; 144],
            item_count: vec![0; 144],
        }
    }
    pub fn set(&mut self, lx: usize, ly: usize, sprite: &str, count: u16) {
        let i = ly * 12 + lx;
        self.top_item_sprite[i] = Some(sprite.to_string());
        self.item_count[i] = count;
    }
}

/// Feet position (world) of a game tile position.
pub fn tile_feet(pos: [i32; 3]) -> Vec3 {
    Vec3::new(
        pos[0] as f32 + 0.5,
        pos[2] as f32 * Z_LEVEL_HEIGHT,
        -(pos[1] as f32 + 0.5),
    )
}

/// Horizontal billboard axes for a camera at `eye` looking at a
/// billboard anchored at `anchor`: returns (right, forward) where
/// forward is the horizontal direction from the billboard toward
/// the camera. Rotation is about Y only — `right.y == 0` always.
pub fn billboard_axes(anchor: Vec3, eye: Vec3) -> (Vec3, Vec3) {
    let mut fwd = eye - anchor;
    fwd.y = 0.0;
    let fwd = if fwd.length_squared() < 1e-8 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        fwd.normalize()
    };
    let right = Vec3::new(fwd.z, 0.0, -fwd.x).normalize_or_zero();
    let right = if right.length_squared() < 1e-8 {
        Vec3::X
    } else {
        right
    };
    (right, fwd)
}

/// One billboard instance: everything the GPU vertex shader (and
/// the CPU picker) needs. `anchor` is the feet-center of the tile;
/// `center_offset` is the sprite's horizontal center relative to
/// the tile center, in world units, applied along the billboard's
/// right axis; `bottom` is the sprite bottom's height above the
/// feet; `size` is (width, height) in world units.
#[derive(Debug, Clone)]
pub struct BillboardInstance {
    pub entity_id: u64,
    pub anchor: Vec3,
    pub center_offset: f32,
    pub bottom: f32,
    pub size: Vec2,
    pub uv_rect: [f32; 4],
    /// Tint / hit-flash mix (rgb tint, a = flash strength).
    pub tint: [f32; 4],
    pub mirrored: bool,
    /// Distance to camera, for back-to-front sorting.
    pub depth: f32,
}

/// Geometry derived from a composite's tile-pixel placement.
pub fn composite_geometry(comp: &Composite, tile_pixel_size: f32) -> (f32, f32, Vec2) {
    let tps = tile_pixel_size.max(1.0);
    let w = comp.width() as f32 / tps;
    let h = comp.height() as f32 / tps;
    // Horizontal: sprite center px vs tile center px (tps/2).
    let center_px = comp.origin_x as f32 + comp.width() as f32 / 2.0;
    let center_offset = (center_px - tps / 2.0) / tps;
    // Vertical: sprite bottom px vs tile bottom (tps). A sprite
    // whose bottom is above the tile bottom floats; the common
    // case (bottom == tps) sits exactly on the feet line.
    let bottom_px = comp.origin_y as f32 + comp.height() as f32;
    let bottom = (tps - bottom_px) / tps;
    (center_offset, bottom, Vec2::new(w, h))
}

/// Build the instance for one entity + its composite. `uv_rect`
/// addresses the composite inside whatever atlas the caller packed
/// it into (identity for a standalone composite texture).
pub fn entity_instance(
    e: &FakeEntity,
    comp: &Composite,
    uv_rect: [f32; 4],
    eye: Vec3,
    time_s: f32,
    tile_pixel_size: f32,
) -> BillboardInstance {
    let (center_offset, bottom, size) = composite_geometry(comp, tile_pixel_size);
    let anchor = tile_feet(e.pos);
    let bob = crate::composite::bob_offset(time_s, e.bob_phase);
    let flash = crate::composite::hit_flash(time_s, e.last_hit_s);
    BillboardInstance {
        entity_id: e.id,
        anchor,
        center_offset,
        bottom: bottom + bob,
        size,
        uv_rect,
        tint: [1.0, 1.0, 1.0, flash],
        mirrored: e.facing.mirrored(),
        depth: anchor.distance(eye),
    }
}

/// The four world-space corners of a billboard instance for the
/// given camera, in (bottom-left, bottom-right, top-right,
/// top-left) order matching UV (0,1),(1,1),(1,0),(0,0) — image row
/// 0 is the sprite top.
pub fn instance_corners(inst: &BillboardInstance, eye: Vec3) -> [Vec3; 4] {
    let (right, _) = billboard_axes(inst.anchor, eye);
    let center = inst.anchor + right * inst.center_offset;
    let hw = inst.size.x / 2.0;
    let base = center + Vec3::Y * inst.bottom;
    [
        base - right * hw,
        base + right * hw,
        base + right * hw + Vec3::Y * inst.size.y,
        base - right * hw + Vec3::Y * inst.size.y,
    ]
}

/// Sort instances back-to-front (painter's order for the blended
/// pass; the cutout pass is depth-tested and order-independent,
/// but sorting both keeps one code path).
pub fn sort_back_to_front(instances: &mut [BillboardInstance]) {
    instances.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

// ---------------------------------------------------------------------
// Blob shadows (gate 6.8)
// ---------------------------------------------------------------------

/// One blob shadow: a flat quad on the ground under an entity.
#[derive(Debug, Clone, Copy)]
pub struct ShadowInstance {
    pub center: Vec3,
    /// Quad half-extent in world units (scales with sprite width).
    pub radius: f32,
    pub strength: f32,
}

/// Shadow for an entity, or `None` when suppressed: under-roof
/// entities cast no blob shadow (the designed suppression flag —
/// a sun blob under a roof reads as a rendering bug).
pub fn shadow_for(e: &FakeEntity, comp: &Composite, tile_pixel_size: f32) -> Option<ShadowInstance> {
    if e.under_roof {
        return None;
    }
    let (_, _, size) = composite_geometry(comp, tile_pixel_size);
    let mut c = tile_feet(e.pos);
    c.y += 0.02; // lift off the floor to avoid z-fighting
    Some(ShadowInstance {
        center: c,
        radius: (size.x * 0.42).clamp(0.15, 1.2),
        strength: 0.45,
    })
}

// ---------------------------------------------------------------------
// Picking (gate 6.9)
// ---------------------------------------------------------------------

/// Ray vs one billboard: intersect the ray with the billboard's
/// (cylindrical, camera-facing) plane, convert to sprite UV, then
/// to composite pixels using the real size/offset geometry, and
/// hit-test the composited alpha. Returns the composite pixel
/// coordinate on a hit. Transparent pixels pass through (None).
pub fn pick_billboard(
    ray_origin: Vec3,
    ray_dir: Vec3,
    inst: &BillboardInstance,
    comp: &Composite,
    eye: Vec3,
) -> Option<(i32, i32)> {
    let corners = instance_corners(inst, eye);
    let (right, fwd) = billboard_axes(inst.anchor, eye);
    // Plane through the anchor with normal = -fwd (facing camera).
    let n = -fwd;
    let denom = ray_dir.dot(n);
    if denom.abs() < 1e-8 {
        return None;
    }
    let center = inst.anchor + right * inst.center_offset + Vec3::Y * (inst.bottom + inst.size.y / 2.0);
    let t = (center - ray_origin).dot(n) / denom;
    if t < 0.0 {
        return None;
    }
    let hit = ray_origin + ray_dir * t;
    let rel = hit - corners[0];
    let u = rel.dot(right) / inst.size.x.max(1e-6);
    let v_from_bottom = rel.y / inst.size.y.max(1e-6);
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v_from_bottom) {
        return None;
    }
    // UV: image row 0 is the top. Mirrored sprites sample flipped.
    let uu = if inst.mirrored { 1.0 - u } else { u };
    let px = (uu * comp.width() as f32) as i32;
    let py = ((1.0 - v_from_bottom) * comp.height() as f32) as i32;
    let px = px.min(comp.width() as i32 - 1);
    let py = py.min(comp.height() as i32 - 1);
    if comp.alpha_at(px, py) > 127 {
        Some((px, py))
    } else {
        None
    }
}

/// Pick against many billboards: nearest hit along the ray wins.
pub fn pick_any(
    ray_origin: Vec3,
    ray_dir: Vec3,
    items: &[(&BillboardInstance, &Composite)],
    eye: Vec3,
) -> Option<u64> {
    let mut best: Option<(f32, u64)> = None;
    for (inst, comp) in items {
        if pick_billboard(ray_origin, ray_dir, inst, comp, eye).is_some() {
            let d = inst.anchor.distance(ray_origin);
            if best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, inst.entity_id));
            }
        }
    }
    best.map(|(_, id)| id)
}

// ---------------------------------------------------------------------
// Item piles (gate 6.7, engine half)
// ---------------------------------------------------------------------

/// Item-pile billboards: one small billboard per tile that has an
/// uppermost item, anchored at the tile with a slight per-pile
/// scale-up when the stack is large. Uses the atlas sprite directly
/// (items have no overlays).
pub fn item_instances(
    tiles: &FakeItemTiles,
    atlas: &crate::atlas::SpriteAtlas,
    eye: Vec3,
) -> Vec<BillboardInstance> {
    let mut out = Vec::new();
    for ly in 0..12 {
        for lx in 0..12 {
            let i = ly * 12 + lx;
            let (Some(name), Some(uv)) = (&tiles.top_item_sprite[i], tiles.top_item_sprite[i].as_ref().and_then(|n| atlas.get(n))) else {
                continue;
            };
            let _ = name;
            let pos = [tiles.origin[0] + lx as i32, tiles.origin[1] + ly as i32, tiles.origin[2]];
            let anchor = tile_feet(pos);
            let tps = TILE_PIXEL_SIZE;
            let w = uv.width as f32 / tps;
            let h = uv.height as f32 / tps;
            let center_offset = ((uv.offset_x as f32 + uv.width as f32 / 2.0) - tps / 2.0) / tps;
            let bottom = (tps - (uv.offset_y as f32 + uv.height as f32)) / tps;
            let pile_boost = if tiles.item_count[i] > 4 { 0.08 } else { 0.0 };
            out.push(BillboardInstance {
                entity_id: 1_000_000 + i as u64,
                anchor,
                center_offset,
                bottom: bottom + pile_boost,
                size: Vec2::new(w, h),
                uv_rect: uv.uv_rect(atlas.image.width(), atlas.image.height()),
                tint: [1.0, 1.0, 1.0, 0.0],
                mirrored: false,
                depth: anchor.distance(eye),
            });
        }
    }
    out
}

/// Variety pass (gate 6.7): resolve every id through the atlas and
/// report the ones that fail — the engine-side analogue of the
/// voxelizer's missing list. Returns (resolved, missing).
pub fn variety_pass<'a>(
    atlas: &crate::atlas::SpriteAtlas,
    ids: impl IntoIterator<Item = &'a str>,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut ok = Vec::new();
    let mut missing = Vec::new();
    for id in ids {
        if atlas.get(id).is_some() {
            ok.push(id);
        } else {
            missing.push(id);
        }
    }
    (ok, missing)
}
