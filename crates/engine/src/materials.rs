//! Minimal Phase 5 integration: load the voxelizer's `materials.json`
//! + `blocks.json` so the engine can resolve a game id to its block
//! texture layers, transparency pass and height.
//!
//! Stage G: the v2 `models` table. Each entry carries one
//! furniture material's solid prop model (voxelizer `PropModel`)
//! as `{material_index, size: [w, h, d], palette: [[r,g,b], ...],
//! voxels: "<hex>"}`. `voxels` has exactly four lowercase hex
//! digits per voxel — the voxel's palette index — in the
//! voxelizer's layout (index `(y * d + z) * w + x`: y-major, then
//! z, then x); index `0xFFFF` is the reserved EMPTY marker.
//! Model axes, for the future world stamper: model x is the
//! sprite's left-to-right (east on the tile), model y is up
//! (y = 0 is the object's bottom), model z is depth with
//! z = d - 1 the FRONT — the face the sprite shows — which maps
//! to the tile's south edge (+y in game tile space, toward the
//! viewer), matching how the extruded sprite presents its front.
//!
//! Stamping status: the loader half is complete (models are
//! parsed, validated and joined to their materials below), but
//! no engine code stamps them into a world yet. The engine's
//! world grid is one block per tile footprint (mesher
//! `WorldScale`, Phase 0 decision: 1 tile = 1 block, 3 blocks
//! per z-level) — there is no sub-tile voxel grid in either
//! renderer for a 32-wide model to land in, so world stamping
//! waits on a sub-tile world representation decision. Picking,
//! LOS and collision are unaffected either way: models are
//! visual only, game tile data governs.
//!
//! Scope note: the mesher's `Block`/`Vertex` types (crates/mesher,
//! read-only for Phase 5) carry flat colors and no UV/texture-layer
//! field, so textured *rendering* needs a mesher+pipeline change that
//! belongs to a follow-up. What lands here is the data half: parsing,
//! validation and id -> material lookup, unit-tested, plus the
//! headless-capture regression proving the Phase 2 renderer is
//! untouched.

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub id: String,
    pub resolved_id: String,
    pub resolution: String,
    pub top_layer: usize,
    pub side_layer: usize,
    pub bottom_layer: usize,
    pub height_voxels: u32,
    pub transparency: String,
    pub cutout: bool,
    pub transparent: bool,
    /// Phase 2d: the voxelizer's multitile flag — this material's
    /// side picture depends on its cardinal neighbors.
    pub multitile: bool,
    /// Phase 2d: subtile name ("center", "corner", ...) -> global
    /// layer index, from the voxelizer's `subtile_layers`. Only the
    /// subtiles the tileset entry actually carries are present.
    pub subtile_layers: BTreeMap<String, usize>,
    /// Stage F: this material's world-space macro (materials.json
    /// v2 `macros` table), when the voxelizer generated one.
    /// `None` in v1 files and for materials without a pooled
    /// family — those keep the per-tile top face.
    pub macro_: Option<MacroEntry>,
    /// Stage G: this material's solid prop model (materials.json
    /// v2 `models` table), when the voxelizer's segmentation
    /// produced one. `None` in v1 files, in v2 files without a
    /// `models` table, and for furniture whose sprite doesn't
    /// segment — those keep the extruded-sprite rendering.
    pub model: Option<PropModelData>,
    /// Stage F part 2: this material's shoreline variants
    /// (materials.json v2 `shores` table): 8-neighbor water mask
    /// (bit order canonical in `mesher::connect`) -> shore layer
    /// index into the `shore_<layer>.png` file set beside
    /// materials.json. Only the primary water material of a run
    /// carries them; empty in v1 files, in v2 files without a
    /// `shores` table, and for every other material. Mask 0xFF
    /// (all-water interior) is never present — it keeps the
    /// plain top layer by rule.
    pub shore_layers: BTreeMap<u8, usize>,
}

/// Stage G: one entry of the materials.json v2 `models` table —
/// a furniture material's solid prop model, decoded into the
/// engine-side representation: a palette plus one palette index
/// per voxel (see the module docs for the wire encoding and the
/// axis convention). Mirrors the voxelizer's `PropModel` layout
/// so `get(x, y, z)` means the same thing on both sides.
#[derive(Debug, Clone, PartialEq)]
pub struct PropModelData {
    /// [width (x), height (y, up), depth (z)] in model voxels.
    pub size: [u32; 3],
    /// Distinct RGB colors; voxel indices address this table.
    pub palette: Vec<[u8; 3]>,
    /// One palette index per voxel, `(y * depth + z) * width + x`
    /// order; `EMPTY` where the model has no voxel.
    pub voxels: Vec<u16>,
}

impl PropModelData {
    /// The reserved palette index for "no voxel here".
    pub const EMPTY: u16 = u16::MAX;

    /// The voxel color at (x, y, z), or `None` for empty /
    /// out-of-bounds cells.
    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<[u8; 3]> {
        let [w, h, d] = self.size;
        if x >= w || y >= h || z >= d {
            return None;
        }
        let idx = self.voxels[((y * d + z) * w + x) as usize];
        if idx == Self::EMPTY {
            None
        } else {
            self.palette.get(idx as usize).copied()
        }
    }

    /// Number of non-empty voxels.
    pub fn solid_count(&self) -> usize {
        self.voxels.iter().filter(|&&v| v != Self::EMPTY).count()
    }

    /// Decode one `models` table entry, validating the encoding
    /// (hex length matches the size product, palette indices are
    /// in range) so a corrupt file fails the load instead of
    /// panicking or silently misrendering later.
    fn parse(e: &serde_json::Value) -> Result<Self, String> {
        let bad = |what: &str| format!("materials.json models entry: {what}");
        let size_v = e["size"].as_array().ok_or_else(|| bad("size is not an array"))?;
        if size_v.len() != 3 {
            return Err(bad("size must have 3 elements"));
        }
        let mut size = [0u32; 3];
        for (i, v) in size_v.iter().enumerate() {
            size[i] = v.as_u64().ok_or_else(|| bad("size element is not a number"))? as u32;
        }
        let count = size.iter().map(|&s| s as usize).product::<usize>();
        if count == 0 {
            return Err(bad("size has a zero dimension"));
        }
        let pal_v = e["palette"]
            .as_array()
            .ok_or_else(|| bad("palette is not an array"))?;
        if pal_v.len() > u16::MAX as usize {
            return Err(bad("palette exceeds the index space"));
        }
        let mut palette = Vec::with_capacity(pal_v.len());
        for p in pal_v {
            let c = p.as_array().ok_or_else(|| bad("palette color is not an array"))?;
            if c.len() != 3 {
                return Err(bad("palette color must have 3 elements"));
            }
            let mut rgb = [0u8; 3];
            for (i, v) in c.iter().enumerate() {
                rgb[i] = v
                    .as_u64()
                    .ok_or_else(|| bad("palette channel is not a number"))?
                    as u8;
            }
            palette.push(rgb);
        }
        let hex = e["voxels"]
            .as_str()
            .ok_or_else(|| bad("voxels is not a string"))?;
        if hex.len() != count * 4 {
            return Err(bad("voxels hex length does not match size"));
        }
        let mut voxels = Vec::with_capacity(count);
        for chunk in hex.as_bytes().chunks_exact(4) {
            let s = std::str::from_utf8(chunk).map_err(|_| bad("voxels is not ASCII hex"))?;
            let idx = u16::from_str_radix(s, 16).map_err(|_| bad("voxels is not hex"))?;
            if idx != Self::EMPTY && idx as usize >= palette.len() {
                return Err(bad("voxel palette index out of range"));
            }
            voxels.push(idx);
        }
        Ok(PropModelData {
            size,
            palette,
            voxels,
        })
    }
}

/// Stage F: one entry of the materials.json v2 `macros` table — a
/// generated 128px macro texture (color + stored height) that the
/// material's top face samples by world position instead of by
/// in-tile UV. `color_layer` / `height_layer` index the *macro*
/// layer set (the `macro_<layer>_color.png` /
/// `macro_<layer>_height.png` files beside materials.json), not
/// the 32px block array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroEntry {
    pub color_layer: usize,
    pub height_layer: usize,
    /// World span of one macro repeat, in tiles (4 for the
    /// voxelizer's 128px macro at 32px tiles).
    pub span_tiles: u32,
}

impl Material {
    /// The side layer for a voxel of this material whose cardinal
    /// connection mask is `mask` (Phase 2d selection — the one
    /// shared rule both renderers resolve through): the
    /// voxelizer's layer for the mask's subtile when the material
    /// is multitile and carries that subtile, otherwise the base
    /// side layer. Top/bottom layers never vary with the mask.
    pub fn side_layer_for_mask(&self, mask: u8) -> usize {
        if self.multitile {
            let (subtile, _rotation) = mesher::connect::subtile_for_mask(mask);
            if let Some(&layer) = self.subtile_layers.get(subtile) {
                return layer;
            }
        }
        self.side_layer
    }

    /// Stage F part 2: the shore top layer for a water tile
    /// whose 8-neighbor water mask is `mask`, or `None` when
    /// this tile keeps its plain top layer — the all-water
    /// interior mask (0xFF), and every mask for a material
    /// carrying no shore set.
    pub fn shore_top_layer(&self, mask: u8) -> Option<usize> {
        if mask == mesher::connect::SHORE_MASK_ALL_WATER {
            return None;
        }
        self.shore_layers.get(&mask).copied()
    }

    /// The side layer for a state-carrying tile/part (Phase 2d
    /// follow-up): the `open`/`broken` subtile layer when the
    /// entry carries one for the part's state, else the base
    /// side layer. State selection replaces mask selection for
    /// stateful parts — the game's vpart renderer never combines
    /// the two (see `mesher::connect::state_subtile`).
    pub fn side_layer_for_state(&self, is_open: bool, is_broken: bool) -> usize {
        if self.multitile {
            if let Some(subtile) = mesher::connect::state_subtile(is_open, is_broken) {
                if let Some(&layer) = self.subtile_layers.get(subtile) {
                    return layer;
                }
            }
        }
        self.side_layer
    }
}

#[derive(Debug, Default)]
pub struct MaterialTable {
    pub tile_size: u32,
    pub layer_count: usize,
    /// Stage F: macro layer edge length in pixels (128 for
    /// voxelizer v2 output), read from the first macro PNG.
    /// 0 when the file is v1 or carries no macros.
    pub macro_size: u32,
    /// Stage F part 2: shore layer edge length in pixels (the
    /// tile size, 32, for voxelizer v2 output), read from the
    /// first shore PNG. 0 when the file is v1 or carries no
    /// shores.
    pub shore_size: u32,
    by_id: BTreeMap<String, Material>,
}

impl MaterialTable {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let blocks: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("blocks.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mats: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("materials.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        // Stage F: v1 is the bare material array; v2 wraps it in
        // {"version": 2, "materials": [...], "macros": [...]}.
        // Anything else (notably a newer version) is a hard error —
        // silently misreading a future format would shade wrong.
        let mut macro_by_index: BTreeMap<usize, MacroEntry> = BTreeMap::new();
        let mut model_by_index: BTreeMap<usize, PropModelData> = BTreeMap::new();
        let mut shore_by_index: BTreeMap<usize, BTreeMap<u8, usize>> = BTreeMap::new();
        let mat_array: Vec<serde_json::Value> = if mats.is_array() {
            mats.as_array().cloned().unwrap_or_default()
        } else if mats.is_object() {
            let version = mats["version"].as_u64().unwrap_or(0);
            if version != 2 {
                return Err(format!(
                    "materials.json: unsupported version {version} (expected 1 or 2)"
                ));
            }
            if let Some(arr) = mats["macros"].as_array() {
                for e in arr {
                    let get = |k: &str| e[k].as_u64().unwrap_or(0) as usize;
                    macro_by_index.insert(
                        get("material_index"),
                        MacroEntry {
                            color_layer: get("color_layer"),
                            height_layer: get("height_layer"),
                            span_tiles: e["span_tiles"].as_u64().unwrap_or(4) as u32,
                        },
                    );
                }
            }
            // Stage G: the `models` table is optional — a v2 file
            // without one (nothing segmented) loads exactly as
            // before, every material model-less.
            if let Some(arr) = mats["models"].as_array() {
                for e in arr {
                    let mi = e["material_index"].as_u64().unwrap_or(0) as usize;
                    model_by_index.insert(mi, PropModelData::parse(e)?);
                }
            }
            // Stage F part 2: the `shores` table is optional —
            // a v2 file without one loads exactly as before,
            // every material shore-less.
            if let Some(arr) = mats["shores"].as_array() {
                for e in arr {
                    let get = |k: &str| e[k].as_u64().unwrap_or(0) as usize;
                    shore_by_index
                        .entry(get("material_index"))
                        .or_default()
                        .insert(e["mask"].as_u64().unwrap_or(0) as u8, get("layer"));
                }
            }
            mats["materials"].as_array().cloned().unwrap_or_default()
        } else {
            return Err("materials.json: expected a v1 array or a v2 object".into());
        };
        let mut table = MaterialTable {
            tile_size: blocks["tile_size"].as_u64().unwrap_or(32) as u32,
            layer_count: blocks["layer_count"].as_u64().unwrap_or(0) as usize,
            macro_size: 0,
            shore_size: 0,
            by_id: BTreeMap::new(),
        };
        for m in mat_array {
            let get = |k: &str| m[k].as_u64().unwrap_or(0) as usize;
            let mut subtile_layers = BTreeMap::new();
            if let Some(obj) = m["subtile_layers"].as_object() {
                for (k, v) in obj {
                    if let Some(l) = v.as_u64() {
                        subtile_layers.insert(k.clone(), l as usize);
                    }
                }
            }
            let mat = Material {
                id: m["id"].as_str().unwrap_or("").to_string(),
                resolved_id: m["resolved_id"].as_str().unwrap_or("").to_string(),
                resolution: m["resolution"].as_str().unwrap_or("").to_string(),
                top_layer: get("top_layer"),
                side_layer: get("side_layer"),
                bottom_layer: get("bottom_layer"),
                height_voxels: m["height_voxels"].as_u64().unwrap_or(0) as u32,
                transparency: m["transparency"].as_str().unwrap_or("opaque").to_string(),
                cutout: m["cutout"].as_bool().unwrap_or(false),
                transparent: m["transparent"].as_bool().unwrap_or(false),
                multitile: m["multitile"].as_bool().unwrap_or(false),
                subtile_layers,
                macro_: macro_by_index.get(&get("material_index")).copied(),
                model: model_by_index.get(&get("material_index")).cloned(),
                shore_layers: shore_by_index
                    .get(&get("material_index"))
                    .cloned()
                    .unwrap_or_default(),
            };
            if table.layer_count > 0 {
                let layers = [mat.top_layer, mat.side_layer, mat.bottom_layer]
                    .into_iter()
                    .chain(mat.subtile_layers.values().copied());
                if layers.into_iter().any(|l| l >= table.layer_count) {
                    return Err(format!("material {} layer out of range", mat.id));
                }
            }
            table.by_id.insert(mat.id.clone(), mat);
        }
        // Stage F: the macro layer size is whatever the first
        // macro color PNG actually is (128 for voxelizer v2).
        if let Some(first) = macro_by_index.values().next() {
            let path = dir.join(format!("macro_{:04}_color.png", first.color_layer));
            let img = image::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if img.width() != img.height() {
                return Err(format!("{}: macro layer is not square", path.display()));
            }
            table.macro_size = img.width();
        }
        // Stage F part 2: the shore layer size is whatever the
        // first shore PNG actually is (32 = the tile size).
        if let Some(first_layer) = shore_by_index.values().flat_map(|m| m.values()).min() {
            let path = dir.join(format!("shore_{first_layer:04}.png"));
            let img = image::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if img.width() != img.height() {
                return Err(format!("{}: shore layer is not square", path.display()));
            }
            table.shore_size = img.width();
        }
        Ok(table)
    }
    pub fn get(&self, id: &str) -> Option<&Material> {
        self.by_id.get(id)
    }
    pub fn len(&self) -> usize {
        self.by_id.len()
    }
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
    /// Render pass name for a material, matching mesher::Pass naming.
    pub fn pass_of(m: &Material) -> &'static str {
        if m.transparent {
            "Transparent"
        } else if m.cutout {
            "Cutout"
        } else {
            "Opaque"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_voxelizer_output() {
        // Build a tiny voxelizer output in a temp dir via JSON literals
        // shaped exactly like the voxelizer's materials.json/blocks.json.
        let dir = std::env::temp_dir().join(format!("engine_mat_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("blocks.json"),
            r#"{"format":"png-layers","tile_size":32,"layer_count":3}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("materials.json"),
            r#"[{"id":"t_wall","resolved_id":"t_wall","resolution":"exact","material_index":0,"top_layer":0,"side_layer":1,"bottom_layer":2,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":true,"subtile_layers":{"center":0,"edge":2}}]"#,
        )
        .unwrap();
        let t = MaterialTable::load(&dir).unwrap();
        assert_eq!(t.len(), 1);
        let wall = t.get("t_wall").unwrap();
        assert_eq!(wall.height_voxels, 3);
        assert_eq!(MaterialTable::pass_of(wall), "Opaque");
        // Phase 2d: multitile + subtile layers are parsed, and mask
        // selection resolves through them (mask 15 = center, mask
        // 10 = E+W edge; a mask whose subtile is absent — corner —
        // falls back to the base side layer).
        assert!(wall.multitile);
        assert_eq!(wall.side_layer_for_mask(15), 0, "center layer");
        assert_eq!(wall.side_layer_for_mask(10), 2, "edge layer");
        assert_eq!(wall.side_layer_for_mask(3), 1, "corner absent -> base side");
        // v1: no macro anywhere.
        assert!(wall.macro_.is_none());
        // v1: no prop model anywhere.
        assert!(wall.model.is_none());
        assert_eq!(t.macro_size, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn write_fixture(dir: &Path, materials_json: &str, with_macro_png: bool) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("blocks.json"),
            r#"{"format":"png-layers","tile_size":32,"layer_count":3}"#,
        )
        .unwrap();
        std::fs::write(dir.join("materials.json"), materials_json).unwrap();
        if with_macro_png {
            let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([10, 20, 30, 255]));
            img.save(dir.join("macro_0000_color.png")).unwrap();
            let h = image::GrayImage::from_pixel(4, 4, image::Luma([77]));
            h.save(dir.join("macro_0001_height.png")).unwrap();
        }
    }

    const ONE_MAT: &str = r#"{"id":"t_grass","resolved_id":"t_grass","resolution":"exact","material_index":0,"top_layer":0,"side_layer":1,"bottom_layer":2,"height_voxels":1,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":false,"subtile_layers":{}}"#;

    #[test]
    fn loads_v2_with_macros() {
        let dir = std::env::temp_dir().join(format!("engine_mat_v2_{}", std::process::id()));
        let json = format!(
            r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[{{"material_index":0,"color_layer":0,"height_layer":1,"span_tiles":4}}]}}"#
        );
        write_fixture(&dir, &json, true);
        let t = MaterialTable::load(&dir).unwrap();
        assert_eq!(t.len(), 1);
        let grass = t.get("t_grass").unwrap();
        assert_eq!(
            grass.macro_,
            Some(MacroEntry {
                color_layer: 0,
                height_layer: 1,
                span_tiles: 4
            })
        );
        // Macro size is read from the actual PNG (4px fixture).
        assert_eq!(t.macro_size, 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_v2_with_models() {
        let dir = std::env::temp_dir().join(format!("engine_mat_mdl_{}", std::process::id()));
        // size [2,1,2]: (y,z,x) voxel order — z=0 row: palette 0,
        // EMPTY; z=1 row: palette 1, palette 0.
        let model = r#"{"material_index":0,"size":[2,1,2],"palette":[[10,20,30],[200,210,220]],"voxels":"0000ffff00010000"}"#;
        let json = format!(
            r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[],"models":[{model}]}}"#
        );
        write_fixture(&dir, &json, false);
        let t = MaterialTable::load(&dir).unwrap();
        let m = t
            .get("t_grass")
            .unwrap()
            .model
            .as_ref()
            .expect("model joined by material_index");
        assert_eq!(m.size, [2, 1, 2]);
        assert_eq!(m.get(0, 0, 0), Some([10, 20, 30]));
        assert_eq!(m.get(1, 0, 0), None, "EMPTY marker decodes to no voxel");
        assert_eq!(m.get(0, 0, 1), Some([200, 210, 220]));
        assert_eq!(m.get(1, 0, 1), Some([10, 20, 30]));
        assert_eq!(m.get(2, 0, 0), None, "out of bounds is empty");
        assert_eq!(m.solid_count(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn v2_without_models_loads_model_less() {
        let dir = std::env::temp_dir().join(format!("engine_mat_nomdl_{}", std::process::id()));
        let json = format!(r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[]}}"#);
        write_fixture(&dir, &json, false);
        let t = MaterialTable::load(&dir).unwrap();
        assert!(t.get("t_grass").unwrap().model.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_corrupt_model() {
        let dir = std::env::temp_dir().join(format!("engine_mat_badmdl_{}", std::process::id()));
        // Hex length (8 digits = 2 voxels) disagrees with size (4 voxels).
        let model =
            r#"{"material_index":0,"size":[2,1,2],"palette":[[10,20,30]],"voxels":"00000000"}"#;
        let json =
            format!(r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[],"models":[{model}]}}"#);
        write_fixture(&dir, &json, false);
        let err = MaterialTable::load(&dir).unwrap_err();
        assert!(err.contains("models entry"), "got: {err}");
        // Palette index 7 with a 1-color palette is out of range.
        let model = r#"{"material_index":0,"size":[2,1,2],"palette":[[10,20,30]],"voxels":"0007000000000000"}"#;
        let json =
            format!(r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[],"models":[{model}]}}"#);
        write_fixture(&dir, &json, false);
        let err = MaterialTable::load(&dir).unwrap_err();
        assert!(err.contains("out of range"), "got: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_v2_with_shores() {
        let dir = std::env::temp_dir().join(format!("engine_mat_shore_{}", std::process::id()));
        let shores =
            r#"[{"material_index":0,"mask":0,"layer":0},{"material_index":0,"mask":28,"layer":1}]"#;
        let json =
            format!(r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[],"shores":{shores}}}"#);
        write_fixture(&dir, &json, false);
        for layer in 0..2 {
            let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]));
            img.save(dir.join(format!("shore_{layer:04}.png"))).unwrap();
        }
        let t = MaterialTable::load(&dir).unwrap();
        assert_eq!(t.shore_size, 4, "shore size read from the first shore PNG");
        let grass = t.get("t_grass").unwrap();
        assert_eq!(grass.shore_top_layer(0), Some(0));
        assert_eq!(grass.shore_top_layer(28), Some(1));
        assert_eq!(
            grass.shore_top_layer(7),
            None,
            "unlisted mask keeps plain top"
        );
        assert_eq!(
            grass.shore_top_layer(0xFF),
            None,
            "all-water interior keeps plain top by rule"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn v2_without_shores_loads_shore_less() {
        let dir = std::env::temp_dir().join(format!("engine_mat_noshore_{}", std::process::id()));
        let json = format!(r#"{{"version":2,"materials":[{ONE_MAT}],"macros":[]}}"#);
        write_fixture(&dir, &json, false);
        let t = MaterialTable::load(&dir).unwrap();
        assert_eq!(t.shore_size, 0);
        assert!(t.get("t_grass").unwrap().shore_layers.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_unknown_version() {
        let dir = std::env::temp_dir().join(format!("engine_mat_v3_{}", std::process::id()));
        let json = format!(r#"{{"version":3,"materials":[{ONE_MAT}],"macros":[]}}"#);
        write_fixture(&dir, &json, false);
        let err = MaterialTable::load(&dir).unwrap_err();
        assert!(err.contains("unsupported version 3"), "got: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
