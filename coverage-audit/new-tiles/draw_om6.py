#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch OM6: 48 overmap terrain tiles.

Overmap tiles are map cells: every sprite is a FULL-TILE OPAQUE 32x32 --
a ground/base fill in the location's data color with a centered iconic
pictogram, bold and readable at map zoom. *_roof ids are the roof plan
of their base building (same footprint/palette, roof texture);
*_2ndfloor is the upper-floor plan. Drawn from each entry's
name/symbol/color in fork-src/cdda/data/json. Flat UltiCa overmap
style. Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_om6.png.

Data notes: the s_* ground floors are city_building ids whose
overmap_terrain entries give name/sym/color (antique A/brown, arcade
A/cyan, daycare/blue, games g/cyan, gardening F/green, jewelry
J/white). outcrop is the surface rock outcropping (O/brown);
outcrop_bottom is its underground base. The portal teleporter is
'elsewhere' (T/black, nether glass) -- its vortex mirrors the trap
batches' portal family in nether violet. Ravines grade rim -> edge
-> floor -> floor edge by depth shading. The river cave set is one
flooded cave system: tunnel, food storage, kraken chamber, nest.
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
EARTH = (122, 86, 54, 255)
EARTH_D = (88, 60, 38, 255)
EARTH_L = (152, 112, 76, 255)
URBAN = (130, 130, 134, 255)
URBAN_D = (102, 102, 106, 255)
URBAN_L = (158, 158, 162, 255)
CONCRETE = (168, 168, 164, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (106, 72, 40, 255)
WOOD_L = (178, 136, 88, 255)
ROOF_BROWN = (128, 76, 48, 255)
ROOF_BROWN_D = (94, 54, 34, 255)
ROOF_GRAY = (116, 116, 120, 255)
ROOF_GRAY_D = (84, 84, 88, 255)
WHITE = (236, 236, 236, 255)
CREAM = (240, 232, 196, 255)
LGRAY = (178, 178, 178, 255)
BLACK = (16, 14, 16, 255)
YELLOW = (232, 202, 66, 255)
PINK = (240, 156, 186, 255)
RED = (178, 44, 36, 255)
DARKRED = (128, 30, 26, 255)
BRICK = (156, 66, 50, 255)
BRICK_D = (116, 46, 36, 255)
LBLUE = (112, 162, 232, 255)
DBLUE = (44, 84, 160, 255)
WATER = (52, 110, 190, 255)
WATER_D = (34, 76, 140, 255)
WATER_L = (96, 156, 224, 255)
ROCK = (138, 134, 128, 255)
ROCK_D = (104, 100, 94, 255)
ROCK_L = (172, 168, 160, 255)
CAVE = (96, 78, 62, 255)           # cave rock earth
CAVE_D = (64, 50, 40, 255)
CAVE_L = (128, 108, 84, 255)
GRAVEL = (156, 150, 138, 255)
GRAVEL_D = (128, 122, 108, 255)
GRAVEL_L = (182, 176, 162, 255)
ASPHALT = (92, 92, 98, 255)
ASPHALT_D = (70, 70, 76, 255)
CYAN = (84, 200, 210, 255)
CYAN_D = (44, 140, 152, 255)
LGREEN = (140, 190, 100, 255)      # prefab mobile home, light_green
LGREEN_D = (104, 152, 70, 255)
POOL = (64, 178, 160, 255)         # cyan_green kraken pool
POOL_D = (36, 120, 110, 255)
POOL_L = (120, 216, 196, 255)
TENT = (140, 52, 84, 255)          # kraken tentacle dark magenta-red
TENT_D = (96, 34, 60, 255)
TENT_L = (186, 92, 120, 255)
VIOLET = (150, 110, 220, 255)      # nether portal
VIOLET_D = (88, 54, 150, 255)
VIOLET_L = (206, 178, 250, 255)
BLUE_STONE = (110, 128, 168, 255)  # ruined tower, c_blue stone
BLUE_STONE_D = (76, 92, 124, 255)
BLUE_STONE_L = (148, 164, 200, 255)


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


def storefront(rng, wall, accent, pictogram):
    """The classic s_* shop row: urban lot, sign band, windows, door."""
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    d.rectangle([0, 24, 31, 31], fill=ASPHALT_D)             # parking strip
    building(d, (4, 5, 28, 24), wall, URBAN_D, door=False)
    d.rectangle([4, 5, 28, 9], fill=accent, outline=OUT)     # sign band
    d.rectangle([7, 13, 12, 20], fill=LBLUE, outline=OUT)    # windows
    d.rectangle([20, 13, 25, 20], fill=LBLUE, outline=OUT)
    d.rectangle([14, 14, 18, 24], fill=URBAN_D, outline=OUT)  # door
    pictogram(d)
    return img


def shop_roof(rng, accent, vents=((10, 11), (22, 18))):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (4, 5, 28, 24), ROOF_GRAY, ROOF_GRAY_D, ridge=False,
              vents=vents)
    d.rectangle([4, 5, 28, 7], fill=accent, outline=OUT)     # fascia stripe
    d.rectangle([13, 13, 19, 17], fill=LGRAY, outline=OUT)   # HVAC box
    return img


# ------------------------------------------------------------- orchard

def t_orchard_stall_roof(rng):
    # 'orchard stall roof', sym T, color light_green: small stall roof
    # plan over the orchard plot -- timber frame, light-green tarp top.
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 26)
    for (x, y) in ((4, 5), (27, 5), (4, 27), (27, 27)):
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=GRASS_D)
    x0, y0, x1, y1 = 8, 10, 24, 22
    d.rectangle([x0, y0, x1, y1], fill=LGREEN, outline=OUT)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=LGREEN_D)
    cy = (y0 + y1) // 2
    d.line([(x0 + 2, cy), (x1 - 2, cy)], fill=LGREEN_D, width=1)
    for x in range(x0 + 3, x1 - 1, 4):
        d.line([(x, y0 + 2), (x, cy)], fill=LGREEN_D)
        d.line([(x, cy), (x, y1 - 2)], fill=LGREEN_D)
    for (px, py) in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)):
        d.rectangle([px - 1, py - 1, px + 1, py + 1], fill=WOOD_D)
    return img


# ------------------------------------------------------------- outcrop

def rock_dome(d, cx, cy, rx, ry, top, mid, dark):
    d.ellipse([cx - rx, cy - ry + 3, cx + rx, cy + ry], fill=dark)
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry - 1], fill=mid)
    d.ellipse([cx - rx + 3, cy - ry + 1, cx + rx - 4, cy + 1], fill=top)
    d.line([(cx - rx + 4, cy + 2), (cx - 1, cy + ry - 3)], fill=dark)
    d.line([(cx + 3, cy + 1), (cx + rx - 3, cy + ry - 2)], fill=dark)


def t_outcrop(rng):
    # 'outcrop', sym O, color brown: surface rock outcropping
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L, EARTH_L], 30)
    rock_dome(d, 16, 16, 11, 9, ROCK_L, ROCK, ROCK_D)
    d.ellipse([11, 10, 15, 13], fill=ROCK_L)                 # bright cap
    for (x, y) in ((5, 24), (26, 22), (7, 7)):
        d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=ROCK, outline=ROCK_D)
    return img


def t_outcrop_bottom(rng):
    # 'outcrop base', underground: the dome's base embedded in dirt
    img, d = ground(rng, EARTH_D, [CAVE_D, EARTH], 40)
    rock_dome(d, 16, 17, 12, 10, ROCK, ROCK_D, CAVE_D)
    d.ellipse([12, 12, 16, 15], fill=ROCK)
    for _ in range(8):
        x, y = rng.randint(1, 30), rng.randint(1, 30)
        d.point((x, y), fill=CAVE_L)
    return img


# -------------------------------------------------------------- portal

def t_portal_teleporter(rng):
    # 'elsewhere', sym T, color black: nether-glass portal vortex.
    # Mirrors the trap batches' two-arm spiral in nether violet.
    img, d = ground(rng, (10, 8, 14, 255), [(24, 18, 32, 255)], 24)
    cx = cy = 15.5
    d.ellipse([2.5, 2.5, 29.5, 29.5], fill=(16, 12, 24, 255))
    d.ellipse([2.5, 2.5, 29.5, 29.5], outline=VIOLET_D)
    d.ellipse([5.5, 5.5, 26.5, 26.5], fill=VIOLET_D)
    for arm in (0.0, math.pi):
        pts = []
        steps = 46
        for k in range(steps + 1):
            t = k / steps
            ang = arm + t * 3.9 * math.pi + rng.uniform(-0.02, 0.02)
            r = 1.0 + t * 9.6
            pts.append((cx + r * math.cos(ang), cy + r * math.sin(ang)))
        d.line(pts, fill=VIOLET, width=3)
        d.line(pts, fill=VIOLET_L, width=1)
    d.ellipse([cx - 3.2, cy - 3.2, cx + 3.2, cy + 3.2],
              fill=(240, 232, 255, 255))
    d.ellipse([cx - 1.4, cy - 1.4, cx + 1.4, cy + 1.4], fill=WHITE)
    # glass shard glints at the rim
    for (x, y) in ((6, 9), (25, 7), (27, 23), (5, 25)):
        d.point((x, y), fill=VIOLET_L)
    return img


# -------------------------------------------------------------- prefab

def t_prefab(rng):
    # 'mobile home', sym ^, color light_green
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 22)
    d.rectangle([0, 26, 31, 31], fill=ASPHALT_D)             # street
    d.rectangle([3, 10, 29, 21], fill=LGREEN, outline=OUT)   # long home
    d.rectangle([3, 10, 29, 12], fill=LGREEN_D, outline=OUT)  # roof band
    for x in (6, 12, 21):
        d.rectangle([x, 14, x + 3, 17], fill=LBLUE, outline=OUT)
    d.rectangle([25, 14, 27, 21], fill=WOOD_D, outline=OUT)  # door
    d.rectangle([24, 21, 28, 23], fill=CONCRETE)             # steps
    d.line([(3, 20), (29, 20)], fill=LGREEN_D)                # skirting
    d.rectangle([5, 24, 9, 25], fill=GRASS_D)                 # planter strip
    return img


def t_prefab_roof(rng):
    # 'mobile home roof': long low-slope roof plan, pale metal seams
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    x0, y0, x1, y1 = 3, 10, 29, 21
    d.rectangle([x0, y0, x1, y1], fill=(196, 200, 188, 255), outline=OUT)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2],
                outline=(150, 154, 142, 255))
    cy = (y0 + y1) // 2
    d.line([(x0 + 2, cy), (x1 - 2, cy)], fill=(150, 154, 142, 255),
           width=1)
    for x in range(x0 + 5, x1 - 2, 6):
        d.line([(x, y0 + 2), (x, y1 - 2)], fill=(168, 172, 160, 255))
    d.rectangle([15, 12, 18, 14], fill=LGRAY, outline=OUT)   # vent stack
    return img


# -------------------------------------------------------- public works
# public_works_SW is the south-west quadrant of a 2x2 depot: the
# warehouse mass continues off the north and east edges.

def t_public_works(rng):
    img, d = ground(rng, CONCRETE, [URBAN, URBAN_L], 26)
    d.rectangle([11, 0, 31, 15], fill=LGRAY, outline=OUT)    # warehouse
    d.rectangle([13, 2, 29, 13], outline=URBAN_D)
    d.rectangle([22, 8, 28, 15], fill=URBAN_D, outline=OUT)  # loading door
    d.line([(22, 11), (28, 11)], fill=LGRAY)
    # yard: crates, pipe stack, drums
    d.rectangle([3, 19, 9, 25], fill=WOOD, outline=OUT)
    d.line([(3, 22), (9, 22)], fill=WOOD_D)
    for i, x in enumerate((13, 16, 19)):
        d.ellipse([x, 21, x + 3, 24], fill=URBAN_D if i % 2 else CYAN_D,
                  outline=OUT)
    for y in (27, 29):
        d.line([(3, y), (16, y)], fill=LGRAY, width=1)       # pipes
    d.rectangle([24, 20, 29, 26], fill=WOOD_L, outline=OUT)  # pallet stack
    return img


def t_public_works_roof(rng):
    img, d = ground(rng, CONCRETE, [URBAN, URBAN_L], 20)
    # roof slab over the same N+E footprint; SW yard is open air below
    d.rectangle([11, 0, 31, 15], fill=ROOF_GRAY, outline=OUT)
    d.rectangle([13, 2, 29, 13], outline=ROOF_GRAY_D)
    for x in range(15, 30, 5):
        d.line([(x, 2), (x, 13)], fill=ROOF_GRAY_D)
    d.rectangle([17, 5, 21, 9], fill=LGRAY, outline=OUT)     # HVAC
    d.rectangle([3, 19, 9, 25], outline=URBAN_D)             # yard from above
    return img


# --------------------------------------------------------- pump station

def pump_station(rng, variant):
    # 'pump station', sym P, color red: small red-brick utility block
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    d.rectangle([0, 25, 31, 31], fill=ASPHALT_D)
    building(d, (7, 6, 25, 21), BRICK, BRICK_D, door=False)
    d.rectangle([7, 6, 25, 9], fill=WHITE, outline=OUT)      # sign band
    d.polygon([(14, 7), (18, 7), (18, 9), (14, 9)], fill=RED)  # 'P' block
    d.rectangle([10, 12, 14, 17], fill=LBLUE, outline=OUT)   # window
    d.rectangle([19, 12, 22, 21], fill=URBAN_D, outline=OUT)  # door
    if variant == 1:
        # tall standpipe + valve wheel on the yard side
        d.rectangle([27, 8, 30, 21], fill=LGRAY, outline=OUT)
        d.ellipse([26, 6, 31, 9], fill=URBAN_D)
        d.ellipse([15, 22, 19, 26], outline=RED, width=2)    # valve wheel
        d.point((17, 24), fill=RED)
    else:
        # horizontal pipe run with two valve wheels + yard hatch
        d.line([(2, 23), (29, 23)], fill=LGRAY, width=3)
        for x in (8, 22):
            d.ellipse([x - 2, 19, x + 2, 23], outline=RED, width=2)
            d.point((x, 21), fill=RED)
        d.rectangle([13, 26, 19, 29], fill=URBAN_D, outline=OUT)
    return img


# ------------------------------------------------------------ substation

def substation(rng, roof=False):
    # 'small power substation', sym H, color light_cyan
    img, d = ground(rng, GRAVEL, [GRAVEL_D, GRAVEL_L], 34)
    # chain-link border: dashed light lines + posts
    for t in range(1, 31, 3):
        d.point((t, 1), fill=LGRAY)
        d.point((t, 30), fill=LGRAY)
        d.point((1, t), fill=LGRAY)
        d.point((30, t), fill=LGRAY)
    for (px, py) in ((1, 1), (30, 1), (1, 30), (30, 30)):
        d.rectangle([px, py, px + 1, py + 1], fill=URBAN_D)
    # H frame: two poles + crossbar, insulator dots (the data sym)
    for px in (9, 22):
        d.rectangle([px, 5, px + 1, 15], fill=WOOD_D)
    d.line([(9, 7), (23, 7)], fill=WOOD_D, width=2)
    for x in (11, 16, 21):
        d.ellipse([x - 1, 5, x + 1, 7], fill=CYAN, outline=CYAN_D)
    d.line([(11, 8), (11, 12)], fill=CYAN_D)                 # drop wires
    d.line([(21, 8), (21, 12)], fill=CYAN_D)
    # transformer boxes: finned sides, bushing studs on top
    for x0 in (6, 19):
        d.rectangle([x0, 17, x0 + 8, 26], fill=(122, 134, 118, 255),
                    outline=OUT)
        for fx in range(x0 + 1, x0 + 8, 2):
            d.line([(fx, 18), (fx, 25)], fill=(88, 100, 84, 255))
        for bx in (x0 + 2, x0 + 5):
            d.rectangle([bx, 15, bx + 1, 17], fill=URBAN_D)
            d.point((bx, 14), fill=CYAN)
    if roof:
        # open-air yard seen at roof level: paler, tops only
        d.rectangle([6, 17, 14, 26], outline=CYAN_D)
        d.rectangle([19, 17, 27, 26], outline=CYAN_D)
    return img


# ------------------------------------------------------------ ranch camp

def t_ranch_barn_roof(rng):
    # ranch_camp_65_roof 'barn roof', sym #, color brown: big shingle
    # roof plan over ranch grass
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 22)
    x0, y0, x1, y1 = 4, 6, 28, 26
    d.rectangle([x0, y0, x1, y1], fill=ROOF_BROWN, outline=OUT)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=ROOF_BROWN_D)
    cy = (y0 + y1) // 2
    d.line([(x0 + 2, cy), (x1 - 2, cy)], fill=ROOF_BROWN_D, width=2)
    for y in range(y0 + 4, y1 - 1, 3):
        if y != cy:
            d.line([(x0 + 2, y), (x1 - 2, y)], fill=ROOF_BROWN_D)
    d.rectangle([14, 9, 18, 12], fill=WOOD_D, outline=OUT)   # loft vent
    return img


def t_ranch_garage(rng):
    # ranch_camp_67 'garage', sym O, color white
    img, d = ground(rng, GRASS, [GRASS_D, EARTH_L], 26)
    d.rectangle([12, 22, 20, 31], fill=EARTH)                # dirt apron
    building(d, (6, 6, 26, 22), WHITE, LGRAY, door=False)
    d.rectangle([10, 11, 22, 22], fill=LGRAY, outline=OUT)   # wide door
    d.line([(10, 16), (22, 16)], fill=URBAN_D)
    d.line([(10, 11), (22, 22)], fill=URBAN_L)
    d.line([(22, 11), (10, 22)], fill=URBAN_L)
    d.rectangle([7, 7, 10, 10], fill=LBLUE, outline=OUT)     # window
    return img


def t_ranch_garage_roof(rng):
    # 'garage roof', color white: pale painted-metal roof plan
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    x0, y0, x1, y1 = 6, 6, 26, 22
    d.rectangle([x0, y0, x1, y1], fill=(214, 214, 210, 255), outline=OUT)
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2],
                outline=(172, 172, 168, 255))
    cy = (y0 + y1) // 2
    d.line([(x0 + 2, cy), (x1 - 2, cy)], fill=(172, 172, 168, 255),
           width=1)
    for x in range(x0 + 5, x1 - 2, 5):
        d.line([(x, y0 + 2), (x, y1 - 2)], fill=(188, 188, 184, 255))
    return img


# ---------------------------------------------------------------- ravine
# Depth grade: ravine (grass rim, deep crack, blue water glint) ->
# ravine_edge (slope face) -> ravine_floor (rocky bottom + stream)
# -> ravine_floor_edge (floor against the rising wall).

def t_ravine(rng):
    img, d = ground(rng, GRASS, [GRASS_D, ROCK], 26)
    # jagged chasm across the middle, walls shaded, water at the bottom
    top = [(0, 11), (7, 13), (13, 10), (20, 13), (26, 11), (31, 13)]
    bot = [(0, 21), (8, 19), (14, 22), (21, 19), (27, 21), (31, 20)]
    d.polygon(top + bot[::-1], fill=ROCK_D)
    inner_t = [(0, 14), (7, 16), (13, 13), (20, 16), (26, 14), (31, 16)]
    inner_b = [(0, 18), (8, 16), (14, 19), (21, 16), (27, 18), (31, 17)]
    d.polygon(inner_t + inner_b[::-1], fill=(24, 22, 26, 255))
    mid = [(0, 15), (7, 17), (13, 15), (20, 17), (26, 15), (31, 16)]
    d.line(mid, fill=WATER, width=2)                         # data: blue
    d.line([(4, 15), (9, 16)], fill=WATER_L, width=1)
    return img


def t_ravine_edge(rng):
    # 'cliffside', sym v: rim at top grading down a strata face
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 16, box=(0, 0, 31, 6))
    d.rectangle([0, 7, 31, 31], fill=ROCK)
    bands = [(7, ROCK_L), (11, ROCK), (15, ROCK_D), (20, (84, 80, 76, 255)),
             (25, (58, 54, 52, 255))]
    for y, c in bands:
        d.rectangle([0, y, 31, y + 2], fill=c)
    for _ in range(12):
        x, y = rng.randint(0, 30), rng.randint(9, 29)
        d.line([(x, y), (x + 1, y + 2)], fill=ROCK_D)        # scree streaks
    # the 'v' drop mark
    d.polygon([(13, 12), (19, 12), (16, 17)], fill=WHITE)
    return img


def t_ravine_floor(rng):
    # 'ravine floor', sym .: rocky bottom with a stream along the south
    img, d = ground(rng, ROCK, [ROCK_D, ROCK_L, CAVE_L], 44)
    for (x, y, r) in ((6, 7, 2), (24, 10, 3), (10, 18, 2), (27, 24, 2)):
        d.ellipse([x - r, y - r, x + r, y + r], fill=ROCK_D)
        d.ellipse([x - r, y - r, x, y], fill=ROCK_L)
    d.polygon([(0, 27), (8, 25), (16, 28), (24, 26), (31, 28),
               (31, 31), (0, 31)], fill=WATER_D)
    d.line([(2, 28), (12, 27), (22, 28), (30, 29)], fill=WATER_L,
           width=1)
    return img


def t_ravine_floor_edge(rng):
    # 'cliffside', sym ^: floor below, the wall rising along the north
    img, d = ground(rng, ROCK, [ROCK_D, ROCK_L], 30, box=(0, 10, 31, 31))
    d.rectangle([0, 0, 31, 9], fill=ROCK_D)                  # wall face
    for y, c in ((1, ROCK), (4, ROCK_L), (7, ROCK_D)):
        d.line([(0, y), (31, y)], fill=c, width=1)
    d.rectangle([0, 10, 31, 12], fill=(40, 36, 34, 255))     # base shadow
    for (x, y) in ((7, 20, ), (22, 25), (26, 16)):
        d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=ROCK_D)  # fallen rocks
    d.polygon([(13, 24), (19, 24), (16, 19)], fill=WHITE)    # '^' mark
    return img


# --------------------------------------------------------- refugee fence
# Evac-center grounds; the fence runs on the edges named by the sym:
# NE ┐ = north+east, NW ┌ = north+west, SE ┘ = south+east,
# SW └ = south+west, v │ = a vertical run through the cell.

def fence_run(d, x0, y0, x1, y1):
    d.line([(x0, y0), (x1, y1)], fill=LGRAY, width=1)
    if x0 == x1:
        for y in range(min(y0, y1), max(y0, y1) + 1, 4):
            d.rectangle([x0 - 1, y, x0 + 1, y + 1], fill=URBAN_D)
    else:
        for x in range(min(x0, x1), max(x0, x1) + 1, 4):
            d.rectangle([x, y0 - 1, x + 1, y0 + 1], fill=URBAN_D)


def fence_cell(rng, edges):
    img, d = ground(rng, GRASS, [GRASS_D, GRAVEL_D, GRASS_L], 30)
    d.ellipse([10, 12, 20, 19], fill=GRAVEL)                 # worn patch
    for _ in range(10):
        x, y = rng.randint(11, 19), rng.randint(13, 18)
        d.point((x, y), fill=GRAVEL_D)
    if "N" in edges:
        fence_run(d, 0, 1, 31, 1)
    if "S" in edges:
        fence_run(d, 0, 30, 31, 30)
    if "W" in edges:
        fence_run(d, 1, 0, 1, 31)
    if "E" in edges:
        fence_run(d, 30, 0, 30, 31)
    if "V" in edges:
        fence_run(d, 15, 0, 15, 31)
    return img


# ------------------------------------------------------------ river cave
# One flooded cave system in brown rock (vision: underground dirt).

def cave(rng, speck_n=40):
    return ground(rng, CAVE, [CAVE_D, CAVE_L, EARTH_D], speck_n)


def t_river_cave(rng):
    # 'river cave tunnel', color brown, LINEAR: water channel through
    # a rock tunnel, flowing west-east
    img, d = cave(rng)
    d.rectangle([0, 12, 31, 20], fill=WATER_D)
    d.rectangle([0, 13, 31, 19], fill=WATER)
    for _ in range(14):
        x, y = rng.randint(0, 30), rng.randint(14, 18)
        d.point((x, y), fill=WATER_L)
    d.rectangle([0, 10, 31, 12], fill=CAVE_L)                # north bank
    d.rectangle([0, 20, 31, 22], fill=CAVE_D)                # south shadow
    for (x, y) in ((5, 5), (25, 6), (14, 26), (28, 27)):
        d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=CAVE_D)  # boulders
    return img


def cave_chamber(rng):
    img, d = cave(rng)
    # pool occupying the chamber's south half
    d.ellipse([3, 13, 29, 30], fill=POOL_D)
    d.ellipse([5, 15, 27, 29], fill=POOL)
    for _ in range(10):
        x, y = rng.randint(6, 26), rng.randint(17, 27)
        d.point((x, y), fill=POOL_L)
    d.rectangle([0, 10, 31, 12], fill=CAVE_L)                # shore ledge
    return img, d


def t_river_cave_food(rng):
    # 'kraken food storage', sym O, color cyan_green: shore piles of
    # fish and meat beside the pool
    img, d = cave_chamber(rng)
    for (x, y) in ((6, 6), (13, 5), (21, 7)):
        d.ellipse([x, y, x + 5, y + 3], fill=(206, 214, 196, 255),
                  outline=CAVE_D)                            # fish pile
        d.polygon([(x + 5, y + 1), (x + 8, y), (x + 8, y + 3)],
                  fill=(170, 178, 160, 255))
    d.ellipse([24, 4, 30, 9], fill=(150, 84, 62, 255), outline=CAVE_D)
    d.ellipse([25, 5, 28, 7], fill=(178, 110, 88, 255))      # meat heap
    return img


def t_river_cave_kraken(rng):
    # 'kraken chamber', sym O, color red: tentacles rise from the pool
    img, d = cave_chamber(rng)
    d.ellipse([3, 13, 29, 30], fill=POOL_D)                  # deeper water
    d.ellipse([6, 16, 26, 29], fill=(28, 92, 88, 255))
    for (bx, sway) in ((9, -2), (16, 1), (23, -1)):
        pts = []
        for k in range(9):
            t = k / 8
            x = bx + sway * math.sin(t * 2.2)
            y = 27 - t * 13
            pts.append((x, y))
        for w, c in ((5, TENT_D), (3, TENT)):
            d.line(pts, fill=c, width=w)
        for k in range(2, 8, 2):                             # suckers
            t = k / 8
            x = bx + sway * math.sin(t * 2.2)
            y = 27 - t * 13
            d.point((int(x), int(y)), fill=TENT_L)
        d.ellipse([pts[-1][0] - 2, pts[-1][1] - 2, pts[-1][0] + 2,
                   pts[-1][1] + 2], fill=TENT)
    d.ellipse([13, 21, 19, 24], fill=TENT_D)                 # submerged bulk
    return img


def t_river_cave_nest(rng):
    # 'river cave nest', sym O, color white: twig nest with pale eggs
    # on the shore ledge
    img, d = cave_chamber(rng)
    cx, cy = 16, 7
    for r, c in ((8, WOOD_D), (6, WOOD), (4, EARTH_L)):
        d.ellipse([cx - r, cy - r + 2, cx + r, cy + r], fill=c)
    for ang in range(0, 360, 30):
        x = cx + int(7 * math.cos(math.radians(ang)))
        y = cy + 1 + int(4 * math.sin(math.radians(ang)))
        d.line([(cx, cy + 1), (x, y)], fill=WOOD_D)
    for (ex, ey) in ((13, 6), (17, 5), (16, 9)):
        d.ellipse([ex - 2, ey - 2, ex + 2, ey + 2], fill=WHITE,
                  outline=LGRAY)                             # pale eggs
        d.point((ex, ey - 1), fill=(210, 200, 220, 255))
    return img


# --------------------------------------------------------------- robofac

def t_robofac_mall_roof(rng):
    # robofac_strip_mall_roof_5, name 'open air', sym ., color blue:
    # the mall's open-air court at roof level -- roof ring around a
    # pale court with blue glass panels
    img, d = ground(rng, ROOF_GRAY, [ROOF_GRAY_D, URBAN_L], 22)
    d.rectangle([2, 2, 29, 29], outline=OUT, width=1)
    d.rectangle([6, 6, 25, 25], fill=CONCRETE, outline=OUT)  # open court
    for _ in range(16):
        x, y = rng.randint(7, 24), rng.randint(7, 24)
        d.point((x, y), fill=URBAN_L)
    d.rectangle([9, 9, 14, 14], fill=LBLUE, outline=DBLUE)   # glass panel
    d.line([(9, 11), (14, 11)], fill=DBLUE)
    d.rectangle([18, 18, 23, 23], fill=LBLUE, outline=DBLUE)
    d.ellipse([19, 8, 24, 12], fill=GRASS_D)                 # planter
    d.ellipse([20, 9, 23, 11], fill=GRASS_L)
    for (vx, vy) in ((4, 4), (27, 4), (4, 27), (27, 27)):
        d.rectangle([vx - 1, vy - 1, vx + 1, vy + 1], fill=ROOF_GRAY_D,
                    outline=OUT)
    return img


def t_robofachq_parking(rng):
    # 'hub 01 parking space', sym 0, color light_blue
    img, d = ground(rng, ASPHALT, [ASPHALT_D, URBAN], 22)
    for x in (2, 29):
        d.line([(x, 2), (x, 29)], fill=LBLUE, width=1)       # stall lines
    d.line([(2, 2), (29, 2)], fill=LBLUE, width=1)
    d.ellipse([10, 9, 22, 23], outline=CYAN, width=2)        # charge ring '0'
    d.ellipse([13, 12, 19, 20], outline=CYAN_D, width=1)
    d.rectangle([15, 14, 17, 18], fill=CYAN_D)               # charge post
    d.point((16, 16), fill=CYAN)
    d.rectangle([4, 26, 8, 29], fill=URBAN_D, outline=OUT)   # wheel stop
    d.rectangle([23, 26, 27, 29], fill=URBAN_D, outline=OUT)
    return img


# ------------------------------------------------------------ rock/ruin

def t_rock_border(rng):
    # 'rock border', sym #, color white: a riprap band of pale
    # boulders crossing stony ground
    img, d = ground(rng, (120, 112, 100, 255),
                    [ROCK_D, EARTH_D, CAVE_L], 36)
    for (x, y, r) in ((4, 13, 3), (10, 15, 4), (17, 12, 3), (23, 15, 4),
                      (29, 13, 3), (7, 19, 3), (14, 20, 3), (21, 19, 4),
                      (27, 21, 2)):
        d.ellipse([x - r, y - r, x + r, y + r], fill=ROCK, outline=ROCK_D)
        d.ellipse([x - r, y - r, x, y], fill=WHITE)
    for (x, y) in ((6, 5), (25, 6), (12, 27), (28, 28)):
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=ROCK_D)
    return img


def ruined_tower(rng, basement=False):
    # 'ruined tower', sym T, color c_blue: broken ring of blue-gray
    # stone; the basement is the same foundation under dark earth
    if basement:
        img, d = ground(rng, (52, 46, 44, 255),
                        [(38, 33, 32, 255), CAVE_D], 36)
        wall, wall_d, hi = BLUE_STONE_D, (52, 62, 88, 255), BLUE_STONE
        inner = (30, 27, 28, 255)
    else:
        img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 28)
        wall, wall_d, hi = BLUE_STONE, BLUE_STONE_D, BLUE_STONE_L
        inner = EARTH
    cx, cy, r = 16, 16, 10
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=inner)
    # broken ring wall: blocks around the circle with a south gap
    for ang in range(0, 360, 15):
        if 70 < ang < 115:                                   # doorway gap
            continue
        a = math.radians(ang)
        x, y = cx + r * math.cos(a), cy + r * math.sin(a)
        h = 3 if (ang // 15) % 3 else 2                      # broken heights
        d.rectangle([x - 2, y - h, x + 2, y + 2], fill=wall, outline=wall_d)
        d.line([(x - 2, y - h), (x + 2, y - h)], fill=hi)
    for _ in range(7):                                       # rubble
        x, y = rng.randint(9, 23), rng.randint(9, 23)
        d.rectangle([x, y, x + 2, y + 1], fill=wall_d)
    if basement:
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3],
                  fill=(14, 12, 14, 255))                    # dark pit
    return img


# ------------------------------------------------------ rural outbuilding

def t_rural_outbuilding(rng):
    # 'outbuilding', sym #, color brown
    img, d = ground(rng, GRASS, [GRASS_D, EARTH_L], 28)
    d.rectangle([13, 21, 19, 31], fill=EARTH)                # dirt path
    building(d, (9, 7, 23, 21), WOOD, WOOD_D)
    d.rectangle([9, 7, 23, 10], fill=ROOF_BROWN, outline=OUT)  # roof cap
    d.rectangle([14, 13, 18, 21], fill=WOOD_D, outline=OUT)  # door
    d.line([(14, 17), (18, 17)], fill=WOOD_L)
    d.rectangle([20, 12, 22, 15], fill=LBLUE, outline=OUT)   # window
    return img


def t_rural_outbuilding_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    roof_plan(d, (9, 7, 23, 21), ROOF_BROWN, ROOF_BROWN_D)
    return img


# ------------------------------------------------------------ s_* shops

def t_s_antique(rng):
    # 'antique store', sym A, color brown: urn pictogram
    def urn(d):
        d.ellipse([14, 5, 18, 7], fill=CREAM)
        d.polygon([(14, 7), (18, 7), (17, 10), (15, 10)], fill=CREAM)
        d.rectangle([14, 10, 18, 11], fill=CREAM)
    return storefront(rng, WOOD, (110, 74, 44, 255), urn)


def t_s_arcade(rng):
    # 'arcade', sym A, color cyan: pixel invader pictogram
    def invader(d):
        for (x, y) in ((13, 6), (19, 6), (14, 7), (15, 7), (16, 7),
                       (17, 7), (18, 7), (13, 8), (19, 8), (15, 9),
                       (17, 9)):
            d.point((x, y), fill=WHITE)
        d.point((14, 5), fill=PINK)
        d.point((18, 5), fill=PINK)
    return storefront(rng, URBAN_D, CYAN_D, invader)


def t_s_daycare(rng):
    # 'daycare center', color blue: alphabet blocks pictogram
    def blocks(d):
        d.rectangle([12, 6, 15, 9], fill=RED, outline=OUT)
        d.rectangle([17, 6, 20, 9], fill=YELLOW, outline=OUT)
        d.point((13, 7), fill=WHITE)
        d.point((18, 7), fill=WHITE)
    return storefront(rng, CONCRETE, DBLUE, blocks)


def t_s_games(rng):
    # 'gaming store', sym g, color cyan: gamepad pictogram
    def pad(d):
        d.ellipse([12, 6, 20, 10], fill=WHITE)
        d.rectangle([13, 5, 19, 10], fill=WHITE)
        d.line([(14, 7), (14, 9)], fill=CYAN_D, width=1)
        d.line([(13, 8), (15, 8)], fill=CYAN_D, width=1)
        d.point((18, 7), fill=RED)
        d.point((19, 8), fill=YELLOW)
    return storefront(rng, URBAN_D, CYAN_D, pad)


def t_s_gardening(rng):
    # 'gardening supply', sym F, color green: potted plant pictogram
    def plant(d):
        d.polygon([(14, 8), (18, 8), (17, 10), (15, 10)],
                  fill=(196, 110, 62, 255))
        d.line([(16, 8), (16, 5)], fill=GRASS_L, width=1)
        d.ellipse([13, 4, 16, 6], fill=GRASS_L)
        d.ellipse([16, 4, 19, 6], fill=GRASS_L)
    return storefront(rng, WOOD, GRASS_D, plant)


def t_s_jewelry(rng):
    # 'jewelry store', sym J, color white: diamond pictogram
    def gem(d):
        d.polygon([(13, 6), (19, 6), (16, 10)], fill=LBLUE)
        d.polygon([(13, 6), (15, 6), (16, 10)], fill=WHITE)
        d.polygon([(17, 6), (19, 6), (16, 10)], fill=(170, 205, 245, 255))
        d.line([(13, 6), (19, 6)], fill=WHITE, width=1)
    return storefront(rng, (74, 70, 84, 255), (200, 200, 210, 255), gem)


def t_s_electronicstore_2ndfloor(rng):
    # 'electronics store 2nd floor', sym ^, color yellow: upper-floor
    # plan -- sales floor with display counters and glowing screens
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 16)
    d.rectangle([3, 3, 29, 29], fill=(188, 184, 176, 255), outline=OUT)
    d.rectangle([5, 5, 27, 27], outline=URBAN_D)
    for (x0, y0) in ((7, 8), (15, 8), (7, 18), (15, 18)):
        d.rectangle([x0, y0, x0 + 6, y0 + 4], fill=LGRAY, outline=OUT)
        d.rectangle([x0 + 1, y0 + 1, x0 + 3, y0 + 2], fill=CYAN)
        d.rectangle([x0 + 4, y0 + 1, x0 + 5, y0 + 2], fill=LBLUE)
    d.rectangle([23, 8, 27, 12], fill=YELLOW, outline=OUT)   # promo stand
    d.rectangle([23, 20, 27, 27], fill=URBAN_D, outline=OUT)  # stairwell
    d.line([(24, 26), (26, 21)], fill=LGRAY, width=1)
    return img


# ------------------------------------------------------------------ ids

DRAW = {
    "orchard_stall_roof": t_orchard_stall_roof,
    "outcrop": t_outcrop,
    "outcrop_bottom": t_outcrop_bottom,
    "portal_storm_dungeon_teleporter": t_portal_teleporter,
    "prefab": t_prefab,
    "prefab_roof": t_prefab_roof,
    "public_works_SW": t_public_works,
    "public_works_SW_roof": t_public_works_roof,
    "pump_station_1": lambda r: pump_station(r, 1),
    "pump_station_2": lambda r: pump_station(r, 2),
    "pwr_sub_s": lambda r: substation(r),
    "pwr_sub_s_roof": lambda r: substation(r, roof=True),
    "ranch_camp_65_roof": t_ranch_barn_roof,
    "ranch_camp_67": t_ranch_garage,
    "ranch_camp_67_roof": t_ranch_garage_roof,
    "ravine": t_ravine,
    "ravine_edge": t_ravine_edge,
    "ravine_floor": t_ravine_floor,
    "ravine_floor_edge": t_ravine_floor_edge,
    "refctr_fence_NE": lambda r: fence_cell(r, "NE"),
    "refctr_fence_NW": lambda r: fence_cell(r, "NW"),
    "refctr_fence_SE": lambda r: fence_cell(r, "SE"),
    "refctr_fence_SW": lambda r: fence_cell(r, "SW"),
    "refctr_fence_v": lambda r: fence_cell(r, "V"),
    "river_cave": t_river_cave,
    "river_cave_food": t_river_cave_food,
    "river_cave_kraken": t_river_cave_kraken,
    "river_cave_nest": t_river_cave_nest,
    "robofac_strip_mall_roof_5": t_robofac_mall_roof,
    "robofachq_surface_parking": t_robofachq_parking,
    "rock_border": t_rock_border,
    "ruined_tower_highlands_basement": lambda r: ruined_tower(r, True),
    "ruined_tower_highlands_ground": lambda r: ruined_tower(r),
    "rural_outbuilding": t_rural_outbuilding,
    "rural_outbuilding_roof": t_rural_outbuilding_roof,
    "s_antique": t_s_antique,
    "s_antique_roof": lambda r: shop_roof(r, (110, 74, 44, 255)),
    "s_arcade": t_s_arcade,
    "s_arcade_roof": lambda r: shop_roof(r, CYAN_D),
    "s_daycare": t_s_daycare,
    "s_daycare_roof": lambda r: shop_roof(r, DBLUE),
    "s_electronicstore_2ndfloor": t_s_electronicstore_2ndfloor,
    "s_games": t_s_games,
    "s_games_roof": lambda r: shop_roof(r, CYAN_D),
    "s_gardening": t_s_gardening,
    "s_gardening_roof": lambda r: shop_roof(r, GRASS_D),
    "s_jewelry": t_s_jewelry,
    "s_jewelry_roof": lambda r: shop_roof(r, (200, 200, 210, 255)),
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
                       "preview_om6.png")
    sheet.save(out)
    print("wrote preview_om6.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
