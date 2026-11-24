//! Procedural texture generation (Phase 5 rework, Stage C): turn a
//! sprite family into a clean, tileable 32x32 block texture.
//!
//! The sprite is the *source*, not the texture: [`analyze_family`]
//! extracts its palette, the palette is de-shaded (below), and a
//! recipe re-renders the material from periodic noise (see
//! [`crate::noise`]) seeded by the family's content hash. Same
//! family in -> byte-identical texture out, always (project
//! determinism rule).
//!
//! De-shading: UltiCa sprites carry painted light (vertical
//! gradients, per-plank jitter, specular paint — see
//! `docs/sprite-anatomy.md`). A generated texture is albedo: the
//! engine lights the scene itself, so baked light in the palette
//! would double-shade. [`deshade_palette`] compresses each
//! cluster's OKLab lightness toward the palette's weight-weighted
//! median lightness by the family's measured shading factor
//! (Stage B [`ShadingInfo::shading_factor`]): a family measured as
//! unshaded keeps its palette untouched; a fully shaded scene
//! illustration collapses to (near) flat albedo lightness while
//! keeping hue/chroma relationships.
//!
//! Family pooling: [`family_sprites`] collects a tileset entry's
//! own fg sprites plus its `center` and `unconnected` subtile
//! sprites, composited over the entry bg. Edge/corner/t-connection
//! subtiles are transition art (grass fringes baked into dirt
//! edges, anatomy SS2) and are deliberately excluded; ground
//! recipes additionally sample sprite centers only (see
//! [`generate`]).

use std::collections::BTreeSet;

use image::{Rgba, RgbaImage};
use tileset::{SpriteRef, TileEntry, Tileset};

use crate::analyze::{analyze_family, content_hash, PaletteColor, SpriteAnalysis};
use crate::classify::MaterialClass;
use crate::color::Oklab;
use crate::faces::composite_over;
use crate::load_sprite_image;
use crate::noise::{seed_from_str, white_noise, Fbm, Worley};

/// Generated textures are 32x32 (plan decision: 32 px stays for v1).
pub const TEXTURE_SIZE: u32 = 32;

/// Palette clusters extracted per family (plan SS8 budget: <= 24).
const PALETTE_K: usize = 10;

/// Which material recipe to run. The recipe is chosen by the
/// caller (Stage B's classifier class -> recipe mapping lives with
/// the caller for now); the four Stage C recipes are the materials
/// that dominate the sample scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeKind {
    Grass,
    Dirt,
    WoodFloor,
    Plaster,
    Concrete,
    Brick,
    Stone,
    Metal,
    Glass,
    Water,
    Road,
    Shingle,
}

impl RecipeKind {
    pub fn name(self) -> &'static str {
        match self {
            RecipeKind::Grass => "grass",
            RecipeKind::Dirt => "dirt",
            RecipeKind::WoodFloor => "wood_floor",
            RecipeKind::Plaster => "plaster",
            RecipeKind::Concrete => "concrete",
            RecipeKind::Brick => "brick",
            RecipeKind::Stone => "stone",
            RecipeKind::Metal => "metal",
            RecipeKind::Glass => "glass",
            RecipeKind::Water => "water",
            RecipeKind::Road => "road",
            RecipeKind::Shingle => "shingle",
        }
    }

    /// Ground materials sample sprite centers (anatomy SS2:
    /// transition fringes live at sprite edges). Water and road
    /// are ground surfaces in the same sense: their sprites are
    /// full-bleed fields whose edges carry transition art.
    pub fn is_ground(self) -> bool {
        matches!(
            self,
            RecipeKind::Grass | RecipeKind::Dirt | RecipeKind::Water | RecipeKind::Road
        )
    }
}

/// One generated texture plus the evidence chain behind it.
#[derive(Debug, Clone)]
pub struct GeneratedTexture {
    /// The 32x32 tileable texture (fully opaque).
    pub image: RgbaImage,
    /// The de-shaded palette the texture was rendered from.
    pub palette: Vec<PaletteColor>,
    /// The family analysis the palette came from (pre-de-shade).
    pub analysis: SpriteAnalysis,
}

/// Pool a tileset entry's exemplar sprites: its own fg leaves plus
/// the `center` and `unconnected` subtiles' fg leaves, each
/// composited over the (sub)entry's bg. Sorted by sprite name,
/// deduplicated — deterministic for a given tileset. Transition
/// subtiles (edge/corner/t_connection/end_piece) are excluded:
/// they are border art between two materials, not exemplars of
/// either (anatomy SS1/SS2).
pub fn family_sprites(ts: &Tileset, entry: &TileEntry) -> Vec<RgbaImage> {
    let mut out = Vec::new();
    collect_family(ts, entry, true, &mut out);
    out
}

fn collect_family(ts: &Tileset, entry: &TileEntry, root: bool, out: &mut Vec<RgbaImage>) {
    let bg = entry
        .bg
        .as_ref()
        .and_then(|r| load_sprite_image(ts, entry, r));
    let mut names = Vec::new();
    if let Some(fg) = &entry.fg {
        fg.all_names(&mut names);
    }
    let names: BTreeSet<String> = names.into_iter().collect();
    if names.is_empty() {
        if let Some(img) = crate::composite_entry(ts, entry) {
            out.push(img);
        }
    } else {
        for name in names {
            let r = SpriteRef::Name(name);
            if let Some(fg_img) = load_sprite_image(ts, entry, &r) {
                out.push(match &bg {
                    Some(b) => composite_over(b, &fg_img),
                    None => fg_img,
                });
            }
        }
    }
    if root {
        for (sub_id, sub) in &entry.additional_tiles {
            if sub_id == "center" || sub_id == "unconnected" {
                collect_family(ts, sub, false, out);
            }
        }
    }
}

/// Deterministic content hash of a sprite family (order matters:
/// callers pass the sorted pool from [`family_sprites`]).
pub fn family_seed(images: &[RgbaImage]) -> u64 {
    let mut buf = Vec::new();
    for img in images {
        buf.extend_from_slice(&img.width().to_le_bytes());
        buf.extend_from_slice(&img.height().to_le_bytes());
        buf.extend_from_slice(img.as_raw());
    }
    content_hash(&buf)
}

/// Remove baked lightness from a palette: compress every cluster's
/// OKLab L toward the palette's weight-weighted median L by
/// `shading_factor` (0 = untouched, 1 = all clusters at the median
/// L). Hue/chroma (a, b) and weights are preserved; `srgb` is
/// recomputed from the shifted color.
pub fn deshade_palette(palette: &[PaletteColor], shading_factor: f32) -> Vec<PaletteColor> {
    if palette.is_empty() {
        return Vec::new();
    }
    let keep = 1.0 - shading_factor.clamp(0.0, 1.0);
    let median = weighted_median_l(palette);
    palette
        .iter()
        .map(|c| {
            let lab = Oklab {
                l: median + (c.lab.l - median) * keep,
                a: c.lab.a,
                b: c.lab.b,
            };
            PaletteColor {
                lab,
                srgb: lab.to_srgb_u8(),
                weight: c.weight,
                count: c.count,
            }
        })
        .collect()
}

/// Weight-weighted median of cluster lightness values.
fn weighted_median_l(palette: &[PaletteColor]) -> f32 {
    let mut ls: Vec<(f32, f32)> = palette.iter().map(|c| (c.lab.l, c.weight)).collect();
    ls.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let total: f32 = ls.iter().map(|x| x.1).sum();
    let mut acc = 0.0f32;
    for (l, w) in &ls {
        acc += w;
        if acc >= total * 0.5 {
            return *l;
        }
    }
    ls.last().map(|x| x.0).unwrap_or(0.0)
}

/// Palette as an L-ordered color ramp: `at(t)` walks the clusters
/// by cumulative weight, interpolating in OKLab, so a uniform `t`
/// field reproduces the palette's own color distribution.
struct Ramp {
    labs: Vec<Oklab>,
    /// Cumulative weight at each cluster's far edge (last == 1).
    cum: Vec<f32>,
}

impl Ramp {
    fn new(palette: &[PaletteColor]) -> Ramp {
        let mut sorted: Vec<&PaletteColor> = palette.iter().collect();
        sorted.sort_by(|a, b| {
            a.lab
                .l
                .partial_cmp(&b.lab.l)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.lab.a.partial_cmp(&b.lab.a).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.lab.b.partial_cmp(&b.lab.b).unwrap_or(std::cmp::Ordering::Equal))
        });
        let total: f32 = sorted.iter().map(|c| c.weight).sum();
        let total = if total > 0.0 { total } else { 1.0 };
        let mut cum = Vec::with_capacity(sorted.len());
        let mut acc = 0.0f32;
        for c in &sorted {
            acc += c.weight / total;
            cum.push(acc);
        }
        if let Some(last) = cum.last_mut() {
            *last = 1.0;
        }
        Ramp {
            labs: sorted.iter().map(|c| c.lab).collect(),
            cum,
        }
    }

    fn at(&self, t: f32) -> Oklab {
        if self.labs.len() == 1 {
            return self.labs[0];
        }
        let t = t.clamp(0.0, 1.0);
        // Segment i covers (cum[i-1], cum[i]]; interpolate within it.
        let mut i = 0usize;
        while i + 1 < self.labs.len() && t > self.cum[i] {
            i += 1;
        }
        if i == 0 {
            // Within the darkest cluster's own weight band the
            // nearest palette color is that cluster itself.
            return self.labs[0];
        }
        let lo = self.cum[i - 1];
        let hi = self.cum[i];
        let f = if hi > lo { (t - lo) / (hi - lo) } else { 0.0 };
        lerp_lab(self.labs[i - 1], self.labs[i], f.clamp(0.0, 1.0))
    }
}

fn lerp_lab(a: Oklab, b: Oklab, f: f32) -> Oklab {
    Oklab {
        l: a.l + (b.l - a.l) * f,
        a: a.a + (b.a - a.a) * f,
        b: a.b + (b.b - a.b) * f,
    }
}

/// Center 50% crop (anatomy SS2 ground rule). Images smaller than
/// 4 px on an axis are returned whole.
fn center_crop(img: &RgbaImage) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    if w < 4 || h < 4 {
        return img.clone();
    }
    image::imageops::crop_imm(img, w / 4, h / 4, w / 2, h / 2).to_image()
}

/// Generate one material texture from its sprite family.
///
/// Ground families (grass, dirt) are analyzed on center crops
/// only; wall/floor families on the full sprites. The palette is
/// de-shaded by the family's measured shading factor before the
/// recipe renders from it.
pub fn generate(kind: RecipeKind, family: &[RgbaImage]) -> GeneratedTexture {
    assert!(!family.is_empty(), "generate: empty family for {}", kind.name());
    let analysis = if kind.is_ground() {
        let crops: Vec<RgbaImage> = family.iter().map(center_crop).collect();
        let refs: Vec<&RgbaImage> = crops.iter().collect();
        analyze_family(&refs, PALETTE_K)
    } else {
        let refs: Vec<&RgbaImage> = family.iter().collect();
        analyze_family(&refs, PALETTE_K)
    };
    let palette = deshade_palette(&analysis.palette, analysis.shading.shading_factor);
    let seed = family_seed(family) ^ seed_from_str(kind.name());
    let image = render(kind, &palette, &analysis, seed);
    GeneratedTexture {
        image,
        palette,
        analysis,
    }
}

/// Stage F part 2 — generated wall end-cap (top face) for one wall
/// family.
///
/// A wall sprite is a face-on illustration of the wall's SIDE
/// (docs/sprite-anatomy.md §4): it contains no top-face pixels, so
/// the cap is GENERATED from the family's own palette, never
/// sampled from the sprite. The palette pipeline is the Stage C
/// one — analyze the family, de-shade by the measured shading
/// factor (per-sprite direction: t_concrete_wall gradients
/// light-at-bottom, others the reverse; the factor is measured,
/// never assumed) — and the render is albedo only: periodic noise
/// and structural albedo pattern, no directional light term (the
/// engine provides lighting).
///
/// The structure is what a wall's top actually is, per class:
/// plaster/concrete caps are the material's cross-section surface
/// (near-flat field, pore grain for concrete); a brick cap is
/// plan-view brick tops (coarser rows than the face courses);
/// a wood cap is the top plate — boards running along the wall
/// with grain along the board. Classes without a cap-specific
/// structure fall back to their Stage C/E surface render (stone
/// cells, metal panels, glass) or a generic mottled cross-section.
pub fn generate_wall_cap(class: MaterialClass, family: &[RgbaImage]) -> GeneratedTexture {
    assert!(!family.is_empty(), "generate_wall_cap: empty family for {}", class.name());
    let refs: Vec<&RgbaImage> = family.iter().collect();
    let analysis = analyze_family(&refs, PALETTE_K);
    let palette = deshade_palette(&analysis.palette, analysis.shading.shading_factor);
    let seed = family_seed(family) ^ seed_from_str("wall_cap") ^ seed_from_str(class.name());
    let ramp = Ramp::new(&palette);
    let mut image = RgbaImage::new(TEXTURE_SIZE, TEXTURE_SIZE);
    match class {
        MaterialClass::Plaster => render_plaster(&mut image, &ramp, seed),
        MaterialClass::Concrete => render_concrete(&mut image, &ramp, seed),
        MaterialClass::Brick => render_brick_cap(&mut image, &ramp, seed),
        MaterialClass::Wood | MaterialClass::WoodFloor => render_wood_cap(&mut image, &ramp, seed),
        MaterialClass::Stone => render_stone(&mut image, &ramp, seed),
        MaterialClass::Metal => render_metal(&mut image, &ramp, seed),
        MaterialClass::Glass => render_glass(&mut image, &ramp, seed),
        _ => render_generic_cap(&mut image, &ramp, seed),
    }
    GeneratedTexture {
        image,
        palette,
        analysis,
    }
}

/// Brick cap: plan-view brick tops. Rows are 8 px (vs the 4 px
/// face courses of [`render_brick`]) — from above, a brick's top
/// is its 65 mm width, not its course height — lengths 16 px,
/// odd rows offset half a brick, both phases shifted so wrap
/// edges fall mid-brick and mid-row. Mortar takes the ramp's
/// light end, as in the face render.
fn render_brick_cap(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 16).octaves(2);
    let brick_seed = seed ^ 0x2545_F491_4F6C_DD1D;
    let row_h: u32 = 8;
    let brick_len: u32 = 16;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let sy = (y + row_h / 2) % TEXTURE_SIZE;
            let row = sy / row_h;
            let row_off = if row % 2 == 1 { brick_len / 2 } else { 0 };
            // Base shift 4 (not brick_len / 2): with a half-brick
            // row offset, an 8 px base would put the odd rows'
            // vertical mortar joint exactly on the wrap edge
            // (measured seam 1.244); 4 px keeps every joint in
            // the tile interior and the wrap columns in the same
            // brick on both sides.
            let sx = (x + 4 + row_off) % TEXTURE_SIZE;
            if sy % row_h == 0 || sx % brick_len == 0 {
                put(img, x, y, ramp.at(0.93));
                continue;
            }
            let brick_id = sx / brick_len + row * (TEXTURE_SIZE / brick_len);
            let brick_off = (white_noise(brick_seed, brick_id, row) - 0.5) * 0.34;
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.48 + brick_off + (grain.sample_uv(u, v) - 0.5) * 0.14;
            put(img, x, y, ramp.at(t.clamp(0.0, 1.0)));
        }
    }
}

/// Wood cap: the wall's top plate — boards running along the
/// wall (x), stacked across its thickness (y). Board width 8 px,
/// phase shifted half a board so the wrap edge falls mid-board.
/// Grain is noise stretched along the board (the wood-floor
/// idiom); each board takes a deterministic ramp offset; the
/// first row of each board is its seam.
fn render_wood_cap(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 8).octaves(3);
    let board_seed = seed ^ 0x2545_F491_4F6C_DD1D;
    let board_w: u32 = 8;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let sy = (y + board_w / 2) % TEXTURE_SIZE;
            let board = sy / board_w;
            let board_off = (white_noise(board_seed, board, 0) - 0.5) * 0.40;
            let g = grain.sample(x as f32 * 0.25, y as f32);
            let t = (0.5 + board_off + (g - 0.5) * 0.3).clamp(0.0, 1.0);
            let mut lab = ramp.at(t);
            let within = sy % board_w;
            if within == 0 {
                lab.l = (lab.l - 0.085).max(0.0);
            } else if within == 1 {
                lab.l = (lab.l + 0.018).min(1.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Generic cap: a mottled cross-section of the wall material —
/// broad + fine fBm through a mid ramp band, no pattern at tile
/// scale. For wall classes with no cap-specific structure.
fn render_generic_cap(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let broad = Fbm::new(seed, 4).octaves(2);
    let fine = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 12).octaves(2);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.5 + (broad.sample_uv(u, v) - 0.5) * 0.24 + (fine.sample_uv(u, v) - 0.5) * 0.10;
            put(img, x, y, ramp.at(t.clamp(0.0, 1.0)));
        }
    }
}

fn render(kind: RecipeKind, palette: &[PaletteColor], analysis: &SpriteAnalysis, seed: u64) -> RgbaImage {
    let ramp = Ramp::new(palette);
    let mut img = RgbaImage::new(TEXTURE_SIZE, TEXTURE_SIZE);
    match kind {
        RecipeKind::Grass => render_grass(&mut img, &ramp, seed),
        RecipeKind::Dirt => render_dirt(&mut img, &ramp, seed),
        RecipeKind::WoodFloor => render_wood_floor(&mut img, &ramp, analysis, seed),
        RecipeKind::Plaster => render_plaster(&mut img, &ramp, seed),
        RecipeKind::Concrete => render_concrete(&mut img, &ramp, seed),
        RecipeKind::Brick => render_brick(&mut img, &ramp, seed),
        RecipeKind::Stone => render_stone(&mut img, &ramp, seed),
        RecipeKind::Metal => render_metal(&mut img, &ramp, seed),
        RecipeKind::Glass => render_glass(&mut img, &ramp, seed),
        RecipeKind::Water => render_water(&mut img, &ramp, seed),
        RecipeKind::Road => render_road(&mut img, &ramp, seed),
        RecipeKind::Shingle => render_shingle(&mut img, &ramp, seed),
    }
    img
}

fn put(img: &mut RgbaImage, x: u32, y: u32, lab: Oklab) {
    let [r, g, b] = lab.to_srgb_u8();
    img.put_pixel(x, y, Rgba([r, g, b, 255]));
}

/// Grass: two-scale fBm mottling through the palette ramp, plus
/// per-pixel blade speckle (deterministic white noise lightening /
/// darkening individual pixels — the source texture's grain).
fn render_grass(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let broad = Fbm::new(seed, 6).octaves(3);
    let fine = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 16).octaves(2);
    let speckle_seed = seed ^ 0xD1B5_4A32_D192_ED03;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.62 * broad.sample_uv(u, v) + 0.38 * fine.sample_uv(u, v);
            let t = (0.5 + (t - 0.5) * 1.15).clamp(0.0, 1.0);
            let mut lab = ramp.at(t);
            let w = white_noise(speckle_seed, x, y);
            if w > 0.94 {
                lab.l = (lab.l + 0.05).min(1.0);
            } else if w < 0.06 {
                lab.l = (lab.l - 0.05).max(0.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Dirt: broad fBm blotches with a finer grain octave, plus sparse
/// pebbles — pixels very close to a Worley feature point inside
/// blotch patches take a light ramp color (stones sitting in the
/// dirt, not a pattern).
fn render_dirt(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let blotch = Fbm::new(seed, 4).octaves(3);
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 12).octaves(2);
    let worley = Worley::new(seed ^ 0x2545_F491_4F6C_DD1D, 9);
    let pebble_seed = seed ^ 0xD1B5_4A32_D192_ED03;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let b = blotch.sample_uv(u, v);
            let t = 0.72 * b + 0.28 * grain.sample_uv(u, v);
            let t = (0.5 + (t - 0.5) * 1.1).clamp(0.0, 1.0);
            let mut lab = ramp.at(t);
            let ws = worley.sample_uv(u, v);
            if ws.f1 < 0.24 && b > 0.5 {
                // Pebble: light end of the ramp, slight per-pixel
                // value jitter so stones are not flat discs.
                lab = ramp.at(0.88);
                lab.l = (lab.l + (white_noise(pebble_seed, x, y) - 0.5) * 0.04).clamp(0.0, 1.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Wood floor: planks. The family's dominant orientation picks the
/// plank direction (t_floor reads Vertical: planks run top-bottom,
/// so plank strips stack along x). Each plank takes a deterministic
/// ramp offset (the source's per-plank value jitter, kept as albedo
/// variation rather than light), grain streaks run along the plank
/// (noise stretched along the plank axis), and the first pixel
/// column of each plank is a dark seam. Plank indices wrap modulo
/// the plank count, and the plank phase is shifted by half a plank
/// so the tile border falls mid-plank: the wrap edge is continuous
/// wood, exactly like crossing a tile boundary on a real floor —
/// the strong seam lines sit in the tile interior.
fn render_wood_floor(img: &mut RgbaImage, ramp: &Ramp, analysis: &SpriteAnalysis, seed: u64) {
    use crate::analyze::Orientation;
    let vertical = !matches!(analysis.orientation.kind, Orientation::Horizontal);
    let planks: u32 = 4;
    let pw = TEXTURE_SIZE / planks; // 8 px per plank
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 8).octaves(3);
    let plank_seed = seed ^ 0x2545_F491_4F6C_DD1D;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let across = if vertical { x } else { y };
            // Half-plank phase shift: borders fall mid-plank.
            let shifted = (across + pw / 2) % TEXTURE_SIZE;
            let plank = (shifted / pw) % planks;
            let plank_off = (white_noise(plank_seed, plank, 0) - 0.5) * 0.44;
            // Stretched grain: fast variation across the plank,
            // slow along it. Dyadic scales keep the field periodic
            // over the tile (see noise.rs Fbm notes).
            let g = if vertical {
                grain.sample(x as f32, y as f32 * 0.25)
            } else {
                grain.sample(x as f32 * 0.25, y as f32)
            };
            let t = (0.5 + plank_off + (g - 0.5) * 0.3).clamp(0.0, 1.0);
            let mut lab = ramp.at(t);
            let within = shifted % pw;
            if within == 0 {
                lab.l = (lab.l - 0.085).max(0.0); // seam shadow
            } else if within == 1 {
                lab.l = (lab.l + 0.018).min(1.0); // seam lip highlight
            }
            put(img, x, y, lab);
        }
    }
}

/// Plaster: near-flat warm field. The ramp is walked only in a
/// narrow band around its middle, so the texture reads as one
/// material with a whisper of large-scale unevenness — no pattern
/// at tile scale.
fn render_plaster(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let broad = Fbm::new(seed, 4).octaves(2);
    let fine = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 8).octaves(2);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.5 + (broad.sample_uv(u, v) - 0.5) * 0.16 + (fine.sample_uv(u, v) - 0.5) * 0.06;
            put(img, x, y, ramp.at(t.clamp(0.0, 1.0)));
        }
    }
}

/// Concrete: fine Worley grain (small cells, ~1.3 px) over a
/// broad fBm unevenness, walked in a narrow ramp band — cast
/// concrete is one material with pores and faint trowel patches,
/// no pattern at tile scale. Sparse dark pits (air holes) via
/// white noise.
fn render_concrete(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let broad = Fbm::new(seed, 4).octaves(2);
    let cells = Worley::new(seed ^ 0x9E37_79B9_7F4A_7C15, 24);
    let pit_seed = seed ^ 0xD1B5_4A32_D192_ED03;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let ws = cells.sample_uv(u, v);
            let t = 0.5 + (broad.sample_uv(u, v) - 0.5) * 0.22 + (ws.f1 - 0.45) * 0.16;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            if white_noise(pit_seed, x, y) < 0.03 {
                lab.l = (lab.l - 0.05).max(0.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Brick: running-bond courses. Course height 4 px, brick length
/// 16 px, odd courses offset half a brick. Both phases are shifted
/// (x + 8, y + 2) so the wrap edges fall mid-brick and mid-course —
/// the Stage C wood-floor lesson: border-aligned mortar lines are
/// what blew the seam budget there. Mortar takes the ramp's light
/// end; each brick gets a deterministic ramp offset plus fine
/// grain, so courses read as individual fired bricks.
fn render_brick(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 16).octaves(2);
    let brick_seed = seed ^ 0x2545_F491_4F6C_DD1D;
    let course_h: u32 = 4;
    let brick_len: u32 = 16;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let sy = (y + course_h / 2) % TEXTURE_SIZE;
            let course = sy / course_h;
            let row_off = if course % 2 == 1 { brick_len / 2 } else { 0 };
            let sx = (x + brick_len / 2 + row_off) % TEXTURE_SIZE;
            let within_y = sy % course_h;
            let within_x = sx % brick_len;
            if within_y == 0 || within_x == 0 {
                // Mortar: horizontal course line + vertical joints.
                put(img, x, y, ramp.at(0.93));
                continue;
            }
            let brick_id = sx / brick_len + course * (TEXTURE_SIZE / brick_len);
            let brick_off = (white_noise(brick_seed, brick_id, course) - 0.5) * 0.34;
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.48 + brick_off + (grain.sample_uv(u, v) - 0.5) * 0.14;
            put(img, x, y, ramp.at(t.clamp(0.0, 1.0)));
        }
    }
}

/// Stone: large Worley cells (~6 px) as individual stones — the
/// F2-F1 signal draws the dark seams between them, and F1 shades
/// each stone radially (lit toward its feature point, darker at
/// the rim), which is how the source's rubble/rock walls read.
fn render_stone(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let cells = Worley::new(seed, 5);
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 12).octaves(2);
    let jitter_seed = seed ^ 0xD1B5_4A32_D192_ED03;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let ws = cells.sample_uv(u, v);
            let t = 0.30 + ws.f1 * 0.38 + (grain.sample_uv(u, v) - 0.5) * 0.20;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            // Anti-aliased seam: blend toward the seam color as
            // F2-F1 closes, instead of a hard 1 px line — the
            // source's mortar joints are soft-edged too, and a
            // hard line makes wrap-edge pixel pairs cliff-edge
            // outliers against the smooth cell interiors.
            let seam_d = ws.f2_minus_f1();
            if seam_d < 0.14 {
                let f = (1.0 - seam_d / 0.14) * 0.7;
                lab = lerp_lab(lab, ramp.at(0.18), f);
            }
            // Per-pixel mineral jitter (the grass-speckle idiom):
            // stone is grainy at pixel scale, and a uniform step
            // floor keeps adjacent-pixel statistics stationary
            // across the wrap border.
            lab.l = (lab.l + (white_noise(jitter_seed, x, y) - 0.5) * 0.03).clamp(0.0, 1.0);
            put(img, x, y, lab);
        }
    }
}

/// Metal: brushed panels. The brush grain is noise stretched
/// along x (one period across the tile horizontally, an 8 px
/// period vertically — the wood-floor stretched-sample idiom), so
/// streaks run horizontally. Panels are 16x16 with the phase
/// shifted by 8 on both axes: seams (dark line + lip highlight)
/// and corner rivets sit in the tile interior, never on the wrap.
fn render_metal(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let brush = Fbm::new(seed, 8).octaves(3);
    let panel: u32 = 16;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let sx = (x + panel / 2) % TEXTURE_SIZE;
            let sy = (y + panel / 2) % TEXTURE_SIZE;
            let g = brush.sample(x as f32 * 0.25, y as f32);
            let t = 0.5 + (g - 0.5) * 0.30;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            let (wx, wy) = (sx % panel, sy % panel);
            if wx == 0 || wy == 0 {
                lab.l = (lab.l - 0.075).max(0.0); // panel seam shadow
            } else if wx == 1 || wy == 1 {
                lab.l = (lab.l + 0.016).min(1.0); // seam lip
            }
            if (wx == 2 || wx == panel - 3) && (wy == 2 || wy == panel - 3) {
                lab.l = (lab.l + 0.055).min(1.0); // rivet heads
            }
            put(img, x, y, lab);
        }
    }
}

/// Glass: near-flat by design — the material is a tinted pane.
/// A faint diagonal specular streak crosses the pane (anatomy:
/// UltiCa glass sprites carry painted diagonal streaks); the
/// streak coordinate `(x + y) mod 32` is continuous across both
/// wrap edges, so the streak tiles seamlessly.
fn render_glass(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let broad = Fbm::new(seed, 4).octaves(2);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.5 + (broad.sample_uv(u, v) - 0.5) * 0.05;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            let d = (x + y) % TEXTURE_SIZE;
            if (10..=12).contains(&d) {
                lab = ramp.at(0.85);
            } else if d == 9 || d == 13 {
                lab.l = (lab.l + 0.03).min(1.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Water: two fBm layers at different scales drifting against
/// each other — broad swells plus a finer ripple — walked in a
/// mid ramp band. No hard features: open water has no pattern at
/// tile scale, and the shoreline (Stage E shore.rs) supplies the
/// edges. Sparse sparkle where the ripple peaks.
fn render_water(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let swell = Fbm::new(seed, 3).octaves(2);
    let ripple = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 9).octaves(2);
    let sparkle_seed = seed ^ 0xD1B5_4A32_D192_ED03;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.5 + (swell.sample_uv(u, v) - 0.5) * 0.5 + (ripple.sample_uv(u, v) - 0.5) * 0.26;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            if white_noise(sparkle_seed, x, y) > 0.97 {
                lab.l = (lab.l + 0.04).min(1.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Road: asphalt — dark fBm aggregate with fine grain and sparse
/// light/dark speckle (exposed aggregate and tar spots). Plain
/// asphalt only; painted markings are map art, not material.
fn render_road(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let broad = Fbm::new(seed, 6).octaves(3);
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 16).octaves(2);
    let speckle_seed = seed ^ 0xD1B5_4A32_D192_ED03;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.44 + (broad.sample_uv(u, v) - 0.5) * 0.34 + (grain.sample_uv(u, v) - 0.5) * 0.16;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            let w = white_noise(speckle_seed, x, y);
            if w > 0.93 {
                lab.l = (lab.l + 0.035).min(1.0);
            } else if w < 0.05 {
                lab.l = (lab.l - 0.035).max(0.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// Shingle: overlapping tab rows. Row pitch 8 px, tab width 8 px,
/// odd rows offset half a tab; both phases shifted (x + 4, y + 4)
/// so wrap edges fall mid-tab and mid-row. The bottom two pixels
/// of each row are the shadow gap under the overlapping row
/// above's lip — the darkest line sits one pixel above the row
/// boundary, in the tile interior's rhythm, not on the wrap.
fn render_shingle(img: &mut RgbaImage, ramp: &Ramp, seed: u64) {
    let grain = Fbm::new(seed ^ 0x9E37_79B9_7F4A_7C15, 12).octaves(2);
    let tab_seed = seed ^ 0x2545_F491_4F6C_DD1D;
    let row_h: u32 = 8;
    let tab_w: u32 = 8;
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let sy = (y + row_h / 2) % TEXTURE_SIZE;
            let row = sy / row_h;
            let row_off = if row % 2 == 1 { tab_w / 2 } else { 0 };
            let sx = (x + tab_w / 2 + row_off) % TEXTURE_SIZE;
            let within_y = sy % row_h;
            let within_x = sx % tab_w;
            let tab_id = sx / tab_w + row * (TEXTURE_SIZE / tab_w);
            let tab_off = (white_noise(tab_seed, tab_id, row) - 0.5) * 0.30;
            let (u, v) = (x as f32 / TEXTURE_SIZE as f32, y as f32 / TEXTURE_SIZE as f32);
            let t = 0.48 + tab_off + (grain.sample_uv(u, v) - 0.5) * 0.12;
            let mut lab = ramp.at(t.clamp(0.0, 1.0));
            if within_x == 0 {
                lab.l = (lab.l - 0.045).max(0.0); // tab side gap
            }
            if within_y == row_h - 1 {
                lab.l = (lab.l - 0.085).max(0.0); // shadow under the lip
            } else if within_y == row_h - 2 {
                lab.l = (lab.l - 0.035).max(0.0);
            }
            put(img, x, y, lab);
        }
    }
}

/// De-shade a whole image with the palette transform of
/// [`deshade_palette`]: every pixel's OKLab L is compressed
/// toward `median_l` by `shading_factor`, hue/chroma untouched.
/// Stage D's quilting path runs this over the exemplars so the
/// synthesized texture lives in the same albedo space as the
/// Stage C noise renders (which draw from the de-shaded palette).
fn deshade_image(img: &RgbaImage, median_l: f32, shading_factor: f32) -> RgbaImage {
    let keep = 1.0 - shading_factor.clamp(0.0, 1.0);
    if keep >= 1.0 {
        return img.clone();
    }
    let mut out = RgbaImage::new(img.width(), img.height());
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] < 128 {
            out.put_pixel(x, y, *p);
            continue;
        }
        let mut lab = Oklab::from_srgb_u8([p[0], p[1], p[2]]);
        lab.l = median_l + (lab.l - median_l) * keep;
        let [r, g, b] = lab.to_srgb_u8();
        out.put_pixel(x, y, Rgba([r, g, b, p[3]]));
    }
    out
}

/// Shared Stage D inputs for one family: the de-shaded palette
/// and analysis (exactly as [`generate`] computes them) plus the
/// exemplar images quilting samples from — center crops for
/// ground families (the same [`center_crop`] convention as the
/// Stage C analysis), de-shaded pixel-wise with the same median
/// and factor as the palette.
fn quilt_inputs(
    kind: RecipeKind,
    family: &[RgbaImage],
) -> (Vec<PaletteColor>, SpriteAnalysis, Vec<RgbaImage>) {
    assert!(!family.is_empty(), "quilt inputs: empty family for {}", kind.name());
    let working: Vec<RgbaImage> = if kind.is_ground() {
        family.iter().map(center_crop).collect()
    } else {
        family.to_vec()
    };
    let refs: Vec<&RgbaImage> = working.iter().collect();
    let analysis = analyze_family(&refs, PALETTE_K);
    let palette = deshade_palette(&analysis.palette, analysis.shading.shading_factor);
    let median_l = weighted_median_l(&analysis.palette);
    let exemplars: Vec<RgbaImage> = working
        .iter()
        .map(|img| deshade_image(img, median_l, analysis.shading.shading_factor))
        .collect();
    (palette, analysis, exemplars)
}

/// Quilting geometry for ground synthesis: 12 px patches with a
/// 4 px overlap (step 8 divides both the 32 px tile and the
/// 128 px macro). Sub-exemplar patches (the ground center crop
/// is 16x16) are the normal Efros-Freeman regime; the geometry
/// was picked on measured gate margins — of the geometries tried
/// (16/8, 14/6, 12/4) it gives the best worst-case seam across
/// grass/dirt x tile/macro (max 0.99 vs the 1.15 target). The
/// selection rule that keeps the synthesis from collapsing into
/// single-exemplar tiling is documented in
/// [`crate::quilt::pick_candidate`].
const QUILT_TILE: crate::quilt::QuiltParams = crate::quilt::QuiltParams {
    patch: 12,
    overlap: 4,
};

/// Stage D alternative generation path for the ground recipes
/// (Grass, Dirt): Efros–Freeman quilting over the family's
/// (center-cropped, de-shaded) sprites instead of a noise render
/// from the palette. The Stage C noise path ([`generate`]) is
/// unchanged and stays the default; this path exists so the two
/// can be compared per material (Stage G sign-off). Output is a
/// 32x32 exactly-tileable texture in the same
/// [`GeneratedTexture`] envelope, with the same de-shaded palette
/// the metrics are scored against.
pub fn generate_quilted(kind: RecipeKind, family: &[RgbaImage]) -> GeneratedTexture {
    assert!(
        kind.is_ground(),
        "generate_quilted: only ground recipes (grass, dirt), got {}",
        kind.name()
    );
    let (palette, analysis, exemplars) = quilt_inputs(kind, family);
    let seed = family_seed(family) ^ seed_from_str(kind.name()) ^ seed_from_str("quilt");
    let image = crate::quilt::quilt(&exemplars, TEXTURE_SIZE, QUILT_TILE, seed);
    GeneratedTexture {
        image,
        palette,
        analysis,
    }
}

/// Macro ground textures are 128x128 = 4x4 tiles at the 32 px
/// density (plan decision, pipeline doc SS3.3).
pub const MACRO_SIZE: u32 = 128;

/// One macro ground texture plus its stored height layer.
#[derive(Debug, Clone)]
pub struct MacroTexture {
    /// The 128x128 quilted macro texture (fully opaque,
    /// exactly tileable over the 4x4-tile span).
    pub image: RgbaImage,
    /// 128x128 stored height layer (OKLab L mapped to 0..=255;
    /// see [`crate::quilt::height_layer`]).
    pub height: image::GrayImage,
    /// The de-shaded family palette the macro was synthesized
    /// from.
    pub palette: Vec<PaletteColor>,
    /// The family analysis behind the palette (pre-de-shade).
    pub analysis: SpriteAnalysis,
}

/// Stage D macro ground for Grass / Dirt: quilted at 128x128 so
/// one texture spans 4x4 tiles — the repetition period moves from
/// 1 tile to 4, which is what makes ground read as terrain rather
/// than wallpaper at world scale. The height layer is derived
/// from the finished macro (per-pixel OKLab L, stored explicitly)
/// and feeds [`crate::quilt::height_blend`] at material borders;
/// engine/GPU wiring of both is Stage F.
pub fn generate_macro(kind: RecipeKind, family: &[RgbaImage]) -> MacroTexture {
    assert!(
        kind.is_ground(),
        "generate_macro: only ground recipes (grass, dirt), got {}",
        kind.name()
    );
    let (palette, analysis, exemplars) = quilt_inputs(kind, family);
    let seed = family_seed(family) ^ seed_from_str(kind.name()) ^ seed_from_str("macro");
    let image = crate::quilt::quilt(&exemplars, MACRO_SIZE, QUILT_TILE, seed);
    let height = crate::quilt::height_layer(&image);
    MacroTexture {
        image,
        height,
        palette,
        analysis,
    }
}

/// Plan SS8 quality metrics for generated textures.
pub mod metrics {
    use super::*;

    /// All metrics for one generated texture against the palette
    /// it was rendered from.
    #[derive(Debug, Clone, Copy)]
    pub struct Metrics {
        /// Mean wrap-edge discontinuity / mean interior adjacent
        /// difference (OKLab distances). 1.0 = the wrap border is
        /// indistinguishable from interior pixel steps. Plan
        /// target: < 1.15.
        pub seam_ratio: f32,
        /// Mean over pixels of the OKLab distance to the nearest
        /// palette cluster. Plan target: < 0.04.
        pub mean_delta_e: f32,
        /// Largest single-pixel delta E (diagnostic).
        pub max_delta_e: f32,
        /// Palette clusters used. Plan target: <= 24.
        pub palette_size: usize,
    }

    pub fn evaluate(img: &RgbaImage, palette: &[PaletteColor]) -> Metrics {
        Metrics {
            seam_ratio: seam_ratio(img),
            mean_delta_e: mean_delta_e(img, palette),
            max_delta_e: max_delta_e(img, palette),
            palette_size: palette.len(),
        }
    }

    fn lab_at(img: &RgbaImage, x: u32, y: u32) -> Oklab {
        let p = img.get_pixel(x, y);
        Oklab::from_srgb_u8([p[0], p[1], p[2]])
    }

    /// Seam ratio: mean OKLab distance across the two wrap borders
    /// (right column to left column, bottom row to top row)
    /// divided by the mean distance between interior adjacent
    /// pixels. A perfectly flat texture scores 1.0 by definition
    /// (0/0 is defined as 1: no seam, no texture).
    pub fn seam_ratio(img: &RgbaImage) -> f32 {
        let (w, h) = (img.width(), img.height());
        let mut edge_sum = 0.0f32;
        let mut edge_n = 0u32;
        for y in 0..h {
            edge_sum += lab_at(img, w - 1, y).distance(lab_at(img, 0, y));
            edge_n += 1;
        }
        for x in 0..w {
            edge_sum += lab_at(img, x, h - 1).distance(lab_at(img, x, 0));
            edge_n += 1;
        }
        let mut in_sum = 0.0f32;
        let mut in_n = 0u32;
        for y in 0..h {
            for x in 0..w - 1 {
                in_sum += lab_at(img, x, y).distance(lab_at(img, x + 1, y));
                in_n += 1;
            }
        }
        for y in 0..h - 1 {
            for x in 0..w {
                in_sum += lab_at(img, x, y).distance(lab_at(img, x, y + 1));
                in_n += 1;
            }
        }
        let edge = edge_sum / edge_n.max(1) as f32;
        let interior = in_sum / in_n.max(1) as f32;
        if interior <= 1e-9 {
            if edge <= 1e-9 {
                1.0
            } else {
                f32::INFINITY
            }
        } else {
            edge / interior
        }
    }

    /// Mean OKLab distance from each pixel to the nearest palette
    /// cluster centroid — how far the generated colors stray from
    /// the family palette distribution.
    pub fn mean_delta_e(img: &RgbaImage, palette: &[PaletteColor]) -> f32 {
        delta_e_stats(img, palette).0
    }

    pub fn max_delta_e(img: &RgbaImage, palette: &[PaletteColor]) -> f32 {
        delta_e_stats(img, palette).1
    }

    fn delta_e_stats(img: &RgbaImage, palette: &[PaletteColor]) -> (f32, f32) {
        if palette.is_empty() {
            return (f32::INFINITY, f32::INFINITY);
        }
        let mut sum = 0.0f32;
        let mut max = 0.0f32;
        let mut n = 0u32;
        for p in img.pixels() {
            let lab = Oklab::from_srgb_u8([p[0], p[1], p[2]]);
            let d = palette
                .iter()
                .map(|c| lab.distance(c.lab))
                .fold(f32::INFINITY, f32::min);
            sum += d;
            if d > max {
                max = d;
            }
            n += 1;
        }
        (sum / n.max(1) as f32, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(rgb: [u8; 3]) -> RgbaImage {
        RgbaImage::from_pixel(32, 32, Rgba([rgb[0], rgb[1], rgb[2], 255]))
    }

    /// Two-tone synthetic family with a painted vertical gradient:
    /// top rows dark, bottom rows light (baked shading).
    fn gradient_family() -> Vec<RgbaImage> {
        let mut img = RgbaImage::new(32, 32);
        for y in 0..32 {
            let v = 60 + (y * 4) as u8;
            for x in 0..32 {
                img.put_pixel(x, y, Rgba([v, v.saturating_sub(10), 40, 255]));
            }
        }
        vec![img]
    }

    #[test]
    fn deshade_flattens_lightness_spread() {
        let fam = gradient_family();
        let refs: Vec<&RgbaImage> = fam.iter().collect();
        let a = analyze_family(&refs, 6);
        assert!(a.shading.shading_factor > 0.2, "fixture must read as shaded: {:?}", a.shading);
        let flat = deshade_palette(&a.palette, 1.0);
        let ls: Vec<f32> = flat.iter().map(|c| c.lab.l).collect();
        let spread = ls.iter().cloned().fold(f32::MIN, f32::max) - ls.iter().cloned().fold(f32::MAX, f32::min);
        assert!(spread < 1e-6, "full deshade -> all clusters at median L, spread {spread}");
        let untouched = deshade_palette(&a.palette, 0.0);
        assert_eq!(untouched, a.palette, "factor 0 leaves the palette alone");
    }

    #[test]
    fn generate_is_deterministic_and_opaque() {
        let fam = vec![solid([90, 100, 50]), solid([70, 85, 40])];
        for kind in [RecipeKind::Grass, RecipeKind::Dirt, RecipeKind::WoodFloor, RecipeKind::Plaster] {
            let a = generate(kind, &fam);
            let b = generate(kind, &fam);
            assert_eq!(a.image.as_raw(), b.image.as_raw(), "{} must be byte-identical", kind.name());
            assert_eq!((a.image.width(), a.image.height()), (TEXTURE_SIZE, TEXTURE_SIZE));
            assert!(a.image.pixels().all(|p| p[3] == 255), "{} output must be opaque", kind.name());
            assert!(a.palette.len() <= 24);
        }
    }

    #[test]
    fn metrics_on_flat_texture() {
        let img = solid([120, 120, 120]);
        assert_eq!(metrics::seam_ratio(&img), 1.0);
        let pal = vec![PaletteColor {
            lab: Oklab::from_srgb_u8([120, 120, 120]),
            srgb: [120, 120, 120],
            weight: 1.0,
            count: 1024,
        }];
        assert!(metrics::mean_delta_e(&img, &pal) < 0.01, "roundtrip through u8 srgb");
    }
}
