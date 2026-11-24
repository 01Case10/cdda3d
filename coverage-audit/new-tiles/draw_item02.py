#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch ITEM02: 52 bionic (CBM) item tiles.

Drawn from each bionic's in-game name/description
(fork-src/cdda/data/json/bionics.json), flat UltiCa style, native 32x32,
bodies fully opaque on transparent background. Matches batch ITEM01's
module language (same palette / canister housing, re-implemented here,
not imported): internal bionics are compact metallic implant modules
with a function pictogram; organ/limb systems are drawn as the body
part itself; faulty bionics are cracked, leaking or misshapen.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_item02.png.
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
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")

OUT = (30, 28, 32, 255)            # outline
STEEL_L = (178, 183, 192, 255)
STEEL = (132, 137, 146, 255)
STEEL_D = (88, 92, 100, 255)
STEEL_XD = (54, 56, 62, 255)
PANEL = (38, 40, 48, 255)          # dark inset panel on a module
BRASS = (186, 148, 74, 255)
BRASS_D = (128, 98, 46, 255)
RED = (196, 52, 44, 255)
RED_D = (138, 32, 30, 255)
GREEN = (84, 178, 84, 255)
ACID = (140, 220, 70, 255)
CYAN = (110, 210, 230, 255)
CYAN_PALE = (196, 238, 246, 255)
BLUE = (84, 130, 220, 255)
YELLOW = (244, 200, 60, 255)
ORANGE = (238, 132, 44, 255)
VIOLET = (168, 110, 230, 255)
PINK = (232, 130, 160, 255)
WHITE = (238, 240, 244, 255)
SKIN = (216, 164, 128, 255)
BONE = (232, 226, 208, 255)


def R(d, box, fill, o=OUT, w=1):
    d.rectangle(box, fill=fill, outline=o, width=w)


def RR(d, box, r, fill, o=OUT, w=1):
    d.rounded_rectangle(box, radius=r, fill=fill, outline=o, width=w)


def E(d, box, fill, o=OUT, w=1):
    d.ellipse(box, fill=fill, outline=o, width=w)


def L(d, pts, fill, w=1):
    d.line(pts, fill=fill, width=w)


def P(d, pts, fill, o=None):
    d.polygon(pts, fill=fill, outline=o)


def canister(d, accent, x0=7, y0=4, x1=25, y1=28):
    """Compact bionic module: steel housing, cap, accent band, dark panel.
    (Same housing as batch ITEM01.)"""
    RR(d, (x0, y0 + 3, x1, y1), 3, STEEL)
    RR(d, (x0 + 2, y0, x1 - 2, y0 + 5), 2, STEEL_D)          # top cap
    R(d, (x0, y0 + 7, x1, y0 + 9), accent)                   # accent band
    RR(d, (x0 + 3, y0 + 11, x1 - 3, y1 - 3), 2, PANEL)       # inset panel
    E(d, (x1 - 5, y0 + 4, x1 - 3, y0 + 6), GREEN, o=None)    # LED
    L(d, [(x0 + 2, y1 - 2), (x0 + 2, y0 + 11)], STEEL_D)
    L(d, [(x1 - 2, y1 - 2), (x1 - 2, y0 + 11)], STEEL_L)


def bolt(d, cx, cy, s, col=YELLOW):
    P(d, [(cx + 1, cy - s), (cx - s + 1, cy + 1), (cx, cy + 1),
          (cx - 1, cy + s), (cx + s - 1, cy - 1), (cx, cy - 1)], col)


def drop(d, cx, cy, s, col):
    P(d, [(cx, cy - s), (cx + s - 1, cy + 2), (cx - s + 1, cy + 2)], col)
    E(d, (cx - s + 1, cy, cx + s - 1, cy + s), col, o=None)


def flame(d, cx, cy, s, col=ORANGE, inner=YELLOW):
    P(d, [(cx, cy - s), (cx + s - 2, cy + 2), (cx + 1, cy + s),
          (cx - 2, cy + s), (cx - s + 1, cy + 1)], col)
    P(d, [(cx, cy - 1), (cx + 2, cy + 2), (cx, cy + s - 1),
          (cx - 2, cy + 2)], inner)


def _cracked(d, rng):
    for _ in range(3):
        x = rng.randint(10, 20)
        L(d, [(x, 12), (x + 2, 16), (x, 19), (x + 1, 23)], STEEL_XD)


def finger(d, x0=12, y0=9, x1=20, y1=29):
    """Skin finger pointing up (same treatment as ITEM01's fingerhack)."""
    RR(d, (x0, y0 + 2, x1, y1), 4, SKIN)
    d.arc((x0, y0 - 3, x1, y0 + 5), start=180, end=360, fill=SKIN, width=1)
    R(d, (x0, y0, x1, y0 + 3), SKIN, o=None)
    L(d, [(x0, y0 + 1), (x0, y1 - 3)], OUT)
    L(d, [(x1, y0 + 1), (x1, y1 - 3)], OUT)
    RR(d, (x0 + 2, y0 + 1, x1 - 2, y0 + 5), 1, (238, 200, 168, 255))  # nail
    for yy in (y0 + 11, y0 + 17):
        d.arc((x0 + 1, yy - 2, x1 - 1, yy + 2), start=0, end=180,
              fill=(188, 138, 96, 255), width=1)


def eye(d, cx=16, cy=16, w=11, h=7, iris=BLUE):
    P(d, [(cx - w, cy), (cx, cy - h), (cx + w, cy), (cx, cy + h)], WHITE)
    P(d, [(cx - w, cy), (cx, cy - h), (cx + w, cy), (cx, cy + h)], None,
      o=OUT)
    E(d, (cx - 4, cy - 4, cx + 4, cy + 4), iris, o=OUT)
    E(d, (cx - 2, cy - 2, cx + 2, cy + 2), STEEL_XD, o=None)
    E(d, (cx - 1, cy - 3, cx + 1, cy - 1), WHITE, o=None)


# ---------------------------------------------------------------- batch

def f_fitnessband(d, rng):
    # wristband with display: calorie flame + digits
    d.arc((3, 6, 29, 30), start=300, end=240, fill=STEEL_XD, width=6)
    d.arc((3, 6, 29, 30), start=300, end=240, fill=STEEL_D, width=4)
    RR(d, (9, 9, 23, 21), 2, PANEL)
    flame(d, 13, 14, 3)
    for x in (17, 20):
        R(d, (x, 12, x + 2, 14), GREEN, o=None)
    for x in (17, 20):
        R(d, (x, 16, x + 2, 18), GREEN, o=None)
    E(d, (15, 24, 17, 26), GREEN, o=None)


def f_flashbang(d, rng):
    canister(d, YELLOW)
    # starburst flash in panel
    for a in range(0, 360, 45):
        x = 16 + int(5.5 * math.cos(math.radians(a)))
        y = 20 + int(4.0 * math.sin(math.radians(a)))
        L(d, [(16, 20), (x, y)], YELLOW)
    E(d, (14, 18, 18, 22), WHITE, o=None)
    # speaker arcs on the cap
    d.arc((12, 0, 20, 7), start=20, end=160, fill=WHITE, width=1)


def f_flashlight(d, rng):
    # brow band with central lamp between the eyes, beams up
    d.arc((4, 12, 28, 30), start=180, end=360, fill=STEEL_D, width=3)
    RR(d, (11, 8, 21, 17), 2, STEEL)
    E(d, (13, 10, 19, 16), CYAN_PALE, o=OUT)
    E(d, (15, 12, 17, 14), WHITE, o=None)
    for dx in (-6, 0, 6):
        L(d, [(16 + dx // 2, 7), (16 + dx, 2)], YELLOW)
    E(d, (8, 21, 11, 24), STEEL_XD, o=None)     # eye hints below band
    E(d, (21, 21, 24, 24), STEEL_XD, o=None)


def f_fuel_cell_gasoline(d, rng):
    canister(d, ORANGE)
    drop(d, 14, 18, 4, ORANGE)
    L(d, [(19, 16), (21, 16), (21, 21)], STEEL_L, w=2)   # pump hose hint
    E(d, (20, 21, 22, 23), STEEL_L, o=None)
    L(d, [(12, 24), (20, 24)], RED_D)


def f_geiger(d, rng):
    canister(d, YELLOW)
    # radiation trefoil
    for a in (270, 30, 150):
        x = 16 + int(3.4 * math.cos(math.radians(a)))
        y = 19 + int(3.4 * math.sin(math.radians(a)))
        d.pieslice((x - 3, y - 3, x + 3, y + 3), start=a - 30, end=a + 30,
                   fill=YELLOW)
    E(d, (15, 18, 17, 20), YELLOW, o=None)
    L(d, [(12, 13), (20, 13)], STEEL_L)


def f_gills(d, rng):
    # throat respirator collar with gill slits and bubbles
    d.arc((5, 8, 27, 28), start=0, end=180, fill=STEEL, width=6)
    d.arc((5, 8, 27, 28), start=0, end=180, fill=STEEL_D, width=4)
    for i, x in enumerate((10, 14, 18, 22)):
        L(d, [(x, 17 + (i % 2)), (x - 2, 23 + (i % 2))], CYAN, w=2)
    E(d, (7, 5, 10, 8), CYAN_PALE, o=CYAN)
    E(d, (22, 3, 26, 7), CYAN_PALE, o=CYAN)
    E(d, (25, 8, 28, 11), CYAN_PALE, o=CYAN)
    E(d, (14, 11, 18, 15), PANEL, o=STEEL_XD)
    E(d, (15, 12, 17, 14), GREEN, o=None)


def f_glowy(d, rng):
    canister(d, GREEN)
    _cracked(d, rng)
    # glowing capacitor core showing through the panel
    E(d, (12, 15, 20, 23), ACID, o=None)
    E(d, (14, 17, 18, 21), (220, 255, 190, 255), o=None)
    bolt(d, 22, 12, 2, col=ACID)
    bolt(d, 10, 24, 2, col=ACID)


def f_ground_sonar(d, rng):
    # foot plate with sonar rings probing the ground
    RR(d, (8, 4, 24, 10), 2, STEEL_D)
    R(d, (10, 10, 22, 13), STEEL, o=OUT)
    E(d, (14, 5, 18, 9), CYAN, o=None)
    for r in (4, 7, 10):
        d.arc((16 - r, 14, 16 + r, 14 + 2 * r), start=20, end=160,
              fill=CYAN, width=1)
    L(d, [(4, 27), (28, 27)], (96, 74, 52, 255), w=2)    # ground line
    L(d, [(6, 30), (12, 30)], (96, 74, 52, 255))
    L(d, [(19, 30), (26, 30)], (96, 74, 52, 255))
    E(d, (21, 22, 25, 26), RED_D, o=None)                # buried thing


def f_heat_absorb(d, rng):
    canister(d, CYAN)
    # warm waves drawn into a cold core
    for x in (11, 14):
        d.arc((x - 2, 13, x + 2, 19), start=90, end=270, fill=ORANGE,
              width=1)
        d.arc((x - 2, 18, x + 2, 24), start=90, end=270, fill=RED,
              width=1)
    E(d, (17, 16, 22, 21), CYAN_PALE, o=CYAN)
    E(d, (18, 17, 20, 19), CYAN, o=None)
    L(d, [(15, 18), (17, 18)], ORANGE)


def f_heatsink(d, rng):
    # finned heatsink block woven into a flesh base
    R(d, (6, 22, 26, 28), SKIN, o=OUT)
    for i, x in enumerate(range(7, 25, 3)):
        R(d, (x, 6, x + 2, 23), STEEL_L if i % 2 else STEEL, o=OUT)
    R(d, (6, 20, 26, 23), STEEL_D, o=OUT)
    for x in range(8, 25, 4):
        E(d, (x, 24, x + 2, 26), (188, 138, 96, 255), o=None)
    L(d, [(9, 4), (9, 6)], ORANGE)
    L(d, [(16, 3), (16, 6)], RED)
    L(d, [(23, 4), (23, 6)], ORANGE)


def f_hydraulics(d, rng):
    # hydraulic piston: cylinder, rod, pressure hose
    R(d, (9, 6, 17, 20), STEEL, o=OUT)
    R(d, (9, 6, 17, 10), STEEL_D, o=OUT)
    R(d, (12, 20, 14, 27), STEEL_L, o=OUT)
    E(d, (10, 24, 16, 30), STEEL_D, o=OUT)
    d.arc((14, 8, 28, 24), start=270, end=90, fill=BLUE, width=2)
    E(d, (11, 12, 13, 14), CYAN, o=None)
    E(d, (11, 16, 13, 18), BLUE, o=None)
    L(d, [(20, 26), (26, 26)], STEEL_XD, w=2)
    L(d, [(23, 23), (23, 26)], STEEL_XD, w=2)


def f_infrared(d, rng):
    eye(d, iris=RED)
    # heat silhouette seen through the eye: warm figure to the side
    E(d, (25, 8, 29, 12), ORANGE, o=None)
    R(d, (26, 12, 28, 18), RED, o=None)
    for r in (5, 8):
        d.arc((2 - r + 4, 16 - r, 2 + r + 4, 16 + r), start=300, end=60,
              fill=RED, width=1)
    L(d, [(6, 26), (10, 26)], RED)
    L(d, [(22, 26), (26, 26)], ORANGE)


def f_int_enhancer(d, rng):
    # brain with coprocessor chip and traces
    E(d, (7, 6, 25, 20), PINK, o=OUT)
    d.arc((7, 4, 17, 14), start=180, end=330, fill=PINK, width=3)
    d.arc((15, 4, 25, 14), start=210, end=360, fill=PINK, width=3)
    L(d, [(11, 10), (9, 14)], RED_D)
    L(d, [(20, 9), (21, 13)], RED_D)
    R(d, (13, 11, 19, 17), STEEL_XD, o=CYAN)
    E(d, (15, 13, 17, 15), CYAN, o=None)
    for x in (14, 18):
        L(d, [(x, 17), (x, 20)], CYAN)
    L(d, [(13, 14), (10, 14)], CYAN)
    L(d, [(19, 14), (22, 14)], CYAN)
    L(d, [(14, 23), (18, 23)], STEEL_D)
    L(d, [(16, 20), (16, 23)], STEEL_D)


def f_itchy(d, rng):
    # misshapen lumpy module, badly installed, itch zigzags
    RR(d, (8, 6, 24, 26), 5, STEEL_D)
    E(d, (7, 10, 13, 16), STEEL, o=OUT)
    E(d, (19, 20, 26, 27), STEEL, o=OUT)
    _cracked(d, rng)
    E(d, (14, 13, 18, 17), RED_D, o=None)
    for (x, y) in [(5, 5), (25, 7), (6, 24)]:
        L(d, [(x, y), (x + 2, y - 2), (x + 4, y)], YELLOW)
    L(d, [(11, 28), (21, 28)], STEEL_XD, w=2)


def f_jointservo(d, rng):
    # knee joint with servo motor housing and gear
    R(d, (11, 3, 21, 11), SKIN, o=OUT)          # thigh stub
    R(d, (11, 21, 21, 29), SKIN, o=OUT)         # shin stub
    E(d, (9, 11, 23, 21), STEEL, o=OUT)         # hinge
    E(d, (13, 14, 19, 18), STEEL_XD, o=None)
    for a in range(0, 360, 60):
        x = 16 + int(6.5 * math.cos(math.radians(a)))
        y = 16 + int(6.5 * math.sin(math.radians(a)))
        E(d, (x - 1, y - 1, x + 1, y + 1), STEEL_D, o=None)
    RR(d, (21, 12, 28, 20), 2, STEEL_D)         # servo motor
    E(d, (23, 14, 26, 17), CYAN, o=None)
    L(d, [(13, 7), (19, 7)], STEEL_D)
    L(d, [(13, 25), (19, 25)], STEEL_D)


def f_lampoil(d, rng):
    canister(d, BRASS)
    drop(d, 16, 16, 3, (52, 44, 40, 255))
    flame(d, 16, 22, 3)
    L(d, [(12, 12), (20, 12)], BRASS)


def f_laser(d, rng):
    finger(d)
    E(d, (13, 2, 19, 8), STEEL_D, o=OUT)        # emitter housing at tip
    E(d, (15, 4, 17, 6), RED, o=None)
    L(d, [(16, 2), (16, 0)], RED, w=1)
    E(d, (15, 0, 17, 1), RED, o=None)
    L(d, [(13, 13), (19, 13)], CYAN)


def f_leaky(d, rng):
    canister(d, ACID)
    _cracked(d, rng)
    drop(d, 12, 26, 2, ACID)
    drop(d, 18, 27, 2, ACID)
    E(d, (10, 28, 22, 30), ACID, o=None)        # puddle
    E(d, (14, 16, 18, 20), ACID, o=None)


def f_leukocyte(d, rng):
    canister(d, WHITE)
    # white blood cells: pale blobs with nuclei
    for (cx, cy, r) in [(13, 17, 3), (19, 20, 4), (14, 23, 2)]:
        E(d, (cx - r, cy - r, cx + r, cy + r), (226, 220, 235, 255),
          o=STEEL_L)
        E(d, (cx - 1, cy - 1, cx + 1, cy + 1), VIOLET, o=None)
    L(d, [(11, 12), (21, 12)], WHITE)


def f_lighter(d, rng):
    finger(d)
    R(d, (13, 3, 19, 8), STEEL_D, o=OUT)        # lighter housing at tip
    flame(d, 16, 1, 2)
    E(d, (15, 5, 17, 7), ORANGE, o=None)
    L(d, [(13, 14), (19, 14)], STEEL_D)


def f_lockpick(d, rng):
    finger(d)
    # thin pick extended from the fingertip toward a lock
    L(d, [(16, 5), (16, 1)], STEEL_L, w=1)
    L(d, [(16, 1), (19, 1)], STEEL_L, w=1)
    RR(d, (22, 6, 30, 16), 2, BRASS_D)
    E(d, (24, 9, 28, 13), PANEL, o=BRASS)
    L(d, [(26, 13), (26, 15)], PANEL, w=2)
    L(d, [(13, 13), (19, 13)], CYAN)


def f_magnet(d, rng):
    # horseshoe electromagnet in a forearm housing
    RR(d, (5, 18, 27, 27), 2, STEEL_D)
    d.arc((9, 5, 23, 21), start=180, end=360, fill=RED, width=5)
    R(d, (9, 12, 13, 19), RED, o=OUT)
    R(d, (19, 12, 23, 19), STEEL_L, o=OUT)
    R(d, (9, 17, 13, 19), WHITE, o=None)
    R(d, (19, 17, 23, 19), RED_D, o=None)
    for (x, y) in [(6, 24), (25, 23)]:
        L(d, [(x, y), (x + 2, y - 2)], STEEL_L)
    E(d, (14, 22, 18, 25), STEEL_XD, o=None)    # attracted bolt


def f_membrane(d, rng):
    eye(d, iris=CYAN)
    # membrane sweeping across from the inner corner + water drops
    d.pieslice((5, 9, 27, 23), start=250, end=340, fill=CYAN_PALE)
    L(d, [(8, 11), (20, 9)], CYAN)
    drop(d, 8, 25, 2, CYAN)
    drop(d, 24, 26, 2, CYAN_PALE)


def f_memory(d, rng):
    canister(d, CYAN)
    # quantum drive stack: layered chips
    for i, y in enumerate((13, 17, 21)):
        R(d, (12, y, 20, y + 3), STEEL_L if i % 2 else STEEL, o=OUT)
        E(d, (18, y + 1, 19, y + 2), CYAN, o=None)
    L(d, [(13, 12), (13, 24)], CYAN)
    L(d, [(19, 12), (19, 24)], CYAN)


def f_metabolics(d, rng):
    canister(d, ORANGE)
    # circular exchange: flame -> battery
    flame(d, 13, 17, 3)
    R(d, (18, 14, 22, 20), None, o=WHITE)
    R(d, (22, 16, 23, 18), WHITE, o=None)
    R(d, (19, 16, 21, 18), YELLOW, o=None)
    d.arc((11, 20, 21, 26), start=0, end=180, fill=ORANGE, width=1)
    P(d, [(11, 23), (9, 21), (13, 21)], ORANGE)
    d.arc((11, 11, 21, 17), start=180, end=360, fill=YELLOW, width=1)
    P(d, [(21, 14), (19, 12), (23, 12)], YELLOW)


def f_meteorologist(d, rng):
    canister(d, CYAN)
    E(d, (12, 13, 17, 18), YELLOW, o=None)      # sun
    E(d, (15, 16, 22, 20), WHITE, o=None)       # cloud
    R(d, (15, 16, 22, 18), WHITE, o=None)
    L(d, [(12, 22), (15, 22)], CYAN_PALE)
    L(d, [(17, 23), (21, 23)], CYAN_PALE)
    L(d, [(13, 24), (16, 24)], CYAN_PALE)


def f_mp3(d, rng):
    # ear with speaker grille, music note, port behind the earlobe
    d.arc((10, 6, 26, 26), start=270, end=120, fill=SKIN, width=5)
    d.arc((14, 10, 23, 21), start=270, end=120, fill=STEEL_D, width=2)
    for yy in (13, 16, 19):
        L(d, [(16, yy), (20, yy)], STEEL_L)
    E(d, (8, 22, 11, 25), STEEL_XD, o=OUT)      # port
    E(d, (9, 23, 10, 24), CYAN, o=None)
    P(d, [(24, 6), (27, 5), (27, 11), (24, 12)], VIOLET)
    E(d, (22, 11, 25, 14), VIOLET, o=None)


def f_multimeter(d, rng):
    # forearm unit: big dial + red/black probes on cables
    RR(d, (7, 8, 21, 26), 2, STEEL_D)
    E(d, (10, 11, 18, 19), WHITE, o=OUT)
    L(d, [(14, 15), (17, 12)], RED)
    E(d, (13, 14, 15, 16), STEEL_XD, o=None)
    for x in (11, 14, 17):
        L(d, [(x, 21), (x, 23)], STEEL_L)
    d.arc((18, 18, 28, 30), start=180, end=300, fill=RED_D, width=1)
    d.arc((16, 20, 26, 30), start=180, end=300, fill=STEEL_XD, width=1)
    L(d, [(25, 20), (27, 16)], RED, w=2)
    L(d, [(23, 22), (24, 18)], STEEL_L, w=2)
    R(d, (8, 5, 20, 8), YELLOW, o=OUT)


def f_nanobots(d, rng):
    canister(d, CYAN)
    # swarm of tiny repair bots around a medical cross
    R(d, (14, 16, 18, 24), RED, o=None)
    R(d, (12, 18, 20, 22), RED, o=None)
    for (x, y) in [(11, 13), (20, 13), (10, 21), (21, 24), (16, 25)]:
        R(d, (x, y, x + 2, y + 2), STEEL_L, o=None)
        E(d, (x, y, x + 1, y + 1), CYAN, o=None)


def f_night(d, rng):
    # sphere of artificial darkness
    E(d, (5, 5, 27, 27), (10, 10, 14, 255), o=STEEL_XD)
    for (x, y) in [(11, 10), (20, 9), (23, 16), (10, 20), (17, 23)]:
        E(d, (x, y, x + 1, y + 1), WHITE, o=None)
    d.arc((8, 8, 24, 24), start=40, end=140, fill=VIOLET, width=1)
    d.arc((8, 8, 24, 24), start=220, end=320, fill=VIOLET, width=1)
    E(d, (21, 6, 24, 9), YELLOW, o=None)        # moon outside the sphere


def f_night_vision(d, rng):
    eye(d, iris=GREEN)
    E(d, (13, 13, 19, 19), GREEN, o=None)
    E(d, (14, 14, 18, 18), (16, 40, 20, 255), o=None)
    E(d, (15, 12, 17, 14), WHITE, o=None)
    E(d, (25, 5, 27, 7), WHITE, o=None)         # star
    L(d, [(26, 4), (26, 8)], WHITE)
    L(d, [(24, 6), (28, 6)], WHITE)
    for r in (12, 15):
        d.arc((16 - r, 16 - r, 16 + r, 16 + r), start=200, end=300,
              fill=GREEN, width=1)


def f_nl_antigravity_jump(d, rng):
    # shin unit with antigravity rings and a leap arc
    R(d, (12, 3, 20, 14), STEEL, o=OUT)          # shin housing
    R(d, (12, 3, 20, 6), STEEL_D, o=OUT)
    E(d, (14, 8, 18, 12), VIOLET, o=None)
    RR(d, (10, 14, 22, 18), 1, STEEL_D)          # foot plate
    for r in (5, 8, 11):
        d.arc((16 - r, 17, 16 + r, 17 + r), start=0, end=180,
              fill=VIOLET, width=1)
    L(d, [(7, 8), (5, 5)], CYAN)                # leap arc hint
    L(d, [(5, 5), (9, 3)], CYAN)
    P(d, [(9, 3), (8, 6), (11, 5)], CYAN)
    L(d, [(6, 27), (26, 27)], (96, 74, 52, 255), w=2)


def f_nl_blood_factory(d, rng):
    # bone cross-section with marrow factory producing red cells
    P(d, [(10, 5), (22, 5), (25, 9), (22, 13), (22, 19), (25, 23),
          (22, 27), (10, 27), (7, 23), (10, 19), (10, 13), (7, 9)],
      BONE)
    P(d, [(10, 5), (22, 5), (25, 9), (22, 13), (22, 19), (25, 23),
          (22, 27), (10, 27), (7, 23), (10, 19), (10, 13), (7, 9)],
      None, o=OUT)
    R(d, (12, 11, 20, 21), (196, 120, 110, 255), o=RED_D)   # marrow
    for (x, y) in [(13, 13), (17, 14), (15, 18), (18, 19), (13, 19)]:
        E(d, (x, y, x + 2, y + 2), RED, o=None)
    E(d, (15, 15, 17, 17), STEEL_L, o=None)      # nanobot


def f_nl_collar(d, rng, active=True):
    # neck stub with a collar ring and mystery implant lump
    R(d, (12, 14, 20, 29), SKIN, o=OUT)
    d.arc((11, 4, 21, 12), start=180, end=360, fill=SKIN, width=3)
    E(d, (6, 10, 26, 18), STEEL_D, o=OUT)        # collar band
    E(d, (8, 12, 24, 16), STEEL_XD, o=None)
    RR(d, (13, 8, 19, 13), 1, STEEL)             # nape lump
    if active:
        E(d, (15, 9, 17, 11), RED, o=None)       # blinking light
        d.arc((4, 6, 28, 20), start=300, end=60, fill=RED, width=1)
    else:
        E(d, (15, 9, 17, 11), STEEL_XD, o=None)  # dark, dead light
        L(d, [(13, 20), (19, 24)], STEEL_XD)     # scuff
    L(d, [(13, 22), (19, 22)], (188, 138, 96, 255))


def f_nl_collar_deactivated(d, rng):
    f_nl_collar(d, rng, active=False)


def f_nl_dimensional_anchor(d, rng):
    # anchor glyph locked inside a dimensional ring
    E(d, (5, 5, 27, 27), None, o=VIOLET, w=2)
    E(d, (8, 8, 24, 24), None, o=CYAN, w=1)
    L(d, [(16, 10), (16, 23)], STEEL_L, w=2)
    E(d, (14, 8, 18, 12), None, o=STEEL_L, w=1)
    L(d, [(11, 14), (21, 14)], STEEL_L, w=2)
    d.arc((10, 17, 22, 27), start=0, end=180, fill=STEEL_L, width=2)
    P(d, [(10, 19), (7, 16), (13, 16)], STEEL_L)
    P(d, [(22, 19), (19, 16), (25, 16)], STEEL_L)


def f_nl_hair(d, rng):
    # scalp with nanotube hair extruded from follicle sphincters
    d.pieslice((6, 10, 26, 28), start=180, end=360, fill=SKIN)
    L(d, [(6, 19), (26, 19)], OUT)
    strands = [(8, 12), (11, 9), (14, 8), (17, 8), (20, 9), (23, 12),
               (9, 15), (22, 15)]
    for (x, y) in strands:
        L(d, [(x, y + 4), (x - 1, y - 3)], STEEL_XD, w=1)
        L(d, [(x, y + 4), (x + 1, y - 2)], STEEL_XD, w=1)
        E(d, (x - 1, y + 3, x + 1, y + 5), STEEL_D, o=None)
    L(d, [(10, 24), (13, 24)], (188, 138, 96, 255))
    L(d, [(19, 24), (22, 24)], (188, 138, 96, 255))


def f_nl_laser_reader(d, rng):
    # open book under a red laser scan line with a reading light
    P(d, [(5, 12), (16, 9), (16, 25), (5, 22)], WHITE)
    P(d, [(27, 12), (16, 9), (16, 25), (27, 22)], WHITE)
    P(d, [(5, 12), (16, 9), (16, 25), (5, 22)], None, o=OUT)
    P(d, [(27, 12), (16, 9), (16, 25), (27, 22)], None, o=OUT)
    for yy in (14, 17, 20):
        L(d, [(8, yy), (14, yy - 1)], STEEL_L)
        L(d, [(18, yy - 1), (24, yy)], STEEL_L)
    L(d, [(6, 15), (26, 13)], RED, w=1)
    E(d, (15, 12, 17, 14), RED, o=None)
    RR(d, (13, 4, 19, 8), 1, STEEL_D)
    E(d, (15, 5, 17, 7), YELLOW, o=None)


def f_nl_psyshield(d, rng):
    # head silhouette coated by a shield layer, waves deflected
    E(d, (10, 6, 22, 20), SKIN, o=OUT)
    R(d, (14, 20, 18, 24), SKIN, o=OUT)
    d.arc((7, 3, 25, 23), start=180, end=360, fill=VIOLET, width=2)
    d.arc((5, 1, 27, 25), start=190, end=350, fill=CYAN_PALE, width=1)
    E(d, (13, 12, 15, 14), STEEL_XD, o=None)
    E(d, (17, 12, 19, 14), STEEL_XD, o=None)
    for (x, y) in [(4, 10), (28, 8)]:
        L(d, [(x, y), (x + 2, y + 2)], VIOLET)
        L(d, [(x + 2, y), (x, y + 2)], VIOLET)


def f_noise(d, rng):
    canister(d, RED)
    _cracked(d, rng)
    # speaker with jagged loud bursts
    R(d, (12, 16, 15, 22), STEEL_L, o=OUT)
    P(d, [(15, 17), (19, 14), (19, 24), (15, 21)], STEEL_L)
    for (x, y) in [(21, 15), (22, 19), (21, 23)]:
        L(d, [(x, y), (x + 2, y - 1), (x + 1, y + 1)], YELLOW)


def f_nostril(d, rng):
    # nose with a module jammed crookedly into one nostril
    P(d, [(16, 4), (20, 16), (22, 21), (19, 24), (13, 24), (10, 21),
          (12, 16)], SKIN)
    P(d, [(16, 4), (20, 16), (22, 21), (19, 24), (13, 24), (10, 21),
          (12, 16)], None, o=OUT)
    E(d, (12, 21, 15, 24), (188, 138, 96, 255), o=OUT)
    E(d, (17, 21, 20, 24), (188, 138, 96, 255), o=OUT)
    RR(d, (11, 19, 16, 25), 1, STEEL_D)          # misplaced module
    E(d, (12, 20, 14, 22), RED, o=None)
    L(d, [(18, 26), (20, 28)], CYAN)            # drip of misery
    drop(d, 19, 28, 1, CYAN)


def f_ods(d, rng):
    # torso silhouette inside a shocking defense field
    E(d, (4, 4, 28, 28), None, o=CYAN, w=2)
    P(d, [(12, 9), (20, 9), (23, 13), (21, 24), (11, 24), (9, 13)],
      STEEL_D)
    P(d, [(12, 9), (20, 9), (23, 13), (21, 24), (11, 24), (9, 13)],
      None, o=OUT)
    E(d, (14, 5, 18, 9), SKIN, o=OUT)
    bolt(d, 8, 12, 3)
    bolt(d, 24, 20, 3)
    bolt(d, 16, 16, 3, col=CYAN_PALE)


def f_painkiller(d, rng):
    canister(d, VIOLET)
    # nerve signal waveform being dulled flat
    L(d, [(11, 17), (13, 17), (14, 14), (15, 20), (16, 17), (18, 17)],
      PINK, w=1)
    L(d, [(18, 19), (21, 19)], CYAN_PALE)
    P(d, [(21, 19), (20, 17), (20, 21)], CYAN_PALE)
    L(d, [(11, 22), (21, 22)], STEEL_D)
    L(d, [(11, 22), (13, 20)], VIOLET)


def f_pitch_perfect(d, rng):
    # reticle eye linked to a throwing arm arc
    E(d, (5, 5, 15, 15), None, o=CYAN, w=1)
    E(d, (8, 8, 12, 12), CYAN, o=None)
    L(d, [(10, 3), (10, 5)], CYAN)
    L(d, [(10, 15), (10, 17)], CYAN)
    d.arc((8, 10, 28, 28), start=270, end=360, fill=WHITE, width=1)
    for t in (0.3, 0.55, 0.8):
        x = 10 + int(16 * t)
        y = 17 - int(6 * math.sin(math.pi * t)) + int(8 * t)
        E(d, (x, y, x + 1, y + 1), WHITE, o=None)
    E(d, (24, 20, 28, 24), ORANGE, o=OUT)        # ball
    L(d, [(14, 13), (20, 16)], VIOLET)          # nerve link
    L(d, [(23, 24), (26, 27)], SKIN, w=2)       # hand hint


def f_pokedeye(d, rng):
    eye(d, iris=RED)
    # dielectric stylette poking the eye, irritation cracks
    L(d, [(24, 4), (18, 13)], STEEL_L, w=2)
    L(d, [(24, 4), (27, 6)], STEEL_L, w=2)
    E(d, (17, 12, 19, 14), RED, o=None)
    L(d, [(12, 20), (10, 24)], RED_D)
    L(d, [(20, 21), (22, 24)], RED_D)
    d.arc((4, 8, 28, 24), start=150, end=200, fill=RED, width=1)


def _battery(d, rng, mk2):
    canister(d, YELLOW)
    R(d, (12, 13, 19, 23), None, o=WHITE)
    R(d, (19, 16, 21, 20), WHITE, o=None)
    if mk2:
        R(d, (13, 14, 18, 22), YELLOW, o=None)   # fuller charge (250 kJ)
        L(d, [(13, 18), (18, 18)], STEEL_D)
        L(d, [(14, 24), (14, 25)], WHITE)        # "II" ticks
        L(d, [(17, 24), (17, 25)], WHITE)
    else:
        R(d, (13, 18, 18, 22), YELLOW, o=None)   # partial charge (100 kJ)
    bolt(d, 16, 10, 2, col=WHITE)


def f_power_storage(d, rng):
    _battery(d, rng, False)


def f_power_storage_mkII(d, rng):
    _battery(d, rng, True)


def f_power_weakness(d, rng):
    canister(d, RED)
    _cracked(d, rng)
    R(d, (12, 14, 18, 21), None, o=WHITE)
    R(d, (18, 16, 20, 19), WHITE, o=None)
    R(d, (13, 15, 17, 20), RED, o=None)
    bolt(d, 21, 12, 3)
    bolt(d, 11, 23, 2, col=CYAN_PALE)
    P(d, [(14, 24), (18, 24), (16, 27)], RED_D)  # sagging output


def f_probability_travel(d, rng):
    # ghosted figure quantum-tunneling through a wall
    for (x, y) in [(14, 6), (20, 6), (14, 12), (20, 12), (14, 18),
                   (20, 18)]:
        R(d, (x, y, x + 5, y + 5), STEEL_D, o=OUT)
    # figure: solid half left, dotted half right of the wall
    E(d, (7, 9, 12, 14), CYAN_PALE, o=CYAN)
    R(d, (8, 14, 11, 21), CYAN, o=None)
    L(d, [(8, 21), (7, 26)], CYAN, w=2)
    L(d, [(11, 21), (12, 26)], CYAN, w=2)
    for (x, y) in [(24, 10), (26, 13), (24, 16), (26, 20), (24, 23)]:
        E(d, (x, y, x + 1, y + 1), CYAN_PALE, o=None)
    for yy in (8, 15, 22):
        L(d, [(3, yy), (6, yy)], VIOLET)
        L(d, [(27, yy), (30, yy)], VIOLET)


def f_purifier(d, rng):
    # trachea tube with filter layers: toxins stopped, clean air through
    R(d, (13, 4, 19, 28), STEEL_L, o=OUT)
    R(d, (13, 4, 19, 8), STEEL_D, o=OUT)
    for yy in (13, 16, 19):
        L(d, [(13, yy), (19, yy)], STEEL_XD, w=2)
        L(d, [(14, yy - 1), (18, yy + 1)], CYAN)
    for (x, y) in [(9, 7), (23, 6), (8, 11)]:
        E(d, (x, y, x + 2, y + 2), ACID, o=None)   # toxin motes outside
    for yy in (23, 26):
        L(d, [(14, yy), (18, yy)], CYAN_PALE)
    E(d, (15, 9, 17, 11), WHITE, o=None)


def _ear_radio(d, rng, pat):
    d.arc((9, 6, 25, 26), start=270, end=120, fill=SKIN, width=5)
    d.arc((13, 10, 22, 21), start=270, end=120, fill=STEEL_D, width=2)
    E(d, (15, 14, 18, 17), STEEL_XD, o=None)
    E(d, (7, 20, 10, 23), STEEL_D, o=OUT)          # earpiece module
    E(d, (8, 21, 9, 22), GREEN, o=None)
    col = VIOLET if pat else CYAN
    for r in (5, 8, 11):
        d.arc((20 - r, 12 - r, 20 + r, 12 + r), start=290, end=70,
              fill=col, width=1)
    if pat:
        # custom software patch: gear mark + extra wave band
        E(d, (24, 20, 29, 25), VIOLET, o=OUT)
        E(d, (25, 21, 28, 24), PANEL, o=None)
        E(d, (26, 22, 27, 23), WHITE, o=None)
        d.arc((9, 1, 31, 23), start=290, end=70, fill=VIOLET, width=1)


def f_radio(d, rng):
    _ear_radio(d, rng, False)


def f_radio_pat(d, rng):
    _ear_radio(d, rng, True)


DRAW = {
    "bio_fitnessband": f_fitnessband,
    "bio_flashbang": f_flashbang,
    "bio_flashlight": f_flashlight,
    "bio_fuel_cell_gasoline": f_fuel_cell_gasoline,
    "bio_geiger": f_geiger,
    "bio_gills": f_gills,
    "bio_glowy": f_glowy,
    "bio_ground_sonar": f_ground_sonar,
    "bio_heat_absorb": f_heat_absorb,
    "bio_heatsink": f_heatsink,
    "bio_hydraulics": f_hydraulics,
    "bio_infrared": f_infrared,
    "bio_int_enhancer": f_int_enhancer,
    "bio_itchy": f_itchy,
    "bio_jointservo": f_jointservo,
    "bio_lampoil": f_lampoil,
    "bio_laser": f_laser,
    "bio_leaky": f_leaky,
    "bio_leukocyte": f_leukocyte,
    "bio_lighter": f_lighter,
    "bio_lockpick": f_lockpick,
    "bio_magnet": f_magnet,
    "bio_membrane": f_membrane,
    "bio_memory": f_memory,
    "bio_metabolics": f_metabolics,
    "bio_meteorologist": f_meteorologist,
    "bio_mp3": f_mp3,
    "bio_multimeter": f_multimeter,
    "bio_nanobots": f_nanobots,
    "bio_night": f_night,
    "bio_night_vision": f_night_vision,
    "bio_nl_antigravity_jump": f_nl_antigravity_jump,
    "bio_nl_blood_factory": f_nl_blood_factory,
    "bio_nl_collar": f_nl_collar,
    "bio_nl_collar_deactivated": f_nl_collar_deactivated,
    "bio_nl_dimensional_anchor": f_nl_dimensional_anchor,
    "bio_nl_hair": f_nl_hair,
    "bio_nl_laser_reader": f_nl_laser_reader,
    "bio_nl_psyshield": f_nl_psyshield,
    "bio_noise": f_noise,
    "bio_nostril": f_nostril,
    "bio_ods": f_ods,
    "bio_painkiller": f_painkiller,
    "bio_pitch_perfect": f_pitch_perfect,
    "bio_pokedeye": f_pokedeye,
    "bio_power_storage": f_power_storage,
    "bio_power_storage_mkII": f_power_storage_mkII,
    "bio_power_weakness": f_power_weakness,
    "bio_probability_travel": f_probability_travel,
    "bio_purifier": f_purifier,
    "bio_radio": f_radio,
    "bio_radio_pat": f_radio_pat,
}

IDS = [l.strip() for l in open("/tmp/item_batches/02.txt") if l.strip()]


def main():
    assert len(IDS) == 52 and set(IDS) == set(DRAW), (
        len(IDS), set(IDS) ^ set(DRAW))
    os.makedirs(BASE, exist_ok=True)
    imgs = {}
    for pid in IDS:
        rng = random.Random(zlib.crc32(pid.encode()))
        img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        DRAW[pid](d, rng)
        # opacity discipline: bodies opaque, background transparent
        alphas = {a for a in img.getchannel("A").getdata()}
        assert alphas <= {0, 255}, (pid, alphas)
        imgs[pid] = img
        folder = os.path.join(BASE, pid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, pid + ".png"))
        with open(os.path.join(folder, pid + ".json"), "w") as f:
            json.dump([{"id": pid, "fg": pid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", pid)
    # contact-sheet preview: 8 cols, 2x scale on dark background
    cols = 8
    rows = (len(IDS) + cols - 1) // cols
    cell = 68
    sheet = Image.new("RGBA", (cols * cell + 4, rows * cell + 4),
                      (34, 32, 36, 255))
    for i, pid in enumerate(IDS):
        big = imgs[pid].resize((64, 64), Image.NEAREST)
        x = 4 + (i % cols) * cell
        y = 4 + (i // cols) * cell
        sheet.alpha_composite(big, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item02.png")
    sheet.save(out)
    print("wrote preview_item02.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
