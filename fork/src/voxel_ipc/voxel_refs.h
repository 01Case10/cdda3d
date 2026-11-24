// CDDA Voxel Frontend -- Phase 3 follow-up: sprite refs + group names.
//
// Two pieces of TileDefs data the exporter cannot derive from the
// game's registries alone, factored into one game-header-free unit
// so the standalone fork harness compiles and gates them exactly
// as the fork build will run them:
//
// 1. fg/bg sprite refs. A def's fg_ref/bg_ref are indices into the
//    tileset's COMPOSED tile_config.json (the game's tiles renderer
//    resolves sprites there; the decomposed per-sprite files the
//    voxelizer consumes carry names, not indices). The mapping is
//    generated offline from the composed config by the tileset
//    crate's fg_refs tool (crates/tileset/src/bin/fg_refs.rs) into
//    a TSV the adapter loads at startup. Refs are the section-local
//    composed indices, matching the schema's single-int fields.
//    An id with no composed entry keeps the -1 sentinel: the
//    exporter must not invent indices (docs/phase3-fork.md SS4.4).
//
// 2. Connect-group names. The game maps name -> group at load
//    (connect_group::load); no index -> name lookup exists. Hook
//    H8 (fork/hooks/hook_patches.md) mirrors each loaded name into
//    the registry below, and connect_group_table() reads it back.
//    An index with no recorded name keeps the stable placeholder
//    "group_<index>" -- indices are the load-bearing data and are
//    always exact.
#pragma once

#include <cstdint>
#include <map>
#include <string>
#include <utility>

namespace voxel_ipc
{

class FgBgRefs
{
    public:
        // Parse TSV text: one "string_id <TAB> fg <TAB> bg" row per
        // def. Blank lines and '#' comments are skipped; malformed
        // rows are skipped (a partial table is better than none --
        // missing ids resolve to the -1 sentinel either way).
        // Returns the number of rows loaded.
        size_t load_text( const std::string &text );
        // Load the TSV from a file. Returns false when the file
        // cannot be read (the table then stays as it was).
        bool load_file( const std::string &path );
        // (fg, bg) for a def's string id; (-1, -1) when unmapped.
        std::pair<int32_t, int32_t> lookup( const std::string &string_id ) const;
        size_t size() const { return refs_.size(); }
        void clear() { refs_.clear(); }

    private:
        std::map<std::string, std::pair<int32_t, int32_t>> refs_;
};

// The connect-group name mirror (hook H8's receiving end).
void record_connect_group_name( int index, const std::string &name );
// Recorded name for an index, or "group_<index>" when H8 has not
// recorded one (e.g. a group the hook did not observe).
std::string connect_group_name( int index );
// Test support: forget every recorded name.
void clear_connect_group_names();

} // namespace voxel_ipc
