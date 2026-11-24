#!/usr/bin/env python3
"""ITEM54: cooking / kitchen equipment -- CDDA Voxel coverage audit.

All 27 items are from items/tool/cooking.json.  Each appliance is
drawn as itself:

  Cookers -- a green-enamel steel heater body with a burner grate,
  told apart by the fuel gear its description names:
    acetylene: brass valve fittings and a gas cylinder beside it;
    biogas: a hose running to a soft gas bag;
    charcoal: open coal bed (black lumps) and a vent pipe;
    gasoline: a small fuel tank with a pump knob;
    oil (kerosene): a wick knob and a taller tank chimney;
    induction hotplate: no fuel at all -- a flat dark top with
      concentric induction-coil rings and touch dots.
  Fermenting / culturing (all _active: the process is running):
    eggs / pickles / sauerkraut in glass jars of brine with
    bubbles rising; a sealed yeast flask with a foamy head;
    crystallizing sugar as a syrup pan sprouting white crystals.
  The rest as themselves: hand-crank can sealer with a tin can,
  a buzzing yellow electric carver (on), a layered-charcoal water
  purifier, a clamped makeshift pressure cooker with a steam
  wisp, a makeshift vacuum sealer with pump hose and bag, a steel
  honing rod, manual (crank + hotplate) and primitive (wooden
  barrel + big screw + kerosene cooker) oil presses, a hand-crank
  pasta extruder with strands emerging, a wooden screw press,
  wicker / lab-steel / makeshift-wire sieves, a home snow machine
  filling a snow-cone cup, a copper still with condenser coil,
  and a bucket sand-and-gravel water filter.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "acetylene_cooker",
    "biogas_cooker",
    "can_sealer",
    "carver_on",
    "char_purifier",
    "charcoal_cooker",
    "crystallizing_sugar_active",
    "eggs_ferment_active",
    "flask_yeast_active",
    "gasoline_cooker",
    "hotplate_induction",
    "makeshift_pressure_cooker",
    "makeshift_sealer",
    "metal_honing_rod",
    "oil_cooker",
    "oil_press_manual",
    "oil_press_primitive",
    "pastaextruder",
    "pickles_ferment_active",
    "sauerkraut_ferment_active",
    "screw_press",
    "sieve_primitive",
    "sieve_steel",
    "sieve_steel_makeshift",
    "snow_machine_home",
    "still",
    "water_filter_sand",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

STEEL = (120, 126, 134, 255)
STEEL_DARK = (70, 74, 82, 255)
STEEL_LIGHT = (178, 184, 192, 255)
GREEN_ENAMEL = (52, 116, 62, 255)
GREEN_DARK = (34, 78, 42, 255)
BRASS = (198, 158, 74, 255)
COPPER = (178, 102, 58, 255)
COPPER_LIGHT = (222, 148, 92, 255)
WOOD = (122, 86, 48, 255)
WOOD_DARK = (84, 58, 30, 255)
WICKER = (186, 152, 96, 255)
GLASS = (196, 222, 226, 255)
BRINE = (196, 178, 92, 255)
FLAME = (240, 140, 40, 255)
FLAME_IN = (250, 210, 90, 255)
BLACK = (26, 26, 30, 255)
PLASTIC_WHITE = (232, 232, 228, 255)
YELLOW = (238, 196, 52, 255)
RED = (198, 52, 44, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ cookers

def cooker_body(d, x0=6, x1=23, y0=12, y1=26):
    """Green-enamel heater: grate on top, body, burner glow, feet."""
    d.rectangle([x0, y0 + 2, x1, y1], fill=GREEN_ENAMEL)
    d.rectangle([x0, y0 + 2, x1, y0 + 4], fill=GREEN_DARK)
    d.line([x0, y1, x1, y1], fill=GREEN_DARK, width=1)
    for gx in range(x0 + 1, x1, 3):  # burner grate bars
        d.line([gx, y0, gx, y0 + 2], fill=STEEL_DARK, width=1)
    d.line([x0, y0 + 1, x1, y0 + 1], fill=STEEL_DARK, width=1)
    d.ellipse([13, y0 + 3, 17, y0 + 6], fill=FLAME)  # burner flame
    d.point((15, y0 + 4), fill=FLAME_IN)
    d.rectangle([x0 + 2, y1 - 5, x0 + 4, y1 - 3], fill=STEEL_DARK)  # knob
    d.line([x0 + 1, y1 + 1, x0 + 2, y1 + 2], fill=STEEL_DARK, width=1)
    d.line([x1 - 2, y1 + 1, x1 - 1, y1 + 2], fill=STEEL_DARK, width=1)


def draw_cooker(d, rng, sid):
    if sid == "charcoal_cooker":
        # vent pipe rising behind, open coal bed instead of a grate
        d.rectangle([21, 3, 23, 13], fill=STEEL)
        d.line([21, 3, 23, 3], fill=STEEL_DARK, width=1)
        d.point((22, 1), fill=(170, 170, 176, 255))  # fume wisps
        d.point((21, 2), fill=(170, 170, 176, 255))
        cooker_body(d)
        d.rectangle([8, 10, 21, 13], fill=STEEL_DARK)  # coal bed rim
        for cx, cy in ((10, 11), (14, 10), (18, 11), (12, 12), (16, 12)):
            d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=BLACK)
        d.point((14, 10), fill=FLAME)  # pilot light glow among coals
        d.point((18, 11), fill=FLAME)
        return
    cooker_body(d)
    if sid == "acetylene_cooker":
        # brass valve block on the body, gas cylinder standing beside
        d.rectangle([25, 10, 29, 27], fill=(104, 142, 158, 255))
        d.rectangle([25, 10, 29, 12], fill=STEEL_DARK)
        d.rectangle([26, 7, 28, 10], fill=BRASS)  # cylinder valve
        d.point((27, 6), fill=BRASS)
        d.rectangle([17, 17, 20, 19], fill=BRASS)  # brass fittings
        d.line([24, 14, 20, 18], fill=BLACK, width=1)  # hose
        d.line([24, 22, 21, 20], fill=BLACK, width=1)
    elif sid == "biogas_cooker":
        # hose to a soft, sagging gas bag on the floor
        d.line([23, 18, 27, 22], fill=BLACK, width=1)
        d.ellipse([24, 21, 30, 28], fill=(140, 128, 84, 255))
        d.ellipse([24, 21, 30, 28], outline=shade((140, 128, 84, 255), 0.6))
        d.line([25, 24, 29, 24], fill=shade((140, 128, 84, 255), 0.7),
               width=1)  # bag seam
        d.rectangle([18, 17, 20, 19], fill=STEEL_DARK)
    elif sid == "gasoline_cooker":
        # small fuel tank with filler cap and pump knob on the right
        d.rectangle([24, 15, 29, 24], fill=STEEL)
        d.rectangle([25, 12, 27, 15], fill=STEEL_DARK)  # filler neck/cap
        d.line([24, 18, 23, 18], fill=BLACK, width=1)
        d.ellipse([25, 20, 27, 22], fill=RED)  # pump knob dot
        d.rectangle([17, 17, 20, 19], fill=STEEL_DARK)
    elif sid == "oil_cooker":
        # kerosene: taller tank chimney behind + wick knob
        d.rectangle([20, 5, 24, 13], fill=STEEL)
        d.rectangle([20, 5, 24, 7], fill=STEEL_DARK)
        d.point((22, 3), fill=(170, 170, 176, 255))  # faint fume
        d.ellipse([16, 17, 19, 20], outline=BRASS, width=1)  # wick knob
        d.point((17, 18), fill=BRASS)


def draw_hotplate(d, rng):
    # flat induction top: dark glass slab, coil rings, touch dots
    d.rectangle([5, 13, 27, 25], fill=(36, 36, 42, 255))
    d.rectangle([5, 13, 27, 25], outline=STEEL_DARK, width=1)
    d.rectangle([5, 23, 27, 25], fill=(52, 52, 60, 255))  # control strip
    for r in (6, 4, 2):  # concentric coil rings on the top face
        d.ellipse([16 - r, 15 + (6 - r) // 2, 16 + r, 21 - (6 - r) // 2],
                  outline=(120, 60, 50, 255), width=1)
    d.point((16, 18), fill=(120, 60, 50, 255))
    for tx in (8, 12, 21, 25):  # touch controls
        d.point((tx, 24), fill=(140, 200, 220, 255))
    d.rectangle([6, 26, 8, 27], fill=STEEL_DARK)
    d.rectangle([24, 26, 26, 27], fill=STEEL_DARK)


# --------------------------------------------------------------------- jars

def jar(d, x0=10, x1=22, y0=6, y1=27):
    """Glass jar outline with lid; returns inner bounds."""
    d.rectangle([x0 + 2, y0, x1 - 2, y0 + 2], fill=STEEL)  # lid
    d.line([x0 + 2, y0, x1 - 2, y0], fill=STEEL_LIGHT, width=1)
    d.rectangle([x0, y0 + 3, x1, y1], fill=shade(GLASS, 0.92))
    d.rectangle([x0, y0 + 3, x1, y1], outline=shade(GLASS, 0.62), width=1)
    d.line([x0 + 1, y0 + 5, x0 + 1, y1 - 2], fill=(238, 250, 250, 255),
           width=1)  # glass highlight
    return x0 + 1, x1 - 1, y0 + 4, y1 - 1


def bubbles(d, rng, x0, x1, y0, y1, n=7):
    for _ in range(n):
        bx = rng.randint(x0, x1)
        by = rng.randint(y0, y1)
        d.point((bx, by), fill=(250, 246, 210, 255))


def draw_ferment(d, rng, sid):
    if sid == "flask_yeast_active":
        ix0, ix1, iy0, iy1 = jar(d, x0=11, x1=21, y0=8, y1=27)
        d.rectangle([ix0, iy0 + 5, ix1, iy1], fill=(206, 168, 92, 255))
        # foamy yeast head crowding the top of the wort
        for fx in range(ix0, ix1 + 1, 2):
            d.ellipse([fx, iy0 + 3, fx + 2, iy0 + 6],
                      fill=(246, 232, 186, 255))
        bubbles(d, rng, ix0 + 1, ix1 - 1, iy0 + 8, iy1 - 1, n=5)
        d.rectangle([15, 4, 17, 8], fill=STEEL_DARK)  # airlock on the seal
        d.point((16, 3), fill=STEEL_LIGHT)
        return
    ix0, ix1, iy0, iy1 = jar(d)
    d.rectangle([ix0, iy0 + 4, ix1, iy1], fill=BRINE)  # brine fill
    if sid == "eggs_ferment_active":
        for ex, ey in ((13, 14), (18, 13), (15, 19), (19, 20), (13, 23)):
            d.ellipse([ex - 2, ey - 3, ex + 2, ey + 3],
                      fill=(244, 238, 220, 255))
            d.ellipse([ex - 2, ey - 3, ex + 2, ey + 3],
                      outline=(196, 182, 148, 255), width=1)
    elif sid == "pickles_ferment_active":
        for px in (12, 16, 20):  # pickle spears standing in the brine
            d.rectangle([px, iy0 + 6, px + 2, iy1 - 1],
                        fill=(74, 128, 52, 255))
            d.line([px, iy0 + 6, px, iy1 - 1],
                   fill=(52, 96, 38, 255), width=1)
            for by in range(iy0 + 8, iy1 - 1, 3):
                d.point((px + 1, by), fill=(120, 168, 92, 255))
        d.ellipse([13, iy0 + 4, 17, iy0 + 7], fill=(86, 142, 60, 255))
    elif sid == "sauerkraut_ferment_active":
        for y in range(iy0 + 7, iy1, 2):  # packed shredded cabbage
            for x in range(ix0 + 1, ix1, 3):
                d.line([x, y, x + 1, y], fill=(232, 224, 178, 255),
                       width=1)
        d.rectangle([ix0 + 1, iy0 + 4, ix1 - 1, iy0 + 6],
                    fill=(214, 206, 160, 255))  # weight plate on top
    bubbles(d, rng, ix0 + 1, ix1 - 1, iy0 + 5, iy1 - 1)


# ------------------------------------------------------------------- others

def draw_can_sealer(d, rng):
    d.rectangle([7, 22, 26, 27], fill=STEEL_DARK)  # cast base
    d.rectangle([8, 8, 12, 22], fill=STEEL)  # column
    d.rectangle([8, 6, 22, 10], fill=STEEL)  # head arm
    d.ellipse([14, 10, 20, 15], fill=STEEL_DARK)  # seaming head
    d.rectangle([15, 15, 19, 22], fill=STEEL_LIGHT)  # the tin can
    d.line([15, 18, 19, 18], fill=STEEL, width=1)
    d.line([22, 8, 26, 5], fill=STEEL_DARK, width=2)  # crank arm
    d.ellipse([25, 3, 28, 6], fill=WOOD_DARK)  # crank handle
    d.rectangle([9, 23, 11, 25], fill=BLACK)


def draw_carver(d, rng):
    # yellow body held diagonally, twin serrated blades buzzing out front
    d.polygon([(6, 20), (14, 12), (18, 14), (12, 23)], fill=YELLOW)
    d.polygon([(6, 20), (14, 12), (16, 13), (9, 21)],
              fill=shade(YELLOW, 0.8))
    d.rectangle([4, 20, 8, 25], fill=shade(YELLOW, 0.7))  # grip end
    for bx in (19, 21):  # twin blades
        pts = [(bx, 13)]
        for i in range(5):
            pts.append((bx + 4 + (i % 2), 14 + i * 2))
        d.line(pts, fill=STEEL_LIGHT, width=1)
    for mx, my in ((26, 10), (28, 16), (25, 22)):  # buzz motion ticks
        d.point((mx, my), fill=STEEL)
    d.line([5, 25, 3, 29], fill=BLACK, width=1)  # cord
    d.line([3, 29, 7, 30], fill=BLACK, width=1)


def draw_char_purifier(d, rng):
    # plastic housing with the layered charcoal fill visible in a window
    d.rectangle([10, 5, 22, 27], fill=PLASTIC_WHITE)
    d.rectangle([10, 5, 22, 27], outline=STEEL, width=1)
    d.rectangle([13, 3, 19, 5], fill=STEEL)  # inlet cap
    d.rectangle([12, 9, 20, 24], fill=(210, 206, 198, 255))
    layers = [(11, (172, 164, 148, 255)), (14, BLACK), (17, (120, 116, 108, 255)),
              (20, BLACK), (22, (186, 178, 160, 255))]
    for ly, col in layers:  # gravel / charcoal / sand bands
        d.rectangle([12, ly, 20, ly + 2], fill=col)
    for _ in range(6):  # charcoal flecks
        d.point((rng.randint(13, 19), rng.randint(14, 21)), fill=(60, 60, 64, 255))
    d.line([16, 24, 16, 26], fill=(120, 170, 220, 255), width=1)  # clean drip


def draw_sugar(d, rng):
    # shallow pan of syrup with sugar crystals forming on strings
    d.polygon([(5, 14), (27, 14), (25, 25), (7, 25)], fill=STEEL)
    d.polygon([(5, 14), (27, 14), (25, 25), (7, 25)], outline=STEEL_DARK)
    d.rectangle([6, 12, 26, 14], fill=(226, 178, 84, 255))  # syrup surface
    for sx in (10, 16, 22):  # strings across the pan
        d.line([sx, 8, sx, 13], fill=WOOD_DARK, width=1)
    d.line([8, 8, 24, 8], fill=WOOD_DARK, width=1)  # skewer bar
    for cx, cy in ((10, 13), (16, 12), (22, 13), (13, 14), (19, 14)):
        for dx, dy in ((0, 0), (-1, 1), (1, 1), (0, 2)):  # crystal clusters
            d.point((cx + dx, cy + dy), fill=(252, 252, 250, 255))
    for _ in range(5):  # sparkle
        d.point((rng.randint(7, 25), rng.randint(10, 13)),
                fill=(252, 252, 250, 255))


def draw_pressure_cooker(d, rng):
    d.polygon([(8, 13), (24, 13), (25, 26), (7, 26)], fill=STEEL)
    d.line([8, 20, 25, 20], fill=STEEL_LIGHT, width=1)
    d.rectangle([7, 10, 25, 13], fill=STEEL_DARK)  # thick clamped lid
    for cx in (10, 16, 22):  # lid clamps
        d.rectangle([cx - 1, 9, cx + 1, 12], fill=BLACK)
    d.rectangle([15, 6, 17, 10], fill=BRASS)  # pressure weight valve
    d.point((16, 4), fill=(230, 230, 234, 255))  # steam wisp
    d.point((17, 3), fill=(230, 230, 234, 255))
    d.line([7, 18, 4, 16], fill=STEEL_DARK, width=2)  # handles
    d.line([25, 18, 28, 16], fill=STEEL_DARK, width=2)


def draw_makeshift_sealer(d, rng):
    d.rectangle([5, 18, 22, 26], fill=PLASTIC_WHITE)  # sealer body
    d.rectangle([5, 18, 22, 26], outline=STEEL, width=1)
    d.rectangle([6, 16, 21, 19], fill=RED)  # heat-seal bar
    # a plastic bag pinched in the bar, air sucked out (wrinkled flat)
    d.polygon([(8, 19), (19, 19), (18, 27), (9, 27)], fill=GLASS)
    for wx in (11, 14, 17):
        d.line([wx, 20, wx - 1, 26], fill=shade(GLASS, 0.7), width=1)
    d.line([22, 22, 27, 24], fill=BLACK, width=1)  # hose to the pump
    d.ellipse([25, 22, 30, 28], fill=STEEL)
    d.point((27, 25), fill=STEEL_DARK)


def draw_honing_rod(d, rng):
    d.line([7, 24, 22, 9], fill=STEEL, width=3)  # the rod, diagonal
    d.line([7, 24, 22, 9], fill=STEEL_LIGHT, width=1)
    for t in range(3, 13, 2):  # fine grit ticks along the rod
        d.point((7 + t, 24 - t), fill=STEEL_DARK)
    d.line([5, 26, 8, 23], fill=BRASS, width=2)  # guard
    d.polygon([(2, 29), (5, 25), (8, 27), (5, 30)], fill=WOOD_DARK)  # handle
    d.point((27, 6), fill=STEEL_LIGHT)  # honed-edge glint at the tip


def press_frame(d, x0=8, x1=24):
    d.rectangle([x0, 6, x0 + 2, 24], fill=STEEL_DARK)
    d.rectangle([x1 - 2, 6, x1, 24], fill=STEEL_DARK)
    d.rectangle([x0, 5, x1, 7], fill=STEEL_DARK)
    d.rectangle([x0, 23, x1, 25], fill=STEEL_DARK)


def draw_oil_press_manual(d, rng):
    press_frame(d)
    d.line([16, 2, 16, 9], fill=STEEL, width=2)  # screw
    d.line([11, 3, 21, 3], fill=WOOD_DARK, width=2)  # crank bar
    d.ellipse([20, 2, 22, 4], fill=WOOD_DARK)
    d.rectangle([12, 9, 20, 12], fill=STEEL)  # press plate
    d.rectangle([12, 12, 20, 18], fill=(196, 170, 110, 255))  # seed cake
    for y in (14, 16):
        d.line([12, y, 20, y], fill=(160, 134, 78, 255), width=1)
    d.line([16, 18, 16, 21], fill=BRASS, width=1)  # oil drip
    d.rectangle([13, 21, 19, 23], fill=STEEL_DARK)  # collection cup
    d.rectangle([10, 26, 22, 28], fill=BLACK)  # hotplate under it all
    d.point((13, 27), fill=FLAME)
    d.point((19, 27), fill=FLAME)


def draw_oil_press_primitive(d, rng):
    # wooden barrel under a big screw, kerosene cooker alongside below
    for sx in range(10, 23, 3):  # barrel staves
        d.rectangle([sx, 14, sx + 2, 26], fill=WOOD)
        d.line([sx, 14, sx, 26], fill=WOOD_DARK, width=1)
    for by in (17, 23):  # barrel hoops
        d.line([10, by, 22, by], fill=STEEL_DARK, width=1)
    d.rectangle([8, 5, 24, 7], fill=WOOD_DARK)  # top beam
    d.rectangle([9, 7, 11, 14], fill=WOOD_DARK)
    d.rectangle([21, 7, 23, 14], fill=WOOD_DARK)
    d.line([16, 1, 16, 11], fill=WOOD, width=3)  # the big screw
    d.line([10, 2, 22, 2], fill=WOOD_DARK, width=2)  # screw T-handle
    d.rectangle([12, 11, 20, 14], fill=WOOD)  # press plate
    d.line([22, 20, 26, 22], fill=BRASS, width=1)  # oil runnel
    d.rectangle([24, 22, 29, 27], fill=GREEN_ENAMEL)  # kerosene cooker
    d.point((26, 23), fill=FLAME)


def draw_screw_press(d, rng):
    # plain wooden screw press: frame, big screw, plate, cake
    d.rectangle([7, 8, 9, 27], fill=WOOD_DARK)
    d.rectangle([23, 8, 25, 27], fill=WOOD_DARK)
    d.rectangle([7, 6, 25, 9], fill=WOOD)
    d.rectangle([7, 25, 25, 27], fill=WOOD)
    d.line([16, 1, 16, 13], fill=WOOD, width=3)  # screw shaft
    for sy in range(3, 12, 2):  # screw thread
        d.line([14, sy, 18, sy + 1], fill=WOOD_DARK, width=1)
    d.line([9, 2, 23, 2], fill=WOOD_DARK, width=2)  # T-handle
    d.rectangle([11, 13, 21, 16], fill=WOOD)  # press plate
    d.rectangle([11, 16, 21, 22], fill=(186, 160, 104, 255))  # pulp cake
    d.line([11, 19, 21, 19], fill=(150, 126, 72, 255), width=1)
    d.line([16, 22, 16, 24], fill=BRASS, width=1)  # juice drip


def draw_pasta(d, rng):
    d.rectangle([8, 10, 22, 20], fill=(88, 88, 94, 255))  # iron body
    d.rectangle([8, 10, 22, 12], fill=(104, 104, 112, 255))
    d.rectangle([10, 5, 16, 10], fill=(104, 104, 112, 255))  # dough hopper
    d.ellipse([11, 4, 15, 6], fill=(238, 224, 178, 255))  # dough
    d.rectangle([22, 13, 25, 18], fill=STEEL_DARK)  # die head
    for sy in (15, 17, 19, 21):  # strands emerging and curling down
        d.line([25, sy - 2, 27, sy], fill=(240, 228, 168, 255), width=1)
        d.line([27, sy, 26, sy + 2], fill=(240, 228, 168, 255), width=1)
    d.line([12, 15, 6, 11], fill=STEEL_DARK, width=2)  # crank
    d.ellipse([4, 9, 7, 12], fill=WOOD_DARK)
    d.rectangle([9, 20, 21, 22], fill=(70, 70, 76, 255))  # base feet bar


def draw_sieve(d, rng, sid):
    if sid == "sieve_primitive":
        # woven wicker bowl: crosshatch of plant strips
        d.polygon([(6, 10), (26, 10), (22, 22), (10, 22)], fill=WICKER)
        for x in range(8, 26, 3):
            d.line([x, 10, x - 2, 22], fill=shade(WICKER, 0.72), width=1)
        for y in range(12, 22, 3):
            d.line([6, y, 26, y], fill=shade(WICKER, 1.14), width=1)
        d.line([5, 9, 27, 9], fill=WOOD_DARK, width=2)  # rim
        for gx in (12, 16, 20):  # sifted grains below
            d.point((gx, 25), fill=(226, 206, 150, 255))
    elif sid == "sieve_steel":
        # lab-grade: deep steel rim, fine regular mesh
        d.polygon([(6, 9), (26, 9), (24, 21), (8, 21)], fill=STEEL)
        d.polygon([(8, 11), (24, 11), (23, 19), (9, 19)],
                  fill=(168, 172, 178, 255))
        for x in range(10, 24, 2):
            d.line([x, 11, x - 1, 19], fill=STEEL_DARK, width=1)
        for y in range(12, 19, 2):
            d.line([8, y, 24, y], fill=STEEL_DARK, width=1)
        d.line([5, 8, 27, 8], fill=STEEL_LIGHT, width=2)  # rolled rim
        for gx in (13, 17, 20):  # fine sifted flour dust below
            d.point((gx, 24), fill=(240, 238, 230, 255))
    else:  # makeshift: mesh lashed to a bent thick-wire frame
        d.ellipse([7, 8, 25, 22], outline=STEEL_DARK, width=2)
        d.ellipse([9, 10, 23, 20], fill=(172, 176, 182, 255))
        for x in range(11, 23, 3):
            d.line([x, 10, x, 20], fill=STEEL, width=1)
        for y in range(12, 20, 3):
            d.line([9, y, 23, y], fill=STEEL, width=1)
        for lx, ly in ((8, 9), (24, 9), (8, 21), (24, 21)):  # lashings
            d.point((lx, ly), fill=BLACK)
        d.line([23, 15, 29, 18], fill=STEEL_DARK, width=2)  # wire handle
        d.line([29, 18, 30, 21], fill=STEEL_DARK, width=1)


def draw_snow_machine(d, rng):
    d.rectangle([8, 6, 24, 22], fill=PLASTIC_WHITE)  # body
    d.rectangle([8, 6, 24, 22], outline=STEEL, width=1)
    d.rectangle([11, 3, 20, 6], fill=STEEL)  # ice hopper lid
    d.rectangle([10, 9, 22, 13], fill=(188, 216, 232, 255))  # ice window
    for ix, iy in ((13, 11), (17, 10), (20, 12)):  # ice chunks
        d.point((ix, iy), fill=(240, 250, 252, 255))
    d.rectangle([13, 15, 19, 17], fill=STEEL_DARK)  # chute
    # snow-cone cup under the chute, heaped with shaved snow
    d.polygon([(13, 24), (19, 24), (17, 29), (15, 29)], fill=RED)
    d.polygon([(12, 20), (20, 20), (19, 24), (13, 24)],
              fill=(248, 250, 252, 255))
    d.ellipse([12, 18, 20, 22], fill=(248, 250, 252, 255))
    for sx, sy in ((11, 16), (21, 17), (16, 15)):  # falling snow flecks
        d.point((sx, sy), fill=(248, 250, 252, 255))
    d.rectangle([8, 22, 11, 24], fill=STEEL_DARK)
    d.rectangle([21, 22, 24, 24], fill=STEEL_DARK)


def draw_still(d, rng):
    # copper boiler, swan neck, coiled condenser, drip
    d.ellipse([8, 15, 22, 28], fill=COPPER)
    d.ellipse([8, 15, 22, 28], outline=shade(COPPER, 0.7))
    d.line([10, 20, 20, 20], fill=COPPER_LIGHT, width=1)
    d.rectangle([13, 11, 17, 15], fill=COPPER)  # cap
    d.rectangle([18, 6, 20, 8], outline=STEEL_DARK, width=1)  # thermometer
    d.point((19, 7), fill=RED)
    pts = [(17, 12), (21, 9), (25, 9), (27, 12)]  # swan neck / lyne arm
    d.line(pts, fill=COPPER, width=2)
    for cy in range(13, 23, 3):  # condenser coil in its jacket
        d.ellipse([24, cy, 30, cy + 3], outline=COPPER_LIGHT, width=1)
    d.rectangle([24, 12, 30, 24], outline=shade(COPPER, 0.7))
    d.line([27, 24, 27, 26], fill=(240, 220, 150, 255), width=1)  # drip
    d.rectangle([24, 26, 30, 29], fill=GLASS)  # collection jar
    d.rectangle([8, 27, 22, 29], fill=BLACK)  # burner base
    d.point((13, 28), fill=FLAME)
    d.point((18, 28), fill=FLAME)


def draw_sand_filter(d, rng):
    # plastic bucket: water over gravel over sand, slow drip below
    d.polygon([(9, 5), (23, 5), (22, 26), (10, 26)], fill=(120, 160, 205, 255))
    d.polygon([(9, 5), (23, 5), (22, 26), (10, 26)], outline=STEEL_DARK)
    d.rectangle([10, 6, 22, 9], fill=(96, 150, 205, 255))  # water layer
    d.rectangle([10, 10, 22, 14], fill=(150, 148, 142, 255))  # gravel
    for gx, gy in ((12, 11), (16, 12), (20, 11), (14, 13), (19, 13)):
        d.point((gx, gy), fill=(110, 108, 102, 255))
    d.rectangle([10, 15, 22, 24], fill=(216, 192, 130, 255))  # sand bed
    for _ in range(8):
        d.point((rng.randint(11, 21), rng.randint(16, 23)),
                fill=(190, 164, 100, 255))
    d.line([9, 5, 23, 5], fill=STEEL_LIGHT, width=1)  # rim
    d.point((16, 27), fill=(120, 170, 220, 255))  # filtered drip
    d.point((16, 29), fill=(120, 170, 220, 255))


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in ("acetylene_cooker", "biogas_cooker", "charcoal_cooker",
               "gasoline_cooker", "oil_cooker"):
        draw_cooker(d, rng, sid)
    elif sid == "hotplate_induction":
        draw_hotplate(d, rng)
    elif sid in ("eggs_ferment_active", "pickles_ferment_active",
                 "sauerkraut_ferment_active", "flask_yeast_active"):
        draw_ferment(d, rng, sid)
    elif sid == "can_sealer":
        draw_can_sealer(d, rng)
    elif sid == "carver_on":
        draw_carver(d, rng)
    elif sid == "char_purifier":
        draw_char_purifier(d, rng)
    elif sid == "crystallizing_sugar_active":
        draw_sugar(d, rng)
    elif sid == "makeshift_pressure_cooker":
        draw_pressure_cooker(d, rng)
    elif sid == "makeshift_sealer":
        draw_makeshift_sealer(d, rng)
    elif sid == "metal_honing_rod":
        draw_honing_rod(d, rng)
    elif sid == "oil_press_manual":
        draw_oil_press_manual(d, rng)
    elif sid == "oil_press_primitive":
        draw_oil_press_primitive(d, rng)
    elif sid == "screw_press":
        draw_screw_press(d, rng)
    elif sid == "pastaextruder":
        draw_pasta(d, rng)
    elif sid in ("sieve_primitive", "sieve_steel", "sieve_steel_makeshift"):
        draw_sieve(d, rng, sid)
    elif sid == "snow_machine_home":
        draw_snow_machine(d, rng)
    elif sid == "still":
        draw_still(d, rng)
    elif sid == "water_filter_sand":
        draw_sand_filter(d, rng)
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
    # contact-sheet preview: 6 cols x 5 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item54.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item54.png {prev.size}")


if __name__ == "__main__":
    main()
