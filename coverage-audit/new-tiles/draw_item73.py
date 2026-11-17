#!/usr/bin/env python3
"""ITEM73: makeshift medical, camp furniture/stoves, solar gear and a
gun tail -- CDDA Voxel coverage audit.

Families (48 ids):

  Makeshift medical (10) -- cloth/medical colours.  The three bandage
    rolls share a roll-with-tail form: plain is a loose frayed roll,
    bleached is tight, bright white with a blue-white sheen, boiled
    is darker and damp with sheen streaks and droplets.  Cotton balls
    are a fluffy cluster, the disinrag a draped damp cloth, the liquid
    bandage a tiny atomizer spray bottle with a red cross, the
    smoxygen kit a small steel tank with regulator and mask, the
    vacutainer a capped blood tube, the rad badge a trefoil badge
    sealed in a crimped plastic bag.
  Camp furniture and stoves (10) -- folded deployables drawn folded:
    folding chair (plastic/steel X-frame), deck chair (wood slats +
    canvas), hobo stove (punched tin can), makeshift brazier (punched
    steel bucket), butcher rack (top bar + hooks), smoking rack
    (tiered shelves), portable stove (box + chimney), screw press
    (wood posts + big screw), solar cooker (faceted dish on tripod),
    spinner target (oar paddles on a frame).
  Solar gear (10) -- standard cells are deep blue with a coarse grid;
    v2 monocrystalline cells are darker navy with fine silver bus
    lines.  Folding panels read folded (slim, seamed, handled) vs
    deployed (spread wide); the v2 folding panel is a suitcase.
    Ground arrays are 2x2; reinforced panels carry a thicker frame,
    corner bolts and glass sheen streaks; solar_cell_v2 is a bare
    octagonal wafer.
  Gun tail (18) -- 20x66 box magazines sized 10/20/40, a ribbed black
    Saiga-410 mag with a shell at the lips, a .454 speedloader disc
    with six brass cartridge heads, a rail-mounted crossbow kit, the
    KSG-25 tube magazine, the RM121 aux shotgun and its modified
    variant (extra straps, full rail, foregrip), and nine .30-06
    cartridges: tip by load (JSP flat lead nose, FMJ full copper,
    M2 AP black tip, tracer orange tip) and case by provenance
    (factory bright brass, black-powder dull with a soot band,
    reloaded tarnished and scratched).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "bandages_makeshift",
    "bandages_makeshift_bleached",
    "bandages_makeshift_boiled",
    "dental_tools",
    "disincotton_ball",
    "disinrag",
    "liq_bandage_spray",
    "smoxygen_tank",
    "vacutainer",
    "wrapped_rad_badge",
    "chair_folding",
    "deck_chair",
    "hobo_stove",
    "makeshift_brazier",
    "metal_butcher_rack",
    "metal_smoking_rack",
    "portable_stove",
    "screw_press_tool",
    "solar_cooker",
    "target_spinner",
    "folding_solar_panel",
    "folding_solar_panel_deployed",
    "folding_solar_panel_v2",
    "folding_solar_panel_v2_deployed",
    "ground_solar_panel",
    "ground_solar_panel_v2",
    "reinforced_solar_panel",
    "reinforced_solar_panel_v2",
    "solar_cell_v2",
    "solar_panel_v2",
    "20x66_10_mag",
    "20x66_20_mag",
    "20x66_40_mag",
    "454_speedloader6",
    "gun_crossbow",
    "ksg25_aux_shotgun",
    "rm121aux",
    "rm121aux_mod",
    "saiga410mag_10rd",
    "3006_jsp",
    "bp_3006",
    "bp_3006_incendiary",
    "bp_3006_jsp",
    "bp_3006fmj",
    "reloaded_3006",
    "reloaded_3006_incendiary",
    "reloaded_3006_jsp",
    "reloaded_3006fmj",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 44, 50, 255)
STEEL = (150, 154, 160, 255)
STEEL_DARK = (94, 98, 104, 255)
STEEL_LIGHT = (190, 194, 200, 255)
PLASTIC_DARK = (58, 60, 66, 255)
WOOD = (124, 86, 48, 255)
WOOD_DARK = (88, 58, 30, 255)
RED = (196, 48, 40, 255)
BRASS = (198, 158, 74, 255)
BRASS_DULL = (166, 130, 64, 255)
COPPER = (178, 112, 62, 255)
LEAD = (126, 126, 132, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- medical

def bandage_roll(d, cloth, edge, style):
    """Cloth roll seen from the side, loose tail trailing right."""
    # tail first (behind the roll)
    d.polygon([(16, 19), (28, 23), (28, 26), (15, 23)], fill=cloth)
    d.line([16, 19, 28, 23], fill=edge, width=1)
    d.line([15, 23, 28, 26], fill=edge, width=1)
    if style == "plain":  # frayed tail end
        for fy in (23, 24, 25):
            d.line([28, fy, 30, fy], fill=edge, width=1)
    d.ellipse([6, 7, 20, 23], fill=cloth, outline=edge)
    if style == "bleached":  # tight, neat windings
        d.ellipse([9, 10, 17, 20], outline=edge)
        d.ellipse([11, 12, 15, 18], outline=edge)
        d.point((13, 15), fill=edge)
        for px, py in ((8, 9), (18, 10), (10, 21)):  # blue-white sheen
            d.point((px, py), fill=(206, 224, 244, 255))
    elif style == "boiled":  # damp: sheen streaks + droplets below
        d.ellipse([9, 10, 17, 20], outline=edge)
        d.point((13, 15), fill=edge)
        for sx in range(9, 17, 3):
            d.line([sx, 9, sx + 2, 12], fill=(216, 212, 198, 255), width=1)
        for dx, dy in ((10, 27), (15, 28), (21, 27)):
            d.point((dx, dy), fill=(110, 148, 184, 255))
            d.point((dx, dy + 1), fill=(110, 148, 184, 255))
    else:  # plain: one loose outer winding, slightly open
        d.arc([9, 10, 17, 20], start=40, end=330, fill=edge, width=1)
        d.point((13, 15), fill=edge)
        d.line([7, 12, 6, 17], fill=edge, width=1)  # loose flap


def draw_bandage_plain(d, rng):
    bandage_roll(d, (203, 196, 176, 255), (148, 140, 118, 255), "plain")


def draw_bandage_bleached(d, rng):
    bandage_roll(d, (248, 247, 242, 255), (192, 198, 208, 255), "bleached")


def draw_bandage_boiled(d, rng):
    bandage_roll(d, (186, 182, 166, 255), (136, 132, 116, 255), "boiled")


def draw_dental_tools(d, rng):
    # three thin steel tools fanning out of a dark zip pouch
    for x1, y1, kind in ((11, 5, "hook"), (16, 3, "pick"), (21, 5, "flat")):
        d.line([16, 21, x1, y1 + 3], fill=STEEL_LIGHT, width=1)
        if kind == "hook":
            d.line([x1, y1 + 3, x1 + 2, y1 + 1], fill=STEEL_LIGHT, width=1)
        elif kind == "pick":
            d.point((x1, y1 + 2), fill=STEEL_LIGHT)
            d.point((x1, y1 + 1), fill=STEEL_LIGHT)
        else:
            d.line([x1 - 1, y1 + 2, x1 + 1, y1 + 3], fill=STEEL_LIGHT,
                   width=1)
        d.line([16, 21, (16 + x1) // 2, (21 + y1) // 2],
               fill=STEEL_DARK, width=1)  # dark handle midsection
    d.rectangle([6, 20, 26, 28], fill=(70, 72, 78, 255), outline=INK)
    d.line([7, 22, 25, 22], fill=STEEL_DARK, width=1)  # zipper
    d.rectangle([22, 21, 24, 23], fill=STEEL, outline=INK)  # zip pull


def draw_cotton_balls(d, rng):
    balls = [(11, 13, 5), (20, 11, 4), (16, 20, 6), (23, 19, 4), (9, 20, 3)]
    for cx, cy, r in balls:
        d.ellipse([cx - r, cy - r, cx + r, cy + r],
                  fill=(250, 250, 248, 255), outline=(198, 198, 198, 255))
        for bx, by in ((cx - r, cy), (cx + r, cy), (cx, cy - r)):
            d.ellipse([bx - 1, by - 1, bx + 1, by + 1],
                      fill=(250, 250, 248, 255))
    for px, py in ((11, 12), (19, 18), (15, 9)):  # antiseptic sheen
        d.point((px, py), fill=(198, 230, 230, 255))


def draw_disinrag(d, rng):
    d.polygon([(7, 11), (12, 8), (17, 11), (22, 9), (26, 12), (25, 24),
               (19, 26), (12, 25), (8, 24)], fill=(240, 238, 232, 255))
    d.polygon([(7, 11), (12, 8), (17, 11), (22, 9), (26, 12), (25, 24),
               (19, 26), (12, 25), (8, 24)], outline=(168, 164, 156, 255))
    for fx in (12, 17, 21):  # fold lines
        d.line([fx, 11, fx - 1, 24], fill=(206, 202, 192, 255), width=1)
    for sx in range(10, 23, 4):  # damp sheen
        d.line([sx, 13, sx + 2, 16], fill=(248, 248, 246, 255), width=1)


def draw_liq_bandage(d, rng):
    # tiny atomizer spray bottle, white plastic, red-cross label
    d.rectangle([10, 9, 14, 12], fill=STEEL_LIGHT, outline=INK)  # nozzle
    d.rectangle([13, 7, 20, 13], fill=(228, 228, 232, 255), outline=INK)
    d.rectangle([14, 13, 19, 15], fill=(200, 200, 204, 255), outline=INK)
    d.rectangle([12, 15, 21, 28], fill=(244, 244, 246, 255), outline=INK)
    d.line([16, 19, 16, 24], fill=RED, width=2)  # cross
    d.line([14, 21, 19, 21], fill=RED, width=2)
    for px, py in ((7, 8), (6, 10), (8, 11)):  # spray mist dots
        d.point((px, py), fill=(170, 200, 220, 255))


def draw_smoxygen(d, rng):
    # small steel O2 tank + regulator + mask on a tube
    d.rectangle([7, 11, 14, 28], fill=(176, 180, 186, 255), outline=INK)
    d.line([9, 13, 9, 26], fill=STEEL_LIGHT, width=1)
    d.rectangle([9, 7, 12, 11], fill=BRASS_DULL, outline=INK)  # valve
    d.ellipse([12, 5, 16, 9], fill=STEEL_DARK, outline=INK)  # regulator dial
    d.arc([14, 8, 28, 22], start=270, end=90, fill=STEEL_DARK, width=1)
    d.polygon([(21, 15), (28, 13), (28, 23), (21, 24)],
              fill=(202, 216, 222, 255))  # mask
    d.polygon([(21, 15), (28, 13), (28, 23), (21, 24)], outline=INK)
    d.line([22, 18, 27, 18], fill=(170, 186, 194, 255), width=1)


def draw_vacutainer(d, rng):
    # capped blood-draw tube, red cap, blood fill, needle beside
    d.rectangle([12, 7, 20, 11], fill=(178, 44, 44, 255), outline=INK)
    d.rectangle([13, 11, 19, 27], fill=(212, 236, 238, 255), outline=INK)
    d.rectangle([14, 19, 18, 26], fill=(150, 30, 34, 255))  # blood
    d.line([14, 15, 18, 15], fill=(240, 248, 248, 255), width=1)
    d.line([24, 6, 27, 15], fill=STEEL_LIGHT, width=1)  # needle
    d.rectangle([23, 15, 26, 18], fill=(178, 44, 44, 255), outline=INK)


def draw_rad_badge(d, rng):
    # badge sealed in a crimped translucent bag
    d.rectangle([9, 6, 23, 27], fill=(198, 212, 222, 255), outline=
                (118, 140, 156, 255))
    for zx in range(10, 23, 3):  # crimp zigzag at the seal
        d.line([zx, 8, zx + 1, 6], fill=(150, 172, 188, 255), width=1)
    d.rectangle([12, 13, 20, 22], fill=(238, 220, 130, 255), outline=INK)
    d.point((16, 17), fill=INK)  # trefoil
    for dx, dy in ((0, -2), (-2, 1), (2, 1)):
        d.point((16 + dx, 17 + dy), fill=INK)
    d.line([11, 25, 21, 25], fill=(170, 190, 204, 255), width=1)


# ------------------------------------------------------------------- camp

def draw_chair_folding(d, rng):
    d.rectangle([11, 4, 21, 15], fill=(178, 182, 188, 255), outline=INK)
    d.line([12, 6, 20, 6], fill=(150, 154, 160, 255), width=1)
    d.rectangle([11, 15, 21, 17], fill=(140, 144, 150, 255), outline=INK)
    d.line([12, 17, 20, 29], fill=STEEL_DARK, width=1)  # folded X frame
    d.line([20, 17, 12, 29], fill=STEEL_DARK, width=1)
    d.point((16, 23), fill=INK)  # pivot rivet


def draw_deck_chair(d, rng):
    d.polygon([(9, 6), (13, 6), (20, 27), (16, 27)], fill=WOOD)
    d.polygon([(9, 6), (13, 6), (20, 27), (16, 27)], outline=WOOD_DARK)
    d.polygon([(17, 6), (21, 6), (26, 27), (22, 27)], fill=WOOD)
    d.polygon([(17, 6), (21, 6), (26, 27), (22, 27)], outline=WOOD_DARK)
    d.polygon([(12, 14), (24, 14), (25, 19), (13, 19)],
              fill=(150, 110, 70, 255))  # canvas sling band
    d.line([12, 16, 24, 16], fill=(120, 86, 52, 255), width=1)


def draw_hobo_stove(d, rng):
    d.ellipse([10, 7, 22, 12], fill=(58, 56, 54, 255), outline=INK)
    d.rectangle([10, 9, 22, 27], fill=(158, 162, 168, 255), outline=INK)
    d.ellipse([10, 7, 22, 12], outline=INK)
    d.ellipse([13, 8, 19, 11], fill=(30, 28, 27, 255))  # dark opening
    for hy in (15, 19, 23):  # punched air holes
        for hx in range(12, 21, 3):
            d.point((hx + (hy % 2), hy), fill=(88, 88, 92, 255))
    d.rectangle([10, 24, 22, 27], fill=(128, 124, 118, 255))  # scorch band


def draw_brazier(d, rng):
    d.arc([10, 3, 22, 12], start=180, end=360, fill=STEEL_DARK, width=1)
    d.polygon([(9, 10), (23, 10), (21, 27), (11, 27)],
              fill=(150, 154, 160, 255))
    d.polygon([(9, 10), (23, 10), (21, 27), (11, 27)], outline=INK)
    d.line([9, 12, 23, 12], fill=STEEL_DARK, width=1)  # rim
    for hy in (16, 20, 24):
        for hx in range(12, 21, 3):
            d.point((hx, hy), fill=(86, 86, 90, 255))
    d.line([11, 27, 21, 27], fill=INK, width=1)


def draw_butcher_rack(d, rng):
    d.rectangle([8, 5, 10, 27], fill=STEEL, outline=INK)
    d.rectangle([22, 5, 24, 27], fill=STEEL, outline=INK)
    d.rectangle([8, 5, 24, 7], fill=STEEL, outline=INK)  # top bar
    for hx in (12, 16, 20):  # S-hooks
        d.arc([hx - 1, 7, hx + 2, 11], start=0, end=270, fill=INK, width=1)
        d.point((hx, 12), fill=INK)
    d.line([8, 27, 5, 29], fill=STEEL_DARK, width=1)  # feet
    d.line([24, 27, 27, 29], fill=STEEL_DARK, width=1)


def draw_smoking_rack(d, rng):
    d.rectangle([9, 5, 23, 27], outline=STEEL_DARK, width=2)
    for sy in (11, 17, 23):  # tiered shelves with slats
        d.line([10, sy, 22, sy], fill=STEEL, width=1)
        for sx in range(11, 22, 3):
            d.line([sx, sy - 2, sx, sy], fill=STEEL, width=1)
    d.line([10, 26, 22, 6], fill=(120, 124, 130, 255), width=1)  # brace


def draw_portable_stove(d, rng):
    d.rectangle([19, 6, 22, 13], fill=STEEL_DARK, outline=INK)  # chimney
    d.rectangle([8, 13, 24, 14], fill=(128, 130, 136, 255), outline=INK)
    d.rectangle([9, 14, 23, 26], fill=(104, 107, 113, 255), outline=INK)
    d.rectangle([13, 17, 19, 24], outline=STEEL_LIGHT, width=1)  # door
    d.point((18, 20), fill=STEEL_LIGHT)  # latch
    d.line([11, 26, 10, 29], fill=INK, width=1)  # legs
    d.line([21, 26, 22, 29], fill=INK, width=1)


def draw_screw_press(d, rng):
    d.rectangle([7, 5, 10, 28], fill=WOOD, outline=WOOD_DARK)
    d.rectangle([22, 5, 25, 28], fill=WOOD, outline=WOOD_DARK)
    d.rectangle([7, 5, 25, 8], fill=WOOD_DARK, outline=INK)  # top beam
    d.rectangle([15, 8, 17, 19], fill=STEEL_DARK, outline=INK)  # screw
    for ty in range(10, 19, 2):
        d.line([15, ty, 17, ty], fill=STEEL_LIGHT, width=1)  # threads
    d.line([9, 11, 23, 11], fill=WOOD_DARK, width=2)  # handle bar
    d.point((9, 11), fill=INK)
    d.point((23, 11), fill=INK)
    d.rectangle([12, 19, 20, 21], fill=WOOD, outline=WOOD_DARK)  # plate
    d.rectangle([7, 26, 25, 28], fill=WOOD_DARK, outline=INK)  # base


def draw_solar_cooker(d, rng):
    cx, cy = 16, 15
    d.polygon([(5, 9), (27, 9), (22, 18), (10, 18)],
              fill=(202, 206, 212, 255))
    d.polygon([(5, 9), (27, 9), (22, 18), (10, 18)], outline=INK)
    for fx in (11, 16, 21):  # facet seams converging on the focus
        d.line([fx, 9, cx, cy], fill=(150, 154, 160, 255), width=1)
    d.line([7, 12, 25, 12], fill=(170, 174, 180, 255), width=1)
    d.ellipse([13, 10, 19, 14], fill=(40, 40, 44, 255))  # pot at focus
    d.line([11, 18, 9, 28], fill=STEEL_DARK, width=1)  # tripod
    d.line([21, 18, 23, 28], fill=STEEL_DARK, width=1)
    d.line([16, 18, 16, 28], fill=STEEL_DARK, width=1)
    d.point((16, 8), fill=(250, 220, 110, 255))  # sun glint


def draw_target_spinner(d, rng):
    d.line([10, 28, 14, 12], fill=STEEL_DARK, width=1)  # stand
    d.line([22, 28, 18, 12], fill=STEEL_DARK, width=1)
    d.line([7, 28, 25, 28], fill=STEEL_DARK, width=1)
    d.line([16, 4, 16, 21], fill=STEEL, width=2)  # pipe
    d.rectangle([9, 4, 16, 10], fill=STEEL_LIGHT, outline=INK)  # top oar
    d.rectangle([16, 15, 23, 21], fill=(140, 144, 150, 255),
                outline=INK)  # heavier bottom oar
    d.point((16, 12), fill=INK)  # pivot


# ------------------------------------------------------------------ solar

CELL_STD = (40, 60, 130, 255)
GRID_STD = (18, 28, 64, 255)
BUS_STD = (140, 170, 220, 255)
CELL_V2 = (22, 30, 70, 255)
GRID_V2 = (10, 14, 36, 255)
BUS_V2 = (168, 178, 198, 255)


def solar_cells(d, x0, y0, x1, y1, v2=False, glass=False):
    base, grid, bus = (CELL_V2, GRID_V2, BUS_V2) if v2 else \
        (CELL_STD, GRID_STD, BUS_STD)
    d.rectangle([x0, y0, x1, y1], fill=base)
    step = 3 if v2 else 4
    for gx in range(x0 + step, x1, step):
        d.line([gx, y0, gx, y1], fill=grid, width=1)
    for gy in range(y0 + step, y1, step):
        d.line([x0, gy, x1, gy], fill=grid, width=1)
    mid = (y0 + y1) // 2
    d.line([x0, mid, x1, mid], fill=bus, width=1)  # busbar
    if glass:  # reinforced-glass sheen streaks
        for sx, sy in ((x0 + 2, y0 + 2), (x0 + 6, y0 + 1), (x0 + 3, y0 + 6)):
            d.line([sx, sy, sx + 4, sy + 4], fill=(168, 196, 226, 255),
                   width=1)


def framed_panel(d, x0, y0, x1, y1, v2=False, glass=False,
                 frame=(118, 122, 128, 255)):
    d.rectangle([x0, y0, x1, y1], fill=frame, outline=INK)
    solar_cells(d, x0 + 2, y0 + 2, x1 - 2, y1 - 2, v2=v2, glass=glass)


def draw_folding_panel(d, rng):
    # folded small panel: slim seamed case with a handle
    d.rectangle([13, 3, 19, 6], outline=INK, width=1)  # handle
    d.rectangle([9, 6, 23, 27], fill=(88, 92, 98, 255), outline=INK)
    solar_cells(d, 11, 8, 21, 25)
    d.line([11, 16, 21, 16], fill=INK, width=1)  # fold seam


def draw_folding_panel_deployed(d, rng):
    # spread wide: two standard panels hinged open, small stands
    framed_panel(d, 2, 9, 15, 23)
    framed_panel(d, 17, 9, 30, 23)
    d.line([16, 9, 16, 23], fill=INK, width=1)  # hinge
    d.line([5, 23, 4, 27], fill=STEEL_DARK, width=1)
    d.line([27, 23, 28, 27], fill=STEEL_DARK, width=1)


def draw_folding_panel_v2(d, rng):
    # folded solar suitcase: v2 cells, handle, latches, thick frame
    d.rectangle([12, 3, 20, 6], outline=INK, width=2)  # suitcase handle
    d.rectangle([6, 6, 26, 27], fill=(56, 60, 66, 255), outline=INK)
    solar_cells(d, 8, 8, 24, 25, v2=True)
    for lx in (10, 20):
        d.rectangle([lx, 5, lx + 2, 8], fill=STEEL, outline=INK)  # latches


def draw_folding_panel_v2_deployed(d, rng):
    framed_panel(d, 2, 10, 15, 22, v2=True, frame=(56, 60, 66, 255))
    framed_panel(d, 17, 10, 30, 22, v2=True, frame=(56, 60, 66, 255))
    d.line([16, 10, 16, 22], fill=INK, width=1)
    d.line([5, 22, 4, 26], fill=STEEL_DARK, width=1)
    d.line([27, 22, 28, 26], fill=STEEL_DARK, width=1)


def draw_ground_panel(d, rng):
    for x0, y0 in ((4, 7), (17, 7), (4, 18), (17, 18)):
        framed_panel(d, x0, y0, x0 + 11, y0 + 9)
    d.line([6, 28, 26, 28], fill=STEEL_DARK, width=1)  # ground rail


def draw_ground_panel_v2(d, rng):
    # four smaller high-performance panels, wider gaps
    for x0, y0 in ((5, 8), (18, 8), (5, 19), (18, 19)):
        framed_panel(d, x0, y0, x0 + 9, y0 + 7, v2=True)
    d.line([7, 28, 25, 28], fill=STEEL_DARK, width=1)


def draw_reinforced_panel(d, rng):
    framed_panel(d, 6, 7, 26, 25, glass=True, frame=(100, 104, 110, 255))
    d.rectangle([6, 7, 26, 25], outline=INK, width=1)
    for bx, by in ((8, 9), (24, 9), (8, 23), (24, 23)):  # corner bolts
        d.point((bx, by), fill=INK)


def draw_reinforced_panel_v2(d, rng):
    framed_panel(d, 6, 7, 26, 25, v2=True, glass=True,
                 frame=(100, 104, 110, 255))
    d.rectangle([6, 7, 26, 25], outline=INK, width=1)
    for bx, by in ((8, 9), (24, 9), (8, 23), (24, 23)):
        d.point((bx, by), fill=INK)


def draw_solar_cell_v2(d, rng):
    # bare monocrystalline wafer: octagon with fine silver fingers
    d.polygon([(13, 9), (19, 9), (23, 13), (23, 19), (19, 23), (13, 23),
               (9, 19), (9, 13)], fill=CELL_V2)
    d.polygon([(13, 9), (19, 9), (23, 13), (23, 19), (19, 23), (13, 23),
               (9, 19), (9, 13)], outline=INK)
    for fx in (12, 15, 18, 21):
        d.line([fx, 10, fx, 22], fill=BUS_V2, width=1)
    d.line([10, 16, 22, 16], fill=BUS_V2, width=1)


def draw_solar_panel_v2(d, rng):
    framed_panel(d, 7, 7, 25, 24, v2=True)
    d.line([11, 24, 10, 28], fill=STEEL_DARK, width=1)  # mounting feet
    d.line([21, 24, 22, 28], fill=STEEL_DARK, width=1)


# -------------------------------------------------------------------- gun

def box_mag(d, x0, y0, x1, y1, body=(84, 86, 92, 255), ribs=True):
    d.polygon([(x0, y0), (x1, y0), (x1 - 1, y1), (x0 + 1, y1)], fill=body)
    d.polygon([(x0, y0), (x1, y0), (x1 - 1, y1), (x0 + 1, y1)], outline=INK)
    if ribs:
        for ry in range(y0 + 4, y1 - 1, 4):
            d.line([x0 + 1, ry, x1 - 1, ry], fill=shade(body, 0.7), width=1)
    d.rectangle([x0 - 1, y1, x1 + 1, y1 + 2], fill=PLASTIC_DARK, outline=INK)


def draw_mag10(d, rng):
    box_mag(d, 12, 9, 20, 21)


def draw_mag20(d, rng):
    box_mag(d, 12, 6, 20, 25)


def draw_mag40(d, rng):
    box_mag(d, 12, 3, 20, 28)


def draw_saiga_mag(d, rng):
    # black ribbed plastic mag, a shell showing at the feed lips
    d.rectangle([14, 4, 18, 6], fill=(178, 44, 40, 255))  # shell hull
    d.rectangle([14, 6, 18, 8], fill=BRASS, outline=INK)  # shell base
    box_mag(d, 12, 8, 20, 23, body=(38, 38, 42, 255))


def draw_speedloader(d, rng):
    d.ellipse([8, 8, 24, 24], fill=(58, 58, 62, 255), outline=INK)
    for k in range(6):  # six cartridge heads around the disc
        a = k * math.pi / 3 + 0.26
        cx, cy = 16 + 5.6 * math.cos(a), 16 + 5.6 * math.sin(a)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=BRASS, outline=INK)
        d.point((int(cx), int(cy)), fill=(110, 82, 38, 255))  # primer
    d.ellipse([14, 14, 18, 18], fill=(92, 92, 98, 255), outline=INK)


def draw_gun_crossbow(d, rng):
    # rail along the barrel line, bow limbs at the front, string back
    d.rectangle([4, 14, 24, 18], fill=WOOD, outline=WOOD_DARK)  # rail body
    d.line([4, 16, 24, 16], fill=STEEL_DARK, width=1)  # bolt groove
    d.line([25, 6, 24, 16], fill=STEEL_DARK, width=2)  # limbs
    d.line([24, 16, 25, 26], fill=STEEL_DARK, width=2)
    d.line([25, 6, 26, 4], fill=STEEL_DARK, width=2)
    d.line([25, 26, 26, 28], fill=STEEL_DARK, width=2)
    d.line([26, 4, 10, 16], fill=(220, 220, 216, 255), width=1)  # string
    d.line([10, 16, 26, 28], fill=(220, 220, 216, 255), width=1)
    d.rectangle([22, 13, 25, 19], fill=STEEL, outline=INK)  # prod mount


def draw_ksg_tube(d, rng):
    d.rectangle([4, 13, 28, 18], fill=(140, 144, 150, 255), outline=INK)
    d.line([5, 15, 27, 15], fill=STEEL_LIGHT, width=1)
    for rx in range(8, 27, 4):  # tube ribs
        d.line([rx, 13, rx, 18], fill=STEEL_DARK, width=1)
    d.rectangle([4, 12, 6, 19], fill=STEEL_DARK, outline=INK)  # end cap
    d.rectangle([10, 10, 22, 12], fill=(110, 114, 120, 255),
                outline=INK)  # top rail stub


def draw_rm121(d, rng, mod=False):
    body = (80, 82, 88, 255)
    d.rectangle([10, 9, 22, 11], fill=STEEL_DARK, outline=INK)  # mount rail
    d.rectangle([8, 12, 24, 19], fill=body, outline=INK)  # receiver
    d.rectangle([19, 13, 23, 18], fill=(196, 198, 200, 255),
                outline=INK)  # ceramic plate
    d.rectangle([24, 14, 29, 17], fill=(58, 60, 64, 255), outline=INK)
    if mod:  # straps over the rail, longer mag, foregrip
        for sx in (12, 19):
            d.rectangle([sx, 8, sx + 2, 12], fill=INK)
        d.rectangle([13, 19, 18, 28], fill=(64, 66, 70, 255), outline=INK)
        d.rectangle([22, 19, 25, 25], fill=(50, 52, 56, 255), outline=INK)
        d.line([11, 21, 11, 27], fill=(150, 152, 156, 255), width=1)
    else:
        d.rectangle([13, 19, 18, 26], fill=(64, 66, 70, 255), outline=INK)
    d.line([14, 21, 17, 21], fill=(150, 152, 156, 255), width=1)


def draw_rm121aux(d, rng):
    draw_rm121(d, rng, mod=False)


def draw_rm121aux_mod(d, rng):
    draw_rm121(d, rng, mod=True)


# ------------------------------------------------------------------- ammo

def cartridge(d, tip, provenance):
    """One .30-06 cartridge, centred.  tip: jsp/fmj/ap/tracer.
    provenance: factory/bp/reloaded."""
    if provenance == "factory":
        case, hi = BRASS, (234, 198, 112, 255)
    elif provenance == "bp":
        case, hi = BRASS_DULL, (196, 158, 92, 255)
    else:
        case, hi = (178, 140, 70, 255), (206, 168, 100, 255)
    # bullet
    if tip == "jsp":  # jacketed sides, flat exposed lead nose
        d.polygon([(14, 11), (18, 11), (18, 6), (14, 6)], fill=COPPER)
        d.rectangle([14, 4, 18, 6], fill=LEAD, outline=INK)
        d.polygon([(14, 11), (18, 11), (18, 6), (14, 6)], outline=INK)
    else:
        d.polygon([(14, 11), (18, 11), (17, 5), (16, 3), (15, 5)],
                  fill=COPPER)
        d.polygon([(14, 11), (18, 11), (17, 5), (16, 3), (15, 5)],
                  outline=INK)
        if tip == "ap":  # black AP tip
            d.polygon([(15, 5), (17, 5), (16, 3)], fill=(30, 30, 34, 255))
        elif tip == "tracer":  # orange tracer tip
            d.polygon([(15, 6), (17, 6), (16, 3), (15, 5)],
                      fill=(224, 92, 30, 255))
            d.point((16, 2), fill=(250, 170, 60, 255))
    # neck and shoulder
    d.polygon([(13, 15), (19, 15), (18, 11), (14, 11)], fill=shade(case, 0.9))
    d.polygon([(13, 15), (19, 15), (18, 11), (14, 11)], outline=INK)
    # case body and rim
    d.rectangle([13, 15, 19, 27], fill=case, outline=INK)
    d.line([15, 16, 15, 26], fill=hi, width=1)  # highlight
    d.rectangle([12, 27, 20, 29], fill=shade(case, 0.92), outline=INK)
    if provenance == "bp":  # soot band at the case mouth
        d.rectangle([13, 15, 19, 18], fill=(74, 64, 50, 255))
        d.line([15, 16, 15, 18], fill=(96, 84, 64, 255), width=1)
    elif provenance == "reloaded":  # tarnish scratches
        for sx, sy in ((17, 18), (14, 21), (18, 24), (15, 25)):
            d.line([sx, sy, sx + 1, sy + 2], fill=(118, 94, 48, 255),
                   width=1)


DRAW = {
    "bandages_makeshift": draw_bandage_plain,
    "bandages_makeshift_bleached": draw_bandage_bleached,
    "bandages_makeshift_boiled": draw_bandage_boiled,
    "dental_tools": draw_dental_tools,
    "disincotton_ball": draw_cotton_balls,
    "disinrag": draw_disinrag,
    "liq_bandage_spray": draw_liq_bandage,
    "smoxygen_tank": draw_smoxygen,
    "vacutainer": draw_vacutainer,
    "wrapped_rad_badge": draw_rad_badge,
    "chair_folding": draw_chair_folding,
    "deck_chair": draw_deck_chair,
    "hobo_stove": draw_hobo_stove,
    "makeshift_brazier": draw_brazier,
    "metal_butcher_rack": draw_butcher_rack,
    "metal_smoking_rack": draw_smoking_rack,
    "portable_stove": draw_portable_stove,
    "screw_press_tool": draw_screw_press,
    "solar_cooker": draw_solar_cooker,
    "target_spinner": draw_target_spinner,
    "folding_solar_panel": draw_folding_panel,
    "folding_solar_panel_deployed": draw_folding_panel_deployed,
    "folding_solar_panel_v2": draw_folding_panel_v2,
    "folding_solar_panel_v2_deployed": draw_folding_panel_v2_deployed,
    "ground_solar_panel": draw_ground_panel,
    "ground_solar_panel_v2": draw_ground_panel_v2,
    "reinforced_solar_panel": draw_reinforced_panel,
    "reinforced_solar_panel_v2": draw_reinforced_panel_v2,
    "solar_cell_v2": draw_solar_cell_v2,
    "solar_panel_v2": draw_solar_panel_v2,
    "20x66_10_mag": draw_mag10,
    "20x66_20_mag": draw_mag20,
    "20x66_40_mag": draw_mag40,
    "454_speedloader6": draw_speedloader,
    "gun_crossbow": draw_gun_crossbow,
    "ksg25_aux_shotgun": draw_ksg_tube,
    "rm121aux": draw_rm121aux,
    "rm121aux_mod": draw_rm121aux_mod,
    "saiga410mag_10rd": draw_saiga_mag,
    "3006_jsp": lambda d, r: cartridge(d, "jsp", "factory"),
    "bp_3006": lambda d, r: cartridge(d, "fmj", "bp"),
    "bp_3006_incendiary": lambda d, r: cartridge(d, "tracer", "bp"),
    "bp_3006_jsp": lambda d, r: cartridge(d, "jsp", "bp"),
    "bp_3006fmj": lambda d, r: cartridge(d, "ap", "bp"),
    "reloaded_3006": lambda d, r: cartridge(d, "fmj", "reloaded"),
    "reloaded_3006_incendiary": lambda d, r: cartridge(d, "tracer",
                                                   "reloaded"),
    "reloaded_3006_jsp": lambda d, r: cartridge(d, "jsp", "reloaded"),
    "reloaded_3006fmj": lambda d, r: cartridge(d, "ap", "reloaded"),
}


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
    assert set(DRAW) == set(IDS) and len(IDS) == len(set(IDS))
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
        prev.alpha_composite(big, ((i % cols) * cell + 1,
                                   (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item73.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item73.png {prev.size}")


if __name__ == "__main__":
    main()
