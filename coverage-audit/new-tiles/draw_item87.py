#!/usr/bin/env python3
"""ITEM87: gun parts, bio gear, surgical bots, atlatl spears, 20ga
shotguns, fasteners, foods, workshop tools, preservation manuals,
rigs, rocket candy + rifle ammo -- CDDA Voxel coverage audit.

Art grounded in each id's game entry:
  barrels: ported (gas ports at the muzzle) vs shortened (stubby);
  upstest is a test-only mod -> checker tag like the ITEM76 debug
  devices.  Bio pseudo-guns are fleshy magenta/cyan growths.
  Surgical bonebots: bone-white rounded medics; the intact one is
  merely inactive (dark visor), the two broken ones are slumped /
  tipped with cracked shells.  Atlatl spears: long fletched darts
  beside their thrower stick, tip by material.  20ga shotguns in
  profile by action (over-under, pump, bolt + detachable mag).
  Masonry saw off vs on (blur disc + sparks).  Nomad harness off vs
  on (its built-in flashlight lit).  Rocket candy: pear-shaped
  green chunk, primed (lit narrow end), active (flame + smoke).
  Ammo upright, profile by caliber, tip by load (M995/M993 black
  AP tips, JHP cavity, +P solid copper, SP exposed lead).

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "barrel_ported",
    "barrel_small",
    "upstest",
    "bio_emp_gun",
    "mut_longpull",
    "mut_quills",
    "bot_nl_surgicalbot",
    "broken_nl_surgicalbot_1_1",
    "broken_nl_surgicalbot_assist",
    "bronze_atlatl_spear",
    "steel_atlatl_spear",
    "wood_atlatl_spear",
    "browning_citori_20",
    "remington_870_20",
    "savage_model_220",
    "buckle_steel",
    "snapfastener_steel",
    "touch_fastener",
    "coconut_water",
    "pekmez",
    "salca",
    "dry_mushroom",
    "mushroom_cooked",
    "mushroom_ketchup",
    "electric_masonrysaw_off",
    "electric_masonrysaw_on",
    "grinder_blade",
    "forgerig",
    "kilnrig",
    "weldkit",
    "knife_sharpener_handheld",
    "lajatang",
    "smart_watch",
    "manual_canning",
    "manual_sealing",
    "vacuum_sealing",
    "nomad_rig",
    "nomad_rig_on",
    "survivor_rig",
    "tool_rocket_candy",
    "tool_rocket_candy_act",
    "tool_rocket_candy_primed",
    "223",
    "556_m995",
    "308",
    "762_51_m993",
    "44fmj",
    "44magnum",
    "450_ar15mag",
    "ruger_arr_mag",
    "4570_pen",
    "4570_sp",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
STEEL = (168, 172, 180, 255)
STEEL_D = (104, 108, 116, 255)
STEEL_DD = (66, 68, 76, 255)
GUNMETAL = (84, 88, 98, 255)
BRASS = (196, 156, 74, 255)
BRASS_D = (150, 114, 48, 255)
COPPER = (198, 118, 66, 255)
LEAD = (148, 148, 154, 255)
WOOD = (122, 84, 48, 255)
WOOD_D = (92, 62, 34, 255)
WOOD_L = (164, 122, 76, 255)
BRONZE = (176, 124, 52, 255)
BONE = (228, 222, 206, 255)
BONE_D = (192, 186, 168, 255)
FLESH = (206, 92, 168, 255)
FLESH_D = (160, 58, 128, 255)
CYAN = (72, 200, 210, 255)
BLUE = (52, 92, 178, 255)
RED = (196, 52, 44, 255)
ORANGE = (232, 118, 40, 255)
FLAME = (250, 196, 72, 255)
SMOKE = (168, 168, 174, 255)
GREEN = (86, 148, 74, 255)
BLACKTIP = (30, 30, 34, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- barrels

def draw_barrel(d, rng, ported):
    y0, y1 = 13, 18
    x0, x1 = (3, 29) if ported else (7, 26)
    d.rectangle([x0, y0, x1, y1], fill=STEEL)
    d.line([x0, y0, x1, y0], fill=shade(STEEL, 1.25), width=1)
    d.line([x0, y1, x1, y1], fill=STEEL_D, width=1)
    d.rectangle([x0, y0 - 1, x0 + 3, y1 + 1], fill=STEEL_D)  # breech block
    d.rectangle([x1 - 2, y0 - 1, x1, y1 + 1], fill=STEEL_DD)  # muzzle
    if ported:  # gas ports on top near the muzzle + faint gas wisps
        for px in range(x1 - 9, x1 - 2, 3):
            d.point((px, y0 - 1), fill=INK)
            d.point((px, y0 - 2), fill=SMOKE)
            d.point((px + 1, y0 - 3), fill=shade(SMOKE, 1.15))
    else:  # shortened: heavier profile, crown ring
        d.rectangle([x0 + 4, y0 - 1, x0 + 5, y1 + 1], fill=STEEL_D)
        d.rectangle([x1 - 5, y0 - 1, x1 - 4, y1 + 1], fill=STEEL_D)


def draw_upstest(d, rng):
    # barrel-like steel test rig with a UPS cable and lightning tag
    d.rectangle([4, 13, 24, 18], fill=STEEL)
    d.line([4, 13, 24, 13], fill=shade(STEEL, 1.25), width=1)
    d.rectangle([22, 12, 27, 19], fill=STEEL_DD)
    d.line([24, 19, 28, 24], fill=INK, width=1)  # cable
    d.rectangle([26, 24, 29, 27], fill=INK)      # UPS plug block
    d.polygon([(9, 20), (13, 20), (11, 23), (14, 23), (8, 28), (9, 24),
               (7, 24)], fill=FLAME)            # lightning bolt
    for i, c in enumerate((INK, (240, 240, 240, 255))):  # checker tag
        d.rectangle([4 + (i % 2) * 2, 6 + (i // 2) * 2, 5 + (i % 2) * 2,
                     7 + (i // 2) * 2], fill=c)
        d.rectangle([6 - (i % 2) * 2, 8 - (i // 2) * 2, 7 - (i % 2) * 2,
                     9 - (i // 2) * 2], fill=c)


# ---------------------------------------------------------------- bio gear

def draw_emp_gun(d, rng):
    # organic projector: fleshy stock, cyan coil rings, arcing muzzle
    d.polygon([(4, 20), (12, 16), (14, 22), (8, 26), (4, 24)], fill=FLESH_D)
    d.rectangle([11, 15, 24, 20], fill=FLESH)
    for x in (15, 19, 23):  # coil rings
        d.line([x, 14, x, 21], fill=CYAN, width=1)
    d.polygon([(24, 16), (29, 17), (24, 19)], fill=CYAN)  # emitter tip
    for (x, y) in ((28, 12), (30, 15), (28, 22)):  # electric arcs
        d.line([26, 17, x, y], fill=(170, 240, 250, 255), width=1)
    d.point((13, 23), fill=CYAN)


def draw_longpull(d, rng):
    # ranged pulling implement: fleshy arm ending in a gripping hook
    pts = [(4, 24), (10, 20), (16, 19), (21, 15), (25, 14)]
    d.line(pts, fill=FLESH, width=3)
    d.line(pts, fill=FLESH_D, width=1)
    d.arc([22, 10, 29, 17], start=300, end=140, fill=STEEL, width=2)  # hook
    d.point((25, 10), fill=STEEL)
    for (x, y) in ((8, 22), (14, 19), (19, 16)):  # knuckle bumps
        d.point((x, y - 2), fill=FLESH_D)


def draw_quills(d, rng):
    # assault barbs: a fleshy pad bristling with barbed quills
    d.ellipse([6, 18, 20, 28], fill=FLESH_D)
    d.ellipse([8, 19, 18, 26], fill=FLESH)
    for (x0, y0, x1, y1) in ((10, 19, 6, 8), (14, 18, 13, 5), (17, 19, 21, 7),
                             (12, 20, 9, 10), (16, 20, 24, 12), (19, 21, 27, 16)):
        d.line([x0, y0, x1, y1], fill=BONE, width=1)
        d.point(((x0 + x1) // 2, (y0 + y1) // 2), fill=FLESH_D)  # barb notch
        d.point((x1, y1), fill=INK)


# ---------------------------------------------------------------- bonebots

def bonebot_body(d, rng, broken, tipped):
    if tipped:  # lying on its side, shell cracked open
        d.ellipse([5, 16, 25, 28], fill=BONE)
        d.ellipse([5, 16, 25, 28], outline=BONE_D)
        d.line([9, 19, 15, 23, 13, 27], fill=STEEL_DD, width=1)  # crack
        d.rectangle([22, 19, 28, 24], fill=BONE_D)  # head unit, dark
        d.point((26, 21), fill=INK)
        d.line([8, 16, 5, 11], fill=BONE_D, width=2)  # limp arm up
        for (x, y) in ((18, 12), (21, 9), (24, 13)):  # sparks
            d.point((x, y), fill=FLAME)
            d.point((x + 1, y + 1), fill=ORANGE)
        return
    # upright rounded torso on a hover/base skirt
    d.polygon([(10, 27), (22, 27), (24, 29), (8, 29)], fill=STEEL_D)
    d.ellipse([8, 9, 24, 28], fill=BONE)
    d.ellipse([8, 9, 24, 28], outline=BONE_D)
    d.rectangle([12, 5, 20, 11], fill=BONE)  # head dome
    d.rectangle([12, 5, 20, 11], outline=BONE_D)
    if broken:  # cracked shell, one arm dangling
        d.line([11, 13, 15, 17, 13, 21], fill=STEEL_DD, width=1)
        d.rectangle([13, 7, 19, 9], fill=INK)  # dead visor
        d.line([9, 14, 5, 23], fill=BONE_D, width=2)
        d.line([23, 14, 26, 19], fill=BONE_D, width=2)
        d.point((26, 20), fill=STEEL_D)  # dropped tool
        d.point((18, 24), fill=FLAME)    # spark
        d.point((19, 23), fill=ORANGE)
    else:  # intact but inactive: dark visor, arms folded with tools
        d.rectangle([13, 7, 19, 9], fill=(52, 84, 108, 255))
        d.line([9, 14, 6, 20], fill=BONE_D, width=2)
        d.line([23, 14, 26, 20], fill=BONE_D, width=2)
        d.point((6, 21), fill=STEEL)   # scalpel glint
        d.point((26, 21), fill=STEEL)  # clamp glint
        d.rectangle([14, 15, 18, 17], fill=(214, 60, 54, 255))  # med cross
        d.rectangle([15, 14, 17, 18], fill=(214, 60, 54, 255))


# ---------------------------------------------------------------- atlatl

def draw_atlatl(d, rng, tip_col, tip_metal):
    # long fletched dart diagonal, thrower stick along the bottom
    d.line([6, 26, 24, 8], fill=WOOD_L, width=2)
    d.line([6, 26, 24, 8], fill=WOOD_D, width=1)
    if tip_metal:  # leaf-shaped metal tip
        d.polygon([(24, 8), (29, 3), (27, 10), (25, 11)], fill=tip_col)
        d.polygon([(24, 8), (29, 3), (27, 10), (25, 11)], outline=INK)
    else:  # sharpened wood point
        d.polygon([(24, 8), (29, 3), (26, 10)], fill=WOOD_L)
    for (x, y) in ((7, 24), (9, 22)):  # fletching feathers
        d.polygon([(x, y), (x - 2, y - 3), (x + 2, y - 1)], fill=(214, 84, 60, 255))
    d.line([5, 29, 27, 29], fill=WOOD, width=2)  # atlatl thrower
    d.point((26, 28), fill=WOOD_D)  # spur cup
    d.point((5, 28), fill=WOOD_D)   # grip wrap


# ---------------------------------------------------------------- shotguns

def shotgun_base(d, rng, butt=(WOOD, WOOD_D)):
    d.polygon([(2, 19), (9, 17), (10, 21), (4, 24), (2, 22)], fill=butt[0])
    d.polygon([(2, 19), (9, 17), (10, 21), (4, 24), (2, 22)], outline=butt[1])
    d.rectangle([9, 16, 17, 20], fill=GUNMETAL)  # receiver
    d.line([9, 16, 17, 16], fill=STEEL_D, width=1)


def draw_citori(d, rng):
    shotgun_base(d, rng)
    d.rectangle([17, 15, 30, 17], fill=STEEL_DD)  # over barrel
    d.rectangle([17, 17, 30, 19], fill=STEEL_DD)  # under barrel
    d.point((30, 16), fill=INK)
    d.point((30, 18), fill=INK)
    d.rectangle([17, 19, 23, 21], fill=WOOD_L)  # forend
    d.point((24, 14), fill=BRASS)  # bead sight


def draw_870(d, rng):
    shotgun_base(d, rng)
    d.rectangle([17, 15, 30, 18], fill=STEEL_DD)  # barrel
    d.rectangle([17, 18, 29, 20], fill=STEEL_D)   # mag tube
    d.rectangle([19, 19, 24, 22], fill=WOOD)      # pump forend
    for x in range(20, 24):
        d.line([x, 19, x, 22], fill=WOOD_D, width=1)
    d.point((30, 14), fill=BRASS)


def draw_220(d, rng):
    shotgun_base(d, rng, butt=((70, 72, 78, 255), STEEL_DD))  # synthetic stock
    d.rectangle([17, 15, 30, 18], fill=STEEL_DD)  # rifled barrel
    d.arc([14, 13, 19, 18], start=270, end=60, fill=STEEL, width=1)  # bolt
    d.point((18, 13), fill=STEEL)
    d.rectangle([13, 20, 16, 25], fill=INK)  # detachable box magazine
    d.point((30, 14), fill=BRASS)


# ---------------------------------------------------------------- fasteners

def draw_buckle(d, rng):
    d.rectangle([8, 9, 24, 23], outline=STEEL_D, width=3)
    d.rectangle([8, 9, 24, 23], outline=shade(STEEL, 1.2))
    d.line([16, 10, 16, 18], fill=STEEL, width=2)  # prong
    d.point((16, 19), fill=STEEL_D)
    d.line([8, 16, 24, 16], fill=STEEL_D, width=1)  # centre bar


def draw_snap(d, rng):
    for cx, cy in ((11, 12), (21, 12), (11, 21), (21, 21)):
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=STEEL)
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], outline=STEEL_D)
    d.ellipse([9, 10, 13, 14], fill=STEEL_D)   # socket hollow
    d.ellipse([19, 10, 23, 14], fill=shade(STEEL, 1.25))  # stud dome
    d.point((21, 12), fill=INK)
    d.point((11, 21), fill=STEEL_D)
    d.point((21, 21), fill=STEEL_D)


def draw_touch(d, rng):
    d.rectangle([5, 9, 27, 14], fill=(88, 88, 94, 255))  # hook strip
    for x in range(7, 26, 3):
        for y in (10, 12):
            d.point((x, y), fill=INK)
    d.rectangle([5, 18, 27, 23], fill=(120, 120, 128, 255))  # loop strip
    for x in range(7, 26, 3):
        for y in (19, 21):
            d.point((x, y), fill=(168, 168, 176, 255))


# ---------------------------------------------------------------- foods

def jar(d, rng, fill, cap=(150, 150, 156, 255), label=None):
    d.rectangle([10, 8, 22, 9], fill=cap)  # lid
    d.rectangle([9, 10, 23, 28], fill=(216, 222, 224, 255))  # glass
    d.rectangle([10, 12, 22, 27], fill=fill)
    d.line([11, 12, 11, 26], fill=(240, 244, 244, 255), width=1)  # glint
    if label:
        d.rectangle([13, 17, 19, 22], fill=label)


def draw_coconut(d, rng):
    d.ellipse([7, 12, 25, 29], fill=(96, 66, 40, 255))  # shell
    d.ellipse([7, 12, 25, 29], outline=WOOD_D)
    d.ellipse([9, 10, 23, 16], fill=(238, 240, 238, 255))  # white flesh rim
    d.ellipse([11, 11, 21, 15], fill=(214, 232, 234, 255))  # cloudy water
    d.line([18, 11, 23, 4], fill=(226, 84, 84, 255), width=2)  # straw
    d.point((13, 20), fill=WOOD_D)
    d.point((19, 24), fill=WOOD_D)


def draw_mushrooms(d, rng, cooked):
    if cooked:  # glossy slices in a small dark pan
        d.ellipse([5, 14, 27, 27], fill=(58, 58, 64, 255))
        d.ellipse([7, 15, 25, 25], fill=(70, 62, 54, 255))
        cols = [(168, 128, 84, 255), (188, 148, 100, 255)]
    else:  # dry shrivelled pile, no pan
        cols = [(146, 106, 66, 255), (128, 92, 54, 255)]
    for i in range(7):
        x = rng.randint(8, 21)
        y = rng.randint(16, 22) if cooked else rng.randint(13, 23)
        c = cols[i % 2]
        d.arc([x, y, x + 6, y + 5], start=180, end=360, fill=c, width=2)
        d.line([x + 3, y + 3, x + 3, y + 5], fill=shade(c, 0.8), width=1)
        if cooked:
            d.point((x + 2, y + 1), fill=(240, 230, 200, 255))  # oil glint


# ---------------------------------------------------------------- tools

def saw_body(d, rng):
    d.rectangle([6, 8, 20, 16], fill=BLUE)  # motor housing
    d.rectangle([6, 8, 20, 16], outline=shade(BLUE, 0.7))
    d.rectangle([16, 16, 21, 21], fill=shade(BLUE, 0.8))  # grip neck
    d.rectangle([4, 5, 12, 8], fill=INK)  # handle
    d.point((9, 11), fill=FLAME)  # switch


def draw_saw(d, rng, on):
    saw_body(d, rng)
    cx, cy = 19, 21
    if on:  # blurred spinning disc + sparks
        for r in (9, 7, 5):
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      outline=shade(STEEL, 0.9 + r * 0.02))
        for (x, y) in ((27, 26), (29, 24), (26, 29), (30, 28)):
            d.point((x, y), fill=FLAME)
            d.point((x, y - 1), fill=ORANGE)
    else:  # still disc with teeth
        d.ellipse([cx - 9, cy - 9, cx + 9, cy + 9], fill=STEEL)
        d.ellipse([cx - 9, cy - 9, cx + 9, cy + 9], outline=STEEL_DD)
        for a in range(0, 360, 30):
            import math
            x = cx + int(9 * math.cos(math.radians(a)))
            y = cy + int(9 * math.sin(math.radians(a)))
            d.point((x, y), fill=STEEL_DD)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_DD)


def draw_grinder_blade(d, rng):
    cx, cy = 16, 16
    d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], fill=(92, 92, 98, 255))
    d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], outline=STEEL_DD)
    d.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], outline=(120, 120, 128, 255))
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=(24, 24, 28, 255))
    d.point((cx - 5, cy - 6), fill=(150, 150, 158, 255))  # abrasive glint


def draw_forgerig(d, rng):
    d.rectangle([5, 10, 27, 28], fill=(96, 88, 84, 255))  # cabinet
    d.rectangle([5, 10, 27, 28], outline=INK)
    d.rectangle([8, 13, 17, 21], fill=(50, 40, 36, 255))  # forge chamber
    d.rectangle([10, 16, 15, 20], fill=ORANGE)  # hot interior
    d.point((12, 18), fill=FLAME)
    for y in (13, 18, 23):  # tool storage drawers
        d.rectangle([20, y, 25, y + 3], outline=STEEL_D)
        d.point((22, y + 1), fill=STEEL)


def draw_kilnrig(d, rng):
    d.rectangle([6, 8, 26, 28], fill=(150, 138, 128, 255))  # insulated body
    d.rectangle([6, 8, 26, 28], outline=INK)
    d.ellipse([10, 12, 22, 24], fill=(110, 100, 92, 255))  # round door
    d.ellipse([10, 12, 22, 24], outline=STEEL_DD)
    d.point((16, 18), fill=INK)  # door hub
    d.ellipse([22, 9, 26, 13], outline=RED)  # temp dial
    d.point((24, 11), fill=RED)


def draw_weldkit(d, rng):
    d.rectangle([4, 12, 18, 24], fill=(178, 52, 44, 255))  # welder box
    d.rectangle([4, 12, 18, 24], outline=INK)
    d.rectangle([6, 14, 10, 17], fill=(40, 40, 44, 255))  # display
    d.point((14, 15), fill=FLAME)  # dial
    d.arc([17, 10, 27, 20], start=300, end=140, fill=INK, width=1)  # cable
    d.line([24, 17, 27, 21], fill=STEEL_D, width=2)  # torch
    d.rectangle([19, 22, 29, 27], fill=(196, 170, 130, 255))  # blanket roll
    d.line([19, 24, 29, 24], fill=(170, 144, 104, 255), width=1)


def draw_sharpener(d, rng):
    d.polygon([(8, 22), (24, 22), (26, 26), (6, 26)], fill=STEEL)
    d.polygon([(8, 22), (24, 22), (26, 26), (6, 26)], outline=STEEL_D)
    for x in (12, 16, 20):  # grit slots
        d.line([x, 22, x + 1, 18], fill=INK, width=1)
        d.line([x + 2, 22, x + 1, 18], fill=INK, width=1)
    d.rectangle([13, 12, 19, 17], fill=WOOD)  # handle stub


# ---------------------------------------------------------------- melee/watch

def draw_lajatang(d, rng):
    # long pole, curved blades on BOTH ends
    d.line([8, 24, 24, 8], fill=WOOD, width=2)
    d.line([8, 24, 24, 8], fill=WOOD_D, width=1)
    d.arc([21, 2, 30, 11], start=90, end=260, fill=STEEL, width=2)   # top blade
    d.arc([2, 21, 11, 30], start=270, end=80, fill=STEEL, width=2)   # bottom blade
    d.point((16, 16), fill=WOOD_D)  # centre wrap


def draw_watch(d, rng):
    d.rectangle([13, 3, 19, 9], fill=(60, 60, 66, 255))   # strap top
    d.rectangle([13, 23, 19, 29], fill=(60, 60, 66, 255))  # strap bottom
    d.rectangle([10, 9, 22, 23], fill=STEEL)  # case
    d.rectangle([10, 9, 22, 23], outline=STEEL_D)
    d.rectangle([12, 11, 20, 21], fill=(36, 60, 84, 255))  # lit touchscreen
    d.line([13, 14, 19, 14], fill=CYAN, width=1)   # time bar
    d.point((14, 17), fill=(120, 220, 130, 255))   # fitness heart dot
    d.line([16, 19, 19, 19], fill=(200, 220, 230, 255), width=1)
    d.point((22, 15), fill=STEEL_D)  # crown


# ---------------------------------------------------------------- manuals

def book(d, rng, cover, motif):
    d.rectangle([8, 4, 25, 28], fill=shade(cover, 0.75))  # back edge
    d.rectangle([6, 3, 23, 27], fill=cover)
    d.rectangle([6, 3, 23, 27], outline=INK)
    d.line([9, 3, 9, 27], fill=shade(cover, 0.7), width=1)  # spine seam
    motif(d)


def draw_manual_canning(d, rng):
    def motif(dd):
        dd.ellipse([13, 9, 20, 17], outline=(240, 238, 228, 255))  # jar
        dd.rectangle([13, 8, 20, 9], fill=(240, 238, 228, 255))    # jar lid
        dd.ellipse([15, 12, 18, 15], fill=(226, 84, 60, 255))      # preserves
        dd.line([12, 21, 21, 21], fill=(240, 238, 228, 255), width=1)
        dd.line([12, 24, 19, 24], fill=(240, 238, 228, 255), width=1)
    book(d, rng, (178, 66, 52, 255), motif)


def draw_manual_sealing(d, rng):
    def motif(dd):
        dd.rectangle([13, 10, 20, 18], outline=(240, 238, 228, 255))  # tin can
        dd.line([13, 12, 20, 12], fill=(240, 238, 228, 255), width=1)  # can rim
        dd.line([13, 16, 20, 16], fill=(240, 238, 228, 255), width=1)
        dd.line([12, 21, 21, 21], fill=(240, 238, 228, 255), width=1)
        dd.line([12, 24, 19, 24], fill=(240, 238, 228, 255), width=1)
    book(d, rng, (84, 110, 140, 255), motif)


def draw_vacuum_sealing(d, rng):
    def motif(dd):
        dd.polygon([(13, 9), (20, 9), (19, 18), (14, 18)],
                   outline=(240, 238, 228, 255))  # vacuum bag, sucked in
        dd.point((16, 13), fill=(240, 238, 228, 255))
        dd.line([12, 21, 21, 21], fill=(240, 238, 228, 255), width=1)
        dd.line([12, 24, 19, 24], fill=(240, 238, 228, 255), width=1)
    book(d, rng, (72, 130, 110, 255), motif)


# ---------------------------------------------------------------- rigs

def harness(d, rng, straps, pouch, lit):
    # torso harness: two shoulder straps, belt, pouches
    d.line([10, 5, 10, 24], fill=straps, width=3)
    d.line([22, 5, 22, 24], fill=straps, width=3)
    d.rectangle([7, 22, 25, 25], fill=shade(straps, 0.8))  # belt
    for (x, y, w, h) in ((7, 12, 5, 6), (20, 12, 5, 6), (13, 16, 6, 6)):
        d.rectangle([x, y, x + w, y + h], fill=pouch)
        d.rectangle([x, y, x + w, y + h], outline=shade(pouch, 0.65))
        d.line([x, y + 2, x + w, y + 2], fill=shade(pouch, 0.65), width=1)
    if lit:  # chest flashlight on: lamp + beam
        d.rectangle([14, 6, 18, 9], fill=INK)
        d.point((16, 7), fill=FLAME)
        d.polygon([(15, 9), (17, 9), (21, 15), (11, 15)], fill=(250, 240, 180, 255))
    else:
        d.rectangle([14, 6, 18, 9], fill=INK)  # dark flashlight body
        d.point((16, 7), fill=STEEL_D)


def draw_survivor_rig(d, rng):
    # thigh + stomach rigging only (worn with a vest): two thigh panels
    for x0 in (6, 19):
        d.rectangle([x0, 14, x0 + 7, 27], fill=(74, 76, 70, 255))
        d.rectangle([x0, 14, x0 + 7, 27], outline=INK)
        d.line([x0, 12, x0 + 3, 14], fill=(74, 76, 70, 255), width=2)  # strap
        for y in (17, 22):
            d.rectangle([x0 + 1, y, x0 + 6, y + 3], outline=WOOD_D)
    d.rectangle([10, 6, 22, 11], fill=WOOD)  # stomach pouch band
    d.rectangle([10, 6, 22, 11], outline=WOOD_D)
    d.line([10, 8, 22, 8], fill=WOOD_D, width=1)


# ---------------------------------------------------------------- rocket candy

def draw_rocket_candy(d, rng, state):
    # pear-shaped chunk: wide base, narrow top end (the lit end)
    d.polygon([(11, 28), (21, 28), (20, 16), (17, 9), (15, 9), (12, 16)],
              fill=(196, 178, 120, 255))
    d.polygon([(11, 28), (21, 28), (20, 16), (17, 9), (15, 9), (12, 16)],
              outline=(150, 132, 84, 255))
    d.point((14, 22), fill=(170, 152, 96, 255))
    d.point((18, 25), fill=(170, 152, 96, 255))
    if state == "primed":  # narrow end ignited, small flame + sparks
        d.polygon([(14, 8), (18, 8), (16, 3)], fill=ORANGE)
        d.point((16, 5), fill=FLAME)
        for (x, y) in ((12, 4), (20, 5), (16, 1)):
            d.point((x, y), fill=FLAME)
    elif state == "act":  # burning hard: big flame and smoke plume
        d.polygon([(13, 8), (19, 8), (16, 1)], fill=ORANGE)
        d.point((16, 4), fill=FLAME)
        for (x, y, r) in ((10, 6, 3), (22, 8, 4), (8, 12, 3), (25, 14, 3),
                          (12, 2, 2)):
            d.ellipse([x - r, y - r, x + r, y + r], fill=SMOKE)
        d.point((16, 20), fill=ORANGE)  # burn front creeping down


# ---------------------------------------------------------------- ammo

def cartridge(d, rng, case_h, case_w, bullet, tip):
    """Upright cartridge centred at x=16, base at y=29."""
    bx = 29
    cx0, cx1 = 16 - case_w // 2, 16 + case_w // 2
    cy = bx - case_h
    d.rectangle([cx0 - 1, bx - 1, cx1 + 1, bx], fill=BRASS_D)  # rim
    d.rectangle([cx0, cy, cx1, bx - 1], fill=BRASS)            # case
    d.line([cx0 + 1, cy, cx0 + 1, bx - 2], fill=shade(BRASS, 1.2), width=1)
    by = cy  # bullet base at case mouth
    if bullet == "spitzer":
        d.polygon([(cx0, by), (cx1, by), (16 + 1, by - 7), (16 - 1, by - 7)],
                  fill=COPPER)
        th = 3
    elif bullet == "round":
        d.rectangle([cx0, by - 4, cx1, by], fill=COPPER)
        d.arc([cx0, by - 7, cx1, by - 1], start=180, end=360, fill=COPPER)
        th = 5
    elif bullet == "flat":  # flat-nose jacketed / solid
        d.rectangle([cx0, by - 5, cx1, by], fill=tip if tip else COPPER)
        th = 5
    else:  # "lead" soft point: jacket sides, exposed lead flat tip
        d.rectangle([cx0, by - 5, cx1, by], fill=COPPER)
        d.rectangle([cx0 + 1, by - 7, cx1 - 1, by - 5], fill=LEAD)
        th = 7
    if tip and bullet in ("spitzer", "round"):
        d.polygon([(16 - 1, by - 7), (16 + 1, by - 7), (16 + 1, by - 7 + th),
                   (16 - 1, by - 7 + th)], fill=tip)
    if bullet == "jhp":
        pass
    return


def cartridge_jhp(d, rng, case_h, case_w):
    bx = 29
    cx0, cx1 = 16 - case_w // 2, 16 + case_w // 2
    cy = bx - case_h
    d.rectangle([cx0 - 1, bx - 1, cx1 + 1, bx], fill=BRASS_D)
    d.rectangle([cx0, cy, cx1, bx - 1], fill=BRASS)
    d.line([cx0 + 1, cy, cx0 + 1, bx - 2], fill=shade(BRASS, 1.2), width=1)
    d.rectangle([cx0, cy - 5, cx1, cy], fill=COPPER)  # jacketed bullet
    d.rectangle([cx0 + 2, cy - 6, cx1 - 2, cy - 4], fill=INK)  # hollow cavity
    d.point((16, cy - 5), fill=LEAD)


def draw_mag(d, rng, col, rounds, curved):
    if curved:
        for i in range(5):
            x = 11 + i
            d.rectangle([x, 8 + i * 2, x + 8, 12 + i * 2], fill=col)
        d.rectangle([10, 6, 20, 10], fill=shade(col, 1.15))
    else:
        d.rectangle([11, 7, 21, 27], fill=col)
        d.rectangle([11, 7, 21, 27], outline=shade(col, 0.7))
        d.rectangle([10, 26, 22, 29], fill=shade(col, 0.75))  # baseplate
    for i in range(rounds):  # cartridge tops peeking out
        d.rectangle([13 + i * 3, 4, 15 + i * 3, 7], fill=BRASS)
        d.point((14 + i * 3, 3), fill=COPPER)
    d.line([18, 12, 18, 22], fill=shade(col, 0.75), width=1)  # witness slot


# ---------------------------------------------------------------- dispatch

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "barrel_ported":
        draw_barrel(d, rng, True)
    elif sid == "barrel_small":
        draw_barrel(d, rng, False)
    elif sid == "upstest":
        draw_upstest(d, rng)
    elif sid == "bio_emp_gun":
        draw_emp_gun(d, rng)
    elif sid == "mut_longpull":
        draw_longpull(d, rng)
    elif sid == "mut_quills":
        draw_quills(d, rng)
    elif sid == "bot_nl_surgicalbot":
        bonebot_body(d, rng, broken=False, tipped=False)
    elif sid == "broken_nl_surgicalbot_1_1":
        bonebot_body(d, rng, broken=True, tipped=False)
    elif sid == "broken_nl_surgicalbot_assist":
        bonebot_body(d, rng, broken=True, tipped=True)
    elif sid == "bronze_atlatl_spear":
        draw_atlatl(d, rng, BRONZE, True)
    elif sid == "steel_atlatl_spear":
        draw_atlatl(d, rng, STEEL, True)
    elif sid == "wood_atlatl_spear":
        draw_atlatl(d, rng, WOOD_L, False)
    elif sid == "browning_citori_20":
        draw_citori(d, rng)
    elif sid == "remington_870_20":
        draw_870(d, rng)
    elif sid == "savage_model_220":
        draw_220(d, rng)
    elif sid == "buckle_steel":
        draw_buckle(d, rng)
    elif sid == "snapfastener_steel":
        draw_snap(d, rng)
    elif sid == "touch_fastener":
        draw_touch(d, rng)
    elif sid == "coconut_water":
        draw_coconut(d, rng)
    elif sid == "pekmez":
        jar(d, rng, (120, 40, 92, 255), label=(240, 226, 200, 255))
    elif sid == "salca":
        jar(d, rng, (196, 56, 36, 255), label=(240, 226, 200, 255))
    elif sid == "dry_mushroom":
        draw_mushrooms(d, rng, cooked=False)
    elif sid == "mushroom_cooked":
        draw_mushrooms(d, rng, cooked=True)
    elif sid == "mushroom_ketchup":
        jar(d, rng, (110, 76, 44, 255), cap=(196, 52, 44, 255),
            label=(240, 226, 200, 255))
    elif sid == "electric_masonrysaw_off":
        draw_saw(d, rng, on=False)
    elif sid == "electric_masonrysaw_on":
        draw_saw(d, rng, on=True)
    elif sid == "grinder_blade":
        draw_grinder_blade(d, rng)
    elif sid == "forgerig":
        draw_forgerig(d, rng)
    elif sid == "kilnrig":
        draw_kilnrig(d, rng)
    elif sid == "weldkit":
        draw_weldkit(d, rng)
    elif sid == "knife_sharpener_handheld":
        draw_sharpener(d, rng)
    elif sid == "lajatang":
        draw_lajatang(d, rng)
    elif sid == "smart_watch":
        draw_watch(d, rng)
    elif sid == "manual_canning":
        draw_manual_canning(d, rng)
    elif sid == "manual_sealing":
        draw_manual_sealing(d, rng)
    elif sid == "vacuum_sealing":
        draw_vacuum_sealing(d, rng)
    elif sid == "nomad_rig":
        harness(d, rng, WOOD, WOOD_L, lit=False)
    elif sid == "nomad_rig_on":
        harness(d, rng, WOOD, WOOD_L, lit=True)
    elif sid == "survivor_rig":
        draw_survivor_rig(d, rng)
    elif sid == "tool_rocket_candy":
        draw_rocket_candy(d, rng, "plain")
    elif sid == "tool_rocket_candy_primed":
        draw_rocket_candy(d, rng, "primed")
    elif sid == "tool_rocket_candy_act":
        draw_rocket_candy(d, rng, "act")
    elif sid == "223":
        cartridge(d, rng, 15, 6, "spitzer", None)
    elif sid == "556_m995":
        cartridge(d, rng, 15, 6, "spitzer", BLACKTIP)
    elif sid == "308":
        cartridge(d, rng, 18, 7, "spitzer", None)
    elif sid == "762_51_m993":
        cartridge(d, rng, 18, 7, "spitzer", BLACKTIP)
    elif sid == "44fmj":
        cartridge(d, rng, 13, 8, "flat", COPPER)
    elif sid == "44magnum":
        cartridge_jhp(d, rng, 13, 8)
    elif sid == "450_ar15mag":
        draw_mag(d, rng, STEEL, 2, curved=False)
    elif sid == "ruger_arr_mag":
        draw_mag(d, rng, (58, 58, 64, 255), 1, curved=False)
    elif sid == "4570_pen":
        cartridge(d, rng, 19, 8, "flat", COPPER)
        d.rectangle([13, 4, 19, 6], fill=COPPER)  # long solid penetrator nose
    elif sid == "4570_sp":
        cartridge(d, rng, 19, 8, "lead", None)
    else:
        raise KeyError(sid)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    # the embedded list is authoritative-checked against the batch file
    batch = os.path.join(os.path.expanduser("~"), "workspace",
                         "item-batches", "87.txt")
    with open(batch) as f:
        file_ids = f.read().split()
    assert IDS == file_ids, "embedded IDS differ from 87.txt"
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
                       "preview_item87.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item87.png {prev.size}")


if __name__ == "__main__":
    main()
