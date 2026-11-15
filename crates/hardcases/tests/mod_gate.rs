//! Gate 8.7 (mod support) at data level: a "mod" TileDefs set —
//! content the base game never exported: a new connect group, a
//! new floor/roof pair, a full door transform chain, a vehicle
//! part, a monster, a shader field — flows through the same tables
//! and rules as base content, and the missing-material report
//! names exactly the defs with no material.

use std::collections::HashSet;

use hardcases::{
    classify_transition, floor_at, missing_materials, missing_report_lines, unreferenced_defs,
    Column, DefTable, Floor, Transition,
};
use proto::{TileDefT, TileDefsT, TileKind, PROTOCOL_VERSION};

fn def(id: u32, string_id: &str, kind: TileKind, fg: i32, bg: i32) -> TileDefT {
    let mut d = TileDefT::default();
    d.id = id;
    d.string_id = Some(string_id.into());
    d.kind = kind;
    d.fg_ref = fg;
    d.bg_ref = bg;
    d
}

fn mod_tiledefs() -> TileDefsT {
    let mut wall = def(200, "t_mod_glowall", TileKind::Terrain, 900, -1);
    wall.connect_groups = Some(vec![1]);
    wall.connects_to_groups = Some(vec![1]);
    let mut floor = def(201, "t_mod_floor", TileKind::Terrain, -1, 901);
    floor.roof_result = Some("t_mod_roof".into());
    let roof = def(202, "t_mod_roof", TileKind::Terrain, 902, -1);
    let mut door = def(203, "t_mod_door_c", TileKind::Terrain, 903, -1);
    door.open_result = Some("t_mod_door_o".into());
    door.bash_result = Some("t_mod_door_broken".into());
    let mut door_open = def(204, "t_mod_door_o", TileKind::Terrain, 904, -1);
    door_open.close_result = Some("t_mod_door_c".into());
    let door_broken = def(205, "t_mod_door_broken", TileKind::Terrain, 905, -1);
    let vpart = def(206, "mod_turret", TileKind::VehiclePart, 906, -1);
    let monster = def(207, "mon_mod_husk", TileKind::Monster, 907, -1);
    let field = def(208, "fd_mod_miasma", TileKind::Field, -1, -1);
    // Every real TileDefs set carries t_open_air (the mod set is
    // cumulative over the base game's defs).
    let open_air = def(209, "t_open_air", TileKind::Terrain, -1, -1);
    let mut t = TileDefsT::default();
    t.protocol_version = PROTOCOL_VERSION;
    t.connect_groups = Some(vec!["wall".into(), "mod_glowall".into()]);
    t.defs = Some(vec![
        wall,
        floor,
        roof,
        door,
        door_open,
        door_broken,
        vpart,
        monster,
        field,
        open_air,
    ]);
    t
}

#[test]
fn gate_8_7_mod_defs_flow_through_base_rules() {
    let table = DefTable::from_tiledefs(&mod_tiledefs());
    // New connect group resolves by name and index.
    assert_eq!(table.connect_groups()[1], "mod_glowall");
    assert_eq!(
        table.def(200).unwrap().connect_groups.as_deref(),
        Some(&[1][..])
    );
    // The mod door's full transform chain classifies.
    assert_eq!(classify_transition(&table, 203, 204), Transition::Open);
    assert_eq!(classify_transition(&table, 204, 203), Transition::Close);
    assert_eq!(classify_transition(&table, 203, 205), Transition::Bash);
    // The mod floor roofs the level above it, via the Phase 8
    // roof export, exactly like a base-game floor.
    let col = Column {
        z_min: 0,
        terrain: vec![201, table.open_air_id],
    };
    assert_eq!(floor_at(&table, &col, 1), Floor::RoofOf(201, 202));
}

#[test]
fn gate_8_7_missing_material_report_is_exact() {
    let table = DefTable::from_tiledefs(&mod_tiledefs());
    // Materials exist for everything except the broken-door
    // variant (205) and the monster (207).
    let have: HashSet<u32> = [200, 201, 202, 203, 204, 206].into_iter().collect();
    let missing = missing_materials(&table, &have);
    let ids: Vec<u32> = missing.iter().map(|m| m.id).collect();
    assert_eq!(ids, vec![205, 207]);
    assert_eq!(missing[0].string_id, "t_mod_door_broken");
    assert_eq!(missing[1].kind, "monster");
    let lines = missing_report_lines(&table, &have);
    assert_eq!(
        lines,
        vec![
            "missing terrain 205 t_mod_door_broken".to_string(),
            "missing monster 207 mon_mod_husk".to_string(),
        ]
    );
    // The shader field and open air reference nothing: listed as
    // unreferenced, never as missing (fields render through the
    // effect system; open air renders as the absence of a block).
    let unref: Vec<u32> = unreferenced_defs(&table).iter().map(|m| m.id).collect();
    assert_eq!(unref, vec![208, 209]);
    // With full materials the report is empty: the mod is clean.
    let full: HashSet<u32> = (200..=207).collect();
    assert!(missing_materials(&table, &full).is_empty());
}
