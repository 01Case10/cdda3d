#!/usr/bin/env python3
"""
Draw the missing regional-pseudo terrain sprites.

Sources (fork-src/cdda/data/json/furniture_and_terrain/
terrain-regional-pseudo.json, CDDA 6a3e59e0) -- every entry in that file
whose id is in coverage-audit/missing.json (missing_shipped, terrain)
and has no folder yet: 29 ids. They are region *templates*: each entry
is named "pseudo terrain" and described as "This should never actually
show up, it should have been replaced with regional terrain" -- the
regional mapgen substitutes a concrete biome terrain for the template.
They still need sprites (the audit counts them missing, and a template
that ever leaks through should render as the ground it stands for),
so each is drawn as the full-tile ground its name/template role names:

  groundcovers  t_region_groundcover (mixed temperate groundcover),
                _urban (trampled city dirt + rubble), _forest (leaf
                litter), _swamp (wet mud + algae), _barren ("trampled
                away grass, or otherwise cleared plant life" -- dry
                cracked dirt), _ravine ("bottom of ravines" -- scree)
  grass/soil    t_region_grass ("clean, curated grass"), t_region_soil
                ("plants have been planted and grass kept clear" --
                tilled furrows)
  tree floors   t_region_tree + _forest / _forest_dense / _forest_other
                (leaf-litter forest floors, graded by density),
                _evergreen (needle litter + cones), _fruit / _fruit_wild
                (fallen fruit / berries), _nut (nut shells), _shade
                (deep-shade litter), _dead (ash litter + bare twigs)
  shrub floors  t_region_shrub + _plains / _swamp / _forest /
                _forest_dense (scrub grounds graded by biome),
                _fruit (berry scrub), _decorative (garden mulch +
                trimmed blobs)
  other regions t_highland_groundcover ("used in the highland
                dimensions" -- pale alpine grit + tough tufts),
                t_string_dimension_groundcover (the string dimension's
                black nothing, faint specks -- matches the batch-2
                void/points), t_netherum_labyrinth_groundcover (dark
                labyrinth stone rubble), t_portal_storm_dungeon_
                groundcover (scorched dungeon ground, ash + faint
                ember/violet flecks)

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet, every entry a full-tile ground (fg = own sprite, bg "").
Muted natural textures; variants differ in palette and surface
feature so the templates are distinguishable. Deterministic: every
random choice comes from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json +
credits) and preview_regional_pseudo.png.
"""
from PIL import Image, ImageDraw
import json, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

# ---------------------------------------------------------------- helpers
def ground(base):
    return Image.new("RGBA", (S, S), base + (255,))

def speckle(rng, d, colors, n, lo=1, hi=2):
    for _ in range(n):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        w = rng.uniform(lo, hi)
        d.ellipse([x, y, x + w, y + w * 0.7], fill=colors[rng.randrange(len(colors))] + (255,))

def blades(rng, d, colors, n, ln_lo=3, ln_hi=6):
    for _ in range(n):
        x = rng.uniform(0, S - 1); y = rng.uniform(2, S - 1)
        ln = rng.uniform(ln_lo, ln_hi); lean = rng.uniform(-1.5, 1.5)
        d.line([(x, y), (x + lean, y - ln)], fill=colors[rng.randrange(len(colors))] + (255,))

def litter(rng, d, colors, n):
    # fallen leaves: small flattened ellipses at random angles (faked
    # with width/height jitter)
    for _ in range(n):
        x = rng.uniform(0, S - 4); y = rng.uniform(0, S - 3)
        w = rng.uniform(2.5, 4.5); h = w * rng.uniform(0.4, 0.6)
        d.ellipse([x, y, x + w, y + h], fill=colors[rng.randrange(len(colors))] + (255,))

def scree(rng, d, colors, n):
    for _ in range(n):
        x = rng.uniform(0, S - 5); y = rng.uniform(0, S - 4)
        w = rng.uniform(3, 5.5); h = w * rng.uniform(0.5, 0.8)
        col = colors[rng.randrange(len(colors))] + (255,)
        d.polygon([(x, y + h), (x + w * 0.4, y), (x + w, y + h * 0.7)], fill=col)

def dots(rng, d, color, n, r_lo=1.0, r_hi=1.8, highlight=False):
    for _ in range(n):
        x = rng.uniform(1, S - 3); y = rng.uniform(1, S - 3)
        r = rng.uniform(r_lo, r_hi)
        d.ellipse([x, y, x + 2 * r, y + 2 * r], fill=color + (255,))
        if highlight:
            d.point((x + r * 0.6, y + r * 0.5), fill=(255, 230, 200, 255))

def blobs(rng, d, colors, n, r_lo=3, r_hi=5.5):
    # low scrub mounds: overlapping ellipses, darker at the base
    for _ in range(n):
        x = rng.uniform(0, S - 6); y = rng.uniform(0, S - 5)
        r = rng.uniform(r_lo, r_hi)
        col = colors[rng.randrange(len(colors))]
        d.ellipse([x, y + r * 0.4, x + 2 * r, y + 2 * r], fill=tuple(max(0, c - 26) for c in col) + (255,))
        d.ellipse([x + r * 0.3, y, x + 1.7 * r, y + 1.3 * r], fill=col + (255,))

# ---------------------------------------------------------------- groundcovers
def draw_groundcover():
    rng = random.Random(3101)
    img = ground((96, 104, 58)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(84, 92, 50), (110, 116, 66), (104, 94, 60)], 60)
    blades(rng, d, [(116, 128, 66), (88, 102, 52)], 42)
    return img

def draw_groundcover_urban():
    rng = random.Random(3102)
    img = ground((106, 98, 86)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(94, 86, 74), (120, 112, 98), (84, 78, 68)], 70)
    speckle(rng, d, [(150, 148, 142), (132, 130, 124)], 14, 1, 2.6)  # rubble
    blades(rng, d, [(102, 112, 62)], 10)  # sparse weeds
    return img

def draw_groundcover_forest():
    rng = random.Random(3103)
    img = ground((74, 66, 44)); d = ImageDraw.Draw(img)
    litter(rng, d, [(96, 82, 50), (110, 92, 54), (64, 74, 40), (88, 70, 44)], 46)
    speckle(rng, d, [(58, 52, 36)], 24)
    return img

def draw_groundcover_swamp():
    rng = random.Random(3104)
    img = ground((52, 58, 40)); d = ImageDraw.Draw(img)
    # algae sheen patches
    for _ in range(7):
        x = rng.uniform(0, S - 8); y = rng.uniform(0, S - 6)
        d.ellipse([x, y, x + rng.uniform(5, 9), y + rng.uniform(3, 5)], fill=(74, 96, 52, 255))
    speckle(rng, d, [(40, 46, 32), (66, 70, 48)], 40)
    # water glints
    for _ in range(9):
        x = rng.uniform(0, S - 3); y = rng.uniform(0, S - 1)
        d.line([(x, y), (x + rng.uniform(1.5, 3), y)], fill=(150, 168, 150, 255))
    return img

def draw_groundcover_barren():
    rng = random.Random(3105)
    img = ground((148, 128, 96)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(134, 114, 84), (162, 142, 110), (124, 106, 78)], 46)
    # dry cracks
    for _ in range(6):
        x = rng.uniform(0, S); y = rng.uniform(0, S)
        pts = [(x, y)]
        for _ in range(3):
            x += rng.uniform(-5, 5); y += rng.uniform(1, 5)
            pts.append((x, y))
        d.line(pts, fill=(104, 88, 62, 255))
    speckle(rng, d, [(170, 162, 148)], 10, 1, 2.2)  # pebbles
    return img

def draw_groundcover_ravine():
    rng = random.Random(3106)
    img = ground((104, 96, 84)); d = ImageDraw.Draw(img)
    scree(rng, d, [(122, 112, 98), (88, 80, 70), (140, 130, 114), (76, 70, 62)], 34)
    speckle(rng, d, [(70, 64, 56)], 20)
    return img

# ---------------------------------------------------------------- grass / soil
def draw_grass():
    rng = random.Random(3110)
    img = ground((86, 128, 54)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(76, 116, 48), (98, 140, 62)], 40)
    blades(rng, d, [(104, 148, 66), (78, 120, 50), (118, 158, 76)], 90, 2, 4)
    return img

def draw_soil():
    rng = random.Random(3111)
    img = ground((102, 76, 50)); d = ImageDraw.Draw(img)
    # tilled furrows: alternating dark trough / lit ridge rows
    for y in range(1, S, 4):
        d.line([(0, y), (S - 1, y)], fill=(82, 60, 38, 255))
        d.line([(0, y + 2), (S - 1, y + 2)], fill=(122, 94, 62, 255))
    speckle(rng, d, [(90, 66, 44), (116, 90, 60)], 30)
    return img

# ---------------------------------------------------------------- tree floors
def _forest_floor(seed, base, leaf_cols, n_leaves, extra=None):
    rng = random.Random(seed)
    img = ground(base); d = ImageDraw.Draw(img)
    litter(rng, d, leaf_cols, n_leaves)
    speckle(rng, d, [(52, 44, 30)], 18)
    if extra:
        extra(rng, d)
    return img

def draw_tree():
    def roots(rng, d):
        for _ in range(4):  # root arcs creeping from the edges
            x = rng.uniform(0, S); y = rng.uniform(0, S)
            d.arc([x - 6, y - 3, x + 6, y + 5], 0, 180, fill=(66, 54, 38, 255))
    return _forest_floor(3120, (80, 68, 44), [(102, 86, 50), (90, 76, 46), (70, 78, 42)], 36, roots)

def draw_tree_forest():
    return _forest_floor(3121, (78, 66, 44), [(100, 84, 50), (88, 74, 44), (68, 76, 40)], 42)

def draw_tree_forest_dense():
    return _forest_floor(3122, (58, 52, 36), [(76, 64, 42), (66, 58, 38), (50, 58, 34)], 62)

def draw_tree_forest_other():
    def stones(rng, d):
        speckle(rng, d, [(128, 124, 116), (108, 104, 98)], 12, 1.5, 3)
    return _forest_floor(3123, (82, 70, 48), [(104, 88, 52), (92, 78, 48)], 30, stones)

def draw_tree_evergreen():
    rng = random.Random(3124)
    img = ground((96, 72, 48)); d = ImageDraw.Draw(img)
    # needle litter: many short thin strokes
    for _ in range(120):
        x = rng.uniform(0, S - 3); y = rng.uniform(0, S - 1)
        d.line([(x, y), (x + rng.uniform(2, 3.5), y + rng.uniform(-0.6, 0.6))],
               fill=(118, 88, 54, 255) if rng.random() < 0.5 else (84, 62, 40, 255))
    dots(rng, d, (70, 50, 30), 6, 1.2, 2.0)  # fallen cones
    return img

def draw_tree_fruit():
    def fruit(rng, d):
        dots(rng, d, (178, 62, 44), 7, 1.2, 1.9, highlight=True)
        dots(rng, d, (206, 132, 52), 5, 1.2, 1.8, highlight=True)
    return _forest_floor(3125, (80, 68, 44), [(102, 86, 50), (90, 76, 46)], 34, fruit)

def draw_tree_fruit_wild():
    def berries(rng, d):
        dots(rng, d, (86, 44, 96), 8, 0.8, 1.3)
        dots(rng, d, (140, 44, 54), 5, 0.8, 1.2)
    return _forest_floor(3126, (76, 64, 42), [(98, 82, 48), (86, 72, 44)], 36, berries)

def draw_tree_nut():
    def nuts(rng, d):
        for _ in range(10):
            x = rng.uniform(1, S - 3); y = rng.uniform(1, S - 3)
            d.ellipse([x, y, x + 2.6, y + 2], fill=(168, 138, 92, 255))
            d.line([(x + 1.3, y), (x + 1.3, y + 2)], fill=(120, 96, 60, 255))
    return _forest_floor(3127, (82, 70, 46), [(104, 88, 52), (92, 78, 48)], 32, nuts)

def draw_tree_shade():
    return _forest_floor(3128, (52, 54, 40), [(66, 68, 46), (58, 62, 42), (46, 50, 36)], 40)

def draw_tree_dead():
    rng = random.Random(3129)
    img = ground((116, 110, 100)); d = ImageDraw.Draw(img)
    litter(rng, d, [(128, 122, 110), (104, 100, 92), (138, 132, 120)], 30)
    # bare pale twigs
    for _ in range(9):
        x = rng.uniform(0, S - 6); y = rng.uniform(0, S - 1)
        d.line([(x, y), (x + rng.uniform(4, 7), y + rng.uniform(-2, 2))], fill=(168, 162, 150, 255))
    speckle(rng, d, [(96, 92, 86)], 20)
    return img

# ---------------------------------------------------------------- shrub floors
def draw_shrub():
    rng = random.Random(3140)
    img = ground((88, 94, 54)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(78, 84, 48), (100, 104, 62)], 34)
    blobs(rng, d, [(70, 96, 46), (60, 84, 40)], 5)
    blades(rng, d, [(108, 118, 64)], 18)
    return img

def draw_shrub_fruit():
    rng = random.Random(3141)
    img = ground((86, 92, 52)); d = ImageDraw.Draw(img)
    blobs(rng, d, [(68, 94, 44), (58, 82, 38)], 5)
    dots(rng, d, (150, 52, 66), 10, 0.8, 1.3)
    dots(rng, d, (104, 52, 120), 7, 0.8, 1.2)
    return img

def draw_shrub_plains():
    rng = random.Random(3142)
    img = ground((110, 122, 62)); d = ImageDraw.Draw(img)
    blades(rng, d, [(124, 136, 72), (96, 110, 54)], 50, 3, 5)
    blobs(rng, d, [(88, 108, 52)], 3, 2.5, 4)
    return img

def draw_shrub_swamp():
    rng = random.Random(3143)
    img = ground((56, 62, 44)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(44, 50, 34), (70, 76, 54)], 30)
    # reed strokes
    for _ in range(22):
        x = rng.uniform(0, S - 1); y = rng.uniform(4, S - 1)
        d.line([(x, y), (x + rng.uniform(-0.7, 0.7), y - rng.uniform(4, 7))], fill=(96, 118, 66, 255))
    blobs(rng, d, [(52, 74, 40)], 3)
    return img

def draw_shrub_forest_dense():
    rng = random.Random(3144)
    img = ground((60, 56, 40)); d = ImageDraw.Draw(img)
    litter(rng, d, [(78, 68, 44), (66, 62, 40)], 30)
    blobs(rng, d, [(52, 70, 38), (44, 60, 32)], 7)
    return img

def draw_shrub_forest():
    rng = random.Random(3145)
    img = ground((72, 66, 44)); d = ImageDraw.Draw(img)
    litter(rng, d, [(94, 80, 48), (82, 72, 44)], 26)
    blobs(rng, d, [(62, 84, 42), (54, 74, 36)], 5)
    return img

def draw_shrub_decorative():
    rng = random.Random(3146)
    img = ground((88, 64, 44)); d = ImageDraw.Draw(img)
    # garden mulch chips
    speckle(rng, d, [(104, 78, 52), (74, 54, 36), (118, 90, 60)], 70, 1, 2.6)
    # neat trimmed blobs in a row-ish arrangement
    blobs(rng, d, [(64, 104, 48)], 4, 3.5, 5)
    return img

# ---------------------------------------------------------------- other regions
def draw_highland_groundcover():
    rng = random.Random(3150)
    img = ground((138, 130, 112)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(124, 116, 100), (152, 144, 126), (110, 104, 92)], 50)
    speckle(rng, d, [(158, 154, 146)], 12, 1, 2.4)  # stones
    blades(rng, d, [(118, 122, 74), (104, 110, 64)], 16, 2, 3.5)  # tough tufts
    return img

def draw_string_dimension_groundcover():
    rng = random.Random(3151)
    img = ground((0, 0, 0)); d = ImageDraw.Draw(img)
    # the string dimension's nothing, with faint far-off specks
    for _ in range(7):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=(120, 120, 128, 255))
    d.point((16, 16), fill=(210, 210, 218, 255))
    return img

def draw_netherum_labyrinth_groundcover():
    rng = random.Random(3152)
    img = ground((52, 50, 54)); d = ImageDraw.Draw(img)
    scree(rng, d, [(68, 66, 72), (40, 38, 44), (84, 80, 88)], 26)
    speckle(rng, d, [(96, 92, 88)], 12)  # pale dust
    return img

def draw_portal_storm_dungeon_groundcover():
    rng = random.Random(3153)
    img = ground((46, 38, 34)); d = ImageDraw.Draw(img)
    speckle(rng, d, [(58, 48, 42), (34, 28, 26), (120, 112, 100)], 44)  # ash
    dots(rng, d, (150, 84, 40), 3, 0.7, 1.1)   # faint embers
    dots(rng, d, (110, 84, 160), 4, 0.7, 1.1)  # arcane violet flecks
    return img

# ---------------------------------------------------------------- jobs
JOBS = [
    ("t_region_groundcover", draw_groundcover()),
    ("t_region_groundcover_urban", draw_groundcover_urban()),
    ("t_region_groundcover_forest", draw_groundcover_forest()),
    ("t_region_groundcover_swamp", draw_groundcover_swamp()),
    ("t_region_groundcover_barren", draw_groundcover_barren()),
    ("t_region_grass", draw_grass()),
    ("t_region_groundcover_ravine", draw_groundcover_ravine()),
    ("t_region_soil", draw_soil()),
    ("t_region_tree", draw_tree()),
    ("t_region_tree_forest_dense", draw_tree_forest_dense()),
    ("t_region_tree_forest", draw_tree_forest()),
    ("t_region_tree_forest_other", draw_tree_forest_other()),
    ("t_region_tree_evergreen", draw_tree_evergreen()),
    ("t_region_tree_fruit", draw_tree_fruit()),
    ("t_region_tree_fruit_wild", draw_tree_fruit_wild()),
    ("t_region_tree_nut", draw_tree_nut()),
    ("t_region_tree_shade", draw_tree_shade()),
    ("t_region_tree_dead", draw_tree_dead()),
    ("t_region_shrub", draw_shrub()),
    ("t_region_shrub_fruit", draw_shrub_fruit()),
    ("t_region_shrub_plains", draw_shrub_plains()),
    ("t_region_shrub_swamp", draw_shrub_swamp()),
    ("t_region_shrub_forest_dense", draw_shrub_forest_dense()),
    ("t_region_shrub_forest", draw_shrub_forest()),
    ("t_region_shrub_decorative", draw_shrub_decorative()),
    ("t_highland_groundcover", draw_highland_groundcover()),
    ("t_string_dimension_groundcover", draw_string_dimension_groundcover()),
    ("t_netherum_labyrinth_groundcover", draw_netherum_labyrinth_groundcover()),
    ("t_portal_storm_dungeon_groundcover", draw_portal_storm_dungeon_groundcover()),
]

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": tid, "bg": ""}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
CELL = 64
COLS = 6
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, img) in enumerate(JOBS):
    t = img.resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_regional_pseudo.png"))
print("wrote preview_regional_pseudo.png", sheet.size)
