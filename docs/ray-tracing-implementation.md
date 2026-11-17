# How ray tracing is implemented in the CDDA voxel frontend

Complete explanation, verified against the code at HEAD `393df0a`
(2026-11-16). This finishes the explanation that was cut off in the
"Explain CDDA voxel ray tracing" chat on 2026-11-16 — the first half
(world → tree, GPU layout, traversal start) is restated here so the
document stands alone, and the second half (LOD, shading, lighting
wiring, what is not there) is the part that never arrived.

Register note (Joey's standing rule): proper technical terms, not ELI5,
with project-specific terms defined inline on first use.

## 0. One paragraph

The ray-traced path is a **software compute ray tracer**: a WGSL compute
shader walks a sparse voxel 64-tree (each internal node has 4×4×4 = 64
children) directly. No triangle mesh, no hardware ray queries
(`EXPERIMENTAL_RAY_QUERY`, BLAS/TLAS), no reflection or refraction rays.
It lives in `crates/treebuild` (world, builder, CPU oracle, tracer) with
engine integration in `crates/engine/src/rt.rs`, behind `--renderer rt`.
Raster is still the default; RT is opt-in until gate 2c.9 (frame time on
the RX 9070 XT) passes. A CPU oracle in `crates/treebuild/src/cpu.rs`
mirrors the WGSL tracer line-for-line, so exact checks (hit UVs,
material ids, LOD) run without a GPU.

## 1. World → tree + material side-table

The engine builds the RT world from the **same** cutaway-filtered chunk
map as the raster path (`world::filtered_map`, shared by construction, so
layer selection cannot diverge), converted to a `VoxelWorld`: a sparse
hash map `[i32;3] -> VoxelCell { color, alpha, material_id, lit, flags }`
in grid space (x east, y south, z up; world→grid is `(wx, -wz, wy)`).

`build_tree` inserts cells in deterministic sorted order into the
`Sparse-Voxel-64Tree` crate. Depth is the smallest d ≤ 6 whose span `4^d`
covers the world — depth 4 (256/axis) for the stored 132×132 bubble
world. Colors are quantized round-to-nearest to u8, which is all the
crate's node stores.

Everything the node *cannot* store rides in a **side-table**: a parallel
array of 16-byte `SideEntry { material_id, lit, flags, alpha }` records,
indexed by flattened node index, `EMPTY` for internal nodes. Flag bits:
1 = transparent, 2 = cutout, 4 = emissive, 8 = memorized (a **memorized**
tile is one the character remembers but cannot currently see — it renders
desaturated and dimmed).

The crate's `Node64` fields are private, so `flatten()`'s bytes are the
only walkable structure. The builder therefore *reconstructs* the
side-table by BFS-walking the flat array, deriving each node's voxel
bounds from its position in BFS order, and verifying every leaf's flat
color against the world voxel at the reconstructed coordinate. Any
mismatch fails the build (`SideTableMismatch`) — gate 2c.2 proves this
catches a shuffled table and a wrong-world reconstruction.

Never-seen (`BLANK`) voxels are excluded at build time and counted in
`excluded_never_seen`. They are simply *absent* from the tree, so no
primary, shadow, or AO query can ever reveal them.

## 2. GPU layout

Two compute pipelines share one traversal source (`TRAVERSAL_WGSL` in
`crates/treebuild/src/render.rs`): `render_main` (per-pixel rendering,
workgroup 8×8) and `query_main` (caller-supplied rays → hit records,
workgroup 64, used for picking). Group 0 of both is the crate's own
`Tree64GpuManager` bind group, used directly — the "R0 primary arm";
gate 2c.10 asserts its node buffer is exactly `node_count × 32` bytes.
Group 1 is ours: side-table, camera uniforms, and the outputs (color
storage texture + f32 depth buffer for render; ray/result buffers for
query).

One hard-won quirk: the slab test and entry-normal helpers are written
vectorized, component-wise, and the debug-LOD palette is an if-chain —
dynamically-indexed local arrays compiled clean through naga but *lost
the llvmpipe device* at pipeline creation.

## 3. Traversal

`descend(p)` walks from the root to the point `p`: at each level it
computes which of the 64 children contains `p`
(`cidx = (pz<<4)|(py<<2)|px`), checks the child mask bit; if the bit is
clear it returns that child as the *largest empty cell containing p*; if
set, it follows `child_ptr + mask_rank` down, returning *occupied* when
it reaches a leaf.

`trace(ro, rd, skip_transparent)` then loops (max 8,192 steps):

1. Slab-test the ray against the root; start at `t = max(entry, 0)`,
   limit `min(root exit, max_dist)`.
2. Sample `p = ro + rd * (t + 1e-4)` (the epsilon keeps the ray from
   re-hitting the cell it just left); `descend(p)`.
3. If the cell is empty, `advance` — jump `t` to the exit of that empty
   cell (`advance` uses the slab exit of the cell's box) — and continue.
   This is what makes the walk fast: it skips whole empty subtrees in
   one step instead of stepping voxel by voxel.
4. If occupied and `skip_transparent` is set and the voxel's flag 1 is
   set, advance past just that 1×1×1 cell and continue.
5. Otherwise slab-test the 1×1×1 cell, take `dist = max(entry, 0)`, and
   return a `Hit` with the node index, the cell minimum, the distance,
   and the entry normal (which face the ray entered through, from the
   slab test — not from a stored normal).

`point_occluded(p)` is the same descent used as a boolean: occupied and
not transparent.

## 4. Hit UVs and texture lookup

UVs are in-voxel — `local = hitp - voxel_min`, so every face maps the
full texture over exactly 1×1 world units:

- top/bottom (`|n.z| > 0.5`): `uv = (local.x, local.y)`; layer is top
  or bottom by the sign of `n.z`
- x-facing side: `uv = (local.y, 1.0 - local.z)`
- y-facing side: `uv = (local.x, 1.0 - local.z)`

The `1.0 - local.z` on sides puts +z at `v = 0`, so textures stand
upright. Which layer is read comes from `mat_layers`, a storage buffer
of `vec4<u32>` per material id: `(top, side, bottom, present)`. A
material with `present == 0` keeps the flat node color.

The sample is `textureSampleLevel(tex_array, tex_samp, uv, layer, lod)`
against a trilinear sampler (Linear mag/min/mip, ClampToEdge,
`lod_max_clamp = mip_levels - 1`). Cutout fallback: after sampling, if
the material's flag 2 is set and `texel.a < 0.5`, return the flat color
— traversal already decided the voxel is solid, so a transparent texel
is not punched as a hole, it just falls back to flat color.

## 5. Explicit LOD — the part that makes it a compute-shader problem

A compute shader has no implicit derivatives, so `textureSample()`
cannot derive a mip level. We use `textureSampleLevel(..., lod)` with a
ray-cone LOD (Akenine-Möller et al., JCGT 2021), implemented twice —
WGSL `cone_lod` and the CPU mirror `ray_cone_lod` in
`crates/treebuild/src/lod.rs`:

```
spread = atan(2 * tan_half_fov / viewport_height)
w_hit  = max(spread * t, 1e-4)              // cone footprint at hit distance t
delta  = 0.5 * log2(texW * texH)            // = 5 for 32x32, constant scene-wide
lod    = delta + log2(w_hit) - log2(|N·V|)  // |N·V| epsilon-clamped to 1e-4
lod    = clamp(lod, 0, max_mip)             // NaN -> 0
```

That constant `delta` is the voxel simplification — no per-triangle
area or barycentrics are needed, because the 1×1 mapping is uniform.
One LOD per hit, from that hit's own distance; a transparent overlay
hit re-enters `shade()` and gets a farther LOD. Shadow and AO rays
sample no textures at all. A mip-colorized debug view
(`render_debug_lod`, one saturated color per mip level) exists to check
that transitions land at the expected distances.

## 6. Mip chain

Generated CPU-side in `TextureSet::from_spec()`; the spec carries level
0 only. `lod::mip_chain()` downsamples 2×2 to 1×1 (6 levels for 32 px):

- premultiplied alpha-aware averaging — fully transparent texels
  contribute no color, so cutout edges do not bleed;
- averaged in stored/display space, not linear, to match level-0
  sampling;
- alpha-coverage preservation: each generated level is rescaled so its
  mean alpha tracks level 0's, otherwise cutouts thin out and vanish at
  distance.

## 7. Shading — `shade()` per hit

1. **Albedo** from §4 at this hit's own LOD.
2. **Memorized** (flag 8): desaturate — mix toward luminance by 0.6.
3. **Emissive** (flag 4): return `albedo * 1.6`, unlit; it feeds the
   Phase 2b bloom pass downstream.
4. Otherwise: base light 0.35 plus a sun term. `light = 0.35 + 0.85 *
   max(dot(n, sun_dir), 0)` when the face points at the sun — but first
   a **shadow ray**: `trace(hitp + n * 0.01, sun_dir, true)`; if it
   hits, the sun term is dropped entirely (hard shadows, one ray).
5. **Occupancy AO**: the four cells just outside the face, diagonally
   adjacent along its tangents (`point_occluded` each); `ao = 1.0 - 0.10
   * occ`, so at most 0.6 in a fully enclosed corner.
6. **Game light level**: `lit_factor(lit)` maps the per-tile light
   level 0–5 to 0.25 / 0.45 / 0.65 / 0.85 / 1.0 / 0.5 — note level 5
   maps to 0.5, not the brightest; the code gives no reason, so treat
   it as a level whose meaning lives in the game-side exporter, not
   the shader.
7. Final color: `albedo * (light * lit_factor * ao)`, all in stored
   (display) space — which is why textures must be flat albedo, not
   pre-shaded (see the texture-pipeline plan, §4.1 de-shade).

## 8. `render_main` — one pixel

```
rd = normalize(cam_fwd + ndc.x * tan_half_fov * aspect * cam_right
               + ndc.y * tan_half_fov * cam_up)   // pixel-center ray
first = trace(ro, rd, false)   // nearest hit, transparent included
opq   = trace(ro, rd, true)    // nearest opaque hit
col = sky(rd)                  // horizon/zenith/ground gradient
if opq.hit: col = shade(ro, rd, opq); depth = opq.dist
if first.hit and transparent and (no opaque hit, or first is nearer):
    col = mix(col, shade(ro, rd, first), alpha)  // ONE transparent layer
    depth = first.dist
```

So transparency is a single blended layer over the opaque (or sky)
result — no multi-layer transparency, no transparency in the depth
buffer beyond the first transparent hit. The f32 depth buffer is what
lets billboards rasterize as depth-tested quads over the traced image
afterwards (Phase 6/2h).

`query_main` reuses the same `trace` for picking: given caller-supplied
rays it returns hit flag, node, distance, voxel, and normal per ray.

## 9. Lighting and frame wiring (Phase 7 + 2h)

What §7's `lit` and memorized handling consumes comes from the lighting
bake (`crates/lighting`, Phase 7): a per-vertex bake by default, with a
per-block bake as a CLI option (`--light-bake per-block|per-vertex`).
The per-vertex bake's resolution is mesh vertex density — a MEMORIZED
patch inside a greedy-merged floor mega-quad has no vertices and changes
0 pixels; the per-block baker is the lever (pinned saturation keep 0.30,
measured 0.301). The RT path is per-voxel, so it gets the bake exactly.

Phase 2h wires the layers into one live frame loop, in canonical order:
**world → fields → billboards → post → UI** (`engine::frame::render_frame`).
Vehicles stamp into the world pass before the shared visibility mask;
transients ride the fields pass; the UI quad draws last. Gate 4.12
measured in-loop: median 8.23 ms, p99 36.56 ms over 30 composed frames
on llvmpipe — the target-hardware re-measure on the RX 9070 XT is still
open, as is 2c.9.

## 10. What is NOT there

- No reflection, refraction, or multi-bounce rays; no denoiser. The
  "ray tracing" is primary + one shadow ray per lit hit + 4 boolean AO
  probes.
- No hardware ray queries; no BLAS/TLAS. Portability is the point — it
  runs on llvmpipe, so CI runs RT scenes without a GPU.
- No temporal accumulation: frames must be stable and golden-testable,
  so sampling is deterministic (hard shadows, fixed AO probes).
- Transparent voxels do not cast shadows (`skip_transparent` on the
  shadow ray) and do not write the depth a billboard would test
  against, beyond the first-layer blend in §8.
- Billboards do not cast real shadows; they get blob shadows.

## Verification

Every claim above was read from the code, not recalled: traversal,
`cone_lod`, `tex_albedo`, `lit_factor`, `shade`, `sky`, `render_main`,
`query_main` in `crates/treebuild/src/render.rs` (`TRAVERSAL_WGSL` /
`RENDER_WGSL` / `QUERY_WGSL`); `ray_cone_lod` in
`crates/treebuild/src/lod.rs`; the CPU oracle in
`crates/treebuild/src/cpu.rs` (`descend`, `cast_ray`, `occupied_at`,
`dda_reference`); engine integration in `crates/engine/src/rt.rs`
(`RtRenderer`, `render_rt_frame`); frame order and bake wiring in
`crates/engine/src/frame.rs` and `docs/phase2h-integration.md` /
`docs/phase7-lighting.md`. Checked 2026-11-16 at HEAD `393df0a`.
