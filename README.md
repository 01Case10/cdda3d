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
- Phase 1 (protocol + skeleton) is next; it does not depend on the
  blocked items.

## Layout
```
plan.md            the single project document (plan + test plan)
docs/
  recon.md         Phase 0 deliverable: hook points, spikes, decisions
  hook-points.md   machine-extracted hook-point catalogue (file:line)
  licensing.md     Phase 0 licensing decision record (test 0.10)
tools/phase0/      Phase 0 probes, spikes, and the fork probe patch
fork-src/          pinned upstream checkouts (sparse; NOT the fork —
                   the fork itself is created in Phase 1/3 per plan §3)
crates/            Rust workspace (Phase 1)
assets/            generated output (gitignored)
```

## Phase 0 re-run
```
tools/phase0/run_all.sh   # hooks + probes + UI spike, all headless
```
Pinned upstream commits: CDDA `6a3e59e08842de34ff3f111c7f098abeffb708a5`,
Tilesets `5ee942f9e6601970c5da356200e7cf5061226262`.
