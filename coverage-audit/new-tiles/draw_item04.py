#!/usr/bin/env python3
"""Batch ITEM04 (Phase 2g.4 scope extension): generic.json first set, 52 ids.

Drawn from each entry's name/description/color/symbol in
fork-src/cdda/data/json/items/generic.json. Items are objects on a
transparent background, native 32x32, flat UltiCa style. The set is
mostly the broken_* wreck family (drones, turrets, humanoid bots,
mechs — parameterized renderers with per-id color/variant, all
slumped and damaged per their descriptions) plus gems, documents,
and material miscellany.

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is
strictly {0, 255}: translucency is faked with pale opaque colors.
Writes ONLY its own 52 folders + preview_item04.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
BASE = os.path.join(HERE, "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [l.strip() for l in open("/tmp/item_batches/04.txt") if l.strip()]

# CDDA color names -> sprite RGB.
COL = {
    "green": (74, 148, 62), "light_green": (128, 214, 96),
    "yellow": (216, 192, 64), "blue": (72, 102, 216),
    "cyan": (64, 196, 216), "light_blue": (136, 192, 240),
    "dark_gray": (92, 92, 98), "light_gray": (172, 172, 178),
    "magenta": (198, 64, 178), "white": (236, 236, 236),
    "red": (198, 62, 52), "brown": (128, 92, 58),
    "pink": (232, 160, 186), "black": (36, 36, 40),
}


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c) + (255,)


def R(c):
    return tuple(c) + (255,)


def new_img():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def cracks(d, rng, n, x0=6, x1=26, y0=6, y1=26, col=(30, 30, 32, 255)):
    for _ in range(n):
        x, y = rng.uniform(x0, x1), rng.uniform(y0, y1)
        pts = [(x, y)]
        for _ in range(rng.randint(2, 3)):
            x += rng.uniform(-4, 4)
            y += rng.uniform(0, 4)
            pts.append((x, y))
        d.line(pts, fill=col, width=1)


def spark(d, rng):
    x, y = rng.uniform(8, 24), rng.uniform(6, 14)
    for dx, dy in ((0, -3), (2, -1), (-2, -1), (1, 2), (-1, 2)):
        d.line([(x, y), (x + dx, y + dy)], fill=(250, 214, 90, 255), width=1)


# ---------------------------------------------------------------- gems/docs
def draw_gem(color):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL[color]
    d.polygon([(8, 12), (13, 6), (19, 6), (24, 12), (16, 27)],
              fill=shade(c, 0.85))
    d.polygon([(8, 12), (24, 12), (19, 6), (13, 6)], fill=R(c))
    d.polygon([(13, 6), (19, 6), (16, 27)], fill=shade(c, 0.6))
    d.polygon([(8, 12), (13, 6), (16, 27)], fill=shade(c, 1.15))
    d.line([(8, 12), (24, 12)], fill=shade(c, 0.5), width=1)
    d.point((12, 9), fill=(255, 255, 255, 255))
    d.point((20, 15), fill=(255, 255, 255, 255))
    return img


def draw_document(kind, rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([8, 4, 24, 28], fill=(238, 236, 228, 255))
    d.polygon([(24, 24), (24, 28), (20, 28)], fill=(196, 192, 180, 255))
    d.rectangle([10, 6, 14, 9], fill=(60, 60, 66, 255))  # XEDRA logo block
    if kind == "haadf":
        d.rectangle([10, 12, 22, 22], fill=(24, 24, 30, 255))
        for _ in range(26):
            d.point((rng.uniform(11, 21), rng.uniform(13, 21)),
                    fill=(190, 220, 235, 255))
        d.line([(10, 25), (22, 25)], fill=(150, 148, 140, 255))
    elif kind == "scp":
        for y in (12, 15, 18):
            d.line([(10, y), (22, y)], fill=(150, 148, 140, 255))
        d.ellipse([15, 19, 23, 27], outline=(198, 62, 52, 255), width=1)
        d.line([(17, 25), (21, 21)], fill=(198, 62, 52, 255))
    return img


# ------------------------------------------------------------------- smalls
def draw_amplifier(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([5, 9, 27, 24], fill=(104, 150, 96, 255))
    d.rectangle([5, 9, 27, 24], outline=(70, 104, 64, 255))
    for x in (3, 29):
        for y in (12, 17, 22):
            d.line([(x, y), (x + (2 if x == 3 else -2), y)],
                   fill=(196, 196, 120, 255))
    d.rectangle([12, 13, 20, 19], fill=(40, 40, 46, 255))
    d.rectangle([7, 20, 10, 22], fill=(150, 60, 50, 255))
    for _ in range(7):
        x0, y0 = rng.uniform(6, 24), rng.uniform(10, 22)
        d.line([(x0, y0), (x0 + rng.uniform(2, 5), y0)],
               fill=(196, 196, 120, 255))
    return img


def draw_ballast(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    flesh = (196, 158, 148)
    for cx, cy, r in ((11, 21, 6), (20, 23, 5), (16, 16, 5), (23, 15, 4)):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=shade(flesh, 0.9))
        d.ellipse([cx - r + 1, cy - r + 1, cx + r - 2, cy + r - 3],
                  fill=R(flesh))
    # two lighter-than-air organs drifting up
    for cx, cy, r in ((10, 6, 3), (22, 8, 2)):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=shade(flesh, 1.12))
        d.line([(cx, cy + r), (cx, cy + r + 3)], fill=shade(flesh, 0.7))
    return img


def draw_bud(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.line([(16, 30), (16, 24)], fill=(70, 120, 60, 255), width=2)
    d.polygon([(10, 25), (6, 20), (11, 21)], fill=(70, 120, 60, 255))
    d.polygon([(22, 25), (26, 20), (21, 21)], fill=(70, 120, 60, 255))
    bud = COL["magenta"]
    d.polygon([(16, 4), (23, 12), (22, 22), (16, 26), (10, 22), (9, 12)],
              fill=shade(bud, 0.8))
    d.polygon([(16, 4), (18, 12), (16, 26), (14, 12)], fill=R(bud))
    d.polygon([(16, 4), (23, 12), (18, 12)], fill=shade(bud, 1.15))
    return img


def draw_scythe_blade(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    steel = COL["light_gray"]
    d.polygon([(4, 24), (8, 12), (18, 5), (28, 6), (20, 9), (12, 14),
               (8, 22), (10, 26)], fill=shade(steel, 0.95))
    d.polygon([(8, 12), (18, 5), (28, 6), (20, 9), (12, 14)],
              fill=shade(steel, 1.12))
    d.rectangle([3, 23, 7, 27], fill=(96, 74, 48, 255))  # tang mount
    return img


def draw_bouquet(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    # glass vase in pale opaque glass tones
    d.polygon([(12, 16), (20, 16), (22, 28), (10, 28)], fill=(196, 222, 226, 255))
    d.rectangle([12, 14, 20, 17], fill=(170, 200, 206, 255))
    wilt = [(196, 178, 140), (178, 150, 130), (206, 196, 170)]
    for i, (cx, cy) in enumerate(((10, 9), (16, 6), (22, 9), (13, 12), (19, 12))):
        d.line([(16, 15), (cx, cy + 2)], fill=(110, 116, 84, 255))
        c = wilt[i % 3]
        d.ellipse([cx - 3, cy - 2, cx + 3, cy + 3], fill=R(c))
        d.ellipse([cx - 1, cy, cx + 1, cy + 2], fill=shade(c, 0.7))
    return img


def draw_apparatus(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    red = COL["red"]
    d.ellipse([10, 22, 22, 29], fill=shade(red, 0.85))
    d.rectangle([14, 12, 18, 23], fill=shade(red, 1.0))
    d.ellipse([13, 9, 19, 13], fill=(70, 70, 76, 255))  # bowl
    d.arc([18, 14, 30, 28], 90, 300, fill=(70, 70, 76, 255), width=2)  # hose
    # flame
    d.polygon([(16, 2), (18, 6), (16, 9), (14, 6)], fill=(250, 170, 60, 255))
    d.polygon([(16, 5), (17, 7), (16, 8), (15, 7)], fill=(250, 226, 130, 255))
    return img


def draw_animal(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = (206, 202, 196)
    # horse silhouette (the "animal" pseudo fuel)
    d.polygon([(7, 14), (10, 10), (20, 10), (24, 6), (26, 6), (25, 11),
               (22, 13), (22, 22), (20, 22), (19, 15), (12, 15), (11, 22),
               (9, 22), (9, 14)], fill=R(c))
    d.polygon([(24, 6), (27, 4), (26, 8)], fill=R(c))  # ear/head top
    d.point((24, 8), fill=(60, 60, 60, 255))
    d.line([(7, 13), (5, 9)], fill=R(c), width=2)  # tail
    return img


def draw_burnt_bionic(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    body = COL["dark_gray"]
    d.rectangle([9, 8, 23, 26], fill=shade(body, 0.8))
    d.rectangle([9, 8, 23, 12], fill=shade(body, 0.55))
    d.rectangle([12, 16, 20, 22], fill=(28, 28, 30, 255))
    d.polygon([(9, 26), (14, 20), (18, 27), (23, 22), (23, 26)],
              fill=(22, 22, 24, 255))  # melted base
    cracks(d, rng, 3, 10, 22, 9, 25, col=(16, 16, 18, 255))
    d.point((20, 14), fill=(120, 70, 40, 255))  # scorch
    d.point((12, 19), fill=(120, 70, 40, 255))
    return img


def draw_canister(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    steel = COL["light_gray"]
    d.rectangle([11, 9, 21, 27], fill=shade(steel, 0.95))
    d.ellipse([11, 6, 21, 11], fill=shade(steel, 1.12))
    d.rectangle([14, 3, 18, 7], fill=shade(steel, 0.7))
    d.rectangle([11, 16, 21, 19], fill=shade(steel, 0.75))
    d.line([(13, 22), (19, 22)], fill=shade(steel, 0.65))
    return img


# ------------------------------------------------------------------- drones
def draw_drone(color, rng, variant="plain"):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL[color]
    # four arms with still propellers, one bent
    for cx, cy in ((8, 9), (24, 9), (8, 23), (24, 23)):
        d.line([(16, 16), (cx, cy)], fill=shade(c, 0.7), width=2)
        d.ellipse([cx - 4, cy - 2, cx + 4, cy + 2], fill=shade(c, 0.55))
    d.line([(20, 7), (26, 4)], fill=shade(c, 0.55), width=1)  # bent prop
    d.rectangle([10, 11, 22, 21], fill=R(c))
    d.rectangle([10, 11, 22, 14], fill=shade(c, 1.12))
    d.ellipse([13, 14, 19, 20], fill=(26, 28, 34, 255))  # camera eye, inert
    if variant == "tazer":
        d.line([(12, 21), (12, 25)], fill=(200, 200, 90, 255), width=1)
        d.line([(20, 21), (20, 25)], fill=(200, 200, 90, 255), width=1)
    if variant == "grenade":
        d.ellipse([12, 12, 20, 20], outline=shade(c, 0.6), width=1)
    cracks(d, rng, 2, 10, 22, 11, 21)
    return img


# ------------------------------------------------------------------ turrets
def draw_turret(color, rng, gun="mg", trim=None):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL[color]
    # slumped base, tipped slightly
    d.polygon([(7, 27), (10, 15), (23, 14), (26, 27)], fill=shade(c, 0.85))
    d.rectangle([9, 10, 24, 16], fill=R(c))
    d.ellipse([14, 11, 19, 15], fill=(26, 28, 34, 255))  # dead sensor eye
    dark = (44, 46, 50, 255)
    if gun == "mg":
        d.line([(19, 12), (30, 9)], fill=dark, width=3)
        d.rectangle([24, 13, 28, 18], fill=shade(c, 0.7))  # ammo box
    elif gun == "bmg":
        d.line([(19, 12), (31, 8)], fill=dark, width=4)
        d.rectangle([23, 13, 29, 19], fill=shade(c, 0.7))
    elif gun == "searchlight":
        d.ellipse([18, 5, 30, 17], fill=shade(c, 0.9))
        d.ellipse([20, 7, 28, 15], fill=(196, 206, 200, 255))
        d.line([(21, 8), (27, 14)], fill=(90, 96, 100, 255))  # cracked lens
        d.line([(27, 8), (21, 14)], fill=(90, 96, 100, 255))
    elif gun == "speaker":
        d.polygon([(19, 8), (29, 5), (29, 17), (19, 14)], fill=R(c))
        d.ellipse([27, 6, 30, 16], fill=shade(c, 0.65))
    elif gun == "riot":
        d.line([(19, 12), (28, 10)], fill=dark, width=2)
        d.rectangle([20, 15, 27, 24], fill=shade(c, 0.75))  # riot plate
        d.rectangle([22, 17, 25, 20], fill=(150, 190, 220, 255))
    if trim == "artisans":
        d.rectangle([9, 18, 24, 21], fill=(168, 140, 84, 255))  # brass trim
        d.rectangle([10, 22, 13, 26], fill=shade(c, 1.1))
        d.rectangle([20, 22, 23, 26], fill=shade(c, 1.1))
    cracks(d, rng, 3, 8, 25, 10, 26)
    return img


# --------------------------------------------------------------- human bots
def draw_humanoid_bot(color, rng, face="vacant", variant="plain"):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL[color]
    # slumped torso
    d.polygon([(9, 28), (10, 14), (22, 13), (24, 28)], fill=R(c))
    d.rectangle([10, 20, 23, 23], fill=shade(c, 0.8))
    # dangling arm
    d.line([(10, 16), (5, 24)], fill=shade(c, 0.75), width=3)
    d.line([(23, 15), (27, 21)], fill=shade(c, 0.75), width=3)
    # head, tilted
    tilt = 2 if variant in ("defective", "busted") else 0
    d.rectangle([11 + tilt, 4, 22 + tilt, 13], fill=shade(c, 1.08))
    if face == "smile":
        d.point((14 + tilt, 8), fill=(30, 30, 34, 255))
        d.point((19 + tilt, 8), fill=(30, 30, 34, 255))
        d.arc([14 + tilt, 7, 20 + tilt, 12], 20, 160, fill=(30, 30, 34, 255))
    elif face == "smooth":
        d.line([(14 + tilt, 8), (20 + tilt, 8)], fill=(52, 54, 60, 255))
    if variant == "busted":
        cracks(d, rng, 4, 10, 24, 4, 27, col=(40, 38, 34, 255))
        d.rectangle([10, 24, 23, 26], fill=(104, 92, 74, 255))  # tarnish
    if variant == "defective":
        spark(d, rng)
        cracks(d, rng, 2)
    if variant == "prototype":
        d.line([(12, 23), (16, 27)], fill=(198, 62, 52, 255))  # exposed wire
        d.line([(18, 23), (21, 27)], fill=(216, 192, 64, 255))
        d.rectangle([15, 16, 19, 20], fill=(40, 40, 46, 255))
    if variant == "secubot":
        cracks(d, rng, 3)
        d.ellipse([8, 27, 25, 30], fill=(70, 96, 88, 255))  # drained fluid
    if variant == "balthazar":
        d.rectangle([13, 16, 20, 19], fill=(168, 140, 84, 255))  # name plate
    return img


def draw_camspy(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["green"]
    d.ellipse([8, 9, 24, 25], fill=R(c))
    d.ellipse([12, 13, 21, 22], fill=(26, 28, 34, 255))
    d.ellipse([14, 15, 18, 19], fill=(70, 110, 130, 255))
    for x in (10, 16, 22):
        d.line([(x, 25), (x - 1, 29)], fill=shade(c, 0.7), width=2)
    cracks(d, rng, 2, 9, 23, 10, 24)
    return img


def draw_skitter(rng, gun=False):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["green"]
    for i, x in enumerate((6, 11, 21, 26)):
        y = 24 + (i % 2)
        d.line([(x, 20), (x - 2, y + 4)], fill=shade(c, 0.65), width=2)
        d.line([(x, 20), (x + 2, y + 4)], fill=shade(c, 0.65), width=2)
    d.polygon([(7, 20), (9, 11), (23, 11), (25, 20)], fill=R(c))
    d.ellipse([13, 13, 19, 17], fill=(26, 28, 34, 255))
    if gun:
        d.line([(20, 12), (29, 9)], fill=(44, 46, 50, 255), width=2)
    cracks(d, rng, 2, 8, 24, 11, 20)
    return img


def draw_hound(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["green"]
    d.rectangle([8, 13, 24, 21], fill=R(c))  # body, limp
    d.rectangle([21, 9, 28, 15], fill=shade(c, 0.9))  # head dropped
    d.point((25, 11), fill=(30, 30, 34, 255))
    for x in (10, 14, 19, 22):
        d.line([(x, 21), (x - 1, 27)], fill=shade(c, 0.7), width=2)
    d.rectangle([11, 15, 17, 19], fill=shade(c, 0.75))
    cracks(d, rng, 2, 9, 24, 12, 21)
    return img


def draw_dispatch(color, rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL[color]
    d.rectangle([6, 10, 26, 25], fill=R(c))
    d.rectangle([6, 10, 26, 14], fill=shade(c, 1.1))
    # mesh belly with fried manhacks inside
    d.rectangle([9, 16, 23, 23], fill=(52, 54, 58, 255))
    for cx, cy in ((12, 19), (17, 20), (21, 18)):
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=(120, 124, 130, 255))
        d.point((cx, cy), fill=(30, 30, 34, 255))
    for x in range(9, 24, 3):
        d.line([(x, 16), (x, 23)], fill=(88, 90, 96, 255))
    d.ellipse([8, 24, 13, 29], fill=(40, 40, 44, 255))  # limp wheels
    d.ellipse([20, 24, 25, 29], fill=(40, 40, 44, 255))
    cracks(d, rng, 2, 7, 25, 10, 25)
    return img


# -------------------------------------------------------------------- mechs
def draw_mech(rng, kind):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["light_green"]
    # collapsed legs
    d.line([(12, 21), (8, 28)], fill=shade(c, 0.7), width=3)
    d.line([(20, 21), (25, 27)], fill=shade(c, 0.7), width=3)
    # frame torso
    d.polygon([(9, 20), (10, 8), (22, 8), (23, 20)], fill=R(c))
    d.rectangle([12, 11, 20, 17], fill=shade(c, 0.72))  # empty cockpit frame
    d.point((16, 9), fill=(30, 30, 34, 255))
    if kind == "combat":
        d.line([(10, 11), (4, 17)], fill=shade(c, 0.8), width=3)
        d.line([(4, 17), (2, 13)], fill=(44, 46, 50, 255), width=3)  # gun arm
        d.line([(22, 11), (28, 16)], fill=shade(c, 0.8), width=3)
    elif kind == "lifter":
        d.line([(10, 11), (4, 15)], fill=shade(c, 0.8), width=3)
        d.polygon([(2, 15), (6, 15), (4, 19)], fill=shade(c, 0.65))  # claw
        d.line([(22, 11), (28, 15)], fill=shade(c, 0.8), width=3)
        d.polygon([(26, 15), (30, 15), (28, 19)], fill=shade(c, 0.65))
    elif kind == "recon":
        d.line([(16, 8), (16, 3)], fill=shade(c, 0.8), width=1)  # antenna
        d.point((16, 3), fill=(30, 30, 34, 255))
        d.line([(10, 11), (5, 18)], fill=shade(c, 0.8), width=2)
        d.line([(22, 11), (27, 18)], fill=shade(c, 0.8), width=2)
    cracks(d, rng, 3, 9, 23, 8, 21)
    return img


def draw_talon(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["green"]
    # tracked UGV base, casing cracked
    d.rectangle([6, 20, 26, 27], fill=shade(c, 0.8))
    for x in range(8, 25, 4):
        d.point((x, 24), fill=(30, 30, 34, 255))
    d.rectangle([10, 13, 22, 21], fill=R(c))
    d.line([(16, 13), (16, 7)], fill=shade(c, 0.75), width=2)  # camera mast
    d.rectangle([13, 4, 19, 8], fill=shade(c, 0.9))
    d.point((16, 6), fill=(30, 30, 34, 255))
    d.ellipse([9, 26, 24, 30], fill=(70, 96, 88, 255))  # drained fluid
    cracks(d, rng, 3, 10, 22, 13, 21)
    return img


def draw_wells_machine(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["light_gray"]
    d.polygon([(4, 24), (6, 12), (14, 6), (24, 8), (28, 16), (26, 25)],
              fill=shade(c, 0.85))
    d.polygon([(6, 12), (14, 6), (24, 8), (20, 14), (10, 15)],
              fill=R(c))
    # twisted weapons
    d.line([(22, 10), (30, 5)], fill=(52, 54, 58, 255), width=3)
    d.line([(30, 5), (27, 10)], fill=(52, 54, 58, 255), width=2)
    d.line([(8, 14), (2, 10)], fill=(52, 54, 58, 255), width=2)
    d.ellipse([13, 13, 19, 19], fill=(30, 30, 34, 255))
    cracks(d, rng, 5, 5, 27, 7, 24, col=(24, 24, 26, 255))
    return img


def draw_wells_leg(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    c = COL["light_gray"]
    pts = [(5, 8), (12, 6), (18, 10), (14, 16), (10, 20), (16, 25), (24, 24)]
    for i in range(len(pts) - 1):
        d.line([pts[i], pts[i + 1]], fill=shade(c, 0.9), width=5)
    for x, y in pts:
        d.ellipse([x - 3, y - 3, x + 3, y + 3], outline=shade(c, 0.6), width=1)
    d.polygon([(24, 24), (29, 22), (28, 27)], fill=shade(c, 0.7))  # foot claw
    return img


# ------------------------------------------------------------------ dispatch
def build(tid, rng):
    if tid in ("alexandrite", "amethyst", "aquamarine", "blue_topaz"):
        return draw_gem({"alexandrite": "green", "amethyst": "blue",
                         "aquamarine": "cyan", "blue_topaz": "light_blue"}[tid])
    if tid == "HAADF_t_substrate":
        return draw_document("haadf", rng)
    if tid == "artifact_scp_doc":
        return draw_document("scp", rng)
    if tid == "amplifier":
        return draw_amplifier(rng)
    if tid == "ballast":
        return draw_ballast(rng)
    if tid == "biollante_bud":
        return draw_bud(rng)
    if tid == "blade_scythe":
        return draw_scythe_blade(rng)
    if tid == "bouquet":
        return draw_bouquet(rng)
    if tid == "apparatus":
        return draw_apparatus(rng)
    if tid == "animal":
        return draw_animal(rng)
    if tid == "burnt_out_bionic":
        return draw_burnt_bionic(rng)
    if tid == "canister_empty":
        return draw_canister(rng)
    drones = {"broken_lab_security_drone_BM": ("blue", "plain"),
              "broken_lab_security_drone_BM2": ("cyan", "plain"),
              "broken_lab_security_drone_BS": ("dark_gray", "plain"),
              "broken_lab_security_drone_GM": ("green", "plain"),
              "broken_lab_security_drone_GR": ("light_green", "plain"),
              "broken_lab_security_drone_YM": ("yellow", "plain"),
              "broken_tazer_hack": ("green", "tazer"),
              "broken_grenade_hack": ("green", "grenade")}
    if tid in drones:
        col, var = drones[tid]
        return draw_drone(col, rng, var)
    turrets = {"broken_turret": ("green", "mg", None),
               "broken_crows_m240": ("green", "mg", None),
               "broken_turret_rifle": ("green", "mg", None),
               "broken_turret_rifle_balthazar": ("green", "mg", None),
               "broken_turret_bmg": ("green", "bmg", None),
               "broken_robofac_laserturret_mk1": ("green", "searchlight", None),
               "broken_turret_searchlight": ("green", "searchlight", None),
               "broken_turret_speaker": ("yellow", "speaker", None),
               "broken_turret_riot": ("blue", "riot", None),
               "broken_turret_artisans": ("green", "mg", "artisans")}
    if tid in turrets:
        col, gun, trim = turrets[tid]
        if tid == "broken_crows_m240":
            gun = "bmg"
        return draw_turret(col, rng, gun, trim)
    bots = {"broken_grocerybot": ("dark_gray", "smile", "plain"),
            "broken_grocerybot_busted": ("dark_gray", "smile", "busted"),
            "broken_nursebot": ("dark_gray", "smooth", "plain"),
            "broken_nursebot_defective": ("dark_gray", "smooth", "defective"),
            "broken_robofac_prototype": ("green", "vacant", "prototype"),
            "broken_secubot": ("green", "vacant", "secubot"),
            "broken_secubot_balthazar": ("green", "vacant", "balthazar")}
    if tid in bots:
        col, face, var = bots[tid]
        return draw_humanoid_bot(col, rng, face, var)
    if tid == "broken_robofac_camspy":
        return draw_camspy(rng)
    if tid == "broken_skitterbot":
        return draw_skitter(rng, gun=False)
    if tid == "broken_robofac_skittergun":
        return draw_skitter(rng, gun=True)
    if tid == "broken_robofac_hound":
        return draw_hound(rng)
    if tid == "broken_dispatch":
        return draw_dispatch("yellow", rng)
    if tid == "broken_dispatch_military":
        return draw_dispatch("green", rng)
    mechs = {"broken_mech_combat": "combat", "broken_mech_lifter": "lifter",
             "broken_mech_recon": "recon"}
    if tid in mechs:
        return draw_mech(rng, mechs[tid])
    if tid == "broken_wells_machine":
        return draw_wells_machine(rng)
    if tid == "broken_wells_machine_leg":
        return draw_wells_leg(rng)
    if tid == "broken_talon_m202a1":
        return draw_talon(rng)
    raise ValueError("no renderer for " + tid)


def main():
    assert len(IDS) == 52, len(IDS)
    for tid in IDS:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = build(tid, rng)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (tid, alphas)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # Contact sheet: 8 cols, sprites at 2x NEAREST on dark ground.
    cell, gap, cols = 64, 3, 8
    rows = math.ceil(len(IDS) / cols)
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, tid in enumerate(IDS):
        img = Image.open(os.path.join(BASE, tid, tid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(HERE, "preview_item04.png")
    sheet.save(out)
    print("wrote preview_item04.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
