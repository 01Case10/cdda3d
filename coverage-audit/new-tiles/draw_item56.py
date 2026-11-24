#!/usr/bin/env python3
"""ITEM56: ballistic armor + misc items -- CDDA Voxel coverage audit.

Two clusters:

  Ballistic armor (ballistic_armor.json) -- curved armor plates,
    plate-carrier vests, and EOD blast jackets:
      ESAPI torso plates and hexagonal ESBI side plates are drawn
      per steel grade, in the same colour language as ITEM47:
        lc mild = light matte gray, mc medium = mid gray,
        hc high = bright polished with a glint, ch hardened =
        dark blue-gray, qt tempered = heat-tint band across the
        steel; scrap = patchwork salvaged steel wrapped in rags.
      ESAPI = shooter's-cut torso plate with fabric edge binding;
      ESBI = smaller hexagonal plate in a padded fabric border.
      deltoid plates = a pair of off-white UHMWPE plates with a
      printed glyph band ("STRIKE FACE"); eod_plate = a polygonal
      layered fragmentation plate (steel core, Kevlar/foam rim);
      heavy_steel / steel_ballistic = tempered NIJ plates told
      apart by bulk and their etched glyph band; lead XEDRA plate
      = matte lead panel on a steel backing; stab_plate = a soft
      stitched Kevlar panel; IOTV add-ons = nylon soft armor
      (groin flap, collar, shoulder caps).  The three light vests
      are plate carriers with MOLLE rows, told apart by their
      pouches: mag pouches along the hem, a shoulder pouch, or one
      big storage pouch.  EOD jackets are bulky quilted blast
      suits; the light one is slimmer and darker.

  Misc (misc.json) -- each item drawn as itself from its
    description: C-4 and tear-gas payloads are debug dummies
    (checker-tagged), the urns share one amphora form in clay /
    crystal / porcelain, whips and crops by length and tip, horns
    by form (truck trumpet vs car disc), and so on.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "ballistic_vest_light",
    "ballistic_vest_light_operator",
    "ballistic_vest_light_pouches",
    "ch_esapi_plate",
    "ch_esbi_plate",
    "deltoid_plates",
    "eod_plate",
    "hc_esapi_plate",
    "hc_esbi_plate",
    "heavy_steel_ballistic_plate",
    "iotv_groin_plate",
    "iotv_neck_plate",
    "iotv_shoulder_plate",
    "jacket_eod",
    "jacket_eod_light",
    "lc_esapi_plate",
    "lc_esbi_plate",
    "lead_plate_xedra_mark",
    "mc_esapi_plate",
    "mc_esbi_plate",
    "qt_esapi_plate",
    "qt_esbi_plate",
    "scrap_esapi_plate",
    "scrap_esbi_plate",
    "stab_plate",
    "steel_ballistic_plate",
    "c4_payload",
    "clay_urn",
    "crystal_urn",
    "dressage_whip",
    "gag_ball",
    "glass_cutter",
    "handcuffs_leather",
    "horn_big",
    "horn_car",
    "nm_teeth",
    "paddle_plastic",
    "porcelain_urn",
    "riding_crop",
    "safe_box",
    "sarcophagus_access_code",
    "sextoy",
    "tape_measure",
    "tear_gas_payload",
    "tear_gas_payload_act",
    "teargas_sprayer",
    "testflames",
    "tindalos_whistle",
    "toilet_travel",
    "toilet_travel_folded",
    "vortex_stone",
    "whistle_multitool",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
NYLON = (86, 90, 66, 255)        # olive carrier fabric
NYLON_DARK = (58, 62, 46, 255)
NYLON_LIGHT = (116, 120, 90, 255)
BIND = (52, 52, 56, 255)         # dark fabric edge binding
KEVLAR = (168, 148, 92, 255)
FOAM = (210, 204, 188, 255)
RAG = (150, 128, 96, 255)

# steel grades, ITEM47 colour language: grade -> (body, dark, light)
STEEL = {
    "lc": ((168, 172, 178, 255), (128, 132, 140, 255), (198, 202, 208, 255)),
    "mc": ((140, 146, 154, 255), (102, 108, 118, 255), (172, 178, 186, 255)),
    "hc": ((192, 198, 206, 255), (146, 152, 162, 255), (232, 236, 242, 255)),
    "ch": ((104, 112, 128, 255), (72, 80, 96, 255), (142, 152, 170, 255)),
    "qt": ((158, 148, 138, 255), (116, 106, 96, 255), (192, 182, 168, 255)),
    "scrap": ((130, 124, 118, 255), (94, 88, 82, 255), (160, 154, 146, 255)),
}
QT_TINTS = [(214, 178, 92, 255), (150, 110, 170, 255), (92, 120, 190, 255)]


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ plates

def grade_of(sid):
    for g in ("lc", "mc", "hc", "ch", "qt", "scrap"):
        if sid.startswith(g + "_"):
            return g
    raise KeyError(sid)


def plate_finish(d, rng, grade, x0, x1, ymid):
    """Per-grade surface marks inside an already-filled plate."""
    if grade == "hc":  # polished glint
        d.line([x0 + 3, ymid - 5, x0 + 6, ymid + 4],
               fill=(246, 248, 250, 255), width=1)
        d.point((x0 + 4, ymid - 6), fill=(246, 248, 250, 255))
    elif grade == "qt":  # heat-tint band across the steel
        for i, col in enumerate(QT_TINTS):
            d.line([x0 + 2, ymid - 1 + i, x1 - 2, ymid - 1 + i],
                   fill=col, width=1)
    elif grade == "scrap":  # patchwork panels + seams
        body, dark, _ = STEEL["scrap"]
        d.rectangle([x0 + 2, ymid - 6, (x0 + x1) // 2, ymid - 1],
                    fill=shade(body, 1.14))
        d.rectangle([(x0 + x1) // 2 + 1, ymid + 1, x1 - 2, ymid + 6],
                    fill=shade(body, 0.88))
        d.line([x0 + 2, ymid, x1 - 2, ymid], fill=dark, width=1)
        for px, py in ((x0 + 3, ymid - 5), (x1 - 3, ymid + 5),
                       ((x0 + x1) // 2, ymid - 3)):
            d.point((px, py), fill=dark)  # crude rivets
    elif grade == "ch":  # hard edge highlight
        d.line([x0 + 2, ymid - 6, x0 + 2, ymid + 6],
               fill=STEEL["ch"][2], width=1)
    else:  # lc / mc: one soft sheen line
        d.line([x0 + 3, ymid - 5, x0 + 3, ymid + 5],
               fill=STEEL[grade][2], width=1)


def draw_esapi(d, rng, grade):
    body, dark, light = STEEL[grade]
    # shooter's-cut torso plate: top corners clipped, curved shading
    poly = [(9, 6), (23, 6), (27, 10), (27, 24), (24, 27), (8, 27),
            (5, 24), (5, 10)]
    d.polygon(poly, fill=BIND)  # fabric edge binding
    inner = [(10, 8), (22, 8), (25, 11), (25, 23), (22, 25), (10, 25),
             (7, 23), (7, 11)]
    d.polygon(inner, fill=body)
    d.line([16, 8, 16, 25], fill=shade(body, 1.10), width=3)  # curve crown
    d.line([8, 11, 8, 22], fill=dark, width=1)
    d.line([24, 11, 24, 22], fill=dark, width=1)
    plate_finish(d, rng, grade, 7, 25, 16)
    if grade == "scrap":  # rags wrapped over the sharp edges
        for rx, ry in ((5, 14), (26, 18), (12, 26)):
            d.rectangle([rx - 1, ry - 2, rx + 1, ry + 2], fill=RAG)
            d.line([rx - 1, ry, rx + 1, ry], fill=shade(RAG, 0.7), width=1)


def draw_esbi(d, rng, grade):
    body, dark, light = STEEL[grade]
    cx, cy = 16, 16
    hexg = [(cx - 6, 7), (cx + 6, 7), (cx + 10, 16), (cx + 6, 25),
            (cx - 6, 25), (cx - 10, 16)]
    d.polygon(hexg, fill=NYLON_DARK)  # padded synthetic-fabric border
    inner = [(cx - 4, 10), (cx + 4, 10), (cx + 7, 16), (cx + 4, 22),
             (cx - 4, 22), (cx - 7, 16)]
    d.polygon(inner, fill=body)
    d.line([cx, 10, cx, 22], fill=shade(body, 1.10), width=2)
    plate_finish(d, rng, grade, cx - 6, cx + 6, cy)
    if grade == "scrap":
        for rx, ry in ((cx - 9, 12), (cx + 9, 20)):
            d.rectangle([rx - 1, ry - 2, rx + 1, ry + 2], fill=RAG)


def draw_deltoid(d, rng):
    # a pair of off-white UHMWPE deltoid plates, "STRIKE FACE" glyphs
    for x0 in (4, 18):
        d.polygon([(x0 + 2, 8), (x0 + 11, 8), (x0 + 12, 12), (x0 + 11, 22),
                   (x0 + 7, 25), (x0 + 3, 22), (x0 + 1, 12)],
                  fill=(226, 224, 216, 255))
        d.polygon([(x0 + 2, 8), (x0 + 11, 8), (x0 + 12, 12), (x0 + 11, 22),
                   (x0 + 7, 25), (x0 + 3, 22), (x0 + 1, 12)], outline=INK)
        d.line([x0 + 6, 10, x0 + 6, 22], fill=(196, 194, 186, 255),
               width=2)
        for gy in (14, 17):  # printed text glyph bars
            d.line([x0 + 3, gy, x0 + 9, gy], fill=(120, 120, 124, 255),
                   width=1)


def draw_eod_plate(d, rng):
    # polygonal fragmentation plate: layered rim, tempered core
    poly = [(11, 5), (21, 5), (27, 11), (27, 21), (21, 27), (11, 27),
            (5, 21), (5, 11)]
    d.polygon(poly, fill=KEVLAR)                 # Kevlar layer
    mid = [(11, 8), (21, 8), (24, 12), (24, 20), (21, 24), (11, 24),
           (8, 20), (8, 12)]
    d.polygon(mid, fill=FOAM)                    # foam layer
    core = [(12, 11), (20, 11), (22, 14), (22, 19), (20, 22), (12, 22),
            (10, 19), (10, 14)]
    body, dark, _ = STEEL["qt"]
    d.polygon(core, fill=body)
    for i, col in enumerate(QT_TINTS):
        d.line([13, 15 + i, 19, 15 + i], fill=col, width=1)
    for px, py in ((11, 8), (21, 8), (11, 24), (21, 24)):
        d.point((px, py), fill=INK)


def draw_nij(d, rng, heavy):
    body, dark, light = STEEL["qt"]
    if heavy:  # bulkier plate, thick double edge
        poly = [(7, 5), (25, 5), (28, 9), (28, 25), (25, 28), (7, 28),
                (4, 25), (4, 9)]
        d.polygon(poly, fill=BIND)
        d.polygon([(6, 7), (26, 7), (26, 26), (6, 26)], fill=dark)
        inner = [(8, 8), (24, 8), (25, 11), (25, 24), (22, 26), (10, 26),
                 (7, 24), (7, 11)]
        d.polygon(inner, fill=body)
        etch_y = 21
    else:
        poly = [(9, 6), (23, 6), (27, 10), (27, 24), (24, 27), (8, 27),
                (5, 24), (5, 10)]
        d.polygon(poly, fill=BIND)
        inner = [(10, 8), (22, 8), (25, 11), (25, 23), (22, 25),
                 (10, 25), (7, 23), (7, 11)]
        d.polygon(inner, fill=body)
        etch_y = 20
    d.line([16, 9, 16, 24], fill=shade(body, 1.10), width=3)
    for i, col in enumerate(QT_TINTS):  # temper tint
        d.line([9, 12 + i, 23, 12 + i], fill=col, width=1)
    # etched rating glyphs on the back band
    n = 4 if heavy else 5
    for i in range(n):
        x = 10 + i * 3
        d.line([x, etch_y, x + 1, etch_y], fill=dark, width=1)
        d.point((x, etch_y + 2), fill=dark)


def draw_lead(d, rng):
    # steel backing offset behind a matte lead front panel
    d.rectangle([8, 7, 27, 26], fill=STEEL["mc"][0])
    d.rectangle([8, 7, 27, 26], outline=STEEL["mc"][1], width=1)
    d.rectangle([5, 5, 24, 24], fill=(102, 104, 112, 255))
    d.rectangle([5, 5, 24, 24], outline=(70, 72, 80, 255), width=1)
    d.line([7, 7, 22, 7], fill=(126, 128, 136, 255), width=1)  # soft sheen
    # XEDRA mark: stamped glyph stamp, bottom right of the lead face
    d.rectangle([17, 17, 22, 22], outline=(70, 72, 80, 255), width=1)
    d.line([18, 21, 21, 18], fill=(70, 72, 80, 255), width=1)
    d.line([18, 18, 19, 19], fill=(70, 72, 80, 255), width=1)


def draw_stab(d, rng):
    # soft Kevlar stab panel: stitched fabric insert, no metal
    d.rectangle([7, 6, 25, 26], fill=KEVLAR)
    d.rectangle([7, 6, 25, 26], outline=shade(KEVLAR, 0.62), width=1)
    for gy in range(9, 25, 4):  # stitch grid
        d.line([8, gy, 24, gy], fill=shade(KEVLAR, 0.78), width=1)
    for gx in range(11, 24, 5):
        d.line([gx, 7, gx, 25], fill=shade(KEVLAR, 0.86), width=1)
    d.line([8, 7, 24, 7], fill=shade(KEVLAR, 1.18), width=1)


# -------------------------------------------------------------------- IOTV

def draw_iotv_groin(d, rng):
    # nylon groin protector: waist band + hanging front/back flaps
    d.rectangle([7, 5, 25, 9], fill=NYLON_DARK)
    d.rectangle([9, 9, 23, 24], fill=NYLON)
    d.rectangle([9, 9, 23, 24], outline=NYLON_DARK, width=1)
    d.polygon([(9, 24), (23, 24), (20, 27), (12, 27)], fill=NYLON)
    for gy in (13, 18):  # soft-insert stitch lines
        d.line([11, gy, 21, gy], fill=NYLON_LIGHT, width=1)
    d.line([16, 10, 16, 24], fill=NYLON_DARK, width=1)


def draw_iotv_neck(d, rng):
    # ballistic collar: a curved U of nylon with soft inserts
    d.arc([5, 8, 27, 30], start=300, end=240, fill=NYLON, width=7)
    d.arc([5, 8, 27, 30], start=300, end=240, fill=NYLON_DARK, width=1)
    d.arc([9, 12, 23, 26], start=300, end=240, fill=NYLON_LIGHT, width=1)
    for a in (320, 350, 20, 50):  # segment stitches around the curve
        x = 16 + 9 * math.cos(math.radians(a))
        y = 19 + 9 * math.sin(math.radians(a))
        d.point((int(x), int(y)), fill=NYLON_DARK)


def draw_iotv_shoulder(d, rng):
    # a pair of nylon shoulder caps with soft inserts
    for x0 in (4, 18):
        d.arc([x0, 10, x0 + 11, 24], start=180, end=360, fill=NYLON,
              width=6)
        d.line([x0, 17, x0 + 11, 17], fill=NYLON, width=4)
        d.line([x0 + 2, 14, x0 + 9, 14], fill=NYLON_LIGHT, width=1)
        d.rectangle([x0 + 3, 18, x0 + 8, 22], fill=NYLON_DARK)  # strap


# -------------------------------------------------------------------- vests

def vest_base(d, rng):
    # plate carrier silhouette: shoulder straps, arm cutouts, MOLLE
    d.polygon([(8, 4), (13, 4), (14, 7), (19, 7), (20, 4), (24, 4),
               (26, 10), (25, 27), (7, 27), (6, 10)], fill=NYLON)
    d.polygon([(8, 4), (13, 4), (14, 7), (19, 7), (20, 4), (24, 4),
               (26, 10), (25, 27), (7, 27), (6, 10)], outline=NYLON_DARK)
    d.rectangle([10, 9, 22, 20], outline=NYLON_DARK, width=1)  # plate pocket
    d.line([11, 11, 21, 11], fill=NYLON_LIGHT, width=1)
    for gy in (22, 24):  # MOLLE webbing rows
        for gx in range(8, 24, 4):
            d.line([gx, gy, gx + 2, gy], fill=NYLON_DARK, width=1)


def draw_vest(d, rng, variant):
    vest_base(d, rng)
    if variant == "mag":  # three mag pouches along the hem (base vest)
        for x0 in (8, 14, 20):
            d.rectangle([x0, 21, x0 + 4, 27], fill=NYLON_DARK)
            d.line([x0, 23, x0 + 4, 23], fill=NYLON_LIGHT, width=1)
    elif variant == "operator":  # shoulder-mounted pouch + strap gear
        d.rectangle([19, 3, 24, 8], fill=NYLON_DARK)
        d.line([20, 5, 23, 5], fill=NYLON_LIGHT, width=1)
        d.rectangle([8, 21, 13, 26], fill=NYLON_DARK)  # one low pouch
        d.point((16, 13), fill=(200, 60, 50, 255))   # marker on pocket
    elif variant == "pouches":  # one big central storage pouch, zipper
        d.rectangle([11, 19, 22, 27], fill=NYLON_DARK)
        d.line([12, 21, 21, 21], fill=NYLON_LIGHT, width=1)
        for gx in range(13, 21, 2):  # zipper teeth
            d.point((gx, 22), fill=NYLON_LIGHT)


def draw_eod_jacket(d, rng, light):
    if light:  # slimmer tactical cut, darker
        body, dark = (104, 108, 84, 255), (74, 78, 58, 255)
        poly = [(10, 6), (22, 6), (25, 11), (24, 27), (8, 27), (7, 11)]
        bands = (12, 17, 22)
    else:      # bulky blast suit: wide body, high collar
        body, dark = KEVLAR, shade(KEVLAR, 0.66)
        poly = [(8, 7), (24, 7), (28, 12), (27, 28), (5, 28), (4, 12)]
        bands = (13, 18, 23)
        d.rectangle([12, 3, 20, 7], fill=dark)  # high protective collar
        d.line([13, 4, 19, 4], fill=shade(body, 1.15), width=1)
    d.polygon(poly, fill=body)
    d.polygon(poly, outline=dark)
    for gy in bands:  # quilted padding bands
        d.line([poly[3][0] - 1, gy, poly[2][0] + 1, gy], fill=dark, width=1)
        d.line([poly[3][0] - 1, gy - 1, poly[2][0] + 1, gy - 1],
               fill=shade(body, 1.14), width=1)
    cx0, cx1 = (11, 21) if not light else (12, 20)
    d.rectangle([cx0, 9, cx1, 16], outline=dark, width=1)  # plate pocket
    d.line([cx0 + 1, 11, cx1 - 1, 11], fill=shade(body, 1.14), width=1)


# -------------------------------------------------------------------- misc

def debug_tag(d):
    # checker tag marking a debug/dummy item
    for i in range(3):
        for j in range(3):
            col = (240, 240, 240, 255) if (i + j) % 2 == 0 else INK
            d.rectangle([26 + i, 3 + j, 26 + i, 3 + j], fill=col)


def draw_c4(d, rng):
    d.rectangle([7, 11, 24, 22], fill=(176, 164, 128, 255))  # C-4 block
    d.rectangle([7, 11, 24, 22], outline=(110, 100, 74, 255), width=1)
    for i in range(3):  # stamped glyph bars
        d.line([10 + i * 4, 15, 12 + i * 4, 15], fill=(110, 100, 74, 255),
               width=2)
    d.line([16, 11, 16, 7], fill=(196, 48, 40, 255), width=1)  # wires
    d.line([16, 7, 21, 7], fill=(30, 30, 34, 255), width=1)
    d.rectangle([20, 5, 24, 9], fill=(70, 70, 76, 255))  # detonator
    d.point((22, 6), fill=(196, 48, 40, 255))
    debug_tag(d)


def urn_shape(d, body, dark, light, deco=None):
    d.ellipse([10, 4, 22, 7], fill=dark)                    # rim
    d.rectangle([12, 6, 20, 9], fill=body)                  # neck
    d.polygon([(12, 9), (20, 9), (25, 14), (25, 20), (21, 26), (11, 26),
               (7, 20), (7, 14)], fill=body)                # belly
    d.rectangle([11, 26, 21, 28], fill=dark)                # foot
    d.line([10, 12, 10, 22], fill=light, width=2)           # sheen
    d.line([8, 17, 24, 17], fill=dark, width=1)             # band


def draw_urn(d, rng, kind):
    if kind == "clay":
        urn_shape(d, (156, 102, 62, 255), (110, 68, 36, 255),
                  (186, 132, 88, 255))
        for px in (13, 18, 21):  # rough fired speckle
            d.point((px, 20), fill=(110, 68, 36, 255))
    elif kind == "crystal":
        urn_shape(d, (226, 140, 200, 255), (168, 84, 148, 255),
                  (246, 200, 228, 255))
        for px, py in ((13, 14), (19, 15), (16, 21), (21, 20)):
            d.point((px, py), fill=(250, 230, 120, 255))  # floral glints
            d.point((px + 1, py), fill=(168, 84, 148, 255))
        d.point((22, 10), fill=(255, 255, 255, 255))      # sparkle
        d.point((9, 24), fill=(255, 255, 255, 255))
    else:  # porcelain: glossy white, faint blue floral
        urn_shape(d, (244, 244, 248, 255), (188, 192, 204, 255),
                  (255, 255, 255, 255))
        for px, py in ((14, 15), (19, 19)):
            d.point((px, py), fill=(120, 150, 210, 255))
            d.point((px + 1, py + 1), fill=(120, 150, 210, 255))
        d.line([12, 11, 12, 23], fill=(214, 224, 240, 255), width=2)


def draw_whip(d, rng, crop):
    # diagonal tapered rod; crop is shorter with a leather popper
    if crop:
        d.line([6, 26, 20, 12], fill=(60, 44, 30, 255), width=3)  # handle
        d.line([20, 12, 25, 7], fill=(40, 40, 44, 255), width=1)   # shaft
        d.polygon([(25, 7), (29, 4), (30, 7), (27, 10)],
                  fill=(122, 84, 50, 255))                         # popper
        d.line([7, 25, 10, 28], fill=(122, 84, 50, 255), width=1)  # loop
    else:
        d.line([4, 27, 9, 22], fill=(60, 44, 30, 255), width=3)    # grip
        pts = [(9 + i, 22 - i) for i in range(17)]
        d.line(pts, fill=(40, 40, 44, 255), width=2)
        d.line([(18, 13), (27, 4)], fill=(40, 40, 44, 255), width=1)
        d.point((28, 3), fill=(40, 40, 44, 255))                   # point
        d.ellipse([3, 25, 7, 29], outline=(122, 84, 50, 255), width=1)


def draw_gag(d, rng):
    d.line([3, 15, 29, 15], fill=(110, 66, 38, 255), width=3)  # strap
    d.ellipse([10, 9, 22, 21], fill=(206, 44, 40, 255))        # ball
    d.ellipse([10, 9, 22, 21], outline=(140, 26, 24, 255), width=1)
    d.point((13, 12), fill=(240, 130, 120, 255))
    d.point((14, 11), fill=(246, 180, 170, 255))
    d.rectangle([25, 12, 28, 18], outline=(170, 172, 178, 255),
                width=1)                                       # buckle


def draw_glass_cutter(d, rng):
    d.line([7, 25, 17, 15], fill=(178, 52, 48, 255), width=4)  # handle
    d.line([8, 24, 16, 16], fill=(216, 110, 104, 255), width=1)
    d.rectangle([16, 10, 21, 15], fill=(150, 154, 162, 255))    # steel head
    d.ellipse([19, 12, 24, 17], outline=(90, 94, 102, 255), width=2)
    d.point((21, 14), fill=(52, 52, 58, 255))                   # wheel
    d.point((18, 11), fill=(232, 234, 238, 255))


def draw_cuffs(d, rng):
    for x0 in (3, 19):  # two wide leather cuffs
        d.rectangle([x0, 11, x0 + 9, 21], fill=(168, 52, 44, 255))
        d.rectangle([x0, 11, x0 + 9, 21], outline=(104, 30, 26, 255),
                    width=1)
        d.line([x0 + 1, 14, x0 + 8, 14], fill=(128, 40, 34, 255), width=1)
        d.rectangle([x0 + 3, 17, x0 + 6, 20], outline=(190, 192, 198, 255),
                    width=1)                                   # buckle
    for i in range(3):  # chain links between them
        d.ellipse([12 + i * 3, 15, 14 + i * 3, 18],
                  outline=(170, 172, 178, 255), width=1)


def draw_horn_big(d, rng):
    # long truck air horn: twin trumpets with a big bell flare
    d.polygon([(4, 13), (20, 14), (27, 10), (27, 19), (20, 16), (4, 16)],
              fill=(178, 182, 188, 255))
    d.polygon([(4, 13), (20, 14), (27, 10), (27, 19), (20, 16), (4, 16)],
              outline=(110, 114, 122, 255))
    d.ellipse([25, 10, 28, 19], fill=(120, 124, 132, 255))     # bell mouth
    d.line([5, 19, 18, 18], fill=(140, 144, 152, 255), width=2)  # 2nd tube
    d.rectangle([3, 12, 6, 20], fill=(90, 94, 102, 255))       # valve body
    d.point((5, 14), fill=(52, 52, 58, 255))


def draw_horn_car(d, rng):
    # compact round disc horn with grille slits
    d.ellipse([8, 8, 24, 24], fill=(168, 172, 178, 255))
    d.ellipse([8, 8, 24, 24], outline=(104, 108, 116, 255), width=1)
    d.ellipse([11, 11, 21, 21], outline=(128, 132, 140, 255), width=1)
    for gy in (14, 17, 20):
        d.line([12, gy, 20, gy], fill=(104, 108, 116, 255), width=1)
    d.ellipse([14, 14, 18, 18], fill=(90, 94, 102, 255))       # centre bolt
    d.rectangle([14, 24, 18, 27], fill=(70, 70, 76, 255))      # bracket


def draw_teeth(d, rng):
    # a fan of impossibly sharp monster fangs, bone with a cyan tint
    fangs = [(6, 26, 9, 8), (11, 26, 13, 5), (16, 26, 18, 7),
             (21, 26, 22, 11), (25, 26, 26, 15)]
    for x0, yb, x1, yt in fangs:
        d.polygon([(x0, yb), ((x0 + x1) // 2, yt), (x1, yb)],
                  fill=(216, 238, 240, 255))
        d.polygon([(x0, yb), ((x0 + x1) // 2, yt), (x1, yb)],
                  outline=(150, 180, 184, 255))
        d.line([x0 + 1, yb - 3, x1 - 1, yb - 3],
               fill=(240, 250, 250, 255), width=1)
    d.line([4, 27, 28, 27], fill=(150, 180, 184, 255), width=2)  # gumline


def draw_paddle(d, rng):
    d.rectangle([12, 19, 17, 28], fill=(52, 52, 58, 255))      # handle
    d.rectangle([8, 4, 21, 20], fill=(70, 70, 78, 255))        # blade
    d.rectangle([8, 4, 21, 20], outline=(36, 36, 42, 255), width=1)
    for px in (12, 16):  # holes in the blade
        d.point((px, 9), fill=(36, 36, 42, 255))
        d.point((px, 14), fill=(36, 36, 42, 255))
    d.line([10, 6, 19, 6], fill=(96, 96, 104, 255), width=1)


def draw_safe(d, rng):
    d.rectangle([5, 7, 27, 25], fill=(168, 172, 178, 255))
    d.rectangle([5, 7, 27, 25], outline=(96, 100, 108, 255), width=2)
    d.ellipse([12, 11, 20, 19], outline=(80, 84, 92, 255), width=2)
    d.point((16, 15), fill=(52, 52, 58, 255))                  # dial
    for a in range(0, 360, 45):
        x = 16 + 4 * math.cos(math.radians(a))
        y = 15 + 4 * math.sin(math.radians(a))
        d.point((int(x), int(y)), fill=(80, 84, 92, 255))
    d.rectangle([22, 13, 25, 17], fill=(90, 94, 102, 255))     # handle
    for px, py in ((7, 9), (25, 9), (7, 23), (25, 23)):
        d.point((px, py), fill=(80, 84, 92, 255))              # rivets


def draw_code_sheet(d, rng):
    d.rectangle([8, 4, 24, 28], fill=(240, 236, 226, 255))
    d.rectangle([8, 4, 24, 28], outline=INK, width=1)
    d.line([10, 7, 22, 7], fill=(148, 148, 152, 255), width=1)
    # the passcode: one row of big digit glyph blocks
    for i in range(5):
        x = 10 + i * 3
        bits = (0x5A + i * 37) & 0x7F
        for r in range(3):
            for c in range(2):
                if bits & (1 << (r * 2 + c)):
                    d.point((x + c, 11 + r), fill=INK)
        d.line([x, 15, x + 1, 15], fill=INK, width=1)
    for y in (19, 22, 25):  # smaller print below
        d.line([10, y, 10 + rng.randint(7, 11), y],
               fill=(148, 148, 152, 255), width=1)


def draw_sextoy(d, rng):
    # an abstract smooth pink wand: rounded capsule, button, base ring
    d.line([13, 26, 19, 10], fill=(232, 130, 170, 255), width=7)
    d.ellipse([15, 4, 22, 12], fill=(240, 156, 190, 255))
    d.line([14, 24, 17, 13], fill=(246, 190, 214, 255), width=2)
    d.rectangle([11, 25, 17, 28], fill=(196, 90, 140, 255))    # base
    d.point((17, 19), fill=(255, 255, 255, 255))               # button


def draw_tape(d, rng):
    d.rectangle([5, 9, 19, 24], fill=(240, 200, 48, 255))      # yellow case
    d.rectangle([5, 9, 19, 24], outline=(170, 140, 26, 255), width=1)
    d.rectangle([5, 18, 19, 24], fill=(52, 52, 58, 255))       # rubber grip
    d.ellipse([8, 12, 14, 18], outline=(170, 140, 26, 255), width=1)
    d.point((11, 15), fill=(52, 52, 58, 255))
    d.rectangle([19, 20, 29, 23], fill=(216, 218, 222, 255))   # blade out
    for gx in range(21, 29, 2):  # measure ticks
        d.point((gx, 21), fill=(90, 94, 102, 255))
    d.rectangle([27, 19, 29, 24], fill=(150, 154, 162, 255))   # blade hook


def gas_can(d, rng):
    d.rectangle([11, 12, 20, 24], fill=(96, 100, 106, 255))    # canister
    d.rectangle([11, 12, 20, 24], outline=(58, 60, 66, 255), width=1)
    d.rectangle([13, 8, 18, 12], fill=(70, 70, 76, 255))       # valve top
    d.rectangle([12, 17, 19, 20], fill=(214, 178, 60, 255))    # hazard band
    d.point((15, 18), fill=INK)


def draw_gas_payload(d, rng, active):
    gas_can(d, rng)
    if active:  # spraying a pale gas cloud, countdown ticks
        for cx, cy, r in ((24, 10, 3), (27, 7, 2), (22, 6, 2)):
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      fill=(196, 214, 188, 255))
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      outline=(150, 168, 142, 255), width=1)
        for i in range(3):
            d.point((24 + i * 2, 16 + i), fill=(196, 48, 40, 255))
    debug_tag(d)


def draw_sprayer(d, rng):
    d.rectangle([12, 11, 21, 27], fill=(206, 52, 44, 255))     # red canister
    d.rectangle([12, 11, 21, 27], outline=(140, 30, 26, 255), width=1)
    d.rectangle([13, 17, 20, 23], fill=(240, 236, 226, 255))   # label
    d.line([14, 19, 19, 19], fill=(148, 148, 152, 255), width=1)
    d.line([14, 21, 18, 21], fill=(148, 148, 152, 255), width=1)
    d.rectangle([14, 6, 19, 11], fill=(52, 52, 58, 255))       # spray head
    d.rectangle([17, 7, 22, 9], fill=(52, 52, 58, 255))        # nozzle arm
    d.point((13, 13), fill=(240, 130, 120, 255))


def draw_testflames(d, rng):
    d.polygon([(10, 22), (22, 22), (24, 27), (8, 27)],
              fill=(130, 134, 142, 255))                       # steel chunk
    d.line([11, 24, 21, 24], fill=(90, 94, 102, 255), width=1)
    flames = [((16, 21), (11, 12), (14, 6), (17, 11), (20, 5),
               (21, 13), (23, 10), (22, 21)),
              ]
    d.polygon([(11, 21), (13, 10), (16, 4), (18, 12), (21, 7),
               (22, 21)], fill=(232, 110, 30, 255))
    d.polygon([(13, 21), (15, 13), (17, 9), (19, 14), (20, 21)],
              fill=(246, 178, 52, 255))
    d.polygon([(15, 21), (16, 16), (18, 21)], fill=(250, 226, 130, 255))
    d.point((24, 14), fill=(232, 110, 30, 255))                # embers
    d.point((8, 16), fill=(246, 178, 52, 255))


def whistle_body(d, x0, y0, body, dark):
    # classic side-profile whistle: mouthpiece + round chamber
    d.rectangle([x0, y0 + 3, x0 + 8, y0 + 6], fill=body)        # mouthpiece
    d.ellipse([x0 + 7, y0, x0 + 19, y0 + 12], fill=body)        # chamber
    d.ellipse([x0 + 7, y0, x0 + 19, y0 + 12], outline=dark, width=1)
    d.rectangle([x0 + 10, y0 + 2, x0 + 15, y0 + 6], fill=dark)  # window
    d.line([x0 + 1, y0 + 4, x0 + 6, y0 + 4], fill=dark, width=1)


def draw_tindalos(d, rng):
    whistle_body(d, 5, 10, (216, 178, 60, 255), (150, 118, 34, 255))
    d.point((9, 12), fill=(250, 226, 130, 255))
    # sharp angular glyphs -- Tindalos is all angles
    d.line([22, 6, 26, 3], fill=INK, width=1)
    d.line([26, 3, 27, 8], fill=INK, width=1)
    d.line([24, 22, 28, 25], fill=INK, width=1)
    d.point((25, 18), fill=INK)


def draw_toilet(d, rng):
    # portable plastic toilet, side view: tank, bowl, seat up
    d.rectangle([6, 6, 12, 20], fill=(236, 236, 240, 255))     # tank
    d.rectangle([6, 6, 12, 20], outline=(180, 182, 190, 255), width=1)
    d.polygon([(12, 12), (24, 12), (25, 17), (14, 19)],
              fill=(236, 236, 240, 255))                       # bowl
    d.ellipse([14, 10, 26, 15], fill=(248, 248, 250, 255))     # seat ring
    d.ellipse([14, 10, 26, 15], outline=(180, 182, 190, 255), width=1)
    d.ellipse([17, 11, 23, 14], fill=(196, 214, 226, 255))     # bowl water
    d.rectangle([10, 20, 22, 26], fill=(214, 216, 222, 255))   # base box
    d.line([11, 23, 21, 23], fill=(180, 182, 190, 255), width=1)


def draw_toilet_folded(d, rng):
    # collapsed flat: a stack of hinged white panels
    d.rectangle([6, 11, 26, 24], fill=(228, 228, 234, 255))
    d.rectangle([6, 11, 26, 24], outline=(168, 170, 178, 255), width=1)
    for gy in (15, 19):  # fold seams
        d.line([7, gy, 25, gy], fill=(168, 170, 178, 255), width=1)
    d.ellipse([12, 12, 20, 15], outline=(190, 192, 200, 255),
              width=1)                                         # seat imprint
    d.rectangle([22, 13, 25, 17], fill=(190, 192, 200, 255))   # latch


def draw_vortex(d, rng):
    d.ellipse([5, 5, 27, 27], fill=(168, 172, 178, 255))       # stone disc
    d.ellipse([5, 5, 27, 27], outline=(110, 114, 122, 255), width=1)
    pts = []  # carved spiral
    for t in range(0, 40):
        a = t * 0.42
        r = 0.4 + t * 0.22
        pts.append((16 + r * math.cos(a), 16 + r * math.sin(a)))
    d.line(pts, fill=(90, 94, 102, 255), width=1)
    for a in range(0, 360, 45):  # holes around the perimeter
        x = 16 + 9 * math.cos(math.radians(a))
        y = 16 + 9 * math.sin(math.radians(a))
        d.point((int(x), int(y)), fill=(70, 70, 76, 255))
    for wx, wy in ((3, 9), (28, 20), (4, 23)):  # gathering air wisps
        d.arc([wx - 2, wy - 2, wx + 3, wy + 3], start=300, end=140,
              fill=(200, 214, 226, 255), width=1)


def draw_whistle_multi(d, rng):
    whistle_body(d, 3, 6, (70, 70, 78, 255), (36, 36, 42, 255))
    # compass bubble under the chamber
    d.ellipse([10, 19, 20, 29], fill=(226, 232, 238, 255))
    d.ellipse([10, 19, 20, 29], outline=(36, 36, 42, 255), width=1)
    d.line([15, 21, 15, 27], fill=(196, 48, 40, 255), width=1)  # needle
    d.line([12, 24, 18, 24], fill=(90, 94, 102, 255), width=1)
    # magnifier ring on a stem, right
    d.line([21, 12, 25, 17], fill=(52, 52, 58, 255), width=2)
    d.ellipse([23, 16, 30, 23], outline=(52, 52, 58, 255), width=2)
    d.point((25, 18), fill=(200, 220, 240, 255))
    # thermometer strip, left
    d.rectangle([4, 18, 6, 28], fill=(240, 236, 226, 255))
    d.line([5, 20, 5, 26], fill=(196, 48, 40, 255), width=1)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid.endswith("_esapi_plate"):
        draw_esapi(d, rng, grade_of(sid))
    elif sid.endswith("_esbi_plate"):
        draw_esbi(d, rng, grade_of(sid))
    elif sid == "deltoid_plates":
        draw_deltoid(d, rng)
    elif sid == "eod_plate":
        draw_eod_plate(d, rng)
    elif sid == "heavy_steel_ballistic_plate":
        draw_nij(d, rng, heavy=True)
    elif sid == "steel_ballistic_plate":
        draw_nij(d, rng, heavy=False)
    elif sid == "lead_plate_xedra_mark":
        draw_lead(d, rng)
    elif sid == "stab_plate":
        draw_stab(d, rng)
    elif sid == "iotv_groin_plate":
        draw_iotv_groin(d, rng)
    elif sid == "iotv_neck_plate":
        draw_iotv_neck(d, rng)
    elif sid == "iotv_shoulder_plate":
        draw_iotv_shoulder(d, rng)
    elif sid == "ballistic_vest_light":
        draw_vest(d, rng, "mag")
    elif sid == "ballistic_vest_light_operator":
        draw_vest(d, rng, "operator")
    elif sid == "ballistic_vest_light_pouches":
        draw_vest(d, rng, "pouches")
    elif sid == "jacket_eod":
        draw_eod_jacket(d, rng, light=False)
    elif sid == "jacket_eod_light":
        draw_eod_jacket(d, rng, light=True)
    elif sid == "c4_payload":
        draw_c4(d, rng)
    elif sid == "clay_urn":
        draw_urn(d, rng, "clay")
    elif sid == "crystal_urn":
        draw_urn(d, rng, "crystal")
    elif sid == "porcelain_urn":
        draw_urn(d, rng, "porcelain")
    elif sid == "dressage_whip":
        draw_whip(d, rng, crop=False)
    elif sid == "riding_crop":
        draw_whip(d, rng, crop=True)
    elif sid == "gag_ball":
        draw_gag(d, rng)
    elif sid == "glass_cutter":
        draw_glass_cutter(d, rng)
    elif sid == "handcuffs_leather":
        draw_cuffs(d, rng)
    elif sid == "horn_big":
        draw_horn_big(d, rng)
    elif sid == "horn_car":
        draw_horn_car(d, rng)
    elif sid == "nm_teeth":
        draw_teeth(d, rng)
    elif sid == "paddle_plastic":
        draw_paddle(d, rng)
    elif sid == "safe_box":
        draw_safe(d, rng)
    elif sid == "sarcophagus_access_code":
        draw_code_sheet(d, rng)
    elif sid == "sextoy":
        draw_sextoy(d, rng)
    elif sid == "tape_measure":
        draw_tape(d, rng)
    elif sid == "tear_gas_payload":
        draw_gas_payload(d, rng, active=False)
    elif sid == "tear_gas_payload_act":
        draw_gas_payload(d, rng, active=True)
    elif sid == "teargas_sprayer":
        draw_sprayer(d, rng)
    elif sid == "testflames":
        draw_testflames(d, rng)
    elif sid == "tindalos_whistle":
        draw_tindalos(d, rng)
    elif sid == "toilet_travel":
        draw_toilet(d, rng)
    elif sid == "toilet_travel_folded":
        draw_toilet_folded(d, rng)
    elif sid == "vortex_stone":
        draw_vortex(d, rng)
    elif sid == "whistle_multitool":
        draw_whistle_multi(d, rng)
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
                       "preview_item56.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item56.png {prev.size}")


if __name__ == "__main__":
    main()
