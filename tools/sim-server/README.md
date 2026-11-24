# sim-server (Phase 11, slices S0 + S1)

A deterministic Rust test game for the CDDA Voxel Frontend: it
stands in for the forked CDDA over the Phase 1 IPC protocol so the
voxel engine can be live-tested before the fork runs (tests
0.1/0.2 are still blocked on a build-capable machine). Full plan:
`docs/test-sim-engine.md` in the repo.

The library lives in `crates/sim`; this crate is the thin binary
wrapper.

## Usage

```sh
cargo run -p sim-server -- /tmp/cdda-voxel.sock [--scene <name>] [--defs <path>]
# then, in another terminal:
cargo run -p proto --bin proto-client -- /tmp/cdda-voxel.sock
```

`sim-server <unix-socket-path>` binds the socket, accepts one
client, serves it until Shutdown/disconnect, then unlinks the
socket and exits cleanly. `--scene` picks a scene from
`crates/sim/scenes/` (default `stub_house`; also
`roofed_house_explicit`, `roofed_house_implicit`). `--defs`
points at the generated def table; resolution order is the flag,
then the `CDDA_SIM_DEFS` env var, then
`tools/sim-defs/generated/defs.json` relative to the CWD — the
error names everything tried.

## Wire behaviour (S0)

Identical framing and handshake to the C++ stub server
(`tools/stub-server`): u32 LE length + FlatBuffers `Envelope`,
verifier-checked decode, exact `PROTOCOL_VERSION` match or a
clean exit with both versions logged.

1. Server sends `Hello` (`proto::samples::sample_hello()`).
2. Client replies `HelloAck`; on version mismatch the server
   closes without sending anything further (no panic).
3. Server sends `TileDefs`, then `Snapshot` (the selected
   scene; seq 1, epoch 1, environment piggybacked). Since S1
   the TileDefs is the **full** S1a table (see below); in S0 it
   was the sample table.
4. Main loop: `Input` is echoed back verbatim; `Ping` is answered
   with `Pong` (same nonce/sent_ms); `Resize` and `PickResult`
   are logged to stderr only; `Shutdown` (or disconnect) exits
   cleanly.

## S1 — full def table, scenes, snapshot parity

- The def registry (`crates/sim/src/defs.rs`) loads the S1a
  generated table: **14,262 defs**, and the server sends all of
  them as the connect-time `TileDefs`, with the 40-entry
  connect-group registry.
- **Measured handshake cost** (this VM, debug build, parent
  re-run 2026-11-23; `cargo test -p sim --test measure_gate`):
  TileDefs frame **1,472,644 bytes** (~1.47 MB — the FlatBuffers
  encoding is far smaller than the 5.5 MB JSON source), registry
  load ~230 ms, connect → TileDefs received + decoded ~144 ms,
  scene Snapshot build ~0.11 ms. Option C's handshake cost is
  real but not a problem at these numbers.
- Scenes are TOML files in `crates/sim/scenes/`, parsed by a
  small hand parser for a documented subset (no `toml` crate in
  the workspace lock): `name/seed/time/weather` keys,
  `player = [x,y,z]`, per-z sections `[z0]`/`[z1]`/`[z-1]` with
  a `legend` inline table and `map` string rows, an optional
  **`furn` layer** per z (furniture chars, `.` = none — furniture
  is a separate wire array, so it is a separate layer, never
  guessed from the terrain char), plus `[[spawn]]`, `[[field]]`
  and `[[items]]` tables.
- Parity gates (`crates/sim/tests/parity_gate.rs`): the sim's
  stub_house / roofed_house_explicit / roofed_house_implicit
  Snapshots are semantically equal to the `proto::samples`
  fixtures — every id translated to its string id through its
  own table, then terrain/furniture/trap/fields/items/
  lit/seen/outside/entities/vehicles compared field by field.
- The piggybacked environment is still
  `proto::samples::sample_environment()` — real calendar/
  weather is S3.
- **Open S1 gate item:** engine capture parity (a capture of a
  sim-fed scene ≡ the same scene as a fixture, both renderers).
  Assessed 2026-11-23: feasible without engine source changes —
  the engine already ingests a `SnapshotT`
  (`snapshot_world::columns_from_snapshot`, `frame.rs`
  constructors) and has a headless GPU capture harness; the test
  would live in `crates/engine/tests/` with a dev-dependency on
  `sim` (or a live socket connect). Not done in S1; dispatch
  separately.

## Current scope — what this is NOT (yet)

- The world is **static**: the selected scene is sent once.
  There is no turn loop yet — Input does not change the world
  and no Deltas are sent. The stateful turn loop (movement,
  doors, diff-shaped Deltas) is slice **S2**.
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
