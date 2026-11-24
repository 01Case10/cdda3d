#!/usr/bin/env python3
"""ITEM65: industrial spares, wheels/rims and wheat foods -- CDDA Voxel
coverage audit.

Three families, each id drawn as itself from its game definition:

  Industrial spares (items/generic/spares.json, 18) -- steel pressure
    hardware and improvised repair parts: compressors as finned
    motor+tank units (industrial one much bulkier), thick-walled pipe
    told apart by what is built into it (plain / valve handwheel /
    pressure dial / membrane band), pressure vessels as capsule tanks
    (large horizontal on saddles, small vertical), pumps (complex =
    blocky flanged machine, well pump = round cast-iron volute),
    makeshift drive belt / air filter / liquid filter / 5-point
    harness / crank / pyrometer / tire repair kit as themselves.
  Wheels and mounts (items/vehicle/wheel.json, 17) -- face-on wheels
    and rims: spoked bicycle and motorbike rims, dished steel car
    rims (medium / wide), solid metal wheel, flanged rail wheels
    (single in profile, small pair on an axle), roller drum as a wide
    steel cylinder, caster sets, skateboard wheels, and the hub
    mounts as bolted plates (wooden one brown, trucks as T-shapes).
  Wheat foods (items/comestibles/wheat.json, 16) -- served and dry
    foods told apart by form and colour: pancake stacks (blood =
    near-black, fruit = golden with berries and syrup), grits bowls
    (plain pale, cheese orange), mac & cheese vs dry elbow macaroni,
    ramen brick, lasagne sheet, French toast slices, granola and oat
    heaps, boiled grain kernels, cloutie dumpling studded with fruit,
    'special' brownie with green flecks, snack bread loaf.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "compressor",
    "drivebelt_makeshift",
    "filter_air_makeshift",
    "filter_liquid_makeshift",
    "five-point_harness",
    "hd_compressor",
    "hd_pipe",
    "hd_pipe_fittings",
    "hd_valve",
    "large_pressure_vessel",
    "nitrogen_membrane_filter",
    "pressure_gauge",
    "pump_complex",
    "pyrometer",
    "simple_crank",
    "small_pressure_vessel",
    "tire_repair_kit",
    "well_pump",
    "hd_steel_drum",
    "wheel_caster",
    "wheel_caster_large",
    "wheel_metal",
    "wheel_mount_concrete_mix",
    "wheel_mount_heavy",
    "wheel_mount_light",
    "wheel_mount_medium",
    "wheel_mount_skateboard",
    "wheel_mount_wood",
    "wheel_rail",
    "wheel_rail_small_pair",
    "wheel_rim_bicycle",
    "wheel_rim_medium",
    "wheel_rim_motorbike",
    "wheel_rim_wide",
    "wheel_skateboard",
    "blood_pancakes",
    "brownie_weed",
    "cheese_grits",
    "cloutie_dumpling",
    "frenchtoast",
    "fruit_pancakes",
    "grain_boiled",
    "granola",
    "grits",
    "lasagne_raw",
    "macaroni_cooked",
    "macaroni_raw",
    "noodles_fast",
    "oat_pulp",
    "oatmeal",
    "snack_bread",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

STEEL = (148, 154, 162, 255)
STEEL_L = (186, 192, 200, 255)
STEEL_D = (104, 110, 120, 255)
STEEL_XD = (72, 78, 88, 255)
IRON = (88, 90, 96, 255)
IRON_D = (58, 60, 66, 255)
ALU = (196, 200, 206, 255)
RUST = (150, 92, 56, 255)
RED = (192, 44, 36, 255)
BRASS = (178, 142, 62, 255)
RUBBER = (44, 44, 48, 255)
WOOD = (128, 92, 54, 255)
WOOD_L = (164, 124, 78, 255)
PAPER = (226, 220, 202, 255)
PLASTIC = (90, 110, 150, 255)
NYLON = (52, 62, 96, 255)
LEATHER = (140, 100, 62, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def rivets(d, pts, col=STEEL_XD):
    for x, y in pts:
        d.point((x, y), fill=col)


# ------------------------------------------------------------- industrial

def pipe_body(d, y0, y1, x0=3, x1=28, base=STEEL):
    """Horizontal thick-walled pipe; open right end shows the wall."""
    d.rectangle([x0, y0, x1, y1], fill=base)
    d.line([x0, y0, x1, y0], fill=STEEL_L, width=1)
    d.line([x0, y1, x1, y1], fill=STEEL_D, width=1)
    d.ellipse([x1 - 3, y0, x1 + 1, y1], fill=STEEL_D)
    d.ellipse([x1 - 1, y0 + 3, x1 + 1, y1 - 3], fill=STEEL_XD)


def draw_compressor(d, rng, big=False):
    ybase = 26 if big else 24
    tw = 26 if big else 20  # tank width
    tx0 = 16 - tw // 2
    # horizontal receiver tank
    d.ellipse([tx0, ybase - 6, tx0 + 6, ybase], fill=STEEL_D)
    d.rectangle([tx0 + 3, ybase - 6, tx0 + tw - 3, ybase], fill=STEEL)
    d.ellipse([tx0 + tw - 6, ybase - 6, tx0 + tw, ybase], fill=STEEL_L)
    d.line([tx0 + 2, ybase - 6, tx0 + tw - 2, ybase - 6], fill=STEEL_L)
    # feet
    d.rectangle([tx0 + 3, ybase, tx0 + 5, ybase + 2], fill=STEEL_XD)
    d.rectangle([tx0 + tw - 6, ybase, tx0 + tw - 4, ybase + 2], fill=STEEL_XD)
    # motor block on the tank
    mw = 13 if big else 9
    mx0, my0 = 16 - mw // 2, ybase - 6 - (11 if big else 8)
    d.rectangle([mx0, my0, mx0 + mw, ybase - 6], fill=IRON)
    for fx in range(mx0 + 1, mx0 + mw, 2):  # cooling fins
        d.line([fx, my0, fx, ybase - 6], fill=IRON_D, width=1)
    d.line([mx0, my0, mx0 + mw, my0], fill=STEEL_D, width=1)
    # cylinder head + outlet pipe
    d.rectangle([mx0 + mw - 3, my0 - 3, mx0 + mw, my0], fill=STEEL_D)
    d.line([mx0 + mw - 1, my0 - 3, mx0 + mw - 1, my0 - 6], fill=STEEL,
           width=1)
    d.line([mx0 + mw - 1, my0 - 6, mx0 + mw + 3, my0 - 6], fill=STEEL,
           width=1)
    if big:  # pressure gauge on the tank + second cylinder
        d.ellipse([tx0 + 2, ybase - 11, tx0 + 6, ybase - 7], fill=PAPER)
        d.ellipse([tx0 + 2, ybase - 11, tx0 + 6, ybase - 7],
                  outline=STEEL_XD)
        d.point((tx0 + 4, ybase - 9), fill=RED)
        d.rectangle([mx0 - 4, my0 + 2, mx0 - 1, ybase - 6], fill=IRON)
    else:  # fan grill on the motor face
        d.ellipse([mx0 + 2, my0 + 2, mx0 + mw - 2, ybase - 8], outline=STEEL_D)
        d.point((16, (my0 + ybase - 6) // 2), fill=STEEL_D)


def draw_drivebelt(d, rng):
    # loop of stitched leather/cotton belting, seen face-on
    d.rounded_rectangle([6, 7, 25, 24], radius=7, outline=LEATHER, width=4)
    d.rounded_rectangle([6, 7, 25, 24], radius=7, outline=shade(LEATHER, 1.2),
                        width=1)
    for x in range(9, 24, 3):  # stitch dashes
        d.point((x, 8), fill=PAPER)
        d.point((x, 23), fill=PAPER)
    d.rectangle([13, 22, 18, 25], fill=STEEL_D)  # laced joint


def draw_filter_air(d, rng):
    # pleated paper panel in a plastic frame
    d.rectangle([5, 8, 26, 23], fill=PLASTIC)
    d.rectangle([7, 10, 24, 21], fill=PAPER)
    for x in range(8, 24, 2):
        d.line([x, 10, x, 21], fill=shade(PAPER, 0.82), width=1)
    d.line([5, 8, 26, 8], fill=shade(PLASTIC, 1.25), width=1)


def draw_filter_liquid(d, rng):
    # spin-on canister: steel cylinder, domed top, threaded stud
    d.rectangle([11, 10, 20, 25], fill=STEEL)
    d.ellipse([11, 7, 20, 12], fill=STEEL_L)
    d.rectangle([14, 4, 17, 7], fill=STEEL_D)
    d.line([13, 5, 13, 6], fill=STEEL_XD, width=1)
    d.line([18, 5, 18, 6], fill=STEEL_XD, width=1)
    d.line([11, 14, 11, 23], fill=STEEL_L, width=1)
    d.line([20, 14, 20, 23], fill=STEEL_D, width=1)
    d.rectangle([11, 19, 20, 22], fill=PLASTIC)  # label band
    d.point((13, 20), fill=PAPER)
    d.point((17, 20), fill=PAPER)


def draw_harness(d, rng):
    cx, cy = 16, 15
    # five straps: two shoulder, two lap, one crotch strap
    for x0, y0 in ((10, 3), (21, 3), (3, 16), (28, 16), (16, 28)):
        d.line([cx, cy, x0, y0], fill=NYLON, width=3)
        d.line([cx, cy, x0, y0], fill=shade(NYLON, 1.3), width=1)
    for x, y in ((9, 3), (20, 3), (3, 15), (27, 15), (15, 27)):
        d.rectangle([x, y, x + 2, y + 2], fill=STEEL_D)  # adjusters
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=STEEL)
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], outline=STEEL_XD)
    d.point((cx, cy), fill=RED)


def draw_hd_pipe(d, rng):
    pipe_body(d, 12, 19)
    d.line([8, 12, 8, 19], fill=STEEL_D, width=1)  # weld seams
    d.line([20, 12, 20, 19], fill=STEEL_D, width=1)


def draw_hd_fittings(d, rng):
    # elbow (top left)
    d.rectangle([4, 6, 11, 12], fill=STEEL)
    d.rectangle([4, 6, 12, 13], fill=STEEL)
    d.rectangle([4, 6, 12, 13], outline=STEEL_D)
    d.ellipse([9, 9, 13, 13], fill=STEEL_XD)
    # tee junction (right)
    d.rectangle([19, 5, 24, 16], fill=STEEL)
    d.rectangle([16, 9, 27, 14], fill=STEEL)
    d.rectangle([16, 9, 27, 14], outline=STEEL_D)
    d.ellipse([20, 4, 23, 7], fill=STEEL_XD)
    # end caps (bottom)
    for x0 in (5, 13, 21):
        d.rectangle([x0, 21, x0 + 5, 26], fill=STEEL_D)
        d.line([x0, 21, x0 + 5, 21], fill=STEEL_L, width=1)
        d.line([x0 + 2, 22, x0 + 2, 25], fill=STEEL_XD, width=1)
    d.line([19, 5, 24, 5], fill=STEEL_L, width=1)


def draw_hd_valve(d, rng):
    pipe_body(d, 15, 22)
    d.rectangle([14, 10, 17, 15], fill=STEEL_D)  # stem housing
    d.ellipse([10, 4, 21, 9], fill=RED)          # handwheel
    d.ellipse([10, 4, 21, 9], outline=shade(RED, 0.7))
    for dx in (-4, 0, 4):
        d.line([15 + dx, 5, 15, 10], fill=shade(RED, 0.75), width=1)
    d.point((15, 6), fill=STEEL_XD)


def draw_pressure_gauge(d, rng):
    pipe_body(d, 16, 23)
    d.rectangle([14, 11, 17, 16], fill=STEEL_D)
    d.ellipse([10, 3, 21, 14], fill=STEEL_XD)    # dial bezel
    d.ellipse([11, 4, 20, 13], fill=PAPER)
    for a in range(0, 360, 45):                  # tick marks
        x = 15.5 + 3.6 * math.cos(math.radians(a))
        y = 8.5 + 3.6 * math.sin(math.radians(a))
        d.point((int(x), int(y)), fill=STEEL_XD)
    d.line([15, 8, 18, 6], fill=RED, width=1)    # needle in the red zone
    d.point((15, 8), fill=STEEL_XD)


def draw_nitrogen_filter(d, rng):
    pipe_body(d, 13, 20)
    # wider membrane section mid-pipe, banded
    d.rectangle([12, 11, 20, 22], fill=STEEL_D)
    d.line([12, 11, 20, 11], fill=STEEL_L, width=1)
    for x in (14, 17):
        d.line([x, 12, x, 21], fill=STEEL_XD, width=1)
    for y in range(13, 21, 2):  # membrane slits
        d.point((15, y), fill=ALU)
        d.point((18, y), fill=ALU)


def draw_large_vessel(d, rng):
    # huge horizontal capsule tank on saddle supports
    d.rectangle([4, 10, 27, 21], fill=STEEL)
    d.ellipse([2, 10, 8, 21], fill=STEEL_D)
    d.ellipse([23, 10, 29, 21], fill=STEEL_L)
    d.line([4, 10, 27, 10], fill=STEEL_L, width=1)
    d.line([4, 21, 27, 21], fill=STEEL_D, width=1)
    for x in (10, 20):  # shell seams
        d.line([x, 10, x, 21], fill=STEEL_D, width=1)
    d.polygon([(7, 21), (11, 21), (10, 26), (8, 26)], fill=STEEL_XD)
    d.polygon([(20, 21), (24, 21), (23, 26), (21, 26)], fill=STEEL_XD)
    d.rectangle([13, 6, 16, 10], fill=STEEL_D)   # top nozzle
    d.ellipse([22, 5, 26, 9], fill=PAPER)       # gauge
    d.ellipse([22, 5, 26, 9], outline=STEEL_XD)
    d.point((24, 7), fill=RED)
    d.line([24, 9, 24, 10], fill=STEEL_D, width=1)


def draw_small_vessel(d, rng):
    # small vertical capsule tank with neck valve
    d.rectangle([11, 9, 20, 24], fill=STEEL)
    d.ellipse([11, 6, 20, 12], fill=STEEL_L)
    d.ellipse([11, 21, 20, 27], fill=STEEL_D)
    d.line([11, 12, 11, 22], fill=STEEL_L, width=1)
    d.line([20, 12, 20, 22], fill=STEEL_D, width=1)
    d.rectangle([14, 3, 17, 7], fill=STEEL_D)
    d.line([12, 4, 19, 4], fill=RED, width=1)    # valve handle
    d.rectangle([12, 25, 19, 27], fill=STEEL_XD)  # base ring


def draw_pump_complex(d, rng):
    # blocky high-pressure pump: base, volute, finned drive, flanged pipes
    d.rectangle([4, 24, 27, 27], fill=IRON_D)    # skid base
    d.ellipse([6, 10, 18, 22], fill=IRON)        # volute casing
    d.ellipse([6, 10, 18, 22], outline=IRON_D)
    d.ellipse([10, 14, 14, 18], fill=STEEL_D)
    d.point((12, 16), fill=STEEL_XD)
    d.rectangle([2, 13, 6, 19], fill=STEEL_D)     # inlet flange
    d.rectangle([18, 8, 22, 12], fill=STEEL_D)    # outlet flange up
    rivets(d, [(3, 14), (3, 18), (19, 9), (21, 9)], STEEL_L)
    d.rectangle([19, 14, 27, 22], fill=IRON)     # drive block
    for fx in range(20, 27, 2):
        d.line([fx, 14, fx, 22], fill=IRON_D, width=1)
    d.line([18, 22, 27, 22], fill=STEEL_XD, width=1)


def draw_well_pump(d, rng):
    # round cast-iron volute with side inlet and top outlet
    d.ellipse([8, 9, 24, 25], fill=IRON)
    d.ellipse([8, 9, 24, 25], outline=IRON_D)
    d.ellipse([12, 13, 20, 21], fill=STEEL_D)
    d.ellipse([14, 15, 18, 19], fill=STEEL_XD)
    for a in range(0, 360, 60):  # casing bolts
        x = 16 + 6.5 * math.cos(math.radians(a))
        y = 17 + 6.5 * math.sin(math.radians(a))
        d.point((int(x), int(y)), fill=STEEL_L)
    d.rectangle([3, 14, 8, 19], fill=STEEL_D)    # inlet pipe
    d.rectangle([14, 4, 18, 9], fill=STEEL_D)    # outlet pipe
    d.line([3, 14, 8, 14], fill=STEEL_L, width=1)


def draw_pyrometer(d, rng):
    # handheld IR thermometer: body, screen, probe tip
    d.polygon([(9, 20), (20, 6), (23, 8), (13, 23)], fill=STEEL_XD)
    d.polygon([(20, 6), (26, 3), (27, 5), (23, 8)], fill=STEEL_D)  # snout
    d.point((26, 4), fill=RED)                   # laser dot sight
    d.polygon([(13, 13), (18, 8), (20, 10), (15, 15)],
              fill=(255, 150, 60, 255))          # hot reading display
    d.rectangle([11, 19, 14, 22], fill=RED)      # trigger button
    d.line([10, 22, 8, 26], fill=STEEL_D, width=2)  # grip


def draw_simple_crank(d, rng):
    # shaft with a Z-bent arm and a free handle
    d.rectangle([3, 8, 12, 11], fill=STEEL_D)    # shaft boss
    d.line([3, 9, 12, 9], fill=STEEL_L, width=1)
    d.line([11, 10, 11, 20], fill=STEEL, width=2)  # arm down
    d.line([11, 20, 21, 20], fill=STEEL, width=2)  # offset out
    d.rectangle([20, 17, 23, 23], fill=WOOD)      # handle grip
    d.line([20, 17, 23, 17], fill=WOOD_L, width=1)
    d.ellipse([9, 7, 13, 12], outline=STEEL_XD)
    d.point((11, 9), fill=STEEL_XD)


def draw_tire_kit(d, rng):
    # open plastic case with plug cords and a glue tube
    d.rectangle([5, 14, 26, 25], fill=PLASTIC)
    d.rectangle([5, 11, 26, 14], fill=shade(PLASTIC, 1.2))
    d.rectangle([7, 16, 24, 23], fill=shade(PLASTIC, 0.7))
    for i, x in enumerate(range(8, 15)):          # brown plug cords
        d.line([x, 17, x, 22], fill=LEATHER, width=1)
    d.rectangle([17, 17, 23, 20], fill=PAPER)     # glue tube
    d.rectangle([22, 17, 24, 18], fill=RED)
    d.line([18, 18, 21, 18], fill=STEEL_XD, width=1)
    d.rectangle([15, 12, 16, 13], fill=STEEL_D)   # latch


# ------------------------------------------------------------------ wheels

def ring(d, cx, cy, r, col, w=2):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=col, width=w)


def draw_steel_drum(d, rng):
    # road-roller drum: very wide steel cylinder, end plate visible
    d.rectangle([6, 7, 27, 24], fill=STEEL)
    d.ellipse([3, 7, 9, 24], fill=STEEL_D)
    d.ellipse([24, 7, 29, 24], fill=STEEL_L)
    d.ellipse([4, 11, 8, 20], fill=STEEL_XD)     # hub recess
    for x in (13, 19):  # plate seams
        d.line([x, 7, x, 24], fill=STEEL_D, width=1)
    d.line([6, 8, 27, 8], fill=STEEL_L, width=1)
    d.line([6, 23, 27, 23], fill=STEEL_D, width=1)
    rivets(d, [(13, 10), (13, 21), (19, 10), (19, 21)])


def caster(d, x0, y0, s, big=False):
    """One caster: swivel plate, fork, small wheel. s = scale unit."""
    d.rectangle([x0, y0, x0 + 5 * s, y0 + s], fill=STEEL_D)      # plate
    d.line([x0 + s, y0 + s, x0 + s, y0 + 3 * s], fill=STEEL, width=1)
    d.line([x0 + 4 * s, y0 + s, x0 + 4 * s, y0 + 3 * s], fill=STEEL,
           width=1)
    r = 2 * s if big else s + 1
    cx, cy = x0 + (5 * s) // 2, y0 + 3 * s + r - 1
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=RUBBER)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=STEEL_L)


def draw_caster(d, rng, big=False):
    if big:
        caster(d, 4, 6, 2, big=True)
        caster(d, 18, 6, 2, big=True)
    else:
        caster(d, 5, 8, 1)
        caster(d, 19, 8, 1)


def draw_wheel_metal(d, rng):
    ring(d, 16, 16, 11, STEEL_D, 3)
    d.ellipse([6, 6, 26, 26], fill=STEEL)
    d.ellipse([6, 6, 26, 26], outline=STEEL_D, width=2)
    for a in range(0, 360, 45):  # lightening holes
        x = 16 + 6.5 * math.cos(math.radians(a))
        y = 16 + 6.5 * math.sin(math.radians(a))
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=STEEL_XD)
    d.ellipse([13, 13, 19, 19], fill=STEEL_D)
    d.point((16, 16), fill=STEEL_XD)


def draw_mount_pair_small(d, rng):
    # concrete-mixer mounts: two small brackets with stub axles
    for x0 in (6, 19):
        d.rectangle([x0, 12, x0 + 7, 20], fill=STEEL_D)
        d.rectangle([x0 + 2, 8, x0 + 4, 12], fill=STEEL)
        d.ellipse([x0 + 1, 14, x0 + 3, 16], fill=STEEL_XD)
        d.ellipse([x0 + 4, 14, x0 + 6, 16], fill=STEEL_XD)
        d.line([x0, 12, x0 + 7, 12], fill=STEEL_L, width=1)


def draw_mount_hub(d, rng, heavy=False, light=False):
    r = 11 if heavy else (7 if light else 9)
    cx = cy = 16
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=STEEL_D)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=STEEL_XD)
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=STEEL_XD)  # bore
    n = 6 if heavy else (4 if light else 5)
    rr = r - 3
    for i in range(n):  # lug holes
        a = math.radians(i * 360 / n + 15)
        x, y = cx + rr * math.cos(a), cy + rr * math.sin(a)
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=STEEL_XD)
    d.arc([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1], 180, 300,
          fill=STEEL_L, width=1)


def draw_mount_skateboard(d, rng):
    # pair of skateboard trucks: baseplate + T hanger + kingpin
    for y0 in (7, 19):
        d.rectangle([8, y0, 23, y0 + 2], fill=STEEL_D)   # baseplate
        d.rectangle([6, y0 + 3, 25, y0 + 5], fill=STEEL)  # hanger bar
        d.rectangle([14, y0 + 2, 17, y0 + 6], fill=STEEL_XD)
        d.point((15, y0 + 4), fill=RED)                   # kingpin bush
        rivets(d, [(9, y0), (22, y0)], STEEL_L)


def draw_mount_wood(d, rng):
    d.rectangle([6, 10, 25, 21], fill=WOOD)
    d.line([6, 13, 25, 13], fill=shade(WOOD, 0.8), width=1)
    d.line([6, 18, 25, 18], fill=shade(WOOD, 0.8), width=1)
    d.line([6, 10, 25, 10], fill=WOOD_L, width=1)
    for x in (10, 15, 20):  # axle/bolt holes
        d.ellipse([x - 1, 14, x + 1, 17], fill=shade(WOOD, 0.45))
    d.point((8, 20), fill=shade(WOOD, 0.6))
    d.point((23, 11), fill=shade(WOOD, 0.6))


def draw_wheel_rail(d, rng):
    # rail wheel in profile: tread disc with a flange lip behind
    d.ellipse([9, 5, 29, 25], fill=STEEL_D)      # flange (larger, offset)
    d.ellipse([5, 7, 25, 27], fill=STEEL)
    d.ellipse([5, 7, 25, 27], outline=STEEL_D, width=2)
    d.ellipse([9, 11, 21, 23], outline=STEEL_D)
    d.ellipse([13, 14, 18, 20], fill=STEEL_D)
    d.point((15, 17), fill=STEEL_XD)


def draw_rail_pair(d, rng):
    # two small flanged wheels on a shared axle
    d.line([4, 16, 28, 16], fill=STEEL_D, width=2)
    for cx in (10, 22):
        d.ellipse([cx - 6, 10, cx + 6, 22], fill=STEEL)
        d.ellipse([cx - 6, 10, cx + 6, 22], outline=STEEL_D, width=1)
        d.ellipse([cx - 7, 11, cx - 4, 21], fill=STEEL_D)  # flange
        d.ellipse([cx - 2, 14, cx + 2, 18], fill=STEEL_D)
    d.rectangle([15, 14, 17, 18], fill=STEEL_XD)


def spoked_rim(d, cx, cy, r, n, col=ALU, hub_r=2):
    ring(d, cx, cy, r, col, 2)
    ring(d, cx, cy, r - 2, shade(col, 0.8), 1)
    for i in range(n):
        a = math.radians(i * 360 / n)
        x = cx + (r - 2) * math.cos(a)
        y = cy + (r - 2) * math.sin(a)
        d.line([cx, cy, x, y], fill=col, width=1)
    d.ellipse([cx - hub_r, cy - hub_r, cx + hub_r, cy + hub_r],
              fill=shade(col, 0.75))


def draw_rim_bicycle(d, rng):
    spoked_rim(d, 16, 16, 12, 12)


def draw_rim_motorbike(d, rng):
    spoked_rim(d, 16, 16, 10, 8, col=STEEL_L, hub_r=3)
    ring(d, 16, 16, 12, STEEL_D, 1)  # brake disc hint


def dished_rim(d, wide=False):
    r_out, r_in = (13, 8) if wide else (12, 7)
    ring(d, 16, 16, r_out, STEEL_D, 3 if wide else 2)
    d.ellipse([16 - r_in, 16 - r_in, 16 + r_in, 16 + r_in], fill=STEEL)
    d.ellipse([16 - r_in, 16 - r_in, 16 + r_in, 16 + r_in],
              outline=STEEL_D)
    for a in range(0, 360, 72):  # vents
        x = 16 + (r_in - 2) * math.cos(math.radians(a))
        y = 16 + (r_in - 2) * math.sin(math.radians(a))
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=STEEL_XD)
    d.ellipse([14, 14, 18, 18], fill=STEEL_D)
    d.point((16, 16), fill=STEEL_XD)


def draw_wheel_skateboard(d, rng):
    # set of four small wheels, two by two
    for cx, cy in ((10, 10), (21, 10), (10, 21), (21, 21)):
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=PAPER)
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], outline=STEEL_D)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=PLASTIC)


# ------------------------------------------------------------------- foods

BOWL = (120, 130, 150, 255)
BOWL_D = (84, 94, 112, 255)


def bowl(d, cx, top, w, contents, speck=None):
    """Served bowl: contents ellipse over a bowl body."""
    x0, x1 = cx - w // 2, cx + w // 2
    d.ellipse([x0, top, x1, top + 6], fill=contents)
    d.polygon([(x0, top + 3), (x1, top + 3), (cx + w // 4, top + 12),
               (cx - w // 4, top + 12)], fill=BOWL)
    d.line([x0, top + 3, x1, top + 3], fill=BOWL_D, width=1)
    d.rectangle([cx - w // 4, top + 12, cx + w // 4, top + 13], fill=BOWL_D)
    if speck:
        rng_pts = [(x0 + 3 + (i * 5) % (w - 5), top + 1 + (i * 2) % 4)
                   for i in range(7)]
        for x, y in rng_pts:
            d.point((x, y), fill=speck)


def pancake_stack(d, cols, fruit=False, syrup=True):
    y = 21
    for i, c in enumerate(cols):
        w = 20 - i * 2
        d.ellipse([16 - w // 2, y - 4, 16 + w // 2, y + 2], fill=c)
        d.ellipse([16 - w // 2, y - 4, 16 + w // 2, y + 2],
                  outline=shade(c, 0.75))
        y -= 4
    if syrup:
        d.line([13, y + 1, 13, y + 5], fill=(150, 92, 30, 255), width=1)
        d.line([19, y + 1, 19, y + 4], fill=(150, 92, 30, 255), width=1)
    if fruit:
        for x, y2, c in ((11, y, RED), (20, y - 1, (70, 60, 140, 255)),
                         (16, y - 2, RED)):
            d.ellipse([x - 1, y2 - 1, x + 1, y2 + 1], fill=c)


def draw_blood_pancakes(d, rng):
    dark = (52, 40, 38, 255)
    pancake_stack(d, [shade(dark, 1.15), dark, shade(dark, 0.85)],
                  syrup=False)
    d.rectangle([13, 7, 18, 9], fill=(120, 40, 36, 255))  # lingonberry jam


def draw_fruit_pancakes(d, rng):
    gold = (206, 158, 92, 255)
    pancake_stack(d, [shade(gold, 1.1), gold, shade(gold, 0.88)], fruit=True)


def draw_brownie(d, rng):
    d.rectangle([8, 10, 23, 22], fill=(74, 48, 30, 255))
    d.line([8, 10, 23, 10], fill=(104, 72, 44, 255), width=2)
    d.line([10, 14, 14, 14], fill=(52, 32, 18, 255), width=1)  # cracks
    d.line([16, 18, 21, 18], fill=(52, 32, 18, 255), width=1)
    for x, y in ((11, 12), (19, 15), (14, 20), (21, 20), (12, 17)):
        d.point((x, y), fill=(86, 148, 62, 255))  # the 'special' flecks
    d.rectangle([8, 22, 23, 23], fill=(52, 32, 18, 255))


def draw_cheese_grits(d, rng):
    bowl(d, 16, 10, 20, (238, 176, 66, 255), speck=(250, 214, 120, 255))
    for x in (11, 16, 21):  # cheese shreds melting over the rim
        d.line([x, 13, x, 16], fill=(250, 196, 84, 255), width=1)


def draw_grits(d, rng):
    bowl(d, 16, 10, 20, (238, 216, 150, 255), speck=(250, 236, 190, 255))
    d.rectangle([14, 10, 18, 12], fill=(250, 226, 120, 255))  # butter pat


def draw_mac_cooked(d, rng):
    bowl(d, 16, 10, 20, (240, 150, 44, 255))
    for x, y in ((10, 11), (14, 13), (18, 11), (21, 13), (12, 12),
                 (20, 12), (16, 12)):  # elbow shapes in the sauce
        d.arc([x, y, x + 3, y + 3], 300, 160, fill=(250, 196, 96, 255),
              width=1)


def draw_mac_raw(d, rng):
    # heap of dry elbow macaroni on a small pile shadow
    d.ellipse([6, 21, 26, 25], fill=(210, 186, 130, 255))
    for i in range(14):
        x = 8 + rng.randint(0, 15)
        y = 12 + rng.randint(0, 8)
        d.arc([x, y, x + 4, y + 4], rng.randint(0, 180),
              rng.randint(200, 340), fill=(232, 200, 118, 255), width=1)


def draw_grain_boiled(d, rng):
    bowl(d, 16, 10, 20, (146, 108, 62, 255))
    for i in range(16):  # whole kernels
        x = 8 + rng.randint(0, 15)
        y = 11 + rng.randint(0, 3)
        d.point((x, y), fill=(178, 140, 88, 255) if i % 2 else WOOD)


def draw_oatmeal(d, rng):
    # dry oat flakes: pale flaky heap
    d.ellipse([6, 20, 26, 25], fill=(206, 198, 178, 255))
    for i in range(18):
        x = 8 + rng.randint(0, 15)
        y = 12 + rng.randint(0, 7)
        d.line([x, y, x + 2, y], fill=(226, 220, 200, 255) if i % 2
               else (188, 178, 156, 255), width=1)


def draw_oat_pulp(d, rng):
    # gritty brown pulp mound
    d.polygon([(6, 23), (10, 13), (16, 10), (22, 13), (26, 23)],
              fill=(128, 100, 66, 255))
    for i in range(14):
        x = 9 + rng.randint(0, 13)
        y = 14 + rng.randint(0, 7)
        d.point((x, y), fill=(104, 80, 50, 255) if i % 2
                else (156, 128, 92, 255))


def draw_granola(d, rng):
    # crunchy baked oat clusters with fruit bits
    d.ellipse([6, 20, 26, 25], fill=(168, 118, 58, 255))
    for i in range(16):
        x = 8 + rng.randint(0, 15)
        y = 12 + rng.randint(0, 7)
        c = (196, 148, 76, 255) if i % 3 else (168, 52, 44, 255)
        d.rectangle([x, y, x + 2, y + 1], fill=c)


def draw_noodles_fast(d, rng):
    # instant ramen brick: wavy noodle block
    d.rectangle([7, 9, 24, 22], fill=(238, 200, 110, 255))
    for y in range(10, 22, 2):
        pts = [(x, y + (1 if (x // 2) % 2 else 0)) for x in range(8, 24)]
        d.line(pts, fill=(214, 170, 78, 255), width=1)
    d.rectangle([7, 9, 24, 22], outline=(196, 148, 66, 255))


def draw_lasagne_raw(d, rng):
    # flat pasta sheets with wavy edges, slightly stacked
    for i, off in enumerate(((2, 3), (0, 0))):
        x0, y0 = 7 + off[0], 9 + off[1]
        d.rectangle([x0, y0, x0 + 17, y0 + 12], fill=(240, 208, 110, 255))
        d.rectangle([x0, y0, x0 + 17, y0 + 12],
                    outline=(206, 168, 74, 255))
        for x in range(x0 + 1, x0 + 17, 3):  # ruffled edges
            d.point((x, y0), fill=(250, 226, 150, 255))
            d.point((x, y0 + 12), fill=(250, 226, 150, 255))


def draw_frenchtoast(d, rng):
    # two overlapping slices of egg-fried bread
    for x0, y0 in ((6, 9), (13, 14)):
        d.rectangle([x0, y0, x0 + 12, y0 + 9], fill=(216, 156, 74, 255))
        d.rectangle([x0, y0, x0 + 12, y0 + 9],
                    outline=(150, 100, 44, 255))
        d.rectangle([x0 + 2, y0 + 2, x0 + 10, y0 + 7],
                    fill=(232, 182, 100, 255))
        d.point((x0 + 4, y0 + 4), fill=(186, 122, 52, 255))
        d.point((x0 + 8, y0 + 5), fill=(186, 122, 52, 255))


def draw_cloutie(d, rng):
    # round cloth-boiled cake studded with dried fruit, tied top
    d.ellipse([7, 10, 25, 25], fill=(168, 122, 70, 255))
    d.ellipse([7, 10, 25, 25], outline=(128, 90, 48, 255))
    d.arc([9, 8, 23, 20], 200, 330, fill=(140, 102, 56, 255), width=1)
    for x, y in ((11, 15), (17, 13), (21, 17), (13, 20), (19, 21),
                 (15, 17)):  # raisins / dried fruit
        d.point((x, y), fill=(84, 44, 40, 255))
    d.line([13, 10, 16, 7], fill=PAPER, width=1)  # cloth tie
    d.line([19, 10, 16, 7], fill=PAPER, width=1)
    d.point((16, 7), fill=RED)


def draw_snack_bread(d, rng):
    # small wheat loaf with score marks
    d.ellipse([6, 12, 26, 24], fill=(206, 158, 92, 255))
    d.ellipse([6, 12, 26, 24], outline=(160, 114, 56, 255))
    d.rectangle([6, 18, 26, 24], fill=(206, 158, 92, 255))
    d.line([6, 24, 26, 24], fill=(160, 114, 56, 255), width=1)
    for x in (11, 16, 21):  # scores
        d.line([x, 13, x + 2, 15], fill=(240, 206, 140, 255), width=1)


DRAW = {
    "compressor": lambda d, r: draw_compressor(d, r),
    "hd_compressor": lambda d, r: draw_compressor(d, r, big=True),
    "drivebelt_makeshift": draw_drivebelt,
    "filter_air_makeshift": draw_filter_air,
    "filter_liquid_makeshift": draw_filter_liquid,
    "five-point_harness": draw_harness,
    "hd_pipe": draw_hd_pipe,
    "hd_pipe_fittings": draw_hd_fittings,
    "hd_valve": draw_hd_valve,
    "pressure_gauge": draw_pressure_gauge,
    "nitrogen_membrane_filter": draw_nitrogen_filter,
    "large_pressure_vessel": draw_large_vessel,
    "small_pressure_vessel": draw_small_vessel,
    "pump_complex": draw_pump_complex,
    "well_pump": draw_well_pump,
    "pyrometer": draw_pyrometer,
    "simple_crank": draw_simple_crank,
    "tire_repair_kit": draw_tire_kit,
    "hd_steel_drum": draw_steel_drum,
    "wheel_caster": lambda d, r: draw_caster(d, r),
    "wheel_caster_large": lambda d, r: draw_caster(d, r, big=True),
    "wheel_metal": draw_wheel_metal,
    "wheel_mount_concrete_mix": draw_mount_pair_small,
    "wheel_mount_heavy": lambda d, r: draw_mount_hub(d, r, heavy=True),
    "wheel_mount_light": lambda d, r: draw_mount_hub(d, r, light=True),
    "wheel_mount_medium": lambda d, r: draw_mount_hub(d, r),
    "wheel_mount_skateboard": draw_mount_skateboard,
    "wheel_mount_wood": draw_mount_wood,
    "wheel_rail": draw_wheel_rail,
    "wheel_rail_small_pair": draw_rail_pair,
    "wheel_rim_bicycle": draw_rim_bicycle,
    "wheel_rim_medium": lambda d, r: dished_rim(d),
    "wheel_rim_motorbike": draw_rim_motorbike,
    "wheel_rim_wide": lambda d, r: dished_rim(d, wide=True),
    "wheel_skateboard": draw_wheel_skateboard,
    "blood_pancakes": draw_blood_pancakes,
    "brownie_weed": draw_brownie,
    "cheese_grits": draw_cheese_grits,
    "cloutie_dumpling": draw_cloutie,
    "frenchtoast": draw_frenchtoast,
    "fruit_pancakes": draw_fruit_pancakes,
    "grain_boiled": draw_grain_boiled,
    "granola": draw_granola,
    "grits": draw_grits,
    "lasagne_raw": draw_lasagne_raw,
    "macaroni_cooked": draw_mac_cooked,
    "macaroni_raw": draw_mac_raw,
    "noodles_fast": draw_noodles_fast,
    "oat_pulp": draw_oat_pulp,
    "oatmeal": draw_oatmeal,
    "snack_bread": draw_snack_bread,
}

# -------------------------------------------------------------------- main

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
    assert set(DRAW) == set(IDS), set(IDS) ^ set(DRAW)
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
        prev.alpha_composite(big, ((i % cols) * cell + 1,
                                   (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item65.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item65.png {prev.size}")


if __name__ == "__main__":
    main()
