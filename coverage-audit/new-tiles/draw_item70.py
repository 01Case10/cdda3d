#!/usr/bin/env python3
"""ITEM70: exotic ammo/fluids, alien + primitive weapons, preserved
fruit foods, and bow/gun sights -- CDDA Voxel coverage audit.

Four families, art grounded in each id's game entry:

  Ammo types & fluids (12, items/ammo_types.json): the 27.3x44mm
    Diyua rocket as a stubby fat rocket round; .30-30 Winchester and
    .460 S&W Magnum as brass cartridges told apart by case shape
    (bottleneck round-nose vs long straight-wall magnum); ammonia,
    biogas, crude lamp oil and hydraulic oil as containers told
    apart by content colour and label motif (hazard diamond + fumes,
    flame, lamp, gear); atlatl spears as a fletched dart bundle;
    bone dart as a small white dart; speargun spear as a barbed
    shaft; Caotel cell as a glowing power cell; plutonium as a
    shielded canister with a radiation trefoil.
  Monster guns (12, monster_special_attacks/monster_gun.json):
    flesh organs drawn as themselves -- acid dart/spit/volley guns
    in pink flesh with green acid sacs told apart by barrel form,
    barb and gene-sting launchers as red flesh with a loaded bony
    barb, bone dart organ as a pale bone tube, boomer head as a
    bloated bile-green zombie head, EMP frond as a red alien frond
    with arcing tips, hurled rubble as a stone chunk, fireworks
    cannon as a steel tube with rockets and starbursts, mi-go
    bio-gun as a segmented chitin gun, huntsman arm as a long bony
    throwing appendage.
  Preserved fruit (12, items/comestibles/fruit_dishes.json): tins,
    jars, packets and bowls as named -- canned fruit and peaches in
    syrup as labelled tins, sugared slices and brandied fruit as
    syrup/alcohol jars, elderberries as a bowl of dark berries,
    dehydrated fruit and drink-mix powders as packets (grape
    purple, orange orange), fruit leather as a pink roll,
    artichoke mash and rehydrated fruit as bowls, pumpkin yeast
    bread as a golden loaf.
  Sights (12, items/gunmod/sights.json): optics in profile with a
    lens glint, sized by type -- bow scope on its bracket, five-pin
    and single-pin bow sights as pin brackets, holo magnifier as a
    compact flip box, iron sights as post + notch, pistol scope
    small, rifle scope long with turrets, high-end rifle scope with
    a top rail, RS1219 as a boxy digital scope, hand-crafted
    telescopic sights as brass tubes, Nagoya 99 as an aged antique.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "273x44RG",
    "3030",
    "460sw",
    "ammonia_liquid",
    "atlatl",
    "biogas",
    "bone_dart",
    "caotel_cell",
    "crude_lamp_oil",
    "fishspear",
    "hydraulic_oil",
    "plutonium",
    "acid_sniper",
    "acid_spit",
    "acid_spray",
    "barb_launcher",
    "bone_dart_launcher",
    "boomer_head",
    "emp_frond",
    "feral_human_thrown_rock",
    "firework_cannon",
    "gene_sting_gun",
    "migo_bio_gun",
    "nether_huntsman_arm",
    "apple_canned",
    "apple_sugar",
    "brandied_fruit",
    "can_peach",
    "cooked_elderberries",
    "dry_fruit",
    "fruit_leather",
    "grapeade_powder",
    "jerusalem_artichoke_fruitmash",
    "orangeade_powder",
    "pumpkin_yeast_bread",
    "rehydrated_fruit",
    "bow_scope",
    "bow_sight",
    "bow_sight_pin",
    "holo_magnifier",
    "improve_sights",
    "pistol_scope",
    "rifle_scope",
    "rifle_scope_high_end_mount",
    "riv_scope",
    "tele_sight",
    "tele_sight_pistol",
    "type99_scope",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
BRASS = (198, 158, 74, 255)
BRASS_D = (150, 116, 48, 255)
COPPER = (186, 108, 58, 255)
STEEL = (128, 132, 140, 255)
STEEL_D = (84, 88, 96, 255)
GUNMETAL = (58, 60, 66, 255)
GLASS = (140, 200, 220, 255)
FLESH = (192, 128, 116, 255)
FLESH_D = (150, 88, 80, 255)
HFLESH = (172, 74, 66, 255)
HFLESH_D = (128, 48, 44, 255)
BONE = (230, 222, 198, 255)
BONE_D = (188, 178, 148, 255)
ACID = (116, 204, 58, 255)
ACID_D = (76, 148, 34, 255)
WOOD = (110, 78, 44, 255)
WOOD_D = (80, 56, 30, 255)
TIN = (176, 180, 186, 255)
TIN_D = (120, 124, 132, 255)
PAPER_L = (238, 230, 214, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def speckle(d, rng, box, col, n=14):
    x0, y0, x1, y1 = box
    for _ in range(n):
        d.point((rng.randint(x0, x1), rng.randint(y0, y1)), fill=col)


# ------------------------------------------------------------- ammo/fluids

def draw_rocket(d, rng):
    # 27.3x44 Diyua rocket: fat stubby rocket round, standing upright
    d.polygon([(13, 3), (19, 3), (21, 8), (21, 12), (11, 12), (11, 8)],
              fill=(168, 60, 48, 255))            # warhead, red band tip
    d.rectangle([11, 12, 21, 24], fill=STEEL)     # fat body
    d.rectangle([11, 12, 21, 14], fill=STEEL_D)
    d.rectangle([14, 17, 18, 21], fill=GUNMETAL)  # motor port plate
    d.polygon([(11, 24), (8, 29), (13, 29), (13, 24)], fill=STEEL_D)  # fins
    d.polygon([(21, 24), (24, 29), (19, 29), (19, 24)], fill=STEEL_D)
    d.rectangle([13, 24, 19, 27], fill=BRASS_D)   # base
    d.point((16, 9), fill=(250, 220, 120, 255))


def draw_cartridge(d, rng, magnum=False):
    # brass cartridge upright; .30-30 bottleneck vs .460 straight wall
    if magnum:
        d.rectangle([13, 16, 19, 28], fill=BRASS)       # long fat case
        d.rectangle([12, 27, 20, 29], fill=BRASS_D)     # rim
        d.rectangle([13, 16, 19, 18], fill=BRASS_D)
        d.polygon([(13, 16), (19, 16), (18, 8), (17, 5), (15, 5),
                   (14, 8)], fill=COPPER)               # big flat bullet
        d.rectangle([14, 5, 18, 7], fill=shade(COPPER, 1.15))
    else:
        d.rectangle([13, 20, 19, 28], fill=BRASS)       # case
        d.rectangle([12, 27, 20, 29], fill=BRASS_D)     # rimmed base
        d.polygon([(13, 20), (19, 20), (17, 15), (13, 15)], fill=BRASS)
        d.polygon([(14, 15), (18, 15), (17, 8), (16, 5), (15, 8)],
                  fill=COPPER)                          # round-nose bullet
        d.point((16, 7), fill=shade(COPPER, 1.2))
    speckle(d, rng, (13, 18, 19, 28), BRASS_D, 5)


def draw_bottle(d, rng, liquid, motif, jug=False):
    # plastic bottle (or squat jug) with liquid fill and a paper label
    if jug:
        d.rectangle([9, 12, 23, 28], fill=(226, 228, 230, 255))
        d.rectangle([13, 8, 19, 12], fill=(226, 228, 230, 255))
        d.rectangle([13, 6, 19, 8], fill=STEEL_D)       # cap
        d.rectangle([10, 14, 22, 27], fill=liquid)
        box = (10, 14, 22, 27)
    else:
        d.rectangle([12, 11, 20, 28], fill=(226, 228, 230, 255))
        d.rectangle([14, 7, 18, 11], fill=(226, 228, 230, 255))
        d.rectangle([14, 5, 18, 7], fill=STEEL_D)
        d.rectangle([13, 13, 19, 27], fill=liquid)
        box = (13, 13, 19, 27)
    d.rectangle([box[0] + 1, 19, box[2] - 1, 25], fill=PAPER_L)
    cx = (box[0] + box[2]) // 2
    if motif == "hazard":      # diamond + fume curls (ammonia)
        d.polygon([(cx, 20), (cx + 2, 22), (cx, 24), (cx - 2, 22)],
                  outline=(200, 60, 40, 255))
        d.point((cx, 22), fill=(200, 60, 40, 255))
        d.point((14, 4), fill=(170, 210, 220, 255))
        d.point((18, 3), fill=(170, 210, 220, 255))
    elif motif == "flame":     # biogas
        d.polygon([(cx, 19), (cx + 2, 22), (cx + 1, 24), (cx - 1, 24),
                   (cx - 2, 22)], fill=(232, 120, 36, 255))
        d.point((cx, 23), fill=(250, 200, 80, 255))
    elif motif == "lamp":      # crude lamp oil
        d.rectangle([cx - 2, 21, cx + 2, 24], fill=INK)
        d.point((cx, 20), fill=(250, 190, 70, 255))
    elif motif == "gear":      # hydraulic oil
        d.ellipse([cx - 2, 20, cx + 2, 24], outline=INK)
        d.point((cx, 22), fill=INK)
    speckle(d, rng, box, shade(liquid, 0.85), 6)


def draw_gas_tank(d, rng):
    # biogas: squat green pressure tank with valve and flame label
    d.rectangle([10, 11, 22, 28], fill=(86, 128, 74, 255))
    d.rectangle([10, 11, 22, 13], fill=(64, 100, 56, 255))
    d.rectangle([14, 7, 18, 11], fill=STEEL_D)
    d.rectangle([15, 5, 17, 7], fill=STEEL)
    d.rectangle([12, 17, 20, 24], fill=PAPER_L)
    d.polygon([(16, 18), (18, 21), (17, 23), (15, 23), (14, 21)],
              fill=(232, 120, 36, 255))
    d.point((16, 22), fill=(250, 200, 80, 255))
    speckle(d, rng, (10, 25, 22, 28), (64, 100, 56, 255), 6)


def draw_atlatl_darts(d, rng):
    # bundle of three long fletched atlatl darts, diagonal
    for i, off in enumerate((-3, 0, 3)):
        x0, y0, x1, y1 = 6 + off, 26, 24 + off, 8
        d.line([x0, y0, x1, y1], fill=WOOD, width=2)
        d.polygon([(x1, y1), (x1 + 3, y1 - 3), (x1 + 1, y1 + 2)],
                  fill=BONE)  # foreshaft point
        # fletching at the butt
        d.polygon([(x0, y0), (x0 - 3, y0 - 2), (x0 - 1, y0 + 2)],
                  fill=(200, 70, 50, 255) if i != 1 else (230, 225, 210, 255))
        d.polygon([(x0, y0), (x0 + 2, y0 - 3), (x0 + 3, y0 + 1)],
                  fill=(200, 70, 50, 255) if i != 1 else (230, 225, 210, 255))
    d.line([4, 24, 27, 24], fill=WOOD_D, width=1)  # binding cord shadow


def draw_bone_dart(d, rng):
    # single small bone dart, diagonal, barbed tip
    d.line([8, 24, 22, 10], fill=BONE, width=3)
    d.line([8, 24, 22, 10], fill=BONE_D, width=1)
    d.polygon([(22, 10), (26, 6), (24, 12)], fill=BONE)
    d.polygon([(20, 13), (23, 12), (21, 15)], fill=BONE_D)  # barb
    d.polygon([(8, 24), (5, 27), (10, 26)], fill=(200, 70, 50, 255))
    d.polygon([(8, 24), (11, 27), (10, 22)], fill=(200, 70, 50, 255))


def draw_fishspear(d, rng):
    # speargun spear: long thin shaft, barbed harpoon head, line loop
    d.line([5, 25, 23, 7], fill=STEEL, width=2)
    d.polygon([(23, 7), (28, 3), (25, 9)], fill=STEEL_D)
    d.polygon([(21, 10), (24, 10), (22, 13)], fill=STEEL_D)  # flopper barb
    d.arc([2, 22, 10, 30], start=300, end=140, fill=(210, 205, 190, 255),
          width=1)  # shooting line loop at the butt
    d.point((6, 24), fill=INK)


def draw_caotel_cell(d, rng):
    # Caotel cell: dark cylinder with a glowing teal core window
    d.rectangle([11, 6, 21, 27], fill=GUNMETAL)
    d.rectangle([13, 4, 19, 6], fill=STEEL)
    d.rectangle([13, 27, 19, 29], fill=STEEL_D)
    d.rectangle([14, 10, 18, 23], fill=(30, 60, 62, 255))
    d.rectangle([15, 11, 17, 22], fill=(64, 224, 208, 255))
    d.rectangle([15, 14, 17, 15], fill=(180, 250, 240, 255))
    for yy in (9, 24):
        d.line([11, yy, 21, yy], fill=STEEL_D, width=1)


def draw_plutonium(d, rng):
    # shielded canister with radiation trefoil
    d.rectangle([9, 9, 23, 28], fill=(104, 106, 112, 255))
    d.rectangle([9, 9, 23, 12], fill=(78, 80, 86, 255))
    d.rectangle([13, 6, 19, 9], fill=(78, 80, 86, 255))
    d.rectangle([11, 16, 21, 25], fill=(240, 200, 60, 255))
    cx, cy = 16, 20
    d.point((cx, cy), fill=INK)
    for dx, dy in ((0, -3), (-3, 2), (3, 2)):
        d.polygon([(cx, cy), (cx + dx - 1, cy + dy), (cx + dx + 1, cy + dy)],
                  fill=INK)
    speckle(d, rng, (9, 26, 23, 28), (78, 80, 86, 255), 5)


# ------------------------------------------------------------ monster guns

def flesh_gun(d, rng, barrel):
    # shared fleshy body: grip mass + sac; barrel form varies
    d.polygon([(8, 20), (14, 16), (20, 18), (20, 24), (12, 27),
               (8, 25)], fill=FLESH)
    d.polygon([(10, 24), (14, 22), (18, 23), (16, 26), (11, 26)],
              fill=FLESH_D)  # grip fold
    d.ellipse([9, 13, 16, 19], fill=FLESH_D)      # acid sac
    d.ellipse([10, 14, 14, 18], fill=ACID)
    d.point((12, 15), fill=shade(ACID, 1.3))
    if barrel == "sniper":     # long thin dart barrel
        d.line([19, 19, 29, 15], fill=FLESH_D, width=2)
        d.point((29, 15), fill=ACID)
    elif barrel == "spit":     # short nozzle, dripping glob
        d.polygon([(19, 18), (26, 17), (26, 22), (19, 23)], fill=FLESH_D)
        d.ellipse([24, 19, 27, 22], fill=ACID)
        d.point((26, 24), fill=ACID)
        d.point((26, 26), fill=ACID_D)
    elif barrel == "spray":    # wide multi-orifice head
        d.polygon([(19, 17), (27, 15), (28, 24), (19, 24)], fill=FLESH_D)
        for py in (17, 20, 23):
            d.point((27, py), fill=ACID)
            d.point((29, py), fill=ACID)
    speckle(d, rng, (8, 16, 20, 27), FLESH_D, 8)


def draw_barb_organ(d, rng, viscous=False):
    # red fleshy launcher with a bony barb loaded in the muzzle
    d.polygon([(6, 18), (16, 15), (24, 17), (24, 23), (14, 26),
               (7, 24)], fill=HFLESH)
    d.polygon([(8, 23), (14, 21), (20, 22), (16, 25), (9, 25)],
              fill=HFLESH_D)
    d.rectangle([22, 18, 26, 22], fill=HFLESH_D)  # muzzle lips
    d.polygon([(26, 20), (30, 18), (30, 22)], fill=BONE)  # barb head
    d.line([24, 20, 30, 20], fill=BONE_D, width=1)
    d.ellipse([10, 12, 17, 17], fill=HFLESH_D)    # pressure sac
    if viscous:
        d.ellipse([11, 13, 15, 16], fill=ACID)    # viscous green fill
        d.point((28, 23), fill=ACID)
        d.point((29, 25), fill=ACID_D)
    else:
        d.ellipse([11, 13, 15, 16], fill=shade(HFLESH, 1.2))
    speckle(d, rng, (6, 15, 24, 26), HFLESH_D, 8)


def draw_bone_organ(d, rng):
    # pale cyber-bone tube organ with a dart ready
    d.polygon([(6, 19), (20, 16), (25, 18), (25, 23), (12, 26),
               (6, 24)], fill=BONE)
    d.line([8, 21, 23, 19], fill=BONE_D, width=1)
    d.line([9, 24, 22, 22], fill=BONE_D, width=1)
    d.rectangle([23, 18, 27, 23], fill=BONE_D)
    d.polygon([(27, 20), (31, 19), (31, 22)], fill=BONE)  # dart tip out
    d.ellipse([9, 13, 15, 18], fill=BONE_D)
    d.point((12, 15), fill=(200, 60, 50, 255))   # red core glow
    speckle(d, rng, (6, 16, 25, 26), BONE_D, 7)


def draw_boomer_head(d, rng):
    # bloated bile-green zombie head
    d.ellipse([8, 6, 25, 24], fill=(150, 158, 92, 255))
    d.ellipse([10, 20, 23, 28], fill=(138, 146, 84, 255))  # jowls
    d.ellipse([11, 10, 15, 14], fill=(120, 128, 70, 255))  # eye sockets
    d.ellipse([18, 10, 22, 14], fill=(120, 128, 70, 255))
    d.point((13, 12), fill=(230, 230, 200, 255))
    d.point((20, 12), fill=(230, 230, 200, 255))
    d.rectangle([13, 21, 20, 24], fill=(84, 70, 44, 255))  # mouth
    for tx in (14, 17, 19):
        d.point((tx, 21), fill=(226, 220, 190, 255))
    d.point((12, 25), fill=(168, 178, 100, 255))  # bile drips
    d.point((18, 26), fill=(168, 178, 100, 255))
    d.point((16, 28), fill=(168, 178, 100, 255))
    speckle(d, rng, (8, 6, 25, 20), (120, 128, 70, 255), 10)


def draw_emp_frond(d, rng):
    # red alien frond: central rib with leaflets, arcs at the tips
    d.line([16, 29, 16, 6], fill=HFLESH_D, width=2)
    for i, yy in enumerate(range(9, 27, 3)):
        w = 8 - abs(yy - 17) // 2
        d.line([16, yy, 16 - w, yy - 3], fill=HFLESH, width=2)
        d.line([16, yy, 16 + w, yy - 3], fill=HFLESH, width=2)
    for tx, ty in ((8, 5), (24, 5), (16, 4), (10, 12), (22, 12)):
        d.point((tx, ty), fill=(250, 230, 90, 255))       # arc sparks
        d.point((tx + 1, ty + 1), fill=(140, 200, 250, 255))
    d.line([7, 4, 9, 6], fill=(250, 230, 90, 255), width=1)
    d.line([25, 4, 23, 6], fill=(250, 230, 90, 255), width=1)


def draw_rock(d, rng):
    d.polygon([(8, 22), (10, 12), (17, 8), (24, 12), (25, 21),
               (20, 26), (12, 26)], fill=(130, 128, 124, 255))
    d.polygon([(10, 12), (17, 8), (18, 16), (12, 19)], fill=(150, 148, 142, 255))
    d.line([18, 16, (24), 21], fill=(104, 102, 98, 255), width=1)
    d.line([12, 19, (18), 25], fill=(104, 102, 98, 255), width=1)
    speckle(d, rng, (9, 10, 24, 25), (104, 102, 98, 255), 10)


def draw_firework_cannon(d, rng):
    # steel tube cannon with loaded firework rockets and starbursts
    d.polygon([(7, 24), (22, 24), (24, 28), (6, 28)], fill=STEEL_D)
    d.rectangle([10, 10, 15, 25], fill=STEEL)
    d.rectangle([10, 10, 15, 13], fill=GUNMETAL)
    d.rectangle([19, 13, 24, 25], fill=STEEL)
    d.rectangle([19, 13, 24, 16], fill=GUNMETAL)
    d.polygon([(11, 10), (14, 10), (12, 4)], fill=(200, 60, 48, 255))
    d.polygon([(20, 13), (23, 13), (21, 7)], fill=(60, 120, 200, 255))
    for cx, cy, col in ((6, 5, (250, 200, 70, 255)),
                        (27, 4, (120, 220, 90, 255)),
                        (26, 10, (250, 200, 70, 255))):
        for dx, dy in ((-2, 0), (2, 0), (0, -2), (0, 2), (-1, -1), (1, 1)):
            d.point((cx + dx, cy + dy), fill=col)
        d.point((cx, cy), fill=(255, 250, 230, 255))
    speckle(d, rng, (10, 14, 24, 24), STEEL_D, 6)


def draw_migo_gun(d, rng):
    # segmented chitin bio-gun, gray-pink
    chit = (168, 148, 142, 255)
    chit_d = (120, 102, 98, 255)
    d.polygon([(6, 19), (14, 16), (22, 17), (26, 19), (25, 23),
               (14, 25), (8, 24)], fill=chit)
    for xx in (12, 17, 22):  # segment seams
        d.line([xx, 17, xx - 1, 24], fill=chit_d, width=1)
    d.polygon([(25, 18), (30, 17), (30, 21), (25, 22)], fill=chit_d)
    d.point((29, 19), fill=(90, 200, 170, 255))   # glowing aperture
    d.polygon([(9, 22), (13, 22), (12, 28), (8, 27)], fill=chit_d)  # grip
    d.ellipse([13, 11, 20, 16], fill=chit)        # dorsal sac
    d.ellipse([14, 12, 18, 15], fill=(196, 170, 160, 255))
    speckle(d, rng, (6, 16, 26, 25), chit_d, 7)


def draw_huntsman_arm(d, rng):
    # long bony segmented throwing arm with a claw, diagonal
    pts = [(6, 26), (11, 21), (16, 17), (21, 12), (25, 8)]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        d.line([x0, y0, x1, y1], fill=BONE, width=4)
        d.line([x0, y0, x1, y1], fill=BONE_D, width=1)
    for x, y in pts[1:-1]:
        d.ellipse([x - 2, y - 2, x + 2, y + 2], outline=BONE_D)
    for dx, dy in ((-3, -2), (0, -4), (3, -2)):   # claw fingers
        d.line([25, 8, 25 + dx, 8 + dy], fill=BONE, width=2)
    d.polygon([(4, 28), (9, 28), (7, 23)], fill=BONE_D)  # shoulder stump
    speckle(d, rng, (5, 6, 27, 28), BONE_D, 6)


# -------------------------------------------------------------------- food

def draw_tin(d, rng, label_col, motif):
    d.rectangle([10, 8, 22, 27], fill=TIN)
    d.rectangle([10, 8, 22, 10], fill=TIN_D)
    d.rectangle([10, 25, 22, 27], fill=TIN_D)
    d.rectangle([10, 13, 22, 22], fill=label_col)
    cx, cy = 16, 17
    if motif == "apple":
        d.ellipse([cx - 3, cy - 2, cx + 3, cy + 4], fill=(216, 60, 48, 255))
        d.line([cx, cy - 2, cx + 1, cy - 4], fill=WOOD_D, width=1)
        d.point((cx + 2, cy - 3), fill=(80, 150, 60, 255))
    elif motif == "peach":
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=(250, 180, 90, 255))
        d.line([cx, cy - 3, cx, cy + 3], fill=(226, 130, 60, 255), width=1)
    speckle(d, rng, (10, 8, 22, 27), TIN_D, 5)


def draw_jar(d, rng, fill_col, chunks, chunk_col):
    d.rectangle([11, 10, 21, 28], fill=(214, 226, 230, 255))
    d.rectangle([12, 12, 20, 27], fill=fill_col)
    d.rectangle([11, 7, 21, 10], fill=(178, 60, 48, 255))  # lid
    for (x, y) in chunks:
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=chunk_col)
        d.point((x - 1, y - 1), fill=shade(chunk_col, 1.25))
    d.line([13, 13, 13, 26], fill=(240, 248, 250, 255), width=1)  # glass shine


def draw_packet(d, rng, col, motif):
    d.polygon([(10, 7), (22, 7), (23, 26), (9, 26)], fill=col)
    d.line([10, 10, 22, 10], fill=shade(col, 0.8), width=1)  # crimp
    d.line([9, 24, 23, 24], fill=shade(col, 0.8), width=1)
    d.rectangle([12, 13, 20, 20], fill=PAPER_L)
    cx, cy = 16, 16
    if motif == "grape":
        for dx, dy in ((-1, 0), (1, 0), (0, 1), (-1, 2), (1, 2), (0, 3)):
            d.point((cx + dx, cy + dy), fill=(110, 60, 160, 255))
        d.point((cx + 1, cy - 1), fill=(80, 150, 60, 255))
    elif motif == "orange":
        d.ellipse([cx - 2, cy, cx + 2, cy + 4], fill=(240, 140, 30, 255))
        d.point((cx, cy), fill=(80, 150, 60, 255))
    elif motif == "flakes":
        for dx, dy in ((-2, 1), (0, 0), (2, 1), (-1, 3), (1, 3)):
            d.point((cx + dx, cy + dy), fill=(196, 90, 110, 255))
    speckle(d, rng, (10, 7, 22, 26), shade(col, 0.85), 6)


def draw_bowl(d, rng, fill_col, chunk_col=None, n=8):
    d.polygon([(7, 14), (25, 14), (22, 24), (10, 24)], fill=(196, 200, 206, 255))
    d.rectangle([7, 12, 25, 14], fill=(214, 218, 224, 255))
    d.rectangle([8, 12, 24, 13], fill=fill_col)
    for _ in range(n):
        x = rng.randint(9, 23)
        d.point((x, 12), fill=chunk_col or shade(fill_col, 1.2))
    d.rectangle([13, 24, 19, 26], fill=(150, 154, 162, 255))
    speckle(d, rng, (9, 15, 23, 23), shade((196, 200, 206, 255), 0.9), 5)


def draw_fruit_leather(d, rng):
    # rolled sheet of pink fruit leather
    d.ellipse([9, 10, 17, 26], fill=(216, 110, 140, 255))
    d.ellipse([10, 11, 16, 25], fill=(196, 88, 118, 255))
    d.ellipse([11, 13, 15, 23], fill=(226, 140, 165, 255))
    d.polygon([(15, 12), (26, 10), (26, 22), (15, 24)], fill=(206, 100, 128, 255))
    d.line([15, 12, 26, 10], fill=(226, 140, 165, 255), width=1)
    speckle(d, rng, (15, 12, 26, 23), (180, 80, 105, 255), 8)


def draw_bread(d, rng):
    # golden-orange pumpkin loaf with scored top
    d.polygon([(8, 20), (9, 13), (14, 10), (20, 10), (24, 14),
               (24, 20)], fill=(216, 138, 52, 255))
    d.rectangle([8, 20, 24, 25], fill=(196, 118, 42, 255))
    for xx in (12, 16, 20):
        d.line([xx, 11, xx + 1, 14], fill=(240, 180, 90, 255), width=1)
    d.rectangle([8, 24, 24, 25], fill=(160, 92, 30, 255))
    speckle(d, rng, (9, 12, 23, 24), (160, 92, 30, 255), 7)


# ------------------------------------------------------------------ sights

def scope_body(d, rng, x0, x1, tube=STEEL_D, hi=False, rail=False,
               antique=False):
    cy = 15
    d.rectangle([x0, cy - 3, x1, cy + 3], fill=tube)          # main tube
    d.rectangle([x0, cy - 4, x0 + 4, cy + 4], fill=shade(tube, 0.85))  # ocular
    d.rectangle([x1 - 5, cy - 5, x1, cy + 5], fill=shade(tube, 0.9))   # objective bell
    d.rectangle([x1 - 2, cy - 4, x1, cy + 4], fill=GLASS)     # lens
    d.point((x1 - 1, cy - 2), fill=(235, 250, 255, 255))      # glint
    mid = (x0 + x1) // 2
    d.rectangle([mid - 2, cy - 7, mid + 2, cy - 3], fill=tube)  # turret
    d.rectangle([mid - 2, cy - 8, mid + 2, cy - 7], fill=shade(tube, 0.8))
    if rail:  # extra rail on top (high-end mount)
        for xx in range(x0 + 6, x1 - 7, 3):
            d.rectangle([xx, cy - 10, xx + 1, cy - 8], fill=tube)
        d.line([x0 + 6, cy - 8, x1 - 7, cy - 8], fill=tube, width=1)
    # mounts dropping below
    for mx in (x0 + 5, x1 - 9):
        d.rectangle([mx, cy + 3, mx + 2, cy + 8], fill=shade(tube, 0.8))
    d.line([x0 + 4, cy + 8, x1 - 8, cy + 8], fill=shade(tube, 0.8), width=1)
    if antique:
        speckle(d, rng, (x0, cy - 3, x1, cy + 3), (96, 74, 40, 255), 8)


def draw_bow_scope(d, rng):
    scope_body(d, rng, 6, 27, tube=(70, 72, 78, 255))
    d.rectangle([8, 21, 11, 26], fill=(70, 72, 78, 255))   # bow bracket
    d.rectangle([8, 25, 16, 26], fill=(52, 54, 60, 255))
    d.point((24, 12), fill=(120, 220, 120, 255))           # peep dot


def draw_bow_sight(d, rng, pins=5):
    d.rectangle([6, 10, 9, 24], fill=(70, 72, 78, 255))     # bracket
    d.rectangle([9, 12, 22, 14], fill=(52, 54, 60, 255))    # sight housing top
    d.rectangle([9, 22, 22, 24], fill=(52, 54, 60, 255))
    d.rectangle([9, 12, 11, 24], fill=(52, 54, 60, 255))
    cols = [(250, 80, 60, 255), (250, 200, 60, 255), (90, 220, 90, 255),
            (90, 180, 250, 255), (250, 120, 200, 255)]
    for i in range(pins):
        yy = 14 + i * 2
        d.line([11, yy, 19, yy], fill=(150, 152, 158, 255), width=1)
        d.point((20, yy), fill=cols[i])
    d.point((24, 18), fill=(200, 205, 210, 255))           # level bubble


def draw_holo_magnifier(d, rng):
    d.rectangle([10, 9, 23, 22], fill=GUNMETAL)            # boxy body
    d.rectangle([19, 11, 23, 20], fill=GLASS)              # front lens
    d.point((21, 13), fill=(235, 250, 255, 255))
    d.rectangle([12, 12, 16, 19], fill=(90, 94, 102, 255))  # rear window
    d.rectangle([13, 22, 20, 25], fill=STEEL_D)            # mount
    d.polygon([(20, 22), (25, 24), (20, 26)], fill=STEEL_D)  # flip lever
    d.line([10, 9, 23, 9], fill=(90, 94, 102, 255), width=1)


def draw_iron_sights(d, rng):
    d.rectangle([5, 22, 27, 24], fill=STEEL_D)             # rail base
    # rear notch sight
    d.rectangle([7, 14, 9, 22], fill=GUNMETAL)
    d.rectangle([13, 14, 15, 22], fill=GUNMETAL)
    d.rectangle([7, 20, 15, 22], fill=GUNMETAL)
    # front post
    d.rectangle([23, 13, 24, 22], fill=GUNMETAL)
    d.point((23, 12), fill=(250, 200, 80, 255))            # tritium dot
    d.polygon([(21, 22), (26, 22), (25, 18), (22, 18)], fill=STEEL_D)


def draw_riv_scope(d, rng):
    # RS1219: boxy white-gray digital scope with a screen
    d.rectangle([7, 10, 26, 21], fill=(196, 198, 204, 255))
    d.rectangle([7, 10, 26, 12], fill=(160, 162, 170, 255))
    d.rectangle([22, 12, 26, 19], fill=GLASS)
    d.point((24, 14), fill=(235, 250, 255, 255))
    d.rectangle([10, 13, 16, 18], fill=(40, 90, 80, 255))   # display
    d.line([11, 16, 15, 16], fill=(120, 250, 200, 255), width=1)
    d.line([13, 14, 13, 18], fill=(120, 250, 200, 255), width=1)
    d.rectangle([12, 21, 21, 25], fill=(140, 142, 150, 255))
    d.point((18, 15), fill=(250, 90, 70, 255))              # offset reflex dot


def draw_tele(d, rng, pistol=False):
    # hand-crafted telescope: brass tube, tapered, wood grip rings
    x0, x1 = (8, 26) if pistol else (5, 28)
    cy = 15
    d.polygon([(x0, cy - 2), (x1 - 4, cy - 4), (x1 - 4, cy + 4),
               (x0, cy + 2)], fill=(168, 132, 70, 255))
    d.rectangle([x1 - 4, cy - 4, x1, cy + 4], fill=STEEL_D)
    d.rectangle([x1 - 2, cy - 3, x1, cy + 3], fill=GLASS)
    d.point((x1 - 1, cy - 1), fill=(235, 250, 255, 255))
    d.rectangle([x0, cy - 3, x0 + 3, cy + 3], fill=STEEL_D)  # ocular cup
    for xx in (x0 + 6, x0 + 11):
        d.line([xx, cy - 3, xx, cy + 3], fill=WOOD_D, width=1)  # wrap rings
    d.rectangle([x0 + 7, cy + 4, x0 + 9, cy + 8], fill=STEEL_D)
    speckle(d, rng, (x0, cy - 3, x1 - 4, cy + 3), (120, 92, 44, 255), 6)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "273x44RG":
        draw_rocket(d, rng)
    elif sid == "3030":
        draw_cartridge(d, rng, magnum=False)
    elif sid == "460sw":
        draw_cartridge(d, rng, magnum=True)
    elif sid == "ammonia_liquid":
        draw_bottle(d, rng, (196, 226, 232, 255), "hazard")
    elif sid == "atlatl":
        draw_atlatl_darts(d, rng)
    elif sid == "biogas":
        draw_gas_tank(d, rng)
    elif sid == "bone_dart":
        draw_bone_dart(d, rng)
    elif sid == "caotel_cell":
        draw_caotel_cell(d, rng)
    elif sid == "crude_lamp_oil":
        draw_bottle(d, rng, (120, 84, 30, 255), "lamp", jug=True)
    elif sid == "fishspear":
        draw_fishspear(d, rng)
    elif sid == "hydraulic_oil":
        draw_bottle(d, rng, (52, 44, 36, 255), "gear", jug=True)
    elif sid == "plutonium":
        draw_plutonium(d, rng)
    elif sid == "acid_sniper":
        flesh_gun(d, rng, "sniper")
    elif sid == "acid_spit":
        flesh_gun(d, rng, "spit")
    elif sid == "acid_spray":
        flesh_gun(d, rng, "spray")
    elif sid == "barb_launcher":
        draw_barb_organ(d, rng)
    elif sid == "bone_dart_launcher":
        draw_bone_organ(d, rng)
    elif sid == "boomer_head":
        draw_boomer_head(d, rng)
    elif sid == "emp_frond":
        draw_emp_frond(d, rng)
    elif sid == "feral_human_thrown_rock":
        draw_rock(d, rng)
    elif sid == "firework_cannon":
        draw_firework_cannon(d, rng)
    elif sid == "gene_sting_gun":
        draw_barb_organ(d, rng, viscous=True)
    elif sid == "migo_bio_gun":
        draw_migo_gun(d, rng)
    elif sid == "nether_huntsman_arm":
        draw_huntsman_arm(d, rng)
    elif sid == "apple_canned":
        draw_tin(d, rng, (240, 235, 225, 255), "apple")
    elif sid == "apple_sugar":
        draw_jar(d, rng, (238, 190, 170, 255),
                 [(14, 17), (18, 20), (15, 23)], (216, 80, 60, 255))
    elif sid == "brandied_fruit":
        draw_jar(d, rng, (196, 130, 50, 255),
                 [(14, 16), (18, 19), (15, 22), (18, 24)],
                 (150, 60, 120, 255))
    elif sid == "can_peach":
        draw_tin(d, rng, (250, 226, 160, 255), "peach")
    elif sid == "cooked_elderberries":
        draw_bowl(d, rng, (62, 40, 84, 255), (96, 66, 128, 255), n=10)
    elif sid == "dry_fruit":
        draw_packet(d, rng, (196, 80, 140, 255), "flakes")
    elif sid == "fruit_leather":
        draw_fruit_leather(d, rng)
    elif sid == "grapeade_powder":
        draw_packet(d, rng, (120, 70, 170, 255), "grape")
    elif sid == "jerusalem_artichoke_fruitmash":
        draw_bowl(d, rng, (232, 196, 110, 255), (216, 90, 60, 255), n=7)
    elif sid == "orangeade_powder":
        draw_packet(d, rng, (240, 140, 30, 255), "orange")
    elif sid == "pumpkin_yeast_bread":
        draw_bread(d, rng)
    elif sid == "rehydrated_fruit":
        draw_bowl(d, rng, (226, 150, 165, 255), (200, 90, 110, 255), n=9)
    elif sid == "bow_scope":
        draw_bow_scope(d, rng)
    elif sid == "bow_sight":
        draw_bow_sight(d, rng, pins=5)
    elif sid == "bow_sight_pin":
        draw_bow_sight(d, rng, pins=1)
    elif sid == "holo_magnifier":
        draw_holo_magnifier(d, rng)
    elif sid == "improve_sights":
        draw_iron_sights(d, rng)
    elif sid == "pistol_scope":
        scope_body(d, rng, 9, 24)
    elif sid == "rifle_scope":
        scope_body(d, rng, 4, 29)
    elif sid == "rifle_scope_high_end_mount":
        scope_body(d, rng, 4, 29, rail=True)
    elif sid == "riv_scope":
        draw_riv_scope(d, rng)
    elif sid == "tele_sight":
        draw_tele(d, rng)
    elif sid == "tele_sight_pistol":
        draw_tele(d, rng, pistol=True)
    elif sid == "type99_scope":
        scope_body(d, rng, 6, 27, tube=(88, 76, 54, 255), antique=True)
    else:
        raise KeyError(sid)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    # verify the embedded list equals the authoritative batch file
    batch = os.path.join(os.path.expanduser("~"), "workspace",
                         "item-batches", "70.txt")
    with open(batch) as f:
        file_ids = [ln.strip() for ln in f if ln.strip()]
    assert IDS == file_ids, "embedded IDS do not match 70.txt"
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
                       "preview_item70.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item70.png {prev.size}")


if __name__ == "__main__":
    main()
