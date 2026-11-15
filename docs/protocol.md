# IPC protocol — versioning and wire rules

Phase 1 deliverable. The schema is `schema/cdda_ipc.fbs`; the Rust
implementation is `crates/proto`; the reference C++ peer for tests is
`tools/stub-server`. Design rationale: plan.md §6.

## Wire format

- Transport: local stream socket (Unix domain socket on Linux; named pipe
  on Windows later). The game is the server, the engine is the client and
  spawns the game with `--frontend=ipc --ipc-path=<path>`.
- One frame = **u32 little-endian length prefix + one FlatBuffers
  `Envelope` payload**. The length prefix is not part of the FlatBuffers
  buffer.
- Max frame payload: **32 MiB** (`MAX_FRAME_LEN` in
  `crates/proto/src/framing.rs`, `kMaxFrame` in the stub). A length prefix
  above the cap is rejected from the prefix alone, before any allocation of
  that size, and the connection is closed.
- Every payload carries the FlatBuffers file identifier `CDDA`. Decoders
  must run the FlatBuffers verifier before any table access, and the Rust
  decoder additionally checks the identifier bytes and rejects an Envelope
  whose union is `NONE` (the stock verifier does neither — found by the
  gate 1.4 bit-flip sweep).
- The UI texture does **not** travel on the socket. It uses shared memory
  with the `UiShmHeader` layout defined in `crates/proto/src/shm.rs`
  (48 bytes, magic `0x43444155`); the socket only carries `UiFrame`
  notifications. The C++ side static_asserts the same layout.

## Versioning rules

- `PROTOCOL_VERSION` is a single integer, currently **1**. It appears in
  `Hello.protocol_version`, `HelloAck.protocol_version`, and
  `TileDefs.protocol_version`, and as a constant in both implementations
  (`proto::PROTOCOL_VERSION`, `house::kProtocolVersion` in the stub; the
  fork will carry its own copy in `src/voxel_ipc/`).
- **Handshake:** versions must match exactly. A mismatch is a clear error
  (`ProtoError::VersionMismatch` in Rust; the stub logs both versions) and
  a clean exit on both sides — never a silent proceed with a guessed dialect.
- **Breaking changes bump `PROTOCOL_VERSION`:** removing or renaming a
  field/table/union member, changing a field's type or id, changing an
  enum's values, changing the framing, the max frame size, or the
  shared-memory header layout.
- **Non-breaking changes do not bump it:** adding a field to a table,
  adding a member to the `Message` union (at the end), adding an enum
  value, adding a new table. Old readers ignore unknown fields by
  FlatBuffers construction (proven by gate test 1.9, which encodes with a
  v2 schema carrying an extra `Hello` field and decodes with the v1
  reader). Senders must therefore treat every field as omittable by peers
  and every enum as open-ended: an unknown enum value must be handled, not
  assumed impossible.
- **Absent vs empty:** for optional result/string fields (`open_result`,
  `looks_like`, ...), "none" is sent as an *absent* field, not an empty
  string. The C++ object API packs `""` as absent, so the two are
  indistinguishable on the wire for C++ peers; receivers must treat absent
  and empty identically.
- **Sync:** state syncs by monotonic `seq` + `epoch`, never by game turn.
  `epoch` increments on every fresh `Snapshot`. A delta with a skipped
  `seq` means the receiver requests a fresh snapshot; a duplicate `seq` is
  discarded (applied zero times); a delta from an old `epoch` is
  discarded. The reference implementation is `SeqTracker` in
  `crates/proto/src/sync.rs` (gate test 1.14).
- **Heartbeat:** either side may `Ping` at any time; the peer answers
  `Pong` with the same nonce. Expected cadence is 1 s; a peer that sends
  nothing for 3 s is considered dead and the other side shuts down
  (`HeartbeatTracker`, gate test 1.10).
- **Shutdown:** either side may send `Shutdown`; the receiver exits
  cleanly (target: within 2 s, gate test 1.11). A dropped connection is
  treated the same as a `Shutdown` with no reason.

## Regenerating code

- Rust: `crates/proto/build.rs` runs flatc 24.3.25 at build time
  (`$FLATC`, else the project copy at `~/workspace/tools/flatbuffers/flatc`,
  else `/tmp/flatc`, else `$PATH`) on `schema/cdda_ipc.fbs` with
  `--rust --gen-object-api`. Nothing generated is checked in on the Rust
  side.
- C++: the generated header **is** checked in at
  `tools/stub-server/cdda_ipc_generated.h` (and the fork will do the same),
  produced with `flatc --cpp --gen-object-api`. CI regenerates it and
  fails on diff, so schema changes must regenerate and check it in.
- Golden files live in `crates/proto/tests/golden/` (written by the C++
  tool: `tools/stub-server/cpp-roundtrip write <dir>`). Regenerate them
  after any intentional schema or sample-house change; CI's cross-language
  tests regenerate and compare live, so stale goldens fail loudly.
