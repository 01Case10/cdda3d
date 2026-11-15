#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch VEH1: 49 vehicle part tiles.

Drawn from each part's in-game name/description/symbol/color
(fork-src/cdda/data/json/vehicleparts/ + items/vehicle/), flat UltiCa
style, native 32x32, bodies fully opaque on transparent background.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_veh1.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "vehicle")
CREDITS = ("Drawn from the vehicle part's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")

OUT = (34, 32, 36, 255)          # outline
STEEL_L = (168, 173, 182, 255)
STEEL = (128, 133, 142, 255)
STEEL_D = (84, 88, 96, 255)
STEEL_XD = (52, 54, 60, 255)
ALU_L = (205, 209, 216, 255)
ALU = (168, 173, 182, 255)
ALU_D = (128, 132, 140, 255)
GREEN_L = (110, 158, 84, 255)
GREEN = (76, 120, 56, 255)
GREEN_D = (50, 84, 38, 255)
YELLOW = (216, 167, 34, 255)
YELLOW_L = (240, 198, 84, 255)
RED = (163, 51, 39, 255)
RED_D = (110, 34, 26, 255)
BROWN = (122, 84, 52, 255)
BROWN_D = (84, 56, 34, 255)
WOOD = (168, 128, 72, 255)
WOOD_D = (122, 90, 48, 255)
CELL = (43, 77, 155, 255)        # solar cell blue
CELL_L = (74, 116, 196, 255)
CELL_DK = (20, 38, 66, 255)      # monocrystalline navy
CELL_DK_L = (38, 66, 104, 255)
PALE = (240, 235, 190, 255)      # lamp glow / beam (pale opaque)
CYAN = (140, 210, 220, 255)
BLACK = (22, 22, 26, 255)


def R(d, box, fill, o=OUT, w=1):
    d.rectangle(box, fill=fill, outline=o, width=w)


def RR(d, box, r, fill, o=OUT, w=1):
    d.rounded_rectangle(box, radius=r, fill=fill, outline=o, width=w)


def E(d, box, fill, o=OUT, w=1):
    d.ellipse(box, fill=fill, outline=o, width=w)


def P(d, pts, fill, o=OUT):
    d.polygon(pts, fill=fill, outline=o)


def L(d, pts, fill, w=1):
    d.line(pts, fill=fill, width=w, joint="curve")


def speckle(d, rng, box, colors, n):
    x0, y0, x1, y1 = box
    if x1 < x0 or y1 < y0:
        return
    for _ in range(n):
        x = rng.randint(x0, x1)
        y = rng.randint(y0, y1)
        d.point((x, y), fill=rng.choice(colors))


# ---------------------------------------------------------------- tanks/lights

def tank_aluminum(d, rng):
    # horizontal cylindrical roof tank, aluminum
    E(d, (2, 11, 9, 22), ALU)
    E(d, (23, 11, 30, 22), ALU_L)
    R(d, (5, 11, 27, 22), ALU_L, o=None)
    L(d, [(5, 11), (27, 11)], OUT)
    L(d, [(5, 22), (27, 22)], OUT)
    for x in (11, 21):  # band straps
        R(d, (x, 11, x + 1, 22), ALU_D, o=None)
    L(d, [(6, 13), (26, 13)], (232, 236, 240, 255))  # highlight
    R(d, (14, 8, 18, 11), ALU_D)                     # filler neck
    R(d, (13, 7, 19, 9), STEEL_D)                    # cap
    R(d, (7, 22, 10, 25), STEEL_D)                   # feet
    R(d, (22, 22, 25, 25), STEEL_D)
    speckle(d, rng, (6, 14, 26, 21), [ALU, ALU_D], 10)


def aisle_lights(d, rng):
    L(d, [(16, 2), (16, 12)], STEEL_D, 1)            # drop wire
    R(d, (2, 12, 30, 18), STEEL_XD)                  # housing bar
    R(d, (2, 12, 30, 13), STEEL_D, o=None)
    for i in range(6):                               # light pods
        x = 4 + i * 4
        R(d, (x, 14, x + 2, 16), PALE, o=None)
        d.point((x + 1, 19), fill=(214, 208, 160, 255))
    R(d, (2, 12, 3, 18), BLACK, o=None)
    R(d, (29, 12, 30, 18), BLACK, o=None)


# ---------------------------------------------------------------- green machines

def ammonia_pipework(d, rng):
    R(d, (3, 25, 29, 27), STEEL_D)                   # base plate
    R(d, (20, 7, 27, 25), GREEN)                     # compressor body
    RR(d, (20, 5, 27, 10), 2, GREEN_L)
    R(d, (22, 12, 25, 15), GREEN_D, o=None)          # motor block
    R(d, (5, 6, 9, 25), GREEN)                       # riser pipe
    R(d, (5, 17, 29, 22), GREEN)                     # main run
    R(d, (5, 17, 29, 18), GREEN_L, o=None)
    for x in (12, 18):                               # flange rings
        R(d, (x, 16, x + 1, 23), GREEN_D, o=None)
    E(d, (11, 8, 17, 14), STEEL)                     # valve wheel
    for a in (0, 60, 120):
        r = math.radians(a)
        L(d, [(14, 11), (14 + 3 * math.cos(r), 11 + 3 * math.sin(r))], STEEL_D)
    L(d, [(14, 14), (14, 17)], STEEL_D, 2)
    speckle(d, rng, (6, 18, 28, 21), [GREEN_L, GREEN_D], 8)


def ammonia_reactor(d, rng):
    R(d, (8, 25, 26, 28), STEEL_D)                   # skirt
    RR(d, (10, 6, 24, 26), 5, GREEN)                 # vessel
    R(d, (10, 12, 24, 14), GREEN_D, o=None)          # band
    R(d, (10, 20, 24, 21), GREEN_D, o=None)
    L(d, [(13, 8), (13, 24)], GREEN_L)               # highlight
    R(d, (15, 3, 19, 7), GREEN_D)                    # top neck
    L(d, [(19, 4), (27, 4), (27, 10)], GREEN, 3)     # outlet pipe
    E(d, (14, 15, 20, 21), ALU_L)                    # gauge
    L(d, [(17, 18), (19, 16)], RED, 1)
    for x in (12, 22):
        d.point((x, 13), fill=STEEL_XD)
        d.point((x, 24), fill=STEEL_XD)
    speckle(d, rng, (11, 7, 23, 25), [GREEN_L, GREEN_D], 8)


def electrolyzer(d, rng):
    RR(d, (5, 14, 27, 28), 2, GREEN)                 # cabinet
    R(d, (5, 14, 27, 16), GREEN_L, o=None)
    for x in (8, 19):                                # gas cylinders
        RR(d, (x, 5, x + 5, 14), 2, (196, 228, 230, 255))
        for _ in range(4):                           # bubbles
            d.point((rng.randint(x + 1, x + 4), rng.randint(8, 12)),
                    fill=(240, 250, 250, 255))
        R(d, (x + 1, 3, x + 4, 5), STEEL_D, o=None)   # cap
    L(d, [(10, 3), (16, 1), (22, 3)], STEEL_D, 1)    # manifold arc
    E(d, (9, 19, 14, 24), ALU_L)                     # gauge
    L(d, [(11, 21), (13, 20)], RED)
    R(d, (21, 19, 25, 22), GREEN_D)                  # switch box
    d.point((23, 20), fill=YELLOW_L)
    L(d, [(27, 24), (30, 27), (30, 29)], BLACK, 1)  # cord


def nitrogen_generator(d, rng):
    RR(d, (6, 12, 26, 28), 2, GREEN)                 # box
    for x in (9, 18):                                # sieve towers
        R(d, (x, 5, x + 5, 12), GREEN_L)
        R(d, (x, 8, x + 5, 9), GREEN_D, o=None)
        R(d, (x + 1, 3, x + 4, 5), STEEL_D, o=None)
    L(d, [(11, 3), (20, 3)], STEEL_D, 1)
    E(d, (10, 16, 15, 21), ALU_L)                    # gauge
    L(d, [(12, 18), (14, 17)], RED)
    R(d, (19, 16, 24, 19), GREEN_D, o=None)          # vent slots
    R(d, (19, 21, 24, 22), GREEN_D, o=None)
    d.point((22, 25), fill=YELLOW_L)
    L(d, [(26, 25), (29, 28)], BLACK, 1)
    speckle(d, rng, (7, 13, 25, 27), [GREEN_L, GREEN_D], 8)


def boiler(d, rng):
    E(d, (6, 7, 26, 28), RED)                        # pot belly
    E(d, (6, 7, 26, 28), None, o=RED_D, w=1)
    R(d, (14, 2, 18, 8), STEEL_XD)                   # chimney
    R(d, (13, 1, 19, 3), BLACK, o=None)
    for i, x in enumerate((10, 15, 20)):              # three gauge "eyes"
        E(d, (x, 9, x + 4, 13), ALU_L)
        d.point((x + 2, 11), fill=RED_D)
    RR(d, (11, 15, 21, 25), 1, STEEL_XD)             # cage door
    L(d, [(16, 15), (16, 25)], STEEL_D, 1)
    L(d, [(11, 20), (21, 20)], STEEL_D, 1)
    E(d, (15, 18, 17, 20), (240, 150, 60, 255), o=None)  # fire glow
    E(d, (26, 15, 30, 19), STEEL)                    # side crank
    L(d, [(28, 17), (30, 14)], STEEL_D, 2)
    speckle(d, rng, (8, 8, 24, 26), [RED_D, (196, 84, 62, 255)], 10)


def crucible(d, rng):
    P(d, [(9, 11), (23, 11), (21, 25), (11, 25)], BROWN)  # pot
    E(d, (9, 8, 23, 13), BROWN_D)
    E(d, (11, 9, 21, 12), (248, 176, 64, 255), o=None)     # molten glass
    E(d, (13, 10, 19, 12), (252, 224, 130, 255), o=None)
    for y in (16, 20):                                     # coil bands
        L(d, [(10, y), (22, y)], BROWN_D, 1)
    R(d, (8, 25, 24, 28), STEEL_D)                         # base
    L(d, [(24, 26), (29, 28), (29, 30)], BLACK, 1)         # cord
    for x, y in ((13, 5), (19, 4), (16, 6)):               # heat sparkles
        d.point((x, y), fill=(250, 210, 110, 255))


# ---------------------------------------------------------------- solar family

def solar_cells(d, box, nx, ny, base, light, frame, fw=2):
    x0, y0, x1, y1 = box
    R(d, box, frame)
    R(d, (x0 + fw, y0 + fw, x1 - fw, y1 - fw), base, o=None)
    cw = (x1 - x0 - 2 * fw) / nx
    ch = (y1 - y0 - 2 * fw) / ny
    for i in range(1, nx):
        x = int(x0 + fw + i * cw)
        L(d, [(x, y0 + fw), (x, y1 - fw)], light, 1)
    for j in range(1, ny):
        y = int(y0 + fw + j * ch)
        L(d, [(x0 + fw, y), (x1 - fw, y)], light, 1)


def folding_panel(d, rng):
    P(d, [(3, 10), (15, 8), (15, 24), (3, 26)], STEEL)
    P(d, [(17, 8), (29, 10), (29, 26), (17, 24)], STEEL)
    for (bx, by) in ((5, 11), (19, 11)):
        for i in range(3):
            for j in range(3):
                R(d, (bx + i * 3, by + j * 4, bx + i * 3 + 2, by + j * 4 + 3),
                  CELL, o=CELL_L)
    R(d, (15, 8, 17, 24), STEEL_D, o=None)            # hinge
    L(d, [(6, 26), (4, 29)], STEEL_D, 2)
    L(d, [(26, 26), (28, 29)], STEEL_D, 2)


def solar_suitcase(d, rng):
    R(d, (3, 9, 29, 26), STEEL_XD)                   # case shell
    R(d, (5, 11, 15, 24), CELL_DK, o=CELL_DK_L)
    R(d, (17, 11, 27, 24), CELL_DK, o=CELL_DK_L)
    for bx in (5, 17):
        for i in range(1, 3):
            L(d, [(bx + i * 3 + 1, 11), (bx + i * 3 + 1, 24)], CELL_DK_L, 1)
        L(d, [(bx, 17), (bx + 10, 17)], CELL_DK_L, 1)
    R(d, (13, 5, 19, 9), STEEL_D)                    # handle
    R(d, (14, 6, 18, 7), BLACK, o=None)
    d.point((16, 24), fill=YELLOW_L)                 # clasp light


def ground_panel(d, rng, v2=False, reinforced=False, small=False):
    if small:
        box = (6, 9, 26, 20)
    else:
        box = (4, 7, 28, 20)
    x0, y0, x1, y1 = box
    L(d, [(x0 + 3, y1), (x0 + 1, 28)], STEEL_D, 2)   # A-frame legs
    L(d, [(x1 - 3, y1), (x1 - 1, 28)], STEEL_D, 2)
    L(d, [(x0 + 2, 24), (x1 - 2, 24)], STEEL_D, 1)
    base = CELL_DK if v2 else CELL
    light = CELL_DK_L if v2 else CELL_L
    frame = STEEL_XD if (v2 or reinforced) else STEEL
    fw = 3 if reinforced else 2
    P(d, [(x0, y0 + 2), (x1, y0), (x1, y1 - 2), (x0, y1)], frame)
    P(d, [(x0 + fw, y0 + 2), (x1 - fw, y0 + 1), (x1 - fw, y1 - 3),
          (x0 + fw, y1 - 1)], base, o=None)
    for i in range(1, 4):
        x = x0 + fw + i * (x1 - x0 - 2 * fw) // 4
        L(d, [(x, y0 + 2), (x, y1 - 2)], light, 1)
    L(d, [(x0 + fw, (y0 + y1) // 2), (x1 - fw, (y0 + y1) // 2)], light, 1)
    if reinforced:
        for cx, cy in ((x0 + 1, y0 + 2), (x1 - 1, y0 + 1), (x0 + 1, y1 - 1),
                       (x1 - 1, y1 - 2)):
            d.point((cx, cy), fill=ALU_L)
        L(d, [(x0, y1), (x1, y0)], STEEL_D, 1)       # cross brace


def flat_panel(d, rng, v2=False, reinforced=False):
    box = (3, 8, 29, 22)
    base = CELL_DK if v2 else CELL
    light = CELL_DK_L if v2 else CELL_L
    frame = STEEL_XD if (v2 or reinforced) else STEEL
    fw = 3 if reinforced else 2
    solar_cells(d, box, 4, 2, base, light, frame, fw)
    if reinforced:
        for cx, cy in ((4, 9), (28, 9), (4, 21), (28, 21)):
            d.point((cx, cy), fill=ALU_L)
    R(d, (6, 22, 26, 24), STEEL_D, o=None)           # mount bar
    L(d, [(9, 24), (8, 28)], STEEL_D, 2)
    L(d, [(23, 24), (24, 28)], STEEL_D, 2)


# ---------------------------------------------------------------- weather stations

def weather_station(d, rng, style="std"):
    if style == "std":
        base_c, acc = YELLOW, STEEL_D
    elif style == "robofac":
        base_c, acc = BROWN, BROWN_D
    else:  # xedra
        base_c, acc = (40, 42, 48, 255), CYAN
    R(d, (10, 22, 22, 29), base_c)                   # base box
    R(d, (10, 22, 22, 24), acc, o=None)
    d.point((13, 26), fill=(90, 220, 90, 255))
    d.point((19, 26), fill=(230, 90, 70, 255))
    R(d, (15, 8, 17, 22), STEEL)                     # telescoping mast
    L(d, [(15, 13), (17, 13)], STEEL_D)
    L(d, [(15, 18), (17, 18)], STEEL_D)
    E(d, (14, 5, 18, 9), STEEL_D)                    # anemometer hub
    for a in (270, 30, 150):                         # cups
        r = math.radians(a)
        cx, cy = 16 + 5 * math.cos(r), 7 + 4 * math.sin(r)
        E(d, (cx - 2, cy - 1, cx + 2, cy + 2), STEEL_L)
    L(d, [(17, 11), (24, 11)], STEEL_D, 1)           # wind vane
    P(d, [(24, 9), (28, 11), (24, 13)], acc, o=None)
    R(d, (11, 14, 15, 18), STEEL_L)                  # sensor box
    if style == "robofac":
        R(d, (18, 14, 22, 18), STEEL_L)              # second pod
        E(d, (12, 17, 20, 22), STEEL_D)              # dome on base
    if style == "xedra":
        L(d, [(16, 22), (16, 26)], CYAN, 1)
        R(d, (12, 15, 14, 17), BLACK, o=CYAN)


# ---------------------------------------------------------------- lights

def floodlight(d, rng, directed=False):
    P(d, [(11, 12), (21, 12), (28, 28), (4, 28)], PALE, o=None)  # beam
    L(d, [(9, 28), (15, 16)], STEEL_D, 2)            # tripod
    L(d, [(23, 28), (17, 16)], STEEL_D, 2)
    L(d, [(16, 28), (16, 17)], STEEL_D, 2)
    R(d, (15, 12, 17, 17), STEEL_D, o=None)          # pole
    if directed:
        P(d, [(6, 4), (26, 4), (23, 12), (9, 12)], STEEL)
        R(d, (9, 6, 23, 10), (250, 244, 200, 255), o=None)
        for x in (12, 16, 20):
            L(d, [(x, 6), (x, 10)], STEEL_D, 1)
    else:
        RR(d, (9, 3, 23, 12), 2, STEEL)
        R(d, (11, 5, 21, 10), (250, 244, 200, 255), o=None)
        L(d, [(16, 5), (16, 10)], STEEL_D, 1)


def wall_light(d, rng):
    P(d, [(10, 14), (22, 14), (27, 27), (5, 27)], PALE, o=None)  # glow cone
    R(d, (13, 3, 19, 29), STEEL_D)                   # wall plate
    for y in (5, 26):
        d.point((16, y), fill=ALU_L)
    P(d, [(8, 8), (24, 8), (21, 13), (11, 13)], STEEL)   # shade
    E(d, (13, 12, 19, 16), (252, 246, 205, 255), o=None)  # bulb


# ---------------------------------------------------------------- wheels & turbines

def water_wheel(d, rng, xl=False):
    cx, cy = 16, 15
    r = 13 if xl else 11
    L(d, [(cx - r + 2, 29), (cx, cy), (cx + r - 2, 29)], WOOD_D, 3)  # frame
    E(d, (cx - r, cy - r, cx + r, cy + r), None, o=WOOD_D, w=2)
    ri = r - 4
    E(d, (cx - ri, cy - ri, cx + ri, cy + ri), None, o=WOOD, w=1)
    n = 10 if xl else 8
    for i in range(n):
        a = 2 * math.pi * i / n
        x2, y2 = cx + r * math.cos(a), cy + r * math.sin(a)
        L(d, [(cx, cy), (x2, y2)], WOOD, 1)
        px, py = cx + (r + 1) * math.cos(a), cy + (r + 1) * math.sin(a)
        R(d, (px - 2, py - 1, px + 2, py + 1), WOOD_D, o=None)  # paddles
    E(d, (cx - 3, cy - 3, cx + 3, cy + 3), STEEL_D)
    d.point((cx, cy), fill=ALU_L)


def wind_turbine(d, rng, xl=False):
    top = 8 if xl else 10
    L(d, [(13, 29), (16, top), (19, 29)], STEEL_D, 2)   # lattice mast
    L(d, [(14, 22), (18, 22)], STEEL_D, 1)
    L(d, [(15, 16), (17, 16)], STEEL_D, 1)
    hub = (16, top)
    bl = 10 if xl else 8
    for a in (270, 30, 150):
        r = math.radians(a)
        tip = (hub[0] + bl * math.cos(r), hub[1] + bl * math.sin(r))
        perp = (hub[0] + 2 * math.cos(r + 1.57), hub[1] + 2 * math.sin(r + 1.57))
        P(d, [hub, tip, perp], (232, 234, 238, 255))
    E(d, (14, top - 2, 18, top + 2), YELLOW)            # nacelle
    L(d, [(18, top), (22, top)], STEEL_D, 1)            # tail boom
    P(d, [(22, top - 2), (25, top), (22, top + 2)], STEEL, o=None)


# ---------------------------------------------------------------- small parts

def stereo(d, rng):
    L(d, [(6, 11), (2, 3)], STEEL_D, 1)               # antenna
    RR(d, (4, 11, 28, 25), 2, STEEL_XD)
    R(d, (4, 11, 28, 13), RED, o=None)                # accent stripe
    for cx in (8, 20):                                # speakers
        E(d, (cx, 15, cx + 5, 22), STEEL_D)
        E(d, (cx + 1, 17, cx + 4, 20), STEEL_L, o=None)
    R(d, (14, 15, 18, 18), CYAN, o=None)              # display
    for x in (14, 16, 18):
        d.point((x, 21), fill=ALU_L)                  # knobs


def workshop(d, rng):
    R(d, (3, 12, 29, 14), WOOD_D)                     # bench top
    R(d, (5, 14, 27, 25), STEEL)
    R(d, (6, 16, 14, 20), STEEL_D, o=None)            # drawers
    R(d, (16, 16, 26, 20), STEEL_D, o=None)
    L(d, [(8, 18), (12, 18)], ALU_L)
    L(d, [(19, 18), (23, 18)], ALU_L)
    R(d, (5, 25, 7, 29), STEEL_D, o=None)             # legs
    R(d, (25, 25, 27, 29), STEEL_D, o=None)
    R(d, (4, 8, 8, 12), STEEL_XD, o=None)             # vice
    L(d, [(12, 10), (17, 7)], STEEL_L, 2)             # wrench on top
    E(d, (22, 8, 26, 12), CYAN, o=None)               # valve/fixture
    L(d, [(26, 12), (28, 17)], CYAN, 1)               # hose


def workbench_recharger(d, rng):
    R(d, (20, 7, 28, 13), RED)                        # charger box
    R(d, (22, 9, 26, 11), CYAN, o=None)
    L(d, [(24, 13), (24, 16)], BLACK, 1)              # cable
    R(d, (4, 14, 28, 21), STEEL)                      # folded bench slab
    R(d, (4, 14, 28, 16), STEEL_L, o=None)
    L(d, [(6, 18), (26, 18)], STEEL_D, 1)
    R(d, (6, 21, 9, 25), STEEL_D, o=None)             # folded legs
    R(d, (23, 21, 26, 25), STEEL_D, o=None)
    d.point((26, 15), fill=YELLOW_L)


def beeper(d, rng):
    RR(d, (9, 11, 23, 23), 2, STEEL_L)
    R(d, (9, 11, 23, 14), STEEL, o=None)
    for i in range(3):
        for j in range(2):
            d.point((12 + i * 4, 17 + j * 3), fill=STEEL_XD)  # grille holes
    d.point((21, 12), fill=(230, 70, 50, 255))        # LED
    L(d, [(12, 23), (11, 28)], BLACK, 1)             # wires
    L(d, [(20, 23), (21, 28)], RED_D, 1)
    R(d, (13, 8, 19, 11), STEEL_D, o=None)            # bracket


def bike_rack(d, rng):
    for x in (8, 23):
        R(d, (x, 7, x + 2, 27), STEEL)                # uprights
    for y in (9, 19):
        R(d, (8, y, 25, y + 2), STEEL, o=None)        # crossbars
    R(d, (5, 25, 28, 28), STEEL_D)                    # wheel tray
    L(d, [(10, 11), (20, 24)], STEEL_XD, 2)          # straps
    L(d, [(23, 11), (14, 24)], STEEL_XD, 2)
    R(d, (15, 16, 18, 19), YELLOW, o=None)            # buckle
    for x in (8, 23):
        d.point((x + 1, 9), fill=STEEL_XD)


def cam_control(d, rng):
    R(d, (5, 5, 27, 24), STEEL_XD)                   # bezel
    R(d, (7, 7, 25, 20), (96, 164, 196, 255), o=None)  # screen
    L(d, [(16, 7), (16, 20)], STEEL_XD, 1)           # quad split
    L(d, [(7, 13), (25, 13)], STEEL_XD, 1)
    L(d, [(8, 17), (14, 17)], (30, 60, 84, 255))     # scene hints
    R(d, (19, 9, 23, 12), (30, 60, 84, 255), o=None)
    L(d, [(9, 10), (13, 10)], (220, 240, 245, 255))
    for x in (11, 16, 21):
        d.point((x, 22), fill=ALU_L)                 # buttons
    R(d, (14, 24, 18, 27), STEEL_D, o=None)          # stand
    R(d, (10, 27, 22, 29), STEEL_D, o=None)


def battery(d, rng, kind):
    sizes = {  # (x0, y0, x1, y1, caps)
        "light_minus": (10, 12, 22, 23, 3),
        "light": (8, 11, 24, 24, 4),
        "medium": (7, 10, 25, 25, 5),
        "heavy": (6, 9, 26, 26, 6),
        "heavy_plus": (5, 8, 27, 27, 6),
    }
    x0, y0, x1, y1, caps = sizes[kind]
    R(d, (x0, y0 + 3, x1, y1), STEEL_XD)              # case
    R(d, (x0, y0 + 3, x0 + 2, y1), BLACK, o=None)     # side shade
    R(d, (x0, y0, x1, y0 + 3), YELLOW)                # top band
    for i in range(caps):                             # cell caps
        cx = x0 + 2 + i * (x1 - x0 - 3) // max(caps - 1, 1)
        d.point((cx, y0 + 1), fill=STEEL_XD)
    R(d, (x0 + 1, y0 - 2, x0 + 4, y0), STEEL_D, o=None)  # terminals
    R(d, (x1 - 4, y0 - 2, x1 - 1, y0), RED_D, o=None)
    R(d, (x0 + 3, y0 + 7, x1 - 3, y0 + 11), YELLOW, o=None)  # label
    L(d, [(x0 + 5, y0 + 9), (x1 - 5, y0 + 9)], STEEL_XD, 1)
    if kind == "heavy_plus":                          # carry handle
        L(d, [(x0 + 4, y0 - 2), (x0 + 4, y0 - 4), (x1 - 4, y0 - 4),
              (x1 - 4, y0 - 2)], STEEL_D, 1)
    speckle(d, rng, (x0 + 1, y0 + 13, x1 - 1, y1 - 1), [BLACK, STEEL_D], 5)


def cargo_bag(d, rng):
    RR(d, (5, 12, 27, 26), 4, (128, 116, 74, 255))   # fabric body
    L(d, [(7, 15), (25, 15)], BROWN_D, 1)            # zipper
    for x in (10, 20):                                # straps
        R(d, (x, 12, x + 2, 26), BROWN_D, o=None)
        R(d, (x, 18, x + 2, 20), STEEL_D, o=None)     # buckle
    P(d, [(11, 12), (13, 8), (19, 8), (21, 12)], BROWN_D, o=None)  # handle
    R(d, (22, 20, 26, 24), (110, 99, 62, 255), o=None)  # pocket
    speckle(d, rng, (6, 13, 26, 25), [(144, 130, 84, 255), (110, 99, 62, 255)], 12)


def cargo_lock(d, rng):
    R(d, (5, 9, 27, 13), STEEL_D)                     # hasp plate
    for x in (8, 19):
        d.arc((x, 10, x + 6, 18), 180, 360, fill=STEEL_L, width=2)  # shackle
        RR(d, (x - 1, 15, x + 7, 25), 2, STEEL_XD)    # lock body
        E(d, (x + 2, 18, x + 4, 21), YELLOW, o=None)  # keyhole plate
        d.point((x + 3, 19), fill=BLACK)
    L(d, [(13, 11), (19, 11)], STEEL_L, 1)            # chain link hint


def cargo_rack(d, rng):
    R(d, (4, 9, 28, 25), STEEL_D)                     # frame
    R(d, (6, 11, 26, 23), (40, 42, 48, 255), o=None)  # mesh bed
    for x in range(9, 26, 4):
        L(d, [(x, 11), (x, 23)], STEEL, 1)
    for y in (15, 19):
        L(d, [(6, y), (26, y)], STEEL, 1)
    R(d, (4, 9, 28, 11), STEEL, o=None)               # rails
    R(d, (4, 23, 28, 25), STEEL, o=None)
    R(d, (8, 25, 11, 28), STEEL_XD, o=None)           # brackets
    R(d, (21, 25, 24, 28), STEEL_XD, o=None)


def cart_handle(d, rng):
    R(d, (6, 24, 10, 28), STEEL_D)                    # mounts
    R(d, (22, 24, 26, 28), STEEL_D)
    R(d, (7, 12, 9, 25), STEEL)                       # posts
    R(d, (23, 12, 25, 25), STEEL)
    RR(d, (7, 8, 25, 13), 2, STEEL)                   # grip bar
    R(d, (12, 9, 20, 12), STEEL_XD, o=None)           # grip wrap
    L(d, [(8, 13), (8, 23)], STEEL_L, 1)


def controls_electronic(d, rng):
    RR(d, (5, 9, 27, 25), 2, STEEL_XD)
    R(d, (5, 9, 27, 11), YELLOW, o=None)              # header stripe
    for i, x in enumerate((8, 12, 16)):               # toggles
        R(d, (x, 14, x + 2, 19), STEEL_D, o=None)
        L(d, [(x + 1, 14), (x + (2 if i % 2 else 0), 12)], ALU_L, 1)
    for cx in (21, 24):                               # knobs
        E(d, (cx - 2, 13, cx + 2, 17), STEEL)
        d.point((cx, 14), fill=STEEL_XD)
    for i, c in enumerate(((90, 220, 90, 255), YELLOW_L, (230, 90, 70, 255))):
        d.point((8 + i * 4, 22), fill=c)              # indicator lights
    L(d, [(16, 22), (25, 22)], STEEL_D, 1)


def controls_turret(d, rng):
    RR(d, (4, 15, 9, 23), 2, STEEL_D)                 # motor pods
    RR(d, (23, 15, 28, 23), 2, STEEL_D)
    RR(d, (9, 11, 23, 26), 2, STEEL_XD)               # main module
    R(d, (9, 11, 23, 13), YELLOW, o=None)
    E(d, (13, 16, 19, 22), BLACK)                     # camera lens
    E(d, (14, 17, 17, 20), (40, 80, 120, 255), o=None)
    d.point((15, 17), fill=(200, 230, 250, 255))
    E(d, (14, 5, 18, 9), STEEL)                       # sensor dome
    L(d, [(16, 5), (16, 2)], STEEL_D, 1)              # antenna
    L(d, [(11, 26), (10, 29)], BLACK, 1)             # cables
    L(d, [(21, 26), (22, 29)], BLACK, 1)
    d.point((21, 15), fill=(230, 90, 70, 255))


def counterweight(d, rng):
    P(d, [(7, 10), (25, 10), (27, 13), (27, 26), (5, 26), (5, 13)], STEEL_D)
    L(d, [(7, 10), (25, 10)], STEEL, 3)               # top face
    R(d, (13, 5, 19, 10), STEEL_D)                    # lifting eye post
    E(d, (14, 3, 18, 8), None, o=STEEL_D, w=2)
    L(d, [(8, 16), (24, 16)], STEEL_XD, 1)            # casting seam
    speckle(d, rng, (7, 12, 25, 25), [STEEL_XD, STEEL], 12)


def crane(d, rng):
    R(d, (4, 21, 12, 28), STEEL_D)                    # pedestal
    R(d, (6, 8, 10, 21), STEEL)                       # mast
    P(d, [(8, 10), (27, 3), (27, 6), (9, 13)], STEEL)  # boom
    L(d, [(6, 10), (2, 8)], STEEL, 3)                 # counter-jib
    R(d, (1, 12, 5, 16), STEEL_XD, o=None)            # counterweight block
    L(d, [(26, 6), (26, 15)], BLACK, 1)              # cable
    P(d, [(24, 15), (28, 15), (26, 19)], YELLOW, o=None)  # hook block
    L(d, [(26, 19), (26, 21)], STEEL_XD, 2)
    for y in (12, 16, 20):
        d.point((8, y), fill=STEEL_XD)


def dashboard(d, rng):
    RR(d, (3, 10, 29, 24), 3, STEEL_XD)
    R(d, (3, 10, 29, 12), STEEL_D, o=None)
    for cx, cy, r in ((10, 16, 4), (21, 16, 4)):      # main gauges
        E(d, (cx - r, cy - r, cx + r, cy + r), CYAN)
        for a in (200, 250, 300, 340):
            rr = math.radians(a)
            d.point((cx + int(3 * math.cos(rr)), cy + int(3 * math.sin(rr))),
                    fill=STEEL_XD)
        a = math.radians(rng.choice((230, 260, 310)))
        L(d, [(cx, cy), (cx + 3 * math.cos(a), cy + 3 * math.sin(a))], RED, 1)
    E(d, (14, 12, 18, 16), CYAN)                      # small gauge
    for i, c in enumerate(((90, 220, 90, 255), YELLOW_L, (230, 90, 70, 255),
                           CYAN)):
        d.point((8 + i * 5, 22), fill=c)
    L(d, [(24, 21), (27, 21)], STEEL_D, 1)


# ---------------------------------------------------------------- engines

def engine(d, rng, kind):
    if kind == "i1":  # single cylinder, air-cooled
        R(d, (10, 15, 23, 25), STEEL)
        R(d, (13, 7, 20, 15), STEEL_D)                # finned tower
        for y in (9, 11, 13):
            L(d, [(12, y), (21, y)], STEEL_XD, 1)
        E(d, (14, 4, 19, 8), STEEL_XD)                # starter cap
        E(d, (5, 15, 11, 22), STEEL_D)                # flywheel
        d.point((8, 18), fill=ALU_L)
        L(d, [(20, 9), (24, 6)], STEEL_XD, 2)         # exhaust
        R(d, (11, 25, 22, 27), STEEL_XD, o=None)
        return
    if kind == "i4":
        x0, x1, y0 = 7, 25, 13
        bolts = 4
    else:  # i6
        x0, x1, y0 = 5, 27, 13
        bolts = 6
    R(d, (x0, y0, x1, 25), STEEL)                     # block
    R(d, (x0, y0, x0 + 2, 25), STEEL_D, o=None)
    R(d, (x0 + 1, y0 - 4, x1 - 1, y0), (120, 168, 178, 255))  # head cover
    for i in range(bolts):
        cx = x0 + 3 + i * (x1 - x0 - 5) // max(bolts - 1, 1)
        d.point((cx, y0 - 2), fill=STEEL_XD)
    L(d, [(x0 + 3, y0 - 7), (x0 + 8, y0 - 9), (x1 - 6, y0 - 9)], STEEL_D, 2)  # intake
    E(d, (x1 - 3, 17, x1 + 2, 23), STEEL_D)           # pulley
    d.point((x1 - 1, 20), fill=ALU_L)
    R(d, (x0 + 2, 25, x1 - 2, 27), STEEL_XD, o=None)  # oil pan
    for i in range(3):
        L(d, [(x0 + 5 + i * 5, 16), (x0 + 5 + i * 5, 22)], STEEL_D, 1)
    speckle(d, rng, (x0 + 3, y0 + 2, x1 - 3, 24), [STEEL_D, STEEL_L], 6)


# ---------------------------------------------------------------- registry

RENDER = {
    "afs_roof_external_tank_aluminum": tank_aluminum,
    "aisle_lights": aisle_lights,
    "ap_ammonia_machine_pipework": ammonia_pipework,
    "ap_ammonia_machine_reactor": ammonia_reactor,
    "ap_electrolyzer_makeshift": electrolyzer,
    "ap_folding_solar_panel": folding_panel,
    "ap_folding_solar_panel_v2": solar_suitcase,
    "ap_glassblowers_crucible": crucible,
    "ap_ground_solar_panel": lambda d, r: ground_panel(d, r),
    "ap_ground_solar_panel_v2": lambda d, r: ground_panel(d, r, v2=True),
    "ap_mobile_weather_station": lambda d, r: weather_station(d, r, "std"),
    "ap_nitrogen_generator": nitrogen_generator,
    "ap_nl_safehouse_boiler": boiler,
    "ap_reinforced_solar_panel": lambda d, r: flat_panel(d, r, reinforced=True),
    "ap_reinforced_solar_panel_v2": lambda d, r: flat_panel(
        d, r, v2=True, reinforced=True),
    "ap_robofac_mobile_weather_station": lambda d, r: weather_station(
        d, r, "robofac"),
    "ap_solar_panel": lambda d, r: ground_panel(d, r, small=True),
    "ap_solar_panel_v2": lambda d, r: ground_panel(d, r, v2=True, small=True),
    "ap_standing_directed_floodlight": lambda d, r: floodlight(d, r, True),
    "ap_standing_floodlight": lambda d, r: floodlight(d, r, False),
    "ap_stereo": stereo,
    "ap_veh_tools_workshop": workshop,
    "ap_wall_light": wall_light,
    "ap_water_wheel": lambda d, r: water_wheel(d, r),
    "ap_wind_turbine": lambda d, r: wind_turbine(d, r),
    "ap_workbench_with_recharger": workbench_recharger,
    "ap_xedra_mobile_weather_station": lambda d, r: weather_station(
        d, r, "xedra"),
    "ap_xl_water_wheel": lambda d, r: water_wheel(d, r, xl=True),
    "ap_xl_wind_turbine": lambda d, r: wind_turbine(d, r, xl=True),
    "beeper": beeper,
    "bike_rack": bike_rack,
    "cam_control": cam_control,
    "car_heavy_battery_cell": lambda d, r: battery(d, r, "heavy"),
    "car_heavy_plus_battery_cell": lambda d, r: battery(d, r, "heavy_plus"),
    "car_light_battery_cell": lambda d, r: battery(d, r, "light"),
    "car_light_minus_battery_cell": lambda d, r: battery(d, r, "light_minus"),
    "car_medium_battery_cell": lambda d, r: battery(d, r, "medium"),
    "cargo_bag": cargo_bag,
    "cargo_lock": cargo_lock,
    "cargo_space_external": cargo_rack,
    "cart_handle": cart_handle,
    "controls_electronic": controls_electronic,
    "controls_turret": controls_turret,
    "counterweight": counterweight,
    "crane_medium_internal": crane,
    "dashboard": dashboard,
    "diesel_engine_i6": lambda d, r: engine(d, r, "i6"),
    "diesel_engine_inline1": lambda d, r: engine(d, r, "i1"),
    "diesel_engine_inline4": lambda d, r: engine(d, r, "i4"),
}

IDS = list(RENDER.keys())


def draw_one(pid):
    rng = random.Random(zlib.crc32(pid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    RENDER[pid](d, rng)
    # opacity rule: alpha must be only {0, 255}
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (pid, alphas)
    return img


def main():
    os.makedirs(BASE, exist_ok=True)
    imgs = {}
    for pid in IDS:
        img = draw_one(pid)
        imgs[pid] = img
        folder = os.path.join(BASE, pid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, pid + ".png"))
        with open(os.path.join(folder, pid + ".json"), "w") as f:
            json.dump([{"id": pid, "fg": pid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # contact sheet: 7x7, 2x scale on neutral backdrop
    cell = 68
    sheet = Image.new("RGBA", (cell * 7, cell * 7), (58, 60, 68, 255))
    sd = ImageDraw.Draw(sheet)
    for i, pid in enumerate(IDS):
        cx, cy = (i % 7) * cell, (i // 7) * cell
        big = imgs[pid].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, (cx + 2, cy + 2))
        sd.rectangle((cx, cy, cx + cell - 1, cy + cell - 1),
                     outline=(30, 30, 34, 255))
    out = os.path.join(os.path.dirname(BASE), "..", "..",
                       "preview_veh1.png")
    out = os.path.normpath(out)
    # preview lives at coverage-audit/new-tiles/preview_veh1.png
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_veh1.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + {out}")


if __name__ == "__main__":
    main()
