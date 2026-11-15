#!/usr/bin/env python3
"""
Draw the missing string-dimension terrain sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/
terrain-string-dimension.json, CDDA 6a3e59e0) -- 20 ids in that file are
missing from UltiCa in the shipped universe (coverage-audit/missing.json);
the file's other ids resolve via looks_like (t_string_grass_alien,
t_string_distress_beacon, t_string_dimension_strconc_red_light,
t_string_dimension_meteoric_rock) or are the charred rock's copy-from
parent chain, and t_string_dimension_groundcover is a regional pseudo
("should never actually show up") and stays out of this batch:

  t_string_void               "an otherworldly nothing... a black wall at
                               the front of your senses"  color black, WALL
  t_string_true_void          same nothing, opaque WALL variant -- the two
                               voids are indistinguishable by description
  t_string_point_small        "A speck in the darkness."  symbol ., white
  t_string_point_big          "A speck in the darkness."  symbol *, white
  t_string_point_small_gray   "A speck in the darkness."  symbol ., light_gray
  t_string_point_big_gray     "A speck in the darkness."  symbol *, light_gray
  t_string_x                  "impossibly thin magenta line... the space
                               around it shimmers"  FLAT/ROAD, ONE_DIMENSIONAL_X
  t_string_y                  "impossibly thin cyan line"  ONE_DIMENSIONAL_Y
  t_string_z                  "impossibly thin yellow line that extends up
                               and down"  ONE_DIMENSIONAL_Z, GOES_UP/DOWN
  t_string_xy                 "impossibly flat blue square"  FLAT/ROAD
  t_string_xy_plane_1         identical name/description to t_string_xy
  t_string_xy_plane_2         "shimmering blue square"
  t_string_xy_plane_3         "wavy square", light_blue
  t_string_xyz                "white cube... it somehow reminds you of home"
  t_nl_sd_2d_sky              "the sky... impossibly flattened into two
                               dimensions"  color black
  t_nl_sd_2d_sky_walkable     "You can stand here, upon the horizon."
  t_nl_sd_2d_ground           "the ground... impossibly flattened... patterns
                               constantly shifting, like an image passing
                               through a veined rock"  color light_gray
  t_string_gates_mech_control "A heavy-duty metal lever with two settings,
                               each marked by an alien symbol."  cyan_red
  t_string_gates_mech_control_inert  the same lever; "Toggling it back and
                               forth does nothing." -- drawn dimmed/dead
  t_string_dimension_meteoric_rock_charred
                              "solid rock, deeply pitted, charred, and coated
                               in a residue with an acrid smell" (copy-from
                               t_rock; the uncharred sibling resolves via
                               looks_like t_rock, this one has none)

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet.
Full-tile tiles (voids, points, planes, sky, ground, charred rock) are
fg with bg ""; the perceived geometry (x/y/z lines, the xyz cube) and
the lever are transparent fg over bg "t_string_void" -- they are things
perceived against the nothing. Deterministic: every random choice comes
from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json + credits).
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

BLACK = (0, 0, 0, 255)

def full(color):
    return Image.new("RGBA", (S, S), color)

def transparent():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

# ---------------------------------------------------------------- voids
def draw_void():
    return full(BLACK)

def draw_true_void():
    # Indistinguishable from t_string_void by its description.
    return full(BLACK)

# ---------------------------------------------------------------- points
def draw_point(big, gray):
    img = full(BLACK)
    d = ImageDraw.Draw(img)
    col = (168, 168, 178, 255) if gray else (245, 245, 250, 255)
    cx = cy = 16
    if big:
        # star speck: plus arms + bright core
        d.line([(cx - 3, cy), (cx + 3, cy)], fill=col)
        d.line([(cx, cy - 3), (cx, cy + 3)], fill=col)
        d.point((cx - 2, cy - 2), fill=col)
        d.point((cx + 2, cy + 2), fill=col)
        d.rectangle([cx - 1, cy - 1, cx, cy], fill=(255, 255, 255, 255) if not gray else col)
    else:
        d.rectangle([cx - 1, cy - 1, cx, cy], fill=col)
    return img

# ---------------------------------------------------------------- lines
def _shimmer(d, rng, horizontal, col):
    # faint offset ghosts around the line: "the space around it shimmers"
    ghost = col[:3] + (52,)
    off = -2 if horizontal else -2
    for _ in range(10):
        p = rng.uniform(0, S - 1)
        ln = rng.uniform(2, 6)
        if horizontal:
            d.line([(p, 16 + off), (p + ln, 16 + off)], fill=ghost)
            d.line([(p, 16 - off), (p + ln, 16 - off)], fill=ghost)
        else:
            d.line([(16 + off, p), (16 + off, p + ln)], fill=ghost)
            d.line([(16 - off, p), (16 - off, p + ln)], fill=ghost)

def draw_line_x():
    rng = random.Random(2201)
    img = transparent()
    d = ImageDraw.Draw(img, "RGBA")
    _shimmer(d, rng, True, (255, 0, 255, 255))
    d.line([(0, 16), (S - 1, 16)], fill=(255, 0, 255, 255))
    return img

def draw_line_y():
    rng = random.Random(2202)
    img = transparent()
    d = ImageDraw.Draw(img, "RGBA")
    _shimmer(d, rng, False, (0, 255, 255, 255))
    d.line([(16, 0), (16, S - 1)], fill=(0, 255, 255, 255))
    return img

def draw_line_z():
    rng = random.Random(2203)
    img = transparent()
    d = ImageDraw.Draw(img, "RGBA")
    _shimmer(d, rng, False, (255, 235, 60, 255))
    d.line([(16, 0), (16, S - 1)], fill=(255, 235, 60, 255))
    # it extends up and down beyond the tile: bright caps at both ends
    d.rectangle([14, 0, 18, 2], fill=(255, 245, 140, 255))
    d.rectangle([14, S - 3, 18, S - 1], fill=(255, 245, 140, 255))
    return img

# ---------------------------------------------------------------- planes
def draw_plane_xy():
    rng = random.Random(2210)
    img = full((38, 56, 214, 255))
    d = ImageDraw.Draw(img)
    # faint shimmer: sparse lighter flecks on the flat square
    for _ in range(26):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=(86, 108, 238, 255))
    return img

def draw_plane_shimmer():
    rng = random.Random(2212)
    img = full((38, 56, 214, 255))
    d = ImageDraw.Draw(img, "RGBA")
    # diagonal sheen bands sweeping the square
    for k in range(-2, 6):
        x0 = k * 8
        d.polygon([(x0, 32), (x0 + 4, 32), (x0 + 20, 0), (x0 + 16, 0)],
                  fill=(120, 148, 255, 64))
    for _ in range(14):
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        d.point((x, y), fill=(210, 224, 255, 255))
    return img

def draw_plane_wavy():
    img = full((104, 148, 236, 255))  # light_blue
    d = ImageDraw.Draw(img)
    # wavy horizontal bands: the square is "waving"
    for row in range(0, S, 4):
        amp = 1.6
        pts = []
        for x in range(S):
            y = row + amp * math.sin(x / 5.0 + row)
            pts.append((x, y))
        d.line(pts, fill=(74, 116, 208, 255), width=1)
        pts2 = [(x, y + 2) for (x, y) in pts]
        d.line(pts2, fill=(150, 182, 248, 255), width=1)
    return img

# ---------------------------------------------------------------- cube
def draw_xyz():
    img = transparent()
    d = ImageDraw.Draw(img)
    # isometric white cube floating in the nothing
    top = [(16, 5), (26, 10), (16, 15), (6, 10)]
    left = [(6, 10), (16, 15), (16, 26), (6, 21)]
    right = [(16, 15), (26, 10), (26, 21), (16, 26)]
    d.polygon(right, fill=(178, 178, 186, 255))
    d.polygon(left, fill=(214, 214, 222, 255))
    d.polygon(top, fill=(250, 250, 252, 255))
    d.line(top + [top[0]], fill=(120, 120, 130, 255))
    d.line(left + [left[0]], fill=(120, 120, 130, 255))
    d.line(right + [right[0]], fill=(120, 120, 130, 255))
    return img

# ---------------------------------------------------------------- flattened sky / ground
def draw_2d_sky():
    img = full((5, 7, 18, 255))
    d = ImageDraw.Draw(img)
    # the whole sky squashed into one thin pale band
    for i, col in enumerate([(52, 66, 104, 255), (96, 116, 156, 255),
                             (150, 172, 202, 255), (96, 116, 156, 255),
                             (52, 66, 104, 255)]):
        d.rectangle([0, 13 + i, S - 1, 13 + i], fill=col)
    return img

def draw_2d_sky_walkable():
    img = full((5, 7, 18, 255))
    d = ImageDraw.Draw(img)
    # the horizon as a floor: a flat pale band you can stand upon
    d.rectangle([0, 21, S - 1, 21], fill=(168, 188, 214, 255))
    d.rectangle([0, 22, S - 1, S - 1], fill=(110, 128, 164, 255))
    d.rectangle([0, 26, S - 1, S - 1], fill=(88, 104, 140, 255))
    return img

def draw_2d_ground():
    rng = random.Random(2230)
    img = full((168, 168, 172, 255))  # light_gray
    d = ImageDraw.Draw(img)
    # veined rock: wandering darker veins, an image through stone
    vein_cols = [(128, 128, 134, 255), (146, 146, 150, 255), (110, 110, 118, 255)]
    for v in range(9):
        col = vein_cols[v % len(vein_cols)]
        x = rng.uniform(-4, S); y = rng.uniform(0, S)
        pts = [(x, y)]
        for _ in range(7):
            x += rng.uniform(2, 6); y += rng.uniform(-4, 4)
            pts.append((x, y))
        d.line(pts, fill=col, width=1)
    for _ in range(40):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=(188, 188, 192, 255))
    return img

# ---------------------------------------------------------------- lever
def _draw_lever(dim):
    img = transparent()
    d = ImageDraw.Draw(img)
    def c(rgb):
        if dim:
            g = int(0.55 * (0.3 * rgb[0] + 0.5 * rgb[1] + 0.2 * rgb[2]))
            return (g, g, g + 6, 255)
        return rgb + (255,)
    # heavy-duty base plate
    d.rounded_rectangle([6, 22, 26, 29], radius=2, fill=c((74, 78, 88)),
                        outline=c((40, 42, 50)))
    # the two settings, each marked by an alien symbol (cyan / red)
    d.polygon([(10, 27), (13, 24), (14, 27)], outline=c((0, 229, 229)))
    d.ellipse([19, 24, 22, 27], outline=c((229, 60, 60)))
    # pivot + angled lever arm + knob
    d.line([(16, 24), (23, 10)], fill=c((52, 55, 64)), width=3)
    d.ellipse([21, 6, 26, 11], fill=c((150, 44, 44)), outline=c((84, 26, 26)))
    d.ellipse([14, 21, 18, 25], fill=c((96, 100, 112)), outline=c((40, 42, 50)))
    return img

def draw_lever():
    return _draw_lever(False)

def draw_lever_inert():
    return _draw_lever(True)

# ---------------------------------------------------------------- charred rock
def draw_charred_rock():
    rng = random.Random(2240)
    img = full((40, 36, 33, 255))
    d = ImageDraw.Draw(img)
    # rock body shading
    for _ in range(30):
        x = rng.uniform(0, S - 4); y = rng.uniform(0, S - 4)
        w = rng.uniform(3, 7); h = rng.uniform(2, 4)
        d.ellipse([x, y, x + w, y + h], fill=(52, 47, 43, 255))
    # deep pits
    for _ in range(16):
        x = rng.uniform(0, S - 3); y = rng.uniform(0, S - 3)
        w = rng.uniform(2, 4.5); h = rng.uniform(1.5, 3.5)
        d.ellipse([x, y, x + w, y + h], fill=(10, 9, 9, 255))
    # charred patches: near-black scorch blotches
    for _ in range(8):
        x = rng.uniform(0, S - 6); y = rng.uniform(0, S - 6)
        w = rng.uniform(4, 8); h = rng.uniform(3, 6)
        d.ellipse([x, y, x + w, y + h], fill=(16, 14, 13, 255))
    # acrid residue: dull olive streaks running down
    for _ in range(6):
        x = rng.uniform(1, S - 2); y = rng.uniform(0, S - 8)
        ln = rng.uniform(5, 9)
        d.line([(x, y), (x + rng.uniform(-1, 1), y + ln)],
               fill=(96, 90, 44, 255), width=1)
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg) -- the perceived geometry and the lever sit
# on the void (bg t_string_void); everything else is a full tile.
JOBS = [
    ("t_string_void", draw_void(), "t_string_void", ""),
    ("t_string_true_void", draw_true_void(), "t_string_true_void", ""),
    ("t_string_point_small", draw_point(False, False), "t_string_point_small", ""),
    ("t_string_point_big", draw_point(True, False), "t_string_point_big", ""),
    ("t_string_point_small_gray", draw_point(False, True), "t_string_point_small_gray", ""),
    ("t_string_point_big_gray", draw_point(True, True), "t_string_point_big_gray", ""),
    ("t_string_x", draw_line_x(), "t_string_x", "t_string_void"),
    ("t_string_y", draw_line_y(), "t_string_y", "t_string_void"),
    ("t_string_z", draw_line_z(), "t_string_z", "t_string_void"),
    ("t_string_xy", draw_plane_xy(), "t_string_xy", ""),
    ("t_string_xy_plane_1", draw_plane_xy(), "t_string_xy_plane_1", ""),
    ("t_string_xy_plane_2", draw_plane_shimmer(), "t_string_xy_plane_2", ""),
    ("t_string_xy_plane_3", draw_plane_wavy(), "t_string_xy_plane_3", ""),
    ("t_string_xyz", draw_xyz(), "t_string_xyz", "t_string_void"),
    ("t_nl_sd_2d_sky", draw_2d_sky(), "t_nl_sd_2d_sky", ""),
    ("t_nl_sd_2d_sky_walkable", draw_2d_sky_walkable(), "t_nl_sd_2d_sky_walkable", ""),
    ("t_nl_sd_2d_ground", draw_2d_ground(), "t_nl_sd_2d_ground", ""),
    ("t_string_gates_mech_control", draw_lever(), "t_string_gates_mech_control", "t_string_void"),
    ("t_string_gates_mech_control_inert", draw_lever_inert(), "t_string_gates_mech_control_inert", "t_string_void"),
    ("t_string_dimension_meteoric_rock_charred", draw_charred_rock(), "t_string_dimension_meteoric_rock_charred", ""),
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

# ---------------------------------------------------------------- previews
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

void = tile_img("t_string_void")

def composited(tid, bg):
    """The tile as the game shows it: fg over its bg."""
    fg = tile_img(tid)
    if not bg:
        return fg
    out = void.copy()
    out.alpha_composite(fg)
    return out

CELL = 64
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img, _fg, bg) in enumerate(JOBS):
    t = composited(tid, bg).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_string_dimension.png"))
print("wrote preview_string_dimension.png", sheet.size)

# combined preview: this batch on top, then the alien-meadow sheet, then
# the monster block (rebuilt exactly as draw_alien_meadow.py built it).
alien = Image.open(os.path.join(BASE, "preview_alien_meadow.png")).convert("RGBA")
monsters = [
    "pngs_normal_32x32/monsters/mon_butterfly/mon_butterfly.png",
    "pngs_normal_32x32/monsters/mon_moth/mon_moth.png",
    "pngs_large_64x64/monsters/mon_butterfly_giant/mon_butterfly_giant.png",
    "pngs_large_64x64/monsters/mon_moth_giant/mon_moth_giant.png",
    "pngs_large_64x64/monsters/mon_caterpillar_butterfly/mon_caterpillar_butterfly.png",
    "pngs_large_64x64/monsters/mon_caterpillar_moth/mon_caterpillar_moth.png",
    "pngs_large_64x64/monsters/mon_butterfly_cocoon/mon_butterfly_cocoon.png",
    "pngs_large_64x64/monsters/mon_moth_cocoon/mon_moth_cocoon.png",
    "pngs_giant_96x96/monsters/mon_butterfly_titan/mon_butterfly_titan.png",
]
MW = 4 * CELL
mons = Image.new("RGBA", (MW, 224), (96, 148, 60, 255))
x = y = 0; rowh = 0
for rel in monsters:
    im = Image.open(os.path.join(BASE, rel)).convert("RGBA")
    if x + im.width > MW:
        x = 0; y += rowh; rowh = 0
    sh = Image.new("RGBA", im.size, (0, 0, 0, 90))
    mons.alpha_composite(sh, (x + 3, y + 5))
    mons.alpha_composite(im, (x, y))
    x += im.width; rowh = max(rowh, im.height)
W = max(sheet.width, alien.width, mons.width)
combined = Image.new("RGBA", (W, sheet.height + alien.height + mons.height),
                     (24, 24, 28, 255))
combined.alpha_composite(sheet, (0, 0))
combined.alpha_composite(alien, (0, sheet.height))
combined.alpha_composite(mons, (0, sheet.height + alien.height))
combined.save(os.path.join(BASE, "preview.png"))
print("wrote preview.png", combined.size)
