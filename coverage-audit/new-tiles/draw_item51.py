#!/usr/bin/env python3
"""ITEM51: eggs + egg dishes -- CDDA Voxel coverage audit.

All 29 items come from items/comestibles/egg.json.  Raw eggs are
drawn as the eggs they are, dishes as themselves:

  bird eggs in a twig nest cup, told apart by shell and size:
    robin -- distinctive light blue, unspotted;  sparrow -- tiny,
    cream with heavy brown speckle;  waxwing -- pale blue-gray,
    dark speckles toward the blunt end;  woodpecker -- small,
    glossy pure white;  turkey -- large, tan, brown-speckled;
    domestic turkey -- larger and paler, faint speckle.
  leathery reptile eggs on soil/sand:
    five-lined skink -- small cream ovals;  worm snake -- small
    elongated thin-shelled whites;  painted turtle -- round white
    spheres;  snapping turtle -- larger round white spheres.
  frog eggs as gelatinous masses:
    spade frog -- short strands of tightly clustered black eggs in
    viscous jelly;  wood frog -- a softball-sized mass like soft
    transparent grapes, dark "seeds" inside growing legs;
    shift bigfrog -- golfball-sized eggs clinging by sticky
    membranes;  strange bigfrog -- a thick flotilla floating in a
    roughly circular mass on water;  vocal bigfrog -- a thick
    clear sack of orange-like spheres.
  fish roe as glistening grain masses:
    warmouth -- fine deep red-orange grains;  white catfish --
    larger amber grains.

  dishes: egg salad (bowl of chopped hard-boiled egg), fried egg
  sandwich (sunny eggs between bread, yolk dripping), fried eggs
  (plate, two sunny-side up, runny yolks, crisp lacy edges),
  menemen (dark skillet of tomato, green pepper and egg curds with
  spice specks), omelet (folded, cheese oozing at the seam),
  supreme omelet (overstuffed -- cheese, vegetables, meat spilling),
  vegetable omelet (cheese and vegetables, no meat), pickled egg
  (brine jar, whole peeled eggs), rehydrated eggs (a shapeless
  pale-yellow mound -- they have lost their complete shape),
  scrambled eggs (fluffy curds on a plate), seasoned scrambled
  (herb and spice flecks), seasoned fried eggs (seasoning flecked
  over the yolks).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "egg_robin",
    "egg_salad",
    "egg_shift_bigfrog",
    "egg_skink_fivelined",
    "egg_spade_frog",
    "egg_sparrow",
    "egg_strange_bigfrog",
    "egg_turkey",
    "egg_turkey_domestic",
    "egg_turtle_painted",
    "egg_turtle_snapping_common",
    "egg_vocal_bigfrog",
    "egg_warmouth",
    "egg_waxwing",
    "egg_white_catfish",
    "egg_wood_frog",
    "egg_woodpecker",
    "egg_wormsnake",
    "fried_egg_sandwich",
    "fried_eggs",
    "menemen",
    "omelet",
    "pickled_egg",
    "rehydrated_eggs",
    "scrambled_eggs",
    "seasoned_fried_eggs",
    "seasoned_scrambled_eggs",
    "supreme_omelet",
    "vegetable_omelet",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

WHITE = (244, 242, 236, 255)
CREAM = (238, 228, 198, 255)
YOLK = (246, 176, 32, 255)
YOLK_DEEP = (232, 138, 20, 255)
EGG_COOKED = (252, 246, 224, 255)
TWIG = (110, 80, 48, 255)
TWIG_DARK = (78, 56, 32, 255)
SOIL = (104, 84, 58, 255)
SAND = (196, 172, 122, 255)
LEAF = (86, 128, 62, 255)
WATER = (86, 138, 178, 255)
JELLY = (206, 226, 230, 255)
JELLY_EDGE = (150, 178, 188, 255)
EMBRYO = (38, 36, 40, 255)
PLATE = (226, 228, 232, 255)
PLATE_EDGE = (168, 172, 180, 255)
PAN = (44, 44, 50, 255)
CHEESE = (250, 178, 44, 255)
BREAD = (226, 182, 116, 255)
BREAD_CRUST = (172, 118, 58, 255)
TOMATO = (204, 62, 40, 255)
PEPPER_GREEN = (62, 148, 62, 255)
MEAT = (146, 88, 58, 255)
HERB = (58, 122, 52, 255)
SPICE_RED = (188, 52, 36, 255)
PEPPER_BLACK = (40, 36, 34, 255)
BRINE = (226, 214, 168, 255)
GLASS = (196, 220, 224, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------- eggs

def ovoid(d, cx, cy, w, h, fill, outline=None):
    """An egg: blunt bottom, narrower top, as a polygon."""
    pts = []
    for i in range(25):
        a = 2 * math.pi * i / 24
        sy = -math.cos(a)          # -1 top .. +1 bottom
        sx = math.sin(a)
        taper = 1.0 - 0.20 * max(0.0, -sy) + 0.06 * max(0.0, sy)
        pts.append((cx + sx * (w / 2) * taper, cy + sy * (h / 2)))
    d.polygon(pts, fill=fill, outline=outline)


def speckles(d, rng, cx, cy, w, h, col, n, blunt_bias=False):
    for _ in range(n):
        a = rng.uniform(0, 2 * math.pi)
        r = math.sqrt(rng.random()) * 0.78
        x = cx + math.cos(a) * r * w / 2
        y = cy + math.sin(a) * r * h / 2
        if blunt_bias and y < cy - h * 0.1 and rng.random() < 0.6:
            continue  # waxwing: speckle crowds the blunt (bottom) end
        d.point((round(x), round(y)), fill=col)
        if rng.random() < 0.3:
            d.point((round(x) + 1, round(y)), fill=col)


def nest(d, rng, cx, cy, w=24):
    """Twig nest cup under a clutch."""
    for i in range(7):
        y = cy + i % 3
        x0 = cx - w // 2 + (i % 2) * 2
        x1 = cx + w // 2 - ((i + 1) % 2) * 2
        d.arc([x0, y - 5, x1, y + 6], start=15, end=165,
              fill=TWIG if i % 2 else TWIG_DARK, width=2)
    for _ in range(5):  # stray twig ends
        x = rng.randint(cx - w // 2, cx + w // 2)
        d.line([x, cy + 3, x + rng.randint(-2, 2), cy + 5], fill=TWIG_DARK,
               width=1)


def draw_bird(d, rng, shell, spec):
    nest(d, rng, 16, 21)
    eggs = spec["eggs"]  # (dx, dy, w, h)
    for dx, dy, w, h in eggs:
        ovoid(d, 16 + dx, 15 + dy, w, h, shell, outline=shade(shell, 0.72))
        d.point((16 + dx - w // 4, 15 + dy - h // 4),
                fill=shade(shell, 1.06) if shell[0] < 250 else WHITE)
    if spec.get("speckle"):
        col, n, bias = spec["speckle"]
        for dx, dy, w, h in eggs:
            speckles(d, rng, 16 + dx, 15 + dy, w, h, col, n, bias)


def draw_leathery(d, rng, spec):
    # soil/sand mound with leaf litter flecks
    d.ellipse([3, 19, 29, 29], fill=spec["ground"])
    for _ in range(10):
        x, y = rng.randint(5, 27), rng.randint(21, 27)
        d.point((x, y), fill=shade(spec["ground"], rng.uniform(0.8, 1.15)))
    for dx, dy, w, h in spec["eggs"]:
        if spec.get("round"):
            d.ellipse([16 + dx - w / 2, 15 + dy - h / 2,
                       16 + dx + w / 2, 15 + dy + h / 2],
                      fill=spec["shell"], outline=shade(spec["shell"], 0.78))
        else:
            ovoid(d, 16 + dx, 15 + dy, w, h, spec["shell"],
                  outline=shade(spec["shell"], 0.78))
        d.point((16 + dx - 2, 15 + dy - 2), fill=WHITE)


def jelly_ball(d, cx, cy, r, centre, centre_r=2, legs=False):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=JELLY,
              outline=JELLY_EDGE)
    d.ellipse([cx - centre_r, cy - centre_r, cx + centre_r,
               cy + centre_r], fill=centre)
    d.point((cx - r // 2, cy - r // 2), fill=WHITE)
    if legs:  # wood frog: the "seeds" are growing legs
        d.line([cx + centre_r, cy + 1, cx + centre_r + 2, cy + 3],
               fill=centre, width=1)
        d.line([cx - centre_r, cy + 1, cx - centre_r - 2, cy + 3],
               fill=centre, width=1)


def draw_frog(d, rng, kind):
    if kind == "strands":  # spade frog: short strands of black eggs
        for sx, sy, n in ((10, 11, 4), (17, 9, 5), (22, 13, 4),
                          (13, 19, 5), (20, 20, 4)):
            for i in range(n):
                jelly_ball(d, sx + i * 2, sy + (i % 2) * 2 - 1, 2,
                           EMBRYO, 1)
        for x in range(7, 27, 4):  # viscous strands linking the clusters
            d.point((x, 16), fill=JELLY_EDGE)
    elif kind == "mass":  # wood frog: transparent grapes, seeds with legs
        balls = [(11, 12), (16, 10), (21, 12), (9, 17), (14, 16),
                 (19, 16), (24, 17), (12, 22), (17, 21), (22, 22)]
        for i, (x, y) in enumerate(balls):
            jelly_ball(d, x, y, 3, EMBRYO, 2, legs=(i % 3 == 0))
    elif kind == "big_sticky":  # shift bigfrog: golfball eggs, membranes
        balls = [(11, 12), (17, 10), (22, 13), (13, 18), (19, 18),
                 (24, 19), (16, 24)]
        for x, y in balls:
            jelly_ball(d, x, y, 4, EMBRYO, 2)
        for x, y in ((14, 14), (20, 15), (16, 21)):  # sticky membrane links
            d.line([x, y, x + 2, y + 1], fill=JELLY_EDGE, width=2)
    elif kind == "big_raft":  # strange bigfrog: circular flotilla on water
        d.ellipse([4, 7, 28, 27], fill=WATER)
        d.ellipse([4, 7, 28, 27], outline=shade(WATER, 0.7))
        balls = [(12, 13), (17, 11), (21, 14), (14, 18), (19, 19),
                 (23, 20), (16, 23)]
        for x, y in balls:
            jelly_ball(d, x, y, 4, EMBRYO, 2)
        d.point((8, 10), fill=shade(WATER, 1.25))
        d.point((25, 24), fill=shade(WATER, 1.25))
    elif kind == "big_sack":  # vocal bigfrog: clear sack of oranges
        d.ellipse([6, 7, 27, 27], fill=JELLY, outline=JELLY_EDGE)
        for x, y in ((12, 13), (18, 11), (22, 15), (13, 19), (19, 19),
                     (23, 21), (16, 24)):
            d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=(248, 186, 84, 255),
                      outline=(224, 156, 52, 255))
            d.point((x - 1, y - 1), fill=(252, 214, 140, 255))


def draw_roe(d, rng, grain, grain_dark, grain_light, r):
    """A glistening mass of fish eggs (grains heaped in a pile)."""
    pts = []
    for cy, half in ((22, 10), (18, 8), (14, 5)):
        for dx in range(-half, half + 1, 2 * r):
            pts.append((16 + dx + rng.randint(-1, 1), cy))
    for x, y in pts:
        d.ellipse([x - r, y - r, x + r, y + r], fill=grain,
                  outline=grain_dark)
        d.point((x - 1, y - 1), fill=grain_light)


BIRDS = {
    "egg_robin": dict(
        shell=(146, 196, 222, 255),
        eggs=[(-5, 0, 9, 11), (4, -1, 9, 11), (0, 3, 9, 11)]),
    "egg_sparrow": dict(
        shell=(232, 222, 204, 255),
        eggs=[(-6, 1, 7, 8), (0, -1, 7, 8), (6, 1, 7, 8), (-3, 3, 7, 8),
              (3, 3, 7, 8)],
        speckle=((150, 108, 72, 255), 14, False)),
    "egg_waxwing": dict(
        shell=(178, 192, 204, 255),
        eggs=[(-5, 0, 8, 10), (4, 0, 8, 10), (0, 3, 8, 10)],
        speckle=((70, 62, 66, 255), 9, True)),
    "egg_woodpecker": dict(
        shell=(250, 250, 246, 255),
        eggs=[(-5, 0, 8, 9), (4, 0, 8, 9), (0, 3, 8, 9)]),
    "egg_turkey": dict(
        shell=(216, 190, 150, 255),
        eggs=[(-6, 0, 11, 13), (5, 0, 11, 13)],
        speckle=((146, 104, 62, 255), 12, False)),
    "egg_turkey_domestic": dict(
        shell=(236, 224, 196, 255),
        eggs=[(-6, 0, 12, 14), (6, 0, 12, 14)],
        speckle=((178, 152, 110, 255), 6, False)),
}

LEATHERY = {
    "egg_skink_fivelined": dict(
        shell=(240, 234, 216, 255), ground=SOIL,
        eggs=[(-6, 1, 7, 9), (0, -1, 7, 9), (6, 1, 7, 9), (-3, 3, 7, 9),
              (3, 3, 7, 9)]),
    "egg_wormsnake": dict(
        shell=(246, 242, 230, 255), ground=SOIL,
        eggs=[(-5, 0, 6, 11), (2, 0, 6, 11), (7, 1, 6, 10)]),
    "egg_turtle_painted": dict(
        shell=(244, 240, 230, 255), ground=SAND, round=True,
        eggs=[(-6, 0, 10, 10), (5, -1, 10, 10), (0, 4, 10, 10)]),
    "egg_turtle_snapping_common": dict(
        shell=(240, 236, 226, 255), ground=SAND, round=True,
        eggs=[(-7, 0, 13, 13), (6, 0, 13, 13)]),
}

FROGS = {
    "egg_spade_frog": "strands",
    "egg_wood_frog": "mass",
    "egg_shift_bigfrog": "big_sticky",
    "egg_strange_bigfrog": "big_raft",
    "egg_vocal_bigfrog": "big_sack",
}


# ----------------------------------------------------------------- dishes

def plate(d, cx=16, cy=17, rx=13, ry=10):
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=PLATE,
              outline=PLATE_EDGE)
    d.ellipse([cx - rx + 3, cy - ry + 2, cx + rx - 3, cy + ry - 2],
              outline=shade(PLATE, 0.92))


def fried_egg(d, cx, cy, r=6, seasoned=False, rng=None):
    """One sunny-side up egg: white with crisp edge, runny yolk."""
    pts = []
    for i in range(17):
        a = 2 * math.pi * i / 16
        rr = r + ((i * 7) % 3 - 1)  # irregular, slightly crisp outline
        pts.append((cx + rr * math.cos(a), cy + rr * 0.78 * math.sin(a)))
    d.polygon(pts, fill=EGG_COOKED, outline=(216, 188, 128, 255))
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=YOLK,
              outline=YOLK_DEEP)
    d.point((cx - 1, cy - 1), fill=(252, 208, 96, 255))
    if seasoned and rng is not None:
        for _ in range(9):
            x = cx + rng.randint(-r, r)
            y = cy + rng.randint(-r // 2, r // 2)
            d.point((x, y), fill=rng.choice([HERB, PEPPER_BLACK, SPICE_RED]))


def scrambled_pile(d, rng, cx, cy, seasoned=False):
    for i in range(9):
        x = cx + rng.randint(-8, 8)
        y = cy + rng.randint(-4, 4)
        col = (250, 208, 92, 255) if i % 3 else (246, 190, 62, 255)
        d.ellipse([x - 4, y - 3, x + 4, y + 3], fill=col,
                  outline=(226, 168, 48, 255))
        d.point((x - 1, y - 1), fill=(252, 226, 140, 255))
    if seasoned:
        for _ in range(14):
            x = cx + rng.randint(-9, 9)
            y = cy + rng.randint(-5, 5)
            d.point((x, y), fill=rng.choice([HERB, SPICE_RED,
                                              PEPPER_BLACK]))


def omelet_body(d, rng, cx, cy, w=22, h=13, fillings=()):
    """A folded omelet: half-moon with a seam, fillings at the seam."""
    d.ellipse([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
              fill=(250, 202, 84, 255), outline=(228, 170, 52, 255))
    d.arc([cx - w // 2 + 2, cy - h // 2 + 1, cx + w // 2 - 2,
           cy + h // 2 - 3], start=0, end=180, fill=(228, 170, 52, 255),
          width=1)  # fold seam
    d.point((cx - 5, cy - 3), fill=(252, 222, 130, 255))
    d.point((cx + 4, cy - 2), fill=(252, 222, 130, 255))
    for i, f in enumerate(fillings):
        x = cx - 6 + i * 4
        y = cy + h // 2 - 2
        if f == "cheese":
            d.polygon([(x, y), (x + 3, y), (x + 1, y + 3)], fill=CHEESE)
        elif f == "pepper":
            d.rectangle([x, y - 1, x + 2, y + 1], fill=PEPPER_GREEN)
        elif f == "tomato":
            d.rectangle([x, y - 1, x + 2, y + 1], fill=TOMATO)
        elif f == "meat":
            d.rectangle([x, y - 1, x + 3, y + 1], fill=MEAT)


def draw_dish(d, rng, sid):
    if sid == "egg_salad":
        # bowl of chopped hard-boiled egg in mayonnaise
        d.ellipse([5, 10, 27, 26], fill=(120, 140, 160, 255),
                  outline=(84, 100, 118, 255))
        d.ellipse([6, 9, 26, 19], fill=(238, 228, 190, 255))
        for _ in range(12):  # chopped white/yolk chunks
            x, y = rng.randint(9, 23), rng.randint(11, 17)
            col = WHITE if rng.random() < 0.55 else (248, 196, 84, 255)
            d.rectangle([x, y, x + 2, y + 1], fill=col)
        for _ in range(4):  # celery/herb flecks
            d.point((rng.randint(9, 23), rng.randint(11, 17)), fill=HERB)
        d.line([5, 22, 27, 22], fill=(84, 100, 118, 255), width=1)
    elif sid == "fried_eggs":
        plate(d)
        fried_egg(d, 12, 15, 6)
        fried_egg(d, 20, 19, 6)
    elif sid == "seasoned_fried_eggs":
        plate(d)
        fried_egg(d, 12, 15, 6, seasoned=True, rng=rng)
        fried_egg(d, 20, 19, 6, seasoned=True, rng=rng)
    elif sid == "fried_egg_sandwich":
        # bottom bread, fried egg with dripping yolk, top bread askew
        d.polygon([(7, 20), (25, 20), (26, 26), (6, 26)], fill=BREAD,
                  outline=BREAD_CRUST)
        d.ellipse([8, 15, 24, 22], fill=EGG_COOKED,
                  outline=(216, 188, 128, 255))
        d.ellipse([13, 15, 19, 20], fill=YOLK, outline=YOLK_DEEP)
        d.line([16, 20, 16, 24], fill=YOLK, width=2)  # runny drip
        d.polygon([(6, 8), (24, 6), (26, 13), (8, 15)], fill=BREAD,
                  outline=BREAD_CRUST)
        d.line([9, 10, 22, 8], fill=shade(BREAD, 1.08), width=1)
    elif sid == "menemen":
        # dark skillet: tomato-pepper sauce with egg curds folded in
        d.line([24, 8, 30, 3], fill=PAN, width=3)  # handle
        d.ellipse([3, 5, 27, 29], fill=PAN)
        d.ellipse([5, 7, 25, 27], fill=TOMATO)
        for _ in range(8):  # egg curds in the sauce
            x, y = rng.randint(8, 22), rng.randint(10, 24)
            d.ellipse([x - 3, y - 2, x + 3, y + 2],
                      fill=(250, 206, 92, 255), outline=(228, 170, 52, 255))
        for _ in range(6):  # green pepper strips
            x, y = rng.randint(7, 22), rng.randint(9, 24)
            d.line([x, y, x + 3, y + 1], fill=PEPPER_GREEN, width=2)
        for _ in range(12):  # black + red pepper specks
            d.point((rng.randint(7, 24), rng.randint(9, 25)),
                    fill=rng.choice([PEPPER_BLACK, (150, 30, 22, 255)]))
    elif sid == "omelet":
        plate(d)
        omelet_body(d, rng, 16, 17, fillings=("cheese", "cheese"))
    elif sid == "vegetable_omelet":
        plate(d)
        omelet_body(d, rng, 16, 17,
                    fillings=("cheese", "pepper", "tomato", "pepper"))
    elif sid == "supreme_omelet":
        plate(d)
        omelet_body(d, rng, 16, 17, w=24, h=15,
                    fillings=("meat", "cheese", "pepper", "meat", "tomato",
                              "cheese"))
        d.point((10, 10), fill=PEPPER_GREEN)  # veg flecks on top
        d.point((20, 9), fill=TOMATO)
    elif sid == "pickled_egg":
        # glass brine jar with whole peeled eggs and a lid
        d.rectangle([9, 4, 23, 7], fill=(150, 62, 52, 255),
                    outline=(110, 42, 36, 255))  # lid
        d.rectangle([8, 8, 24, 28], fill=BRINE, outline=GLASS)
        for x, y in ((13, 13), (19, 12), (16, 19), (12, 23), (20, 23)):
            d.ellipse([x - 3, y - 4, x + 3, y + 4], fill=(250, 246, 232, 255),
                      outline=(214, 202, 160, 255))
        d.line([10, 9, 10, 27], fill=(240, 250, 250, 255), width=1)
        for _ in range(5):  # spice motes in the brine
            d.point((rng.randint(10, 22), rng.randint(10, 26)),
                    fill=SPICE_RED)
    elif sid == "rehydrated_eggs":
        # a bowl of shapeless pale egg -- it has lost its shape
        d.ellipse([5, 13, 27, 27], fill=(150, 116, 84, 255),
                  outline=(110, 84, 58, 255))
        pts = [(8, 15), (13, 12), (19, 13), (24, 15), (23, 19), (17, 18),
               (11, 19), (8, 18)]
        d.polygon(pts, fill=(240, 208, 118, 255))
        for _ in range(7):  # lumpy, uneven surface
            x, y = rng.randint(10, 22), rng.randint(13, 18)
            d.ellipse([x - 2, y - 1, x + 2, y + 2],
                      fill=(234, 196, 102, 255))
        d.point((12, 14), fill=(248, 224, 156, 255))
    elif sid == "scrambled_eggs":
        plate(d)
        scrambled_pile(d, rng, 16, 17)
    elif sid == "seasoned_scrambled_eggs":
        plate(d)
        scrambled_pile(d, rng, 16, 17, seasoned=True)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in BIRDS:
        draw_bird(d, rng, BIRDS[sid]["shell"], BIRDS[sid])
    elif sid in LEATHERY:
        draw_leathery(d, rng, LEATHERY[sid])
    elif sid in FROGS:
        draw_frog(d, rng, FROGS[sid])
    elif sid == "egg_warmouth":
        draw_roe(d, rng, (216, 92, 52, 255), (168, 58, 32, 255),
                 (248, 160, 110, 255), 2)
    elif sid == "egg_white_catfish":
        draw_roe(d, rng, (232, 168, 74, 255), (186, 124, 42, 255),
                 (252, 208, 130, 255), 3)
    else:
        draw_dish(d, rng, sid)
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
    # contact-sheet preview: 6 cols x 5 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item51.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item51.png {prev.size}")


if __name__ == "__main__":
    main()
