#include <algorithm>
// CDDA Voxel Frontend — Phase 3 server/framing. See voxel_server.h.
#include "voxel_server.h"

#include "voxel_types.h"

#include <cstring>

#if defined(__linux__) || defined(__unix__)
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#endif

namespace voxel_ipc
{
namespace ipc = cdda::ipc;

std::vector<uint8_t> frame_message( const ipc::MessageUnion &msg )
{
    flatbuffers::FlatBufferBuilder b;
    ipc::EnvelopeT env;
    env.message = msg;
    auto off = ipc::Envelope::Pack( b, &env );
    ipc::FinishEnvelopeBuffer( b, off );
    const uint32_t len = b.GetSize();
    std::vector<uint8_t> out( 4 + len );
    out[0] = static_cast<uint8_t>( len & 0xff );
    out[1] = static_cast<uint8_t>( ( len >> 8 ) & 0xff );
    out[2] = static_cast<uint8_t>( ( len >> 16 ) & 0xff );
    out[3] = static_cast<uint8_t>( ( len >> 24 ) & 0xff );
    std::memcpy( out.data() + 4, b.GetBufferPointer(), len );
    return out;
}

bool decode_payload( const uint8_t *data, size_t len, ipc::EnvelopeT &out )
{
    if( len < 8 || len > kMaxFrame ) { return false; }
    // Stock verifier does not check the file identifier (Phase 1 finding).
    if( std::memcmp( data + 4, "CDDA", 4 ) != 0 ) { return false; }
    flatbuffers::Verifier v( data, len );
    if( !ipc::VerifyEnvelopeBuffer( v ) ) { return false; }
    const ipc::Envelope *env = ipc::GetEnvelope( data );
    if( env == nullptr || env->message_type() == ipc::Message_NONE ) { return false; }
    env->UnPackTo( &out );
    return true;
}

BoundedOutbox::Push BoundedOutbox::push( std::vector<uint8_t> frame, bool is_delta )
{
    // DroppedDelta means "at least one delta was lost by this push" —
    // either this frame itself, or an older queued delta evicted to make
    // room for it. Either way the receiver now has a seq gap and the
    // caller MUST force a resync (test 3.17 policy); the frame itself is
    // still queued whenever it fits after eviction.
    const uint64_t dropped_before = dropped_;
    if( frame.size() > cap_ ) {
        if( is_delta ) { dropped_++; return Push::DroppedDelta; }
        queue_.clear(); // an oversized control frame still must go out
        bytes_ = 0;
    }
    while( bytes_ + frame.size() > cap_ ) {
        // Evict the oldest delta first; non-deltas are never evicted.
        auto it = std::find_if( queue_.begin(), queue_.end(),
        []( const auto &p ) { return p.second; } );
        if( it == queue_.end() ) {
            if( is_delta ) { dropped_++; return Push::DroppedDelta; }
            break; // control frame: exceed cap rather than lose it
        }
        bytes_ -= it->first.size();
        dropped_++;
        queue_.erase( it );
    }
    bytes_ += frame.size();
    queue_.emplace_back( std::move( frame ), is_delta );
    return dropped_ > dropped_before ? Push::DroppedDelta : Push::Ok;
}

bool BoundedOutbox::pop( std::vector<uint8_t> &out )
{
    if( queue_.empty() ) { return false; }
    out = std::move( queue_.front().first );
    bytes_ -= out.size();
    queue_.pop_front();
    return true;
}

bool SeqTracker::accept_snapshot( uint64_t seq, uint64_t epoch )
{
    if( has_epoch_ && epoch < epoch_ ) { return false; }
    has_epoch_ = true;
    epoch_ = epoch;
    last_seq_ = seq;
    return true;
}

DeltaDecision SeqTracker::observe_delta( uint64_t seq, uint64_t epoch )
{
    if( !has_epoch_ ) { return DeltaDecision::NeedsSnapshot; }
    if( epoch < epoch_ ) { return DeltaDecision::Discard; }       // stale epoch (3.12)
    if( epoch > epoch_ ) { return DeltaDecision::NeedsSnapshot; }
    if( seq <= last_seq_ ) { return DeltaDecision::Discard; }     // duplicate (3.12)
    if( seq == last_seq_ + 1 ) {
        last_seq_ = seq;
        return DeltaDecision::Apply;
    }
    return DeltaDecision::NeedsSnapshot;                          // seq gap (3.11)
}

Server::~Server() { close(); }

bool Server::listen_on( const std::string &path )
{
#if defined(__linux__) || defined(__unix__)
    path_ = path;
    ::unlink( path.c_str() );
    listen_fd_ = ::socket( AF_UNIX, SOCK_STREAM, 0 );
    if( listen_fd_ < 0 ) { return false; }
    sockaddr_un addr{};
    addr.sun_family = AF_UNIX;
    std::strncpy( addr.sun_path, path.c_str(), sizeof( addr.sun_path ) - 1 );
    if( ::bind( listen_fd_, reinterpret_cast<sockaddr *>( &addr ), sizeof( addr ) ) < 0 ) {
        return false;
    }
    if( ::listen( listen_fd_, 1 ) < 0 ) { return false; }
    ::fcntl( listen_fd_, F_SETFL, ::fcntl( listen_fd_, F_GETFL, 0 ) | O_NONBLOCK );
    return true;
#else
    ( void )path;
    return false; // Windows named-pipe transport is Phase 8 packaging scope
#endif
}

bool Server::accept_client()
{
#if defined(__linux__) || defined(__unix__)
    if( client_fd_ >= 0 ) { return true; }
    client_fd_ = ::accept( listen_fd_, nullptr, nullptr );
    if( client_fd_ < 0 ) { return false; }
    ::fcntl( client_fd_, F_SETFL, ::fcntl( client_fd_, F_GETFL, 0 ) | O_NONBLOCK );
    return true;
#else
    return false;
#endif
}

bool Server::flush( BoundedOutbox &outbox )
{
#if defined(__linux__) || defined(__unix__)
    std::vector<uint8_t> frame;
    while( outbox.pop( frame ) ) {
        size_t sent = 0;
        while( sent < frame.size() ) {
            ssize_t w = ::send( client_fd_, frame.data() + sent, frame.size() - sent,
                                MSG_NOSIGNAL | MSG_DONTWAIT );
            if( w < 0 ) {
                if( errno == EAGAIN || errno == EWOULDBLOCK ) {
                    // Slow consumer: put it back at the head is not
                    // supported by the deque API; treat as would-block and
                    // stop — the frame is re-queued by the caller policy
                    // (deltas are droppable; see voxel_frontend.cpp).
                    return true;
                }
                return false; // peer gone
            }
            sent += static_cast<size_t>( w );
        }
    }
    return true;
#else
    ( void )outbox;
    return false;
#endif
}

bool Server::poll_input( const std::function<void( const ipc::MessageUnion & )> &cb )
{
#if defined(__linux__) || defined(__unix__)
    uint8_t buf[16384];
    ssize_t r = ::recv( client_fd_, buf, sizeof( buf ), MSG_DONTWAIT );
    if( r == 0 ) { return false; } // clean EOF == Shutdown (protocol.md)
    if( r < 0 ) { return errno == EAGAIN || errno == EWOULDBLOCK; }
    inbuf_.insert( inbuf_.end(), buf, buf + r );
    for( ;; ) {
        if( inbuf_.size() < 4 ) { break; }
        const uint32_t len = static_cast<uint32_t>( inbuf_[0] ) |
                             ( static_cast<uint32_t>( inbuf_[1] ) << 8 ) |
                             ( static_cast<uint32_t>( inbuf_[2] ) << 16 ) |
                             ( static_cast<uint32_t>( inbuf_[3] ) << 24 );
        if( len > kMaxFrame ) { return false; } // rejected from prefix alone
        if( inbuf_.size() < 4 + len ) { break; }
        ipc::EnvelopeT env;
        if( !decode_payload( inbuf_.data() + 4, len, env ) ) { return false; }
        cb( env.message );
        inbuf_.erase( inbuf_.begin(), inbuf_.begin() + 4 + len );
    }
    return true;
#else
    ( void )cb;
    return false;
#endif
}

void Server::close()
{
#if defined(__linux__) || defined(__unix__)
    if( client_fd_ >= 0 ) { ::close( client_fd_ ); client_fd_ = -1; }
    if( listen_fd_ >= 0 ) { ::close( listen_fd_ ); listen_fd_ = -1; }
    if( !path_.empty() ) { ::unlink( path_.c_str() ); path_.clear(); }
#endif
}

} // namespace voxel_ipc
