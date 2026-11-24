#!/usr/bin/env python3
"""ITEM43: seeds (items/comestibles/seed.json) -- CDDA Voxel coverage audit.

Draws each seed id as a small pile/scatter of the seeds themselves,
parameterized by the real seed's size, shape and color, with a hint of
the parent plant where it helps identity. Self-contained: the id list
is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "seed_artichoke",
    "seed_asparagus",
    "seed_bell_pepper",
    "seed_bottle_gourd",
    "seed_brussels_sprout",
    "seed_cauliflower",
    "seed_eggplant",
    "seed_garlic_clove",
    "seed_gherkin",
    "seed_groundnut",
    "seed_horseradish",
    "seed_japanese_knotweed",
    "seed_jerusalem_artichoke",
    "seed_kale",
    "seed_lotus",
    "seed_lotus_boiled",
    "seed_maianthemum_stellatum",
    "seed_mayapple",
    "seed_melon",
    "seed_mustard",
    "seed_pea",
    "seed_popcorn",
    "seed_poppy_bud",
    "seed_radish",
    "seed_raw_bamboo",
    "seed_raw_burdock",
    "seed_red_winter_wheat",
    "seed_spinach",
    "seed_watermelon",
    "seed_wild_garlic",
    "seed_wild_rice",
    "seed_wild_sarsaparilla",
    "seed_wintergreen",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def rot_ellipse_pts(cx, cy, rx, ry, ang, n=14):
    ca, sa = math.cos(ang), math.sin(ang)
    pts = []
    for i in range(n):
        t = 2 * math.pi * i / n
        x, y = rx * math.cos(t), ry * math.sin(t)
        pts.append((cx + x * ca - y * sa, cy + x * sa + y * ca))
    return pts


def draw_seed(d, cx, cy, rx, ry, ang, base, rng, mark=None):
    """One opaque seed: darker outline, base fill, optional marking."""
    edge = shade(base, 0.55)
    d.polygon(rot_ellipse_pts(cx, cy, rx + 0.8, ry + 0.8, ang), fill=edge)
    d.polygon(rot_ellipse_pts(cx, cy, rx, ry, ang), fill=base)
    # top-light crescent
    hx, hy = cx - rx * 0.3, cy - ry * 0.35
    d.polygon(rot_ellipse_pts(hx, hy, rx * 0.45, ry * 0.4, ang),
              fill=shade(base, 1.25))
    if mark == "groove":  # wheat-style crease along the long axis
        ca, sa = math.cos(ang), math.sin(ang)
        x1, y1 = cx - ca * rx * 0.8, cy - sa * rx * 0.8
        x2, y2 = cx + ca * rx * 0.8, cy + sa * rx * 0.8
        d.line([x1, y1, x2, y2], fill=shade(base, 0.6), width=1)
    elif mark == "rim":  # pale rim inside the edge (watermelon)
        d.polygon(rot_ellipse_pts(cx, cy, rx * 0.68, ry * 0.68, ang),
                  fill=shade(base, 1.5))
        d.polygon(rot_ellipse_pts(cx, cy, rx * 0.45, ry * 0.45, ang),
                  fill=base)
    elif mark == "stripe":  # sunflower-style length stripe
        ca, sa = math.cos(ang), math.sin(ang)
        off = rx * 0.35
        x1, y1 = cx - ca * rx * 0.7 - sa * off, cy - sa * rx * 0.7 + ca * off
        x2, y2 = cx + ca * rx * 0.7 - sa * off, cy + sa * rx * 0.7 + ca * off
        d.line([x1, y1, x2, y2], fill=shade(base, 1.6), width=1)


def pile(img, rng, rx, ry, count, colors, mark=None, spread=(11, 6.5),
         center=(16, 18)):
    """A loose mound of seeds, lower rows drawn last (overlap downward)."""
    d = ImageDraw.Draw(img)
    spots = []
    for _ in range(count):
        # denser toward the middle of the mound
        a = rng.uniform(0, 2 * math.pi)
        r = math.sqrt(rng.uniform(0, 1))
        x = center[0] + math.cos(a) * r * spread[0]
        y = center[1] + math.sin(a) * r * spread[1] - (1 - r) * 1.5
        spots.append((y, x))
    spots.sort()
    for y, x in spots:
        ang = rng.uniform(0, math.pi)
        base = rng.choice(colors)
        jr = 1 + rng.uniform(-0.15, 0.15)
        draw_seed(d, x, y, rx * jr, ry * jr, ang, base, rng, mark)


def leaf_hint(img, color=(58, 122, 44, 255), pos=(6, 7), flip=False):
    """A small parent-plant leaf in a top corner."""
    d = ImageDraw.Draw(img)
    x, y = pos
    s = -1 if flip else 1
    pts = [(x, y), (x + 6 * s, y - 3), (x + 9 * s, y + 2), (x + 4 * s, y + 4),
           (x + 1 * s, y + 3)]
    d.polygon(pts, fill=color)
    d.line([x, y + 1, x + 8 * s, y + 1], fill=shade(color, 0.6), width=1)


def garlic_cloves(img, rng):
    """Several teardrop cloves, papery ivory skin, pointed tips up."""
    d = ImageDraw.Draw(img)
    cols = [(232, 222, 196, 255), (222, 208, 176, 255), (238, 230, 208, 255)]
    layout = [(10, 20, -0.25), (16, 22, 0.05), (22, 20, 0.3), (13, 14, -0.1),
              (19, 14, 0.15)]
    for cx, cy, tilt in layout:
        c = rng.choice(cols)
        w, h = 4.6, 7.5
        pts = []
        for i in range(16):
            t = 2 * math.pi * i / 16
            # teardrop: narrow at top, round at bottom
            px = math.sin(t) * w * (0.55 + 0.45 * (math.cos(t) * -0.5 + 0.5))
            py = math.cos(t) * h * 0.5
            pts.append((cx + px * math.cos(tilt) - py * math.sin(tilt),
                        cy + px * math.sin(tilt) + py * math.cos(tilt)))
        d.polygon([(x + 0.7, y + 0.7) for x, y in pts], fill=shade(c, 0.6))
        d.polygon(pts, fill=c)
        tip = pts[8]
        d.line([pts[4], tip], fill=shade(c, 0.75), width=1)
        d.line([tip, (tip[0] + 1, tip[1] - 2)], fill=shade(c, 0.7), width=1)


def knotweed_stems(img, rng):
    """Green stem cuttings with red speckles and pale node rings."""
    d = ImageDraw.Draw(img)
    green = (96, 148, 62, 255)
    for (x0, y0, x1, y1) in [(6, 24, 22, 8), (10, 27, 27, 13), (5, 18, 19, 5)]:
        d.line([x0, y0, x1, y1], fill=shade(green, 0.6), width=5)
        d.line([x0, y0, x1, y1], fill=green, width=3)
        for t in (0.33, 0.66):  # node rings
            nx, ny = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            d.ellipse([nx - 2.5, ny - 2.5, nx + 2.5, ny + 2.5],
                      outline=(206, 198, 160, 255), width=1)
        for _ in range(7):  # red speckles
            t = rng.uniform(0.05, 0.95)
            sx = x0 + (x1 - x0) * t + rng.uniform(-1, 1)
            sy = y0 + (y1 - y0) * t + rng.uniform(-1, 1)
            d.point((sx, sy), fill=(168, 52, 44, 255))
        d.ellipse([x1 - 2, y1 - 2, x1 + 2, y1 + 2], fill=(214, 206, 168, 255))


def poppy_bud(img, rng):
    """Green ribbed poppy seed head with tiny dark seeds spilling out."""
    d = ImageDraw.Draw(img)
    body = (104, 148, 74, 255)
    d.ellipse([10, 8, 23, 24], fill=shade(body, 0.6))
    d.ellipse([9, 7, 22, 23], fill=body)
    for dx in (-4, -1.5, 1.5, 4):
        d.line([15.5 + dx, 9, 15.5 + dx, 21], fill=shade(body, 0.75), width=1)
    d.ellipse([11, 5, 20, 9], fill=shade(body, 1.2))  # crown disc
    for a in range(0, 360, 45):
        x = 15.5 + 3.4 * math.cos(math.radians(a))
        y = 7 + 1.4 * math.sin(math.radians(a))
        d.point((x, y), fill=shade(body, 0.65))
    darks = [(38, 36, 44, 255), (52, 50, 58, 255)]
    for _ in range(26):
        x = rng.gauss(17, 5.5)
        y = rng.gauss(26.5, 1.8)
        if 3 <= x <= 29 and 22 <= y <= 30:
            d.ellipse([x - 1, y - 0.7, x + 1, y + 0.7], fill=rng.choice(darks))


def burdock_bur(img, rng):
    """A round hooked bur with a few brown achenes beside it."""
    d = ImageDraw.Draw(img)
    bur = (122, 96, 58, 255)
    d.ellipse([6, 6, 20, 20], fill=shade(bur, 0.65))
    d.ellipse([5, 5, 19, 19], fill=bur)
    for a in range(0, 360, 24):
        x1 = 12 + 6.5 * math.cos(math.radians(a))
        y1 = 12 + 6.5 * math.sin(math.radians(a))
        x2 = 12 + 9 * math.cos(math.radians(a))
        y2 = 12 + 9 * math.sin(math.radians(a))
        d.line([x1, y1, x2, y2], fill=shade(bur, 0.8), width=1)
        d.point((x2, y2), fill=shade(bur, 1.25))
    ach = [(140, 108, 66, 255), (124, 94, 54, 255)]
    for (x, y, a) in [(23, 22, 0.4), (26, 25, 1.2), (21, 26, 2.2)]:
        draw_seed(d, x, y, 2.6, 1.5, a, rng.choice(ach), rng)


# profile: (seed rx, ry, count, colors, mark, hint)
P = {
    # medium tan-gray ovals
    "seed_artichoke": (3.0, 2.0, 14, [(168, 150, 116, 255), (150, 130, 98, 255)], None, "leaf"),
    # small round near-black
    "seed_asparagus": (1.8, 1.8, 30, [(40, 38, 42, 255), (56, 52, 54, 255)], None, None),
    # flat pale cream discs
    "seed_bell_pepper": (2.8, 2.0, 16, [(232, 222, 188, 255), (220, 208, 168, 255)], None, None),
    # flat elongated cream, larger
    "seed_bottle_gourd": (3.6, 2.0, 11, [(226, 210, 172, 255), (208, 190, 148, 255)], None, "leaf"),
    # tiny round dark brown brassica
    "seed_brussels_sprout": (1.6, 1.6, 34, [(74, 56, 44, 255), (90, 70, 54, 255)], None, "leaf"),
    # tiny round mid-brown brassica
    "seed_cauliflower": (1.6, 1.6, 34, [(104, 78, 56, 255), (120, 94, 66, 255)], None, None),
    # small flat yellowish-tan discs
    "seed_eggplant": (2.6, 1.8, 18, [(196, 164, 104, 255), (182, 148, 90, 255)], None, None),
    # cucumber: pale elongated pointed ovals
    "seed_gherkin": (3.2, 1.7, 14, [(234, 228, 200, 255), (220, 212, 178, 255)], None, None),
    # small reddish-brown beans
    "seed_groundnut": (2.8, 2.1, 15, [(146, 92, 60, 255), (128, 78, 50, 255)], None, None),
    # small round brown
    "seed_horseradish": (1.9, 1.9, 26, [(128, 96, 62, 255), (146, 112, 74, 255)], None, None),
    # small brown seeds (sunchoke)
    "seed_jerusalem_artichoke": (2.6, 1.8, 16, [(158, 124, 82, 255), (140, 108, 70, 255)], None, "leaf"),
    # tiny dark brassica + curly leaf
    "seed_kale": (1.6, 1.6, 34, [(52, 46, 44, 255), (68, 60, 56, 255)], None, "leaf"),
    # big hard dark olive ovals
    "seed_lotus": (4.2, 3.2, 9, [(84, 88, 54, 255), (66, 70, 44, 255)], None, None),
    # boiled: plump pale cream ovals
    "seed_lotus_boiled": (4.0, 3.0, 9, [(238, 228, 200, 255), (226, 212, 178, 255)], None, None),
    # tiny pale round + red berry hint
    "seed_maianthemum_stellatum": (1.7, 1.7, 24, [(216, 200, 168, 255), (200, 182, 148, 255)], None, "berry"),
    # small tan-brown ovals + yellow fruit hint
    "seed_mayapple": (2.6, 1.9, 16, [(172, 138, 92, 255), (156, 122, 78, 255)], None, "fruit"),
    # flat cream cantaloupe ovals
    "seed_melon": (3.2, 2.0, 14, [(230, 216, 178, 255), (214, 198, 156, 255)], None, None),
    # tiny round yellow (data color)
    "seed_mustard": (1.6, 1.6, 36, [(226, 188, 74, 255), (208, 170, 60, 255)], None, None),
    # fresh green peas
    "seed_pea": (2.9, 2.9, 13, [(104, 178, 74, 255), (88, 158, 62, 255)], None, "pod"),
    # dry corn kernels
    "seed_popcorn": (3.0, 2.2, 15, [(232, 176, 66, 255), (216, 152, 48, 255)], None, None),
    # small round reddish-tan
    "seed_radish": (2.0, 1.9, 24, [(168, 118, 84, 255), (152, 104, 70, 255)], None, None),
    # elongated grain-like tan-green
    "seed_raw_bamboo": (3.6, 1.5, 14, [(178, 172, 116, 255), (158, 154, 98, 255)], "groove", None),
    # amber-red grooved wheatberries
    "seed_red_winter_wheat": (3.4, 1.8, 16, [(176, 118, 62, 255), (158, 102, 50, 255)], "groove", None),
    # rough spiky tan balls
    "seed_spinach": (2.2, 2.2, 20, [(168, 156, 128, 255), (150, 138, 110, 255)], "spiky", None),
    # flat dark teardrops, pale rim
    "seed_watermelon": (3.4, 2.2, 13, [(62, 50, 44, 255), (78, 64, 54, 255)], "rim", None),
    # angular black onion-like seeds
    "seed_wild_garlic": (2.0, 1.7, 24, [(36, 34, 38, 255), (52, 50, 54, 255)], None, None),
    # long thin dark grains
    "seed_wild_rice": (4.4, 1.3, 14, [(84, 62, 44, 255), (66, 48, 34, 255)], "groove", None),
    # tiny dark seeds + berry hint
    "seed_wild_sarsaparilla": (1.7, 1.7, 26, [(58, 48, 46, 255), (76, 62, 58, 255)], None, "berry"),
    # tiny pale seeds + red berry hint
    "seed_wintergreen": (1.5, 1.5, 32, [(206, 192, 160, 255), (190, 174, 140, 255)], None, "berry"),
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "seed_garlic_clove":
        garlic_cloves(img, rng)
    elif sid == "seed_japanese_knotweed":
        knotweed_stems(img, rng)
    elif sid == "seed_poppy_bud":
        poppy_bud(img, rng)
    elif sid == "seed_raw_burdock":
        burdock_bur(img, rng)
    else:
        rx, ry, count, colors, mark, hint = P[sid]
        if hint == "leaf":
            leaf_hint(img)
        elif hint == "berry":
            d.ellipse([23, 3, 30, 10], fill=(178, 44, 44, 255))
            d.ellipse([24, 4, 26, 6], fill=(226, 120, 110, 255))
            d.line([26, 3, 28, 1], fill=(70, 110, 50, 255), width=1)
        elif hint == "fruit":
            d.ellipse([22, 2, 30, 10], fill=(226, 196, 84, 255))
            d.ellipse([23, 3, 26, 6], fill=(244, 224, 140, 255))
        elif hint == "pod":
            pts = [(4, 4), (13, 2), (15, 6), (7, 9), (4, 7)]
            d.polygon(pts, fill=(86, 158, 60, 255))
            d.line([5, 6, 14, 5], fill=(64, 128, 44, 255), width=1)
        if mark == "spiky":
            # rough prickly balls instead of smooth seeds
            spots = []
            for _ in range(count):
                a = rng.uniform(0, 2 * math.pi)
                r = math.sqrt(rng.uniform(0, 1))
                spots.append((18 + math.sin(a) * r * 6.5,
                              16 + math.cos(a) * r * 11))
            spots.sort()
            for y, x in spots:
                c = rng.choice(colors)
                for a in range(0, 360, 45):
                    x2 = x + 2.6 * math.cos(math.radians(a))
                    y2 = y + 2.6 * math.sin(math.radians(a))
                    d.line([x, y, x2, y2], fill=shade(c, 0.7), width=1)
                d.ellipse([x - 1.8, y - 1.8, x + 1.8, y + 1.8], fill=c)
        else:
            pile(img, rng, rx, ry, count, colors, mark)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    os.makedirs(BASE, exist_ok=True)
    imgs = {}
    for sid in IDS:
        img = draw_one(sid)
        imgs[sid] = img
        folder = os.path.join(BASE, sid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, sid + ".png"))
        with open(os.path.join(folder, sid + ".json"), "w") as f:
            json.dump([{"id": sid, "fg": sid, "bg": ""}], f)
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS + "\n")
    # contact-sheet preview: 6 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item43.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item43.png {prev.size}")


if __name__ == "__main__":
    main()
