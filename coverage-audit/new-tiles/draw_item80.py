#!/usr/bin/env python3
"""ITEM80: sauces/spices, fencing gear, chainsaws, hats, grooming kits,
toys and a workshop tail -- CDDA Voxel coverage audit.

Families (48 ids, grounded in each id's game entry):

  Sauces and spices -- containers drawn as themselves: buffalo sauce
    (orange-red bottle, buffalo glyph on the label), hot sauce (deep
    red bottle, flame glyph), garlic powder (shaker jar, garlic bulb
    motif), seasoned salt (speckled tan jar, herb flecks), dry
    spicebush berries (heap of wrinkled dark-red berries), sprinkles
    (clear shaker full of multicolour sugar rods).
  Fencing -- the torso garment itself: light sheet-metal chest guard
    (lc steel plate, leather straps) and its hardened ch_steel twin
    (darker blue-gray), lamé foil (metallic vest, foil glyph), lamé
    saber (metallic jacket with sleeves, curved saber glyph), cotton
    plastron (asymmetric underarm protector), plastic plastron (rigid
    plate with molded cups).
  Chainsaws -- side profile, guide bar to the right: gasoline
    chainsaw (on) with chain blur and exhaust puffs; electric
    chainsaw off/on (red plastic, battery hump; on adds blur and
    sparks); corded electric off/on (trailing cord and plug). Plus
    the makeshift axe: a bare hammered steel axe head, no handle.
  Hats -- drinking hat (blue cap, two cans, hoses), golden hat (tall
    gold-leaf crown with moon-phase row), hijab (dark drape, face
    visible), niqab (dark veil, only the eyes visible), pointed hat
    (dark witch cone), wizard hat (light blue cone, stitched stars).
  Grooming -- emery board (grit stick), hair dye kit (open box with
    applicator bottle and brush), makeup kit (open case, colour pans,
    lipstick, nail polish), shaving kit (white travel case),
    makeshift haircut kit (leather roll, scissors and comb),
    makeshift shaving kit (leather pouch, straight razor).
  Toys -- fidget spinner (green, three lobes), threadbare cotton doll
    arms (cyan, fraying), buttons and thread (mismatched buttons,
    dull yellow thread), floppy doll legs (different colours and
    lengths), large figurine (cyan plastic figure), yoyo (light gray,
    on its string).
  Workshop tail -- glassblower's crucible (clay/steel pot), standing
    floodlight and directed floodlight (tripod lamps, the directed
    one hooded), tempering kiln (long light-blue steel box), wall
    light (bracket and shade), wall wiring (white cable coil, copper
    ends), chitin dust (pale pile, moth-wing fragment), and shed
    snakeskins: one-piece translucent skin with eye caps in little /
    standard / big / giant sizes, plus a handful of oversized brown
    scales.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "buffalo_sauce",
    "garlic_powder",
    "hot_sauce",
    "seasoning_salt",
    "spicebush_berries_smoked",
    "sprinkles",
    "ch_chestguard_metal_sheets_light",
    "chestguard_metal_sheets_light",
    "lame_foil",
    "lame_saber",
    "plastron_cotton",
    "plastron_plastic",
    "chainsaw_on",
    "elec_chainsaw_cord_off",
    "elec_chainsaw_cord_on",
    "elec_chainsaw_off",
    "elec_chainsaw_on",
    "makeshift_axe",
    "drinking_hat",
    "golden_hat",
    "hijab",
    "niqab",
    "pointed_hat",
    "wizard_hat_costume",
    "emery_board",
    "hair_dye_kit",
    "makeup_kit",
    "shavingkit",
    "survivor_hairtrimmer",
    "survivor_shavingkit",
    "fidget_spinner",
    "imperfect_doll_arms",
    "imperfect_doll_features",
    "imperfect_doll_legs",
    "toy_figurine_large",
    "yoyo",
    "glassblowers_crucible",
    "standing_directed_floodlight",
    "standing_floodlight",
    "steel_kiln",
    "wall_light",
    "wall_wiring",
    "insect_dust",
    "shed_snakeskin",
    "shed_snakeskin_big",
    "shed_snakeskin_giant",
    "shed_snakeskin_little",
    "shed_snakeskin_scales",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (52, 52, 58, 255)
WHITE = (240, 240, 238, 255)
PAPER = (240, 236, 226, 255)
SKIN = (232, 196, 158, 255)
STEEL = (150, 154, 160, 255)
STEEL_LT = (186, 190, 196, 255)
STEEL_DK = (96, 100, 108, 255)
CH_STEEL = (92, 106, 128, 255)   # hardened: dark blue-gray
LC_STEEL = (178, 182, 188, 255)  # light/mild: light matte gray
LEATHER = (122, 84, 50, 255)
WOOD = (96, 66, 38, 255)
RED = (196, 48, 40, 255)
GOLD = (224, 178, 62, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- sauces

def sauce_bottle(d, liquid, cap, motif):
    """Tall thin hot-sauce bottle, label band with a motif."""
    d.rectangle([13, 11, 18, 28], fill=liquid)            # body
    d.polygon([(13, 11), (18, 11), (17, 7), (14, 7)], fill=liquid)  # shoulder
    d.rectangle([14, 4, 17, 7], fill=cap)                  # cap
    d.rectangle([12, 17, 19, 24], fill=PAPER)              # label
    d.rectangle([12, 17, 19, 24], outline=INK, width=1)
    if motif == "buffalo":  # dark buffalo silhouette
        b = (90, 56, 34, 255)
        d.rectangle([13, 20, 17, 22], fill=b)              # body + hump
        d.point((13, 19), fill=b)
        d.rectangle([17, 21, 18, 23], fill=b)              # head, lowered
        d.point((18, 20), fill=b)                          # horn nub
        d.point((14, 23), fill=b)                          # legs
        d.point((16, 23), fill=b)
    elif motif == "flame":
        d.polygon([(15, 23), (14, 20), (15, 18), (16, 20),
                   (17, 19), (17, 23)], fill=(232, 110, 30, 255))
        d.polygon([(15, 23), (15, 21), (16, 20), (16, 23)],
                  fill=(248, 200, 70, 255))
    d.point((14, 13), fill=shade(liquid, 1.25))            # glass glint


def draw_buffalo_sauce(d, rng):
    sauce_bottle(d, (228, 106, 38, 255), RED, "buffalo")   # buttery orange-red


def draw_hot_sauce(d, rng):
    sauce_bottle(d, (188, 30, 26, 255), INK, "flame")      # flamin' deep red


def shaker_jar(d, fill_fn, lid_col, motif):
    """Short wide spice jar with a shaker lid."""
    d.rectangle([10, 12, 21, 28], fill=(214, 208, 196, 255))  # jar body
    fill_fn(11, 15, 20, 27)
    d.rectangle([9, 8, 22, 12], fill=lid_col)              # lid
    for hx in (12, 15, 18):                                # shaker holes
        d.point((hx, 10), fill=INK)
    d.rectangle([10, 19, 21, 25], fill=PAPER)              # label
    d.rectangle([10, 19, 21, 25], outline=INK, width=1)
    motif()


def draw_garlic_powder(d, rng):
    def fill(x0, y0, x1, y1):
        d.rectangle([x0, y0, x1, y1], fill=(238, 232, 214, 255))
    def motif():  # garlic bulb: white bulb, clove lines, green sprout
        d.ellipse([13, 20, 18, 24], fill=(246, 244, 236, 255))
        d.ellipse([13, 20, 18, 24], outline=(170, 164, 148, 255), width=1)
        d.line([15, 20, 15, 24], fill=(190, 184, 166, 255), width=1)
        d.point((15, 19), fill=(96, 150, 74, 255))
    shaker_jar(d, fill, STEEL_DK, motif)


def draw_seasoning_salt(d, rng):
    def fill(x0, y0, x1, y1):
        d.rectangle([x0, y0, x1, y1], fill=(196, 160, 110, 255))
        for _ in range(14):  # speckled herb/salt blend
            d.point((rng.randint(x0, x1), rng.randint(y0, y1)),
                    fill=rng.choice([(240, 236, 220, 255), (110, 84, 50, 255),
                                     (90, 120, 60, 255)]))
    def motif():  # herb flecks
        for px, py in ((12, 21), (15, 23), (18, 21), (14, 22), (17, 24)):
            d.point((px, py), fill=(90, 120, 60, 255))
    shaker_jar(d, fill, (150, 110, 62, 255), motif)


def draw_spicebush(d, rng):
    # heap of wrinkled dried berries, formerly red, now dark red-brown
    berries = [(10, 23), (14, 25), (18, 23), (21, 25), (12, 20), (16, 21),
               (20, 20), (14, 17), (18, 17), (16, 24)]
    for bx, by in berries:
        col = rng.choice([(128, 54, 38, 255), (112, 44, 32, 255),
                          (142, 66, 44, 255)])
        d.ellipse([bx - 2, by - 2, bx + 2, by + 2], fill=col)
        d.arc([bx - 2, by - 2, bx + 2, by + 2], start=200, end=320,
              fill=shade(col, 0.7), width=1)               # wrinkle
        d.point((bx - 1, by - 1), fill=shade(col, 1.3))    # highlight
    d.polygon([(7, 27), (10, 25), (9, 29)], fill=(96, 84, 44, 255))  # dry leaf
    d.polygon([(24, 28), (22, 25), (26, 26)], fill=(96, 84, 44, 255))


def draw_sprinkles(d, rng):
    d.rectangle([10, 12, 21, 28], fill=(226, 228, 224, 255))  # clear jar
    d.rectangle([10, 12, 21, 28], outline=STEEL, width=1)
    cols = [(224, 60, 60, 255), (70, 130, 220, 255), (80, 180, 80, 255),
            (248, 200, 60, 255), (240, 130, 190, 255), (250, 250, 250, 255)]
    for _ in range(46):  # sugar rods
        x, y = rng.randint(11, 19), rng.randint(13, 26)
        c = rng.choice(cols)
        if rng.random() < 0.5:
            d.line([x, y, x + 1, y], fill=c, width=1)
        else:
            d.line([x, y, x, y + 1], fill=c, width=1)
    d.rectangle([9, 8, 22, 12], fill=WHITE)                # shaker cap
    for hx in (12, 15, 18):
        d.point((hx, 10), fill=INK)


# ------------------------------------------------------------ fencing

def vest(d, fill, y1=26, sleeves=False):
    """Torso garment silhouette: shoulders, armholes, tapered waist."""
    d.polygon([(11, 5), (20, 5), (23, 8), (22, y1), (9, y1), (8, 8)],
              fill=fill)
    d.polygon([(11, 5), (20, 5), (23, 8), (22, y1), (9, y1), (8, 8)],
              outline=shade(fill, 0.72))
    d.rectangle([14, 5, 17, 8], fill=shade(fill, 0.85))    # neck opening
    if sleeves:
        d.polygon([(8, 8), (4, 10), (5, 19), (9, 18)], fill=fill)   # left arm
        d.polygon([(23, 8), (27, 10), (26, 19), (22, 18)], fill=fill)  # right


def metal_chestguard(d, steel, rivet_extra=False):
    # one sheet of metal covering only the upper torso, leather straps
    d.rectangle([13, 20, 18, 26], fill=LEATHER)            # hanging belt straps
    d.polygon([(9, 6), (22, 6), (23, 10), (22, 19), (9, 19), (8, 10)],
              fill=steel)
    d.polygon([(9, 6), (22, 6), (23, 10), (22, 19), (9, 19), (8, 10)],
              outline=shade(steel, 0.7))
    d.arc([9, 12, 22, 24], start=0, end=180, fill=shade(steel, 0.8),
          width=1)                                         # curved cut edge
    for rx, ry in ((10, 8), (21, 8), (10, 17), (21, 17), (15, 8)):
        d.point((rx, ry), fill=shade(steel, 0.55))         # rivets
    if rivet_extra:
        for rx, ry in ((12, 12), (19, 12), (15, 17)):
            d.point((rx, ry), fill=shade(steel, 0.55))
    d.line([11, 9, 11, 16], fill=shade(steel, 1.18), width=1)  # sheen
    d.rectangle([7, 12, 8, 15], fill=LEATHER)              # side straps
    d.rectangle([23, 12, 24, 15], fill=LEATHER)


def draw_chestguard_lc(d, rng):
    metal_chestguard(d, LC_STEEL)


def draw_chestguard_ch(d, rng):
    metal_chestguard(d, CH_STEEL, rivet_extra=True)        # hardened, darker


def lame(d, rng, weapon):
    fill = (198, 200, 206, 255)
    vest(d, fill, y1=27 if weapon == "saber" else 25,
         sleeves=(weapon == "saber"))
    for _ in range(26):  # woven steel-strand sparkle
        d.point((rng.randint(9, 22), rng.randint(6, 24)),
                fill=rng.choice([WHITE, STEEL]))
    if weapon == "foil":  # thin straight foil, diagonal
        d.line([12, 22, 20, 10], fill=INK, width=1)
        d.line([11, 23, 13, 21], fill=GOLD, width=1)       # guard
    else:  # saber: curved blade
        d.arc([10, 8, 24, 24], start=250, end=40, fill=INK, width=1)
        d.rectangle([11, 21, 14, 23], fill=GOLD)           # bell guard


def draw_lame_foil(d, rng):
    lame(d, rng, "foil")


def draw_lame_saber(d, rng):
    lame(d, rng, "saber")


def draw_plastron_cotton(d, rng):
    # asymmetric underarm protector: dominant shoulder + one torso side
    d.polygon([(12, 5), (21, 5), (23, 9), (22, 26), (10, 26), (9, 14),
               (10, 8)], fill=(238, 234, 224, 255))
    d.polygon([(12, 5), (21, 5), (23, 9), (22, 26), (10, 26), (9, 14),
               (10, 8)], outline=(176, 170, 156, 255))
    d.rectangle([10, 5, 15, 10], fill=(232, 228, 216, 255))  # shoulder cap
    for sx in range(12, 21, 3):  # stitch dashes
        d.point((sx, 24), fill=(176, 170, 156, 255))
    d.line([20, 10, 20, 22], fill=(214, 208, 194, 255), width=1)


def draw_plastron_plastic(d, rng):
    # rigid plastic plate with molded cups
    d.polygon([(10, 6), (21, 6), (23, 10), (22, 26), (9, 26), (8, 10)],
              fill=(242, 242, 244, 255))
    d.polygon([(10, 6), (21, 6), (23, 10), (22, 26), (9, 26), (8, 10)],
              outline=STEEL)
    for cx in (13, 18):  # molded cups
        d.ellipse([cx - 2, 12, cx + 2, 17], outline=STEEL_LT, width=1)
        d.point((cx - 1, 13), fill=WHITE)
    d.line([11, 8, 11, 11], fill=WHITE, width=1)           # gloss


# ---------------------------------------------------------- chainsaws

def chainsaw(d, rng, body, on=False, cord=False, electric=False):
    # guide bar to the right, chain along it
    d.rectangle([17, 13, 30, 17], fill=STEEL_LT)
    d.rectangle([17, 13, 30, 17], outline=STEEL_DK, width=1)
    if on:  # chain blur: alternating dashes + motion streaks
        for x in range(18, 30, 2):
            d.point((x, 13), fill=INK)
            d.point((x + 1, 17), fill=INK)
        d.line([18, 11, 29, 11], fill=(200, 200, 204, 255), width=1)
        d.line([19, 19, 30, 19], fill=(200, 200, 204, 255), width=1)
    else:
        for x in range(18, 30, 2):
            d.point((x, 14), fill=STEEL_DK)
            d.point((x, 16), fill=STEEL_DK)
    # engine/motor housing
    d.rectangle([4, 11, 17, 21], fill=body)
    d.rectangle([4, 11, 17, 21], outline=shade(body, 0.65), width=1)
    d.rectangle([5, 8, 12, 11], fill=INK)                  # rear handle
    d.line([14, 9, 17, 12], fill=INK, width=2)             # front handle
    if electric:  # battery hump
        d.rectangle([6, 7, 13, 10], fill=(60, 60, 66, 255))
        d.point((8, 8), fill=(90, 200, 90, 255))
    else:  # fuel cap + pull starter
        d.point((7, 13), fill=INK)
        d.ellipse([11, 14, 15, 18], outline=shade(body, 0.6), width=1)
    if cord:  # short cord trailing to a plug
        d.line([5, 21, 3, 25], fill=INK, width=1)
        d.line([3, 25, 6, 28], fill=INK, width=1)
        d.rectangle([6, 27, 9, 29], fill=INK)
        d.point((10, 27), fill=STEEL)
        d.point((10, 29), fill=STEEL)
    if on:
        if electric:  # sparks at the bar tip
            for px, py in ((30, 10), (28, 8), (31, 14), (29, 20)):
                d.point((px, py), fill=(250, 190, 60, 255))
        else:  # exhaust puffs at the rear
            for cx, cy, r in ((3, 7, 1), (1, 5, 2), (4, 3, 1)):
                d.ellipse([cx - r, cy - r, cx + r, cy + r],
                          fill=(168, 168, 172, 255))


def draw_chainsaw_on(d, rng):
    chainsaw(d, rng, (216, 92, 32, 255), on=True)          # gasoline, orange


def draw_elec_chainsaw_off(d, rng):
    chainsaw(d, rng, RED, electric=True)


def draw_elec_chainsaw_on(d, rng):
    chainsaw(d, rng, RED, on=True, electric=True)


def draw_elec_chainsaw_cord_off(d, rng):
    chainsaw(d, rng, (52, 96, 160, 255), cord=True, electric=True)


def draw_elec_chainsaw_cord_on(d, rng):
    chainsaw(d, rng, (52, 96, 160, 255), on=True, cord=True, electric=True)


def draw_makeshift_axe(d, rng):
    # bare steel axe head: a hammered chunk, one edge ground bright
    d.polygon([(8, 9), (20, 7), (24, 12), (23, 20), (14, 23), (8, 19)],
              fill=STEEL)
    d.polygon([(8, 9), (20, 7), (24, 12), (23, 20), (14, 23), (8, 19)],
              outline=STEEL_DK)
    d.line([23, 12, 22, 20], fill=(228, 230, 234, 255), width=1)  # edge
    d.rectangle([11, 12, 15, 16], fill=shade(STEEL, 0.72))  # eye slot (empty)
    for _ in range(12):  # hammer dents
        d.point((rng.randint(9, 21), rng.randint(9, 21)),
                fill=rng.choice([STEEL_DK, STEEL_LT]))


# --------------------------------------------------------------- hats

def draw_drinking_hat(d, rng):
    for cx in (6, 25):  # two drink cans on a wire harness
        d.rectangle([cx - 2, 8, cx + 2, 16], fill=STEEL_LT)
        d.rectangle([cx - 2, 10, cx + 2, 12], fill=RED)
        d.line([cx, 8, cx, 5], fill=STEEL_DK, width=1)
    d.line([8, 12, 12, 20], fill=(70, 70, 76, 255), width=1)   # hoses
    d.line([23, 12, 19, 20], fill=(70, 70, 76, 255), width=1)
    d.point((15, 22), fill=(70, 70, 76, 255))                  # mouthpiece
    d.pieslice([9, 8, 22, 20], start=180, end=360, fill=(62, 96, 178, 255))
    d.rectangle([7, 19, 24, 21], fill=(62, 96, 178, 255))      # brim band
    d.rectangle([7, 19, 24, 21], outline=shade((62, 96, 178, 255), 0.7))


def draw_golden_hat(d, rng):
    d.rectangle([11, 3, 20, 20], fill=GOLD)                # tall crown
    d.rectangle([11, 3, 20, 20], outline=shade(GOLD, 0.7), width=1)
    for sx in (14, 17):  # crown segments
        d.line([sx, 3, sx, 20], fill=shade(GOLD, 0.78), width=1)
    for i, cx in enumerate((12, 15, 18)):  # moon phases row
        d.ellipse([cx, 10, cx + 2, 13], fill=(250, 240, 200, 255))
        if i == 1:
            d.rectangle([cx + 1, 10, cx + 2, 13], fill=GOLD)
        if i == 2:
            d.ellipse([cx, 10, cx + 2, 13], fill=shade(GOLD, 0.8))
    d.rectangle([11, 17, 20, 20], fill=LEATHER)            # leather base band
    d.rectangle([6, 20, 25, 22], fill=shade(GOLD, 0.9))    # brim
    d.rectangle([6, 20, 25, 22], outline=shade(GOLD, 0.65), width=1)


def head_drape(d, drape, face=True, eyes_only=False):
    d.polygon([(16, 3), (23, 6), (24, 16), (25, 28), (7, 28), (8, 16),
               (9, 6)], fill=drape)
    d.polygon([(16, 3), (23, 6), (24, 16), (25, 28), (7, 28), (8, 16),
               (9, 6)], outline=shade(drape, 0.7))
    if face:  # oval of face left visible
        d.ellipse([12, 7, 20, 17], fill=SKIN)
        d.point((14, 11), fill=INK)
        d.point((18, 11), fill=INK)
        d.point((16, 15), fill=(190, 110, 100, 255))
    if eyes_only:  # eye slit in the veil
        d.rectangle([12, 10, 20, 13], fill=SKIN)
        d.point((14, 11), fill=INK)
        d.point((18, 11), fill=INK)


def draw_hijab(d, rng):
    head_drape(d, (86, 86, 94, 255), face=True)


def draw_niqab(d, rng):
    head_drape(d, (44, 44, 52, 255), face=False, eyes_only=True)


def pointed_cone(d, fill, bent=True):
    tip_x = 19 if bent else 16
    d.polygon([(16, 12), (tip_x, 2), (21, 13), (23, 21), (9, 21), (10, 13)],
              fill=fill)
    d.polygon([(16, 12), (tip_x, 2), (21, 13), (23, 21), (9, 21), (10, 13)],
              outline=shade(fill, 0.7))
    d.ellipse([5, 20, 27, 24], fill=fill)                  # brim
    d.ellipse([5, 20, 27, 24], outline=shade(fill, 0.7))


def draw_pointed_hat(d, rng):
    pointed_cone(d, (86, 86, 94, 255))


def draw_wizard_hat(d, rng):
    pointed_cone(d, (110, 160, 220, 255))
    for sx, sy in ((13, 8), (18, 12), (12, 15), (19, 17), (15, 5)):
        d.point((sx, sy), fill=(250, 226, 90, 255))        # stitched stars
        d.point((sx, sy - 1), fill=(250, 226, 90, 255))
        d.point((sx - 1, sy), fill=(250, 226, 90, 255))
        d.point((sx + 1, sy), fill=(250, 226, 90, 255))


# ----------------------------------------------------------- grooming

def draw_emery_board(d, rng):
    d.polygon([(6, 19), (8, 14), (26, 9), (27, 12), (9, 18)],
              fill=(172, 134, 88, 255))                    # cardboard stick
    d.polygon([(6, 19), (8, 14), (26, 9), (27, 12), (9, 18)],
              outline=shade((172, 134, 88, 255), 0.7))
    for _ in range(16):  # abrasive grit
        t = rng.random()
        x = int(8 + t * 17)
        y = int(15 - t * 4) + rng.randint(-1, 2)
        d.point((x, y), fill=rng.choice([(120, 88, 54, 255),
                                         (210, 180, 130, 255)]))


def kit_box(d, body, lid=True):
    if lid:  # open lid above
        d.rectangle([7, 5, 24, 12], fill=shade(body, 0.9))
        d.rectangle([7, 5, 24, 12], outline=shade(body, 0.65), width=1)
    d.rectangle([6, 13, 25, 27], fill=body)
    d.rectangle([6, 13, 25, 27], outline=shade(body, 0.65), width=1)


def draw_hair_dye_kit(d, rng):
    kit_box(d, (172, 134, 88, 255))                         # cardboard box
    d.rectangle([10, 6, 13, 13], fill=WHITE)               # applicator bottle
    d.polygon([(10, 6), (13, 6), (11, 3), (12, 3)], fill=(90, 140, 200, 255))
    d.line([19, 12, 22, 6], fill=WOOD, width=2)            # brush handle
    d.rectangle([21, 4, 24, 7], fill=INK)                  # brush head
    d.ellipse([16, 18, 23, 24], fill=(170, 205, 235, 255))  # plastic glove
    d.rectangle([8, 20, 13, 25], fill=(150, 80, 130, 255))  # dye tube


def draw_makeup_kit(d, rng):
    kit_box(d, (58, 44, 62, 255))                          # dark case
    d.rectangle([9, 7, 22, 11], fill=(206, 222, 230, 255))  # mirror in lid
    pans = [(200, 130, 140, 255), (160, 110, 80, 255),
            (226, 196, 160, 255), (178, 60, 70, 255)]
    for i, c in enumerate(pans):                           # colour pans
        d.rectangle([8 + i * 4, 16, 11 + i * 4, 20], fill=c)
    d.rectangle([9, 22, 11, 26], fill=INK)                 # lipstick case
    d.rectangle([9, 21, 11, 23], fill=(205, 50, 60, 255))  # lipstick tip
    d.rectangle([19, 21, 22, 26], fill=(150, 40, 90, 255))  # nail polish
    d.rectangle([20, 19, 21, 21], fill=INK)


def draw_shavingkit(d, rng):
    d.rectangle([8, 11, 23, 26], fill=(238, 238, 240, 255))  # white case
    d.rectangle([8, 11, 23, 26], outline=STEEL, width=1)
    d.line([8, 15, 23, 15], fill=STEEL, width=1)           # zipper
    for zx in range(10, 22, 3):
        d.point((zx, 15), fill=STEEL_DK)
    d.line([11, 21, 19, 19], fill=STEEL_DK, width=2)       # razor silhouette
    d.point((20, 19), fill=STEEL)
    d.ellipse([11, 22, 15, 25], fill=(226, 220, 200, 255))  # soap puck


def leather_kit(d, tools):
    d.rectangle([7, 13, 24, 27], fill=LEATHER)             # leather roll/pouch
    d.rectangle([7, 13, 24, 27], outline=shade(LEATHER, 0.65), width=1)
    d.line([7, 18, 24, 18], fill=shade(LEATHER, 0.75), width=1)  # flap seam
    d.point((15, 21), fill=GOLD)                           # snap button
    tools()


def draw_survivor_hairtrimmer(d, rng):
    def tools():
        d.line([10, 13, 8, 7], fill=STEEL_LT, width=1)     # scissor blades
        d.line([12, 13, 14, 7], fill=STEEL_LT, width=1)
        d.ellipse([7, 5, 10, 8], outline=STEEL_LT, width=1)  # finger rings
        d.ellipse([13, 5, 16, 8], outline=STEEL_LT, width=1)
        d.rectangle([19, 6, 23, 12], fill=(70, 50, 34, 255))  # comb
        for tx in range(19, 24):
            d.point((tx, 13), fill=(70, 50, 34, 255))
    leather_kit(d, tools)


def draw_survivor_shavingkit(d, rng):
    def tools():
        d.line([9, 13, 20, 5], fill=STEEL_LT, width=1)     # straight razor blade
        d.line([9, 13, 12, 16], fill=(50, 40, 30, 255), width=2)  # handle
        d.rectangle([20, 7, 23, 13], fill=(90, 130, 90, 255))  # glass bottle
        d.rectangle([21, 5, 22, 7], fill=STEEL_DK)
    leather_kit(d, tools)


# --------------------------------------------------------------- toys

def draw_fidget_spinner(d, rng):
    green = (70, 170, 80, 255)
    for cx, cy in ((16, 9), (9, 21), (23, 21)):            # three lobes
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=green)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_DK)
    d.ellipse([11, 11, 21, 21], fill=green)                # hub
    d.ellipse([13, 13, 19, 19], fill=STEEL)                # center bearing
    d.point((16, 16), fill=INK)


def draw_doll_arms(d, rng):
    cyan = (96, 200, 210, 255)
    for flip in (0, 1):  # two threadbare cloth arms
        x0 = 7 if flip == 0 else 19
        d.polygon([(x0, 8), (x0 + 5, 8), (x0 + 4, 20), (x0 + 1, 20)],
                  fill=cyan if flip == 0 else shade(cyan, 0.85))
        for fy in range(20, 24):  # fraying spun-cloth fingers
            d.point((x0 + 1 + (fy % 2), fy), fill=cyan)
            d.point((x0 + 3, fy + (1 if flip else 0)), fill=cyan)
        for sy in (11, 15):  # stitch dashes
            d.point((x0 + 2, sy), fill=shade(cyan, 0.6))


def draw_doll_features(d, rng):
    d.ellipse([7, 10, 15, 18], fill=(70, 70, 78, 255))     # big dark button
    for hx, hy in ((10, 12), (12, 12), (10, 15), (12, 15)):
        d.point((hx, hy), fill=INK)
    d.ellipse([18, 13, 24, 19], fill=(140, 100, 66, 255))  # smaller button
    for hx, hy in ((20, 15), (22, 17)):
        d.point((hx, hy), fill=INK)
    yel = (206, 178, 84, 255)                              # dull yellow thread
    d.line([6, 24, 12, 21], fill=yel, width=1)
    d.line([12, 21, 19, 24], fill=yel, width=1)
    d.line([19, 24, 25, 22], fill=yel, width=1)
    d.line([22, 26, 26, 26], fill=yel, width=1)            # loose end


def draw_doll_legs(d, rng):
    # floppy legs: different colours, different lengths
    d.polygon([(9, 6), (14, 6), (13, 24), (10, 24)], fill=(84, 84, 92, 255))
    d.ellipse([9, 23, 14, 26], fill=(84, 84, 92, 255))     # foot
    d.polygon([(18, 6), (23, 6), (22, 19), (19, 19)],
              fill=(96, 140, 146, 255))                    # shorter, teal-gray
    d.ellipse([18, 18, 23, 21], fill=(96, 140, 146, 255))
    for sy in (10, 15, 20):
        d.point((11, sy), fill=(52, 52, 58, 255))          # stitches
    for sy in (10, 14):
        d.point((20, sy), fill=(60, 92, 96, 255))


def draw_figurine(d, rng):
    cyan = (96, 200, 210, 255)
    d.ellipse([13, 4, 19, 10], fill=cyan)                  # head
    d.rectangle([12, 10, 20, 19], fill=cyan)               # torso
    d.rectangle([9, 11, 12, 19], fill=shade(cyan, 0.9))    # arms
    d.rectangle([20, 11, 23, 19], fill=shade(cyan, 0.9))
    d.rectangle([12, 19, 15, 26], fill=shade(cyan, 0.82))  # legs
    d.rectangle([17, 19, 20, 26], fill=shade(cyan, 0.82))
    d.ellipse([9, 26, 23, 28], fill=STEEL_DK)              # stand
    d.point((14, 7), fill=shade(cyan, 0.6))                # molded face dots
    d.point((17, 7), fill=shade(cyan, 0.6))
    d.line([12, 14, 20, 14], fill=shade(cyan, 0.7), width=1)  # mold seam


def draw_yoyo(d, rng):
    d.line([16, 2, 16, 9], fill=(200, 200, 200, 255), width=1)  # string
    d.arc([13, 0, 19, 5], start=180, end=360, fill=(200, 200, 200, 255),
          width=1)                                         # finger loop
    d.ellipse([8, 10, 15, 24], fill=STEEL_LT)              # two discs
    d.ellipse([17, 10, 24, 24], fill=STEEL_LT)
    d.ellipse([8, 10, 15, 24], outline=STEEL_DK)
    d.ellipse([17, 10, 24, 24], outline=STEEL_DK)
    d.rectangle([15, 15, 17, 18], fill=STEEL_DK)           # axle
    d.ellipse([10, 13, 13, 20], outline=STEEL, width=1)    # disc detail


# ----------------------------------------------------------- workshop

def draw_crucible(d, rng):
    d.rectangle([14, 4, 18, 7], fill=STEEL_DK)             # lifting lug
    d.polygon([(9, 8), (23, 8), (21, 26), (11, 26)], fill=(146, 100, 66, 255))
    d.polygon([(9, 8), (23, 8), (21, 26), (11, 26)], outline=WOOD)
    d.ellipse([10, 6, 22, 10], fill=(60, 48, 40, 255))     # dark mouth
    d.ellipse([12, 7, 20, 9], fill=(96, 60, 40, 255))      # inner clay
    for by in (13, 19):  # steel bands
        d.line([10, by, 22, by], fill=STEEL_DK, width=1)
    d.line([23, 16, 27, 20], fill=INK, width=1)            # power cord stub
    d.point((16, 22), fill=RED)                            # control knob


def floodlight(d, directed):
    d.line([16, 15, 9, 28], fill=STEEL_DK, width=1)        # tripod legs
    d.line([16, 15, 23, 28], fill=STEEL_DK, width=1)
    d.line([16, 15, 16, 28], fill=STEEL_DK, width=1)
    d.rectangle([15, 12, 17, 16], fill=STEEL_DK)           # post
    if directed:  # hooded head with barn doors, aimed one way
        d.polygon([(8, 5), (24, 5), (27, 8), (27, 14), (24, 17), (8, 17),
                   (5, 14), (5, 8)], fill=(70, 70, 76, 255))
        d.rectangle([9, 7, 23, 15], fill=(250, 240, 190, 255))
        d.line([27, 8, 29, 6], fill=INK, width=1)          # barn doors
        d.line([27, 14, 29, 16], fill=INK, width=1)
    else:
        d.rectangle([8, 5, 24, 16], fill=(70, 70, 76, 255))
        d.rectangle([10, 7, 22, 14], fill=(250, 240, 190, 255))
    for gx in range(12, 22, 3):  # lens grid
        d.line([gx, 7, gx, 14], fill=(210, 200, 160, 255), width=1)


def draw_floodlight(d, rng):
    floodlight(d, directed=False)


def draw_directed_floodlight(d, rng):
    floodlight(d, directed=True)


def draw_steel_kiln(d, rng):
    body = (150, 180, 205, 255)                            # light blue steel
    d.rectangle([3, 12, 29, 22], fill=body)                # very long box
    d.rectangle([3, 12, 29, 22], outline=shade(body, 0.65), width=1)
    for sx in (10, 17, 24):  # door seams
        d.line([sx, 12, sx, 22], fill=shade(body, 0.75), width=1)
    d.rectangle([25, 14, 28, 18], fill=(90, 110, 130, 255))  # control box
    d.point((26, 16), fill=(250, 240, 190, 255))           # dial
    d.rectangle([12, 9, 20, 12], fill=shade(body, 0.85))   # top handle/vent
    for lx in (6, 25):  # legs
        d.rectangle([lx, 22, lx + 2, 26], fill=STEEL_DK)


def draw_wall_light(d, rng):
    d.rectangle([5, 6, 8, 26], fill=STEEL_DK)              # wall backplate
    d.line([8, 9, 15, 9], fill=STEEL_DK, width=2)          # arm
    d.pieslice([10, 8, 22, 16], start=0, end=180, fill=(238, 238, 240, 255))
    d.pieslice([10, 8, 22, 16], start=0, end=180, outline=STEEL)
    d.ellipse([14, 13, 18, 17], fill=(250, 240, 190, 255))  # bulb
    d.point((11, 20), fill=(250, 240, 190, 255))           # light spill dots
    d.point((21, 21), fill=(250, 240, 190, 255))
    d.point((16, 23), fill=(250, 240, 190, 255))


def draw_wall_wiring(d, rng):
    for r in (9, 6, 3):  # coil of white-insulated cable
        d.ellipse([16 - r, 15 - r, 16 + r, 15 + r], outline=(228, 228, 228),
                  width=2)
        d.ellipse([16 - r, 15 - r, 16 + r, 15 + r], outline=STEEL, width=1)
    d.line([12, 23, 10, 27], fill=(228, 228, 228), width=2)  # loose ends
    d.line([20, 23, 22, 27], fill=(228, 228, 228), width=2)
    d.point((10, 28), fill=(196, 110, 50, 255))           # copper tips
    d.point((22, 28), fill=(196, 110, 50, 255))
    d.point((10, 29), fill=(196, 110, 50, 255))
    d.point((22, 29), fill=(196, 110, 50, 255))


def draw_insect_dust(d, rng):
    d.ellipse([8, 20, 24, 27], fill=(238, 234, 220, 255))  # pale dust pile
    d.ellipse([11, 17, 21, 23], fill=(242, 238, 226, 255))
    for _ in range(12):  # dirt specks
        d.point((rng.randint(10, 22), rng.randint(19, 26)),
                fill=rng.choice([(190, 182, 160, 255), (160, 148, 124, 255)]))
    # moth wing fragment on top
    d.ellipse([17, 9, 25, 15], fill=(226, 214, 196, 255))
    d.ellipse([17, 9, 25, 15], outline=(176, 164, 140, 255))
    d.line([18, 12, 24, 12], fill=(176, 164, 140, 255), width=1)
    d.point((20, 11), fill=(216, 130, 50, 255))           # wing scale colour
    d.point((22, 13), fill=(60, 56, 52, 255))


def snakeskin(d, rng, scale=1.0, w=5):
    """Sinuous translucent shed skin with scale pattern and eye caps."""
    col = (238, 238, 234, 255)
    pts_top, pts_bot = [], []
    n = int(20 * scale)
    for i in range(n + 1):
        x = 16 + (i - n / 2) * 1.1
        y = 7 + i * (20 * scale) / max(n, 1)
        cx = x + 3.2 * math.sin(i * 0.55)
        pts_top.append((cx - w / 2, y))
        pts_bot.append((cx + w / 2, y))
    d.polygon(pts_top + pts_bot[::-1], fill=col)
    d.line(pts_top, fill=(196, 196, 190, 255), width=1)
    d.line(pts_bot, fill=(196, 196, 190, 255), width=1)
    for i in range(2, n, 2):  # scale arcs across the band
        y = 7 + i * (20 * scale) / max(n, 1)
        cx = 16 + (i - n / 2) * 1.1 + 3.2 * math.sin(i * 0.55)
        d.arc([cx - w / 2, y - 2, cx + w / 2, y + 2], start=0, end=180,
              fill=(200, 200, 194, 255), width=1)
    hx = 16 + (0 - n / 2) * 1.1 + 3.2 * math.sin(0)  # head end: eye caps
    d.point((int(hx - 1), 6), fill=(210, 210, 204, 255))
    d.point((int(hx + 1), 6), fill=(210, 210, 204, 255))


def draw_snakeskin(d, rng):
    snakeskin(d, rng, scale=1.0, w=5)


def draw_snakeskin_little(d, rng):
    snakeskin(d, rng, scale=0.55, w=3)


def draw_snakeskin_big(d, rng):
    snakeskin(d, rng, scale=1.15, w=8)


def draw_snakeskin_giant(d, rng):
    snakeskin(d, rng, scale=1.25, w=12)


def draw_snakeskin_scales(d, rng):
    # a handful of greatly oversized individual scales, brown and hard
    scales = [(11, 10), (17, 9), (22, 12), (13, 16), (20, 17), (16, 22),
              (23, 22)]
    for cx, cy in scales:
        col = rng.choice([(146, 108, 70, 255), (132, 96, 60, 255),
                          (158, 120, 80, 255)])
        d.pieslice([cx - 4, cy - 3, cx + 4, cy + 5], start=0, end=180,
                   fill=col)
        d.arc([cx - 4, cy - 3, cx + 4, cy + 5], start=0, end=180,
              fill=shade(col, 0.65), width=1)
        d.point((cx - 1, cy), fill=shade(col, 1.25))       # hard sheen


# ---------------------------------------------------------------- main

DRAW = {
    "buffalo_sauce": draw_buffalo_sauce,
    "garlic_powder": draw_garlic_powder,
    "hot_sauce": draw_hot_sauce,
    "seasoning_salt": draw_seasoning_salt,
    "spicebush_berries_smoked": draw_spicebush,
    "sprinkles": draw_sprinkles,
    "ch_chestguard_metal_sheets_light": draw_chestguard_ch,
    "chestguard_metal_sheets_light": draw_chestguard_lc,
    "lame_foil": draw_lame_foil,
    "lame_saber": draw_lame_saber,
    "plastron_cotton": draw_plastron_cotton,
    "plastron_plastic": draw_plastron_plastic,
    "chainsaw_on": draw_chainsaw_on,
    "elec_chainsaw_cord_off": draw_elec_chainsaw_cord_off,
    "elec_chainsaw_cord_on": draw_elec_chainsaw_cord_on,
    "elec_chainsaw_off": draw_elec_chainsaw_off,
    "elec_chainsaw_on": draw_elec_chainsaw_on,
    "makeshift_axe": draw_makeshift_axe,
    "drinking_hat": draw_drinking_hat,
    "golden_hat": draw_golden_hat,
    "hijab": draw_hijab,
    "niqab": draw_niqab,
    "pointed_hat": draw_pointed_hat,
    "wizard_hat_costume": draw_wizard_hat,
    "emery_board": draw_emery_board,
    "hair_dye_kit": draw_hair_dye_kit,
    "makeup_kit": draw_makeup_kit,
    "shavingkit": draw_shavingkit,
    "survivor_hairtrimmer": draw_survivor_hairtrimmer,
    "survivor_shavingkit": draw_survivor_shavingkit,
    "fidget_spinner": draw_fidget_spinner,
    "imperfect_doll_arms": draw_doll_arms,
    "imperfect_doll_features": draw_doll_features,
    "imperfect_doll_legs": draw_doll_legs,
    "toy_figurine_large": draw_figurine,
    "yoyo": draw_yoyo,
    "glassblowers_crucible": draw_crucible,
    "standing_directed_floodlight": draw_directed_floodlight,
    "standing_floodlight": draw_floodlight,
    "steel_kiln": draw_steel_kiln,
    "wall_light": draw_wall_light,
    "wall_wiring": draw_wall_wiring,
    "insect_dust": draw_insect_dust,
    "shed_snakeskin": draw_snakeskin,
    "shed_snakeskin_big": draw_snakeskin_big,
    "shed_snakeskin_giant": draw_snakeskin_giant,
    "shed_snakeskin_little": draw_snakeskin_little,
    "shed_snakeskin_scales": draw_snakeskin_scales,
}


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
    # contact-sheet preview: 8 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item80.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item80.png {prev.size}")


if __name__ == "__main__":
    main()
