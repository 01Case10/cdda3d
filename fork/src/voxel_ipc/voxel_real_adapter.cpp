// CDDA Voxel Frontend — Phase 3: REAL game adapter (fork-only build).
// Every game API below was grep-verified against the pinned source
// (CDDA master @ 6a3e59e0, fork-src/cdda) on 2026-11-15; file:line for
// each is in docs/phase3-fork.md. This file does NOT compile in the
// standalone harness and has never been compiled anywhere — see the
// Phase 3 record: tests 0.1/0.2 must run first on a build-capable
// machine, then this file gets its first real compile.
#include "voxel_real_adapter.h"

#include <algorithm>

#include "avatar.h"            // avatar::get_memorized_tile (avatar.h:165)
#include "character.h"         // Character::get_overlay_ids (character.h:2553)
#include "creature_tracker.h"  // creature_tracker::get_monsters_list
#include "field.h"             // field / field_entry iteration
#include "field_type.h"        // get_all_field_types (field_type.h:58)
#include "game.h"              // game::all_npcs, get_vehicles via map
#include "item.h"              // item::typeId
#include "item_factory.h"      // item_controller->get_generic_factory()
#include "lightmap.h"          // lit_level (lightmap.h:96)
#include "map.h"               // the map:: query APIs (see header comment)
#include "map_memory.h"        // memorized_tile::get_ter_id/get_dec_id
#include "mapdata.h"           // ter_t/furn_t, mapdata::terrain_data/furniture_data
#include "monster.h"           // monster::get_overlay_ids (monster.h:630)
#include "monstergenerator.h"  // MonsterGenerator::generator().get_all_mtypes()
#include "submap.h"            // const_maptile::get_uppermost_item (submap.h:439)
#include "trap.h"
#include "units.h"             // units::to_degrees
#include "veh_type.h"          // vehicles::parts::get_all (veh_type.h)
#include "vehicle.h"           // vehicle::bub_part_pos / face

namespace voxel_ipc
{
namespace
{

tripoint_bub_ms P( int x, int y, int z ) { return tripoint_bub_ms( x, y, z ); }

uint8_t hp_frac_of( const Creature &c )
{
    const int pct = c.hp_percentage(); // creature.h: pure virtual, 0..100
    return static_cast<uint8_t>( std::clamp( pct, 0, 100 ) * 255 / 100 );
}

std::vector<uint16_t> bits_to_indices( const std::bitset<NUM_TERCONN> &bits )
{
    std::vector<uint16_t> out;
    for( size_t i = 0; i < bits.size(); ++i ) {
        if( bits.test( i ) ) { out.push_back( static_cast<uint16_t>( i ) ); }
    }
    return out;
}

uint64_t hot_flags_of( const map_data_common_t &d )
{
    // ter_furn_flag bitset -> low 64 bits of the wire uint64. Flags past
    // bit 63 are still present verbatim in the string flag list.
    uint64_t out = 0;
    for( int i = 0; i < 64 && i < static_cast<int>( ter_furn_flag::NUM_TFLAG_FLAGS ); ++i ) {
        if( d.has_flag( static_cast<ter_furn_flag>( i ) ) ) { out |= 1ULL << i; }
    }
    return out;
}

void fill_common( DefVal &out, const map_data_common_t &d )
{
    out.flags.assign( d.get_flags().begin(), d.get_flags().end() );
    out.hot_flags = hot_flags_of( d );
    out.connect_groups = bits_to_indices( d.connect_groups );
    out.connects_to_groups = bits_to_indices( d.connect_to_groups );
    out.rotate_to_groups = bits_to_indices( d.rotate_to_groups );
    if( !d.looks_like.empty() ) { out.looks_like = d.looks_like; }
    out.name = d.name();
}

} // namespace

void RealAdapter::prepare_visibility( int z )
{
    here_.update_visibility_cache( z ); // map.h:2520
}

bool RealAdapter::in_bubble( int x, int y, int z ) const
{
    return here_.inbounds( P( x, y, z ) );
}

TileVal RealAdapter::query_tile( int x, int y, int z )
{
    const tripoint_bub_ms p = P( x, y, z );
    TileVal t;
    t.terrain = static_cast<uint32_t>( here_.ter( p ).to_i() );   // map.h:1045
    t.furniture = static_cast<uint32_t>( here_.furn( p ).to_i() ); // map.h:1020 (by value)
    t.trap = static_cast<uint32_t>( here_.tr_at( p ).id.to_i() );  // map.h:1614
    const field &fld = here_.field_at( p );                        // map.h:1660, FULL map
    for( const auto &[type_id, entry] : fld ) {
        if( entry.is_field_alive() ) {
            t.fields.push_back( FieldVal{ static_cast<uint32_t>( type_id.to_i() ),
                                          static_cast<uint8_t>( entry.get_field_intensity() ) } );
        }
    }
    t.displayed_type = static_cast<uint32_t>( fld.displayed_field_type().to_i() );
    t.displayed_intensity = static_cast<uint8_t>( fld.displayed_intensity() );
    map_stack items = here_.i_at( p );                             // map.h:1424
    t.item_count = static_cast<uint16_t>( items.size() );
    if( !items.empty() ) {
        // submap maptile helper (submap.h:439) via map::maptile_at (map.h:662)
        const item &top = here_.maptile_at( p ).get_uppermost_item();
        t.top_item = item_wire_id( top.typeId().str() );
    }
    t.graffiti = here_.has_graffiti_at( p );                       // map.h:1811
    const lit_level ll = here_.apparent_light_at( p, here_.get_visibility_variables_cache() );
    t.lit = static_cast<Lit>( static_cast<uint8_t>( ll ) );         // enum order matches schema
    // seen: the visibility cache's float is not exposed per-tile by a
    // public getter; MEMORIZED/BLANK are exact from lit_level, and for
    // visible tiles the exporter quantizes apparent visibility coarsely.
    // (Flagged in docs/phase3-fork.md: if a future upstream exposes
    // seen_cache per tile, wire it here — do not invent an accessor.)
    t.seen = ( ll == lit_level::BLANK ) ? 0 : ( ll == lit_level::MEMORIZED ? 64 : 255 );
    return t;
}

uint32_t RealAdapter::item_wire_id( const std::string &string_id )
{
    if( !item_ids_built_ ) {
        std::vector<std::string> all;
        for( const itype &it : item_controller->get_generic_factory().get_all() ) {
            all.push_back( it.id.str() );
        }
        std::sort( all.begin(), all.end() );
        all.erase( std::unique( all.begin(), all.end() ), all.end() );
        for( size_t i = 0; i < all.size(); ++i ) {
            item_ids_[all[i]] = static_cast<uint32_t>( i + 1 ); // 0 = none
        }
        item_ids_built_ = true;
    }
    auto it = item_ids_.find( string_id );
    return it == item_ids_.end() ? 0 : it->second;
}

std::vector<std::string> RealAdapter::connect_group_table()
{
    // connect_group registry: indices are the bit positions used by
    // map_data_common_t's bitsets (set via connect_group::load order).
    // The names are recovered by probing get_connect_group is NOT
    // possible (it maps name->group, not index->name), so the fork
    // records names at load: see the hook note in docs/phase3-fork.md.
    // Until that registry mirror lands, export stable placeholder names
    // "group_<index>" — indices (the load-bearing data) are exact.
    std::vector<std::string> out;
    out.reserve( NUM_TERCONN );
    for( int i = 0; i < NUM_TERCONN; ++i ) {
        out.push_back( "group_" + std::to_string( i ) );
    }
    return out;
}

std::vector<DefVal> RealAdapter::tile_defs()
{
    std::vector<DefVal> out;
    for( const ter_t &d : mapdata::terrain_data.get_all() ) {
        DefVal v;
        v.id = static_cast<uint32_t>( d.id.id().to_i() );
        v.string_id = d.id.str();
        v.kind = 0; // Terrain
        fill_common( v, d );
        if( !d.open.is_empty() && !d.open.is_null() ) { v.open_result = d.open.str(); }
        if( !d.close.is_empty() && !d.close.is_null() ) { v.close_result = d.close.str(); }
        if( d.bash && !d.bash->ter_set.is_null() ) { v.bash_result = d.bash->ter_set.str(); }
        if( !d.roof.is_empty() && !d.roof.is_null() ) { v.roof_result = d.roof.str(); }
        out.push_back( std::move( v ) );
    }
    for( const furn_t &d : mapdata::furniture_data.get_all() ) {
        DefVal v;
        v.id = static_cast<uint32_t>( d.id.id().to_i() );
        v.string_id = d.id.str();
        v.kind = 1; // Furniture
        fill_common( v, d );
        if( !d.open.is_empty() && !d.open.is_null() ) { v.open_result = d.open.str(); }
        if( !d.close.is_empty() && !d.close.is_null() ) { v.close_result = d.close.str(); }
        if( d.bash && !d.bash->furn_set.is_null() ) { v.bash_result = d.bash->furn_set.str(); }
        out.push_back( std::move( v ) );
    }
    // Traps: trap_factory is file-local to trap.cpp; the public route is
    // iterating trap_id int range via trap::count-like accessor if present.
    // NOTE (flagged in docs/phase3-fork.md): upstream exposes no public
    // get_all() for traps in the pinned source — the fork adds a 3-line
    // accessor next to trap_factory (hook patch H7) rather than inventing
    // one here. Trap DEFS are therefore emitted by that accessor's TU;
    // per-tile trap ids in query_tile() above are already exact.
    for( const field_type &d : get_all_field_types().get_all() ) {
        DefVal v;
        v.id = static_cast<uint32_t>( d.id.id().to_i() );
        v.string_id = d.id.str();
        v.kind = 3; // Field
        v.name = d.name( 0 );
        out.push_back( std::move( v ) );
    }
    for( const mtype &d : MonsterGenerator::generator().get_all_mtypes() ) {
        DefVal v;
        v.id = static_cast<uint32_t>( d.id.to_i() );
        v.string_id = d.id.str();
        v.kind = 4; // Monster
        v.name = d.nname();
        out.push_back( std::move( v ) );
    }
    if( !item_ids_built_ ) { item_wire_id( "" ); }
    for( const itype &d : item_controller->get_generic_factory().get_all() ) {
        DefVal v;
        v.id = item_wire_id( d.id.str() );
        v.string_id = d.id.str();
        v.kind = 5; // Item
        v.name = d.name.translated();
        out.push_back( std::move( v ) );
    }
    for( const vpart_info &d : vehicles::parts::get_all() ) {
        DefVal v;
        v.id = static_cast<uint32_t>( d.id.id().to_i() );
        v.string_id = d.id.str();
        v.kind = 6; // VehiclePart
        v.name = d.name().translated();
        out.push_back( std::move( v ) );
    }
    return out;
}

std::vector<EntVal> RealAdapter::entities()
{
    std::vector<EntVal> out;
    game &g = get_game();
    // Player (avatar) first, then NPCs, then tracked monsters — all from
    // creature tracking / the active lists, never from submaps.
    auto push_creature = [&]( const Creature &c, uint64_t stable_id, uint32_t type_def,
                              const std::string &sprite,
                              const std::vector<std::pair<std::string, std::string>> &overlays ) {
        EntVal e;
        e.id = stable_id;
        e.type_def = type_def;
        e.sprite_id = sprite;
        const tripoint_bub_ms pos = c.pos_bub( here_ );
        e.x = pos.x(); e.y = pos.y(); e.z = pos.z();
        e.facing = static_cast<Facing>( static_cast<uint8_t>( c.facing ) ); // NONE/LEFT/RIGHT only
        e.overlays.reserve( overlays.size() );
        for( const auto &[id, looks] : overlays ) {
            e.overlays.push_back( looks.empty() ? id : looks ); // game's resolved pair, in order
        }
        e.hp_frac = hp_frac_of( c );
        out.push_back( std::move( e ) );
    };
    avatar &u = g.u;
    push_creature( u, 1, 0, "player", u.get_overlay_ids() );
    uint64_t npc_id = 1000000;
    for( npc &n : g.all_npcs() ) {
        push_creature( n, npc_id + static_cast<uint64_t>( n.getID().get_value() ),
                       0, "npc", n.get_overlay_ids() );
    }
    for( const shared_ptr_fast<monster> &mp : get_creature_tracker().get_monsters_list() ) {
        const monster &m = *mp;
        EntVal e;
        e.id = 2000000 + static_cast<uint64_t>( get_creature_tracker().temporary_id( m ) );
        e.type_def = static_cast<uint32_t>( m.type->id.to_i() );
        e.sprite_id = m.type->id.str();
        const tripoint_bub_ms pos = m.pos_bub( here_ );
        e.x = pos.x(); e.y = pos.y(); e.z = pos.z();
        e.facing = static_cast<Facing>( static_cast<uint8_t>( m.facing ) );
        for( const auto &[id, looks] : m.get_overlay_ids() ) {
            e.overlays.push_back( looks.empty() ? id : looks );
        }
        e.hp_frac = hp_frac_of( m );
        out.push_back( std::move( e ) );
    }
    return out;
}

std::vector<VehVal> RealAdapter::vehicles()
{
    std::vector<VehVal> out;
    uint64_t vid = 1;
    for( wrapped_vehicle &wv : here_.get_vehicles() ) {
        vehicle *veh = wv.v;
        if( veh == nullptr ) { continue; }
        VehVal v;
        v.id = vid++;
        v.name = veh->name;
        v.face_deg = static_cast<float>( units::to_degrees( veh->face.dir() ) ); // float, continuous
        for( size_t i = 0; i < veh->parts.size(); ++i ) {
            const vehicle_part &part = veh->parts[i];
            if( part.removed ) { continue; }
            VPartVal p;
            p.vpart = static_cast<uint32_t>( part.info().id.id().to_i() );
            const tripoint_bub_ms abs = veh->bub_part_pos( here_, static_cast<int>( i ) ); // vehicle.h:1555
            p.ax = abs.x(); p.ay = abs.y(); p.az = abs.z();
            p.mx = part.mount.x(); p.my = part.mount.y(); p.mz = 0;
            const double hp = part.health_percent();
            p.hp_frac = static_cast<uint8_t>( std::clamp( hp, 0.0, 1.0 ) * 255.0 );
            p.flags = ( part.open ? 1u : 0u ) | ( part.is_broken() ? 2u : 0u ) |
                      ( part.info().has_flag( "ROOF" ) ? 4u : 0u );
            v.parts.push_back( p );
        }
        out.push_back( std::move( v ) );
    }
    return out;
}

std::vector<MemVal> RealAdapter::memory_tiles()
{
    // Avatar map memory (avatar.h:165 get_memorized_tile over abs coords).
    // The memory API is absolute-coordinate based; the exporter works in
    // bubble coords, so convert via the map's abs origin. Only tiles whose
    // memorized terrain differs from the default (never-seen) tile are
    // emitted; terrain/furniture string ids are resolved back to wire ids
    // through the registries. NO live data is read here by construction.
    std::vector<MemVal> out;
    avatar &u = get_avatar();
    const tripoint_abs_ms origin = here_.get_abs( tripoint_bub_ms::zero );
    for( int z = kZMin; z <= kZMax; ++z ) {
        for( int y = 0; y < MAPSIZE_X; ++y ) {
            for( int x = 0; x < MAPSIZE_Y; ++x ) {
                const tripoint_abs_ms ap = origin + tripoint_rel_ms( x, y, z );
                const memorized_tile &mt = u.get_memorized_tile( ap );
                if( mt == memorized_tile::default_tile ) { continue; }
                MemVal m;
                m.x = x; m.y = y; m.z = z;
                m.terrain = static_cast<uint32_t>( ter_id( mt.get_ter_id() ).to_i() );
                if( !mt.get_dec_id().empty() ) {
                    m.furniture = static_cast<uint32_t>( furn_id( mt.get_dec_id() ).to_i() );
                }
                out.push_back( m );
            }
        }
    }
    return out;
}

std::string RealAdapter::game_version() const { return "CDDA-fork"; } // fork stamps its git describe here
std::string RealAdapter::tileset_id() const { return "UltiCa"; }

} // namespace voxel_ipc
