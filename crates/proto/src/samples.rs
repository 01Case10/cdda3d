//! Canonical sample messages: the "small procedural house" that the C++
//! stub server (tools/stub-server) also builds, plus one sample of every
//! other message type. Shared by the Rust tests, the proto-client, and the
//! cross-language golden checks so all sides assert against the same data.

use crate::{
    DeltaT, EffectEventT, EffectKind, EffectsT, EntityT, EnvironmentUpdateT, Facing, FieldOnTileT,
    HelloAckT, HelloT, InputKind, InputT, LightUpdateT, LitLevel, MessageT, PickResultT, PingT,
    PongT, ResizeT, ShutdownT, SnapshotT, SubmapBlockT, TileChangeT, TileDefT, TileDefsT, TileKind,
    UiFrameT, UiMode, VehiclePartStateT, VehicleT, PROTOCOL_VERSION,
};

/// The canonical sample environment: noon, clear, spring (Phase 10).
/// The C++ stub builds exactly this (tools/stub-server/house.h
/// BuildEnvironment); the fork harness's mock sequence starts here too
/// (tools/fork-harness env_sequence, noon-clear step).
pub fn sample_environment() -> EnvironmentUpdateT {
    EnvironmentUpdateT {
        seq: 1,
        epoch: 1,
        // Day 21, solar noon: 21 * 14400 turns/day + 7200 = 309600,
        // and 7200 turns * 6 s = 43200 s = time_of_day_sec.
        turn: 309_600,
        time_of_day_sec: 43_200.0,
        sunrise_sec: 21_600.0,
        sunset_sec: 64_800.0,
        season: 0, // spring
        sun_light: 125.0,
        moon_light: 1.0,
        sun_azimuth_deg: 180.0,
        sun_altitude_deg: 55.0,
        moon_phase: 4, // MOON_FULL
        weather_id: Some("weather_clear".into()),
        light_multiplier: 1.0,
        light_modifier: 0.0,
        sun_multiplier: 1.0,
        precip_class: 0,
        rains: false,
        sight_penalty: 0.0,
        tint_rgb: None,
        tint_strength: 0.0,
        temperature_c: 20.0,
        windspeed_mph: 5.0,
        wind_dir_deg: 90.0,
        lightning_active: false,
    }
}

// Def ids used by the sample house (the C++ stub uses the same ids).
pub const TER_FLOOR: u32 = 1;
pub const TER_WALL: u32 = 2;
pub const TER_DOOR: u32 = 3;
pub const TER_WINDOW: u32 = 4;
pub const TER_DOOR_OPEN: u32 = 5;
pub const FURN_TABLE: u32 = 20;
pub const FIELD_SMOKE: u32 = 30;
pub const FIELD_FIRE: u32 = 31;
pub const ITEM_ROCK: u32 = 40;
pub const MON_ZOMBIE: u32 = 50;
pub const VPART_WHEEL: u32 = 60;

pub const SUBMAP: usize = 12;
pub const TILES: usize = SUBMAP * SUBMAP;

pub fn idx(x: usize, y: usize) -> usize {
    y * SUBMAP + x
}

pub fn v3(x: i32, y: i32, z: i32) -> crate::Vec3iT {
    crate::Vec3iT { x, y, z }
}

pub fn sample_hello() -> HelloT {
    HelloT {
        protocol_version: PROTOCOL_VERSION,
        game_version: Some("0.G-stub".into()),
        tileset_id: Some("UltiCa".into()),
        tile_pixel_size: 32,
        z_min: -10,
        z_max: 10,
    }
}

pub fn sample_hello_ack() -> HelloAckT {
    HelloAckT {
        protocol_version: PROTOCOL_VERSION,
        engine_version: Some("proto-client-0.1.0".into()),
        screen_width: 1920,
        screen_height: 1080,
        dpi: 1.0,
        capabilities: Some(vec!["ui-shm-v1".into()]),
    }
}

pub fn sample_tiledefs() -> TileDefsT {
    let def =
        |id: u32, string_id: &str, name: &str, kind: TileKind, fg_ref: i32, bg_ref: i32| TileDefT {
            id,
            string_id: Some(string_id.into()),
            name: Some(name.into()),
            kind,
            flags: Some(vec!["TRANSPARENT".into()]),
            hot_flags: 0b101,
            connect_groups: None,
            connects_to_groups: None,
            rotate_to_groups: None,
            // None = no such transform / no looks_like. (The C++ object API
            // cannot distinguish an absent string from an empty one when
            // packing, so both sides use absent for "none"; see docs/protocol.md.)
            open_result: None,
            close_result: None,
            bash_result: None,
            looks_like: None,
            fg_ref,
            bg_ref,
            // Phase 8 additive field: absent = roofs nothing. The
            // sample values stay None so the committed goldens
            // (which encode these samples) remain byte-identical.
            roof_result: None,
            // Lane J additive field: absent = no colour. None
            // here too, for the same golden reason.
            color: None,
        };
    let mut wall = def(TER_WALL, "t_wall", "wall", TileKind::Terrain, 101, -1);
    wall.connect_groups = Some(vec![0]);
    wall.connects_to_groups = Some(vec![0]);
    let mut door = def(
        TER_DOOR,
        "t_door_c",
        "closed door",
        TileKind::Terrain,
        102,
        -1,
    );
    door.open_result = Some("t_door_o".into());
    let mut floor = def(TER_FLOOR, "t_floor", "floor", TileKind::Terrain, -1, 100);
    floor.flags = None;
    TileDefsT {
        connect_groups: Some(vec!["wall".into(), "door".into()]),
        protocol_version: PROTOCOL_VERSION,
        defs: Some(vec![
            floor,
            wall,
            door,
            def(TER_WINDOW, "t_window", "window", TileKind::Terrain, 103, -1),
            def(
                TER_DOOR_OPEN,
                "t_door_o",
                "open door",
                TileKind::Terrain,
                104,
                -1,
            ),
            def(FURN_TABLE, "f_table", "table", TileKind::Furniture, 120, -1),
            def(FIELD_SMOKE, "fd_smoke", "smoke", TileKind::Field, -1, -1),
            def(FIELD_FIRE, "fd_fire", "fire", TileKind::Field, -1, -1),
            def(ITEM_ROCK, "rock", "rock", TileKind::Item, 140, -1),
            def(
                MON_ZOMBIE,
                "mon_zombie",
                "zombie",
                TileKind::Monster,
                150,
                -1,
            ),
            def(
                VPART_WHEEL,
                "wheel",
                "wheel",
                TileKind::VehiclePart,
                160,
                -1,
            ),
        ]),
    }
}

/// The fake house: one 12x12 block, wall perimeter, door at (5,11), window
/// at (2,0), a table, a two-field tile, an item pile, one entity, one
/// vehicle part list. The C++ stub builds exactly this.
pub fn sample_house_block() -> SubmapBlockT {
    let mut terrain = vec![TER_FLOOR; TILES];
    for i in 0..SUBMAP {
        terrain[idx(i, 0)] = TER_WALL;
        terrain[idx(i, SUBMAP - 1)] = TER_WALL;
        terrain[idx(0, i)] = TER_WALL;
        terrain[idx(SUBMAP - 1, i)] = TER_WALL;
    }
    terrain[idx(5, 11)] = TER_DOOR;
    terrain[idx(2, 0)] = TER_WINDOW;

    let mut furniture = vec![0u32; TILES];
    furniture[idx(3, 3)] = FURN_TABLE;

    // Sparse fields: tile (6,6) has TWO fields (smoke + fire), tile (7,6)
    // has one (smoke). field_index points at each tile's first entry.
    let fields = vec![
        FieldOnTileT {
            tile_index: idx(6, 6) as u16,
            type_: FIELD_SMOKE,
            intensity: 2,
        },
        FieldOnTileT {
            tile_index: idx(6, 6) as u16,
            type_: FIELD_FIRE,
            intensity: 1,
        },
        FieldOnTileT {
            tile_index: idx(7, 6) as u16,
            type_: FIELD_SMOKE,
            intensity: 1,
        },
    ];
    let mut field_index = vec![-1i32; TILES];
    field_index[idx(6, 6)] = 0;
    field_index[idx(7, 6)] = 2;
    let mut displayed = vec![
        crate::DisplayedFieldT {
            field_type: 0,
            intensity: 0
        };
        TILES
    ];
    displayed[idx(6, 6)] = crate::DisplayedFieldT {
        field_type: FIELD_FIRE,
        intensity: 1,
    };
    displayed[idx(7, 6)] = crate::DisplayedFieldT {
        field_type: FIELD_SMOKE,
        intensity: 1,
    };

    let mut top_item = vec![0u32; TILES];
    let mut item_count = vec![0u16; TILES];
    top_item[idx(4, 4)] = ITEM_ROCK;
    item_count[idx(4, 4)] = 3;

    let mut graffiti = vec![false; TILES];
    graffiti[idx(1, 1)] = true;

    // Sky exposure (Phase 10): the shared stub-house rule, mirrored in
    // house.h — tiles strictly inside the wall perimeter (the interior
    // floor area, x/y in 1..=10) are indoors and not sky-exposed;
    // everything else (walls, corners, the door/window ring) is.
    let mut outside = vec![true; TILES];
    for y in 1..SUBMAP - 1 {
        for x in 1..SUBMAP - 1 {
            outside[idx(x, y)] = false;
        }
    }

    SubmapBlockT {
        origin: Some(v3(0, 0, 0)),
        terrain: Some(terrain),
        furniture: Some(furniture),
        trap: Some(vec![0; TILES]),
        fields: Some(fields),
        field_index: Some(field_index),
        displayed_field: Some(displayed),
        top_item: Some(top_item),
        item_count: Some(item_count),
        lit: Some(vec![LitLevel::LIT; TILES]),
        seen: Some(vec![255; TILES]),
        has_graffiti: Some(graffiti),
        outside: Some(outside),
        // Phase 11 S3 frac/tint: this fixture predates them and
        // reads as absent (= derive frac from lit/seen, tint white).
        light_frac: None,
        light_tint: None,
    }
}

pub fn sample_entity() -> EntityT {
    EntityT {
        id: 1,
        type_def: MON_ZOMBIE,
        sprite_id: Some("mon_zombie".into()),
        pos: Some(v3(6, 5, 0)),
        facing: Facing::RIGHT,
        overlays: Some(vec![
            "overlay_wielded_rock".into(),
            "overlay_worn_jacket".into(),
        ]),
        hp_frac: 200,
        flags: 2,
    }
}

pub fn sample_vehicle() -> VehicleT {
    VehicleT {
        id: 7,
        name: Some("car".into()),
        face_deg: 37.5,
        parts: Some(vec![VehiclePartStateT {
            vpart: VPART_WHEEL,
            abs_pos: Some(v3(20, 20, 0)),
            mount: Some(v3(0, 0, 0)),
            hp_frac: 255,
            flags: 4,
        }]),
    }
}

pub fn sample_snapshot() -> SnapshotT {
    SnapshotT {
        view_origin: Some(v3(0, 0, 0)),
        view_radius: 60,
        blocks: Some(vec![sample_house_block()]),
        memory_blocks: None,
        entities: Some(vec![sample_entity()]),
        vehicles: Some(vec![sample_vehicle()]),
        seq: 1,
        epoch: 1,
        // Phase 10 piggyback: a snapshot always carries the current
        // environment (the C++ stub's BuildSnapshot does the same).
        environment: Some(Box::new(sample_environment())),
    }
}

// ---------------------------------------------------------------------
// Roofed house (2026-11-17): fake data in the exact shape the fork's
// exporter sends for a player standing inside a roofed house, built
// from the game's own data and the exporter's own rules — NOT from
// the engine's stub-house layout:
//
// - View window: `Exporter::build_snapshot` with the default
//   `ViewSpec` (voxel_exporter.h): centre = player tile, radius
//   <= 60, `z_below = 1, z_above = 1` around the player's z. A
//   player at z 0 therefore receives SubmapBlocks for z -1, 0, +1.
// - One SubmapBlock per (submap, z): origin = (sx*12, sy*12, z),
//   per-tile arrays of 144, row-major (block_to_wire).
// - Ground floor (bungalow-style mapgen, e.g. bungalow01_1):
//   grass outside, a 6x6 footprint at (3..=8, 3..=8) — wall
//   perimeter, wood floor inside, door in the south wall, windows
//   in the north/west walls, a table, an item pile.
// - Roof level: bungalow01_roof is a separate mapgen at z +1 with
//   fill `t_shingle_flat_roof` over the whole footprint (walls
//   included) and `t_open_air` everywhere else. That explicit form
//   is `sample_roofed_house_snapshot`.
// - The implicit form also exists in the game: where the z +1 tile
//   is `t_open_air`, the roof is each z 0 def's `roof` field
//   (ter_t::roof, exported as TileDef.roof_result). Real values
//   from the pinned game data: t_floor -> t_wood_treated_roof,
//   t_wall -> t_flat_roof, t_window -> t_flat_roof,
//   t_door_c -> t_wood_roof, t_grass -> none. That form is
//   `sample_implicit_roof_snapshot` (identical except z +1 is all
//   open air).
// - Player entity: the real adapter emits the avatar first with
//   id 1, type_def 0, sprite_id "player" (voxel_real_adapter.cpp).
// ---------------------------------------------------------------------

pub const TER_TREE: u32 = 12;
pub const TER_CHAINFENCE: u32 = 13;
pub const TER_GRASS: u32 = 6;
pub const TER_ROOF_SHINGLE: u32 = 7;
pub const TER_OPEN_AIR: u32 = 8;
pub const TER_ROOF_FLAT: u32 = 9;
pub const TER_ROOF_WOOD_TREATED: u32 = 10;
pub const TER_ROOF_WOOD: u32 = 11;

/// TileDefs for the roofed house, with the game's real `roof`
/// values carried in `roof_result`.
pub fn sample_roofed_tiledefs() -> TileDefsT {
    let def = |id: u32, string_id: &str, name: &str, roof: Option<&str>| TileDefT {
        id,
        string_id: Some(string_id.into()),
        name: Some(name.into()),
        kind: TileKind::Terrain,
        flags: None,
        hot_flags: 0,
        connect_groups: None,
        connects_to_groups: None,
        rotate_to_groups: None,
        open_result: None,
        close_result: None,
        bash_result: None,
        looks_like: None,
        fg_ref: -1,
        bg_ref: -1,
        roof_result: roof.map(|s| s.to_string()),
        color: None,
    };
    let mut wall = def(TER_WALL, "t_wall", "wall", Some("t_flat_roof"));
    wall.connect_groups = Some(vec![0]);
    wall.connects_to_groups = Some(vec![0]);
    let mut door = def(TER_DOOR, "t_door_c", "closed door", Some("t_wood_roof"));
    door.open_result = Some("t_door_o".into());
    TileDefsT {
        connect_groups: Some(vec!["wall".into()]),
        protocol_version: PROTOCOL_VERSION,
        defs: Some(vec![
            def(TER_OPEN_AIR, "t_open_air", "open air", None),
            def(TER_GRASS, "t_grass", "grass", None),
            def(TER_FLOOR, "t_floor", "floor", Some("t_wood_treated_roof")),
            wall,
            door,
            def(TER_DOOR_OPEN, "t_door_o", "open door", Some("t_wood_roof")),
            def(TER_WINDOW, "t_window", "window", Some("t_flat_roof")),
            def(
                TER_ROOF_SHINGLE,
                "t_shingle_flat_roof",
                "shingle flat roof",
                None,
            ),
            def(TER_ROOF_FLAT, "t_flat_roof", "flat roof", None),
            def(
                TER_ROOF_WOOD_TREATED,
                "t_wood_treated_roof",
                "wood treated roof",
                None,
            ),
            def(TER_ROOF_WOOD, "t_wood_roof", "wood roof", None),
            {
                let mut t = def(FURN_TABLE, "f_table", "table", None);
                t.kind = TileKind::Furniture;
                t
            },
        ]),
    }
}

fn roofed_house_block(z: i32, explicit_roof: bool) -> SubmapBlockT {
    let mut terrain = vec![TER_GRASS; TILES];
    let mut furniture = vec![0u32; TILES];
    let mut top_item = vec![0u32; TILES];
    let mut item_count = vec![0u16; TILES];
    let (lit, seen) = if z < 0 {
        // The z_below = 1 window filler under a surface house:
        // open air, dark, never seen.
        terrain = vec![TER_OPEN_AIR; TILES];
        (LitLevel::DARK, 0u8)
    } else if z == 0 {
        for y in 3..=8usize {
            for x in 3..=8usize {
                let edge = x == 3 || x == 8 || y == 3 || y == 8;
                terrain[idx(x, y)] = if edge { TER_WALL } else { TER_FLOOR };
            }
        }
        terrain[idx(6, 8)] = TER_DOOR;
        terrain[idx(5, 3)] = TER_WINDOW;
        terrain[idx(3, 6)] = TER_WINDOW;
        furniture[idx(5, 5)] = FURN_TABLE;
        top_item[idx(4, 4)] = ITEM_ROCK;
        item_count[idx(4, 4)] = 3;
        (LitLevel::LIT, 255u8)
    } else {
        // z +1: the roof mapgen level (explicit) or plain open
        // air over everything (implicit — roofs then come from
        // the z 0 defs' roof_result).
        terrain = vec![TER_OPEN_AIR; TILES];
        if explicit_roof {
            for y in 3..=8usize {
                for x in 3..=8usize {
                    terrain[idx(x, y)] = TER_ROOF_SHINGLE;
                }
            }
        }
        (LitLevel::BRIGHT, 255u8)
    };
    SubmapBlockT {
        origin: Some(v3(0, 0, z)),
        terrain: Some(terrain),
        furniture: Some(furniture),
        trap: Some(vec![0; TILES]),
        fields: Some(vec![]),
        field_index: Some(vec![-1i32; TILES]),
        displayed_field: Some(vec![
            crate::DisplayedFieldT {
                field_type: 0,
                intensity: 0
            };
            TILES
        ]),
        top_item: Some(top_item),
        item_count: Some(item_count),
        lit: Some(vec![lit; TILES]),
        seen: Some(vec![seen; TILES]),
        has_graffiti: Some(vec![false; TILES]),
        // Phase 10 outside[] is exercised by the stub-house samples;
        // these fixtures predate it and read as absent (= not exposed).
        outside: None,
        // Phase 11 S3 frac/tint likewise absent in these fixtures.
        light_frac: None,
        light_tint: None,
    }
}

/// The avatar entity the real adapter emits (id 1, type 0,
/// sprite "player"), standing inside the house at (6, 6, 0).
pub fn sample_player_entity() -> EntityT {
    EntityT {
        id: 1,
        type_def: 0,
        sprite_id: Some("player".into()),
        pos: Some(v3(6, 6, 0)),
        facing: Facing::RIGHT,
        overlays: Some(vec!["mut_base".into()]),
        hp_frac: 255,
        flags: 0,
    }
}

fn roofed_snapshot(explicit_roof: bool) -> SnapshotT {
    SnapshotT {
        view_origin: Some(v3(6, 6, 0)),
        view_radius: 12,
        blocks: Some(vec![
            roofed_house_block(-1, explicit_roof),
            roofed_house_block(0, explicit_roof),
            roofed_house_block(1, explicit_roof),
        ]),
        memory_blocks: None,
        entities: Some(vec![sample_player_entity()]),
        vehicles: None,
        seq: 1,
        epoch: 1,
        environment: None,
    }
}

/// Snapshot for a player inside a house whose roof arrives as
/// explicit `t_shingle_flat_roof` terrain at z +1 (bungalow-style
/// roof mapgen).
pub fn sample_roofed_house_snapshot() -> SnapshotT {
    roofed_snapshot(true)
}

/// Same house, but z +1 is all `t_open_air`: the roof exists
/// only as the z 0 defs' `roof_result` values in
/// [`sample_roofed_tiledefs`].
pub fn sample_implicit_roof_snapshot() -> SnapshotT {
    roofed_snapshot(false)
}

/// TileDefs for the terrain-model wiring gate: a grass
/// field's worth of terrain kinds — ground (grass, floor),
/// standing (tree, wall), a fence — plus the floor's
/// `roof_result`, so the level above seats the roof
/// material's model from open air.
pub fn sample_terrain_tiledefs() -> TileDefsT {
    let def = |id: u32, string_id: &str, roof: Option<&str>| TileDefT {
        id,
        string_id: Some(string_id.into()),
        name: Some(string_id.into()),
        kind: TileKind::Terrain,
        flags: None,
        hot_flags: 0,
        connect_groups: None,
        connects_to_groups: None,
        rotate_to_groups: None,
        open_result: None,
        close_result: None,
        bash_result: None,
        looks_like: None,
        fg_ref: -1,
        bg_ref: -1,
        roof_result: roof.map(|s| s.to_string()),
        color: None,
    };
    TileDefsT {
        connect_groups: Some(vec![]),
        protocol_version: PROTOCOL_VERSION,
        defs: Some(vec![
            def(TER_OPEN_AIR, "t_open_air", None),
            def(TER_GRASS, "t_grass", None),
            def(TER_TREE, "t_tree", None),
            def(TER_CHAINFENCE, "t_chainfence", None),
            def(TER_WALL, "t_wall", None),
            def(TER_FLOOR, "t_floor", Some("t_shingle_flat_roof")),
            def(TER_ROOF_SHINGLE, "t_shingle_flat_roof", None),
        ]),
    }
}

fn terrain_block(z: i32, terrain: Vec<u32>) -> SubmapBlockT {
    SubmapBlockT {
        origin: Some(v3(0, 0, z)),
        terrain: Some(terrain),
        furniture: Some(vec![0; TILES]),
        trap: Some(vec![0; TILES]),
        fields: Some(vec![]),
        field_index: Some(vec![-1i32; TILES]),
        displayed_field: Some(vec![
            crate::DisplayedFieldT {
                field_type: 0,
                intensity: 0
            };
            TILES
        ]),
        top_item: Some(vec![0; TILES]),
        item_count: Some(vec![0; TILES]),
        lit: Some(vec![LitLevel::LIT; TILES]),
        seen: Some(vec![255; TILES]),
        has_graffiti: Some(vec![false; TILES]),
        outside: None,
        light_frac: None,
        light_tint: None,
    }
}

/// Snapshot for the terrain-model wiring gate: one 12x12
/// submap at z 0 — grass, a chain-link fence run along
/// y = 5 with a south leg at (6, 6), one isolated fence
/// tile at (9, 9), a tree at (2, 2), a wall at (8, 2) and
/// a 2x2 floor patch at (3..4, 8..9) — under a z +1 level
/// of plain open air (the floor patch's roof arrives via
/// `roof_result`).
pub fn sample_terrain_snapshot() -> SnapshotT {
    let mut t0 = vec![TER_GRASS; TILES];
    t0[idx(4, 5)] = TER_CHAINFENCE;
    t0[idx(5, 5)] = TER_CHAINFENCE;
    t0[idx(6, 5)] = TER_CHAINFENCE;
    t0[idx(6, 6)] = TER_CHAINFENCE;
    t0[idx(9, 9)] = TER_CHAINFENCE;
    t0[idx(2, 2)] = TER_TREE;
    t0[idx(8, 2)] = TER_WALL;
    for (x, y) in [(3, 8), (4, 8), (3, 9), (4, 9)] {
        t0[idx(x, y)] = TER_FLOOR;
    }
    SnapshotT {
        view_origin: Some(v3(6, 6, 0)),
        view_radius: 12,
        blocks: Some(vec![
            terrain_block(0, t0),
            terrain_block(1, vec![TER_OPEN_AIR; TILES]),
        ]),
        memory_blocks: None,
        entities: None,
        vehicles: None,
        seq: 1,
        epoch: 1,
        environment: None,
    }
}

pub fn sample_delta() -> DeltaT {
    DeltaT {
        seq: 2,
        epoch: 1,
        tile_changes: Some(vec![TileChangeT {
            pos: Some(v3(5, 11, 0)),
            terrain: TER_DOOR_OPEN,
            furniture: 0,
            trap: 0,
            fields: None,
            top_item: 0,
            item_count: 0,
            lit: LitLevel::LIT,
            seen: 255,
            has_graffiti: false,
            // Schema fix 2026-11-15: TileChange carries the displayed
            // field. This tile has no fields, so 0/0 = none.
            displayed_field: Some(crate::displayed_field(0, 0)),
            // Phase 10: the door tile sits on the house perimeter —
            // sky-exposed, matching the block rule in sample_house_block.
            outside: true,
            // Phase 11 S3: a LIT tile part-way between the vision
            // threshold and full light, uncoloured light.
            light_frac: 128,
            light_tint_r: 255,
            light_tint_g: 255,
            light_tint_b: 255,
        }]),
        entity_updates: Some(vec![sample_entity()]),
        entity_removed: Some(vec![99]),
        vehicle_updates: Some(vec![sample_vehicle()]),
        vehicle_removed: Some(vec![98]),
        // Usually absent; present only when the environment changed
        // at this send boundary (Phase 10 §2.3).
        environment: None,
    }
}

pub fn sample_input() -> InputT {
    InputT {
        kind: InputKind::Key,
        key_code: 'i' as i32,
        text: None,
        mouse_x: 0,
        mouse_y: 0,
        button: -1,
        pressed: true,
        wheel_delta: 0,
        modifiers: 0,
        focused: false,
    }
}

/// One sample of every union member, for the round-trip gate (1.1).
pub fn all_sample_messages() -> Vec<MessageT> {
    vec![
        MessageT::Hello(Box::new(sample_hello())),
        MessageT::HelloAck(Box::new(sample_hello_ack())),
        MessageT::TileDefs(Box::new(sample_tiledefs())),
        MessageT::Snapshot(Box::new(sample_snapshot())),
        MessageT::Delta(Box::new(sample_delta())),
        MessageT::LightUpdate(Box::new(LightUpdateT {
            seq: 3,
            epoch: 1,
            origin: Some(v3(0, 0, 0)),
            lit: Some(vec![
                LitLevel::DARK,
                LitLevel::LOW,
                LitLevel::BRIGHT_ONLY,
                LitLevel::LIT,
                LitLevel::BRIGHT,
                LitLevel::MEMORIZED,
                LitLevel::BLANK,
            ]),
            seen: Some(vec![0, 32, 64, 128, 200, 255, 0]),
            // Phase 11 S3: frac follows the lit row (0 except the
            // LOW/LIT fractions and BRIGHT = 255); the LOW tile is
            // warm-tinted, the rest uncoloured (white).
            light_frac: Some(vec![0, 100, 0, 200, 255, 0, 0]),
            light_tint: Some(vec![
                255, 255, 255, // DARK
                255, 180, 80, // LOW, warm
                255, 255, 255, // BRIGHT_ONLY
                255, 255, 255, // LIT
                255, 255, 255, // BRIGHT
                255, 255, 255, // MEMORIZED
                255, 255, 255, // BLANK
            ]),
        })),
        MessageT::EnvironmentUpdate(Box::new(sample_environment())),
        MessageT::Effects(Box::new(EffectsT {
            seq: 4,
            events: Some(vec![EffectEventT {
                kind: EffectKind::Projectile,
                effect_id: Some("bullet".into()),
                from_pos: Some(v3(1, 1, 0)),
                to_pos: Some(v3(6, 5, 0)),
                duration_ms: 120,
            }]),
        })),
        MessageT::UiFrame(Box::new(UiFrameT {
            buffer_index: 1,
            width: 1600,
            height: 800,
            stride: 6400,
            seq: 9,
            dirty_x: 10,
            dirty_y: 20,
            dirty_w: 300,
            dirty_h: 200,
            mode: UiMode::UiInput,
            terrain_rect_x: 0,
            terrain_rect_y: 0,
            terrain_rect_w: 800,
            terrain_rect_h: 800,
        })),
        MessageT::Input(Box::new(sample_input())),
        MessageT::Resize(Box::new(ResizeT {
            width: 1280,
            height: 720,
            dpi: 1.25,
        })),
        MessageT::PickResult(Box::new(PickResultT {
            request_id: 42,
            hit: true,
            pos: Some(v3(3, 4, 0)),
        })),
        MessageT::Ping(Box::new(PingT {
            nonce: 1234,
            sent_ms: 5000,
        })),
        MessageT::Pong(Box::new(PongT {
            nonce: 1234,
            sent_ms: 5000,
        })),
        MessageT::Shutdown(Box::new(ShutdownT {
            reason: Some("test done".into()),
        })),
    ]
}
