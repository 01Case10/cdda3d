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
