#!/usr/bin/env python3
"""Batch ITEM14 (Phase 2g.4 scope extension): fluff items,
first set.

52 ids from data/json/items/fluff.json (authoritative list
embedded below; source list:
~/workspace/item-batches/14.txt). Every id was found in
fluff.json; copy-from parents (file, lab_postit_bio,
sheet_canvas) only supply the base paper/canvas nature --
each sprite is drawn from its own entry's name/description.

Parameterized family renderers driven by the entries' data:

- Modern euro-game boxes (light_blue in the data): a 3D box
  whose cover carries the game's theme -- Farmland's farm
  and fields, For Odin's viking ship, Arle Lands' windmill,
  Survival Island's palm isle, Enigma Emporium's dark gate,
  Knight Mage's sword and shield, Le Pourt's harbor,
  Crazy Mansions' haunted house, Nemo's Sea's submarine,
  Pathogen's globe and virus dots.
- Classic game sets (wooden/cardboard boxes and boards):
  alquerque's diagonal grid, backgammon's pointed case,
  chaturanga's chess pieces, the Chinese checkers star,
  Four in a Row's disc grid, dominoes, a go board with
  stones, hnefatafl's throne board, jackstraws' tube of
  sticks, a jigsaw box, latrunculi, a Ludo board, mahjong
  tiles, mancala's pitted board, Reversi's disc grid,
  hand-drawn makeshift checkers on paper and cardboard
  makeshift dominoes, Rummy-o tiles on a rack, Scrabble
  letter tiles, War Tanks' tank, Ambuscade!'s crosshair,
  and Patrol Leader as the paper map it is printed on.
- LIXA paperwork: the data file as a sheaf of graph-covered
  papers, the staff brief, the stamped military notice, and
  the cyan post-it note.
- Small objects drawn as themselves: the black glass
  shard, three butterfly nets (makeshift stick net, the
  collapsible net folded and extended), cigar cutter and
  punch, the broken black glass boat with its smashed
  hull, a gun cable lock with its key, a smoking incense
  rod, an old brass key, a framed painting, red-and-silver
  pom poms, the Pop-up Pirate barrel, a cyan ruler, and a
  small clay sculpture.

Items are objects on transparency, native 32x32, flat
UltiCa style, bodies fully opaque (alpha in {0,255});
translucency faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item14.png.
"""
import json
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
    "LIXA_generic_file",
    "LIXA_lab_file",
    "LIXA_lab_postit",
    "LIXA_mil_file",
    "agricola",
    "alquerque",
    "ambush",
    "backgammon",
    "black_glass_shard",
    "butterfly_net_collapsible",
    "butterfly_net_collapsible_extended",
    "butterfly_net_makeshift",
    "chaturanga",
    "chinese_checkers",
    "cigar_cutter",
    "cigar_punch",
    "connect_four",
    "domino",
    "feast_for_odin",
    "fields_of_arle",
    "friday",
    "glass_boat",
    "gloomhaven",
    "go",
    "gun_lock",
    "hnefatafl",
    "incense",
    "jackstraws",
    "jigsaw",
    "knight_mage",
    "latrunculi",
    "le_havre",
    "ludo",
    "mahjong",
    "makeshift_checkers",
    "makeshift_dominoes",
    "mancala",
    "mansions_of_madness",
    "nemo",
    "old_key",
    "openfire",
    "othello",
    "ouija",
    "painting",
    "pandemic",
    "pom_poms",
    "poppirate",
    "ranger",
    "ruler",
    "rummy",
    "scrabble",
    "sculpture_clay",
]


def C(r, g, b):
    return (r, g, b, 255)


def shade(col, f):
    return (max(0, min(255, int(col[0] * f))),
            max(0, min(255, int(col[1] * f))),
            max(0, min(255, int(col[2] * f))), 255)


def canvas():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


# CDDA-ish palette
WHITE = C(238, 238, 232)
PAPER = C(232, 226, 205)
PAPER_D = C(198, 190, 165)
INK = C(52, 52, 60)
GRAY = C(128, 128, 134)
DARK = C(38, 38, 44)
BLACK = C(14, 14, 16)
WOOD = C(146, 104, 62)
WOOD_D = C(106, 72, 40)
WOOD_L = C(182, 140, 92)
CARDBOARD = C(176, 148, 106)
LBLUE = C(116, 168, 208)
LBLUE_D = C(84, 128, 168)
LBLUE_L = C(158, 200, 232)
RED = C(196, 52, 48)
GREEN = C(74, 148, 66)
BLUE = C(52, 92, 178)
YELLOW = C(232, 190, 60)
TAN = C(216, 182, 128)


def box3d(d, x0, y0, x1, y1, front, lift=3):
    """A game box: front face, lighter top, darker side."""
    top = shade(front, 1.25)
    side = shade(front, 0.72)
    d.polygon([(x0, y0), (x0 + lift, y0 - lift), (x1 + lift, y0 - lift),
               (x1, y0)], fill=top)
    d.polygon([(x1, y0), (x1 + lift, y0 - lift), (x1 + lift, y1 - lift),
               (x1, y1)], fill=side)
    d.rectangle([x0, y0, x1, y1], fill=front)
    d.line([(x0, y0), (x1, y0)], fill=shade(front, 1.4))
    return (x0 + 2, y0 + 2, x1 - 2, y1 - 2)


def title_bar(d, box, col=WHITE):
    x0, y0, x1, _ = box
    d.rectangle([x0 + 2, y0 + 1, x1 - 2, y0 + 2], fill=col)


def sheet(d, x0, y0, x1, y1, col=PAPER):
    d.rectangle([x0, y0, x1, y1], fill=col)
    d.rectangle([x0, y0, x1, y1], outline=PAPER_D)


def text_lines(d, x0, y0, x1, n, col=None, gap=3):
    col = col or C(120, 118, 110)
    for i in range(n):
        y = y0 + i * gap
        d.line([(x0, y), (x1 - (2 if i % 2 else 0), y)], fill=col)


# ---------------------------------------------------------------- LIXA paper

def r_lixa_generic(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # sheaf of papers, slightly fanned
    sheet(d, 9, 6, 27, 27)
    sheet(d, 7, 5, 25, 26)
    sheet(d, 5, 4, 23, 25)
    d.rectangle([7, 6, 21, 8], fill=C(44, 62, 110))       # letterhead
    d.line([(8, 7), (13, 7)], fill=WHITE)
    # graph: axes + jagged plot line
    d.line([(8, 20), (8, 11)], fill=INK)
    d.line([(8, 20), (20, 20)], fill=INK)
    pts = [(8, 18), (11, 14), (13, 16), (16, 11), (19, 13), (20, 10)]
    d.line(pts, fill=RED, width=1)
    # small pie chart
    d.pieslice([15, 15, 21, 21], 0, 130, fill=C(84, 128, 168))
    d.pieslice([15, 15, 21, 21], 130, 360, fill=C(120, 170, 90))
    # equation squiggles
    text_lines(d, 8, 22, 20, 1)
    return img


def r_lixa_lab(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    sheet(d, 8, 3, 24, 28)
    d.rectangle([10, 5, 22, 7], fill=C(44, 62, 110))       # letterhead
    d.line([(11, 6), (16, 6)], fill=WHITE)
    d.rectangle([10, 9, 18, 10], fill=INK)                  # title
    text_lines(d, 10, 13, 22, 4)
    text_lines(d, 10, 25, 17, 1)
    return img


def r_lixa_mil(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    sheet(d, 8, 3, 24, 28)
    d.rectangle([10, 5, 22, 7], fill=C(70, 74, 60))         # mil letterhead
    d.line([(11, 6), (16, 6)], fill=WHITE)
    text_lines(d, 10, 10, 22, 3)
    # serial barcode
    x = 10
    while x < 20:
        w = rng.choice([1, 1, 2])
        d.rectangle([x, 20, x + w - 1, 23], fill=INK)
        x += w + 1
    # red stamp
    d.ellipse([16, 16, 23, 23], outline=RED, width=1)
    d.line([(18, 19), (21, 19)], fill=RED)
    # signature squiggle
    d.line([(10, 26), (13, 25), (15, 26), (18, 25)], fill=C(40, 50, 120))
    return img


def r_lixa_postit(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    cyan = C(64, 224, 232)
    # note, slightly askew, with a darker curled corner
    d.polygon([(9, 7), (24, 5), (25, 21), (10, 23)], fill=cyan)
    d.polygon([(10, 23), (25, 21), (21, 19), (13, 20)],
              fill=shade(cyan, 0.78))
    d.line([(12, 10), (21, 9)], fill=C(30, 110, 130))
    d.line([(12, 13), (22, 12)], fill=C(30, 110, 130))
    d.line([(12, 16), (18, 15)], fill=C(30, 110, 130))
    d.point((23, 7), fill=shade(cyan, 0.85))
    return img


# ------------------------------------------------------- euro game box covers

def euro_box(rng, cover_fn, base=LBLUE):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 5, 9, 25, 27, base)
    cover_fn(d, inner)
    return img


def cov_farm(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(140, 196, 120))     # fields
    for i in range(3):
        d.line([(x0, y1 - 2 - i * 3), (x1, y1 - 4 - i * 3)],
               fill=C(96, 148, 70))
    d.rectangle([x0 + 2, y0 + 2, x0 + 7, y0 + 6], fill=C(178, 84, 60))
    d.polygon([(x0 + 1, y0 + 2), (x0 + 4, y0), (x0 + 8, y0 + 2)],
              fill=C(120, 54, 40))                            # farmhouse
    d.ellipse([x1 - 6, y0 + 1, x1 - 2, y0 + 5], fill=YELLOW)  # sun


def cov_ship(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(70, 120, 170))       # sea
    for yy in (y1 - 3, y1 - 6):
        for xx in range(x0 + 1, x1 - 3, 5):
            d.arc([xx, yy, xx + 4, yy + 3], 180, 360, fill=WHITE)
    d.polygon([(x0 + 4, y1 - 7), (x1 - 4, y1 - 7), (x1 - 6, y1 - 4),
               (x0 + 6, y1 - 4)], fill=WOOD_D)                # hull
    d.line([(13, y1 - 7), (13, y0 + 2)], fill=WOOD_D)
    d.polygon([(13, y0 + 2), (13, y1 - 8), (19, y1 - 8)], fill=WHITE)
    d.line([(14, y0 + 5), (18, y0 + 5)], fill=RED)            # striped sail
    d.line([(14, y0 + 8), (17, y0 + 8)], fill=RED)


def cov_windmill(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(150, 200, 130))
    for i in range(3):
        d.line([(x0, y1 - 2 - i * 3), (x1, y1 - 3 - i * 3)],
               fill=C(104, 156, 84))
    cx, cy = 15, y0 + 8
    d.polygon([(cx - 2, y1 - 4), (cx + 2, y1 - 4), (cx + 1, cy),
               (cx - 1, cy)], fill=C(160, 130, 96))           # mill body
    for dx, dy in ((-4, -4), (4, -4), (-4, 4), (4, 4)):
        d.line([(cx, cy), (cx + dx, cy + dy)], fill=WHITE, width=1)
    d.point((cx, cy), fill=DARK)


def cov_island(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(80, 140, 190))
    d.ellipse([x0 + 3, y1 - 8, x1 - 3, y1 - 2], fill=C(232, 208, 140))
    d.line([(15, y1 - 7), (15, y0 + 5)], fill=WOOD_D)         # palm trunk
    for dx, dy in ((-5, -2), (-2, -3), (2, -3), (5, -2), (0, -4)):
        d.arc([15 + dx - 2, y0 + 5 + dy, 15 + dx + 3, y0 + 9 + dy],
              180, 330, fill=GREEN)
    d.point((15, y0 + 5), fill=C(110, 76, 40))


def cov_gate(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(34, 36, 52))         # dark cover
    d.rectangle([x0 + 4, y0 + 3, x1 - 4, y1 - 1], fill=C(58, 60, 78))
    d.rectangle([x0 + 7, y0 + 7, x1 - 7, y1 - 1], fill=C(240, 190, 90))
    d.rectangle([x0 + 4, y0 + 3, x0 + 6, y0 + 6], fill=C(24, 24, 34))
    d.rectangle([x1 - 6, y0 + 3, x1 - 4, y0 + 6], fill=C(24, 24, 34))
    d.point((15, y0 + 5), fill=YELLOW)


def cov_sword(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(96, 104, 140))
    d.polygon([(x0 + 3, y0 + 3), (x0 + 8, y0 + 2), (x0 + 8, y1 - 3),
               (x0 + 5, y1 - 1), (x0 + 3, y1 - 3)],
              fill=C(150, 60, 54))                            # shield
    d.line([(18, y0 + 2), (18, y1 - 6)], fill=C(220, 224, 230), width=2)
    d.polygon([(17, y0 + 1), (19, y0 + 1), (18, y0 - 0)], fill=WHITE)
    d.line([(15, y1 - 5), (21, y1 - 5)], fill=YELLOW, width=1)
    d.point((18, y1 - 3), fill=YELLOW)


def cov_harbor(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(96, 150, 190))
    d.rectangle([x0, y1 - 5, x1, y1], fill=C(60, 110, 160))
    d.line([(x0 + 3, y0 + 3), (x0 + 3, y1 - 5)], fill=DARK)   # crane
    d.line([(x0 + 3, y0 + 3), (x0 + 11, y0 + 3)], fill=DARK)
    d.line([(x0 + 10, y0 + 3), (x0 + 10, y0 + 6)], fill=DARK)
    d.rectangle([x0 + 8, y0 + 6, x0 + 12, y0 + 9], fill=CARDBOARD)
    d.polygon([(x0 + 12, y1 - 5), (x1 - 2, y1 - 5), (x1 - 4, y1 - 2),
               (x0 + 13, y1 - 2)], fill=C(90, 60, 44))        # ship hull
    d.rectangle([x1 - 8, y1 - 9, x1 - 5, y1 - 5], fill=WHITE)


def cov_mansion(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(30, 32, 48))
    d.ellipse([x1 - 7, y0 + 1, x1 - 3, y0 + 5], fill=C(220, 220, 200))
    d.rectangle([x0 + 4, y0 + 7, x1 - 4, y1 - 1], fill=C(52, 48, 66))
    d.polygon([(x0 + 3, y0 + 7), (15, y0 + 3), (x1 - 3, y0 + 7)],
              fill=C(24, 22, 32))                             # roof
    for wx, wy in ((x0 + 6, y0 + 10), (x1 - 8, y0 + 10), (x0 + 6, y1 - 5),
                   (x1 - 8, y1 - 5)):
        d.rectangle([wx, wy, wx + 2, wy + 2], fill=C(245, 200, 90))
    d.rectangle([14, y1 - 5, 16, y1 - 1], fill=C(20, 18, 26))


def cov_sub(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(44, 92, 140))
    for xx in range(x0 + 1, x1 - 3, 6):
        d.arc([xx, y0 + 1, xx + 4, y0 + 4], 180, 360, fill=C(140, 190, 230))
    d.ellipse([x0 + 2, y0 + 9, x1 - 2, y1 - 4], fill=C(52, 58, 66))
    d.rectangle([13, y0 + 6, 17, y0 + 9], fill=C(52, 58, 66))  # conning tower
    for px in (x0 + 6, x0 + 10, x1 - 9):
        d.ellipse([px, y0 + 11, px + 2, y0 + 13], fill=C(250, 220, 130))
    d.polygon([(x1 - 3, y0 + 9), (x1 - 1, y0 + 8), (x1 - 1, y1 - 3),
               (x1 - 3, y1 - 4)], fill=C(40, 46, 54))         # tail fin


def cov_globe(d, b):
    x0, y0, x1, y1 = b
    d.rectangle([x0, y0, x1, y1], fill=C(240, 240, 235))
    d.ellipse([x0 + 4, y0 + 2, x1 - 4, y1 - 2], fill=C(90, 150, 210))
    d.polygon([(x0 + 7, y0 + 5), (x0 + 11, y0 + 4), (x0 + 10, y0 + 8),
               (x0 + 7, y0 + 8)], fill=GREEN)                 # continents
    d.polygon([(x1 - 10, y1 - 8), (x1 - 6, y1 - 9), (x1 - 7, y1 - 5),
               (x1 - 10, y1 - 5)], fill=GREEN)
    for vx, vy, vc in ((x0 + 5, y0 + 3, RED), (x1 - 6, y0 + 4, C(240, 140, 40)),
                       (x0 + 8, y1 - 5, C(160, 80, 180))):
        d.ellipse([vx, vy, vx + 3, vy + 3], fill=vc)          # virus dots
        d.point((vx + 1, vy + 1), fill=WHITE)


def r_agricola(rng):
    return euro_box(rng, cov_farm)


def r_feast_for_odin(rng):
    return euro_box(rng, cov_ship)


def r_fields_of_arle(rng):
    return euro_box(rng, cov_windmill)


def r_friday(rng):
    return euro_box(rng, cov_island)


def r_gloomhaven(rng):
    return euro_box(rng, cov_gate, base=C(84, 96, 130))


def r_knight_mage(rng):
    return euro_box(rng, cov_sword)


def r_le_havre(rng):
    return euro_box(rng, cov_harbor)


def r_mansions_of_madness(rng):
    return euro_box(rng, cov_mansion, base=C(96, 100, 140))


def r_nemo(rng):
    return euro_box(rng, cov_sub)


def r_pandemic(rng):
    return euro_box(rng, cov_globe)


# ------------------------------------------------------------ classic games

def r_ambush(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 5, 9, 25, 27, C(84, 120, 62))
    x0, y0, x1, y1 = inner
    d.rectangle([x0, y0, x1, y1], fill=C(60, 92, 48))
    cx, cy = 15, (y0 + y1) // 2 + 1
    d.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], outline=RED, width=1)
    d.line([(cx - 7, cy), (cx + 7, cy)], fill=RED)
    d.line([(cx, cy - 7), (cx, cy + 7)], fill=RED)
    d.polygon([(cx - 2, cy + 1), (cx + 2, cy + 1), (cx + 2, cy + 4),
               (cx - 2, cy + 4)], fill=DARK)                  # helmet dome
    d.arc([cx - 3, cy - 2, cx + 3, cy + 3], 180, 360, fill=DARK)
    return img


def r_openfire(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 5, 9, 25, 27, CARDBOARD)
    x0, y0, x1, y1 = inner
    d.rectangle([x0, y0, x1, y1], fill=C(120, 128, 84))
    d.rectangle([x0 + 3, y1 - 8, x1 - 4, y1 - 4], fill=C(56, 74, 44))
    d.rectangle([x0 + 6, y1 - 11, x1 - 8, y1 - 8], fill=C(56, 74, 44))
    d.line([(x0 + 12, y1 - 10), (x1 - 2, y1 - 11)], fill=C(40, 52, 32),
           width=2)                                            # barrel
    for wx in (x0 + 5, x0 + 10, x1 - 7):
        d.ellipse([wx, y1 - 4, wx + 3, y1 - 1], fill=DARK)
    d.polygon([(x0 + 4, y0 + 2), (x0 + 6, y0 + 4), (x0 + 4, y0 + 6),
               (x0 + 2, y0 + 4)], fill=WHITE)                  # star
    return img


def r_ranger(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # folded paper tactical map
    sheet(d, 5, 5, 27, 27, col=C(214, 208, 178))
    d.line([(12, 5), (12, 27)], fill=PAPER_D)
    d.line([(20, 5), (20, 27)], fill=PAPER_D)
    for i in range(4):                                        # contour lines
        d.arc([6 + i * 2, 8 + i * 3, 18 + i * 2, 16 + i * 3], 200, 340,
              fill=C(150, 140, 100))
    pts = [(7, 24), (11, 19), (15, 21), (19, 15), (24, 12)]
    for a, bpt in zip(pts, pts[1:]):
        d.line([a, bpt], fill=RED)
    d.line([(23, 10), (26, 13)], fill=RED)
    d.line([(26, 10), (23, 13)], fill=RED)                     # X objective
    d.ellipse([6, 23, 8, 25], fill=BLUE)
    return img


def board_box(rng, base, draw_board):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 4, 8, 26, 27, base)
    draw_board(d, inner)
    return img


def r_alquerque(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=TAN)
        for i in range(5):
            gx = x0 + 1 + i * 4
            gy = y0 + 1 + i * 4
            d.line([(gx, y0 + 1), (gx, y1 - 1)], fill=WOOD_D)
            d.line([(x0 + 1, gy), (x1 - 1, gy)], fill=WOOD_D)
        d.line([(x0 + 1, y0 + 1), (x1 - 1, y1 - 1)], fill=WOOD_D)
        d.line([(x1 - 1, y0 + 1), (x0 + 1, y1 - 1)], fill=WOOD_D)
        for i in range(5):
            for j in range(5):
                if (i + j) % 2:
                    continue
                cx, cy = x0 + 1 + i * 4, y0 + 1 + j * 4
                col = DARK if (i * 5 + j) % 3 else WHITE
                d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=col)
    return board_box(rng, WOOD, bd)


def r_backgammon(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # open dark wooden case, two halves with points
    d.rectangle([4, 7, 28, 26], fill=C(70, 48, 32))
    d.rectangle([5, 8, 27, 25], fill=C(96, 66, 42))
    d.line([(16, 8), (16, 25)], fill=C(60, 40, 24), width=2)
    for half in (0, 1):
        bx = 6 + half * 11
        for i in range(3):
            px = bx + i * 3 + 1
            col = C(226, 214, 190) if i % 2 == 0 else C(150, 52, 44)
            d.polygon([(px, 9), (px + 2, 9), (px + 1, 15)], fill=col)
            d.polygon([(px, 24), (px + 2, 24), (px + 1, 18)], fill=col)
    for cx, cy, col in ((8, 12, WHITE), (12, 21, WHITE), (20, 12, DARK),
                        (24, 21, DARK), (10, 16, DARK)):
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=col)
    return img


def r_chaturanga(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        for i in range(4):
            for j in range(4):
                col = C(226, 204, 160) if (i + j) % 2 == 0 else C(140, 96, 58)
                d.rectangle([x0 + i * 5, y0 + j * 4, x0 + i * 5 + 4,
                             y0 + j * 4 + 3], fill=col)
        # pieces: pawn, rook, king
        d.ellipse([x0 + 3, y0 + 9, x0 + 6, y0 + 12], fill=WOOD_D)
        d.rectangle([x0 + 3, y0 + 12, x0 + 6, y0 + 14], fill=WOOD_D)
        d.rectangle([x0 + 12, y0 + 3, x0 + 16, y0 + 9], fill=C(60, 44, 30))
        d.rectangle([x0 + 12, y0 + 2, x0 + 16, y0 + 3], fill=C(60, 44, 30))
        d.polygon([(x1 - 5, y1 - 2), (x1 - 2, y1 - 8), (x1 - 1, y1 - 2)],
                  fill=C(226, 204, 160))
        d.point((x1 - 3, y1 - 9), fill=YELLOW)
    return board_box(rng, WOOD_D, bd)


def r_chinese_checkers(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=TAN)
        cx, cy = 15, (y0 + y1) // 2
        import math
        pts = []
        for k in range(6):
            ang = math.radians(60 * k - 90)
            pts.append((cx + 8 * math.cos(ang), cy + 8 * math.sin(ang)))
            ang2 = math.radians(60 * k - 30)
            pts.append((cx + 3.4 * math.cos(ang2), cy + 3.4 * math.sin(ang2)))
        d.polygon(pts, fill=C(196, 158, 104))
        d.polygon(pts, fill=C(242, 228, 196))
        d.line(pts + [pts[0]], fill=WOOD_D)
        cols = [RED, C(240, 140, 40), YELLOW, GREEN, BLUE, C(150, 80, 180)]
        import math as _m
        for k in range(6):
            px, py = pts[2 * k]
            d.ellipse([px - 1, py - 1, px + 1, py + 1], fill=cols[k])
            ang = _m.radians(60 * k - 90)
            mx, my = cx + 5 * _m.cos(ang), cy + 5 * _m.sin(ang)
            d.ellipse([mx - 1, my - 1, mx + 1, my + 1], fill=cols[k])
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=WHITE)
    return board_box(rng, WOOD, bd)


def r_connect_four(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([7, 6, 25, 24], fill=C(44, 84, 170))          # blue grid
    d.rectangle([6, 24, 26, 27], fill=C(30, 60, 130))         # base
    d.line([(9, 24), (8, 28)], fill=DARK, width=2)
    d.line([(23, 24), (24, 28)], fill=DARK, width=2)
    filled = {(0, 5): RED, (1, 5): YELLOW, (2, 5): RED, (3, 5): RED,
              (1, 4): RED, (2, 4): YELLOW, (4, 5): YELLOW, (5, 5): RED,
              (3, 4): YELLOW, (6, 5): YELLOW}
    for cx in range(7):
        for cy in range(6):
            px, py = 8 + cx * 2 + (cx), 7 + cy * 2 + cy
            col = filled.get((cx, cy), C(16, 24, 40))
            d.ellipse([px, py, px + 2, py + 2], fill=col)
    return img


PIPS = {0: (), 1: ((2, 2),), 2: ((1, 1), (3, 3)),
        3: ((1, 1), (2, 2), (3, 3)), 4: ((1, 1), (3, 1), (1, 3), (3, 3)),
        5: ((1, 1), (3, 1), (2, 2), (1, 3), (3, 3)),
        6: ((1, 0), (3, 0), (1, 2), (3, 2), (1, 4), (3, 4))}


def domino_tile(d, tx, ty, a, bpt):
    """A 5x10 domino tile standing upright at (tx, ty)."""
    d.rectangle([tx, ty, tx + 4, ty + 9], fill=C(238, 232, 214))
    d.rectangle([tx, ty, tx + 4, ty + 9], outline=C(178, 170, 150))
    d.line([(tx, ty + 4), (tx + 4, ty + 4)], fill=GRAY)
    d.line([(tx, ty + 5), (tx + 4, ty + 5)], fill=GRAY)
    for n, oy in ((a, 0), (bpt, 5)):
        for px, py in PIPS[n]:
            d.point((tx + px, ty + oy + py), fill=DARK)


def r_domino(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # three tiles standing in a neat row behind the open box
    domino_tile(d, 7, 5, 1, 3)
    domino_tile(d, 14, 4, 6, 6)
    domino_tile(d, 21, 5, 2, 5)
    d.rectangle([5, 15, 27, 25], fill=WOOD)                   # open box
    d.rectangle([6, 16, 26, 24], fill=WOOD_D)
    d.rectangle([5, 15, 27, 17], fill=shade(WOOD, 1.15))
    # one tile lying flat in the box
    d.rectangle([11, 19, 20, 22], fill=C(238, 232, 214))
    d.line([(15, 19), (15, 22)], fill=GRAY)
    for px, py in PIPS[4]:
        d.point((11 + px, 19 + (py * 3) // 5), fill=DARK)
    for px, py in PIPS[2]:
        d.point((16 + px, 19 + (py * 3) // 5), fill=DARK)
    return img


def r_go(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=C(222, 188, 128))
        for i in range(6):
            gx = x0 + 1 + i * 3 + (i // 3)
            gy = y0 + 1 + i * 3
            d.line([(gx, y0 + 1), (gx, y1 - 1)], fill=C(120, 92, 52))
            d.line([(x0 + 1, gy), (x1 - 1, gy)], fill=C(120, 92, 52))
        stones = [(1, 1, BLACK), (3, 2, BLACK), (2, 3, BLACK),
                  (4, 4, BLACK), (2, 2, WHITE), (3, 3, WHITE),
                  (4, 1, WHITE), (1, 4, WHITE)]
        for sx, sy, col in stones:
            px = x0 + 1 + sx * 3 + (sx // 3)
            py = y0 + 1 + sy * 3
            d.ellipse([px - 1, py - 1, px + 1, py + 1], fill=col)
    return board_box(rng, WOOD_D, bd)


def r_hnefatafl(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=C(206, 176, 124))
        for i in range(6):
            gx = x0 + 1 + i * 3 + (i // 3)
            gy = y0 + 1 + i * 3
            d.line([(gx, y0 + 1), (gx, y1 - 1)], fill=WOOD_D)
            d.line([(x0 + 1, gy), (x1 - 1, gy)], fill=WOOD_D)
        d.rectangle([13, 14, 17, 18], outline=RED)            # throne
        d.ellipse([14, 13, 16, 16], fill=C(240, 200, 80))     # king
        d.point((15, 12), fill=C(240, 200, 80))
        for px, py in ((x0 + 2, 15), (x1 - 3, 15), (15, y0 + 2),
                       (15, y1 - 3), (x0 + 5, y0 + 5), (x1 - 6, y1 - 6)):
            d.ellipse([px - 1, py - 1, px + 1, py + 1], fill=DARK)
        for px, py in ((x0 + 5, 15), (x1 - 6, 15)):
            d.ellipse([px - 1, py - 1, px + 1, py + 1], fill=WHITE)
    return board_box(rng, WOOD, bd)


def r_jackstraws(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    cols = [RED, YELLOW, GREEN, BLUE, C(240, 140, 40), C(150, 80, 180)]
    for i in range(6):                                        # sticks fanning
        ang = -0.5 + i * 0.2
        import math
        ex = 16 + int(11 * math.sin(ang))
        ey = 12 - int(9 * math.cos(ang))
        d.line([(16, 13), (ex, ey)], fill=cols[i], width=1)
    d.rectangle([11, 12, 21, 27], fill=CARDBOARD)             # tube
    d.ellipse([11, 11, 21, 14], fill=shade(CARDBOARD, 1.15))
    d.rectangle([11, 19, 21, 22], fill=RED)
    text_lines(d, 13, 24, 19, 1, col=WOOD_D)
    return img


def r_jigsaw(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 4, 9, 24, 27, CARDBOARD)
    x0, y0, x1, y1 = inner
    d.rectangle([x0, y0, x1, y1], fill=C(120, 170, 220))      # sky picture
    d.polygon([(x0, y1 - 5), (x0 + 7, y0 + 5), (x0 + 12, y1 - 5)],
              fill=C(110, 130, 100))                          # mountain
    d.polygon([(x0 + 9, y1 - 5), (x1 - 3, y0 + 7), (x1, y1 - 5)],
              fill=C(90, 116, 88))
    d.ellipse([x0 + 2, y0 + 2, x0 + 5, y0 + 5], fill=YELLOW)
    d.rectangle([x0, y1 - 5, x1, y1], fill=C(90, 150, 90))
    # loose puzzle piece
    d.rectangle([24, 20, 29, 25], fill=C(120, 170, 220))
    d.ellipse([25, 18, 28, 21], fill=C(120, 170, 220))
    d.rectangle([24, 20, 29, 25], outline=C(70, 110, 150))
    return img


def r_latrunculi(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=C(198, 168, 118))
        for i in range(5):
            gx = x0 + 1 + i * 4
            gy = y0 + 1 + i * 4 - (i // 4)
            d.line([(gx, y0 + 1), (gx, y1 - 1)], fill=WOOD_D)
            d.line([(x0 + 1, min(gy, y1 - 1)), (x1 - 1, min(gy, y1 - 1))],
                   fill=WOOD_D)
        for i, (px, py) in enumerate(((x0 + 3, y0 + 3), (x0 + 7, y0 + 7),
                                      (x1 - 4, y0 + 3), (x1 - 8, y1 - 4),
                                      (x0 + 3, y1 - 4))):
            col = C(178, 60, 48) if i % 2 == 0 else C(70, 70, 120)
            d.ellipse([px - 1, py - 1, px + 1, py + 1], fill=col)
    return board_box(rng, WOOD_D, bd)


def r_ludo(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=WHITE)
        quads = [(x0, y0, RED), (x1 - 7, y0, GREEN), (x0, y1 - 7, BLUE),
                 (x1 - 7, y1 - 7, YELLOW)]
        for qx, qy, col in quads:
            d.rectangle([qx, qy, qx + 6, qy + 6], fill=col)
            d.ellipse([qx + 2, qy + 2, qx + 4, qy + 4], fill=WHITE)
        d.rectangle([x0 + 7, y0, x1 - 8, y1], outline=PAPER_D)
        d.rectangle([7 + x0, 7 + y0, x1 - 8, y1 - 8], outline=PAPER_D)
        for px, py, col in ((15, 8, RED), (16, 15, GREEN), (14, 22, BLUE),
                            (22, 15, YELLOW)):
            d.ellipse([px - 1, py - 1, px + 1, py + 1], fill=col)
        d.rectangle([20, 20, 23, 23], fill=WHITE)             # die
        d.point((21, 21), fill=DARK)
        d.point((22, 22), fill=DARK)
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 4, 8, 26, 27, WHITE)
    bd(d, inner)
    return img


def r_mahjong(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 4, 9, 25, 27, C(120, 84, 54))
    x0, y0, x1, y1 = inner
    d.rectangle([x0, y0, x1, y1], fill=C(104, 70, 42))
    # three face-up tiles
    for tx in (x0 + 1, x0 + 8, x0 + 15):
        d.rectangle([tx, y0 + 4, tx + 5, y1 - 3], fill=C(240, 236, 220))
        d.rectangle([tx, y0 + 4, tx + 5, y1 - 3], outline=C(190, 184, 160))
    for i in range(3):                                        # bamboo tile
        d.line([(x0 + 3, y0 + 7 + i * 3), (x0 + 3, y0 + 9 + i * 3)],
               fill=GREEN, width=1)
        d.line([(x0 + 5, y0 + 7 + i * 3), (x0 + 5, y0 + 9 + i * 3)],
               fill=GREEN, width=1)
    for i in range(3):                                        # dots tile
        d.ellipse([x0 + 10 + (i % 2) * 2, y0 + 7 + (i // 2) * 3,
                   x0 + 11 + (i % 2) * 2, y0 + 8 + (i // 2) * 3], fill=BLUE)
    d.line([(x0 + 17, y0 + 7), (x0 + 20, y0 + 7)], fill=RED)  # character tile
    d.line([(x0 + 17, y0 + 10), (x0 + 20, y0 + 10)], fill=INK)
    d.line([(x0 + 18, y0 + 13), (x0 + 19, y0 + 13)], fill=INK)
    return img


def r_makeshift_checkers(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    sheet(d, 5, 5, 27, 27)                                    # scribbled paper
    for i in range(8):
        for j in range(8):
            if (i + j) % 2:
                d.rectangle([7 + i * 2 + (1 if j == 3 else 0), 7 + j * 2,
                             8 + i * 2 + (1 if j == 3 else 0), 8 + j * 2],
                            fill=C(90, 88, 84))
    for px, py, col in ((8, 8, RED), (12, 10, RED), (18, 8, RED),
                        (10, 20, DARK), (16, 22, DARK), (20, 18, DARK)):
        d.ellipse([px - 1, py - 1, px + 2, py + 1], fill=col)  # bottle caps
        d.point((px, py), fill=shade(col, 1.3))
    return img


def r_makeshift_dominoes(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([6, 14, 26, 26], fill=CARDBOARD)              # container
    d.rectangle([6, 14, 26, 16], fill=shade(CARDBOARD, 1.15))
    tiles = [(8, 7, 2, 3), (14, 6, 5, 1), (20, 7, 4, 4)]
    for tx, ty, a, bpt in tiles:
        d.rectangle([tx, ty, tx + 4, ty + 8], fill=C(226, 216, 192))
        d.rectangle([tx, ty, tx + 4, ty + 8], outline=PAPER_D)
        d.line([(tx, ty + 4), (tx + 4, ty + 4)], fill=GRAY)
        for n, oy in ((a, 1), (bpt, 5)):
            if n % 2:
                d.point((tx + 2, ty + oy + 1), fill=INK)
            if n >= 2:
                d.point((tx + 1, ty + oy), fill=INK)
                d.point((tx + 3, ty + oy + 2), fill=INK)
            if n >= 4:
                d.point((tx + 3, ty + oy), fill=INK)
                d.point((tx + 1, ty + oy + 2), fill=INK)
    d.rectangle([9, 19, 13, 22], fill=C(226, 216, 192))       # one lying in box
    d.line([(9, 20), (13, 20)], fill=GRAY)
    return img


def r_mancala(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([3, 10, 29, 22], fill=WOOD)                   # long board
    d.rectangle([3, 10, 29, 22], outline=WOOD_D)
    d.ellipse([4, 12, 7, 20], fill=WOOD_D)                    # stores
    d.ellipse([25, 12, 28, 20], fill=WOOD_D)
    bead_cols = [RED, YELLOW, GREEN, BLUE, C(240, 140, 40), WHITE]
    for row in range(2):
        for i in range(6):
            px, py = 9 + i * 3, 12 + row * 5
            d.ellipse([px - 1, py - 1, px + 2, py + 2], fill=shade(WOOD, 0.8))
            d.point((px, py), fill=bead_cols[(i + row * 3) % 6])
            d.point((px + 1, py + 1), fill=bead_cols[(i + row + 2) % 6])
    return img


def r_othello(rng):
    def bd(d, b):
        x0, y0, x1, y1 = b
        d.rectangle([x0, y0, x1, y1], fill=C(40, 130, 70))
        for i in range(5):
            gx = x0 + 1 + i * 4
            gy = y0 + 1 + i * 4 - (1 if i == 4 else 0)
            d.line([(gx, y0 + 1), (gx, y1 - 1)], fill=C(28, 96, 50))
            d.line([(x0 + 1, min(gy, y1 - 1)), (x1 - 1, min(gy, y1 - 1))],
                   fill=C(28, 96, 50))
        discs = [(1, 1, BLACK), (2, 2, WHITE), (3, 2, BLACK), (2, 3, WHITE),
                 (1, 3, BLACK), (3, 1, WHITE), (0, 2, BLACK)]
        for sx, sy, col in discs:
            px, py = x0 + 2 + sx * 4, y0 + 2 + sy * 4
            d.ellipse([px - 1, py - 1, px + 2, py + 2], fill=col)
    return board_box(rng, CARDBOARD, bd)


def r_ouija(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([4, 8, 28, 25], fill=C(168, 118, 72))         # wooden board
    d.rectangle([4, 8, 28, 25], outline=WOOD_D)
    d.rectangle([5, 9, 27, 24], outline=C(128, 84, 48))
    import math
    for k in range(13):                                       # alphabet arc
        ang = math.radians(195 - k * 12.5)
        px = 16 + int(10 * math.cos(ang))
        py = 19 - int(7 * math.sin(ang))
        d.rectangle([px, py, px + 1, py + 1], fill=C(56, 34, 18))
    for k in range(10):                                       # number row
        d.point((7 + k * 2, 22), fill=C(56, 34, 18))
    d.line([(6, 10), (9, 10)], fill=C(56, 34, 18))            # YES
    d.line([(6, 11), (8, 11)], fill=C(56, 34, 18))
    d.line([(23, 10), (26, 10)], fill=C(56, 34, 18))          # NO
    d.line([(24, 11), (26, 11)], fill=C(56, 34, 18))
    # planchette: pale teardrop with a clear lens
    d.polygon([(12, 13), (20, 13), (20, 16), (16, 21), (12, 16)],
              fill=C(216, 186, 128))
    d.polygon([(12, 13), (20, 13), (20, 16), (16, 21), (12, 16)],
              outline=C(96, 66, 34))
    d.ellipse([14, 14, 18, 18], fill=C(96, 66, 34))
    d.ellipse([15, 15, 17, 17], fill=C(205, 226, 240))
    return img


# 3x5 pixel letters for the Scrabble tiles
FONT = {
    "A": ("010", "101", "111", "101", "101"),
    "C": ("011", "100", "100", "100", "011"),
    "E": ("111", "100", "110", "100", "111"),
    "L": ("100", "100", "100", "100", "111"),
    "O": ("010", "101", "101", "101", "010"),
    "P": ("110", "101", "110", "100", "100"),
    "R": ("110", "101", "110", "101", "101"),
    "S": ("011", "100", "010", "001", "110"),
    "Y": ("101", "101", "010", "010", "010"),
}


def draw_letter(d, x, y, ch, col):
    for r, row in enumerate(FONT[ch]):
        for c, bit in enumerate(row):
            if bit == "1":
                d.point((x + c, y + r), fill=col)


def r_scrabble(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 4, 9, 25, 27, CARDBOARD)
    x0, y0, x1, y1 = inner
    d.rectangle([x0, y0, x1, y1], fill=C(226, 210, 178))      # board
    for i in range(4):
        for j in range(3):
            if (i + j) % 3 == 0:
                d.rectangle([x0 + 1 + i * 4, y0 + 1 + j * 4,
                             x0 + 4 + i * 4, y0 + 4 + j * 4],
                            fill=C(240, 180, 180))
    # letter tiles spelling PLAY across the board
    for i, ch in enumerate("PLAY"):
        tx, ty = x0 + 2 + i * 4, y1 - 7
        d.rectangle([tx, ty, tx + 3, ty + 5], fill=C(238, 226, 192))
        d.rectangle([tx, ty, tx + 3, ty + 5], outline=C(190, 170, 130))
        draw_letter(d, tx, ty, ch, INK)
    # one tile drawn properly with a letter beside the box
    d.rectangle([25, 19, 30, 25], fill=C(238, 226, 192))
    d.rectangle([25, 19, 30, 25], outline=C(190, 170, 130))
    draw_letter(d, 26, 20, "S", INK)
    return img


def r_rummy(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    inner = box3d(d, 5, 10, 25, 27, CARDBOARD)
    x0, y0, x1, y1 = inner
    d.rectangle([x0, y0, x1, y1], fill=C(150, 132, 100))
    d.rectangle([x0 + 2, y1 - 6, x1 - 2, y1 - 3], fill=WOOD_D)  # rack
    tiles = [(RED, 2), (BLUE, 1), (C(240, 140, 40), 3), (INK, 2), (RED, 1)]
    for i, (col, n) in enumerate(tiles):
        tx, ty = x0 + 2 + i * 4, y1 - 12
        d.rectangle([tx, ty, tx + 3, ty + 6], fill=C(242, 240, 232))
        for k in range(n):
            d.point((tx + 1 + (k % 2), ty + 1 + (k // 2) * 2), fill=col)
        d.line([(tx + 1, ty + 5), (tx + 2, ty + 5)], fill=col)
    return img


# -------------------------------------------------------------- small things

def r_black_glass_shard(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    pts = [(10, 6), (20, 9), (24, 18), (17, 26), (9, 22), (7, 13)]
    d.polygon(pts, fill=C(18, 18, 22))
    d.line([(10, 6), (17, 26)], fill=C(52, 52, 60))
    d.line([(20, 9), (17, 26)], fill=C(40, 40, 48))
    d.line([(10, 6), (20, 9)], fill=C(70, 70, 80))
    d.point((15, 12), fill=C(96, 96, 108))                     # faint glint
    return img


def net_head(d, cx, cy, rx, ry, frame, mesh=C(214, 208, 190)):
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], outline=frame, width=1)
    for i in range(-rx + 2, rx - 1, 3):
        d.line([(cx + i, cy - ry + 2), (cx + i + 2, cy + ry - 2)], fill=mesh)
    for i in range(-ry + 2, ry - 1, 3):
        d.line([(cx - rx + 2, cy + i), (cx + rx - 2, cy + i + 2)], fill=mesh)
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], outline=frame, width=1)


def r_net_makeshift(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.line([(8, 27), (19, 13)], fill=WOOD_D, width=2)         # stick
    net_head(d, 21, 10, 8, 7, WOOD)
    return img


def r_net_extended(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.line([(5, 28), (18, 13)], fill=C(168, 172, 178), width=2)
    d.line([(7, 25), (10, 22)], fill=C(120, 124, 130), width=2)
    net_head(d, 21, 10, 8, 7, C(168, 172, 178))
    return img


def r_net_collapsible(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # folded bundle: canvas wrap around collapsed aluminum frame
    d.rectangle([6, 13, 26, 19], fill=C(196, 186, 158))
    d.rectangle([6, 13, 26, 19], outline=C(160, 150, 120))
    for x in (10, 15, 20, 24):
        d.line([(x, 13), (x, 19)], fill=C(168, 172, 178), width=1)
    d.line([(6, 16), (26, 16)], fill=C(140, 130, 102))
    d.rectangle([14, 11, 18, 13], fill=C(120, 124, 130))      # hinge cap
    d.line([(8, 21), (12, 24)], fill=C(168, 172, 178), width=1)
    return img


def r_cigar_cutter(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    gold = C(206, 168, 64)
    d.ellipse([10, 9, 22, 23], fill=gold)
    d.ellipse([10, 9, 22, 23], outline=shade(gold, 0.7))
    d.ellipse([13, 12, 19, 18], fill=C(30, 26, 22))           # cutting hole
    d.line([(13, 15), (19, 15)], fill=C(210, 214, 220))       # blade edge
    d.rectangle([14, 6, 18, 9], fill=shade(gold, 0.85))       # top tab
    return img


def r_cigar_punch(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    steel = C(178, 182, 188)
    d.rectangle([13, 11, 19, 24], fill=steel)                 # tube body
    d.rectangle([13, 11, 19, 24], outline=shade(steel, 0.7))
    d.ellipse([13, 9, 19, 13], fill=shade(steel, 1.12))       # rim
    d.ellipse([15, 10, 17, 12], fill=DARK)                    # bore
    d.rectangle([13, 20, 19, 21], fill=shade(steel, 0.8))
    d.ellipse([14, 4, 18, 8], outline=GRAY)                   # keyring loop
    return img


def r_glass_boat(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    glass = C(22, 22, 26)
    glass_l = C(96, 96, 110)
    sail = C(58, 58, 68)
    # masts and sails (dark glass, pale rigging edges)
    d.line([(12, 4), (12, 18)], fill=glass_l, width=1)
    d.line([(21, 6), (21, 18)], fill=glass_l, width=1)
    d.polygon([(12, 4), (12, 13), (18, 13)], fill=sail)
    d.line([(12, 4), (18, 13)], fill=glass_l)
    d.polygon([(21, 6), (21, 14), (26, 14)], fill=sail)
    d.polygon([(12, 14), (12, 18), (7, 18)], fill=sail)
    d.line([(12, 4), (26, 14)], fill=glass_l)
    # solid black hull
    d.polygon([(4, 18), (28, 18), (25, 26), (8, 26)], fill=glass)
    d.line([(4, 18), (28, 18)], fill=glass_l)
    d.line([(8, 26), (25, 26)], fill=glass_l)
    d.rectangle([14, 16, 18, 18], fill=glass)                 # sterncastle
    # the smashed hole: jagged transparent gap with bright broken edges
    hole = [(16, 20), (20, 19), (22, 21), (21, 24), (18, 25), (16, 23)]
    d.polygon(hole, fill=(0, 0, 0, 0))
    d.line(hole + [hole[0]], fill=C(170, 170, 185))
    d.line([(20, 19), (23, 18)], fill=C(170, 170, 185))       # crack
    # display stand
    d.line([(10, 28), (22, 28)], fill=C(90, 70, 48), width=1)
    d.line([(13, 26), (13, 28)], fill=C(90, 70, 48))
    d.line([(19, 26), (19, 28)], fill=C(90, 70, 48))
    return img


def r_gun_lock(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # steel cable loop
    d.arc([9, 4, 23, 18], 180, 360, fill=C(150, 154, 160), width=2)
    d.arc([9, 4, 23, 18], 200, 340, fill=C(110, 114, 120), width=1)
    # lock body
    d.rectangle([10, 14, 22, 25], fill=C(58, 58, 64))
    d.rectangle([10, 14, 22, 25], outline=C(30, 30, 34))
    d.ellipse([14, 17, 18, 21], fill=C(20, 20, 24))
    d.rectangle([15, 20, 17, 23], fill=C(20, 20, 24))
    # its key, beside
    d.ellipse([24, 17, 28, 21], outline=C(190, 160, 90), width=1)
    d.line([(26, 21), (26, 26)], fill=C(190, 160, 90), width=1)
    d.line([(26, 24), (28, 24)], fill=C(190, 160, 90))
    return img


def r_incense(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.line([(10, 26), (19, 9)], fill=C(120, 82, 52), width=1)  # rod
    d.line([(17, 12), (19, 9)], fill=C(90, 90, 90), width=1)   # ash
    d.point((19, 9), fill=C(255, 120, 40))                     # ember
    d.point((20, 8), fill=C(255, 170, 80))
    # smoke wisp (pale opaque)
    smoke = [(20, 6), (19, 4), (21, 2)]
    for px, py in smoke:
        d.point((px, py), fill=C(196, 196, 200))
    d.point((22, 5), fill=C(170, 170, 176))
    return img


def r_old_key(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    brass = C(178, 148, 84)
    d.ellipse([6, 11, 13, 18], outline=brass, width=2)         # bow
    d.line([(12, 14), (25, 14)], fill=brass, width=2)          # shaft
    d.rectangle([19, 14, 20, 17], fill=brass)                  # teeth
    d.rectangle([23, 14, 24, 18], fill=brass)
    d.rectangle([11, 13, 14, 15], fill=shade(brass, 1.15))     # collar
    return img


def r_painting(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    d.rectangle([6, 5, 26, 27], fill=C(110, 78, 44))           # frame
    d.rectangle([8, 7, 24, 25], fill=C(150, 190, 225))         # sky
    d.ellipse([19, 8, 23, 12], fill=C(250, 220, 110))          # sun
    d.polygon([(8, 19), (14, 12), (19, 19)], fill=C(96, 140, 84))
    d.polygon([(14, 19), (20, 14), (24, 19)], fill=C(80, 124, 74))
    d.rectangle([8, 19, 24, 25], fill=C(110, 164, 92))         # meadow
    d.rectangle([11, 16, 12, 20], fill=C(96, 66, 40))          # tree
    d.ellipse([9, 13, 14, 17], fill=C(70, 130, 60))
    d.line([(8, 7), (24, 7)], fill=C(120, 90, 54))
    return img


def r_pom_poms(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    import math
    for cx, cy, cols in ((11, 12, (RED, C(240, 90, 84))),
                         (21, 12, (C(200, 204, 210), C(160, 164, 170)))):
        for k in range(14):
            ang = math.radians(k * 360 / 14 + 8)
            ex = cx + int(6 * math.cos(ang))
            ey = cy + int(6 * math.sin(ang))
            d.line([(cx, cy), (ex, ey)], fill=cols[k % 2], width=1)
        d.point((cx, cy), fill=cols[0])
        d.rectangle([cx - 1, cy + 6, cx + 1, cy + 10], fill=C(60, 60, 66))
    return img


def r_poppirate(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # pirate head popping out
    d.ellipse([12, 4, 20, 12], fill=C(232, 186, 148))          # face
    d.rectangle([12, 4, 20, 7], fill=RED)                      # bandana
    d.point((18, 5), fill=WHITE)
    d.ellipse([13, 8, 15, 10], fill=DARK)                      # eye patch
    d.point((17, 9), fill=DARK)                               # eye
    d.arc([14, 9, 18, 12], 20, 160, fill=C(150, 90, 60))       # grin
    # barrel
    d.rectangle([8, 12, 24, 27], fill=C(150, 104, 58))
    for x in (12, 16, 20):
        d.line([(x, 12), (x, 27)], fill=WOOD_D)
    d.rectangle([8, 15, 24, 17], fill=C(70, 70, 76))
    d.rectangle([8, 23, 24, 25], fill=C(70, 70, 76))
    # little sword hilts stuck in the slots
    for sx, col in ((9, BLUE), (22, GREEN), (15, YELLOW)):
        d.line([(sx, 13), (sx + 1, 10)], fill=C(200, 204, 210), width=1)
        d.point((sx + 1, 9), fill=col)
    return img


def r_ruler(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    cyan = C(64, 200, 214)
    d.polygon([(6, 20), (24, 6), (27, 9), (9, 23)], fill=cyan)
    d.polygon([(6, 20), (24, 6), (27, 9), (9, 23)],
              outline=shade(cyan, 0.7))
    import math
    for i in range(9):                                        # tick marks
        t = i / 8
        bx = 8 + (24 - 8) * t
        by = 21 + (6 - 21) * t
        ln = 3 if i % 2 == 0 else 2
        d.line([(bx, by), (bx + ln * 0.7, by + ln * 0.7)], fill=C(20, 90, 100))
    d.ellipse([7, 19, 9, 21], fill=(0, 0, 0, 0))              # hanging hole
    d.ellipse([7, 19, 9, 21], outline=shade(cyan, 0.7))
    return img


def r_sculpture_clay(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    clay = C(156, 106, 74)
    clay_d = shade(clay, 0.78)
    d.rectangle([9, 25, 23, 27], fill=clay_d)                  # base
    d.ellipse([11, 16, 21, 26], fill=clay)                     # body
    d.ellipse([13, 9, 20, 17], fill=clay)                      # head
    d.polygon([(19, 12), (24, 14), (19, 15)], fill=clay)       # beak
    d.point((16, 12), fill=C(40, 30, 24))                      # eye
    d.polygon([(11, 19), (7, 16), (8, 22)], fill=clay_d)       # tail
    d.arc([12, 17, 20, 24], 300, 60, fill=clay_d)              # wing line
    return img


RENDER = {
    "LIXA_generic_file": r_lixa_generic,
    "LIXA_lab_file": r_lixa_lab,
    "LIXA_lab_postit": r_lixa_postit,
    "LIXA_mil_file": r_lixa_mil,
    "agricola": r_agricola,
    "alquerque": r_alquerque,
    "ambush": r_ambush,
    "backgammon": r_backgammon,
    "black_glass_shard": r_black_glass_shard,
    "butterfly_net_collapsible": r_net_collapsible,
    "butterfly_net_collapsible_extended": r_net_extended,
    "butterfly_net_makeshift": r_net_makeshift,
    "chaturanga": r_chaturanga,
    "chinese_checkers": r_chinese_checkers,
    "cigar_cutter": r_cigar_cutter,
    "cigar_punch": r_cigar_punch,
    "connect_four": r_connect_four,
    "domino": r_domino,
    "feast_for_odin": r_feast_for_odin,
    "fields_of_arle": r_fields_of_arle,
    "friday": r_friday,
    "glass_boat": r_glass_boat,
    "gloomhaven": r_gloomhaven,
    "go": r_go,
    "gun_lock": r_gun_lock,
    "hnefatafl": r_hnefatafl,
    "incense": r_incense,
    "jackstraws": r_jackstraws,
    "jigsaw": r_jigsaw,
    "knight_mage": r_knight_mage,
    "latrunculi": r_latrunculi,
    "le_havre": r_le_havre,
    "ludo": r_ludo,
    "mahjong": r_mahjong,
    "makeshift_checkers": r_makeshift_checkers,
    "makeshift_dominoes": r_makeshift_dominoes,
    "mancala": r_mancala,
    "mansions_of_madness": r_mansions_of_madness,
    "nemo": r_nemo,
    "old_key": r_old_key,
    "openfire": r_openfire,
    "othello": r_othello,
    "ouija": r_ouija,
    "painting": r_painting,
    "pandemic": r_pandemic,
    "pom_poms": r_pom_poms,
    "poppirate": r_poppirate,
    "ranger": r_ranger,
    "ruler": r_ruler,
    "rummy": r_rummy,
    "scrabble": r_scrabble,
    "sculpture_clay": r_sculpture_clay,
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
    cols, cell = 8, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item14.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item14.png {sheet.size}")


if __name__ == "__main__":
    main()
