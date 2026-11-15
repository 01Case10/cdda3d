//! The mod/debug report (Phase 8 §5, "Large mods"): every def the
//! game exported for which the frontend has no material, listed by
//! string id and kind.
//!
//! The gate this serves (8.7) thresholds on this report: a mod is
//! supported when its missing list is empty or known-small. The
//! report must therefore be *exact* — it is built from the same
//! `DefTable` the world model resolves through, and "has a
//! material" is the caller's set (in production, the voxelizer's
//! material table). A def with `fg_ref < 0 && bg_ref < 0` carries
//! no sprite reference at all (fields render as shader effects,
//! some defs are abstract); those are reported separately so the
//! count of *renderable-but-unresolved* defs is not inflated.

use std::collections::HashSet;

use crate::doors::DefTable;

/// One unresolved def: numeric id, string id, kind name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDef {
    pub id: u32,
    pub string_id: String,
    pub kind: &'static str,
}

fn kind_name(kind: proto::TileKind) -> &'static str {
    use proto::TileKind as K;
    match kind {
        K::Terrain => "terrain",
        K::Furniture => "furniture",
        K::Field => "field",
        K::Item => "item",
        K::Monster => "monster",
        K::VehiclePart => "vehicle_part",
        _ => "other",
    }
}

fn string_id(d: &proto::TileDefT) -> String {
    d.string_id.clone().unwrap_or_else(|| format!("#{}", d.id))
}

/// Defs that reference a sprite/material slot (fg or bg) but whose
/// id is not in `has_material`, sorted by id. This is the list a
/// mod author works through; empty means the mod renders fully.
pub fn missing_materials(table: &DefTable, has_material: &HashSet<u32>) -> Vec<MissingDef> {
    let mut out: Vec<MissingDef> = table
        .defs()
        .filter(|d| (d.fg_ref >= 0 || d.bg_ref >= 0) && !has_material.contains(&d.id))
        .map(|d| MissingDef {
            id: d.id,
            string_id: string_id(d),
            kind: kind_name(d.kind),
        })
        .collect();
    out.sort_by_key(|m| m.id);
    out
}

/// Defs with no sprite reference at all (abstract defs, shader
/// fields). Reported for completeness; they are not "missing".
pub fn unreferenced_defs(table: &DefTable) -> Vec<MissingDef> {
    let mut out: Vec<MissingDef> = table
        .defs()
        .filter(|d| d.fg_ref < 0 && d.bg_ref < 0)
        .map(|d| MissingDef {
            id: d.id,
            string_id: string_id(d),
            kind: kind_name(d.kind),
        })
        .collect();
    out.sort_by_key(|m| m.id);
    out
}

/// The report as plain text lines, one def per line:
/// `missing <kind> <id> <string_id>`.
pub fn missing_report_lines(table: &DefTable, has_material: &HashSet<u32>) -> Vec<String> {
    missing_materials(table, has_material)
        .iter()
        .map(|m| format!("missing {} {} {}", m.kind, m.id, m.string_id))
        .collect()
}
