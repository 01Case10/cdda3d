// Stub IPC server for the CDDA Voxel Frontend (Phase 1): stands in for the
// fork's voxel_ipc server so the Rust client and protocol tests can run
// without building the game.
//
//   stub-server <unix-socket-path>
//
// Protocol: sends Hello; on HelloAck sends TileDefs + the fake-house
// Snapshot; echoes Input back verbatim; answers Ping with Pong; exits
// cleanly on Shutdown (or when the client disconnects).
//
// Framing: u32 LE length + FlatBuffers Envelope payload, max 32 MiB — the
// same rules as crates/proto/src/framing.rs. Every inbound buffer is run
// through the FlatBuffers verifier before any table access.

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include "cdda_ipc_generated.h"
#include "house.h"

// --- Shared-memory UI header layout check (gate 1.13, C++ side) -----------
// Must match crates/proto/src/shm.rs byte-for-byte.
struct UiShmHeader {
    uint32_t magic;
    uint32_t width;
    uint32_t height;
    uint32_t stride;
    uint64_t seq;
    int32_t dirty_x;
    int32_t dirty_y;
    int32_t dirty_w;
    int32_t dirty_h;
    uint8_t buffer_index;
    uint8_t _pad[7];
};
static_assert(sizeof(UiShmHeader) == 48, "UiShmHeader must be 48 bytes");
static_assert(offsetof(UiShmHeader, seq) == 16, "UiShmHeader.seq offset");
static_assert(offsetof(UiShmHeader, dirty_x) == 24, "UiShmHeader.dirty_x offset");
static_assert(offsetof(UiShmHeader, buffer_index) == 40, "UiShmHeader.buffer_index offset");
constexpr uint32_t kUiShmMagic = 0x43444155;  // "CDAU"
static_assert(kUiShmMagic == 0x43444155, "magic");

namespace {

using namespace cdda::ipc;
constexpr uint32_t kMaxFrame = 32u * 1024 * 1024;

bool ReadExact(int fd, uint8_t* buf, size_t n) {
    size_t got = 0;
    while (got < n) {
        ssize_t r = ::read(fd, buf + got, n - got);
        if (r <= 0) return false;
        got += static_cast<size_t>(r);
    }
    return true;
}

bool WriteExact(int fd, const uint8_t* buf, size_t n) {
    size_t sent = 0;
    while (sent < n) {
        ssize_t w = ::write(fd, buf + sent, n - sent);
        if (w <= 0) return false;
        sent += static_cast<size_t>(w);
    }
    return true;
}

// Returns false on EOF / oversize / io error.
bool ReadFrame(int fd, std::vector<uint8_t>* out) {
    uint8_t hdr[4];
    if (!ReadExact(fd, hdr, 4)) return false;
    uint32_t len = static_cast<uint32_t>(hdr[0]) | (static_cast<uint32_t>(hdr[1]) << 8) |
                   (static_cast<uint32_t>(hdr[2]) << 16) | (static_cast<uint32_t>(hdr[3]) << 24);
    if (len > kMaxFrame) {
        std::fprintf(stderr, "stub: oversize frame %u, closing\n", len);
        return false;
    }
    out->resize(len);
    return ReadExact(fd, out->data(), len);
}

bool SendFrame(int fd, const flatbuffers::FlatBufferBuilder& b) {
    uint32_t len = b.GetSize();
    uint8_t hdr[4] = {static_cast<uint8_t>(len & 0xff), static_cast<uint8_t>((len >> 8) & 0xff),
                      static_cast<uint8_t>((len >> 16) & 0xff), static_cast<uint8_t>((len >> 24) & 0xff)};
    return WriteExact(fd, hdr, 4) && WriteExact(fd, b.GetBufferPointer(), len);
}

template <typename T, typename Native>
bool SendNative(int fd, Message type, const Native& native) {
    flatbuffers::FlatBufferBuilder b;
    auto off = T::Pack(b, &native);
    auto env = CreateEnvelope(b, type, off.Union());
    FinishEnvelopeBuffer(b, env);
    return SendFrame(fd, b);
}

bool VerifyEnvelope(const std::vector<uint8_t>& buf) {
    flatbuffers::Verifier v(buf.data(), buf.size());
    return VerifyEnvelopeBuffer(v);
}

}  // namespace

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "usage: stub-server <unix-socket-path>\n");
        return 2;
    }
    const char* path = argv[1];
    ::unlink(path);
    int listener = ::socket(AF_UNIX, SOCK_STREAM, 0);
    if (listener < 0) { std::perror("socket"); return 1; }
    sockaddr_un addr{};
    addr.sun_family = AF_UNIX;
    std::strncpy(addr.sun_path, path, sizeof(addr.sun_path) - 1);
    if (::bind(listener, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) < 0) {
        std::perror("bind");
        return 1;
    }
    if (::listen(listener, 1) < 0) { std::perror("listen"); return 1; }
    std::fprintf(stderr, "stub: listening on %s\n", path);
    int fd = ::accept(listener, nullptr, nullptr);
    if (fd < 0) { std::perror("accept"); return 1; }

    // Handshake: Hello out, HelloAck in.
    auto hello = house::BuildHello();
    if (!SendNative<Hello>(fd, Message_Hello, hello)) return 1;
    std::vector<uint8_t> frame;
    if (!ReadFrame(fd, &frame) || !VerifyEnvelope(frame)) {
        std::fprintf(stderr, "stub: bad first frame\n");
        return 1;
    }
    const Envelope* env = GetEnvelope(frame.data());
    if (env->message_type() != Message_HelloAck) {
        std::fprintf(stderr, "stub: expected HelloAck, got %d\n", env->message_type());
        return 1;
    }
    const HelloAck* ack = env->message_as_HelloAck();
    if (ack->protocol_version() != house::kProtocolVersion) {
        std::fprintf(stderr, "stub: protocol mismatch: engine=%u stub=%u\n",
                     ack->protocol_version(), house::kProtocolVersion);
        return 1;
    }
    std::fprintf(stderr, "stub: helloack engine=%s screen=%ux%u\n",
                 ack->engine_version() ? ack->engine_version()->c_str() : "?",
                 ack->screen_width(), ack->screen_height());

    auto defs = house::BuildTileDefs();
    if (!SendNative<TileDefs>(fd, Message_TileDefs, defs)) return 1;
    auto snap = house::BuildSnapshot();
    if (!SendNative<Snapshot>(fd, Message_Snapshot, snap)) return 1;
    std::fprintf(stderr, "stub: sent tiledefs + snapshot\n");

    // Main loop.
    while (true) {
        if (!ReadFrame(fd, &frame)) {
            std::fprintf(stderr, "stub: client disconnected\n");
            break;
        }
        if (!VerifyEnvelope(frame)) {
            std::fprintf(stderr, "stub: verifier rejected frame, closing\n");
            break;
        }
        env = GetEnvelope(frame.data());
        switch (env->message_type()) {
            case Message_Input: {
                const Input* in = env->message_as_Input();
                std::fprintf(stderr, "stub: input kind=%d key=%d pressed=%d\n",
                             in->kind(), in->key_code(), in->pressed());
                // Echo the input back verbatim (re-encoded) for round-trip
                // assertion by the client.
                std::unique_ptr<InputT> copy(in->UnPack());
                if (!SendNative<Input>(fd, Message_Input, *copy)) return 1;
                break;
            }
            case Message_Ping: {
                const Ping* ping = env->message_as_Ping();
                flatbuffers::FlatBufferBuilder b;
                auto off = CreatePong(b, ping->nonce(), ping->sent_ms());
                auto e = CreateEnvelope(b, Message_Pong, off.Union());
                FinishEnvelopeBuffer(b, e);
                if (!SendFrame(fd, b)) return 1;
                break;
            }
            case Message_Resize: {
                const Resize* r = env->message_as_Resize();
                std::fprintf(stderr, "stub: resize %ux%u dpi=%f\n", r->width(), r->height(), r->dpi());
                break;
            }
            case Message_PickResult: {
                const PickResult* p = env->message_as_PickResult();
                std::fprintf(stderr, "stub: pick hit=%d pos=(%d,%d,%d)\n", p->hit(),
                             p->pos()->x(), p->pos()->y(), p->pos()->z());
                break;
            }
            case Message_Shutdown: {
                const Shutdown* s = env->message_as_Shutdown();
                std::fprintf(stderr, "stub: shutdown reason=%s\n",
                             s->reason() ? s->reason()->c_str() : "");
                ::close(fd);
                ::close(listener);
                ::unlink(path);
                std::fprintf(stderr, "stub: clean exit\n");
                return 0;
            }
            default:
                std::fprintf(stderr, "stub: unexpected message type %d\n", env->message_type());
                break;
        }
    }
    ::close(fd);
    ::close(listener);
    ::unlink(path);
    return 0;
}
