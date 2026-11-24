// CDDA Voxel Frontend — Phase 3 exporter: game-API-agnostic value types.
//
// These plain structs are the ONLY currency between the game adapter
// (voxel_adapter.h) and the serializer/diff core (voxel_exporter.cpp).
// The real fork adapter fills them from CDDA's query APIs; the standalone
// harness (tools/fork-harness) fills them from a mock world with the same
// shapes. Nothing in this header includes a CDDA header.
#pragma once

#include <array>
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
    // Phase 10: sky-exposed — the game's own test, resolved game-side:
    // map::is_outside(p) && !map::is_roofed(p) (docs/phase10-sky-gi.md
    // §3.4/§11). Drives the per-tile sky factor for sky GI.
    bool sky_exposed = false;
    bool operator==( const TileVal &o ) const {
        return terrain == o.terrain && furniture == o.furniture && trap == o.trap &&
               fields == o.fields && displayed_type == o.displayed_type &&
               displayed_intensity == o.displayed_intensity && top_item == o.top_item &&
               item_count == o.item_count && lit == o.lit && seen == o.seen &&
               graffiti == o.graffiti && sky_exposed == o.sky_exposed;
    }
    bool operator!=( const TileVal &o ) const { return !( *this == o ); }
};

// Game time + weather at the player, in wire terms (schema
// EnvironmentUpdate, docs/phase10-sky-gi.md §2.2). Mirrors the wire
// fields 1:1 EXCEPT seq/epoch, which the exporter stamps. Weather is
// string id + resolved numerics, never a closed enum: weather types
// are JSON-defined and mod-extensible.
struct EnvVal {
    int64_t turn = 0;                  // turns since calendar::turn_zero
    float time_of_day_sec = 0.0f;      // 0..86400, fractional
    float sunrise_sec = 0.0f;          // today, game-local
    float sunset_sec = 0.0f;
    uint8_t season = 0;                // season_type: 0 spring .. 3 winter
    float sun_light = 0.0f;            // incident_sunlight(wtype, turn)
    float moon_light = 0.0f;           // incident_moonlight(wtype, turn)
    float sun_azimuth_deg = 0.0f;      // from South, west-positive
    float sun_altitude_deg = 0.0f;     // negative below horizon
    uint8_t moon_phase = 0;            // get_moon_phase(turn), 0..MOON_PHASE_MAX
    std::string weather_id;            // e.g. "weather_rain"
    float light_multiplier = 1.0f;     // weather_type fields, resolved game-side
    float light_modifier = 0.0f;
    float sun_multiplier = 1.0f;
    uint8_t precip_class = 0;          // precip_class: 0 none .. 3 heavy
    bool rains = false;
    float sight_penalty = 0.0f;
    std::optional<std::array<float, 3>> tint_rgb; // weather_type tint_color; absent = no tint
    float tint_strength = 0.0f;
    float temperature_c = 0.0f;
    float windspeed_mph = 0.0f;
    float wind_dir_deg = 0.0f;
    bool lightning_active = false;
    bool operator==( const EnvVal &o ) const {
        return turn == o.turn && time_of_day_sec == o.time_of_day_sec &&
               sunrise_sec == o.sunrise_sec && sunset_sec == o.sunset_sec &&
               season == o.season && sun_light == o.sun_light && moon_light == o.moon_light &&
               sun_azimuth_deg == o.sun_azimuth_deg && sun_altitude_deg == o.sun_altitude_deg &&
               moon_phase == o.moon_phase && weather_id == o.weather_id &&
               light_multiplier == o.light_multiplier && light_modifier == o.light_modifier &&
               sun_multiplier == o.sun_multiplier && precip_class == o.precip_class &&
               rains == o.rains && sight_penalty == o.sight_penalty &&
               tint_rgb == o.tint_rgb && tint_strength == o.tint_strength &&
               temperature_c == o.temperature_c && windspeed_mph == o.windspeed_mph &&
               wind_dir_deg == o.wind_dir_deg && lightning_active == o.lightning_active;
    }
    bool operator!=( const EnvVal &o ) const { return !( *this == o ); }
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
