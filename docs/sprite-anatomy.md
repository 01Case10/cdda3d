# UltiCa sprite anatomy — what the pixels actually are

Phase 5 rework, Stage B. Purpose: before generating tileable textures
from UltiCa sprites (Stages C–F), record **what a sprite's pixels mean**
— which regions are a top face, which are a front face drawn in 3/4
view, where the painted shading lives — so generation samples the
right regions and strips the right shading.

Method: entry JSONs in the decomposed UltiCa checkout
(`fork-src/tilesets/gfx/UltimateCataclysm/`) were resolved to their
fg/bg sprite PNGs; each cited sprite below was opened and measured
(row-band mean colors over opaque pixels, opaque fraction) on
2026-11-16, and cross-checked against the Stage B analyzer output
(`voxelizer preview`, sheet at `target/preview_house.png` in the
Stage B worktree) and the classifier coverage audit. Paths below are
relative to the tileset root above. All sprites are 32x32 unless
noted.

## 1. The layering model (applies to everything)

- A tile is **fg composited over bg**; either may be absent. Neither
  layer alone is "the material": `t_door_c`'s fg has a fully
  transparent top band (the door leaf starts below the frame line)
  and its bg is a separate full-tile sprite, `t_door_frame`
  (`pngs_normal_32x32/terrain/t_door/`). `t_swater_dp`'s fg is the
  water blob; its bg is a plain grass tile (sprite literally named
  `t_grass`). **Always analyze the composite, never the fg alone.**
- **Multitile subtiles**: connected terrain/walls ship as
  center/edge/corner variants plus an `*_unconnected` variant for the
  isolated tile. E.g. `t_grass`'s fg is `t_grass_center`. The
  **center subtile is the purest texture exemplar** (no borders);
  `*_unconnected` variants carry trim/borders on all four edges.
- **Facing variants**: furniture (and some terrain) sprites come in
  directional facings — `f_chair`'s fg is `f_chair_faceS`
  (`pngs_normal_32x32/furniture/f_chair/`). Facing is view dressing,
  not material.
- **Tall/giant sprites**: trees and buildings exceed one tile —
  `t_tree_apple`'s fg is 96x96 (`pngs_giant_96x96/`), `s_apt` is
  32x64 (`pngs_tall_32x64/overmap/`). The voxelizer anchors by sheet
  offset; generators must not assume 32x32.

## 2. Ground terrain — top-down textures, but with transition fringes

Grass is the well-behaved case: `t_grass_center`
(`pngs_normal_32x32/terrain/t_grass/`) is a genuine top-down noise
texture — row-band means are rgb(82,87,47) at the top, middle *and*
bottom; nothing is painted "in front". Dirt is not: in
`t_dirt_unconnected` (`.../terrain/t_dirt/`) the top and bottom row
bands are rgb(84,87,48) — **grass green** — while the center is dirt
rgb(97,89,59). The sprite bakes the grass→dirt transition fringe into
its own edges, because in the 2D game it is drawn between grass
tiles. Used raw as a repeating texture, every dirt tile grows a
grass frame (one of the artifacts behind the rework).

**Rule for generation:** for ground materials, sample the center
region (or the center subtile where one exists); never treat a full
`*_unconnected` ground sprite as a tileable exemplar.

## 3. Floors — top-down, with per-plank painted shading

`t_floor_unconnected` (`.../terrain/t_floor/`) is top-down wood
planks: top band rgb(81,57,37), middle rgb(101,72,49), bottom
rgb(81,58,38) — the variation is plank-to-plank shading and seam
shadow, not a light gradient (the Stage B analyzer reads its
dominant orientation as **Vertical**: planks run vertically in
sprite space). `t_metal_floor_unconnected` is 100% opaque, flat
plate. Floors are the closest thing to ready textures after grass;
what needs removing is the per-plank value jitter if a uniform
floor is wanted, and the seams are *signal* (they set the plank
period for the Stage C wood-floor recipe).

## 4. Walls — front-view illustrations; there is no top face in the sprite

This is the single most important anatomy fact for the rework. A
UltiCa wall sprite is a **face-on illustration of the wall's side**,
drawn for a 2D top-down game:

- `t_wall_unconnected` (`.../terrain/t_wall/`): top band
  rgb(191,177,156) (a darker trim/header line), middle
  rgb(209,202,188) (plaster field), bottom rgb(205,200,189) with a
  baseboard line. The voxelizer's current rule — sprite becomes the
  side face verbatim — is why walls look "right" in 2D terms; the
  block's **top face has no source pixels at all** and is currently
  synthesized from the side's edge rows (see faces.rs), which is
  exactly the streaking Stage F replaces.
- `t_concrete_wall_unconnected` (`.../terrain/t_concrete_wall/`):
  the painted vertical gradient runs **light-at-the-bottom**:
  top rgb(155,154,153) → mid rgb(169,169,167) → bottom
  rgb(190,183,176) (floor bounce light). Other walls gradient the
  other way. **Baked-shading direction is per-sprite, not a global
  constant** — this is why Stage B's analyzer *measures* the
  gradient sign per sprite instead of assuming one.
- `t_brick_wall_unconnected` (`.../terrain/t_brick_wall/`): near-
  uniform brick red in all bands (123,62,49); the structure is the
  brick courses (horizontal mortar lines), which the structure
  tensor reads as horizontal orientation.
- `t_wall_wood_unconnected` (`.../terrain/t_wall_wood/`): vertical
  wood siding, mid band lightest rgb(137,94,35) vs top
  rgb(110,74,28) / bottom rgb(100,66,25) — a center-weighted
  vignette rather than a gradient.

**Rule for generation:** wall *side* textures come from the sprite's
middle field (skip the trim band at the top and the baseboard at
the bottom); wall *top* textures must be generated (Stage F end-cap
rule), never sampled from the sprite — the pixels don't exist.

## 5. Doors and windows — framed illustrations with specular paint

- `t_window` (`.../terrain/t_window_and_variants/t_window.png`):
  top band rgb(224,220,212) is frame/header; the glass field
  rgb(180,201,208) carries **diagonal white specular streaks**
  (clearly visible in the preview sheet) — painted glass shine that
  must not survive into a generated glass texture. The analyzer
  flags it via highlight fraction; shading factor 0.26.
- `t_door_c` over `t_door_frame` (see §1): the leaf is a face-on
  door with frame, handle highlight, and a top-light gradient
  (analyzer shading factor 0.28 — the highest of the house set).
  Doors classify as wood/metal by id, but as *textures* they are
  objects; Stage E treats them via the decal/extrusion route, not
  as tileable wood.

## 6. Water — a shoreline blob over somebody else's terrain

`t_water_dp_unconnected` (`.../terrain/t_water_dp/`): only 4 opaque
pixels in the top row band, 110 in the middle — an irregular pond
shape whose *edge is a shoreline*, composited over a grass bg.
Its mid-band color rgb(61,92,100) is the real water color; its
palette over the composite is dominated by shore/grass tones,
which is why the classifier's pattern rule (id says water) must
outrank the color fallback — the fallback alone reads this family
as dirt/grass. Deep-water center pixels are the exemplar; the
shoreline ring is transition paint, same disease as dirt (§2).

## 7. Vegetation and trees — canopy illustrations with painted shadow

`t_tree_apple` fg (96x96, `pngs_giant_96x96/terrain/t_tree_apple/`):
canopy fills the top and middle (rgb(59,74,37) mid), bottom rows
transparent; the bg is `t_grass_giant`, grass appearing only in the
bg's lower rows. A soft contact shadow is painted under the canopy.
Trees are billboard/extrusion content in this pipeline, not block
textures — recorded here because their palettes (dark greens,
rgb(35,46,26) top) define the `foliage` family the classifier's
color fallback compares against.

## 8. Furniture — object illustrations, partial alpha, not textures

`f_chair_faceS` is 23% opaque: a chair drawn face-on on
transparency. `f_table_unconnected` is 93% opaque (a table drawn
near-top-down fills its tile), `f_bed_unconnected` 80%. Furniture
feeds the voxelizer's extrusion path (plan §8), and its sprites
must never be sampled as surface textures — a chair's "palette" is
upholstery + shadow, not a material. The classifier still labels
furniture ids (audits want full coverage) but Stage C+ recipes only
consume terrain/floor/wall families.

## 9. Mapgen and roof sprites — scene illustrations

There is **no `t_roof` / `t_shingle_roof` terrain entry in UltiCa
decomposed at all**. Roof-looking ids are overmap/mapgen scene
sprites: `s_apt_roof`'s fg is `s_apt` (32x64,
`pngs_tall_32x64/overmap/s_apt/s_apt.png`), a whole apartment
building illustration over an `open_air_tall` bg; the Stage B
analyzer gives the family a shading factor of **1.00** — it is a
shaded drawing end to end. Consequence: the `roof` material class
has no clean terrain exemplar in this tileset; Stage C's roof
recipe must work from palettes of these scenes' roof regions or
from `t_glass_roof`-style terrain, and the house sample scene's
roof slabs (fake-data materials) take their palette from the
recipe defaults. Flagged for the Stage C work order.

## 10. Where the baked shading lives — summary

| Form | Where | Measured example |
|---|---|---|
| Vertical gradient, either sign | wall fields | t_concrete_wall: L rises downward; t_wall_wood: center vignette |
| Trim/baseboard bands | wall top/bottom rows | t_wall: rgb(191,177,156) header vs rgb(209,202,188) field |
| Diagonal specular streaks | glass | t_window glass field rgb(180,201,208) + white streaks |
| Top-light gradient + handle highlight | doors | t_door_c composite, shading factor 0.28 |
| Per-plank value jitter | floors | t_floor bands 81→101→81 (x,57..72,x) |
| Transition fringes | ground/water edges | t_dirt grass-green edge bands; water shoreline ring |
| Whole-scene shading | mapgen/building sprites | s_apt family, shading factor 1.00 |
| Contact shadow | trees/furniture bases | t_tree_apple bg/shadow rows |

Stage B's `analyze.rs` reports the first two numerically per sprite
(vertical gradient + shading factor, highlight fraction); Stages
C–F are responsible for *removing* them from generated textures:
de-shading by the measured gradient (Stage F), center-region
sampling for ground/water (§2, §6), and generated end-caps for wall
tops (§4).
