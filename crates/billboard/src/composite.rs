//! Overlay compositor + cache, facing, and sprite animation.
//!
//! An *overlay composite* is the single RGBA image the engine draws
//! for one creature: its base sprite first, then each overlay id in
//! the exact order the game returns from `get_overlay_ids()` (the
//! protocol's `Entity.overlays` — plan §5 Phase 6 / §6). The engine
//! NEVER re-sorts that list: draw order is the game's semantics
//! (clothing over body, wielded item last), and gate 6.4 compares
//! this compositor against the game's own
//! `cata_tiles::draw_entity_with_overlays`.
//!
//! Each layer is placed by its real geometry: a layer's pixels land
//! at `(offset_x, offset_y)` in *tile-pixel space* (the 32x32 tile
//! cell is `(0,0)..(32,32)`; oversized sprites have negative offsets
//! and extend outside the cell). The composite canvas is the union
//! of all layers' rects in that space, and the composite remembers
//! its origin so world anchoring (see `instance`) stays exact.

use image::RgbaImage;
use std::collections::HashMap;
use std::rc::Rc;

use crate::atlas::SpriteAtlas;

/// Creature facing, mirroring the protocol's `Facing` enum
/// (`schema/cdda_ipc.fbs`): horizontal flip only. There are no other
/// values and no 8-way creature sprites in this system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facing {
    None,
    Left,
    Right,
}

impl Facing {
    /// Wire value -> facing (0 none, 1 left, 2 right).
    pub fn from_wire(v: u8) -> Facing {
        match v {
            1 => Facing::Left,
            2 => Facing::Right,
            _ => Facing::None,
        }
    }
    pub fn mirrored(self) -> bool {
        self == Facing::Left
    }
}

/// Cache key: the ordered sprite ids (base first, then overlays in
/// game order) plus facing. Order is significant — the same ids in a
/// different order are a different composite.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub sprite_ids: Vec<String>,
    pub facing: Facing,
}

/// One composited creature image plus its placement in tile-pixel
/// space: pixel `(0,0)` of the image is tile-pixel
/// `(origin_x, origin_y)`.
#[derive(Debug, Clone)]
pub struct Composite {
    pub image: RgbaImage,
    pub origin_x: i32,
    pub origin_y: i32,
    pub signature: Signature,
}

impl Composite {
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }
    /// Alpha at image pixel (x, y); 0 outside the canvas.
    pub fn alpha_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.image.width() as i32 || y >= self.image.height() as i32 {
            0
        } else {
            self.image.get_pixel(x as u32, y as u32).0[3]
        }
    }
}

/// Straight-alpha "source over" blend of one pixel, computed in f32.
/// Working on straight colors with the correct over operator is what
/// keeps soft edges free of dark halos (gate 6.3): the GPU path uses
/// the premultiplied equivalent (see `render`).
fn over(dst: &mut [u8; 4], src: [u8; 4]) {
    let sa = src[3] as f32 / 255.0;
    if sa <= 0.0 {
        return;
    }
    if sa >= 1.0 {
        *dst = src;
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let oa = sa + da * (1.0 - sa);
    if oa <= 0.0 {
        *dst = [0, 0, 0, 0];
        return;
    }
    for c in 0..3 {
        let sc = src[c] as f32 / 255.0;
        let dc = dst[c] as f32 / 255.0;
        dst[c] = (((sc * sa + dc * da * (1.0 - sa)) / oa) * 255.0).round() as u8;
    }
    dst[3] = (oa * 255.0).round() as u8;
}

/// Compose `sprite_ids` (base first, overlays in game order) from
/// `atlas` into one image. Missing ids are skipped and reported in
/// the returned list (the caller logs them; a fully missing base
/// yields a 1x1 transparent composite).
pub fn compose(
    atlas: &SpriteAtlas,
    sprite_ids: &[String],
    facing: Facing,
) -> (Composite, Vec<String>) {
    let mut missing = Vec::new();
    let mut layers: Vec<(RgbaImage, i32, i32)> = Vec::new();
    for id in sprite_ids {
        match (atlas.get(id), atlas.sprite_image(id)) {
            (Some(uv), Some(img)) => layers.push((img, uv.offset_x, uv.offset_y)),
            _ => missing.push(id.clone()),
        }
    }
    if layers.is_empty() {
        return (
            Composite {
                image: RgbaImage::new(1, 1),
                origin_x: 0,
                origin_y: 0,
                signature: Signature {
                    sprite_ids: sprite_ids.to_vec(),
                    facing,
                },
            },
            missing,
        );
    }
    let min_x = layers.iter().map(|(_, ox, _)| *ox).min().unwrap();
    let min_y = layers.iter().map(|(_, _, oy)| *oy).min().unwrap();
    let max_x = layers
        .iter()
        .map(|(img, ox, _)| *ox + img.width() as i32)
        .max()
        .unwrap();
    let max_y = layers
        .iter()
        .map(|(img, _, oy)| *oy + img.height() as i32)
        .max()
        .unwrap();
    let mut canvas = RgbaImage::new((max_x - min_x) as u32, (max_y - min_y) as u32);
    for (img, ox, oy) in &layers {
        for y in 0..img.height() {
            for x in 0..img.width() {
                let src = img.get_pixel(x, y).0;
                if src[3] == 0 {
                    continue;
                }
                let cx = (*ox - min_x) as u32 + x;
                let cy = (*oy - min_y) as u32 + y;
                let mut dst = canvas.get_pixel(cx, cy).0;
                over(&mut dst, src);
                canvas.put_pixel(cx, cy, image::Rgba(dst));
            }
        }
    }
    if facing.mirrored() {
        canvas = image::imageops::flip_horizontal(&canvas);
        // Mirroring flips the canvas in tile-pixel space too: the
        // new origin is the mirror of the old far edge about the
        // tile cell. Tile-pixel x mirrors as x' = 32 - (x + w) for
        // a rect starting at x with width w... applied to the union
        // canvas: origin' = 32 - max_x.
        return (
            Composite {
                image: canvas,
                origin_x: 32 - max_x,
                origin_y: min_y,
                signature: Signature {
                    sprite_ids: sprite_ids.to_vec(),
                    facing,
                },
            },
            missing,
        );
    }
    (
        Composite {
            image: canvas,
            origin_x: min_x,
            origin_y: min_y,
            signature: Signature {
                sprite_ids: sprite_ids.to_vec(),
                facing,
            },
        },
        missing,
    )
}

/// The documented game draw order for one creature, encoded as the
/// gate 6.4 reference fixture: base sprite first, then
/// `get_overlay_ids()` in returned order (`cata_tiles::
/// draw_entity_with_overlays`, plan §5 Phase 6). The game-side half
/// of 6.4 (capturing the tiles build) is blocked on Phase 0 tests
/// 0.1/0.2; this fixture is the contract the engine half is checked
/// against, not a fake of that capture.
pub fn game_draw_order(base: &str, overlay_ids_in_game_order: &[String]) -> Vec<String> {
    let mut v = vec![base.to_string()];
    v.extend(overlay_ids_in_game_order.iter().cloned());
    v
}

/// Composite cache keyed by [`Signature`] (gate 6.5): a repeated
/// signature is a hit and creates nothing; any change — an added,
/// removed, reordered overlay, or a facing flip — creates exactly
/// one new composite.
#[derive(Debug, Default)]
pub struct CompositeCache {
    map: HashMap<Signature, Rc<Composite>>,
    pub hits: usize,
    pub misses: usize,
    pub created: usize,
}

impl CompositeCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_or_compose(
        &mut self,
        atlas: &SpriteAtlas,
        base: &str,
        overlays_in_game_order: &[String],
        facing: Facing,
    ) -> (Rc<Composite>, Vec<String>) {
        let ids = game_draw_order(base, overlays_in_game_order);
        let sig = Signature {
            sprite_ids: ids.clone(),
            facing,
        };
        if let Some(c) = self.map.get(&sig) {
            self.hits += 1;
            return (c.clone(), Vec::new());
        }
        self.misses += 1;
        self.created += 1;
        let (comp, missing) = compose(atlas, &ids, facing);
        let rc = Rc::new(comp);
        self.map.insert(sig, rc.clone());
        (rc, missing)
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

// ---------------------------------------------------------------------
// Animation: weighted frame selection + shader effect parameters
// ---------------------------------------------------------------------

/// Weighted/animated sprite entry (the tile config's fg may be a
/// list of `{sprite, weight}` choices — plan §5 Phase 6): frames are
/// selected by weight over a repeating cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFrames {
    pub frames: Vec<(String, u32)>,
    /// Milliseconds each weight unit lasts.
    pub unit_ms: u32,
}

impl WeightedFrames {
    pub fn total_weight(&self) -> u32 {
        self.frames.iter().map(|(_, w)| *w).sum()
    }
    /// Pure time -> frame index: the cycle length is
    /// `total_weight * unit_ms`; within a cycle, frame i owns the
    /// span of `weight_i` consecutive units.
    pub fn frame_index_at(&self, time_ms: u64) -> usize {
        let total = self.total_weight();
        if total == 0 || self.frames.is_empty() {
            return 0;
        }
        let unit = (time_ms / self.unit_ms.max(1) as u64) % total as u64;
        let mut acc = 0u64;
        for (i, (_, w)) in self.frames.iter().enumerate() {
            acc += *w as u64;
            if unit < acc {
                return i;
            }
        }
        self.frames.len() - 1
    }
    pub fn frame_at(&self, time_ms: u64) -> Option<&str> {
        self.frames
            .get(self.frame_index_at(time_ms))
            .map(|(n, _)| n.as_str())
    }
}

/// Idle bob offset in world units (fed to the instance as a Y
/// offset / shader parameter): a gentle sine, zero at t=0.
pub fn bob_offset(time_s: f32, phase: f32) -> f32 {
    (time_s * 2.2 + phase).sin() * 0.03
}

/// Hit-flash strength 0..1, decaying exponentially after a hit at
/// `hit_time_s` (the shader mixes the sprite toward white/red by
/// this amount).
pub fn hit_flash(now_s: f32, hit_time_s: f32) -> f32 {
    let dt = now_s - hit_time_s;
    if dt < 0.0 {
        0.0
    } else {
        (-dt * 6.0).exp()
    }
}
