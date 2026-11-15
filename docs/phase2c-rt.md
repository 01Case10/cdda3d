# Phase 2c — Ray-traced renderer on Sparse-Voxel-64Tree

Status: **implemented, conditional gate pass 2026-11-15** (condition:
gate 2c.9 frame time on the RX 9070 XT). Implemented directly in the
main session after the two earlier subagent dispatches were refused
by a safety policy before any work (see plan.md Progress log).

## R0 spike report — which arm won

**Primary arm won.** `Tree64GpuManager`'s bind group/layout is used
directly as group 0 of both compute pipelines (render + ray query);
no fallback to self-managed upload of the flattened bytes was
needed. Gate 2c.10 asserts the manager's node buffer is exactly
`node_count × 32` bytes and the R0 smoke render (fake house,
320×180, >1/10 of pixels closer than max distance) runs through
that bind group. The crate's wgpu `^24.0.1` resolves cleanly
against the workspace's pinned 24.0.5.

One integration fact the crate does not document: `Node64`'s fields
are private, so the tree cannot be walked on CPU — the only
observable structure is `flatten()`'s byte output. The side-table
design below is built around that.

## Design

### Crates and modules

- `crates/treebuild` (new): the whole RT core, engine-independent.
  - `world.rs` — `VoxelWorld`: a sparse voxel map
    (`[i32;3] -> VoxelCell{color, alpha, material_id, lit, flags}`)
    plus `from_chunk_map` (adapts the mesher's `ChunkMap`; material
    id = mesher tile id, `lit` defaults to `LIT`, flags from the
    block's pass) and `subset_box` (extract a sub-region world, the
    unit of the per-column rebuild in gate 2c.5).
  - `build.rs` — `build_tree`: inserts the world into a
    `Sparse64Tree` (deterministic sorted order, colors quantized
    round-to-nearest to u8), flattens it, and **reconstructs the
    material side-table by walking the flat array breadth-first**,
    deriving each node's bounds from its position in the BFS order
    and verifying every leaf's flat color against the world voxel
    at the reconstructed coordinate — an independent ground truth.
    Any mismatch fails the build (`BuildError::SideTableMismatch`);
    gate 2c.2 additionally proves the check catches a shuffled
    side-table and a wrong-world reconstruction. Side entries are
    16-byte records `{material_id, lit, flags, alpha}` indexed by
    flattened node index, `EMPTY` for internal nodes. Tree depth
    defaults to the smallest d ≤ 6 covering the world extent
    (depth 4 = 256/axis covers the 132×132 × ≤63-block stored
    world).
  - `cpu.rs` — the CPU oracle: a traversal that mirrors the WGSL
    line-for-line (see below), plus `dda_reference`, an
    Amanatides–Woo voxel DDA over `VoxelWorld` kept as the
    mesher-era comparison for picking parity (2c.7).
  - `render.rs` — the WGSL traversal + two compute pipelines
    sharing group 0 (the crate manager's bind group): `render_main`
    (primary rays, shading, one transparent layer) and `query_main`
    (caller-supplied rays → hit records, the GPU picking path).
    `RtRenderer::with_scene` builds renderer + scene (manager,
    side-table buffer) from a `BuiltTree`.
- `crates/engine/src/rt.rs` — engine integration: builds the RT
  world from the **same cutaway-filtered chunk map** as the raster
  path (`world::filtered_map`, extracted from `mesh_world` so both
  renderers share it by construction), the capture path
  (`--renderer rt` in `--headless-capture`), and the windowed blit
  (fullscreen triangle sampling the RT color texture).
  `--renderer raster|rt` is parsed in both headless and windowed
  modes; **raster remains the default** (decision below).

### Traversal

One algorithm in both languages: descend the tree at the ray's
current point to either an occupied leaf (hit) or the largest empty
child cell containing the point, then advance the ray to that
cell's exit plane and repeat. Empty-space skipping comes from the
tree's sparsity rather than a step-per-voxel march, and because the
Rust and WGSL implementations are structurally identical, gate
2c.3's parity is exact on voxel/normal/node and <1e-3 on distance.
Two modes: `FirstHit` (rendering; stops at glass) and `OpaqueOnly`
(shadow rays; transparent/cutout voxels don't block light).

### Shading

Per primary hit: flat albedo from the tree node's color × game
`lit_level` factor (DARK .25 / LOW .45 / BRIGHT_ONLY .65 / LIT .85 /
BRIGHT 1.0 / MEMORIZED .5, memorized additionally desaturated toward
luminance) + sun directional light with a shadow ray (OpaqueOnly) +
occupancy AO (four tangent-neighbor occupancy queries, −0.10 each).
Emissive-flagged voxels shade unlit ×1.6 (they feed 2b's bloom once
fx is wired in). One transparent layer: a `FirstHit` transparent
voxel is blended over the opaque hit behind it by its side-table
alpha. Misses shade a sky gradient whose horizon is the raster
path's clear color.

### Visibility discipline (Phase 7 contract)

`BLANK` (never-seen) cells are **excluded at build time** — counted
in `excluded_never_seen`, absent from the tree, so neither primary
nor shadow rays can reveal them (gate 2c.8 proves a shadow ray
passes through a BLANK voxel's cell and that a MEMORIZED voxel is
present, flagged, and does block shadow). The game keeps visibility
truth; the tracer only lights what it is given.

### Cutaway

A build-time filter, not a traversal clip: the engine builds the RT
world from `filtered_map` with the same visible-levels selection as
the raster capture/windowed paths, so layer selection is identical
by construction (gate 2c.6: tree leaf difference between cutaway
z=0 and z=1 is exactly the 144-block roof layer; z=−10 yields an
empty root-only tree; z=+10 the full house).

## Findings worth keeping

- **llvmpipe device loss from dynamically-indexed local arrays.**
  The first WGSL traversal compiled clean (naga reported no errors)
  but `create_compute_pipeline` lost the device on lavapipe.
  Bisection isolated it to `array<f32,3>` locals indexed by a loop
  variable (in the slab test / entry-normal helpers): naga's SPIR-V
  output for dynamic local-array indexing kills llvmpipe at
  pipeline creation. Rewriting both helpers vectorized
  (component-wise min/max) fixed it; both pipelines now create and
  run on lavapipe. If a future shader loses the device on llvmpipe
  with clean compilation, suspect dynamic indexing of local arrays
  first.
- **RT captures are linear-valued.** The raster capture renders to
  an sRGB surface format (values encoded on write); the RT color
  texture is a plain storage texture, so `house_rt.png` holds the
  raw shading values and reads darker than `house.png` for the same
  scene. The windowed blit compensates with an approximate inverse
  transfer (`pow 2.2`). A future visual-diff harness should compare
  RT captures against RT captures, or encode on readback first.
- **Texture array not yet sampled.** The shader shades from the
  flat node color + side-table (material/lit/flags/alpha). The
  Phase 5 texture array is not bound because the engine has no GPU
  texture array yet — the identical open follow-up as the raster
  path (engine `materials.rs` loads the materials JSON only). The
  side-table already carries `material_id` per leaf, so binding the
  array later is a shader change, not a data change.
  **Closed the same day** — see "Texture hookup" below.

## Gate evidence (parent re-run, 2026-11-15)

All in `crates/treebuild/tests/gate_tests.rs` and
`crates/engine/tests/rt_gate.rs`; full workspace suite 91/91.

| Gate | Result |
|---|---|
| 2c.1 | ✅ house + 132×132×3-level bubble exhaustive point queries; node-count bounds; proptest random worlds; proptest tree-vs-DDA ray parity (both modes, voxel+normal exact, dist <1e-3) |
| 2c.2 | ✅ BFS-reconstruction side-table verified against world ground truth on every build; negative cases (shuffled table, wrong world) fail as designed |
| 2c.3 | ✅ GPU vs CPU oracle on lavapipe: 16-ray orbit ring + axis rays, both trace modes — hit/miss, voxel, normal, node exact; dist <1e-3 |
| 2c.4 | ✅ silhouette IoU **1.000** (rt-only 0 px, raster-only 0 px at 320×180); mean \|color diff\| on shared geometry 70.2/255, itemized as the lighting delta (sun shadow + AO + lit factor; plus the linear-vs-sRGB capture encoding above). Captures: `captures/house.png` vs `captures/house_rt.png` |
| 2c.5 | ⚠️ measured, budgets are target-hardware: full stored-world build (27,588 voxels, 29,736 nodes, depth 4) 91.7 ms release on this 1.5-CPU VM (budget 20 ms on target hardware); single-tree delta = full rebuild 88.4 ms (misses the 2 ms delta budget, as the plan anticipated); **dirty-column rebuild 0.110 ms** (229 voxels) — the per-column path is the recorded delta mechanism |
| 2c.6 | ✅ data-level (leaf counts, extremes z=−10/+10) + render-level (cutaway capture loses roof silhouette; raster agrees on direction) |
| 2c.7 | ✅ CPU picking parity vs DDA reference: 24 orbit rays, ≥12 hits, tile coordinates equal |
| 2c.8 | ✅ BLANK excluded and shadow-transparent; MEMORIZED present/flagged/shadow-blocking; EMISSIVE flag reaches the side-table |
| 2c.9 | ⛔ target hardware only — RX 9070 XT, 1440p bubble scene; not run |
| 2c.10 | ✅ primary arm: manager bind group used directly; buffer size == nodes×32 asserted; R0 render smoke passes |

## Default-renderer decision

**Raster stays the default; RT is opt-in via `--renderer rt`.**
The 2c.4 correctness evidence is complete (IoU 1.000), but the
decision rule in the plan weighs 2c.9 frame time, which does not
exist yet. Revisit when Joey runs the RT capture/frame timing on
the RX 9070 XT: `tools/capture.sh --renderer rt` on the bubble
scene at 2560×1440, median frame time under 16 ms flips the
default.

## Texture hookup (Phase 5 follow-up, closed 2026-11-15)

Added on Joey's word after the 2c gate, RT path only:

- `TextureSpec`/`TextureSet` in treebuild's render module: a
  scene-local texture array plus a per-material `(top, side,
  bottom, present)` layer lookup buffer, bound at group 1 bindings
  4–6. Scenes without textures bind a 1×1 dummy set, so the
  pipeline layout is unconditional and untextured materials shade
  flat exactly as before.
- The render shader samples by face class from the hit normal
  (+z top, −z bottom, else side) with in-voxel UVs; side faces map
  +z to v=0 so textures stand upright. Texels are sampled raw
  (Rgba8Unorm), matching the node-color convention. Cutout
  materials with a mostly-transparent texel fall back to flat
  color (traversal already decided the voxel is solid). No mipmaps
  yet — nearest sampling; distant faces will alias until a mip
  chain lands.
- Engine side (`crates/engine/src/textures.rs`): builds the spec
  from a voxelizer output dir via `MaterialTable`, decoding only
  the layers the scene's materials reference — the fake house
  uploads **27 layers** out of UltiCa's 26,389. Numeric→string
  material resolution is the caller's: fake data uses
  `rt::fake_material_ids` (mesher fake tile id → verified UltiCa
  id, e.g. wall → `t_wall`, roof → `t_shingle_flat_roof`); real
  scenes will pass TileDefs once Phase 4 wires snapshots in.
- CLI: `--textures <voxelizer-dir>` on headless capture and the
  windowed app (RT mode). Evidence: treebuild checkerboard gate
  (textured voxel shows both texel colors, flat control stays
  near-uniform) + engine gate against the real UltiCa output
  (9/9 fake materials textured, 0 missing; depth bit-identical to
  the flat render; mean |color diff| 45.3/255 on geometry;
  distinct colors 38 → 232 at 320×180). Captures:
  `captures/house_rt.png` (flat) vs `captures/house_rt_textured.png`.
- Not covered: multitile/subtile layer selection (walls use their
  base side layer regardless of neighbors — needs connectivity at
  build time), the raster path (still needs the mesher
  UV/texture-layer field), mipmapping. **All three were scheduled
  as plan phases on 2026-11-15: 2d (multitile), 2e (mipmaps),
  2f (raster-path textures).**

## Follow-ups (not 2c scope)

- 2c.9 on the RX 9070 XT (above).
- Texture mipmaps in the RT shader; multitile layer selection;
  raster-path textures (mesher UV follow-up, still open).
- fx (Phase 2b) wiring into the engine renderer: the post stack
  composites over whichever renderer produced the frame; RT frames
  currently bypass it.
- Per-column trees as the live delta path: the mechanism is
  measured (2c.5) but the engine still rebuilds the single tree per
  cutaway change; wire dirty-column rebuilds when real Snapshot/
  Delta data (Phase 3/4) drives updates.
- Windowed RT has never run live on this VM (same status as the
  windowed app generally); the headless capture path is the
  verified surface.
