// CDDA Voxel Frontend — Phase 3: framing, bounded outbox, socket server.
//
// Wire rules are Phase 1's (docs/protocol.md): u32 LE length prefix +
// FlatBuffers Envelope, 32 MiB cap rejected from the prefix alone, CDDA
// file identifier checked on decode, verifier before any table access.
//
// Slow-consumer policy (test 3.17, plan Phase 3): the game NEVER blocks
// on the socket. Outbound frames go into a bounded byte-capped outbox;
// a Delta that does not fit is DROPPED and the exporter is told to force
// a resync (new epoch + fresh Snapshot) — a Snapshot is never silently
// dropped: if it does not fit, the outbox is cleared for it.
#pragma once

#include <cstdint>
#include <deque>
#include <functional>
#include <string>
#include <vector>

#include "cdda_ipc_generated.h"

namespace voxel_ipc
{

// Serialize one native message into a framed wire frame (prefix included).
std::vector<uint8_t> frame_message( const cdda::ipc::MessageUnion &msg );
// Verify + unpack one payload (no prefix). Returns false on any failure.
bool decode_payload( const uint8_t *data, size_t len, cdda::ipc::EnvelopeT &out );

class BoundedOutbox
{
    public:
        explicit BoundedOutbox( size_t byte_cap = 8u * 1024 * 1024 ) : cap_( byte_cap ) {}
        // DroppedDelta: a delta was lost by this push (this frame, or an
        // older queued delta evicted for it) — caller must force resync.
        enum class Push { Ok, DroppedDelta };
        // is_delta frames may be dropped on overflow; non-delta frames
        // evict queued deltas to make room (snapshot priority).
        Push push( std::vector<uint8_t> frame, bool is_delta );
        bool pop( std::vector<uint8_t> &out );
        size_t bytes() const { return bytes_; }
        size_t size() const { return queue_.size(); }
        uint64_t dropped_deltas() const { return dropped_; }
    private:
        size_t cap_;
        size_t bytes_ = 0;
        uint64_t dropped_ = 0;
        std::deque<std::pair<std::vector<uint8_t>, bool>> queue_; // (frame, is_delta)
};

// Minimal seq/epoch tracker — the C++ mirror of crates/proto SeqTracker
// (gate 1.14 semantics), used fork-side to validate engine requests and
// by the harness to model the engine.
enum class DeltaDecision { Apply, Discard, NeedsSnapshot };
class SeqTracker
{
    public:
        bool accept_snapshot( uint64_t seq, uint64_t epoch );
        DeltaDecision observe_delta( uint64_t seq, uint64_t epoch );
        bool has_epoch() const { return has_epoch_; }
        uint64_t epoch() const { return epoch_; }
        uint64_t last_seq() const { return last_seq_; }
    private:
        bool has_epoch_ = false;
        uint64_t epoch_ = 0;
        uint64_t last_seq_ = 0;
};

// Heartbeat bookkeeping (pure, clock injected — gate 1.10 shape).
struct Heartbeat {
    uint64_t timeout_ms = 3000;
    uint64_t interval_ms = 1000;
    uint64_t last_seen_ms = 0;
    uint64_t last_ping_ms = 0;
    void note_seen( uint64_t now ) { last_seen_ms = now; }
    bool should_ping( uint64_t now ) const { return now - last_ping_ms >= interval_ms; }
    bool timed_out( uint64_t now ) const { return now - last_seen_ms > timeout_ms; }
};

// Unix-socket server (Linux v1). Non-blocking accept/poll; the game loop
// calls poll() at send boundaries, never inside simulation.
class Server
{
    public:
        ~Server();
        bool listen_on( const std::string &path );
        bool accept_client();            // non-blocking; true once connected
        bool connected() const { return client_fd_ >= 0; }
        // Drain the outbox to the socket (non-blocking writes). Returns
        // false when the peer is gone (caller shuts down cleanly).
        bool flush( BoundedOutbox &outbox );
        // Read at most one inbound frame; cb runs only after verification.
        bool poll_input( const std::function<void( const cdda::ipc::MessageUnion & )> &cb );
        void close();
    private:
        int listen_fd_ = -1;
        int client_fd_ = -1;
        std::string path_;
        std::vector<uint8_t> inbuf_;
};

} // namespace voxel_ipc
