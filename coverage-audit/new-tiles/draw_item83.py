#!/usr/bin/env python3
"""ITEM83: casseroles, sandwiches, shelter kits, clothing, holiday
decor, oven parts, phase immersion gear, farm implements and methanol
cartridges -- CDDA Voxel coverage audit.

  Casseroles (5) -- a glass baking dish seen from above-ish, the
    topping surface carrying the dish: golden cheese + spinach
    flecks, tomato/cheese/eggplant parmigiana, corn kernels with
    chili heat, buttery cracker crumb with lobster chunks, creamy
    noodle with peas and tuna.
  Sandwiches (5) -- side profile, fillings layered by type: crispy
    chicken bun, lobster roll heaped in a split bun, egg salad on
    white, fluffernutter (PB + white fluff), grilled reuben on rye.
  Shelter kits (4) + tent pole -- bundles of sticks and rolled hide
    skins tied with cord; damaged variants torn, patched and with a
    snapped stick; the pole a segmented aluminium diagonal.
  Clothing (5) -- denim overalls, a light-gray fursuit with ears
    and tail, ski bib zipped (closed zip) vs unzipped (open dark
    V), dark zentai full-body suit.
  Holiday / artifacts (9) -- fog machine with fog puffs, inflatable
    dragon and ghost, jack-o'-lantern unlit (dark carved face) vs
    lit (glowing face), glowing magenta string in a test tube, red
    mp3 player off (dark screen) vs on (lit screen), electric rock
    with arcs, transparent cube-within-cube hypercube.
  Oven parts (5) -- budget-steel appliance pieces: broiler pan,
    stripped hollow cabinet, control panel with LCD and knobs,
    glass door, stovetop with its elements pulled off.
  Phase immersion (5) -- DARPA CCA electronics bundle off vs on
    (lit LED/screen), astronaut-like helmet, helmet with visor
    raised, helmet with SCBA regulator and lit status light.
  Farm implements (5) -- seed drill vs advanced (electronics box),
    plow with moldboard blades, reaper cutter bar vs advanced
    reaper with collection hopper.
  Methanol cartridges (4) -- yellow plastic fuel-cell tanks scaled
    by capacity (5 < 10 < 28 < 60 L) with coupling cap and hazard
    label.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "cheesy_spinach_casserole",
    "eggplant_parmigiana",
    "hot_corn_casserole",
    "lazy_lobster_casserole",
    "tuna_noodle_casserole",
    "chicken_sandwich",
    "lobster_roll",
    "sandwich_egg_salad",
    "sandwich_pbf",
    "sandwich_reuben",
    "damaged_shelter_kit",
    "large_damaged_shelter_kit",
    "large_shelter_kit",
    "shelter_kit",
    "tent_pole",
    "denim_overalls",
    "fursuit",
    "ski_bib",
    "ski_bib_unzipped",
    "zentai",
    "fog_machine",
    "inflatable_dragon",
    "inflatable_ghost",
    "jackolantern",
    "jackolantern_lit",
    "inert_glowing_string",
    "mp3_on_reverberation",
    "mp3_reverberation",
    "rock_electric",
    "string_dimension_hypercube",
    "oven_broiler_pan",
    "oven_cabinet",
    "oven_controls",
    "oven_door",
    "oven_stovetop",
    "phase_immersion_cca",
    "phase_immersion_cca_on",
    "phase_immersion_helmet",
    "phase_immersion_helmet_raised",
    "phase_immersion_helmet_scba_on",
    "v_planter_item",
    "v_planter_item_advanced",
    "v_plow_item",
    "v_reaper_item",
    "v_reaper_item_advanced",
    "10l_methanol_tank",
    "28l_methanol_tank",
    "5l_methanol_tank",
    "60l_methanol_tank",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (52, 52, 58, 255)
STEEL = (168, 174, 180, 255)
STEEL_DARK = (104, 110, 118, 255)
STEEL_PALE = (206, 210, 214, 255)
GLASS = (140, 200, 210, 255)
GLASS_DARK = (70, 110, 122, 255)
WOOD = (120, 86, 50, 255)
WOOD_DARK = (88, 60, 34, 255)
LEATHER = (150, 108, 66, 255)
LEATHER_DARK = (112, 78, 46, 255)
CHEESE = (238, 186, 74, 255)
BREAD = (226, 178, 104, 255)
BREAD_PALE = (244, 228, 188, 255)
CRUST = (186, 138, 72, 255)
RED = (198, 52, 42, 255)
GREEN = (66, 148, 74, 255)
LEAF = (76, 140, 66, 255)
CYAN = (90, 220, 230, 255)
MAGENTA = (232, 84, 220, 255)
YELLOW_TANK = (240, 198, 58, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def speckle(d, rng, box, colours, n):
    x0, y0, x1, y1 = box
    for _ in range(n):
        d.point((rng.randint(x0, x1), rng.randint(y0, y1)),
                fill=rng.choice(colours))


# ------------------------------------------------------------- casseroles

def casserole(d, rng, kind):
    # glass dish: pale blue-gray body, rim band, topping inside
    d.rectangle([4, 9, 27, 23], fill=(172, 192, 200, 255))
    d.rectangle([4, 9, 27, 23], outline=INK, width=1)
    d.rectangle([2, 12, 4, 17], fill=(172, 192, 200, 255), outline=INK)
    d.rectangle([27, 12, 29, 17], fill=(172, 192, 200, 255), outline=INK)
    inner = [6, 11, 25, 21]
    if kind == "spinach":
        d.rectangle(inner, fill=CHEESE)
        for _ in range(16):  # spinach leaves showing through the cheese
            x, y = rng.randint(7, 23), rng.randint(12, 20)
            d.ellipse([x, y, x + 2, y + 1], fill=(44, 104, 50, 255))
        speckle(d, rng, inner, [shade(CHEESE, 1.1), shade(CHEESE, 0.85)], 22)
    elif kind == "parmigiana":
        d.rectangle(inner, fill=(158, 74, 46, 255))  # tomato sauce
        for _ in range(6):  # melted cheese patches
            x, y = rng.randint(7, 21), rng.randint(12, 18)
            d.ellipse([x, y, x + 3, y + 2], fill=(240, 202, 112, 255))
        for _ in range(4):  # eggplant slices
            x, y = rng.randint(7, 22), rng.randint(12, 19)
            d.ellipse([x, y, x + 2, y + 2], fill=(96, 62, 112, 255))
        speckle(d, rng, inner, [(176, 96, 60, 255)], 14)
    elif kind == "corn":
        d.rectangle(inner, fill=(242, 202, 84, 255))
        for y in range(12, 21, 2):  # kernel rows
            for x in range(7, 25, 2):
                d.point((x, y), fill=(252, 226, 120, 255))
                d.point((x, y + 1), fill=(216, 168, 52, 255))
        for _ in range(6):  # chili heat
            d.point((rng.randint(7, 24), rng.randint(12, 20)), fill=RED)
        speckle(d, rng, inner, [LEAF], 6)
    elif kind == "lobster":
        d.rectangle(inner, fill=(216, 180, 122, 255))  # cracker crumb
        speckle(d, rng, inner, [(196, 156, 96, 255), (232, 200, 148, 255)], 26)
        for _ in range(7):  # lobster chunks
            x, y = rng.randint(7, 22), rng.randint(12, 19)
            d.ellipse([x, y, x + 2, y + 1], fill=(234, 122, 108, 255))
            d.point((x + 1, y), fill=(248, 220, 200, 255))
    elif kind == "tuna":
        d.rectangle(inner, fill=(238, 226, 182, 255))  # cream sauce
        for _ in range(7):  # noodle swirls
            x, y = rng.randint(7, 22), rng.randint(12, 19)
            d.arc([x, y, x + 3, y + 2], start=0, end=300,
                  fill=(222, 198, 132, 255), width=1)
        for _ in range(6):  # peas
            d.point((rng.randint(7, 24), rng.randint(12, 20)),
                    fill=(92, 162, 72, 255))
        speckle(d, rng, inner, [(192, 152, 142, 255)], 8)  # tuna flakes


# ------------------------------------------------------------- sandwiches

def sandwich_chicken(d, rng):
    d.polygon([(6, 13), (8, 8), (23, 8), (25, 13)], fill=BREAD)  # bun dome
    d.line([6, 13, 25, 13], fill=CRUST, width=1)
    speckle(d, rng, (9, 9, 22, 11), [(250, 240, 210, 255)], 5)  # sesame
    for x in range(7, 25, 3):  # lettuce fringe
        d.line([x, 14, x + 1, 16], fill=LEAF, width=1)
    d.rectangle([7, 15, 24, 16], fill=(226, 84, 60, 255))  # tomato
    # crispy chicken slab with a bumpy fried edge
    d.rectangle([6, 17, 25, 21], fill=(198, 132, 52, 255))
    for x in range(6, 26, 2):
        d.point((x, 21), fill=(172, 106, 38, 255))
        d.point((x, 17), fill=(224, 164, 84, 255))
    d.rectangle([7, 22, 24, 24], fill=BREAD)  # bun base
    d.line([7, 24, 24, 24], fill=CRUST, width=1)


def sandwich_lobster(d, rng):
    # split-top roll lying horizontally, lobster heaped in the split
    d.rectangle([4, 16, 27, 23], fill=BREAD)
    d.rectangle([4, 16, 27, 23], outline=CRUST, width=1)
    d.line([4, 19, 27, 19], fill=CRUST, width=1)
    for _ in range(11):  # lobster chunks mounded above the split
        x, y = rng.randint(6, 24), rng.randint(10, 15)
        d.ellipse([x, y, x + 3, y + 2], fill=(238, 138, 122, 255))
        d.point((x + 1, y), fill=(250, 228, 208, 255))
    d.line([6, 15, 25, 15], fill=(250, 228, 208, 255), width=1)  # mayo sheen


def sandwich_stack(d, rng, bread_top, bread_bot, layers):
    """Generic two-slice sandwich; layers = list of (colour, y0, y1)."""
    d.rectangle([7, 9, 24, 12], fill=bread_top)
    d.rectangle([7, 9, 24, 12], outline=CRUST, width=1)
    y = 13
    for col, h in layers:
        d.rectangle([8, y, 23, y + h - 1], fill=col)
        y += h
    d.rectangle([7, y, 24, y + 3], fill=bread_bot)
    d.rectangle([7, y, 24, y + 3], outline=CRUST, width=1)


def sandwich_egg(d, rng):
    sandwich_stack(d, rng, (250, 244, 226, 255), (250, 244, 226, 255),
                   [((246, 226, 142, 255), 3), ((238, 214, 120, 255), 2)])
    for _ in range(8):  # chopped egg white/yolk bits bulging out
        d.point((rng.randint(8, 23), rng.randint(13, 17)),
                fill=rng.choice([(255, 252, 240, 255), (240, 190, 60, 255)]))


def sandwich_pbf(d, rng):
    sandwich_stack(d, rng, (250, 244, 226, 255), (250, 244, 226, 255),
                   [((168, 112, 60, 255), 2), ((252, 250, 246, 255), 3)])
    d.line([8, 15, 23, 15], fill=(252, 250, 246, 255), width=1)  # fluff ooze


def sandwich_reuben(d, rng):
    rye = (122, 86, 54, 255)
    sandwich_stack(d, rng, rye, rye,
                   [((208, 122, 118, 255), 2), ((234, 228, 202, 255), 2),
                    ((208, 122, 118, 255), 1)])
    for x in (10, 15, 20):  # grill marks on the top slice
        d.line([x, 9, x + 2, 12], fill=(88, 58, 34, 255), width=1)
    d.line([12, 19, 12, 21], fill=(242, 172, 62, 255), width=1)  # cheese drip
    d.line([19, 19, 19, 20], fill=(242, 172, 62, 255), width=1)


# ------------------------------------------------------------ shelter kit

def shelter_kit(d, rng, large, damaged):
    x0, x1 = (3, 28) if large else (6, 25)
    ry0, ry1 = (7, 16) if large else (9, 16)
    # rolled hide skin bundle
    d.rectangle([x0, ry0, x1, ry1], fill=LEATHER)
    d.rectangle([x0, ry0, x1, ry1], outline=LEATHER_DARK, width=1)
    d.ellipse([x0, ry0, x0 + 4, ry1], fill=shade(LEATHER, 0.85),
              outline=LEATHER_DARK)
    d.ellipse([x1 - 4, ry0, x1, ry1], fill=shade(LEATHER, 1.1),
              outline=LEATHER_DARK)
    speckle(d, rng, (x0 + 2, ry0 + 1, x1 - 2, ry1 - 1),
            [shade(LEATHER, 0.9), shade(LEATHER, 1.08)], 14)
    # sticks underneath
    for i, yy in enumerate(range(ry1 + 3, ry1 + 8, 2)):
        d.line([x0 + (1 if i % 2 else 0), yy, x1 - (0 if i % 2 else 1), yy],
               fill=WOOD if i != 1 else WOOD_DARK, width=1)
    # cord straps
    for sx in (x0 + 5, x1 - 5):
        d.line([sx, ry0 - 1, sx, ry1 + 6], fill=(70, 52, 36, 255), width=1)
    if damaged:
        for _ in range(5):  # tears in the hide
            x, y = rng.randint(x0 + 2, x1 - 3), rng.randint(ry0 + 1, ry1 - 1)
            d.rectangle([x, y, x + 1, y], fill=(52, 36, 24, 255))
        d.rectangle([x0 + 6, ry0 + 2, x0 + 9, ry0 + 4],
                    fill=(168, 150, 118, 255), outline=LEATHER_DARK)  # patch
        # snapped stick poking out at an angle
        d.line([x1 - 2, ry1 + 5, x1 + 2, ry1 + 9], fill=WOOD_DARK, width=1)
        d.point((x1 + 2, ry1 + 9), fill=(210, 190, 160, 255))
        # one strap frayed: gap in the middle
        d.rectangle([x0 + 5, ry0 + 3, x0 + 5, ry0 + 5], fill=LEATHER)


def tent_pole(d, rng):
    # segmented aluminium pole on the diagonal, shock cord at the end
    pts = [(6, 25), (11, 20), (16, 15), (21, 10), (26, 5)]
    for a, b in zip(pts, pts[1:]):
        d.line([a, b], fill=(140, 144, 150, 255), width=2)
    for p in pts[1:-1]:
        d.ellipse([p[0] - 1, p[1] - 1, p[0] + 1, p[1] + 1],
                  fill=STEEL_DARK)  # joint ferrules
    d.line([26, 5, 28, 8], fill=INK, width=1)  # tip + cord
    d.arc([24, 6, 30, 12], start=90, end=270, fill=INK, width=1)


# ---------------------------------------------------------------- clothing

def overalls(d, rng, bib_col, leg_col, zipper):
    """Bib-overall form; zipper None = denim buttons, 'closed'/'open' ski."""
    # straps
    d.line([12, 3, 12, 7], fill=bib_col, width=2)
    d.line([19, 3, 19, 7], fill=bib_col, width=2)
    # bib
    d.rectangle([11, 7, 20, 14], fill=bib_col)
    d.rectangle([11, 7, 20, 14], outline=shade(bib_col, 0.8), width=1)
    d.point((12, 8), fill=(206, 172, 84, 255))  # strap buttons
    d.point((19, 8), fill=(206, 172, 84, 255))
    d.rectangle([14, 10, 17, 13], outline=shade(bib_col, 0.8))  # bib pocket
    # waist + legs
    d.rectangle([10, 14, 21, 28], fill=leg_col)
    d.line([15, 18, 15, 28], fill=shade(leg_col, 0.8), width=1)
    d.line([16, 18, 16, 28], fill=shade(leg_col, 0.8), width=1)
    d.rectangle([10, 14, 21, 15], fill=shade(leg_col, 0.88))  # waistband
    if zipper == "closed":
        for y in range(8, 14):
            d.point((15, y), fill=STEEL_PALE)
            d.point((16, y), fill=STEEL_DARK)
    elif zipper == "open":
        d.polygon([(13, 7), (18, 7), (16, 14), (15, 14)],
                  fill=(52, 60, 74, 255))  # dark inner V
        for y in range(8, 13):
            d.point((13 + (y - 8) // 3, y), fill=STEEL_PALE)
            d.point((18 - (y - 8) // 3, y), fill=STEEL_PALE)


def fursuit(d, rng):
    fur = (192, 192, 198, 255)
    fur_d = (164, 164, 172, 255)
    # tail curling out to the right
    d.arc([18, 16, 30, 29], start=270, end=90, fill=fur_d, width=3)
    # ears + head
    d.polygon([(11, 6), (12, 2), (14, 5)], fill=fur, outline=fur_d)
    d.polygon([(17, 5), (19, 2), (20, 6)], fill=fur, outline=fur_d)
    d.ellipse([10, 5, 21, 14], fill=fur, outline=fur_d)
    d.ellipse([13, 10, 18, 13], fill=(214, 208, 200, 255))  # muzzle
    d.point((15, 11), fill=(90, 80, 84, 255))  # nose
    d.point((12, 8), fill=INK)
    d.point((18, 8), fill=INK)
    # body, arms, legs
    d.rectangle([11, 14, 20, 22], fill=fur, outline=fur_d)
    d.ellipse([13, 16, 18, 21], fill=(214, 208, 200, 255))  # belly patch
    d.line([11, 15, 8, 21], fill=fur, width=3)
    d.line([20, 15, 23, 21], fill=fur, width=3)
    d.line([13, 22, 12, 28], fill=fur, width=3)
    d.line([18, 22, 19, 28], fill=fur, width=3)
    d.point((8, 22), fill=fur_d)  # paw pads
    d.point((23, 22), fill=fur_d)


def zentai(d, rng):
    suit = (72, 74, 80, 255)
    suit_d = (54, 56, 62, 255)
    d.ellipse([12, 3, 19, 11], fill=suit, outline=suit_d)  # covered head
    d.rectangle([11, 11, 20, 21], fill=suit, outline=suit_d)
    d.line([11, 12, 9, 20], fill=suit, width=3)  # arms tight to the body
    d.line([20, 12, 22, 20], fill=suit, width=3)
    d.line([13, 21, 13, 28], fill=suit, width=3)
    d.line([18, 21, 18, 28], fill=suit, width=3)
    d.line([13, 5, 13, 9], fill=shade(suit, 1.25), width=1)  # fabric sheen
    d.line([12, 13, 12, 19], fill=shade(suit, 1.2), width=1)


# ------------------------------------------------------- holiday/artifacts

def fog_machine(d, rng):
    d.rectangle([8, 14, 25, 25], fill=(74, 76, 84, 255))
    d.rectangle([8, 14, 25, 25], outline=INK, width=1)
    d.line([12, 14, 12, 11], fill=STEEL_DARK, width=1)  # handle
    d.line([12, 11, 21, 11], fill=STEEL_DARK, width=1)
    d.line([21, 11, 21, 14], fill=STEEL_DARK, width=1)
    d.polygon([(8, 17), (4, 19), (8, 21)], fill=STEEL_DARK)  # nozzle
    for (x, y, r) in ((2, 14, 2), (3, 11, 1), (1, 19, 1), (4, 23, 1)):
        d.ellipse([x, y, x + r + 1, y + r], fill=(206, 208, 214, 255))  # fog
    d.point((22, 17), fill=RED)  # power LED
    d.rectangle([11, 20, 18, 23], outline=STEEL_DARK)  # fluid window


def inflatable_dragon(d, rng):
    body = (84, 182, 94, 255)
    body_d = (58, 140, 70, 255)
    belly = (240, 226, 160, 255)
    # tail + body
    d.arc([3, 18, 14, 29], start=0, end=180, fill=body_d, width=3)
    d.ellipse([10, 14, 24, 26], fill=body, outline=body_d)
    d.ellipse([14, 17, 20, 25], fill=belly)  # belly
    # wing
    d.polygon([(18, 14), (26, 5), (27, 14), (22, 12)], fill=(140, 214, 140, 255),
              outline=body_d)
    # neck + head
    d.line([12, 15, 9, 9], fill=body, width=4)
    d.ellipse([5, 4, 13, 10], fill=body, outline=body_d)
    d.polygon([(6, 4), (5, 1), (8, 3)], fill=(250, 240, 200, 255))  # horns
    d.polygon([(10, 3), (10, 0), (12, 3)], fill=(250, 240, 200, 255))
    d.point((7, 6), fill=INK)
    d.point((12, 8), fill=RED)  # nostril
    d.rectangle([13, 26, 20, 28], fill=(90, 92, 98, 255))  # fan base


def inflatable_ghost(d, rng):
    ghost = (246, 246, 249, 255)
    edge = (188, 190, 200, 255)
    d.polygon([(9, 27), (8, 12), (10, 6), (14, 3), (18, 3), (22, 6),
               (24, 12), (23, 27), (20, 25), (17, 27), (14, 25), (11, 27)],
              fill=ghost, outline=edge)
    d.line([8, 14, 4, 17], fill=ghost, width=3)  # arms
    d.line([24, 14, 28, 17], fill=ghost, width=3)
    d.ellipse([12, 9, 14, 13], fill=INK)  # eyes
    d.ellipse([18, 9, 20, 13], fill=INK)
    d.ellipse([14, 15, 18, 19], fill=INK)  # mouth


def jackolantern(d, rng, lit):
    body = (238, 132, 42, 255) if lit else (226, 118, 36, 255)
    rib = shade(body, 0.82)
    d.ellipse([6, 8, 26, 27], fill=body)
    for x in (11, 16, 21):  # ribs
        d.arc([x - 5, 8, x + 5, 27], start=90, end=270, fill=rib, width=1)
        d.arc([x - 5, 8, x + 5, 27], start=270, end=90, fill=rib, width=1)
    d.rectangle([14, 4, 17, 8], fill=(96, 116, 52, 255))  # stem
    face = (255, 214, 84, 255) if lit else (44, 28, 20, 255)
    d.polygon([(10, 13), (13, 13), (11, 16)], fill=face)  # eyes
    d.polygon([(19, 13), (22, 13), (20, 16)], fill=face)
    d.point((15, 17), fill=face)  # nose
    d.polygon([(9, 20), (12, 22), (14, 20), (16, 22), (18, 20), (20, 22),
               (23, 20), (22, 23), (10, 23)], fill=face)  # jagged grin
    if lit:  # candle glow spilling around the carving
        speckle(d, rng, (8, 12, 24, 24), [(255, 190, 90, 255)], 8)


def glowing_string(d, rng):
    # test tube with a cork, magenta string twitching inside
    d.rectangle([13, 3, 18, 6], fill=(186, 146, 92, 255))  # cork
    d.rectangle([12, 6, 19, 27], outline=(196, 216, 222, 255), width=1)
    d.line([13, 8, 13, 25], fill=(228, 240, 244, 255), width=1)  # glass shine
    pts = [(14 + (2 if (y // 3) % 2 else 0), y) for y in range(10, 25)]
    d.line(pts, fill=MAGENTA, width=1)
    speckle(d, rng, (12, 9, 19, 26), [(255, 160, 250, 255)], 5)  # flicker


def mp3_player(d, rng, on):
    body = (226, 112, 106, 255)
    d.rectangle([10, 4, 21, 28], fill=body)
    d.rectangle([10, 4, 21, 28], outline=shade(body, 0.75), width=1)
    if on:
        d.rectangle([12, 6, 19, 14], fill=(214, 242, 236, 255))
        for i, w in enumerate((5, 3, 4)):  # "TRAVEL // JOURNEY" glyph bars
            d.line([13, 8 + i * 2, 13 + w, 8 + i * 2], fill=INK, width=1)
    else:
        d.rectangle([12, 6, 19, 14], fill=(48, 48, 58, 255))
    d.ellipse([12, 17, 19, 24], outline=(246, 206, 198, 255), width=1)  # wheel
    d.ellipse([14, 19, 17, 22], fill=(246, 206, 198, 255))
    if on:
        d.point((20, 5), fill=GREEN)


def rock_electric(d, rng):
    d.polygon([(9, 24), (7, 14), (12, 7), (20, 5), (25, 12), (23, 24)],
              fill=(30, 30, 38, 255), outline=(70, 70, 82, 255))
    d.polygon([(12, 7), (16, 12), (12, 18), (9, 14)], fill=(58, 58, 70, 255))
    d.line([14, 8, 17, 11], fill=(200, 205, 215, 255), width=1)  # shine
    arc = (130, 205, 255, 255)
    d.line([5, 10, 3, 13, 5, 15], fill=arc, width=1)  # sparks
    d.line([26, 18, 28, 20, 26, 22], fill=arc, width=1)
    d.line([18, 2, 17, 4], fill=arc, width=1)
    d.point((27, 8), fill=arc)


def hypercube(d, rng):
    line = (196, 240, 246, 255)
    outer = [(6, 6), (25, 6), (25, 25), (6, 25)]
    inner = [(12, 12), (20, 12), (20, 20), (12, 20)]
    for pts in (outer, inner):
        for a, b in zip(pts, pts[1:] + pts[:1]):
            d.line([a, b], fill=line, width=1)
    for a, b in zip(outer, inner):  # tesseract connectors
        d.line([a, b], fill=shade(line, 0.7), width=1)
    for p in outer + inner:
        d.point(p, fill=(255, 255, 255, 255))
    d.point((16, 16), fill=MAGENTA)  # impossible centre


# -------------------------------------------------------------- oven parts

def oven_broiler_pan(d, rng):
    d.rectangle([5, 13, 26, 21], fill=STEEL)
    d.rectangle([5, 13, 26, 21], outline=STEEL_DARK, width=1)
    d.rectangle([7, 15, 24, 19], fill=shade(STEEL, 0.88))
    for x in range(9, 24, 3):  # broiler slats
        d.line([x, 15, x, 19], fill=STEEL_DARK, width=1)
    d.rectangle([12, 17, 19, 18], fill=STEEL_DARK)  # handle slot shadow


def oven_cabinet(d, rng):
    d.rectangle([5, 5, 26, 27], fill=STEEL)
    d.rectangle([5, 5, 26, 27], outline=STEEL_DARK, width=1)
    d.rectangle([8, 8, 23, 24], fill=(56, 58, 64, 255))  # hollow interior
    d.rectangle([8, 8, 23, 24], outline=STEEL_DARK, width=1)
    for p in ((7, 7), (24, 7), (7, 25), (24, 25)):  # screw holes
        d.point(p, fill=INK)
    for y in (11, 14):  # stripped mounting rails
        d.line([9, y, 12, y], fill=STEEL_DARK, width=1)


def oven_controls(d, rng):
    d.rectangle([4, 11, 27, 21], fill=STEEL_PALE)
    d.rectangle([4, 11, 27, 21], outline=STEEL_DARK, width=1)
    d.rectangle([6, 13, 14, 18], fill=GLASS)  # LCD
    d.rectangle([6, 13, 14, 18], outline=GLASS_DARK, width=1)
    for i, w in enumerate((6, 4)):
        d.line([7, 15 + i * 2, 7 + w, 15 + i * 2], fill=(30, 90, 96, 255),
               width=1)
    for cx in (18, 22, 26 - 1):  # knobs
        d.ellipse([cx - 2, 13, cx + 2, 18], fill=STEEL_DARK, outline=INK)
        d.point((cx, 14), fill=STEEL_PALE)
    for x in (7, 10, 13):  # membrane buttons under the LCD edge
        d.point((x, 20), fill=STEEL_DARK)


def oven_door(d, rng):
    d.rectangle([6, 4, 25, 28], fill=STEEL)
    d.rectangle([6, 4, 25, 28], outline=STEEL_DARK, width=1)
    d.rectangle([8, 5, 23, 7], fill=STEEL_DARK)  # handle bar
    d.rectangle([9, 10, 22, 23], fill=GLASS_DARK)  # window
    d.rectangle([9, 10, 22, 23], outline=INK, width=1)
    d.line([11, 21, 16, 11], fill=shade(GLASS, 1.05), width=2)  # glare streak
    d.line([18, 22, 21, 16], fill=GLASS, width=1)


def oven_stovetop(d, rng):
    d.rectangle([4, 10, 27, 22], fill=(88, 92, 98, 255))
    d.rectangle([4, 10, 27, 22], outline=INK, width=1)
    for (cx, cy, r) in ((10, 14, 3), (21, 14, 3), (10, 19, 2), (21, 19, 2)):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=STEEL_DARK,
                  width=1)  # empty burner seats, elements pulled off
        d.point((cx, cy), fill=INK)  # screw hole
    d.line([6, 8, 10, 10], fill=(150, 90, 60, 255), width=1)  # wire stubs
    d.line([22, 8, 24, 10], fill=(150, 90, 60, 255), width=1)
    d.rectangle([24, 6, 27, 9], fill=STEEL_DARK)  # leftover component block


# --------------------------------------------------------- phase immersion

def cca(d, rng, on):
    dark = (78, 82, 90, 255)
    # headband + earcups (headphones with active noise suppression)
    d.arc([9, 5, 22, 16], start=180, end=360, fill=dark, width=3)
    d.rectangle([7, 11, 11, 18], fill=dark, outline=INK)
    d.rectangle([20, 11, 24, 18], fill=dark, outline=INK)
    d.line([9, 18, 13, 22], fill=dark, width=1)  # mic boom
    d.point((13, 22), fill=INK)
    # carrier box with battery + panoramic NV electronics
    d.rectangle([11, 21, 21, 28], fill=(96, 100, 108, 255), outline=INK)
    d.arc([21, 22, 27, 28], start=270, end=90, fill=dark, width=1)  # cable
    if on:
        d.rectangle([13, 23, 17, 26], fill=(120, 240, 200, 255))  # live screen
        d.point((19, 24), fill=GREEN)
        d.point((19, 26), fill=CYAN)
    else:
        d.rectangle([13, 23, 17, 26], fill=(44, 46, 52, 255))
        d.point((19, 24), fill=(90, 60, 56, 255))


def helmet(d, rng, raised=False, scba=False):
    shell = (202, 206, 210, 255)
    shell_d = (150, 154, 160, 255)
    d.pieslice([6, 3, 25, 22], start=180, end=360, fill=shell, outline=shell_d)
    d.rectangle([6, 12, 25, 19], fill=shell)
    d.rectangle([6, 19, 25, 21], fill=shell_d)  # rim band
    d.ellipse([7, 10, 10, 14], fill=shell_d)  # side pods
    d.ellipse([21, 10, 24, 14], fill=shell_d)
    if raised:
        # visor flipped up above the brow, face opening dark beneath
        d.rectangle([9, 1, 22, 6], fill=GLASS_DARK, outline=INK)
        d.rectangle([10, 13, 21, 19], fill=(40, 42, 48, 255))
    else:
        d.rectangle([10, 10, 21, 18], fill=GLASS_DARK)  # ballistic visor
        d.rectangle([10, 10, 21, 18], outline=INK, width=1)
        d.line([12, 17, 16, 11], fill=GLASS, width=2)  # glare
    if scba:
        d.ellipse([13, 19, 18, 24], fill=(70, 74, 80, 255), outline=INK)
        d.point((15, 21), fill=INK)  # regulator
        d.arc([16, 20, 27, 30], start=0, end=120, fill=(70, 74, 80, 255),
              width=2)  # hose
        d.point((23, 12), fill=GREEN)  # SCBA status light


# -------------------------------------------------------------- farm parts

def wheels(d, cx1, cx2, cy, r=3):
    for cx in (cx1, cx2):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(52, 52, 56, 255),
                  outline=INK)
        d.point((cx, cy), fill=STEEL)


def planter(d, rng, advanced):
    frame = STEEL_DARK if advanced else (128, 134, 140, 255)
    d.rectangle([10, 7, 21, 13], fill=(110, 118, 126, 255), outline=INK)
    speckle(d, rng, (11, 8, 20, 12), [(214, 190, 130, 255)], 8)  # seed fill
    d.line([5, 16, 26, 16], fill=frame, width=2)  # tool bar
    for x in (9, 14, 19, 24):  # planting tubes + spikes
        d.line([x, 13, x, 16], fill=frame, width=1)
        d.polygon([(x - 1, 16), (x + 1, 16), (x, 21)], fill=frame)
    wheels(d, 9, 23, 24)
    if advanced:
        d.rectangle([22, 5, 28, 11], fill=(52, 56, 62, 255), outline=INK)
        d.rectangle([23, 6, 26, 8], fill=CYAN)  # controller screen
        d.line([22, 11, 19, 13], fill=INK, width=1)  # wiring


def plow(d, rng):
    d.line([4, 13, 27, 13], fill=(120, 126, 134, 255), width=2)  # frame
    for x in (8, 15, 22):  # moldboard blades curving under
        d.polygon([(x, 13), (x + 4, 13), (x + 5, 19), (x + 2, 21),
                   (x, 18)], fill=(148, 156, 164, 255), outline=STEEL_DARK)
        d.line([x + 1, 14, x + 3, 19], fill=STEEL_PALE, width=1)
    wheels(d, 8, 24, 24)
    d.line([4, 13, 2, 9], fill=STEEL_DARK, width=2)  # hitch


def reaper(d, rng, advanced):
    d.line([4, 20, 27, 20], fill=STEEL_DARK, width=2)  # cutter bar
    for x in range(5, 27, 3):  # teeth
        d.polygon([(x, 20), (x + 2, 20), (x + 1, 23)], fill=STEEL)
    d.line([6, 20, 6, 13], fill=STEEL_DARK, width=1)  # supports
    d.line([25, 20, 25, 13], fill=STEEL_DARK, width=1)
    d.line([6, 13, 25, 13], fill=STEEL_DARK, width=1)
    wheels(d, 9, 23, 26, r=2)
    if advanced:
        d.rectangle([10, 6, 22, 13], fill=(104, 112, 120, 255), outline=INK)
        d.line([10, 9, 22, 9], fill=STEEL_DARK, width=1)  # collection hopper
        d.rectangle([23, 5, 28, 10], fill=(52, 56, 62, 255), outline=INK)
        d.point((25, 7), fill=GREEN)  # electronics
    else:
        d.line([20, 13, 24, 8], fill=WOOD_DARK, width=1)  # engage lever
        d.point((24, 8), fill=INK)


# ---------------------------------------------------------- methanol tanks

def methanol_tank(d, rng, scale):
    """scale = (x0, y0, x1, y1) body box; bigger for bigger cartridges."""
    x0, y0, x1, y1 = scale
    d.rectangle([x0, y0 + 3, x1, y1], fill=YELLOW_TANK)
    d.rectangle([x0, y0 + 3, x1, y1], outline=shade(YELLOW_TANK, 0.72),
                width=1)
    d.line([x0 + 2, y0 + 5, x0 + 2, y1 - 2], fill=shade(YELLOW_TANK, 1.1),
           width=1)  # moulded rib sheen
    # specialised fuel-cell coupling on top
    cx = (x0 + x1) // 2
    d.rectangle([cx - 2, y0, cx + 2, y0 + 3], fill=STEEL_DARK, outline=INK)
    d.point((cx, y0 + 1), fill=CYAN)
    # hazard label
    lw = max(4, (x1 - x0) // 3)
    lx, ly = cx - lw // 2, (y0 + y1) // 2 - 2
    d.rectangle([lx, ly, lx + lw, ly + 5], fill=(248, 246, 240, 255),
                outline=INK)
    d.polygon([(cx, ly + 1), (cx + 1, ly + 2), (cx, ly + 3), (cx - 1, ly + 2)],
              fill=RED)  # flammable diamond
    d.line([lx + 1, ly + 4, lx + lw - 1, ly + 4], fill=INK, width=1)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "cheesy_spinach_casserole":
        casserole(d, rng, "spinach")
    elif sid == "eggplant_parmigiana":
        casserole(d, rng, "parmigiana")
    elif sid == "hot_corn_casserole":
        casserole(d, rng, "corn")
    elif sid == "lazy_lobster_casserole":
        casserole(d, rng, "lobster")
    elif sid == "tuna_noodle_casserole":
        casserole(d, rng, "tuna")
    elif sid == "chicken_sandwich":
        sandwich_chicken(d, rng)
    elif sid == "lobster_roll":
        sandwich_lobster(d, rng)
    elif sid == "sandwich_egg_salad":
        sandwich_egg(d, rng)
    elif sid == "sandwich_pbf":
        sandwich_pbf(d, rng)
    elif sid == "sandwich_reuben":
        sandwich_reuben(d, rng)
    elif sid == "shelter_kit":
        shelter_kit(d, rng, large=False, damaged=False)
    elif sid == "large_shelter_kit":
        shelter_kit(d, rng, large=True, damaged=False)
    elif sid == "damaged_shelter_kit":
        shelter_kit(d, rng, large=False, damaged=True)
    elif sid == "large_damaged_shelter_kit":
        shelter_kit(d, rng, large=True, damaged=True)
    elif sid == "tent_pole":
        tent_pole(d, rng)
    elif sid == "denim_overalls":
        overalls(d, rng, (110, 150, 192, 255), (96, 134, 178, 255), None)
    elif sid == "fursuit":
        fursuit(d, rng)
    elif sid == "ski_bib":
        overalls(d, rng, (122, 172, 222, 255), (106, 156, 206, 255), "closed")
    elif sid == "ski_bib_unzipped":
        overalls(d, rng, (122, 172, 222, 255), (106, 156, 206, 255), "open")
    elif sid == "zentai":
        zentai(d, rng)
    elif sid == "fog_machine":
        fog_machine(d, rng)
    elif sid == "inflatable_dragon":
        inflatable_dragon(d, rng)
    elif sid == "inflatable_ghost":
        inflatable_ghost(d, rng)
    elif sid == "jackolantern":
        jackolantern(d, rng, lit=False)
    elif sid == "jackolantern_lit":
        jackolantern(d, rng, lit=True)
    elif sid == "inert_glowing_string":
        glowing_string(d, rng)
    elif sid == "mp3_reverberation":
        mp3_player(d, rng, on=False)
    elif sid == "mp3_on_reverberation":
        mp3_player(d, rng, on=True)
    elif sid == "rock_electric":
        rock_electric(d, rng)
    elif sid == "string_dimension_hypercube":
        hypercube(d, rng)
    elif sid == "oven_broiler_pan":
        oven_broiler_pan(d, rng)
    elif sid == "oven_cabinet":
        oven_cabinet(d, rng)
    elif sid == "oven_controls":
        oven_controls(d, rng)
    elif sid == "oven_door":
        oven_door(d, rng)
    elif sid == "oven_stovetop":
        oven_stovetop(d, rng)
    elif sid == "phase_immersion_cca":
        cca(d, rng, on=False)
    elif sid == "phase_immersion_cca_on":
        cca(d, rng, on=True)
    elif sid == "phase_immersion_helmet":
        helmet(d, rng)
    elif sid == "phase_immersion_helmet_raised":
        helmet(d, rng, raised=True)
    elif sid == "phase_immersion_helmet_scba_on":
        helmet(d, rng, scba=True)
    elif sid == "v_planter_item":
        planter(d, rng, advanced=False)
    elif sid == "v_planter_item_advanced":
        planter(d, rng, advanced=True)
    elif sid == "v_plow_item":
        plow(d, rng)
    elif sid == "v_reaper_item":
        reaper(d, rng, advanced=False)
    elif sid == "v_reaper_item_advanced":
        reaper(d, rng, advanced=True)
    elif sid == "5l_methanol_tank":
        methanol_tank(d, rng, (11, 13, 20, 27))
    elif sid == "10l_methanol_tank":
        methanol_tank(d, rng, (10, 11, 21, 28))
    elif sid == "28l_methanol_tank":
        methanol_tank(d, rng, (8, 8, 23, 29))
    elif sid == "60l_methanol_tank":
        methanol_tank(d, rng, (6, 5, 25, 30))
    else:
        raise KeyError(sid)
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
    # contact-sheet preview: 7 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item83.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item83.png {prev.size}")


if __name__ == "__main__":
    main()
