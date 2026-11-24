// CDDA Voxel Frontend — Phase 3 frontend singleton. See voxel_frontend.h.
#include "voxel_frontend.h"

#include <chrono>

namespace voxel_ipc
{
namespace ipc = cdda::ipc;
namespace
{
bool g_requested = false;
std::string g_path = "/tmp/cdda-voxel.sock";
uint64_t now_ms()
{
    return static_cast<uint64_t>( std::chrono::duration_cast<std::chrono::milliseconds>(
               std::chrono::steady_clock::now().time_since_epoch() ).count() );
}
} // namespace

bool frontend_requested() { return g_requested; }
void set_frontend_requested( const std::string &ipc_path )
{
    g_requested = true;
    if( !ipc_path.empty() ) { g_path = ipc_path; }
}
const std::string &frontend_ipc_path() { return g_path; }

Frontend &Frontend::instance()
{
    static Frontend f;
    return f;
}

void Frontend::enable( IGameAdapter &adapter, std::string socket_path )
{
    adapter_ = &adapter;
    path_ = std::move( socket_path );
    exp_ = std::make_unique<Exporter>( adapter );
    enabled_ = true;
    hb_.last_seen_ms = hb_.last_ping_ms = now_ms();
}

void Frontend::disable()
{
    if( started_ ) {
        ipc::MessageUnion m;
        ipc::ShutdownT sd; sd.reason = "game exiting"; m.Set( sd );
        // best effort: queue + one flush attempt, never block on exit
        outbox_.push( frame_message( m ), false );
        server_.flush( outbox_ );
    }
    server_.close();
    enabled_ = false;
    started_ = false;
    handshaken_ = false;
}

bool Frontend::handshake_done()
{
    if( handshaken_ ) { return true; }
    if( !started_ ) {
        if( !server_.listen_on( path_ ) ) { return false; }
        started_ = true;
    }
    if( !server_.accept_client() ) { return false; }
    ipc::MessageUnion hello;
    hello.Set( exp_->build_hello() );
    outbox_.push( frame_message( hello ), false );
    server_.flush( outbox_ );
    bool ok = false;
    server_.poll_input( [&]( const ipc::MessageUnion & m ) {
        if( m.type == ipc::Message_HelloAck ) {
            const auto *ack = m.AsHelloAck();
            ok = ack != nullptr && ack->protocol_version == kProtocolVersion; // mismatch = clean stop
        }
    } );
    if( !ok ) { return false; }
    ipc::MessageUnion defs;
    defs.Set( exp_->build_tiledefs() );
    outbox_.push( frame_message( defs ), false );
    server_.flush( outbox_ );
    hb_.note_seen( now_ms() );
    handshaken_ = true;
    return true;
}

void Frontend::send_snapshot( const ViewSpec &view )
{
    ipc::MessageUnion m;
    m.Set( exp_->build_snapshot( view ) );
    outbox_.push( frame_message( m ), false ); // snapshots evict deltas, never drop
}

void Frontend::send_delta()
{
    ipc::MessageUnion m;
    m.Set( exp_->build_delta() );
    if( outbox_.push( frame_message( m ), true ) == BoundedOutbox::Push::DroppedDelta ) {
        // Test 3.17 policy: drop the delta, force resync via new epoch.
        exp_->force_resync();
    }
}

void Frontend::on_send_boundary( const ViewSpec &view )
{
    if( !enabled_ || !handshake_done() ) { return; }
    const uint64_t now = now_ms();
    if( hb_.timed_out( now ) ) {
        disable(); // peer dead: clean frontend shutdown, game continues headless-safe
        return;
    }
    bool alive = server_.poll_input( [&]( const ipc::MessageUnion & m ) {
        hb_.note_seen( now_ms() );
        switch( m.type ) {
            case ipc::Message_Input:
                input_.enqueue( *m.AsInput() );
                break;
            case ipc::Message_Ping: {
                ipc::MessageUnion pong;
                auto p = ipc::PongT{};
                p.nonce = m.AsPing()->nonce;
                p.sent_ms = now_ms();
                pong.Set( std::move( p ) );
                outbox_.push( frame_message( pong ), false );
                break;
            }
            case ipc::Message_Shutdown:
                disable();
                break;
            default:
                break;
        }
    } );
    if( !alive ) { disable(); return; }
    if( exp_->needs_snapshot() ) {
        send_snapshot( view );
    } else {
        send_delta();
    }
    // Phase 10: a due environment update that did not ride the
    // snapshot/delta above goes out as a standalone message.
    if( auto env = exp_->build_environment_update() ) {
        ipc::MessageUnion m;
        m.Set( std::move( *env ) );
        outbox_.push( frame_message( m ), false );
    }
    if( hb_.should_ping( now ) ) {
        ipc::MessageUnion ping;
        auto p = ipc::PingT{};
        p.nonce = exp_->seq() + 1;
        p.sent_ms = now;
        ping.Set( std::move( p ) );
        outbox_.push( frame_message( ping ), false );
        hb_.last_ping_ms = now;
    }
    server_.flush( outbox_ );
}

} // namespace voxel_ipc
