#!/usr/bin/env python3
"""
Draw the missing wall-terrain sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/terrain-walls.json,
CDDA 6a3e59e0) -- every terrain id in that file which appears in
coverage-audit/missing.json (missing_shipped, category terrain) and has
no folder yet: 20 ids.

  t_wall_metal_shipping_container (+ 14 color variants by copy-from)
                              "A shipping container made of corrugated
                               weathering steel."  symbol #, base color
                               light_gray; variants: black, white, red,
                               light_red, green, light_green, blue,
                               light_blue, cyan, light_cyan, magenta,
                               brown, pink, yellow
  t_wall_warped               "At first glance, this looks like a simple
                               stone wall.  But the angles don't add up,
                               and the shadows are wrong.  Whatever this
                               is, it doesn't belong here."  color magenta
  t_reinforced_glass_lab      copy-from t_reinforced_glass: "A thicker
                               pane of glass with thin metal wires
                               embedded throughout, strengthening the
                               structural properties."
  t_paper_brood               copy-from t_paper ("paper wall"): "A huge
                               wall of ambiguous pulpy matter, covered in
                               sticky wasp saliva." -- the brood-nest
                               paper: layered, grayer, glossier
  t_paper_hard                "layered paper wall": "A thick wall of
                               layered tree pulp and wasp saliva,
                               hardened to withstand the elements."
                               color dark_gray
  t_wood_sunshade_pole        "A wooden pole supporting a canvas
                               sunshade."  color brown, symbol 1,
                               WALL/PERMEABLE

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet, flat fills, dark outlines, one highlight pass. Entries are
full-tile fg with bg "" (the pole's sprite itself carries alpha --
a pole is mostly air). Deterministic: every random choice comes from
a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json + credits).
"""
from PIL import Image, ImageDraw
import json, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def full(color):
    return Image.new("RGBA", (S, S), color)

def transparent():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def shade(rgb, f):
    return tuple(max(0, min(255, int(c * f))) for c in rgb) + (255,)

# ------------------------------------------------------- shipping containers
# Corrugated weathering steel: vertical ribs (highlight / face / shadow
# columns), top and bottom rails, rivets, rust streaks bleeding down
# from the top rail and grime along the bottom -- the same panel in
# each of the game's paint colors.
CONTAINERS = [
    ("t_wall_metal_shipping_container", (172, 174, 180)),          # light_gray
    ("t_wall_metal_shipping_container_black", (28, 28, 30)),
    ("t_wall_metal_shipping_container_white", (238, 238, 236)),
    ("t_wall_metal_shipping_container_red", (168, 24, 22)),
    ("t_wall_metal_shipping_container_light_red", (228, 92, 80)),
    ("t_wall_metal_shipping_container_green", (30, 128, 42)),
    ("t_wall_metal_shipping_container_light_green", (110, 208, 108)),
    ("t_wall_metal_shipping_container_blue", (30, 60, 178)),
    ("t_wall_metal_shipping_container_light_blue", (110, 160, 238)),
    ("t_wall_metal_shipping_container_cyan", (18, 178, 188)),
    ("t_wall_metal_shipping_container_light_cyan", (140, 234, 238)),
    ("t_wall_metal_shipping_container_magenta", (188, 30, 168)),
    ("t_wall_metal_shipping_container_brown", (128, 84, 44)),
    ("t_wall_metal_shipping_container_pink", (238, 148, 188)),
    ("t_wall_metal_shipping_container_yellow", (234, 204, 38)),
]

def draw_container(base, seed):
    rng = random.Random(seed)
    img = full(base + (255,))
    d = ImageDraw.Draw(img, "RGBA")
    hi = shade(base, 1.22)
    lo = shade(base, 0.70)
    rail = shade(base, 0.82)
    # corrugation ribs, 4px pitch: hi | face | face | shadow
    for x in range(0, S, 4):
        d.rectangle([x, 0, x, S - 1], fill=hi)
        d.rectangle([x + 3, 0, x + 3, S - 1], fill=lo)
    # top and bottom rails
    d.rectangle([0, 0, S - 1, 2], fill=rail)
    d.rectangle([0, S - 3, S - 1, S - 1], fill=rail)
    d.line([(0, 3), (S - 1, 3)], fill=lo)
    d.line([(0, S - 4), (S - 1, S - 4)], fill=hi)
    # rivets along the rails
    for x in (2, 15, 29):
        for y in (1, S - 2):
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=lo)
    # weathering, kept faint: rust stains (the rust color blended
    # toward the paint, so a stain reads as a stain on bright colors
    # too, not as black drips) and a few small paint-wear flecks
    rust = tuple(int(0.35 * r + 0.65 * b) for r, b in zip((150, 84, 38), base)) + (26,)
    for _ in range(3):
        x = rng.uniform(1, S - 2)
        ln = rng.uniform(2, 4)
        d.rectangle([x, 3, x + 1, 3 + ln], fill=rust)
    wear = shade(base, 0.78)[:3] + (34,)
    for _ in range(6):
        x = rng.uniform(0, S - 2); y = rng.uniform(4, S - 5)
        d.rectangle([x, y, x + rng.uniform(1, 2), y + 1], fill=wear)
    # bottom grime
    d.rectangle([0, S - 6, S - 1, S - 4], fill=(40, 32, 24, 30))
    return img

# ------------------------------------------------------- warped wall
def draw_warped():
    rng = random.Random(3301)
    img = full((96, 84, 102, 255))  # mortar, magenta-tainted
    d = ImageDraw.Draw(img)
    stone = [(148, 142, 150), (138, 130, 142), (156, 150, 158), (142, 134, 138)]
    # stone courses whose offsets refuse to stay consistent: each row
    # shifts a little further, then one block shears into a trapezoid
    y = 0
    row = 0
    while y < S:
        h = 7
        x = -((row * 5) % 11)
        col = stone[row % len(stone)]
        while x < S:
            w = 13
            if row == 2 and x > 4:
                # the wrong block: a trapezoid, corners disagreeing
                d.polygon([(x, y + 1), (x + w, y - 1), (x + w - 3, y + h),
                           (x + 2, y + h + 1)], fill=col + (255,),
                          outline=(58, 50, 66, 255))
            else:
                d.rectangle([x + 1, y + 1, x + w - 1, y + h - 1],
                            fill=col + (255,), outline=(58, 50, 66, 255))
            # the shadows are wrong: shade falls on the upper-left,
            # against the light every other wall in the world obeys
            d.rectangle([x + 1, y + 1, x + w - 1, y + 2], fill=(104, 92, 116, 255))
            d.rectangle([x + 1, y + 1, x + 2, y + h - 1], fill=(104, 92, 116, 255))
            x += w
        y += h
        row += 1
    # a faint magenta wrongness seeping through the mortar
    for _ in range(9):
        x = rng.uniform(0, S - 1); yy = rng.uniform(0, S - 1)
        d.point((x, yy), fill=(196, 90, 180, 255))
    return img

# ------------------------------------------------------- reinforced glass
def draw_reinforced_glass():
    img = full((166, 206, 202, 255))  # thick pale glass
    d = ImageDraw.Draw(img, "RGBA")
    # the embedded wire mesh: thin steel wires throughout the pane
    wire = (104, 116, 120, 255)
    for x in range(4, S, 6):
        d.line([(x, 0), (x, S - 1)], fill=wire)
    for y in range(4, S, 6):
        d.line([(0, y), (S - 1, y)], fill=wire)
    # glass body shading + one sheen pass
    d.rectangle([0, 0, S - 1, 0], fill=(196, 228, 224, 255))
    d.polygon([(4, 28), (14, 4), (20, 4), (10, 28)], fill=(255, 255, 255, 56))
    d.polygon([(16, 28), (24, 8), (28, 8), (20, 28)], fill=(255, 255, 255, 36))
    # steel frame + corner bolts
    frame = (82, 92, 98, 255)
    d.rectangle([0, 0, S - 1, S - 1], outline=frame, width=2)
    for (x, y) in ((2, 2), (S - 3, 2), (2, S - 3), (S - 3, S - 3)):
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(56, 64, 70, 255))
    return img

# ------------------------------------------------------- wasp paper walls
def _paper_layers(base_bands, fleck_cols, gloss, seed):
    rng = random.Random(seed)
    img = full(base_bands[0] + (255,))
    d = ImageDraw.Draw(img, "RGBA")
    # horizontal nest-paper strata with wavy boundaries
    y = 0
    i = 0
    while y < S:
        h = rng.uniform(3, 5.5)
        col = base_bands[i % len(base_bands)]
        pts_top = [(x, y + 1.2 * ((x * 7 + i * 13) % 5) / 4.0) for x in range(S)]
        d.polygon(pts_top + [(S - 1, y + h), (0, y + h)], fill=col + (255,))
        d.line(pts_top, fill=shade(col, 0.78), width=1)
        y += h
        i += 1
    # pulp flecks
    for _ in range(46):
        x = rng.uniform(0, S - 1); yy = rng.uniform(0, S - 1)
        d.point((x, yy), fill=fleck_cols[rng.randrange(len(fleck_cols))] + (255,))
    # sticky saliva gloss: a few pale shiny patches
    for _ in range(gloss):
        x = rng.uniform(2, S - 8); yy = rng.uniform(2, S - 5)
        d.ellipse([x, yy, x + rng.uniform(3, 6), yy + rng.uniform(2, 3)],
                  fill=(255, 250, 230, 64))
    return img

def draw_paper_brood():
    # t_paper's pulpy wasp-nest wall, brood grade: grayer, glossier
    return _paper_layers(
        [(198, 184, 158), (208, 196, 172), (188, 172, 142)],
        [(168, 152, 120), (222, 212, 190), (150, 134, 104)], gloss=6, seed=3310)

def draw_paper_hard():
    # layered pulp hardened dark: tight, compressed, heavy strata
    img = _paper_layers(
        [(116, 106, 94), (106, 96, 86), (124, 114, 102)],
        [(88, 78, 68), (140, 130, 116), (74, 66, 58)], gloss=2, seed=3311)
    d = ImageDraw.Draw(img)
    # hard compressed edges: extra dark strata lines
    for y in range(3, S, 4):
        d.line([(0, y), (S - 1, y)], fill=(70, 62, 54, 255))
    return img

# ------------------------------------------------------- sunshade pole
def draw_sunshade_pole():
    img = transparent()
    d = ImageDraw.Draw(img, "RGBA")
    # canvas canopy: a trapezoid awning with seams and a shaded underside
    d.polygon([(2, 3), (30, 3), (26, 10), (6, 10)], fill=(208, 188, 150, 255),
              outline=(146, 128, 92, 255))
    d.polygon([(6, 10), (26, 10), (24, 12), (8, 12)], fill=(168, 148, 110, 255))
    for x in (10, 16, 22):
        d.line([(x, 3), (x - 1, 10)], fill=(178, 158, 118, 255))
    d.line([(2, 3), (30, 3)], fill=(232, 214, 176, 255))
    # the wooden pole
    d.rectangle([14, 10, 18, 31], fill=(124, 86, 48, 255))
    d.rectangle([14, 10, 15, 31], fill=(150, 108, 62, 255))
    d.rectangle([17, 10, 18, 31], fill=(96, 64, 34, 255))
    for y in (15, 21, 26):
        d.line([(14, y), (18, y)], fill=(96, 64, 34, 255))
    # guy-lines from canopy corners down to the pole
    d.line([(3, 4), (14, 16)], fill=(90, 78, 60, 255))
    d.line([(29, 4), (18, 16)], fill=(90, 78, 60, 255))
    # ground shadow
    d.ellipse([9, 29, 23, 32], fill=(0, 0, 0, 70))
    return img

# ---------------------------------------------------------------- jobs
JOBS = [(tid, draw_container(base, 3400 + n))
        for n, (tid, base) in enumerate(CONTAINERS)]
JOBS += [
    ("t_wall_warped", draw_warped()),
    ("t_reinforced_glass_lab", draw_reinforced_glass()),
    ("t_paper_brood", draw_paper_brood()),
    ("t_paper_hard", draw_paper_hard()),
    ("t_wood_sunshade_pole", draw_sunshade_pole()),
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
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, img) in enumerate(JOBS):
    t = img.resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_walls.png"))
print("wrote preview_walls.png", sheet.size)
