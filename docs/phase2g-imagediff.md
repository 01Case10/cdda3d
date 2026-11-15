# Phase 2g.3 — Formal image-diff golden gate (gates 2.10 / 5.13)

Closes the long-open conditions **2.10** (Phase 2's golden image-diff)
and **5.13** (Phase 5's visual-scenes image-diff): until now the
headless captures in `captures/` were visually verified by a human
but never diff-gated. This workstream adds the harness, the
committed goldens, and the negative test that proves the gate can
fail.

## Where things live

| Piece | Path |
|---|---|
| Harness (tests) | `crates/engine/tests/golden_gate.rs` |
| Committed goldens | `crates/engine/tests/golden/*.png` (mirrors the voxelizer's `tests/golden/` layout) |
| Wrapper | `tools/golden.sh` (`--update` regenerates) |
| Failure artifacts | `target/golden-failures/` — `<scene>.actual.png`, `<scene>.diff.png`, `SKIPPED.txt` |

## Scene set

All scenes render the Phase 2 fake house (`--snapshot stub-house`)
at **640×360**, target tile (6,6), distance 28 — small enough that
each capture takes 0.25–1.8 s on lavapipe, large enough that
materials and cutaway geometry are legible.

| Scene | Renderer | Textures | Camera / cutaway |
|---|---|---|---|
| `house_raster_flat` | raster | — | yaw 45°, pitch 45°, cutaway z≤1 (full view) |
| `house_raster_cutaway0` | raster | — | yaw 135°, pitch 50°, cutaway z≤0 (roof removed; the `captures/house_cutaway0.png` view) |
| `house_rt_flat` | rt | — | yaw 45°, pitch 45°, cutaway z≤1 |
| `house_raster_textured_ultica` | raster | `assets/ultica_run1` | yaw 45°, pitch 45°, cutaway z≤1 |
| `house_rt_textured_ultica` | rt | `assets/ultica_run1` | yaw 45°, pitch 45°, cutaway z≤1 |
| `house_raster_textured_fixture` | raster | hermetic fixture | yaw 45°, pitch 45°, cutaway z≤1 |
| `house_rt_textured_fixture` | rt | hermetic fixture | yaw 45°, pitch 45°, cutaway z≤1 |
| `house_raster_effects` | raster | — (fx demo defs) | yaw 135°, pitch 50°, cutaway z≤0, fields + post on |

Two textured variants, deliberately:

- **UltiCa** (`assets/ultica_run1`, the dir the existing textured
  captures use) is the real-art evidence for 5.13. `assets/` is
  gitignored, so on a checkout without the voxelizer output these
  two scenes **skip loudly** (named message per scene) — the same
  conditional pattern `rt_gate`/`raster_tex_gate` already use.
- **Fixture** is a hermetic checkerboard tileset built by the
  harness itself (same construction as `raster_tex_gate`'s
  fixture: material *k* owns layers 3k/3k+1/3k+2, unique colors per
  layer). It keeps the textured path golden-gated on any machine,
  and it is what the negative test mutates.

Effects: the set originally shipped without an effects variant
(the fx→engine wiring, 2g.1, was landing in parallel and its
capture flags were not stable). **Added 2026-11-15** once the
flags settled: `house_raster_effects` — the interior camera
(135°/50°, cutaway 0, the fx_gate view) with field volumes +
HD-2D post on, defs injected from fx's hermetic demo set via
`CaptureArgs::effects_defs` (deterministic; `fx_time` is pinned
in `CaptureArgs::default`). 2,533 distinct colors vs 15 for the
flat frame — the effects are unambiguously in the picture.

## Tolerance model

Exact byte equality is wrong for GPU output: lavapipe here and a
real GPU on the target machine differ in interpolation, filtering
and sRGB quantization. The gate therefore uses three criteria, all
of which must hold:

1. **Per-pixel tolerance 8/255 per channel.** A pixel *matches*
   when every RGBA channel is within 8 of the golden. This absorbs
   few-LSB driver differences; a real material or geometry change
   shifts channels by tens to hundreds (the negative test's max
   delta is 179).
2. **Mismatch budget 1% of pixels** may exceed the tolerance.
   Legitimate cross-driver variance concentrates on rasterization
   edges, well under 1% of these mostly flat-shaded frames.
   Measured cross-backend evidence (this VM, Vulkan ICD hidden so
   wgpu fell back to a different backend): ≤1 mismatched pixel per
   frame (0.000%), max delta 27 on a single edge pixel.
3. **Mean absolute channel error ≤ 2.0** over the whole frame.
   Catches a global tint shift that sits everywhere just under the
   per-pixel tolerance and would otherwise pass.

Same-machine reruns are currently exact (0 mismatched pixels,
max delta 0 on all 8 scenes) — the tolerance is headroom for other
drivers, not slack the current renderer needs.

## Regeneration

Mirrors the voxelizer's 5.1 flow (`gate_5_1_fixture_golden`):

```sh
UPDATE_GOLDEN=1 cargo test -p engine --test golden_gate
# or
tools/golden.sh --update
```

Regeneration overwrites the golden PNGs from fresh captures and
passes unconditionally — review the PNG diff in the change before
committing it, exactly as with the voxelizer goldens. A missing
golden in comparison mode is a failure with instructions, not a
skip.

## No-adapter behavior (the vacuous-skip caveat)

The harness probes for an adapter first with a 64×36 capture. On
`CaptureError::NoAdapter` it prints a named banner to stdout **and**
stderr —

```
2g.3 GOLDEN GATE: NOT RUN — no wgpu adapter available …
This run proves NOTHING about rendering. …
```

— writes the same text to `target/golden-failures/SKIPPED.txt`,
and returns. A run with no adapter is recorded as **not run**,
never as a pass; 2.10/5.13 must not be marked closed from such a
run. Two further anti-vacuity guards: the probe asserts the frame
is non-blank (quads > 0, > 1 distinct color), and every scene
capture asserts quads > 0 and > 5 distinct colors before any
comparison.

Verification note: the live NoAdapter branch could not be forced
on this VM — the engine pins `Backends::all()` and both mesa
Vulkan (lavapipe) and GL adapters are present, and loader/ICD env
overrides only changed which adapter answered. The branch is the
same `CaptureError::NoAdapter` path gate 2.10's capture test
exercises, and the banner/marker half is unit-tested
(`gate_2g3_skip_notice_is_loud`).

## Negative test (gate 2g.3's failure proof)

`gate_2g3_negative_one_material_change_fails`:

1. Captures the fixture-textured raster scene from a scratch
   fixture in a temp dir (no committed data is touched).
2. Control: the frame diffed against itself passes the budget.
3. Recolors **one material** — the wall's side layer (fixture
   layer 4) — solid red in the scratch copy, and re-captures.
4. Asserts the diff **fails** the golden budget, and writes the
   diff artifacts to `target/golden-failures/`:
   `negative_one_material.diff.png` (mismatched pixels in magenta
   over the dimmed changed frame — visually, exactly the house's
   wall sides light up) and `negative_one_material.actual.png`.

Measured on this VM: **3,986 mismatched px of 230,400 (1.730%)**
vs the 1.0% budget, mean |err| 1.385, max delta 179. The margin
over budget is ~1.7×, not an order of magnitude — the wall sides
are ~1.7% of this frame — so the assertion is calibrated to the
measured signal (> budget and > 1,000 px), not to an assumed one.

On a real golden failure the main gate writes the same pair of
artifacts per failing scene (`<scene>.actual.png`,
`<scene>.diff.png`) and names both paths in the failure message.

## Results (2026-11-15, this VM, lavapipe)

- `cargo test -p engine --test golden_gate`: **3/3 pass**
  (`gate_2g3_golden_image_diff`, `gate_2g3_negative_one_material_change_fails`,
  `gate_2g3_skip_notice_is_loud`), ~6–9 s wall including captures.
- Golden gate detail: 7/7 scenes compared, 0 mismatched px each,
  capture times 0.25–1.8 s per scene. **Update 2026-11-15:** with
  the effects scene added, 8/8 scenes compared, 0 mismatched px
  each; regeneration reproduced the 7 existing goldens
  byte-identically, and the negative test is unchanged
  (1.730% vs the 1% budget).
- Full `cargo test -p engine` at the same tree state: **31/31
  pass** (17 lib + 2 capture + 3 golden + 3 multitile + 3
  raster-tex + 3 rt).
- Goldens are non-blank by construction and by assertion:
  distinct colors 15 (flat raster), 43 (flat RT), 1,630–3,465
  (textured).

## Deviations

- Scene size is 640×360, not the 1280×720 of `captures/house.png`:
  keeps the full set at ~5 s of capture time in the normal test
  suite on a 1.5-CPU VM; the fixed cameras otherwise match the
  existing captures (45°/45° full view, 135°/50° cutaway).
- Plan 2g.3 text says "fake house + fixture tileset scenes"; the
  set implements that as fake-house scenes *textured by* the
  fixture tileset (plus the real UltiCa set), which is the only
  scene the capture CLI can currently produce
  (`--snapshot stub-house` is the sole snapshot source).
- Effects golden deferred (see Scene set) — 2g.1's flags were not
  stable in-tree when this landed.
