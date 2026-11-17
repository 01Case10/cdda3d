#!/usr/bin/env python3
"""ITEM68: drinks, lights and shotgun magazines -- CDDA Voxel
coverage audit.  Mixed tail batch, three families:

  Drinks (14) -- each drawn in its own vessel with its own liquid:
    atomic_coffee (dark mug + caffeine pills), ayran (frothy white
    yogurt drink), coffee_sweetened (milky coffee + sugar cubes),
    drink_virgin_mary (red cocktail, celery stick), sparkling /
    sparkling_peach / sparkling_pomegranate (bottled fizzy water,
    plain / peach / ruby), sweet_milk (glass of milk),
    sweet_milk_fortified (milk carton), tomato_juice (glass, tomato
    wedge), water_clean_scarlet (acrid yellowish water),
    water_mineral (fancy tall bottle), water_murky (sediment jar),
    water_spring (mountain-label bottle).
  Lights (14) -- candles thick/thin, lit/unlit; diving flashlights
    small and variable-power (beam width by level); standard and
    heavy flashlights on; magnesium hand flare vs signal flare;
    coiled inactive LED strip; steel menorah with nine flames.
  Shotgun magazines (14) -- dark box magazines whose length, curve
    and build follow capacity and material (M26-MASS 3/5, MKA 1919
    5/10, Mossberg 590M 5/10/15/20, Saiga 10, Savage 220 2-round),
    speedloader tubes (6/8), a cloth 20-shell belt, and the SRM
    1216 rotary drum.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "atomic_coffee",
    "ayran",
    "coffee_sweetened",
    "drink_virgin_mary",
    "sparkling",
    "sparkling_peach",
    "sparkling_pomegranate",
    "sweet_milk",
    "sweet_milk_fortified",
    "tomato_juice",
    "water_clean_scarlet",
    "water_mineral",
    "water_murky",
    "water_spring",
    "candle_lit",
    "candle_small",
    "candle_small_lit",
    "diving_flashlight_small_hipower_on",
    "diving_flashlight_small_on",
    "diving_flashlight_variable_on_hi",
    "diving_flashlight_variable_on_low",
    "diving_flashlight_variable_on_med",
    "flashlight_on",
    "handflare_lit",
    "heavy_flashlight_on",
    "lightstrip_inactive",
    "menorah",
    "sigflare_lit",
    "m26_mass_mag_3",
    "m26_mass_mag_5",
    "mka_1919_10",
    "mka_1919_5",
    "mossberg_590m_mag_10",
    "mossberg_590m_mag_15",
    "mossberg_590m_mag_20",
    "mossberg_590m_mag_5",
    "saiga10mag",
    "savage_220_magazine",
    "shot_speedloader6",
    "shot_speedloader8",
    "shotbelt_20",
    "srm_1216_mag_16",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

GLASS = (168, 196, 204, 255)
GLASS_DARK = (110, 138, 148, 255)
WHITE = (246, 246, 242, 255)
CREAM = (240, 230, 204, 255)
INK = (44, 44, 50, 255)
STEEL = (140, 144, 150, 255)
STEEL_DARK = (84, 88, 96, 255)
POLY = (52, 54, 60, 255)        # dark-gray magazine polymer
POLY_DARK = (34, 36, 42, 255)
BRASS = (198, 158, 62, 255)
SHELL_RED = (178, 44, 38, 255)
FLAME = (250, 176, 52, 255)
FLAME_CORE = (254, 240, 180, 255)
BEAM = (252, 244, 190, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ drinks

def mug(d, rng, liquid, dark=False):
    body = (120, 66, 44, 255) if dark else (178, 84, 52, 255)
    d.rectangle([9, 12, 22, 27], fill=body)
    d.rectangle([9, 12, 22, 27], outline=shade(body, 0.7), width=1)
    d.arc([19, 15, 27, 24], start=270, end=90, fill=shade(body, 0.8),
          width=2)  # handle
    d.ellipse([10, 11, 21, 14], fill=liquid)  # surface
    for sx in (12, 17):  # steam curls
        d.point((sx, 8), fill=(200, 200, 200, 255))
        d.point((sx + 1, 6), fill=(200, 200, 200, 255))
        d.point((sx, 4), fill=(216, 216, 216, 255))


def glass(d, rng, liquid, top=9, froth=False):
    d.polygon([(10, top), (22, top), (21, 27), (11, 27)], fill=GLASS)
    d.polygon([(11, top + 2), (21, top + 2), (20, 26), (12, 26)],
              fill=liquid)
    d.polygon([(10, top), (22, top), (21, 27), (11, 27)], outline=GLASS_DARK)
    if froth:
        d.rectangle([11, top + 1, 21, top + 3], fill=WHITE)
        for bx in range(12, 21, 2):
            d.point((bx, top), fill=WHITE)


def bottle(d, rng, liquid, label, tall=False):
    y0 = 3 if tall else 6
    d.rectangle([14, y0, 18, y0 + 4], fill=GLASS_DARK)  # cap/neck
    d.polygon([(14, y0 + 4), (18, y0 + 4), (21, y0 + 8), (21, 28),
               (11, 28), (11, y0 + 8)], fill=GLASS)
    d.polygon([(12, y0 + 9), (20, y0 + 9), (20, 27), (12, 27)], fill=liquid)
    d.polygon([(14, y0 + 4), (18, y0 + 4), (21, y0 + 8), (21, 28),
               (11, 28), (11, y0 + 8)], outline=GLASS_DARK)
    d.rectangle([12, 18, 20, 24], fill=label)
    d.rectangle([12, 18, 20, 24], outline=GLASS_DARK, width=1)


def bubbles(d, rng, x0, x1, y0, y1, col=(240, 250, 252, 255)):
    for _ in range(10):
        d.point((rng.randint(x0, x1), rng.randint(y0, y1)), fill=col)


def draw_drink(d, rng, sid):
    if sid == "atomic_coffee":
        mug(d, rng, (26, 16, 10, 255), dark=True)  # extra-dark coffee
        # caffeine pills beside the mug
        d.ellipse([24, 24, 27, 27], fill=WHITE)
        d.ellipse([24, 24, 27, 27], outline=INK)
        d.ellipse([26, 20, 29, 23], fill=WHITE)
        d.ellipse([26, 20, 29, 23], outline=INK)
        d.point((28, 8), fill=(250, 210, 70, 255))  # atomic sparkle
        d.point((27, 9), fill=(250, 210, 70, 255))
        d.point((29, 9), fill=(250, 210, 70, 255))
        d.point((28, 10), fill=(250, 210, 70, 255))
    elif sid == "coffee_sweetened":
        mug(d, rng, (122, 78, 46, 255))  # milky sweet coffee
        for cx, cy in ((25, 25), (28, 22)):  # sugar cubes
            d.rectangle([cx, cy, cx + 3, cy + 3], fill=WHITE)
            d.rectangle([cx, cy, cx + 3, cy + 3], outline=GLASS_DARK,
                        width=1)
    elif sid == "ayran":
        glass(d, rng, (238, 238, 230, 255), froth=True)  # white yogurt
        d.point((19, 6), fill=(70, 140, 70, 255))  # mint leaf
        d.point((20, 5), fill=(70, 140, 70, 255))
        d.point((20, 7), fill=(56, 118, 56, 255))
    elif sid == "drink_virgin_mary":
        glass(d, rng, (186, 40, 32, 255), top=10)
        d.rectangle([19, 2, 21, 12], fill=(96, 158, 64, 255))  # celery
        d.rectangle([19, 2, 21, 12], outline=(64, 110, 40, 255), width=1)
        for _ in range(5):  # spice specks
            d.point((rng.randint(12, 19), rng.randint(14, 24)),
                    fill=(90, 30, 20, 255))
    elif sid == "tomato_juice":
        glass(d, rng, (196, 48, 36, 255), top=10)
        d.polygon([(20, 6), (25, 8), (21, 11)], fill=(214, 70, 52, 255))
        d.polygon([(20, 6), (25, 8), (21, 11)], outline=(150, 40, 30, 255))
        d.point((22, 8), fill=(240, 150, 130, 255))  # seeds
    elif sid == "sweet_milk":
        glass(d, rng, (250, 248, 240, 255), top=9)
        for sx in (13, 18):  # sugar sparkle in the milk
            d.point((sx, 16 + sx % 5), fill=(226, 214, 170, 255))
    elif sid == "sweet_milk_fortified":
        # gable-top milk carton, fortified cross on the label
        d.polygon([(11, 10), (21, 10), (18, 5), (14, 5)], fill=CREAM)
        d.rectangle([11, 10, 21, 28], fill=WHITE)
        d.rectangle([11, 10, 21, 28], outline=GLASS_DARK, width=1)
        d.line([11, 10, 21, 10], fill=GLASS_DARK, width=1)
        d.rectangle([15, 16, 17, 22], fill=(196, 48, 40, 255))
        d.rectangle([14, 18, 18, 20], fill=(196, 48, 40, 255))
    elif sid in ("sparkling", "sparkling_peach", "sparkling_pomegranate"):
        if sid == "sparkling":
            bottle(d, rng, (196, 232, 238, 255), (120, 180, 200, 255))
            bubbles(d, rng, 13, 19, 12, 26)
        elif sid == "sparkling_peach":
            bottle(d, rng, (240, 206, 160, 255), (244, 168, 110, 255))
            bubbles(d, rng, 13, 19, 12, 26)
            d.ellipse([14, 19, 18, 23], outline=(226, 130, 70, 255))
            d.line([16, 19, 16, 23], fill=(226, 130, 70, 255), width=1)
        else:
            bottle(d, rng, (216, 120, 130, 255), (168, 44, 60, 255))
            bubbles(d, rng, 13, 19, 12, 26)
            for sx, sy in ((14, 21), (17, 20), (16, 23)):  # seeds
                d.point((sx, sy), fill=(140, 26, 40, 255))
    elif sid == "water_clean_scarlet":
        # acrid water: yellowish, faint warning swirl on the label
        bottle(d, rng, (228, 208, 110, 255), (240, 226, 150, 255))
        d.polygon([(16, 19), (14, 23), (18, 23)], outline=(178, 44, 38, 255))
        d.point((16, 22), fill=(178, 44, 38, 255))
    elif sid == "water_mineral":
        bottle(d, rng, (200, 226, 240, 255), (70, 110, 170, 255),
               tall=True)  # fancy tall bottle, blue label
        d.line([13, 20, 19, 20], fill=WHITE, width=1)
        d.line([14, 22, 18, 22], fill=WHITE, width=1)
    elif sid == "water_murky":
        # squat jar of brown-green water with sediment and specks
        d.rectangle([10, 10, 22, 28], fill=GLASS)
        d.rectangle([11, 13, 21, 27], fill=(128, 118, 74, 255))
        d.rectangle([11, 24, 21, 27], fill=(94, 82, 52, 255))  # sediment
        d.rectangle([10, 10, 22, 28], outline=GLASS_DARK, width=1)
        d.rectangle([12, 8, 20, 10], fill=STEEL_DARK)  # lid
        for _ in range(7):
            d.point((rng.randint(12, 20), rng.randint(14, 23)),
                    fill=(160, 150, 100, 255))
    elif sid == "water_spring":
        bottle(d, rng, (186, 220, 242, 255), (240, 246, 248, 255))
        # mountain motif on the label
        d.polygon([(13, 23), (16, 19), (18, 22), (19, 20), (20, 23)],
                  fill=(90, 140, 190, 255))


# ------------------------------------------------------------------ lights

def flame(d, cx, cy, s=1):
    d.ellipse([cx - 1 - s, cy - 3 * s, cx + 1 + s, cy + s], fill=FLAME)
    d.ellipse([cx - s, cy - 2 * s, cx + s, cy], fill=FLAME_CORE)
    d.point((cx, cy - 4 * s), fill=FLAME)


def candle(d, rng, thick, lit):
    w = 5 if thick else 2
    top = 12 if thick else 10
    d.rectangle([16 - w, top, 16 + w, 28],
                fill=(238, 232, 214, 255) if thick else (226, 232, 240, 255))
    d.rectangle([16 - w, top, 16 + w, 28], outline=(178, 172, 156, 255),
                width=1)
    d.line([16, top, 16, top - 2], fill=INK, width=1)  # wick
    if thick:  # wax drips
        d.rectangle([16 - w, top, 16 - w + 1, top + 4],
                    fill=(246, 240, 224, 255))
        d.rectangle([16 + w - 1, top, 16 + w, top + 2],
                    fill=(246, 240, 224, 255))
    d.rectangle([10, 28, 22, 29], fill=STEEL_DARK)  # holder dish
    if lit:
        flame(d, 16, top - 4, s=1 if thick else 0)


def flashlight(d, rng, body_len, head_w, beam_w, beam_len, accent=None):
    """Horizontal flashlight, head at right, beam fanning right."""
    x0 = 4
    x1 = x0 + body_len
    d.rectangle([x0, 14, x1, 18], fill=STEEL)
    d.rectangle([x0, 14, x1, 18], outline=STEEL_DARK, width=1)
    d.rectangle([x0, 13, x0 + 2, 19], fill=STEEL_DARK)  # tail cap
    if accent:
        d.rectangle([x0 + 3, 14, x0 + 5, 18], fill=accent)  # grip ring
    hy0, hy1 = 16 - head_w // 2, 16 + head_w // 2
    d.rectangle([x1, hy0, x1 + 3, hy1], fill=STEEL_DARK)
    d.rectangle([x1 + 1, hy0 + 1, x1 + 3, hy1 - 1], fill=BEAM)
    bx = x1 + 4
    d.polygon([(bx, 16 - beam_w // 4), (bx + beam_len, 16 - beam_w // 2),
               (bx + beam_len, 16 + beam_w // 2), (bx, 16 + beam_w // 4)],
              fill=(250, 240, 170, 255))


def draw_light(d, rng, sid):
    if sid == "candle_lit":
        candle(d, rng, thick=True, lit=True)
    elif sid == "candle_small":
        candle(d, rng, thick=False, lit=False)
    elif sid == "candle_small_lit":
        candle(d, rng, thick=False, lit=True)
    elif sid == "flashlight_on":
        flashlight(d, rng, 12, 6, 8, 9)
    elif sid == "heavy_flashlight_on":
        flashlight(d, rng, 14, 10, 14, 10)
        d.rectangle([6, 19, 12, 21], fill=STEEL_DARK)  # battery bulge
    elif sid == "diving_flashlight_small_on":
        flashlight(d, rng, 9, 6, 6, 7, accent=(52, 110, 170, 255))
    elif sid == "diving_flashlight_small_hipower_on":
        flashlight(d, rng, 9, 8, 14, 11, accent=(52, 110, 170, 255))
        d.point((24, 8), fill=BEAM)  # extra glare sparks
        d.point((27, 22), fill=BEAM)
    elif sid.startswith("diving_flashlight_variable"):
        level = sid.rsplit("_", 1)[1]  # low / med / hi
        beam_w, beam_len = {"low": (4, 5), "med": (8, 8),
                            "hi": (13, 11)}[level]
        flashlight(d, rng, 13, 8, beam_w, beam_len,
                   accent=(52, 110, 170, 255))
        # power dial on the body: marker position by level
        d.rectangle([8, 12, 14, 13], fill=STEEL_DARK)
        mx = {"low": 9, "med": 11, "hi": 13}[level]
        d.point((mx, 12), fill=(250, 210, 70, 255))
    elif sid == "handflare_lit":
        # chunky magnesium flare, orange flame, dripping sparks
        d.rectangle([13, 12, 18, 29], fill=(150, 138, 120, 255))
        d.rectangle([13, 12, 18, 29], outline=(104, 94, 78, 255), width=1)
        d.rectangle([13, 12, 18, 15], fill=(120, 60, 40, 255))
        flame(d, 15, 8, s=1)
        d.point((15, 3), fill=(250, 120, 40, 255))
        for sx, sy in ((10, 16), (21, 19), (11, 23), (20, 26)):
            d.point((sx, sy), fill=(250, 160, 60, 255))  # sparks
    elif sid == "sigflare_lit":
        # slim red signal flare, white-hot flame, smoke trail
        d.rectangle([14, 13, 17, 29], fill=(178, 40, 34, 255))
        d.rectangle([14, 13, 17, 29], outline=(120, 26, 22, 255), width=1)
        d.rectangle([14, 20, 17, 22], fill=WHITE)
        d.ellipse([13, 5, 18, 13], fill=(254, 246, 200, 255))
        d.ellipse([14, 7, 17, 12], fill=WHITE)
        for sx, sy in ((11, 4), (20, 3), (22, 6)):  # smoke puffs
            d.point((sx, sy), fill=(170, 170, 170, 255))
    elif sid == "lightstrip_inactive":
        # coiled roll of LED strip, coloured but unlit diodes
        for r, col in ((11, (120, 124, 130, 255)), (8, (104, 108, 116, 255)),
                       (5, (88, 92, 100, 255))):
            d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=col, width=2)
        cols = [(200, 90, 90, 255), (90, 170, 90, 255), (90, 120, 200, 255),
                (200, 180, 90, 255)]
        for i in range(12):  # dim diode dots around the outer coil
            import math
            a = i * 2 * math.pi / 12
            x = int(16 + 11 * math.cos(a))
            y = int(16 + 11 * math.sin(a))
            d.point((x, y), fill=shade(cols[i % 4], 0.55))
        d.line([24, 24, 28, 28], fill=STEEL_DARK, width=1)  # loose end
        d.point((28, 28), fill=(150, 60, 60, 255))
        d.point((29, 29), fill=(60, 60, 150, 255))
    elif sid == "menorah":
        # steel hanukkiah: centre stem, 4 arms each side, nine flames
        d.line([16, 12, 16, 27], fill=STEEL, width=2)
        d.rectangle([11, 27, 21, 28], fill=STEEL_DARK)  # base
        for k in range(1, 5):
            for sgn in (-1, 1):
                x = 16 + sgn * k * 3
                d.line([16, 20, x, 20 - k], fill=STEEL, width=1)
                d.line([x, 20 - k, x, 13], fill=STEEL, width=1)
                d.rectangle([x - 1, 12, x + 1, 13], fill=STEEL_DARK)
                flame(d, x, 10, s=0)
        d.rectangle([15, 10, 17, 12], fill=STEEL_DARK)  # shamash cup
        flame(d, 16, 8, s=0)


# --------------------------------------------------------------- magazines

def box_mag(d, rng, length, curve=0, steel_top=False, wide=False):
    """Vertical box magazine, feed lips at top; length in px of body."""
    x0, x1 = (12, 20) if wide else (13, 19)
    y0 = 6
    y1 = y0 + length
    d.rectangle([x0, y0, x1, y0 + 3], fill=STEEL_DARK)  # feed lips
    d.point((x0 + 1, y0 + 1), fill=BRASS)  # top shell glimpse
    d.point((x1 - 1, y0 + 1), fill=BRASS)
    body = POLY
    if curve:
        for i, yy in enumerate(range(y0 + 3, y1, 3)):
            off = int(curve * ((yy - y0) / max(1, length)) ** 2 * 3)
            d.rectangle([x0 + off, yy, x1 + off, min(yy + 3, y1)], fill=body)
            d.rectangle([x0 + off, yy, x1 + off, min(yy + 3, y1)],
                        outline=POLY_DARK, width=1)
        bend = int(curve * 3)
    else:
        d.rectangle([x0, y0 + 3, x1, y1], fill=body)
        d.rectangle([x0, y0 + 3, x1, y1], outline=POLY_DARK, width=1)
        bend = 0
    if steel_top:  # steel upper section (mka 5: lc_steel + plastic)
        d.rectangle([x0, y0 + 3, x1, y0 + 8], fill=STEEL)
        d.rectangle([x0, y0 + 3, x1, y0 + 8], outline=STEEL_DARK, width=1)
    # rib lines down the body
    for yy in range(y0 + 6, y1 - 1, 4):
        off = int(curve * ((yy - y0) / max(1, length)) ** 2 * 3) if curve else 0
        d.line([x0 + off + 1, yy, x1 + off - 1, yy], fill=POLY_DARK, width=1)
    d.rectangle([x0 + bend - 1, y1, x1 + bend + 1, y1 + 2],
                fill=POLY_DARK)  # baseplate


def draw_mag(d, rng, sid):
    if sid == "m26_mass_mag_3":
        box_mag(d, rng, 9)  # stubby 3-round
    elif sid == "m26_mass_mag_5":
        box_mag(d, rng, 13)
    elif sid == "mka_1919_5":
        box_mag(d, rng, 12, steel_top=True)
    elif sid == "mka_1919_10":
        box_mag(d, rng, 19, curve=1)
    elif sid == "mossberg_590m_mag_5":
        box_mag(d, rng, 11)
        d.rectangle([14, 12, 18, 14], outline=STEEL, width=1)  # window
    elif sid == "mossberg_590m_mag_10":
        box_mag(d, rng, 16)
        d.rectangle([14, 12, 18, 18], outline=STEEL, width=1)
        d.point((16, 14), fill=SHELL_RED)
        d.point((16, 16), fill=SHELL_RED)
    elif sid == "mossberg_590m_mag_15":
        box_mag(d, rng, 20)
        d.rectangle([14, 12, 18, 22], outline=STEEL, width=1)
        for yy in (14, 17, 20):
            d.point((16, yy), fill=SHELL_RED)
    elif sid == "mossberg_590m_mag_20":
        box_mag(d, rng, 22, wide=True)  # fat double-stack 20
        d.rectangle([14, 12, 18, 24], outline=STEEL, width=1)
        for yy in (14, 17, 20, 23):
            d.point((16, yy), fill=SHELL_RED)
    elif sid == "saiga10mag":
        box_mag(d, rng, 18, curve=2)  # strong AK-style curve, all plastic
    elif sid == "savage_220_magazine":
        box_mag(d, rng, 7)  # tiny 2-round 20ga
        d.point((16, 10), fill=BRASS)
    elif sid in ("shot_speedloader6", "shot_speedloader8"):
        n = 6 if sid.endswith("6") else 8
        ln = 17 if n == 6 else 21
        d.rectangle([13, 5, 18, 5 + ln], fill=(196, 198, 202, 255))
        d.rectangle([13, 5, 18, 5 + ln], outline=(140, 142, 148, 255),
                    width=1)
        d.rectangle([11, 9, 20, 12], fill=(150, 152, 158, 255))  # handle
        d.rectangle([11, 9, 20, 12], outline=(110, 112, 118, 255), width=1)
        for i in range(n):  # shell rims visible along the tube
            yy = 7 + i * (ln - 3) // n
            d.line([14, yy, 17, yy], fill=SHELL_RED, width=1)
            d.point((15, yy), fill=BRASS)
    elif sid == "shotbelt_20":
        # cloth belt draped in a shallow U, shells in loops
        d.rectangle([4, 10, 28, 14], fill=(168, 158, 138, 255))
        d.rectangle([4, 22, 28, 26], fill=(168, 158, 138, 255))
        d.rectangle([4, 10, 6, 26], fill=(168, 158, 138, 255))
        d.rectangle([26, 10, 28, 26], fill=(168, 158, 138, 255))
        d.rectangle([4, 10, 28, 26], outline=(120, 110, 90, 255), width=1)
        for i in range(10):  # top row of shells
            x = 5 + i * 2 + (1 if i % 2 else 0)
            d.rectangle([x, 7, x + 1, 10], fill=SHELL_RED)
            d.point((x, 6), fill=BRASS)
        for i in range(10):  # bottom row
            x = 5 + i * 2 + (1 if i % 2 else 0)
            d.rectangle([x, 26, x + 1, 29], fill=SHELL_RED)
            d.point((x, 30), fill=BRASS)
    elif sid == "srm_1216_mag_16":
        # rotary drum: central hub with 4 tube clusters around it
        d.ellipse([7, 7, 25, 25], fill=STEEL)
        d.ellipse([7, 7, 25, 25], outline=STEEL_DARK, width=1)
        for cx, cy in ((11, 11), (21, 11), (11, 21), (21, 21)):
            d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=POLY)
            d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3],
                      outline=POLY_DARK, width=1)
            d.point((cx, cy), fill=SHELL_RED)
        d.ellipse([14, 14, 18, 18], fill=STEEL_DARK)
        d.rectangle([14, 3, 18, 7], fill=STEEL_DARK)  # feed neck


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in IDS[:14]:
        draw_drink(d, rng, sid)
    elif sid in IDS[14:28]:
        draw_light(d, rng, sid)
    else:
        draw_mag(d, rng, sid)
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
    # contact-sheet preview: 7 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item68.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item68.png {prev.size}")


if __name__ == "__main__":
    main()
