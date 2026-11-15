// CDDA Voxel Frontend — Phase 3 exporter core (serializer + diff).
//
// Owns: seq/epoch, the dirty-submap set, the cached copy of everything
// last sent, and conversion of adapter values into the Phase 1
// FlatBuffers schema (schema/cdda_ipc.fbs, generated header alongside).
//
// Delta mechanism (plan §5 Phase 3, recon §1 Q2): there is NO per-tile
// change hook upstream. The hooks only accumulate dirty submaps; at each
// send boundary this class diffs those submaps against its cached copy.
// The diff IS the mechanism — a change made by any means (fire spread,
// a monster opening a door, a direct submap write) is emitted as long as
// its submap was invalidated, and test 3.10's direct-mutation case is
// covered because the harness marks the submap dirty exactly the way
// map::invalidate_map_cache does.
#pragma once

#include <array>
#include <cstdint>
#include <map>
#include <set>
#include <string>
#include <tuple>
#include <vector>

#include "cdda_ipc_generated.h"
#include "voxel_adapter.h"

namespace voxel_ipc
{

struct BlockKey {
    int sx = 0, sy = 0, z = 0; // submap coords (tile origin / 12) + z-level
    bool operator<( const BlockKey &o ) const {
        return std::tie( sx, sy, z ) < std::tie( o.sx, o.sy, o.z );
    }
    bool operator==( const BlockKey &o ) const {
        return sx == o.sx && sy == o.sy && z == o.z;
    }
};

struct ViewSpec {
    int cx = 0, cy = 0, cz = 0;   // view centre (player tile)
    int radius = kMaxViewDistance; // <= 60
    int z_below = 1, z_above = 1;  // cutaway z-window around cz
};

struct ExportStats {
    uint64_t snapshots = 0;
    uint64_t deltas = 0;
    uint64_t tiles_diffed = 0;
    uint64_t tiles_changed = 0;
    double last_snapshot_ms = 0.0;
    double last_delta_ms = 0.0;
    size_t last_dirty_submaps = 0;
};

class Exporter
{
    public:
        explicit Exporter( IGameAdapter &adapter ) : adapter_( adapter ) {}

        // --- handshake / defs ---
        cdda::ipc::HelloT build_hello() const;
        cdda::ipc::TileDefsT build_tiledefs();

        // --- state sync ---
        // Full snapshot: bumps epoch, rebuilds the cached copy, sets seq.
        cdda::ipc::SnapshotT build_snapshot( const ViewSpec &view );
        // Delta for the accumulated dirty set (plus entity/vehicle diffs).
        // If no snapshot exists yet, or a resync was forced (shift /
        // slow-consumer overflow), the caller must send a snapshot instead:
        // needs_snapshot() reports that.
        cdda::ipc::DeltaT build_delta();
        bool needs_snapshot() const {
            return !has_snapshot_ || force_resync_;
        }

        // --- dirty hooks (called by the one-line hooks in map.cpp/map.h) ---
        void mark_dirty_submap( int sx, int sy, int z );
        void mark_dirty_tile( int x, int y, int z );
        void mark_dirty_z( int z );           // invalidate_map_cache(z) semantics
        void mark_entities_dirty() { entities_dirty_ = true; }
        void mark_vehicles_dirty() { vehicles_dirty_ = true; }
        // map::shift: bubble recentred — epoch bump + full resync.
        void on_map_shift();
        // Slow-consumer overflow (test 3.17): drop deltas, force resync.
        void force_resync() {
            force_resync_ = true;
        }

        uint64_t epoch() const { return epoch_; }
        uint64_t seq() const { return seq_; }
        const ExportStats &stats() const { return stats_; }
        size_t dirty_count() const { return dirty_.size(); }

        // Deterministic hash over the EXPORTED tile/entity/vehicle data
        // only (plan Phase 3 exit criteria / Principle 4 field list):
        // per block origin + terrain/furniture/trap/fields/top_item/
        // item_count/lit/seen/graffiti, then entities, then vehicles.
        // Excludes seq/epoch/timing by construction.
        static uint64_t hash_snapshot( const cdda::ipc::SnapshotT &snap );
        static uint64_t hash_tile_arrays( const std::map<BlockKey, std::array<TileVal, kTilesPerBlock>> &blocks );

    private:
        std::array<TileVal, kTilesPerBlock> read_block( const BlockKey &k );
        cdda::ipc::SubmapBlockT block_to_wire( const BlockKey &k,
                                               const std::array<TileVal, kTilesPerBlock> &tiles ) const;
        std::vector<cdda::ipc::SubmapBlockT> build_memory_blocks() const;
        static cdda::ipc::EntityT ent_to_wire( const EntVal &e );
        static cdda::ipc::VehicleT veh_to_wire( const VehVal &v );

        IGameAdapter &adapter_;
        uint64_t seq_ = 0;
        uint64_t epoch_ = 0;
        bool has_snapshot_ = false;
        bool force_resync_ = false;
        bool entities_dirty_ = true;
        bool vehicles_dirty_ = true;
        ViewSpec last_view_{};
        std::set<BlockKey> dirty_;
        std::map<BlockKey, std::array<TileVal, kTilesPerBlock>> cache_;
        std::map<uint64_t, EntVal> ent_cache_;
        std::map<uint64_t, VehVal> veh_cache_;
        ExportStats stats_;
};

} // namespace voxel_ipc
