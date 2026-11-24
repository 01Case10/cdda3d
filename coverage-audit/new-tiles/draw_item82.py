#!/usr/bin/env python3
"""ITEM82: .50 AE ammo, holsters, cloaks/shields, pastries, laser
gunmods, radiosphere corpses, standalone shotguns, fuels/alcohols and
vehicle lifting gear -- CDDA Voxel coverage audit.

Families, each drawn from its game entry:
  .50 AE (5) -- one big fat upright cartridge; bullet profile by load
    (FMJ flat-nose jacketed, JHP with a dark hollow cavity) and finish
    by source (factory clean brass, bp_ soot-smudged black powder,
    matchhead_ reddish match-head smudge, reloaded_ scuffed two-tone).
  Holsters (5) -- XS hip holster (small leather, belt clip), belly
    band (wide lycra band with gun + mag pockets), deep concealment
    band (slim lycra, small pocket), fast-draw leather holster
    (low-cut), shoulder holster (leather harness, holster one side,
    double mag pouch the other).
  Cloaks/shields (5) -- artifact "distorted air" (empty shimmering
    ring, no fill), debug energy shield (solid paneled plate), sci-fi
    heraldic cape (sable/argent diagonal, full-moon sigil, holo
    specks), military poncho liner (quilted dark-gray blanket),
    makeshift poncho (glossy black trash bag, head/arm holes).
  Pastries (5) -- baklava (diamond-cut layers, nut specks), gateau
    St. Honore slice (choux balls on puff/cream), hazelnut cake slice
    (nuts on top), pumpkin muffin (domed top, ribbed cup), religieuse
    au chocolat (two stacked choux, ganache caps, cream ruff).
  Laser gunmods (5) -- steel components: beam scatterer (prism
    splitting the beam), effective emitter (finned, green LED),
    electrolaser conversion (electrode prongs, blue arcs), focusing
    lens (cyan lens, thin beam), high-density capacitor (fat
    cylinder, terminals, charge band).
  Radiosphere corpses (5) -- torn cloth flailer bird, jade yugg
    statue (segmented serpent), mutflesh yugg (fat flesh tube),
    rubber yugg (hollow black tube), dead solar beetle (open elytra
    with degraded solar-panel grid, splayed legs).
  Shotguns (5) -- full profiles: Browning A5 (humpback receiver,
    wood), DP-12 (bullpup, twin barrels/tubes, plastic), Ithaca 37
    (pump, corncob forend, wood), M26-MASS standalone (short, pistol
    grip, wire stock), Remington 1100 (sleek semi, vent rib, wood).
  Fuels/alcohols (10) -- containers by content: camphine tin (flame
    label), methanol lab bottle (hazard diamond), methanol fuel-cell
    cartridge, turpentine bottle (pine label), denatured alcohol
    (violet tint), Canadian whiskey (maple-leaf label), cheap whiskey
    (crooked plain label), single pot Irish whiskey (green glass),
    Bloody Mary (glass, celery, straw), 'very' Bloody Mary (dark
    blood red, rim drip, no celery).
  Vehicle gear (5) -- steel boom (truss), telescopic cantilever
    (nested sections), pallet lifter (mast + forks), airjack
    (telescoping pylon), motorcycle kickstand (angled leg + pad).

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "50ae_fmj",
    "50ae_jhp",
    "bp_50ae_fmj",
    "matchhead_50ae_fmj",
    "reloaded_50ae_fmj",
    "XS_holster",
    "bbholster",
    "bholster",
    "sholster",
    "shoulder_holster",
    "artifact_energy_shield",
    "cape_sci_fi",
    "debug_energy_shield",
    "military_poncho_liner",
    "poncho_makeshift",
    "baklava",
    "gateau_st_honore",
    "hazelnut_cake",
    "pumpkin_muffins",
    "religieuse_au_chocolat",
    "beam_scatterer",
    "effective_emitter",
    "electrolaser_conversion",
    "focusing_lens",
    "high_density_capacitor",
    "broken_flailer_radiosphere",
    "corpse_yugg_jade",
    "corpse_yugg_mutflesh",
    "corpse_yugg_rubber",
    "dead_solar_beetle_radiosphere",
    "browning_a5",
    "dp_12",
    "ithaca37",
    "m26_mass_standalone",
    "remington_1100",
    "camphine",
    "chem_methanol",
    "chem_methanol_fuelcell",
    "chem_turpentine",
    "denat_alcohol",
    "canadian_whiskey",
    "cheap_whiskey",
    "drink_bloody_mary",
    "drink_bloody_mary_blood",
    "single_pot_whiskey",
    "cantilever_medium",
    "cantilever_small",
    "pallet_lifter",
    "v_airjack_item",
    "v_motorcycle_kickstand_item",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (46, 46, 52, 255)
BRASS = (198, 160, 64, 255)
BRASS_PALE = (226, 194, 100, 255)
BRASS_DARK = (148, 114, 44, 255)
COPPER = (180, 104, 60, 255)
SOOT = (58, 54, 48, 255)
MATCH_SMUDGE = (146, 74, 44, 255)
LEATHER = (128, 84, 48, 255)
LEATHER_DARK = (96, 60, 32, 255)
LYCRA = (64, 66, 76, 255)
LYCRA_LIGHT = (110, 112, 124, 255)
STEEL = (150, 152, 156, 255)
STEEL_DARK = (96, 98, 104, 255)
STEEL_LIGHT = (196, 198, 204, 255)
WOOD = (122, 84, 46, 255)
WOOD_DARK = (90, 60, 30, 255)
PLASTIC_BLACK = (40, 40, 46, 255)
RED = (196, 48, 40, 255)
ENERGY = (110, 180, 250, 255)
ENERGY_PALE = (190, 226, 252, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# -------------------------------------------------------------------- ammo

def cartridge(d, rng, tip, finish):
    """One big .50 AE cartridge, upright, centred."""
    case = BRASS
    if finish == "bp":
        case = shade(BRASS, 0.66)
    elif finish == "match":
        case = shade(BRASS, 0.8)
    # bullet: y 4..12, slightly narrower than the fat case
    if tip == "fmj":  # jacketed flat nose
        d.rectangle([12, 5, 19, 12], fill=COPPER)
        d.rectangle([12, 5, 19, 6], fill=shade(COPPER, 0.8))
        d.line([13, 7, 13, 11], fill=shade(COPPER, 1.15), width=1)
    else:  # jhp: hollow cavity at the tip
        d.rectangle([12, 5, 19, 12], fill=COPPER)
        d.rectangle([14, 4, 17, 6], fill=(38, 34, 32, 255))
        d.line([13, 7, 13, 11], fill=shade(COPPER, 1.15), width=1)
    # case: y 12..27, rim y 27..29
    d.rectangle([11, 12, 20, 27], fill=case)
    d.line([11, 12, 20, 12], fill=shade(case, 0.78), width=1)
    d.line([13, 13, 13, 26], fill=shade(case, 1.16), width=1)
    d.rectangle([10, 27, 21, 29], fill=shade(case, 0.8))
    d.point((15, 28), fill=(150, 62, 40, 255))  # primer
    d.point((16, 28), fill=(150, 62, 40, 255))
    if finish == "bp":  # black-powder soot smudges
        for _ in range(16):
            d.point((rng.randint(11, 20), rng.randint(12, 27)), fill=SOOT)
        d.rectangle([11, 25, 20, 27], fill=shade(case, 0.72))
    elif finish == "match":  # match-head powder: reddish smudge, neater
        for _ in range(9):
            d.point((rng.randint(11, 20), rng.randint(18, 27)),
                    fill=MATCH_SMUDGE)
        d.line([18, 13, 18, 26], fill=shade(case, 1.1), width=1)
    elif finish == "reloaded":  # scuffed, two-tone mixed brass
        d.rectangle([16, 12, 20, 27], fill=shade(BRASS, 0.84))
        for _ in range(7):
            d.point((rng.randint(11, 20), rng.randint(13, 26)),
                    fill=BRASS_DARK)
        d.line([11, 19, 20, 19], fill=BRASS_DARK, width=1)


# ---------------------------------------------------------------- holsters

def holster_pouch(d, x0, y0, w, col, strap=True):
    """Pistol-pocket pouch: wide top, tapered barrel-down bottom."""
    h = w + 7
    d.polygon([(x0, y0), (x0 + w, y0), (x0 + w - 2, y0 + h - 4),
               (x0 + w - 5, y0 + h), (x0 + 2, y0 + h), (x0, y0 + h - 5)],
              fill=col)
    d.line([x0, y0, x0 + w, y0], fill=shade(col, 0.7), width=1)
    if strap:  # retention strap across the top
        d.line([x0 + 1, y0 + 3, x0 + w - 1, y0 + 3],
               fill=shade(col, 0.72), width=1)
        d.point((x0 + w // 2, y0 + 3), fill=STEEL_LIGHT)


def draw_holster(d, rng, sid):
    if sid == "XS_holster":  # small leather hip holster, belt clip
        d.rectangle([13, 4, 18, 8], fill=STEEL_DARK)  # belt clip
        holster_pouch(d, 11, 8, 10, LEATHER)
    elif sid == "sholster":  # fast draw: low-cut leather, belt loop
        d.rectangle([12, 3, 19, 7], fill=LEATHER_DARK)  # belt loop
        d.polygon([(10, 7), (21, 7), (20, 16), (18, 24), (13, 24),
                   (11, 16)], fill=LEATHER)
        d.line([11, 10, 20, 10], fill=LEATHER_DARK, width=1)  # low cut
        d.point((15, 10), fill=STEEL_LIGHT)
    elif sid == "bbholster":  # belly band: wide elastic band
        d.rectangle([2, 11, 29, 21], fill=LYCRA)
        for x in range(4, 29, 4):  # elastic stitch columns
            d.line([x, 12, x, 20], fill=shade(LYCRA, 1.3), width=1)
        d.rectangle([19, 12, 27, 20], outline=LYCRA_LIGHT, width=1)
        d.rectangle([5, 12, 10, 20], outline=LYCRA_LIGHT, width=1)
    elif sid == "bholster":  # deep concealment: slim band, small pocket
        d.rectangle([3, 13, 28, 19], fill=LYCRA_LIGHT)
        for x in range(5, 28, 3):
            d.line([x, 14, x, 18], fill=shade(LYCRA_LIGHT, 0.8), width=1)
        d.rectangle([12, 11, 19, 21], fill=shade(LYCRA_LIGHT, 0.86))
        d.rectangle([12, 11, 19, 21], outline=LYCRA, width=1)
    elif sid == "shoulder_holster":  # leather harness
        d.line([8, 3, 15, 14], fill=LEATHER, width=2)   # straps
        d.line([23, 3, 16, 14], fill=LEATHER, width=2)
        d.line([8, 3, 23, 3], fill=LEATHER, width=2)
        holster_pouch(d, 5, 13, 9, LEATHER)              # gun side
        d.rectangle([19, 14, 27, 22], fill=LEATHER)      # double mag side
        d.line([23, 14, 23, 22], fill=LEATHER_DARK, width=1)
        d.line([19, 17, 27, 17], fill=LEATHER_DARK, width=1)


# ------------------------------------------------------------------ cloaks

def draw_cloak(d, rng, sid):
    if sid == "artifact_energy_shield":  # distorted air: shimmer ring
        for r in (11, 9):
            d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=ENERGY,
                      width=1)
        for x in (10, 14, 18, 22):  # wavy distortion lines
            pts = [(x + (1 if (y // 3) % 2 else -1), y)
                   for y in range(8, 25, 2)]
            d.line(pts, fill=ENERGY_PALE, width=1)
        for _ in range(8):
            d.point((rng.randint(7, 25), rng.randint(7, 25)), fill=ENERGY)
    elif sid == "debug_energy_shield":  # solid debug panel
        d.rectangle([7, 5, 25, 27], fill=(104, 128, 158, 255))
        d.rectangle([7, 5, 25, 27], outline=STEEL_DARK, width=1)
        d.rectangle([9, 7, 23, 25], outline=ENERGY_PALE, width=1)
        for gy in range(11, 25, 4):
            d.line([10, gy, 22, gy], fill=(84, 106, 134, 255), width=1)
        d.ellipse([13, 12, 19, 18], outline=ENERGY, width=1)
        d.point((16, 15), fill=ENERGY)
        for px, py in ((8, 6), (24, 6), (8, 26), (24, 26)):
            d.point((px, py), fill=STEEL_LIGHT)
    elif sid == "cape_sci_fi":  # heraldic cape, sable & argent diagonal
        d.polygon([(9, 4), (23, 4), (27, 28), (5, 28)], fill=(30, 30, 36, 255))
        d.polygon([(23, 4), (27, 28), (14, 28)], fill=(206, 208, 214, 255))
        d.line([9, 4, 23, 4], fill=(70, 70, 78, 255), width=2)  # shoulders
        d.ellipse([13, 6, 19, 12], fill=(238, 238, 226, 255))  # full moon
        d.ellipse([13, 6, 19, 12], outline=(160, 160, 150, 255), width=1)
        for _ in range(10):  # holographic motif specks
            col = (90, 220, 220, 255) if rng.random() < 0.5 else (230, 120, 210, 255)
            d.point((rng.randint(7, 25), rng.randint(14, 27)), fill=col)
    elif sid == "military_poncho_liner":  # quilted woobie blanket
        d.rectangle([5, 8, 27, 26], fill=(84, 86, 90, 255))
        for i in range(-2, 6):  # diamond quilting
            d.line([5 + i * 5, 8, 5 + i * 5 + 9, 26], fill=(64, 66, 70, 255),
                   width=1)
            d.line([5 + i * 5 + 9, 8, 5 + i * 5, 26], fill=(64, 66, 70, 255),
                   width=1)
        d.rectangle([5, 24, 27, 26], fill=(96, 98, 102, 255))  # folded hem
        d.line([5, 8, 27, 8], fill=(110, 112, 116, 255), width=1)
    elif sid == "poncho_makeshift":  # black trash-bag poncho
        d.polygon([(12, 4), (20, 4), (26, 28), (6, 28)],
                  fill=(26, 26, 30, 255))
        d.ellipse([13, 3, 19, 7], fill=(8, 8, 10, 255))  # head hole
        d.ellipse([13, 3, 19, 7], outline=(90, 90, 96, 255), width=1)
        d.ellipse([7, 12, 10, 15], fill=(8, 8, 10, 255))   # arm holes
        d.ellipse([22, 12, 25, 15], fill=(8, 8, 10, 255))
        d.line([11, 9, 9, 24], fill=(120, 120, 128, 255), width=1)  # sheen
        d.line([21, 10, 23, 22], fill=(84, 84, 92, 255), width=1)


# ---------------------------------------------------------------- pastries

def draw_pastry(d, rng, sid):
    if sid == "baklava":
        d.rectangle([5, 11, 27, 24], fill=(196, 138, 62, 255))
        for y in (15, 19):  # filo layers
            d.line([5, y, 27, y], fill=(160, 104, 40, 255), width=1)
        for i in range(-1, 4):  # diamond cuts
            d.line([5 + i * 7, 11, 5 + i * 7 + 6, 24], fill=(140, 88, 30, 255),
                   width=1)
            d.line([5 + i * 7 + 6, 11, 5 + i * 7, 24], fill=(140, 88, 30, 255),
                   width=1)
        for _ in range(12):  # chopped nuts on top
            col = (110, 150, 70, 255) if rng.random() < 0.4 else (120, 84, 40, 255)
            d.point((rng.randint(6, 26), rng.randint(11, 13)), fill=col)
        d.line([6, 12, 26, 12], fill=(240, 200, 110, 255), width=1)  # syrup
    elif sid == "gateau_st_honore":
        d.rectangle([5, 23, 27, 26], fill=(214, 178, 116, 255))  # puff base
        d.rectangle([5, 20, 27, 23], fill=(246, 238, 214, 255))   # cream
        for cx in (9, 16, 23):  # choux balls with caramel glaze
            d.ellipse([cx - 3, 12, cx + 3, 20], fill=(232, 200, 140, 255))
            d.ellipse([cx - 3, 12, cx + 3, 16], fill=(196, 128, 52, 255))
            d.point((cx - 1, 13), fill=(250, 226, 160, 255))
    elif sid == "hazelnut_cake":
        d.polygon([(6, 26), (26, 26), (26, 13), (6, 19)], fill=(146, 96, 52, 255))
        d.polygon([(6, 19), (26, 13), (26, 16), (6, 22)],
                  fill=(104, 62, 30, 255))  # frosting slope
        for cx, cy in ((11, 15), (18, 13), (23, 12)):  # whole hazelnuts
            d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=(168, 116, 64, 255))
            d.point((cx, cy - 2), fill=(90, 58, 28, 255))
        for _ in range(8):  # nut specks in the crumb
            d.point((rng.randint(7, 25), rng.randint(20, 25)),
                    fill=(190, 140, 84, 255))
    elif sid == "pumpkin_muffins":
        d.polygon([(10, 17), (22, 17), (20, 27), (12, 27)],
                  fill=(196, 168, 128, 255))  # ribbed paper cup
        for x in (13, 16, 19):
            d.line([x, 18, x - 1, 26], fill=(164, 134, 92, 255), width=1)
        d.ellipse([8, 9, 24, 18], fill=(208, 116, 44, 255))  # domed top
        d.line([11, 11, 15, 13], fill=(170, 88, 30, 255), width=1)  # cracks
        d.line([17, 10, 20, 12], fill=(170, 88, 30, 255), width=1)
        for _ in range(6):
            d.point((rng.randint(10, 22), rng.randint(10, 16)),
                    fill=(150, 76, 26, 255))
    elif sid == "religieuse_au_chocolat":
        d.ellipse([8, 17, 24, 28], fill=(232, 200, 140, 255))  # large choux
        d.ellipse([8, 17, 24, 22], fill=(74, 44, 28, 255))      # ganache cap
        for x in range(9, 24, 2):  # cream ruff collar
            d.point((x, 16), fill=(250, 246, 236, 255))
            d.point((x + 1, 17), fill=(250, 246, 236, 255))
        d.ellipse([12, 7, 20, 16], fill=(232, 200, 140, 255))  # small choux
        d.ellipse([12, 7, 20, 11], fill=(74, 44, 28, 255))
        d.point((16, 6), fill=(250, 246, 236, 255))  # cream button


# ------------------------------------------------------------- laser parts

def laser_base(d):
    d.rectangle([8, 12, 22, 24], fill=STEEL)
    d.rectangle([8, 12, 22, 24], outline=STEEL_DARK, width=1)
    d.rectangle([11, 24, 19, 27], fill=STEEL_DARK)  # mount foot
    d.line([9, 14, 21, 14], fill=STEEL_LIGHT, width=1)


def draw_laser(d, rng, sid):
    if sid == "beam_scatterer":
        laser_base(d)
        d.polygon([(22, 15), (27, 18), (22, 21)], fill=(170, 220, 240, 255))
        for dy in (-4, 0, 4):  # diffracted fan of beams
            d.line([27, 18, 31, 18 + dy], fill=RED, width=1)
        d.point((12, 18), fill=(70, 70, 76, 255))
    elif sid == "effective_emitter":
        laser_base(d)
        for x in range(10, 22, 3):  # heatsink fins on top
            d.rectangle([x, 8, x + 1, 12], fill=STEEL_DARK)
        d.ellipse([17, 16, 21, 20], fill=(250, 220, 120, 255))  # emitter
        d.point((11, 21), fill=(70, 220, 90, 255))  # efficiency LED
    elif sid == "electrolaser_conversion":
        laser_base(d)
        d.rectangle([22, 14, 24, 17], fill=STEEL_DARK)  # electrode prongs
        d.rectangle([22, 20, 24, 23], fill=STEEL_DARK)
        d.line([24, 15, 28, 18], fill=(120, 190, 250, 255), width=1)  # arcs
        d.line([28, 18, 25, 21], fill=(120, 190, 250, 255), width=1)
        d.line([25, 18, 29, 18], fill=(190, 226, 252, 255), width=1)
        d.point((12, 18), fill=(120, 190, 250, 255))
    elif sid == "focusing_lens":
        d.rectangle([6, 14, 20, 22], fill=STEEL)  # lens barrel
        d.rectangle([6, 14, 20, 22], outline=STEEL_DARK, width=1)
        d.rectangle([10, 22, 16, 25], fill=STEEL_DARK)
        d.ellipse([19, 13, 25, 23], fill=(140, 210, 226, 255))  # convex lens
        d.ellipse([19, 13, 25, 23], outline=STEEL_DARK, width=1)
        d.point((21, 15), fill=(235, 250, 252, 255))
        d.line([25, 18, 31, 18], fill=RED, width=1)  # concentrated beam
    elif sid == "high_density_capacitor":
        d.rectangle([10, 10, 22, 27], fill=STEEL)  # fat cylinder body
        d.rectangle([10, 10, 22, 27], outline=STEEL_DARK, width=1)
        d.rectangle([12, 6, 14, 10], fill=STEEL_DARK)  # terminals
        d.rectangle([18, 6, 20, 10], fill=STEEL_DARK)
        d.rectangle([10, 19, 22, 23], fill=(44, 140, 132, 255))  # charge band
        d.line([11, 13, 21, 13], fill=(240, 200, 60, 255), width=1)  # warning
        d.line([12, 16, 12, 26], fill=STEEL_LIGHT, width=1)


# ---------------------------------------------------------------- corpses

def draw_corpse(d, rng, sid):
    if sid == "broken_flailer_radiosphere":  # torn cloth bird facsimile
        cloth = (206, 206, 210, 255)
        d.ellipse([8, 13, 22, 24], fill=cloth)  # body
        d.ellipse([20, 8, 27, 15], fill=cloth)  # head
        d.polygon([(27, 11), (31, 12), (27, 14)], fill=(226, 180, 90, 255))
        d.point((23, 11), fill=INK)  # button eye
        d.polygon([(10, 16), (3, 10), (5, 20), (9, 21)], fill=shade(cloth, 0.9))
        for fx in (8, 11, 14):  # tail feather strips
            d.line([fx, 23, fx - 2, 29], fill=shade(cloth, 0.82), width=2)
        for x in range(11, 20, 3):  # stitch dashes
            d.point((x, 18), fill=(140, 140, 146, 255))
        d.line([14, 20, 18, 24], fill=(240, 240, 244, 255), width=1)  # tear
        d.line([18, 24, 21, 22], fill=(240, 240, 244, 255), width=1)
        d.line([26, 15, 29, 18], fill=cloth, width=1)  # loose thread
    elif sid == "corpse_yugg_jade":  # jade statue of a worm-like yugg
        jade = (52, 148, 96, 255)
        d.rectangle([8, 27, 26, 29], fill=shade(jade, 0.55))  # base
        segs = [(12, 24, 9), (13, 19, 10), (15, 14, 9), (18, 10, 8),
                (21, 7, 6)]
        for cx, cy, r in segs:  # rising segmented coils
            d.ellipse([cx - r, cy - r + 3, cx + r, cy + r + 3], fill=jade)
            d.arc([cx - r, cy - r + 3, cx + r, cy + r + 3], start=200,
                  end=340, fill=shade(jade, 0.6), width=1)
            d.point((cx - r + 2, cy + 1), fill=(140, 230, 180, 255))
        d.point((22, 6), fill=(20, 80, 50, 255))  # eye pits
        d.point((24, 7), fill=(20, 80, 50, 255))
    elif sid == "corpse_yugg_mutflesh":  # massive fleshy tube
        flesh = (196, 130, 118, 255)
        d.ellipse([4, 11, 28, 25], fill=flesh)
        d.ellipse([4, 11, 28, 25], outline=shade(flesh, 0.7), width=1)
        for x in (10, 16, 22):  # fold creases
            d.arc([x - 4, 12, x + 4, 24], start=90, end=270,
                  fill=shade(flesh, 0.72), width=1)
        for _ in range(10):  # faintly green patches
            d.point((rng.randint(6, 26), rng.randint(13, 23)),
                    fill=(150, 170, 110, 255))
        d.line([7, 14, 14, 13], fill=(232, 180, 168, 255), width=1)  # sheen
        d.ellipse([24, 15, 28, 21], fill=shade(flesh, 0.6))  # open end
    elif sid == "corpse_yugg_rubber":  # hollow black rubber tube
        rub = (44, 44, 50, 255)
        d.ellipse([3, 11, 27, 25], fill=rub)
        d.ellipse([20, 12, 29, 24], fill=shade(rub, 1.25))  # rim of opening
        d.ellipse([23, 15, 28, 21], fill=(10, 10, 12, 255))  # hollow
        for x in (8, 13, 18):  # segment bulges
            d.arc([x - 4, 12, x + 4, 24], start=90, end=270,
                  fill=(24, 24, 28, 255), width=1)
        d.line([6, 14, 13, 13], fill=(110, 110, 118, 255), width=1)  # sheen
        d.polygon([(10, 25), (12, 29), (14, 25)], fill=rub)  # semi-liquid drip
        d.polygon([(19, 25), (20, 28), (22, 25)], fill=rub)
    elif sid == "dead_solar_beetle_radiosphere":
        shell = (86, 130, 196, 255)
        for lx, ly in ((8, 20), (7, 24), (9, 27), (23, 20), (24, 24),
                       (22, 27)):  # splayed legs
            d.line([16, 18, lx, ly], fill=STEEL_DARK, width=1)
        d.ellipse([9, 8, 23, 22], fill=shade(shell, 0.8))  # body underside
        # open elytra: two wing covers angled up, solar-panel outer faces
        for sgn in (-1, 1):
            x0 = 16 + (1 if sgn > 0 else -9)
            d.rectangle([x0, 3, x0 + 8, 12], fill=(38, 60, 110, 255))
            for gx in range(x0 + 2, x0 + 8, 2):
                d.line([gx, 3, gx, 12], fill=(110, 160, 220, 255), width=1)
            for gy in (6, 9):
                d.line([x0, gy, x0 + 8, gy], fill=(110, 160, 220, 255),
                       width=1)
            d.rectangle([x0, 3, x0 + 8, 12], outline=shade(shell, 0.7),
                        width=1)
        d.ellipse([13, 18, 19, 24], fill=STEEL_DARK)  # head
        d.point((14, 20), fill=(250, 90, 80, 255))  # dead eye lens
        for _ in range(7):  # scratches on the shell
            d.point((rng.randint(10, 22), rng.randint(9, 20)),
                    fill=(200, 216, 235, 255))


# --------------------------------------------------------------- shotguns

def gun_wood_stock(d, x1=12):
    d.polygon([(2, 14), (x1, 12), (x1, 18), (4, 21), (2, 19)], fill=WOOD)
    d.line([3, 16, 9, 14], fill=WOOD_DARK, width=1)


def draw_shotgun(d, rng, sid):
    if sid == "browning_a5":  # humpback semi-auto, wood
        gun_wood_stock(d)
        d.rectangle([11, 10, 19, 17], fill=STEEL_DARK)  # receiver
        d.rectangle([11, 8, 16, 10], fill=STEEL_DARK)   # the hump
        d.rectangle([19, 11, 30, 13], fill=STEEL)       # long barrel
        d.point((30, 10), fill=STEEL_LIGHT)             # bead
        d.rectangle([19, 14, 24, 17], fill=WOOD)        # forend
        d.line([13, 17, 15, 20], fill=INK, width=1)     # trigger
    elif sid == "remington_1100":  # sleek semi-auto, vent rib, wood
        gun_wood_stock(d)
        d.rectangle([11, 11, 19, 17], fill=STEEL_DARK)  # rounded receiver
        d.rectangle([19, 11, 30, 13], fill=STEEL)
        for x in range(20, 30, 2):  # vent rib posts
            d.point((x, 10), fill=STEEL_DARK)
        d.rectangle([19, 14, 25, 17], fill=WOOD)
        d.line([13, 17, 15, 20], fill=INK, width=1)
    elif sid == "ithaca37":  # pump-action, corncob forend, wood
        gun_wood_stock(d)
        d.rectangle([11, 11, 18, 17], fill=STEEL_DARK)
        d.rectangle([18, 11, 29, 13], fill=STEEL)
        d.point((29, 10), fill=STEEL_LIGHT)
        d.rectangle([19, 14, 24, 17], fill=WOOD)  # pump forend
        for x in range(20, 24):  # corncob grooves
            d.line([x, 14, x, 17], fill=WOOD_DARK, width=1)
        d.line([13, 17, 15, 20], fill=INK, width=1)
    elif sid == "dp_12":  # bullpup twin-barrel, plastic
        d.rectangle([3, 12, 8, 19], fill=PLASTIC_BLACK)  # butt pad at rear
        d.rectangle([7, 10, 22, 20], fill=(52, 54, 60, 255))  # bulky body
        d.rectangle([9, 7, 18, 10], fill=PLASTIC_BLACK)  # carry handle
        d.rectangle([22, 10, 30, 12], fill=STEEL_DARK)  # twin barrels
        d.rectangle([22, 14, 30, 16], fill=STEEL_DARK)
        d.rectangle([22, 17, 29, 19], fill=(52, 54, 60, 255))  # twin tubes
        d.rectangle([23, 20, 27, 23], fill=PLASTIC_BLACK)  # pump grip
        d.line([11, 20, 13, 23], fill=INK, width=1)
        d.point((29, 8), fill=STEEL_LIGHT)
    elif sid == "m26_mass_standalone":  # compact, pistol grip, wire stock
        d.line([2, 13, 9, 13], fill=STEEL_DARK, width=1)  # wire stock
        d.line([2, 16, 9, 15], fill=STEEL_DARK, width=1)
        d.rectangle([2, 12, 4, 17], fill=PLASTIC_BLACK)  # butt plate
        d.rectangle([9, 11, 20, 17], fill=(52, 54, 60, 255))  # receiver
        d.rectangle([20, 11, 29, 14], fill=STEEL_DARK)  # short barrel
        d.rectangle([20, 15, 26, 17], fill=STEEL)  # mag tube
        d.polygon([(13, 17), (16, 17), (15, 23), (12, 23)],
                  fill=PLASTIC_BLACK)  # pistol grip
        d.point((18, 9), fill=STEEL_LIGHT)  # bolt handle / sight


# ------------------------------------------------------------ fuels/drinks

def bottle(d, x0, x1, liquid, label_col, glass=(196, 216, 205, 255),
           cap=(120, 60, 30, 255)):
    """Straight bottle: neck y 4..9, body y 9..28."""
    cx0, cx1 = (x0 + x1) // 2 - 2, (x0 + x1) // 2 + 2
    d.rectangle([cx0, 4, cx1, 9], fill=glass)
    d.rectangle([cx0 - 1, 3, cx1 + 1, 5], fill=cap)
    d.polygon([(cx0, 9), (cx1, 9), (x1, 13), (x1, 28), (x0, 28), (x0, 13)],
              fill=glass)
    d.rectangle([x0 + 1, 14, x1 - 1, 27], fill=liquid)
    d.rectangle([x0 + 3, 17, x1 - 3, 24], fill=label_col)
    d.line([x0 + 2, 15, x0 + 2, 27], fill=(240, 248, 244, 255), width=1)


def draw_fuel(d, rng, sid):
    if sid == "camphine":  # lamp-fuel tin with flame label
        d.rectangle([9, 9, 23, 28], fill=(216, 210, 192, 255))
        d.rectangle([9, 9, 23, 28], outline=STEEL_DARK, width=1)
        d.rectangle([13, 5, 19, 9], fill=STEEL_DARK)  # cap neck
        d.rectangle([10, 5, 12, 8], fill=STEEL_DARK)  # handle stub
        d.polygon([(16, 13), (19, 19), (17, 23), (15, 23), (13, 19)],
                  fill=(240, 140, 40, 255))  # flame glyph
        d.polygon([(16, 17), (17, 20), (16, 22), (15, 20)],
                  fill=(250, 210, 90, 255))
        d.line([11, 26, 21, 26], fill=STEEL_DARK, width=1)
    elif sid == "chem_methanol":  # lab bottle, hazard diamond
        bottle(d, 10, 22, (196, 226, 235, 255), (240, 240, 240, 255),
               cap=(60, 60, 66, 255))
        d.polygon([(16, 18), (19, 21), (16, 24), (13, 21)],
                  fill=(240, 200, 60, 255))  # hazard diamond
        d.point((16, 21), fill=INK)
    elif sid == "chem_methanol_fuelcell":  # fuel-cell cartridge
        d.rectangle([9, 8, 23, 28], fill=(150, 156, 162, 255))
        d.rectangle([9, 8, 23, 28], outline=STEEL_DARK, width=1)
        d.rectangle([13, 4, 19, 8], fill=STEEL_DARK)  # connector cap
        d.point((16, 6), fill=(90, 200, 210, 255))
        d.rectangle([9, 15, 23, 19], fill=(44, 140, 132, 255))  # teal band
        for x in range(11, 23, 3):  # vent slots
            d.line([x, 23, x, 26], fill=STEEL_DARK, width=1)
    elif sid == "chem_turpentine":  # bottle, pine label
        bottle(d, 10, 22, (232, 214, 150, 255), (240, 240, 240, 255),
               cap=(96, 60, 32, 255))
        d.polygon([(16, 17), (19, 22), (13, 22)], fill=(70, 132, 62, 255))
        d.line([16, 22, 16, 24], fill=WOOD_DARK, width=1)
    elif sid == "denat_alcohol":  # violet-tinted bottle
        bottle(d, 10, 22, (168, 140, 205, 255), (240, 240, 240, 255),
               cap=(60, 60, 66, 255))
        d.line([14, 19, 18, 23], fill=RED, width=1)  # warning cross-out
        d.line([18, 19, 14, 23], fill=RED, width=1)
    elif sid == "canadian_whiskey":
        bottle(d, 10, 22, (198, 118, 34, 255), (178, 40, 36, 255),
               glass=(150, 110, 70, 255), cap=(178, 40, 36, 255))
        d.polygon([(16, 18), (17, 20), (19, 20), (17, 21), (18, 23),
                   (16, 22), (14, 23), (15, 21), (13, 20), (15, 20)],
                  fill=(245, 245, 245, 255))  # maple leaf hint
    elif sid == "cheap_whiskey":
        bottle(d, 11, 21, (172, 104, 36, 255), (216, 204, 178, 255),
               glass=(140, 104, 66, 255), cap=(90, 90, 96, 255))
        d.rectangle([13, 18, 19, 23], fill=(200, 188, 160, 255))  # crooked
        d.line([13, 23, 19, 22], fill=(160, 148, 124, 255), width=1)
    elif sid == "single_pot_whiskey":  # green glass Irish bottle
        bottle(d, 10, 22, (198, 118, 34, 255), (240, 234, 214, 255),
               glass=(52, 96, 62, 255), cap=(40, 40, 44, 255))
        d.point((16, 20), fill=(178, 40, 36, 255))  # red seal dot
    elif sid in ("drink_bloody_mary", "drink_bloody_mary_blood"):
        blood = sid.endswith("_blood")
        drink = (150, 16, 20, 255) if blood else (214, 52, 40, 255)
        d.polygon([(10, 9), (22, 9), (21, 27), (11, 27)],
                  fill=(210, 228, 235, 255))  # highball glass
        d.polygon([(11, 12), (21, 12), (20, 26), (12, 26)], fill=drink)
        d.line([19, 4, 16, 12], fill=(230, 120, 130, 255), width=1)  # straw
        if blood:  # drip over the rim, no garnish
            d.polygon([(11, 9), (13, 9), (12, 14), (11, 12)],
                      fill=(120, 10, 14, 255))
            d.point((20, 15), fill=(120, 10, 14, 255))
        else:  # celery stalk
            d.rectangle([12, 3, 14, 12], fill=(120, 190, 90, 255))
            d.point((13, 2), fill=(150, 210, 110, 255))
            d.point((17, 14), fill=(250, 220, 120, 255))  # spice fleck


# ------------------------------------------------------------ vehicle gear

def draw_vehicle(d, rng, sid):
    if sid == "cantilever_medium":  # large rigid steel boom: truss
        d.rectangle([2, 12, 29, 14], fill=STEEL)  # chords
        d.rectangle([2, 20, 29, 22], fill=STEEL)
        for x in range(3, 28, 5):  # webbing diagonals
            d.line([x, 14, x + 5, 20], fill=STEEL_DARK, width=1)
            d.line([x, 20, x + 5, 14], fill=STEEL_DARK, width=1)
        d.rectangle([2, 10, 6, 24], fill=STEEL_DARK)  # base mount
        d.line([28, 14, 28, 18], fill=INK, width=1)  # hook
        d.point((27, 19), fill=INK)
    elif sid == "cantilever_small":  # telescopic cantilever
        d.rectangle([3, 14, 14, 21], fill=STEEL)  # nested sections
        d.rectangle([13, 15, 21, 20], fill=shade(STEEL, 0.88))
        d.rectangle([20, 16, 28, 19], fill=shade(STEEL, 0.76))
        d.rectangle([3, 12, 7, 23], fill=STEEL_DARK)  # base mount
        d.point((28, 15), fill=INK)
        d.line([6, 17, 12, 17], fill=STEEL_DARK, width=1)
    elif sid == "pallet_lifter":  # mast + forks, makeshift
        d.rectangle([6, 5, 9, 27], fill=STEEL)  # mast
        d.rectangle([6, 5, 16, 8], fill=STEEL_DARK)  # top handle bar
        d.rectangle([9, 23, 27, 25], fill=STEEL)  # fork prongs
        d.rectangle([9, 18, 27, 20], fill=shade(STEEL, 0.85))
        d.rectangle([9, 18, 12, 25], fill=STEEL_DARK)  # carriage
        for px, py in ((7, 10), (7, 15), (7, 21)):  # weld beads
            d.point((px, py), fill=(240, 200, 90, 255))
        d.ellipse([5, 26, 9, 29], fill=INK)  # wheel
    elif sid == "v_airjack_item":  # extendable pylon
        d.rectangle([8, 26, 24, 29], fill=STEEL_DARK)  # base plate
        d.rectangle([13, 18, 19, 26], fill=STEEL)  # outer column
        d.rectangle([14, 11, 18, 18], fill=shade(STEEL, 0.85))  # inner
        d.rectangle([15, 6, 17, 11], fill=shade(STEEL, 0.72))  # ram
        d.rectangle([12, 4, 20, 6], fill=STEEL_DARK)  # saddle
        pts = [(20, 24), (25, 22), (26, 17), (24, 13)]  # air hose curve
        d.line(pts, fill=(60, 60, 66, 255), width=1)
    elif sid == "v_motorcycle_kickstand_item":
        d.rectangle([19, 5, 27, 9], fill=STEEL_DARK)  # pivot mount
        d.point((23, 7), fill=STEEL_LIGHT)  # pivot bolt
        d.line([21, 8, 10, 24], fill=STEEL, width=2)  # angled leg
        d.rectangle([5, 24, 12, 27], fill=STEEL_DARK)  # foot pad
        for i in range(3):  # return spring coils
            d.point((18 - i, 11 + i * 2), fill=STEEL_LIGHT)
            d.point((19 - i, 12 + i * 2), fill=STEEL_DARK)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid.startswith(("50ae", "bp_50ae", "matchhead_50ae", "reloaded_50ae")):
        tip = "jhp" if sid == "50ae_jhp" else "fmj"
        finish = ("bp" if sid.startswith("bp_") else
                  "match" if sid.startswith("matchhead_") else
                  "reloaded" if sid.startswith("reloaded_") else "factory")
        cartridge(d, rng, tip, finish)
    elif sid in ("XS_holster", "bbholster", "bholster", "sholster",
                 "shoulder_holster"):
        draw_holster(d, rng, sid)
    elif sid in ("artifact_energy_shield", "cape_sci_fi",
                 "debug_energy_shield", "military_poncho_liner",
                 "poncho_makeshift"):
        draw_cloak(d, rng, sid)
    elif sid in ("baklava", "gateau_st_honore", "hazelnut_cake",
                 "pumpkin_muffins", "religieuse_au_chocolat"):
        draw_pastry(d, rng, sid)
    elif sid in ("beam_scatterer", "effective_emitter",
                 "electrolaser_conversion", "focusing_lens",
                 "high_density_capacitor"):
        draw_laser(d, rng, sid)
    elif sid in ("broken_flailer_radiosphere", "corpse_yugg_jade",
                 "corpse_yugg_mutflesh", "corpse_yugg_rubber",
                 "dead_solar_beetle_radiosphere"):
        draw_corpse(d, rng, sid)
    elif sid in ("browning_a5", "dp_12", "ithaca37", "m26_mass_standalone",
                 "remington_1100"):
        draw_shotgun(d, rng, sid)
    elif sid in ("camphine", "chem_methanol", "chem_methanol_fuelcell",
                 "chem_turpentine", "denat_alcohol", "canadian_whiskey",
                 "cheap_whiskey", "drink_bloody_mary",
                 "drink_bloody_mary_blood", "single_pot_whiskey"):
        draw_fuel(d, rng, sid)
    elif sid in ("cantilever_medium", "cantilever_small", "pallet_lifter",
                 "v_airjack_item", "v_motorcycle_kickstand_item"):
        draw_vehicle(d, rng, sid)
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
    # contact-sheet preview: 8 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item82.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item82.png {prev.size}")


if __name__ == "__main__":
    main()
