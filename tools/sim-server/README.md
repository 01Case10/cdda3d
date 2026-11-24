# sim-server (Phase 11, slices S0 + S1 + S2)

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
`roofed_house_explicit`, `roofed_house_implicit`, and since S2
`street_block`). `--defs`
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
4. Main loop (S2 semantics — see the S2 section below): a
   pressed Key `Input` is one game turn, answered by exactly one
   `Delta` (or a fresh `Snapshot` on a map shift); key releases
   and non-Key inputs get no reply. `Ping` is answered with
   `Pong` (same nonce/sent_ms); `Resize` and `PickResult` are
   logged to stderr only; `Shutdown` (or disconnect) exits
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

## S2 — turn loop: movement, doors, diff-shaped Deltas

- **Input semantics changed — the echo is gone.** In S0/S1 the
  server echoed every `Input` back verbatim. That was *stub*
  behaviour, kept only to mirror the C++ stub server. From S2
  the sim uses real-game semantics: **the real game never
  echoes input**. A pressed Key Input is one game action = one
  turn; after each turn the server sends exactly one `Delta`
  (even if nothing changed — seq still advances by exactly 1),
  except when the turn causes a map shift, where it sends a
  fresh `Snapshot` with epoch + 1 instead. Key releases and
  non-Key inputs are not turns and get no reply. Consequence,
  stated plainly: **`proto-client`'s echo step is a stub-era
  check and will no longer pass against sim-server** (it still
  passes against the C++ stub, which keeps the echo). The
  sim's own gates (`crates/sim/tests/server_gate.rs`,
  `turn_gate.rs`) assert the new semantics.
- **Keys** (key_code = the key's Unicode char): movement
  vi-keys `h`/`j`/`k`/`l` + diagonals `y`/`u`/`b`/`n`, WASD
  cardinals as aliases; `.` or ` ` waits; any other pressed key
  is a no-op turn. `o` / `c` arm a pending Open / Close — the
  next movement-direction key applies it to the adjacent tile
  in that direction instead of moving (CDDA's `o`/<dir>
  pattern); arming itself is not a turn and sends nothing, any
  non-direction key cancels the pending action, and a direction
  key that applies a pending action still consumes the turn.
- **Movement/collision:** a step succeeds iff the target
  terrain does not carry the WALL flag (the resolved
  `hot_flags` bit 25 = `ter_furn_flag::TFLAG_WALL` — the
  engine-facing bitset, never flag-string parsing), has no
  `open_result` (a closed door/window blocks; bumping does NOT
  auto-open — the step fails, the turn is still consumed), and
  no creature occupies the tile. Furniture does not block.
  Facing follows the step: LEFT if dx < 0, RIGHT if dx > 0.
- **Open/close** follow the terrain def's `open_result` /
  `close_result` through the registry (e.g. `t_door_c` →
  `t_door_o` → `t_door_c`). A no-op open/close (no transform)
  emits NO TileChange for that tile. A transform naming a def
  missing from the registry is logged and treated as a no-op,
  never a panic.
- **Delta generation is diff-shaped, like the real exporter:**
  mutations mark their submap dirty; at flush, each dirty
  submap is diffed tile-by-tile against the last-sent state
  and only changed tiles emit a `TileChange` (all wire fields;
  lit/seen/outside are carried from stored tile state —
  recomputing them is S3). Entities are diffed by id into
  `entity_updates` / `entity_removed`. **Map shift:** if the
  player's submap (x/12, y/12, z) differs from the one they
  were in at the last Snapshot, the flush is a fresh Snapshot
  (epoch + 1, seq + 1).
- **Scene-extent simplification (documented):** a Snapshot
  contains ALL of the scene's submaps — the scene extent is
  the bubble; there is no view-window culling yet. Scenes may
  now span multiple submaps per z (map rows/width in multiples
  of 12; `street_block` is 24x24 = 2x2).
- The turn loop is also drivable **in-process** —
  `World::apply_input(&InputT) -> Option<MessageT>` — which is
  what the S2 gates use and what S7's scenario runner will use.
- Gates (`crates/sim/tests/turn_gate.rs`): **T1** (world/wire
  half) on the new `street_block` scene — bump the closed door,
  open it, step out, cross the x=12 submap boundary (fresh
  Snapshot, epoch 2), walk back in (epoch 3), close the door;
  **T5** the full door cycle including no-op open/close; and a
  **200-turn mirror invariant**: a mirror fed every Delta
  equals a fresh Snapshot after every turn.

## Current scope — what this is NOT (yet)

- No visibility or environment simulation: lit/seen/outside
  are the S1 stored values, carried through Deltas unchanged
  (recompute is slice **S3**). Creatures do not move on their
  own yet (S5); there is no bash/destruction (S6), no items
  pickup (S6), no vehicles moving.
- No UI shared memory / `UiFrame` yet (slice S4).

## Deviation from the C++ stub

The C++ stub sends one standalone `EnvironmentUpdate` after the
Snapshot. sim-server deliberately does **not** (yet): in S0/S1
the reason was that `proto-client` expected the Input echo as
the next message after the Snapshot; since S2 the echo itself
is gone (see the S2 section), and the point is simply that the
Snapshot already piggybacks the current environment, so
nothing is lost. Standalone EnvironmentUpdates arrive with
the S3 environment slice, sent on change and on the ratified
10-game-minute time-only cadence.
