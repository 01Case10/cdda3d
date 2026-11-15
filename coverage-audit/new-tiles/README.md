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
