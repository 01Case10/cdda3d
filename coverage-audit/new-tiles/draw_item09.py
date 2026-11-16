#!/usr/bin/env python3
"""Batch ITEM09 (Phase 2g.4 scope extension): carnivore
comestibles, final set.

28 ids from data/json/items/comestibles/carnivore.json
(authoritative list embedded below; source list:
~/workspace/item-batches/09.txt). Parameterized family
renderers driven by each entry's resolved data (copy-from
chains resolved by hand against carnivore.json):

- Meat chunks (copy-from meat/flesh/poultry): mutflesh is the
  meat chunk gone off-green per its description; poultry scrap
  raw (tiny, pale) vs cooked (tiny, browned); poultry_cooked a
  golden-brown chunk; poultry_smoked a dark smoked strip;
  raw_curing_fatty_meat_active a porkbelly slab with fat layers
  and salt crystals.
- Offal (copy-from offal): cooked organ pile; canned = open tin
  of organ meat; pickled = brine jar of organ shapes; tainted
  innards (animal/human) as green-gray rotted piles.
- Boiled stomachs (red, ',' ): tied pouch sacs, tinted per
  source -- animal red-brown, human grayer, demihuman faintly
  green.
- Alien flesh (black_red): nether_grub a segmented chitin grub
  sheltering a pale larva; nether_huntsman_organ a ridged
  accordion tube; nl_flesh_powerbox a perfect oily flesh cube;
  tainted_marrow_human a bone end with green spongy marrow.
- Folded raw hides (','): hleather smooth pink, demihuman pink
  with an alien sheen, hfur brown with fur strokes, tainted fur
  mangy and patchy, tainted leather green-gray with cuts and
  lumps, tainted leather scraps as small shreds.
- Shellfish/salted: oyster in rough gray shell, shucked oyster
  on the half shell in its liquor, salted_fish a stiff salt-
  crusted dried fish, salted_mollusk shriveled reddish pieces.

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
(brine, liquor) faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 28 folders + preview_item09.png.
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

IDS = [
    "mutflesh",
    "nether_grub",
    "nether_huntsman_organ",
    "nl_flesh_powerbox",
    "offal_canned",
    "offal_cooked",
    "offal_pickled",
    "oyster",
    "oyster_shucked",
    "poultry_cooked",
    "poultry_scrap",
    "poultry_scrap_cooked",
    "poultry_smoked",
    "raw_curing_fatty_meat_active",
    "raw_demihumanleather",
    "raw_hfur",
    "raw_hleather",
    "raw_tainted_fur",
    "raw_tainted_leather",
    "raw_tainted_leather_scraps",
    "salted_fish",
    "salted_mollusk",
    "small_demihuman_stomach_boiled",
    "small_hstomach_boiled",
    "small_stomach_boiled",
    "tainted_innards",
    "tainted_innards_human",
    "tainted_marrow_human",
]

# palette helpers -------------------------------------------------


def shade(col, f):
    return tuple(max(0, min(255, int(c * f))) for c in col) + (255,)


def blob_pts(rng, cx, cy, rx, ry, wob=0.22, n=14):
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        r = 1 + rng.uniform(-wob, wob)
        pts.append((cx + rx * r * math.cos(a), cy + ry * r * math.sin(a)))
    return pts


def speckle(d, rng, box, col, count, rmax=1):
    x0, y0, x1, y1 = box
    for _ in range(count):
        x = rng.uniform(x0, x1)
        y = rng.uniform(y0, y1)
        r = rng.uniform(0.4, rmax)
        d.ellipse([x - r, y - r, x + r, y + r], fill=col)


# family renderers ------------------------------------------------

MEAT = (176, 56, 50)
POULTRY_RAW = (236, 198, 172)
POULTRY_COOKED = (172, 112, 52)
FAT = (240, 231, 210)


def meat_chunk(rng, base, size=1.0, fat_streaks=True, patches=None):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx, cy = 16, 17
    rx, ry = 11 * size, 8 * size
    pts = blob_pts(rng, cx, cy, rx, ry)
    d.polygon(pts, fill=base + (255,), outline=shade(base, 0.55))
    speckle(d, rng, (cx - rx * 0.7, cy - ry * 0.6, cx + rx * 0.7,
                      cy + ry * 0.6), shade(base, 1.18), int(26 * size))
    speckle(d, rng, (cx - rx * 0.7, cy - ry * 0.6, cx + rx * 0.7,
                      cy + ry * 0.6), shade(base, 0.78), int(16 * size))
    if fat_streaks:
        for _ in range(3):
            y = cy + rng.uniform(-ry * 0.5, ry * 0.5)
            d.arc([cx - rx * 0.75, y - 3, cx + rx * 0.75, y + 3],
                  200, 340, fill=FAT, width=1)
    if patches:
        for col, n in patches:
            speckle(d, rng, (cx - rx * 0.65, cy - ry * 0.55,
                             cx + rx * 0.65, cy + ry * 0.55), col, n,
                    rmax=1.8)
    return img


def smoked_strip(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base = (116, 70, 38)
    pts_top, pts_bot = [], []
    for x in range(4, 29, 3):
        pts_top.append((x, 13 + rng.uniform(-1, 1)))
        pts_bot.append((x, 20 + rng.uniform(-1, 1)))
    d.polygon(pts_top + pts_bot[::-1], fill=base + (255,),
              outline=shade(base, 0.5))
    d.line([(6, 16.5), (26, 16.5)], fill=shade(base, 1.35), width=2)
    speckle(d, rng, (6, 14, 26, 19), shade(base, 0.6), 14)
    speckle(d, rng, (6, 14, 26, 19), shade(base, 1.25), 8)
    return img


def porkbelly_slab(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # slab with fat/meat layers, slight perspective top face
    d.polygon([(6, 12), (26, 10), (27, 13), (7, 15)],
              fill=(196, 96, 84, 255), outline=(120, 52, 46, 255))
    bands = [((7, 15), (27, 13), (27, 17), (7, 19), FAT),
             ((7, 19), (27, 17), (27, 20), (7, 22), (178, 70, 60, 255)),
             ((7, 22), (27, 20), (27, 22), (7, 24), FAT),
             ((7, 24), (27, 22), (27, 25), (7, 27), (168, 62, 54, 255))]
    for x0, x1, x2, x3, col in [(b[0], b[1], b[2], b[3], b[4])
                                for b in bands]:
        d.polygon([x0, x1, x2, x3], fill=col,
                  outline=shade(col[:3], 0.6))
    # curing salt crystals on the top face
    speckle(d, rng, (8, 11, 25, 14), (255, 255, 250, 255), 22, rmax=0.8)
    speckle(d, rng, (8, 16, 25, 26), shade((178, 70, 60), 1.2), 12)
    return img


def offal_pile(rng, liver, kidney, gut, slime=None):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # intestine coils underneath
    for (x, y, r) in [(12, 21, 5), (19, 22, 5.5), (15.5, 18.5, 5)]:
        d.ellipse([x - r, y - r * 0.62, x + r, y + r * 0.62],
                  fill=gut + (255,), outline=shade(gut, 0.6))
        d.ellipse([x - r * 0.55, y - r * 0.34, x + r * 0.55,
                   y + r * 0.34], outline=shade(gut, 1.25))
    # liver: big smooth lobe
    pts = blob_pts(rng, 13, 13, 8, 5, wob=0.15)
    d.polygon(pts, fill=liver + (255,), outline=shade(liver, 0.55))
    d.arc([8, 10, 18, 16], 180, 320, fill=shade(liver, 1.3), width=1)
    # kidney bean
    d.ellipse([19, 10, 27, 17], fill=kidney + (255,),
              outline=shade(kidney, 0.55))
    d.ellipse([21.5, 12, 24.5, 15], fill=shade(kidney, 0.7))
    speckle(d, rng, (7, 9, 27, 24), shade(liver, 1.15), 12)
    if slime:
        speckle(d, rng, (7, 9, 27, 24), slime, 16, rmax=1.6)
    return img


def offal_can(rng):
    img = offal_pile(rng, (150, 84, 48), (128, 52, 44), (188, 128, 96))
    d = ImageDraw.Draw(img)
    # redraw as a can: can body over lower half, offal heaped at top
    img2 = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d2 = ImageDraw.Draw(img2)
    d2.rectangle([9, 13, 23, 27], fill=(148, 150, 152, 255),
                 outline=(96, 98, 100, 255))
    d2.rectangle([9, 18, 23, 23], fill=(140, 62, 52, 255))
    d2.line([(11, 14), (11, 26)], fill=(178, 180, 182, 255))
    d2.ellipse([9, 11, 23, 15], fill=(170, 172, 174, 255),
               outline=(96, 98, 100, 255))
    img2.alpha_composite(img.crop((0, 0, 32, 14)), (0, 0))
    d3 = ImageDraw.Draw(img2)
    d3.ellipse([9, 11, 23, 15], outline=(96, 98, 100, 255))
    return img2


def offal_jar(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    brine = (196, 204, 158)  # pale opaque brine
    d.rounded_rectangle([10, 8, 22, 27], 3, fill=brine + (255,),
                        outline=(120, 128, 96, 255))
    # organ shapes suspended in the brine
    for (x, y, rx, ry, col) in [(14, 15, 3.4, 2.6, (146, 96, 84)),
                                (18.5, 20, 3.8, 2.8, (128, 74, 66)),
                                (14.5, 23, 3.0, 2.2, (160, 110, 92)),
                                (19, 12.5, 2.6, 2.0, (138, 88, 76))]:
        d.ellipse([x - rx, y - ry, x + rx, y + ry], fill=col + (255,),
                  outline=shade(col, 0.6))
    d.rectangle([10, 8, 22, 11], fill=(110, 112, 114, 255),
                outline=(74, 76, 78, 255))  # lid
    d.line([(12, 13), (12, 25)], fill=(226, 232, 200, 255), width=1)
    speckle(d, rng, (11, 12, 21, 26), (226, 232, 196, 255), 8, rmax=0.7)
    return img


def stomach(rng, base):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # tied neck
    d.rectangle([14.5, 5, 17.5, 9], fill=shade(base, 0.8))
    d.line([(13.5, 8.5), (18.5, 8.5)], fill=shade(base, 0.45), width=2)
    pts = blob_pts(rng, 16, 18, 8.5, 8, wob=0.13)
    d.polygon(pts, fill=base + (255,), outline=shade(base, 0.55))
    d.arc([11, 12, 21, 24], 100, 250, fill=shade(base, 1.25), width=1)
    speckle(d, rng, (10, 12, 22, 24), shade(base, 1.12), 14)
    speckle(d, rng, (10, 12, 22, 24), shade(base, 0.8), 10)
    return img


def marrow(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    bone = (228, 218, 196)
    pts = blob_pts(rng, 16, 16, 11, 9, wob=0.3)
    d.polygon(pts, fill=bone + (255,), outline=shade(bone, 0.65))
    green = (104, 138, 66)
    pts2 = blob_pts(rng, 16, 16, 7, 5.5, wob=0.3)
    d.polygon(pts2, fill=green + (255,), outline=shade(green, 0.6))
    speckle(d, rng, (10, 12, 22, 20), shade(green, 1.3), 22, rmax=1.3)
    speckle(d, rng, (10, 12, 22, 20), shade(green, 0.7), 14, rmax=1.2)
    speckle(d, rng, (6, 8, 26, 24), shade(bone, 0.85), 8)
    return img


def grub(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    shell = (70, 26, 30)       # black_red chitin
    larva = (206, 178, 150)
    # curled segmented shell: arcs of overlapping segment ellipses
    segs = [(10, 20), (13, 15), (18, 12.5), (23, 14), (25.5, 18.5),
            (24, 23), (19, 25.5), (13.5, 24.5)]
    for i, (x, y) in enumerate(segs):
        r = 4.6 - i * 0.18
        d.ellipse([x - r, y - r * 0.8, x + r, y + r * 0.8],
                  fill=shade(shell, 1 + (i % 2) * 0.16),
                  outline=(30, 10, 14, 255))
    # curious markings: pale dots along the shell
    for (x, y) in [(13, 14), (18, 11.5), (23, 13), (25, 18)]:
        d.ellipse([x - 0.8, y - 0.8, x + 0.8, y + 0.8],
                  fill=(196, 120, 90, 255))
    # pale larva head poking out at the front
    d.ellipse([6.5, 16.5, 12, 22.5], fill=larva + (255,),
              outline=shade(larva, 0.6))
    d.ellipse([8, 18.5, 9.4, 19.9], fill=(40, 20, 16, 255))
    return img


def huntsman_organ(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base = (96, 34, 38)  # black_red leathery tube
    # tapered tube body
    d.polygon([(5, 13), (27, 11), (27, 21), (5, 20)],
              fill=base + (255,), outline=(44, 14, 18, 255))
    # accordion ridges
    for i, x in enumerate(range(7, 27, 3)):
        w = 2 if i % 2 == 0 else 1
        d.line([(x, 12.5), (x, 20.5)], fill=shade(base, 0.55), width=w)
        d.line([(x + 1, 12.5), (x + 1, 20.5)], fill=shade(base, 1.3),
               width=1)
    # slimy sheen along the top
    d.line([(6, 12.5), (26, 11)], fill=(196, 130, 120, 255), width=1)
    speckle(d, rng, (6, 12, 26, 20), shade(base, 1.2), 10, rmax=0.8)
    return img


def flesh_cube(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    top = (150, 62, 62)
    left = (122, 44, 48)
    right = (94, 32, 38)   # black_red family
    d.polygon([(16, 5), (26, 10), (16, 15), (6, 10)],
              fill=top + (255,), outline=(50, 16, 20, 255))
    d.polygon([(6, 10), (16, 15), (16, 27), (6, 22)],
              fill=left + (255,), outline=(50, 16, 20, 255))
    d.polygon([(16, 15), (26, 10), (26, 22), (16, 27)],
              fill=right + (255,), outline=(50, 16, 20, 255))
    # oily sheen streaks
    d.line([(9, 11), (15, 8)], fill=(216, 150, 140, 255), width=1)
    d.line([(8, 13), (8, 19)], fill=(170, 92, 92, 255), width=1)
    d.line([(23, 13), (23, 19)], fill=(140, 66, 70, 255), width=1)
    speckle(d, rng, (8, 12, 24, 24), shade(top, 0.8), 8, rmax=0.8)
    return img


def folded_hide(rng, base, fur_col=None, taint=0, sheen=None):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # folded stack: two overlapping folded sheets
    d.polygon([(6, 12), (25, 10), (26, 22), (7, 24)],
              fill=shade(base, 0.82), outline=shade(base, 0.5))
    d.polygon([(7, 9), (26, 8), (25, 19), (6, 20)],
              fill=base + (255,), outline=shade(base, 0.5))
    d.line([(7, 14), (25.5, 13)], fill=shade(base, 0.62), width=1)
    if fur_col:
        for _ in range(90):
            x = rng.uniform(8, 25)
            y = rng.uniform(9, 18)
            ln = rng.uniform(1.5, 3.2)
            d.line([(x, y), (x + rng.uniform(-0.7, 0.7), y + ln)],
                   fill=fur_col, width=1)
    if sheen:
        speckle(d, rng, (8, 9, 25, 18), sheen, 14, rmax=1.4)
    if taint:
        # cuts and divots
        for _ in range(6):
            x = rng.uniform(9, 23)
            y = rng.uniform(10, 17)
            d.line([(x, y), (x + rng.uniform(2, 4), y + rng.uniform(-1, 1))],
                   fill=(52, 54, 40, 255), width=1)
        # malformed lumps on the fold edge
        for (x, y) in [(11, 8.6), (19, 8.2), (24, 12)]:
            d.ellipse([x - 1.6, y - 1.6, x + 1.6, y + 1.6],
                      fill=shade(base, 1.12), outline=shade(base, 0.5))
        speckle(d, rng, (8, 9, 25, 20), (88, 102, 62, 255), 16, rmax=1.5)
    speckle(d, rng, (8, 9, 25, 20), shade(base, 1.1), 8)
    return img


def mangy_fur(rng):
    img = folded_hide(rng, (112, 84, 58), fur_col=(140, 112, 76, 255))
    d = ImageDraw.Draw(img)
    # bare mangy patches where fur is gone + sickly spots
    for (x, y, rx, ry) in [(12, 12, 3.4, 2.2), (20, 15.5, 2.8, 1.9),
                           (16.5, 10.5, 2.4, 1.6)]:
        d.ellipse([x - rx, y - ry, x + rx, y + ry],
                  fill=(150, 116, 96, 255), outline=(96, 74, 54, 255))
    speckle(d, rng, (8, 9, 25, 20), (96, 108, 66, 255), 14, rmax=1.4)
    return img


def leather_scraps(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base = (196, 128, 122)  # pink, sickly
    for (cx, cy, rx, ry) in [(11, 12, 4.5, 3), (21, 10.5, 3.6, 2.6),
                             (16, 18, 5, 3.2), (23.5, 20, 3.4, 2.4),
                             (10, 22.5, 3.2, 2.2)]:
        pts = blob_pts(rng, cx, cy, rx, ry, wob=0.35)
        d.polygon(pts, fill=base + (255,), outline=shade(base, 0.55))
    speckle(d, rng, (7, 8, 27, 25), (110, 124, 78, 255), 12, rmax=1.2)
    speckle(d, rng, (7, 8, 27, 25), shade(base, 1.15), 8)
    return img


def oyster_shell(rng, shucked=False):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    shell = (168, 168, 172)  # light_gray
    # fan-shaped rough bottom valve
    pts = [(16, 7)]
    for i in range(13):
        a = math.pi * (0.08 + 0.84 * i / 12)
        pts.append((16 + 11.5 * math.cos(a), 8 + 15 * math.sin(a)))
    d.polygon(pts, fill=shell + (255,), outline=shade(shell, 0.55))
    for i in range(5):  # radial ridges
        a = math.pi * (0.2 + 0.6 * i / 4)
        d.line([(16, 8), (16 + 10.5 * math.cos(a), 8 + 14 * math.sin(a))],
               fill=shade(shell, 0.78), width=1)
    speckle(d, rng, (7, 10, 25, 22), shade(shell, 0.7), 12)
    if not shucked:
        # top valve ajar above, dark gap between
        pts2 = [(16, 5)]
        for i in range(13):
            a = math.pi * (0.08 + 0.84 * i / 12)
            pts2.append((16 + 10.5 * math.cos(a), 6 + 10 * math.sin(a)))
        d.polygon(pts2, fill=shade(shell, 1.1), outline=shade(shell, 0.55))
        d.line([(7, 13), (25, 13)], fill=(52, 48, 46, 255), width=2)
    else:
        # oyster meat in its liquor on the half shell
        meat = (226, 214, 196)
        pts3 = blob_pts(rng, 16, 15, 6.5, 5, wob=0.18)
        d.polygon(pts3, fill=meat + (255,), outline=shade(meat, 0.7))
        d.ellipse([12.5, 12.5, 18, 17], fill=(206, 188, 164, 255))
        d.arc([11, 11, 21, 19], 200, 320, fill=(250, 246, 238, 255),
              width=1)
    return img


def salted_fish(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    body = (178, 178, 180)  # light_gray, dried stiff
    d.polygon([(4, 16), (9, 11), (21, 11.5), (27, 8), (26, 16),
               (27, 24), (21, 20.5), (9, 21), (4, 16)],
              fill=body + (255,), outline=shade(body, 0.55))
    d.line([(9, 12), (9, 20)], fill=shade(body, 0.7), width=1)  # gill
    d.ellipse([6.2, 14.4, 8.2, 16.4], fill=(40, 40, 44, 255))  # eye
    speckle(d, rng, (5, 10, 26, 22), (252, 252, 250, 255), 30, rmax=0.8)
    speckle(d, rng, (5, 10, 26, 22), shade(body, 0.8), 10)
    return img


def salted_mollusk(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base = (206, 128, 112)  # light_red, dried
    for (cx, cy, rx, ry) in [(12, 13, 5, 3.6), (20, 18, 5.6, 4),
                             (13, 21.5, 4.2, 3)]:
        pts = blob_pts(rng, cx, cy, rx, ry, wob=0.3)
        d.polygon(pts, fill=base + (255,), outline=shade(base, 0.55))
        for k in range(3):  # wrinkle arcs
            d.arc([cx - rx + k * 1.6, cy - ry, cx + rx - k * 1.6,
                   cy + ry], 300, 60, fill=shade(base, 0.75), width=1)
    speckle(d, rng, (7, 9, 26, 25), (252, 250, 246, 255), 22, rmax=0.8)
    return img


# dispatch ----------------------------------------------------------

def render(item_id, rng):
    if item_id == "mutflesh":
        return meat_chunk(rng, (150, 84, 62),
                          patches=[((104, 138, 66, 255), 18),
                                   ((128, 150, 84, 255), 10)])
    if item_id == "poultry_scrap":
        return meat_chunk(rng, POULTRY_RAW, size=0.55, fat_streaks=False)
    if item_id == "poultry_scrap_cooked":
        return meat_chunk(rng, POULTRY_COOKED, size=0.55,
                          fat_streaks=False)
    if item_id == "poultry_cooked":
        return meat_chunk(rng, POULTRY_COOKED, size=0.9)
    if item_id == "poultry_smoked":
        return smoked_strip(rng)
    if item_id == "raw_curing_fatty_meat_active":
        return porkbelly_slab(rng)
    if item_id == "offal_cooked":
        return offal_pile(rng, (146, 82, 46), (124, 54, 44),
                          (182, 124, 94))
    if item_id == "offal_canned":
        return offal_can(rng)
    if item_id == "offal_pickled":
        return offal_jar(rng)
    if item_id == "tainted_innards":
        return offal_pile(rng, (104, 110, 66), (92, 96, 58),
                          (128, 130, 96), slime=(150, 168, 104, 255))
    if item_id == "tainted_innards_human":
        return offal_pile(rng, (96, 100, 70), (84, 88, 62),
                          (118, 118, 96), slime=(140, 156, 100, 255))
    if item_id == "small_stomach_boiled":
        return stomach(rng, (156, 66, 54))
    if item_id == "small_hstomach_boiled":
        return stomach(rng, (146, 74, 62))
    if item_id == "small_demihuman_stomach_boiled":
        return stomach(rng, (140, 78, 58))
    if item_id == "tainted_marrow_human":
        return marrow(rng)
    if item_id == "nether_grub":
        return grub(rng)
    if item_id == "nether_huntsman_organ":
        return huntsman_organ(rng)
    if item_id == "nl_flesh_powerbox":
        return flesh_cube(rng)
    if item_id == "raw_hleather":
        return folded_hide(rng, (228, 158, 148))
    if item_id == "raw_demihumanleather":
        return folded_hide(rng, (224, 152, 146),
                           sheen=(168, 190, 140, 255))
    if item_id == "raw_hfur":
        return folded_hide(rng, (128, 96, 64),
                           fur_col=(158, 126, 84, 255))
    if item_id == "raw_tainted_fur":
        return mangy_fur(rng)
    if item_id == "raw_tainted_leather":
        return folded_hide(rng, (158, 158, 128), taint=1)
    if item_id == "raw_tainted_leather_scraps":
        return leather_scraps(rng)
    if item_id == "oyster":
        return oyster_shell(rng, shucked=False)
    if item_id == "oyster_shucked":
        return oyster_shell(rng, shucked=True)
    if item_id == "salted_fish":
        return salted_fish(rng)
    if item_id == "salted_mollusk":
        return salted_mollusk(rng)
    raise ValueError(item_id)


def main():
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = render(item_id, rng)
        assert set(img.getchannel("A").getdata()) <= {0, 255}, item_id
        imgs[item_id] = img
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f,
                      indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols, cell = 7, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item09.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item09.png {sheet.size}")


if __name__ == "__main__":
    main()
