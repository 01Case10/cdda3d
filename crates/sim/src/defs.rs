//! Def registry (slice S1): the sim's TileDefs table, loaded from
//! the S1a importer's generated table
//! (`tools/sim-defs/generated/defs.json`, 14,262 defs at the pinned
//! commit) and indexed both ways — `(kind, string_id) -> def` and
//! `(kind, id) -> def`.
//!
//! Wire ids are PER-KIND (the real adapter emits each kind's own
//! game int ids, which collide across kinds; context is the tile
//! field). The importer assigns them deterministically as sorted
//! 1..N within a kind; 0 always means "none".
//!
//! Path resolution for the defs file, in order:
//! 1. an explicit path (server `--defs`, or [`DefRegistry::load`]),
//! 2. the `CDDA_SIM_DEFS` environment variable,
//! 3. `tools/sim-defs/generated/defs.json` relative to the current
//!    working directory.
//! There is deliberately NO exe-relative guessing: if none of the
//! above finds a file, the error names everything that was tried.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use proto::{TileDefT, TileDefsT, TileKind, PROTOCOL_VERSION};
use serde::Deserialize;

/// Def kinds, matching the importer's `kind` ints and the schema's
/// `TileKind` enum order (schema/cdda_ipc.fbs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Terrain = 0,
    Furniture = 1,
    Trap = 2,
    Field = 3,
    Monster = 4,
    Item = 5,
    VehiclePart = 6,
}

impl Kind {
    pub fn from_int(n: i64) -> Option<Kind> {
        Some(match n {
            0 => Kind::Terrain,
            1 => Kind::Furniture,
            2 => Kind::Trap,
            3 => Kind::Field,
            4 => Kind::Monster,
            5 => Kind::Item,
            6 => Kind::VehiclePart,
            _ => return None,
        })
    }

    pub fn to_tile_kind(self) -> TileKind {
        match self {
            Kind::Terrain => TileKind::Terrain,
            Kind::Furniture => TileKind::Furniture,
            Kind::Trap => TileKind::Trap,
            Kind::Field => TileKind::Field,
            Kind::Monster => TileKind::Monster,
            Kind::Item => TileKind::Item,
            Kind::VehiclePart => TileKind::VehiclePart,
        }
    }
}

/// One def, as stored in the generated table (S1a). Mirrors the
/// wire `TileDef` field for field.
#[derive(Debug, Clone, Deserialize)]
pub struct Def {
    pub id: u32,
    pub string_id: String,
    #[serde(default)]
    pub name: String,
    pub kind: i64,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub hot_flags: u64,
    /// CDDA movement cost: terrain `move_cost` (0 = impassable),
    /// furniture `move_cost_mod` (negative = impassable). Present
    /// only for terrain/furniture in the generated table.
    /// Sim-internal — TileDef has no such field on the wire; the
    /// passability rule (rules.rs) mirrors the game's
    /// `map::move_cost_ter_furn`.
    #[serde(default)]
    pub move_cost: Option<i32>,
    #[serde(default)]
    pub connect_groups: Vec<u16>,
    #[serde(default)]
    pub connects_to_groups: Vec<u16>,
    #[serde(default)]
    pub rotate_to_groups: Vec<u16>,
    pub open_result: Option<String>,
    pub close_result: Option<String>,
    pub bash_result: Option<String>,
    pub looks_like: Option<String>,
    pub roof_result: Option<String>,
    #[serde(default = "minus_one")]
    pub fg_ref: i32,
    #[serde(default = "minus_one")]
    pub bg_ref: i32,
}

fn minus_one() -> i32 {
    -1
}

impl Def {
    pub fn kind_enum(&self) -> Kind {
        Kind::from_int(self.kind).expect("importer emits kinds 0..=6")
    }

    fn to_tile_def(&self) -> TileDefT {
        let non_empty = |v: &Vec<String>| {
            if v.is_empty() {
                None
            } else {
                Some(v.clone())
            }
        };
        let non_empty_u16 = |v: &Vec<u16>| {
            if v.is_empty() {
                None
            } else {
                Some(v.clone())
            }
        };
        // Generated table types are #[non_exhaustive] outside the
        // proto crate: build via Default + field assignment.
        let mut t = TileDefT::default();
        t.id = self.id;
        t.string_id = Some(self.string_id.clone());
        t.name = if self.name.is_empty() {
            None
        } else {
            Some(self.name.clone())
        };
        t.kind = self.kind_enum().to_tile_kind();
        t.flags = non_empty(&self.flags);
        t.hot_flags = self.hot_flags;
        t.connect_groups = non_empty_u16(&self.connect_groups);
        t.connects_to_groups = non_empty_u16(&self.connects_to_groups);
        t.rotate_to_groups = non_empty_u16(&self.rotate_to_groups);
        // The importer stores JSON null for "no transform"; an
        // empty string would mean the same on the wire (senders
        // use absent), so normalise both to None.
        t.open_result = self.open_result.clone().filter(|s| !s.is_empty());
        t.close_result = self.close_result.clone().filter(|s| !s.is_empty());
        t.bash_result = self.bash_result.clone().filter(|s| !s.is_empty());
        t.looks_like = self.looks_like.clone().filter(|s| !s.is_empty());
        t.fg_ref = self.fg_ref;
        t.bg_ref = self.bg_ref;
        t.roof_result = self.roof_result.clone().filter(|s| !s.is_empty());
        t
    }
}

#[derive(Debug, Deserialize)]
struct DefsFile {
    #[serde(default)]
    connect_groups: Vec<String>,
    defs: Vec<Def>,
}

/// The loaded registry: every def, indexed by (kind, string_id)
/// and by (kind, id), plus the connect-group registry table.
#[derive(Debug)]
pub struct DefRegistry {
    defs: Vec<Def>,
    by_string: HashMap<(Kind, String), usize>,
    by_id: HashMap<(Kind, u32), usize>,
    connect_groups: Vec<String>,
    /// The built wire table (built once at load; the server sends
    /// it verbatim at every connect).
    tiledefs: TileDefsT,
}

impl DefRegistry {
    /// Load from an explicit path.
    pub fn load(path: &Path) -> Result<DefRegistry, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read defs file {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    /// Resolve the defs path (explicit -> `CDDA_SIM_DEFS` ->
    /// CWD-relative default) and load. The error names what was
    /// tried.
    pub fn load_resolved(explicit: Option<&Path>) -> Result<DefRegistry, String> {
        let path = resolve_defs_path(explicit)?;
        Self::load(&path)
    }

    pub fn from_json(text: &str) -> Result<DefRegistry, String> {
        let file: DefsFile =
            serde_json::from_str(text).map_err(|e| format!("defs JSON parse failed: {e}"))?;
        let mut by_string = HashMap::with_capacity(file.defs.len());
        let mut by_id = HashMap::with_capacity(file.defs.len());
        for (i, d) in file.defs.iter().enumerate() {
            let kind = Kind::from_int(d.kind)
                .ok_or_else(|| format!("def {} has unknown kind {}", d.string_id, d.kind))?;
            if by_string.insert((kind, d.string_id.clone()), i).is_some() {
                return Err(format!(
                    "duplicate def (kind {kind:?}, string_id {})",
                    d.string_id
                ));
            }
            if by_id.insert((kind, d.id), i).is_some() {
                return Err(format!("duplicate def (kind {kind:?}, id {})", d.id));
            }
        }
        let mut tiledefs = TileDefsT::default();
        tiledefs.connect_groups = Some(file.connect_groups.clone());
        tiledefs.protocol_version = PROTOCOL_VERSION;
        tiledefs.defs = Some(file.defs.iter().map(|d| d.to_tile_def()).collect());
        Ok(DefRegistry {
            defs: file.defs,
            by_string,
            by_id,
            connect_groups: file.connect_groups,
            tiledefs,
        })
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn connect_groups(&self) -> &[String] {
        &self.connect_groups
    }

    /// The wire TileDefs message (all defs + the registry table).
    pub fn tiledefs(&self) -> &TileDefsT {
        &self.tiledefs
    }

    pub fn def_by_string(&self, kind: Kind, string_id: &str) -> Option<&Def> {
        self.by_string
            .get(&(kind, string_id.to_string()))
            .map(|&i| &self.defs[i])
    }

    pub fn def_by_id(&self, kind: Kind, id: u32) -> Option<&Def> {
        self.by_id.get(&(kind, id)).map(|&i| &self.defs[i])
    }

    /// Wire id for a string id within a kind; 0 if unknown.
    pub fn id_of(&self, kind: Kind, string_id: &str) -> u32 {
        self.def_by_string(kind, string_id).map_or(0, |d| d.id)
    }

    /// String id for a wire id within a kind (parity/test helper).
    pub fn string_of(&self, kind: Kind, id: u32) -> Option<&str> {
        self.def_by_id(kind, id).map(|d| d.string_id.as_str())
    }
}

/// The CWD-relative default location of the generated table.
pub const DEFAULT_DEFS_REL: &str = "tools/sim-defs/generated/defs.json";
/// Environment variable consulted between the explicit path and
/// the CWD-relative default.
pub const DEFS_ENV_VAR: &str = "CDDA_SIM_DEFS";

pub fn resolve_defs_path(explicit: Option<&Path>) -> Result<PathBuf, String> {
    let mut tried: Vec<String> = Vec::new();
    if let Some(p) = explicit {
        tried.push(format!("explicit path {}", p.display()));
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
    }
    if let Ok(env) = std::env::var(DEFS_ENV_VAR) {
        tried.push(format!("${DEFS_ENV_VAR}={env}"));
        let p = PathBuf::from(&env);
        if p.is_file() {
            return Ok(p);
        }
    } else {
        tried.push(format!("${DEFS_ENV_VAR} (unset)"));
    }
    let rel = PathBuf::from(DEFAULT_DEFS_REL);
    tried.push(format!("CWD-relative {}", rel.display()));
    if rel.is_file() {
        return Ok(rel);
    }
    Err(format!("defs file not found; tried: {}", tried.join("; ")))
}
