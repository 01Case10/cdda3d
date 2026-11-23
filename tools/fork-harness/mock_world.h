// Fork harness mock world: a tiny game-state layer with the SAME query
// shapes as the real adapter (per-tile query, creature list, vehicle list
// with game-side absolute part positions, map memory, def registries).
// Seeded and deterministic. Worlds: shelter, house, road+field,
// multi-storey+basement, vehicle-on-road (plan Part II shared fixtures).
#pragma once

#include <cstdint>
#include <map>
#include <tuple>
#include <vector>

#include "voxel_adapter.h"
#include "voxel_refs.h"

namespace mock
{
using namespace voxel_ipc;

// Def ids (wire space, per kind distinct ranges like the Phase 1 stub).
inline constexpr uint32_t T_FLOOR = 1, T_WALL = 2, T_DOOR_C = 3, T_WINDOW = 4,
                          T_DOOR_O = 5, T_GRASS = 6, T_ROAD = 7, T_DIRT = 8,
                          T_ROOF = 9, T_OPEN_AIR = 10, T_BASHED_DOOR = 11;
inline constexpr uint32_t F_TABLE = 20, F_BED = 21;
inline constexpr uint32_t TRAP_SPIKE = 25;
inline constexpr uint32_t FD_SMOKE = 30, FD_FIRE = 31;
inline constexpr uint32_t IT_ROCK = 40, IT_CAN = 41;
inline constexpr uint32_t MON_ZOMBIE = 50, MON_DOG = 51;
inline constexpr uint32_t VP_FRAME = 60, VP_WHEEL = 61, VP_DOOR = 62, VP_ROOF = 63;

struct Rng {
    uint64_t s;
    explicit Rng( uint64_t seed ) : s( seed ? seed : 0x9e3779b97f4a7c15ULL ) {}
    uint64_t next() {
        s ^= s >> 12; s ^= s << 25; s ^= s >> 27;
        return s * 0x2545F4914F6CDD1DULL;
    }
    int range( int n ) { return static_cast<int>( next() % static_cast<uint64_t>( n ) ); }
};

using Key = std::tuple<int, int, int>;

struct World {
    std::map<Key, TileVal> tiles;   // sparse; missing = default tile (see query)
    TileVal default_tile;           // grass field at z=0 etc, set per world
    std::vector<EntVal> ents;
    std::vector<VehVal> vehs;
    std::vector<MemVal> memory;
    EnvVal env;                     // current environment (Phase 10); tests mutate it
    int minx = -24, maxx = 24, miny = -24, maxy = 24;

    TileVal query( int x, int y, int z ) const {
        auto it = tiles.find( Key{ x, y, z } );
        if( it != tiles.end() ) { return it->second; }
        TileVal t = default_tile;
        if( z != 0 ) { t.terrain = T_OPEN_AIR; t.lit = Lit::BLANK; t.seen = 0; }
        if( z < 0 ) { t.sky_exposed = false; } // underground is never outside (game rule)
        return t;
    }
    void set( int x, int y, int z, const TileVal &t ) { tiles[Key{ x, y, z }] = t; }
    TileVal &mut( int x, int y, int z ) {
        auto it = tiles.find( Key{ x, y, z } );
        if( it == tiles.end() ) {
            TileVal t = query( x, y, z );
            it = tiles.emplace( Key{ x, y, z }, t ).first;
        }
        return it->second;
    }
};

World make_shelter();
World make_house();
World make_road_field();
World make_multistorey();
World make_vehicle_road( float face_deg );

// Scripted environment sequence (Phase 10, gate 10.1b): noon-clear ->
// storm -> midnight. Step 0 matches the proto crate's canonical
// sample environment (crates/proto/src/samples.rs sample_environment).
std::vector<EnvVal> env_sequence();

class MockAdapter : public IGameAdapter
{
    public:
        explicit MockAdapter( World &w ) : w_( w ) {}
        TileVal query_tile( int x, int y, int z ) override { return w_.query( x, y, z ); }
        void prepare_visibility( int ) override {}
        bool in_bubble( int x, int y, int z ) const override {
            return x >= w_.minx && x <= w_.maxx && y >= w_.miny && y <= w_.maxy &&
                   z >= kZMin && z <= kZMax;
        }
        std::vector<DefVal> tile_defs() override;
        std::vector<std::string> connect_group_table() override {
            return { "wall", "door", "fence" };
        }
        std::vector<EntVal> entities() override { return w_.ents; }
        std::vector<VehVal> vehicles() override { return w_.vehs; }
        std::vector<MemVal> memory_tiles() override { return w_.memory; }
        EnvVal environment() override { return w_.env; }
        std::string game_version() const override { return "mock-1"; }
        std::string tileset_id() const override { return "UltiCa"; }
    private:
        World &w_;
};

} // namespace mock
