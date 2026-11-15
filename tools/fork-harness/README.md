# Fork harness — Phase 3 gate evidence (standalone)

Builds the REAL exporter core (`fork/src/voxel_ipc/voxel_exporter.cpp`,
`voxel_server.cpp`, `voxel_input.cpp`, `voxel_frontend.cpp`) against a
mock game-state layer (`mock_world.*`: shelter, house with doors/windows,
road+field, multi-storey+basement, vehicle on a road) implementing the
same `IGameAdapter` interface as the fork's real adapter, and runs the
Part II §6 gate checks that do not require the compiled game:
3.1 (harness form), 3.3, 3.4, 3.5, 3.5b, 3.6, 3.7, 3.8, 3.9 (1,100 turns,
10 seeds), 3.9b, 3.10, 3.11, 3.12, 3.14, 3.17, 3.18 (measured), plus a
schema round-trip and input-order check. It then decodes the produced
snapshot with the real Rust `proto` crate (`rust-decode/`).

Run: `./run.sh` — exit 0 iff every check passes.

What this does NOT prove: anything in-game. Tests 3.1 (full registry),
3.2, 3.13, 3.15, 3.16, 3.19, 3.20 and the in-game forms of the rest wait
for tests 0.1/0.2 on a build-capable machine (see docs/phase3-fork.md §5).
