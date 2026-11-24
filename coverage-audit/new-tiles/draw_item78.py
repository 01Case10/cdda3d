#!/usr/bin/env python3
"""ITEM78: speedloader chutes, battery mods, vehicle tanks/coolers,
gun rail mounts, .22 pistols and .450 Bushmaster ammo -- CDDA Voxel
coverage audit.

Families (47 items):

  Arredondo speedloader chutes (7) -- an aluminum ramp under a
    shotgun receiver hint (barrel line + receiver block), indexing a
    speedloader tube drawn below with one shell segment per round of
    capacity.  The generic chute is unfinished (dull, empty mounting
    holes, paper tag, no tube).  Variants: Benelli SA sleek curved
    ramp (4), Mossberg 500 (6), Mossberg 590 (9, longest tube),
    Mossberg 930 auto-loader (8, gas ring under the barrel),
    Remington 870 hunting (5, wooden forend), 870 breacher (4,
    serrated standoff muzzle ring, short tube).
  Battery mods (7) -- adapter boxes: bionic conversion (circuit
    traces + round SNI connector on a cable), UPS conversion
    (two-prong plug glyph), vehicle battery compartment mod (big
    box, red/black terminal posts), light/medium/heavy battery mods
    (boxes sized by class with 1/2/3 cell windows and a bolt glyph),
    pseudo magazine mod (dashed ghost outline, question mark).
  Tanks and coolers (5) -- refrigerated tanks 60 L / 100 L (white,
    snowflake, latches), heated tank (red band, heat wisps,
    thermometer), small integrated cooler (fan grille, brackets),
    steel vehicle cooler (lid, exhaust pipe stub, bolted feet).
  Vehicle/robot bits (3) -- programmable autopilot (screen with a
    dotted route and waypoint, steering glyph, cables), wooden
    towel hanger with a striped draped towel, MIPIM army
    flashlight-laser module off (dark lenses) and on (lit lens
    with starburst + red laser beam).
  Gun mount hardware (12) -- small rail blocks and brackets drawn
    as themselves: muzzle weight under a barrel stub, 45-degree
    offset grip / sight rail / iron sights, rail laser sight (red
    lens), bayonet lug with ring, side mount plate, sights mounts
    (standard, wide launcher saddle, small pistol plate),
    replaceable stock kit laid out as parts, bottom mount with the
    rail hanging below a barrel hint.
  .22 pistols (6) -- profiles, barrel right: 1911 (long slide,
    wood grips, hammer), Glock 44 (blocky polymer, no hammer),
    Rough Rider single-action revolvers with 6.5" and 9" barrels,
    compact SR22, stainless 10-shot S&W 617 revolver with underlug.
  .450 Bushmaster ammo (6) -- one big straight-wall cartridge:
    FTX (red flex tip) and Xtreme Hunter penetrator (solid copper,
    flat meplat), each in factory finish, black-powder (soot
    dither, dark band) and reloaded (scratches, wobble crimp,
    silver primer).

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "arredondo_chute",
    "arredondo_chute_benelli_sa",
    "arredondo_chute_mossberg_500",
    "arredondo_chute_mossberg_590",
    "arredondo_chute_mossberg_930",
    "arredondo_chute_remington_870",
    "arredondo_chute_remington_870_breacher",
    "battery_bionic",
    "battery_ups",
    "magazine_battery_heavy_mod",
    "magazine_battery_light_mod",
    "magazine_battery_medium_mod",
    "magazine_battery_mod",
    "pseudo_magazine_mod",
    "fridgetank",
    "fridgetank_large",
    "heattank",
    "integrated_cooler",
    "mountable_cooler",
    "programmable_autopilot",
    "towel_hanger",
    "mipim",
    "mipim_on",
    "muzzle_pistol_weight",
    "offset_grip",
    "offset_sight_rail",
    "offset_sights",
    "rail_laser_sight",
    "rail_bayonet_lug",
    "rail_mount",
    "sights_mount",
    "sights_mount_launcher",
    "sights_mount_pistol",
    "stock_mount",
    "underbarrel_mount",
    "1911_22",
    "glock_44",
    "rough_rider_pistol",
    "rough_rider_pistol_9",
    "sr22",
    "sw617",
    "450_ftx",
    "450_penetrator",
    "bp_450_ftx",
    "bp_450_penetrator",
    "reloaded_450_ftx",
    "reloaded_450_penetrator",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
STEEL = (96, 104, 114, 255)
STEEL_D = (58, 64, 72, 255)
STEEL_L = (158, 164, 172, 255)
ALUM = (178, 184, 190, 255)
ALUM_D = (128, 134, 140, 255)
PLASTIC = (44, 46, 50, 255)
POLYMER = (92, 94, 98, 255)
BRASS = (202, 164, 78, 255)
BRASS_D = (158, 122, 54, 255)
COPPER = (188, 112, 66, 255)
RED = (198, 52, 42, 255)
RED_D = (150, 34, 28, 255)
SHELL_RED = (188, 66, 54, 255)
WOOD = (120, 84, 48, 255)
WOOD_L = (156, 112, 66, 255)
WHITE = (233, 236, 238, 255)
TEAL = (46, 138, 130, 255)
GOLD = (226, 188, 92, 255)
YELLOW = (250, 214, 96, 255)
GREEN_L = (110, 220, 130, 255)
ARMY = (76, 82, 72, 255)
SOOT = (84, 70, 48, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def bolt(d, cx, cy, col=YELLOW):
    """Tiny lightning bolt glyph."""
    d.line([cx + 1, cy - 3, cx - 1, cy, cx, cy, cx - 1, cy + 3],
           fill=col, width=1)
    d.point((cx + 1, cy - 3), fill=col)


# ------------------------------------------------------------- chutes

def draw_chute(d, rng, cap, style):
    # barrel + receiver hint along the top
    d.rectangle([13, 3, 30, 5], fill=STEEL_D)
    d.rectangle([3, 4, 14, 10], fill=STEEL, outline=INK)
    d.rectangle([5, 6, 8, 8], fill=STEEL_D)  # ejection port shadow
    if style == "r870":  # wooden hunting forend under the barrel
        d.rectangle([16, 6, 23, 9], fill=WOOD, outline=INK)
    if style == "m930":  # gas-system ring under the barrel
        d.rectangle([20, 6, 23, 8], fill=STEEL_D, outline=INK)
    if style == "breacher":  # serrated standoff ring at the muzzle
        d.rectangle([27, 2, 30, 6], fill=STEEL_D)
        for gx in (28, 29):
            d.line([gx, 2, gx, 6], fill=INK, width=1)
    body = ALUM_D if style == "generic" else ALUM
    # the ramp: a wedge sloping down from the loading port
    d.polygon([(5, 11), (13, 11), (17, 19), (9, 19)], fill=body)
    d.polygon([(5, 11), (13, 11), (17, 19), (9, 19)], outline=INK)
    d.line([7, 12, 11, 18], fill=shade(body, 0.78), width=1)   # guide lip
    d.line([12, 12, 15, 18], fill=shade(body, 0.78), width=1)  # guide lip
    if style == "benelli":  # extra curved lip line, sleeker ramp
        d.line([9, 12, 13, 18], fill=shade(body, 0.88), width=1)
    if style == "generic":
        # unfinished: empty mounting holes + a hanging paper tag
        for hx, hy in ((8, 14), (13, 16)):
            d.ellipse([hx - 1, hy - 1, hx + 1, hy + 1], outline=INK)
        d.line([16, 15, 20, 12], fill=INK, width=1)
        d.polygon([(20, 12), (26, 13), (25, 18), (19, 17)],
                  fill=(238, 232, 214, 255))
        d.polygon([(20, 12), (26, 13), (25, 18), (19, 17)], outline=INK)
        d.line([21, 15, 24, 15], fill=ALUM_D, width=1)
        return
    for sx, sy in ((7, 13), (14, 17)):  # mounting screws
        d.point((sx, sy), fill=INK)
    # speedloader tube below, one shell segment per round of capacity
    segw = 3 if cap < 8 else 2
    x0, x1 = 5, 5 + cap * segw + 2
    d.rectangle([x0, 22, x1, 26], fill=(54, 54, 60, 255), outline=INK)
    for i in range(cap):
        sx = x0 + 1 + i * segw
        d.rectangle([sx, 23, sx + segw - 2, 25], fill=SHELL_RED)
        d.rectangle([sx + segw - 2, 23, sx + segw - 2, 25], fill=BRASS)


# ------------------------------------------------------------ batteries

def batt_shell(d, x0, y0, x1, y1, fill):
    d.rectangle([x0, y0, x1, y1], fill=fill, outline=INK)
    d.line([x0 + 1, y0 + 2, x1 - 1, y0 + 2], fill=shade(fill, 1.18), width=1)


def cell_window(d, x0, y0, x1, y1):
    d.rectangle([x0, y0, x1, y1], fill=(30, 34, 40, 255), outline=INK)
    bolt(d, (x0 + x1) // 2, (y0 + y1) // 2)


def draw_battery(d, rng, sid):
    if sid == "battery_bionic":
        batt_shell(d, 7, 8, 23, 24, (66, 92, 98, 255))
        # circuit traces
        for pts in (((10, 12), (16, 12), (16, 16)), ((10, 20), (14, 20)),
                    ((19, 12), (21, 12), (21, 18)), ((12, 16), (12, 21))):
            d.line(list(pts), fill=GOLD, width=1)
        for px, py in ((10, 12), (16, 16), (21, 18), (12, 21), (19, 12)):
            d.point((px, py), fill=GOLD)
        # cable to a round SNI connector
        d.line([20, 24, 24, 27], fill=INK, width=2)
        d.ellipse([22, 25, 28, 30], fill=STEEL_L, outline=INK)
        d.point((25, 27), fill=TEAL)
    elif sid == "battery_ups":
        batt_shell(d, 7, 8, 23, 24, (84, 88, 96, 255))
        # two-prong plug glyph on a lighter panel
        d.rectangle([11, 12, 19, 20], fill=(196, 200, 204, 255), outline=INK)
        d.rectangle([13, 14, 14, 17], fill=INK)
        d.rectangle([17, 14, 18, 17], fill=INK)
        d.point((15, 19), fill=INK)
        d.line([23, 16, 27, 16], fill=INK, width=2)  # cable stub
        d.line([27, 16, 27, 21], fill=INK, width=2)
    elif sid == "magazine_battery_mod":
        # vehicle-battery sized box with terminal posts
        d.rectangle([10, 5, 12, 8], fill=RED_D)
        d.rectangle([20, 5, 22, 8], fill=STEEL_D)
        d.line([11, 8, 21, 8], fill=INK, width=1)  # carry handle
        batt_shell(d, 5, 8, 27, 26, (70, 74, 80, 255))
        d.rectangle([9, 13, 23, 17], fill=(30, 34, 40, 255), outline=INK)
        bolt(d, 16, 15)
        for px, py in ((8, 22), (24, 22)):
            d.point((px, py), fill=STEEL_L)
    elif sid == "magazine_battery_light_mod":
        batt_shell(d, 10, 10, 22, 25, (104, 110, 118, 255))
        cell_window(d, 13, 14, 19, 21)
    elif sid == "magazine_battery_medium_mod":
        batt_shell(d, 8, 9, 24, 25, (96, 102, 110, 255))
        cell_window(d, 11, 13, 15, 21)
        cell_window(d, 17, 13, 21, 21)
    elif sid == "magazine_battery_heavy_mod":
        d.line([11, 9, 21, 9], fill=INK, width=2)  # carry handle
        batt_shell(d, 6, 9, 26, 26, (78, 84, 92, 255))
        cell_window(d, 9, 13, 13, 21)
        cell_window(d, 14, 13, 18, 21)
        cell_window(d, 19, 13, 23, 21)
        for px, py in ((8, 11), (24, 11), (8, 24), (24, 24)):
            d.point((px, py), fill=STEEL_L)
    elif sid == "pseudo_magazine_mod":
        # ghost adapter: pale fill, dashed outline, question mark
        d.rectangle([8, 10, 24, 25], fill=(206, 208, 210, 255))
        for x in range(8, 25, 3):
            d.line([x, 10, min(x + 1, 24), 10], fill=INK, width=1)
            d.line([x, 25, min(x + 1, 24), 25], fill=INK, width=1)
        for y in range(10, 26, 3):
            d.line([8, y, 8, min(y + 1, 25)], fill=INK, width=1)
            d.line([24, y, 24, min(y + 1, 25)], fill=INK, width=1)
        for px, py in ((15, 14), (16, 13), (18, 13), (19, 14), (19, 16),
                       (18, 17), (17, 18), (17, 19)):
            d.point((px, py), fill=INK)
        d.point((17, 22), fill=INK)


# -------------------------------------------------------- tanks/coolers

def snowflake(d, cx, cy, r):
    for dx, dy in ((0, -1), (0, 1), (-1, 0), (1, 0), (-1, -1), (1, 1),
                   (-1, 1), (1, -1)):
        d.line([cx, cy, cx + dx * r, cy + dy * r], fill=TEAL, width=1)
    d.point((cx, cy), fill=TEAL)


def draw_tank(d, rng, sid):
    if sid in ("fridgetank", "fridgetank_large"):
        big = sid == "fridgetank_large"
        x0, y0, x1, y1 = (4, 6, 28, 27) if big else (6, 8, 26, 26)
        d.rectangle([x0, y0, x1, y1], fill=WHITE, outline=INK)
        d.line([x0, y0 + 3, x1, y0 + 3], fill=ALUM_D, width=1)  # lid seam
        snowflake(d, 16, (y0 + y1) // 2 + 1, 4 if big else 3)
        latches = (11, 21) if big else (16,)
        for lx in latches:
            d.rectangle([lx - 1, y0 + 2, lx + 1, y0 + 5], fill=STEEL_D)
        d.rectangle([x0 + 2, y1 - 2, x0 + 4, y1], fill=STEEL_D)  # feet
        d.rectangle([x1 - 4, y1 - 2, x1 - 2, y1], fill=STEEL_D)
    elif sid == "heattank":
        d.rectangle([6, 8, 26, 26], fill=(216, 208, 198, 255), outline=INK)
        d.line([6, 11, 26, 11], fill=ALUM_D, width=1)
        d.rectangle([6, 15, 26, 19], fill=RED)  # heat band
        d.line([6, 17, 26, 17], fill=RED_D, width=1)
        for wx in (11, 16, 21):  # heat wisps above the tank
            pts = [(wx + (1 if (y // 2) % 2 else -1), y)
                   for y in range(2, 7)]
            d.line(pts, fill=(232, 120, 60, 255), width=1)
        # thermometer motif
        d.line([16, 21, 16, 24], fill=INK, width=1)
        d.ellipse([15, 23, 17, 25], fill=RED, outline=INK)
    elif sid == "integrated_cooler":
        d.rectangle([8, 10, 24, 25], fill=(176, 182, 188, 255), outline=INK)
        for r in (5, 3):  # fan grille
            d.ellipse([16 - r, 17 - r, 16 + r, 17 + r], outline=STEEL_D)
        d.line([11, 17, 21, 17], fill=STEEL_D, width=1)
        d.line([16, 12, 16, 22], fill=STEEL_D, width=1)
        d.point((16, 17), fill=INK)
        for bx in (5, 25):  # mounting brackets
            d.rectangle([bx, 14, bx + 2, 20], fill=STEEL_D)
            d.point((bx + 1, 17), fill=INK)
        d.line([8, 12, 24, 12], fill=ALUM_D, width=1)
    elif sid == "mountable_cooler":
        # exhaust pipe stub on top
        d.polygon([(20, 10), (24, 10), (25, 4), (21, 4)], fill=STEEL_D)
        d.ellipse([21, 3, 25, 5], fill=INK)
        d.rectangle([6, 10, 26, 25], fill=(148, 154, 160, 255), outline=INK)
        d.line([6, 13, 26, 13], fill=STEEL_D, width=1)  # lid seam
        d.rectangle([14, 12, 18, 15], fill=STEEL_D)     # latch
        d.line([10, 20, 22, 20], fill=STEEL_D, width=1)  # handle line
        for fx in (8, 22):  # bolted feet
            d.rectangle([fx, 25, fx + 3, 27], fill=STEEL_D)
            d.point((fx + 1, 26), fill=INK)


# ------------------------------------------------------------ misc bits

def draw_autopilot(d, rng):
    d.rectangle([7, 8, 25, 24], fill=(96, 102, 110, 255), outline=INK)
    d.rectangle([10, 11, 22, 18], fill=(28, 56, 62, 255), outline=INK)
    # dotted route across the screen to an X waypoint
    for i, (px, py) in enumerate(((12, 16), (14, 15), (16, 15), (18, 13))):
        d.point((px, py), fill=GREEN_L)
    d.line([19, 12, 21, 14], fill=GREEN_L, width=1)
    d.line([21, 12, 19, 14], fill=GREEN_L, width=1)
    # steering glyph under the screen
    d.arc([13, 19, 19, 25], start=0, end=360, fill=STEEL_L, width=1)
    d.line([16, 22, 16, 20], fill=STEEL_L, width=1)
    d.line([16, 22, 14, 23], fill=STEEL_L, width=1)
    d.line([16, 22, 18, 23], fill=STEEL_L, width=1)
    d.line([12, 24, 11, 29], fill=INK, width=1)  # cables
    d.line([20, 24, 21, 29], fill=INK, width=1)


def draw_towel_hanger(d, rng):
    d.rectangle([4, 7, 28, 9], fill=WOOD, outline=INK)
    for px in (8, 24):
        d.rectangle([px, 9, px + 1, 11], fill=WOOD)
    # draped towel with a fold and blue stripes
    d.polygon([(11, 9), (21, 9), (22, 23), (19, 24), (16, 23), (13, 24),
               (10, 23)], fill=(238, 238, 240, 255))
    d.polygon([(11, 9), (21, 9), (22, 23), (19, 24), (16, 23), (13, 24),
               (10, 23)], outline=INK)
    d.line([16, 9, 16, 23], fill=ALUM_D, width=1)
    d.line([11, 20, 21, 20], fill=(84, 120, 190, 255), width=1)
    d.line([11, 22, 21, 22], fill=(84, 120, 190, 255), width=1)


def draw_mipim(d, rng, on):
    for x in range(9, 24, 3):  # rail teeth
        d.rectangle([x, 25, x + 1, 27], fill=STEEL_D)
    d.rectangle([8, 10, 24, 25], fill=ARMY, outline=INK)
    d.rectangle([14, 7, 18, 10], fill=STEEL_D)  # top knob
    d.point((11, 13), fill=STEEL_L)             # side switch
    if on:
        d.ellipse([18, 12, 23, 18], fill=YELLOW, outline=INK)
        for ex, ey in ((30, 9), (30, 15), (30, 21)):  # starburst rays
            d.line([23, 15, ex, ey], fill=YELLOW, width=1)
        d.point((29, 15), fill=(255, 244, 190, 255))
        d.ellipse([18, 20, 21, 23], fill=RED, outline=INK)
        d.line([21, 21, 31, 21], fill=RED, width=1)  # laser beam
        d.point((30, 21), fill=(255, 120, 110, 255))
    else:
        d.ellipse([18, 12, 23, 18], fill=(52, 64, 88, 255), outline=INK)
        d.point((20, 14), fill=(210, 220, 235, 255))  # lens glint
        d.ellipse([18, 20, 21, 23], fill=RED_D, outline=INK)


# --------------------------------------------------------------- mounts

def rail_teeth(d, x0, x1, y, col=STEEL_D):
    for x in range(x0, x1, 3):
        d.rectangle([x, y, x + 1, y + 2], fill=col)


def draw_mount(d, rng, sid):
    if sid == "muzzle_pistol_weight":
        d.rectangle([4, 10, 28, 13], fill=STEEL, outline=INK)  # barrel stub
        d.rectangle([20, 13, 28, 20], fill=STEEL_D, outline=INK)
        d.point((24, 16), fill=STEEL_L)
        d.point((24, 18), fill=STEEL_L)
        d.line([6, 16, 16, 16], fill=ALUM_D, width=1)
    elif sid == "offset_grip":
        d.rectangle([5, 7, 21, 10], fill=STEEL_D, outline=INK)
        rail_teeth(d, 6, 20, 5)
        # grip hanging at 45 degrees
        d.line([14, 10, 20, 23], fill=PLASTIC, width=5)
        for gx, gy in ((16, 15), (18, 19)):
            d.point((gx, gy), fill=POLYMER)
        d.rectangle([12, 9, 16, 12], fill=STEEL, outline=INK)
    elif sid == "offset_sight_rail":
        d.rectangle([5, 19, 27, 22], fill=STEEL_D, outline=INK)  # base rail
        rail_teeth(d, 6, 26, 17)
        d.line([10, 19, 20, 9], fill=STEEL, width=3)  # 45-degree riser
        for tx, ty in ((13, 15), (16, 12), (19, 9)):
            d.point((tx, ty), fill=INK)
        d.rectangle([18, 6, 24, 9], fill=STEEL, outline=INK)  # offset rail
        rail_teeth(d, 19, 23, 4)
    elif sid == "offset_sights":
        d.polygon([(6, 22), (26, 22), (20, 14), (12, 14)],
                  fill=STEEL_D)  # 45-degree wedge base
        d.polygon([(6, 22), (26, 22), (20, 14), (12, 14)], outline=INK)
        d.ellipse([9, 8, 14, 13], outline=STEEL_L, width=2)  # rear aperture
        d.rectangle([21, 7, 22, 14], fill=STEEL_L)           # front post
        d.point((21, 6), fill=RED)
    elif sid == "rail_laser_sight":
        rail_teeth(d, 9, 23, 23)
        d.rectangle([8, 13, 24, 23], fill=PLASTIC, outline=INK)
        d.ellipse([19, 15, 23, 19], fill=RED, outline=INK)
        d.point((25, 17), fill=RED)
        d.rectangle([11, 10, 15, 13], fill=STEEL_D)  # top switch
    elif sid == "rail_bayonet_lug":
        d.rectangle([5, 8, 27, 11], fill=STEEL_D, outline=INK)
        rail_teeth(d, 6, 26, 6)
        d.rectangle([21, 11, 23, 16], fill=STEEL)  # lug post
        d.ellipse([19, 16, 25, 22], outline=STEEL_L, width=2)  # barrel ring
    elif sid == "rail_mount":
        d.rectangle([10, 6, 22, 25], fill=POLYMER, outline=INK)  # side plate
        for px, py in ((12, 8), (20, 8), (12, 23), (20, 23)):
            d.point((px, py), fill=INK)  # fasteners
        for y in range(9, 23, 3):  # rail teeth on the outer edge
            d.rectangle([22, y, 24, y + 1], fill=STEEL_D)
        d.line([13, 12, 19, 12], fill=ALUM_D, width=1)
    elif sid == "sights_mount":
        d.rectangle([6, 15, 26, 19], fill=POLYMER, outline=INK)
        rail_teeth(d, 11, 22, 13, col=STEEL)
        for px in (9, 23):
            d.point((px, 17), fill=INK)
        d.line([6, 21, 26, 21], fill=ALUM_D, width=1)
    elif sid == "sights_mount_launcher":
        # wide curved saddle for a launcher tube
        d.arc([4, 12, 28, 30], start=0, end=180, fill=STEEL_D, width=3)
        d.rectangle([8, 10, 24, 14], fill=POLYMER, outline=INK)
        rail_teeth(d, 12, 21, 8, col=STEEL)
        for px in (10, 22):
            d.point((px, 12), fill=INK)
    elif sid == "sights_mount_pistol":
        d.arc([9, 10, 23, 22], start=0, end=180, fill=STEEL_D, width=2)
        d.rectangle([9, 13, 23, 16], fill=POLYMER, outline=INK)
        rail_teeth(d, 13, 20, 11, col=STEEL)
        for px in (11, 21):
            d.point((px, 14), fill=INK)
    elif sid == "stock_mount":
        # kit laid out as parts: bracket, tang bar, screws and washers
        d.polygon([(6, 10), (12, 10), (12, 16), (17, 16), (17, 20),
                   (6, 20)], fill=STEEL, outline=INK)  # L-bracket
        d.rectangle([20, 11, 27, 14], fill=STEEL_D, outline=INK)  # tang bar
        for px, py in ((21, 19), (25, 19), (21, 23), (25, 23)):
            d.ellipse([px - 1, py - 1, px + 1, py + 1], outline=STEEL_L)
        for px, py in ((9, 13), (14, 18)):
            d.point((px, py), fill=INK)
    elif sid == "underbarrel_mount":
        d.rectangle([4, 6, 28, 9], fill=STEEL_D, outline=INK)  # barrel hint
        d.rectangle([9, 9, 23, 13], fill=POLYMER, outline=INK)  # mount plate
        rail_teeth(d, 10, 22, 13, col=STEEL)
        for px in (12, 20):
            d.point((px, 11), fill=INK)
        d.rectangle([14, 15, 18, 18], fill=STEEL_D)  # accessory stub


# -------------------------------------------------------------- pistols

def draw_1911(d, rng):
    d.rectangle([4, 7, 7, 9], fill=STEEL_D)   # rear sight
    d.rectangle([3, 8, 5, 11], fill=STEEL_D)  # hammer
    d.rectangle([4, 9, 28, 13], fill=STEEL, outline=INK)  # slide
    for sx in (6, 8, 10):  # rear serrations
        d.line([sx, 10, sx, 12], fill=STEEL_D, width=1)
    d.point((27, 8), fill=STEEL_D)  # front sight
    d.rectangle([12, 13, 28, 15], fill=STEEL_D)  # frame
    d.line([14, 15, 14, 19], fill=STEEL_D, width=1)  # trigger guard
    d.line([14, 19, 20, 19], fill=STEEL_D, width=1)
    d.line([20, 19, 20, 15], fill=STEEL_D, width=1)
    d.polygon([(5, 13), (12, 13), (10, 26), (4, 26)], fill=STEEL_D)
    d.polygon([(6, 15), (11, 15), (9, 24), (5, 24)], fill=WOOD_L)
    d.rectangle([4, 26, 10, 27], fill=INK)  # magazine base


def draw_glock(d, rng):
    d.rectangle([5, 6, 8, 8], fill=PLASTIC)  # rear sight
    d.rectangle([5, 8, 27, 13], fill=(52, 54, 58, 255), outline=INK)
    for sx in (23, 25):  # front serrations
        d.line([sx, 9, sx, 12], fill=INK, width=1)
    d.rectangle([5, 13, 27, 16], fill=POLYMER)  # polymer frame
    d.rectangle([15, 16, 22, 20], outline=POLYMER)  # squared trigger guard
    d.line([18, 16, 18, 19], fill=INK, width=1)
    d.polygon([(6, 16), (13, 16), (11, 27), (5, 27)], fill=POLYMER)
    for gy in (20, 23):  # finger grooves
        d.line([6, gy, 12, gy], fill=shade(POLYMER, 0.8), width=1)
    d.rectangle([5, 27, 11, 28], fill=PLASTIC)


def draw_revolver(d, rng, barrel_end, stainless=False):
    body = STEEL_L if stainless else STEEL
    dark = STEEL if stainless else STEEL_D
    d.rectangle([15, 10, barrel_end, 12], fill=body, outline=INK)  # barrel
    d.point((barrel_end - 1, 9), fill=dark)  # front sight
    if stainless:  # underlug
        d.rectangle([16, 12, barrel_end - 2, 14], fill=body, outline=INK)
    else:  # ejector rod under the barrel
        d.line([16, 14, barrel_end - 3, 14], fill=dark, width=1)
    cyl_y1 = 18 if stainless else 17
    d.rectangle([10, 9, 16, cyl_y1], fill=body, outline=INK)  # cylinder
    if stainless:
        for px, py in ((12, 11), (14, 11), (12, 14), (14, 14), (13, 16)):
            d.point((px, py), fill=dark)  # chamber hints, 10-shot
    else:
        for fx in (11, 13, 15):  # flutes
            d.line([fx, 10, fx, cyl_y1 - 1], fill=dark, width=1)
    d.rectangle([6, 6, 9, 10], fill=dark)  # hammer
    d.line([8, 9, 16, 9], fill=dark, width=1)  # top strap
    d.line([10, cyl_y1, 10, 20], fill=dark, width=1)  # trigger guard
    d.line([10, 20, 15, 20], fill=dark, width=1)
    if stainless:  # black rubber grip
        d.polygon([(5, 15), (11, 15), (10, 22), (8, 27), (4, 26), (3, 19)],
                  fill=PLASTIC)
        for gy in (19, 22):
            d.line([5, gy, 9, gy], fill=POLYMER, width=1)
    else:  # plow-handle wooden grip
        d.polygon([(4, 13), (10, 13), (9, 20), (7, 26), (4, 25), (3, 18)],
                  fill=WOOD)
        d.polygon([(4, 13), (10, 13), (9, 20), (7, 26), (4, 25), (3, 18)],
                  outline=INK)


def draw_sr22(d, rng):
    d.rectangle([8, 8, 11, 10], fill=STEEL_D)  # rear sight
    d.rectangle([8, 10, 25, 14], fill=STEEL, outline=INK)  # short slide
    d.line([22, 11, 22, 13], fill=STEEL_D, width=1)
    d.rectangle([8, 14, 25, 16], fill=STEEL_D)
    d.line([15, 16, 15, 19], fill=STEEL_D, width=1)
    d.line([15, 19, 20, 19], fill=STEEL_D, width=1)
    d.polygon([(9, 16), (15, 16), (13, 25), (8, 25)], fill=PLASTIC)
    d.rectangle([8, 25, 13, 26], fill=INK)


# ----------------------------------------------------------------- ammo

def draw_450(d, rng, tip, finish):
    # one big straight-wall cartridge, standing upright
    d.rectangle([12, 26, 20, 28], fill=BRASS_D, outline=INK)  # rim
    case = BRASS
    d.rectangle([13, 15, 19, 26], fill=case, outline=INK)
    if tip == "ftx":
        d.polygon([(13, 15), (19, 15), (17, 9), (15, 9)], fill=COPPER)
        d.polygon([(13, 15), (19, 15), (17, 9), (15, 9)], outline=INK)
        d.polygon([(15, 9), (17, 9), (16, 4)], fill=RED)  # flex tip
        d.polygon([(15, 9), (17, 9), (16, 4)], outline=INK)
    else:  # solid-copper penetrator, flat meplat
        d.polygon([(13, 15), (19, 15), (18, 6), (14, 6)], fill=COPPER)
        d.polygon([(13, 15), (19, 15), (18, 6), (14, 6)], outline=INK)
        d.point((16, 6), fill=INK)
    if finish == "bp":
        d.line([13, 20, 19, 20], fill=SOOT, width=2)  # fouling band
        for _ in range(26):  # soot dither on the case
            d.point((rng.randint(13, 19), rng.randint(16, 25)), fill=SOOT)
    elif finish == "reloaded":
        for x, y in ((14, 18), (17, 21), (15, 24)):  # scratches
            d.line([x, y, x + 2, y + 1], fill=BRASS_D, width=1)
        d.line([13, 15, 19, 15], fill=INK, width=1)  # wobble crimp
        d.point((16, 27), fill=STEEL_L)  # silver primer


# -------------------------------------------------------------- dispatch

CHUTES = {
    "arredondo_chute": (0, "generic"),
    "arredondo_chute_benelli_sa": (4, "benelli"),
    "arredondo_chute_mossberg_500": (6, "m500"),
    "arredondo_chute_mossberg_590": (9, "m590"),
    "arredondo_chute_mossberg_930": (8, "m930"),
    "arredondo_chute_remington_870": (5, "r870"),
    "arredondo_chute_remington_870_breacher": (4, "breacher"),
}
BATTERIES = {
    "battery_bionic", "battery_ups", "magazine_battery_heavy_mod",
    "magazine_battery_light_mod", "magazine_battery_medium_mod",
    "magazine_battery_mod", "pseudo_magazine_mod",
}
TANKS = {"fridgetank", "fridgetank_large", "heattank",
         "integrated_cooler", "mountable_cooler"}
MOUNTS = {
    "muzzle_pistol_weight", "offset_grip", "offset_sight_rail",
    "offset_sights", "rail_laser_sight", "rail_bayonet_lug", "rail_mount",
    "sights_mount", "sights_mount_launcher", "sights_mount_pistol",
    "stock_mount", "underbarrel_mount",
}
AMMO = {
    "450_ftx": ("ftx", "factory"),
    "450_penetrator": ("pen", "factory"),
    "bp_450_ftx": ("ftx", "bp"),
    "bp_450_penetrator": ("pen", "bp"),
    "reloaded_450_ftx": ("ftx", "reloaded"),
    "reloaded_450_penetrator": ("pen", "reloaded"),
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in CHUTES:
        cap, style = CHUTES[sid]
        draw_chute(d, rng, cap, style)
    elif sid in BATTERIES:
        draw_battery(d, rng, sid)
    elif sid in TANKS:
        draw_tank(d, rng, sid)
    elif sid == "programmable_autopilot":
        draw_autopilot(d, rng)
    elif sid == "towel_hanger":
        draw_towel_hanger(d, rng)
    elif sid in ("mipim", "mipim_on"):
        draw_mipim(d, rng, sid == "mipim_on")
    elif sid in MOUNTS:
        draw_mount(d, rng, sid)
    elif sid == "1911_22":
        draw_1911(d, rng)
    elif sid == "glock_44":
        draw_glock(d, rng)
    elif sid == "rough_rider_pistol":
        draw_revolver(d, rng, 29)
    elif sid == "rough_rider_pistol_9":
        draw_revolver(d, rng, 31)
    elif sid == "sr22":
        draw_sr22(d, rng)
    elif sid == "sw617":
        draw_revolver(d, rng, 29, stainless=True)
    elif sid in AMMO:
        tip, finish = AMMO[sid]
        draw_450(d, rng, tip, finish)
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
    # contact-sheet preview: 8 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item78.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item78.png {prev.size}")


if __name__ == "__main__":
    main()
