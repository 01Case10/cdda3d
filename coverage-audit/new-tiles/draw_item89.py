#!/usr/bin/env python3
"""ITEM89: mixed tail -- launchers, mech lasers, hammocks, lunar veil,
power gear, cables, seafood, artisan weapons, holographic shards,
1911 mags, magnetron, platter, brewing states, monolith relics,
mutagen, offal, NL chem thrower, pressurized tanks, vehicle plates,
RM13 armor, scythes, bench seats, slings, sourdough, garments and
throwable extinguishers -- CDDA Voxel coverage audit.

Each item is drawn as itself from its game description.  Steel
grades follow the ITEM47 language where a grade is named: lc mild
= light matte gray, hc high = bright polished with white glint.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "crude_40mm",
    "pseudo_m203",
    "drum_percussion",
    "drum_percussion_primitive",
    "gatling_mech_laser",
    "recon_mech_laser",
    "hammock",
    "hammock_net",
    "helmet_sci_fi",
    "helmet_sci_fi_lowered",
    "inflatable_airbag",
    "sail",
    "it_handheld_battery_mount",
    "power_station_portable",
    "jumper_cable_debug",
    "power_cord",
    "lobster",
    "mollusk",
    "louverture",
    "pulverizer",
    "ls_sd_passcode",
    "ls_sd_passcode_used",
    "m1911bigmag",
    "m1911mag",
    "magnetron",
    "melchior_platter",
    "malting_grain",
    "soaking_dandelion",
    "monolith_brick",
    "monolith_disc",
    "mutagen",
    "offal",
    "nl_chem_thrower_ammo",
    "nl_chem_thrower_mag",
    "pressurized_tank",
    "pressurized_tank_modern",
    "rebar_plate",
    "spring_plate",
    "rm13_armor_on",
    "rm13_armor_rebreather_on",
    "scythe_fake",
    "scythe_toy",
    "seat_bench",
    "seat_bench_leather",
    "shoulder_strap_front",
    "shoulder_strap_simple",
    "sourdough_split_uncovered",
    "sourdough_young_uncovered",
    "thermal_shirt",
    "unitard",
    "throw_extinguisher",
    "throw_extinguisher_act",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 44, 50, 255)
STEEL = (128, 132, 140, 255)
STEEL_DARK = (84, 88, 96, 255)
LC_STEEL = (168, 170, 172, 255)      # lc mild: light matte gray
HC_STEEL = (206, 212, 220, 255)      # hc high: bright polished
WOOD = (110, 78, 44, 255)
WOOD_DARK = (84, 58, 32, 255)
WOOD_LIGHT = (150, 112, 70, 255)
BLACK = (30, 30, 34, 255)
PLASTIC_GRAY = (96, 98, 104, 255)
RED = (188, 44, 38, 255)
ORANGE = (228, 116, 40, 255)
GOLD = (216, 170, 60, 255)
CYAN = (72, 200, 210, 255)
TEAL = (44, 140, 132, 255)
GREEN = (74, 148, 70, 255)
CREAM = (238, 230, 210, 255)
TAN = (196, 168, 124, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- launchers

def draw_crude_40mm(d, rng):
    # side profile: wooden stock left, fat launcher tube right
    d.polygon([(3, 15), (11, 14), (11, 19), (5, 21), (3, 19)], fill=WOOD)
    d.line([4, 20, 10, 15], fill=WOOD_DARK, width=1)
    d.rectangle([10, 13, 17, 20], fill=STEEL_DARK)          # receiver pipe
    d.rectangle([16, 12, 28, 19], fill=STEEL)               # barrel tube
    d.rectangle([26, 11, 29, 20], fill=STEEL_DARK)          # muzzle ring
    d.ellipse([27, 13, 29, 18], fill=BLACK)                 # bore
    for bx in (18, 22):                                     # crude weld bands
        d.line([bx, 12, bx, 19], fill=shade(STEEL, 0.7), width=1)
        d.point((bx, 12), fill=LC_STEEL)
    d.rectangle([13, 20, 16, 24], fill=WOOD_DARK)           # grip
    d.point((19, 14), fill=HC_STEEL)                        # glint


def draw_pseudo_m203(d, rng):
    # array of six M203 tubes in a UGV mounting frame, muzzles forward
    d.rectangle([4, 5, 27, 26], fill=STEEL_DARK)
    d.rectangle([4, 5, 27, 7], fill=STEEL)                  # top rail
    for r in range(2):
        for c in range(3):
            cx, cy = 8 + c * 7, 12 + r * 8
            d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=STEEL)
            d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=BLACK)
            d.point((cx - 1, cy - 2), fill=HC_STEEL)
    d.rectangle([12, 26, 19, 29], fill=BLACK)               # vehicle mount
    d.line([5, 24, 26, 24], fill=shade(STEEL, 0.75), width=1)


# --------------------------------------------------------------------- drums

def draw_drum(d, rng, sid):
    prim = sid.endswith("primitive")
    shell = WOOD if not prim else WOOD_DARK
    head = CREAM if not prim else (216, 192, 152, 255)      # nylon vs rawhide
    d.rectangle([8, 11, 23, 27], fill=shell)
    for x in range(10, 23, 3):                              # shell shading
        d.line([x, 12, x, 26], fill=shade(shell, 0.88), width=1)
    d.ellipse([7, 7, 24, 14], fill=head)                    # drum head
    d.ellipse([7, 7, 24, 14], outline=INK, width=1)
    d.ellipse([11, 9, 20, 12], outline=shade(head, 0.85), width=1)
    if prim:
        # rope tension zigzag, leather
        for x in range(9, 23, 4):
            d.line([x, 13, x + 2, 19], fill=TAN, width=1)
            d.line([x + 2, 19, x, 25], fill=TAN, width=1)
        d.line([8, 13, 23, 13], fill=TAN, width=1)
        d.line([8, 25, 23, 25], fill=TAN, width=1)
    else:
        d.rectangle([7, 12, 24, 14], fill=STEEL)            # hoops
        d.rectangle([7, 24, 24, 26], fill=STEEL)
        for x in (10, 15, 20):                              # tension rods/lugs
            d.rectangle([x, 15, x + 1, 23], fill=STEEL_DARK)
            d.point((x, 18), fill=HC_STEEL)


# --------------------------------------------------------------- mech lasers

def draw_gatling(d, rng):
    # rotary barrel cluster seen from the front, finned housing
    d.rectangle([10, 24, 21, 29], fill=PLASTIC_GRAY)        # mount base
    d.ellipse([5, 5, 26, 26], fill=STEEL_DARK)              # housing ring
    d.ellipse([7, 7, 24, 24], fill=STEEL)
    for k in range(6):                                      # six barrels
        a = k * math.pi / 3
        cx, cy = 15.5 + 6 * math.cos(a), 15.5 + 6 * math.sin(a)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_DARK)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=BLACK)
        d.point((int(cx), int(cy)), fill=RED)               # emitter cores
    d.ellipse([13, 13, 18, 18], fill=STEEL_DARK)            # hub
    d.point((15, 15), fill=CYAN)
    for k in range(8):                                      # cooling fins
        a = k * math.pi / 4 + 0.4
        x0, y0 = 15.5 + 11 * math.cos(a), 15.5 + 11 * math.sin(a)
        x1, y1 = 15.5 + 13 * math.cos(a), 15.5 + 13 * math.sin(a)
        d.line([x0, y0, x1, y1], fill=STEEL_DARK, width=1)


def draw_recon(d, rng):
    # long marksman laser rifle in profile
    d.rectangle([3, 17, 9, 21], fill=BLACK)                 # stock
    d.rectangle([8, 15, 20, 20], fill=PLASTIC_GRAY)         # receiver
    d.rectangle([19, 16, 29, 18], fill=STEEL_DARK)          # shrouded barrel
    d.rectangle([28, 15, 30, 19], fill=STEEL)               # emitter housing
    d.point((29, 17), fill=CYAN)                            # emitter lens
    d.rectangle([12, 11, 18, 14], fill=BLACK)               # scope
    d.ellipse([17, 12, 19, 14], fill=CYAN)
    d.rectangle([13, 20, 16, 24], fill=BLACK)               # grip
    d.rectangle([21, 20, 23, 25], fill=STEEL_DARK)          # magazine cell
    d.line([9, 16, 19, 16], fill=shade(PLASTIC_GRAY, 1.25), width=1)


# ------------------------------------------------------------------ hammocks

def draw_hammock(d, rng, sid):
    net = sid == "hammock_net"
    col = (150, 158, 120, 255) if not net else (170, 176, 160, 255)
    # ropes fanning from two tie points to the cloth ends
    for sx, ex in ((2, 9), (29, 22)):
        for dy in (-3, 0, 3):
            d.line([sx, 8, ex, 13 + dy], fill=TAN, width=1)
    # sagging body: stacked arcs
    for i in range(6):
        y = 13 + i
        inset = int(3.2 * math.sin(math.pi * i / 6))
        d.line([9 + (inset if i else 0), y, 22 - (inset if i else 0), y],
               fill=shade(col, 1 - i * 0.05), width=1)
    d.arc([7, 8, 24, 22], start=20, end=160, fill=shade(col, 0.8), width=1)
    if net:
        for x in range(10, 22, 3):                          # diamond mesh
            d.line([x, 13, x + 3, 19], fill=shade(col, 0.65), width=1)
            d.line([x + 3, 13, x, 19], fill=shade(col, 0.65), width=1)
    else:
        for x in (12, 16, 20):                              # cloth folds
            d.line([x, 14, x - 1, 18], fill=shade(col, 0.8), width=1)


# --------------------------------------------------------------- lunar veils

def draw_veil(d, rng, sid):
    lowered = sid.endswith("lowered")
    d.ellipse([4, 6, 27, 12], fill=(176, 168, 150, 255))    # wide brim
    d.ellipse([9, 2, 22, 9], fill=(196, 188, 168, 255))     # dome
    d.line([10, 4, 21, 4], fill=(222, 216, 196, 255), width=1)
    if lowered:
        # veil gathered: crinkled scarf bunched under the chin
        d.rectangle([11, 12, 20, 17], fill=(214, 200, 178, 255))  # face
        for y in (19, 21, 23):
            d.arc([9, y - 3, 22, y + 3], start=0, end=180,
                  fill=(188, 178, 158, 255), width=2)
        d.point((13, 14), fill=INK)
        d.point((18, 14), fill=INK)
    else:
        # veil hanging: latticed cloth over the face, crescent eye-slit
        d.rectangle([10, 11, 21, 26], fill=(206, 198, 178, 255))
        for x in range(11, 21, 3):
            d.line([x, 12, x, 25], fill=(168, 160, 140, 255), width=1)
        for y in range(13, 26, 3):
            d.line([11, y, 20, y], fill=(168, 160, 140, 255), width=1)
        d.arc([12, 13, 19, 19], start=300, end=120, fill=GOLD, width=2)
        d.point((16, 15), fill=GOLD)                       # crescent slit


# -------------------------------------------------------------- boat / power

def draw_airbag(d, rng):
    d.rounded_rectangle([6, 9, 26, 24], radius=6, fill=(196, 208, 226, 255))
    d.rounded_rectangle([6, 9, 26, 24], radius=6, outline=(140, 156, 180, 255),
                        width=1)
    d.arc([9, 11, 23, 22], start=200, end=320, fill=(226, 234, 244, 255),
          width=2)                                          # sheen
    d.line([8, 17, 24, 17], fill=(160, 176, 198, 255), width=1)  # seam
    d.ellipse([22, 20, 25, 23], fill=(120, 136, 160, 255))  # valve


def draw_sail(d, rng):
    d.line([9, 3, 9, 28], fill=WOOD, width=2)               # mast
    d.line([9, 26, 26, 26], fill=WOOD, width=1)             # boom
    d.polygon([(11, 5), (11, 24), (25, 24)], fill=CREAM)    # sail
    d.polygon([(11, 5), (11, 24), (25, 24)], outline=(180, 170, 148, 255))
    for t in (0.33, 0.66):                                  # seam lines
        d.line([11, 5 + 19 * t, 11 + 14 * t, 24],
               fill=(206, 198, 172, 255), width=1)
    d.line([11, 5, 25, 24], fill=(216, 208, 184, 255), width=1)


def draw_battery_mount(d, rng):
    d.rectangle([6, 10, 20, 24], fill=PLASTIC_GRAY)         # mount housing
    d.rectangle([8, 6, 18, 13], fill=ORANGE)                # tool battery pack
    d.rectangle([8, 6, 18, 8], fill=BLACK)
    d.rectangle([21, 14, 27, 22], fill=STEEL_DARK)          # converter box
    d.point((24, 17), fill=GREEN)                           # charge LED
    pts = [(20, 18), (23, 20), (21, 22), (24, 23)]          # wires
    d.line(pts, fill=(176, 60, 40, 255), width=1)
    d.line([7, 24, 7, 28], fill=BLACK, width=1)             # harness strap
    d.line([19, 24, 19, 28], fill=BLACK, width=1)


def draw_power_station(d, rng):
    d.rectangle([6, 10, 26, 27], fill=(104, 108, 116, 255))
    d.rectangle([6, 10, 26, 27], outline=INK, width=1)
    d.arc([11, 4, 21, 12], start=180, end=360, fill=INK, width=2)  # handle
    d.rectangle([9, 14, 15, 20], fill=(226, 228, 230, 255))  # AC outlet
    d.point((11, 16), fill=INK)
    d.point((13, 16), fill=INK)
    d.point((12, 18), fill=INK)
    d.rectangle([18, 14, 24, 18], fill=TEAL)                # LCD
    d.line([19, 16, 23, 16], fill=(180, 240, 235, 255), width=1)
    for x in (19, 21, 23):                                  # USB ports
        d.rectangle([x, 21, x + 1, 23], fill=BLACK)
    for y in (22, 24):                                      # vents
        d.line([9, y, 14, y], fill=shade(PLASTIC_GRAY, 0.7), width=1)


# -------------------------------------------------------------------- cables

def draw_cable(d, rng, sid):
    debug = sid == "jumper_cable_debug"
    col = (120, 220, 235, 255) if debug else (52, 52, 56, 255)
    # coiled loops
    for r in (9, 6, 3):
        d.ellipse([16 - r, 15 - r, 16 + r, 15 + r], outline=col, width=2)
    d.line([22, 20, 27, 25], fill=col, width=2)             # tail
    if debug:
        d.rectangle([25, 24, 29, 27], fill=RED)             # clamp
        d.rectangle([3, 24, 7, 27], fill=INK)
        for px, py in ((6, 5), (25, 7), (10, 26), (27, 14), (4, 13)):
            d.point((px, py), fill=(255, 255, 255, 255))    # shimmer
            d.point((px + 1, py), fill=(210, 160, 240, 255))
    else:
        d.rectangle([26, 24, 29, 26], fill=BLACK)           # outlet block
        d.point((27, 25), fill=(200, 200, 200, 255))
        d.point((28, 25), fill=(200, 200, 200, 255))
        d.rectangle([4, 22, 6, 27], fill=BLACK)             # plug
        d.line([5, 20, 5, 22], fill=LC_STEEL, width=1)      # prongs
        d.line([6, 20, 6, 22], fill=LC_STEEL, width=1)


# ------------------------------------------------------------------- seafood

def draw_lobster(d, rng):
    # tail-meat chunk: curled segmented fan, shell remnant on top
    d.polygon([(10, 8), (22, 8), (24, 13), (20, 15), (12, 15), (8, 13)],
              fill=(206, 92, 66, 255))                      # shell
    d.polygon([(11, 13), (21, 13), (23, 20), (19, 26), (13, 26), (9, 20)],
              fill=(244, 214, 196, 255))                    # meat
    for y in (17, 20, 23):                                  # segments
        d.arc([10, y - 4, 22, y + 3], start=20, end=160,
              fill=(226, 168, 148, 255), width=1)
    d.point((13, 15), fill=(255, 240, 230, 255))


def draw_mollusk(d, rng):
    # raw fillet slab with darker mantle rim
    d.ellipse([6, 11, 26, 23], fill=(168, 152, 138, 255))
    d.ellipse([6, 11, 26, 23], outline=(120, 104, 90, 255), width=1)
    d.ellipse([9, 13, 23, 21], fill=(188, 174, 158, 255))
    d.arc([10, 13, 22, 21], start=180, end=330, fill=(214, 202, 186, 255),
          width=2)                                          # sheen
    d.point((20, 19), fill=(140, 124, 108, 255))


# ------------------------------------------------------------ artisan weapons

def draw_louverture(d, rng):
    # vented mace, diagonal; fuel reservoir in the pommel
    d.line([11, 27, 21, 9], fill=WOOD_DARK, width=3)        # haft
    d.polygon([(18, 4), (24, 6), (25, 12), (20, 14), (17, 10)],
              fill=STEEL)                                   # flanged head
    for px, py in ((20, 7), (22, 10), (19, 11)):            # vents
        d.point((px, py), fill=BLACK)
        d.point((px, py - 1), fill=ORANGE)                  # heat glow
    d.ellipse([8, 25, 13, 30], fill=STEEL_DARK)             # pommel tank
    d.point((10, 27), fill=ORANGE)                          # fuel cap


def draw_pulverizer(d, rng):
    # big sledge: boxy head with firing mechanism, long handle
    d.line([8, 27, 22, 12], fill=WOOD, width=3)             # handle
    d.rectangle([17, 4, 28, 13], fill=STEEL_DARK)           # head
    d.rectangle([17, 4, 21, 13], fill=STEEL)                # striking face
    d.rectangle([24, 6, 29, 10], fill=BLACK)                # firing mech
    d.point((26, 8), fill=RED)                              # safety
    d.line([18, 5, 20, 5], fill=HC_STEEL, width=1)
    d.rectangle([12, 19, 14, 21], fill=STEEL_DARK)          # grip band


# --------------------------------------------------------------------- shards

def draw_shard(d, rng, sid):
    used = sid.endswith("_used")
    pts = [(13, 3), (20, 8), (22, 17), (17, 28), (11, 22), (10, 10)]
    base = (120, 140, 150, 255) if used else (150, 225, 235, 255)
    d.polygon(pts, fill=base)
    d.polygon(pts, outline=shade(base, 0.7))
    if used:
        d.line([13, 6, 17, 14], fill=shade(base, 0.75), width=1)  # crack
        d.line([17, 14, 15, 21], fill=shade(base, 0.75), width=1)
        d.line([12, 10, 20, 12], fill=shade(base, 1.15), width=1)
    else:
        bands = [(200, 120, 220, 255), (250, 210, 110, 255),
                 (110, 240, 180, 255)]
        for i, bcol in enumerate(bands):                    # prismatic bands
            d.line([12 + i, 8 + i * 5, 20 - i, 10 + i * 5], fill=bcol,
                   width=2)
        d.ellipse([14, 13, 19, 19], outline=(255, 255, 255, 255),
                  width=1)                                  # something within
        d.point((16, 16), fill=(255, 255, 255, 255))
        d.point((11, 6), fill=(255, 255, 255, 255))


# ------------------------------------------------------------------ magazines

def draw_1911mag(d, rng, sid):
    big = sid == "m1911bigmag"
    top = 5 if big else 10
    d.rectangle([12, top, 19, 26], fill=STEEL)
    d.rectangle([12, top, 19, 26], outline=STEEL_DARK, width=1)
    d.rectangle([13, top, 15, 25], fill=shade(STEEL, 0.85))  # flat shading
    for y in range(top + 4, 23, 4):                          # witness holes
        d.point((17, y), fill=BLACK)
    d.rectangle([13, top - 2 if big else top - 2, 18, top],
                fill=(196, 158, 84, 255))                    # top cartridge
    if big:
        d.rectangle([11, 26, 20, 29], fill=BLACK)            # extended base
    else:
        d.rectangle([11, 26, 20, 28], fill=STEEL_DARK)       # baseplate


# --------------------------------------------------------------- electronics

def draw_magnetron(d, rng):
    d.rectangle([13, 10, 19, 26], fill=STEEL)               # anode block
    for y in range(11, 26, 3):                              # cooling fins
        d.rectangle([9, y, 23, y + 1], fill=STEEL_DARK)
    d.rectangle([14, 5, 18, 10], fill=(226, 214, 200, 255))  # ceramic seal
    d.line([16, 2, 16, 5], fill=(176, 108, 60, 255), width=2)  # terminal
    d.ellipse([14, 16, 18, 21], fill=BLACK)                 # cavity port
    d.point((15, 12), fill=HC_STEEL)


def draw_platter(d, rng):
    d.ellipse([4, 9, 28, 24], fill=GOLD)
    d.ellipse([4, 9, 28, 24], outline=shade(GOLD, 0.7), width=1)
    for r in (10, 7, 4):                                    # circuit rings
        d.ellipse([16 - r, 16 - r // 2, 16 + r, 17 + r // 2],
                  outline=shade(GOLD, 0.72), width=1)
    for a in range(0, 360, 45):                             # traces
        x = 16 + int(9 * math.cos(math.radians(a)))
        y = 16 + int(5 * math.sin(math.radians(a)))
        d.point((x, y), fill=shade(GOLD, 0.6))
    d.ellipse([14, 14, 18, 18], fill=shade(GOLD, 0.8))      # hub
    d.arc([6, 10, 20, 18], start=180, end=300, fill=(250, 226, 150, 255),
          width=2)                                          # glint


# ------------------------------------------------------------------- brewing

def draw_malting_grain(d, rng):
    d.polygon([(7, 20), (25, 20), (22, 28), (10, 28)], fill=(150, 116, 74, 255))
    d.ellipse([6, 15, 26, 22], fill=(206, 176, 116, 255))   # grain heap
    for _ in range(26):                                     # kernels
        x, y = rng.randint(8, 24), rng.randint(16, 21)
        d.point((x, y), fill=(188, 156, 96, 255))
    for x in (11, 16, 21):                                  # sprouts
        d.line([x, 16, x, 13], fill=(240, 240, 220, 255), width=1)
        d.point((x, 12), fill=GREEN)


def draw_soaking_dandelion(d, rng):
    d.rectangle([10, 8, 22, 28], fill=(150, 190, 205, 255))  # jar of water
    d.rectangle([10, 8, 22, 28], outline=(110, 140, 155, 255), width=1)
    d.rectangle([10, 8, 22, 10], fill=(190, 215, 225, 255))  # rim
    for cx, cy in ((13, 14), (18, 17), (14, 22)):            # flowers soaking
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=(240, 205, 70, 255))
        d.point((cx, cy), fill=ORANGE)
    d.arc([16, 18, 22, 26], start=90, end=200, fill=GREEN, width=2)  # leaves
    for px, py in ((12, 12), (20, 20), (15, 25)):            # bubbles
        d.point((px, py), fill=(235, 248, 252, 255))


# ------------------------------------------------------------------ monolith

def draw_monolith_brick(d, rng):
    # warped brick: slightly skewed, perfectly smooth dark stone
    d.polygon([(7, 12), (26, 10), (25, 22), (6, 23)], fill=(52, 50, 56, 255))
    d.polygon([(7, 12), (26, 10), (27, 7), (9, 8)], fill=(66, 64, 72, 255))
    d.line([9, 9, 26, 8], fill=(84, 82, 92, 255), width=1)  # top sheen
    d.line([7, 21, 24, 20], fill=(40, 38, 44, 255), width=1)


def draw_monolith_disc(d, rng):
    d.ellipse([7, 6, 25, 26], fill=(52, 50, 56, 255))
    d.ellipse([7, 6, 25, 26], outline=(30, 28, 34, 255), width=1)
    for r in (7, 4):                                        # engraved swirls
        d.arc([16 - r, 16 - r, 16 + r, 16 + r], start=40, end=300,
              fill=(96, 94, 104, 255), width=1)
    d.arc([13, 13, 19, 19], start=200, end=80, fill=(110, 108, 118, 255),
          width=1)
    d.point((16, 16), fill=(120, 118, 128, 255))


# ------------------------------------------------------------- mutagen/offal

def draw_mutagen(d, rng):
    d.polygon([(14, 10), (18, 10), (22, 18), (24, 27), (8, 27), (10, 18)],
              fill=(120, 170, 120, 255))                    # flask glass tint
    d.polygon([(11, 19), (21, 19), (23, 26), (9, 26)],
              fill=(96, 148, 74, 255))                      # slurry
    d.arc([12, 20, 20, 26], start=0, end=180, fill=(150, 96, 170, 255),
          width=2)                                          # purple swirl
    d.rectangle([13, 6, 19, 10], fill=(150, 120, 84, 255))  # cork
    for px, py in ((13, 22), (18, 24), (15, 21)):           # bubbles
        d.point((px, py), fill=(190, 230, 170, 255))
    d.polygon([(14, 10), (18, 10), (22, 18), (24, 27), (8, 27), (10, 18)],
              outline=(80, 120, 84, 255))


def draw_offal(d, rng):
    d.polygon([(9, 20), (15, 12), (23, 14), (24, 22), (18, 27), (10, 25)],
              fill=(172, 84, 78, 255))                      # organ pile
    for r in (5, 3):                                        # intestine coil
        d.ellipse([16 - r, 19 - r // 2, 16 + r, 20 + r // 2],
                  outline=(226, 156, 148, 255), width=1)
    d.polygon([(10, 14), (16, 12), (17, 18), (11, 19)],
              fill=(128, 52, 48, 255))                      # liver chunk
    d.ellipse([19, 21, 24, 26], fill=(150, 66, 66, 255))    # kidney
    d.point((13, 15), fill=(240, 200, 190, 255))            # glisten
    d.point((20, 16), fill=(240, 200, 190, 255))


# -------------------------------------------------------------- chem thrower

def draw_chem_ammo(d, rng):
    d.rectangle([12, 9, 20, 27], fill=(64, 58, 66, 255))    # charge canister
    d.rectangle([13, 13, 19, 24], fill=(110, 230, 130, 255))  # glowing chem
    d.rectangle([13, 13, 19, 15], fill=(190, 250, 200, 255))
    d.rectangle([13, 5, 19, 9], fill=(210, 200, 185, 255))  # bone cap
    for y in (17, 21):                                      # bubbles
        d.point((16, y), fill=(220, 255, 225, 255))
    d.line([12, 27, 20, 27], fill=INK, width=1)


def draw_chem_mag(d, rng):
    d.rectangle([9, 10, 23, 26], fill=(216, 206, 188, 255))  # dense bone pack
    d.rectangle([9, 10, 23, 26], outline=(170, 158, 138, 255), width=1)
    for x in (12, 16, 20):                                  # gum-like terminals
        d.ellipse([x - 2, 5, x + 2, 10], fill=(196, 130, 110, 255))
        d.point((x, 7), fill=(200, 130, 70, 255))           # copper core
    for y in (15, 20):                                      # bone seams
        d.line([10, y, 22, y], fill=(186, 174, 154, 255), width=1)
    d.point((12, 13), fill=(236, 228, 210, 255))


# --------------------------------------------------------------------- tanks

def draw_tank(d, rng, sid):
    modern = sid.endswith("modern")
    body = (36, 36, 40, 255) if modern else (178, 182, 188, 255)
    d.rectangle([11, 9, 21, 28], fill=body)
    d.ellipse([11, 6, 21, 12], fill=shade(body, 1.12))
    if modern:                                              # carbon weave
        for y in range(12, 28, 3):
            d.line([11, y, 21, y], fill=(58, 58, 64, 255), width=1)
        for x in range(13, 21, 3):
            d.line([x, 11, x, 28], fill=(52, 52, 58, 255), width=1)
        d.rectangle([14, 3, 18, 7], fill=STEEL_DARK)        # valve
        d.ellipse([17, 3, 21, 7], outline=CYAN, width=1)    # gauge
    else:                                                   # 1970s aluminum
        for y in (14, 21):
            d.line([11, y, 21, y], fill=shade(body, 0.85), width=1)
        for x in (14, 17):                                  # date-code stamp
            d.line([x, 24, x, 26], fill=shade(body, 0.6), width=1)
        d.rectangle([14, 3, 18, 7], fill=STEEL_DARK)
        d.line([18, 5, 23, 8], fill=STEEL_DARK, width=2)    # hose stub
    d.line([12, 10, 12, 27], fill=shade(body, 1.25), width=1)


# -------------------------------------------------------------- vehicle plates

def draw_rebar_plate(d, rng):
    # interlocked rebar grid, lc steel: light matte gray
    for x in range(7, 26, 4):
        d.line([x, 6, x, 26], fill=LC_STEEL, width=2)
        for y in range(8, 26, 4):                           # ribbing
            d.point((x - 1, y), fill=shade(LC_STEEL, 0.8))
    for y in range(6, 27, 4):
        d.line([6, y, 26, y], fill=shade(LC_STEEL, 0.92), width=2)
    for x in range(7, 26, 4):                               # weld blobs
        for y in range(6, 27, 4):
            d.point((x, y), fill=shade(LC_STEEL, 0.7))


def draw_spring_plate(d, rng):
    d.rectangle([6, 22, 26, 27], fill=STEEL_DARK)           # scrap base plate
    for cx in (10, 16, 22):                                 # coil springs, hc
        pts = []
        for i in range(9):
            pts.append((cx + (2 if i % 2 else -2), 21 - i * 2))
        d.line(pts, fill=HC_STEEL, width=1)
        d.point((cx, 5), fill=(255, 255, 255, 255))         # polished glint
    d.rectangle([6, 3, 26, 6], fill=LC_STEEL)               # top plate
    for x in (8, 24):
        d.point((x, 24), fill=BLACK)                        # bolts


# ------------------------------------------------------------------ RM13 suit

def draw_rm13(d, rng, sid):
    reb = sid.endswith("rebreather_on")
    suit = (26, 26, 30, 255)
    seam = (60, 130, 140, 255)                              # faint power seams
    d.ellipse([12, 3, 20, 11], fill=suit)                   # helmet
    d.rectangle([13, 7, 19, 9], fill=(120, 30, 30, 255))    # visor slit
    d.rectangle([10, 11, 22, 21], fill=suit)                # torso
    d.rectangle([6, 12, 10, 23], fill=suit)                 # arms
    d.rectangle([22, 12, 26, 23], fill=suit)
    d.rectangle([11, 21, 15, 29], fill=suit)                # legs
    d.rectangle([17, 21, 21, 29], fill=suit)
    for pts in (((16, 12), (16, 20)), ((11, 16), (21, 16)),
                ((8, 14), (8, 21)), ((24, 14), (24, 21))):
        d.line(list(pts), fill=seam, width=1)               # panel seams
    d.point((16, 14), fill=CYAN)                            # chest core
    if reb:
        d.rectangle([12, 8, 20, 12], fill=(40, 40, 44, 255))  # rebreather mask
        d.ellipse([10, 9, 13, 13], fill=STEEL_DARK)         # filter cans
        d.ellipse([19, 9, 22, 13], fill=STEEL_DARK)
        d.arc([12, 10, 20, 19], start=0, end=180, fill=STEEL_DARK, width=1)


# -------------------------------------------------------------------- scythes

def draw_scythe(d, rng, sid):
    toy = sid == "scythe_toy"
    d.line([10, 29, 16, 6], fill=(20, 20, 22, 255) if toy else WOOD,
           width=2)                                         # snath
    d.line([12, 20, 8, 18], fill=WOOD_DARK, width=1)        # handles
    d.line([14, 13, 10, 12], fill=WOOD_DARK, width=1)
    if toy:                                                 # crooked plastic
        d.arc([14, 2, 30, 14], start=180, end=330, fill=(60, 60, 66, 255),
              width=3)
        d.ellipse([21, 5, 25, 9], outline=(230, 230, 230, 255), width=1)
        d.point((22, 6), fill=(230, 230, 230, 255))         # skull engraving
        d.point((24, 6), fill=(230, 230, 230, 255))
    else:                                                   # dull aluminum
        d.arc([14, 2, 30, 14], start=180, end=330, fill=(170, 174, 178, 255),
              width=3)
        d.arc([14, 2, 30, 14], start=180, end=330, fill=(140, 144, 148, 255),
              width=1)


# ---------------------------------------------------------------------- seats

def draw_bench(d, rng, sid):
    leather = sid.endswith("leather")
    col = (128, 88, 58, 255) if leather else (110, 122, 140, 255)
    d.rectangle([5, 8, 27, 19], fill=col)                   # backrest
    d.rectangle([5, 8, 27, 19], outline=shade(col, 0.7), width=1)
    d.rectangle([4, 19, 28, 25], fill=shade(col, 1.08))     # cushion
    d.rectangle([4, 19, 28, 25], outline=shade(col, 0.7), width=1)
    d.line([16, 9, 16, 18], fill=shade(col, 0.75), width=1)  # centre seam
    if leather:
        for x in (8, 24):
            d.line([x, 10, x, 17], fill=(196, 160, 120, 255), width=1)
        d.line([6, 21, 26, 21], fill=(196, 160, 120, 255), width=1)
    else:
        for y in (12, 15):
            d.line([6, y, 26, y], fill=shade(col, 0.88), width=1)
    d.rectangle([7, 25, 9, 28], fill=LC_STEEL)              # frame legs
    d.rectangle([23, 25, 25, 28], fill=LC_STEEL)


# ---------------------------------------------------------------------- slings

def draw_sling(d, rng, sid):
    if sid == "shoulder_strap_front":
        col = (70, 74, 66, 255)
        # front-hang loop: two straps down from shoulders to a low join
        d.line([10, 4, 13, 22], fill=col, width=3)
        d.line([22, 4, 19, 22], fill=col, width=3)
        d.line([13, 22, 19, 22], fill=col, width=2)
        d.rectangle([9, 10, 12, 13], fill=STEEL_DARK)       # adjust buckles
        d.rectangle([20, 10, 23, 13], fill=STEEL_DARK)
        d.point((16, 24), fill=STEEL)                       # rifle clip
        d.line([16, 22, 16, 24], fill=STEEL, width=1)
    else:
        col = (150, 138, 104, 255)                          # cotton webbing
        d.line([7, 6, 25, 24], fill=col, width=3)           # diagonal strap
        d.line([7, 6, 25, 24], fill=shade(col, 0.85), width=1)
        d.ellipse([5, 4, 9, 8], outline=STEEL, width=1)     # snap hooks
        d.ellipse([23, 22, 27, 26], outline=STEEL, width=1)


# ------------------------------------------------------------------ sourdough

def draw_sourdough(d, rng, sid):
    split = "split" in sid
    d.rectangle([10, 12, 22, 28], fill=(196, 206, 210, 255))  # glass jar
    d.rectangle([10, 12, 22, 28], outline=(140, 152, 156, 255), width=1)
    d.rectangle([11, 17, 21, 27], fill=(232, 222, 196, 255))  # paste
    d.ellipse([10, 14, 22, 19], fill=(238, 230, 206, 255))    # surface
    if split:
        d.line([12, 16, 20, 16], fill=(196, 182, 148, 255), width=1)
        d.line([15, 15, 17, 17], fill=(196, 182, 148, 255), width=1)
        for px, py in ((13, 18), (18, 20), (15, 23), (19, 25), (12, 24)):
            d.point((px, py), fill=(206, 196, 164, 255))      # bubbles
    else:
        for px, py in ((14, 21), (18, 24)):
            d.point((px, py), fill=(210, 200, 170, 255))      # few bubbles
        for px in (12, 15, 19):                               # flour dusting
            d.point((px, 15), fill=(250, 248, 240, 255))


# ------------------------------------------------------------------- garments

def draw_thermal_shirt(d, rng):
    col = (178, 182, 186, 255)                              # heather gray lycra
    d.polygon([(11, 6), (21, 6), (26, 10), (28, 20), (24, 21),
               (22, 14), (22, 27), (10, 27), (10, 14), (8, 21),
               (4, 20), (6, 10)], fill=col)
    d.arc([13, 4, 19, 9], start=0, end=180, fill=shade(col, 0.8), width=1)
    for x in (8, 24):                                       # ribbed cuffs
        d.line([x, 19, x, 21], fill=shade(col, 0.82), width=1)
    d.line([10, 25, 22, 25], fill=shade(col, 0.85), width=1)  # hem


def draw_unitard(d, rng):
    col = (106, 74, 150, 255)                               # purple lycra
    d.polygon([(11, 5), (21, 5), (23, 12), (21, 16), (21, 29),
               (17, 29), (16, 19), (15, 19), (14, 29), (10, 29),
               (10, 16), (8, 12)], fill=col)
    d.arc([13, 3, 19, 8], start=0, end=180, fill=shade(col, 0.75), width=1)
    d.line([11, 15, 20, 15], fill=shade(col, 0.8), width=1)  # waist seam
    d.line([12, 7, 12, 27], fill=shade(col, 1.15), width=1)  # sheen


# ---------------------------------------------------------------- extinguishers

def draw_extinguisher(d, rng, sid):
    act = sid.endswith("_act")
    d.rectangle([12, 11, 20, 27], fill=RED)                 # canister body
    d.rectangle([12, 11, 20, 27], outline=shade(RED, 0.7), width=1)
    d.rectangle([12, 18, 20, 22], fill=(240, 240, 240, 255))  # label band
    d.point((16, 20), fill=RED)
    d.rectangle([14, 7, 18, 11], fill=STEEL_DARK)           # neck
    if act:
        # plug pulled and foam bursting out
        d.ellipse([18, 2, 22, 6], fill=(200, 200, 205, 255))  # flying plug
        d.line([19, 6, 16, 8], fill=STEEL, width=1)
        for cx, cy, r in ((16, 4, 3), (11, 6, 2), (21, 7, 2), (16, 1, 2)):
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      fill=(250, 250, 250, 255))             # foam burst
        d.point((9, 10), fill=(250, 250, 250, 255))
        d.point((23, 11), fill=(250, 250, 250, 255))
    else:
        d.rectangle([14, 4, 18, 7], fill=(200, 200, 205, 255))  # plastic plug
        d.ellipse([19, 5, 23, 9], outline=STEEL, width=1)       # pin ring


# -------------------------------------------------------------------- dispatch

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "crude_40mm":
        draw_crude_40mm(d, rng)
    elif sid == "pseudo_m203":
        draw_pseudo_m203(d, rng)
    elif sid in ("drum_percussion", "drum_percussion_primitive"):
        draw_drum(d, rng, sid)
    elif sid == "gatling_mech_laser":
        draw_gatling(d, rng)
    elif sid == "recon_mech_laser":
        draw_recon(d, rng)
    elif sid in ("hammock", "hammock_net"):
        draw_hammock(d, rng, sid)
    elif sid in ("helmet_sci_fi", "helmet_sci_fi_lowered"):
        draw_veil(d, rng, sid)
    elif sid == "inflatable_airbag":
        draw_airbag(d, rng)
    elif sid == "sail":
        draw_sail(d, rng)
    elif sid == "it_handheld_battery_mount":
        draw_battery_mount(d, rng)
    elif sid == "power_station_portable":
        draw_power_station(d, rng)
    elif sid in ("jumper_cable_debug", "power_cord"):
        draw_cable(d, rng, sid)
    elif sid == "lobster":
        draw_lobster(d, rng)
    elif sid == "mollusk":
        draw_mollusk(d, rng)
    elif sid == "louverture":
        draw_louverture(d, rng)
    elif sid == "pulverizer":
        draw_pulverizer(d, rng)
    elif sid in ("ls_sd_passcode", "ls_sd_passcode_used"):
        draw_shard(d, rng, sid)
    elif sid in ("m1911bigmag", "m1911mag"):
        draw_1911mag(d, rng, sid)
    elif sid == "magnetron":
        draw_magnetron(d, rng)
    elif sid == "melchior_platter":
        draw_platter(d, rng)
    elif sid == "malting_grain":
        draw_malting_grain(d, rng)
    elif sid == "soaking_dandelion":
        draw_soaking_dandelion(d, rng)
    elif sid == "monolith_brick":
        draw_monolith_brick(d, rng)
    elif sid == "monolith_disc":
        draw_monolith_disc(d, rng)
    elif sid == "mutagen":
        draw_mutagen(d, rng)
    elif sid == "offal":
        draw_offal(d, rng)
    elif sid == "nl_chem_thrower_ammo":
        draw_chem_ammo(d, rng)
    elif sid == "nl_chem_thrower_mag":
        draw_chem_mag(d, rng)
    elif sid in ("pressurized_tank", "pressurized_tank_modern"):
        draw_tank(d, rng, sid)
    elif sid == "rebar_plate":
        draw_rebar_plate(d, rng)
    elif sid == "spring_plate":
        draw_spring_plate(d, rng)
    elif sid in ("rm13_armor_on", "rm13_armor_rebreather_on"):
        draw_rm13(d, rng, sid)
    elif sid in ("scythe_fake", "scythe_toy"):
        draw_scythe(d, rng, sid)
    elif sid in ("seat_bench", "seat_bench_leather"):
        draw_bench(d, rng, sid)
    elif sid in ("shoulder_strap_front", "shoulder_strap_simple"):
        draw_sling(d, rng, sid)
    elif sid in ("sourdough_split_uncovered", "sourdough_young_uncovered"):
        draw_sourdough(d, rng, sid)
    elif sid == "thermal_shirt":
        draw_thermal_shirt(d, rng)
    elif sid == "unitard":
        draw_unitard(d, rng)
    elif sid in ("throw_extinguisher", "throw_extinguisher_act"):
        draw_extinguisher(d, rng, sid)
    else:
        raise KeyError(sid)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    # the embedded list is authoritative-verified against the batch file
    batch = os.path.expanduser("~/workspace/item-batches/89.txt")
    with open(batch) as f:
        assert [ln.strip() for ln in f if ln.strip()] == IDS, \
            "embedded IDS differ from 89.txt"
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
                       "preview_item89.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item89.png {prev.size}")


if __name__ == "__main__":
    main()
