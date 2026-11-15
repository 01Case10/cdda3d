// CDDA Voxel Frontend — Phase 3: adapter interface between the exporter
// core and a game-state source.
//
// The fork implements this in voxel_real_adapter.cpp against the REAL
// CDDA APIs verified in docs/hook-points.md / docs/recon.md §4:
//   map::ter(const tripoint_bub_ms&)            map.h:1045
//   map::furn(tripoint_bub_ms) BY VALUE         map.h:1020
//   map::tr_at(const tripoint_bub_ms&)          map.h:1614
//   map::field_at(const tripoint_bub_ms&)       map.h:1660 (full field map)
//   map::i_at(const tripoint_bub_ms&)           map.h:1424
//   submap maptile::get_uppermost_item()        submap.h:439
//   map::has_graffiti_at(const tripoint_bub_ms&) map.h:1811
//   map::update_visibility_cache(int)           map.h:2520
//   map::apparent_light_at(p, cache)            map.h:585
//   Character/monster::get_overlay_ids()        character.h:2553 / monster.h:630
//   vehicle::bub_part_pos(map, index)           vehicle.h:1555
//   vehicle::face.dir()                         vehicle.h:2560 (tileray)
// The harness implements the same interface over a mock world, so the
// serializer/diff logic under test is byte-for-byte the fork's logic.
#pragma once

#include <string>
#include <vector>

#include "voxel_types.h"

namespace voxel_ipc
{

class IGameAdapter
{
    public:
        virtual ~IGameAdapter() = default;

        // --- tile queries (one tile, absolute bubble coordinates) ---
        virtual TileVal query_tile( int x, int y, int z ) = 0;
        // Refresh the visibility cache for one z-level before lit/seen are
        // read (real adapter: map::update_visibility_cache(z)).
        virtual void prepare_visibility( int z ) = 0;
        // True when (x,y,z) is inside the loaded reality bubble.
        virtual bool in_bubble( int x, int y, int z ) const = 0;

        // --- registries (TileDefs) ---
        virtual std::vector<DefVal> tile_defs() = 0;
        virtual std::vector<std::string> connect_group_table() = 0;

        // --- creatures, from creature tracking — NOT from submaps ---
        virtual std::vector<EntVal> entities() = 0;
        // --- vehicles, absolute part positions computed game-side ---
        virtual std::vector<VehVal> vehicles() = 0;
        // --- avatar map memory: terrain/furniture only ---
        virtual std::vector<MemVal> memory_tiles() = 0;

        virtual std::string game_version() const = 0;
        virtual std::string tileset_id() const = 0;
};

} // namespace voxel_ipc
