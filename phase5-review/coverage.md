# Phase 5 — real UltiCa run summary (2026-11-15)

Command: `voxelizer --tileset fork-src/tilesets/gfx/UltimateCataclysm --out assets/ultica_run1` (release build)

- Sparse checkout expanded on demand: 16,932 files on disk = git index count; 14,451 PNGs fetched.
- Materials: 7,717 ids indexed by the loader (from 4,650 decomposed entry objects; multi-id entries share sprites), 26,389 texture layers, 14,450 sprites in atlas table, 0 missing within this id universe.
- Id universe measured: the decomposed tileset entry ids themselves. Game JSON ids / TileDefs do NOT exist yet (Phase 3 in parallel; CDDA sparse checkout has no data/json), so test 5.10's ">=98% of base-game terrain+furniture ids" is measured here against (a) tileset entry ids: 100% exact, and (b) sprite references: 9,845 unique fg/bg/subtile sprite names, 100.00% have a PNG on disk. The TileDefs-based form of 5.10 remains open until Phase 3 lands.
- Determinism (5.2): second run to assets/ultica_run2, `diff -r` byte-identical.
- Performance (5.15): 10.8 s / 9.4 s voxelizer time (14.5 s / 10.0 s wall incl. process start) on this 1.5-CPU VM — budget <60 s. No compose.py/pyvips step in the pipeline (decomposed form consumed directly).
- Contact sheet: ultica-contact.png in this folder (copy of assets/ultica_run1/contact.png).
