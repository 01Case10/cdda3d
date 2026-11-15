// Fork harness mock worlds. See mock_world.h.
#include "mock_world.h"

namespace mock
{
namespace
{
using namespace voxel_ipc;

TileVal base( uint32_t ter ) {
    TileVal t;
    t.terrain = ter;
    t.lit = Lit::LIT;
    t.seen = 255;
    return t;
}

void room( World &w, int x0, int y0, int x1, int y1, int z, uint32_t floor_ter ) {
    for( int y = y0; y <= y1; ++y ) {
        for( int x = x0; x <= x1; ++x ) {
            const bool edge = x == x0 || x == x1 || y == y0 || y == y1;
            w.set( x, y, z, base( edge ? T_WALL : floor_ter ) );
        }
    }
}

EntVal ent( uint64_t id, uint32_t type, const char *sprite, int x, int y, int z,
            Facing f, std::vector<std::string> overlays ) {
    EntVal e;
    e.id = id; e.type_def = type; e.sprite_id = sprite;
    e.x = x; e.y = y; e.z = z; e.facing = f;
    e.overlays = std::move( overlays );
    e.hp_frac = 255;
    return e;
}

} // namespace

World make_shelter()
{
    World w;
    w.default_tile = base( T_GRASS );
    room( w, -5, -5, 5, 5, 0, T_FLOOR );
    w.mut( 0, 5, 0 ).terrain = T_DOOR_C;
    w.mut( -2, -5, 0 ).terrain = T_WINDOW;
    w.mut( 1, 1, 0 ).furniture = F_BED;
    w.mut( 2, 2, 0 ).top_item = IT_ROCK;
    w.mut( 2, 2, 0 ).item_count = 3;
    w.ents.push_back( ent( 1, 0, "player", 0, 0, 0, Facing::RIGHT,
                           { "mut_base", "clothing_coat", "wield_rock" } ) );
    // Remembered street to the south: terrain/furniture memory only.
    for( int x = -5; x <= 5; ++x ) {
        w.memory.push_back( MemVal{ x, 14, 0, T_ROAD, 0 } );
    }
    w.memory.push_back( MemVal{ 3, 14, 0, T_ROAD, F_TABLE } );
    return w;
}

World make_house()
{
    World w;
    w.default_tile = base( T_GRASS );
    room( w, -6, -4, 6, 4, 0, T_FLOOR );
    w.mut( 0, 4, 0 ).terrain = T_DOOR_C;
    w.mut( -3, -4, 0 ).terrain = T_WINDOW;
    w.mut( 3, -4, 0 ).terrain = T_WINDOW;
    w.mut( 0, 0, 0 ).terrain = T_WALL; // interior wall
    w.mut( -2, 1, 0 ).furniture = F_TABLE;
    TileVal fire = w.query( 4, 2, 0 );
    fire.fields = { FieldVal{ FD_FIRE, 1 }, FieldVal{ FD_SMOKE, 2 } };
    fire.displayed_type = FD_SMOKE;
    fire.displayed_intensity = 2;
    w.set( 4, 2, 0, fire );
    w.mut( -4, -2, 0 ).trap = TRAP_SPIKE;
    w.mut( -4, -2, 0 ).graffiti = true;
    w.ents.push_back( ent( 1, 0, "player", 0, 2, 0, Facing::LEFT,
                           { "mut_base", "clothing_armor" } ) );
    w.ents.push_back( ent( 7, MON_ZOMBIE, "mon_zombie", 8, 0, 0, Facing::NONE,
                           { "overlay_zombie" } ) );
    return w;
}

World make_road_field()
{
    World w;
    w.default_tile = base( T_GRASS );
    for( int x = -20; x <= 20; ++x ) {
        for( int dy = -2; dy <= 2; ++dy ) {
            w.set( x, dy, 0, base( T_ROAD ) );
        }
    }
    w.mut( 5, 5, 0 ).terrain = T_DIRT;
    return w;
}

World make_multistorey()
{
    World w;
    w.default_tile = base( T_GRASS );
    room( w, -4, -4, 4, 4, -1, T_FLOOR );       // basement
    room( w, -4, -4, 4, 4, 0, T_FLOOR );        // ground
    room( w, -4, -4, 4, 4, 1, T_FLOOR );        // first floor
    for( int y = -4; y <= 4; ++y ) {            // roof over first floor
        for( int x = -4; x <= 4; ++x ) {
            w.set( x, y, 2, base( T_ROOF ) );
        }
    }
    w.mut( 0, 4, 0 ).terrain = T_DOOR_C;
    w.mut( 0, 0, 1 ).terrain = T_OPEN_AIR;      // stairwell opening
    w.mut( 0, 0, 0 ).terrain = T_OPEN_AIR;
    w.ents.push_back( ent( 1, 0, "player", 0, 0, 1, Facing::RIGHT, { "mut_base" } ) );
    return w;
}

World make_vehicle_road( float face_deg )
{
    World w = make_road_field();
    // Game-side absolute positions: the mock "precalc" places parts on a
    // 3x2 footprint rotated to an axis-aligned heading for 0/90/180/270
    // and on the exact same tiles for the 37-degree case (tile positions
    // are integers; the FLOAT facing is what must survive export).
    VehVal v;
    v.id = 42;
    v.name = "test car";
    v.face_deg = face_deg;
    auto part = []( uint32_t vp, int ax, int ay, int mx, int my, uint32_t flags ) {
        VPartVal p;
        p.vpart = vp; p.ax = ax; p.ay = ay; p.az = 0;
        p.mx = mx; p.my = my; p.mz = 0; p.flags = flags;
        return p;
    };
    v.parts.push_back( part( VP_FRAME, 0, 0, 0, 0, 0 ) );
    v.parts.push_back( part( VP_WHEEL, 1, -1, 1, -1, 0 ) );
    v.parts.push_back( part( VP_WHEEL, 1, 1, 1, 1, 0 ) );
    v.parts.push_back( part( VP_DOOR, 0, 1, 0, 1, 1 ) );  // open
    v.parts.push_back( part( VP_ROOF, 0, 0, 0, 0, 4 ) );  // roof flag
    w.vehs.push_back( v );
    w.ents.push_back( ent( 1, 0, "player", 3, 3, 0, Facing::RIGHT, {} ) );
    return w;
}

std::vector<DefVal> MockAdapter::tile_defs()
{
    std::vector<DefVal> out;
    auto def = [&]( uint32_t id, const char *sid, const char *name, uint8_t kind ) {
        DefVal d;
        d.id = id; d.string_id = sid; d.name = name; d.kind = kind;
        out.push_back( d );
        return out.size() - 1;
    };
    def( T_FLOOR, "t_floor", "floor", 0 );
    size_t wall = def( T_WALL, "t_wall", "wall", 0 );
    out[wall].flags = { "CONNECT_TO_WALL" };
    out[wall].hot_flags = 0b101;
    out[wall].connect_groups = { 0 };
    out[wall].connects_to_groups = { 0 };
    out[wall].rotate_to_groups = { 0 };
    size_t door = def( T_DOOR_C, "t_door_c", "closed door", 0 );
    out[door].open_result = "t_door_o";
    out[door].bash_result = "t_door_bashed";
    out[door].looks_like = "t_door_c";
    def( T_WINDOW, "t_window", "window", 0 );
    size_t open = def( T_DOOR_O, "t_door_o", "open door", 0 );
    out[open].close_result = "t_door_c";
    def( T_GRASS, "t_grass", "grass", 0 );
    def( T_ROAD, "t_road", "road", 0 );
    def( T_DIRT, "t_dirt", "dirt", 0 );
    def( T_ROOF, "t_roof", "roof", 0 );
    def( T_OPEN_AIR, "t_open_air", "open air", 0 );
    def( T_BASHED_DOOR, "t_door_bashed", "bashed door", 0 );
    def( F_TABLE, "f_table", "table", 1 );
    def( F_BED, "f_bed", "bed", 1 );
    def( TRAP_SPIKE, "tr_spike", "spike trap", 2 );
    def( FD_SMOKE, "fd_smoke", "smoke", 3 );
    def( FD_FIRE, "fd_fire", "fire", 3 );
    def( IT_ROCK, "rock", "rock", 5 );
    def( IT_CAN, "can", "can", 5 );
    def( MON_ZOMBIE, "mon_zombie", "zombie", 4 );
    def( MON_DOG, "mon_dog", "dog", 4 );
    def( VP_FRAME, "frame", "frame", 6 );
    def( VP_WHEEL, "wheel", "wheel", 6 );
    def( VP_DOOR, "door", "vehicle door", 6 );
    def( VP_ROOF, "roof", "vehicle roof", 6 );
    // fg/bg sprite refs, filled through the same FgBgRefs table
    // the real adapter loads from CDDA_VOXEL_FG_REFS (voxel_refs.h):
    // the mock's "composed config" maps three of its ids; every
    // other def keeps the -1 sentinel, exactly as an unmapped id
    // behaves in the fork build.
    FgBgRefs refs;
    refs.load_text( "# mock composed mapping\n"
                    "t_floor\t0\t1\n"
                    "t_wall\t3\t-1\n"
                    "t_door_c\t7\t2\n" );
    for( DefVal &d : out ) {
        const std::pair<int32_t, int32_t> fg_bg = refs.lookup( d.string_id );
        d.fg_ref = fg_bg.first;
        d.bg_ref = fg_bg.second;
    }
    return out;
}

} // namespace mock
