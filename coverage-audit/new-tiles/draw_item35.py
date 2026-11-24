#!/usr/bin/env python3
"""Batch ITEM35 (Phase 2g.4 scope extension): jewelry.

43 ids (authoritative list embedded below; source list:
~/workspace/item-batches/35.txt). All 43 resolve to ITEM
entries in data/json/items/armor/jewelry.json. Copy-from
chains resolved against the data before drawing:
badge_fbi / badge_foodkid / pin_badge inherit badge_abstract
(brass, yellow); badge_foodkid overrides to plastic,
pin_badge to steel. faux_fur_cat_ears / faux_fur_cat_tail /
faux_fur_collar inherit the fur_* forms in garish pink
faux fur.

Families, parameterized by metal / material from the data:

- Loop rings (lip / nose, gold & silver): plain closed band
  for lip rings; captive-bead loop (bead seated at the
  bottom of the loop) for nose rings.
- Studs (lip / nose, gold & silver): lip stud = labret
  (flat base disc, post, ball); nose stud = small ball on
  a short bent post with a sparkle.
- Band rings: ring_wedding (gold band, diamond solitaire),
  ring_signet (gold band, wide worn stamp face), ring_purity
  (silver band, engraved ticks + tiny cross), toe rings
  (smaller open band with ball ends, gold & silver).
- Anklets: gold / silver = tiny chain links along an
  ellipse; bat_anklet = dark red cord with bat silhouettes
  and blood-drop charms; flower_anklet = pink cord with
  multicolor flower charms; woven_anklet = braided cord
  segments in bright repeating colors.
- Medals: gold / silver discs on striped ribbons (star
  motif); service_medal = ribbon bar + bronze star.
- Badges: badge_fbi = brass shield, eagle crest, center
  seal; badge_foodkid = yellow plastic star; pin_badge =
  round steel pin-back button.
- Headbands: cat ears (brown fur / pink faux fur / black
  leather), bunny ears (black), devil horns (red plastic),
  angel halo (fluffy pale halo over a band).
- Tails: cat tails in fur / faux fur / leather following
  the same curve; devil_tail = slick red S-curve ending
  in a spade.
- Collars: fur / faux-fur kitty collars with a bell;
  collar_playboy = white detached collar + black bowtie;
  priest_collar = black stand, white band, center tab.
- Singles: cuffs (pair of white shirt cuffs), ear_spool
  (dark plastic ear plug), jade_brooch (green cabochon in
  a brass rim), necklace_teeth (bone teeth on a cord),
  platinum_dental_grill (bright tooth row).
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "angel_halo_headband",
    "badge_fbi",
    "badge_foodkid",
    "bat_anklet",
    "bunny_ears_black",
    "collar_playboy",
    "cuffs",
    "devil_horn_headband",
    "devil_tail",
    "ear_spool",
    "faux_fur_cat_ears",
    "faux_fur_cat_tail",
    "faux_fur_collar",
    "flower_anklet",
    "fur_cat_ears",
    "fur_cat_tail",
    "fur_collar",
    "gold_anklet",
    "gold_lip_ring_loop",
    "gold_lip_ring_stud",
    "gold_medal",
    "gold_nose_ring_loop",
    "gold_nose_ring_stud",
    "gold_toe_ring",
    "jade_brooch",
    "leather_cat_ears",
    "leather_cat_tail",
    "necklace_teeth",
    "pin_badge",
    "platinum_dental_grill",
    "priest_collar",
    "ring_purity",
    "ring_signet",
    "ring_wedding",
    "service_medal",
    "silver_anklet",
    "silver_lip_ring_loop",
    "silver_lip_ring_stud",
    "silver_medal",
    "silver_nose_ring_loop",
    "silver_nose_ring_stud",
    "silver_toe_ring",
    "woven_anklet",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")


def C(rgb):
    return (rgb[0], rgb[1], rgb[2], 255)


def shade(rgb, f):
    return C(tuple(max(0, min(255, int(v * f))) for v in rgb))


def new_sprite():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


# --- palettes: (base, dark, light, outline) ---------------------------------
GOLD = ((214, 176, 66), (158, 120, 32), (250, 226, 132), (116, 86, 18))
SILVER = ((198, 202, 210), (140, 146, 156), (240, 244, 250), (96, 102, 112))
PLATINUM = ((220, 226, 236), (160, 168, 182), (253, 254, 255), (110, 118, 132))
BRASS = ((190, 154, 74), (142, 110, 44), (234, 202, 118), (102, 76, 26))
BRONZE = ((160, 108, 58), (114, 74, 34), (208, 156, 98), (82, 52, 20))
JADE = ((56, 170, 108), (32, 118, 70), (126, 218, 162), (20, 84, 48))
FUR = ((132, 98, 62), (96, 68, 40), (170, 134, 90), (70, 48, 26))
FAUXFUR = ((234, 130, 192), (198, 90, 152), (252, 184, 224), (150, 60, 116))
LEATHER = ((54, 52, 58), (30, 28, 34), (104, 100, 112), (14, 13, 18))
REDPLAS = ((208, 46, 50), (156, 28, 34), (244, 100, 100), (110, 18, 22))
DARKPLAS = ((72, 72, 78), (44, 44, 50), (110, 110, 118), (22, 22, 26))
WHITE = ((238, 238, 240), (196, 196, 202), (255, 255, 255), (150, 150, 158))
BONE = ((232, 224, 200), (196, 184, 152), (250, 246, 230), (160, 148, 112))
BLACK = ((24, 24, 28), (10, 10, 12), (58, 58, 64), (0, 0, 0))


def sparkle(d, x, y, col=(255, 255, 255)):
    c = C(col)
    d.point([(x, y - 2), (x, y + 2), (x - 2, y), (x + 2, y), (x, y)], fill=c)


def fuzz(d, rng, pts, pal, n=26, ln=2):
    """Short fur strokes around a set of anchor points."""
    base, dark, light, _ = pal
    for _ in range(n):
        x, y = pts[rng.randrange(len(pts))]
        dx = rng.choice([-1, 0, 1])
        dy = rng.choice([-1, 0, 1])
        col = C(rng.choice([base, dark, light]))
        d.line([x, y, x + dx * ln, y + dy * ln], fill=col)


def annulus_layer(cx, cy, r_out, r_in, pal):
    """Metal annulus: base fill, dark lower arc, light upper arc."""
    base, dark, light, out = pal
    layer = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    ld.ellipse([cx - r_out, cy - r_out, cx + r_out, cy + r_out], fill=C(out))
    ld.ellipse([cx - r_out + 1, cy - r_out + 1, cx + r_out - 1, cy + r_out - 1],
               fill=C(base))
    mask = Image.new("L", (32, 32), 0)
    md = ImageDraw.Draw(mask)
    md.ellipse([cx - r_in, cy - r_in, cx + r_in, cy + r_in], fill=255)
    clear = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    layer = Image.composite(clear, layer, mask)
    ld = ImageDraw.Draw(layer)
    w = max(1, r_out - r_in - 1)
    rm = (r_out + r_in) // 2
    ld.arc([cx - rm, cy - rm, cx + rm, cy + rm], 20, 150, fill=C(dark), width=w)
    ld.arc([cx - rm, cy - rm, cx + rm, cy + rm], 200, 330, fill=C(light),
           width=w)
    return layer


def ball(d, cx, cy, r, pal):
    base, dark, light, out = pal
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=C(base), outline=C(out))
    d.arc([cx - r, cy - r, cx + r, cy + r], 90, 200, fill=C(dark), width=1)
    d.point([(cx - r // 2, cy - r // 2)], fill=C(light))
    d.ellipse([cx - r // 2, cy - r // 2, cx - r // 2 + 1, cy - r // 2 + 1],
              fill=C(light))


# --- rings -------------------------------------------------------------------

def loop_ring(pal, bead):
    img = new_sprite()
    img.alpha_composite(annulus_layer(16, 16, 10, 7, pal))
    d = ImageDraw.Draw(img)
    if bead:  # captive bead seated at the bottom of the loop
        ball(d, 16, 25, 3, pal)
    sparkle(d, 9, 8)
    return img


def lip_stud(pal):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    # labret: flat base disc (drawn as an ellipse seen edge-on), post, ball
    d.ellipse([10, 23, 22, 27], fill=C(dark), outline=C(out))
    d.rectangle([15, 14, 17, 24], fill=C(base), outline=C(out))
    ball(d, 16, 11, 5, pal)
    sparkle(d, 13, 7)
    return img


def nose_stud(pal):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    # small ball on a short bent post
    d.line([17, 15, 20, 21], fill=C(dark), width=2)
    d.line([20, 21, 23, 21], fill=C(dark), width=2)
    ball(d, 15, 12, 5, pal)
    sparkle(d, 12, 8)
    sparkle(d, 21, 9)
    return img


def toe_ring(pal):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    # open band: thick arc with a gap at the top and ball ends
    d.arc([8, 8, 24, 24], 300, 240, fill=C(out), width=6)
    d.arc([8, 8, 24, 24], 300, 240, fill=C(base), width=4)
    d.arc([9, 7, 25, 23], 300, 380, fill=C(light), width=1)
    for ang in (300, 240):
        x = 16 + int(8 * math.cos(math.radians(ang)))
        y = 16 + int(8 * math.sin(math.radians(ang)))
        ball(d, x, y, 2, pal)
    return img


def band_ring(pal, kind):
    img = new_sprite()
    img.alpha_composite(annulus_layer(16, 18, 9, 6, pal))
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    if kind == "wedding":
        # prongs + faceted diamond solitaire at the top of the band
        d.polygon([(13, 10), (19, 10), (17, 13), (15, 13)], fill=C(base))
        d.polygon([(16, 2), (21, 6), (19, 11), (13, 11), (11, 6)],
                  fill=C((214, 232, 248)), outline=C((140, 170, 200)))
        d.line([11, 6, 21, 6], fill=C((140, 170, 200)))
        d.line([16, 2, 13, 11], fill=C((170, 200, 230)))
        d.line([16, 2, 19, 11], fill=C((170, 200, 230)))
        sparkle(d, 22, 3)
    elif kind == "signet":
        # wide flat stamp face, worn swirl
        d.ellipse([10, 3, 22, 12], fill=C(SILVER[0]), outline=C(out))
        d.arc([12, 5, 20, 10], 20, 300, fill=C(SILVER[1]), width=1)
        d.arc([14, 6, 18, 9], 200, 480, fill=C(SILVER[1]), width=1)
    elif kind == "purity":
        # engraved tick marks around the front of the band + tiny cross
        for x in range(9, 24, 3):
            d.line([x, 24, x, 26], fill=C(dark), width=1)
        d.line([16, 8, 16, 12], fill=C(dark), width=1)
        d.line([14, 10, 18, 10], fill=C(dark), width=1)
        sparkle(d, 10, 10)
    return img


# --- anklets -----------------------------------------------------------------

def ellipse_pt(cx, cy, rx, ry, deg):
    a = math.radians(deg)
    return (cx + int(rx * math.cos(a)), cy + int(ry * math.sin(a)))


def chain_anklet(pal):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    for deg in range(0, 360, 18):
        x, y = ellipse_pt(16, 15, 11, 8, deg)
        col = light if (deg // 18) % 2 == 0 else base
        d.ellipse([x - 2, y - 1, x + 2, y + 1], fill=C(col), outline=C(out))
    x, y = ellipse_pt(16, 15, 11, 8, 90)
    ball(d, x, y + 2, 2, pal)
    return img


def woven_anklet(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    cols = [(216, 60, 54), (240, 150, 50), (240, 210, 70),
            (90, 180, 90), (70, 130, 210), (150, 90, 200)]
    for i, deg in enumerate(range(0, 360, 15)):
        x, y = ellipse_pt(16, 15, 11, 8, deg)
        d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=C(cols[i % len(cols)]),
                    outline=C((40, 40, 44)))
    # tassel ends at the bottom
    d.line([14, 24, 13, 28], fill=C(cols[0]), width=2)
    d.line([18, 24, 19, 28], fill=C(cols[3]), width=2)
    return img


def bat_charm(d, x, y):
    # tiny bat silhouette: scalloped wings + body
    d.polygon([(x - 5, y - 1), (x - 3, y - 3), (x - 1, y - 1), (x, y - 3),
               (x + 1, y - 1), (x + 3, y - 3), (x + 5, y - 1), (x + 3, y + 2),
               (x + 1, y + 1), (x, y + 3), (x - 1, y + 1), (x - 3, y + 2)],
              fill=C(BLACK[0]), outline=C(BLACK[3]))


def drop_charm(d, x, y, col=(196, 30, 36)):
    d.polygon([(x, y - 3), (x + 2, y + 1), (x, y + 3), (x - 2, y + 1)],
              fill=C(col))
    d.point([(x - 1, y + 1)], fill=C((244, 120, 120)))


def flower_charm(d, rng, x, y):
    petal = rng.choice([(240, 130, 180), (250, 220, 90), (240, 240, 245),
                        (140, 190, 240), (240, 110, 90)])
    for dx, dy in [(0, -2), (2, 0), (0, 2), (-2, 0)]:
        d.ellipse([x + dx - 1, y + dy - 1, x + dx + 1, y + dy + 1],
                  fill=C(petal))
    d.point([(x, y)], fill=C((250, 180, 60)))


def charm_anklet(rng, kind):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    cord = (120, 40, 44) if kind == "bat" else (226, 130, 170)
    d.ellipse([5, 7, 27, 23], outline=C(cord), width=2)
    for deg in (55, 90, 125):
        x, y = ellipse_pt(16, 15, 11, 8, deg)
        if kind == "bat":
            if deg == 90:
                drop_charm(d, x, y + 3)
            else:
                bat_charm(d, x, y + 3)
        else:
            flower_charm(d, rng, x, y + 3)
    if kind == "bat":
        drop_charm(d, *ellipse_pt(16, 15, 11, 8, 20), )
        x, y = ellipse_pt(16, 15, 11, 8, 160)
        drop_charm(d, x, y)
    return img


# --- medals & badges ---------------------------------------------------------

def medal(pal, ribbon_cols, star=True):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    # ribbon: three vertical stripes tapering to the disc
    x0 = 11
    for i, col in enumerate(ribbon_cols):
        d.polygon([(x0 + i * 4, 3), (x0 + i * 4 + 4, 3),
                   (x0 + i * 4 + 3, 13), (x0 + i * 4 + 1, 13)], fill=C(col))
    d.line([11, 3, 23, 3], fill=C((90, 90, 96)), width=1)
    # suspension ring + disc
    d.ellipse([14, 12, 18, 16], outline=C(out), width=2)
    d.ellipse([9, 15, 23, 29], fill=C(base), outline=C(out))
    d.ellipse([11, 17, 21, 27], outline=C(dark))
    if star:
        cx, cy = 16, 22
        pts = []
        for i in range(10):
            r = 4 if i % 2 == 0 else 2
            a = math.radians(-90 + i * 36)
            pts.append((cx + int(r * math.cos(a)), cy + int(r * math.sin(a))))
        d.polygon(pts, fill=C(light), outline=C(dark))
    sparkle(d, 11, 17)
    return img


def service_medal():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = BRONZE
    # ribbon bar: horizontal stripes
    cols = [(150, 40, 44), (240, 240, 240), (52, 84, 150),
            (240, 240, 240), (150, 40, 44)]
    for i, col in enumerate(cols):
        d.rectangle([8 + i * 3, 5, 11 + i * 3, 11], fill=C(col))
    d.rectangle([8, 5, 23, 11], outline=C((70, 70, 76)))
    d.line([15, 11, 16, 14], fill=C(dark), width=2)
    # bronze star pendant
    cx, cy = 16, 21
    pts = []
    for i in range(10):
        r = 7 if i % 2 == 0 else 3
        a = math.radians(-90 + i * 36)
        pts.append((cx + int(r * math.cos(a)), cy + int(r * math.sin(a))))
    d.polygon(pts, fill=C(base), outline=C(out))
    d.ellipse([14, 19, 18, 23], fill=C(light), outline=C(dark))
    return img


def badge_fbi():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = BRASS
    # eagle crest: spread wings + body above the shield
    d.polygon([(6, 8), (13, 5), (15, 8), (10, 10)], fill=C(dark))
    d.polygon([(26, 8), (19, 5), (17, 8), (22, 10)], fill=C(dark))
    d.ellipse([14, 4, 18, 9], fill=C(base), outline=C(out))
    # shield
    d.polygon([(9, 9), (23, 9), (24, 15), (22, 23), (16, 28), (10, 23),
               (8, 15)], fill=C(base), outline=C(out))
    d.polygon([(11, 11), (21, 11), (22, 15), (20, 21), (16, 25), (12, 21),
               (10, 15)], outline=C(dark))
    # Lady Justice: column figure with scale beam
    d.line([13, 14, 19, 14], fill=C(light), width=1)
    d.line([16, 13, 16, 20], fill=C(light), width=1)
    d.polygon([(13, 14), (12, 17), (14, 17)], outline=C(light))
    d.polygon([(19, 14), (18, 17), (20, 17)], outline=C(light))
    d.ellipse([15, 20, 17, 22], fill=C(light))
    return img


def badge_foodkid():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    cx, cy = 16, 16
    pts = []
    for i in range(10):
        r = 12 if i % 2 == 0 else 5
        a = math.radians(-90 + i * 36)
        pts.append((cx + int(r * math.cos(a)), cy + int(r * math.sin(a))))
    d.polygon(pts, fill=C((240, 200, 60)), outline=C((200, 60, 50)))
    d.ellipse([12, 12, 20, 20], fill=C((230, 70, 60)), outline=C((170, 40, 36)))
    # bold lettering suggested by three blocky strokes
    d.rectangle([14, 14, 18, 15], fill=C((250, 240, 220)))
    d.rectangle([14, 16, 16, 18], fill=C((250, 240, 220)))
    return img


def pin_badge():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = SILVER
    d.ellipse([6, 6, 26, 26], fill=C(dark), outline=C(out))
    d.ellipse([8, 8, 24, 24], fill=C(base), outline=C(dark))
    # band-pin motif: dark disc with a lightning slash
    d.ellipse([11, 11, 21, 21], fill=C((44, 44, 52)), outline=C(out))
    d.polygon([(17, 12), (14, 17), (16, 17), (15, 20), (19, 15), (16, 15)],
              fill=C((250, 210, 80)))
    d.arc([8, 8, 24, 24], 200, 300, fill=C(light), width=2)
    return img


# --- headbands ---------------------------------------------------------------

def headband(d, pal, cy=24):
    base, dark, light, out = pal
    d.arc([4, 12, 28, 30], 180, 360, fill=C(out), width=4)
    d.arc([4, 12, 28, 30], 180, 360, fill=C(base), width=2)


def cat_ears(rng, pal, inner, fuzzy_strokes):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    headband(d, pal)
    anchors = []
    for cx in (9, 23):
        d.polygon([(cx - 6, 15), (cx + 6, 15), (cx + 2, 3), (cx - 1, 2)],
                  fill=C(base), outline=C(out))
        d.ellipse([cx - 2, 1, cx + 3, 5], fill=C(base))
        d.polygon([(cx - 3, 14), (cx + 4, 14), (cx + 1, 6)],
                  fill=C(inner))
        anchors += [(cx - 5, 13), (cx + 5, 13), (cx, 3), (cx - 2, 8),
                    (cx + 3, 8)]
    if fuzzy_strokes:
        fuzz(d, rng, anchors, pal, n=34, ln=2)
    else:  # leather: glossy sheen streaks
        d.line([7, 12, 10, 5], fill=C(light), width=1)
        d.line([21, 12, 24, 5], fill=C(light), width=1)
    return img


def bunny_ears(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = LEATHER
    base, dark, light, out = pal
    headband(d, pal)
    anchors = []
    for cx, tilt in ((10, -3), (22, 3)):
        pts = []
        for t in range(0, 17):
            a = math.radians(-90 + tilt * 4)
            ex = cx + int(t * math.cos(a) * 0.35 + tilt * t * 0.06)
            ey = 16 - t
            pts.append((ex, ey))
        for (ex, ey), w in zip(pts, [3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 1,
                                    1, 1, 1, 0]):
            d.ellipse([ex - w, ey - 1, ex + w, ey + 1], fill=C(base),
                      outline=C(out) if w >= 2 else None)
            anchors.append((ex, ey))
        # inner ear
        d.line([cx + tilt // 2, 13, cx + tilt, 5], fill=C((150, 112, 118)),
               width=2)
    fuzz(d, rng, anchors, pal, n=22, ln=2)
    return img


def devil_horns():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = REDPLAS
    base, dark, light, out = pal
    headband(d, pal)
    for cx, flip in ((9, 1), (23, -1)):
        pts = [(cx - 5 * flip, 16), (cx - 3 * flip, 7), (cx - 1 * flip, 3),
               (cx + 2 * flip, 2), (cx + 2 * flip, 7), (cx + 4 * flip, 16)]
        d.polygon(pts, fill=C(base), outline=C(out))
        d.line([cx, 13, cx + 1 * flip, 6], fill=C(light), width=1)
    return img


def angel_halo(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    headband(d, DARKPLAS)
    cream = [(252, 246, 214), (244, 232, 178), (255, 252, 236)]
    for i, col in enumerate(cream):
        d.ellipse([7 + i, 3 + i // 2, 25 - i, 11 - i // 2], outline=C(col),
                  width=2)
    anchors = [(x, 4) for x in range(8, 25, 3)] + \
              [(x, 10) for x in range(8, 25, 3)] + [(7, 7), (25, 7)]
    fuzz(d, rng, anchors, ((250, 242, 200), (232, 216, 150),
                           (255, 252, 236), (200, 180, 110)), n=30, ln=2)
    # support struts from band to halo
    d.line([11, 12, 12, 15], fill=C(DARKPLAS[1]), width=1)
    d.line([21, 12, 20, 15], fill=C(DARKPLAS[1]), width=1)
    return img


# --- tails -------------------------------------------------------------------

def tail_curve():
    pts = []
    for t in range(0, 21):
        f = t / 20.0
        x = 6 + int(18 * f)
        y = 5 + int(20 * f * f) + int(4 * math.sin(f * math.pi))
        pts.append((x, y))
    return pts


def cat_tail(rng, pal, kind):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    pts = tail_curve()
    for i, (x, y) in enumerate(pts):
        r = 4 if i < 14 else 3
        d.ellipse([x - r, y - r, x + r, y + r], fill=C(base))
    # bead weight near the tip (fur / faux fur)
    if kind in ("fur", "faux"):
        x, y = pts[-1]
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=C(DARKPLAS[0]),
                  outline=C(DARKPLAS[3]))
    if kind == "leather":
        for x, y in pts[::4]:
            d.point([(x - 1, y - 2)], fill=C(light))
            d.ellipse([x - 2, y - 3, x, y - 1], fill=C(light))
    else:
        fuzz(d, rng, pts, pal, n=44, ln=3)
        # darker tip band for the fur tail
        if kind == "fur":
            x, y = pts[-2]
            d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=C(dark))
    return img


def devil_tail():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = REDPLAS
    pts = []
    for t in range(0, 21):
        f = t / 20.0
        x = 5 + int(14 * f) + int(5 * math.sin(f * math.pi * 1.4))
        y = 4 + int(17 * f)
        pts.append((x, y))
    for i, (x, y) in enumerate(pts):
        r = 3 if i < 16 else 2
        d.ellipse([x - r, y - r, x + r, y + r], fill=C(base))
        if i % 3 == 0:
            d.point([(x - 1, y - 1)], fill=C(light))
    # spade tip
    x, y = pts[-1]
    d.polygon([(x, y + 9), (x - 5, y + 1), (x - 1, y - 1), (x, y + 3),
               (x + 1, y - 1), (x + 5, y + 1)], fill=C(base), outline=C(out))
    return img


# --- collars -----------------------------------------------------------------

def kitty_collar(rng, pal):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = pal
    d.arc([3, 6, 29, 26], 0, 180, fill=C(out), width=7)
    d.arc([3, 6, 29, 26], 0, 180, fill=C(base), width=5)
    anchors = [(x, 20) for x in range(6, 27, 3)] + [(6, 12), (26, 12)]
    fuzz(d, rng, anchors, pal, n=30, ln=2)
    # bell at the front
    d.line([16, 21, 16, 23], fill=C(GOLD[1]), width=2)
    ball(d, 16, 26, 4, GOLD)
    d.line([13, 27, 19, 27], fill=C(GOLD[3]), width=1)
    return img


def collar_playboy():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = WHITE
    # detached collar: band with two pointed flaps forming a V
    d.polygon([(5, 10), (27, 10), (27, 15), (17, 15), (16, 19), (15, 15),
               (5, 15)], fill=C(base), outline=C(out))
    d.polygon([(6, 11), (14, 11), (15, 14), (16, 17), (13, 14), (6, 14)],
              fill=C(light))
    d.polygon([(26, 11), (18, 11), (17, 14), (16, 17), (19, 14), (26, 14)],
              fill=C(light))
    # black bowtie at the throat
    d.polygon([(16, 20), (9, 16), (9, 24)], fill=C(BLACK[0]),
              outline=C(BLACK[3]))
    d.polygon([(16, 20), (23, 16), (23, 24)], fill=C(BLACK[0]),
              outline=C(BLACK[3]))
    d.rectangle([14, 18, 18, 22], fill=C(BLACK[2]), outline=C(BLACK[3]))
    return img


def priest_collar():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    # black collar stand with the white band and center tab
    d.rectangle([5, 11, 27, 21], fill=C(BLACK[0]), outline=C(BLACK[3]))
    d.rectangle([5, 12, 27, 17], fill=C(WHITE[0]), outline=C(WHITE[3]))
    d.rectangle([14, 12, 18, 20], fill=C(WHITE[2]), outline=C(WHITE[3]))
    d.line([6, 18, 13, 18], fill=C(BLACK[2]), width=1)
    d.line([19, 18, 26, 18], fill=C(BLACK[2]), width=1)
    return img


# --- singles -----------------------------------------------------------------

def cuffs():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = WHITE
    for x0 in (4, 17):
        d.rectangle([x0, 9, x0 + 11, 23], fill=C(base), outline=C(out))
        d.line([x0, 13, x0 + 11, 13], fill=C(dark), width=1)
        d.line([x0, 20, x0 + 11, 20], fill=C(dark), width=1)
        # button + stitch dashes
        d.ellipse([x0 + 4, 15, x0 + 7, 18], fill=C(light), outline=C(out))
        for y in (10, 22):
            d.line([x0 + 1, y, x0 + 10, y], fill=C(dark), width=1)
    return img


def ear_spool():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = DARKPLAS
    d.ellipse([6, 6, 26, 26], fill=C(base), outline=C(out))
    d.ellipse([9, 9, 23, 23], fill=C(dark), outline=C(out))
    d.ellipse([12, 12, 20, 20], fill=C(base), outline=C(out))
    d.ellipse([14, 14, 18, 18], fill=C(BLACK[0]))
    d.arc([6, 6, 26, 26], 190, 300, fill=C(light), width=2)
    sparkle(d, 10, 9)
    return img


def jade_brooch():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = JADE
    # brass rim + carved oval cabochon
    d.ellipse([5, 9, 27, 24], fill=C(BRASS[0]), outline=C(BRASS[3]))
    d.ellipse([7, 11, 25, 22], fill=C(base), outline=C(out))
    d.arc([10, 12, 22, 21], 30, 300, fill=C(light), width=2)
    d.arc([13, 14, 19, 19], 200, 480, fill=C(dark), width=1)
    d.ellipse([10, 12, 14, 15], fill=C(light))
    # pin clasp hint at the top
    d.rectangle([14, 6, 18, 9], fill=C(BRASS[1]), outline=C(BRASS[3]))
    return img


def necklace_teeth():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.ellipse([4, 3, 28, 21], outline=C((104, 78, 48)), width=2)
    # teeth hanging along the bottom arc, longest at the center
    for deg, ln in ((50, 5), (70, 7), (90, 9), (110, 7), (130, 5),
                    (60, 6), (120, 6)):
        x, y = ellipse_pt(16, 12, 12, 9, deg)
        d.polygon([(x - 2, y), (x + 2, y), (x, y + ln)], fill=C(BONE[0]),
                  outline=C(BONE[3]))
        d.line([x, y + 2, x, y + ln - 2], fill=C(BONE[1]), width=1)
    return img


def dental_grill():
    img = new_sprite()
    d = ImageDraw.Draw(img)
    base, dark, light, out = PLATINUM
    # tooth row: band + scalloped bottom edge
    d.rectangle([6, 11, 26, 17], fill=C(base), outline=C(out))
    for x in range(8, 26, 4):
        d.ellipse([x - 2, 15, x + 2, 20], fill=C(base), outline=C(out))
    for x in range(10, 26, 4):
        d.line([x, 12, x, 18], fill=C(dark), width=1)
    d.line([7, 12, 25, 12], fill=C(light), width=1)
    sparkle(d, 9, 9)
    sparkle(d, 23, 21)
    return img


# --- dispatch ----------------------------------------------------------------

def build(item_id):
    rng = random.Random(zlib.crc32(item_id.encode()))
    b = BUILDERS[item_id]
    img = b(rng) if b in (woven_anklet, charm_anklet_bat, charm_anklet_flower,
                          cat_ears_fur, cat_ears_faux, cat_ears_leather,
                          bunny_ears_wrap, angel_halo_wrap, kitty_collar_fur,
                          kitty_collar_faux, cat_tail_fur, cat_tail_faux,
                          cat_tail_leather) else b()
    assert img.size == (32, 32)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (item_id, alphas)
    return img


def charm_anklet_bat(rng):
    return charm_anklet(rng, "bat")


def charm_anklet_flower(rng):
    return charm_anklet(rng, "flower")


def cat_ears_fur(rng):
    return cat_ears(rng, FUR, (216, 178, 150), True)


def cat_ears_faux(rng):
    return cat_ears(rng, FAUXFUR, (252, 214, 236), True)


def cat_ears_leather(rng):
    return cat_ears(rng, LEATHER, (88, 86, 94), False)


def bunny_ears_wrap(rng):
    return bunny_ears(rng)


def angel_halo_wrap(rng):
    return angel_halo(rng)


def kitty_collar_fur(rng):
    return kitty_collar(rng, FUR)


def kitty_collar_faux(rng):
    return kitty_collar(rng, FAUXFUR)


def cat_tail_fur(rng):
    return cat_tail(rng, FUR, "fur")


def cat_tail_faux(rng):
    return cat_tail(rng, FAUXFUR, "faux")


def cat_tail_leather(rng):
    return cat_tail(rng, LEATHER, "leather")


BUILDERS = {
    "angel_halo_headband": angel_halo_wrap,
    "badge_fbi": badge_fbi,
    "badge_foodkid": badge_foodkid,
    "bat_anklet": charm_anklet_bat,
    "bunny_ears_black": bunny_ears_wrap,
    "collar_playboy": collar_playboy,
    "cuffs": cuffs,
    "devil_horn_headband": devil_horns,
    "devil_tail": devil_tail,
    "ear_spool": ear_spool,
    "faux_fur_cat_ears": cat_ears_faux,
    "faux_fur_cat_tail": cat_tail_faux,
    "faux_fur_collar": kitty_collar_faux,
    "flower_anklet": charm_anklet_flower,
    "fur_cat_ears": cat_ears_fur,
    "fur_cat_tail": cat_tail_fur,
    "fur_collar": kitty_collar_fur,
    "gold_anklet": lambda: chain_anklet(GOLD),
    "gold_lip_ring_loop": lambda: loop_ring(GOLD, False),
    "gold_lip_ring_stud": lambda: lip_stud(GOLD),
    "gold_medal": lambda: medal(GOLD, [(200, 50, 55), (240, 240, 240),
                                       (52, 84, 160)]),
    "gold_nose_ring_loop": lambda: loop_ring(GOLD, True),
    "gold_nose_ring_stud": lambda: nose_stud(GOLD),
    "gold_toe_ring": lambda: toe_ring(GOLD),
    "jade_brooch": jade_brooch,
    "leather_cat_ears": cat_ears_leather,
    "leather_cat_tail": cat_tail_leather,
    "necklace_teeth": necklace_teeth,
    "pin_badge": pin_badge,
    "platinum_dental_grill": dental_grill,
    "priest_collar": priest_collar,
    "ring_purity": lambda: band_ring(SILVER, "purity"),
    "ring_signet": lambda: band_ring(GOLD, "signet"),
    "ring_wedding": lambda: band_ring(GOLD, "wedding"),
    "service_medal": service_medal,
    "silver_anklet": lambda: chain_anklet(SILVER),
    "silver_lip_ring_loop": lambda: loop_ring(SILVER, False),
    "silver_lip_ring_stud": lambda: lip_stud(SILVER),
    "silver_medal": lambda: medal(SILVER, [(120, 170, 220), (240, 240, 240),
                                           (120, 170, 220)]),
    "silver_nose_ring_loop": lambda: loop_ring(SILVER, True),
    "silver_nose_ring_stud": lambda: nose_stud(SILVER),
    "silver_toe_ring": lambda: toe_ring(SILVER),
    "woven_anklet": woven_anklet,
}


def main():
    assert len(IDS) == 43, len(IDS)
    assert set(IDS) == set(BUILDERS), set(IDS) ^ set(BUILDERS)
    for item_id in IDS:
        img = build(item_id)
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols = 8
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * 36, rows * 36), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        img = build(item_id)
        sheet.alpha_composite(img, ((i % cols) * 36 + 2, (i // cols) * 36 + 2))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item35.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item35.png {sheet.size}")


if __name__ == "__main__":
    main()
