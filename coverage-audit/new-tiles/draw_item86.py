#!/usr/bin/env python3
"""ITEM86: motors, Nanite Labs devices, ammo, speedloaders, retool
kits, hornet's-nest shells and a mixed tail -- CDDA Voxel coverage
audit.

  Motors (4) -- cylindrical housings with cooling fins, end bell,
    shaft and terminal box, scaled by power class (small moped
    motor up to the huge 1,300 HP train engine).
  Nanite Labs / netherum devices (4) -- drawn from their labyrinth
    descriptions: the clamp-set guide-board (7 clamps in a circle),
    the laser calipers (laser tips meeting at centre), the folded
    mass of six robotic arms (7-fingered hand, camera dome), and
    the dark-glass brain of the surgical bonebot.
  Ammo -- each cartridge upright, case profile by caliber
    (bottleneck rifle / straight pistol / rimmed), bullet by load
    (FMJ point, soft-point lead tip, JHP cavity, copper penetrator,
    FTX red polymer tip, AP black tip, tracer tip), finish by
    prefix: bp_ soot-smudged, reloaded_ scuffed two-tone brass.
    20ga shells get the traditional yellow hull with the load at
    the crimp (bird dots, buck pellets, slug dome).
  Speedloaders (4) -- top-down discs with the rounds arranged in a
    ring: 5, 6 or 7 chambers; the 10mm loader has fatter rounds.
  Retool kits (2) -- CZ 600 barrel assembly + bolt head on a roll,
    .223 slim vs 7.62 thicker.
  Hornet's nests (3) -- fat 40mm adapter face with four .410 shell
    bases around a central primer; submunition told by the fill
    (bird dots, scrap chunks, slug domes).
  Tail (9) -- aim game (laser pistol + target), wooden mannequin,
    foam kickboard, defaced golden badge, shifting keyring, ornate
    bird-handled scissors, and three freshly-siliconed glass
    aquariums (drying) scaled 30 / 50 / 100 gallon.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "motor_enhanced",
    "motor_small",
    "motor_super",
    "motor_train1300",
    "nl_grav_aware_jig",
    "nl_laser_mass_indexer",
    "nl_robo_surg_app",
    "nl_surg_int_unit",
    "reloaded_3030",
    "reloaded_3030_bp",
    "reloaded_3030_fmj",
    "reloaded_3030_fmj_bp",
    "10mmP",
    "10mm_jhp",
    "reloaded_10mmP",
    "20shot_bird",
    "20shot_buck",
    "20shot_slug",
    "22_cb",
    "22_lr",
    "22_ratshot",
    "3006",
    "3006_incendiary",
    "3006fmj",
    "338lapua_api",
    "338lapua_fmjbt",
    "338lapua_hpbt",
    "38_speedloader",
    "38_speedloader5",
    "38_speedloader6",
    "40_speedloader6",
    "retool_cz600_223rem",
    "retool_cz600_762",
    "40x46mm_hornets_nest_410_bird",
    "40x46mm_hornets_nest_410_scrap",
    "40x46mm_hornets_nest_410_slug",
    "460sw_jhp",
    "reloaded_460sw",
    "reloaded_460sw_jhp",
    "77mm_arisaka_jsp",
    "bp_77mm_arisaka_jsp",
    "reloaded_77mm_arisaka_jsp",
    "aim_game",
    "mannequin",
    "swimming_kickboard",
    "altered_badge",
    "altered_keyring",
    "altered_scissors",
    "aquarium_large_drying",
    "aquarium_medium_drying",
    "aquarium_small_drying",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (46, 46, 52, 255)
STEEL = (148, 152, 160, 255)
STEEL_D = (104, 108, 118, 255)
STEEL_L = (188, 192, 200, 255)
IRON = (74, 76, 84, 255)
BRASS = (196, 156, 74, 255)
BRASS_D = (158, 118, 46, 255)
BRASS_L = (228, 192, 110, 255)
BRASS_DULL = (168, 138, 76, 255)
COPPER = (196, 110, 62, 255)
COPPER_D = (158, 82, 44, 255)
LEAD = (128, 130, 138, 255)
LEAD_D = (96, 98, 106, 255)
BLACK = (30, 30, 34, 255)
RED = (198, 48, 40, 255)
RED_D = (150, 32, 28, 255)
ORANGE = (232, 116, 40, 255)
GOLD = (216, 172, 60, 255)
GOLD_D = (172, 132, 40, 255)
TEAL = (46, 148, 140, 255)
TEAL_L = (96, 200, 190, 255)
WHITE = (236, 238, 240, 255)
CREAM = (240, 234, 218, 255)
WOOD = (140, 102, 60, 255)
WOOD_D = (106, 74, 40, 255)
YELLOW_HULL = (232, 196, 60, 255)
YELLOW_D = (196, 160, 44, 255)
BLUE = (70, 110, 190, 255)
BLUE_L = (140, 180, 230, 255)
GLASS = (196, 224, 230, 255)
GLASS_D = (140, 180, 190, 255)
SOOT = (58, 54, 50, 255)
PLASTIC_D = (52, 54, 62, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ motors

def draw_motor(d, rng, scale, accent=False, train=False):
    """Side view of a finned cylindrical electric motor."""
    cx = 16
    w, h = scale  # body half-width, half-height
    x0, x1 = cx - w, cx + w
    y0, y1 = 16 - h, 16 + h
    # mounting feet
    d.rectangle([x0 + 2, y1, x0 + 5, y1 + 2], fill=IRON)
    d.rectangle([x1 - 5, y1, x1 - 2, y1 + 2], fill=IRON)
    # body with cooling fins (vertical ribs)
    d.rectangle([x0, y0, x1, y1], fill=STEEL)
    for fx in range(x0 + 2, x1 - 1, 3):
        d.line([fx, y0, fx, y1], fill=STEEL_D, width=1)
    d.line([x0, y0, x1, y0], fill=STEEL_L, width=1)
    # end bell (left) with copper winding ring
    d.rectangle([x0 - 2, y0 + 2, x0, y1 - 2], fill=STEEL_D)
    d.line([x0 - 2, y0 + 4, x0 - 2, y1 - 4], fill=COPPER, width=1)
    # shaft (right)
    d.rectangle([x1 + 1, 15, x1 + (5 if not train else 3), 17], fill=STEEL_L)
    d.point((x1 + 2, 16), fill=INK)
    # terminal box on top
    if train:
        for tx in (cx - 5, cx, cx + 5):  # 3-phase terminals
            d.rectangle([tx - 1, y0 - 4, tx + 1, y0 - 1], fill=IRON)
            d.point((tx, y0 - 4), fill=COPPER)
        d.rectangle([x0 + 3, y0 - 2, x1 - 3, y0 - 1], fill=STEEL_D)
    else:
        d.rectangle([cx - 3, y0 - 3, cx + 3, y0 - 1], fill=IRON)
        d.point((cx, y0 - 2), fill=COPPER)
    if accent:  # enhanced: copper performance band
        d.line([x1 - 3, y0 + 1, x1 - 3, y1 - 1], fill=COPPER, width=2)


# ------------------------------------------------------- netherum lab tools

def draw_nl_jig(d, rng):
    # guide-board seen at a slant, 7 clamps in a circular configuration
    d.polygon([(5, 10), (27, 8), (28, 24), (6, 26)], fill=STEEL_D)
    d.polygon([(5, 10), (27, 8), (28, 24), (6, 26)], outline=INK)
    cx, cy = 16, 17
    for gx, gy in ((cx, cy), (cx - 6, cy - 3), (cx + 6, cy - 4),
                   (cx - 3, cy + 4), (cx + 4, cy + 4)):  # depressed guide marks
        d.point((gx, gy), fill=INK)
    d.line([cx - 7, cy + 1, cx + 7, cy - 2], fill=STEEL, width=1)
    for i in range(7):
        a = 2 * math.pi * i / 7
        x = int(cx + 7 * math.cos(a))
        y = int(cy + 5 * math.sin(a))
        d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=STEEL_L)
        d.point((x, y), fill=INK)  # clamp screw


def draw_nl_indexer(d, rng):
    # calipers: two long jaws from a spring handle; laser tips meet centre
    d.line([11, 4, 14, 14], fill=STEEL, width=2)   # left jaw
    d.line([21, 4, 18, 14], fill=STEEL, width=2)   # right jaw
    d.point((14, 14), fill=RED)
    d.point((18, 14), fill=RED)
    d.line([14, 14, 16, 15], fill=RED, width=1)    # lasers meeting
    d.line([18, 14, 16, 15], fill=RED, width=1)
    d.point((16, 15), fill=ORANGE)
    # spring-loaded handle below the pivot
    d.rectangle([13, 16, 19, 27], fill=STEEL_D)
    d.rectangle([13, 16, 19, 27], outline=INK)
    for sy in range(18, 26, 2):  # spring coils
        d.line([14, sy, 18, sy], fill=STEEL_L, width=1)
    # measurement screen on the handle
    d.rectangle([20, 18, 24, 23], fill=INK)
    d.line([21, 20, 23, 20], fill=TEAL_L, width=1)
    d.line([21, 22, 22, 22], fill=TEAL_L, width=1)
    d.line([19, 20, 20, 21], fill=STEEL, width=1)


def draw_nl_robo(d, rng):
    cx, cy = 16, 15
    # central hub
    d.ellipse([cx - 4, cy - 3, cx + 4, cy + 3], fill=STEEL)
    d.ellipse([cx - 4, cy - 3, cx + 4, cy + 3], outline=INK)
    d.point((cx, cy), fill=TEAL_L)
    # six folded arms (jointed segments), like a dead spider
    arms = [(-11, -6, -13, 2), (-8, -9, -4, -12), (8, -9, 5, -12),
            (11, -6, 13, 2), (-10, 7, -12, 12), (10, 7, 12, 12)]
    for dx, dy, ex, ey in arms:
        jx, jy = cx + dx, cy + dy
        d.line([cx, cy, jx, jy], fill=STEEL_D, width=2)
        d.point((jx, jy), fill=INK)
        d.line([jx, jy, cx + ex, cy + ey], fill=STEEL_D, width=1)
    # big arm 1 (upper left): human-like hand with 7 fingers
    hx, hy = cx - 13, cy + 2
    d.rectangle([hx - 2, hy - 1, hx + 1, hy + 2], fill=STEEL_L)
    for fi in range(7):
        fx = hx - 3 + fi
        d.point((fx, hy + 3 + (1 if fi in (0, 6) else 0)), fill=STEEL_L)
    # big arm 2 (upper right): dome security camera
    dx_, dy_ = cx + 13, cy + 2
    d.arc([dx_ - 3, dy_ - 3, dx_ + 3, dy_ + 2], start=180, end=360,
          fill=WHITE, width=2)
    d.line([dx_ - 3, dy_ + 1, dx_ + 3, dy_ + 1], fill=WHITE, width=1)
    d.point((dx_, dy_), fill=INK)
    d.point((dx_, dy_ - 1), fill=TEAL_L)


def draw_nl_brain(d, rng):
    # brain-shaped core of dark glass on its capsule rim
    d.rectangle([10, 24, 22, 26], fill=STEEL_D)  # capsule rim
    d.line([10, 24, 22, 24], fill=COPPER, width=1)
    blob = [(9, 18), (9, 12), (12, 8), (16, 7), (20, 8), (23, 12),
            (23, 18), (20, 22), (12, 22)]
    d.polygon(blob, fill=(52, 56, 74, 255))
    d.polygon(blob, outline=INK)
    d.line([16, 8, 16, 21], fill=INK, width=1)  # centre fissure
    sulci = [((11, 12), (14, 13), (12, 16)), ((20, 11), (18, 14), (21, 16)),
             ((12, 19), (14, 18)), ((19, 20), (18, 18))]
    for seg in sulci:
        d.line(list(seg), fill=(86, 94, 118, 255), width=1)
    for gx, gy in ((13, 10), (19, 13), (15, 19)):  # anomaly glow
        d.point((gx, gy), fill=TEAL_L)


# ------------------------------------------------------------------- ammo

def finish_marks(d, rng, finish, x0, x1, y0, y1):
    """bp_ soot smudge / reloaded_ scuffs over the case area."""
    if finish == "bp":
        for _ in range(14):
            x = rng.randint(x0, x1)
            y = rng.randint(y0, y1)
            d.point((x, y), fill=SOOT)
        d.line([x0, y0, x1, y0], fill=SOOT, width=1)
    elif finish == "reloaded":
        for _ in range(7):
            x = rng.randint(x0, x1)
            y = rng.randint(y0, y1)
            d.point((x, y), fill=BRASS_L if rng.random() < 0.5 else BRASS_D)


def bullet_shape(d, kind, cx, top, bw=2):
    """Draw the bullet above case mouth y=top; returns nothing."""
    if kind == "fmj":  # pointed copper spitzer
        d.polygon([(cx - bw, top), (cx - bw, top - 4), (cx, top - 8),
                   (cx + bw, top - 4), (cx + bw, top)], fill=COPPER)
        d.line([cx, top - 8, cx, top - 2], fill=COPPER_D, width=1)
    elif kind == "sp":  # soft point: copper with exposed lead tip
        d.polygon([(cx - bw, top), (cx - bw, top - 4), (cx - 1, top - 6),
                   (cx + 1, top - 6), (cx + bw, top - 4), (cx + bw, top)],
                  fill=COPPER)
        d.rectangle([cx - 1, top - 7, cx + 1, top - 6], fill=LEAD)
    elif kind == "jsp":  # jacketed soft point, flatter lead face
        d.rectangle([cx - bw, top - 5, cx + bw, top], fill=COPPER)
        d.rectangle([cx - bw, top - 6, cx + bw, top - 5], fill=LEAD)
        d.line([cx - bw, top - 3, cx - bw, top], fill=COPPER_D, width=1)
    elif kind == "jhp":  # hollow point: flat face with dark cavity
        d.rectangle([cx - bw, top - 5, cx + bw, top], fill=COPPER)
        d.line([cx - bw, top - 5, cx + bw, top - 5], fill=COPPER_D, width=1)
        d.point((cx, top - 5), fill=BLACK)
        d.point((cx, top - 4), fill=LEAD_D)
    elif kind == "copper_pen":  # solid copper penetrator, long point
        d.polygon([(cx - bw, top), (cx - bw, top - 5), (cx, top - 9),
                   (cx + bw, top - 5), (cx + bw, top)], fill=COPPER)
        d.line([cx - bw, top - 3, cx + bw, top - 3], fill=COPPER_D, width=1)
    elif kind == "ftx":  # red polymer flex tip
        d.rectangle([cx - bw, top - 4, cx + bw, top], fill=COPPER)
        d.polygon([(cx - bw, top - 4), (cx, top - 8), (cx + bw, top - 4)],
                  fill=RED)
        d.point((cx, top - 8), fill=RED_D)
    elif kind == "ap":  # black-tip armour piercing
        d.polygon([(cx - bw, top), (cx - bw, top - 4), (cx, top - 8),
                   (cx + bw, top - 4), (cx + bw, top)], fill=COPPER)
        d.polygon([(cx - 1, top - 5), (cx, top - 8), (cx + 1, top - 5)],
                  fill=BLACK)
    elif kind == "api":  # AP incendiary: silver band + red tip
        d.polygon([(cx - bw, top), (cx - bw, top - 4), (cx, top - 8),
                   (cx + bw, top - 4), (cx + bw, top)], fill=COPPER)
        d.line([cx - 1, top - 4, cx + 1, top - 4], fill=STEEL_L, width=1)
        d.point((cx, top - 7), fill=RED)
        d.point((cx, top - 6), fill=RED)
    elif kind == "tracer":
        d.polygon([(cx - bw, top), (cx - bw, top - 4), (cx, top - 8),
                   (cx + bw, top - 4), (cx + bw, top)], fill=COPPER)
        d.polygon([(cx - 1, top - 5), (cx, top - 8), (cx + 1, top - 5)],
                  fill=ORANGE)
    elif kind == "hpbt":  # hollow-point boat tail: tiny dark tip dot
        d.polygon([(cx - bw, top), (cx - bw, top - 4), (cx, top - 8),
                   (cx + bw, top - 4), (cx + bw, top)], fill=COPPER)
        d.point((cx, top - 8), fill=BLACK)
    elif kind == "lead_rn":  # unjacketed round-nose lead
        d.rectangle([cx - bw, top - 4, cx + bw, top], fill=LEAD)
        d.ellipse([cx - bw, top - 6, cx + bw, top - 2], fill=LEAD)
        d.point((cx - 1, top - 5), fill=LEAD_D)
    elif kind == "conical":  # .22 CB conical ball, short
        d.polygon([(cx - bw, top), (cx, top - 4), (cx + bw, top)], fill=LEAD)
    elif kind == "capsule":  # ratshot: blue fragmenting capsule of shot
        d.rectangle([cx - bw, top - 5, cx + bw, top], fill=BLUE_L)
        d.ellipse([cx - bw, top - 6, cx + bw, top - 3], fill=BLUE_L)
        for px, py in ((cx - 1, top - 4), (cx + 1, top - 3), (cx, top - 2)):
            d.point((px, py), fill=LEAD_D)


def cartridge(d, rng, case_h=13, case_w=3, neck=True, bullet="fmj",
              finish="factory", rim=False, cx=16, base=29):
    top = base - case_h
    brass = BRASS_DULL if finish == "bp" else BRASS
    brass_d = shade(brass, 0.78)
    if rim:
        d.rectangle([cx - case_w - 1, base - 1, cx + case_w + 1, base],
                    fill=brass_d)
    body_base = base - (2 if rim else 0)
    if neck:
        sh = top + 4
        d.polygon([(cx - case_w, body_base), (cx - case_w, sh),
                   (cx - 2, top), (cx + 2, top), (cx + case_w, sh),
                   (cx + case_w, body_base)], fill=brass)
        d.line([cx - case_w, body_base, cx - case_w, sh], fill=brass_d,
               width=1)
        d.line([cx + 1, sh, cx + 1, body_base], fill=BRASS_L, width=1)
    else:
        d.rectangle([cx - case_w, top, cx + case_w, body_base], fill=brass)
        d.line([cx - case_w, top, cx - case_w, body_base], fill=brass_d,
               width=1)
        d.line([cx + case_w - 1, top, cx + case_w - 1, body_base],
               fill=BRASS_L, width=1)
    if finish == "reloaded":  # two-tone case: duller upper half
        mid = (top + body_base) // 2
        if neck:
            d.line([cx - case_w, mid, cx + case_w, mid], fill=brass_d,
                   width=1)
        else:
            d.rectangle([cx - case_w + 1, top, cx + case_w - 1, mid],
                        fill=BRASS_DULL)
    bullet_shape(d, bullet, cx, top, bw=2 if case_w <= 3 else 3)
    finish_marks(d, rng, finish, cx - case_w, cx + case_w, top, body_base)


def shotshell(d, rng, load="bird", cx=16, base=29):
    # 20 gauge: yellow plastic hull, brass head, load shown at the crimp
    top = base - 15
    d.rectangle([cx - 4, top, cx + 4, base - 4], fill=YELLOW_HULL)
    d.line([cx - 4, top, cx - 4, base - 4], fill=YELLOW_D, width=1)
    d.line([cx + 3, top, cx + 3, base - 4], fill=YELLOW_D, width=1)
    d.rectangle([cx - 4, base - 4, cx + 4, base], fill=BRASS)
    d.line([cx - 4, base - 4, cx + 4, base - 4], fill=BRASS_D, width=1)
    d.line([cx - 5, base - 1, cx + 5, base - 1], fill=BRASS_D, width=1)
    if load == "bird":  # folded crimp with fine pellet speckle below
        d.line([cx - 4, top, cx + 4, top], fill=YELLOW_D, width=2)
        d.line([cx, top, cx - 3, top + 2], fill=YELLOW_D, width=1)
        d.line([cx, top, cx + 3, top + 2], fill=YELLOW_D, width=1)
        for px in range(cx - 3, cx + 4, 2):
            d.point((px, top + 4), fill=YELLOW_D)
    elif load == "buck":  # big pellets bulging at the mouth
        for px in (cx - 3, cx, cx + 3):
            d.ellipse([px - 1, top - 1, px + 1, top + 1], fill=LEAD)
            d.point((px, top), fill=LEAD_D)
        d.line([cx - 4, top + 1, cx + 4, top + 1], fill=YELLOW_D, width=1)
    elif load == "slug":  # single slug dome
        d.ellipse([cx - 3, top - 2, cx + 3, top + 2], fill=LEAD)
        d.line([cx - 3, top, cx + 3, top], fill=LEAD_D, width=1)
        d.point((cx - 1, top - 1), fill=STEEL_L)


# ------------------------------------------------------------- speedloaders

def speedloader(d, rng, rounds, big=False):
    cx, cy = 16, 16
    r_out = 10 if not big else 11
    d.ellipse([cx - r_out, cy - r_out, cx + r_out, cy + r_out],
              fill=PLASTIC_D)
    d.ellipse([cx - r_out, cy - r_out, cx + r_out, cy + r_out], outline=INK)
    rr = 3 if not big else 4  # cartridge radius
    ring = 6 if not big else 7
    for i in range(rounds):
        a = 2 * math.pi * i / rounds - math.pi / 2
        x = int(cx + ring * math.cos(a))
        y = int(cy + ring * math.sin(a))
        d.ellipse([x - rr, y - rr, x + rr, y + rr], fill=BRASS)
        d.ellipse([x - rr + 1, y - rr + 1, x + rr - 1, y + rr - 1],
                  fill=COPPER)
        d.point((x, y), fill=LEAD_D)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL)
    d.point((cx, cy), fill=INK)  # centre release knob


# -------------------------------------------------------------- retool kits

def retool(d, rng, thick):
    # barrel assembly lying on an unrolled cloth, bolt head beside it
    d.rectangle([3, 20, 29, 27], fill=(88, 74, 58, 255))  # tool roll cloth
    d.line([3, 20, 29, 20], fill=WOOD_D, width=1)
    y = 14
    h = 4 if thick else 3
    d.rectangle([6, y, 27, y + h], fill=STEEL)  # barrel
    d.line([6, y, 27, y], fill=STEEL_L, width=1)
    d.rectangle([27, y - 1, 29, y + h + 1], fill=STEEL_D)  # muzzle
    d.rectangle([3, y - 2, 7, y + h + 2], fill=STEEL_D)  # chamber
    d.point((5, y + 1), fill=INK)
    # bolt head (small block with lugs) below the chamber
    d.rectangle([4, 22, 8, 25], fill=STEEL_L)
    d.point((4, 22), fill=INK)
    d.point((8, 25), fill=INK)
    # three fasteners in a row on the cloth
    for fx in (13, 18, 23):
        d.point((fx, 24), fill=STEEL_L)
        d.point((fx, 23), fill=INK)
    # caliber band
    d.line([24, y, 24, y + h], fill=(RED if thick else BLUE), width=1)


# ------------------------------------------------------------ hornet's nest

def hornets_nest(d, rng, fill_kind):
    # face of the fat 40mm adapter: 4 .410 shell bases + central primer
    cx, cy = 16, 16
    d.ellipse([cx - 11, cy - 11, cx + 11, cy + 11], fill=STEEL_L)
    d.ellipse([cx - 11, cy - 11, cx + 11, cy + 11], outline=INK)
    d.ellipse([cx - 9, cy - 9, cx + 9, cy + 9], fill=STEEL)
    for dx, dy in ((-5, -5), (5, -5), (-5, 5), (5, 5)):
        x, y = cx + dx, cy + dy
        d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=BRASS)
        d.ellipse([x - 3, y - 3, x + 3, y + 3], outline=BRASS_D)
        if fill_kind == "bird":
            for px, py in ((x - 1, y - 1), (x + 1, y), (x, y + 1)):
                d.point((px, py), fill=LEAD_D)
        elif fill_kind == "scrap":
            for px, py in ((x - 1, y - 1), (x + 1, y - 1), (x, y + 1),
                           (x - 1, y + 1)):
                d.point((px, py), fill=IRON)
        elif fill_kind == "slug":
            d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=LEAD)
            d.point((x, y), fill=LEAD_D)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=BRASS_D)  # primer
    d.point((cx, cy), fill=INK)


# --------------------------------------------------------------------- tail

def draw_aim_game(d, rng):
    # target board with a hit spot, toy laser pistol in front
    d.rectangle([18, 5, 29, 16], fill=WHITE)
    d.rectangle([18, 5, 29, 16], outline=INK)
    d.ellipse([20, 7, 27, 14], outline=RED)
    d.ellipse([22, 9, 25, 12], outline=BLUE)
    d.point((23, 10), fill=RED)  # the hit
    d.line([23, 10, 23, 10], fill=RED, width=1)
    d.rectangle([22, 16, 25, 19], fill=STEEL_D)  # stand
    # laser pistol (bright toy plastic, orange tip)
    d.rectangle([3, 18, 15, 21], fill=BLUE)
    d.point((15, 19), fill=ORANGE)
    d.rectangle([14, 18, 16, 20], fill=ORANGE)
    d.rectangle([5, 21, 8, 26], fill=BLUE_L)  # grip
    d.rectangle([9, 16, 12, 18], fill=BLUE_L)  # sight hump
    d.point((18, 12), fill=RED)  # stray laser dot on the board edge


def draw_mannequin(d, rng):
    cx = 16
    d.ellipse([cx - 3, 3, cx + 3, 9], fill=WOOD)  # head
    d.rectangle([cx - 1, 9, cx + 1, 11], fill=WOOD_D)  # neck
    d.rectangle([cx - 4, 11, cx + 4, 19], fill=WOOD)  # torso
    d.line([cx - 4, 13, cx - 7, 18], fill=WOOD, width=2)  # arms
    d.line([cx + 4, 13, cx + 7, 18], fill=WOOD, width=2)
    d.line([cx - 2, 19, cx - 3, 24], fill=WOOD, width=2)  # legs
    d.line([cx + 2, 19, cx + 3, 24], fill=WOOD, width=2)
    for jx, jy in ((cx - 4, 13), (cx + 4, 13), (cx - 2, 19), (cx + 2, 19)):
        d.point((jx, jy), fill=WOOD_D)  # joints
    d.rectangle([cx - 1, 24, cx + 1, 28], fill=IRON)  # stand pole
    d.rectangle([cx - 5, 28, cx + 5, 29], fill=IRON)  # base


def draw_kickboard(d, rng):
    # foam kickboard: flat board with rounded top corners, seen face-on
    d.polygon([(9, 29), (9, 12), (11, 6), (21, 6), (23, 12), (23, 29)],
              fill=BLUE)
    d.polygon([(9, 29), (9, 12), (11, 6), (21, 6), (23, 12), (23, 29)],
              outline=BLUE_L)
    d.line([11, 10, 21, 10], fill=BLUE_L, width=1)
    d.rectangle([12, 14, 20, 17], fill=BLUE_L)  # handhold / logo patch
    for px, py in ((11, 22), (20, 24), (14, 26)):  # foam texture
        d.point((px, py), fill=BLUE_L)


def draw_altered_badge(d, rng):
    # golden shield badge, lettering scratched out, tall thin figures
    shield = [(16, 4), (24, 6), (24, 14), (20, 21), (16, 24), (12, 21),
              (8, 14), (8, 6)]
    d.polygon(shield, fill=GOLD)
    d.polygon(shield, outline=GOLD_D)
    # two tall thin faceless figures
    for fx in (13, 19):
        d.point((fx, 10), fill=GOLD_D)
        d.line([fx, 11, fx, 17], fill=GOLD_D, width=1)
    # scratched-out lettering: crossed scribble rows
    for y in (19, 21):
        d.line([11, y, 21, y], fill=GOLD_D, width=1)
        d.line([11, y, 14, y - 1], fill=STEEL_L, width=1)
        d.line([18, y - 1, 21, y], fill=STEEL_L, width=1)


def draw_altered_keyring(d, rng):
    # heavy steel ring crowded with keys of shifting design
    d.ellipse([10, 3, 22, 13], outline=STEEL_L, width=2)
    keys = [(9, 12, 7), (13, 13, 9), (17, 13, 6), (21, 12, 8), (24, 10, 6)]
    for kx, ky, ln in keys:
        d.rectangle([kx - 1, ky, kx + 1, ky + 2], fill=GOLD_D)  # bow
        d.line([kx, ky + 2, kx, ky + 2 + ln], fill=STEEL, width=1)
        d.point((kx + 1, ky + 2 + ln - 1), fill=STEEL)  # teeth
        d.point((kx + 1, ky + 2 + ln - 3), fill=STEEL)
    d.point((12, 4), fill=WHITE)  # ring glint


def draw_altered_scissors(d, rng):
    # ornate scissors: crossed blades, bird-decorated handle loops
    d.polygon([(10, 4), (12, 4), (17, 16), (15, 17)], fill=STEEL_L)
    d.polygon([(22, 4), (20, 4), (15, 16), (17, 17)], fill=STEEL)
    d.point((16, 16), fill=INK)  # pivot
    d.ellipse([8, 19, 14, 27], outline=GOLD_D, width=2)  # handle loops
    d.ellipse([18, 19, 24, 27], outline=GOLD_D, width=2)
    # vicious bird motif on the left handle
    d.point((10, 21), fill=RED_D)
    d.line([9, 23, 12, 22], fill=GOLD_D, width=1)
    d.point((13, 21), fill=INK)  # bird eye/beak hint
    d.line([11, 5, 11, 8], fill=WHITE, width=1)  # blade glint


def draw_aquarium(d, rng, x0, x1, y0, y1):
    # empty glass tank, fresh silicone beads along the seams (drying)
    d.rectangle([x0, y0, x1, y1], fill=GLASS)
    d.rectangle([x0, y0, x1, y1], outline=GLASS_D)
    d.line([x0, y0, x1, y0], fill=WHITE, width=2)  # top rim glare
    d.line([x0 + 2, y0 + 3, x0 + 2, y1 - 3], fill=WHITE, width=1)
    # silicone bead dots at corners and vertical seams
    for sx in (x0, x1):
        for sy in range(y0 + 2, y1 - 1, 3):
            d.point((sx, sy), fill=WHITE)
    for sx in range(x0 + 2, x1 - 1, 3):
        d.point((sx, y1), fill=WHITE)
    # a wet smear / drip of uncured silicone
    d.line([x1 - 3, y0 + 2, x1 - 3, y0 + 5], fill=WHITE, width=1)
    d.point((x1 - 3, y0 + 6), fill=WHITE)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "motor_small":
        draw_motor(d, rng, (6, 5))
    elif sid == "motor_enhanced":
        draw_motor(d, rng, (8, 7), accent=True)
    elif sid == "motor_super":
        draw_motor(d, rng, (10, 8))
    elif sid == "motor_train1300":
        draw_motor(d, rng, (11, 9), train=True)
    elif sid == "nl_grav_aware_jig":
        draw_nl_jig(d, rng)
    elif sid == "nl_laser_mass_indexer":
        draw_nl_indexer(d, rng)
    elif sid == "nl_robo_surg_app":
        draw_nl_robo(d, rng)
    elif sid == "nl_surg_int_unit":
        draw_nl_brain(d, rng)
    elif sid == "reloaded_3030":
        cartridge(d, rng, case_h=13, bullet="sp", finish="reloaded", rim=True)
    elif sid == "reloaded_3030_bp":
        cartridge(d, rng, case_h=13, bullet="sp", finish="bp", rim=True)
    elif sid == "reloaded_3030_fmj":
        cartridge(d, rng, case_h=13, bullet="fmj", finish="reloaded",
                  rim=True)
    elif sid == "reloaded_3030_fmj_bp":
        cartridge(d, rng, case_h=13, bullet="fmj", finish="bp", rim=True)
    elif sid == "10mmP":
        cartridge(d, rng, case_h=9, neck=False, bullet="copper_pen")
    elif sid == "10mm_jhp":
        cartridge(d, rng, case_h=9, neck=False, bullet="jhp")
    elif sid == "reloaded_10mmP":
        cartridge(d, rng, case_h=9, neck=False, bullet="copper_pen",
                  finish="reloaded")
    elif sid == "20shot_bird":
        shotshell(d, rng, "bird")
    elif sid == "20shot_buck":
        shotshell(d, rng, "buck")
    elif sid == "20shot_slug":
        shotshell(d, rng, "slug")
    elif sid == "22_cb":
        cartridge(d, rng, case_h=6, case_w=2, neck=False, bullet="conical")
    elif sid == "22_lr":
        cartridge(d, rng, case_h=7, case_w=2, neck=False, bullet="lead_rn")
    elif sid == "22_ratshot":
        cartridge(d, rng, case_h=7, case_w=2, neck=False, bullet="capsule")
    elif sid == "3006":
        cartridge(d, rng, case_h=15, bullet="fmj")
    elif sid == "3006_incendiary":
        cartridge(d, rng, case_h=15, bullet="tracer")
    elif sid == "3006fmj":
        cartridge(d, rng, case_h=15, bullet="ap")
    elif sid == "338lapua_api":
        cartridge(d, rng, case_h=16, case_w=4, bullet="api")
    elif sid == "338lapua_fmjbt":
        cartridge(d, rng, case_h=16, case_w=4, bullet="ap")
    elif sid == "338lapua_hpbt":
        cartridge(d, rng, case_h=16, case_w=4, bullet="hpbt")
    elif sid == "38_speedloader":
        speedloader(d, rng, 7)
    elif sid == "38_speedloader5":
        speedloader(d, rng, 5)
    elif sid == "38_speedloader6":
        speedloader(d, rng, 6)
    elif sid == "40_speedloader6":
        speedloader(d, rng, 6, big=True)
    elif sid == "retool_cz600_223rem":
        retool(d, rng, thick=False)
    elif sid == "retool_cz600_762":
        retool(d, rng, thick=True)
    elif sid == "40x46mm_hornets_nest_410_bird":
        hornets_nest(d, rng, "bird")
    elif sid == "40x46mm_hornets_nest_410_scrap":
        hornets_nest(d, rng, "scrap")
    elif sid == "40x46mm_hornets_nest_410_slug":
        hornets_nest(d, rng, "slug")
    elif sid == "460sw_jhp":
        cartridge(d, rng, case_h=12, case_w=4, neck=False, bullet="jhp",
                  rim=True)
    elif sid == "reloaded_460sw":
        cartridge(d, rng, case_h=12, case_w=4, neck=False, bullet="ftx",
                  finish="reloaded", rim=True)
    elif sid == "reloaded_460sw_jhp":
        cartridge(d, rng, case_h=12, case_w=4, neck=False, bullet="jhp",
                  finish="reloaded", rim=True)
    elif sid == "77mm_arisaka_jsp":
        cartridge(d, rng, case_h=14, bullet="jsp")
    elif sid == "bp_77mm_arisaka_jsp":
        cartridge(d, rng, case_h=14, bullet="jsp", finish="bp")
    elif sid == "reloaded_77mm_arisaka_jsp":
        cartridge(d, rng, case_h=14, bullet="jsp", finish="reloaded")
    elif sid == "aim_game":
        draw_aim_game(d, rng)
    elif sid == "mannequin":
        draw_mannequin(d, rng)
    elif sid == "swimming_kickboard":
        draw_kickboard(d, rng)
    elif sid == "altered_badge":
        draw_altered_badge(d, rng)
    elif sid == "altered_keyring":
        draw_altered_keyring(d, rng)
    elif sid == "altered_scissors":
        draw_altered_scissors(d, rng)
    elif sid == "aquarium_small_drying":
        draw_aquarium(d, rng, 9, 23, 12, 26)
    elif sid == "aquarium_medium_drying":
        draw_aquarium(d, rng, 7, 25, 10, 27)
    elif sid == "aquarium_large_drying":
        draw_aquarium(d, rng, 5, 27, 8, 28)
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
    # contact-sheet preview: 8 cols, 2x scale, dark backdrop
    cols = 8
    rows = (len(IDS) + cols - 1) // cols
    cell = 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1,
                                   (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item86.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item86.png {prev.size}")


if __name__ == "__main__":
    main()
