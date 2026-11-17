#!/usr/bin/env python3
"""ITEM77: small-caliber ammo families, storage, coins + books --
CDDA Voxel coverage audit.

Ammo families (.22 LR, .303 British, .30 Carbine, .45 Colt): one
upright cartridge per id, bullet profile by load (FMJ spitzer /
round nose, OTM open tip, JSP/SP exposed lead tip, LFN flat lead)
and case finish by variant -- factory clean bright brass, bp_
soot-darkened handmade, matchhead_ polished with a red tip band,
reloaded_ scuffed handload with scratch marks.

Storage: collapsible rubber drums (black 432 gal FLAMMABLE /
beige 55 gal POTABLE WATER), cardboard grenade and mortar tubes,
the twin-rocket Carl Gustav can, steel engine container base + lid.

Coins: reeded-edge discs with face motifs (flattened copper coin,
Hub 01 circuitry gold, Buffalo gold, Silver Eagle, paper coin
wrapper roll, signed paper chit, Tacoma bottle-cap token).

Books: bound book form with per-title cover motif; recipe_migo_plate
is a loose sheaf of papers instead.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "22_fmj",
    "bp_22_fmj",
    "bp_22_lr",
    "matchhead_22_fmj",
    "matchhead_22_lr",
    "reloaded_22_fmj",
    "reloaded_22_lr",
    "303_bthp",
    "303_fmjbt",
    "303_sp",
    "bp_303fmj",
    "bp_303sp",
    "reloaded_303fmj",
    "reloaded_303sp",
    "30carbine_jsp",
    "bp_30carbine",
    "bp_30carbine_jsp",
    "matchhead_30carbine",
    "matchhead_30carbine_jsp",
    "reloaded_30carbine",
    "reloaded_30carbine_jsp",
    "432gal_drum_rubber",
    "55gal_drum_rubber",
    "ammunition_box_grenade",
    "ammunition_can_carlgustav",
    "ammunition_tube_mortar",
    "engine_container_base",
    "engine_container_lid",
    "45colt_cowboy",
    "bp_45colt_cowboy",
    "bp_45colt_fmj",
    "matchhead_45colt_cowboy",
    "matchhead_45colt_fmj",
    "reloaded_45colt_cowboy",
    "reloaded_45colt_fmj",
    "FlatCoin",
    "RobofacCoin",
    "coin_gold",
    "coin_silver",
    "coin_wrapper",
    "signed_chit",
    "tacoma_drink_token",
    "alloy_book",
    "bronze_book",
    "bronze_mag",
    "metal_casting_book",
    "recipe_migo_plate",
    "textbook_mesoam",
    "textbook_weparabic",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (52, 52, 58, 255)
COPPER = (184, 115, 51, 255)
COPPER_DARK = (140, 84, 36, 255)
LEAD = (148, 150, 156, 255)
LEAD_DARK = (104, 106, 112, 255)
STEEL = (150, 154, 160, 255)
STEEL_DARK = (96, 100, 106, 255)
GOLD = (216, 178, 62, 255)
GOLD_DARK = (168, 132, 40, 255)
SILVER = (206, 210, 216, 255)
SILVER_DARK = (150, 156, 164, 255)
RED = (196, 48, 40, 255)
YELLOW = (240, 200, 60, 255)
PAPER = (240, 236, 226, 255)
PAPER_DARK = (206, 200, 184, 255)

# finish -> (case bright, case dark, bullet tone factor)
FINISH = {
    "factory": ((206, 172, 84, 255), (164, 132, 58, 255), 1.0),
    "bp": ((146, 116, 66, 255), (104, 82, 46, 255), 0.72),
    "match": ((226, 192, 100, 255), (182, 148, 70, 255), 1.08),
    "reloaded": ((182, 146, 74, 255), (138, 108, 54, 255), 0.9),
}

# caliber -> (case width, case height, bullet height, rimmed, bottleneck)
CALIBER = {
    "22": (5, 8, 6, True, False),
    "303": (7, 13, 10, True, True),
    "30carbine": (6, 12, 7, False, False),
    "45colt": (9, 10, 7, True, False),
}

# sid -> (caliber, bullet profile, finish)
AMMO = {
    "22_fmj": ("22", "fmj_rn", "factory"),
    "bp_22_fmj": ("22", "fmj_rn", "bp"),
    "bp_22_lr": ("22", "lr", "bp"),
    "matchhead_22_fmj": ("22", "fmj_rn", "match"),
    "matchhead_22_lr": ("22", "lr", "match"),
    "reloaded_22_fmj": ("22", "fmj_rn", "reloaded"),
    "reloaded_22_lr": ("22", "lr", "reloaded"),
    "303_bthp": ("303", "otm", "factory"),
    "303_fmjbt": ("303", "spitzer", "factory"),
    "303_sp": ("303", "sp", "factory"),
    "bp_303fmj": ("303", "spitzer", "bp"),
    "bp_303sp": ("303", "sp", "bp"),
    "reloaded_303fmj": ("303", "spitzer", "reloaded"),
    "reloaded_303sp": ("303", "sp", "reloaded"),
    "30carbine_jsp": ("30carbine", "jsp", "factory"),
    "bp_30carbine": ("30carbine", "fmj_rn", "bp"),
    "bp_30carbine_jsp": ("30carbine", "jsp", "bp"),
    "matchhead_30carbine": ("30carbine", "fmj_rn", "match"),
    "matchhead_30carbine_jsp": ("30carbine", "jsp", "match"),
    "reloaded_30carbine": ("30carbine", "fmj_rn", "reloaded"),
    "reloaded_30carbine_jsp": ("30carbine", "jsp", "reloaded"),
    "45colt_cowboy": ("45colt", "lfn", "factory"),
    "bp_45colt_cowboy": ("45colt", "lfn", "bp"),
    "bp_45colt_fmj": ("45colt", "fmj_rn", "bp"),
    "matchhead_45colt_cowboy": ("45colt", "lfn", "match"),
    "matchhead_45colt_fmj": ("45colt", "fmj_rn", "match"),
    "reloaded_45colt_cowboy": ("45colt", "lfn", "reloaded"),
    "reloaded_45colt_fmj": ("45colt", "fmj_rn", "reloaded"),
}


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ ammo

def draw_bullet(d, cx, mouth_y, bw, bh, profile, tone):
    """Bullet above the case mouth; bw/bh = width/height."""
    x0, x1 = cx - bw // 2, cx + bw // 2
    top = mouth_y - bh
    cop = shade(COPPER, tone)
    cop_d = shade(COPPER_DARK, tone)
    lead = shade(LEAD, tone)
    lead_d = shade(LEAD_DARK, tone)
    if profile == "spitzer" or profile == "otm":
        pts = [(x0, mouth_y), (x0, top + bh // 2),
               (cx - 1, top + 1), (cx, top), (cx + 1, top + 1),
               (x1 - 1, top + bh // 2), (x1 - 1, mouth_y)]
        d.polygon(pts, fill=cop)
        d.line([x0, mouth_y, x0, top + bh // 2], fill=cop_d, width=1)
        if profile == "otm":  # open tip: dark hollow dot at the point
            d.point((cx, top), fill=INK)
            d.point((cx, top + 1), fill=lead_d)
    elif profile == "sp":  # soft point: jacket taper, exposed lead flat
        pts = [(x0, mouth_y), (x0, top + 3), (cx - 1, top + 1),
               (cx + 1, top + 1), (x1 - 1, top + 3), (x1 - 1, mouth_y)]
        d.polygon(pts, fill=cop)
        d.rectangle([cx - 1, top, cx + 1, top + 2], fill=lead)
        d.line([cx - 1, top, cx + 1, top], fill=lead_d, width=1)
    elif profile == "jsp":  # round nose, lead tip exposed
        d.rectangle([x0, top + 2, x1 - 1, mouth_y], fill=cop)
        d.ellipse([x0, top, x1 - 1, top + 5], fill=cop)
        d.ellipse([cx - 2, top, cx + 1, top + 3], fill=lead)
        d.line([x0, mouth_y - 1, x0, top + 3], fill=cop_d, width=1)
    elif profile == "fmj_rn":  # copper round nose
        d.rectangle([x0, top + 2, x1 - 1, mouth_y], fill=cop)
        d.ellipse([x0, top, x1 - 1, top + 5], fill=cop)
        d.line([x0, mouth_y - 1, x0, top + 3], fill=cop_d, width=1)
        d.point((cx - 1, top + 1), fill=shade(cop, 1.15))
    elif profile == "lr":  # heeled lead round nose (rimfire)
        d.rectangle([x0, top + 2, x1 - 1, mouth_y], fill=lead)
        d.ellipse([x0, top, x1 - 1, top + 5], fill=lead)
        d.line([x0, mouth_y - 1, x0, top + 3], fill=lead_d, width=1)
        d.line([x0 + 1, top + 3, x1 - 2, top + 3], fill=lead_d, width=1)
    elif profile == "lfn":  # lead flat nose
        d.rectangle([x0, top + 1, x1 - 1, mouth_y], fill=lead)
        d.rectangle([x0, top, x1 - 1, top + 1], fill=lead_d)
        d.line([x0, mouth_y - 1, x0, top + 1], fill=lead_d, width=1)


def draw_cartridge(d, rng, sid):
    cal, profile, finish = AMMO[sid]
    cw, ch, bh, rimmed, neck = CALIBER[cal]
    case_c, case_d, tone = FINISH[finish]
    cx = 16
    base_y = 28
    mouth_y = base_y - ch
    bw = cw - (2 if neck else 0)
    # bullet first (case overlaps its base)
    draw_bullet(d, cx, mouth_y, bw, bh, profile, tone)
    x0, x1 = cx - cw // 2, cx + cw // 2
    if neck:  # bottleneck: shoulder slopes from case to neck
        d.polygon([(x0, base_y), (x0, mouth_y + 3),
                   (cx - bw // 2, mouth_y), (cx + bw // 2, mouth_y),
                   (x1 - 1, mouth_y + 3), (x1 - 1, base_y)], fill=case_c)
        d.line([x0, base_y, x0, mouth_y + 3], fill=case_d, width=1)
    else:
        d.rectangle([x0, mouth_y, x1 - 1, base_y], fill=case_c)
        d.line([x0, mouth_y, x0, base_y], fill=case_d, width=1)
        d.line([x1 - 2, mouth_y, x1 - 2, base_y], fill=shade(case_c, 1.1),
               width=1)
    # case mouth line + primer / rim
    d.line([x0, mouth_y, x1 - 1, mouth_y], fill=case_d, width=1)
    if rimmed:
        d.rectangle([x0 - 1, base_y, x1, base_y + 1], fill=case_d)
    else:
        d.rectangle([x0, base_y, x1 - 1, base_y + 1], fill=case_d)
    if cal != "22":  # primer cup (rimfire .22 has none visible)
        d.ellipse([cx - 1, base_y, cx + 1, base_y + 1], fill=case_d)
    if finish == "bp":  # soot smudges over case and bullet base
        for _ in range(14):
            px = rng.randint(x0, x1 - 1)
            py = rng.randint(mouth_y - 2, base_y)
            d.point((px, py), fill=(70, 62, 52, 255))
    elif finish == "match":  # polished sheen + red tip band
        d.line([x0 + 1, mouth_y + 2, x0 + 1, base_y - 1],
               fill=shade(case_c, 1.18), width=1)
        band_y = mouth_y - bh + 2
        d.line([cx - bw // 2 + 1, band_y, cx + bw // 2 - 1, band_y],
               fill=RED, width=1)
    elif finish == "reloaded":  # scratches + a duller patch
        for _ in range(7):
            px = rng.randint(x0, x1 - 2)
            py = rng.randint(mouth_y + 1, base_y - 2)
            d.line([px, py, px + 1, py + 1],
                   fill=rng.choice([shade(case_c, 1.2), case_d]), width=1)
        d.rectangle([x0 + 1, base_y - 3, x0 + 2, base_y - 1],
                    fill=shade(case_c, 0.88))


# --------------------------------------------------------------- storage

def drum(d, body, dark, big, label_col, glyphs):
    x0, x1 = (4, 27) if big else (7, 24)
    y0, y1 = (6, 27) if big else (9, 27)
    d.rounded_rectangle([x0, y0, x1, y1], radius=4, fill=body)
    d.rounded_rectangle([x0, y0, x1, y1], radius=4, outline=dark, width=1)
    # collapsed-rubber horizontal creases
    for y in range(y0 + 6, y1 - 2, 5):
        d.line([x0 + 2, y, x1 - 2, y], fill=dark, width=1)
    # valve on top + D-ring handles
    d.rectangle([14, y0 - 2, 17, y0], fill=dark)
    d.point((15, y0 - 2), fill=label_col)
    d.arc([x0 - 1, y0 + 2, x0 + 3, y0 + 7], start=90, end=270, fill=dark,
          width=1)
    d.arc([x1 - 2, y0 + 2, x1 + 2, y0 + 7], start=270, end=90, fill=dark,
          width=1)
    # stencilled text on the top face: glyph ticks in the label colour
    for i in range(glyphs):
        gx = x0 + 3 + i * 3
        d.line([gx, y0 + 2, gx + 1, y0 + 2], fill=label_col, width=1)
        d.line([gx, y0 + 4, gx + 1, y0 + 4], fill=label_col, width=1)


def draw_storage(d, rng, sid):
    if sid == "432gal_drum_rubber":  # black, FLAMMABLE in yellow
        drum(d, (38, 38, 42, 255), (16, 16, 20, 255), True, YELLOW, 7)
        d.point((8, 12), fill=RED)
        d.point((23, 12), fill=RED)
    elif sid == "55gal_drum_rubber":  # beige, POTABLE WATER ONLY
        drum(d, (198, 182, 148, 255), (150, 134, 102, 255), False,
             (90, 84, 70, 255), 5)
    elif sid == "ammunition_box_grenade":
        # small thick cardboard cylinder with a lid seam
        d.rectangle([10, 7, 21, 26], fill=(122, 106, 84, 255))
        d.rectangle([10, 7, 21, 26], outline=(84, 72, 56, 255), width=1)
        d.rectangle([10, 7, 21, 11], fill=(140, 124, 98, 255))
        d.line([10, 11, 21, 11], fill=(84, 72, 56, 255), width=1)
        d.ellipse([13, 16, 18, 21], outline=(84, 72, 56, 255), width=1)
        d.point((15, 18), fill=(84, 72, 56, 255))
    elif sid == "ammunition_tube_mortar":
        # taller thick cardboard tube with cap band
        d.rectangle([12, 4, 19, 28], fill=(122, 106, 84, 255))
        d.rectangle([12, 4, 19, 28], outline=(84, 72, 56, 255), width=1)
        d.rectangle([12, 4, 19, 8], fill=(140, 124, 98, 255))
        d.line([12, 8, 19, 8], fill=(84, 72, 56, 255), width=1)
        d.line([12, 24, 19, 24], fill=(84, 72, 56, 255), width=1)
        for gy in (13, 17, 21):
            d.line([14, gy, 17, gy], fill=(96, 84, 64, 255), width=1)
    elif sid == "ammunition_can_carlgustav":
        # two massive rocket-shaped steel tubes side by side
        for cx in (11, 20):
            d.polygon([(cx - 3, 12), (cx, 5), (cx + 3, 12),
                       (cx + 3, 25), (cx - 3, 25)], fill=STEEL)
            d.polygon([(cx - 3, 12), (cx, 5), (cx + 3, 12),
                       (cx + 3, 25), (cx - 3, 25)], outline=STEEL_DARK)
            d.line([cx - 3, 12, cx + 3, 12], fill=STEEL_DARK, width=1)
            d.rectangle([cx - 3, 22, cx + 3, 25], fill=STEEL_DARK)
            d.point((cx, 17), fill=(240, 200, 60, 255))
        d.line([8, 26, 23, 26], fill=STEEL_DARK, width=1)
    elif sid == "engine_container_base":
        # open steel base: front wall low, brackets visible inside
        d.polygon([(5, 14), (26, 14), (24, 27), (7, 27)], fill=STEEL)
        d.polygon([(5, 14), (26, 14), (24, 27), (7, 27)],
                  outline=STEEL_DARK)
        d.rectangle([7, 16, 24, 19], fill=shade(STEEL, 0.8))
        for bx in (10, 15, 20):  # engine bolt brackets
            d.rectangle([bx, 17, bx + 2, 20], fill=STEEL_DARK)
            d.point((bx + 1, 18), fill=INK)
        d.line([5, 23, 26, 23], fill=STEEL_DARK, width=1)
    elif sid == "engine_container_lid":
        # heavy flat lid slab with rim and lifting eyes
        d.rectangle([4, 11, 27, 20], fill=STEEL)
        d.rectangle([4, 11, 27, 20], outline=STEEL_DARK, width=1)
        d.rectangle([6, 13, 25, 18], outline=STEEL_DARK, width=1)
        for ex in (8, 22):
            d.arc([ex, 8, ex + 2, 12], start=180, end=360, fill=STEEL_DARK,
                  width=1)
        d.line([4, 22, 27, 22], fill=STEEL_DARK, width=1)
        d.line([6, 24, 25, 24], fill=shade(STEEL, 0.85), width=1)


# ----------------------------------------------------------------- coins

def coin_disc(d, cx, cy, r, face, edge, motif):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=face)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=edge, width=1)
    # reeding: ticks around the rim
    for a in range(0, 360, 30):
        import math
        px = cx + int((r - 1) * math.cos(math.radians(a)))
        py = cy + int((r - 1) * math.sin(math.radians(a)))
        d.point((px, py), fill=edge)
    d.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2],
              outline=edge, width=1)
    motif(d, cx, cy, edge)


def draw_coin(d, rng, sid):
    if sid == "FlatCoin":
        # flattened in a novelty machine: wide oval, design stretched
        d.ellipse([4, 11, 27, 21], fill=COPPER)
        d.ellipse([4, 11, 27, 21], outline=COPPER_DARK, width=1)
        d.ellipse([6, 13, 25, 19], outline=COPPER_DARK, width=1)
        for gx in range(9, 23, 3):  # smeared elongated motif lines
            d.line([gx, 15, gx + 2, 15], fill=COPPER_DARK, width=1)
            d.line([gx, 17, gx + 1, 17], fill=COPPER_DARK, width=1)
    elif sid == "RobofacCoin":
        def circuit(d, cx, cy, col):
            d.line([cx - 4, cy, cx + 3, cy], fill=col, width=1)
            d.line([cx - 1, cy, cx - 1, cy - 4], fill=col, width=1)
            d.line([cx + 2, cy, cx + 2, cy + 4], fill=col, width=1)
            for px, py in ((cx - 4, cy), (cx - 1, cy - 4), (cx + 3, cy),
                           (cx + 2, cy + 4)):
                d.point((px, py), fill=col)
            d.rectangle([cx - 3, cy + 2, cx - 2, cy + 3], outline=col)
        coin_disc(d, 16, 16, 10, GOLD, GOLD_DARK, circuit)
    elif sid == "coin_gold":
        def buffalo(d, cx, cy, col):
            d.rectangle([cx - 5, cy - 2, cx + 2, cy + 2], fill=col)
            d.polygon([(cx + 2, cy - 3), (cx + 5, cy - 2), (cx + 4, cy + 1),
                       (cx + 2, cy + 1)], fill=col)  # head + hump
            d.point((cx + 4, cy - 1), fill=GOLD)  # eye
            for lx in (cx - 4, cx + 1):  # legs
                d.line([lx, cy + 2, lx, cy + 4], fill=col, width=1)
            d.line([cx - 5, cy - 1, cx - 6, cy + 1], fill=col, width=1)
        coin_disc(d, 16, 16, 11, GOLD, GOLD_DARK, buffalo)
    elif sid == "coin_silver":
        def eagle(d, cx, cy, col):
            d.polygon([(cx, cy - 5), (cx - 6, cy + 2), (cx - 2, cy + 2),
                       (cx, cy - 1), (cx + 2, cy + 2), (cx + 6, cy + 2)],
                      fill=col)  # spread wings
            d.ellipse([cx - 1, cy - 3, cx + 2, cy], fill=col)  # head
            d.line([cx, cy + 2, cx, cy + 5], fill=col, width=1)
            d.line([cx - 2, cy + 5, cx + 2, cy + 5], fill=col, width=1)
        coin_disc(d, 16, 16, 11, SILVER, SILVER_DARK, eagle)
    elif sid == "coin_wrapper":
        # paper roll of coins, horizontal cylinder, crimped ends
        d.rectangle([6, 12, 25, 20], fill=(168, 160, 148, 255))
        d.rectangle([6, 12, 25, 20], outline=(120, 112, 100, 255), width=1)
        for gx in range(9, 25, 3):  # coin edges showing through the roll
            d.line([gx, 13, gx, 19], fill=(120, 112, 100, 255), width=1)
        d.polygon([(6, 12), (3, 14), (3, 18), (6, 20)],
                  fill=(150, 142, 130, 255))  # crimped end
        d.polygon([(25, 12), (28, 14), (28, 18), (25, 20)],
                  fill=(150, 142, 130, 255))
        d.line([8, 15, 23, 15], fill=(150, 60, 50, 255), width=1)
    elif sid == "signed_chit":
        d.rectangle([8, 6, 24, 26], fill=PAPER)
        d.rectangle([8, 6, 24, 26], outline=(150, 146, 138, 255), width=1)
        for gy in (10, 13, 16):  # text glyph rows
            d.line([10, gy, 22, gy], fill=(168, 166, 160, 255), width=1)
        # signature squiggle
        pts = [(10, 22), (12, 20), (14, 23), (16, 20), (18, 22),
               (20, 21), (22, 22)]
        d.line(pts, fill=(40, 50, 140, 255), width=1)
    elif sid == "tacoma_drink_token":
        # bottle cap from above: crimped starburst edge, stamped logo
        import math
        pts = []
        for i in range(42):
            a = math.radians(i * 360 / 42)
            r = 11 if i % 2 == 0 else 9
            pts.append((16 + r * math.cos(a), 16 + r * math.sin(a)))
        d.polygon(pts, fill=SILVER)
        d.polygon(pts, outline=SILVER_DARK)
        d.ellipse([10, 10, 22, 22], outline=SILVER_DARK, width=1)
        # ranch brand: T under a bar
        d.line([12, 13, 20, 13], fill=STEEL_DARK, width=1)
        d.line([16, 13, 16, 19], fill=STEEL_DARK, width=1)
        d.line([14, 19, 18, 19], fill=STEEL_DARK, width=1)


# ----------------------------------------------------------------- books

def book(d, cover, dark, motif, mag=False, sheaf=False):
    if sheaf:  # loose papers, slightly askew stack
        for i, off in enumerate(((1, 1), (0, 0))):
            d.rectangle([8 + off[0], 5 + off[1], 23 + off[0], 27 + off[1]],
                        fill=PAPER if i else PAPER_DARK,
                        outline=(150, 146, 138, 255), width=1)
        for gy in range(10, 22, 3):
            d.line([10, gy, 21, gy], fill=(168, 166, 160, 255), width=1)
        motif(d, dark)
        return
    x0, x1, y0, y1 = 8, 24, 4, 28
    if mag:
        y0, y1 = 6, 26
    d.rectangle([x0 + 1, y0 + 1, x1 + 1, y1 + 1], fill=PAPER_DARK)
    d.rectangle([x0, y0, x1, y1], fill=cover)
    d.rectangle([x0, y0, x1, y1], outline=dark, width=1)
    d.rectangle([x0, y0, x0 + 3, y1], fill=dark)  # spine
    for gy in range(y0 + 3, y1 - 2, 4):  # spine bands
        d.line([x0, gy, x0 + 3, gy], fill=shade(dark, 0.8), width=1)
    d.rectangle([x1 - 1, y0 + 1, x1, y1 - 1], fill=PAPER)  # page block
    motif(d, dark)


def draw_book(d, rng, sid):
    blue = (58, 84, 148, 255)
    blue_d = (38, 56, 104, 255)
    ochre = (196, 156, 62, 255)
    ochre_d = (140, 108, 40, 255)
    if sid == "alloy_book":
        def m(d, col):  # ingot with a droplet
            d.polygon([(11, 19), (21, 19), (19, 15), (13, 15)], fill=col)
            d.point((16, 11), fill=col)
            d.ellipse([15, 12, 17, 14], fill=col)
        book(d, blue, blue_d, m)
    elif sid == "metal_casting_book":
        def m(d, col):  # crucible pouring a stream
            d.polygon([(12, 13), (18, 13), (17, 18), (13, 18)], fill=col)
            d.line([18, 14, 21, 18], fill=col, width=1)
            d.line([21, 18, 21, 21], fill=col, width=1)
            d.point((21, 22), fill=col)
        book(d, blue, blue_d, m)
    elif sid == "bronze_book":  # huge textbook: wider frame
        def m(d, col):  # spearhead over a shield arc
            d.polygon([(16, 10), (18, 14), (16, 17), (14, 14)], fill=col)
            d.line([16, 17, 16, 20], fill=col, width=1)
            d.arc([11, 17, 21, 25], start=0, end=180, fill=col, width=1)
        book(d, ochre, ochre_d, m)
    elif sid == "bronze_mag":
        def m(d, col):  # crossed spear + round shield
            d.line([12, 12, 20, 20], fill=col, width=1)
            d.polygon([(19, 11), (21, 11), (20, 13)], fill=col)
            d.ellipse([12, 16, 17, 21], outline=col, width=1)
            d.point((14, 18), fill=col)
        book(d, ochre, ochre_d, m, mag=True)
    elif sid == "recipe_migo_plate":
        def m(d, col):  # organic armor plate with vein lines
            d.ellipse([11, 14, 21, 23], fill=(122, 84, 132, 255))
            d.ellipse([11, 14, 21, 23], outline=(84, 56, 96, 255), width=1)
            d.line([13, 17, 19, 20], fill=(84, 56, 96, 255), width=1)
            d.line([19, 16, 14, 21], fill=(84, 56, 96, 255), width=1)
        book(d, blue, blue_d, m, sheaf=True)
    elif sid == "textbook_mesoam":
        def m(d, col):  # stepped pyramid
            for i, w in enumerate((10, 7, 4)):
                y = 20 - i * 3
                d.rectangle([16 - w // 2, y - 2, 16 + w // 2, y], fill=col)
            d.rectangle([15, 9, 17, 11], fill=col)
        book(d, blue, blue_d, m)
    elif sid == "textbook_weparabic":
        def m(d, col):  # scimitar curve
            d.arc([10, 10, 24, 22], start=300, end=140, fill=col, width=2)
            d.line([11, 19, 13, 21], fill=col, width=1)
            d.point((13, 21), fill=col)
        book(d, blue, blue_d, m)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in AMMO:
        draw_cartridge(d, rng, sid)
    elif sid in ("432gal_drum_rubber", "55gal_drum_rubber",
                 "ammunition_box_grenade", "ammunition_can_carlgustav",
                 "ammunition_tube_mortar", "engine_container_base",
                 "engine_container_lid"):
        draw_storage(d, rng, sid)
    elif sid in ("FlatCoin", "RobofacCoin", "coin_gold", "coin_silver",
                 "coin_wrapper", "signed_chit", "tacoma_drink_token"):
        draw_coin(d, rng, sid)
    else:
        draw_book(d, rng, sid)
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
    # contact-sheet preview: 7 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item77.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item77.png {prev.size}")


if __name__ == "__main__":
    main()
