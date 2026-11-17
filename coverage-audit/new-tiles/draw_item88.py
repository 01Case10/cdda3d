#!/usr/bin/env python3
"""ITEM88: Beowulf mags, mortar shells, LIXA attacks, clothing, alien
fruit, presses, pauldrons, cardboard gear, ballista, bio weapons,
bile, bottles, bp_/reloaded_ ammo, whips, fish spears, pet gear and
an exoskeleton -- CDDA Voxel coverage audit.

Ammo pairs are drawn as one upright cartridge: case profile by
caliber (.223 and .308 bottleneck rifle cases, .458 WM a belted
magnum, .44/.454/.500 straight revolver cases), bullet by load
(FMJ full copper jacket, JSP exposed flat lead tip), finish by
prefix: bp_ soot-darkened black-powder brass, reloaded_ scuffed
two-tone handloads.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "50beowulf_ar15mag",
    "stanag30_beowulf",
    "60mm_shell_m720a1",
    "60mm_shell_m768",
    "LIXA_spit_light",
    "LIXA_vector_volley",
    "abaya",
    "jacket_ninja",
    "advanced_ecig",
    "crackpipe",
    "alien_meadow_bush_2_fruit",
    "alien_meadow_tree_3_fruit",
    "arbor_press",
    "press_dowel",
    "armguard_metal_sheets_shoulders",
    "ch_armguard_metal_sheets_shoulders",
    "armor_cardboard_cuirass",
    "badge_cardboard",
    "ballista",
    "bolt_ballista_steel",
    "barb_paralysis",
    "gene_sting_barb",
    "beeper",
    "chimes",
    "bio_blade_weapon",
    "bio_claws_weapon",
    "boomer_bile",
    "boomer_bile_adv",
    "bottle_folding",
    "bottle_metal",
    "bp_223",
    "reloaded_223",
    "bp_308",
    "reloaded_308",
    "bp_44fmj",
    "reloaded_44fmj",
    "bp_454_Casull",
    "reloaded_454_Casull",
    "bp_458wm",
    "reloaded_458wm",
    "bp_500_Magnum",
    "reloaded_500_Magnum",
    "bronze_nail",
    "rivets",
    "bullwhip",
    "bullwhip_razor",
    "c_fishspear",
    "m_fishspear",
    "chicken_cage",
    "horse_tack",
    "combat_exoskeleton_medium",
    "combat_exoskeleton_medium_on",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 44, 50, 255)
STEEL = (150, 156, 164, 255)
STEEL_DARK = (96, 102, 110, 255)
STEEL_LIGHT = (188, 194, 202, 255)
LC_STEEL = (168, 172, 178, 255)      # mild sheet steel
CH_STEEL = (86, 96, 118, 255)        # hardened, blue-dark
ALUMINUM = (176, 180, 186, 255)
BRASS = (198, 160, 76, 255)
BRASS_DARK = (158, 122, 54, 255)
COPPER = (186, 112, 66, 255)
LEAD = (128, 132, 140, 255)
WOOD = (110, 78, 44, 255)
WOOD_DARK = (82, 56, 30, 255)
LEATHER = (122, 84, 50, 255)
LEATHER_DARK = (92, 60, 34, 255)
CARDBOARD = (178, 142, 92, 255)
CARDBOARD_DARK = (146, 112, 66, 255)
BLACK_CLOTH = (38, 38, 44, 255)
CLOTH_SHADE = (62, 62, 70, 255)
BONE = (226, 218, 196, 255)
BONE_SHADE = (196, 186, 158, 255)
GLASS = (198, 220, 226, 255)
GLASS_SHADE = (158, 186, 198, 255)
GLOW = (250, 240, 170, 255)
CYAN = (86, 210, 220, 255)
BILE = (110, 168, 52, 255)
BILE_ADV = (140, 220, 70, 255)
OLIVE = (104, 110, 62, 255)
OLIVE_DARK = (78, 84, 48, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------- ammo

def cartridge(d, rng, cx, case_w, case_h, bullet, finish, bottleneck=False,
              belt=False):
    """One upright cartridge, base rim at y=28.

    bullet: 'fmj' copper ogive, 'jsp' flat exposed lead tip.
    finish: 'bp' soot-dark, 'reloaded' scuffed two-tone.
    """
    y_base = 28
    y_top = y_base - case_h
    hw = case_w // 2
    case = BRASS
    if finish == "bp":
        case = shade(BRASS, 0.62)
    # case body
    if bottleneck:
        d.polygon([(cx - hw, y_base), (cx - hw, y_top + 4),
                   (cx - 2, y_top), (cx + 2, y_top),
                   (cx + hw, y_top + 4), (cx + hw, y_base)], fill=case)
        d.line([cx - hw, y_top + 4, cx + hw, y_top + 4],
               fill=shade(case, 0.8), width=1)
        neck_y = y_top
    else:
        d.rectangle([cx - hw, y_top, cx + hw, y_base], fill=case)
        neck_y = y_top
    if belt:  # belted magnum: raised belt ring just above the rim
        d.rectangle([cx - hw - 1, y_base - 4, cx + hw + 1, y_base - 2],
                    fill=shade(case, 0.9))
    # base rim
    d.rectangle([cx - hw - 1, y_base, cx + hw + 1, y_base + 1],
                fill=shade(case, 0.78))
    # case shading: left light, right dark
    d.line([cx - hw + 1, y_top + 2, cx - hw + 1, y_base - 1],
           fill=shade(case, 1.18), width=1)
    d.line([cx + hw - 1, y_top + 2, cx + hw - 1, y_base - 1],
           fill=shade(case, 0.72), width=1)
    if finish == "reloaded":
        # two-tone: duller upper half + scratch marks
        d.rectangle([cx - hw, y_top, cx + hw, (y_top + y_base) // 2],
                    fill=shade(case, 0.82))
        d.line([cx - hw + 1, y_top + 2, cx - hw + 1, y_base - 1],
               fill=shade(case, 1.1), width=1)
        for _ in range(3):
            sx = rng.randint(cx - hw + 1, cx + hw - 2)
            sy = rng.randint(y_top + 2, y_base - 3)
            d.line([sx, sy, sx + 1, sy + 2], fill=shade(case, 0.6), width=1)
    if finish == "bp":
        # soot smudges over the case mouth and body
        for _ in range(6):
            sx = rng.randint(cx - hw, cx + hw - 1)
            sy = rng.randint(y_top, y_top + case_h // 2)
            d.point((sx, sy), fill=(52, 46, 40, 255))
            d.point((sx, sy + 1), fill=(66, 58, 48, 255))
    # bullet
    bw = max(2, hw - 1)
    if bullet == "fmj":
        d.polygon([(cx - bw, neck_y), (cx - bw, neck_y - 4),
                   (cx, neck_y - 9), (cx + bw, neck_y - 4),
                   (cx + bw, neck_y)], fill=COPPER)
        d.line([cx - 1, neck_y - 3, cx - 1, neck_y - 1],
               fill=shade(COPPER, 1.2), width=1)
    else:  # jsp: jacket walls with a flat exposed lead tip
        d.rectangle([cx - bw, neck_y - 5, cx + bw, neck_y], fill=COPPER)
        d.rectangle([cx - bw + 1, neck_y - 7, cx + bw - 1, neck_y - 5],
                    fill=LEAD)
        d.line([cx - bw + 1, neck_y - 7, cx + bw - 1, neck_y - 7],
               fill=shade(LEAD, 1.15), width=1)


def draw_ammo(d, rng, sid):
    # sid -> (case_w, case_h, bullet, bottleneck, belt)
    spec = {
        "223": (5, 13, "fmj", True, False),
        "308": (6, 15, "fmj", True, False),
        "44fmj": (6, 14, "fmj", False, False),
        "454_Casull": (7, 15, "jsp", False, False),
        "458wm": (7, 16, "jsp", True, True),
        "500_Magnum": (8, 16, "jsp", False, False),
    }
    finish = "bp" if sid.startswith("bp_") else "reloaded"
    key = sid.split("_", 1)[1]
    cw, chh, bullet, bn, belt = spec[key]
    cartridge(d, rng, 16, cw, chh, bullet, finish, bottleneck=bn, belt=belt)


# ------------------------------------------------------------------- mags

def draw_mag(d, rng, sid):
    if sid == "50beowulf_ar15mag":
        body, x0, y0, curve = ALUMINUM, 11, 4, 3
        rounds = 4
    else:  # stanag30_beowulf: full 30-round STANAG body, darker, beveled lip
        body, x0, y0, curve = shade(ALUMINUM, 0.8), 11, 3, 5
        rounds = 5
    # curved box: horizontal bands stepping right as they descend
    for i in range(18):
        y = y0 + i
        off = int(curve * (i / 18.0) ** 1.5)
        d.rectangle([x0 + off, y, x0 + off + 9, y], fill=body)
        if i % 4 == 3:
            d.line([x0 + off, y, x0 + off + 9, y], fill=shade(body, 0.82),
                   width=1)
    # feed lips at top
    d.rectangle([x0, y0 - 2, x0 + 8, y0], fill=shade(body, 0.9))
    if sid == "stanag30_beowulf":  # beveled front lip
        d.polygon([(x0 + 6, y0 - 2), (x0 + 9, y0 - 2), (x0 + 8, y0)],
                  fill=shade(body, 0.7))
    # window with fat .50 cartridges stacked single-file
    wx = x0 + curve // 2 + 2
    for r in range(rounds):
        cy = y0 + 4 + r * 3
        off = int(curve * ((cy - y0) / 18.0) ** 1.5)
        d.ellipse([wx + off - curve // 2, cy, wx + off - curve // 2 + 5,
                   cy + 2], fill=BRASS)
        d.point((wx + off - curve // 2 + 1, cy), fill=shade(BRASS, 1.2))
    # baseplate
    bx = int(curve * 1.0)
    d.rectangle([x0 + bx - 1, y0 + 18, x0 + bx + 10, y0 + 19],
                fill=shade(body, 0.75))


# ------------------------------------------------------------- mortar shell

def draw_mortar(d, rng, sid):
    modern = sid.endswith("m768")
    body = OLIVE_DARK if modern else OLIVE
    cx = 16
    # fuze
    d.rectangle([cx - 1, 2, cx + 1, 5], fill=STEEL_DARK)
    d.point((cx, 1), fill=STEEL_LIGHT)
    # teardrop body
    for i in range(15):
        y = 5 + i
        w = int(2 + 4.5 * math.sin(math.pi * min(1.0, i / 11.0)))
        if i > 11:
            w = max(2, 6 - (i - 11))
        d.rectangle([cx - w, y, cx + w, y], fill=body)
        d.point((cx - w + 1, y), fill=shade(body, 1.18))
    # band: yellow HE band (old) vs red-brown band (modern)
    band = (226, 190, 60, 255) if not modern else (168, 74, 52, 255)
    d.line([cx - 5, 11, cx + 5, 11], fill=band, width=1)
    d.line([cx - 5, 12, cx + 5, 12], fill=band, width=1)
    # tail boom + fins
    d.rectangle([cx - 1, 20, cx + 1, 26], fill=STEEL_DARK)
    d.polygon([(cx - 1, 24), (cx - 6, 29), (cx - 1, 29)], fill=STEEL)
    d.polygon([(cx + 1, 24), (cx + 6, 29), (cx + 1, 29)], fill=STEEL)
    d.rectangle([cx - 1, 27, cx + 1, 29], fill=STEEL)


# ------------------------------------------------------------------- LIXA

def draw_lixa(d, rng, sid):
    cx, cy = 16, 15
    if sid == "LIXA_spit_light":
        # glob of congealed light: bright core, halo points, sparkles
        for r, col in ((9, (214, 196, 110, 255)), (7, (240, 224, 140, 255)),
                       (5, GLOW), (3, (255, 252, 230, 255))):
            d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=col)
        for dx, dy in ((-11, -6), (11, -8), (-12, 5), (12, 6), (0, -12),
                       (-7, 11), (8, 11)):
            d.point((cx + dx, cy + dy), fill=GLOW)
            d.point((cx + dx, cy + dy - 1), fill=(230, 214, 130, 255))
        # drips of light congealing below
        d.point((cx - 3, cy + 8), fill=GLOW)
        d.point((cx + 2, cy + 9), fill=GLOW)
        d.point((cx + 2, cy + 10), fill=(230, 214, 130, 255))
    else:
        # coordinate rotation volley: axes, rotation arcs, matrix grid
        ring = (150, 110, 220, 255)
        for r in (11, 8):
            d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=ring, width=1)
        d.arc([cx - 11, cy - 11, cx + 11, cy + 11], start=300, end=80,
              fill=CYAN, width=2)
        d.arc([cx - 8, cy - 8, cx + 8, cy + 8], start=120, end=260,
              fill=CYAN, width=2)
        # arrowheads on the arcs
        d.polygon([(cx + 10, cy - 7), (cx + 13, cy - 4), (cx + 8, cy - 3)],
                  fill=CYAN)
        d.polygon([(cx - 10, cy + 7), (cx - 13, cy + 4), (cx - 8, cy + 3)],
                  fill=CYAN)
        # coordinate axes through the centre
        d.line([cx - 6, cy, cx + 6, cy], fill=GLOW, width=1)
        d.line([cx, cy - 6, cx, cy + 6], fill=GLOW, width=1)
        d.point((cx, cy), fill=(255, 252, 230, 255))
        # matrix dots in the corners
        for gx in (3, 6):
            for gy in (3, 6, 24, 27):
                d.point((gx, gy), fill=ring)
                d.point((31 - gx, gy), fill=ring)


# --------------------------------------------------------------- clothing

def draw_abaya(d, rng):
    # loose full-length black robe, wide sleeves, gentle folds
    d.polygon([(13, 3), (19, 3), (21, 6), (26, 16), (24, 17), (22, 8),
               (24, 29), (8, 29), (10, 8), (8, 17), (6, 16), (11, 6)],
              fill=BLACK_CLOTH)
    d.ellipse([13, 2, 19, 6], fill=BLACK_CLOTH)  # head opening drape
    for fx in (12, 16, 20):  # fold shading
        d.line([fx, 10, fx - 1, 28], fill=CLOTH_SHADE, width=1)
    d.line([16, 6, 16, 28], fill=shade(BLACK_CLOTH, 1.6), width=1)


def draw_ninja(d, rng):
    # black wrap jacket: diagonal wrap, sash, two gauntlets below
    d.rectangle([9, 5, 23, 20], fill=BLACK_CLOTH)
    d.polygon([(9, 5), (17, 5), (9, 14)], fill=CLOTH_SHADE)  # wrap flap
    d.line([9, 14, 23, 5], fill=shade(BLACK_CLOTH, 1.7), width=1)
    d.rectangle([7, 7, 9, 15], fill=BLACK_CLOTH)   # sleeves
    d.rectangle([23, 7, 25, 15], fill=BLACK_CLOTH)
    d.rectangle([9, 17, 23, 19], fill=(120, 40, 40, 255))  # sash
    d.rectangle([20, 17, 23, 21], fill=(120, 40, 40, 255))  # sash tail
    # gauntlets
    for gx in (10, 19):
        d.rectangle([gx, 23, gx + 3, 28], fill=CLOTH_SHADE)
        d.line([gx, 25, gx + 3, 25], fill=BLACK_CLOTH, width=1)


def draw_smoking(d, rng, sid):
    if sid == "advanced_ecig":
        # sleek steel pen: body, tank window, glowing tip
        d.rectangle([14, 6, 17, 24], fill=STEEL)
        d.rectangle([14, 6, 17, 9], fill=STEEL_DARK)          # mouthpiece
        d.rectangle([15, 12, 16, 17], fill=(214, 170, 90, 255))  # liquid
        d.line([14, 12, 14, 24], fill=STEEL_LIGHT, width=1)
        d.rectangle([14, 24, 17, 26], fill=STEEL_DARK)
        d.point((15, 27), fill=CYAN)                          # LED
        d.point((16, 27), fill=CYAN)
        # faint vapour curl
        d.point((13, 4), fill=(210, 216, 222, 255))
        d.point((12, 3), fill=(210, 216, 222, 255))
        d.point((13, 2), fill=(222, 228, 234, 255))
    else:  # crackpipe: glass tube with a bulb bowl
        d.ellipse([6, 12, 16, 22], fill=GLASS)                # bulb
        d.ellipse([8, 14, 14, 20], fill=GLASS_SHADE)
        d.point((10, 13), fill=INK)                           # bowl hole
        d.rectangle([15, 15, 27, 17], fill=GLASS)             # stem
        d.line([15, 15, 27, 15], fill=shade(GLASS, 1.1), width=1)
        d.rectangle([27, 14, 28, 18], fill=GLASS_SHADE)       # mouthpiece
        d.point((9, 19), fill=(120, 90, 70, 255))             # residue
        d.point((11, 20), fill=(120, 90, 70, 255))


# ------------------------------------------------------------ alien fruit

def draw_alien_fruit(d, rng, sid):
    if sid == "alien_meadow_bush_2_fruit":
        # handful of white berries on thin stems
        stems = [(10, 20), (16, 22), (22, 20), (13, 14), (19, 14), (16, 9)]
        for sx, sy in stems:
            d.line([16, 26, sx, sy], fill=(96, 130, 74, 255), width=1)
        for sx, sy in stems:
            d.ellipse([sx - 3, sy - 3, sx + 3, sy + 3],
                      fill=(240, 240, 234, 255))
            d.point((sx - 1, sy - 1), fill=(255, 255, 250, 255))
            d.point((sx + 1, sy + 2), fill=(206, 210, 190, 255))
        d.ellipse([13, 24, 19, 28], fill=(110, 148, 84, 255))  # leaf
    else:
        # large hard-rinded fruit: mottled ridged oval
        d.ellipse([7, 5, 25, 27], fill=(122, 96, 148, 255))
        for i in range(5):  # rind ridges
            x = 10 + i * 3
            d.arc([x - 3, 6, x + 3, 26], start=270, end=90,
                  fill=(96, 72, 122, 255), width=1)
        for _ in range(10):  # mottling
            mx = rng.randint(9, 22)
            my = rng.randint(7, 24)
            d.point((mx, my), fill=(150, 178, 96, 255))
        d.rectangle([15, 3, 17, 5], fill=WOOD_DARK)            # stem
        d.point((11, 9), fill=(168, 140, 190, 255))           # highlight


# ---------------------------------------------------------------- presses

def draw_press(d, rng, sid):
    if sid == "arbor_press":
        # iron C-frame press: base, column, ram, lever with ball ends
        d.rectangle([8, 26, 24, 28], fill=STEEL_DARK)          # base
        d.rectangle([8, 6, 12, 26], fill=STEEL_DARK)           # column
        d.rectangle([8, 6, 22, 9], fill=STEEL_DARK)            # top arm
        d.rectangle([18, 9, 21, 17], fill=STEEL)               # ram
        d.rectangle([17, 17, 22, 19], fill=STEEL_DARK)         # ram face
        d.rectangle([11, 21, 24, 23], fill=shade(STEEL_DARK, 1.2))  # table
        d.line([12, 8, 26, 3], fill=INK, width=2)              # lever
        d.ellipse([25, 1, 28, 4], fill=(168, 52, 44, 255))     # lever ball
        d.ellipse([5, 12, 8, 15], fill=STEEL)                  # pinion hub
        d.point((20, 13), fill=STEEL_LIGHT)
    else:
        # makeshift shotshell press: plank with hole, nail, whittled dowel
        d.rectangle([5, 14, 27, 19], fill=WOOD)                # plank
        d.line([5, 14, 27, 14], fill=shade(WOOD, 1.25), width=1)
        d.ellipse([13, 15, 19, 18], fill=WOOD_DARK)            # hole
        d.ellipse([14, 15, 18, 17], fill=(30, 24, 18, 255))
        # nail standing beside the plank
        d.line([9, 6, 9, 12], fill=STEEL_LIGHT, width=1)
        d.rectangle([7, 5, 11, 6], fill=STEEL)
        # whittled dowel lying below
        d.polygon([(8, 23), (24, 23), (26, 25), (24, 27), (8, 27),
                   (6, 25)], fill=shade(WOOD, 1.3))
        d.line([10, 25, 22, 25], fill=WOOD, width=1)


# -------------------------------------------------------------- pauldrons

def draw_pauldrons(d, rng, sid):
    hardened = sid.startswith("ch_")
    metal = CH_STEEL if hardened else LC_STEEL
    # two large half-cylinder shoulder plates, two smaller beneath
    for cx, cy, w, h in ((10, 9, 11, 9), (22, 9, 11, 9),
                         (10, 20, 9, 7), (22, 20, 9, 7)):
        d.pieslice([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
                   start=180, end=360, fill=metal)
        d.rectangle([cx - w // 2, cy, cx + w // 2, cy + h // 2], fill=metal)
        d.arc([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
              start=180, end=360, fill=shade(metal, 1.25), width=1)
        d.line([cx - w // 2, cy + h // 2, cx + w // 2, cy + h // 2],
               fill=shade(metal, 0.7), width=1)
        d.point((cx - 2, cy + 1), fill=shade(metal, 0.6))      # rivets
        d.point((cx + 2, cy + 1), fill=shade(metal, 0.6))
        if hardened:
            d.point((cx, cy - 2), fill=shade(metal, 1.4))


# ------------------------------------------------------------- cardboard

def draw_cardboard(d, rng, sid):
    if sid == "armor_cardboard_cuirass":
        # torso cuirass of layered cardboard, tape seams, corrugation
        d.polygon([(10, 5), (22, 5), (24, 10), (23, 26), (9, 26), (8, 10)],
                  fill=CARDBOARD)
        d.ellipse([13, 3, 19, 7], fill=CARDBOARD_DARK)         # neck cut
        for y in range(9, 26, 3):  # corrugation lines
            d.line([9, y, 23, y], fill=CARDBOARD_DARK, width=1)
        d.rectangle([15, 5, 17, 26], fill=(206, 188, 150, 255))  # tape seam
        d.line([16, 5, 16, 26], fill=CARDBOARD_DARK, width=1)
        d.rectangle([8, 12, 10, 18], fill=CARDBOARD_DARK)      # strap tabs
        d.rectangle([22, 12, 24, 18], fill=CARDBOARD_DARK)
    else:
        # cardboard badge: shield with shrinking scribbled letters
        d.polygon([(10, 6), (22, 6), (22, 16), (16, 24), (10, 16)],
                  fill=CARDBOARD)
        d.polygon([(11, 7), (21, 7), (21, 15), (16, 22), (11, 15)],
                  outline=CARDBOARD_DARK)
        # letter rows dwindling in size down the badge
        for row, (y, w) in enumerate(((9, 8), (13, 6), (16, 4))):
            x = 12
            for _ in range(3):
                d.rectangle([x, y, x + w // 3, y + 2 - (1 if row == 2 else 0)],
                            fill=INK)
                x += w // 3 + 1
        d.point((16, 4), fill=(168, 52, 44, 255))              # pin dot


# --------------------------------------------------------------- ballista

def draw_ballista(d, rng, sid):
    if sid == "ballista":
        # carroballista: frame, two torsion prods, string, crank, bolt
        d.rectangle([4, 20, 28, 22], fill=WOOD_DARK)           # frame bed
        d.rectangle([6, 22, 8, 27], fill=WOOD_DARK)            # legs
        d.rectangle([24, 22, 26, 27], fill=WOOD_DARK)
        d.rectangle([14, 12, 18, 20], fill=WOOD)               # slider
        # torsion skeins + prods angled forward
        d.rectangle([5, 9, 9, 15], fill=LEATHER_DARK)
        d.rectangle([23, 9, 27, 15], fill=LEATHER_DARK)
        d.line([7, 9, 3, 4], fill=WOOD, width=2)
        d.line([25, 9, 29, 4], fill=WOOD, width=2)
        d.line([3, 4, 16, 13], fill=(220, 214, 200, 255), width=1)  # string
        d.line([29, 4, 16, 13], fill=(220, 214, 200, 255), width=1)
        # loaded bolt along the trough
        d.line([16, 6, 16, 19], fill=shade(WOOD, 1.3), width=2)
        d.polygon([(14, 6), (18, 6), (16, 2)], fill=STEEL_LIGHT)
        # hand crank at the rear
        d.ellipse([24, 23, 29, 28], outline=STEEL_DARK, width=1)
        d.point((26, 25), fill=STEEL_DARK)
    else:
        # big bolt: thick shaft, steel tip, shaft-protrusion fletching
        d.line([16, 10, 16, 27], fill=WOOD, width=3)
        d.line([15, 12, 15, 26], fill=shade(WOOD, 1.25), width=1)
        d.polygon([(13, 10), (19, 10), (16, 2)], fill=STEEL)   # tip
        d.line([16, 3, 16, 9], fill=STEEL_LIGHT, width=1)
        # base thicker + protrusion fletching
        d.rectangle([14, 22, 18, 28], fill=WOOD_DARK)
        d.polygon([(14, 23), (10, 27), (14, 27)], fill=WOOD)
        d.polygon([(18, 23), (22, 27), (18, 27)], fill=WOOD)


# ----------------------------------------------------------------- barbs

def draw_barb(d, rng, sid):
    gene = sid.startswith("gene")
    # curved bone barb with sponge tissue at the base, venom droplets
    d.polygon([(13, 26), (15, 12), (18, 4), (20, 3), (19, 10), (18, 16),
               (17, 26)], fill=BONE)
    d.line([15, 24, 16, 10], fill=BONE_SHADE, width=1)
    d.ellipse([12, 24, 20, 29], fill=BONE_SHADE)               # sponge base
    drop = CYAN if gene else (232, 196, 92, 255)
    for dx, dy in ((11, 27), (20, 28), (16, 29)):
        d.point((dx, dy), fill=drop)
    d.point((19, 6), fill=drop)                                # venom bead


# ------------------------------------------------------------ noise items

def draw_noise(d, rng, sid):
    if sid == "beeper":
        # vehicle back-up beeper: plastic box, speaker grille, amber lens
        d.rectangle([9, 10, 23, 22], fill=(70, 70, 76, 255))
        d.rectangle([9, 10, 23, 12], fill=(88, 88, 94, 255))
        for gx in range(12, 21, 3):
            for gy in range(15, 20, 2):
                d.point((gx, gy), fill=INK)
        d.rectangle([19, 18, 22, 21], fill=(240, 160, 50, 255))  # lens
        # sound arcs
        d.arc([22, 8, 30, 20], start=300, end=60, fill=(240, 160, 50, 255),
              width=1)
        d.arc([3, 8, 10, 20], start=120, end=240, fill=(240, 160, 50, 255),
              width=1)
        d.rectangle([13, 22, 19, 24], fill=STEEL_DARK)           # mount
    else:
        # chime loudspeakers: stereo box, two speaker cones, music notes
        d.rectangle([5, 12, 27, 24], fill=(96, 66, 44, 255))
        for cx in (11, 21):
            d.ellipse([cx - 4, 14, cx + 4, 22], fill=INK)
            d.ellipse([cx - 2, 16, cx + 2, 20], fill=(150, 140, 128, 255))
            d.point((cx, 18), fill=(200, 190, 175, 255))
        # musical notes above
        for nx, ny in ((9, 5), (16, 3), (23, 5)):
            d.ellipse([nx - 1, ny + 3, nx + 2, ny + 5],
                      fill=(240, 200, 80, 255))
            d.line([nx + 2, ny + 4, nx + 2, ny], fill=(240, 200, 80, 255),
                   width=1)
            d.point((nx + 3, ny), fill=(240, 200, 80, 255))


# ------------------------------------------------------------- bio weapons

def draw_bio(d, rng, sid):
    alloy = (196, 206, 216, 255)
    if sid == "bio_blade_weapon":
        # foot-long monomolecular blade, glowing nanocrystal edge
        d.polygon([(10, 24), (13, 8), (20, 3), (22, 2), (19, 10),
                   (14, 24)], fill=alloy)
        d.line([13, 9, 20, 3], fill=CYAN, width=1)             # edge glow
        d.line([14, 10, 19, 5], fill=CYAN, width=1)
        d.rectangle([9, 24, 15, 27], fill=INK)                 # grip
        d.point((21, 2), fill=(220, 250, 252, 255))
    else:
        # bionic claws: three short curved claws from a knuckle plate
        d.rectangle([9, 20, 23, 25], fill=STEEL_DARK)          # knuckle base
        for i, cx in enumerate((11, 16, 21)):
            tip_y = 4 + (2 if i == 1 else 0)
            d.polygon([(cx - 2, 20), (cx - 1, 10), (cx + 1, tip_y),
                       (cx + 2, 10), (cx + 2, 20)], fill=alloy)
            d.line([cx + 1, tip_y + 2, cx + 1, 18], fill=CYAN, width=1)
        d.point((12, 22), fill=CYAN)
        d.point((20, 22), fill=CYAN)


# ------------------------------------------------------------------- bile

def draw_bile(d, rng, sid):
    adv = sid.endswith("_adv")
    liquid = BILE_ADV if adv else BILE
    # squat jar of boomer bile
    d.rectangle([11, 8, 21, 9], fill=STEEL_DARK)               # lid
    d.rectangle([10, 10, 22, 26], fill=(206, 214, 200, 255))   # jar glass
    d.rectangle([11, 13, 21, 25], fill=liquid)
    for _ in range(7 if adv else 4):                           # bubbles
        bx = rng.randint(12, 20)
        by = rng.randint(14, 23)
        d.point((bx, by), fill=shade(liquid, 1.3))
    d.point((13, 15), fill=shade(liquid, 0.75))
    d.point((18, 21), fill=shade(liquid, 0.75))
    if adv:  # refined: biohazard-ish stripe + brighter glow flecks
        d.line([11, 11, 21, 11], fill=(240, 200, 60, 255), width=1)
        d.point((16, 17), fill=(230, 255, 190, 255))
    d.line([11, 12, 11, 24], fill=(240, 244, 238, 255), width=1)


# ---------------------------------------------------------------- bottles

def draw_bottle(d, rng, sid):
    if sid == "bottle_folding":
        # collapsed accordion plastic bottle, flattened for storage
        d.rectangle([14, 4, 18, 7], fill=(120, 170, 210, 255))  # cap
        for i in range(5):
            y = 8 + i * 4
            w = 6 - (i % 2) * 2
            d.rectangle([16 - w, y, 16 + w, y + 3],
                        fill=(140, 190, 220, 255) if i % 2 == 0
                        else (110, 160, 196, 255))
        d.rectangle([11, 27, 21, 28], fill=(110, 160, 196, 255))
    else:
        # stainless steel bottle
        d.rectangle([14, 3, 18, 6], fill=STEEL_DARK)            # cap
        d.rectangle([13, 6, 19, 8], fill=STEEL)
        d.rectangle([11, 8, 21, 27], fill=STEEL)
        d.line([13, 9, 13, 26], fill=STEEL_LIGHT, width=1)
        d.line([19, 9, 19, 26], fill=STEEL_DARK, width=1)
        d.rectangle([11, 24, 21, 27], fill=shade(STEEL, 0.9))   # base band


# ------------------------------------------------------------ small stuff

def draw_nails(d, rng, sid):
    if sid == "bronze_nail":
        bronze = (178, 128, 66, 255)
        for x, y, ln in ((11, 8, 15), (16, 6, 17), (21, 9, 14)):
            d.line([x, y + 2, x, y + ln], fill=bronze, width=2)
            d.rectangle([x - 2, y, x + 2, y + 2], fill=shade(bronze, 1.15))
            d.point((x, y + ln + 1), fill=shade(bronze, 0.8))
    else:
        # a small pile of steel rivets: domed heads with short shanks
        for x, y in ((10, 12), (16, 10), (22, 12), (13, 19), (19, 19),
                     (16, 25)):
            d.pieslice([x - 3, y - 2, x + 3, y + 3], start=180, end=360,
                       fill=STEEL)
            d.rectangle([x - 1, y + 2, x + 1, y + 4], fill=STEEL_DARK)
            d.point((x - 1, y - 1), fill=STEEL_LIGHT)


def draw_whip(d, rng, sid):
    if sid == "bullwhip":
        # coiled braided leather whip with a handle
        for r in (9, 6):
            d.ellipse([16 - r, 14 - r // 2, 16 + r, 14 + r], outline=LEATHER,
                      width=2)
        d.ellipse([13, 11, 19, 17], outline=LEATHER_DARK, width=2)
        d.line([22, 16, 27, 24], fill=LEATHER, width=2)        # fall
        d.point((28, 25), fill=LEATHER_DARK)                   # cracker
        d.rectangle([6, 20, 9, 28], fill=WOOD_DARK)            # handle
        d.line([7, 21, 7, 27], fill=LEATHER, width=1)
    else:
        # scourge: handle with nine tails, each tipped with steel razors
        d.rectangle([14, 3, 18, 11], fill=WOOD_DARK)
        d.line([15, 4, 15, 10], fill=LEATHER, width=1)
        for i in range(9):
            x0 = 10 + i * 1.5
            sway = (i % 3) - 1
            pts = [(16, 11), (x0 + sway, 18), (x0 + sway * 2, 24)]
            d.line(pts, fill=LEATHER, width=1)
            tx, ty = pts[-1]
            d.polygon([(tx - 1, ty), (tx + 1, ty), (tx, ty + 3)],
                      fill=STEEL_LIGHT)                        # razor tip


def draw_fishspear(d, rng, sid):
    carbon = sid.startswith("c_")
    shaft = (52, 52, 58, 255) if carbon else (140, 144, 150, 255)
    # long pole spear angled across the frame, pronged steel tip
    d.line([7, 26, 22, 8], fill=shaft, width=2)
    d.line([22, 8, 26, 3], fill=STEEL_LIGHT, width=1)          # centre prong
    d.line([21, 9, 22, 4], fill=STEEL_LIGHT, width=1)
    d.line([23, 7, 27, 5], fill=STEEL_LIGHT, width=1)
    d.point((26, 3), fill=INK)                                 # barb
    if carbon:
        for t in (0.3, 0.55, 0.8):  # carbon weave bands
            x = int(7 + 15 * t)
            y = int(26 - 18 * t)
            d.point((x, y), fill=(90, 90, 98, 255))
    else:
        d.line([9, 24, 11, 22], fill=STEEL_DARK, width=1)      # grip wrap
        d.line([11, 22, 13, 20], fill=STEEL_DARK, width=1)
    d.rectangle([5, 26, 8, 28], fill=INK)                      # butt cap


# -------------------------------------------------------------- pet gear

def draw_pet(d, rng, sid):
    if sid == "chicken_cage":
        # wire cage: mesh box with a door and handle
        d.rectangle([6, 10, 26, 26], outline=STEEL_DARK, width=1)
        for gx in range(9, 26, 3):
            d.line([gx, 10, gx, 26], fill=STEEL, width=1)
        for gy in range(13, 26, 3):
            d.line([6, gy, 26, gy], fill=STEEL, width=1)
        d.rectangle([13, 14, 19, 22], outline=STEEL_DARK, width=1)  # door
        d.point((18, 18), fill=INK)                                 # latch
        d.arc([12, 5, 20, 11], start=180, end=360, fill=STEEL_DARK,
              width=1)                                              # handle
        d.rectangle([6, 26, 26, 27], fill=STEEL_DARK)
    else:
        # horse tack: saddle with bridle straps
        d.pieslice([7, 8, 25, 22], start=180, end=360, fill=LEATHER)
        d.rectangle([7, 14, 25, 17], fill=LEATHER)
        d.rectangle([13, 11, 19, 14], fill=LEATHER_DARK)       # seat
        d.rectangle([14, 17, 18, 24], fill=LEATHER_DARK)       # girth
        d.point((16, 20), fill=STEEL_LIGHT)                    # buckle
        # bridle: headstall straps + bit rings, hanging at the side
        d.line([24, 10, 28, 16], fill=LEATHER_DARK, width=1)
        d.line([28, 16, 25, 22], fill=LEATHER_DARK, width=1)
        d.ellipse([23, 21, 26, 24], outline=STEEL, width=1)
        d.line([10, 17, 6, 24], fill=LEATHER_DARK, width=1)    # reins
        d.line([22, 17, 26, 24], fill=LEATHER_DARK, width=1)


# ------------------------------------------------------------- exoskeleton

def draw_exo(d, rng, sid):
    on = sid.endswith("_on")
    frame = (104, 110, 118, 255)
    suit = (74, 80, 74, 255) if not on else (84, 92, 84, 255)
    # torso suit under a mechanical frame: shoulders, spine, hip belt
    d.rectangle([10, 8, 22, 24], fill=suit)
    d.rectangle([8, 8, 24, 11], fill=frame)                    # shoulder bar
    d.rectangle([15, 8, 17, 24], fill=frame)                   # spine strut
    d.rectangle([9, 21, 23, 24], fill=frame)                   # hip belt
    # arm struts with piston segments
    for x0 in (5, 24):
        d.rectangle([x0, 10, x0 + 3, 17], fill=frame)
        d.rectangle([x0 + 1, 17, x0 + 2, 22], fill=STEEL_LIGHT)  # piston
        d.rectangle([x0, 22, x0 + 3, 26], fill=frame)
    # leg struts
    for x0 in (11, 19):
        d.rectangle([x0, 24, x0 + 2, 29], fill=frame)
    # chest control box
    d.rectangle([13, 13, 19, 17], fill=STEEL_DARK)
    if on:
        d.point((14, 14), fill=(120, 255, 140, 255))          # power LEDs
        d.point((16, 14), fill=CYAN)
        d.point((18, 14), fill=(120, 255, 140, 255))
        d.line([13, 16, 19, 16], fill=CYAN, width=1)           # lit display
        d.point((6, 12), fill=CYAN)                           # joint lights
        d.point((25, 12), fill=CYAN)
        d.point((16, 20), fill=(120, 255, 140, 255))
    else:
        d.point((14, 14), fill=(70, 76, 82, 255))
        d.point((18, 14), fill=(70, 76, 82, 255))
        d.line([13, 16, 19, 16], fill=(70, 76, 82, 255), width=1)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in ("50beowulf_ar15mag", "stanag30_beowulf"):
        draw_mag(d, rng, sid)
    elif sid in ("60mm_shell_m720a1", "60mm_shell_m768"):
        draw_mortar(d, rng, sid)
    elif sid in ("LIXA_spit_light", "LIXA_vector_volley"):
        draw_lixa(d, rng, sid)
    elif sid == "abaya":
        draw_abaya(d, rng)
    elif sid == "jacket_ninja":
        draw_ninja(d, rng)
    elif sid in ("advanced_ecig", "crackpipe"):
        draw_smoking(d, rng, sid)
    elif sid in ("alien_meadow_bush_2_fruit", "alien_meadow_tree_3_fruit"):
        draw_alien_fruit(d, rng, sid)
    elif sid in ("arbor_press", "press_dowel"):
        draw_press(d, rng, sid)
    elif sid in ("armguard_metal_sheets_shoulders",
                 "ch_armguard_metal_sheets_shoulders"):
        draw_pauldrons(d, rng, sid)
    elif sid in ("armor_cardboard_cuirass", "badge_cardboard"):
        draw_cardboard(d, rng, sid)
    elif sid in ("ballista", "bolt_ballista_steel"):
        draw_ballista(d, rng, sid)
    elif sid in ("barb_paralysis", "gene_sting_barb"):
        draw_barb(d, rng, sid)
    elif sid in ("beeper", "chimes"):
        draw_noise(d, rng, sid)
    elif sid in ("bio_blade_weapon", "bio_claws_weapon"):
        draw_bio(d, rng, sid)
    elif sid in ("boomer_bile", "boomer_bile_adv"):
        draw_bile(d, rng, sid)
    elif sid in ("bottle_folding", "bottle_metal"):
        draw_bottle(d, rng, sid)
    elif sid in ("bronze_nail", "rivets"):
        draw_nails(d, rng, sid)
    elif sid in ("bullwhip", "bullwhip_razor"):
        draw_whip(d, rng, sid)
    elif sid in ("c_fishspear", "m_fishspear"):
        draw_fishspear(d, rng, sid)
    elif sid in ("chicken_cage", "horse_tack"):
        draw_pet(d, rng, sid)
    elif sid in ("combat_exoskeleton_medium",
                 "combat_exoskeleton_medium_on"):
        draw_exo(d, rng, sid)
    elif sid.startswith(("bp_", "reloaded_")):
        draw_ammo(d, rng, sid)
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
                       "preview_item88.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item88.png {prev.size}")


if __name__ == "__main__":
    main()
