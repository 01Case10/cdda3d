#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch OM1: 48 overmap terrain tiles.

Overmap tiles are map cells: every sprite is a FULL-TILE OPAQUE 32x32 --
a ground/base fill in the location's data color with a centered iconic
pictogram (building footprint, ant chamber contents, tentacle, cave
mouth, tree rows). *_roof ids are the roof plan of their base building:
same footprint and palette, roof texture instead of the ground icon.
Drawn from each entry's name/symbol/color in fork-src/cdda/data/json
(overmap/overmap_terrain/*.json). Flat UltiCa overmap style.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_om1.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "overmap")
CREDITS = ("Drawn from the overmap terrain's in-game description for the "
           "CDDA Voxel coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")

OUT = (30, 28, 30, 255)            # dark outline
GRASS = (86, 140, 58, 255)
GRASS_D = (62, 108, 42, 255)
GRASS_L = (112, 170, 78, 255)
IGREEN = (98, 164, 66, 255)        # i_green ground
LGREEN = (140, 190, 100, 255)      # light_green ground
EARTH = (122, 86, 54, 255)
EARTH_D = (88, 60, 38, 255)
EARTH_L = (152, 112, 76, 255)
URBAN = (130, 130, 134, 255)
URBAN_D = (102, 102, 106, 255)
URBAN_L = (158, 158, 162, 255)
CONCRETE = (168, 168, 164, 255)
CONCRETE_D = (128, 128, 124, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (106, 72, 40, 255)
ROOF_RED = (158, 62, 48, 255)
ROOF_RED_D = (120, 44, 36, 255)
ROOF_GRAY = (116, 116, 120, 255)
FLESH = (216, 120, 108, 255)
FLESH_L = (238, 156, 140, 255)
FLESH_D = (178, 84, 76, 255)
FLESH_XD = (138, 56, 52, 255)
RED = (178, 44, 36, 255)
DARKRED = (128, 30, 26, 255)
YELLOW = (232, 202, 66, 255)
IYELLOW = (246, 218, 96, 255)
CREAM = (240, 232, 196, 255)
WHITE = (236, 236, 236, 255)
LGRAY = (178, 178, 178, 255)
PINK = (240, 156, 186, 255)
IPINK = (246, 178, 204, 255)
CYAN = (64, 192, 204, 255)
ICYAN = (128, 226, 236, 255)
LCYAN = (146, 222, 232, 255)
LBLUE = (112, 162, 232, 255)
BLUE = (62, 92, 202, 255)
ACID = (168, 224, 60, 255)         # acid-ant tint
ACID_D = (120, 168, 36, 255)
GREEN_FOOD = (86, 168, 66, 255)
GREEN_FOOD_D = (56, 128, 44, 255)
BLACK = (16, 14, 16, 255)


def ground(rng, base, speck, n=46, box=(0, 0, 31, 31)):
    img = Image.new("RGBA", (32, 32), base)
    d = ImageDraw.Draw(img)
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=rng.choice(speck))
    return img, d


def tree(d, x, y, r=3, canopy=GRASS_D, hi=GRASS_L):
    d.rectangle([x, y + r - 1, x, y + r + 1], fill=WOOD_D)
    d.ellipse([x - r, y - r, x + r, y + r], fill=canopy)
    d.ellipse([x - r + 1, y - r, x + r - 1, y + 1], fill=hi)


def tree_rows(d, rng, rows=((8, 8), (16, 8), (24, 8), (8, 16), (16, 16),
                            (24, 16), (8, 24), (16, 24), (24, 24)),
              canopy=GRASS_D, hi=GRASS_L):
    for (x, y) in rows:
        tree(d, x + rng.randint(-1, 1), y + rng.randint(-1, 1), 3,
             canopy, hi)


def building(d, rect, wall, wall_d=None, door=True):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=wall, outline=OUT, width=1)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2],
                outline=wall_d or OUT)
    if door:
        cx = (x0 + x1) // 2
        d.rectangle([cx - 1, y1 - 3, cx + 1, y1], fill=OUT)


def roof_plan(d, rect, base, dark, ridge=True, vents=()):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=base, outline=OUT, width=1)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=dark)
    if ridge:
        cy = (y0 + y1) // 2
        d.line([(x0 + 2, cy), (x1 - 2, cy)], fill=dark, width=1)
        for x in range(x0 + 4, x1 - 2, 4):
            d.line([(x, y0 + 2), (x, cy)], fill=dark)
            d.line([(x, cy), (x, y1 - 2)], fill=dark)
    for (vx, vy) in vents:
        d.rectangle([vx - 1, vy - 1, vx + 1, vy + 1], fill=dark,
                    outline=OUT)


def ant_chamber(rng, contents, acid=False):
    """Earth ground + a dug chamber ring; contents(d, cx, cy) fills it."""
    img, d = ground(rng, EARTH, [EARTH_D, EARTH_L], 40)
    rim = ACID_D if acid else EARTH_D
    fillc = (104, 74, 46, 255)
    d.ellipse([5, 5, 27, 27], fill=rim)
    d.ellipse([7, 7, 25, 25], fill=fillc)
    if acid:
        for _ in range(10):
            a = rng.uniform(0, 6.283)
            x = 16 + int(9.5 * math.cos(a))
            y = 16 + int(9.5 * math.sin(a))
            d.point((x, y), fill=ACID)
    contents(d, 16, 16)
    return img


def ant_queen(d, cx, cy, body=RED, dark=DARKRED):
    # legs
    for s in (-1, 1):
        for i, dy in enumerate((-3, 0, 3)):
            d.line([(cx + s * 2, cy + dy), (cx + s * 6, cy + dy * 2)],
                   fill=dark)
    # big abdomen, thorax, head
    d.ellipse([cx - 4, cy + 1, cx + 4, cy + 9], fill=body, outline=dark)
    d.ellipse([cx - 2, cy - 3, cx + 2, cy + 2], fill=body, outline=dark)
    d.ellipse([cx - 2, cy - 7, cx + 2, cy - 3], fill=dark)
    d.line([(cx - 1, cy - 7), (cx - 4, cy - 10)], fill=dark)
    d.line([(cx + 1, cy - 7), (cx + 4, cy - 10)], fill=dark)


def flesh_ground(rng, n=40):
    return ground(rng, FLESH, [FLESH_L, FLESH_D], n)


# ---------------------------------------------------------------- ids

def t_treefarm_1a(rng):
    img, d = ground(rng, IGREEN, [GRASS_D, GRASS_L], 30)
    tree_rows(d, rng)
    return img


def t_treefarm_1a_roof(rng):
    img, d = ground(rng, IGREEN, [GRASS_D, GRASS_L], 18)
    # corner trees peek around the farm building's roof plan
    for (x, y) in ((4, 4), (28, 4), (4, 28), (28, 28)):
        tree(d, x, y, 3)
    roof_plan(d, (8, 9, 24, 23), ROOF_RED, ROOF_RED_D, vents=((16, 12),))
    return img


def t_treefarm_1b(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 26)
    for (x, y) in ((7, 7), (15, 7), (7, 15), (15, 15), (7, 23), (15, 23)):
        tree(d, x + rng.randint(-1, 1), y + rng.randint(-1, 1), 3)
    building(d, (20, 18, 29, 27), CREAM)   # farmhouse footprint
    return img


def ant_tunnel(rng, acid=False):
    img, d = ground(rng, EARTH, [EARTH_D, EARTH_L], 36)
    pts = []
    for x in range(0, 32, 2):
        y = 16 + int(5 * math.sin(x / 5.0 + (1.0 if acid else 0.0)))
        pts.append((x, y))
    d.line(pts, fill=EARTH_D, width=4)
    d.line(pts, fill=(100, 70, 44, 255), width=2)
    # entrance hole
    d.ellipse([13, 12, 19, 17], fill=BLACK)
    d.arc([13, 12, 19, 17], 180, 360, fill=EARTH_L, width=1)
    if acid:
        for _ in range(8):
            d.point((rng.randint(0, 31), rng.randint(0, 31)), fill=ACID)
    return img


def ant_food(rng, acid=False):
    def contents(d, cx, cy):
        for _ in range(14):
            x = cx + rng.randint(-6, 6)
            y = cy + rng.randint(-6, 6)
            if (x - cx) ** 2 + (y - cy) ** 2 <= 42:
                d.ellipse([x - 2, y - 2, x + 2, y + 2],
                          fill=rng.choice([GREEN_FOOD, GREEN_FOOD_D,
                                           YELLOW]))
        if acid:
            for _ in range(6):
                d.point((cx + rng.randint(-6, 6), cy + rng.randint(-6, 6)),
                        fill=ACID)
    return ant_chamber(rng, contents, acid)


def ant_fungus(rng, acid=False):
    def contents(d, cx, cy):
        # shelf fungus: stacked pale arcs + two mushroom caps
        for i, w in enumerate((7, 5, 3)):
            y = cy + 4 - i * 3
            d.arc([cx - w, y - 3, cx + w, y + 3], 180, 360, fill=LGRAY,
                  width=2)
        for (x, y) in ((cx - 5, cy + 2), (cx + 4, cy - 1)):
            d.rectangle([x, y, x, y + 3], fill=WHITE)
            d.pieslice([x - 3, y - 3, x + 3, y + 2], 180, 360, fill=LGRAY)
        if acid:
            d.point((cx - 3, cy - 4), fill=ACID)
            d.point((cx + 2, cy + 5), fill=ACID)
    return ant_chamber(rng, contents, acid)


def ant_larvae(rng, acid=False):
    def contents(d, cx, cy):
        for _ in range(9):
            x = cx + rng.randint(-6, 5)
            y = cy + rng.randint(-6, 5)
            d.ellipse([x, y, x + 4, y + 2], fill=WHITE, outline=LGRAY)
        if acid:
            for _ in range(4):
                d.point((cx + rng.randint(-6, 6), cy + rng.randint(-6, 6)),
                        fill=ACID)
    return ant_chamber(rng, contents, acid)


def ant_queen_ch(rng, acid=False):
    def contents(d, cx, cy):
        ant_queen(d, cx, cy)
    return ant_chamber(rng, contents, acid)


def alien_ground(rng, n=30):
    # alien meadow: data color red -- ruddy alien sward
    return ground(rng, (172, 66, 54, 255),
                  [(146, 50, 44, 255), (198, 88, 66, 255)], n)


def alien_tufts(d, rng, spots):
    for (x, y) in spots:
        for dx in (-2, 0, 2):
            d.line([(x + dx, y + 2), (x + dx * 2, y - 3)],
                   fill=(226, 120, 90, 255))
        d.point((x, y - 3), fill=(250, 190, 120, 255))


def t_alien_core(rng):
    img, d = alien_ground(rng)
    alien_tufts(d, rng, [(6, 7), (26, 6), (7, 26), (25, 25), (16, 5)])
    # central alien growth: bulb heart with tendrils
    for a in range(0, 360, 45):
        x = 16 + int(8 * math.cos(math.radians(a)))
        y = 16 + int(8 * math.sin(math.radians(a)))
        d.line([(16, 16), (x, y)], fill=FLESH_XD, width=2)
    d.ellipse([10, 10, 22, 22], fill=(196, 84, 70, 255), outline=FLESH_XD)
    d.ellipse([13, 12, 19, 18], fill=(232, 130, 100, 255))
    d.point((16, 15), fill=(250, 200, 140, 255))
    return img


def t_alien_end(rng):
    img, d = alien_ground(rng)
    # meadow tapers off toward the bottom: dense tufts top, bare bottom
    alien_tufts(d, rng, [(7, 7), (16, 5), (25, 7), (10, 13), (22, 12)])
    for _ in range(6):
        d.point((rng.randint(2, 29), rng.randint(20, 30)),
                fill=(146, 50, 44, 255))
    return img


def t_alien_join(rng):
    img, d = alien_ground(rng)
    # a dense joining band of tufts crossing the middle
    alien_tufts(d, rng, [(5, 16), (11, 15), (17, 16), (23, 15), (28, 16),
                         (8, 6), (24, 25)])
    return img


def paw(d, cx, cy, color):
    d.ellipse([cx - 3, cy + 1, cx + 3, cy + 6], fill=color)
    for dx in (-4, -1, 2, 4):
        d.ellipse([cx + dx - 1, cy - 3 + abs(dx) // 2, cx + dx + 1,
                   cy - 1 + abs(dx) // 2], fill=color)


def t_animalpound(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    building(d, (6, 8, 26, 24), (172, 138, 96, 255), WOOD_D)
    paw(d, 16, 13, OUT)
    # kennel run lines
    d.line([(8, 21), (24, 21)], fill=WOOD_D)
    return img


def t_animalpound_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (6, 8, 26, 24), (150, 118, 84, 255), WOOD_D,
              vents=((10, 12), (22, 12)))
    return img


def t_animalshelter(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    building(d, (6, 8, 26, 24), IPINK, (196, 110, 148, 255))
    # heart over paw
    paw(d, 16, 15, (196, 60, 100, 255))
    d.polygon([(13, 9), (19, 9), (16, 13)], fill=(196, 60, 100, 255))
    d.ellipse([12, 7, 16, 11], fill=(196, 60, 100, 255))
    d.ellipse([16, 7, 20, 11], fill=(196, 60, 100, 255))
    return img


def t_animalshelter_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (6, 8, 26, 24), (226, 150, 180, 255),
              (196, 110, 148, 255), vents=((10, 12), (22, 20)))
    return img


def t_apartment_courtyard(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 22)
    # U of apartment blocks around a green courtyard
    d.rectangle([4, 4, 28, 12], fill=LGRAY, outline=OUT)
    d.rectangle([4, 4, 11, 28], fill=LGRAY, outline=OUT)
    d.rectangle([21, 4, 28, 28], fill=LGRAY, outline=OUT)
    d.rectangle([13, 14, 19, 26], fill=GRASS)
    tree(d, 16, 19, 3)
    d.rectangle([14, 24, 18, 25], fill=URBAN_L)
    return img


def t_art_gallery(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    building(d, (6, 7, 26, 25), CREAM, URBAN_D)
    # picture frame icon
    d.rectangle([11, 11, 21, 19], fill=IYELLOW, outline=OUT)
    d.polygon([(12, 18), (16, 13), (19, 16), (20, 18)], fill=GRASS_D)
    d.point((18, 13), fill=RED)
    return img


def t_art_gallery_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (6, 7, 26, 25), ROOF_GRAY, URBAN_D, ridge=False)
    # skylights
    d.rectangle([10, 11, 15, 15], fill=ICYAN, outline=OUT)
    d.rectangle([17, 16, 22, 20], fill=ICYAN, outline=OUT)
    return img


def t_bakery(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    building(d, (6, 8, 26, 24), CREAM, (196, 178, 128, 255))
    # bread loaf
    d.ellipse([10, 12, 22, 19], fill=(216, 164, 92, 255), outline=WOOD_D)
    for x in (13, 16, 19):
        d.line([(x, 13), (x - 1, 16)], fill=CREAM, width=1)
    return img


def t_bakery_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (6, 8, 26, 24), (222, 208, 160, 255),
              (196, 178, 128, 255), vents=((22, 11),))
    return img


def t_bakery_upper_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (6, 8, 26, 24), (222, 208, 160, 255),
              (196, 178, 128, 255), ridge=False)
    # raised upper roof section
    d.rectangle([11, 11, 21, 20], fill=(232, 220, 178, 255), outline=OUT)
    d.rectangle([13, 13, 19, 18], outline=(196, 178, 128, 255))
    return img


def t_barn_aban1_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 22)
    for (x, y) in ((4, 5), (28, 6), (5, 27), (27, 27)):
        tree(d, x, y, 3)
    roof_plan(d, (8, 9, 24, 23), ROOF_RED, ROOF_RED_D)
    # collapsed patch: dark hole with splinters
    d.rectangle([17, 15, 22, 20], fill=BLACK)
    d.line([(17, 15), (21, 19)], fill=WOOD)
    d.line([(18, 20), (22, 16)], fill=WOOD)
    return img


def t_bowling_alley_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (5, 8, 27, 24), (172, 84, 66, 255), ROOF_RED_D,
              ridge=False, vents=((8, 11), (24, 21)))
    # bowling pin + ball painted on the roof
    d.ellipse([14, 10, 18, 13], fill=WHITE)
    d.rectangle([15, 13, 17, 15], fill=WHITE)
    d.ellipse([14, 15, 18, 20], fill=WHITE, outline=OUT)
    d.line([(14, 16), (18, 16)], fill=RED)
    d.ellipse([20, 16, 23, 19], fill=OUT)
    return img


def bunker_block(d, rect):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=CONCRETE, outline=OUT)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=CONCRETE_D)
    cy = (y0 + y1) // 2
    d.rectangle([x0 + 3, cy - 1, x1 - 3, cy], fill=BLACK)  # firing slit


def t_bunker(rng):
    img, d = ground(rng, (110, 118, 92, 255),
                    [(92, 100, 76, 255), (128, 136, 108, 255)], 34)
    bunker_block(d, (6, 9, 26, 23))
    d.rectangle([14, 23, 18, 26], fill=CONCRETE_D, outline=OUT)  # entry
    return img


def t_bunker_basement(rng):
    img, d = ground(rng, (58, 56, 58, 255),
                    [(44, 42, 44, 255), (74, 72, 74, 255)], 30)
    bunker_block(d, (6, 8, 26, 22))
    # stairs up glyph
    for i in range(3):
        d.rectangle([12 + i * 3, 27 - i * 2, 14 + i * 3, 28 - i * 2],
                    fill=LGRAY)
    d.polygon([(22, 28), (25, 24), (28, 28)], fill=RED)
    return img


def t_bunker_roof(rng):
    img, d = ground(rng, (110, 118, 92, 255),
                    [(92, 100, 76, 255), (128, 136, 108, 255)], 26)
    d.rectangle([6, 9, 26, 23], fill=CONCRETE, outline=OUT)
    d.rectangle([8, 11, 24, 21], outline=CONCRETE_D)
    d.line([(6, 9), (26, 23)], fill=CONCRETE_D)
    d.rectangle([14, 13, 18, 17], fill=CONCRETE_D, outline=OUT)  # hatch
    d.rectangle([7, 10, 10, 12], fill=RED)                       # marker
    return img


def t_cave_entrance(rng):
    img, d = ground(rng, (146, 138, 120, 255),
                    [(124, 116, 100, 255), (168, 160, 140, 255)], 26)
    # river along the left edge
    d.rectangle([0, 0, 9, 31], fill=LBLUE)
    for y in range(2, 31, 5):
        d.arc([2, y, 8, y + 3], 0, 180, fill=LCYAN)
    # rocky bank + dark cave mouth
    d.ellipse([12, 6, 30, 26], fill=(150, 150, 152, 255), outline=OUT)
    d.ellipse([16, 11, 27, 24], fill=BLACK)
    d.arc([16, 11, 27, 24], 180, 330, fill=LGRAY, width=2)
    return img


def cabin_icon(d, broken=False):
    d.rectangle([10, 12, 22, 24], fill=WOOD, outline=OUT)
    for y in (16, 20):
        d.line([(10, y), (22, y)], fill=WOOD_D)
    d.polygon([(8, 12), (16, 6), (24, 12)], fill=ROOF_RED_D)
    if broken:
        d.polygon([(16, 6), (24, 12), (19, 12)], fill=BLACK)
        d.line([(18, 8), (21, 12)], fill=WOOD)
    d.rectangle([14, 19, 18, 24], fill=OUT)


def t_cabin_aban1(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 24)
    for (x, y) in ((5, 6), (27, 5), (6, 26), (26, 27), (28, 16)):
        tree(d, x, y, 3)
    cabin_icon(d, broken=True)
    return img


def t_cabin_aban1_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 20)
    for (x, y) in ((4, 4), (28, 4), (4, 28), (28, 28)):
        tree(d, x, y, 3)
    roof_plan(d, (9, 9, 23, 23), ROOF_RED, ROOF_RED_D)
    d.rectangle([16, 14, 21, 19], fill=BLACK)   # collapse hole
    d.line([(16, 19), (20, 14)], fill=WOOD)
    return img


def t_prepper_shelter(rng):
    img, d = ground(rng, (110, 118, 92, 255),
                    [(92, 100, 76, 255), (128, 136, 108, 255)], 30)
    # buried hatch: steel square with cross handle (+ sym)
    d.rectangle([9, 9, 23, 23], fill=CONCRETE_D, outline=OUT)
    d.rectangle([11, 11, 21, 21], outline=OUT)
    d.line([(16, 12), (16, 20)], fill=OUT, width=2)
    d.line([(12, 16), (20, 16)], fill=OUT, width=2)
    d.rectangle([9, 9, 12, 12], fill=RED)
    return img


def t_campground_roof(rng):
    img, d = ground(rng, IGREEN, [GRASS_D, GRASS_L], 24)
    # pavilion roof plan in the middle of the campground
    roof_plan(d, (9, 10, 23, 20), WOOD, WOOD_D, vents=())
    # tent + fire at the edges
    d.polygon([(4, 28), (8, 22), (12, 28)], fill=YELLOW, outline=OUT)
    d.ellipse([24, 24, 28, 28], outline=OUT)
    d.point((26, 26), fill=RED)
    d.point((25, 25), fill=YELLOW)
    return img


def t_campsite_cabin_incomplete_roof(rng):
    img, d = ground(rng, LGREEN, [GRASS_D, GRASS_L], 24)
    # cabin walls with bare rafters: the roof is unfinished
    d.rectangle([9, 10, 23, 24], outline=OUT)
    d.rectangle([9, 22, 23, 24], fill=WOOD)
    for x in range(10, 23, 3):
        d.line([(x, 10), (x, 22)], fill=WOOD_D, width=1)
    d.line([(9, 10), (23, 10)], fill=WOOD_D, width=2)
    tree(d, 5, 6, 3)
    tree(d, 27, 27, 3)
    return img


def car_top(d, cx, cy, body):
    d.rectangle([cx - 5, cy - 3, cx + 5, cy + 3], fill=body, outline=OUT)
    d.rectangle([cx - 2, cy - 2, cx + 2, cy + 2], fill=ICYAN, outline=OUT)
    for dx in (-4, 4):
        for dy in (-3, 3):
            d.point((cx + dx, cy + dy), fill=BLACK)


def t_car_corner(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 22)
    # dirt corner track sweeping through the forest
    d.arc([-8, 8, 30, 46], 180, 270, fill=EARTH, width=7)
    d.arc([-8, 8, 30, 46], 180, 270, fill=EARTH_L, width=2)
    for (x, y) in ((6, 5), (26, 6), (27, 26)):
        tree(d, x, y, 3)
    car_top(d, 12, 24, RED)
    return img


def t_car_corner_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 20)
    d.arc([-8, 8, 30, 46], 180, 270, fill=EARTH, width=7)
    # carport roof plan sheltering the corner
    roof_plan(d, (6, 17, 20, 29), ROOF_GRAY, URBAN_D, ridge=False)
    for (x, y) in ((26, 6), (27, 26), (6, 5)):
        tree(d, x, y, 3)
    return img


def t_coop_chicken(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 26)
    # coop house
    d.rectangle([7, 8, 17, 17], fill=WOOD, outline=OUT)
    d.polygon([(6, 8), (12, 4), (18, 8)], fill=ROOF_RED)
    d.rectangle([10, 12, 13, 17], fill=OUT)
    # fenced run with chickens
    d.rectangle([19, 18, 29, 28], outline=WOOD_D)
    for (x, y) in ((22, 21), (26, 24), (21, 26)):
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=WHITE)
        d.point((x, y - 2), fill=RED)
    return img


def tentacle(d, path, body=LBLUE, dark=BLUE):
    for (x, y) in path:
        d.ellipse([x - 4, y - 4, x + 4, y + 4], fill=dark)
    for (x, y) in path:
        d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=body)
    for (x, y) in path[::2]:
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(210, 230, 250, 255))


def t_corpse_bowels_tentacle_edge(rng):
    img, d = flesh_ground(rng)
    # bowel coils: nested tube rings
    for r in (11, 8, 5):
        d.ellipse([16 - r, 13 - r, 16 + r, 13 + r], outline=FLESH_XD,
                  width=3)
        d.arc([16 - r, 13 - r, 16 + r, 13 + r], 200, 300, fill=FLESH_L,
              width=1)
    # a tentacle slides off the bottom edge
    tentacle(d, [(10, 24), (12, 28), (13, 31)])
    return img


def t_corpse_surface(rng):
    img, d = flesh_ground(rng, 30)
    # hide surface: mottled patches, cracks, sparse bristles
    for _ in range(5):
        x, y = rng.randint(3, 24), rng.randint(3, 24)
        d.ellipse([x, y, x + 5, y + 3], fill=FLESH_L)
    for _ in range(4):
        x, y = rng.randint(2, 26), rng.randint(2, 26)
        d.line([(x, y), (x + 3, y + 2)], fill=FLESH_XD)
        d.line([(x, y + 3), (x + 1, y)], fill=OUT)
    return img


def t_corpse_tentacle(rng):
    img, d = flesh_ground(rng)
    path = [(15 + int(2 * math.sin(y / 6.0)), y) for y in range(0, 32, 4)]
    tentacle(d, path)
    return img


def entry_hole(d, cx, cy):
    d.ellipse([cx - 6, cy - 5, cx + 6, cy + 5], fill=FLESH_XD)
    d.ellipse([cx - 4, cy - 3, cx + 4, cy + 3], fill=BLACK)
    for a in range(0, 360, 40):
        x = cx + int(6 * math.cos(math.radians(a)))
        y = cy + int(5 * math.sin(math.radians(a)))
        d.point((x, y), fill=FLESH_L)


def t_corpse_tentacle_entry(rng):
    img, d = flesh_ground(rng)
    tentacle(d, [(8, 2), (9, 6), (11, 10)])
    entry_hole(d, 19, 19)
    return img


def t_corpse_tentacle_surface_entry(rng):
    img, d = flesh_ground(rng, 30)
    for _ in range(3):
        x, y = rng.randint(3, 24), rng.randint(3, 24)
        d.ellipse([x, y, x + 5, y + 3], fill=FLESH_L)
    entry_hole(d, 16, 15)
    tentacle(d, [(24, 26), (26, 29), (27, 31)])
    return img


def t_craft_shop_upper_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (5, 7, 27, 25), (104, 168, 176, 255), (64, 128, 138, 255),
              ridge=False)
    # raised upper section + skylight
    d.rectangle([10, 11, 20, 19], fill=(128, 192, 200, 255), outline=OUT)
    d.rectangle([12, 13, 18, 17], fill=ICYAN, outline=OUT)
    d.rectangle([23, 9, 25, 11], fill=URBAN_D, outline=OUT)
    return img


def t_car_showroom(rng, second=False):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    building(d, (5, 8, 27, 24), (196, 208, 214, 255), URBAN_D, door=False)
    # glass front full of cyan panes
    d.rectangle([7, 10, 25, 17], fill=ICYAN, outline=OUT)
    for x in range(11, 25, 4):
        d.line([(x, 10), (x, 17)], fill=(64, 128, 138, 255))
    car_top(d, 16, 20, RED)
    if second:
        # second-floor band with windows above the glass front
        d.rectangle([7, 5, 25, 8], fill=(196, 208, 214, 255), outline=OUT)
        for x in range(9, 25, 4):
            d.rectangle([x, 6, x + 2, 7], fill=CYAN)
    return img


DRAW = {
    "TreeFarm_1a": t_treefarm_1a,
    "TreeFarm_1a_roof": t_treefarm_1a_roof,
    "TreeFarm_1b": t_treefarm_1b,
    "acid_ants": lambda r: ant_tunnel(r, acid=True),
    "acid_ants_food": lambda r: ant_food(r, acid=True),
    "acid_ants_fungus_colony": lambda r: ant_fungus(r, acid=True),
    "acid_ants_larvae": lambda r: ant_larvae(r, acid=True),
    "acid_ants_queen": lambda r: ant_queen_ch(r, acid=True),
    "alien_meadow_core": t_alien_core,
    "alien_meadow_end": t_alien_end,
    "alien_meadow_join": t_alien_join,
    "animalpound": t_animalpound,
    "animalpound_roof": t_animalpound_roof,
    "animalshelter": t_animalshelter,
    "animalshelter_roof": t_animalshelter_roof,
    "ants": lambda r: ant_tunnel(r),
    "ants_food": lambda r: ant_food(r),
    "ants_fungus_colony": lambda r: ant_fungus(r),
    "ants_larvae": lambda r: ant_larvae(r),
    "ants_queen": lambda r: ant_queen_ch(r),
    "apartment_complex_courtyard": t_apartment_courtyard,
    "art_gallery": t_art_gallery,
    "art_gallery_roof": t_art_gallery_roof,
    "bakery": t_bakery,
    "bakery_roof": t_bakery_roof,
    "bakery_upper_roof": t_bakery_upper_roof,
    "barn_aban1_roof": t_barn_aban1_roof,
    "bowling_alley_roof": t_bowling_alley_roof,
    "bunker": t_bunker,
    "bunker_basement_1": t_bunker_basement,
    "bunker_roof": t_bunker_roof,
    "c_cave_entrance": t_cave_entrance,
    "cabin_aban1": t_cabin_aban1,
    "cabin_aban1_roof": t_cabin_aban1_roof,
    "cabin_prepper_shelter": t_prepper_shelter,
    "campground_roof": t_campground_roof,
    "campsite_cabin_incomplete_roof": t_campsite_cabin_incomplete_roof,
    "car_corner_aban1": t_car_corner,
    "car_corner_aban1_roof": t_car_corner_roof,
    "coop_chicken": t_coop_chicken,
    "corpse_bowels_tentacle_edge": t_corpse_bowels_tentacle_edge,
    "corpse_surface": t_corpse_surface,
    "corpse_tentacle": t_corpse_tentacle,
    "corpse_tentacle_entry": t_corpse_tentacle_entry,
    "corpse_tentacle_surface_entry": t_corpse_tentacle_surface_entry,
    "craft_shop_upper_roof": t_craft_shop_upper_roof,
    "cs_car_showroom": lambda r: t_car_showroom(r),
    "cs_car_showroom_2ndfloor": lambda r: t_car_showroom(r, second=True),
}


def main():
    ids = sorted(DRAW)
    assert len(ids) == 48, len(ids)
    for tid in ids:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = DRAW[tid](rng)
        assert img.size == (32, 32), tid
        alphas = set(img.getchannel("A").getdata())
        assert alphas == {255}, (tid, alphas)   # map cells: fully opaque
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", tid)

    # Contact sheet: 8 cols x 6 rows at 2x NEAREST.
    cell, gap, cols = 64, 3, 8
    rows = (len(ids) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, tid in enumerate(ids):
        img = Image.open(os.path.join(BASE, tid, tid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_om1.png")
    sheet.save(out)
    print("wrote preview_om1.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
