//! Sprite atlas model: consumes exactly the voxelizer's billboard
//! output format (plan §8) — `sprites.png` (one packed RGBA image)
//! plus `sprites.json` (a list of [`SpriteUv`]: name, atlas position,
//! real width/height, and the per-sprite offset from `tile_info.json`).
//!
//! A *sprite* here is one named tileset image at its real pixel size.
//! Creature sprites are NOT all 32x32 (plan §8): `tall` is 32x64,
//! `large`/`centered` are 64x64, `giant` is 96x96, each with its own
//! offset. Every consumer in this crate works in real pixels and
//! converts through [`SpriteUv`] — nothing assumes 32x32.

use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// One atlas entry — field-for-field the voxelizer's `SpriteUv`
/// (`crates/voxelizer/src/lib.rs`), so `sprites.json` deserializes
/// directly with no translation layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteUv {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub sheet: String,
}

impl SpriteUv {
    /// Normalized UV rect (u0, v0, u1, v1) in an atlas of `aw x ah`.
    pub fn uv_rect(&self, aw: u32, ah: u32) -> [f32; 4] {
        [
            self.x as f32 / aw as f32,
            self.y as f32 / ah as f32,
            (self.x + self.width) as f32 / aw as f32,
            (self.y + self.height) as f32 / ah as f32,
        ]
    }
}

/// A loaded sprite atlas: the packed image plus its UV table.
#[derive(Debug, Clone)]
pub struct SpriteAtlas {
    pub image: RgbaImage,
    pub uvs: BTreeMap<String, SpriteUv>,
    /// Tile-entry id -> sprite name, from the voxelizer's
    /// `tile_sprites.json` (absent = empty). Creatures and items
    /// are named by tile-entry ID on the wire — a monster type
    /// id, the avatar's `player_male` — but the atlas is keyed
    /// by sprite name, and an entry's fg is often a different
    /// name (`player_male` -> `skin_brown_m`) or a weighted
    /// variant list (`mon_zombie` -> `mon_zombie_2`, the
    /// primary). `resolve` bridges the two; without it those
    /// ids composite to nothing.
    pub tile_sprites: BTreeMap<String, String>,
}

impl SpriteAtlas {
    /// Load the voxelizer output pair from a directory containing
    /// `sprites.json` + `sprites.png` (plus the optional
    /// `tile_sprites.json` entry table).
    pub fn load(dir: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(dir.join("sprites.json"))
            .map_err(|e| format!("sprites.json: {e}"))?;
        let list: Vec<SpriteUv> =
            serde_json::from_str(&text).map_err(|e| format!("sprites.json parse: {e}"))?;
        let image = image::open(dir.join("sprites.png"))
            .map_err(|e| format!("sprites.png: {e}"))?
            .to_rgba8();
        let mut atlas = Self::from_parts(image, list);
        let table_path = dir.join("tile_sprites.json");
        if table_path.exists() {
            let text = std::fs::read_to_string(&table_path)
                .map_err(|e| format!("tile_sprites.json: {e}"))?;
            atlas.tile_sprites = serde_json::from_str(&text)
                .map_err(|e| format!("tile_sprites.json parse: {e}"))?;
        }
        Ok(atlas)
    }

    pub fn from_parts(image: RgbaImage, list: Vec<SpriteUv>) -> Self {
        let uvs = list.into_iter().map(|u| (u.name.clone(), u)).collect();
        SpriteAtlas {
            image,
            uvs,
            tile_sprites: BTreeMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&SpriteUv> {
        self.uvs.get(name)
    }

    /// The sprite name a game id draws as: the id itself when
    /// it names a sprite, else its tile entry's fg sprite via
    /// `tile_sprites` (when that sprite is in this atlas).
    pub fn resolve<'a>(&'a self, id: &'a str) -> Option<&'a str> {
        if self.uvs.contains_key(id) {
            return Some(id);
        }
        let target = self.tile_sprites.get(id)?;
        if self.uvs.contains_key(target) {
            Some(target.as_str())
        } else {
            None
        }
    }

    /// `get` for a game id (see `resolve`).
    pub fn get_resolved(&self, id: &str) -> Option<&SpriteUv> {
        self.resolve(id).and_then(|name| self.get(name))
    }

    /// Crop one sprite's pixels out of the packed atlas image.
    pub fn sprite_image(&self, name: &str) -> Option<RgbaImage> {
        let uv = self.uvs.get(name)?;
        if uv.x + uv.width > self.image.width() || uv.y + uv.height > self.image.height() {
            return None;
        }
        Some(
            image::imageops::crop_imm(&self.image, uv.x, uv.y, uv.width, uv.height).to_image(),
        )
    }

    /// Write this atlas back out in the voxelizer's format
    /// (`sprites.json` + `sprites.png`) — used by the fixture so the
    /// load path is exercised against real files, not just memory.
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let list: Vec<&SpriteUv> = self.uvs.values().collect();
        std::fs::write(
            dir.join("sprites.json"),
            serde_json::to_string_pretty(&list).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        self.image
            .save(dir.join("sprites.png"))
            .map_err(|e| e.to_string())?;
        if !self.tile_sprites.is_empty() {
            std::fs::write(
                dir.join("tile_sprites.json"),
                serde_json::to_string_pretty(&self.tile_sprites).unwrap(),
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------
// Fixture atlas (tests + headless captures)
// ---------------------------------------------------------------------

fn put(img: &mut RgbaImage, x: u32, y: u32, c: [u8; 4]) {
    if x < img.width() && y < img.height() {
        img.put_pixel(x, y, image::Rgba(c));
    }
}

fn fill_rect(img: &mut RgbaImage, x0: u32, y0: u32, w: u32, h: u32, c: [u8; 4]) {
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            put(img, x, y, c);
        }
    }
}

/// Deterministic in-crate fixture atlas. Deliberately includes:
/// * a standard 32x32 creature (`mon_zombie`) with transparent corners,
/// * overlay layers (`overlay_jacket`, `overlay_jeans`,
///   `overlay_knife`) that only cover part of the base,
/// * an oversized sprite (`mon_hulk`, 64x64, offset (-16,-32) — the
///   UltiCa `large` geometry from plan §8),
/// * a soft-edged sprite (`mon_ghost`) whose border pixels are
///   semi-transparent over a bright color — the dark-halo probe,
/// * item sprites (`item_rock`, `item_crowbar`) and a second creature
///   (`mon_skeleton`) for the variety pass.
///
/// There is deliberately NO player sprite here (Joey's call,
/// 2026-11-24: do not invent sprites). The avatar's art is the
/// tileset's own — the `player_male` entry's fg (`skin_brown_m`
/// in UltiCa) plus its gear overlays — and it resolves through
/// `tile_sprites` in the real atlas. A fixture stand-in would
/// only ever prove the fixture.
pub fn fixture_atlas() -> SpriteAtlas {
    // Shelf-pack by hand at fixed positions (deterministic).
    // Layout (atlas 256 wide): zombie 0,0 32x32; jacket 32,0; jeans
    // 64,0; knife 96,0; skeleton 128,0; rock 160,0; crowbar 192,0;
    // ghost 224,0; hulk 0,32 64x64.
    let mut img = RgbaImage::new(256, 128);
    let mut uvs = Vec::new();
    let mut add = |uv: SpriteUv| uvs.push(uv);

    // mon_zombie: green humanoid block, transparent corners.
    fill_rect(&mut img, 4, 2, 24, 28, [70, 140, 60, 255]);
    fill_rect(&mut img, 10, 2, 12, 10, [90, 160, 80, 255]); // head
    add(SpriteUv { name: "mon_zombie".into(), x: 0, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // overlay_jacket: torso only.
    fill_rect(&mut img, 32 + 6, 12, 20, 12, [40, 60, 160, 255]);
    add(SpriteUv { name: "overlay_jacket".into(), x: 32, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // overlay_jeans: legs only.
    fill_rect(&mut img, 64 + 8, 24, 16, 8, [30, 30, 90, 255]);
    add(SpriteUv { name: "overlay_jeans".into(), x: 64, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // overlay_knife: small held item at the right side.
    fill_rect(&mut img, 96 + 26, 14, 4, 10, [200, 200, 210, 255]);
    add(SpriteUv { name: "overlay_knife".into(), x: 96, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // mon_skeleton: pale, thinner.
    fill_rect(&mut img, 128 + 8, 2, 16, 28, [220, 215, 200, 255]);
    add(SpriteUv { name: "mon_skeleton".into(), x: 128, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // item_rock / item_crowbar: small ground items.
    fill_rect(&mut img, 160 + 10, 20, 12, 8, [120, 115, 110, 255]);
    add(SpriteUv { name: "item_rock".into(), x: 160, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    fill_rect(&mut img, 192 + 4, 22, 24, 4, [150, 60, 40, 255]);
    add(SpriteUv { name: "item_crowbar".into(), x: 192, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // mon_ghost: bright white core, semi-transparent bright border.
    // If sampling/blending were done on straight (non-premultiplied)
    // colors against a dark clear, the border would fringe dark.
    fill_rect(&mut img, 224, 0, 32, 32, [255, 255, 255, 110]);
    fill_rect(&mut img, 224 + 6, 6, 20, 20, [250, 250, 255, 255]);
    add(SpriteUv { name: "mon_ghost".into(), x: 224, y: 0, width: 32, height: 32, offset_x: 0, offset_y: 0, sheet: "normal.png".into() });
    // mon_hulk: oversized 64x64, offset (-16,-32) (UltiCa `large`).
    fill_rect(&mut img, 8, 32 + 6, 48, 58, [100, 60, 140, 255]);
    fill_rect(&mut img, 20, 32, 24, 16, [120, 80, 160, 255]);
    add(SpriteUv { name: "mon_hulk".into(), x: 0, y: 32, width: 64, height: 64, offset_x: -16, offset_y: -32, sheet: "large.png".into() });
    SpriteAtlas::from_parts(img, uvs)
}
