//! Scene loader (slice S1): parses the plan §3.4 scene format —
//! a small, deliberately strict TOML *subset*, hand-parsed (no
//! toml crate is in the workspace lock; adding one is out of
//! scope). Anything outside the subset is rejected with an error
//! naming the line.
//!
//! # The subset
//!
//! - Comments: `#` to end of line (outside strings).
//! - Values: strings (`"..."`, escapes `\"` `\\` `\n`), integers,
//!   floats, booleans, arrays (may span lines), and inline tables
//!   (`{ "k" = value, ... }`).
//! - Top-level keys: `name` (string), `seed` (int), `time`,
//!   `weather` (strings, informational in S1), `player`,
//!   `view_origin` (`[x, y, z]`), `view_radius` (int),
//!   `environment` (bool: piggyback the sample environment on the
//!   Snapshot — the Phase 10 fixtures predate the piggyback and
//!   omit it, the stub house carries it), `emit_outside` (bool:
//!   compute the `outside[]` array with the S1 enclosure rule;
//!   fixtures that predate Phase 10 leave it off and send no
//!   `outside[]`, exactly like the samples).
//! - Per-z sections `[z0]`, `[z1]`, `[z-1]`, each with:
//!   - `legend = { "#" = "t_wall", ... }` — char -> string id.
//!   - `map = [ "row", ... ]` — the TERRAIN layer. S1 scenes are
//!     exactly one submap: 12 rows of 12 chars. Every char must
//!     appear in the legend and resolve to a Terrain def.
//!   - `furn = [ "row", ... ]` (optional) — the FURNITURE layer,
//!     a separate layer because the wire carries furniture as its
//!     own array: `'.'` always means "no furniture"; any other
//!     char must appear in the legend and resolve to a Furniture
//!     def. (There is no paired-terrain guessing: the terrain
//!     under a furniture tile is whatever `map` says.)
//!   - `trap = [ "row", ... ]` (optional) — same shape, Trap defs,
//!     `'.'` = none.
//! - Array tables (top-level, after any sections; `pos` is always
//!   an absolute `[x, y, z]`):
//!   - `[[spawn]]`: `type` (a Monster string id, or the special
//!     `"player"` — type_def 0, sprite `"player"`, like the real
//!     adapter's avatar), `pos`, `hp_frac`, optional `facing`
//!     (`"NONE"`/`"LEFT"`/`"RIGHT"`, default NONE), `overlays`
//!     (string array, game order — never re-sorted), `flags`,
//!     `sprite` (override). Entities are emitted in file order;
//!     ids are assigned 1.. in that order, so author the player
//!     first.
//!   - `[[field]]`: `pos`, `type` (Field string id), `intensity`.
//!     File order is the wire order (a tile's entries stay
//!     contiguous in the block's field list).
//!   - `[[items]]`: `pos`, `top` (Item string id), `count`.
//!   - `[[graffiti]]`: `pos`.
//!   - `[[vehicle]]`: `id`, `name`, `face_deg` (float, continuous —
//!     never quantised).
//!   - `[[vpart]]`: `vehicle` (id of a `[[vehicle]]`), `type`
//!     (VehiclePart string id), `pos` (game-computed absolute),
//!     `mount`, `hp_frac`, `flags`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const SUBMAP: usize = 12;

// ---------------------------------------------------------------------
// Parsed scene
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ZSection {
    pub z: i32,
    pub legend: HashMap<char, String>,
    pub map: Vec<String>,
    pub furn: Option<Vec<String>>,
    pub trap: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Spawn {
    pub type_id: String,
    pub pos: [i32; 3],
    pub hp_frac: u8,
    pub facing: String,
    pub overlays: Vec<String>,
    pub flags: u32,
    pub sprite: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldPlace {
    pub pos: [i32; 3],
    pub type_id: String,
    pub intensity: u8,
}

#[derive(Debug, Clone)]
pub struct ItemsPlace {
    pub pos: [i32; 3],
    pub top: String,
    pub count: u16,
}

#[derive(Debug, Clone)]
pub struct VehicleSpec {
    pub id: u64,
    pub name: String,
    pub face_deg: f32,
}

#[derive(Debug, Clone)]
pub struct VPartSpec {
    pub vehicle: u64,
    pub type_id: String,
    pub pos: [i32; 3],
    pub mount: [i32; 3],
    pub hp_frac: u8,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    pub seed: i64,
    pub time: String,
    pub weather: String,
    pub player: Option<[i32; 3]>,
    pub view_origin: [i32; 3],
    pub view_radius: u32,
    pub environment: bool,
    pub emit_outside: bool,
    pub zsections: Vec<ZSection>,
    pub spawns: Vec<Spawn>,
    pub fields: Vec<FieldPlace>,
    pub items: Vec<ItemsPlace>,
    pub graffiti: Vec<[i32; 3]>,
    pub vehicles: Vec<VehicleSpec>,
    pub vparts: Vec<VPartSpec>,
}

impl Scene {
    pub fn load_file(path: &Path) -> Result<Scene, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read scene {}: {e}", path.display()))?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Scene, String> {
        let doc = Parser::parse(text)?;
        doc.into_scene()
    }
}

/// Resolve a scene name (or explicit path) to a file. Search
/// order: the argument as a path; `$CDDA_SIM_SCENE_DIR/<name>.toml`;
/// `crates/sim/scenes/<name>.toml` relative to CWD. The error
/// names everything tried.
pub fn resolve_scene(name_or_path: &str) -> Result<PathBuf, String> {
    let mut tried: Vec<String> = Vec::new();
    let direct = PathBuf::from(name_or_path);
    tried.push(format!("path {}", direct.display()));
    if direct.is_file() {
        return Ok(direct);
    }
    let file_name = if name_or_path.ends_with(".toml") {
        name_or_path.to_string()
    } else {
        format!("{name_or_path}.toml")
    };
    if let Ok(dir) = std::env::var("CDDA_SIM_SCENE_DIR") {
        let p = PathBuf::from(&dir).join(&file_name);
        tried.push(format!("$CDDA_SIM_SCENE_DIR/{}", file_name));
        if p.is_file() {
            return Ok(p);
        }
    } else {
        tried.push("$CDDA_SIM_SCENE_DIR (unset)".to_string());
    }
    let rel = PathBuf::from("crates/sim/scenes").join(&file_name);
    tried.push(format!("CWD-relative {}", rel.display()));
    if rel.is_file() {
        return Ok(rel);
    }
    Err(format!(
        "scene '{name_or_path}' not found; tried: {}",
        tried.join("; ")
    ))
}

// ---------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Str(String),
    Int(i64),
    Float(f64),
    Ident(String),
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Eq,
    Comma,
    Newline,
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    line: usize,
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let err = |line: usize, msg: &str| format!("scene parse error at line {line}: {msg}");
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\n' => {
                out.push(Token {
                    tok: Tok::Newline,
                    line,
                });
                line += 1;
                i += 1;
            }
            ' ' | '\t' | '\r' => i += 1,
            '#' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '[' => {
                out.push(Token {
                    tok: Tok::LBracket,
                    line,
                });
                i += 1;
            }
            ']' => {
                out.push(Token {
                    tok: Tok::RBracket,
                    line,
                });
                i += 1;
            }
            '{' => {
                out.push(Token {
                    tok: Tok::LBrace,
                    line,
                });
                i += 1;
            }
            '}' => {
                out.push(Token {
                    tok: Tok::RBrace,
                    line,
                });
                i += 1;
            }
            '=' => {
                out.push(Token { tok: Tok::Eq, line });
                i += 1;
            }
            ',' => {
                out.push(Token {
                    tok: Tok::Comma,
                    line,
                });
                i += 1;
            }
            '"' => {
                let start_line = line;
                i += 1;
                let mut s = String::new();
                loop {
                    if i >= chars.len() {
                        return Err(err(start_line, "unterminated string"));
                    }
                    match chars[i] {
                        '"' => {
                            i += 1;
                            break;
                        }
                        '\\' => {
                            i += 1;
                            if i >= chars.len() {
                                return Err(err(start_line, "unterminated escape"));
                            }
                            match chars[i] {
                                '"' => s.push('"'),
                                '\\' => s.push('\\'),
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                other => {
                                    return Err(err(
                                        start_line,
                                        &format!("unsupported escape \\{other}"),
                                    ))
                                }
                            }
                            i += 1;
                        }
                        '\n' => return Err(err(start_line, "newline in string")),
                        ch => {
                            s.push(ch);
                            i += 1;
                        }
                    }
                }
                out.push(Token {
                    tok: Tok::Str(s),
                    line: start_line,
                });
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '-')
                {
                    i += 1;
                }
                out.push(Token {
                    tok: Tok::Ident(chars[start..i].iter().collect()),
                    line,
                });
            }
            c if c.is_ascii_digit() || c == '-' || c == '+' => {
                let start = i;
                if c == '-' || c == '+' {
                    i += 1;
                }
                let mut is_float = false;
                while i < chars.len()
                    && (chars[i].is_ascii_digit()
                        || chars[i] == '.'
                        || chars[i] == 'e'
                        || chars[i] == 'E')
                {
                    if chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E' {
                        is_float = true;
                    }
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                if is_float {
                    let v: f64 = text
                        .parse()
                        .map_err(|_| err(line, &format!("bad number '{text}'")))?;
                    out.push(Token {
                        tok: Tok::Float(v),
                        line,
                    });
                } else {
                    let v: i64 = text
                        .parse()
                        .map_err(|_| err(line, &format!("bad integer '{text}'")))?;
                    out.push(Token {
                        tok: Tok::Int(v),
                        line,
                    });
                }
            }
            other => return Err(err(line, &format!("unexpected character '{other}'"))),
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------
// Parser: tokens -> raw document (ordered key/value maps)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Table(Vec<(String, Value)>),
}

type KvMap = Vec<(String, Value, usize)>; // (key, value, line)

#[derive(Debug, Default)]
struct RawDoc {
    top: KvMap,
    zsections: Vec<(i32, KvMap)>,
    tables: Vec<(String, KvMap)>, // (table name, entries) in file order
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    Top,
    Z(usize),
    Table(usize),
}

impl Parser {
    fn parse(text: &str) -> Result<RawDoc, String> {
        let toks = tokenize(text)?;
        let mut p = Parser { toks, pos: 0 };
        p.document()
    }

    fn err(&self, line: usize, msg: &str) -> String {
        format!("scene parse error at line {line}: {msg}")
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }

    fn peek2(&self) -> Option<&Tok> {
        self.toks.get(self.pos + 1).map(|t| &t.tok)
    }

    fn line(&self) -> usize {
        self.toks.get(self.pos).map_or(0, |t| t.line)
    }

    fn skip_newlines(&mut self) {
        while self.peek() == Some(&Tok::Newline) {
            self.pos += 1;
        }
    }

    fn document(&mut self) -> Result<RawDoc, String> {
        let mut doc = RawDoc::default();
        let mut ctx = Ctx::Top;
        loop {
            self.skip_newlines();
            match self.peek().cloned() {
                None => break,
                Some(Tok::LBracket) if self.peek2() == Some(&Tok::LBracket) => {
                    // [[name]] array table
                    let line = self.line();
                    self.pos += 2;
                    let name = match self.peek().cloned() {
                        Some(Tok::Ident(s)) => {
                            self.pos += 1;
                            s
                        }
                        _ => return Err(self.err(line, "expected table name after '[['")),
                    };
                    if self.peek() != Some(&Tok::RBracket) || self.peek2() != Some(&Tok::RBracket) {
                        return Err(self.err(line, "expected ']]' after table name"));
                    }
                    self.pos += 2;
                    doc.tables.push((name, Vec::new()));
                    ctx = Ctx::Table(doc.tables.len() - 1);
                }
                Some(Tok::LBracket) => {
                    // [zN] section
                    let line = self.line();
                    self.pos += 1;
                    let name = match self.peek().cloned() {
                        Some(Tok::Ident(s)) => {
                            self.pos += 1;
                            s
                        }
                        _ => return Err(self.err(line, "expected section name after '['")),
                    };
                    if self.peek() != Some(&Tok::RBracket) {
                        return Err(self.err(line, "expected ']' after section name"));
                    }
                    self.pos += 1;
                    let z = parse_z_section(&name)
                        .ok_or_else(|| self.err(line, &format!("unknown section '[{name}]' (only [z<integer>] sections are in the subset)")))?;
                    doc.zsections.push((z, Vec::new()));
                    ctx = Ctx::Z(doc.zsections.len() - 1);
                }
                Some(Tok::Ident(key)) => {
                    let line = self.line();
                    self.pos += 1;
                    if self.peek() != Some(&Tok::Eq) {
                        return Err(self.err(line, &format!("expected '=' after key '{key}'")));
                    }
                    self.pos += 1;
                    let value = self.value()?;
                    match ctx {
                        Ctx::Top => doc.top.push((key, value, line)),
                        Ctx::Z(i) => doc.zsections[i].1.push((key, value, line)),
                        Ctx::Table(i) => doc.tables[i].1.push((key, value, line)),
                    }
                }
                Some(other) => {
                    return Err(self.err(self.line(), &format!("unexpected token {other:?}")));
                }
            }
        }
        Ok(doc)
    }

    fn value(&mut self) -> Result<Value, String> {
        let line = self.line();
        match self.peek().cloned() {
            Some(Tok::Str(s)) => {
                self.pos += 1;
                Ok(Value::Str(s))
            }
            Some(Tok::Int(n)) => {
                self.pos += 1;
                Ok(Value::Int(n))
            }
            Some(Tok::Float(f)) => {
                self.pos += 1;
                Ok(Value::Float(f))
            }
            Some(Tok::Ident(s)) if s == "true" || s == "false" => {
                self.pos += 1;
                Ok(Value::Bool(s == "true"))
            }
            Some(Tok::LBracket) => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_newlines();
                    match self.peek() {
                        Some(Tok::RBracket) => {
                            self.pos += 1;
                            break;
                        }
                        Some(Tok::Comma) => {
                            self.pos += 1;
                        }
                        None => return Err(self.err(line, "unterminated array")),
                        _ => {
                            let v = self.value()?;
                            items.push(v);
                        }
                    }
                }
                Ok(Value::Array(items))
            }
            Some(Tok::LBrace) => {
                self.pos += 1;
                let mut entries = Vec::new();
                loop {
                    self.skip_newlines();
                    match self.peek().cloned() {
                        Some(Tok::RBrace) => {
                            self.pos += 1;
                            break;
                        }
                        Some(Tok::Comma) => {
                            self.pos += 1;
                        }
                        Some(Tok::Str(k)) => {
                            self.pos += 1;
                            if self.peek() != Some(&Tok::Eq) {
                                return Err(self.err(line, "expected '=' in inline table"));
                            }
                            self.pos += 1;
                            let v = self.value()?;
                            entries.push((k, v));
                        }
                        None => return Err(self.err(line, "unterminated inline table")),
                        other => {
                            return Err(self
                                .err(line, &format!("unexpected token {other:?} in inline table")))
                        }
                    }
                }
                Ok(Value::Table(entries))
            }
            other => Err(self.err(line, &format!("unexpected value token {other:?}"))),
        }
    }
}

fn parse_z_section(name: &str) -> Option<i32> {
    let rest = name.strip_prefix('z')?;
    if rest.is_empty() {
        return None;
    }
    rest.parse::<i32>().ok()
}

// ---------------------------------------------------------------------
// RawDoc -> Scene (typed extraction + validation)
// ---------------------------------------------------------------------

fn get<'a>(map: &'a KvMap, key: &str) -> Option<&'a Value> {
    map.iter()
        .rev()
        .find(|(k, _, _)| k == key)
        .map(|(_, v, _)| v)
}

fn req<'a>(map: &'a KvMap, key: &str, ctx: &str) -> Result<&'a Value, String> {
    get(map, key).ok_or_else(|| format!("scene: {ctx} is missing required key '{key}'"))
}

fn as_string(v: &Value, ctx: &str) -> Result<String, String> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        _ => Err(format!("scene: {ctx} must be a string")),
    }
}

fn as_int(v: &Value, ctx: &str) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        _ => Err(format!("scene: {ctx} must be an integer")),
    }
}

fn as_float(v: &Value, ctx: &str) -> Result<f64, String> {
    match v {
        Value::Float(f) => Ok(*f),
        Value::Int(n) => Ok(*n as f64),
        _ => Err(format!("scene: {ctx} must be a number")),
    }
}

fn as_bool(v: &Value, ctx: &str) -> Result<bool, String> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(format!("scene: {ctx} must be a boolean")),
    }
}

fn as_pos(v: &Value, ctx: &str) -> Result<[i32; 3], String> {
    match v {
        Value::Array(items) if items.len() == 3 => {
            let mut out = [0i32; 3];
            for (i, item) in items.iter().enumerate() {
                out[i] = as_int(item, ctx)? as i32;
            }
            Ok(out)
        }
        _ => Err(format!("scene: {ctx} must be [x, y, z]")),
    }
}

fn as_str_array(v: &Value, ctx: &str) -> Result<Vec<String>, String> {
    match v {
        Value::Array(items) => items.iter().map(|i| as_string(i, ctx)).collect(),
        _ => Err(format!("scene: {ctx} must be an array of strings")),
    }
}

fn opt_string(map: &KvMap, key: &str, ctx: &str) -> Result<Option<String>, String> {
    match get(map, key) {
        Some(v) => Ok(Some(as_string(v, &format!("{ctx}.{key}"))?)),
        None => Ok(None),
    }
}

fn opt_int(map: &KvMap, key: &str, ctx: &str) -> Result<Option<i64>, String> {
    match get(map, key) {
        Some(v) => Ok(Some(as_int(v, &format!("{ctx}.{key}"))?)),
        None => Ok(None),
    }
}

fn opt_bool(map: &KvMap, key: &str, ctx: &str) -> Result<Option<bool>, String> {
    match get(map, key) {
        Some(v) => Ok(Some(as_bool(v, &format!("{ctx}.{key}"))?)),
        None => Ok(None),
    }
}

fn opt_pos(map: &KvMap, key: &str, ctx: &str) -> Result<Option<[i32; 3]>, String> {
    match get(map, key) {
        Some(v) => Ok(Some(as_pos(v, &format!("{ctx}.{key}"))?)),
        None => Ok(None),
    }
}

impl RawDoc {
    fn into_scene(self) -> Result<Scene, String> {
        let top = &self.top;
        let name = opt_string(top, "name", "scene")?.unwrap_or_else(|| "unnamed".to_string());
        let seed = opt_int(top, "seed", "scene")?.unwrap_or(1);
        let time = opt_string(top, "time", "scene")?.unwrap_or_default();
        let weather = opt_string(top, "weather", "scene")?.unwrap_or_default();
        let player = opt_pos(top, "player", "scene")?;
        let view_origin = opt_pos(top, "view_origin", "scene")?
            .or(player)
            .unwrap_or([0, 0, 0]);
        let view_radius = opt_int(top, "view_radius", "scene")?.unwrap_or(60) as u32;
        let environment = opt_bool(top, "environment", "scene")?.unwrap_or(false);
        let emit_outside = opt_bool(top, "emit_outside", "scene")?.unwrap_or(false);

        let mut zsections = Vec::new();
        for (z, map) in &self.zsections {
            let ctx = format!("[z{z}]");
            let legend_v = req(map, "legend", &ctx)?;
            let mut legend: HashMap<char, String> = HashMap::new();
            match legend_v {
                Value::Table(entries) => {
                    for (k, v) in entries {
                        let mut chars = k.chars();
                        let c = chars
                            .next()
                            .ok_or_else(|| format!("scene: {ctx}.legend has an empty key"))?;
                        if chars.next().is_some() {
                            return Err(format!(
                                "scene: {ctx}.legend key '{k}' is not a single character"
                            ));
                        }
                        legend.insert(c, as_string(v, &format!("{ctx}.legend"))?);
                    }
                }
                _ => return Err(format!("scene: {ctx}.legend must be an inline table")),
            }
            let map_rows = as_str_array(req(map, "map", &ctx)?, &format!("{ctx}.map"))?;
            check_layer(&map_rows, &legend, &ctx, "map", false)?;
            let furn = match get(map, "furn") {
                Some(v) => {
                    let rows = as_str_array(v, &format!("{ctx}.furn"))?;
                    check_layer(&rows, &legend, &ctx, "furn", true)?;
                    Some(rows)
                }
                None => None,
            };
            let trap = match get(map, "trap") {
                Some(v) => {
                    let rows = as_str_array(v, &format!("{ctx}.trap"))?;
                    check_layer(&rows, &legend, &ctx, "trap", true)?;
                    Some(rows)
                }
                None => None,
            };
            zsections.push(ZSection {
                z: *z,
                legend,
                map: map_rows,
                furn,
                trap,
            });
        }
        if zsections.is_empty() {
            return Err("scene: no [zN] sections — a scene needs at least one map layer".into());
        }

        let mut spawns = Vec::new();
        let mut fields = Vec::new();
        let mut items = Vec::new();
        let mut graffiti = Vec::new();
        let mut vehicles = Vec::new();
        let mut vparts = Vec::new();
        for (tname, map) in &self.tables {
            match tname.as_str() {
                "spawn" => {
                    let ctx = "[[spawn]]";
                    spawns.push(Spawn {
                        type_id: as_string(req(map, "type", ctx)?, "spawn.type")?,
                        pos: as_pos(req(map, "pos", ctx)?, "spawn.pos")?,
                        hp_frac: opt_int(map, "hp_frac", ctx)?.unwrap_or(255) as u8,
                        facing: opt_string(map, "facing", ctx)?
                            .unwrap_or_else(|| "NONE".to_string()),
                        overlays: match get(map, "overlays") {
                            Some(v) => as_str_array(v, "spawn.overlays")?,
                            None => Vec::new(),
                        },
                        flags: opt_int(map, "flags", ctx)?.unwrap_or(0) as u32,
                        sprite: opt_string(map, "sprite", ctx)?,
                    });
                }
                "field" => {
                    let ctx = "[[field]]";
                    fields.push(FieldPlace {
                        pos: as_pos(req(map, "pos", ctx)?, "field.pos")?,
                        type_id: as_string(req(map, "type", ctx)?, "field.type")?,
                        intensity: opt_int(map, "intensity", ctx)?.unwrap_or(1) as u8,
                    });
                }
                "items" => {
                    let ctx = "[[items]]";
                    items.push(ItemsPlace {
                        pos: as_pos(req(map, "pos", ctx)?, "items.pos")?,
                        top: as_string(req(map, "top", ctx)?, "items.top")?,
                        count: opt_int(map, "count", ctx)?.unwrap_or(1) as u16,
                    });
                }
                "graffiti" => {
                    let ctx = "[[graffiti]]";
                    graffiti.push(as_pos(req(map, "pos", ctx)?, "graffiti.pos")?);
                }
                "vehicle" => {
                    let ctx = "[[vehicle]]";
                    vehicles.push(VehicleSpec {
                        id: opt_int(map, "id", ctx)?.unwrap_or(1) as u64,
                        name: opt_string(map, "name", ctx)?.unwrap_or_default(),
                        face_deg: match get(map, "face_deg") {
                            Some(v) => as_float(v, "vehicle.face_deg")? as f32,
                            None => 0.0,
                        },
                    });
                }
                "vpart" => {
                    let ctx = "[[vpart]]";
                    vparts.push(VPartSpec {
                        vehicle: opt_int(map, "vehicle", ctx)?.unwrap_or(1) as u64,
                        type_id: as_string(req(map, "type", ctx)?, "vpart.type")?,
                        pos: as_pos(req(map, "pos", ctx)?, "vpart.pos")?,
                        mount: opt_pos(map, "mount", ctx)?.unwrap_or([0, 0, 0]),
                        hp_frac: opt_int(map, "hp_frac", ctx)?.unwrap_or(255) as u8,
                        flags: opt_int(map, "flags", ctx)?.unwrap_or(0) as u32,
                    });
                }
                other => {
                    return Err(format!(
                        "scene: unknown array table [[{other}]] (subset has spawn/field/items/graffiti/vehicle/vpart)"
                    ));
                }
            }
        }

        Ok(Scene {
            name,
            seed,
            time,
            weather,
            player,
            view_origin,
            view_radius,
            environment,
            emit_outside,
            zsections,
            spawns,
            fields,
            items,
            graffiti,
            vehicles,
            vparts,
        })
    }
}

fn check_layer(
    rows: &[String],
    legend: &HashMap<char, String>,
    ctx: &str,
    layer: &str,
    dot_is_none: bool,
) -> Result<(), String> {
    if rows.len() != SUBMAP {
        return Err(format!(
            "scene: {ctx}.{layer} must have exactly {SUBMAP} rows (S1 scenes are one submap), got {}",
            rows.len()
        ));
    }
    for (y, row) in rows.iter().enumerate() {
        let chars: Vec<char> = row.chars().collect();
        if chars.len() != SUBMAP {
            return Err(format!(
                "scene: {ctx}.{layer} row {y} must be {SUBMAP} chars, got {}",
                chars.len()
            ));
        }
        for c in chars {
            if dot_is_none && c == '.' {
                continue;
            }
            if !legend.contains_key(&c) {
                return Err(format!(
                    "scene: {ctx}.{layer} uses char '{c}' which is not in the legend"
                ));
            }
        }
    }
    Ok(())
}
