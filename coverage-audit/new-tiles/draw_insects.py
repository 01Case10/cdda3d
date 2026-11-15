#!/usr/bin/env python3
"""
Draw the missing insect/spider sprites from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/insect_spider.json, CDDA 6a3e59e0).
26 ids from that file are in the audit's missing_shipped/monster list with no
folder yet (the file's other missing ids were already drawn by the butterfly
and moth batches). Resolved volumes (through copy-from) drive sheet placement
per the standing rule: <100 L -> 32x32, 100-400 L -> 64x64, >400 L -> 96x96.

  96x96: mon_ant_acid_soldier_mega (625 L, cow-sized woolly brown ant, acid
         seeping from huge mandibles), mon_fly_mega (625 L, cow-sized
         housefly), mon_halfworm_graboid (600 L, pink severed graboid chunk)
  64x64: mon_cicada (108.5 L, person-sized, mostly harmless), mon_mantis_giant
         (120 L, bear-sized, dance-like stance), mon_moth_titan (120 L, lunar
         moth, pale colors, glittering dust), mon_spider_cellar_mom (108.5 L,
         mommy long-legs carrying a ball of spiderlings), mon_dermatik_
         incubator_deer (108.5 L, paralyzed emaciated deer host),
         mon_halfworm_small (108.5 L, severed oversized-worm chunk)
  32x32: everything else -- cicada nymph (digging claws), dermatik midwife
         (red, wings/ovipositor chewed off), firefly/fly/moth-fly/mosquito/
         worm larvae and grubs, oversized fly, giant + nymph grasshoppers,
         mantis nymph, giant moth fly, detached branching cellar-spider leg,
         three spiderlings (jumping white / trapdoor blue / wolf brown),
         water scorpion nymph.

Design: top-down poses like the butterfly/moth batches (the deer is a lying
side profile, as a paralyzed host reads best that way). Flat fills, dark
outlines, one highlight pass -- UltiCa monster style. Translucent wings are
painted on a separate overlay and alpha-composited; bodies stay opaque.
Output is UltiCa decomposed layout (png + tile-entry json per folder).
"""
from PIL import Image, ImageDraw
import json, math, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))

def rng_for(name):
    return random.Random(zlib.crc32(name.encode()))

def mirror(pts):
    return [(1.0 - x, y) for x, y in reversed(pts)]

def S(pts, size):
    return [(x * size, y * size) for x, y in pts]

def W(size, frac=32):
    return max(1, size // frac)

# ---------------------------------------------------------------- ants / wasps
def draw_ant(size, body, hi, dark, woolly=False, acid=False, rng=None):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = 0.5
    # legs: 3 pairs from the thorax, jointed
    for i, (ly, spread, bend) in enumerate(((0.40, 0.30, -0.10), (0.50, 0.36, 0.02), (0.60, 0.32, 0.14))):
        for sx in (1, -1):
            hip = (cx + sx * 0.05, ly)
            knee = (cx + sx * spread * 0.62, ly + bend)
            foot = (cx + sx * spread, ly + bend + (0.16 if i == 2 else 0.10 if i == 1 else 0.02))
            d.line(S([hip, knee, foot], size), fill=dark, width=W(size, 22))
    # gaster (big rear oval), thorax, head
    d.ellipse([(cx - 0.155) * size, 0.56 * size, (cx + 0.155) * size, 0.94 * size], fill=dark)
    d.ellipse([(cx - 0.135) * size, 0.575 * size, (cx + 0.135) * size, 0.925 * size], fill=body)
    d.ellipse([(cx - 0.10) * size, 0.60 * size, (cx + 0.01) * size, 0.80 * size], fill=hi)
    for t in (0.68, 0.78, 0.87):  # gaster banding
        y = t * size
        d.line([(cx - 0.128) * size, y, (cx + 0.128) * size, y], fill=dark, width=W(size, 32))
    d.ellipse([(cx - 0.075) * size, 0.40 * size, (cx + 0.075) * size, 0.60 * size], fill=dark)
    d.ellipse([(cx - 0.062) * size, 0.415 * size, (cx + 0.062) * size, 0.585 * size], fill=body)
    # head with huge mandibles
    d.ellipse([(cx - 0.105) * size, 0.13 * size, (cx + 0.105) * size, 0.40 * size], fill=dark)
    d.ellipse([(cx - 0.09) * size, 0.145 * size, (cx + 0.09) * size, 0.385 * size], fill=body)
    d.ellipse([(cx - 0.055) * size, 0.16 * size, (cx + 0.005) * size, 0.27 * size], fill=hi)
    for sx in (1, -1):  # eyes
        ex, ey = (cx + sx * 0.062) * size, 0.235 * size
        r = max(1.4, size * 0.020)
        d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(20, 12, 8, 255))
        # mandibles: thick curved pincers
        pts = [(cx + sx * 0.055, 0.155), (cx + sx * 0.115, 0.085), (cx + sx * 0.075, 0.035),
               (cx + sx * 0.028, 0.085)]
        d.polygon(S(pts, size), fill=dark)
        # antennae: elbowed
        pts = [(cx + sx * 0.03, 0.15), (cx + sx * 0.10, 0.06), (cx + sx * 0.17, 0.075)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
    if acid:  # corrosive droplets seeping from the mandibles
        for (x, y, r) in ((0.40, 0.045, 0.016), (0.585, 0.06, 0.013), (0.47, 0.02, 0.011)):
            d.ellipse([(x - r) * size, (y - r) * size, (x + r) * size, (y + r) * size],
                      fill=(120, 200, 60, 255))
            d.ellipse([(x - r * 0.4) * size, (y - r * 0.5) * size, (x + r * 0.2) * size, (y + r * 0.1) * size],
                      fill=(190, 240, 120, 255))
    if woolly and rng:  # pale fuzz speckle over the whole body
        px = img.load()
        for _ in range(size * size // 9):
            x = rng.uniform(0.30, 0.70); y = rng.uniform(0.12, 0.94)
            xi, yi = int(x * size), int(y * size)
            if 0 <= xi < size and 0 <= yi < size and px[xi, yi][3] > 0:
                px[xi, yi] = (196, 164, 120, 255)
    return img

def draw_dermatik(size, body, hi, dark):
    """Wingless red wasp-kin: broad head, narrow waist, big striped gaster,
    only chewed wing stubs (per the midwife's description)."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = 0.5
    for i, (ly, spread, bend) in enumerate(((0.42, 0.30, -0.08), (0.52, 0.35, 0.03), (0.62, 0.31, 0.13))):
        for sx in (1, -1):
            hip = (cx + sx * 0.045, ly)
            knee = (cx + sx * spread * 0.6, ly + bend)
            foot = (cx + sx * spread, ly + bend + 0.12)
            d.line(S([hip, knee, foot], size), fill=dark, width=W(size, 24))
    # gaster with banding + blunt ovipositor stub at the tip
    d.ellipse([(cx - 0.14) * size, 0.58 * size, (cx + 0.14) * size, 0.93 * size], fill=dark)
    d.ellipse([(cx - 0.12) * size, 0.595 * size, (cx + 0.12) * size, 0.915 * size], fill=body)
    d.ellipse([(cx - 0.085) * size, 0.62 * size, (cx + 0.005) * size, 0.80 * size], fill=hi)
    for t in (0.70, 0.80, 0.88):
        d.line([(cx - 0.112) * size, t * size, (cx + 0.112) * size, t * size], fill=dark, width=W(size, 32))
    d.polygon(S([(cx - 0.022, 0.925), (cx + 0.022, 0.925), (cx, 0.975)], size), fill=dark)
    # thorax + chewed wing stubs
    d.ellipse([(cx - 0.085) * size, 0.38 * size, (cx + 0.085) * size, 0.60 * size], fill=dark)
    d.ellipse([(cx - 0.07) * size, 0.395 * size, (cx + 0.07) * size, 0.585 * size], fill=body)
    for sx in (1, -1):
        stub = [(cx + sx * 0.06, 0.42), (cx + sx * 0.115, 0.46), (cx + sx * 0.085, 0.52),
                (cx + sx * 0.055, 0.50)]
        d.polygon(S(stub, size), fill=dark)
        stub_in = [(cx + sx * 0.065, 0.435), (cx + sx * 0.10, 0.465), (cx + sx * 0.078, 0.50),
                   (cx + sx * 0.06, 0.49)]
        d.polygon(S(stub_in, size), fill=hi)
    # head: big compound eyes, short antennae, small mandibles
    d.ellipse([(cx - 0.10) * size, 0.12 * size, (cx + 0.10) * size, 0.39 * size], fill=dark)
    d.ellipse([(cx - 0.085) * size, 0.135 * size, (cx + 0.085) * size, 0.375 * size], fill=body)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.052) * size, 0.21 * size
        r = max(1.6, size * 0.030)
        d.ellipse([ex - r, ey - r * 1.25, ex + r, ey + r * 1.25], fill=(52, 16, 10, 255))
        pts = [(cx + sx * 0.025, 0.14), (cx + sx * 0.075, 0.06), (cx + sx * 0.12, 0.075)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
        pts = [(cx + sx * 0.04, 0.135), (cx + sx * 0.075, 0.085), (cx + sx * 0.045, 0.05)]
        d.polygon(S(pts, size), fill=dark)
    return img

# ---------------------------------------------------------------- flies
def draw_fly(size, body, hi, dark, eye=(150, 30, 26, 255), hairy=False, rng=None):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = 0.5
    for i, (ly, spread, bend) in enumerate(((0.44, 0.33, -0.06), (0.55, 0.38, 0.04), (0.66, 0.34, 0.13))):
        for sx in (1, -1):
            hip = (cx + sx * 0.05, ly)
            knee = (cx + sx * spread * 0.6, ly + bend)
            foot = (cx + sx * spread, ly + bend + 0.11)
            d.line(S([hip, knee, foot], size), fill=dark, width=W(size, 24))
    # abdomen (rear), banded
    d.ellipse([(cx - 0.15) * size, 0.55 * size, (cx + 0.15) * size, 0.95 * size], fill=dark)
    d.ellipse([(cx - 0.132) * size, 0.565 * size, (cx + 0.132) * size, 0.935 * size], fill=body)
    d.ellipse([(cx - 0.095) * size, 0.60 * size, (cx) * size, 0.82 * size], fill=hi)
    for t in (0.68, 0.79, 0.88):
        d.line([(cx - 0.122) * size, t * size, (cx + 0.122) * size, t * size], fill=dark, width=W(size, 32))
    # thorax, broad and bristly
    d.ellipse([(cx - 0.125) * size, 0.34 * size, (cx + 0.125) * size, 0.62 * size], fill=dark)
    d.ellipse([(cx - 0.108) * size, 0.355 * size, (cx + 0.108) * size, 0.605 * size], fill=body)
    d.ellipse([(cx - 0.07) * size, 0.37 * size, (cx + 0.005) * size, 0.52 * size], fill=hi)
    # head: dominated by the two compound eyes
    d.ellipse([(cx - 0.115) * size, 0.10 * size, (cx + 0.115) * size, 0.37 * size], fill=dark)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.058) * size, 0.225 * size
        rx, ry = size * 0.052, size * 0.062
        d.ellipse([ex - rx, ey - ry, ex + rx, ey + ry], fill=eye)
        d.ellipse([ex - rx * 0.45, ey - ry * 0.55, ex + rx * 0.05, ey - ry * 0.05],
                  fill=(226, 120, 110, 255))
        pts = [(cx + sx * 0.02, 0.13), (cx + sx * 0.06, 0.055), (cx + sx * 0.10, 0.065)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
    d.ellipse([(cx - 0.018) * size, 0.315 * size, (cx + 0.018) * size, 0.375 * size], fill=dark)  # proboscis
    if hairy and rng:
        px = img.load()
        for _ in range(size * size // 12):
            x = rng.uniform(0.33, 0.67); y = rng.uniform(0.33, 0.95)
            xi, yi = int(x * size), int(y * size)
            if 0 <= xi < size and 0 <= yi < size and px[xi, yi][3] > 0:
                px[xi, yi] = dark
    # translucent wings on an overlay, alpha-composited (never drawn direct)
    ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for sx in (1, -1):
        wing = [(cx + sx * 0.03, 0.40), (cx + sx * 0.30, 0.30), (cx + sx * 0.40, 0.42),
                (cx + sx * 0.33, 0.60), (cx + sx * 0.10, 0.60), (cx + sx * 0.03, 0.52)]
        od.polygon(S(wing, size), fill=(225, 235, 240, 110), outline=(90, 100, 105, 160))
        vein = [(cx + sx * 0.05, 0.42), (cx + sx * 0.30, 0.38), (cx + sx * 0.36, 0.44)]
        od.line(S(vein, size), fill=(120, 130, 135, 150), width=1)
    img.alpha_composite(ov)
    return img

def draw_moth_fly(size):
    """Tiny fuzzy fly: broad hairy wings held flat, plump grey body."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    border = (70, 62, 52, 255); base = (178, 170, 158, 255); hi = (206, 198, 184, 255)
    body = (130, 120, 106, 255); dark = (52, 46, 38, 255)
    cx = 0.5
    for sx in (1, -1):  # small legs peeking out
        for ly, dx in ((0.45, 0.20), (0.58, 0.23), (0.70, 0.21)):
            d.line(S([(cx + sx * 0.06, ly), (cx + sx * dx, ly + 0.07)], size), fill=dark, width=W(size, 32))
    for sx in (1, -1):  # broad fuzzy wings
        wing = [(cx + sx * 0.02, 0.34), (cx + sx * 0.30, 0.24), (cx + sx * 0.42, 0.36),
                (cx + sx * 0.38, 0.58), (cx + sx * 0.16, 0.70), (cx + sx * 0.03, 0.58)]
        d.polygon(S(wing, size), fill=border)
        inner = [(cx + (x - cx) * 0.86, 0.47 + (y - 0.47) * 0.84) for x, y in wing]
        d.polygon(S(inner, size), fill=base)
        inner2 = [(cx + (x - cx) * 0.55, 0.47 + (y - 0.47) * 0.5) for x, y in wing]
        d.polygon(S(inner2, size), fill=hi)
    d.ellipse([(cx - 0.075) * size, 0.36 * size, (cx + 0.075) * size, 0.78 * size], fill=dark)
    d.ellipse([(cx - 0.06) * size, 0.375 * size, (cx + 0.06) * size, 0.765 * size], fill=body)
    for t in (0.52, 0.62, 0.71):
        d.line([(cx - 0.055) * size, t * size, (cx + 0.055) * size, t * size], fill=dark, width=1)
    r = size * 0.05
    d.ellipse([cx * size - r, 0.26 * size, cx * size + r, 0.26 * size + 2 * r], fill=dark)
    for sx in (1, -1):
        pts = [(cx + sx * 0.01, 0.28), (cx + sx * 0.06, 0.17), (cx + sx * 0.11, 0.12)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
        ex, ey = (cx + sx * 0.028) * size, 0.30 * size
        er = max(1.2, size * 0.018)
        d.ellipse([ex - er, ey - er, ex + er, ey + er], fill=(30, 26, 20, 255))
    return img

# ---------------------------------------------------------------- moths
def draw_big_moth(size, base, hi, band, grain, pale, body_c, body_hi, dark,
                  dust=False, tails=False, rng=None):
    """Lepidopteran with spread wings; lunar-moth tails + glitter dust for
    the titan, bark bands for the ordinary giants."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    fore = [(0.515, 0.40), (0.64, 0.15), (0.82, 0.08), (0.95, 0.16),
            (0.96, 0.31), (0.87, 0.44), (0.65, 0.49), (0.53, 0.46)]
    hind = [(0.525, 0.52), (0.70, 0.50), (0.87, 0.59), (0.885, 0.74),
            (0.76, 0.875), (0.60, 0.82), (0.525, 0.64)]
    wings = []
    for wing in (fore, hind):
        for side in (wing, mirror(wing)):
            wings.append(side)
            d.polygon(S(side, size), fill=dark)
            cxw = sum(x for x, _ in side) / len(side); cyw = sum(y for _, y in side) / len(side)
            inner = [(cxw + (x - cxw) * 0.87, cyw + (y - cyw) * 0.85) for x, y in side]
            d.polygon(S(inner, size), fill=base)
            inner2 = [(cxw + (x - cxw) * 0.58, cyw + (y - cyw) * 0.55) for x, y in side]
            d.polygon(S(inner2, size), fill=hi)
    if tails:  # luna-style hindwing tails, drawn under the body pass
        for sx in (1, -1):
            pts = [(0.5 + sx * 0.20, 0.80), (0.5 + sx * 0.26, 0.93), (0.5 + sx * 0.315, 0.985)]
            d.line(S(pts, size), fill=dark, width=W(size, 24))
            pts2 = [(0.5 + sx * 0.20, 0.80), (0.5 + sx * 0.245, 0.92), (0.5 + sx * 0.29, 0.975)]
            d.line(S(pts2, size), fill=base, width=W(size, 32))
    # transverse bands clipped to the wings, pixel-plotted
    px = img.load()
    def inside(px_, py_, poly):
        n = len(poly); hit = False; j = n - 1
        for i in range(n):
            xi, yi = poly[i]; xj, yj = poly[j]
            if ((yi > py_) != (yj > py_)) and (px_ < (xj - xi) * (py_ - yi) / (yj - yi) + xi):
                hit = not hit
            j = i
        return hit
    for band_i, (ybase, thick) in enumerate(((0.22, 1), (0.36, 2), (0.57, 1), (0.71, 2))):
        tpx = max(1, size // 32) * (2 if thick > 1 else 1)
        for xi in range(size):
            x = xi / size
            y = ybase + 0.02 * math.sin(x * 15.0 + band_i * 2.3)
            if any(inside(x, y, w) for w in wings):
                for dy in range(tpx):
                    yy = int(y * size) + dy
                    if 0 <= yy < size:
                        px[xi, yy] = band
    for sx in (1, -1):  # eyespot near each forewing tip
        cxp, cyp = (0.5 + sx * 0.315) * size, 0.205 * size
        r = max(1.6, size * 0.030)
        d.ellipse([cxp - r, cyp - r * 0.7, cxp + r, cyp + r * 0.7], fill=dark)
        r2 = r * 0.6
        d.ellipse([cxp - r2, cyp - r2 * 0.7, cxp + r2, cyp + r2 * 0.7], fill=pale)
    # feathery antennae
    for sx in (1, -1):
        tip = (0.5 + sx * 0.145, 0.10); root = (0.5 + sx * 0.015, 0.29)
        d.line(S([root, tip], size), fill=dark, width=W(size, 32))
        for t in (0.25, 0.45, 0.65, 0.85):
            bx = root[0] + (tip[0] - root[0]) * t; by = root[1] + (tip[1] - root[1]) * t
            bl = 0.028 * (1.0 - t * 0.4)
            d.line(S([(bx, by), (bx + sx * bl, by - bl * 0.7)], size), fill=dark, width=1)
            d.line(S([(bx, by), (bx - sx * bl * 0.5, by + bl * 0.8)], size), fill=dark, width=1)
    # thick fuzzy body
    w = size * 0.068
    d.ellipse([size * 0.5 - w, size * 0.27, size * 0.5 + w, size * 0.52], fill=body_c)
    d.ellipse([size * 0.5 - w * 0.82, size * 0.47, size * 0.5 + w * 0.82, size * 0.82], fill=body_c)
    for i in range(4):
        y = size * (0.56 + i * 0.065)
        d.line([size * 0.5 - w * 0.8, y, size * 0.5 + w * 0.8, y], fill=body_hi, width=W(size, 32))
    r = size * 0.052
    d.ellipse([size * 0.5 - r, size * 0.225, size * 0.5 + r, size * 0.225 + 2 * r], fill=body_c)
    for sx in (1, -1):
        er = max(1.2, size * 0.020)
        ex, ey = size * (0.5 + sx * 0.026), size * 0.265
        d.ellipse([ex - er, ey - er, ex + er, ey + er], fill=(24, 20, 16, 255))
    if dust and rng:  # glittering dust shed around the wings
        for _ in range(26):
            x = rng.uniform(0.04, 0.96); y = rng.uniform(0.55, 0.99)
            xi, yi = int(x * size), int(y * size)
            if 0 <= xi < size and 0 <= yi < size and px[xi, yi][3] == 0:
                px[xi, yi] = (190, 245, 235, 255)
    return img

# ---------------------------------------------------------------- cicada
def draw_cicada(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (74, 72, 66, 255); body = (168, 166, 158, 255); hi = (198, 196, 186, 255)
    cx = 0.5
    for i, (ly, spread) in enumerate(((0.42, 0.34), (0.54, 0.38), (0.66, 0.35))):
        for sx in (1, -1):
            hip = (cx + sx * 0.06, ly)
            knee = (cx + sx * spread * 0.6, ly + 0.03)
            foot = (cx + sx * spread, ly + 0.13)
            d.line(S([hip, knee, foot], size), fill=dark, width=W(size, 24))
    # abdomen under the wings: segmented taper
    d.polygon(S([(cx - 0.13, 0.42), (cx + 0.13, 0.42), (cx + 0.05, 0.94), (cx - 0.05, 0.94)], size), fill=dark)
    d.polygon(S([(cx - 0.105, 0.44), (cx + 0.105, 0.44), (cx + 0.038, 0.92), (cx - 0.038, 0.92)], size), fill=body)
    for t in (0.55, 0.66, 0.77, 0.86):
        half = 0.10 - (t - 0.44) * 0.13
        d.line([(cx - half) * size, t * size, (cx + half) * size, t * size], fill=dark, width=W(size, 32))
    # thorax + broad head with wide-set compound eyes and three ocelli
    d.ellipse([(cx - 0.15) * size, 0.28 * size, (cx + 0.15) * size, 0.50 * size], fill=dark)
    d.ellipse([(cx - 0.132) * size, 0.295 * size, (cx + 0.132) * size, 0.485 * size], fill=body)
    d.ellipse([(cx - 0.075) * size, 0.31 * size, (cx + 0.01) * size, 0.42 * size], fill=hi)
    d.ellipse([(cx - 0.185) * size, 0.10 * size, (cx + 0.185) * size, 0.32 * size], fill=dark)
    d.ellipse([(cx - 0.168) * size, 0.115 * size, (cx + 0.168) * size, 0.305 * size], fill=body)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.125) * size, 0.20 * size
        rx, ry = size * 0.045, size * 0.055
        d.ellipse([ex - rx, ey - ry, ex + rx, ey + ry], fill=(96, 52, 30, 255))
        d.ellipse([ex - rx * 0.4, ey - ry * 0.5, ex + rx * 0.05, ey], fill=(170, 110, 70, 255))
        pts = [(cx + sx * 0.03, 0.13), (cx + sx * 0.075, 0.05), (cx + sx * 0.115, 0.06)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
    for fx in (-0.035, 0.0, 0.035):  # ocelli triangle
        ex, ey = (cx + fx) * size, 0.175 * size
        r = max(1.1, size * 0.012)
        d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(210, 90, 50, 255))
    # long translucent wings folded rooflike over the back (overlay pass)
    ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for sx in (1, -1):
        wing = [(cx + sx * 0.015, 0.30), (cx + sx * 0.155, 0.36), (cx + sx * 0.145, 0.78),
                (cx + sx * 0.075, 0.985), (cx + sx * 0.02, 0.80)]
        od.polygon(S(wing, size), fill=(228, 232, 226, 105), outline=(100, 102, 96, 170))
        for t in (0.45, 0.62, 0.79):
            od.line(S([(cx + sx * 0.03, 0.34), (cx + sx * 0.125, t)], size),
                    fill=(130, 132, 126, 130), width=1)
    img.alpha_composite(ov)
    return img

def draw_cicada_nymph(size):
    """Pale grub-like nymph with oversized digging forelegs."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (120, 104, 82, 255); body = (232, 226, 214, 255); hi = (248, 244, 234, 255)
    cx = 0.5
    for i, (ly, spread) in enumerate(((0.52, 0.30), (0.64, 0.33), (0.76, 0.30))):
        for sx in (1, -1):
            hip = (cx + sx * 0.07, ly)
            foot = (cx + sx * spread, ly + 0.09)
            d.line(S([hip, (cx + sx * spread * 0.6, ly + 0.02), foot], size), fill=dark, width=W(size, 28))
    # plump segmented abdomen
    d.ellipse([(cx - 0.16) * size, 0.42 * size, (cx + 0.16) * size, 0.95 * size], fill=dark)
    d.ellipse([(cx - 0.14) * size, 0.435 * size, (cx + 0.14) * size, 0.935 * size], fill=body)
    d.ellipse([(cx - 0.09) * size, 0.47 * size, (cx + 0.005) * size, 0.72 * size], fill=hi)
    for t in (0.58, 0.68, 0.78, 0.87):
        d.line([(cx - 0.128) * size, t * size, (cx + 0.128) * size, t * size], fill=dark, width=W(size, 32))
    # thorax + head
    d.ellipse([(cx - 0.125) * size, 0.24 * size, (cx + 0.125) * size, 0.46 * size], fill=dark)
    d.ellipse([(cx - 0.108) * size, 0.255 * size, (cx + 0.108) * size, 0.445 * size], fill=body)
    d.ellipse([(cx - 0.10) * size, 0.09 * size, (cx + 0.10) * size, 0.27 * size], fill=dark)
    d.ellipse([(cx - 0.085) * size, 0.105 * size, (cx + 0.085) * size, 0.255 * size], fill=body)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.058) * size, 0.165 * size
        r = max(1.3, size * 0.020)
        d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(60, 44, 30, 255))
        pts = [(cx + sx * 0.02, 0.11), (cx + sx * 0.055, 0.045), (cx + sx * 0.09, 0.055)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
        # digging foreleg: thick femur + pointed claw, angled forward-out
        pts = [(cx + sx * 0.085, 0.30), (cx + sx * 0.20, 0.235), (cx + sx * 0.245, 0.13)]
        d.line(S(pts, size), fill=dark, width=W(size, 10))
        claw = [(cx + sx * 0.245, 0.13), (cx + sx * 0.285, 0.075), (cx + sx * 0.225, 0.10)]
        d.polygon(S(claw, size), fill=dark)
    return img

# ---------------------------------------------------------------- grubs & worms
def draw_grub(size, body, hi, dark, glow=False, curve=0.0):
    """Plump segmented larva, head at top; optional glowing rear segments."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    n = 7
    pts = []
    for i in range(n):
        t = i / (n - 1)
        x = 0.5 + curve * math.sin(t * math.pi) * 0.5
        y = 0.14 + 0.74 * t
        r = 0.105 + 0.055 * math.sin(min(t * 1.25, 1.0) * math.pi) - 0.03 * t
        pts.append((x, y, r))
    for x, y, r in reversed(pts):
        d.ellipse([(x - r) * size, (y - r * 0.82) * size, (x + r) * size, (y + r * 0.82) * size], fill=dark)
        ri = r * 0.84
        d.ellipse([(x - ri) * size, (y - ri * 0.82) * size, (x + ri) * size, (y + ri * 0.82) * size], fill=body)
        rh = r * 0.42
        d.ellipse([(x - rh) * size, (y - r * 0.5) * size, (x + rh * 0.4) * size, (y - r * 0.02) * size], fill=hi)
    hx, hy, hr = pts[0]  # darker head cap + tiny mandibles
    d.ellipse([(hx - hr * 0.55) * size, (hy - hr * 0.85) * size, (hx + hr * 0.55) * size, (hy - hr * 0.1) * size],
              fill=dark)
    for sx in (1, -1):
        d.polygon(S([(hx + sx * 0.03, hy - hr * 0.9), (hx + sx * 0.065, hy - hr * 1.25),
                     (hx + sx * 0.012, hy - hr * 1.15)], size), fill=dark)
    if glow:  # firefly larva: bright lantern segments near the rear
        for (x, y, r) in pts[-2:]:
            d.ellipse([(x - r * 0.5) * size, (y - r * 0.32) * size, (x + r * 0.5) * size, (y + r * 0.32) * size],
                      fill=(250, 226, 92, 255))
            d.ellipse([(x - r * 0.22) * size, (y - r * 0.15) * size, (x + r * 0.22) * size, (y + r * 0.15) * size],
                      fill=(255, 246, 170, 255))
    return img

def draw_worm(size, body, hi, dark):
    """Segmented S-curve worm (earthworm / severed half-worm chunk)."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    spine = []
    for i in range(10):
        t = i / 9.0
        x = 0.14 + 0.72 * t
        y = 0.68 - 0.40 * t + 0.13 * math.sin(t * 4.6)
        r = 0.075 + 0.038 * math.sin(t * math.pi)
        spine.append((x, y, r))
    for x, y, r in reversed(spine):
        d.ellipse([(x - r) * size, (y - r) * size, (x + r) * size, (y + r) * size], fill=dark)
        ri = r * 0.82
        d.ellipse([(x - ri) * size, (y - ri) * size, (x + ri) * size, (y + ri) * size], fill=body)
        rh = r * 0.45
        d.ellipse([(x - rh) * size, (y - rh * 1.15) * size, (x + rh) * size, (y + rh * 0.6) * size], fill=hi)
    # clitellum band on the front third
    x, y, r = spine[6]
    d.line([(x - r * 0.3) * size, (y - r * 1.02) * size, (x + r * 0.55) * size, (y - r * 1.02) * size],
           fill=hi, width=W(size, 16))
    return img

def draw_halfworm(size, body, hi, dark):
    """Thick severed worm chunk: a fat curved segment with two torn ends."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    spine = []
    for i in range(7):
        t = i / 6.0
        x = 0.16 + 0.68 * t
        y = 0.74 - 0.44 * t + 0.10 * math.sin(t * 3.6)
        r = 0.115 + 0.03 * math.sin(t * math.pi)
        spine.append((x, y, r))
    for x, y, r in reversed(spine):
        d.ellipse([(x - r) * size, (y - r) * size, (x + r) * size, (y + r) * size], fill=dark)
        ri = r * 0.85
        d.ellipse([(x - ri) * size, (y - ri) * size, (x + ri) * size, (y + ri) * size], fill=body)
        rh = r * 0.45
        d.ellipse([(x - rh) * size, (y - rh * 1.15) * size, (x + rh) * size, (y + rh * 0.6) * size], fill=hi)
        d.line([(x - r * 0.55) * size, (y - r * 0.95) * size, (x + r * 0.55) * size, (y - r * 0.95) * size],
               fill=dark, width=W(size, 32))
    for (x, y, r) in (spine[0], spine[-1]):  # torn end: pale flesh disc + dark core
        d.ellipse([(x - r * 0.52) * size, (y - r * 0.52) * size, (x + r * 0.52) * size, (y + r * 0.52) * size],
                  fill=(238, 178, 168, 255))
        d.ellipse([(x - r * 0.22) * size, (y - r * 0.22) * size, (x + r * 0.22) * size, (y + r * 0.22) * size],
                  fill=(168, 84, 76, 255))
    return img

def draw_mosquito_larva(size):
    """Comma-shaped wriggler hanging head-down, breathing siphon at the tail."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (94, 66, 40, 255); body = (146, 112, 74, 255); hi = (182, 148, 104, 255)
    spine = []
    for i in range(8):
        t = i / 7.0
        x = 0.40 + 0.26 * t + 0.10 * math.sin(t * 2.6)
        y = 0.16 + 0.66 * t
        r = 0.085 - 0.038 * t + (0.028 if i == 1 else 0)
        spine.append((x, y, r))
    for x, y, r in reversed(spine):
        d.ellipse([(x - r) * size, (y - r) * size, (x + r) * size, (y + r) * size], fill=dark)
        ri = r * 0.82
        d.ellipse([(x - ri) * size, (y - ri) * size, (x + ri) * size, (y + ri) * size], fill=body)
        rh = r * 0.45
        d.ellipse([(x - rh) * size, (y - rh * 1.1) * size, (x + rh) * size, (y + rh * 0.55) * size], fill=hi)
    hx, hy, hr = spine[0]  # head: dark cap, brush mouthparts
    d.ellipse([(hx - hr * 0.6) * size, (hy - hr * 1.05) * size, (hx + hr * 0.6) * size, (hy - hr * 0.25) * size], fill=dark)
    for dx in (-0.03, 0.0, 0.03):
        d.line([(hx + dx) * size, (hy - hr * 1.0) * size, (hx + dx * 1.8) * size, (hy - hr * 1.55) * size],
               fill=dark, width=1)
    tx, ty, tr = spine[-1]  # siphon tube at the rear tip
    d.line([(tx + tr * 0.3) * size, (ty + tr * 0.4) * size, (tx + tr * 1.5) * size, (ty + tr * 1.5) * size],
           fill=dark, width=W(size, 20))
    return img

# ---------------------------------------------------------------- mantis / hopper
def draw_mantis(size, body, hi, dark, winged=True):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = 0.5
    # walking legs: mid + hind pairs, long and thin
    for (ly, spread, bend) in ((0.55, 0.36, 0.06), (0.68, 0.38, 0.16)):
        for sx in (1, -1):
            hip = (cx + sx * 0.035, ly)
            knee = (cx + sx * spread * 0.62, ly + bend - 0.05)
            foot = (cx + sx * spread, ly + bend + 0.10)
            d.line(S([hip, knee, foot], size), fill=dark, width=W(size, 26))
    # long abdomen tapering back, with wing cases if adult
    d.polygon(S([(cx - 0.085, 0.52), (cx + 0.085, 0.52), (cx + 0.035, 0.95), (cx - 0.035, 0.95)], size), fill=dark)
    d.polygon(S([(cx - 0.068, 0.53), (cx + 0.068, 0.53), (cx + 0.026, 0.935), (cx - 0.026, 0.935)], size), fill=body)
    for t in (0.62, 0.72, 0.82, 0.90):
        half = 0.062 - (t - 0.52) * 0.085
        d.line([(cx - half) * size, t * size, (cx + half) * size, t * size], fill=dark, width=W(size, 32))
    if winged:
        d.polygon(S([(cx - 0.045, 0.53), (cx - 0.006, 0.53), (cx - 0.012, 0.93), (cx - 0.028, 0.93)], size), fill=hi)
        d.polygon(S([(cx + 0.045, 0.53), (cx + 0.006, 0.53), (cx + 0.012, 0.93), (cx + 0.028, 0.93)], size), fill=hi)
    # long prothorax
    d.polygon(S([(cx - 0.055, 0.30), (cx + 0.055, 0.30), (cx + 0.075, 0.54), (cx - 0.075, 0.54)], size), fill=dark)
    d.polygon(S([(cx - 0.042, 0.315), (cx + 0.042, 0.315), (cx + 0.06, 0.525), (cx - 0.06, 0.525)], size), fill=body)
    d.polygon(S([(cx - 0.03, 0.32), (cx + 0.0, 0.32), (cx + 0.01, 0.52), (cx - 0.035, 0.52)], size), fill=hi)
    # raptorial forelegs: folded femur + tibia, raised beside the head
    for sx in (1, -1):
        sh = (cx + sx * 0.045, 0.33)
        elbow = (cx + sx * 0.155, 0.235)
        wrist = (cx + sx * 0.085, 0.115)
        d.line(S([sh, elbow], size), fill=dark, width=W(size, 11))
        d.line(S([elbow, wrist], size), fill=dark, width=W(size, 16))
        d.line(S([(sh[0] + sx * 0.008, sh[1]), (elbow[0], elbow[1] - 0.008)], size), fill=body, width=W(size, 20))
        for t in (0.35, 0.55, 0.75):  # femur spines
            fx = elbow[0] + (wrist[0] - elbow[0]) * t; fy = elbow[1] + (wrist[1] - elbow[1]) * t
            d.line([(fx) * size, fy * size, (fx - sx * 0.018) * size, (fy - 0.02) * size], fill=dark, width=1)
    # triangular head, huge side eyes, thread antennae
    d.polygon(S([(cx - 0.085, 0.16), (cx + 0.085, 0.16), (cx, 0.315)], size), fill=dark)
    d.polygon(S([(cx - 0.068, 0.168), (cx + 0.068, 0.168), (cx, 0.295)], size), fill=body)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.058) * size, 0.175 * size
        rx, ry = size * 0.030, size * 0.042
        d.ellipse([ex - rx, ey - ry, ex + rx, ey + ry], fill=(52, 84, 30, 255))
        d.ellipse([ex - rx * 0.4, ey - ry * 0.5, ex + rx * 0.05, ey], fill=(140, 190, 90, 255))
        pts = [(cx + sx * 0.018, 0.155), (cx + sx * 0.09, 0.05), (cx + sx * 0.155, 0.035)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
    for fx in (-0.016, 0.016):  # pseudopupil dots
        ex, ey = (cx + fx) * size, 0.225 * size
        r = max(1.0, size * 0.010)
        d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(30, 44, 18, 255))
    return img

def draw_grasshopper(size, body, hi, dark, winged=True):
    """Side profile: the hopping pose reads best from the side."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # giant folded hind leg: femur slab up over the back, tibia trailing down
    d.polygon(S([(0.30, 0.46), (0.62, 0.30), (0.70, 0.38), (0.40, 0.56)], size), fill=dark)
    d.polygon(S([(0.335, 0.455), (0.615, 0.325), (0.665, 0.375), (0.415, 0.535)], size), fill=body)
    d.line(S([(0.36, 0.44), (0.60, 0.335)], size), fill=hi, width=W(size, 20))
    d.line(S([(0.665, 0.375), (0.86, 0.78)], size), fill=dark, width=W(size, 22))
    d.line(S([(0.86, 0.78), (0.93, 0.78)], size), fill=dark, width=W(size, 28))
    # small front/mid legs
    for (hx, hy, fx) in ((0.30, 0.62, 0.22), (0.44, 0.66, 0.40)):
        d.line(S([(hx, hy), ((hx + fx) / 2, hy + 0.10), (fx, hy + 0.16)], size), fill=dark, width=W(size, 26))
    # body: thorax + long tapering abdomen
    d.polygon(S([(0.24, 0.40), (0.52, 0.36), (0.88, 0.52), (0.90, 0.60), (0.50, 0.64), (0.26, 0.58)], size), fill=dark)
    d.polygon(S([(0.265, 0.415), (0.515, 0.378), (0.855, 0.525), (0.868, 0.585), (0.50, 0.62), (0.285, 0.565)], size), fill=body)
    d.polygon(S([(0.30, 0.425), (0.52, 0.395), (0.80, 0.52), (0.50, 0.55), (0.32, 0.52)], size), fill=hi)
    for t in (0.60, 0.68, 0.76, 0.83):  # abdomen segments
        x = t
        d.line([(x) * size, (0.475 + (t - 0.55) * 0.16) * size, (x) * size, (0.60 + (t - 0.55) * 0.05) * size],
               fill=dark, width=W(size, 32))
    if winged:  # tegmen wing case along the back
        d.polygon(S([(0.34, 0.385), (0.86, 0.50), (0.875, 0.545), (0.36, 0.45)], size), fill=hi)
        d.line(S([(0.36, 0.415), (0.85, 0.52)], size), fill=dark, width=1)
    # head: sloped face, big eye, long antenna sweeping back
    d.polygon(S([(0.155, 0.36), (0.27, 0.33), (0.285, 0.55), (0.20, 0.62), (0.145, 0.52)], size), fill=dark)
    d.polygon(S([(0.172, 0.375), (0.258, 0.35), (0.268, 0.535), (0.205, 0.595), (0.163, 0.51)], size), fill=body)
    ex, ey = 0.215 * size, 0.435 * size
    rx, ry = size * 0.032, size * 0.048
    d.ellipse([ex - rx, ey - ry, ex + rx, ey + ry], fill=(58, 74, 26, 255))
    d.ellipse([ex - rx * 0.4, ey - ry * 0.55, ex + rx * 0.05, ey - ry * 0.05], fill=(150, 175, 95, 255))
    pts = [(0.20, 0.35), (0.38, 0.16), (0.62, 0.10)]
    d.line(S(pts, size), fill=dark, width=W(size, 32))
    return img

# ---------------------------------------------------------------- spiders
def draw_spider(size, body, hi, dark, kind="wolf", rng=None):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = 0.5
    leg_w = W(size, 26) if kind != "jumping" else W(size, 20)
    leg_spread = {"wolf": 0.40, "trapdoor": 0.36, "jumping": 0.33}[kind]
    for i, ly in enumerate((0.40, 0.475, 0.55, 0.625)):
        for sx in (1, -1):
            hip = (cx + sx * 0.045, ly)
            bend = (ly - 0.51) * 0.9
            knee = (cx + sx * leg_spread * 0.58, ly + bend - 0.035)
            foot = (cx + sx * leg_spread, ly + bend + (0.10 if i > 1 else 0.02))
            d.line(S([hip, knee, foot], size), fill=dark, width=leg_w)
    # abdomen
    ab = {"wolf": (0.155, 0.56, 0.95), "trapdoor": (0.165, 0.55, 0.94), "jumping": (0.135, 0.57, 0.90)}[kind]
    d.ellipse([(cx - ab[0]) * size, ab[1] * size, (cx + ab[0]) * size, ab[2] * size], fill=dark)
    d.ellipse([(cx - ab[0] * 0.86) * size, (ab[1] + 0.015) * size, (cx + ab[0] * 0.86) * size, (ab[2] - 0.015) * size], fill=body)
    d.ellipse([(cx - ab[0] * 0.5) * size, (ab[1] + 0.05) * size, (cx + ab[0] * 0.1) * size, (ab[1] + 0.20) * size], fill=hi)
    if kind == "wolf":  # dorsal stripe + chevrons
        d.polygon(S([(cx - 0.028, 0.60), (cx + 0.028, 0.60), (cx + 0.016, 0.90), (cx - 0.016, 0.90)], size), fill=hi)
        for t in (0.68, 0.78, 0.87):
            d.line(S([(cx - 0.075, t), (cx, t + 0.035), (cx + 0.075, t)], size), fill=dark, width=W(size, 32))
    if kind == "trapdoor":  # glossy dome: strong top highlight
        d.ellipse([(cx - ab[0] * 0.55) * size, (ab[1] + 0.03) * size, (cx + ab[0] * 0.30) * size, (ab[1] + 0.17) * size], fill=hi)
    if kind == "jumping":  # pale banding on the abdomen
        for t in (0.66, 0.78):
            d.line([(cx - ab[0] * 0.8) * size, t * size, (cx + ab[0] * 0.8) * size, t * size], fill=hi, width=W(size, 24))
    # cephalothorax
    cw = {"wolf": 0.105, "trapdoor": 0.115, "jumping": 0.115}[kind]
    d.ellipse([(cx - cw) * size, 0.30 * size, (cx + cw) * size, 0.58 * size], fill=dark)
    d.ellipse([(cx - cw * 0.85) * size, 0.315 * size, (cx + cw * 0.85) * size, 0.565 * size], fill=body)
    d.ellipse([(cx - cw * 0.5) * size, 0.33 * size, (cx + cw * 0.05) * size, 0.45 * size], fill=hi)
    # eyes + chelicerae
    if kind == "jumping":  # two huge front eyes -- the jumping spider face
        for sx in (1, -1):
            ex, ey = (cx + sx * 0.042) * size, 0.335 * size
            r = max(1.8, size * 0.034)
            d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(16, 14, 12, 255))
            d.ellipse([ex - r * 0.35, ey - r * 0.45, ex + r * 0.1, ey], fill=(210, 220, 215, 255))
            ex2, ey2 = (cx + sx * 0.088) * size, 0.345 * size
            r2 = max(1.1, size * 0.016)
            d.ellipse([ex2 - r2, ey2 - r2, ex2 + r2, ey2 + r2], fill=(16, 14, 12, 255))
    else:
        for sx in (1, -1):
            for (dx, dy, rr) in ((0.028, 0.335, 0.020), (0.062, 0.345, 0.015)):
                ex, ey = (cx + sx * dx) * size, dy * size
                r = max(1.1, size * rr)
                d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(16, 14, 12, 255))
    for sx in (1, -1):  # chelicerae / pedipalps
        pts = [(cx + sx * 0.03, 0.315), (cx + sx * 0.055, 0.27), (cx + sx * 0.045, 0.235)]
        d.line(S(pts, size), fill=dark, width=W(size, 22))
    return img

def draw_cellar_mom(size, rng):
    """Mommy long-legs: tiny body on very long thin legs, holding a
    head-sized ball of spiderlings in front of her."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (84, 62, 40, 255); body = (140, 112, 76, 255); hi = (178, 150, 104, 255)
    cx, cy = 0.5, 0.52
    for i, ang in enumerate((-2.65, -2.05, -1.45, -0.85, -0.35, 0.15, 0.65, 1.15)):
        # eight very long legs radiating; knees high, feet at the rim
        kx = cx + math.cos(ang) * 0.20; ky = cy + math.sin(ang) * 0.20 - 0.05
        fx = cx + math.cos(ang) * 0.465; fy = cy + math.sin(ang) * 0.44
        d.line(S([(cx, cy), (kx, ky), (fx, fy)], size), fill=dark, width=W(size, 40))
    # small oval body
    d.ellipse([(cx - 0.075) * size, (cy - 0.10) * size, (cx + 0.075) * size, (cy + 0.115) * size], fill=dark)
    d.ellipse([(cx - 0.062) * size, (cy - 0.088) * size, (cx + 0.062) * size, (cy + 0.10) * size], fill=body)
    d.ellipse([(cx - 0.038) * size, (cy - 0.075) * size, (cx + 0.005) * size, (cy + 0.01) * size], fill=hi)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.03) * size, (cy - 0.075) * size
        r = max(1.1, size * 0.014)
        d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(20, 14, 8, 255))
    # the precious cargo: a ball of spiderlings clutched below the head
    bx, by, br = cx, cy - 0.235, 0.105
    d.ellipse([(bx - br) * size, (by - br) * size, (bx + br) * size, (by + br) * size], fill=dark)
    d.ellipse([(bx - br * 0.86) * size, (by - br * 0.86) * size, (bx + br * 0.86) * size, (by + br * 0.86) * size],
              fill=(168, 140, 100, 255))
    for _ in range(46):  # spiderling bodies packed in the ball
        a = rng.uniform(0, 2 * math.pi); rr = rng.uniform(0, br * 0.78)
        x, y = bx + math.cos(a) * rr, by + math.sin(a) * rr
        r = max(1.0, size * rng.uniform(0.010, 0.020))
        d.ellipse([(x * size - r), (y * size - r), (x * size + r), (y * size + r)], fill=dark)
        d.ellipse([(x * size - r * 0.5), (y * size - r * 0.6), (x * size + r * 0.2), (y * size + r * 0.1)], fill=hi)
    for sx in (1, -1):  # pedipalps cradling the ball
        pts = [(cx + sx * 0.045, cy - 0.06), (cx + sx * 0.10, by + 0.02), (cx + sx * 0.075, by + 0.085)]
        d.line(S(pts, size), fill=dark, width=W(size, 30))
    return img

def draw_spider_leg(size):
    """A single detached cellar-spider leg, strangely branching, with
    irregular clawed appendages -- flailing blindly."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (84, 62, 40, 255); body = (146, 116, 78, 255); hi = (182, 152, 106, 255)
    # main limb: thick base at bottom-left, jointed, tapering to a claw
    main = [(0.16, 0.86), (0.30, 0.66), (0.50, 0.55), (0.68, 0.38), (0.84, 0.20)]
    d.line(S(main, size), fill=dark, width=W(size, 9))
    d.line(S(main, size), fill=body, width=W(size, 14))
    for (x, y) in main[1:-1]:  # joint knobs
        r = max(1.6, size * 0.026)
        d.ellipse([x * size - r, y * size - r, x * size + r, y * size + r], fill=dark)
        d.ellipse([x * size - r * 0.55, y * size - r * 0.55, x * size + r * 0.55, y * size + r * 0.55], fill=hi)
    # torn base
    x, y = main[0]
    r = size * 0.055
    d.ellipse([x * size - r, y * size - r, x * size + r, y * size + r], fill=(196, 150, 120, 255))
    d.ellipse([x * size - r * 0.45, y * size - r * 0.45, x * size + r * 0.45, y * size + r * 0.45], fill=(150, 96, 74, 255))
    # branches: irregular offshoots with their own little claws
    branches = [((0.30, 0.66), (0.20, 0.48), (0.24, 0.33)),
                ((0.50, 0.55), (0.44, 0.36), (0.52, 0.24)),
                ((0.68, 0.38), (0.60, 0.22), (0.66, 0.10)),
                ((0.50, 0.55), (0.66, 0.62), (0.78, 0.55))]
    for (a, b, c) in branches:
        d.line(S([a, b, c], size), fill=dark, width=W(size, 18))
        d.line(S([a, b, c], size), fill=body, width=W(size, 26))
        for sx in (1, -1):  # claw pair at each branch tip
            tip = c
            d.line(S([tip, (tip[0] + sx * 0.045, tip[1] - 0.05)], size), fill=dark, width=W(size, 30))
    # main claw at the end of the limb
    tip = main[-1]
    for sx in (1, -1):
        d.line(S([tip, (tip[0] + sx * 0.05, tip[1] - 0.065)], size), fill=dark, width=W(size, 22))
        d.line(S([(tip[0] + sx * 0.05, tip[1] - 0.065), (tip[0] + sx * 0.028, tip[1] - 0.105)], size),
               fill=dark, width=W(size, 30))
    return img

def draw_water_scorpion(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (92, 94, 96, 255); body = (172, 174, 172, 255); hi = (204, 206, 200, 255)
    cx = 0.5
    # breathing tube: two filaments trailing from the rear
    for sx in (1, -1):
        pts = [(cx + sx * 0.02, 0.90), (cx + sx * 0.045, 0.965)]
        d.line(S(pts, size), fill=dark, width=W(size, 28))
    # flat oval body
    d.ellipse([(cx - 0.14) * size, 0.30 * size, (cx + 0.14) * size, 0.92 * size], fill=dark)
    d.ellipse([(cx - 0.122) * size, 0.315 * size, (cx + 0.122) * size, 0.905 * size], fill=body)
    d.ellipse([(cx - 0.075) * size, 0.35 * size, (cx + 0.005) * size, 0.62 * size], fill=hi)
    for t in (0.52, 0.63, 0.74, 0.84):  # abdominal segmentation
        half = 0.115 - (t - 0.30) * 0.05
        d.line([(cx - half) * size, t * size, (cx + half) * size, t * size], fill=dark, width=W(size, 32))
    d.line(S([(cx, 0.33), (cx, 0.90)], size), fill=dark, width=1)  # center ridge
    # walking legs
    for (ly, spread) in ((0.55, 0.30), (0.68, 0.32), (0.80, 0.29)):
        for sx in (1, -1):
            hip = (cx + sx * 0.09, ly)
            foot = (cx + sx * spread, ly + 0.10)
            d.line(S([hip, (cx + sx * spread * 0.62, ly + 0.03), foot], size), fill=dark, width=W(size, 26))
    # raptorial forelegs folded at the front
    for sx in (1, -1):
        sh = (cx + sx * 0.055, 0.34)
        elbow = (cx + sx * 0.155, 0.27)
        tip = (cx + sx * 0.095, 0.165)
        d.line(S([sh, elbow], size), fill=dark, width=W(size, 13))
        d.line(S([elbow, tip], size), fill=dark, width=W(size, 18))
    # head + eyes
    d.ellipse([(cx - 0.075) * size, 0.17 * size, (cx + 0.075) * size, 0.33 * size], fill=dark)
    d.ellipse([(cx - 0.062) * size, 0.182 * size, (cx + 0.062) * size, 0.318 * size], fill=body)
    for sx in (1, -1):
        ex, ey = (cx + sx * 0.042) * size, 0.225 * size
        r = max(1.3, size * 0.020)
        d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(30, 30, 30, 255))
        pts = [(cx + sx * 0.015, 0.185), (cx + sx * 0.05, 0.115), (cx + sx * 0.085, 0.10)]
        d.line(S(pts, size), fill=dark, width=W(size, 32))
    return img

# ---------------------------------------------------------------- the deer host
def draw_deer(size):
    """Paralyzed, emaciated deer lying on its side: ribs showing, thin legs
    folded flat beneath it, head raised on a stiff neck, eye wide."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    dark = (92, 66, 42, 255); body = (146, 110, 72, 255); hi = (178, 142, 98, 255)
    pale = (216, 196, 164, 255); rib = (104, 78, 48, 255)
    # tail, tucked at the rear
    d.polygon(S([(0.115, 0.50), (0.07, 0.565), (0.12, 0.60)], size), fill=dark)
    # legs folded flat along the ground: thin horizontal shapes, cannon
    # bone lying forward/back, dark hooves at the ends
    for (x0, x1, y0, hoof_first) in ((0.24, 0.47, 0.69, True), (0.56, 0.79, 0.705, False)):
        d.polygon(S([(x0, y0), (x1, y0), (x1, y0 + 0.034), (x0, y0 + 0.034)], size), fill=dark)
        d.polygon(S([(x0 + 0.010, y0 + 0.006), (x1 - 0.010, y0 + 0.006),
                     (x1 - 0.010, y0 + 0.028), (x0 + 0.010, y0 + 0.028)], size), fill=body)
        kx = (x0 + x1) / 2  # knee bump on top of the folded cannon
        d.ellipse([(kx - 0.016) * size, (y0 - 0.012) * size, (kx + 0.016) * size, (y0 + 0.012) * size], fill=dark)
        d.ellipse([(kx - 0.011) * size, (y0 - 0.008) * size, (kx + 0.011) * size, (y0 + 0.008) * size], fill=body)
        hx = x0 if hoof_first else x1 - 0.026
        d.polygon(S([(hx, y0 + 0.002), (hx + 0.026, y0 + 0.002), (hx + 0.026, y0 + 0.032), (hx, y0 + 0.032)], size),
                  fill=(40, 30, 22, 255))  # hoof
    # a second pair, slightly higher and offset (the far-side legs)
    for (x0, x1) in ((0.30, 0.50), (0.60, 0.82)):
        d.line(S([(x0, 0.665), (x1, 0.665)], size), fill=dark, width=W(size, 22))
    # emaciated torso: hip and shoulder bones jutting, sunken flank
    torso = [(0.125, 0.455), (0.30, 0.395), (0.55, 0.41), (0.76, 0.435), (0.83, 0.51),
             (0.79, 0.63), (0.55, 0.675), (0.30, 0.66), (0.145, 0.595)]
    d.polygon(S(torso, size), fill=dark)
    inner = [(0.478 + (x - 0.478) * 0.93, 0.535 + (y - 0.535) * 0.87) for x, y in torso]
    d.polygon(S(inner, size), fill=body)
    d.ellipse([0.185 * size, 0.415 * size, 0.315 * size, 0.515 * size], fill=hi)   # hip bone
    d.ellipse([0.685 * size, 0.43 * size, 0.80 * size, 0.52 * size], fill=hi)     # shoulder
    for i in range(5):  # ribs: pale arcs standing out over the sunken flank
        x = 0.385 + i * 0.052
        d.arc([x * size, 0.40 * size, (x + 0.085) * size, 0.68 * size], start=295, end=115,
              fill=rib, width=W(size, 24))
        d.arc([x * size, 0.405 * size, (x + 0.085) * size, 0.675 * size], start=295, end=115,
              fill=hi, width=1)
    d.ellipse([0.40 * size, 0.51 * size, 0.60 * size, 0.635 * size], fill=(130, 98, 62, 255))  # sunken belly
    # neck: thick, stiff, angled up from the shoulder
    d.polygon(S([(0.755, 0.475), (0.845, 0.315), (0.905, 0.345), (0.845, 0.515)], size), fill=dark)
    d.polygon(S([(0.775, 0.465), (0.852, 0.34), (0.888, 0.358), (0.838, 0.495)], size), fill=body)
    d.polygon(S([(0.79, 0.45), (0.855, 0.345), (0.868, 0.355), (0.815, 0.465)], size), fill=hi)
    # head: long deer face held high, big ear swept back
    d.polygon(S([(0.825, 0.245), (0.885, 0.275), (0.878, 0.315), (0.815, 0.295)], size), fill=dark)  # ear
    d.polygon(S([(0.838, 0.26), (0.872, 0.28), (0.868, 0.30), (0.83, 0.285)], size), fill=pale)
    head = [(0.83, 0.29), (0.905, 0.295), (0.955, 0.345), (0.958, 0.395),
            (0.915, 0.42), (0.845, 0.385), (0.822, 0.335)]
    d.polygon(S(head, size), fill=dark)
    inner_h = [(0.889 + (x - 0.889) * 0.88, 0.355 + (y - 0.355) * 0.85) for x, y in head]
    d.polygon(S(inner_h, size), fill=body)
    d.polygon(S([(0.845, 0.315), (0.90, 0.315), (0.925, 0.345), (0.86, 0.345)], size), fill=hi)
    # eye: white showing around a dark pupil -- terrified, darting
    ex, ey = 0.878 * size, 0.338 * size
    r = max(1.7, size * 0.021)
    d.ellipse([ex - r, ey - r, ex + r, ey + r], fill=(250, 246, 238, 255))
    r2 = r * 0.52
    d.ellipse([ex - r2, ey - r2 * 0.4, ex + r2, ey + r2 * 1.6], fill=(24, 16, 10, 255))
    d.ellipse([0.932 * size, 0.372 * size, 0.958 * size, 0.398 * size], fill=(30, 22, 16, 255))  # nose
    d.line(S([(0.915, 0.408), (0.945, 0.402)], size), fill=(30, 22, 16, 255), width=1)  # mouth line
    # dermatik swellings under the skin of the neck and flank
    for (x, y, rr) in ((0.815, 0.455, 0.020), (0.335, 0.465, 0.024), (0.615, 0.44, 0.017)):
        d.ellipse([(x - rr) * size, (y - rr) * size, (x + rr) * size, (y + rr) * size], fill=hi)
        d.ellipse([(x - rr * 0.4) * size, (y - rr * 0.45) * size, (x + rr * 0.25) * size, (y + rr * 0.2) * size],
                  fill=pale)
    return img

# ---------------------------------------------------------------- jobs
BROWN_ANT = dict(body=(146, 104, 58, 255), hi=(182, 140, 84, 255), dark=(94, 64, 32, 255))
LUNA = dict(base=(196, 238, 226, 255), hi=(232, 252, 246, 255), band=(148, 205, 196, 255),
            grain=(120, 180, 170, 255), pale=(250, 254, 250, 255), body_c=(178, 224, 210, 255),
            body_hi=(214, 244, 234, 255), dark=(84, 128, 118, 255))

JOBS = [
    # 96x96 (resolved volume > 400 L)
    ("pngs_giant_96x96/monsters/mon_ant_acid_soldier_mega/mon_ant_acid_soldier_mega", 96,
     lambda s: draw_ant(s, woolly=True, acid=True, rng=rng_for("mon_ant_acid_soldier_mega"), **BROWN_ANT)),
    ("pngs_giant_96x96/monsters/mon_fly_mega/mon_fly_mega", 96,
     lambda s: draw_fly(s, (122, 148, 84, 255), (158, 184, 112, 255), (70, 84, 44, 255),
                        hairy=True, rng=rng_for("mon_fly_mega"))),
    ("pngs_giant_96x96/monsters/mon_halfworm_graboid/mon_halfworm_graboid", 96,
     lambda s: draw_halfworm(s, (232, 148, 138, 255), (246, 186, 172, 255), (172, 92, 84, 255))),
    # 64x64 (100-400 L)
    ("pngs_large_64x64/monsters/mon_cicada/mon_cicada", 64, lambda s: draw_cicada(s)),
    ("pngs_large_64x64/monsters/mon_mantis_giant/mon_mantis_giant", 64,
     lambda s: draw_mantis(s, (116, 186, 84, 255), (156, 216, 118, 255), (62, 110, 40, 255))),
    ("pngs_large_64x64/monsters/mon_moth_titan/mon_moth_titan", 64,
     lambda s: draw_big_moth(s, dust=True, tails=True, rng=rng_for("mon_moth_titan"), **LUNA)),
    ("pngs_large_64x64/monsters/mon_spider_cellar_mom/mon_spider_cellar_mom", 64,
     lambda s: draw_cellar_mom(s, rng_for("mon_spider_cellar_mom"))),
    ("pngs_large_64x64/monsters/mon_dermatik_incubator_deer/mon_dermatik_incubator_deer", 64,
     lambda s: draw_deer(s)),
    ("pngs_large_64x64/monsters/mon_halfworm_small/mon_halfworm_small", 64,
     lambda s: draw_halfworm(s, (232, 148, 138, 255), (246, 186, 172, 255), (172, 92, 84, 255))),
    # 32x32 (< 100 L)
    ("pngs_normal_32x32/monsters/mon_cicada_nymph/mon_cicada_nymph", 32, lambda s: draw_cicada_nymph(s)),
    ("pngs_normal_32x32/monsters/mon_dermatik_midwife/mon_dermatik_midwife", 32,
     lambda s: draw_dermatik(s, (188, 52, 40, 255), (224, 102, 84, 255), (110, 26, 20, 255))),
    ("pngs_normal_32x32/monsters/mon_firefly_larva/mon_firefly_larva", 32,
     lambda s: draw_grub(s, (196, 148, 60, 255), (228, 188, 100, 255), (120, 84, 34, 255), glow=True)),
    ("pngs_normal_32x32/monsters/mon_fly_larva/mon_fly_larva", 32,
     lambda s: draw_grub(s, (178, 214, 130, 255), (208, 238, 168, 255), (110, 142, 66, 255), curve=0.06)),
    ("pngs_normal_32x32/monsters/mon_fly_small/mon_fly_small", 32,
     lambda s: draw_fly(s, (140, 168, 96, 255), (174, 200, 128, 255), (80, 98, 52, 255))),
    ("pngs_normal_32x32/monsters/mon_grasshopper_giant/mon_grasshopper_giant", 32,
     lambda s: draw_grasshopper(s, (128, 196, 92, 255), (168, 224, 130, 255), (66, 116, 44, 255))),
    ("pngs_normal_32x32/monsters/mon_grasshopper_nymph/mon_grasshopper_nymph", 32,
     lambda s: draw_grasshopper(s, (140, 204, 104, 255), (178, 230, 142, 255), (74, 124, 50, 255), winged=False)),
    ("pngs_normal_32x32/monsters/mon_mantis_nymph/mon_mantis_nymph", 32,
     lambda s: draw_mantis(s, (124, 194, 92, 255), (164, 222, 126, 255), (68, 118, 44, 255), winged=False)),
    ("pngs_normal_32x32/monsters/mon_mosquito_larva/mon_mosquito_larva", 32, lambda s: draw_mosquito_larva(s)),
    ("pngs_normal_32x32/monsters/mon_moth_fly/mon_moth_fly", 32, lambda s: draw_moth_fly(s)),
    ("pngs_normal_32x32/monsters/mon_moth_fly_larva/mon_moth_fly_larva", 32,
     lambda s: draw_grub(s, (196, 84, 66, 255), (228, 128, 106, 255), (122, 44, 34, 255), curve=-0.08)),
    ("pngs_normal_32x32/monsters/mon_spider_cellar_leg/mon_spider_cellar_leg", 32, lambda s: draw_spider_leg(s)),
    ("pngs_normal_32x32/monsters/mon_spider_jumping_giant_s/mon_spider_jumping_giant_s", 32,
     lambda s: draw_spider(s, (226, 224, 218, 255), (246, 244, 238, 255), (120, 116, 108, 255), kind="jumping")),
    ("pngs_normal_32x32/monsters/mon_spider_trapdoor_s/mon_spider_trapdoor_s", 32,
     lambda s: draw_spider(s, (86, 110, 178, 255), (140, 168, 224, 255), (44, 60, 110, 255), kind="trapdoor")),
    ("pngs_normal_32x32/monsters/mon_spider_wolf_giant_s/mon_spider_wolf_giant_s", 32,
     lambda s: draw_spider(s, (138, 106, 70, 255), (176, 144, 98, 255), (84, 60, 34, 255), kind="wolf")),
    ("pngs_normal_32x32/monsters/mon_water_scorpion_larva/mon_water_scorpion_larva", 32,
     lambda s: draw_water_scorpion(s)),
    ("pngs_normal_32x32/monsters/mon_worm_larva/mon_worm_larva", 32,
     lambda s: draw_worm(s, (236, 232, 224, 255), (250, 248, 242, 255), (168, 162, 150, 255))),
]

CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for rel, size, make in JOBS:
    folder = os.path.join(BASE, os.path.dirname(rel))
    os.makedirs(folder, exist_ok=True)
    name = os.path.basename(rel)
    img = make(size)
    img.save(os.path.join(BASE, rel + ".png"))
    entry = {"id": name, "fg": name, "bg": ""}
    with open(os.path.join(BASE, rel + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", rel + ".png", img.size)
print("total", len(JOBS))
