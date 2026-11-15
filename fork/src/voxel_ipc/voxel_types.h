// CDDA Voxel Frontend — Phase 3 exporter: game-API-agnostic value types.
//
// These plain structs are the ONLY currency between the game adapter
// (voxel_adapter.h) and the serializer/diff core (voxel_exporter.cpp).
// The real fork adapter fills them from CDDA's query APIs; the standalone
// harness (tools/fork-harness) fills them from a mock world with the same
// shapes. Nothing in this header includes a CDDA header.
#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace voxel_ipc
{

inline constexpr int kSubmapSize = 12;              // SEEX/SEEY (map_scale_constants.h)
inline constexpr int kTilesPerBlock = 144;          // 12*12
inline constexpr int kMaxViewDistance = 60;         // MAX_VIEW_DISTANCE
inline constexpr int kZMin = -10;                   // OVERMAP_DEPTH
inline constexpr int kZMax = 10;                    // OVERMAP_HEIGHT
inline constexpr uint32_t kProtocolVersion = 1;     // docs/protocol.md
inline constexpr uint32_t kMaxFrame = 32u * 1024 * 1024;

// lit_level (src/lightmap.h:96) — exactly 7 members, exported verbatim.
// Values match schema LitLevel and the game's enum order.
enum class Lit : uint8_t {
    DARK = 0, LOW = 1, BRIGHT_ONLY = 2, LIT = 3, BRIGHT = 4, MEMORIZED = 5, BLANK = 6
};

// FacingDirection (src/creature.h:217) — flip only, no other values exist.
enum class Facing : uint8_t { NONE = 0, LEFT = 1, RIGHT = 2 };

struct FieldVal {
    uint32_t type = 0;
    uint8_t intensity = 0;
    bool operator==( const FieldVal &o ) const {
        return type == o.type && intensity == o.intensity;
    }
    bool operator!=( const FieldVal &o ) const { return !( *this == o ); }
};

// Everything the exporter needs about one tile, in wire terms.
struct TileVal {
    uint32_t terrain = 0;
    uint32_t furniture = 0;
    uint32_t trap = 0;
    std::vector<FieldVal> fields;           // full (type,intensity) list, sorted by type
    uint32_t displayed_type = 0;            // field::displayed_field_type()
    uint8_t displayed_intensity = 0;        // field::displayed_intensity()
    uint32_t top_item = 0;                  // maptile::get_uppermost_item(), 0 = none
    uint16_t item_count = 0;                // i_at() stack size
    Lit lit = Lit::BLANK;
    uint8_t seen = 0;                       // quantized seen_cache float, 0..255
    bool graffiti = false;
    bool operator==( const TileVal &o ) const {
        return terrain == o.terrain && furniture == o.furniture && trap == o.trap &&
               fields == o.fields && displayed_type == o.displayed_type &&
               displayed_intensity == o.displayed_intensity && top_item == o.top_item &&
               item_count == o.item_count && lit == o.lit && seen == o.seen &&
               graffiti == o.graffiti;
    }
    bool operator!=( const TileVal &o ) const { return !( *this == o ); }
};

struct EntVal {
    uint64_t id = 0;
    uint32_t type_def = 0;
    std::string sprite_id;
    int x = 0, y = 0, z = 0;
    Facing facing = Facing::NONE;
    std::vector<std::string> overlays;      // get_overlay_ids() order — never re-sorted
    uint8_t hp_frac = 255;
    uint32_t flags = 0;
    bool operator==( const EntVal &o ) const {
        return id == o.id && type_def == o.type_def && sprite_id == o.sprite_id &&
               x == o.x && y == o.y && z == o.z && facing == o.facing &&
               overlays == o.overlays && hp_frac == o.hp_frac && flags == o.flags;
    }
    bool operator!=( const EntVal &o ) const { return !( *this == o ); }
};

struct VPartVal {
    uint32_t vpart = 0;
    int ax = 0, ay = 0, az = 0;             // absolute pos, game-side (bub_part_pos)
    int mx = 0, my = 0, mz = 0;             // mount (vehicle_part::mount)
    uint8_t hp_frac = 255;
    uint32_t flags = 0;                     // bit0 open, bit1 broken, bit2 roof
    bool operator==( const VPartVal &o ) const {
        return vpart == o.vpart && ax == o.ax && ay == o.ay && az == o.az &&
               mx == o.mx && my == o.my && mz == o.mz && hp_frac == o.hp_frac &&
               flags == o.flags;
    }
    bool operator!=( const VPartVal &o ) const { return !( *this == o ); }
};

struct VehVal {
    uint64_t id = 0;
    std::string name;
    float face_deg = 0.0f;                  // vehicle::face.dir(), float degrees
    std::vector<VPartVal> parts;
    bool operator==( const VehVal &o ) const {
        return id == o.id && name == o.name && face_deg == o.face_deg && parts == o.parts;
    }
    bool operator!=( const VehVal &o ) const { return !( *this == o ); }
};

// One TileDef in wire terms (schema TileDef). Absent optional = no transform.
struct DefVal {
    uint32_t id = 0;
    std::string string_id;
    std::string name;
    uint8_t kind = 0;                       // schema TileKind
    std::vector<std::string> flags;         // string flags as authored
    uint64_t hot_flags = 0;                 // resolved ter_furn_flag bitset (low 64 bits)
    std::vector<uint16_t> connect_groups;
    std::vector<uint16_t> connects_to_groups;
    std::vector<uint16_t> rotate_to_groups;
    std::optional<std::string> open_result;
    std::optional<std::string> close_result;
    std::optional<std::string> bash_result; // ter bash->ter_set / furn bash->furn_set
    std::optional<std::string> roof_result; // ter_t::roof (terrain only; Phase 8, gate 8.4b)
    std::optional<std::string> looks_like;
    int32_t fg_ref = -1;
    int32_t bg_ref = -1;
};

// A remembered tile (avatar map memory): terrain/furniture ONLY by contract.
struct MemVal {
    int x = 0, y = 0, z = 0;
    uint32_t terrain = 0;
    uint32_t furniture = 0;
};

} // namespace voxel_ipc
