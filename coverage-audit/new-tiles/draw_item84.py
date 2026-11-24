#!/usr/bin/env python3
"""ITEM84: magazines, pistol/rifle ammo finishes, explosives, plates,
belt links, electronics references, billets and traps -- CDDA Voxel
coverage audit.

Ammo families are drawn as one upright cartridge: case profile by
caliber (.357 SIG bottlenecked, .357 Magnum a long straight case,
.270 Winchester a long bottleneck rifle case, the rest straight
pistol cases), bullet by load (JHP shows the dark hollow cavity,
JSP an exposed flat lead tip, FMJ a full copper jacket), and finish
by prefix: factory clean bright brass, bp_ soot-darkened black
powder, matchhead_ polished with a red-brown matchhead band at the
case head, reloaded_ scuffed two-tone handloads.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "22_ar15mag_10",
    "22_ar15mag_28",
    "22_speedloader8",
    "mp1522_mag",
    "32_acp_jhp",
    "bp_32_acp",
    "matchhead_32_acp",
    "reloaded_32_acp",
    "357mag_jhp",
    "bp_357mag_fmj",
    "matchhead_357mag_fmj",
    "reloaded_357mag_fmj",
    "357sig_jhp",
    "bp_357sig_fmj",
    "matchhead_357sig_fmj",
    "reloaded_357sig_fmj",
    "38_super",
    "bp_38super_fmj",
    "matchhead_38_super_fmj",
    "reloaded_38super_fmj",
    "40fmj",
    "bp_40fmj",
    "matchhead_40fmj",
    "reloaded_40fmj",
    "60mm_propellant",
    "c4",
    "c4armed",
    "gelled_molotov",
    "alloy_sheet",
    "chitin_plate",
    "counterweight",
    "wood_plate",
    "ammolink223",
    "ammolink308",
    "ammolink40mm",
    "ammolink50",
    "arduino_experiments",
    "exodii_wiring_kit_reference",
    "ic_reference_electronics",
    "robofac_yrax_trifacet_deactivation_manual",
    "billet_bone",
    "billet_wood",
    "drill_rock_primitive",
    "punch_bone",
    "boobytrap",
    "caltrops_teeth",
    "crossbow_trap",
    "light_snare_kit",
    "bp_270win_fmj",
    "bp_270win_jsp",
    "reloaded_270win_fmj",
    "reloaded_270win_jsp",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
BRASS = (198, 160, 76, 255)
BRASS_BRIGHT = (226, 190, 102, 255)
BRASS_SOOT = (146, 116, 62, 255)
BRASS_DULL = (172, 138, 70, 255)
SOOT = (58, 50, 44, 255)
COPPER = (186, 116, 66, 255)
COPPER_DARK = (150, 88, 50, 255)
LEAD = (128, 130, 138, 255)
LEAD_DARK = (88, 90, 98, 255)
STEEL = (150, 154, 162, 255)
STEEL_DARK = (92, 96, 104, 255)
POLYMER = (44, 44, 50, 255)
POLYMER_LIGHT = (74, 74, 82, 255)
RED = (196, 48, 40, 255)
MATCHHEAD = (150, 62, 40, 255)
BONE = (232, 224, 200, 255)
BONE_DARK = (196, 186, 158, 255)
WOOD = (120, 84, 48, 255)
WOOD_DARK = (88, 60, 32, 255)
WOOD_LIGHT = (156, 116, 72, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# -------------------------------------------------------------------- ammo

def draw_cartridge(d, rng, profile, bullet, finish):
    """One upright cartridge centred at x=16, base at y=28.

    profile: (case_w, case_top_y, neck) -- neck=True draws a
    bottleneck shoulder. bullet: 'fmj' | 'jhp' | 'jsp'.
    finish: 'factory' | 'bp' | 'matchhead' | 'reloaded'.
    """
    case_w, case_top, neck = profile
    base = 28
    brass = {"factory": BRASS, "bp": BRASS_SOOT,
             "matchhead": BRASS_BRIGHT, "reloaded": BRASS_DULL}[finish]
    cx = 16
    x0, x1 = cx - case_w // 2, cx + case_w // 2 - 1
    # rim / case head
    d.rectangle([x0 - 1, base - 2, x1 + 1, base], fill=shade(brass, 0.78))
    if neck:
        shoulder = case_top + 5
        nw = max(3, case_w - 4)
        nx0, nx1 = cx - nw // 2, cx + nw // 2 - 1
        d.rectangle([x0, shoulder, x1, base - 2], fill=brass)
        d.polygon([(x0, shoulder), (x1, shoulder), (nx1, case_top + 2),
                   (nx0, case_top + 2)], fill=brass)
        d.rectangle([nx0, case_top, nx1, case_top + 2], fill=brass)
        mouth_y, bw = case_top, nw
    else:
        d.rectangle([x0, case_top, x1, base - 2], fill=brass)
        mouth_y, bw = case_top, case_w
    # case shading: dark left edge, light right edge
    d.line([x0, case_top + (5 if neck else 0), x0, base - 2],
           fill=shade(brass, 0.8), width=1)
    d.line([x1, case_top + (5 if neck else 0), x1, base - 2],
           fill=shade(brass, 1.12), width=1)
    # bullet
    bx0, bx1 = cx - bw // 2, cx + bw // 2 - 1
    btop = mouth_y - 6
    if bullet == "fmj":
        d.rectangle([bx0, mouth_y - 3, bx1, mouth_y], fill=COPPER)
        d.polygon([(bx0, mouth_y - 3), (bx1, mouth_y - 3),
                   (bx1 - 1, btop + 1), (bx0 + 1, btop + 1)], fill=COPPER)
        d.line([bx0 + 1, btop + 1, bx1 - 1, btop + 1], fill=COPPER_DARK,
               width=1)
    elif bullet == "jhp":
        d.rectangle([bx0, btop + 1, bx1, mouth_y], fill=COPPER)
        d.rectangle([cx - 1, btop, cx + 1, btop + 2], fill=LEAD_DARK)
        d.point((cx, btop + 1), fill=(30, 30, 34, 255))
    elif bullet == "jsp":
        d.rectangle([bx0, mouth_y - 3, bx1, mouth_y], fill=COPPER)
        d.rectangle([bx0 + 1, btop + 1, bx1 - 1, mouth_y - 3], fill=LEAD)
        d.line([bx0 + 1, btop + 1, bx1 - 1, btop + 1], fill=LEAD_DARK,
               width=1)
    # finish treatment
    if finish == "bp":  # soot smudges and specks
        for _ in range(9):
            sx = rng.randint(x0, x1)
            sy = rng.randint(case_top + 1, base - 3)
            d.point((sx, sy), fill=SOOT)
        d.line([x0, case_top + (6 if neck else 1), x1,
                case_top + (6 if neck else 1)], fill=SOOT, width=1)
    elif finish == "matchhead":  # polished band + highlight
        d.rectangle([x0, base - 5, x1, base - 4], fill=MATCHHEAD)
        d.line([cx + 1, case_top + 1, cx + 1, base - 6],
               fill=(246, 222, 150, 255), width=1)
    elif finish == "reloaded":  # scuffs, two-tone case, worn rim
        for _ in range(5):
            sy = rng.randint(case_top + 1, base - 4)
            sx = rng.randint(x0, x1 - 2)
            d.line([sx, sy, sx + 2, sy], fill=shade(brass, 0.72), width=1)
        d.rectangle([x0, (case_top + base) // 2, x1,
                     (case_top + base) // 2 + 1], fill=shade(brass, 0.88))
        d.point((x0 - 1, base - 1), fill=STEEL_DARK)


# caliber profiles: (case_w, case_top, neck)
AMMO = {
    # .32 ACP: small straight case
    "32_acp_jhp": ((6, 13, False), "jhp", "factory"),
    "bp_32_acp": ((6, 13, False), "fmj", "bp"),
    "matchhead_32_acp": ((6, 13, False), "fmj", "matchhead"),
    "reloaded_32_acp": ((6, 13, False), "fmj", "reloaded"),
    # .357 Magnum: long straight case
    "357mag_jhp": ((7, 8, False), "jhp", "factory"),
    "bp_357mag_fmj": ((7, 8, False), "fmj", "bp"),
    "matchhead_357mag_fmj": ((7, 8, False), "fmj", "matchhead"),
    "reloaded_357mag_fmj": ((7, 8, False), "fmj", "reloaded"),
    # .357 SIG: bottleneck
    "357sig_jhp": ((8, 11, True), "jhp", "factory"),
    "bp_357sig_fmj": ((8, 11, True), "fmj", "bp"),
    "matchhead_357sig_fmj": ((8, 11, True), "fmj", "matchhead"),
    "reloaded_357sig_fmj": ((8, 11, True), "fmj", "reloaded"),
    # .38 Super: medium straight case
    "38_super": ((7, 11, False), "jhp", "factory"),
    "bp_38super_fmj": ((7, 11, False), "fmj", "bp"),
    "matchhead_38_super_fmj": ((7, 11, False), "fmj", "matchhead"),
    "reloaded_38super_fmj": ((7, 11, False), "fmj", "reloaded"),
    # .40 S&W: medium straight case, flat-point FMJ
    "40fmj": ((8, 12, False), "fmj", "factory"),
    "bp_40fmj": ((8, 12, False), "fmj", "bp"),
    "matchhead_40fmj": ((8, 12, False), "fmj", "matchhead"),
    "reloaded_40fmj": ((8, 12, False), "fmj", "reloaded"),
    # .270 Winchester: long bottleneck rifle case (no factory in batch)
    "bp_270win_fmj": ((8, 5, True), "fmj", "bp"),
    "bp_270win_jsp": ((8, 5, True), "jsp", "bp"),
    "reloaded_270win_fmj": ((8, 5, True), "fmj", "reloaded"),
    "reloaded_270win_jsp": ((8, 5, True), "jsp", "reloaded"),
}


# --------------------------------------------------------------- magazines

def draw_mag(d, rng, sid):
    if sid == "22_speedloader8":
        # circular speedloader: disc with 8 rounds nose-in around a hub
        cx, cy = 16, 16
        d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], fill=POLYMER)
        d.ellipse([cx - 10, cy - 10, cx + 10, cy + 10], outline=INK)
        import math
        for i in range(8):
            a = i * math.pi / 4
            px = int(cx + 6.5 * math.cos(a))
            py = int(cy + 6.5 * math.sin(a))
            d.ellipse([px - 2, py - 2, px + 2, py + 2], fill=BRASS)
            d.point((px, py), fill=COPPER_DARK)
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=STEEL_DARK)
        d.point((cx, cy), fill=STEEL)
        return
    if sid == "22_ar15mag_10":
        x0, x1, y0, y1 = 12, 20, 9, 27  # short straight mag
        curve = 0
    elif sid == "22_ar15mag_28":
        x0, x1, y0, y1 = 11, 20, 5, 28  # long, curved
        curve = 3
    else:  # mp1522_mag: ribbed straight varmint mag
        x0, x1, y0, y1 = 12, 20, 8, 27
        curve = 1
    # body (slight forward curve at the base for the 28-rounder)
    d.polygon([(x0, y0), (x1, y0), (x1 + curve, y1 - 3),
               (x0 + curve, y1 - 3)], fill=POLYMER)
    d.polygon([(x0, y0), (x1, y0), (x1 + curve, y1 - 3),
               (x0 + curve, y1 - 3)], outline=INK)
    # feed lips + top cartridge
    d.rectangle([x0 + 1, y0 - 2, x1 - 1, y0], fill=POLYMER_LIGHT)
    d.rectangle([15, y0 - 5, 17, y0 - 2], fill=COPPER)
    # ribs
    for ry in range(y0 + 4, y1 - 5, 4):
        off = curve * (ry - y0) // max(1, (y1 - y0))
        d.line([x0 + 1 + off, ry, x1 - 1 + off, ry], fill=POLYMER_LIGHT,
               width=1)
    # witness window with rounds + spring zigzag (mp1522)
    if sid == "mp1522_mag":
        d.rectangle([14, y0 + 5, 18, y1 - 7], fill=(24, 24, 28, 255))
        for wy in range(y0 + 7, y1 - 9, 3):
            d.line([15, wy, 17, wy], fill=BRASS_DULL, width=1)
        pts = [(15, y1 - 8), (17, y1 - 6), (15, y1 - 4), (17, y1 - 2)]
        d.line(pts, fill=STEEL, width=1)
    # baseplate
    d.rectangle([x0 + curve - 1, y1 - 3, x1 + curve + 1, y1 - 1],
                fill=POLYMER_LIGHT)


# -------------------------------------------------------------- explosives

def draw_explosive(d, rng, sid):
    if sid == "60mm_propellant":
        # donut charge with a wedge missing ("donut without a piece")
        d.ellipse([6, 6, 26, 26], fill=(216, 188, 130, 255))
        d.ellipse([6, 6, 26, 26], outline=INK)
        d.ellipse([12, 12, 20, 20], fill=(0, 0, 0, 0))
        d.ellipse([12, 12, 20, 20], outline=INK)
        d.polygon([(16, 16), (27, 8), (28, 15)], fill=(0, 0, 0, 0))
        d.line([16, 16, 27, 8], fill=INK, width=1)
        d.line([16, 16, 28, 15], fill=INK, width=1)
        for _ in range(6):  # pressed-powder grain
            d.point((rng.randint(8, 24), rng.randint(8, 24)),
                    fill=(188, 160, 104, 255))
        return
    if sid in ("c4", "c4armed"):
        # wrapped off-white block with printed label
        d.rectangle([7, 10, 25, 24], fill=(224, 210, 172, 255))
        d.rectangle([7, 10, 25, 24], outline=INK)
        d.line([7, 14, 25, 14], fill=(196, 182, 140, 255), width=1)
        d.rectangle([10, 16, 22, 22], fill=(240, 236, 226, 255))
        d.rectangle([10, 16, 22, 22], outline=INK)
        d.line([11, 18, 21, 18], fill=RED, width=1)
        d.line([11, 20, 18, 20], fill=INK, width=1)
        if sid == "c4armed":
            # blasting cap in the top face, wires to a timer
            d.rectangle([14, 5, 17, 11], fill=STEEL)
            d.rectangle([14, 5, 17, 7], fill=STEEL_DARK)
            d.line([15, 5, 24, 3], fill=RED, width=1)
            d.line([16, 5, 25, 5], fill=INK, width=1)
            d.rectangle([23, 2, 29, 8], fill=POLYMER)
            d.rectangle([23, 2, 29, 8], outline=INK)
            d.point((25, 4), fill=RED)
            d.point((27, 4), fill=RED)
            d.point((26, 6), fill=(250, 190, 80, 255))
        return
    if sid == "gelled_molotov":
        # bottle of thickened fuel with a rag wick
        d.polygon([(13, 8), (19, 8), (19, 12), (22, 16), (22, 27),
                   (10, 27), (10, 16), (13, 12)], fill=(86, 74, 40, 255))
        d.polygon([(13, 8), (19, 8), (19, 12), (22, 16), (22, 27),
                   (10, 27), (10, 16), (13, 12)], outline=INK)
        # viscous amber gel fill with lighter blobs clinging
        d.rectangle([11, 18, 21, 26], fill=(206, 118, 34, 255))
        for bx, by in ((13, 20), (18, 22), (15, 24), (19, 19)):
            d.ellipse([bx, by, bx + 2, by + 2], fill=(232, 148, 52, 255))
        d.line([12, 17, 12, 26], fill=(240, 170, 80, 255), width=1)
        # rag wick stuffed in the neck, one end draped
        d.polygon([(13, 8), (15, 2), (17, 6), (19, 8)], fill=(226, 218, 196, 255))
        d.polygon([(17, 6), (21, 9), (19, 12), (16, 9)],
                  fill=(206, 198, 174, 255))
        return


# ------------------------------------------------------------------ plates

def draw_plate(d, rng, sid):
    if sid == "alloy_sheet":
        d.polygon([(6, 9), (26, 7), (26, 23), (6, 25)], fill=(168, 178, 196, 255))
        d.polygon([(6, 9), (26, 7), (26, 23), (6, 25)], outline=INK)
        d.line([8, 12, 24, 10], fill=(206, 216, 232, 255), width=1)
        d.line([8, 21, 24, 19], fill=(140, 150, 172, 255), width=1)
        for px, py in ((8, 10), (24, 8), (8, 23), (24, 21)):
            d.point((px, py), fill=STEEL_DARK)
    elif sid == "chitin_plate":
        d.polygon([(7, 8), (25, 8), (27, 16), (24, 25), (9, 25), (5, 16)],
                  fill=(74, 54, 40, 255))
        d.polygon([(7, 8), (25, 8), (27, 16), (24, 25), (9, 25), (5, 16)],
                  outline=INK)
        for ry in (12, 16, 20):  # segmented shell ridges
            d.arc([6, ry - 5, 26, ry + 5], start=20, end=160,
                  fill=(104, 78, 56, 255), width=1)
        d.line([9, 10, 9, 23], fill=(52, 38, 28, 255), width=1)
    elif sid == "counterweight":
        d.polygon([(8, 11), (24, 11), (26, 14), (26, 25), (6, 25),
                   (6, 14)], fill=(70, 72, 78, 255))
        d.polygon([(8, 11), (24, 11), (26, 14), (26, 25), (6, 25),
                   (6, 14)], outline=INK)
        d.rectangle([13, 7, 19, 11], outline=STEEL_DARK, width=2)  # handle
        d.line([8, 15, 24, 15], fill=(96, 98, 106, 255), width=1)
        for _ in range(5):  # cast-iron pitting
            d.point((rng.randint(8, 24), rng.randint(17, 23)),
                    fill=(52, 54, 60, 255))
    elif sid == "wood_plate":
        for i, px in enumerate((6, 12, 18)):  # bundle of three planks
            d.rectangle([px, 7 + (i % 2), px + 7, 25], fill=WOOD)
            d.rectangle([px, 7 + (i % 2), px + 7, 25], outline=WOOD_DARK)
            for gy in range(10, 24, 4):
                d.line([px + 1, gy, px + 6, gy + 1],
                       fill=shade(WOOD, 0.85), width=1)
            d.point((px + 3, 9 + (i % 2)), fill=STEEL_DARK)
            d.point((px + 3, 23), fill=STEEL_DARK)
        d.line([5, 16, 27, 16], fill=(60, 60, 66, 255), width=1)  # strap


# -------------------------------------------------------------- belt links

def draw_link(d, rng, sid):
    # (round width, round height, nose kind) per caliber
    spec = {"ammolink223": (5, 17, "fmj"),
            "ammolink308": (6, 20, "fmj"),
            "ammolink40mm": (9, 13, "grenade"),
            "ammolink50": (7, 23, "fmj")}[sid]
    w, h, kind = spec
    base = 27
    top = base - h
    for i, cx in enumerate((9, 16, 23)):
        x0, x1 = cx - w // 2, cx + w // 2 - 1
        if kind == "grenade":
            d.rectangle([x0, top + 4, x1, base], fill=BRASS_DULL)
            d.ellipse([x0, top, x1, top + 7], fill=(104, 110, 96, 255))
            d.ellipse([x0, top, x1, top + 7], outline=INK)
        else:
            d.rectangle([x0, top + 5, x1, base], fill=BRASS)
            d.polygon([(x0, top + 5), (x1, top + 5), (x1 - 1, top),
                       (x0 + 1, top)], fill=COPPER)
        if i < 2:  # steel link clip joining to the next round
            d.rectangle([x1 + 1, top + h // 2, x1 + 3, top + h // 2 + 3],
                        fill=STEEL_DARK)
            d.line([x1 + 1, top + h // 2 + 1, x1 + 3, top + h // 2 + 1],
                   fill=STEEL, width=1)


# ------------------------------------------------------------------- books

def draw_book(d, rng, sid):
    if sid == "robofac_yrax_trifacet_deactivation_manual":
        # heavy ring binder
        d.rectangle([8, 4, 26, 28], fill=(58, 62, 74, 255))
        d.rectangle([8, 4, 26, 28], outline=INK)
        d.rectangle([8, 4, 11, 28], fill=(44, 48, 58, 255))
        for ry in (8, 16, 24):  # binder rings
            d.ellipse([7, ry - 2, 12, ry + 2], outline=STEEL, width=1)
        # warning triangle + trifacet gem
        d.polygon([(19, 9), (24, 17), (14, 17)], outline=(250, 190, 80, 255))
        d.polygon([(19, 12), (21, 16), (17, 16)], fill=(120, 220, 200, 255))
        d.line([12, 21, 25, 21], fill=STEEL, width=1)
        d.line([12, 24, 22, 24], fill=STEEL_DARK, width=1)
        return
    covers = {"arduino_experiments": (52, 122, 62, 255),
              "exodii_wiring_kit_reference": (226, 222, 208, 255),
              "ic_reference_electronics": (66, 84, 120, 255)}
    cover = covers[sid]
    d.rectangle([9, 5, 25, 27], fill=cover)
    d.rectangle([9, 5, 25, 27], outline=INK)
    d.rectangle([9, 5, 11, 27], fill=shade(cover, 0.78))
    if sid == "arduino_experiments":
        # PCB traces and pads on the cover
        for pts in ([(14, 10), (20, 10), (20, 14)],
                    [(14, 22), (18, 22), (18, 18)],
                    [(22, 20), (22, 24)]):
            d.line(pts, fill=(180, 230, 170, 255), width=1)
        for px, py in ((14, 10), (20, 14), (14, 22), (18, 18), (22, 24)):
            d.point((px, py), fill=(240, 220, 130, 255))
        d.rectangle([15, 12, 19, 16], fill=INK)  # chip
    elif sid == "exodii_wiring_kit_reference":
        # alien glyph marks + a wire curve
        for gx, gy in ((14, 8), (18, 8), (21, 9), (15, 12), (20, 13)):
            d.line([gx, gy, gx + 1, gy + 2], fill=(128, 84, 178, 255),
                   width=1)
            d.point((gx + 2, gy + 1), fill=(44, 140, 132, 255))
        pts = [(13, 24), (16, 18), (19, 22), (22, 17)]
        d.line(pts, fill=(196, 90, 60, 255), width=1)
        d.line([(13, 25), (22, 25)], fill=(110, 110, 116, 255), width=1)
    else:  # ic_reference_electronics: pinout diagram
        d.rectangle([15, 9, 21, 19], fill=INK)
        for py in range(10, 19, 2):
            d.line([13, py, 15, py], fill=STEEL, width=1)
            d.line([21, py, 23, py], fill=STEEL, width=1)
        d.point((17, 11), fill=(240, 236, 226, 255))
        d.line([13, 23, 23, 23], fill=(190, 200, 220, 255), width=1)
        d.line([13, 25, 20, 25], fill=(150, 162, 190, 255), width=1)


# ------------------------------------------------------------ billets/traps

def draw_tool(d, rng, sid):
    if sid in ("billet_bone", "billet_wood", "drill_rock_primitive",
               "punch_bone"):
        if sid == "billet_bone":
            # thick short bone lying across, knobby ends
            d.rectangle([8, 13, 24, 18], fill=BONE)
            for ex in (7, 25):
                d.ellipse([ex - 3, 11, ex + 3, 16], fill=BONE)
                d.ellipse([ex - 3, 15, ex + 3, 20], fill=BONE)
            d.rectangle([8, 13, 24, 18], outline=BONE_DARK)
            d.line([10, 16, 22, 16], fill=BONE_DARK, width=1)
        elif sid == "billet_wood":
            d.rectangle([7, 13, 25, 19], fill=WOOD)
            d.rectangle([7, 13, 25, 19], outline=WOOD_DARK)
            d.ellipse([23, 13, 27, 19], fill=WOOD_LIGHT)
            d.ellipse([23, 13, 27, 19], outline=WOOD_DARK)
            for gy in (15, 17):
                d.line([8, gy, 23, gy], fill=shade(WOOD, 0.85), width=1)
        elif sid == "drill_rock_primitive":
            # thick blunted stick, worn pale tip down
            d.polygon([(13, 4), (19, 4), (18, 20), (16, 27), (14, 20)],
                      fill=WOOD)
            d.polygon([(13, 4), (19, 4), (18, 20), (16, 27), (14, 20)],
                      outline=WOOD_DARK)
            d.polygon([(14, 20), (18, 20), (16, 27)], fill=WOOD_LIGHT)
            d.line([16, 6, 16, 18], fill=shade(WOOD, 0.85), width=1)
        else:  # punch_bone: bone tapering to a pointy tip
            d.polygon([(12, 5), (20, 5), (19, 16), (16, 27), (13, 16)],
                      fill=BONE)
            d.polygon([(12, 5), (20, 5), (19, 16), (16, 27), (13, 16)],
                      outline=BONE_DARK)
            d.ellipse([10, 3, 15, 8], fill=BONE)
            d.ellipse([17, 3, 22, 8], fill=BONE)
            d.line([16, 18, 16, 25], fill=BONE_DARK, width=1)
        return
    if sid == "boobytrap":
        # crude charge, pull-ring detonator, string running off to a stake
        d.rectangle([10, 18, 22, 26], fill=(120, 110, 84, 255))
        d.rectangle([10, 18, 22, 26], outline=INK)
        d.rectangle([14, 14, 18, 18], fill=STEEL_DARK)
        d.ellipse([14, 11, 18, 15], outline=STEEL, width=1)
        d.line([16, 12, 27, 6], fill=(226, 222, 208, 255), width=1)
        d.line([27, 6, 27, 12], fill=WOOD_DARK, width=2)
        d.line([11, 21, 21, 21], fill=RED, width=1)
        return
    if sid == "caltrops_teeth":
        # cluster of impossibly sharp little teeth glued together
        teeth = [(9, 22), (14, 19), (19, 22), (12, 25), (17, 25),
                 (22, 20), (15, 22)]
        for tx, ty in teeth:
            d.polygon([(tx - 2, ty + 4), (tx + 2, ty + 4), (tx, ty)],
                      fill=(240, 238, 230, 255))
            d.polygon([(tx - 2, ty + 4), (tx + 2, ty + 4), (tx, ty)],
                      outline=BONE_DARK)
            d.line([tx - 1, ty + 4, tx + 1, ty + 4], fill=(214, 190, 150, 255),
                   width=1)  # glue line at the root
        return
    if sid == "crossbow_trap":
        # small crossbow in profile, bolt loaded, tripwire from trigger
        d.line([6, 10, 26, 10], fill=WOOD_DARK, width=2)  # prod (bow)
        d.arc([4, 4, 28, 16], start=200, end=340, fill=STEEL_DARK, width=1)
        d.rectangle([14, 9, 18, 26], fill=WOOD)  # stock
        d.rectangle([14, 9, 18, 26], outline=WOOD_DARK)
        d.line([6, 10, 16, 13], fill=(226, 222, 208, 255), width=1)
        d.line([26, 10, 16, 13], fill=(226, 222, 208, 255), width=1)
        d.line([16, 6, 16, 12], fill=STEEL, width=1)  # bolt
        d.polygon([(15, 3), (17, 3), (16, 6)], fill=STEEL_DARK)
        d.line([16, 20, 25, 27], fill=(226, 222, 208, 255), width=1)
        d.point((25, 27), fill=WOOD_DARK)
        return
    if sid == "light_snare_kit":
        # coiled string loop, two anchor stakes, small hammer
        d.ellipse([8, 6, 22, 20], outline=(226, 222, 208, 255), width=2)
        d.ellipse([11, 9, 19, 17], outline=(206, 202, 188, 255), width=1)
        d.line([20, 18, 24, 23], fill=(226, 222, 208, 255), width=1)
        for sx in (7, 12):  # anchor stakes
            d.polygon([(sx, 22), (sx + 2, 22), (sx + 1, 29)], fill=WOOD_DARK)
        d.rectangle([19, 22, 26, 25], fill=STEEL_DARK)  # hammer head
        d.line([22, 25, 25, 29], fill=WOOD, width=2)
        return


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in AMMO:
        profile, bullet, finish = AMMO[sid]
        draw_cartridge(d, rng, profile, bullet, finish)
    elif sid in ("22_ar15mag_10", "22_ar15mag_28", "22_speedloader8",
                 "mp1522_mag"):
        draw_mag(d, rng, sid)
    elif sid in ("60mm_propellant", "c4", "c4armed", "gelled_molotov"):
        draw_explosive(d, rng, sid)
    elif sid in ("alloy_sheet", "chitin_plate", "counterweight",
                 "wood_plate"):
        draw_plate(d, rng, sid)
    elif sid in ("ammolink223", "ammolink308", "ammolink40mm",
                 "ammolink50"):
        draw_link(d, rng, sid)
    elif sid in ("arduino_experiments", "exodii_wiring_kit_reference",
                 "ic_reference_electronics",
                 "robofac_yrax_trifacet_deactivation_manual"):
        draw_book(d, rng, sid)
    elif sid in ("billet_bone", "billet_wood", "drill_rock_primitive",
                 "punch_bone", "boobytrap", "caltrops_teeth",
                 "crossbow_trap", "light_snare_kit"):
        draw_tool(d, rng, sid)
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
                       "preview_item84.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item84.png {prev.size}")


if __name__ == "__main__":
    main()
