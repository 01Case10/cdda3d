#!/usr/bin/env python3
"""ITEM63: chemicals + hair dye, and hand armor -- CDDA Voxel
coverage audit.

Cluster 1 (resources/chemicals.json, 19): reagent containers --
jar / bottle / tin / box / sachet / flask -- with the contents
colour visible and a simple label motif per chemical:
  baking soda: white powder in a paperboard box, scoop motif;
  calcium chloride: white powder jar, snowflake (de-icer) motif;
  lead azide + mercury fulminate: hazard-striped explosive tins
    (white vs gray powder, detonator-cap vs spark motif);
  potassium + sodium: gray metal chunks immersed in mineral oil
    in glass jars (amber oil, flame-off warning tick), told apart
    by chunk shape and label band colour;
  potassium alum: jar of white crystals, crystal-cluster motif;
  abstract hair dye: generic dye bottle, rainbow swatch stripe;
  dye powder: sachet with piles of colourful cloth-dye powder;
  hair dye (black/blond/blue/brown/gray/green/pink/red/white):
    applicator bottle with a hair-swatch stripe in the dye colour;
  mercury: a heavy squat flask of silver liquid metal, droplet
    motif and a meniscus line.

Cluster 2 (armor/gloves.json, 19): hand armor drawn as itself.
  chainmail hands: pairs of ring-mesh mittens in the five steel
    grades, colours exactly as ITEM47 (lc mild light matte, mc
    medium mid-gray, hc high bright polished, ch hardened dark
    blue-gray, qt tempered with a heat-tint sheen row), in
    standard, xl_ (larger) and xs_ (smaller) sizes;
  chainmail_junk_hands: patchwork rings with rust/corrosion
    patches and mismatched repair plates;
  gauntlet_fencing / _l: a SINGLE white suede fencing gauntlet
    (its description says a single glove) -- quilted padded back,
    cotton wrist cuff; the _l variant is left-handed (mirrored);
  gloves_eod: bulky EOD overhand protectors -- thick layered
    kevlar pads strapped over the backs of the hands.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "abstract_hair_dye",
    "chem_baking_soda",
    "chem_calcium_chloride",
    "chem_lead_azide",
    "chem_mercury_fulminate",
    "chem_potassium",
    "chem_potassium_alum",
    "chem_sodium",
    "dye_powder",
    "hair_dye_black",
    "hair_dye_blond",
    "hair_dye_blue",
    "hair_dye_brown",
    "hair_dye_gray",
    "hair_dye_green",
    "hair_dye_pink",
    "hair_dye_red",
    "hair_dye_white",
    "mercury",
    "ch_chainmail_hands",
    "chainmail_junk_hands",
    "gauntlet_fencing",
    "gauntlet_fencing_l",
    "gloves_eod",
    "hc_chainmail_hands",
    "lc_chainmail_hands",
    "mc_chainmail_hands",
    "qt_chainmail_hands",
    "xl_ch_chainmail_hands",
    "xl_hc_chainmail_hands",
    "xl_lc_chainmail_hands",
    "xl_mc_chainmail_hands",
    "xl_qt_chainmail_hands",
    "xs_ch_chainmail_hands",
    "xs_hc_chainmail_hands",
    "xs_lc_chainmail_hands",
    "xs_mc_chainmail_hands",
    "xs_qt_chainmail_hands",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

# steel grades, exactly as ITEM47: (base, dark, light)
STEELS = {
    "lc": ((168, 172, 176, 255), (124, 128, 132, 255), (196, 200, 204, 255)),
    "mc": ((128, 132, 138, 255), (84, 88, 96, 255), (170, 174, 180, 255)),
    "hc": ((196, 202, 212, 255), (108, 116, 128, 255), (248, 250, 252, 255)),
    "ch": ((74, 86, 106, 255), (42, 52, 70, 255), (118, 134, 158, 255)),
    "qt": ((138, 134, 148, 255), (88, 84, 102, 255), (186, 180, 196, 255)),
}
TINT_STRAW = (216, 178, 92, 255)
TINT_BLUE = (84, 110, 190, 255)
TINT_PURPLE = (150, 92, 180, 255)

GLASS = (196, 214, 218, 255)
GLASS_DARK = (120, 140, 146, 255)
LABEL = (238, 234, 222, 255)
LABEL_EDGE = (120, 116, 104, 255)
HAZARD_Y = (240, 196, 48, 255)
HAZARD_K = (28, 28, 30, 255)
RED = (196, 48, 40, 255)
WHITE_POWDER = (246, 246, 242, 255)
GRAY_POWDER = (168, 168, 172, 255)
OIL = (226, 190, 110, 255)

DYE_COLOURS = {
    "black": (24, 22, 26, 255),
    "blond": (232, 196, 92, 255),
    "blue": (48, 84, 196, 255),
    "brown": (110, 70, 40, 255),
    "gray": (128, 128, 132, 255),
    "green": (52, 156, 72, 255),
    "pink": (240, 140, 180, 255),
    "red": (198, 44, 36, 255),
    "white": (242, 240, 236, 255),
}


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- containers

def jar(d, x0, x1, y0, y1, contents, cap=(150, 60, 48, 255)):
    """Glass jar: screw cap, glass rim, contents fill, front label."""
    d.rectangle([x0 + 2, y0, x1 - 2, y0 + 2], fill=cap)
    d.line([x0 + 2, y0 + 1, x1 - 2, y0 + 1], fill=shade(cap, 0.7), width=1)
    d.rectangle([x0, y0 + 3, x1, y1], fill=GLASS)
    d.rectangle([x0 + 1, y0 + 6, x1 - 1, y1 - 1], fill=contents)
    d.line([x0 + 1, y0 + 3, x0 + 1, y1], fill=(255, 255, 255, 255),
           width=1)  # glass glint
    d.rectangle([x0, y0 + 3, x1, y1], outline=GLASS_DARK, width=1)


def label(d, x0, x1, yc, motif_fn):
    d.rectangle([x0 + 3, yc - 4, x1 - 3, yc + 4], fill=LABEL)
    d.rectangle([x0 + 3, yc - 4, x1 - 3, yc + 4], outline=LABEL_EDGE,
                width=1)
    motif_fn((x0 + x1) // 2, yc)


def powder_top(d, rng, x0, x1, y, col):
    """Uneven powder surface speckle along a fill line."""
    for x in range(x0 + 2, x1 - 1):
        if rng.random() < 0.5:
            d.point((x, y - rng.randint(0, 1)), fill=col)


# label motifs, drawn around centre (cx, cy)

def m_scoop(d, cx, cy):
    d.line([cx - 3, cy + 2, cx + 2, cy - 2], fill=(150, 110, 60, 255),
           width=2)
    d.polygon([(cx + 1, cy - 3), (cx + 4, cy - 1), (cx + 2, cy + 1)],
              fill=(190, 190, 196, 255))


def m_flake(d, cx, cy):
    col = (90, 140, 200, 255)
    for dx, dy in ((0, -3), (0, 3), (-3, 0), (3, 0), (-2, -2), (2, 2),
                   (-2, 2), (2, -2)):
        d.line([cx, cy, cx + dx, cy + dy], fill=col, width=1)


def m_crystal(d, cx, cy):
    col = (110, 130, 150, 255)
    d.polygon([(cx - 3, cy + 3), (cx - 2, cy - 2), (cx, cy + 3)],
              outline=col)
    d.polygon([(cx - 1, cy + 3), (cx + 1, cy - 3), (cx + 3, cy + 3)],
              outline=col)
    d.point((cx + 1, cy - 1), fill=(230, 240, 246, 255))


def m_chunk(d, cx, cy, col):
    d.polygon([(cx - 3, cy + 2), (cx - 1, cy - 2), (cx + 2, cy - 1),
               (cx + 3, cy + 2)], fill=col)
    d.point((cx, cy), fill=shade(col, 1.35))


def m_drop(d, cx, cy, col=(240, 240, 240, 255)):
    d.polygon([(cx, cy - 3), (cx + 2, cy + 1), (cx + 1, cy + 3),
               (cx - 1, cy + 3), (cx - 2, cy + 1)], fill=col)
    d.point((cx - 1, cy + 1), fill=(255, 255, 255, 255))


def hazard_lid(d, x0, x1, y0):
    for i, x in enumerate(range(x0, x1, 4)):
        col = HAZARD_Y if i % 2 == 0 else HAZARD_K
        d.rectangle([x, y0, min(x + 3, x1), y0 + 2], fill=col)


# ------------------------------------------------------------ cluster 1

def draw_box_soda(d, rng):
    # paperboard box of white powder, flap open, powder heap on top
    d.rectangle([8, 9, 24, 28], fill=(232, 138, 52, 255))
    d.rectangle([8, 9, 24, 28], outline=(150, 84, 28, 255), width=1)
    d.polygon([(8, 9), (24, 9), (21, 5), (11, 5)],
              fill=(246, 168, 84, 255))  # open flap
    d.polygon([(12, 6), (20, 6), (18, 4), (14, 4)],
              fill=WHITE_POWDER)  # powder heap
    d.rectangle([10, 13, 22, 22], fill=LABEL)
    d.rectangle([10, 13, 22, 22], outline=LABEL_EDGE, width=1)
    m_scoop(d, 16, 17)
    for _ in range(6):  # spilt grains at the base
        d.point((rng.randint(7, 25), rng.randint(26, 29)),
                fill=WHITE_POWDER)


def draw_powder_jar(d, rng, motif):
    jar(d, 9, 23, 4, 28, WHITE_POWDER)
    powder_top(d, rng, 9, 23, 11, WHITE_POWDER)
    label(d, 9, 23, 19, lambda cx, cy: motif(d, cx, cy))


def draw_explosive_tin(d, rng, powder, motif):
    # squat metal tin, hazard-striped lid, powder visible at the rim
    d.rectangle([7, 8, 25, 27], fill=(158, 160, 166, 255))
    d.rectangle([7, 8, 25, 27], outline=(88, 90, 96, 255), width=1)
    hazard_lid(d, 7, 25, 5)
    d.rectangle([7, 5, 25, 7], outline=(88, 90, 96, 255), width=1)
    d.rectangle([9, 10, 23, 13], fill=powder)  # powder at the open rim
    powder_top(d, rng, 9, 23, 10, powder)
    d.rectangle([10, 16, 22, 24], fill=LABEL)
    d.rectangle([10, 16, 22, 24], outline=RED, width=1)
    motif(d, 16, 20)
    d.line([8, 26, 24, 26], fill=(88, 90, 96, 255), width=1)


def m_detonator(d, cx, cy):
    d.rectangle([cx - 1, cy - 3, cx + 1, cy + 2], fill=(178, 140, 60, 255))
    d.line([cx, cy - 3, cx - 2, cy - 5 + 2], fill=RED, width=1)
    d.line([cx, cy - 3, cx + 2, cy - 5 + 2], fill=(52, 52, 58, 255),
           width=1)
    d.point((cx, cy + 3), fill=RED)


def m_spark(d, cx, cy):
    for dx, dy in ((0, -3), (-3, -1), (3, -1), (-2, 3), (2, 3), (0, 3)):
        d.line([cx, cy, cx + dx, cy + dy], fill=(240, 160, 40, 255),
               width=1)
    d.point((cx, cy), fill=RED)


def draw_metal_in_oil(d, rng, band, chunk_col, chunk_pts):
    # glass jar, amber mineral oil, a gray reactive-metal chunk sunk in it
    jar(d, 9, 23, 4, 28, OIL, cap=(70, 74, 80, 255))
    pts = [(x + 16, y + 20) for x, y in chunk_pts]
    d.polygon(pts, fill=chunk_col)
    d.line(pts[:2], fill=shade(chunk_col, 1.3), width=1)
    for _ in range(4):  # bubbles in the oil
        d.point((rng.randint(11, 21), rng.randint(9, 16)),
                fill=(250, 226, 160, 255))
    # label with the band colour and a crossed water-drop warning
    d.rectangle([12, 12, 20, 18], fill=LABEL)
    d.rectangle([12, 12, 20, 18], outline=LABEL_EDGE, width=1)
    d.rectangle([12, 12, 20, 14], fill=band)
    m_drop(d, 16, 16, col=(90, 140, 200, 255))
    d.line([13, 13, 19, 17], fill=RED, width=1)


def dye_bottle(d, x0, x1, body, swatch_cols):
    """Applicator bottle: nozzle cap, tinted body, hair-swatch label."""
    d.rectangle([15, 3, 17, 6], fill=(52, 52, 58, 255))  # nozzle
    d.rectangle([13, 6, 19, 8], fill=(90, 90, 96, 255))  # cap ring
    d.rectangle([x0, 8, x1, 28], fill=body)
    d.rectangle([x0, 8, x1, 28], outline=shade(body, 0.6), width=1)
    d.line([x0 + 2, 9, x0 + 2, 27], fill=shade(body, 1.25), width=1)
    # label with a wavy hair-swatch stripe
    d.rectangle([x0 + 3, 13, x1 - 3, 24], fill=LABEL)
    d.rectangle([x0 + 3, 13, x1 - 3, 24], outline=LABEL_EDGE, width=1)
    n = len(swatch_cols)
    for i, col in enumerate(swatch_cols):
        sx = x0 + 5 + i * ((x1 - x0 - 9) // max(n - 1, 1) if n > 1 else 0)
        pts = [(sx, 15), (sx + 1, 18), (sx, 21), (sx + 1, 22)]
        d.line(pts, fill=col, width=2 if n == 1 else 1)
    d.line([x0 + 4, 26, x1 - 4, 26], fill=shade(body, 0.75), width=1)


def draw_dye_powder(d, rng):
    # paper sachet, torn open, colourful powder piles spilling
    d.polygon([(8, 10), (24, 8), (25, 27), (9, 28)],
              fill=(214, 188, 150, 255))
    d.polygon([(8, 10), (24, 8), (25, 27), (9, 28)],
              outline=(140, 116, 82, 255))
    d.polygon([(8, 10), (14, 6), (19, 9), (24, 8), (20, 11), (12, 12)],
              fill=(190, 162, 118, 255))  # torn top
    piles = [(80, 60, 180, 255), (196, 52, 120, 255), (52, 156, 90, 255),
             (232, 150, 40, 255)]
    for i, col in enumerate(piles):
        cx, cy = 12 + i * 3, 15 + (i % 2)
        d.polygon([(cx - 2, cy + 2), (cx, cy - 1), (cx + 2, cy + 2)],
                  fill=col)
    d.rectangle([11, 20, 22, 25], fill=LABEL)
    d.rectangle([11, 20, 22, 25], outline=LABEL_EDGE, width=1)
    for i, col in enumerate(piles):  # colour dots on the label
        d.point((13 + i * 2, 22), fill=col)
    for _ in range(7):  # spilt colour dust
        d.point((rng.randint(6, 26), rng.randint(27, 30)),
                fill=rng.choice(piles))


def draw_mercury(d, rng):
    # heavy squat flask of silver liquid metal
    d.ellipse([7, 13, 25, 29], fill=(172, 176, 182, 255))
    d.ellipse([7, 13, 25, 29], outline=(96, 100, 106, 255), width=1)
    d.rectangle([13, 6, 19, 14], fill=(188, 192, 198, 255))
    d.rectangle([13, 6, 19, 14], outline=(96, 100, 106, 255), width=1)
    d.rectangle([12, 3, 20, 6], fill=(70, 74, 80, 255))  # stopper
    d.ellipse([9, 20, 23, 28], fill=(206, 210, 216, 255))  # liquid body
    d.line([10, 20, 22, 20], fill=(248, 250, 252, 255),
           width=1)  # bright meniscus
    d.ellipse([11, 22, 15, 25], fill=(238, 240, 244, 255))  # glint pool
    # label with a silver droplet
    d.rectangle([12, 14, 20, 19], fill=LABEL)
    d.rectangle([12, 14, 20, 19], outline=LABEL_EDGE, width=1)
    m_drop(d, 16, 16, col=(150, 154, 160, 255))
    # stray beads of mercury beside the flask
    for bx, by, r in ((27, 26, 1), (5, 28, 1), (28, 29, 0)):
        d.ellipse([bx - r, by - r, bx + r, by + r],
                  fill=(216, 220, 226, 255))


# ------------------------------------------------------------ cluster 2

def mitten_outline(d, cx, top, w, h, fill):
    """One mitten/glove silhouette: fingers merged, thumb nub."""
    x0, x1 = cx - w // 2, cx + w // 2 - 1
    bot = top + h - 1
    d.rectangle([x0, top + 2, x1, bot], fill=fill)
    d.ellipse([x0, top, x1, top + 5], fill=fill)
    return x0, x1, bot


def ring_mesh(d, rng, x0, y0, x1, y1, base, dark, light, junk=False,
              tint_row=False):
    """Chainmail ring texture inside a region: dark gaps, ring dots."""
    d.rectangle([x0, y0, x1, y1], fill=dark)
    row = 0
    for y in range(y0 + 1, y1, 3):
        off = 1 if row % 2 else 0
        for x in range(x0 + 1 + off, x1, 3):
            col = base
            r = rng.random()
            if junk and r < 0.16:
                col = (150, 92, 52, 255)  # rust patch
            elif junk and r < 0.24:
                col = (104, 96, 88, 255)  # corroded ring
            elif r < 0.3:
                col = light
            d.ellipse([x, y, x + 1, y + 1], outline=col, width=1)
        row += 1
    if tint_row:  # tempered steel: one heat-tint row across the mesh
        ym = (y0 + y1) // 2
        for x, col in ((x0 + 1, TINT_STRAW), (x0 + 5, TINT_PURPLE),
                       (x0 + 9, TINT_BLUE)):
            if x < x1:
                d.line([x, ym, min(x + 3, x1), ym], fill=col, width=1)


def draw_chainmail(d, rng, grade, scale, junk=False):
    # glove pair, size by scale: (w, h, centres, top)
    dims = {"xs": (7, 15, (10, 21), 10), "std": (9, 19, (9, 22), 7),
            "xl": (11, 23, (8, 23), 5)}[scale]
    w, h, centres, top = dims
    if junk:
        base, dark, light = ((142, 138, 130, 255), (78, 74, 70, 255),
                             (176, 170, 160, 255))
    else:
        base, dark, light = STEELS[grade]
    for cx in centres:
        x0, x1, bot = mitten_outline(d, cx, top, w, h, dark)
        ring_mesh(d, rng, x0, top + 1, x1, bot - 3, base, dark, light,
                  junk=junk, tint_row=(grade == "qt"))
        # thumb nub on the inner side
        tx = x1 if cx == centres[0] else x0 - 1
        d.ellipse([min(tx, x1), top + h // 2, min(tx, x1) + 1,
                   top + h // 2 + 3], outline=base, width=1)
        # cuff band at the wrist
        d.rectangle([x0, bot - 2, x1, bot], fill=shade(base, 0.8))
        d.line([x0, bot - 2, x1, bot - 2], fill=light, width=1)
        if junk:  # repair plates riveted over worn spots
            d.rectangle([x0 + 1, top + 4, x0 + 3, top + 6],
                        fill=(120, 116, 110, 255))
            d.point((x0 + 1, top + 4), fill=light)
            d.point((x0 + 3, top + 6), fill=light)


def draw_fencing(d, rng, left):
    # a SINGLE suede gauntlet: padded quilted back, cotton cuff
    suede = (238, 232, 218, 255)
    suede_d = (206, 198, 178, 255)
    cotton = (226, 224, 220, 255)
    cx = 16
    x0, x1, bot = mitten_outline(d, cx, 4, 11, 24, suede)
    d.rectangle([x0, 4 + 2, x1, bot], fill=suede)
    d.ellipse([x0, 4, x1, 9], fill=suede)
    # separated fingers hinted by seams on the top third
    for fx in (x0 + 3, x0 + 5, x0 + 7):
        d.line([fx, 5, fx, 11], fill=suede_d, width=1)
    # quilted padding diamonds on the back
    for qy in range(13, 21, 3):
        for qx in range(x0 + 1, x1 - 1, 3):
            d.point((qx + (1 if (qy // 3) % 2 else 0), qy), fill=suede_d)
    d.line([x0 + 1, 12, x1 - 1, 15], fill=suede_d, width=1)
    d.line([x0 + 1, 15, x1 - 1, 12], fill=suede_d, width=1)
    # thumb: right side for a right glove, left side for the _l variant
    if left:
        d.ellipse([x0 - 2, 13, x0, 17], fill=suede)
        d.ellipse([x0 - 2, 13, x0, 17], outline=suede_d, width=1)
    else:
        d.ellipse([x1, 13, x1 + 2, 17], fill=suede)
        d.ellipse([x1, 13, x1 + 2, 17], outline=suede_d, width=1)
    # sturdy cotton wrist cuff with stitch rows
    d.rectangle([x0 - 1, 23, x1 + 1, 28], fill=cotton)
    d.rectangle([x0 - 1, 23, x1 + 1, 28], outline=suede_d, width=1)
    for sx in range(x0, x1, 2):
        d.point((sx, 25), fill=suede_d)
        d.point((sx, 27), fill=suede_d)


def draw_eod(d, rng):
    # bulky overhand protectors: thick kevlar pads strapped over hands
    kev = (168, 170, 150, 255)
    kev_d = (116, 118, 100, 255)
    strap = (80, 78, 66, 255)
    for cx in (10, 22):
        x0, x1 = cx - 6, cx + 5
        # underlying glove, mostly hidden
        mitten_outline(d, cx, 10, 10, 18, kev_d)
        # the thick pad over the back of the hand: layered slabs
        d.rectangle([x0, 6, x1, 17], fill=kev)
        d.rectangle([x0, 6, x1, 17], outline=kev_d, width=1)
        for ly in (9, 12, 15):  # layered kevlar seams
            d.line([x0 + 1, ly, x1 - 1, ly], fill=kev_d, width=1)
        d.rectangle([x0 + 2, 7, x1 - 2, 8], fill=shade(kev, 1.12))
        # knuckle ridge
        for kx in range(x0 + 2, x1 - 1, 3):
            d.point((kx, 10), fill=shade(kev, 0.85))
        # straps across the pad and around the wrist
        d.rectangle([x0, 13, x1, 14], fill=strap)
        d.rectangle([cx - 2, 13, cx, 14], fill=(190, 190, 196, 255))
        d.rectangle([x0 + 1, 24, x1 - 1, 26], fill=strap)
        d.line([x0 + 1, 20, x1 - 1, 20], fill=kev_d, width=1)


# -------------------------------------------------------------------- main

CHEM = {
    "chem_baking_soda", "chem_calcium_chloride", "chem_lead_azide",
    "chem_mercury_fulminate", "chem_potassium", "chem_potassium_alum",
    "chem_sodium",
}


def parse_chainmail(sid):
    s = sid
    scale = "std"
    if s.startswith("xl_"):
        scale, s = "xl", s[3:]
    elif s.startswith("xs_"):
        scale, s = "xs", s[3:]
    grade = s.split("_")[0]
    return grade, scale


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "chem_baking_soda":
        draw_box_soda(d, rng)
    elif sid == "chem_calcium_chloride":
        draw_powder_jar(d, rng, m_flake)
    elif sid == "chem_potassium_alum":
        draw_powder_jar(d, rng, m_crystal)
        # visible crystals heaped above the powder line
        m_crystal(d, 13, 9)
        m_crystal(d, 19, 10)
    elif sid == "chem_lead_azide":
        draw_explosive_tin(d, rng, WHITE_POWDER, m_detonator)
    elif sid == "chem_mercury_fulminate":
        draw_explosive_tin(d, rng, GRAY_POWDER, m_spark)
    elif sid == "chem_potassium":
        draw_metal_in_oil(d, rng, (150, 60, 160, 255), (158, 158, 164, 255),
                          [(-4, 2), (-1, -2), (3, -1), (4, 2)])
    elif sid == "chem_sodium":
        draw_metal_in_oil(d, rng, (52, 110, 180, 255), (182, 184, 190, 255),
                          [(-3, 2), (-3, -1), (1, -2), (4, 1)])
    elif sid == "abstract_hair_dye":
        rainbow = [(198, 44, 36, 255), (232, 150, 40, 255),
                   (52, 156, 72, 255), (48, 84, 196, 255)]
        dye_bottle(d, 10, 22, (120, 116, 128, 255), rainbow)
    elif sid == "dye_powder":
        draw_dye_powder(d, rng)
    elif sid.startswith("hair_dye_"):
        col = DYE_COLOURS[sid[len("hair_dye_"):]]
        body = shade(col, 0.55) if col != DYE_COLOURS["white"] else \
            (196, 194, 190, 255)
        dye_bottle(d, 10, 22, body, [col])
    elif sid == "mercury":
        draw_mercury(d, rng)
    elif sid == "chainmail_junk_hands":
        draw_chainmail(d, rng, None, "std", junk=True)
    elif sid.endswith("_chainmail_hands"):
        grade, scale = parse_chainmail(sid)
        draw_chainmail(d, rng, grade, scale)
    elif sid == "gauntlet_fencing":
        draw_fencing(d, rng, left=False)
    elif sid == "gauntlet_fencing_l":
        draw_fencing(d, rng, left=True)
    elif sid == "gloves_eod":
        draw_eod(d, rng)
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
    # contact-sheet preview: 6 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item63.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item63.png {prev.size}")


if __name__ == "__main__":
    main()
