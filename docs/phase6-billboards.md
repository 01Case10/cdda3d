# Phase 6: Billboards — implementation record

Status: engine-side implementation complete in `crates/billboard`;
gate **6.4 is PARTIAL** (game-side half blocked, see below). All
other targeted gates pass. Nothing here is committed; `plan.md`
updates are the parent's job.

## Terms, defined on first use

- **Billboard**: a camera-facing 2D quad carrying an original
  tileset sprite. Billboards are the one place original sprites
  appear as-is in the 3D view. Field *effects* are never billboards
  (Phase 2b renders them as shader volumes); terrain, furniture and
  vehicles are voxel geometry.
- **Cylindrical billboarding**: the quad rotates about the world Y
  axis only, so it always faces the camera horizontally and never
  tilts — creatures stay upright at any camera pitch.
- **Overlay composite**: the single RGBA image produced for one
  creature by drawing its base sprite first, then each overlay
  sprite in the exact order the game returns from
  `get_overlay_ids()` (the protocol `Entity.overlays` list). The
  engine never re-sorts that list; draw order is game semantics.
- **Tile-pixel space**: the 2D coordinate system of the original
  tiles renderer. The tile cell is `(0,0)..(32,32)` for UltiCa
  (`tile_pixel_size = 32`); a sprite occupies
  `(offset_x, offset_y)..(offset_x + width, offset_y + height)`
  there, and oversized sprites (e.g. UltiCa `large`, 64x64 at
  offset (-16,-32)) extend outside the cell.
- **Premultiplied alpha**: a texture whose RGB has already been
  multiplied by its alpha. Filtering and blending premultiplied
  data cannot produce dark halos, because a transparent texel's
  color is already proportionally black.
- **Displayed field / Snapshot / Delta**: game-protocol terms from
  Phases 1/3 — not used by this phase except that entity and
  item-pile inputs mirror the `Entity` and `SubmapBlock` wire
  shapes field-for-field.

## What was built (`crates/billboard`)

| Module | Contents |
|---|---|
| `atlas.rs` | `SpriteAtlas` loading the voxelizer's exact output format (`sprites.json` = the voxelizer's `SpriteUv` list, field-for-field; `sprites.png` = the packed image). `fixture_atlas()` builds a deterministic in-crate fixture: 32x32 zombie/skeleton, three partial overlays, an oversized 64x64 sprite at offset (-16,-32), a soft-edged ghost (semi-transparent bright border), two item sprites. |
| `composite.rs` | CPU compositor (`compose`: union canvas in tile-pixel space, per-layer placement by real width/height/offset, f32 straight-alpha *source-over*), `game_draw_order` (the documented `cata_tiles::draw_entity_with_overlays` order encoded as the 6.4 reference fixture), `CompositeCache` keyed by `Signature { ordered sprite ids, facing }` with hit/miss/created counters, `Facing` (NONE/LEFT/RIGHT, wire mapping), `WeightedFrames` (weighted animation frame selection), `bob_offset`, `hit_flash`. |
| `instance.rs` | Pure instance generation: `billboard_axes` (Y-only), `composite_geometry` (world size = real pixels / `tile_pixel_size`; horizontal center offset and floor-line bottom derived from the sprite's tile-pixel rect — anchored, never centered or clamped to 32x32), `entity_instance`, `instance_corners`, back-to-front sort, blob shadows (`shadow_for`, with the under-roof suppression flag on `FakeEntity`), alpha picking (`pick_billboard` / `pick_any` against the composited alpha using real geometry), item-pile instances from `FakeItemTiles` (mirrors `SubmapBlock.top_item`/`item_count`), and the `variety_pass` missing-id report. Fake types mirror the protocol `Entity` table. |
| `render.rs` | `BillboardPass`: the reusable instanced wgpu pass (uniform + premultiplied atlas texture + sampler + storage instance buffer; cylindrical rotation computed in the vertex shader from the camera eye). Two alpha pipelines: **Blend** (premultiplied, blend `(One, OneMinusSrcAlpha)`, depth-test `LessEqual`, no depth write) and **Cutout** (discard below alpha 0.5, depth write on). Blob-shadow instanced pass (radial falloff, ground quads). `pack_composites` shelf-packs cached composites into the one texture the pass uploads. The pass only *tests* depth — it never creates or clears it — which is the interface the Phase 2c traced depth feeds (see wiring below). Plus the headless harness (fx-crate pattern: offscreen target, `compatible_surface: None`, staging readback, named `NoAdapter` error) and a box wall/window scene. |
| `scene.rs` | Fake-data scenes in wire shapes: the gate 6.11 zombie crowd (equipped zombies, one LEFT-facing, one zombie behind the wall visible only through the window, one fully hidden behind the wall, hulk, ghost, skeletons, one under-roof zombie) and the 2000-billboard stress crowd. |
| `src/bin/billboard-capture.rs` | Writes `captures/billboard_crowd.png` and `captures/billboard_stress.png`, printing llvmpipe frame times labeled as such. |

World convention matches `crates/mesher`/`crates/fx`: game
(x east, y south, z-level) → world `(x + 0.5, z * 3.0, -(y + 0.5))`;
one tile = one world unit; one z-level = 3 units (Phase 0 scale).

## Design decisions

- **Composite on the CPU, pack once, instance on the GPU.** Overlay
  composition is per-creature and changes rarely (equipment changes),
  so it belongs in a cached CPU step, not a per-frame shader layer
  stack. The GPU sees one texture rect per creature. Repacking
  happens only when the cache creates a new composite.
- **Facing is part of the cache signature** (per the task spec), so
  a LEFT composite is a mirrored image, not just a UV flip — this
  keeps picking and rendering on one geometry path. (The shader
  also supports a per-instance mirror flag for callers that pack
  unmirrored composites; the crate's own pipeline bakes it in.)
- **Alpha: premultiplied upload + sorted blend as the default.**
  Sprites are uploaded premultiplied (`render::premultiply`) and
  sampled linearly; the fragment output stays premultiplied and the
  blend state is `(One, OneMinusSrcAlpha)`. The cutout pipeline is
  kept for opaque-bodied sprites where depth-writing gives free
  occlusion; the harness crowd uses Blend with back-to-front
  sorting.
- **Shadows are a separate instanced pass**, drawn before the
  billboards in the same render pass, depth-tested, no depth
  write. Suppression is an explicit per-entity flag (`under_roof`)
  the engine derives from the tile's roof/lit state at wiring
  time — the crate does not guess indoor-ness from position.
- **Picking reuses the render geometry exactly**: the same
  `instance_corners` the vertex shader's math mirrors, then an
  alpha test (>127) on the composited image at the hit pixel.
  Oversized sprites hit over their real bounds.

## Gate results (Part II §9) — `cargo test -p billboard`, 14/14 pass

| Gate | Result | Evidence |
|---|---|---|
| 6.1 Orientation | ✅ | `gate_6_1_cylindrical_orientation_all_yaws`: 8 yaws; right/forward axes horizontal (Y-only), edges level/vertical, width/height invariant, quad faces camera. |
| 6.2 Depth sorting | ✅ | `gate_6_2_depth_occlusion_walls_and_window` (GPU, lavapipe, re-run standalone 0.55 s): entity fully behind the wall renders **byte-identical** to the empty scene; entity behind the window opening differs (visible); front entity's green present (>50 px). |
| 6.3 Alpha edges | ✅ | CPU: `gate_6_3_premultiply_no_dark_halo_math` (premultiplied composite of the ghost's 110-alpha bright border over dark bg never darkens, reads >100). GPU: `gate_6_3_gpu_soft_edges_no_dark_halo` (ghost close-up: core bright >100 px; **zero** pixels darker than the clear color). |
| 6.4 Overlay vs game | ⚠️ **PARTIAL** | Engine half ✅: `gate_6_4_engine_compositor_game_order` — jacket/jeans/knife land on torso/legs/hand, head stays base, and swapping a semi-transparent overlay's order changes the result (order is semantic). The game's documented draw order is encoded as `game_draw_order`. **Game-side half BLOCKED**: capturing `cata_tiles::draw_entity_with_overlays` in tiles mode needs the real tiles build, which is blocked on Phase 0 tests 0.1/0.2 (compile capacity). Not faked. |
| 6.5 Overlay cache | ✅ | `gate_6_5_overlay_cache_signature`: repeat signature = hit (same `Rc`), one changed overlay = exactly one new composite, reordered overlays = new signature, facing flip = new signature. |
| 6.6 Facing | ✅ | `gate_6_6_facing_mirror_only_left`: LEFT composite is the horizontal mirror (knife moves from x≈28 to x≈3); RIGHT ≡ NONE pixel-identical; wire mapping 0/1/2 verified. No other facing cases exist. |
| 6.7 Variety + items | ✅ (fixture scope) | `gate_6_7_variety_and_items`: all 6 fixture creature/item ids resolve, the one bogus id is reported missing; item piles produce exactly one billboard per tile with an uppermost item. The full-UltiCa form of this gate inherits the Phase 5 coverage audit's known missing list (plan §5 Phase 5 addendum) — it is a tileset-content gap, not a billboard-code gap, and is not re-measured here. |
| 6.8 Shadows | ✅ | `gate_6_8_blob_shadow_follows_and_suppressed_under_roof`: shadow center tracks the entity tile (+0.02 lift), moves with it, suppressed when `under_roof`. GPU crowd frame builds 9 shadows for 10 entities (1 suppressed), asserted in 6.11. |
| 6.9 Picking | ✅ | `gate_6_9_picking_alpha_and_oversized`: opaque center hits (and `pick_any` returns the entity id), transparent corner passes through, hulk hits at a point outside the 32x32 tile box but inside its real 64x64 bounds. |
| 6.10 Stress | ✅ (llvmpipe numbers) | `gate_6_10_stress_2000_billboards`: 2000 instances; compose+pack+generate **≈143–151 ms** (debug build, this VM); render+readback **≈550–680 ms** per frame at 640x360 **on llvmpipe/lavapipe — NOT target hardware** and not a budget claim. No crash; instance buffers are (re)created per `prepare` without resize logic to break. |
| 6.11 Visual scene | ✅ | `gate_6_11_visual_crowd_scene` + capture `captures/billboard_crowd.png` (640x360, 693 distinct colors, ~160 ms llvmpipe): equipped zombies (blue jacket visible), LEFT-facing zombie, hulk, ghost, skeletons, zombie visible through the window, hidden zombie absent. No committed golden image-diff exists for this phase (same status as 2.10/5.13 — captures are visually inspected evidence, not a diff gate). Stress capture: `captures/billboard_stress.png`. |

GPU tests are adapter-conditional (skip with a message if no
adapter) but were run for real here on lavapipe, including
standalone re-runs of 6.2 (0.55 s, four 640x360 renders with
byte-exact occlusion assertions — these cannot pass vacuously).

## Deviations from the plan

1. **6.4 partial** (above) — the only blocked gate; the block is
   Phase 0's compile capacity, inherited, not a Phase 6 gap.
2. **HP bars / names / status icons: not built.** Plan §5 lists
   them as "billboard overlays or left to the game's UI". They are
   left to the game's UI (Phase 4 composites it last, on top) —
   drawing a second, engine-side version would duplicate the
   game's own presentation and drift from it.
3. **Per-frame buffer recreation in `BillboardPass::prepare`** —
   harness-grade simplicity (thousands of instances; measured
   stress build is fine). A persistent, grow-only instance buffer
   is an engine-integration optimization, noted for the parent.
4. **Item billboards use the raw atlas**, not composites — items
   have no overlays, so compositing them would be a wasted cache
   entry. Creature path and item path share the instance format.
5. **Captures live in the repo-root `captures/`** directory,
   following the Phase 2/2b convention, although this phase's file
   ownership named only `crates/billboard/**` and this doc — they
   are generated evidence, not source.

## Engine wiring (for the parent)

Frame order (extends the Phase 2c hybrid contract): world/RT pass
→ **blob shadows + billboards** → field effects (Phase 2b) → HD-2D
post → UI quad. (Phase 2g's recorded order lists fields before
billboards; billboards here depth-test against world depth only
and do not interact with field volumes, so either relative order
composites correctly — pick one and record it.)

1. Add `billboard = { path = "../billboard" }` to
   `crates/engine/Cargo.toml`; keep one `billboard::render::
   BillboardPass` beside the world renderer, created with the
   engine's color format and `Depth32Float`.
2. Per Snapshot/Delta: map each protocol `Entity` to
   `billboard::instance::FakeEntity` (rename/replace that type
   with the proto type at wiring time — fields are 1:1:
   `sprite_id`, `pos`, `facing` via `Facing::from_wire`,
   `overlays` as-is in game order, `hp_frac`, `flags`;
   `under_roof` from the tile's roof/lit state at the entity's
   position).
3. Maintain one `CompositeCache` + the tileset's `SpriteAtlas`
   (loaded from the voxelizer output dir). Per frame: for each
   visible entity, `get_or_compose`; if the cache grew, repack
   with `render::pack_composites` and keep the packed image +
   rects; build instances with `instance::entity_instance`
   (tile_pixel_size from `Hello`), sort with
   `sort_back_to_front`, collect `shadow_for` results, and add
   item-pile instances from each `SubmapBlock`'s `top_item` /
   `item_count` via `instance::item_instances` (needs a
   def-id → sprite-name resolution step through TileDefs, the one
   piece the crate's fake data pre-resolves).
4. Render: after the world/RT pass has written color + depth,
   begin (or continue) a render pass with `LoadOp::Load` on both
   attachments — for RT, the color/depth views are the tracer's
   output textures — call `pass.prepare(...)` before the pass and
   `pass.draw(&mut rpass, AlphaMode::Blend)` inside it. The
   billboard pass must run **before** the fx field pass and the
   post stack, and the UI quad stays last.
5. Picking: on click, before falling back to voxel traversal,
   test billboards with `instance::pick_any` using the camera eye
   and the cursor ray; a billboard hit reports that entity's tile
   position as the `PickResult`.
6. Animation tick: pass the frame time into `entity_instance`
   (bob/hit-flash are already parameterized there); when an
   entity takes damage, set its `last_hit_s`. Weighted sprite
   animation: resolve `WeightedFrames::frame_at(time)` to the
   frame's sprite id before composing (cache signature includes
   it, so animated creatures re-composite per frame change —
   bound this by only animating on-screen creatures).
