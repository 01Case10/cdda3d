#!/usr/bin/env python3
"""ITEM62: chainmail/chitin foot armor + Exodii ammunition --
CDDA Voxel coverage audit.

Two clusters:

  Foot armor (20, armor/boots.json) -- each drawn as a pair of
    foot covers, front view (shaft above a rounded toe box):
      *_chainmail_feet -- chainmail chausses "like booties made of
        chainmail": a bootie silhouette filled with interlocked
        rings, in the five steel grades of ITEM47 (lc mild light
        matte, mc medium mid-gray, hc high bright polished,
        ch hardened dark blue-gray, qt tempered heat-tinted).
        Sizes: plain standard, xl_ oversize (bigger in frame),
        xs_ undersize (smaller; no xs_mc in this batch).
      chainmail_junk_feet -- budget-steel chain + leather, rust
        and corrosion showing: patchwork rings, mismatched tones,
        leather patches, rust spots.
      foot_protectors_eod -- bulky EOD blast protectors: padded
        Nomex shaft with thick plastic/steel plates and straps.
      toecaps -- rubbery overshoes covering only the toes, with
        shiny steel toe domes: drawn low and small in frame.
      boots_chitin / boots_acidchitin (xl_/xs_ only here) --
        chitin boots: overlapping dark segmented shell plates on
        a leather backing; acidchitin is the green-tinted
        biosilicified variant from acidic ants.

  Exodii ammunition (19, ammo/exodii.json) -- each drawn as the
    cartridge / shell / payload it is, sized by caliber:
      123ln (long rifle cartridge, brass case): 123ln_ap has a
        black AP tip; bp_* are tarnished black-powder loads;
        reloaded_* show scratches, a dent and red primer sealant.
      273x110 Diyua (huge stubby shells): canister open with shot
        visible, flock packed with tiny pellets, slugs as solid
        projectiles graded heavy > medium > light by bulk.
      273x44 Diyua (short shells): HE with a point fuze and
        yellow band, flare with a glowing red star tip,
        thermobaric silver with an orange band, trainer with the
        blue-painted tip of its hollow aluminum projectile.
      33naval_ball -- a hefty brass naval FMJ round.
      khuunaofaai_ammo_* -- pressurized alien payload canisters
        (rounded tank + valve), contents colour-coded: corrosive
        acid green, firework multicolour sparks, foam pale and
        bubbly, incendiary orange-red.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "ch_chainmail_feet",
    "chainmail_junk_feet",
    "foot_protectors_eod",
    "hc_chainmail_feet",
    "lc_chainmail_feet",
    "mc_chainmail_feet",
    "qt_chainmail_feet",
    "toecaps",
    "xl_boots_acidchitin",
    "xl_boots_chitin",
    "xl_ch_chainmail_feet",
    "xl_hc_chainmail_feet",
    "xl_lc_chainmail_feet",
    "xl_mc_chainmail_feet",
    "xl_qt_chainmail_feet",
    "xs_boots_acidchitin",
    "xs_ch_chainmail_feet",
    "xs_hc_chainmail_feet",
    "xs_lc_chainmail_feet",
    "xs_qt_chainmail_feet",
    "123ln_ap",
    "273x110canister",
    "273x110flock",
    "273x110slugheavy",
    "273x110sluglight",
    "273x110slugmedium",
    "273x44HE",
    "273x44flare",
    "273x44thermobaric",
    "273x44trainer",
    "33naval_ball",
    "bp_123ln",
    "bp_123ln_ap",
    "khuunaofaai_ammo_corrosive",
    "khuunaofaai_ammo_firework",
    "khuunaofaai_ammo_foam",
    "khuunaofaai_ammo_incendiary",
    "reloaded_123ln",
    "reloaded_123ln_ap",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

# steel finishes from ITEM47: (base, dark, light)
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

LEATHER = (110, 78, 52, 255)
LEATHER_DARK = (76, 52, 34, 255)
RUST = (150, 84, 40, 255)
RUST_DARK = (110, 58, 28, 255)
NOMEX = (122, 116, 92, 255)
NOMEX_DARK = (88, 84, 64, 255)
PLASTIC_PAD = (70, 72, 78, 255)
RUBBER = (44, 44, 48, 255)
CHITIN = (62, 48, 40, 255)
CHITIN_DARK = (36, 28, 24, 255)
CHITIN_LIGHT = (104, 84, 64, 255)
ACID = (86, 116, 52, 255)
ACID_DARK = (52, 74, 30, 255)
ACID_LIGHT = (140, 176, 92, 255)

BRASS = (202, 168, 82, 255)
BRASS_DARK = (150, 120, 54, 255)
BRASS_TARNISH = (146, 124, 76, 255)
COPPER = (184, 116, 72, 255)
LEAD = (96, 98, 106, 255)
LEAD_DARK = (62, 64, 70, 255)
STEEL_CASE = (140, 144, 150, 255)
BLACK_TIP = (26, 26, 30, 255)
RED_SEAL = (178, 44, 38, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def inside(px, py, poly):
    """Ray-casting point-in-polygon test."""
    ok = False
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if (y1 > py) != (y2 > py):
            if px < (x2 - x1) * (py - y1) / (y2 - y1) + x1:
                ok = not ok
    return ok


# ------------------------------------------------------------- foot armor

def boot_poly(cx, top, h, w):
    """One boot, front view: shaft over a wider rounded toe box."""
    sw = w * 0.72  # shaft half-ish width factor
    x0, x1 = cx - w / 2.0, cx + w / 2.0
    sx0, sx1 = cx - sw / 2.0, cx + sw / 2.0
    toe_y = top + h * 0.62
    return [(sx0, top), (sx1, top), (sx1, toe_y - 2),
            (x1, toe_y), (x1, top + h - 2), (x1 - 2, top + h),
            (x0 + 2, top + h), (x0, top + h - 2), (x0, toe_y),
            (sx0, toe_y - 2)]


def pair_polys(size):
    """(polygons, top, height, width) for a pair at the given size."""
    if size == "xl":
        w, h, top = 13, 26, 3
    elif size == "xs":
        w, h, top = 9, 18, 8
    else:
        w, h, top = 11, 22, 5
    return [boot_poly(9, top, h, w), boot_poly(23, top, h, w)], top, h, w


def draw_chainmail(d, rng, grade, size):
    polys, top, h, w = pair_polys(size)
    base, dark, light = STEELS[grade]
    for poly in polys:
        d.polygon(poly, fill=dark)
        d.polygon(poly, outline=shade(dark, 0.6))
    # interlocked rings: offset rows of small circles inside the boots
    step = 3 if size != "xs" else 2
    row = 0
    y = top + 1
    while y < top + h - 1:
        xoff = (row % 2) * (step // 2 + 1)
        x = 3 + xoff
        while x < 30:
            if any(inside(x, y, p) for p in polys):
                col = light if (x + y + row) % 3 == 0 else base
                if grade == "qt" and row == 3:  # heat-tint band row
                    col = (TINT_STRAW, TINT_PURPLE, TINT_BLUE)[(x // 4) % 3]
                d.ellipse([x - 1, y - 1, x + 1, y + 1], outline=col,
                          width=1)
            x += step
        y += step - 1
        row += 1
    if grade == "hc":  # polished glints
        for poly in polys:
            d.point((int(poly[0][0]) + 1, top + 2), fill=(255, 255, 255, 255))
            d.point((int(poly[0][0]) + 2, top + 5), fill=(255, 255, 255, 255))


def draw_junk_chainmail(d, rng):
    polys, top, h, w = pair_polys("standard")
    tones = [STEELS["lc"], STEELS["mc"],
             ((150, 128, 96, 255), (104, 88, 62, 255), (188, 164, 124, 255)),
             ((138, 110, 84, 255), (96, 74, 54, 255), (176, 144, 108, 255))]
    for poly in polys:
        d.polygon(poly, fill=tones[2][1])
        d.polygon(poly, outline=LEATHER_DARK)
    # a leather patch on the left boot, stitched on
    d.rectangle([5, top + 9, 11, top + 14], fill=LEATHER)
    d.rectangle([5, top + 9, 11, top + 14], outline=LEATHER_DARK, width=1)
    # patchwork rings in mismatched tones, irregular spacing
    y = top + 1
    row = 0
    while y < top + h - 1:
        x = 3 + (row % 2)
        while x < 30:
            if any(inside(x, y, p) for p in polys) \
                    and not (5 <= x <= 11 and top + 9 <= y <= top + 14):
                base, dark, light = tones[rng.randrange(len(tones))]
                col = light if rng.random() < 0.3 else base
                d.ellipse([x - 1, y - 1, x + 1, y + 1], outline=col, width=1)
                if rng.random() < 0.22:  # rust bloom on the ring
                    d.point((x, y), fill=RUST)
            x += rng.choice([3, 3, 4])
        y += 2
        row += 1
    for _ in range(6):  # corrosion spots
        x, y = rng.randrange(4, 28), rng.randrange(top + 2, top + h - 2)
        if any(inside(x, y, p) for p in polys):
            d.point((x, y), fill=RUST_DARK)
            d.point((x + 1, y), fill=RUST)


def draw_eod(d, rng):
    polys, top, h, w = pair_polys("xl")  # bulky: drawn at xl bulk
    for poly in polys:
        d.polygon(poly, fill=NOMEX)
        d.polygon(poly, outline=NOMEX_DARK)
    for cx in (9, 23):
        # thick blast plate over the toe and shin
        d.rectangle([cx - 5, top + 13, cx + 5, top + 24], fill=PLASTIC_PAD)
        d.rectangle([cx - 5, top + 13, cx + 5, top + 24],
                    outline=shade(PLASTIC_PAD, 0.55), width=1)
        d.rectangle([cx - 4, top + 2, cx + 4, top + 11],
                    fill=shade(PLASTIC_PAD, 1.15))
        d.rectangle([cx - 4, top + 2, cx + 4, top + 11],
                    outline=shade(PLASTIC_PAD, 0.55), width=1)
        for ry in (top + 4, top + 8, top + 15, top + 19, top + 23):
            d.point((cx - 3, ry), fill=(200, 204, 210, 255))  # rivets
            d.point((cx + 3, ry), fill=(200, 204, 210, 255))
        # straps across the shaft
        d.line([cx - 5, top + 12, cx + 5, top + 12], fill=NOMEX_DARK,
               width=2)


def draw_toecaps(d, rng):
    # rubbery overshoes covering only the toes: low in frame
    for cx in (9, 23):
        d.polygon([(cx - 6, 20), (cx + 6, 20), (cx + 6, 27),
                   (cx + 4, 29), (cx - 4, 29), (cx - 6, 27)], fill=RUBBER)
        d.polygon([(cx - 6, 20), (cx + 6, 20), (cx + 6, 27),
                   (cx + 4, 29), (cx - 4, 29), (cx - 6, 27)],
                  outline=(20, 20, 24, 255))
        # shiny steel toe dome
        d.ellipse([cx - 5, 19, cx + 5, 27], fill=STEELS["hc"][0])
        d.ellipse([cx - 5, 19, cx + 5, 27], outline=STEELS["hc"][1],
                  width=1)
        d.ellipse([cx - 3, 20, cx, 23], fill=(248, 250, 252, 255))
        d.line([cx - 6, 28, cx + 6, 28], fill=(20, 20, 24, 255), width=1)


def draw_chitin(d, rng, acid, size):
    polys, top, h, w = pair_polys(size)
    if acid:
        base, dark, light = ACID, ACID_DARK, ACID_LIGHT
    else:
        base, dark, light = CHITIN, CHITIN_DARK, CHITIN_LIGHT
    for poly in polys:
        d.polygon(poly, fill=LEATHER_DARK)  # leather backing at the edges
    # overlapping segmented shell plates: rows of arcs down each boot
    for poly in polys:
        xs = [p[0] for p in poly]
        x0, x1 = int(min(xs)) + 1, int(max(xs)) - 1
        rows = max(3, h // 4)
        for r in range(rows):
            y0 = top + 1 + r * 4
            y1 = min(y0 + 5, top + h - 1)
            if y0 >= top + h - 1:
                break
            col = base if r % 2 == 0 else shade(base, 0.82)
            d.rectangle([x0, y0, x1, y1], fill=col)
            d.arc([x0, y0 - 2, x1, y1 + 1], start=0, end=180,
                  fill=light, width=1)
            d.line([x0, y1, x1, y1], fill=dark, width=1)
            mid = (x0 + x1) // 2  # plate split / spine ridge
            d.line([mid, y0, mid, y1], fill=dark, width=1)
            if acid and r == 1:  # acid sheen
                d.line([x0 + 1, y0 + 1, x1 - 1, y0 + 1],
                       fill=ACID_LIGHT, width=1)
        d.polygon(poly, outline=dark)


# ------------------------------------------------------------------- ammo

def cartridge(d, cx, top, case_h, case_w, case_col, case_dark):
    """A vertical cartridge case; returns the case mouth y."""
    x0, x1 = cx - case_w // 2, cx + case_w // 2
    d.rectangle([x0, top, x1, top + case_h], fill=case_col)
    d.line([x0, top, x0, top + case_h], fill=case_dark, width=1)
    d.line([x1, top, x1, top + case_h], fill=case_dark, width=1)
    d.line([x0 + 1, top, x0 + 1, top + case_h], fill=shade(case_col, 1.18),
           width=1)
    # extractor groove + rim at the base
    d.line([x0, top + case_h - 2, x1, top + case_h - 2], fill=case_dark,
           width=1)
    d.rectangle([x0 - 1, top + case_h, x1 + 1, top + case_h + 1],
                fill=case_dark)
    return top


def bullet(d, cx, y_base, h, w, col, dark, tip_col=None, tip_h=3):
    """A pointed projectile sitting on the case mouth at y_base."""
    x0, x1 = cx - w // 2, cx + w // 2
    top = y_base - h
    d.polygon([(x0, y_base), (x0, top + 3), (cx, top), (x1, top + 3),
               (x1, y_base)], fill=col)
    d.polygon([(x0, y_base), (x0, top + 3), (cx, top), (x1, top + 3),
               (x1, y_base)], outline=dark)
    d.line([cx - 1, top + 3, cx - 1, y_base], fill=shade(col, 1.15),
           width=1)
    if tip_col is not None:
        d.polygon([(cx - max(1, w // 4), top + tip_h), (cx, top),
                   (cx + max(1, w // 4), top + tip_h)], fill=tip_col)
    return top


def draw_123ln(d, rng, sid):
    ap = sid.endswith("_ap")
    bp = sid.startswith("bp_")
    reloaded = sid.startswith("reloaded_")
    case_col = BRASS_TARNISH if (bp or reloaded) else BRASS
    case_dark = shade(case_col, 0.68)
    mouth = cartridge(d, 16, 13, 15, 6, case_col, case_dark)
    tip = BLACK_TIP if ap else None
    bullet(d, 16, mouth, 10, 5, COPPER, shade(COPPER, 0.65),
           tip_col=tip, tip_h=4)
    if bp:  # black-powder fouling band at the mouth
        d.line([13, 12, 19, 12], fill=(40, 36, 32, 255), width=2)
    if reloaded:  # scratches, a dent, red primer sealant ring
        d.line([14, 18, 15, 22], fill=case_dark, width=1)
        d.line([18, 20, 17, 25], fill=case_dark, width=1)
        d.point((19, 24), fill=case_dark)
        d.line([13, 28, 19, 28], fill=RED_SEAL, width=1)
    # a second loose round leaning beside it
    mouth2 = cartridge(d, 25, 17, 11, 5, case_col, case_dark)
    bullet(d, 25, mouth2, 8, 4, COPPER, shade(COPPER, 0.65),
           tip_col=tip, tip_h=3)


def draw_273x110(d, rng, sid):
    kind = sid[len("273x110"):]
    # huge stubby shell: fat short case, mouth at y=10
    mouth = cartridge(d, 16, 10, 17, 15, BRASS, BRASS_DARK)
    x0, x1 = 16 - 7, 16 + 7
    if kind == "canister":
        # open canister mouth with shot pellets visible inside
        d.rectangle([x0, 4, x1, 10], fill=LEAD_DARK)
        for px in range(x0 + 1, x1, 3):
            for py in (5, 8):
                d.ellipse([px, py, px + 2, py + 2], fill=LEAD)
                d.point((px, py), fill=(210, 214, 220, 255))
        d.line([x0, 4, x1, 4], fill=BRASS_DARK, width=1)
    elif kind == "flock":
        # packed with a ludicrous number of tiny pellets, some spilling
        d.rectangle([x0, 5, x1, 10], fill=LEAD_DARK)
        for _ in range(46):
            px, py = rng.randrange(x0 + 1, x1 - 1), rng.randrange(5, 10)
            d.point((px, py), fill=(196, 200, 208, 255))
        for px, py in ((7, 12), (25, 14), (9, 16), (24, 11)):
            d.point((px, py), fill=LEAD)  # spilled pellets on the ground
    else:  # slugs: solid projectile, bulk graded by load
        bulk = {"slugheavy": (13, 9, LEAD_DARK),
                "slugmedium": (11, 7, LEAD),
                "sluglight": (9, 6, (150, 154, 162, 255))}[kind]
        w, h, col = bulk
        sx0, sx1 = 16 - w // 2, 16 + w // 2
        d.rectangle([sx0, 10 - h, sx1, 10], fill=col)
        d.rectangle([sx0, 10 - h, sx1, 10], outline=shade(col, 0.6),
                    width=1)
        d.line([sx0 + 1, 10 - h + 1, sx0 + 1, 9], fill=shade(col, 1.25),
               width=1)
        if kind == "sluglight":  # hollow point
            d.rectangle([16 - 1, 10 - h, 16 + 1, 10 - h + 2],
                        fill=LEAD_DARK)
        if kind == "slugheavy":  # driving bands
            d.line([sx0, 7, sx1, 7], fill=BRASS_DARK, width=1)
    # caliber band on the case
    d.line([x0, 20, x1, 20], fill=BRASS_DARK, width=1)
    d.line([x0, 23, x1, 23], fill=BRASS_DARK, width=1)


def draw_273x44(d, rng, sid):
    kind = sid[len("273x44"):]
    # short shell: case mouth at y=16, much shorter than the 110
    mouth = cartridge(d, 16, 16, 12, 13, BRASS, BRASS_DARK)
    if kind == "HE":
        bullet(d, 16, mouth, 12, 9, (96, 110, 62, 255),
               (62, 74, 40, 255))
        d.line([12, 10, 20, 10], fill=(232, 200, 60, 255),
               width=2)  # yellow HE band
        d.point((16, 4), fill=LEAD_DARK)  # point-detonating fuze
    elif kind == "flare":
        bullet(d, 16, mouth, 11, 9, (178, 44, 38, 255),
               (120, 28, 24, 255))
        # glowing star tip
        d.ellipse([14, 5, 18, 9], fill=(255, 120, 60, 255))
        d.point((16, 7), fill=(255, 230, 160, 255))
        for px, py in ((11, 4), (21, 5), (16, 2)):
            d.point((px, py), fill=(255, 170, 80, 255))
    elif kind == "thermobaric":
        bullet(d, 16, mouth, 11, 10, (168, 172, 178, 255),
               (104, 108, 114, 255))
        d.line([11, 11, 21, 11], fill=(232, 126, 40, 255),
               width=2)  # orange band
        d.rectangle([14, 5, 18, 7], fill=(120, 124, 130, 255))  # cap
    elif kind == "trainer":
        # hollow aluminum projectile, blue at the tip
        bullet(d, 16, mouth, 12, 9, (196, 200, 206, 255),
               (130, 134, 140, 255), tip_col=(52, 92, 190, 255), tip_h=5)
    d.line([10, 22, 22, 22], fill=BRASS_DARK, width=1)


def draw_naval(d, rng):
    # hefty naval FMJ: big brass case, full-metal-jacket bullet
    mouth = cartridge(d, 15, 12, 16, 9, BRASS, BRASS_DARK)
    bullet(d, 15, mouth, 11, 7, COPPER, shade(COPPER, 0.65))
    d.line([11, 19, 19, 19], fill=BRASS_DARK, width=1)
    # a second round lying at the base
    d.rectangle([21, 24, 29, 27], fill=BRASS)
    d.polygon([(29, 24), (31, 25), (29, 27)], fill=COPPER)
    d.line([23, 24, 23, 27], fill=BRASS_DARK, width=1)


KHUU = {
    "khuunaofaai_ammo_corrosive": ((120, 200, 70, 255), "acid"),
    "khuunaofaai_ammo_firework": ((210, 190, 200, 255), "firework"),
    "khuunaofaai_ammo_foam": ((238, 240, 238, 255), "foam"),
    "khuunaofaai_ammo_incendiary": ((232, 110, 40, 255), "fire"),
}


def draw_khuunaofaai(d, rng, sid):
    fill, kind = KHUU[sid]
    # pressurized payload tank: rounded canister with a valve top
    d.rectangle([11, 10, 21, 27], fill=(150, 154, 160, 255))
    d.ellipse([11, 7, 21, 13], fill=(170, 174, 180, 255))
    d.rectangle([11, 10, 21, 27], outline=(88, 92, 98, 255), width=1)
    d.rectangle([14, 4, 18, 8], fill=(104, 108, 114, 255))  # valve
    d.line([16, 2, 16, 4], fill=(104, 108, 114, 255), width=2)
    # contents window showing the payload colour
    d.rectangle([13, 14, 19, 24], fill=fill)
    d.rectangle([13, 14, 19, 24], outline=(88, 92, 98, 255), width=1)
    if kind == "acid":  # corrosive: drips + bubbles, warning band
        for px, py in ((14, 16), (17, 19), (15, 22)):
            d.point((px, py), fill=(230, 255, 200, 255))
        d.line([13, 24, 19, 24], fill=(40, 60, 20, 255), width=1)
    elif kind == "firework":  # multicolour sparks in the window
        for col in ((230, 60, 50, 255), (240, 200, 60, 255),
                    (70, 120, 230, 255), (90, 200, 90, 255)):
            for _ in range(3):
                d.point((rng.randrange(14, 19), rng.randrange(15, 24)),
                        fill=col)
        for px, py in ((9, 8), (23, 6), (24, 12)):  # sparks outside
            d.point((px, py), fill=(240, 200, 60, 255))
    elif kind == "foam":  # pale foam: bubble circles
        for px, py, r in ((15, 17, 1), (17, 20, 1), (15, 22, 0),
                          (18, 16, 0), (16, 19, 0)):
            if r:
                d.ellipse([px - 1, py - 1, px + 1, py + 1],
                          outline=(170, 176, 170, 255), width=1)
            else:
                d.point((px, py), fill=(170, 176, 170, 255))
    elif kind == "fire":  # incendiary: flame tongues in the window
        for fx in (14, 16, 18):
            d.polygon([(fx, 22), (fx + 1, 17), (fx + 2, 22)],
                      fill=(240, 180, 50, 255))
        d.point((16, 20), fill=(255, 240, 180, 255))
    # alien glyph band under the tank
    for gx in range(10, 23, 3):
        d.point((gx, 29), fill=(104, 108, 114, 255))
        d.point((gx + 1, 30), fill=(104, 108, 114, 255))


# -------------------------------------------------------------------- main

FOOT_GRADES = ("ch", "hc", "lc", "mc", "qt")


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid.endswith("_chainmail_feet"):
        rest = sid[:-len("_chainmail_feet")]
        size = "standard"
        if rest.startswith("xl_"):
            size, rest = "xl", rest[3:]
        elif rest.startswith("xs_"):
            size, rest = "xs", rest[3:]
        draw_chainmail(d, rng, rest, size)
    elif sid == "chainmail_junk_feet":
        draw_junk_chainmail(d, rng)
    elif sid == "foot_protectors_eod":
        draw_eod(d, rng)
    elif sid == "toecaps":
        draw_toecaps(d, rng)
    elif "boots_acidchitin" in sid or "boots_chitin" in sid:
        size = "xl" if sid.startswith("xl_") else "xs"
        draw_chitin(d, rng, "acidchitin" in sid, size)
    elif sid in ("123ln_ap", "bp_123ln", "bp_123ln_ap",
                 "reloaded_123ln", "reloaded_123ln_ap"):
        draw_123ln(d, rng, sid)
    elif sid.startswith("273x110"):
        draw_273x110(d, rng, sid)
    elif sid.startswith("273x44"):
        draw_273x44(d, rng, sid)
    elif sid == "33naval_ball":
        draw_naval(d, rng)
    elif sid in KHUU:
        draw_khuunaofaai(d, rng, sid)
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
    # contact-sheet preview: 7 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item62.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item62.png {prev.size}")


if __name__ == "__main__":
    main()
