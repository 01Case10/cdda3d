#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch OM5: 48 overmap terrain tiles.

Overmap tiles are map cells: every sprite is a FULL-TILE OPAQUE 32x32 --
a ground/base fill in the location's data color with a centered iconic
pictogram. *_roof ids are the roof plan of their base building.
Families: the microlab underground lab system (light_blue "science lab"
sym "L"; ratkin variants in green; surface lab in blue; subway/pump
connectors below), the mi-go camp (light_gray organic alien structures,
portal in dark_gray, sym "="), the military minefield perimeter
(light_gray, corner box-glyph syms), ocean cells graded by depth, and
omt_obsolete ("nether-chewed field", sym "%") in the project's debug
treatment: black tile, red border, red glyph.
Drawn from each entry's name/symbol/color in fork-src/cdda/data/json
(overmap/overmap_terrain/*.json). Flat UltiCa overmap style.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_om5.png.
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
BLACK = (16, 14, 16, 255)
WHITE = (236, 236, 236, 255)
CREAM = (240, 232, 196, 255)
LGRAY = (178, 178, 178, 255)
GRAY = (130, 130, 134, 255)
GRAY_D = (102, 102, 106, 255)
CONCRETE = (168, 168, 164, 255)
CONCRETE_D = (128, 128, 124, 255)
URBAN = (130, 130, 134, 255)
URBAN_D = (102, 102, 106, 255)
URBAN_L = (158, 158, 162, 255)
GRASS = (86, 140, 58, 255)
GRASS_D = (62, 108, 42, 255)
GRASS_L = (112, 170, 78, 255)
IGREEN = (98, 164, 66, 255)
LGREEN = (140, 190, 100, 255)
EARTH = (122, 86, 54, 255)
EARTH_D = (88, 60, 38, 255)
EARTH_L = (152, 112, 76, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (106, 72, 40, 255)
RED = (178, 44, 36, 255)
DARKRED = (128, 30, 26, 255)
YELLOW = (232, 202, 66, 255)
IYELLOW = (246, 218, 96, 255)
ICYAN = (128, 226, 236, 255)
CYAN = (64, 192, 204, 255)
DEBUG_RED = (232, 40, 32, 255)

# microlab palette: dark lab floor + family accents from the data colors
LAB = (54, 60, 74, 255)
LAB_D = (40, 45, 58, 255)
LAB_L = (76, 84, 102, 255)
LAB_BLUE = (150, 190, 235, 255)    # light_blue accent
LAB_BLUE_D = (96, 140, 200, 255)
DEEP_BLUE = (70, 110, 220, 255)    # blue (surface lab)
RAT = (96, 128, 74, 255)           # ratkin floor tint (warm green-brown)
RAT_D = (72, 98, 56, 255)
RAT_L = (122, 156, 96, 255)
GREEN_A = (110, 200, 100, 255)     # green accent
LRED = (235, 130, 115, 255)        # light_red (distorted hallway)
VIOLET = (168, 110, 220, 255)      # nether glow

# mi-go palette: pale gray alien ground, pink/violet organic forms
MIGO = (168, 160, 170, 255)
MIGO_D = (138, 128, 142, 255)
MIGO_L = (196, 188, 198, 255)
MIGO_FLESH = (222, 148, 168, 255)
MIGO_FLESH_D = (182, 104, 128, 255)
MIGO_VIOLET = (146, 110, 178, 255)
MIGO_GLOW = (120, 240, 196, 255)
PORTAL_G = (104, 100, 110, 255)    # dark_gray structure ground
PORTAL_G_D = (76, 72, 82, 255)

# ocean palette, graded by depth
SEA_SURF = (66, 118, 198, 255)
SEA_SHALLOW = (118, 178, 224, 255)
SEA_CUBE = (38, 84, 168, 255)
SEA_BED = (24, 56, 104, 255)
SAND = (232, 214, 160, 255)
SAND_D = (206, 184, 128, 255)

SCRUB = (150, 128, 92, 255)        # minefield dirt
SCRUB_D = (126, 104, 70, 255)
SCRUB_L = (174, 152, 114, 255)


def ground(rng, base, speck, n=46, box=(0, 0, 31, 31)):
    img = Image.new("RGBA", (32, 32), base)
    d = ImageDraw.Draw(img)
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=rng.choice(speck))
    return img, d


def building(d, rect, wall, wall_d=None, door=True):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=wall, outline=OUT, width=1)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=wall_d or OUT)
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


def tree(d, x, y, r=3, canopy=GRASS_D, hi=GRASS_L):
    d.rectangle([x, y + r - 1, x, y + r + 1], fill=WOOD_D)
    d.ellipse([x - r, y - r, x + r, y + r], fill=canopy)
    d.ellipse([x - r + 1, y - r, x + r - 1, y + 1], fill=hi)


# ------------------------------------------------------------ microlab

def lab_ground(rng, base=LAB, dark=LAB_D, light=LAB_L, n=30):
    img, d = ground(rng, base, [dark, light], n)
    # lab floor panel seams
    for k in (10, 21):
        d.line([(0, k), (31, k)], fill=dark)
        d.line([(k, 0), (k, 31)], fill=dark)
    return img, d


def flask(d, cx, cy, accent, liquid):
    # Erlenmeyer flask pictogram
    d.polygon([(cx - 2, cy - 7), (cx + 2, cy - 7), (cx + 2, cy - 2),
               (cx + 6, cy + 6), (cx - 6, cy + 6), (cx - 2, cy - 2)],
              outline=accent, width=1)
    d.polygon([(cx - 4, cy + 2), (cx + 4, cy + 2), (cx + 6, cy + 6),
               (cx - 6, cy + 6)], fill=liquid)
    d.point((cx - 1, cy + 4), fill=accent)
    d.point((cx + 2, cy + 5), fill=accent)


def stairs_down(d, x0, y0, accent, dark):
    # descending steps, top-left high to bottom-right low, + down arrow
    for i in range(4):
        d.rectangle([x0 + i * 3, y0 + i * 3, x0 + i * 3 + 8, y0 + i * 3 + 2],
                    fill=dark, outline=accent)
    d.polygon([(x0 + 16, y0 + 8), (x0 + 21, y0 + 8), (x0 + 18, y0 + 14)],
              fill=accent)


def vent(d, cx, cy, accent, dark):
    d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], fill=dark, outline=accent,
              width=2)
    for a in (0, 60, 120):
        x = cx + int(5 * math.cos(math.radians(a)))
        y = cy + int(5 * math.sin(math.radians(a)))
        d.line([(cx, cy), (x, y)], fill=accent, width=2)
        x = cx - int(5 * math.cos(math.radians(a)))
        y = cy - int(5 * math.sin(math.radians(a)))
        d.line([(cx, cy), (x, y)], fill=accent, width=2)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=accent)


def burrow(d, cx, cy, dark):
    # ratkin burrow hole with gnawed rim + scattered debris
    d.ellipse([cx - 5, cy - 4, cx + 5, cy + 4], fill=dark)
    d.ellipse([cx - 3, cy - 2, cx + 3, cy + 2], fill=BLACK)
    for dx in (-6, 6, -4, 5):
        d.point((cx + dx, cy + 5), fill=dark)


def t_microlab_generic(rng):
    img, d = lab_ground(rng)
    flask(d, 16, 15, LAB_BLUE, LAB_BLUE_D)
    # bench line under the flask
    d.line([(7, 25), (25, 25)], fill=LAB_BLUE_D, width=2)
    d.line([(9, 27), (9, 29)], fill=LAB_BLUE_D)
    d.line([(23, 27), (23, 29)], fill=LAB_BLUE_D)
    return img


def t_microlab_edge(rng):
    img, d = lab_ground(rng)
    # the complex's edge: a heavy wall band across the top with hazard
    # chevrons, lab floor below
    d.rectangle([0, 0, 31, 8], fill=CONCRETE_D, outline=OUT)
    for x in range(1, 30, 6):
        d.polygon([(x, 8), (x + 3, 3), (x + 6, 8)], fill=YELLOW)
    flask(d, 16, 19, LAB_BLUE, LAB_BLUE_D)
    return img


def t_microlab_sub_entry(rng):
    img, d = lab_ground(rng)
    stairs_down(d, 5, 6, LAB_BLUE, LAB_D)
    d.rectangle([24, 22, 29, 27], outline=LAB_BLUE)   # entry hatch frame
    d.line([(24, 27), (29, 22)], fill=LAB_BLUE)
    return img


def t_microlab_surface(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    # surface lab building in the data's blue
    building(d, (6, 9, 26, 24), (120, 150, 205, 255), DEEP_BLUE)
    flask(d, 16, 14, WHITE, LAB_BLUE)
    return img


def t_microlab_surface_connector(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    # stairwell head-house connecting the surface to the lab below
    d.rectangle([9, 8, 23, 20], fill=CONCRETE, outline=OUT)
    d.polygon([(7, 8), (16, 3), (25, 8)], fill=GRAY_D)
    stairs_down(d, 10, 11, LAB_BLUE_D, GRAY_D)
    d.rectangle([13, 22, 19, 26], fill=LAB_D, outline=OUT)  # shaft below
    return img


def t_microlab_surface_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    roof_plan(d, (6, 9, 26, 24), (120, 150, 205, 255), DEEP_BLUE,
              vents=((10, 12), (22, 20)))
    d.rectangle([13, 13, 19, 18], fill=ICYAN, outline=OUT)  # skylight
    return img


def t_microlab_vent(rng):
    img, d = lab_ground(rng)
    vent(d, 16, 16, LAB_BLUE, LAB_D)
    return img


def t_microlab_distorted(rng):
    img, d = ground(rng, (66, 52, 56, 255),
                    [(52, 40, 44, 255), (82, 64, 68, 255)], 26)
    # a hallway whose walls wobble: two sine-offset corridor walls
    top, bot = [], []
    for x in range(0, 32, 2):
        off = int(3 * math.sin(x / 4.5))
        top.append((x, 9 + off))
        bot.append((x, 22 + off))
    d.line(top, fill=LRED, width=2)
    d.line(bot, fill=LRED, width=2)
    for x in range(2, 31, 5):
        off = int(3 * math.sin(x / 4.5))
        d.line([(x, 11 + off), (x, 20 + off)], fill=(120, 74, 72, 255))
    # warped glow at the corridor's heart
    d.ellipse([13, 13, 19, 19], fill=LRED)
    d.ellipse([14, 14, 18, 18], fill=(248, 190, 170, 255))
    return img


def t_microlab_nether_glass(rng):
    img, d = ground(rng, (14, 12, 18, 255),
                    [(22, 19, 28, 255), (8, 7, 10, 255)], 22)
    # nether-glass elevator shaft: glass rectangle, violet glow seams,
    # up/down arrows on black
    d.rectangle([10, 4, 21, 27], outline=VIOLET, width=2)
    d.rectangle([12, 6, 19, 25], outline=(84, 60, 110, 255))
    d.line([(16, 4), (16, 27)], fill=(84, 60, 110, 255))
    d.polygon([(16, 9), (13, 14), (19, 14)], outline=VIOLET)
    d.polygon([(16, 22), (13, 17), (19, 17)], outline=VIOLET)
    for (x, y) in ((5, 6), (26, 9), (6, 24), (25, 25)):
        d.point((x, y), fill=VIOLET)
    return img


def rat_ground(rng):
    return lab_ground(rng, base=RAT, dark=RAT_D, light=RAT_L, n=30)


def t_ratkin(rng):
    img, d = rat_ground(rng)
    flask(d, 13, 13, GREEN_A, (70, 150, 66, 255))
    burrow(d, 23, 23, RAT_D)
    return img


def t_ratkin_edge(rng):
    img, d = rat_ground(rng)
    d.rectangle([0, 0, 31, 8], fill=(110, 92, 64, 255), outline=OUT)
    for x in range(1, 30, 6):
        d.polygon([(x, 8), (x + 3, 3), (x + 6, 8)], fill=(196, 170, 110, 255))
    burrow(d, 11, 19, RAT_D)
    flask(d, 22, 18, GREEN_A, (70, 150, 66, 255))
    return img


def t_ratkin_sub_entry(rng):
    img, d = rat_ground(rng)
    stairs_down(d, 5, 6, GREEN_A, RAT_D)
    burrow(d, 25, 24, RAT_D)
    return img


def t_ratkin_surface_connector(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 26)
    # overgrown head-house: the ratkin way down from the surface
    d.rectangle([9, 9, 23, 21], fill=(110, 92, 64, 255), outline=OUT)
    d.polygon([(7, 9), (16, 4), (25, 9)], fill=RAT_D)
    stairs_down(d, 10, 12, GREEN_A, RAT_D)
    burrow(d, 26, 26, EARTH_D)
    tree(d, 4, 5, 2)
    return img


def t_ratkin_vent(rng):
    img, d = rat_ground(rng)
    vent(d, 16, 16, GREEN_A, RAT_D)
    return img


def rails(d, y=19, accent=LGRAY, dark=LAB_D):
    d.line([(0, y), (31, y)], fill=dark, width=3)
    d.line([(0, y - 1), (31, y - 1)], fill=accent)
    d.line([(0, y + 1), (31, y + 1)], fill=accent)
    for x in range(1, 31, 4):
        d.line([(x, y - 3), (x, y + 3)], fill=dark)


def t_sub_connector(rng):
    img, d = lab_ground(rng)
    d.rectangle([0, 0, 31, 10], fill=LAB_D)          # platform
    d.line([(0, 10), (31, 10)], fill=LAB_BLUE, width=1)
    rails(d, 21, LAB_BLUE, LAB_D)
    return img


def t_sub_connector_ratkin(rng):
    img, d = lab_ground(rng)
    d.rectangle([0, 0, 31, 10], fill=LAB_D)
    d.line([(0, 10), (31, 10)], fill=LAB_BLUE, width=1)
    rails(d, 21, LAB_BLUE, LAB_D)
    burrow(d, 8, 5, BLACK)                            # ratkin have been here
    d.point((26, 5), fill=GREEN_A)
    d.point((28, 7), fill=GREEN_A)
    return img


def t_sub_station(rng):
    img, d = lab_ground(rng)
    d.rectangle([0, 0, 31, 12], fill=LAB_D)           # platform
    d.line([(0, 12), (31, 12)], fill=YELLOW, width=1)  # safety line
    # station sign + bench
    d.rectangle([9, 2, 23, 7], fill=LAB_BLUE, outline=OUT)
    d.line([(11, 4), (21, 4)], fill=WHITE, width=1)
    d.rectangle([11, 9, 21, 10], fill=WOOD_D)
    rails(d, 23, LAB_BLUE, LAB_D)
    return img


def t_sub_station_ratkin(rng):
    img, d = rat_ground(rng)
    # pump station: manifold pipes + a big valve wheel
    for y in (9, 13):
        d.line([(2, y), (29, y)], fill=(150, 158, 150, 255), width=2)
    d.line([(8, 4), (8, 27)], fill=(150, 158, 150, 255), width=2)
    d.ellipse([17, 15, 27, 25], outline=RED, width=2)
    for a in (0, 60, 120):
        x = 22 + int(4 * math.cos(math.radians(a)))
        y = 20 + int(4 * math.sin(math.radians(a)))
        d.line([(22, 20), (x, y)], fill=RED, width=1)
        d.line([(22, 20), (44 - x, 40 - y)], fill=RED, width=1)
    d.ellipse([20, 18, 24, 22], fill=GREEN_A)
    d.rectangle([3, 22, 10, 28], fill=RAT_D, outline=OUT)  # pump block
    return img


# ------------------------------------------------------------ mi-go

def migo_ground(rng, n=30):
    return ground(rng, MIGO, [MIGO_D, MIGO_L, MIGO_VIOLET], n)


def migo_growths(d, rng, spots):
    for (x, y) in spots:
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=MIGO_FLESH,
                  outline=MIGO_FLESH_D)
        d.point((x, y - 1), fill=MIGO_GLOW)


def t_migo_spire(rng):
    img, d = migo_ground(rng)
    migo_growths(d, rng, [(5, 26), (27, 25), (6, 7)])
    # tapering organic spire of stacked rings, glowing tip
    tiers = [(9, 26, 7), (7, 21, 6), (5, 16, 5), (4, 12, 4), (3, 8, 3)]
    for (hw, yb, hh) in tiers:
        d.ellipse([16 - hw, yb - hh, 16 + hw, yb + hh], fill=MIGO_FLESH,
                  outline=MIGO_FLESH_D)
        d.arc([16 - hw, yb - hh, 16 + hw, yb + hh], 20, 160,
              fill=MIGO_L, width=1)
    d.ellipse([14, 3, 18, 7], fill=MIGO_VIOLET, outline=OUT)
    d.point((16, 5), fill=MIGO_GLOW)
    d.point((15, 4), fill=MIGO_GLOW)
    return img


def t_migo_platform(rng):
    img, d = migo_ground(rng)
    # flat landing disc ("_" sym): rim, radial markings, glowing center
    d.ellipse([4, 6, 28, 26], fill=MIGO_D, outline=OUT)
    d.ellipse([6, 8, 26, 24], fill=MIGO_L, outline=MIGO_VIOLET)
    for a in range(0, 360, 45):
        x = 16 + int(7 * math.cos(math.radians(a)))
        y = 16 + int(5 * math.sin(math.radians(a)))
        d.point((x, y), fill=MIGO_VIOLET)
        d.point((x + 1, y), fill=MIGO_VIOLET)
    d.ellipse([13, 13, 19, 19], fill=MIGO_FLESH, outline=MIGO_FLESH_D)
    d.point((16, 16), fill=MIGO_GLOW)
    return img


def t_migo_landing_roof(rng):
    img, d = migo_ground(rng)
    # the platform's ribbed organic dome cover
    d.ellipse([4, 6, 28, 26], fill=MIGO_FLESH_D, outline=OUT)
    for r in (9, 6, 3):
        d.ellipse([16 - r, 16 - int(r * 0.8), 16 + r, 16 + int(r * 0.8)],
                  outline=MIGO_FLESH)
    for a in range(0, 360, 60):
        x = 16 + int(10 * math.cos(math.radians(a)))
        y = 16 + int(8 * math.sin(math.radians(a)))
        d.line([(16, 16), (x, y)], fill=MIGO_FLESH_D, width=1)
    d.ellipse([14, 14, 18, 18], fill=MIGO_VIOLET)
    d.point((16, 16), fill=MIGO_GLOW)
    return img


def t_migo_portal(rng):
    img, d = ground(rng, PORTAL_G, [PORTAL_G_D, MIGO_VIOLET], 26)
    # two dark pylons; between them the "=" portal field: two glowing
    # horizontal energy bands
    for x0 in (5, 23):
        d.polygon([(x0, 26), (x0 + 1, 8), (x0 + 3, 5), (x0 + 4, 26)],
                  fill=(52, 48, 60, 255), outline=OUT)
        d.point((x0 + 2, 7), fill=MIGO_GLOW)
    for y in (12, 19):
        d.rectangle([8, y, 24, y + 2], fill=MIGO_GLOW)
        d.rectangle([8, y - 1, 24, y - 1], fill=(60, 140, 110, 255))
    d.line([(8, 16), (24, 16)], fill=VIOLET)
    migo_growths(d, rng, [(16, 28), (3, 4), (29, 4)])
    return img


def t_migo_portal_roof(rng):
    img, d = ground(rng, PORTAL_G, [PORTAL_G_D, MIGO_VIOLET], 22)
    # roof plan of the structure: dark slab, the "=" glyph glowing
    d.rectangle([5, 6, 27, 26], fill=(58, 54, 66, 255), outline=OUT)
    d.rectangle([7, 8, 25, 24], outline=PORTAL_G_D)
    for y in (13, 18):
        d.rectangle([10, y, 22, y + 2], fill=MIGO_GLOW)
    for (x, y) in ((7, 8), (25, 8), (7, 24), (25, 24)):
        d.point((x, y), fill=MIGO_VIOLET)
    return img


# ------------------------------------------------------------ minefield

def minefield_ground(rng):
    return ground(rng, SCRUB, [SCRUB_D, SCRUB_L], 40)


def mine(d, x, y):
    d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=(40, 40, 44, 255),
              outline=OUT)
    d.point((x, y - 2), fill=RED)


def fence_h(d, y, x0=0, x1=31, gap=None):
    for x in range(x0, x1 + 1, 5):
        if gap and gap[0] - 2 <= x <= gap[1] + 2:
            continue
        d.rectangle([x, y - 2, x, y + 2], fill=WOOD_D)
    for yy in (y - 1, y + 1):
        if gap:
            d.line([(x0, yy), (gap[0], yy)], fill=GRAY_D)
            d.line([(gap[1], yy), (x1, yy)], fill=GRAY_D)
        else:
            d.line([(x0, yy), (x1, yy)], fill=GRAY_D)


def fence_v(d, x, y0=0, y1=31):
    for y in range(y0, y1 + 1, 5):
        d.rectangle([x - 2, y, x + 2, y], fill=WOOD_D)
    for xx in (x - 1, x + 1):
        d.line([(xx, y0), (xx, y1)], fill=GRAY_D)


def mine_corner(rng, edges):
    img, d = minefield_ground(rng)
    if "n" in edges:
        fence_h(d, 3)
    if "s" in edges:
        fence_h(d, 28)
    if "w" in edges:
        fence_v(d, 3)
    if "e" in edges:
        fence_v(d, 28)
    for (x, y) in ((11, 12), (20, 15), (14, 21), (23, 24), (9, 25)):
        mine(d, x, y)
    # warning sign
    d.rectangle([15, 5, 16, 9], fill=WOOD_D)
    d.polygon([(13, 1), (18, 1), (16, 5)], fill=YELLOW, outline=OUT)
    return img


def mine_gate(rng, side):
    img, d = minefield_ground(rng)
    gap = (2, 9) if side == "left" else (22, 29)
    fence_h(d, 3, gap=gap)
    fence_h(d, 28)
    # gate posts + swung gate leaf at the opening
    gx = gap[0] if side == "left" else gap[1]
    d.rectangle([gx - 1, 1, gx + 1, 6], fill=OUT)
    leaf_x = gap[1] if side == "left" else gap[0]
    d.line([(gx, 3), (leaf_x, 10)], fill=GRAY_D, width=2)
    d.line([(gx, 5), (leaf_x, 12)], fill=GRAY_D, width=1)
    # gravel track through the gate
    cx = (gap[0] + gap[1]) // 2
    d.rectangle([cx - 3, 0, cx + 3, 31], fill=SCRUB_L)
    for (x, y) in ((12, 14), (21, 18), (14, 24), (24, 11)):
        mine(d, x, y)
    d.polygon([(cx - 2, 13), (cx + 3, 13), (cx, 18)], fill=YELLOW,
              outline=OUT)
    return img


# ------------------------------------------------------------ singles

def t_mine_finale(rng):
    img, d = ground(rng, EARTH, [EARTH_D, EARTH_L], 34)
    # headframe tower over a dark shaft, spoil heap beside
    d.polygon([(9, 26), (13, 6), (17, 26)], outline=WOOD_D, width=2)
    d.line([(10, 19), (16, 19)], fill=WOOD_D, width=1)
    d.line([(11, 13), (15, 13)], fill=WOOD_D, width=1)
    d.ellipse([11, 4, 15, 8], fill=GRAY_D, outline=OUT)     # sheave wheel
    d.rectangle([10, 24, 16, 28], fill=BLACK, outline=OUT)  # shaft
    d.line([(13, 8), (13, 24)], fill=GRAY_D)
    for (x, y) in ((21, 24), (24, 22), (22, 27), (26, 26)):
        d.point((x, y), fill=EARTH_L)
    d.polygon([(19, 28), (24, 20), (29, 28)], fill=EARTH_D)
    d.polygon([(22, 24), (24, 20), (26, 24)], fill=EARTH_L)
    return img


def t_mine_tunnel(rng):
    img, d = ground(rng, (24, 22, 26, 255),
                    [(34, 32, 36, 255), (14, 12, 16, 255)], 30)
    # timber-framed tunnel cross-section, rails running out of the dark
    d.rectangle([6, 5, 25, 27], outline=WOOD_D, width=2)
    d.rectangle([9, 8, 22, 27], outline=WOOD, width=1)
    for x in (12, 19):
        d.line([(x, 27), (x - 2, 31)], fill=LGRAY, width=1)
        d.line([(x, 20), (x, 27)], fill=GRAY_D)
    for y in range(22, 32, 3):
        d.line([(9, y), (22, y)], fill=WOOD_D)
    d.point((16, 12), fill=YELLOW)   # a distant lamp
    d.point((16, 11), fill=(120, 100, 40, 255))
    return img


def t_mortuary(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    # somber stone building, blue-gray trim, cross emblem, chimney
    building(d, (6, 9, 26, 25), (150, 152, 158, 255), GRAY_D)
    d.rectangle([21, 4, 24, 9], fill=GRAY_D, outline=OUT)   # chimney
    d.rectangle([13, 12, 19, 18], fill=(170, 200, 240, 255), outline=OUT)
    d.line([(16, 13), (16, 17)], fill=WHITE, width=2)
    d.line([(14, 15), (18, 15)], fill=WHITE, width=2)
    return img


def t_mortuary_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    # i_blue roof plan
    roof_plan(d, (6, 9, 26, 25), (170, 200, 240, 255), (120, 156, 205, 255),
              vents=((10, 12),))
    d.rectangle([21, 6, 24, 10], fill=GRAY_D, outline=OUT)  # chimney cap
    return img


def t_movietheater(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    building(d, (5, 10, 27, 26), CREAM, (196, 178, 128, 255), door=False)
    # marquee: yellow bulb-framed sign over the entrance
    d.rectangle([8, 4, 24, 11], fill=RED, outline=OUT)
    for x in range(9, 24, 3):
        d.point((x, 5), fill=IYELLOW)
        d.point((x, 10), fill=IYELLOW)
    d.line([(11, 7), (21, 7)], fill=WHITE, width=2)
    d.rectangle([13, 20, 19, 26], fill=OUT)                  # doors
    d.line([(16, 20), (16, 26)], fill=CREAM)
    for x in (8, 22):                                        # poster cases
        d.rectangle([x, 14, x + 3, 19], fill=ICYAN, outline=OUT)
    return img


def t_museum(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    # classical facade: pediment + columns on a plinth
    d.rectangle([5, 22, 27, 25], fill=CONCRETE_D, outline=OUT)
    for x in (8, 12, 16, 20, 24):
        d.rectangle([x - 1, 11, x + 1, 22], fill=WHITE, outline=GRAY_D)
    d.rectangle([5, 9, 27, 11], fill=CREAM, outline=OUT)
    d.polygon([(5, 9), (16, 3), (27, 9)], fill=CREAM, outline=OUT)
    d.point((16, 6), fill=RED)
    return img


def t_museum_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    roof_plan(d, (5, 7, 27, 25), WHITE, LGRAY, ridge=False)
    # central glass dome
    d.ellipse([11, 11, 21, 21], fill=ICYAN, outline=OUT)
    d.ellipse([13, 13, 19, 19], outline=(64, 128, 138, 255))
    d.point((16, 16), fill=WHITE)
    return img


def t_natural_spring(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 30)
    # stone-ringed spring pool ("o") with bubbles and a trickling stream
    d.ellipse([8, 7, 24, 23], fill=(90, 130, 190, 255))
    d.ellipse([10, 9, 22, 21], fill=(80, 150, 225, 255))
    for a in range(0, 360, 36):
        x = 16 + int(8 * math.cos(math.radians(a)))
        y = 15 + int(8 * math.sin(math.radians(a)))
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=LGRAY, outline=GRAY_D)
    for (x, y) in ((14, 13), (18, 16), (15, 18), (19, 11)):
        d.point((x, y), fill=ICYAN)
        d.point((x + 1, y), fill=WHITE)
    pts = [(18, 23), (17, 26), (19, 29), (18, 31)]
    d.line(pts, fill=(80, 150, 225, 255), width=2)
    tree(d, 4, 4, 2)
    tree(d, 28, 5, 2)
    return img


def t_sparkling_chasm(rng):
    img, d = ground(rng, (88, 86, 92, 255),
                    [(70, 68, 74, 255), (106, 104, 110, 255)], 30)
    # jagged impassable crack ("*") with sparkling glints
    pts = [(2, 8), (9, 13), (13, 11), (17, 17), (22, 15), (26, 21),
           (30, 19)]
    d.line(pts, fill=BLACK, width=5)
    d.line(pts, fill=(40, 36, 44, 255), width=2)
    pts2 = [(4, 24), (11, 20), (16, 25), (24, 23), (29, 27)]
    d.line(pts2, fill=BLACK, width=3)
    for (x, y) in ((8, 6), (20, 8), (27, 12), (6, 18), (15, 27), (24, 28)):
        d.line([(x - 1, y), (x + 1, y)], fill=IYELLOW)
        d.line([(x, y - 1), (x, y + 1)], fill=IYELLOW)
        d.point((x, y), fill=WHITE)
    return img


def wave_arcs(d, rng, rows, color):
    for y in rows:
        for x in range(rng.randint(0, 3), 30, 8):
            d.arc([x, y, x + 6, y + 3], 0, 180, fill=color)


def t_ocean_surface(rng):
    img, d = ground(rng, SEA_SURF, [(52, 102, 178, 255),
                                    (84, 136, 214, 255)], 20)
    wave_arcs(d, rng, (4, 12, 20, 27), ICYAN)
    return img


def t_ocean_shore(rng):
    img, d = ground(rng, SEA_SHALLOW, [(100, 160, 208, 255),
                                       (140, 196, 234, 255)], 14)
    # sandy shore band along the top, foam at the waterline
    d.rectangle([0, 0, 31, 7], fill=SAND)
    for _ in range(16):
        d.point((rng.randint(0, 31), rng.randint(0, 6)), fill=SAND_D)
    for x in range(0, 32, 4):
        d.arc([x, 6, x + 4, 10], 0, 180, fill=WHITE)
    wave_arcs(d, rng, (14, 22, 28), ICYAN)
    return img


def t_ocean_water_cube(rng):
    img, d = ground(rng, SEA_CUBE, [(30, 70, 148, 255),
                                    (52, 100, 186, 255)], 16)
    # submerged column: light rays slanting down, rising bubbles
    for x0 in (5, 14, 23):
        d.line([(x0, 0), (x0 - 3, 14)], fill=(70, 120, 200, 255))
    for (x, y) in ((9, 22), (20, 18), (25, 26), (12, 28)):
        d.ellipse([x, y, x + 2, y + 2], outline=ICYAN)
    return img


def t_ocean_bed(rng):
    img, d = ground(rng, SEA_BED, [(18, 44, 88, 255),
                                   (34, 68, 122, 255)], 14)
    # the bed itself: dark sand/rock patches and kelp silhouettes below
    d.rectangle([0, 25, 31, 31], fill=(52, 62, 58, 255))
    for _ in range(14):
        d.point((rng.randint(0, 31), rng.randint(25, 31)),
                fill=(86, 92, 74, 255))
    for x in (6, 15, 25):
        d.line([(x, 25), (x - 1, 16)], fill=(20, 70, 44, 255), width=2)
        d.line([(x, 25), (x + 2, 18)], fill=(20, 70, 44, 255), width=2)
    d.ellipse([10, 27, 14, 29], fill=(70, 72, 66, 255), outline=OUT)
    return img


def t_anduril(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    # warehouse block with loading dock + small office annex
    d.rectangle([4, 8, 24, 24], fill=(150, 168, 190, 255), outline=OUT)
    for x in (7, 13, 19):
        d.rectangle([x, 17, x + 4, 24], fill=GRAY_D, outline=OUT)
        d.line([(x, 20), (x + 4, 20)], fill=OUT)
    d.rectangle([24, 12, 29, 24], fill=CREAM, outline=OUT)
    d.rectangle([25, 15, 28, 18], fill=ICYAN, outline=OUT)
    d.rectangle([25, 20, 28, 24], fill=OUT)
    return img


def t_anduril_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    roof_plan(d, (4, 8, 29, 24), (150, 190, 235, 255), (104, 150, 200, 255),
              ridge=False, vents=((9, 12), (17, 12), (24, 19)))
    d.rectangle([6, 18, 12, 21], outline=(104, 150, 200, 255))
    return img


def t_obsolete(rng):
    # omt_obsolete: deprecated tile -- project debug treatment (black
    # tile, red border) with its own data symbol "%"
    img = Image.new("RGBA", (32, 32), (8, 8, 10, 255))
    d = ImageDraw.Draw(img)
    d.rectangle([1, 1, 30, 30], outline=DEBUG_RED, width=2)
    # big "%": two rings and a slash
    d.ellipse([8, 7, 14, 13], outline=DEBUG_RED, width=2)
    d.ellipse([18, 19, 24, 25], outline=DEBUG_RED, width=2)
    d.line([(23, 6), (9, 26)], fill=DEBUG_RED, width=2)
    for (x, y) in ((4, 27), (27, 4)):
        d.point((x, y), fill=DEBUG_RED)
    return img


def t_orchard_processing(rng):
    img, d = ground(rng, LGREEN, [GRASS_D, GRASS_L], 26)
    # processing shed with crates and a conveyor line
    building(d, (5, 6, 20, 18), (196, 178, 128, 255), WOOD_D)
    d.line([(20, 12), (28, 12)], fill=GRAY_D, width=2)
    for (x, y) in ((23, 16), (27, 20), (8, 22), (13, 24), (22, 25)):
        d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=WOOD, outline=WOOD_D)
        d.point((x, y - 1), fill=RED)
        d.point((x - 1, y), fill=YELLOW)
    tree(d, 28, 5, 2)
    return img


def t_orchard_processing_roof(rng):
    img, d = ground(rng, LGREEN, [GRASS_D, GRASS_L], 18)
    roof_plan(d, (5, 6, 20, 18), (170, 160, 130, 255), WOOD_D,
              vents=((17, 8),))
    tree(d, 28, 5, 2)
    tree(d, 6, 27, 2)
    return img


def t_orchard_stall(rng):
    img, d = ground(rng, LGREEN, [GRASS_D, GRASS_L], 26)
    # market stall: posts, striped awning, fruit crates
    for x in (8, 23):
        d.line([(x, 12), (x, 26)], fill=WOOD_D, width=2)
    d.polygon([(6, 12), (25, 12), (27, 7), (8, 7)], fill=CREAM, outline=OUT)
    for x in range(8, 26, 4):
        d.line([(x, 8), (x - 1, 11)], fill=RED, width=2)
    d.rectangle([8, 20, 24, 26], fill=WOOD, outline=WOOD_D)
    for (x, y) in ((11, 17), (16, 16), (21, 17), (13, 22), (19, 23)):
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=RED, outline=DARKRED)
    d.ellipse([15, 20, 18, 23], fill=YELLOW, outline=WOOD_D)
    return img


DRAW = {
    "mi-go_camp2-1": t_migo_spire,
    "mi-go_camp2-2": t_migo_platform,
    "mi-go_camp2-3": t_migo_landing_roof,
    "mi-go_portal": t_migo_portal,
    "mi-go_portal_roof": t_migo_portal_roof,
    "microlab_distorted_hallway": t_microlab_distorted,
    "microlab_generic": t_microlab_generic,
    "microlab_generic_edge": t_microlab_edge,
    "microlab_generic_sub_entry": t_microlab_sub_entry,
    "microlab_generic_surface": t_microlab_surface,
    "microlab_generic_surface_connector": t_microlab_surface_connector,
    "microlab_generic_surface_roof": t_microlab_surface_roof,
    "microlab_generic_vent_shaft_connector": t_microlab_vent,
    "microlab_portal_elevator_nether_glass": t_microlab_nether_glass,
    "microlab_ratkin": t_ratkin,
    "microlab_ratkin_edge": t_ratkin_edge,
    "microlab_ratkin_sub_entry": t_ratkin_sub_entry,
    "microlab_ratkin_surface_connector": t_ratkin_surface_connector,
    "microlab_ratkin_vent_shaft_connector": t_ratkin_vent,
    "microlab_sub_connector": t_sub_connector,
    "microlab_sub_connector_ratkin": t_sub_connector_ratkin,
    "microlab_sub_station": t_sub_station,
    "microlab_sub_station_ratkin": t_sub_station_ratkin,
    "mil_base_minefield_gate_left": lambda r: mine_gate(r, "left"),
    "mil_base_minefield_gate_right": lambda r: mine_gate(r, "right"),
    "mil_base_minefield_ne": lambda r: mine_corner(r, "ne"),
    "mil_base_minefield_nw": lambda r: mine_corner(r, "nw"),
    "mil_base_minefield_se": lambda r: mine_corner(r, "se"),
    "mil_base_minefield_sw": lambda r: mine_corner(r, "sw"),
    "mine_finale": t_mine_finale,
    "mine_tunnel": t_mine_tunnel,
    "mortuary": t_mortuary,
    "mortuary_roof": t_mortuary_roof,
    "movietheater_1_0": t_movietheater,
    "museum": t_museum,
    "museum_roof": t_museum_roof,
    "natural_spring": t_natural_spring,
    "netherum_labyrinth_sparkling_chasm": t_sparkling_chasm,
    "ocean_bed": t_ocean_bed,
    "ocean_shore": t_ocean_shore,
    "ocean_surface": t_ocean_surface,
    "ocean_water_cube": t_ocean_water_cube,
    "office_warehouse_anduril": t_anduril,
    "office_warehouse_anduril_roof": t_anduril_roof,
    "omt_obsolete": t_obsolete,
    "orchard_processing": t_orchard_processing,
    "orchard_processing_roof": t_orchard_processing_roof,
    "orchard_stall": t_orchard_stall,
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
                       "preview_om5.png")
    sheet.save(out)
    print("wrote preview_om5.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
