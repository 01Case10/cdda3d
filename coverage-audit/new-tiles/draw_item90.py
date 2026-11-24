#!/usr/bin/env python3
"""ITEM90: vehicle furniture, base factory cartridges, speedloaders
and a mixed tail -- CDDA Voxel coverage audit.

Clusters:

  Vehicle/furniture -- fancy and crude wooden tables, water wheels
    (the XL wheel fills the frame on support posts), leather yoke
    harnesses (the heavy-duty one on steel beams).
  Base factory cartridges -- one upright cartridge each, in the
    ITEM84 house style: case profile by caliber (bottleneck
    shoulders for the rifle and necked pistol cases, belted magnum
    cases get a belt above the rim), bullet by load (FMJ full copper
    jacket, JHP with the dark hollow cavity, JSP with an exposed
    lead tip, open-tip match with a pin-hole tip, the 5.7 LF with a
    pale aluminium tip band and the SS with a dark steel tip).  The
    .36 Navy and .44 Army are paper cartridges: a tan paper tube
    with a round lead ball on top.  The .22 blank is a tiny crimped
    rimfire case with no bullet.
  Speedloaders -- a steel disc seen face-on with the rounds held
    nose-in around a central knob (6 for the .44, 5 for the .454).
  Mixed tail -- each item drawn as itself from its description:
    the stone of stabbing, mercenary coat, canvas brass catcher,
    Exodii over/under gun, broken stiltwalker, cracking handbook,
    clothes hanger, flechette, butchering crane, alien crater bomb,
    curing roe, hair curler, dimensional perambulator, door lock
    set, fermenting fish mixture, wicker fish trap, fuel bunker,
    Glock magazine and pistol, blackwood greatbow, heavy-duty
    freezer.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "v_table",
    "w_table",
    "water_wheel",
    "xl_water_wheel",
    "yoke_harness",
    "yoke_harness_heavyduty",
    "22_blank",
    "270win_jsp",
    "300_winmag",
    "300blk",
    "300blk_ss",
    "30carbine",
    "32_acp",
    "357mag_fmj",
    "357sig_fmj",
    "36navy",
    "380_FMJ",
    "38_special",
    "38super_fmj",
    "40sw",
    "44_speedloader6",
    "44army",
    "454_Casull",
    "454_speedloader5",
    "458wm",
    "45_jhp",
    "45colt_jhp",
    "500_Magnum",
    "57mm_lf",
    "57mm_ss",
    "762_jhp",
    "DEBUG_stabbing",
    "armor_mercenary_top",
    "big_brass_catcher",
    "brogyaga",
    "broken_stiltwalker_1_1",
    "catalytic_cracking_handbook",
    "clothes_hanger",
    "combatnail",
    "crane_pseudo_item",
    "crater_bomb",
    "curing_roe",
    "curler_hair",
    "dimensional_perambulator",
    "door_lock",
    "fermentable_fish_mixture",
    "fish_trap_basket",
    "fuel_bunker",
    "glock40mag_9",
    "glock_34",
    "greatbow_meteor",
    "heavy_duty_freezer",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
BRASS = (198, 160, 76, 255)
COPPER = (186, 116, 66, 255)
COPPER_DARK = (150, 88, 50, 255)
LEAD = (128, 130, 138, 255)
LEAD_DARK = (88, 90, 98, 255)
STEEL = (150, 154, 162, 255)
STEEL_DARK = (92, 96, 104, 255)
POLYMER = (44, 44, 50, 255)
POLYMER_LIGHT = (74, 74, 82, 255)
RED = (196, 48, 40, 255)
PAPER_TAN = (216, 196, 156, 255)
PAPER_DARK = (188, 164, 116, 255)
WOOD = (120, 84, 48, 255)
WOOD_DARK = (88, 60, 32, 255)
WOOD_LIGHT = (156, 116, 72, 255)
MAHOGANY = (122, 62, 44, 255)
MAHOGANY_DARK = (92, 44, 32, 255)
LEATHER = (140, 96, 58, 255)
LEATHER_DARK = (104, 68, 38, 255)
CANVAS = (168, 164, 148, 255)
CANVAS_DARK = (128, 124, 108, 255)
PURPLE = (128, 84, 178, 255)
TEAL = (44, 140, 132, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# -------------------------------------------------------------------- ammo

def draw_cartridge(d, rng, profile, bullet, bh=6):
    """One upright cartridge centred at x=16, base at y=28.

    profile: (case_w, case_top_y, neck, belt).  bullet: 'fmj' |
    'jhp' | 'jsp' | 'otm' | 'lf' | 'ss'.  bh: bullet height.
    """
    case_w, case_top, neck, belt = profile
    base = 28
    brass = BRASS
    cx = 16
    x0, x1 = cx - case_w // 2, cx + case_w // 2 - 1
    # rim / case head
    d.rectangle([x0 - 1, base - 2, x1 + 1, base], fill=shade(brass, 0.78))
    if belt:  # belted magnum: a wider belt just above the rim
        d.rectangle([x0 - 1, base - 5, x1 + 1, base - 3],
                    fill=shade(brass, 0.88))
    if neck:
        shoulder = case_top + 5
        nw = max(3, case_w - 4)
        nx0, nx1 = cx - nw // 2, cx + nw // 2 - 1
        d.rectangle([x0, shoulder, x1, base - 2], fill=brass)
        d.polygon([(x0, shoulder), (x1, shoulder), (nx1, case_top + 2),
                   (nx0, case_top + 2)], fill=brass)
        d.rectangle([nx0, case_top, nx1, case_top + 2], fill=brass)
        mouth_y, bw = case_top, nw
    else:
        d.rectangle([x0, case_top, x1, base - 2], fill=brass)
        mouth_y, bw = case_top, case_w
    d.line([x0, case_top + (5 if neck else 0), x0, base - 2],
           fill=shade(brass, 0.8), width=1)
    d.line([x1, case_top + (5 if neck else 0), x1, base - 2],
           fill=shade(brass, 1.12), width=1)
    # bullet
    bx0, bx1 = cx - bw // 2, cx + bw // 2 - 1
    btop = mouth_y - bh
    if bullet in ("fmj", "otm", "lf", "ss"):
        d.rectangle([bx0, mouth_y - 3, bx1, mouth_y], fill=COPPER)
        d.polygon([(bx0, mouth_y - 3), (bx1, mouth_y - 3),
                   (bx1 - 1, btop + 1), (bx0 + 1, btop + 1)], fill=COPPER)
        d.line([bx0 + 1, btop + 1, bx1 - 1, btop + 1], fill=COPPER_DARK,
               width=1)
        if bullet == "otm":  # open tip: a pin-hole at the very tip
            d.point((cx, btop + 1), fill=(30, 30, 34, 255))
        elif bullet == "lf":  # aluminium core: pale tip band
            d.line([bx0 + 1, btop + 2, bx1 - 1, btop + 2],
                   fill=(206, 210, 214, 255), width=1)
        elif bullet == "ss":  # steel penetrator tip
            d.line([bx0 + 1, btop + 1, bx1 - 1, btop + 1],
                   fill=STEEL_DARK, width=1)
            d.point((cx, btop + 2), fill=STEEL_DARK)
    elif bullet == "jhp":
        d.rectangle([bx0, btop + 1, bx1, mouth_y], fill=COPPER)
        d.rectangle([cx - 1, btop, cx + 1, btop + 2], fill=LEAD_DARK)
        d.point((cx, btop + 1), fill=(30, 30, 34, 255))
    elif bullet == "jsp":
        d.rectangle([bx0, mouth_y - 3, bx1, mouth_y], fill=COPPER)
        d.rectangle([bx0 + 1, btop + 1, bx1 - 1, mouth_y - 3], fill=LEAD)
        d.line([bx0 + 1, btop + 1, bx1 - 1, btop + 1], fill=LEAD_DARK,
               width=1)


def draw_paper_cartridge(d, rng, case_w, case_top):
    """Black-powder paper cartridge: tan paper tube, round lead ball."""
    base = 28
    cx = 16
    x0, x1 = cx - case_w // 2, cx + case_w // 2 - 1
    d.rectangle([x0, case_top, x1, base], fill=PAPER_TAN)
    d.line([x0, case_top, x0, base], fill=PAPER_DARK, width=1)
    d.line([x1, case_top, x1, base], fill=PAPER_DARK, width=1)
    # tied-off paper seams
    for sy in (case_top + 5, case_top + 10):
        d.line([x0, sy, x1, sy], fill=PAPER_DARK, width=1)
    d.line([x0, base, x1, base], fill=PAPER_DARK, width=1)
    # round ball seated on top
    r = case_w // 2
    d.ellipse([cx - r, case_top - 2 * r + 1, cx + r, case_top + 1],
              fill=LEAD)
    d.point((cx - 1, case_top - r - 1), fill=(196, 198, 204, 255))


def draw_blank(d, rng):
    """.22 blank: a tiny rimfire case, star-crimped shut, no bullet."""
    base = 28
    cx = 16
    x0, x1 = cx - 2, cx + 1
    d.rectangle([x0 - 1, base - 2, x1 + 1, base], fill=shade(BRASS, 0.78))
    d.rectangle([x0, 17, x1, base - 2], fill=BRASS)
    # crimped mouth: folds converging to a centre point
    d.polygon([(x0, 17), (x1, 17), (cx, 13)], fill=shade(BRASS, 0.9))
    d.point((cx, 14), fill=shade(BRASS, 0.7))
    d.line([x0, 17, cx, 13], fill=shade(BRASS, 0.7), width=1)
    d.line([x1, 17, cx, 13], fill=shade(BRASS, 0.7), width=1)


def draw_speedloader(d, rng, rounds):
    """Steel speedloader disc face-on: rounds nose-in around a knob."""
    cx, cy = 16, 16
    d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], fill=STEEL)
    d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], outline=INK)
    d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], outline=STEEL_DARK)
    for i in range(rounds):
        a = 2 * math.pi * i / rounds - math.pi / 2
        px = int(round(cx + 6 * math.cos(a)))
        py = int(round(cy + 6 * math.sin(a)))
        # brass case head with a dark primer centre
        d.ellipse([px - 2, py - 2, px + 2, py + 2], fill=BRASS)
        d.point((px, py), fill=LEAD_DARK)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_DARK)
    d.point((cx, cy), fill=INK)


# caliber profiles: (case_w, case_top, neck, belt), bullet, bullet height
AMMO = {
    "270win_jsp": ((8, 5, True, False), "jsp", 6),
    "300_winmag": ((9, 4, True, True), "jhp", 6),
    "300blk": ((7, 9, True, False), "otm", 6),
    "300blk_ss": ((7, 9, True, False), "otm", 9),
    "30carbine": ((6, 10, False, False), "fmj", 5),
    "32_acp": ((6, 13, False, False), "fmj", 5),
    "357mag_fmj": ((7, 8, False, False), "fmj", 6),
    "357sig_fmj": ((8, 11, True, False), "fmj", 5),
    "380_FMJ": ((6, 14, False, False), "fmj", 5),
    "38_special": ((7, 9, False, False), "jhp", 6),
    "38super_fmj": ((7, 11, False, False), "fmj", 5),
    "40sw": ((8, 12, False, False), "jhp", 5),
    "454_Casull": ((8, 7, False, False), "jsp", 6),
    "458wm": ((10, 6, False, True), "jsp", 6),
    "45_jhp": ((8, 12, False, False), "jhp", 5),
    "45colt_jhp": ((8, 10, False, False), "jhp", 6),
    "500_Magnum": ((10, 5, False, False), "fmj", 7),
    "57mm_lf": ((5, 10, True, False), "lf", 5),
    "57mm_ss": ((5, 10, True, False), "ss", 5),
    "762_jhp": ((8, 8, True, False), "jhp", 6),
}
PAPER = {"36navy": (6, 12), "44army": (7, 11)}


# ------------------------------------------------------- furniture/vehicle

def draw_table(d, rng, fancy):
    top = MAHOGANY if fancy else WOOD_LIGHT
    dark = MAHOGANY_DARK if fancy else WOOD_DARK
    # top slab with a bevelled edge
    d.rectangle([3, 9, 28, 11], fill=top)
    d.rectangle([4, 12, 27, 13], fill=dark)
    d.line([3, 9, 28, 9], fill=shade(top, 1.15), width=1)
    if fancy:  # inlay line on the polished top
        d.line([6, 10, 25, 10], fill=shade(top, 1.25), width=1)
    else:  # crude planks: gaps and nail heads
        for gx in (11, 19):
            d.line([gx, 9, gx, 11], fill=dark, width=1)
        for nx in (6, 16, 25):
            d.point((nx, 10), fill=STEEL_DARK)
    # apron
    d.rectangle([6, 14, 25, 16], fill=dark if fancy else WOOD)
    # legs: turned (fancy) or straight square posts (crude)
    for lx in (6, 24):
        if fancy:
            d.rectangle([lx - 1, 17, lx + 1, 20], fill=dark)
            d.ellipse([lx - 2, 20, lx + 2, 24], fill=dark)
            d.rectangle([lx - 1, 24, lx + 1, 29], fill=dark)
            d.rectangle([lx - 2, 29, lx + 2, 30], fill=dark)
        else:
            d.rectangle([lx - 1, 17, lx + 1, 30], fill=WOOD)
            d.line([lx - 1, 17, lx - 1, 30], fill=dark, width=1)
    # lower stretcher between the legs
    d.rectangle([7, 26, 24, 27], fill=dark if fancy else WOOD)


def draw_water_wheel(d, rng, xl):
    cx, cy = 16, 16
    r = 13 if xl else 10
    if xl:  # wooden support posts and a top beam
        for px in (2, 28):
            d.rectangle([px, 4, px + 2, 30], fill=WOOD_DARK)
        d.rectangle([2, 3, 30, 5], fill=WOOD)
    # paddles: short planks sticking out around the rim
    for i in range(12):
        a = 2 * math.pi * i / 12
        px = cx + (r + 1) * math.cos(a)
        py = cy + (r + 1) * math.sin(a)
        d.rectangle([px - 1, py - 1, px + 1, py + 1], fill=WOOD_LIGHT)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=WOOD_DARK, width=2)
    d.ellipse([cx - r + 3, cy - r + 3, cx + r - 3, cy + r - 3],
              outline=WOOD, width=1)
    for i in range(8):  # spokes
        a = 2 * math.pi * i / 8
        d.line([cx, cy, cx + (r - 1) * math.cos(a),
                cy + (r - 1) * math.sin(a)], fill=WOOD, width=1)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=WOOD_DARK)
    d.point((cx, cy), fill=STEEL_DARK)


def draw_yoke(d, rng, heavy):
    beam = STEEL if heavy else WOOD
    beam_dark = STEEL_DARK if heavy else WOOD_DARK
    # main beam across the top
    d.rectangle([3, 6, 28, 9], fill=beam)
    d.line([3, 6, 28, 6], fill=shade(beam, 1.12), width=1)
    if heavy:  # bolt heads along the steel beam
        for bx in range(5, 28, 4):
            d.point((bx, 8), fill=STEEL_DARK)
    # two neck bows hanging from the beam
    for bx in (10, 21):
        d.arc([bx - 4, 9, bx + 4, 21], start=0, end=180, fill=beam_dark,
              width=2)
        d.line([bx - 4, 9, bx - 4, 14], fill=beam_dark, width=1)
        d.line([bx + 4, 9, bx + 4, 14], fill=beam_dark, width=1)
    # leather harness straps with buckles below
    for sx in (6, 15, 25):
        d.rectangle([sx, 12, sx + 1, 26], fill=LEATHER)
        d.line([sx, 12, sx, 26], fill=LEATHER_DARK, width=1)
        d.rectangle([sx - 1, 19, sx + 2, 21], outline=STEEL, width=1)
    if heavy:  # a hanging chain between the straps
        for i in range(5):
            d.point((11 + i * 2, 24 + (1 if i % 2 else 0)), fill=STEEL_DARK)


# ---------------------------------------------------------------- tail items

def draw_stone_of_stabbing(d, rng):
    # enchanted gray stone with a steel spike driven through it
    d.polygon([(9, 26), (7, 18), (10, 10), (16, 6), (22, 9), (24, 17),
               (22, 26)], fill=(118, 118, 124, 255))
    d.polygon([(9, 26), (7, 18), (10, 10), (16, 6), (18, 14), (16, 26)],
              fill=(140, 140, 148, 255))
    for _ in range(8):  # stone speckle
        d.point((rng.randint(10, 21), rng.randint(10, 24)),
                fill=(96, 96, 102, 255))
    # the stabbing spike: a slim blade rising from the top of the stone
    d.polygon([(15, 10), (17, 10), (16, 2)], fill=STEEL)
    d.line([16, 9, 16, 3], fill=(210, 214, 220, 255), width=1)
    # enchantment glints
    for gx, gy in ((6, 8), (25, 13)):
        d.point((gx, gy), fill=PURPLE)
        d.point((gx, gy - 1), fill=PURPLE)
        d.point((gx - 1, gy), fill=PURPLE)


def draw_mercenary_coat(d, rng):
    olive = (86, 92, 58, 255)
    olive_dark = (62, 68, 42, 255)
    # coat body: shoulders, long skirt
    d.polygon([(10, 5), (22, 5), (24, 12), (25, 28), (7, 28), (8, 12)],
              fill=olive)
    d.polygon([(10, 5), (16, 5), (16, 28), (7, 28), (8, 12)], fill=olive)
    # collar and lapels
    d.polygon([(12, 5), (16, 9), (20, 5), (18, 4), (14, 4)], fill=olive_dark)
    # ballistic vest panel on the chest
    d.rectangle([12, 10, 20, 19], fill=(52, 56, 40, 255))
    d.line([12, 14, 20, 14], fill=olive_dark, width=1)
    # insert pockets and skirt pockets
    for px, py in ((13, 21), (18, 21)):
        d.rectangle([px, py, px + 3, py + 3], outline=olive_dark, width=1)
    d.line([16, 19, 16, 28], fill=olive_dark, width=1)
    # sleeves
    d.polygon([(8, 12), (5, 10), (4, 22), (7, 23)], fill=olive_dark)
    d.polygon([(24, 12), (27, 10), (28, 22), (25, 23)], fill=olive_dark)


def draw_brass_catcher(d, rng):
    # mounting frame at the top, canvas bag hanging below it
    d.rectangle([7, 4, 18, 6], fill=STEEL_DARK)
    d.rectangle([9, 6, 11, 9], fill=STEEL_DARK)
    # bag: trapezoid canvas pouch
    d.polygon([(8, 9), (24, 9), (26, 26), (22, 29), (10, 29), (6, 26)],
              fill=CANVAS)
    d.polygon([(8, 9), (16, 9), (15, 29), (10, 29), (6, 26)],
              fill=shade(CANVAS, 1.06))
    d.line([6, 26, 26, 26], fill=CANVAS_DARK, width=1)
    # open mouth with spent brass inside
    d.rectangle([9, 9, 23, 11], fill=(70, 62, 48, 255))
    for bx in (12, 16, 20):
        d.rectangle([bx, 8, bx + 2, 11], fill=BRASS)
    # stitch lines
    for sx in (11, 21):
        d.line([sx, 13, sx, 25], fill=CANVAS_DARK, width=1)


def draw_brogyaga(d, rng):
    # Exodii over/under express gun in profile, muzzle to the right
    # barrels: two long stacked tubes
    d.rectangle([13, 11, 30, 13], fill=STEEL_DARK)
    d.rectangle([13, 14, 30, 16], fill=STEEL_DARK)
    d.line([13, 12, 30, 12], fill=STEEL, width=1)
    d.point((30, 11), fill=INK)
    d.point((30, 15), fill=INK)
    d.point((20, 10), fill=BRASS)  # bead sight
    # receiver
    d.rectangle([9, 11, 13, 17], fill=(70, 72, 78, 255))
    # wooden fore-end under the barrels
    d.rectangle([14, 17, 22, 19], fill=WOOD)
    # stock sweeping down to the left
    d.polygon([(9, 12), (3, 14), (2, 20), (5, 21), (9, 18)], fill=WOOD)
    d.polygon([(9, 17), (5, 21), (6, 24), (10, 20)], fill=WOOD_DARK)
    # trigger and guard
    d.arc([10, 17, 14, 21], start=0, end=180, fill=INK, width=1)


def draw_stiltwalker(d, rng):
    body = (168, 172, 180, 255)
    # angular netherum body pod at the top
    d.polygon([(12, 4), (20, 4), (22, 10), (18, 14), (13, 13), (10, 9)],
              fill=body)
    d.polygon([(12, 4), (16, 4), (16, 13), (13, 13), (10, 9)],
              fill=shade(body, 1.1))
    d.point((16, 8), fill=RED)  # a dim eye
    # sound left stilt leg
    d.line([13, 13, 11, 29], fill=STEEL_DARK, width=2)
    d.line([11, 29, 9, 29], fill=STEEL_DARK, width=1)
    # right leg snapped mid-shin: upper segment, jagged break, fallen piece
    d.line([19, 13, 21, 20], fill=STEEL_DARK, width=2)
    for jx, jy in ((21, 21), (22, 20), (20, 22)):
        d.point((jx, jy), fill=STEEL)
    d.line([19, 27, 24, 24], fill=STEEL_DARK, width=2)  # fallen segment
    d.point((25, 23), fill=STEEL)
    # damage spark
    for sx, sy in ((24, 17), (26, 19)):
        d.point((sx, sy), fill=(240, 200, 80, 255))


def draw_handbook(d, rng):
    # colourful textbook, light-cyan cover, pages on the right
    d.rectangle([9, 4, 23, 28], fill=(240, 236, 226, 255))
    d.rectangle([7, 4, 21, 28], fill=(150, 216, 224, 255))
    d.rectangle([7, 4, 9, 28], fill=(110, 180, 190, 255))
    # rainbow band across the cover (solarpunk colours)
    for i, col in enumerate(((226, 110, 36, 255), (240, 200, 80, 255),
                             (62, 148, 74, 255), TEAL)):
        d.rectangle([11, 7 + i * 2, 20, 8 + i * 2], fill=col)
    # cracking-tower motif: a small refinery column with a flame tip
    d.rectangle([14, 18, 17, 25], fill=(240, 236, 226, 255))
    d.line([14, 21, 17, 21], fill=(110, 180, 190, 255), width=1)
    d.point((15, 16), fill=(226, 110, 36, 255))
    d.point((16, 17), fill=(240, 200, 80, 255))
    d.line([11, 26, 20, 26], fill=(240, 236, 226, 255), width=1)


def draw_hanger(d, rng):
    col = (150, 104, 74, 255)  # brown plastic
    # metal hook
    d.arc([13, 2, 19, 9], start=180, end=330, fill=STEEL, width=1)
    d.line([16, 6, 16, 10], fill=STEEL, width=1)
    # triangular body
    d.line([16, 10, 5, 22], fill=col, width=2)
    d.line([16, 10, 27, 22], fill=col, width=2)
    d.line([5, 22, 27, 22], fill=col, width=2)
    d.line([9, 18, 23, 18], fill=shade(col, 0.85), width=1)


def draw_flechette(d, rng):
    # a single steel flechette, point up, cruciform fins at the base
    d.polygon([(15, 3), (17, 3), (17, 20), (15, 20)], fill=STEEL)
    d.line([16, 4, 16, 19], fill=(206, 210, 216, 255), width=1)
    d.polygon([(15, 20), (11, 26), (15, 25)], fill=STEEL_DARK)
    d.polygon([(17, 20), (21, 26), (17, 25)], fill=STEEL_DARK)
    d.rectangle([15, 25, 17, 28], fill=STEEL_DARK)


def draw_crane(d, rng):
    # butchering crane: post, jib arm, pulley, chain and gambrel hook
    d.rectangle([6, 6, 8, 29], fill=WOOD_DARK)
    d.rectangle([6, 5, 24, 7], fill=WOOD)
    d.line([8, 7, 20, 14], fill=WOOD_DARK, width=1)  # brace
    d.ellipse([21, 6, 25, 10], outline=STEEL_DARK, width=1)  # pulley
    for cy in range(10, 16):  # chain links
        d.point((23, cy), fill=STEEL_DARK)
    # gambrel bar with two hooks for suspending a carcass
    d.line([18, 16, 28, 16], fill=STEEL, width=1)
    d.arc([17, 16, 21, 21], start=180, end=360, fill=STEEL, width=1)
    d.arc([25, 16, 29, 21], start=180, end=360, fill=STEEL, width=1)


def draw_crater_bomb(d, rng):
    # alien bomb: dark sphere, glowing red core, stub fins
    d.ellipse([8, 8, 24, 24], fill=(84, 86, 94, 255))
    d.ellipse([8, 8, 24, 24], outline=INK)
    d.polygon([(10, 10), (16, 7), (15, 14), (11, 15)], fill=(112, 114, 122, 255))
    d.ellipse([13, 13, 19, 19], fill=RED)
    d.ellipse([14, 14, 18, 18], fill=(240, 120, 90, 255))
    d.point((16, 16), fill=(255, 220, 180, 255))
    for fx, fy in ((6, 6), (24, 6), (6, 24), (24, 24)):  # fins
        d.rectangle([fx - 1, fy - 1, fx + 1, fy + 1], fill=STEEL_DARK)
    d.line([16, 8, 16, 4], fill=STEEL_DARK, width=1)  # antenna
    d.point((16, 3), fill=RED)
    for gx, gy in ((4, 16), (27, 16)):  # pulsing glow ticks
        d.point((gx, gy), fill=(240, 120, 90, 255))


def draw_roe(d, rng):
    # shallow bowl of salt-curing roe: orange-pink eggs, salt crystals
    d.polygon([(5, 14), (27, 14), (24, 24), (8, 24)], fill=(104, 108, 116, 255))
    d.rectangle([4, 12, 28, 14], fill=(140, 144, 152, 255))
    eggs = ((10, 10), (13, 9), (16, 10), (19, 9), (22, 10), (12, 12),
            (15, 12), (18, 12), (21, 12), (16, 8))
    for ex, ey in eggs:
        d.ellipse([ex - 2, ey - 2, ex + 2, ey + 2], fill=(232, 130, 84, 255))
        d.point((ex - 1, ey - 1), fill=(246, 180, 130, 255))
    for _ in range(7):  # salt crystals
        d.point((rng.randint(8, 24), rng.randint(8, 12)),
                fill=(250, 250, 250, 255))


def draw_curler(d, rng):
    # soft plastic hair curler: a horizontal perforated cylinder
    body = (226, 150, 180, 255)
    d.rectangle([6, 12, 25, 20], fill=body)
    d.ellipse([23, 12, 27, 20], fill=shade(body, 0.85))
    d.ellipse([4, 12, 8, 20], fill=shade(body, 1.1))
    for hx in range(10, 23, 4):  # grip holes
        for hy in (14, 17):
            d.point((hx, hy), fill=shade(body, 0.7))
    d.line([6, 12, 25, 12], fill=shade(body, 1.12), width=1)


def draw_perambulator(d, rng):
    # a pram-like carriage with a dimensional swirl under the canopy
    d.polygon([(8, 10), (24, 10), (22, 19), (10, 19)], fill=STEEL_DARK)
    d.arc([8, 4, 24, 16], start=180, end=360, fill=(58, 60, 66, 255), width=2)
    # portal swirl inside
    for r in (4, 2):
        d.ellipse([16 - r, 11 - r, 16 + r, 11 + r], outline=PURPLE, width=1)
    d.point((16, 11), fill=(210, 180, 240, 255))
    d.line([24, 12, 28, 9], fill=STEEL_DARK, width=1)  # handle
    for wx in (11, 21):  # big spoked wheels
        d.ellipse([wx - 3, 22, wx + 3, 28], outline=STEEL, width=1)
        d.point((wx, 25), fill=STEEL)
        d.line([wx - 3, 25, wx + 3, 25], fill=STEEL, width=1)
        d.line([wx, 22, wx, 28], fill=STEEL, width=1)
    d.line([11, 19, 11, 22], fill=STEEL_DARK, width=1)
    d.line([21, 19, 21, 22], fill=STEEL_DARK, width=1)


def draw_door_lock(d, rng):
    # lock set: backplate, knob, keyhole, plus gears and a wire
    d.rectangle([9, 6, 17, 26], fill=STEEL)
    d.rectangle([9, 6, 17, 26], outline=STEEL_DARK)
    d.ellipse([10, 8, 16, 14], fill=STEEL_DARK)  # knob
    d.point((12, 10), fill=(210, 214, 220, 255))
    d.ellipse([12, 18, 14, 21], fill=INK)  # keyhole
    d.rectangle([12, 21, 13, 23], fill=INK)
    # exposed gears to the right
    for gx, gy, gr in ((23, 11, 3), (25, 18, 2), (22, 23, 2)):
        d.ellipse([gx - gr, gy - gr, gx + gr, gy + gr], outline=BRASS, width=1)
        d.point((gx, gy), fill=BRASS)
    d.line([18, 24, 24, 27], fill=RED, width=1)  # connecting wire
    d.line([18, 25, 23, 28], fill=(52, 52, 58, 255), width=1)


def draw_fish_mixture(d, rng):
    # open tank of fermenting fish: murky liquid, fish shapes, bubbles
    d.polygon([(6, 10), (26, 10), (24, 27), (8, 27)], fill=(96, 100, 106, 255))
    d.rectangle([5, 8, 27, 10], fill=STEEL)
    d.rectangle([6, 10, 26, 13], fill=(110, 116, 74, 255))  # murky surface
    for fx, fy in ((11, 11), (19, 12)):  # fish silhouettes at the surface
        d.polygon([(fx - 3, fy), (fx + 2, fy - 1), (fx + 4, fy - 2),
                   (fx + 3, fy + 1), (fx + 2, fy + 1)], fill=(150, 152, 140, 255))
        d.point((fx - 2, fy), fill=INK)
    for _ in range(6):  # ferment bubbles
        d.point((rng.randint(8, 24), rng.randint(10, 12)),
                fill=(196, 200, 160, 255))
    d.line([8, 20, 24, 20], fill=STEEL_DARK, width=1)


def draw_fish_trap(d, rng):
    # long conical wicker trap lying horizontally, funnel mouth at left
    d.polygon([(6, 10), (26, 13), (26, 19), (6, 22), (3, 16)],
              fill=(150, 116, 74, 255))
    # woven bands across the cone
    for i, bx in enumerate((8, 12, 16, 20, 24)):
        w = 6 - i // 2
        d.line([bx, 16 - w, bx, 16 + w], fill=WOOD_DARK, width=1)
    for by in (13, 16, 19):
        d.line([5, by, 26, by], fill=shade((150, 116, 74, 255), 0.85), width=1)
    # funnel throat inside the mouth
    d.polygon([(6, 12), (11, 15), (11, 17), (6, 20)], fill=WOOD_DARK)
    d.ellipse([3, 13, 6, 19], outline=WOOD_DARK, width=1)


def draw_fuel_bunker(d, rng):
    # steel bin for solid fuel, open top full of coal
    d.polygon([(7, 11), (25, 11), (23, 27), (9, 27)], fill=(196, 198, 202, 255))
    d.polygon([(7, 11), (16, 11), (15, 27), (9, 27)], fill=(214, 216, 220, 255))
    d.rectangle([6, 9, 26, 11], fill=STEEL)
    for cx, cy in ((11, 8), (16, 7), (21, 8), (14, 9), (19, 9)):  # coal
        d.polygon([(cx - 2, cy + 1), (cx, cy - 2), (cx + 2, cy + 1)],
                  fill=(40, 40, 46, 255))
    d.rectangle([13, 18, 19, 21], outline=STEEL_DARK, width=1)  # hatch
    d.point((16, 19), fill=STEEL_DARK)


def draw_glock_mag(d, rng):
    # compact polymer Glock magazine, witness holes down the back
    d.polygon([(12, 6), (20, 6), (21, 25), (11, 25)], fill=POLYMER)
    d.rectangle([10, 25, 22, 28], fill=POLYMER_LIGHT)  # baseplate
    d.line([13, 7, 13, 24], fill=POLYMER_LIGHT, width=1)
    for wy in (10, 14, 18, 22):  # witness holes with brass behind
        d.point((18, wy), fill=BRASS)
        d.point((19, wy), fill=POLYMER_LIGHT)


def draw_glock34(d, rng):
    # Glock 34 in profile: long slide, polymer frame, muzzle right
    d.rectangle([6, 9, 29, 13], fill=(52, 52, 58, 255))  # long slide
    d.line([6, 9, 29, 9], fill=POLYMER_LIGHT, width=1)
    for sx in range(8, 13):  # rear serrations
        d.line([sx, 10, sx - 1, 12], fill=INK, width=1)
    d.point((28, 10), fill=STEEL)  # front sight
    d.rectangle([6, 13, 24, 16], fill=POLYMER)  # frame
    d.rectangle([22, 13, 26, 15], fill=POLYMER)  # accessory rail block
    # grip sweeping down-left
    d.polygon([(7, 16), (12, 16), (11, 27), (6, 27)], fill=POLYMER)
    d.line([8, 19, 11, 19], fill=POLYMER_LIGHT, width=1)
    d.line([7, 23, 11, 23], fill=POLYMER_LIGHT, width=1)
    # trigger and guard
    d.arc([13, 15, 19, 20], start=0, end=180, fill=INK, width=1)
    d.line([15, 16, 15, 18], fill=INK, width=1)


def draw_greatbow(d, rng):
    # tall blackwood greatbow, asymmetric limbs, near-vertical
    black = (34, 30, 30, 255)
    black_light = (58, 50, 48, 255)
    # upper limb (longer) and lower limb curving from the grip
    d.line([14, 15, 11, 3], fill=black, width=2)
    d.line([11, 3, 13, 2], fill=black, width=1)
    d.line([14, 17, 12, 29], fill=black, width=2)
    d.line([12, 29, 14, 30], fill=black, width=1)
    d.line([13, 2, 14, 30], fill=(206, 200, 188, 255), width=1)  # string
    # wrapped grip at the centre
    d.rectangle([13, 14, 16, 18], fill=LEATHER_DARK)
    d.line([13, 15, 16, 15], fill=LEATHER, width=1)
    d.line([13, 17, 16, 17], fill=LEATHER, width=1)
    d.line([14, 15, 12, 4], fill=black_light, width=1)  # grain highlight


def draw_freezer(d, rng):
    # heavy-duty upright freezer: tall box, split door, vent grille
    d.rectangle([8, 3, 24, 29], fill=(226, 228, 232, 255))
    d.rectangle([8, 3, 24, 29], outline=STEEL_DARK)
    d.line([9, 4, 9, 28], fill=(240, 242, 246, 255), width=1)
    d.line([8, 11, 24, 11], fill=STEEL_DARK, width=1)  # door split
    d.rectangle([21, 5, 22, 9], fill=STEEL_DARK)  # top handle
    d.rectangle([21, 13, 22, 18], fill=STEEL_DARK)  # main handle
    for vx in range(10, 23, 2):  # bottom vent slots
        d.line([vx, 25, vx, 27], fill=STEEL_DARK, width=1)
    for fx, fy in ((11, 14), (14, 20), (19, 22)):  # frost sparkles
        d.point((fx, fy), fill=(170, 205, 230, 255))
        d.point((fx + 1, fy), fill=(170, 205, 230, 255))


DRAWERS = {
    "v_table": lambda d, r: draw_table(d, r, True),
    "w_table": lambda d, r: draw_table(d, r, False),
    "water_wheel": lambda d, r: draw_water_wheel(d, r, False),
    "xl_water_wheel": lambda d, r: draw_water_wheel(d, r, True),
    "yoke_harness": lambda d, r: draw_yoke(d, r, False),
    "yoke_harness_heavyduty": lambda d, r: draw_yoke(d, r, True),
    "22_blank": draw_blank,
    "44_speedloader6": lambda d, r: draw_speedloader(d, r, 6),
    "454_speedloader5": lambda d, r: draw_speedloader(d, r, 5),
    "DEBUG_stabbing": draw_stone_of_stabbing,
    "armor_mercenary_top": draw_mercenary_coat,
    "big_brass_catcher": draw_brass_catcher,
    "brogyaga": draw_brogyaga,
    "broken_stiltwalker_1_1": draw_stiltwalker,
    "catalytic_cracking_handbook": draw_handbook,
    "clothes_hanger": draw_hanger,
    "combatnail": draw_flechette,
    "crane_pseudo_item": draw_crane,
    "crater_bomb": draw_crater_bomb,
    "curing_roe": draw_roe,
    "curler_hair": draw_curler,
    "dimensional_perambulator": draw_perambulator,
    "door_lock": draw_door_lock,
    "fermentable_fish_mixture": draw_fish_mixture,
    "fish_trap_basket": draw_fish_trap,
    "fuel_bunker": draw_fuel_bunker,
    "glock40mag_9": draw_glock_mag,
    "glock_34": draw_glock34,
    "greatbow_meteor": draw_greatbow,
    "heavy_duty_freezer": draw_freezer,
}


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in AMMO:
        profile, bullet, bh = AMMO[sid]
        draw_cartridge(d, rng, profile, bullet, bh)
    elif sid in PAPER:
        case_w, case_top = PAPER[sid]
        draw_paper_cartridge(d, rng, case_w, case_top)
    elif sid in DRAWERS:
        DRAWERS[sid](d, rng)
    else:
        raise KeyError(sid)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    # the batch file is authoritative: refuse to run out of sync with it
    batch_path = os.path.expanduser("~/workspace/item-batches/90.txt")
    with open(batch_path) as f:
        batch_ids = [line.strip() for line in f if line.strip()]
    assert batch_ids == IDS, "embedded IDS differ from 90.txt"
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
                       "preview_item90.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item90.png {prev.size}")


if __name__ == "__main__":
    main()
