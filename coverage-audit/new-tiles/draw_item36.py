#!/usr/bin/env python3
"""Batch ITEM36 (Phase 2g.4 scope extension): raw
vegetables -- 40 ids, mostly from
data/json/items/comestibles/raw_veggy.json
(authoritative list embedded below; source list:
~/workspace/item-batches/36.txt).

Every entry was read before drawing; copy-from chains
resolved (tomato_cut <- tomato: red; veggy is the ITEM
"triffid flesh" in raw_veggy.json, not the materials.json
material of the same id; pea_pods' description is a
copy-paste of kale's in the data, but its name is "pea
pods" and it is drawn as pods).

Drawn as the whole unprepared produce, as the entries
describe:
- WHOLE specimens: raw_artichoke (layered bud) with its
  pale artichoke_heart, cauliflower_head (white curd dome)
  vs. the shredded handful cauliflower, kale_head (curly
  rosette) vs. shredded kale, lettuce_head (big pale
  iceberg ball), brussels_sprout (one mini cabbage) vs.
  brussels_sprout_stalk (stalk studded with sprouts),
  gherkin (small bumpy cucumber), plantain (angular
  cooking banana), horseradish_root (long white taper),
  sugar_beet (big ringed root), jerusalem_artichoke
  (knobby tuber), salsify_raw (thin dark taproot),
  wild_sarsaparilla_root (stringy rhizome tangle),
  raw_asparagus (banded spear bunch), lotus_rhizome
  (segment chain, cut face showing the hole pattern),
  plant_stalk and japanese_knotweed_stem (segmented
  stalks; knotweed red-speckled).
- CUT/CHOPPED piles: bell_pepper_cut, chili_pepper_cut,
  cucumber_cut, gherkin_cut, tomato_cut (wedges, seeds
  removed), zucchini_cut, pumpkin_cut (hollowed quarter
  wedge).
- HANDFULS/PILES: pea (round peas), pea_pods (one split
  showing the row), raw_beans (kidney beans), chicory /
  horseradish / wintergreen leaf bunches, flour_wheat_free
  (white flour spilling from its small box).
- SEAWEEDS: bladderwrack (brown fronds with round air
  bladders), kelp_sugar (one wide brown ribbon),
  sea_lettuce (bright ruffled green sheets), seaweed
  (small slimy tangle) vs. seaweed_pile (big heap).
- veggy: triffid flesh -- a yellow-green fibrous chunk
  leaking clear fluid (pale opaque drip).

All sprite alpha is in {0, 255}, asserted per sprite.
Writes ONLY its own 40 folders + preview_item36.png.
PIL only; per-sprite random.Random(crc32(id)).
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
    "artichoke_heart",
    "bell_pepper_cut",
    "bladderwrack",
    "brussels_sprout",
    "brussels_sprout_stalk",
    "cauliflower",
    "cauliflower_head",
    "chicory_leaves",
    "chili_pepper_cut",
    "cucumber_cut",
    "flour_wheat_free",
    "gherkin",
    "gherkin_cut",
    "horseradish_greens",
    "horseradish_root",
    "japanese_knotweed_stem",
    "jerusalem_artichoke",
    "kale",
    "kale_head",
    "kelp_sugar",
    "lettuce_head",
    "lotus_rhizome",
    "pea",
    "pea_pods",
    "plant_stalk",
    "plantain",
    "pumpkin_cut",
    "raw_artichoke",
    "raw_asparagus",
    "raw_beans",
    "salsify_raw",
    "sea_lettuce",
    "seaweed",
    "seaweed_pile",
    "sugar_beet",
    "tomato_cut",
    "veggy",
    "wild_sarsaparilla_root",
    "wintergreen_leaves",
    "zucchini_cut",
]


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def rpoly(cx, cy, pts, ang):
    """Rotate a point list around (cx, cy) by ang degrees."""
    a = math.radians(ang)
    ca, sa = math.cos(a), math.sin(a)
    return [(cx + (x - cx) * ca - (y - cy) * sa,
             cy + (x - cx) * sa + (y - cy) * ca) for x, y in pts]


def leaf(d, cx, cy, ln, wd, ang, fill, vein=None):
    """A pointed leaf: lens shape along direction ang."""
    tip = (cx + math.cos(math.radians(ang)) * ln,
           cy + math.sin(math.radians(ang)) * ln)
    px = math.cos(math.radians(ang + 90)) * wd
    py = math.sin(math.radians(ang + 90)) * wd
    mid = (cx + (tip[0] - cx) * 0.5, cy + (tip[1] - cy) * 0.5)
    pts = [(cx, cy), (mid[0] + px, mid[1] + py), tip,
           (mid[0] - px, mid[1] - py)]
    d.polygon(pts, fill=fill)
    if vein:
        d.line([(cx, cy), tip], fill=vein, width=1)


def speckles(d, rng, box, n, colors, rmax=1):
    x0, y0, x1, y1 = box
    for _ in range(n):
        x = rng.uniform(x0, x1)
        y = rng.uniform(y0, y1)
        r = rng.uniform(0.4, rmax)
        d.ellipse([x - r, y - r, x + r, y + r],
                  fill=rng.choice(colors))


# ---------------------------------------------------------------- whole veg

def draw_raw_artichoke(rng):
    img, d = canvas()
    # short stem
    d.rectangle([14, 26, 18, 30], fill=(96, 110, 62, 255))
    # layered scale rows, bud tapering to a point at top
    rows = [
        (24, 26, 9.5, (88, 104, 58, 255)),
        (19, 23, 8.0, (99, 118, 63, 255)),
        (14, 20, 6.5, (110, 130, 70, 255)),
        (10, 17, 5.0, (120, 140, 78, 255)),
        (6, 14, 3.6, (131, 150, 88, 255)),
    ]
    for top, bot, hw, col in rows:
        n = 5
        for i in range(n):
            cx = 16 + (i - (n - 1) / 2) * (2 * hw / n) * 1.55
            d.polygon([(cx - hw / n * 1.25, bot), (cx, top),
                       (cx + hw / n * 1.25, bot)], fill=col)
            d.line([(cx, top), (cx, bot - 2)], fill=shade(col, 0.8))
    d.polygon([(13.5, 8), (16, 3), (18.5, 8)], fill=(140, 158, 96, 255))
    return img


def draw_artichoke_heart(rng):
    img, d = canvas()
    # pale tender heart: rounded cone of soft inner leaves
    d.polygon([(9, 27), (10, 12), (13, 6), (19, 6), (22, 12), (23, 27)],
              fill=(196, 205, 150, 255))
    for i, x in enumerate((12.5, 16, 19.5)):
        d.polygon([(x - 2.6, 26), (x, 8 + (i % 2) * 2), (x + 2.6, 26)],
                  fill=(208, 216, 168, 255) if i != 1 else (186, 198, 138, 255))
        d.line([(x, 9), (x, 25)], fill=(168, 182, 118, 255))
    # choke fuzz at the top center
    speckles(d, rng, (13, 5, 19, 9), 8, [(226, 224, 190, 255)])
    d.rectangle([11, 27, 21, 29], fill=(178, 190, 128, 255))
    return img


def draw_cauliflower_head(rng):
    img, d = canvas()
    # base leaves
    leaf(d, 9, 27, 9, 3.4, 160, (92, 138, 62, 255))
    leaf(d, 23, 27, 9, 3.4, 20, (92, 138, 62, 255))
    leaf(d, 16, 28, 8, 3.2, 90, (104, 150, 70, 255))
    # curd dome: clustered bumps
    bumps = [(16, 12, 8.6), (9, 17, 5.6), (23, 17, 5.6), (12, 21, 6.2),
             (20, 21, 6.2), (16, 20, 7.0), (8, 13, 3.6), (24, 13, 3.6),
             (13, 10, 4.4), (19, 10, 4.4)]
    for cx, cy, r in bumps:
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(238, 236, 224, 255))
    for cx, cy, r in bumps:
        for _ in range(4):
            x = cx + rng.uniform(-r * 0.6, r * 0.6)
            y = cy + rng.uniform(-r * 0.6, r * 0.6)
            d.ellipse([x - 1.2, y - 1.2, x + 1.2, y + 1.2],
                      fill=(222, 220, 202, 255))
    return img


def draw_cauliflower(rng):
    img, d = canvas()
    # handful of shredded cauliflower: small floret bits
    for _ in range(16):
        cx = rng.uniform(7, 25)
        cy = rng.uniform(12, 26)
        r = rng.uniform(1.8, 3.4)
        d.ellipse([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8],
                  fill=rng.choice([(240, 238, 226, 255), (230, 228, 210, 255)]))
        d.line([(cx, cy + r * 0.6), (cx, cy + r * 1.5)],
               fill=(214, 212, 194, 255))
    return img


def draw_kale_head(rng):
    img, d = canvas()
    # rosette of dark curly leaves
    for ang in range(0, 360, 36):
        leaf(d, 16, 18, 11, 4.6, ang - 90, (46, 96, 40, 255),
             vein=(70, 128, 58, 255))
    for ang in range(18, 360, 45):
        leaf(d, 16, 17, 8, 3.8, ang - 90, (56, 112, 46, 255),
             vein=(84, 146, 66, 255))
    d.ellipse([12, 13, 20, 21], fill=(66, 124, 52, 255))
    # curly edge dots
    for ang in range(0, 360, 20):
        x = 16 + math.cos(math.radians(ang)) * rng.uniform(9, 12)
        y = 17 + math.sin(math.radians(ang)) * rng.uniform(8, 11)
        d.ellipse([x - 1.3, y - 1.3, x + 1.3, y + 1.3], fill=(40, 88, 36, 255))
    return img


def draw_kale(rng):
    img, d = canvas()
    # handful of shredded kale: dark curly shreds
    for _ in range(15):
        cx = rng.uniform(6, 26)
        cy = rng.uniform(9, 26)
        ang = rng.uniform(0, 360)
        leaf(d, cx, cy, rng.uniform(4, 7), rng.uniform(1.4, 2.2), ang,
             rng.choice([(46, 96, 40, 255), (56, 112, 46, 255),
                         (40, 86, 36, 255)]))
    return img


def draw_lettuce_head(rng):
    img, d = canvas()
    # big crisp iceberg head
    d.ellipse([4, 6, 28, 29], fill=(168, 208, 130, 255))
    d.arc([6, 8, 27, 28], 200, 520, fill=(140, 188, 104, 255), width=2)
    d.arc([9, 10, 25, 27], 180, 500, fill=(186, 220, 148, 255), width=2)
    d.arc([12, 13, 23, 25], 160, 480, fill=(150, 196, 112, 255), width=1)
    speckles(d, rng, (7, 9, 25, 26), 12, [(196, 228, 158, 255)], 1.1)
    d.ellipse([13, 26, 19, 30], fill=(196, 214, 160, 255))  # cut stem base
    return img


def draw_brussels_sprout(rng):
    img, d = canvas()
    # one miniature cabbage, drawn large; a second small one behind
    d.ellipse([19, 19, 29, 29], fill=(88, 142, 60, 255))
    d.arc([20, 20, 28, 28], 0, 360, fill=(70, 122, 48, 255))
    d.ellipse([5, 7, 25, 27], fill=(104, 158, 66, 255))
    d.arc([7, 9, 23, 25], 300, 620, fill=(80, 134, 52, 255), width=2)
    d.arc([9, 10, 22, 24], 260, 560, fill=(124, 176, 82, 255), width=2)
    d.arc([11, 12, 20, 22], 220, 520, fill=(88, 144, 58, 255), width=1)
    d.ellipse([13, 25, 19, 28], fill=(196, 210, 150, 255))  # stem scar
    return img


def draw_brussels_sprout_stalk(rng):
    img, d = canvas()
    # thick pale diagonal stalk with round sprouts attached on
    # short stems, alternating sides
    d.line([(7, 29), (25, 4)], fill=(150, 186, 110, 255), width=6)
    d.line([(7, 29), (25, 4)], fill=(170, 204, 130, 255), width=2)
    sprouts = [(6, 22, -1), (13, 25, 1), (12, 16, -1), (19, 18, 1),
               (18, 10, -1), (24, 11, 1), (24, 5, -1)]
    for cx, cy, side in sprouts:
        r = rng.uniform(3.2, 4.0)
        d.line([(cx - side * r * 0.4, cy), (cx - side * (r + 1.5), cy + 1)],
               fill=(140, 178, 100, 255), width=2)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(104, 158, 66, 255))
        d.arc([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1], 300, 620,
              fill=(80, 134, 52, 255))
        d.arc([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2], 260, 560,
              fill=(126, 178, 84, 255))
    return img


def draw_gherkin(rng):
    img, d = canvas()
    # small bumpy cucumber, diagonal
    body = [(7, 21), (11, 12), (17, 7), (23, 6), (26, 9), (24, 15),
            (17, 22), (11, 25), (7, 24)]
    d.polygon(body, fill=(74, 138, 52, 255))
    d.polygon([(10, 19), (14, 12), (19, 8), (22, 8), (19, 13), (13, 20)],
              fill=(92, 156, 62, 255))
    speckles(d, rng, (9, 8, 24, 22), 16, [(196, 224, 160, 255),
                                          (60, 118, 44, 255)], 0.9)
    d.ellipse([23, 4, 27, 8], fill=(120, 170, 84, 255))  # stem end
    return img


def draw_plantain(rng):
    img, d = canvas()
    # angular cooking banana: thick curved body with ridges
    d.polygon([(6, 8), (10, 6), (18, 9), (24, 16), (26, 24), (23, 26),
               (16, 20), (9, 14)], fill=(196, 178, 62, 255))
    d.polygon([(8, 10), (17, 12), (23, 19), (24, 23), (17, 18), (10, 13)],
              fill=(212, 196, 78, 255))
    d.line([(9, 9), (16, 15), (23, 23)], fill=(168, 150, 48, 255), width=1)
    d.line([(12, 7), (19, 13), (25, 20)], fill=(176, 160, 52, 255), width=1)
    speckles(d, rng, (10, 9, 24, 23), 6, [(120, 104, 40, 255)], 0.8)
    d.rectangle([4, 5, 8, 9], fill=(104, 96, 44, 255))  # stem
    return img


def draw_horseradish_root(rng):
    img, d = canvas()
    # long tapered off-white root, diagonal, brownish skin patches
    d.polygon([(9, 5), (14, 4), (17, 9), (15, 18), (11, 27), (8, 29),
               (7, 22), (8, 12)], fill=(228, 220, 198, 255))
    d.polygon([(10, 6), (13, 6), (15, 10), (13, 17), (10, 24), (9, 20),
               (9, 11)], fill=(238, 232, 212, 255))
    for y in (10, 15, 20, 24):
        d.line([(8, y), (14, y - 1)], fill=(186, 168, 138, 255))
    speckles(d, rng, (8, 6, 15, 26), 8, [(176, 156, 122, 255)], 0.9)
    # rootlet hairs near the tip
    d.line([(9, 27), (6, 30)], fill=(200, 188, 158, 255))
    d.line([(11, 27), (10, 31)], fill=(200, 188, 158, 255))
    # green top remnant
    leaf(d, 12, 5, 6, 2.2, -70, (96, 148, 62, 255))
    leaf(d, 13, 5, 5, 2.0, -110, (86, 136, 56, 255))
    return img


def draw_sugar_beet(rng):
    img, d = canvas()
    # big swollen root: round shoulder tapering to a tail
    d.polygon([(8, 10), (12, 6), (20, 6), (24, 10), (23, 17), (19, 23),
               (17, 29), (15, 29), (13, 23), (9, 17)], fill=(206, 188, 164, 255))
    d.polygon([(10, 10), (13, 8), (19, 8), (21, 11), (20, 16), (16, 21),
               (14, 16), (11, 15)], fill=(220, 204, 182, 255))
    for y in (11, 15, 19):
        d.arc([9, y - 3, 23, y + 4], 20, 160, fill=(178, 156, 128, 255))
    speckles(d, rng, (9, 8, 22, 22), 10, [(150, 128, 102, 255),
                                          (226, 190, 178, 255)], 1.0)
    # leafy top remnant
    leaf(d, 15, 6, 6, 2.4, -90, (92, 142, 60, 255))
    leaf(d, 14, 6, 5, 2.0, -55, (104, 154, 68, 255))
    leaf(d, 17, 6, 5, 2.0, -125, (84, 132, 54, 255))
    return img


def draw_jerusalem_artichoke(rng):
    img, d = canvas()
    # knobby irregular tuber: overlapping bumps
    bumps = [(13, 13, 6.5), (20, 12, 5.5), (11, 19, 5.8), (18, 19, 6.8),
             (15, 15, 6.0), (22, 17, 4.2), (9, 15, 3.8)]
    for cx, cy, r in bumps:
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(206, 178, 116, 255))
    for cx, cy, r in bumps:
        d.ellipse([cx - r * 0.55, cy - r * 0.6, cx + r * 0.2, cy + r * 0.1],
                  fill=(222, 198, 138, 255))
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(176, 148, 92, 255))
    speckles(d, rng, (7, 8, 25, 24), 8, [(160, 130, 80, 255)], 0.8)
    return img


def draw_salsify_raw(rng):
    img, d = canvas()
    # long thin dark taproot, diagonal, pale broken tip
    d.polygon([(8, 5), (12, 5), (17, 14), (23, 26), (21, 28), (14, 16),
               (8, 9)], fill=(104, 84, 62, 255))
    d.line([(9, 6), (15, 15), (21, 26)], fill=(128, 106, 80, 255), width=1)
    for t in (0.3, 0.5, 0.7):
        x = 10 + t * 11
        y = 7 + t * 18
        d.line([(x - 2, y), (x + 2, y - 1)], fill=(84, 66, 48, 255))
    d.polygon([(20, 24), (23, 26), (21, 29), (19, 27)],
              fill=(232, 224, 204, 255))  # pale flesh at tip
    leaf(d, 10, 5, 5, 1.8, -80, (100, 150, 64, 255))
    return img


def draw_wild_sarsaparilla_root(rng):
    img, d = canvas()
    # tangled stringy rhizome strands
    for _ in range(9):
        x0 = rng.uniform(6, 14)
        y0 = rng.uniform(8, 14)
        pts = [(x0, y0)]
        x, y = x0, y0
        for _ in range(3):
            x += rng.uniform(2, 6)
            y += rng.uniform(2, 6)
            pts.append((x, y))
        d.line(pts, fill=rng.choice([(128, 98, 66, 255), (112, 84, 54, 255),
                                     (144, 112, 76, 255)]), width=2)
    for _ in range(7):  # nodes / buds
        x = rng.uniform(8, 26)
        y = rng.uniform(10, 26)
        d.ellipse([x - 1.6, y - 1.6, x + 1.6, y + 1.6], fill=(156, 124, 86, 255))
    return img


def draw_raw_asparagus(rng):
    img, d = canvas()
    # bunch of spears, banded at the base
    for i, x in enumerate((9, 13, 17, 21, 24)):
        tip_col = (122, 128, 84, 255) if i % 2 else (110, 140, 74, 255)
        d.rectangle([x, 11, x + 3, 28], fill=(116, 164, 84, 255))
        d.line([(x + 1, 12), (x + 1, 27)], fill=(140, 186, 104, 255))
        # scaly tip
        d.polygon([(x - 1, 12), (x + 1.5, 4 + (i % 2)), (x + 4, 12)],
                  fill=tip_col)
        d.line([(x, 9), (x + 3, 9)], fill=shade(tip_col, 0.8))
    d.rectangle([8, 23, 28, 26], fill=(150, 116, 74, 255))  # twine band
    d.rectangle([8, 24, 28, 25], fill=(128, 96, 58, 255))
    return img


def draw_lotus_rhizome(rng):
    img, d = canvas()
    # chain of pale segments, diagonal; front cut face shows holes
    segs = [(9, 20, 5.2), (14, 15, 5.6), (19, 10, 5.2)]
    for cx, cy, r in segs:
        d.ellipse([cx - r, cy - r * 0.85, cx + r, cy + r * 0.85],
                  fill=(226, 210, 182, 255))
        d.arc([cx - r, cy - r * 0.85, cx + r, cy + r * 0.85], 90, 270,
              fill=(198, 178, 146, 255))
    speckles(d, rng, (5, 6, 24, 24), 8, [(188, 168, 136, 255)], 0.8)
    # cut face on the lowest segment: ring of tubular holes
    cx, cy = 9, 21
    d.ellipse([cx - 5.4, cy - 5.4, cx + 5.4, cy + 5.4], fill=(238, 226, 200, 255))
    holes = [(0, 0), (-3, -1.5), (3, -1.5), (-2, 2.5), (2, 2.5), (0, -3.4)]
    for hx, hy in holes:
        d.ellipse([cx + hx - 1.1, cy + hy - 1.1, cx + hx + 1.1, cy + hy + 1.1],
                  fill=(172, 150, 116, 255))
    return img


def draw_plant_stalk(rng):
    img, d = canvas()
    # stiff thick stalk section with nodes, slight diagonal
    d.polygon([(12, 4), (19, 4), (21, 28), (14, 28)], fill=(110, 162, 74, 255))
    d.polygon([(14, 4), (16.5, 4), (18, 28), (15.5, 28)],
              fill=(132, 182, 92, 255))
    for y in (11, 19):
        d.rectangle([12.6 - (y - 4) * 0.07, y, 20 - (y - 4) * 0.07, y + 2],
                    fill=(88, 138, 58, 255))
    d.ellipse([12, 2, 19, 6], fill=(206, 224, 168, 255))  # cut top face
    d.ellipse([14, 26, 21, 30], fill=(196, 214, 156, 255))  # cut base
    return img


def draw_japanese_knotweed_stem(rng):
    img, d = canvas()
    # segmented hollow stems, green with red speckles
    for x in (10, 17):
        d.rectangle([x, 6, x + 5, 28], fill=(146, 186, 104, 255))
        d.line([(x + 2, 7), (x + 2, 27)], fill=(168, 204, 124, 255))
        for y in (12, 19, 25):
            d.rectangle([x - 1, y, x + 6, y + 2], fill=(116, 158, 80, 255))
        d.ellipse([x, 4, x + 5, 7], fill=(214, 228, 178, 255))
    speckles(d, rng, (9, 7, 23, 27), 14, [(178, 70, 56, 255)], 0.9)
    leaf(d, 15, 6, 6, 2.6, -40, (104, 156, 68, 255), vein=(80, 128, 52, 255))
    return img


# ---------------------------------------------------------------- cut piles

def _pile(d, rng, n, fn, box=(5, 9, 27, 27)):
    for _ in range(n):
        fn(rng.uniform(box[0], box[2]), rng.uniform(box[1], box[3]))


def draw_bell_pepper_cut(rng):
    img, d = canvas()

    def piece(cx, cy):
        # chunky curved wall chunk: thick green shell, pale rib inside
        ang = rng.uniform(0, 360)
        pts = rpoly(cx, cy, [(cx - 4.4, cy - 2.6), (cx + 3.4, cy - 3.4),
                             (cx + 4.8, cy + 0.6), (cx + 2.2, cy + 3.2),
                             (cx - 3.4, cy + 2.8)], ang)
        d.polygon(pts, fill=(58, 142, 48, 255))
        pts2 = rpoly(cx, cy, [(cx - 3.0, cy - 1.2), (cx + 2.4, cy - 1.9),
                              (cx + 3.2, cy + 0.6), (cx + 1.2, cy + 2.0),
                              (cx - 2.2, cy + 1.6)], ang)
        d.polygon(pts2, fill=(172, 212, 134, 255))
        pts3 = rpoly(cx, cy, [(cx - 4.4, cy - 2.6), (cx + 3.4, cy - 3.4),
                              (cx + 4.0, cy - 1.4), (cx - 3.4, cy - 0.8)], ang)
        d.polygon(pts3, fill=(84, 168, 62, 255))  # glossy skin band
    _pile(d, rng, 9, piece)
    return img


def draw_chili_pepper_cut(rng):
    img, d = canvas()

    def piece(cx, cy):
        r = rng.uniform(2.2, 3.2)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(186, 44, 34, 255))
        d.ellipse([cx - r * 0.55, cy - r * 0.55, cx + r * 0.55, cy + r * 0.55],
                  fill=(214, 92, 62, 255))
    _pile(d, rng, 11, piece)
    return img


def draw_cucumber_cut(rng):
    img, d = canvas()

    def piece(cx, cy):
        r = rng.uniform(2.8, 3.8)
        d.pieslice([cx - r, cy - r, cx + r, cy + r], 180, 360,
                   fill=(52, 116, 44, 255))
        d.pieslice([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1], 180, 360,
                   fill=(196, 224, 168, 255))
        d.ellipse([cx - 0.8, cy - 1.6, cx + 0.8, cy - 0.2],
                  fill=(226, 238, 200, 255))
    _pile(d, rng, 10, piece)
    return img


def draw_gherkin_cut(rng):
    img, d = canvas()

    def piece(cx, cy):
        r = rng.uniform(2.0, 2.8)
        d.ellipse([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8],
                  fill=(58, 124, 46, 255))
        d.ellipse([cx - r * 0.6, cy - r * 0.45, cx + r * 0.6, cy + r * 0.45],
                  fill=(186, 216, 156, 255))
        speckles(d, rng, (cx - 1.5, cy - 1, cx + 1.5, cy + 1), 2,
                 [(220, 234, 190, 255)], 0.6)
    _pile(d, rng, 11, piece)
    return img


def draw_tomato_cut(rng):
    img, d = canvas()

    def wedge(cx, cy):
        ang = rng.uniform(0, 360)
        pts = rpoly(cx, cy, [(cx, cy), (cx + 5, cy - 3), (cx + 5.5, cy + 2),
                             (cx + 1, cy + 3.4)], ang)
        d.polygon(pts, fill=(206, 52, 40, 255))
        pts2 = rpoly(cx, cy, [(cx + 1.2, cy - 0.4), (cx + 4.2, cy - 1.8),
                              (cx + 4.4, cy + 1.2), (cx + 1.4, cy + 2.2)], ang)
        d.polygon(pts2, fill=(226, 88, 66, 255))
        # hollow gel pocket (seeds removed)
        pts3 = rpoly(cx, cy, [(cx + 2.2, cy - 0.6), (cx + 3.6, cy - 0.8),
                              (cx + 3.4, cy + 0.8), (cx + 2.2, cy + 0.8)], ang)
        d.polygon(pts3, fill=(240, 140, 110, 255))
    _pile(d, rng, 7, wedge)
    return img


def draw_zucchini_cut(rng):
    img, d = canvas()

    def piece(cx, cy):
        r = rng.uniform(2.8, 3.8)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(70, 128, 52, 255))
        d.ellipse([cx - r + 1.2, cy - r + 1.2, cx + r - 1.2, cy + r - 1.2],
                  fill=(216, 230, 178, 255))
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(228, 240, 198, 255))
    _pile(d, rng, 9, piece)
    return img


def draw_pumpkin_cut(rng):
    img, d = canvas()
    # quarter wedge: rind arc outside, thick orange flesh, hollowed face
    d.pieslice([4, 4, 30, 30], 20, 130, fill=(196, 108, 34, 255))
    d.pieslice([6.5, 6.5, 27.5, 27.5], 20, 130, fill=(226, 138, 44, 255))
    d.pieslice([11, 11, 24, 24], 25, 125, fill=(240, 178, 92, 255))
    d.line([(16, 5), (16, 3)], fill=(110, 96, 54, 255), width=2)  # stem bit
    speckles(d, rng, (10, 12, 22, 22), 5, [(248, 200, 120, 255)], 0.9)
    return img


# ---------------------------------------------------------------- handfuls

def draw_pea(rng):
    img, d = canvas()
    for _ in range(14):
        cx = rng.uniform(7, 25)
        cy = rng.uniform(10, 26)
        r = rng.uniform(1.7, 2.5)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(96, 172, 62, 255))
        d.ellipse([cx - r * 0.5, cy - r * 0.6, cx, cy - r * 0.1],
                  fill=(140, 204, 96, 255))
    return img


def draw_pea_pods(rng):
    img, d = canvas()
    pods = [(9, 10, 30), (8, 17, -8), (10, 24, 18)]
    for cx, cy, ang in pods:
        pts = rpoly(cx + 7, cy, [(cx, cy), (cx + 13, cy - 2.6),
                                 (cx + 15, cy), (cx + 13, cy + 2.6),
                                 (cx, cy + 1.6)], ang)
        d.polygon(pts, fill=(88, 156, 58, 255))
        d.line(rpoly(cx + 7, cy, [(cx + 1, cy), (cx + 14, cy)], ang),
               fill=(66, 128, 46, 255))
    # one split pod showing the row of peas
    cx, cy = 9, 20
    d.polygon([(cx, cy + 3), (cx + 13, cy + 1), (cx + 15, cy + 3.4),
               (cx + 12, cy + 6), (cx + 1, cy + 5.6)], fill=(76, 140, 52, 255))
    for i in range(4):
        x = cx + 3 + i * 3
        d.ellipse([x - 1.5, cy + 1.8, x + 1.5, cy + 4.8], fill=(110, 186, 70, 255))
    return img


def draw_raw_beans(rng):
    img, d = canvas()
    for _ in range(11):
        cx = rng.uniform(7, 25)
        cy = rng.uniform(10, 26)
        ang = rng.uniform(0, 360)
        pts = rpoly(cx, cy, [(cx - 3.4, cy - 2), (cx + 1, cy - 3),
                             (cx + 3.6, cy - 0.6), (cx + 3, cy + 2.2),
                             (cx - 1, cy + 3), (cx - 3.6, cy + 1)], ang)
        d.polygon(pts, fill=rng.choice([(150, 84, 58, 255), (138, 74, 50, 255),
                                        (162, 96, 66, 255)]))
        hp = rpoly(cx, cy, [(cx - 0.6, cy - 0.8)], ang)[0]
        d.ellipse([hp[0] - 0.9, hp[1] - 0.7, hp[0] + 0.9, hp[1] + 0.7],
                  fill=(216, 188, 160, 255))  # hilum
    return img


def draw_chicory_leaves(rng):
    img, d = canvas()
    # jagged elongated leaves fanned from a base point
    for ang in (-58, -30, -4, 24, 52):
        leaf(d, 15, 27, 17, 3.0, ang - 90, (140, 196, 92, 255),
             vein=(110, 168, 66, 255))
    for ang in (-44, -16, 10, 38):
        leaf(d, 15, 27, 13, 2.6, ang - 90, (156, 210, 108, 255),
             vein=(124, 182, 78, 255))
    # jagged teeth
    speckles(d, rng, (5, 8, 27, 24), 10, [(120, 178, 74, 255)], 1.0)
    return img


def draw_horseradish_greens(rng):
    img, d = canvas()
    # bunch of large oval leaves on stems
    for dx, ang in ((-4, -115), (0, -90), (4, -65)):
        d.line([(16 + dx, 28), (16 + dx, 18)], fill=(120, 168, 84, 255),
               width=2)
        leaf(d, 16 + dx, 18, 12, 4.6, ang, (108, 166, 66, 255),
             vein=(140, 192, 92, 255))
    leaf(d, 10, 22, 9, 3.8, -130, (96, 152, 58, 255))
    leaf(d, 22, 22, 9, 3.8, -50, (96, 152, 58, 255))
    return img


def draw_wintergreen_leaves(rng):
    img, d = canvas()
    # pile of glossy oval serrated leaves
    for _ in range(9):
        cx = rng.uniform(8, 24)
        cy = rng.uniform(10, 25)
        ang = rng.uniform(0, 360)
        leaf(d, cx, cy, rng.uniform(5, 7), rng.uniform(2.2, 3.0), ang,
             rng.choice([(42, 108, 48, 255), (52, 122, 56, 255)]),
             vein=(120, 176, 100, 255))
    # one red wintergreen berry
    d.ellipse([20, 8, 25, 13], fill=(196, 52, 44, 255))
    d.ellipse([21, 9, 23, 11], fill=(226, 96, 84, 255))
    return img


def draw_flour_wheat_free(rng):
    img, d = canvas()
    # small box tipped, white flour spilling into a heap
    d.polygon([(7, 12), (17, 9), (19, 17), (9, 20)], fill=(178, 148, 104, 255))
    d.polygon([(9, 13.5), (16, 11.5), (17, 16), (10, 18)],
              fill=(150, 122, 84, 255))
    d.polygon([(17, 9), (21, 7), (23, 14), (19, 17)], fill=(196, 166, 120, 255))
    # flour heap
    d.polygon([(8, 26), (13, 18), (19, 17), (25, 21), (27, 26)],
              fill=(246, 244, 238, 255))
    d.ellipse([13, 19, 21, 24], fill=(252, 251, 247, 255))
    speckles(d, rng, (6, 22, 28, 28), 10, [(238, 236, 228, 255)], 1.0)
    return img


def draw_veggy(rng):
    img, d = canvas()
    # triffid flesh: weird yellow-green fibrous chunk, leaking fluid
    d.polygon([(8, 10), (15, 6), (23, 8), (26, 15), (23, 23), (15, 27),
               (8, 23), (6, 15)], fill=(158, 172, 62, 255))
    d.polygon([(10, 11), (16, 9), (21, 11), (23, 16), (20, 21), (14, 23),
               (10, 20)], fill=(176, 188, 76, 255))
    # thick fibers
    for pts in ([(10, 12), (14, 17), (12, 22)], [(16, 9), (17, 15), (20, 21)],
                [(21, 11), (20, 17), (22, 21)], [(12, 20), (17, 19), (21, 15)]):
        d.line(pts, fill=(196, 204, 96, 255), width=1)
        d.line(pts, fill=(128, 142, 48, 255), width=0)
    speckles(d, rng, (8, 8, 24, 25), 8, [(140, 156, 54, 255)], 1.0)
    # leaked clear fluid: pale drip + pool (opaque)
    d.polygon([(14, 27), (16.5, 27), (15.5, 30)], fill=(214, 232, 220, 255))
    d.ellipse([11, 29, 20, 31.5], fill=(198, 220, 206, 255))
    return img


# ---------------------------------------------------------------- seaweeds

def draw_bladderwrack(rng):
    img, d = canvas()
    # brown fronds with round air bladders along the midrib
    for x0, sway in ((10, -2), (16, 1), (22, 3)):
        pts = []
        for i in range(6):
            y = 27 - i * 4.2
            x = x0 + sway * math.sin(i * 0.9)
            pts.append((x, y))
        d.line(pts, fill=(128, 102, 52, 255), width=3)
        d.line(pts, fill=(146, 118, 62, 255), width=1)
        for i in (1, 2, 3, 4):
            y = 27 - i * 4.2 - 2
            x = x0 + sway * math.sin(i * 0.9)
            s = 1 if i % 2 else -1
            d.ellipse([x + s * 2.4 - 1.8, y - 1.8, x + s * 2.4 + 1.8, y + 1.8],
                      fill=(168, 140, 74, 255))
            d.ellipse([x + s * 2.4 - 0.7, y - 1, x + s * 2.4 + 0.5, y + 0.4],
                      fill=(196, 168, 96, 255))
    return img


def draw_kelp_sugar(rng):
    img, d = canvas()
    # one wide brown ribbon frond with ruffled edges, diagonal
    pts_l, pts_r = [], []
    for i in range(9):
        t = i / 8
        cx = 9 + t * 13
        cy = 27 - t * 21
        w = 4.6 - t * 1.2
        ruffle = math.sin(t * 12) * 1.1
        pts_l.append((cx - w, cy + ruffle))
        pts_r.append((cx + w, cy - ruffle))
    d.polygon(pts_l + pts_r[::-1], fill=(122, 94, 48, 255))
    mid = [(9 + t * 13, 27 - t * 21) for t in (i / 8 for i in range(9))]
    d.line(mid, fill=(104, 78, 38, 255), width=2)
    d.line([(x - 1.6, y) for x, y in mid], fill=(146, 116, 60, 255), width=1)
    return img


def draw_sea_lettuce(rng):
    img, d = canvas()
    # bright ruffled green sheets
    for cx, cy, r in ((12, 13, 7.5), (20, 12, 6.5), (16, 20, 8.5),
                      (10, 21, 5.5), (23, 20, 5.5)):
        d.ellipse([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8],
                  fill=(86, 178, 74, 255))
        d.arc([cx - r + 1, cy - r * 0.8 + 1, cx + r - 1, cy + r * 0.8 - 1],
              0, 360, fill=(64, 152, 56, 255))
        d.ellipse([cx - r * 0.4, cy - r * 0.35, cx + r * 0.3, cy + r * 0.2],
                  fill=(110, 198, 92, 255))
    return img


def _seaweed_tangle(rng, spread, n):
    img, d = canvas()
    x0, y0, x1, y1 = spread
    for _ in range(n):
        sx = rng.uniform(x0, x1)
        sy = rng.uniform(y0, y1)
        pts = [(sx, sy)]
        x, y = sx, sy
        for _ in range(3):
            x += rng.uniform(-5, 5)
            y += rng.uniform(-3, 4)
            pts.append((x, y))
        d.line(pts, fill=rng.choice([(52, 118, 48, 255), (44, 104, 42, 255),
                                     (64, 134, 56, 255)]), width=2)
    speckles(d, rng, spread, 8, [(150, 200, 130, 255)], 0.8)  # slime sheen
    return img


def draw_seaweed(rng):
    return _seaweed_tangle(rng, (9, 11, 22, 22), 6)


def draw_seaweed_pile(rng):
    return _seaweed_tangle(rng, (4, 6, 28, 28), 14)


DRAW = {
    "artichoke_heart": draw_artichoke_heart,
    "bell_pepper_cut": draw_bell_pepper_cut,
    "bladderwrack": draw_bladderwrack,
    "brussels_sprout": draw_brussels_sprout,
    "brussels_sprout_stalk": draw_brussels_sprout_stalk,
    "cauliflower": draw_cauliflower,
    "cauliflower_head": draw_cauliflower_head,
    "chicory_leaves": draw_chicory_leaves,
    "chili_pepper_cut": draw_chili_pepper_cut,
    "cucumber_cut": draw_cucumber_cut,
    "flour_wheat_free": draw_flour_wheat_free,
    "gherkin": draw_gherkin,
    "gherkin_cut": draw_gherkin_cut,
    "horseradish_greens": draw_horseradish_greens,
    "horseradish_root": draw_horseradish_root,
    "japanese_knotweed_stem": draw_japanese_knotweed_stem,
    "jerusalem_artichoke": draw_jerusalem_artichoke,
    "kale": draw_kale,
    "kale_head": draw_kale_head,
    "kelp_sugar": draw_kelp_sugar,
    "lettuce_head": draw_lettuce_head,
    "lotus_rhizome": draw_lotus_rhizome,
    "pea": draw_pea,
    "pea_pods": draw_pea_pods,
    "plant_stalk": draw_plant_stalk,
    "plantain": draw_plantain,
    "pumpkin_cut": draw_pumpkin_cut,
    "raw_artichoke": draw_raw_artichoke,
    "raw_asparagus": draw_raw_asparagus,
    "raw_beans": draw_raw_beans,
    "salsify_raw": draw_salsify_raw,
    "sea_lettuce": draw_sea_lettuce,
    "seaweed": draw_seaweed,
    "seaweed_pile": draw_seaweed_pile,
    "sugar_beet": draw_sugar_beet,
    "tomato_cut": draw_tomato_cut,
    "veggy": draw_veggy,
    "wild_sarsaparilla_root": draw_wild_sarsaparilla_root,
    "wintergreen_leaves": draw_wintergreen_leaves,
    "zucchini_cut": draw_zucchini_cut,
}


def main():
    assert set(DRAW) == set(IDS), set(IDS) ^ set(DRAW)
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = DRAW[item_id](rng)
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
    cols, cell = 8, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell),
                      (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item36.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item36.png {sheet.size}")


if __name__ == "__main__":
    main()
