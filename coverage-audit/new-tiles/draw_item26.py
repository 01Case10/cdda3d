#!/usr/bin/env python3
"""Batch ITEM26 (Phase 2g.4 scope extension): vegetable
dishes, first set -- 52 ids from
data/json/items/comestibles/veggy_dishes.json
(authoritative list embedded below; source list:
~/workspace/item-batches/26.txt).

Every entry was read before drawing; copy-from chains
resolved (most inherit the plain cooked-veg presentation
of their parents). Dishes are drawn AS SERVED, and the
presentation follows the preparation in the data:

- PLATE presentations: roasted/glazed whole vegetables
  (carrot_glazed with herb flecks vs. the plainer darker
  carrot_roasted), cored cooked_bell_pepper, lotus root
  slices with their unmistakable hole pattern, sauteed
  Jerusalem artichoke rounds, salsify sticks, nopalitos
  cactus pieces, tofu_stirfry.
- BATTERED/FRIED on plates: bamboo_fried, burdock_fried,
  dandelion_fried (golden flower puffs), and the mixed
  dandelionburdock_fried -- all golden crispy lumps,
  distinct from the plain cooked versions.
- BOWLS: buckwheat porridge, mashed pumpkin, hominy
  (swollen pale kernels), the bean dishes
  (deluxe_veggy_beans brown; beansnrice split bean/rice;
  succotash corn+lima, deluxe with pepper dice), fried
  rices (deluxe_veggy_rice yellow with dice; tofu_rice
  with white tofu cubes; sushi_rice plain white),
  rehydrated_veggy flakes, salsa (red, chunky),
  melitzanosalata (brown eggplant dip), sauerkraut
  (pale shreds; _onions with onion dice).
- SINGLES drawn as themselves: corn_on_cob (foil at the
  base), pickle (one bumpy cucumber), kombu (a dark
  wavy seaweed sheet), dry_chili (wrinkled, dark) vs.
  rehydrated_chili (plump, glossy), dry_garlic
  (shriveled) vs. rehydrated_garlic (plump), starch
  (a gooey paste puddle), roasted_cattail_rhizome
  (charred black, split to the dry inside),
  plantain_cooked (split roasted plantain),
  dahlia_baked (split baked root), dried_salad
  (a box of flakes with mayo/ketchup packets),
  dry rice / wild rice / veggy flake piles,
  dry_veggy_tainted (alien fungus chunks),
  sushi_veggyroll (three hosomaki cross-sections),
  veggy_aspic (a pale jelly mold with vegetables
  suspended inside, translucency faked with pale
  opaque colors -- this Pillow build replaces rather
  than blends).

All sprite alpha is in {0, 255}, asserted per sprite.
Writes ONLY its own 52 folders + preview_item26.png.
PIL only; per-sprite random.Random(crc32(id)).
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
    "bamboo_cooked",
    "bamboo_fried",
    "buckwheat_cooked",
    "burdock_cooked",
    "burdock_fried",
    "carrot_glazed",
    "carrot_roasted",
    "cooked_bell_pepper",
    "cooked_burdock_stalk",
    "cooked_cattail_stalk",
    "cooked_pumpkin",
    "corn_on_cob",
    "dahlia_baked",
    "dandelion_cooked",
    "dandelion_fried",
    "dandelionburdock_fried",
    "deluxe_veggy_beans",
    "deluxe_veggy_beansnrice",
    "deluxe_veggy_rice",
    "dried_salad",
    "dry_chili",
    "dry_garlic",
    "dry_rice",
    "dry_veggy",
    "dry_veggy_tainted",
    "dry_wild_rice",
    "hominy",
    "jerusalem_artichoke_boiled",
    "jerusalem_artichoke_sauteed",
    "kombu",
    "lotus_rhizome_boiled",
    "melitzanosalata",
    "nopalitos",
    "onion_sauteed",
    "pickle",
    "plantain_cooked",
    "rehydrated_chili",
    "rehydrated_garlic",
    "rehydrated_veggy",
    "roasted_cattail_rhizome",
    "salsa",
    "salsify_baked",
    "sauerkraut",
    "sauerkraut_onions",
    "starch",
    "succotash",
    "succotash_deluxe",
    "sushi_rice",
    "sushi_veggyroll",
    "tofu_rice",
    "tofu_stirfry",
    "veggy_aspic",
]

INK = (44, 38, 34, 255)
PLATE = (233, 231, 225, 255)
PLATE_D = (188, 184, 176, 255)
PLATE_IN = (214, 210, 202, 255)
BOWL_C = (106, 116, 128, 255)     # blue-gray ceramic bowl
BOWL_D = (74, 84, 98, 255)
BOWL_IN = (140, 152, 166, 255)


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def plate(d, cy=20):
    d.ellipse([2, cy - 8, 30, cy + 8], fill=PLATE, outline=INK)
    d.ellipse([5, cy - 5, 27, cy + 5], fill=PLATE_IN)


def bowl(d, contents):
    """Ceramic bowl; contents = fill color of the food surface."""
    d.pieslice([3, 7, 29, 29], start=0, end=180, fill=BOWL_C,
               outline=INK)
    d.rectangle([13, 27, 19, 29], fill=BOWL_D)
    d.ellipse([3, 7, 29, 17], fill=BOWL_IN, outline=INK)
    d.ellipse([5, 8, 27, 16], fill=contents)


def speck(d, rng, box, colors, n, rmax=1):
    x0, y0, x1, y1 = box
    cx, cy = (x0 + x1) / 2.0, (y0 + y1) / 2.0
    rx, ry = (x1 - x0) / 2.0, (y1 - y0) / 2.0
    for _ in range(n):
        for _try in range(8):
            x = rng.uniform(x0, x1)
            y = rng.uniform(y0, y1)
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 0.92:
                r = rng.randint(0, rmax)
                c = colors[rng.randrange(len(colors))]
                d.ellipse([x - r, y - r, x + r, y + r], fill=c)
                break


def chunks(d, rng, box, colors, n, smin=2, smax=3):
    """Random chunky pieces (rounded rects) inside an ellipse box."""
    x0, y0, x1, y1 = box
    cx, cy = (x0 + x1) / 2.0, (y0 + y1) / 2.0
    rx, ry = (x1 - x0) / 2.0, (y1 - y0) / 2.0
    for _ in range(n):
        for _try in range(8):
            x = rng.uniform(x0, x1)
            y = rng.uniform(y0, y1)
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 0.8:
                s = rng.randint(smin, smax)
                c = colors[rng.randrange(len(colors))]
                d.rectangle([x - s, y - s, x + s, y + s], fill=c)
                break


def leaf(d, cx, cy, w, h, color, vein):
    d.polygon([(cx - w, cy), (cx, cy - h), (cx + w, cy), (cx, cy + h)],
              fill=color, outline=INK)
    d.line([(cx - w + 1, cy), (cx + w - 1, cy)], fill=vein)


# ---------------------------------------------------------------- plates

def draw_bamboo_cooked(rng):
    img, d = canvas()
    plate(d)
    seg = (238, 226, 188, 255)
    seg_d = (206, 190, 142, 255)
    for cx, cy, a in ((11, 16, -20), (19, 15, 15), (15, 21, -8),
                      (22, 20, 25)):
        d.rectangle([cx - 4, cy - 2, cx + 4, cy + 2], fill=seg,
                    outline=INK)
        d.line([(cx - 1, cy - 2), (cx - 1, cy + 2)], fill=seg_d)
        d.line([(cx + 2, cy - 2), (cx + 2, cy + 2)], fill=seg_d)
    return img


def _fried_plate(rng, base_cols, extra=None):
    img, d = canvas()
    plate(d)
    chunks(d, rng, [6, 12, 26, 24], base_cols, 11, 2, 3)
    speck(d, rng, [6, 12, 26, 24], [(146, 96, 32, 255)], 8)
    if extra:
        extra(d, rng)
    return img


def draw_bamboo_fried(rng):
    return _fried_plate(rng, [(216, 164, 74, 255), (228, 182, 92, 255),
                              (198, 142, 52, 255)])


def draw_burdock_fried(rng):
    return _fried_plate(rng, [(196, 158, 66, 255), (170, 148, 60, 255),
                              (120, 140, 62, 255)])


def draw_dandelion_fried(rng):
    img, d = canvas()
    plate(d)
    for cx, cy in ((11, 16), (18, 14), (22, 19), (14, 21), (20, 22)):
        d.line([(cx, cy + 3), (cx, cy + 5)], fill=(96, 124, 52, 255))
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3],
                  fill=(232, 186, 74, 255), outline=INK)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1],
                  fill=(246, 214, 120, 255))
    return img


def draw_dandelionburdock_fried(rng):
    img, d = canvas()
    plate(d)
    chunks(d, rng, [6, 12, 26, 24], [(196, 158, 66, 255),
                                     (140, 152, 64, 255)], 7, 2, 3)
    for cx, cy in ((12, 15), (21, 20)):
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3],
                  fill=(232, 186, 74, 255), outline=INK)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1],
                  fill=(246, 214, 120, 255))
    return img


def draw_burdock_cooked(rng):
    img, d = canvas()
    plate(d)
    for cx, cy, w, h in ((11, 16, 5, 3), (19, 15, 6, 3), (15, 20, 6, 4),
                         (22, 20, 4, 3)):
        leaf(d, cx, cy, w, h, (74, 116, 52, 255), (52, 88, 38, 255))
    return img


def draw_dandelion_cooked(rng):
    img, d = canvas()
    plate(d)
    for cx, cy, w, h in ((11, 16, 5, 3), (19, 15, 6, 3), (15, 20, 6, 4),
                         (22, 20, 4, 3)):
        leaf(d, cx, cy, w, h, (56, 96, 44, 255), (38, 70, 30, 255))
    speck(d, rng, [7, 12, 25, 24], [(86, 128, 62, 255)], 6)
    return img


def _carrot(rng, body, dark, flecks):
    img, d = canvas()
    plate(d)
    # whole carrot lying diagonally, greens at the top-left
    for gx, gy in ((6, 10), (8, 8), (10, 10)):
        d.line([(11, 13), (gx, gy)], fill=(70, 128, 52, 255), width=1)
    d.polygon([(10, 12), (14, 11), (25, 21), (26, 24), (23, 24),
               (12, 16)], fill=body, outline=INK)
    d.line([(14, 13), (23, 22)], fill=dark)
    d.line([(16, 12), (24, 20)], fill=dark)
    if flecks:
        speck(d, rng, [12, 12, 24, 23], [(84, 138, 58, 255),
                                         (240, 240, 230, 255)], 7, 0)
        d.line([(13, 12), (20, 19)], fill=(255, 226, 168, 255))
    return img


def draw_carrot_glazed(rng):
    return _carrot(rng, (238, 128, 52, 255), (204, 96, 34, 255), True)


def draw_carrot_roasted(rng):
    img = _carrot(rng, (206, 104, 40, 255), (164, 74, 26, 255), False)
    d = ImageDraw.Draw(img)
    speck(d, rng, [12, 12, 24, 23], [(110, 58, 24, 255)], 6, 0)
    return img


def draw_cooked_bell_pepper(rng):
    img, d = canvas()
    plate(d)
    green = (72, 148, 62, 255)
    green_d = (50, 112, 44, 255)
    inner = (168, 204, 138, 255)
    d.polygon([(10, 13), (22, 13), (24, 19), (22, 25), (10, 25),
               (8, 19)], fill=green, outline=INK)
    d.ellipse([11, 11, 21, 16], fill=inner, outline=INK)   # cored top
    d.ellipse([14, 12, 18, 15], fill=(226, 238, 200, 255))
    d.line([(10, 18), (10, 23)], fill=green_d)
    d.line([(22, 18), (22, 23)], fill=green_d)
    return img


def _stalks(rng, col, col_d, thick):
    img, d = canvas()
    plate(d)
    for cx, cy in ((11, 16), (19, 15), (15, 21), (22, 20)):
        d.rectangle([cx - 4, cy - thick, cx + 4, cy + thick],
                    fill=col, outline=INK)
        d.line([(cx, cy - thick), (cx, cy + thick)], fill=col_d)
    return img


def draw_cooked_burdock_stalk(rng):
    return _stalks(rng, (226, 230, 200, 255), (192, 198, 158, 255), 2)


def draw_cooked_cattail_stalk(rng):
    return _stalks(rng, (242, 242, 236, 255), (206, 206, 196, 255), 1)


def draw_dahlia_baked(rng):
    img, d = canvas()
    plate(d)
    d.ellipse([8, 12, 24, 25], fill=(150, 108, 66, 255), outline=INK)
    d.polygon([(11, 13), (21, 13), (18, 18), (14, 18)],
              fill=(232, 204, 156, 255))     # split top, soft interior
    speck(d, rng, [9, 18, 23, 24], [(118, 82, 46, 255)], 5, 0)
    return img


def draw_salsify_baked(rng):
    img, d = canvas()
    plate(d)
    for cx, cy, a in ((11, 15, -14), (18, 14, 8), (14, 20, -6),
                      (21, 20, 16)):
        d.rectangle([cx - 5, cy - 1, cx + 5, cy + 2],
                    fill=(228, 206, 164, 255), outline=INK)
        d.point((cx + 4, cy), fill=(168, 120, 62, 255))
    return img


def draw_jerusalem_artichoke_sauteed(rng):
    img, d = canvas()
    plate(d)
    for cx, cy in ((11, 16), (18, 14), (22, 19), (14, 21), (19, 22)):
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3],
                  fill=(222, 188, 128, 255), outline=(168, 120, 62, 255))
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1],
                  fill=(238, 210, 156, 255))
    return img


def draw_lotus_rhizome_boiled(rng):
    img, d = canvas()
    plate(d)
    flesh = (240, 236, 226, 255)
    hole = (198, 188, 168, 255)
    for cx, cy, holes in ((12, 16, ((0, 0), (-2, -1), (2, -1), (0, 2))),
                          (20, 15, ((0, 0), (-2, 1), (2, 1))),
                          (16, 21, ((0, 0), (-2, -1), (2, -1), (-1, 2),
                                    (1, 2)))):
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=flesh,
                  outline=INK)
        for hx, hy in holes:
            d.ellipse([cx + hx - 1, cy + hy - 1, cx + hx + 1,
                       cy + hy + 1], fill=hole)
    return img


def draw_nopalitos(rng):
    img, d = canvas()
    plate(d)
    pad = (84, 156, 74, 255)
    pad_d = (58, 120, 52, 255)
    for cx, cy in ((11, 16), (19, 14), (15, 21), (22, 20), (12, 22)):
        d.ellipse([cx - 3, cy - 2, cx + 3, cy + 2], fill=pad,
                  outline=INK)
        d.point((cx, cy), fill=pad_d)
    return img


def draw_onion_sauteed(rng):
    img, d = canvas()
    plate(d)
    cols = [(196, 138, 62, 255), (216, 164, 84, 255), (172, 112, 46, 255)]
    for _ in range(9):
        x = rng.randint(8, 22)
        y = rng.randint(13, 23)
        c = cols[rng.randrange(len(cols))]
        d.arc([x, y - 2, x + 6, y + 3], start=200, end=340, fill=c,
              width=1)
        d.arc([x, y - 1, x + 5, y + 2], start=20, end=160, fill=c,
              width=1)
    return img


def draw_tofu_stirfry(rng):
    img, d = canvas()
    plate(d)
    sauce = (150, 84, 40, 255)
    d.ellipse([6, 13, 26, 25], fill=sauce)
    chunks(d, rng, [7, 13, 25, 24], [(240, 234, 214, 255),
                                     (244, 238, 222, 255)], 6, 2, 2)
    chunks(d, rng, [7, 13, 25, 24], [(84, 156, 74, 255),
                                     (226, 120, 44, 255),
                                     (196, 52, 44, 255)], 7, 1, 2)
    speck(d, rng, [7, 13, 25, 24], [(255, 226, 160, 255)], 6, 0)
    return img


# ---------------------------------------------------------------- bowls

def draw_buckwheat_cooked(rng):
    img, d = canvas()
    bowl(d, (148, 122, 96, 255))
    speck(d, rng, [6, 9, 26, 15], [(122, 98, 72, 255),
                                   (172, 148, 118, 255)], 26, 0)
    return img


def draw_cooked_pumpkin(rng):
    img, d = canvas()
    bowl(d, (232, 128, 48, 255))
    d.arc([10, 8, 22, 16], start=20, end=300, fill=(204, 104, 34, 255),
          width=1)
    speck(d, rng, [6, 9, 26, 15], [(244, 152, 70, 255)], 8, 0)
    return img


def draw_hominy(rng):
    img, d = canvas()
    bowl(d, (238, 214, 140, 255))
    speck(d, rng, [6, 9, 26, 15], [(248, 232, 178, 255),
                                   (224, 192, 108, 255)], 22, 1)
    return img


def draw_jerusalem_artichoke_boiled(rng):
    img, d = canvas()
    bowl(d, (120, 96, 70, 255))
    for cx, cy in ((11, 12), (17, 11), (22, 12), (14, 14), (20, 14)):
        d.ellipse([cx - 3, cy - 2, cx + 3, cy + 2],
                  fill=(186, 152, 108, 255), outline=INK)
        d.point((cx + 1, cy - 1), fill=(160, 126, 84, 255))
    return img


def draw_deluxe_veggy_beans(rng):
    img, d = canvas()
    bowl(d, (122, 80, 46, 255))
    speck(d, rng, [6, 9, 26, 15], [(146, 100, 58, 255),
                                   (104, 64, 36, 255)], 20, 1)
    speck(d, rng, [6, 9, 26, 15], [(226, 120, 44, 255),
                                   (84, 156, 74, 255)], 5, 0)
    return img


def draw_deluxe_veggy_beansnrice(rng):
    img, d = canvas()
    bowl(d, (232, 222, 192, 255))          # rice side as the base
    # bean half spooned over the left of the surface
    d.pieslice([5, 8, 27, 16], start=90, end=270,
               fill=(122, 80, 46, 255))
    d.rectangle([5, 11, 6, 13], fill=(122, 80, 46, 255))
    speck(d, rng, [6, 9, 16, 15], [(146, 100, 58, 255),
                                   (104, 64, 36, 255)], 12, 1)
    speck(d, rng, [16, 9, 26, 15], [(248, 244, 224, 255),
                                    (214, 202, 168, 255)], 12, 0)
    speck(d, rng, [6, 9, 26, 15], [(226, 120, 44, 255),
                                   (84, 156, 74, 255)], 4, 0)
    return img


def _rice_bowl(rng, base, dice_cols, tofu=False):
    img, d = canvas()
    bowl(d, base)
    speck(d, rng, [6, 9, 26, 15], dice_cols, 16, 0)
    if tofu:
        chunks(d, rng, [7, 9, 25, 15], [(244, 240, 226, 255)], 5, 1, 2)
    return img


def draw_deluxe_veggy_rice(rng):
    return _rice_bowl(rng, (240, 200, 96, 255),
                      [(96, 176, 80, 255), (230, 122, 44, 255),
                       (248, 222, 140, 255)])


def draw_tofu_rice(rng):
    return _rice_bowl(rng, (240, 200, 96, 255),
                      [(96, 176, 80, 255), (230, 122, 44, 255)], tofu=True)


def draw_sushi_rice(rng):
    img, d = canvas()
    bowl(d, (246, 244, 238, 255))
    speck(d, rng, [6, 9, 26, 15], [(228, 226, 216, 255),
                                   (255, 255, 252, 255)], 18, 0)
    return img


def _succotash(rng, deluxe):
    img, d = canvas()
    bowl(d, (240, 198, 84, 255))
    speck(d, rng, [6, 9, 26, 15], [(248, 218, 120, 255),
                                   (226, 178, 58, 255)], 16, 1)
    speck(d, rng, [6, 9, 26, 15], [(196, 216, 150, 255),
                                   (170, 196, 120, 255)], 9, 1)
    if deluxe:
        speck(d, rng, [6, 9, 26, 15], [(208, 56, 44, 255),
                                       (240, 150, 60, 255)], 6, 0)
        speck(d, rng, [6, 9, 26, 15], [(255, 240, 190, 255)], 6, 0)
    return img


def draw_succotash(rng):
    return _succotash(rng, False)


def draw_succotash_deluxe(rng):
    return _succotash(rng, True)


def draw_salsa(rng):
    img, d = canvas()
    bowl(d, (196, 52, 40, 255))
    speck(d, rng, [6, 9, 26, 15], [(222, 84, 62, 255),
                                   (168, 38, 30, 255)], 14, 1)
    speck(d, rng, [6, 9, 26, 15], [(96, 176, 80, 255),
                                   (240, 236, 210, 255)], 6, 0)
    return img


def draw_melitzanosalata(rng):
    img, d = canvas()
    bowl(d, (128, 96, 66, 255))
    d.arc([9, 8, 23, 16], start=20, end=300, fill=(104, 76, 50, 255),
          width=1)
    speck(d, rng, [6, 9, 26, 15], [(240, 236, 214, 255),
                                   (216, 196, 140, 255)], 7, 0)
    return img


def _kraut(rng, onions):
    img, d = canvas()
    bowl(d, (206, 216, 168, 255))
    cols = [(188, 200, 146, 255), (224, 230, 192, 255),
            (168, 182, 126, 255)]
    for _ in range(12):
        x = rng.randint(7, 21)
        y = rng.randint(9, 14)
        d.arc([x, y, x + 5, y + 3], start=180, end=360,
              fill=cols[rng.randrange(len(cols))], width=1)
    if onions:
        speck(d, rng, [6, 9, 26, 15], [(242, 240, 234, 255)], 8, 0)
    return img


def draw_sauerkraut(rng):
    return _kraut(rng, False)


def draw_sauerkraut_onions(rng):
    return _kraut(rng, True)


def draw_rehydrated_veggy(rng):
    img, d = canvas()
    bowl(d, (128, 178, 92, 255))
    speck(d, rng, [6, 9, 26, 15], [(230, 124, 46, 255),
                                   (240, 206, 110, 255),
                                   (84, 148, 62, 255)], 16, 1)
    return img


# -------------------------------------------------------------- singles

def draw_corn_on_cob(rng):
    img, d = canvas()
    yel = (246, 200, 62, 255)
    yel_d = (224, 172, 40, 255)
    d.polygon([(12, 6), (18, 5), (21, 18), (19, 23), (13, 23),
               (11, 18)], fill=yel, outline=INK)
    for x in (14, 17, 20):
        d.line([(x - 1, 7), (x, 21)], fill=yel_d)
    for y in (10, 14, 18):
        d.line([(12, y), (20, y)], fill=yel_d)
    # foil wrap crumpled around the base
    d.polygon([(11, 20), (21, 20), (22, 27), (10, 27)],
              fill=(206, 210, 216, 255), outline=INK)
    d.line([(13, 22), (16, 26)], fill=(168, 174, 182, 255))
    d.line([(19, 22), (17, 26)], fill=(168, 174, 182, 255))
    return img


def draw_pickle(rng):
    img, d = canvas()
    green = (86, 140, 62, 255)
    d.polygon([(9, 12), (13, 9), (19, 10), (23, 15), (22, 21),
               (17, 24), (11, 22), (8, 17)], fill=green, outline=INK)
    speck(d, rng, [10, 11, 22, 22], [(110, 164, 82, 255),
                                     (64, 116, 48, 255)], 12, 0)
    d.point((10, 13), fill=(196, 220, 160, 255))
    return img


def draw_kombu(rng):
    img, d = canvas()
    col = (74, 66, 44, 255)
    col_d = (52, 46, 30, 255)
    d.polygon([(7, 8), (24, 6), (26, 12), (24, 18), (25, 24),
               (8, 25), (6, 18), (8, 12)], fill=col, outline=INK)
    for y in (11, 16, 21):
        d.line([(8, y), (24, y - 1)], fill=col_d)
    d.polygon([(20, 6), (24, 6), (25, 10), (21, 9)],
              fill=(96, 88, 58, 255))     # curled corner
    return img


def _chili(rng, col, col_d, wrinkled):
    img, d = canvas()
    d.polygon([(13, 8), (17, 9), (20, 14), (19, 21), (15, 26),
               (12, 27), (12, 20), (10, 14)], fill=col, outline=INK)
    d.polygon([(12, 8), (18, 8), (16, 5), (13, 5)],
              fill=(84, 110, 52, 255), outline=INK)   # stem cap
    if wrinkled:
        for y in (12, 16, 20, 23):
            d.line([(12, y), (18, y + 1)], fill=col_d)
    else:
        d.line([(14, 11), (13, 22)], fill=(255, 190, 170, 255))
    return img


def draw_dry_chili(rng):
    return _chili(rng, (158, 40, 32, 255), (116, 26, 22, 255), True)


def draw_rehydrated_chili(rng):
    return _chili(rng, (206, 52, 40, 255), (168, 34, 28, 255), False)


def _garlic(rng, col, col_d, plump):
    img, d = canvas()
    d.polygon([(16, 6), (18, 10), (21, 14), (21, 21), (18, 26),
               (14, 26), (11, 21), (11, 14), (14, 10)],
              fill=col, outline=INK)
    d.line([(16, 10), (16, 25)], fill=col_d)
    d.line([(13, 12), (13, 23)], fill=col_d)
    d.line([(19, 12), (19, 23)], fill=col_d)
    if not plump:
        d.line([(12, 17), (20, 16)], fill=col_d)
        d.line([(12, 21), (20, 21)], fill=col_d)
    else:
        d.point((14, 13), fill=(255, 255, 250, 255))
    return img


def draw_dry_garlic(rng):
    return _garlic(rng, (226, 220, 204, 255), (196, 188, 164, 255),
                   False)


def draw_rehydrated_garlic(rng):
    return _garlic(rng, (244, 240, 228, 255), (214, 208, 188, 255),
                   True)


def _grain_pile(rng, cols, long_grains):
    img, d = canvas()
    d.ellipse([6, 16, 26, 27], fill=cols[0])
    for _ in range(46):
        x = rng.randint(8, 23)
        y = rng.randint(15, 25)
        c = cols[rng.randrange(len(cols))]
        if long_grains:
            d.line([(x, y), (x + 2, y + 1)], fill=c)
        else:
            d.point((x, y), fill=c)
    return img


def draw_dry_rice(rng):
    return _grain_pile(rng, [(238, 234, 220, 255), (250, 248, 240, 255),
                             (214, 210, 194, 255)], True)


def draw_dry_wild_rice(rng):
    return _grain_pile(rng, [(84, 62, 44, 255), (104, 82, 58, 255),
                             (62, 46, 32, 255)], True)


def draw_dry_veggy(rng):
    img, d = canvas()
    d.ellipse([6, 15, 26, 27], fill=(110, 140, 70, 255))
    speck(d, rng, [7, 14, 25, 26], [(140, 168, 92, 255),
                                    (216, 130, 54, 255),
                                    (196, 190, 130, 255),
                                    (84, 120, 56, 255)], 30, 1)
    return img


def draw_dry_veggy_tainted(rng):
    img, d = canvas()
    cols = [(116, 142, 88, 255), (96, 122, 72, 255),
            (140, 164, 104, 255)]
    edge = (62, 82, 44, 255)
    for _ in range(11):
        x = rng.randint(9, 23)
        y = rng.randint(12, 23)
        s = rng.randint(2, 3)
        d.rectangle([x - s, y - s, x + s, y + s],
                    fill=cols[rng.randrange(len(cols))], outline=edge)
    speck(d, rng, [7, 11, 25, 25], [(178, 220, 140, 255)], 7, 0)
    return img


def draw_dried_salad(rng):
    img, d = canvas()
    box = (186, 148, 96, 255)
    box_d = (150, 116, 70, 255)
    d.polygon([(6, 13), (26, 13), (24, 27), (8, 27)], fill=box,
              outline=INK)
    d.polygon([(6, 13), (26, 13), (28, 9), (4, 9)], fill=box_d,
              outline=INK)
    speck(d, rng, [8, 10, 24, 13], [(110, 150, 70, 255),
                                    (216, 130, 54, 255)], 8, 0)
    # mayo + ketchup packets leaning at the front
    d.polygon([(9, 20), (14, 20), (14, 26), (9, 26)],
              fill=(244, 242, 236, 255), outline=INK)
    d.polygon([(18, 20), (23, 20), (23, 26), (18, 26)],
              fill=(196, 52, 40, 255), outline=INK)
    return img


def draw_starch(rng):
    img, d = canvas()
    col = (206, 206, 200, 255)
    d.polygon([(8, 18), (12, 13), (19, 12), (24, 16), (25, 21),
               (21, 25), (12, 26), (8, 23)], fill=col, outline=INK)
    d.ellipse([13, 15, 18, 18], fill=(228, 228, 222, 255))
    d.point((15, 16), fill=(246, 246, 242, 255))
    return img


def draw_roasted_cattail_rhizome(rng):
    img, d = canvas()
    d.rectangle([6, 13, 26, 20], fill=(52, 48, 46, 255),
                outline=INK)
    d.polygon([(10, 14), (22, 14), (20, 17), (12, 17)],
              fill=(226, 214, 186, 255))   # split showing dry inside
    speck(d, rng, [7, 18, 25, 20], [(30, 28, 26, 255)], 5, 0)
    return img


def draw_plantain_cooked(rng):
    img, d = canvas()
    peel = (146, 104, 40, 255)
    flesh = (240, 196, 92, 255)
    d.polygon([(7, 10), (12, 8), (22, 12), (25, 18), (22, 23),
               (13, 24), (8, 19)], fill=peel, outline=INK)
    d.polygon([(10, 12), (14, 10), (21, 13), (22, 17), (19, 20),
               (13, 20), (10, 16)], fill=flesh)
    speck(d, rng, [11, 12, 21, 19], [(176, 118, 44, 255)], 5, 0)
    return img


def draw_sushi_veggyroll(rng):
    img, d = canvas()
    plate(d)
    wrap = (52, 96, 52, 255)
    rice = (246, 244, 238, 255)
    centers = [(226, 120, 44, 255), (96, 176, 80, 255),
               (208, 56, 44, 255)]
    for (cx, cy), cen in zip(((11, 16), (20, 14), (16, 22)), centers):
        d.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=wrap,
                  outline=INK)
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=rice)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=cen)
    return img


def draw_veggy_aspic(rng):
    img, d = canvas()
    plate(d)
    jelly = (178, 216, 156, 255)
    jelly_d = (148, 192, 124, 255)
    d.pieslice([7, 4, 25, 26], start=180, end=360, fill=jelly,
               outline=INK)
    d.rectangle([7, 14, 25, 15], fill=jelly)
    d.line([(7, 15), (25, 15)], fill=INK)
    # vegetables suspended inside the mold
    speck(d, rng, [9, 8, 23, 14], [(226, 120, 44, 255),
                                   (84, 156, 74, 255),
                                   (240, 200, 84, 255)], 9, 1)
    d.line([(11, 6), (10, 13)], fill=(226, 244, 210, 255))
    return img


DRAW = {
    "bamboo_cooked": draw_bamboo_cooked,
    "bamboo_fried": draw_bamboo_fried,
    "buckwheat_cooked": draw_buckwheat_cooked,
    "burdock_cooked": draw_burdock_cooked,
    "burdock_fried": draw_burdock_fried,
    "carrot_glazed": draw_carrot_glazed,
    "carrot_roasted": draw_carrot_roasted,
    "cooked_bell_pepper": draw_cooked_bell_pepper,
    "cooked_burdock_stalk": draw_cooked_burdock_stalk,
    "cooked_cattail_stalk": draw_cooked_cattail_stalk,
    "cooked_pumpkin": draw_cooked_pumpkin,
    "corn_on_cob": draw_corn_on_cob,
    "dahlia_baked": draw_dahlia_baked,
    "dandelion_cooked": draw_dandelion_cooked,
    "dandelion_fried": draw_dandelion_fried,
    "dandelionburdock_fried": draw_dandelionburdock_fried,
    "deluxe_veggy_beans": draw_deluxe_veggy_beans,
    "deluxe_veggy_beansnrice": draw_deluxe_veggy_beansnrice,
    "deluxe_veggy_rice": draw_deluxe_veggy_rice,
    "dried_salad": draw_dried_salad,
    "dry_chili": draw_dry_chili,
    "dry_garlic": draw_dry_garlic,
    "dry_rice": draw_dry_rice,
    "dry_veggy": draw_dry_veggy,
    "dry_veggy_tainted": draw_dry_veggy_tainted,
    "dry_wild_rice": draw_dry_wild_rice,
    "hominy": draw_hominy,
    "jerusalem_artichoke_boiled": draw_jerusalem_artichoke_boiled,
    "jerusalem_artichoke_sauteed": draw_jerusalem_artichoke_sauteed,
    "kombu": draw_kombu,
    "lotus_rhizome_boiled": draw_lotus_rhizome_boiled,
    "melitzanosalata": draw_melitzanosalata,
    "nopalitos": draw_nopalitos,
    "onion_sauteed": draw_onion_sauteed,
    "pickle": draw_pickle,
    "plantain_cooked": draw_plantain_cooked,
    "rehydrated_chili": draw_rehydrated_chili,
    "rehydrated_garlic": draw_rehydrated_garlic,
    "rehydrated_veggy": draw_rehydrated_veggy,
    "roasted_cattail_rhizome": draw_roasted_cattail_rhizome,
    "salsa": draw_salsa,
    "salsify_baked": draw_salsify_baked,
    "sauerkraut": draw_sauerkraut,
    "sauerkraut_onions": draw_sauerkraut_onions,
    "starch": draw_starch,
    "succotash": draw_succotash,
    "succotash_deluxe": draw_succotash_deluxe,
    "sushi_rice": draw_sushi_rice,
    "sushi_veggyroll": draw_sushi_veggyroll,
    "tofu_rice": draw_tofu_rice,
    "tofu_stirfry": draw_tofu_stirfry,
    "veggy_aspic": draw_veggy_aspic,
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
                       "preview_item26.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item26.png {sheet.size}")


if __name__ == "__main__":
    main()
