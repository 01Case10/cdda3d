// Pure C++ protocol checks (gates 1.2 and 1.3):
//
//   cpp_roundtrip                  encode+decode every sample in C++, assert
//                                  equality of the important fields.
//   cpp_roundtrip write <dir>      write golden envelope payloads
//                                  (golden_snapshot.bin, golden_tiledefs.bin,
//                                  golden_hello.bin) for the Rust side to read.
//   cpp_roundtrip verify <file>    read a Rust-written Snapshot envelope
//                                  payload, verify + decode it, and assert it
//                                  is semantically equal to the C++ build.
//
// Golden files are raw Envelope payloads (no length prefix).

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <memory>
#include <string>
#include <vector>

#include "cdda_ipc_generated.h"
#include "house.h"

namespace {

using namespace cdda::ipc;

int g_failures = 0;
#define CHECK(cond)                                                     \
    do {                                                                \
        if (!(cond)) {                                                  \
            std::fprintf(stderr, "CHECK FAILED %s:%d: %s\n", __FILE__, __LINE__, #cond); \
            ++g_failures;                                               \
        }                                                               \
    } while (0)

template <typename T, typename Native>
std::vector<uint8_t> PackEnvelope(Message type, const Native& native) {
    flatbuffers::FlatBufferBuilder b;
    auto off = T::Pack(b, &native);
    auto env = CreateEnvelope(b, type, off.Union());
    FinishEnvelopeBuffer(b, env);
    return {b.GetBufferPointer(), b.GetBufferPointer() + b.GetSize()};
}

bool Verify(const std::vector<uint8_t>& buf) {
    flatbuffers::Verifier v(buf.data(), buf.size());
    return VerifyEnvelopeBuffer(v);
}

void CheckEnvironmentEqual(const EnvironmentUpdateT& a, const EnvironmentUpdateT& b);

void CheckSnapshotEqual(const SnapshotT& a, const SnapshotT& b) {
    CHECK(a.seq == b.seq);
    CHECK(a.epoch == b.epoch);
    CHECK(a.view_radius == b.view_radius);
    CHECK(a.blocks.size() == b.blocks.size());
    if (a.blocks.size() == 1 && b.blocks.size() == 1) {
        const SubmapBlockT& x = *a.blocks[0];
        const SubmapBlockT& y = *b.blocks[0];
        CHECK(x.terrain == y.terrain);
        CHECK(x.furniture == y.furniture);
        CHECK(x.trap == y.trap);
        CHECK(x.field_index == y.field_index);
        CHECK(x.top_item == y.top_item);
        CHECK(x.item_count == y.item_count);
        CHECK(x.lit == y.lit);
        CHECK(x.seen == y.seen);
        CHECK(x.has_graffiti == y.has_graffiti);
        // Phase 10: per-tile sky exposure, same stub-house rule.
        CHECK(x.outside == y.outside);
        if (x.outside.size() == house::TILES) {
            CHECK(!x.outside[house::idx(6, 6)]);  // interior: not exposed
            CHECK(x.outside[house::idx(0, 0)]);   // corner: exposed
        }
        CHECK(x.fields.size() == y.fields.size());
        CHECK(x.fields.size() == 3);
        for (size_t i = 0; i < x.fields.size() && i < y.fields.size(); ++i) {
            CHECK(x.fields[i]->tile_index == y.fields[i]->tile_index);
            CHECK(x.fields[i]->type == y.fields[i]->type);
            CHECK(x.fields[i]->intensity == y.fields[i]->intensity);
        }
        CHECK(x.displayed_field.size() == y.displayed_field.size());
        if (x.displayed_field.size() == house::TILES) {
            CHECK(x.displayed_field[house::idx(6, 6)].field_type() == house::FIELD_FIRE);
            CHECK(x.displayed_field[house::idx(6, 6)].intensity() == 1);
        }
        // The two-fields-on-one-tile invariant, explicitly:
        int fields_on_centre = 0;
        for (auto& f : x.fields) {
            if (f->tile_index == house::idx(6, 6)) ++fields_on_centre;
        }
        CHECK(fields_on_centre == 2);
        CHECK(x.item_count[house::idx(4, 4)] == 3);
        CHECK(x.top_item[house::idx(4, 4)] == house::ITEM_ROCK);
    }
    CHECK(a.entities.size() == b.entities.size());
    if (!a.entities.empty() && !b.entities.empty()) {
        CHECK(a.entities[0]->id == b.entities[0]->id);
        CHECK(a.entities[0]->facing == b.entities[0]->facing);
        CHECK(a.entities[0]->overlays == b.entities[0]->overlays);
        CHECK(a.entities[0]->pos->x() == 6 && a.entities[0]->pos->y() == 5);
    }
    CHECK(a.vehicles.size() == b.vehicles.size());
    if (!a.vehicles.empty() && !b.vehicles.empty()) {
        CHECK(a.vehicles[0]->face_deg == b.vehicles[0]->face_deg);
        CHECK(a.vehicles[0]->parts.size() == b.vehicles[0]->parts.size());
        CHECK(a.vehicles[0]->parts[0]->abs_pos->x() == 20);
    }
    // Phase 10: the piggyback environment must be present and equal.
    CHECK((a.environment == nullptr) == (b.environment == nullptr));
    if (a.environment && b.environment) {
        CheckEnvironmentEqual(*a.environment, *b.environment);
    }
}

void CheckEnvironmentEqual(const EnvironmentUpdateT& a, const EnvironmentUpdateT& b) {
    CHECK(a.seq == b.seq);
    CHECK(a.epoch == b.epoch);
    CHECK(a.turn == b.turn);
    CHECK(a.time_of_day_sec == b.time_of_day_sec);
    CHECK(a.sunrise_sec == b.sunrise_sec);
    CHECK(a.sunset_sec == b.sunset_sec);
    CHECK(a.season == b.season);
    CHECK(a.sun_light == b.sun_light);
    CHECK(a.moon_light == b.moon_light);
    CHECK(a.sun_azimuth_deg == b.sun_azimuth_deg);
    CHECK(a.sun_altitude_deg == b.sun_altitude_deg);
    CHECK(a.moon_phase == b.moon_phase);
    CHECK(a.weather_id == b.weather_id);
    CHECK(a.light_multiplier == b.light_multiplier);
    CHECK(a.light_modifier == b.light_modifier);
    CHECK(a.sun_multiplier == b.sun_multiplier);
    CHECK(a.precip_class == b.precip_class);
    CHECK(a.rains == b.rains);
    CHECK(a.sight_penalty == b.sight_penalty);
    CHECK(a.tint_rgb == b.tint_rgb);
    CHECK(a.tint_strength == b.tint_strength);
    CHECK(a.temperature_c == b.temperature_c);
    CHECK(a.windspeed_mph == b.windspeed_mph);
    CHECK(a.wind_dir_deg == b.wind_dir_deg);
    CHECK(a.lightning_active == b.lightning_active);
}

bool WriteFile(const std::string& path, const std::vector<uint8_t>& data) {
    std::ofstream out(path, std::ios::binary);
    out.write(reinterpret_cast<const char*>(data.data()), data.size());
    return out.good();
}

bool ReadFile(const std::string& path, std::vector<uint8_t>* data) {
    std::ifstream in(path, std::ios::binary);
    if (!in) return false;
    *data = {std::istreambuf_iterator<char>(in), std::istreambuf_iterator<char>()};
    return true;
}

}  // namespace

int main(int argc, char** argv) {
    std::string mode = argc > 1 ? argv[1] : "selftest";

    if (mode == "write") {
        if (argc != 3) { std::fprintf(stderr, "usage: cpp_roundtrip write <dir>\n"); return 2; }
        std::string dir = argv[2];
        auto snap = house::BuildSnapshot();
        auto defs = house::BuildTileDefs();
        auto hello = house::BuildHello();
        auto env = house::BuildEnvironment();
        bool ok = WriteFile(dir + "/golden_snapshot.bin", PackEnvelope<Snapshot>(Message_Snapshot, snap)) &&
                  WriteFile(dir + "/golden_tiledefs.bin", PackEnvelope<TileDefs>(Message_TileDefs, defs)) &&
                  WriteFile(dir + "/golden_hello.bin", PackEnvelope<Hello>(Message_Hello, hello)) &&
                  WriteFile(dir + "/golden_environment.bin",
                            PackEnvelope<EnvironmentUpdate>(Message_EnvironmentUpdate, env));
        std::printf("CPP WRITE %s\n", ok ? "PASS" : "FAIL");
        return ok ? 0 : 1;
    }

    if (mode == "verify") {
        if (argc != 3) { std::fprintf(stderr, "usage: cpp_roundtrip verify <snapshot-file>\n"); return 2; }
        std::vector<uint8_t> buf;
        if (!ReadFile(argv[2], &buf)) { std::fprintf(stderr, "cannot read %s\n", argv[2]); return 1; }
        CHECK(Verify(buf));
        if (g_failures) { std::printf("CPP VERIFY FAIL\n"); return 1; }
        const Envelope* env = GetEnvelope(buf.data());
        CHECK(env->message_type() == Message_Snapshot);
        std::unique_ptr<SnapshotT> got(env->message_as_Snapshot()->UnPack());
        auto want = house::BuildSnapshot();
        CheckSnapshotEqual(*got, want);
        std::printf("CPP VERIFY %s\n", g_failures ? "FAIL" : "PASS");
        return g_failures ? 1 : 0;
    }

    // selftest: pure C++ round-trip (gate 1.2).
    {
        auto snap = house::BuildSnapshot();
        auto bytes = PackEnvelope<Snapshot>(Message_Snapshot, snap);
        CHECK(Verify(bytes));
        const Envelope* env = GetEnvelope(bytes.data());
        CHECK(env->message_type() == Message_Snapshot);
        std::unique_ptr<SnapshotT> back(env->message_as_Snapshot()->UnPack());
        CheckSnapshotEqual(*back, snap);
    }
    {
        auto defs = house::BuildTileDefs();
        auto bytes = PackEnvelope<TileDefs>(Message_TileDefs, defs);
        CHECK(Verify(bytes));
        const Envelope* env = GetEnvelope(bytes.data());
        std::unique_ptr<TileDefsT> back(env->message_as_TileDefs()->UnPack());
        CHECK(back->defs.size() == defs.defs.size());
        CHECK(back->connect_groups == defs.connect_groups);
        bool found_wall = false;
        for (auto& d : back->defs) {
            if (d->string_id == "t_wall") {
                found_wall = true;
                CHECK(d->fg_ref == 101);
                CHECK(d->bg_ref == -1);
                CHECK(d->connect_groups.size() == 1 && d->connect_groups[0] == 0);
            }
            if (d->string_id == "t_door_c") {
                CHECK(d->open_result == "t_door_o");
            }
            if (d->string_id == "t_floor") {
                CHECK(d->fg_ref == -1);
                CHECK(d->bg_ref == 100);
            }
        }
        CHECK(found_wall);
    }
    {
        auto hello = house::BuildHello();
        auto bytes = PackEnvelope<Hello>(Message_Hello, hello);
        CHECK(Verify(bytes));
        const Envelope* env = GetEnvelope(bytes.data());
        std::unique_ptr<HelloT> back(env->message_as_Hello()->UnPack());
        CHECK(back->protocol_version == hello.protocol_version);
        CHECK(back->tileset_id == "UltiCa");
        CHECK(back->z_min == -10 && back->z_max == 10);
    }
    {
        // Phase 10 (gate 10.1a, C++ half): EnvironmentUpdate round-trip.
        auto e = house::BuildEnvironment();
        auto bytes = PackEnvelope<EnvironmentUpdate>(Message_EnvironmentUpdate, e);
        CHECK(Verify(bytes));
        const Envelope* env = GetEnvelope(bytes.data());
        CHECK(env->message_type() == Message_EnvironmentUpdate);
        std::unique_ptr<EnvironmentUpdateT> back(env->message_as_EnvironmentUpdate()->UnPack());
        CheckEnvironmentEqual(*back, e);
    }
    std::printf("CPP ROUNDTRIP %s\n", g_failures ? "FAIL" : "PASS");
    return g_failures ? 1 : 0;
}
