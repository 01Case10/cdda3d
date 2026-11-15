//! Tileset loading for the CDDA Voxel Frontend (plan §8).
//!
//! Two input forms are supported:
//! * **Decomposed** — per-sprite PNGs + per-sprite tile-entry JSON under
//!   `pngs_*/**`, plus `tile_info.json` (per-sheet sprite size + offsets)
//!   and `tileset.txt`. This is the form checked into CDDA-Tilesets.
//! * **Composed** — a single sheet PNG + `tile_config.json` produced by
//!   CDDA's `tools/gfx_tools/compose.py`, where `fg`/`bg` are sprite
//!   *indices* into the sheet named by each section's `file`.
//!
//! Sprite resolution follows the game's chain in `cata_tiles`
//! (`find_tile_looks_like` / `find_tile_with_season`): exact → variant
//! (`<id>_var_<variant>`) → season suffix (`<id>_season_<season>`) →
//! `looks_like` (recursive, jump-limited) → filler. Nothing is declared
//! missing until every stage has failed.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub mod resolve;
pub use resolve::{resolve, Resolution, Stage};

/// Per-sheet sprite geometry from `tile_info.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetInfo {
    pub file: String,
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub sprite_offset_x: i32,
    pub sprite_offset_y: i32,
    pub sprites_across: Option<u32>,
    pub source: Option<String>,
    pub filler: bool,
    pub fallback: bool,
}

/// One weighted sprite choice. In decomposed entries `sprite` is a name;
/// in composed entries it is an index into the section's sheet.
#[derive(Debug, Clone, PartialEq)]
pub enum SpriteRef {
    Name(String),
    Index(u32),
    WeightedName {
        sprite: String,
        weight: u32,
    },
    WeightedIndex {
        sprite: u32,
        weight: u32,
    },
    /// A nested list (the game allows lists inside lists in fg/bg).
    List(Vec<SpriteRef>),
}

impl SpriteRef {
    /// Flatten to the deterministic primary choice: highest weight,
    /// ties broken by list order (the game's weighted pick is random at
    /// runtime; the voxelizer must be deterministic, so it bakes the
    /// primary variant and records the rest in the entry).
    pub fn primary(&self) -> Option<SpriteRef> {
        match self {
            SpriteRef::Name(_) | SpriteRef::Index(_) => Some(self.clone()),
            SpriteRef::WeightedName { sprite, .. } => Some(SpriteRef::Name(sprite.clone())),
            SpriteRef::WeightedIndex { sprite, .. } => Some(SpriteRef::Index(*sprite)),
            SpriteRef::List(items) => {
                let mut best: Option<(&SpriteRef, u32)> = None;
                for it in items {
                    let w = it.weight();
                    if best.map(|(_, bw)| w > bw).unwrap_or(true) {
                        best = Some((it, w));
                    }
                }
                best.and_then(|(r, _)| r.primary())
            }
        }
    }
    pub fn weight(&self) -> u32 {
        match self {
            SpriteRef::WeightedName { weight, .. } | SpriteRef::WeightedIndex { weight, .. } => {
                *weight
            }
            _ => 1,
        }
    }
    pub fn name(&self) -> Option<&str> {
        match self {
            SpriteRef::Name(n) => Some(n),
            SpriteRef::WeightedName { sprite, .. } => Some(sprite),
            _ => None,
        }
    }
    pub fn index(&self) -> Option<u32> {
        match self {
            SpriteRef::Index(i) => Some(*i),
            SpriteRef::WeightedIndex { sprite, .. } => Some(*sprite),
            _ => None,
        }
    }
    /// All leaf sprite names (for coverage / atlas collection).
    pub fn all_names(&self, out: &mut Vec<String>) {
        match self {
            SpriteRef::Name(n) => out.push(n.clone()),
            SpriteRef::WeightedName { sprite, .. } => out.push(sprite.clone()),
            SpriteRef::List(items) => {
                for it in items {
                    it.all_names(out);
                }
            }
            _ => {}
        }
    }
}

/// Parse an fg/bg value in either form (names or indices).
fn parse_sprite_ref(v: &serde_json::Value) -> Option<SpriteRef> {
    match v {
        serde_json::Value::String(s) => {
            if s.is_empty() {
                None
            } else {
                Some(SpriteRef::Name(s.clone()))
            }
        }
        serde_json::Value::Number(n) => n.as_u64().map(|i| SpriteRef::Index(i as u32)),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for it in items {
                if let Some(r) = parse_sprite_ref(it) {
                    out.push(r);
                }
            }
            if out.is_empty() {
                None
            } else if out.len() == 1 {
                out.into_iter().next()
            } else {
                Some(SpriteRef::List(out))
            }
        }
        serde_json::Value::Object(map) => {
            let sprite = map.get("sprite")?;
            let weight = map.get("weight").and_then(|w| w.as_u64()).unwrap_or(1) as u32;
            match sprite {
                serde_json::Value::String(s) => Some(SpriteRef::WeightedName {
                    sprite: s.clone(),
                    weight,
                }),
                serde_json::Value::Number(n) => Some(SpriteRef::WeightedIndex {
                    sprite: n.as_u64()? as u32,
                    weight,
                }),
                serde_json::Value::Array(_) => parse_sprite_ref(sprite),
                _ => None,
            }
        }
        _ => None,
    }
}

/// One tile entry (one game id, or several ids sharing a sprite).
#[derive(Debug, Clone)]
pub struct TileEntry {
    pub ids: Vec<String>,
    pub fg: Option<SpriteRef>,
    pub bg: Option<SpriteRef>,
    pub multitile: bool,
    pub rotates: bool,
    /// subtile id ("center", "corner", ...) -> entry
    pub additional_tiles: BTreeMap<String, TileEntry>,
    /// Sheet file this entry's indices refer to (composed form only).
    pub sheet_file: Option<String>,
}

fn parse_entry(v: &serde_json::Value, sheet_file: Option<&str>) -> Option<TileEntry> {
    let obj = v.as_object()?;
    let ids: Vec<String> = match obj.get("id")? {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(a) => a
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect(),
        _ => return None,
    };
    if ids.is_empty() {
        return None;
    }
    let mut additional = BTreeMap::new();
    if let Some(serde_json::Value::Array(list)) = obj.get("additional_tiles") {
        for sub in list {
            if let Some(e) = parse_entry(sub, sheet_file) {
                for id in &e.ids {
                    additional.insert(id.clone(), e.clone());
                }
            }
        }
    }
    Some(TileEntry {
        ids,
        fg: obj.get("fg").and_then(parse_sprite_ref),
        bg: obj.get("bg").and_then(parse_sprite_ref),
        multitile: obj
            .get("multitile")
            .and_then(|b| b.as_bool())
            .unwrap_or(false),
        rotates: obj
            .get("rotates")
            .and_then(|b| b.as_bool())
            .unwrap_or(false),
        additional_tiles: additional,
        sheet_file: sheet_file.map(|s| s.to_string()),
    })
}

/// Geometry + provenance of one sprite.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteGeom {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub sheet_file: String,
    /// Decomposed form: path of the PNG file. Composed: None (use index).
    pub png_path: Option<PathBuf>,
}

/// A loaded tileset (either form).
#[derive(Debug, Default)]
pub struct Tileset {
    pub name: String,
    pub base_width: u32,
    pub base_height: u32,
    pub sheets: BTreeMap<String, SheetInfo>,
    /// id -> entry (sorted for determinism)
    pub entries: BTreeMap<String, TileEntry>,
    /// sprite name -> geometry (decomposed form)
    pub sprites_by_name: BTreeMap<String, SpriteGeom>,
    /// (sheet file, sprite index) -> geometry (composed form)
    pub sprites_by_index: BTreeMap<(String, u32), SpriteGeom>,
    /// Composed sheet images are loaded lazily by the voxelizer.
    pub root: PathBuf,
    pub composed: bool,
}

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Msg(String),
}
impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "io: {e}"),
            LoadError::Json(e) => write!(f, "json: {e}"),
            LoadError::Msg(m) => write!(f, "{m}"),
        }
    }
}
impl std::error::Error for LoadError {}
impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError::Io(e)
    }
}
impl From<serde_json::Error> for LoadError {
    fn from(e: serde_json::Error) -> Self {
        LoadError::Json(e)
    }
}

#[derive(Deserialize)]
struct SheetInfoRaw {
    sprite_width: Option<u32>,
    sprite_height: Option<u32>,
    sprite_offset_x: Option<i32>,
    sprite_offset_y: Option<i32>,
    sprites_across: Option<u32>,
    source: Option<String>,
    filler: Option<bool>,
    fallback: Option<bool>,
}

/// Parse `tile_info.json`: a list whose first element is the base
/// `{pixelscale,width,height}` and whose later elements each map one
/// sheet filename to its geometry.
pub fn parse_tile_info(text: &str) -> Result<(u32, u32, BTreeMap<String, SheetInfo>), LoadError> {
    let v: serde_json::Value = serde_json::from_str(text)?;
    let arr = v
        .as_array()
        .ok_or_else(|| LoadError::Msg("tile_info.json is not a list".into()))?;
    let mut base = (32u32, 32u32);
    let mut sheets = BTreeMap::new();
    for (i, item) in arr.iter().enumerate() {
        let obj = match item.as_object() {
            Some(o) => o,
            None => continue,
        };
        if i == 0 || obj.contains_key("pixelscale") {
            if let (Some(w), Some(h)) = (
                obj.get("width").and_then(|x| x.as_u64()),
                obj.get("height").and_then(|x| x.as_u64()),
            ) {
                base = (w as u32, h as u32);
            }
            continue;
        }
        for (file, raw) in obj {
            let r: SheetInfoRaw = serde_json::from_value(raw.clone())?;
            sheets.insert(
                file.clone(),
                SheetInfo {
                    file: file.clone(),
                    sprite_width: r.sprite_width.unwrap_or(base.0),
                    sprite_height: r.sprite_height.unwrap_or(base.1),
                    sprite_offset_x: r.sprite_offset_x.unwrap_or(0),
                    sprite_offset_y: r.sprite_offset_y.unwrap_or(0),
                    sprites_across: r.sprites_across,
                    source: r.source,
                    filler: r.filler.unwrap_or(false),
                    fallback: r.fallback.unwrap_or(false),
                },
            );
        }
    }
    Ok((base.0, base.1, sheets))
}

/// Map a decomposed `pngs_*` folder name to its sheet file:
/// `pngs_normal_32x32` -> `normal.png`, `pngs_human_body_plus_32x48` ->
/// `human_body_plus.png`, `pngs_fillergiant_96x96` -> `fillergiant.png`.
pub fn sheet_file_for_folder(folder: &str) -> Option<String> {
    let rest = folder.strip_prefix("pngs_")?;
    // strip a trailing _<w>x<h> suffix if present
    let base = match rest.rfind('_') {
        Some(pos) => {
            let suffix = &rest[pos + 1..];
            let is_size = suffix
                .split('x')
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
                && suffix.contains('x');
            if is_size {
                &rest[..pos]
            } else {
                rest
            }
        }
        None => rest,
    };
    Some(format!("{base}.png"))
}

fn collect_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().map(|x| x == ext).unwrap_or(false) {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
}

/// Load a tileset directory in decomposed form.
pub fn load_decomposed(root: &Path) -> Result<Tileset, LoadError> {
    let mut ts = Tileset {
        root: root.to_path_buf(),
        composed: false,
        ..Default::default()
    };
    // tileset.txt name
    if let Ok(txt) = std::fs::read_to_string(root.join("tileset.txt")) {
        for line in txt.lines() {
            if let Some(n) = line.trim().strip_prefix("NAME:") {
                ts.name = n.trim().to_string();
            }
        }
    }
    let info_path = root.join("tile_info.json");
    if info_path.exists() {
        let (bw, bh, sheets) = parse_tile_info(&std::fs::read_to_string(&info_path)?)?;
        ts.base_width = bw;
        ts.base_height = bh;
        ts.sheets = sheets;
    } else {
        ts.base_width = 32;
        ts.base_height = 32;
    }
    // Index PNGs by stem, remembering their sheet geometry.
    let mut pngs = Vec::new();
    collect_files(root, "png", &mut pngs);
    for p in &pngs {
        let stem = match p.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        // sheet folder = the pngs_* ancestor directory
        let mut sheet_file = String::new();
        let mut anc = p.parent();
        while let Some(d) = anc {
            if let Some(name) = d.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("pngs_") {
                    sheet_file = sheet_file_for_folder(name).unwrap_or_default();
                    break;
                }
            }
            anc = d.parent();
        }
        let (w, h, ox, oy) = match ts.sheets.get(&sheet_file) {
            Some(s) => (
                s.sprite_width,
                s.sprite_height,
                s.sprite_offset_x,
                s.sprite_offset_y,
            ),
            None => (ts.base_width, ts.base_height, 0, 0),
        };
        ts.sprites_by_name
            .entry(stem.clone())
            .or_insert(SpriteGeom {
                name: stem,
                width: w,
                height: h,
                offset_x: ox,
                offset_y: oy,
                sheet_file,
                png_path: Some(p.clone()),
            });
    }
    // Parse every decomposed tile-entry JSON.
    let mut jsons = Vec::new();
    collect_files(root, "json", &mut jsons);
    for p in jsons {
        if p.file_name()
            .map(|n| n == "tile_info.json")
            .unwrap_or(false)
            || p.file_name().map(|n| n == "layering.json").unwrap_or(false)
        {
            continue;
        }
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let items: Vec<&serde_json::Value> = match &v {
            serde_json::Value::Array(a) => a.iter().collect(),
            serde_json::Value::Object(_) => vec![&v],
            _ => continue,
        };
        for item in items {
            if let Some(entry) = parse_entry(item, None) {
                for id in &entry.ids {
                    ts.entries
                        .entry(id.clone())
                        .or_insert_with(|| entry.clone());
                }
            }
        }
    }
    Ok(ts)
}

/// Load a composed tileset: `tile_config.json` (+ its `tile_info`).
/// `sheet_pngs` maps sheet file -> decoded sheet image dimensions are
/// not needed here; sprite geometry comes from tile_info.
pub fn load_composed(config_path: &Path) -> Result<Tileset, LoadError> {
    let text = std::fs::read_to_string(config_path)?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    let mut ts = Tileset {
        root: config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default(),
        composed: true,
        ..Default::default()
    };
    if let Some(info) = v.get("tile_info") {
        let (bw, bh, sheets) = parse_tile_info(&info.to_string())?;
        ts.base_width = bw;
        ts.base_height = bh;
        ts.sheets = sheets;
    } else {
        ts.base_width = 32;
        ts.base_height = 32;
    }
    let sections = v
        .get("tiles-new")
        .and_then(|t| t.as_array())
        .ok_or_else(|| LoadError::Msg("tile_config.json has no tiles-new".into()))?;
    for section in sections {
        let file = section
            .get("file")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .to_string();
        let sheet = ts.sheets.get(&file).cloned();
        if let Some(tiles) = section.get("tiles").and_then(|t| t.as_array()) {
            for t in tiles {
                if let Some(entry) = parse_entry(t, Some(&file)) {
                    // Register index geometry for every referenced index.
                    let mut refs = Vec::new();
                    if let Some(fg) = &entry.fg {
                        collect_index_refs(fg, &mut refs);
                    }
                    if let Some(bg) = &entry.bg {
                        collect_index_refs(bg, &mut refs);
                    }
                    for (_, sub) in &entry.additional_tiles {
                        if let Some(fg) = &sub.fg {
                            collect_index_refs(fg, &mut refs);
                        }
                        if let Some(bg) = &sub.bg {
                            collect_index_refs(bg, &mut refs);
                        }
                    }
                    for idx in refs {
                        let (w, h, ox, oy) = match &sheet {
                            Some(s) => (
                                s.sprite_width,
                                s.sprite_height,
                                s.sprite_offset_x,
                                s.sprite_offset_y,
                            ),
                            None => (ts.base_width, ts.base_height, 0, 0),
                        };
                        ts.sprites_by_index
                            .entry((file.clone(), idx))
                            .or_insert(SpriteGeom {
                                name: format!("{file}#{idx}"),
                                width: w,
                                height: h,
                                offset_x: ox,
                                offset_y: oy,
                                sheet_file: file.clone(),
                                png_path: None,
                            });
                    }
                    for id in &entry.ids {
                        ts.entries
                            .entry(id.clone())
                            .or_insert_with(|| entry.clone());
                    }
                }
            }
        }
    }
    Ok(ts)
}

fn collect_index_refs(r: &SpriteRef, out: &mut Vec<u32>) {
    match r {
        SpriteRef::Index(i) => out.push(*i),
        SpriteRef::WeightedIndex { sprite, .. } => out.push(*sprite),
        SpriteRef::List(items) => {
            for it in items {
                collect_index_refs(it, out);
            }
        }
        _ => {}
    }
}

impl Tileset {
    /// Geometry for a sprite reference (primary choice).
    pub fn geom_for_ref(&self, r: &SpriteRef) -> Option<&SpriteGeom> {
        let p = r.primary()?;
        if let Some(n) = p.name() {
            self.sprites_by_name.get(n)
        } else if let Some(i) = p.index() {
            // sheet file comes from the entry; caller resolves via entry
            self.sprites_by_index
                .values()
                .find(|g| g.name.ends_with(&format!("#{i}")))
        } else {
            None
        }
    }
    pub fn geom_for_entry_ref(&self, entry: &TileEntry, r: &SpriteRef) -> Option<&SpriteGeom> {
        let p = r.primary()?;
        if let Some(n) = p.name() {
            self.sprites_by_name.get(n)
        } else if let Some(i) = p.index() {
            let file = entry.sheet_file.as_deref()?;
            self.sprites_by_index.get(&(file.to_string(), i))
        } else {
            None
        }
    }
}
