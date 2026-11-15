# src/voxel_ipc — Phase 3 game-side exporter (fork module)

Layout and mapping into a real CDDA checkout: docs/phase3-fork.md.
Core (voxel_types/adapter/exporter/server/input/frontend) is
game-API-agnostic and is exercised by tools/fork-harness; only
voxel_real_adapter.cpp includes game headers and compiles solely in the
fork build. cdda_ipc_generated.h is checked-in flatc 24.3.25 output for
schema/cdda_ipc.fbs (regenerate on schema change, per docs/protocol.md).
