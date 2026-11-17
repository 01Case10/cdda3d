#!/usr/bin/env python3
"""ITEM67: breads/toasts + containers + dairy -- CDDA Voxel coverage
audit.  A mixed tail batch (45 ids) spanning bread.json, containers.json
and dairy.json.  Three families:

  Breads (15) -- toast slices share a crust-and-crumb slice form and
    differ by topping: plain toasted, butter pat, red jam, peanut
    butter swirl, PB&J halves, egg-in-a-hole.  The rest are their own
    forms: puffy frybread, dense johnnycake, domed hallula roll,
    docker-holed hardtack slab (single, stacked crackers, and a
    waterlogged soggy slab with drooped corners), and three open
    crocks of sourdough paste (bubbly yellow starter, pale floury
    young paste going grey, two-tone swirled split starter).
  Containers (15) -- glass aquariums graded 30/50/100 gallon (water
    line, gravel, fish), layered black garbage bag, transparent
    gallon zipper bag, staved wooden bucket, gable-top milk carton,
    string-tied stomach pouches (sealed, large), green lunchbox,
    flier stand with fanned fliers, leather waterskins graded
    1.5/3/5 litre, and the crimped DaiZoom protein-bar wrapper.
  Dairy (15) -- milks told apart by vessel and contents: raw in a
    glass bottle with a cream line, UHT in an aseptic brick carton,
    chocolate in a brown bottle, reconstituted in a pale jug.
    Cheeses as themselves: powder heap + packet, goopy rehydrated
    sauce bowl, lumpy cottage-cheese bowls (plain, fruit, savory
    veg), breaded mozzarella sticks with a cheese pull, mascarpone
    tub, smoked mozzarella pear, pecorino wedge, white raw-butter
    stick, layered tiramisu slice with cocoa dusting.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "PBJ_Toast",
    "buttered_toast",
    "frybread",
    "hallula",
    "hardtack",
    "hardtack_cracker",
    "jam_toast",
    "johnnycake",
    "peanut_butter_toast",
    "soggy_hardtack",
    "sourdough_split_uncovered_active",
    "sourdough_starter_uncovered",
    "sourdough_young_uncovered_active",
    "toad_in_a_hole",
    "toast",
    "aquarium_large",
    "aquarium_medium",
    "aquarium_small",
    "bag_garbage_reinforced",
    "bag_zipper_gallon",
    "bucket_wood",
    "carton_milk",
    "large_stomach_sealed",
    "lunchbox",
    "stand_flyer",
    "stomach_sealed",
    "waterskin",
    "waterskin2",
    "waterskin3",
    "wrapper_pr",
    "cheese_powder",
    "cheese_rehydrated",
    "cottage_cheese",
    "cottage_cheese_with_fruit",
    "cottage_cheese_with_savory_veg",
    "fried_mozzarella_sticks",
    "mascarpone",
    "milk_UHT",
    "milk_choc",
    "milk_raw",
    "milk_reconstituted",
    "mozzarella_cheese_smoked",
    "pecorino_cheese",
    "raw_butter",
    "tiramisu",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (52, 48, 44, 255)
CRUST = (178, 122, 58, 255)
CRUST_DARK = (142, 92, 40, 255)
CRUMB = (238, 198, 128, 255)        # toasted crumb
CRUMB_PALE = (244, 224, 178, 255)
BUTTER = (250, 216, 94, 255)
BUTTER_DARK = (238, 192, 66, 255)
JAM = (172, 34, 66, 255)
JAM_LIGHT = (206, 74, 100, 255)
PB = (198, 140, 80, 255)
PB_DARK = (172, 116, 60, 255)
EGG_WHITE = (250, 248, 240, 255)
YOLK = (248, 178, 34, 255)
YOLK_DARK = (232, 152, 22, 255)
HARDTACK = (246, 240, 226, 255)
HARDTACK_EDGE = (206, 196, 172, 255)
SOGGY = (190, 174, 148, 255)
SOGGY_DARK = (160, 144, 116, 255)
GLASS = (168, 216, 228, 255)
GLASS_EDGE = (104, 164, 182, 255)
WATER = (86, 164, 208, 255)
WATER_DEEP = (58, 132, 188, 255)
LEATHER = (140, 96, 58, 255)
LEATHER_DARK = (110, 72, 40, 255)
LEATHER_LIGHT = (168, 122, 78, 255)
GUT = (156, 118, 84, 255)
GUT_DARK = (124, 90, 60, 255)
GUT_LIGHT = (182, 146, 106, 255)
WOOD = (150, 106, 62, 255)
WOOD_DARK = (112, 76, 42, 255)
WOOD_LIGHT = (178, 134, 84, 255)
CREAM = (250, 246, 236, 255)
MILK = (248, 248, 244, 255)
CHEESE = (246, 196, 84, 255)
CHEESE_POWDER = (244, 168, 62, 255)
SAUCE = (240, 156, 52, 255)
BOWL = (108, 124, 140, 255)
BOWL_DARK = (80, 94, 110, 255)
GREEN_BOX = (66, 148, 84, 255)
GREEN_BOX_DARK = (46, 112, 62, 255)
BLACK_BAG = (44, 44, 50, 255)
BLACK_BAG_LIGHT = (84, 84, 94, 255)
PAPER = (242, 238, 228, 255)
RED = (198, 52, 44, 255)
LEAF = (70, 140, 64, 255)
CARROT = (232, 122, 44, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def speckles(d, rng, box, col, n, size=1):
    x0, y0, x1, y1 = box
    for _ in range(n):
        x = rng.randint(x0, x1)
        y = rng.randint(y0, y1)
        d.point((x, y), fill=col)
        if size > 1:
            d.point((x + 1, y), fill=col)


# ------------------------------------------------------------------ bread

def bread_slice(d, crumb=CRUMB, crust=CRUST):
    """Toast-slice silhouette: arched top, flat bottom, crust rim."""
    d.pieslice([7, 4, 25, 20], start=180, end=360, fill=crust)
    d.rectangle([7, 12, 25, 27], fill=crust)
    d.pieslice([9, 6, 23, 20], start=180, end=360, fill=crumb)
    d.rectangle([9, 12, 23, 25], fill=crumb)
    d.line([9, 25, 23, 25], fill=crust, width=1)


def toast_marks(d, rng):
    for _ in range(7):
        x = rng.randint(11, 21)
        y = rng.randint(9, 22)
        d.point((x, y), fill=shade(CRUMB, 0.88))
        d.point((x + 1, y), fill=shade(CRUMB, 0.92))


def draw_toast(d, rng):
    bread_slice(d)
    toast_marks(d, rng)


def draw_buttered_toast(d, rng):
    bread_slice(d)
    toast_marks(d, rng)
    # melting butter pat, slightly rotated square with drip runs
    d.rectangle([12, 11, 20, 17], fill=BUTTER)
    d.rectangle([12, 11, 20, 17], outline=BUTTER_DARK, width=1)
    d.point((13, 12), fill=(252, 232, 140, 255))
    d.line([14, 18, 14, 20], fill=BUTTER, width=1)
    d.line([18, 18, 18, 21], fill=BUTTER, width=1)
    d.point((16, 22), fill=BUTTER)


def spread(d, rng, col, col_dark, col_light):
    """Wavy spread layer over the crumb with gloss highlights."""
    d.pieslice([10, 7, 22, 19], start=180, end=360, fill=col)
    d.rectangle([10, 12, 22, 24], fill=col)
    for _ in range(6):  # swirl ridges
        x = rng.randint(11, 20)
        y = rng.randint(10, 22)
        d.line([x, y, x + 2, y], fill=col_dark, width=1)
    for _ in range(4):  # gloss
        x = rng.randint(11, 20)
        y = rng.randint(9, 20)
        d.point((x, y), fill=col_light)


def draw_jam_toast(d, rng):
    bread_slice(d)
    spread(d, rng, JAM, shade(JAM, 0.8), JAM_LIGHT)


def draw_pb_toast(d, rng):
    bread_slice(d)
    spread(d, rng, PB, PB_DARK, shade(PB, 1.12))


def draw_pbj_toast(d, rng):
    bread_slice(d)
    # peanut butter on the left half, jelly on the right half
    d.pieslice([10, 7, 22, 19], start=180, end=360, fill=PB)
    d.rectangle([10, 12, 22, 24], fill=PB)
    d.pieslice([16, 7, 22, 19], start=270, end=360, fill=JAM)
    d.rectangle([16, 12, 22, 24], fill=JAM)
    d.line([16, 8, 16, 24], fill=PB_DARK, width=1)
    for _ in range(3):
        x = rng.randint(17, 21)
        y = rng.randint(10, 20)
        d.point((x, y), fill=JAM_LIGHT)
    d.point((12, 12), fill=BUTTER)  # butter sheen dab under the spreads
    d.point((13, 20), fill=shade(PB, 1.12))


def draw_toad_in_a_hole(d, rng):
    bread_slice(d)
    # fried egg set into a hole in the middle of the slice
    d.ellipse([11, 10, 21, 21], fill=EGG_WHITE)
    d.ellipse([11, 10, 21, 21], outline=CRUST_DARK, width=1)
    d.ellipse([14, 13, 18, 18], fill=YOLK)
    d.point((15, 14), fill=(252, 206, 92, 255))
    d.point((17, 17), fill=YOLK_DARK)
    toast_marks(d, rng)


def draw_frybread(d, rng):
    # puffy irregular fried disc
    d.ellipse([5, 9, 27, 25], fill=CRUST)
    d.ellipse([6, 10, 26, 24], fill=(232, 176, 92, 255))
    d.ellipse([10, 12, 20, 20], fill=(240, 192, 108, 255))
    for _ in range(9):  # fried blister spots
        x = rng.randint(8, 24)
        y = rng.randint(11, 22)
        d.point((x, y), fill=CRUST_DARK if rng.random() < 0.5 else CRUST)
    d.point((8, 21), fill=CRUST)  # ragged edge bumps
    d.point((24, 12), fill=CRUST)


def draw_johnnycake(d, rng):
    # small, thick, dense corn cake: darker rim, coarse crumb
    d.ellipse([8, 10, 24, 26], fill=CRUST_DARK)
    d.ellipse([9, 10, 23, 24], fill=(228, 172, 76, 255))
    d.ellipse([9, 8, 23, 22], fill=(236, 186, 92, 255))
    d.ellipse([11, 10, 21, 19], fill=(242, 198, 110, 255))
    for _ in range(10):
        x = rng.randint(11, 21)
        y = rng.randint(11, 21)
        d.point((x, y), fill=CRUST if rng.random() < 0.6 else CRUST_DARK)


def draw_hallula(d, rng):
    # large round scone-like roll, domed, with a pale split on top
    d.ellipse([5, 11, 27, 27], fill=shade(CRUST, 0.92))
    d.ellipse([5, 8, 27, 24], fill=(186, 130, 66, 255))
    d.ellipse([8, 10, 24, 21], fill=(200, 146, 78, 255))
    d.line([11, 13, 21, 13], fill=CRUMB_PALE, width=2)  # split
    d.line([12, 12, 20, 12], fill=CRUMB_PALE, width=1)
    for _ in range(6):
        x = rng.randint(8, 24)
        y = rng.randint(16, 23)
        d.point((x, y), fill=CRUST_DARK)


def hardtack_slab(d, x0, y0, x1, y1, base, edge):
    d.rectangle([x0, y0, x1, y1], fill=base)
    d.rectangle([x0, y0, x1, y1], outline=edge, width=1)
    for hx in range(x0 + 3, x1 - 1, 4):  # docker holes
        for hy in range(y0 + 3, y1 - 1, 4):
            d.point((hx, hy), fill=edge)
            d.point((hx + 1, hy), fill=shade(base, 0.94))


def draw_hardtack(d, rng):
    hardtack_slab(d, 8, 7, 24, 25, HARDTACK, HARDTACK_EDGE)
    d.line([9, 24, 23, 24], fill=HARDTACK_EDGE, width=1)


def draw_hardtack_cracker(d, rng):
    # a stack of crackers, offset so the pile reads as many pieces
    hardtack_slab(d, 7, 6, 22, 21, shade(HARDTACK, 0.96), HARDTACK_EDGE)
    hardtack_slab(d, 9, 10, 25, 25, HARDTACK, HARDTACK_EDGE)
    hardtack_slab(d, 8, 15, 23, 28, shade(HARDTACK, 0.99), HARDTACK_EDGE)


def draw_soggy_hardtack(d, rng):
    # waterlogged slab: darker, swollen, corners drooping, wet sheen
    d.polygon([(8, 8), (24, 8), (25, 22), (22, 26), (10, 26), (7, 22)],
              fill=SOGGY)
    d.polygon([(8, 8), (24, 8), (25, 22), (22, 26), (10, 26), (7, 22)],
              outline=SOGGY_DARK)
    for hx in range(11, 23, 4):
        for hy in range(11, 23, 4):
            d.point((hx, hy), fill=SOGGY_DARK)
    for _ in range(6):  # dark water blotches
        x = rng.randint(9, 23)
        y = rng.randint(9, 24)
        d.line([x, y, x + 2, y], fill=shade(SOGGY, 0.88), width=1)
    d.line([12, 10, 17, 10], fill=(226, 220, 204, 255), width=1)  # sheen
    d.line([16, 27, 16, 29], fill=WATER_DEEP, width=1)  # drip


def sourdough_crock(d, rng, paste, paste_dark, bubbles, spots=None):
    """Open crock seen from front-above: rim ellipse, paste surface."""
    d.rectangle([8, 13, 24, 27], fill=(168, 148, 120, 255))  # crock body
    d.rectangle([8, 13, 24, 27], outline=(120, 100, 74, 255), width=1)
    d.line([8, 20, 24, 20], fill=(140, 122, 94, 255), width=1)
    d.ellipse([7, 8, 25, 16], fill=(188, 168, 138, 255))
    d.ellipse([7, 8, 25, 16], outline=(120, 100, 74, 255), width=1)
    d.ellipse([9, 9, 23, 15], fill=paste)
    for _ in range(bubbles):  # froth bubbles on the surface
        x = rng.randint(10, 22)
        y = rng.randint(10, 14)
        d.point((x, y), fill=paste_dark)
        if rng.random() < 0.4:
            d.point((x, y - 1), fill=paste_dark)
    if spots:
        for sx, sy, col in spots:
            d.point((sx, sy), fill=col)
            d.point((sx + 1, sy), fill=col)
    # flour dust on the rim and table edge
    for fx in (8, 12, 21, 24):
        d.point((fx, 8), fill=(248, 246, 240, 255))


def draw_sourdough_starter(d, rng):
    # precious live starter: yellowish, well frothed
    sourdough_crock(d, rng, (242, 226, 168, 255), (228, 204, 128, 255), 12)


def draw_sourdough_young(d, rng):
    # young paste going bad: pale floury, sparse bubbles, grey-green spots
    sourdough_crock(d, rng, (238, 232, 214, 255), (214, 208, 188, 255), 4,
                    spots=[(13, 11, (158, 164, 138, 255)),
                           (19, 13, (158, 164, 138, 255))])


def draw_sourdough_split(d, rng):
    # starter freshly mixed into flour paste: two-tone swirl, active
    sourdough_crock(d, rng, (240, 230, 196, 255), (226, 208, 150, 255), 8)
    d.arc([10, 9, 22, 15], start=20, end=200, fill=(248, 244, 232, 255),
          width=2)
    d.arc([12, 10, 20, 14], start=200, end=360, fill=(236, 214, 140, 255),
          width=1)


# ------------------------------------------------------------- containers

def aquarium(d, rng, w, fish_n, plants):
    x0, x1 = 16 - w // 2, 16 + w // 2
    y0, y1 = 7, 26
    d.rectangle([x0, y0, x1, y1], fill=(196, 228, 236, 255))
    d.rectangle([x0, y0 + 4, x1, y1 - 3], fill=WATER)  # water body
    d.line([x0, y0 + 4, x1, y0 + 4], fill=(220, 244, 250, 255), width=1)
    for gx in range(x0 + 1, x1, 3):  # gravel bed
        d.point((gx, y1 - 2), fill=(196, 170, 120, 255))
        d.point((gx + 1, y1 - 1), fill=(172, 146, 100, 255))
    d.rectangle([x0, y1 - 3, x1, y1 - 3], fill=(186, 160, 112, 255))
    for i in range(plants):  # plant strands
        px = x0 + 3 + i * 6
        d.line([px, y1 - 4, px, y0 + 9], fill=LEAF, width=1)
        d.line([px, y0 + 12, px + 1, y0 + 10], fill=LEAF, width=1)
    for i in range(fish_n):  # little fish silhouettes
        fx = x0 + 4 + i * 7
        fy = y0 + 8 + (i % 2) * 5
        d.polygon([(fx, fy), (fx + 3, fy - 1), (fx + 3, fy + 1)],
                  fill=(240, 140, 50, 255))
        d.point((fx + 4, fy), fill=(240, 140, 50, 255))
        d.point((fx + 5, fy), fill=(240, 140, 50, 255))
    d.rectangle([x0, y0, x1, y1], outline=GLASS_EDGE, width=1)
    d.line([x0, y0, x0, y1], fill=GLASS_EDGE, width=1)
    d.line([x0 + 2, y0 + 2, x0 + 2, y0 + 8], fill=(236, 250, 252, 255),
           width=1)  # glass glare


def draw_aquarium_small(d, rng):
    aquarium(d, rng, 18, 1, 1)


def draw_aquarium_medium(d, rng):
    aquarium(d, rng, 22, 2, 2)


def draw_aquarium_large(d, rng):
    aquarium(d, rng, 26, 3, 3)


def draw_garbage_bag(d, rng):
    # ten layered bags: fat black sack, tied neck, heavy fold lines
    d.polygon([(13, 6), (19, 6), (21, 10), (24, 16), (24, 25), (22, 28),
               (10, 28), (8, 25), (8, 16), (11, 10)], fill=BLACK_BAG)
    d.rectangle([13, 4, 19, 7], fill=BLACK_BAG)  # knot
    d.line([14, 4, 18, 4], fill=BLACK_BAG_LIGHT, width=1)
    d.line([11, 10, 21, 10], fill=BLACK_BAG_LIGHT, width=1)  # layered rims
    d.line([11, 12, 21, 12], fill=(60, 60, 68, 255), width=1)
    for fx, fy0, fy1 in ((12, 14, 24), (16, 13, 26), (20, 14, 23)):
        d.line([fx, fy0, fx - 1, fy1], fill=BLACK_BAG_LIGHT, width=1)
    d.point((10, 18), fill=(110, 110, 122, 255))


def draw_zipper_bag(d, rng):
    # transparent gallon bag: pale film, blue/red zip strips, contents
    d.rectangle([9, 6, 23, 27], fill=(226, 238, 242, 255))
    d.rectangle([9, 6, 23, 27], outline=(150, 178, 190, 255), width=1)
    d.line([9, 8, 23, 8], fill=(66, 110, 200, 255), width=1)   # blue zip
    d.line([9, 10, 23, 10], fill=RED, width=1)                  # red zip
    # vague contents seen through the film
    d.ellipse([12, 17, 19, 24], fill=(206, 200, 178, 255))
    d.rectangle([16, 14, 21, 20], fill=(198, 206, 190, 255))
    for cx, cy in ((11, 13), (20, 24), (13, 25)):  # crinkle highlights
        d.line([cx, cy, cx + 2, cy - 1], fill=(248, 252, 252, 255), width=1)


def draw_bucket_wood(d, rng):
    # staved wooden bucket with handle and water inside
    d.arc([8, 2, 24, 12], start=180, end=360, fill=WOOD_DARK, width=2)
    d.polygon([(8, 9), (24, 9), (22, 27), (10, 27)], fill=WOOD)
    for sx in (12, 16, 20):  # stave seams
        d.line([sx, 10, sx - 1, 26], fill=WOOD_DARK, width=1)
    d.line([9, 15, 23, 15], fill=(70, 70, 76, 255), width=2)   # iron bands
    d.line([10, 23, 22, 23], fill=(70, 70, 76, 255), width=2)
    d.ellipse([9, 7, 23, 12], fill=WATER)  # water surface
    d.ellipse([9, 7, 23, 12], outline=WOOD_DARK, width=1)
    d.line([11, 9, 16, 9], fill=(200, 232, 244, 255), width=1)
    d.line([10, 18, 22, 18], fill=WOOD_LIGHT, width=1)


def draw_carton_milk(d, rng):
    # half-gallon gable-top paper carton with a threaded cap
    d.polygon([(10, 12), (10, 7), (16, 4), (22, 7), (22, 12)], fill=PAPER)
    d.polygon([(10, 12), (10, 7), (16, 4), (22, 7), (22, 12)],
              outline=(170, 168, 158, 255))
    d.line([16, 4, 16, 12], fill=(190, 188, 178, 255), width=1)
    d.rectangle([19, 7, 22, 9], fill=(66, 110, 200, 255))  # screw cap
    d.rectangle([10, 12, 22, 27], fill=PAPER)
    d.rectangle([10, 12, 22, 27], outline=(170, 168, 158, 255), width=1)
    d.rectangle([10, 17, 22, 22], fill=(66, 110, 200, 255))  # label band
    d.ellipse([14, 18, 18, 21], fill=PAPER)  # cow-spot motif on the band
    d.point((15, 19), fill=INK)
    d.line([12, 24, 20, 24], fill=(150, 148, 140, 255), width=1)
    d.line([12, 26, 18, 26], fill=(150, 148, 140, 255), width=1)


def stomach(d, rng, scale_w, scale_h):
    cx = 16
    x0, x1 = cx - scale_w // 2, cx + scale_w // 2
    y0 = 12 - (scale_h - 16) // 2
    y1 = y0 + scale_h
    d.ellipse([x0, y0 + 3, x1, y1], fill=GUT)
    d.ellipse([x0 + 2, y0 + 5, x1 - 4, y1 - 4], fill=GUT_LIGHT)
    # gathered neck + string tie
    d.polygon([(cx - 3, y0 + 4), (cx + 3, y0 + 4), (cx + 1, y0 - 2),
               (cx - 1, y0 - 2)], fill=GUT_DARK)
    for ty in range(y0 - 1, y0 + 4):
        d.line([cx - 3, ty, cx + 3, ty], fill=(216, 198, 160, 255), width=1)
    d.line([cx + 3, y0, cx + 6, y0 - 3], fill=(216, 198, 160, 255), width=1)
    d.line([cx + 3, y0 + 2, cx + 6, y0 + 2], fill=(216, 198, 160, 255),
           width=1)
    for _ in range(6):  # gut-skin mottling
        x = rng.randint(x0 + 2, x1 - 2)
        y = rng.randint(y0 + 6, y1 - 2)
        d.point((x, y), fill=GUT_DARK)


def draw_stomach_sealed(d, rng):
    stomach(d, rng, 16, 17)


def draw_large_stomach_sealed(d, rng):
    stomach(d, rng, 22, 21)


def waterskin(d, rng, w, h):
    cx = 16
    x0, x1 = cx - w // 2, cx + w // 2
    y1 = 28
    y0 = y1 - h
    d.ellipse([x0, y0 + 4, x1, y1], fill=LEATHER)
    d.ellipse([x0 + 2, y0 + 6, x1 - 5, y1 - 5], fill=LEATHER_LIGHT)
    d.rectangle([cx - 2, y0, cx + 2, y0 + 5], fill=LEATHER_DARK)  # neck
    d.rectangle([cx - 3, y0 - 2, cx + 3, y0], fill=WOOD_DARK)     # stopper
    d.line([cx - 2, y0 + 3, cx + 2, y0 + 3], fill=(216, 198, 160, 255),
           width=1)  # cord wrap
    # carrying strap arcing off the neck
    d.arc([cx + 1, y0 - 4, cx + 11, y0 + 8], start=270, end=90,
          fill=LEATHER_DARK, width=1)
    d.line([x0 + 3, y1 - 6, x1 - 3, y1 - 6], fill=LEATHER_DARK, width=1)


def draw_waterskin(d, rng):
    waterskin(d, rng, 14, 16)


def draw_waterskin2(d, rng):
    waterskin(d, rng, 18, 20)


def draw_waterskin3(d, rng):
    waterskin(d, rng, 22, 24)


def draw_lunchbox(d, rng):
    d.rectangle([7, 12, 25, 27], fill=GREEN_BOX)
    d.rectangle([7, 12, 25, 27], outline=GREEN_BOX_DARK, width=1)
    d.rectangle([7, 12, 25, 17], fill=shade(GREEN_BOX, 1.1))  # lid
    d.line([7, 17, 25, 17], fill=GREEN_BOX_DARK, width=1)
    d.arc([12, 7, 20, 13], start=180, end=360, fill=GREEN_BOX_DARK, width=2)
    for lx in (10, 20):  # latches
        d.rectangle([lx, 16, lx + 2, 19], fill=(196, 198, 204, 255))
    d.line([9, 21, 15, 21], fill=shade(GREEN_BOX, 1.14), width=1)


def draw_stand_flyer(d, rng):
    # white plastic display stand, fliers fanned in the front pocket
    d.polygon([(9, 27), (23, 27), (21, 20), (11, 20)], fill=(228, 228, 232, 255))
    d.polygon([(11, 20), (21, 20), (20, 6), (12, 6)], fill=(238, 238, 242, 255))
    d.polygon([(11, 20), (21, 20), (20, 6), (12, 6)],
              outline=(168, 168, 176, 255))
    # three fliers with coloured header bars
    for i, (fx, col) in enumerate(((13, RED), (16, (66, 110, 200, 255)),
                                   (18, (240, 168, 40, 255)))):
        fy = 8 + (i % 2)
        d.rectangle([fx, fy, fx + 4, fy + 9], fill=PAPER)
        d.rectangle([fx, fy, fx + 4, fy + 2], fill=col)
        d.line([fx + 1, fy + 5, fx + 3, fy + 5], fill=(150, 148, 140, 255),
               width=1)
        d.line([fx + 1, fy + 7, fx + 3, fy + 7], fill=(150, 148, 140, 255),
               width=1)
    d.rectangle([10, 20, 22, 23], fill=(214, 214, 220, 255))  # pocket lip
    d.rectangle([10, 20, 22, 23], outline=(168, 168, 176, 255), width=1)


def draw_wrapper_pr(d, rng):
    # crimped greaseproof protein-bar wrapper, brand band + glyph text
    d.polygon([(5, 12), (27, 12), (27, 21), (5, 21)], fill=(196, 204, 188, 255))
    for cx_ in (5, 27):  # zigzag crimped ends
        for zy in range(12, 22, 2):
            d.point((cx_ + (1 if cx_ == 5 else -1), zy), fill=(150, 158, 142, 255))
    d.rectangle([5, 12, 27, 21], outline=(130, 138, 122, 255), width=1)
    d.rectangle([9, 14, 23, 17], fill=(84, 140, 74, 255))  # brand band
    for gx in range(10, 22, 2):  # "DaiZoom" as glyph bars
        h = 1 + (gx * 7) % 2
        d.line([gx, 16, gx, 16 - h], fill=(240, 244, 236, 255), width=1)
    d.ellipse([12, 18, 15, 20], fill=(170, 150, 96, 255))  # soy bean motif
    d.line([20, 19, 24, 19], fill=(130, 138, 122, 255), width=1)
    for _ in range(5):  # grease spots / crinkles
        x = rng.randint(6, 26)
        y = rng.randint(13, 20)
        d.point((x, y), fill=(214, 220, 206, 255))


# ------------------------------------------------------------------ dairy

def bowl(d, x0=8, x1=24, y0=13, y1=26):
    d.polygon([(x0, y0), (x1, y0), (x1 - 3, y1), (x0 + 3, y1)], fill=BOWL)
    d.polygon([(x0, y0), (x1, y0), (x1 - 3, y1), (x0 + 3, y1)],
              outline=BOWL_DARK)
    d.ellipse([x0, y0 - 2, x1, y0 + 3], fill=BOWL_DARK)
    d.line([x0 + 4, y1 + 1, x1 - 4, y1 + 1], fill=BOWL_DARK, width=1)


def curds(d, rng, extras=()):
    """Lumpy cottage-cheese surface inside the bowl rim."""
    d.ellipse([9, 11, 23, 16], fill=(238, 234, 214, 255))  # whey
    for _ in range(14):
        x = rng.randint(10, 21)
        y = rng.randint(11, 14)
        d.ellipse([x, y, x + 2, y + 2], fill=CREAM)
    for kind, col in extras:
        for _ in range(5):
            x = rng.randint(10, 21)
            y = rng.randint(11, 14)
            d.point((x, y), fill=col)
            if kind == "chunk":
                d.point((x + 1, y), fill=col)


def draw_cottage_cheese(d, rng):
    bowl(d)
    curds(d, rng)


def draw_cottage_cheese_fruit(d, rng):
    bowl(d)
    curds(d, rng, extras=[("chunk", RED), ("chunk", (244, 166, 84, 255))])


def draw_cottage_cheese_veg(d, rng):
    bowl(d)
    curds(d, rng, extras=[("chunk", CARROT), ("speck", LEAF)])


def draw_cheese_powder(d, rng):
    # kraft packet behind a heap of orange powder
    d.rectangle([11, 8, 21, 20], fill=(188, 156, 104, 255))
    d.rectangle([11, 8, 21, 20], outline=(150, 120, 76, 255), width=1)
    d.line([11, 11, 21, 11], fill=(150, 120, 76, 255), width=1)
    d.polygon([(6, 26), (12, 18), (20, 18), (26, 26)], fill=CHEESE_POWDER)
    for _ in range(22):
        x = rng.randint(8, 24)
        y = rng.randint(19, 25)
        if y > 18 + abs(x - 16) * 0.6:
            d.point((x, y), fill=shade(CHEESE_POWDER, 0.9))
    d.point((14, 20), fill=(250, 196, 96, 255))
    d.point((18, 22), fill=(250, 196, 96, 255))
    for sx in (8, 24):  # spilled dust
        d.point((sx, 27), fill=CHEESE_POWDER)


def draw_cheese_rehydrated(d, rng):
    bowl(d)
    d.ellipse([9, 11, 23, 16], fill=SAUCE)
    d.arc([11, 11, 21, 16], start=200, end=340, fill=shade(SAUCE, 1.14),
          width=2)  # swirl
    d.line([22, 14, 22, 19], fill=SAUCE, width=2)  # goopy drip over the rim
    d.point((22, 20), fill=SAUCE)
    d.point((13, 13), fill=(250, 186, 92, 255))


def draw_mozzarella_sticks(d, rng):
    # three breaded sticks, the front one broken with a cheese pull
    for i, (x0, y0) in enumerate(((9, 9), (13, 13), (10, 18))):
        d.line([x0, y0 + 6, x0 + 11, y0], fill=(196, 138, 62, 255), width=4)
        d.line([x0, y0 + 6, x0 + 11, y0], fill=(222, 168, 88, 255), width=2)
    for _ in range(10):  # breadcrumb speckle
        x = rng.randint(9, 24)
        y = rng.randint(9, 24)
        d.point((x, y), fill=CRUST_DARK)
    # broken tip + stretchy cheese strand
    d.line([21, 18, 25, 15], fill=(222, 168, 88, 255), width=3)
    d.line([22, 19, 24, 17], fill=CREAM, width=1)
    d.line([23, 20, 25, 18], fill=CREAM, width=1)


def draw_mascarpone(d, rng):
    # round tub of smooth spreadable cream
    d.rectangle([9, 13, 23, 26], fill=(236, 236, 240, 255))
    d.rectangle([9, 13, 23, 26], outline=(176, 176, 184, 255), width=1)
    d.ellipse([8, 9, 24, 15], fill=CREAM)
    d.ellipse([8, 9, 24, 15], outline=(206, 202, 190, 255), width=1)
    d.arc([11, 10, 21, 14], start=180, end=360, fill=(232, 226, 208, 255),
          width=2)  # cream swirl
    d.point((16, 11), fill=(240, 234, 218, 255))
    d.rectangle([9, 19, 23, 22], fill=(150, 178, 150, 255))  # label band


def draw_milk_raw(d, rng):
    # glass bottle, unhomogenized: cream layer floating on top
    d.rectangle([13, 8, 19, 12], fill=GLASS)
    d.rectangle([11, 12, 21, 27], fill=GLASS)
    d.rectangle([11, 12, 21, 27], outline=GLASS_EDGE, width=1)
    d.rectangle([12, 13, 20, 17], fill=(250, 240, 208, 255))  # cream top
    d.rectangle([12, 18, 20, 26], fill=MILK)
    d.line([12, 17, 20, 17], fill=(238, 226, 186, 255), width=1)
    d.rectangle([13, 5, 19, 8], fill=RED)  # foil cap
    d.line([13, 14, 13, 25], fill=(236, 250, 252, 255), width=1)


def draw_milk_choc(d, rng):
    # brown bottle of chocolate milk
    d.rectangle([13, 8, 19, 12], fill=(122, 80, 48, 255))
    d.rectangle([11, 12, 21, 27], fill=(134, 88, 52, 255))
    d.rectangle([11, 12, 21, 27], outline=(96, 60, 32, 255), width=1)
    d.rectangle([12, 16, 20, 23], fill=(160, 108, 66, 255))  # choc body
    d.rectangle([13, 5, 19, 8], fill=(240, 200, 90, 255))  # gold cap
    d.rectangle([11, 13, 21, 15], fill=PAPER)  # label
    d.ellipse([14, 13, 17, 15], fill=(134, 88, 52, 255))
    d.line([13, 18, 13, 25], fill=(178, 128, 82, 255), width=1)


def draw_milk_uht(d, rng):
    # aseptic brick carton: tall box, folded top seam, UHT band
    d.rectangle([11, 7, 21, 27], fill=PAPER)
    d.rectangle([11, 7, 21, 27], outline=(170, 168, 158, 255), width=1)
    d.line([11, 9, 21, 9], fill=(190, 188, 178, 255), width=2)  # top fold
    d.rectangle([11, 13, 21, 18], fill=(150, 52, 60, 255))  # brand band
    for gx in range(13, 20, 2):  # "UHT" glyph bars
        d.line([gx, 16, gx, 14], fill=(246, 240, 232, 255), width=1)
    d.line([13, 21, 19, 21], fill=(150, 148, 140, 255), width=1)
    d.line([13, 23, 19, 23], fill=(150, 148, 140, 255), width=1)
    d.ellipse([14, 24, 18, 26], fill=(200, 200, 206, 255))  # straw seal dot


def draw_milk_reconstituted(d, rng):
    # jug of reconstituted milk: paler, thinner, powder dust at the rim
    d.polygon([(12, 11), (20, 11), (22, 15), (22, 27), (10, 27), (10, 15)],
              fill=(214, 228, 234, 255))
    d.polygon([(12, 11), (20, 11), (22, 15), (22, 27), (10, 27), (10, 15)],
              outline=GLASS_EDGE)
    d.rectangle([11, 16, 21, 26], fill=(240, 244, 246, 255))  # thin milk
    d.line([11, 16, 21, 16], fill=(226, 234, 238, 255), width=1)
    d.rectangle([12, 8, 20, 11], fill=(196, 198, 204, 255))  # jug cap
    for _ in range(4):  # undissolved powder specks near the surface
        x = rng.randint(12, 20)
        d.point((x, 17), fill=(252, 252, 248, 255))
    d.arc([20, 13, 26, 21], start=270, end=90, fill=GLASS_EDGE, width=1)


def draw_mozzarella_smoked(d, rng):
    # smoked mozzarella pear: brown rind, tied knot at the top
    d.ellipse([10, 11, 22, 27], fill=(168, 116, 62, 255))
    d.ellipse([11, 12, 20, 25], fill=(188, 138, 78, 255))
    d.ellipse([12, 14, 18, 22], fill=(204, 156, 94, 255))
    d.rectangle([14, 7, 18, 12], fill=(150, 100, 54, 255))  # knot
    d.line([16, 4, 16, 7], fill=(216, 198, 160, 255), width=1)  # string
    for _ in range(5):  # smoke-dark patches
        x = rng.randint(11, 21)
        y = rng.randint(13, 25)
        d.point((x, y), fill=(128, 84, 44, 255))


def draw_pecorino(d, rng):
    # hard-cheese wedge with a dark rind on the curved edge
    d.polygon([(9, 24), (23, 24), (16, 9)], fill=(240, 226, 178, 255))
    d.polygon([(9, 24), (23, 24), (16, 9)], outline=(206, 188, 138, 255))
    d.line([9, 24, 16, 9], fill=(168, 124, 66, 255), width=2)  # rind
    for hx, hy in ((15, 17), (18, 20), (13, 21), (16, 13)):  # small holes
        d.point((hx, hy), fill=(222, 202, 148, 255))
    for cx_, cy_ in ((10, 27), (22, 27), (24, 25)):  # crumbs
        d.point((cx_, cy_), fill=(240, 226, 178, 255))


def draw_raw_butter(d, rng):
    # white stick of raw butter on its paper, scored cut face
    d.polygon([(8, 20), (24, 20), (24, 25), (8, 25)], fill=(206, 200, 182, 255))
    d.polygon([(8, 20), (24, 20), (24, 25), (8, 25)], outline=(168, 162, 144, 255))
    d.rectangle([9, 14, 23, 21], fill=(248, 246, 238, 255))
    d.rectangle([9, 14, 23, 21], outline=(216, 212, 198, 255), width=1)
    d.polygon([(9, 14), (12, 11), (26, 11), (23, 14)], fill=(252, 250, 244, 255))
    d.polygon([(9, 14), (12, 11), (26, 11), (23, 14)],
              outline=(216, 212, 198, 255))
    d.polygon([(23, 14), (26, 11), (26, 18), (23, 21)], fill=(240, 238, 228, 255))
    d.line([24, 13, 24, 19], fill=(216, 212, 198, 255), width=1)  # score


def draw_tiramisu(d, rng):
    # slice on a plate: cocoa top, cream/sponge layers
    d.line([6, 26, 26, 26], fill=(196, 198, 204, 255), width=2)  # plate
    d.rectangle([9, 10, 23, 25], fill=(96, 62, 38, 255))  # coffee sponge base
    d.rectangle([9, 10, 23, 13], fill=(74, 46, 30, 255))  # cocoa dusting
    d.rectangle([9, 14, 23, 17], fill=(246, 238, 220, 255))  # mascarpone cream
    d.rectangle([9, 18, 23, 20], fill=(110, 74, 46, 255))  # sponge layer
    d.rectangle([9, 21, 23, 25], fill=(246, 238, 220, 255))  # cream base
    for _ in range(10):  # cocoa speckle
        x = rng.randint(10, 22)
        y = rng.randint(10, 12)
        d.point((x, y), fill=(52, 32, 20, 255))
    d.point((11, 15), fill=(252, 246, 232, 255))


DRAW = {
    "PBJ_Toast": draw_pbj_toast,
    "buttered_toast": draw_buttered_toast,
    "frybread": draw_frybread,
    "hallula": draw_hallula,
    "hardtack": draw_hardtack,
    "hardtack_cracker": draw_hardtack_cracker,
    "jam_toast": draw_jam_toast,
    "johnnycake": draw_johnnycake,
    "peanut_butter_toast": draw_pb_toast,
    "soggy_hardtack": draw_soggy_hardtack,
    "sourdough_split_uncovered_active": draw_sourdough_split,
    "sourdough_starter_uncovered": draw_sourdough_starter,
    "sourdough_young_uncovered_active": draw_sourdough_young,
    "toad_in_a_hole": draw_toad_in_a_hole,
    "toast": draw_toast,
    "aquarium_large": draw_aquarium_large,
    "aquarium_medium": draw_aquarium_medium,
    "aquarium_small": draw_aquarium_small,
    "bag_garbage_reinforced": draw_garbage_bag,
    "bag_zipper_gallon": draw_zipper_bag,
    "bucket_wood": draw_bucket_wood,
    "carton_milk": draw_carton_milk,
    "large_stomach_sealed": draw_large_stomach_sealed,
    "lunchbox": draw_lunchbox,
    "stand_flyer": draw_stand_flyer,
    "stomach_sealed": draw_stomach_sealed,
    "waterskin": draw_waterskin,
    "waterskin2": draw_waterskin2,
    "waterskin3": draw_waterskin3,
    "wrapper_pr": draw_wrapper_pr,
    "cheese_powder": draw_cheese_powder,
    "cheese_rehydrated": draw_cheese_rehydrated,
    "cottage_cheese": draw_cottage_cheese,
    "cottage_cheese_with_fruit": draw_cottage_cheese_fruit,
    "cottage_cheese_with_savory_veg": draw_cottage_cheese_veg,
    "fried_mozzarella_sticks": draw_mozzarella_sticks,
    "mascarpone": draw_mascarpone,
    "milk_UHT": draw_milk_uht,
    "milk_choc": draw_milk_choc,
    "milk_raw": draw_milk_raw,
    "milk_reconstituted": draw_milk_reconstituted,
    "mozzarella_cheese_smoked": draw_mozzarella_smoked,
    "pecorino_cheese": draw_pecorino,
    "raw_butter": draw_raw_butter,
    "tiramisu": draw_tiramisu,
}


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    DRAW[sid](d, rng)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    os.makedirs(BASE, exist_ok=True)
    imgs = {}
    for sid in IDS:
        img = draw_one(sid)
        imgs[sid] = img
        folder = os.path.join(BASE, sid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, sid + ".png"))
        with open(os.path.join(folder, sid + ".json"), "w") as f:
            json.dump([{"id": sid, "fg": sid, "bg": ""}], f)
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS + "\n")
    # contact-sheet preview: 8 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item67.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item67.png {prev.size}")


if __name__ == "__main__":
    main()
