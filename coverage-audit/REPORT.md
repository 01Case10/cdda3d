# Coverage audit — base-game ids vs the UltiCa tileset (2026-11-15)

Question (Joey): are there game tiles or enemies with no representation in
the tileset, and if so, create a tile for one from its description?

**Answer: yes, a lot of them.** Phase 5's "0 missing" was measured against
the tileset's own entry ids — a circular universe. Measured against the
base game's own ids (the open 5.10 TileDefs-form, run here against
`data/json`, CDDA pinned 6a3e59e0, sparse checkout expanded for
`data/json` only), UltiCa does **not** meet 5.10's ≥98% bar for terrain
or furniture, and monsters are the worst of the three.

Method: `tools/audit_missing_tiles.py`. Every game id is resolved through
the game's own chain (exact → variant → season → `looks_like`, inherited
through `copy-from`, jump limit 10 → filler), replicating
`crates/tileset/src/resolve.rs`. Filler = not represented (it renders as
the magenta/black checker placeholder). Full lists: `coverage.md` /
`missing.json` in this folder.

Two universes are reported, because they differ:

- **shipped** — what a player running UltiCa in CDDA actually gets:
  tile-entry JSON ids **plus** the stems of PNGs no entry references,
  because the official tileset build composes UltiCa with
  `compose.py --use-all` (CDDA `.github/workflows/compose-tilesets.yml`),
  and `--use-all` turns every *unreferenced* PNG into a tile entry
  `{id: <png stem>, fg: <sprite>}`. (First pass of this audit unioned
  *every* PNG stem; that overcounted — 7,166 stems are sprite names
  referenced by other entries and never become ids. Strict universe:
  12,925 ids = 7,531 JSON + 5,395 unreferenced stems. The correction
  moved only vehicle parts, 198→202 missing, and items, 3,770→3,778;
  terrain/furniture/monster numbers were unchanged.)
- **our loader** — JSON entry ids only: what `crates/tileset`'s decomposed
  loader saw before the fix below.

## Headline numbers

| category | game ids | missing (shipped) | missing (our loader) | covered only by an orphan PNG |
|---|---|---|---|---|
| terrain | 1,271 | 202 (173 player-visible*) | 288 | 86 |
| furniture | 713 | 174 (164 player-visible*) | 271 | 97 |
| monsters | 1,203 | 468 | 543 | 75 |
| vehicle parts | 483 | 202 | 236 | 34 |
| field types | 172 | 90 | 126 | 36 |
| items (ground sprite) | 10,270 | 3,778 | 7,417 | 3,639 |
| traps | 150 | 121 | 124 | 3 |
| overmap terrain | 484 | 339 | 343 | 4 |

\* player-visible excludes pseudo/internal ids that are never meant to
draw: `t_region_*`/`f_region_*` ("pseudo terrain/furniture — should never
actually show up"), abstract-style entries whose description says "If you
see this in-game, it's a bug", and spawnpoint specials (39 ids across
terrain+furniture).

Caveats, by category:

- **Traps** overstate: 56 of the 121 have no name at all — they are
  internal EOC/mapgen trigger traps (`tr_LIXA_*`, teleport triggers,
  corridor/light triggers), and many real ones are buried/invisible by
  design (`tr_beartrap_buried`). Traps are not a "draw everything" list.
- **Field types**: many `fd_*` ids are emitters/internal states, not
  displayed fields; and per Phase 2b our engine renders fields as shader
  effects parameterized from sprites, not as pasted tiles, so this row
  does not block rendering the way terrain does.
- **Overmap terrain**: the main UltiCa set is not the overmap tileset
  (overmap has dedicated sets), and the plan keeps the overmap 2D — this
  row is informational only.
- **Items** are Phase 6 (billboard) scope; the count is ground sprites
  only, not worn/wielded overlays.

## Two distinct problems

1. **UltiCa genuinely lacks tiles for current-master content.** The
   missing terrain clusters are the newer areas: alien meadow (16),
   string-dimension (18), nether labyrinth `t_nl_*` (17), vitrified (16),
   plus pane-glass window variants whose `looks_like` targets exist only
   as orphan PNGs. Furniture gaps include the whole adobe/brick/clay/
   gravel/log/pipe/plank/rebar/rock/sand/stick **stack-and-mound family**
   (short/mid/tall × ~11 materials), vitrified furniture, and sea flora
   (algae/kelp/anemone). Monster gaps are broad — 468 of 1,203 — including
   whole families: the lepidoptera below, amalgamation zombies, mega
   ants, pupating zeds, feral variants whose `looks_like` is the literal
   string `chud` (not a monster id — a dead end in the chain), and most
   nether creatures.

2. **Our loader was missing a compose.py behaviour — FIXED 2026-11-15.**
   258 terrain/furniture/monster ids are covered in the shipped game
   *only* because `--use-all` promotes an orphan PNG (referenced by no
   tile entry) into an entry: `mon_antlion_larva`, `mon_creeper_hub`,
   `t_window_no_curtains`, `t_ladder_up`, … `crates/tileset`'s
   `load_decomposed` now synthesizes the same entries
   (`synthesize_use_all_entries`: referenced set collected from fg/bg
   incl. `additional_tiles` at any depth; explicit JSON entries win;
   fallback sheet and non-`pngs_*` strays excluded), with a dedicated
   loader test (`use_all_synthesis`). Verified on real UltiCa: voxelizer
   run **12,206 materials / 39,856 layers / 14,450 sprites / 0 missing**,
   byte-identical re-run (was 7,717 materials before the fix; bake time
   6.2 s → 31 s on this VM); the example ids above all resolve `exact`.
   Our loader's coverage now equals the shipped column. Remaining gap is
   gap (1) only: tiles UltiCa genuinely does not have.

## Created: the butterfly lifecycle (in `new-tiles/`)

The entire butterfly line in `data/json/monsters/insect_spider.json` had
no tile in any form — no JSON entry, no orphan PNG, no `looks_like`:

- `mon_caterpillar_butterfly` — "about your size", light green, symbol C;
  upgrades into the cocoon.
- `mon_butterfly_cocoon` — 120 L, immobile; upgrades into the giant.
- `mon_butterfly` — "vibrant colored markings… a very peaceful
  nectivore", symbol l, white.
- `mon_butterfly_giant` — "larger than a human… feeds mainly on the
  nectar produced by triffid flowers", symbol L, white.
- `mon_butterfly_titan` — "enormous… regal looking wings… leaving dust
  as it passes", copy-from giant, symbol L, **yellow**.

Drawn from those descriptions in UltiCa decomposed layout (PNG + tile
entry JSON + credits per folder, CC BY-SA 3.0): monarch-pattern orange/
black for the vibrant line, gold wings with royal-purple borders and
shed dust sparkles for the regal titan, a segmented light-green
caterpillar, and a pale jade chrysalis with the monarch's gold dots.
`new-tiles/draw_butterflies.py` regenerates them; `new-tiles/preview.png`
is the contact sheet.

Verified with the project's own tooling: a test tileset containing these
folders (plus the two shadow sprites their entries reference) run through
the release voxelizer produces **5 materials, all resolution `exact`,
0 missing**.

The moth line was drawn the same day, on Joey's word — `mon_moth`
("typical wood-like motifs… as peaceful as the mutated butterflies",
light gray, nocturnal: bark-banded grey-brown wings, feathery antennae,
big night eyes), `mon_moth_giant` ("larger than a human… silently flies
with its huge wings after dusk"), `mon_caterpillar_moth` (the same
larva, but game color green rather than light green — deeper green,
rusty side spots), and `mon_moth_cocoon` (a fibrous pale-grey *silk*
cocoon, thread-wrapped — distinct from the butterfly's hard chrysalis).
Combined test-tileset run: **9 materials, all `exact`, 0 missing**.
`new-tiles/preview_moths.png` is their contact sheet; `preview.png`
now shows all nine.

## Status

Working tree only — not committed, not pushed (other sessions have
uncommitted 2d/2e/2f work in this repo). `fork-src/` is gitignored, so
expanding its sparse checkout for `data/json` does not dirty the repo.
Gate 5.10 stays open, now with a measured list instead of an unknown:
as written (≥98% of base-game terrain+furniture) it cannot pass on
current UltiCa — closing it means either re-scoping the gate or a
tile-creation/`looks_like`-entry effort against `missing.json`.
