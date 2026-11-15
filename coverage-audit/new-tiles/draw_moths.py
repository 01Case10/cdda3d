#!/usr/bin/env python3
"""
Draw the missing moth-family sprites from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/insect_spider.json, CDDA 6a3e59e0):
  mon_moth              "An oversized moth with its typical wood-like motifs.
                         It is as peaceful as the mutated butterflies found
                         during the day."  symbol l, color light_gray, 15 L,
                         vision_night (the nocturnal counterpart of the
                         butterfly), upgrades into mon_moth_giant
  mon_moth_giant        "A mutated moth larger than a human.  It silently
                         flies with its huge wings after dusk looking for
                         nocturnal flowers to feed upon."  symbol L,
                         color light_gray, 70 L
  mon_caterpillar_moth  "The larval stage of a butterfly or a moth.  This
                         one appears to be about your size..."  symbol C,
                         color green (darker than the butterfly caterpillar's
                         light_green), 60 L, upgrades into mon_moth_cocoon
  mon_moth_cocoon       "A caterpillar undergoing a metamorphosis. ...
                         the emerging adult will be massive as well."
                         symbol 0, color light_gray, 120 L, IMMOBILE,
                         upgrades into mon_moth_giant

Design: where the butterfly line is a hard jade chrysalis + monarch wings,
the moth line is a *silk* cocoon (fibrous, pale grey, thread-wrapped) and
bark-camouflage adults -- grey-brown wings with wavy wood-grain bands
("wood-like motifs"), a thick fuzzy body, big night eyes, and feathery
antennae (the classic moth trait, vs the butterfly's clubs). The moth
caterpillar is the butterfly caterpillar's darker-green sibling: deeper
green, rusty side spots, cream lateral stripe instead of a yellow dorsal
one. Top-down spread-wing pose, dark outline -- UltiCa monster style.
Output is UltiCa decomposed layout (png + tile-entry json per folder).
"""
from PIL import Image, ImageDraw
import json, math, os

BASE = os.path.dirname(os.path.abspath(__file__))

def mirror(pts):
    return [(1.0 - x, y) for x, y in reversed(pts)]

def inside(px, py, poly):
    """Ray-casting point-in-polygon (poly in unit coords, point in px)."""
    n = len(poly); hit = False; j = n - 1
    for i in range(n):
        xi, yi = poly[i]; xj, yj = poly[j]
        if ((yi > py) != (yj > py)) and (px < (xj - xi) * (py - yi) / (yj - yi) + xi):
            hit = not hit
        j = i
    return hit

def draw_moth(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda pts: [(x * size, y * size) for x, y in pts]
    border = (43, 33, 24, 255); base = (166, 148, 120, 255)
    hi = (198, 182, 150, 255); band = (128, 110, 84, 255)
    grain = (96, 78, 56, 255); pale = (222, 208, 178, 255)
    body = (110, 94, 70, 255); body_hi = (168, 150, 116, 255)
    fore = [(0.515, 0.40), (0.64, 0.15), (0.82, 0.08), (0.95, 0.16),
            (0.96, 0.31), (0.87, 0.44), (0.65, 0.49), (0.53, 0.46)]
    hind = [(0.525, 0.52), (0.70, 0.50), (0.87, 0.59), (0.89, 0.74),
            (0.77, 0.86), (0.60, 0.82), (0.525, 0.64)]
    wings = []
    for wing in (fore, hind):
        for side in (wing, mirror(wing)):
            wings.append(side)
            d.polygon(S(side), fill=border)
            cx = sum(x for x, _ in side) / len(side); cy = sum(y for _, y in side) / len(side)
            inner = [(cx + (x - cx) * 0.87, cy + (y - cy) * 0.85) for x, y in side]
            d.polygon(S(inner), fill=base)
            inner2 = [(cx + (x - cx) * 0.58, cy + (y - cy) * 0.55) for x, y in side]
            d.polygon(S(inner2), fill=hi)
    # wood-grain: wavy transverse bands + speckles, clipped to the wings.
    # The mid-wing band is wide and dark -- the classic moth transverse line.
    px = img.load()
    for band_i, (ybase, thick) in enumerate(((0.20, 1), (0.335, 3), (0.55, 1), (0.70, 2))):
        col = grain
        tpx = max(thick, size // 24) if thick > 1 else max(1, size // 32)
        for xi in range(size):
            x = xi / size
            y = ybase + 0.022 * math.sin(x * 17.0 + band_i * 2.1)
            if any(inside(x, y, w) for w in wings):
                for dy in range(tpx):
                    yy = int(y * size) + dy
                    if 0 <= yy < size:
                        px[xi, yy] = col
    for i in range(size * 4):  # deterministic speckle
        x = ((i * 37) % 97) / 97.0; y = ((i * 53) % 89) / 89.0
        if any(inside(x, y, w) for w in wings):
            xi, yi = int(x * size), int(y * size)
            if 0 <= xi < size and 0 <= yi < size:
                px[xi, yi] = grain if i % 3 else pale
    # pale reniform spot near each forewing tip (night marking)
    for sx in (1, -1):
        cxp, cyp = (0.5 + sx * 0.315) * size, 0.205 * size
        r = max(1.6, size * 0.030)
        d.ellipse([cxp - r, cyp - r * 0.7, cxp + r, cyp + r * 0.7], fill=border)
        r2 = r * 0.62
        d.ellipse([cxp - r2, cyp - r2 * 0.7, cxp + r2, cyp + r2 * 0.7], fill=pale)
    # feathery antennae: stem + barbs
    for sx in (1, -1):
        tip = (0.5 + sx * 0.145, 0.10)
        root = (0.5 + sx * 0.015, 0.29)
        d.line([root[0] * size, root[1] * size, tip[0] * size, tip[1] * size],
               fill=border, width=max(1, size // 32))
        for t in (0.25, 0.45, 0.65, 0.85):
            bx = root[0] + (tip[0] - root[0]) * t; by = root[1] + (tip[1] - root[1]) * t
            bl = 0.028 * (1.0 - t * 0.4)
            d.line([bx * size, by * size, (bx + sx * bl) * size, (by - bl * 0.7) * size],
                   fill=border, width=1)
            d.line([bx * size, by * size, (bx - sx * bl * 0.5) * size, (by + bl * 0.8) * size],
                   fill=border, width=1)
    # thick fuzzy body
    w = size * 0.068
    d.ellipse([size * 0.5 - w, size * 0.27, size * 0.5 + w, size * 0.52], fill=body)
    d.ellipse([size * 0.5 - w * 0.82, size * 0.47, size * 0.5 + w * 0.82, size * 0.82], fill=body)
    for i in range(4):
        y = size * (0.56 + i * 0.065)
        d.line([size * 0.5 - w * 0.8, y, size * 0.5 + w * 0.8, y], fill=body_hi, width=max(1, size // 32))
    for i in range(size // 2):  # fuzz speckle on the thorax
        x = 0.5 + (((i * 29) % 13) - 6) / 13.0 * 0.10
        y = 0.30 + ((i * 41) % 17) / 17.0 * 0.20
        xi, yi = int(x * size), int(y * size)
        if 0 <= xi < size and 0 <= yi < size:
            px[xi, yi] = body_hi
    r = size * 0.052  # head with two big night eyes
    d.ellipse([size * 0.5 - r, size * 0.225, size * 0.5 + r, size * 0.225 + 2 * r], fill=body)
    for sx in (1, -1):
        er = max(1.2, size * 0.020)
        ex, ey = size * (0.5 + sx * 0.026), size * 0.265
        d.ellipse([ex - er, ey - er, ex + er, ey + er], fill=(24, 18, 12, 255))
    return img

def draw_caterpillar_moth(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark, mid, light = (26, 66, 26, 255), (66, 138, 48, 255), (104, 178, 76, 255)
    stripe, spot = (226, 224, 180, 255), (178, 96, 40, 255)
    spine = []
    for i in range(9):
        t = i / 8.0
        x = 0.16 + 0.68 * t
        y = 0.80 - 0.58 * t + 0.10 * math.sin(t * 4.2)
        r = 0.085 + 0.045 * math.sin(t * math.pi) + (0.02 if i == 8 else 0)
        spine.append((x, y, r))
    for x, y, r in spine:
        for sx in (-0.5, 0.5):
            d.ellipse([(x + sx * r - 0.016) * size, (y + r * 0.72) * size,
                       (x + sx * r + 0.016) * size, (y + r * 0.72 + 0.045) * size], fill=dark)
    for x, y, r in reversed(spine):
        d.ellipse([(x - r) * size, (y - r) * size, (x + r) * size, (y + r) * size], fill=dark)
        ri = r * 0.82
        d.ellipse([(x - ri) * size, (y - ri) * size, (x + ri) * size, (y + ri) * size], fill=mid)
        rh = r * 0.45
        d.ellipse([(x - rh) * size, (y - rh * 1.15) * size, (x + rh) * size, (y + rh * 0.6) * size], fill=light)
        # cream lateral stripe + rusty side spot per segment
        d.line([(x - r * 0.55) * size, (y + r * 0.42) * size, (x + r * 0.55) * size, (y + r * 0.42) * size],
               fill=stripe, width=max(1, size // 24))
        sr = max(1.2, size * 0.02)
        sxp, syp = (x + r * 0.52) * size, (y - r * 0.18) * size
        d.ellipse([sxp - sr, syp - sr, sxp + sr, syp + sr], fill=spot)
    hx, hy, hr = spine[-1]
    for sx in (-0.38, 0.38):
        d.ellipse([(hx + sx * hr - 0.02) * size, (hy - 0.03) * size,
                   (hx + sx * hr + 0.02) * size, (hy + 0.01) * size], fill=dark)
    return img

def draw_cocoon_moth(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    outline, body, hi, thread, shadow = (104, 98, 84, 255), (188, 178, 156, 255), (216, 208, 188, 255), (232, 226, 210, 255), (150, 140, 118, 255)
    cx, cy, w, h = 0.5, 0.52, 0.58, 0.42  # plump horizontal oval, slight taper
    pts = []
    for i in range(25):
        t = i / 24.0 * 2 * math.pi
        x = cx + (w / 2) * math.cos(t) * (1.0 - 0.10 * math.cos(t))
        y = cy + (h / 2) * math.sin(t)
        pts.append((x, y))
    d.polygon([(x * size, y * size) for x, y in pts], fill=outline)
    inner = [(cx + (x - cx) * 0.90, cy + (y - cy) * 0.88) for x, y in pts]
    d.polygon([(x * size, y * size) for x, y in inner], fill=body)
    d.ellipse([(cx - w * 0.34) * size, (cy - h * 0.34) * size, (cx + w * 0.10) * size, (cy + h * 0.05) * size], fill=hi)
    # silk thread wraps: near-vertical wavy columns, clipped to the body
    # (drawing arcs with a tall bounding box overshoots the silhouette --
    # plot pixel-by-pixel inside the polygon instead)
    px = img.load()
    for i, fx in enumerate((-0.20, -0.12, -0.04, 0.04, 0.12, 0.20)):
        col = thread if i % 2 == 0 else shadow
        for yi in range(size):
            y = yi / size
            x = cx + fx + 0.012 * math.sin(y * 22.0 + i * 1.7)
            if inside(x, y, pts):
                for dx in range(max(1, size // 32)):
                    xi = int(x * size) + dx
                    if 0 <= xi < size:
                        px[xi, yi] = col
    return img

JOBS = [
    ("pngs_normal_32x32/monsters/mon_moth/mon_moth", 32,
     draw_moth(32), "shadow_32x32_normal"),
    ("pngs_large_64x64/monsters/mon_moth_giant/mon_moth_giant", 64,
     draw_moth(64), "shadow_64x64_normal"),
    ("pngs_large_64x64/monsters/mon_caterpillar_moth/mon_caterpillar_moth", 64,
     draw_caterpillar_moth(64), "shadow_64x64_normal"),
    ("pngs_large_64x64/monsters/mon_moth_cocoon/mon_moth_cocoon", 64,
     draw_cocoon_moth(64), "shadow_64x64_normal"),
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
