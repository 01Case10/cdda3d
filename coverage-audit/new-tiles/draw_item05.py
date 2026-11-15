#!/usr/bin/env python3
"""Batch ITEM05 (Phase 2g.4 scope extension): generic items, second set.

52 ids from data/json/items/generic.json (authoritative list:
/tmp/item_batches/05.txt). Each sprite drawn from its entry's
name/description/material/color. Items are objects on transparency,
native 32x32, flat UltiCa style, bodies fully opaque (alpha {0,255};
glass/water faked with pale opaque colors).

Notes on odd entries:
- fire ("nearby fire"), metabolism, muscle are pseudo items whose
  description literally reads "seeing this is a bug" -- fire drawn
  as the flame it represents; metabolism/muscle drawn as debug
  tiles per the project precedent (dark tile, "?" + BUG mark).
- diamond/metabolism/muscle/fire also exist as material/scenario
  ids; the ITEM entries in generic.json are the ones drawn.

PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item05.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")
IDS = [l.strip() for l in open("/tmp/item_batches/05.txt") if l.strip()]

# CDDA-ish palette
WHITE = (232, 232, 228, 255)
PAPER = (238, 234, 220, 255)
LGRAY = (172, 172, 176, 255)
MGRAY = (118, 118, 124, 255)
DGRAY = (74, 74, 80, 255)
STEEL = (150, 156, 164, 255)
STEEL_D = (96, 102, 110, 255)
GOLD = (226, 182, 44, 255)
GOLD_D = (168, 128, 22, 255)
YELLOW = (238, 205, 60, 255)
GREEN = (66, 160, 74, 255)
GREEN_D = (36, 104, 46, 255)
RED = (196, 52, 40, 255)
RED_D = (140, 32, 26, 255)
BROWN = (138, 90, 43, 255)
BROWN_D = (96, 60, 26, 255)
TAN = (198, 164, 110, 255)
CYAN_P = (168, 226, 226, 255)
BLUE = (52, 84, 190, 255)
ORANGE = (238, 128, 32, 255)
PINK = (226, 150, 170, 255)
BLACK = (26, 26, 30, 255)


def new():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def check(img, tid):
    a = set(img.getchannel("A").getdata())
    assert a <= {0, 255}, (tid, a)


def coil_wire(d, cx, cy, r, color, dark, rng, turns=4):
    for t in range(turns):
        y = cy - r + t * (2 * r / max(turns - 1, 1))
        d.arc([cx - r, y - 3, cx + r, y + 3], 0, 180, fill=dark, width=2)
        d.arc([cx - r, y - 3, cx + r, y + 3], 180, 360, fill=color, width=2)
    d.line([(cx - r, cy - r), (cx - r - 3, cy - r - 4)], fill=dark, width=2)
    d.line([(cx + r, cy + r), (cx + r + 3, cy + r + 4)], fill=dark, width=2)


def gem(d, cx, cy, r, base, light, dark):
    # faceted gem: crown trapezoid + pavilion triangle
    d.polygon([(cx - r, cy - 2), (cx + r, cy - 2), (cx + r - 3, cy - r + 2),
               (cx - r + 3, cy - r + 2)], fill=light)
    d.polygon([(cx - r, cy - 2), (cx + r, cy - 2), (cx, cy + r + 2)], fill=base)
    d.polygon([(cx - r, cy - 2), (cx, cy - 2), (cx, cy + r + 2)], fill=dark)
    d.line([(cx - r + 3, cy - r + 2), (cx, cy - 2), (cx + r - 3, cy - r + 2)],
           fill=(255, 255, 255, 255))
    d.line([(cx - r, cy - 2), (cx + r, cy - 2)], fill=(255, 255, 255, 255), width=1)


def sparkle(d, x, y, s, color=(255, 255, 255, 255)):
    d.line([(x - s, y), (x + s, y)], fill=color)
    d.line([(x, y - s), (x, y + s)], fill=color)
    d.point((x, y), fill=color)


def paper_sheet(d, x0, y0, x1, y1, rng, accent, lines=5, stain=None):
    d.rectangle([x0, y0, x1, y1], fill=PAPER, outline=MGRAY)
    d.rectangle([x0 + 2, y0 + 2, x0 + 7, y0 + 5], fill=accent)
    for i in range(lines):
        y = y0 + 8 + i * 3
        if y > y1 - 2:
            break
        w = (x1 - x0 - 5) - rng.randrange(0, 6)
        d.line([(x0 + 3, y), (x0 + 3 + w, y)], fill=MGRAY)
    if stain:
        d.ellipse([x1 - 8, y1 - 7, x1 - 2, y1 - 2], fill=stain)


def usb_drive(d, cx, cy, body, label, rng, angle=0):
    d.rectangle([cx - 4, cy - 11, cx + 4, cy - 5], fill=STEEL, outline=STEEL_D)
    d.rectangle([cx - 2, cy - 10, cx - 1, cy - 7], fill=DGRAY)
    d.rectangle([cx + 1, cy - 10, cx + 2, cy - 7], fill=DGRAY)
    d.rounded_rectangle([cx - 6, cy - 5, cx + 6, cy + 11], 3, fill=body,
                        outline=DGRAY)
    d.rectangle([cx - 4, cy + 1, cx + 4, cy + 6], fill=label)
    d.point((cx, cy - 2), fill=RED)


def draw(tid, rng):
    img, d = new()

    if tid == "ch_wire":
        coil_wire(d, 16, 15, 9, STEEL_D, DGRAY, rng)
        d.line([(6, 26), (11, 21)], fill=STEEL_D, width=2)
    elif tid == "mc_wire":
        coil_wire(d, 16, 15, 9, STEEL, MGRAY, rng)
        d.line([(6, 26), (11, 21)], fill=STEEL, width=2)
    elif tid == "chalice":
        d.polygon([(9, 5), (23, 5), (21, 13), (10, 13)], fill=GOLD,
                  outline=GOLD_D)
        d.ellipse([9, 3, 23, 7], fill=(246, 214, 92, 255), outline=GOLD_D)
        d.rectangle([15, 13, 17, 21], fill=GOLD, outline=GOLD_D)
        d.polygon([(10, 25), (22, 25), (20, 21), (12, 21)], fill=GOLD,
                  outline=GOLD_D)
        d.arc([8, 6, 13, 14], 90, 270, fill=GOLD_D, width=2)
        d.arc([19, 6, 24, 14], 270, 90, fill=GOLD_D, width=2)
        sparkle(d, 13, 8, 2)
    elif tid == "chamomile":
        for (x, y) in [(10, 12), (16, 9), (22, 12), (13, 19), (20, 19),
                       (16, 24)]:
            for a in range(6):
                ang = a * math.pi / 3
                d.ellipse([x + 3 * math.cos(ang) - 1, y + 3 * math.sin(ang) - 1,
                           x + 3 * math.cos(ang) + 2, y + 3 * math.sin(ang) + 2],
                          fill=WHITE)
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=YELLOW)
        d.line([(16, 14), (16, 28)], fill=GREEN_D, width=2)
        d.line([(16, 22), (11, 18)], fill=GREEN_D, width=1)
        d.line([(16, 22), (21, 18)], fill=GREEN_D, width=1)
    elif tid == "circuit":
        d.rectangle([5, 8, 27, 24], fill=(28, 110, 52, 255), outline=GREEN_D)
        for (x, y, w, h) in [(8, 11, 5, 4), (18, 11, 6, 5), (10, 18, 7, 4),
                             (21, 19, 4, 3)]:
            d.rectangle([x, y, x + w, y + h], fill=BLACK)
            for px in range(x, x + w + 1, 2):
                d.point((px, y - 1), fill=LGRAY)
                d.point((px, y + h + 1), fill=LGRAY)
        d.line([(7, 21), (13, 21), (13, 13)], fill=(120, 220, 140, 255))
        d.line([(25, 10), (20, 10), (20, 16)], fill=(120, 220, 140, 255))
        for (x, y) in [(7, 10), (25, 22), (15, 22)]:
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=GOLD)
    elif tid == "citrine":
        gem(d, 16, 15, 9, (216, 164, 32, 255), (250, 220, 110, 255),
            (172, 122, 20, 255))
        sparkle(d, 11, 8, 2)
        sparkle(d, 22, 19, 2)
    elif tid == "emerald":
        gem(d, 16, 15, 9, (28, 158, 74, 255), (110, 226, 140, 255),
            (18, 104, 48, 255))
        sparkle(d, 12, 8, 2)
    elif tid == "garnet":
        gem(d, 16, 15, 9, (168, 30, 44, 255), (238, 110, 120, 255),
            (110, 18, 30, 255))
        sparkle(d, 21, 9, 2)
    elif tid == "malachite":
        gem(d, 16, 15, 9, (30, 138, 84, 255), (96, 206, 140, 255),
            (16, 92, 54, 255))
        d.arc([9, 6, 23, 16], 200, 340, fill=(170, 240, 190, 255))
        d.arc([11, 10, 21, 19], 200, 340, fill=(12, 70, 40, 255))
    elif tid == "diamond":
        gem(d, 16, 15, 9, (206, 226, 238, 255), (255, 255, 255, 255),
            (158, 186, 204, 255))
        sparkle(d, 10, 7, 3)
        sparkle(d, 23, 12, 2)
        sparkle(d, 13, 22, 2)
    elif tid == "clockworks":
        def gear(cx, cy, r, color):
            for a in range(8):
                ang = a * math.pi / 4
                d.rectangle([cx + (r) * math.cos(ang) - 1,
                             cy + (r) * math.sin(ang) - 1,
                             cx + (r) * math.cos(ang) + 1,
                             cy + (r) * math.sin(ang) + 1], fill=color)
            d.ellipse([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1],
                      fill=color)
            d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=DGRAY)
        gear(12, 13, 6, STEEL)
        gear(21, 19, 5, LGRAY)
        gear(20, 9, 3, GOLD)
        d.ellipse([8, 22, 12, 26], outline=STEEL_D)
    elif tid == "condensor_coil":
        d.rectangle([5, 7, 27, 25], fill=DGRAY, outline=BLACK)
        for x in range(7, 26, 3):
            d.line([(x, 9), (x, 23)], fill=MGRAY)
        d.ellipse([10, 10, 22, 22], fill=STEEL_D, outline=LGRAY)
        for a in range(3):
            ang = a * 2 * math.pi / 3 + 0.4
            d.polygon([(16, 16),
                       (16 + 6 * math.cos(ang), 16 + 6 * math.sin(ang)),
                       (16 + 6 * math.cos(ang + 0.9),
                        16 + 6 * math.sin(ang + 0.9))], fill=LGRAY)
        d.ellipse([15, 15, 17, 17], fill=BLACK)
    elif tid == "evaporator_coil":
        for row in range(4):
            y = 8 + row * 5
            if row % 2 == 0:
                d.line([(7, y), (25, y)], fill=STEEL, width=3)
                d.arc([22, y, 28, y + 5], 270, 90, fill=STEEL, width=3)
            else:
                d.line([(7, y), (25, y)], fill=STEEL, width=3)
                d.arc([4, y - 5, 10, y], 90, 270, fill=STEEL, width=3)
        d.line([(25, 23), (25, 27)], fill=STEEL_D, width=3)
    elif tid == "credit_card":
        d.rounded_rectangle([4, 10, 28, 22], 2, fill=YELLOW, outline=GOLD_D)
        d.rectangle([4, 13, 28, 16], fill=BLACK)
        d.line([(6, 19), (14, 19)], fill=GOLD_D)
        d.line([(6, 21), (11, 21)], fill=GOLD_D)
        d.rectangle([20, 18, 26, 21], outline=GOLD_D)
    elif tid == "crude_heating_element":
        for t in range(5):
            x = 8 + t * 4
            d.arc([x, 9, x + 5, 17], 180, 0, fill=MGRAY, width=2)
            d.arc([x, 15, x + 5, 23], 0, 180, fill=MGRAY, width=2)
        d.line([(8, 13), (5, 13)], fill=STEEL_D, width=2)
        d.line([(26, 19), (29, 19)], fill=STEEL_D, width=2)
        d.point((12, 10), fill=DGRAY)
        d.point((20, 22), fill=DGRAY)
    elif tid == "element":
        d.rectangle([6, 12, 26, 20], fill=STEEL, outline=STEEL_D)
        for t in range(4):
            x = 9 + t * 4
            d.arc([x, 13, x + 4, 19], 180, 0, fill=ORANGE, width=1)
            d.arc([x, 13, x + 4, 19], 0, 180, fill=RED, width=1)
        d.rectangle([4, 14, 6, 18], fill=DGRAY)
        d.rectangle([26, 14, 28, 18], fill=DGRAY)
    elif tid == "debug_template":
        d.ellipse([7, 7, 25, 25], fill=GOLD, outline=GOLD_D)
        d.ellipse([10, 10, 22, 22], fill=(246, 214, 92, 255))
        d.ellipse([14, 14, 18, 18], fill=DGRAY)
        d.polygon([(16, 7), (25, 16), (16, 16)], fill=(255, 240, 170, 255))
        d.rectangle([13, 26, 19, 28], fill=DGRAY)
        sparkle(d, 11, 11, 2)
    elif tid == "delay_fuze":
        d.rectangle([12, 8, 20, 24], fill=STEEL, outline=STEEL_D)
        d.ellipse([11, 4, 21, 10], fill=LGRAY, outline=STEEL_D)
        for a in range(8):
            ang = a * math.pi / 4
            d.point((16 + 4 * math.cos(ang), 7 + 2 * math.sin(ang)), fill=RED)
        d.rectangle([14, 14, 18, 18], fill=DGRAY)
        d.point((16, 16), fill=YELLOW)
        d.line([(13, 24), (13, 28)], fill=STEEL_D, width=2)
        d.line([(19, 24), (19, 28)], fill=STEEL_D, width=2)
    elif tid == "impact_fuze":
        d.polygon([(13, 14), (19, 14), (17, 24), (15, 24)], fill=STEEL,
                  outline=STEEL_D)
        d.polygon([(16, 3), (18, 9), (14, 9)], fill=RED, outline=RED_D)
        d.rectangle([13, 9, 19, 14], fill=LGRAY, outline=STEEL_D)
        d.line([(11, 6), (8, 3)], fill=MGRAY, width=1)
        d.line([(21, 6), (24, 3)], fill=MGRAY, width=1)
        d.line([(14, 26), (18, 26)], fill=STEEL_D, width=2)
    elif tid == "disassembly":
        d.rectangle([6, 10, 16, 20], fill=WHITE, outline=MGRAY)
        d.line([(8, 13), (14, 13)], fill=CYAN_P)
        d.rectangle([19, 8, 24, 13], fill=DGRAY)
        d.ellipse([20, 17, 25, 22], outline=STEEL_D)
        d.rectangle([9, 23, 13, 26], fill=GOLD)
        for (x, y) in [(27, 16), (18, 25), (27, 24)]:
            d.point((x, y), fill=STEEL_D)
            d.point((x + 1, y), fill=STEEL_D)
        d.line([(16, 15), (20, 12)], fill=MGRAY)
        d.line([(14, 20), (21, 20)], fill=MGRAY)
    elif tid == "dog_tag_dog":
        d.rounded_rectangle([9, 10, 23, 22], 4, fill=GREEN, outline=GREEN_D)
        d.ellipse([11, 12, 14, 15], fill=WHITE)
        d.line([(17, 14), (21, 14)], fill=WHITE)
        d.line([(15, 17), (21, 17)], fill=WHITE)
        d.line([(15, 20), (19, 20)], fill=WHITE)
        d.arc([6, 4, 14, 12], 180, 360, fill=STEEL, width=2)
    elif tid == "drum_skin":
        d.ellipse([6, 8, 26, 24], fill=TAN, outline=BROWN_D)
        d.ellipse([9, 10, 23, 22], fill=(214, 184, 130, 255))
        for a in range(8):
            ang = a * math.pi / 4
            x, y = 16 + 10 * math.cos(ang), 16 + 8 * math.sin(ang)
            d.line([(x, y), (16 + 7 * math.cos(ang), 16 + 5 * math.sin(ang))],
                   fill=BROWN_D)
    elif tid == "egg_void_spider":
        d.ellipse([9, 6, 23, 26], fill=(206, 232, 232, 255),
                  outline=(140, 190, 196, 255))
        d.polygon([(12, 9), (16, 5), (20, 9)], fill=(226, 244, 244, 255))
        for (x, y, c) in [(13, 14, (120, 240, 220, 255)),
                          (18, 18, (240, 200, 120, 255)),
                          (15, 21, (200, 140, 240, 255)),
                          (19, 12, (140, 200, 250, 255))]:
            sparkle(d, x, y, 2, c)
    elif tid == "exodii_chop_shop_head":
        d.polygon([(9, 6), (23, 6), (24, 16), (21, 24), (11, 24), (8, 16)],
                  fill=RED, outline=RED_D)
        d.rectangle([12, 11, 20, 15], fill=BLACK)
        d.point((14, 13), fill=(120, 240, 220, 255))
        d.point((18, 13), fill=(120, 240, 220, 255))
        d.rectangle([13, 18, 19, 21], fill=DGRAY)
        for x in (14, 16, 18):
            d.line([(x, 18), (x, 21)], fill=STEEL)
        d.line([(9, 6), (6, 10)], fill=RED_D, width=2)
        d.rectangle([14, 24, 18, 28], fill=STEEL_D)
        d.point((23, 9), fill=ORANGE)
    elif tid == "fema_data":
        usb_drive(d, 16, 16, BLUE, WHITE, rng)
        d.rectangle([12, 17, 20, 19], fill=BLUE)
        d.line([(13, 18), (19, 18)], fill=WHITE)
    elif tid == "lixa_data":
        usb_drive(d, 16, 16, BLACK, CYAN_P, rng)
        d.rectangle([12, 17, 20, 19], fill=BLACK)
        d.line([(13, 18), (19, 18)], fill=CYAN_P)
    elif tid == "nano_fabricator_encryption_code":
        usb_drive(d, 16, 16, GOLD, BLACK, rng)
        d.point((14, 19), fill=GOLD)
        d.point((18, 19), fill=GOLD)
    elif tid == "fertilizer":
        d.ellipse([8, 8, 24, 24], fill=GREEN, outline=GREEN_D)
        d.ellipse([10, 10, 22, 22], outline=(150, 230, 150, 255))
        d.line([(16, 20), (16, 13)], fill=WHITE, width=2)
        d.polygon([(16, 13), (11, 10), (12, 15)], fill=WHITE)
        d.polygon([(16, 13), (21, 10), (20, 15)], fill=WHITE)
    elif tid == "fire":
        def flame(cx, base, h, w, c_out, c_in):
            d.polygon([(cx - w, base), (cx, base - h), (cx + w, base),
                       (cx + w // 2, base), (cx - w // 2, base)], fill=c_out)
            d.polygon([(cx - w // 2, base), (cx, base - h // 2 - 2),
                       (cx + w // 2, base)], fill=c_in)
        flame(16, 26, 20, 8, ORANGE, YELLOW)
        flame(10, 26, 12, 5, RED, ORANGE)
        flame(23, 26, 14, 5, RED, ORANGE)
        d.rectangle([7, 26, 25, 28], fill=BROWN_D)
    elif tid in ("metabolism", "muscle"):
        d.rectangle([4, 4, 28, 28], fill=BLACK, outline=RED)
        # pixel "?"
        d.rectangle([13, 9, 19, 11], fill=WHITE)
        d.rectangle([17, 11, 19, 14], fill=WHITE)
        d.rectangle([13, 14, 17, 16], fill=WHITE)
        d.rectangle([15, 16, 17, 18], fill=WHITE)
        d.rectangle([15, 21, 17, 23], fill=WHITE)
        for i, x in enumerate((10, 14, 18)):
            d.rectangle([x, 25, x + 3, 27], fill=RED)
    elif tid == "fire_brick":
        d.rectangle([5, 11, 27, 21], fill=(164, 84, 52, 255),
                    outline=BROWN_D)
        d.line([(5, 16), (27, 16)], fill=BROWN_D)
        d.line([(16, 11), (16, 16)], fill=BROWN_D)
        d.line([(10, 16), (10, 21)], fill=BROWN_D)
        d.line([(22, 16), (22, 21)], fill=BROWN_D)
        for _ in range(8):
            d.point((rng.randrange(7, 26), rng.randrange(12, 21)),
                    fill=(190, 110, 70, 255))
    elif tid == "fish_bowl":
        d.ellipse([7, 8, 25, 26], fill=CYAN_P, outline=(120, 180, 190, 255))
        d.rectangle([11, 5, 21, 8], fill=(190, 235, 235, 255),
                    outline=(120, 180, 190, 255))
        d.ellipse([8, 15, 24, 25], fill=(110, 190, 220, 255))
        d.polygon([(12, 19), (18, 19), (21, 21), (18, 22), (12, 22),
                   (10, 20)], fill=ORANGE)
        d.polygon([(18, 19), (22, 17), (22, 22)], fill=(220, 100, 40, 255))
        d.point((13, 20), fill=BLACK)
        d.line([(14, 19), (13, 16)], fill=BROWN_D)
        d.line([(16, 19), (17, 16)], fill=BROWN_D)
        d.point((13, 16), fill=BROWN_D)
        d.point((17, 16), fill=BROWN_D)
        d.line([(9, 25), (23, 25)], fill=TAN)
    elif tid == "godcotrader_accounting_records_clean":
        paper_sheet(d, 9, 5, 24, 27, rng, GREEN, lines=6)
        paper_sheet(d, 11, 7, 26, 29, rng, GREEN, lines=5)
        d.line([(13, 12), (24, 12)], fill=GREEN)
    elif tid == "godcotrader_accounting_records_dirty":
        paper_sheet(d, 8, 6, 23, 28, rng, MGRAY, lines=6,
                    stain=(170, 140, 90, 255))
        d.line([(11, 14), (20, 20)], fill=RED, width=1)
        d.line([(20, 14), (11, 20)], fill=RED, width=1)
        d.ellipse([17, 8, 22, 11], fill=(170, 140, 90, 255))
    elif tid == "helmet_mounting_brackets":
        for (x, y) in [(8, 9), (18, 9), (8, 19), (18, 19)]:
            d.polygon([(x, y + 6), (x, y), (x + 6, y), (x + 6, y + 3),
                       (x + 3, y + 3), (x + 3, y + 6)], fill=DGRAY,
                      outline=BLACK)
            d.point((x + 1, y + 1), fill=LGRAY)
        d.line([(14, 12), (18, 12)], fill=MGRAY)
        d.line([(14, 22), (18, 22)], fill=MGRAY)
    elif tid == "household_dishwasher":
        d.rectangle([7, 6, 25, 27], fill=WHITE, outline=MGRAY)
        d.rectangle([7, 6, 25, 10], fill=LGRAY, outline=MGRAY)
        d.point((10, 8), fill=GREEN)
        d.point((13, 8), fill=RED)
        d.rectangle([17, 7, 23, 9], fill=DGRAY)
        d.line([(9, 14), (23, 14)], fill=MGRAY, width=2)
    elif tid == "household_washing_machine":
        d.rectangle([7, 6, 25, 27], fill=WHITE, outline=MGRAY)
        d.rectangle([7, 6, 25, 10], fill=LGRAY, outline=MGRAY)
        d.point((10, 8), fill=BLUE)
        d.ellipse([10, 12, 22, 25], fill=DGRAY, outline=MGRAY)
        d.ellipse([12, 14, 20, 23], fill=(110, 190, 220, 255))
        d.arc([12, 14, 20, 23], 200, 320, fill=WHITE, width=2)
    elif tid in ("household_water_heater", "household_water_heater_family"):
        big = tid.endswith("family")
        x0, x1 = (6, 26) if big else (9, 23)
        d.rectangle([x0, 5, x1, 26], fill=WHITE, outline=MGRAY)
        d.ellipse([x0, 3, x1, 7], fill=LGRAY, outline=MGRAY)
        d.rectangle([x0 + 3, 10, x0 + 8, 14], fill=STEEL_D)
        d.point((x0 + 5, 12), fill=RED)
        cx = x1 - 4
        d.line([(cx, 16), (cx, 20)], fill=STEEL_D, width=2)
        d.polygon([(cx - 3, 20), (cx + 3, 20), (cx, 24)], fill=STEEL_D)
        d.line([(x0 + 2, 26), (x0 + 2, 28)], fill=MGRAY, width=2)
        d.line([(x1 - 2, 26), (x1 - 2, 28)], fill=MGRAY, width=2)
    elif tid == "large_lcd_screen":
        d.rectangle([4, 7, 28, 23], fill=DGRAY, outline=BLACK)
        d.rectangle([6, 9, 26, 21], fill=(30, 40, 52, 255))
        d.polygon([(8, 21), (16, 9), (20, 9), (12, 21)],
                  fill=(70, 90, 110, 255))
        d.rectangle([14, 23, 18, 26], fill=DGRAY)
        d.line([(10, 27), (22, 27)], fill=DGRAY, width=2)
    elif tid == "large_printout_portal_storm_data":
        d.rectangle([10, 4, 22, 10], fill=PAPER, outline=MGRAY)
        d.ellipse([12, 5, 20, 9], fill=(210, 200, 180, 255), outline=MGRAY)
        d.polygon([(10, 10), (22, 10), (24, 27), (8, 27)], fill=PAPER,
                  outline=MGRAY)
        for i in range(6):
            y = 13 + i * 2
            d.line([(11, y), (21, y)], fill=(150, 90, 60, 255))
        d.rectangle([14, 18, 19, 23], fill=(150, 90, 60, 255))
    elif tid == "medium_printout_portal_storm_data":
        d.rectangle([11, 6, 21, 11], fill=PAPER, outline=MGRAY)
        d.ellipse([13, 7, 19, 10], fill=(210, 200, 180, 255), outline=MGRAY)
        d.polygon([(11, 11), (21, 11), (22, 24), (10, 24)], fill=PAPER,
                  outline=MGRAY)
        for i in range(4):
            y = 14 + i * 2
            d.line([(12, y), (20, y)], fill=(150, 90, 60, 255))
        d.rectangle([14, 19, 18, 22], fill=(150, 90, 60, 255))
    elif tid == "lens_small":
        d.ellipse([10, 10, 22, 22], fill=CYAN_P, outline=(110, 170, 180, 255))
        d.ellipse([12, 12, 20, 20], fill=(200, 242, 242, 255))
        d.arc([11, 11, 21, 21], 180, 290, fill=WHITE, width=2)
        sparkle(d, 24, 9, 2, (160, 220, 220, 255))
    elif tid == "log_immersion27":
        paper_sheet(d, 9, 5, 24, 27, rng, DGRAY, lines=6)
        d.rectangle([10, 6, 14, 9], outline=RED)
        d.line([(18, 6), (23, 6)], fill=RED)
        d.rectangle([17, 5, 24, 8], outline=RED)
    elif tid == "melchior_science_doc":
        paper_sheet(d, 9, 5, 24, 27, rng, BLUE, lines=6)
        d.line([(11, 10), (22, 10)], fill=BLUE)
        d.ellipse([18, 20, 23, 25], outline=BLUE)
        d.line([(19, 23), (21, 21)], fill=BLUE)
    elif tid == "monitoring_equipment_doc":
        paper_sheet(d, 9, 5, 24, 27, rng, DGRAY, lines=6)
        d.rectangle([11, 18, 16, 23], outline=DGRAY)
        d.line([(12, 21), (13, 19), (14, 22), (15, 20)], fill=GREEN_D)
    elif tid == "msu14_password":
        d.polygon([(10, 8), (24, 10), (23, 24), (9, 22)], fill=PAPER,
                  outline=MGRAY)
        for i in range(3):
            y = 12 + i * 3
            d.line([(12, y), (21, y)], fill=DGRAY)
        d.line([(12, 21), (16, 21)], fill=RED)
    elif tid == "makeshift_pillow":
        d.rounded_rectangle([7, 10, 25, 22], 6, fill=BLUE,
                            outline=(34, 56, 140, 255))
        d.arc([7, 10, 25, 22], 90, 270, fill=(34, 56, 140, 255), width=2)
        d.arc([7, 10, 25, 22], 270, 90, fill=(120, 150, 230, 255), width=1)
        d.line([(9, 13), (9, 19)], fill=(34, 56, 140, 255))
        d.line([(23, 13), (23, 19)], fill=(34, 56, 140, 255))
        for (x, y) in [(12, 12), (19, 20)]:
            d.point((x, y), fill=(230, 200, 120, 255))
    elif tid == "makeshift_tree_spile":
        d.polygon([(10, 12), (24, 9), (25, 13), (12, 17)], fill=TAN,
                  outline=BROWN_D)
        d.polygon([(10, 12), (12, 17), (8, 16), (7, 12)], fill=BROWN,
                  outline=BROWN_D)
        d.line([(13, 13), (23, 11)], fill=BROWN_D)
        d.polygon([(21, 17), (23, 17), (22, 21)], fill=(240, 210, 110, 255))
        d.ellipse([24, 20, 27, 24], fill=(110, 190, 220, 255))
    elif tid == "module_meteorologist":
        d.rectangle([9, 9, 23, 23], fill=DGRAY, outline=BLACK)
        d.rectangle([12, 12, 20, 17], fill=(28, 110, 52, 255))
        for x in range(10, 23, 3):
            d.line([(x, 23), (x, 26)], fill=GOLD)
            d.line([(x, 9), (x, 6)], fill=GOLD)
        d.line([(23, 12), (27, 7)], fill=MGRAY, width=1)
        d.point((27, 7), fill=RED)
        d.point((13, 20), fill=(120, 240, 220, 255))
    elif tid == "motor_micro":
        d.rectangle([10, 11, 20, 21], fill=STEEL, outline=STEEL_D)
        for x in (13, 16, 19):
            d.line([(x, 11), (x, 21)], fill=STEEL_D)
        d.rectangle([14, 7, 16, 11], fill=DGRAY)
        d.line([(20, 14), (26, 10)], fill=RED, width=1)
        d.line([(20, 17), (26, 20)], fill=BLACK, width=1)
        d.ellipse([8, 13, 10, 19], fill=LGRAY, outline=STEEL_D)

    check(img, tid)
    return img


def main():
    for tid in IDS:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = draw(tid, rng)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # contact sheet: 8 cols, 2x scale
    cell, gap, cols = 64, 3, 8
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, tid in enumerate(IDS):
        img = Image.open(os.path.join(BASE, tid, tid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item05.png")
    sheet.save(out)
    print("wrote", len(IDS), "sprites + preview_item05.png", sheet.size)


if __name__ == "__main__":
    main()
