#!/usr/bin/env python3
"""ITEM69: nuts, wood materials, paints, HESCO cell + plastic misc --
CDDA Voxel coverage audit.

Families (ids embedded below, verbatim from the batch file):

  Nuts (13) -- handfuls drawn as small heaps, shape/colour per
    species: capped acorns (roasted dark / blanched pale and capless),
    elongated butternuts (roasted / raw shelled), green edamame beans,
    round seamed hickory nuts, peanut kernels, round soy nuts, and
    the processed forms: gritty pulp mounds (almond light, hazelnut
    mid, walnut dark/coarse), a jar of peanut butter spread, and a
    pale vegan-butter stick.
  Wood (13) -- firewood (split log end), giant wooden screw, long
    pole (and its oil-soaked, glossier twin), long/short 2x4 planks,
    a bent stick, fine-grained tonewood block, small tuning pegs, a
    small wood block, and 4x4 posts in long/standard/short lengths.
  HESCO MIL 3 cell -- flat-packed wire-mesh panel with a tan
    geotextile liner band.
  Paints (12) -- one can form; the paint colour is the open surface,
    the lid band and a drip down the can front.
  Plastic misc (12) -- 10 L / 5 L solid blocks, epoxy / polycarbonate
    / thermoplastic-resin chunks (glossy / clear / matte), a 40 mm
    filter port, an inhalation valve, Lycra and nylon fabric patches,
    a panoramic lens strip, a small flexible plastic sheet and a
    large rigid polycarbonate sheet.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "acorn_roasted",
    "acorns_blanched",
    "almond_pulp",
    "butternut_roasted",
    "butternut_shelled",
    "edamame_roasted",
    "hazelnut_pulp",
    "hickory_nut_roasted",
    "peanut_shelled",
    "soy_nuts",
    "spread_peanutbutter",
    "vegan_butter",
    "walnut_pulp",
    "firewood",
    "giant_wood_screw",
    "long_pole",
    "plank_long",
    "plank_short",
    "soaking_long_pole",
    "stick_common",
    "tonewood",
    "tuning_pegs",
    "wood_block",
    "wooden_post",
    "wooden_post_long",
    "wooden_post_short",
    "hesco_mil3_cell",
    "paint_black",
    "paint_blue",
    "paint_brown",
    "paint_cyan",
    "paint_gray",
    "paint_green",
    "paint_orange",
    "paint_pink",
    "paint_purple",
    "paint_red",
    "paint_white",
    "paint_yellow",
    "10l_plastic",
    "5l_plastic",
    "epoxy_chunk",
    "filter_port_40mm",
    "inhalation_valve",
    "lycra_patch",
    "nylon",
    "panoramic_eyepiece",
    "plastic_sheet_small",
    "polycarbonate_chunk",
    "rigid_plastic_sheet",
    "thermo_resin_chunk",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (46, 38, 30, 255)
WOOD = (150, 108, 62, 255)
WOOD_DARK = (110, 76, 40, 255)
WOOD_LIGHT = (186, 148, 96, 255)
BARK = (96, 70, 44, 255)
ENDGRAIN = (206, 172, 118, 255)
METAL = (168, 172, 178, 255)
METAL_DARK = (110, 114, 120, 255)
PLASTIC_BLUE = (150, 196, 228, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# --------------------------------------------------------------------- nuts

def heap_pos(rng, n, cx=16, cy=19):
    """Heap layout: rows of nuts, seeded jitter."""
    rows = [(4, cy + 5), (3, cy + 1), (2, cy - 3), (1, cy - 7)]
    pts = []
    for count, y in rows:
        for i in range(count):
            x = cx + (i - (count - 1) / 2) * 6
            pts.append((x + rng.randint(-1, 1), y + rng.randint(-1, 1)))
            if len(pts) >= n:
                return pts
    return pts


def nut_acorn(d, x, y, body, cap):
    d.ellipse([x - 3, y - 2, x + 3, y + 4], fill=body)          # body
    d.polygon([(x - 3, y + 3), (x + 3, y + 3), (x, y + 6)], fill=body)
    d.pieslice([x - 4, y - 5, x + 4, y + 2], 180, 360, fill=cap)  # cupule
    d.point((x, y - 5), fill=cap)                                # stem nub
    d.point((x - 1, y), fill=shade(body, 1.25))                  # highlight


def nut_oval(d, x, y, col, w=3, h=5, seam=False):
    d.ellipse([x - w, y - h + 2, x + w, y + h - 1], fill=col)
    d.polygon([(x - w, y + h - 3), (x + w, y + h - 3), (x, y + h + 1)],
              fill=col)
    if seam:
        d.line([x, y - h + 3, x, y + h - 1], fill=shade(col, 0.7), width=1)
    d.point((x - 1, y - 1), fill=shade(col, 1.3))


def nut_round(d, x, y, col, r=3, seam=False):
    d.ellipse([x - r, y - r, x + r, y + r], fill=col)
    if seam:  # husk segments
        d.arc([x - r, y - r, x + r, y + r], 90, 200, fill=shade(col, 0.65))
        d.arc([x - r, y - r, x + r, y + r], 270, 20, fill=shade(col, 0.65))
        d.line([x, y - r, x, y + r], fill=shade(col, 0.65), width=1)
    d.point((x - 1, y - 1), fill=shade(col, 1.3))


def nut_peanut(d, x, y, col):
    d.ellipse([x - 2, y - 4, x + 2, y], fill=col)      # top lobe
    d.ellipse([x - 2, y - 1, x + 2, y + 4], fill=col)  # bottom lobe
    d.point((x - 1, y - 2), fill=shade(col, 1.25))


def nut_bean(d, x, y, col):
    d.ellipse([x - 3, y - 2, x + 3, y + 2], fill=col)
    d.point((x - 1, y), fill=shade(col, 1.3))
    d.point((x + 2, y), fill=shade(col, 0.75))  # hilum


def pulp_pile(d, rng, col, coarse=False):
    d.polygon([(6, 26), (11, 15), (16, 12), (22, 15), (26, 26)], fill=col)
    n = 46 if coarse else 70
    for _ in range(n):
        x = rng.randint(8, 24)
        y = rng.randint(14, 25)
        if y > 26 - abs(x - 16) // 2:
            continue
        c = shade(col, rng.choice((0.8, 0.9, 1.1, 1.22)))
        d.point((x, y), fill=c)
        if coarse and rng.random() < 0.4:
            d.point((x + 1, y), fill=c)


def draw_nut(d, rng, sid):
    if sid == "acorn_roasted":
        for x, y in heap_pos(rng, 8):
            nut_acorn(d, x, y, (122, 78, 38, 255), (84, 56, 30, 255))
    elif sid == "acorns_blanched":
        for x, y in heap_pos(rng, 8):  # capless, pale and mushy
            nut_oval(d, x, y, (206, 178, 138, 255), w=3, h=4)
    elif sid == "butternut_roasted":
        for x, y in heap_pos(rng, 7):
            nut_oval(d, x, y, (146, 96, 44, 255), seam=True)
    elif sid == "butternut_shelled":
        for x, y in heap_pos(rng, 7):
            nut_oval(d, x, y, (188, 144, 92, 255), seam=True)
    elif sid == "edamame_roasted":
        for x, y in heap_pos(rng, 8):
            nut_bean(d, x, y, (96, 148, 62, 255))
        # one pod, split open, beans inside
        d.ellipse([8, 22, 24, 27], fill=(74, 122, 48, 255))
        for bx in (12, 16, 20):
            d.ellipse([bx - 2, 22, bx + 2, 26], fill=(116, 168, 80, 255))
    elif sid == "hickory_nut_roasted":
        for x, y in heap_pos(rng, 7):
            nut_round(d, x, y, (104, 66, 34, 255), seam=True)
    elif sid == "peanut_shelled":
        for x, y in heap_pos(rng, 8):
            nut_peanut(d, x, y, (196, 148, 104, 255))
    elif sid == "soy_nuts":
        for x, y in heap_pos(rng, 9):
            nut_round(d, x, y, (186, 152, 102, 255), r=2)
    elif sid == "almond_pulp":
        pulp_pile(d, rng, (196, 168, 128, 255))
    elif sid == "hazelnut_pulp":
        pulp_pile(d, rng, (164, 122, 82, 255))
    elif sid == "walnut_pulp":
        pulp_pile(d, rng, (128, 92, 58, 255), coarse=True)
    elif sid == "spread_peanutbutter":
        # open glass jar of smooth spread, lid leaning beside it
        d.rectangle([10, 10, 23, 27], fill=(196, 214, 220, 255))
        d.rectangle([11, 13, 22, 26], fill=(188, 138, 84, 255))
        d.ellipse([11, 11, 22, 15], fill=(206, 158, 100, 255))  # surface
        d.arc([12, 11, 21, 15], 200, 320, fill=(170, 122, 70, 255))
        d.rectangle([10, 10, 23, 12], outline=METAL_DARK, width=1)
        d.ellipse([24, 21, 29, 27], fill=(150, 60, 50, 255))    # lid
        d.ellipse([25, 22, 28, 26], outline=shade((150, 60, 50, 255), 0.7))
    elif sid == "vegan_butter":
        # pale butter stick on its wrapper
        d.polygon([(6, 24), (26, 24), (28, 27), (4, 27)],
                  fill=(222, 224, 226, 255))
        d.rectangle([8, 15, 25, 24], fill=(216, 188, 148, 255))
        d.polygon([(8, 15), (11, 12), (28, 12), (25, 15)],
                  fill=(230, 204, 168, 255))
        d.polygon([(25, 15), (28, 12), (28, 21), (25, 24)],
                  fill=(198, 168, 128, 255))
        d.line([10, 17, 23, 17], fill=(238, 220, 188, 255), width=1)


# -------------------------------------------------------------------- wood

def grain_h(d, x0, x1, y, col=WOOD_DARK):
    d.line([x0, y, x1, y], fill=col, width=1)


def draw_wood(d, rng, sid):
    if sid == "firewood":
        d.rectangle([8, 11, 28, 21], fill=BARK)                # bark side
        for x in range(10, 28, 4):
            d.line([x, 12, x - 1, 20], fill=shade(BARK, 0.75), width=1)
        d.ellipse([3, 9, 12, 23], fill=ENDGRAIN)               # split end
        for r in (3, 2, 1):
            d.ellipse([6 - r + 1, 13 - r, 6 + r + 1, 13 + r + 3],
                      outline=WOOD_LIGHT, width=1)
        d.line([4, 12, 10, 19], fill=WOOD_DARK, width=1)       # split crack
        d.line([28, 11, 28, 21], fill=shade(BARK, 0.7), width=1)
    elif sid == "giant_wood_screw":
        d.polygon([(8, 4), (15, 4), (13, 9), (10, 9)], fill=WOOD_DARK)  # head
        d.polygon([(10, 9), (13, 9), (14, 24), (12, 28), (10, 24)],
                  fill=WOOD)                                            # shaft
        for i, y in enumerate(range(11, 24, 3)):                        # thread
            d.line([9 - (i % 2), y, 15, y + 2], fill=WOOD_DARK, width=2)
        d.line([11, 10, 11, 24], fill=WOOD_LIGHT, width=1)
    elif sid in ("long_pole", "soaking_long_pole"):
        soaked = sid == "soaking_long_pole"
        col = shade(WOOD, 0.72) if soaked else WOOD
        d.line([6, 26, 26, 6], fill=col, width=5)
        d.line([7, 24, 24, 7], fill=shade(col, 1.25), width=1)
        d.ellipse([23, 3, 29, 9], fill=ENDGRAIN)               # cut end
        d.point((26, 6), fill=WOOD_DARK)
        if soaked:  # oily sheen streaks + a drip
            for x, y in ((11, 20), (16, 15), (20, 11)):
                d.line([x, y, x + 2, y - 2], fill=(238, 232, 210, 255),
                       width=1)
            d.ellipse([5, 27, 8, 30], fill=(70, 58, 34, 255))
    elif sid in ("plank_long", "plank_short"):
        x0, x1 = (2, 30) if sid == "plank_long" else (7, 25)
        d.rectangle([x0, 12, x1, 20], fill=WOOD)
        d.rectangle([x0, 12, x1, 13], fill=WOOD_LIGHT)
        grain_h(d, x0 + 2, x1 - 2, 16)
        grain_h(d, x0 + 3, x1 - 4, 18, shade(WOOD, 0.82))
        kx = (x0 + x1) // 2 + 3
        d.ellipse([kx - 1, 15, kx + 1, 17], outline=WOOD_DARK)  # knot
        d.rectangle([x0, 12, x0 + 1, 20], fill=ENDGRAIN)        # end grain
    elif sid == "stick_common":
        pts = [(5 + i, 24 - i + (2 if 8 < i < 15 else 0)) for i in range(22)]
        d.line(pts, fill=BARK, width=3)
        d.line([(14, 15), (18, 11)], fill=BARK, width=2)        # side twig
        d.point((6, 23), fill=ENDGRAIN)
        d.point((17, 12), fill=ENDGRAIN)
    elif sid == "tonewood":
        d.rectangle([8, 10, 24, 24], fill=(216, 182, 128, 255))
        for x in range(10, 24, 2):  # fine straight grain
            d.line([x, 11, x, 23], fill=(196, 160, 104, 255), width=1)
        d.polygon([(8, 10), (11, 7), (27, 7), (24, 10)],
                  fill=(232, 200, 150, 255))
        d.polygon([(24, 10), (27, 7), (27, 21), (24, 24)],
                  fill=(188, 150, 96, 255))
    elif sid == "tuning_pegs":
        for cx, cy in ((10, 10), (17, 14), (23, 9)):
            d.polygon([(cx - 3, cy - 2), (cx + 1, cy - 3), (cx + 2, cy),
                       (cx - 2, cy + 1)], fill=WOOD_DARK)       # paddle
            d.line([cx, cy + 1, cx + 4, cy + 6], fill=WOOD, width=2)  # shaft
            d.point((cx + 4, cy + 6), fill=INK)                 # string hole
        d.line([6, 26, 26, 22], fill=METAL_DARK, width=1)       # loose string
    elif sid == "wood_block":
        d.rectangle([9, 13, 23, 25], fill=WOOD)
        d.polygon([(9, 13), (13, 9), (27, 9), (23, 13)],
                  fill=WOOD_LIGHT)
        d.polygon([(23, 13), (27, 9), (27, 21), (23, 25)],
                  fill=WOOD_DARK)
        for r in (2, 1):  # end-grain rings on the top face
            d.arc([17 - r, 10, 17 + r + 2, 13], 0, 180, fill=WOOD_DARK)
        grain_h(d, 11, 21, 19)
    elif sid.startswith("wooden_post"):
        top, bot = {"wooden_post_long": (4, 29), "wooden_post": (8, 28),
                    "wooden_post_short": (14, 28)}[sid]
        d.rectangle([11, top, 21, bot], fill=WOOD)
        d.rectangle([11, top, 21, top + 3], fill=ENDGRAIN)      # 4x4 end
        d.rectangle([11, top, 21, top + 3], outline=WOOD_DARK, width=1)
        d.line([14, top + 5, 14, bot - 2], fill=shade(WOOD, 0.85), width=1)
        d.line([18, top + 5, 18, bot - 2], fill=shade(WOOD, 1.15), width=1)
        d.line([11, bot, 21, bot], fill=WOOD_DARK, width=1)


# ------------------------------------------------------------- hesco + paint

def draw_hesco(d, rng):
    # flat-packed: three offset folded panels, wire grid + liner band
    for i, (x0, y0) in enumerate(((6, 9), (7, 13), (8, 17))):
        x1, y1 = x0 + 17, y0 + 7
        d.rectangle([x0, y0, x1, y1], fill=(128, 132, 128, 255))
        for gx in range(x0 + 3, x1, 3):
            d.line([gx, y0, gx, y1], fill=METAL_DARK, width=1)
        for gy in range(y0 + 2, y1, 2):
            d.line([x0, gy, x1, gy], fill=METAL_DARK, width=1)
        d.rectangle([x0, y0, x1, y1], outline=INK, width=1)
    # tan geotextile liner spilling from the top fold
    d.polygon([(8, 6), (24, 6), (23, 10), (9, 10)], fill=(178, 158, 120, 255))
    d.line([10, 8, 22, 8], fill=(150, 130, 96, 255), width=1)
    for px in (6, 25):  # connecting pins
        d.line([px, 8, px, 25], fill=METAL, width=1)


PAINTS = {
    "paint_black": (24, 24, 28, 255), "paint_blue": (44, 84, 190, 255),
    "paint_brown": (122, 80, 46, 255), "paint_cyan": (52, 196, 210, 255),
    "paint_gray": (150, 150, 154, 255), "paint_green": (58, 140, 62, 255),
    "paint_orange": (232, 118, 36, 255), "paint_pink": (240, 150, 186, 255),
    "paint_purple": (138, 74, 178, 255), "paint_red": (198, 44, 38, 255),
    "paint_white": (242, 242, 240, 255), "paint_yellow": (240, 200, 52, 255),
}


def draw_paint(d, rng, sid):
    col = PAINTS[sid]
    d.rectangle([10, 11, 23, 27], fill=METAL)                  # can body
    d.rectangle([10, 11, 13, 27], fill=shade(METAL, 1.12))
    d.rectangle([20, 11, 23, 27], fill=shade(METAL, 0.82))
    d.rectangle([10, 17, 23, 22], fill=col)                    # lid band/label
    d.line([12, 19, 17, 19], fill=(245, 245, 245, 255), width=1)
    d.line([12, 20, 15, 20], fill=(245, 245, 245, 255), width=1)
    d.ellipse([9, 7, 24, 13], fill=METAL_DARK)                 # rim
    d.ellipse([10, 8, 23, 12], fill=col)                       # paint surface
    d.arc([12, 8, 21, 12], 180, 300, fill=shade(col, 1.3))
    dx = rng.choice((12, 16, 20))                              # drip
    dl = rng.randint(4, 8)
    d.rectangle([dx, 12, dx + 1, 12 + dl], fill=col)
    d.ellipse([dx - 1, 11 + dl, dx + 2, 13 + dl], fill=col)
    d.rectangle([10, 27, 23, 27], fill=METAL_DARK)


# ------------------------------------------------------------------ plastic

def block3d(d, x0, y0, x1, y1, col):
    d.rectangle([x0, y0 + 3, x1, y1], fill=col)
    d.polygon([(x0, y0 + 3), (x0 + 3, y0), (x1 + 3, y0), (x1, y0 + 3)],
              fill=shade(col, 1.18))
    d.polygon([(x1, y0 + 3), (x1 + 3, y0), (x1 + 3, y1 - 3), (x1, y1)],
              fill=shade(col, 0.8))
    d.rectangle([x0, y0 + 3, x1, y1], outline=shade(col, 0.6), width=1)


def chunk(d, rng, col, finish):
    pts = [(9, 13), (14, 9), (21, 10), (25, 15), (23, 22), (17, 25),
           (11, 23), (8, 18)]
    d.polygon(pts, fill=col)
    d.line([(14, 9), (16, 17), (11, 23)], fill=shade(col, 0.78), width=1)
    d.line([(16, 17), (23, 22)], fill=shade(col, 0.85), width=1)
    d.line([(16, 17), (21, 10)], fill=shade(col, 1.1), width=1)
    if finish == "gloss":      # epoxy: hard glossy glints
        d.line([13, 12, 17, 11], fill=(240, 250, 255, 255), width=1)
        d.point((19, 19), fill=(240, 250, 255, 255))
    elif finish == "clear":    # polycarbonate: bright edges, pale fill
        d.polygon(pts, outline=(238, 248, 255, 255))
        d.line([10, 16, 15, 12], fill=(238, 248, 255, 255), width=1)
    else:                      # thermo resin: matte speckle
        for _ in range(14):
            d.point((rng.randint(10, 23), rng.randint(11, 23)),
                    fill=shade(col, 0.92))


def patch(d, col, stitch):
    d.rectangle([8, 9, 24, 24], fill=col)
    for x in range(10, 24, 3):  # stitch dashes
        d.point((x, 11), fill=stitch)
        d.point((x, 22), fill=stitch)
    for y in range(13, 22, 3):
        d.point((10, y), fill=stitch)
        d.point((22, y), fill=stitch)
    d.rectangle([8, 9, 24, 24], outline=shade(col, 0.7), width=1)


def draw_plastic(d, rng, sid):
    if sid == "10l_plastic":
        block3d(d, 6, 9, 24, 26, PLASTIC_BLUE)
        d.line([10, 17, 16, 17], fill=shade(PLASTIC_BLUE, 0.65), width=2)
    elif sid == "5l_plastic":
        block3d(d, 9, 13, 22, 25, PLASTIC_BLUE)
        d.line([12, 19, 15, 19], fill=shade(PLASTIC_BLUE, 0.65), width=2)
    elif sid == "epoxy_chunk":
        chunk(d, rng, (168, 208, 222, 255), "gloss")
    elif sid == "polycarbonate_chunk":
        chunk(d, rng, (196, 226, 240, 255), "clear")
    elif sid == "thermo_resin_chunk":
        chunk(d, rng, (110, 156, 190, 255), "matte")
    elif sid == "filter_port_40mm":
        d.ellipse([7, 7, 25, 25], fill=METAL)                  # housing
        for a in range(0, 360, 30):                            # thread ridges
            x = 16 + int(8.5 * math.cos(math.radians(a)))
            y = 16 + int(8.5 * math.sin(math.radians(a)))
            d.point((x, y), fill=METAL_DARK)
        d.ellipse([10, 10, 22, 22], fill=(176, 216, 236, 255))
        d.ellipse([13, 13, 19, 19], fill=(40, 44, 48, 255))    # air passage
        d.point((14, 13), fill=(230, 244, 252, 255))
    elif sid == "inhalation_valve":
        d.ellipse([9, 17, 23, 26], fill=(52, 52, 56, 255))     # rubber base
        d.pieslice([10, 8, 22, 22], 180, 360, fill=PLASTIC_BLUE)  # dome
        d.rectangle([10, 15, 22, 17], fill=PLASTIC_BLUE)
        d.line([16, 10, 16, 15], fill=shade(PLASTIC_BLUE, 0.6), width=1)
        d.point((13, 12), fill=(235, 246, 252, 255))
    elif sid == "lycra_patch":
        patch(d, (140, 190, 224, 255), (90, 130, 170, 255))
        d.line([10, 20, 22, 12], fill=(190, 222, 244, 255), width=1)  # sheen
    elif sid == "nylon":
        patch(d, (240, 240, 238, 255), (170, 170, 168, 255))
        d.line([11, 13, 21, 13], fill=(222, 222, 220, 255), width=1)
    elif sid == "panoramic_eyepiece":
        d.polygon([(4, 13), (28, 13), (26, 21), (6, 21)],
                  fill=(170, 214, 238, 255))                   # curved lens
        d.polygon([(4, 13), (28, 13), (26, 21), (6, 21)],
                  outline=(90, 130, 160, 255))
        d.line([7, 16, 20, 16], fill=(238, 250, 255, 255), width=2)
        d.line([5, 12, 27, 12], fill=(70, 74, 80, 255), width=1)  # frame edge
    elif sid == "plastic_sheet_small":
        d.polygon([(8, 10), (20, 8), (25, 13), (23, 22), (14, 25), (8, 20)],
                  fill=(196, 198, 200, 255))                   # crumpled sheet
        d.line([(11, 11), (14, 18), (12, 23)], fill=(160, 162, 166, 255),
               width=1)
        d.line([(20, 9), (19, 16), (21, 21)], fill=(160, 162, 166, 255),
               width=1)
        d.line([9, 14, 16, 15], fill=(226, 228, 230, 255), width=1)
    elif sid == "rigid_plastic_sheet":
        d.polygon([(7, 6), (26, 8), (25, 26), (6, 24)],
                  fill=(186, 222, 242, 255))                   # big pane
        d.polygon([(7, 6), (26, 8), (25, 26), (6, 24)],
                  outline=(96, 140, 168, 255))
        d.line([10, 22, 22, 9], fill=(240, 250, 255, 255), width=2)
        d.line([6, 24, 25, 26], fill=(120, 168, 198, 255), width=2)  # edge


# -------------------------------------------------------------------- main

NUTS = {"acorn_roasted", "acorns_blanched", "almond_pulp", "butternut_roasted",
        "butternut_shelled", "edamame_roasted", "hazelnut_pulp",
        "hickory_nut_roasted", "peanut_shelled", "soy_nuts",
        "spread_peanutbutter", "vegan_butter", "walnut_pulp"}
WOODS = {"firewood", "giant_wood_screw", "long_pole", "plank_long",
         "plank_short", "soaking_long_pole", "stick_common", "tonewood",
         "tuning_pegs", "wood_block", "wooden_post", "wooden_post_long",
         "wooden_post_short"}
PLASTICS = {"10l_plastic", "5l_plastic", "epoxy_chunk", "filter_port_40mm",
            "inhalation_valve", "lycra_patch", "nylon", "panoramic_eyepiece",
            "plastic_sheet_small", "polycarbonate_chunk",
            "rigid_plastic_sheet", "thermo_resin_chunk"}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in NUTS:
        draw_nut(d, rng, sid)
    elif sid in WOODS:
        draw_wood(d, rng, sid)
    elif sid == "hesco_mil3_cell":
        draw_hesco(d, rng)
    elif sid in PAINTS:
        draw_paint(d, rng, sid)
    elif sid in PLASTICS:
        draw_plastic(d, rng, sid)
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
    # contact-sheet preview: 8 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item69.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item69.png {prev.size}")


if __name__ == "__main__":
    main()
