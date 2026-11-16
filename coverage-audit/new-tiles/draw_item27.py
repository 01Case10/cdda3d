#!/usr/bin/env python3
"""Batch ITEM27 (Phase 2g.4 scope extension): vegetable
dishes, final set -- the generic veggy preparations and
cooked wild rice.

7 ids (authoritative list embedded below; source list:
~/workspace/item-batches/27.txt). All seven are defined
in data/json/items/comestibles/veggy_dishes.json.

Entries read first, copy-from chains resolved:
- veggy_canned: "canned veggy" -- "This mushy pile of
  vegetable matter was boiled and canned in an earlier
  life." Container can_food, color green. Drawn as an
  open tin can with mushy olive-green vegetable matter
  mounded over the rim and one drip down the front, a
  peeled lid flap bent back.
- veggy_pickled: "pickled veggy" -- "A serving of
  crisply brined and canned vegetable matter."
  Container jar_glass_sealed, color light_green,
  copy-from the veggy_raw abstract. Drawn as a sealed
  glass jar: ridged metal cap, pale glass, light-green
  brine with cucumber rounds, carrot sticks and onion
  bits inside.
- veggy_pickled_fried: "fried pickled veggy" -- "A
  serving of fried pickled vegetables, tangy with a
  crunchy golden-brown coating." Drawn as a small plate
  of golden-brown battered pieces, green pickle showing
  at the broken ends of two.
- veggy_salad: "vegetable salad" -- "A salad made with
  all kind of vegetables." Container bowl_plastic,
  color green. Drawn as a pale plastic bowl heaped with
  mixed leaves, tomato wedges, carrot shreds and
  cucumber slices.
- veggy_salted: "salted veggy chunk" -- "Vegetable
  chunks pickled in a salt bath." Container
  plastic_bag_vac, color green. Drawn as a crimp-sealed
  vacuum bag packed with green vegetable chunks and
  white salt crystals.
- veggy_wild_cooked: "cooked wild vegetables" --
  "Cooked wild edible plants.  An interesting mix of
  flavors." Copy-from veggy_raw (green), EATEN_HOT.
  Drawn as a loose heap of wilted cooked wild greens --
  varied leaf shapes, stems, one small yellow flower --
  with two pale steam wisps.
- wild_rice_cooked: "cooked wild rice" -- "A hearty
  serving of cooked wild rice." Color brown,
  EATEN_HOT. Drawn as a dark bowl heaped with long
  dark-brown wild rice grains with tan tips, faint
  steam wisps.

Glass/plastic translucency is faked with pale opaque
colors (this Pillow build replaces rather than
blends); all sprite alpha is in {0, 255}, asserted
per sprite.

Writes ONLY its own 7 folders + preview_item27.png.
PIL only; per-sprite random.Random(crc32(id)).
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "veggy_canned",
    "veggy_pickled",
    "veggy_pickled_fried",
    "veggy_salad",
    "veggy_salted",
    "veggy_wild_cooked",
    "wild_rice_cooked",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.\n")

INK = (24, 26, 24, 255)
# greens
LEAF_D = (43, 102, 40, 255)
LEAF_M = (70, 142, 54, 255)
LEAF_L = (118, 186, 88, 255)
MUSH = (100, 124, 56, 255)
MUSH_D = (74, 96, 42, 255)
MUSH_L = (140, 160, 84, 255)
BRINE = (172, 208, 128, 255)
COOK_D = (48, 88, 44, 255)
COOK_M = (66, 116, 52, 255)
# vegetables accents
CARROT = (226, 126, 36, 255)
CARROT_D = (188, 96, 24, 255)
TOMATO = (206, 52, 40, 255)
TOMATO_D = (164, 36, 30, 255)
CUKE = (196, 228, 160, 255)
CUKE_RIM = (84, 148, 66, 255)
ONION = (232, 226, 205, 255)
# materials
STEEL = (168, 172, 178, 255)
STEEL_D = (110, 114, 120, 255)
STEEL_L = (206, 210, 216, 255)
GLASS = (206, 232, 216, 255)
GLASS_D = (148, 186, 162, 255)
PLASTIC = (196, 210, 220, 255)
PLASTIC_D = (140, 158, 172, 255)
BAG = (186, 206, 214, 255)
BAG_D = (128, 152, 164, 255)
GOLD = (206, 148, 52, 255)
GOLD_D = (160, 106, 30, 255)
GOLD_L = (238, 190, 96, 255)
PLATE = (208, 212, 216, 255)
PLATE_D = (148, 154, 160, 255)
BOWL = (96, 66, 48, 255)
BOWL_D = (64, 42, 30, 255)
BOWL_L = (138, 100, 70, 255)
RICE = (74, 52, 36, 255)
RICE_D = (48, 34, 24, 255)
RICE_TIP = (186, 152, 104, 255)
STEAM = (216, 222, 224, 255)
SALT = (244, 246, 244, 255)


def chunk(d, cx, cy, r, fill, rng, squash=0.75):
    """An irregular rounded vegetable chunk."""
    pts = []
    n = 7
    for i in range(n):
        ang = 6.2832 * i / n
        rr = r * rng.uniform(0.78, 1.12)
        pts.append((cx + rr * 0.99 * __import__("math").cos(ang),
                    cy + rr * squash * __import__("math").sin(ang)))
    d.polygon(pts, fill=fill)


def leaf(d, cx, cy, w, h, fill, ang_deg=0):
    import math
    a = math.radians(ang_deg)
    ux, uy = math.cos(a), math.sin(a)
    vx, vy = -uy, ux
    tip = (cx + ux * w / 2, cy + uy * w / 2)
    base = (cx - ux * w / 2, cy - uy * w / 2)
    left = (cx + vx * h / 2, cy + vy * h / 2)
    right = (cx - vx * h / 2, cy - vy * h / 2)
    d.polygon([base, left, tip, right], fill=fill)
    d.line([base, tip], fill=COOK_D)


def steam_wisp(d, x, y):
    d.point((x, y), fill=STEAM)
    d.point((x + 1, y - 2), fill=STEAM)
    d.point((x, y - 4), fill=STEAM)
    d.point((x - 1, y - 6), fill=STEAM)


def draw_canned(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # peeled lid flap bent back on the right
    d.polygon([(23, 10), (29, 6), (30, 10), (25, 13)], fill=STEEL_L)
    d.polygon([(23, 10), (29, 6), (30, 10), (25, 13)], outline=STEEL_D)
    # can body
    d.rectangle([8, 12, 24, 29], fill=STEEL)
    d.rectangle([8, 12, 11, 29], fill=STEEL_L)
    d.rectangle([21, 12, 24, 29], fill=STEEL_D)
    # faded label band
    d.rectangle([8, 19, 24, 25], fill=(178, 148, 104, 255))
    d.rectangle([8, 21, 24, 22], fill=MUSH_D)
    d.rectangle([8, 28, 24, 29], fill=STEEL_D)
    # dark opening + mushy mound over the rim
    d.ellipse([8, 9, 24, 15], fill=(52, 54, 50, 255))
    chunk(d, 13, 9, 4.4, MUSH, rng)
    chunk(d, 19, 8, 4.8, MUSH, rng)
    chunk(d, 16, 10, 4.2, MUSH_L, rng)
    chunk(d, 11, 11, 3.2, MUSH_D, rng)
    chunk(d, 21, 11, 3.0, MUSH_D, rng)
    # mush speckles: carrot and pale celery bits
    for (x, y, c) in [(14, 8, CARROT), (19, 9, CARROT_D), (16, 7, ONION),
                      (12, 10, CARROT), (20, 11, ONION)]:
        d.point((x, y), fill=c)
        d.point((x + 1, y), fill=c)
    # one drip oozing down the front
    d.line([(15, 13), (15, 18)], fill=MUSH, width=2)
    d.point((15, 19), fill=MUSH_D)
    return img


def draw_pickled(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # ridged metal cap
    d.rectangle([10, 4, 22, 8], fill=STEEL)
    for x in range(11, 22, 2):
        d.line([(x, 4), (x, 8)], fill=STEEL_D)
    d.rectangle([10, 8, 22, 9], fill=STEEL_D)
    # glass jar body
    d.rounded_rectangle([9, 9, 23, 29], radius=3, fill=GLASS)
    # brine inside
    d.rounded_rectangle([11, 12, 21, 27], radius=2, fill=BRINE)
    # contents: cucumber rounds, carrot sticks, onion bits
    for (x, y) in [(13, 15), (18, 18), (14, 23)]:
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=CUKE, outline=CUKE_RIM)
    for (x, y) in [(19, 13), (12, 19)]:
        d.line([(x, y), (x + 1, y + 4)], fill=CARROT, width=2)
    for (x, y) in [(16, 21), (19, 25), (13, 26)]:
        d.point((x, y), fill=ONION)
        d.point((x + 1, y), fill=ONION)
    # glass outline + highlight
    d.rounded_rectangle([9, 9, 23, 29], radius=3, outline=GLASS_D)
    d.line([(11, 11), (11, 24)], fill=(240, 250, 242, 255), width=1)
    return img


def draw_pickled_fried(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # plate
    d.ellipse([4, 22, 28, 29], fill=PLATE)
    d.ellipse([4, 22, 28, 29], outline=PLATE_D)
    d.ellipse([7, 23, 25, 28], fill=(188, 194, 200, 255))
    # battered pieces piled on the plate
    pieces = [(11, 19, 4.6), (17, 17, 5.0), (22, 20, 4.4),
              (14, 22, 4.2), (20, 23, 4.0), (16, 20, 3.6)]
    for (x, y, r) in pieces:
        chunk(d, x, y, r, GOLD, rng)
        chunk(d, x - 1, y - 1, r * 0.55, GOLD_L, rng)
    # crust speckles
    for _ in range(16):
        x = rng.randrange(8, 25)
        y = rng.randrange(14, 25)
        d.point((x, y), fill=GOLD_D)
    # green pickle showing at two broken ends
    d.ellipse([8, 17, 12, 20], fill=CUKE_RIM)
    d.ellipse([9, 17, 11, 19], fill=CUKE)
    d.ellipse([21, 14, 25, 17], fill=CUKE_RIM)
    d.ellipse([22, 15, 24, 17], fill=CUKE)
    return img


def draw_salad(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # salad heap first (bowl drawn over its base)
    leaf(d, 11, 13, 10, 6, LEAF_M, -25)
    leaf(d, 21, 13, 10, 6, LEAF_D, 20)
    leaf(d, 16, 11, 11, 7, LEAF_L, 0)
    leaf(d, 13, 16, 9, 5, LEAF_D, -10)
    leaf(d, 20, 16, 9, 5, LEAF_M, 15)
    # tomato wedges
    for (x, y) in [(10, 14), (22, 11), (17, 15)]:
        d.polygon([(x, y), (x + 4, y + 1), (x + 1, y + 4)], fill=TOMATO)
        d.point((x + 1, y + 1), fill=TOMATO_D)
    # cucumber slices
    for (x, y) in [(14, 12), (20, 18)]:
        d.ellipse([x - 2, y - 2, x + 2, y + 2], fill=CUKE, outline=CUKE_RIM)
    # carrot shreds
    for (x, y) in [(12, 17), (18, 12), (21, 15), (15, 18)]:
        d.line([(x, y), (x + 2, y + 1)], fill=CARROT, width=1)
    # plastic bowl
    d.polygon([(5, 19), (27, 19), (24, 28), (8, 28)], fill=PLASTIC)
    d.polygon([(5, 19), (27, 19), (24, 28), (8, 28)], outline=PLASTIC_D)
    d.rectangle([4, 17, 28, 19], fill=PLASTIC)
    d.rectangle([4, 17, 28, 18], fill=(228, 238, 244, 255))
    d.line([(9, 21), (23, 21)], fill=PLASTIC_D)
    return img


def draw_salted(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # vacuum bag body
    d.rounded_rectangle([7, 8, 25, 29], radius=4, fill=BAG)
    # packed vegetable chunks inside
    for (x, y, r, c) in [(12, 14, 3.6, LEAF_M), (19, 13, 3.8, LEAF_D),
                         (15, 19, 4.2, LEAF_M), (21, 20, 3.4, MUSH),
                         (11, 22, 3.4, LEAF_D), (17, 25, 3.8, LEAF_L),
                         (22, 25, 2.8, MUSH_D)]:
        chunk(d, x, y, r, c, rng)
    # salt crystals
    for _ in range(14):
        x = rng.randrange(9, 24)
        y = rng.randrange(11, 28)
        d.point((x, y), fill=SALT)
    # crimped seal at the top
    d.rectangle([7, 6, 25, 9], fill=BAG_D)
    for x in range(8, 25, 2):
        d.point((x, 7), fill=BAG)
    # bag outline + sheen
    d.rounded_rectangle([7, 8, 25, 29], radius=4, outline=BAG_D)
    d.line([(10, 11), (10, 24)], fill=(232, 242, 246, 255))
    return img


def draw_wild_cooked(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    steam_wisp(d, 11, 9)
    steam_wisp(d, 21, 8)
    # heap of wilted cooked wild greens, loose pile
    leaf(d, 10, 20, 13, 6, COOK_D, -18)
    leaf(d, 22, 20, 13, 6, COOK_M, 16)
    leaf(d, 16, 17, 14, 7, COOK_M, -4)
    leaf(d, 12, 24, 12, 6, COOK_M, 10)
    leaf(d, 20, 24, 12, 6, COOK_D, -12)
    leaf(d, 16, 21, 12, 6, LEAF_D, 0)
    # stems poking out
    for (x, y) in [(8, 26), (24, 26), (16, 27)]:
        d.line([(x, y), (x + 1, y + 2)], fill=(120, 140, 84, 255), width=1)
    # a small yellow wild flower and a couple of berries of detail
    d.point((23, 15), fill=(240, 204, 64, 255))
    d.point((22, 16), fill=(240, 204, 64, 255))
    d.point((24, 16), fill=(240, 204, 64, 255))
    d.point((23, 17), fill=(240, 204, 64, 255))
    # glossy cooked highlights
    for (x, y) in [(13, 19), (18, 22), (15, 24)]:
        d.point((x, y), fill=(150, 190, 120, 255))
    return img


def draw_wild_rice(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    steam_wisp(d, 12, 8)
    steam_wisp(d, 20, 7)
    # heaped grains above the bowl rim
    for _ in range(46):
        x = rng.randrange(8, 24)
        y = rng.randrange(10, 17)
        c = RICE if rng.random() < 0.7 else RICE_D
        d.line([(x, y), (x + 3, y + 1)], fill=c, width=1)
        if rng.random() < 0.4:
            d.point((x + 3, y + 1), fill=RICE_TIP)
    d.ellipse([7, 12, 25, 18], fill=RICE_D)
    for _ in range(30):
        x = rng.randrange(8, 24)
        y = rng.randrange(12, 17)
        d.line([(x, y), (x + 3, y)], fill=RICE, width=1)
    # bowl
    d.polygon([(5, 17), (27, 17), (23, 27), (9, 27)], fill=BOWL)
    d.polygon([(5, 17), (27, 17), (23, 27), (9, 27)], outline=BOWL_D)
    d.rectangle([4, 15, 28, 17], fill=BOWL_L)
    d.rectangle([12, 28, 20, 29], fill=BOWL_D)
    return img


RENDER = {
    "veggy_canned": draw_canned,
    "veggy_pickled": draw_pickled,
    "veggy_pickled_fried": draw_pickled_fried,
    "veggy_salad": draw_salad,
    "veggy_salted": draw_salted,
    "veggy_wild_cooked": draw_wild_cooked,
    "wild_rice_cooked": draw_wild_rice,
}


def main():
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
    cols, cell = 4, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item27.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item27.png {sheet.size}")


if __name__ == "__main__":
    main()
