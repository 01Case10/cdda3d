# sim-server (Phase 11, slice S0)

A deterministic Rust test game for the CDDA Voxel Frontend: it
stands in for the forked CDDA over the Phase 1 IPC protocol so the
voxel engine can be live-tested before the fork runs (tests
0.1/0.2 are still blocked on a build-capable machine). Full plan:
`docs/test-sim-engine.md` in the repo.

The library lives in `crates/sim`; this crate is the thin binary
wrapper.

## Usage

```sh
cargo run -p sim-server -- /tmp/cdda-voxel.sock
# then, in another terminal:
cargo run -p proto --bin proto-client -- /tmp/cdda-voxel.sock
```

`sim-server <unix-socket-path>` binds the socket, accepts one
client, serves it until Shutdown/disconnect, then unlinks the
socket and exits cleanly.

## Wire behaviour (S0)

Identical framing and handshake to the C++ stub server
(`tools/stub-server`): u32 LE length + FlatBuffers `Envelope`,
verifier-checked decode, exact `PROTOCOL_VERSION` match or a
clean exit with both versions logged.

1. Server sends `Hello` (`proto::samples::sample_hello()`).
2. Client replies `HelloAck`; on version mismatch the server
   closes without sending anything further (no panic).
3. Server sends `TileDefs` (sample table), then `Snapshot`
   (the sample house; seq 1, epoch 1, environment piggybacked).
4. Main loop: `Input` is echoed back verbatim; `Ping` is answered
   with `Pong` (same nonce/sent_ms); `Resize` and `PickResult`
   are logged to stderr only; `Shutdown` (or disconnect) exits
   cleanly.

## Current scope — what S0 is NOT

- The world is **static**: the sample house from
  `proto::samples`, sent once. There is no turn loop yet —
  Input does not change the world and no Deltas are sent.
  The stateful turn loop (movement, doors, diff-shaped Deltas)
  is slice **S2**.
- The def table is the small **sample** TileDefs, not the full
  real def universe (Joey's Option C). The full importer is
  slice **S1a** and the full table lands with **S1**.
- No UI shared memory / `UiFrame` yet (slice S4), no entities
  moving, no environment changes.

## Deviation from the C++ stub

The C++ stub sends one standalone `EnvironmentUpdate` after the
Snapshot. sim-server deliberately does **not** (yet): the
existing `proto-client` expects the Input echo as the next
message after the Snapshot and fails on an intervening
`EnvironmentUpdate`. The Snapshot already piggybacks the current
environment, so nothing is lost. Standalone EnvironmentUpdates
arrive with the S3 environment slice, sent on change and on the
ratified 10-game-minute time-only cadence.
