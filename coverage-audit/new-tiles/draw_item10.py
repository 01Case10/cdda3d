#!/usr/bin/env python3
"""Batch ITEM10 (Phase 2g.4 scope extension): meat dishes,
first set.

52 ids from data/json/items/comestibles/meat_dishes.json
(authoritative list embedded below; source list:
~/workspace/item-batches/10.txt). Parameterized family
renderers driven by each entry's name/description/color:

- Tin cans (open, contents visible): chicken paste (off-
  white), chowder (white soup, lumps), clams (gray chopped),
  salmon (bright pink), tuna (gray flakes), canned fish,
  canned lobster, canned meat; sardines get the flat oval
  tin with the lid peeled back and the little fish lined up.
- Sealed glass jars (jar_glass_sealed): pickled herring in
  white sauce, pickled fish, pickled meat -- pale opaque
  "glass", brine faked pale.
- Sliced cold meats: bologna (round red slices), lunch meat
  (thin fanned slices).
- Burgers: hamburger, cheeseburger (cheese corners),
  deluxe cheeseburger (lettuce + tomato layers).
- Fried things: fried chicken (golden drumstick + thigh),
  buttermilk fried chicken (paler gold, leg on the right),
  batter fried fish, fried frog legs (pair), fried SPAM
  slices, crab cakes / lobster cakes (copy-from fish_fried:
  round deep-fried patties, lobster redder).
- Plated dishes: chicken parmesan (pasta bed, red sauce,
  cheese), Hainanese chicken rice, tandoori chicken (deep
  red), escargot (shells, green herb butter), lobster
  platter (red lobster, lemon, greens), iskender (doner
  slices, tomato sauce, yogurt), deluxe scrambled eggs,
  meat fried rice, baked beans, deluxe beans and rice,
  hamburger helper (mac & cheese with ground meat).
- Flatbreads: kiymali pide (boat shape), lahmacun (round,
  minced topping).
- Cracklins (copy-from cracklins): crispy curled rind pile,
  tinted per flesh -- demihuman gray-pink, human pale,
  mutant greenish, mutant humanoid pale-green.
- Dried (copy-from the cooked forms): dehydrated flakes /
  shriveled pieces -- fish (light gray), meat (dark red),
  poultry (white), mollusk (light red curls), lobster,
  frog legs (shriveled pair).
- Singles: haggis (tied stomach pudding, oat speckle),
  lutefisk (pale gelatinous blob, glossy), salted meat
  slice (salt crystals, vac-bag sheen), Indian taco
  (frybread, beans, meat, lettuce), holy SPAM of debugging
  (pink block with a golden halo -- it is holy, and debug).

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
(glass, brine, gelatin) faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item10.png.
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
    "bologna",
    "can_chicken",
    "can_chowder",
    "can_clams",
    "can_herring",
    "can_salmon",
    "can_sardine",
    "can_tuna",
    "cheeseburger",
    "chicken_fried",
    "chicken_fried_buttermilk",
    "chicken_parm",
    "chicken_rice_hianese",
    "chicken_tandoori",
    "crab_cakes",
    "debug_nutrition",
    "deluxe_beans",
    "deluxe_beansnrice",
    "deluxe_cheeseburger",
    "deluxe_eggs",
    "deluxe_rice",
    "demihuman_cracklins",
    "dry_fish",
    "dry_froglegs",
    "dry_lobster",
    "dry_meat",
    "dry_mollusk",
    "dry_poultry",
    "escargot",
    "fish_canned",
    "fish_fried",
    "fish_pickled",
    "fried_froglegs",
    "fried_spam",
    "haggis",
    "hamburger",
    "human_cracklins",
    "iskender",
    "kiymali_pide",
    "lahmacun",
    "lobster_cakes",
    "lobster_canned",
    "lobster_platter",
    "lunchmeat",
    "lutefisk",
    "macaroni_helper",
    "meat_canned",
    "meat_pickled",
    "meat_salted",
    "mutant_cracklins",
    "mutant_human_cracklins",
    "ndn_taco",
]

# palette helpers -------------------------------------------------


def shade(col, f):
    return tuple(max(0, min(255, int(c * f))) for c in col) + (255,)


def blob_pts(rng, cx, cy, rx, ry, wob=0.18, n=14):
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        r = 1 + rng.uniform(-wob, wob)
        pts.append((cx + rx * r * math.cos(a), cy + ry * r * math.sin(a)))
    return pts


def speckle(d, rng, box, col, count, rmax=1.0):
    x0, y0, x1, y1 = box
    for _ in range(count):
        x = rng.uniform(x0, x1)
        y = rng.uniform(y0, y1)
        r = rng.uniform(0.4, rmax)
        d.ellipse([x - r, y - r, x + r, y + r], fill=col)


def new_img():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


STEEL = (168, 172, 178)
STEEL_D = (110, 114, 120)
STEEL_L = (208, 212, 218)


def plate(d, cx=16, cy=19, rx=13, ry=8):
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=(196, 198, 202, 255),
              outline=(140, 142, 148, 255))
    d.ellipse([cx - rx + 3, cy - ry + 2, cx + rx - 3, cy + ry - 2],
              fill=(222, 224, 228, 255))


def tin_can(rng, content, label):
    """Open round tin, contents drawn into the top ellipse."""
    img, d = new_img()
    d.rectangle([7, 11, 25, 27], fill=STEEL, outline=STEEL_D)
    d.rectangle([7, 17, 25, 19], fill=label)
    d.rectangle([7, 25, 25, 27], fill=STEEL_D)
    d.ellipse([7, 5, 25, 13], fill=STEEL_L, outline=STEEL_D)
    d.ellipse([9, 6, 23, 12], fill=content + (255,))
    return img, d


def can_contents_lumps(d, rng, col, n=7, box=(10, 6.5, 22, 11.5), rmax=1.7):
    speckle(d, rng, box, shade(col, 0.8), n, rmax=rmax)
    speckle(d, rng, box, shade(col, 1.15), n, rmax=rmax * 0.7)


def glass_jar(rng, content_col, chunk_col, n_chunks=6):
    """Sealed glass jar: pale glass, metal lid, chunks in brine."""
    img, d = new_img()
    d.rectangle([9, 9, 23, 28], fill=(206, 226, 230, 255), outline=(150, 176, 182, 255))
    d.rectangle([11, 12, 21, 27], fill=content_col + (255,))
    for _ in range(n_chunks):
        x = rng.uniform(12.5, 19.5)
        y = rng.uniform(14, 24.5)
        d.ellipse([x, y, x + rng.uniform(2.5, 4), y + rng.uniform(2, 3)],
                  fill=chunk_col + (255,), outline=shade(chunk_col, 0.75))
    d.rectangle([10, 13, 12, 27], fill=(232, 244, 246, 255))
    d.rectangle([8, 5, 24, 9], fill=(178, 146, 74, 255), outline=(128, 104, 50, 255))
    return img


# family renderers ------------------------------------------------

PINK_MEAT = (216, 118, 116)
RED_MEAT = (178, 62, 56)


def slices_round(rng, col):
    img, d = new_img()
    for dx, dy in ((0, 3), (-3, -1), (3, -2)):
        cx, cy = 16 + dx, 15 + dy
        d.ellipse([cx - 8, cy - 7, cx + 8, cy + 7], fill=col + (255,),
                  outline=shade(col, 0.62))
        speckle(d, rng, (cx - 6, cy - 5, cx + 6, cy + 5),
                (242, 226, 208, 255), 7, rmax=0.9)
        speckle(d, rng, (cx - 6, cy - 5, cx + 6, cy + 5),
                shade(col, 1.14), 8, rmax=0.9)
    return img


def slices_fanned(rng, col):
    img, d = new_img()
    for i in range(4):
        x = 6 + i * 4
        d.rounded_rectangle([x, 10, x + 13, 23], 2, fill=col + (255,),
                            outline=shade(col, 0.6))
        d.line([x + 2, 12, x + 11, 12], fill=shade(col, 1.16))
    speckle(d, rng, (8, 12, 26, 21), (242, 226, 208, 255), 8, rmax=0.8)
    return img


def burger(rng, cheese=False, deluxe=False):
    img, d = new_img()
    # wrapper peeking out at the bottom
    d.polygon([(4, 26), (28, 26), (24, 30), (8, 30)], fill=(214, 208, 196, 255))
    # bottom bun
    d.pieslice([6, 14, 26, 30], 0, 180, fill=(216, 164, 96, 255),
               outline=(168, 122, 62, 255))
    # patty
    d.rectangle([6, 20, 26, 23], fill=(110, 66, 40, 255))
    d.rectangle([6, 20, 26, 21], fill=(88, 50, 30, 255))
    if cheese:
        d.polygon([(6, 19), (26, 19), (26, 21), (21, 21), (19, 24),
                   (15, 21), (10, 21), (8, 23), (6, 21)],
                  fill=(244, 178, 44, 255))
    if deluxe:
        # tomato + lettuce above the patty
        d.rectangle([6, 17, 26, 19], fill=(196, 52, 44, 255))
        for x in range(6, 27, 4):
            d.arc([x, 14, x + 5, 19], 0, 180, fill=(86, 158, 62, 255), width=2)
    # top bun
    d.pieslice([6, 2, 26, 20], 180, 360, fill=(226, 176, 104, 255),
               outline=(168, 122, 62, 255))
    d.pieslice([9, 5, 23, 16], 180, 360, fill=(236, 192, 122, 255))
    for _ in range(6):
        x = rng.uniform(10, 22)
        y = rng.uniform(5, 9)
        d.ellipse([x, y, x + 1.6, y + 1], fill=(246, 232, 196, 255))
    return img


GOLD = (198, 138, 52)
GOLD_D = (150, 96, 34)


def fried_chicken(rng, pale=False):
    img, d = new_img()
    base = (228, 182, 96) if pale else GOLD
    dark = (186, 138, 62) if pale else GOLD_D
    # thigh blob behind
    d.polygon(blob_pts(rng, 12, 13, 8, 6.5), fill=base + (255,),
              outline=dark + (255,))
    # drumstick: meat bulb + bone
    d.polygon(blob_pts(rng, 21, 19, 7, 6), fill=base + (255,),
              outline=dark + (255,))
    box = (6, 8, 27, 25)
    speckle(d, rng, box, dark + (255,), 16 if pale else 24, rmax=1.1)
    speckle(d, rng, box, shade(base, 1.18), 14, rmax=0.9)
    if not pale:
        speckle(d, rng, box, (120, 74, 26, 255), 8, rmax=0.8)
    # bone drawn last so it stays visible through the breading
    d.line([25, 24, 29, 28], fill=(240, 236, 226, 255), width=2)
    d.ellipse([27.5, 26.5, 30.5, 29.5], fill=(240, 236, 226, 255))
    d.ellipse([28, 22, 31, 25], fill=(240, 236, 226, 255))
    return img


def fried_fish(rng):
    img, d = new_img()
    d.polygon([(7, 13), (20, 10), (24, 14), (24, 19), (20, 23), (7, 20),
               (4, 16)], fill=GOLD + (255,), outline=GOLD_D + (255,))
    d.polygon([(22, 14), (28, 11), (28, 21), (22, 19)], fill=GOLD + (255,),
              outline=GOLD_D + (255,))
    speckle(d, rng, (6, 11, 26, 22), GOLD_D + (255,), 20, rmax=1.0)
    speckle(d, rng, (6, 11, 26, 22), (232, 182, 92, 255), 14, rmax=0.9)
    return img


def froglegs(rng, dried=False):
    img, d = new_img()
    col = (168, 108, 66) if dried else GOLD
    dark = (128, 78, 40) if dried else GOLD_D
    for cx in (12, 20):
        # thigh + shin: two-segment leg
        d.line([cx, 8, cx - 2, 17], fill=col + (255,), width=5)
        d.line([cx - 2, 17, cx + 2, 26], fill=col + (255,), width=4)
        d.line([cx, 8, cx - 2, 17], fill=dark + (255,), width=1)
        d.ellipse([cx - 2.5, 5.5, cx + 2.5, 10], fill=col + (255,),
                  outline=dark + (255,))
    if not dried:
        speckle(d, rng, (7, 6, 25, 27), GOLD_D + (255,), 14, rmax=0.9)
        speckle(d, rng, (7, 6, 25, 27), (232, 182, 92, 255), 10, rmax=0.8)
    return img


def cakes(rng, fleck):
    img, d = new_img()
    plate(d)
    for cx, cy in ((12, 17), (20, 20)):
        d.ellipse([cx - 6, cy - 4.5, cx + 6, cy + 4.5], fill=GOLD + (255,),
                  outline=GOLD_D + (255,))
        speckle(d, rng, (cx - 5, cy - 3.5, cx + 5, cy + 3.5),
                fleck + (255,), 8, rmax=1.0)
        speckle(d, rng, (cx - 5, cy - 3.5, cx + 5, cy + 3.5),
                GOLD_D + (255,), 6, rmax=0.8)
    return img


def cracklins(rng, tint):
    img, d = new_img()
    base = tint
    for _ in range(11):
        cx = rng.uniform(8, 24)
        cy = rng.uniform(10, 24)
        rx = rng.uniform(3, 5.5)
        ry = rx * 0.55
        start = rng.uniform(0, 360)
        d.arc([cx - rx, cy - ry, cx + rx, cy + ry], start, start + 300,
              fill=shade(base, 0.6), width=3)
        d.arc([cx - rx, cy - ry, cx + rx, cy + ry], start, start + 300,
              fill=base + (255,), width=2)
    speckle(d, rng, (6, 8, 26, 26), shade(base, 1.25), 10, rmax=0.9)
    speckle(d, rng, (6, 8, 26, 26), shade(base, 0.7), 8, rmax=0.9)
    return img


def dry_flakes(rng, cols, leg_pair=False):
    img, d = new_img()
    if leg_pair:
        return froglegs(rng, dried=True)
    for _ in range(16):
        cx = rng.uniform(7, 25)
        cy = rng.uniform(9, 25)
        col = cols[rng.randrange(len(cols))]
        rx = rng.uniform(2.5, 4.5)
        ry = rx * 0.45
        a = rng.uniform(0, math.pi)
        pts = [(cx + rx * math.cos(a + t), cy + ry * math.sin(a + t))
               for t in (0, 2.1, 4.2)]
        d.polygon(pts, fill=col + (255,), outline=shade(col, 0.7))
    speckle(d, rng, (6, 8, 26, 26), (240, 238, 230, 255), 5, rmax=0.7)
    return img


def render(item_id, rng):
    # --- tin cans ---
    if item_id == "can_chicken":
        img, d = tin_can(rng, (238, 226, 208), (206, 120, 116))
        can_contents_lumps(d, rng, (238, 226, 208), 6)
        return img
    if item_id == "can_salmon":
        img, d = tin_can(rng, (240, 130, 128), (206, 120, 116))
        can_contents_lumps(d, rng, (240, 130, 128), 8)
        return img
    if item_id == "can_tuna":
        img, d = tin_can(rng, (196, 188, 178), (120, 140, 160))
        can_contents_lumps(d, rng, (196, 188, 178), 9, rmax=1.3)
        return img
    if item_id == "can_clams":
        img, d = tin_can(rng, (206, 210, 208), (120, 140, 160))
        can_contents_lumps(d, rng, (168, 170, 166), 8, rmax=1.5)
        return img
    if item_id == "can_chowder":
        img, d = tin_can(rng, (246, 242, 232), (196, 52, 44))
        speckle(d, rng, (10, 6.5, 22, 11.5), (226, 196, 140, 255), 5, rmax=1.4)
        speckle(d, rng, (10, 6.5, 22, 11.5), (186, 178, 168, 255), 4, rmax=1.2)
        speckle(d, rng, (10, 6.5, 22, 11.5), (238, 150, 130, 255), 3, rmax=1.0)
        return img
    if item_id == "fish_canned":
        img, d = tin_can(rng, (188, 190, 188), (120, 140, 160))
        for x in (12, 16.5, 21):
            d.ellipse([x - 2, 7, x + 2, 11.5], fill=(172, 174, 172, 255),
                      outline=(130, 132, 130, 255))
        return img
    if item_id == "lobster_canned":
        img, d = tin_can(rng, (226, 176, 166), (196, 84, 62))
        can_contents_lumps(d, rng, (214, 130, 116), 7, rmax=1.5)
        return img
    if item_id == "meat_canned":
        img, d = tin_can(rng, (172, 84, 70), (150, 96, 40))
        can_contents_lumps(d, rng, (160, 70, 60), 7, rmax=1.6)
        return img
    if item_id == "can_sardine":
        img, d = new_img()
        # peeled lid, bent back at the top
        d.polygon([(8, 3), (24, 3), (26, 8), (6, 8)], fill=STEEL_L,
                  outline=STEEL_D)
        d.ellipse([5, 9, 27, 25], fill=STEEL, outline=STEEL_D)
        d.ellipse([7, 11, 25, 23], fill=(196, 182, 150, 255))
        for i, y in enumerate((13.5, 17, 20.5)):
            x0 = 9 + (i % 2) * 2
            d.ellipse([x0, y, x0 + 13, y + 2.6], fill=(158, 164, 170, 255),
                      outline=(112, 118, 124, 255))
            d.polygon([(x0 + 13, y + 0.4), (x0 + 15.5, y + 1.3),
                       (x0 + 13, y + 2.2)], fill=(140, 146, 152, 255))
            d.point((x0 + 2, y + 1), fill=(40, 40, 44, 255))
        return img
    # --- jars ---
    if item_id == "can_herring":
        return glass_jar(rng, (240, 238, 228), (206, 208, 204), 5)
    if item_id == "fish_pickled":
        return glass_jar(rng, (226, 230, 214), (188, 190, 184), 6)
    if item_id == "meat_pickled":
        return glass_jar(rng, (228, 214, 204), (226, 138, 140), 6)
    # --- sliced cold meats ---
    if item_id == "bologna":
        return slices_round(rng, PINK_MEAT)
    if item_id == "lunchmeat":
        return slices_fanned(rng, (222, 128, 120))
    if item_id == "meat_salted":
        img, d = new_img()
        d.rounded_rectangle([7, 10, 25, 23], 3, fill=RED_MEAT + (255,),
                            outline=shade(RED_MEAT, 0.6))
        d.line([9, 13, 23, 13], fill=shade(RED_MEAT, 1.2))
        speckle(d, rng, (8, 11, 24, 22), (250, 248, 244, 255), 16, rmax=0.8)
        # vac-bag sheen corners
        d.line([5, 8, 12, 8], fill=(214, 232, 236, 255))
        d.line([5, 8, 5, 14], fill=(214, 232, 236, 255))
        d.line([27, 25, 20, 25], fill=(214, 232, 236, 255))
        d.line([27, 25, 27, 19], fill=(214, 232, 236, 255))
        return img
    if item_id == "fried_spam":
        img, d = new_img()
        plate(d)
        for dx in (-5, 4):
            d.rounded_rectangle([11 + dx, 12, 21 + dx, 24], 2,
                                fill=PINK_MEAT + (255,),
                                outline=(168, 84, 70, 255))
            speckle(d, rng, (12 + dx, 13, 20 + dx, 23),
                    (172, 92, 62, 255), 7, rmax=0.9)
        return img
    if item_id == "debug_nutrition":
        img, d = new_img()
        # golden halo
        d.ellipse([8, 2, 24, 8], outline=(246, 200, 74, 255), width=2)
        for a in range(0, 360, 45):
            x = 16 + 11 * math.cos(math.radians(a))
            y = 5 + 3.4 * math.sin(math.radians(a))
            d.point((x, y), fill=(250, 216, 110, 255))
        # the holy block of SPAM
        d.rectangle([9, 12, 23, 27], fill=PINK_MEAT + (255,),
                    outline=(168, 84, 70, 255))
        d.rectangle([9, 12, 23, 16], fill=(228, 140, 134, 255))
        d.rectangle([11, 19, 21, 24], fill=(240, 226, 208, 255))
        speckle(d, rng, (10, 17, 22, 26), (242, 226, 208, 255), 6, rmax=0.8)
        return img
    # --- burgers ---
    if item_id == "hamburger":
        return burger(rng)
    if item_id == "cheeseburger":
        return burger(rng, cheese=True)
    if item_id == "deluxe_cheeseburger":
        return burger(rng, cheese=True, deluxe=True)
    # --- fried ---
    if item_id == "chicken_fried":
        return fried_chicken(rng)
    if item_id == "chicken_fried_buttermilk":
        return fried_chicken(rng, pale=True)
    if item_id == "fish_fried":
        return fried_fish(rng)
    if item_id == "fried_froglegs":
        return froglegs(rng)
    if item_id == "crab_cakes":
        return cakes(rng, (240, 226, 208))
    if item_id == "lobster_cakes":
        return cakes(rng, (226, 110, 96))
    # --- plated dishes ---
    if item_id == "chicken_parm":
        img, d = new_img()
        plate(d)
        speckle(d, rng, (6, 14, 26, 25), (238, 220, 160, 255), 22, rmax=1.3)
        d.polygon(blob_pts(rng, 16, 18, 9, 5.5), fill=GOLD + (255,),
                  outline=GOLD_D + (255,))
        d.polygon(blob_pts(rng, 16, 17.5, 7.5, 4.2, wob=0.12),
                  fill=(198, 56, 44, 255))
        speckle(d, rng, (10, 14, 22, 21), (246, 240, 220, 255), 9, rmax=1.2)
        return img
    if item_id == "chicken_rice_hianese":
        img, d = new_img()
        plate(d)
        d.polygon(blob_pts(rng, 16, 20, 10, 5), fill=(240, 234, 214, 255),
                  outline=(206, 196, 168, 255))
        speckle(d, rng, (8, 16, 24, 24), (226, 208, 170, 255), 12, rmax=0.9)
        for i in range(3):
            x = 9 + i * 6
            d.rounded_rectangle([x, 11, x + 7, 17], 2,
                                fill=(238, 222, 200, 255),
                                outline=(198, 172, 142, 255))
        d.rectangle([22, 12, 25, 16], fill=(110, 168, 84, 255))
        return img
    if item_id == "chicken_tandoori":
        img, d = new_img()
        plate(d)
        for cx, cy, rx, ry in ((12, 16, 7, 5), (20, 21, 8, 5.5)):
            d.polygon(blob_pts(rng, cx, cy, rx, ry),
                      fill=(186, 62, 38, 255), outline=(128, 38, 24, 255))
        speckle(d, rng, (6, 12, 26, 25), (110, 34, 20, 255), 10, rmax=1.0)
        speckle(d, rng, (6, 12, 26, 25), (222, 108, 62, 255), 10, rmax=0.9)
        d.ellipse([24, 10, 28, 14], fill=(246, 214, 92, 255),
                  outline=(206, 168, 52, 255))
        return img
    if item_id == "escargot":
        img, d = new_img()
        plate(d, cy=18)
        for cx, cy in ((10, 14), (17, 12), (23, 15), (12, 21), (20, 22)):
            d.ellipse([cx - 3.5, cy - 3.5, cx + 3.5, cy + 3.5],
                      fill=(150, 116, 82, 255), outline=(104, 78, 54, 255))
            d.arc([cx - 2, cy - 2, cx + 2, cy + 2], 30, 330,
                  fill=(84, 60, 38, 255))
            d.ellipse([cx - 1.4, cy - 1.4, cx + 1.4, cy + 1.4],
                      fill=(96, 158, 66, 255))
        speckle(d, rng, (6, 10, 26, 25), (110, 176, 80, 255), 8, rmax=0.7)
        return img
    if item_id == "lobster_platter":
        img, d = new_img()
        plate(d, rx=14, ry=9)
        red = (204, 62, 48)
        d.polygon(blob_pts(rng, 16, 18, 8, 5.5), fill=red + (255,),
                  outline=(146, 38, 30, 255))
        for sx in (-1, 1):
            cx = 16 + sx * 10
            d.ellipse([cx - 3, 9, cx + 3, 15], fill=red + (255,),
                      outline=(146, 38, 30, 255))
            d.polygon([(cx, 12), (cx + sx * 4, 9), (cx + sx * 1, 14)],
                      fill=(146, 38, 30, 255))
            for ly in (16, 20, 24):
                d.line([16 + sx * 6, ly, 16 + sx * 10, ly + 2],
                       fill=(146, 38, 30, 255))
        d.polygon([(5, 22), (9, 20), (9, 25)], fill=(246, 214, 92, 255),
                  outline=(206, 168, 52, 255))
        speckle(d, rng, (20, 22, 27, 26), (96, 158, 66, 255), 6, rmax=0.9)
        return img
    if item_id == "iskender":
        img, d = new_img()
        plate(d)
        speckle(d, rng, (6, 14, 26, 25), (228, 196, 140, 255), 14, rmax=1.4)
        for i in range(4):
            y = 12.5 + i * 3
            d.rounded_rectangle([9, y, 23, y + 2.2], 1,
                                fill=(128, 80, 50, 255),
                                outline=(96, 56, 34, 255))
        speckle(d, rng, (8, 11, 24, 24), (204, 62, 48, 255), 16, rmax=1.3)
        d.ellipse([21, 10, 27, 15], fill=(246, 244, 238, 255),
                  outline=(208, 204, 194, 255))
        return img
    if item_id == "deluxe_eggs":
        img, d = new_img()
        plate(d)
        d.polygon(blob_pts(rng, 16, 18, 10, 6, wob=0.28),
                  fill=(246, 200, 74, 255), outline=(214, 164, 52, 255))
        speckle(d, rng, (8, 13, 24, 23), (250, 220, 130, 255), 14, rmax=1.2)
        speckle(d, rng, (8, 13, 24, 23), (172, 84, 70, 255), 8, rmax=1.1)
        speckle(d, rng, (8, 13, 24, 23), (240, 130, 128, 255), 4, rmax=1.0)
        return img
    if item_id == "deluxe_rice":
        img, d = new_img()
        plate(d)
        d.polygon(blob_pts(rng, 16, 18, 10.5, 6), fill=(232, 196, 120, 255),
                  outline=(196, 152, 84, 255))
        speckle(d, rng, (7, 13, 25, 23), (240, 220, 170, 255), 16, rmax=0.9)
        speckle(d, rng, (7, 13, 25, 23), (160, 74, 60, 255), 8, rmax=1.1)
        speckle(d, rng, (7, 13, 25, 23), (110, 168, 84, 255), 6, rmax=0.9)
        speckle(d, rng, (7, 13, 25, 23), (230, 130, 60, 255), 5, rmax=0.9)
        return img
    if item_id == "deluxe_beans":
        img, d = new_img()
        d.ellipse([5, 12, 27, 28], fill=(150, 96, 54, 255),
                  outline=(104, 64, 36, 255))
        d.ellipse([7, 9, 25, 17], fill=(122, 70, 38, 255),
                  outline=(88, 50, 28, 255))
        speckle(d, rng, (8, 10, 24, 16), (150, 92, 52, 255), 12, rmax=1.4)
        speckle(d, rng, (8, 10, 24, 16), (104, 58, 34, 255), 8, rmax=1.2)
        speckle(d, rng, (8, 10, 24, 16), (172, 84, 70, 255), 4, rmax=1.3)
        return img
    if item_id == "deluxe_beansnrice":
        img, d = new_img()
        plate(d, rx=14, ry=9)
        d.polygon(blob_pts(rng, 11, 18, 6.5, 5), fill=(122, 70, 38, 255),
                  outline=(88, 50, 28, 255))
        d.polygon(blob_pts(rng, 21, 18, 6.5, 5), fill=(240, 234, 214, 255),
                  outline=(206, 196, 168, 255))
        speckle(d, rng, (6, 14, 16, 22), (150, 92, 52, 255), 8, rmax=1.2)
        speckle(d, rng, (16, 14, 26, 22), (226, 208, 170, 255), 8, rmax=0.9)
        speckle(d, rng, (8, 13, 24, 23), (172, 84, 70, 255), 5, rmax=1.1)
        return img
    if item_id == "macaroni_helper":
        img, d = new_img()
        d.ellipse([5, 12, 27, 28], fill=(84, 110, 150, 255),
                  outline=(56, 76, 106, 255))
        d.ellipse([7, 9, 25, 17], fill=(244, 168, 62, 255),
                  outline=(206, 132, 44, 255))
        speckle(d, rng, (8, 10, 24, 16), (250, 196, 96, 255), 13, rmax=1.3)
        speckle(d, rng, (8, 10, 24, 16), (140, 84, 52, 255), 9, rmax=1.2)
        return img
    if item_id == "ndn_taco":
        img, d = new_img()
        d.ellipse([4, 8, 28, 27], fill=(222, 172, 96, 255),
                  outline=(178, 128, 60, 255))
        d.polygon(blob_pts(rng, 16, 17, 9, 6), fill=(150, 92, 52, 255))
        speckle(d, rng, (8, 12, 24, 22), (104, 58, 34, 255), 9, rmax=1.2)
        speckle(d, rng, (8, 12, 24, 22), (160, 74, 60, 255), 7, rmax=1.1)
        speckle(d, rng, (8, 11, 24, 20), (110, 176, 80, 255), 10, rmax=1.0)
        return img
    # --- flatbreads ---
    if item_id == "kiymali_pide":
        img, d = new_img()
        d.polygon([(4, 16), (10, 9), (22, 9), (28, 16), (22, 23), (10, 23)],
                  fill=(222, 176, 104, 255), outline=(172, 126, 62, 255))
        d.polygon([(8, 16), (12, 12), (20, 12), (24, 16), (20, 20),
                   (12, 20)], fill=(150, 92, 58, 255))
        speckle(d, rng, (10, 12, 22, 20), (196, 62, 48, 255), 6, rmax=0.9)
        speckle(d, rng, (10, 12, 22, 20), (110, 168, 84, 255), 5, rmax=0.9)
        speckle(d, rng, (10, 12, 22, 20), (240, 226, 190, 255), 5, rmax=0.8)
        return img
    if item_id == "lahmacun":
        img, d = new_img()
        d.ellipse([4, 5, 28, 28], fill=(228, 186, 116, 255),
                  outline=(178, 134, 70, 255))
        d.ellipse([6.5, 7.5, 25.5, 25.5], fill=(188, 106, 66, 255))
        speckle(d, rng, (8, 9, 24, 24), (160, 78, 50, 255), 14, rmax=1.0)
        speckle(d, rng, (8, 9, 24, 24), (204, 72, 54, 255), 7, rmax=0.9)
        speckle(d, rng, (8, 9, 24, 24), (110, 168, 84, 255), 7, rmax=0.8)
        return img
    # --- cracklins family ---
    if item_id == "demihuman_cracklins":
        return cracklins(rng, (196, 148, 128))
    if item_id == "human_cracklins":
        return cracklins(rng, (226, 178, 150))
    if item_id == "mutant_cracklins":
        return cracklins(rng, (168, 158, 96))
    if item_id == "mutant_human_cracklins":
        return cracklins(rng, (196, 182, 122))
    # --- dried family ---
    if item_id == "dry_fish":
        return dry_flakes(rng, [(188, 190, 188), (168, 170, 168),
                                (206, 206, 200)])
    if item_id == "dry_meat":
        return dry_flakes(rng, [(150, 56, 48), (128, 46, 42), (172, 76, 62)])
    if item_id == "dry_poultry":
        return dry_flakes(rng, [(240, 236, 226), (226, 216, 196),
                                (206, 196, 172)])
    if item_id == "dry_mollusk":
        img, d = new_img()
        for _ in range(9):
            cx = rng.uniform(8, 24)
            cy = rng.uniform(10, 24)
            r = rng.uniform(2.4, 3.6)
            d.arc([cx - r, cy - r, cx + r, cy + r], rng.uniform(0, 360),
                  rng.uniform(380, 560), fill=(196, 96, 84, 255), width=2)
        speckle(d, rng, (6, 8, 26, 26), (222, 140, 128, 255), 8, rmax=0.8)
        return img
    if item_id == "dry_lobster":
        return dry_flakes(rng, [(206, 186, 178), (196, 150, 138),
                                (188, 170, 164)])
    if item_id == "dry_froglegs":
        return dry_flakes(rng, [(168, 108, 66)], leg_pair=True)
    # --- singles ---
    if item_id == "haggis":
        img, d = new_img()
        plate(d)
        d.polygon(blob_pts(rng, 16, 18, 8.5, 6.5, wob=0.12),
                  fill=(146, 104, 70, 255), outline=(104, 72, 46, 255))
        d.rectangle([14, 10, 18, 13], fill=(120, 84, 56, 255))
        d.line([13, 10, 19, 10], fill=(226, 208, 170, 255), width=1)
        speckle(d, rng, (9, 13, 23, 23), (226, 208, 170, 255), 13, rmax=0.9)
        speckle(d, rng, (9, 13, 23, 23), (110, 76, 48, 255), 8, rmax=0.9)
        return img
    if item_id == "lutefisk":
        img, d = new_img()
        d.polygon(blob_pts(rng, 16, 17, 10, 8, wob=0.3),
                  fill=(216, 220, 216, 255), outline=(172, 178, 172, 255))
        d.polygon(blob_pts(rng, 13, 13, 5, 3, wob=0.3),
                  fill=(236, 240, 234, 255))
        speckle(d, rng, (8, 11, 24, 23), (196, 202, 194, 255), 8, rmax=1.0)
        return img
    raise ValueError(f"no renderer for {item_id}")


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
    cols, cell = 8, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item10.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item10.png {sheet.size}")


if __name__ == "__main__":
    main()
