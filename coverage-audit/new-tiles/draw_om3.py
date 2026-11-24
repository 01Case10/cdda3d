#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch OM3: 48 overmap terrain tiles.

Overmap tiles are map cells: every sprite is a FULL-TILE OPAQUE 32x32 --
a ground/base fill in the location's data color with a centered iconic
pictogram (faction compound, farm building, FEMA camp block, fungal
form, storefront). *_roof ids are the roof plan of their base building:
same footprint and palette, roof texture instead of the ground icon.
*_hayloft is the stable's loft level: same footprint, hay treatment.
Drawn from each entry's name/symbol/color in fork-src/cdda/data/json
(overmap/overmap_terrain/*.json). Flat UltiCa overmap style.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_om3.png.
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
ROOF_RED = (158, 62, 48, 255)
ROOF_RED_D = (120, 44, 36, 255)
ROOF_BROWN = (128, 76, 48, 255)
ROOF_BROWN_D = (94, 54, 34, 255)
ROOF_GRAY = (116, 116, 120, 255)
ROOF_GRAY_D = (84, 84, 88, 255)
BARN_RED = (166, 58, 44, 255)
BARN_RED_D = (122, 40, 32, 255)
WHITE = (236, 236, 236, 255)
CREAM = (240, 232, 196, 255)
LGRAY = (178, 178, 178, 255)
BLACK = (16, 14, 16, 255)
YELLOW = (232, 202, 66, 255)
HAY = (226, 190, 92, 255)
HAY_D = (190, 152, 58, 255)
PINK = (240, 156, 186, 255)
PINK_D = (206, 106, 140, 255)
BROWN_BOOK = (140, 92, 52, 255)
LGREEN = (140, 190, 100, 255)
GREEN_SOFA = (74, 140, 70, 255)
GREEN_SOFA_D = (50, 104, 48, 255)
STONE = (198, 192, 180, 255)       # mansion interior floor
STONE_D = (164, 156, 142, 255)
STONE_WALL = (146, 138, 124, 255)
SALT = (244, 244, 240, 255)
BRINE = (206, 120, 96, 255)
CYAN_ACC = (84, 180, 196, 255)
CYAN_D = (52, 128, 142, 255)
GRAVEL = (156, 150, 138, 255)
GRAVEL_D = (128, 122, 108, 255)
GRAVEL_L = (182, 176, 162, 255)
ASPHALT = (92, 92, 98, 255)
ASPHALT_D = (70, 70, 76, 255)
FEMA_BLUE = (84, 110, 190, 255)
FEMA_BLUE_D = (58, 80, 146, 255)
LBLUE = (112, 162, 232, 255)
RED = (178, 44, 36, 255)
DARKRED = (128, 30, 26, 255)
MYC = (128, 112, 140, 255)         # fungal mycelium ground
MYC_D = (98, 84, 114, 255)
MYC_L = (160, 144, 172, 255)
FCYAN = (96, 216, 224, 255)        # fungal flowers, data color cyan
FCYAN_D = (52, 160, 170, 255)
FYELLOW = (240, 208, 84, 255)      # fungal tower, data color yellow
FAIRWAY = (78, 152, 64, 255)
FAIRWAY_D = (58, 122, 46, 255)
FAIRWAY_L = (104, 178, 88, 255)
LEAF = (66, 156, 72, 255)
LEAF_D = (44, 116, 50, 255)
YGREEN = (168, 196, 74, 255)       # home improvement, yellow_green
ROCK = (138, 134, 128, 255)
ROCK_D = (104, 100, 94, 255)
COBBLE = (120, 118, 122, 255)
COBBLE_D = (88, 86, 92, 255)
COBBLE_L = (152, 150, 154, 255)


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


def palisade(d):
    """Faction-camp compound border: wooden post fence on all edges."""
    for x in range(1, 31, 3):
        d.rectangle([x, 0, x + 1, 2], fill=WOOD_D)
        d.rectangle([x, 29, x + 1, 31], fill=WOOD_D)
    for y in range(1, 31, 3):
        d.rectangle([0, y, 2, y + 1], fill=WOOD_D)
        d.rectangle([29, y, 31, y + 1], fill=WOOD_D)


def compound(rng, speck_n=30):
    img, d = ground(rng, EARTH, [EARTH_D, EARTH_L, GRASS_D], speck_n)
    # worn grass patches inside the compound
    for _ in range(8):
        x, y = rng.randint(3, 26), rng.randint(3, 26)
        d.ellipse([x, y, x + 3, y + 2], fill=GRASS_D)
    palisade(d)
    return img, d


def mansion_room(rng, rug):
    """Interior cell of the mansion base: stone floor, mansion wall
    border, colored rug in the room's data color."""
    img, d = ground(rng, STONE, [STONE_D], 22)
    d.rectangle([0, 0, 31, 31], outline=STONE_WALL, width=2)
    d.rectangle([2, 2, 29, 29], outline=STONE_D, width=1)
    d.rectangle([8, 9, 23, 22], fill=rug)
    d.rectangle([9, 10, 22, 21], outline=STONE_D)
    return img, d


# ------------------------------------------------------- faction base

def t_mansion_kitchen(rng):
    # 'mansion base kitchen', color pink
    img, d = mansion_room(rng, PINK)
    # range block with two burners and a pot
    d.rectangle([10, 11, 21, 16], fill=LGRAY, outline=OUT)
    d.ellipse([11, 12, 14, 15], fill=OUT)
    d.ellipse([17, 12, 20, 15], fill=OUT)
    d.rectangle([13, 17, 18, 20], fill=PINK_D, outline=OUT)   # oven front
    d.line([(24, 10), (24, 15)], fill=OUT, width=1)          # utensil rail
    d.line([(24, 15), (26, 18)], fill=OUT, width=1)
    return img


def t_mansion_library(rng):
    # 'mansion base library', color brown
    img, d = mansion_room(rng, BROWN_BOOK)
    # two bookshelf blocks with colored spines
    for (x0, x1) in ((7, 13), (18, 24)):
        d.rectangle([x0, 9, x1, 22], fill=WOOD_D, outline=OUT)
        for x in range(x0 + 1, x1, 2):
            c = rng.choice([RED, FEMA_BLUE, GREEN_SOFA, YELLOW, PINK_D])
            d.line([(x, 11), (x, 20)], fill=c, width=1)
        d.line([(x0, 15), (x1, 15)], fill=WOOD_L, width=1)
    return img


def t_mansion_bedroom(rng):
    # 'mansion base bedroom', color light_green
    img, d = mansion_room(rng, LGREEN)
    # bed: frame, white pillow, light-green blanket
    d.rectangle([10, 8, 21, 23], fill=WOOD_D, outline=OUT)
    d.rectangle([11, 9, 20, 12], fill=WHITE)
    d.rectangle([11, 13, 20, 22], fill=LGREEN)
    d.line([(11, 17), (20, 17)], fill=GRASS_D, width=1)
    d.rectangle([23, 9, 25, 12], fill=WOOD, outline=OUT)     # nightstand
    return img


def t_mansion_living(rng):
    # 'mansion base living rooms', color green
    img, d = mansion_room(rng, GREEN_SOFA)
    # sofa: back, seat, arms
    d.rectangle([9, 10, 22, 14], fill=GREEN_SOFA_D, outline=OUT)
    d.rectangle([9, 14, 22, 19], fill=GREEN_SOFA, outline=OUT)
    d.rectangle([8, 12, 10, 20], fill=GREEN_SOFA_D, outline=OUT)
    d.rectangle([21, 12, 23, 20], fill=GREEN_SOFA_D, outline=OUT)
    d.rectangle([13, 21, 18, 23], fill=WOOD, outline=OUT)    # coffee table
    return img


def t_saltworks(rng):
    # 'saltworks survey', color red, copy-from faction_base_camp_0
    img, d = compound(rng)
    # evaporation pans: red brine with white salt crust gathering
    for (x0, y0) in ((6, 6), (18, 6), (6, 18), (18, 18)):
        d.rectangle([x0, y0, x0 + 8, y0 + 8], fill=BRINE, outline=OUT)
        d.rectangle([x0 + 2, y0 + 2, x0 + 6, y0 + 6], fill=SALT)
        d.point((x0 + 4, y0 + 4), fill=BRINE)
    return img


def storehouse(rng, tier2=False):
    # 'store house survey' / 'store house survey 2', color cyan
    img, d = compound(rng)
    # crate stacks, cyan survey tarp over the top one
    d.rectangle([8, 14, 15, 22], fill=WOOD, outline=OUT)
    d.line([(8, 18), (15, 18)], fill=WOOD_D)
    d.line([(11, 14), (11, 22)], fill=WOOD_D)
    d.rectangle([16, 16, 23, 22], fill=WOOD_L, outline=OUT)
    d.rectangle([11, 8, 18, 13], fill=WOOD, outline=OUT)
    d.rectangle([11, 8, 18, 10], fill=CYAN_ACC)
    if tier2:
        d.rectangle([24, 10, 27, 17], fill=WOOD_D, outline=OUT)  # barrel
        d.ellipse([24, 8, 27, 11], fill=WOOD_L, outline=OUT)
        d.rectangle([6, 24, 25, 26], fill=CYAN_D)                # survey line
    return img


def workshop(rng, tier2=False):
    # 'fabrication workshop survey' / 'fabrication workshop 2 survey'
    img, d = compound(rng)
    # workbench with vise + anvil
    d.rectangle([7, 12, 20, 16], fill=WOOD, outline=OUT)
    d.rectangle([8, 17, 10, 21], fill=WOOD_D)
    d.rectangle([17, 17, 19, 21], fill=WOOD_D)
    d.rectangle([8, 10, 11, 12], fill=LGRAY, outline=OUT)    # vise
    d.polygon([(21, 19), (26, 19), (24, 21), (24, 23), (22, 23),
               (22, 21)], fill=URBAN_D, outline=OUT)         # anvil
    d.line([(13, 9), (17, 5)], fill=LGRAY, width=2)          # wrench
    d.ellipse([16, 4, 19, 7], outline=LGRAY, width=1)
    if tier2:
        # gear beside the bench in the upgraded shop
        cx, cy = 25, 9
        for a in range(0, 360, 45):
            x = cx + int(3.4 * math.cos(math.radians(a)))
            y = cy + int(3.4 * math.sin(math.radians(a)))
            d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=CYAN_D)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=CYAN_ACC,
                  outline=OUT)
    return img


def t_hide_site(rng):
    # 'hide site', color white: a concealed camp under netting + trees
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 34)
    for (x, y) in ((5, 5), (26, 6), (6, 26), (27, 26), (16, 4)):
        tree(d, x, y, 3)
    # low camo tent, leaf-dappled, white guy-lines
    d.polygon([(9, 20), (16, 11), (23, 20)], fill=GREEN_SOFA_D,
              outline=OUT)
    d.polygon([(12, 20), (16, 14), (20, 20)], fill=GREEN_SOFA)
    for _ in range(10):
        x, y = rng.randint(10, 22), rng.randint(13, 19)
        d.point((x, y), fill=rng.choice([GRASS_L, EARTH_L]))
    d.line([(9, 20), (5, 24)], fill=WHITE, width=1)
    d.line([(23, 20), (27, 24)], fill=WHITE, width=1)
    return img


# ---------------------------------------------------------------- farms
# The agricultural set is all data color i_brown, copy-from
# generic_rural_building: grass farmyard, wooden brown buildings.
# The Isherwood ids copy-from these, so they share the renderers.

def coop(rng):
    # 'chicken coop'
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 30)
    d.rectangle([0, 25, 31, 25], fill=WOOD_D)                # fence line
    for x in range(2, 31, 5):
        d.rectangle([x, 23, x + 1, 27], fill=WOOD_D)
    building(d, (9, 7, 23, 17), WOOD, WOOD_D)                # coop box
    d.polygon([(8, 7), (16, 3), (24, 7)], fill=ROOF_RED)     # gable cap
    d.line([(20, 17), (24, 22)], fill=WOOD_L, width=2)       # ramp
    # chicken: white body, red comb
    d.ellipse([12, 20, 17, 24], fill=WHITE, outline=OUT)
    d.ellipse([16, 18, 19, 21], fill=WHITE, outline=OUT)
    d.rectangle([17, 17, 18, 18], fill=RED)
    d.point((22, 23), fill=CREAM)                            # eggs
    d.point((24, 24), fill=CREAM)
    return img


def coop_roof(rng):
    # 'chicken coop roof'
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    for (x, y) in ((4, 4), (28, 4), (4, 28), (28, 28)):
        tree(d, x, y, 3)
    roof_plan(d, (9, 8, 23, 20), ROOF_RED, ROOF_RED_D)
    return img


def carriage_house(rng):
    # 'carriage house': wide building, double doors, carriage wheel
    img, d = ground(rng, GRASS, [GRASS_D, EARTH_L], 26)
    building(d, (6, 8, 26, 22), WOOD, WOOD_D, door=False)
    d.rectangle([10, 12, 15, 22], fill=WOOD_D, outline=OUT)  # door L
    d.rectangle([17, 12, 22, 22], fill=WOOD_D, outline=OUT)  # door R
    d.line([(10, 17), (15, 17)], fill=WOOD_L)
    d.line([(17, 17), (22, 17)], fill=WOOD_L)
    d.ellipse([24, 17, 29, 22], outline=OUT, width=1)        # spare wheel
    d.point((26, 19), fill=OUT)
    return img


def carriage_house_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    roof_plan(d, (6, 8, 26, 22), ROOF_BROWN, ROOF_BROWN_D,
              vents=((16, 11),))
    return img


def horse_head(d, cx, cy):
    # simple horse-head pictogram in profile
    d.polygon([(cx - 3, cy - 6), (cx + 1, cy - 6), (cx + 4, cy - 1),
               (cx + 4, cy + 2), (cx + 1, cy + 2), (cx - 1, cy + 6),
               (cx - 3, cy + 6)], fill=WOOD, outline=OUT)
    d.polygon([(cx - 3, cy - 6), (cx - 1, cy - 8), (cx, cy - 6)],
              fill=WOOD_D)                                   # ear
    d.point((cx + 2, cy - 2), fill=OUT)                      # eye
    d.line([(cx - 3, cy - 5), (cx - 1, cy + 5)], fill=BLACK, width=1)


def stable(rng):
    # 'horse stable': long block with three stall doors + horse head
    img, d = ground(rng, GRASS, [GRASS_D, EARTH_L], 26)
    building(d, (4, 9, 28, 21), BARN_RED, BARN_RED_D, door=False)
    for x0 in (6, 13, 20):
        d.rectangle([x0, 13, x0 + 5, 21], fill=WOOD_D, outline=OUT)
        d.line([(x0, 17), (x0 + 5, 17)], fill=WOOD_L)
    horse_head(d, 16, 7)
    return img


def stable_hayloft(rng):
    # 'horse stable hayloft': the loft level -- same footprint, hay
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 16)
    d.rectangle([4, 9, 28, 21], fill=HAY, outline=OUT)
    for _ in range(40):
        x, y = rng.randint(5, 27), rng.randint(10, 20)
        d.point((x, y), fill=rng.choice([HAY_D, CREAM]))
    for (x0, y0) in ((7, 11), (18, 11), (12, 16)):
        d.rectangle([x0, y0, x0 + 6, y0 + 3], fill=HAY_D, outline=OUT)
    d.rectangle([14, 13, 18, 17], fill=BLACK)                # loft hatch
    d.rectangle([14, 13, 18, 17], outline=WOOD_D)
    return img


def stable_roof(rng):
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 18)
    roof_plan(d, (4, 9, 28, 21), ROOF_BROWN, ROOF_BROWN_D,
              vents=((10, 12), (22, 12)))
    return img


def t_farm_road(rng):
    # farm_dairy_twd_11, name 'farm road'
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 30)
    d.polygon([(12, 0), (20, 0), (21, 31), (11, 31)], fill=EARTH)
    for y in range(1, 31, 3):
        d.line([(13, y), (13, y + 1)], fill=EARTH_D)         # wheel ruts
        d.line([(19, y), (19, y + 1)], fill=EARTH_D)
    tree(d, 5, 6, 3)
    tree(d, 27, 24, 3)
    return img


def t_farm_a(rng):
    # farm_dairy_twd_7, name 'farm': barn + field rows
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 22)
    for y in range(20, 31, 3):
        d.line([(2, y), (29, y)], fill=GRASS_D)              # crop rows
    building(d, (5, 5, 15, 15), BARN_RED, BARN_RED_D)
    d.polygon([(4, 5), (10, 2), (16, 5)], fill=WHITE)        # barn gable
    d.rectangle([20, 6, 24, 14], fill=LGRAY, outline=OUT)    # silo
    d.ellipse([20, 4, 24, 7], fill=URBAN_D)
    return img


def t_farm_b(rng):
    # farm_dairy_twd_8, name 'farm': farmhouse + silo + field rows
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 22)
    for y in range(21, 31, 3):
        d.line([(2, y), (29, y)], fill=EARTH_D)
    building(d, (6, 6, 17, 16), CREAM, WOOD_D)               # farmhouse
    d.rectangle([6, 6, 17, 9], fill=ROOF_RED, outline=OUT)   # roof band
    d.line([(7, 8), (16, 8)], fill=ROOF_RED_D)
    d.rectangle([22, 5, 26, 15], fill=LGRAY, outline=OUT)    # silo
    d.ellipse([22, 3, 26, 6], fill=URBAN_D)
    tree(d, 28, 20, 3)
    return img


def t_orchard(rng):
    # farm_stills_11, name 'orchard' (data color brown)
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 24)
    d.rectangle([14, 0, 17, 31], fill=EARTH)                 # dirt lane
    for (x, y) in ((7, 6), (7, 14), (7, 22), (24, 6), (24, 14),
                   (24, 22)):
        tree(d, x, y, 4)
        d.point((x - 1, y), fill=RED)                        # fruit
        d.point((x + 2, y + 1), fill=RED)
    return img


# ----------------------------------------------------------------- FEMA
# One camp language: pale gravel ground, white/blue blocks.

def fema_ground(rng):
    return ground(rng, GRAVEL, [GRAVEL_D, GRAVEL_L], 40)


def t_fema_bus(rng):
    # 'FEMA camp bus stop', color dark_gray
    img, d = fema_ground(rng)
    d.rectangle([0, 20, 31, 31], fill=ASPHALT)               # road strip
    d.line([(0, 25), (31, 25)], fill=YELLOW)                 # curb line
    # bus: long body, window band, wheels
    d.rectangle([5, 8, 27, 18], fill=FEMA_BLUE, outline=OUT)
    d.rectangle([7, 10, 25, 13], fill=LBLUE)
    for x in range(10, 25, 4):
        d.line([(x, 10), (x, 13)], fill=FEMA_BLUE_D)
    d.ellipse([8, 16, 12, 20], fill=BLACK)
    d.ellipse([20, 16, 24, 20], fill=BLACK)
    d.rectangle([13, 4, 19, 7], fill=WHITE, outline=OUT)     # stop sign
    d.point((16, 5), fill=RED)
    return img


def t_fema_entrance(rng):
    # 'FEMA camp entrance checkpoint', color light_blue
    img, d = fema_ground(rng)
    d.rectangle([13, 0, 19, 31], fill=ASPHALT)               # entry lane
    for x in (2, 29):                                        # fence posts
        for y in range(1, 31, 5):
            d.rectangle([x, y, x + 1, y + 2], fill=URBAN_D)
    d.line([(0, 14), (12, 14)], fill=LGRAY, width=1)         # fence wire
    d.line([(20, 14), (31, 14)], fill=LGRAY, width=1)
    # barrier arm across the lane + guard booth
    d.line([(13, 16), (24, 11)], fill=WHITE, width=2)
    d.line([(13, 16), (24, 11)], fill=RED, width=1)
    d.rectangle([22, 17, 28, 24], fill=LBLUE, outline=OUT)   # booth
    d.rectangle([23, 18, 25, 20], fill=WHITE)
    return img


def t_fema_housing(rng):
    # 'FEMA camp housing', color blue: two trailer blocks
    img, d = fema_ground(rng)
    for y0 in (6, 19):
        d.rectangle([6, y0, 26, y0 + 7], fill=WHITE, outline=OUT)
        d.rectangle([6, y0, 26, y0 + 2], fill=FEMA_BLUE)
        d.rectangle([14, y0 + 3, 17, y0 + 7], fill=FEMA_BLUE_D)  # door
        d.rectangle([8, y0 + 3, 10, y0 + 5], fill=LBLUE)         # window
        d.rectangle([21, y0 + 3, 23, y0 + 5], fill=LBLUE)
    return img


def t_fema_military(rng):
    # 'FEMA camp outpost', color red: blockhouse + star + sandbags
    img, d = fema_ground(rng)
    building(d, (9, 8, 23, 22), CONCRETE, URBAN_D, door=False)
    d.rectangle([14, 12, 18, 15], fill=BLACK)                # firing slit
    cx, cy = 16, 19                                        # star emblem
    d.polygon([(cx, cy - 2), (cx + 1, cy), (cx + 2, cy),
               (cx + 1, cy + 1), (cx + 2, cy + 3), (cx, cy + 2),
               (cx - 2, cy + 3), (cx - 1, cy + 1), (cx - 2, cy),
               (cx - 1, cy)], fill=RED)
    for x in range(6, 26, 4):                                # sandbag line
        d.ellipse([x, 25, x + 3, 27], fill=EARTH_L, outline=EARTH_D)
    return img


def t_fema_road(rng):
    # 'FEMA camp road', color dark_gray
    img, d = ground(rng, ASPHALT, [ASPHALT_D, URBAN], 26)
    for y in range(1, 31, 6):
        d.rectangle([15, y, 16, y + 3], fill=YELLOW)         # centerline
    d.line([(3, 0), (3, 31)], fill=LGRAY, width=1)           # edge lines
    d.line([(28, 0), (28, 31)], fill=LGRAY, width=1)
    return img


def t_fema_storage(rng):
    # 'FEMA camp storage', color light_blue: shipping containers
    img, d = fema_ground(rng)
    d.rectangle([5, 7, 27, 14], fill=LBLUE, outline=OUT)
    for x in range(8, 27, 4):
        d.line([(x, 7), (x, 14)], fill=FEMA_BLUE_D)
    d.rectangle([8, 18, 20, 25], fill=FEMA_BLUE, outline=OUT)
    for x in range(11, 20, 3):
        d.line([(x, 18), (x, 25)], fill=FEMA_BLUE_D)
    d.rectangle([22, 19, 26, 24], fill=WOOD, outline=OUT)    # crate
    return img


def t_mutagen_lab_roof(rng):
    # 'Trans-Coast Logistics roof', color light_gray: big flat roof
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 24)
    roof_plan(d, (3, 4, 29, 28), ROOF_GRAY, ROOF_GRAY_D, ridge=False,
              vents=((9, 10), (22, 10), (9, 21), (22, 21)))
    d.rectangle([13, 14, 19, 18], fill=LGRAY, outline=OUT)   # HVAC box
    d.rectangle([5, 5, 12, 7], fill=LBLUE, outline=OUT)      # skylight
    d.rectangle([20, 24, 27, 26], fill=LBLUE, outline=OUT)
    return img


# --------------------------------------------------------------- fungal

def fungal_ground(rng):
    img, d = ground(rng, MYC, [MYC_D, MYC_L], 44)
    for _ in range(7):                                       # mycelium veins
        x, y = rng.randint(2, 28), rng.randint(2, 28)
        d.line([(x, y), (x + rng.randint(-5, 5), y + rng.randint(-5, 5))],
               fill=MYC_L)
    return img, d


def t_fungal_flowers(rng):
    # 'fungal flowers', color cyan: stalks with glowing caps
    img, d = fungal_ground(rng)
    for (x, y, r) in ((9, 10, 4), (21, 8, 3), (15, 18, 5), (24, 22, 3),
                      (7, 24, 3)):
        d.line([(x, y + r), (x, y)], fill=MYC_L, width=2)    # stalk
        d.ellipse([x - r, y - r, x + r, y + r - 1], fill=FCYAN,
                  outline=FCYAN_D)
        d.ellipse([x - r + 2, y - r + 1, x + r - 2, y], fill=WHITE)
        d.point((x, y + r + 2), fill=FCYAN_D)                # spore dots
        d.point((x + 3, y + r + 1), fill=FCYAN)
    return img


def t_fungal_tower(rng):
    # 'fungal tower', color yellow: top-down concentric tower crown
    img, d = fungal_ground(rng)
    cx, cy = 16, 16
    for r, c in ((12, MYC_D), (10, FYELLOW), (8, MYC_L), (6, FYELLOW),
                 (4, FCYAN_D)):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=c, outline=MYC_D)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=FYELLOW, outline=OUT)
    for a in range(0, 360, 60):                              # buttress roots
        x = cx + int(13 * math.cos(math.radians(a)))
        y = cy + int(13 * math.sin(math.radians(a)))
        d.line([(cx, cy), (x, y)], fill=MYC_D, width=2)
    return img


# ------------------------------------------------------------ commercial

def storefront(rng, wall, accent, sign_h, pictogram):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    d.rectangle([0, 24, 31, 31], fill=ASPHALT_D)             # parking strip
    building(d, (4, 5, 28, 24), wall, URBAN_D, door=False)
    d.rectangle([4, 5, 28, 9], fill=accent, outline=OUT)     # sign band
    d.rectangle([7, 13, 12, 20], fill=LBLUE, outline=OUT)    # windows
    d.rectangle([20, 13, 25, 20], fill=LBLUE, outline=OUT)
    d.rectangle([14, 14, 18, 24], fill=URBAN_D, outline=OUT)  # door
    pictogram(d)
    return img


def t_furniture_store(rng):
    # 'furniture store', color brown: sofa pictogram on the sign band
    def sofa(d):
        d.rectangle([13, 6, 19, 8], fill=WHITE)
        d.rectangle([12, 7, 13, 9], fill=WHITE)
        d.rectangle([19, 7, 20, 9], fill=WHITE)
    return storefront(rng, WOOD, BROWN_BOOK, 4, sofa)


def t_headshop(rng):
    # 'head shop', color i_white: leaf pictogram
    def leaf(d):
        d.polygon([(16, 5), (18, 7), (17, 9), (16, 8), (15, 9),
                   (14, 7)], fill=LEAF)
        d.line([(16, 8), (16, 10)], fill=LEAF_D, width=1)
    return storefront(rng, WHITE, LEAF, 4, leaf)


def t_home_improvement(rng):
    # 'home improvement store', color yellow_green: hammer pictogram
    def hammer(d):
        d.rectangle([13, 5, 18, 7], fill=URBAN_D)
        d.line([(16, 7), (18, 10)], fill=WOOD_D, width=2)
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 26)
    d.rectangle([0, 25, 31, 31], fill=ASPHALT_D)
    building(d, (2, 4, 30, 25), CONCRETE, URBAN_D, door=False)  # big box
    d.rectangle([2, 4, 30, 9], fill=YGREEN, outline=OUT)
    hammer(d)
    d.rectangle([6, 13, 11, 20], fill=LBLUE, outline=OUT)
    d.rectangle([21, 13, 26, 20], fill=LBLUE, outline=OUT)
    d.rectangle([14, 14, 18, 25], fill=URBAN_D, outline=OUT)
    d.rectangle([24, 21, 28, 24], fill=WOOD, outline=OUT)    # lumber stack
    d.line([(24, 22), (28, 22)], fill=WOOD_D)
    return img


def t_home_improvement_roof(rng):
    img, d = ground(rng, URBAN, [URBAN_D, URBAN_L], 20)
    roof_plan(d, (2, 4, 30, 25), ROOF_GRAY, ROOF_GRAY_D, ridge=False,
              vents=((8, 9), (16, 9), (24, 9), (8, 19), (24, 19)))
    d.rectangle([13, 12, 19, 16], fill=LGRAY, outline=OUT)   # HVAC
    d.rectangle([4, 21, 11, 23], fill=LBLUE, outline=OUT)    # skylights
    d.rectangle([21, 21, 28, 23], fill=LBLUE, outline=OUT)
    return img


# ------------------------------------------------------------ golf course

def t_golf_parking(rng):
    # golfcourse_30 'golf course parking lot', color light_gray
    img, d = ground(rng, ASPHALT, [ASPHALT_D, URBAN], 22)
    d.rectangle([0, 0, 31, 3], fill=FAIRWAY)                 # green fringe
    for x in range(3, 30, 7):                                # stall lines
        d.line([(x, 8), (x, 16)], fill=WHITE, width=1)
        d.line([(x, 20), (x, 28)], fill=WHITE, width=1)
    d.rectangle([17, 10, 23, 14], fill=RED, outline=OUT)     # parked car
    d.rectangle([18, 11, 22, 12], fill=LBLUE)
    return img


def golf_building(rng, second=False):
    # golfcourse_31 'golf course service building' on fairway green
    img, d = ground(rng, FAIRWAY, [FAIRWAY_D, FAIRWAY_L], 30)
    d.ellipse([24, 22, 30, 27], fill=(226, 214, 160, 255))   # sand bunker
    building(d, (6, 7, 22, 20), CONCRETE, URBAN_D, door=False)
    d.rectangle([8, 10, 12, 14], fill=LBLUE, outline=OUT)
    if second:
        # 'golf course bar': upper-floor window row + cocktail glass
        d.rectangle([14, 10, 20, 14], fill=LBLUE, outline=OUT)
        d.polygon([(24, 8), (28, 8), (26, 11)], outline=OUT)
        d.line([(26, 11), (26, 14)], fill=OUT)
        d.line([(24, 14), (28, 14)], fill=OUT)
    else:
        d.rectangle([13, 12, 17, 20], fill=URBAN_D, outline=OUT)
    d.line([(25, 20), (25, 10)], fill=OUT, width=1)          # flag pole
    d.polygon([(25, 10), (29, 12), (25, 14)], fill=RED)      # golf flag
    d.ellipse([24, 20, 26, 22], fill=WHITE, outline=OUT)     # green cup
    return img


def t_golf_roof(rng):
    img, d = ground(rng, FAIRWAY, [FAIRWAY_D, FAIRWAY_L], 22)
    roof_plan(d, (6, 7, 22, 20), ROOF_GRAY, ROOF_GRAY_D, vents=((16, 10),))
    return img


# --------------------------------------------------------------- highland

def t_highland_outcrop(rng):
    # highland_dead_exodii 'floral outcropping', color light_blue
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 26)
    d.ellipse([7, 9, 25, 23], fill=ROCK, outline=ROCK_D)     # rock outcrop
    d.ellipse([10, 11, 18, 17], fill=ROCK_D)
    d.ellipse([17, 14, 23, 20], fill=COBBLE_L)
    for (x, y, c) in ((10, 20, PINK), (14, 9, LBLUE), (22, 11, YELLOW),
                      (24, 19, PINK), (8, 14, WHITE), (19, 22, LBLUE)):
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=c)      # flowers
        d.point((x, y), fill=YELLOW)
    return img


def t_highland_road(rng):
    # 'cobblestone road', color dark_gray: full-tile cobbles
    img = Image.new("RGBA", (32, 32), COBBLE_D)
    d = ImageDraw.Draw(img)
    rows = ((1, 6), (8, 13), (15, 20), (22, 27))
    for ri, (y0, y1) in enumerate(rows):
        off = 4 if ri % 2 else 0
        for x in range(-4 + off, 32, 8):
            d.ellipse([x + 1, y0, x + 6, y1], fill=COBBLE,
                      outline=COBBLE_D)
            d.arc([x + 1, y0, x + 6, y1], 180, 320, fill=COBBLE_L)
    for _ in range(16):
        x, y = rng.randint(0, 31), rng.randint(0, 31)
        d.point((x, y), fill=GRASS_D)                        # moss flecks
    return img


def t_godco_enter(rng):
    # godco_enter, name 'dirt road' (entrance road to the Godco site)
    img, d = ground(rng, GRASS, [GRASS_D, GRASS_L], 28)
    d.polygon([(12, 0), (20, 0), (21, 31), (11, 31)], fill=EARTH)
    for y in range(1, 31, 3):
        d.line([(13, y), (13, y + 1)], fill=EARTH_D)
        d.line([(19, y), (19, y + 1)], fill=EARTH_D)
    d.rectangle([24, 12, 25, 20], fill=WOOD_D)               # signpost
    d.rectangle([21, 9, 28, 12], fill=WOOD, outline=OUT)     # blank sign
    return img


# ------------------------------------------------------------------ ids

DRAW = {
    "faction_base_mansion_t4": t_mansion_kitchen,
    "faction_base_mansion_t5": t_mansion_library,
    "faction_base_mansion_t6": t_mansion_bedroom,
    "faction_base_mansion_t7": t_mansion_living,
    "faction_base_saltworks_0": t_saltworks,
    "faction_base_storehouse_0": lambda r: storehouse(r),
    "faction_base_storehouse_2_0": lambda r: storehouse(r, tier2=True),
    "faction_base_workshop_0": lambda r: workshop(r),
    "faction_base_workshop_2_0": lambda r: workshop(r, tier2=True),
    "faction_hide_site_0": t_hide_site,
    "farm_1_coop": coop,
    "farm_1_coop_roof": coop_roof,
    "farm_dairy_twd_11": t_farm_road,
    "farm_dairy_twd_7": t_farm_a,
    "farm_dairy_twd_8": t_farm_b,
    "farm_horse_carriage_house_1": carriage_house,
    "farm_horse_carriage_house_1_roof": carriage_house_roof,
    "farm_horse_stable_1": stable,
    "farm_horse_stable_1_hayloft": stable_hayloft,
    "farm_horse_stable_1_roof": stable_roof,
    "farm_stills_11": t_orchard,
    "fema_bus": t_fema_bus,
    "fema_entrance": t_fema_entrance,
    "fema_housing": t_fema_housing,
    "fema_military_building": t_fema_military,
    "fema_road": t_fema_road,
    "fema_storage": t_fema_storage,
    "full_mutagen_lab_roof": t_mutagen_lab_roof,
    "fungal_flowers": t_fungal_flowers,
    "fungal_tower": t_fungal_tower,
    "furniture": t_furniture_store,
    "godco_enter": t_godco_enter,
    "golfcourse_30": t_golf_parking,
    "golfcourse_31": lambda r: golf_building(r),
    "golfcourse_31_2ndfloor": lambda r: golf_building(r, second=True),
    "golfcourse_31_roof": t_golf_roof,
    "headshop": t_headshop,
    "highland_dead_exodii": t_highland_outcrop,
    "highland_road": t_highland_road,
    "home_improvement": t_home_improvement,
    "home_improvement_roof": t_home_improvement_roof,
    "horse_farm_isherwood_10": coop,
    "horse_farm_isherwood_10_roof": coop_roof,
    "horse_farm_isherwood_4": carriage_house,
    "horse_farm_isherwood_4_roof": carriage_house_roof,
    "horse_farm_isherwood_7": stable,
    "horse_farm_isherwood_7_hayloft": stable_hayloft,
    "horse_farm_isherwood_7_roof": stable_roof,
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
                       "preview_om3.png")
    sheet.save(out)
    print("wrote preview_om3.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
