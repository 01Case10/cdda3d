# Phase 4: UI compositing — `crates/ui-share`

Engine side of the UI handoff, implemented 2026-11-15. The game (fork,
Route B — `catacurses`-level capture, the Phase 0 primary route) renders
every UI window except the terrain window into a CPU RGBA buffer in
shared memory and suppresses the map window; this crate reads those
frames, composites them over the engine's 3D view, and maps engine
input back to the game. Route A (`display_buffer` readback) would
deliver the identical contract — an RGBA frame in the store plus a
`UiFrame` notification — so nothing here is route-specific.

Terms used below: a **UiFrame notification** is the small FlatBuffers
`UiFrame` socket message announcing a ready frame (buffer index,
sequence, dirty rect, `UiMode`, terrain rect); the pixels never travel
on the socket. The **terrain rect** (`w_terrain` rect) is the screen
rectangle the terrain window would have occupied, reported by the game
per frame because it depends on terminal size and the user's panel
layout. **Display-buffer pixels** are the game's UI buffer pixels;
**window pixels** are the engine framebuffer's physical pixels; the two
differ under DPI scaling (§4).

## What was built

| Module | Contents |
|---|---|
| `store` | Double-buffered shared-memory store over the Phase 1 48-byte `proto::shm::UiShmHeader`: header at offset 0, then two slots, each a 32-byte slot header (`seq`, FNV-1a checksum over dimensions + pixels, geometry) followed by RGBA pixels. In-memory backing and file-backed backing (std-only, `read_at`/`write_at`). |
| `frame` | `UiFrame` validation against the store header; terrain-rect masking; dirty-rect `UploadPlan` + CPU plan application. |
| `compositor` | wgpu fullscreen-quad pipeline (fx-crate offscreen pattern): background quad, then the UI quad inside the scaling viewport, `ALPHA_BLENDING`, texel access via `textureLoad` by pixel coordinate (nearest by construction); full and dirty-rect texture uploads; CPU src-over oracles for both. |
| `scaling` | Pure resize/DPI functions: logical↔physical size, `Resize` message construction, window↔display mouse conversion, stretch/integer-scale viewports, click-through-viewport mapping, resize-action decision. |
| `input` | Engine key/text/mouse/wheel/focus events → `proto::Input`; key-code table (printable = Unicode scalar, non-printable ≥ 0x1000); schema modifier bitmask (1 shift, 2 ctrl, 4 alt, 8 meta); `InputRouter` implementing `UiMode` gating and click routing. |
| `picking` | `PickCamera` (engine orbit-camera math), ray through a pixel via inverse view-projection, Amanatides–Woo DDA in tile space (1 tile = 1×3×1 world units: x, up, −y), `pick_from_ui_click` → `PickResult`. |
| `synthetic` | Deterministic CPU-drawn stand-ins for the main menu, character creation, inventory, crafting, and gameplay-sidebar frames (block graphics — panel geometry and list rows, no font rasterizer in scope), plus the tolerance image-diff helper and PNG export. |
| `latency` | Stage timers, median/percentile, and the labeled `LatencyReport`. |

Captures (from the gate tests, lavapipe): `crates/ui-share/captures/`
— `ui_main_menu.png`, `ui_character_creation.png`, `ui_inventory.png`,
`ui_crafting.png`, `ui_gameplay_sidebar.png` (raw frames) and
`ui_composited_gameplay.png` (GPU composite over a green "world"
background, visible through the terrain hole).

## Design decisions

- **Torn-frame protocol.** The writer always writes the inactive slot
  (pixels, then slot header) and publishes last by rewriting the main
  header with the bumped `seq` and flipped `buffer_index`. The reader
  reads the header, copies the indicated slot, re-reads the header,
  and requires: slot `seq` == header `seq` (both reads), geometry
  agreement, and a checksum match. Any disagreement is an error after
  bounded retries — never a returned partial frame. The checksum is
  load-bearing, not redundant: a sequence check alone cannot detect a
  writer that died mid-pixel-write in the *active* slot (proven by
  `gate_4_3_checksum_catches_corruption`).
- **Masking is alpha-only.** `apply_terrain_mask` zeroes alpha inside
  the reported terrain rect and touches nothing else; RGB inside the
  rect is preserved (blend ignores it at alpha 0). An unknown rect
  (`-1` fields) means *no* masking — compositing an unmasked frame is
  wrong-looking but safe, whereas inventing a rect could hide real UI.
- **Picking separation is a routing decision, not a picking detail.**
  `InputRouter::is_world_pick` is the single rule — `MapInput` mode
  AND inside the terrain rect AND outside every panel rect — and both
  click routing (`input`) and ray casting (`picking`) consult it, so
  the two can never drift apart.
- **Nearest sampling by `textureLoad`.** The compositor computes the
  source texel from the fragment's framebuffer position, the viewport
  origin, and the scale — sampling at output pixel centers, the
  standard nearest-neighbor convention. The CPU oracle initially used
  pixel corners and disagreed with the GPU at non-integer scales
  (gate 4.6, 200×150 case, 382 mismatched pixels); the oracle was
  fixed to the center convention, not the GPU.
- **Generated FlatBuffers types are `#[non_exhaustive]` outside the
  proto crate**, so `Input`/`Resize`/`UiFrame`/`PickResult` values are
  built via `Default` + field assignment throughout this crate.

## Gate results (Part II §7)

Suite: `cargo test -p ui-share` — **30/30 pass** (14 lib + 16 gate),
GPU tests re-run standalone with `--nocapture` on lavapipe
(`llvmpipe (LLVM 20.1.2, 256 bits)`) to rule out vacuous passes under
parallel adapter contention; standalone timings 0.3–1.0 s per GPU test
(real renders, not skips).

| Gate | Verdict | Evidence |
|---|---|---|
| 4.1 golden frames | ✅ harness level | `gate_4_1_synthetic_golden_frames_roundtrip_and_diff`: all 5 synthetic frames round-trip the store byte-exact, self-diff 0, menu-vs-inventory diff > 5 % of pixels; PNGs in `crates/ui-share/captures/`. Real goldens need the fork in-game (blocked, below). File-backed store round-trip also passes (`gate_4_1_file_backed_store_roundtrip`). |
| 4.2 map-region masking | ✅ | CPU: `gate_4_2_terrain_rect_masking` (offset rect 10,20 100×80 — alpha 0 inside, 255 outside, byte-exact). GPU: `gate_4_2_gpu_…` — terrain-hole pixel shows the background (40,90,40)±2, sidebar pixel is the panel color exactly; capture `ui_composited_gameplay.png`. The rect always comes from the `UiFrame`, never assumed. |
| 4.3 torn frames | ✅ harness form | `gate_4_3_torn_frame_detection_under_concurrent_write`: writer publishing at max rate for 2 s while the reader samples continuously — many thousands of reads, **0 torn frames observed** (every returned frame's bytes are uniform per its sequence pattern; inconsistent reads are rejected, not returned). The gate's 10-minute form needs the in-game writer. Plus the corruption test above. |
| 4.4 double-buffer sequencing | ✅ | `gate_4_4_…`: scribbling half of frame B into the inactive slot leaves the reader on complete frame A; completing B flips buffer index and bumps seq by exactly 1. |
| 4.5 dirty rect | ✅ CPU + GPU | CPU: dirty-only resident update is byte-identical to a full upload; stale-seq notifications rejected. GPU: `gate_4_5_gpu_dirty_upload_matches_full` — 0 mismatched pixels vs full upload, and the full result matches the CPU oracle (passes at tolerance 2, budget 0.1 %). |
| 4.6 resize and DPI | ✅ | Pure functions: DPI 1.25/1.5/2.0 round-trips, mouse conversion at scaling factor 2, integer-scale letterboxing, `Resize` carries physical size + DPI. GPU: captures at 160×100, 320×200 stretch, 400×300 integer (letterbox corners verified background), 200×150 non-uniform stretch — all match the CPU oracle (tolerance 2, budget 0.2 %). |
| 4.7 key matrix | ✅ logic level | `gate_4_7_key_matrix_and_repeat`: letters/digits/symbols/arrows/F-keys/Escape/Enter/Tab/Backspace, press/repeat/release, shift bitmask — each also round-tripped through the real FlatBuffers envelope. The action-level form ("opens inventory" via `input_context`) needs the fork in-game. |
| 4.8 text input | ✅ logic level | `gate_4_8_text_input_wire_roundtrip`: ASCII, `café`, `☃`, Japanese — verbatim through encode/decode. |
| 4.9 focus handling | ✅ logic level | `gate_4_9_…`: `UiInput` → camera drags `Ignored`, terrain-rect clicks stay UI clicks; `MapInput` → camera drags allowed. Driven by the `UiFrame` mode flag via `InputRouter`. |
| 4.10 mouse to UI | ✅ logic level | `gate_4_10_…`: window click at 2× scale routes to the exact display-buffer coordinates of the menu entry; letterbox clicks are `Ignored`. |
| 4.11 mouse picking | ✅ logic level | `gate_4_11_picking_calibration_scene`: center clicks pick exactly (5,5,0), (8,3,0), (2,9,1); empty scene returns `hit=false`; panel clicks and `UiInput` clicks produce no `PickResult` at all (separation rule). |
| 4.12 latency | ⚠️ harness segment measured | See below. The end-to-end form (key press → screen, in-game) is blocked. |
| 4.13 end-to-end script | ⛔ BLOCKED | Requires the real fork running in-game — Phase 0 tests 0.1/0.2 (curses build + `cata_test`) have never run; they need a build-capable machine. Not faked. |
| 4.14 soak | ⛔ BLOCKED | Same dependency as 4.13. Not faked. |

### 4.12 numbers (llvmpipe, this VM — read the label)

`gate_4_12_latency_harness_segment`, 320×200 frame, 5 samples of
store-write → store-read → GPU upload → composite → readback:
**median 15.7 ms, p99 21.3 ms** (samples 14.0–21.3 ms). These are
llvmpipe software-adapter numbers on a 1.5-CPU VM, for the
engine-side segment only — they exclude game-side capture, socket
delivery, and window present, and the harness's one-shot path creates
a fresh wgpu device per frame (a resident engine pipeline will be
cheaper per frame). They are *not* target-hardware numbers and do not
by themselves pass gate 4.12; they show the segment this crate owns
is not the bottleneck risk — the in-game measurement remains to be
taken on a build-capable machine / the RX 9070 XT.

## Deviations from the plan

1. **Gate 4.1 goldens are synthetic stand-ins, not fork captures** —
   the real menu/inventory/crafting captures require the fork
   in-game (0.1/0.2 blocked). The pipeline around the goldens (store
   round-trip, masking, compositing, diffing) is fully exercised.
2. **Gate 4.3's duration is 2 s, not 10 min** — same reason; the
   writer is this crate's own store writer at max rate, not the
   game's. The detection mechanism is identical.
3. **Checksum is FNV-1a, not CRC** — the store is same-machine shared
   memory, not a wire; the checksum's job is torn/corruption
   detection, and FNV-1a over the full frame is sufficient and cheap.
   It is a private layout detail of the slots (the 48-byte header
   contract from gate 1.13 is unchanged).
4. **No new dependencies.** The scaffolded `Cargo.toml` was sufficient
   (the file-backed store uses std `FileExt::read_at`/`write_at`
   instead of a shared-memory crate); `Cargo.toml` is untouched.
5. **Picking traverses a caller-supplied occupancy predicate**, not
   the Sparse-Voxel-64Tree directly — `ui-share` may not depend on
   `treebuild`/`engine`. The engine wiring (below) supplies the
   predicate from its world model; the DDA result is the same tile
   coordinate either way.

## Engine wiring (for the parent — not done here, by ownership)

1. **Module registration:** add `ui-share` as a dependency of
   `crates/engine` (`ui-share = { path = "../ui-share" }` in
   `crates/engine/Cargo.toml` — the workspace member already exists).
   No engine source changes are needed to *build*; the steps below
   are the runtime integration.
2. **Store attach:** at startup, the engine opens the file-backed
   store the fork created (`UiShmStore::open_file(path)`, path passed
   alongside the socket handshake / `--ui-shm <path>` flag) and keeps
   one handle for the process lifetime.
3. **Frame path (per `UiFrame` socket message):**
   `validate_frame(&store, &frame)` → `store.read_latest()` →
   `masked_tight_pixels(&snap, validated.terrain_rect)` →
   upload: first frame (or any resize) = full `queue.write_texture`;
   later frames = build `UploadPlan::for_frame(w, h, validated.dirty)`
   and write only that rect (the compositor's `write_dirty_rect`
   logic moves into the resident renderer verbatim) →
   set the renderer's `InputRouter` mode from `validated.mode`.
4. **Quad pass hook (render loop):** the UI quad is the **last**
   pass of the frame, after world → fields (fx) → billboards → post:
   in `crates/engine/src/render.rs`'s frame encoder, after the
   existing world/post draws, bind the UI texture + nearest sampler,
   set the viewport from
   `ui_share::scaling::compute_viewport(window_physical, ui_size, mode)`
   and draw the fullscreen triangle (shader in
   `compositor.rs::SHADER` is directly reusable). The headless
   capture path (`capture.rs`) should run the same pass after its
   world render so captures include UI.
5. **Input hook (winit event loop, `main.rs`):** translate winit
   key/text/mouse events through `ui_share::input` mapping functions
   (window mouse coords first go through
   `scaling::window_click_to_ui` with the current viewport); send the
   resulting `InputT` as `MessageT::Input` on the IPC socket. Route
   clicks through `InputRouter::route_window_click`: `UiClick` →
   send; `WorldPick` → step 6; `Ignored` → drop. Gate camera orbit
   drags on `router.camera_drag_allowed()`. On window resize, send
   `MessageT::Resize(scaling::resize_message(logical, dpi))` when
   `scaling::resize_action` says `Reconfigure`.
6. **Picking hook:** on `WorldPick { ui_x, ui_y }`, build a
   `picking::PickCamera` from the engine's current `OrbitCamera`
   (same target/yaw/pitch/distance; FOV 60°) and call
   `pick_from_ui_click(request_id, &router, &cam, ui_x, ui_y, |x,y,z| world_occupied(x,y,z))`,
   where `world_occupied` consults the engine's chunk map (tile has
   terrain/furniture — the RT path may substitute its tree query
   later; the result contract is identical). Send the returned
   `PickResultT` as `MessageT::PickResult`.
