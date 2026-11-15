//! Canonical sample messages: the "small procedural house" that the C++
//! stub server (tools/stub-server) also builds, plus one sample of every
//! other message type. Shared by the Rust tests, the proto-client, and the
//! cross-language golden checks so all sides assert against the same data.

use crate::{
    DeltaT, EffectEventT, EffectKind, EffectsT, EntityT, Facing, FieldOnTileT, HelloAckT, HelloT,
    InputKind, InputT, LightUpdateT, LitLevel, MessageT, PickResultT, PingT, PongT, ResizeT,
    ShutdownT, SnapshotT, SubmapBlockT, TileChangeT, TileDefT, TileDefsT, TileKind, UiFrameT,
    UiMode, VehiclePartStateT, VehicleT, PROTOCOL_VERSION,
};

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
        }]),
        entity_updates: Some(vec![sample_entity()]),
        entity_removed: Some(vec![99]),
        vehicle_updates: Some(vec![sample_vehicle()]),
        vehicle_removed: Some(vec![98]),
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
        })),
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
