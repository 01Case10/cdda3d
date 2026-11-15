// See voxel_refs.h. Pure stdlib: no game headers, so the fork
// harness build and the fork build compile the identical unit.
#include "voxel_refs.h"

#include <cstdlib>
#include <fstream>
#include <sstream>

namespace voxel_ipc
{

size_t FgBgRefs::load_text( const std::string &text )
{
    size_t loaded = 0;
    std::istringstream in( text );
    std::string line;
    while( std::getline( in, line ) ) {
        if( !line.empty() && line.back() == '\r' ) {
            line.pop_back();
        }
        if( line.empty() || line[0] == '#' ) {
            continue;
        }
        const size_t t1 = line.find( '\t' );
        if( t1 == std::string::npos ) {
            continue;
        }
        const size_t t2 = line.find( '\t', t1 + 1 );
        if( t2 == std::string::npos ) {
            continue;
        }
        const std::string id = line.substr( 0, t1 );
        const int32_t fg = static_cast<int32_t>(
                               std::strtol( line.substr( t1 + 1, t2 - t1 - 1 ).c_str(), nullptr, 10 ) );
        const int32_t bg = static_cast<int32_t>(
                               std::strtol( line.substr( t2 + 1 ).c_str(), nullptr, 10 ) );
        if( id.empty() ) {
            continue;
        }
        refs_[id] = { fg, bg };
        ++loaded;
    }
    return loaded;
}

bool FgBgRefs::load_file( const std::string &path )
{
    std::ifstream f( path );
    if( !f ) {
        return false;
    }
    std::ostringstream ss;
    ss << f.rdbuf();
    load_text( ss.str() );
    return true;
}

std::pair<int32_t, int32_t> FgBgRefs::lookup( const std::string &string_id ) const
{
    const auto it = refs_.find( string_id );
    if( it == refs_.end() ) {
        return { -1, -1 };
    }
    return it->second;
}

namespace
{
std::map<int, std::string> &group_names()
{
    static std::map<int, std::string> names;
    return names;
}
} // namespace

void record_connect_group_name( int index, const std::string &name )
{
    if( index >= 0 && !name.empty() ) {
        group_names()[index] = name;
    }
}

std::string connect_group_name( int index )
{
    const auto it = group_names().find( index );
    if( it != group_names().end() ) {
        return it->second;
    }
    return "group_" + std::to_string( index );
}

void clear_connect_group_names()
{
    group_names().clear();
}

} // namespace voxel_ipc
