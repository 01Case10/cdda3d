//! Voxelizer core (plan §8): turns tileset sprites into block face
//! textures, a material table, a billboard sprite atlas, furniture
//! extrusion models, attribution, a contact sheet and a missing log.
//! Everything iterates in sorted order so reruns are byte-identical.

pub mod analyze;
pub mod classify;
pub mod color;
pub mod effects;
pub mod faces;
pub mod material;
pub mod multitile;
pub mod noise;
pub mod overrides;

use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tileset::{resolve, Stage, TileEntry, Tileset};

pub use analyze::*;
pub use classify::*;
pub use color::*;
pub use effects::*;
pub use faces::*;
pub use material::*;
pub use multitile::*;
pub use noise::*;
pub use overrides::*;

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
        let (top_img, alpha) = match &sprite {
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
        let side_img = make_side_face(&top_img, wall_like, mat_idx as u32);
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
        let is_furniture = id.starts_with("f_");
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

    // Materials
    std::fs::write(
        out_dir.join("materials.json"),
        serde_json::to_string_pretty(&materials).unwrap(),
    )
    .map_err(|e| e.to_string())?;

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
