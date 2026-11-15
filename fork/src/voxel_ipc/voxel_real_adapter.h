// CDDA Voxel Frontend — Phase 3: REAL game adapter (fork-only).
//
// This translation unit compiles ONLY inside the CDDA fork build: it
// includes the game's own headers and reads state through the query APIs
// verified in docs/hook-points.md. It is deliberately NOT part of the
// standalone harness build (tools/fork-harness uses a mock adapter over
// the same IGameAdapter interface).
//
// Wire-id rule: TileDef.id is the registry int_id value, unique per
// TileKind (the protocol keeps terrain/furniture/trap/field/item arrays
// separate, exactly like the game does). Items have string ids upstream,
// so item wire ids are assigned in sorted string-id order at first use
// and kept in a map shared by tile_defs() and query_tile().
#pragma once

#include <map>
#include <string>

#include "voxel_adapter.h"
#include "voxel_refs.h"

class map;

namespace voxel_ipc
{

class RealAdapter : public IGameAdapter
{
    public:
        explicit RealAdapter( map &here ) : here_( here ) {}

        TileVal query_tile( int x, int y, int z ) override;
        void prepare_visibility( int z ) override;
        bool in_bubble( int x, int y, int z ) const override;
        std::vector<DefVal> tile_defs() override;
        std::vector<std::string> connect_group_table() override;
        std::vector<EntVal> entities() override;
        std::vector<VehVal> vehicles() override;
        std::vector<MemVal> memory_tiles() override;
        std::string game_version() const override;
        std::string tileset_id() const override;

    private:
        uint32_t item_wire_id( const std::string &string_id );
        // The composed-config fg/bg table (lazy-loaded once from
        // the CDDA_VOXEL_FG_REFS env path; empty when unset).
        const FgBgRefs &fg_bg();
        map &here_;
        std::map<std::string, uint32_t> item_ids_;
        bool item_ids_built_ = false;
        FgBgRefs fg_bg_;
        bool fg_bg_loaded_ = false;
};

} // namespace voxel_ipc
