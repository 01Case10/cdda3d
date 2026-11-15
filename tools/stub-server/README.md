# Stub IPC server (Phase 1)

A standalone C++ stand-in for the fork's future `src/voxel_ipc` server, so
the Rust protocol crate and client can be developed and tested without
building CDDA.

- `stub_server.cpp` — listens on a Unix socket (path = argv[1]), handshakes,
  sends TileDefs + a fake procedural-house Snapshot, echoes Input, answers
  Ping with Pong, exits cleanly on Shutdown.
- `cpp_roundtrip.cpp` — pure C++ round-trip checks (gate 1.2) and the
  golden-file tool for cross-language tests (gate 1.3):
  `cpp-roundtrip write <dir>` / `cpp-roundtrip verify <snapshot-file>`.
- `house.h` — the fake house + tile defs. Must stay semantically in sync
  with `crates/proto/src/samples.rs`; the cross-language tests compare them.
- `cdda_ipc_generated.h` — flatc 24.3.25 output (`--cpp --gen-object-api`)
  from `schema/cdda_ipc.fbs`; checked in, CI fails if it drifts.
- `third_party/flatbuffers/` — vendored flatbuffers 24.3.25 headers.

Build: `./build.sh` (g++ only, no other dependencies).
