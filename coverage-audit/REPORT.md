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

## Created: the alien meadow (Phase 2g.4 terrain batch, 2026-11-15)

The first terrain batch under Phase 2g.4 (priority: player-visible
terrain first): **all 16 ids in `terrain_alien_meadow.json`** — the
first cluster named in §"Two distinct problems" above — drawn from
their in-game descriptions in the same decomposed layout under
`new-tiles/pngs_normal_32x32/terrain/<id>/` (32×32, CC BY-SA 3.0;
`new-tiles/draw_alien_meadow.py` regenerates them deterministically;
`new-tiles/preview_alien_meadow.png` is the contact sheet, and
`preview.png` now shows all batches). Design notes: flat ground
(`t_alien_meadow_grass`, `t_alien_meadow_pond_goo`) is full-tile fg with
`bg: ""`; plants/trees are transparent fg over `bg:
t_alien_meadow_grass` (the `t_shrub`-over-`t_grass` pattern); and the
two hidden trap-root grasses (`…_roots_long_grass`, `…_roots_tall_grass`)
are entry-only, their `fg` referencing the plain grass sprite itself —
the game data says they must look exactly the same, so they do by
construction.

Verified per the 2g.4 gate: a test tileset containing the batch run
through the release voxelizer produces **16 materials, all resolution
`exact`, 0 missing** (48 layers, 14 sprites), byte-identical on re-run.

Coverage movement: `tools/audit_missing_tiles.py` was re-run; its
outputs are byte-identical to the previous run, because the script
measures the UltiCa tree in `fork-src/` and `new-tiles/` is an overlay
that has not been dropped into it (the same reason `mon_butterfly`
still appears in `missing.json`). Measured as an overlay — batch ids
added to the shipped universe, `looks_like` chains re-resolved —
terrain missing (shipped) moves **202 → 184**: the 16 direct ids plus
`t_nl_shrub_glowberries` and `t_nl_shrub_glowberries_harvested`, whose
`looks_like` targets `t_alien_meadow_shrub_2` / `_harvested`. Terrain
coverage **84.1% → 85.5%** (1,069/1,271 → 1,087/1,271); player-visible
terrain missing **173 → 155**. The script's headline numbers will show
this only once the batch lands in an actual tileset tree.

## Created: the string dimension (Phase 2g.4 batch 2, 2026-11-15)

The second terrain batch, same priority order: **the 20 missing ids in
`terrain-string-dimension.json`** (the audit's second named cluster;
the file's other ids already resolve via `looks_like`, and
`t_string_dimension_groundcover` is a regional pseudo that never
actually shows up, so it stays out). Drawn from their in-game
descriptions in the same decomposed layout
(`new-tiles/draw_string_dimension.py` regenerates them
deterministically; `new-tiles/preview_string_dimension.png` is the
contact sheet, and `preview.png` now stacks this batch above the
alien meadow and the monsters). Design notes: the two voids are pure
black and indistinguishable, as described; the points are specks on
that black; the perceived geometry — impossibly thin magenta/cyan/
yellow lines, the flat blue planes (plain, shimmering, wavy), and the
white xyz cube — plus the rotary lever are transparent `fg` over
`bg: t_string_void`, things perceived against the nothing; the
flattened sky is the whole sky squashed into one pale band, the
walkable horizon is that band as a floor; the flattened ground is
light-gray veined rock; the charred meteoric rock is pitted,
scorch-blotched, and streaked with dull olive residue; the inert
lever is the working lever drawn dimmed and desaturated.

Verified per the 2g.4 gate: a test tileset containing the batch run
through the release voxelizer produces **20 materials, all resolution
`exact`, 0 missing** (60 layers, 20 sprites), byte-identical on
re-run.

Coverage movement (same overlay method as the alien meadow — the
computation first reproduces that batch's 184 and its two glowberry
rescues exactly): no `looks_like` rescues this time, so terrain
missing (shipped) moves **184 → 164**, terrain coverage **85.5% →
87.1%** (1,087/1,271 → 1,107/1,271), player-visible terrain missing
**155 → 135**. Remaining named terrain clusters: the nether
labyrinth `t_nl_*` (17) and the vitrified set (16).

## Created: the regional-pseudo templates (Phase 2g.4 batch 3, 2026-11-15)

The zero-drive (Joey: "drive the coverage until nothing is missing")
opened with the audit's least visible row: **all 29 missing ids in
`terrain-regional-pseudo.json`** — the `t_region_*` groundcover/soil/
tree/shrub templates, highland groundcover, and the string-dimension,
netherum-labyrinth, and portal-storm groundcovers. They are marked
"should never actually show up" in game, but zero is zero, and each
is drawn as the full-tile ground its template role names (curated
lawn vs tilled furrows, swamp algae, barren cracked dirt, ravine
scree, forest floors graded by density, shrub grounds), in the same
decomposed layout (`new-tiles/draw_regional_pseudo.py`;
`new-tiles/preview_regional_pseudo.png`).

Verified per the 2g.4 gate (parent re-run): **29 materials, all
resolution `exact`, 0 missing** (87 layers, 29 sprites),
byte-identical re-run. Coverage movement (overlay method, baseline
164 reproduced first): 29 direct, no rescues — terrain missing
**164 → 135**, coverage **87.1% → 89.4%** (1,107/1,271 →
1,136/1,271). Side effect worth noting: the non-player-visible
terrain set is now empty — every one of the remaining 135 missing
terrain ids is player-visible.

## Created: the nether labyrinth (Phase 2g.4 batch 4, 2026-11-15)

**13 ids**: the missing `t_nl_*` labyrinth terrain (cratered metal
floor with glowing cracks, starfield chasm, scrap columns — one
carrying a light, over `bg: t_nl_column` — a red-knobbed mechanism
lever over the metal floor, fluorescent cybergrass, an overgrown
welded wall, sparkling soil, a diamond-lattice suspended bridge)
plus `t_slope_to_bosky_labyrinth`, and the three mission obelisks,
which the game data gives identical name/description/symbol/color
("Metal stele") and are therefore drawn identically.
`new-tiles/draw_nether_labyrinth.py`;
`new-tiles/preview_nether_labyrinth.png`. (The glowberry pair is
*not* in this batch: it has no folders of its own and is covered by
its `looks_like` resolution to the alien-meadow shrubs, counted
since batch 1.)

Verified per the 2g.4 gate (parent re-run): **13 materials, all
`exact`, 0 missing** (39 layers), byte-identical re-run. Coverage
movement (overlay method, baseline 135 reproduced first): 13
direct, no rescues — terrain missing **135 → 122**, coverage
**89.4% → 90.4%** (1,136/1,271 → 1,149/1,271).

## Created: floors, roofs, z-level transitions (Phase 2g.4 batch 5, 2026-11-15)

**22 ids** across three files: indoor floors (brick courses, the
warped-stone family in four glow variants, cratered broken metal,
the rad platform's bolted ring, greenhouse seedling rows vs bare
tilled ridges), roofs (warped, stitched blue canvas, and four
treetops — mixed canopy, evergreen, dead limbs, giant red fern),
and z-level transitions (four earth ramps with direction chevrons,
the low stairs begin/end pair). `new-tiles/draw_floors_roofs.py`;
`new-tiles/preview_floors_roofs.png`.

One process note: the batch's first script run produced
semi-transparent pixels in 9 sprites (this Pillow build's
`ImageDraw` replaces rather than blends); the agent caught it,
restructured those marks through alpha-composited overlays, and the
parent re-check confirms **all 22 sprites are fully opaque**.
Verified per the 2g.4 gate (parent re-run): **22 materials, all
`exact`, 0 missing** (66 layers), byte-identical re-run. Coverage
movement (overlay method, baseline 122 reproduced first): 22
direct, no rescues — terrain missing **122 → 100**, coverage
**90.4% → 92.1%** (1,149/1,271 → 1,171/1,271).

## Created: the vitrified terrain (Phase 2g.4 batch 6, 2026-11-15)

**All 16 missing ids in `terrain_vitrified.json`** — a landscape
fused to glass: pebbled black-glass dirt, wood-whorl glass floor
and wall, octagonal glass tile, dusty vitrified sand, and the glass
wall with murky shapes frozen inside it (drawn full-tile: the
translucency is a property of the pane itself), plus the feature
set over `bg: t_vitrified_floor` — razor grass, spiral bush, thick
glass bars, faceted column, draped-glass curtains, barbed coil
fence, both stairs, and the smashed window frame. Palette:
near-black blue-teal glass with pale specular light.
`new-tiles/draw_vitrified_terrain.py`;
`new-tiles/preview_vitrified_terrain.png`.

Verified per the 2g.4 gate (parent re-run): **16 materials, all
`exact`, 0 missing** (48 layers), byte-identical re-run. Coverage
movement (overlay method, sequential chain from batch 5's 100):
16 direct, no rescues — terrain missing **100 → 84**, coverage
**92.1% → 93.4%** (1,171/1,271 → 1,187/1,271).

## Created: the wall terrain (Phase 2g.4 batch 7, 2026-11-15)

**All 20 missing ids in `terrain-walls.json`**: the shipping-
container wall in 15 game colors (one parameterized corrugated-
steel panel — vertical ribs, rails, rivets, faint weathering),
`t_wall_warped` (stone courses that visibly disagree, magenta
seeping through the mortar, shadows on the wrong side, per its
description), `t_reinforced_glass_lab` (thick pane, embedded wire
grid, bolted frame), the wasp-paper pair `t_paper_brood` /
`t_paper_hard` (layered pulp strata, gray vs dark-compressed), and
`t_wood_sunshade_pole` (canvas canopy on a grained pole — the
sprite is mostly air by design). `new-tiles/draw_walls.py`;
`new-tiles/preview_walls.png`. **Wave 1 of the zero-drive is
complete with this batch.**

Verified per the 2g.4 gate (parent re-run): **20 materials, all
`exact`, 0 missing** (60 layers), byte-identical re-run. Coverage
movement (overlay method, baseline 84 reproduced first): 20
direct, no rescues — terrain missing **84 → 64**, coverage
**93.4% → 95.0%** (1,187/1,271 → 1,207/1,271). The remaining 64
are exactly wave 2's scope: manufactured/mechanisms/sunken-
helicopter (18), triffid/reverberations/ice (20), and the 26-id
scatter.

## TERRAIN COMPLETE: 0 missing (Phase 2g.4 batches 8–10, 2026-11-15)

Wave 2 closed the category. **Terrain: 0 missing of 1,271 —
100.0% coverage** in the shipped-universe audit (overlay method,
full chain re-resolved against every folder in `new-tiles/`).

- **Batch 8 — triffid, reverberations, ice (20 ids).** Bark walls
  and floors, root-chevrons slopes, bramble doors (closed weave vs
  gaping open), the inner-cabins void pools and the magenta exit
  portal over `bg: t_inner_cabins_void`, radiosphere tree and bush,
  thin/thick ice over shallow and deep water, and the two-tone
  checker `t_pseudo_phase`. `draw_triffid_ice.py`. Overlay
  (baseline 64 reproduced): 64 → **44**, 95.0% → 96.5%.
- **Batch 9 — the scatter (26 ids).** Everything left: natural
  beehive, grazed and holy grass, gelatinous groundcover, the LIXA
  folded/unfolded walls and infinite shafts, nether basalt and
  black glass, scarlet-sea water and scaffolding, water cubes,
  shore boulders (one oyster-crusted), the Isherwood spawn dirt,
  fungal colony and transformed fungal wall, bone floor,
  portcullis, the radiosphere glowing floor, shuttered window.
  `draw_scatter_terrain.py`. Overlay: 44 → **18**, 96.5% → 98.6%.
- **Batch 10 — manufactured, mechanisms, sunken helicopter
  (18 ids).** The missile and its exploded twin, the radiosphere
  radio tower with its spotlight, nanofab body (whole and broken),
  milking machine, lean-to, brick oven base, boom barrier, the
  Balthazar intercom, elevator controls (lit and powerless), the
  nuclear reactor, and the five waterlogged helicopter tiles (the
  rotor composited over `bg: t_sunkenhelicopter_floor`).
  `draw_manufactured.py`. Overlay: 18 → **0**.

All three batches verified per the 2g.4 gate (parent re-run):
voxelizer all-`exact`, 0 missing (60/78/54 layers), byte-identical
re-runs, opacity checks clean (no semi-transparent pixels outside
the by-design transparent feature sprites), previews inspected.

The drive continues with furniture (174 missing at the audit) and
monsters (468).

## Created: the alien furniture (Phase 2g.4 batch 11, 2026-11-15)

The furniture drive opens: **19 ids** from `furniture-alien.json`
and `furniture_alien_meadow.json` — the gas-belching alien scar,
the Exodii portal enclosure and both floor portal rings (verbatim-
identical game entries, one drawing), the locator antenna, the
crystalline flower, the impaled officer and soldier over the
glowing spike (and the empty spike), the string-dimension pylons
(humming vs inert), the imploded housing, the warped shelf and
crate, the dead Exodii (whole and heartless), and the three alien
weeds. Palette anchored to the alien-meadow terrain batch.
`new-tiles/draw_alien_furniture.py`;
`new-tiles/preview_alien_furniture.png`.

Verified per the 2g.4 gate (parent re-run): **19 materials, all
`exact`, 0 missing** (57 layers), byte-identical re-run. Coverage
movement (furniture overlay, baseline 174 reproduced): 19 direct,
no rescues — furniture missing **174 → 155**, coverage **75.6% →
78.3%** (539/713 → 558/713). (A folder-based cross-check already
reads 104 because the stack/mound batch's 51 folders are on disk
awaiting their own verification; the chain reconciles when it
lands.)

## Created: the stack/mound family + cardboard (Phase 2g.4 batch 12, 2026-11-15)

**All 51 missing ids in `furniture-terrains.json`**: the
stack/mound family — mounds (sand, gravel, pebble, clay, glass,
rock, rock-large) and stacks (log, plank, brick, adobe, rebar,
pipe, stick, long-stick), each in short/mid/tall heights taken
from the game data's coverage values (short tops at the knees,
mid at the shoulders, tall well above the head), drawn by two
parameterized renderers with per-material textures — plus the six
cardboard pieces (fort, wall, floor, roof, and the cut-and-fold
door, closed and open) drawn individually.
`new-tiles/draw_furniture_terrains.py`;
`new-tiles/preview_furniture_terrains.png`.

Verified per the 2g.4 gate (parent re-run): **51 materials, all
`exact`, 0 missing** (153 layers), byte-identical re-run.
Coverage movement (furniture overlay, baseline 155 reproduced —
the post-batch-11 state): 51 direct, no rescues — furniture
missing **155 → 104**, coverage **78.3% → 85.4%** (558/713 →
609/713). This is exactly the figure the batch-11 cross-check
predicted, so the chain is fully reconciled.

## Created: the vitrified furniture (Phase 2g.4 batch 13, 2026-11-15)

**All 32 missing ids in `furniture_vitrified.json`** — the whole
household in black glass, matching batch 6's terrain palette:
bed, bathtub (frozen onyx column still pouring from the tap),
dresser, open fridge, armchair, chair, desk, locker, oven crowned
with flame shards, mailbox, rack, ripple-frozen sink, toilet,
cupboard, planter, bench, washer and dryer (portholes filled with
black sand), both bookcases (the talker's missing book is a dark
slot), counter, table, birdbath, shower, water heater, fireplace,
firering, floor lamp, box, sofa, dishwasher — and the set's one
non-glass object, the matte cloth doll lying on the glass sofa.
`new-tiles/draw_vitrified_furniture.py`;
`new-tiles/preview_vitrified_furniture.png`.

Verified per the 2g.4 gate (parent re-run): **32 materials, all
`exact`, 0 missing** (96 layers); the parent's full-directory
snapshot test (all 306 files under the furniture tree, md5 before
and after re-running the batch's script) diffs **empty** — the
output is byte-identical and the re-run touches nothing else, which
also closes the folder-deletion incident from this batch's first
run (an `rm -rf` cleanup by the agent, owned in its report; the
step is removed and the script contains no deletion calls).
Coverage movement (furniture overlay, baseline 104 reproduced):
32 direct, no rescues — furniture missing **104 → 72**, coverage
**85.4% → 89.9%** (609/713 → 641/713). The remaining 72 are the
two wave-2 furniture batches now running.

## Created: shores — regional flora, sea flora, decor (Phase 2g.4 batch 14, 2026-11-15)

**All 23 missing ids in `furniture-regional-pseudo.json`,
`furniture-flora_marine.json`, and `furniture-decorative.json`**:
the ten regional-pseudo placeholders (drawn anyway — zero is
zero — each as the flora its regional role names: wildflower
patch, scraggly weed, lily pads and reeds, fern undergrowth,
cattails, cultivated flower bed, submerged ribbon weeds); the
seven marine pieces (red and green algae, sugar kelp, sea
lettuce, bladderwrack with its paired air bladders, sea anemone,
sea sponge); and the six decorative pieces (wall shackle with
sagging chains, gazing ball on its cement pedestal, lighthouse
light, styling station, both indoor flagpoles).
`new-tiles/draw_furniture_shores.py`;
`new-tiles/preview_furniture_shores.png`.

Verified per the 2g.4 gate (parent re-run): **23 materials, all
`exact`, 0 missing** (69 layers), byte-identical re-run, and the
furniture folder count held at 125 across the re-run — no sibling
impact. Coverage movement (furniture overlay, baseline 72
reproduced): 23 direct, no rescues — furniture missing
**72 → 49**, coverage **89.9% → 93.1%** (641/713 → 664/713). The
remaining 49 ids are enumerated in the audit working notes; they
are the furniture-remainder batch's set, now running.

## Created: the furniture remainder — FURNITURE COMPLETE (Phase 2g.4 batch 15, 2026-11-15)

**All 49 remaining missing furniture ids**: the four generators
(one body plan from their shared copy-from, four paints — the
military one in camo), the nether-labyrinth set (autodoc and its
couch, both boilers, climbing vine, pipe plant, both powerboxes —
the open one shows the fleshy wired interior), the mutant tree
line (sprout, sapling, adult), medical pair, torah ark, military
crates, butter churn, hobo stove, solar cooker, brambles, midden,
the M224 mortar, roulette table, barber chair, the down beds —
and the two debug placeholders `f_clear` / `f_no_item`, drawn as
black tiles with a magenta "? BUG" mark, matching their
"seeing this is a bug" nature. `new-tiles/draw_furniture_remainder.py`;
`new-tiles/preview_furniture_remainder.png`.

Verified per the 2g.4 gate (parent re-run): **49 materials, all
`exact`, 0 missing** (147 layers), byte-identical re-run, folder
count stable at 174. The final check is a full overlay
re-resolution of every furniture id against every new-tiles
folder (not chain arithmetic): **furniture missing 0 of 713 —
100.0%**. With terrain already at 0 of 1,271, gate 5.10's ≥98%
terrain+furniture coverage criterion is now met on both halves;
the gate's TileDefs-form re-run remains blocked on Phase 0 tests
0.1/0.2 like every in-game gate. Monsters (468) are the drive's
last category.

## Created: the fungal zombies — monsters begin (Phase 2g.4 batch 16, 2026-11-15)

**All 50 missing ids in the fungus-zombie file**: every dusted /
fungalize / full-bloom variant of the zombie line (standard,
smoker, skeleton, child, runner, rot, tough, crawler, fat,
acidic, beekeeper), the prone wretch with its chest burst open
like a gory flower, the hand-winged spawn raptor, gasbags and
boomers, brutes and hulks, the fungal mounds and constructs, and
the zougar and ziger. Treatments follow the descriptions:
*dusted* is a gray powder wash composited over the finished body,
*fungalize* is one stalk bursting from the skull, *fungus/fungal*
is full bloom. Sheet placement follows the butterfly precedent by
resolved **volume** (the entries carry no explicit size field):
standard 62.5 L humanoids at 32×32, bulky 100–150 L at 64×64,
875 L hulks and massive mounds at 96×96 — this volume rule is the
standing placement rule for the monster batches.
`new-tiles/draw_fungus_zombies.py`;
`new-tiles/preview_fungus_zombies.png`.

Verified per the 2g.4 gate (parent re-run): **50 materials, all
`exact`, 0 missing** (150 layers); all 59 monster PNGs on disk
(this batch plus the butterfly/moth set) byte-identical across
the re-run. Coverage movement (monster overlay; baseline 454 at
verification time because 14 fish-batch folders had already
landed): 50 direct, no rescues — monsters missing **454 → 404**,
coverage **66.4%** (799/1,203).

## Created: fish, first half (Phase 2g.4 batch 17, 2026-11-15)

**52 of the 104 missing ids in `fish.json`** (alphabetical first
half, `mon_crayfish_giant_larva` … `mon_fry_blackfish`): the two
crayfish, flatfish with both eyes on the eyed side, catfish with
barbels, sturgeon with scute rows, the gar's needle jaw, chain
pickerel reticulation, the haddock's black shoulder blotch, the
rainbow trout's pink band, and the tiny pale fry with its yolk
sac. `new-tiles/draw_fish_a.py`; `new-tiles/preview_fish_a.png`.

Verified per the 2g.4 gate (parent re-run): **52 materials, all
`exact`, 0 missing** (156 layers), byte-identical re-run.
Coverage movement (monster overlay, baseline 404 reproduced):
52 direct + 1 rescue (`mon_nether_fish`, whose `looks_like`
chain reaches a drawn fish) — monsters missing **404 → 351**,
coverage **70.8%** (852/1,203). Placement note: all 52 sit on the
32×32 sheet per the brief's letter (no entry carries an explicit
size field); a parent volume census finds exactly three at
≥75 L resolved volume — `mon_crayfish_mega` (625 L),
`mon_fish_bluefin_tuna` (250 L), `mon_fish_halibut` (75 L).
Re-sheeting those three is a placement-only change (coverage is
unaffected); the fish second half is briefed with volume-based
placement from the start.

## Created: fish, second half — fish complete (Phase 2g.4 batch 18, 2026-11-15)

**The remaining 52 missing ids in `fish.json`**: 48 of them are
fry — every one copy-froms the `mon_fry` abstract ("minuscule
half-translucent fish larva, impossible to identify which
species"), so they share the first half's larva design with small
seeded jitter in tint, yolk, and proportions: the same
unidentifiable creature, never a species portrait. Plus the
crayfish zoea larva, the oversized mutant salmon fry, and the two
mutant salmon — `mon_mutant_salmon_huge` (312 L) and
`mon_mutant_salmon_mega` (exactly 400 L, with its two
flesh-colored water-jet tubes beneath the fins), both drawn
natively at 64×64 under the volume placement rule (only *over*
400 L goes to the giant sheet; the mega's boundary placement is
flagged as a placement-only question). `fish.json` is now fully
covered: 104/104 across batches 17–18.
`new-tiles/draw_fish_b.py`; `new-tiles/preview_fish_b.png`.

Verified per the 2g.4 gate (parent re-run): **52 materials, all
`exact`, 0 missing** (156 layers); every monster PNG on disk
byte-identical across the re-run; sampled sprites carry alpha
values only {0, 255}. Coverage movement (monster overlay,
baseline 351 reproduced): 52 direct, no rescues — monsters
missing **351 → 299**, coverage **75.1%** (904/1,203).

## Created: reptiles & amphibians (Phase 2g.4 batch 19, 2026-11-15)

**All 56 missing ids in `reptile_amphibian.json`**: garter and
worm snakes (pink belly showing), rattlesnakes from the baby's
tail button to the 620 L mega deathrattle at 96×96 (club tail
with bone segments, red muscle ropes flanking the maw), the
five-lined skinks (the baby's tail is blue), turtle hatchlings,
eleven tadpoles and six giant tadpoles (tail fins as composited
overlays — the gray tadpole's red-orange black-dotted tip, the
shift bigtad's crimson-to-yellow gradient), and the whole
frog/toad radiation: leopard spots with halos, the peeper's X,
the palette frog's bright square patches, pattern frogs with
static markings and retained gills, the tusked foul giganfrog,
and the 650 L megafrogs at 96×96.
`new-tiles/draw_reptiles.py`; `new-tiles/preview_reptiles.png`.

Verified per the 2g.4 gate (parent re-run): **56 materials, all
`exact`, 0 missing** (168 layers); every monster PNG on disk
byte-identical across the re-run. One environment note: the
workspace's root `Cargo.toml` currently lists three crate
members another session is scaffolding (vehicles / transient /
hardcases) whose directories are not all on disk yet, so
`cargo run` cannot load the workspace; verification used the
prebuilt `target/release/voxelizer` binary (built today from this
tree) with the standard CLI — the same tool, unaffected by
workspace membership. Coverage movement (monster overlay,
baseline 299 reproduced): 56 direct, no rescues — monsters
missing **299 → 243**, coverage **79.8%** (960/1,203).

## Created: insects & spiders (Phase 2g.4 batch 20, 2026-11-15)

**All 26 missing ids in `insect_spider.json`** (the file's other
missing ids were the already-drawn butterflies and moths): the
625 L mega acid ant and mega fly at 96×96, the graboid halfworm,
cicada with alpha-composited rooflike wings, giant mantis, the
lunar moth titan with glittering dust, the cellar spider mother
clutching a ball of ~46 individually drawn spiderlings, the
dermatik's deer host (emaciated, ribs showing, terrified eye —
the monster is the host body), and the larva/nymph/small-spider
tail down to 0.8 L. `new-tiles/draw_insects.py`;
`new-tiles/preview_insects.png`.

Verified per the 2g.4 gate (parent re-run): **26 materials, all
`exact`, 0 missing** (78 layers); every monster PNG on disk
byte-identical across the re-run. Coverage movement (monster
overlay, baseline 243 reproduced): 26 direct, no rescues —
monsters missing **243 → 217**, coverage **82.0%** (986/1,203).

## Created: the nether creatures (Phase 2g.4 batch 21, 2026-11-15)

**All 52 missing ids in `nether.json` (37) and
`netherum/labyrinth_monsters.json` (15)**: the dish-head's
fang-ringed dish face, the halo tangle's golden rings around its
dark flame, the roper's worm-mass over its maw, star vampires
with maw-tipped tentacles, the void spider family (all inheriting
1,800 L through copy-from, so the spiderlings sit on the giant
sheet drawn juvenile-scaled within the frame), the wells machine,
the LS robot arm, and the reflections — each wrong in its own
way: pale and wispy, twisted with one too-long arm, the blank
copy's too-even smile. `new-tiles/draw_nether_monsters.py`;
`new-tiles/preview_nether_monsters.png`.

Verified per the 2g.4 gate (parent re-run): **52 materials, all
`exact`, 0 missing** (156 layers); every monster PNG on disk
byte-identical across the re-run. Coverage movement (monster
overlay, baseline 217 reproduced): 51 newly covered + 1 upgrade
(`mon_nether_fish`, already rescued in batch 17 via its
`looks_like` to the brown trout, now has its own sprite) —
monsters missing **217 → 166**, coverage **86.2%** (1,037/1,203).

## Created: feral humans & the zed clusters (Phase 2g.4 batch 22, 2026-11-15)

**All 40 missing ids in the feral-humans, zed_misc,
zed_amphibian, and zed_amalgamation files (14 + 7 + 12 + 7)**:
the ferals are living-skinned with the descriptions' shared
tells — blackened bulging vein webs, bloodshot wide eyes,
hunched twitching posture — differentiated by uniform and prop
(sailors with wrench, crash axe, mop, lug wrench; the Marine's
gleaming bayonet; the fungal-infected cop's sprouting vest; the
six serum users in orange jumpsuit remnants, from Locutus's
healthy glow to Anaximander's shadow-blend). The amalgamations
are drawn to each entry's own creature plan, not as fused
humanoids: the spotter's twin head-sized compound eyes on waving
limbs, the corroder's caustic proboscis, the zapper crab's
glowing organ, the soldier's vertical mouth. The zed frogs and
toads share a parameterized renderer tuned per description
(bone death-mask and beak, mummy wraps, smoke haze, shadow
wreath). `new-tiles/draw_ferals_zeds.py`;
`new-tiles/preview_ferals_zeds.png`.

Verified per the 2g.4 gate (parent re-run): **40 materials, all
`exact`, 0 missing** (120 layers); every monster PNG on disk
byte-identical across the re-run. Coverage movement (monster
overlay; baseline 124 at verification time because 42
mammals/birds folders had already landed): 40 direct, no
rescues — monsters missing **124 → 84**, coverage **93.0%**
(1,119/1,203).

## Created: mammals, birds & mutant mammals (Phase 2g.4 batch 23, 2026-11-15)

**All 46 missing ids across `mammal.json` (17), `bird.json`
(16), and `mutant_mammal.json` (13)**: the chickadee's black cap
and bib, the waxwing's swept crest and red waxy wing dot, the
guinea fowl's bony helmet casque; thirteen chicks whose palettes
resolve through the generic chick abstracts with species hints
(coot chick's orange-red head patch, grebe chick's striped head,
the yellow duckling's flat bill); the black cats' prominent
green eyes, the mole's huge pink digging paws, the porcupine's
banded white-tipped quills, the skunk's dorsal stripe; the three
corpse placeholders (`mon_human`, `mon_child`,
`mon_human_scorched`) drawn as plain prone figures in pallor and
charred tones, no gore, matching their placeholder data nature;
and the mutant mammals, deformity as silhouette — the
wyrmhound's furless serpentine head and forked tongue, the
mauler's widened tooth-crowded jaw, the thrasher's oversized
digging paws, the raccoon mutant standing on hindlimbs with long
fingered paws. All 46 resolved under 100 L through their
copy-from chains, so all sit on the 32×32 sheet.
`new-tiles/draw_mammals_birds.py`;
`new-tiles/preview_mammals_birds.png`.

Verified per the 2g.4 gate (parent re-run): **46 materials, all
`exact`, 0 missing** (138 layers); every monster PNG on disk
byte-identical across the re-run (383 files). Coverage movement
(monster overlay, folders-minus-batch baseline 126 reproduced):
42 newly covered + 4 upgraded from `looks_like` rescue to their
own sprites — monsters missing **84 → 84** (the headline holds
because 42 of this batch's folders had already landed inside
batch 22's verification window and were counted in its
baseline), coverage **93.0%** (1,119/1,203). Remaining: 84 ids,
enumerated exactly — the finisher wave.

## Created: the cosmic & misc tail (Phase 2g.4 batch 25, 2026-11-15)

**42 of the 84 enumerated remaining ids** (the finisher wave's
second half): the seven yrax as gold faceted geometric forms —
apeirogon disc, delta, triakis, trifacet, the quadraphract and
its leg, the sphenocorona — plus the golden monolith on the same
inherited 930 L frame; the devourers (daitya with clone-heads
straining out of the flesh mass, the hekatonkheire, lab_sec's
pitch-black central figure, the melded task force); kraken guard
and queen (950 L, beak over foam); the stalker octopus and its
spawn; the ocean zhark; three snails up to the colossal 875 L
shell; both solar reverberation beetles; biollante sprig and
sprout; the three linelanders as bright vibrating filaments with
oscillating echoes (yellow drawn yellow — its data color says
`red` but the entry's name and its cyan/magenta siblings say
otherwise); the singularity Reclaimer (950 L) and its spawn; the
two TALON secubots with sensor arrays (the Liberty Edition's
flag decal) and Balthazar's rifle turret; and the singles —
robofac hound, radiosphere flailer, cabin tunnel shadow,
interstice horror, black ratkin (drawn as the plain black rat
its description specifies), sewer blob, camouflaged mutant, the
fungal wasp queen, the whisper fungus, and `mon_poly_special_test`
drawn as the ancient red dragon its entry describes, with a
small checkerboard calibration patch on one wing as the
test-artifact nod. Placement by resolved volume: 19 at 32×32,
13 at 64×64, 10 at 96×96. `new-tiles/draw_finisher_cosmic.py`;
`new-tiles/preview_finisher_cosmic.png`.

Verified per the 2g.4 gate (parent re-run): **42 materials, all
`exact`, 0 missing** (126 layers); every monster PNG on disk
byte-identical across the re-run (425 files). Coverage movement
(monster overlay, baseline 84 reproduced): 42 direct, no
rescues — monsters missing **84 → 42**, coverage **96.5%**
(1,161/1,203). The remaining 42 are exactly batch 24's
assignment (undead & lab tail), whose dispatch ended empty and
is re-run on Joey's word.

## Created: the undead & lab tail (Phase 2g.4 batch 24, 2026-11-15)

**The final 42 ids**: four skeletons (the executioner's
sword-blade forearms and fused-jaw mask, the reaper's scythe
arms with black goo dripping from its sockets, necro, slasher);
the four zombie horses at 550 L (the smoldering black stallion's
smoke pillars, the pale horse's green bile reservoir showing
through its rotund belly, red, white); the scorched/rust/
fireproof zed line; the two no-weakpoints zeds drawn as straight
soldier/survivor zombies (their descriptions are testing notes);
the grue's swollen gray eyes and stark white slack maw in a
shadow body (435 L); the string-dimension zeds and zomborgs; six
weird dogs (the antimemetic dog's static-smudge head, the
triclopean dog's lidded third eye, the incorporeal dog's pale
ghost palette, the scorched hulk at 440 L); the liquid and void
cats; three lab ghouls; the brute and hulk pupae with their
decoys deliberately near-identical to their twins (their
in-game function), the hulk pupae at 875 L; and the two
`pseudo_*` test monsters as readable range targets (bullseye,
bullet holes, marked glowing weak points). Placement by resolved
volume: 24 at 32×32, 10 at 64×64, 8 at 96×96.
`new-tiles/draw_finisher_undead.py`;
`new-tiles/preview_finisher_undead.png`. (Batch 24's first
dispatch ended empty — no files, no report; the batch was
re-dispatched on Joey's word and completed in full.)

Verified per the 2g.4 gate (parent re-run): **42 materials, all
`exact`, 0 missing** (126 layers); every monster PNG on disk
byte-identical across the re-run (467 files). Coverage movement
(monster overlay, baseline 42 reproduced): 42 direct, no
rescues — monsters missing **42 → 0**, coverage **100.0%**
(1,203/1,203).

## Zero-drive complete (2026-11-15)

Final audit re-run, same folder-based full-chain overlay method
as every batch gate (shipped universe ∪ all drawn folders,
effective `looks_like` through copy-from chains, 10-hop rescue):

- **Terrain: 0 missing of 1,271 — 100.0%**
- **Furniture: 0 missing of 713 — 100.0%**
- **Monsters: 0 missing of 1,203 — 100.0%**

Gate 5.10's coverage criterion (≥98% of base-game
terrain+furniture) is met at 100% on both halves, with monsters
also at 100%. The gate's TileDefs-form re-run (measuring through
the fork's in-game exporter) remains blocked on Phase 0 tests
0.1/0.2, as do all in-game gates. Out of scope throughout, per
the audit's own caveats: items (3,778; Phase 6 ground-sprite
scope), field types (shader effects, Phase 2b), traps (mostly
internal triggers), overmap (own tilesets), vehicle parts
(Phase 8).

## Scope extension: all tiles means all tiles (2026-11-15)

On Joey's word the drive reopens for the five categories above.
Re-resolved with audit semantics (drawn folders in the
universe), the exact remaining work is: **items 3,742**,
**overmap terrain 336**, **vehicle parts 195**, **traps 121**,
**field types 90** — 4,484 ids (46 already rescued by chains
through the first drive's folders). Same method, same gates,
sprites native 32×32 under `pngs_normal_32x32/` in the shipped
subdirs (`items`, `overmap`, `vehicle`, `traps`, `fields`).
Batch records follow below as they land; the finish line is
0 missing in all eight audit categories.

## Extension batch TRAP1: the LIXA portal family (2026-11-15)

**41 trap ids** — the LIXA facility's invisible teleport
triggers (`tr_LIXA_back_*`, `tr_LIXA_forward_*`,
`tr_LIXA_unfold_entry_*`, numbered channel indices into the
teleport chain, confirmed from the linked EOCs/spells). One
parameterized renderer: green portal pads in the shipped
`tr_portal` treatment, family glyphs (down-chevron / up-chevron
/ nested-squares unfold) with a slight hue lean per family, and
the channel number encoded as lit pips in a 2×8 grid.
`new-tiles/draw_trap1.py`; `new-tiles/preview_trap1.png`.

Verified per the 2g.4 gate (parent re-run): **41 materials, all
`exact`, 0 missing** (123 layers) — the tileset loader does read
the `traps/` subdir; every trap PNG byte-identical across the
re-run. Coverage: traps **121 → 80**. (Bookkeeping note: the
batch brief's hand-typed list and the alphabetical split file
disagreed on the unfold_entry numbering; the drawn set is
back 1–16, forward 1–16, unfold_entry 1–9, and the remaining
trap batches were re-split from the measured still-missing
list, so the union is exact.)

## Extension batches VEH1 + VEH2: vehicle parts, first half (2026-11-15)

**98 vehicle part ids**: VEH1 — the appliance/solar cluster
(solar panels in folding/ground/reinforced/XL treatments, water
wheels, wind turbines, three differentiated weather stations,
the safehouse boiler's gauge-eyed pot belly, the glassblower's
crucible, floodlights, size-graded car battery cells drawn from
their shared abstract) and VEH2 — engines and frames (V-bank
diesels sized by cylinder count, finned single-cylinder family,
steam boilers with flywheels, turbine intake rings, electric
motor cans up to the train traction motor, the dimensional
perambulator's pram under its violet vortex, fridge/heater
tanks told apart by cold/warm stripes, the debug jumper cable
as a checkerboard-tagged artifact). `new-tiles/draw_veh1.py`,
`new-tiles/draw_veh2.py`; previews alongside.

Verified per the 2g.4 gate (parent re-run, each batch
separately): **49 + 49 materials, all `exact`, 0 missing**
(147 layers each) — the loader reads the `vehicle/` subdir;
each batch's PNGs byte-identical across the parent re-run.
Coverage (vehicle overlay, baseline 195 reproduced): 98 direct,
no rescues — vehicle parts **195 → 97** of 483 total.

## Extension batch TRAP2: devices and triggers (2026-11-15)

**40 trap ids** — the LIXA unfold pads 10–16 (TRAP1's renderer
reused verbatim, pixel-consistent with 1–9), the visible devices
drawn as devices (dissector's arcing conduit, crossbow, buried
bear trap, spinning blade, booby trap's shelled pipe clamp,
dermatik burrow, the dormant corpse drawn as a corpse), and the
invisible mapgen triggers as subtle floor markers (the
`tr_light_on_*` light pools differentiated by edge tick and pip
count, lab door glyphs with sensor dots). `new-tiles/draw_trap2.py`;
`new-tiles/preview_trap2.png`.

Verified per the 2g.4 gate (parent re-run): **40 materials, all
`exact`, 0 missing** (120 layers); every PNG byte-identical
across the re-run. Coverage (trap overlay, baseline 80
reproduced): traps **80 → 40**.

## Extension batches VEH3 + VEH4: vehicle parts complete (2026-11-15)

**97 vehicle part ids** — VEH3: storage batteries and their
mounts, pressure tanks, chitin plating (acid variant sheened),
rail wheels, the concrete-plugged ordnance rifle, the literal
`null` part as a red debug tile, and 18 stowed appliances unified
by orange ratchet straps (fridges/freezers told apart by form,
the golden monolith whole vs. cracked, the broken Yrax
quadraphract and its leg). VEH4: the stowed tail (rope-lashed),
the tank family graded by capacity and material, wheel mounts
as bolt fixtures distinct from wheels (steerable = kingpin
knuckle), the vehicle weather stations on roof racks.
`new-tiles/draw_veh3.py`, `new-tiles/draw_veh4.py`; previews
alongside. (One brief typo — a phantom
`veh_mobileroof_weather_station` — was caught against the
authoritative list file and correctly not drawn.)

Verified per the 2g.4 gate (parent re-run, each batch
separately): **49 + 48 materials, all `exact`, 0 missing**;
each batch's PNGs byte-identical across the parent re-run.
Coverage (vehicle overlay, baseline 97 reproduced): **vehicle
parts 97 → 0 of 483 — category complete at 100%.**

## Extension batches FIELD1 + FIELD2: field types complete (2026-11-15)

**90 field type ids** — the two batches split the zone-manager
family between them (~66 `*_zone` fields: one shared treatment,
corner-bracketed area square in a per-category accent with a
category pictogram; NPC zones get figure glyphs with slash /
"?" / retreat-arrow marks, perishables a snowflake; the two
batches, dispatched in parallel, independently converged on
the same treatment) and the physical fields: FIELD1's bee
swarm, bioluminescent motes, clairvoyant eye, full-tile
darkness (its data is `light_override: 0.0`), construction
site, and cold/hot air graded 1–4 in density and color depth;
FIELD2's scarlet family (atmosphere/fog as full-tile washes,
sea gas wisps, cresting sea wave, storm lightning over dark
scarlet), mi-go atmosphere, nauseating smell, full-tile shadow,
push-items arrows, last-known ghost pin. `new-tiles/draw_field1.py`,
`new-tiles/draw_field2.py`; previews alongside.

Verified per the 2g.4 gate (parent re-run, each batch
separately): **45 + 45 materials, all `exact`, 0 missing** —
the loader reads the `fields/` subdir; each batch's PNGs
byte-identical across the parent re-run. Coverage (field
overlay, baseline 90 reproduced): **field types 90 → 0 of
172 — category complete at 100%.** These sprites also feed
the Phase 2b effects sampler (palette/emissive/density), so
the scarlet/mi-go/air sets close a downstream gap there too.

## Extension batch TRAP3: traps complete (2026-11-15)

**40 trap ids** — the portal/stairs/hazard tail: the dungeon
teleporter family as badged spiral vortices (entrance/exit/
elsewhere/next-level told apart by chevron, daylight core, and
star badges), the scarlet and string-dimension tunnel rings,
the microlab shift pads and shifting halls, NL stair pairs with
circled direction chevrons, LIXA fall shafts (the climbing
variant roped), the "mysterious hallway" trio (the
skip-the-teleporter one carrying an amber "!" as its own mark),
and the hazards as themselves — sinkhole, sparkling chasm,
coiled snake, trapdoor spider den, moonpool and its exit line,
temple flood ledge and pressure plate, thick vs. thin ice,
the void spider rift. `new-tiles/draw_trap3.py`;
`new-tiles/preview_trap3.png`.

Verified per the 2g.4 gate (parent re-run): **40 materials, all
`exact`, 0 missing** (120 layers); every PNG byte-identical
across the re-run. Coverage (trap overlay, baseline 40
reproduced): **traps 40 → 0 of 150 — category complete at
100%.**

## Extension batch OM1: overmap, first set (2026-11-15)

**48 overmap terrain ids** — the first overmap batch establishes
the category treatment: full-tile opaque map cells, ground fill
in the entry's color with a centered pictogram. Tree farm
orchard rows (roof = the farm building's roof plan over the
same plot), both ant colonies on one dug-chamber renderer
(chamber contents per function, acid set specked acid-green),
the red alien meadow (core/end/join), base/roof building pairs
sharing footprint and palette, the bunker trio, and the giant
corpse cluster on one flesh ground. `new-tiles/draw_om1.py`;
`new-tiles/preview_om1.png`.

Verified per the 2g.4 gate (parent re-run): **48 materials, all
`exact`, 0 missing** (144 layers) — the loader reads the
`overmap/` subdir; every PNG byte-identical across the re-run.
Coverage (overmap overlay, baseline 336 reproduced): overmap
terrain **336 → 288** of 484.

## Extension batch OM2: shops, EXO, faction camps (2026-11-15)

**48 overmap terrain ids** — city shops with their roof plans,
the EXO/exodii stone safehouses graded 0→2 by growing alien-tech
nodes, and the faction-base camp system: one palisade compound
with a per-function pictogram (anvil, tent, canteen, crop rows,
garage, helipad, kitchen, livestock), the `_2_0` tiers upgraded
to stone/metal palisades with the same icons, and the mansion
components on hedged lawns (t3 drawn with a "?" — its data name
is literally "mansion base???"). Debug tiles in the project's
black/red treatment. `new-tiles/draw_om2.py`;
`new-tiles/preview_om2.png`.

Verified per the 2g.4 gate (parent re-run): **48 materials, all
`exact`, 0 missing** (144 layers); every PNG byte-identical
across the re-run. Coverage (overmap overlay, baseline 288
reproduced): overmap terrain **288 → 240**.

## Extension batch OM3: mansion tail, farms, FEMA (2026-11-15)

**48 overmap terrain ids** — the mansion tower rooms (kitchen,
library, bedroom, living room on colored rugs), the remaining
faction surveys (saltworks pans, storehouse/workshop tiers, the
camouflaged hide site), the farm set and its Isherwood copies
(coop, carriage house, stable + hayloft, every roof plan — two
ids drawn as their data names rather than their id names:
`farm_dairy_twd_11` is "farm road", `farm_stills_11` is
"orchard"), the FEMA camp in one pale-gravel language, fungal
flowers/tower, the golf course cells, and storefronts
(`full_mutagen_lab_roof` drawn as its data name, a Trans-Coast
Logistics roof). `new-tiles/draw_om3.py`;
`new-tiles/preview_om3.png`.

Verified per the 2g.4 gate (parent re-run): **48 materials, all
`exact`, 0 missing** (144 layers); every PNG byte-identical
across the re-run. Coverage (overmap overlay, baseline 240
reproduced): overmap terrain **240 → 192**.

## Extension batch OM4: labyrinth, kelp, looted set (2026-11-15)

**48 overmap terrain ids** — the labyrinth complex on one
dark-floor/stone-wall system (maze rings, roof block-plans, the
gold-sparkle chasm maw in core/end widths, flooded and
overgrown states, stairs, safehouse, boiler exit), kelp forest
graded core/end on open water, highway and its bridge, the
looted hospital/house set with damage (scraped cross, broken
windows, a faded helipad H on the hospital roof), LAN center
pair, landfill, lighthouse, the near-black inner-cabins depths
(its data symbol is a space on black — intentionally almost
pure black), and the mi-go camp's resin spires. `new-tiles/draw_om4.py`;
`new-tiles/preview_om4.png`.

Verified per the 2g.4 gate (parent re-run): **48 materials, all
`exact`, 0 missing** (144 layers); every PNG byte-identical
across the re-run. Coverage (overmap overlay, baseline 192
reproduced): overmap terrain **192 → 144**.

## Extension batch OM5: microlab, mi-go, minefield (2026-11-15)

**48 overmap terrain ids** — the microlab system on dark paneled
floors (generic/edge/sub-entry/surface/vent variants, the
ratkin set re-paletted green on warm floors, the nether-glass
portal elevator as a violet shaft on black), the mi-go camp
segments and portal (pale organic ground, pink flesh forms,
teal glow; the portal's "=" data symbol rendered as twin energy
bands), the minefield perimeter (fence lines on the named
edges, mine discs, gates with swung leaves), the ocean depth
grade (shore → surface → water cube → bed), and `omt_obsolete`
as a debug tile carrying its own "%" glyph. `new-tiles/draw_om5.py`;
`new-tiles/preview_om5.png`.

Verified per the 2g.4 gate (parent re-run): **48 materials, all
`exact`, 0 missing** (144 layers); every PNG byte-identical
across the re-run. Coverage (overmap overlay, baseline 144
reproduced): overmap terrain **144 → 96**.

## Extension batches OM6 + OM7 + null terrain: overmap complete (2026-11-15)

**96 overmap terrain ids** — OM6: the classic shop row on one
storefront language (sign band in the data color, per-shop
pictogram, fascia-striped roofs), the ravine depth grade, the
river-cave system (food stores, nest, the kraken's tentacle
chamber), refugee-center fence corners running on exactly the
edges their line glyphs name, and the portal-storm teleporter
as a nether-violet vortex. OM7: the shop tail, slime pits
graded z0 → z−2 into darkness, string-dimension line geometry
on void black, the triffid grade from field shoots to the
finale's red heart, true stadium corner segments, and the
`unexplored` cell as the classic dark `#`-hatched unknown.
Plus a parent-drawn micro-batch: the audit's 336th overmap id
is the game data's **null terrain** — an `overmap_terrain`
whose id is the empty string (name "nothing", sym `%`, white).
The decomposed format represents it (a folder whose entry JSON
declares `"id": ""` — the voxelizer resolves a material with
id `""` as `exact`), so it is drawn as a void cell with a faint
white `%`. `new-tiles/draw_om6.py`, `new-tiles/draw_om7.py`,
`new-tiles/draw_nothing.py`; previews alongside.

Verified per the 2g.4 gate (parent re-run, each batch
separately): **48 + 47 materials, all `exact`, 0 missing**, the
null terrain 1/1 `exact` with id `""`; every PNG byte-identical
across the parent re-runs. Coverage (overmap overlay with
JSON-declared ids counted, baseline 96 = 48 + 47 + the null
terrain): **overmap terrain 96 → 0 of 484 — category complete
at 100%.**

## Extension item batches (2026-11-15)

The 3,742 missing items are batched by defining file (91
batches, `/tmp`-side index; batch = one file cluster, split at
52). Item records are grouped below as batches verify; every
batch individually passes the full gate (parent re-run:
byte-identical, voxelizer all-`exact`/0-missing on its own
folders, overlay count).

- **ITEM01** — bionics 1/3 (52): CBM canisters with function
  pictograms, armor body-part shells, deployed blade/claws,
  the Exodii cyberlimbs and Cyberframes, the brass Cree'yn
  heart. Items 3,742 → 3,690. `draw_item01.py`.
- **ITEM03** — bionics 3/3 (46): deployed razors/scalpels,
  synthetic lungs, the Hub 01 XM exo interfaces as hardware,
  faulty units cracked/glitched. Items → 3,644. `draw_item03.py`.
- **ITEM02** — bionics 2/3 (52): emitters, sensors, power
  storage graded by charge, probability travel half through a
  wall. **Bionics cluster complete (150).** Items → 3,592.
  `draw_item02.py`.
- **ITEM04** — generic 1/3 (52): the broken_* wreck family
  parameterized by weapon/form (drones, turrets, humanoid bots,
  superalloy mech wrecks), faceted gems, XEDRA documents.
  Items → 3,540. `draw_item04.py`.
- **ITEM05** — generic 2/3 (52): appliances, gems, documents,
  USB drives; the `metabolism`/`muscle` pseudo-items as debug
  tiles (their descriptions say "seeing this is a bug").
  Items → 3,488. `draw_item05.py`.
- **ITEM06** — generic 3/3 (37): gems, nanofabricator template
  slates, NRE printouts, the sun/wind pseudo-items as glyphs.
  **Generic cluster complete (141).** Items → 3,451.
  `draw_item06.py`.
- **ITEM07** — eggs 1/2 (52): species-parameterized bird eggs,
  fish roe, frog egg clusters per their real arrangements,
  prepared egg dishes. Items → 3,395 measured (incl. 4
  knock-on `looks_like` rescues — intact bot/exoskeleton items
  whose targets ITEM04 drew). `draw_item07.py`.

- **ITEM08 + ITEM09** — carnivore comestibles (52 + 28):
  parameterized meats/hearts/stomachs/seafood, offal piles,
  boiled stomachs, folded hides. **Carnivore cluster complete
  (80).** Items → 3,315. `draw_item08.py`, `draw_item09.py`.

- **ITEM10 + ITEM11** — meat dishes (52 + 24): canned goods
  with visible contents, burgers graded plain/cheese/deluxe,
  plated dishes, paella, dumplings, pemmican, sashimi, the
  halo'd debugging SPAM. **Meat-dishes cluster complete
  (76).** Items → 3,239. `draw_item10.py`, `draw_item11.py`.

- **ITEM12 + ITEM13** — tool-armor (52 + 23): the survivor-
  mask matrix across sizes, goggles dim vs. lit, thermal-
  electric garments with heating coils when on, smart
  watches, welding/PAPR helmets. **Tool-armor cluster
  complete (75).** Items → 3,164. `draw_item12.py`,
  `draw_item13.py`.

- **ITEM14 + ITEM15** — fluff (52 + 19): board games drawn
  as their actual boards (backgammon, go, hnefatafl, senet,
  shogi, mahjong, Scrabble spelling PLAY) and euro-game
  boxes with per-game cover art. **Fluff cluster complete
  (71).** Items → 3,093. `draw_item14.py`, `draw_item15.py`.

- **ITEM16 + ITEM17** — head attachments (52 + 17): NVGs
  by tube count and state, face shields, and the 45-piece
  chainmail family graded across five steels (tempered
  gets gold inlay and a gem; high steel gets its mustache).
  **Head-attachments cluster complete (69).** Items →
  3,024. `draw_item16.py`, `draw_item17.py`.

- **ITEM18 + ITEM19** — swords and blades (52 + 12): one
  analytic diagonal renderer, profiles per weapon (estoc
  needle, khopesh hook, kukri belly, kris wave), shading
  per steel grade, fakes blunt and inferiors nicked.
  **Swords cluster complete (64).** Items → 2,960.
  `draw_item18.py`, `draw_item19.py`.

- **ITEM20 + ITEM21** — metal resources (52 + 10): ingots
  scaled by volume and graded by metal, bar stock flat vs.
  round, I-beams, rail profile, porous slag. **Metal
  cluster complete (62).** Items → 2,898. `draw_item20.py`,
  `draw_item21.py`.

- **ITEM22 + ITEM23** — fake items (52 + 10): the internal
  stand-ins drawn as the objects they name — workshop
  machines (lathe, mill, presses, arc furnace), hoists and
  anvils graded, training dummies, spinning wheel, kiln.
  **Fake cluster complete (62).** Items → 2,836.
  `draw_item22.py`, `draw_item23.py`.

- **ITEM24 + ITEM25** — mutagens (52 + 9): IV primers as
  injectors and oral mutagens as flasks, one renderer per
  form, liquid color and label motif per mutation line;
  the mutated arm/leg drawn as limbs. **Mutagen cluster
  complete (61).** Items → 2,775. `draw_item24.py`,
  `draw_item25.py`.

- **ITEM26 + ITEM27** — vegetable dishes (52 + 7): plated
  roasted/fried vegetables, split beans-and-rice bowl,
  pickles in their jars, aspic, vac-bagged salted veg.
  **Veggy-dishes cluster complete (59).** Items → 2,716.
  `draw_item26.py`, `draw_item27.py`.

- **ITEM29** — integrated armor (52): mutation-grown body
  features as panels and mounts — fur/scale/bark/chitin
  graded by coverage, antlers, horns, fang mouths, the
  back-shell growth series, a glowing reality-fragment
  shard. Items → 2,664. `draw_item29.py`.
- **ITEM28** — ammo components (52): bullet cores sized by
  caliber weight, gunpowders by grain form, primer cups,
  gas bottles with real shoulder bands, RTG/uranium with
  trefoils. **Ammo cluster complete.** Items → 2,612.
  `draw_item28.py`.

- **ITEM30** — chemicals and resources (50): substances in
  their containers — acid bottles by liquid color, bagged
  and piled powders by grain style, fertilizer sacks,
  fermentable jugs, trefoiled cobalt/uranium. Items →
  2,562. `draw_item30.py`.

- **ITEM31** — junk food (47): snacks in their packaging —
  candy bags told apart by the candies themselves,
  freeze-dried pouches sealed vs. hydrated, fast-food
  cartons, the debug angel cake under a golden halo.
  Items → 2,515. `draw_item31.py`.

- **ITEM32** — medical items (45): pill bottles with per-
  drug label motifs and loose pills, the panacea gel cap
  as itself, birth-control dial pack, 13 contact-lens
  cases told apart by Rx glyph and cap tint. Items →
  2,470. `draw_item32.py`.

- **ITEM33** — tailoring resources (44): fabric patches
  with stitch borders by weave, wide sheets with hems,
  patchwork grids, scrap handfuls, the Kevlar plate and
  ballistic panel as themselves. Items → 2,426.
  `draw_item33.py`.

- **ITEM34** — shell casings (43): one parameterized pile
  renderer with real cartridge proportions (bottleneck,
  straight, rimmed, belted); fired vs. unprimed told by
  primer state and brass brightness; alien Diyua shells
  with glyph-ticked bases. Items → 2,383.
  `draw_item34.py`.

- **ITEM35** — jewelry (43): pieces drawn large and
  centered — band rings by metal and stone, loop and stud
  piercings, badges and medals, costume ears/horns/tails
  in fur, faux fur, and leather. Items → 2,340.
  `draw_item35.py`.

**Process note (2026-11-15):** a VM replacement wiped `/tmp`,
taking the item batch lists and the overlay tool with it. All
343 drawn item folders and every commit were intact under
`~/workspace`. The lists were recovered (01–05 from their
commits, 06–07 by defining file), the overlay tool was rebuilt
at `~/workspace/audit-tools/overlay_cat.py`, and the remaining
3,395 ids were re-batched by defining file into batches 08–91
at `~/workspace/item-batches/`. New batch scripts embed their
id lists and read them from the durable path.

## Status

Batches are committed locally as they land (butterfly/moth in
`5f740f2`, alien meadow in `13e91e2`, string dimension with this
record); pushes follow Joey's word via the PC-clone bundle route.
`fork-src/` is gitignored, so expanding its sparse checkout for
`data/json` does not dirty the repo.
Gate 5.10 stays open, now with a measured list instead of an unknown:
as written (≥98% of base-game terrain+furniture) it cannot pass on
current UltiCa — closing it means either re-scoping the gate or a
tile-creation/`looks_like`-entry effort against `missing.json`.
