# New tiles: the butterfly and moth lifecycles, the alien meadow, the string dimension, the regional templates

## TERRAIN COMPLETE — batches 8–10 (Phase 2g.4 wave 2, 2026-11-15)

**Terrain coverage is 100%: 0 missing of 1,271** in the
shipped-universe audit. Wave 2's three batches:

- **Batch 8 (20 ids)** — triffid bark walls/floors, root slopes,
  bramble doors; reverberation voids and the magenta exit portal;
  radiosphere tree/bush; thin/thick shallow/deep ice; the checker
  placeholder. `draw_triffid_ice.py`, `preview_triffid_ice.png`.
  Missing 64 → 44 (96.5%).
- **Batch 9 (26 ids)** — the scatter: beehive, grasses, gel
  groundcover, LIXA walls and shafts, nether basalt/glass, scarlet
  sea, water cubes, shore boulders, fungal pair, bone floor,
  portcullis, glowing floor, shuttered window.
  `draw_scatter_terrain.py`, `preview_scatter_terrain.png`.
  Missing 44 → 18 (98.6%).
- **Batch 10 (18 ids)** — missile pair, radio tower, nanofab pair,
  milking machine, lean-to, brick oven, boom barrier, intercom,
  elevator controls, reactor, and the five sunken-helicopter tiles.
  `draw_manufactured.py`, `preview_manufactured.png`.
  Missing 18 → **0 (100%)**.

All parent re-verified: voxelizer all-`exact`/0 missing,
byte-identical re-runs, opacity checks clean, previews inspected.
Furniture and monsters are the remaining drive.

## Furniture batch 15: the remainder — FURNITURE COMPLETE (Phase 2g.4, 2026-11-15)

**All 49 remaining missing furniture ids** — generators, the
nether-labyrinth set, mutant trees, medical, storage, the mortar,
the roulette table, and the `f_clear`/`f_no_item` debug tiles
drawn as black "? BUG" tiles per their nature. Drawn by
`draw_furniture_remainder.py` (PIL, deterministic).

Verified: release voxelizer test-tileset run, **49 materials, all
`exact`, 0 missing** (147 layers), byte-identical re-run, parent
re-verified; final full-overlay check: **furniture missing 0 of
713 — 100.0%**. `preview_furniture_remainder.png` is the contact
sheet. Gate 5.10's ≥98% terrain+furniture coverage criterion is
met on both halves (terrain 0 of 1,271 since batch 10).

## Furniture batch 14: shores — regional flora, sea flora, decor (Phase 2g.4, 2026-11-15)

**All 23 missing ids in `furniture-regional-pseudo.json`,
`furniture-flora_marine.json`, and `furniture-decorative.json`** —
the ten regional-pseudo placeholders drawn as the flora their
roles name, the seven marine pieces (algae, kelp, sea lettuce,
bladderwrack, anemone, sponge), and six decorative pieces
(shackle, gazing ball, lighthouse light, styling station, two
flagpoles). Drawn by `draw_furniture_shores.py` (PIL,
deterministic).

Verified: release voxelizer test-tileset run, **23 materials, all
`exact`, 0 missing** (69 layers), byte-identical re-run, furniture
folder count stable across the re-run (no sibling impact), parent
re-verified. `preview_furniture_shores.png` is the contact sheet.
Coverage: furniture missing **72 → 49**, coverage
**89.9% → 93.1%**.

## Furniture batch 13: the vitrified furniture (Phase 2g.4, 2026-11-15)

**All 32 missing ids in `furniture_vitrified.json`** — the whole
household in black glass (bed, bath, fridge, oven, washer, dryer,
bookcases, sofa, and the one matte cloth doll), palette matched to
the vitrified terrain batch. Drawn by `draw_vitrified_furniture.py`
(PIL, deterministic).

Verified: release voxelizer test-tileset run, **32 materials, all
`exact`, 0 missing** (96 layers), and a parent-run full-directory
snapshot diff (306 files, md5 before/after the script re-run)
came back **empty** — byte-identical output, zero sibling impact,
which also closes this batch's first-run folder-deletion incident
(the agent's `rm -rf` cleanup, owned in its report and removed).
`preview_vitrified_furniture.png` is the contact sheet. Coverage:
furniture missing **104 → 72**, coverage **85.4% → 89.9%**.

## Furniture batch 12: stacks, mounds, cardboard (Phase 2g.4, 2026-11-15)

**All 51 missing ids in `furniture-terrains.json`**: mounds (sand,
gravel, pebble, clay, glass, rock, rock-large) and stacks (log,
plank, brick, adobe, rebar, pipe, stick, long-stick) in short/mid/
tall heights from the game data's coverage values, via two
parameterized renderers with per-material textures — plus the six
cardboard pieces (fort, wall, floor, roof, door closed/open) drawn
individually. Drawn by `draw_furniture_terrains.py` (PIL,
deterministic).

Verified: release voxelizer test-tileset run, **51 materials, all
`exact`, 0 missing** (153 layers), byte-identical re-run, parent
re-verified. `preview_furniture_terrains.png` is the contact
sheet. Coverage: furniture missing **155 → 104**, coverage
**78.3% → 85.4%**.

## Furniture batch 11: the alien furniture (Phase 2g.4, 2026-11-15)

**19 ids** from `furniture-alien.json` +
`furniture_alien_meadow.json`: the alien scar, Exodii portal
enclosure and floor rings, locator antenna, crystalline flower,
the impaled pair and empty spike, string-dimension pylons (live
and inert), imploded housing, warped shelf and crate, dead Exodii
pair, three alien weeds. Drawn by `draw_alien_furniture.py` (PIL,
deterministic), palette anchored to the alien-meadow terrain.

Verified: release voxelizer test-tileset run, **19 materials, all
`exact`, 0 missing** (57 layers), byte-identical re-run, parent
re-verified. `preview_alien_furniture.png` is the contact sheet.
Coverage: furniture missing **174 → 155**, coverage **75.6% →
78.3%**.

## Terrain batch 7: the walls (Phase 2g.4, 2026-11-15)

**All 20 missing ids in `terrain-walls.json`**: the shipping-
container wall in 15 game colors (one parameterized corrugated
panel), the warped wall (courses that disagree, magenta mortar,
wrong-side shadows), reinforced lab glass (wire grid in a bolted
frame), the wasp-paper pair (brood vs hardened strata), and the
wooden sunshade pole (canopy + pole, mostly air by design). Drawn
by `draw_walls.py` (PIL, deterministic). **Zero-drive wave 1
complete.**

Verified: release voxelizer test-tileset run, **20 materials, all
`exact`, 0 missing** (60 layers), byte-identical re-run, parent
re-verified. `preview_walls.png` is the contact sheet. Coverage:
terrain missing **84 → 64**, coverage **93.4% → 95.0%** — the
remaining 64 are wave 2's three batches.

## Terrain batch 6: the vitrified terrain (Phase 2g.4, 2026-11-15)

**All 16 missing ids in `terrain_vitrified.json`** — black-glass
dirt, whorled glass floor/wall, octagonal glass tile, vitrified
sand, and the glass wall with murky shapes frozen inside (full-tile;
the translucency is the pane's own property), plus razor grass,
spiral bush, glass bars, column, curtains, barbed fence, both
stairs, and the smashed window frame over `bg: t_vitrified_floor`.
Near-black blue-teal glass, pale specular light. Drawn by
`draw_vitrified_terrain.py` (PIL, deterministic).

Verified: release voxelizer test-tileset run, **16 materials, all
`exact`, 0 missing** (48 layers), byte-identical re-run, parent
re-verified. `preview_vitrified_terrain.png` is the contact sheet.
Coverage: terrain missing **100 → 84**, coverage **92.1% → 93.4%**.

## Terrain batch 5: floors, roofs, z-level transitions (Phase 2g.4, 2026-11-15)

**22 ids** from `terrain-floors-indoor.json`, `terrain-roofs.json`,
and `terrain-zlevel-transitions.json`: brick floor, the warped-stone
floors in four glow variants, cratered broken metal floor, the rad
platform, greenhouse floors (seedlings vs bare tilled ridges),
warped and canvas roofs, four treetops (mixed, evergreen, dead,
giant red fern), four earth ramps with direction chevrons, and the
low-stairs pair. Drawn by `draw_floors_roofs.py` (PIL,
deterministic; all sprites fully opaque — the first run's
translucent-ink marks were caught and restructured through
alpha-composited overlays).

Verified: release voxelizer test-tileset run, **22 materials, all
`exact`, 0 missing** (66 layers), byte-identical re-run, parent
re-verified including the opacity check. `preview_floors_roofs.png`
is the contact sheet. Coverage: terrain missing **122 → 100**,
coverage **90.4% → 92.1%**.

## Terrain batch 4: the nether labyrinth (Phase 2g.4, 2026-11-15)

**13 ids**: the missing `t_nl_*` labyrinth terrain plus
`t_slope_to_bosky_labyrinth` — cratered glowing metal floor,
starfield chasm, scrap columns (the lit one composited over
`bg: t_nl_column`), the mechanism lever over the metal floor,
fluorescent cybergrass, overgrown welded wall, sparkling soil,
lattice suspended bridge, and the three mission obelisks (identical
in game data, drawn identically). The glowberry pair is covered by
`looks_like` to the alien-meadow shrubs, not by folders here. Drawn
by `draw_nether_labyrinth.py` (PIL, deterministic).

Verified: release voxelizer test-tileset run, **13 materials, all
`exact`, 0 missing** (39 layers), byte-identical re-run, parent
re-verified. `preview_nether_labyrinth.png` is the contact sheet.
Coverage: terrain missing **135 → 122**, coverage **89.4% → 90.4%**.

---

## Terrain batch 3: the regional-pseudo templates (Phase 2g.4, 2026-11-15)

The zero-drive's first batch: **all 29 missing ids in
`terrain-regional-pseudo.json`** — the `t_region_*` groundcover,
soil, tree, and shrub region templates, plus `t_highland_groundcover`
and the string-dimension / netherum-labyrinth / portal-storm
groundcovers. All are named "pseudo terrain" in game ("should never
actually show up"), and all are drawn anyway — zero is zero — as the
full-tile ground each template names: curated lawn, tilled furrow
soil, swamp algae with water glints, barren cracked dirt, ravine
scree, forest floors graded by density (evergreen needle litter and
cones, fallen fruit, wild berries, nut shells, deep shade, ash-gray
dead), shrub grounds, highland grit, the string dimension's black
specked ground, netherum rubble, portal-storm scorched ash. Drawn by
`draw_regional_pseudo.py` (PIL, deterministic); all entries are
full-tile (`bg: ""`).

Verified: release voxelizer test-tileset run, **29 materials, all
`exact`, 0 missing** (87 layers), byte-identical re-run, parent
re-verified. `preview_regional_pseudo.png` is the contact sheet;
`preview.png` now stacks this batch on top. Coverage: terrain
missing **164 → 135**, coverage **87.1% → 89.4%** — and the
non-player-visible terrain set is now empty.

---

## Terrain batch 2: the string dimension (Phase 2g.4, 2026-11-15)

The second terrain batch, same priority order: **the 20 missing
terrain ids in `data/json/furniture_and_terrain/terrain-string-dimension.json`**
— the audit's second named cluster (the file's remaining ids resolve
via `looks_like`; `t_string_dimension_groundcover` is a regional
pseudo and stays out). All 32×32 on the normal sheet under
`pngs_normal_32x32/terrain/<id>/`, drawn from the in-game descriptions
by `draw_string_dimension.py` (PIL, deterministic):

| id | drawn from |
|---|---|
| `t_string_void` / `t_string_true_void` | "an otherworldly nothing… a black wall at the front of your senses" — pure black, the two indistinguishable as described |
| `t_string_point_small` / `_big` | "A speck in the darkness." — white dot / white star speck on black (symbols `.` / `*`) |
| `t_string_point_small_gray` / `_big_gray` | the same specks in light gray |
| `t_string_x` | "impossibly thin magenta line… the space around it shimmers" — 1px horizontal magenta line, faint shimmer ghosts, over the void |
| `t_string_y` | the cyan line, vertical |
| `t_string_z` | the yellow line "that extends up and down" — vertical, bright caps at both ends |
| `t_string_xy` / `t_string_xy_plane_1` | "impossibly flat blue square" — flat blue, identical descriptions drawn identically |
| `t_string_xy_plane_2` | "shimmering blue square" — diagonal sheen bands + sparkles |
| `t_string_xy_plane_3` | "wavy square" — light blue with wavy bands |
| `t_string_xyz` | "white cube… it somehow reminds you of home" — isometric white cube over the void |
| `t_nl_sd_2d_sky` | "the sky… impossibly flattened into two dimensions" — near-black with the whole sky squashed into one pale band |
| `t_nl_sd_2d_sky_walkable` | "You can stand here, upon the horizon." — the band as a floor |
| `t_nl_sd_2d_ground` | "the ground… flattened… patterns constantly shifting, like an image passing through a veined rock" — light-gray veined marbling |
| `t_string_gates_mech_control` | "A heavy-duty metal lever with two settings, each marked by an alien symbol." — plate, angled arm, red knob, cyan/red glyph marks, over the void |
| `t_string_gates_mech_control_inert` | the same lever, "Toggling it back and forth does nothing" — dimmed, desaturated |
| `t_string_dimension_meteoric_rock_charred` | "solid rock, deeply pitted, charred, and coated in a residue with an acrid smell" — dark pitted rock, scorch blotches, olive residue streaks |

Verified: run through this project's release voxelizer as a test
tileset, all 20 produce materials with resolution `exact`, **0
missing** (60 layers, 20 sprites), byte-identical re-run.
`preview_string_dimension.png` is the batch contact sheet (each tile
composited over its bg, as the game shows it); `preview.png` now
stacks this batch above the alien meadow and the monsters.

Coverage: no `looks_like` rescues this time — terrain missing
(shipped) **184 → 164**, coverage **85.5% → 87.1%**, player-visible
missing **155 → 135** (see `../REPORT.md`).

---

## Terrain batch: the alien meadow (Phase 2g.4, 2026-11-15)

The first terrain batch, in the plan's priority order (player-visible
terrain first): **all 16 terrain ids in
`data/json/furniture_and_terrain/terrain_alien_meadow.json`** — the
audit's first named cluster — missing from UltiCa in every form (no
entry, no orphan PNG, no `looks_like`). All 32×32 on the normal sheet
under `pngs_normal_32x32/terrain/<id>/`, drawn from the in-game
descriptions by `draw_alien_meadow.py` (PIL, deterministic):

| id | drawn from |
|---|---|
| `t_alien_meadow_grass` | "light reddish color and the stalks are much broader… waxy residue" — full-tile light-red ground, broad blades, pale waxy streaks (flat: fg full tile, `bg: ""`) |
| `t_alien_meadow_grass_long` | "Dark red… almost waist-high, each stalk splits into two to four other stalks… ending in fine fern-like fronds" — recursively splitting stalks with fern fronds, fg over `t_alien_meadow_grass` |
| `t_alien_meadow_grass_tall` | "Head high and such a dark red as to be nearly black… burst of fine fronds… forming a canopy" — near-black stalks, tangled frond canopy |
| `t_alien_meadow_tree_trap_roots_long_grass` | game data: "Looks exactly the same so it's not obvious unless the player spots the trap" — entry-only, `fg` references the `t_alien_meadow_grass_long` sprite itself; no separate PNG, so the trap is indistinguishable by construction |
| `t_alien_meadow_tree_trap_roots_tall_grass` | same, over the tall-grass sprite |
| `t_alien_meadow_tree_trap_roots_obvious` | "A cluster of dark roots from an alien tree are visible here. Did one of them just move?" — snaking dark roots, one tip raised mid-motion |
| `t_alien_meadow_shrub_1` | "dark blue-green… leaves are all interwoven, forming a sphere except for spots where the leaves have been eaten away… glows faintly in the dark" — interwoven teal sphere, eaten holes, sparse dim glow points (`light_emitted: 5`) |
| `t_alien_meadow_shrub_2` | "very similar to a terrestrial bush despite the dark purple leaves… clusters of white berries" |
| `t_alien_meadow_shrub_2_harvested` | the same bush with the berries gone — bare berry stems, no white clusters |
| `t_alien_meadow_shrub_3` | "branches split and split again into very fine tendrils with bulbs on the end that float upward, holding its yellow leaves(?) up" |
| `t_alien_meadow_tree_1` | "a group of individual dark-colored trunks that intertwine… branches have nodes of dark red leaves interspersed" |
| `t_alien_meadow_tree_2` | "vaguely like a willow with yellow leaves… branches that touch the ground have put down roots and are forming secondary trunks" — drooping yellow branches, root flares where tips ground |
| `t_alien_meadow_tree_3` | "trunk lifted off the ground on a cluster of roots. Giant dark-red globes the size of watermelons hang from its branches amidst clusters of dark-red leaves" |
| `t_alien_meadow_tree_3_harvested` | the same root-lifted tree, globes harvested — "clusters of long, dark-red leaves" |
| `t_alien_meadow_tree_trap` | "a group of thick dark-colored trunks half-fused together. The drooping branches have nodes of dark red leaves" — the mature tree_1 form, drawn thicker/fused |
| `t_alien_meadow_pond_goo` | "A pool of some very thick, nearly-opaque liquid. The surface is almost perfectly smooth." — flat green pool, faint swirls under one smooth specular sheen |

Verified: run through this project's release voxelizer as a test
tileset, all 16 produce materials with resolution `exact`, **0 missing**
(48 layers, 14 sprites — the two shared-fg trap entries add no sprite),
byte-identical re-run. `preview_alien_meadow.png` is the batch contact
sheet (each tile composited over its bg ground, as the game shows it);
`preview.png` now shows all batches — terrain grid above the monsters.

Knock-on: covering `t_alien_meadow_shrub_2` / `_harvested` also rescues
`t_nl_shrub_glowberries` / `_harvested` (nether labyrinth), whose
`looks_like` points at them. With this batch overlaid on the audit
universe, terrain missing (shipped) goes **202 → 184** and terrain
coverage **84.1% → 85.5%** (see `../REPORT.md`).

---

# New tiles: the butterfly and moth lifecycles

Nine monsters in base CDDA (`data/json/monsters/insect_spider.json`,
pinned 6a3e59e0) had **no representation in UltiCa in any form** — no
tile-entry JSON, no orphan PNG for `compose.py --use-all` to promote, and
no `looks_like`. In the shipped game they fall through to the filler /
ASCII fallback; in our voxel frontend they would get the placeholder
material. See `../REPORT.md` for the full audit.

Created here, from the in-game descriptions, in UltiCa's decomposed
layout — drop the `pngs_*` folders into `gfx/UltimateCataclysm/` (or an
overlay tileset dir) and both CDDA's compose step and our voxelizer pick
them up unchanged:

| id | sheet | drawn from |
|---|---|---|
| `mon_butterfly` | `pngs_normal_32x32` (32×32) | "oversized butterfly with vibrant colored markings… a very peaceful nectivore" — monarch pattern, white border spots |
| `mon_butterfly_giant` | `pngs_large_64x64` (64×64) | "larger than a human… feeds mainly on the nectar produced by triffid flowers" — same monarch pattern, more vein/spot detail |
| `mon_butterfly_titan` | `pngs_giant_96x96` (96×96) | "enormous butterfly with regal looking wings… leaving dust as it passes", game color yellow — gold wings, royal-purple borders, dust sparkles. No `bg`: giant-sheet monsters in UltiCa conventionally carry no shadow entry |
| `mon_caterpillar_butterfly` | `pngs_large_64x64` (64×64) | "about your size", light green — segmented S-curve caterpillar, dorsal stripe, prolegs, head eyespots |
| `mon_butterfly_cocoon` | `pngs_large_64x64` (64×64) | "a caterpillar undergoing a metamorphosis", 120 L, immobile — pale jade chrysalis, segment ridges, monarch gold-dot row |

| id | sheet | drawn from |
|---|---|---|
| `mon_moth` | `pngs_normal_32x32` (32×32) | "oversized moth with its typical wood-like motifs… as peaceful as the mutated butterflies", light gray, nocturnal — bark-banded grey-brown wings, pale reniform spots, feathery antennae, fuzzy body, big night eyes |
| `mon_moth_giant` | `pngs_large_64x64` (64×64) | "mutated moth larger than a human. It silently flies with its huge wings after dusk looking for nocturnal flowers" — same wood pattern at large size |
| `mon_caterpillar_moth` | `pngs_large_64x64` (64×64) | same larva description as the butterfly's, but game color **green** (not light green) — deeper green, rusty side spots, cream lateral stripe |
| `mon_moth_cocoon` | `pngs_large_64x64` (64×64) | same cocoon description, light gray — a *silk* cocoon (fibrous pale-grey oval, thread wraps), not a chrysalis: moths spin silk, butterflies don't |

Regenerate or tweak: `python3 draw_butterflies.py` and
`python3 draw_moths.py` (PIL, deterministic). `preview.png` is a
combined contact sheet of all nine on grass green with a contact shadow;
`preview_moths.png` is the moth set alone (the sprites themselves bake
no shadow — the entries reference UltiCa's `shadow_*` bg sprites, as
other monster entries do).

Verified: run through this project's release voxelizer as a test
tileset, all nine produce materials with resolution `exact`, 0 missing.

License: CC BY-SA 3.0, matching the tileset these are made for.
