#!/usr/bin/env python3
"""Batch ITEM15 (Phase 2g.4 scope extension): fluff items,
final set.

19 ids from data/json/items/fluff.json (authoritative list
embedded below; source list:
~/workspace/item-batches/15.txt). Every entry was read before
drawing. Parameterized family renderers driven by the data:

- Board-game boxes (data color light_blue, wood): one painted
  box renderer with a per-game lid motif -- brass gears for
  Steam Gears (steampunk), a farm with crop rows for Soyang
  Farmers, an island in a sea panel for Deity Island, the
  gray cratered planet with a terraforming dome for
  Terraforming - Mercury, a grape bunch for Vigneto, and a
  globe for World Nations.
- Wooden game sets (data color brown): the senet board as a
  3x10 grid in its box with gold marks on the front band;
  shogi pentagonal pieces with glyph strokes standing on
  their board; the compact checkers box open with stacked
  round tokens; the compact chess box with five silhouetted
  pieces (pawn, rook, knight, queen, king) in white and
  black; the tangram pieces -- two large triangles, a medium
  triangle, two small triangles, a square and a
  parallelogram, each a different color -- laid in their
  tray.
- The compact checkerboard (data color white) as its two
  hinged checkered halves with a metal seam.
- Snakes and Ladders (green cardboard): gridded board with a
  wavy snake and a ladder drawn on it.
- The tic-tac-toe set as its literal description: a sheet of
  paper with a big '#' grid and six buttons, three red (on
  the winning diagonal) and three blue.
- Singles: the stone sculpture as a carved bust on a plinth
  (light_gray stone); the pencil sharpener with a pencil
  stub in its hole; the sketch as graphite art (sun,
  mountains, a house) on a sheet of paper; the electrolyte
  slurry as an iridescent cyan puddle with rainbow sheen
  patches; the NRE microphone as a long mic inside a glass
  ampoule on its squat tripod.

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 19 folders + preview_item15.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "sculpture_rock",
    "scythe_game",
    "senet",
    "sharpener",
    "shogi",
    "sketch",
    "small_checkboard",
    "small_checkers",
    "small_chess",
    "snakesnladders",
    "soyang",
    "spirit_island",
    "tangram",
    "terraforming_mercury",
    "tic_tac",
    "viticulture",
    "waste_electrolyte_slurry",
    "world_nations",
    "xedra_microphone",
]

OUTLINE = (28, 28, 34, 255)
WOOD = (146, 104, 62, 255)        # data color brown
WOOD_D = (110, 76, 44, 255)
WOOD_L = (178, 138, 88, 255)
WOOD_PALE = (216, 182, 118, 255)
BOXBLUE = (140, 175, 225, 255)    # data color light_blue
BOXBLUE_D = (96, 130, 180, 255)
BOXBLUE_L = (178, 205, 240, 255)
PAPER = (243, 242, 236, 255)
PAPER_D = (206, 204, 194, 255)
GRAPHITE = (96, 96, 102, 255)
STONE = (178, 178, 184, 255)      # data color light_gray
STONE_D = (128, 128, 136, 255)
STONE_L = (208, 208, 214, 255)
BRASS = (206, 164, 66, 255)
BRASS_D = (160, 122, 42, 255)
STEEL = (150, 156, 166, 255)
RED = (198, 52, 44, 255)
BLUE = (58, 92, 198, 255)
GREEN = (84, 168, 74, 255)
GREEN_D = (44, 116, 54, 255)


def canvas():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def speckle(rng, d, box, colors, n):
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=rng.choice(colors))


# ---------------------------------------------------------------- boxes

def game_box(d, motif):
    """Painted light-blue board-game box; motif(d) draws the lid art."""
    d.rectangle([3, 6, 29, 27], fill=BOXBLUE, outline=OUTLINE)
    d.rectangle([3, 6, 29, 10], fill=BOXBLUE_D, outline=OUTLINE)
    d.line([6, 8, 12, 8], fill=BOXBLUE_L, width=1)
    d.line([20, 8, 26, 8], fill=BOXBLUE_L, width=1)
    d.rectangle([26, 11, 29, 27], fill=BOXBLUE_D)
    d.line([26, 11, 26, 27], fill=OUTLINE)
    motif(d)


def gear(d, cx, cy, r_out, r_in, teeth, fill):
    pts = []
    steps = teeth * 2
    for i in range(steps):
        ang = math.pi * i / steps - math.pi / 2
        r = r_out if i % 2 == 0 else r_in
        pts.append((cx + r * math.cos(ang), cy + r * math.sin(ang)))
    d.polygon(pts, fill=fill, outline=OUTLINE)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=OUTLINE)


def draw_scythe_game(rng):
    img = canvas()
    d = ImageDraw.Draw(img)

    def motif(dd):
        gear(dd, 14, 19, 6, 4, 8, BRASS)
        gear(dd, 23, 14, 3, 2, 7, STEEL)
        dd.line([19, 22, 25, 22], fill=BRASS_D, width=2)
        dd.point((8, 13), fill=BOXBLUE_L)

    game_box(d, motif)
    return img


def draw_soyang(rng):
    img = canvas()
    d = ImageDraw.Draw(img)

    def motif(dd):
        dd.ellipse([21, 12, 25, 16], fill=(250, 214, 84, 255),
                   outline=OUTLINE)
        dd.rectangle([8, 14, 13, 19], fill=PAPER, outline=OUTLINE)
        dd.polygon([(7, 14), (14, 14), (10, 11)], fill=RED,
                   outline=OUTLINE)
        dd.rectangle([5, 20, 25, 25], fill=GREEN, outline=OUTLINE)
        for x in (9, 13, 17, 21):
            dd.line([x, 21, x, 24], fill=GREEN_D, width=1)

    game_box(d, motif)
    return img


def draw_spirit_island(rng):
    img = canvas()
    d = ImageDraw.Draw(img)

    def motif(dd):
        dd.rectangle([6, 12, 25, 25], fill=(52, 96, 178, 255),
                     outline=OUTLINE)
        dd.ellipse([9, 16, 22, 24], fill=(226, 204, 150, 255))
        dd.ellipse([10, 15, 21, 23], fill=GREEN)
        dd.ellipse([13, 17, 18, 21], fill=GREEN_D)
        for (x, y) in ((11, 13), (14, 12), (17, 13), (20, 12)):
            dd.point((x, y), fill=(255, 255, 255, 255))
        dd.arc([12, 10, 20, 16], start=200, end=340,
               fill=(255, 255, 255, 255), width=1)

    game_box(d, motif)
    return img


def draw_terraforming_mercury(rng):
    img = canvas()
    d = ImageDraw.Draw(img)

    def motif(dd):
        for (x, y) in ((6, 13), (24, 12), (8, 23), (23, 24)):
            dd.point((x, y), fill=(255, 255, 255, 255))
        dd.ellipse([9, 12, 23, 26], fill=(168, 168, 176, 255),
                   outline=OUTLINE)
        dd.ellipse([12, 15, 15, 18], fill=STONE_D)
        dd.ellipse([17, 20, 21, 24], fill=STONE_D)
        dd.ellipse([18, 13, 20, 15], fill=STONE_D)
        dd.arc([11, 21, 21, 27], start=180, end=360,
               fill=(205, 230, 238, 255), width=2)
        dd.line([16, 23, 16, 21], fill=GREEN_D, width=1)
        dd.ellipse([14, 19, 18, 22], fill=GREEN)

    game_box(d, motif)
    return img


def draw_viticulture(rng):
    img = canvas()
    d = ImageDraw.Draw(img)

    def motif(dd):
        dd.line([16, 12, 16, 14], fill=WOOD_D, width=2)
        dd.ellipse([16, 11, 21, 15], fill=GREEN, outline=OUTLINE)
        grape = (110, 62, 156, 255)
        rows = ((15, 15), (18, 15), (13, 18), (16, 18), (19, 18),
                (14, 21), (17, 21), (16, 24))
        for (x, y) in rows:
            dd.ellipse([x - 2, y - 2, x + 2, y + 2], fill=grape,
                       outline=OUTLINE)
        dd.line([6, 25, 25, 25], fill=GREEN_D, width=1)

    game_box(d, motif)
    return img


def draw_world_nations(rng):
    img = canvas()
    d = ImageDraw.Draw(img)

    def motif(dd):
        dd.ellipse([9, 11, 23, 25], fill=(70, 122, 204, 255),
                   outline=OUTLINE)
        dd.ellipse([11, 14, 15, 18], fill=GREEN)
        dd.ellipse([16, 19, 21, 23], fill=GREEN)
        dd.ellipse([18, 12, 21, 15], fill=GREEN)
        dd.line([13, 26, 19, 26], fill=WOOD_D, width=2)
        for (x, y) in ((6, 13), (25, 20)):
            dd.point((x, y), fill=(255, 255, 255, 255))

    game_box(d, motif)
    return img


# ------------------------------------------------------- wooden sets

def draw_senet(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([3, 7, 29, 27], fill=WOOD, outline=OUTLINE)
    d.rectangle([5, 9, 27, 20], fill=WOOD_PALE, outline=OUTLINE)
    # 3 rows x 10 cols grid
    for r in range(3):
        for c in range(10):
            if (r + c) % 2:
                d.rectangle([5 + c * 2 + (1 if c else 0), 9 + r * 3,
                             7 + c * 2 + (1 if c else 0), 12 + r * 3],
                            fill=(186, 148, 92, 255))
    for c in range(11):
        x = 5 + round(c * 2.2)
        d.line([x, 9, x, 20], fill=WOOD_D)
    for r in range(4):
        y = 9 + round(r * 11 / 3)
        d.line([5, y, 27, y], fill=WOOD_D)
    # pieces: black and white cones on the grid
    for (x, y) in ((8, 12), (14, 15), (21, 12)):
        d.polygon([(x, y - 2), (x + 2, y + 1), (x - 2, y + 1)],
                  fill=(40, 36, 34, 255), outline=OUTLINE)
    for (x, y) in ((11, 18), (18, 18), (24, 15)):
        d.polygon([(x, y - 2), (x + 2, y + 1), (x - 2, y + 1)],
                  fill=PAPER, outline=OUTLINE)
    # gold marks on the front band
    d.rectangle([5, 22, 27, 25], fill=WOOD_D)
    for x in (9, 16, 23):
        d.ellipse([x - 1, 22, x + 1, 24], outline=BRASS)
        d.line([x, 24, x, 25], fill=BRASS)
    return img


def draw_shogi(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([4, 8, 28, 26], fill=WOOD, outline=OUTLINE)
    d.rectangle([6, 10, 26, 19], fill=WOOD_PALE, outline=OUTLINE)
    for i in (1, 2):
        d.line([6 + i * 6, 10, 6 + i * 6, 19], fill=WOOD_D)
        d.line([6, 10 + i * 3, 26, 10 + i * 3], fill=WOOD_D)
    # three pentagonal pieces standing on the front edge
    for cx in (10, 16, 22):
        d.polygon([(cx, 16), (cx + 4, 20), (cx + 3, 25),
                   (cx - 3, 25), (cx - 4, 20)],
                  fill=(226, 196, 132, 255), outline=OUTLINE)
        d.line([cx - 1, 20, cx + 1, 20], fill=OUTLINE, width=1)
        d.line([cx, 20, cx, 23], fill=OUTLINE, width=1)
    return img


def draw_small_checkers(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # open lid flap
    d.polygon([(5, 10), (27, 10), (25, 4), (7, 4)],
              fill=WOOD_L, outline=OUTLINE)
    d.rectangle([5, 10, 27, 26], fill=WOOD, outline=OUTLINE)
    d.rectangle([7, 12, 25, 24], fill=WOOD_D)

    def disc(cx, cy, fill):
        d.ellipse([cx - 4, cy - 2, cx + 4, cy + 2], fill=fill,
                  outline=OUTLINE)
        d.ellipse([cx - 2, cy - 1, cx + 2, cy + 1], outline=OUTLINE)

    # stacked tokens and loose discs
    for i in range(3):
        disc(11, 21 - i * 2, (52, 44, 40, 255))
    for i in range(2):
        disc(20, 21 - i * 2, (172, 66, 50, 255))
    disc(14, 15, (172, 66, 50, 255))
    disc(22, 16, (52, 44, 40, 255))
    return img


def chess_piece(d, kind, cx, base, h, fill):
    if kind == "pawn":
        d.ellipse([cx - 2, base - h, cx + 2, base - h + 4], fill=fill,
                  outline=OUTLINE)
        d.polygon([(cx - 2, base - h + 4), (cx + 2, base - h + 4),
                   (cx + 3, base), (cx - 3, base)], fill=fill,
                  outline=OUTLINE)
    elif kind == "rook":
        d.rectangle([cx - 2, base - h, cx + 2, base], fill=fill,
                    outline=OUTLINE)
        for dx in (-2, 0, 2):
            d.rectangle([cx + dx - 1, base - h - 2, cx + dx + 1,
                         base - h], fill=fill, outline=OUTLINE)
        d.line([cx - 2, base - h + 3, cx + 2, base - h + 3],
               fill=OUTLINE)
    elif kind == "knight":
        d.polygon([(cx - 2, base), (cx - 2, base - h + 3),
                   (cx - 1, base - h), (cx + 2, base - h + 1),
                   (cx + 3, base - h + 4), (cx + 1, base - 3),
                   (cx + 3, base)], fill=fill, outline=OUTLINE)
        d.point((cx, base - h + 2), fill=OUTLINE)
    elif kind == "queen":
        d.polygon([(cx - 3, base), (cx - 2, base - h + 2),
                   (cx + 2, base - h + 2), (cx + 3, base)],
                  fill=fill, outline=OUTLINE)
        for dx in (-2, 0, 2):
            d.polygon([(cx + dx, base - h - 2),
                       (cx + dx + 1, base - h + 2),
                       (cx + dx - 1, base - h + 2)], fill=fill,
                      outline=OUTLINE)
    elif kind == "king":
        d.polygon([(cx - 3, base), (cx - 2, base - h),
                   (cx + 2, base - h), (cx + 3, base)],
                  fill=fill, outline=OUTLINE)
        d.line([cx, base - h - 3, cx, base - h], fill=fill, width=2)
        d.line([cx - 1, base - h - 2, cx + 1, base - h - 2],
               fill=fill, width=1)


def draw_small_chess(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    white = (238, 235, 228, 255)
    black = (42, 42, 48, 255)
    chess_piece(d, "pawn", 6, 20, 6, white)
    chess_piece(d, "rook", 11, 20, 8, white)
    chess_piece(d, "knight", 16, 20, 9, black)
    chess_piece(d, "queen", 21, 20, 10, white)
    chess_piece(d, "king", 26, 20, 10, black)
    # box: top surface and checkered front face
    d.rectangle([3, 20, 29, 23], fill=WOOD_L, outline=OUTLINE)
    d.rectangle([3, 23, 29, 27], fill=WOOD, outline=OUTLINE)
    for i in range(8):
        if i % 2 == 0:
            d.rectangle([3 + i * 3, 23, 6 + i * 3, 27], fill=PAPER_D)
    d.line([3, 23, 29, 23], fill=OUTLINE)
    return img


def draw_small_checkboard(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    dark = (110, 75, 45, 255)
    light = (240, 238, 232, 255)

    def half(x0, y0):
        for r in range(8):
            for c in range(4):
                col = light if (r + c) % 2 == 0 else dark
                d.rectangle([x0 + c * 3, y0 + r * 2,
                             x0 + c * 3 + 3, y0 + r * 2 + 2], fill=col)

    half(3, 8)
    half(17, 9)
    d.rectangle([3, 8, 15, 24], outline=OUTLINE)
    d.rectangle([17, 9, 29, 25], outline=OUTLINE)
    # hinge seam with clasps
    d.rectangle([15, 8, 17, 25], fill=STEEL, outline=OUTLINE)
    d.ellipse([15, 11, 17, 13], fill=STEEL, outline=OUTLINE)
    d.ellipse([15, 19, 17, 21], fill=STEEL, outline=OUTLINE)
    return img


def draw_tangram(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([4, 7, 28, 26], fill=WOOD, outline=OUTLINE)
    d.rectangle([5, 8, 27, 25], fill=WOOD_D, outline=OUTLINE)
    # the seven tans, each a different color, laid in the tray
    d.polygon([(6, 10), (13, 10), (6, 17)], fill=RED, outline=OUTLINE)
    d.polygon([(14, 10), (21, 10), (21, 17)], fill=(240, 140, 40, 255),
              outline=OUTLINE)
    d.polygon([(22, 10), (26, 10), (26, 14)], fill=GREEN,
              outline=OUTLINE)
    d.polygon([(23, 15), (26, 15), (26, 18)], fill=(64, 120, 220, 255),
              outline=OUTLINE)
    d.polygon([(9, 18), (12, 21), (9, 24), (6, 21)],
              fill=(250, 214, 84, 255), outline=OUTLINE)
    d.polygon([(14, 19), (18, 19), (18, 23)], fill=(150, 84, 200, 255),
              outline=OUTLINE)
    d.polygon([(20, 19), (24, 19), (22, 23), (18, 23)],
              fill=(240, 130, 180, 255), outline=OUTLINE)
    return img


def draw_snakesnladders(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([4, 4, 28, 28], fill=(238, 242, 226, 255),
                outline=GREEN_D)
    d.rectangle([4, 4, 28, 7], fill=GREEN, outline=GREEN_D)
    for i in range(1, 5):
        d.line([4 + i * 4, 8, 4 + i * 4, 28], fill=(178, 208, 168, 255))
        d.line([4, 7 + i * 4, 28, 7 + i * 4], fill=(178, 208, 168, 255))
    for r in range(5):
        for c in range(5):
            if (r + c) % 2:
                d.rectangle([4 + c * 4, 8 + r * 4, 8 + c * 4,
                             12 + r * 4], fill=(214, 232, 200, 255))
    # ladder
    d.line([10, 25, 18, 13], fill=(150, 100, 55, 255), width=2)
    d.line([13, 26, 21, 14], fill=(150, 100, 55, 255), width=2)
    for t in (0.2, 0.45, 0.7):
        x1 = 10 + (18 - 10) * t
        y1 = 25 + (13 - 25) * t
        d.line([x1, y1, x1 + 3, y1 + 1], fill=(150, 100, 55, 255),
               width=1)
    # snake: wavy body of overlapping circles, head at top right
    pts = []
    for i in range(13):
        x = 24 - i * 1.3
        y = 12 + i * 1.1 + 2.2 * math.sin(i * 1.1)
        pts.append((x, y))
    for (x, y) in reversed(pts):
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=(150, 200, 60, 255),
                  outline=GREEN_D)
    hx, hy = pts[0]
    d.ellipse([hx - 3, hy - 3, hx + 3, hy + 3], fill=(150, 200, 60, 255),
              outline=GREEN_D)
    d.point((hx + 1, hy - 1), fill=OUTLINE)
    d.line([hx + 3, hy + 1, hx + 5, hy + 2], fill=RED, width=1)
    return img


def draw_tic_tac(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([6, 4, 26, 28], fill=PAPER, outline=OUTLINE)
    # the big '#' grid
    d.line([13, 7, 13, 23], fill=GRAPHITE, width=2)
    d.line([19, 7, 19, 23], fill=GRAPHITE, width=2)
    d.line([9, 11, 23, 11], fill=GRAPHITE, width=2)
    d.line([9, 18, 23, 18], fill=GRAPHITE, width=2)

    def button(cx, cy, fill, dot):
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=fill,
                  outline=OUTLINE)
        d.point((cx, cy), fill=dot)

    # three red on the winning diagonal, three blue
    for (x, y) in ((11, 9), (16, 14), (21, 20)):
        button(x, y, RED, (120, 24, 20, 255))
    for (x, y) in ((21, 9), (11, 20), (24, 25)):
        button(x, y, BLUE, (26, 44, 110, 255))
    return img


# ------------------------------------------------------------- singles

def draw_sculpture_rock(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([8, 25, 24, 29], fill=STONE_D, outline=OUTLINE)
    d.rectangle([11, 22, 21, 25], fill=STONE, outline=OUTLINE)
    # bust: shoulders, neck, head
    d.polygon([(9, 22), (23, 22), (20, 15), (12, 15)], fill=STONE,
              outline=OUTLINE)
    d.rectangle([14, 12, 18, 15], fill=STONE, outline=OUTLINE)
    d.ellipse([12, 5, 20, 13], fill=STONE, outline=OUTLINE)
    # shading on the right side + chisel speckle
    d.polygon([(16, 5), (20, 5), (20, 13), (17, 13)], fill=STONE_D)
    d.polygon([(18, 15), (20, 15), (23, 22), (19, 22)], fill=STONE_D)
    speckle(rng, d, (10, 6, 22, 28), [STONE_D, STONE_L], 22)
    d.line([12, 22, 20, 22], fill=OUTLINE)
    return img


def draw_sharpener(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # pencil stub angling into the hole (drawn first, behind body)
    d.line([19, 11, 27, 3], fill=(240, 200, 80, 255), width=3)
    d.line([19, 11, 21, 9], fill=(226, 196, 150, 255), width=2)
    d.point((19, 11), fill=GRAPHITE)
    d.rectangle([24, 3, 28, 6], fill=(240, 130, 160, 255),
                outline=OUTLINE)
    # sharpener body: wedge of light-gray plastic
    d.polygon([(9, 22), (23, 22), (21, 13), (11, 13)], fill=STONE,
              outline=OUTLINE)
    d.ellipse([11, 10, 21, 15], fill=STONE_L, outline=OUTLINE)
    # steel blade strip with a screw
    d.polygon([(12, 14), (20, 12), (20, 14), (12, 16)], fill=STEEL,
              outline=OUTLINE)
    d.ellipse([15, 12, 17, 14], fill=STEEL, outline=OUTLINE)
    # the conical hole
    d.ellipse([14, 10, 18, 14], fill=(36, 36, 42, 255), outline=OUTLINE)
    d.line([10, 20, 22, 20], fill=STONE_D, width=1)
    speckle(rng, d, (11, 16, 21, 21), [STONE_D], 5)
    return img


def draw_sketch(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([9, 6, 27, 30], fill=PAPER_D, outline=OUTLINE)
    d.rectangle([6, 3, 24, 27], fill=PAPER, outline=OUTLINE)
    # graphite drawing: sun, mountains, house
    d.ellipse([9, 6, 13, 10], outline=GRAPHITE)
    d.point((11, 8), fill=GRAPHITE)
    d.line([7, 20, 12, 13, 15, 17, 18, 11, 23, 20], fill=GRAPHITE,
           width=1)
    d.line([7, 20, 23, 20], fill=GRAPHITE)
    d.rectangle([9, 21, 14, 25], outline=GRAPHITE)
    d.line([8, 21, 11, 18, 15, 21], fill=GRAPHITE)
    # pencil shading strokes under the ridge
    for i in range(6):
        x = 13 + rng.randint(0, 8)
        y = 21 + rng.randint(0, 3)
        d.line([x, y, x + 2, y + 1], fill=GRAPHITE)
    d.line([17, 24, 22, 24], fill=GRAPHITE)
    return img


def draw_waste_electrolyte_slurry(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    cyan = (64, 205, 215, 255)
    d.ellipse([6, 13, 26, 24], fill=cyan)
    d.ellipse([9, 11, 20, 18], fill=cyan)
    d.ellipse([14, 16, 28, 22], fill=cyan)
    d.ellipse([4, 17, 10, 21], fill=cyan)
    # rainbow oil-sheen patches
    sheen = [(220, 110, 200, 255), (240, 220, 90, 255),
             (120, 230, 140, 255), (150, 120, 230, 255),
             (240, 150, 90, 255)]
    for i, col in enumerate(sheen):
        x = rng.randint(9, 22)
        y = rng.randint(13, 20)
        d.ellipse([x, y, x + rng.randint(3, 5), y + 2], fill=col)
    d.arc([9, 13, 20, 20], start=190, end=320,
          fill=(240, 254, 255, 255), width=1)
    d.ellipse([12, 25, 15, 27], fill=cyan)
    d.ellipse([20, 24, 22, 26], fill=cyan)
    return img


def draw_xedra_microphone(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    dark = (74, 74, 82, 255)
    # squat tripod
    d.line([16, 19, 8, 28], fill=dark, width=2)
    d.line([16, 19, 16, 29], fill=dark, width=2)
    d.line([16, 19, 24, 28], fill=dark, width=2)
    d.line([16, 19, 16, 15], fill=dark, width=2)
    d.ellipse([13, 17, 19, 21], fill=dark, outline=OUTLINE)
    # cable drooping from the hub
    pts = [(19, 19 + i * 0.5) for i in range(6)]
    pts += [(20 + i, 22 + i * 0.4) for i in range(5)]
    d.line(pts, fill=OUTLINE, width=1)
    # glass ampoule (pale opaque glass) with the long mic inside
    d.rounded_rectangle([12, 3, 20, 16], radius=4,
                        fill=(205, 230, 238, 255), outline=OUTLINE)
    d.rectangle([14, 5, 18, 13], fill=(52, 52, 58, 255),
                outline=OUTLINE)
    for y in (6, 8, 10):
        d.line([15, y, 17, y], fill=STEEL)
    d.line([14, 5, 14, 12], fill=(240, 252, 255, 255), width=1)
    d.rectangle([13, 14, 19, 16], fill=dark, outline=OUTLINE)
    return img


RENDER = {
    "sculpture_rock": draw_sculpture_rock,
    "scythe_game": draw_scythe_game,
    "senet": draw_senet,
    "sharpener": draw_sharpener,
    "shogi": draw_shogi,
    "sketch": draw_sketch,
    "small_checkboard": draw_small_checkboard,
    "small_checkers": draw_small_checkers,
    "small_chess": draw_small_chess,
    "snakesnladders": draw_snakesnladders,
    "soyang": draw_soyang,
    "spirit_island": draw_spirit_island,
    "tangram": draw_tangram,
    "terraforming_mercury": draw_terraforming_mercury,
    "tic_tac": draw_tic_tac,
    "viticulture": draw_viticulture,
    "waste_electrolyte_slurry": draw_waste_electrolyte_slurry,
    "world_nations": draw_world_nations,
    "xedra_microphone": draw_xedra_microphone,
}


def main():
    assert set(RENDER) == set(IDS), set(IDS) ^ set(RENDER)
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = RENDER[item_id](rng)
        assert img.size == (32, 32), item_id
        assert set(img.getchannel("A").getdata()) <= {0, 255}, item_id
        imgs[item_id] = img
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f,
                      indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols, cell = 5, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item15.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item15.png {sheet.size}")


if __name__ == "__main__":
    main()
