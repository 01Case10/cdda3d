#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch OM4: 48 overmap terrain tiles.

Overmap tiles are map cells: every sprite is a FULL-TILE OPAQUE 32x32 --
a ground/base fill in the location's data color with a centered iconic
pictogram. *_roof ids are the roof plan of their base building: same
footprint and palette, roof texture instead of the ground icon.
Families: the netherum labyrinth set (dark floors, stone maze walls;
flooded = water sheen, overgrown = vines, sparkling maw = gold-sparkle
chasm crack), kelp forest on open water, highway + bridge, the looted
set (intact building types with breakage and debris), LAN center pair,
megastore/mall roofs, and the mi-go camp (organic resin towers).
Drawn from each entry's name/symbol/color in fork-src/cdda/data/json
(overmap/overmap_terrain/*.json), copy-from chains resolved.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_om4.png.
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
CONCRETE_D = (128, 128, 124, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (106, 72, 40, 255)
WOOD_L = (176, 134, 88, 255)
ROOF_GRAY = (116, 116, 120, 255)
ROOF_RED = (158, 62, 48, 255)
ROOF_RED_D = (120, 44, 36, 255)
WHITE = (236, 236, 236, 255)
CREAM = (240, 232, 196, 255)
LGRAY = (178, 178, 178, 255)
BLACK = (16, 14, 16, 255)
YELLOW = (232, 202, 66, 255)
GOLD = (250, 214, 92, 255)
RED = (196, 48, 40, 255)
DARKRED = (140, 32, 28, 255)
PINK = (240, 156, 186, 255)
IPINK = (246, 178, 204, 255)
CYAN = (64, 192, 204, 255)
ICYAN = (128, 226, 236, 255)
LCYAN = (146, 222, 232, 255)
LBLUE = (112, 162, 232, 255)
ASPHALT = (74, 74, 80, 255)
ASPHALT_D = (56, 56, 62, 255)
WATER = (44, 94, 170, 255)
WATER_D = (30, 70, 138, 255)
WATER_L = (92, 146, 214, 255)
FOAM = (206, 232, 244, 255)
SAND = (232, 208, 138, 255)
SAND_D = (206, 178, 108, 255)
GLASS = (168, 224, 228, 255)
GLASS_D = (110, 178, 190, 255)
FRAME = (226, 232, 226, 255)
# labyrinth (netherum): dark floors, pale stone
LAB_FLOOR = (54, 51, 60, 255)
LAB_FLOOR_D = (40, 38, 46, 255)
LAB_FLOOR_L = (70, 66, 78, 255)
LAB_WALL = (152, 150, 158, 255)
LAB_WALL_D = (104, 102, 112, 255)
LAB_BLUE = (126, 156, 196, 255)    # labyrinth_hall light_blue structures
FLOOD = (64, 116, 178, 255)
FLOOD_L = (110, 164, 216, 255)
VINE = (74, 148, 62, 255)
VINE_D = (48, 110, 44, 255)
VINE_L = (116, 190, 92, 255)
ROOT = (168, 96, 84, 255)          # overgrown entrance: light_red roots
ROOT_D = (128, 64, 58, 255)
# fungal house: cyan
FUNG = (72, 178, 186, 255)
FUNG_D = (44, 128, 140, 255)
FUNG_L = (140, 224, 222, 255)
MARLOSS = (86, 62, 150, 255)
# kelp
KELP = (40, 122, 62, 255)
KELP_D = (26, 88, 44, 255)
KELP_L = (96, 188, 100, 255)
DEEP = (26, 74, 138, 255)
DEEP_D = (18, 56, 110, 255)
# brick lab
BRICK = (158, 88, 66, 255)
BRICK_D = (118, 62, 46, 255)
BRICK_L = (186, 116, 88, 255)
# irradiator: light_green industrial
IND = (158, 196, 130, 255)
IND_D = (110, 148, 84, 255)
IND_L = (196, 226, 164, 255)
# landfill: i_brown waste
WASTE = (128, 96, 62, 255)
WASTE_D = (96, 70, 44, 255)
WASTE_L = (160, 126, 84, 255)
# mi-go: resinous pink-gray
MIGO_G = (148, 138, 140, 255)
MIGO_G_D = (112, 102, 106, 255)
RESIN = (206, 164, 148, 255)
RESIN_D = (164, 118, 104, 255)
RESIN_L = (236, 204, 186, 255)
MIGO_GLOW = (168, 232, 120, 255)


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


def debris(d, rng, box, colors, n=14):
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        c = rng.choice(colors)
        if rng.random() < 0.4:
            d.rectangle([x, y, x + 1, y], fill=c)
        else:
            d.point((x, y), fill=c)


def sparkle(d, rng, box, n=12, color=GOLD):
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=color)
        if rng.random() < 0.5:
            d.point((x + 1, y), fill=color)
            d.point((x, y + 1), fill=color)


# ------------------------------------------------------- isherwood farm

def t_greenhouse(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 26)
    # glasshouse: pale frame grid over translucent-looking cyan panes,
    # green crop rows showing through
    x0, y0, x1, y1 = 5, 7, 27, 25
    d.rectangle([x0, y0, x1, y1], fill=GLASS, outline=OUT)
    for ry in (12, 17, 22):
        d.line([(x0 + 2, ry), (x1 - 2, ry)], fill=KELP_D, width=2)
    for gx in range(x0, x1 + 1, 4):
        d.line([(gx, y0), (gx, y1)], fill=FRAME, width=1)
    for gy in range(y0, y1 + 1, 6):
        d.line([(x0, gy), (x1, gy)], fill=FRAME, width=1)
    d.rectangle([x0, y0, x1, y1], outline=OUT)
    # fence posts of the paddock in front
    for fx in (3, 29):
        d.rectangle([fx, 27, fx, 29], fill=WOOD_D)
    return img


def t_greenhouse_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    # peaked glass roof plan: two pane slopes meeting at a ridge
    x0, y0, x1, y1 = 5, 7, 27, 25
    d.rectangle([x0, y0, x1, y1], fill=GLASS, outline=OUT)
    cy = (y0 + y1) // 2
    d.rectangle([x0, cy - 1, x1, cy + 1], fill=GLASS_D, outline=OUT)
    for gx in range(x0, x1 + 1, 4):
        d.line([(gx, y0), (gx, cy - 1)], fill=FRAME)
        d.line([(gx, cy + 1), (gx, y1)], fill=FRAME)
    for (gx, gy) in ((9, 11), (23, 11), (9, 21), (23, 21)):
        d.point((gx, gy), fill=KELP_L)
        d.point((gx + 1, gy), fill=KELP)
    d.rectangle([x0, y0, x1, y1], outline=OUT)
    return img


# ------------------------------------------------------- fungal house

def fungus_blobs(d, rng, box, n=7):
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=FUNG_L)
        d.point((x, y), fill=WHITE)
        if rng.random() < 0.5:
            d.point((x + 2, y + 1), fill=MARLOSS)


def t_house_fungus(rng):
    img, d = ground(rng, URBAN_D, [URBAN, URBAN_L], 22)
    building(d, (7, 8, 25, 24), FUNG, FUNG_D)
    # window + door choked with growth
    d.rectangle([10, 12, 13, 15], fill=FUNG_L, outline=FUNG_D)
    fungus_blobs(d, rng, (6, 6, 26, 25), 8)
    # marloss berries creeping across the yard
    for (x, y) in ((4, 27), (9, 28), (20, 27), (26, 28), (15, 29)):
        d.point((x, y), fill=MARLOSS)
        d.point((x + 1, y), fill=FUNG_L)
    return img


def t_house_fungus_roof(rng):
    img, d = ground(rng, URBAN_D, [URBAN, URBAN_L], 16)
    roof_plan(d, (7, 8, 25, 24), (120, 190, 192, 255), FUNG_D,
              vents=((11, 11),))
    fungus_blobs(d, rng, (8, 9, 24, 23), 9)
    return img


# ------------------------------------------------------- roads / water

def t_hw_road(rng):
    img, d = ground(rng, ASPHALT, [ASPHALT_D, URBAN_D], 20)
    # vertical highway: two carriageways split by a double-yellow median
    d.line([(15, 0), (15, 31)], fill=YELLOW, width=1)
    d.line([(17, 0), (17, 31)], fill=YELLOW, width=1)
    for x in (7, 24):                       # dashed lane lines
        for y in range(1, 31, 6):
            d.line([(x, y), (x, y + 3)], fill=WHITE, width=1)
    d.line([(1, 0), (1, 31)], fill=WHITE, width=1)
    d.line([(30, 0), (30, 31)], fill=WHITE, width=1)
    return img


def t_hw_road_bridge(rng):
    img, d = ground(rng, WATER, [WATER_D, WATER_L], 30)
    # horizontal bridge deck over the water
    d.rectangle([0, 10, 31, 21], fill=CONCRETE_D, outline=OUT)
    d.rectangle([0, 12, 31, 19], fill=ASPHALT)
    for x in range(1, 31, 6):               # dashed center line
        d.line([(x, 15), (x + 3, 15)], fill=WHITE)
    d.line([(0, 11), (31, 11)], fill=LGRAY, width=1)   # rails
    d.line([(0, 20), (31, 20)], fill=LGRAY, width=1)
    for x in (5, 16, 27):                   # piers below the deck
        d.rectangle([x - 1, 22, x + 1, 26], fill=CONCRETE_D)
        d.line([(x - 2, 27), (x + 2, 27)], fill=FOAM)
    return img


def t_island_sand(rng):
    img, d = ground(rng, WATER, [WATER_D, WATER_L], 30)
    # foam fringe, then the sand blob
    d.ellipse([4, 7, 28, 25], fill=FOAM)
    d.ellipse([5, 8, 27, 24], fill=SAND)
    for _ in range(22):
        x = rng.randint(8, 24)
        y = rng.randint(10, 22)
        if (x - 16) ** 2 / 90.0 + (y - 16) ** 2 / 49.0 < 1.0:
            d.point((x, y), fill=SAND_D)
    for (x, y) in ((11, 13), (20, 18), (16, 11)):   # rocks / scrub dots
        d.rectangle([x, y, x + 1, y + 1], fill=URBAN_D)
    return img


def kelp_strand(d, x, base_y, h, sway):
    pts = []
    for i in range(h + 1):
        pts.append((x + int(sway * math.sin(i / 3.0)), base_y - i))
    d.line(pts, fill=KELP_D, width=2)
    d.line(pts, fill=KELP, width=1)
    d.point(pts[-1], fill=KELP_L)


def t_kelp_core(rng):
    img, d = ground(rng, DEEP, [DEEP_D, WATER_D], 24)
    # dense stand: tall strands wall to wall
    for x in range(2, 31, 3):
        h = rng.randint(20, 28)
        kelp_strand(d, x, 31, h, rng.uniform(0.5, 2.0))
    for _ in range(8):                      # rising bubbles
        d.point((rng.randint(1, 30), rng.randint(2, 20)), fill=LCYAN)
    return img


def t_kelp_end(rng):
    img, d = ground(rng, WATER, [WATER_D, WATER_L], 26)
    # the stand thins out: short sparse strands along the bottom
    for x in (3, 9, 15, 22, 28):
        h = rng.randint(6, 12)
        kelp_strand(d, x, 31, h, rng.uniform(0.5, 1.5))
    return img


# ------------------------------------------------------- shops / city

def t_icecream_shop(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    building(d, (6, 8, 26, 24), (226, 238, 244, 255), GLASS_D)
    # pink-and-white striped awning
    for i, x in enumerate(range(7, 25, 3)):
        d.rectangle([x, 10, x + 2, 13],
                    fill=IPINK if i % 2 == 0 else WHITE)
    d.line([(7, 13), (25, 13)], fill=OUT)
    # ice cream cone pictogram
    d.polygon([(14, 19), (18, 19), (16, 24)], fill=(216, 164, 92, 255),
              outline=OUT)
    d.ellipse([13, 14, 19, 20], fill=IPINK, outline=OUT)
    d.point((16, 14), fill=RED)
    return img


def t_icecream_shop_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    roof_plan(d, (6, 8, 26, 24), (206, 224, 234, 255), GLASS_D,
              vents=((22, 20),))
    # the cone sign's base plate, seen from above
    d.rectangle([14, 13, 18, 17], fill=IPINK, outline=OUT)
    d.point((16, 15), fill=RED)
    return img


def t_lancenter(rng, variant=False):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    building(d, (5, 8, 27, 24), LGRAY, URBAN_D, door=False)
    # glass front with a row of glowing monitors
    d.rectangle([7, 10, 25, 17], fill=(52, 60, 72, 255), outline=OUT)
    for x in range(8, 24, 4):
        d.rectangle([x, 12, x + 2, 15], fill=CYAN)
        d.point((x + 1, 16), fill=LGRAY)
    if variant:
        # second shopfront: keyboard trays + a blue accent band
        d.rectangle([7, 19, 25, 21], fill=LBLUE)
        for x in range(8, 24, 4):
            d.rectangle([x, 19, x + 2, 20], fill=WHITE)
    else:
        d.rectangle([14, 21, 18, 24], fill=OUT)   # entrance
    return img


def t_lancenter_roof(rng, variant=False):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 18)
    roof_plan(d, (5, 8, 27, 24), ROOF_GRAY, URBAN_D, ridge=False,
              vents=((8, 11), (24, 11), (8, 21)))
    if variant:
        d.rectangle([12, 13, 20, 19], fill=ICYAN, outline=OUT)  # skylight
    else:
        d.rectangle([13, 12, 19, 16], fill=URBAN_D, outline=OUT)  # HVAC
        d.rectangle([14, 20, 18, 22], fill=URBAN_D, outline=OUT)
    return img


def t_landfill(rng):
    img, d = ground(rng, WASTE, [WASTE_D, WASTE_L], 34)
    # compacted trash mounds with junk glinting in them
    for (cx, cy, r) in ((10, 11, 6), (22, 9, 5), (16, 21, 7), (26, 24, 4)):
        d.ellipse([cx - r, cy - r + 2, cx + r, cy + r], fill=WASTE_D)
        d.ellipse([cx - r + 1, cy - r, cx + r - 1, cy + r - 3],
                  fill=WASTE_L)
        debris(d, rng, (cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 2),
               [LGRAY, URBAN_L, (172, 60, 52, 255), (84, 120, 168, 255)],
               n=8)
    # dirt track winding through
    d.line([(0, 28), (10, 24), (20, 27), (31, 23)], fill=EARTH_D, width=2)
    return img


def t_lab_entrance(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 22)
    # brick research block: coursed brickwork, white entrance doors
    x0, y0, x1, y1 = 4, 6, 28, 24
    d.rectangle([x0, y0, x1, y1], fill=BRICK, outline=OUT)
    for y in range(y0 + 3, y1, 3):
        d.line([(x0 + 1, y), (x1 - 1, y)], fill=BRICK_D)
    for y in range(y0 + 1, y1, 6):
        for x in range(x0 + 4, x1 - 2, 6):
            d.point((x, y), fill=BRICK_L)
    for x in (7, 12, 20, 25):               # slit windows
        d.rectangle([x, 9, x + 1, 12], fill=(52, 60, 72, 255))
    d.rectangle([13, 17, 19, 24], fill=WHITE, outline=OUT)   # doors
    d.line([(16, 17), (16, 24)], fill=URBAN_D)
    d.rectangle([11, 25, 21, 27], fill=CONCRETE_D)           # steps
    return img


def t_irradiator(rng):
    img, d = ground(rng, CONCRETE_D, [CONCRETE, URBAN_D], 24)
    building(d, (4, 9, 28, 23), IND, IND_D, door=False)
    # radiation trefoil on the plant's flank
    cx, cy = 11, 16
    for a in (90, 210, 330):
        x = cx + int(3.4 * math.cos(math.radians(a)))
        y = cy + int(3.4 * math.sin(math.radians(a)))
        d.pieslice([x - 3, y - 3, x + 3, y + 3], a - 30, a + 30,
                   fill=YELLOW, outline=OUT)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=YELLOW, outline=OUT)
    # pipe run + vent stack
    d.line([(19, 12), (26, 12)], fill=IND_D, width=2)
    d.rectangle([24, 6, 27, 12], fill=URBAN_D, outline=OUT)
    d.rectangle([19, 18, 26, 21], fill=IND_D, outline=OUT)
    return img


def t_irradiator_roof(rng):
    img, d = ground(rng, CONCRETE_D, [CONCRETE, URBAN_D], 18)
    roof_plan(d, (4, 9, 28, 23), (176, 204, 150, 255), IND_D, ridge=False)
    # faded trefoil painted on the roof + stacks
    cx, cy = 11, 16
    for a in (90, 210, 330):
        x = cx + int(3.2 * math.cos(math.radians(a)))
        y = cy + int(3.2 * math.sin(math.radians(a)))
        d.pieslice([x - 3, y - 3, x + 3, y + 3], a - 30, a + 30,
                   fill=(206, 178, 84, 255))
    d.rectangle([23, 11, 26, 15], fill=URBAN_D, outline=OUT)
    d.rectangle([18, 18, 21, 21], fill=IND_D, outline=OUT)
    return img


def t_lighthouse(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 24)
    # water bites into the right edge: the point the light guards
    d.rectangle([26, 0, 31, 31], fill=WATER)
    d.rectangle([24, 0, 25, 31], fill=SAND)
    for y in range(3, 31, 6):
        d.arc([27, y, 31, y + 3], 0, 180, fill=LCYAN)
    # small lighthouse: striped tapered tower, lamp room on top
    d.polygon([(11, 24), (14, 10), (18, 10), (21, 24)], fill=WHITE,
              outline=OUT)
    d.polygon([(12, 19), (13, 15), (19, 15), (20, 19)], fill=RED)
    d.rectangle([13, 6, 19, 10], fill=(52, 60, 72, 255), outline=OUT)
    d.rectangle([14, 7, 18, 9], fill=YELLOW)
    d.polygon([(12, 6), (20, 6), (16, 3)], fill=OUT)
    d.rectangle([15, 21, 17, 24], fill=OUT)
    return img


def t_megastore_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 14)
    # one huge flat roof: parapet, HVAC boxes, skylight strips
    d.rectangle([2, 3, 30, 29], fill=(196, 198, 202, 255), outline=OUT)
    d.rectangle([4, 5, 28, 27], outline=URBAN_D)
    for (x, y) in ((7, 8), (22, 8), (7, 22), (22, 22), (15, 15)):
        d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=LGRAY, outline=OUT)
    for y in (11, 21):
        d.rectangle([6, y, 26, y + 1], fill=ICYAN)
    return img


def t_megastore(rng):
    img, d = ground(rng, ASPHALT, [ASPHALT_D, URBAN_D], 18)
    # parking rows in front of the big box
    for x in range(3, 30, 6):
        d.line([(x, 24), (x, 29)], fill=WHITE)
    building(d, (3, 4, 29, 21), WHITE, URBAN_D, door=False)
    d.rectangle([3, 8, 29, 13], fill=LBLUE)          # light-blue fascia
    d.rectangle([5, 10, 27, 12], fill=(52, 60, 72, 255))
    for x in range(6, 27, 4):
        d.line([(x, 10), (x, 12)], fill=CYAN)
    d.rectangle([13, 16, 19, 21], fill=GLASS_D, outline=OUT)  # entry
    return img


def t_mall_roof(rng):
    img, d = ground(rng, URBAN_D, [URBAN, URBAN_D], 12)
    # loading-bay roof: red-gray slab, dock doors along the bottom edge
    d.rectangle([1, 2, 31, 30], fill=(168, 128, 118, 255), outline=OUT)
    d.rectangle([3, 4, 29, 24], outline=ROOF_RED_D)
    for y in (9, 16):
        d.rectangle([5, y, 27, y + 1], fill=ICYAN)   # skylight strips
    for (x, y) in ((6, 6), (25, 6), (16, 20)):
        d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=URBAN_D,
                    outline=OUT)
    for x in range(4, 29, 6):                        # dock doors
        d.rectangle([x, 26, x + 4, 30], fill=URBAN_D, outline=OUT)
        d.line([(x, 28), (x + 4, 28)], fill=OUT)
    return img


# ------------------------------------------------------- underground

def t_inner_cabins_depths(rng):
    # sym ' ', color black: a lightless cellar -- near-black throughout,
    # only the faintest stair down out of the dark
    img, d = ground(rng, (10, 10, 13, 255), [(16, 16, 20, 255),
                                             (6, 6, 8, 255)], 30)
    for i in range(4):
        d.rectangle([12 + i, 11 + i * 3, 20 - i, 12 + i * 3],
                    fill=(30, 30, 36, 255))
    d.polygon([(14, 26), (18, 26), (16, 29)], fill=(38, 38, 44, 255))
    return img


def t_lmoe_under(rng):
    img, d = ground(rng, (64, 58, 54, 255),
                    [(50, 44, 42, 255), (80, 72, 66, 255)], 28)
    # buried shelter hatch: steel square, red cross (+ sym, red data)
    d.rectangle([8, 8, 24, 24], fill=URBAN_D, outline=OUT)
    d.rectangle([10, 10, 22, 22], outline=BLACK)
    d.line([(16, 11), (16, 21)], fill=RED, width=3)
    d.line([(11, 16), (21, 16)], fill=RED, width=3)
    for (x, y) in ((8, 8), (24, 8), (8, 24), (24, 24)):
        d.point((x, y), fill=YELLOW)
    return img


# ------------------------------------------------------- labyrinth

def lab_floor(rng, n=30):
    return ground(rng, LAB_FLOOR, [LAB_FLOOR_D, LAB_FLOOR_L], n)


def maze_walls(d, openings=((16, 4), (4, 16), (27, 16), (16, 27))):
    """Concentric broken stone rings: a labyrinth in one cell."""
    for r in (13, 8):
        x0, y0, x1, y1 = 16 - r, 16 - r, 16 + r, 16 + r
        d.rectangle([x0, y0, x1, y1], outline=LAB_WALL, width=2)
    # carve the openings back out in floor color
    for (x, y) in openings:
        d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=LAB_FLOOR)
    d.rectangle([14, 14, 18, 18], outline=LAB_WALL_D, width=1)


def t_labyrinth2(rng):
    img, d = lab_floor(rng)
    maze_walls(d)
    return img


def t_labyrinth_roof(rng, variant=False):
    img, d = lab_floor(rng, 22)
    # "labyrinthian buildings": flat stone rooftops in a maze-like plan
    blocks = [(3, 3, 13, 12), (19, 3, 29, 12), (3, 19, 13, 29),
              (19, 19, 29, 29), (12, 13, 20, 19)]
    if variant:
        blocks = [(3, 3, 29, 10), (3, 22, 29, 29), (3, 12, 12, 20),
                  (20, 12, 29, 20), (14, 13, 18, 19)]
    for (x0, y0, x1, y1) in blocks:
        d.rectangle([x0, y0, x1, y1], fill=LAB_WALL_D, outline=OUT)
        d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=LAB_FLOOR_D)
    d.rectangle([15, 15, 17, 17], fill=LAB_WALL, outline=OUT)
    return img


def chasm(d, rng, wide=True):
    # jagged black maw across the middle, gold sparkles in the dark
    top, bot = (10, 22) if wide else (13, 19)
    pts_t = [(x, top + rng.randint(-2, 2)) for x in range(0, 32, 4)]
    pts_b = [(x, bot + rng.randint(-2, 2)) for x in range(31, -1, -4)]
    d.polygon(pts_t + pts_b, fill=BLACK)
    d.line(pts_t, fill=LAB_WALL_D, width=1)
    d.line(pts_b, fill=LAB_WALL_D, width=1)
    sparkle(d, rng, (2, top + 1, 29, bot - 1), 16 if wide else 7)


def t_chasm_core(rng):
    img, d = lab_floor(rng)
    chasm(d, rng, wide=True)
    return img


def t_chasm_end(rng):
    img, d = lab_floor(rng)
    # the maw tapers to a point at the right edge
    d.polygon([(0, 12), (20, 14), (31, 16), (20, 18), (0, 20)],
              fill=BLACK)
    d.line([(0, 12), (20, 14), (31, 16)], fill=LAB_WALL_D)
    d.line([(0, 20), (20, 18), (31, 16)], fill=LAB_WALL_D)
    sparkle(d, rng, (2, 13, 22, 18), 7)
    return img


def t_labyrinth_stairs(rng):
    img, d = lab_floor(rng)
    # a stair flight descending left ('<' sym): pale steps + rail
    for i in range(6):
        x = 6 + i * 3
        y = 8 + i * 2
        d.rectangle([x, y, x + 4, y + 1], fill=LAB_WALL)
        d.rectangle([x, y + 1, x + 4, y + 1], fill=LAB_WALL_D)
    d.line([(5, 6), (26, 20)], fill=LAB_WALL_D, width=1)
    d.polygon([(6, 22), (11, 19), (11, 25)], fill=WHITE)   # '<' chevron
    return img


def terminal_ring(d, cx, cy, ring, glow):
    d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], fill=(24, 22, 28, 255),
              outline=OUT)
    d.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], outline=ring, width=2)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=glow)
    for a in range(0, 360, 60):
        x = cx + int(6 * math.cos(math.radians(a)))
        y = cy + int(6 * math.sin(math.radians(a)))
        d.point((x, y), fill=ring)


def t_labyrinth_entrance(rng):
    img, d = lab_floor(rng)
    terminal_ring(d, 16, 16, RED, (250, 160, 140, 255))
    return img


def t_labyrinth_flooded_entrance(rng):
    img, d = ground(rng, FLOOD, [FLOOD_L, WATER_D], 26)
    # water sheen ripples around the drowned terminal
    for y in (5, 26):
        for x in range(3, 28, 7):
            d.arc([x, y, x + 4, y + 2], 0, 180, fill=LCYAN)
    terminal_ring(d, 16, 16, CYAN, ICYAN)
    return img


def t_labyrinth_flooded_hall(rng):
    img, d = ground(rng, FLOOD, [FLOOD_L, WATER_D], 26)
    # gangway planks + rails crossing the flooded hall ('_' sym)
    d.rectangle([0, 13, 31, 19], fill=WOOD, outline=OUT)
    for x in range(2, 31, 5):
        d.line([(x, 13), (x, 19)], fill=WOOD_D)
    d.line([(0, 11), (31, 11)], fill=LAB_WALL_D, width=1)
    d.line([(0, 21), (31, 21)], fill=LAB_WALL_D, width=1)
    for x in (4, 15, 26):                   # wall stubs at the edges
        d.rectangle([x, 2, x + 3, 7], fill=LAB_WALL_D, outline=OUT)
        d.rectangle([x, 25, x + 3, 30], fill=LAB_WALL_D, outline=OUT)
    return img


def t_labyrinth_hall(rng):
    img, d = lab_floor(rng)
    # "labyrinthian structures": light-blue stone blocks and pillars
    for (x0, y0, x1, y1) in ((3, 3, 12, 10), (20, 3, 29, 10),
                             (3, 22, 12, 29), (20, 22, 29, 29)):
        d.rectangle([x0, y0, x1, y1], fill=LAB_BLUE, outline=OUT)
        d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=LAB_WALL_D)
    for (x, y) in ((7, 16), (24, 16), (16, 7), (16, 25)):
        d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=LAB_BLUE,
                    outline=OUT)
    d.line([(0, 16), (31, 16)], fill=LAB_WALL_D)
    return img


def t_labyrinth_boiler(rng):
    img, d = lab_floor(rng)
    # central node: red ring around the exit boiler
    d.ellipse([5, 5, 27, 27], outline=RED, width=2)
    d.ellipse([7, 7, 25, 25], outline=DARKRED, width=1)
    # boiler tank, stack, gauge, pipe stubs
    d.rectangle([11, 12, 21, 24], fill=URBAN_L, outline=OUT)
    d.rectangle([14, 6, 18, 12], fill=URBAN_D, outline=OUT)
    d.ellipse([13, 14, 17, 18], fill=CREAM, outline=OUT)
    d.line([(15, 17), (16, 15)], fill=RED)
    d.line([(11, 20), (6, 20)], fill=URBAN_D, width=2)
    d.line([(21, 20), (26, 20)], fill=URBAN_D, width=2)
    d.rectangle([13, 22, 19, 23], fill=DARKRED)
    return img


def t_labyrinth_overgrown_entrance(rng):
    img, d = lab_floor(rng)
    # a root mass swallowing the entrance: roots converge on a dark O
    for a in range(0, 360, 30):
        x = 16 + int(13 * math.cos(math.radians(a)))
        y = 16 + int(13 * math.sin(math.radians(a)))
        d.line([(16, 16), (x, y)], fill=ROOT_D, width=3)
        d.line([(16, 16), (x, y)], fill=ROOT, width=1)
    for _ in range(10):
        x, y = rng.randint(2, 29), rng.randint(2, 29)
        d.ellipse([x - 2, y - 1, x + 2, y + 1], fill=VINE)
    d.ellipse([11, 11, 21, 21], fill=BLACK, outline=ROOT_D)
    d.ellipse([13, 13, 19, 19], outline=ROOT, width=1)
    return img


def t_labyrinth_overgrown_hall(rng):
    img, d = lab_floor(rng, 22)
    maze_walls(d)
    # vines crawl the walls: green blobs + tendrils along the rings
    for r in (13, 8):
        for _ in range(12):
            a = rng.uniform(0, 6.283)
            x = 16 + int(r * math.cos(a))
            y = 16 + int(r * math.sin(a))
            d.ellipse([x - 2, y - 2, x + 2, y + 2],
                      fill=rng.choice([VINE, VINE_D, VINE_L]))
    for _ in range(8):
        x, y = rng.randint(2, 29), rng.randint(2, 29)
        d.line([(x, y), (x + rng.randint(-3, 3), y + 3)], fill=VINE_D)
    return img


def t_labyrinth_safehouse(rng):
    img, d = lab_floor(rng, 20)
    # "home away from home": a warm little room inside the labyrinth
    d.rectangle([7, 7, 25, 25], fill=(196, 164, 110, 255), outline=OUT)
    d.rectangle([9, 9, 23, 23], fill=(226, 198, 144, 255))
    d.rectangle([10, 19, 15, 22], fill=(168, 84, 84, 255),
                outline=OUT)                                # bed
    d.rectangle([10, 19, 11, 22], fill=WHITE)
    d.rectangle([19, 10, 22, 13], fill=WOOD, outline=OUT)   # table
    d.ellipse([14, 12, 18, 16], fill=YELLOW)                # lamp glow
    d.point((16, 14), fill=WHITE)
    d.rectangle([20, 20, 22, 22], fill=(120, 140, 170, 255),
                outline=OUT)                                # supplies
    return img


# ------------------------------------------------------- shore / forest

def dock_planks(d, x0, y0, x1, y1):
    d.rectangle([x0, y0, x1, y1], fill=WOOD, outline=OUT)
    for x in range(x0 + 3, x1, 4):
        d.line([(x, y0), (x, y1)], fill=WOOD_D)
    for (px, py) in ((x0 + 1, y0 + 1), (x1 - 1, y0 + 1),
                     (x0 + 1, y1 - 1), (x1 - 1, y1 - 1)):
        d.rectangle([px - 1, py - 1, px + 1, py + 1], fill=WOOD_D)


def t_lake_dock(rng):
    img, d = ground(rng, WATER, [WATER_D, WATER_L], 28)
    dock_planks(d, 4, 12, 26, 19)           # dock reaching across the lake
    d.rectangle([22, 8, 27, 11], fill=WOOD_L, outline=OUT)  # rowboat
    d.line([(23, 9), (26, 9)], fill=WOOD_D)
    return img


def t_lake_shore_dock(rng):
    img, d = ground(rng, WATER, [WATER_D, WATER_L], 22)
    # shore on the left: sand over grass, dock reaching right into water
    d.rectangle([0, 0, 9, 31], fill=GRASS)
    d.rectangle([7, 0, 10, 31], fill=SAND)
    for _ in range(10):
        d.point((rng.randint(7, 10), rng.randint(0, 31)), fill=SAND_D)
    dock_planks(d, 8, 13, 24, 18)
    return img


def t_lumbermill_forest(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    # dense forest being worked: canopy wall to wall, stumps + logs
    spots = [(5, 5), (13, 4), (21, 5), (28, 8), (7, 13), (16, 12),
             (25, 14), (4, 22), (12, 21), (21, 23), (28, 25), (16, 28),
             (8, 29)]
    for (x, y) in spots:
        tree(d, x, y, 4)
    for (x, y) in ((10, 17), (24, 19)):      # stumps
        d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=WOOD_L, outline=WOOD_D)
    d.rectangle([26, 28, 31, 30], fill=WOOD, outline=WOOD_D)  # log pile
    d.line([(26, 29), (31, 29)], fill=WOOD_D)
    return img


# ------------------------------------------------------- looted set

def broken_window(d, x, y, w=4, h=4):
    d.rectangle([x, y, x + w, y + h], fill=BLACK, outline=OUT)
    d.line([(x, y), (x + w, y + h)], fill=LGRAY)
    d.line([(x + 1, y + h), (x + w, y + 1)], fill=URBAN_D)


def t_looted_hospital(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    building(d, (4, 5, 28, 26), WHITE, URBAN_D, door=False)
    # red cross, half scraped away
    d.rectangle([14, 8, 18, 16], fill=RED)
    d.rectangle([12, 10, 20, 14], fill=RED)
    d.rectangle([17, 12, 20, 16], fill=WHITE)
    for (x, y) in ((6, 8), (23, 8), (6, 19), (23, 19), (10, 20)):
        broken_window(d, x, y)
    debris(d, rng, (2, 26, 29, 30),
           [LGRAY, WHITE, URBAN_D, (172, 60, 52, 255)], 16)
    debris(d, rng, (5, 6, 27, 25), [URBAN_D, LGRAY], 8)
    return img


def t_looted_hospital_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 14)
    roof_plan(d, (4, 5, 28, 26), (206, 206, 208, 255), URBAN_D,
              ridge=False, vents=((7, 8), (25, 8)))
    # faded helipad H + a scorched collapse patch
    d.ellipse([12, 10, 20, 18], outline=LGRAY, width=1)
    d.line([(14, 14), (18, 14)], fill=LGRAY, width=2)
    d.line([(14, 11), (14, 17)], fill=LGRAY, width=2)
    d.line([(18, 11), (18, 17)], fill=LGRAY, width=2)
    d.rectangle([21, 18, 26, 23], fill=(70, 62, 58, 255))
    debris(d, rng, (5, 6, 27, 25), [URBAN_D, LGRAY, (120, 100, 90, 255)],
           14)
    return img


def t_looted_house(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 22)
    building(d, (8, 9, 24, 23), CREAM, WOOD_D, door=False)
    d.polygon([(6, 9), (16, 4), (26, 9)], fill=ROOF_RED_D)
    broken_window(d, 10, 12)
    broken_window(d, 19, 12)
    # door hanging ajar + ransack debris spilling out
    d.polygon([(15, 18), (19, 19), (19, 23), (15, 23)], fill=WOOD_D)
    debris(d, rng, (6, 24, 26, 30),
           [LGRAY, WOOD, (172, 60, 52, 255), WHITE], 14)
    d.rectangle([21, 25, 24, 27], fill=WOOD, outline=OUT)   # tipped crate
    return img


def t_looted_house_basement(rng):
    img, d = ground(rng, (44, 40, 42, 255),
                    [(34, 30, 32, 255), (56, 50, 48, 255)], 24)
    # foundation outline, stairs down, debris in the dark
    d.rectangle([6, 6, 26, 26], outline=URBAN_D, width=2)
    for i in range(3):
        d.rectangle([8 + i * 2, 8 + i * 2, 11 + i * 2, 9 + i * 2],
                    fill=LGRAY)
    debris(d, rng, (8, 14, 24, 24),
           [URBAN_D, LGRAY, (120, 100, 90, 255)], 16)
    d.arc([20, 6, 27, 13], 90, 180, fill=(90, 88, 90, 255))  # cobweb
    d.arc([5, 19, 12, 26], 270, 360, fill=(90, 88, 90, 255))
    return img


def t_looted_house_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 16)
    roof_plan(d, (8, 8, 24, 24), ROOF_RED, ROOF_RED_D)
    # a hole torn in the roof + scattered shingles below
    d.rectangle([17, 15, 22, 20], fill=BLACK)
    d.line([(17, 20), (21, 15)], fill=WOOD)
    d.line([(18, 15), (22, 19)], fill=WOOD)
    debris(d, rng, (6, 25, 26, 30), [ROOF_RED_D, WOOD, LGRAY], 10)
    return img


# ------------------------------------------------------- mi-go camp

def resin_spire(d, cx, base_y, h, w):
    # segmented organic tower tapering to a curl
    for i in range(h):
        t = i / max(1, h - 1)
        ww = max(1, int(w * (1 - t * 0.7)))
        y = base_y - i
        col = RESIN if i % 3 else RESIN_D
        d.line([(cx - ww, y), (cx + ww, y)], fill=col, width=1)
    d.point((cx, base_y - h), fill=RESIN_L)
    d.point((cx + 1, base_y - h + 2), fill=MIGO_GLOW)


def t_migo_camp(rng):
    img, d = ground(rng, MIGO_G, [MIGO_G_D, (172, 162, 164, 255)], 30)
    # webby strands binding the towers
    d.line([(9, 8), (23, 6)], fill=RESIN_L)
    d.line([(9, 12), (16, 20)], fill=RESIN_L)
    d.line([(23, 10), (16, 20)], fill=RESIN_L)
    resin_spire(d, 9, 26, 19, 4)
    resin_spire(d, 23, 27, 22, 5)
    resin_spire(d, 16, 29, 12, 3)
    # resin pods at the feet of the towers
    for (x, y) in ((5, 28), (27, 29), (13, 30), (20, 24)):
        d.ellipse([x - 2, y - 2, x + 2, y + 1], fill=RESIN, outline=RESIN_D)
        d.point((x, y - 1), fill=MIGO_GLOW)
    return img


DRAW = {
    "horse_farm_isherwood_9": t_greenhouse,
    "horse_farm_isherwood_9_roof": t_greenhouse_roof,
    "house_fungus_01": t_house_fungus,
    "house_fungus_roof_01": t_house_fungus_roof,
    "hw_road": t_hw_road,
    "hw_road_bridge": t_hw_road_bridge,
    "icecream_shop": t_icecream_shop,
    "icecream_shop_roof": t_icecream_shop_roof,
    "inner_cabins_depths": t_inner_cabins_depths,
    "irradiator_1_8": t_irradiator,
    "irradiator_1_8_roof": t_irradiator_roof,
    "island_sand": t_island_sand,
    "kelp_forest_core": t_kelp_core,
    "kelp_forest_end": t_kelp_end,
    "lab_surface_brick_blockC3": t_lab_entrance,
    "labyrinth2": t_labyrinth2,
    "labyrinth2_roof": lambda r: t_labyrinth_roof(r),
    "labyrinth2_roof2": lambda r: t_labyrinth_roof(r, variant=True),
    "labyrinth2_sparkling_chasm_field_core": t_chasm_core,
    "labyrinth2_sparkling_chasm_field_end": t_chasm_end,
    "labyrinth2_stairs": t_labyrinth_stairs,
    "labyrinth_entrance": t_labyrinth_entrance,
    "labyrinth_flooded_entrance": t_labyrinth_flooded_entrance,
    "labyrinth_flooded_hall": t_labyrinth_flooded_hall,
    "labyrinth_hall": t_labyrinth_hall,
    "labyrinth_nl_exit_boiler": t_labyrinth_boiler,
    "labyrinth_overgrown_entrance": t_labyrinth_overgrown_entrance,
    "labyrinth_overgrown_hall": t_labyrinth_overgrown_hall,
    "labyrinth_safehouse": t_labyrinth_safehouse,
    "lake_dock_small": t_lake_dock,
    "lake_shore_dock_small": t_lake_shore_dock,
    "lancenter": lambda r: t_lancenter(r),
    "lancenter_1": lambda r: t_lancenter(r, variant=True),
    "lancenter_roof": lambda r: t_lancenter_roof(r),
    "lancenter_roof_1": lambda r: t_lancenter_roof(r, variant=True),
    "landfill": t_landfill,
    "lighthouse_small_ground": t_lighthouse,
    "lmoe_under": t_lmoe_under,
    "looted_hospital": t_looted_hospital,
    "looted_hospital_roof": t_looted_hospital_roof,
    "looted_house": t_looted_house,
    "looted_house_basement": t_looted_house_basement,
    "looted_house_roof": t_looted_house_roof,
    "lumbermill_dforest": t_lumbermill_forest,
    "mall_upper_roof_4": t_mall_roof,
    "megastore_1_1_roof": t_megastore_roof,
    "megastore_1_2_0": t_megastore,
    "mi-go_camp2": t_migo_camp,
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
                       "preview_om4.png")
    sheet.save(out)
    print("wrote preview_om4.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
