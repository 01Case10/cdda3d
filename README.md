# cdda-voxel

A voxel-rendered frontend for Cataclysm: Dark Days Ahead. The game (a
lightly forked CDDA) runs all simulation; a separate Rust engine renders
the world as voxels and talks to the game over IPC. See `plan.md` for the
full project plan (Parts I + II) — **the plan is a living document and
must be updated after each phase** (see its maintenance rule).

## Status
- **Phase 0 (recon): complete at harness level, conditional gate pass** —
  see `docs/recon.md` for the exit answers, spike results, decisions,
  and the blocked build items (0.1/0.2) with their unblock path.
- **Phase 1 (protocol + skeleton): conditional gate pass 2026-11-15** —
  FlatBuffers schema, Rust `proto` crate, C++ stub server, live
  client↔stub round-trip; see `docs/protocol.md` and plan.md's Phase 1
  record. CI workflow exists; first GitHub run pending repo creation.
- **Phase 2 (engine MVP, fake data): conditional gate pass 2026-11-15** —
  mesher + engine crates, fake house, headless capture mode
  (`tools/capture.sh`). Pixel gates (2.10/2.13) still owe a run on a
  GPU machine — this VM has no `wgpu` adapter; see plan.md's Phase 2
  record.
- Phase 3 (game-side export) is next on the critical path, subject to
  Phase 0's condition: tests 0.1/0.2 must run on a build-capable
  machine first.

## Layout
```
plan.md            the single project document (plan + test plan)
docs/
  recon.md         Phase 0 deliverable: hook points, spikes, decisions
  hook-points.md   machine-extracted hook-point catalogue (file:line)
  licensing.md     Phase 0 licensing decision record (test 0.10)
  protocol.md      Phase 1 deliverable: protocol versioning rules
schema/            cdda_ipc.fbs — the IPC FlatBuffers schema (Phase 1)
crates/
  proto/           schema codegen, framing, sync, proto-client (Phase 1)
  mesher/          chunk data + naive/greedy meshers, no GPU deps (Phase 2)
  engine/          wgpu/winit engine + --headless-capture mode (Phase 2)
tools/
  phase0/          Phase 0 probes, spikes, and the fork probe patch
  stub-server/     C++ fake-snapshot server for protocol work (Phase 1)
  capture.sh       headless PNG capture wrapper (Phase 2)
fork-src/          pinned upstream checkouts (sparse; NOT the fork —
                   the fork itself is created in Phase 3 per plan §3)
assets/            generated output (gitignored)
```

## Quick checks
```
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --workspace                       # proto + mesher + engine gates
tools/stub-server/build.sh && tools/stub-server/cpp-roundtrip
tools/capture.sh --snapshot stub-house --camera 6,6,45,45 \
  --size 1280x720 --out captures/house.png   # needs a wgpu adapter
```

## Phase 0 re-run
```
tools/phase0/run_all.sh   # hooks + probes + UI spike, all headless
```
Pinned upstream commits: CDDA `6a3e59e08842de34ff3f111c7f098abeffb708a5`,
Tilesets `5ee942f9e6601970c5da356200e7cf5061226262`.
