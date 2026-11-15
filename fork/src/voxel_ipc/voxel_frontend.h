// CDDA Voxel Frontend — Phase 3 frontend singleton: the ONLY object the
// one-line hooks in do_turn.cpp / game.cpp / map.cpp / main.cpp touch.
//
// Lifecycle: main.cpp's --frontend=ipc handler calls enable() with the
// socket path; the first send boundary starts the server, handshakes
// (Hello -> HelloAck -> TileDefs -> Snapshot), then flushes a Delta at
// every boundary. Shutdown (wire message, peer EOF, or heartbeat
// timeout) stops the frontend without touching simulation state.
#pragma once

#include <memory>
#include <string>

#include "voxel_exporter.h"
#include "voxel_input.h"
#include "voxel_server.h"

namespace voxel_ipc
{

class Frontend
{
    public:
        static Frontend &instance();

        void enable( IGameAdapter &adapter, std::string socket_path );
        bool enabled() const { return enabled_; }
        void disable();

        // Hook entry points (all no-ops when disabled).
        void on_send_boundary( const ViewSpec &view ); // do_turn exit + draw redraw
        void on_invalidate_map_cache( int z ) { if( exp_ ) { exp_->mark_dirty_z( z ); } }
        void on_seen_dirty( int x, int y, int z ) { if( exp_ ) { exp_->mark_dirty_tile( x, y, z ); } }
        void on_transparency_dirty( int z ) { if( exp_ ) { exp_->mark_dirty_z( z ); } }
        void on_map_shift() { if( exp_ ) { exp_->on_map_shift(); } }
        void on_creatures_changed() { if( exp_ ) { exp_->mark_entities_dirty(); } }
        void on_vehicles_changed() { if( exp_ ) { exp_->mark_vehicles_dirty(); } }

        InputInjector &input() { return input_; }
        Exporter *exporter() { return exp_.get(); }
        bool peer_alive( uint64_t now_ms ) const { return !hb_.timed_out( now_ms ); }

    private:
        Frontend() = default;
        bool handshake_done();
        void send_snapshot( const ViewSpec &view );
        void send_delta();

        bool enabled_ = false;
        bool started_ = false;
        bool handshaken_ = false;
        std::string path_;
        IGameAdapter *adapter_ = nullptr;
        std::unique_ptr<Exporter> exp_;
        Server server_;
        BoundedOutbox outbox_;
        InputInjector input_;
        Heartbeat hb_;
};

// Process-wide flag set by main.cpp's --frontend=ipc parse (cli_opts).
bool frontend_requested();
void set_frontend_requested( const std::string &ipc_path );
const std::string &frontend_ipc_path();

} // namespace voxel_ipc
