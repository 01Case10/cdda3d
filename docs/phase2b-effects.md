# Phase 2b — Shader field effects and the HD-2D look

What was built, where, and how it performed against the Part II §5b
gate. Joey's direction governs throughout: **fields/effects are never
rendered from 2D sprites** — sprites contribute only extracted
parameters; every visual is a procedural shader effect. Look target:
modern HD-2D (bloom on emissives, tilt-shift DoF, warm grade).

## 1. Effect extraction (voxelizer)

`crates/voxelizer/src/effects.rs`, wired into `lib.rs` (module +
re-export) and the bin (`--effects`, `--field-ids <file>`).

- `run_effects(ts, cfg, field_ids, out_dir)` writes `effects.json` +
  `effects_fallback.log`. It is a **separate entry point from
  `run()`** on purpose: the Phase 5 golden file set (gate 5.1) is
  unchanged, and all 13 Phase 5 gate tests still pass untouched.
- Per field type, the entry's sprite is resolved through the game's
  own chain (exact → variant → season → looks_like) and composited
  fg-over-bg, then **sampled, never drawn**: `sample_sprite` computes
  the alpha-weighted mean color (premultiplied-clean — poison colors
  under alpha=0 cannot leak), the hottest visible texel's color
  (emissive tint), alpha coverage, and mean luminance. Integer
  accumulation ⇒ deterministic; two runs are byte-identical
  (asserted in tests, and verified on the real UltiCa run below).
- Field ids default to every `fd_*` entry in the tileset (CDDA's
  field namespace); `--field-ids` adds explicit ids (e.g. from a
  TileDefs export once Phase 3 is in-game).
- Every requested id yields an entry or an explicit **logged
  fallback** (class defaults + neutral palette, `fallback: true`,
  one line in `effects_fallback.log`). Never a sprite, never silence.
- Hand-tuning uses the existing `overrides.toml` mechanism:
  `OverrideEntry` gained four optional keys — `effect_density`,
  `effect_emissive`, `effect_noise_scale`, `effect_class` — validated
  like the existing keys (gate 5.9 behavior preserved).

### Render-class mapping (`classify_field`, first match wins)

| substring(s) in field id | class |
|---|---|
| `fire`, `flame` | flame |
| `smoke` (incl. cig/crack/meth/weed smoke) | smoke |
| `gas`, `fungal`, `haze`, `dust`, `spore`, `seep`, `stench`, `tear` | gas |
| `blood`, `acid`, `bile`, `sludge`, `fluid`, `water`, `goo`, `sap`, `ink` | liquid |
| `electric`, `light`, `plasma`, `spark`, `radiation` | energy |
| anything else (`web`, `splinters`, …) | generic |

Class defaults set density scale, noise scale/speed, buoyancy/drift
and base emissive strength; the sprite's coverage modulates density
and its luminance modulates emissive strength for flame/energy.

### Real UltiCa run (this VM, 2026-11-15)

`voxelizer --tileset fork-src/tilesets/gfx/UltimateCataclysm --out <dir> --effects`
(the Phase 5 invocation plus `--effects`):

- **39 field types, 39 with sprites, 0 fallbacks.**
- Classes: liquid 20, gas 6, smoke 5, flame 3, energy 3, generic 2.
- Spot checks: `fd_fire_int1` → flame, base (1.00, 0.71, 0.15),
  emissive strength 2.71; `fd_blood` → liquid, dark red (0.37, 0.04,
  0.08); `fd_electricity` → energy, pale blue, emissive 2.12;
  `fd_smoke_overlay` → smoke, neutral grey.
- Second run's `effects.json` is **byte-identical** (`diff` clean).
- The same run reproduced the Phase 5 totals (7,717 materials,
  26,389 layers, 14,450 sprites, 0 missing).

## 2. fx crate — field effect system

`crates/fx` (deps exactly as scaffolded: wgpu 24, glam, bytemuck,
pollster, image, serde/serde_json — no new dependencies).

- `effect.rs` — the `effects.json` contract, mirrored field-for-field
  from the voxelizer's `EffectParams` (fx deliberately does not
  depend on the voxelizer; the JSON is the contract, like
  `materials.json`). Includes `load_effects_json` and the
  game→world drift conversion (game z-up → renderer Y-up).
- `world.rs` — per-tile field model mirroring the protocol
  (`FieldOnTile` + `DisplayedField`): `TileFields { fields,
  displayed }`, and `composite_weights`, a pure function. The
  displayed field weighs `intensity × 4.0`, others `intensity`, so a
  displayed intensity-1 field still dominates an intensity-3 rival —
  the game's cheap path decides, the engine never re-derives it.
- `density.rs` — intensity→density curve (0 → 0, 1 → 0.35, 2 → 0.70,
  3 → 1.0, × the type's density scale), Beer–Lambert `obscurance`
  (intensity-3 smoke: >99% through one tile; intensity-1: ~83% peak
  before falloff, not full), and the **CPU reference** of the shader
  density field: value-noise fbm (4 octaves), radial falloff, class
  shaping (flame base-heavy + flicker, gas/liquid low), drift scroll.
  Animation is time-based only; nothing per-turn crosses the IPC.
- `render.rs` — GPU passes (see §4 for the harness caveat):
  - **Field pass**: instanced unit volumes from a storage buffer.
    The fragment shader intersects the view ray with the volume,
    samples fbm at the **chord midpoint**, and scales by chord
    length + interior falloff — long chords through the middle are
    dense, silhouette chords fade. (A radial falloff evaluated on
    the cube *surface* is zero everywhere; the chord formulation is
    the fix, found during this phase.) Flame adds emissive flicker
    and a taller volume; gas/liquid volumes are flattened and
    low-weighted; smoke drifts and rises via the noise scroll.
    Instances are sorted back-to-front on CPU. **The field shader
    binds no texture of any kind** (gate 2b.2 asserts this on the
    shader source).
  - **Post pass**: one fullscreen pass whose uniform flags toggle
    bloom / DoF / grade / chromatic independently (mirrors the CPU
    reference in `post.rs`). All-off returns the input texel exactly.

## 3. HD-2D post stack

`crates/fx/src/post.rs` is the renderer-agnostic reference
implementation over (color, depth) frames; `render.rs` carries the
WGSL twin. Passes, in order: **bloom** (additive blurred
bright-pass), **tilt-shift DoF** (sharp at the focal depth; blur
grows with |depth − focal| and with vertical distance from the
focal band), **chromatic offset** (channel shift growing from the
center), **warm grade + vignette** (×(1.07, 1.01, 0.93), corner
darkening). Each is independently toggleable; all-off is
pixel-identical to the input (proven exactly in the CPU reference
and by the shader's early-out). The stack consumes only a finished
world frame + depth, so it applies unchanged to the Phase 2c ray
tracer. **UI is never inside the stack**: `composite_ui` alpha-composites
the UI frame strictly after, and opaque UI pixels come out bit-exact.

One parameterization difference: the CPU reference takes focal depth
in view-space distance; the WGSL pass takes it in projected (0..1)
depth, computed from the camera by `Camera::focal_projected_depth`.
Same semantics — sharp band at the follow target — different units,
because the GPU depth texture stores projected depth.

## 4. Test scene + captures (standalone harness)

`crates/fx/src/scene.rs` + `src/bin/fx-capture.rs`. Because fx may
not depend on `crates/engine`, the base layer is a simple box scene
rasterized inside fx (floor, bright backdrop wall with a dark
doorway, side walls, crates) with a row of field tiles in front of
the backdrop: smoke, fire, gas at intensities 1–3. This is a
**standalone test harness**, documented as such; wiring the passes
into the real engine is the follow-up in §6. Rendering follows
`crates/engine/src/capture.rs`'s pattern (`compatible_surface:
None`, offscreen target, staging readback); on no-adapter it exits 2
with `fx-capture: no wgpu adapter available` and writes nothing.

Captures in `captures/` (640×360, lavapipe, time pinned at 1.25 s):
`effects_post_off.png`, `effects_post_on.png`, `effects_bloom.png`,
`effects_dof.png`, `effects_grade.png`. Visually inspected: smoke
reads as soft grey-white volumes, fire glows yellow-orange and
blooms in the post-on capture, gas stays low and green; the post-on
capture shows the warm grade + vignette with no geometry change.

## 5. Gate results (Part II §5b)

| # | Result | Evidence |
|---|---|---|
| 2b.1 | ✅ PASS | `voxelizer/tests/effects_gate.rs::gate_2b_1_extraction_fixture`: every id → entry or logged fallback; palette matches an independent direct sample of the composited sprite (< 0.005/channel); two runs byte-identical (also byte-identical on real UltiCa: 39/39 entries, 0 fallbacks) |
| 2b.2 | ✅ PASS | `fx/tests/gate_tests.rs::gate_2b_2_no_sprite_rendering`: FIELD_SHADER contains no `texture`/`sampler`; render.rs touches no sprite asset; CPU density center > edge, silhouette corner exactly 0 |
| 2b.3 | ✅ PASS | `gate_2b_3_intensity_monotonic_and_obscurance`: curve strictly monotonic; obscurance int-3 smoke > 0.99, int-1 < 0.99; rendered CPU density monotonic at a fixed point |
| 2b.4 | ✅ PASS | `gate_2b_4_displayed_field_weighting`: displayed dominates in both configurations, including displayed-intensity-1 vs rival-intensity-3 |
| 2b.5 | ✅ PASS (adapter-conditional) | `gate_2b_5_effect_captures` on lavapipe: scene renders non-blank (> 10 distinct colors at 320×180 test size; 1,424 at capture size); single-volume diff-vs-empty measurement: int-3 mean delta > 1.3 × int-1, peak delta 23 of a ~28 ceiling for grey-on-white. Captures in `captures/effects_*.png` |
| 2b.6 | ✅ PASS | `gate_2b_6_post_isolation` (CPU reference): all-off == input exactly; bloom leaves distant dark pixels untouched and only adds light; DoF focal band bit-exact, far field blurs; grade warms (B/R ratio drops). GPU all-off identity holds by the shader's early-out branch (same code path as the post-off capture, which is the pre-2b box renderer's output) |
| 2b.7 | ✅ PASS | `gate_2b_7_ui_composited_after_post`: UI rect pixels bit-exact after compositing over a fully graded frame; by construction render.rs contains no UI handling — compositing is the caller's post-stack step |
| 2b.8 | ⛔ NOT RUN — target hardware only | Not claimed. lavapipe timings (whole 640×360 5-variant capture run ≈ 2.6 s wall incl. startup on 1.5 CPUs) are software-render numbers and are **not evidence** about the 3 ms budget on the RX 9070 XT |

Totals: `cargo test -p voxelizer` — 13 Phase 5 gate tests + 4 new
effects tests, all pass. `cargo test -p fx` — 7 unit + 7 gate tests,
all pass.

## 6. Engine integration (follow-up for the parent)

The fx passes are written to drop into `crates/engine`:

1. **World model**: engine's snapshot/delta handling already
   produces per-tile field lists; convert each `SubmapBlock` tile's
   `fields` + `displayed_field` into `fx::world::TileFields`, and
   each field type def id → `fx::effect::EffectDef` (loaded once
   from the voxelizer's `effects.json` via `load_effects_json`,
   keyed by the TileDef's string id).
2. **Field pass**: after the opaque mesh pass in
   `engine::render::Renderer::draw`, draw field instances for tiles
   in the cutaway window: one instance per (tile, field) with
   `density = fx::density::effective_density(def, intensity) *
   weight` from `composite_weights`. Reuse fx's `FIELD_SHADER` and
   `FieldInstGpu` layout verbatim (move them behind a small
   `FieldPass` struct in fx if preferred). Depth: test on, write off,
   sorted back-to-front — as in the harness.
3. **Post stack**: the engine's capture/window paths currently draw
   directly to the target. Route the world render (mesher passes +
   field pass + later billboards/RT) into an offscreen color+depth
   pair, run fx's post pass with `PostConfig` (focal depth = camera
   distance to the follow target, projected), then composite the UI
   quad **after** — Phase 4's compositor already lands last; keep it
   that way and gate 2b.7 holds in the real engine.
4. **Captures**: extend `tools/capture.sh` with an `--effects`
   scene (fake house + the fx test-scene field row) so 2b.5's
   golden-diff harness (when 2.10's is built) covers both.

## 7. Deviations from the plan

- **Post stack is one fused fullscreen pass with per-pass uniform
  toggles**, not four separate passes. Toggleability and isolation
  semantics are unchanged (each effect is independently switchable
  and the CPU reference implements them as separate functions);
  fusing avoids three extra full-screen texture round-trips in the
  standalone harness. Splitting it in the engine integration is a
  mechanical change if profiling wants it.
- **GPU DoF focal depth is in projected-depth units** (see §3);
  the CPU reference uses view distance. Noted rather than hidden
  because gate 2b.6's DoF assertions run against the CPU reference.
- **`effects.json` is produced by a separate voxelizer entry point**
  (`run_effects` / `--effects`), not added to `run()`'s output set,
  to keep the Phase 5 golden (5.1) intact.
- **No depth-texture soft-particle fade** for field volumes: edge
  softness comes from the chord/silhouette falloff. A depth-fade
  against the scene depth is a natural engine-integration addition
  (the depth texture is already bound in the post pass).
- **2b.8 not attempted** (target hardware only, per the plan).
