#!/usr/bin/env python3
"""
Draw the missing lepidoptera sprites from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/insect_spider.json, CDDA 6a3e59e0):
  mon_butterfly         "An oversized butterfly with vibrant colored markings.
                         ... a very peaceful nectivore."  symbol l, color white, 15 L
  mon_butterfly_giant   "A mutated butterfly larger than a human. ... feeds
                         mainly on the nectar produced by triffid flowers."
                         symbol L, color white, 75 L
  mon_butterfly_titan   "An enormous butterfly with regal looking wings. ...
                         leaving dust as it passes."  copy-from giant,
                         symbol L, color yellow, 120 L
  mon_caterpillar_butterfly "The larval stage of a butterfly or a moth.  This
                         one appears to be about your size..."  symbol C,
                         color light_green, 60 L, upgrades into the cocoon
  mon_butterfly_cocoon  "A caterpillar undergoing a metamorphosis. ...
                         the emerging adult will be massive as well."
                         symbol 0, color light_gray, 120 L, IMMOBILE,
                         upgrades into mon_butterfly_giant

Design: monarch-style vibrant orange/black for the butterfly line (the
canonical "vibrant colored markings" butterfly, and its chrysalis is the
jade-and-gold one drawn for the cocoon, paled toward the game's light_gray);
the titan swaps to gold wings with royal-purple borders ("regal", game
color yellow) plus the dust it sheds. Top-down spread-wing pose, dark
outline, flat fills + one highlight band -- UltiCa monster style.
Output is UltiCa decomposed layout (png + tile-entry json per folder).
"""
from PIL import Image, ImageDraw
import json, os

BASE = os.path.dirname(os.path.abspath(__file__))

def mirror(pts):
    return [(1.0 - x, y) for x, y in reversed(pts)]

def draw_butterfly(size, base, hi, border, vein, spot, body, body_hi, dust=False):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda pts: [(x * size, y * size) for x, y in pts]
    fore = [(0.515, 0.40), (0.66, 0.16), (0.84, 0.09), (0.96, 0.17),
            (0.965, 0.32), (0.88, 0.44), (0.66, 0.485), (0.53, 0.46)]
    hind = [(0.525, 0.52), (0.70, 0.50), (0.87, 0.60), (0.885, 0.76),
            (0.76, 0.875), (0.60, 0.82), (0.525, 0.64)]
    for wing in (fore, hind):
        for side in (wing, mirror(wing)):
            d.polygon(S(side), fill=border)                       # dark border
            cx = sum(x for x, _ in side) / len(side); cy = sum(y for _, y in side) / len(side)
            inner = [(cx + (x - cx) * 0.86, cy + (y - cy) * 0.84) for x, y in side]
            d.polygon(S(inner), fill=base)                        # main fill
            inner2 = [(cx + (x - cx) * 0.60, cy + (y - cy) * 0.58) for x, y in side]
            d.polygon(S(inner2), fill=hi)                         # highlight core
            # veins from the wing root
            root = (0.53, 0.44) if wing is fore else (0.535, 0.55)
            if side is not wing:
                root = (1.0 - root[0], root[1])
            for vx, vy in side[1:5]:
                if side is not wing:
                    vx = 1.0 - vx
                d.line([root[0] * size, root[1] * size, vx * size, vy * size],
                       fill=vein, width=max(1, size // 32))
            # white spots along the dark border
            for i in range(1, 6):
                x, y = side[i]
                if side is not wing:
                    x = 1.0 - x
                r = max(1.0, size * 0.016)
                d.ellipse([x * size - r, y * size - r, x * size + r, y * size + r], fill=spot)
    # antennae (clubbed)
    for sx in (1, -1):
        pts = [(0.5 + sx * 0.012, 0.30), (0.5 + sx * 0.07, 0.20), (0.5 + sx * 0.13, 0.115)]
        d.line([(x * size, y * size) for x, y in pts], fill=body, width=max(1, size // 32))
        ex, ey = pts[-1]
        r = max(1.2, size * 0.02)
        d.ellipse([ex * size - r, ey * size - r, ex * size + r, ey * size + r], fill=body)
    # body: thorax + segmented abdomen
    w = size * 0.052
    d.ellipse([size * 0.5 - w, size * 0.28, size * 0.5 + w, size * 0.50], fill=body)
    d.ellipse([size * 0.5 - w * 0.8, size * 0.46, size * 0.5 + w * 0.8, size * 0.80], fill=body)
    for i in range(3):
        y = size * (0.55 + i * 0.075)
        d.line([size * 0.5 - w * 0.78, y, size * 0.5 + w * 0.78, y],
               fill=body_hi, width=max(1, size // 32))
    r = size * 0.045  # head
    d.ellipse([size * 0.5 - r, size * 0.235, size * 0.5 + r, size * 0.235 + 2 * r], fill=body)
    if dust:  # titan sheds dust as it passes
        for (x, y, s) in [(0.10, 0.90, 2), (0.24, 0.95, 1), (0.86, 0.92, 2),
                          (0.70, 0.96, 1), (0.50, 0.94, 1), (0.06, 0.72, 1),
                          (0.93, 0.68, 1)]:
            cx, cy = x * size, y * size
            d.line([cx - s, cy, cx + s, cy], fill=(255, 236, 170, 255), width=1)
            d.line([cx, cy - s, cx, cy + s], fill=(255, 236, 170, 255), width=1)
    return img

def draw_caterpillar(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark, mid, light, stripe = (38, 92, 38, 255), (96, 178, 66, 255), (140, 214, 96, 255), (214, 232, 130, 255)
    # S-curve spine, tail bottom-left -> head top-right
    import math
    spine = []
    for i in range(9):
        t = i / 8.0
        x = 0.16 + 0.68 * t
        y = 0.80 - 0.58 * t + 0.10 * math.sin(t * 4.2)
        r = 0.085 + 0.045 * math.sin(t * math.pi) + (0.02 if i == 8 else 0)
        spine.append((x, y, r))
    for x, y, r in spine:  # prolegs
        for sx in (-0.5, 0.5):
            d.ellipse([(x + sx * r - 0.016) * size, (y + r * 0.72) * size,
                       (x + sx * r + 0.016) * size, (y + r * 0.72 + 0.045) * size], fill=dark)
    for x, y, r in reversed(spine):
        d.ellipse([(x - r) * size, (y - r) * size, (x + r) * size, (y + r) * size], fill=dark)
        ri = r * 0.82
        d.ellipse([(x - ri) * size, (y - ri) * size, (x + ri) * size, (y + ri) * size], fill=mid)
        rh = r * 0.45
        d.ellipse([(x - rh) * size, (y - rh * 1.15) * size, (x + rh) * size, (y + rh * 0.6) * size], fill=light)
        d.line([(x - r * 0.4) * size, (y - r * 0.92) * size, (x + r * 0.4) * size, (y - r * 0.92) * size],
               fill=stripe, width=max(1, size // 24))
    hx, hy, hr = spine[-1]  # head eyespots + nubs
    for sx in (-0.38, 0.38):
        d.ellipse([(hx + sx * hr - 0.02) * size, (hy - 0.03) * size,
                   (hx + sx * hr + 0.02) * size, (hy + 0.01) * size], fill=dark)
    return img

def draw_cocoon(size):
    import math
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    outline, body, hi, ridge, gold = (70, 86, 60, 255), (146, 178, 120, 255), (186, 214, 152, 255), (106, 138, 84, 255), (232, 190, 60, 255)
    cx, top, bot, w = 0.5, 0.13, 0.93, 0.225
    def halfw(t):  # dome top, widest at t~0.38, long taper to a soft point
        return w * (0.30 + 0.70 * math.sin(math.pi * min(0.12 + 0.88 * t, 1.0))) * (1.0 - 0.45 * max(0.0, t - 0.68) / 0.32)
    pts = []
    for i in range(17):
        t = i / 16.0
        pts.append((cx + halfw(t), top + (bot - top) * t))
    pts += [(1.0 - x, y) for x, y in reversed(pts)]
    d.polygon([(x * size, y * size) for x, y in pts], fill=outline)
    inner = [(cx + (x - cx) * 0.87, 0.5 + (y - 0.5) * 0.95) for x, y in pts]
    d.polygon([(x * size, y * size) for x, y in inner], fill=body)
    d.ellipse([(cx - w * 0.40) * size, (top + 0.06) * size, (cx + w * 0.02) * size, (bot - 0.22) * size], fill=hi)
    for i in range(5):  # soft segment ridges, following the taper
        t = 0.22 + i * 0.13
        y = top + (bot - top) * t
        ww = halfw(t) * 0.86
        d.arc([(cx - ww) * size, (y - 0.045) * size, (cx + ww) * size, (y + 0.065) * size],
              start=20, end=160, fill=ridge, width=max(1, size // 32))
    # monarch-style gold dots near the top (no hard band -- reads as a rim)
    y = (top + 0.085) * size
    for fx in (-0.55, -0.28, 0.0, 0.28, 0.55):
        x = (cx + fx * halfw(0.10) * 1.55) * size
        r = max(1.2, size * 0.02)
        d.ellipse([x - r, y - r, x + r, y + r], fill=gold)
    d.line([cx * size, size * 0.03, cx * size, top * size], fill=outline, width=max(1, size // 24))  # silk stem
    return img

MONARCH = dict(base=(224, 123, 31, 255), hi=(246, 176, 66, 255), border=(30, 19, 12, 255),
               vein=(46, 28, 16, 255), spot=(250, 246, 232, 255), body=(30, 20, 13, 255),
               body_hi=(92, 62, 36, 255))
REGAL = dict(base=(232, 168, 18, 255), hi=(255, 217, 77, 255), border=(52, 26, 84, 255),
             vein=(74, 40, 110, 255), spot=(255, 240, 190, 255), body=(38, 24, 52, 255),
             body_hi=(110, 78, 150, 255))

JOBS = [
    ("pngs_normal_32x32/monsters/mon_butterfly/mon_butterfly", 32,
     draw_butterfly(32, **MONARCH), "shadow_32x32_normal"),
    ("pngs_large_64x64/monsters/mon_butterfly_giant/mon_butterfly_giant", 64,
     draw_butterfly(64, **MONARCH), "shadow_64x64_normal"),
    ("pngs_giant_96x96/monsters/mon_butterfly_titan/mon_butterfly_titan", 96,
     draw_butterfly(96, dust=True, **REGAL), None),
    ("pngs_large_64x64/monsters/mon_caterpillar_butterfly/mon_caterpillar_butterfly", 64,
     draw_caterpillar(64), "shadow_64x64_normal"),
    ("pngs_large_64x64/monsters/mon_butterfly_cocoon/mon_butterfly_cocoon", 64,
     draw_cocoon(64), "shadow_64x64_normal"),
]
for rel, size, img, bg in JOBS:
    folder = os.path.join(BASE, os.path.dirname(rel))
    os.makedirs(folder, exist_ok=True)
    name = os.path.basename(rel)
    img.save(os.path.join(BASE, rel + ".png"))
    entry = {"id": name, "fg": name}
    if bg:
        entry["bg"] = bg
    with open(os.path.join(BASE, rel + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write("Drawn from the monster's in-game description for the CDDA Voxel\n"
                "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")
    print("wrote", rel + ".png", img.size)
