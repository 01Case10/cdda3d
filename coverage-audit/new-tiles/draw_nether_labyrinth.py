#!/usr/bin/env python3
"""
Draw the missing nether-labyrinth terrain sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/netherum/
terrain_labyrinth_*.json, CDDA 6a3e59e0) -- the terrain ids in
coverage-audit/missing.json (missing_shipped) that start with t_nl_ or
are defined under netherum/, minus the ones already covered:
t_nl_sd_* were drawn in the string-dimension batch, and
t_nl_shrub_glowberries / _harvested resolve via looks_like to
t_alien_meadow_shrub_2 / _harvested (alien-meadow batch), so neither
needs a sprite of its own. 13 ids remain:

  t_nl_broken_metal_floor     "Something has cratered this metal
                               flooring... Through the cracks and dents
                               emanates a faint eery glow."  light_cyan
  t_nl_chasm_no_trap          "sparkling chasm... it might be full of
                               stars. Still deeper, perhaps those stars
                               are eyes?"  symbol *, dark_gray
  t_nl_column                 "scrap metal column: A massive column of
                               angular plates, pipes, and wires."
                               symbol |, dark_gray, WALL
  t_nl_column_light           "protruding light: A bulbous light hangs
                               here amongst the other electronics. Its
                               wires snake into the column..."  white,
                               light_emitted 90
  t_nl_gates_mech_control     "rotary lever: A heavy-duty metal lever
                               with two settings."  symbol 6, red
  t_nl_grass_very_tall        "tangled cybergrass harness: A massive,
                               floor-to-ceiling tangle of wires and...
                               grass? The long blades glow and hum like
                               old, dim fluorescent lights. Something is
                               moving, behind it."  green, light 5, WALL
  t_nl_metal_wall_overgrown   "overgrown metal wall: sheet metal with
                               fractal welds and joins. All manner of
                               strange, plastic-like plants are poking
                               their way through the crevices. The plants
                               glow..."  green, light 5, WALL
  t_nl_obelisk_mission_autodoc / _living_space_1 / _utility_space
                              "Metal stele: A floor to ceiling, shining
                               column. A massive face with a gaping jaw
                               is cast upon it. Inside the mouth are
                               several pre-fabricated slots."  symbol A,
                               green -- all three ids carry identical
                               name/description/symbol/color in the game
                               data, so all three are drawn identically
  t_nl_soil                   "solid earth... full of small, sparkling
                               grains, which seem to twist at the edge
                               of your vision."  symbol #, brown, WALL
  t_nl_suspended_bridge       "walkway: A metallic bridge... The pattern
                               of the tempered, latticed metal..."
                               light_gray, FLAT
  t_slope_to_bosky_labyrinth  "overgrown slope down: A downward-facing
                               slope, so overgrown with brush you can't
                               make out what lied beyond."  brown

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet. Full-tile tiles (floors, walls, chasm, bridge, slope) are fg
with bg ""; the two feature sprites sit on their substrate from the
same batch: the column light over t_nl_column (it hangs on the
column), the rotary lever over t_nl_broken_metal_floor. Flat fills,
dark outlines, one highlight pass. Deterministic: every random
choice comes from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json + credits).
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def full(color):
    return Image.new("RGBA", (S, S), color)

def transparent():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

# ---------------------------------------------------------------- broken metal floor
def draw_broken_metal_floor():
    rng = random.Random(3301)
    img = full((96, 110, 116, 255))  # pale cyan-steel
    d = ImageDraw.Draw(img)
    # plate seams: 2x2 plates with dark gaps + rivets
    for x in (0, 16):
        for y in (0, 16):
            d.rectangle([x, y, x + 15, y + 15], outline=(58, 68, 74, 255))
            for rx, ry in ((x + 2, y + 2), (x + 13, y + 2), (x + 2, y + 13), (x + 13, y + 13)):
                d.ellipse([rx - 1, ry - 1, rx + 1, ry + 1], fill=(140, 158, 164, 255),
                          outline=(58, 68, 74, 255))
    # highlight pass: top edges of the plates
    d.line([(0, 0), (31, 0)], fill=(150, 168, 174, 255))
    d.line([(0, 16), (31, 16)], fill=(150, 168, 174, 255))
    # the crater: a dark dented basin, lower-left of centre
    d.ellipse([6, 12, 22, 27], fill=(52, 60, 66, 255))
    d.ellipse([9, 15, 20, 25], fill=(38, 44, 50, 255))
    d.arc([6, 12, 22, 27], start=180, end=330, fill=(140, 158, 164, 255))
    # cracks radiating out, with the faint eerie glow inside them
    cracks = [((14, 19), (4, 8)), ((14, 19), (2, 20)), ((14, 19), (8, 30)),
              ((14, 19), (24, 28)), ((14, 19), (27, 16)), ((14, 19), (22, 6))]
    for (x0, y0), (x1, y1) in cracks:
        mx, my = (x0 + x1) // 2 + rng.randint(-2, 2), (y0 + y1) // 2 + rng.randint(-2, 2)
        d.line([(x0, y0), (mx, my), (x1, y1)], fill=(30, 34, 38, 255), width=1)
        d.line([(x0, y0), (mx, my)], fill=(126, 226, 238, 255), width=1)
    # scattered small dents
    for _ in range(7):
        x = rng.uniform(1, S - 4); y = rng.uniform(1, S - 4)
        d.ellipse([x, y, x + 2.5, y + 1.8], fill=(66, 76, 82, 255))
    return img

# ---------------------------------------------------------------- chasm
def draw_chasm():
    rng = random.Random(3302)
    img = full((7, 7, 11, 255))  # darkness
    d = ImageDraw.Draw(img)
    # stars: many faint specks, a few brighter
    for _ in range(70):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        b = rng.choice([70, 95, 125, 165])
        d.point((x, y), fill=(b, b, b + 12, 255))
    for _ in range(5):
        x = rng.randint(3, S - 4); y = rng.randint(3, S - 4)
        d.line([(x - 2, y), (x + 2, y)], fill=(200, 200, 215, 255))
        d.line([(x, y - 2), (x, y + 2)], fill=(200, 200, 215, 255))
        d.point((x, y), fill=(245, 245, 250, 255))
    # ...or are they eyes? three faint pairs, low in the dark
    for cx, cy in ((9, 21), (21, 25), (15, 13)):
        d.point((cx, cy), fill=(112, 108, 84, 255))
        d.point((cx + 3, cy), fill=(112, 108, 84, 255))
    return img

# ---------------------------------------------------------------- scrap column
def draw_column():
    rng = random.Random(3303)
    img = full((44, 46, 52, 255))
    d = ImageDraw.Draw(img)
    # the column mass: a broad central band of angular plates
    d.rectangle([4, 0, 27, 31], fill=(66, 70, 78, 255))
    plates = [(4, 0, 15, 10), (16, 0, 27, 7), (4, 11, 12, 21), (13, 8, 27, 17),
              (4, 22, 18, 31), (19, 18, 27, 31)]
    fills = [(74, 78, 88, 255), (60, 64, 72, 255), (80, 84, 94, 255),
             (56, 60, 68, 255), (70, 74, 84, 255), (62, 66, 74, 255)]
    for (x0, y0, x1, y1), f in zip(plates, fills):
        d.polygon([(x0, y0), (x1, y0), (x1 - 2, y1), (x0, y1)], fill=f,
                  outline=(26, 28, 32, 255))
    # pipes running down the face
    for px in (8, 23):
        d.line([(px, 0), (px, 31)], fill=(34, 36, 42, 255), width=3)
        d.line([(px - 1, 0), (px - 1, 31)], fill=(104, 110, 120, 255), width=1)
        for by in (6, 17, 27):
            d.rectangle([px - 2, by, px + 2, by + 2], fill=(48, 50, 58, 255),
                        outline=(26, 28, 32, 255))
    # wires snaking across
    for off in (0, 5):
        pts = []
        for x in range(4, 28, 3):
            pts.append((x, 12 + off + 3 * math.sin(x / 4.0 + off)))
        d.line(pts, fill=(22, 22, 26, 255), width=1)
    # bolts
    for _ in range(10):
        x = rng.uniform(5, 26); y = rng.uniform(1, 30)
        d.point((x, y), fill=(120, 126, 136, 255))
    return img

# ---------------------------------------------------------------- protruding light
def draw_column_light():
    rng = random.Random(3304)
    img = transparent()
    d = ImageDraw.Draw(img, "RGBA")
    # fractal clump of mechanical devices across the top
    for i in range(7):
        x = 2 + i * 4 + rng.randint(-1, 1)
        d.rectangle([x, 0, x + 3, rng.randint(3, 6)], fill=(52, 55, 62, 255),
                    outline=(24, 26, 30, 255))
        d.point((x + 1, 1), fill=(150, 60, 60, 255))
    # wires snaking from the clump down to the bulb and off right,
    # vanishing into the column behind
    pts = [(16, 4)]
    for k in range(1, 6):
        pts.append((16 + int(3 * math.sin(k * 1.2)), 4 + k * 1.6))
    d.line(pts, fill=(26, 27, 31, 255), width=1)
    d.line([(20, 6), (28, 10), (31, 10)], fill=(26, 27, 31, 255), width=1)
    d.line([(12, 5), (5, 9), (0, 9)], fill=(26, 27, 31, 255), width=1)
    # glow halo, then the bulbous light itself
    for r, a in ((11, 36), (8, 64), (6, 96)):
        d.ellipse([16 - r, 17 - r, 16 + r, 17 + r], fill=(250, 240, 190, a))
    d.ellipse([11, 12, 21, 22], fill=(252, 246, 205, 255), outline=(120, 110, 70, 255))
    d.ellipse([13, 14, 18, 19], fill=(255, 255, 240, 255))
    d.point((14, 15), fill=(255, 255, 255, 255))
    return img

# ---------------------------------------------------------------- rotary lever
def draw_lever():
    img = transparent()
    d = ImageDraw.Draw(img)
    # heavy-duty base plate
    d.rounded_rectangle([6, 22, 26, 29], radius=2, fill=(74, 78, 88, 255),
                        outline=(40, 42, 50, 255))
    # the two settings: plain notch marks, the engaged one in red
    d.line([(9, 27), (9, 24)], fill=(30, 32, 38, 255), width=2)
    d.line([(23, 27), (23, 24)], fill=(198, 52, 48, 255), width=2)
    # pivot + angled lever arm + red knob
    d.line([(16, 24), (23, 10)], fill=(52, 55, 64, 255), width=3)
    d.ellipse([21, 6, 26, 11], fill=(178, 46, 42, 255), outline=(96, 26, 24, 255))
    d.ellipse([14, 21, 18, 25], fill=(96, 100, 112, 255), outline=(40, 42, 50, 255))
    return img

# ---------------------------------------------------------------- cybergrass tangle
def draw_cybergrass():
    rng = random.Random(3306)
    img = full((16, 22, 16, 255))
    d = ImageDraw.Draw(img)
    # something moving behind it: a vague dark bulk in the middle
    d.ellipse([9, 7, 23, 27], fill=(9, 12, 9, 255))
    d.ellipse([12, 4, 20, 12], fill=(9, 12, 9, 255))
    blade_cols = [(52, 128, 58, 255), (74, 168, 74, 255), (40, 102, 48, 255),
                  (96, 198, 96, 255)]
    # floor-to-ceiling blades, rooted top and bottom, sagging between
    for _ in range(46):
        x = rng.uniform(0, S - 1)
        col = blade_cols[rng.randrange(len(blade_cols))]
        sway = rng.uniform(-3, 3)
        pts = [(x, 31), (x + sway * 0.4, 20), (x + sway, 10), (x + sway * 0.5, 0)]
        d.line(pts, fill=col, width=1)
        # glowing tip, like a dim fluorescent tube end
        if rng.random() < 0.55:
            d.point((x + sway * 0.5, 1), fill=(178, 240, 178, 255))
    # the wires in the tangle: dark cords crossing horizontally
    for _ in range(9):
        y = rng.uniform(2, 30)
        pts = [(0, y)]
        for x in range(4, 33, 4):
            pts.append((x, y + rng.uniform(-2.5, 2.5)))
        d.line(pts, fill=(10, 12, 10, 255), width=1)
    # sparse glow flecks along the blades
    for _ in range(16):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=(150, 226, 150, 255))
    return img

# ---------------------------------------------------------------- overgrown metal wall
def draw_overgrown_wall():
    rng = random.Random(3307)
    img = full((86, 92, 98, 255))  # sheet metal
    d = ImageDraw.Draw(img)
    # panels + fractal weld beads along the seams
    d.line([(0, 15), (31, 15)], fill=(52, 57, 63, 255), width=1)
    d.line([(15, 0), (15, 31)], fill=(52, 57, 63, 255), width=1)
    d.line([(0, 0), (31, 0)], fill=(128, 136, 144, 255))
    for x in range(2, 32, 3):
        d.point((x, 15), fill=(150, 158, 166, 255))
        d.point((x + 1, 16), fill=(110, 118, 126, 255))
    for y in range(2, 32, 3):
        d.point((15, y), fill=(150, 158, 166, 255))
    # crevice shadow pockets
    for (x, y, w, h) in ((3, 4, 7, 3), (21, 9, 8, 3), (6, 22, 6, 3), (24, 24, 6, 4)):
        d.rectangle([x, y, x + w, y + h], fill=(58, 63, 69, 255))
    # plastic-like plants poking through the crevices, faintly glowing
    tufts = [(6, 5), (25, 10), (9, 23), (27, 25), (16, 15), (2, 28), (29, 3)]
    for cx, cy in tufts:
        for dx, dy, ln in ((-2, -3, 3), (0, -4, 4), (2, -3, 3), (-1, 2, 2), (2, 1, 2)):
            d.line([(cx, cy), (cx + dx, cy + dy)], fill=(64, 158, 78, 255), width=1)
            d.point((cx + dx, cy + dy), fill=(168, 238, 168, 255))
        d.point((cx, cy), fill=(120, 210, 120, 255))
    return img

# ---------------------------------------------------------------- metal stele (obelisk)
def draw_obelisk():
    img = full((30, 36, 30, 255))
    d = ImageDraw.Draw(img)
    # the shining floor-to-ceiling column, green-tinted steel
    d.rectangle([5, 0, 26, 31], fill=(104, 126, 106, 255))
    d.rectangle([5, 0, 9, 31], fill=(140, 164, 140, 255))   # shine stripe
    d.rectangle([23, 0, 26, 31], fill=(70, 88, 72, 255))     # shaded edge
    d.line([(5, 0), (5, 31)], fill=(34, 44, 34, 255))
    d.line([(26, 0), (26, 31)], fill=(34, 44, 34, 255))
    # the massive cast face: heavy brow, deep eye sockets, nose ridge
    d.rectangle([9, 5, 22, 8], fill=(58, 74, 58, 255))      # brow
    d.ellipse([10, 9, 14, 13], fill=(16, 20, 16, 255))      # left eye
    d.ellipse([17, 9, 21, 13], fill=(16, 20, 16, 255))      # right eye
    d.polygon([(15, 12), (17, 12), (16, 17)], fill=(84, 104, 84, 255),
              outline=(40, 52, 40, 255))                    # nose
    # the gaping jaw: a wide open mouth, slots pre-fabricated inside
    d.ellipse([9, 18, 22, 29], fill=(12, 14, 12, 255))
    d.arc([9, 18, 22, 29], start=0, end=360, fill=(150, 174, 150, 255))
    for i, sx in enumerate((11, 14, 17)):
        d.rectangle([sx, 22, sx + 2, 26], fill=(168, 190, 168, 255),
                    outline=(60, 76, 60, 255))
    # chin shadow under the jaw
    d.arc([10, 24, 21, 31], start=20, end=160, fill=(52, 66, 52, 255))
    return img

# ---------------------------------------------------------------- solid earth
def draw_soil():
    rng = random.Random(3309)
    img = full((94, 70, 46, 255))  # brown earth
    d = ImageDraw.Draw(img)
    # clods and strata
    for _ in range(26):
        x = rng.uniform(0, S - 5); y = rng.uniform(0, S - 4)
        w = rng.uniform(3, 6); h = rng.uniform(2, 3.5)
        col = rng.choice([(80, 58, 38, 255), (108, 82, 54, 255), (70, 52, 34, 255)])
        d.ellipse([x, y, x + w, y + h], fill=col)
    d.line([(0, 10), (31, 12)], fill=(74, 54, 36, 255))
    d.line([(0, 22), (31, 20)], fill=(74, 54, 36, 255))
    # sparkling grains, some with a tiny twist of light
    for _ in range(34):
        x = rng.randint(1, S - 2); y = rng.randint(1, S - 2)
        d.point((x, y), fill=(238, 230, 190, 255))
        if rng.random() < 0.3:
            d.point((x + 1, y), fill=(200, 190, 140, 255))
            d.point((x, y + 1), fill=(200, 190, 140, 255))
    return img

# ---------------------------------------------------------------- suspended bridge
def draw_bridge():
    img = full((148, 152, 158, 255))  # light gray metal
    d = ImageDraw.Draw(img)
    # tempered lattice: diamond cross-hatch over the whole walkway
    for k in range(-4, 9):
        d.line([(k * 8, 0), (k * 8 + 32, 32)], fill=(98, 102, 108, 255), width=2)
        d.line([(k * 8, 32), (k * 8 + 32, 0)], fill=(98, 102, 108, 255), width=2)
    for k in range(-4, 9):
        d.line([(k * 8, 0), (k * 8 + 32, 32)], fill=(188, 192, 198, 255), width=1)
    # edge rails with rivets
    for y in (0, 29):
        d.rectangle([0, y, 31, y + 2], fill=(110, 114, 120, 255))
        d.line([(0, y), (31, y)], fill=(196, 200, 206, 255))
        for x in range(3, 32, 6):
            d.point((x, y + 1), fill=(70, 74, 80, 255))
    return img

# ---------------------------------------------------------------- overgrown slope
def draw_slope():
    rng = random.Random(3311)
    img = full((26, 22, 18, 255))  # the unseen beyond, in shadow
    d = ImageDraw.Draw(img)
    # the slope: an earth wedge descending left (high) to right (low)
    d.polygon([(0, 6), (31, 18), (31, 31), (0, 31)], fill=(98, 74, 48, 255))
    d.polygon([(0, 6), (31, 18), (31, 22), (0, 10)], fill=(112, 86, 56, 255))
    # lip shadow under the crest
    d.line([(0, 10), (31, 22)], fill=(52, 38, 26, 255), width=2)
    # brush overgrowing the crest and spilling down the face
    brush_cols = [(58, 104, 44, 255), (76, 130, 54, 255), (44, 84, 36, 255)]
    for _ in range(30):
        t = rng.random()
        cx = t * 31; cy = 6 + t * 12 + rng.uniform(-2, 3)
        col = brush_cols[rng.randrange(len(brush_cols))]
        for dx, dy in ((-2, -2), (0, -3), (2, -2), (-1, 1), (1, 0)):
            d.line([(cx, cy), (cx + dx, cy + dy)], fill=col, width=1)
        d.point((cx, cy - 3), fill=(120, 176, 92, 255))
    for _ in range(10):
        x = rng.uniform(2, 30); y = rng.uniform(16, 29)
        d.point((x, y), fill=brush_cols[rng.randrange(len(brush_cols))])
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg) -- feature sprites sit on their substrate
# from this batch; everything else is a full tile.
FLOOR = "t_nl_broken_metal_floor"
COLUMN = "t_nl_column"
JOBS = [
    ("t_nl_broken_metal_floor", draw_broken_metal_floor(), "t_nl_broken_metal_floor", ""),
    ("t_nl_chasm_no_trap", draw_chasm(), "t_nl_chasm_no_trap", ""),
    ("t_nl_column", draw_column(), "t_nl_column", ""),
    ("t_nl_column_light", draw_column_light(), "t_nl_column_light", COLUMN),
    ("t_nl_gates_mech_control", draw_lever(), "t_nl_gates_mech_control", FLOOR),
    ("t_nl_grass_very_tall", draw_cybergrass(), "t_nl_grass_very_tall", ""),
    ("t_nl_metal_wall_overgrown", draw_overgrown_wall(), "t_nl_metal_wall_overgrown", ""),
    ("t_nl_obelisk_mission_autodoc", draw_obelisk(), "t_nl_obelisk_mission_autodoc", ""),
    ("t_nl_obelisk_mission_living_space_1", draw_obelisk(), "t_nl_obelisk_mission_living_space_1", ""),
    ("t_nl_obelisk_mission_utility_space", draw_obelisk(), "t_nl_obelisk_mission_utility_space", ""),
    ("t_nl_soil", draw_soil(), "t_nl_soil", ""),
    ("t_nl_suspended_bridge", draw_bridge(), "t_nl_suspended_bridge", ""),
    ("t_slope_to_bosky_labyrinth", draw_slope(), "t_slope_to_bosky_labyrinth", ""),
]

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img, fg, bg in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": fg, "bg": bg}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

def composited(tid, bg):
    """The tile as the game shows it: fg over its bg."""
    fg = tile_img(tid)
    if not bg:
        return fg
    out = tile_img(bg).copy()
    out.alpha_composite(fg)
    return out

CELL = 64
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img, _fg, bg) in enumerate(JOBS):
    t = composited(tid, bg).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_nether_labyrinth.png"))
print("wrote preview_nether_labyrinth.png", sheet.size)
