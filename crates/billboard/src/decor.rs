//! Decoration scatter (Procedural Texture Pipeline Stage F part 2,
//! pipeline doc §10.2): purely visual grass tufts / shrubs /
//! flowering tufts standing on exposed grass and dirt tiles.
//!
//! Placement is **client-side only**: it is a pure function of the
//! world tile coordinates in the caller's ground-tile list —
//! nothing new crosses IPC, no protocol change, and the same
//! snapshot yields the identical decoration list on every client
//! and in both renderers (both consume this one function through
//! the engine's billboard pass).
//!
//! Hashing: the integer hash below is the voxelizer noise module's
//! construction verbatim in spirit and in constants — FNV-1a +
//! splitmix64 avalanche for content seeds
//! (`voxelizer::noise::seed_from_str`), splitmix64 finalizer over
//! coordinate-multiplied inputs for per-tile draws. It is
//! re-implemented here because the billboard crate is a runtime
//! crate and the voxelizer is the offline tool (the billboard
//! crate may not depend on it); the Stage F2 gate cross-checks
//! `seed_from_str` against `voxelizer::noise::seed_from_str` so
//! the two cannot silently drift.
//!
//! Density modulation (§10.2): where the caller supplies a macro
//! height sample for the tile's material (the Stage D macro height
//! layer, sampled on the CPU at the tile's world position in the
//! macro's span — see `engine::frame::sample_scene_ground_tiles`
//! for the engine-side contract), the sample scales both draw
//! probabilities, so decorations clump on high ground instead of
//! sprinkling uniformly. Where a material has **no** macro (v1
//! assets, or a v2 material without a pooled family), the caller
//! passes `None` and the scatter uses the documented fallback
//! [`FALLBACK_HEIGHT`] — the neutral mid-height, i.e. the average
//! density — never zero and never maximum.
//!
//! Exclusions (§10.2): floors, water and under-roof tiles get no
//! decorations, by kind/flag check in [`scatter`], and no other
//! ground kind is eligible. Decorations are visual only: they are
//! never registered for picking (the engine's picker iterates
//! entity instances only; decoration instances carry the reserved
//! [`DECORATION_ENTITY_ID`] and live in their own list), and they
//! touch no collision / LOS / gameplay state.

use glam::Vec3;

use crate::atlas::SpriteAtlas;
use crate::instance::{tile_feet, BillboardInstance, TILE_PIXEL_SIZE};

/// Decoration sprite names, in variant-index order. Must agree
/// with `voxelizer::decor::DECOR_NAMES` (the generation side);
/// the Stage F2 gate asserts it.
pub const DECOR_SPRITE_NAMES: [&str; 3] = ["decor_tuft", "decor_shrub", "decor_flowering"];

/// Reserved entity id carried by every decoration instance. Real
/// entity ids come from the game and never take this value;
/// decoration instances are drawn, never picked — the id exists
/// so a decoration that somehow reached the picker is recognizable
/// (and the gate asserts the engine picker never returns it).
pub const DECORATION_ENTITY_ID: u64 = u64::MAX;

/// Macro-height fallback for materials without a macro height
/// layer: the neutral mid-height (see the module docs).
pub const FALLBACK_HEIGHT: f32 = 0.5;

/// What a ground tile's surface is, for scatter eligibility. The
/// caller derives this from its own world representation (game
/// tile ids in the live path; the engine's fake-data tile ids for
/// the sample scene).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundKind {
    Grass,
    Dirt,
    Floor,
    Water,
    /// Walls, foliage, furniture footprints, anything else.
    Other,
}

/// One ground tile as the scatter sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundTile {
    /// Game tile coordinates (x east, y south, z-level).
    pub pos: [i32; 3],
    pub kind: GroundKind,
    /// A roof (or any cover) exists over this tile.
    pub under_roof: bool,
    /// Stage D macro height sample in `[0, 1]` for this tile's
    /// material, or `None` when the material has no macro (the
    /// [`FALLBACK_HEIGHT`] density then applies).
    pub macro_height: Option<f32>,
}

impl GroundTile {
    pub fn new(pos: [i32; 3], kind: GroundKind) -> Self {
        GroundTile {
            pos,
            kind,
            under_roof: false,
            macro_height: None,
        }
    }

    pub fn under_roof(mut self, yes: bool) -> Self {
        self.under_roof = yes;
        self
    }

    pub fn macro_height(mut self, h: Option<f32>) -> Self {
        self.macro_height = h;
        self
    }

    /// Eligible for scatter: exposed grass or dirt, nothing else.
    pub fn eligible(&self) -> bool {
        matches!(self.kind, GroundKind::Grass | GroundKind::Dirt) && !self.under_roof
    }
}

/// One placed decoration: which tile, which variant sprite, and
/// the in-tile jitter (tile units, `[-0.3, 0.3]` per axis).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decoration {
    pub tile: [i32; 3],
    /// Index into [`DECOR_SPRITE_NAMES`] (0..3).
    pub variant: u8,
    /// In-tile offset from the tile center, tile units.
    pub jitter: [f32; 2],
    /// Which of the tile's (up to) two decorations this is.
    pub slot: u8,
}

// ---------------------------------------------------------------------
// Integer hashing (mirrors voxelizer::noise — see module docs)
// ---------------------------------------------------------------------

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

fn splitmix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// FNV-1a 64 + splitmix64 avalanche — identical to
/// `voxelizer::noise::seed_from_str`.
pub fn seed_from_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    splitmix64(h)
}

/// Hash one tile + a draw salt. Coordinates go through their bit
/// patterns (`as u32`) so negative world coordinates hash exactly
/// like the noise module's wrapped lattice coordinates.
fn tile_hash(seed: u64, pos: [i32; 3], salt: u64) -> u64 {
    let z = seed
        ^ (pos[0] as u32 as u64).wrapping_mul(GOLDEN)
        ^ (pos[1] as u32 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (pos[2] as u32 as u64).wrapping_mul(0x1656_6679_19E3_779F)
        ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    splitmix64(z)
}

/// Top 24 bits of a hash as a float in `[0, 1)` (the noise
/// module's `unit_from_hash` — exact in f32).
fn unit(h: u64) -> f32 {
    ((h >> 40) as f32) * (1.0 / 16_777_216.0)
}

/// Uniform integer in `[0, n)` from a hash (multiply-high, the
/// noise module `Rng::below` construction).
fn below(h: u64, n: u32) -> u32 {
    (((h >> 32) * (n as u64)) >> 32) as u32
}

// ---------------------------------------------------------------------
// Scatter
// ---------------------------------------------------------------------

/// The scatter seed: one fixed content string for the whole
/// system, so a given world always decorates identically.
fn scatter_seed() -> u64 {
    seed_from_str("decor-scatter-v1")
}

/// Draw probabilities for a tile at macro height `h` (already
/// resolved through the fallback): `(p_first, p_second)`. Both
/// rise with height, which is what makes decorations clump on
/// high macro ground (expected count at h=0.1 is 0.32/tile, at
/// h=0.9 it is 1.28/tile).
fn density_probs(h: f32) -> (f32, f32) {
    (0.15 + 0.75 * h, 0.05 + 0.45 * h)
}

/// Scatter decorations over `tiles`. Pure and deterministic: the
/// output depends only on the tiles' coordinates/kinds/heights,
/// not on input order (tiles are sorted first) or on any RNG
/// state. Every eligible tile yields 0, 1 or 2 decorations.
pub fn scatter(tiles: &[GroundTile]) -> Vec<Decoration> {
    let seed = scatter_seed();
    let mut sorted: Vec<&GroundTile> = tiles.iter().collect();
    // Total order (position, then kind / roof / height), so the
    // output is input-order independent even if a caller passes
    // two entries for one position.
    let rank = |k: GroundKind| match k {
        GroundKind::Grass => 0u8,
        GroundKind::Dirt => 1,
        GroundKind::Floor => 2,
        GroundKind::Water => 3,
        GroundKind::Other => 4,
    };
    sorted.sort_by_key(|t| {
        (
            t.pos,
            rank(t.kind),
            t.under_roof,
            t.macro_height.map(f32::to_bits),
        )
    });
    let mut out = Vec::new();
    for tile in sorted {
        if !tile.eligible() {
            continue;
        }
        let h = tile
            .macro_height
            .map(|v| v.clamp(0.0, 1.0))
            .unwrap_or(FALLBACK_HEIGHT);
        let (p_first, p_second) = density_probs(h);
        if unit(tile_hash(seed, tile.pos, 1)) >= p_first {
            continue;
        }
        for slot in 0..2u8 {
            if slot == 1 && unit(tile_hash(seed, tile.pos, 2)) >= p_second {
                break;
            }
            let variant = below(tile_hash(seed, tile.pos, 16 + slot as u64), 3) as u8;
            let jitter = [
                (unit(tile_hash(seed, tile.pos, 32 + 2 * slot as u64)) - 0.5) * 0.6,
                (unit(tile_hash(seed, tile.pos, 33 + 2 * slot as u64)) - 0.5) * 0.6,
            ];
            out.push(Decoration {
                tile: tile.pos,
                variant,
                jitter,
                slot,
            });
        }
    }
    out
}

// ---------------------------------------------------------------------
// Instances (drawing only — never picking)
// ---------------------------------------------------------------------

/// Build billboard instances for decorations, addressed into an
/// atlas that carries the decoration sprites (the voxelizer's
/// decoration atlas, or a scene atlas merged with it). Decorations
/// whose sprite is absent from the atlas are skipped. Instances
/// are sorted back-to-front like entity instances; they carry
/// [`DECORATION_ENTITY_ID`] and no shadow (a tuft's blob shadow
/// would read as a creature's).
pub fn decoration_instances(
    decorations: &[Decoration],
    atlas: &SpriteAtlas,
    eye: Vec3,
) -> Vec<BillboardInstance> {
    let seed = scatter_seed();
    let mut out = Vec::with_capacity(decorations.len());
    for d in decorations {
        let name = DECOR_SPRITE_NAMES[d.variant as usize % DECOR_SPRITE_NAMES.len()];
        let Some(uv) = atlas.get(name) else {
            continue;
        };
        let mut anchor = tile_feet(d.tile);
        anchor.x += d.jitter[0];
        anchor.z -= d.jitter[1]; // tile +y (south) is world -z
        let w = uv.width as f32 / TILE_PIXEL_SIZE;
        let h = uv.height as f32 / TILE_PIXEL_SIZE;
        let center_offset =
            ((uv.offset_x as f32 + uv.width as f32 / 2.0) - TILE_PIXEL_SIZE / 2.0) / TILE_PIXEL_SIZE;
        let bottom = (TILE_PIXEL_SIZE - (uv.offset_y as f32 + uv.height as f32)) / TILE_PIXEL_SIZE;
        out.push(BillboardInstance {
            entity_id: DECORATION_ENTITY_ID,
            anchor,
            center_offset,
            bottom,
            size: glam::Vec2::new(w, h),
            uv_rect: uv.uv_rect(atlas.image.width(), atlas.image.height()),
            tint: [1.0, 1.0, 1.0, 0.0],
            mirrored: tile_hash(seed, d.tile, 48 + d.slot as u64) & 1 == 1,
            depth: anchor.distance(eye),
        });
    }
    crate::instance::sort_back_to_front(&mut out);
    out
}
