# engine — CDDA Voxel Frontend (Phase 2 MVP)

Windowed `winit` + `wgpu` app and headless capture mode over the
`mesher` crate's fake-data world (a procedural stub house). All testable
logic is in the lib (`src/lib.rs`); the bin is a thin shell.

## Design choices

- **Floor/ceiling thickness (plan §14 Q1): thin slabs.** Decided in the
  mesher crate's `WorldScale` (the single config place): a floor tile
  occupies the bottom 1 of 3 blocks of its z-level, a roof tile the top
  1 of 3, full-height tiles (walls, doors, windows) fill all 3. Set
  `floor_slab_blocks == blocks_per_z_level` (3) in `WorldScale` for
  full-block floors instead. Interiors get 2 blocks of clear headroom.
- **Shared rendering:** windowed and headless paths both use
  `render::Renderer` (pipelines, mesh upload, draw) — they differ only
  in the target texture format/surface. A capture is evidence about the
  real renderer, not a parallel one.
- **Debug overlay** is lib state (`overlay::DebugOverlay` + `MeshStats`
  + `FpsCounter`); the windowed app shows its summary in the window
  title (no GUI toolkit in the MVP). The chunk-borders toggle (`B`) is
  overlay state; drawing border geometry is not in the MVP.
- **Frame pacing:** render only when the world is dirty or the camera
  moved (`overlay::FramePacer`); the event loop waits otherwise.

## Windowed controls

| Input | Action |
|---|---|
| Arrow Left / Right | orbit yaw |
| Arrow Up / Down | orbit pitch (clamped 20–85°) |
| Q / E | zoom out / in (clamped) |
| W / A / S / D | move the follow target over the ground plane |
| PgUp / PgDn | cutaway: raise / lower the player-z (visible levels follow) |
| B | toggle chunk-borders overlay flag |
| Esc | quit |

Resize is handled via `overlay::resize_action`: zero-size (minimized)
resizes are ignored and rendering is skipped at zero size; a real
resize reconfigures the surface and recreates the depth buffer.

## Headless capture

```
cargo run -p engine -- --headless-capture \
  --snapshot stub-house \
  --camera 6,6,45,45 [--distance 28] [--cutaway 1] \
  --size 1280x720 --out captures/house.png
# or: tools/capture.sh --snapshot stub-house --camera 6,6,45,45 --size 1280x720 --out captures/house.png
```

No window, no surface, no event loop: offscreen `Rgba8UnormSrgb`
texture + depth, `device.poll(Wait)`, staging-buffer readback, PNG via
the `image` crate. `--camera` is `target_tile_x,target_tile_y,yaw_deg,
pitch_deg`; `--cutaway <z>` draws game levels `<= z`.

On a machine with **no wgpu adapter** (e.g. the recon VM: no
`/dev/dri`, no Vulkan ICD) the mode exits non-zero with
`headless-capture: no wgpu adapter available` and writes no PNG. Pixel
captures need lavapipe/WARP or a real GPU (see plan.md test 2.10's
adapter condition).

`--snapshot` currently accepts only `stub-house` (Phase 2 fake data);
snapshot-file / stub-server sources arrive with Phase 3 data.

## Scene report (2.13 data proxy)

```
cargo run -p engine -- --scene-report
```

Builds and meshes the full stored bubble (132×132 tiles × 3 z-levels,
121 chunk columns) on CPU and prints chunk/tile/block counts, quad and
vertex totals, and total meshing time. This is a data-level proxy, not
a frame-time measurement.
