# Phase 2d — Multitile layer selection (connected walls)

Status: **implemented, gate passed 2026-11-15** (all four gates run
on this VM, GPU halves on lavapipe). Implemented directly in the
main session — the earlier parallel-subagent dispatches for
2d/2e/2f were refused by a safety policy before any work, the same
pattern Phase 2c hit; 2e/2f were being implemented concurrently in
another session while this landed, so this phase's commit is
scoped to the 2d files only.

## Problem

The Phase 2c texture hookup always sampled a material's base side
layer, ignoring neighbors: every wall block showed the standalone
wall picture, so runs, corners and junctions never visually
connected. The game's tileset picks a wall's sprite from its four
cardinal neighbors (straight run, corner, T, cross, end cap,
isolated are different pictures). Phase 5's voxelizer already
emits the data — per-material `subtile_layers` (subtile name ->
layer index) — but nothing selected from it.

## Design

### One canonical mask table

`mesher::connect` now holds the game's mask -> subtile rule
(N=1, E=2, S=4, W=8; `subtile_for_mask`, `mask_from_connects`,
`all_mask_mappings`) — the table Phase 5 verified against
`cata_tiles::get_rotation_and_subtile`. The mesher crate is the
workspace's leaf, so every consumer can share it:
`voxelizer::multitile` delegates to it (its connect-*group* helper
`neighbors_connect` stays in the voxelizer, where game ids and
group indices live), and a mesher gate test pins the shared table
against the same hand-computed expectations as gate 5.5.

### Masks over the voxel world

`treebuild::connect::connection_mask(world, pos, connects)`
computes a voxel's cardinal mask in the XY plane at its own z
(north = -y), under a caller-supplied connection predicate. The
fake-data predicate is `same_material` (the plan's stated fake
rule). Real scenes will pass a connect-group predicate built from
Phase 3's TileDefs export — the game's own `connects_to` rule —
through the same seam; nothing here changes when they do. A
full-height wall stack gets one mask at every block, because its
neighbors are full-height too.

### Selection: virtual materials (deviation from the plan's expected shape)

The plan expected "a per-node layer override in (or alongside)
the side-table; the shader prefers the override when present".
What shipped instead: **virtual materials**.

`treebuild::connect::assign_variants` computes every multitile
voxel's mask *before* touching any id (rewriting first would
corrupt the same-material test), allocates one fresh material id
per distinct `(base, mask)` pair that actually occurs — sorted,
numbered from `max_id + 1`, so the same world always produces the
same assignment — and rewrites those voxels' `material_id` to the
virtual id. The engine's spec builder
(`textures::build_texture_spec_multitile`, sharing the base
builder's layer loading/compaction) emits a `mat_layers` entry at
each virtual id: base top, base bottom, and the side layer chosen
by the one shared selection rule,
`Material::side_layer_for_mask` (multitile + subtile present ->
that subtile's voxelizer layer; otherwise the base side layer).

Why this shape over the side-table override:

- The RT shader, the side-table format, the CPU oracle and all of
  2c's parity machinery are untouched — the shader already
  resolves layers through `mat_layers[material_id]`. Gate 2d.4's
  IoU parity is therefore structural, not re-earned.
- The same virtual ids feed the raster path in Phase 2f (its
  per-block `tex_layers` stamping reads the same spec entries),
  so both renderers select identically by construction — the
  property gate 2f.3 asserts for the base builder.
- Cost is bounded by what occurs: at most 16 variants per
  multitile material *present in the scene*, and only masks that
  actually occur get an entry.

`engine::materials::Material` now parses `multitile` +
`subtile_layers` (previously dropped) and validates subtile layer
indices against the layer count like the base layers.

### Wiring

RT only (the raster path is Phase 2f), in both RT entry points:

- Capture: `rt::capture_rt_with_device` builds the world, and
  when `--textures` is set calls `rt::prepare_fake_variants`
  (loads the table, derives the multitile numeric ids through
  `fake_material_ids`, runs `assign_variants`) *before*
  `build_tree`, then attaches the multitile spec. Without
  `--textures` the world is untouched.
- Windowed: `App::remesh` runs the same prepare -> build ->
  attach sequence when a textures dir is configured.

### Scope held

- Side faces only; top/bottom never vary. Rotation variants are
  not applied (the mapping's rotation half is computed and
  test-pinned but unused) — Phase 8 polish, per the plan.
- Fake-data connection semantics only (same material). Connect
  groups arrive with TileDefs-driven scenes.

## Gate evidence (re-run in the implementing session)

| Gate | Result |
|---|---|
| 2d.1 | ✅ `treebuild/tests/multitile_gate.rs`: six layouts (isolated, E-W run, N-S run, corner, T, cross) — every voxel's mask equals the hand-computed mask (19 positions); negative cases (different material, diagonal, different z) don't connect; a 3-high stack shares one mask; custom predicates honored. Assignment test: deterministic virtual ids, side-table carries them (`verify_side_table` + leaf spot-check) |
| 2d.2 | ✅ `engine/tests/multitile_gate.rs`: all 16 masks resolve to the fixture's subtile layers, 100%; non-multitile -> base for all masks; missing subtile -> per-mask fallback. Real UltiCa: `t_wall` (all 6 subtiles) — each variant entry's side layer is byte-identical to the voxelizer's own layer file for that mask's global layer; top/bottom equal base. (Note: t_wall's *unconnected* picture is the same sprite its base side was derived from — same pixels, different layer index; index fidelity is the criterion, not pixel difference.) Spec-builder unit test pins compact remapping incl. fallback |
| 2d.3 | ✅ adapter-conditional, ran on lavapipe: fixture with one solid hue per subtile layer; face-on renders of isolated / E-W run / cross / T scenes — the center pixel (first wall face along the view ray) is the variant hue: blue unconnected, green edge, cyan end_piece (the cross's north arm — the mask-15 center voxel is fully occluded from outside by construction, so the *center* subtile is pinned at selection level by 2d.1/2d.2, not by a render), white t_connection (T viewed from its open side). The base-spec render of the same scene is red at the same pixel, depth bit-identical, sky pixels identical, >half of wall pixels change, and the no-textures render of the assigned world is pixel-identical to the unassigned world. (An early draft of this gate expected yellow at the cross's center pixel — geometrically wrong, it ignored the occluding arm; the standalone-tree verification run caught it, see below.) |
| 2d.4 | ✅ full workspace suite green (113 tests at verification time, including the concurrently-landing 2e/2f suites); 2c.4 parity test re-run green as part of it |

The commit was additionally verified standalone: the staged 2d
diff applied to a clean HEAD worktree (with the untracked
`assets/` and `fork-src/` fixtures linked in) builds and passes
its full workspace suite — that run is what exposed the cross
expectation error above, because in the combined working tree
the GPU half of 2d.3 can skip silently when parallel test
binaries contend for the lavapipe adapter.

End-to-end on real data: `tools/capture.sh --renderer rt
--textures assets/ultica_run1` on the fake house now renders its
wall runs with the *edge* subtile picture (continuous vertical
siding) instead of the standalone block repeated — compare
`captures/house_rt_textured.png` (base selection, pre-2d) with
`captures/house_rt_textured_2d.png` (same camera/size; the roof
also differs because the concurrently-landed Phase 2e mip chain
smooths distant sampling — not a 2d effect, roof top layers never
vary here).

## Follow-ups (not 2d scope)

- Connect-group predicates from real TileDefs (Phase 4 wiring):
  swap `same_material` for a group predicate at the
  `assign_variants` call sites.
- Rotation variants (Phase 8): apply `subtile_for_mask`'s
  rotation to side-face UVs.
- The raster path consumes the same virtual entries in Phase 2f.
