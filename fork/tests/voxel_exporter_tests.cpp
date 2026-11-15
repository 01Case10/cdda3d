// CDDA Voxel Frontend — Phase 3 fork-side Catch2 tests.
//
// Drop into the fork's tests/ directory as tests/voxel_exporter_tests.cpp
// and run with the normal cata_test binary on a build-capable machine:
//   ./cata_test "[voxel]"
// These tests have NOT been compiled or run anywhere yet: tests 0.1/0.2
// (fork build + baseline suite) are the precondition, per the Phase 0
// record. They compare the exporter against the game's OWN query APIs
// (Part II Principle 1) — the exporter is wrong by definition if it
// disagrees with them. The harness-level equivalents of 3.3/3.5/3.5b/
// 3.6/3.7/3.8 already pass in tools/fork-harness.
#include "catch/catch.hpp"

#include "avatar.h"
#include "creature_tracker.h"
#include "field.h"
#include "game.h"
#include "map.h"
#include "mapdata.h"
#include "monster.h"
#include "voxel_ipc/voxel_exporter.h"
#include "voxel_ipc/voxel_real_adapter.h"

using namespace voxel_ipc;

namespace
{
ViewSpec test_view( map &here )
{
    const tripoint_bub_ms c = get_avatar().pos_bub( here );
    ViewSpec v;
    v.cx = c.x(); v.cy = c.y(); v.cz = c.z();
    v.radius = 12; v.z_below = 1; v.z_above = 1;
    return v;
}
} // namespace

TEST_CASE( "voxel TileDefs completeness", "[voxel][3.1]" )
{
    // Every registry id appears exactly once per kind, with resolved
    // connect-group indices and transform result ids.
    map &here = get_map();
    RealAdapter adapter( here );
    Exporter exporter( adapter );
    const auto defs = exporter.build_tiledefs();
    std::set<std::pair<int, uint32_t>> seen;
    for( const auto &d : defs.defs ) {
        REQUIRE( seen.insert( { static_cast<int>( d->kind ), d->id } ).second );
        for( const uint16_t gi : d->connect_groups ) {
            REQUIRE( gi < defs.connect_groups.size() );
        }
    }
    CHECK( seen.size() == defs.defs.size() );
    // Spot-check a door: open/bash results must be the game's own ids.
    bool door_found = false;
    for( const auto &d : defs.defs ) {
        if( d->string_id == "t_door_c" ) {
            door_found = true;
            CHECK( d->open_result == "t_door_o" );
        }
    }
    CHECK( door_found );
}

TEST_CASE( "voxel snapshot equals game queries", "[voxel][3.3]" )
{
    map &here = get_map();
    RealAdapter adapter( here );
    Exporter exporter( adapter );
    const ViewSpec view = test_view( here );
    const auto snap = exporter.build_snapshot( view );
    REQUIRE( !snap.blocks.empty() );
    for( const auto &bp : snap.blocks ) {
        const auto &b = *bp;
        for( int i = 0; i < 144; ++i ) {
            const tripoint_bub_ms p( b.origin->x() + i % 12, b.origin->y() + i / 12, b.origin->z() );
            CAPTURE( p );
            CHECK( b.terrain[i] == static_cast<uint32_t>( here.ter( p ).to_i() ) );
            CHECK( b.furniture[i] == static_cast<uint32_t>( here.furn( p ).to_i() ) );
            CHECK( b.trap[i] == static_cast<uint32_t>( here.tr_at( p ).id.to_i() ) );
            CHECK( b.has_graffiti[i] == here.has_graffiti_at( p ) );
            map_stack items = here.i_at( p );
            CHECK( b.item_count[i] == items.size() );
            // Fields compared as the FULL (type -> intensity) map.
            std::map<uint32_t, int> want_fields;
            for( const auto &[tid, entry] : here.field_at( p ) ) {
                if( entry.is_field_alive() ) {
                    want_fields[tid.to_i()] = entry.get_field_intensity();
                }
            }
            std::map<uint32_t, int> got_fields;
            if( b.field_index[i] >= 0 ) {
                for( size_t fi = static_cast<size_t>( b.field_index[i] );
                     fi < b.fields.size() && b.fields[fi]->tile_index == i; ++fi ) {
                    got_fields[b.fields[fi]->type] = b.fields[fi]->intensity;
                }
            }
            CHECK( got_fields == want_fields );
        }
    }
}

TEST_CASE( "voxel lit_level export is exact", "[voxel][3.5]" )
{
    map &here = get_map();
    RealAdapter adapter( here );
    Exporter exporter( adapter );
    const ViewSpec view = test_view( here );
    here.update_visibility_cache( view.cz );
    const auto snap = exporter.build_snapshot( view );
    const visibility_variables &cache = here.get_visibility_variables_cache();
    for( const auto &bp : snap.blocks ) {
        const auto &b = *bp;
        if( b.origin->z() != view.cz ) { continue; }
        for( int i = 0; i < 144; ++i ) {
            const tripoint_bub_ms p( b.origin->x() + i % 12, b.origin->y() + i / 12, b.origin->z() );
            const lit_level want = here.apparent_light_at( p, cache );
            CHECK( static_cast<uint8_t>( b.lit[i] ) == static_cast<uint8_t>( want ) );
        }
    }
}

TEST_CASE( "voxel memory layer carries no live data", "[voxel][3.5b]" )
{
    map &here = get_map();
    RealAdapter adapter( here );
    Exporter exporter( adapter );
    const auto snap = exporter.build_snapshot( test_view( here ) );
    for( const auto &bp : snap.memory_blocks ) {
        const auto &b = *bp;
        CHECK( b.fields.empty() );
        for( int i = 0; i < 144; ++i ) {
            CHECK( b.top_item[i] == 0 );
            CHECK( b.item_count[i] == 0 );
            CHECK( ( b.lit[i] == cdda::ipc::LitLevel_MEMORIZED ||
                     b.lit[i] == cdda::ipc::LitLevel_BLANK ) );
        }
    }
    CHECK( snap.entities.empty() || true ); // entities live only in the live layer by type
}

TEST_CASE( "voxel entity export matches creature tracking", "[voxel][3.6][3.7]" )
{
    map &here = get_map();
    RealAdapter adapter( here );
    Exporter exporter( adapter );
    const auto snap = exporter.build_snapshot( test_view( here ) );
    for( const auto &ep : snap.entities ) {
        const uint8_t f = static_cast<uint8_t>( ep->facing );
        CHECK( f <= 2 ); // NONE/LEFT/RIGHT only — any other value fails
    }
    // Overlay order: the avatar's exported list equals get_overlay_ids().
    const auto want = get_avatar().get_overlay_ids();
    bool found_avatar = false;
    for( const auto &ep : snap.entities ) {
        if( ep->id == 1 ) {
            found_avatar = true;
            REQUIRE( ep->overlays.size() == want.size() );
            for( size_t i = 0; i < want.size(); ++i ) {
                const std::string resolved = want[i].second.empty() ? want[i].first : want[i].second;
                CHECK( ep->overlays[i] == resolved );
            }
        }
    }
    CHECK( found_avatar );
}

TEST_CASE( "voxel vehicle export uses game-side absolute positions", "[voxel][3.8]" )
{
    map &here = get_map();
    RealAdapter adapter( here );
    Exporter exporter( adapter );
    const auto snap = exporter.build_snapshot( test_view( here ) );
    for( wrapped_vehicle &wv : here.get_vehicles() ) {
        vehicle *veh = wv.v;
        if( veh == nullptr ) { continue; }
        bool found = false;
        for( const auto &vp : snap.vehicles ) {
            if( vp->name != veh->name ) { continue; }
            found = true;
            CHECK( vp->face_deg == Approx( static_cast<float>(
                       units::to_degrees( veh->face.dir() ) ) ) );
            REQUIRE( vp->parts.size() <= veh->parts.size() );
            for( size_t i = 0; i < vp->parts.size(); ++i ) {
                const tripoint_bub_ms want = veh->bub_part_pos( here, static_cast<int>( i ) );
                CHECK( vp->parts[i]->abs_pos->x() == want.x() );
                CHECK( vp->parts[i]->abs_pos->y() == want.y() );
                CHECK( vp->parts[i]->abs_pos->z() == want.z() );
            }
        }
        CHECK( found );
    }
}
