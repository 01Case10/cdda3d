#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch ITEM01: 52 bionic (CBM) item tiles.

Drawn from each bionic's in-game name/description
(fork-src/cdda/data/json/bionics.json), flat UltiCa style, native 32x32,
bodies fully opaque on transparent background. Internal bionics are drawn
as compact metallic implant modules with a function pictogram; armor
meshes as plated body-part shells; the Exodii cyberlimbs as the limbs
themselves (left/right mirrored, armored variants plated).
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_item01.png.
"""
import json
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
CARBON = (40, 40, 46, 255)
CARBON_L = (74, 74, 84, 255)


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
    """Compact bionic module: steel housing, cap, accent band, dark panel."""
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


def heart(d, cx, cy, s, col=RED):
    E(d, (cx - s, cy - s + 1, cx - 1, cy + 2), col, o=None)
    E(d, (cx + 1, cy - s + 1, cx + s, cy + 2), col, o=None)
    P(d, [(cx - s, cy + 1), (cx + s, cy + 1), (cx, cy + s + 2)], col)


def drop(d, cx, cy, s, col):
    P(d, [(cx, cy - s), (cx + s - 1, cy + 2), (cx - s + 1, cy + 2)], col)
    E(d, (cx - s + 1, cy, cx + s - 1, cy + s), col, o=None)


def mesh(d, box, base=STEEL, line=STEEL_XD):
    """Plated shell with crosshatch mesh lines, clipped by caller shapes."""
    R(d, box, base, o=None)
    x0, y0, x1, y1 = box
    for x in range(x0 + 3, x1, 4):
        L(d, [(x, y0), (x - 3, y1)], line)
        L(d, [(x, y0), (x + 3, y1)], line)


# ---------------------------------------------------------------- modules

def f_adrenaline(d, rng):
    canister(d, RED)
    heart(d, 16, 17, 4)
    L(d, [(11, 17), (14, 17), (15, 14), (17, 20), (18, 17), (21, 17)],
      WHITE)


def f_ads(d, rng):
    # forcefield dome over a small emitter base
    E(d, (4, 3, 28, 27), None, o=CYAN, w=2)
    E(d, (7, 6, 25, 24), None, o=CYAN_PALE, w=1)
    RR(d, (11, 22, 21, 28), 2, STEEL_D)
    E(d, (14, 18, 18, 22), CYAN, o=OUT)
    L(d, [(6, 10), (9, 12)], CYAN_PALE)
    L(d, [(24, 8), (22, 11)], CYAN_PALE)


def f_alarm(d, rng):
    canister(d, YELLOW)
    E(d, (14, 15, 18, 19), YELLOW, o=None)
    d.arc((10, 11, 22, 23), start=300, end=60, fill=YELLOW, width=1)
    d.arc((8, 9, 24, 25), start=300, end=60, fill=YELLOW, width=1)
    d.arc((10, 11, 22, 23), start=120, end=240, fill=YELLOW, width=1)
    d.arc((8, 9, 24, 25), start=120, end=240, fill=YELLOW, width=1)


def f_ankles(d, rng):
    # pair of squeaky ankle joints with sound marks
    for cx in (11, 21):
        RR(d, (cx - 3, 8, cx + 3, 18), 2, STEEL)
        E(d, (cx - 2, 16, cx + 2, 21), STEEL_D, o=OUT)
        RR(d, (cx - 4, 21, cx + 4, 26), 1, STEEL_D)   # foot plate
        d.arc((cx - 6, 4, cx + 6, 12), start=300, end=30, fill=YELLOW,
              width=1)
        d.arc((cx - 6, 4, cx + 6, 12), start=150, end=240, fill=YELLOW,
              width=1)
    L(d, [(14, 12), (18, 12)], STEEL_XD)


def f_ar(d, rng):
    # thin AR lenses with overlay brackets
    RR(d, (4, 11, 15, 20), 3, CYAN_PALE)
    RR(d, (17, 11, 28, 20), 3, CYAN_PALE)
    L(d, [(15, 14), (17, 14)], STEEL_D)
    for (x, y) in [(4, 11), (28, 11), (4, 20), (28, 20)]:
        pass
    for x0, y0, sx, sy in [(5, 8, 1, 1), (27, 8, -1, 1), (5, 23, 1, -1),
                           (27, 23, -1, -1)]:
        L(d, [(x0, y0), (x0 + 4 * sx, y0)], CYAN)
        L(d, [(x0, y0), (x0, y0 + 4 * sy)], CYAN)
    L(d, [(8, 15), (12, 15)], BLUE)
    L(d, [(20, 17), (24, 17)], BLUE)


def f_armor_arms(d, rng):
    # forearm shell with mesh plating
    P(d, [(10, 4), (20, 4), (22, 14), (20, 27), (12, 27), (9, 14)], STEEL)
    P(d, [(10, 4), (20, 4), (22, 14), (20, 27), (12, 27), (9, 14)],
      None, o=OUT)
    for yy in (9, 14, 19, 24):
        L(d, [(10, yy), (21, yy - 1)], STEEL_XD)
    for xx in (13, 17):
        L(d, [(xx, 5), (xx + 1, 26)], STEEL_D)
    E(d, (12, 24, 20, 29), STEEL_D, o=OUT)   # hand hint


def f_armor_eyes(d, rng):
    # mirrored protective lenses in a dark frame
    RR(d, (3, 10, 29, 22), 5, STEEL_XD)
    E(d, (5, 11, 15, 21), CYAN_PALE, o=OUT)
    E(d, (17, 11, 27, 21), CYAN_PALE, o=OUT)
    L(d, [(7, 13), (11, 19)], WHITE)
    L(d, [(19, 13), (23, 19)], WHITE)
    L(d, [(15, 14), (17, 14)], STEEL_L)


def f_armor_head(d, rng):
    # head shell with mesh plating
    E(d, (7, 3, 25, 21), STEEL, o=OUT)
    d.pieslice((7, 3, 25, 21), start=180, end=360, fill=STEEL_L)
    for yy in (8, 12, 16):
        d.arc((8, 2, 24, 22), start=200, end=340, fill=STEEL_XD, width=1)
        break
    L(d, [(9, 10), (23, 10)], STEEL_XD)
    L(d, [(10, 15), (22, 15)], STEEL_XD)
    L(d, [(12, 5), (11, 19)], STEEL_D)
    L(d, [(20, 5), (21, 19)], STEEL_D)
    RR(d, (12, 20, 20, 27), 2, STEEL_D)   # jaw plate


def f_armor_legs(d, rng):
    # pair of shin shells with mesh plating
    for x0 in (6, 18):
        P(d, [(x0, 4), (x0 + 8, 4), (x0 + 7, 22), (x0 + 6, 27),
              (x0 + 1, 27), (x0, 22)], STEEL)
        P(d, [(x0, 4), (x0 + 8, 4), (x0 + 7, 22), (x0 + 6, 27),
              (x0 + 1, 27), (x0, 22)], None, o=OUT)
        for yy in (9, 14, 19):
            L(d, [(x0 + 1, yy), (x0 + 7, yy)], STEEL_XD)
        L(d, [(x0 + 4, 5), (x0 + 4, 26)], STEEL_D)


def f_armor_torso(d, rng):
    # torso cuirass shell with mesh plating
    P(d, [(9, 4), (23, 4), (26, 10), (24, 24), (21, 28), (11, 28),
          (8, 24), (6, 10)], STEEL)
    P(d, [(9, 4), (23, 4), (26, 10), (24, 24), (21, 28), (11, 28),
          (8, 24), (6, 10)], None, o=OUT)
    for yy in (9, 14, 19, 24):
        L(d, [(8, yy), (24, yy)], STEEL_XD)
    L(d, [(16, 5), (16, 27)], STEEL_D)
    L(d, [(10, 6), (22, 6)], STEEL_L)


def f_batteries(d, rng):
    canister(d, YELLOW)
    R(d, (12, 13, 19, 21), None, o=WHITE)
    R(d, (19, 15, 21, 19), WHITE, o=None)
    R(d, (13, 15, 15, 19), YELLOW, o=None)
    R(d, (16, 15, 18, 19), YELLOW, o=None)


def f_blade(d, rng):
    # forearm housing with extended monomolecular blade
    RR(d, (4, 18, 16, 27), 2, STEEL_D)
    R(d, (5, 20, 15, 22), STEEL)
    P(d, [(13, 19), (28, 4), (29, 6), (16, 21)], STEEL_L)
    L(d, [(14, 20), (28, 5)], WHITE)
    E(d, (6, 21, 9, 24), STEEL_XD, o=None)


def f_blood_anal(d, rng):
    canister(d, RED)
    heart(d, 16, 15, 3, col=RED_D)
    drop(d, 16, 20, 3, RED)
    L(d, [(12, 24), (14, 24), (15, 22), (17, 25), (18, 24), (20, 24)],
      CYAN)


def f_blood_filter(d, rng):
    canister(d, CYAN)
    P(d, [(12, 12), (20, 12), (17, 17), (17, 19), (15, 19), (15, 17)],
      STEEL_L)
    L(d, [(12, 14), (20, 14)], CYAN)
    drop(d, 16, 22, 2, CYAN)
    drop(d, 13, 10, 1, RED)


def f_cable(d, rng):
    # hip port with coiled jumper cable
    E(d, (9, 6, 23, 20), STEEL, o=OUT)
    E(d, (12, 9, 20, 17), PANEL, o=STEEL_XD)
    E(d, (14, 11, 16, 13), STEEL_XD, o=None)
    E(d, (17, 11, 19, 13), STEEL_XD, o=None)
    d.arc((6, 14, 26, 30), start=20, end=160, fill=RED_D, width=2)
    d.arc((8, 15, 24, 28), start=20, end=160, fill=RED, width=1)
    R(d, (22, 22, 26, 25), STEEL_D, o=OUT)


def f_carbon(d, rng):
    # subdermal carbon filament weave patch
    RR(d, (6, 7, 26, 25), 3, CARBON)
    for i in range(4):
        for j in range(3):
            col = CARBON_L if (i + j) % 2 == 0 else CARBON
            R(d, (8 + i * 4, 9 + j * 5, 11 + i * 4, 13 + j * 5), col,
              o=None)
    RR(d, (6, 7, 26, 25), 3, None, o=STEEL_XD)
    L(d, [(8, 9), (13, 9)], WHITE)


def f_chain_lightning(d, rng):
    canister(d, CYAN)
    bolt(d, 15, 17, 5)
    bolt(d, 20, 20, 3, col=CYAN_PALE)
    bolt(d, 11, 21, 2, col=CYAN_PALE)


def f_claws(d, rng):
    # fist housing with three extended claws
    RR(d, (9, 16, 23, 28), 3, STEEL_D)
    R(d, (10, 18, 22, 21), STEEL)
    for x in (11, 16, 21):
        P(d, [(x - 2, 17), (x, 4), (x + 2, 17)], STEEL_L)
        L(d, [(x, 5), (x, 16)], WHITE)


def f_climate(d, rng):
    canister(d, ORANGE)
    R(d, (15, 12, 17, 20), WHITE, o=OUT)
    E(d, (13, 18, 19, 24), RED, o=OUT)
    R(d, (15, 15, 17, 19), RED, o=None)
    L(d, [(11, 13), (13, 13)], CYAN)
    L(d, [(19, 13), (21, 13)], ORANGE)


def f_cloak(d, rng):
    canister(d, VIOLET)
    # dotted fading silhouette
    for (x, y) in [(16, 12), (14, 14), (18, 14), (13, 17), (16, 17),
                   (19, 17), (14, 20), (18, 20), (13, 23), (19, 23)]:
        E(d, (x, y, x + 1, y + 1), CYAN_PALE, o=None)
    E(d, (11, 11, 13, 13), VIOLET, o=None)
    E(d, (19, 21, 21, 23), VIOLET, o=None)


def f_cqb(d, rng):
    canister(d, ORANGE)
    # processor chip with fist glyph
    R(d, (12, 12, 20, 20), STEEL_XD, o=STEEL_L)
    for x in (13, 16, 19):
        L(d, [(x, 10), (x, 12)], STEEL_L)
        L(d, [(x, 20), (x, 22)], STEEL_L)
    P(d, [(14, 18), (14, 14), (16, 13), (18, 14), (18, 18)], ORANGE)


def f_creeyncaggii(d, rng):
    # ornate mechanical heart of the Exodii
    heart(d, 16, 16, 7, col=BRASS_D)
    heart(d, 16, 15, 5, col=RED_D)
    E(d, (13, 12, 19, 18), (64, 220, 200, 255), o=OUT)
    L(d, [(16, 6), (16, 9)], BRASS)
    L(d, [(10, 8), (12, 11)], BRASS)
    L(d, [(22, 8), (20, 11)], BRASS)
    d.arc((8, 20, 24, 29), start=0, end=180, fill=BRASS, width=1)


def f_deformity(d, rng):
    # scarred, asymmetric face plate with stitches
    E(d, (7, 4, 25, 24), SKIN, o=OUT)
    E(d, (11, 11, 14, 14), STEEL_XD, o=None)
    E(d, (19, 10, 22, 13), STEEL_XD, o=None)
    L(d, [(11, 20), (21, 19)], RED_D)
    for x in (9, 13, 17, 21):
        L(d, [(x, 5), (x + 2, 8)], RED_D)
        L(d, [(x + 2, 5), (x, 8)], RED_D)
    L(d, [(20, 15), (24, 18)], RED_D)


def f_dex_enhancer(d, rng):
    canister(d, CYAN)
    L(d, [(12, 21), (14, 15), (16, 19), (18, 13), (20, 17)], CYAN_PALE)
    E(d, (13, 13, 16, 16), CYAN, o=None)
    E(d, (17, 11, 20, 14), CYAN, o=None)
    L(d, [(11, 23), (21, 23)], CYAN)


def f_digestion(d, rng):
    # three synthetic stomachs in a row
    for cx in (10, 16, 22):
        E(d, (cx - 4, 10, cx + 4, 20), PINK, o=OUT)
        d.arc((cx - 4, 8, cx + 4, 16), start=180, end=330, fill=PINK,
              width=2)
    L(d, [(8, 21), (24, 21)], PINK)
    d.arc((10, 19, 22, 27), start=0, end=180, fill=PINK, width=2)


def _cracked(d, rng):
    for _ in range(3):
        x = rng.randint(10, 20)
        L(d, [(x, 12), (x + 2, 16), (x, 19), (x + 1, 23)], STEEL_XD)


def f_dis_acid(d, rng):
    canister(d, ACID)
    _cracked(d, rng)
    drop(d, 13, 22, 2, ACID)
    drop(d, 19, 24, 2, ACID)
    E(d, (18, 13, 22, 17), ACID, o=None)


def f_dis_shock(d, rng):
    canister(d, YELLOW)
    _cracked(d, rng)
    bolt(d, 12, 10, 3)
    bolt(d, 21, 22, 3)
    bolt(d, 20, 9, 2, col=CYAN_PALE)


def f_drain(d, rng):
    canister(d, STEEL_D)
    R(d, (12, 12, 19, 18), None, o=STEEL_L)
    R(d, (19, 14, 21, 16), STEEL_L, o=None)
    P(d, [(15, 19), (17, 19), (16, 22)], STEEL_L)
    L(d, [(16, 22), (16, 24)], STEEL_L)
    E(d, (13, 23, 19, 25), STEEL_XD, o=None)


def f_ears(d, rng):
    # ear shell with incoming sound waves
    d.arc((10, 7, 26, 25), start=270, end=120, fill=SKIN, width=5)
    d.arc((14, 11, 23, 21), start=270, end=120, fill=STEEL_D, width=2)
    E(d, (16, 14, 19, 17), STEEL_XD, o=None)
    for r in (5, 8, 11):
        d.arc((16 - r, 16 - r, 16 + r, 16 + r), start=290, end=70,
              fill=CYAN, width=1)


def f_electrokit(d, rng):
    canister(d, BLUE)
    # multimeter dial + soldering iron
    E(d, (12, 12, 17, 17), WHITE, o=None)
    L(d, [(14, 14), (16, 13)], RED)
    L(d, [(18, 21), (21, 14)], STEEL_L, w=2)
    P(d, [(21, 14), (22, 12), (20, 13)], ORANGE)


def f_emp(d, rng):
    # forearm stub with parabolic EMP dish
    RR(d, (5, 19, 15, 27), 2, STEEL_D)
    d.arc((13, 6, 29, 22), start=300, end=130, fill=STEEL_L, width=3)
    d.arc((16, 9, 26, 19), start=300, end=130, fill=BLUE, width=1)
    L(d, [(14, 20), (20, 13)], STEEL)
    E(d, (19, 12, 22, 15), CYAN, o=None)


def f_emp_armgun(d, rng):
    # forearm projector with barrel at the palm + reticle
    RR(d, (4, 17, 18, 26), 2, STEEL_D)
    R(d, (16, 18, 24, 24), STEEL, o=OUT)
    E(d, (18, 19, 23, 24), PANEL, o=STEEL_XD)
    E(d, (19, 20, 22, 23), BLUE, o=None)
    E(d, (23, 6, 30, 13), None, o=RED)
    L(d, [(26, 4), (26, 6)], RED)
    L(d, [(26, 13), (26, 15)], RED)
    L(d, [(21, 9), (23, 9)], RED)
    L(d, [(30, 9), (31, 9)], RED)


def f_ethanol(d, rng):
    canister(d, ORANGE)
    drop(d, 16, 14, 2, BLUE)
    P(d, [(13, 22), (16, 16), (19, 22)], ORANGE)
    P(d, [(14, 23), (16, 19), (18, 23)], YELLOW)


def f_evap(d, rng):
    canister(d, CYAN)
    E(d, (12, 17, 20, 24), STEEL_XD, o=STEEL_L)
    for a in (0, 60, 120, 180, 240, 300):
        import math
        x = 16 + int(3.2 * math.cos(math.radians(a)))
        y = 20 + int(3.2 * math.sin(math.radians(a)))
        L(d, [(16, 20), (x, y)], CYAN)
    drop(d, 13, 12, 1, CYAN)
    drop(d, 19, 11, 1, CYAN_PALE)


# ------------------------------------------------------------- cyberlimbs

def _arm(d, side, armored):
    """Cybernetic arm, vertical. side: -1 left (hand at left), +1 right."""
    w_up = 7 if armored else 5      # upper arm width
    w_fo = 8 if armored else 6      # forearm width
    cx = 16
    E(d, (cx - 5, 2, cx + 5, 9), STEEL_D, o=OUT)              # shoulder
    R(d, (cx - w_up // 2, 7, cx + w_up // 2, 15),
      STEEL if not armored else STEEL_D, o=OUT)               # upper arm
    E(d, (cx - 3, 14, cx + 3, 19), STEEL_XD, o=OUT)           # elbow
    R(d, (cx - w_fo // 2, 18, cx + w_fo // 2, 25), STEEL, o=OUT)  # forearm
    if armored:
        R(d, (cx - w_fo // 2 - 1, 19, cx - w_fo // 2 + 1, 24), STEEL_L,
          o=OUT)
        R(d, (cx + w_fo // 2 - 1, 19, cx + w_fo // 2 + 1, 24), STEEL_L,
          o=OUT)
        R(d, (cx - w_up // 2 - 1, 8, cx - w_up // 2 + 1, 13), STEEL_L,
          o=OUT)
    # hand: palm + fingers on the given side
    hx = cx + side * 2
    RR(d, (hx - 3, 25, hx + 3, 29), 1, STEEL_D)
    for i in range(3):
        fx = hx - 2 + i * 2
        L(d, [(fx, 29), (fx, 31)], STEEL_D, w=1)
    L(d, [(cx + side * (w_fo // 2), 21), (hx + side * 2, 26)], STEEL_D)
    E(d, (cx - 1, 10, cx + 1, 12), CYAN, o=None)


def _leg(d, side, armored):
    """Cybernetic leg, vertical, foot pointing to `side`."""
    w_th = 8 if armored else 6
    w_sh = 7 if armored else 5
    cx = 16
    R(d, (cx - w_th // 2, 3, cx + w_th // 2, 13),
      STEEL if not armored else STEEL_D, o=OUT)               # thigh
    E(d, (cx - 3, 12, cx + 3, 17), STEEL_XD, o=OUT)           # knee
    R(d, (cx - w_sh // 2, 16, cx + w_sh // 2, 25), STEEL, o=OUT)  # shin
    if armored:
        R(d, (cx - w_sh // 2 - 1, 17, cx - w_sh // 2 + 1, 24), STEEL_L,
          o=OUT)
        R(d, (cx - w_th // 2 - 1, 4, cx - w_th // 2 + 1, 11), STEEL_L,
          o=OUT)
    # foot
    fx0 = cx - 3 if side < 0 else cx - 1
    P(d, [(cx - 3, 25), (cx + 3, 25), (cx + 3 + side * 2, 28),
          (cx + 3 + side * 2, 30), (cx - 3, 30)], STEEL_D)
    E(d, (cx - 1, 6, cx + 1, 8), CYAN, o=None)


def f_exo_arm_armored_left(d, rng):
    _arm(d, -1, True)


def f_exo_arm_armored_right(d, rng):
    _arm(d, 1, True)


def f_exo_arm_left(d, rng):
    _arm(d, -1, False)


def f_exo_arm_right(d, rng):
    _arm(d, 1, False)


def f_exo_leg_armored_left(d, rng):
    _leg(d, -1, True)


def f_exo_leg_armored_right(d, rng):
    _leg(d, 1, True)


def f_exo_leg_left(d, rng):
    _leg(d, -1, False)


def f_exo_leg_right(d, rng):
    _leg(d, 1, False)


def f_exo_treads(d, rng):
    # two side-by-side triangular tread units
    for cx in (10, 22):
        P(d, [(cx - 6, 24), (cx, 10), (cx + 6, 24)], STEEL_D)
        P(d, [(cx - 6, 24), (cx, 10), (cx + 6, 24)], None, o=OUT)
        E(d, (cx - 5, 20, cx - 1, 25), STEEL_XD, o=STEEL_L)
        E(d, (cx + 1, 20, cx + 5, 25), STEEL_XD, o=STEEL_L)
        E(d, (cx - 2, 13, cx + 2, 17), STEEL_XD, o=STEEL_L)
        L(d, [(cx - 5, 24), (cx + 5, 24)], STEEL_XD, w=2)
    R(d, (13, 6, 19, 10), STEEL, o=OUT)   # hip mount
    L(d, [(16, 10), (10, 13)], STEEL_D)
    L(d, [(16, 10), (22, 13)], STEEL_D)


def f_cyberframe_1a(d, rng):
    # bulky frame: head merged into a solid metal torso core
    RR(d, (6, 5, 26, 28), 4, STEEL_D)
    RR(d, (9, 8, 23, 25), 3, STEEL)
    E(d, (13, 10, 19, 15), PANEL, o=STEEL_XD)      # face slit area
    L(d, [(14, 12), (18, 12)], CYAN)               # eye slit
    for yy in (18, 21, 24):
        L(d, [(11, yy), (21, yy)], STEEL_XD)
    E(d, (15, 16, 17, 18), RED, o=None)            # core light
    R(d, (4, 10, 6, 20), STEEL_XD, o=OUT)          # shoulder stubs
    R(d, (26, 10, 28, 20), STEEL_XD, o=OUT)


def f_cyberframe_1b(d, rng):
    # human-like robotic frame: distinct head + torso
    E(d, (12, 2, 20, 9), STEEL, o=OUT)             # head
    L(d, [(14, 5), (18, 5)], CYAN)                 # eyes
    R(d, (15, 9, 17, 11), STEEL_D, o=None)         # neck
    P(d, [(10, 11), (22, 11), (24, 16), (22, 27), (10, 27), (8, 16)],
      STEEL)                                       # torso
    P(d, [(10, 11), (22, 11), (24, 16), (22, 27), (10, 27), (8, 16)],
      None, o=OUT)
    RR(d, (13, 14, 19, 20), 2, PANEL)              # chest plate
    E(d, (15, 16, 17, 18), CYAN, o=None)
    L(d, [(11, 23), (21, 23)], STEEL_D)
    L(d, [(8, 13), (5, 20)], STEEL_D, w=2)         # arm stubs
    L(d, [(24, 13), (27, 20)], STEEL_D, w=2)


# ---------------------------------------------------------------- others

def f_eye_enhancer(d, rng):
    # eye with amplifier arcs
    P(d, [(5, 16), (16, 7), (27, 16), (16, 25)], WHITE)
    P(d, [(5, 16), (16, 7), (27, 16), (16, 25)], None, o=OUT)
    E(d, (11, 11, 21, 21), BLUE, o=OUT)
    E(d, (14, 14, 18, 18), STEEL_XD, o=None)
    E(d, (15, 12, 17, 14), WHITE, o=None)
    for r in (12, 15):
        d.arc((16 - r, 16 - r, 16 + r, 16 + r), start=300, end=60,
              fill=CYAN, width=1)


def f_eye_optic(d, rng):
    # eye with telescopic lens stack
    P(d, [(3, 16), (13, 9), (13, 23)], WHITE)
    P(d, [(3, 16), (13, 9), (13, 23)], None, o=OUT)
    for i, (x, r) in enumerate([(15, 6), (20, 5), (24, 4)]):
        E(d, (x - r, 16 - r, x + r, 16 + r),
          CYAN_PALE if i % 2 == 0 else CYAN, o=OUT)
    E(d, (24, 14, 27, 17), STEEL_XD, o=None)
    L(d, [(16, 11), (18, 13)], WHITE)


def f_face_mask(d, rng):
    # smooth face plate with subtle shifting contours
    E(d, (8, 3, 24, 25), SKIN, o=OUT)
    d.arc((10, 8, 22, 18), start=20, end=160, fill=STEEL_D, width=1)
    E(d, (12, 11, 14, 13), STEEL_XD, o=None)
    E(d, (18, 11, 20, 13), STEEL_XD, o=None)
    d.arc((12, 16, 20, 23), start=10, end=170, fill=RED_D, width=1)
    d.arc((9, 5, 23, 27), start=300, end=40, fill=VIOLET, width=1)
    d.arc((9, 5, 23, 27), start=140, end=220, fill=CYAN, width=1)


def f_faraday(d, rng):
    canister(d, CYAN)
    # network of miniature capacitors
    for (x, y) in [(12, 13), (17, 13), (12, 18), (17, 18)]:
        R(d, (x, y, x + 3, y + 4), STEEL_L, o=OUT)
        L(d, [(x + 1, y - 1), (x + 1, y)], STEEL_L)
    L(d, [(13, 15), (18, 15)], CYAN_PALE)
    L(d, [(13, 20), (18, 20)], CYAN_PALE)
    bolt(d, 21, 12, 2, col=CYAN_PALE)


def f_faulty_em_leak(d, rng):
    canister(d, RED)
    _cracked(d, rng)
    for r in (6, 9, 12):
        d.arc((16 - r, 16 - r, 16 + r, 16 + r), start=200, end=320,
              fill=RED, width=1)
        d.arc((16 - r, 16 - r, 16 + r, 16 + r), start=20, end=140,
              fill=RED, width=1)
    E(d, (14, 22, 18, 25), RED_D, o=None)


def f_faulty_grossfood(d, rng):
    canister(d, GREEN)
    # tongue with bitter fumes
    P(d, [(13, 14), (19, 14), (20, 20), (16, 23), (12, 20)], PINK)
    L(d, [(16, 15), (16, 21)], RED_D)
    for x in (12, 16, 20):
        d.arc((x - 2, 6, x + 2, 12), start=90, end=270, fill=ACID,
              width=1)


def f_fingerhack(d, rng):
    # finger with glowing circuit traces and hack prongs at the tip
    RR(d, (12, 10, 20, 29), 4, SKIN)
    d.arc((12, 4, 20, 12), start=180, end=360, fill=SKIN, width=1)
    R(d, (12, 6, 20, 11), SKIN, o=None)
    L(d, [(13, 6), (13, 10)], OUT)
    L(d, [(19, 6), (19, 10)], OUT)
    L(d, [(16, 12), (16, 20)], CYAN)
    L(d, [(16, 16), (13, 18)], CYAN)
    L(d, [(16, 18), (19, 20)], CYAN)
    E(d, (15, 19, 17, 21), CYAN, o=None)
    L(d, [(14, 4), (14, 1)], STEEL_L, w=1)
    L(d, [(18, 4), (18, 1)], STEEL_L, w=1)
    for yy in (23, 26):
        d.arc((12, yy - 3, 20, yy + 3), start=0, end=180, fill=STEEL_D,
              width=1)


DRAW = {
    "bio_adrenaline": f_adrenaline,
    "bio_ads": f_ads,
    "bio_alarm": f_alarm,
    "bio_ankles": f_ankles,
    "bio_ar": f_ar,
    "bio_armor_arms": f_armor_arms,
    "bio_armor_eyes": f_armor_eyes,
    "bio_armor_head": f_armor_head,
    "bio_armor_legs": f_armor_legs,
    "bio_armor_torso": f_armor_torso,
    "bio_batteries": f_batteries,
    "bio_blade": f_blade,
    "bio_blood_anal": f_blood_anal,
    "bio_blood_filter": f_blood_filter,
    "bio_cable": f_cable,
    "bio_carbon": f_carbon,
    "bio_chain_lightning": f_chain_lightning,
    "bio_claws": f_claws,
    "bio_climate": f_climate,
    "bio_cloak": f_cloak,
    "bio_cqb": f_cqb,
    "bio_creeyncaggii": f_creeyncaggii,
    "bio_deformity": f_deformity,
    "bio_dex_enhancer": f_dex_enhancer,
    "bio_digestion": f_digestion,
    "bio_dis_acid": f_dis_acid,
    "bio_dis_shock": f_dis_shock,
    "bio_drain": f_drain,
    "bio_ears": f_ears,
    "bio_electrokit": f_electrokit,
    "bio_emp": f_emp,
    "bio_emp_armgun": f_emp_armgun,
    "bio_ethanol": f_ethanol,
    "bio_evap": f_evap,
    "bio_exodii_arm_bionic_basic_armored_left": f_exo_arm_armored_left,
    "bio_exodii_arm_bionic_basic_armored_right": f_exo_arm_armored_right,
    "bio_exodii_arm_bionic_basic_left": f_exo_arm_left,
    "bio_exodii_arm_bionic_basic_right": f_exo_arm_right,
    "bio_exodii_cyberframe_1a": f_cyberframe_1a,
    "bio_exodii_cyberframe_1b": f_cyberframe_1b,
    "bio_exodii_leg_bionic_basic_armored_left": f_exo_leg_armored_left,
    "bio_exodii_leg_bionic_basic_armored_right": f_exo_leg_armored_right,
    "bio_exodii_leg_bionic_basic_left": f_exo_leg_left,
    "bio_exodii_leg_bionic_basic_right": f_exo_leg_right,
    "bio_exodii_leg_bionic_treads": f_exo_treads,
    "bio_eye_enhancer": f_eye_enhancer,
    "bio_eye_optic": f_eye_optic,
    "bio_face_mask": f_face_mask,
    "bio_faraday": f_faraday,
    "bio_faulty_electromagnetic_leak": f_faulty_em_leak,
    "bio_faulty_grossfood": f_faulty_grossfood,
    "bio_fingerhack": f_fingerhack,
}

IDS = [
    "bio_adrenaline", "bio_ads", "bio_alarm", "bio_ankles", "bio_ar",
    "bio_armor_arms", "bio_armor_eyes", "bio_armor_head", "bio_armor_legs",
    "bio_armor_torso", "bio_batteries", "bio_blade", "bio_blood_anal",
    "bio_blood_filter", "bio_cable", "bio_carbon", "bio_chain_lightning",
    "bio_claws", "bio_climate", "bio_cloak", "bio_cqb", "bio_creeyncaggii",
    "bio_deformity", "bio_dex_enhancer", "bio_digestion", "bio_dis_acid",
    "bio_dis_shock", "bio_drain", "bio_ears", "bio_electrokit", "bio_emp",
    "bio_emp_armgun", "bio_ethanol", "bio_evap",
    "bio_exodii_arm_bionic_basic_armored_left",
    "bio_exodii_arm_bionic_basic_armored_right",
    "bio_exodii_arm_bionic_basic_left",
    "bio_exodii_arm_bionic_basic_right",
    "bio_exodii_cyberframe_1a", "bio_exodii_cyberframe_1b",
    "bio_exodii_leg_bionic_basic_armored_left",
    "bio_exodii_leg_bionic_basic_armored_right",
    "bio_exodii_leg_bionic_basic_left",
    "bio_exodii_leg_bionic_basic_right",
    "bio_exodii_leg_bionic_treads",
    "bio_eye_enhancer", "bio_eye_optic", "bio_face_mask", "bio_faraday",
    "bio_faulty_electromagnetic_leak", "bio_faulty_grossfood",
    "bio_fingerhack",
]


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
                       "preview_item01.png")
    sheet.save(out)
    print("wrote preview_item01.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
