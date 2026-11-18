//! Voxelizer core (plan §8): turns tileset sprites into block face
//! textures, a material table, a billboard sprite atlas, furniture
//! extrusion models, attribution, a contact sheet and a missing log.
//! Everything iterates in sorted order so reruns are byte-identical.

pub mod analyze;
pub mod archetypes;
pub mod classify;
pub mod color;
pub mod decor;
pub mod effects;
pub mod faces;
pub mod generate;
pub mod material;
pub mod multitile;
pub mod noise;
pub mod overrides;
pub mod props;
pub mod quilt;
pub mod shore;

use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tileset::{resolve, Stage, TileEntry, Tileset};

pub use analyze::*;
pub use archetypes::*;
pub use classify::*;
pub use color::*;
pub use decor::*;
pub use effects::*;
pub use faces::*;
pub use generate::*;
pub use material::*;
pub use multitile::*;
pub use noise::*;
pub use overrides::*;
pub use props::*;
pub use quilt::*;
pub use shore::*;

/// Deterministic placeholder: magenta/black checker (plan §8 fallback).
pub fn placeholder_image(size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);
    let cell = (size / 4).max(1);
    for y in 0..size {
        for x in 0..size {
            let on = ((x / cell) + (y / cell)) % 2 == 0;
            img.put_pixel(
                x,
                y,
                if on {
                    Rgba([255, 0, 255, 255])
                } else {
                    Rgba([0, 0, 0, 255])
                },
            );
        }
    }
    img
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialOut {
    pub id: String,
    pub resolved_id: String,
    pub resolution: String,
    pub material_index: usize,
    pub top_layer: usize,
    pub side_layer: usize,
    pub bottom_layer: usize,
    pub height_voxels: u32,
    pub transparency: String, // "opaque" | "cutout" | "transparent"
    pub cutout: bool,
    pub transparent: bool,
    pub tint: [u8; 3],
    pub sub_model: Option<SubModel>,
    pub multitile: bool,
    pub subtile_layers: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubModel {
    pub mode: String, // "extruded" | "box" | "full_block"
    pub height_voxels: u32,
    /// [x, y] sprite-pixel columns that are solid (extruded mode).
    pub columns: Vec<[u32; 2]>,
}

#[derive(Debug, Default)]
pub struct RunReport {
    pub materials: usize,
    pub missing: Vec<(String, String)>, // (id, failed stage note)
    pub stage_counts: BTreeMap<String, usize>,
    pub layers: usize,
    pub sprites: usize,
}

pub struct VoxelizerConfig {
    pub season: String,
    pub tile_size: u32,
    pub overrides: Overrides,
    pub looks_like: BTreeMap<String, String>,
    /// Extra ids to materialize beyond the tileset's own entry ids.
    pub extra_ids: Vec<String>,
    pub tileset_license_text: Option<String>,
    /// Stage F: emit `materials.json` in the v2 form (a
    /// `{"version": 2, "materials": [...], "macros": [...]}` object
    /// plus `macro_<layer>_color.png` / `macro_<layer>_height.png`
    /// files beside it) instead of the v1 bare array. Stage G: the
    /// v2 object also gains a `models` table (solid prop models
    /// for segmentable furniture) when any furniture segments.
    /// Stage F part 2: the v2 object also gains a `shores` table
    /// (per-mask shoreline top-face variants for the primary
    /// water material, §10.1) plus `shore_<layer>.png` files
    /// beside it, when water/dirt/grass families all pool.
    /// Default false: the v1 output stays byte-identical to the
    /// pre-v2 pipeline.
    pub emit_v2: bool,
}

impl Default for VoxelizerConfig {
    fn default() -> Self {
        VoxelizerConfig {
            season: "spring".into(),
            tile_size: 32,
            overrides: Overrides::default(),
            looks_like: BTreeMap::new(),
            extra_ids: Vec::new(),
            tileset_license_text: None,
            emit_v2: false,
        }
    }
}

/// Load a sprite image for a reference: decomposed PNG file, resized
/// (nearest) is NOT done here — the raw sprite is returned at its real
/// size; face generation crops/anchors it.
pub fn load_sprite_image(
    ts: &Tileset,
    entry: &TileEntry,
    r: &tileset::SpriteRef,
) -> Option<RgbaImage> {
    let p = r.primary()?;
    if let Some(name) = p.name() {
        let geom = ts.sprites_by_name.get(name)?;
        let path = geom.png_path.as_ref()?;
        image::open(path).ok().map(|i| i.to_rgba8())
    } else if let Some(idx) = p.index() {
        // Composed form: slice the sheet image.
        let file = entry.sheet_file.as_ref()?;
        let geom = ts.sprites_by_index.get(&(file.clone(), idx))?;
        let sheet_path = ts.root.join(file);
        let sheet = image::open(&sheet_path).ok()?.to_rgba8();
        let across = ts
            .sheets
            .get(file)
            .and_then(|s| s.sprites_across)
            .unwrap_or_else(|| sheet.width() / geom.width.max(1));
        let sx = (idx % across) * geom.width;
        let sy = (idx / across) * geom.height;
        if sx + geom.width > sheet.width() || sy + geom.height > sheet.height() {
            return None;
        }
        Some(image::imageops::crop_imm(&sheet, sx, sy, geom.width, geom.height).to_image())
    } else {
        None
    }
}

/// The FG sprite a terrain model is built from. For a
/// multitile entry the base fg is often just the
/// unconnected variant (a lone fence post); the model
/// builders want the "center" variant -- the full
/// straight segment / connected face. Falls back to the
/// entry's own fg when no center variant exists.
pub fn terrain_fg_image(ts: &Tileset, entry: &TileEntry) -> Option<RgbaImage> {
    // "_olight" variants are a floor plus a drawn
    // overhead light fixture (fg = "overhead_light",
    // bg = the floor sprite). The fixture belongs to
    // the lighting system, not the terrain model:
    // build from the floor alone. (Building from the
    // lamp produced a black slab with a grey smudge.)
    if !entry.ids.is_empty()
        && entry.ids.iter().all(|i| i.ends_with("_olight"))
    {
        if let Some(bg) = entry
            .bg
            .as_ref()
            .and_then(|r| load_sprite_image(ts, entry, r))
        {
            return Some(bg);
        }
    }
    let img = if let Some(center) = entry.additional_tiles.get("center") {
        center
            .fg
            .as_ref()
            .and_then(|r| load_sprite_image(ts, center, r))
            .or_else(|| {
                entry.fg.as_ref().and_then(|r| load_sprite_image(ts, entry, r))
            })
    } else {
        entry.fg.as_ref().and_then(|r| load_sprite_image(ts, entry, r))
    }?;
    // Overlay entries (locked / pickable / alarmed
    // variants): the fg is a small status icon
    // ("pickable", "armed_trap") and the object itself
    // is the bg sprite, named by tile id. Composite the
    // icon over the object, exactly as the 2D tile
    // draws -- building from the icon alone produced a
    // modal-colour slab with a glyph on it.
    let opaque = img.pixels().filter(|px| px[3] >= 128).count();
    if opaque < 64 {
        if let Some(bg) = entry
            .bg
            .as_ref()
            .and_then(|r| load_sprite_image(ts, entry, r))
        {
            return Some(composite_over(&bg, &img));
        }
    }
    Some(img)
}

/// Composite an entry's fg over its bg (either may be absent).
pub fn composite_entry(ts: &Tileset, entry: &TileEntry) -> Option<RgbaImage> {
    let bg = entry
        .bg
        .as_ref()
        .and_then(|r| load_sprite_image(ts, entry, r));
    let fg = entry
        .fg
        .as_ref()
        .and_then(|r| load_sprite_image(ts, entry, r));
    match (bg, fg) {
        (Some(b), Some(f)) => Some(composite_over(&b, &f)),
        (None, Some(f)) => Some(f),
        (Some(b), None) => Some(b),
        (None, None) => None,
    }
}

/// Stage G: encode one prop model for the v2 `models` table.
///
/// Encoding (the engine loader in crates/engine/src/materials.rs
/// is the other half of this contract): `palette` holds the
/// model's distinct RGB colors in first-appearance order over
/// the voxel scan; `voxels` is a hex string with exactly four
/// lowercase hex digits per voxel — the voxel's palette index —
/// in [`PropModel`]'s own layout (index `(y * depth + z) *
/// width + x`, i.e. y-major, then z, then x). Index `0xFFFF` is
/// the reserved EMPTY marker (no voxel), so a palette may hold
/// at most 0xFFFF entries. Alpha is dropped: model voxels are
/// opaque solids by construction (cutout/transparency stay with
/// the material's texture layers, which serve unmodeled
/// materials). Returns `None` if the palette overflows the
/// index space — not reachable for real sprites (a 32x64 sprite
/// has at most 2048 pixels), but the encoding must never wrap
/// an index into the EMPTY marker.
fn encode_prop_model(
    material_index: usize,
    model: &PropModel,
    kind: Option<crate::props::TerrainModelKind>,
) -> Option<serde_json::Value> {
    use std::collections::HashMap;
    let mut palette: Vec<[u8; 3]> = Vec::new();
    let mut index_of: HashMap<[u8; 3], u16> = HashMap::new();
    let mut hex = String::with_capacity(model.voxels.len() * 4);
    for v in &model.voxels {
        let idx = match v {
            None => u16::MAX,
            Some(px) => {
                let rgb = [px[0], px[1], px[2]];
                match index_of.get(&rgb) {
                    Some(&i) => i,
                    None => {
                        if palette.len() >= u16::MAX as usize {
                            return None;
                        }
                        let i = palette.len() as u16;
                        palette.push(rgb);
                        index_of.insert(rgb, i);
                        i
                    }
                }
            }
        };
        hex.push_str(&format!("{idx:04x}"));
    }
    let mut entry = serde_json::json!({
        "material_index": material_index,
        "size": model.size,
        "palette": palette,
        "voxels": hex,
    });
    // Terrain wiring (2026-11-18): terrain entries record
    // how the engine seats the model on the snapshot path
    // (ground / fence / standing). Furniture entries omit
    // the field -- absence means standing, the pre-wiring
    // behaviour, so older readers are unaffected.
    if let Some(kind) = kind {
        entry["kind"] = serde_json::json!(kind.as_str());
    }
    Some(entry)
}

/// Run the full pipeline into `out_dir`.
pub fn run(ts: &Tileset, cfg: &VoxelizerConfig, out_dir: &Path) -> Result<RunReport, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let blocks_dir = out_dir.join("blocks");
    std::fs::create_dir_all(&blocks_dir).map_err(|e| e.to_string())?;
    let tile = cfg.tile_size;

    // Id universe: tileset entries + extras, sorted.
    let mut ids: Vec<String> = ts.entries.keys().cloned().collect();
    for e in &cfg.extra_ids {
        if !ids.contains(e) {
            ids.push(e.clone());
        }
    }
    ids.sort();
    ids.dedup();

    let mut report = RunReport::default();
    let mut materials: Vec<MaterialOut> = Vec::new();
    // Stage F (v2 only): generated ground macros, in material
    // order — (material_index, color, height). Filled during the
    // material loop when `cfg.emit_v2`; written after it.
    let mut macros: Vec<(usize, RgbaImage, image::GrayImage)> = Vec::new();
    // Stage G (v2 only): solid prop models for furniture materials
    // whose sprite segments, in material order —
    // (material_index, model). Filled during the material loop
    // when `cfg.emit_v2`; written after it.
    let mut models: Vec<(usize, PropModel, Option<crate::props::TerrainModelKind>)> =
        Vec::new();
    // Stage F part 2 (v2 only): the first poolable family per
    // ground class (grass/dirt/water), for the §10.1 shore set —
    // the shore textures are generated from the Stage C/E
    // generated textures of these families, not from sprites.
    let mut shore_families: BTreeMap<MaterialClass, Vec<RgbaImage>> = BTreeMap::new();
    // Stage F part 2 (v2 only): terrain materials classifying as
    // Water, in material order. The shore set attaches to ONE of
    // them (see the writing code below).
    let mut shore_water_materials: Vec<usize> = Vec::new();
    let mut layers: Vec<RgbaImage> = Vec::new(); // global layer list
    let mut missing_lines: Vec<String> = Vec::new();

    let push_layer = |layers: &mut Vec<RgbaImage>, img: RgbaImage| -> usize {
        layers.push(img);
        layers.len() - 1
    };

    for (mat_idx, id) in ids.iter().enumerate() {
        let res = resolve(ts, id, &cfg.looks_like, None, &cfg.season);
        *report
            .stage_counts
            .entry(res.stage.name().to_string())
            .or_insert(0) += 1;
        // Filler is a coverage stage (5.10), not a material: an id that
        // only reaches the generic filler has no real material, so it
        // gets the placeholder + a missing.log line (plan §8 fallback).
        if res.stage == Stage::Missing || res.stage == Stage::Filler {
            let note = format!("{id}\tmissing after exact/variant/season/looks_like (filler-only)");
            missing_lines.push(note.clone());
            report.missing.push((id.clone(), note));
            let ph = placeholder_image(tile);
            let top = push_layer(&mut layers, ph.clone());
            let side = push_layer(&mut layers, ph.clone());
            let bottom = push_layer(&mut layers, ph);
            materials.push(MaterialOut {
                id: id.clone(),
                resolved_id: id.clone(),
                resolution: "missing".into(),
                material_index: mat_idx,
                top_layer: top,
                side_layer: side,
                bottom_layer: bottom,
                height_voxels: 3,
                transparency: "opaque".into(),
                cutout: false,
                transparent: false,
                tint: [255, 255, 255],
                sub_model: None,
                multitile: false,
                subtile_layers: BTreeMap::new(),
            });
            continue;
        }
        let entry = res.entry.unwrap().clone();
        let sprite = composite_entry(ts, &entry);
        let (sprite_top, alpha) = match &sprite {
            Some(img) => {
                let t = make_top_face(img, ts, &entry, tile);
                let a = analyze_alpha(&t);
                (t, a)
            }
            None => (
                placeholder_image(tile),
                analyze_alpha(&placeholder_image(tile)),
            ),
        };
        let wall_like = is_wall_like(id, &entry);
        let is_furniture = id.starts_with("f_");
        // Side: wall sides stay the sprite verbatim (its trim band
        // sits at the wall's top and its baseboard at its bottom —
        // their correct positions on a side face; see the Stage F
        // part 2 record). Non-wall sides follow the edge-row rule.
        let side_img = make_side_face(&sprite_top, wall_like, mat_idx as u32);
        // Top: Stage F part 2 generated end-cap for wall materials.
        // A wall sprite has no top-face pixels (sprite-anatomy §4),
        // so a wall's top is generated from its family palette
        // (generate_wall_cap), never the sprite lying flat. Only
        // the top layer changes: side, bottom and subtile (side
        // variant) layers keep the sprite. Furniture is excluded —
        // its top feeds the extrusion columns.
        //
        // Multitile ground (terrain, fields, roads — not walls,
        // not furniture): the entry's base fg is the UNCONNECTED
        // variant by tileset convention — in the 2D game the base
        // sprite is what an isolated tile draws (for water, a
        // lone puddle composited over grass). A voxel block's
        // plain face is the opposite default: the fully-connected
        // CENTER variant, with connection shaping left to the
        // variant systems (shore tops for water). Taking the
        // base sprite as the plain top made every connected
        // water tile — interiors above all, which keep the plain
        // top by the shore rule — render as its own separate
        // puddle. The alpha flags follow the face actually used.
        let (top_img, alpha) = if wall_like && !is_furniture && tile == TEXTURE_SIZE {
            let family = family_sprites(ts, &entry);
            if family.is_empty() {
                (sprite_top.clone(), alpha)
            } else {
                let palette = analyze_image(&sprite_top, 5).palette;
                let class = classify(
                    id,
                    Some(TileKind::from_id_prefix(id)),
                    Some(&palette),
                    &cfg.overrides,
                )
                .class;
                (generate_wall_cap(class, &family).image, alpha)
            }
        } else if entry.multitile && !wall_like && !is_furniture {
            match entry.additional_tiles.get("center").and_then(|sub| {
                composite_entry(ts, sub).map(|img| make_top_face(&img, ts, sub, tile))
            }) {
                Some(t) => {
                    let a = analyze_alpha(&t);
                    (t, a)
                }
                None => (sprite_top.clone(), alpha),
            }
        } else {
            (sprite_top.clone(), alpha)
        };
        let bottom_img = darken(&side_img, 0.5);
        let top_layer = push_layer(&mut layers, top_img.clone());
        let side_layer = push_layer(&mut layers, side_img);
        let bottom_layer = push_layer(&mut layers, bottom_img);

        // Multitile subtile layers (sorted subtile order).
        let mut subtile_layers = BTreeMap::new();
        if entry.multitile {
            for (sub_id, sub) in &entry.additional_tiles {
                if let Some(img) = composite_entry(ts, sub) {
                    let t = make_top_face(&img, ts, sub, tile);
                    let l = push_layer(&mut layers, t);
                    subtile_layers.insert(sub_id.clone(), l);
                }
            }
        }

        let ov = cfg.overrides.entries.get(id);
        if let Some(o) = ov {
            o.validate()
                .map_err(|e| format!("override for {id}: {e}"))?;
        }
        let (height, sub_model) = if is_furniture {
            let h = furniture_height(id, ov);
            let mode = ov
                .map(|o| o.mode.clone())
                .unwrap_or_else(|| "extruded".into());
            let cols = if mode == "extruded" {
                extrusion_columns(&top_img)
            } else {
                Vec::new()
            };
            (
                h,
                Some(SubModel {
                    mode,
                    height_voxels: h,
                    columns: cols,
                }),
            )
        } else if wall_like {
            (3, None)
        } else {
            (1, None)
        };
        let height = ov.and_then(|o| o.height).unwrap_or(height);
        let (cutout, transparent) = alpha_flags(id, &alpha, ov);
        materials.push(MaterialOut {
            id: id.clone(),
            resolved_id: res.resolved_id.clone(),
            resolution: res.stage.name().into(),
            material_index: mat_idx,
            top_layer,
            side_layer,
            bottom_layer,
            height_voxels: height,
            transparency: if transparent {
                "transparent".into()
            } else if cutout {
                "cutout".into()
            } else {
                "opaque".into()
            },
            cutout,
            transparent,
            tint: ov.map(|o| o.tint).unwrap_or([255, 255, 255]),
            sub_model,
            multitile: entry.multitile,
            subtile_layers,
        });

        // Stage F (v2 only): a Grass/Dirt material whose family
        // pools gets a generated macro (Stage D's generate_macro).
        // Classification follows the preview's recipe: palette from
        // the composited sprite's analysis, kind from the id
        // prefix. A family that can't be pooled (no sprites) omits
        // the macro — the engine then keeps the v1 per-tile top
        // face for this material.
        if cfg.emit_v2 {
            if let Some(img) = &sprite {
                let palette = analyze_image(img, 5).palette;
                let class = classify(
                    id,
                    Some(TileKind::from_id_prefix(id)),
                    Some(&palette),
                    &cfg.overrides,
                )
                .class;
                let kind = match class {
                    MaterialClass::Grass => Some(RecipeKind::Grass),
                    MaterialClass::Dirt => Some(RecipeKind::Dirt),
                    _ => None,
                };
                if let Some(kind) = kind {
                    let family = family_sprites(ts, &entry);
                    if !family.is_empty() {
                        let m = generate_macro(kind, &family);
                        macros.push((mat_idx, m.image, m.height));
                    }
                }
            }
        }

        // Stage F part 2 (v2 only): pool the first Grass/Dirt/
        // Water family of the run for the shore set, and remember
        // the terrain water materials. Classification mirrors the
        // macro block above (palette from the composited sprite,
        // kind from the id prefix).
        if cfg.emit_v2 {
            if let Some(img) = &sprite {
                let palette = analyze_image(img, 5).palette;
                let class = classify(
                    id,
                    Some(TileKind::from_id_prefix(id)),
                    Some(&palette),
                    &cfg.overrides,
                )
                .class;
                // First poolable family per class wins, except
                // the canonical source ids (t_grass / t_dirt /
                // t_water_dp — the trio the Stage E gate composes)
                // always take over their class, so the shore
                // set's sources are named, not accidental.
                let canonical = matches!(
                    (class, id.as_str()),
                    (MaterialClass::Grass, "t_grass")
                        | (MaterialClass::Dirt, "t_dirt")
                        | (MaterialClass::Water, "t_water_dp")
                );
                if matches!(
                    class,
                    MaterialClass::Grass | MaterialClass::Dirt | MaterialClass::Water
                ) && (canonical || !shore_families.contains_key(&class))
                {
                    let family = family_sprites(ts, &entry);
                    if !family.is_empty() {
                        shore_families.insert(class, family);
                    }
                }
                if class == MaterialClass::Water && id.starts_with("t_") {
                    shore_water_materials.push(mat_idx);
                }
            }
        }

        // Stage G (v2 only): a furniture material gets its prop
        // model in the v2 `models` table via the view-routed
        // props::model_for (front-view archetype/solid for most
        // ids; the G.3 rework's view-faithful builders for the
        // pinned top-down / end-on ids); the engine loader
        // documents the encoding (crates/engine/src/materials.rs).
        // Furniture model_for declines (no segmentation, no
        // readable plan/cap) keeps the extruded-sprite sub_model
        // it already carries — models replace the extrusion only
        // where a prop model exists.
        if cfg.emit_v2 {
            if let Some(img) = &sprite {
                // Furniture takes the prop builders; terrain
                // takes the terrain builders (fence, trees --
                // ids whose block rendering is a textured
                // cube). Each declines ids it does not serve.
                let model: Option<(PropModel, Option<crate::props::TerrainModelKind>)> =
                    if is_furniture {
                        model_for(id, img).map(|m| (m, None))
                    } else {
                    // Terrain builders read the FG sprite
                    // alone: the composite bakes the bg
                    // (grass under a tree) into the
                    // silhouette they measure.
                    // t_chainfence keeps its base fg: its
                    // approved model samples colours from
                    // it and the builder is geometric.
                    let fg = if id == "t_chainfence"
                        || id.starts_with("t_chainfence_season")
                    {
                        entry
                            .fg
                            .as_ref()
                            .and_then(|r| load_sprite_image(ts, &entry, r))
                    } else {
                        terrain_fg_image(ts, &entry)
                    };
                        crate::props::terrain_model_for(id, fg.as_ref().unwrap_or(img))
                            .map(|(m, k)| (m, Some(k)))
                    };
                if let Some((model, kind)) = model {
                    models.push((mat_idx, model, kind));
                }
            }
        }
    }
    report.materials = materials.len();
    report.layers = layers.len();

    // Write block layers as PNGs (documented choice: PNG layers instead
    // of ktx2 — no dependency-free ktx2 encoder exists for this offline
    // pipeline; PNG is lossless and the engine uploads each PNG as one
    // texture-array layer, so the layer indexing is identical).
    for (i, img) in layers.iter().enumerate() {
        img.save(blocks_dir.join(format!("layer_{i:04}.png")))
            .map_err(|e| e.to_string())?;
    }
    let blocks_manifest = serde_json::json!({
        "format": "png-layers",
        "note": "Each PNG is one texture-array layer (tile_size x tile_size). PNG chosen over ktx2: lossless, no external encoder dependency; layer indices match materials.json exactly.",
        "tile_size": tile,
        "layer_count": layers.len(),
    });
    std::fs::write(
        out_dir.join("blocks.json"),
        serde_json::to_string_pretty(&blocks_manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;

    // Materials. v1 (default) is the bare array, byte-identical to
    // the pre-v2 pipeline. v2 wraps the same array in an object
    // with a version field and the macro table, and writes each
    // macro's color/height PNGs beside it: macro layer indices are
    // assigned in material order, color = 2j, height = 2j + 1 for
    // the j-th macro, and the files are named by that layer index
    // (`macro_<layer>_color.png` / `macro_<layer>_height.png`).
    if cfg.emit_v2 {
        let mut macro_entries = Vec::new();
        for (j, (mat_idx, color, height)) in macros.iter().enumerate() {
            let (color_layer, height_layer) = (2 * j, 2 * j + 1);
            color
                .save(out_dir.join(format!("macro_{color_layer:04}_color.png")))
                .map_err(|e| e.to_string())?;
            height
                .save(out_dir.join(format!("macro_{height_layer:04}_height.png")))
                .map_err(|e| e.to_string())?;
            macro_entries.push(serde_json::json!({
                "material_index": mat_idx,
                "color_layer": color_layer,
                "height_layer": height_layer,
                "span_tiles": color.width() / tile.max(1),
            }));
        }
        let mut v2 = serde_json::json!({
            "version": 2,
            "materials": serde_json::to_value(&materials).unwrap(),
            "macros": macro_entries,
        });
        // Stage F part 2: the `shores` table (§10.1 engine half).
        // Design, decided for this slice and recorded here:
        // - ONE shore set per run, generated from the Stage C/E
        //   generated textures of the run's first poolable
        //   water/dirt/grass families (the same trio the Stage E
        //   gate composes), attached to the primary water
        //   material: `t_water_dp` when the run has it, else the
        //   first terrain water material in material order. The
        //   other water ids (season/underground/moving variants)
        //   keep their plain top faces in this slice — a shore
        //   set per water id would multiply the layer count by
        //   the ~40 water ids of a real tileset for variants the
        //   sample scenes never select.
        // - Mask set: every mask except 0xFF (all-water = an
        //   interior tile, which keeps the plain water texture
        //   byte-for-byte — shore_texture(0xFF) is the identity
        //   by the Stage E gate, so emitting it would waste a
        //   layer). 255 variants, one 32x32 PNG each
        //   (`shore_<layer>.png` beside materials.json, layer
        //   indices assigned in ascending mask order).
        // - The table is present only when all three families
        //   pooled and a terrain water material exists, so a v2
        //   file with no shore set stays byte-identical to the
        //   Stage F/G form.
        if !shore_water_materials.is_empty() {
            let trio = (
                shore_families.get(&MaterialClass::Water),
                shore_families.get(&MaterialClass::Dirt),
                shore_families.get(&MaterialClass::Grass),
            );
            if let (Some(water_fam), Some(dirt_fam), Some(grass_fam)) = trio {
                let primary = shore_water_materials
                    .iter()
                    .copied()
                    .find(|&i| materials[i].id == "t_water_dp")
                    .unwrap_or(shore_water_materials[0]);
                let water = generate(RecipeKind::Water, water_fam).image;
                let dirt = generate(RecipeKind::Dirt, dirt_fam).image;
                let grass = generate(RecipeKind::Grass, grass_fam).image;
                let mut shore_entries = Vec::new();
                for mask in 0..=254u8 {
                    let layer = shore_entries.len();
                    shore_texture(mask, &water, &dirt, &grass)
                        .save(out_dir.join(format!("shore_{layer:04}.png")))
                        .map_err(|e| e.to_string())?;
                    shore_entries.push(serde_json::json!({
                        "material_index": primary,
                        "mask": mask,
                        "layer": layer,
                    }));
                }
                v2["shores"] = serde_json::Value::Array(shore_entries);
            }
        }
        // Stage G: the `models` table is present only when at
        // least one furniture material segmented, so a v2 file
        // with nothing to model stays byte-identical to the
        // Stage F form.
        if !models.is_empty() {
            let entries: Vec<serde_json::Value> = models
                .iter()
                .filter_map(|(mat_idx, m, kind)| encode_prop_model(*mat_idx, m, *kind))
                .collect();
            if !entries.is_empty() {
                v2["models"] = serde_json::Value::Array(entries);
            }
        }
        std::fs::write(
            out_dir.join("materials.json"),
            serde_json::to_string_pretty(&v2).unwrap(),
        )
        .map_err(|e| e.to_string())?;
    } else {
        std::fs::write(
            out_dir.join("materials.json"),
            serde_json::to_string_pretty(&materials).unwrap(),
        )
        .map_err(|e| e.to_string())?;
    }

    // Sprite atlas for billboards: pack every named sprite sorted by
    // name, shelf packing at real sizes, carrying w/h/offset.
    let (atlas, uvs) = build_atlas(ts)?;
    atlas
        .save(out_dir.join("sprites.png"))
        .map_err(|e| e.to_string())?;
    std::fs::write(
        out_dir.join("sprites.json"),
        serde_json::to_string_pretty(&uvs).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    report.sprites = uvs.len();

    // Contact sheet: one top face per material, grid.
    let contact = build_contact_sheet(&layers, &materials, tile);
    contact
        .save(out_dir.join("contact.png"))
        .map_err(|e| e.to_string())?;

    // Missing log (sorted, one line per id).
    missing_lines.sort();
    std::fs::write(out_dir.join("missing.log"), missing_lines.join("\n") + "\n")
        .map_err(|e| e.to_string())?;

    // Attribution
    let attribution = build_attribution(ts, cfg);
    std::fs::write(out_dir.join("ATTRIBUTION.md"), attribution).map_err(|e| e.to_string())?;

    Ok(report)
}

fn build_atlas(ts: &Tileset) -> Result<(RgbaImage, Vec<SpriteUv>), String> {
    // Shelf pack, sorted by name. Width = max sprite width * 8 shelves.
    let mut items: Vec<(&String, &tileset::SpriteGeom)> = ts.sprites_by_name.iter().collect();
    items.sort_by(|a, b| a.0.cmp(b.0));
    let atlas_w: u32 = 512;
    let mut x = 0u32;
    let mut y = 0u32;
    let mut shelf_h = 0u32;
    let mut placements: Vec<(String, u32, u32)> = Vec::new();
    for (name, g) in &items {
        if g.width > atlas_w {
            continue;
        }
        if x + g.width > atlas_w {
            x = 0;
            y += shelf_h;
            shelf_h = 0;
        }
        placements.push(((*name).clone(), x, y));
        x += g.width;
        shelf_h = shelf_h.max(g.height);
    }
    let atlas_h = (y + shelf_h).max(1);
    let mut atlas = RgbaImage::new(atlas_w, atlas_h);
    let mut uvs = Vec::new();
    for (name, px, py) in &placements {
        let g = &ts.sprites_by_name[name];
        if let Some(path) = &g.png_path {
            if let Ok(img) = image::open(path) {
                let img = img.to_rgba8();
                image::imageops::replace(&mut atlas, &img, *px as i64, *py as i64);
            }
        }
        uvs.push(SpriteUv {
            name: name.clone(),
            x: *px,
            y: *py,
            width: g.width,
            height: g.height,
            offset_x: g.offset_x,
            offset_y: g.offset_y,
            sheet: g.sheet_file.clone(),
        });
    }
    Ok((atlas, uvs))
}

fn build_contact_sheet(layers: &[RgbaImage], materials: &[MaterialOut], tile: u32) -> RgbaImage {
    let n = materials.len().max(1);
    let cols = (n as f64).sqrt().ceil() as u32;
    let rows = (n as u32 + cols - 1) / cols;
    let mut img = RgbaImage::new(cols * tile, rows * tile);
    for (i, m) in materials.iter().enumerate() {
        let cx = (i as u32 % cols) * tile;
        let cy = (i as u32 / cols) * tile;
        if let Some(layer) = layers.get(m.top_layer) {
            image::imageops::replace(&mut img, layer, cx as i64, cy as i64);
        }
    }
    img
}

fn build_attribution(ts: &Tileset, cfg: &VoxelizerConfig) -> String {
    let mut s = String::new();
    s.push_str("# Attribution\n\n");
    s.push_str(&format!(
        "Tileset: {} ({})\n\n",
        if ts.name.is_empty() {
            "unknown"
        } else {
            &ts.name
        },
        ts.root.display()
    ));
    s.push_str("Derived textures in this directory are derivative works of the tileset sprites and are licensed under **CC BY-SA 3.0**, with attribution to the original tileset authors (CDDA-Tilesets, I-am-Erk and contributors).\n\n");
    s.push_str("## Per-sheet sources (from tile_info.json)\n\n");
    for (file, sheet) in &ts.sheets {
        s.push_str(&format!(
            "- `{file}`: {}x{} offset ({}, {}){} {}\n",
            sheet.sprite_width,
            sheet.sprite_height,
            sheet.sprite_offset_x,
            sheet.sprite_offset_y,
            if sheet.filler { " [filler]" } else { "" },
            sheet
                .source
                .as_ref()
                .map(|src| format!("source: {src}"))
                .unwrap_or_default()
        ));
    }
    s.push_str("\n## License\n\n");
    s.push_str("Cataclysm: Dark Days Ahead and CDDA-Tilesets are licensed under the Creative Commons Attribution-ShareAlike 3.0 Unported License (http://creativecommons.org/licenses/by-sa/3.0/).\n\n");
    s.push_str("## Third-party notices (from the Tilesets LICENSE.txt)\n\n");
    if let Some(text) = &cfg.tileset_license_text {
        // Everything after the CC BY-SA header paragraph is notices.
        let body = text.splitn(2, "\n\n").nth(1).unwrap_or(text.as_str());
        s.push_str(body);
        if !body.ends_with('\n') {
            s.push('\n');
        }
    } else {
        s.push_str("(LICENSE.txt not supplied to this run.)\n");
    }
    s
}

/// Collect output file list (for golden tests).
pub fn list_outputs(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.filter_map(|e| e.ok()) {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    out
}
