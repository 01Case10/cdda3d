#!/usr/bin/env python3
"""
Draw the missing regional-pseudo, marine-flora, and decorative furniture
sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/, CDDA 6a3e59e0) --
23 furniture ids missing from UltiCa in the shipped universe
(coverage-audit/missing.json):

  Regional pseudo (furniture-regional-pseudo.json) -- all ten are named
  "pseudo furniture": "This should never actually show up, it should have
  been replaced with regional furniture." No color/symbol in game data;
  drawn anyway (zero is zero), each as the small flora the regional role
  names:
    f_region_flower            wildflower patch
    f_region_weed              scraggly weed clump
    f_region_water_plant       emergent water plant, pads and reeds
    f_region_plain             plain grass groundcover tuft
    f_region_forest            forest-floor fern and litter
    f_region_forest_dense      denser, darker undergrowth
    f_region_forest_water      swamp reeds and cattails
    f_region_flower_decorative neat cultivated flower bed
    f_region_lake_bed          submerged lakebed ribbon weed
    f_region_ocean_bed         oceanbed kelp ribbons

  Marine flora (furniture-flora_marine.json):
    f_algae_red      "grassy reddish marine plants... rubbery little
                      fronds"  color red, symbol ','
    f_algae_green    "A short patch of grassy green seaweed"  green, ','
    f_kelp_sugar     "brown algae... a single leaflike blade that can
                      reach more than twice the height of a man"  brown, 'i'
    f_sea_lettuce    "leafy algae... grows in leafy little patches around
                      shallows and tide pools"  green, ','
    f_bladderwrack   "the air bladders that grow all over its leafy
                      fronds"  green, ','
    f_sea_anemone    "This sessile polyp waves its stinging tentacles
                      with the motion of the tides"  pink, 'f'
    f_sea_sponge     "An immobile, lumpy little creature... its porous
                      body passively filters plankton"  brown, 'f'

  Decorative (furniture-decorative.json):
    f_shackle               "A pair of metal shackles with heavy chains
                             mounted to a wall or floor."  light_gray, '8'
    f_gazing_ball           "A mirrored glass globe mounted on a cement
                             pedestal."  white, 'o'
    f_lighthouse_light      "Very high power, modern lighthouse LED light
                             source with attached heatsinks."  light_gray
    f_styling_station       "a countertop with a mirror mounted above it
                             and just enough room and cubbies to store a
                             stylist's equipment"  white, '}'
    f_indoor_flagpole       "A 2.5-meter tall flagpole on a weighted
                             stand."  light_gray, 'F'
    f_indoor_flagpole_wooden  the same in wood.  brown, 'F'

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet.
Furniture draws over the terrain beneath it, so every sprite is an
object on a transparent background with entry bg "". Flat fills, dark
outlines, one highlight pass -- UltiCa style. All drawn pixels are
fully opaque; sheen is done with opaque lighter shades, never
semi-transparent ink. Deterministic: every random choice comes from a
per-sprite seeded Random.
Output: pngs_normal_32x32/furniture/<id>/ (png + tile-entry json +
credits). This script creates ONLY its own 23 folders.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def blades(d, rng, n, cols, ybase=31, hmin=6, hmax=13, sway=2.2, width=1):
    """A tuft of grass-like blades rising from ybase."""
    for _ in range(n):
        x = rng.uniform(2, S - 3)
        h = rng.uniform(hmin, hmax)
        tip = x + rng.uniform(-sway, sway)
        col = cols[rng.randrange(len(cols))]
        d.line([x, ybase, (x + tip) / 2, ybase - h * 0.6, tip, ybase - h],
               fill=col, width=width)

def leaf_blob(d, cx, cy, r, cols, rng):
    for _ in range(int(r * r * 1.1)):
        a = rng.uniform(0, math.tau); rr = r * math.sqrt(rng.random()) * 0.92
        x, y = cx + math.cos(a) * rr, cy + math.sin(a) * rr * 0.8
        lr = rng.uniform(1.1, 2.4)
        d.ellipse([x - lr, y - lr * 0.6, x + lr, y + lr * 0.6],
                  fill=cols[rng.randrange(len(cols))])

# ------------------------------------------------------- regional pseudo
def draw_region_plain():
    rng = random.Random(7101)
    img = new_img(); d = ImageDraw.Draw(img)
    blades(d, rng, 22, [(74, 128, 52, 255), (94, 150, 62, 255),
                        (60, 108, 44, 255)], hmin=5, hmax=10)
    return img

def draw_region_flower():
    rng = random.Random(7102)
    img = new_img(); d = ImageDraw.Draw(img)
    blades(d, rng, 16, [(70, 124, 50, 255), (90, 146, 60, 255)],
           hmin=5, hmax=9)
    # wildflower heads: small colored dots on thin stems
    cols = [(238, 200, 66, 255), (232, 120, 160, 255), (240, 240, 244, 255),
            (196, 110, 220, 255)]
    for _ in range(9):
        x = rng.uniform(3, 29); y = rng.uniform(14, 24)
        d.line([x, 31, x, y + 2], fill=(70, 124, 50, 255), width=1)
        c = cols[rng.randrange(len(cols))]
        for (dx, dy) in [(-1.4, 0), (1.4, 0), (0, -1.4), (0, 1.4)]:
            d.ellipse([x + dx - 1, y + dy - 1, x + dx + 1, y + dy + 1], fill=c)
        d.point((x, y), fill=(250, 226, 120, 255))
    return img

def draw_region_weed():
    rng = random.Random(7103)
    img = new_img(); d = ImageDraw.Draw(img)
    stem = (88, 104, 52, 255)
    # scraggly thin stems leaning every which way, jagged leaves
    for _ in range(7):
        x = rng.uniform(4, 28); lean = rng.uniform(-5, 5)
        top = rng.uniform(8, 16)
        d.line([x, 31, x + lean * 0.4, 20, x + lean, top], fill=stem, width=1)
        for t in (0.35, 0.6, 0.85):
            px = x + lean * t; py = 31 - (31 - top) * t
            for sgn in (-1, 1):
                d.polygon([(px, py), (px + sgn * 3.2, py - 1.2),
                           (px + sgn * 1.2, py + 1.4)], fill=(104, 122, 58, 255))
    # a couple of dry seed heads
    for (x, y) in [(9, 10), (23, 8)]:
        for a in range(6):
            ang = a * math.tau / 6
            d.line([x, y, x + 2.4 * math.cos(ang), y + 2.4 * math.sin(ang)],
                   fill=(168, 150, 96, 255), width=1)
    return img

def draw_region_water_plant():
    rng = random.Random(7104)
    img = new_img(); d = ImageDraw.Draw(img)
    # emergent reeds
    blades(d, rng, 9, [(52, 116, 66, 255), (70, 140, 78, 255)],
           hmin=10, hmax=17, sway=1.4)
    # lily pads floating at the base
    for (cx, cy, r) in [(9, 27, 4.4), (20, 29, 5.2), (26, 26, 3.4)]:
        d.ellipse([cx - r, cy - r * 0.42, cx + r, cy + r * 0.42],
                  fill=(44, 122, 58, 255), outline=(30, 92, 42, 255))
        d.polygon([(cx, cy), (cx + r, cy - r * 0.3), (cx + r * 0.7, cy + r * 0.3)],
                  fill=(0, 0, 0, 0))
        d.ellipse([cx - r, cy - r * 0.42, cx + r, cy + r * 0.42],
                  outline=(30, 92, 42, 255))
    # one small white water flower
    d.ellipse([17.4, 23.4, 20.6, 26.6], fill=(244, 244, 238, 255))
    d.point((19, 25), fill=(246, 210, 90, 255))
    return img

def draw_region_forest(dense):
    rng = random.Random(7106 if dense else 7105)
    img = new_img(); d = ImageDraw.Draw(img)
    dark = (26, 74, 36, 255) if dense else (36, 96, 46, 255)
    mid = (44, 116, 54, 255) if dense else (58, 138, 66, 255)
    # fern fronds: arcing stems with pinnae
    n = 9 if dense else 6
    for _ in range(n):
        x = rng.uniform(3, 29); h = rng.uniform(11, 17 if dense else 14)
        lean = rng.uniform(-4, 4)
        pts = [(x + lean * (t * t), 31 - h * t) for t in [i / 8 for i in range(9)]]
        d.line(pts, fill=dark, width=1)
        for (px, py) in pts[2:-1]:
            pl = rng.uniform(1.8, 3.2)
            d.line([px, py, px - pl, py + 1.2], fill=mid, width=1)
            d.line([px, py, px + pl, py + 1.2], fill=mid, width=1)
    # leaf litter at the base
    for _ in range(10 if dense else 7):
        x = rng.uniform(1, 29); y = rng.uniform(27, 31)
        d.ellipse([x, y, x + rng.uniform(2, 3.6), y + 1.4],
                  fill=(110, 84, 44, 255) if rng.random() < 0.5 else (84, 66, 34, 255))
    return img

def draw_region_forest_water():
    rng = random.Random(7107)
    img = new_img(); d = ImageDraw.Draw(img)
    # tall wet reeds
    blades(d, rng, 12, [(46, 104, 56, 255), (62, 128, 66, 255)],
           hmin=13, hmax=20, sway=1.6)
    # cattail heads: brown cylinders on two stems
    for (x, top) in [(10, 9), (22, 12)]:
        d.line([x, 31, x, top], fill=(52, 110, 58, 255), width=1)
        d.rectangle([x - 1.2, top, x + 1.2, top + 6], fill=(110, 72, 38, 255))
        d.line([x, top, x, top - 2.4], fill=(52, 110, 58, 255), width=1)
    # dark wet mud sheen at the base
    d.rectangle([2, 30, 30, 31], fill=(52, 44, 34, 255))
    return img

def draw_region_flower_decorative():
    rng = random.Random(7108)
    img = new_img(); d = ImageDraw.Draw(img)
    # neat cultivated rows: dark soil band with ordered flower clusters
    d.rectangle([1, 26, 31, 31], fill=(74, 56, 38, 255))
    cols = [(226, 62, 62, 255), (240, 150, 40, 255), (236, 110, 170, 255),
            (244, 244, 248, 255)]
    for row, y in enumerate((18, 24)):
        for k in range(5):
            x = 4 + k * 6 + (2 if row else 0)
            leaf_blob(d, x, y + 3, 2.6, [(52, 116, 48, 255), (66, 138, 56, 255)], rng)
            c = cols[(k + row * 2) % len(cols)]
            for _ in range(4):
                fx = x + rng.uniform(-2, 2); fy = y + rng.uniform(-1.6, 1.6)
                d.ellipse([fx - 1.4, fy - 1.4, fx + 1.4, fy + 1.4], fill=c)
            d.ellipse([x - 0.8, y - 0.8, x + 0.8, y + 0.8],
                      fill=(250, 224, 110, 255))
    return img

def draw_region_lake_bed():
    rng = random.Random(7109)
    img = new_img(); d = ImageDraw.Draw(img)
    # submerged ribbon weed: wavy vertical ribbons, blue-green
    for _ in range(6):
        x0 = rng.uniform(3, 28); h = rng.uniform(13, 20); ph = rng.uniform(0, 6)
        pts = [(x0 + 1.8 * math.sin(y / 4.5 + ph), 31 - y) for y in range(0, int(h), 2)]
        d.line(pts, fill=(38, 118, 96, 255), width=2)
        d.line([(x - 0.6, y) for x, y in pts], fill=(64, 156, 126, 255), width=1)
    # pale pebbles on the bed
    for _ in range(7):
        x = rng.uniform(1, 29); y = rng.uniform(28, 31)
        d.ellipse([x, y, x + rng.uniform(2, 3.4), y + 1.4], fill=(148, 152, 138, 255))
    return img

def draw_region_ocean_bed():
    rng = random.Random(7110)
    img = new_img(); d = ImageDraw.Draw(img)
    # deeper teal kelp ribbons, taller and broader than the lake weed
    for _ in range(4):
        x0 = rng.uniform(4, 27); h = rng.uniform(19, 25); ph = rng.uniform(0, 6)
        pts = [(x0 + 2.4 * math.sin(y / 5.0 + ph), 31 - y) for y in range(0, int(h), 2)]
        d.line(pts, fill=(24, 96, 92, 255), width=3)
        d.line([(x - 0.8, y) for x, y in pts], fill=(44, 134, 124, 255), width=1)
    # shell fragments on the sand
    for (x, y) in [(7, 29), (24, 30)]:
        d.pieslice([x - 2.4, y - 2.4, x + 2.4, y + 2.4], 180, 360,
                   fill=(226, 214, 192, 255))
    return img

# ---------------------------------------------------------- marine flora
def draw_algae_red():
    rng = random.Random(7111)
    img = new_img(); d = ImageDraw.Draw(img)
    cols = [(168, 44, 48, 255), (140, 32, 40, 255), (192, 70, 62, 255)]
    # rubbery branching fronds: short stems splitting into rounded blades
    def frond(x, y, ang, ln, depth):
        ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
        d.line([x, y, ex, ey], fill=cols[1], width=2)
        if depth <= 0:
            d.ellipse([ex - 2.0, ey - 2.6, ex + 2.0, ey + 1.2],
                      fill=cols[rng.randrange(len(cols))])
            return
        for na in (ang - 0.5, ang + 0.15, ang + 0.55):
            frond(ex, ey, na + rng.uniform(-0.08, 0.08), ln * 0.62, depth - 1)
    for x in (7, 14, 21, 26):
        frond(x, 31, rng.uniform(-0.15, 0.15), rng.uniform(7, 9), 2)
    return img

def draw_algae_green():
    rng = random.Random(7112)
    img = new_img(); d = ImageDraw.Draw(img)
    # short patch of grassy green seaweed: many thin blades, low
    blades(d, rng, 34, [(56, 152, 62, 255), (74, 176, 76, 255),
                        (44, 128, 52, 255)], hmin=4, hmax=9, sway=2.6)
    return img

def draw_kelp_sugar():
    rng = random.Random(7113)
    img = new_img(); d = ImageDraw.Draw(img)
    # a single tall leaflike blade with ruffled edges, from a holdfast
    cx = 16.0
    left, right = [], []
    for y in range(31, 1, -2):
        t = (31 - y) / 30.0
        wdt = 3.4 * math.sin(math.pi * min(1.0, 0.25 + t * 0.9)) + 0.8
        sway = 2.6 * math.sin(t * 4.2)
        ruf = 0.9 * math.sin(t * 22.0)
        left.append((cx + sway - wdt + ruf * 0.4, y))
        right.append((cx + sway + wdt - ruf * 0.4, y))
    d.polygon(left + right[::-1], fill=(122, 84, 40, 255),
              outline=(84, 56, 24, 255))
    # central rib + ruffle shading lines
    rib = [(cx + 2.6 * math.sin(((31 - y) / 30.0) * 4.2), y) for y in range(31, 1, -2)]
    d.line(rib, fill=(150, 108, 56, 255), width=1)
    for y in range(28, 4, -5):
        t = (31 - y) / 30.0
        x = cx + 2.6 * math.sin(t * 4.2)
        d.line([x - 2.6, y, x + 2.6, y - 1], fill=(104, 70, 32, 255), width=1)
    # holdfast: gnarled root ball at the base
    d.ellipse([12.6, 28.4, 19.4, 31.6], fill=(92, 62, 30, 255))
    return img

def draw_sea_lettuce():
    rng = random.Random(7114)
    img = new_img(); d = ImageDraw.Draw(img)
    # leafy little patches: ruffled rosette leaves, bright green
    for (cx, cy, r) in [(10, 24, 6.4), (21, 27, 7.2), (25, 20, 4.8)]:
        for k in range(6):
            a = k * math.tau / 6 + rng.uniform(-0.2, 0.2)
            lx, ly = cx + math.cos(a) * r * 0.55, cy + math.sin(a) * r * 0.4
            d.ellipse([lx - r * 0.52, ly - r * 0.34, lx + r * 0.52, ly + r * 0.34],
                      fill=(86, 186, 82, 255), outline=(56, 148, 54, 255))
        d.ellipse([cx - r * 0.4, cy - r * 0.26, cx + r * 0.4, cy + r * 0.26],
                  fill=(110, 208, 100, 255), outline=(56, 148, 54, 255))
        # ruffle highlight arcs
        d.arc([cx - r * 0.5, cy - r * 0.3, cx + r * 0.3, cy + r * 0.3],
              200, 320, fill=(168, 236, 150, 255), width=1)
    return img

def draw_bladderwrack():
    rng = random.Random(7115)
    img = new_img(); d = ImageDraw.Draw(img)
    frond_c = (96, 128, 52, 255); rib_c = (70, 98, 38, 255)
    blad_c = (168, 148, 74, 255); blad_hi = (206, 188, 104, 255)
    # flat forking fronds with a midrib, air bladders in pairs along them
    def frond(x, y, ang, ln, depth):
        ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
        d.line([x, y, ex, ey], fill=frond_c, width=3)
        d.line([x, y, ex, ey], fill=rib_c, width=1)
        # a pair of round air bladders flanking the midrib
        mx, my = (x + ex) / 2, (y + ey) / 2
        for sgn in (-1, 1):
            bx = mx + sgn * 2.6 * math.cos(ang); by = my - sgn * 2.6 * math.sin(ang)
            d.ellipse([bx - 1.7, by - 1.7, bx + 1.7, by + 1.7], fill=blad_c,
                      outline=(120, 102, 48, 255))
            d.point((bx - 0.5, by - 0.5), fill=blad_hi)
        if depth <= 0:
            return
        for na in (ang - 0.45, ang + 0.45):
            frond(ex, ey, na, ln * 0.66, depth - 1)
    for x in (9, 17, 24):
        frond(x, 31, rng.uniform(-0.12, 0.12), rng.uniform(8, 10), 2)
    return img

def draw_sea_anemone():
    rng = random.Random(7116)
    img = new_img(); d = ImageDraw.Draw(img)
    # squat pink column base
    d.polygon([(10, 31), (11, 24), (21, 24), (22, 31)], fill=(196, 96, 120, 255),
              outline=(150, 62, 88, 255))
    # oral disc
    d.ellipse([9.6, 21.4, 22.4, 26.2], fill=(222, 130, 152, 255),
              outline=(150, 62, 88, 255))
    # waving stinging tentacles: tapered strokes radiating from the disc
    for _ in range(26):
        a = rng.uniform(-1.35, 1.35)
        ln = rng.uniform(5.5, 9.5)
        bx = 16 + math.sin(a) * 3.4; by = 23.6
        ex = 16 + math.sin(a) * (5.5 + ln * 0.55); ey = by - math.cos(a) * ln
        sway = rng.uniform(-1.6, 1.6)
        d.line([bx, by, (bx + ex) / 2 + sway, (by + ey) / 2, ex, ey],
               fill=(238, 156, 176, 255), width=2)
        d.ellipse([ex - 1.1, ey - 1.1, ex + 1.1, ey + 1.1],
                  fill=(250, 196, 210, 255))
    d.ellipse([14.8, 21.8, 17.2, 24.0], fill=(240, 170, 188, 255))
    return img

def draw_sea_sponge():
    rng = random.Random(7117)
    img = new_img(); d = ImageDraw.Draw(img)
    body = (140, 104, 62, 255); dark = (104, 74, 42, 255); lite = (172, 136, 86, 255)
    # lumpy mound: union of overlapping bumps anchored to the floor
    for (cx, cy, r) in [(10, 27, 5.4), (16, 24.6, 6.6), (22, 27, 5.2),
                        (13, 29, 5.8), (19, 29.4, 6.0)]:
        d.ellipse([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8], fill=body)
    # two upright tube oscula
    for (x, top) in [(13.5, 15.5), (19.5, 17.5)]:
        d.rectangle([x - 1.8, top + 2, x + 1.8, top + 9], fill=body)
        d.ellipse([x - 2.1, top, x + 2.1, top + 3.4], fill=dark)
        d.ellipse([x - 1.2, top + 0.7, x + 1.2, top + 2.7], fill=(52, 36, 20, 255))
    # pores all over the surface
    for _ in range(42):
        x = rng.uniform(5, 27); y = rng.uniform(20, 31)
        r = rng.uniform(0.6, 1.3)
        d.ellipse([x - r, y - r, x + r, y + r], fill=dark)
    # highlight pass on the upper-left bumps
    for _ in range(12):
        x = rng.uniform(6, 20); y = rng.uniform(19, 26)
        d.point((x, y), fill=lite)
    return img

# ------------------------------------------------------------- decorative
def draw_shackle():
    img = new_img(); d = ImageDraw.Draw(img)
    steel = (168, 172, 180, 255); dark = (84, 88, 96, 255); lite = (216, 220, 228, 255)
    # mounting plate at top center (wall mount)
    d.rectangle([12, 2, 20, 6], fill=steel, outline=dark)
    d.point((14, 4), fill=dark); d.point((18, 4), fill=dark)
    # heavy chain: two sagging runs of links down to the cuffs
    def chain(x0, y0, x1, y1, sag):
        n = 7
        for i in range(n + 1):
            t = i / n
            x = x0 + (x1 - x0) * t
            y = y0 + (y1 - y0) * t + sag * math.sin(t * math.pi)
            d.ellipse([x - 1.5, y - 1.1, x + 1.5, y + 1.1], outline=steel, width=1)
    chain(14, 6, 7, 22, 3.4)
    chain(18, 6, 25, 22, 3.4)
    # the pair of cuffs: open rings at the bottom corners
    for cx in (7, 25):
        d.ellipse([cx - 4.4, 20, cx + 4.4, 29.6], outline=steel, width=3)
        d.ellipse([cx - 4.4, 20, cx + 4.4, 29.6], outline=dark, width=1)
        d.rectangle([cx - 1.6, 18.6, cx + 1.6, 21.6], fill=dark)  # hinge/lock block
        d.point((cx - 2.4, 22), fill=lite)
    return img

def draw_gazing_ball():
    img = new_img(); d = ImageDraw.Draw(img)
    cem = (148, 148, 152, 255); cem_d = (104, 104, 108, 255)
    # cement pedestal: cap, tapered column, base
    d.rectangle([10, 19, 22, 21.4], fill=cem, outline=cem_d)
    d.polygon([(12, 21.4), (20, 21.4), (21.4, 29), (10.6, 29)], fill=cem,
              outline=cem_d)
    d.rectangle([9, 29, 23, 31], fill=cem, outline=cem_d)
    # mirrored globe: shaded sphere with a horizon reflection band
    d.ellipse([7, 2, 25, 20], fill=(188, 200, 214, 255), outline=(110, 118, 130, 255))
    d.pieslice([7, 2, 25, 20], 15, 165, fill=(150, 172, 200, 255))
    d.rectangle([7, 12.6, 25, 14.6], fill=(226, 234, 242, 255))
    d.rectangle([7, 14.6, 25, 16.4], fill=(120, 138, 164, 255))
    d.ellipse([7, 15.4, 25, 20], fill=(96, 116, 148, 255))
    # warped bright highlight (the viewer's reflection)
    d.ellipse([10.6, 4.6, 15.4, 9.4], fill=(252, 252, 254, 255))
    d.ellipse([17.6, 6.4, 19.6, 8.8], fill=(238, 244, 250, 255))
    return img

def draw_lighthouse_light():
    img = new_img(); d = ImageDraw.Draw(img)
    metal = (150, 154, 162, 255); dark = (78, 82, 90, 255); lite = (206, 210, 218, 255)
    # heatsink fins: vertical plates fanning behind/above the lamp head
    for k in range(7):
        x = 5 + k * 3.6
        d.rectangle([x, 3.4, x + 1.8, 12], fill=metal, outline=dark)
    d.rectangle([4, 11, 28, 13.4], fill=dark)
    # lamp housing: drum with a bright LED lens facing the viewer
    d.rectangle([8, 13.4, 24, 26], fill=metal, outline=dark)
    d.ellipse([10.4, 15, 21.6, 24.4], fill=(240, 246, 252, 255),
              outline=dark)
    d.ellipse([13, 17.2, 19, 22.2], fill=(255, 252, 214, 255))
    d.ellipse([14.8, 18.8, 17.2, 20.8], fill=(255, 255, 255, 255))
    # side heatsink ribs on the housing
    for y in (14.6, 18.6, 22.6):
        d.line([8, y, 5.6, y], fill=dark, width=2)
        d.line([24, y, 26.4, y], fill=dark, width=2)
    # yoke mount and base
    d.line([11, 26, 9, 29.4], fill=dark, width=2)
    d.line([21, 26, 23, 29.4], fill=dark, width=2)
    d.rectangle([7, 29.4, 25, 31.4], fill=metal, outline=dark)
    return img

def draw_styling_station():
    img = new_img(); d = ImageDraw.Draw(img)
    frame = (120, 88, 58, 255); frame_d = (84, 60, 38, 255)
    glass = (196, 220, 228, 255); glass_hi = (240, 250, 252, 255)
    body = (226, 224, 220, 255); body_d = (150, 148, 144, 255)
    # mirror mounted above: framed, with a diagonal sheen
    d.rectangle([8, 1, 24, 15], fill=frame, outline=frame_d)
    d.rectangle([10, 3, 22, 13], fill=glass)
    d.polygon([(11, 13), (17, 3), (20, 3), (14, 13)], fill=glass_hi)
    # countertop slab
    d.rectangle([4, 16.4, 28, 18.4], fill=body, outline=body_d)
    # cabinet body with cubbies: two open shelves + a drawer line
    d.rectangle([5.4, 18.4, 26.6, 30.6], fill=body, outline=body_d)
    d.line([16, 18.4, 16, 30.6], fill=body_d, width=1)
    d.line([5.4, 24.4, 26.6, 24.4], fill=body_d, width=1)
    d.point((14.6, 21.4), fill=frame_d); d.point((17.4, 21.4), fill=frame_d)
    # stylist's kit in the cubbies: bottle silhouettes
    for (x, h, c) in [(8, 4.4, (196, 90, 90, 255)), (11.4, 3.4, (90, 130, 196, 255)),
                      (20, 4.2, (96, 170, 110, 255)), (23.4, 3.2, (210, 170, 70, 255))]:
        d.rectangle([x, 24.4 - h + 6, x + 2, 30.4], fill=c)
        d.rectangle([x + 0.6, 24.4 - h + 4.6, x + 1.4, 24.4 - h + 6], fill=body_d)
    # a brush and scissors resting on the counter
    d.line([19, 17.4, 24.4, 17.4], fill=frame_d, width=1)
    d.line([6, 17.4, 9.6, 16.2], fill=(90, 90, 96, 255), width=1)
    return img

def draw_flagpole(wooden):
    img = new_img(); d = ImageDraw.Draw(img)
    if wooden:
        pole = (128, 92, 54, 255); dark = (88, 60, 34, 255); lite = (168, 128, 82, 255)
    else:
        pole = (172, 176, 184, 255); dark = (96, 100, 108, 255); lite = (224, 228, 234, 255)
    # weighted stand: broad round base
    d.ellipse([7, 28.2, 25, 31.6], fill=pole, outline=dark)
    d.ellipse([10.4, 27.2, 21.6, 29.8], fill=pole, outline=dark)
    # the 2.5 m pole
    d.rectangle([15.1, 4.4, 16.9, 28.4], fill=pole, outline=dark)
    d.line([15.7, 5, 15.7, 28], fill=lite, width=1)
    # ball finial
    d.ellipse([14.2, 1.2, 17.8, 4.8], fill=pole, outline=dark)
    d.point((15.2, 2.4), fill=lite)
    # cleat with a wrap of halyard rope halfway down
    d.rectangle([16.9, 17, 18.6, 18.2], fill=dark)
    for i in range(3):
        d.line([15.2, 18.6 + i * 0.9, 16.8, 18.6 + i * 0.9],
               fill=(196, 178, 140, 255), width=1)
    if wooden:  # wood grain streaks
        for (x, y0, y1) in [(15.9, 8, 15), (16.3, 20, 27)]:
            d.line([x, y0, x, y1], fill=dark, width=1)
    return img

# ------------------------------------------------------------------ jobs
JOBS = [
    ("f_region_plain", draw_region_plain()),
    ("f_region_flower", draw_region_flower()),
    ("f_region_weed", draw_region_weed()),
    ("f_region_water_plant", draw_region_water_plant()),
    ("f_region_forest", draw_region_forest(False)),
    ("f_region_forest_dense", draw_region_forest(True)),
    ("f_region_forest_water", draw_region_forest_water()),
    ("f_region_flower_decorative", draw_region_flower_decorative()),
    ("f_region_lake_bed", draw_region_lake_bed()),
    ("f_region_ocean_bed", draw_region_ocean_bed()),
    ("f_algae_red", draw_algae_red()),
    ("f_algae_green", draw_algae_green()),
    ("f_kelp_sugar", draw_kelp_sugar()),
    ("f_sea_lettuce", draw_sea_lettuce()),
    ("f_bladderwrack", draw_bladderwrack()),
    ("f_sea_anemone", draw_sea_anemone()),
    ("f_sea_sponge", draw_sea_sponge()),
    ("f_shackle", draw_shackle()),
    ("f_gazing_ball", draw_gazing_ball()),
    ("f_lighthouse_light", draw_lighthouse_light()),
    ("f_styling_station", draw_styling_station()),
    ("f_indoor_flagpole", draw_flagpole(False)),
    ("f_indoor_flagpole_wooden", draw_flagpole(True)),
]

CREDITS = ("Drawn from the furniture's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": tid, "bg": ""}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

CELL = 64
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (32, 36, 40, 255))
for i, (tid, _img) in enumerate(JOBS):
    t = tile_img(tid).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_furniture_shores.png"))
print("wrote preview_furniture_shores.png", sheet.size)
