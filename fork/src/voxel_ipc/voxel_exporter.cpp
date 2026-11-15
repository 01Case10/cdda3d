// CDDA Voxel Frontend — Phase 3 exporter core. See voxel_exporter.h.
#include "voxel_exporter.h"

#include <algorithm>
#include <chrono>
#include <cstring>

namespace voxel_ipc
{
namespace
{
using namespace cdda::ipc;

int floor_div( int a, int b ) {
    int q = a / b;
    if( ( a % b != 0 ) && ( ( a < 0 ) != ( b < 0 ) ) ) {
        --q;
    }
    return q;
}

void fnv( uint64_t &h, uint64_t v ) {
    for( int i = 0; i < 8; ++i ) {
        h ^= ( v >> ( i * 8 ) ) & 0xffu;
        h *= 1099511628211ULL;
    }
}
void fnv_str( uint64_t &h, const std::string &s ) {
    fnv( h, s.size() );
    for( unsigned char c : s ) {
        h ^= c;
        h *= 1099511628211ULL;
    }
}

Vec3i v3( int x, int y, int z ) { return Vec3i( x, y, z ); }

} // namespace

HelloT Exporter::build_hello() const
{
    HelloT h;
    h.protocol_version = kProtocolVersion;
    h.game_version = adapter_.game_version();
    h.tileset_id = adapter_.tileset_id();
    h.tile_pixel_size = 32;
    h.z_min = kZMin;
    h.z_max = kZMax;
    return h;
}

TileDefsT Exporter::build_tiledefs()
{
    TileDefsT out;
    out.protocol_version = kProtocolVersion;
    out.connect_groups = adapter_.connect_group_table();
    for( auto &d : adapter_.tile_defs() ) {
        auto w = std::make_unique<TileDefT>();
        w->id = d.id;
        w->string_id = d.string_id;
        w->name = d.name;
        w->kind = static_cast<TileKind>( d.kind );
        w->flags = d.flags;
        w->hot_flags = d.hot_flags;
        w->connect_groups = d.connect_groups;
        w->connects_to_groups = d.connects_to_groups;
        w->rotate_to_groups = d.rotate_to_groups;
        if( d.open_result ) { w->open_result = *d.open_result; }
        if( d.close_result ) { w->close_result = *d.close_result; }
        if( d.bash_result ) { w->bash_result = *d.bash_result; }
        if( d.roof_result ) { w->roof_result = *d.roof_result; }
        if( d.looks_like ) { w->looks_like = *d.looks_like; }
        w->fg_ref = d.fg_ref;
        w->bg_ref = d.bg_ref;
        out.defs.push_back( std::move( w ) );
    }
    return out;
}

std::array<TileVal, kTilesPerBlock> Exporter::read_block( const BlockKey &k )
{
    std::array<TileVal, kTilesPerBlock> tiles;
    adapter_.prepare_visibility( k.z );
    for( int dy = 0; dy < kSubmapSize; ++dy ) {
        for( int dx = 0; dx < kSubmapSize; ++dx ) {
            const int x = k.sx * kSubmapSize + dx;
            const int y = k.sy * kSubmapSize + dy;
            TileVal t;
            if( adapter_.in_bubble( x, y, k.z ) ) {
                t = adapter_.query_tile( x, y, k.z );
                std::sort( t.fields.begin(), t.fields.end(),
                []( const FieldVal &a, const FieldVal &b ) { return a.type < b.type; } );
            } else {
                t.lit = Lit::BLANK; // unexplored / outside bubble: never fabricated
            }
            tiles[dy * kSubmapSize + dx] = std::move( t );
        }
    }
    return tiles;
}

SubmapBlockT Exporter::block_to_wire( const BlockKey &k,
                                      const std::array<TileVal, kTilesPerBlock> &tiles ) const
{
    SubmapBlockT b;
    b.origin = std::make_unique<Vec3i>( v3( k.sx * kSubmapSize, k.sy * kSubmapSize, k.z ) );
    b.terrain.resize( kTilesPerBlock );
    b.furniture.resize( kTilesPerBlock );
    b.trap.resize( kTilesPerBlock );
    b.field_index.assign( kTilesPerBlock, -1 );
    b.displayed_field.assign( kTilesPerBlock, DisplayedField( 0, 0 ) );
    b.top_item.resize( kTilesPerBlock );
    b.item_count.resize( kTilesPerBlock );
    b.lit.resize( kTilesPerBlock );
    b.seen.resize( kTilesPerBlock );
    b.has_graffiti.resize( kTilesPerBlock );
    for( int i = 0; i < kTilesPerBlock; ++i ) {
        const TileVal &t = tiles[i];
        b.terrain[i] = t.terrain;
        b.furniture[i] = t.furniture;
        b.trap[i] = t.trap;
        if( !t.fields.empty() ) {
            b.field_index[i] = static_cast<int32_t>( b.fields.size() );
            for( const FieldVal &f : t.fields ) {
                auto fo = std::make_unique<FieldOnTileT>();
                fo->tile_index = static_cast<uint16_t>( i );
                fo->type = f.type;
                fo->intensity = f.intensity;
                b.fields.push_back( std::move( fo ) );
            }
        }
        b.displayed_field[i] = DisplayedField( t.displayed_type, t.displayed_intensity );
        b.top_item[i] = t.top_item;
        b.item_count[i] = t.item_count;
        b.lit[i] = static_cast<LitLevel>( static_cast<uint8_t>( t.lit ) );
        b.seen[i] = t.seen;
        b.has_graffiti[i] = t.graffiti;
    }
    return b;
}

std::vector<SubmapBlockT> Exporter::build_memory_blocks() const
{
    // Memory layer contract (test 3.5b): terrain/furniture ONLY, lit =
    // MEMORIZED, and NO live fields/items/entities — even if the real tile
    // has since gained them. Enforced here by construction: MemVal carries
    // nothing else, and every other array is zeroed.
    std::map<BlockKey, std::array<TileVal, kTilesPerBlock>> grouped;
    for( const MemVal &m : adapter_.memory_tiles() ) {
        BlockKey k{ floor_div( m.x, kSubmapSize ), floor_div( m.y, kSubmapSize ), m.z };
        auto &arr = grouped[k];
        TileVal t;
        t.terrain = m.terrain;
        t.furniture = m.furniture;
        t.lit = Lit::MEMORIZED;
        const int dx = m.x - k.sx * kSubmapSize;
        const int dy = m.y - k.sy * kSubmapSize;
        arr[dy * kSubmapSize + dx] = t;
    }
    std::vector<SubmapBlockT> out;
    for( auto &[k, arr] : grouped ) {
        // Tiles in a memory block that were never memorized stay BLANK.
        for( auto &t : arr ) {
            if( t.lit != Lit::MEMORIZED ) { t.lit = Lit::BLANK; }
        }
        out.push_back( block_to_wire( k, arr ) );
    }
    return out;
}

EntityT Exporter::ent_to_wire( const EntVal &e )
{
    EntityT w;
    w.id = e.id;
    w.type_def = e.type_def;
    w.sprite_id = e.sprite_id;
    w.pos = std::make_unique<Vec3i>( v3( e.x, e.y, e.z ) );
    w.facing = static_cast<cdda::ipc::Facing>( static_cast<uint8_t>( e.facing ) );
    w.overlays = e.overlays; // game order, verbatim
    w.hp_frac = e.hp_frac;
    w.flags = e.flags;
    return w;
}

VehicleT Exporter::veh_to_wire( const VehVal &v )
{
    VehicleT w;
    w.id = v.id;
    w.name = v.name;
    w.face_deg = v.face_deg; // float degrees, never quantized
    for( const VPartVal &p : v.parts ) {
        auto wp = std::make_unique<VehiclePartStateT>();
        wp->vpart = p.vpart;
        wp->abs_pos = std::make_unique<Vec3i>( v3( p.ax, p.ay, p.az ) );
        wp->mount = std::make_unique<Vec3i>( v3( p.mx, p.my, p.mz ) );
        wp->hp_frac = p.hp_frac;
        wp->flags = p.flags;
        w.parts.push_back( std::move( wp ) );
    }
    return w;
}

SnapshotT Exporter::build_snapshot( const ViewSpec &view )
{
    const auto t0 = std::chrono::steady_clock::now();
    last_view_ = view;
    ++epoch_;               // every fresh Snapshot is a new epoch (protocol.md)
    force_resync_ = false;
    cache_.clear();
    ent_cache_.clear();
    veh_cache_.clear();

    SnapshotT snap;
    snap.view_origin = std::make_unique<Vec3i>( v3( view.cx, view.cy, view.cz ) );
    snap.view_radius = static_cast<uint32_t>( std::max( 0, std::min( view.radius, kMaxViewDistance ) ) );

    const int zlo = std::max( kZMin, view.cz - view.z_below );
    const int zhi = std::min( kZMax, view.cz + view.z_above );
    const int r = static_cast<int>( snap.view_radius );
    const int sx0 = floor_div( view.cx - r, kSubmapSize );
    const int sx1 = floor_div( view.cx + r, kSubmapSize );
    const int sy0 = floor_div( view.cy - r, kSubmapSize );
    const int sy1 = floor_div( view.cy + r, kSubmapSize );
    for( int z = zlo; z <= zhi; ++z ) {
        for( int sy = sy0; sy <= sy1; ++sy ) {
            for( int sx = sx0; sx <= sx1; ++sx ) {
                BlockKey k{ sx, sy, z };
                auto tiles = read_block( k );
                snap.blocks.push_back( std::make_unique<SubmapBlockT>( block_to_wire( k, tiles ) ) );
                cache_[k] = std::move( tiles );
            }
        }
    }
    snap.memory_blocks.clear();
    for( auto &mb : build_memory_blocks() ) {
        snap.memory_blocks.push_back( std::make_unique<SubmapBlockT>( std::move( mb ) ) );
    }
    for( const EntVal &e : adapter_.entities() ) {
        snap.entities.push_back( std::make_unique<EntityT>( ent_to_wire( e ) ) );
        ent_cache_[e.id] = e;
    }
    for( const VehVal &v : adapter_.vehicles() ) {
        snap.vehicles.push_back( std::make_unique<VehicleT>( veh_to_wire( v ) ) );
        veh_cache_[v.id] = v;
    }
    snap.seq = ++seq_;
    snap.epoch = epoch_;
    has_snapshot_ = true;
    dirty_.clear();
    entities_dirty_ = vehicles_dirty_ = false;
    stats_.snapshots++;
    stats_.last_snapshot_ms =
        std::chrono::duration<double, std::milli>( std::chrono::steady_clock::now() - t0 ).count();
    return snap;
}

DeltaT Exporter::build_delta()
{
    const auto t0 = std::chrono::steady_clock::now();
    DeltaT d;
    stats_.last_dirty_submaps = dirty_.size();
    for( const BlockKey &k : dirty_ ) {
        auto it = cache_.find( k );
        if( it == cache_.end() ) {
            continue; // outside the snapshot window: nothing was ever sent
        }
        auto fresh = read_block( k );
        for( int i = 0; i < kTilesPerBlock; ++i ) {
            stats_.tiles_diffed++;
            if( fresh[i] != it->second[i] ) {
                stats_.tiles_changed++;
                auto tc = std::make_unique<TileChangeT>();
                tc->pos = std::make_unique<Vec3i>( v3( k.sx * kSubmapSize + ( i % kSubmapSize ),
                              k.sy * kSubmapSize + ( i / kSubmapSize ), k.z ) );
                tc->terrain = fresh[i].terrain;
                tc->furniture = fresh[i].furniture;
                tc->trap = fresh[i].trap;
                for( const FieldVal &f : fresh[i].fields ) {
                    auto fo = std::make_unique<FieldOnTileT>();
                    fo->tile_index = static_cast<uint16_t>( i );
                    fo->type = f.type;
                    fo->intensity = f.intensity;
                    tc->fields.push_back( std::move( fo ) );
                }
                tc->displayed_field = std::make_unique<DisplayedField>(
                                          fresh[i].displayed_type, fresh[i].displayed_intensity );
                tc->top_item = fresh[i].top_item;
                tc->item_count = fresh[i].item_count;
                tc->lit = static_cast<LitLevel>( static_cast<uint8_t>( fresh[i].lit ) );
                tc->seen = fresh[i].seen;
                tc->has_graffiti = fresh[i].graffiti;
                d.tile_changes.push_back( std::move( tc ) );
            }
        }
        it->second = std::move( fresh );
    }
    dirty_.clear();

    if( entities_dirty_ ) {
        std::set<uint64_t> seen_ids;
        for( const EntVal &e : adapter_.entities() ) {
            seen_ids.insert( e.id );
            auto it = ent_cache_.find( e.id );
            if( it == ent_cache_.end() || it->second != e ) {
                d.entity_updates.push_back( std::make_unique<EntityT>( ent_to_wire( e ) ) );
            }
            ent_cache_[e.id] = e;
        }
        for( auto it = ent_cache_.begin(); it != ent_cache_.end(); ) {
            if( !seen_ids.count( it->first ) ) {
                d.entity_removed.push_back( it->first );
                it = ent_cache_.erase( it );
            } else { ++it; }
        }
        entities_dirty_ = false;
    }
    if( vehicles_dirty_ ) {
        std::set<uint64_t> seen_ids;
        for( const VehVal &v : adapter_.vehicles() ) {
            seen_ids.insert( v.id );
            auto it = veh_cache_.find( v.id );
            if( it == veh_cache_.end() || it->second != v ) {
                d.vehicle_updates.push_back( std::make_unique<VehicleT>( veh_to_wire( v ) ) );
            }
            veh_cache_[v.id] = v;
        }
        for( auto it = veh_cache_.begin(); it != veh_cache_.end(); ) {
            if( !seen_ids.count( it->first ) ) {
                d.vehicle_removed.push_back( it->first );
                it = veh_cache_.erase( it );
            } else { ++it; }
        }
        vehicles_dirty_ = false;
    }
    d.seq = ++seq_;
    d.epoch = epoch_;
    stats_.deltas++;
    stats_.last_delta_ms =
        std::chrono::duration<double, std::milli>( std::chrono::steady_clock::now() - t0 ).count();
    return d;
}

void Exporter::mark_dirty_submap( int sx, int sy, int z ) { dirty_.insert( BlockKey{ sx, sy, z } ); }
void Exporter::mark_dirty_tile( int x, int y, int z )
{
    mark_dirty_submap( floor_div( x, kSubmapSize ), floor_div( y, kSubmapSize ), z );
}
void Exporter::mark_dirty_z( int z )
{
    for( const auto &[k, v] : cache_ ) {
        ( void )v;
        if( k.z == z ) { dirty_.insert( k ); }
    }
}
void Exporter::on_map_shift()
{
    // Bubble recentred: cached absolute positions are meaningless now.
    // Force a fresh snapshot under a new epoch (build_snapshot bumps it).
    force_resync_ = true;
    cache_.clear();
    dirty_.clear();
}

uint64_t Exporter::hash_tile_arrays( const std::map<BlockKey, std::array<TileVal, kTilesPerBlock>> &blocks )
{
    uint64_t h = 1469598103934665603ULL;
    for( const auto &[k, arr] : blocks ) {
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( k.sx ) ) );
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( k.sy ) ) );
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( k.z ) ) );
        for( const TileVal &t : arr ) {
            fnv( h, t.terrain );
            fnv( h, t.furniture );
            fnv( h, t.trap );
            fnv( h, t.fields.size() );
            for( const FieldVal &f : t.fields ) {
                fnv( h, f.type );
                fnv( h, f.intensity );
            }
            fnv( h, t.top_item );
            fnv( h, t.item_count );
            fnv( h, static_cast<uint8_t>( t.lit ) );
            fnv( h, t.seen );
            fnv( h, t.graffiti ? 1 : 0 );
        }
    }
    return h;
}

uint64_t Exporter::hash_snapshot( const SnapshotT &snap )
{
    uint64_t h = 1469598103934665603ULL;
    for( const auto &bp : snap.blocks ) {
        const SubmapBlockT &b = *bp;
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( b.origin->x() ) ) );
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( b.origin->y() ) ) );
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( b.origin->z() ) ) );
        for( int i = 0; i < kTilesPerBlock; ++i ) {
            fnv( h, b.terrain[i] );
            fnv( h, b.furniture[i] );
            fnv( h, b.trap[i] );
            fnv( h, b.top_item[i] );
            fnv( h, b.item_count[i] );
            fnv( h, static_cast<uint8_t>( b.lit[i] ) );
            fnv( h, b.seen[i] );
            fnv( h, b.has_graffiti[i] ? 1 : 0 );
        }
        for( const auto &fp : b.fields ) {
            fnv( h, fp->tile_index );
            fnv( h, fp->type );
            fnv( h, fp->intensity );
        }
    }
    for( const auto &ep : snap.entities ) {
        const EntityT &e = *ep;
        fnv( h, e.id );
        fnv( h, e.type_def );
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( e.pos->x() ) ) );
        fnv( h, static_cast<uint64_t>( static_cast<int64_t>( e.pos->y() ) ) );
        fnv( h, static_cast<uint8_t>( e.facing ) );
        fnv( h, e.hp_frac );
        for( const std::string &o : e.overlays ) { fnv_str( h, o ); }
    }
    for( const auto &vp : snap.vehicles ) {
        const VehicleT &v = *vp;
        fnv( h, v.id );
        uint32_t fb = 0;
        static_assert( sizeof( fb ) == sizeof( v.face_deg ) );
        std::memcpy( &fb, &v.face_deg, sizeof( fb ) );
        fnv( h, fb );
        for( const auto &pp : v.parts ) {
            fnv( h, pp->vpart );
            fnv( h, static_cast<uint64_t>( static_cast<int64_t>( pp->abs_pos->x() ) ) );
            fnv( h, static_cast<uint64_t>( static_cast<int64_t>( pp->abs_pos->y() ) ) );
            fnv( h, pp->hp_frac );
            fnv( h, pp->flags );
        }
    }
    return h;
}

} // namespace voxel_ipc
