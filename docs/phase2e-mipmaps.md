# Phase 2e — Ray-cone texture LOD for the voxel tree tracer

Implementation plan for Phase 2e (texture mipmaps), integrating the
ray-cone LOD approach of Akenine-Möller et al., *Improved Shader
and Texture Level of Detail Using Ray Cones* (JCGT 2021) — the
"improved ray cones" refinement of Ray Tracing Gems ch. 20 —
adapted to this project's renderer. Source plan supplied by Joey
2026-11-15 (written against a generic wgpu ray-query triangle
renderer); this document is that plan **mapped onto our
architecture**, with every divergence decided explicitly below.

## 1. How the source plan maps onto this renderer

Our renderer is not the source plan's renderer. It is a compute
tracer over a sparse voxel 64-tree (`crates/treebuild`), with a
**texture array** of per-material face layers, per-voxel UVs
computed from the hit position, exactly one secondary ray (the sun
shadow ray, which samples no textures), and one transparent overlay
hit on the primary ray. wgpu is pinned at 24.0.5.

| Source-plan element | Verdict here | Why / what instead |
|---|---|---|
| Ray-cone LOD core (`width`, `spread`, `textureSampleLevel` at cone-derived LOD) | **Adopted** | This is the phase. §2 below. |
| Mip chain generation | **Adopted, CPU-side** | Scene-local layer counts are small (fake house: 27); CPU generation at spec-build time with the voxelizer's premultiplied alpha-aware averaging. GPU mipgen passes are unjustified at this scale. |
| Per-triangle Δ precompute (uvArea/worldArea per triangle) | **Collapses to a constant** | Every voxel face maps the full texture across exactly 1×1 world units: uvArea = 1, worldArea = 1 for every face in the scene. Δ = 0.5·log2(texW·texH), identical for all faces of a texture. No per-triangle buffer exists or is needed. |
| wgpu ray queries, BLAS/TLAS, `EXPERIMENTAL_RAY_QUERY` | **Not applicable** | Traversal is our own WGSL over the 64-tree; there are no acceleration structures and no ray-query calls anywhere in the project. No feature gating changes. |
| Bindless texture binding arrays | **Not applicable — already on the plan's own fallback** | The source plan's fallback for binding-array gaps is "texture atlas or array texture". We use a texture array by design (one array, per-material layer indices in a lookup buffer). Nothing to build. |
| Cone state through bounces; curvature β; reflection/refraction cones | **Mostly not applicable** | Voxel faces are flat (curvature is 0 or undefined at edges — there is no smooth surface to spread/focus the cone), and this renderer has no reflection or refraction rays. The shadow ray samples no textures. The one place cone state carries past a hit is the **transparent overlay**: it is the same primary ray continued, so its texture LOD is evaluated with the same cone at its own (farther) hit distance. If a future phase adds reflections, this section must be revisited with the Falcor port (see §5). |
| Alpha-tested candidate loop | **Adapted** | We have no candidate-intersection loop; cutout is a shading decision (texel alpha < 0.5 → flat-color fallback, recorded in the texture hookup). What carries over is **alpha-coverage-preserving mip generation**: without it, cutout textures thin out at coarse mips and the fallback decision flickers at distance. |
| Normal-map mip handling (renormalize, Toksvig) | **Not applicable** | Voxelizer block layers are albedo only; there are no normal maps in the material model. |
| sRGB filtering in linear space | **Deviation, recorded** | Our textures are sampled raw (Rgba8Unorm, display-value convention — see docs/phase2c-rt.md), so mip averaging happens in the stored space, consistently with how level 0 is sampled today. Filtering in linear space would make coarse mips inconsistent with the un-mipped render. Revisit only if the whole renderer changes color convention. |
| Hybrid raster primary visibility (their M6) | **Out of scope** | That is Phase 2f's renderer, which gets hardware derivatives for free in its fragment shader and needs none of this. |
| Debug LOD visualization | **Adopted** | A debug uniform flag renders `floor(lod)` as false color. |
| Validation vs supersampled reference + CPU/GPU cross-check | **Adopted, adapted** | CPU reference mirrors the WGSL LOD math (the project's established cpu.rs oracle pattern); analytic cases from the source testing matrix that apply to flat voxel faces; two-pose variance gate (already 2e.3). |
| Performance pass (their M8) | **Folded into 2c.9** | Frame timing happens on the RX 9070 XT with the 2c.9 run; coarse mips are also cheaper to fetch, so note the RT frame time with mips on/off there. |
| Patent note (NVIDIA ray-cone patents) | **Recorded** | The paper describes its implementation as public domain; the source plan flags US11443475 and a refraction-cone patent. This project is a non-commercial frontend; the note is recorded here for completeness, not as legal advice. |

## 2. The LOD math, as it applies to voxel faces

Ray cone state for a primary ray (full-angle convention, matching
the paper and Falcor — the Refraction Ray Cones chapter's half-angle
convention is *not* used anywhere here):

- `spread = atan(2 · tan(fovY/2) / imageHeight)` — computable in the
  render shader from existing uniforms (`tan_half_fov`,
  `viewport.y`); no new uniform needed. `width = 0` at the eye.
- Footprint at a hit at distance `t`: `w_hit = spread · t`.
- Per-face Δ: with uvArea = worldArea = 1, `Δ = 0.5 · log2(texW ·
  texH)`, from `textureDimensions(tex_array, 0)` in the shader. For
  the standard 32×32 layers, Δ = 5.
- `lod = Δ + log2(w_hit) − log2(|dot(N_geom, V)|)`, clamped to
  `[0, max_mip]`. `N_geom` is the voxel face normal (the traversal's
  entry normal — for axis-aligned faces it *is* the geometric
  normal; there is no separate shading normal in this renderer).
- Sample: `textureSampleLevel(tex_array, samp, uv, layer, lod)`
  with a `Linear` mipmap-filter sampler (trilinear between levels).

Numerical hygiene (from the source plan, all applicable):
clamp `w_hit` to a small positive epsilon before `log2`; clamp
`|N·V|` below (1e-4) — grazing faces are where this term explodes,
and the final clamp to `max_mip` is what keeps them finite; a
debug-build NaN check (`if lod != lod { lod = 0 }` plus a marker
color) while the gates are being established.

Worked expectations (the analytic gate cases): a fronto-parallel
face at distance `t` has `|N·V| = 1`, so `lod = Δ + log2(spread·t)`
— LOD rises by exactly 1 per doubling of distance. Tilting the face
by θ adds `−log2(cos θ)`.

## 3. Work steps (source milestones, mapped)

- **W1 — Test scene + baseline.** A textured wall plane at distance
  and at a grazing angle in a treebuild gate world (checkerboard +
  UV-grid layers), captured with level-0 nearest sampling: the
  moiré baseline. (Their M0.)
- **W2 — Mip chain generation.** In `TextureSet::from_spec` (or the
  spec builder): levels `floor(log2(tile_size)) + 1` per layer,
  CPU downsample 2×2 with premultiplied alpha-aware averaging +
  edge clamping (Phase 5 §8 rules); alpha coverage rescale for
  cutout materials so mean alpha per level tracks level 0.
  Texture created with the full `mip_level_count`; sampler gains
  `mipmap_filter: Linear` and no restrictive LOD clamp (a too-low
  max clamp silently degrades mips to level 0 — the source plan's
  warning, directly applicable). (Their M1.)
- **W3 — Cone LOD sampling.** `tex_albedo` takes the ray cone +
  hit distance and samples at the §2 LOD; primary hits and the
  transparent overlay hit both use it. Debug LOD false-color mode
  behind a uniform flag. (Their M3, with M5's coverage behavior
  coming from W2's generation.)
- **W4 — Validation.** CPU reference implementation of the LOD
  formula in Rust; analytic cases (distance doubling, slant) checked
  against GPU readback of the debug view; two-pose variance gate;
  close-up identity check. (Their M7, scoped to primary rays.)

Out of scope, explicitly: secondary-bounce cones (no such rays),
anisotropic `textureSampleGrad` filtering (their M9 — a possible
later extension; if attempted, the errata's gradient ×2 scaling
applies), refraction cones, GPU mip generation.

## 4. References

- Akenine-Möller et al., *Improved Shader and Texture Level of
  Detail Using Ray Cones*, JCGT 2021:
  https://jcgt.org/published/0010/01/01/paper.pdf — errata and the
  public-domain code link at
  https://research.nvidia.com/publication/2021-04_improved-shader-and-texture-level-detail-using-ray-cones
- Falcor `TexLODHelpers.slang` — the canonical implementation; only
  needed if secondary rays ever appear (see §1).
- Akenine-Möller et al., *Texture Level of Detail Strategies for
  Real-Time Ray Tracing*, Ray Tracing Gems ch. 20 (2019).
- TU Clausthal AliasfreeShadowRayCones — cones for alpha-tested
  shadow rays; relevant only if shadow rays ever sample textures
  (ours do not).
