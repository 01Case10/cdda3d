#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch OM7 (final overmap batch):
47 overmap terrain tiles.

Overmap tiles are map cells: every sprite is a FULL-TILE OPAQUE 32x32 --
a ground/base fill in the location's data color with a centered iconic
pictogram. *_roof ids are the roof plan of their base building: same
footprint and palette, roof texture instead of the ground icon.
Families: the s_* shop tail (sign-band pictograms, shared shop-row
language), slime pit graded by z-level, string-dimension field/plane
cells (pale geometry on void black, after the terrain string-dimension
set), triffid field/grove/heart graded toward the red heart, football
stadium corner segments, trailers by use, and the unexplored cell as
the classic dark unknown.
Drawn from each entry's name/symbol/color in fork-src/cdda/data/json
(overmap/overmap_terrain/*.json), copy-from chains resolved.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_om7.png.
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

OUT = (30, 28, 30, 255)
GRASS = (86, 140, 58, 255)
GRASS_D = (62, 108, 42, 255)
GRASS_L = (112, 170, 78, 255)
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
WOOD_L = (176, 134, 88, 255)
BROWN = (110, 76, 46, 255)
ROOF_GRAY = (116, 116, 120, 255)
ROOF_D = (92, 92, 96, 255)
WHITE = (236, 236, 236, 255)
CREAM = (240, 232, 196, 255)
LGRAY = (178, 178, 178, 255)
DGRAY = (92, 92, 98, 255)
BLACK = (14, 12, 16, 255)
YELLOW = (232, 202, 66, 255)
GOLD = (250, 214, 92, 255)
RED = (196, 48, 40, 255)
DARKRED = (140, 32, 28, 255)
LRED = (230, 110, 90, 255)
PINK = (240, 156, 186, 255)
IPINK = (246, 178, 204, 255)
CYAN = (64, 192, 204, 255)
LCYAN = (146, 222, 232, 255)
LBLUE = (112, 162, 232, 255)
IGREEN = (64, 200, 80, 255)
ASPHALT = (74, 74, 80, 255)
ASPHALT_D = (56, 56, 62, 255)
GLASS = (168, 224, 228, 255)
SLIME = (140, 220, 120, 255)
SLIME_D = (86, 168, 74, 255)
VOID = (8, 6, 12, 255)
STRING_W = (232, 236, 244, 255)
STRING_C = (110, 220, 235, 255)
STRING_M = (225, 110, 205, 255)
STRING_B = (120, 150, 245, 255)


def ground(rng, base, speck, n=46, box=(0, 0, 31, 31)):
    img = Image.new("RGBA", (32, 32), base)
    d = ImageDraw.Draw(img)
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=speck)
    return img, d


def building(d, rect, wall, wall_d=None, door=True):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=wall, outline=OUT, width=1)
    d.line([(x0, y1 - 3), (x1, y1 - 3)], fill=wall_d or wall)
    if door:
        d.rectangle([15, y1 - 4, 17, y1 - 1], fill=OUT)
    # windows
    for wx in (x0 + 3, x1 - 5):
        d.rectangle([wx, y0 + 4, wx + 2, y0 + 6], fill=GLASS, outline=OUT)


def roof_plan(d, rect, base, dark, ridge=True, vents=()):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=base, outline=OUT, width=1)
    for y in range(y0 + 3, y1 - 1, 3):
        d.line([(x0 + 1, y), (x1 - 1, y)], fill=dark)
    if ridge:
        d.line([(x0 + 2, (y0 + y1) // 2), (x1 - 2, (y0 + y1) // 2)],
               fill=dark, width=2)
    for (vx, vy) in vents:
        d.rectangle([vx, vy, vx + 2, vy + 2], fill=dark, outline=OUT)


def sign_band(d, rect, band_color, glyph):
    """Shop-row storefront: building with a colored sign band + glyph fn."""
    x0, y0, x1, y1 = rect
    building(d, rect, CREAM, CONCRETE_D)
    d.rectangle([x0 + 1, y0 + 1, x1 - 1, y0 + 6], fill=band_color,
                outline=OUT)
    glyph(d)


def note(d, cx, cy, color=WHITE):
    d.ellipse([cx - 4, cy + 2, cx - 1, cy + 5], fill=color)
    d.line([(cx - 1, cy + 3), (cx - 1, cy - 4)], fill=color, width=1)
    d.line([(cx - 1, cy - 4), (cx + 3, cy - 3)], fill=color, width=1)
    d.ellipse([cx + 1, cy + 1, cx + 4, cy + 4], fill=color)
    d.line([(cx + 4, cy + 2), (cx + 4, cy - 3)], fill=color, width=1)


def tag(d, cx, cy, color=WHITE):
    d.polygon([(cx - 4, cy - 3), (cx + 1, cy - 3), (cx + 4, cy),
               (cx - 1, cy + 4), (cx - 4, cy + 1)], fill=color)
    d.ellipse([cx - 3, cy - 2, cx - 1, cy], fill=OUT)


def star(d, cx, cy, r, color):
    pts = []
    for i in range(10):
        ang = -math.pi / 2 + i * math.pi / 5
        rr = r if i % 2 == 0 else r * 0.45
        pts.append((cx + rr * math.cos(ang), cy + rr * math.sin(ang)))
    d.polygon(pts, fill=color)


# ---------------- shops ----------------

def t_music(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    sign_band(d, (5, 8, 26, 27), BROWN, lambda dd: note(dd, 16, 13))
    return img


def t_music_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 7, 26, 26), ROOF_GRAY, ROOF_D, vents=((9, 11), (21, 20)))
    note(d, 16, 16, LGRAY)
    return img


def t_sports_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 7, 26, 26), ROOF_GRAY, ROOF_D, vents=((22, 10),))
    d.ellipse([12, 12, 20, 20], outline=LCYAN, width=2)
    d.line([(13, 18), (19, 14)], fill=LCYAN)
    return img


def t_thrift(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    sign_band(d, (5, 8, 26, 27), LGRAY, lambda dd: tag(dd, 16, 13, OUT))
    return img


def t_thrift_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 7, 26, 26), ROOF_GRAY, ROOF_D, vents=((10, 19), (20, 12)))
    return img


def t_vfw(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    sign_band(d, (5, 8, 26, 27), IGREEN, lambda dd: star(dd, 16, 13, 3, WHITE))
    # flag pole beside
    d.line([(3, 6), (3, 24)], fill=OUT)
    d.rectangle([3, 6, 6, 9], fill=RED)
    return img


def t_vfw_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 7, 26, 26), ROOF_GRAY, ROOF_D, vents=((9, 12),))
    star(d, 16, 16, 3, IGREEN)
    return img


def scissors(d, cx, cy, color=WHITE):
    d.line([(cx - 3, cy - 3), (cx + 3, cy + 3)], fill=color, width=1)
    d.line([(cx - 3, cy + 3), (cx + 3, cy - 3)], fill=color, width=1)
    d.ellipse([cx - 5, cy - 4, cx - 2, cy - 1], outline=color)
    d.ellipse([cx - 5, cy + 1, cx - 2, cy + 4], outline=color)


def t_salon(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    sign_band(d, (5, 8, 26, 27), PINK, lambda dd: scissors(dd, 16, 13))
    # barber pole
    d.rectangle([28, 12, 30, 22], fill=WHITE, outline=OUT)
    d.line([(28, 15), (30, 13)], fill=RED, width=1)
    d.line([(28, 19), (30, 17)], fill=CYAN, width=1)
    return img


def t_salon_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 7, 26, 26), (150, 130, 138, 255), (118, 100, 108, 255),
              vents=((20, 19),))
    return img


def t_vet_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 7, 26, 26), ROOF_GRAY, ROOF_D, vents=((10, 11),))
    # paw print
    d.ellipse([14, 15, 18, 19], fill=IPINK)
    for (px, py) in ((12, 13), (15, 12), (18, 13)):
        d.ellipse([px, py, px + 2, py + 2], fill=IPINK)
    return img


def t_school(rng):
    img, d = ground(rng, GRASS, GRASS_D)
    building(d, (4, 10, 27, 26), LBLUE, (84, 128, 196, 255), door=False)
    d.rectangle([13, 22, 18, 26], fill=OUT)  # main doors
    for wx in range(6, 26, 4):
        d.rectangle([wx, 14, wx + 2, 17], fill=GLASS, outline=OUT)
    d.line([(16, 3), (16, 10)], fill=OUT)  # flagpole
    d.rectangle([16, 3, 20, 6], fill=RED)
    d.rectangle([6, 27, 25, 28], fill=CONCRETE_D)  # walk
    return img


def t_shootingrange_roof(rng):
    img, d = ground(rng, EARTH, EARTH_D)
    roof_plan(d, (4, 6, 27, 25), (140, 96, 84, 255), (108, 70, 62, 255),
              ridge=False)
    for r, c in ((6, RED), (4, WHITE), (2, RED)):
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=c, width=1)
    d.ellipse([15, 15, 17, 17], fill=RED)
    return img


def t_scrapyard(rng):
    img, d = ground(rng, EARTH_D, EARTH)
    d.rectangle([2, 2, 29, 29], outline=LGRAY, width=1)  # fence
    for (x, y, c) in ((7, 8, RED), (18, 7, URBAN_L), (8, 19, LBLUE),
                      (20, 18, DARKRED)):
        d.rectangle([x, y, x + 7, y + 4], fill=c, outline=OUT)  # crushed cars
        d.rectangle([x + 2, y - 2, x + 5, y], fill=c, outline=OUT)
    d.rectangle([24, 24, 28, 28], fill=ROOF_D, outline=OUT)  # crusher
    return img


# ---------------- rural buildings ----------------

def t_smokehouse(rng):
    img, d = ground(rng, GRASS, GRASS_D)
    building(d, (8, 12, 23, 26), WOOD, WOOD_D)
    d.rectangle([19, 5, 22, 12], fill=EARTH_D, outline=OUT)  # chimney
    for i in range(3):  # smoke curls
        d.arc([20 - i, 1 + i, 24 - i, 5 + i], 200, 340, fill=LGRAY)
    return img


def t_smokehouse_roof(rng):
    img, d = ground(rng, GRASS, GRASS_D)
    roof_plan(d, (8, 11, 23, 26), WOOD_D, (84, 56, 32, 255))
    d.rectangle([19, 12, 22, 15], fill=EARTH_D, outline=OUT)
    return img


def t_sugar_house(rng):
    img, d = ground(rng, GRASS, GRASS_D)
    building(d, (8, 12, 23, 26), WOOD_L, WOOD)
    d.rectangle([13, 5, 18, 12], fill=ROOF_D, outline=OUT)  # cupola/vent
    # maple leaf
    d.polygon([(16, 15), (14, 19), (15, 19), (13, 21), (19, 21), (17, 19),
               (18, 19)], fill=(214, 92, 40, 255))
    d.rectangle([24, 20, 27, 24], fill=LGRAY, outline=OUT)  # sap bucket
    return img


def t_sugar_parking(rng):
    img, d = ground(rng, ASPHALT, ASPHALT_D)
    for x in (5, 13, 21):
        d.line([(x, 6), (x, 14)], fill=WHITE)
        d.line([(x + 6, 6), (x + 6, 14)], fill=WHITE)
    d.rectangle([9, 20, 15, 24], fill=LRED, outline=OUT)  # a car
    d.rectangle([20, 21, 24, 23], fill=CONCRETE_D, outline=OUT)
    return img


def t_sugar_roof(rng):
    img, d = ground(rng, GRASS, GRASS_D)
    roof_plan(d, (8, 11, 23, 26), ROOF_RED if False else (150, 84, 60, 255),
              (116, 62, 44, 255))
    d.rectangle([13, 12, 18, 15], fill=ROOF_D, outline=OUT)  # vent hood
    return img


# ---------------- sewers & specials ----------------

def sewer_base(rng):
    img, d = ground(rng, (66, 70, 66, 255), (52, 56, 52, 255), n=30)
    d.rectangle([0, 0, 31, 31], outline=CONCRETE_D, width=2)
    return img, d


def t_sewer_connector(rng):
    img, d = sewer_base(rng)
    d.line([(16, 0), (16, 31)], fill=WATER_D if False else (40, 90, 60, 255),
           width=7)
    d.line([(0, 16), (31, 16)], fill=(40, 90, 60, 255), width=7)
    d.line([(16, 0), (16, 31)], fill=(64, 130, 88, 255), width=3)
    d.line([(0, 16), (31, 16)], fill=(64, 130, 88, 255), width=3)
    d.ellipse([12, 12, 20, 20], outline=CONCRETE, width=2)  # junction ring (X)
    return img


def t_urban_sewer(rng):
    img, d = sewer_base(rng)
    # corner bend entering from top, exiting right (sym right-angle corner)
    d.line([(16, 0), (16, 16)], fill=(40, 90, 60, 255), width=7)
    d.line([(16, 16), (31, 16)], fill=(40, 90, 60, 255), width=7)
    d.line([(16, 0), (16, 16)], fill=(64, 130, 88, 255), width=3)
    d.line([(16, 16), (31, 16)], fill=(64, 130, 88, 255), width=3)
    return img


def t_field_highway(rng):
    img, d = ground(rng, GRASS, GRASS_D, n=60)
    d.rectangle([13, 0, 18, 31], fill=ASPHALT)
    d.line([(15, 0), (15, 31)], fill=YELLOW)
    d.line([(16, 0), (16, 31)], fill=YELLOW)
    for (x, y) in ((5, 7), (24, 12), (7, 22), (25, 25)):
        d.ellipse([x, y, x + 2, y + 3], fill=GRASS_L)  # field tufts
    return img


def t_spider_pit(rng):
    img, d = ground(rng, EARTH_D, EARTH)
    d.ellipse([5, 5, 26, 26], fill=BLACK)
    d.ellipse([5, 5, 26, 26], outline=LGRAY)
    for ang in range(0, 360, 45):  # web radials
        a = math.radians(ang)
        d.line([(16, 16), (16 + 11 * math.cos(a), 16 + 11 * math.sin(a))],
               fill=WHITE)
    for r in (4, 7, 10):  # web rings
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=LGRAY)
    return img


def stadium(rng, left):
    img, d = ground(rng, URBAN, URBAN_D)
    # bowl: stands arc in the named corner, field in the opposite corner
    if left:
        d.rectangle([0, 0, 31, 31], fill=CONCRETE_D)
        d.rectangle([10, 10, 31, 31], fill=(70, 160, 66, 255))
        for r in (9, 6, 3):
            d.arc([-r, -r, 20 + r, 20 + r], 0, 90, fill=LGRAY, width=2)
        d.line([(16, 12), (16, 30)], fill=WHITE)
        d.line([(12, 21), (30, 21)], fill=WHITE)
    else:
        d.rectangle([0, 0, 31, 31], fill=CONCRETE_D)
        d.rectangle([0, 10, 21, 31], fill=(70, 160, 66, 255))
        for r in (9, 6, 3):
            d.arc([11 - r, -r, 31 + r, 20 + r], 90, 180, fill=LGRAY, width=2)
        d.line([(15, 12), (15, 30)], fill=WHITE)
        d.line([(1, 21), (19, 21)], fill=WHITE)
    return img


def t_standing_stones(rng, highlands=False):
    base = (104, 110, 84, 255) if highlands else GRASS
    speck = (84, 92, 66, 255) if highlands else GRASS_D
    img, d = ground(rng, base, speck, n=54)
    if highlands:
        for (x, y) in ((4, 24), (24, 5), (27, 22)):
            d.polygon([(x, y), (x + 4, y - 3), (x + 7, y)], fill=DGRAY)
    for ang in range(0, 360, 45):
        a = math.radians(ang)
        x, y = 16 + 8 * math.cos(a), 16 + 8 * math.sin(a)
        d.rectangle([x - 1, y - 3, x + 1, y + 2], fill=DGRAY, outline=OUT)
    d.ellipse([14, 14, 18, 18], fill=EARTH_L, outline=OUT)  # center altar
    return img


def t_temple(rng):
    img, d = ground(rng, GRASS_D, GRASS)
    d.polygon([(6, 26), (26, 26), (22, 18), (10, 18)], fill=(176, 158, 92, 255),
              outline=OUT)
    d.polygon([(10, 18), (22, 18), (19, 12), (13, 12)], fill=(196, 178, 108, 255),
              outline=OUT)
    d.rectangle([13, 6, 19, 12], fill=GOLD, outline=OUT)
    d.rectangle([15, 8, 17, 12], fill=DARKRED)  # doorway
    star(d, 16, 4, 2, YELLOW)
    return img


def t_unexplored(rng):
    img, d = ground(rng, (38, 38, 44, 255), (30, 30, 36, 255), n=20)
    for y in range(2, 31, 6):  # faint # hatch of the unknown cell
        for x in range(2, 31, 6):
            d.line([(x, y + 1), (x + 3, y + 1)], fill=(58, 58, 66, 255))
            d.line([(x + 1, y), (x + 1, y + 3)], fill=(58, 58, 66, 255))
    return img


def t_void_spider(rng):
    img, d = ground(rng, BLACK, (24, 20, 30, 255), n=16)
    for ang in range(0, 360, 60):
        a = math.radians(ang)
        d.line([(16, 15), (16 + 12 * math.cos(a), 15 + 12 * math.sin(a))],
               fill=(70, 62, 84, 255))
    for r in (5, 9):
        d.ellipse([16 - r, 15 - r, 16 + r, 15 + r], outline=(70, 62, 84, 255))
    d.ellipse([13, 12, 19, 18], fill=(30, 26, 38, 255))  # spider body
    for (ex, ey) in ((14, 13), (17, 13)):  # pale eyes
        d.point((ex, ey), fill=(220, 220, 235, 255))
    return img


def sinkhole(rng, base_deep):
    img, d = ground(rng, GRASS, GRASS_D)
    d.ellipse([3, 4, 28, 27], fill=EARTH_D)
    d.ellipse([5, 6, 26, 25], fill=EARTH)
    d.ellipse([8, 9, 23, 22], fill=(60, 42, 28, 255))
    d.ellipse([11, 11, 20, 20], fill=base_deep)
    d.arc([3, 4, 28, 27], 200, 320, fill=EARTH_L, width=2)  # crumbled rim
    return img


# ---------------- slime pit ----------------

def slime(rng, pool, deep, bubbles):
    img, d = ground(rng, EARTH_D, EARTH, n=26)
    d.ellipse([3, 5, 28, 27], fill=pool)
    d.ellipse([7, 9, 24, 23], fill=deep)
    for _ in range(bubbles):
        x, y = rng.randint(8, 22), rng.randint(10, 21)
        r = rng.randint(1, 2)
        d.ellipse([x - r, y - r, x + r, y + r], outline=SLIME)
    d.arc([6, 7, 20, 15], 180, 300, fill=WHITE)  # sheen
    return img


def t_slime_z0(rng):
    return slime(rng, SLIME, SLIME_D, 6)


def t_slime_z1(rng):
    return slime(rng, SLIME_D, (52, 122, 48, 255), 4)


def t_slime_z2(rng):
    return slime(rng, (52, 122, 48, 255), (18, 44, 20, 255), 2)


# ---------------- string dimension ----------------

def sd_base(rng):
    img = Image.new("RGBA", (32, 32), VOID)
    d = ImageDraw.Draw(img)
    for _ in range(7):  # distant specks
        d.point((rng.randint(1, 30), rng.randint(1, 30)), fill=(90, 90, 110, 255))
    return img, d


def t_sd_field_core(rng):
    img, d = sd_base(rng)
    for i in range(9):  # dense web of impossibly thin lines
        y = 4 + i * 3
        c = (STRING_W, STRING_C, STRING_M)[i % 3]
        d.line([(1, y), (30, 31 - y)], fill=c)
        d.line([(1, 31 - y), (30, y)], fill=c)
    d.ellipse([13, 13, 19, 19], outline=STRING_W)
    d.point((16, 16), fill=STRING_W)
    return img


def t_sd_field_end(rng):
    img, d = sd_base(rng)
    for i, y in enumerate((8, 14, 20, 25)):  # lines tapering off
        c = (STRING_W, STRING_C, STRING_M)[i % 3]
        d.line([(1, y), (14 + i * 3, 31 - y)], fill=c)
    d.point((25, 7), fill=STRING_C)
    return img


def t_sd_plane_core(rng):
    img, d = sd_base(rng)
    d.polygon([(7, 11), (25, 11), (28, 21), (4, 21)], fill=STRING_B)
    for x in (12, 17, 22):
        d.line([(x - 1, 11), (x - 2, 21)], fill=STRING_W)
    d.line([(5, 16), (27, 16)], fill=STRING_W)
    d.polygon([(7, 11), (25, 11), (28, 21), (4, 21)], outline=STRING_W)
    return img


def t_sd_plane_end(rng):
    img, d = sd_base(rng)
    d.polygon([(9, 13), (21, 13), (23, 19), (7, 19)], fill=(70, 92, 160, 255))
    d.polygon([(9, 13), (21, 13), (23, 19), (7, 19)], outline=STRING_B)
    d.line([(10, 16), (22, 16)], fill=STRING_B)
    d.point((25, 24), fill=STRING_B)  # fragment drifting away
    return img


# ---------------- triffid ----------------

def triffid_ground(rng):
    return ground(rng, (74, 148, 54, 255), (56, 122, 40, 255), n=54)


def shoot(d, x, y, h, color):
    d.line([(x, y), (x, y - h)], fill=color, width=1)
    d.polygon([(x, y - h), (x - 2, y - h + 2), (x + 2, y - h + 2)], fill=color)


def t_triffid_field(rng):
    img, d = triffid_ground(rng)
    for _ in range(9):
        shoot(d, rng.randint(3, 28), rng.randint(14, 29), rng.randint(3, 6),
              (150, 220, 90, 255))
    d.ellipse([13, 24, 19, 29], fill=(168, 84, 60, 255), outline=OUT)  # bulb
    return img


def t_triffid_grove(rng):
    img, d = triffid_ground(rng)
    for (x, y, s) in ((9, 13, 5), (22, 11, 6), (16, 22, 5), (7, 24, 4),
                      (25, 24, 4)):
        d.line([(x, y + s), (x, y)], fill=EARTH_D, width=2)  # stalk (T)
        d.ellipse([x - s, y - s, x + s, y + s // 2], fill=(104, 178, 66, 255),
                  outline=(70, 140, 44, 255))
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=LRED)  # frond heart
    return img


def t_triffid_roof(rng):
    img, d = ground(rng, (62, 128, 46, 255), (48, 106, 36, 255), n=30)
    for (x, y, s) in ((9, 10, 6), (22, 9, 7), (16, 20, 6), (8, 23, 5),
                      (25, 22, 5)):
        d.ellipse([x - s, y - s, x + s, y + s], fill=(104, 178, 66, 255),
                  outline=(70, 140, 44, 255))
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=LRED)
    return img


def t_triffid_finale(rng):
    img, d = triffid_ground(rng)
    for ang in range(0, 360, 30):  # root crown
        a = math.radians(ang)
        d.line([(16, 17), (16 + 12 * math.cos(a), 17 + 11 * math.sin(a))],
               fill=(120, 60, 40, 255), width=2)
    d.ellipse([9, 8, 23, 24], fill=RED, outline=DARKRED)  # the heart
    d.ellipse([12, 11, 17, 16], fill=LRED)  # pulse highlight
    d.line([(16, 24), (16, 29)], fill=(120, 60, 40, 255), width=2)
    return img


# ---------------- trailers ----------------

def trailer(d, rect, wall, trim):
    x0, y0, x1, y1 = rect
    d.rectangle(rect, fill=wall, outline=OUT)
    for y in range(y0 + 3, y1, 4):  # siding ribs
        d.line([(x0 + 1, y), (x1 - 1, y)], fill=trim)
    d.rectangle([x0 + 3, y0 + 4, x0 + 6, y0 + 7], fill=GLASS, outline=OUT)
    d.rectangle([x1 - 6, y0 + 4, x1 - 3, y0 + 7], fill=GLASS, outline=OUT)
    d.rectangle([14, y1 - 6, 17, y1 - 1], fill=OUT)  # door
    d.ellipse([x0 + 2, y1, x0 + 6, y1 + 3], fill=OUT)  # wheels
    d.ellipse([x1 - 6, y1, x1 - 2, y1 + 3], fill=OUT)


def t_trailer_laundromat(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    trailer(d, (4, 10, 27, 25), CREAM, CONCRETE_D)
    for x in (8, 14, 20):  # washer portholes on the sign band
        d.ellipse([x, 12, x + 3, 15], fill=GLASS, outline=OUT)
    return img


def t_trailer_store(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    trailer(d, (4, 10, 27, 25), CREAM, CONCRETE_D)
    d.rectangle([5, 11, 26, 15], fill=RED)  # store awning band
    d.rectangle([6, 18, 12, 22], fill=GLASS, outline=OUT)  # shop window
    return img


def t_trailer_hub(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    building(d, (5, 9, 26, 26), (196, 232, 236, 255), (150, 190, 198, 255),
             door=False)
    d.rectangle([13, 21, 18, 26], fill=OUT)
    d.rectangle([5, 9, 26, 13], fill=CYAN)  # admin band
    star(d, 16, 11, 2, WHITE)
    return img


def t_trailer_hub_roof(rng):
    img, d = ground(rng, URBAN, URBAN_D)
    roof_plan(d, (5, 8, 26, 25), ROOF_GRAY, ROOF_D, vents=((9, 11), (21, 19)))
    d.line([(5, 8), (26, 8)], fill=CYAN, width=2)  # cyan trim edge
    return img


def t_trailer_public(rng):
    img, d = ground(rng, GRASS, GRASS_D)
    d.polygon([(5, 10), (27, 10), (24, 6), (8, 6)], fill=(96, 150, 70, 255),
              outline=OUT)  # pavilion roof
    for x in (7, 15, 24):  # open posts
        d.line([(x, 12), (x, 25)], fill=WOOD_D, width=2)
    d.rectangle([6, 20, 26, 22], fill=WOOD)  # picnic bench
    d.rectangle([6, 25, 26, 27], fill=CONCRETE_D)  # slab
    return img


DRAW = {
    "s_music": t_music,
    "s_music_roof": t_music_roof,
    "s_sports_roof": t_sports_roof,
    "s_thrift": t_thrift,
    "s_thrift_roof": t_thrift_roof,
    "s_vfw": t_vfw,
    "s_vfw_roof": t_vfw_roof,
    "salon": t_salon,
    "salon_roof": t_salon_roof,
    "school_1_2": t_school,
    "sewer_connector": t_sewer_connector,
    "shootingrange_1a_roof": t_shootingrange_roof,
    "slimepit_z-1": t_slime_z1,
    "slimepit_z-2": t_slime_z2,
    "slimepit_z0": t_slime_z0,
    "smallscrapyard": t_scrapyard,
    "smokehouse": t_smokehouse,
    "smokehouse_roof": t_smokehouse_roof,
    "special_field_highway": t_field_highway,
    "spider_pit": t_spider_pit,
    "stadium_football_0_0": lambda r: stadium(r, True),
    "stadium_football_8_0": lambda r: stadium(r, False),
    "standing_stones": lambda r: t_standing_stones(r, False),
    "standing_stones_highlands": lambda r: t_standing_stones(r, True),
    "string_dimension_field_core": t_sd_field_core,
    "string_dimension_field_end": t_sd_field_end,
    "string_dimension_plane_core": t_sd_plane_core,
    "string_dimension_plane_end": t_sd_plane_end,
    "sugar_house": t_sugar_house,
    "sugar_house_parking": t_sugar_parking,
    "sugar_house_roof": t_sugar_roof,
    "temple_finale": t_temple,
    "trailer_commercial_00": t_trailer_laundromat,
    "trailer_commercial_10": t_trailer_store,
    "trailer_hub_10": t_trailer_hub,
    "trailer_hub_10_roof": t_trailer_hub_roof,
    "trailer_public_10": t_trailer_public,
    "triffid_field": t_triffid_field,
    "triffid_finale": t_triffid_finale,
    "triffid_grove": t_triffid_grove,
    "triffid_grove_roof": t_triffid_roof,
    "unexplored": t_unexplored,
    "urban_25_sewer_connector": t_urban_sewer,
    "veterinarian_roof": t_vet_roof,
    "void_spider_lair": t_void_spider,
    "ws_giant_sinkhole_1": lambda r: sinkhole(r, (24, 16, 12, 255)),
    "ws_giant_sinkhole_2": lambda r: sinkhole(r, BLACK),
}


def main():
    ids = sorted(DRAW)
    assert len(ids) == 47, len(ids)
    for tid in ids:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = DRAW[tid](rng)
        assert img.size == (32, 32)
        alphas = set(img.getchannel("A").getdata())
        assert alphas == {255}, (tid, alphas)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", tid)

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
                       "preview_om7.png")
    sheet.save(out)
    print("wrote preview_om7.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
