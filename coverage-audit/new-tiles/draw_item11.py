#!/usr/bin/env python3
"""Batch ITEM11 (Phase 2g.4 scope extension): meat dishes,
final set.

24 ids from data/json/items/comestibles/meat_dishes.json
(authoritative list embedded below; source list:
~/workspace/item-batches/11.txt). Parameterized family
renderers driven by each entry's data:

- Paella: dark paellera pan of saffron rice with pepper
  strips, beans, chicken, meat and snails.
- Dumplings/pasta: pelmeni as a pile of pinched half-moon
  dumplings; ravioli as crimped dough squares on a sauce smear.
- Pemmican bars: one compressed-bar renderer -- berry
  speckles (pemmican), plain fat-flecked (pemmican_meat),
  green plant flecks (pemmican_veggy).
- Pork and beans: bowl of beans in brown sauce with pale
  pork-fat chunks.
- Rehydrated family (copy-from the cooked proteins): flaked
  white fish, a pair of frog legs, segmented lobster tail
  with meat fan, a pile of meat chunks, open mollusks with
  pink meat, a poultry drumstick and breast chunk.
- Royal beef: meat chunk under a glossy milky-amber royal
  jelly glaze.
- Sarma: stacked grape-leaf rolls with spiral ends.
- Sashimi: fanned raw fish slivers on a dark plate with
  garnish and roe.
- Sausages: one capsule renderer -- plump red raw links,
  a dark wrinkled smoked wasteland ring, a leaner gray-red
  raw wasteland link, glossy maple-glazed sweet links.
- Sausage gravy: biscuits drowned in pale gravy with
  sausage crumbles.
- Scrapple: stacked pan-fried loaf slices with grain flecks.
- Sushi: maki slices (nori/rice/pink fish cross-sections)
  for the fish roll; a nori temaki cone with meat strips
  for the meat roll.

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 24 folders + preview_item11.png.
"""
import json
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
    "paella_valenciana",
    "pelmeni",
    "pemmican",
    "pemmican_meat",
    "pemmican_veggy",
    "pork_beans",
    "ravioli",
    "rehydrated_fish",
    "rehydrated_froglegs",
    "rehydrated_lobster",
    "rehydrated_meat",
    "rehydrated_mollusk",
    "rehydrated_poultry",
    "royal_beef",
    "sarma",
    "sashimi",
    "sausage_raw",
    "sausage_wasteland",
    "sausage_wasteland_raw",
    "sausagegravy",
    "scrapple",
    "sushi_fishroll",
    "sushi_meatroll",
    "sweet_sausage",
]


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- paella
def paella(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pan = (46, 46, 54, 255)
    d.ellipse([1, 10, 31, 28], fill=pan)                      # pan body
    d.ellipse([3, 8, 29, 25], fill=shade(pan, 1.35))          # rim
    d.ellipse([4, 9, 28, 24], fill=(226, 168, 58, 255))       # saffron rice
    for _ in range(46):                                       # rice grains
        x, y = rng.randint(6, 26), rng.randint(11, 22)
        c = rng.choice([(240, 190, 84, 255), (208, 148, 44, 255)])
        d.line([x, y, x + 1, y], fill=c)
    d.polygon([(7, 12), (15, 10), (16, 13), (8, 15)],
              fill=(196, 54, 40, 255))                        # pepper strip
    d.polygon([(18, 19), (26, 17), (27, 20), (19, 22)],
              fill=(188, 44, 34, 255))
    for x, y in ((10, 19), (21, 12), (14, 16)):                # green beans
        d.line([x, y, x + 4, y - 1], fill=(84, 128, 52, 255), width=2)
    d.ellipse([11, 13, 16, 18], fill=(214, 164, 104, 255),
              outline=(168, 118, 66, 255))                    # chicken piece
    d.rectangle([20, 13, 24, 17], fill=(146, 92, 58, 255),
                outline=(110, 64, 40, 255))                   # rabbit/meat
    for x, y in ((8, 21), (24, 21)):                           # snails
        d.ellipse([x, y, x + 3, y + 3], fill=(122, 96, 62, 255),
                  outline=(84, 64, 40, 255))
    d.polygon([(26, 9), (29, 11), (26, 13)], fill=(240, 208, 84, 255))
    return img                                                # lemon wedge


# ------------------------------------------------------------- dumplings
def _dumpling(d, cx, cy, w=9, h=6):
    dough = (234, 208, 172, 255)
    d.pieslice([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
               0, 180, fill=dough, outline=shade(dough, 0.78))
    d.rectangle([cx - w // 2, cy - 1, cx + w // 2, cy + h // 2],
                fill=dough)
    d.line([cx - w // 2, cy + h // 2, cx + w // 2, cy + h // 2],
           fill=shade(dough, 0.78))
    for i in range(4):                                        # pinched edge
        x = cx - w // 2 + 1 + i * 2
        d.point((x, cy - h // 2 + 1), fill=shade(dough, 0.7))


def pelmeni(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for cx, cy in ((10, 21), (17, 22), (24, 21), (13, 15), (21, 15),
                   (17, 9)):
        _dumpling(d, cx, cy)
    return img


def ravioli(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([4, 12, 28, 26], fill=(206, 118, 88, 255))      # sauce smear
    dough = (228, 196, 150, 255)
    for cx, cy, a in ((11, 12, 12), (21, 11, -18), (9, 20, -8),
                      (17, 19, 20), (25, 19, 6)):
        import math
        pts = []
        for k in range(4):
            ang = math.radians(a + 45 + k * 90)
            pts.append((cx + 5 * math.cos(ang), cy + 5 * math.sin(ang)))
        d.polygon(pts, fill=dough, outline=shade(dough, 0.72))
        for px, py in pts:                                    # crimp marks
            d.point((int(px), int(py)), fill=shade(dough, 0.65))
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2],
                  fill=shade(dough, 1.06))
    return img


# -------------------------------------------------------------- pemmican
def pemmican(rng, kind):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base = (126, 82, 50, 255)
    d.rectangle([7, 13, 25, 23], fill=shade(base, 0.8))       # side face
    d.rectangle([7, 10, 25, 21], fill=base)                   # top block
    d.rectangle([7, 10, 25, 12], fill=shade(base, 1.15))
    for _ in range(10):                                       # fat flecks
        x, y = rng.randint(8, 24), rng.randint(13, 20)
        d.point((x, y), fill=(238, 226, 204, 255))
    if kind == "berry":
        for _ in range(9):
            x, y = rng.randint(8, 24), rng.randint(12, 20)
            c = rng.choice([(128, 34, 44, 255), (84, 48, 108, 255),
                            (160, 52, 48, 255)])
            d.ellipse([x, y, x + 1, y + 1], fill=c)
    elif kind == "veggy":
        for _ in range(9):
            x, y = rng.randint(8, 24), rng.randint(12, 20)
            c = rng.choice([(96, 128, 62, 255), (140, 128, 84, 255)])
            d.point((x, y), fill=c)
            d.point((x + 1, y), fill=c)
    return img


# ------------------------------------------------------------ pork/beans
def pork_beans(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([4, 16, 28, 28], fill=(70, 62, 66, 255))        # bowl
    d.ellipse([4, 9, 28, 21], fill=(110, 66, 36, 255))        # sauce
    d.ellipse([5, 10, 27, 20], fill=(128, 80, 44, 255))
    for _ in range(16):                                       # beans
        x, y = rng.randint(7, 23), rng.randint(11, 18)
        d.ellipse([x, y, x + 3, y + 2], fill=(198, 152, 102, 255))
    for _ in range(5):                                        # pork fat
        x, y = rng.randint(8, 22), rng.randint(11, 17)
        d.rectangle([x, y, x + 3, y + 2], fill=(240, 216, 198, 255),
                    outline=(214, 178, 160, 255))
    return img


# ------------------------------------------------------------ rehydrated
def rehydrated_fish(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    white = (240, 236, 228, 255)
    d.polygon([(6, 14), (20, 10), (27, 15), (22, 21), (9, 21)],
              fill=white, outline=(196, 190, 178, 255))       # fillet
    for x in (11, 15, 19, 23):                                # flake seams
        d.line([x, 12, x - 2, 20], fill=(200, 194, 182, 255))
    for _ in range(4):                                        # loose flakes
        x, y = rng.randint(7, 24), rng.randint(23, 26)
        d.ellipse([x, y, x + 3, y + 1], fill=white)
    return img


def rehydrated_froglegs(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    skin = (233, 172, 158, 255)
    out = (192, 118, 106, 255)
    for dx, flip in ((0, 1), (9, -1)):                         # a pair of legs
        cx = 11 + dx
        d.ellipse([cx - 4, 7, cx + 4, 16], fill=skin, outline=out)  # thigh
        d.polygon([(cx - 2, 15), (cx + 2, 15),
                   (cx + 3 * flip, 26), (cx + 1 * flip, 26)],
                  fill=skin, outline=out)                     # shin
        d.line([cx + 1 * flip, 26, cx + 4 * flip, 27], fill=out)
    return img


def rehydrated_lobster(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    shell = (204, 84, 48, 255)
    for i in range(4):                                        # tail segments
        x = 9 + i * 4
        d.rectangle([x, 13 - (i % 2), x + 5, 20 - (i % 2)],
                    fill=shade(shell, 1.0 - i * 0.07),
                    outline=shade(shell, 0.7))
    d.polygon([(25, 12), (30, 9), (29, 15), (30, 20), (25, 18)],
              fill=shell, outline=shade(shell, 0.7))          # tail fan
    d.polygon([(6, 14), (10, 13), (10, 19), (6, 19)],
              fill=(244, 238, 230, 255))                      # meat end
    return img


def rehydrated_meat(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cooked = (168, 88, 62, 255)
    for cx, cy, w, h in ((11, 12, 10, 7), (20, 11, 9, 6), (9, 19, 9, 6),
                         (17, 18, 11, 7), (24, 19, 7, 5)):
        d.ellipse([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
                  fill=cooked, outline=shade(cooked, 0.72))
        d.point((cx - 1, cy), fill=(238, 220, 198, 255))      # fat fleck
        d.point((cx + 2, cy + 1), fill=shade(cooked, 1.18))
    return img


def rehydrated_mollusk(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    shell = (148, 128, 108, 255)
    meat = (233, 152, 162, 255)
    for cx, cy in ((11, 13), (21, 12), (16, 21)):
        d.pieslice([cx - 6, cy - 5, cx + 6, cy + 5], 180, 360,
                   fill=shell, outline=shade(shell, 0.7))     # top shell
        d.pieslice([cx - 6, cy - 2, cx + 6, cy + 6], 0, 180,
                   fill=shade(shell, 1.12), outline=shade(shell, 0.7))
        d.ellipse([cx - 3, cy - 2, cx + 3, cy + 3], fill=meat,
                  outline=shade(meat, 0.78))                  # pink meat
    return img


def rehydrated_poultry(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    skin = (228, 182, 130, 255)
    out = (178, 128, 78, 255)
    d.ellipse([7, 9, 18, 20], fill=skin, outline=out)         # drumstick meat
    d.polygon([(15, 17), (19, 19), (24, 26), (21, 28), (16, 21)],
              fill=skin, outline=out)
    d.ellipse([21, 25, 26, 29], fill=(244, 240, 232, 255),
              outline=out)                                    # bone knob
    d.ellipse([19, 10, 28, 17], fill=shade(skin, 1.05),
              outline=out)                                    # breast chunk
    return img


# -------------------------------------------------------------- specials
def royal_beef(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    meat = (152, 72, 54, 255)
    d.polygon([(8, 13), (22, 10), (27, 16), (24, 24), (11, 25), (6, 19)],
              fill=meat, outline=shade(meat, 0.7))
    jelly = (242, 216, 148, 255)
    d.polygon([(8, 13), (22, 10), (27, 16), (20, 15), (14, 17), (9, 17)],
              fill=jelly)                                     # glaze cap
    for x, ln in ((11, 5), (17, 7), (23, 4)):                  # drips
        d.rectangle([x, 14, x + 2, 14 + ln], fill=jelly)
    d.line([10, 12, 20, 11], fill=(252, 244, 214, 255))       # gloss
    d.ellipse([12, 24, 22, 27], fill=shade(jelly, 0.94))      # pooled jelly
    return img


def sarma(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    leaf = (76, 118, 52, 255)
    for cx, cy, w in ((11, 20, 13), (22, 20, 13), (16, 13, 14)):
        d.rectangle([cx - w // 2, cy - 3, cx + w // 2, cy + 3],
                    fill=leaf, outline=shade(leaf, 0.68))
        d.ellipse([cx + w // 2 - 3, cy - 3, cx + w // 2 + 1, cy + 3],
                  fill=shade(leaf, 1.14), outline=shade(leaf, 0.68))
        d.ellipse([cx + w // 2 - 2, cy - 1, cx + w // 2, cy + 1],
                  outline=shade(leaf, 0.6))                   # spiral end
        d.line([cx - w // 2 + 2, cy, cx + w // 2 - 4, cy],
               fill=shade(leaf, 0.8))                         # leaf vein
    return img


def sashimi(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([3, 11, 29, 26], fill=(52, 56, 66, 255))        # dark plate
    d.ellipse([5, 12, 27, 24], fill=(66, 72, 84, 255))
    for i in range(5):                                        # fanned slivers
        x = 8 + i * 3
        c = (246, 214, 198, 255) if i % 2 == 0 else (240, 168, 152, 255)
        d.polygon([(x, 20), (x + 4, 12 + (i % 2)), (x + 7, 13 + (i % 2)),
                   (x + 3, 21)], fill=c, outline=shade(c, 0.8))
    for _ in range(5):                                        # roe dots
        x, y = rng.randint(20, 25), rng.randint(19, 22)
        d.point((x, y), fill=(234, 122, 58, 255))
    d.ellipse([6, 20, 9, 23], fill=(110, 160, 70, 255))       # wasabi
    for x in (12, 18, 23):                                    # green garnish
        d.line([x, 22, x + 2, 20], fill=(96, 150, 74, 255))
    return img


# -------------------------------------------------------------- sausages
def _capsule(d, box, fill, outline=None):
    d.rounded_rectangle(box, radius=(box[3] - box[1]) // 2, fill=fill,
                        outline=outline or shade(fill, 0.72))


def sausage_raw(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    red = (188, 76, 64, 255)
    _capsule(d, [5, 9, 27, 16], red)
    _capsule(d, [5, 18, 27, 25], shade(red, 0.94))
    d.line([8, 11, 23, 11], fill=(224, 128, 110, 255))        # sheen
    d.line([8, 20, 23, 20], fill=(218, 118, 100, 255))
    for _ in range(7):                                        # fat speckle
        x, y = rng.randint(7, 25), rng.choice([12, 13, 21, 22])
        d.point((x, y), fill=(240, 214, 198, 255))
    d.rectangle([3, 11, 5, 14], fill=shade(red, 0.8))         # tie ends
    d.rectangle([27, 20, 29, 23], fill=shade(red, 0.8))
    return img


def sausage_wasteland(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    smoked = (130, 76, 50, 255)
    d.arc([6, 6, 27, 27], 300, 240, fill=smoked, width=6)     # smoked ring
    d.arc([6, 6, 27, 27], 300, 240, fill=shade(smoked, 1.18), width=2)
    for a in range(0, 360, 45):                               # wrinkle ticks
        import math
        x = 16 + int(10.5 * math.cos(math.radians(a)))
        y = 16 + int(10.5 * math.sin(math.radians(a)))
        d.point((x, y), fill=shade(smoked, 0.66))
    for _ in range(8):                                        # salt specks
        x, y = rng.randint(8, 24), rng.randint(8, 24)
        if 5 < (x - 16) ** 2 + (y - 16) ** 2 < 130:
            d.point((x, y), fill=(240, 236, 226, 255))
    d.line([22, 7, 25, 5], fill=shade(smoked, 0.8))           # tied end
    return img


def sausage_wasteland_raw(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    lean = (172, 98, 86, 255)
    _capsule(d, [4, 13, 28, 19], lean)
    d.line([7, 15, 25, 15], fill=(204, 140, 126, 255))
    for _ in range(9):                                        # salt cure
        x, y = rng.randint(6, 26), rng.randint(14, 18)
        d.point((x, y), fill=(242, 238, 228, 255))
    d.rectangle([2, 14, 4, 17], fill=shade(lean, 0.8))
    d.rectangle([28, 14, 30, 17], fill=shade(lean, 0.8))
    return img


def sweet_sausage(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    brown = (152, 94, 54, 255)
    _capsule(d, [5, 9, 27, 16], brown)
    _capsule(d, [5, 18, 27, 25], shade(brown, 0.92))
    for y in (11, 20):                                        # maple glaze
        d.line([8, y, 24, y], fill=(236, 176, 96, 255))
        d.line([10, y + 1, 20, y + 1], fill=(250, 226, 170, 255))
    return img


# ---------------------------------------------------------------- dishes
def sausagegravy(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    biscuit = (216, 170, 105, 255)
    d.ellipse([6, 15, 17, 26], fill=biscuit, outline=shade(biscuit, 0.78))
    d.ellipse([16, 15, 27, 26], fill=biscuit, outline=shade(biscuit, 0.78))
    gravy = (219, 211, 201, 255)
    d.ellipse([5, 10, 28, 21], fill=gravy)                    # gravy blanket
    d.polygon([(9, 19), (12, 19), (11, 24), (9, 24)], fill=gravy)  # drips
    d.polygon([(20, 19), (23, 19), (22, 25), (20, 24)], fill=gravy)
    for _ in range(10):                                       # sausage bits
        x, y = rng.randint(8, 25), rng.randint(12, 18)
        d.rectangle([x, y, x + 2, y + 1], fill=(146, 94, 60, 255))
    return img


def scrapple(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    loaf = (152, 102, 64, 255)
    for i, y in enumerate((19, 13, 7)):                        # stacked slices
        d.rectangle([8, y, 25, y + 5], fill=shade(loaf, 1.0 - i * 0.05),
                    outline=(108, 68, 38, 255))
        for _ in range(7):                                    # grain flecks
            x = rng.randint(9, 24)
            c = rng.choice([(224, 184, 96, 255), (110, 72, 44, 255)])
            d.point((x, y + rng.randint(1, 4)), fill=c)
    return img


def sushi_fishroll(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    nori = (30, 48, 36, 255)
    for cx, cy in ((11, 12), (21, 12), (16, 21)):
        d.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], fill=nori)
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4],
                  fill=(246, 243, 236, 255))                 # rice
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2],
                  fill=(240, 152, 140, 255))                  # fish center
    return img


def sushi_meatroll(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    nori = (30, 48, 36, 255)
    d.polygon([(7, 8), (25, 8), (16, 28)], fill=nori,
              outline=shade(nori, 1.5))                       # temaki cone
    d.ellipse([7, 6, 25, 12], fill=(246, 243, 236, 255))      # rice mouth
    for x in (11, 15, 19):                                    # meat strips
        d.polygon([(x, 7), (x + 3, 7), (x + 2, 1), (x, 1)],
                  fill=(188, 72, 60, 255))
    d.polygon([(21, 7), (24, 7), (23, 2)], fill=(96, 150, 74, 255))
    return img


RENDER = {
    "paella_valenciana": paella,
    "pelmeni": pelmeni,
    "pemmican": lambda r: pemmican(r, "berry"),
    "pemmican_meat": lambda r: pemmican(r, "meat"),
    "pemmican_veggy": lambda r: pemmican(r, "veggy"),
    "pork_beans": pork_beans,
    "ravioli": ravioli,
    "rehydrated_fish": rehydrated_fish,
    "rehydrated_froglegs": rehydrated_froglegs,
    "rehydrated_lobster": rehydrated_lobster,
    "rehydrated_meat": rehydrated_meat,
    "rehydrated_mollusk": rehydrated_mollusk,
    "rehydrated_poultry": rehydrated_poultry,
    "royal_beef": royal_beef,
    "sarma": sarma,
    "sashimi": sashimi,
    "sausage_raw": sausage_raw,
    "sausage_wasteland": sausage_wasteland,
    "sausage_wasteland_raw": sausage_wasteland_raw,
    "sausagegravy": sausagegravy,
    "scrapple": scrapple,
    "sushi_fishroll": sushi_fishroll,
    "sushi_meatroll": sushi_meatroll,
    "sweet_sausage": sweet_sausage,
}


def main():
    assert set(RENDER) == set(IDS), set(IDS) ^ set(RENDER)
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = RENDER[item_id](rng)
        assert img.size == (32, 32), item_id
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
    cols, cell = 6, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item11.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item11.png {sheet.size}")


if __name__ == "__main__":
    main()
