//! S1 registry gate: the S1a generated table loads into the
//! registry, spot lookups match the hand-checked real values, and
//! the built wire TileDefs carries every def + the registry table.

use std::path::PathBuf;

use sim::defs::resolve_defs_path;
use sim::{DefRegistry, Kind};

pub fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

pub fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

#[test]
fn registry_loads_full_table() {
    let reg = DefRegistry::load(&defs_path()).expect("defs.json must load");
    assert_eq!(reg.len(), 13_532, "full Option C def count (0.I-1 base census)");
    assert_eq!(reg.connect_groups().len(), 36, "connect-group registry (0.I-1 base census)");

    // Spot lookups (hand-checked against the real JSON in S1a).
    let door = reg
        .def_by_string(Kind::Terrain, "t_door_c")
        .expect("t_door_c exists");
    assert_eq!(door.open_result.as_deref(), Some("t_door_o"));
    assert_eq!(door.roof_result.as_deref(), Some("t_wood_roof"));
    assert_eq!(door.bash_result.as_deref(), Some("t_door_b"));
    assert_ne!(reg.id_of(Kind::Item, "rock"), 0, "item rock exists");
    assert_ne!(
        reg.id_of(Kind::Monster, "mon_zombie"),
        0,
        "mon_zombie exists"
    );
    // Ids are per-kind and round-trip.
    let wheel_item = reg.id_of(Kind::Item, "wheel");
    let wheel_part = reg.id_of(Kind::VehiclePart, "wheel");
    assert_ne!(wheel_item, 0);
    assert_ne!(wheel_part, 0);
    assert_eq!(reg.string_of(Kind::VehiclePart, wheel_part), Some("wheel"));
    assert_eq!(reg.string_of(Kind::Item, wheel_item), Some("wheel"));

    // Wire table: every def + the registry strings.
    let td = reg.tiledefs();
    assert_eq!(td.defs.as_ref().unwrap().len(), 13_532);
    assert_eq!(td.connect_groups.as_ref().unwrap().len(), 36);
    let wire_door = td
        .defs
        .as_ref()
        .unwrap()
        .iter()
        .find(|d| d.string_id.as_deref() == Some("t_door_c"))
        .expect("wire t_door_c");
    assert_eq!(wire_door.open_result.as_deref(), Some("t_door_o"));
    assert_eq!(wire_door.roof_result.as_deref(), Some("t_wood_roof"));
    // fg/bg: -1 for every def until the fg_refs join landed
    // (Phase 11 follow-up, 2026-11-24). t_door_c's composed UltiCa
    // entry primaries are fg 6076 / bg 6077 — the same values the
    // importer's validate spot-checks pin against the 0.I-1 tree's
    // bundled tile_config.json (cross-verified against the tileset
    // crate's fg_refs tool, 12,143 ids / 0 diffs).
    assert_eq!(wire_door.fg_ref, 6076);
    assert_eq!(wire_door.bg_ref, 6077);
}

#[test]
fn registry_resolution_error_names_what_was_tried() {
    let bogus = PathBuf::from("/nonexistent/definitely-not-here/defs.json");
    match resolve_defs_path(Some(&bogus)) {
        Ok(p) => {
            // Only acceptable if the environment genuinely provides
            // a defs file through another step of the resolution
            // order — in the test environment it does not.
            panic!("resolution unexpectedly succeeded via {}", p.display());
        }
        Err(e) => {
            assert!(e.contains("tried:"), "error must name the steps: {e}");
            assert!(
                e.contains("explicit path"),
                "error must name the explicit path step: {e}"
            );
            assert!(
                e.contains("CDDA_SIM_DEFS"),
                "error must name the env step: {e}"
            );
        }
    }
}
