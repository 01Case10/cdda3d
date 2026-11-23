# CDDA Voxel Frontend: Project Plan — v2 (verified)

> ## ⚠ Maintenance rule — read first
> **This plan is a living document and MUST be updated with all progress after each phase.** When a phase finishes — and whenever a phase's scope, design, or status changes mid-phase — update this document *before* starting the next phase. A phase is not done until its test gate (Part II) passes **and** this update is written.
>
> Each phase update must record:
> - **Status and date** in the Progress log below and on the phase's heading in §5 (e.g. `— ✅ GATE PASSED 2026-…` / `— 🚧 IN PROGRESS` / `— ⛔ BLOCKED: <reason>`)
> - **What was actually built**, where it differs from what §5 planned, and **every deviation and its reason**
> - **Decisions made** and open questions closed (§14) or newly opened
> - **Test-gate results** from Part II for that phase (pass/fail per gate test, known gaps with tickets)
> - **New risks, upstream changes, or rebase events** (§12), and any resulting changes to later phases' scope, estimates, or dependencies (§5, §13)
>
> If reality disagrees with this document, fix the document in the same change. Do not let the plan drift, and do not carry progress only in chat, commits, or memory — if it isn't written here, it didn't happen.

## Progress log

| Phase | Status | Updated | Notes |
|---|---|---|---|
| 0 — Recon confirmation and decisions | ✅ GATE PASSED (conditional — see Phase 0 record) | 2026-11-15 | Mandatory gate tests 0.3/0.4/0.6/0.7/0.8 pass at harness level; 0.1/0.2 not yet run — the only build they need is the curses/text build (the tiles/SDL3 build is **not required** by this project at all, updated 2026-11-15), and compiling even that is infeasible in the recon environment (1.5 CPU/6 GB) — with an exact unblock path; in-game spike halves written as a ready-to-apply probe patch. Must be run on a build-capable machine before Phase 3. **Unblock scripted 2026-11-15: `tools/pc-build/Build-CddaVoxel.ps1` (one command on Joey's PC: MSYS2 UCRT64 deps, full pinned clone, baseline 0.1/0.2 with JUnit record, then fork staging + `[voxel]` cases + 3.19 failure-set diff; hooks H1-H7 intentionally not applied -- not needed by the Catch2 gates). Awaiting its first PC run.** Full record: Phase 0 section below + `docs/recon.md` |
| 1 — Protocol and skeleton | ✅ GATE PASSED (conditional — see Phase 1 record) | 2026-11-15 | All Linux gate tests pass locally (parent-verified: `cargo test -p proto` 16/16, C++ `cpp-roundtrip` PASS, live stub↔client flow). Conditions: CI workflow written but not yet run on GitHub (no remote yet — Joey creates it); 1.5 is a fuzz-shaped sweep (cargo-fuzz not run); 1.13 is header-layout agreement only (real shm attach is Phase 4 work); 1.15 Windows not run. Full record: Phase 1 section below |
| 2 — Engine MVP with fake data | ✅ GATE PASSED (conditional — see Phase 2 record) | 2026-11-15 | Run in parallel with Phase 1; paused mid-run by Joey to add **headless capture** to scope, resumed same day with it built in. Parent-verified: mesher 8/8 + engine 16/16 tests, bench 2.12 greedy 0.397 ms/chunk (< 2 ms). **Headless rendering subsequently PROVEN on this VM** (2026-11-15, after Joey asked): `mesa-vulkan-drivers` (lavapipe/llvmpipe) was installed, and `tools/capture.sh` produced real 1280×720 PNGs of the fake house — full view (roof slab, walls, windows) and `--cutaway 0` view (interior floor, windows, door) — both visually inspected and saved in `captures/`. Remaining conditions: 2.10's formal golden image-diff (3 fixed cameras vs committed goldens) not yet established — captures are visually verified, not diff-gated **(established 2026-11-15 by Phase 2g.3 — golden harness + 7 committed goldens)**; 2.13 frame benchmark still not run (llvmpipe frame times don't represent target hardware; run on Joey's PC); windowed app compiled + lib-tested but never run live. Full record: Phase 2 section below |
| 2b — Shader field effects and HD-2D look | ✅ GATE PASSED (conditional — see Phase 2b record) | 2026-11-15 | Parent-verified: `cargo test -p voxelizer -p fx` all pass, workspace **75/75**; real UltiCa effect extraction **39 field types, 39 with sprites, 0 fallbacks**, effects.json byte-identical on re-run; lavapipe captures visually inspected (smoke/fire/gas volumes, post on/off pair — same geometry, warm grade + bloom). Conditions: 2b.8 performance is target-hardware only (not run); the fx crate is standalone — wiring it into the engine's renderer/capture path is the recorded follow-up (design in `docs/phase2b-effects.md`). Full record: Phase 2b section below. **Flame redo same day (Joey's call, against his HD-2D reference screenshots):** dedicated flame path (tapered tongues, heat ramp from extracted colors, embers), flames now light the base pass (ground pool/wall wash), night showcase `captures/effects_fire.png`; fx 10 lib + 7 gate tests pass, visually iterated ×3 on lavapipe — see `docs/phase2b-effects.md` §8. **Smoke redo same day (Joey's pick: Loop-Box's Stylized Smoke Shader, godotshaders.com, CC0):** voronoi cellular puffs + alpha-curve remap + flat dark-to-light gradient, procedural in WGSL (no textures, gate 2b.2 intact), CPU-mirrored; smoke column added over the fire-scene blaze; fx 12 lib + 7 gate pass — §9. **Smoke-over-fire coupling same day (Joey rejected §9 smoke over fire):** NOT a merged shader — per-instance `heat` (fire on/below tile: faster rise, eroded thin dark base, world-height orange underlight) + `link` (column continuity, no per-tile waists) + 4-step marched smoke shading; gradient moved to shadow range; fx 13 lib + 7 gate pass — §10 |
| 2c — Ray-traced renderer (Sparse-Voxel-64Tree) | **Done (conditional pass)** | 2026-11-15 | Added 2026-11-15 (Joey's direction): replace the raster mesher path with a compute ray tracer over a sparse voxel 64-tree ([Sparse-Voxel-64Tree](https://docs.rs/Sparse-Voxel-64Tree/latest/Sparse_Voxel_64Tree/) crate, wgpu-compatible — verified 2026-11-15). **Run attempts 2026-11-15 (×2, incl. one retry on Joey's word): both subagents were refused by a safety policy ~1 s after dispatch, before any work — no reason given, no 2c code written; not a project/code/crate failure.** Scaffolding is prepped and committed: `crates/treebuild` workspace member (stub), `Sparse-Voxel-64Tree =0.1.0` verified to resolve/download against wgpu 24.0.5. **Implemented 2026-11-15 directly in the main session on Joey's re-ask:** treebuild crate (world/tree builder/material side-table/CPU oracle/WGSL tracer) + engine `--renderer rt` (headless + windowed); gates 2c.1–2c.8 + 2c.10 ✅ (2c.4 silhouette IoU 1.000), 2c.9 ⛔ target hardware. **Raster stays the default** until 2c.9 evidence exists. See Phase 2c section + docs/phase2c-rt.md |
| 2d — Multitile layer selection (connected walls) | ✅ GATE PASSED 2026-11-15 | 2026-11-15 | Virtual-material selection: `mesher::connect` is the one canonical mask→subtile table (voxelizer delegates to it); `treebuild::connect` computes per-voxel cardinal masks (fake rule: same material) and rewrites multitile voxels to virtual material ids; the engine spec builder emits variant entries (base top/bottom, mask-selected side via `Material::side_layer_for_mask`). Gates 2d.1–2d.4 ✅ (2d.3 on lavapipe). Full record: Phase 2d section below + `docs/phase2d-multitile.md` |
| 2e — Texture mipmaps (RT renderer) | ✅ GATE PASSED (conditional) 2026-11-15 | 2026-11-15 | Mip chains in treebuild's `TextureSet` (premultiplied alpha-aware downsample + alpha-coverage rescale; trilinear sampler, no restrictive LOD clamp); ray-cone LOD per `docs/phase2e-mipmaps.md` via `textureSampleLevel` on primary + transparent hits; debug LOD view behind a uniform. Gates 2e.1–2e.4 ✅ on lavapipe (2e.3: two-pose variance 0.00 vs 143.44 simulated level-0 baseline). Full record: Phase 2e section below |
| 2f — Raster-path textures (mesher UVs) | ✅ GATE PASSED (conditional) 2026-11-15 | 2026-11-15 | Mesher emits per-face UVs + texture layers (`Block.tex_layers` in the greedy merge key); raster pipeline samples the shared scene texture spec (same builder as RT, gate 2f.3); `--textures` works with `--renderer raster`. Gates 2f.1–2f.4 ✅ on lavapipe (textured raster capture 3,187 distinct colors vs 15 flat). Condition: windowed path still flat — capture path only **(closed 2026-11-15 by Phase 2g.2 — windowed remesh now shares the capture's scene preparation and calls `set_textures`)**. Full record: Phase 2f section below |
| 2g — Integration follow-ups and coverage closure | **2g.1–2g.4 ✅ DONE 2026-11-15 (2g.4 zero-drive complete: 0 missing in terrain/furniture/monsters)** | 2026-11-15 | 2g.1 fx→engine wiring landed (field + post passes in both renderers' captures, soft-particle fade; closes 2b's follow-up), 2g.2 windowed `set_textures` landed (closes 2f's condition), 2g.3 formal image-diff gate landed (closes 2.10 + 5.13). 2g.4 batch 1 (alien meadow, 16 ids) landed: terrain missing 202→184, coverage 84.1%→85.5%. Batch 2 (string dimension, 20 ids) landed the same day: terrain missing 184→164, coverage 87.1%, player-visible 155→135; the effects-scene golden also landed (2g.3 set now 8 scenes). **Zero-drive (Joey's word, 2026-11-15): 2g.4 now runs to zero missing in terrain/furniture/monsters.** Batch 3 (regional-pseudo templates, 29 ids): terrain missing 164→135, coverage 89.4% — the non-visible terrain set is now empty. Batch 4 (nether labyrinth, 13 ids): 135→122, 90.4%. Batch 5 (floors/roofs/z-level, 22 ids): 122→100, 92.1%. Batch 6 (vitrified terrain, 16 ids): 100→84, 93.4%. Batch 7 (walls, 20 ids): 84→64, 95.0% — zero-drive wave 1 complete. Wave 2: batch 8 (triffid/reverberations/ice, 20) 64→44, batch 9 (scatter, 26) 44→18, batch 10 (manufactured/mechanisms/sunken helicopter, 18) 18→0 — **TERRAIN COMPLETE: 0 missing of 1,271 (100%)**. Furniture batch 11 (alien furniture, 19 ids): furniture missing 174→155, coverage 78.3%. Batch 12 (stack/mound family + cardboard, 51 ids): 155→104, 85.4%. Batch 13 (vitrified furniture, 32 ids): 104→72, 89.9%. Batch 14 (shores: regional/marine/decorative, 23 ids): 72→49, 93.1%. Batch 15 (remainder, 49 ids): 49→0 — **FURNITURE COMPLETE, 713/713 = 100.0%**; gate 5.10's ≥98% terrain+furniture coverage criterion met on both halves. Batch 16 (fungal zombies, 50 ids): monsters 454→404, 66.4%. Batch 17 (fish first half, 52 ids + 1 rescue): 404→351, 70.8%. Batch 18 (fish second half, 52 ids): 351→299, 75.1% — fish.json complete (104/104). Batch 19 (reptiles & amphibians, 56 ids): 299→243, 79.8%. Batch 20 (insects & spiders, 26 ids): 243→217, 82.0%. Batch 21 (nether creatures, 52 ids): 217→166, 86.2%. Batch 22 (ferals & zed clusters, 40 ids): 124→84, 93.0%. Batch 23 (mammals/birds/mutant mammals, 46 ids): 84→84, 93.0% (42 newly covered vs the folders-minus-batch baseline of 126 + 4 rescue→direct upgrades; headline holds because 42 folders had landed inside batch 22's window). Remaining monsters: 84 ids, enumerated — finisher wave next. Batch 25 (cosmic & misc tail, 42 ids): 84→42, 96.5%. Batch 24 (undead & lab tail, 42 ids; first dispatch ended empty, re-dispatched on Joey's word): 42→0, 100.0% — **ZERO-DRIVE COMPLETE: terrain 0/1,271, furniture 0/713, monsters 0/1,203 on the final audit re-run; gate 5.10's ≥98% coverage criterion met at 100% on both halves (TileDefs-form re-run still ⛔ 0.1/0.2)**. Full record: Phase 2g section below |
| 2h — Engine integration (live frame loop) | **✅ GATE PASSED (conditional) 2026-11-15** | 2026-11-15 | One composition (`engine::frame::render_frame`) now renders every frame — windowed and headless — in the canonical order world → fields → billboards → post → UI; the capture path delegates to it (all 8 pre-existing 2g.3 goldens byte-identical after delegation) and the windowed app presents it via `PresentBlit` (closes 2g.1 deviation 1). Gates 2h.1–2h.5 green on lavapipe (frame_gate.rs): windowed/capture parity 0.000%; real-UltiCa crowd occluded/revealed/picked in-engine; day/night on the Phase 7 curve, BLANK absent + MEMORIZED desaturated in both renderers; UI masked per the frame itself, click round-trips to (6, 6, 1), 4.12 in-loop median 8.23 ms / p99 36.56 ms. Conditions: live data seating (0.1/0.2), 4.12 target-hardware re-measure, windowed app not run live on the VM. Full record: Phase 2h section below + docs/phase2h-integration.md |
| 3 — Game-side export in the fork | ✅ GATE PASSED (conditional — see Phase 3 record) | 2026-11-15 | Run in parallel with Phase 5. Parent-verified harness run: ALL HARNESS TESTS PASS — 3.3 (3,888 tiles, 0 mismatches), 3.9 delta invariant (1,100 turns × 10 seeds, mirror == fresh snapshot), 3.14 hash-identical, C++ serde ↔ Rust `proto` decode PASS. The exporter module (`fork/src/voxel_ipc/`), hook patches H1–H7 and Catch2 cases are written against grep-verified real APIs but **never compiled in-game**: the full gate still requires 0.1/0.2 on a build-capable machine (3.2/3.13/3.15/3.16/3.19 blocked there). Full record: Phase 3 section below |
| 4 — UI compositing | ✅ GATE PASSED (conditional — see Phase 4 record) | 2026-11-15 | `crates/ui-share`: double-buffered shm store on the 48-byte `UiShmHeader` (seq + FNV-1a, torn frames rejected), terrain-rect masking from the frame itself, dirty-rect upload proven pixel-equal to full upload, DPI/resize math, input mapping via the real FlatBuffers envelope, DDA picking. Parent-verified `cargo test -p ui-share` 30/30. Gates 4.1–4.11 ✅ harness/logic; 4.12 segment-only measurement (stays open); 4.13/4.14 ⛔ blocked on 0.1/0.2. Full record: Phase 4 section below |
| 5 — Voxelizer and textures | ✅ GATE PASSED (conditional — see Phase 5 record) | 2026-11-15 | Run in parallel with Phase 3. Parent-verified: `cargo test -p voxelizer -p tileset` 16/16, workspace 57/57; parent's own real UltiCa run byte-identical to the agent's (`diff -r` clean): **7,717 materials, 26,389 layers, 14,450 sprites, 0 missing, 6.2 s** (sparse checkout expanded — 14,451 PNGs now on disk). Conditions: 5.10's TileDefs-based form must re-run once the fork exports real TileDefs (tileset-universe coverage is 100%); 5.13 image-diff gate not established (same status as 2.10) **(established 2026-11-15 by Phase 2g.3)**; 5.14 manual contact-sheet review ✅ signed off by Joey 2026-11-15 ("looks fine for now" — provisional; overrides can still be filed later); textured *rendering* not wired (mesher has no UV/texture-layer field — named follow-up). Full record: Phase 5 section below |
| 6 — Billboards | ✅ GATE PASSED (conditional — see Phase 6 record) | 2026-11-15 | `crates/billboard`: overlay compositor in the game's own draw order + signature cache, cylindrical instances from real sprite geometry, depth-test-only pass, blob shadows, alpha picking. Parent-verified 14/14 incl. GPU gates. Gates 6.1/6.2/6.3/6.5/6.6/6.7-fixture/6.8/6.9/6.11 ✅; 6.4 partial (game-side half blocked on 0.1/0.2); 6.10 llvmpipe numbers only. Real-UltiCa-sprite capture added on Joey's catch (the gate captures use a hermetic fixture atlas): `captures/billboard_real_crowd.png`. Full record: Phase 6 section below |
| 7 — Lighting, FOV and atmosphere | ✅ GATE PASSED (conditional — see Phase 7 record) | 2026-11-15 | `crates/lighting` (pure CPU): disjoint lit bands make brightness strictly monotonic in `lit_level` + seen; MEMORIZED pinned at 0.30 saturation-keep × 0.45 brightness; BLANK excluded from meshing and the RT tree by one shared `VisibilityMask`; air-side face sampling solves no-leak + cross-chunk smoothing together; full 132×132 bake ≈45 ms release on this VM. Parent-verified 14/14. Full record: Phase 7 section below |
| 9 — Fine-grid world (Route A prop stamping) | ✅ GATE PASSED 2026-11-17 (slices 1–3; 9.1–9.10) | 2026-11-17 | Slice 1 (parent re-verified): `mesher::fine`, `stamp_furniture`, synthetic materials; fine_gate 7/7, phase9_stamp_gate 1/1; bookcase volume 25.5× under dense. Slice 2 (parent re-verified): `mesh_chunk_fine` + boundary rule, `--fine-props` rollout, `PropScene` fixtures; phase9_render_gate 3/3 — OFF vs ON-empty **0 mismatched px**, fixture props 8,970 quads vs 210 coarse-only, boundary coverage set-equal to brute-force reference; engine 66/66, golden 3/3 unmoved. Lab captures: `captures/phase9_lab.png` (+ closeup). Slice 3 (2026-11-17, eleven micro-slices 3a–3k on Joey's word after the whole-slice dispatch was refused helper-side; all gated + committed on main in-session): 9.7 rotation (mesher + engine facing), 9.6 limit documented (`docs/phase9-fine.md`), 9.5 two-tree RT (LOD −5, `from_fine_layer` adapter, `cast_ray_two` CPU + WGSL `trace_two`, engine wiring — RT OFF vs ON-empty **0 mismatched px**, BLANK-masked props vanish exactly), 9.8 composition regression (vehicles × props, both renderers), 9.9 numbers recorded (fixture fine tree 19,998 nodes / ~130–200 ms build, ~131 KB fine layer per prop tile; lavapipe shape only). Full engine + treebuild + mesher + voxelizer suites green. **Default stays OFF** pending Joey's sign-off on the RT lab capture (9.10) and the deferred RX 9070 XT frame-time measurement (2c.9). Stage F part 2 border height-blending hold RELEASED by 9.5. Full records: §11b below |
| 10 — Skybox and sky-driven global illumination | 🚧 IN PROGRESS 2026-11-23 (10.0 spike done; Wave 1 MERGED to main @ dbbc059, all suites re-verified in the main tree) | 2026-11-23 | Plan on Joey's request. Full design in `docs/phase10-sky-gi.md`: additive `EnvironmentUpdate` wire message (game time + resolved weather numerics, string weather id — weather types are JSON/mod-defined), one `SkyState` in `crates/lighting` extending the Phase 7 `Environment`, shared-WGSL procedural sky in both renderers, hemisphere + sun/moon GI under the Phase 7 ceiling rule, fog/precip finally consumed. Slices 10.0 spike → 10.6 live seating + sign-off capture set. All six §9 decisions MADE 2026-11-23 by Joey: 1A procedural sky, 2A seasonal sun path (65°/25°), 3C dark cave gradient underground, 4A per-tile sky factor in both renderers (4B pre-agreed fallback), 5A lightning flash only, 6A stars + moon disc. 10.0 spike DONE (doc §11 — game exports real sun azimuth/altitude; sky_exposed = is_outside && !is_roofed). Wave 1 built by 3 parallel subagents in worktrees and MERGED to main 2026-11-23 on Joey's word, one at a time with suites re-run in the main tree after each: 10.1 protocol (merge 4d7fb92; proto suite + fork harness ALL PASS incl. gate 10.1b; merge initially failed cross-language tests on STALE in-tree stub-server binaries from 2026-11-15 — the proto test skips its build.sh when binaries exist; deleting them fixed it, root cause not a code regression), 10.2 SkyState (merge 4bbecd2; lighting 22/22), 10.5-early precip (merge dbbc059; fx 21+7+5 green); cargo check --workspace clean. §2.3 cadence conflict ratified by Joey: 10 game-minutes. **Wave 2 MERGED to main 2026-11-23 on Joey's word** (freeze 588f67c; 10.3 raster bfe2713 + direction fix 37e7fc8, merge 65dbcca; 10.4 RT 90101eb + direction fix 8da166a + F2 classifier rework d148173, merge 783158f). Merge conflicts resolved and recorded in the merge commits: frame.rs eye+set_sky composition; a real FLAG BIT COLLISION (FLAG_WATER vs FLAG_SKY_EXPOSED both bit 4 — sky exposure moved to bit 5/32u); treebuild shade tail composition (water spec + §5.3 + memorized desaturation); engine water Blinn now uses the GI sun dir (= legacy dir at neutral). Also fixed: camera-uniform layout visibility VERTEX|FRAGMENT (latent main bug — fragment reads camera_pos; every adapter-machine raster pipeline creation panicked; phase8_wire_gate's 3 GPU tests failed pre-merge for this reason and now pass). Ratified close-out executed on the merged tree: §6 golden regen set (reviewed signatures: raster bg-only + 58px geometry residue; RT bg exact, 8.557%/8.558% geometry), mip 2e.3 threshold >120 with §5.3 derivation (measured max 130), 2h.3 bg-mask rework (pair-identity mask; medians 0.543/0.591/0.701 inside ±0.13), 2c.4 silhouette rework (sky-mirror bg; IoU 1.000), F2 chromaticity classifier (pond green in merged tree: raster dirt 881, RT dirt 865, plain 0/1). Workspace: 105 suites green; TWO known failures, neither merge-caused: (1) puddle_water_gate raster GPU half — its frame-mean brightening threshold (>3.0) was never validated (the gate never ran on any adapter machine; per-pixel water brightening measures +55 luminance on 1,530 puddle px = +0.7 frame mean) — rework RATIFIED by Joey 2026-11-23 and landed same day (region-mean assertion: 1,530 px, water 199.9 vs plain 142.3 luminance; gate 4/4); (2) phase8_wire_gate subtile real-data half — fails identically on pre-merge main @ 40acf62 with assets linked (hardcoded layer 23635 vs the item-4 ultica_run1 regen), pre-existing. **Wave 3 (slice 10.6 live seating) BUILT 2026-11-23 in worktree `cdda-voxel-wt-wave3`, branch `p10-wave3` — NOT merged, awaiting Joey's word** (5 commits on main @ 8d64cc0: f1b40d3, ceb3d07, c587562, e76b161, d4bc8ac). The deferred wire feed is wired end to end: `TileLight.outside` + proto adapter ingest `SubmapBlock.outside`/`TileChange.outside` (absent = not exposed, the schema convention) with delta dirty-tracking; adapter converts `EnvironmentUpdate` → `GameEnvironment` field-for-field (no sanitizing in the adapter). RT sky exposure re-sourced from the Wave 2 column scan to the wire grid (§3.4 option (a) — the scan disagreed with the game on perimeter wall tiles; new sky_live_gate pins both flip directions); raster exposure derives from the same grid via `LightingScene::from_grid` (fake_house routes through it, set unchanged). `SkyScene::from_wire`/`from_snapshot` seat scenes from the wire; windowed app seats its sky at startup (`--light-hour`'s fake day, else §6 neutral — before, the sky stayed None while the bake moved) and reseats the §4 cave factor per frame from the camera target tile (`SkyScene::cave_factor_for`: below-ground + sheltered only). Fake weather environments (`GameEnvironment::fake_weather`, numerics round-trip through `classify_weather`) + one `SkyScene::named` resolver for the §8 matrix, shared by `--sky` and the gates (pre-10.6 names byte-identical, gate-pinned). **Real bug found by the capture set and fixed, failing-first pinned:** `render_frame`'s RT branch called the no-sky entry — RT frames through the live loop always rendered under the built-in neutral sky whatever `CaptureArgs::sky` said (10.4 gates called the sky entry directly); now `render_rt_frame_sky` with `SkyScene::uniforms_grid_space`, no-sky path byte-unchanged. Gate 10.6 visual set rendered (stub house + sample scene × 4 times × 4 weathers × both renderers, 320×180, sheets in the worktree's `sky-gate-10.6/`): raster 16/16 distinct frames both scenes, RT 15/16 (the one identity is DESIGNED — §5.3 overcast collapse at dawn/dusk, sun luminance ~0.11 < 0.18; pinned), luminance orderings hold (house/rt clear: noon 92.1 > dawn = dusk 30.6 > midnight 7.5; noon thunderstorm 74.7 < clear). **Acceptance = Joey's sign-off on the contact sheets — outstanding.** Workspace in the worktree: 106 suites green, golden gates green without regen (§6 holds); 3 failures, none a Wave 3 regression: (1) gate_2h3 RT MEMORIZED saturation (0.257 vs visible 0.206, bound < 0.55×) — the memorized wall run is a perimeter run, exposed under the wire feed where the scan sheltered it; `present_color` and the §5.3 composition are unchanged, the bound was calibrated on scan-fed colours — **needs Joey's ratification** (re-derive the instrument for the wire-fed scene, or a composition change §5.3 scoped out of Phase 10); (2) voxelizer gate_5_1 + (3) tileset tile_info_all_sheet_sizes — worktree checkout artifacts (golden embeds the main-tree absolute path; data file not materialized in the worktree), both pass in the main tree. Remaining: merge to main on Joey's word; the fork-side half of 10.6 (client consuming live EnvironmentUpdates over IPC) stays conditional on 0.1/0.2. |
| 8 — Hard cases and release | ✅ GATE PASSED (conditional) | 2026-11-15 | Completed 2026-11-15 in two slices, isolated from 2g.4/2h throughout. Slice 1 (8.1–8.5 model level): `crates/vehicles` (continuous facing bit-exact, absolute part positions verbatim, shortest-arc interpolation exact at endpoints, roof split, damage/removal), `crates/transient` (Effects → transient emitters in the 2b parameter space, idempotent ingest, no leftovers), `crates/hardcases` (8.4 state = id change on the protocol's own sample flow; 8.4b floor = below tile's `roof`); **schema:** TileDef additive `roof_result` (vtable 34) + fork fill + regenerated C++ headers, goldens byte-identical. Slice 2 (Joey's direction: dummy data carries the phase): vehicle render model (stamped tiles == truth at 23°, roof composes/slabs, clear restores air), `crates/release` — 8.6 overmap flow on the Phase 4 store (UiInput validates, UI routing, MapInput restores picks), 8.7 mod fixture through all stores + exact missing-material report, 8.8/8.9 soak (7,200 steps; RSS +128 KiB after warm-up; no cost creep), 8.10 perf (vehicle delta 11.37 µs/20 vehicles, ingest 3.21 µs — release, this VM), 8.11 Linux package smoke PASS from clean dir/env (`tools/package_smoke.sh`), 8.12 stamp-invalidated cache + resync, 8.13 versioned crash report (frontend + protocol + pinned fork commit). **Conditions (the complete blocked list):** in-game halves ⛔ 0.1/0.2 (compute capacity, standing since Phase 0); 8.11 Windows half (no Windows here); frame-time budgets/2c.9 (RX 9070 XT); 8.14 manual playthrough. Engine wiring of the new crates: **done 2026-11-15** (frame-loop follow-up addendum under Phase 2h). Full record: Phase 8 section below + `docs/phase8-hardcases.md` |

A voxel-rendered frontend for Cataclysm: Dark Days Ahead. The unmodified-in-spirit game (a lightly forked CDDA) runs all simulation. A separate Rust engine renders the world as voxels and talks to the game over IPC. Characters and monsters stay as 2D billboard sprites (Doom-style). All art comes from [I-am-Erk/CDDA-Tilesets](https://github.com/I-am-Erk/CDDA-Tilesets), and voxel textures are derived from those sprites.

> **What changed in v2:** every item the original plan marked **(verify)** has been checked against the actual source, and the plan has been corrected where the source disagreed. Verification was done 2026-11-15 against:
> - CDDA `master` @ `6a3e59e08842de34ff3f111c7f098abeffb708a5`
> - CDDA-Tilesets `master` @ `5ee942f9e6601970c5da356200e7cf5061226262`
>
> Full evidence — file, symbol, and verdict for each claim — is in **Appendix A**. Corrections are also folded into the body, so the body is safe to follow on its own. The three corrections that change the design, not just details:
>
> 1. **Fields and items are not single ids per tile.** A tile's field is a *map of field entries* (type + intensity + age), and its items are a *colony (list) of items*. The v1 protocol's `fields:[uint32]` / `top_item` would have silently lost smoke density, fire intensity, and item stacks. (§6)
> 2. **There is no per-tile "map changed" hook to tap.** CDDA's invalidation is coarse — per-z-level and per-submap dirty bits (`invalidate_map_cache(z)`, `set_seen_cache_dirty`, `set_transparency_cache_dirty`). Delta export therefore becomes: hook those to collect *dirty submaps*, then diff those submaps. Budget for the diff as the primary mechanism, not the fallback. (§5 Phase 3)
> 3. **UI compositing is easier than feared — in a specific way.** The game code draws UI as many separate `catacurses` windows (terrain window + a configurable panel stack + a `ui_adaptor` stack for menus), *but* the SDL3 backend already composites everything into a single terminal-sized `SDL_Texture` (`display_buffer` in `src/sdltiles.cpp`) before presenting. Phase 4 is now a spike between two concrete routes, not an open question. (§5 Phase 4)

---

## 1. Goals and non-goals

### Goals
- Play the full, normal CDDA game through a 3D voxel view with a free/follow camera and z-level cutaway.
- Keep the game the single source of truth: no simulation logic in the engine.
- Characters, monsters, NPCs and items render as camera-facing 2D sprites from the existing tilesets.
- Terrain, furniture, walls and vehicles render as voxels with textures generated from tileset sprites.
- Keep the game's existing menus, inventory, crafting and message UI working, composited over the 3D view.
- Work with mods without per-mod engine changes (data-driven via a tile-definition table).

### Non-goals (initially)
- Replacing the game's menus with a native 3D/GUI UI (possible later).
- Multiplayer.
- A physically accurate world scale; the look is stylized.
- Mobile/web targets.
- Overmap in 3D (stays 2D at first).

---

## 2. Architecture

```
+------------------------------+        local socket (messages)         +-----------------------------+
|  cdda-fork (C++)             | <-------------------------------------> |  voxel-engine (Rust)        |
|                              |                                          |                             |
|  - full simulation           |  handshake, tile defs, snapshots,        |  - window, GPU (wgpu)       |
|  - map, z-levels, vehicles   |  deltas, entities, input, UI events      |  - ray tracing (64-tree)    |
|  - FOV / lightmap            |                                          |  - billboards               |
|  - UI -> offscreen RGBA -----+---- shared memory (UI texture) --------->|  - lighting / fog           |
|  - input_context             | <-- key / mouse / resize events ---------|  - UI compositing, input    |
+------------------------------+                                          +-----------------------------+
```

- **Engine owns the window and GPU.** It launches the game as a child process and is the IPC client; the game is the server.
- **Game owns all rules and all UI layout.** Menus are rendered by the game into a shared offscreen buffer; the engine draws that buffer over the 3D scene and forwards raw input back.
- **Turn-based means low bandwidth — with one exception.** Monsters, NPCs, weather, fields and animations also change *between* player inputs, and the game redraws on its own schedule. Deltas are therefore sent on a fork-side "world changed since last send" basis (§6 sync), not strictly per player turn. The UI texture is the only large payload and it goes through shared memory, not the socket.

### Fallback
If compositing the UI proves too painful, switch to an in-process integration: build the engine as a Rust static library with a C ABI and link it into the fork, replacing the map-draw path. The snapshot/delta API designed here is reusable either way.

---

## 3. Repositories and layout

```
cdda-voxel/
  plan.md
  Cargo.toml                  # workspace
  crates/
    proto/                    # FlatBuffers schema + generated Rust
    engine/                   # binary: window, render loop, IPC client
    mesher/                   # chunk -> mesh (greedy meshing), no GPU deps
    voxelizer/                # offline tool: tileset -> atlas + material table
    tileset/                  # tile_config.json / sprite loader (shared)
    ui-share/                 # shared-memory UI texture reader
  schema/
    cdda_ipc.fbs
  fork/                       # git submodule: our CDDA fork
  assets/                     # generated output (gitignored)
  docs/
```

Fork branch strategy: `voxel-frontend` branch off CDDA `master`. Keep changes isolated in new files (`src/voxel_ipc/*`) plus small, well-marked hooks, so rebasing onto upstream stays cheap.

**Pin, don't float.** Record the exact upstream commit the fork branched from in `docs/recon.md` and rebase deliberately. Upstream is actively refactoring the exact areas this project touches (the tiles renderer, smooth lighting and the SDL3 backend all changed shape recently), so "rebase onto master whenever" is a risk, not a routine. See §12.

---

## 4. Licensing

- CDDA code and content, and the Tilesets repo, are released under **CC BY-SA 3.0** — verified in `LICENSE.txt` at the root of *both* repos.
- That is not the whole story: both `LICENSE.txt` files list third-party exceptions with their own licenses (fonts under OFL/Apache/CC BY, Catch under Boost, PLF colony under zlib, libbacktrace under BSD, and more). The fork's attribution file must reproduce those notices, not just the CC BY-SA header.
- The fork must remain CC BY-SA 3.0. Textures derived from the sprites are derivative works and must also be share-alike with attribution to the original tileset authors.
- Base art on **UltimateCataclysm** (folder `gfx/UltimateCataclysm`, display name **UltiCa** — it is the default tileset) and optionally **Altica** (display name AltiCa) and **Chibi_Ultica** for characters. The character set the original plan called "MSX+" is folder `gfx/MShockXotto+`, abbreviated **MSX** in the Tilesets README.
- Avoid the Mushroom Dream tileset (`gfx/Mushroom-Dream`): verified in the Tilesets README — only a small stub hosted there is CC BY-SA; the complete version is under a more restrictive license via the author's Patreon.
- The Rust engine can be licensed separately if it contains no CDDA code or assets, but the generated asset pipeline output is share-alike. Decide and document this in Phase 0.
- Keep an `ATTRIBUTION.md` generated by the voxelizer listing tileset sources, per-sprite source folders where the tileset provides them (UltiCa's `tile_info.json` carries per-sheet `source` entries, e.g. for filler and fallback sheets), and the third-party notices above.

---

## 5. Phases

### Phase dependencies and parallelism

Read this before scheduling. **Hard dependency** = that phase's gate must pass before this phase can pass its own gate. **Soft dependency** = needed only for the gate; most of the work can start earlier against stubs/fake data. Phases in different codebases (C++ fork vs Rust engine vs offline voxelizer) parallelize best.

| Phase | Hard dependencies | Soft dependencies | Can run in parallel with |
|---|---|---|---|
| 0 — Recon | — (starts immediately) | — | Nothing; everything waits for its gate. Only repo/CI scaffolding may start alongside |
| 1 — Protocol and skeleton | **0** (schema, scale and headless decisions come from recon) | — | **2** — once a draft schema exists, engine work on fake data need not wait for the Phase 1 gate |
| 2 — Engine MVP (fake data) | **0** (world scale, chunk size) | 1 — final world/chunk format for its gate; mesher and camera work need only Phase 0 | **1, 3, 5** |
| 2b — Shader field effects / HD-2D | **2** (renderer + headless capture to composite over), **5** (sprite-derived effect parameters from the voxelizer) | 3 — real field export; fake field tiles suffice during development | **3, 4, 5** — engine-side; its post stack is renderer-agnostic and carries into **2c** |
| 2c — Ray-traced renderer | **2** (world model, camera, capture harness) | 5 — materials/texture array for the material side-table; flat-color tree rendering needs only Phase 2 · 3 — real snapshots; fake house suffices during development | **3, 4, 5, 2b** — engine-core work on fake data |
| 2d — Multitile layer selection | **2c texture hookup** (f4c1c45), **5** (`subtile_layers` data) | 3 — connect-group semantics for real data; fake data uses same-material adjacency | **2f** (same per-voxel layer choice feeds the mesher), 3, 4 |
| 2e — Texture mipmaps (RT) | **2c texture hookup** (f4c1c45) | 5 — voxelizer's alpha-aware averaging rules for the downsample | 2d, 2f |
| 2f — Raster-path textures | **2** (mesher + raster pipeline), **5** (materials) | 2c texture hookup — its scene-local spec builder is reused · 2d — shared layer selection | 2d, 3, 4 |
| 2g — Integration follow-ups + coverage closure | **2b, 2c, 2f** (fx wiring, windowed textures), **2, 5** (image-diff goldens), **5** + coverage audit (tile creation) | 4 — UI-quad ordering for the post pass; wiring can proceed against the current frame layout | **4, 6, 7** — 2g.1/2g.2 share `crates/engine` render files with 4 and 6 (coordinate ownership or sequence after 4); 2g.3 (tests) and 2g.4 (`coverage-audit/`) are conflict-free |
| 2h — Engine integration (live frame loop) | **2g.1, 2g.2** (fx passes + windowed textures in the engine), **4** (ui-share), **6** (billboard), **7** (lighting) — the crates it seats | 3 — live game data; the loop is built and gated on stub/snapshot data, live seating follows 0.1/0.2 | **2g.4** (conflict-free), Phase 8 sub-tasks that don't need the loop |
| 3 — Game-side export (fork) | **0, 1** (hook points + protocol to export over) | — | **2, 5** — different codebase (C++ fork vs Rust), the main parallel track |
| 4 — UI compositing | **3** (fork scaffolding: `--frontend=ipc`, `voxel_ipc` server, input injector) | 2 — an engine window/quad to composite onto; engine-side compositing can start earlier against Phase 1 stub UI frames | **5**; engine-side parts alongside **3** |
| 5 — Voxelizer and textures | **0** (tileset study) | 1 — TileDefs format for materials keyed by def; the voxelizer can start from game JSON + composed `tile_config.json` before the exporter exists | **2, 3, 4**, and the sprite-atlas parts of **6** |
| 6 — Billboards | **2** (rendering/instancing), **5** (sprite atlas) — for its gate | 3 — real entity export; billboards can be developed against fake entities from Phase 2 | **7** |
| 7 — Lighting, FOV, atmosphere | **2, 3** (`lit_level`/seen export to consume) — for its gate | 5 — works on flat colors before textures land | **6** |
| 8 — Hard cases and release | **4, 5, 6, 7** for the release gate (playable + textured + billboards + lighting) | Individual hard cases only need their own inputs: vehicles and animations need **3** (+ **4** for effects-in-UI checks) | Sub-tasks inside Phase 8 (vehicles, animations, packaging prep) are parallel with each other |

**Critical path to playable (M4): 0 → 1 → 3 → 4.** Phase 3 is the long pole. The texture path **0 → 1 → 5 → 6** and engine path **2** run alongside it, which is why M5/M6 in §13 land close behind M4 instead of a full phase-duration later. Phase 7 slots in as soon as Phase 3's light/visibility export is stable, in parallel with 6. **Phases 2b/2c (added 2026-11-15)** are engine-track work in the Phase 2 family: 2b (shader effects + HD-2D grade) and 2c (ray-traced renderer) both run on fake data in parallel with 3/4/5; neither is on the critical path to playable, but 2c's gate decides which renderer is the default from then on, and 2b's post stack is shared by both renderers. The cumulative times in §13 assume these tracks actually run concurrently (multiple developers, or one developer interleaving); a strictly serial single-developer run takes roughly the sum of the individual phase estimates.


Time estimates assume one developer working full time; scale accordingly. Phase 0 is shorter than in v1 because the source-reading part is already done — Appendix A *is* most of the recon output. What remains is confirming the hook points in a build, and two spikes.

### Phase 0: Recon confirmation and decisions (1 week) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** none — starts immediately. **Blocks:** every other phase's gate. **Parallel with:** nothing (repo/CI scaffolding only).

**Already answered by source verification (Appendix A) — do not re-litigate:**
- World constants: submap = 12×12 (`SEEX`/`SEEY`), reality bubble = 11×11 submaps = **132×132 tiles** (`MAPSIZE`), max view distance = **60 tiles** (`MAX_VIEW_DISTANCE`), z-levels = **−10…+10, 21 layers** (`OVERMAP_DEPTH`/`OVERMAP_HEIGHT`), 3D FOV z-range = 10 (`fov_3d_z_range`). All in `src/map_scale_constants.h`.
- Exact map draw order (11 layers) and entity overlay order source. (§6, Phase 6)
- Where UI is drawn: `game::draw(ui_adaptor&)` → `draw_ter()` into `w_terrain`, then `draw_panels()`; menus are separate `ui_adaptor`s managed by `src/ui_manager.*`. Everything is composited by the SDL3 backend into one `display_buffer` texture.
- Delta tracking: only coarse invalidation exists — dirty-submap collection + diff is the design.
- Build system: **Makefile is primary; CMake also works.** CMake options: `TILES` (default **OFF**), `CURSES` (default ON). The tiles backend is **SDL3 only** — SDL2 support was removed upstream. There is no single "tiles vs ncurses" binary choice at runtime; they are separate builds.

**Tasks**
- Build the fork's curses (text) version locally; confirm toolchain. **The tiles (SDL3) version is not built and not needed** (updated 2026-11-15): the shipped fork is the text build + null `catacurses` backend — the engine does all graphics — so SDL3 is out of scope for this project entirely.
- Read and annotate, with the corrected file list:
  - `src/cata_tiles.cpp/.h`: map and sprite drawing, the 11-layer order, `height_3d` pseudo-3D offsets, `looks_like` and season-variant resolution
  - `src/sdltiles.cpp/.h`: the `display_buffer`, present path, coordinate conversion (`window_to_display_buffer_coords`)
  - `src/cursesport.cpp/.h`, `src/catacurses.h`: the `catacurses::window` abstraction all UI drawing passes through
  - `src/map.h`, `src/submap.h`, `src/mapdata.h`, `src/map_scale_constants.h`: terrain/furniture/field/item storage (see §6 for the actual shapes)
  - `src/lightmap.h`, `src/map.cpp` (`build_seen_cache`, `update_visibility_cache`): FOV and light — the exportable product is the per-tile `lit_level` enum plus float seen/light values, not a raw lightmap
  - `src/vehicle.h/.cpp`, `src/vehicle_part.h`: part layout; `face` is a `tileray` (continuous `units::angle`), mounts are pre-rotated by `precalc_mounts`
  - `src/ui_manager.*` (`ui_adaptor` stack), `src/output.*`, `src/panels.*`, `src/widget.*`: UI rendering path — note there is **no `src/ui.*`** (that file in v1 does not exist)
  - `src/input.h` (`input_manager`, `input_event`) and **`src/input_context.h`** (`input_context` lives there, not in `input.*`)
  - `src/main.cpp`, `src/game.cpp`: main loop; `game::do_turn()` and the `ui_manager::redraw()` call sites are the candidate frame/turn boundaries
  - `src/animation.h/.cpp`: animation dispatch — its curses/tiles function variants are the natural hook for forwarding effects to the engine
  - Tileset loading and the composed `tile_config.json` format; `tools/gfx_tools/compose.py` (in the CDDA repo) which generates it from the Tilesets repo's per-sprite files
- Study the Tilesets repo layout: per-sprite PNG + tile-entry JSON under `gfx/<Tileset>/pngs_*/**`, `tile_info.json` (sheet list with per-sheet sprite size and offsets), multitile entries, overlays, filler sheets. Note the composed `tile_config.json` is a *build artifact* — it is not in the CDDA repo checkout (checked: 404 on master), so the pipeline must run `compose.py` or consume a release build's output.
- **Spike A (UI):** prove one Phase 4 route end-to-end at prototype quality (§ Phase 4).
- **Spike B (headless):** decide how the game runs without its own window. **Decided in Phase 0 (2026-11-15): option (2)** — a null/custom `catacurses` backend in a curses build that renders UI windows to an offscreen buffer and never draws the map. Option (1) (tiles build with the SDL window hidden) is **dropped**: it exists only to serve the tiles build, which this project does not need. Do not assume an SDL "dummy video driver" gives you a working renderer — that assumption is exactly what option (2) avoids.
- Decide: world scale (default 1 tile = 1 block, 1 z-level = 3 blocks tall), chunk size (12×12 tiles to match submaps), engine licensing, FlatBuffers vs protobuf.

**Deliverables**
- `docs/recon.md`: Appendix A plus the two spike results, hook points, and decisions.
- A list of the exact functions where snapshot data is read and where the UI draw can be redirected.

**Exit criteria**: written answers to: where is a safe turn boundary, how do we read visible tiles per z-level, how do we redirect UI drawing to a texture — each backed by a running spike, not just source reading.

> #### Phase 0 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** the `cdda-voxel` project scaffold (this plan, `docs/`, `tools/phase0/`); sparse blobless checkouts of both pinned upstreams (CDDA `6a3e59e0`, Tilesets `5ee942f9` — commits verified by `git rev-parse`); a machine-extracted hook-point catalogue (`docs/hook-points.md`, **32/32 found**); a runnable Phase 0 harness (`tools/phase0/run_all.sh`: source probes + a compiled C++ UI-capture spike + headless runner); `docs/recon.md` (exit answers, spike results, decisions) and `docs/licensing.md`.
>
> **Exit answers (detail in recon §1–2):** send boundary = `game::do_turn()` exit + `game::draw()` redraw point; visible tiles read via the named `map::ter/furn/tr_at/field_at/i_at` + `submap` uppermost-item + `apparent_light_at` after `update_visibility_cache`; UI redirect = **Route B (catacurses-level capture) as the primary route**, Route A (`display_buffer` capture) as fallback; headless = **null `catacurses` backend**.
>
> **Decisions made:** world scale 1 tile = 1 block, 1 z-level = 3 blocks; chunk = 12×12 submap column × cutaway window; **FlatBuffers**; engine license **MIT OR Apache-2.0** dual, fork CC BY-SA 3.0, derived assets CC BY-SA 3.0; first tileset **UltiCa only**; fork base **pinned `master` @ 6a3e59e0**; v1 platform **Linux first** (§14 questions 4/5 narrowed accordingly).
>
> **Deviations and why:**
> - **`game::do_turn()` is defined in `src/do_turn.cpp:536`, not `src/game.cpp`** (the plan named game.cpp). Hook catalogue and Phase 3/9 file lists corrected — this is also new risk R2 below.
> - **`map::furn()` takes `tripoint_bub_ms` by value** (map.h:1020), and **`get_uppermost_item()` lives on the submap maptile** (`submap.h:439`), not on `map`. Exporter signatures in Phase 3 must use the real ones.
> - **Test 0.9 corrected:** no decomposed tileset entry carries all eight multitile slots; entries carry the **6 neighbour subtiles** (max found, e.g. `f_metal_bench`), and `open`/`broken` are state-variant ids — consistent with test 5.5's note. The probe asserts the verified shape; the test-plan row is updated.
> - **Spikes ran as standalone harnesses, not in-game**, because the fork cannot build in the recon environment (below). The in-game halves are written as an exact probe patch (`tools/phase0/fork_probe_patch.md`) so they can be applied unchanged on a build machine.
>
> **Test-gate results (Part II §3):** 0.3 ✅ (harness + source sites), 0.4 ✅, 0.5 ✅, 0.6 ✅ (both routes prototyped; Route A CPU-side readback floor 0.39 ms mean at 1600×800 — GPU stall still to measure in-game), 0.7 ✅, 0.8 ✅ (suite + 50 scripted turns, DISPLAY unset), 0.9 ✅ with the correction above (compose.py *run* deferred: needs pyvips + the full UltiCa PNG tree), 0.10 ✅. **0.1 ⛔ NOT RUN HERE, 0.2 ⛔ NOT RUN HERE** — and the scope is smaller than first recorded (updated 2026-11-15): the tiles/SDL3 build is **not part of 0.1 anymore** (the fork never needs SDL3; see the clarification below), so the only outstanding build is the curses/text build plus `cata_test`. Its configure stops at the Curses check in this environment (dev package installable, but a ~1,500-TU compile is not feasible at 1.5 shared CPUs / 6 GB). Unblock: Joey's PC or a ≥4-core/≥16 GB CI runner — **no SDL3 needed on the target machine** — see recon §6 for the exact commands.
>
> **Gate verdict: CONDITIONAL PASS.** The gate's mandatory tests pass at the level this environment can prove and every Phase 0 decision is recorded, so Phases 1, 2 and 5 (which work against the schema/stubs) may start. **Condition:** 0.1/0.2 and the in-game spike halves must pass on a build-capable machine before Phase 3 starts; if they contradict this recon, Phase 0 reopens.
>
> **Clarification (2026-11-15, Joey's question):** the fork does **not** need SDL3 at all. SDL3 is only the original game's graphics library, linked only by the *tiles* build (`CMakeLists.txt` searches SDL3 only under `if(TILES)`). The shipped fork is the text/curses build + null `catacurses` backend (Route B) — the engine does all graphics. The tiles build in test 0.1 was only an unmodified-upstream baseline and Route A's home; with Route A demoted to fallback, **the SDL3 half of the 0.1 block is cancelled**. The only build the project needs is the curses/text build (+ `cata_test`); what remains blocked in the recon environment is purely compile capacity.
>
> **New risks / knock-on changes:** R1 — build environment is a first-class dependency: standing up the `fork-build-and-test` CI job on a capable runner is now explicitly **Phase 1 scope**, not a Phase 3 discovery. R2 — upstream already split `do_turn` out of `game.cpp`; Phase 3/§9 hook call-sites now name three files (`do_turn.cpp`, `game.cpp`, `sdltiles.cpp`) and all hook logic stays in `src/voxel_ipc/`. R3 — Phase 5's pipeline inherits the compose.py requirement (full Tilesets checkout + libvips on the build machine). No other change to later phases' scope, estimates, or dependencies; the critical path 0 → 1 → 3 → 4 stands.

---

### Phase 1: Protocol and skeleton (1 week) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 0 gate. **Parallel with:** Phase 2 may start on the draft schema before this gate passes.

**Tasks**
- Write `schema/cdda_ipc.fbs` (see section 6).
- Create the Rust workspace and the `proto` crate with codegen.
- Implement a stub C++ server (outside the fork) that sends a fake snapshot (a small procedural house) and listens for input.
- Implement the Rust client: connect, handshake, receive snapshot, log it.
- Define versioning rules: bump `protocol_version` on breaking changes; unknown fields ignored.

**Deliverables**: engine connects to stub server and prints the snapshot; round-trip input event works.

**Exit criteria**: CI builds both sides and a test verifies schema compatibility.

> #### Phase 1 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `schema/cdda_ipc.fbs` (Envelope union of all 14 message types from §6, sparse `FieldOnTile` + `field_index`, `lit:[LitLevel]`, facing NONE/LEFT/RIGHT only, vehicle `face_deg` float + absolute part positions); `crates/proto` (length-prefixed framing with a 32 MiB cap rejected from the prefix alone, verified decode, handshake, `SeqTracker`, `HeartbeatTracker`, `UiShmHeader`, `proto-client` bin; flatbuffers pinned `=24.3.25`, codegen via flatc in `build.rs`); the C++ stub server in `tools/stub-server/` (vendored flatbuffers headers, checked-in generated header, procedural-house snapshot, input echo); `docs/protocol.md` (versioning rules); `.github/workflows/ci.yml` (Linux: flatc download, C++ header regen-diff, stub build, C++ round-trip, `cargo test -p proto`, fmt). Ran in parallel with Phase 2 as §5 allows.
>
> **Test-gate results (Part II §4), parent-verified by re-running the gates:** 1.1 ✅ (all messages + 200 randomized), 1.2 ✅ (`cpp-roundtrip` PASS), 1.3 ✅ both directions (C++ goldens ↔ Rust, semantic equality + committed goldens in `crates/proto/tests/golden/`), 1.4 ✅ with the nuance below, 1.5 ⚠️ partial (fuzz-shaped sweep: every single-byte flip + 20k mutations, no panic; cargo-fuzz itself not run), 1.6 ✅, 1.7 ✅, 1.8 ✅, 1.9 ✅ (real v2-schema encode → v1 decode), 1.10 ✅ (pure logic, clock-injected), 1.11 ✅ (live stub, both directions), 1.12 ✅ (live Unix socket), 1.13 ⚠️ partial (48-byte header, Rust asserts + C++ `static_assert`s agree; no cross-language shm attach — that is Phase 4 work), 1.14 ✅, 1.15 ⛔ not run (Linux-first per §14). Totals from the parent's own run: `cargo test -p proto` **16/16**.
>
> **Findings / deviations:** (1) Stock flatbuffers 24.3.25 `root()` does **not** check the file identifier and a flipped root offset could land on a valid empty envelope — `decode_message` now checks the `CDDA` identifier explicitly and rejects `MessageT::NONE`. Two inherent FlatBuffers limits are documented in the tests: a content-byte flip inside a string can verify with altered data (no checksum exists), and the builder's trailing ≤3 alignment-padding bytes are unreferenced, so truncating only those yields the identical message (the test asserts error-or-identical there, hard error elsewhere). (2) `displayed_field` is a `[DisplayedField]` struct vector, not packed `[uint32]` — lossless, no packing ambiguity. (3) "No transform" for open/close/bash/`looks_like` is an **absent** field, not `""` — the C++ object API packs `""` as absent, so absent is the only cross-language-representable form; documented in `docs/protocol.md` (absent ≡ empty). (4) `TileDef` also carries `rotate_to_groups` (named in Phase 3's exporter text).
>
> **Gate verdict: CONDITIONAL PASS.** Everything runnable on Linux in this environment passes; the conditions are: the CI workflow has not yet run on GitHub (no remote exists yet — Joey creates it, then CI must go green there), cargo-fuzz (1.5 full form) runs as a nightly job once CI exists, and 1.13's real shared-memory attach lands with Phase 4. Phase 3 may start on this gate **subject to Phase 0's own condition** (0.1/0.2 on a build-capable machine).

---

### Phase 2: Engine MVP with fake data (1-2 weeks) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 0 gate · soft — Phase 1 (final chunk format, for its gate only). **Parallel with:** Phases 1, 3, 5.

**Tasks**
- `winit` + `wgpu` window, depth buffer, resize handling.
- Chunk data structure and greedy mesher; flat color per terrain id.
- Camera: follow-player with orbit, zoom, pitch limits; z-level cutaway (hide everything above player z + N, or fade roofs).
- Basic frame pacing (render only on change or when camera moves; the game is turn-based, but camera movement and effects are not).
- Debug overlay: chunk borders, mesh stats, fps.
- **Headless capture mode (added 2026-11-15, Joey's direction — the CDDA Voxel analogue of the Godot Xvfb capture workflow):** the engine must support offscreen rendering with no window and no X server — `cargo run -p engine -- --headless-capture --snapshot <stub-house | snapshot-file | stub-server> --camera <x,y,yaw,pitch> --size <WxH> --out <png>`. Inside that mode: create the `wgpu` instance/adapter/device with `compatible_surface: None` (no `winit` window, no surface), render to an offscreen `Rgba8UnormSrgb` texture (`RENDER_ATTACHMENT | COPY_SRC`) plus a depth texture using **the same pipelines and the exact same chunk meshes as the windowed path**, `device.poll(Wait)`, copy texture → staging buffer, map it, write a PNG via the `image` crate, and exit — no event loop. It feeds on Phase 2 fake data now and on real `Snapshot`/`Delta` data later (Phases 3+). The game side needs no equivalent: it is already headless by design (Phase 0, null `catacurses` backend) and its UI is already a CPU RGBA buffer whose PNG dump was proven in the Phase 0 spike (`tools/phase0/spike_out/spike_ui.png`).
  - **Environment split (verified 2026-11-15 on the recon VM):** that VM initially had **no `/dev/dri`, no Vulkan ICD, and `mesa-vulkan-drivers` not installed**, so `wgpu` had no adapter there at all — the same class of limit as Godot 3D under Xvfb on that machine, not an engine bug. **Later the same day `mesa-vulkan-drivers` was installed on it and lavapipe captures succeeded** (see the Phase 2 record addendum) — so route (b) below is proven, with the expected speed caveat (~2.6 s for one 1280×720 small-scene frame at 1.5 CPUs). Therefore: (a) CI gates on adapter-less machines rely on mesh/snapshot hash tests and UI-buffer PNG dumps, **not** pixel captures; (b) pixel captures require either `mesa-vulkan-drivers` (lavapipe, software Vulkan — real but slow; small scenes only, a full 132×132 capture will crawl) or running the capture binary on Joey's PC (RX 9070 XT — the actual target hardware, and fast). The capture code must fail cleanly with a named "no adapter" error when no adapter exists, never panic or emit a blank PNG presented as a pass.

**Deliverables**: fake house rendered as colored cubes, navigable camera, z-slider; **headless capture binary mode (above) plus `tools/capture.sh`** wrapping it (scene/camera/size/out arguments, non-zero exit on no-adapter).

**Exit criteria**: 60 fps rendering a **132×132-tile area × the z-levels in the cutaway** (the real bubble footprint — v1's "60×60×10" understated the width by >2×; 60 is the view *radius*, not the width) on mid-range hardware.

> #### Phase 2 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `crates/mesher` (pure Rust: 12×12 chunk columns, naive + greedy meshers, cross-chunk culling via a sampler, pass classification, dirty propagation, `WorldScale` config); `crates/engine` (camera with pitch clamp [20°, 85°], z-cutaway layer selection, fake-house world — ground + wood floor, wall perimeter, south door, 3 windows, roof-slab layer —, shared `render::Renderer` used by both the winit/wgpu windowed app and the headless path, debug-overlay state, render-on-change pacing, resize decision logic); **headless capture** exactly as scoped mid-phase (§5 task above, §7): `--headless-capture` offscreen mode + `tools/capture.sh`. Ran in parallel with Phase 1.
>
> **Test-gate results (Part II §5), parent-verified by re-running the gates:** 2.1–2.6 ✅ (mesher gate tests), 2.7 ✅ (camera lib tests incl. NaN/∞-corrupted inputs still yielding finite matrices), 2.8 ✅ (cutaway at z = −10/+10 and out-of-range ±100), 2.9 ✅, 2.10 ⚠️ partial — capture deliverable built and its adapter-less behavior proven (parent's own run: `tools/capture.sh …` → exit 2, stderr `headless-capture: no wgpu adapter available`, no PNG written); the pixel comparison itself is **not run** (no adapter on this VM) and its adapter-present branch has never run anywhere yet, 2.11 ✅ at lib level (`resize_action`: zero-size/unchanged → Ignore, real change → Reconfigure; the bin wires exactly this; a live window resize was not exercised — no display here), 2.12 ✅ (parent's own bench run: greedy **0.397 ms/chunk**, naive 0.300 ms/chunk, budget 2 ms), 2.13 ⛔ not run (no GPU) — data proxy instead (parent's own `--scene-report` run: 121 chunks, 17,424 tiles stored, 3 z-levels, 27,954 non-air blocks → 5,132 quads / 20,528 vertices, ~1,288 ms total meshing in a *debug* build, ~10.6 ms/chunk unoptimized — not comparable to the optimized 2.12 bench). Totals from the parent's own run: mesher 8/8, engine 16/16, workspace 40/40.
>
> **Decisions / deviations:** floor/ceiling (§14 Q1) = **thin slabs** — floor is the bottom 1 of a z-level's 3 blocks, roof the top 1, walls full height; a `WorldScale` config value, not a constant. Debug overlay renders as a window-title summary, not in-scene GUI; the chunk-borders toggle is state-only (border geometry not drawn in the MVP). Cutout pass shares the opaque pipeline (fake-data cutouts are fully opaque; alpha testing arrives with Phase 5 textures). Follow-target WASD moves in world axes, not yaw-relative. The windowed app has never run live (no display/adapter here) — first live run is on an adapter machine.
>
> **Gate verdict: CONDITIONAL PASS.** All data-level gates pass on this machine. **Conditions:** 2.10's pixel comparison and 2.13's frame benchmark must pass on an adapter-equipped machine (Joey's PC with the RX 9070 XT is the natural target — it is also where the windowed app gets its first live run) before Phase 2 is called fully closed; per the amended 2.10 gate line this does not block Phases 3/5, which do not depend on pixel output.
>
> **Addendum (2026-11-15, later the same day — Joey: "Did you try rendering headless?"):** at completion time the honest answer was "only far enough to hit the no-adapter error". That gap is now closed on this VM: `mesa-vulkan-drivers` 25.2.8 (lavapipe, `llvmpipe` device) was apt-installed, giving `wgpu` a real software adapter. Parent-run results: `tools/capture.sh --snapshot stub-house --camera 6,6,45,45 --size 1280x720` → exit 0 in ~2.6 s, PNG with 65 quads / 15 distinct colors, **visually inspected: cream walls, window insets, red-brown roof slab, correct isometric projection**; a second capture with `--cutaway 0` (camera yaw 135°, pitch 50°) → 59 quads, **visually inspected: roof removed by the cutaway, interior wood floor, window insets, and the brown door all render correctly**. Both PNGs are saved in `captures/` (`house.png`, `house_cutaway0.png`) as the first visual references. `cargo test -p engine --test capture_gate` also passes with the adapter present. What this changes: the offscreen pipeline, mesher output, camera, and cutaway are now verified end-to-end with real pixels, not just data. What it does **not** change: 2.10's formal gate (image diff against committed goldens from 3 fixed cameras) is still not established — these captures are the raw material for those goldens, not yet a diff harness; and 2.13 remains not-run, because llvmpipe frame times (~seconds per frame at 1.5 CPUs) say nothing about the 16 ms budget on real hardware.

---

### Phase 2b: Shader field effects and the HD-2D look (1-2 weeks) — added 2026-11-15, Joey's direction — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2 gate (renderer + headless capture), Phase 5 gate (sprite-derived parameters) · soft — Phase 3 (real field export; fake field tiles suffice during development). **Parallel with:** Phases 3, 4, 5, 2c.

**Direction (Joey, 2026-11-15): fields and effects are NOT rendered from the 2D sprites.** No sprite quads pasted into the 3D world for smoke, fire, gas or other field effects. Instead, each effect is a **shader effect parameterized from its sprite**: the sprite contributes its palette and character (dominant colors, emissive tint, alpha/density profile), and a procedural shader produces the actual visuals. The art target is the modern HD-2D look (the Final Fantasy / Octopath-style remakes): crisp voxel geometry, dramatic lighting, bloom on emissives, tilt-shift depth of field, and a warm graded finish — not flat 2D art floating in 3D.

**Tasks**
- **Effect definitions (data-driven, per field type):** extend the Phase 5 voxelizer with an effect-parameter extraction pass — for each field type in the (fixture or real) TileDefs set, read its sprite and emit an `effects.json` entry: base color, emissive color + strength, density scale, noise scale/speed, buoyancy/drift direction, render class (`smoke`, `flame`, `gas`, `liquid`, `energy`, `generic`). Hand-tunable via the same `overrides.toml` mechanism as materials. Every field type resolves to an effect definition or an explicit logged fallback — never a sprite quad.
- **Effect rendering (engine):** fields live in the engine world model as per-tile `(type, intensity)` (the protocol already carries exactly this, §6). Each field tile renders as a shader volume: an instanced unit volume whose fragment/compute shader does animated fbm noise, soft silhouette falloff, and density ∝ game intensity; `flame` adds emissive flicker and upward scroll, `smoke` adds rise + lateral drift and soft depth-faded edges, `gas` stays low and translucent. Animation is time-based in the shader — the game stays turn-based; no per-turn effect updates cross the IPC.
- **Displayed-field parity stays the cheap path:** the game's `displayed_field` (§6 / decision 1) still tells the engine which field dominates a tile; the effect pass weights volumes by it instead of re-deriving the game's rule.
- **HD-2D post stack (fullscreen, renderer-agnostic):** bloom (emissive-driven), tilt-shift DoF (sharp band at the follow target's depth, blur near/far), vignette + warm color grade, optional subtle chromatic offset. Implemented as post passes over color+depth so the identical stack applies to the Phase 2 raster renderer now and the Phase 2c ray-traced renderer later — 2b must not bake any raster-only assumptions into the stack.
- **Shared effect system:** Phase 8's animations/projectiles (muzzle flash, explosions, thrown items) consume this same shader-effect system with transient emitters, not sprite-frame animation — Phase 8's scope is updated accordingly.
- **Fake-data first:** a scripted test scene (fake house + placed smoke/fire/gas tiles at intensities 1–3) drives development and the headless-capture gates, exactly as Phase 2 used the fake house.

**Deliverables**: `effects.json` from the voxelizer (+ overrides support); engine effect pass + HD-2D post stack, both toggleable per-pass for A/B captures; headless captures of the effect test scene.

**Exit criteria**: the effect test scene reads clearly at all intensities (smoke obscures, fire glows and blooms, gas stays low); no field is ever rendered as a pasted sprite; post stack on/off captures show the grade without changing geometry or UI legibility.

> #### Phase 2b completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** voxelizer effect extraction (`crates/voxelizer/src/effects.rs`: `classify_field` mapping, sprite palette/emissive/density sampling, deterministic `effects.json`; `--effects`/`--field-ids` bin flags; `overrides.toml` extended with validated `effect_density`/`effect_emissive`/`effect_noise_scale`/`effect_class` keys, 5.9 behavior preserved) and a **new standalone crate `crates/fx`** (wgpu 24): field world model mirroring `FieldOnTile`+`DisplayedField` with pure `composite_weights` (displayed ×4.0), intensity→density curve (0/0.35/0.70/1.0) with Beer–Lambert obscurance + CPU fbm/falloff reference, GPU field-volume pass (instanced volumes, chord-evaluated fbm — no sprite texture bound, ever), HD-2D post (bloom, tilt-shift DoF, warm grade/vignette, chromatic offset; per-pass toggles), a standalone box-scene headless-capture harness (`fx-capture`, same offscreen pattern as the engine's capture), and `docs/phase2b-effects.md` (incl. the engine-integration wiring design). Ran solo after the Phase 2c agent was blocked (see Progress log); parent scaffolded `crates/fx` + `crates/treebuild` as workspace members before dispatch.
>
> **Test-gate results (Part II §5b), parent-verified by re-running the gates:** 2b.1 ✅ (fixture: every id → entry or logged fallback, palette matches an independent sprite sample; **real UltiCa: 39 field types, 39 with sprites, 0 fallbacks** — liquid 20, gas 6, smoke 5, flame 3, energy 3, generic 2; re-run byte-identical; Phase 5 totals reproduced 7,717/26,389/14,450) · 2b.2 ✅ (field WGSL contains no texture/sampler at all, asserted on source; CPU density center > edge, silhouette corner exactly 0) · 2b.3 ✅ (curve strictly monotonic; int-3 smoke obscurance >0.99, int-1 <0.99) · 2b.4 ✅ (displayed dominates in both configurations, incl. displayed int-1 vs rival int-3) · 2b.5 ✅ adapter-conditional (lavapipe captures non-blank; int-3 vs int-1 volume diff ratio >1.3×) · 2b.6 ✅ (CPU reference: all-off pixel-identical exactly; bloom/DoF/grade each isolated; GPU all-off identity via shader early-out) · 2b.7 ✅ (UI rect bit-exact over a graded frame; post is world-only by construction) · 2b.8 ⛔ not run (RX 9070 XT only). Totals from the parent's own run: `cargo test -p voxelizer -p fx` all pass (voxelizer 13+4, fx 7+7), **workspace 75/75**; parent visually inspected `captures/effects_post_on.png` vs `effects_post_off.png` — identical geometry, post-on adds warm grade, vignette and fire bloom; gas sits low and green, smoke soft grey, fire glowing.
>
> **Deviations and why:** (1) The post stack is **one fused fullscreen pass with per-pass uniform toggles**, not four separate passes — isolation semantics unchanged, the CPU reference implements them separately. (2) GPU DoF focal depth is in projected-depth units (CPU reference uses view distance) — same sharp-band semantics. (3) `effects.json` comes from a **separate entry point** (`run_effects`/`--effects`), not `run()`'s output set, to keep the Phase 5 golden (5.1) intact. (4) No depth-texture soft-particle fade — edge softness is chord/silhouette falloff; soft particles are an engine-integration addition. (5) Two bugs found and fixed in-run: radial falloff on the cube *surface* is identically 0 (fixed by chord evaluation through the volume), and an empty field list created a zero-sized storage buffer (fixed with a dummy instance + draw guard).
>
> **Gate verdict: CONDITIONAL PASS.** Conditions: 2b.8 runs on the RX 9070 XT; and **fx is not yet wired into `crates/engine`** — the engine renderer/capture integration (field pass after the opaque world, post before the UI quad, per §5b/2b.7) is the recorded follow-up in `docs/phase2b-effects.md`, deliberately not done in this run because the engine's renderer is also Phase 2c's integration surface. Phase 8's animation scope is unchanged (it consumes this same fx system).

---

### Phase 2c: Ray-traced renderer on Sparse-Voxel-64Tree (2-3 weeks) — added 2026-11-15, Joey's direction — ✅ DONE (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2 gate (world model, camera, capture harness) · soft — Phase 5 (materials for the side-table; flat-color rendering needs only Phase 2), Phase 3 (real snapshots; fake data suffices during development). **Parallel with:** Phases 3, 4, 5, 2b.

**Direction (Joey, 2026-11-15): the engine is to be ray traced**, using the [Sparse-Voxel-64Tree](https://docs.rs/Sparse-Voxel-64Tree/latest/Sparse_Voxel_64Tree/) crate as the voxel acceleration structure. This replaces the greedy-mesher raster path as the primary renderer (the mesher stays as a fallback/comparison path until this gate decides the default).

**Crate facts (verified against docs.rs + crates.io, 2026-11-15 — re-verify at implementation time):**
- `Sparse-Voxel-64Tree` **0.1.0** (single release, 2025-03-14, MIT, Rust 2024): a sparse **64-tree** ("contree") — each node branches 4×4×4, with a 64-bit child mask. `Sparse64Tree` holds `Node64`s in a slotmap with stable keys; `insert(x, y, z, depth, color)` adds a voxel; the tree flattens (breadth-first, contiguous child pointers) into `GpuNode64` records for GPU upload.
- `GpuNode64` is 32 bytes `#[repr(C)]`: child mask (2×u32), child pointer + leaf flag (u32), padding, and **`color: [f32; 3]` — one RGB color per node, nothing else.** There is no material index, no per-face data, no flags. This is the central integration constraint (see the material side-table task below).
- `Tree64GpuManager` is **wgpu-native**: it owns a `wgpu` bind group/layout, builds from `(&Device, &Sparse64Tree)`, and re-uploads on update. **Its wgpu dependency is `^24.0.1` and this project pins wgpu 24.0.5 — compatible** (checked 2026-11-15). What the crate does *not* ship is the traversal shader: "GPU tracing support" means the GPU-resident tree + binding; the ray-traversal WGSL is the consumer's (the author's Lepton Engine is the reference consumer).
- There is **no incremental GPU update**: an update re-flattens and re-uploads the tree. Turn-based deltas are small, but rebuild cost at real scene size must be measured, not assumed (see tasks).
- Single-release, single-author crate = supply-chain risk: pin `=0.1.0`, and keep the vendoring option open. The GPU node layout is small and documented, so a fallback in-house flatten (same 32-byte format, our own traversal) is feasible if the crate stalls — the plan must not become hostage to it.

**Tasks**
- **Spike R0 (compat + traversal proof, do first):** depend on the crate, build a tree from the Phase 2 fake house, upload via `Tree64GpuManager`, and write a minimal WGSL compute traversal (primary rays only, flat color from `GpuNode64.color`) into the existing `--headless-capture` path. Pass = the fake house renders recognizably through the tree, on lavapipe, at small size. If the crate's wgpu surface fights ours, fall back to consuming its flattened bytes with our own buffer/bind group (same spike, alternate arm) and record which arm won.
- **Tree builder (world → tree):** a `crates/treebuild` (or engine module) that converts the engine world model (12×12 chunk columns, 1 tile = 1 block, 1 z-level = 3 blocks) into a `Sparse64Tree` at block resolution: stored world 132×132 tiles × cutaway z (≤63 blocks) fits a depth-4 tree (4⁴ = 256 per axis). Snapshot = full build; Delta = apply to the world model, then rebuild-and-reupload, with the rebuild cost measured per gate 2c.5. If full rebuilds miss the budget, switch to **one tree per chunk column** via the crate's `TreeMemoryManager` (multiple trees in one GPU buffer) and rebuild only dirty columns — the builder's chunking must make this a configuration change, not a rewrite.
- **Material side-table (the texture story):** leaves need Phase 5 materials (texture-array layer per face class, transparency class, tint), not a flat RGB. Design: the flatten order is deterministic breadth-first over a tree *we* built, so generate a **parallel GPU buffer indexed by flattened node index** carrying `{material_id, lit_level, flags}` per node, built by walking the same tree in the same order; a round-trip test (gate 2c.2) proves the indexing matches the crate's flatten on every build. The hit shader resolves the material, samples the Phase 5 texture array (top/side by hit normal), and applies tint. Fallback if the parallel-order assumption breaks: pack a palette index into `color` (flat-shaded look) and treat the side-table as best-effort — but the gate decides this on evidence, not preference. **Consequence for Phase 5:** its "mesher UV/texture-layer follow-up" is superseded on the RT path — the mesher only needs UVs if the raster fallback is kept long-term; that decision is this phase's gate output.
- **Shading model:** primary ray → hit (voxel, normal, material) → textured albedo × game `lit_level` visibility (Phase 7's source of truth for *what is visible* — never-seen tiles are simply not in the tree / rendered black) + a sun directional light with a **shadow ray**, plus cheap AO from tree occupancy (neighbor queries in the traversal). Emissive materials (fire-lit, lava, screens) feed 2b's bloom. This is *illumination*, deliberately separate from the game's visibility: the game decides what you may see; the tracer decides how it is lit.
- **Hybrid composition (unchanged contracts):** the tracer writes color + depth to offscreen textures; billboards (Phase 6) rasterize on top depth-tested against the traced depth; field effects (Phase 2b) composite/march after; the UI quad (Phase 4) stays last. `--headless-capture` captures the same composed frame.
- **Picking:** mouse picking (§7) reuses the tree: a CPU traversal of the slotmap tree (or a one-ray GPU dispatch) replaces the mesher-era chunk DDA — same tile-coordinate result, one code path for "what is under this ray" in both rendering and picking.
- **Cutaway:** z-cutaway becomes a tree-build filter (don't insert above the cut plane) or a traversal max-z clip — pick by measurement; the Phase 2 cutaway semantics (gate 2.8) must hold identically.
- **Fallback policy:** keep the Phase 2 raster path runnable behind a `--renderer raster|rt` flag through this phase; the gate compares both on the same scenes (correctness first, then frame time on the RX 9070 XT) and records which becomes the default. Deleting the raster path is a later, separate decision.

**Deliverables**: R0 spike report; tree builder + material side-table; WGSL traversal/shading shaders; RT mode in the windowed app and in `--headless-capture`; renderer comparison captures + frame-time table (lavapipe for correctness, RX 9070 XT for performance).

**Exit criteria**: fake-house and bubble-scale scenes render through the tree with textures and shadows, visually matching or beating the raster path on the same captures; deltas update the render within the turn budget; picking and cutaway behave identically to the raster path; a recorded default-renderer decision.

> #### Phase 2c completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `crates/treebuild` — `VoxelWorld` (sparse voxel map + `ChunkMap` adapter + `subset_box`), the tree builder (deterministic sorted inserts into `Sparse64Tree`; depth auto-selected ≤6, depth 4 for the stored world), the **material side-table reconstructed by BFS-walking the flattened array and verifying every leaf's flat color against the world voxel at its reconstructed coordinate** (the crate's `Node64` fields are private — `flatten()` bytes are the only walkable structure; a mismatch fails the build), a CPU traversal oracle mirroring the WGSL line-for-line, an Amanatides–Woo DDA reference, and the WGSL tracer (two compute pipelines sharing the crate manager's bind group as group 0: `render_main` — primary rays, sun + shadow ray, occupancy AO, `lit_level` shading, one transparent layer; `query_main` — caller rays for GPU picking). Engine integration in `crates/engine/src/rt.rs`: RT world built from the **same `world::filtered_map`** as the raster path (extracted from `mesh_world`), `--renderer raster|rt` in headless capture and the windowed app (offscreen RT texture + fullscreen blit), `tools/capture.sh` passes `--renderer` through. Full report: `docs/phase2c-rt.md`.
>
> **Test-gate results (Part II §5c), parent-verified by re-running the suites (workspace 91/91):** 2c.1 ✅ (house + bubble exhaustive point queries, proptest random worlds, proptest tree-vs-DDA ray parity both modes), 2c.2 ✅ (incl. shuffled-table and wrong-world negative cases), 2c.3 ✅ (GPU-vs-CPU parity on lavapipe: voxel/normal/node exact, dist <1e-3, both trace modes), 2c.4 ✅ (**silhouette IoU 1.000**, 0 disagreeing pixels at 320×180; mean |color diff| 70.2/255 on shared geometry, itemized as lighting delta), 2c.5 ⚠️ measured (release, this 1.5-CPU VM: full stored-world build 91.7 ms vs 20 ms target-hardware budget; single-tree delta = full rebuild 88.4 ms misses 2 ms, as anticipated; **dirty-column rebuild 0.110 ms** is the recorded delta mechanism), 2c.6 ✅ (leaf-count difference exactly the 144-block roof layer; z=−10 empty root-only tree, z=+10 full; render-level roof loss confirmed), 2c.7 ✅ (24 orbit rays vs DDA, tile coords equal), 2c.8 ✅ (BLANK excluded at build and shadow-transparent; MEMORIZED present/flagged/shadow-blocking), 2c.9 ⛔ target hardware (RX 9070 XT), 2c.10 ✅ (**primary arm won** — `Tree64GpuManager` bind group used directly; buffer == nodes×32 asserted).
>
> **Default-renderer decision (recorded per the gate): raster stays the default; RT is opt-in via `--renderer rt`.** 2c.4 correctness evidence is complete, but the decision rule weighs 2c.9 frame time, which does not exist yet; a sub-16 ms median at 1440p on the RX 9070 XT flips it.
>
> **Findings / deviations:** (1) **Textures:** the RT shader shades from flat node color + side-table (material/lit/flags/alpha); the Phase 5 texture array is **not sampled** because the engine has no GPU texture array yet — the same open follow-up as the raster path (materials JSON loads, no array). Exit-criteria "with textures" is therefore met at the data level (per-leaf material ids proven by 2c.2), not the sampling level. **Addendum (2026-11-15, Joey's word): texture sampling is now hooked up on the RT path** — scene-local texture array + per-material (top/side/bottom) layer lookup in treebuild, engine spec builder decoding only referenced layers (fake house: 27 of 26,389), `--textures <dir>` on capture/windowed; gates: treebuild checkerboard test + engine UltiCa gate (9/9 materials, depth bit-identical, distinct colors 38→232); captures `house_rt.png` vs `house_rt_textured.png`; details in docs/phase2c-rt.md. Still open at the time: mipmaps, multitile layer selection, and the raster path's mesher-UV follow-up — **scheduled 2026-11-15 as Phases 2e, 2d and 2f respectively (Joey's direction)**. (2) **llvmpipe device loss:** dynamically-indexed local arrays in WGSL (`array<f32,3>` indexed by a loop variable) compile clean under naga but lose the llvmpipe device at pipeline creation; vectorizing the slab/entry-normal helpers fixed it — recorded in docs/phase2c-rt.md. (3) **RT captures are linear-valued** (storage texture, no sRGB encode on write, unlike the raster surface format), so `captures/house_rt.png` reads darker than `house.png`; the windowed blit compensates with pow 2.2. (4) **Delta path:** per-column rebuild is measured and available via `subset_box`, but the engine still rebuilds the single tree per cutaway change; wiring dirty-column updates is deferred until real Snapshot/Delta data (Phases 3/4) drives updates. (5) **fx (2b) not composited over RT frames yet** — fx→engine wiring was deferred until 2c landed; it is now unblocked and remains a named follow-up. (6) Windowed RT has never run live on this VM (same status as the windowed app generally); the headless capture path is the verified surface.
>
> **Gate verdict: CONDITIONAL PASS.** Condition: 2c.9 on the RX 9070 XT. Everything else mandatory is green on lavapipe + CPU evidence.

---

### Phase 2d: Multitile layer selection — connected walls (small follow-up, days) — added 2026-11-15, Joey's direction — ✅ GATE PASSED 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2c texture hookup (landed f4c1c45), Phase 5 gate (`subtile_layers` data) · soft — Phase 3 (connect-group semantics for real data; fake data uses same-material adjacency). **Parallel with:** 2e, 2f, Phases 3, 4.

**Problem (Joey's words, 2026-11-15):** the texture hookup always uses the plain standalone-wall picture for every wall block, ignoring neighbors — walls look repetitive and corners/joins don't visually connect. In the game's tileset, a wall's sprite depends on its neighbors: straight run, corner, T-junction, cross, end cap and isolated pillar are different pictures, so a wall run reads as one continuous structure.

**What exists already:** Phase 5's voxelizer emits per-material `subtile_layers` in `materials.json` — the variant layer indices for all 16 cardinal connection masks plus connect-group cases — and Phase 3 exports the game's connect-group data in TileDefs. What is missing is purely the *selection*: nothing looks at a voxel's neighbors when its layers are chosen.

**Tasks**
- **Connectivity at build time:** when the engine builds a scene (RT side-table / texture spec, and the mesher under 2f), compute each voxel's cardinal connection mask (N/E/S/W neighbors that connect: same material, or same connect group once TileDefs drives real scenes) and select the side layer from the material's `subtile_layers` for that mask. Top/bottom layers are unchanged. The per-voxel choice must ride into the RT shader without breaking the per-material lookup: a per-node layer override in (or alongside) the side-table is the expected shape — the shader prefers the override when present.
- **Semantics match the game:** connection is the game's own rule (connect groups / `connects_to`), not "same sprite": a wall must connect to a door frame or fence exactly when the tileset says it does. Fake data (no TileDefs yet) uses same-material adjacency and says so.
- **Scope:** side faces of wall-like (multitile) materials first. Floors/roofs keep base layers; rotation variants are Phase 8 polish, not this phase.

**Deliverables**: connectivity selection in the engine texture path (RT first); captures of a wall-run test scene (straight, corner, T, cross, end cap, isolated) flat vs selected; gate results in Part II §5d.

**Exit criteria**: the wall-run scene renders each layout with the variant the voxelizer assigned to that connection mask; non-multitile materials and the no-textures path are byte-identical to before; 2c.4's silhouette parity is untouched.

> #### Phase 2d completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `mesher::connect` — the mask→subtile table moved into the leaf crate as the workspace's one canonical copy (`voxelizer::multitile` now delegates to it; its connect-group helper stays put); `treebuild::connect` — `connection_mask` over a `VoxelWorld` under a caller-supplied predicate (fake rule `same_material`) plus `assign_variants`; engine `materials::Material` parses `multitile`/`subtile_layers` and owns the shared selection rule `side_layer_for_mask`; `textures::build_texture_spec_multitile` emits variant entries through the base builder's shared compaction path; RT wiring in capture (`rt::prepare_fake_variants` before `build_tree`) and the windowed remesh, both only when `--textures` is set. Implemented directly in the main session after the parallel-subagent dispatches for 2d/2e/2f were refused by a safety policy; 2e/2f landed concurrently in other sessions, so the 2d commit is scoped to 2d files only. Full design + evidence: `docs/phase2d-multitile.md`.
>
> **Deviation and why:** the plan expected a per-node layer override in/alongside the side-table with the shader preferring the override. Shipped instead: **virtual materials** — each occurring `(base material, mask)` pair gets a fresh material id whose existing `mat_layers` entry carries the variant side layer, and the voxel's side-table entry carries the virtual id. The shader, side-table format, CPU oracle and 2c parity machinery are untouched (2d.4 parity is structural), and the same virtual entries feed the raster path's per-block layer stamping in 2f. Exit criteria are behavioral and all pass.
>
> **Test-gate results (Part II §5d), re-run by the implementing session:** 2d.1 ✅ (six layouts, 19 hand-computed masks; negative cases; deterministic assignment into the side-table) · 2d.2 ✅ (16/16 masks on a fixture; real UltiCa `t_wall`: variant side layers byte-identical to the voxelizer's layer files for each mask — note the *unconnected* picture legitimately shares pixels with the base side's source sprite, so the criterion is layer-index fidelity) · 2d.3 ✅ on lavapipe (per-layout center-pixel variant hues vs red base control; depth bit-identical; sky identical; no-textures render pixel-identical) · 2d.4 ✅ (full workspace suite green — 113 tests at verification time including the in-flight 2e/2f suites; 2c.4 parity re-run green). Fake-house evidence: `captures/house_rt_textured_2d.png` vs the pre-2d `captures/house_rt_textured.png` (wall runs now show the *edge* subtile's continuous siding).
>
> **Scope held / open follow-ups:** side faces only, rotation variants remain Phase 8; connection semantics are the fake same-material rule — connect-group predicates plug into `assign_variants` unchanged when TileDefs drives real scenes (Phase 4 wiring). No target-hardware condition: 2d has no hardware gate.

---

### Phase 2e: Texture mipmaps for the RT renderer (small follow-up, days) — added 2026-11-15, Joey's direction — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2c texture hookup (landed f4c1c45) · soft — Phase 5 (its alpha-aware premultiplied averaging rules are the downsample reference). **Parallel with:** 2d, 2f, Phases 3, 4.

**Problem (plain form):** RT textures are sampled at full resolution with nearest filtering no matter the distance. A far-away face carries far more texture detail than the screen has pixels for, and the single texel picked per pixel jumps around as the camera moves — distant textured surfaces alias: they shimmer and crawl. The standard cure is mipmaps: pre-shrunk copies of each texture (half, quarter, … size) with the detail averaged out, sampled at the level matching the face's on-screen size.

**Design constraint that shapes the work:** the RT renderer is a *compute* shader, and compute shaders have no implicit derivatives — `textureSample` (which auto-picks a mip level in fragment shaders) is unavailable. The LOD must be estimated explicitly from ray geometry and sampled with `textureSampleLevel`. Getting that estimate right — neither blurry near nor shimmering far — is the phase's real work; generating the chain is the easy half.

**Method (integrated 2026-11-15, Joey supplied the source plan): improved ray cones** (Akenine-Möller et al., JCGT 2021 — the Ray Tracing Gems ch. 20 refinement), mapped onto this renderer in **`docs/phase2e-mipmaps.md`**, which is the phase's implementation plan. The mapping's load-bearing simplifications: per-face Δ collapses to the constant `0.5·log2(texW·texH)` (every voxel face maps the full texture over exactly 1×1 world units); no ray queries/BLAS/bindless are involved (custom tree traversal + texture array — the source plan's own fallback design); curvature/reflection/refraction cone machinery does not apply (flat faces, no secondary texture-sampling rays — the transparent overlay reuses the primary cone at its own hit distance); mip filtering happens in the stored (display) space, consistent with this renderer's raw-sampling convention. Full-angle cone convention throughout.

**Tasks**
- **Mip chain generation** for the scene texture array (per layer, down to 1×1), using the voxelizer's alpha-aware premultiplied averaging with edge clamping (Phase 5 §8 rules) so cutout edges don't bleed color, plus alpha-coverage rescaling for cutout materials so they don't thin out at coarse levels. CPU generation at spec-build time is acceptable at scene-local layer counts; measure before optimizing.
- **Ray-cone LOD + sampling** in the RT render shader per docs/phase2e-mipmaps.md §2: `spread = atan(2·tan(fovY/2)/imageHeight)` from existing uniforms, `lod = Δ + log2(spread·t) − log2(|N·V|)` clamped to the chain, `textureSampleLevel` with a trilinear sampler. Applies to primary hits and the transparent overlay hit. A debug false-color LOD view ships behind a uniform flag (validation + future tuning).
- Sampler/filtering changes stay inside `TextureSet` (incl. no restrictive sampler LOD clamp — a too-low max clamp silently degrades mips to level 0); the flat-color dummy path is unaffected.

**Deliverables**: mip chain in `TextureSet`; ray-cone LOD sampling + debug LOD view in the RT shader; CPU reference of the LOD formula with analytic-case tests; before/after captures of a distant textured wall (two camera poses a fraction of a pixel apart); gate results in Part II §5e.

**Exit criteria**: distant textured faces are stable between the two poses (variance metric in gate 2e.3 drops materially vs the level-0-only build); close-up captures are unchanged (level 0 sampled); GPU LOD agrees with the CPU reference on the analytic cases; existing texture gates still pass.

> #### Phase 2e completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `treebuild/src/lod.rs` — the CPU reference for the improved-ray-cone LOD (`spread_angle`, `ray_cone_lod`: per-face Δ collapses to the constant `0.5·log2(texW·texH)`; `lod = Δ + log2(spread·t) − log2(|N·V|)`, clamped to the chain) plus the mip generation itself (`downsample2x`: premultiplied-alpha averaging with edge clamping per the Phase 5 §8 rules; `mip_chain` with mean-alpha coverage rescaling so cutout materials don't thin out at coarse levels). `TextureSet` now uploads the full chain and samples with a trilinear (Linear/Linear/Linear) sampler whose `lod_max_clamp` is `levels − 1` (no restrictive clamp); the WGSL tracer computes `cone_lod()` per hit and samples via `textureSampleLevel` on both the primary and the transparent-overlay hit through the shared `shade()` path. A debug false-color LOD view ships behind a uniform (`RtRenderer::render_debug_lod`). Implemented by a subagent session; gates re-run and captures verified by the parent session.
>
> **Test-gate results (Part II §5e), re-run by the parent session on lavapipe:** 2e.1 ✅ (chain vs CPU reference; premultiply/no-bleed; coverage preserved) · 2e.2 ✅ (CPU analytic cases + GPU debug-view readback: distance doubling = +1.00 LOD ±0.15) · 2e.3 ✅ with a recorded substitution — the level-0 baseline is a CPU simulation of nearest level-0 sampling, not a second GPU build (no level-0-only build remains in-tree): two-pose variance on the distant wall dropped from 143.44 (baseline) to 0.00 (mip build); close-up pixel-identity against the pre-mip build was NOT re-verified · 2e.4 ✅ (texture hookup gates + full workspace suite green at verification time).
>
> **Gate verdict: CONDITIONAL PASS.** Conditions: the 2e.3 substitution above stands until a level-0-only comparison build is kept; whether mip sampling is on by default at 1440p on target hardware folds into the open 2c.9 measurement (RX 9070 XT). Note the sampler change (Nearest → Linear) also affects magnification on all RT texture sampling; the pre-existing checkerboard/UltiCa texture gates still pass under it.

---

### Phase 2f: Raster-path textures — mesher UVs (small follow-up, days) — added 2026-11-15, Joey's direction — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2 gate (mesher + raster pipeline), Phase 5 gate (materials) · soft — Phase 2c texture hookup (its scene-local spec builder and texture set are reused, not duplicated), Phase 2d (the same per-voxel layer selection feeds the mesher). **Parallel with:** 2d, 2e, Phases 3, 4.

**Problem (plain form):** the raster renderer — still the default — shades flat color only. Painting a texture on a rasterized triangle needs each vertex to carry UV coordinates (which point of the picture this corner corresponds to) plus which texture layer to sample; the mesher's `Vertex` has neither field, so the raster pipeline cannot sample textures at all. This is Phase 5's named follow-up ("mesher UV/texture-layer field"), scheduled here as its own phase. The RT renderer never needed UVs — it computes the hit position mathematically — which is why it got textures first.

**Tasks**
- **Mesher change:** extend `Vertex` with UV + texture-layer fields; the greedy mesher emits per-face UVs spanning the face and the material's layer for that face class (top/side/bottom), resolved from the same `MaterialTable` the RT path uses. Greedy merging must not merge faces across different layers (it already must not merge across materials — verify the merge key includes the layer).
- **Raster pipeline:** bind the scene texture array (the engine's existing spec builder from the RT hookup, shared code) and sample in the fragment shader; materials without textures keep the flat vertex color. Alpha/cutout pass behavior (Phase 2 notes) must survive: cutout samples alpha-test, transparent blends.
- **`--textures <dir>`** works with `--renderer raster` exactly as with `rt`; no flag = today's flat captures, byte-identical.

**Deliverables**: mesher UV/layer fields + raster sampling; raster textured fake-house capture next to the RT one; gate results in Part II §5f.

**Exit criteria**: the raster fake-house capture with `--textures` shows the same materials as the RT textured capture (same layers chosen — one shared spec builder); flat captures without `--textures` are unchanged; the full workspace suite stays green.

> #### Phase 2f completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** the mesher carries texture layers end to end — `Block.tex_layers: Option<[u32;3]>` (top/side/bottom spec-layer indices) is part of the greedy merge key, so faces with different layers never merge; `Mesh` gains per-vertex `uvs` + `layers` (`FLAT_LAYER = u32::MAX` marks untextured geometry), with grid-space face UVs matching the RT path's face orientation (one world unit = the full [0,1]² of the layer). The engine raster pipeline samples the scene texture array (`Renderer::set_textures`: RGBA8 array + trilinear sampler over a CPU mip chain; a cutout pipeline variant discards below alpha 0.5); the capture path stamps the cutaway-filtered map's blocks via the SAME `build_texture_spec` the RT path uses and meshes after stamping, so `--renderer raster --textures <dir>` resolves identical (material → layers) as RT. The no-flag flat path is untouched. Implemented by a subagent session; gates re-run and captures verified by the parent session.
>
> **Test-gate results (Part II §5f), re-run by the parent session on lavapipe:** 2f.1 ✅ (UV span / per-class layer / merge-key separation) · 2f.2 ✅ (raster textured vs flat silhouette IoU 1.000 — same geometry, only shading changed; parent captures: raster textured 65 quads / 3,187 distinct colors vs 15 flat, RT textured 3,465, same materials visually in both renderers; a direct raster-vs-RT PNG IoU is not meaningful across lighting models, so the comparison chains through the existing 2c.4 RT-vs-raster parity) · 2f.3 ✅ (one shared spec builder, asserted by test) · 2f.4 ✅ (no-flag flat path byte-unchanged in behavior — the flat-color golden in `capture_gate` passes; full workspace suite green at verification time).
>
> **Gate verdict: CONDITIONAL PASS.** Condition: the **windowed** path still renders flat — only the headless capture path got raster textures in this phase (the windowed entry point was outside the phase's file scope); wiring `set_textures` into the windowed remesh is a named follow-up. Raster texture format is RGBA8 (the RT spec's format); sRGB handling is unchanged from the flat path.

---

### Phase 2g: Integration follow-ups and coverage closure (small follow-ups, days each) — added 2026-11-15, Joey's direction

> **Dependencies:** ⛔ hard — Phase 2b, 2c and 2f gates (2g.1/2g.2), Phases 2 and 5 (2g.3 goldens), Phase 5 + the coverage audit (2g.4) · soft — Phase 4 (UI-quad ordering for the post pass; the wiring proceeds against the current frame layout and re-seats when Phase 4 lands). **Parallel with:** Phases 4, 6, 7 — with one coordination rule: 2g.1 and 2g.2 touch `crates/engine`'s render/capture surface, which Phases 4 (UI quad) and 6 (billboard pass) also touch. When run concurrently, assign file ownership per pass (4: compositing, 6: billboards, 2g.1: field + post passes) or sequence 2g.1 after Phase 4's compositing lands. 2g.3 touches only tests; 2g.4 only `coverage-audit/` — both are conflict-free against everything.

**Problem (plain form):** four recorded loose ends are each too small for their own phase but none of them close themselves: the fx crate is built and gated yet never renders in the actual engine; textured rendering works headless but the windowed app still draws flat; two gate conditions (2.10, 5.13) wait on a formal image-diff harness that doesn't exist; and gate 5.10's measured missing list (202 terrain / 174 furniture / 468 monsters) only shrinks if someone draws the tiles. This phase bundles them so they are scheduled, gated, and closed instead of living as footnotes in other phases' records.

**Workstreams**

- **2g.1 — fx → engine wiring** (closes Phase 2b's recorded follow-up; design in `docs/phase2b-effects.md`). The engine consumes `effects.json` and field state in the Phase 3 export shape (fake field tiles until the fork runs in-game). Field-volume pass renders after the opaque world in both renderers (for RT: composited over the traced frame, depth-tested like billboards); the HD-2D post stack (bloom / tilt-shift DoF / warm grade) runs as the final world pass, before the UI quad. The soft-particle depth fade deferred in 2b lands here, where a real depth buffer exists. Pass order in the frame: world → fields → billboards (Phase 6) → post → UI (Phase 4).
- **2g.2 — Windowed textures** (closes Phase 2f's condition). Wire `Renderer::set_textures` into the windowed remesh path (`crates/engine` main loop) for both renderers, behind the same `--textures <dir>` flag as the capture path. The windowed app then shows the same materials as the headless textured captures.
- **2g.3 — Formal image-diff gate** (closes conditions 2.10 and 5.13). A golden harness over the existing headless capture: a fixed camera set (fake house + fixture tileset scenes, both renderers, flat + textured + effects variants), committed golden PNGs, a per-pixel tolerance with a mismatch budget, and an `UPDATE_GOLDEN=1` regeneration flow mirroring the voxelizer's 5.1 golden flow. Runs in the normal test suite wherever an adapter exists (lavapipe here); the vacuous-skip caveat applies — a run with no adapter proves nothing and must say so.
- **2g.4 — Tile creation from the coverage audit** (works down gate 5.10's missing list). Batches drawn from `coverage-audit/missing.json` in priority order — player-visible terrain first, then furniture (the stack/mound family is one batch), then monster families — each sprite drawn from its in-game description in UltiCa decomposed layout under `coverage-audit/new-tiles/`, CC BY-SA, verified per batch through the release voxelizer (all `exact`, 0 missing) with the audit re-run to record coverage movement. First batches already landed (butterfly + moth lifecycles, commit `5f740f2`). This is a long tail by design: 5.10 itself closes only at ≥98% per category or when Joey re-scopes the gate; batches are independently committable and never block the other workstreams. **Update 2026-11-15: the drive ran to zero — all three categories at 100% (see the completion record below); 5.10's coverage criterion is met, its TileDefs-form re-run still waits on 0.1/0.2.**

**Gate items**
- 2g.1a: fx crate suite stays green; engine capture pair with fields off/on — geometry identical, field volumes present and lit per the coupled smoke/fire rules.
- 2g.1b: post on/off capture pair — same geometry, grade/bloom/DoF only; post sits before the UI quad in the pass order (asserted in code structure until Phase 4 provides a real quad).
- 2g.2: windowed code path exercised by test (spec built + `set_textures` called on remesh); headless parity unchanged. Live windowed run is target-hardware evidence (same class as 2.13), not a VM gate.
- 2g.3: harness green on current captures; negative test — a deliberate one-material change in a scratch scene fails the gate; goldens committed.
- 2g.4: per batch — voxelizer test-tileset run all-`exact`/0-missing, audit re-run numbers recorded in `coverage-audit/REPORT.md`, batch committed.

**Exit criteria**: 2g.1–2g.3 close their conditions (2b follow-up, 2f condition, 2.10, 5.13); 2g.4 continues batch by batch with the audit percentages as the scoreboard and no fixed end date.

> #### Phase 2g completion record (2026-11-15, per the maintenance rule)
> **Run shape:** 2g.3 and 2g.4 ran as subagents in parallel with Phases 4/6/7 (disjoint ownership: tests / `coverage-audit/`). The 2g.1+2g.2 subagent dispatch was **refused by a safety policy before any work** (same pattern as the first 2c dispatches — no reason given, no code written); on Joey's re-ask both were implemented directly in the main session, the 2c/2d precedent. All gates below are parent-verified by re-running them.
>
> **2g.1 — fx → engine wiring ✅ DONE.** New `fx::passes` module lifts the two passes out of the standalone harness for reuse: `PostPass` (the fused HD-2D stack, shader verbatim) and `FieldPass` — the **soft-particle variant** of the field shader, derived from `FIELD_SHADER` by anchored, test-pinned string edits (extended uniforms, a scene-depth binding, each `fs_main` return wrapped in a `soften()` helper). The base shader is untouched and still binds no texture (gate 2b.2 intact); the fade band is computed in projected-depth units at the working distance, since projected depth is nonlinear. Engine side: `crates/engine/src/fxwire.rs` — effect defs load from the voxelizer's `effects.json` (regenerated from the real UltiCa tileset for the record captures: **39 defs, 39 with sprites, 0 fallbacks**, now at `assets/ultica_run1/effects.json`); the fake-house field scene is placed in the Phase 3 export shape (per-tile field lists) with defs selected by render class, pairing the extraction's per-intensity ids (`fd_fire_int3` at intensity 3, etc.); placement uses engine scale (floor top y=1+3z, volume 1.6). RT depth: the tracer emits a linear ray-distance buffer, not a depth texture, so `RtDepthConvert` (a depth-only pass writing `@builtin(frag_depth)`) converts it to projected depth — view depth = ray distance × cos(angle to view axis), then the engine projection's mapping. Capture CLI: `--effects <path>`, `--fx-fields-off`, `--fx-post-off`, both renderers; frame order in code: world → fields → post (billboards seat between fields and post, the UI quad after post, when the Phase 4/6 engine integration lands — the ordering rule is recorded in `fxwire.rs`). **Gates:** fx suite green (16 lib + 7 gate, incl. new passes tests pinning the shader-derivation anchors and heat/link coupling through `build_instances`); engine `fx_gate` 2g.1a ✅ on lavapipe — fields off/on pairs in BOTH renderers differ only in a localized region (raster: 1.3% of pixels at 640×360), and hot flame-core pixels exist only with fields on; 2g.1b ✅ — post pair differs broadly (>50% of pixels) with the red/blue ratio rising (warm grade), geometry untouched. Record captures `captures/house_fx_{fields_off,fields_on,post_on,rt_fields,rt_post}.png`, visually inspected: coupled fire + smoke column inside the house, fungal-haze gas tile, same composition over the traced frame. **Deviations:** (1) the windowed path does not yet route through offscreen + post — 2g.1 wiring is capture-path; the passes are engine-resident now, so the windowed route is a mechanical follow-on. (2) The flame ground-light loop (a feature of fx's box-scene base shader) was not ported into the engine's world shaders; in-engine, fire glow is carried by emissive + bloom. (3) Field time is pinned at 1.25 s in captures for determinism, as in the fx harness.
>
> **2g.2 — Windowed textures ✅ DONE (closes Phase 2f's condition).** Scene preparation was extracted to `crates/engine/src/scene_prep.rs` (`prepare_raster_house`: shared spec builder → stamp → mesh) and is now called by BOTH the capture path and the windowed remesh — they cannot drift. The windowed raster branch installs the spec via `Renderer::set_textures` behind the same `--textures <dir>` flag (the windowed RT branch had its textures since 2d). **Gate 2g.2 ✅** (`fx_gate`): fixture dir → spec with layers + stamped meshes carrying real layer indices, flat prep unchanged (no spec, all `FLAT_LAYER`), and the windowed sequence itself — `set_textures` + upload + draw — exercised on a real adapter. Headless parity: the golden gate (2g.3) stayed green across the refactor, and the non-textured capture is byte-stable.
>
> **2g.3 — Formal image-diff gate ✅ DONE (closes 2.10 and 5.13).** `crates/engine/tests/golden_gate.rs` + 7 committed goldens in `crates/engine/tests/golden/` (fake house, both renderers, flat + textured), tolerance 8/255 per channel, ≤1% mismatched pixels, mean |err| ≤ 2.0; mismatch artifacts land in `target/golden-failures/`. Negative test ✅: one recolored material fails the gate as designed (3,986 px, 1.73% vs the 1% budget). No-adapter runs skip LOUDLY (banner + `SKIPPED.txt`), never silently. `UPDATE_GOLDEN=1` regeneration + `tools/golden.sh` wrapper; record in `docs/phase2g-imagediff.md`. **Deviation:** goldens are 640×360, not 1280×720 (VM suite runtime); the effects-scene golden waits on the 2g.1 flags, which now exist — adding it is a small follow-on. **(Follow-on landed 2026-11-15: `house_raster_effects` — interior camera, fx demo defs injected via `CaptureArgs::effects_defs`, fields + post on; the set is now 8 scenes, regeneration reproduced the 7 existing goldens byte-identically, compare run 8/8 at 0 mismatched px, negative test unchanged at 1.730% vs the 1% budget; the effects frame carries 2,533 distinct colors vs 15 flat.)**
>
> **2g.4 — Tile creation, batch 1 ✅ (workstream continues).** Alien meadow terrain: all 16 ids from `terrain_alien_meadow.json` drawn from their in-game descriptions under `coverage-audit/new-tiles/` (16 entry folders, 14 PNGs — 2 trap grasses are entry-only per the game data), draw script `draw_alien_meadow.py` re-run byte-identical; release voxelizer: 16 materials, all `exact`, 0 missing. Audit overlay: terrain missing **202→184**, coverage **84.1%→85.5%**, player-visible missing 173→155. Record in `coverage-audit/REPORT.md`. Next batches follow the audit's priority order.
>
> **2g.4 — batch 2 ✅ (2026-11-15, Joey's "continue with 4").** String-dimension terrain: the 20 missing ids from `terrain-string-dimension.json` (the file's other ids resolve via `looks_like`; the regional-pseudo groundcover stays out), drawn from descriptions by `draw_string_dimension.py` (byte-identical re-run; the two voids pure black and indistinguishable as described, perceived geometry + lever as transparent fg over `bg: t_string_void`); release voxelizer: 20 materials, all `exact`, 0 missing (60 layers, 20 sprites), parent-verified. Audit overlay (the computation first reproduces batch 1's 184 and its two glowberry rescues exactly): no rescues this batch — terrain missing **184→164**, coverage **85.5%→87.1%** (1,107/1,271), player-visible missing **155→135**. Remaining named terrain clusters: nether labyrinth `t_nl_*` (17), vitrified (16). Record in `coverage-audit/REPORT.md`.
>
> **Zero-drive (2026-11-15, Joey: "drive the coverage until nothing is missing").** 2g.4's end state is now zero missing in the audit's three drawn categories (terrain, furniture, monsters — the rows the audit itself scopes as drawable; items/fields/traps/overmap/vehicle-parts stay out per the audit's caveats). Batches run by game-data file/cluster, drawn by parallel subagents under disjoint ownership, parent-verified per batch (byte-identical re-run, test-tileset voxelizer all-`exact`/0-missing, preview inspection, overlay recomputation), recorded and committed per batch.
>
> **2g.4 — batch 3 ✅ (zero-drive wave 1).** Regional-pseudo templates: all 29 missing ids from `terrain-regional-pseudo.json` (the `t_region_*` templates + highland/string/netherum/portal-storm groundcovers — "should never actually show up" in game, drawn anyway: zero is zero), `draw_regional_pseudo.py` byte-identical; release voxelizer 29 materials, all `exact`, 0 missing (87 layers), parent re-verified (own voxelizer run + preview inspection). Audit overlay (baseline 164 reproduced): 29 direct, no rescues — terrain missing **164→135**, coverage **87.1%→89.4%** (1,136/1,271); the non-player-visible terrain set is now empty, so all 135 remaining missing terrain ids are player-visible. Record in `coverage-audit/REPORT.md`.
>
> **2g.4 — batch 4 ✅ (zero-drive wave 1).** Nether labyrinth: 13 ids (the missing `t_nl_*` terrain + `t_slope_to_bosky_labyrinth`; the three mission obelisks are identical in game data and drawn identically; the glowberry pair stays covered by its `looks_like` resolution, no folders), `draw_nether_labyrinth.py` byte-identical; release voxelizer 13 materials, all `exact`, 0 missing (39 layers), parent re-verified. Audit overlay (baseline 135 reproduced): 13 direct, no rescues — terrain missing **135→122**, coverage **89.4%→90.4%** (1,149/1,271).
>
> **2g.4 — batch 5 ✅ (zero-drive wave 1).** Floors, roofs, z-level transitions: 22 ids from `terrain-floors-indoor.json` / `terrain-roofs.json` / `terrain-zlevel-transitions.json` (warped-stone family in four glow variants, four treetops, ramp chevrons), `draw_floors_roofs.py` byte-identical; release voxelizer 22 materials, all `exact`, 0 missing (66 layers), parent re-verified — including the opacity check (the batch's first run left semi-transparent pixels in 9 sprites via non-blending ImageDraw marks; restructured through alpha-composited overlays, all 22 sprites now fully opaque). Audit overlay (baseline 122 reproduced): 22 direct, no rescues — terrain missing **122→100**, coverage **90.4%→92.1%** (1,171/1,271). Process lesson: the batch-3 commit's blanket `git add coverage-audit/` swept sibling agents' in-flight files (incl. this batch's pre-fix PNGs); caught before any push, history reset and recommitted with scoped paths — batch commits now stage explicit paths only.
>
> **2g.4 — batch 6 ✅ (zero-drive wave 1).** Vitrified terrain: all 16 missing ids from `terrain_vitrified.json` (black-glass landscape; the glass wall drawn full-tile — its translucency is the pane's own property; features over `bg: t_vitrified_floor`), `draw_vitrified_terrain.py` byte-identical; release voxelizer 16 materials, all `exact`, 0 missing (48 layers), parent re-verified. Audit overlay (sequential chain from 100): 16 direct, no rescues — terrain missing **100→84**, coverage **92.1%→93.4%** (1,187/1,271).
>
> **2g.4 — batch 7 ✅ (zero-drive wave 1 complete).** Walls: all 20 missing ids from `terrain-walls.json` (shipping-container wall in 15 game colors from one parameterized corrugated panel; warped wall with wrong-side shadows; reinforced lab glass; wasp-paper pair; sunshade pole), `draw_walls.py` byte-identical; release voxelizer 20 materials, all `exact`, 0 missing (60 layers), parent re-verified. Audit overlay (baseline 84 reproduced): 20 direct, no rescues — terrain missing **84→64**, coverage **93.4%→95.0%** (1,207/1,271). The remaining 64 terrain ids are wave 2's scope (manufactured/mechanisms/sunken-helicopter 18, triffid/reverberations/ice 20, scatter 26), already dispatched.
>
> **2g.4 — batches 8–10 ✅ (zero-drive wave 2) — TERRAIN COMPLETE.** Batch 8: triffid/reverberations/ice, 20 ids (`draw_triffid_ice.py`) — overlay 64→44 (96.5%). Batch 9: the 26-id scatter (`draw_scatter_terrain.py`) — 44→18 (98.6%). Batch 10: manufactured/mechanisms/sunken-helicopter, 18 ids (`draw_manufactured.py`) — 18→0. All parent re-verified (voxelizer all-`exact`/0-missing at 60/78/54 layers, byte-identical re-runs, opacity checks clean, previews inspected). **Final terrain state: 0 missing of 1,271 — 100.0% shipped-universe coverage** (full overlay re-resolution against every new-tiles folder). Terrain's half of gate 5.10 is thereby satisfied beyond its ≥98% criterion; the gate itself closes when furniture also reaches ≥98% (zero-drive target: 0). Next: furniture waves, then monsters.
>
> **2g.4 — batch 11 ✅ (furniture wave 1).** Alien furniture: 19 ids from `furniture-alien.json` + `furniture_alien_meadow.json` (scar, portal enclosure/rings, pylons live+inert, impaled pair, dead Exodii pair, weeds), `draw_alien_furniture.py` byte-identical; release voxelizer 19 materials, all `exact`, 0 missing (57 layers), parent re-verified. Furniture overlay (baseline 174 reproduced): 19 direct, no rescues — furniture missing **174→155**, coverage **75.6%→78.3%** (558/713). (Folder cross-check reads 104 already: the stack/mound batch's 51 folders are on disk awaiting verification.)
>
> **2g.4 — batch 12 ✅ (furniture wave 1).** Stack/mound family + cardboard: all 51 missing ids from `furniture-terrains.json` (7 mound materials + 8 stack materials, each short/mid/tall with heights from the game data's coverage values, two parameterized renderers; 6 cardboard pieces drawn individually), `draw_furniture_terrains.py` byte-identical; release voxelizer 51 materials, all `exact`, 0 missing (153 layers), parent re-verified. Furniture overlay (baseline 155 reproduced): 51 direct, no rescues — furniture missing **155→104**, coverage **78.3%→85.4%** (609/713) — exactly the batch-11 cross-check figure; the chain is reconciled. Incident during this batch's commit: the in-flight vitrified-furniture agent's run wiped every sibling folder under `pngs_normal_32x32/furniture/` (its script cleared the category dir); batch 11's folders were restored from git and this batch's regenerated byte-identical from its deterministic script before committing — no work lost; the agent was instructed to remove the clearing step and verify siblings untouched on any re-run. Lesson: batch scripts must never clear a shared category directory.
>
> **2g.4 — batch 13 ✅ (furniture wave 1 complete).** Vitrified furniture: all 32 missing ids from `furniture_vitrified.json` (the household in black glass, palette matched to batch 6; the talker bookcase's missing book as a dark slot; the matte cloth doll as the set's one non-glass object), `draw_vitrified_furniture.py`; release voxelizer 32 materials, all `exact`, 0 missing (96 layers), parent re-verified — including a full-directory snapshot diff (306 files md5'd before/after the script re-run: empty), which proves byte-identical output, zero sibling impact, and closes the batch's first-run `rm -rf` incident (owned in the agent's report; the step is removed, the script has no deletion calls). Furniture overlay (baseline 104 reproduced): 32 direct, no rescues — furniture missing **104→72**, coverage **85.4%→89.9%** (641/713). Remaining 72 = furniture wave 2 (running).
>
> **2g.4 — batch 14 ✅ (furniture wave 2, first half).** Shores: all 23 missing ids from `furniture-regional-pseudo.json` (10 placeholders drawn as the flora their regional roles name), `furniture-flora_marine.json` (7), and `furniture-decorative.json` (6 — shackle, gazing ball, lighthouse light, styling station, flagpoles), `draw_furniture_shores.py` byte-identical, furniture folder count stable across the re-run; release voxelizer 23 materials, all `exact`, 0 missing (69 layers), parent re-verified. Furniture overlay (baseline 72 reproduced): 23 direct, no rescues — furniture missing **72→49**, coverage **89.9%→93.1%** (664/713). The remaining 49 ids are the furniture-remainder batch's set (running).
>
> **2g.4 — batch 15 ✅ — FURNITURE COMPLETE.** The remainder: all 49 remaining missing furniture ids (generators ×4 from their shared copy-from, nether-labyrinth furniture ×8, mutant tree line, medical, storage, mortar, roulette table, and the `f_clear`/`f_no_item` debug placeholders drawn as black "? BUG" tiles), `draw_furniture_remainder.py` byte-identical; release voxelizer 49 materials, all `exact`, 0 missing (147 layers), parent re-verified. Final check = full overlay re-resolution of every furniture id against every new-tiles folder: **furniture missing 0 of 713 = 100.0%**. Gate 5.10's ≥98% terrain+furniture coverage criterion is now met on both halves (terrain 0 of 1,271 since batch 10); the gate's TileDefs-form re-run stays blocked on 0.1/0.2. Monsters (468) are the drive's last category.
>
> **2g.4 — batch 16 ✅ (monsters wave 1).** Fungal zombies: all 50 missing ids from the fungus-zombie file (dusted/fungalize/full-bloom variants of the zombie line, mounds, constructs, zougar, ziger), `draw_fungus_zombies.py` — all 59 monster PNGs on disk byte-identical across the parent re-run; release voxelizer 50 materials, all `exact`, 0 missing (150 layers), parent re-verified. Sheet placement by resolved volume (entries carry no size field; 62.5 L → 32×32, 100–150 L → 64×64, 875 L → 96×96) — the standing monster placement rule. Monster overlay (baseline 454 with 14 fish-A folders already landed): 50 direct, no rescues — monsters missing **454→404**, coverage **66.4%** (799/1,203).
>
> **2g.4 — batch 17 ✅ (monsters wave 1).** Fish, first half: 52 of the 104 missing ids in `fish.json` (alphabetical, `mon_crayfish_giant_larva`…`mon_fry_blackfish`), `draw_fish_a.py` byte-identical; release voxelizer 52 materials, all `exact`, 0 missing (156 layers), parent re-verified. Monster overlay (baseline 404 reproduced): 52 direct + 1 rescue (`mon_nether_fish`) — monsters missing **404→351**, coverage **70.8%** (852/1,203). Placement note: all 52 are on the 32×32 sheet (no explicit size field in the data, brief's letter); the parent volume census names exactly three ≥75 L sprites (crayfish_mega 625 L, bluefin tuna 250 L, halibut 75 L) — re-sheeting them is a placement-only change; fish-B is briefed with volume placement from the start.
>
> **2g.4 — batch 18 ✅ (monsters wave 2).** Fish, second half: the remaining 52 missing ids in `fish.json` (48 fry sharing the `mon_fry` abstract's unidentifiable-larva design with seeded jitter; crayfish zoea; mutant salmon fry; huge/mega mutant salmon at 64×64 native — 312 L / exactly 400 L, the mega boundary placement flagged as placement-only), `draw_fish_b.py` — every monster PNG on disk byte-identical across the parent re-run; release voxelizer 52 materials, all `exact`, 0 missing (156 layers), parent re-verified. `fish.json` fully covered (104/104 across batches 17–18). Monster overlay (baseline 351 reproduced): 52 direct, no rescues — monsters missing **351→299**, coverage **75.1%** (904/1,203).
>
> **2g.4 — batch 19 ✅ (monsters wave 2).** Reptiles & amphibians: all 56 missing ids in `reptile_amphibian.json` (snakes, skinks, turtle hatchlings, tadpoles, the frog/toad radiation; volume placement — mega deathrattle 620 L and megafrogs 650 L at 96×96 native), `draw_reptiles.py` — every monster PNG on disk byte-identical across the parent re-run; voxelizer 56 materials, all `exact`, 0 missing (168 layers), parent re-verified via the prebuilt `target/release/voxelizer` binary (the root Cargo.toml currently lists another session's in-flight crates vehicles/transient/hardcases, not all on disk, so `cargo run` cannot load the workspace — noted, not touched). Monster overlay (baseline 299 reproduced): 56 direct, no rescues — monsters missing **299→243**, coverage **79.8%** (960/1,203).
>
> **2g.4 — batch 20 ✅ (monsters wave 3).** Insects & spiders: all 26 missing ids in `insect_spider.json` (mega ant/fly at 96×96, graboid, cicada, mantis, moth titan, cellar spider mother, dermatik deer host, larva tail), `draw_insects.py` — every monster PNG on disk byte-identical across the parent re-run; voxelizer (prebuilt binary) 26 materials, all `exact`, 0 missing (78 layers), parent re-verified. Monster overlay (baseline 243 reproduced): 26 direct, no rescues — monsters missing **243→217**, coverage **82.0%** (986/1,203).
>
> **2g.4 — batch 21 ✅ (monsters wave 3).** Nether creatures: all 52 missing ids in `nether.json` + `netherum/labyrinth_monsters.json` (dish-head, halo tangle, star vampires, void spider family at inherited 1,800 L, wells machine, reflections), `draw_nether_monsters.py` — every monster PNG on disk byte-identical across the parent re-run; voxelizer (prebuilt binary) 52 materials, all `exact`, 0 missing (156 layers), parent re-verified (an apparent 51 on the first parent run was a missing-trailing-newline bug in the parent's own id-list file, root-caused and re-run clean). Monster overlay (baseline 217 reproduced): 51 newly covered + `mon_nether_fish` upgraded from rescue to direct — monsters missing **217→166**, coverage **86.2%** (1,037/1,203).
>
> **2g.4 — batch 22 ✅ (monsters wave 4).** Feral humans & the zed clusters: all 40 missing ids in the feral-humans / zed_misc / zed_amphibian / zed_amalgamation files (ferals with uniforms+props and the shared vein-web/bloodshot tells; amalgamations on their own creature plans; zed frogs/toads), `draw_ferals_zeds.py` — every monster PNG on disk byte-identical across the parent re-run; voxelizer (prebuilt binary) 40 materials, all `exact`, 0 missing (120 layers), parent re-verified. Monster overlay (baseline 124 with 42 mammals/birds folders already landed): 40 direct, no rescues — monsters missing **124→84**, coverage **93.0%** (1,119/1,203).
>
> **2g.4 — batch 23 ✅ (monsters wave 4).** Mammals, birds & mutant mammals: all 46 missing ids in `mammal.json` / `bird.json` / `mutant_mammal.json` (species-hinted chicks through the generic chick abstracts; black cats, mole, porcupine, skunk; the three corpse placeholders as plain prone figures, no gore; mutant dogs/beaver/coyotes + standing raccoon mutant), `draw_mammals_birds.py` — all 46 resolved <100 L → 32×32 sheet; every monster PNG on disk byte-identical across the parent re-run (383 files); voxelizer (prebuilt binary) 46 materials, all `exact`, 0 missing (138 layers), parent re-verified. Monster overlay (folders-minus-batch baseline 126 reproduced): 42 newly covered + 4 rescue→direct upgrades — monsters missing **84→84**, coverage **93.0%** (1,119/1,203); the headline holds because 42 of the batch's folders had already landed inside batch 22's verification window. Remaining 84 ids enumerated exactly (yraq 7, string-dimension zeds, skeletons, zombie horses, scorched/fireproof zeds, weird dogs, devourers, lab ghouls, pupae, snails, kraken, misc singles, 2 pseudo test monsters) — finisher wave.
>
> **2g.4 — batch 25 ✅ (finisher wave, second half).** Cosmic & misc tail: 42 of the enumerated 84 (geometric yrax forms + golden monolith; devourers; kraken guard/queen; stalker octopus; zhark; snails; solar beetles; biollante; linelanders as vibrating filaments — yellow drawn yellow despite its `red` data color, per its name and siblings; singularity reclaimers; TALON secubots + Balthazar turret; misc singles; `mon_poly_special_test` as its described red dragon with a calibration-patch nod), `draw_finisher_cosmic.py` — placement 19×32² / 13×64² / 10×96² by resolved volume; every monster PNG on disk byte-identical across the parent re-run (425 files); voxelizer (prebuilt binary) 42 materials, all `exact`, 0 missing (126 layers), parent re-verified. Monster overlay (baseline 84 reproduced): 42 direct, no rescues — monsters missing **84→42**, coverage **96.5%** (1,161/1,203). The remaining 42 are exactly batch 24's assignment; batch 24's first dispatch ended empty (no script, no folders, no report — the established empty-completion pattern) and is re-run on Joey's word, in-session per the 2c/2d precedent or by fresh dispatch.
>
> **2g.4 — batch 24 ✅ (finisher wave, first half; re-dispatched).** Undead & lab tail: the final 42 ids (skeletons; four zombie horses at 550 L; scorched/rust/fireproof zeds; no-weakpoints pair as straight soldier/survivor zombies; inner-cabins grue at 435 L; string-dimension zeds + zomborgs; six weird dogs incl. antimemetic static-smudge and scorched hulk at 440 L; liquid/void cats; lab ghouls; brute/hulk pupae + near-identical decoys, hulk pupae at 875 L; both `pseudo_*` test monsters as range targets), `draw_finisher_undead.py` — placement 24×32² / 10×64² / 8×96² by resolved volume; every monster PNG on disk byte-identical across the parent re-run (467 files); voxelizer (prebuilt binary) 42 materials, all `exact`, 0 missing (126 layers), parent re-verified. Monster overlay (baseline 42 reproduced): 42 direct, no rescues — monsters missing **42→0**, coverage **100.0%** (1,203/1,203).
>
> **2g.4 ✅ COMPLETE — ZERO-DRIVE DONE (2026-11-15).** Final audit re-run (folder-based full-chain overlay, same method as every batch gate): **terrain 0 missing of 1,271, furniture 0 missing of 713, monsters 0 missing of 1,203 — 100.0% in all three drawn categories.** 806 sprites created across batches 1–25 (plus the butterfly/moth sets) under `coverage-audit/new-tiles/`, CC BY-SA, each batch parent-verified (byte-identical re-run, voxelizer all-`exact`/0-missing, overlay baseline reproduction). Gate 5.10's ≥98% terrain+furniture coverage criterion is **met at 100%**; the gate's TileDefs-form re-run remains ⛔ on Phase 0 tests 0.1/0.2 like every in-game gate. Out of scope per the audit's caveats: items (Phase 6 ground sprites), fields (2b shader effects), traps, overmap, vehicle parts (Phase 8).
>
> **2g.4 — SCOPE EXTENSION (Joey, 2026-11-15 17:09: "When I said all tiles I meant all tiles").** The drive reopens for the audit's remaining five categories. Exact remaining lists re-resolved with audit semantics (all-type entry load, `looks_like` inherited through copy-from, drawn folders in the universe — 46 ids were already rescued by chains through the first drive's folders): **items 3,742** (of 3,778 raw), **overmap terrain 336** (of 339), **vehicle parts 195** (of 202), **traps 121**, **field types 90** — **4,484 ids**. Same method and gates as batches 1–25: per-batch scripts with disjoint ownership, all sprites native 32×32 in `pngs_normal_32x32/<subdir>/` (`items`, `overmap`, `vehicle`, `traps`, `fields` — the shipped layout), parent re-verification (byte-identical, voxelizer all-`exact`/0-missing, overlay), records + scoped commits per batch. The three-category completion above stands as the record for the original scope; the drive's new finish line is 0 missing in all eight audit categories.
>
> **Extension progress.** TRAP1 ✅ (LIXA portal family, 41 ids; loader reads `traps/` — proven by the batch's voxelizer run): traps **121→80**. VEH1+VEH2 ✅ (98 ids; loader reads `vehicle/`): vehicle parts **195→97**. TRAP2 ✅ (40 ids): traps **80→40**. VEH3+VEH4 ✅ (97 ids): **vehicle parts 97→0 of 483 — CATEGORY COMPLETE.** FIELD1+FIELD2 ✅ (90 ids): **field types 90→0 of 172 — CATEGORY COMPLETE.** TRAP3 ✅ (40 ids): **traps 40→0 of 150 — CATEGORY COMPLETE.** OM1 ✅ (48 ids): overmap **336→288**. OM2 ✅ (48 ids): overmap **288→240**. OM3 ✅ (48 ids): overmap **240→192**. OM4 ✅ (48 ids): overmap **192→144**. OM5 ✅ (48 ids): overmap **144→96**. OM6+OM7+null ✅ (96 ids): **overmap 96→0 of 484 — CATEGORY COMPLETE** (incl. the null terrain, id ""). Items in progress (file-cluster batches): ITEM01–ITEM42 ✅ — items **3,742→2,084** (1,654 drawn + 4 looks_like rescues). A VM replacement wiped /tmp mid-drive; lists recovered/rebuilt durably (~/workspace/item-batches/, batches renumbered 08–91 for the remaining 3,395). Remaining: items 3,395.
>
> **Gate verdict: 2g.1–2g.3 DONE, conditions closed (2b follow-up, 2f condition, 2.10, 5.13); 2g.4 continues by design.** Remaining engine-integration work (billboard pass + UI quad + fx into the live windowed frame loop, one frame order: world → fields → billboards → post → UI) is now a wiring exercise over landed, gated crates — it is **Phase 2h** below (labeled a phase 2026-11-15 on Joey's word), not part of this phase.

### Phase 2h: Engine integration — the live frame loop (1-2 weeks) — added 2026-11-15, Joey's direction

> **Dependencies:** ⛔ hard — **2g.1, 2g.2** (fx passes + windowed textures in the engine), **4** (`ui-share`), **6** (`billboard`), **7** (`lighting`) · soft — 3 (live game data; the loop is built and gated on stub/snapshot data, live seating follows 0.1/0.2). **Parallel with:** 2g.4 (conflict-free), Phase 8 sub-tasks that don't need the loop.

**Problem (plain form):** every rendering layer now exists as a landed, gated crate — the voxel world (raster + RT), field effects, billboards, the lighting bake, UI compositing — but they have never shared a frame. The windowed app still draws the Phase 2 world alone, and the capture path composes world → fields → post with no billboards, no lighting, and no UI. This phase wires the crates into the one live frame loop, in the canonical order the earlier records fixed: **world → fields → billboards → post → UI**. Until it lands, "playable window" claims stop at crate level.

**Workstreams**

- **2h.1 — Frame-loop skeleton + windowed post route.** `crates/engine` takes dependencies on `ui-share`, `billboard`, `lighting`; the windowed path renders through the same offscreen composition as the capture path (this closes 2g.1 deviation 1 — windowed offscreen + post), so windowed and headless frames are one code path, per §7's capture-parity rule.
- **2h.2 — Billboard pass in-engine.** Entities from snapshot/stub data render through `billboard`'s pass between fields and post, depth-tested against world/RT depth (for RT, against the converted depth from 2g.1); alpha picking wired to engine input; the overlay `CompositeCache` fed by exported overlay lists.
- **2h.3 — Lighting bake in the frame.** The mesh path and the RT tree build consume `lighting`'s bake and the shared `VisibilityMask` (BLANK excluded from both, per the Phase 7 wiring contract, `docs/phase7-lighting.md` §12); day/night and weather state drive the bake inputs.
- **2h.4 — UI quad + input round-trip.** The `ui-share` store's current frame draws as the last pass, after post; engine input maps back through the FlatBuffers envelope, map clicks resolving to tiles via the crate's DDA picking. Gate 4.12's in-loop frame-cost measurement runs here.
- **2h.5 — Composed golden.** The 2g.3 harness gains a composed scene (world + fields + billboards + post + stub UI) so the integrated frame is regression-gated like every layer under it.

**Gate items**
- 2h.1: pass order asserted in code structure; a windowed-path frame and a headless capture of the same scene agree within the 2g.3 tolerance.
- 2h.2: engine-rendered crowd scene with real UltiCa sprites — the Phase 6 real-capture composition through the engine: behind-wall entity occluded, window entity visible; picking the hulk returns its entity id.
- 2h.3: day/night capture pair differs per the Phase 7 curve and only in lighting; a BLANK tile renders in neither renderer; MEMORIZED tiles show the pinned desaturation (0.30 saturation-keep × 0.45 brightness).
- 2h.4: stub UI frame composited over the post-processed world, terrain rect masked per the frame itself; a synthetic click round-trips to the expected tile coordinate; 4.12 measured in-loop and recorded (llvmpipe number; target-hardware re-measure noted as a condition).
- 2h.5: composed golden committed and green; the 2g.3 negative test still fails the gate.

**Exit criteria**: the windowed app — and the identical headless path — renders one frame containing all five layers in order, on stub/snapshot data. Seating live game data in the loop remains gated on 0.1/0.2, like every in-game gate.

> **Completion record — ✅ GATE PASSED (conditional) 2026-11-15** (implemented in the main session on Joey's word; gates in `crates/engine/tests/frame_gate.rs`, run standalone with `--nocapture`; full record in `docs/phase2h-integration.md`). One composition, `engine::frame::render_frame`, renders every frame in the canonical order; `CaptureArgs` gained `lighting` / `billboards` / `ui` scene inputs (all `None` = the pre-2h render). **2h.1**: `FRAME_ORDER` asserted in structure and execution; windowed route (render_frame + new `PresentBlit`) and headless capture of the composed scene both agree with the in-test composition at **0.000% mismatch**; `run_capture` delegates to the composition and all eight pre-existing 2g.3 goldens still match at 0 mismatched pixels (2g.1 deviation 1 closed). **2h.2**: real-UltiCa crowd in-engine — behind-wall skeleton occluded (frame delta 1 px at 1 LSB = packed-atlas UV rounding through the window blend, bounded in the gate), window zombie visible (0.028% px beyond tolerance), hulk picks as entity 8, skeleton as 7; billboard pass gained `prepare_with_view_proj` (draws with the world camera — the harness camera's 55° FOV disagreed with the engine's 60°). **2h.3**: day/night median ratios 0.543/0.591/0.701 vs the Phase 7 curve's 0.572/0.617/0.733, silhouettes identical; BLANK patch distance-to-background 45.5→5.0 (raster), 160.4→6.1 (RT); MEMORIZED saturation ratio 0.30 (raster — the pinned keep, exact), 0.28 (RT), raster brightness ratio 0.822 (vertex interpolation band). **Finding:** the per-vertex bake's resolution is mesh vertex density — a MEMORIZED patch inside the greedy-merged floor mega-quad has no vertices and changed 0 pixels; §12's identical-handling claim holds at vertices (RT is per-voxel, exact). Recorded in the doc; the per-block baker is the known lever if per-tile fidelity is ever required. **2h.4**: stub UI composited last and verbatim over the post-processed world; terrain rect masked per the frame itself (opaque-red store paint never appears); click round-trips to (6, 6, 1) (analytic first-hit tile; the pitch-45 variant correctly picks the slab-entry tile (7, 5, 1)); envelope round-trip clean; **gate 4.12 measured in-loop: median 8.23 ms, p99 36.56 ms** over 30 composed frames (llvmpipe) — after a measurement correction: `ui_stage` drains the queue before timing, otherwise the stage timer charges the world/fx GPU backlog (105.79/273.86 ms uncorrected). Windowed input routes clicks through the Phase 4 router into an in-process stub sink (protocol-shaped; the engine has no IPC client yet). **2h.5**: composed golden `house_composed.png` committed and green; the 2g.3 negative test still fails (1.730% detected). **Gate verdict: CONDITIONAL PASS.** Conditions: live game data seating (0.1/0.2); 4.12 target-hardware re-measure; windowed app not run live on the VM (lavapipe headless only); per-vertex bake resolution limit (above).

> **Follow-up addendum (2026-11-15, later — Joey: "do the rest of the things we can do from here").** The three items this record and Phase 8's left to the frame surface are now done, all parent-verified (full workspace 220 tests green, fork harness ALL PASS, package smoke PASS):
> - **Phase 8 crates wired into the frame loop.** `CaptureArgs` gained `vehicles` / `transients` scene inputs (both `None` = the pre-wiring render, byte-for-byte — all goldens unchanged). Vehicles stamp into the world in the world pass (raster + RT), *before* the shared visibility mask in the lit path, so a part on a BLANK tile is masked with the world's own blocks; transients ride the fields pass, merged into the one stable-sorted field-instance draw (fields-only order reproduces exactly); open-air columns resolve through `hardcases::floor_at` in one shared builder (`engine::snapshot_world`) feeding both renderers. CLI/windowed fixture flags `--vehicle-fixture` / `--transient-fixture <ms>`. Gates: `crates/engine/tests/phase8_wire_gate.rs` (stamping, composition, expiry, both renderers).
> - **2d `open`/`broken` subtile follow-up closed.** Source evidence (pinned `cata_tiles.cpp`, both vpart draw paths): state selection is `open` if open, else `broken` if broken — open wins. Implemented once in the canonical table (`mesher::connect::state_subtile`), consumed by `vehicles::part_state_subtile` and by the texture spec builder's new state-variant assignments (`engine::textures::build_texture_spec_state_variants` + `Material::side_layer_for_state`, same fallback rule as connection masks). Gate: real-UltiCa `vp_door` open variant's side layer is byte-identical to the composed open layer on disk (597 of 7,717 real materials already carry open/broken subtile layers).
> - **Per-block bake wired as an option — the mega-quad limit now has its lever.** `LightingScene.bake_mode` (`PerVertex` default = this record's behavior; `PerBlock` = `bake_chunk_map_with_offset` before meshing, so light boundaries are block-color boundaries the greedy mesher cannot merge across), CLI `--light-bake per-block|per-vertex`. Gate: a 3×3 MEMORIZED patch at the floor mega-quad's center — per-vertex saturation ratio 1.00 (the recorded limit, asserted as the A/B), per-block 0.301 (the pinned keep) with channels at the pinned presentation of the floor albedo. Per-block's known trade stands (a wall block takes its own tile's centre sample on both faces; per-vertex remains the leak-proof default). The 2h.3 condition is narrowed, not removed: the *default* path keeps the limit.

> **Sample scene + billboard floor-lift fix (2026-11-15, later still — Joey: "show me a sample scene with monsters in a room and outside, everything textured").** New sample world (`world::build_sample_scene`, `--snapshot sample-scene`): a 24×24 grass yard, a house (wood floor, door, three windows, roof), a pond, trees, a chain-link fence run; `BillboardScene::sample_crowd` (`--billboards sample:<dir>`) seats real-UltiCa monsters inside the room (zombie, skeleton, dog) and outside (two zombies, a hulk, a skeleton). Building it exposed a real bug the gates had never caught: **billboards were anchored at the z-level base, but a floor slab's top surface sits one block (1.0 world unit) above the base** — the slab box depth-occluded the lower ~⅔ of every creature in the composed frame, leaving a sliver at the surface line (measured: zombie fragment depth 0.99287 vs stored floor depth 0.99272 at the same pixels). Gate 2h.2 missed it because it asserts pixel *deltas*, which the sliver satisfied, and the 2h.5 golden baked the sunk render in as its own reference; the billboard crate's harness floor is a plane at the level base, so its gates couldn't see it either. **Fix:** the engine lifts billboard anchors, blob shadows, and item piles by `floor_surface_lift()` (WorldScale `floor_slab_blocks × block_size`) in `draw_billboards` — one place, both renderers. The composed golden was regenerated under `UPDATE_GOLDEN=1` (only `house_composed.png` changed); workspace 221 tests green. Captures: `~/workspace/your_files/cdda_voxel_sample_room.png` / `cdda_voxel_sample_wide.png`.

---

### Phase 3: Game-side export in the fork (3-4 weeks) (highest risk) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 0 and Phase 1 gates. **Parallel with:** Phases 2 and 5 (different codebases). This is the critical path.

**Tasks**
- Add `src/voxel_ipc/` with: server, serializer, shared-memory writer, input injector.
- **Tile definitions export** at connect: for each terrain, furniture, trap, field type, monster type, item type, vehicle part type: numeric id, string id, name, and:
  - flags — export the string flag set plus the resolved hot flags (`ter_furn_flag` bitset: transparent, wall-like, etc.); do not make the engine parse JSON flags itself
  - connection data — **resolved group names**, not raw JSON: in code these are bitsets (`connect_groups` / `connect_to_groups` / `rotate_to_groups` in `map_data_common_t`) indexed into the `connect_group` registry, so export a group table once and per-def group indices
  - open/close/bash *result ids* (`ter_t::open`, `ter_t::close`, bash → `ter_set`/`furn_set` result) so the engine can predict the post-bash voxel without simulating
  - `looks_like` id, so sprite fallback matches the game's own resolution
  - tileset sprite refs as **fg/bg entry references into the composed `tile_config.json`** (§8), including multitile and season variants — not bare sprite-name strings
- **Snapshot export**: for the player's view (radius ≤ `MAX_VIEW_DISTANCE` = 60, z within the cutaway window — *not* the whole 132×132×21 bubble by default; see §11 for the size math), export per tile: terrain id, furniture id, trap id, **field list** `(type_id, intensity)` (plus the `displayed_field_type()` for the cheap path), **item summary** (uppermost item id + stack count — full item list only on demand for examine/pickup UI, which the game's own UI handles anyway), graffiti flag, light/visibility as the game's own values: `lit_level` enum from the visibility cache (`DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT, MEMORIZED, BLANK`) plus the float seen value where smooth falloff matters. Remembered-but-not-visible tiles come from the avatar's map memory, not from live submaps — export them as a separate memory layer with terrain/furniture ids only, never live entities/items.
- **Delta export**: hook `map::invalidate_map_cache(z)`, `set_seen_cache_dirty` and `set_transparency_cache_dirty` (and vehicle/entity changes) to accumulate a **dirty-submap set**; at each send boundary, diff only those submaps against the fork's cached copy and emit tile changes. There is no finer-grained invalidation upstream to hook — this diff *is* the mechanism. Send boundaries: after each player turn **and** after any batch of non-player changes (monster moves, field spread, weather-driven changes) that occurred while the game was waiting/redrawing.
- **Entities**: source them from the game's creature tracking (`game` / critter tracker and the active NPC list), *not* from submaps — creatures are not stored per-tile. Per entity: stable id, type id, position, `FacingDirection` (**only `NONE`/`LEFT`/`RIGHT` — a horizontal flip, there are no 8-way creature sprites**), hp state, and the overlay list **in the game's own order**: `Character::get_overlay_ids()` / `monster::get_overlay_ids()` return ordered `(id, looks_like)` pairs, with mutation overlay ordering from `overlay_ordering` / the loaded tileset. The engine composes in exactly that order and must not re-sort.
- **Vehicles**: export per-vehicle: id, `face.dir()` as float degrees (continuous angle — see Phase 8 note), and a part list with **absolute tile positions computed game-side** from the pre-rotated mounts (`precalc_mounts` already does the pivot rotation; do not re-derive mount rotation in the engine). Per part: vpart id, absolute pos, mount pos, hp/damage state, open/closed and roof flags (the renderer itself splits parts into roof / no-roof layers).
- **Frontend mode flag**: `--frontend=ipc` that disables native map drawing (`draw_ter()` path) but keeps UI drawing (redirected to the offscreen buffer).
- **Input**: translate engine input events into the game's input path (`input_manager` → `input_context::get_raw_input()` / `handle_input()`) so existing keybindings, contexts and menus work unchanged. Mouse coordinates must be delivered in the same space the SDL backend would produce (display-buffer pixels → cell conversion is the backend's job today; the fork-side injector owns that conversion).
- **Save/load and world creation** must work through the same UI path.

**Deliverables**: real game running with engine connected; snapshot and deltas visible in engine logs; menus still functional via forwarded input (UI not yet visible, so use debug logging).

**Exit criteria**: replay test: a scripted input sequence on a **seeded test world** produces identical *tile-array* snapshot hashes across two runs. (Hash only the exported tile/entity data, not the full snapshot: weather, RNG-driven ambient state and timestamps will differ run to run unless the whole game is seeded, and chasing those is not a frontend bug.)

> #### Phase 3 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `fork/src/voxel_ipc/` — the exporter module, split so the core is game-API-agnostic behind `IGameAdapter`: TileDefs exporter (all kinds + connect-group table with per-def indices, resolved hot flags, open/close/bash result ids, `looks_like`, fg/bg refs), Snapshot exporter (radius ≤60 × cutaway z-window, sparse `(type,intensity)` fields + `field_index` + displayed field, top item + stack count, graffiti, exact `lit_level` + seen, **separate memory layer, terrain/furniture only**), Delta exporter (dirty-submap set from the invalidation hooks + **per-submap diff against a cached copy**; `map::shift` → epoch bump + forced resync), entity exporter (creature tracking, overlay order verbatim, facing NONE/LEFT/RIGHT only), vehicle exporter (game-side absolute part positions, float `face_deg`), server with Phase 1 framing (32 MiB cap, CDDA identifier, verifier first) and a **bounded outbox** (drop deltas → force resync via new epoch; snapshots never drop), input injector for the `get_raw_input()` path, and `voxel_real_adapter.cpp` — the fork-only TU against the real CDDA APIs, every one grep-verified in `fork-src/cdda` @ 6a3e59e0. Plus `fork/hooks/hook_patches.md` (H1–H7, one-liner hooks; all logic stays in `src/voxel_ipc/`), `fork/tests/voxel_exporter_tests.cpp` (real Catch2 cases for 3.1/3.3/3.5/3.5b/3.6/3.7/3.8 — **written, not compiled, not run**), a standalone mock-game harness in `tools/fork-harness/` (mock worlds: shelter, house, road+field, multi-storey+basement, vehicle on road; its snapshots decode with the real Rust `proto` crate), and `docs/phase3-fork.md` (file map, build lines, unblock steps). Ran in parallel with Phase 5 as §5 allows.
>
> **Test-gate results (Part II §6), parent-verified by re-running `tools/fork-harness/run.sh` (exit 0, ALL HARNESS TESTS PASS):** 3.1 ⚠️ harness-level (full-registry form awaits cata_test), 3.2 ⛔ blocked (in-game mod loading), 3.3 ✅ (3,888 tiles, 0 mismatches, fields as full maps), 3.4 ✅, 3.5 ✅ (lit exact, all 7 values), 3.5b ✅ (memory layer carries zero fields/items under live contamination), 3.6 ✅, 3.7 ✅, 3.8 ✅ (positions exact at 0°/37.5°/179.9°/359.99°, `face_deg` float-exact), 3.9 ✅ (**1,100 turns × 10 seeds**, mirror == fresh snapshot exactly), 3.9b ✅, 3.10 ✅ (direct submap mutation caught by the diff), 3.11 ✅, 3.12 ✅, 3.13 ⛔ blocked by 0.1/0.2, 3.14 ✅ harness (both runs hash `90b795eb6673ecae`; serde round-trip `2d31687aa671fee7`; Rust decode PASS), 3.15 ⛔ blocked in-game, 3.16 ⛔ blocked in-game (EOF/Shutdown paths implemented), 3.17 ✅ (25 deltas dropped under a 2 KiB outbox → resync forced → mirror recovered), 3.18 ✅ measured (parent's run: snapshot 5.31 ms at radius 60 × 3 z, typical delta 0.012 ms — on this noisy 1.5-CPU VM against a `std::map` mock; treat as indicative, re-measure in-game), 3.19 ⛔ blocked (needs the 0.2 baseline), 3.20 ⛔ blocked in-game (its resync mechanism is harness-proven via 3.9b/3.17).
>
> **Findings / deviations:** (1) **Root-cause bug caught by 3.17 during the run:** outbox eviction of a queued delta originally reported success, so the drop→resync policy never fired; `BoundedOutbox::push` now reports `DroppedDelta` whenever a push loses a delta. (2) **Schema gap — FIXED 2026-11-15 (Joey's call, commit after 7b26b40):** `TileChange` gained `displayed_field: DisplayedField` (appended last; additive/non-breaking, no protocol-version bump). C++ headers regenerated with flatc 24.3.25 (`--cpp --gen-object-api`) for both `tools/stub-server/` and `fork/src/voxel_ipc/`; the exporter's delta path populates it from the tile's displayed type/intensity; the harness mirror now reads it from the delta and **3.9 compares displayed fields strictly — the preserve-prior-value exemption is gone** (re-verified: 1,100 turns × 10 seeds, mirror == fresh snapshot exact). Proto sample updated; gates re-run green: workspace tests (19 suites), proto fmt, C++ round-trip, full fork harness. (3) **Trap registry:** upstream exposes no public `get_all()` for traps (`trap_factory` is file-local to `trap.cpp`); hook H7 adds a 3-line accessor — in-game 3.1 depends on it. (4) **Connect-group names:** no index→name reverse lookup exists in the pinned source, so the real adapter exports `group_<index>` names; the indices (the load-bearing data) are exact. Clean fix: record names at `connect_group::load` time. (5) **Seen float:** no public per-tile `seen_cache` getter; the real adapter quantizes from `lit_level` (BLANK=0, MEMORIZED=64, visible=255) — in-game 3.5 decides whether a cache-accessor hook is needed. (6) fg/bg refs export as −1 in the real adapter until Phase 5's composed-index mapping is wired into TileDefs (Phase 5 landed the same day — this wiring is now unblocked and is a named follow-up).
>
> **Gate verdict: CONDITIONAL PASS.** Everything provable in this environment passes at harness level, parent-verified. **Condition:** 0.1/0.2 must run on a build-capable machine, then the fork code gets its first compile, the Catch2 cases run under `cata_test`, and 3.2/3.13/3.15/3.16/3.19/3.20 run in-game (exact steps: `docs/phase3-fork.md` §5). Phase 4 may start on this gate subject to that condition, exactly as Phases 1/2/5 proceeded on theirs.

---

### Phase 4: UI compositing (1-2 weeks) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 3 gate (fork scaffolding and input injection) · soft — Phase 2 (engine window; stub frames allow early engine-side work). **Parallel with:** Phase 5.

Verified shape of the problem: `game::draw()` erases `w_terrain`, calls `draw_ter()`, runs draw callbacks, then `draw_panels()` draws the sidebar/message-log panels — each panel is a *fresh* `catacurses` window from the `panel_manager` layout (layouts are user-configurable; the message log is the panel with height −2; sidebar side is an option). Menus/dialogs are separate `ui_adaptor`s on the `ui_manager` stack. There is no single "UI surface" in game code to grab. **But** the SDL3 backend (`src/sdltiles.cpp`) already renders all of it into one terminal-sized `SDL_Texture` — `display_buffer` — and presents it with a single `RenderCopy`. Two routes:

- **Route A (preferred, spike first): backend capture.** In frontend mode, skip `draw_ter()` (leave the terrain window's rect transparent/cleared), let panels and menus draw normally into `display_buffer`, and read the buffer back at the present point into shared memory. Risks to prove in the spike: GPU→CPU readback cost/stalls per frame (mitigate: UI only changes on redraw — capture only on dirty frames; double-buffer), DPI/`SCALING_FACTOR` mismatch between display-buffer pixels and window pixels, and the shader/variant-pass work the current backend does around the buffer.
- **Route B (fallback): `catacurses`-level capture.** Implement the offscreen target one layer down: every `catacurses::window` renders into the shared buffer at its cell position, and the map window is suppressed. More fork code, but no GPU readback and it also works for a curses-backend build.

**Tasks**
- Implement the chosen route; transparent background exactly where the map used to draw.
- Double-buffered shared memory with a sequence counter and dirty rect; notification message on socket when a new frame is ready.
- Engine uploads the UI texture and draws it as a full-screen quad above the 3D view, with nearest-neighbor or integer scaling.
- Handle resize, DPI scaling, and fonts (keep the game's font rendering).
- Input mapping: key events (including text input and modifiers), mouse position in UI space, mouse clicks mapped to game coordinates when the game expects map-click input (requires an engine-side picking ray → tile coordinate → message; note the game also uses mouse clicks *on UI panels*, which stay in UI space — the fork reports which mode it is in via the UI frame/input context).

**Deliverables**: full playable game: menus over a flat-colored voxel world.

**Exit criteria**: create a character, start a game, open inventory, craft something, save and quit, all through the engine window.

> #### Phase 4 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `crates/ui-share` — the engine side of Route B (catacurses-level capture): a double-buffered shared-memory frame store on the 48-byte `UiShmHeader` (sequence counter + FNV-1a checksum; torn frames are rejected, never displayed), terrain-rect masking taken from the `UiFrame` itself (the rect is never assumed), dirty-rect texture upload proven pixel-equal to a full upload, DPI/resize scaling math, engine→game input mapping through the real FlatBuffers envelope, and DDA-line tile picking for map clicks. Wiring steps for the engine frame loop are in `docs/phase4-ui.md` (the UI quad is the last pass, after the post stack — the order 2g.1's record also asserts). Ran as a subagent in parallel with Phases 6/7 under disjoint file ownership (own crate + own doc).
>
> **Test-gate results (Part II §7), parent-verified by re-running `cargo test -p ui-share` (30/30 — 14 lib + 16 gate, the gate suite doing real lavapipe renders):** 4.1–4.11 ✅ at harness/logic level; 4.12 ⚠️ segment-only measurement (median 15.7 ms / p99 21.3 ms at 320×200 on llvmpipe — a software-renderer number, not the frame-cost verdict; the gate stays open until it can be measured in the real frame loop); 4.13/4.14 ⛔ blocked on 0.1/0.2 (they need the fork running in-game), not faked.
>
> **Gate verdict: CONDITIONAL PASS.** Everything provable without the in-game fork passes, parent-verified. **Conditions:** 4.12's in-loop measurement and 4.13/4.14 await the same 0.1/0.2 unblock as Phases 1/3; the exit-criteria playthrough (character creation → craft → save/quit through the engine window) is an in-game test by construction and lands with that integration.

---

### Phase 5: Voxelizer and textures (2-3 weeks, can overlap Phase 4) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 0 gate · soft — Phase 1 (TileDefs format). **Parallel with:** Phases 2, 3, 4 — this phase is deliberately *not* on the critical path; start it as early as Phase 1 allows.

See section 8 for the pipeline.

**Deliverables**: `voxelizer` CLI producing `assets/<tileset>/blocks.ktx2 (or png array)`, `materials.json`, `sprites.png atlas`, and `ATTRIBUTION.md`; engine loads them and renders textured terrain.

**Exit criteria**: houses, roads, fields, forests, and the starter shelter look coherent; unknown ids fall back to a visible placeholder and are logged.

> #### Phase 5 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `crates/tileset` — loader for BOTH forms: decomposed (per-sprite PNGs + entry JSONs, `tile_info.json`, `tileset.txt`; folder→sheet mapping carries per-sprite w/h/offset through everything) and composed (`tile_config.json`: fg/bg indices, list-of-ids entries, weighted variants, `additional_tiles`). Resolution follows `cata_tiles` as verified in source: exact → variant (`_var_*`) → season suffix at a pinned season → `looks_like` recursive (jump limit 10) → filler. `crates/voxelizer` — CLI + lib producing `blocks/` PNG layers + `blocks.json`, `materials.json`, `sprites.png` atlas + `sprites.json` (real w/h/offset per sprite), `ATTRIBUTION.md` (per-sheet sources + CC BY-SA 3.0 + third-party notices), `contact.png`, `missing.log`. Face rules per §8 (top = fg OVER bg; walls direct; floors bottom-row sample darkened ×0.65 + deterministic hash noise; bottom = side ×0.5); all 16 cardinal multitile masks + connect-group connection; alpha-aware premultiplied mips with edge clamping; furniture extrusion from alpha columns; `overrides.toml` (invalid override = located error); unknown → magenta/black checker + exactly one `missing.log` line. A generated fixture tileset (multitile wall, fg-over-bg, 32×64/64×64 oversized sprites, looks_like-only/variant-only/season-only ids, glass, fence, table+bookcase) with committed goldens (`UPDATE_GOLDEN=1` flow). Engine side: `crates/engine/src/materials.rs` loads/validates `materials.json`+`blocks.json` (unit-tested) — textured *rendering* is NOT wired (see deviations). Review artifacts in `phase5-review/` (UltiCa contact sheet + coverage summary). Ran in parallel with Phase 3 as §5 allows.
>
> **Test-gate results (Part II §8), parent-verified by re-running the gates:** 5.1 ✅ (fixture goldens byte-identical), 5.2 ✅ (parent's own second UltiCa run `diff -r` byte-identical to the first), 5.3 ✅ (fg-over-bg pixel-exact; fg-only fails by design), 5.4 ✅, 5.5 ✅ (16 masks + connect-group case), 5.5b ✅, 5.6 ✅, 5.7 ✅, 5.8 ✅, 5.8b ✅ (all sheet sizes incl. `large_ridden` −27y), 5.9 ✅, 5.10 ⚠️ partial (see below), 5.11 ✅, 5.12 ✅, 5.13 ⛔ not run as an image-diff gate (same status as 2.10's formal gate; contact sheet + a parent re-run of the headless capture regression — 65 quads, exit 0 — are the visual evidence), 5.14 ✅ manual — Joey reviewed the contact sheet 2026-11-15 and signed off ("looks fine for now"; provisional acceptance — problem ids can still be filed as overrides later), 5.15 ✅ (parent's run: 6.2 s voxelizer time on this VM, budget <60 s). Totals from the parent's own runs: `cargo test -p voxelizer -p tileset` **16/16**, workspace **57/57**.
>
> **Real UltiCa run (pixel-level):** the tilesets sparse checkout was expanded on demand — **14,451 PNGs** fetched (16,932 files = git index count). Parent-verified run: **7,717 materials, 26,389 layers, 14,450 atlas sprites, 0 missing**; 9,845 unique fg/bg/subtile sprite references across 4,650 entry objects (853 multitile) — 100.00% have a PNG. The pipeline consumes the decomposed form directly; compose.py/pyvips is **not** a dependency (composed-form input is supported via the loader).
>
> **Decisions / deviations:** (1) **Blocks ship as PNG layers, not ktx2** — no dependency-free ktx2 encoder exists; each PNG is one texture-array layer and the indices in `blocks.json` are identical either way (documented in the manifest). (2) **Filler is a coverage stage, not a material:** an id reaching only filler gets the placeholder + a `missing.log` line — otherwise 5.11 could never trigger. (3) **5.10's exact form stays open:** the measured universe is tileset entry ids + sprite references (100%); the "≥98% of base-game terrain/furniture ids" form must re-run against real TileDefs — Phase 3's exporter now exists, so this re-run is unblocked once the fork runs in-game (the CDDA sparse checkout carries no `data/json`). (4) **Textured rendering follow-up (named):** the mesher's `Block`/`Vertex` carry flat color with no UV/texture-layer field, so wiring textures into the render path needs a mesher change — deliberately not smuggled into this phase; engine-side material loading is in and tested, and the fake-house capture regression still passes.
>
> **Gate verdict: CONDITIONAL PASS.** All fixture gates and the real-UltiCa pixel run pass, parent-verified. **Conditions:** 5.10's TileDefs-based re-run, 5.13's formal image-diff, and the mesher UV/texture-layer follow-up before "engine renders textured terrain" is literally true. (5.14's manual sign-off landed 2026-11-15 — Joey accepted the contact sheet provisionally.)
>
> **Addendum (2026-11-16, Procedural Texture Pipeline Stage A — rendering fixes, Joey's go):** Phase 5 rework per `docs/procedural-texture-pipeline.md` (sprite as palette/exemplar source, not as the texture). Stage A fixed rendering before any procedural work. **Diagnosis confirmed by re-render:** the bad screenshot was the RASTER path — before/after sample-scene captures from the same camera (pristine HEAD worktree vs fixed tree) show the grass yard/roof/walls as one stretched sprite before, per-tile after (A/B sheet `docs/ab/stage-a.png`). **Raster:** mesher UVs are tile units (`face_uv` no longer divides by merged-quad extent; a merged quad spans [0,w]×[0,h], `Repeat` sampler wraps per tile); `raster-texture-sampler` mag Nearest, min/mip Linear, Repeat. **RT:** `rt-texture-sampler` mag Nearest (was Linear), Repeat; ray-cone LOD gained the draft's tunables — `LOD_BIAS −0.5`, `NDOTV_WEIGHT 0.5` — as uniforms mirrored by `lod::ray_cone_lod` (`mip_gate` 2e.2 analytic expectations updated to the weighted slant term). **Internal resolution:** `CaptureArgs.render_scale` / `--render-scale N` (capture + windowed): composition at size/N, integer upscale via the existing PresentBlit; scale 1 is byte-for-byte the old frame. **Gates:** new `stage_a_gate` 3/3 on lavapipe (mesher checker: merged-quad UV span == tile extent; render-scale-2 output blocks exactly uniform in both renderers; textured captures stay rich in both renderers); `uv_gate` 2f.1 amended — its old "merged quad spans [0,1]²" expectation encoded the bug; golden gate: flat/effects scenes 0 mismatched px, the 4 textured goldens regenerated via `UPDATE_GOLDEN=1` after visual verification of the actuals (mismatch 2.7–10.0% was the deliberate change); engine 51/51, treebuild+mesher all pass, workspace green. **Deviation (forced, recorded):** the draft's "Nearest mag + anisotropy 8x" is impossible in wgpu — anisotropy >1 requires all-Linear filters (validation panics otherwise, hit on the first run); Nearest mag won, anisotropy stays 1, oblique minification rides the mip chain. **Environment note:** this VM lost mesa-vulkan-drivers to a replacement and the system apt queue held the dpkg lock; gates ran on lavapipe side-loaded from the .deb (extracted under `~/workspace/stage-a-debs/`, `VK_ICD_FILENAMES` at its lvp ICD) — same 25.2.8 build as the previous system install. Nothing committed — Joey commits/pushes on his word as usual. **Stage B+ scope decision is Joey's, on this A/B evidence.**
>
> **Addendum (2026-11-16, Procedural Texture Pipeline Stage B + noise foundation — Joey's parallel go):** Built by two subagents in separate worktrees, merged and re-verified by the parent in the main tree. **Stage B:** `crates/voxelizer/src/color.rs` (sRGB↔linear, OKLab; published-primary values + roundtrip tests), `analyze.rs` (deterministic OKLab k-means palettes — FNV-1a content-hash seed, farthest-point init; baked-shading factor from top-vs-bottom L gradient; structure-tensor orientation), `classify.rs` (19 classes; rule order override → kind gate → id-pattern → keyword → OKLab nearest-centroid fallback → unknown; every result names the rule that fired). **Coverage, parent re-run** (`cargo run -p voxelizer --example classify_coverage`, live UltiCa decomposed tileset, 12,206 entries / 10,686 base ids; audit TSV byte-identical across two runs): block scope (t_/f_/vp_, 2,081 ids) **100.0% classified, 0.0% unknown** (51.6% ruled, 48.4% color fallback); terrain+furniture (1,425) 100.0%; whole universe 93.5% with only 37 fully unknown (fd_* sprites + facial-hair overlays, no loadable palette); 754 monsters gated `kind-not-block` (billboards). Stage B exit (≥90%) MET. Note: `assets/ultica_run1/materials.json` on disk is the older 7,717-material Phase 5 run — enumeration used the live tileset instead. **Anatomy:** `docs/sprite-anatomy.md`, measured on cited sprites — wall sprites are front-view illustrations with NO top-face pixels (Stage F end-caps must be generated); baked-shading direction is per-sprite (hence measured de-shading); ground sprites bake grass transition fringes at edges (sample centers); glass carries diagonal specular streaks. **Preview:** `voxelizer preview` subcommand renders sprite + palette strip + class band per id (no text in PNG — no font renderer in deps; the stdout table carries labels). **Noise foundation (Stage C first half):** `noise.rs` — splitmix64 RNG, `seed_from_str` (FNV-1a), periodic value noise, Worley F1/F2/F2−F1, fBm with integer lacunarity (fractional would break exact periodicity); all-integer hashing, no transcendentals → cross-platform bit-identical. `noise_gate` 8/8: edge tileability bit-exact, interior shift 1e-5 epsilon (f32 input rounding, documented). **Parent verification (main tree):** voxelizer lib 18/18, gate_tests 13/13, stage_b_gate 2/2, noise_gate 8/8, effects 4/4, tileset 5/5. **OPEN for Stage C:** there is NO `t_roof` terrain entry in UltiCa — the roof class's only palette sources are mapgen scene illustrations (e.g. `s_apt_roof`, shading factor 1.00); the roof recipe needs a palette call from Joey before Stage C builds it. Fallback skew note: over items, gray/brown ids fall back to `road`/`roof` classes — harmless (billboards), not material demand.
>
> **Addendum (2026-11-16, Procedural Texture Pipeline Stage C — first generated materials):** `crates/voxelizer/src/generate.rs` — recipe framework: family pooling (base entry's fg leaves + center/unconnected subtiles over bg; transition subtiles excluded per anatomy §2; `_season_*` entries NOT pooled — winter variants are snow sprites and polluted palettes, grass came out 18% snow-blue in testing), `deshade_palette` (cluster OKLab L compressed toward the weight-weighted median by the measured shading factor), content-hash seed, four recipes (grass, dirt, wood_floor, plaster) on the periodic noise foundation. **§8 metrics, parent re-run in the main tree** (`stage_c_gate` 1/1; targets seam < 1.15, mean ΔE_OK < 0.04, palette ≤ 24): grass seam 1.064 / ΔE 0.0061, dirt 1.094 / 0.0067 (center-50% crop drops the fringe-green clusters — mean shifts from [98,88,60] to clean browns), wood_floor 0.257 / 0.0073, plaster 0.682 / 0.0284 (closest to target — wall clusters are far apart in L); all palettes 10 colors; byte-identical determinism asserted. Wood floor initially failed seam at 2.682 (plank seams on the tile border) — fixed by shifting plank phase half a plank so the wrap edge is mid-plank; the target was NOT relaxed. **Engine check:** variant assets (top layers replaced for t_grass/t_dirt/t_floor/t_floor_primitive/t_wall) captured in both renderers from the sample scene (216 quads / 742 voxels, counts unchanged); the lawn's generated grass is visibly finer and less repetitive than Stage A's sprite repeat; contact sheet at `target/stage-c/contact.png` (ephemeral) via `examples/stage_c_showcase.rs`. Caveat: the scene's roof slab occludes most floor/wall tops — captures prove wiring + grass; per-material evidence is the contact sheet. Main-tree totals: voxelizer lib 21/21, gate_tests 13/13, stage_b 2/2, stage_c 1/1, noise 8/8, effects 4/4. Roof decision from earlier today stands resolved: no separate roof recipe — roof terrains inherit floor-family sprites via game `looks_like`; shingle (own UltiCa entry) joins Stage E.

> **Addendum (2026-11-16, mockup-driven scope — Joey's direction):** the finished-look sample-scene mockup (concept image, not an engine render) added two items to the pipeline plan, recorded in full in `docs/procedural-texture-pipeline.md` §10: (1) **Stage E** — circular ponds via shoreline autotiling: 8-neighbor-mask shore variants with curved waterlines over a dirt bank (Stage D height blend, order water < dirt < grass), matching dirt ring on adjacent land, shallow/deep by mask erosion; voxelizer-only, no shader changes, works in both renderers. (2) **Stage F** — decoration scatter: voxelizer-generated tuft/shrub alpha sprites placed client-side by world-coordinate integer hash (0–2 per exposed grass/dirt tile, density modulated by the Stage D macro height layer), rendered through the existing billboard pass, not pickable, no IPC/protocol change; frame-time verdict waits for the RX 9070 XT.

> **Addendum (2026-11-16, Procedural Texture Pipeline Stage D — quilting + macro ground):** Built by a subagent in a worktree, merged and re-verified by the parent in the main tree. `crates/voxelizer/src/quilt.rs` — Efros–Freeman image quilting on a torus: patch candidates wrap the exemplar, blocks sit on a toroidal canvas (step divides the output, so the last column/row wraps and overlaps the first by exactly the overlap), overlap error is squared OKLab distance against the wrapped canvas, every strip composited through a min-error boundary cut (DP over the error surface); deterministic (fixed candidate enumeration, (error, index) tie-break, seeded Rng). `generate.rs` gained `generate_quilted` and `generate_macro -> MacroTexture { image, height, palette, analysis }` for grass/dirt — additive only, Stage C's `generate` and gate untouched; exemplars reuse Stage C conventions literally (family_sprites pool, same center_crop, pixel-level de-shading into Stage C's albedo space). Height layer = per-pixel OKLab L as 8-bit gray (the stored-height decision from §3); `height_blend` is the CPU reference (effective height = stored height + weight bias x2, higher wins, ties to A) — engine/GPU wiring is Stage F. **Parent re-run in the main tree** (`stage_d_gate` 2/2; Stage C targets NOT relaxed): quilted 32px grass seam 0.952 / mean dE_OK 0.0034 (max 0.0102), dirt 0.782 / 0.0051 (max 0.0216), palettes 10; macro 128px grass seam 0.986 (height mean 112.6, std 12.74), dirt 0.960 (mean 120.7, std 4.55); byte-identical determinism asserted for quilted tiles, macros, and height layers; blend weight 0 == A exactly, 1 == B exactly, synthetic 255-height B patch punches through A at 0.5. Main-tree totals: voxelizer lib 24/24, gate_tests 13/13, stage_b 2/2, stage_c 1/1, stage_d 2/2, noise 8/8, effects 4/4. Contact sheet (`target/stage-d/contact.png`, via `examples/stage_d_showcase.rs`) visually checked by the parent: quilted tiles carry real sprite grain vs Stage C's mottled noise, height layers show real structure, blend strip transitions dirt -> grass with height-driven punch-through. **Finding (kept for Stage E/F):** a selection-rule attractor — on a torus an exact self-continuation (error 0.0) always exists, so relative-band and top-K rules collapse the synthesis into tiling one exemplar (period-16 match 16384/16384 px; first dirt macro seam 1.529); an absolute band failed the other way (flat sprites flood in, height std 1.70). Shipped rule: best candidate per exemplar, seeded pick among those within 4x global best + 1e-3, plus a wrap-relaxation pass — period-16 match ~14%, documented at `pick_candidate` in quilt.rs. Final geometry patch 12 / overlap 4 / step 8, chosen on measured margins.

> **Addendum (2026-11-16, Procedural Texture Pipeline Stage F part 1 — materials.json v2 + world-space macro ground):** Built by a subagent in a worktree, merged and re-verified by the parent in the main tree (full engine + mesher + treebuild + voxelizer suites green on lavapipe; v1 golden gate 0 mismatched px; `stage_f_gate` 3/3). **Contract:** v1 = bare JSON array, voxelizer default output byte-identical (gate-asserted); `--v2` wraps the identical array as `{"version":2,"materials":[...],"macros":[...]}` + 128px `macro_<layer>_color.png`/`_height.png` beside it; Grass/Dirt materials with poolable families get macros (span_tiles 4), unpoolable omitted (v1 behavior per material). Loader accepts v1/v2, rejects other versions with a clear error (unit tests). **Sampling:** mesher encodes macro tops in the layer word (bit 31 flag, bits 24-30 span, bits 0-23 layer) with world-position UVs from absolute grid corners (naive + greedy); raster samples a second texture array by world UV (Repeat); RT tex_albedo branches on the material macro entry for +Z faces, uv = (hitp + tree_origin).xy / span, LOD adjusted by -log2(span). Height layer uploaded in the same array, proven sampled via `--macro-debug-height` in BOTH renderers (gate: height-debug captures differ from color captures with a large near-grayscale population); height does not yet shade normal rendering. GPU proof: v2 vs v1 sample-scene captures differ >500 px in both renderers; macro UV phase exact every 4 tiles (8-tile slab u in [0,2]; offset slab u in [1,3]). Fixture tileset used for emission (has t_grass/t_dirt). NOT in part 1 by design: border height-blending, end-caps/decals, decoration scatter (§10.2) — Stage F part 2. The v2 format is also the carrier Stage G engine stamping will ride on.

> **Addendum (2026-11-16, Procedural Texture Pipeline Stage F part 2 — end-caps slice; Joey's parallel call with Phase 9, height-blend HELD):** Joey directed Stage F part 2 to run in parallel with Phase 9, split into three worktree slices (decoration scatter, wall end-caps, pond engine integration), with **border height-blending explicitly held until Phase 9 items 9.4/9.5 land** (it edits the same mesher/treebuild sampling files as the fine path; the other slices are logically and file-wise independent of the fine grid). **End-caps DONE @ 342f946 (NOT pushed), parent re-verified in the main tree (isolated target dir; full voxelizer suite green incl. gate_tests 13/13 — the worktree-only 5.1 failure was the known ATTRIBUTION.md absolute-path artifact — and `stage_f2_caps_gate` 2/2):** wall_like non-furniture materials at tile size 32 no longer use the sprite as their top layer (anatomy §4: a wall sprite is a face-on side illustration with no top-face pixels). `generate::generate_wall_cap` pools the family, de-shades its OKLab palette (k=10) by the measured per-family shading factor, and renders a tileable albedo-only cap per class (plaster stucco field; concrete Worley pore grain; brick plan-view brick tops, 8px rows/half-offset; wood top plate, boards along x; stone/metal/glass fall back to their Stage C/E renders; else generic mottled cross-section), seeded `family_seed ^ "wall_cap" ^ class`. Metrics on real UltiCa (targets NOT relaxed): t_wall seam 0.406 / mean ΔE 0.0287, t_concrete_wall 0.961 / 0.0127, t_brick_wall 0.123 / 0.0112, t_wall_wood 0.530 / 0.0199; palettes 10; caps pairwise distinct, opaque, ≠ sprite-as-top, byte-identical re-runs. Sides deliberately UNCHANGED (trim/baseboard bands sit at the wall's real top/bottom on a side face; the artifact was top-specific) — gate 5.4 rewritten accordingly (was: top == side, encoding the artifact; now: side == sprite, top != side, reason in the test). Fixture goldens moved only in the wall top layer + contact sheet; non-wall layers byte-identical, layer indices unmoved. Caveat for visual inspection: caps are opaque, so a glass wall's top reads as frame material. Engine textured house goldens (raster + RT) will move only after `assets/ultica_run1` is regenerated with this voxelizer — not yet done. Scatter + pond slices still in flight at this record.

> **Addendum (2026-11-17, Procedural Texture Pipeline Stage F part 2 — scatter + pond slices DONE, merged to main; pipeline doc §30/§31):** Built by subagents in worktrees `cdda-voxel-f2scatter` / `cdda-voxel-f2pond` (both off 5f7773e), held overnight on a GPU-verification blocker (VM replacement wiped the system lavapipe; the scatter frame_gate 2h.4 latency failure under parallel load could not be re-checked without an adapter), then merged on Joey's word 2026-11-17 after the parent re-ran the held gates on the restored adapter at idle load: **frame_gate 5/5 standalone (2h.4 median 8.04 ms / p99 16.37 ms vs budgets 50/100 — the earlier failure was parallel-load flakiness, independently corroborated by the pond agent hitting the same failure on an untouched code path and passing alone), pond gate 4/4 with capture counts identical to the agent's.** Merged as scatter `565b786` (clean cherry-pick) and pond `e015786` (two additive conflicts resolved against Phase 9: TextureStats keeps both `prop_colors` and `shore_variants`; scene_prep keeps the fine-grid imports). **Parent verification of the merged tree** (pinned at e015786 in a scratch worktree, isolated target dir): scatter gate 8/8, pond gate 4/4, frame_gate 5/5, golden 3/3, multitile 3/3, stage_f 3/3, caps 2/2, Phase 9 coexistence fine_gate 7/7 + phase9_render 3/3 + phase9_stamp 1/1; the only failure was voxelizer gate_5_1 in the scratch worktree (known ATTRIBUTION.md absolute-path artifact), re-confirmed 13/13 in the main tree. **Scatter:** voxelizer `decor.rs` generates tuft/shrub/flowering 32×32 alpha sprites from grass-family palette colors only (seeded `family_seed ^ seed_from_str(name)`), packed as a 96×32 billboard atlas beside materials.json (emission untouched); `billboard::decor::scatter` is a pure, order-independent placement function (FNV-1a + splitmix64, cross-checked against voxelizer::noise; 0–2 per eligible tile, jitter ±0.3, density from the tile's Stage F macro height, fallback 0.5 when height is unavailable — currently always, since macro height is GPU-side only; density over 3,600 grass tiles: h=0.1 → 849, fallback → 2,424, h=0.9 → 4,353); engine draws decorations as a third billboard pass with sentinel entity id `u64::MAX`, never fed to picking. Sample scene: 330 instances over 491 eligible tiles. Frame-time verdict still deferred to the RX 9070 XT. **Pond (§10.1 engine half):** the canonical 8-neighbor shore mask now lives in `mesher::connect` and `voxelizer::shore::mask_for` delegates to it, so generator and engine cannot disagree on bit order; v2 gains an optional `shores` table + `shore_<layer>.png` files — ONE shore set per run (255 masks, 0xFF excluded as identity) generated from the canonical t_water_dp/t_dirt/t_grass trio and attached to the primary water material; absent when any family fails to pool (v2-without-shore byte-identical, v1 untouched); the other ~40 water ids keep plain tops in this slice. Engine assignment lives in a shared scene-prep/spec path consumed by both renderers (deliberately NOT `snapshot_world::build_columns`, keeping it clear of Phase 9's stamping location); capture gate: raster bank pixels 1,079 vs 1 plain, RT 557 vs 0, interior water ~21,800 px in both. **Honest verdict unchanged: the pond still reads rounded-square, not circular** — shore.rs curves were not retuned (Stage H tuning item). **Stage F part 2 now stands at 3 of 4 slices landed; border height-blending remains HELD until Phase 9 item 9.5 lands.** **Hold RELEASED 2026-11-17:** Phase 9 item 9.5 landed as slice 3 (two-tree RT, §11b slice 3 record); border height-blending is unblocked and proceeds as its own slice whenever scheduled — it is not part of Phase 9. Pre-existing breakage noted in passing (not F2's, not fixed here): G.4's `c87b723` added Archetype::LaundryAppliance/VendingMachine without updating `examples/archetypes_showcase.rs`, whose exhaustive match no longer compiles — plain `cargo test -p voxelizer` fails on that example until its owner adds the arms. (Fixed 2026-11-17 @ e4cb1d5 by the Phase 9 chat.)

> **Addendum (2026-11-17, Procedural Texture Pipeline Stage F part 2 — border height-blending DONE, merged to main @ 0ace695; pipeline doc §33 — STAGE F PART 2 COMPLETE):** Built by a subagent in worktree `cdda-voxel-f2blend` off 3242629, parent re-verified and merged (cherry-pick clean over the Stage G coverage-fix record 37a354f). **Design:** pre-blended variant textures cannot carry the blend (macros are world-anchored, span 4 — every border phase would need pre-baking), so assignment reuses the shore/multitile virtual-id machinery while the decision runs per-pixel in both shaders: each samples own + neighbor macro color/height at the same world UV and applies the Stage D rule with a spatial weight (W = 0.5 at the border line → 0 half a tile in; neighbor wins iff h_nbr + W·2 > h_own + (1−W)·2, ties to own). RT carries neighbor macros in `TextureSpec::material_blends`; raster packs the blend into the mesher's top-face layer word. **Parent verification:** blend gate 5/5 re-run in the merged main tree on lavapipe with numbers identical to the agent's (raster interpenetration 78/80 px vs controls 0, band diffs 359, outside 1 px at Δ=1 — a greedy re-split flip, gate-bounded; RT 63/70, band 333, outside exactly 0; uniform scene byte-identical in both renderers; decision mirror == `quilt::height_blend` per-pixel, endpoints exact); frame_gate 5/5 (median 7.95 ms / p99 54.91 ms vs 50/100); full affected suites in the worktree 44 binaries green. Scope limits (in blend.rs): one border per tile, same-span pairs only, water excluded by construction (shore owns it), RT far borders fade via existing LOD node shading, raster 2048-macro-color cap with plain-word fallback. **Two failures in the worktree run, both explained:** gate_5_1 (known path artifact) and `stage_g_model_round_trip_fixture` — parent reproduced the latter IN THE MAIN TREE at pre-merge HEAD 3242629: a pre-existing regression from the coverage-drive merge, NOT this slice; a Stage G coverage fix round was already in flight in the main tree at merge time. CORRECTION (2026-11-17, parent re-check at 41a731b): the failure direction is the reverse of an early misreading — the gate asserts the fixture f_table emits NO model entry ("f_table does not segment"), and it now fails because the coverage drive's new builders DO emit a model for f_table; the gate's model-less premise is stale, not the segmentation. Still failing after coverage fix rounds 2–3 (those rounds gated on the voxelizer suite; this is an engine gate). **Stage F part 2 stands complete: end-caps, scatter, pond integration, height-blending.** Remaining texture-pipeline work is Stage H tuning (pond circularity, tank barrel ribbing) and the `assets/ultica_run1` regeneration.

> **Addendum (2026-11-16, Procedural Texture Pipeline Stage E — remaining materials + shoreline):** Built by a subagent in a worktree, merged and re-verified by the parent in the main tree. Eight new recipes in `generate.rs` (Concrete, Brick, Stone, Metal, Glass, Water, Road, Shingle; Water/Road join is_ground center-crop analysis; structured features phase-shifted off the wrap edge per the Stage C wood-floor lesson). **Parent re-run** (`stage_e_gate` 3/3; targets NOT relaxed; all palettes 10, byte-identical determinism): concrete seam 1.045 / dE 0.0006, brick 0.695 / 0.0130, stone 1.052 / 0.0069, metal 0.339 / 0.0051, glass 0.901 / 0.0121, water 0.437 / 0.0138, road 0.968 / 0.0064, shingle 0.816 / 0.0109 (family = t_shingle_flat_roof fetched by id). Main-tree totals: voxelizer lib 30/30, gate_tests 13/13, stage_b 2/2, stage_c 1/1, stage_d 2/2, stage_e 3/3, props 2/2, noise 8/8, effects 4/4. **Shore (`shore.rs`, §10.1 voxelizer half):** chamfer distance transform over the 96x96 neighborhood frame, radius-4 box blur, ±2.5px fBm wobble, height profile through fixed material heights (water 0.0 / dirt 0.45 / grass 1.0) via quilt height_blend, shallow lighten within 14px, depth_shade = field-at-center / 20; fields cached per neighborhood (512 masks). Gate: convex-corner texels dirt (never water), straight-edge waterline crosses y~10-13, all-water mask returns the input byte-for-byte, pond mosaic corner-tile water coverage 38.6% (< 60% target; square pond ~100%). Two fights recorded: (1) first field design (iterated polynomial smooth-min over per-square SDFs) under/over-subtracted and speckled dirt inside all-water tiles — replaced by the distance-transform field, corner coverage 85.3% -> 38.6%; (2) stone seam failed 3x (1.153/1.270/1.208) on cliff steps where wrap pairs cross hard 1px seam lines — fixed with a shallower anti-aliased seam (rolled-texture check 0.938, field stationary, not border-tuned). **HONEST VISUAL VERDICT (parent):** the pond mosaic (`target/stage-e/pond.png`) reads rounded-SQUARE, not circular — corners round, bank ring continuous, straights straight; rounding radius caps near one tile, so the mockup's circular pond is not matched at 3x3 size. A larger mid-edge bow is the lever; recorded as a Stage H tuning decision, not baked in. Engine-side mask assignment + §10.1 engine pond capture remain Stage F integration work.

> **Addendum (2026-11-16, Stage G inserted — Props 3D, Joey's direction):** on the lab mockup review Joey directed that props (furniture) become real 3D objects based on their sprites, eventually — not extruded-sprite models. Recorded in full in `docs/procedural-texture-pipeline.md` §12 as the new **Stage G**, between Stage F and tuning/sign-off (which moves to **Stage H**; Stage G references in §3/§6/§10.1 updated to match). G.1 box decomposition (sprite segmented top/front/side via analyze.rs, un-projected onto a proportioned cuboid; shares Stage F end-cap machinery), G.2 column voxelization (sprite pixel = column footprint on a 32x32 sub-tile grid, height from front-segment position + shading relief; plain voxels so both renderers consume it unchanged; carrier is the Stage F materials.json v2 per-id model blob; picking still resolves to the tile), G.3 parametric archetypes for hero props (containment tank, console, dissection table, bunk, shelf), textured from sprite palettes with sprite-front decals, prioritized lab set first, then jail, then city. Rules: deterministic generation only (ML image-to-3D explicitly out — untestable against byte-identical gates); inferred geometry is visual only, collision/movement/LOS stay with game tile data; unsegmentable ids fall back to the current extruded-sprite model rather than a guessed shape.

> **Addendum (2026-11-16, Procedural Texture Pipeline Stage G.1/G.2 — props generation side):** Built by a subagent in a worktree, merged and re-verified by the parent in the main tree. `crates/voxelizer/src/props.rs`: `segment()` (silhouette gating; boundary candidates scored on horizontal seam edge + local OKLab L-step; FIRST-strong-edge choice because furniture fronts carry repeated internal seams below the true one; structure-tensor orientation demoted to a bonus/penalty term — the cleanest tops are flat isotropic fields with no orientation to flip; low confidence returns None and the caller keeps the extruded-sprite fallback per §12 rules), `box_model` (G.1 cuboid, faces un-projected from their sprite regions, sliver-aware sides), `column_model` (G.2 front wall + top cap shell on the 32x32 sub-grid). Shared `PropModel` dense voxel grid. **Parent re-run in the main tree:** props_gate 2/2 — box front face equals the sprite front pixels on the sampled grid; column front projection equals the front silhouette EXACTLY (exhaustive, colors included); determinism byte-identical for segmentation and both models; f_chair and t_grass correctly return None; lib 27/27 incl. 3 new props unit tests; stage_b 2/2, stage_c 1/1, stage_d 2/2, noise 8/8, effects 4/4, gate_tests 13/13. Segmentation on the gate set: f_counter boundary 20 (top 0..20, conf 0.81), f_bookcase 27 (0.76), f_table 19 (0.86), f_fridge 17 (top only 9..17 on a 47-row front, 0.55 — weakest pass). **Known failure (documented in the module header):** f_locker segments at its mid door-seam (boundary 38, upper door read as a deep top) — the stacked-seam failure mode; excluded from the gate set, slated as a G.3 per-id override case rather than a cleverer global rule. Depth: documented 2:1 foreshortening heuristic (DEPTH_PER_TOP_ROW), clamped to 32. Engine stamping still waits on the Stage F v2 model-blob carrier. Showcase `examples/props_showcase.rs` -> `target/stage-g/props.png` presented to Joey 2026-11-16 for the §12 quality sign-off gate before G.3 archetypes.

> **Addendum (2026-11-16, Stage G weak-case fixes — Joey's sign-off condition):** Joey reviewed the G.1/G.2 showcase: "right direction, but fix the weak cases first." Fixes built by a subagent in a worktree, merged and re-verified by the parent in the main tree (props_gate 3/3, lib 32/32, all voxelizer suites green). **(1) Table open base (general rule, `base_plan`):** a front-extent column is grounded iff its bottom-row pixel is solid paint (alpha >= 254 — UltiCa under-table shadow is painted at exactly alpha 128); open fraction >= 0.25 + a solid header band (>= 0.90 solid columns/row) + >= 1 grounded column -> slab-and-legs box (header band + top face; each maximal grounded run extruded through full depth, front verbatim / interior x0.7 / back x0.55; under-space empty — gate-asserted at every depth). Measured: table open 0.81 (header 1 row) vs counter 0.10, bookcase 0.00, fridge 0.16, locker 0.00 — all stay solid. **(2) Column shells closed:** back panel at z=0 (front silhouette mirrored, per-pixel colors x0.7), side closure strips at each front row's opaque extremes across the full depth, bottom strip x0.5; interior stays hollow (unit-tested); front plane never written by closure — exhaustive front-projection == silhouette still passes for all five props. **(3) f_locker override:** static `BOUNDARY_OVERRIDES` table + `segment_for(id, sprite)` (the overrides.toml route did not fit — segment is a pure sprite function; the TOML is consumed at material-build time). Boundary 28 verified from the row profile: silhouette starts row 26 (dark rim, L 0.375), row 27 brightest (L 0.588) = the 2-row cap, body widens x 0..30 -> 0..31 at row 28. Override bypasses the confidence floor (honest computed score 0.33 — cap seam genuinely weak: edge 0.076, support 0.39) but keeps silhouette gating/side detection; gate proves bare segment still picks 38 while segment_for gives 28 and its box front matches the sprite front. **(4) Fridge confidence recalibration:** the margin-over-runner-up term scored fridge 0 (its door seams at rows 19/30/55 are stronger than the top seam — normal for tall furniture and exactly what first-strong-edge presupposes) and edge normalization (/0.30) undervalued its 0.203 seam; margin term removed, seam-support term added (fraction of seam columns with per-column OKLab break >= 0.08; fridge 1.00) at 0.15 weight, edge rescaled /0.25 at 0.35, small region-L term. New confidences: fridge 0.55 -> 0.75 (gate floor 0.65, justified headroom), counter 0.85, bookcase 0.84, table 0.92; f_chair/t_grass still None via both entry points. Showcase regenerated with f_locker as the fifth row and re-presented to Joey 2026-11-16 for the §12 sign-off.

> **Addendum (2026-11-16, Stage G unified solid model — Joey's direction):** Joey on the two-variant showcase: "This should be a real 3d model" — one model, solid; the column shell's open side wedge on the fridge was the visible defect, and the far-right column-model back view read most correct. `solid_model(sprite, seg)` in props.rs replaces both as THE model (box_model/column_model kept, doc-deprecated; the column one's note names the wedge failure): cross-section = front silhouette exactly, extruded through full depth d = seg.depth_cells(), filled at EVERY z — no hollow interior exists. Front plane verbatim sprite pixels; back x0.55; interior x0.7; side extremes via side_color (sliver un-projection or front x0.7). Top surface STEPPED (judgment call, documented): each column's top sits at its own topmost solid voxel, colored from the top region via top_row_for_z for z in 0..d-1; the top voxel's front plane keeps the front pixel, so the top reads as a rim behind the front-top edge. Open base reuses base_plan (table slab solid across, grounded legs only, under-space empty at every z incl. front plane). **Parent re-verified in the main tree:** props_gate 3/3 — solidity asserted EXHAUSTIVELY (every silhouette cell filled at every z): counter 11,840 voxels (32x12x32), bookcase 16,128 (32x36x14), table 2,944 (32x11x32), fridge 19,232 (32x39x16), locker 4,608 (32x36x4); front-projection color exactness + byte-identical determinism for all five; lib 34/34, all voxelizer suites green. **Joey's follow-up direction (2026-11-16):** next showcase shows front / back / left / right views, and unseen faces (sides, back) get EDUCATED inference rather than edge-stretch or mirrored front detail — that pass is the next G work item, then re-presentation for the §12 sign-off.

> **Addendum (2026-11-16, Stage G inferred unseen faces + 4-angle showcase):** Joey required educated inference for unseen faces and front/back/left/right showcase views. Built by a subagent in a worktree, merged and re-verified by the parent in the main tree (props_gate 4/4, lib 34/34, all voxelizer suites green). **Row-field rule** (`row_field` in props.rs): each front row's modal solid-paint color (alpha >= 254; outermost 2 front-extent columns trimmed; fallbacks untrimmed solid then opaque; ties -> smallest RGBA key, deterministic). Side interior cells = field x0.85; back plane = field x0.7 (old x0.55 mirrored pixel only when a row has no field — never in the gate set). Because the field is per-row, horizontal bands wrap at their heights; vertical detail never dominates a row mode and never wraps. Detected slivers (real evidence) still win on their side. The optional corner-post cell was skipped (keeps side cells uniformly field-colored and exhaustively assertable). **Gate evidence:** fridge 532 side cells == independently recomputed field x0.85; 195 back cells == field x0.7, 45 over front detail (handles/seams) the mirrored back would have ghosted; bookcase shelf band wraps on the non-sliver side at model y=5. **Renderer handedness bug found+fixed:** the showcase renderer's screen basis (right = view x up) mirrored EVERY render; corrected to right-handed (right = up x view) — FRONT now matches the sprite's handedness (fridge handles on the correct side); earlier sheets' model renders were mirrored (sprite column was drawn directly, unaffected). Showcase now sprite | FRONT | BACK | LEFT | RIGHT, orthographic azimuths at ~33 deg elevation, five props. Per-prop unseen faces: counter back = two-tone rear panel; bookcase back = dark panel with shelf banding, right side bands wrap; table back/sides keep open under-space; fridge back = plain panel with the freezer split as a band, no ghost doors/handles, left side plain metal; locker back = plain dark panel, no louver ghosts. **Remaining weakness (parent verdict):** f_locker depth = 4 sub-cells (2-row cap x 2:1 rule) — side views read paper-thin vs a real locker's ~1/3-width depth; a category minimum-depth rule is the candidate fix, Joey's call. Re-presented to Joey 2026-11-16 for the §12 sign-off.

> **Addendum (2026-11-16, Stage G fixes round 3 — depth floor, plain-panel backs, black artifacts root-caused):** Joey approved the minimum-depth rule, rejected rule-derived backs ("you can't fix that with a rule — make educated guesses"), and flagged a black column on the fridge RIGHT view + a black spot on the bookcase. Built by a subagent in a worktree, merged and re-verified by the parent in the main tree (props_gate 7/7, lib 36/36, all voxelizer suites green). **Black artifacts — root causes found with evidence (both color data, NOT the renderer; its face factors are 1.0/0.82/0.64/0.45 and offending cells were literally [0,0,0]):** (1) fridge column = transparent-black fringe: sprite column x=31 rows 23-55 is [0,0,0,128] (alpha < 254 = no color data) but the alpha>=128 silhouette test admitted it as structure; it extended the front extent to x=31 and was copied verbatim + slivered (model right flank L by z: [0.69, 0.66x4, 0.71x5, 0.00x6]). Fix: STRUCTURE test (opaque AND not fringe) — fringe never extends silhouette/extent, never a color source (side_color/row_field filtered too). Fringe census on gate sprites: fridge 36 px, counter x=31 + 3 px, bookcase/locker none; the table's painted shadow is interior and already non-structure under the base rule. (2) bookcase spot/bands = shelf-shadow rows as material: shelf interiors are fully-opaque pure black (280 px, 24% of front, palette cluster #2), so those rows' FIELD was [0,0,0], painted across sides and the whole back row at model y=6. Fix: shadow sources (L < 0.15) are recesses, not material — side sources borrow the nearest non-shadow row's field (ties prefer above); backs no longer use fields at all. **Minimum depth:** `depth_cells_for(width)` — silhouette >= 24 rows -> d = max(raw, ceil(width/3)); locker 4 -> 11; counter 32 / bookcase 14 / table 32 / fridge 16 unchanged (gate-pinned; deprecated box/column keep the raw rule). **Plain-panel backs (educated construction):** back plane = ONE panel in the front region's primary material (largest NON-shadow palette cluster from analyze_image k=5 — bookcase -> wood [69,48,10], correctly not its 24% black cluster), L modulated by stretched fBm (seed = sprite content hash, sampled (x/6, y) so grain runs horizontally; amplitude wood hue ±9%, flat/metal ±3%, else ±5%). TWO panels only when the top is a distinct slab: top/front mean OKLab distance >= 0.15 AND top >= 1/3 of silhouette height — counter (0.155, 20/32) and table fire; fridge teal cap (0.172 but 8/47) correctly does not split. Split rows scanned from the boundary by field-to-primary affinity with proportional fallback. Gate: bookcase back within 0.06 of primary with row-mean L variance < 0.001 (no banding); counter two-panel asserted via rim-band mean + achromaticity (per-cell distance there is a measured 0.058-0.065 coin flip); no-black gate over all 5 props (986-2,200 inferred side/back cells each, all L >= 0.10). Re-presented to Joey 2026-11-16 for the §12 sign-off.

> **Addendum (2026-11-16, Stage G albedo-only models — Joey's ruling):** Joey: "these models have light baked in. The engine should be providing the lighting. We should just be providing the textures." Built by a subagent in a worktree, merged and re-verified by the parent in the main tree (props_gate 8/8, lib 36/36, all voxelizer suites green). **Multipliers removed from solid_model:** side field x0.85 -> de-shaded field at FULL value; sliver x0.85 (inside shared side_color) -> raw sliver sample via new sliver_sample, de-shaded; interior x0.7 -> front pixel verbatim; back panel = de-shaded primary at full value (fBm grain kept — texture variation, not light); bottom = verbatim front-bottom-row pixels. Front plane + top face were already verbatim, untouched. Deprecated box/column keep their multipliers (reference-only). **De-shading:** per-row gain from the front region's mean-L profile (structure pixels, trimmed extent, sliver columns excluded, radius-2 smoothing), gain = median row L / row L clamped [0.80, 1.25], applied to OKLab L only; borrowed shadow fields de-shaded by their SOURCE row's gain; sliver pixels get row gain + re-anchor onto the row's de-shaded field (keeps the sliver pattern, removes its baked face shading; same-material assumption documented as a limit). Panel primary aligned with generate::deshade_palette: L re-anchored to the palette's WEIGHTED MEDIAN L over non-shadow clusters (cumulative-weight rule copied from weighted_median_l), same clamp — measurement forced this: a region-median-gain first cut failed because the synthetic's palette split the ramp into 3 near-tied clusters (0.300/0.295/0.295 at L 0.576/0.645/0.717) and the "largest" was a band edge no profile gain can correct; the weighted-median anchor (0.645) is the material center (true 0.651). **Synthetic albedo gate** (`stage_g_solid_model_albedo_only`): one uniform material [150,142,132] (L 0.651), lit top x1.45, front ramp x1.15->x0.85, sliver x0.78 further — measured side max OKLab distance 0.013 (tol 0.025), back max 0.023 (tol 0.05), back row-mean L spread 0.012 (bound 0.03), side L spread <= 0.025 both flanks; front plane asserted verbatim WITH its gradient (bottom-front L still > 0.05 below the material). Real-prop gates updated: fridge 532 side cells == independently recomputed de-shaded fields exactly; 1,136 back cells within 0.06 of the re-anchored primary; bookcase back variance 0.00002. Showcase: face differentiation now comes only from the renderer's factors (the engine-lighting stand-in). Re-presented to Joey 2026-11-16 for the §12 sign-off.

> **Addendum (2026-11-16, Stage G case-by-case side construction — Joey's ruling):** Joey: bookcase sides should be flat wooden slabs; "I am not sure you can fill in missing detail on the side with a global rule. It needs to go case by case." Built by a subagent in a worktree, merged and re-verified by the parent in the main tree (props_gate 9/9, lib 36/36, all voxelizer suites green). **Construction profiles:** `Construction { Panel, Structural, FieldWrap }` + `construction_for(id, sprite, seg)`; CONSTRUCTION_OVERRIDES pin table (starts empty — defaults serve all reviewed cases; pins accrete per reviewed case, mirroring BOUNDARY_OVERRIDES). Defaults: open base (base_plan) -> Structural; everything else that segments -> Panel. FieldWrap (the old row-field wrap) assigned to NOTHING by default — retained for per-case pinning (e.g. slatted crates). `solid_model_for(id, ...)` added; solid_model delegates via construction_for(None). **Panel flanks** reuse the BackPanels machinery (de-shaded primary, two-panel split included) with grain sampled at (z, y/6) so streaks run vertically; detected sliver still wins on its flank; shadow/missing sliver sample on a Panel flank falls back to the panel (was: row field). Gate set pinned by stage_g_construction_profiles: counter/bookcase/fridge/locker Panel, table Structural; solid_model_for == solid_model for all five (no pins yet). **Gate evidence:** fridge non-sliver flank 532 cells within 0.06 of de-shaded primary, row-mean L variance 0.00001 (its side==de-shaded-field assertion REPLACED — the field rule no longer colors fridge sides); bookcase right flank 420 cells panel (variance 0.00002, band-wrap assertion removed), left flank sliver un-projection cell-exact (312 cells); locker flank 315 cells panel (variance 0.00001, no louver banding); counter flank two-panel split (30 achromatic stone rim cells over 330 body cells within 0.06). Albedo synthetic disclosed adjustment: its left flank is Panel by default -> panel budget (per-cell 0.05, row-mean spread <= 0.03) while the sliver flank keeps the near-exact 0.025 budget; measured side max 0.023 overall. Re-presented to Joey 2026-11-16 for the §12 sign-off.

> **Addendum (2026-11-16, Stage G.3 archetypes — lab set):** Joey signed off the base model (options widget + heart) and G.3 proceeded. Built by a subagent in a worktree, merged and re-verified by the parent in the main tree (archetypes_gate 6/6, lib 36/36, props_gate 9/9, all voxelizer suites green). `crates/voxelizer/src/archetypes.rs`: `archetype_for(id)` explicit table + pins, parametric builders fitted to (sprite, seg) on the signed-off albedo/de-shading machinery (props.rs helpers reused via pub(crate), no behavior change). **Mapping:** ShelfUnit <- f_bookcase (primary), f_rack, f_utility_shelf, f_warehouse_shelf (looks_like -> f_rack), f_vitrified_bookcase + _talker (assigned, NO sprite exists). ContainmentTank <- f_standing_tank (primary), f_storage_tank; CDDA has NO true containment/specimen furniture — f_alien_pod does not segment (pod on a stalk), f_fvat_empty/full are barrels, both deliberately unassigned; specimen SKIPPED (no assigned sprite reads one cleanly). ConsoleDesk <- f_console (primary), f_console_broken, f_aut_gas_console + _o. DissectionTable <- f_surgical_table — NO sprite at all (no entry, no looks_like); fitted/showcased on f_slab; f_autodoc_couch excluded (boundary meaningless for fitting). Bunk <- f_bunkbed {2}, f_bunkbed_down {2}, f_triple_bunkbed {3} via looks_like to f_bed. **Fitted vs assumed (documented per archetype):** ShelfUnit boards from the front's dark recess bands (bookcase 5 bands at model-y {6,7},{13,14},{20,21},{27,28},{34,35} + base board = 6 boards; compartments genuinely OPEN — empty front plane + interior, back panel behind); Tank liquid line from the strongest row-mean color step in the middle band (fired, liquid_top 11/39; fallback 55% if no step >= 0.05), glass = palest blue-green cluster (lands achromatic light gray on this sprite), frame = darkest non-shadow cluster; ConsoleDesk desk_h 18 = top-region height (console rise), ONE screen rect [14,22,25,27] from the accent-pixel bounding box, accent = green cluster [0,196,0], body front = full sprite decal (gate-verified exhaustive); DissectionTable slab 3 thick, rim +1 over a recessed channel, pedestal 0.55 + plinth 0.75 (assumed); Bunk slab tops [8,23] (3 levels verified on the same sprite), mattress = brightest cluster, frame = wood-hue cluster, 2x2 posts + right-end ladder (assumed; sprite is top-down, no headboard). **Gate-design note:** the albedo darkness floor uses RAW palette clusters — de-shading lifts dark clusters toward the median by design (f_slab front primary L 0.521 vs de-shaded darkest cluster 0.588), so a de-shaded floor would be vacuous. Showcase examples/archetypes_showcase.rs -> target/stage-g/archetypes.png (sprite | FRONT | BACK | LEFT | RIGHT), presented to Joey 2026-11-16. Engine stamping (v2 models section) is the parallel track, landing separately.

> **Addendum (2026-11-16, Stage G v2 models carrier DONE; world stamping BLOCKED on architecture):** subagent-built in a worktree, parent-merged and re-verified (patch applied clean over the G.3 lib.rs state; stage_g_stamp_gate round-trip 3/3 — fixture f_bookcase model [32,22,20] round-trips size + solid count + EVERY voxel color exhaustively, f_table fixture sprite does not segment and stays model-less; loader units 6/6 incl. corrupt-model rejection; golden gate 0 px; engine+voxelizer suites green; RELEASE full-UltiCa --v2 run by the parent: 119 models emitted, keys {macros, materials, models, version}). **Format:** v2 gains "models" only when non-empty (v2-without-models byte-identical to Stage F v2; v1 untouched) — palette = distinct RGB in first-appearance order, voxels = hex string, 4 lowercase digits per voxel = palette index in PropModel (y*d+z)*w+x layout, 0xFFFF = EMPTY; loader validates hex length vs size product, palette range, zero dims, failing with named errors; orientation contract in materials.rs docs (model x -> tile-local east, y -> up, z = d-1 front -> south edge/+y). **BLOCKER (stated plainly):** the engine has NO sub-tile world grid and no furniture placement step — WorldScale (Phase 0) is 1 tile = 1 block footprint x 3 blocks per z-level, both renderers consume that block grid (gate 2c.6 shared filtered map), and NOTHING consumes even the existing sub_model extrusion data today: furniture currently renders as textured blocks. §12's "engine stamps at build time" assumed a grid that does not exist. Routes: (a) fine-grid world rework (mesher/tree/RT core — major), (b) model-instancing render path — props as per-instance voxel models at tile positions in both renderers, parallel to the billboard pass, block grid keeps terrain/walls/floors (RECOMMENDED; also fixes the latent sub_model gap). Joey's call; picking/LOS/collision untouched under either. Multitile-variant models also wait on this decision.

> **Addendum (2026-11-16, Stage G.3 REWORK — view-faithful generation; spec = pipeline doc §24):** Joey rejected 4 of 5 archetypes (only ShelfUnit correct), corrected two mockup rounds, and supplied the governing rule + a photo of the real f_standing_tank referent: **a prop model is a 3D model of what the sprite depicts** (sprite = 3/4 illustration of a real object; identify the referent first; no redesigns/inventions). Generator root cause: segment() assumes every sprite is a FRONT view. f_bed/f_slab are TOP-DOWN sprites (bunk + pedestal table were inventions — both archetypes DELETED), f_standing_tank is END-ON (horizontal leg tank: end cap + barrel + hatch neck + leg stands), f_console is Front but its archetype replaced the front with palette blocks. **Approved mockup reference sheets** (acceptance target, target/stage-g/mockups/): tank.webp, console.webp, slab.webp, bed.png — bed BACK = plain wood slab fully occluding the mattress (Joey), slab identical from all angles + level top (Joey), console approved as drawn. **Fix (Joey approved):** (1) view_for(id, sprite) -> Front|TopDown|EndOn before segmentation; heuristics propose, VIEW_OVERRIDES pins decide, default Front. (2) Per-view builders: Front = signed-off pipeline UNTOUCHED; TopDown = sprite as verbatim top face (bed: border frame down + mattress interior + head-end plain-slab headboard; slab: terrace rings = chamfer heightmap); EndOn = end-face cross-section extruded along depth (tank disc -> cylinder, hatch on top near front, leg stands; barrel length ~= cap diameter, pinnable). Console archetype rebuilt: full-front decal + stepped depth + depicted base compartment open. (3) Approved-model protection: regression gate pins voxel-data hashes of f_counter/f_bookcase(solid + ShelfUnit)/f_fridge/f_locker/f_table computed on the PRE-rework tree; Front path byte-identical code; new builders fire only for the pinned TopDown/EndOn ids. Acceptance: per-builder structural gates + 4-angle showcase vs the mockup sheets for Joey's sign-off.

> **Addendum (2026-11-16, Stage G.3 rework IMPLEMENTED — pipeline doc §25):** subagent-built in a worktree, parent-merged and re-verified in the main tree (voxelizer + engine suites ALL green, incl. updated stamp gate). props.rs: view_for (Front|TopDown|EndOn; ONLY VIEW_OVERRIDES pins trigger the new builders — unpinned heuristic proposals keep Front), model_for routing, topdown_model (terrace + framed; TOPDOWN_KIND_/HEAD_END_ pins), endon_model. archetypes.rs: ContainmentTank/DissectionTable/Bunk DELETED; ConsoleDesk rebuilt (front verbatim, depth 24->13 stepped, knee-hole cols 12..28 rows 0..8 open); ShelfUnit byte-preserved. v2 emit now calls model_for -> f_bookcase emits the ShelfUnit archetype instead of the solid (both approved; parent updated the engine stamp gate solid_model_for -> model_for). **Approved-model protection PROVEN:** tests/approved_models_gate.rs pins pre-rework FNV-1a hashes (counter 5f8ae4c1c989e335, bookcase solid 38f2cab4a45c56ed, bookcase ShelfUnit 4d61655ef4cc5e59, fridge 610f325837044965, locker 13b0bb8433473346, table 1258d2b757eb88cb) — all pass post-rework, zero voxels moved. As built: tank cap d=30, length=diameter, legs 4, front = cap mapped to disc (1.32x vertical rescale), neck x11..20 + olive fitting; bed frame [101,55,22], mattress verbatim +1 proud, headboard slab rows 0..5 to y=18 (mattress top y=8), back plane 100% frame gate-asserted; slab 4 chamfer levels x2 = h8, heights = pure chamfer function (sprite NOT 90-deg symmetric — gate uses independent BFS). Caveat (parent): tank barrel sides show ribbing from the cap sprite's row banding vs the mockup's smooth barrel + straps — Stage H texture decision. Showcase examples/views_showcase.rs -> target/stage-g/rework.png, presented to Joey 2026-11-16 for sign-off vs the mockup sheets.

> **Addendum (2026-11-16, Stage G.3 rework round 2 — bed sides + console sides):** Joey: the bed's foot end is a footboard and part of its side was missing; the console looks wrong from the sides. Fixed by a subagent, parent-merged and re-verified (voxelizer suites green; approved_models_gate hashes UNCHANGED; ShelfUnit untouched). **Bed:** footboard panel (full width x 2..29, frame ground -> top = mattress_top 8, top face verbatim); side walls rebuilt from the SILHOUETTE PERIMETER (every perimeter cell gets a frame wall ground -> mattress underside regardless of top color — the gaps were wood shades outside the frame tolerance built as floating mattress); gates exhaustive on both side planes + foot-end plane. **Console:** 55% base step deleted — common back plane z=0, desk/base fronts z=23, terminal front recessed exactly 4 (screen at z=19 verbatim, air in front); per-band flank materials from edge columns (terminal [66,63,60], desk [66,63,60] — same dark sliver in the sprite, rule applied literally; base [171,157,132] beige, base_h 11 fitted by first-beige-row). Old stepped-depth gate assertions replaced by the new silhouette gates. Showcase regenerated (target/stage-g/rework.png), presented to Joey 2026-11-16.

> **Addendum (2026-11-16, process rule — two-pass prop generation; pipeline doc §27):** Joey accepted the G.3 rework round 2 ("this is fine") and set a standing rule for all prop objects: **procedural generation is the FIRST PASS only; after it, an agent must review the rendered output (4-angle showcase vs sprite/mockup) and make manual tweaks (per-id pins, overrides, fitted-parameter corrections) until it looks correct.** A green structural gate is not acceptance — the G.3 archetypes and the round-1 bed both passed their gates while looking wrong. Loop: generate -> render 4 angles -> compare against the depicted object -> tweak -> re-render; Joey's sign-off is final acceptance for showcase sets.
>
> **Addendum (2026-11-16, Stage G.4 — jail + city archetypes; pipeline doc §29):** Built by a subagent in worktree `cdda-voxel-g4`, parent-merged and re-verified in the main tree (g4_gate 3/3, archetypes_gate 3/3, props_gate 9/9, views_gate 4/4, approved_models_gate 2/2 hashes unchanged, lib 36/36). **Audit first, per the faithfulness rule — all three primaries DROPPED with evidence:** f_toilet (unsegmentable; mixed ¾-front/plan drawing), f_shower (view genuinely ambiguous: plan tray + flattened elevation fixtures), f_armchair (whole object, clean front, but arm/seat junction has no measurable signature — proportions would be assumed wholesale); also dropped f_dresser, f_filing_cabinet, f_entertainment_center; f_cupboard excluded as multitile alongside f_sofa/f_bathtub/f_sink/f_bench/f_metal_bench/f_desk. **Built from the substitute list, 2 archetypes / 8 served ids:** LaundryAppliance (f_washer, f_dryer, f_autoclave_full via looks_like; closed enamel cabinet, front verbatim, fitted control/plinth bands, per-side flank materials, depth 11) and VendingMachine (f_vending_c/_reinforced/_o + the two `_off` looks_like variants; front verbatim, dispensing bay a genuine recess rows 7..9 cols 5..23 with back wall, fitted per-side flanks; assumed depth 16 / bay depth 8, documented). §27 second pass: flank materials fitted per side after pass 1 pooled them (both vending flanks had taken the white highlight color). archetypes.rs diff is pure additions; props.rs and the v2 carrier untouched. Showcase `examples/g4_showcase.rs` -> `target/stage-g/g4.png`, presented to Joey 2026-11-16. **JOEY SIGN-OFF GRANTED 2026-11-17** — G.4 accepted as built.
>
> **Addendum (2026-11-17, Stage G coverage drive — prop models 119 -> 222 of 351; pipeline doc §32):** Parent audit (`examples/prop_coverage_audit.rs`) proved all 232 unmodeled furniture materials fail at `segment() == None`. Built in worktree `cdda-voxel-gcov` (branch `prop-coverage`), parent re-verified (audit 222, TSV diff 119 identical + 103 new, battery 63/63, full voxelizer suite 98/98 in main tree), committed `fce048e`, merged to main as `3242629`. New `coverage_model` fallthrough in `model_for` (fires only when `segment_for` is None, id not view-pinned): seamless solid 71 ids (measured firing criteria; synthesized segmentation, edge-row top, precedented depth rule), plan slabs 16 via `PLAN_SLAB_PINS`, panel slabs 16 via `PANEL_SLAB_PINS`, 11 `SEAMLESS_EXCLUSIONS` pins. §27 pass 2: wreath excluded on sheet review (223 -> 222); f_toilet pedestal fixed via `force_closed` rebuild (open-plan dropped-structure detection; pre-existing callers unaffected). f_toilet/f_armchair are BASE-treated only — the G.4 archetype drops stand. Remainder 129 unmodeled by documented reason (vegetation 46, skeleton/below-criteria 55, exclusions 11, ambiguous/organism 17). Showcase `examples/prop_coverage_showcase.rs` -> `target/stage-g/prop_coverage.png`, presented to Joey 2026-11-17 for sign-off. **FIX ROUND on Joey's strip review (2026-11-17, committed `0484649`, suite 100/100, total still 222):** seamless tops were edge-row streaks — now one fitted material per top (gate-asserted); round-plan objects were square — new `cylinder_model` + 17 `CYLINDER_PINS` (barrels, vats, heater, keg, trashcan, boulders...) with inscribed-disc footprints. All 103 strips re-rendered and re-presented for sign-off. **FIX ROUND 2 on Joey's second review (2026-11-17):** cylinder tops still wrong — the barrel sprites' top band is the container's INTERIOR (dark; fire paint for fd_fire), so extruded columns capped them with interior paint. Two new archetypes replace the treatment for 8 ids: **HollowCylinder** (barrels x4, trashcan — real cavity, contents surface fitted from the top band at 0.55*diameter below the rim; fd_fire's surface is the fire) and **ClosedCylinder** (keg, fvats — flat fitted lid). Archetype set now 6 archetypes / 26 served ids. Strips re-presented for sign-off. **FIX ROUND 3 on Joey's two further reviews (2026-11-17, committed `850ede7`, suite 103/103, total still 222):** (a) cylinders STILL capped wrong — the archetypes now measure the sprite's top ellipse band and build only the wall below it; contents surface sits just under the rim so the fd_fire fire reads as the sprite's disc. (b) Irregular round shapes (Joey on f_alien_gasper: front must match back, no flat back/sides) — new `lathe_model` + 12 LATHE_PINS (gasper, pod, egg sacks, kilns, boulders, fungal clump): per-row discs following the profile, hollow mouth where the sprite draws one. Strips re-presented for sign-off.
 **HORIZONTAL BARREL (Joey's identification, 2026-11-17, committed `e5f1328`):** f_fvat_empty/full are a horizontal alcohol barrel on a stand (Joey supplied a photo of the real object) — new `HorizontalBarrel` archetype: sprite fitted far-curve/body/head 5/20/18, diameter 24, built lying down, tap protrusion from the head's lower-half blue cluster; keg stays ClosedCylinder. **FIX ROUND 4 on Joey's per-model reviews (2026-11-17, committed `fc3f750`, suite 105/105, total still 222):** barrels un-squished (full band height; center-column opening measure — 55-gal's true ellipse is 8 rows; rim section in clamped wall paint around a well, contents `drop` below the rim); base [0,0,0] outline paint de-voided in column_paint + blob; side-pattern wrap (`wrap_side_faces`/`wrap_row_pattern`) — kiln cobbles and barrel pattern now run front -> side -> back, pure black never wraps; gasper mouth ring takes one rim color (dashed-crown artifact gone); boulders x4 + fungal clump + egg sacks x2 OFF the lathe onto `blob_model` (irregular per-column depth, grounded dome taper, fBm lumps; gate asserts asymmetry a revolution cannot have). Lathe keeps gasper/pod/kilns. Strips re-rendered; sign-off pending.
>
> **Addendum (2026-11-15, coverage audit — Joey's ask):** 5.10's game-id form was measured against base-game `data/json` ids directly (CDDA sparse checkout expanded for `data/json` only; script `tools/audit_missing_tiles.py`, full report `coverage-audit/REPORT.md`). The tileset-universe 100% was circular: against game ids, shipped-UltiCa coverage is **terrain 84.1% / furniture 75.6% / monsters 61.1%** — 5.10's ≥98% criterion **fails** and stays open with a measured missing list (`coverage-audit/missing.json`). Two distinct gaps: (1) UltiCa genuinely lacks tiles for newer content (alien meadow, string-dimension, nether labyrinth, vitrified, stack/mound furniture family, 468 monsters); (2) **our decomposed loader misses compose.py's `--use-all` behaviour** — the official UltiCa build promotes every unreferenced PNG to a tile entry, so 258 terrain/furniture/monster ids (e.g. `mon_antlion_larva`, `mon_creeper_hub`, `t_window_no_curtains`) are covered in the shipped game but invisible to `crates/tileset` (loader coverage: terrain 77.3% / furniture 62.0% / monsters 54.9%). Loader follow-up DONE 2026-11-15 (Joey's word): `load_decomposed` synthesizes use-all entries (`synthesize_use_all_entries` in crates/tileset; referenced set from fg/bg incl. `additional_tiles`; JSON entries win; fallback sheet excluded; loader test `use_all_synthesis`); real-UltiCa voxelizer run after the fix: 12,206 materials / 39,856 layers / 14,450 sprites / 0 missing, byte-identical re-run, audit examples (`mon_antlion_larva`, `mon_creeper_hub`, `t_window_no_curtains`, `t_ladder_up`) all `exact`. Audit strict-universe correction same day: shipped universe is 12,925 ids (7,531 JSON + 5,395 unreferenced stems), not every-stem; only vehicle parts (202 missing) and items (3,778) moved. First created tiles: the full butterfly lifecycle (`mon_caterpillar_butterfly`, `mon_butterfly_cocoon`, `mon_butterfly`, `mon_butterfly_giant`, `mon_butterfly_titan` — previously no entry, no PNG, no `looks_like`) drawn from their game descriptions in `coverage-audit/new-tiles/`, verified through the release voxelizer (5 materials, all `exact`, 0 missing). The moth lifecycle followed the same day on Joey's word (`mon_caterpillar_moth`, `mon_moth_cocoon`, `mon_moth`, `mon_moth_giant` — bark-patterned wings, silk cocoon; combined run 9 materials, all `exact`, 0 missing).

> **Addendum (2026-11-17, sign-off + rest-of-props phase — Joey's instructions):** Joey signed off on the 222 modeled props "for now" (egg sacks PAUSED mid-fix at the egg-pile + side-seats build; cone profile rejected; flat-side question left open). Two standing rules enter the workflow from this point:
>
> 1. **Save renders of the models from every angle for review.** Every model gets its [sprite | FRONT | BACK | LEFT | RIGHT] strip rendered and saved into the repo (`docs/prop-views/strips/`, regenerated by `examples/prop_strips.rs`) — review happens from saved renders, not ad-hoc screenshots.
> 2. **The review workflow (Joey's words, verbatim):** "once models are generated look at the model from all angles. Decide if it matches the source sprite/looks good. If it does, present it to me. If it does not, create a mockup of what the model should look like and generate the model from that"
>
> **New phase: model creation for the rest of the props** — the 129 unmodeled furniture ids (inventory: `docs/prop-views/rest-of-props-inventory.md`; classes: exclusion pins 11, ambiguous view / organism / section ~17, vegetation / seasonal / fungal flora ~46, skeleton-thin / below-criteria ~55). The drive-era reasons for leaving them unmodeled (no invented mass for vegetation, below firing criteria for thin structures, multitile exclusions) are superseded by Joey's instruction to model them; each id still goes through the workflow above individually, and a mockup precedes generation whenever the first build does not match the sprite.

---

### Phase 6: Billboards (1-2 weeks) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2 and Phase 5 gates · soft — Phase 3 (real entities; fake entities suffice during development). **Parallel with:** Phase 7.

> **Renderer note (2026-11-15, Phase 2c):** billboards stay rasterized quads, composited over the ray-traced frame and depth-tested against the traced depth buffer — they are the one place original sprites appear as-is, per this phase's design; field *effects* are never billboards (Phase 2b).

- Instanced camera-facing quads (cylindrical billboarding: rotate around Y only) for creatures, NPCs, items on ground.
- Overlay compositing: the engine composes layered sprites (body + clothing + wielded) into a single billboard texture, cached by overlay signature. **Draw order is verified, not guessed** — the tile layer order in `cata_tiles::draw()` is: terrain → furniture → graffiti → trap → part_con → field_or_item → vpart (no roof) → vpart (roof) → critter → zone mark → zombie-revival indicator; and within a creature, the base sprite first, then `get_overlay_ids()` in returned order. For billboards only the creature part applies, but the same ordering source governs what the game's own examine/look output expects.
- Facing: `FacingDirection` is `LEFT`/`RIGHT`/`NONE` only — mirror horizontally for `LEFT`, never look for 8-way directional creature sprites (they don't exist in this system; rotation exists for *terrain/vehicle* tiles via multitile/rotation, not creatures). Use each sprite's real width/height/offset from `tile_info.json` (§8) — creature sprites are not all 32×32.
- Shadows: blob shadow decal under each billboard.
- Sorting and alpha: alpha-test with dithering or sorted transparent pass; depth-write for cutouts.
- HP bars, status icons and names as billboard overlays or left to the game's UI.
- Animation: sprite frame animation if the tileset defines it (weighted/animated fg entries in the tile config); simple bob/hit-flash effects in engine.

**Exit criteria**: a crowd of zombies renders correctly sorted against walls and windows.

> #### Phase 6 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `crates/billboard` — sprite atlas loading in the voxelizer's `sprites.png`/`sprites.json` format; the overlay compositor (base sprite first, then overlays in the game's own returned order — never re-sorted) with a signature cache; cylindrical (Y-only) billboard instances generated from each sprite's real width/height/offset; a depth-test-only wgpu pass (tests against world/RT depth, writes none); blob shadows with the under-roof suppression rule; alpha picking; item piles. Wiring steps in `docs/phase6-billboards.md` (frame order: world → billboards → fields → post → UI in that doc's terms; the canonical order recorded by 2g.1 is world → fields → billboards → post → UI). Ran as a subagent in parallel with Phases 4/7 under disjoint file ownership.
>
> **Test-gate results (Part II §9), parent-verified by re-running `cargo test -p billboard` (14/14 incl. GPU gates):** 6.1/6.2/6.3/6.5/6.6/6.8/6.9/6.11 ✅; 6.7 ✅ at fixture level; 6.4 ⚠️ partial — the engine half passes, the game-side capture half is blocked on 0.1/0.2; 6.10 ⚠️ llvmpipe numbers only (frame-time evidence is target-hardware, as everywhere). Captures: `captures/billboard_crowd.png`, `captures/billboard_stress.png` — rendered against the crate's hermetic **fixture atlas** (synthetic rectangles), which is what the gates are designed to measure.
>
> **Addendum (2026-11-15, Joey's catch):** the fixture captures were presented without flagging that they carry no real art — Joey's correction ("There are no sprites or textures in the scene") was right. A real-atlas harness now exists (`crates/billboard/src/bin/real-capture.rs`, fails loudly on any sprite id missing from the atlas) and `captures/billboard_real_crowd.png` renders the same crowd layout with the real UltiCa atlas: `mon_zombie_1/2/3`, `mon_skeleton`, `mon_zombie_cop`, the 64×96 `mon_zombie_hulk`, `mon_shadow`, `mon_dog`, a real `overlay_effect_bleed` composite, the window zombie visible and the behind-wall skeleton correctly absent (10,827 distinct colors at 1280×720). Fixture captures are gate evidence for the math; only real-atlas captures are scene evidence.
>
> **Gate verdict: CONDITIONAL PASS.** Conditions: 6.4's game-side half (0.1/0.2), 6.10's target-hardware timing, and engine integration of the pass (the crate is landed and gated; the frame loop wiring is Phase 2h).

---

### Phase 7: Lighting, FOV and atmosphere (2 weeks) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phase 2 and Phase 3 gates (for its own gate) · soft — Phase 5 (flat colors work first). **Parallel with:** Phase 6.

> **Scope split (2026-11-15, Phase 2c):** with a ray-traced renderer, *illumination* (sun shadows, AO, emissive bounce feel) is computed by the tracer in Phase 2c. This phase keeps what only the game knows: **visibility and memory** — `lit_level`/seen as the source of truth for what may be shown at all, remembered-vs-seen presentation, day/night and weather *state* from the game. The tracer must never reveal a tile the game marks unseen, no matter how the light rays travel.

- Use the game's per-tile `lit_level` and seen values as the source of truth (do not recompute FOV). Note the game also computes *colored* light (`light_color_rgb`, float RGB energy) — v1 of the frontend may quantize to intensity + a single tint per tile, but the protocol field should be sized so colored light can be added without a breaking change.
- Bake light into vertex data or a per-chunk light texture; smooth with simple averaging across neighbors.
- Unseen-but-remembered tiles (`lit_level::MEMORIZED` / map-memory layer): desaturated and darker. Never-seen tiles (`BLANK`): black / not meshed.
- Day/night tint, weather overlays (rain particles, fog), flashlight cones from game light data.
- Optional later: screen-space ambient occlusion, shadow mapping from sun.

> #### Phase 7 completion record (2026-11-15, per the maintenance rule)
> **What was actually built:** `crates/lighting` — deliberately pure CPU (no wgpu): the `lit_level`→intensity mapping as **disjoint bands** (DARK .06–.14 / LOW .18–.28 / BRIGHT_ONLY .34–.46 / LIT .54–.70 / BRIGHT .80–1.00) with the seen value lerped inside its band, so baked brightness is strictly monotonic in both the enum and seen; `LightSample` carries an RGB tint, so colored light (`light_color_rgb`) is representable without a breaking change. MEMORIZED is pinned at saturation-keep 0.30 × brightness 0.45. BLANK is excluded from meshing AND from the RT tree by one shared `VisibilityMask` (`is_meshed == include_in_tree`) — the tracer can never reveal an unseen tile. The no-leak/smoothing pair is solved together by **air-side face sampling + in-plane vertex averaging only** (no vertical averaging, no propagation, no FOV recompute): the same wall bakes bright on its lit side and dark on its dark side, and chunk-border gradients are seam-free because the sampler reads the whole grid. The dirty-chunk radius equals the smoothing radius, so a partial re-bake is mesh-equal to a full re-bake. Day/night is a keyframe curve; weather is a 7-state `WeatherKind` table (fog/precip parameters, O(1)). Wiring contract in `docs/phase7-lighting.md` §12. Ran as a subagent in parallel with Phases 4/6 under disjoint file ownership.
>
> **Test-gate results (Part II §10), parent-verified by re-running `cargo test -p lighting` (14/14 — 9 gate + 5 proto-adapter):** 7.1 ✅ (monotonicity), 7.2 ✅ (MEMORIZED presentation + BLANK exclusion), 7.3 ✅ (no leak), 7.4 ✅ (cross-chunk smoothing), 7.5 ✅ (partial == full re-bake), 7.6 ✅ (day/night curve), 7.7 ✅ (weather table), 7.8 ✅ (CPU-rendered PNG scenes in `crates/lighting/output/` — the isometric night house reads clearly; the top-down/flashlight dumps are unexposed linear data, dark by design), 7.9 ✅ ≈45 ms for a full 132×132×3 bake in release on this VM (VM number; target hardware will differ).
>
> **Findings / deviations:** (1) Two bakers exist (per-face and per-vertex) — only the per-vertex one is leak-proof under the air-side rule, and it is the one the wiring contract names. (2) The day/night curve is tempered when baking (`0.4 + 0.6×curve`) because `lit_level` is already the game's time-aware truth — applying the full curve would double-count night. (3) `WeatherKind` is defined in-crate: the Phase 1 schema has no weather field yet; when the protocol gains one, the table keys off it unchanged. (4) The `LightUpdate` wire form is documented as one 12×12 submap block, matching the delta export's granularity.
>
> **Gate verdict: CONDITIONAL PASS.** All gates pass at the crate level, parent-verified. **Condition:** the bake is not yet wired into the engine's mesh/RT paths (contract written, §12 of the doc) and in-game light values arrive only when the fork runs (0.1/0.2) — Phase 2h seats this bake and the Phase 4/6 passes in the frame loop.

---

### Phase 8: Hard cases (3-4+ weeks) — ✅ GATE PASSED (conditional) 2026-11-15

> **Dependencies:** ⛔ hard — Phases 4, 5, 6, 7 gates for the release gate; individual hard cases need only their own phase (vehicles/animations: Phase 3). **Parallel with:** its own sub-tasks are parallel with each other.

- **Vehicles**: orientation is a **continuous angle** (`vehicle::face` is a `tileray`; `face.dir()` returns `units::angle`). The "15-degree steps" in v1 came from a real but narrower fact: `vehicle::immediate_path()` clamps a *pathing* angle "to multiples of 15". Do not quantize exported facing to 15° — export float degrees, let the engine smooth/interpolate, and build the voxel model from the game-side absolute part positions (Phase 3), so pivot/precalc logic is never duplicated. Smooth turning animation interpolates between exported facings.
- **Doors, windows, multitile terrain**: open/closed state arrives as a *different terrain/furniture id* after the game's own transform (`open`/`close` ids) — the engine needs no door logic, just the delta. Bashed/broken states likewise. Connecting walls use the multitile set — all **eight** keys: `center, corner, edge, t_connection, end_piece, unconnected, open, broken` (`multitile_keys` in `cata_tiles.cpp`; v1 listed six and missed `open`/`broken`).
- **Open air, roofs and pits**: terrain `t_open_air` has a `roof` field naming the floor above; z-level meshing must treat open air as "no block, floor comes from below/roof rules", or every upper floor and rooftop breaks. Test explicitly with a two-storey house and a basement.
- **Animations and projectiles**: hook the game's animation dispatch (`src/animation.*` — it already selects curses vs tiles implementations per effect: bullets, explosions, thrown items) and forward effect events; the engine renders them through the **Phase 2b shader-effect system** (transient emitters: flashes, bursts, trails), never as sprite-frame animation.
- **Overmap**: keep the game's 2D overmap drawn in the UI texture; optionally a 3D terrain view later.
- **Large mods and tilesets**: validate against popular mods; add a debug report for ids with no sprite/material. Mods can also add *connect groups, field types and vparts* — all are in the TileDefs export, so no engine change should be needed; test with one content mod to prove it.
- **Performance**: chunk streaming, mesh rebuild budgets, instancing, culling, LOD for distant chunks.
- **Packaging**: Windows/Linux builds, bundled fork binary, asset generation step (`compose.py` run or pre-composed tileset from a release), installer or archive.

> #### Phase 8 start record (2026-11-15, per the maintenance rule)
> **What was actually built:** three new pure-CPU crates, isolated from the concurrently running 2g.4 (coverage) and 2h (engine frame loop) work — no engine/billboard/ui-share/tileset/voxelizer files touched. `crates/vehicles`: wire→model decoding with the fork's part-flag bits (open/broken/roof), wholesale snapshot/delta store with previous-state retention, and presentation interpolation (shortest-arc facing; part positions lerped between exports, exact at both endpoints); mock game-truth fixtures in the shape of `precalc_mounts`. `crates/transient`: `Effects` events → transient emitters parameterized in the Phase 2b space (render-class ids mirror `fx::RenderClass::id()`; the crate mirrors the constants rather than depending on wgpu, the fx↔voxelizer precedent), idempotent (seq, index) ingest, stale-seq drop, explicit monotonic clock, [start, start+duration) lifetime. `crates/hardcases`: `DefTable` transform classification (Open/Close/Bash/Unchanged/Other) + Delta application for 8.4, and the single-step open-air floor resolution for 8.4b (floor = below terrain's `roof`; no roof declared → no floor; open air over open air → no floor at that level). Design + full gate table: `docs/phase8-hardcases.md`.
>
> **Schema change (the one deviation from "start where the plan stands"):** gate 8.4b was **not implementable** from the Phase 3 export — TileDef carried no `ter_t::roof`. Added `roof_result: string` appended as TileDef's **last** field (vtable id 34; all existing ids unchanged, old readers ignore it — the displayed_field precedent). Fork side: `DefVal::roof_result`, exporter copy, real-adapter fill for terrain defs (`d.roof`, mirroring the `open` fill). Both checked-in C++ headers regenerated with flatc 24.3.25; the regen diff is exactly the new field. Proto samples keep the field absent, so Phase 1 goldens stay byte-identical. `Cargo.lock` deliberately left uncommitted: the working-tree lock also carries 2h's in-flight engine dependency edges (it regenerates on build; CI is not `--locked`).
>
> **Test-gate results (Part II §11), parent-run:** 8.1 ✅ model level (exact part positions + bit-exact facing at 0/7/15/23/45/90/179/270/359°; quantization counter-proof: snapping 7°→15° moves a part), 8.2 ✅ model level (scripted turning drive, endpoints exact at every step, 359°→7° midpoint 3°), 8.3 ✅ model level (broken/open/hp, part + vehicle removal, roof split 1 vs rest), 8.4 ✅ model level (protocol sample flow: house door (5,11) + sample Delta → Open, fg_ref 102→104; close/bash/other/unchanged + furniture parity), 8.4b ✅ model level (two-storey + basement fixture, no phantom/fabricated floors, cutaway independence, `roof_result` wire round-trip), 8.5 ✅ model level (projectile right tiles/times, batch expiry leaves the system empty, resend/stale-seq cannot respawn). Suites: vehicles 6/6, transient 4/4, hardcases 5/5, proto 16/16, fork harness ALL PASS + RUST-DECODE PASS, rebuilt cpp-roundtrip PASS; regression sample (mesher/lighting/fx/treebuild/tileset/voxelizer) green. **In-game halves of 8.1–8.5 remain ⛔ blocked on 0.1/0.2**, like every in-game gate.
>
> **Protocol truth recorded:** an `EffectEvent` carries endpoints + duration and **no arc/curve** — transient trajectories are linear in tile space; a thrown item's parabola is a presentation choice, not exported data (docs §transient).
>
> **Remaining Phase 8 scope (unchanged, scoped in the doc):** 8.6 overmap (a 2h frame-loop check by design), 8.7 mod smoke (needs the runnable fork), 8.8/8.9 soak + memory (need the live loop), 8.10 performance (several budgets are RX 9070 XT numbers; 2c.9 still open), 8.11/8.12 packaging + upgrade (Windows checks on Joey's PC), 8.13 crash reporting, 8.14 final playthrough; plus engine wiring of this slice's crates and the 2d `open`/`broken` subtile follow-up, both owned by 2h's frame surface. **Release gate not met — Phase 8 stays IN PROGRESS.**
>
> #### Phase 8 completion addendum (2026-11-15, later — supersedes the "remaining scope" paragraph above)
> On Joey's word ("You should be able to simulate dummy data the engine will send you to finish the phase"), the remaining gates were driven on dummy data the same day. **Built:** `vehicles::mesh` (render model: stamped tile set == mock truth at 23°, roof parts compose a shared tile's column or slab alone, broken parts darken, clear restores air — 2 more gate tests); `hardcases::report` + mod gate (mod TileDefs fixture — new connect group, floor/roof pair, door chain, vpart, monster, shader field — through the base rules; missing-material report exact: precisely the 2 unmaterialed defs, field/open-air as unreferenced); `crates/release` (workspace member; depends on proto/vehicles/transient/hardcases/ui-share, touches none of their files): 8.6 overmap sequence on the Phase 4 store (UiInput frame validates with no terrain rect, clicks route as UI, drags swallowed, MapInput frame restores world picks + camera drag), 8.7 cross-store mod drive, 8.12 stamp/cache + epoch resync, 8.13 panic-hook crash report (frontend version, protocol version, pinned fork commit `6a3e59e0…`, message + location — verified by catching a real panic), soak driver (8.8/8.9: 7,200 steps = 2 h at 1 turn/s through all three stores; max 2 transient alive, drains to 0; vehicle store ends with exactly the live set; per-step 6.0 → 17.4 µs (debug test build), no creep; RSS 3,340 → 3,520 warm → 3,648 KiB end, +128 KiB after warm-up), `perf_probe` bin (8.10, release build, this VM: vehicle delta apply 20 vehicles 11.37 µs, transient ingest 3.21 µs, classify 0.05 µs, floor_at 0.02 µs — vs the plan's ms-scale turn budgets); `tools/package_smoke.sh` (8.11 Linux half: package dir assembled, launched via `env -i` from an unrelated cwd — PACKAGE SMOKE PASS: handshake → TileDefs → snapshot → input round-trip → clean shutdown). Suites after slice 2 (parent-run): vehicles 8/8, transient 4/4, hardcases 7/7, release 5/5, proto 16/16, mesher green. Two soak-driver bugs found and fixed in-run (advance-before-ingest ordering against the transient clock; a wrong end-state expectation — the traffic vehicle is inside its present window at step 7199).
>
> **What is actually blocked — the complete list** (docs §"What is actually blocked"): (1) 0.1/0.2 — compiling the real fork (~1,500 TU) + `cata_test` exceeds this VM's compute (1.5 CPU / 6 GB); not data, cannot be simulated; every in-game half inherits it. (2) 8.11's Windows half — no Windows OS here. (3) Frame-time budgets + 2c.9 — RX 9070 XT numbers. (4) 8.14's manual hour per platform (scripted analogue ✅). Nothing else in §11 remains unproven. **Phase 8: GATE PASSED (conditional)** on those four conditions — the release gate's required set (8.1, 8.4b, 8.7, 8.8, 8.10, 8.11) passes at the levels above. Engine wiring of the Phase 8 crates + the 2d subtile follow-up remain 2h's, unchanged.

> **Superseding note (2026-11-15, later):** the last sentence above is now historical — the engine wiring of this phase's crates and the 2d `open`/`broken` subtile follow-up were both completed the same day; see the follow-up addendum under the Phase 2h completion record. Phase 8's blocked list is unchanged (the four conditions above are the whole of it).

---

## 6. IPC design

### Transport
- **Messages**: local stream socket (Unix domain socket on Linux/macOS, named pipe on Windows) via Rust `interprocess`; length-prefixed FlatBuffers frames.
- **UI texture**: shared memory region (Rust `shared_memory` / C++ `shm_open` or `CreateFileMapping`) with header `{ magic, width, height, stride, seq, dirty_rect }` and two buffers.
- **Lifecycle**: engine spawns the game with `--frontend=ipc --ipc-path=<path>`; if either side exits, the other shuts down cleanly. Heartbeat every second.

### Message types (sketch)

| Direction | Message | Purpose |
|---|---|---|
| G->E | `Hello` | protocol version, game version, tileset id, tile pixel size, z-range |
| E->G | `HelloAck` | engine version, screen size, DPI, capabilities |
| G->E | `TileDefs` | id tables for terrain, furniture, traps, fields, monsters, items, vehicle parts, plus the connect-group table |
| G->E | `Snapshot` | full view: origin, extents, per-submap tile arrays + memory layer |
| G->E | `Delta` | changed tiles, entity add/move/remove, vehicle changes |
| G->E | `LightUpdate` | per-tile `lit_level` and visibility for the view (fold into Delta when small) |
| G->E | `Effects` | animations, projectiles, sound markers |
| G->E | `UiFrame` | shared buffer index, dirty rect, and whether the game is in a menu/UI-input mode vs map-input mode |
| E->G | `Input` | key, text, mouse, wheel, modifiers, focus |
| E->G | `Resize` | new size and DPI |
| E->G | `PickResult` | tile coordinates under cursor click |
| both | `Ping/Pong`, `Shutdown` | liveness |

### FlatBuffers schema sketch

```fbs
namespace cdda.ipc;

struct Vec3i { x:int; y:int; z:int; }

table TileDef {
  id:uint32;
  string_id:string;
  name:string;
  kind:TileKind;          // terrain, furniture, trap, field, monster, item, vpart
  flags:[string];         // string flags as authored
  hot_flags:uint64;       // resolved ter_furn_flag bitset for engine-fast checks
  connect_groups:[uint16]; // indices into the Hello/TileDefs connect-group table
  connects_to_groups:[uint16];
  open_result:string;     // id after open transform, "" if none
  close_result:string;
  bash_result:string;     // terrain/furniture id after a successful bash
  looks_like:string;
  fg_ref:int32;           // index into composed tile_config tiles; -1 = none
  bg_ref:int32;           // floors/background sprite matters for voxelizer (§8)
}

table FieldOnTile {       // a tile can hold SEVERAL fields at once (smoke + fire + ...)
  type:uint32;            // field type def id
  intensity:ubyte;        // field_entry intensity
}

table TileDefs { defs:[TileDef]; connect_groups:[string]; protocol_version:uint32; }

table SubmapBlock {
  origin:Vec3i;           // absolute tile coordinates of the block's corner
  terrain:[uint32];       // 12*12 entries per z layer, row-major
  furniture:[uint32];
  trap:[uint32];
  fields:[FieldOnTile];   // sparse: only tiles with fields, with tile index — NOT one id per tile
  field_index:[int32];    // per tile: offset into fields, -1 = none (or use displayed-only fast path below)
  displayed_field:[uint32]; // per tile: field_type the game itself would draw + intensity packed
  top_item:[uint32];      // uppermost item id per tile (game has get_uppermost_item())
  item_count:[uint16];    // stack size per tile; >1 means "pile", engine need not enumerate
  lit:[ubyte];            // lit_level enum per tile (DARK..BLANK), NOT a 0-255 light value
  seen:[ubyte];           // quantized seen/visibility float, 0 = never seen
}

table Entity {
  id:uint64;
  sprite_id:string;
  pos:Vec3i;
  facing:ubyte;           // FacingDirection: 0 none, 1 left, 2 right — flip only
  overlays:[string];      // in game order (get_overlay_ids); do not re-sort engine-side
  hp_frac:ubyte;
  flags:uint32;           // friendly, hostile, sleeping, etc.
}

table VehiclePartState {
  vpart:uint32;
  abs_pos:Vec3i;          // computed game-side from pre-rotated mounts
  mount:Vec3i;
  hp_frac:ubyte;
  flags:uint32;           // open, broken, roof, etc.
}
table Vehicle { id:uint64; face_deg:float; parts:[VehiclePartState]; }

table Snapshot { view_origin:Vec3i; blocks:[SubmapBlock]; entities:[Entity]; vehicles:[Vehicle]; seq:uint64; epoch:uint64; }
table Delta    { seq:uint64; epoch:uint64; tile_changes:[TileChange]; entity_updates:[Entity]; entity_removed:[uint64]; vehicle_updates:[Vehicle]; }
```

(Exact layout to be finalized in Phase 1; prefer per-block arrays over per-tile records for speed. Field/item representation above is the minimum that doesn't lose gameplay-visible information — smoke you can't see through and a fire's intensity are not optional detail.)

### Determinism and sync
- Sync is by **fork-side monotonic `seq` + `epoch`**, not by the game's calendar turn: non-player actors, fields and weather change state outside player turns, and calendar time is a game-time value, not a message counter. `epoch` increments on every fresh `Snapshot`; a `Delta` names the epoch and the seq it applies to.
- A `Delta` is always applicable to the last acknowledged snapshot; if the engine detects a gap (wrong epoch, skipped seq), it requests a fresh `Snapshot`.
- The game never waits on the engine except for input.

---

## 7. Engine design (Rust)

### Crate choices
- Rendering: `wgpu`, `winit`
- Math: `glam`
- Image: `image`, `ktx2` (optional), `rectangle-pack` or custom atlas
- Serialization: `flatbuffers`
- IPC: `interprocess`, `shared_memory`
- Concurrency: `rayon` for meshing, channels for IPC thread -> render thread
- Logging/diagnostics: `tracing`, `tracy-client` (optional)
- Tests: `insta` for snapshot tests

### Threads
1. **IPC thread**: reads socket, decodes messages, pushes to a queue.
2. **Mesh workers** (rayon): build chunk meshes off the render thread.
3. **Render thread**: applies updates, uploads buffers, draws.

### World representation
- World stored as chunks aligned to game submaps (12x12 tiles, per z-level; a chunk column groups all z in the cutaway window — the full column is 21 levels, the *stored* column should be sized to the cutaway, not hard-coded to 21).
- A block per tile: `{ terrain, furniture, flags, lit_level }` plus optional sub-voxel model reference for furniture.
- Dirty flags per chunk; neighbor chunks marked dirty on boundary changes.

### Meshing
- Greedy meshing per chunk for opaque full blocks, producing per-face texture-array layers.
- Separate passes for: opaque, cutout (alpha-tested, e.g. fences, foliage), transparent (glass, water, fields).
- Walls choose face textures from multitile variants (all eight slots, § Phase 8) based on neighbor connectivity using exported connect-group data.
- Use a **texture array** (not a single atlas) for block faces to avoid mipmap bleeding.

### Scale and proportion
- 1 tile = 1x1 block footprint; 1 z-level = 3 blocks tall so walls read as walls and billboards (about 2 blocks tall) fit through doorways. Parameterized in one config struct so it can be tuned.
- Floors/ceilings are thin slabs or full blocks per design choice; decide visually in Phase 2. Whichever is chosen, `t_open_air` + `roof` handling (Phase 8) is part of the decision, not an afterthought.

### Camera and view
- Perspective camera with follow target, orbit yaw, pitch in [20, 85] degrees, zoom.
- Z-cutaway modes: hide all above player z; fade roof tiles; or show only the current floor +1.
- Optional classic top-down/isometric preset to compare against the original look.

### UI compositing
- Full-screen quad with the shared UI texture; alpha blend; integer scaling option.
- Input focus: when the game reports a menu is open (UiFrame mode flag), the 3D camera ignores mouse drags; otherwise camera controls are active and game keys pass through.

### Picking
- Ray from cursor through the world; hit-test chunks (DDA voxel traversal) and billboards; send tile coordinate on click.

### Headless capture (offscreen rendering)
- A `--headless-capture` engine mode (Phase 2 deliverable, §5 Phase 2) renders the same chunk meshes through the same pipelines to an offscreen texture with no window/surface, reads the texture back, and writes a PNG — the direct analogue of the Godot projects' viewport `get_image()` readback, minus the X server. Windowed and headless paths must share mesh, camera, and pipeline code so a capture is evidence about the real renderer, not a parallel one.
- Adapter reality: `wgpu` needs a Vulkan/Metal/DX adapter — a software one (lavapipe/WARP) counts for small scenes; a machine with no adapter at all (the recon VM as of 2026-11-15) can only run the data-level gates, and captures move to a machine with an adapter (developer's PC).

---

### Ray-traced renderer (Phase 2c, added 2026-11-15)
- The primary renderer is a **compute ray tracer over a sparse voxel 64-tree** ([Sparse-Voxel-64Tree](https://docs.rs/Sparse-Voxel-64Tree/latest/Sparse_Voxel_64Tree/) crate: slotmap tree on CPU, flattened `GpuNode64` buffer + wgpu bind group on GPU; our WGSL does the traversal). The world model stays chunk-based (12×12 columns); a builder converts world → tree at block resolution, and deltas rebuild/re-upload the tree (per-chunk-column trees via `TreeMemoryManager` if full rebuilds measure too slow).
- The crate's GPU node carries only an RGB color, so materials ride in a **parallel side-table buffer** indexed by the crate's deterministic flatten order: `{material_id, lit_level, flags}` per node; the hit shader samples the Phase 5 texture array by material + hit normal.
- Shading: textured albedo × game `lit_level` (visibility truth) + sun light with shadow rays + occupancy AO. Billboards, field effects and the UI composite *after* the traced color+depth, in that order.
- The greedy mesher/raster path remains as a `--renderer raster` fallback until the Phase 2c gate records the default. Meshing costs in §11 apply to the fallback; the RT budgets are the 2c rows there.

### Field effects and HD-2D grade (Phase 2b, added 2026-11-15)
- Fields are **shader volumes, never sprite quads**: per-tile instanced volumes with animated-noise fragment/compute shading, density from the game's field intensity, palettes and emissive/density parameters extracted from the sprites by the voxelizer (`effects.json`). The game's `displayed_field` weights which effect dominates a tile.
- The HD-2D finish is a fullscreen post stack over color+depth — bloom, tilt-shift DoF, vignette, warm grade — shared unchanged between the raster and ray-traced renderers.

## 8. Texture and sprite pipeline (voxelizer)

### Inputs
- Tileset directory from CDDA-Tilesets (e.g. `gfx/UltimateCataclysm`), in its **decomposed** form: per-sprite PNGs + per-sprite tile-entry JSON under `pngs_*/` folders, plus `tile_info.json` and `tileset.txt`.
- The **composed** form, produced by running CDDA's `tools/gfx_tools/compose.py` on that directory (requires Python + `pyvips`/libvips): a single `tiles.png` sheet + `tile_config.json`. In the composed config, each entry has an `id` (string, or list of ids sharing a sprite), `fg`/`bg` as sprite *indices* (each either a single index or a list of weighted variants), optional `multitile: true` with `additional_tiles` for the eight multitile slots, `rotates`, and season variants are separate entries whose ids carry `_season_spring/summer/autumn/winter` suffixes. Sprite resolution in-game also follows a `looks_like` fallback chain — the voxelizer should resolve ids the same way (exact → variant → season → `looks_like`) or it will report false "missing" sprites.
- TileDefs exported by the game (for flags and connect groups).
- Optional override file `overrides.toml` (hand-tuned entries).

### Sprite geometry — not uniform 32×32
UltiCa's base tile size is 32×32, but `tile_info.json` defines many sheets with different sprite sizes **and offsets**, all of which the voxelizer and billboard code must carry through per sprite:
`small` 20×20 (offset +6x), `tall` 32×64 (−32y), `human_body` 32×36 (−8y), `human_body_plus` 32×48 (−16y), `centered` 64×64 (−16,−16), `large` 64×64 (−16,−32), `huge` 64×96 (−16,−64), `giant` 96×96 (−32,−64), plus filler variants of several of these. A sprite larger than a tile is anchored by its offset, not centered by default. Block faces use only tile-sized sprites; tall/large/giant sprites belong to the billboard/furniture-model path.

### Outputs
- `blocks` texture array: per material, top, side and bottom faces at tile resolution (32x32 for UltiCa base; configurable per tileset — other tilesets differ, e.g. RetroDays is 10×10).
- `materials.json`: terrain/furniture id -> material, faces, height in voxels, transparency, tint, sub-model reference.
- `sprites` atlas for billboards (characters, monsters, items) with UV table, **per-sprite width/height/offset**, and overlay layering metadata.
- `effects.json` (Phase 2b, added 2026-11-15): per field type, shader-effect parameters extracted from its sprite — base/emissive colors, density and noise scales, render class (`smoke`/`flame`/`gas`/`liquid`/`energy`/`generic`). Sprites parameterize effects; they are never rendered as field visuals.
- `ATTRIBUTION.md`.

### Face generation rules
1. **Top face**: the tile's own sprite — specifically its `fg` composited over its `bg` where both exist (many floor sprites are a transparent `fg` over a `bg` ground sprite; using `fg` alone produces holes).
2. **Side faces**:
   - Walls: use wall sprites as sides directly (they already depict a vertical-ish surface in UltiCa); fall back to sampling the sprite's most opaque rows.
   - Ground/floor: sample the bottom edge rows and stretch, darkened 25-40%, with subtle noise to avoid obvious stretching.
3. **Bottom face**: darkened copy of the side material; rarely visible.
4. **Multitile walls and connections**: map the game's multitile slots (`center, corner, t_connection, edge, end_piece, unconnected, open, broken`) to face variants selected at mesh time by neighbor mask and connect groups.
5. **Mipmaps**: generate with alpha-aware downsampling; clamp edges.
6. **Transparency**: alpha cutout flag when sprite has significant alpha holes; separate transparent flag for glass and water.

### Furniture and objects
- Default: furniture is a block-height-fraction extruded silhouette: take the sprite's alpha mask, extrude by `height_voxels` (heuristic by id/flags; e.g. tables 1, bookcases 3, beds 1, appliances 2). Sub-voxel resolution (sprite pixel = voxel column) is supported for these models.
- Top face uses the sprite; sides use darkened edge-sampled colors.
- `overrides.toml` can force a model: `{ id = "f_bookcase", height = 3, mode = "box" }`.

### Billboards (no voxelizing)
- Characters, monsters, NPCs, items: use original sprites unchanged, with their real sizes/offsets (above).
- Compose overlay layers in the engine at runtime and cache by signature.

### Fallbacks and QA
- Unknown id -> magenta/black checker material and a line in `missing.log`. Resolution must try `looks_like` before declaring an id missing (see Inputs).
- Generate a contact sheet (`contact.png`) of all materials for visual review.
- Visual regression: render a fixed test scene per tileset and compare with `insta` image snapshots.

---

## 9. Fork changes (C++), checklist

- [ ] New `--frontend=ipc` option and null/offscreen video path (per Phase 0 Spike B decision: **null `catacurses` backend**; hook call-sites in `src/do_turn.cpp`, `src/game.cpp`, `src/sdltiles.cpp` — `do_turn` lives in its own file since the upstream split found in Phase 0)
- [ ] `src/voxel_ipc/` module: server, framing, serializer
- [ ] Tile definitions exporter (terrain, furniture, trap, field, monster, item, vpart + connect-group table)
- [ ] Snapshot exporter (view radius, z-range, `lit_level`/seen, map-memory layer)
- [ ] Delta tracking (dirty-submap collection from existing invalidation hooks + per-submap diff)
- [ ] Entity exporter with overlay list (game order), facing (flip only)
- [ ] Vehicle exporter (absolute part positions, float facing)
- [ ] Offscreen UI render target and shared-memory writer (Phase 4 chosen route)
- [ ] Input injector feeding the existing input path (`input_context::get_raw_input`)
- [ ] Clean shutdown, crash handling, heartbeat
- [ ] Config flag to keep original tiles mode working (don't break upstream behavior)
- [ ] Rebase script and notes for syncing with upstream; pinned upstream commit recorded

Design principle: **additive hooks, no behavior changes** in the simulation.

---

## 10. Testing and CI

- **Schema tests**: round-trip serialization for both languages; golden binary files.
- **Replay tests**: scripted input into the fork on a seeded test world, compare exported tile-array hashes across runs (see Phase 3 exit criteria for why the hash scope is limited).
- **Mesher unit tests**: known chunk layouts -> expected face counts; greedy merge correctness.
- **Voxelizer tests**: tiny fixture tileset -> expected atlas and materials; include one multitile entry, one fg-over-bg entry, one non-32×32 sprite, and one `looks_like`-only id in the fixture — those are the four cases that break naive pipelines.
- **Visual snapshots**: headless wgpu render of fixed scenes (on a software adapter in CI).
- **Integration test**: launch engine + fork, create a world, walk N turns, quit; check no errors.
- CI: GitHub Actions matrix (Linux first, then Windows/macOS); cache Cargo and CDDA build objects (the fork build is slow).

---

## 11. Performance budgets

Snapshot size math, corrected: one z-level of the full bubble is 132×132 = 17,424 tiles. At ~6 bytes/tile of core arrays (terrain+furniture+trap+lit+seen, ids as uint16 where the def count allows) that's ≈100 KB per z-level — a full 21-level bubble snapshot is ≈2 MB *before* fields, items, entities and vehicles. Hence the §5/§6 rule: snapshots cover the view window and cutaway z-range, not the whole bubble, and full resends stay rare.

| Item | Target |
|---|---|
| Frame time | under 16 ms at 1080p with typical view |
| Visible area | view radius up to 60 tiles; stored world 132×132 tiles × cutaway z-range |
| Chunk remesh | under 2 ms per chunk, off the render thread |
| Delta apply | under 1 ms per turn |
| UI texture upload | only on dirty frames; under 1 ms (GPU readback on the game side is budgeted in Phase 4's spike, not here) |
| RT frame time (Phase 2c) | under 16 ms at 1440p on the RX 9070 XT (primary + shadow ray, bubble-scale scene); correctness gates run on lavapipe at small sizes, performance is never gated on llvmpipe |
| Tree rebuild + re-upload (Phase 2c) | full stored-world rebuild under 20 ms; per-turn delta rebuild under 2 ms if per-column trees are needed to reach it |
| Effect pass (Phase 2b) | field volumes + post stack under 3 ms at 1440p on the RX 9070 XT |
| Snapshot size | under 2 MB *for the view window*; full resend rare |
| Memory | under 1.5 GB total for engine |

---

## 12. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| CDDA not built to be frontend-swappable; map and UI drawing intertwined | Phase 3-4 slip | Phase 0 spikes (done at prototype level before committing); additive hooks; fallback to in-process integration |
| UI drawn as many windows, not one surface | Compositing looks wrong | Work at the backend's existing `display_buffer` present point (Route A) or the `catacurses` layer (Route B) — both verified to exist; never try to scrape individual windows |
| Delta tracking coarser than hoped (no per-tile hooks) | Phase 3 slip | Already the plan: dirty-submap diff; cap diff cost by submap count per send |
| Mouse-driven map interactions (look, target, examine) | Broken UX | Engine picking -> tile coordinates message; UiFrame mode flag separates UI clicks from map clicks; keep keyboard flow first |
| Sprite sizes/offsets vary widely across sheets | Billboards and furniture models misaligned | Per-sprite geometry from `tile_info.json` carried through the whole pipeline (§8); fixture tests |
| Extruded-sprite textures look inconsistent across thousands of ids | Art quality | Overrides file, per-category rules, contact sheets, iterate |
| Upstream churn in CDDA — the renderer/SDL backend is actively being refactored | Rebase pain | Isolate in new files; small hooks; pin a commit and rebase deliberately on a schedule, not continuously |
| Mods adding content without sprites | Placeholder art | Fallback materials by flags; `looks_like` resolution before declaring missing; missing report |
| Transparent sorting (windows, fields, billboards) | Visual glitches | Separate passes, dithered alpha, tuned depth rules |
| Vehicle rotation and multi-tile parts | Complexity | Defer to Phase 8; game-side absolute part positions from day one so no rotation math is duplicated |
| Cross-platform shared memory/input quirks | Delays | Linux first; abstract platform layer; test early on Windows |
| Licensing mistakes | Legal/attribution | Phase 0 decision doc; auto-generated attribution including third-party notices from `LICENSE.txt` |
| Sparse-Voxel-64Tree is a single-release (0.1.0) single-author crate | Phase 2c stalls or breaks on a wgpu bump | Pin `=0.1.0` (its wgpu `^24.0.1` matches our 24.0.5 today); vendor if needed; the 32-byte GPU node layout is documented, so an in-house flatten/traversal fallback is feasible — the design keeps the crate behind the tree-builder boundary |
| Ray tracing underperforms on non-RT / software adapters | Phase 2c frame budget missed off the target GPU | Budgets are gated on the RX 9070 XT only; lavapipe runs correctness at small sizes; the raster path stays as `--renderer raster` fallback until the 2c gate |
| Shader effects diverge from game readability (smoke that hides too much/little) | Phase 2b art risk | Density is driven by the game's own intensity + `displayed_field`; per-type overrides; capture gates at intensities 1–3 |

---

## 13. Milestones and rough timeline

Cumulative times are **critical-path** times (see §5 "Phase dependencies and parallelism"): they assume Phases 2, 5 and 6 run in parallel with the 0 → 1 → 3 → 4 path, not after it. Each milestone row must be dated in the Progress log when it is reached, per the maintenance rule at the top of this plan.

| Milestone | Cumulative time | Outcome |
|---|---|---|
| M0 Recon confirmed + spikes | ~1 week | Hook points proven, UI and headless routes chosen |
| M1 Protocol + stub | ~2 weeks | Rust client talks to fake server |
| M2 Engine MVP | ~4 weeks | Navigable colored-voxel scene with z-slicing |
| M2b Effects / HD-2D | ~6 weeks | Shader field effects (smoke/fire/gas) + HD-2D post stack over the engine |
| M2c Ray-traced renderer | ~8 weeks | World rendered by compute ray tracing over the 64-tree, textured + shadowed; default-renderer decision recorded |
| M3 Real game export | ~8 weeks | Live snapshots/deltas from the fork |
| M4 UI compositing | ~10 weeks | **Playable**: real menus over flat voxels |
| M5 Textures | ~12 weeks | Textured world from UltiCa |
| M6 Billboards | ~14 weeks | Characters/monsters as sprites |
| M7 Lighting/FOV | ~16 weeks | Atmosphere matches game visibility |
| M8 Polish and hard cases | ~20+ weeks | Vehicles, animations, packaging |

---

## 14. Open questions

1. Final world scale and floor/ceiling thickness (decide visually in Phase 2).
2. Should the original 2D tiles view remain selectable at runtime (toggle between frontends)?
3. How much of mouse interaction is required for v1?
4. Which tileset(s) to ship first: UltiCa only, or UltiCa plus a chibi character set (Altica / Chibi_Ultica)? — **Narrowed in Phase 0 (2026-11-15): UltiCa only for v1**; chibi character sets deferred, not decided against.
5. Target platforms for v1 (Linux only, or Windows too)? — **Narrowed in Phase 0 (2026-11-15): Linux first**; Windows packaging lands in Phase 8.
6. Should engine and fork be distributed together or as separate downloads?
7. Colored light: quantize to intensity+tint for v1, or carry float RGB from the start? (Protocol field is sized for it either way — §7.)
8. Map-memory layer fidelity: does the engine need furniture-level memory, or terrain-only like the game's own muted memory rendering?

---

## 15. First-week task list

1. Clone the fork, build the curses (text) version, run it. (No tiles/SDL3 build — not needed; see test 0.1 as updated 2026-11-15.)
2. Turn Appendix A into `docs/recon.md`; add the pinned upstream commit.
3. Spike A: suppress `draw_ter()`, capture `display_buffer` at present, display it in a standalone window.
4. Spike B: decide the headless route from the spike results.
5. Create the Rust workspace and `proto` crate; draft `cdda_ipc.fbs` per §6.
6. Build the stub server and the Rust client handshake.
7. Run `tools/gfx_tools/compose.py` on `gfx/UltimateCataclysm` from the Tilesets repo; inspect the composed `tile_config.json`, one multitile wall entry (all eight slots), and one fg-over-bg floor entry.
8. Make the engine draw a single textured cube with a camera.

---

## Appendix A: Verification record (2026-11-15)

Checked against CDDA `master` @ `6a3e59e0` and CDDA-Tilesets `master` @ `5ee942f9`, by reading the named files/functions directly. Verdicts: **Verified** = v1 was right · **Corrected** = right idea, wrong detail · **Refuted** = v1 was wrong and the design changed.

| # | v1 claim | Verdict | Evidence |
|---|---|---|---|
| 1 | Recon files: `src/cata_tiles.cpp`, `sdltiles.cpp`, `map.h`, `submap.h`, `mapdata.h`, `lightmap.cpp`, `vehicle*.cpp`, `output.*`, `main.cpp`, `game.cpp` | Verified | All exist on master (HTTP 200 on raw source check), as do `vehicle_part.cpp`, `animation.*`, `panels.*`, `widget.cpp` |
| 2 | `src/ui.*` for the UI path | **Refuted** | `src/ui.cpp`/`src/ui.h` return 404. The UI stack is `src/ui_manager.*` (`ui_adaptor`); panels are `src/panels.*` + `src/widget.*` |
| 3 | `src/input.*`: `input_context` and event handling | Corrected | `input_manager`/`input_event` are in `src/input.h`, but `input_context` is its own file: `src/input_context.h` (with `get_raw_input()`, `handle_input()`, `registered_actions`) |
| 4 | "`catacurse` wrappers" | Corrected | Spelled `catacurses`: `src/catacurses.h`, implementation in `src/cursesport.cpp/.h` (`cata_cursesport::WINDOW`, `catacurses::window`) |
| 5 | Chunk size 12×12 to match submaps; snapshot 60×60×10 | Corrected | `src/map_scale_constants.h`: `SEEX=12`, `SEEY=SEEX` ✓ — but `MAPSIZE=11` (bubble 132×132), `MAX_VIEW_DISTANCE = SEEX*HALF_MAPSIZE = 60` is a *radius*, `OVERMAP_DEPTH=10`, `OVERMAP_HEIGHT=10`, `OVERMAP_LAYERS=21`, `fov_3d_z_range=10` (`game_constants.h`) |
| 6 | Hook existing "map changed" invalidation for deltas, else diff | Corrected (diff is primary) | `src/map.cpp/.h`: `invalidate_map_cache(z)`, `set_seen_cache_dirty(p|z)`, `set_transparency_cache_dirty(p|z)` exist, but granularity is whole-z-level / per-submap dirty bit in the level cache. No per-tile change list exists |
| 7 | Per-tile snapshot: single field id, top item id, light level, visibility bits | **Refuted** (fields/items); Corrected (light/vis) | `src/submap.h`: `fld` is `field` = `std::map<field_type_id, field_entry{type,intensity,age,causer}>` (`src/field.h`) — multiple fields per tile; `itm` is `cata::colony<item>` per tile (a stack; `get_uppermost_item()` exists, `MAX_ITEM_IN_SQUARE=4096`). Light/visibility product is `lit_level` enum (`src/lightmap.h`: DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT, MEMORIZED, BLANK) from `visibility_cache`, plus float `seen_cache`; colored light exists as `light_color_rgb` float RGB |
| 8 | UI may be drawn into fixed regions incl. the map area | Corrected | `game::draw(ui_adaptor&)` (`src/game.cpp`): `werase(w_terrain)`, `draw_ter()`, callbacks, `draw_panels(true)` — panels are transient windows from `panel_manager` layouts (message log = height −2 panel; sidebar side is an option). Menus are separate `ui_adaptor`s (`src/ui_manager.h`). **And** `src/sdltiles.cpp` composites everything into one `SDL_Texture display_buffer`, presented via `RenderCopy`, with `window_to_display_buffer_coords` conversion and a `display_buffer_draw_scope` RAII bind — the capture point for Route A |
| 9 | Tiles backend is SDL (software or render-target) | Corrected | `CMakeLists.txt`: "SDL2 support was removed; SDL3 is the only tiles backend", `option(TILES ... OFF)`, `option(CURSES ... ON)`. Makefile remains the primary build |
| 10 | Entity overlays: match the game's draw order | Verified, and now exact | `cata_tiles::draw()` `drawing_layers[11]`: terrain, furniture, graffiti, trap, part_con, field_or_item, vpart_no_roof, vpart_roof, critter_at, zone_mark, zombie_revival_indicators. Creature-internal: base sprite, then `Character::get_overlay_ids()` / `monster::get_overlay_ids()` in order (`src/cata_tiles.cpp` `draw_entity_with_overlays`), mutation ordering via `overlay_ordering.h` / `tileset_mutation_overlay_ordering` |
| 11 | Creature facing: directional variants if the tileset has them | **Refuted** | `src/creature.h`: `enum class FacingDirection { NONE=0, LEFT=1, RIGHT=2 }`, commented "for sdl horizontal flip". Creatures flip; they do not rotate |
| 12 | Vehicles: orientation in 15-degree steps | Corrected | `src/vehicle.h`: `tileray face; units::angle turn_dir;` — `face.dir()` is a continuous `units::angle`. The 15° figure is real but narrower: `vehicle::immediate_path()` in `src/vehicle.cpp` — "clamp to multiples of 15" — for pathing. Mount rotation is precomputed game-side (`precalc_mounts`, `pivot_rotation`) |
| 13 | Multitile slots: center, corner, t_connection, edge, end_piece, unconnected | Corrected | `multitile_keys` in `src/cata_tiles.cpp` has **eight**: center, corner, edge, t_connection, end_piece, unconnected, **open, broken** |
| 14 | `connects_to` / `connects_to_group` as per-def string data | Corrected | `map_data_common_t` (`src/mapdata.h`): `std::bitset<NUM_TERCONN> connect_groups / connect_to_groups / rotate_to_groups`, set from JSON group names via the `connect_group` registry (`connect_group::load`, `group_flags`, `connects_to_flags`). Flags likewise dual: string set + `enum_bitset<ter_furn_flag>` |
| 15 | UltiCa sprites are 32×32; faces at tile resolution | Corrected | Tilesets `gfx/UltimateCataclysm/tile_info.json`: base 32×32, plus sheets at 20×20, 32×36, 32×48, 32×64, 64×64, 64×96, 96×96 with per-sheet `sprite_offset_x/y` (e.g. tall −32y, giant −32/−64). `tileset.txt`: `NAME: UltimateCataclysm`, `VIEW: UltiCa`, `JSON: tile_config.json`, `TILESET: tiles.png` |
| 16 | Tileset names: UltiCa default; "AltiCa, Chibi UltiCa and MSX+" | Corrected | Tilesets README + `gfx/` listing: folders `UltimateCataclysm` (UltiCa, "the default tileset"), `Altica` (AltiCa), `Chibi_Ultica`, `MShockXotto+` ("often abbreviated MSX" — not MSX+). Isometric variants also exist (`Ultica_iso`, `IsoMSX+`) |
| 17 | Mushroom Dream: only a stub is CC BY-SA, full version restrictive | Verified | Tilesets README, verbatim in substance: "We host here a small stub of the tileset that was initially released under cc by sa, a more complete version released under a more restrictive license is available on Albert's Patreon" (`gfx/Mushroom-Dream`) |
| 18 | CDDA + Tilesets are CC BY-SA 3.0 per READMEs | Verified, with exceptions | `LICENSE.txt` in both repos: CC BY-SA 3.0 Unported, followed by third-party notices (fonts, Catch/Boost, PLF/zlib, libbacktrace/BSD, …) that must travel with attribution |
| 19 | `tools/gfx_tools/compose.py` and `tile_config.json` format | Verified, with a caveat | `compose.py` exists in CDDA; it converts per-sprite tile-entry JSON (sprite *names* in `fg`/`bg`, weighted variants, `additional_tiles`) into composed `tile_config.json` (sprite *indices*), per its own conversion code. Caveat: composed UltiCa `tile_config.json` is **not in the CDDA repo** (404 on master) — it's generated at tileset-build/release time, so the pipeline must run compose or consume a release |
| 20 | Sprite resolution is by id | Corrected | `cata_tiles` resolves exact id → variant (`_var_`) → season suffix (`_season_spring/summer/autumn/winter`, `find_tile_type_by_season`) → `looks_like` chain (`find_tile_looks_like`, per-category, recursive with a jump limit). Any exporter/voxelizer that skips this chain will misreport coverage |

### Not verified / still open
- Whether an SDL3 "dummy" video driver yields a usable renderer for a headless tiles build — deliberately left to Phase 0 Spike B; do not plan on it.
- Exact cost of `display_buffer` readback on target hardware — Spike A measures it.
- Whether the fork should target `master` or a stable release branch — a decision for Joey/the project, recorded in Phase 0; the constants and file layout above are from `master` and may differ on a release branch.


---

# Part II — Test Plan

# CDDA Voxel Frontend: Test Plan — v2 (verified)

> **Maintenance:** the plan's maintenance rule (top of Part I) applies here too — record each phase's gate results in this part and the Progress log when the phase completes, before starting the next phase. Phase dependencies for scheduling are in plan §5 and repeated under each phase heading below.

Part II of this document (the plan is Part I — there is no separate test-plan file). Each phase has a **gate**: a set of tests that must pass before the phase is considered done. Later phases also re-run every earlier gate as a regression suite.

> **What changed in v2:** every **(verify)** item has been checked against the source (same pins as the plan: CDDA `master` @ `6a3e59e0`, Tilesets `master` @ `5ee942f9`, 2026-11-15), and the tests that depended on wrong assumptions have been rewritten. Summary of the corrections — details in Appendix A of the plan and in the rows below:
> - **Catch2 harness: verified.** CDDA's tests are Catch2 (`tests/test_main.cpp` runs a `Catch::Session`, Catch is vendored at `tests/catch/catch.hpp`, binary built via `tests/CMakeLists.txt`). Bonus fact the determinism tests rely on: the test listener **seeds the game RNG from the Catch seed at the start of every section**, so fork-side tests get reproducible RNG for free.
> - **Vehicle orientation steps: refuted.** There are not 24 orientations. `vehicle::face` is a continuous angle; test 8.1 now samples arbitrary angles and compares part positions against the game's own `abs_part_pos()` / `bub_part_pos()`.
> - **Creature facing up/down: refuted.** `FacingDirection` is `NONE`/`LEFT`/`RIGHT` only; test 6.6 now tests mirroring only.
> - **Stale-delta test rewritten.** The protocol syncs by `seq` + `epoch`, not game turn (plan §6); test 3.12 tests seq/epoch semantics.
> - **Snapshot comparison APIs are now named and exact:** `map::ter()`, `map::furn()`, `map::tr_at()`, `map::field_at()` (full field map, not one id), `map::i_at()` (full item stack), `map::has_graffiti_at()`, `map::apparent_light_at()` / `update_visibility_cache()` for `lit_level`.
> - **Scene sizes corrected** to the plan's real numbers (bubble 132×132, view radius 60, z −10…+10) in 2.13, 7.9 and the Phase 3 cost tests.
> - **New tests added** for the gaps v1 missed: map-memory layer (3.5b), map shift / bubble recentering (3.9b), field intensity changes in the delta invariant (3.9), open-air/roof handling (8.4b), season-variant and `looks_like` sprite resolution (5.5b, 5.10), and the fg-over-bg top-face rule (5.3).

---

## 1. Principles

1. **The game is the source of truth.** Most tests compare engine state or output against what the game itself reports — via its own query APIs (`map::ter/furn/tr_at/field_at/i_at`, `vehicle::abs_part_pos`, `Character::get_overlay_ids`), never against a re-implementation of the game's logic in the test.
2. **The core invariant:** applying every `Delta` to the last `Snapshot` must always equal a fresh `Snapshot`. This is tested continuously from Phase 3 onward.
3. **The fork must not change simulation.** A game run with the IPC frontend must produce the same game state as a run without it.
4. **Determinism first.** Seeded worlds plus scripted input make nearly everything replayable and diffable. Fork-side tests run under CDDA's Catch2 harness, which seeds the RNG per section; engine-side tests pin their own seeds the same way. Anything that is *known* non-deterministic even under a seed (timestamps, weather RNG streams not covered by the seed) is excluded from hashes explicitly, by field list — not by loosening the comparison.
5. **Automate by default.** Manual checks are limited to visual art review and are always backed by a generated artifact (contact sheet, screenshots).

---

## 2. Test layers and tooling

| Layer | Scope | Tooling |
|---|---|---|
| Unit | pure logic (mesher, camera, protocol, voxelizer rules) | `cargo test`, `proptest`, `insta` |
| Fuzz | decoders and framing | `cargo-fuzz` (libFuzzer) |
| Benchmark | meshing, delta apply, upload | `criterion` |
| Visual regression | headless renders of fixed scenes | `wgpu` software adapter (lavapipe on Linux, WARP on Windows), image diff with tolerance |
| Fork unit | exporters inside the game | CDDA's Catch2 harness — **verified**: `tests/`, `Catch::Session` in `tests/test_main.cpp`, vendored `tests/catch/catch.hpp`; new exporter tests are ordinary `TEST_CASE`s in a `tests/voxel_*.cpp` file, run with the normal `cata_test` binary |
| Cross-language | C++ <-> Rust compatibility | golden binary files plus a two-way round-trip test |
| Replay / integration | game + engine together, scripted | Python or Rust test driver that spawns both, injects input, asserts hashes |
| Soak | long runs, leaks | scripted play loop, RSS and handle sampling |
| Manual | art quality, feel | checklists with screenshots attached to the PR |

### Shared fixtures
- **Tiny worlds:** a handful of seeded, small worlds (starter shelter, a house with doors and windows, a road and field, a forest edge, a multi-story building **with a roof and a basement** — needed for open-air/roof tests, a vehicle on a road).
- **Fixture tileset:** a fake tileset for the voxelizer that must include the four cases that break naive pipelines: one multitile entry (all 8 slots), one fg-over-bg entry, one non-32×32 sprite with offsets, and one id resolvable only via `looks_like`. (v1's "20-sprite" count is fine as a size, but content requirements matter more than count.)
- **Golden files:** serialized snapshots, TileDefs, UI frames, atlas outputs. Stored in the repo with a documented update command (`UPDATE_GOLDEN=1`).
- **Input scripts:** plain text key/mouse sequences with expected checkpoints. Scripts must pin season/time-of-day where sprite or light output is compared (season-variant sprites and lighting are time-dependent).

### CI jobs
1. `rust-unit` (fmt, clippy, tests, proptest with fixed seed plus a nightly random seed)
2. `fuzz-smoke` (60 s per target on PRs, longer nightly)
3. `proto-compat` (C++ and Rust golden round-trip)
4. `fork-build-and-test` (cached, Linux first; runs the fork's Catch2 suite including the voxel `TEST_CASE`s)
5. `replay-integration` (engine + fork, headless)
6. `visual-regression` (software GPU)
7. `bench` (nightly; fails on more than 20% regression against stored baseline)
8. `soak` (nightly, 30 min; weekly, 2 h)

> **CI status (2026-11-15):** all eight jobs now exist. `.github/workflows/ci.yml` (per push/PR): `proto-compat` (standing since Phase 1), `rust-unit` (workspace tests; the fmt/clippy leg is **not** gated — the tree carries pre-existing rustfmt drift in 49 files and a formatting pass is its own change), `fork-harness` (the Phase 3 mock-world gates 3.1–3.22 via `tools/fork-harness/run.sh`), `visual-regression` (golden/frame/wiring gates on lavapipe; real-UltiCa halves loud-skip, the atlas is not in git), `replay-integration` (harness form: `tools/package_smoke.sh`; the game+engine form rides the fork build), `fuzz-smoke` (cargo-fuzz targets `decode_message` + `decode_roundtrip` in `crates/proto/fuzz`, 60 s each, seeded from the golden frames — gate 1.5's full form; targets verified locally in standalone mode over the goldens + 400 mutants, no crash). `.github/workflows/ci-nightly.yml` (weekly + dispatch): `bench` (mesh_bench), `soak` (gate 8.8 harness form), `fuzz-long` (600 s per target), `fork-build-and-test` (dispatch/weekly; mirrors `tools/pc-build` in Linux form — pinned tarball, baseline build + full `cata_test` = gates 0.1/0.2, fork staging, `cata_test "[voxel]"`, gate 3.19 diff; **its first run validates the job itself**, like the PC script's first run). Jobs are stood up, not yet observed green on GitHub — nothing here clears 0.1/0.2 until a run passes.

### Flaky-test policy
A test that fails intermittently is quarantined within 48 hours with a tracking issue. Gate tests may not be quarantined. A test that turns out to depend on unseeded game RNG is not "flaky" — it is miswritten; fix the seed or the hash field list (Principle 4) instead of quarantining.

---

## 3. Phase 0: Recon and decisions

> **Phase dependencies:** none — first phase. Its gate blocks Phases 1, 2, 3 and 5 (see plan §5).

Goal: prove the hook points exist before building on them. (Most source questions are already answered in the plan's Appendix A; these spikes prove them *in a running build*.)

| # | Test (spike) | Method | Pass criteria |
|---|---|---|---|
| 0.1 | Fork builds (curses/text build only) | CI build of the curses build (`CURSES=ON`, `TILES=OFF`) | the binary launches to the main menu. **The tiles/SDL3 build is NOT required and is not part of this test** (updated 2026-11-15): the fork ships as the text build + null `catacurses` backend and the Rust engine does all graphics, so SDL3 — the original game's graphics library, linked only by the tiles build — has no role in this project. A tiles build may still be produced optionally as an unmodified-upstream visual baseline, but no gate depends on it |
| 0.2 | Baseline upstream tests | run CDDA's Catch2 suite (`cata_test`) on the unmodified fork branch point | pass rate recorded; failures documented as pre-existing. This baseline is the comparison for 3.19 |
| 0.3 | Send-boundary probe | temporary log line at each candidate send boundary (`game::do_turn()` exit, redraw points) during a scripted walk, a multi-turn activity (crafting), and idle waiting | **Not** "exactly one per turn" (v1 was wrong: free actions, multi-turn activities and non-player changes break that). Pass = every scripted *state change* is followed by a boundary before the next input is consumed, and no boundary storm (≤ a small bound per turn) |
| 0.4 | Visible-tile probe | log count and checksum of exported terrain per z for a seeded world + script | values stable across two runs with the same seed, script, season and time |
| 0.5 | Light/visibility probe | log `lit_level` (via the visibility cache after `update_visibility_cache`) at known tiles: lit room, dark field, remembered-but-unseen tile | readings match expectations: room = `LIT`/`BRIGHT`, field at night = `DARK`/`LOW`, remembered tile = `MEMORIZED` |
| 0.6 | Offscreen UI capture probe | implement the plan's Route A at prototype quality: suppress `draw_ter()`, capture the SDL backend's `display_buffer` at present, dump PNG. If Route A fails, prototype Route B (`catacurses`-level) instead | PNG shows sidebar panels and a menu; the `w_terrain` rect is identifiable, empty/transparent, and its rect is reported with the frame. Readback cost per frame is measured and recorded (feeds the Phase 4 budget) |
| 0.7 | Input injection probe | feed a key event through `input_context::get_raw_input()`'s path programmatically | game reacts as if typed (e.g. opens inventory) |
| 0.8 | Headless/no-window probe | run per the plan's Spike B decision — the **null `catacurses` backend** (decided in Phase 0; the hidden-window tiles variant is dropped, as the tiles build is not part of this project) | game runs a scripted turn sequence without a visible window. Do not pass this on an SDL dummy-video-driver assumption — the run must actually render UI frames |
| 0.9 | Tileset parse probe | run `tools/gfx_tools/compose.py` on UltiCa, then parse the composed `tile_config.json` **and** one decomposed per-sprite entry; extract a multitile wall entry | composed output exists (it is not in the CDDA repo — generating it is part of the test); extracted entry has fg/bg plus the **six neighbour multitile subtiles** (`center, corner, edge, t_connection, end_piece, unconnected`) — **corrected in Phase 0 (2026-11-15):** decomposed entries carry at most these six; `open`/`broken` are state-variant ids, not additional_tiles (all eight names exist in `multitile_keys` in `cata_tiles.cpp`); one season-suffixed variant and one `looks_like` chain resolve |
| 0.10 | Licensing check | script lists licenses of bundled assets/code used, from both repos' `LICENSE.txt` including third-party notices | `docs/licensing.md` reviewed; Mushroom Dream excluded |

**Gate:** 0.3 (as redefined), 0.4, 0.6, 0.7 and 0.8 pass, or a documented fallback (in-process integration) is chosen.

**Phase 0 gate results (2026-11-15):** 0.3 ✅ harness + source · 0.4 ✅ · 0.5 ✅ · 0.6 ✅ standalone prototype (both routes) · 0.7 ✅ harness · 0.8 ✅ headless harness · 0.9 ✅ as corrected (compose.py run deferred — pyvips + full PNG tree) · 0.10 ✅ · **0.1 ⛔ not run here (curses/text build only — the tiles/SDL3 build is not required by this project, updated 2026-11-15; the curses compile is infeasible at 1.5 CPU/6 GB in the recon environment) · 0.2 ⛔ not run (needs 0.1)**. Verdict: **conditional pass** — mandatory tests pass at harness level; 0.1/0.2 and the in-game spike halves (probe patch in `tools/phase0/fork_probe_patch.md`) are a hard precondition for Phase 3, not for Phases 1/2/5. Full record in Part I's Phase 0 completion record and `docs/recon.md`.

---

## 4. Phase 1: Protocol and skeleton

> **Phase dependencies:** ⛔ requires the Phase 0 gate. Phase 2 work may start on the draft schema in parallel, but this gate is required by Phase 3.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 1.1 | Schema round-trip (Rust) | build every message with random valid data, encode, decode — including the sparse `FieldOnTile` list, a connect-group table with per-def indices, and `fg_ref`/`bg_ref` = −1 | decoded equals original |
| 1.2 | Schema round-trip (C++) | same in C++ | decoded equals original |
| 1.3 | Cross-language golden files | C++ writes golden files; Rust reads them; and the reverse | byte-identical and semantically equal |
| 1.4 | Verifier on corrupt input | flip bits and truncate golden files | decoder returns an error, never panics or reads out of bounds (FlatBuffers verifier must run before any table access, both languages) |
| 1.5 | Fuzz targets | `cargo-fuzz` on frame decoder and each message | no crashes in 10 min on CI-nightly, 60 s on PR |
| 1.6 | Framing | split frames across arbitrary read boundaries; coalesce several frames in one read | all messages recovered in order |
| 1.7 | Oversize/malformed length | send frame with absurd length prefix | connection rejected cleanly, no large allocation |
| 1.8 | Handshake | version match, mismatch (older/newer), missing hello | match proceeds; mismatch gives clear error and clean exit |
| 1.9 | Forward compatibility | add an unknown field to a golden message | old reader ignores it |
| 1.10 | Heartbeat | stop answering pings | peer detects within the timeout and shuts down |
| 1.11 | Shutdown both ways | kill engine; kill server; send `Shutdown` | other side exits within 2 s, no orphan processes |
| 1.12 | Input round-trip | engine sends key events to stub server | server receives the same sequence, order preserved |
| 1.13 | Shared-memory header | create/attach/destroy region from both languages | header fields (incl. `seq` and dirty rect) read identically; region removed on exit |
| 1.14 | **Seq/epoch semantics** | send deltas with a skipped `seq`, a duplicated `seq`, and an old `epoch` | skip → engine requests a fresh snapshot; duplicate → applied zero times (idempotent discard); old epoch → discarded. (Replaces any turn-number-based test: the protocol has no turn field) |
| 1.15 | Platform matrix | run 1.6-1.14 on Linux, then Windows | same results |

**Gate:** all of the above on Linux; CI green; golden files committed.

**Phase 1 gate results (2026-11-15):** 1.1 ✅ · 1.2 ✅ · 1.3 ✅ · 1.4 ✅ (with documented FlatBuffers limits) · 1.5 ⚠️ fuzz-shaped sweep only · 1.6 ✅ · 1.7 ✅ · 1.8 ✅ · 1.9 ✅ · 1.10 ✅ (logic) · 1.11 ✅ · 1.12 ✅ · 1.13 ⚠️ header agreement only · 1.14 ✅ · 1.15 ⛔ Windows not run. Verdict: **conditional pass** — CI workflow written, first GitHub run pending repo creation. Full record in Part I's Phase 1 completion record.

---

## 5. Phase 2: Engine MVP (fake data)

> **Phase dependencies:** ⛔ requires the Phase 0 gate; soft-requires Phase 1 for its gate. Runs in parallel with Phases 1, 3, 5.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2.1 | Single block mesh | one opaque block | 6 faces, correct normals and winding |
| 2.2 | Adjacent blocks | two touching blocks | shared faces removed |
| 2.3 | Greedy merge | 16x16 flat slab | one quad per visible large face; vertex count matches expectation |
| 2.4 | Chunk boundaries | block at edge, neighbor present/absent | face culled only when the neighbor chunk has an opaque block |
| 2.5 | Property: greedy equals naive | `proptest` random chunks | visible surface area and covered cells identical between greedy and naive meshers; no overlapping quads |
| 2.6 | Transparent/cutout passes | mixed block kinds | each block lands in the correct pass |
| 2.7 | Camera math | yaw/pitch/zoom clamps; follow target | values stay within limits; no NaN at extremes |
| 2.8 | Z-cutaway selection | each cutaway mode, player at several z including z = −10 and +10 extremes | expected set of z-layers/chunks drawn; no index errors at the extremes |
| 2.9 | Dirty propagation | change a boundary block | both chunks marked dirty; unrelated chunks untouched |
| 2.10 | Visual snapshot: fake house | `--headless-capture` mode (offscreen `wgpu`, no window/surface — §5 Phase 2, §7) from 3 fixed cameras | image diff within tolerance of golden. **Environment condition (2026-11-15):** requires a `wgpu` adapter — lavapipe/WARP software adapter or a real GPU. On a machine with no adapter (the recon VM: no `/dev/dri`, no Vulkan ICD), this test is **not run** and the gate relies on 2.1–2.9 mesh-level tests instead; a blank or no-adapter render is never a pass. The capture mode itself (CLI, offscreen path, clean no-adapter error) is part of this test's deliverable even where the pixel comparison cannot run |
| 2.11 | Resize and minimize | resize sequence, zero-size surface | no panic; depth buffer recreated |
| 2.12 | Benchmark: meshing | `criterion`, 12x12xN chunk (N = cutaway depth) | under 2 ms per chunk (single thread) |
| 2.13 | Benchmark: frame | scene filling the real view: up to 132×132 tiles stored, camera view at radius ~60, cutaway z-range | median under 16 ms on the reference machine. (v1's "60x60x10" scene understated the world; the scene must match plan §11) |

**Gate:** 2.1-2.11 pass; benchmarks meet budgets. 2.10's pixel comparison is adapter-conditional per its row (2026-11-15): where no `wgpu` adapter exists it is recorded as not-run with the capture deliverable still required, and must pass on an adapter-equipped machine before Phase 2 is called fully closed.

**Phase 2 gate results (2026-11-15):** 2.1 ✅ · 2.2 ✅ · 2.3 ✅ · 2.4 ✅ · 2.5 ✅ · 2.6 ✅ · 2.7 ✅ · 2.8 ✅ · 2.9 ✅ · 2.10 ⚠️ capture deliverable proven; **real headless renders produced and visually verified the same day via lavapipe** (see the Phase 2 record addendum — `captures/house.png`, `captures/house_cutaway0.png`), but the formal golden image-diff is not yet established · 2.11 ✅ (lib level) · 2.12 ✅ greedy 0.397 ms/chunk · 2.13 ⛔ not run (llvmpipe frame times are not evidence for the 16 ms budget), data proxy: 121 chunks / 17,424 tiles → 5,132 quads meshed. Verdict: **conditional pass**. Full record in Part I's Phase 2 completion record.

---

## 5b. Phase 2b: Shader field effects and HD-2D look

> **Phase dependencies:** ⛔ requires the Phase 2 and Phase 5 gates; soft-requires Phase 3. Runs in parallel with Phases 3, 4, 5, 2c.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2b.1 | Effect-parameter extraction | run the voxelizer effect pass on the fixture tileset + UltiCa field sprites | every field type yields an `effects.json` entry (or a logged explicit fallback); extraction is deterministic (two runs byte-identical); palette values match a direct sample of the source sprite |
| 2b.2 | No sprite rendering of fields | render the effect test scene; inspect the render path and captures | no field is drawn from a sprite texture/quad — fields come only from the effect shader pass (code-level check + captures show volumetric falloff, not a flat quad) |
| 2b.3 | Intensity mapping | smoke/fire/gas tiles at game intensities 1, 2, 3 | rendered density/opacity is strictly monotonic with intensity; intensity 3 smoke fully obscures the tile behind it, intensity 1 does not |
| 2b.4 | Displayed-field weighting | tile with two fields (smoke + fire), `displayed_field` = each in turn | the displayed field dominates the composite in both configurations |
| 2b.5 | Effect captures | `--headless-capture` of the effect test scene (fixed cameras, pinned time) | image diff within tolerance of goldens (adapter-conditional like 2.10: lavapipe small scenes here, never a blank/no-adapter pass) |
| 2b.6 | Post stack isolation | toggle bloom, DoF, grade independently; capture with all off | all-off output is pixel-identical to the pre-2b renderer on the same scene; each pass changes only its own aspect (bloom adds no geometry shift, DoF keeps the focal band sharp) |
| 2b.7 | UI legibility under grade | Phase 4 UI frame composited over a graded frame | UI pixels are not graded/bloomed (post applies to the world only, UI composites after) |
| 2b.8 | Effect performance | full-view scene with 500 active field tiles, 1440p, RX 9070 XT | effect pass + post stack under 3 ms (§11) |

**Gate:** 2b.1–2b.4, 2b.6 mandatory; 2b.5 adapter-conditional as in 2.10; 2b.8 on target hardware.

**Phase 2b gate results (2026-11-15):** 2b.1 ✅ (fixture + real UltiCa 39/39 field types, 0 fallbacks, byte-identical re-run) · 2b.2 ✅ · 2b.3 ✅ · 2b.4 ✅ · 2b.5 ✅ (lavapipe, adapter-conditional) · 2b.6 ✅ · 2b.7 ✅ · 2b.8 ⛔ not run (target hardware). Verdict: **conditional pass** — parent re-ran `cargo test -p voxelizer -p fx` (all pass) and the full workspace (**75/75**), and visually inspected the post on/off capture pair. Engine integration of the fx crate is a recorded follow-up, not a gate item. Full record in Part I's Phase 2b completion record.

---

## 5c. Phase 2c: Ray-traced renderer (Sparse-Voxel-64Tree)

> **Phase dependencies:** ⛔ requires the Phase 2 gate; soft-requires Phases 5 and 3. Runs in parallel with Phases 3, 4, 5, 2b.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2c.1 | Tree build correctness | build trees from known worlds (fake house, random proptest worlds); traverse on CPU | every solid block in the world model is hit by a CPU traversal ray cast at it; every air block is missed; tree node count within a sane bound of the occupied-voxel count |
| 2c.2 | Material side-table indexing | for the same worlds, read back / recompute the flattened order and compare side-table entries per node | 100% of leaf nodes carry the material/lit/flags of the voxel they represent — a single indexing mismatch fails the test |
| 2c.3 | GPU traversal vs CPU oracle | cast a fixed ray set (grid of origins × directions) through both the WGSL traversal (readback) and a CPU reference traversal | identical hit voxel, normal and distance (within float tolerance) for every ray |
| 2c.4 | Render parity captures | `--headless-capture --renderer rt` vs `--renderer raster` on the fake house + a textured scene, fixed cameras | same visible geometry (silhouette diff within tolerance); RT adds shadows/AO — differences confined to lighting, itemized in the comparison report |
| 2c.5 | Rebuild cost | time tree build + flatten + upload for: full stored world (132×132 × cutaway z) and a typical per-turn delta | full build under 20 ms; delta path under 2 ms — if the single-tree design misses the delta budget, the per-column-tree design is measured instead and the numbers recorded either way |
| 2c.6 | Cutaway parity | RT captures at the Phase 2 cutaway settings incl. z = −10/+10 | identical layer selection to gate 2.8's expectations |
| 2c.7 | Picking parity | cast picking rays at known tiles through the tree path | returned tile coordinates equal the mesher-era DDA results on the calibration scene |
| 2c.8 | Visibility discipline | world with `BLANK`/never-seen and `MEMORIZED` regions | never-seen voxels are absent from the tree (or provably rendered black); shadow rays do not leak identifiable detail from unseen areas |
| 2c.9 | Frame time (target hardware) | bubble-scale scene, 1440p, RX 9070 XT, primary + shadow rays | median under 16 ms (§11). Never gated on llvmpipe numbers |
| 2c.10 | Crate-compat smoke | workspace builds against pinned `Sparse-Voxel-64Tree =0.1.0` with our wgpu 24.0.5 | builds clean; `Tree64GpuManager` bind group used in a capture (or the R0 spike's fallback arm — own upload of the crate's flattened bytes — is the recorded path with its own smoke test) |

**Gate:** 2c.1–2c.8, 2c.10 mandatory; 2c.9 on target hardware; the default-renderer decision is recorded in Part I with the 2c.4/2c.9 evidence.

> **Gate results (2026-11-15, parent re-run; workspace 91/91):** 2c.1 ✅ · 2c.2 ✅ · 2c.3 ✅ · 2c.4 ✅ (IoU 1.000) · 2c.5 ⚠️ measured (full build 91.7 ms release on this VM vs 20 ms target-hardware budget; dirty-column delta 0.110 ms) · 2c.6 ✅ · 2c.7 ✅ · 2c.8 ✅ · 2c.9 ⛔ RX 9070 XT · 2c.10 ✅ (primary arm). **CONDITIONAL PASS** — condition is 2c.9 only. Default renderer: **raster** (RT opt-in) until 2c.9 evidence exists. Full record: Phase 2c completion record in Part I + docs/phase2c-rt.md.

---

## 5d. Phase 2d: Multitile layer selection (connected walls)

> **Phase dependencies:** ⛔ requires the Phase 2c texture hookup and the Phase 5 gate; soft-requires Phase 3 for real connect-group semantics. Runs in parallel with 2e, 2f, Phases 3, 4.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2d.1 | Connection masks | build known wall layouts (isolated, straight run, corner, T, cross, end cap) as voxel worlds; compute each voxel's cardinal mask | every voxel's mask equals the hand-computed mask for its layout |
| 2d.2 | Layer selection | for a multitile material with known `subtile_layers`, resolve layers for each mask in 2d.1 | selected side layer == the voxelizer's layer for that mask, 100% of cases; non-multitile materials resolve to base layers |
| 2d.3 | Render evidence | RT capture of the wall-run scene with textures, per-layout close-ups | each layout visibly samples its variant layer (pixel comparison against the base-layer render differs exactly on the connected faces); no-textures render unchanged |
| 2d.4 | No regression | re-run 2c.4 parity + full workspace suite | silhouette IoU still 1.000; suite green |

**Gate:** 2d.1–2d.4 mandatory.

> **Gate results (2026-11-15, implementing session's own re-run):** 2d.1 ✅ · 2d.2 ✅ (fixture 16/16 + real-UltiCa index fidelity) · 2d.3 ✅ (lavapipe; per-layout variant hues, depth/sky/no-textures invariants) · 2d.4 ✅ (workspace suite green, 2c.4 parity re-run green). **GATE PASSED** — no conditions (2d has no target-hardware gate). Full record: Phase 2d completion record in Part I + docs/phase2d-multitile.md.

---

## 5e. Phase 2e: Texture mipmaps (RT renderer)

> **Phase dependencies:** ⛔ requires the Phase 2c texture hookup. Runs in parallel with 2d, 2f, Phases 3, 4.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2e.1 | Mip chain correctness | generate the chain for known layers (checkerboard, gradient, cutout with transparent edges); compare against a CPU reference downsample using the Phase 5 premultiplied/edge-clamped rules | every level within tolerance of the reference; cutout edge texels carry no color bleed from transparent neighbors; cutout mean alpha per level tracks level 0 (coverage preservation) |
| 2e.2 | LOD selection (ray cones, analytic) | CPU reference of the LOD formula (docs/phase2e-mipmaps.md §2) vs GPU debug-LOD-view readback on a textured plane: fronto-parallel at doubling distances, then at increasing slant | distance doubling raises LOD by 1 (±0.15); slant adds −log2(cos θ) (±0.15); GPU and CPU agree within 0.15 LOD at every probed point, silhouettes excluded |
| 2e.3 | Aliasing reduction | distant textured wall captured from two camera poses offset by a sub-pixel amount, mip build vs level-0-only build | inter-pose pixel variance on the wall region drops by at least half with mips; close-up capture (level 0 range) is pixel-identical to the pre-mip build |
| 2e.4 | No regression | re-run the texture hookup gates (treebuild checkerboard, engine UltiCa) + full workspace suite | all pass |

**Gate:** 2e.1–2e.4 mandatory. Method: improved ray cones per docs/phase2e-mipmaps.md (the integrated implementation plan, 2026-11-15).

> **Gate results (2026-11-15, parent session's own re-run on lavapipe):** 2e.1 ✅ · 2e.2 ✅ (GPU debug-LOD readback matches CPU reference; doubling = +1.00 ±0.15) · 2e.3 ✅ with substitution (level-0 baseline is a CPU simulation of nearest sampling; two-pose variance 143.44 → 0.00; close-up pixel-identity vs the pre-mip build not re-verified) · 2e.4 ✅ (workspace suite green). **CONDITIONAL PASS** — conditions in the Phase 2e completion record in Part I.

---

## 5f. Phase 2f: Raster-path textures (mesher UVs)

> **Phase dependencies:** ⛔ requires the Phase 2 and Phase 5 gates; soft-requires the Phase 2c texture hookup (shared spec builder) and 2d (shared layer selection). Runs in parallel with 2d, 2e, Phases 3, 4.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 2f.1 | Mesher UV/layer output | mesh a known single-material chunk; inspect emitted vertices | every face's UVs span the full [0,1]² of its layer; layer index equals the material's layer for that face class; greedy merging never merges faces with different layers |
| 2f.2 | Raster textured capture | `--headless-capture --renderer raster --textures <ultica_run1>` on the fake house | capture shows the same materials as `captures/house_rt_textured.png` (silhouette IoU vs the RT capture ≥ 0.99; color differences itemized as lighting-model differences only) |
| 2f.3 | Shared selection | same scene through both renderers' spec builders | identical (material → layers) resolution in both paths (one shared builder, asserted by test) |
| 2f.4 | No regression | raster capture without `--textures`; full workspace suite | flat capture byte-identical to the pre-2f golden; suite green |

**Gate:** 2f.1–2f.4 mandatory.

> **Gate results (2026-11-15, parent session's own re-run on lavapipe):** 2f.1 ✅ · 2f.2 ✅ (silhouette IoU 1.000 vs flat; textured raster 3,187 distinct colors vs 15 flat; same materials as RT capture, chained via 2c.4 parity) · 2f.3 ✅ (shared spec builder asserted) · 2f.4 ✅ (flat golden unchanged; workspace suite green). **CONDITIONAL PASS** — condition: windowed path still flat (capture path only); see the Phase 2f completion record in Part I.

---

## 6. Phase 3: Game-side export (highest risk)

> **Phase dependencies:** ⛔ requires the Phase 0 and Phase 1 gates. Runs in parallel with Phases 2 and 5. On the critical path to playable.

### 6a. Exporter correctness
All comparisons in this section are fork-side Catch2 tests calling the game's own query APIs — the exporter is wrong by definition if it disagrees with them.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 3.1 | TileDefs completeness | load base data; enumerate all terrain, furniture, traps, field types, monster types, item types, vehicle parts from the game's registries | every id appears exactly once, with name, string flags + resolved hot flags, connect-group indices that resolve into the exported group table, open/close/bash result ids, `looks_like`, and fg/bg refs |
| 3.2 | TileDefs with mods | load 3 representative mods, chosen by content type: one adding terrain/furniture, one adding monsters/items, one adding field types or vehicle parts | mod ids present in the right tables; no duplicates; no crashes |
| 3.3 | Snapshot vs game queries | for a seeded world, compare each exported tile against direct queries: `map::ter()`, `map::furn()`, `map::tr_at()` (trap id), `map::field_at()` — compared as a **full map of (type → intensity)**, `map::i_at()` — compared as **stack count + uppermost item id**, `map::has_graffiti_at()` | 100% match across all exported z-levels. A single-id field or item comparison does not satisfy this test |
| 3.4 | Snapshot bounds | player near a bubble edge (132×132) and near z limits (−10, +10) | no out-of-range reads; unexplored areas flagged `BLANK`/never-seen, not fabricated |
| 3.5 | Light and visibility export | after `map::update_visibility_cache(z)`, compare exported values per tile to the visibility cache and `map::apparent_light_at()` | `lit_level` enum matches **exactly** per tile (it is an enum, not a quantized float — no tolerance); the exported seen float is within the documented quantization of the cache value |
| 3.5b | Map-memory layer | visit a street, move away / break line of sight, then export | remembered tiles appear in the memory layer with terrain/furniture ids and `MEMORIZED` lit level; they carry **no** live entities, items, or fields, even if the real tiles have since gained them |
| 3.6 | Entity export | spawn player, NPC, several monsters via the game's creature tracking | positions, type ids and flags match game state; **`facing` is only ever `NONE`/`LEFT`/`RIGHT`** — any other value fails the test |
| 3.7 | Overlay list | player with armor, wielded item, mutation; one monster with overlays | overlay ids **and order** equal `Character::get_overlay_ids()` / `monster::get_overlay_ids()` output, which is what `cata_tiles::draw_entity_with_overlays` consumes |
| 3.8 | Vehicle export | vehicle on a road at several facings, including a non-axis-aligned one | per-part absolute positions equal `vehicle::bub_part_pos()` / `abs_part_pos()` for every part; exported `face_deg` equals `face.dir()` within float tolerance. The engine must not be re-deriving positions from mounts — this test compares against the game's pre-rotated results |

### 6b. Delta invariant

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 3.9 | **Delta equals snapshot** | random seeded action sequences (move, open/close doors, bash, pick up and drop items — including onto existing stacks, fire, craft, wait, start a fire and let it spread) plus non-action changes; after each send boundary apply deltas to a mirror and compare with a fresh snapshot — comparison includes **field maps with intensities, item stack counts, and vehicle part positions**, not just terrain ids | identical in 1000+ turns across 10 seeds |
| 3.9b | **Map shift** | scripted walk that crosses a submap boundary so `map::shift` recenters the bubble, and a z-level change (stairs) | mirror stays in sync across the shift; snapshot origin/epoch handling is exercised — a shift must not corrupt the mirror or be silently dropped |
| 3.10 | Missed-change hunt | scripted events known to change tiles outside the player's action (fire spread, monsters opening doors, field decay/intensity change, explosions, vehicle motion). Plus a unit test: mutate a submap directly with no player action and confirm the dirty-submap diff emits it | mirror stays in sync in all cases |
| 3.11 | Gap recovery | drop a delta, then send next | engine detects the `seq` gap and requests a fresh snapshot; state recovers |
| 3.12 | Stale and duplicate deltas | send a delta with an old `epoch`, and re-send the last delta unchanged | old-epoch discarded; duplicate discarded with no double-apply (e.g. no duplicated field intensity) |

### 6c. Non-interference and robustness

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 3.13 | **No simulation change** | run the same seed and input script with `--frontend=ipc` and without; compare a **canonical state dump** (exported tile/entity/vehicle state + avatar stats) at fixed checkpoints | dumps identical. Do **not** hash raw save files: saves embed timestamps and other non-simulation data, so a raw-file hash can fail without any simulation difference — if save comparison is wanted, diff parsed save JSON with a documented exclusion list |
| 3.14 | Replay determinism | same seed and script twice (Catch-seeded RNG fork-side) | exported snapshot hashes identical at every checkpoint, over the hash field list defined in Principle 4 |
| 3.15 | Headless operation | run 500 turns with no window (per the Phase 0 headless decision) | no crash, no window created |
| 3.16 | Engine disconnect | kill engine mid-turn | game autosaves or exits cleanly per design; no corrupted save |
| 3.17 | Slow engine | engine stops reading for 10 s | game never blocks on the socket beyond a bounded buffer; on overflow the documented behavior (drop deltas + force resync via new epoch, or pause) is what actually happens, verified by the mirror recovering |
| 3.18 | Export cost | time the exporter per send boundary on the full view window | snapshot export under budget (target: under 20 ms for the view window in plan §11); delta under 2 ms for a typical turn, with the dirty-submap count logged so cost regressions are attributable |
| 3.19 | Upstream suite | run CDDA's Catch2 suite on the fork | no new failures compared with baseline 0.2 |
| 3.20 | Save/load | save, reload, continue | exporter state rebuilds; engine resyncs via a new epoch |

**Gate:** 3.3, 3.5, 3.9, 3.13, 3.14, 3.19 are mandatory; others must pass or have a ticketed known gap.

**Phase 3 gate results (2026-11-15):** 3.1 ⚠️ harness-level · 3.2 ⛔ blocked (0.1/0.2) · 3.3 ✅ · 3.4 ✅ · 3.5 ✅ · 3.5b ✅ · 3.6 ✅ · 3.7 ✅ · 3.8 ✅ · 3.9 ✅ (1,100 turns × 10 seeds) · 3.9b ✅ · 3.10 ✅ · 3.11 ✅ · 3.12 ✅ · 3.13 ⛔ blocked · 3.14 ✅ harness (hash `90b795eb6673ecae`) · 3.15 ⛔ blocked · 3.16 ⛔ blocked · 3.17 ✅ · 3.18 ✅ measured · 3.19 ⛔ blocked · 3.20 ⛔ blocked. Verdict: **conditional pass** — harness-level proof parent-verified (`tools/fork-harness/run.sh` exit 0); mandatory in-game tests 3.13/3.19 and the rest of the blocked set run on a build-capable machine per `docs/phase3-fork.md` §5. Full record in Part I's Phase 3 completion record.

---

## 7. Phase 4: UI compositing and input

> **Phase dependencies:** ⛔ requires the Phase 3 gate; soft-requires Phase 2. Runs in parallel with Phase 5.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 4.1 | UI golden frames | capture main menu, character creation, inventory, crafting, message log, sidebar — at a fixed terminal size, font and panel layout (layouts are user-configurable; goldens pin one) | image diff within tolerance of goldens |
| 4.2 | Map-region masking | capture gameplay UI | the **`w_terrain` rect reported by the fork** (not an assumed fixed rect — it depends on terminal size and panel layout) is transparent (alpha 0) where the engine draws the world; everything outside it is opaque as the game drew it |
| 4.3 | Torn-frame test | writer produces frames with an embedded checksum at max rate; reader samples continuously | zero torn frames in 10 min |
| 4.4 | Double-buffer sequencing | pause reader/writer at random points | reader always sees a complete frame |
| 4.5 | Dirty rect | change a small region | only that rect is re-uploaded; the full frame matches a full re-upload |
| 4.6 | Resize and DPI | resize through several sizes and DPI scales | UI texture and engine stay consistent (display-buffer pixels vs window pixels under the backend's scaling factor are handled); no crash; text readable |
| 4.7 | Key matrix | letters, digits, symbols, arrows, function keys, modifiers, key repeat | each reaches the game as the same key the player pressed, via `input_context` resolution — test at the action level ("opens inventory") as well as the raw-key level |
| 4.8 | Text input | name entry, search boxes, non-ASCII characters | text appears correctly |
| 4.9 | Focus handling | menu open vs closed, driven by the UiFrame mode flag | camera drags ignored when a menu is open; game keys pass through when appropriate |
| 4.10 | Mouse to UI | click menu entries and sidebar panels | correct entry activates (UI clicks stay in UI/display-buffer coordinates) |
| 4.11 | Mouse picking | click world tiles for look/target/examine on a calibration scene | tile coordinates sent match the tile under the cursor; and a click that lands on a UI panel is *not* treated as a world pick (mode flag separation) |
| 4.12 | Latency | key press to updated UI frame on screen | median under 50 ms, p99 under 100 ms. If Route A (display-buffer readback) is used, its measured readback cost from 0.6 is included in this budget |
| 4.13 | **End-to-end script** | create character, start game, move, open inventory, craft, save, quit, relaunch, load | script completes with all checkpoints |
| 4.14 | Soak | scripted random play for 30 min | no crash; RSS growth under 5%; no handle growth |

**Gate:** 4.1-4.9, 4.13 mandatory; 4.11 may ship as keyboard-only if documented.

---

## 8. Phase 5: Voxelizer and textures

> **Phase dependencies:** ⛔ requires the Phase 0 gate; soft-requires Phase 1. Runs in parallel with Phases 2, 3, 4.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 5.1 | Fixture tileset golden | run on the fixture tileset (with multitile, fg-over-bg, oversized-sprite and `looks_like`-only cases) | outputs byte-identical to goldens |
| 5.2 | Determinism | run twice on UltiCa | byte-identical atlas, texture array and `materials.json` |
| 5.3 | Top face | for each terrain material in the fixture + samples from UltiCa | top texture equals the **fg composited over bg** source (pixel-exact after compositing); an fg-only comparison fails by design for transparent-fg floors |
| 5.4 | Side face rules | floor, wall, grass samples | side darkening within configured range; walls use wall sprite directly |
| 5.5 | Multitile mapping | all **16 neighbor masks** from the 4 cardinal neighbors (this is the game's own model: `cata_tiles::get_tile_values` takes a 4-neighborhood array and derives subtile + rotation via `get_rotation_and_subtile`), **plus** connect-group cases where the neighbor is a *different* id in the same connect group (`get_tile_values_with_ter`) | each mask/group case maps to the variant the game's function pair produces — expected values are taken from the game's logic, not a hand-written table alone. Slots covered: center, corner, t_connection, edge, end_piece, unconnected. (`open`/`broken` are state variants, not neighbor-mask variants — covered in 8.4) |
| 5.5b | Sprite resolution chain | ids that resolve via variant (`_var_`), via season suffix at a pinned season, and via `looks_like` | each resolves to the same sprite the game's `find_tile_looks_like` chain resolves; none is reported missing |
| 5.6 | Alpha handling | sprites with holes (fences, trees) and glass | correct cutout/transparent flags; no color bleed at edges |
| 5.7 | Mipmaps | generate mip chain | no edge bleeding between array layers; alpha-aware downsample verified on test patterns |
| 5.8 | Extrusion | furniture with known alpha mask and height | voxel columns match mask; height matches override or heuristic |
| 5.8b | Sprite geometry | sprites from every UltiCa sheet size (20×20, 32×36/48, 32×64, 64×64, 64×96, 96×96) | atlas UVs and stored width/height/offset equal the `tile_info.json` values per sheet; oversized sprites are anchored by their offsets, not centered |
| 5.9 | Override file | override a material and a height | override applied; invalid overrides give a clear error |
| 5.10 | **Coverage** | run against full UltiCa plus the game's TileDefs, resolving every id through the full chain (exact → variant → season → `looks_like` → filler) **before** counting it missing | at least 98% of base-game terrain and furniture ids have non-placeholder materials; the rest are listed in `missing.log` with the resolution stage that failed |
| 5.11 | Unknown id fallback | request an id with no material and no `looks_like` | placeholder renders; log entry emitted once |
| 5.12 | Attribution | generate `ATTRIBUTION.md` | lists tileset, per-sheet sources where `tile_info.json` provides them, authors/sources, license **including the third-party notices**; matches inputs used |
| 5.13 | Visual scenes | headless render of house, road, field, forest, shelter | image diff within tolerance of goldens |
| 5.14 | Contact sheet review (manual) | review `contact.png` | reviewer checklist signed off; problem ids filed as override tasks |
| 5.15 | Performance | voxelizer run time on UltiCa (including the `compose.py` step if the pipeline performs it) | under 60 s on reference machine |

**Gate:** 5.1-5.3, 5.5, 5.10-5.13 pass; manual review done.

**Phase 5 gate results (2026-11-15):** 5.1 ✅ · 5.2 ✅ (parent re-run, `diff -r` byte-identical) · 5.3 ✅ · 5.4 ✅ · 5.5 ✅ · 5.5b ✅ · 5.6 ✅ · 5.7 ✅ · 5.8 ✅ · 5.8b ✅ · 5.9 ✅ · 5.10 ⚠️ tileset-universe 100% (7,717 materials, 0 missing); TileDefs-based form open · 5.11 ✅ · 5.12 ✅ · 5.13 ⛔ image-diff not established (contact sheet + capture regression instead) · 5.14 ✅ manual — Joey signed off 2026-11-15 ("looks fine for now", provisional) · 5.15 ✅ 6.2 s. Verdict: **conditional pass**. Full record in Part I's Phase 5 completion record.

---

## 9. Phase 6: Billboards

> **Phase dependencies:** ⛔ requires the Phase 2 and Phase 5 gates; soft-requires Phase 3. Runs in parallel with Phase 7.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 6.1 | Orientation | camera at several yaws | quads always face the camera, upright (rotation about Y only) |
| 6.2 | Depth sorting | entities in front of and behind walls and windows | correct occlusion in headless render |
| 6.3 | Alpha edges | sprites with soft edges | no dark halos; matches alpha-test/dither design |
| 6.4 | Overlay composition vs game | compose the same entity two ways: (a) the game's own rendering — `cata_tiles::draw_entity_with_overlays` in tiles mode, captured offscreen; (b) the engine's compositor fed the exported overlay list | pixel diff within tolerance for player with several equipment sets and one mutated character. Tolerance must allow for the game's `height_3d` offsets; geometry/scale differences are aligned before diffing |
| 6.5 | Overlay cache | change one overlay; same signature again | new composite only when the signature changes; cache hit otherwise |
| 6.6 | Facing and mirroring | facing `LEFT`, `RIGHT`, `NONE` | `LEFT` mirrored horizontally, `RIGHT`/`NONE` unmirrored. There is **no up/down case** — v1's version of this test described sprites that do not exist in this tileset system |
| 6.7 | Item and creature variety | all monster sprites in UltiCa, plus item piles (uppermost item per tile) | no missing sprites after the resolution chain (5.5b); missing list generated for genuine gaps |
| 6.8 | Shadows | blob shadow under entity | follows entity, hidden when under roof if designed so |
| 6.9 | Picking billboards | click on sprite pixels, including transparent areas and oversized sprites (offset geometry from 5.8b) | opaque pixels hit; transparent pixels pass through; hit area matches the sprite's real size/offset, not a 32×32 box |
| 6.10 | Stress | 2000 billboards visible | frame time within budget; instance buffer resizes without crash |
| 6.11 | Visual scene | zombie crowd near walls and windows | matches golden |

**Gate:** 6.1, 6.2, 6.4, 6.5, 6.10, 6.11.

---

## 10. Phase 7: Lighting, FOV and atmosphere

> **Phase dependencies:** ⛔ requires the Phase 2 and Phase 3 gates; soft-requires Phase 5. Runs in parallel with Phase 6.

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 7.1 | Light mapping | tiles at each `lit_level` (`DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT`) plus a seen-float gradient | rendered brightness is monotonic with the enum order and with the seen float within a level |
| 7.2 | Visibility states | currently seen, remembered (`MEMORIZED`), never seen (`BLANK`) | bright, desaturated/dark, and black/not-meshed respectively |
| 7.3 | No light leaking | lit room next to a dark room through a wall | dark side stays dark in render |
| 7.4 | Smoothing | gradient across neighbors | no hard seams at chunk borders |
| 7.5 | Light updates | toggle a light source in-game | engine updates only affected chunks within one frame of the delta |
| 7.6 | Day/night | game time across a day (pinned times) | tint transitions smoothly; matches game time of day |
| 7.7 | Weather | rain/fog states from the game | correct effect on/off; no performance cliff |
| 7.8 | Visual scenes | lit house at night, flashlight in field, remembered street | match goldens |
| 7.9 | Performance | full lighting over the real view (radius ~60, cutaway z-range, 132×132 stored) | frame budget maintained. (Scene size corrected from v1's 60x60x10) |

**Gate:** 7.1-7.3, 7.5, 7.8, 7.9.

---

## 11. Phase 8: Hard cases and release

> **Phase dependencies:** ⛔ release gate requires the Phase 4, 5, 6 and 7 gates; individual tests need only their own phase (8.1–8.3: Phase 3; 8.5: Phases 3–4).

| # | Test | Method | Pass criteria |
|---|---|---|---|
| 8.1 | Vehicle orientations | sample facings across the full circle — 0°, 7°, 15°, 23°, 45°, 90°, 179°, 270°, 359° and values hit during a scripted turn — there is **no fixed step count**: `face.dir()` is a continuous `units::angle` (the 15° in v1 is a pathing clamp in `vehicle::immediate_path()`, not an orientation quantum) | for every sampled facing, each part's engine position equals the game's `vehicle::abs_part_pos()` / `bub_part_pos()` for that part (exact tile match); facing error within float tolerance |
| 8.2 | Vehicle movement | drive a scripted path, including a turn | smooth model motion; no part drift (re-check 8.1 at intervals during the drive); deltas keep in sync (3.9 invariant holds with vehicles included) |
| 8.3 | Vehicle damage/removal | break or remove parts, open/close vehicle doors, fold/remove a roof part | model updates correctly, including the roof / no-roof part split the game's renderer uses |
| 8.4 | Door and window states | open, closed, broken, bashed — for terrain doors and furniture doors | each action arrives as a delta carrying the **new terrain/furniture id** (state is an id change via the game's open/close/bash transforms, not a state flag); model and texture match the new id after each change |
| 8.4b | Open air, roofs, multi-storey | two-storey house fixture + basement: stand on floor 1, floor 2, and outside looking in, at several cutaway settings | no phantom floor over `t_open_air`, roof terrain renders as the floor above per the game's `roof` field, no missing ceilings/floors at any cutaway |
| 8.5 | Animations/projectiles | fire weapons, throw, explosion | effects appear at the right tiles and times; no leftover effects |
| 8.6 | Overmap | open the overmap | UI texture shows it correctly (it is game-drawn UI, incl. its own overmap sprites); input works; returning restores the 3D view |
| 8.7 | Mod smoke | launch with top popular mods, one at a time | no crash; missing-material report under agreed threshold (measured with the 5.10 resolution chain) |
| 8.8 | Long soak | 2 h scripted play | no crash; RSS growth under 5%; frame time stable |
| 8.9 | Memory leak check | sample RSS and GPU memory every minute | no monotonic growth after warm-up |
| 8.10 | Performance regression | full benchmark suite | all budgets in `plan.md` section 11 met |
| 8.11 | Clean-machine install | install package on fresh Linux and Windows VMs | engine launches the game, generates or loads assets (incl. running or shipping the `compose.py` step), reaches gameplay |
| 8.12 | Upgrade path | install new version over old, load an old save | save loads; assets regenerate; no stale cache issues; engine resyncs via new epoch |
| 8.13 | Crash reporting | force engine and game crashes | logs captured with version info (including the pinned upstream commit of the fork); user-facing message clear |
| 8.14 | Final manual playthrough | 1 h session on each platform | checklist: UI, camera, combat, driving, crafting, saving |

**Gate (release):** all earlier gates green; 8.1, 8.4b, 8.7, 8.8, 8.10, 8.11 pass.

---

## 11b. Phase 9: Fine-grid world — prop model stamping (Route A)

> **Decision (Joey, 2026-11-16): Route A.** The v2 carrier (Stage G) emits per-furniture voxel models (`Material.model`, 119 over full UltiCa) but the engine world is a coarse block grid (Phase 0 `WorldScale`: 1 tile = 1 block, 3 blocks per z-level) with no sub-tile space, so furniture renders as textured blocks and no model is ever drawn. Route B (a model-instancing pass parallel to billboards) was considered and set aside. Route A gives the world a fine resolution and stamps models into it at world-build time, so props become ordinary world geometry for both renderers. Picking, LOS and collision are untouched by construction: they run on game tile data, never on the visual grid (Stage G rule).

**Design lock (9.0):**
- **Resolution.** FINE = 32 cells per block edge: 1 cell = 1/32 tile in plan, and a z-level is 3 blocks = 96 cells. A model cell (1 sprite pixel) is exactly 1 fine cell — stamping is a copy, never a resample.
- **Representation.** NOT a uniform dense subdivision (a flat 32× blow-up of the dense `VoxelGrid` is a non-starter on memory). The world stays a `ChunkMap` of coarse `Block`s; a **fine layer** stores per-tile fine volumes only for tiles whose furniture carries a model, in run-length columns (props are mostly solid/empty runs). The unified read path is a **fine sampler** honouring the existing `VoxelSampler` contract (`mesher::grid`) at fine resolution: a fine cell resolves to (a) the stamped cell if its tile has a fine volume, else (b) its containing coarse block, expanded. Chunks containing no fine tiles keep today's coarse fast path end to end; a per-chunk flag records which path a chunk takes.
- **Prop colours.** Model voxels carry RGB, not atlas layers. At scene build, every distinct palette colour across the scene's models is assigned a **synthetic material** whose texture layers are 1×1 solid-colour images appended to the scene texture array (deduped scene-wide; `engine::textures::build_texture_spec`). Stamped cells store that material index, so the mesher's layer/UV machinery and the RT `tex_albedo` path work unchanged, and 1×1 layers make mip/LOD behaviour trivial. Model voxels are opaque albedo; EMPTY stays air.
- **Stamping.** In `snapshot_world::build_columns` — the one builder feeding both renderers. A furniture placement whose material has a model **suppresses** its coarse furniture block and stamps the model into the tile's fine volume. Origin: the tile footprint's south-west corner at floor-slab top (fine z = (`level_base_block` + `floor_slab_blocks`) × 32). Orientation per the v2 contract (model front → tile south); 90°-step rotation by tile facing where the snapshot supplies one (9.7), unrotated until then. A model exceeding its tile footprint is a build error, never a silent clip (the shipped set is within-tile by construction: width ≤ 32, tank length = cap diameter).
- **Deltas.** A furniture change on a tile restamps that tile's fine volume (dropping it if the new furniture has no model) and recomputes the chunk's fine flag — the 8.4 door/furniture id-change path is the test vehicle.

| # | Work item | Method | Pass criteria |
|---|---|---|---|
| 9.1 | Fine volume + fine sampler | `mesher` crate: per-tile run-length fine volumes + a `VoxelSampler` at fine resolution over (fine layer, coarse `ChunkMap`) | sampler unit tests: stamped cell wins, coarse expansion exact, cross-boundary reads correct; memory per prop tile measured and recorded |
| 9.2 | World-build stamping | `snapshot_world::build_columns` stamps + suppresses; fake/sample scenes gain prop fixtures (counter, bookcase, fridge, table, tank, bed) | stamped scene's fine volumes match the source `PropModel`s voxel-for-voxel (round-trip through the world, not just the file) |
| 9.3 | Synthetic solid materials | scene texture spec appends 1×1 layers for prop colours; material index space shared with atlas materials | colour round-trip: a stamped cell's rendered colour equals its palette RGB in both renderers; dedupe count = distinct colours |
| 9.4 | Raster mesher fine path | flagged chunks mesh at fine resolution (naive per-cell faces first); boundary rule: a coarse face against a fine tile is culled iff all 32×32 fine cells behind it are opaque, and vice versa via the fine sampler | boundary fixture (prop against a wall, prop in a doorway) renders with no holes and no z-fighting; face-count budget recorded for the lab scene |
| 9.5 | RT tree over the fine grid | `treebuild` `VoxelWorld` gains a fine adapter; `build_tree` consumes the fine sampler; per-leaf texture scale: a fine cell of a coarse material maps 1/32 of the block's texture per axis, so the Phase 2e LOD constant Δ adjusts by the leaf's cell fraction (synthetic solids are scale-free) | node-count and build-time budgets recorded on the sample scenes; analytic mip gates (2e.2 family) extended to fine leaves and passing |
| 9.6 | Lighting | v1: a fine cell takes its containing coarse block's baked light | capture documents the known limit — props do not occlude the coarse bake (light can leak through a fridge) — as a named Stage H candidate, not a silent acceptance |
| 9.7 | Rotation + multitile scope | rotate stamped models in 90° steps by snapshot facing where present; multitile furniture stays per-tile (models are per-tile by construction) | rotated fixture matches the unrotated model under the same rotation, voxel-for-voxel; out-of-tile model is a build error (9.0) |
| 9.8 | Frame composition regression | fields / vehicles / transients passes (Phase 8 wiring: vehicles stamped in the world pass before the visibility mask) run against a fine-layer world | Phase 8 harness gates 3.21/3.22 and the cross-phase regression suite (§12) all pass unchanged |
| 9.9 | Perf + memory | world-build time, tree nodes, fine-layer memory, lavapipe frame captures on the sample scenes + a prop-dense lab stress scene | numbers recorded in the phase record; RX 9070 XT frame-time verdict stays the standing deferred measurement (raster remains default until it exists) |
| 9.10 | Rollout | behind a world-build flag (`--fine-props`), default OFF | flag OFF: renders byte-identical to today (golden gate); flag ON with no models in the scene: also byte-identical. Default flips only after 9.1–9.9 pass and Joey's sign-off on the lab-scene capture |

**Gate (Phase 9):** 9.1–9.5 and 9.8 pass; 9.6's limit documented; 9.9's numbers recorded; 9.10's byte-identical proofs green. Then the two-pass rule (pipeline doc §27) applies to in-scene prop review exactly as it did to showcases.

> #### Phase 9 slice 1 completion record (2026-11-16, per the maintenance rule)
> **What was actually built (9.1–9.3, CPU foundation; built in worktree `cdda-voxel-p9` off 5f7773e, parent-merged and re-verified in the main tree):** `crates/mesher/src/fine.rs` — `FINE = 32`; `FineVolume` (run-length columns: one RLE column per (x, y) of the 32×32 footprint, runs along fine z, `memory_bytes()` measured on real capacities); `FineLayer` (tile → volumes, level depth stored on the layer, 96 at default scale; level-scoped removal for restamps); `ModelCells` trait mirroring `PropModelData`'s layout so mesher stays engine-independent; `stamp_model` (front-flush-south: model z = d−1 lands at fine y offset 31; copy, never resample; out-of-tile = `ExceedsTile` error, layer unchanged); `FineSampler` (stamped cell wins, else the containing coarse block via `div_euclid` expansion — exact for negative coords); `Chunk.has_fine` flag + `ChunkMap` flag refresh. `crates/engine/src/snapshot_world.rs` — `FurniturePlacement`, `stamp_furniture` (origin at floor-slab top; modelled furniture writes NO coarse block — the suppression contract, gate-pinned; batch-validated up front so a bad model fails the batch atomically), `restamp_tile` (drop/replace a tile's volume at a level — the 9.0 delta rule at layer level), `PropModelCells` adapter. `crates/engine/src/textures.rs` — `PropColorSet` (distinct palette colours, first-appearance dedupe, synthetic id = base+1+index) and `extend_texture_spec_with_props` (one solid tile_size² layer per colour — uniform array dimensions force tile_size, the layer is solid so mip/LOD is trivial — with `[l, l, l, 1]` material entries). `build_columns` is byte-unchanged.
>
> **Test-gate results, parent's own re-runs in the main tree:** fine_gate **7/7** (stamped-wins, coarse expansion exact over every unstamped fine cell incl. slab boundaries, cross-boundary + negative tiles, front-flush mapping, out-of-tile atomicity, restamp/drop + flag clearing, memory bound); phase9_stamp_gate **1/1** (counter + tank fixtures: every fine cell of both prop tiles round-trips through the world, suppression = coarse grid above the floor slab all AIR, dedupe count = distinct colours, spec layer bytes == palette RGB, restamp both directions); mesher **20/20**, treebuild **17/17**, engine **63/63** across all 14 targets (incl. frame_gate 5/5 and golden_gate **3/3 — 6 scenes 0 mismatched px on lavapipe, captures with real quads/colours, negative test 1.73% as designed**); voxelizer gate_tests **13/13** in the main tree (its worktree-only 5.1 failure was the known ATTRIBUTION.md absolute-path artifact, proven by path-normalized byte-identity). **Memory (9.1):** bookcase-like volume (32×22×20, 11,552 cells) = **154,152 B vs 3,932,160 B dense (25.5× smaller)**, under the dense/4 bound.
>
> **Deviations:** (1) the stamping resolver carries an explicit lifetime (`Option<&'m dyn ModelCells>`) — the brief's elided form does not compile (E0106); semantics unchanged. (2) `stamp_furniture` validates the whole batch before stamping (stronger than per-placement atomicity). (3) frame_gate's 4.12 latency assert (p99 < 100 ms) flaked twice under parallel VM load during the worktree runs and passed standalone and in both full main-tree runs — no frame/render code is touched by this slice.
>
> **Not yet done (at slice 1):** slice 2 = 9.4 (raster fine meshing + boundary rule) and 9.10 (`--fine-props` rollout + lab-scene capture for Joey's sign-off); slice 3 = 9.5 (RT fine adapter + per-leaf texture scale), 9.6 (lighting limit, documented when the fine path first renders), 9.7 (rotation), 9.8 (Phase 8 regression against a fine-layer world), 9.9 (perf/memory numbers). Stage F part 2's border height-blending stays held until 9.4/9.5 land (its own record, §5 Stage F addendum).

> #### Phase 9 slice 2 completion record (2026-11-16, per the maintenance rule)
> **What was actually built (9.4 + 9.10; built in worktree `cdda-voxel-p9s2` off 8cd1d95, parent-merged and re-verified in the main tree):** `mesher::fine::mesh_chunk_fine` — a chunk flagged `has_fine` meshes in two passes: a naive coarse pass applying the **boundary rule** (a coarse face whose neighbor cell is AIR inside a fine volume's tile is culled iff all 32×32 fine cells of that volume on the face plane are opaque — the floor slab under a full-footprint prop culls, under a table it does not) and a naive fine pass resolving every stamped cell's neighbors through the `FineSampler`. Unflagged chunks keep the existing `mesh_chunk` — **the 9.10 dispatch contract**: props present but nothing stamped ⇒ meshes are literally the pre-Phase-9 meshes. Engine: `CaptureArgs.fine_props` + `props: Option<PropScene>`; `frame::PropScene` with hand-built fixture models (bookcase/counter/fridge/table/tank/bed, furn ids 200–205) placed in the sample-scene house and the fake house; `scene_prep` `_vp` variants thread props through both raster paths — textured scenes extend the spec via slice 1's `PropColorSet` before `stamp_tex_layers`, untextured scenes get `textures::prop_only_spec`. Lit path: visibility mask applied **before** stamping (placements whose tile lost its floor slab are skipped); the position-based per-vertex baker gives fine quads their containing tile's light — **the 9.6 v1 rule, in force from this slice**. RT + fine props returns a named `CaptureError::Device` (slice 3 wires it) rather than silently dropping props. `--fine-props` parses in the shared scene-flags path (headless + windowed).
>
> **Test-gate results, parent's own re-runs in the main tree:** `phase9_render_gate` **3/3** — CPU boundary fixture (wall face culled by a full-footprint prop, drawn by the naive mesher as contrast; doorway wall faces and slab top drawn; fine-pass coverage **set-equal** to a brute-force `FineSampler` reference over every stamped cell, expanded count == set size in both passes — no duplicates, no holes); face-count budget (sample scene: **210** coarse-only quads → **8,970** total with fixture props, of which **8,092** fine-pass; budget 250,000); GPU 9.10 proofs on lavapipe with real runtimes — OFF vs ON-with-empty-PropScene **0 mismatched px, mean |err| 0** (byte-identical), fixture props vs OFF **849 px (0.368%)** differ with 6 of 13 fixture palette colours present in frame. mesher **20/20**, engine **66/66** across all targets, golden gate **3/3** with goldens unmoved (flag defaults OFF).
>
> **Lab capture (9.10 deliverable):** `captures/phase9_lab.png` (1280×720, sample scene, `--fine-props`, cutaway 0, 8,970 quads) + `captures/phase9_lab_closeup.png` — props read in place: bookcase with book-spine colours, legged table, counter, fridge, yard tank. These are hand-built **fixture** models proving the pipeline; real furniture models arrive via the voxelizer's `PropModelData` when live scenes wire real furniture ids.
>
> **Deviations:** (1) PerBlock light-bake mode **with props** routes through the per-vertex bake for that frame (documented in `prepare_raster_lit_fine_v`; no double-bake) — per-block averaging over mixed coarse/fine geometry is a slice-3-or-later question. (2) The windowed path needed `fine_props` threaded through `WindowConfig`/`App` in addition to the shared flag parse. (3) `prop_only_spec` on an empty colour set emits one dummy white layer (a spec must carry ≥1 layer); the prepare paths never call it for empty scenes. (4) The fixture bed model is built but placed in neither fake scene.
>
> **Not yet done:** slice 3 = 9.5 (RT fine adapter + per-leaf LOD), 9.6 (lighting-limit documentation — the v1 rule above is implemented; the written limit + Stage H note remain), 9.7 (rotation), 9.8 (Phase 8 regression against a fine-layer world), 9.9 (perf/memory numbers; RX 9070 XT verdict still deferred). Stage F part 2's border height-blending stays held until 9.5 lands.

> #### Phase 9 slice 3 completion record (2026-11-17, per the maintenance rule)
> **Context:** the first whole-slice-3 dispatch was refused by a safety policy on the helper side before any work began (no worktree, no code). On Joey's word ("do phase 9 slice 3 in smaller parts, break it up as much as possible") slice 3 was rebuilt as eleven micro-slices (3a–3k), each built, gated and committed separately on `main` in the main session — no subagent dispatch. All commits local, NOT pushed.
>
> **What was actually built (9.5 + 9.6 + 9.7 + 9.8 + 9.9):**
> - **9.7 rotation (3a `758a33f`, 3b `1d829a3`):** `mesher::fine::stamp_model_rotated` — the front-flush-south stamp composed with `quarter_turns % 4` steps of `(lx, ly) → (FINE−1−ly, lx)`, height unchanged, `ExceedsTile` validated at every turn; `stamp_model` delegates at 0 turns. Engine half: `FurniturePlacement.quarter_turns` (0 = front south … 3 = east; 0 is exactly the pre-9.7 stamp) threaded through `stamp_furniture`. Multitile scope statement (plan 9.7): models are per-tile by construction, so rotation is per-tile and a stamp never writes outside its tile — gate-pinned.
> - **9.6 documentation (3c `3d676e2`):** `docs/phase9-fine.md` §1 — the v1 rule (fine cell takes its containing coarse block's baked light; mask before stamping; PerBlock-with-props routes through the per-vertex baker, never double-baked; RT follows via the fine adapter's per-tile `lit` rewrite) and its named limit (props do not occlude the coarse bake: light leaks through a fridge, a prop's back is lit like its front; error bounded by one tile's light because game truth is per-tile `lit_level`), with fine-aware occlusion recorded as a Stage H candidate.
> - **9.5 LOD (3d `668cc6c`):** `treebuild::lod::FINE_LOD_ADJUST = −5.0` (= −log2(32)) and `ray_cone_lod_fine`, adjustment applied *before* the clamp via a shared adjusted helper; `ray_cone_lod` unchanged. Synthetic solids are scale-free (every mip identical), so the adjustment is for the adapter's general case (a fine cell carrying a coarse material's texture).
> - **9.5 adapter (3e `42d3728`):** `treebuild::world::VoxelWorld::from_fine_layer` — the fine tree's world holds ONLY stamped cells, in fine coordinates, material/colour/flags from the stamped block, `lit` defaulting to `LIT_LIT` for the caller's per-tile rewrite; new `FineLayer::tiles()` iterator. Empty layer → empty world → no fine tree. Pure additions; the coarse path is untouched.
> - **9.5 CPU trace (3f `362dbf1`):** `treebuild::cpu::cast_ray_two` — ray scaled into fine space, fine distance divided back by 32, nearer hit wins in block units, fine wins an exact tie; `None`/empty fine returns `cast_ray`'s result bit-for-bit.
> - **9.5 GPU trace (3g `70b480d`):** `RtScene` gains a fine tree (second `Tree64GpuManager` at group 2, built with its own layout object; side table + `FineUniforms` at group 1 bindings 9/10) via `RtRenderer::with_scene_fine`; `with_scene` delegates with `fine_present = false`. WGSL `fine_descend`/`fine_trace` mirror the coarse traversal over the fine globals; `trace_two` composites by distance (mirror of `cast_ray_two`), converts the fine hit's `voxel_min` back to coarse space for shading and keeps `fvoxel_min` for query output. `shade()` reads node/side from the winning tree, applies the −5 LOD adjustment before the clamp (`cone_lod_adj`), and shadow rays use `trace_two` (props occlude the sun); the occupancy-AO probe stays coarse-only in v1 (noted in `docs/phase9-fine.md` §2). `point_occluded` unchanged.
> - **9.5 engine wiring (3h `48662df`):** the slice-2 RT rejection in `frame.rs` is removed; `render_rt_frame` resolves the effective prop scene (explicit `props` win, else the fixture under `--fine-props`) and dispatches to `render_rt_frame_props`, mirroring the raster fine preparation into two trees: visibility mask before stamping on the lit path (placements on `Hidden` tiles skipped), `lit_rewrite` post-pass applying the Phase 7 contract to both worlds (fine cells by containing tile — the 9.6 rule), variant preparation on the coarse world as before, and the attached spec = the variant spec extended with the prop colours at its own end, with fine ids shifted onto it (`shift_fine_ids`) so variant virtual ids and synthetic ids cannot collide. `RtFrameOut` gains `fine_leaves`/`fine_nodes` (0 on the coarse-only path, which is otherwise untouched). `scene_prep`'s `PropPrep`/`prep_prop_spec`/`stamp_props_into` are now `pub(crate)` and shared.
> - **9.8 regression (3i `b2be14a`):** new composition test — stub-house, BOTH renderers, plain / props-only / car-only / combined captures; the combined frame differs from each single-input frame (>0.05% px), so the Phase 8 vehicle pass and the fine pass compose against a fine-layer world without dropping each other.
> - **9.9 numbers (3j `125b662`):** `phase9_numbers_gate` measures and `docs/phase9-fine.md` §3 records: coarse world 686 cells / 727 nodes / ~2.3 ms build; fixture fine tree 19,576 cells / 19,998 nodes / ~130–200 ms build, fine layer 668,856 B (~131 KB per prop tile); stress scene (200 placements, 100 tiles) 420,528 cells / 429,363 nodes / ~3.1–5.0 s build, layer 15,014,976 B; lavapipe frame times 640×360 — raster 105 / 401 / 3,406 ms, RT 328 / 504 / 8,747 ms (coarse-only / fixture / stress). The fine tree runs ~1 leaf per solid cell (no same-material leaf merging at build time). Lavapipe on this throttled VM establishes shape, not hardware predictions.
>
> **Test-gate results, own re-runs in the main tree:** mesher fine_gate **10/10** (3a: rotated stamps match an independent rotation at every turn, turn-0 identity, oversize errors at every turn); phase9_stamp_gate **2/2** (3b: rotated placements match the mesher's rotated stamp voxel-for-voxel at turns 0–3 across two tiles, no spill); treebuild mip_gate **5/5** (3d analytic fine-LOD gate: exact −5 offset in the unclamped region, both clamps exercised — the first draft of this gate failed legitimately at the max-mip clamp and was fixed by construction, not by relaxing); phase9_fine_world_gate **5/5** (3e adapter exactness incl. negative tiles and pass→flags mapping; 3f two-tree nearer-wins + no-fine identity by `RayHit` equality; 3g GPU `trace_rays` parity with `cast_ray_two` in both modes + render occlusion, centre depth 5.0 vs 8.0); phase9_rt_gate **3/3** (3h: OFF vs ON-empty **0 mismatched px, mean |err| 0**; fixture >0.1% px with ≥3 palette colours; lit fixture changes the frame and BLANK-masked props vanish exactly; 3i composition); phase9_numbers_gate **2/2**; full engine suite green (lib 27, frame_gate 5, golden_gate 3 — goldens unmoved, phase8_wire_gate 6/6 unchanged, all stage gates), full treebuild suite green, mesher + voxelizer suites green.
>
> **Deviations / incidents:** (1) A workspace-wide `cargo fmt --all` during 3b reformatted 88 files (the repo is not fmt-clean under this rustfmt); everything was reverted and the 3b edits re-applied cleanly before gating — never run `cargo fmt --all` in this repo. (2) The 9.9 gate's first run failed its suppression assertion: on the sample scene the fixture tiles carry no coarse furniture blocks, so stamped and unstamped coarse counts are equal — the assertion was wrong, not the code (suppression is proven in phase9_stamp_gate); the gate now asserts stamping never *adds* coarse cells, with the reason recorded in the test. (3) The RT fine tree re-uploads the coarse tree as its dummy when no fine tree is supplied (a `Tree64GpuManager` needs a tree); `fine_present = 0` means it is never traced — memory cost only, on a path that predates per-scene manager reuse anyway.
>
> **Phase 9 gate verdict: PASS** — 9.1–9.5 and 9.8 pass; 9.6's limit documented; 9.9's numbers recorded; 9.10's byte-identical proofs green in both renderers. **Remaining before `--fine-props` can flip default:** Joey's sign-off on the RT lab capture (the 9.10 contract — default stays OFF), and the standing RX 9070 XT frame-time measurement (2c.9). **Stage F part 2's border height-blending hold is RELEASED** by 9.5 landing (its own record, §5 Stage F addendum); it is a separate slice, not part of Phase 9.

---

## 12. Cross-phase regression suite

These run on every PR once their phase has shipped:

- Protocol golden and fuzz-smoke (Phase 1), including seq/epoch semantics (1.14)
- Mesher properties and benchmarks (Phase 2)
- Effect-parameter determinism and intensity mapping (Phase 2b); tree-build correctness, side-table indexing and GPU-vs-CPU traversal parity (Phase 2c)
- Delta equals snapshot, map shift, and no-simulation-change (Phase 3)
- UI golden frames and the end-to-end script (Phase 4)
- Voxelizer determinism, resolution chain and coverage (Phase 5)
- Billboard overlay comparison (Phase 6)
- Lighting visual scenes (Phase 7)

---

## 13. Phase gate checklist (copy into each phase PR)

- [ ] All gate tests for this phase pass in CI
- [ ] All earlier gates still pass
- [ ] Benchmarks meet the budgets in `plan.md` section 11
- [ ] New golden files reviewed in the PR diff
- [ ] No quarantined gate tests
- [ ] Known gaps listed with tickets
- [ ] Docs updated (`recon.md`, protocol notes, voxelizer rules)
- [ ] Manual review items (if any) signed off with screenshots attached

---

## Appendix: verification notes for this test plan

All items below checked 2026-11-15 against CDDA `master` @ `6a3e59e0` (raw source):

- **Catch2 harness (v1 §2, test layer "Fork unit") — Verified.** `tests/test_main.cpp`: `#define CATCH_CONFIG_RUNNER`, a `Catch::Session`, and a `CataListener : Catch::TestEventListenerBase` whose section-start handler initializes "the cata RNG with the Catch seed". Catch is vendored (`tests/catch/catch.hpp`); `tests/CMakeLists.txt` exists. Fork exporter tests belong in this harness.
- **Map query APIs (test 3.3) — Verified, named.** `src/map.h`: `ter_id ter(const tripoint_bub_ms&)`, `furn_id furn(tripoint_bub_ms)`, `const trap &tr_at(const tripoint_bub_ms&)`, `const field &field_at(const tripoint_bub_ms&)`, `map_stack i_at(const tripoint_bub_ms&)`, `bool has_graffiti_at(const tripoint_bub_ms&)`.
- **Light/visibility APIs (tests 0.5, 3.5) — Verified, named.** `src/map.h`: `lit_level apparent_light_at(p, cache)`, `visibility_type get_visibility(lit_level, cache)`, `update_visibility_cache(int zlev)`, `invalidate_visibility_cache()`, per-level caches in `level_cache`. `lit_level` enum in `src/lightmap.h`.
- **Overlay APIs (test 3.7) — Verified** in the plan's Appendix A (#10): `get_overlay_ids()` order is the composition order.
- **Vehicle part positions (tests 3.8, 8.1) — Verified, named.** `src/vehicle.h`: `tripoint_bub_ms bub_part_pos(const map&, int)`, `tripoint_abs_ms abs_part_pos(int)`, `precalc_mounts(int, const units::angle&, const point_rel_ms&)`, `pivot_rotation` array, `tileray face`. Continuous angle — no orientation step count exists to test against.
- **Multitile model (test 5.5) — Verified.** `cata_tiles::get_tile_values(int t, const std::array<int,4> &tn, …)` builds a 4-bit value from the cardinal neighborhood (order seen in callers: south, east, west, north) and calls `get_rotation_and_subtile`; `get_tile_values_with_ter` is the connect-group-aware variant. 16 masks is the correct cardinal case count.
- **Not verified / deliberately left open:** exact readback cost of the SDL `display_buffer` (measured by spike 0.6, budgeted in 4.12); whether a stable release branch differs from `master` in any of the APIs above (re-check this appendix if the fork pins a release instead).

> **Addendum (2026-11-17, Stage H pond circularity — worktree `cdda-voxel-pond` branch `pond-tuning` off 60b70da, Joey sign-off 2026-11-17, MERGED to main @ 35cdafd, NOT pushed):** shore.rs box blur replaced by a windowed morphological opening of the waterline region (adaptive radius <= 28, pond-corner detection via 1-water-incident grid points + >= 2-edge shore runs both directions; land-point isthmus tiles like 0xF8 deliberately excluded). 3x3 mosaic: best-disk IoU 0.837 -> 0.951, radial std 4.14 -> 1.25 px, corner coverage 38.6% -> 21.3%; straights unchanged. Gates: stage_e 3/3, shore unit 3/3, engine pond 4/4, golden 3/3; only failure is the known gate_5_1 worktree path artifact (reproduced on pristine HEAD). Full record: pipeline doc §34. Remaining Stage H items unchanged (tank barrel ribbing, `assets/ultica_run1` regeneration, fine-aware lighting occlusion candidate).

D2 (frames/racks) addendum (2026-11-17): all 15 hand-built in props.rs
(d2_model, sprite-referent builds): stepladder (stepped front + runged
rear A-frame), ladder (single plane, rails + rungs + top board),
butcher rack (A-frame ends, top rail + pegs, hanging hook blocks),
smoking racks (tripod over a stone fire ring; active adds fire + meat
chunks), coat rack (post, 4 hooks, 4 feet), rotary clothesline (pole,
4 arms, 3 line squares), stooks (pole stand; full = bound sheaf with
flared grain head), target box (stamped crate), police barricade
(stamped board, A-frame ends), counter gate (open slab on hinge post +
counter stub), trailer bed head (ribbed panel), camp chair (X-frame +
fabric), stool (round seat, splayed legs). Coverage 263 -> 278 of 351.
64px sprites built at tile scale from their proportions.

D3 (machines/bodies) addendum (2026-11-17): all 18 hand-built in
props.rs (d3_model): alembic (copper boiler, glass arm, receiver
flasks), beverly shear, drill press (profile build), hydraulic press
(H-frame), tire changer (rim ring + swing arm), ergometer (stationary
bike), bench press station, mannequin (pale humanoid), training
dummies x2 (pell: padded torso + helm on a post), pinball (stepped
cabinet, art sampled from the sprite, stamped backbox), mailbox
(tunnel profile + flag), recycle bin (stamped front), water pump,
windmill (lattice tower + fan wheel), exodii portal tower (banded
legs + cradle), warped lamp string (glow blob on a leaning post),
eyewash station. Coverage 278 -> 296 of 351.

Vegetation (class C) addendum (2026-11-17): all 44 modeled in props.rs
(veg_model). Upright flowers/herbs/crops (34) take the cross
construction (plant_model, the zapper/tendril pattern). Flat mats and
water plants take plan slabs: lilypad + flower spurge (2 thick),
fungal mass + fungal tangle (3 thick); the lotus is a plan slab plus
its bud raised at the sprite's pinkest cluster. The four indoor
plants get a terracotta lathe pot under a cross of the foliage rows.
Coverage 296 -> 340 of 351; only the 11 exclusion-pin ids remain.

Exclusions (class A) addendum (2026-11-17): the 11 former exclusion
pins modeled on Joey's instruction -- furniture coverage is now
351 of 351, and the gate's decline test is replaced by a
full-coverage test. Hand builds in props.rs (excl_model): anemone
(rock mound + teal tentacles), bed frame (plan slab, headboard band
raised -- the sprite is a plan view), the three bullet-trailer table
fragments (plan slabs), chair (grey frame, tan panels), deckchair
(wood frame, blue-white striped sling with a back drape -- without
the drape the stepped sling grazes out of view from behind), scrap
antenna (an antenna STATION: brown cabinet with indicators and a
control box, gold-element mast, curved dish arm, second mast),
small dish (the sprite's disc on a dished surface + feed arm),
traffic light (portable: concrete base, green pole, three-lens
head), winter wreath (green ring, brown bow accents).

Prop gallery (2026-11-17): crates/engine/examples/prop_gallery.rs
runs the voxelizer over the real tileset (v2), stamps 124 prop
models into the engine's sample scene and captures the frame
(docs/prop-views/prop_gallery_engine.png). Findings: 4 models built
from 64px sprites exceed the one-tile stamp footprint (FINE=32) and
are refused by design; the frame builder assigns one texture-array
layer per distinct prop colour -- a full gallery has ~3,600, over
this VM software adapter's 256-layer cap (and over the 2,048
typical hardware cap), so the example quantises its copies to a
shared 240-colour palette. The emitted model data is unaffected.

## Addendum 2026-11-17 (evening): fixture water + roof seating fixes (Joey's catches from the prop gallery)

- WATER: mesher `tile_shape` classified TILE_WATER as full-height (3 blocks) while grass/floors are 1-block slabs, so the sample pond rendered as a translucent tank standing 2 blocks proud of the grass. Water is a FloorSlab now, matching the real snapshot path (t_water_dp height_voxels 1). Shore test premise updated (pond = 16 blocks, not 48).
- ROOF: TILE_ROOF stamped as the TOP slab of its level, so a roof authored on the level above its walls (both fixture scenes, CDDA-style) floated 2 blocks over them. First fix tried (roof moved into the walls' level) broke every cutaway-to-interior gate (~7: 2c.4/2c.6, phase9 props-under-roof, 2g.1a knock-ons). Final fix: RoofSlab stamps as the BOTTOM slab of its level -- a deck seated on the wall plate, exactly the snapshot path's Floor::RoofOf placement; scale.rs doc corrected; frame.rs under-roof probe follows (roof_z = first block of the level above). Cutaway gates pass with their original intent.
- Gate premises updated with reasons recorded in situ: world cutaway test (quad totals are not monotonic under greedy meshing -- seated deck: full 57 vs no-roof 59), rt_gate raster cutaway (silhouette instead of quad count), stage_a richness threshold 100 -> 64 (flat fallback ~35 colors; the deck hides the roof slab's underside, 98 colors on raster). House goldens re-blessed after eyeballing the deck captures; composed golden byte-identical, untouched.
- Full engine suite: only remaining failure is the pre-existing stage_g_model_round_trip_fixture (stale premise from the coverage drive, see the 2026-11-17 plan note at fe5fa62).

## Addendum 2026-11-17 (evening): terrain models -- trees + chain-link fence (Joey: "The fence and trees should also be models" / "Do that engine step")

Voxelizer: `terrain_model_for` emits v2 models for t_tree and t_chainfence from the FG-only sprite. Tree (32x80x32): per-layer crown discs from the foliage silhouette (solid core, sprite leaf pixels on the front shell snapped to the top-24 foliage palette -- bounded colours for the texture-array budget), bark discs drawn over (trunk cap 3.4, limbs 2.0), root flare. Fence: a HALF-segment in the full tile frame -- centre post to west edge post, top rail, diamond mesh on the centre planes (z 15..=16, symmetric about the tile centre so quarter turns map it onto itself). Review strips: docs/prop-views/t_tree.png, t_chainfence.png, t_chainfence_run3.png, t_chainfence_corner.png (fence approved by Joey; tree proportions tall-80 vs squatter still his open call).

Engine: `scene_prep::extend_prop_scene_with_terrain` runs in both fine paths (raster prepare_raster_fine_v, RT render_rt_frame_props) when a textures dir is present. It scans the level-filtered map for TILE_FOLIAGE / TILE_FENCE tiles whose material carries a model: trees get one placement as-is; fence tiles compute their 4-neighbour mask and get ONE placement of a composed model (rotated half-segments unioned per mask, deduped by mask; turns W0 N1 E2 S3 via rot_model, which matches stamp_model_rotated's (lx,ly)->(31-ly,lx)). Modelled tiles are suppressed by replacing them with the TILE_GROUND slab -- NOT air: models stand on the slab top (fine_z_origin = level base + floor slab), so airing the tile left a black pit under the first gallery tree. Isolated fence tiles (mask 0) keep their block. Scope: the fixture-scene fine paths; the snapshot path (build_columns worlds) does not call the extension yet.

Verification: voxelizer suite green; engine suite 19/20 binaries green, only failure the pre-existing stage_g_model_round_trip_fixture (stale premise, see the 2026-11-17 water/roof addendum). Gallery re-rendered: fence run reads as chain-link in the world, trees stand on grass (docs/prop-views/prop_gallery_engine.png + gallery_zoom_fence/tree.png).

## Terrain drive (2026-11-17, Joey: models for every tile, all seasons)
Universe 1,074 terrain materials (387 season variants as separate ids, each from its own sprite);
gate terrain_coverage_gate TERRAIN_TOTAL=1074 GREEN (open-air family exempt: transparent sprite =
empty space). Family builders v1 in props::terrain_model_for (vegetation_model structural zones,
fence panels from the multitile CENTER variant via terrain_fg_image, sprite-faced wall boxes,
doors with swung open panels, window frames, ground/liquid relief slabs, stairs, silhouette
fallback). Sample sheet docs/prop-views/terrain_samples_v4.png. State: ~/workspace/terrain-drive/state.txt.
Batch review follows (two-pass rule); engine snapshot-path wiring is the follow-on.

## Addendum 2026-11-18: terrain drive complete + snapshot-path wiring

Terrain drive COMPLETE: all 1,074 terrain ids batch-reviewed from all angles and fixed
(vegetation f04b0fe, fences aaefa3a, doors+windows 55ad55f, walls a34e9ed, ground+liquids
2e1384e, stairs+others 165b35a; sheets docs/prop-views/{veg,fence,door,wall,ground,other}_batch_*.png).
End-to-end v2 table: 1,422 models (351 furniture + 1,071 terrain; the 3 open-air ids exempt).
Flagged for Joey's verdicts in the batch presentations: bamboo_dead_winter sparseness, burnt-tree
totems, rope-fence drape, dark chainfence posts, wall pseudo-3D "T" shading, puddles as full dirt
slabs, t_water_pump, monkey bars/slides (no fitting builder), thin water tanks as fat boxes,
t_column as plain box, hay feeder, railroad crossing signal.

Snapshot-path wiring (this step): the models now seat on real wire data, not only in the
fixture scene. The voxelizer records a seating kind per terrain model in the v2 `models`
table ("kind": ground|fence|standing; furniture omits it, absence = standing) because only
the router knows which builder produced a model. Engine `Material.model_kind` parses it
(unknown kind = load error); `PropModelData::rotated`/`union_from` moved out of scene_prep's
private copies. `snapshot_world::stamp_terrain` resolves each wire tile's def id -> game id
and seats by kind: Ground replaces the coarse floor slab at the level base in a volume only
as deep as the model (full-depth would shadow the tile's furniture volume); Fence composes
per same-def neighbour mask (W=1 N=2 E=4 S=8) on the slab top, mask 0 keeps the coarse block;
Standing stamps as-is on the slab top. Open-air tiles whose floor resolves via `roof_result`
seat the roof material's model as Ground. All candidates validate up front (oversize fails
the batch atomically). Gate terrain_snapshot_gate (proto sample fixtures): 147 volumes over a
grass/fence/tree/wall/floor/roof submap + atomic oversize rejection. Suites proto+mesher+
voxelizer+engine 251/251; full UltiCa v2 emission verified (kinds on all 1,071 terrain
models, none on furniture).

Seams flagged for Joey: (1) Standing models taller than the level's remaining band (96-cell
walls, 84-cell doors) stamp at the slab-top origin and extend into the level above — the
fixture path's tree convention, kept; a per-kind origin refinement is a design call.
(2) Furniture on modelled ground floats: furniture origin is the slab top (+32) while ground
models top out at +10/+16. (3) A tall standing model and the next level's ground volume on
the same column can overlap in z-range; first-inserted wins lookups (benign: the upper
floor's cells inside a solid wall are hidden anyway).

## Addendum 2026-11-23: seam (1) fixed — tall Standing models seat at the level base (fix 5a)

Joey's call 2026-11-23 (option A1): in `snapshot_world::stamp_terrain`, a Standing model
taller than the band above the slab (level depth 96 - slab 32 = 64 cells at the default
scale) now seats at the level base instead of the slab-top origin. A 96-cell wall fills
its own level exactly (0..96, was 32..128); an 84-cell door ends at 84 (was 116). The
coarse slab is NOT aired — stamped fine cells win over the coarse grid, and the slab
fills the gaps of sparse tall models (door panels) instead of leaving a pit. Shorter
Standing models (fixture tree, 40) keep the slab-top origin. Side effect, flagged to
Joey: real trees (80 tall) also exceed the band, so they seat at the base too — the
model itself is unchanged (his keep-tall decision), but its top now ends at 80 instead
of poking into the level above. Gate: terrain_snapshot_gate updated — the new wall
assertions (base at fine 0, top at 95, nothing at 96) FAIL on the pre-fix code and pass
after; suites proto+mesher+voxelizer+engine all green. Seam (2) — furniture floating
over modelled ground — is NOT fixed by this: furniture stamps through the separate
`stamp_furniture` path at the slab-top origin, unchanged. Seam (3) is moot for tall
Standing models (no more z-overlap with the level above from them).

## Addendum 2026-11-23 (2): seam (2) fixed — furniture seats on the measured ground top (fix 5b)

Joey's call 2026-11-23 (option B — measure, don't publish): `stamp_furniture` now looks
for the tile's Ground volume at its level (the shallow volume seated at the level base;
a full-depth base volume is a tall Standing model under fix 5a, not ground) and seats
the furniture at that volume's top (base + depth: 10 for a floor, 16 for a mound)
instead of the slab-top origin 32. No Ground volume -> slab-top origin unchanged (the
coarse slab is really there). No v2 format change. Requires terrain stamped before
furniture on the combined path. Gate: new `furniture_seats_on_measured_ground_top` in
terrain_snapshot_gate (grass and floor tiles seat at 10, a groundless tile at 32) —
verified failing on the pre-fix code, passing after; phase9_stamp_gate unmoved.

Also recorded 2026-11-23, no code: gasper side stripes CLOSED on Joey's call (option A —
keep the sprite's longitudinal stripes; the blotch mockup stays unapplied, faithfulness
rule intact). `assets/ultica_run1` regeneration deferred on Joey's call (option A) to
the close-out of the terrain-fixes batch (after bamboo/puddle/hand-build sign-offs).

## Addendum 2026-11-23 (3): egg sacks — Joey's route C, step 0 + fix A

Step 0 (measured on the pre-fix models, both pins): the side SURFACE is not flat
(201-208 distinct colours on the side faces, modal share ~10%, luminance std ~27-30,
left-face min-x std 5-6 — comparable to the accepted boulder) and the depth-per-row
profile is not straight (longest equal-extent run 2-3 rows). The flatness is the
OUTLINE: the side top outline sat at the model's full height for the front 12-14 z
cells in a dead-straight run — a rectangular top-front corner, because every column
reaches the front plane at its full sprite height. Fix A (Joey's C route): a
profile-rounding pass at the end of `egg_pile_model` cuts that corner with a quarter
ellipse in the (y, z) plane (centre 0.55/0.55, radii 0.45) by shifting upper columns
back — never emptying one, so the approved front silhouette is unchanged. Post-fix:
the front-most top sits 6-10 cells below the maximum and descends to it; the
mid-depth plateau that remains is the sprite's own flat top row (faithful). Gate:
`egg_sack_side_profile_is_rounded_not_rectangular` in prop_coverage_gate — verified
failing pre-fix (front run 11), passing after; full voxelizer suite green. Presented
to Joey 2026-11-23 (flat orthographic 4-angle sheet); step B (surface fix) only if
he says it still reads flat.

## Addendum 2026-11-23 (4): tank straps — Stage H ribbing item closed (8a-A)

Joey's call 2026-11-23 (option A): smooth barrel + strap bands, cap face as-is.
Measured first: the current EndOn builder's barrel paint is ALREADY a single flat
colour (205 grey, uniform along the length; 4 distinct side colours total, all
neck/stand) — the ribbing in the G.3 rework sheet was the stepped disc's shading in
the render, not banding in the paint, so there was nothing to remove. The missing
mockup element was the straps: `endon_model` now adds two raised strap bands
(`tank_strap_z`: {c-1, c} for c in {l/3, 2l/3}, never the cap plane) — a one-cell-
proud shell ring in the barrel darkened x0.73 (`TankEndFit.strap`), with anchor legs
at the ring's flank extremes running to the ground, drawn before the neck so the
neck wins where they meet. Gates: views_gate's exhaustive tank shape test gained
the strap term (recomputed from the spec) plus band-top/smooth-top colour asserts
and a cap-plane FNV pin (0xb01988571e19d8cf, measured pre-fix — the cap is
byte-identical); verified failing with the strap pass disabled. prop_coverage_gate's
pre-drive count for f_standing_tank updated 23224 -> 23716 (+492 strap cells) as a
deliberate, annotated change. Presented to Joey 2026-11-23 (flat 4-angle sheet).

## Addendum 2026-11-23 (5): fine-aware lighting occlusion landed (8c-A)

Joey's call 2026-11-23 (option A — geometric pass, invented lighting, tuning risk
accepted). `mesher::fine::face_occlusion`: for a visible fine face, count opaque
cells around the air cell in front of it (4 in-plane neighbours + the cell straight
ahead; Pass::Opaque only), factor = 1 - 0.11 x count, floor 0.56; an open face is
exactly 1.0, so unoccluded scenes mesh byte-identically. Raster: `mesh_chunk_fine`
multiplies each fine face's colour by its factor (floor contact shades a prop's
base row 0.89). RT: `engine::rt::fine_occlusion_rewrite` runs after the lit
rewrite in the capture path — `cell_occlusion` (mean over exposed faces) steps the
categorical lit down 1 level below 0.85, 2 below 0.70; Memorized/Blank untouched.
Still a named limit (docs/phase9-fine.md §1 updated): coarse floor/wall cells
BEHIND a prop keep their tile light — leak-through is not fixed, only the prop's
own enclosed/tucked faces darken. Constants are assistant first-pass values.
Gates: mesher fine_occlusion_gate (3 tests incl. mesher-level colour asserts),
engine fine_occlusion_rt_gate; suites mesher+engine green (exit 0, 31 binaries).

## Addendum 2026-11-23 (6): sign-offs — egg sacks and tank straps accepted

Joey's verdict 2026-11-23 on the presented 4-angle sheets ("That looks fine"):
egg-sack fix A is accepted — item 6 CLOSED, step B (surface fix) not done. The
tank strap bands (8a-A) are accepted. The 5b-8 batch is fully closed: 5a b1cd3aa,
5b b530a77, 6 816a786, 7 closed no-work, 8a 60c7b75, 8c 1f9a80c; 8b (ultica_run1
regen) remains deferred to the terrain-fixes batch close-out by its own terms.

## Addendum 2026-11-23 (7): bamboo densify landed (terrain item 1)

Joey's call: dead bamboo densifies to match the sprite. Measured root cause (both
plan suspects): the dead sprites (32x64) carry 7 culms (column clusters at a 0.30
opacity threshold, no gap-merging: widths 2-4 px, gaps 2-3 px); the shared builder's
0.45 threshold plus gap-merge <= 2 fused them into 2 clusters, each drawn as one
fat 3x3 culm. Fix, scoped to the dead family only (`bamboo_model(sprite, dense)`,
dense = id starts with bamboo_dead): threshold 0.30, gap-merge 0, thin
cross-sections (single cell for 1-2 px clusters, plus for wider). Live bamboo is
untouched by construction — and pinned: whole-model FNV hashes for t_bamboo,
t_bamboo_long, t_bamboo_tall in the new gate `bamboo_dead_is_dense_and_live_is_unchanged`
(terrain_coverage_gate), measured pre-fix; the same gate counts >= 6 culm
components at y=20 for both dead ids (was exactly 2). Voxelizer suite green.
Presented to Joey 2026-11-23 (4-angle sheet incl. live bamboo for comparison).

## Addendum 2026-11-23 (8): bamboo sign-off

Joey signed off on the densified dead bamboo 2026-11-23 ("I sign off on this",
on the 4-angle sheet). Terrain item 1 CLOSED. Remaining terrain batch: item 2
puddles (in progress, side thread, puddle-fix branch), item 3 hand-builds (prep
in progress, side thread), then item 4 ultica_run1 regen as close-out (8b-A).

## Addendum 2026-11-23 (9): terrain item 3 hand-builds — built + signed off

Built in worktree cdda-voxel-handbuild, branch hand-build @ 569a93e (NOT merged,
NOT pushed): terrain_manual_model dispatch before the silhouette fallback —
3a t_water_pump, 3b t_vat (Joey's tentative call, furniture tanks untouched),
3c t_monkey_bars x4 (corner-post segments) + t_slide x4 (incline + ladder,
Joey-approved invention), 3d t_column + t_little_column, t_railroad_crossing_
signal x2 (twin red lamps front+back), t_crossbuck_metal/wood x4 (no lamps,
no equipment box). Gate hand_build_gate 6/6, verified failing pre-fix;
voxelizer suite 18 binaries ok (only gate_5_1, the known worktree path
artifact). First signal strip's bowtie X was caught in self-review and rebuilt
square before presentation. JOEY SIGN-OFF GRANTED 2026-11-23 in the hand-build
thread ("They look fine. I sign off") on the 4-angle strips for all families.
Terrain item 3 CLOSED. Remaining terrain batch: item 2 puddles (rework in its
thread — water-shader route approved there), then item 4 ultica_run1 regen.

## Addendum 2026-11-23 (10): hand-build branch merged to main

Branch hand-build fast-forward merged to main 2026-11-23 on Joey's word in the
hand-build thread (main 3b965e7 -> b060e59; the Phase 10 thread's uncommitted
plan.md row was stashed around the merge and restored intact). Post-merge
verification in the main tree: hand_build_gate 6/6, terrain_coverage_gate 2/2,
gate_tests 13/13 (gate_5_1 passes in its real home, as expected). Still NOT
pushed — the stack moves via Joey's PC bundle route; a fresh bundle is needed
to bring the remote current. Remaining terrain batch: item 2 puddles (water-
shader rework in its thread), then item 4 ultica_run1 regen as close-out.

> **Terrain-fixes addendum 11 (2026-11-23, item 4 / 8b — ultica_run1 regen DONE; terrain-fixes batch CLOSED):** full v2 emission re-run from current main (voxelizer CLI, `--tileset fork-src/tilesets/gfx/UltimateCataclysm --v2`, release build; 12,206 materials, 39,856 layers, 14,450 sprites, 0 missing, ~258 s) and installed into `assets/ultica_run1`, replacing the stale Phase 5 v1 table that had been sitting there (7,717 materials, no models). NOTE: `/assets/*` is gitignored — ultica_run1 is a local emission dir by design; this regen is an on-disk state change, not a commit. Verification: counts match the last verified emission (12,206 materials / 1,422 models / 904 macros / 255 shores); all 1,422 model entries structurally validated (hex length == 4*w*h*d, palette indices in range, material_index in range); seating kinds present on all terrain models (745 standing / 246 ground / 80 fence; 351 furniture kindless); exactly 4 `surface: "water"` entries (the puddle seasons); batch spot-checks in the emitted table — f_standing_tank solid count 23,716 (straps in), egg-sack solid counts identical to the pre-fix table (fix A moved cells only), t_bamboo_dead(_season_winter) 6 culm components at y=20, live t_bamboo size [32,40,32] (current-code shape), hand-build ids present as standing. `effects.json` in the dir is untouched by the v2 run and preserved. The 1,076-entry diff vs the Nov 17-18 `target/gallery_assets` emission is format/code evolution since that emission (kind field added 2026-11-18, fd_* transparency reclassification, builder changes through the terrain drive), not regen error — the producing code is what the gates pin. With this, every terrain-fixes item (1 trees record-only, 2 bamboo, 3 puddles, 4 hand-builds, 5a/5b seating, 6 egg sacks, 7 gasper record-only, 8a tank straps, 8b this regen, 8c fine occlusion) is closed.
