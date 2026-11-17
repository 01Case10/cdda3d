#!/usr/bin/env python3
"""ITEM61: gun stocks + robot parts -- CDDA Voxel coverage audit.

Two clusters, each drawn as itself:

  Gun stocks (gunmod/stock.json, 20) -- all in profile off a
    receiver mounting block at the left, stock extending right to
    a buttplate:
      adjustable: telescoping rails + adjustment notch row;
      butt hook: aluminum buttplate with a hook curving down and
        forward (held under the armpit);
      cheek pad: raised padded cheek rest on the comb;
      folding / high-end folding: hinge disc at the mount, stock
        full-length when extended; the _folded variants are swung
        forward alongside the receiver -- half the length;
      m26_mass: adjustable stock with an integrated pistol grip;
      pistol: compact stock behind a handgun grip;
      recoil: thick layered recoil pad + absorber spring slot;
      scout: hollow polymer butt with a spare magazine stored in
        the bottom cavity;
      stock_mauser: the C96's hollow WOODEN holster-stock (brown,
        grain, dark hollow mouth, steel bracket);
      stock_none: a sawn-off -- bare steel tang with saw marks
        and a little paper tag, no stock at all;
      sub2000 mechanism: a bar of hinges/pivots; _folded is the
        same bar doubled back on itself, hinge at the far end;
      tail hook: handgun add-on arm ending in a pivoting forearm
        hook (aluminum);
      under-folding: twin wire rails + buttplate + push-button
        catch; _folded swings the wire frame under the receiver;
      wire: a collapsing wire loop, extended vs collapsed short.
  Robot parts (robot_parts.json, 20):
      modules -- circuit boards (green PCB, traces, gold edge
        fingers, black chips), told apart by what's on the board:
        AI cores by chip density/tier (basic 1 small chip,
        standard 2, advanced a big many-pinned processor), gun =
        gun pictogram, identification = viewfinder brackets + face
        dots, memory = banks of identical memory chips,
        pathfinding = dotted route with waypoints, targeting =
        crosshair reticle; sensor_module is a sensor-array panel
        (lenses, not a plain board) and self_monitoring a
        diagnostic panel with a pulse trace and LEDs.
      mechanical -- android arms / legs (jointed human-like
        limbs), android chassis (skeleton torso), copbot chassis
        (armored police-bot shell, blue stripe), turret chassis
        (boxy frame + empty turret ring), omni wheels (rollers
        around the rim), quad rotors (four rotor discs on an X),
        spidery legs big (frame-filling) vs small (skitterbot,
        centred and fine), tank tread (track loop + road wheels).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "adjustable_stock",
    "butt_hook",
    "cheek_pad",
    "folding_stock",
    "folding_stock_folded",
    "high_end_folding_stock",
    "high_end_folding_stock_folded",
    "m26_mass_stock",
    "pistol_stock",
    "recoil_stock",
    "scout_stock",
    "stock_mauser",
    "stock_none",
    "sub2000_folding_mechanism",
    "sub2000_folding_mechanism_folded",
    "tail_hook_stock",
    "under_folding_stock",
    "under_folding_stock_folded",
    "wire_stock",
    "wire_stock_folded",
    "ai_module",
    "ai_module_advanced",
    "ai_module_basic",
    "android_arms",
    "android_chassis",
    "android_legs",
    "copbot_chassis",
    "gun_module",
    "identification_module",
    "memory_module",
    "omni_wheel",
    "pathfinding_module",
    "quad_rotors",
    "self_monitoring_module",
    "sensor_module",
    "spidery_legs_big",
    "spidery_legs_small",
    "tank_tread",
    "targeting_module",
    "turret_chassis",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

POLY = (76, 78, 84, 255)          # dark-gray polymer
POLY_DARK = (50, 52, 58, 255)
POLY_LIGHT = (104, 106, 114, 255)
STEEL = (128, 132, 138, 255)
STEEL_DARK = (78, 82, 88, 255)
STEEL_LIGHT = (172, 176, 182, 255)  # robot steel / plastic shell
ALU = (168, 172, 178, 255)        # aluminum
ALU_DARK = (118, 122, 128, 255)
WOOD = (130, 90, 50, 255)
WOOD_DARK = (94, 62, 34, 255)
WOOD_LIGHT = (164, 120, 72, 255)
RUBBER = (34, 34, 38, 255)
ACCENT = (206, 120, 44, 255)      # high-end accent stripe
PCB = (34, 106, 54, 255)
PCB_DARK = (24, 78, 40, 255)
TRACE = (86, 178, 104, 255)
GOLD = (216, 182, 84, 255)
CHIP = (22, 22, 26, 255)
CHIP_TOP = (44, 44, 50, 255)
JOINT = (66, 70, 76, 255)
LENS = (52, 120, 190, 255)
RED = (206, 52, 44, 255)
GREEN_LED = (72, 200, 92, 255)
PAPER = (238, 232, 218, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ stocks
# Profile layout: receiver mount block x 2..6, stock runs right.

def mount(d, tall=False):
    d.rectangle([2, 12 if tall else 13, 6, 20 if tall else 19], fill=STEEL_DARK)
    d.rectangle([2, 12 if tall else 13, 6, 13 if tall else 14],
                fill=STEEL)
    d.point((4, 16), fill=STEEL_LIGHT)  # mount pin


def buttstock_body(d, x0=6, drop=3, pad=True, col=POLY):
    """Plain rifle buttstock: comb sloping down to a buttplate."""
    d.polygon([(x0, 13), (28, 13 + drop - 2), (28, 24), (14, 22),
               (x0, 19)], fill=col)
    d.line([x0, 13, 28, 13 + drop - 2], fill=shade(col, 1.25), width=1)
    d.polygon([(x0, 19), (14, 22), (28, 24), (28, 26), (12, 25),
               (x0, 21)], fill=shade(col, 0.8))  # belly shadow
    if pad:
        d.rectangle([28, 13 + drop - 2, 30, 25], fill=RUBBER)
        d.line([29, 13 + drop - 1, 29, 24], fill=POLY_DARK, width=1)


def hinge(d, cx=8, cy=15):
    d.ellipse([cx - 2, cy - 3, cx + 2, cy + 3], fill=STEEL)
    d.ellipse([cx - 2, cy - 3, cx + 2, cy + 3], outline=STEEL_DARK, width=1)
    d.point((cx, cy), fill=STEEL_DARK)


def draw_adjustable(d, rng):
    mount(d)
    # telescoping rails from the receiver to a separated butt piece
    d.rectangle([6, 14, 22, 16], fill=STEEL_DARK)
    d.rectangle([6, 18, 22, 19], fill=STEEL_DARK)
    for x in range(9, 22, 3):  # adjustment notches on the top rail
        d.point((x, 14), fill=STEEL_LIGHT)
    d.polygon([(22, 12), (29, 13), (29, 25), (22, 23)], fill=POLY)
    d.line([22, 12, 29, 13], fill=POLY_LIGHT, width=1)
    d.rectangle([29, 13, 30, 25], fill=RUBBER)  # buttpad
    d.rectangle([23, 17, 27, 19], outline=POLY_DARK, width=1)  # lever housing
    d.line([24, 20, 27, 22], fill=STEEL, width=1)  # locking lever


def draw_butt_hook(d, rng):
    mount(d)
    d.polygon([(6, 13), (24, 14), (24, 22), (12, 21), (6, 19)], fill=POLY)
    d.rectangle([24, 13, 27, 23], fill=ALU_DARK)  # buttplate
    # the hook: aluminum arm down from the butt, curving forward
    d.rectangle([25, 23, 26, 28], fill=ALU)
    d.rectangle([18, 27, 26, 28], fill=ALU)
    d.rectangle([18, 25, 19, 28], fill=ALU)  # hook tip rising
    d.line([25, 23, 25, 27], fill=shade(ALU, 1.2), width=1)
    d.point((25, 15), fill=STEEL_LIGHT)


def draw_cheek_pad(d, rng):
    mount(d)
    buttstock_body(d)
    # raised padded cheek rest sitting on the comb
    d.rectangle([13, 8, 26, 12], fill=shade(POLY, 1.35))
    d.rectangle([13, 8, 26, 12], outline=POLY_DARK, width=1)
    d.rectangle([14, 12, 25, 13], fill=POLY_DARK)  # riser posts shadow
    for x in (16, 20, 24):  # pad stitching
        d.point((x, 10), fill=POLY_DARK)


def draw_folding(d, rng, high_end=False):
    mount(d, tall=True)
    hinge(d)
    col = (58, 60, 66, 255) if high_end else POLY
    d.polygon([(10, 12), (28, 13), (28, 23), (16, 22), (10, 18)],
              fill=col)
    d.line([10, 12, 28, 13], fill=shade(col, 1.3), width=1)
    if high_end:  # multi-position rail + accent stripe + fat pad
        for x in range(13, 25, 3):
            d.point((x, 20), fill=STEEL_LIGHT)
        d.line([11, 15, 27, 16], fill=ACCENT, width=1)
        d.rectangle([28, 12, 30, 24], fill=RUBBER)
        d.rectangle([28, 12, 30, 24], outline=POLY_DARK, width=1)
    else:
        d.rectangle([28, 13, 30, 24], fill=RUBBER)


def draw_folding_folded(d, rng, high_end=False):
    # receiver stub runs right along the top; the stock is swung
    # forward alongside it, buttplate back at the mount -- compact.
    d.rectangle([2, 9, 24, 13], fill=STEEL_DARK)  # receiver
    d.line([2, 9, 24, 9], fill=STEEL, width=1)
    d.rectangle([20, 10, 24, 12], fill=STEEL_DARK)  # ejection detail
    hinge(d, cx=8, cy=16)
    col = (58, 60, 66, 255) if high_end else POLY
    d.polygon([(6, 17), (26, 16), (26, 23), (10, 23)], fill=col)
    d.line([6, 17, 26, 16], fill=shade(col, 1.3), width=1)
    d.rectangle([4, 16, 6, 24], fill=RUBBER)  # buttpad, at the mount end
    if high_end:
        d.line([8, 19, 25, 18], fill=ACCENT, width=1)
        for x in range(11, 24, 3):
            d.point((x, 21), fill=STEEL_LIGHT)


def draw_m26_mass(d, rng):
    mount(d, tall=True)
    # integrated pistol grip under the mount
    d.polygon([(7, 18), (12, 18), (11, 27), (7, 27)], fill=POLY_DARK)
    d.line([8, 20, 11, 20], fill=POLY, width=1)
    d.line([8, 23, 11, 23], fill=POLY, width=1)
    # adjustable butt piece on rails, like draw_adjustable
    d.rectangle([10, 13, 21, 15], fill=STEEL_DARK)
    for x in range(12, 21, 3):
        d.point((x, 13), fill=STEEL_LIGHT)
    d.polygon([(21, 11), (28, 12), (28, 23), (21, 21)], fill=POLY)
    d.line([21, 11, 28, 12], fill=POLY_LIGHT, width=1)
    d.rectangle([28, 12, 30, 23], fill=RUBBER)


def draw_pistol_stock(d, rng):
    # handgun at the left: slide stub + grip, compact stock behind
    d.rectangle([2, 11, 12, 14], fill=STEEL_DARK)  # pistol slide
    d.line([2, 11, 12, 11], fill=STEEL, width=1)
    d.polygon([(5, 14), (9, 14), (8, 22), (4, 22)], fill=POLY_DARK)  # grip
    d.rectangle([11, 13, 14, 17], fill=STEEL_DARK)  # stock mount
    d.polygon([(14, 13), (27, 14), (27, 21), (18, 20), (14, 17)],
              fill=POLY)
    d.line([14, 13, 27, 14], fill=POLY_LIGHT, width=1)
    d.rectangle([27, 14, 29, 21], fill=RUBBER)


def draw_recoil(d, rng):
    mount(d)
    buttstock_body(d, pad=False)
    # absorber spring visible through a slot in the body
    pts = []
    for i in range(7):
        pts.append((13 + i * 2, 16 if i % 2 == 0 else 19))
    d.rectangle([12, 15, 26, 20], fill=POLY_DARK)
    d.line(pts, fill=STEEL_LIGHT, width=1)
    # thick layered recoil pad
    d.rectangle([28, 11, 29, 25], fill=shade(RUBBER, 1.5))
    d.rectangle([29, 11, 30, 25], fill=RUBBER)
    d.line([29, 12, 29, 24], fill=POLY_DARK, width=1)


def draw_scout(d, rng):
    mount(d)
    buttstock_body(d)
    # hollow cavity in the belly with a spare magazine inside
    d.rectangle([13, 20, 26, 25], fill=(26, 26, 30, 255))
    d.rectangle([13, 20, 26, 25], outline=POLY_DARK, width=1)
    d.polygon([(17, 21), (22, 21), (21, 25), (16, 25)],
              fill=STEEL_DARK)  # stored magazine
    d.line([17, 22, 21, 22], fill=STEEL, width=1)


def draw_mauser(d, rng):
    # the hollow wooden holster-stock: bracket at the front (left),
    # big wooden body with a dark hollow mouth on top.
    d.rectangle([2, 13, 5, 19], fill=STEEL_DARK)  # steel bracket
    d.point((3, 16), fill=STEEL_LIGHT)
    d.polygon([(5, 11), (27, 12), (29, 15), (29, 23), (20, 25),
               (6, 21)], fill=WOOD)
    d.line([5, 11, 27, 12], fill=WOOD_LIGHT, width=1)
    d.rectangle([9, 13, 24, 16], fill=WOOD_DARK)  # hollow mouth
    d.rectangle([10, 14, 23, 15], fill=(40, 26, 14, 255))
    for x0, y0, x1, y1 in ((7, 18, 16, 19), (18, 20, 27, 21),
                           (8, 22, 14, 23)):  # grain
        d.line([x0, y0, x1, y1], fill=WOOD_DARK, width=1)
    d.rectangle([28, 14, 29, 23], fill=WOOD_DARK)  # butt cap


def draw_stock_none(d, rng):
    # sawn-off: a bare steel tang with a jagged saw-cut end, no
    # stock -- plus a little paper tag tied on with string.
    d.rectangle([6, 14, 15, 18], fill=STEEL_DARK)
    d.line([6, 14, 15, 14], fill=STEEL, width=1)
    for x in range(7, 15, 2):  # saw marks
        d.line([x, 15, x + 1, 17], fill=STEEL, width=1)
    d.polygon([(15, 14), (18, 15), (16, 16), (18, 17), (15, 18)],
              fill=STEEL)  # jagged cut face
    d.line([10, 18, 13, 23], fill=(180, 164, 120, 255), width=1)  # string
    d.rectangle([11, 23, 19, 28], fill=PAPER)  # tag
    d.rectangle([11, 23, 19, 28], outline=STEEL_DARK, width=1)
    d.point((13, 25), fill=RED)
    d.line([15, 26, 18, 26], fill=STEEL_DARK, width=1)


def draw_sub2000(d, rng):
    # a bar of hinges and pivots, locked open: three knuckles
    d.rectangle([3, 14, 29, 18], fill=POLY)
    d.line([3, 14, 29, 14], fill=POLY_LIGHT, width=1)
    for cx in (9, 16, 23):  # hinge knuckles + pivot pins
        d.rectangle([cx - 2, 13, cx + 2, 19], fill=STEEL_DARK)
        d.ellipse([cx - 1, 14, cx + 1, 17], fill=STEEL)
        d.point((cx, 15), fill=STEEL_LIGHT)
    d.rectangle([27, 12, 29, 20], fill=STEEL_DARK)  # latch block
    d.point((28, 16), fill=RED)


def draw_sub2000_folded(d, rng):
    # doubled back on itself: two parallel bars, hinge at the
    # right end joining them, latch locked at the left.
    d.rectangle([3, 11, 25, 15], fill=POLY)
    d.rectangle([3, 18, 25, 22], fill=shade(POLY, 0.85))
    d.line([3, 11, 25, 11], fill=POLY_LIGHT, width=1)
    d.rectangle([25, 11, 28, 22], fill=STEEL_DARK)  # the fold hinge
    d.ellipse([25, 15, 28, 18], fill=STEEL)
    d.point((26, 16), fill=STEEL_LIGHT)
    for cx in (10, 17):  # pivot pins on both bars
        d.point((cx, 13), fill=STEEL_LIGHT)
        d.point((cx, 20), fill=STEEL_LIGHT)
    d.rectangle([3, 15, 6, 18], fill=STEEL_DARK)  # locked latch
    d.point((4, 16), fill=RED)


def draw_tail_hook(d, rng):
    d.rectangle([2, 12, 9, 16], fill=STEEL_DARK)  # handgun/SMG mount
    d.line([2, 12, 9, 12], fill=STEEL, width=1)
    d.rectangle([9, 13, 24, 16], fill=ALU_DARK)  # arm
    d.line([9, 13, 24, 13], fill=ALU, width=1)
    # pivoting hook at the end: joint disc, hook curves down+back
    d.ellipse([22, 13, 27, 18], fill=ALU)
    d.point((24, 15), fill=STEEL_DARK)
    d.rectangle([24, 17, 25, 24], fill=ALU)
    d.rectangle([18, 23, 25, 24], fill=ALU)
    d.rectangle([18, 21, 19, 24], fill=ALU)  # hook tip


def wire_frame(d, x0, x1, y0, y1, col=STEEL):
    d.rectangle([x0, y0, x1, y0 + 1], outline=col, width=1)
    d.line([x0, y0, x1, y0], fill=col, width=1)
    d.line([x0, y1, x1, y1], fill=col, width=1)
    d.line([x0, y0, x0, y1], fill=col, width=1)
    d.line([x1, y0, x1, y1], fill=col, width=1)


def draw_under_folding(d, rng):
    mount(d, tall=True)
    d.ellipse([8, 10, 12, 14], fill=RED)  # push-button release catch
    d.point((10, 12), fill=(240, 200, 190, 255))
    # twin wire rails to a flat buttplate
    d.line([6, 14, 27, 14], fill=STEEL, width=1)
    d.line([6, 19, 27, 19], fill=STEEL, width=1)
    d.line([14, 14, 14, 19], fill=STEEL, width=1)  # cross brace
    d.rectangle([27, 11, 29, 22], fill=STEEL_DARK)  # buttplate
    d.line([27, 11, 27, 22], fill=STEEL_LIGHT, width=1)


def draw_under_folding_folded(d, rng):
    # receiver bar along the top; wire frame folded underneath it
    d.rectangle([2, 8, 29, 13], fill=STEEL_DARK)  # receiver/chassis
    d.line([2, 8, 29, 8], fill=STEEL, width=1)
    d.rectangle([24, 9, 28, 12], fill=STEEL_DARK)
    d.ellipse([5, 14, 9, 18], fill=RED)  # release catch
    # folded wire frame tucked under the receiver
    d.line([10, 16, 27, 16], fill=STEEL, width=1)
    d.line([10, 20, 27, 20], fill=STEEL, width=1)
    d.line([18, 16, 18, 20], fill=STEEL, width=1)
    d.rectangle([8, 15, 10, 21], fill=STEEL_DARK)  # buttplate, by the mount


def draw_wire(d, rng):
    mount(d)
    # extended wire loop: thin rectangle of wire to the buttplate
    d.line([6, 13, 27, 13], fill=STEEL, width=1)
    d.line([6, 20, 27, 20], fill=STEEL, width=1)
    d.line([13, 13, 13, 20], fill=STEEL, width=1)  # slider stop
    d.rectangle([27, 11, 29, 22], fill=STEEL_DARK)
    d.line([27, 11, 27, 22], fill=STEEL_LIGHT, width=1)
    d.point((9, 16), fill=STEEL_LIGHT)


def draw_wire_folded(d, rng):
    mount(d)
    # collapsed: wire loop slid home, buttplate close to receiver
    d.line([6, 13, 15, 13], fill=STEEL, width=1)
    d.line([6, 20, 15, 20], fill=STEEL, width=1)
    d.rectangle([15, 11, 17, 22], fill=STEEL_DARK)  # buttplate
    d.line([15, 11, 15, 22], fill=STEEL_LIGHT, width=1)
    # loose wire end poking past, and an empty dashed hint of the
    # extended position stays out -- keep it plain: a slide slot
    d.rectangle([8, 15, 13, 17], outline=STEEL_DARK, width=1)


STOCKS = {
    "adjustable_stock": draw_adjustable,
    "butt_hook": draw_butt_hook,
    "cheek_pad": draw_cheek_pad,
    "folding_stock": lambda d, r: draw_folding(d, r, False),
    "folding_stock_folded": lambda d, r: draw_folding_folded(d, r, False),
    "high_end_folding_stock": lambda d, r: draw_folding(d, r, True),
    "high_end_folding_stock_folded": lambda d, r: draw_folding_folded(
        d, r, True),
    "m26_mass_stock": draw_m26_mass,
    "pistol_stock": draw_pistol_stock,
    "recoil_stock": draw_recoil,
    "scout_stock": draw_scout,
    "stock_mauser": draw_mauser,
    "stock_none": draw_stock_none,
    "sub2000_folding_mechanism": draw_sub2000,
    "sub2000_folding_mechanism_folded": draw_sub2000_folded,
    "tail_hook_stock": draw_tail_hook,
    "under_folding_stock": draw_under_folding,
    "under_folding_stock_folded": draw_under_folding_folded,
    "wire_stock": draw_wire,
    "wire_stock_folded": draw_wire_folded,
}


# ------------------------------------------------------------------- robot

def board(d, rng, w=24, h=20, x0=4, y0=6):
    """A green PCB with traces and gold edge fingers."""
    d.rectangle([x0, y0, x0 + w, y0 + h], fill=PCB)
    d.rectangle([x0, y0, x0 + w, y0 + h], outline=PCB_DARK, width=1)
    for _ in range(14):  # traces
        x = rng.randint(x0 + 2, x0 + w - 5)
        y = rng.randint(y0 + 2, y0 + h - 3)
        ln = rng.randint(2, 5)
        col = TRACE if rng.random() < 0.7 else GOLD
        if rng.random() < 0.5:
            d.line([x, y, x + ln, y], fill=col, width=1)
        else:
            d.line([x, y, x, y + ln], fill=col, width=1)
        d.point((x, y), fill=GOLD)
    for fx in range(x0 + 2, x0 + w - 1, 3):  # edge fingers
        d.rectangle([fx, y0 + h - 2, fx + 1, y0 + h - 1], fill=GOLD)
    for cx, cy in ((x0 + 1, y0 + 1), (x0 + w - 1, y0 + 1)):  # screw holes
        d.point((cx, cy), fill=PCB_DARK)


def chip(d, cx, cy, w, h, pins=True):
    d.rectangle([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
                fill=CHIP)
    d.rectangle([cx - w // 2, cy - h // 2, cx + w // 2, cy - h // 2 + 1],
                fill=CHIP_TOP)
    if pins:
        for px in range(cx - w // 2, cx + w // 2 + 1, 2):
            d.point((px, cy - h // 2 - 1), fill=STEEL_LIGHT)
            d.point((px, cy + h // 2 + 1), fill=STEEL_LIGHT)
    d.point((cx, cy), fill=STEEL_LIGHT)


def draw_ai(d, rng, tier):
    board(d, rng)
    if tier == 0:  # basic: one small chip, sparse
        chip(d, 13, 13, 6, 5)
        d.point((22, 10), fill=GREEN_LED)
    elif tier == 1:  # standard AI core: two chips + core dot
        chip(d, 12, 12, 8, 6)
        chip(d, 22, 17, 5, 4)
        d.point((12, 12), fill=LENS)
        d.point((24, 9), fill=GREEN_LED)
    else:  # advanced: big many-pinned processor, dense satellites
        chip(d, 15, 14, 12, 10)
        for px in range(9, 22, 2):
            d.point((px, 8), fill=GOLD)
            d.point((px, 20), fill=GOLD)
        d.rectangle([13, 12, 17, 16], outline=TRACE, width=1)
        d.point((15, 14), fill=LENS)
        chip(d, 25, 9, 4, 3, pins=False)
        chip(d, 6, 21, 4, 3, pins=False)
        d.point((26, 22), fill=GREEN_LED)
        d.point((7, 8), fill=RED)


def draw_gun_module(d, rng):
    board(d, rng)
    chip(d, 10, 10, 5, 4)
    # gun pictogram, lower half of the board
    d.rectangle([8, 18, 22, 20], fill=STEEL_LIGHT)  # receiver/barrel line
    d.rectangle([20, 17, 24, 19], fill=STEEL_LIGHT)  # barrel tip
    d.polygon([(11, 20), (14, 20), (13, 24), (10, 24)],
              fill=STEEL_LIGHT)  # grip
    d.rectangle([17, 20, 19, 23], fill=STEEL_LIGHT)  # magazine
    d.point((24, 12), fill=RED)  # armed LED


def draw_ident_module(d, rng):
    board(d, rng)
    # viewfinder brackets + face dots (friend-or-foe recognition)
    for bx, by, dx, dy in ((9, 9, 1, 1), (23, 9, -1, 1), (9, 21, 1, -1),
                           (23, 21, -1, -1)):
        d.line([bx, by, bx + 3 * dx, by], fill=STEEL_LIGHT, width=1)
        d.line([bx, by, bx, by + 3 * dy], fill=STEEL_LIGHT, width=1)
    d.point((14, 13), fill=STEEL_LIGHT)
    d.point((18, 13), fill=STEEL_LIGHT)
    d.arc([13, 13, 19, 19], start=20, end=160, fill=STEEL_LIGHT, width=1)
    chip(d, 16, 25, 6, 3, pins=False)


def draw_memory_module(d, rng):
    board(d, rng)
    # banks of identical memory chips: 2 rows of 3
    for r in range(2):
        for c in range(3):
            chip(d, 10 + c * 6, 11 + r * 7, 4, 4, pins=False)
    d.line([7, 22, 25, 22], fill=GOLD, width=1)  # bank bus
    d.point((26, 9), fill=GREEN_LED)


def draw_pathfinding(d, rng):
    board(d, rng)
    chip(d, 8, 9, 4, 3, pins=False)
    # dotted route winding across the board to a destination pin
    pts = [(8, 21), (12, 21), (12, 16), (17, 16), (17, 11), (22, 11)]
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        steps = max(abs(bx - ax), abs(by - ay))
        for s in range(steps + 1):
            if s % 2 == 0:
                d.point((ax + (bx - ax) * s // max(steps, 1),
                         ay + (by - ay) * s // max(steps, 1)), fill=GOLD)
    for wx, wy in pts[1:-1]:
        d.point((wx, wy), fill=STEEL_LIGHT)
    d.ellipse([21, 7, 24, 10], fill=RED)  # destination pin head
    d.point((22, 10), fill=RED)
    d.point((22, 8), fill=(240, 220, 210, 255))


def draw_targeting(d, rng):
    board(d, rng)
    cx, cy = 15, 14
    d.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], outline=STEEL_LIGHT,
              width=1)
    d.line([cx - 8, cy, cx - 3, cy], fill=STEEL_LIGHT, width=1)
    d.line([cx + 3, cy, cx + 8, cy], fill=STEEL_LIGHT, width=1)
    d.line([cx, cy - 8, cx, cy - 3], fill=STEEL_LIGHT, width=1)
    d.line([cx, cy + 3, cx, cy + 8], fill=STEEL_LIGHT, width=1)
    d.point((cx, cy), fill=RED)
    chip(d, 25, 22, 4, 3, pins=False)


def draw_sensor_module(d, rng):
    # a sensor-array panel: dark housing, lenses across the face
    d.rectangle([4, 9, 28, 23], fill=(58, 62, 68, 255))
    d.rectangle([4, 9, 28, 23], outline=JOINT, width=1)
    d.line([4, 9, 28, 9], fill=STEEL_LIGHT, width=1)
    d.ellipse([7, 12, 13, 18], fill=CHIP)  # main camera lens
    d.ellipse([7, 12, 13, 18], outline=STEEL_LIGHT, width=1)
    d.point((10, 15), fill=LENS)
    d.point((9, 13), fill=(220, 235, 245, 255))
    d.ellipse([16, 13, 20, 17], fill=CHIP)  # IR lens
    d.point((18, 15), fill=RED)
    d.ellipse([23, 13, 26, 16], fill=CHIP)  # small lens
    d.point((24, 14), fill=(120, 220, 140, 255))
    for x in range(7, 26, 3):  # sensor pips along the bottom
        d.point((x, 21), fill=GOLD)
    d.rectangle([14, 5, 18, 9], fill=JOINT)  # mounting stalk


def draw_self_monitoring(d, rng):
    # diagnostic panel: little screen with a pulse trace + LEDs
    d.rectangle([5, 7, 27, 25], fill=(58, 62, 68, 255))
    d.rectangle([5, 7, 27, 25], outline=JOINT, width=1)
    d.rectangle([8, 10, 24, 19], fill=(16, 30, 22, 255))
    d.rectangle([8, 10, 24, 19], outline=STEEL_DARK, width=1)
    pts = [(9, 15), (12, 15), (13, 11), (15, 18), (16, 15), (20, 15),
           (21, 12), (23, 15)]
    d.line(pts, fill=GREEN_LED, width=1)  # ECG pulse
    for i, col in enumerate((GREEN_LED, GREEN_LED, (240, 200, 60, 255),
                             GREEN_LED)):
        d.point((9 + i * 4, 22), fill=col)  # diagnostic LEDs
    d.rectangle([22, 21, 25, 23], outline=STEEL_LIGHT, width=1)


def limb(d, x, y, upper, fore, flip=False):
    """One jointed android limb segment chain (vertical)."""
    s = -1 if flip else 1
    d.rectangle([x - 2, y, x + 2, y + upper], fill=STEEL_LIGHT)
    d.line([x - 2, y, x - 2, y + upper], fill=shade(STEEL_LIGHT, 0.75),
           width=1)
    jy = y + upper
    d.ellipse([x - 2, jy - 2, x + 2, jy + 2], fill=JOINT)
    d.rectangle([x - 2, jy + 2, x + 1, jy + 2 + fore], fill=STEEL_LIGHT)
    d.line([x + 1, jy + 2, x + 1, jy + 2 + fore],
           fill=shade(STEEL_LIGHT, 0.75), width=1)
    return jy + 2 + fore


def draw_android_arms(d, rng):
    for x in (10, 22):  # a pair of human-like arms
        d.ellipse([x - 3, 3, x + 3, 7], fill=STEEL_LIGHT)  # shoulder cap
        end = limb(d, x, 6, 8, 7)
        # simple hand: palm + three finger lines
        d.rectangle([x - 2, end, x + 2, end + 2], fill=STEEL_LIGHT)
        for fx in (x - 2, x, x + 2):
            d.line([fx, end + 2, fx, end + 4], fill=STEEL_LIGHT, width=1)
    d.line([10, 4, 22, 4], fill=JOINT, width=1)  # display bar


def draw_android_legs(d, rng):
    d.rectangle([7, 4, 25, 7], fill=JOINT)  # hip bar
    for x in (11, 21):
        end = limb(d, x, 7, 8, 8)
        d.rectangle([x - 3, end, x + 3, end + 2],
                    fill=STEEL_LIGHT)  # foot
        d.line([x - 3, end + 2, x + 3, end + 2], fill=JOINT, width=1)


def draw_android_chassis(d, rng):
    cx = 16
    d.rectangle([cx - 2, 3, cx + 2, 6], fill=JOINT)  # neck stub
    d.line([8, 8, 24, 8], fill=STEEL_LIGHT, width=2)  # shoulder bar
    d.point((8, 8), fill=JOINT)
    d.point((24, 8), fill=JOINT)
    d.rectangle([cx - 1, 8, cx + 1, 24], fill=STEEL_LIGHT)  # spine
    for i, ry in enumerate((11, 15, 19)):  # rib arcs
        w = 6 - i
        d.arc([cx - w, ry - 2, cx + w, ry + 4], start=0, end=180,
              fill=STEEL_LIGHT, width=1)
    d.rectangle([cx - 5, 24, cx + 5, 27], fill=JOINT)  # pelvis
    d.point((cx, 13), fill=LENS)  # chest core


def draw_copbot_chassis(d, rng):
    # armored police-bot shell: heavier plates than the android
    d.rectangle([9, 6, 23, 20], fill=STEEL_LIGHT)
    d.rectangle([9, 6, 23, 20], outline=JOINT, width=1)
    d.rectangle([9, 9, 23, 11], fill=LENS)  # police blue stripe
    d.rectangle([6, 7, 9, 15], fill=shade(STEEL_LIGHT, 0.9))  # shoulder armor
    d.rectangle([23, 7, 26, 15], fill=shade(STEEL_LIGHT, 0.9))
    d.ellipse([14, 13, 18, 17], outline=JOINT, width=1)  # badge ring
    d.point((16, 15), fill=GOLD)
    d.rectangle([12, 20, 20, 24], fill=JOINT)  # waist frame
    for x in (11, 21):  # leg stubs
        d.rectangle([x - 2, 24, x + 2, 28], fill=STEEL_LIGHT)
    d.point((16, 7), fill=RED)  # beacon nub


def draw_turret_chassis(d, rng):
    # boxy interior frame with an empty turret ring on top
    d.ellipse([9, 4, 23, 10], outline=STEEL_LIGHT, width=2)
    d.ellipse([12, 5, 20, 9], outline=JOINT, width=1)
    d.rectangle([8, 12, 24, 26], outline=STEEL_LIGHT, width=2)
    d.line([8, 12, 24, 26], fill=JOINT, width=1)  # frame struts
    d.line([24, 12, 8, 26], fill=JOINT, width=1)
    d.rectangle([13, 17, 19, 22], fill=(52, 56, 62, 255))  # empty bay
    for px, py in ((8, 12), (24, 12), (8, 26), (24, 26)):
        d.point((px, py), fill=GOLD)


def draw_omni_wheel(d, rng):
    for cx, cy, r in ((11, 13, 7), (22, 13, 7), (16, 23, 6)):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=JOINT)
        d.ellipse([cx - r, cy - r, cx + r, cy + r],
                  outline=STEEL_LIGHT, width=1)
        for k in range(8):  # rollers around the rim
            a = k * math.pi / 4 + 0.4
            d.point((int(cx + (r - 1) * math.cos(a)),
                     int(cy + (r - 1) * math.sin(a))), fill=STEEL_LIGHT)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_LIGHT)
        d.point((cx, cy), fill=JOINT)


def draw_quad_rotors(d, rng):
    cx, cy = 16, 16
    d.line([10, 10, 22, 22], fill=JOINT, width=1)  # X arms
    d.line([22, 10, 10, 22], fill=JOINT, width=1)
    d.rectangle([cx - 3, cy - 3, cx + 3, cy + 3], fill=(58, 62, 68, 255))
    d.point((cx, cy), fill=LENS)
    for rx, ry in ((8, 8), (24, 8), (8, 24), (24, 24)):  # rotor discs
        d.ellipse([rx - 5, ry - 2, rx + 5, ry + 2],
                  outline=STEEL_LIGHT, width=1)
        d.line([rx - 4, ry, rx + 4, ry], fill=STEEL_LIGHT, width=1)
        d.point((rx, ry), fill=JOINT)


def draw_spidery(d, rng, big):
    cx, cy = 16, 13 if big else 15
    reach = 13 if big else 8
    body_r = 4 if big else 3
    for sgn in (-1, 1):  # three legs a side, jointed, pointy feet
        for i, dy in enumerate((-4, 0, 4)):
            kx = cx + sgn * (reach - 4)
            ky = cy + dy - (2 if big else 1)
            fx = cx + sgn * reach
            fy = cy + dy + (7 if big else 5)
            d.line([cx + sgn * body_r, cy + dy // 2, kx, ky],
                   fill=STEEL_DARK, width=1)
            d.line([kx, ky, fx, fy], fill=STEEL_DARK, width=1)
            d.point((kx, ky), fill=JOINT)
            d.point((fx, fy), fill=STEEL_LIGHT)  # pointy tip
    d.ellipse([cx - body_r, cy - body_r, cx + body_r, cy + body_r],
              fill=STEEL_LIGHT)
    d.ellipse([cx - body_r, cy - body_r, cx + body_r, cy + body_r],
              outline=JOINT, width=1)
    d.point((cx - 1, cy - 1), fill=RED)
    d.point((cx + 1, cy - 1), fill=RED)


def draw_tank_tread(d, rng):
    # main tread in profile, second tread peeking behind (offset)
    d.rounded_rectangle([10, 6, 29, 20], radius=6, fill=JOINT)
    d.rounded_rectangle([4, 10, 23, 24], radius=6, fill=RUBBER)
    d.rounded_rectangle([4, 10, 23, 24], radius=6,
                        outline=STEEL_DARK, width=1)
    for x in range(6, 22, 3):  # tread pads, top and bottom runs
        d.line([x, 10, x, 12], fill=STEEL_DARK, width=1)
        d.line([x, 22, x, 24], fill=STEEL_DARK, width=1)
    for wx in (9, 14, 19):  # road wheels
        d.ellipse([wx - 2, 15, wx + 2, 19], fill=STEEL_LIGHT)
        d.point((wx, 17), fill=JOINT)
    d.ellipse([12, 14, 16, 18], outline=GOLD, width=1)  # drive sprocket


ROBOT = {
    "ai_module": lambda d, r: draw_ai(d, r, 1),
    "ai_module_advanced": lambda d, r: draw_ai(d, r, 2),
    "ai_module_basic": lambda d, r: draw_ai(d, r, 0),
    "android_arms": draw_android_arms,
    "android_chassis": draw_android_chassis,
    "android_legs": draw_android_legs,
    "copbot_chassis": draw_copbot_chassis,
    "gun_module": draw_gun_module,
    "identification_module": draw_ident_module,
    "memory_module": draw_memory_module,
    "omni_wheel": draw_omni_wheel,
    "pathfinding_module": draw_pathfinding,
    "quad_rotors": draw_quad_rotors,
    "self_monitoring_module": draw_self_monitoring,
    "sensor_module": draw_sensor_module,
    "spidery_legs_big": lambda d, r: draw_spidery(d, r, True),
    "spidery_legs_small": lambda d, r: draw_spidery(d, r, False),
    "tank_tread": draw_tank_tread,
    "targeting_module": draw_targeting,
    "turret_chassis": draw_turret_chassis,
}


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in STOCKS:
        STOCKS[sid](d, rng)
    elif sid in ROBOT:
        ROBOT[sid](d, rng)
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
    # contact-sheet preview: 8 cols x 5 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1,
                                   (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item61.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item61.png {prev.size}")


if __name__ == "__main__":
    main()
