# New tiles: the butterfly and moth lifecycles, the alien meadow

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
