#!/usr/bin/env python3
"""Batch ITEM20 (Phase 2g.4 scope extension): metal
resources, first set.

52 ids from data/json/items/resources/metal.json, plus
slag from data/json/materials.json (authoritative list
embedded below; source list:
~/workspace/item-batches/20.txt). Steel grades inherit
their form from generic steel abstracts via copy-from
(10l_steel / 5l_steel / steel_bar_flat / steel_bar_round /
small_steel_bar_*); each grade is shaded by its material.

The set, from the entries' names/descriptions/materials:

- 1 L ingots (aluminum, brass, bronze, copper, gold,
  lead, platinum, silver, tin, zinc): "a one-liter ingot
  of solid <metal>" -- one trapezoid ingot per metal in
  its data color (brass/bronze/gold yellow, copper brown,
  the rest silvery grays; zinc near-white).
- 5 L / 10 L steel ingots (budget, high, mild, medium):
  the same trapezoid scaled up by volume, shaded per
  grade -- budget dull and rough, mild soft gray, medium
  mid gray, high darker blue-gray.
- chunks (budget / hardened / high / mild / medium /
  tempered steel, cast iron): "a misshapen chunk ...
  makes a decent weapon in a pinch" -- jagged irregular
  blobs with facet highlights; cast iron darkest and
  roughest, tempered with a straw-blue tint.
- lumps (same grades + cast iron): "a heavy, formed
  piece" -- smoother rounded blobs, same palettes.
- flat bar stock (aluminum, high / mild / medium steel,
  plus small aluminum and small high-steel): "a
  rectangular bar ... ready to be milled" -- long flat
  bars with a lit top face; small ones shorter/thinner.
- round bar stock (same families): "a round bar ...
  ready to be turned on the lathe" -- horizontal
  cylinders with a bright core line.
- scrap (brass, cast iron, tin): "small pieces of scrap"
  -- a scatter of little jagged shards.
- sheet_aluminum: "a large sheet of aluminum" -- a broad
  flat parallelogram with a thin bright edge.
- fanblade_metal: "a multi-bladed metal fan" -- five
  blades radiating from a hub, light budget steel.
- railroad_track_item / _half: a one-meter (or
  half-meter) length of rail -- the classic rail profile
  (head, web, flange) run horizontally, end face showing
  the I cross-section; the half is shorter.
- slag (materials.json): the smelting by-product as an
  item -- a dark, glassy, porous lump full of pinholes.

Writes ONLY its own 52 folders + preview_item20.png.
PIL only; per-sprite random.Random(crc32(id)); item
bodies fully opaque (alpha in {0, 255}), background
transparent.
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageChops, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "10l_budget_steel", "10l_hc_steel", "10l_lc_steel", "10l_mc_steel",
    "1l_aluminum", "1l_brass", "1l_bronze", "1l_copper", "1l_gold",
    "1l_lead", "1l_platinum", "1l_silver", "1l_tin", "1l_zinc",
    "5l_budget_steel", "5l_hc_steel", "5l_lc_steel", "5l_mc_steel",
    "aluminum_bar_flat", "aluminum_bar_round",
    "budget_steel_chunk", "budget_steel_lump",
    "ch_steel_chunk", "ch_steel_lump", "chunk_cast_iron",
    "fanblade_metal",
    "hc_steel_bar_flat", "hc_steel_bar_round",
    "hc_steel_chunk", "hc_steel_lump",
    "lc_steel_bar_flat", "lc_steel_bar_round",
    "lc_steel_chunk", "lc_steel_lump", "lump_cast_iron",
    "mc_steel_bar_flat", "mc_steel_bar_round",
    "mc_steel_chunk", "mc_steel_lump",
    "qt_steel_chunk", "qt_steel_lump",
    "railroad_track_item", "railroad_track_item_half",
    "scrap_brass", "scrap_cast_iron", "scrap_tin",
    "sheet_aluminum", "slag",
    "small_aluminum_bar_flat", "small_aluminum_bar_round",
    "small_hc_steel_bar_flat", "small_hc_steel_bar_round",
]

# (base, dark, light) per metal / steel grade
PAL = {
    "aluminum":     ((176, 196, 204), (118, 140, 150), (224, 238, 242)),
    "brass":        ((198, 160, 58),  (138, 106, 34),  (244, 208, 112)),
    "bronze":       ((168, 118, 52),  (110, 74, 30),   (218, 168, 96)),
    "copper":       ((186, 102, 60),  (126, 62, 34),   (234, 152, 102)),
    "gold":         ((234, 186, 46),  (166, 126, 22),  (255, 228, 122)),
    "lead":         ((118, 122, 132), (80, 84, 94),    (162, 166, 176)),
    "platinum":     ((206, 209, 216), (148, 151, 160), (242, 244, 249)),
    "silver":       ((199, 201, 206), (144, 146, 152), (240, 242, 246)),
    "tin":          ((179, 183, 189), (126, 130, 137), (227, 230, 234)),
    "zinc":         ((216, 219, 221), (158, 162, 166), (246, 247, 248)),
    "budget_steel": ((126, 126, 129), (84, 84, 87),    (170, 170, 173)),
    "lc_steel":     ((151, 153, 157), (104, 106, 111), (194, 196, 200)),
    "mc_steel":     ((140, 143, 150), (95, 98, 105),   (186, 189, 196)),
    "hc_steel":     ((116, 123, 139), (76, 82, 97),    (170, 178, 198)),
    "ch_steel":     ((94, 99, 110),   (60, 64, 75),    (142, 148, 162)),
    "qt_steel":     ((128, 117, 138), (83, 75, 95),    (196, 172, 138)),
    "cast_iron":    ((90, 90, 94),    (56, 56, 60),    (132, 132, 136)),
    "slag":         ((56, 54, 59),    (32, 30, 34),    (99, 94, 100)),
}


def shade(img, d, poly, pal, rng, speckle=0):
    """Fill a polygon with the base tone, a darker lower
    band and a lighter upper facet, plus optional grain.
    The band overlay is clipped to the polygon mask so no
    semi-transparent pixels leak outside the body."""
    base, dark, light = pal
    d.polygon(poly, fill=base + (255,))
    xs = [p[0] for p in poly]
    ys = [p[1] for p in poly]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    mask = Image.new("L", img.size, 0)
    ImageDraw.Draw(mask).polygon(poly, fill=255)
    overlay = Image.new("RGBA", img.size, (0, 0, 0, 0))
    od = ImageDraw.Draw(overlay)
    mid = (y0 + y1) // 2
    od.polygon([(x, y) for (x, y) in poly if y >= mid] +
               [(x1, mid), (x0, mid)], fill=dark + (110,))
    od.polygon([(x, y) for (x, y) in poly if y <= y0 + (y1 - y0) // 3] +
               [(x1, y0 + (y1 - y0) // 3), (x0, y0 + (y1 - y0) // 3)],
               fill=light + (120,))
    oa = overlay.getchannel("A")
    overlay.putalpha(ImageChops.darker(oa, mask))
    img.alpha_composite(overlay)
    if speckle:
        px = img.load()
        for _ in range(speckle):
            x = rng.randint(int(x0) + 1, int(x1) - 1)
            y = rng.randint(int(y0) + 1, int(y1) - 1)
            if px[x, y][3] == 255:
                px[x, y] = (dark if rng.random() < 0.5 else light) + (255,)


def ingot(pal, w, h, rng, rough=0):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base, dark, light = pal
    cx, cy = 16, 17
    tw = w * 0.72  # top face narrower than the base
    top = [(cx - tw / 2, cy - h / 2 + 3), (cx + tw / 2, cy - h / 2 + 3),
           (cx + tw / 2 + 2, cy - h / 2), (cx - tw / 2 - 2, cy - h / 2)]
    body = [(cx - tw / 2, cy - h / 2 + 3), (cx + tw / 2, cy - h / 2 + 3),
            (cx + w / 2, cy + h / 2), (cx - w / 2, cy + h / 2)]
    d.polygon(body, fill=base + (255,))
    d.polygon([(cx - tw / 2, cy + 1), (cx + tw / 2, cy + 1),
               (cx + w / 2, cy + h / 2), (cx - w / 2, cy + h / 2)],
              fill=dark + (255,))
    d.polygon([(cx - tw / 2, cy - h / 2 + 3), (cx + tw / 2, cy - h / 2 + 3),
               (cx + tw / 2, cy + 1), (cx - tw / 2, cy + 1)],
              fill=base + (255,))
    d.polygon(top, fill=light + (255,))
    d.line([(cx - w / 2, cy + h / 2), (cx + w / 2, cy + h / 2)],
           fill=dark + (255,))
    if rough:
        px = img.load()
        for _ in range(rough):
            x = rng.randint(cx - w // 2 + 1, cx + w // 2 - 1)
            y = rng.randint(cy - h // 2 + 1, cy + h // 2 - 1)
            if px[x, y][3] == 255:
                px[x, y] = dark + (255,)
    return img


def blob(pal, rng, jag, r=9, speckle=0, pores=0):
    """Irregular lump: jag=0 smooth formed lump, jag=1
    misshapen chunk. Pores punch dark pits (slag)."""
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx, cy = 16, 16
    pts = []
    n = 9
    for i in range(n):
        a = 2 * math.pi * i / n
        rr = r + (rng.randint(-3, 3) if jag else rng.randint(-1, 1))
        pts.append((cx + rr * math.cos(a), cy + rr * 0.78 * math.sin(a)))
    shade(img, d, pts, pal, rng, speckle=speckle)
    if pores:
        px = img.load()
        for _ in range(pores):
            x = rng.randint(cx - r + 2, cx + r - 2)
            y = rng.randint(cy - r + 2, cy + r - 2)
            if px[x, y][3] == 255:
                px[x, y] = pal[1] + (255,)
                if rng.random() < 0.5 and px[x + 1, y][3] == 255:
                    px[x + 1, y] = pal[1] + (255,)
    return img


def bar_flat(pal, length, thick, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base, dark, light = pal
    x0, x1 = 16 - length // 2, 16 + length // 2
    y0 = 16 - thick // 2
    d.rectangle([x0, y0 + 2, x1, y0 + thick], fill=base + (255,))
    d.rectangle([x0, y0, x1, y0 + 2], fill=light + (255,))
    d.rectangle([x0, y0 + thick - 1, x1, y0 + thick], fill=dark + (255,))
    d.rectangle([x1 - 2, y0, x1, y0 + thick], fill=dark + (255,))
    return img


def bar_round(pal, length, thick, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base, dark, light = pal
    x0, x1 = 16 - length // 2, 16 + length // 2
    y0, y1 = 16 - thick // 2, 16 + thick // 2
    d.rounded_rectangle([x0, y0, x1, y1], radius=thick // 2,
                        fill=base + (255,))
    d.rectangle([x0 + thick // 2, y1 - 2, x1 - thick // 2, y1],
                fill=dark + (255,))
    d.line([(x0 + 2, 16 - 1), (x1 - 2, 16 - 1)], fill=light + (255,),
           width=2)
    d.ellipse([x1 - 3, y0 + 1, x1 - 1, y1 - 1], fill=dark + (255,))
    return img


def scrap(pal, rng, count=4):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    spots = [(10, 11), (21, 10), (13, 21), (22, 20), (16, 15)][:count]
    for (cx, cy) in spots:
        r = rng.randint(3, 5)
        pts = []
        for i in range(6):
            a = 2 * math.pi * i / 6 + rng.random() * 0.5
            rr = r + rng.randint(-1, 1)
            pts.append((cx + rr * math.cos(a), cy + rr * 0.7 * math.sin(a)))
        shade(img, d, pts, pal, rng, speckle=2)
    return img


def sheet(pal, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base, dark, light = pal
    poly = [(5, 10), (25, 7), (27, 22), (7, 25)]
    d.polygon(poly, fill=base + (255,))
    d.polygon([(7, 25), (27, 22), (27, 24), (7, 27)], fill=dark + (255,))
    for t in (0.3, 0.55, 0.8):
        x_a = 5 + (7 - 5) * t + 2
        y_a = 10 + (25 - 10) * t
        x_b = 25 + (27 - 25) * t - 2
        y_b = 7 + (22 - 7) * t
        d.line([(x_a, y_a), (x_b, y_b)], fill=light + (255,))
    d.line([(5, 10), (25, 7)], fill=light + (255,), width=2)
    return img


def fanblade(pal, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    base, dark, light = pal
    for k in range(5):
        a = 2 * math.pi * k / 5 - math.pi / 2
        blade = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        bd = ImageDraw.Draw(blade)
        cx, cy = 16 + 8 * math.cos(a), 16 + 8 * math.sin(a)
        px_, py_ = -math.sin(a), math.cos(a)
        pts = [(cx + 6 * math.cos(a) + 3 * px_, cy + 6 * math.sin(a) + 3 * py_),
               (cx + 6 * math.cos(a) - 3 * px_, cy + 6 * math.sin(a) - 3 * py_),
               (cx - 5 * math.cos(a) - 2 * px_, cy - 5 * math.sin(a) - 2 * py_),
               (cx - 5 * math.cos(a) + 2 * px_, cy - 5 * math.sin(a) + 2 * py_)]
        bd.polygon(pts, fill=base + (255,))
        bd.line([pts[0], pts[3]], fill=light + (255,))
        bd.line([pts[1], pts[2]], fill=dark + (255,))
        img.alpha_composite(blade)
    d = ImageDraw.Draw(img)
    d.ellipse([12, 12, 20, 20], fill=dark + (255,))
    d.ellipse([13, 13, 19, 19], fill=base + (255,))
    d.ellipse([15, 15, 17, 17], fill=dark + (255,))
    return img


def rail(pal, length):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base, dark, light = pal
    x0, x1 = 16 - length // 2, 16 + length // 2
    # side silhouette: head (top), web, flange (bottom)
    d.rectangle([x0, 10, x1, 13], fill=base + (255,))          # head
    d.rectangle([x0, 13, x1, 18], fill=dark + (255,))          # web shadow
    d.rectangle([x0 + 3, 13, x1 - 3, 18], fill=base + (255,))  # web
    d.rectangle([x0, 18, x1, 22], fill=base + (255,))          # flange
    d.rectangle([x0, 10, x1, 11], fill=light + (255,))         # running surface
    d.rectangle([x0, 21, x1, 22], fill=dark + (255,))
    # end face at right: I cross-section
    d.rectangle([x1 - 3, 9, x1, 23], fill=dark + (255,))
    d.rectangle([x1 - 3, 9, x1, 13], fill=base + (255,))
    d.rectangle([x1 - 2, 13, x1 - 1, 18], fill=base + (255,))
    d.rectangle([x1 - 3, 19, x1, 23], fill=base + (255,))
    return img


STEEL_GRADE = {"budget": "budget_steel", "lc": "lc_steel", "mc": "mc_steel",
               "hc": "hc_steel", "ch": "ch_steel", "qt": "qt_steel"}


def render(item_id, rng):
    # 1 L metal ingots
    if item_id.startswith("1l_"):
        metal = item_id[3:]
        return ingot(PAL[metal], 20, 12, rng,
                     rough=6 if metal in ("lead", "tin") else 0)
    # 5 L / 10 L steel ingots
    if item_id.startswith(("5l_", "10l_")):
        size = 5 if item_id.startswith("5l_") else 10
        grade = item_id.split("_")[1]
        pal = PAL[STEEL_GRADE[grade]]
        return ingot(pal, 22 if size == 5 else 26, 14 if size == 5 else 17,
                     rng, rough=10 if grade == "budget" else 0)
    # chunks and lumps
    if item_id.endswith("_chunk") or item_id == "chunk_cast_iron":
        if "cast_iron" in item_id:
            return blob(PAL["cast_iron"], rng, jag=1, r=9, speckle=26)
        grade = item_id.split("_")[0]
        return blob(PAL[STEEL_GRADE[grade]], rng, jag=1, r=9,
                    speckle=18 if grade == "budget" else 8)
    if item_id.endswith("_lump") or item_id == "lump_cast_iron":
        if "cast_iron" in item_id:
            return blob(PAL["cast_iron"], rng, jag=0, r=10, speckle=22)
        grade = item_id.split("_")[0]
        return blob(PAL[STEEL_GRADE[grade]], rng, jag=0, r=10,
                    speckle=12 if grade == "budget" else 4)
    # bar stock
    if "bar_flat" in item_id or "bar_round" in item_id:
        if "aluminum" in item_id:
            pal = PAL["aluminum"]
        else:
            pal = PAL["hc_steel"] if "_hc_" in item_id or item_id.startswith(
                ("hc_", "small_hc_")) else PAL["mc_steel"]
            if item_id.startswith("lc_"):
                pal = PAL["lc_steel"]
            elif item_id.startswith("mc_"):
                pal = PAL["mc_steel"]
        small = item_id.startswith("small_")
        length = 20 if small else 27
        thick = 5 if small else 7
        if "bar_flat" in item_id:
            return bar_flat(pal, length, thick, rng)
        return bar_round(pal, length, thick - 1, rng)
    if item_id.startswith("scrap_"):
        metal = {"brass": "brass", "cast_iron": "cast_iron",
                 "tin": "tin"}[item_id[6:]]
        return scrap(PAL[metal], rng, count=5 if metal == "cast_iron" else 4)
    if item_id == "sheet_aluminum":
        return sheet(PAL["aluminum"], rng)
    if item_id == "fanblade_metal":
        return fanblade(PAL["budget_steel"], rng)
    if item_id == "railroad_track_item":
        return rail(PAL["mc_steel"], 28)
    if item_id == "railroad_track_item_half":
        return rail(PAL["mc_steel"], 19)
    if item_id == "slag":
        return blob(PAL["slag"], rng, jag=1, r=10, speckle=10, pores=26)
    raise ValueError(item_id)


def main():
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = render(item_id, rng)
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
    sheet_out = Image.new("RGBA", (cols * cell, rows * cell),
                          (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet_out.alpha_composite(big, ((i % cols) * cell + 1,
                                        (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item20.png")
    sheet_out.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item20.png {sheet_out.size}")


if __name__ == "__main__":
    main()
