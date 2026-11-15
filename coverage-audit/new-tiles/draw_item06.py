#!/usr/bin/env python3
"""Batch ITEM06 (Phase 2g.4 scope extension): generic.json final set, 37 ids.

Drawn from each entry's name/description/color/symbol in
fork-src/cdda/data/json/items/generic.json. Items are objects on a
transparent background, native 32x32, flat UltiCa style: gems in their
data colors, the nanofabricator glass slates with inscribed patterns,
XEDRA/NRE paper printouts, coils/springs/wire, and the two PSEUDO
items (sunlight, wind -- "seeing this is a bug") drawn as their
natural glyph with a "?" badge.

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is
strictly {0, 255}: translucency is faked with pale opaque colors.
Writes ONLY its own 37 folders + preview_item06.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
BASE = os.path.join(HERE, "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [l.strip() for l in open("/tmp/item_batches/06.txt") if l.strip()]
assert len(IDS) == 37, len(IDS)


def R(c):
    return tuple(c) + (255,)


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c) + (255,)


def new_img():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def gem(rng, base, speckles=()):
    """Faceted hexagonal gem in the given base color."""
    img = new_img()
    d = ImageDraw.Draw(img)
    pts = [(16, 3), (25, 9), (27, 15), (21, 26), (11, 26), (5, 15), (7, 9)]
    d.polygon(pts, fill=R(base))
    # crown facets
    table = [(12, 9), (20, 9), (23, 14), (19, 20), (13, 20), (9, 14)]
    d.polygon(table, fill=shade(base, 1.35))
    for p in pts:
        d.line([p, (16, 14)], fill=shade(base, 0.62), width=1)
    d.line([(12, 9), (16, 14)], fill=shade(base, 1.6), width=1)
    d.line([(16, 3), (16, 9)], fill=shade(base, 1.6), width=1)
    d.point((13, 11), fill=shade(base, 1.8))
    d.point((14, 10), fill=shade(base, 1.8))
    for col in speckles:
        for _ in range(5):
            x = rng.randint(9, 23)
            y = rng.randint(8, 22)
            d.point((x, y), fill=R(col))
    return img


def paper_strip(rng, tall=True, orderly=False):
    """Thermal-paper printout with oxide-ink plotted number waves."""
    img = new_img()
    d = ImageDraw.Draw(img)
    x0, x1 = 10, 22
    y0, y1 = (3, 29) if tall else (8, 24)
    d.rectangle([x0, y0, x1, y1], fill=R((238, 236, 228)))
    d.line([x0, y0, x1, y0], fill=R((200, 198, 188)))
    oxide = (148, 82, 42)
    rows = range(y0 + 3, y1 - 2, 4)
    for ry in rows:
        if orderly:
            # short dash "number" rows, almost orderly
            x = x0 + 2
            while x < x1 - 3:
                n = rng.randint(1, 3)
                d.line([x, ry, x + n, ry], fill=R(oxide))
                x += n + 2
        else:
            pts = []
            for x in range(x0 + 1, x1):
                y = ry + int(1.6 * math.sin((x - x0) * 0.9 + ry))
                pts.append((x, y))
            d.line(pts, fill=R(oxide))
    # dense ink blots where the numbers pack too thick
    nblot = 3 if tall else 1
    for _ in range(nblot):
        bx = rng.randint(x0 + 3, x1 - 3)
        by = rng.randint(y0 + 4, y1 - 4)
        d.ellipse([bx - 2, by - 1, bx + 2, by + 1], fill=R((120, 62, 30)))
    return img


def slate(rng, tint, pattern):
    """Nanofabricator template: transparent glass slate, inscribed pattern."""
    img = new_img()
    d = ImageDraw.Draw(img)
    d.rectangle([8, 4, 24, 28], fill=R(tint))
    d.rectangle([8, 4, 24, 28], outline=R((110, 116, 120)), width=1)
    d.line([10, 26, 22, 6], fill=R((246, 250, 250)))  # glass shine
    dark = shade(tint, 0.55)
    if pattern == "circuit":
        for y in (10, 16, 22):
            d.line([11, y, 21, y], fill=dark)
        for x in (13, 19):
            d.line([x, 8, x, 24], fill=dark)
        for x, y in ((13, 10), (19, 16), (13, 22)):
            d.rectangle([x - 1, y - 1, x + 1, y + 1], outline=dark)
    elif pattern == "exo":
        d.ellipse([14, 7, 18, 11], outline=dark)          # head
        d.line([16, 11, 16, 19], fill=dark)               # spine
        d.line([11, 13, 21, 13], fill=dark)               # arms
        d.line([16, 19, 12, 25], fill=dark)               # legs
        d.line([16, 19, 20, 25], fill=dark)
        d.rectangle([10, 26, 22, 27], fill=dark)          # caption bar
    elif pattern == "armor":
        for y in (9, 14, 19, 24):
            for x in range(10, 22, 4):
                d.arc([x, y - 2, x + 5, y + 3], 0, 180, fill=dark)
    elif pattern == "photonics":
        for y in (10, 15, 20):
            d.line([10, y, 22, y], fill=dark)
        d.ellipse([13, 12, 19, 18], outline=dark)         # ring resonator
        pts = [(10, 24 + int(1.5 * math.sin(x * 0.8))) for x in range(0, 13)]
        d.line([(10 + i, y) for i, y in enumerate([p[1] for p in pts])],
               fill=dark)
    return img


def coil_wire(rng, color, width=18, loops=6, y0=6, y1=26):
    img = new_img()
    d = ImageDraw.Draw(img)
    cx = 16
    step = (y1 - y0) / loops
    for i in range(loops):
        y = y0 + i * step
        d.ellipse([cx - width // 2, y, cx + width // 2, y + step + 1],
                  outline=R(color), width=1)
    return img


def spring(rng, halfw, coils, y0, y1):
    img = new_img()
    d = ImageDraw.Draw(img)
    steel = (176, 176, 182)
    step = (y1 - y0) / coils
    for i in range(coils):
        y = y0 + i * step
        d.ellipse([16 - halfw, y, 16 + halfw, y + step + 1],
                  outline=R(steel), width=1)
    d.line([16 - halfw, y0, 16 + halfw, y0], fill=R(steel))
    d.line([16 - halfw, y1 + 1, 16 + halfw, y1 + 1], fill=R(steel))
    return img


def qmark(d, x, y, col=(30, 30, 34)):
    d.arc([x, y, x + 4, y + 4], 180, 400, fill=R(col), width=1)
    d.line([x + 4, y + 3, x + 2, y + 5], fill=R(col))
    d.point((x + 2, y + 7), fill=R(col))


DRAW = {}


def reg(name):
    def deco(fn):
        DRAW[name] = fn
        return fn
    return deco


@reg("neoprene")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rounded_rectangle([6, 9, 26, 23], 3, fill=R((72, 72, 76)))
    d.rounded_rectangle([8, 11, 24, 21], 2, fill=R((86, 86, 90)))
    for x in range(9, 25, 3):
        d.point((x, 10), fill=R((130, 130, 134)))
        d.point((x, 22), fill=R((130, 130, 134)))
    d.line([10, 16, 22, 15], fill=R((60, 60, 64)))
    return img


@reg("nre_anomaly_printout_data")
def _(rng):
    return paper_strip(rng, tall=True, orderly=False)


@reg("small_printout_portal_storm_data")
def _(rng):
    return paper_strip(rng, tall=False, orderly=True)


@reg("onyx")
def _(rng):
    return gem(rng, (42, 42, 46))


@reg("opal")
def _(rng):
    return gem(rng, (226, 224, 218),
               speckles=[(240, 178, 198), (168, 228, 228), (182, 234, 190)])


@reg("peridot")
def _(rng):
    return gem(rng, (106, 176, 58))


@reg("ruby")
def _(rng):
    return gem(rng, (196, 38, 48))


@reg("sapphire")
def _(rng):
    return gem(rng, (48, 88, 208))


@reg("tourmaline")
def _(rng):
    # elongated prismatic crystal rather than a round cut
    img = new_img(); d = ImageDraw.Draw(img)
    base = (232, 138, 142)
    d.polygon([(12, 4), (20, 4), (22, 10), (20, 26), (12, 26), (10, 10)],
              fill=R(base))
    d.line([16, 4, 16, 26], fill=shade(base, 0.7))
    d.line([12, 4, 12, 26], fill=shade(base, 1.25))
    d.line([20, 4, 20, 26], fill=shade(base, 0.8))
    d.line([10, 10, 22, 10], fill=shade(base, 1.2))
    d.point((14, 7), fill=shade(base, 1.5))
    return img


@reg("piano_wire")
def _(rng):
    img = coil_wire(rng, (186, 186, 190), width=16, loops=7, y0=5, y1=25)
    d = ImageDraw.Draw(img)
    d.line([24, 22, 28, 27], fill=R((186, 186, 190)))  # loose tail
    return img


@reg("qt_wire")
def _(rng):
    img = coil_wire(rng, (168, 168, 174), width=20, loops=5, y0=7, y1=24)
    d = ImageDraw.Draw(img)
    d.line([6, 10, 3, 5], fill=R((168, 168, 174)))
    d.line([26, 21, 29, 26], fill=R((168, 168, 174)))
    return img


@reg("spring_medium")
def _(rng):
    return spring(rng, 8, 6, 6, 25)


@reg("spring_small")
def _(rng):
    return spring(rng, 5, 5, 9, 23)


@reg("pilot_light")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([11, 11, 21, 25], fill=R((172, 172, 178)))
    d.rectangle([11, 11, 21, 14], fill=R((140, 140, 146)))
    d.rectangle([14, 7, 18, 11], fill=R((96, 96, 102)))       # nozzle
    d.ellipse([13, 17, 19, 23], fill=R((198, 58, 48)))        # piezo button
    d.point((16, 20), fill=R((240, 150, 130)))
    for dx, dy in ((-3, -3), (0, -4), (3, -3)):               # spark
        d.point((16 + dx, 5 + dy + 3), fill=R((250, 230, 140)))
    d.rectangle([11, 25, 21, 27], fill=R((60, 60, 64)))
    return img


@reg("plastic_six_dice")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.polygon([(8, 12), (13, 7), (27, 7), (22, 12)], fill=R((246, 246, 246)))
    d.polygon([(22, 12), (27, 7), (27, 21), (22, 26)], fill=R((206, 206, 210)))
    d.rectangle([8, 12, 22, 26], fill=R((232, 232, 232)))
    for x, y in ((11, 15), (19, 15), (15, 19), (11, 23), (19, 23)):
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=R((40, 40, 44)))
    return img


@reg("power_supply")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([5, 8, 27, 24], fill=R((172, 172, 178)))
    d.rectangle([5, 8, 27, 24], outline=R((110, 110, 116)))
    d.ellipse([8, 11, 18, 21], outline=R((90, 90, 96)), width=1)
    d.ellipse([11, 14, 15, 18], fill=R((120, 120, 126)))
    for a in range(0, 360, 60):
        x = 13 + int(4 * math.cos(math.radians(a)))
        y = 16 + int(4 * math.sin(math.radians(a)))
        d.line([13, 16, x, y], fill=R((90, 90, 96)))
    for y in (11, 14, 17, 20):
        d.line([21, y, 25, y], fill=R((120, 120, 126)))
    d.rectangle([20, 21, 26, 23], fill=R((216, 192, 64)))  # rating label
    return img


@reg("processor")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    for i in range(6, 27, 3):
        d.line([i, 5, i, 8], fill=R((190, 190, 194)))
        d.line([i, 24, i, 27], fill=R((190, 190, 194)))
        d.line([5, i, 8, i], fill=R((190, 190, 194)))
        d.line([24, i, 27, i], fill=R((190, 190, 194)))
    d.rectangle([8, 8, 24, 24], fill=R((66, 66, 72)))
    d.rectangle([12, 12, 20, 20], fill=R((112, 112, 120)))
    d.rectangle([14, 14, 18, 18], fill=R((216, 192, 64)))
    d.point((9, 9), fill=R((216, 192, 64)))
    return img


@reg("radio_repeater_mod")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.line([10, 12, 10, 3], fill=R((120, 120, 126)), width=1)
    d.ellipse([7, 2, 13, 5], outline=R((120, 120, 126)))
    d.rectangle([6, 12, 26, 26], fill=R((168, 168, 174)))
    d.rectangle([6, 12, 26, 26], outline=R((104, 104, 110)))
    d.rectangle([9, 15, 17, 20], fill=R((52, 58, 52)))
    for i, c in enumerate(((90, 200, 90), (220, 80, 60), (240, 200, 80))):
        d.point((20 + i * 2, 16), fill=R(c))
    for r in (4, 7):
        d.arc([22 - r, 8 - r, 22 + r, 8 + r], -60, 30, fill=R((120, 200, 230)))
    d.line([9, 23, 23, 23], fill=R((120, 120, 126)))
    return img


@reg("railroad_track_small")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    for x in range(3, 30, 5):
        d.rectangle([x, 6, x + 3, 26], fill=R((128, 92, 58)))
        d.line([x, 6, x, 26], fill=R((104, 74, 46)))
    for y in (11, 20):
        d.rectangle([2, y, 30, y + 2], fill=R((150, 150, 156)))
        d.line([2, y, 30, y], fill=R((196, 196, 200)))
    return img


@reg("refrigerant_tank")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([11, 9, 21, 27], fill=R((148, 208, 220)))
    d.ellipse([11, 6, 21, 11], fill=R((170, 222, 232)))
    d.line([13, 10, 13, 26], fill=R((178, 230, 238)))
    d.rectangle([11, 17, 21, 20], fill=R((238, 238, 238)))
    d.line([12, 18, 20, 18], fill=R((90, 160, 190)))
    d.rectangle([14, 3, 18, 6], fill=R((110, 110, 116)))
    d.line([12, 4, 20, 4], fill=R((110, 110, 116)))
    return img


@reg("report_t_substrate")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([8, 4, 24, 28], fill=R((240, 240, 236)))
    d.rectangle([8, 4, 24, 28], outline=R((196, 196, 190)))
    d.rectangle([10, 6, 14, 10], fill=R((40, 90, 100)))   # XEDRA logo block
    d.line([10, 10, 14, 6], fill=R((240, 240, 236)))
    d.line([10, 6, 14, 10], fill=R((240, 240, 236)))
    for y in range(13, 26, 3):
        d.line([10, y, rng.randint(18, 22), y], fill=R((168, 168, 164)))
    return img


@reg("robofac_test_data")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rounded_rectangle([6, 10, 26, 22], 2, fill=R((242, 226, 138)))
    d.rounded_rectangle([6, 10, 26, 22], 2, outline=R((196, 170, 70)))
    d.rectangle([9, 13, 14, 18], outline=R((170, 144, 50)))
    d.line([11, 13, 11, 18], fill=R((170, 144, 50)))
    for y in (13, 16, 19):
        d.line([17, y, 24, y], fill=R((206, 182, 84)))
        d.line([17, y, 17, y + 1], fill=R((206, 182, 84)))
    return img


@reg("sample_t_substrate")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.polygon([(9, 10), (16, 6), (23, 10), (16, 13)], fill=R((236, 243, 245)))
    d.polygon([(9, 10), (16, 13), (16, 26), (9, 22)], fill=R((206, 219, 224)))
    d.polygon([(23, 10), (16, 13), (16, 26), (23, 22)], fill=R((186, 203, 210)))
    d.polygon([(16, 14), (19, 18), (16, 22), (13, 18)], fill=R((24, 24, 28)))
    d.point((16, 16), fill=R((70, 70, 78)))
    return img


@reg("sassafras_root")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.ellipse([11, 4, 21, 8], fill=R((168, 128, 84)))
    d.polygon([(12, 6), (20, 6), (18, 16), (17, 27), (15, 27), (14, 16)],
              fill=R((130, 95, 55)))
    for x0, y0, x1, y1 in ((14, 12, 8, 17), (18, 13, 24, 18),
                           (15, 19, 10, 24), (17, 20, 22, 25)):
        d.line([x0, y0, x1, y1], fill=R((116, 84, 48)))
    d.line([15, 8, 15, 24], fill=R((108, 78, 44)))
    return img


@reg("shrapnel")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    pts = [(7, 20), (13, 10), (18, 13), (26, 6), (21, 16), (25, 22),
           (16, 20), (15, 26), (10, 22)]
    d.polygon(pts, fill=R((206, 206, 210)))
    d.polygon([(13, 10), (18, 13), (16, 20), (10, 22)], fill=R((168, 168, 174)))
    d.line([18, 13, 21, 16], fill=R((140, 140, 146)))
    d.point((14, 13), fill=R((240, 240, 244)))
    return img


@reg("small_lcd_screen")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([6, 7, 26, 24], fill=R((58, 58, 64)))
    d.rectangle([8, 9, 24, 22], fill=R((198, 232, 238)))
    for y in (12, 15):
        d.line([10, y, 20, y], fill=R((90, 150, 170)))
    pts = [(10, 20), (13, 17), (16, 19), (19, 15), (22, 16)]
    d.line(pts, fill=R((60, 120, 160)))
    d.rectangle([13, 24, 19, 27], fill=R((58, 58, 64)))
    return img


@reg("soft_adobe_brick_active")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rounded_rectangle([5, 10, 27, 23], 2, fill=R((138, 99, 60)))
    d.rounded_rectangle([5, 10, 27, 23], 2, outline=R((110, 78, 46)))
    d.rectangle([6, 19, 26, 22], fill=R((122, 87, 52)))  # wet base
    for _ in range(14):
        x = rng.randint(7, 24)
        y = rng.randint(12, 21)
        d.line([x, y, x + 2, y - 1], fill=R((196, 168, 110)))  # straw fiber
    return img


@reg("spare_tire_carrier")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    steel = (96, 96, 102)
    d.line([6, 26, 16, 12], fill=R(steel), width=2)
    d.line([26, 26, 16, 12], fill=R(steel), width=2)
    d.rectangle([4, 26, 28, 28], fill=R(steel))
    d.ellipse([11, 8, 21, 18], fill=R((76, 76, 82)))
    d.ellipse([14, 11, 18, 15], fill=R((150, 150, 156)))
    for x, y in ((16, 9), (12, 13), (20, 13), (16, 17)):
        d.point((x, y), fill=R((40, 40, 44)))
    return img


@reg("standard_template_construct")
def _(rng):
    return slate(rng, (168, 228, 234), "circuit")


@reg("standard_template_construct_combat_exoskeleton")
def _(rng):
    return slate(rng, (168, 228, 234), "exo")


@reg("template_armor")
def _(rng):
    return slate(rng, (238, 216, 128), "armor")


@reg("template_photonics")
def _(rng):
    return slate(rng, (238, 216, 128), "photonics")


@reg("thermostat")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rounded_rectangle([9, 6, 23, 26], 3, fill=R((178, 178, 182)))
    d.rounded_rectangle([9, 6, 23, 26], 3, outline=R((130, 130, 134)))
    d.ellipse([11, 9, 21, 19], fill=R((238, 238, 238)))
    for a in range(200, 521, 40):
        x = 16 + int(4.4 * math.cos(math.radians(a)))
        y = 14 + int(4.4 * math.sin(math.radians(a)))
        d.point((x, y), fill=R((90, 90, 96)))
    d.line([16, 14, 19, 11], fill=R((198, 58, 48)), width=1)
    d.ellipse([14, 21, 18, 24], outline=R((150, 110, 60)))  # bimetal coil hint
    return img


@reg("usb_drive_nano")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([3, 14, 10, 20], fill=R((196, 196, 200)))
    d.point((5, 16), fill=R((90, 90, 96)))
    d.point((7, 16), fill=R((90, 90, 96)))
    d.rounded_rectangle([10, 11, 25, 23], 3, fill=R((236, 236, 236)))
    d.rounded_rectangle([10, 11, 25, 23], 3, outline=R((170, 170, 174)))
    d.rectangle([10, 11, 13, 23], fill=R((206, 206, 210)))  # bumper end
    d.point((22, 14), fill=R((90, 200, 90)))
    return img


@reg("wallet_travel")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    d.arc([8, 0, 24, 12], 180, 360, fill=R((80, 80, 86)), width=1)  # neck strap
    d.rounded_rectangle([7, 9, 25, 25], 2, fill=R((82, 82, 88)))
    d.rounded_rectangle([7, 9, 25, 25], 2, outline=R((56, 56, 62)))
    d.rectangle([10, 12, 22, 17], outline=R((150, 150, 156)))  # ID window
    d.line([10, 20, 22, 20], fill=R((150, 150, 156)))
    d.line([10, 22, 19, 22], fill=R((150, 150, 156)))
    return img


@reg("sunlight")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    for a in range(0, 360, 30):
        x0 = 14 + int(8 * math.cos(math.radians(a)))
        y0 = 14 + int(8 * math.sin(math.radians(a)))
        x1 = 14 + int(11 * math.cos(math.radians(a)))
        y1 = 14 + int(11 * math.sin(math.radians(a)))
        d.line([x0, y0, x1, y1], fill=R((240, 208, 88)), width=1)
    d.ellipse([8, 8, 20, 20], fill=R((250, 240, 178)))
    d.ellipse([10, 10, 15, 15], fill=R((252, 248, 210)))
    qmark(d, 23, 21)
    return img


@reg("wind")
def _(rng):
    img = new_img(); d = ImageDraw.Draw(img)
    col = (196, 224, 236)
    for y, ln in ((10, 20), (16, 24), (22, 18)):
        d.line([4, y, 4 + ln, y], fill=R(col), width=1)
        d.arc([4 + ln - 5, y - 4, 4 + ln + 2, y + 3], -90, 160, fill=R(col))
    qmark(d, 24, 23)
    return img


def main():
    for tid in IDS:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = DRAW[tid](rng)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (tid, alphas)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", tid)

    cell, gap, cols = 64, 3, 7
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, tid in enumerate(IDS):
        img = Image.open(os.path.join(BASE, tid, tid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(HERE, "preview_item06.png")
    sheet.save(out)
    print("wrote preview_item06.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
