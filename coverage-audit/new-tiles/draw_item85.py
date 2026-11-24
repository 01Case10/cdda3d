#!/usr/bin/env python3
"""ITEM85: broken Exodii robots, leg armor, Highlands relics, ferment
jars, dive gear, folded vehicles, filters, magazines, batteries,
throwing darts, leg rigs and masks -- CDDA Voxel coverage audit.

Families (art grounded in each id's game entry):

  Broken Exodii robots (dead_exodii.json, qt_steel) -- slumped,
    cracked hulls with exposed wiring and dead optics; silhouettes
    per type: quadruped walker (huge, one leg buckled), balloon
    drone (shredded envelope, intact chassis), turret (drooping
    barrel), worker (small biped, arms dangling).
  Leg armor -- sheet-metal skirts: four lc_steel sheets on a belt;
    the hardened ch_ version darker blued steel.  EOD trousers:
    bulky Kevlar/Nomex padding, light-green standard vs slimmer
    light-gray tactical.
  Highlands relics -- silver obol (profile face / tree-and-stars
    reverse), ivory funeral mask with bronze rings, gold medallion,
    landscape miniature painting (fog, mountains, ruins).
  Ferment jars -- glass jars mid-process: eggs in brine, green
    pickles, shredded sauerkraut, all bubbling; crystallizing
    sugar as syrup pan with crystals growing on strings.
  Dive gear -- full-face masks (dark modular with pods vs blue
    plain with open port), wetsuits sized xl/xs, air cylinders
    closed vs open (open = turned valve wheel, lit gauge).
  Folded vehicles, filters (black CBRN canister vs small yellow
    cartridge; rebreather scrubber+tank, blue AIR vs green O2),
    pistol magazines scaled by capacity, storage batteries graded
    small/medium/large plus the empty battery case, fletched war
    darts (iron/bronze/alien bone), a lead weighted dart, leg bags
    and pouches, a net backpack, and three masks (plague doctor,
    pumpkin, zombie) plus the gold-latticed biocoif.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "broken_exodii_quad",
    "broken_exodii_sniper_drone",
    "broken_exodii_turret",
    "broken_exodii_worker",
    "ch_legguard_metal_sheets_hip",
    "legguard_metal_sheets_hip",
    "trousers_eod",
    "trousers_eod_light",
    "coin_highlands",
    "highland_funeral_mask",
    "highland_medallion",
    "higlands_teleport",
    "crystallizing_sugar",
    "eggs_ferment",
    "pickles_ferment",
    "sauerkraut_ferment",
    "divemask_full_modular",
    "divemask_full_scuba_notready",
    "xl_wetsuit",
    "xs_wetsuit",
    "firefighter_scba_tank",
    "firefighter_scba_tank_on",
    "small_scuba_tank",
    "small_scuba_tank_on",
    "folded_bicycle",
    "folded_skateboard_generic",
    "folded_wheelchair_generic",
    "generic_folded_vehicle",
    "gasfilter_med",
    "gasfilter_sm",
    "rebreather_cartridge_air",
    "rebreather_cartridge_o2",
    "glock17_10",
    "p226mag_10rd_9x19mm",
    "p226mag_15rd_9x19mm",
    "p228mag_13rd_9x19mm",
    "it_battery_mount",
    "large_storage_battery",
    "medium_storage_battery",
    "small_storage_battery",
    "javelin_fletched",
    "javelin_fletched_bronze",
    "nether_huntsman_javelin",
    "weighted_dart",
    "leg_bag",
    "legrig",
    "legrig_wetsuit",
    "net_backpack",
    "mask_plague",
    "pumpkin_mask",
    "string_dimension_biocoif",
    "zombie_mask",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 46, 52, 255)
STEEL = (168, 172, 178, 255)       # qt_steel hull
STEEL_DARK = (104, 108, 116, 255)
STEEL_DEAD = (74, 78, 86, 255)
WIRE_RED = (178, 60, 48, 255)
WIRE_COP = (196, 120, 52, 255)
LC_STEEL = (186, 190, 196, 255)
CH_STEEL = (92, 100, 116, 255)     # hardened, blued
CH_STEEL_L = (122, 132, 150, 255)
KEVLAR = (128, 142, 96, 255)       # EOD light green
KEVLAR_D = (96, 108, 70, 255)
TACT = (172, 174, 176, 255)        # tactical light gray
TACT_D = (128, 130, 132, 255)
SILVER = (206, 208, 214, 255)
SILVER_D = (150, 152, 160, 255)
IVORY = (234, 228, 206, 255)
IVORY_D = (198, 190, 164, 255)
BRONZE = (164, 116, 62, 255)
BRONZE_D = (122, 84, 44, 255)
GOLD = (226, 182, 72, 255)
GOLD_D = (178, 138, 44, 255)
GLASS = (168, 196, 204, 255)
GLASS_D = (110, 140, 150, 255)
BRINE = (226, 214, 150, 255)
WOOD = (120, 88, 52, 255)
WOOD_D = (88, 62, 36, 255)
BLACK = (40, 40, 46, 255)
RUBBER = (58, 58, 64, 255)
YELLOW = (232, 190, 60, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def crack(d, pts, col=STEEL_DEAD):
    d.line(pts, fill=col, width=1)


def wires(d, rng, x, y, n=3):
    for i in range(n):
        col = (WIRE_RED, WIRE_COP, STEEL_DEAD)[i % 3]
        d.line([x, y, x + rng.randint(-3, 3), y + 2 + i,
                x + rng.randint(-4, 4), y + 4 + i], fill=col, width=1)


# ------------------------------------------------------------- exodii robots

def draw_quad(d, rng):
    # huge slumped quadruped walker: hull y9..18, four legs, one buckled
    d.rectangle([5, 9, 26, 18], fill=STEEL, outline=INK)
    d.rectangle([5, 9, 26, 12], fill=shade(STEEL, 0.88))
    d.rectangle([21, 11, 29, 17], fill=STEEL_DARK, outline=INK)  # head
    d.point((27, 14), fill=BLACK)                                # dead optic
    d.point((26, 14), fill=STEEL_DEAD)
    crack(d, [(8, 10), (11, 13), (10, 16), (13, 18)])
    crack(d, [(18, 9), (17, 12), (19, 14)])
    for lx in (7, 12):   # left legs, standing but stiff
        d.line([lx, 18, lx, 27], fill=STEEL_DARK, width=2)
        d.line([lx - 1, 27, lx + 2, 27], fill=INK, width=1)
    d.line([19, 18, 20, 24], fill=STEEL_DARK, width=2)           # buckled leg
    d.line([20, 24, 24, 26], fill=STEEL_DARK, width=2)
    d.line([25, 18, 27, 23], fill=STEEL_DEAD, width=1)           # snapped stub
    wires(d, rng, 26, 19, 3)
    d.rectangle([13, 4, 18, 8], fill=STEEL_DARK, outline=INK)    # dorsal mount
    d.point((15, 6), fill=WIRE_RED)


def draw_drone(d, rng):
    # shredded balloon envelope over an intact chassis
    d.pieslice([6, 1, 26, 14], start=180, end=360, fill=YELLOW,
               outline=INK)
    d.rectangle([6, 7, 26, 11], fill=YELLOW)
    for tx in range(8, 26, 4):   # torn shreds hanging from the envelope
        d.polygon([(tx, 11), (tx + 2, 11), (tx + 1, 14 + (tx % 3))],
                  fill=shade(YELLOW, 0.82))
    crack(d, [(10, 3), (13, 6), (12, 9)], col=shade(YELLOW, 0.6))
    d.point((20, 4), fill=shade(YELLOW, 0.6))
    d.line([10, 12, 12, 17], fill=INK, width=1)                  # rigging
    d.line([22, 12, 20, 17], fill=INK, width=1)
    d.rectangle([11, 17, 21, 25], fill=STEEL, outline=INK)       # chassis
    d.rectangle([13, 20, 17, 24], fill=STEEL_DARK)               # sensor pod
    d.point((15, 22), fill=BLACK)
    d.line([21, 19, 25, 17], fill=STEEL_DARK, width=2)           # prop arm
    d.line([25, 14, 25, 20], fill=STEEL_DEAD, width=1)           # bent prop
    wires(d, rng, 12, 25, 2)


def draw_turret(d, rng):
    d.polygon([(7, 28), (25, 28), (22, 21), (10, 21)], fill=STEEL_DARK,
              outline=INK)                                       # base
    d.rectangle([13, 16, 19, 21], fill=STEEL, outline=INK)       # column
    d.rectangle([8, 7, 21, 16], fill=STEEL, outline=INK)         # head
    d.rectangle([8, 7, 21, 10], fill=shade(STEEL, 0.88))
    d.point((11, 13), fill=BLACK)                                # dead optic
    d.line([21, 11, 30, 14], fill=STEEL_DEAD, width=2)           # drooping gun
    d.line([21, 13, 29, 16], fill=STEEL_DEAD, width=1)
    crack(d, [(15, 8), (17, 11), (16, 15)])
    wires(d, rng, 18, 17, 3)
    d.line([9, 28, 23, 28], fill=INK, width=1)


def draw_worker(d, rng):
    d.rectangle([12, 3, 19, 9], fill=STEEL, outline=INK)         # head
    d.point((15, 6), fill=BLACK)
    d.line([13, 3, 13, 1], fill=STEEL_DEAD, width=1)             # bent antenna
    d.rectangle([10, 10, 21, 20], fill=STEEL, outline=INK)       # torso
    d.rectangle([13, 13, 18, 17], fill=STEEL_DARK)               # chest hatch
    crack(d, [(11, 11), (14, 14), (13, 18)])
    d.line([10, 11, 6, 20], fill=STEEL_DARK, width=2)            # dangling arm
    d.line([6, 20, 6, 25], fill=STEEL_DARK, width=2)
    d.line([21, 11, 25, 19], fill=STEEL_DARK, width=2)           # other arm
    d.line([25, 19, 24, 24], fill=STEEL_DARK, width=2)
    d.line([12, 20, 12, 28], fill=STEEL_DARK, width=2)           # legs
    d.line([19, 20, 20, 28], fill=STEEL_DARK, width=2)
    d.line([10, 28, 14, 28], fill=INK, width=1)
    d.line([18, 28, 22, 28], fill=INK, width=1)
    wires(d, rng, 17, 20, 2)


# --------------------------------------------------------------- leg armor

def draw_metal_skirt(d, rng, hard):
    main = CH_STEEL if hard else LC_STEEL
    lite = CH_STEEL_L if hard else shade(LC_STEEL, 1.0)
    d.rectangle([5, 5, 26, 8], fill=WOOD_D)                      # belt
    d.rectangle([14, 5, 17, 8], fill=GOLD_D)                     # buckle
    for i, x0 in enumerate((5, 11, 17, 22)):                     # four sheets
        w = 5 if i < 3 else 4
        d.polygon([(x0, 9), (x0 + w, 9), (x0 + w - 1, 25 + (i % 2)),
                   (x0 + 1, 25 + (i % 2))], fill=main, outline=INK)
        d.line([x0 + 2, 11, x0 + 2, 22], fill=lite, width=1)
        d.point((x0 + 2, 10), fill=INK)                          # rivets
        d.point((x0 + 2, 23), fill=INK)
    if hard:   # temper colours along the top edge
        for x in range(6, 26, 3):
            d.point((x, 9), fill=(150, 110, 140, 255))


def draw_eod_trousers(d, rng, light):
    main = TACT if light else KEVLAR
    dark = TACT_D if light else KEVLAR_D
    d.rectangle([9, 4, 22, 9], fill=dark)                        # waistband
    for lx in (9, 17):                                           # legs
        d.rectangle([lx, 9, lx + 5, 28], fill=main, outline=INK)
        d.rectangle([lx + 1, 11, lx + 4, 17], fill=dark)         # thigh pad
        d.rectangle([lx + 1, 20, lx + 4, 23], fill=dark)         # knee pad
        if not light:
            d.rectangle([lx + 1, 25, lx + 4, 27], fill=dark)     # shin pad
    d.line([15, 9, 15, 14], fill=INK, width=1)
    for x in range(10, 22, 3):
        d.point((x, 6), fill=INK)


# --------------------------------------------------------------- highlands

def draw_coin(d, rng):
    d.ellipse([15, 10, 27, 22], fill=SILVER_D, outline=INK)      # reverse
    for a in range(0, 360, 45):                                  # three star-eyes
        pass
    d.line([21, 19, 21, 14], fill=INK, width=1)                  # tree trunk
    d.ellipse([18, 11, 24, 16], fill=SILVER_D, outline=INK)      # tree crown
    for sx, sy in ((19, 12), (21, 11), (23, 12)):
        d.point((sx, sy), fill=GOLD_D)
    d.ellipse([4, 8, 20, 24], fill=SILVER, outline=INK)          # obverse
    d.ellipse([5, 9, 19, 23], outline=SILVER_D)                  # rim
    # faded profile facing right: brow/nose/chin bumps in darker silver
    prof = [(9, 12), (12, 11), (14, 12), (15, 14), (14, 15),
            (15, 17), (14, 19), (12, 21), (9, 20)]
    d.line(prof, fill=SILVER_D, width=1)
    d.point((11, 14), fill=SILVER_D)


def draw_funeral_mask(d, rng):
    for r in (13, 11):   # concentric bronze rings around the mask
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=BRONZE_D)
    d.ellipse([8, 3, 24, 29], fill=IVORY, outline=INK)           # ivory face
    d.ellipse([9, 4, 23, 28], outline=IVORY_D)
    d.arc([11, 10, 15, 14], start=20, end=160, fill=INK)         # closed eyes
    d.arc([17, 10, 21, 14], start=20, end=160, fill=INK)
    d.line([16, 14, 15, 19], fill=IVORY_D, width=1)              # nose
    d.arc([13, 20, 19, 25], start=20, end=160, fill=INK)         # serene mouth
    for gx, gy in ((16, 5), (16, 27), (9, 16), (23, 16)):
        d.point((gx, gy), fill=BRONZE)


def draw_medallion(d, rng):
    for i in range(6):   # chain links rising to both corners
        d.point((6 + i * 2, 8 - i), fill=GOLD_D)
        d.point((25 - i * 2, 8 - i), fill=GOLD_D)
    d.ellipse([9, 9, 23, 25], fill=GOLD, outline=INK)
    d.ellipse([11, 11, 21, 23], outline=GOLD_D)
    d.polygon([(16, 13), (17, 16), (20, 16), (18, 18), (19, 21),
               (16, 19), (13, 21), (14, 18), (12, 16), (15, 16)],
              fill=GOLD_D)                                       # star
    d.point((16, 17), fill=(250, 226, 140, 255))


def draw_landscape(d, rng):
    d.rectangle([4, 6, 27, 26], fill=WOOD_D, outline=INK)        # frame
    d.rectangle([6, 8, 25, 24], fill=(196, 200, 204, 255))      # fog sea
    d.polygon([(6, 24), (12, 13), (17, 24)], fill=(96, 100, 112, 255))
    d.polygon([(14, 24), (20, 11), (25, 24)], fill=(78, 82, 96, 255))
    d.rectangle([8, 17, 10, 21], fill=(60, 62, 72, 255))         # jagged ruin
    d.polygon([(8, 17), (9, 14), (10, 17)], fill=(60, 62, 72, 255))
    d.rectangle([21, 18, 23, 22], fill=(60, 62, 72, 255))
    for y in (19, 22):   # drifting fog bands
        d.line([6, y, 25, y], fill=(222, 224, 226, 255), width=1)
    d.point((13, 10), fill=(240, 240, 235, 255))                 # shimmer


# --------------------------------------------------------------- ferments

def jar(d, contents):
    d.rectangle([10, 6, 21, 8], fill=STEEL_DARK, outline=INK)    # lid
    d.rectangle([9, 8, 22, 29], outline=GLASS_D)
    d.rectangle([10, 9, 21, 28], fill=(214, 228, 230, 255))
    contents()
    d.line([11, 10, 11, 27], fill=(240, 248, 248, 255), width=1)  # glass shine


def bubbles(d, rng, n=7):
    for _ in range(n):
        d.point((rng.randint(11, 20), rng.randint(11, 26)),
                fill=(246, 248, 240, 255))


def draw_ferment(d, rng, kind):
    if kind == "eggs":
        def c():
            d.rectangle([10, 14, 21, 28], fill=BRINE)
            for ex, ey in ((12, 17), (17, 16), (14, 22), (18, 23), (12, 25)):
                d.ellipse([ex, ey, ex + 4, ey + 5], fill=(246, 240, 214, 255),
                          outline=(196, 182, 130, 255))
            bubbles(d, rng)
    elif kind == "pickles":
        def c():
            d.rectangle([10, 14, 21, 28], fill=(168, 186, 110, 255))
            for ex, ey in ((11, 16), (16, 15), (13, 21), (18, 22), (12, 25)):
                d.ellipse([ex, ey, ex + 3, ey + 6], fill=(88, 138, 62, 255),
                          outline=(56, 96, 40, 255))
            bubbles(d, rng)
    else:  # sauerkraut
        def c():
            d.rectangle([10, 13, 21, 28], fill=(232, 224, 180, 255))
            for _ in range(26):
                x = rng.randint(11, 19)
                y = rng.randint(14, 26)
                d.line([x, y, x + 2, y + 1], fill=(206, 194, 140, 255),
                       width=1)
            bubbles(d, rng, 5)
    jar(d, c)


def draw_crystallizing(d, rng):
    d.polygon([(5, 22), (26, 22), (24, 29), (7, 29)], fill=GLASS_D,
              outline=INK)                                       # shallow pan
    d.rectangle([6, 20, 25, 22], fill=(238, 200, 110, 255))     # syrup
    for sx in (10, 15, 20):   # strings with growing crystals
        d.line([sx, 12, sx, 21], fill=WOOD_D, width=1)
        for cy in range(15, 21, 2):
            d.rectangle([sx - 1, cy, sx + 1, cy + 1],
                        fill=(250, 248, 244, 255), outline=(206, 200, 190, 255))
    d.line([8, 12, 23, 12], fill=WOOD, width=2)                  # skewer bar
    d.point((13, 19), fill=(255, 255, 255, 255))


# --------------------------------------------------------------- dive gear

def draw_divemask(d, rng, modular):
    frame = (52, 54, 60, 255) if modular else (52, 96, 178, 255)
    d.ellipse([5, 6, 27, 25], fill=frame, outline=INK)           # mask body
    d.ellipse([8, 9, 24, 20], fill=(36, 52, 66, 255), outline=INK)  # visor
    d.line([10, 17, 17, 10], fill=(120, 160, 180, 255), width=2)  # glare
    d.rectangle([13, 21, 19, 27], fill=RUBBER, outline=INK)      # port
    if modular:
        d.ellipse([12, 22, 20, 26], fill=STEEL_DARK)             # regulator cap
        d.point((16, 24), fill=STEEL)
        for px in (5, 24):   # side pods
            d.rectangle([px, 12, px + 3, 18], fill=frame, outline=INK)
        d.point((7, 8), fill=YELLOW)                             # module dot
    else:
        d.ellipse([14, 22, 18, 26], outline=STEEL_DARK)          # open port
        d.point((16, 24), fill=BLACK)
    d.line([5, 10, 2, 8], fill=INK, width=2)                     # straps
    d.line([27, 10, 30, 8], fill=INK, width=2)


def draw_wetsuit(d, rng, big):
    s = 1.0 if big else 0.72
    cx, cy = 16, 16
    w, h = int(11 * s), int(26 * s)
    x0, x1 = cx - w // 2, cx + w // 2
    y0, y1 = cy - h // 2, cy + h // 2
    body = (52, 54, 60, 255)
    d.rectangle([x0, y0, x1, y0 + int(9 * s)], fill=body, outline=INK)
    d.rectangle([x0, y0 + int(9 * s), cx - 1, y1], fill=body, outline=INK)
    d.rectangle([cx + 1, y0 + int(9 * s), x1, y1], fill=body, outline=INK)
    d.rectangle([x0 - int(3 * s), y0 + 1, x0 - 1, y0 + int(10 * s)],
                fill=body, outline=INK)                          # arms
    d.rectangle([x1 + 1, y0 + 1, x1 + int(3 * s), y0 + int(10 * s)],
                fill=body, outline=INK)
    d.line([cx, y0 + 1, cx, y0 + int(8 * s)], fill=YELLOW, width=1)  # zip
    d.rectangle([x0, y0, x1, y0 + 2], fill=(30, 60, 110, 255))   # collar trim
    d.point((cx, y0 + int(4 * s)), fill=(200, 60, 50, 255))      # logo


def draw_tank(d, rng, big, on):
    x0, x1 = (10, 21) if big else (11, 20)
    y0, y1 = (8, 29) if big else (11, 29)
    d.rectangle([x0, y0 + 3, x1, y1], fill=(196, 198, 202, 255), outline=INK)
    d.pieslice([x0, y0, x1, y0 + 7], start=180, end=360,
               fill=(196, 198, 202, 255), outline=INK)
    if big:   # carbon-fiber wrap bands
        for y in range(y0 + 7, y1 - 2, 4):
            d.line([x0 + 1, y, x1 - 1, y], fill=(70, 70, 78, 255), width=1)
            d.line([x0 + 1, y + 1, x1 - 1, y + 1], fill=(120, 120, 128, 255),
                   width=1)
    else:
        d.rectangle([x0, y0 + 10, x1, y0 + 13], fill=(52, 96, 178, 255))
    vx = (x0 + x1) // 2
    d.rectangle([vx - 1, y0 - 3, vx + 1, y0 + 1], fill=BRONZE_D)  # valve stem
    if on:   # open: wheel spokes up, gauge lit green, air ticks
        d.ellipse([vx - 3, y0 - 6, vx + 3, y0 - 1], outline=INK)
        d.line([vx - 3, y0 - 3, vx + 3, y0 - 3], fill=INK, width=1)
        d.line([vx, y0 - 6, vx, y0 - 1], fill=INK, width=1)
        d.ellipse([x1 - 1, y0 + 2, x1 + 4, y0 + 7], fill=(120, 230, 140, 255),
                  outline=INK)
        for ax, ay in ((vx + 5, y0 - 5), (vx + 7, y0 - 3)):
            d.point((ax, ay), fill=GLASS)
    else:
        d.rectangle([vx - 3, y0 - 5, vx + 3, y0 - 2], fill=STEEL_DARK,
                    outline=INK)                                 # closed wheel
        d.ellipse([x1 - 1, y0 + 2, x1 + 4, y0 + 7], fill=(210, 210, 210, 255),
                  outline=INK)                                   # dim gauge
        d.point((x1 + 1, y0 + 4), fill=INK)


# --------------------------------------------------------------- folded kit

def draw_folded_bicycle(d, rng):
    for cx in (10, 21):   # two wheels side by side
        d.ellipse([cx - 6, 12, cx + 6, 24], outline=INK, width=2)
        d.ellipse([cx - 6, 12, cx + 6, 24], outline=STEEL_DARK)
        d.point((cx, 18), fill=INK)
        for dx, dy in ((-4, -3), (4, -3), (-4, 3), (4, 3)):
            d.point((cx + dx, 18 + dy), fill=STEEL_DARK)
    d.line([10, 18, 16, 9], fill=WIRE_COP, width=2)              # folded frame
    d.line([16, 9, 21, 18], fill=WIRE_COP, width=2)
    d.line([10, 18, 21, 18], fill=shade(WIRE_COP, 0.8), width=1)
    d.line([14, 9, 19, 9], fill=INK, width=2)                    # handlebar stub
    d.rectangle([13, 6, 18, 8], fill=BLACK)                      # saddle


def draw_skateboard(d, rng):
    d.polygon([(4, 13), (25, 13), (28, 15), (25, 17), (4, 17), (2, 15)],
              fill=(196, 90, 60, 255), outline=INK)              # deck
    d.line([5, 15, 26, 15], fill=(90, 90, 96, 255), width=1)     # grip tape
    for wx in (8, 23):
        d.rectangle([wx - 1, 17, wx + 1, 19], fill=STEEL_DARK)   # trucks
        d.ellipse([wx - 2, 19, wx + 2, 24], fill=(240, 240, 240, 255),
                  outline=INK)                                   # wheels
    d.point((15, 14), fill=(250, 220, 120, 255))                # logo dot


def draw_folded_wheelchair(d, rng):
    d.ellipse([3, 8, 21, 26], outline=INK, width=2)              # big wheel
    d.ellipse([4, 9, 20, 25], outline=STEEL_DARK)
    d.point((12, 17), fill=INK)
    for dx, dy in ((-5, -5), (5, -5), (-5, 5), (5, 5), (0, -6), (0, 6)):
        d.point((12 + dx, 17 + dy), fill=STEEL_DARK)
    d.line([19, 10, 27, 6], fill=STEEL_DARK, width=2)            # folded frame
    d.line([20, 17, 27, 13], fill=STEEL_DARK, width=2)
    d.line([19, 10, 20, 17], fill=INK, width=1)                  # X brace
    d.line([20, 17, 19, 10], fill=INK, width=1)
    d.ellipse([24, 20, 29, 25], outline=INK)                     # caster
    d.rectangle([18, 4, 26, 6], fill=BLACK)                      # folded seat top


def draw_generic_folded(d, rng):
    d.polygon([(8, 8), (24, 8), (26, 24), (10, 26)], fill=(150, 152, 158, 255),
              outline=INK)                                       # frame bundle
    d.line([9, 14, 25, 13], fill=STEEL_DARK, width=2)
    d.line([10, 20, 25, 20], fill=STEEL_DARK, width=2)
    d.line([12, 8, 14, 26], fill=STEEL_DEAD, width=1)
    d.line([20, 8, 21, 25], fill=STEEL_DEAD, width=1)
    d.rectangle([13, 15, 19, 19], fill=YELLOW, outline=INK)      # warning tag
    d.point((16, 16), fill=INK)
    d.line([16, 17, 16, 18], fill=INK, width=1)                  # "!" mark
    d.point((23, 23), fill=WIRE_RED)


# --------------------------------------------------------------- filters

def draw_gasfilter(d, rng, med):
    if med:
        d.rectangle([11, 9, 20, 27], fill=(44, 44, 50, 255), outline=INK)
        for y in range(11, 16):   # cap ridges
            d.line([11, y, 20, y], fill=(70, 70, 78, 255), width=1)
        d.rectangle([12, 19, 19, 23], fill=(120, 34, 34, 255))   # CBRN band
        d.point((15, 21), fill=(240, 240, 240, 255))
        d.rectangle([13, 5, 18, 9], fill=STEEL_DARK, outline=INK)  # thread
    else:
        d.rectangle([12, 13, 19, 26], fill=YELLOW, outline=INK)
        d.rectangle([13, 17, 18, 20], fill=(240, 240, 240, 255), outline=INK)
        d.rectangle([13, 9, 18, 13], fill=STEEL_DARK, outline=INK)
        d.point((15, 24), fill=INK)


def draw_rebreather(d, rng, o2):
    d.rectangle([6, 12, 16, 26], fill=(150, 152, 158, 255), outline=INK)
    for y in range(15, 25, 3):   # scrubber granule lines
        d.line([8, y, 14, y], fill=STEEL_DEAD, width=1)
    d.rectangle([18, 8, 25, 26], fill=(196, 198, 202, 255), outline=INK)
    d.pieslice([18, 5, 25, 12], start=180, end=360,
               fill=(196, 198, 202, 255), outline=INK)           # tank
    band = (52, 150, 74, 255) if o2 else (52, 96, 178, 255)
    d.rectangle([18, 15, 25, 19], fill=band)                     # AIR blue/O2 green
    d.rectangle([20, 3, 23, 6], fill=BRONZE_D)                   # valve
    d.arc([8, 4, 24, 14], start=180, end=360, fill=RUBBER, width=2)  # hose


# --------------------------------------------------------------- magazines

def draw_magazine(d, rng, h, col, holes, base_col=None):
    x0, x1 = 13, 19
    y0 = 28 - h
    d.polygon([(x0, y0), (x1, y0), (x1 - 1, 27), (x0 - 1, 27)], fill=col,
              outline=INK)
    d.rectangle([x0 - 2, 27, x1 + 1, 29], fill=base_col or shade(col, 0.8),
                outline=INK)                                     # baseplate
    d.rectangle([x0 + 1, y0 - 3, x1 - 1, y0], fill=(206, 170, 90, 255),
                outline=INK)                                     # top round
    for i in range(holes):   # witness holes
        d.point((16, y0 + 4 + i * 4), fill=INK)


# --------------------------------------------------------------- batteries

def draw_battery(d, rng, size):
    dims = {"s": (10, 15, 21, 27, 3), "m": (8, 12, 24, 28, 4),
            "l": (6, 9, 26, 29, 6)}[size]
    x0, y0, x1, y1, caps = dims
    d.rectangle([x0, y0, x1, y1], fill=(148, 168, 176, 255), outline=INK)
    d.rectangle([x0, y0, x1, y0 + 3], fill=(110, 132, 142, 255))
    step = (x1 - x0 - 2) / caps
    for i in range(caps):   # cell caps
        cx = int(x0 + 2 + i * step)
        d.ellipse([cx, y0 + 5, cx + 2, y0 + 7], fill=STEEL_DARK, outline=INK)
    d.rectangle([x0 + 1, y0 + 1, x0 + 3, y0 + 3], fill=WIRE_RED)  # + terminal
    d.rectangle([x1 - 3, y0 + 1, x1 - 1, y0 + 3], fill=INK)       # - terminal
    d.line([x0 + 2, y1 - 3, x1 - 2, y1 - 3], fill=(90, 110, 120, 255),
           width=1)


def draw_battery_mount(d, rng):
    d.polygon([(7, 10), (25, 10), (23, 27), (9, 27)], outline=INK, width=2)
    d.polygon([(7, 10), (25, 10), (23, 27), (9, 27)],
              fill=(120, 124, 130, 255))
    d.polygon([(11, 14), (21, 14), (20, 24), (12, 24)],
              fill=(52, 54, 60, 255))                            # empty slot
    d.line([8, 17, 24, 16], fill=WOOD_D, width=2)                # harness strap
    d.line([9, 22, 23, 23], fill=WOOD_D, width=2)
    d.rectangle([20, 5, 26, 9], fill=(62, 130, 76, 255), outline=INK)
    d.point((22, 7), fill=GOLD)                                 # controller chip
    d.point((24, 7), fill=GOLD)
    d.arc([24, 8, 30, 16], start=90, end=270, fill=WIRE_COP, width=1)  # wire


# --------------------------------------------------------------- darts

def draw_javelin(d, rng, tip_col, alien=False):
    d.line([7, 25, 22, 10], fill=WOOD, width=2)                  # shaft
    if alien:
        d.line([7, 25, 22, 10], fill=(196, 190, 160, 255), width=2)
        d.polygon([(22, 10), (27, 4), (24, 12)], fill=(222, 216, 190, 255),
                  outline=(150, 160, 90, 255))                   # bone point
        for gx, gy in ((20, 13), (16, 17), (24, 9)):             # acid sheen
            d.point((gx, gy), fill=(140, 190, 80, 255))
        d.line([9, 22, 6, 26], fill=(222, 216, 190, 255), width=1)
        d.line([10, 21, 9, 26], fill=(200, 194, 168, 255), width=1)
    else:
        d.polygon([(21, 11), (28, 3), (23, 13)], fill=tip_col, outline=INK)
        for fx, fy in ((6, 23), (8, 25), (5, 26)):               # fletching
            d.polygon([(9, 22), (fx, fy), (11, 24)],
                      fill=(226, 226, 230, 255), outline=STEEL_DARK)
        d.arc([13, 14, 19, 20], start=300, end=120, fill=WOOD_D, width=1)


def draw_weighted_dart(d, rng):
    d.polygon([(15, 2), (17, 2), (16, 9)], fill=STEEL, outline=INK)  # point
    d.ellipse([13, 8, 19, 15], fill=(70, 70, 78, 255), outline=INK)  # lead
    d.rectangle([15, 15, 17, 23], fill=WOOD, outline=INK)        # shaft
    for fx in (12, 19):   # fletching fins
        d.polygon([(16, 22), (fx, 27), (16, 26)], fill=(200, 70, 60, 255),
                  outline=INK)
    d.point((16, 11), fill=(120, 120, 128, 255))


# --------------------------------------------------------------- leg rigs

def straps(d, x0, x1, y0=3):
    for sx in (x0 + 2, x1 - 3):
        d.line([sx, y0, sx, y0 + 4], fill=INK, width=2)


def draw_leg_bag(d, rng):
    straps(d, 9, 22)
    d.rectangle([9, 7, 22, 26], fill=(84, 84, 90, 255), outline=INK)
    d.rectangle([9, 7, 22, 13], fill=(70, 70, 76, 255))          # flap
    d.rectangle([14, 11, 17, 14], fill=STEEL_DARK, outline=INK)  # buckle
    d.rectangle([11, 18, 20, 24], outline=(60, 60, 66, 255))     # pocket
    d.point((20, 9), fill=WIRE_RED)


def draw_legrig(d, rng, wetsuit):
    if wetsuit:
        d.rectangle([6, 5, 26, 9], fill=(40, 42, 48, 255))       # neoprene band
        for x0 in (7, 17):
            d.rectangle([x0, 9, x0 + 8, 24], fill=(66, 70, 78, 255),
                        outline=INK)
            d.rectangle([x0, 9, x0 + 8, 14], fill=(52, 56, 64, 255))
            d.rectangle([x0 + 3, 12, x0 + 5, 15], fill=STEEL_DARK)
    else:
        straps(d, 11, 21)
        d.rectangle([11, 7, 21, 22], fill=(88, 92, 84, 255), outline=INK)
        d.rectangle([11, 7, 21, 12], fill=(72, 76, 68, 255))
        d.rectangle([14, 10, 18, 13], fill=STEEL_DARK, outline=INK)
        d.line([12, 17, 20, 17], fill=(60, 64, 56, 255), width=1)


def draw_net_backpack(d, rng):
    d.polygon([(9, 8), (23, 8), (25, 26), (7, 26)], fill=(70, 130, 70, 255),
              outline=INK)
    for i in range(-4, 9):   # net diamonds
        d.line([9 + i * 2, 8, 7 + i * 2, 26], fill=(44, 96, 44, 255), width=1)
        d.line([9 + i * 2, 8, 11 + i * 2, 26], fill=(44, 96, 44, 255), width=1)
    d.arc([11, 2, 21, 10], start=180, end=360, fill=INK, width=2)  # straps
    d.rectangle([13, 24, 19, 26], fill=(96, 70, 40, 255))        # contents hint


# --------------------------------------------------------------- masks

def draw_plague_mask(d, rng):
    d.ellipse([9, 4, 23, 18], fill=(128, 92, 58, 255), outline=INK)
    d.polygon([(13, 14), (19, 14), (17, 27), (15, 27)],
              fill=(110, 78, 48, 255), outline=INK)              # beak
    d.line([15, 18, 16, 24], fill=WOOD_D, width=1)
    for ex in (12, 18):   # brass-rimmed lenses
        d.ellipse([ex, 8, ex + 4, 12], fill=(40, 44, 50, 255), outline=BRONZE_D)
    d.line([9, 8, 4, 6], fill=WOOD_D, width=2)                   # straps
    d.line([23, 8, 28, 6], fill=WOOD_D, width=2)
    for sx in range(11, 22, 3):
        d.point((sx, 5), fill=WOOD_D)                            # stitching


def draw_pumpkin_mask(d, rng):
    d.ellipse([6, 7, 26, 28], fill=(232, 126, 34, 255), outline=INK)
    for x in (11, 16, 21):   # ribs
        d.arc([x - 5, 8, x + 5, 27], start=80, end=280,
              fill=(196, 100, 26, 255))
    d.rectangle([14, 3, 18, 7], fill=(70, 110, 50, 255), outline=INK)  # stem
    d.polygon([(10, 13), (14, 13), (12, 17)], fill=BLACK)        # eyes
    d.polygon([(18, 13), (22, 13), (20, 17)], fill=BLACK)
    d.polygon([(11, 21), (13, 24), (15, 21), (17, 24), (19, 21),
               (21, 24), (21, 21)], fill=BLACK)                  # jagged mouth
    d.point((12, 15), fill=(250, 200, 90, 255))


def draw_biocoif(d, rng):
    d.ellipse([8, 3, 24, 27], fill=(238, 238, 240, 255), outline=INK)
    for i in range(5):   # gold lattice
        d.line([9 + i * 3, 4, 8 + i * 3, 26], fill=GOLD, width=1)
        d.line([9, 6 + i * 5, 23, 5 + i * 5], fill=GOLD, width=1)
    d.ellipse([12, 10, 20, 20], fill=(226, 196, 168, 255), outline=GOLD_D)
    d.point((14, 14), fill=INK)                                  # eyes through
    d.point((18, 14), fill=INK)
    d.arc([14, 17, 18, 20], start=20, end=160, fill=INK)


def draw_zombie_mask(d, rng):
    d.ellipse([8, 3, 24, 28], fill=(196, 204, 178, 255), outline=INK)
    for ex in (11, 18):   # sunken eyes
        d.ellipse([ex, 10, ex + 4, 14], fill=(50, 54, 44, 255))
        d.point((ex + 1, 11), fill=(20, 22, 18, 255))
    d.line([16, 14, 15, 19], fill=(150, 158, 132, 255), width=1)
    for sx in range(11, 21, 2):   # stitched mouth
        d.line([sx, 23, sx + 1, 21], fill=(90, 96, 78, 255), width=1)
    d.line([11, 22, 21, 22], fill=(90, 96, 78, 255), width=1)
    d.polygon([(19, 5), (22, 7), (20, 9)], fill=(150, 70, 60, 255))  # wound
    d.point((10, 18), fill=(120, 128, 104, 255))                 # rot spot


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "broken_exodii_quad":
        draw_quad(d, rng)
    elif sid == "broken_exodii_sniper_drone":
        draw_drone(d, rng)
    elif sid == "broken_exodii_turret":
        draw_turret(d, rng)
    elif sid == "broken_exodii_worker":
        draw_worker(d, rng)
    elif sid == "legguard_metal_sheets_hip":
        draw_metal_skirt(d, rng, hard=False)
    elif sid == "ch_legguard_metal_sheets_hip":
        draw_metal_skirt(d, rng, hard=True)
    elif sid == "trousers_eod":
        draw_eod_trousers(d, rng, light=False)
    elif sid == "trousers_eod_light":
        draw_eod_trousers(d, rng, light=True)
    elif sid == "coin_highlands":
        draw_coin(d, rng)
    elif sid == "highland_funeral_mask":
        draw_funeral_mask(d, rng)
    elif sid == "highland_medallion":
        draw_medallion(d, rng)
    elif sid == "higlands_teleport":
        draw_landscape(d, rng)
    elif sid == "crystallizing_sugar":
        draw_crystallizing(d, rng)
    elif sid == "eggs_ferment":
        draw_ferment(d, rng, "eggs")
    elif sid == "pickles_ferment":
        draw_ferment(d, rng, "pickles")
    elif sid == "sauerkraut_ferment":
        draw_ferment(d, rng, "kraut")
    elif sid == "divemask_full_modular":
        draw_divemask(d, rng, modular=True)
    elif sid == "divemask_full_scuba_notready":
        draw_divemask(d, rng, modular=False)
    elif sid == "xl_wetsuit":
        draw_wetsuit(d, rng, big=True)
    elif sid == "xs_wetsuit":
        draw_wetsuit(d, rng, big=False)
    elif sid == "firefighter_scba_tank":
        draw_tank(d, rng, big=True, on=False)
    elif sid == "firefighter_scba_tank_on":
        draw_tank(d, rng, big=True, on=True)
    elif sid == "small_scuba_tank":
        draw_tank(d, rng, big=False, on=False)
    elif sid == "small_scuba_tank_on":
        draw_tank(d, rng, big=False, on=True)
    elif sid == "folded_bicycle":
        draw_folded_bicycle(d, rng)
    elif sid == "folded_skateboard_generic":
        draw_skateboard(d, rng)
    elif sid == "folded_wheelchair_generic":
        draw_folded_wheelchair(d, rng)
    elif sid == "generic_folded_vehicle":
        draw_generic_folded(d, rng)
    elif sid == "gasfilter_med":
        draw_gasfilter(d, rng, med=True)
    elif sid == "gasfilter_sm":
        draw_gasfilter(d, rng, med=False)
    elif sid == "rebreather_cartridge_air":
        draw_rebreather(d, rng, o2=False)
    elif sid == "rebreather_cartridge_o2":
        draw_rebreather(d, rng, o2=True)
    elif sid == "glock17_10":
        draw_magazine(d, rng, 21, (38, 38, 42, 255), 3)
    elif sid == "p226mag_10rd_9x19mm":
        draw_magazine(d, rng, 15, STEEL_DARK, 2)
    elif sid == "p226mag_15rd_9x19mm":
        draw_magazine(d, rng, 20, STEEL_DARK, 3)
    elif sid == "p228mag_13rd_9x19mm":
        draw_magazine(d, rng, 18, STEEL_DARK, 3)
    elif sid == "it_battery_mount":
        draw_battery_mount(d, rng)
    elif sid == "large_storage_battery":
        draw_battery(d, rng, "l")
    elif sid == "medium_storage_battery":
        draw_battery(d, rng, "m")
    elif sid == "small_storage_battery":
        draw_battery(d, rng, "s")
    elif sid == "javelin_fletched":
        draw_javelin(d, rng, STEEL)
    elif sid == "javelin_fletched_bronze":
        draw_javelin(d, rng, BRONZE)
    elif sid == "nether_huntsman_javelin":
        draw_javelin(d, rng, None, alien=True)
    elif sid == "weighted_dart":
        draw_weighted_dart(d, rng)
    elif sid == "leg_bag":
        draw_leg_bag(d, rng)
    elif sid == "legrig":
        draw_legrig(d, rng, wetsuit=False)
    elif sid == "legrig_wetsuit":
        draw_legrig(d, rng, wetsuit=True)
    elif sid == "net_backpack":
        draw_net_backpack(d, rng)
    elif sid == "mask_plague":
        draw_plague_mask(d, rng)
    elif sid == "pumpkin_mask":
        draw_pumpkin_mask(d, rng)
    elif sid == "string_dimension_biocoif":
        draw_biocoif(d, rng)
    elif sid == "zombie_mask":
        draw_zombie_mask(d, rng)
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
                       "preview_item85.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item85.png {prev.size}")


if __name__ == "__main__":
    main()
