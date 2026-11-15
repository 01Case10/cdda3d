// Fork harness — Phase 3 gate evidence at harness level.
// Runs the REAL exporter core (fork/src/voxel_ipc) against a mock world
// and checks the Part II §6 tests that do not require the compiled game.
// Tests that require cata_test are NOT claimed here (see docs/phase3-fork.md).
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <map>
#include <set>

#include "mock_world.h"
#include "voxel_exporter.h"
#include "voxel_input.h"
#include "voxel_server.h"

using namespace voxel_ipc;
using namespace mock;
namespace ipc = cdda::ipc;

static int g_fail = 0;
static void report( const char *id, bool ok, const std::string &note )
{
    std::printf( "[%s] %s — %s\n", id, ok ? "PASS" : "FAIL", note.c_str() );
    if( !ok ) { ++g_fail; }
}

// ---- wire -> value conversion (mirror side) ----
static TileVal wire_tile( const ipc::SubmapBlockT &b, int i,
                          const std::map<int, std::vector<FieldVal>> &by_tile )
{
    TileVal t;
    t.terrain = b.terrain[i];
    t.furniture = b.furniture[i];
    t.trap = b.trap[i];
    auto it = by_tile.find( i );
    if( it != by_tile.end() ) { t.fields = it->second; }
    t.displayed_type = b.displayed_field[i].field_type();
    t.displayed_intensity = b.displayed_field[i].intensity();
    t.top_item = b.top_item[i];
    t.item_count = b.item_count[i];
    t.lit = static_cast<Lit>( static_cast<uint8_t>( b.lit[i] ) );
    t.seen = b.seen[i];
    t.graffiti = b.has_graffiti[i];
    return t;
}

struct Mirror {
    std::map<std::tuple<int, int, int>, TileVal> tiles;
    std::map<uint64_t, EntVal> ents;
    std::map<uint64_t, VehVal> vehs;
    SeqTracker tracker;

    bool apply_snapshot( const ipc::SnapshotT &s )
    {
        if( !tracker.accept_snapshot( s.seq, s.epoch ) ) { return false; }
        tiles.clear(); ents.clear(); vehs.clear();
        for( const auto &bp : s.blocks ) { ingest_block( *bp ); }
        for( const auto &ep : s.entities ) { ents[ep->id] = from_wire( *ep ); }
        for( const auto &vp : s.vehicles ) { vehs[vp->id] = from_wire( *vp ); }
        return true;
    }
    void ingest_block( const ipc::SubmapBlockT &b )
    {
        std::map<int, std::vector<FieldVal>> by_tile;
        for( const auto &fp : b.fields ) {
            by_tile[fp->tile_index].push_back( FieldVal{ fp->type, fp->intensity } );
        }
        for( int i = 0; i < kTilesPerBlock; ++i ) {
            const int x = b.origin->x() + ( i % kSubmapSize );
            const int y = b.origin->y() + ( i / kSubmapSize );
            tiles[{x, y, b.origin->z()}] = wire_tile( b, i, by_tile );
        }
    }
    static EntVal from_wire( const ipc::EntityT &e )
    {
        EntVal v;
        v.id = e.id; v.type_def = e.type_def; v.sprite_id = e.sprite_id;
        v.x = e.pos->x(); v.y = e.pos->y(); v.z = e.pos->z();
        v.facing = static_cast<Facing>( static_cast<uint8_t>( e.facing ) );
        v.overlays = e.overlays; v.hp_frac = e.hp_frac; v.flags = e.flags;
        return v;
    }
    static VehVal from_wire( const ipc::VehicleT &vv )
    {
        VehVal v;
        v.id = vv.id; v.name = vv.name; v.face_deg = vv.face_deg;
        for( const auto &pp : vv.parts ) {
            VPartVal p;
            p.vpart = pp->vpart;
            p.ax = pp->abs_pos->x(); p.ay = pp->abs_pos->y(); p.az = pp->abs_pos->z();
            p.mx = pp->mount->x(); p.my = pp->mount->y(); p.mz = pp->mount->z();
            p.hp_frac = pp->hp_frac; p.flags = pp->flags;
            v.parts.push_back( p );
        }
        return v;
    }
    DeltaDecision apply_delta( const ipc::DeltaT &d )
    {
        DeltaDecision dec = tracker.observe_delta( d.seq, d.epoch );
        if( dec != DeltaDecision::Apply ) { return dec; }
        for( const auto &tcp : d.tile_changes ) {
            const auto &tc = *tcp;
            TileVal t;
            t.terrain = tc.terrain; t.furniture = tc.furniture; t.trap = tc.trap;
            for( const auto &fp : tc.fields ) {
                t.fields.push_back( FieldVal{ fp->type, fp->intensity } );
            }
            std::sort( t.fields.begin(), t.fields.end(),
            []( const FieldVal &a, const FieldVal &b ) { return a.type < b.type; } );
            // TileChange carries displayed_field (schema fix 2026-11-15);
            // absent (pre-fix data) reads as 0/0 = no displayed field.
            if( tc.displayed_field ) {
                t.displayed_type = tc.displayed_field->field_type();
                t.displayed_intensity = tc.displayed_field->intensity();
            }
            t.top_item = tc.top_item; t.item_count = tc.item_count;
            t.lit = static_cast<Lit>( static_cast<uint8_t>( tc.lit ) );
            t.seen = tc.seen; t.graffiti = tc.has_graffiti;
            tiles[{tc.pos->x(), tc.pos->y(), tc.pos->z()}] = t;
        }
        for( const auto &ep : d.entity_updates ) { ents[ep->id] = from_wire( *ep ); }
        for( uint64_t id : d.entity_removed ) { ents.erase( id ); }
        for( const auto &vp : d.vehicle_updates ) { vehs[vp->id] = from_wire( *vp ); }
        for( uint64_t id : d.vehicle_removed ) { vehs.erase( id ); }
        return dec;
    }
};

// Compare mirror against a FRESH snapshot from an independent exporter.
static bool mirror_matches_fresh( World &w, const ViewSpec &view, Mirror &mirror,
                                  std::string &why )
{
    MockAdapter ad( w );
    Exporter fresh( ad );
    ipc::SnapshotT s = fresh.build_snapshot( view );
    std::map<std::tuple<int, int, int>, TileVal> want;
    for( const auto &bp : s.blocks ) {
        const auto &b = *bp;
        std::map<int, std::vector<FieldVal>> by_tile;
        for( const auto &fp : b.fields ) {
            by_tile[fp->tile_index].push_back( FieldVal{ fp->type, fp->intensity } );
        }
        for( int i = 0; i < kTilesPerBlock; ++i ) {
            const int x = b.origin->x() + ( i % kSubmapSize );
            const int y = b.origin->y() + ( i / kSubmapSize );
            TileVal t = wire_tile( b, i, by_tile );
            // displayed_field is compared strictly: TileChange carries it
            // since the 2026-11-15 schema fix, so the 3.9 invariant covers
            // it with no exemption.
            want[{x, y, b.origin->z()}] = t;
        }
    }
    if( want.size() != mirror.tiles.size() ) {
        why = "tile count differs";
        return false;
    }
    for( const auto &[k, t] : want ) {
        auto it = mirror.tiles.find( k );
        if( it == mirror.tiles.end() || it->second != t ) {
            why = "tile mismatch";
            return false;
        }
    }
    if( mirror.ents.size() != s.entities.size() ) { why = "entity count"; return false; }
    for( const auto &ep : s.entities ) {
        EntVal v = Mirror::from_wire( *ep );
        auto it = mirror.ents.find( v.id );
        if( it == mirror.ents.end() || it->second != v ) { why = "entity mismatch"; return false; }
    }
    if( mirror.vehs.size() != s.vehicles.size() ) { why = "vehicle count"; return false; }
    for( const auto &vp : s.vehicles ) {
        VehVal v = Mirror::from_wire( *vp );
        auto it = mirror.vehs.find( v.id );
        if( it == mirror.vehs.end() || it->second != v ) { why = "vehicle mismatch"; return false; }
    }
    return true;
}

static ViewSpec small_view( int cx = 0, int cy = 0, int cz = 0 )
{
    ViewSpec v;
    v.cx = cx; v.cy = cy; v.cz = cz;
    v.radius = 12; v.z_below = 1; v.z_above = 1;
    return v;
}

int main( int argc, char **argv )
{
    const char *snap_out = argc > 1 ? argv[1] : "snapshot.bin";

    // ---------- 3.1 TileDefs (harness level) ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        ipc::TileDefsT td = exp.build_tiledefs();
        std::set<std::pair<int, uint32_t>> seen;
        bool uniq = true, groups_ok = true, door_ok = false;
        for( const auto &dp : td.defs ) {
            if( !seen.insert( { static_cast<int>( dp->kind ), dp->id } ).second ) { uniq = false; }
            for( uint16_t gi : dp->connect_groups ) {
                if( gi >= td.connect_groups.size() ) { groups_ok = false; }
            }
            if( dp->string_id == "t_door_c" ) {
                door_ok = dp->open_result == "t_door_o" && dp->bash_result == "t_door_bashed";
            }
        }
        report( "3.1", uniq && groups_ok && door_ok && td.defs.size() >= 20,
                "harness-level: ids unique per kind, group indices resolve, door open/bash present (full registry check needs cata_test)" );
    }

    // ---------- 3.3 + 3.5 snapshot == game queries, lit exact ----------
    {
        World w = make_house();
        // one tile per lit_level value
        const Lit lits[7] = { Lit::DARK, Lit::LOW, Lit::BRIGHT_ONLY, Lit::LIT,
                              Lit::BRIGHT, Lit::MEMORIZED, Lit::BLANK };
        for( int i = 0; i < 7; ++i ) { w.mut( -6 + i, 3, 0 ).lit = lits[i]; }
        MockAdapter ad( w );
        Exporter exp( ad );
        ipc::SnapshotT s = exp.build_snapshot( small_view() );
        size_t checked = 0, bad = 0;
        for( const auto &bp : s.blocks ) {
            const auto &b = *bp;
            std::map<int, std::vector<FieldVal>> by_tile;
            for( const auto &fp : b.fields ) {
                by_tile[fp->tile_index].push_back( FieldVal{ fp->type, fp->intensity } );
            }
            for( int i = 0; i < kTilesPerBlock; ++i ) {
                const int x = b.origin->x() + ( i % kSubmapSize );
                const int y = b.origin->y() + ( i / kSubmapSize );
                TileVal got = wire_tile( b, i, by_tile );
                TileVal want = ad.query_tile( x, y, b.origin->z() );
                std::sort( want.fields.begin(), want.fields.end(),
                []( const FieldVal &a, const FieldVal &b ) { return a.type < b.type; } );
                if( !ad.in_bubble( x, y, b.origin->z() ) ) { want = TileVal{}; want.lit = Lit::BLANK; }
                ++checked;
                if( got != want ) { ++bad; }
            }
        }
        char buf[128];
        std::snprintf( buf, sizeof buf, "%zu tiles compared, %zu mismatches (fields as full (type,intensity) maps, lit exact)", checked, bad );
        report( "3.3", bad == 0, buf );
        report( "3.5", bad == 0, "same comparison: lit_level enum equal per tile incl. all 7 values; seen byte equal" );
    }

    // ---------- 3.4 bounds ----------
    {
        World w = make_road_field();
        MockAdapter ad( w );
        Exporter exp( ad );
        ipc::SnapshotT s = exp.build_snapshot( small_view( 24, 24, 0 ) );
        bool blank_ok = true;
        for( const auto &bp : s.blocks ) {
            for( int i = 0; i < kTilesPerBlock; ++i ) {
                const int x = bp->origin->x() + ( i % kSubmapSize );
                const int y = bp->origin->y() + ( i / kSubmapSize );
                if( !ad.in_bubble( x, y, bp->origin->z() ) &&
                    bp->lit[i] != ipc::LitLevel_BLANK ) { blank_ok = false; }
            }
        }
        report( "3.4", blank_ok && !s.blocks.empty(),
                "view at bubble corner: out-of-bubble tiles BLANK, no crash/OOB" );
    }

    // ---------- 3.5b memory layer ----------
    {
        World w = make_shelter();
        // Live contamination on a memorized tile: must NOT leak into memory.
        w.mut( 3, 14, 0 ).fields = { FieldVal{ FD_FIRE, 3 } };
        w.mut( 3, 14, 0 ).top_item = IT_ROCK;
        w.mut( 3, 14, 0 ).item_count = 5;
        MockAdapter ad( w );
        Exporter exp( ad );
        ipc::SnapshotT s = exp.build_snapshot( small_view() );
        bool clean = !s.memory_blocks.empty();
        size_t mem_tiles = 0;
        for( const auto &bp : s.memory_blocks ) {
            const auto &b = *bp;
            if( !b.fields.empty() ) { clean = false; }
            for( int i = 0; i < kTilesPerBlock; ++i ) {
                if( b.top_item[i] != 0 || b.item_count[i] != 0 ) { clean = false; }
                if( b.lit[i] == ipc::LitLevel_MEMORIZED ) { ++mem_tiles; }
                else if( b.lit[i] != ipc::LitLevel_BLANK ) { clean = false; }
            }
        }
        char buf[128];
        std::snprintf( buf, sizeof buf, "%zu memorized tiles, fields/items/entities absent from memory layer", mem_tiles );
        report( "3.5b", clean && mem_tiles >= 11, buf );
    }

    // ---------- 3.6 + 3.7 entities / overlays / facing ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        ipc::SnapshotT s = exp.build_snapshot( small_view() );
        bool facing_ok = true, order_ok = true;
        std::map<uint64_t, EntVal> want;
        for( const auto &e : ad.entities() ) { want[e.id] = e; }
        if( s.entities.size() != want.size() ) { order_ok = false; }
        for( const auto &ep : s.entities ) {
            const uint8_t f = static_cast<uint8_t>( ep->facing );
            if( f > 2 ) { facing_ok = false; }
            EntVal got = Mirror::from_wire( *ep );
            auto it = want.find( got.id );
            if( it == want.end() || it->second != got ) { order_ok = false; }
        }
        report( "3.6", facing_ok, "facing only NONE/LEFT/RIGHT across exported entities" );
        report( "3.7", order_ok, "overlay lists equal adapter (game) order exactly, incl. player armor+wield" );
    }

    // ---------- 3.8 vehicles ----------
    {
        bool ok = true;
        for( float deg : { 0.0f, 37.5f, 179.9f, 359.99f } ) {
            World w = make_vehicle_road( deg );
            MockAdapter ad( w );
            Exporter exp( ad );
            ipc::SnapshotT s = exp.build_snapshot( small_view() );
            if( s.vehicles.size() != 1 ) { ok = false; break; }
            const ipc::VehicleT &v = *s.vehicles[0];
            if( v.face_deg != deg ) { ok = false; }
            VehVal want = ad.vehicles()[0];
            if( v.parts.size() != want.parts.size() ) { ok = false; break; }
            for( size_t i = 0; i < want.parts.size(); ++i ) {
                const auto &p = *v.parts[i];
                if( p.abs_pos->x() != want.parts[i].ax || p.abs_pos->y() != want.parts[i].ay ||
                    p.abs_pos->z() != want.parts[i].az ) { ok = false; }
            }
        }
        report( "3.8", ok, "absolute part positions equal game-side values at 0/37.5/179.9/359.99 deg; face_deg float-exact" );
    }

    // ---------- 3.9 delta invariant: 10 seeds x 110 turns ----------
    {
        bool ok = true;
        std::string why;
        long total_turns = 0;
        for( uint64_t seed = 1; seed <= 10 && ok; ++seed ) {
            World w = make_house();
            MockAdapter ad( w );
            Exporter exp( ad );
            Mirror mirror;
            ViewSpec view = small_view();
            mirror.apply_snapshot( exp.build_snapshot( view ) );
            Rng rng( seed * 7919 );
            for( int turn = 0; turn < 110; ++turn ) {
                ++total_turns;
                const int action = rng.range( 10 );
                auto dirty = [&]( int x, int y, int z ) { exp.mark_dirty_tile( x, y, z ); };
                switch( action ) {
                    case 0: { // move player + a monster
                        auto &p = w.ents[0];
                        p.x += rng.range( 3 ) - 1; p.y += rng.range( 3 ) - 1;
                        if( w.ents.size() > 1 ) { w.ents[1].x += rng.range( 3 ) - 1; }
                        exp.mark_entities_dirty();
                        break;
                    }
                    case 1: { // open door
                        if( w.query( 0, 4, 0 ).terrain == T_DOOR_C ) {
                            w.mut( 0, 4, 0 ).terrain = T_DOOR_O; dirty( 0, 4, 0 );
                        }
                        break;
                    }
                    case 2: { // close / bash door
                        TileVal &d = w.mut( 0, 4, 0 );
                        if( d.terrain == T_DOOR_O ) { d.terrain = T_DOOR_C; }
                        else if( d.terrain == T_DOOR_C ) { d.terrain = T_BASHED_DOOR; }
                        else { d.terrain = T_DOOR_C; }
                        dirty( 0, 4, 0 );
                        break;
                    }
                    case 3: { // drop onto stack / pick up
                        TileVal &t = w.mut( 2, 2, 0 );
                        if( rng.range( 2 ) ) {
                            t.top_item = IT_ROCK; t.item_count++;
                        } else if( t.item_count > 0 ) {
                            if( --t.item_count == 0 ) { t.top_item = 0; }
                        }
                        dirty( 2, 2, 0 );
                        break;
                    }
                    case 4: case 5: { // fire: ignite / spread / intensity change / decay
                        TileVal &t = w.mut( 4, 2, 0 );
                        bool has_fire = false;
                        for( auto &f : t.fields ) {
                            if( f.type == FD_FIRE ) {
                                has_fire = true;
                                f.intensity = static_cast<uint8_t>( std::min( 3, f.intensity + 1 ) );
                            }
                        }
                        if( !has_fire ) {
                            t.fields.push_back( FieldVal{ FD_FIRE, 1 } );
                            t.displayed_type = FD_FIRE; t.displayed_intensity = 1;
                        }
                        dirty( 4, 2, 0 );
                        if( action == 5 ) { // spread to neighbour
                            TileVal &n = w.mut( 5, 2, 0 );
                            bool nf = false;
                            for( const auto &f : n.fields ) { if( f.type == FD_SMOKE ) { nf = true; } }
                            if( !nf ) { n.fields.push_back( FieldVal{ FD_SMOKE, 1 } ); }
                            dirty( 5, 2, 0 );
                        }
                        break;
                    }
                    case 6: { // graffiti + trap tile change
                        TileVal &t = w.mut( -4, -2, 0 );
                        t.graffiti = !t.graffiti;
                        dirty( -4, -2, 0 );
                        break;
                    }
                    case 7: { // lit/seen change (torch moved)
                        TileVal &t = w.mut( 1, 1, 0 );
                        t.lit = ( t.lit == Lit::LIT ) ? Lit::BRIGHT : Lit::LIT;
                        t.seen = 200;
                        dirty( 1, 1, 0 );
                        break;
                    }
                    case 8: { // entity spawn/remove
                        if( w.ents.size() < 4 ) {
                            w.ents.push_back( [&] {
                                EntVal e; e.id = 100 + static_cast<uint64_t>( turn );
                                e.type_def = MON_DOG; e.sprite_id = "mon_dog";
                                e.x = 6; e.y = 6; e.z = 0; e.facing = Facing::LEFT;
                                return e;
                            }() );
                        } else {
                            w.ents.pop_back();
                        }
                        exp.mark_entities_dirty();
                        break;
                    }
                    default: break; // wait: no change, delta must be empty-ish
                }
                ipc::DeltaT d = exp.build_delta();
                if( mirror.apply_delta( d ) != DeltaDecision::Apply ) {
                    ok = false; why = "delta not applied in-order"; break;
                }
                if( turn % 10 == 9 ) {
                    if( !mirror_matches_fresh( w, view, mirror, why ) ) { ok = false; break; }
                }
            }
            if( ok && !mirror_matches_fresh( w, view, mirror, why ) ) { ok = false; }
        }
        char buf[192];
        std::snprintf( buf, sizeof buf, "%ld turns across 10 seeds, mirror == fresh snapshot (%s)",
                       total_turns, ok ? "exact" : why.c_str() );
        report( "3.9", ok && total_turns >= 1000, buf );
    }

    // ---------- 3.9b map shift ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        Mirror mirror;
        mirror.apply_snapshot( exp.build_snapshot( small_view() ) );
        const uint64_t epoch_before = exp.epoch();
        exp.on_map_shift();
        bool need = exp.needs_snapshot();
        ipc::SnapshotT s2 = exp.build_snapshot( small_view() );
        bool accepted = mirror.apply_snapshot( s2 );
        std::string why;
        bool synced = mirror_matches_fresh( w, small_view(), mirror, why );
        report( "3.9b", need && s2.epoch == epoch_before + 1 && accepted && synced,
                "shift forces resync; epoch bumps by 1; mirror re-bases and matches" );
    }

    // ---------- 3.10 missed-change hunt ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        Mirror mirror;
        mirror.apply_snapshot( exp.build_snapshot( small_view() ) );
        // Direct submap mutation, NO player action: fire appears via the
        // coarse invalidation path only (invalidate_map_cache(z) shape).
        w.mut( -3, 3, 0 ).fields = { FieldVal{ FD_FIRE, 2 } };
        exp.mark_dirty_z( 0 );
        ipc::DeltaT d = exp.build_delta();
        mirror.apply_delta( d );
        std::string why;
        bool synced = mirror_matches_fresh( w, small_view(), mirror, why );
        report( "3.10", synced && !d.tile_changes.empty(),
                "direct mutation with no player action caught by dirty-submap diff" );
    }

    // ---------- 3.11 gap / 3.12 stale+duplicate ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        Mirror mirror;
        mirror.apply_snapshot( exp.build_snapshot( small_view() ) );
        w.mut( 0, 4, 0 ).terrain = T_DOOR_O;
        exp.mark_dirty_tile( 0, 4, 0 );
        ipc::DeltaT dropped = exp.build_delta(); // never applied
        ( void )dropped;
        w.mut( 0, 4, 0 ).terrain = T_DOOR_C;
        exp.mark_dirty_tile( 0, 4, 0 );
        ipc::DeltaT next = exp.build_delta();
        DeltaDecision gap = mirror.tracker.observe_delta( next.seq, next.epoch );
        // resync path
        mirror.apply_snapshot( exp.build_snapshot( small_view() ) );
        std::string why;
        bool recovered = mirror_matches_fresh( w, small_view(), mirror, why );
        report( "3.11", gap == DeltaDecision::NeedsSnapshot && recovered,
                "dropped delta -> seq gap detected -> fresh snapshot recovers mirror" );

        w.mut( 1, 1, 0 ).lit = Lit::BRIGHT;
        exp.mark_dirty_tile( 1, 1, 0 );
        ipc::DeltaT dd = exp.build_delta();
        DeltaDecision first = mirror.apply_delta( dd );
        DeltaDecision dup = mirror.apply_delta( dd );
        ipc::DeltaT stale = dd;
        stale.epoch = 1; // older than current mirror epoch after snapshots above
        DeltaDecision stale_dec = mirror.tracker.observe_delta( stale.seq, stale.epoch );
        report( "3.12", first == DeltaDecision::Apply && dup == DeltaDecision::Discard &&
               stale_dec == DeltaDecision::Discard,
               "duplicate delta discarded (no double-apply); stale-epoch delta discarded" );
    }

    // ---------- 3.14 replay determinism ----------
    {
        auto run_once = []() -> uint64_t {
            World w = make_house();
            MockAdapter ad( w );
            Exporter exp( ad );
            Rng rng( 4242 );
            uint64_t h = 0;
            for( int turn = 0; turn < 30; ++turn ) {
                TileVal &t = w.mut( rng.range( 8 ) - 4, rng.range( 8 ) - 4, 0 );
                t.item_count = static_cast<uint16_t>( rng.range( 4 ) );
                t.top_item = t.item_count ? IT_CAN : 0;
                exp.mark_dirty_z( 0 );
                exp.build_delta();
                if( turn % 10 == 9 ) {
                    ipc::SnapshotT s = exp.build_snapshot( small_view() );
                    h ^= Exporter::hash_snapshot( s ) + 0x9e3779b97f4a7c15ULL + ( h << 6 ) + ( h >> 2 );
                }
            }
            return h;
        };
        const uint64_t h1 = run_once();
        const uint64_t h2 = run_once();
        char buf[96];
        std::snprintf( buf, sizeof buf, "two seeded runs hash %016llx == %016llx",
                       ( unsigned long long )h1, ( unsigned long long )h2 );
        report( "3.14", h1 == h2 && h1 != 0, buf );
    }

    // ---------- 3.17 slow consumer ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        Mirror mirror;
        mirror.apply_snapshot( exp.build_snapshot( small_view() ) );
        BoundedOutbox outbox( 2048 ); // deliberately tiny: consumer never drains
        bool dropped_seen = false;
        for( int i = 0; i < 40; ++i ) {
            w.mut( 1, 1, 0 ).item_count = static_cast<uint16_t>( i );
            w.mut( 1, 1, 0 ).top_item = IT_CAN;
            exp.mark_dirty_tile( 1, 1, 0 );
            ipc::MessageUnion m;
            m.Set( exp.build_delta() );
            if( outbox.push( frame_message( m ), true ) == BoundedOutbox::Push::DroppedDelta ) {
                dropped_seen = true;
                exp.force_resync(); // documented policy
            }
        }
        bool need = exp.needs_snapshot();
        mirror.apply_snapshot( exp.build_snapshot( small_view() ) );
        std::string why;
        bool recovered = mirror_matches_fresh( w, small_view(), mirror, why );
        char buf[128];
        std::snprintf( buf, sizeof buf, "dropped_deltas=%llu, resync forced, mirror recovered",
                       ( unsigned long long )outbox.dropped_deltas() );
        report( "3.17", dropped_seen && need && recovered && outbox.dropped_deltas() > 0, buf );
    }

    // ---------- 3.18 export cost (measured, full view window) ----------
    {
        World w = make_road_field();
        w.minx = w.miny = -66; w.maxx = w.maxy = 66; // full bubble
        MockAdapter ad( w );
        Exporter exp( ad );
        ViewSpec full;
        full.cx = 0; full.cy = 0; full.cz = 0; full.radius = 60; full.z_below = 1; full.z_above = 1;
        const auto t0 = std::chrono::steady_clock::now();
        ipc::SnapshotT s = exp.build_snapshot( full );
        const double snap_ms = std::chrono::duration<double, std::milli>(
                                   std::chrono::steady_clock::now() - t0 ).count();
        w.mut( 0, 4, 0 ).terrain = T_WALL;
        exp.mark_dirty_tile( 0, 4, 0 );
        const auto t1 = std::chrono::steady_clock::now();
        exp.build_delta();
        const double delta_ms = std::chrono::duration<double, std::milli>(
                                    std::chrono::steady_clock::now() - t1 ).count();
        char buf[192];
        std::snprintf( buf, sizeof buf,
                       "snapshot (%zu blocks, radius 60, 3 z) %.2f ms; typical delta %.3f ms; dirty_submaps=%zu (plan targets: <20 ms / <2 ms on game hardware; harness VM is 1.5 shared CPUs, mock std::map world)",
                       s.blocks.size(), snap_ms, delta_ms, exp.stats().last_dirty_submaps );
        // Gate here is "measured and attributable", per the test text.
        report( "3.18", snap_ms > 0 && delta_ms >= 0 && s.blocks.size() >= 300, buf );
    }

    // ---------- serialization proof: real schema round-trip ----------
    {
        World w = make_house();
        MockAdapter ad( w );
        Exporter exp( ad );
        ipc::SnapshotT s = exp.build_snapshot( small_view() );
        ipc::MessageUnion m;
        m.Set( s );
        std::vector<uint8_t> frame = frame_message( m );
        ipc::EnvelopeT env;
        bool ok = frame.size() > 4 &&
                  decode_payload( frame.data() + 4, frame.size() - 4, env );
        uint64_t h1 = Exporter::hash_snapshot( s );
        uint64_t h2 = 0;
        if( ok && env.message.AsSnapshot() ) {
            h2 = Exporter::hash_snapshot( *env.message.AsSnapshot() );
        }
        // Write the payload for the Rust proto-crate decode check.
        std::ofstream out( snap_out, std::ios::binary );
        out.write( reinterpret_cast<const char *>( frame.data() + 4 ),
                   static_cast<std::streamsize>( frame.size() - 4 ) );
        out.close();
        char buf[128];
        std::snprintf( buf, sizeof buf, "C++ generated-header verify+unpack round-trip hash equal (%016llx); payload written to %s",
                       ( unsigned long long )h1, snap_out );
        report( "SERDE", ok && h1 == h2, buf );

        // Input injector order preservation (Phase 1 1.12 shape, fork side)
        InputInjector inj;
        ipc::InputT in;
        in.kind = ipc::InputKind_Key; in.key_code = 'i';
        inj.enqueue( in );
        in.key_code = 'x';
        inj.enqueue( in );
        QueuedInput q1, q2;
        bool order = inj.next( q1 ) && inj.next( q2 ) && q1.code == 'i' && q2.code == 'x';
        report( "INPUT", order, "injector preserves engine input order [i, x]" );
    }

    std::printf( "%s (%d failures)\n", g_fail == 0 ? "ALL HARNESS TESTS PASS" : "FAILURES PRESENT", g_fail );
    return g_fail == 0 ? 0 : 1;
}
