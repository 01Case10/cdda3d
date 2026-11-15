# Licensing (Phase 0, test 0.10 — recorded 2026-11-15)

Generated from the `LICENSE.txt` files of the two pinned repos (CDDA
`6a3e59e0`, Tilesets `5ee942f9`) by `tools/phase0/probes.py` (test 0.10)
plus the manual extraction below. **This file is the Phase 0 licensing
decision record.**

## Base licenses
- **CDDA** (`LICENSE.txt`, 174 lines): Creative Commons
  Attribution-ShareAlike 3.0 Unported (CC BY-SA 3.0).
- **CDDA-Tilesets** (`LICENSE.txt`, 45 lines): CC BY-SA 3.0 Unported.

## Third-party notices that must travel with the fork's attribution file
From CDDA `LICENSE.txt`:
- GNU Unifont — SIL OFL 1.1 / GPL v2+ with font embedding exception
- Terminus Font, Vecterminus Font — SIL OFL (LICENSE-OFL-Terminus-Font.txt)
- Roboto font — Apache License 2.0 (LICENSE-Apache-Roboto-Font.txt)
- Catch unit-test framework (`tests/catch/catch.hpp`) — Boost Software License
- PLF List / PLF Colony (`src/list.h`, `src/colony.h`) — zlib license
- getpost (`tools/json_tools/format/getpost.h`) — MIT
- {fmt} — MIT-style (Copyright Victor Zverovich and contributors)
- libbacktrace — BSD
- Microsoft / Meta components — MIT / BSD respectively (see LICENSE.txt)
- JCash/voronoi — MIT (Copyright Mathias Westerdahl)

From Tilesets `LICENSE.txt`:
- The Square font — CC BY 3.0 (Wouter van Oortmerssen)
- GNU Unifont — GPL v2+ with font embedding exception
- Terminus Font — SIL OFL
- Catch — Boost Software License

## Decisions (Phase 0)
1. **The fork (`cdda-fork`) remains CC BY-SA 3.0**, and its attribution
   file reproduces *all* notices above, not just the CC BY-SA header.
2. **Textures/sprites derived from tileset art are derivative works**:
   CC BY-SA 3.0 with attribution to the tileset authors. The voxelizer's
   generated `ATTRIBUTION.md` (Phase 5) must carry the per-sheet `source`
   entries from UltiCa's `tile_info.json` plus this notice list.
3. **The Rust engine is licensed separately under MIT OR Apache-2.0**
   (dual), because it contains no CDDA code or assets. Generated asset
   pipeline *output* is share-alike per (2) regardless of the engine's
   license. (Plan §14 open question — distribution bundling — stays open;
   if engine and fork ship as one bundle, the bundle as a whole must
   satisfy CC BY-SA for the fork+assets parts.)
4. **Mushroom Dream is excluded.** Verified in the Tilesets `readme.md`
   at `5ee942f9`: only a small stub hosted in the repo is CC BY-SA; the
   complete version is under a more restrictive license via the author's
   Patreon. The voxelizer must not consume `gfx/Mushroom-Dream` beyond
   that stub, and v1 ships UltiCa only (see recon §Decisions).
