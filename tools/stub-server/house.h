// The fake procedural house + tile defs, shared by the stub server and the
// C++ round-trip/golden tool. These builders MUST stay semantically in sync
// with crates/proto/src/samples.rs (the cross-language test compares them).
#pragma once

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

#include "cdda_ipc_generated.h"

namespace house {

using namespace cdda::ipc;

constexpr uint32_t kProtocolVersion = 1;
constexpr uint32_t TER_FLOOR = 1;
constexpr uint32_t TER_WALL = 2;
constexpr uint32_t TER_DOOR = 3;
constexpr uint32_t TER_WINDOW = 4;
constexpr uint32_t TER_DOOR_OPEN = 5;
constexpr uint32_t FURN_TABLE = 20;
constexpr uint32_t FIELD_SMOKE = 30;
constexpr uint32_t FIELD_FIRE = 31;
constexpr uint32_t ITEM_ROCK = 40;
constexpr uint32_t MON_ZOMBIE = 50;
constexpr uint32_t VPART_WHEEL = 60;
constexpr int SUBMAP = 12;
constexpr int TILES = SUBMAP * SUBMAP;

inline int idx(int x, int y) { return y * SUBMAP + x; }

inline HelloT BuildHello() {
    HelloT h;
    h.protocol_version = kProtocolVersion;
    h.game_version = "0.G-stub";
    h.tileset_id = "UltiCa";
    h.tile_pixel_size = 32;
    h.z_min = -10;
    h.z_max = 10;
    return h;
}

inline std::unique_ptr<TileDefT> Def(uint32_t id, const char* sid, const char* name,
                                     TileKind kind, int32_t fg, int32_t bg) {
    auto d = std::make_unique<TileDefT>();
    d->id = id;
    d->string_id = sid;
    d->name = name;
    d->kind = kind;
    d->flags = {"TRANSPARENT"};
    d->hot_flags = 0b101;
    d->fg_ref = fg;
    d->bg_ref = bg;
    return d;
}

inline TileDefsT BuildTileDefs() {
    TileDefsT td;
    td.connect_groups = {"wall", "door"};
    td.protocol_version = kProtocolVersion;
    auto floor = Def(TER_FLOOR, "t_floor", "floor", TileKind_Terrain, -1, 100);
    floor->flags.clear();
    td.defs.push_back(std::move(floor));
    auto wall = Def(TER_WALL, "t_wall", "wall", TileKind_Terrain, 101, -1);
    wall->connect_groups = {0};
    wall->connects_to_groups = {0};
    td.defs.push_back(std::move(wall));
    auto door = Def(TER_DOOR, "t_door_c", "closed door", TileKind_Terrain, 102, -1);
    door->open_result = "t_door_o";
    td.defs.push_back(std::move(door));
    td.defs.push_back(Def(TER_WINDOW, "t_window", "window", TileKind_Terrain, 103, -1));
    td.defs.push_back(Def(TER_DOOR_OPEN, "t_door_o", "open door", TileKind_Terrain, 104, -1));
    td.defs.push_back(Def(FURN_TABLE, "f_table", "table", TileKind_Furniture, 120, -1));
    td.defs.push_back(Def(FIELD_SMOKE, "fd_smoke", "smoke", TileKind_Field, -1, -1));
    td.defs.push_back(Def(FIELD_FIRE, "fd_fire", "fire", TileKind_Field, -1, -1));
    td.defs.push_back(Def(ITEM_ROCK, "rock", "rock", TileKind_Item, 140, -1));
    td.defs.push_back(Def(MON_ZOMBIE, "mon_zombie", "zombie", TileKind_Monster, 150, -1));
    td.defs.push_back(Def(VPART_WHEEL, "wheel", "wheel", TileKind_VehiclePart, 160, -1));
    return td;
}

inline SubmapBlockT BuildBlock() {
    SubmapBlockT b;
    b.origin = std::make_unique<Vec3i>(0, 0, 0);
    b.terrain.assign(TILES, TER_FLOOR);
    for (int i = 0; i < SUBMAP; ++i) {
        b.terrain[idx(i, 0)] = TER_WALL;
        b.terrain[idx(i, SUBMAP - 1)] = TER_WALL;
        b.terrain[idx(0, i)] = TER_WALL;
        b.terrain[idx(SUBMAP - 1, i)] = TER_WALL;
    }
    b.terrain[idx(5, 11)] = TER_DOOR;
    b.terrain[idx(2, 0)] = TER_WINDOW;
    b.furniture.assign(TILES, 0);
    b.furniture[idx(3, 3)] = FURN_TABLE;
    b.trap.assign(TILES, 0);

    // Sparse fields: tile (6,6) has TWO fields, tile (7,6) has one.
    auto f1 = std::make_unique<FieldOnTileT>();
    f1->tile_index = static_cast<uint16_t>(idx(6, 6));
    f1->type = FIELD_SMOKE;
    f1->intensity = 2;
    b.fields.push_back(std::move(f1));
    auto f2 = std::make_unique<FieldOnTileT>();
    f2->tile_index = static_cast<uint16_t>(idx(6, 6));
    f2->type = FIELD_FIRE;
    f2->intensity = 1;
    b.fields.push_back(std::move(f2));
    auto f3 = std::make_unique<FieldOnTileT>();
    f3->tile_index = static_cast<uint16_t>(idx(7, 6));
    f3->type = FIELD_SMOKE;
    f3->intensity = 1;
    b.fields.push_back(std::move(f3));

    b.field_index.assign(TILES, -1);
    b.field_index[idx(6, 6)] = 0;
    b.field_index[idx(7, 6)] = 2;
    b.displayed_field.assign(TILES, DisplayedField(0, 0));
    b.displayed_field[idx(6, 6)] = DisplayedField(FIELD_FIRE, 1);
    b.displayed_field[idx(7, 6)] = DisplayedField(FIELD_SMOKE, 1);

    b.top_item.assign(TILES, 0);
    b.item_count.assign(TILES, 0);
    b.top_item[idx(4, 4)] = ITEM_ROCK;
    b.item_count[idx(4, 4)] = 3;

    b.lit.assign(TILES, LitLevel_LIT);
    b.seen.assign(TILES, 255);
    b.has_graffiti.assign(TILES, false);
    b.has_graffiti[idx(1, 1)] = true;
    return b;
}

inline SnapshotT BuildSnapshot() {
    SnapshotT s;
    s.view_origin = std::make_unique<Vec3i>(0, 0, 0);
    s.view_radius = 60;
    auto block = std::make_unique<SubmapBlockT>(BuildBlock());
    s.blocks.push_back(std::move(block));
    auto e = std::make_unique<EntityT>();
    e->id = 1;
    e->type_def = MON_ZOMBIE;
    e->sprite_id = "mon_zombie";
    e->pos = std::make_unique<Vec3i>(6, 5, 0);
    e->facing = Facing_RIGHT;
    e->overlays = {"overlay_wielded_rock", "overlay_worn_jacket"};
    e->hp_frac = 200;
    e->flags = 2;
    s.entities.push_back(std::move(e));
    auto v = std::make_unique<VehicleT>();
    v->id = 7;
    v->name = "car";
    v->face_deg = 37.5f;
    auto part = std::make_unique<VehiclePartStateT>();
    part->vpart = VPART_WHEEL;
    part->abs_pos = std::make_unique<Vec3i>(20, 20, 0);
    part->mount = std::make_unique<Vec3i>(0, 0, 0);
    part->hp_frac = 255;
    part->flags = 4;
    v->parts.push_back(std::move(part));
    s.vehicles.push_back(std::move(v));
    s.seq = 1;
    s.epoch = 1;
    return s;
}

}  // namespace house
