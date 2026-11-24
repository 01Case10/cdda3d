#!/usr/bin/env python3
"""Phase 2g.4 scope extension, batch ITEM03: 46 bionic (CBM) item tiles.

Drawn from each bionic's in-game name/description
(fork-src/cdda/data/json/bionics.json), flat UltiCa style, native 32x32,
bodies fully opaque on transparent background. Follows batch ITEM01's
module language (self-contained copy of its palette/helpers): internal
bionics are compact metallic implant modules (canister housing, accent
band, dark inset panel) with a function pictogram; deployed or worn
systems (razors, shotgun arm, XM interfaces, lungs, watch) are drawn as
the body-part units they are. Faulty bionics are cracked and sparking.
Deterministic: per-sprite seeded random.Random (crc32 of id).
Only creates/overwrites its own folders + preview_item03.png.
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
BONE = (226, 220, 204, 255)


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


def drop(d, cx, cy, s, col):
    P(d, [(cx, cy - s), (cx + s - 1, cy + 2), (cx - s + 1, cy + 2)], col)
    E(d, (cx - s + 1, cy, cx + s - 1, cy + s), col, o=None)


def cracked(d, rng):
    for _ in range(3):
        x = rng.randint(10, 20)
        L(d, [(x, 12), (x + 2, 16), (x, 19), (x + 1, 23)], STEEL_XD)


def trefoil(d, cx, cy, r, col):
    """Radiation trefoil: center disc + three 60-degree blades."""
    for a in (-90, 30, 150):
        x = cx + int((r * 0.62) * math.cos(math.radians(a)))
        y = cy + int((r * 0.62) * math.sin(math.radians(a)))
        d.pieslice((x - r // 2 - 1, y - r // 2 - 1, x + r // 2 + 1,
                    y + r // 2 + 1), start=a - 30, end=a + 30, fill=col)
    E(d, (cx - 1, cy - 1, cx + 1, cy + 1), col, o=None)


def zzz(d, x, y, col, s=3):
    """One blocky Z glyph with top-left corner at (x, y)."""
    L(d, [(x, y), (x + s, y)], col)
    L(d, [(x + s, y), (x, y + s)], col)
    L(d, [(x, y + s), (x + s, y + s)], col)


def hand_base(d, x0=9, y0=12, x1=23, y1=28, fill=SKIN):
    """Open hand: palm block + four finger bumps + thumb."""
    RR(d, (x0, y0 + 4, x1, y1), 3, fill)
    for i in range(4):
        fx = x0 + 1 + i * 3
        RR(d, (fx, y0, fx + 2, y0 + 6), 1, fill)
    P(d, [(x1, y0 + 10), (x1 + 4, y0 + 13), (x1, y0 + 16)], fill)


# ---------------------------------------------------------------- modules

def f_radleak(d, rng):
    canister(d, ACID)
    cracked(d, rng)
    trefoil(d, 16, 17, 6, ACID)
    drop(d, 12, 24, 2, ACID)
    drop(d, 20, 25, 1, ACID)


def f_radscrubber(d, rng):
    canister(d, CYAN)
    trefoil(d, 13, 15, 4, YELLOW)
    P(d, [(17, 12), (21, 12), (19, 16), (19, 18), (18, 18), (18, 16)],
      STEEL_L)
    drop(d, 19, 21, 2, CYAN)
    L(d, [(11, 23), (15, 23)], CYAN)


def f_razors(d, rng):
    hand_base(d)
    # small razor blades extending past each fingertip
    for i in range(4):
        fx = 10 + i * 3
        P(d, [(fx, 12), (fx + 1, 7), (fx + 2, 12)], STEEL_L)
        L(d, [(fx + 1, 8), (fx + 1, 11)], WHITE)
    L(d, [(12, 22), (20, 22)], STEEL_D)


def f_recoil(d, rng):
    # pair of wrist/elbow shock absorbers: joint blocks + coil springs
    for cx in (11, 21):
        RR(d, (cx - 3, 5, cx + 3, 11), 2, STEEL_D)
        for yy in (13, 16, 19):
            d.arc((cx - 3, yy - 2, cx + 3, yy + 2), start=0, end=180,
                  fill=STEEL_L, width=1)
            d.arc((cx - 3, yy - 2, cx + 3, yy + 2), start=180, end=360,
                  fill=STEEL, width=1)
        RR(d, (cx - 3, 21, cx + 3, 27), 2, STEEL_D)
        E(d, (cx - 1, 7, cx + 1, 9), CYAN, o=None)
    L(d, [(14, 16), (18, 16)], STEEL_XD)


def f_recycler(d, rng):
    canister(d, GREEN)
    # chasing-arrows recycle glyph
    for a in (270, 30, 150):
        d.arc((11, 12, 21, 22), start=a, end=a + 80, fill=GREEN, width=2)
    for (x, y, dx, dy) in [(16, 11, 2, 1), (21, 19, -1, 2), (11, 19, -1, -2)]:
        P(d, [(x, y), (x + dx * 2, y + dy), (x, y + dy * 2)], GREEN)
    drop(d, 16, 24, 1, CYAN)


def f_remote(d, rng):
    canister(d, BLUE)
    L(d, [(16, 12), (16, 8)], STEEL_L)
    E(d, (15, 6, 17, 8), RED, o=None)
    for r in (4, 7):
        d.arc((16 - r, 14 - r, 16 + r, 14 + r), start=290, end=70,
              fill=CYAN_PALE, width=1)
    R(d, (13, 17, 19, 22), CYAN, o=STEEL_L)
    L(d, [(14, 19), (18, 19)], PANEL)


def f_resonator(d, rng):
    canister(d, VIOLET)
    E(d, (14, 15, 18, 19), VIOLET, o=None)
    for r in (4, 7, 10):
        d.arc((16 - r, 17 - r, 16 + r, 17 + r), start=300, end=60,
              fill=VIOLET, width=1)
        d.arc((16 - r, 17 - r, 16 + r, 17 + r), start=120, end=240,
              fill=VIOLET, width=1)
    L(d, [(11, 24), (21, 24)], STEEL_XD)


def f_robofac_xmarm(d, rng):
    # forearm interface unit: steel forearm with gold contact collar
    RR(d, (11, 6, 21, 26), 3, STEEL)
    R(d, (11, 9, 21, 13), BRASS_D, o=OUT)
    for x in (13, 16, 19):
        E(d, (x - 1, 10, x + 1, 12), BRASS, o=None)
    RR(d, (13, 16, 19, 23), 2, PANEL)
    E(d, (15, 18, 17, 20), CYAN, o=None)
    L(d, [(12, 24), (20, 24)], STEEL_D)
    R(d, (9, 4, 23, 6), STEEL_D, o=OUT)      # mounting collar


def f_robofac_xmback(d, rng):
    # back plate interface: wide plate, shoulder straps, contact ports
    RR(d, (6, 7, 26, 27), 3, STEEL)
    L(d, [(8, 7), (8, 3)], STEEL_D, w=2)
    L(d, [(24, 7), (24, 3)], STEEL_D, w=2)
    R(d, (6, 12, 26, 14), BRASS_D, o=OUT)
    for x in (10, 16, 22):
        E(d, (x - 1, 12, x + 1, 14), BRASS, o=None)
    RR(d, (11, 17, 21, 24), 2, PANEL)
    E(d, (15, 19, 17, 21), CYAN, o=None)
    L(d, [(9, 25), (23, 25)], STEEL_D)


def f_robofac_xmleg(d, rng):
    # shin interface unit with contact collar at the knee
    RR(d, (11, 8, 21, 27), 3, STEEL)
    R(d, (9, 4, 23, 8), STEEL_D, o=OUT)      # knee collar
    R(d, (11, 11, 21, 15), BRASS_D, o=OUT)
    for x in (13, 16, 19):
        E(d, (x - 1, 12, x + 1, 14), BRASS, o=None)
    RR(d, (13, 18, 19, 24), 2, PANEL)
    E(d, (15, 20, 17, 22), CYAN, o=None)
    P(d, [(11, 27), (21, 27), (23, 30), (11, 30)], STEEL_D)  # foot plate


def f_robofac_xmlens(d, rng):
    # headgear interface: headband with central lens + contact pins
    d.arc((5, 6, 27, 24), start=180, end=360, fill=STEEL_D, width=4)
    E(d, (12, 8, 20, 16), CYAN_PALE, o=OUT)
    E(d, (14, 10, 18, 14), CYAN, o=None)
    L(d, [(13, 10), (16, 13)], WHITE)
    for x in (8, 24):
        L(d, [(x, 12), (x, 16)], BRASS, w=2)
        E(d, (x - 1, 16, x + 1, 18), BRASS, o=None)
    L(d, [(10, 22), (13, 18)], STEEL_D)
    L(d, [(22, 22), (19, 18)], STEEL_D)


def f_scent_mask(d, rng):
    canister(d, GREEN)
    # nose/mask glyph with fading scent wisps
    P(d, [(13, 13), (19, 13), (20, 18), (12, 18)], STEEL_L)
    L(d, [(13, 21), (19, 21)], STEEL_L)
    for i, x in enumerate((12, 16, 20)):
        col = GREEN if i == 0 else STEEL_XD
        d.arc((x - 2, 19, x + 2, 25), start=90, end=270, fill=col, width=1)


def f_shakes(d, rng):
    canister(d, RED)
    cracked(d, rng)
    # jitter: doubled offset zigzag waveform
    for off in (0, 1):
        pts = [(11 + off, 20), (13 + off, 15), (15 + off, 21),
               (17 + off, 14), (19 + off, 20), (21 + off, 16)]
        L(d, pts, RED if off == 0 else YELLOW)
    bolt(d, 21, 11, 2, col=CYAN_PALE)


def f_shock(d, rng):
    canister(d, YELLOW)
    bolt(d, 16, 17, 6)
    bolt(d, 11, 21, 2, col=CYAN_PALE)
    bolt(d, 21, 13, 2, col=CYAN_PALE)
    E(d, (19, 20, 22, 23), YELLOW, o=None)


def f_shockwave(d, rng):
    canister(d, ORANGE)
    E(d, (14, 15, 18, 19), ORANGE, o=None)
    E(d, (13, 14, 19, 20), None, o=ORANGE)
    for r in (6, 9):
        d.arc((16 - r, 17 - r, 16 + r, 17 + r), start=0, end=360,
              fill=ORANGE, width=1)
    for (x, y) in [(9, 10), (23, 10), (9, 24), (23, 24)]:
        L(d, [(x, y), (x + (1 if x < 16 else -1) * 2, y)], WHITE)


def f_shotgun(d, rng):
    # forearm housing, horizontal, with 12-gauge muzzle at the wrist end
    RR(d, (3, 12, 24, 22), 3, STEEL_D)
    R(d, (5, 14, 16, 17), STEEL, o=OUT)
    E(d, (21, 13, 29, 21), STEEL_XD, o=OUT)          # muzzle ring
    E(d, (23, 15, 27, 19), PANEL, o=STEEL_L)         # bore
    R(d, (6, 18, 10, 20), RED_D, o=OUT)              # shell hint
    L(d, [(12, 19), (18, 19)], STEEL_XD)
    E(d, (7, 15, 9, 17), CYAN, o=None)


def f_sleep_shutdown(d, rng):
    canister(d, BLUE)
    d.arc((12, 12, 19, 19), start=90, end=300, fill=CYAN_PALE, width=2)
    zzz(d, 18, 13, CYAN_PALE, 3)
    zzz(d, 21, 11, CYAN, 2)
    E(d, (15, 20, 17, 22), BLUE, o=None)


def f_sleepy(d, rng):
    canister(d, VIOLET)
    # droopy eyelid over a half-closed eye + a drip of enervator fluid
    d.arc((11, 13, 21, 21), start=0, end=180, fill=WHITE, width=2)
    L(d, [(11, 17), (21, 17)], WHITE)
    L(d, [(12, 15), (20, 15)], VIOLET)
    drop(d, 16, 21, 2, VIOLET)
    zzz(d, 20, 10, VIOLET, 2)
    L(d, [(11, 24), (13, 25)], STEEL_XD)


def f_sonar(d, rng):
    canister(d, CYAN)
    d.arc((12, 12, 20, 22), start=270, end=120, fill=SKIN, width=3)
    for r in (4, 7, 10):
        d.arc((17 - r, 17 - r, 17 + r, 17 + r), start=290, end=70,
              fill=CYAN_PALE, width=1)
    d.arc((10, 23, 22, 28), start=180, end=360, fill=BLUE, width=1)


def f_soporific(d, rng):
    canister(d, PINK)
    # brain glyph with an electrode dot
    E(d, (11, 12, 17, 17), PINK, o=None)
    E(d, (15, 12, 21, 17), PINK, o=None)
    E(d, (12, 15, 20, 20), PINK, o=None)
    L(d, [(16, 13), (16, 19)], RED_D)
    E(d, (15, 14, 17, 16), STEEL_L, o=OUT)
    zzz(d, 19, 20, CYAN_PALE, 2)


def f_spasm(d, rng):
    canister(d, RED)
    cracked(d, rng)
    # solenoid cylinder with crossed wires
    R(d, (13, 12, 19, 17), BRASS_D, o=OUT)
    L(d, [(13, 12), (19, 17)], RED)
    L(d, [(19, 12), (13, 17)], RED)
    pts = [(11, 22), (14, 19), (16, 23), (18, 19), (21, 22)]
    L(d, pts, YELLOW)
    bolt(d, 12, 10, 2)


def f_speed(d, rng):
    canister(d, CYAN)
    bolt(d, 17, 16, 5)
    for yy in (13, 17, 21):
        L(d, [(10, yy), (13, yy)], CYAN_PALE)
    E(d, (12, 12, 14, 14), CYAN, o=None)
    E(d, (20, 20, 22, 22), CYAN, o=None)
    L(d, [(13, 13), (20, 21)], CYAN)


def f_stiff(d, rng):
    canister(d, STEEL_D)
    # rigid stick figure locked straight by taut wires
    E(d, (15, 12, 17, 14), WHITE, o=None)
    L(d, [(16, 14), (16, 20)], WHITE)
    L(d, [(16, 16), (12, 16)], WHITE)
    L(d, [(16, 16), (20, 16)], WHITE)
    L(d, [(16, 20), (13, 24)], WHITE)
    L(d, [(16, 20), (19, 24)], WHITE)
    L(d, [(12, 13), (20, 13)], RED_D)
    L(d, [(11, 24), (21, 24)], RED_D)


def f_str_enhancer(d, rng):
    # upper arm with bulging bicep threaded with myomer fibers
    d.arc((6, 6, 22, 22), start=180, end=330, fill=SKIN, width=7)
    E(d, (10, 8, 22, 20), SKIN, o=OUT)
    for a in (200, 240, 280, 320):
        x0 = 16 + int(6 * math.cos(math.radians(a)))
        y0 = 14 + int(6 * math.sin(math.radians(a)))
        L(d, [(x0, y0), (16, 16)], RED_D)
    L(d, [(9, 20), (8, 27)], SKIN, w=4)
    RR(d, (5, 25, 12, 29), 1, STEEL_D)      # forearm cuff
    E(d, (13, 11, 16, 14), STEEL_L, o=None)


def f_sunglasses(d, rng):
    # dark transition lenses, half in sun glare
    RR(d, (3, 11, 29, 21), 5, STEEL_XD)
    E(d, (5, 12, 15, 20), STEEL_XD, o=OUT)
    E(d, (17, 12, 27, 20), STEEL_XD, o=OUT)
    E(d, (6, 13, 14, 19), (70, 60, 90, 255), o=None)
    E(d, (18, 13, 26, 19), (70, 60, 90, 255), o=None)
    L(d, [(7, 14), (11, 18)], CYAN_PALE)
    E(d, (24, 5, 28, 9), YELLOW, o=None)
    for (x, y) in [(26, 3), (29, 7), (22, 4)]:
        L(d, [(x, y), (26, 6)], YELLOW)


def f_surgical_razor(d, rng):
    # forearm housing with two scalpel blades extending past the wrist
    RR(d, (5, 17, 20, 27), 2, STEEL_D)
    R(d, (6, 19, 19, 21), STEEL, o=OUT)
    for x in (10, 16):
        P(d, [(x - 2, 18), (x, 5), (x + 3, 18)], STEEL_L)
        L(d, [(x, 6), (x + 1, 17)], WHITE)
        E(d, (x - 1, 15, x + 1, 17), STEEL_XD, o=None)
    L(d, [(7, 24), (18, 24)], CYAN)


def f_synaptic_regen(d, rng):
    # stimulator: head node + spine column with emitter nodes
    E(d, (12, 3, 20, 10), SKIN, o=OUT)
    E(d, (14, 5, 18, 8), STEEL_D, o=None)   # implant cap on back of head
    for i, yy in enumerate((12, 16, 20, 24)):
        E(d, (14, yy, 18, yy + 3), STEEL, o=OUT)
        E(d, (15, yy + 1, 17, yy + 2), CYAN, o=None)
        if i < 3:
            L(d, [(16, yy + 3), (16, yy + 4)], STEEL_D)
    for r in (5, 8):
        d.arc((16 - r, 6 - r + 3, 16 + r, 6 + r + 3), start=300, end=60,
              fill=CYAN_PALE, width=1)


def f_synlungs(d, rng):
    # pair of synthetic lungs with bronchi tree
    L(d, [(16, 4), (16, 10)], STEEL_L, w=2)
    L(d, [(16, 10), (12, 13)], STEEL_L)
    L(d, [(16, 10), (20, 13)], STEEL_L)
    for (x0, flip) in [(6, 1), (18, -1)]:
        P(d, [(x0 + 5, 11), (x0 + 10, 14), (x0 + 9, 24), (x0 + 4, 27),
              (x0, 24), (x0 + 1, 15)], PINK)
        P(d, [(x0 + 5, 11), (x0 + 10, 14), (x0 + 9, 24), (x0 + 4, 27),
              (x0, 24), (x0 + 1, 15)], None, o=OUT)
        for yy in (16, 20, 24):
            L(d, [(x0 + 2, yy), (x0 + 8, yy - 1)], RED_D)
    E(d, (14, 25, 18, 28), STEEL_D, o=OUT)


def f_syringe(d, rng):
    # middle finger with retractable needle at the tip, artery tube inside
    RR(d, (13, 9, 19, 29), 3, SKIN)
    d.arc((13, 5, 19, 11), start=180, end=360, fill=SKIN, width=1)
    R(d, (13, 7, 19, 10), SKIN, o=None)
    L(d, [(14, 7), (14, 10)], OUT)
    L(d, [(18, 7), (18, 10)], OUT)
    L(d, [(16, 11), (16, 26)], RED)
    L(d, [(16, 18), (14, 21)], RED)
    P(d, [(15, 5), (16, 1), (17, 5)], STEEL_L)   # needle
    L(d, [(16, 2), (16, 5)], WHITE)


def f_targeting(d, rng):
    # eye inside a targeting reticle with range ticks
    P(d, [(4, 16), (16, 8), (28, 16), (16, 24)], WHITE)
    P(d, [(4, 16), (16, 8), (28, 16), (16, 24)], None, o=OUT)
    E(d, (11, 11, 21, 21), BLUE, o=OUT)
    E(d, (14, 14, 18, 18), STEEL_XD, o=None)
    E(d, (12, 5, 20, 27), None, o=RED)
    L(d, [(16, 3), (16, 8)], RED)
    L(d, [(16, 24), (16, 29)], RED)
    L(d, [(3, 16), (8, 16)], RED)
    L(d, [(24, 16), (29, 16)], RED)
    for x in (10, 22):
        L(d, [(x, 26), (x, 28)], RED)


def f_taser(d, rng):
    hand_base(d, x0=10, y0=13, x1=22, y1=28)
    # twin prongs at the palm edge + arc between them
    L(d, [(12, 13), (12, 8)], STEEL_L, w=2)
    L(d, [(20, 13), (20, 8)], STEEL_L, w=2)
    bolt(d, 16, 10, 3)
    d.arc((11, 6, 21, 13), start=200, end=340, fill=CYAN_PALE, width=1)


def f_taste_blocker(d, rng):
    canister(d, PINK)
    # tongue with sensor dots, taste nullified by a bold X
    P(d, [(12, 13), (20, 13), (21, 19), (16, 23), (11, 19)], PINK)
    for (x, y) in [(14, 16), (18, 16), (16, 19)]:
        E(d, (x, y, x + 1, y + 1), STEEL_L, o=None)
    L(d, [(12, 13), (20, 22)], STEEL_XD, w=2)
    L(d, [(20, 13), (12, 22)], STEEL_XD, w=2)


def f_tattoo_led(d, rng):
    # skin patch with a glowing LED dot-matrix heart
    RR(d, (5, 6, 27, 26), 4, SKIN)
    pts = [(12, 12), (15, 12), (17, 12), (20, 12), (11, 15), (13, 15),
           (16, 15), (19, 15), (21, 15), (13, 18), (16, 18), (19, 18),
           (14, 21), (18, 21), (16, 23)]
    for (x, y) in pts:
        E(d, (x - 1, y - 1, x + 1, y + 1), CYAN, o=None)
    for (x, y) in [(8, 9), (24, 9), (8, 23), (24, 23)]:
        E(d, (x, y, x + 1, y + 1), CYAN_PALE, o=None)


def f_teleport(d, rng):
    canister(d, VIOLET)
    # folded-space swirl between two points
    d.arc((11, 12, 21, 22), start=0, end=270, fill=VIOLET, width=2)
    d.arc((13, 14, 19, 20), start=180, end=450, fill=CYAN_PALE, width=1)
    E(d, (11, 12, 14, 15), CYAN, o=None)
    E(d, (18, 19, 21, 22), VIOLET, o=WHITE)
    L(d, [(12, 24), (20, 24)], STEEL_XD)


def f_thumbs(d, rng):
    # two thumbs locked by steel bands
    for x in (9, 19):
        RR(d, (x, 10, x + 5, 24), 2, SKIN)
        d.arc((x, 6, x + 5, 12), start=180, end=360, fill=SKIN, width=1)
        R(d, (x - 1, 15, x + 6, 18), STEEL_D, o=OUT)   # locking band
        E(d, (x + 2, 16, x + 3, 17), STEEL_XD, o=None)
        L(d, [(x + 1, 22), (x + 4, 22)], STEEL_D)
    L(d, [(13, 16), (19, 16)], STEEL_XD)


def f_time_freeze(d, rng):
    canister(d, CYAN)
    # clock face with stopped hands + frost crystals
    E(d, (10, 11, 22, 23), WHITE, o=OUT)
    L(d, [(16, 17), (16, 13)], STEEL_XD)
    L(d, [(16, 17), (20, 17)], STEEL_XD)
    for (x, y) in [(11, 12), (21, 12), (11, 22), (21, 22)]:
        E(d, (x, y, x + 1, y + 1), CYAN, o=None)
    for (x, y, dx, dy) in [(9, 10, -1, -1), (23, 10, 1, -1),
                           (9, 24, -1, 1), (23, 24, 1, 1)]:
        L(d, [(x, y), (x + dx, y + dy)], CYAN_PALE)
        L(d, [(x, y), (x + dx, y)], CYAN_PALE)
        L(d, [(x, y), (x, y + dy)], CYAN_PALE)


def f_tools(d, rng):
    # forearm with tool bits deployed from the wrist
    RR(d, (6, 18, 22, 28), 2, STEEL_D)
    R(d, (7, 20, 21, 23), STEEL, o=OUT)
    # screwdriver
    L(d, [(10, 18), (10, 10)], STEEL_L, w=2)
    P(d, [(9, 10), (11, 10), (10, 7)], STEEL_L)
    # wrench
    d.arc((16, 6, 22, 12), start=300, end=200, fill=STEEL_L, width=2)
    L(d, [(18, 11), (17, 18)], STEEL_L, w=2)
    # welder tip with spark
    L(d, [(13, 18), (14, 12)], BRASS, w=2)
    bolt(d, 14, 10, 2, col=ORANGE)
    E(d, (8, 24, 11, 27), STEEL_XD, o=None)


def f_torsionratchet(d, rng):
    canister(d, BRASS)
    # ratchet gear with pawl + circular arrow
    E(d, (11, 12, 21, 22), BRASS_D, o=OUT)
    for a in range(0, 360, 45):
        x = 16 + int(6 * math.cos(math.radians(a)))
        y = 17 + int(6 * math.sin(math.radians(a)))
        R(d, (x - 1, y - 1, x + 1, y + 1), BRASS_D, o=None)
    E(d, (14, 15, 18, 19), PANEL, o=BRASS)
    P(d, [(19, 12), (21, 14), (18, 15)], STEEL_L)
    d.arc((10, 11, 22, 23), start=300, end=40, fill=CYAN_PALE, width=1)


def f_trip(d, rng):
    canister(d, VIOLET)
    # eye glitching into pixels
    P(d, [(11, 14), (16, 12), (21, 14), (16, 20)], WHITE)
    E(d, (14, 14, 18, 18), VIOLET, o=None)
    for (x, y, c) in [(19, 12, CYAN), (21, 15, VIOLET), (18, 19, CYAN_PALE),
                      (11, 20, VIOLET), (22, 19, STEEL_L)]:
        R(d, (x, y, x + 2, y + 2), c, o=None)
    L(d, [(11, 23), (14, 23)], STEEL_XD)


def f_uncanny_dodge(d, rng):
    canister(d, CYAN)
    # figure leaning away with motion-blur ghosts and a dodge arc
    for i, col in enumerate((STEEL_XD, CYAN, WHITE)):
        x = 13 + i * 2
        E(d, (x, 12, x + 2, 14), col, o=None)
        L(d, [(x + 1, 14), (x + 2, 19)], col)
        L(d, [(x + 2, 19), (x, 23)], col)
    d.arc((10, 10, 24, 25), start=250, end=340, fill=CYAN_PALE, width=1)
    L(d, [(21, 12), (23, 10)], CYAN_PALE)


def f_ups(d, rng):
    canister(d, YELLOW)
    # plug + socket with charge bars
    R(d, (12, 13, 16, 17), STEEL_L, o=OUT)
    L(d, [(13, 13), (13, 11)], STEEL_L)
    L(d, [(15, 13), (15, 11)], STEEL_L)
    L(d, [(14, 17), (14, 20)], STEEL_XD, w=2)
    for i in range(3):
        R(d, (18, 20 - i * 3, 20, 21 - i * 3), GREEN if i < 2 else YELLOW,
          o=None)
    E(d, (18, 12, 20, 14), YELLOW, o=None)


def f_voice(d, rng):
    canister(d, RED)
    # throat voice-box with a jagged robotic waveform
    RR(d, (12, 12, 20, 17), 2, STEEL_D)
    pts = [(10, 20), (12, 20), (13, 17), (14, 23), (15, 18), (16, 22),
           (17, 19), (18, 21), (20, 20), (22, 20)]
    L(d, pts, RED)
    for yy in (13, 15):
        L(d, [(14, yy), (18, yy)], STEEL_XD)


def f_voice_b(d, rng):
    canister(d, VIOLET)
    # AudioVox: speaker grille + smooth neon waveform
    E(d, (12, 12, 16, 16), STEEL_D, o=STEEL_L)
    E(d, (13, 13, 15, 15), PANEL, o=None)
    pts = [(10, 21), (12, 19), (14, 23), (16, 18), (18, 22), (20, 20),
           (22, 21)]
    L(d, pts, CYAN_PALE)
    L(d, [(18, 13), (21, 13)], VIOLET)
    E(d, (19, 15, 21, 17), VIOLET, o=None)


def f_watch(d, rng):
    # chronometer: round face on a wrist band, atomic orbit rings
    R(d, (12, 3, 20, 9), STEEL_D, o=OUT)
    R(d, (12, 23, 20, 29), STEEL_D, o=OUT)
    E(d, (8, 8, 24, 24), STEEL, o=OUT)
    E(d, (10, 10, 22, 22), WHITE, o=STEEL_XD)
    L(d, [(16, 16), (16, 12)], STEEL_XD)
    L(d, [(16, 16), (19, 17)], STEEL_XD)
    E(d, (15, 15, 17, 17), RED, o=None)
    d.arc((7, 12, 25, 20), start=0, end=360, fill=CYAN, width=1)
    E(d, (23, 15, 25, 17), CYAN, o=None)


def f_water_extractor(d, rng):
    hand_base(d)
    # nanotube pore dots in the palm + pumped droplet rising
    for (x, y) in [(13, 19), (16, 18), (19, 19), (14, 22), (18, 22),
                   (16, 21)]:
        E(d, (x, y, x + 1, y + 1), STEEL_XD, o=None)
    drop(d, 16, 8, 3, CYAN)
    L(d, [(12, 12), (14, 14)], CYAN)
    L(d, [(20, 12), (18, 14)], CYAN)


def f_weight(d, rng):
    # titanium bracing: spine strut + knee brace over bone-white frame
    L(d, [(16, 4), (16, 26)], BONE, w=3)
    for yy in (7, 12, 17, 22):
        R(d, (13, yy, 19, yy + 2), STEEL_L, o=OUT)
    # elbow/knee side braces
    for (x, yy) in [(8, 10), (24, 10), (8, 20), (24, 20)]:
        E(d, (x - 2, yy - 2, x + 2, yy + 2), STEEL_L, o=OUT)
        E(d, (x - 1, yy - 1, x + 1, yy + 1), STEEL_XD, o=None)
    L(d, [(10, 10), (13, 10)], STEEL_D, w=2)
    L(d, [(22, 10), (19, 10)], STEEL_D, w=2)
    L(d, [(10, 20), (13, 21)], STEEL_D, w=2)
    L(d, [(22, 20), (19, 21)], STEEL_D, w=2)


DRAW = {
    "bio_radleak": f_radleak,
    "bio_radscrubber": f_radscrubber,
    "bio_razors": f_razors,
    "bio_recoil": f_recoil,
    "bio_recycler": f_recycler,
    "bio_remote": f_remote,
    "bio_resonator": f_resonator,
    "bio_robofac_xmarm": f_robofac_xmarm,
    "bio_robofac_xmback": f_robofac_xmback,
    "bio_robofac_xmleg": f_robofac_xmleg,
    "bio_robofac_xmlens": f_robofac_xmlens,
    "bio_scent_mask": f_scent_mask,
    "bio_shakes": f_shakes,
    "bio_shock": f_shock,
    "bio_shockwave": f_shockwave,
    "bio_shotgun": f_shotgun,
    "bio_sleep_shutdown": f_sleep_shutdown,
    "bio_sleepy": f_sleepy,
    "bio_sonar": f_sonar,
    "bio_soporific": f_soporific,
    "bio_spasm": f_spasm,
    "bio_speed": f_speed,
    "bio_stiff": f_stiff,
    "bio_str_enhancer": f_str_enhancer,
    "bio_sunglasses": f_sunglasses,
    "bio_surgical_razor": f_surgical_razor,
    "bio_synaptic_regen": f_synaptic_regen,
    "bio_synlungs": f_synlungs,
    "bio_syringe": f_syringe,
    "bio_targeting": f_targeting,
    "bio_taser": f_taser,
    "bio_taste_blocker": f_taste_blocker,
    "bio_tattoo_led": f_tattoo_led,
    "bio_teleport": f_teleport,
    "bio_thumbs": f_thumbs,
    "bio_time_freeze": f_time_freeze,
    "bio_tools": f_tools,
    "bio_torsionratchet": f_torsionratchet,
    "bio_trip": f_trip,
    "bio_uncanny_dodge": f_uncanny_dodge,
    "bio_ups": f_ups,
    "bio_voice": f_voice,
    "bio_voice_b": f_voice_b,
    "bio_watch": f_watch,
    "bio_water_extractor": f_water_extractor,
    "bio_weight": f_weight,
}

IDS = [
    "bio_radleak", "bio_radscrubber", "bio_razors", "bio_recoil",
    "bio_recycler", "bio_remote", "bio_resonator", "bio_robofac_xmarm",
    "bio_robofac_xmback", "bio_robofac_xmleg", "bio_robofac_xmlens",
    "bio_scent_mask", "bio_shakes", "bio_shock", "bio_shockwave",
    "bio_shotgun", "bio_sleep_shutdown", "bio_sleepy", "bio_sonar",
    "bio_soporific", "bio_spasm", "bio_speed", "bio_stiff",
    "bio_str_enhancer", "bio_sunglasses", "bio_surgical_razor",
    "bio_synaptic_regen", "bio_synlungs", "bio_syringe", "bio_targeting",
    "bio_taser", "bio_taste_blocker", "bio_tattoo_led", "bio_teleport",
    "bio_thumbs", "bio_time_freeze", "bio_tools", "bio_torsionratchet",
    "bio_trip", "bio_uncanny_dodge", "bio_ups", "bio_voice", "bio_voice_b",
    "bio_watch", "bio_water_extractor", "bio_weight",
]


def main():
    assert len(IDS) == 46 and set(IDS) == set(DRAW), (
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
                       "preview_item03.png")
    sheet.save(out)
    print("wrote preview_item03.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
