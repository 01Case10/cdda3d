//! Scenario DSL runner (slice S7; plan §5).
//!
//! A scenario is a TOML file in the scene parser's subset:
//! top-level `scene = "<name>"`, optional `name`, `seed`
//! (overrides the scene's seed) and `status` (`"s8"` marks a
//! file whose test belongs to S8 — written, not in the green
//! set), plus `steps = [ ... ]`, an array of inline tables.
//! Each step is either an action (`{ do = "...", ... }`) or an
//! assertion (`{ assert_* = ..., ... }`).
//!
//! Two driving modes, same world code (§3.1):
//!
//! - **In-process** ([`run_in_process`]): steps drive a
//!   [`World`] directly. Script-only world events (spawn,
//!   damage, vehicle mutation, weather, clock) are native.
//! - **Process** ([`run_process`]): steps drive a real
//!   `sim-server` subprocess over its Unix socket, speaking the
//!   framed Phase 1 wire through `proto` — the runner is a
//!   client, so world-event steps have no honest client
//!   equivalent and are REFUSED with a named error (author the
//!   event into the scene instead, as `t3_process.toml` does
//!   with a `[[spawn]]`). `disconnect` / `reconnect` / `ping`
//!   are native here; in-process they map to dropping and
//!   rebuilding the client view (mirror) from a fresh Snapshot,
//!   and a log-only no-op respectively. `snapshot` forces a
//!   resync Snapshot in-process; in process mode it is a
//!   disconnect + reconnect, whose fresh Snapshot must agree
//!   with the delta-fed mirror (the process-mode form of the
//!   mirror invariant).
//!
//! Settled step semantics (S7):
//!
//! - `bash` and `damage_tile` are the same step: tiles have no
//!   HP in the sim, so damaging a tile is one step along its
//!   bash chain via the S6 machinery (`World::bash_tile`), no
//!   player involved. (The arm-and-direction player form is
//!   exercised with `key` steps: `s` + a direction.)
//! - `damage_entity` deals `amount` damage to the first live
//!   entity of `type` through the S5 combat path
//!   (`World::damage_entity`) — death, corpse and all.
//! - Script-mutation steps (spawn / bash / damage_* / vehicle
//!   steps) flush a turn after applying (`World::flush_turn`),
//!   exactly as if the game itself had produced the event
//!   during a `do_turn`: the change rides that step's Delta,
//!   and fields/creatures tick once.
//! - `capture` is REJECTED at parse time: pixel capture is not
//!   available to the sim runner (the standing CaptureArgs
//!   limitation — captures take fixture names, not wire
//!   snapshots). Green-set scenarios must not use it.
//! - After every action step the runner checks the T12
//!   invariant in-process: a mirror fed ONLY the emitted
//!   message stream must equal a fresh Snapshot of the world.
//!   `assert_mirror` exists as an explicit marker step; the
//!   check runs regardless.
//!
//! Failure shape (§5): the run stops at the first unmet
//! assertion; the error carries the step number, the step
//! itself, and the message log up to that step.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use proto::samples::sample_hello_ack;
use proto::{
    decode_message, encode_message, FrameDecoder, FrameEncoder, InputKind, InputT, LitLevel,
    MessageT, PingT, ResizeT, ShutdownT, UiMode,
};

use crate::defs::{DefRegistry, Kind};
use crate::environment;
use crate::mirror::Mirror;
use crate::scenes::{self, Value};
use crate::world::World;
use crate::Scene;

// ---------------------------------------------------------------------
// Parsed scenario
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: Option<String>,
    pub scene: String,
    pub seed: Option<i64>,
    /// `None` / `"green"` = part of the green set; anything
    /// else (e.g. `"s8"`) = written but deferred to that slice.
    pub status: Option<String>,
    pub steps: Vec<Step>,
}

impl Scenario {
    pub fn is_green(&self) -> bool {
        matches!(self.status.as_deref(), None | Some("green"))
    }

    pub fn load_file(path: &Path) -> Result<Scenario, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read scenario {}: {e}", path.display()))?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Scenario, String> {
        let top =
            scenes::parse_top_level(text).map_err(|e| format!("scenario parse error: {e}"))?;
        let map: Vec<(String, Value)> = top.into_iter().map(|(k, v, _line)| (k, v)).collect();
        let scene = req_str(&map, "scene", "scenario")?;
        let name = opt_str(&map, "name");
        let seed = opt_int(&map, "seed");
        let status = opt_str(&map, "status");
        let steps_v = map
            .iter()
            .rev()
            .find(|(k, _)| k == "steps")
            .map(|(_, v)| v)
            .ok_or("scenario parse error: missing required key 'steps'")?;
        let items = match steps_v {
            Value::Array(items) => items,
            _ => return Err("scenario parse error: 'steps' must be an array".into()),
        };
        let mut steps = Vec::new();
        for (i, item) in items.iter().enumerate() {
            steps.push(parse_step(item, i + 1)?);
        }
        Ok(Scenario {
            name,
            scene,
            seed,
            status,
            steps,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    /// 1-based position in the file's step list.
    pub index: usize,
    pub kind: StepKind,
    /// Short human form for logs and failure dumps.
    pub desc: String,
}

#[derive(Debug, Clone)]
pub enum StepKind {
    Action(Action),
    Assert(Assert),
}

#[derive(Debug, Clone)]
pub enum Action {
    Key(i32),
    Text(String),
    Click {
        x: i32,
        y: i32,
    },
    Resize {
        width: u32,
        height: u32,
    },
    AdvanceTurns(u32),
    /// Seconds (from `minutes` or `seconds` in the file).
    AdvanceTime(i64),
    SetWeather(String),
    Spawn {
        type_id: String,
        pos: [i32; 3],
    },
    Bash {
        pos: [i32; 3],
    },
    DamageEntity {
        type_id: String,
        amount: i32,
    },
    DamageVehiclePart {
        vehicle: u64,
        part: String,
        mount: [i32; 3],
        amount: u8,
    },
    RotateVehicle {
        vehicle: u64,
        face_deg: f32,
    },
    TranslateVehicle {
        vehicle: u64,
        delta: [i32; 3],
    },
    RemoveVehiclePart {
        vehicle: u64,
        part: String,
        mount: [i32; 3],
    },
    Snapshot,
    Disconnect,
    Reconnect,
    Ping,
}

#[derive(Debug, Clone)]
pub enum Assert {
    Tile {
        pos: [i32; 3],
        terrain: Option<String>,
        furniture: Option<String>,
    },
    Player([i32; 3]),
    PlayerHp {
        lt: Option<i64>,
        gte: Option<i64>,
        eq: Option<i64>,
    },
    Outside {
        pos: [i32; 3],
        value: bool,
    },
    Lit {
        pos: [i32; 3],
        value: Option<LitLevel>,
        not_value: Option<LitLevel>,
    },
    Field {
        pos: [i32; 3],
        type_id: String,
        min_intensity: Option<u8>,
    },
    DisplayedField {
        pos: [i32; 3],
        type_id: String,
    },
    FieldCount {
        type_id: String,
        eq: Option<usize>,
        gte: Option<usize>,
    },
    Items {
        pos: [i32; 3],
        top: Option<String>,
        count: Option<u16>,
    },
    Carried {
        type_id: String,
        count: u16,
    },
    EntityVisible(String),
    EntityHidden(String),
    EntityAbsent(String),
    EntityAt {
        type_id: String,
        pos: [i32; 3],
    },
    UiMode(String),
    UiScreen(String),
    Env {
        weather: Option<String>,
        weather_id: Option<String>,
        sun_altitude_lt: Option<f64>,
        sun_altitude_gt: Option<f64>,
    },
    Message {
        kind: String,
        seq: Option<u64>,
        epoch: Option<u64>,
    },
    DeltaContainsTile {
        pos: [i32; 3],
        terrain: Option<String>,
        furniture: Option<String>,
    },
    DeltaContainsEntity {
        type_id: String,
    },
    DeltaContainsRemoved {
        type_id: String,
    },
    NoTileChange([i32; 3]),
    VehicleFace {
        vehicle: u64,
        face_deg: f32,
    },
    VehiclePart {
        vehicle: u64,
        part: String,
        mount: [i32; 3],
        pos: Option<[i32; 3]>,
        hp_frac: Option<u8>,
        broken: Option<bool>,
        roof: Option<bool>,
    },
    VehiclePartAbsent {
        vehicle: u64,
        part: String,
        mount: [i32; 3],
    },
    MirrorNow,
}

// ---------------------------------------------------------------------
// Step parsing (over the scene parser's Value tables)
// ---------------------------------------------------------------------

type Table = Vec<(String, Value)>;

fn tget<'a>(t: &'a Table, key: &str) -> Option<&'a Value> {
    t.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn tstr(t: &Table, key: &str, ctx: &str) -> Result<String, String> {
    match tget(t, key) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(_) => Err(format!("{ctx}: '{key}' must be a string")),
        None => Err(format!("{ctx}: missing required key '{key}'")),
    }
}

fn ostr(t: &Table, key: &str) -> Option<String> {
    match tget(t, key) {
        Some(Value::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

fn oint(t: &Table, key: &str) -> Option<i64> {
    match tget(t, key) {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    }
}

fn ofloat(t: &Table, key: &str) -> Option<f64> {
    match tget(t, key) {
        Some(Value::Float(f)) => Some(*f),
        Some(Value::Int(n)) => Some(*n as f64),
        _ => None,
    }
}

fn obool(t: &Table, key: &str) -> Option<bool> {
    match tget(t, key) {
        Some(Value::Bool(b)) => Some(*b),
        _ => None,
    }
}

fn tpos(t: &Table, key: &str, ctx: &str) -> Result<[i32; 3], String> {
    match tget(t, key) {
        Some(v) => scenes::as_pos(v, &format!("{ctx}.{key}")),
        None => Err(format!("{ctx}: missing required key '{key}'")),
    }
}

fn opos(t: &Table, key: &str) -> Option<[i32; 3]> {
    tget(t, key).and_then(|v| scenes::as_pos(v, key).ok())
}

fn req_str(map: &[(String, Value)], key: &str, ctx: &str) -> Result<String, String> {
    match map.iter().rev().find(|(k, _)| k == key) {
        Some((_, Value::Str(s))) => Ok(s.clone()),
        Some(_) => Err(format!("{ctx}: '{key}' must be a string")),
        None => Err(format!("{ctx}: missing required key '{key}'")),
    }
}

fn opt_str(map: &[(String, Value)], key: &str) -> Option<String> {
    match map.iter().rev().find(|(k, _)| k == key) {
        Some((_, Value::Str(s))) => Some(s.clone()),
        _ => None,
    }
}

fn opt_int(map: &[(String, Value)], key: &str) -> Option<i64> {
    match map.iter().rev().find(|(k, _)| k == key) {
        Some((_, Value::Int(n))) => Some(*n),
        _ => None,
    }
}

/// Resolve a scenario `key` name to a protocol key code: a
/// single character, or `"Esc"` (the engine's escape code,
/// ui-share `KEY_ESCAPE`).
fn key_code(name: &str, ctx: &str) -> Result<i32, String> {
    if name == "Esc" {
        return Ok(ui_share::input::KEY_ESCAPE);
    }
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c as i32),
        _ => Err(format!(
            "{ctx}: key must be a single character or \"Esc\", got '{name}'"
        )),
    }
}

fn lit_level(name: &str, ctx: &str) -> Result<LitLevel, String> {
    match name {
        "DARK" => Ok(LitLevel::DARK),
        "LOW" => Ok(LitLevel::LOW),
        "BRIGHT_ONLY" => Ok(LitLevel::BRIGHT_ONLY),
        "LIT" => Ok(LitLevel::LIT),
        "BRIGHT" => Ok(LitLevel::BRIGHT),
        "MEMORIZED" => Ok(LitLevel::MEMORIZED),
        "BLANK" => Ok(LitLevel::BLANK),
        _ => Err(format!("{ctx}: unknown lit level '{name}'")),
    }
}

fn parse_step(item: &Value, index: usize) -> Result<Step, String> {
    let ctx = format!("step {index}");
    let table = match item {
        Value::Table(entries) => entries.clone(),
        _ => return Err(format!("{ctx}: a step must be an inline table")),
    };
    if let Some(do_v) = tget(&table, "do") {
        let what = match do_v {
            Value::Str(s) => s.clone(),
            _ => return Err(format!("{ctx}: 'do' must be a string")),
        };
        let action = parse_action(&table, &what, &ctx)?;
        return Ok(Step {
            index,
            desc: format!("do {what}"),
            kind: StepKind::Action(action),
        });
    }
    let assert_keys: Vec<&String> = table
        .iter()
        .map(|(k, _)| k)
        .filter(|k| k.starts_with("assert_"))
        .collect();
    match assert_keys.len() {
        0 => Err(format!(
            "{ctx}: a step needs a 'do' key or exactly one assert_* key"
        )),
        1 => {
            let kind = assert_keys[0].clone();
            let assert = parse_assert(&table, &kind, &ctx)?;
            Ok(Step {
                index,
                desc: kind,
                kind: StepKind::Assert(assert),
            })
        }
        _ => Err(format!(
            "{ctx}: multiple assert_* keys in one step ({assert_keys:?})"
        )),
    }
}

fn parse_action(t: &Table, what: &str, ctx: &str) -> Result<Action, String> {
    match what {
        "key" => Ok(Action::Key(key_code(&tstr(t, "key", ctx)?, ctx)?)),
        "text" => Ok(Action::Text(tstr(t, "text", ctx)?)),
        "click" => {
            // UI-pixel clicks only: a tile click is an engine
            // pick (the engine owns picking, S4), so the sim
            // runner has no honest tile form — refuse it.
            if tget(t, "tile").is_some() {
                return Err(format!(
                    "{ctx}: click by tile is an engine pick, not a sim input; use x/y UI pixels"
                ));
            }
            let x = oint(t, "x").ok_or(format!("{ctx}: click needs 'x'"))? as i32;
            let y = oint(t, "y").ok_or(format!("{ctx}: click needs 'y'"))? as i32;
            Ok(Action::Click { x, y })
        }
        "resize" => {
            let width = oint(t, "width").ok_or(format!("{ctx}: resize needs 'width'"))? as u32;
            let height = oint(t, "height").ok_or(format!("{ctx}: resize needs 'height'"))? as u32;
            Ok(Action::Resize { width, height })
        }
        "advance_turns" => {
            let n = oint(t, "count")
                .or_else(|| oint(t, "turns"))
                .ok_or(format!("{ctx}: advance_turns needs 'count'"))?;
            Ok(Action::AdvanceTurns(n.max(0) as u32))
        }
        "advance_time" => {
            let seconds = match (oint(t, "minutes"), oint(t, "seconds")) {
                (Some(m), None) => m * 60,
                (None, Some(s)) => s,
                _ => {
                    return Err(format!(
                        "{ctx}: advance_time needs exactly one of 'minutes' / 'seconds'"
                    ))
                }
            };
            Ok(Action::AdvanceTime(seconds))
        }
        "set_weather" => Ok(Action::SetWeather(
            tstr(t, "name", ctx).or_else(|_| tstr(t, "weather", ctx))?,
        )),
        "spawn" => Ok(Action::Spawn {
            type_id: tstr(t, "type", ctx)?,
            pos: tpos(t, "pos", ctx)?,
        }),
        "bash" => Ok(Action::Bash {
            pos: tpos(t, "pos", ctx)?,
        }),
        "damage_tile" => Ok(Action::Bash {
            pos: tpos(t, "pos", ctx)?,
        }),
        "damage_entity" => Ok(Action::DamageEntity {
            type_id: tstr(t, "type", ctx)?,
            amount: oint(t, "amount").ok_or(format!("{ctx}: damage_entity needs 'amount'"))? as i32,
        }),
        "damage_vehicle_part" => Ok(Action::DamageVehiclePart {
            vehicle: oint(t, "vehicle").ok_or(format!("{ctx}: needs 'vehicle'"))? as u64,
            part: tstr(t, "part", ctx)?,
            mount: tpos(t, "mount", ctx)?,
            amount: oint(t, "amount").ok_or(format!("{ctx}: needs 'amount'"))? as u8,
        }),
        "rotate_vehicle" => Ok(Action::RotateVehicle {
            vehicle: oint(t, "vehicle").ok_or(format!("{ctx}: needs 'vehicle'"))? as u64,
            face_deg: ofloat(t, "face_deg")
                .ok_or(format!("{ctx}: rotate_vehicle needs 'face_deg'"))?
                as f32,
        }),
        "translate_vehicle" => Ok(Action::TranslateVehicle {
            vehicle: oint(t, "vehicle").ok_or(format!("{ctx}: needs 'vehicle'"))? as u64,
            delta: tpos(t, "delta", ctx)?,
        }),
        "remove_vehicle_part" => Ok(Action::RemoveVehiclePart {
            vehicle: oint(t, "vehicle").ok_or(format!("{ctx}: needs 'vehicle'"))? as u64,
            part: tstr(t, "part", ctx)?,
            mount: tpos(t, "mount", ctx)?,
        }),
        "snapshot" => Ok(Action::Snapshot),
        "disconnect" => Ok(Action::Disconnect),
        "reconnect" => Ok(Action::Reconnect),
        "ping" => Ok(Action::Ping),
        "capture" => Err(format!(
            "{ctx}: 'capture' is not available to the sim runner — pixel capture takes \
             fixture names, not wire snapshots (the standing CaptureArgs limitation); \
             green-set scenarios must not use it"
        )),
        other => Err(format!("{ctx}: unknown action '{other}'")),
    }
}

fn parse_assert(outer: &Table, kind: &str, ctx: &str) -> Result<Assert, String> {
    let raw = tget(outer, kind).expect("assert key present");
    // Two spellings, one meaning: the table form
    // `{ assert_x = { pos = [...], ... } }` (fields inside the
    // value) and the bare form `{ assert_x = [...] }` /
    // `= "..."` (the value is the payload; any fields are
    // step-level siblings). Normalize before the arms: `t` is
    // always the field table, `val` the position payload,
    // `sval` the string payload.
    let (t_owned, pos_v, str_v);
    match raw {
        Value::Table(entries) => {
            pos_v = tget(entries, "pos").cloned().unwrap_or(Value::Bool(true));
            str_v = tget(entries, "type")
                .or_else(|| tget(entries, "kind"))
                .cloned()
                .unwrap_or(Value::Bool(true));
            t_owned = entries.clone();
        }
        _ => {
            pos_v = raw.clone();
            str_v = raw.clone();
            t_owned = outer.clone();
        }
    }
    let t = &t_owned;
    let val = &pos_v;
    let sval = &str_v;
    match kind {
        "assert_tile" => {
            let pos = scenes::as_pos(val, ctx)?;
            Ok(Assert::Tile {
                pos,
                terrain: ostr(t, "terrain"),
                furniture: ostr(t, "furniture"),
            })
        }
        "assert_player" => Ok(Assert::Player(scenes::as_pos(val, ctx)?)),
        "assert_player_hp" => Ok(Assert::PlayerHp {
            lt: oint(t, "lt"),
            gte: oint(t, "gte"),
            eq: oint(t, "eq"),
        }),
        "assert_outside" => {
            let pos = scenes::as_pos(val, ctx)?;
            let value = obool(t, "value").ok_or(format!("{ctx}: assert_outside needs 'value'"))?;
            Ok(Assert::Outside { pos, value })
        }
        "assert_lit" => {
            let pos = scenes::as_pos(val, ctx)?;
            let value = match ostr(t, "value") {
                Some(s) => Some(lit_level(&s, ctx)?),
                None => None,
            };
            let not_value = match ostr(t, "not_value") {
                Some(s) => Some(lit_level(&s, ctx)?),
                None => None,
            };
            if value.is_none() && not_value.is_none() {
                return Err(format!("{ctx}: assert_lit needs 'value' or 'not_value'"));
            }
            Ok(Assert::Lit {
                pos,
                value,
                not_value,
            })
        }
        "assert_field" => {
            let pos = scenes::as_pos(val, ctx)?;
            Ok(Assert::Field {
                pos,
                type_id: tstr(t, "type", ctx)?,
                min_intensity: oint(t, "min_intensity")
                    .or_else(|| oint(t, "intensity_gte"))
                    .map(|n| n as u8),
            })
        }
        "assert_displayed" => {
            let pos = scenes::as_pos(val, ctx)?;
            Ok(Assert::DisplayedField {
                pos,
                type_id: tstr(t, "type", ctx)?,
            })
        }
        "assert_field_count" => {
            let type_id = match sval {
                Value::Str(s) => s.clone(),
                _ => {
                    return Err(format!(
                        "{ctx}: assert_field_count takes a field type string"
                    ))
                }
            };
            Ok(Assert::FieldCount {
                type_id,
                eq: oint(t, "eq").map(|n| n as usize),
                gte: oint(t, "gte").map(|n| n as usize),
            })
        }
        "assert_items" => {
            let pos = scenes::as_pos(val, ctx)?;
            Ok(Assert::Items {
                pos,
                top: ostr(t, "top"),
                count: oint(t, "count").map(|n| n as u16),
            })
        }
        "assert_carried" => {
            let type_id = tstr(t, "type", ctx)?;
            let count = oint(t, "count").ok_or(format!("{ctx}: assert_carried needs 'count'"))?;
            Ok(Assert::Carried {
                type_id,
                count: count as u16,
            })
        }
        "assert_entity_visible" => Ok(Assert::EntityVisible(tstr_from(val, ctx)?)),
        "assert_entity_hidden" => Ok(Assert::EntityHidden(tstr_from(val, ctx)?)),
        "assert_entity_absent" => Ok(Assert::EntityAbsent(tstr_from(val, ctx)?)),
        "assert_entity_at" => {
            let pos = scenes::as_pos(val, ctx)?;
            Ok(Assert::EntityAt {
                type_id: tstr(t, "type", ctx)?,
                pos,
            })
        }
        "assert_ui_mode" => Ok(Assert::UiMode(tstr_from(val, ctx)?)),
        "assert_ui_screen" => Ok(Assert::UiScreen(tstr_from(val, ctx)?)),
        "assert_env" => Ok(Assert::Env {
            weather: ostr(t, "weather"),
            weather_id: ostr(t, "weather_id"),
            sun_altitude_lt: ofloat(t, "sun_altitude_lt"),
            sun_altitude_gt: ofloat(t, "sun_altitude_gt"),
        }),
        "assert_message" => {
            let kind_name = tstr_from(sval, ctx)?;
            Ok(Assert::Message {
                kind: kind_name,
                seq: oint(t, "seq").map(|n| n as u64),
                epoch: oint(t, "epoch").map(|n| n as u64),
            })
        }
        "assert_delta_contains" => {
            if let Some(entity) = ostr(t, "entity") {
                return Ok(Assert::DeltaContainsEntity { type_id: entity });
            }
            if let Some(removed) = ostr(t, "removed") {
                return Ok(Assert::DeltaContainsRemoved { type_id: removed });
            }
            let pos = scenes::as_pos(val, ctx)?;
            Ok(Assert::DeltaContainsTile {
                pos,
                terrain: ostr(t, "terrain"),
                furniture: ostr(t, "furniture"),
            })
        }
        "assert_no_tile_change" => Ok(Assert::NoTileChange(scenes::as_pos(val, ctx)?)),
        "assert_vehicle_face" => Ok(Assert::VehicleFace {
            vehicle: oint(t, "vehicle")
                .ok_or(format!("{ctx}: assert_vehicle_face needs 'vehicle'"))?
                as u64,
            face_deg: ofloat(t, "face_deg")
                .ok_or(format!("{ctx}: assert_vehicle_face needs 'face_deg'"))?
                as f32,
        }),
        "assert_vehicle_part" => Ok(Assert::VehiclePart {
            vehicle: oint(t, "vehicle")
                .ok_or(format!("{ctx}: assert_vehicle_part needs 'vehicle'"))?
                as u64,
            part: tstr(t, "part", ctx)?,
            mount: tpos(t, "mount", ctx)?,
            pos: opos(t, "pos"),
            hp_frac: oint(t, "hp_frac").map(|n| n as u8),
            broken: obool(t, "broken"),
            roof: obool(t, "roof"),
        }),
        "assert_vehicle_part_absent" => Ok(Assert::VehiclePartAbsent {
            vehicle: oint(t, "vehicle")
                .ok_or(format!("{ctx}: assert_vehicle_part_absent needs 'vehicle'"))?
                as u64,
            part: tstr(t, "part", ctx)?,
            mount: tpos(t, "mount", ctx)?,
        }),
        "assert_mirror" => Ok(Assert::MirrorNow),
        other => Err(format!("{ctx}: unknown assertion '{other}'")),
    }
}

fn tstr_from(v: &Value, ctx: &str) -> Result<String, String> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        _ => Err(format!("{ctx}: expected a string value")),
    }
}

// ---------------------------------------------------------------------
// Run outcome / failure
// ---------------------------------------------------------------------

#[derive(Debug)]
pub struct RunOutcome {
    pub steps_executed: usize,
    /// One line per step (message summaries), for the report.
    pub log: Vec<String>,
}

#[derive(Debug)]
pub struct ScenarioFailure {
    pub step_index: usize,
    pub step_desc: String,
    pub error: String,
    pub log: Vec<String>,
}

impl std::fmt::Display for ScenarioFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "scenario failed at step {} ({}): {}",
            self.step_index, self.step_desc, self.error
        )?;
        writeln!(f, "--- message log up to that step ---")?;
        for line in &self.log {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ScenarioFailure {}

/// A short wire summary of one message for the log.
fn describe(msg: &MessageT) -> String {
    match msg {
        MessageT::Hello(h) => format!("Hello v{}", h.protocol_version),
        MessageT::HelloAck(_) => "HelloAck".to_string(),
        MessageT::TileDefs(td) => format!(
            "TileDefs ({} defs)",
            td.defs.as_ref().map_or(0, |d| d.len())
        ),
        MessageT::Snapshot(s) => format!("Snapshot seq={} epoch={}", s.seq, s.epoch),
        MessageT::Delta(d) => format!(
            "Delta seq={} epoch={} ({} tile changes, {} entity updates, {} removed)",
            d.seq,
            d.epoch,
            d.tile_changes.as_ref().map_or(0, |v| v.len()),
            d.entity_updates.as_ref().map_or(0, |v| v.len()),
            d.entity_removed.as_ref().map_or(0, |v| v.len())
        ),
        MessageT::LightUpdate(lu) => format!("LightUpdate seq={} epoch={}", lu.seq, lu.epoch),
        MessageT::EnvironmentUpdate(e) => format!(
            "EnvironmentUpdate seq={} turn={} weather={:?}",
            e.seq,
            e.turn,
            e.weather_id.as_deref().unwrap_or("?")
        ),
        MessageT::Effects(fx) => format!(
            "Effects seq={} ({} events)",
            fx.seq,
            fx.events.as_ref().map_or(0, |v| v.len())
        ),
        MessageT::UiFrame(fr) => format!("UiFrame seq={} mode={:?}", fr.seq, fr.mode),
        MessageT::Ping(p) => format!("Ping nonce={}", p.nonce),
        MessageT::Pong(p) => format!("Pong nonce={}", p.nonce),
        MessageT::Input(_) => "Input".to_string(),
        MessageT::Resize(_) => "Resize".to_string(),
        MessageT::PickResult(_) => "PickResult".to_string(),
        MessageT::Shutdown(_) => "Shutdown".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// The (seq, epoch) a message carries, if it is part of the
/// world's sequenced stream (UiFrame has its own seq space;
/// transport messages carry none).
fn stream_seq(msg: &MessageT) -> Option<(u64, u64)> {
    match msg {
        MessageT::Snapshot(s) => Some((s.seq, s.epoch)),
        MessageT::Delta(d) => Some((d.seq, d.epoch)),
        MessageT::LightUpdate(lu) => Some((lu.seq, lu.epoch)),
        MessageT::EnvironmentUpdate(e) => Some((e.seq, e.epoch)),
        MessageT::Effects(fx) => Some((fx.seq, 0)),
        _ => None,
    }
}

/// State shared by both runners: the mirror, the last flush,
/// the entity id -> type map, seq discipline, latest env.
struct RunState {
    mirror: Mirror,
    last_flush: Vec<MessageT>,
    entity_types: HashMap<u64, String>,
    last_seq: u64,
    last_epoch: u64,
    last_env: Option<proto::EnvironmentUpdateT>,
    log: Vec<String>,
}

impl RunState {
    fn new() -> RunState {
        RunState {
            mirror: Mirror::default(),
            last_flush: Vec::new(),
            entity_types: HashMap::new(),
            last_seq: 0,
            last_epoch: 0,
            last_env: None,
            log: Vec::new(),
        }
    }

    /// Feed one message: seq discipline, type map, env
    /// tracking, mirror. Transport/handshake messages are
    /// logged but carry no mirror state.
    fn feed(&mut self, msg: &MessageT) -> Result<(), String> {
        if let Some((seq, epoch)) = stream_seq(msg) {
            if seq != self.last_seq + 1 {
                return Err(format!(
                    "seq discipline broken: expected {}, got {} ({})",
                    self.last_seq + 1,
                    seq,
                    describe(msg)
                ));
            }
            self.last_seq = seq;
            if epoch > 0 {
                if epoch < self.last_epoch {
                    return Err(format!(
                        "epoch regressed: {} -> {} ({})",
                        self.last_epoch,
                        epoch,
                        describe(msg)
                    ));
                }
                self.last_epoch = epoch;
            }
        }
        match msg {
            MessageT::Snapshot(s) => {
                for e in s.entities.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                    self.note_entity_type(e);
                }
                if let Some(env) = &s.environment {
                    self.last_env = Some((**env).clone());
                }
            }
            MessageT::Delta(d) => {
                for e in d
                    .entity_updates
                    .as_ref()
                    .map(|v| v.as_slice())
                    .unwrap_or(&[])
                {
                    self.note_entity_type(e);
                }
            }
            MessageT::EnvironmentUpdate(e) => {
                self.last_env = Some((**e).clone());
            }
            _ => {}
        }
        // At a Snapshot, before the reseed: the accumulated
        // mirror's memory must agree with the memory the
        // Snapshot transmits wherever both remember a tile
        // (memory membership itself is not wire-trackable
        // between snapshots — see mirror.rs module docs).
        if let MessageT::Snapshot(snap) = msg {
            if !self.mirror.tiles.is_empty() {
                self.mirror.assert_memory_consistent(snap)?;
            }
        }
        match msg {
            MessageT::Snapshot(_)
            | MessageT::Delta(_)
            | MessageT::LightUpdate(_)
            | MessageT::EnvironmentUpdate(_)
            | MessageT::Effects(_)
            | MessageT::UiFrame(_) => self.mirror.apply(msg),
            _ => Ok(()),
        }
    }

    fn note_entity_type(&mut self, e: &proto::EntityT) {
        if let Some(sprite) = &e.sprite_id {
            self.entity_types.insert(e.id, sprite.clone());
        }
    }

    /// Feed a whole flush and remember it as the last one.
    fn feed_flush(&mut self, msgs: Vec<MessageT>) -> Result<(), String> {
        for msg in &msgs {
            self.feed(msg)?;
        }
        self.last_flush = msgs;
        Ok(())
    }

    fn log_step(&mut self, step: &Step, msgs: &[MessageT]) {
        if msgs.is_empty() {
            self.log.push(format!(
                "step {}: {} -> (no messages)",
                step.index, step.desc
            ));
        } else if msgs.len() > 8 {
            let first = describe(&msgs[0]);
            let last = describe(&msgs[msgs.len() - 1]);
            self.log.push(format!(
                "step {}: {} -> {} messages [{first} .. {last}]",
                step.index,
                step.desc,
                msgs.len()
            ));
        } else {
            let parts: Vec<String> = msgs.iter().map(describe).collect();
            self.log.push(format!(
                "step {}: {} -> {}",
                step.index,
                step.desc,
                parts.join(", ")
            ));
        }
    }
}

// ---------------------------------------------------------------------
// Assertion evaluation (shared; mode-specific views injected)
// ---------------------------------------------------------------------

struct EnvView {
    weather_preset: Option<String>,
    weather_id: Option<String>,
    sun_altitude_deg: f64,
}

struct Eval<'a> {
    reg: &'a DefRegistry,
    state: &'a RunState,
    /// Ui mode as the mode's source of truth knows it.
    ui_mode: Option<UiMode>,
    /// Screen name — in-process only (the wire carries the
    /// mode, never the screen's identity).
    ui_screen: Option<String>,
    /// Carried stacks — in-process only (ditto).
    carried: Option<Vec<(u32, u16)>>,
    env: Option<EnvView>,
    mode_name: &'a str,
}

impl Eval<'_> {
    fn tile(&self, pos: [i32; 3]) -> Result<&crate::mirror::TileMirror, String> {
        self.state.mirror.tiles.get(&pos).ok_or_else(|| {
            format!("tile {pos:?} is not in the mirror (outside the snapshot window?)")
        })
    }

    fn id_of(&self, kind: Kind, name: &str) -> u32 {
        self.reg.id_of(kind, name)
    }

    fn entity_ids_of_type(&self, type_id: &str) -> Vec<u64> {
        self.state
            .mirror
            .entities
            .iter()
            .filter(|(id, e)| {
                self.state.entity_types.get(id).map(String::as_str) == Some(type_id)
                    || e.sprite_id.as_deref() == Some(type_id)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    fn eval(&self, a: &Assert) -> Result<(), String> {
        match a {
            Assert::Tile {
                pos,
                terrain,
                furniture,
            } => {
                let t = self.tile(*pos)?;
                if let Some(want) = terrain {
                    let got = self.reg.string_of(Kind::Terrain, t.terrain).unwrap_or("?");
                    if got != want {
                        return Err(format!("tile {pos:?}: terrain is {got}, want {want}"));
                    }
                }
                if let Some(want) = furniture {
                    let got_id = t.furniture;
                    if want == "none" {
                        if got_id != 0 {
                            let got = self.reg.string_of(Kind::Furniture, got_id).unwrap_or("?");
                            return Err(format!("tile {pos:?}: furniture is {got}, want none"));
                        }
                    } else {
                        let got = self.reg.string_of(Kind::Furniture, got_id).unwrap_or("?");
                        if got != want {
                            return Err(format!("tile {pos:?}: furniture is {got}, want {want}"));
                        }
                    }
                }
                Ok(())
            }
            Assert::Player(want) => {
                let got = self
                    .state
                    .mirror
                    .player_pos()
                    .ok_or("the mirror has no player entity")?;
                if got != *want {
                    return Err(format!("player is at {got:?}, want {want:?}"));
                }
                Ok(())
            }
            Assert::PlayerHp { lt, gte, eq } => {
                let e = self
                    .state
                    .mirror
                    .player_entity()
                    .ok_or("the mirror has no player entity")?;
                let frac = e.hp_frac as i64;
                if let Some(v) = lt {
                    if frac >= *v {
                        return Err(format!("player hp_frac is {frac}, want < {v}"));
                    }
                }
                if let Some(v) = gte {
                    if frac < *v {
                        return Err(format!("player hp_frac is {frac}, want >= {v}"));
                    }
                }
                if let Some(v) = eq {
                    if frac != *v {
                        return Err(format!("player hp_frac is {frac}, want {v}"));
                    }
                }
                Ok(())
            }
            Assert::Outside { pos, value } => {
                let t = self.tile(*pos)?;
                if t.outside != *value {
                    return Err(format!(
                        "tile {pos:?}: outside is {}, want {}",
                        t.outside, value
                    ));
                }
                Ok(())
            }
            Assert::Lit {
                pos,
                value,
                not_value,
            } => {
                let t = self.tile(*pos)?;
                if let Some(want) = value {
                    if t.lit != *want {
                        return Err(format!("tile {pos:?}: lit is {:?}, want {:?}", t.lit, want));
                    }
                }
                if let Some(not) = not_value {
                    if t.lit == *not {
                        return Err(format!(
                            "tile {pos:?}: lit is {:?}, want anything else",
                            not
                        ));
                    }
                }
                Ok(())
            }
            Assert::Field {
                pos,
                type_id,
                min_intensity,
            } => {
                let t = self.tile(*pos)?;
                let fid = self.id_of(Kind::Field, type_id);
                let found = t.fields.iter().find(|&&(ty, _)| ty == fid);
                match found {
                    None => Err(format!(
                        "tile {pos:?}: no field {type_id} (has {:?})",
                        t.fields
                    )),
                    Some(&(_, inten)) => {
                        if let Some(min) = min_intensity {
                            if inten < *min {
                                return Err(format!(
                                    "tile {pos:?}: field {type_id} intensity {inten} < {min}"
                                ));
                            }
                        }
                        Ok(())
                    }
                }
            }
            Assert::DisplayedField { pos, type_id } => {
                let t = self.tile(*pos)?;
                let fid = self.id_of(Kind::Field, type_id);
                if t.displayed.0 != fid {
                    let got = if t.displayed.0 == 0 {
                        "none".to_string()
                    } else {
                        self.reg
                            .string_of(Kind::Field, t.displayed.0)
                            .unwrap_or("?")
                            .to_string()
                    };
                    return Err(format!(
                        "tile {pos:?}: displayed field is {got}, want {type_id}"
                    ));
                }
                Ok(())
            }
            Assert::FieldCount { type_id, eq, gte } => {
                let fid = self.id_of(Kind::Field, type_id);
                let n = self
                    .state
                    .mirror
                    .tiles
                    .values()
                    .filter(|t| t.fields.iter().any(|&(ty, _)| ty == fid))
                    .count();
                if let Some(want) = eq {
                    if n != *want {
                        return Err(format!("{n} tiles carry {type_id}, want exactly {want}"));
                    }
                }
                if let Some(want) = gte {
                    if n < *want {
                        return Err(format!("{n} tiles carry {type_id}, want >= {want}"));
                    }
                }
                Ok(())
            }
            Assert::Items { pos, top, count } => {
                let t = self.tile(*pos)?;
                if let Some(want) = top {
                    if want == "none" {
                        if t.top_item != 0 {
                            let got = self.reg.string_of(Kind::Item, t.top_item).unwrap_or("?");
                            return Err(format!("tile {pos:?}: top item is {got}, want none"));
                        }
                    } else {
                        let got = self.reg.string_of(Kind::Item, t.top_item).unwrap_or("?");
                        if got != want {
                            return Err(format!("tile {pos:?}: top item is {got}, want {want}"));
                        }
                    }
                }
                if let Some(want) = count {
                    if t.item_count != *want {
                        return Err(format!(
                            "tile {pos:?}: item count is {}, want {}",
                            t.item_count, want
                        ));
                    }
                }
                Ok(())
            }
            Assert::Carried { type_id, count } => {
                let carried = self.carried.as_ref().ok_or_else(|| {
                    format!(
                        "assert_carried is {}-mode-unavailable: the carried list is not on the wire",
                        self.mode_name
                    )
                })?;
                let iid = self.id_of(Kind::Item, type_id);
                let got = carried
                    .iter()
                    .find(|&&(id, _)| id == iid)
                    .map(|&(_, n)| n)
                    .unwrap_or(0);
                if got != *count {
                    return Err(format!("carried {type_id} count is {got}, want {count}"));
                }
                Ok(())
            }
            Assert::EntityVisible(type_id) => {
                let ids = self.entity_ids_of_type(type_id);
                if ids.is_empty() {
                    return Err(format!("no {type_id} entity on the wire"));
                }
                let visible = ids.iter().any(|id| {
                    let e = &self.state.mirror.entities[id];
                    let pos = e.pos.as_ref().map(|p| [p.x, p.y, p.z]);
                    pos.and_then(|p| self.state.mirror.tiles.get(&p))
                        .is_some_and(|t| t.lit != LitLevel::BLANK)
                });
                if !visible {
                    return Err(format!(
                        "no {type_id} entity stands on a non-BLANK tile (not visible)"
                    ));
                }
                Ok(())
            }
            Assert::EntityHidden(type_id) => {
                let ids = self.entity_ids_of_type(type_id);
                if ids.is_empty() {
                    return Err(format!("no {type_id} entity on the wire"));
                }
                let hidden = ids.iter().any(|id| {
                    let e = &self.state.mirror.entities[id];
                    let pos = e.pos.as_ref().map(|p| [p.x, p.y, p.z]);
                    pos.and_then(|p| self.state.mirror.tiles.get(&p))
                        .is_some_and(|t| t.lit == LitLevel::BLANK)
                });
                if !hidden {
                    return Err(format!("no {type_id} entity stands on a BLANK tile"));
                }
                Ok(())
            }
            Assert::EntityAbsent(type_id) => {
                let ids = self.entity_ids_of_type(type_id);
                if !ids.is_empty() {
                    return Err(format!("{type_id} entities still on the wire: {ids:?}"));
                }
                Ok(())
            }
            Assert::EntityAt { type_id, pos } => {
                let ids = self.entity_ids_of_type(type_id);
                let positions: Vec<[i32; 3]> = ids
                    .iter()
                    .filter_map(|id| {
                        self.state.mirror.entities[id]
                            .pos
                            .as_ref()
                            .map(|p| [p.x, p.y, p.z])
                    })
                    .collect();
                if !positions.contains(pos) {
                    return Err(format!(
                        "no {type_id} entity at {pos:?} (positions: {positions:?})"
                    ));
                }
                Ok(())
            }
            Assert::UiMode(want) => {
                let mode = self.ui_mode.ok_or("no UiFrame seen yet (mode unknown)")?;
                let got = if mode == UiMode::MapInput {
                    "MapInput"
                } else if mode == UiMode::UiInput {
                    "UiInput"
                } else {
                    "?"
                };
                if got != want {
                    return Err(format!("UI mode is {got}, want {want}"));
                }
                Ok(())
            }
            Assert::UiScreen(want) => {
                let screen = self.ui_screen.as_ref().ok_or_else(|| {
                    format!(
                        "assert_ui_screen is {}-mode-unavailable: the wire carries the UI \
                         mode, never the screen's identity",
                        self.mode_name
                    )
                })?;
                if screen != want {
                    return Err(format!("UI screen is {screen}, want {want}"));
                }
                Ok(())
            }
            Assert::Env {
                weather,
                weather_id,
                sun_altitude_lt,
                sun_altitude_gt,
            } => {
                let env = self.env.as_ref().ok_or("no environment state seen yet")?;
                if let Some(want) = weather {
                    match &env.weather_preset {
                        Some(got) if got == want => {}
                        Some(got) => return Err(format!("weather preset is {got}, want {want}")),
                        None => {
                            return Err(format!(
                                "weather preset is {}-mode-unavailable (assert weather_id instead)",
                                self.mode_name
                            ))
                        }
                    }
                }
                if let Some(want) = weather_id {
                    match &env.weather_id {
                        Some(got) if got == want => {}
                        Some(got) => return Err(format!("weather id is {got}, want {want}")),
                        None => return Err("no weather id seen yet".to_string()),
                    }
                }
                if let Some(lt) = sun_altitude_lt {
                    if !(env.sun_altitude_deg < *lt) {
                        return Err(format!(
                            "sun altitude is {:.2} deg, want < {lt}",
                            env.sun_altitude_deg
                        ));
                    }
                }
                if let Some(gt) = sun_altitude_gt {
                    if !(env.sun_altitude_deg > *gt) {
                        return Err(format!(
                            "sun altitude is {:.2} deg, want > {gt}",
                            env.sun_altitude_deg
                        ));
                    }
                }
                Ok(())
            }
            Assert::Message { kind, seq, epoch } => {
                let found = self.state.last_flush.iter().any(|m| {
                    if message_kind_name(m) != kind {
                        return false;
                    }
                    if let Some(want_seq) = seq {
                        if stream_seq(m).map(|(s, _)| s) != Some(*want_seq) {
                            return false;
                        }
                    }
                    if let Some(want_epoch) = epoch {
                        if stream_seq(m).map(|(_, e)| e) != Some(*want_epoch) {
                            return false;
                        }
                    }
                    true
                });
                if !found {
                    let have: Vec<String> = self.state.last_flush.iter().map(describe).collect();
                    return Err(format!(
                        "last flush has no {kind} message (seq {seq:?}, epoch {epoch:?}); it has: {have:?}"
                    ));
                }
                Ok(())
            }
            Assert::DeltaContainsTile {
                pos,
                terrain,
                furniture,
            } => {
                for m in &self.state.last_flush {
                    let MessageT::Delta(d) = m else { continue };
                    for tc in d.tile_changes.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                        let p = tc.pos.as_ref().expect("tile change pos");
                        if [p.x, p.y, p.z] != *pos {
                            continue;
                        }
                        if let Some(want) = terrain {
                            let got = self.reg.string_of(Kind::Terrain, tc.terrain).unwrap_or("?");
                            if got != want {
                                continue;
                            }
                        }
                        if let Some(want) = furniture {
                            if want == "none" {
                                if tc.furniture != 0 {
                                    continue;
                                }
                            } else {
                                let got = self
                                    .reg
                                    .string_of(Kind::Furniture, tc.furniture)
                                    .unwrap_or("?");
                                if got != want {
                                    continue;
                                }
                            }
                        }
                        return Ok(());
                    }
                }
                Err(format!(
                    "no Delta in the last flush changes tile {pos:?} as asserted"
                ))
            }
            Assert::DeltaContainsEntity { type_id } => {
                for m in &self.state.last_flush {
                    let MessageT::Delta(d) = m else { continue };
                    for e in d
                        .entity_updates
                        .as_ref()
                        .map(|v| v.as_slice())
                        .unwrap_or(&[])
                    {
                        if e.sprite_id.as_deref() == Some(type_id.as_str())
                            || self.state.entity_types.get(&e.id).map(String::as_str)
                                == Some(type_id.as_str())
                        {
                            return Ok(());
                        }
                    }
                }
                Err(format!(
                    "no Delta in the last flush updates a {type_id} entity"
                ))
            }
            Assert::DeltaContainsRemoved { type_id } => {
                for m in &self.state.last_flush {
                    let MessageT::Delta(d) = m else { continue };
                    for id in d
                        .entity_removed
                        .as_ref()
                        .map(|v| v.as_slice())
                        .unwrap_or(&[])
                    {
                        if self.state.entity_types.get(id).map(String::as_str)
                            == Some(type_id.as_str())
                        {
                            return Ok(());
                        }
                    }
                }
                Err(format!(
                    "no Delta in the last flush removes a {type_id} entity"
                ))
            }
            Assert::NoTileChange(pos) => {
                for m in &self.state.last_flush {
                    let MessageT::Delta(d) = m else { continue };
                    for tc in d.tile_changes.as_ref().map(|v| v.as_slice()).unwrap_or(&[]) {
                        let p = tc.pos.as_ref().expect("tile change pos");
                        if [p.x, p.y, p.z] == *pos {
                            return Err(format!(
                                "the last flush changes tile {pos:?} (terrain {}), want no change",
                                tc.terrain
                            ));
                        }
                    }
                }
                Ok(())
            }
            Assert::VehicleFace { vehicle, face_deg } => {
                let v = self
                    .state
                    .mirror
                    .vehicles
                    .get(vehicle)
                    .ok_or_else(|| format!("vehicle {vehicle} is not in the mirror"))?;
                if v.face_deg != *face_deg {
                    return Err(format!(
                        "vehicle {vehicle} face_deg is {}, want {}",
                        v.face_deg, face_deg
                    ));
                }
                Ok(())
            }
            Assert::VehiclePart {
                vehicle,
                part,
                mount,
                pos,
                hp_frac,
                broken,
                roof,
            } => {
                let v = self
                    .state
                    .mirror
                    .vehicles
                    .get(vehicle)
                    .ok_or_else(|| format!("vehicle {vehicle} is not in the mirror"))?;
                let want_id = self.id_of(Kind::VehiclePart, part);
                let found = v
                    .parts
                    .as_ref()
                    .map(|ps| {
                        ps.iter().find(|p| {
                            p.vpart == want_id
                                && p.mount.as_ref().map(|m| [m.x, m.y, m.z]) == Some(*mount)
                        })
                    })
                    .unwrap_or(None);
                let p = found.ok_or_else(|| {
                    format!("vehicle {vehicle} has no part {part} at mount {mount:?}")
                })?;
                if let Some(want) = pos {
                    let got = p.abs_pos.as_ref().map(|a| [a.x, a.y, a.z]);
                    if got != Some(*want) {
                        return Err(format!(
                            "vehicle {vehicle} part {part} {mount:?} is at {got:?}, want {want:?}"
                        ));
                    }
                }
                if let Some(want) = hp_frac {
                    if p.hp_frac != *want {
                        return Err(format!(
                            "vehicle {vehicle} part {part} hp_frac is {}, want {}",
                            p.hp_frac, want
                        ));
                    }
                }
                if let Some(want) = broken {
                    let is = p.flags & 2 != 0;
                    if is != *want {
                        return Err(format!(
                            "vehicle {vehicle} part {part} broken is {is}, want {want}"
                        ));
                    }
                }
                if let Some(want) = roof {
                    let is = p.flags & 4 != 0;
                    if is != *want {
                        return Err(format!(
                            "vehicle {vehicle} part {part} roof is {is}, want {want}"
                        ));
                    }
                }
                Ok(())
            }
            Assert::VehiclePartAbsent {
                vehicle,
                part,
                mount,
            } => {
                let want_id = self.id_of(Kind::VehiclePart, part);
                if let Some(v) = self.state.mirror.vehicles.get(vehicle) {
                    let present = v
                        .parts
                        .as_ref()
                        .map(|ps| {
                            ps.iter().any(|p| {
                                p.vpart == want_id
                                    && p.mount.as_ref().map(|m| [m.x, m.y, m.z]) == Some(*mount)
                            })
                        })
                        .unwrap_or(false);
                    if present {
                        return Err(format!(
                            "vehicle {vehicle} still has part {part} at mount {mount:?}"
                        ));
                    }
                }
                Ok(())
            }
            Assert::MirrorNow => Ok(()), // the invariant runs after every action step
        }
    }
}

fn message_kind_name(msg: &MessageT) -> &'static str {
    match msg {
        MessageT::Hello(_) => "Hello",
        MessageT::HelloAck(_) => "HelloAck",
        MessageT::TileDefs(_) => "TileDefs",
        MessageT::Snapshot(_) => "Snapshot",
        MessageT::Delta(_) => "Delta",
        MessageT::LightUpdate(_) => "LightUpdate",
        MessageT::EnvironmentUpdate(_) => "EnvironmentUpdate",
        MessageT::Effects(_) => "Effects",
        MessageT::UiFrame(_) => "UiFrame",
        MessageT::Input(_) => "Input",
        MessageT::Ping(_) => "Ping",
        MessageT::Pong(_) => "Pong",
        MessageT::Resize(_) => "Resize",
        MessageT::PickResult(_) => "PickResult",
        MessageT::Shutdown(_) => "Shutdown",
        _ => "Unknown",
    }
}

fn key_input(code: i32) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = code;
    input.pressed = true;
    input
}

fn fail<T>(state: &RunState, step: &Step, error: String) -> Result<T, ScenarioFailure> {
    Err(ScenarioFailure {
        step_index: step.index,
        step_desc: step.desc.clone(),
        error,
        log: state.log.clone(),
    })
}

// ---------------------------------------------------------------------
// In-process runner
// ---------------------------------------------------------------------

/// Run a scenario against a World built from `scene` (the
/// scenario's `seed`, when present, overrides the scene's).
/// The scene is taken by value because the seed override and
/// the world build consume it.
pub fn run_in_process(
    reg: &DefRegistry,
    mut scene: Scene,
    scenario: &Scenario,
) -> Result<RunOutcome, ScenarioFailure> {
    if let Some(seed) = scenario.seed {
        scene.seed = seed;
    }
    let mut world = World::from_scene(reg, &scene).map_err(|e| ScenarioFailure {
        step_index: 0,
        step_desc: "world build".to_string(),
        error: format!("world build failed: {e}"),
        log: Vec::new(),
    })?;
    let mut state = RunState::new();
    let mut connected = true;

    // The session's opening Snapshot (what the server sends
    // after TileDefs) seeds the mirror.
    let snap = world.snapshot(reg);
    state
        .feed_flush(vec![MessageT::Snapshot(Box::new(snap))])
        .map_err(|e| ScenarioFailure {
            step_index: 0,
            step_desc: "initial snapshot".to_string(),
            error: e,
            log: state.log.clone(),
        })?;
    state
        .log
        .push("step 0: connect -> Snapshot seq=1".to_string());

    let mut executed = 0usize;
    for step in &scenario.steps {
        executed += 1;
        match &step.kind {
            StepKind::Action(action) => {
                let msgs = match execute_in_process(
                    reg,
                    &mut world,
                    &mut state,
                    action,
                    &mut connected,
                    step,
                ) {
                    Ok(msgs) => msgs,
                    Err(e) => return fail(&state, step, e),
                };
                state.log_step(step, &msgs);
                if let Err(e) = state.feed_flush(msgs) {
                    return fail(&state, step, e);
                }
                // The T12 invariant, after every action step:
                // the stream-fed mirror must equal a fresh
                // Snapshot of the world. (While disconnected
                // the client view is dropped by design.)
                if connected {
                    let fresh = world.build_snapshot(reg, 0, 0);
                    if let Err(e) = state.mirror.assert_matches(&fresh) {
                        return fail(&state, step, format!("mirror invariant: {e}"));
                    }
                }
            }
            StepKind::Assert(assert) => {
                let eval = Eval {
                    reg,
                    state: &state,
                    ui_mode: Some(world.ui_state().mode()),
                    ui_screen: Some(world.ui_state().name().to_string()),
                    carried: Some(world.carried_items().to_vec()),
                    env: Some(EnvView {
                        weather_preset: Some(world.weather.preset.clone()),
                        weather_id: Some(world.weather.wire_id.clone()),
                        sun_altitude_deg: environment::sun_azimuth_altitude(world.game_turn).1,
                    }),
                    mode_name: "in-process",
                };
                if let Err(e) = eval.eval(assert) {
                    return fail(&state, step, e);
                }
                state
                    .log
                    .push(format!("step {}: {} ok", step.index, step.desc));
            }
        }
    }
    Ok(RunOutcome {
        steps_executed: executed,
        log: state.log,
    })
}

fn execute_in_process(
    reg: &DefRegistry,
    world: &mut World,
    _state: &mut RunState,
    action: &Action,
    connected: &mut bool,
    step: &Step,
) -> Result<Vec<MessageT>, String> {
    match action {
        Action::Disconnect => {
            if !*connected {
                return Err("disconnect while already disconnected".to_string());
            }
            *connected = false;
            // The client view is dropped; the world (the
            // server side) persists untouched.
            _state.mirror = Mirror::default();
            _state.last_flush = Vec::new();
            return Ok(Vec::new());
        }
        Action::Reconnect => {
            if *connected {
                return Err(
                    "reconnect while connected (use the snapshot step for a resync)".to_string(),
                );
            }
            *connected = true;
            let snap = world.snapshot(reg);
            return Ok(vec![MessageT::Snapshot(Box::new(snap))]);
        }
        Action::Ping => {
            // Transport-level: no socket exists in-process.
            // The honest equivalent is a log line — the world
            // is trivially responsive to the next step.
            let _ = step;
            return Ok(Vec::new());
        }
        _ => {}
    }
    if !*connected {
        return Err(format!(
            "step {} runs while disconnected (only reconnect is legal)",
            step.index
        ));
    }
    let msgs = match action {
        Action::Key(code) => world.apply_input(reg, &key_input(*code)),
        Action::Text(text) => {
            let mut input = InputT::default();
            input.kind = InputKind::Text;
            input.text = Some(text.clone());
            world.apply_input(reg, &input)
        }
        Action::Click { x, y } => {
            let mut input = InputT::default();
            input.kind = InputKind::MouseButton;
            input.mouse_x = *x;
            input.mouse_y = *y;
            input.button = 0;
            input.pressed = true;
            input.focused = true;
            world.apply_input(reg, &input)
        }
        Action::Resize { width, height } => world.apply_resize(reg, *width, *height),
        Action::AdvanceTurns(n) => {
            let mut msgs = Vec::new();
            for _ in 0..*n {
                msgs.extend(world.flush_turn(reg));
            }
            msgs
        }
        Action::AdvanceTime(seconds) => world.advance_clock(reg, *seconds),
        Action::SetWeather(name) => world.set_weather(reg, name)?,
        Action::Spawn { type_id, pos } => {
            if world.spawn_monster(reg, type_id, *pos).is_none() {
                return Err(format!(
                    "spawn refused: unknown type '{type_id}' or off-scene position {pos:?}"
                ));
            }
            world.flush_turn(reg)
        }
        Action::Bash { pos } => {
            world.bash_tile(reg, *pos);
            world.flush_turn(reg)
        }
        Action::DamageEntity { type_id, amount } => {
            let id = world
                .entities
                .iter()
                .find(|e| {
                    e.sprite_id.as_deref() == Some(type_id.as_str())
                        && Some(e.id) != world.player_id
                })
                .map(|e| e.id)
                .ok_or_else(|| format!("no live {type_id} entity to damage"))?;
            world.damage_entity(reg, id, *amount);
            world.flush_turn(reg)
        }
        Action::DamageVehiclePart {
            vehicle,
            part,
            mount,
            amount,
        } => {
            let idx = world
                .vehicle_part_index(reg, *vehicle, part, *mount)
                .ok_or_else(|| {
                    format!("vehicle {vehicle} has no part {part} at mount {mount:?}")
                })?;
            if !world.damage_vehicle_part(*vehicle, idx, *amount) {
                return Err(format!("damage_vehicle_part refused for vehicle {vehicle}"));
            }
            world.flush_turn(reg)
        }
        Action::RotateVehicle { vehicle, face_deg } => {
            if !world.set_vehicle_face(*vehicle, *face_deg) {
                return Err(format!("rotate refused: no vehicle {vehicle}"));
            }
            world.flush_turn(reg)
        }
        Action::TranslateVehicle { vehicle, delta } => {
            if !world.translate_vehicle(*vehicle, *delta) {
                return Err(format!("translate refused: no vehicle {vehicle}"));
            }
            world.flush_turn(reg)
        }
        Action::RemoveVehiclePart {
            vehicle,
            part,
            mount,
        } => {
            let idx = world
                .vehicle_part_index(reg, *vehicle, part, *mount)
                .ok_or_else(|| {
                    format!("vehicle {vehicle} has no part {part} at mount {mount:?}")
                })?;
            if !world.remove_vehicle_part(*vehicle, idx) {
                return Err(format!("remove_vehicle_part refused for vehicle {vehicle}"));
            }
            world.flush_turn(reg)
        }
        Action::Snapshot => {
            let snap = world.snapshot(reg);
            vec![MessageT::Snapshot(Box::new(snap))]
        }
        Action::Disconnect | Action::Reconnect | Action::Ping => unreachable!("handled above"),
    };
    Ok(msgs)
}

// ---------------------------------------------------------------------
// Process runner: drive a real sim-server subprocess as a client
// ---------------------------------------------------------------------

/// Where the process runner finds the server binary and its
/// data (the gate passes the workspace paths explicitly).
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub server_bin: PathBuf,
    pub defs_path: PathBuf,
    pub scenes_dir: PathBuf,
}

/// One client connection to the spawned server.
struct Client {
    stream: UnixStream,
    dec: FrameDecoder,
}

impl Client {
    fn send(&mut self, msg: &MessageT) -> Result<(), String> {
        let framed = FrameEncoder::encode(&encode_message(msg));
        self.stream
            .write_all(&framed)
            .map_err(|e| format!("client send: {e}"))
    }

    /// Read one message, waiting up to `timeout`. `Ok(None)` =
    /// the wait elapsed with no complete frame (or EOF —
    /// callers that require a message treat that as failure).
    fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<MessageT>, String> {
        if let Some(frame) = self.dec.next_frame().map_err(|e| format!("frame: {e}"))? {
            return Ok(Some(
                decode_message(&frame).map_err(|e| format!("decode: {e}"))?,
            ));
        }
        self.stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| format!("set timeout: {e}"))?;
        let mut buf = [0u8; 16384];
        match self.stream.read(&mut buf) {
            Ok(0) => Ok(None),
            Ok(n) => {
                self.dec.push(&buf[..n]);
                match self.dec.next_frame().map_err(|e| format!("frame: {e}"))? {
                    Some(frame) => Ok(Some(
                        decode_message(&frame).map_err(|e| format!("decode: {e}"))?,
                    )),
                    None => self.recv_timeout(timeout),
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                Ok(None)
            }
            Err(e) => Err(format!("client read: {e}")),
        }
    }

    /// Collect a turn's flush: the server answers an input
    /// with 0..n messages and there is no end-of-flush marker
    /// on the wire (the real engine reads asynchronously too),
    /// so the flush is everything that arrives before a quiet
    /// window elapses.
    fn collect_flush(&mut self) -> Result<Vec<MessageT>, String> {
        let mut msgs = Vec::new();
        let first = self.recv_timeout(Duration::from_secs(10))?;
        match first {
            Some(m) => msgs.push(m),
            None => return Ok(msgs),
        }
        loop {
            match self.recv_timeout(Duration::from_millis(200))? {
                Some(m) => msgs.push(m),
                None => break,
            }
        }
        Ok(msgs)
    }
}

fn connect_client(socket: &Path) -> Result<Client, String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match UnixStream::connect(socket) {
            Ok(stream) => {
                return Ok(Client {
                    stream,
                    dec: FrameDecoder::new(),
                })
            }
            Err(e) => {
                if Instant::now() >= deadline {
                    return Err(format!(
                        "server never listened on {}: {e}",
                        socket.display()
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// The client handshake: Hello in, HelloAck out, then
/// TileDefs + Snapshot + the initial UiFrame. Returns the
/// handshake messages (the Snapshot seeds the mirror).
fn handshake(client: &mut Client) -> Result<Vec<MessageT>, String> {
    let mut msgs = Vec::new();
    let hello = client
        .recv_timeout(Duration::from_secs(30))?
        .ok_or("server closed before Hello")?;
    match &hello {
        MessageT::Hello(h) => {
            if h.protocol_version != proto::PROTOCOL_VERSION {
                return Err(format!(
                    "server speaks protocol {}, we speak {}",
                    h.protocol_version,
                    proto::PROTOCOL_VERSION
                ));
            }
        }
        other => return Err(format!("expected Hello, got {}", describe(other))),
    }
    msgs.push(hello);
    client.send(&MessageT::HelloAck(Box::new(sample_hello_ack())))?;
    for want in ["TileDefs", "Snapshot", "UiFrame"] {
        let msg = client
            .recv_timeout(Duration::from_secs(30))?
            .ok_or_else(|| format!("server closed before {want}"))?;
        if message_kind_name(&msg) != want {
            return Err(format!("expected {want}, got {}", describe(&msg)));
        }
        msgs.push(msg);
    }
    Ok(msgs)
}

/// Run a scenario against a spawned `sim-server` subprocess.
/// Steps a client cannot perform (world events) fail with a
/// named mode error — see the module docs for the matrix.
pub fn run_process(
    reg: &DefRegistry,
    scenario: &Scenario,
    config: &ProcessConfig,
) -> Result<RunOutcome, ScenarioFailure> {
    // Unique per run: gates execute tests in parallel threads,
    // and two runs of the same scene must never share a socket.
    static RUN_NONCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nonce = RUN_NONCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let socket = std::env::temp_dir().join(format!(
        "sim-scenario-{}-{}-{}.sock",
        std::process::id(),
        scenario.scene,
        nonce
    ));
    if socket.exists() {
        let _ = std::fs::remove_file(&socket);
    }
    let stderr_path = std::env::temp_dir().join(format!(
        "sim-scenario-{}-{}-{}.stderr",
        std::process::id(),
        scenario.scene,
        nonce
    ));
    let stderr_file = std::fs::File::create(&stderr_path).map_err(|e| ScenarioFailure {
        step_index: 0,
        step_desc: "server spawn".to_string(),
        error: format!("cannot create server stderr file: {e}"),
        log: Vec::new(),
    })?;
    let mut child = Command::new(&config.server_bin)
        .arg(&socket)
        .arg("--scene")
        .arg(&scenario.scene)
        .env("CDDA_SIM_SCENE_DIR", &config.scenes_dir)
        .env("CDDA_SIM_DEFS", &config.defs_path)
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|e| ScenarioFailure {
            step_index: 0,
            step_desc: "server spawn".to_string(),
            error: format!("cannot spawn {}: {e}", config.server_bin.display()),
            log: Vec::new(),
        })?;

    let result = run_process_session(reg, scenario, &socket);
    // Teardown, whatever happened: a graceful Shutdown when a
    // connection can be had, then wait, then kill.
    let clean = shutdown_child(&socket, &mut child);
    let _ = std::fs::remove_file(&socket);
    match (result, clean) {
        (Ok(mut outcome), Ok(())) => {
            outcome
                .log
                .push("teardown: server shut down cleanly".to_string());
            Ok(outcome)
        }
        (Ok(_), Err(e)) => Err(ScenarioFailure {
            step_index: 0,
            step_desc: "teardown".to_string(),
            error: e,
            log: Vec::new(),
        }),
        (Err(mut failure), _) => {
            if let Ok(tail) = std::fs::read_to_string(&stderr_path) {
                let tail: Vec<&str> = tail.lines().rev().take(5).collect();
                if !tail.is_empty() {
                    failure.error = format!(
                        "{} (server stderr tail: {})",
                        failure.error,
                        tail.into_iter().rev().collect::<Vec<_>>().join(" | ")
                    );
                }
            }
            Err(failure)
        }
    }
}

fn shutdown_child(socket: &Path, child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|e| format!("wait: {e}"))?
        .is_some()
    {
        return Ok(()); // already exited on its own
    }
    // One graceful attempt: connect, handshake, Shutdown.
    if let Ok(mut client) = connect_client(socket) {
        if handshake(&mut client).is_ok() {
            let mut shutdown = ShutdownT::default();
            shutdown.reason = Some("scenario runner teardown".to_string());
            let _ = client.send(&MessageT::Shutdown(Box::new(shutdown)));
        }
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                return Err(format!("server exited with {status}"));
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("server did not exit within 5 s of Shutdown; killed".to_string());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(format!("wait: {e}")),
        }
    }
}

fn run_process_session(
    reg: &DefRegistry,
    scenario: &Scenario,
    socket: &Path,
) -> Result<RunOutcome, ScenarioFailure> {
    let mut state = RunState::new();
    let mut client = Some(connect_client(socket).map_err(|e| ScenarioFailure {
        step_index: 0,
        step_desc: "connect".to_string(),
        error: e,
        log: Vec::new(),
    })?);
    {
        let msgs = handshake(client.as_mut().expect("client")).map_err(|e| ScenarioFailure {
            step_index: 0,
            step_desc: "handshake".to_string(),
            error: e,
            log: state.log.clone(),
        })?;
        state.log.push(format!(
            "step 0: connect -> {}",
            msgs.iter().map(describe).collect::<Vec<_>>().join(", ")
        ));
        if let Err(e) = state.feed_flush(msgs) {
            return Err(ScenarioFailure {
                step_index: 0,
                step_desc: "handshake".to_string(),
                error: e,
                log: state.log.clone(),
            });
        }
    }
    // The latest UiFrame's mode (process mode's UI truth).
    let mut ui_mode = Some(UiMode::MapInput);

    let mut executed = 0usize;
    for step in &scenario.steps {
        executed += 1;
        match &step.kind {
            StepKind::Action(action) => {
                let msgs = match execute_process(&mut client, socket, &mut state, action, step) {
                    Ok(msgs) => msgs,
                    Err(e) => return fail(&state, step, e),
                };
                for m in &msgs {
                    if let MessageT::UiFrame(f) = m {
                        ui_mode = Some(f.mode);
                    }
                }
                state.log_step(step, &msgs);
                if let Err(e) = state.feed_flush(msgs) {
                    return fail(&state, step, e);
                }
            }
            StepKind::Assert(assert) => {
                let env = state.last_env.as_ref().map(|e| EnvView {
                    weather_preset: None,
                    weather_id: e.weather_id.clone(),
                    sun_altitude_deg: e.sun_altitude_deg as f64,
                });
                let eval = Eval {
                    reg,
                    state: &state,
                    ui_mode,
                    ui_screen: None,
                    carried: None,
                    env,
                    mode_name: "process",
                };
                if let Err(e) = eval.eval(assert) {
                    return fail(&state, step, e);
                }
                state
                    .log
                    .push(format!("step {}: {} ok", step.index, step.desc));
            }
        }
    }
    Ok(RunOutcome {
        steps_executed: executed,
        log: state.log,
    })
}

/// World-event steps have no client equivalent: the protocol
/// a real engine speaks has no message that spawns a monster
/// or changes the weather. Refuse them by name.
fn world_event_refusal(action: &Action) -> Option<String> {
    let name = match action {
        Action::Spawn { .. } => "spawn",
        Action::Bash { .. } => "bash/damage_tile",
        Action::DamageEntity { .. } => "damage_entity",
        Action::DamageVehiclePart { .. } => "damage_vehicle_part",
        Action::RotateVehicle { .. } => "rotate_vehicle",
        Action::TranslateVehicle { .. } => "translate_vehicle",
        Action::RemoveVehiclePart { .. } => "remove_vehicle_part",
        Action::SetWeather(_) => "set_weather",
        Action::AdvanceTime(_) => "advance_time",
        _ => return None,
    };
    Some(format!(
        "'{name}' is a world-event step: a client cannot inject it over the wire. \
         Author the event into the scene (e.g. [[spawn]]) or run the scenario in-process."
    ))
}

fn execute_process(
    client: &mut Option<Client>,
    socket: &Path,
    state: &mut RunState,
    action: &Action,
    _step: &Step,
) -> Result<Vec<MessageT>, String> {
    if let Some(refusal) = world_event_refusal(action) {
        return Err(refusal);
    }
    match action {
        Action::Disconnect => {
            if client.is_none() {
                return Err("disconnect while already disconnected".to_string());
            }
            *client = None; // dropping the stream IS the disconnect
            state.mirror = Mirror::default();
            state.last_flush = Vec::new();
            Ok(Vec::new())
        }
        Action::Reconnect => {
            if client.is_some() {
                return Err(
                    "reconnect while connected (use the snapshot step for a resync)".to_string(),
                );
            }
            let mut c = connect_client(socket)?;
            let msgs = handshake(&mut c)?;
            // The process-mode mirror invariant: the fresh
            // Snapshot must agree with the delta-fed mirror
            // from before the disconnect.
            let snap = msgs.iter().find_map(|m| match m {
                MessageT::Snapshot(s) => Some(&**s),
                _ => None,
            });
            if let Some(snap) = snap {
                if !state.mirror.tiles.is_empty() || !state.mirror.entities.is_empty() {
                    state
                        .mirror
                        .assert_matches(snap)
                        .map_err(|e| format!("resync invariant: {e}"))?;
                }
            }
            *client = Some(c);
            Ok(msgs)
        }
        Action::Snapshot => {
            // No client message forces a Snapshot; the honest
            // equivalent is a reconnect's fresh Snapshot, with
            // the same invariant check.
            if client.is_none() {
                return Err("snapshot step while disconnected".to_string());
            }
            *client = None;
            let mut c = connect_client(socket)?;
            let msgs = handshake(&mut c)?;
            let snap = msgs.iter().find_map(|m| match m {
                MessageT::Snapshot(s) => Some(&**s),
                _ => None,
            });
            if let Some(snap) = snap {
                state
                    .mirror
                    .assert_matches(snap)
                    .map_err(|e| format!("resync invariant: {e}"))?;
            }
            *client = Some(c);
            Ok(msgs)
        }
        Action::Ping => {
            let c = client.as_mut().ok_or("ping while disconnected")?;
            let mut ping = PingT::default();
            ping.nonce = 0x5CE7;
            ping.sent_ms = 0;
            c.send(&MessageT::Ping(Box::new(ping)))?;
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                match c.recv_timeout(Duration::from_secs(2))? {
                    Some(MessageT::Pong(p)) if p.nonce == 0x5CE7 => return Ok(Vec::new()),
                    Some(other) => {
                        return Err(format!(
                            "expected Pong for the ping step, got {}",
                            describe(&other)
                        ))
                    }
                    None => {
                        if Instant::now() >= deadline {
                            return Err("ping step: no Pong within 10 s".to_string());
                        }
                    }
                }
            }
        }
        Action::Key(code) => {
            let c = client.as_mut().ok_or("key step while disconnected")?;
            c.send(&MessageT::Input(Box::new(key_input(*code))))?;
            c.collect_flush()
        }
        Action::Text(text) => {
            let c = client.as_mut().ok_or("text step while disconnected")?;
            let mut input = InputT::default();
            input.kind = InputKind::Text;
            input.text = Some(text.clone());
            c.send(&MessageT::Input(Box::new(input)))?;
            c.collect_flush()
        }
        Action::Click { x, y } => {
            let c = client.as_mut().ok_or("click step while disconnected")?;
            let mut input = InputT::default();
            input.kind = InputKind::MouseButton;
            input.mouse_x = *x;
            input.mouse_y = *y;
            input.button = 0;
            input.pressed = true;
            input.focused = true;
            c.send(&MessageT::Input(Box::new(input)))?;
            c.collect_flush()
        }
        Action::Resize { width, height } => {
            let c = client.as_mut().ok_or("resize step while disconnected")?;
            let mut resize = ResizeT::default();
            resize.width = *width;
            resize.height = *height;
            resize.dpi = 1.0;
            c.send(&MessageT::Resize(Box::new(resize)))?;
            c.collect_flush()
        }
        Action::AdvanceTurns(n) => {
            let c = client.as_mut().ok_or("advance_turns while disconnected")?;
            let mut all = Vec::new();
            for _ in 0..*n {
                // A client waits by sending wait keys — the
                // honest client form of advancing turns.
                c.send(&MessageT::Input(Box::new(key_input('.' as i32))))?;
                all.extend(c.collect_flush()?);
            }
            Ok(all)
        }
        Action::Spawn { .. }
        | Action::Bash { .. }
        | Action::DamageEntity { .. }
        | Action::DamageVehiclePart { .. }
        | Action::RotateVehicle { .. }
        | Action::TranslateVehicle { .. }
        | Action::RemoveVehiclePart { .. }
        | Action::SetWeather(_)
        | Action::AdvanceTime(_) => unreachable!("refused above"),
    }
}
