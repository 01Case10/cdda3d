#!/usr/bin/env python3
"""ITEM91: final mixed tail batch -- CDDA Voxel coverage audit.

37 unrelated ids, each drawn as itself from its game entry:

  honey_scraper (flat steel uncapping blade, wooden handle),
  horseshoes_wearable (pair of U-shaped steel hoof bands),
  integrated_giant_pincer (brown sclerotin crab claw),
  laptop_boris (worn laptop lid plastered with stickers),
  large_repairkit (gunsmith repair case with tools + battery),
  leotard (dark lycra torso garment), light_cell_rechargeable
  (green-banded light cell), light_minus_disposable_cell (thin
  coin cell), m1911_10mag (single-stack steel pistol magazine),
  mortar_m224 (60mm mortar on bipod + baseplate, firing tables),
  necronomicon (hefty leather grimoire with metal clasps),
  nl_safehouse_boiler (pot-bellied otherworldly boiler, three
  gauges, cage door), nl_safehouse_boiler_plans (graven slab with
  glowing etched boiler pictograph), nl_shrub_glowberries
  (glowing green berry bunch), null (void token, faint null
  glyph), obi_gi (broad black obi belt), office_letter_opener
  (silver dagger-like opener), outfit_storage (hanger/clothes
  storage bundle), piano (upright piano with keys), plectrum
  (red plastic pick), pressurized_tank_chem (2 L steel chemical
  canister), pseudo_bio_picklock (fleshy bio lockpick),
  rmi2_corpse (desiccated corpse, chest puncture wound),
  rubik_farthing (brass farthing, stern profile), scar_16s
  (civilian SCAR-L rifle), soup_blood (bowl of brown blood
  soup), staff_sling (wooden staff with leather sling),
  string_dimension_biosuit (white biosuit, gold hex lattice),
  survnote_cabin_reverberation_hint (hastily scribbled note,
  cabin sketch), tarp_raincatcher (yellow tarp on sticks),
  tearer (vehicle shredder with circular blades),
  tempered_glass_sheet (pale cyan glass sheet), text_gunsmith
  (hardbound Ballistics textbook), veh_tools_workshop
  (workshop station bench), void_spider_test_item (void weaver
  leg effigy), wax_slug (12ga shell, wax-filled top),
  zpap85mag (curved ZPAP M90 30-round magazine).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "honey_scraper",
    "horseshoes_wearable",
    "integrated_giant_pincer",
    "laptop_boris",
    "large_repairkit",
    "leotard",
    "light_cell_rechargeable",
    "light_minus_disposable_cell",
    "m1911_10mag",
    "mortar_m224",
    "necronomicon",
    "nl_safehouse_boiler",
    "nl_safehouse_boiler_plans",
    "nl_shrub_glowberries",
    "null",
    "obi_gi",
    "office_letter_opener",
    "outfit_storage",
    "piano",
    "plectrum",
    "pressurized_tank_chem",
    "pseudo_bio_picklock",
    "rmi2_corpse",
    "rubik_farthing",
    "scar_16s",
    "soup_blood",
    "staff_sling",
    "string_dimension_biosuit",
    "survnote_cabin_reverberation_hint",
    "tarp_raincatcher",
    "tearer",
    "tempered_glass_sheet",
    "text_gunsmith",
    "veh_tools_workshop",
    "void_spider_test_item",
    "wax_slug",
    "zpap85mag",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 44, 50, 255)
STEEL = (168, 172, 178, 255)
STEEL_DARK = (104, 108, 116, 255)
STEEL_LIGHT = (206, 210, 216, 255)
WOOD = (110, 78, 44, 255)
WOOD_DARK = (78, 54, 30, 255)
LEATHER = (122, 84, 52, 255)
LEATHER_DARK = (84, 56, 34, 255)
PAPER = (240, 236, 226, 255)
PAPER_DARK = (204, 198, 182, 255)
RED = (186, 44, 38, 255)
GREEN = (62, 148, 74, 255)
GOLD = (206, 164, 60, 255)
BRASS = (188, 150, 66, 255)
BLACK = (30, 30, 34, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- items

def d_honey_scraper(d, rng):
    # flat wide steel blade (left) with serrated uncapping edge, wood grip
    d.polygon([(3, 13), (19, 12), (19, 19), (3, 19)], fill=STEEL)
    d.polygon([(3, 13), (19, 12), (19, 19), (3, 19)], outline=INK)
    for x in range(4, 19, 3):  # serration teeth along the top edge
        d.point((x, 11), fill=STEEL_DARK)
    d.line([5, 16, 17, 15], fill=STEEL_LIGHT, width=1)
    d.rectangle([19, 13, 29, 18], fill=WOOD)  # handle
    d.rectangle([19, 13, 29, 18], outline=WOOD_DARK)
    d.point((27, 15), fill=STEEL_DARK)  # rivet
    d.point((22, 15), fill=STEEL_DARK)


def d_horseshoes(d, rng):
    # two U-shaped steel bands with nail holes
    for cx in (10, 22):
        for r in (6, 4):
            d.arc([cx - r, 6, cx + r, 6 + 2 * r], start=0, end=180,
                  fill=STEEL if r == 6 else STEEL_LIGHT, width=2)
        d.line([cx - 6, 12, cx - 6, 20], fill=STEEL, width=2)
        d.line([cx + 6, 12, cx + 6, 20], fill=STEEL, width=2)
        for hx, hy in ((-6, 15), (6, 15), (-5, 10), (5, 10), (0, 7)):
            d.point((cx + hx // 1 if hx else cx, hy), fill=INK)
        d.rectangle([cx - 7, 20, cx - 5, 22], fill=STEEL_DARK)
        d.rectangle([cx + 5, 20, cx + 7, 22], fill=STEEL_DARK)


def d_pincer(d, rng):
    chit = (128, 84, 48, 255)
    chit_dark = (92, 58, 32, 255)
    # fixed lower finger: big curved claw sweeping up-right
    d.polygon([(6, 26), (10, 12), (16, 6), (24, 4), (20, 10), (14, 14),
               (12, 22), (12, 27)], fill=chit)
    d.polygon([(6, 26), (10, 12), (16, 6), (24, 4), (20, 10), (14, 14),
               (12, 22), (12, 27)], outline=chit_dark)
    # movable upper finger, open
    d.polygon([(8, 24), (16, 16), (26, 12), (27, 15), (19, 21), (12, 26)],
              fill=shade(chit, 1.12))
    d.polygon([(8, 24), (16, 16), (26, 12), (27, 15), (19, 21), (12, 26)],
              outline=chit_dark)
    for tx, ty in ((16, 15), (19, 18), (15, 12)):  # inner teeth
        d.point((tx, ty), fill=chit_dark)
    d.ellipse([4, 23, 11, 29], fill=chit_dark)  # wrist stump


def d_laptop(d, rng):
    # lid exterior (back of screen) plastered with stickers, base below
    d.rectangle([6, 4, 26, 22], fill=(88, 92, 100, 255))
    d.rectangle([6, 4, 26, 22], outline=INK)
    d.rectangle([7, 5, 25, 21], fill=(104, 108, 118, 255))
    stickers = [(255, 120, 90), (120, 200, 120), (110, 160, 240),
                (250, 210, 90), (220, 120, 200), (140, 230, 220),
                (250, 160, 60)]
    spots = [(9, 7, 4, 3), (15, 6, 5, 4), (21, 8, 3, 3), (10, 13, 3, 4),
             (17, 12, 4, 3), (22, 15, 3, 4), (12, 17, 5, 3)]
    for (sx, sy, sw, sh), col in zip(spots, stickers):
        d.rectangle([sx, sy, sx + sw - 1, sy + sh - 1],
                    fill=col + (255,))
    d.rectangle([4, 23, 28, 26], fill=(70, 74, 82, 255))  # base edge
    d.rectangle([4, 23, 28, 26], outline=INK)
    d.line([8, 24, 24, 24], fill=(150, 154, 162, 255), width=1)


def d_repairkit(d, rng):
    # open steel case: lid up with foam, base with tools + battery pack
    d.rectangle([5, 3, 27, 11], fill=STEEL_DARK)
    d.rectangle([5, 3, 27, 11], outline=INK)
    d.rectangle([7, 5, 25, 9], fill=(60, 62, 68, 255))  # lid foam
    d.rectangle([4, 12, 28, 27], fill=(120, 126, 134, 255))
    d.rectangle([4, 12, 28, 27], outline=INK)
    d.rectangle([13, 10, 19, 12], fill=STEEL)  # handle
    # tools in foam slots: wrench, screwdrivers, brush
    d.line([8, 16, 12, 22], fill=STEEL_LIGHT, width=2)
    d.point((8, 15), fill=STEEL_LIGHT)
    d.line([16, 15, 16, 23], fill=RED, width=2)
    d.line([16, 23, 16, 25], fill=STEEL_LIGHT, width=1)
    d.line([21, 15, 21, 23], fill=(60, 110, 200, 255), width=2)
    d.line([21, 23, 21, 25], fill=STEEL_LIGHT, width=1)
    d.rectangle([24, 16, 27, 21], fill=BLACK)  # battery pack
    d.point((25, 17), fill=GREEN)


def d_leotard(d, rng):
    col = (66, 66, 74, 255)
    d.polygon([(11, 5), (14, 4), (18, 4), (21, 5), (23, 10), (22, 17),
               (24, 24), (19, 25), (17, 19), (15, 19), (13, 25), (8, 24),
               (10, 17), (9, 10)], fill=col)
    d.polygon([(11, 5), (14, 4), (18, 4), (21, 5), (23, 10), (22, 17),
               (24, 24), (19, 25), (17, 19), (15, 19), (13, 25), (8, 24),
               (10, 17), (9, 10)], outline=INK)
    # deep scoop back: paper-dark inset at the back panel
    d.polygon([(13, 6), (19, 6), (20, 12), (16, 14), (12, 12)],
              fill=shade(col, 1.35))
    d.line([11, 8, 10, 16], fill=shade(col, 0.8), width=1)
    d.line([21, 8, 22, 16], fill=shade(col, 0.8), width=1)


def d_light_cell(d, rng):
    # AA-style light battery, green rechargeable band
    d.rectangle([13, 6, 19, 7], fill=STEEL_LIGHT)  # nub cap
    d.rectangle([12, 8, 20, 27], fill=(210, 214, 220, 255))
    d.rectangle([12, 8, 20, 27], outline=INK)
    d.rectangle([12, 13, 20, 21], fill=GREEN)
    d.line([14, 17, 18, 17], fill=(240, 240, 240, 255), width=1)
    d.point((16, 15), fill=(240, 240, 240, 255))
    d.line([12, 24, 20, 24], fill=STEEL_DARK, width=1)
    d.point((16, 10), fill=INK)  # + mark post
    d.line([14, 10, 18, 10], fill=INK, width=1)


def d_coin_cell(d, rng):
    # thin coin-like disposable cell, seen edge-on at a tilt
    d.ellipse([8, 12, 24, 20], fill=STEEL)
    d.ellipse([8, 12, 24, 20], outline=INK)
    d.ellipse([8, 10, 24, 18], fill=STEEL_LIGHT)
    d.ellipse([8, 10, 24, 18], outline=INK)
    d.ellipse([13, 12, 19, 16], outline=STEEL_DARK)
    d.line([15, 14, 17, 14], fill=INK, width=1)  # + mark
    d.point((16, 13), fill=INK)
    d.point((16, 15), fill=INK)


def d_mag1911(d, rng):
    # single-stack steel pistol magazine with witness holes
    d.rectangle([13, 8, 19, 26], fill=STEEL)
    d.rectangle([13, 8, 19, 26], outline=INK)
    d.rectangle([12, 26, 20, 28], fill=STEEL_DARK)  # baseplate
    d.ellipse([14, 4, 18, 8], fill=BRASS)  # top round
    d.ellipse([14, 4, 18, 8], outline=INK)
    for hy in (12, 16, 20, 24):
        d.point((16, hy), fill=INK)  # witness holes
    d.line([14, 9, 14, 25], fill=STEEL_LIGHT, width=1)


def d_mortar(d, rng):
    # 60mm mortar: tube on bipod, round baseplate, firing-table tag
    d.line([10, 26, 24, 26], fill=STEEL_DARK, width=2)  # baseplate
    d.polygon([(13, 8), (16, 7), (22, 22), (19, 23)], fill=STEEL)
    d.polygon([(13, 8), (16, 7), (22, 22), (19, 23)], outline=INK)
    d.ellipse([12, 5, 17, 8], fill=STEEL_DARK)  # muzzle
    d.line([20, 20, 26, 27], fill=STEEL_DARK, width=2)  # bipod legs
    d.line([20, 20, 15, 27], fill=STEEL_DARK, width=2)
    d.rectangle([21, 12, 26, 17], fill=PAPER)  # firing tables
    d.rectangle([21, 12, 26, 17], outline=INK)
    for ly in (13, 15):
        d.line([22, ly, 25, ly], fill=(120, 120, 124, 255), width=1)
    d.point((18, 14), fill=RED)


def d_necronomicon(d, rng):
    # hefty leather book, metal clasps on the fore-edge, blank covers
    d.rectangle([8, 4, 25, 28], fill=LEATHER_DARK)
    d.rectangle([8, 4, 25, 28], outline=INK)
    d.rectangle([10, 6, 23, 26], outline=shade(LEATHER, 1.25))
    d.line([9, 5, 9, 27], fill=shade(LEATHER_DARK, 0.7), width=1)  # spine
    for cy in (9, 22):  # clasps
        d.rectangle([24, cy, 28, cy + 3], fill=BRASS)
        d.rectangle([24, cy, 28, cy + 3], outline=INK)
        d.point((26, cy + 1), fill=INK)
    d.line([25, 6, 25, 26], fill=PAPER_DARK, width=1)  # page block edge
    d.point((16, 16), fill=shade(LEATHER_DARK, 1.3))  # worn patch


def d_boiler(d, rng):
    body = (168, 62, 48, 255)
    # pot-bellied boiler: chimney, round body, cage door, three gauges
    d.rectangle([14, 2, 18, 6], fill=STEEL_DARK)  # chimney stub
    d.ellipse([7, 5, 25, 12], fill=shade(body, 1.1))
    d.rectangle([6, 9, 26, 27], fill=body)
    d.ellipse([6, 22, 26, 29], fill=shade(body, 0.85))
    d.rectangle([6, 9, 26, 27], outline=INK)
    for gx in (10, 16, 22):  # three unmarked gauges like alien eyes
        d.ellipse([gx - 2, 11, gx + 2, 15], fill=(232, 228, 210, 255))
        d.ellipse([gx - 2, 11, gx + 2, 15], outline=INK)
        d.point((gx, 13), fill=RED)
    # hinged cage over the combustion chamber
    d.rectangle([11, 18, 21, 26], fill=(52, 44, 40, 255))
    d.rectangle([11, 18, 21, 26], outline=STEEL_DARK)
    for bx in (14, 17, 20):
        d.line([bx, 18, bx, 26], fill=STEEL_DARK, width=1)
    d.line([11, 22, 21, 22], fill=STEEL_DARK, width=1)
    d.point((16, 23), fill=(250, 150, 60, 255))  # ember glow
    d.rectangle([24, 16, 27, 19], fill=STEEL_DARK)  # side handle/valve


def d_boiler_plans(d, rng):
    # thin otherworldly slab, glowing etched pictograph of the boiler
    d.polygon([(7, 5), (26, 3), (25, 27), (6, 28)], fill=(176, 180, 188, 255))
    d.polygon([(7, 5), (26, 3), (25, 27), (6, 28)], outline=INK)
    glow = (96, 230, 210, 255)
    # etched round vessel
    d.ellipse([11, 9, 21, 20], outline=glow)
    d.line([14, 9, 14, 6], fill=glow, width=1)
    d.line([18, 9, 18, 6], fill=glow, width=1)
    for gx in (13, 16, 19):
        d.point((gx, 12), fill=glow)
    d.rectangle([13, 16, 19, 19], outline=glow)
    # wavy animated groove lines beneath
    for row, ph in ((22, 0.0), (25, 1.3)):
        pts = [(x, row + 1.2 * math.sin((x - 8) * 0.7 + ph))
               for x in range(9, 24)]
        d.line(pts, fill=glow, width=1)
    d.point((9, 7), fill=glow)


def d_glowberries(d, rng):
    # bunch of glowing green berries on plastic-plant stems
    for sx, sy in ((12, 8), (16, 6), (20, 8)):
        d.line([16, 16, sx, sy + 4], fill=(90, 110, 70, 255), width=1)
    d.polygon([(20, 5), (26, 3), (25, 8), (21, 8)], fill=(86, 140, 70, 255))
    berries = [(10, 12), (15, 10), (20, 12), (12, 17), (18, 17), (15, 21),
               (22, 18), (9, 19)]
    for bx, by in berries:
        d.ellipse([bx - 3, by - 3, bx + 3, by + 3], fill=(96, 210, 90, 255))
        d.ellipse([bx - 3, by - 3, bx + 3, by + 3],
                  outline=(50, 120, 50, 255))
        d.point((bx - 1, by - 1), fill=(220, 255, 200, 255))  # glow core
        d.point((bx + 4, by + 3), fill=(150, 240, 140, 255))  # halo fleck


def d_null(d, rng):
    # void token: near-black disc, faint null glyph (circle + slash)
    d.ellipse([5, 5, 27, 27], fill=(12, 10, 18, 255))
    d.ellipse([5, 5, 27, 27], outline=(70, 62, 96, 255))
    d.ellipse([10, 10, 22, 22], outline=(96, 88, 128, 255))
    d.line([11, 21, 21, 11], fill=(96, 88, 128, 255), width=1)
    d.point((16, 16), fill=(50, 44, 72, 255))


def d_obi(d, rng):
    # broad black obi: wrapped band with a flat knot and hanging ends
    d.rectangle([5, 12, 27, 19], fill=BLACK)
    d.rectangle([5, 12, 27, 19], outline=(70, 70, 78, 255))
    d.line([5, 15, 27, 15], fill=(58, 58, 66, 255), width=1)
    d.rectangle([13, 11, 19, 20], fill=(40, 40, 46, 255))  # knot
    d.rectangle([13, 11, 19, 20], outline=(80, 80, 88, 255))
    d.polygon([(14, 20), (17, 20), (17, 27), (13, 26)], fill=BLACK)
    d.polygon([(17, 20), (20, 20), (21, 26), (18, 27)], fill=(38, 38, 44, 255))


def d_letter_opener(d, rng):
    # sterling silver dagger-like opener, diagonal
    d.line([7, 24, 20, 11], fill=STEEL_LIGHT, width=2)  # blade
    d.line([7, 24, 20, 11], fill=STEEL, width=1)
    d.polygon([(6, 25), (8, 23), (9, 26)], fill=STEEL_LIGHT)  # point end
    d.line([19, 10, 23, 14], fill=BRASS, width=2)  # crossguard
    d.line([21, 12, 26, 7], fill=(120, 90, 60, 255), width=3)  # handle
    d.point((26, 6), fill=BRASS)  # pommel
    d.point((12, 18), fill=(240, 244, 248, 255))  # glint


def d_outfit_storage(d, rng):
    # plastic storage bundle: hangers poking out of a clothes press box
    for hx in (10, 16, 22):  # hanger hooks + triangles
        d.arc([hx - 2, 3, hx + 2, 7], start=180, end=360, fill=STEEL_DARK,
              width=1)
        d.polygon([(hx - 4, 11), (hx + 4, 11), (hx, 7)], outline=STEEL_DARK)
    d.rectangle([5, 12, 27, 27], fill=(84, 88, 120, 255))
    d.rectangle([5, 12, 27, 27], outline=INK)
    d.rectangle([5, 12, 27, 15], fill=(104, 110, 146, 255))  # press lid
    for ly in (19, 23):  # folded clothes seams
        d.line([7, ly, 25, ly], fill=(140, 146, 178, 255), width=1)
    d.rectangle([14, 20, 18, 24], fill=(200, 90, 90, 255))  # shirt patch
    d.point((16, 13), fill=STEEL_LIGHT)


def d_piano(d, rng):
    # upright piano: dark wood body, key slip with white/black keys
    d.rectangle([5, 4, 27, 28], fill=(38, 30, 26, 255))
    d.rectangle([5, 4, 27, 28], outline=INK)
    d.rectangle([7, 6, 25, 12], fill=(52, 42, 36, 255))  # top panel
    d.line([7, 9, 25, 9], fill=GOLD, width=1)  # trim
    d.rectangle([6, 14, 26, 17], fill=(70, 56, 44, 255))  # fallboard
    d.rectangle([6, 18, 26, 22], fill=(238, 234, 224, 255))  # white keys
    for kx in range(7, 26, 2):
        d.line([kx, 18, kx, 22], fill=(170, 166, 158, 255), width=1)
    for kx in (8, 11, 13, 16, 18, 21, 23):  # black keys
        d.rectangle([kx, 18, kx + 1, 20], fill=BLACK)
    d.rectangle([6, 18, 26, 22], outline=INK)
    for px in (9, 16, 23):  # pedals
        d.rectangle([px, 26, px + 1, 28], fill=BRASS)


def d_plectrum(d, rng):
    # red plastic teardrop pick, point down
    d.polygon([(16, 28), (8, 18), (7, 10), (11, 5), (21, 5), (25, 10),
               (24, 18)], fill=RED)
    d.polygon([(16, 28), (8, 18), (7, 10), (11, 5), (21, 5), (25, 10),
               (24, 18)], outline=shade(RED, 0.6))
    d.polygon([(12, 8), (17, 7), (14, 14)], fill=shade(RED, 1.35))  # sheen
    d.point((19, 20), fill=shade(RED, 0.75))


def d_chem_tank(d, rng):
    # 2 L steel pressurized canister, valve + gauge + feed hose stub
    d.rectangle([14, 3, 18, 6], fill=STEEL_DARK)  # valve stem
    d.rectangle([11, 5, 21, 7], fill=BRASS)  # valve wheel block
    d.rectangle([11, 8, 21, 27], fill=STEEL)
    d.ellipse([11, 25, 21, 29], fill=STEEL_DARK)
    d.rectangle([11, 8, 21, 27], outline=INK)
    d.rectangle([11, 14, 21, 19], fill=(226, 160, 40, 255))  # hazard band
    d.polygon([(16, 15), (18, 18), (14, 18)], fill=INK)  # warning triangle
    d.ellipse([17, 9, 21, 13], fill=(232, 232, 226, 255))  # gauge
    d.ellipse([17, 9, 21, 13], outline=INK)
    d.point((19, 11), fill=RED)
    d.line([21, 10, 25, 13], fill=STEEL_DARK, width=2)  # hose stub
    d.line([13, 22, 13, 25], fill=STEEL_LIGHT, width=1)


def d_bio_picklock(d, rng):
    # fleshy organic lockpick: bulbous base, curved tendril pick
    flesh = (196, 130, 118, 255)
    d.ellipse([6, 18, 15, 27], fill=flesh)
    d.ellipse([6, 18, 15, 27], outline=shade(flesh, 0.6))
    d.point((9, 21), fill=shade(flesh, 0.7))
    d.point((12, 24), fill=shade(flesh, 0.7))
    pts = [(13, 20), (18, 16), (23, 12), (26, 7)]
    d.line(pts, fill=flesh, width=3)
    d.line(pts, fill=shade(flesh, 1.15), width=1)
    d.polygon([(26, 7), (29, 5), (27, 9)], fill=shade(flesh, 0.8))  # hook
    d.point((24, 10), fill=(150, 60, 60, 255))  # vein fleck


def d_corpse(d, rng):
    # desiccated corpse lying diagonally, ribs + chest puncture wound
    skin = (206, 192, 168, 255)
    skin_d = (168, 152, 128, 255)
    d.ellipse([6, 5, 12, 11], fill=skin)  # skull
    d.ellipse([6, 5, 12, 11], outline=skin_d)
    d.point((8, 8), fill=INK)  # eye socket
    d.point((11, 8), fill=INK)
    d.line([10, 10, 20, 18], fill=skin, width=4)  # torso
    for i in range(3):  # ribs
        d.line([13 + i * 2, 13 + i * 2, 17 + i * 2, 11 + i * 2],
               fill=skin_d, width=1)
    d.ellipse([15, 13, 19, 17], fill=(70, 40, 36, 255))  # puncture wound
    d.point((17, 15), fill=(30, 16, 14, 255))
    d.line([19, 19, 26, 26], fill=skin, width=2)  # leg
    d.line([17, 20, 21, 28], fill=skin, width=2)
    d.line([12, 12, 6, 18], fill=skin, width=2)  # arm
    d.line([6, 18, 4, 22], fill=skin_d, width=1)


def d_farthing(d, rng):
    # shiny brass farthing: beaded rim, stern profile facing right
    d.ellipse([5, 5, 27, 27], fill=BRASS)
    d.ellipse([5, 5, 27, 27], outline=shade(BRASS, 0.55))
    d.ellipse([7, 7, 25, 25], outline=shade(BRASS, 0.7))
    for a in range(0, 360, 30):  # beaded rim
        bx = 16 + int(9.5 * math.cos(math.radians(a)))
        by = 16 + int(9.5 * math.sin(math.radians(a)))
        d.point((bx, by), fill=shade(BRASS, 0.65))
    # profile silhouette
    d.polygon([(13, 10), (18, 9), (20, 12), (19, 14), (21, 15), (19, 16),
               (19, 19), (21, 22), (13, 22), (12, 16)], fill=shade(BRASS, 0.62))
    d.point((17, 12), fill=BRASS)  # eye
    d.arc([10, 20, 22, 26], start=0, end=180, fill=shade(BRASS, 0.62),
          width=1)  # inscription arc hint


def d_scar(d, rng):
    # civilian SCAR-L: tan polymer rifle in profile, muzzle right
    tan = (178, 148, 104, 255)
    d.rectangle([2, 13, 6, 17], fill=shade(tan, 0.8))  # buttstock
    d.rectangle([6, 12, 16, 18], fill=tan)  # receiver
    d.rectangle([6, 12, 16, 18], outline=INK)
    d.rectangle([16, 13, 27, 15], fill=shade(tan, 0.9))  # handguard
    d.rectangle([27, 13, 30, 14], fill=STEEL_DARK)  # muzzle
    d.line([7, 11, 26, 11], fill=STEEL_DARK, width=1)  # top rail
    for rx in range(9, 26, 3):
        d.point((rx, 10), fill=STEEL_DARK)
    d.polygon([(10, 18), (13, 18), (12, 23), (9, 23)], fill=shade(tan, 0.75))
    d.polygon([(15, 18), (19, 18), (20, 25), (17, 25)],
              fill=(60, 60, 64, 255))  # curved magazine
    d.point((8, 15), fill=INK)
    d.point((24, 16), fill=INK)


def d_soup_blood(d, rng):
    # bowl of mushy brown blood soup with flour lumps
    d.ellipse([6, 10, 26, 18], fill=(120, 52, 40, 255))
    d.ellipse([6, 10, 26, 18], outline=INK)
    for lx, ly in ((11, 13), (17, 12), (21, 14), (14, 15)):
        d.ellipse([lx - 1, ly - 1, lx + 2, ly + 1],
                  fill=(196, 170, 140, 255))  # flour lumps
    d.polygon([(6, 14), (26, 14), (23, 24), (9, 24)], fill=(96, 100, 108, 255))
    d.polygon([(6, 14), (26, 14), (23, 24), (9, 24)], outline=INK)
    d.line([22, 8, 27, 4], fill=STEEL, width=2)  # spoon handle
    d.ellipse([20, 7, 24, 10], fill=STEEL_LIGHT)


def d_staff_sling(d, rng):
    # wooden staff diagonally, leather pouch + cords at the top
    d.line([8, 28, 22, 6], fill=WOOD, width=3)
    d.line([8, 28, 22, 6], fill=shade(WOOD, 1.25), width=1)
    d.polygon([(21, 4), (26, 6), (24, 10), (20, 8)], fill=LEATHER)
    d.polygon([(21, 4), (26, 6), (24, 10), (20, 8)], outline=LEATHER_DARK)
    d.line([24, 8, 28, 13], fill=LEATHER_DARK, width=1)  # loose cord
    d.line([22, 9, 27, 15], fill=LEATHER_DARK, width=1)
    d.ellipse([23, 5, 26, 8], fill=(150, 148, 144, 255))  # loaded stone


def d_biosuit(d, rng):
    # stark-white bodysuit with a gold hex lattice
    suit = (238, 238, 240, 255)
    d.polygon([(12, 4), (20, 4), (22, 8), (21, 15), (23, 16), (22, 24),
               (19, 24), (18, 18), (17, 28), (14, 28), (13, 18), (12, 24),
               (9, 24), (10, 15), (10, 8)], fill=suit)
    d.polygon([(12, 4), (20, 4), (22, 8), (21, 15), (23, 16), (22, 24),
               (19, 24), (18, 18), (17, 28), (14, 28), (13, 18), (12, 24),
               (9, 24), (10, 15), (10, 8)], outline=INK)
    d.ellipse([13, 2, 19, 6], outline=GOLD)  # collar ring
    # hex lattice hints
    for hx, hy in ((13, 9), (18, 9), (15, 13), (12, 17), (19, 17),
                   (15, 21), (14, 25)):
        d.point((hx, hy), fill=GOLD)
        d.line([hx - 1, hy + 1, hx + 1, hy + 1], fill=GOLD, width=1)
    d.line([16, 6, 16, 16], fill=shade(GOLD, 0.9), width=1)  # front seam


def d_survnote(d, rng):
    # hastily scribbled note: torn paper, frantic lines, cabin sketch
    d.polygon([(8, 5), (24, 4), (26, 24), (18, 27), (7, 25)],
              fill=(238, 232, 214, 255))
    d.polygon([(8, 5), (24, 4), (26, 24), (18, 27), (7, 25)],
              outline=(120, 116, 104, 255))
    # cabin sketch, top
    d.polygon([(11, 12), (15, 8), (19, 12)], outline=INK)
    d.rectangle([12, 12, 18, 16], outline=INK)
    d.point((15, 14), fill=INK)
    # frantic scribbles
    for y, x1 in ((19, 23), (21, 20), (23, 24)):
        pts = [(x, y + rng.choice((-1, 0, 0, 1))) for x in range(10, x1)]
        d.line(pts, fill=(70, 70, 90, 255), width=1)
    d.line([10, 25, 22, 25], fill=RED, width=1)  # urgent underline


def d_raincatcher(d, rng):
    # yellow tarp funnel on crossed sticks, collecting a drip
    for x0, y0, x1, y1 in ((8, 10, 15, 27), (24, 10, 17, 27),
                           (10, 27, 22, 27)):
        d.line([x0, y0, x1, y1], fill=WOOD, width=2)
    d.polygon([(5, 6), (27, 6), (18, 15), (14, 15)], fill=(240, 200, 60, 255))
    d.polygon([(5, 6), (27, 6), (18, 15), (14, 15)], outline=shade(
        (240, 200, 60, 255), 0.6))
    d.line([7, 8, 15, 13], fill=(250, 220, 110, 255), width=1)
    d.line([25, 8, 17, 13], fill=(250, 220, 110, 255), width=1)
    d.polygon([(16, 18), (18, 21), (16, 23), (14, 21)],
              fill=(110, 170, 230, 255))  # water drop
    d.rectangle([12, 26, 20, 29], fill=(110, 170, 230, 255))  # puddle


def d_tearer(d, rng):
    # vehicle shredder: frame beam with two toothed circular blades
    d.rectangle([3, 6, 29, 10], fill=STEEL_DARK)
    d.rectangle([3, 6, 29, 10], outline=INK)
    for cx in (11, 21):
        for a in range(0, 360, 45):  # teeth
            tx = cx + int(7 * math.cos(math.radians(a)))
            ty = 19 + int(7 * math.sin(math.radians(a)))
            d.line([cx, 19, tx, ty], fill=STEEL, width=2)
        d.ellipse([cx - 5, 14, cx + 5, 24], fill=STEEL)
        d.ellipse([cx - 5, 14, cx + 5, 24], outline=INK)
        d.point((cx, 19), fill=INK)
        d.line([cx, 11, cx, 14], fill=STEEL_DARK, width=2)  # axle
    d.point((6, 8), fill=RED)
    d.point((26, 8), fill=RED)


def d_glass_sheet(d, rng):
    # large pale-cyan tempered glass sheet with edge tint + glare
    glass = (196, 228, 232, 255)
    d.polygon([(8, 4), (26, 6), (24, 28), (6, 26)], fill=glass)
    d.polygon([(8, 4), (26, 6), (24, 28), (6, 26)],
              outline=(110, 160, 168, 255))
    d.polygon([(8, 4), (12, 5), (9, 26), (6, 26)], fill=(224, 242, 244, 255))
    d.line([14, 8, 20, 20], fill=(240, 250, 250, 255), width=2)  # glare
    d.line([18, 7, 23, 16], fill=(240, 250, 250, 255), width=1)
    d.rectangle([20, 22, 23, 25], outline=(110, 160, 168, 255))  # stamp mark


def d_text_gunsmith(d, rng):
    # hefty hardbound textbook: dark cloth cover, title band, crosshair
    d.rectangle([8, 3, 24, 29], fill=(44, 56, 96, 255))
    d.rectangle([8, 3, 24, 29], outline=INK)
    d.line([10, 4, 10, 28], fill=shade((44, 56, 96, 255), 0.7), width=2)
    d.rectangle([12, 6, 22, 10], fill=GOLD)  # title band
    d.line([24, 5, 24, 27], fill=PAPER_DARK, width=1)  # page edge
    # crosshair + bullet motif
    d.ellipse([13, 14, 21, 22], outline=(230, 230, 235, 255))
    d.line([17, 12, 17, 24], fill=(230, 230, 235, 255), width=1)
    d.line([11, 18, 23, 18], fill=(230, 230, 235, 255), width=1)
    d.point((17, 18), fill=RED)
    d.polygon([(15, 24), (19, 24), (18, 27), (16, 27)], fill=BRASS)


def d_workshop(d, rng):
    # workshop station: bench top, drawers, hanging tools, cable + valve
    d.rectangle([4, 12, 28, 14], fill=WOOD)
    d.rectangle([4, 12, 28, 14], outline=WOOD_DARK)
    for lx in (6, 25):
        d.rectangle([lx, 15, lx + 2, 28], fill=WOOD_DARK)
    d.rectangle([10, 16, 22, 27], fill=(96, 100, 108, 255))  # drawer bank
    d.rectangle([10, 16, 22, 27], outline=INK)
    for dy in (18, 22, 26):
        d.line([10, dy, 22, dy], fill=INK, width=1)
        d.line([15, dy - 1, 17, dy - 1], fill=STEEL_LIGHT, width=1)
    # tools on the bench
    d.line([7, 8, 9, 12], fill=STEEL, width=2)  # wrench
    d.rectangle([12, 7, 15, 12], fill=RED)  # power tool body
    d.line([18, 9, 22, 12], fill=INK, width=1)  # cable
    d.ellipse([23, 7, 27, 11], outline=BRASS)  # valve wheel
    d.point((25, 9), fill=BRASS)


def d_void_leg(d, rng):
    # severed void weaver leg as an ornament: long segmented dark leg
    seg = (58, 40, 66, 255)
    pts = [(8, 4), (13, 10), (11, 16), (17, 21), (15, 27)]
    d.line(pts, fill=seg, width=3)
    for jx, jy in pts[1:-1]:  # joints in sickly red
        d.ellipse([jx - 2, jy - 2, jx + 2, jy + 2], fill=(196, 84, 84, 255))
    d.polygon([(15, 27), (12, 30), (17, 29)], fill=seg)  # foot point
    d.line([8, 4, 5, 2], fill=STEEL_DARK, width=1)  # mounting loop
    d.point((20, 8), fill=(150, 60, 70, 255))  # ichor fleck
    d.point((6, 20), fill=(150, 60, 70, 255))


def d_wax_slug(d, rng):
    # 12ga shell: brass base, red hull, opened top filled with pale wax
    d.rectangle([12, 22, 20, 28], fill=BRASS)
    d.rectangle([12, 22, 20, 28], outline=INK)
    d.rectangle([12, 10, 20, 22], fill=RED)
    d.rectangle([12, 10, 20, 22], outline=shade(RED, 0.6))
    d.ellipse([12, 6, 20, 11], fill=(238, 230, 205, 255))  # wax fill
    d.ellipse([12, 6, 20, 11], outline=shade(RED, 0.6))
    for px, py in ((14, 8), (17, 7), (19, 9)):  # shot dimples in wax
        d.point((px, py), fill=(120, 120, 124, 255))
    d.line([13, 14, 13, 21], fill=shade(RED, 1.3), width=1)
    d.point((16, 25), fill=STEEL_DARK)  # primer


def d_zpap_mag(d, rng):
    # curved 30-round AK-pattern magazine for the ZPAP M90
    d.polygon([(12, 5), (18, 5), (22, 14), (24, 24), (19, 26), (15, 16),
               (12, 12)], fill=(52, 54, 60, 255))
    d.polygon([(12, 5), (18, 5), (22, 14), (24, 24), (19, 26), (15, 16),
               (12, 12)], outline=INK)
    for t in (0.25, 0.5, 0.75):  # curve ribs
        rx = int(14 + 8 * t)
        ry = int(8 + 16 * t)
        d.line([rx - 2, ry, rx + 3, ry - 2], fill=(86, 90, 98, 255),
               width=1)
    d.rectangle([11, 3, 19, 5], fill=STEEL_DARK)  # feed lips
    d.ellipse([13, 1, 17, 4], fill=BRASS)  # top round
    d.rectangle([18, 24, 24, 27], fill=STEEL_DARK)  # baseplate


DRAW = {
    "honey_scraper": d_honey_scraper,
    "horseshoes_wearable": d_horseshoes,
    "integrated_giant_pincer": d_pincer,
    "laptop_boris": d_laptop,
    "large_repairkit": d_repairkit,
    "leotard": d_leotard,
    "light_cell_rechargeable": d_light_cell,
    "light_minus_disposable_cell": d_coin_cell,
    "m1911_10mag": d_mag1911,
    "mortar_m224": d_mortar,
    "necronomicon": d_necronomicon,
    "nl_safehouse_boiler": d_boiler,
    "nl_safehouse_boiler_plans": d_boiler_plans,
    "nl_shrub_glowberries": d_glowberries,
    "null": d_null,
    "obi_gi": d_obi,
    "office_letter_opener": d_letter_opener,
    "outfit_storage": d_outfit_storage,
    "piano": d_piano,
    "plectrum": d_plectrum,
    "pressurized_tank_chem": d_chem_tank,
    "pseudo_bio_picklock": d_bio_picklock,
    "rmi2_corpse": d_corpse,
    "rubik_farthing": d_farthing,
    "scar_16s": d_scar,
    "soup_blood": d_soup_blood,
    "staff_sling": d_staff_sling,
    "string_dimension_biosuit": d_biosuit,
    "survnote_cabin_reverberation_hint": d_survnote,
    "tarp_raincatcher": d_raincatcher,
    "tearer": d_tearer,
    "tempered_glass_sheet": d_glass_sheet,
    "text_gunsmith": d_text_gunsmith,
    "veh_tools_workshop": d_workshop,
    "void_spider_test_item": d_void_leg,
    "wax_slug": d_wax_slug,
    "zpap85mag": d_zpap_mag,
}


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    DRAW[sid](d, rng)
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
    # contact-sheet preview: 7 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item91.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item91.png {prev.size}")


if __name__ == "__main__":
    main()
