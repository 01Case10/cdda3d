#!/usr/bin/env python3
"""Batch ITEM07 (Phase 2g.4 scope extension): eggs, first set.

52 ids from data/json/items/comestibles/egg.json (authoritative
list: /tmp/item_batches/07.txt). One parameterized renderer per
family, driven by each entry's resolved data (copy-from chains
resolved by hand against egg.json):

- Bird eggs (copy-from egg_chicken: brown, 50 ml, sym 'o'):
  shell color/speckle/size per species from the real bird --
  blue-green speckled blue jay/crow/raven, tiny white hummingbird
  pair (1 ml), large white goose eggs, olive pheasant, pear-
  shaped speckled guineafowl, chalky elongated cormorant, etc.
- Fish roe (copy-from egg_fish: red, 25 ml): bead mounds tinted
  per species; perch roe as its real gelatinous ribbon; mutant
  salmon roe as the egg_fish_mutant abstract describes -- one
  large half-translucent glob with a fry inside; cured/curing
  roe as glossy dark 'caviar' beads / beads brining in a tub.
- Frog egg clusters (copy-from egg_proxy_frog -> egg_fish):
  gelatinous pale beads with dark embryos, arranged per the
  entry descriptions -- bullfrog surface film disc, toad
  strands, freedom toad spiral on vegetation, mink frog polar
  ball (dark tops, white bottoms), pickerel bicolor glob, etc.
- Giant frog eggs (copy-from egg_proxy_bigfrog: 1 L, softball-
  sized, transparent): a few huge gel globes, strand/spiral/
  caviar-cluster/clump per description.
- egg_moth_fly (copy-from egg_insect: white): tiny grain raft.
- Prepared: boiled egg (white, in shell, cracked), deluxe
  fried eggs (sunny-side up, runny yolks, crisp edges, garnish)
  and the two egg sandwiches.

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
(gel, roe) faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item07.png.
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
IDS = [l.strip() for l in open("/tmp/item_batches/07.txt") if l.strip()]
assert len(IDS) == 52, len(IDS)


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)


def egg_mask(cx, cy, rx, ry, asym=0.18):
    """Egg silhouette (wider below centre) as an 'L' mask image."""
    m = Image.new("L", (32, 32), 0)
    d = ImageDraw.Draw(m)
    pts = []
    n = 60
    for i in range(n):
        t = 2 * math.pi * i / n
        x = cx + rx * math.sin(t) * (1 + asym * math.cos(t))
        y = cy + ry * math.cos(t)
        pts.append((x, y))
    d.polygon(pts, fill=255)
    return m


def clip_paste(img, layer, mask):
    """Paste layer onto img only where mask is set (layer alpha kept)."""
    a = layer.getchannel("A")
    a = ImageChops.multiply(a, mask)
    layer.putalpha(a)
    img.alpha_composite(layer)


def draw_egg(img, rng, cx, cy, rx, ry, base, speckle=None, n_speck=0,
             blotch=None, n_blotch=0, asym=0.18, crack=False):
    mask = egg_mask(cx, cy, rx, ry, asym)
    layer = Image.new("RGBA", (32, 32), base)
    img.paste(layer, (0, 0), mask)
    d = ImageDraw.Draw(img)
    # outline: polygon edge in darkened base
    pts = []
    n = 60
    for i in range(n + 1):
        t = 2 * math.pi * (i % n) / n
        pts.append((cx + rx * math.sin(t) * (1 + asym * math.cos(t)),
                    cy + ry * math.cos(t)))
    d.line(pts, fill=shade(base, 0.62), width=1)
    # bottom shading band + top-left highlight, clipped to shell
    ov = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    od.ellipse([cx - rx, cy + ry * 0.35, cx + rx, cy + ry * 1.05],
               fill=shade(base, 0.86))
    od.ellipse([cx - rx * 0.62, cy - ry * 0.78, cx - rx * 0.05,
                cy - ry * 0.28], fill=shade(base, 1.16))
    clip_paste(img, ov, mask)
    mp = mask.load()
    if speckle and n_speck:
        for _ in range(n_speck):
            for _try in range(12):
                x = rng.uniform(cx - rx, cx + rx)
                y = rng.uniform(cy - ry, cy + ry)
                if mp[int(y), int(x)]:
                    r = rng.uniform(0.6, 1.2)
                    d.ellipse([x - r, y - r, x + r, y + r], fill=speckle)
                    break
    if blotch and n_blotch:
        for _ in range(n_blotch):
            for _try in range(12):
                x = rng.uniform(cx - rx * 0.8, cx + rx * 0.8)
                y = rng.uniform(cy - ry * 0.8, cy + ry * 0.8)
                if mp[int(y), int(x)]:
                    r = rng.uniform(1.2, 2.4)
                    d.ellipse([x - r, y - r * 0.7, x + r, y + r * 0.7],
                              fill=blotch)
                    break
    if crack:
        pts = []
        x = cx - rx * 0.55
        while x < cx + rx * 0.55:
            pts.append((x, cy + rng.uniform(-1.2, 1.2)))
            x += 2.2
        d.line(pts, fill=(150, 148, 140, 255), width=1)


def bead(d, x, y, r, base, hi=True):
    d.ellipse([x - r, y - r, x + r, y + r], fill=base,
              outline=shade(base, 0.68))
    if hi and r >= 1.6:
        d.point((int(x - r * 0.35), int(y - r * 0.35)),
                fill=shade(base, 1.4))


def draw_roe(img, rng, base, bead_r=2.2, rows=(4, 6, 7, 6, 4)):
    d = ImageDraw.Draw(img)
    cy = 13.0
    for ri, cnt in enumerate(rows):
        y = cy + ri * (bead_r * 1.55)
        for i in range(cnt):
            x = 16 + (i - (cnt - 1) / 2) * (bead_r * 1.8) \
                + rng.uniform(-0.7, 0.7)
            c = shade(base, rng.uniform(0.88, 1.1))
            bead(d, x, y + rng.uniform(-0.5, 0.5), bead_r, c)


def draw_roe_ribbon(img, rng, base):
    d = ImageDraw.Draw(img)
    for row in range(3):
        y0 = 11 + row * 5.5
        x = 5.0
        while x < 28:
            y = y0 + (2.2 if int(x) % 8 < 4 else -0.5)
            bead(d, x, y, 2.0, shade(base, rng.uniform(0.9, 1.08)))
            x += 3.1


GEL = (206, 222, 214, 255)
GEL_RIM = (148, 172, 164, 255)
EMBRYO = (42, 38, 34, 255)


def gel_bead(d, x, y, r, embryo_frac=0.38, top=None, bottom=None):
    if top is not None:  # bicolor: dark top / pale bottom
        d.ellipse([x - r, y - r, x + r, y + r], fill=bottom or GEL,
                  outline=GEL_RIM)
        d.pieslice([x - r, y - r, x + r, y + r], 180, 360, fill=top)
        d.ellipse([x - r, y - r, x + r, y + r], outline=GEL_RIM)
    else:
        d.ellipse([x - r, y - r, x + r, y + r], fill=GEL, outline=GEL_RIM)
    er = max(0.7, r * embryo_frac)
    d.ellipse([x - er, y - er, x + er, y + er], fill=EMBRYO)


def frog_positions(style, rng):
    """Return list of (x, y, r) gel beads for a frog-cluster style."""
    out = []
    if style == "disc":  # bullfrog: flat circular surface film
        for ring, cnt, rr in ((0, 1, 0), (1, 6, 4.6), (2, 11, 9.0)):
            for i in range(cnt):
                a = 2 * math.pi * i / max(cnt, 1) + ring * 0.5
                out.append((16 + rr * math.cos(a) * 1.15,
                            16 + rr * math.sin(a) * 0.8, 2.3))
    elif style == "strand":  # fowler's toad: long strands
        for s in (-3.2, 3.2):
            x = 4.5
            while x < 28:
                out.append((x, 16 + s + 2.4 * math.sin(x / 4.5), 2.1))
                x += 3.6
    elif style in ("spiral", "spiral_veg"):
        t = 0.0
        while t < 5.6 * math.pi:
            r = 1.1 * t / math.pi + 1.5
            out.append((16 + r * math.cos(t) * 1.1,
                        16 + r * math.sin(t) * 0.78, 2.0))
            t += 0.85
    elif style == "patch":  # gray frog: small patch
        for _ in range(11):
            a = rng.uniform(0, 2 * math.pi)
            rr = rng.uniform(0, 5.5)
            out.append((16 + rr * math.cos(a) * 1.25,
                        16 + rr * math.sin(a) * 0.8, 2.1))
    elif style == "sheet":  # green frog: thin peppered sheet
        for _ in range(17):
            a = rng.uniform(0, 2 * math.pi)
            rr = math.sqrt(rng.uniform(0, 1)) * 9.5
            out.append((16 + rr * math.cos(a),
                        16 + rr * math.sin(a) * 0.55, 2.0))
    elif style == "dense":  # leopard frog: dense dark, caviar-like
        for row in range(4):
            for i in range(5 - abs(row - 1)):
                out.append((9.5 + i * 3.4 + (row % 2) * 1.6,
                            10.5 + row * 3.6, 2.2))
    elif style == "ball_polar":  # mink frog: tight spherical polar ball
        for row, cnt in enumerate((3, 4, 4, 3)):
            for i in range(cnt):
                out.append((16 + (i - (cnt - 1) / 2) * 4.4,
                            10.5 + row * 4.2, 2.6))
    elif style == "sparse_veg":  # spring peeper: few eggs on vegetation
        for (x, y) in ((11, 12), (15, 10), (19, 13), (13, 17), (18, 18),
                       (22, 11)):
            out.append((x, y, 2.1))
    elif style == "glob_bicolor":  # pickerel: globular bicolor mass
        for row, cnt in enumerate((3, 4, 3)):
            for i in range(cnt):
                out.append((16 + (i - (cnt - 1) / 2) * 4.6,
                            11.5 + row * 4.6, 2.9))
    elif style == "bigclump":  # proxy bigfrog: huge globes clump
        for (x, y) in ((11, 12), (21, 11), (16, 19), (10, 21), (22, 20)):
            out.append((x, y, 5.4))
    elif style == "bigstrand":  # foul bigfrog: buoy-line strand
        x = 5.0
        while x < 29:
            out.append((x, 16 + 3.0 * math.sin(x / 5.0), 4.9))
            x += 7.6
    elif style == "bigspiral":  # odd bigfrog: spiral strand
        t = 0.0
        while t < 3.4 * math.pi:
            r = 1.9 * t / math.pi + 2.0
            out.append((16 + r * math.cos(t), 16 + r * math.sin(t) * 0.8,
                        4.6))
            t += 1.5
    elif style == "bigcaviar":  # pattern bigfrog: gigantic caviar
        for row, cnt in enumerate((2, 3, 2)):
            for i in range(cnt):
                out.append((16 + (i - (cnt - 1) / 2) * 8.2,
                            10.5 + row * 7.6, 4.5))
    return out


def draw_frog(img, rng, style):
    d = ImageDraw.Draw(img)
    if style in ("spiral_veg", "sparse_veg"):
        stem = (74, 116, 62, 255)
        d.line([(16, 30), (15, 20), (17, 10), (16, 3)], fill=stem, width=2)
        d.ellipse([8, 6, 15, 10], fill=(96, 140, 74, 255),
                  outline=shade((96, 140, 74), 0.7))
    polar = style in ("ball_polar", "glob_bicolor")
    for (x, y, r) in frog_positions(style, rng):
        if polar:
            gel_bead(d, x, y, r, top=(96, 76, 52, 255),
                     bottom=(228, 216, 178, 255))
        elif style == "dense":
            gel_bead(d, x, y, r, embryo_frac=0.62)
        elif style.startswith("big"):
            gel_bead(d, x, y, r, embryo_frac=0.2)
        else:
            gel_bead(d, x, y, r)


def draw_mutant_roe(img, rng):
    d = ImageDraw.Draw(img)
    glob = (216, 122, 96, 255)
    d.ellipse([4, 7, 28, 25], fill=glob, outline=shade(glob, 0.66))
    d.ellipse([7, 9, 15, 14], fill=shade(glob, 1.18))
    # fry silhouette inside: body + tail + eye
    fry = (96, 42, 36, 255)
    d.ellipse([11, 14, 21, 19], fill=fry)
    d.polygon([(20, 15), (25, 12.5), (25, 20)], fill=fry)
    d.point((13, 16), fill=(230, 200, 180, 255))
    bead(d, 6.5, 26, 2.0, (200, 95, 70, 255))
    bead(d, 26, 26.5, 1.8, (200, 95, 70, 255))


def draw_cured(img, rng):
    d = ImageDraw.Draw(img)
    darks = [(104, 54, 24, 255), (64, 36, 20, 255), (140, 78, 34, 255)]
    for ri, cnt in enumerate((4, 6, 7, 6, 4)):
        y = 13.0 + ri * 3.4
        for i in range(cnt):
            x = 16 + (i - (cnt - 1) / 2) * 3.9 + rng.uniform(-0.6, 0.6)
            c = rng.choice(darks)
            bead(d, x, y, 2.3, c)
            d.point((int(x - 0.8), int(y - 0.8)), fill=(245, 235, 220, 255))


def draw_curing(img, rng):
    d = ImageDraw.Draw(img)
    tub = (148, 148, 154, 255)
    d.ellipse([3, 9, 29, 27], fill=tub, outline=shade(tub, 0.6))
    d.ellipse([5.5, 11, 26.5, 24], fill=(196, 128, 84, 255))
    for _ in range(26):
        x = rng.uniform(9, 23)
        y = rng.uniform(13.5, 22)
        if ((x - 16) / 9.5) ** 2 + ((y - 17.5) / 5.5) ** 2 < 1:
            bead(d, x, y, 1.9, (216, 142, 92, 255))
    for _ in range(5):  # brine sheen streaks
        x = rng.uniform(8, 21)
        y = rng.uniform(13, 21)
        d.line([(x, y), (x + 3, y)], fill=(242, 240, 234, 255))


def fried_white(d, rng, cx, cy, r):
    pts = []
    for i in range(14):
        t = 2 * math.pi * i / 14
        rr = r * rng.uniform(0.82, 1.12)
        pts.append((cx + rr * math.cos(t), cy + rr * math.sin(t) * 0.82))
    d.polygon(pts, fill=(246, 244, 238, 255))
    d.line(pts + [pts[0]], fill=(198, 158, 104, 255), width=1)
    return pts


def draw_fried(img, rng):
    d = ImageDraw.Draw(img)
    fried_white(d, rng, 11.5, 13, 8.2)
    fried_white(d, rng, 21, 20, 8.6)
    for (x, y) in ((12, 12.5), (20.5, 19.5)):  # runny yolks
        d.ellipse([x - 3.4, y - 3.4, x + 3.4, y + 3.4],
                  fill=(246, 166, 38, 255), outline=(214, 128, 26, 255))
        d.point((int(x - 1), int(y - 1)), fill=(255, 214, 112, 255))
    for _ in range(9):  # garnish flecks + pepper flakes
        x, y = rng.uniform(5, 27), rng.uniform(6, 27)
        d.point((int(x), int(y)), fill=(92, 142, 62, 255))
        d.point((int(x) + 6, int(y) + 3), fill=(190, 62, 42, 255))


def draw_sandwich(img, rng, caviar):
    d = ImageDraw.Draw(img)
    crust = (172, 128, 78, 255)
    crumb = (233, 201, 148, 255)
    # bottom slice
    d.rounded_rectangle([5, 19, 27, 27], 3, fill=crust)
    d.rounded_rectangle([6.5, 20, 25.5, 26], 2, fill=crumb)
    if caviar:
        # sliced boiled egg rounds + caviar on the filling line
        for x in (10.5, 16, 21.5):
            d.ellipse([x - 3, 15.5, x + 3, 21.5], fill=(244, 242, 236, 255),
                      outline=(196, 190, 178, 255))
            d.ellipse([x - 1.4, 17.1, x + 1.4, 19.9],
                      fill=(242, 182, 62, 255))
        for _ in range(14):
            bead(d, rng.uniform(7, 25), rng.uniform(16, 20.5), 1.1,
                 rng.choice([(70, 40, 22, 255), (110, 58, 26, 255)]),
                 hi=False)
    else:
        # fried egg bulging out of the sandwich, yolk dome in front
        d.ellipse([4, 14, 28, 21], fill=(246, 244, 238, 255),
                  outline=(198, 158, 104, 255))
        d.pieslice([12.5, 12.5, 19.5, 20.5], 180, 360,
                   fill=(246, 166, 38, 255), outline=(214, 128, 26, 255))
    # top slice, slightly offset
    d.rounded_rectangle([5, 6, 27, 15], 3, fill=crust)
    d.rounded_rectangle([6.5, 7.5, 25.5, 13.5], 2, fill=crumb)
    for _ in range(7):  # toast flecks
        d.point((rng.randint(8, 24), rng.randint(8, 12)),
                fill=shade(crumb, 0.8))


def draw_insect(img, rng):
    d = ImageDraw.Draw(img)
    grain = (236, 229, 202, 255)
    for _ in range(15):
        x = rng.uniform(9, 23)
        y = rng.uniform(10, 22)
        a = rng.uniform(0, math.pi)
        dx, dy = 2.2 * math.cos(a), 2.2 * math.sin(a)
        d.line([(x - dx, y - dy), (x + dx, y + dy)], fill=grain, width=2)
        d.point((int(x), int(y)), fill=shade(grain, 0.85))


# id -> draw spec. Bird eggs: ("egg", base, speckle, n_speck, rx, ry, extra)
BIRD = {
    "egg_bird_unfert": ((226, 214, 188, 255), None, 0, 8.5, 10.5, {}),
    "egg_bluejay": ((140, 190, 180, 255), (110, 80, 55, 255), 26, 6.5, 8, {}),
    "egg_cardinal": ((196, 182, 158, 255), (130, 95, 70, 255), 24, 7, 8.5, {}),
    "egg_chickadee": ((238, 236, 228, 255), (150, 90, 70, 255), 14, 5, 6.5, {}),
    "egg_coot": ((205, 188, 150, 255), (90, 75, 60, 255), 34, 8, 10, {}),
    "egg_cormorant": ((168, 205, 195, 255), (140, 180, 168, 255), 10, 7, 11, {}),
    "egg_crow": ((130, 180, 165, 255), None, 0, 7.5, 9.5,
                 {"blotch": (105, 85, 60, 255), "n_blotch": 7}),
    "egg_duck": ((215, 220, 190, 255), None, 0, 8.5, 10.5, {}),
    "egg_duck_domestic": ((238, 232, 216, 255), None, 0, 9, 11, {}),
    "egg_goose": ((240, 238, 230, 255), None, 0, 10.5, 13, {}),
    "egg_goose_canadian": ((232, 224, 200, 255), None, 0, 10, 12.5, {}),
    "egg_grebe": ((228, 218, 192, 255), None, 0, 7, 11,
                  {"blotch": (188, 168, 132, 255), "n_blotch": 5}),
    "egg_grouse": ((208, 186, 146, 255), (140, 105, 70, 255), 26, 7.5, 9.5, {}),
    "egg_guinea_fowl": ((196, 164, 120, 255), (120, 90, 60, 255), 34, 7, 9.5,
                        {"asym": 0.3}),
    "egg_pheasant": ((158, 142, 104, 255), None, 0, 7.5, 9.5, {}),
    "egg_pigeon": ((238, 238, 234, 255), None, 0, 6, 7.5, {}),
    "egg_raven": ((125, 175, 160, 255), None, 0, 8.5, 10.5,
                  {"blotch": (88, 78, 66, 255), "n_blotch": 8}),
    "egg_moorhen": ((202, 182, 142, 255), None, 0, 7.5, 9.5,
                    {"blotch": (95, 70, 50, 255), "n_blotch": 6}),
}
ROE = {
    "egg_blackfish": (120, 60, 45, 255),
    "egg_bluefin_tuna": (185, 70, 40, 255),
    "egg_bluefish": (200, 95, 45, 255),
    "egg_channel_catfish": (215, 170, 80, 255),
    "egg_cod": (225, 150, 110, 255),
    "egg_flounder": (230, 180, 160, 255),
    "egg_haddock": (220, 165, 95, 255),
    "egg_halibut": (232, 215, 180, 255),
    "egg_mackerel": (180, 160, 85, 255),
    "egg_pollock": (225, 155, 130, 255),
    "egg_porgy": (215, 120, 90, 255),
}
FROG = {
    "egg_bullfrog_frog": "disc",
    "egg_fowler_toad": "strand",
    "egg_freedom_toad": "spiral_veg",
    "egg_gray_frog": "patch",
    "egg_green_frog": "sheet",
    "egg_leo_frog": "dense",
    "egg_mink_frog": "ball_polar",
    "egg_peeper_frog": "sparse_veg",
    "egg_pickerel_frog": "glob_bicolor",
    "egg_foul_bigfrog": "bigstrand",
    "egg_odd_bigfrog": "bigspiral",
    "egg_pattern_bigfrog": "bigcaviar",
    "egg_proxy_bigfrog": "bigclump",
}


def render(item_id, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    if item_id in BIRD:
        base, speck, n, rx, ry, extra = BIRD[item_id]
        draw_egg(img, rng, 16, 16.5, rx, ry, base, speckle=speck,
                 n_speck=n, **extra)
    elif item_id == "egg_hummingbird":
        draw_egg(img, rng, 12.5, 16, 3.4, 4.4, (240, 240, 236, 255))
        draw_egg(img, rng, 19.5, 16.5, 3.4, 4.4, (240, 240, 236, 255))
    elif item_id == "boiled_egg":
        draw_egg(img, rng, 16, 16.5, 8.5, 10.5, (240, 238, 232, 255),
                 crack=True)
    elif item_id in ROE:
        draw_roe(img, rng, ROE[item_id],
                 bead_r=1.8 if item_id == "egg_flounder" else 2.2)
    elif item_id == "egg_perch":
        draw_roe_ribbon(img, rng, (225, 195, 110, 255))
    elif item_id == "egg_mutant_salmon":
        draw_mutant_roe(img, rng)
    elif item_id == "cured_roe":
        draw_cured(img, rng)
    elif item_id == "curing_roe_active":
        draw_curing(img, rng)
    elif item_id in FROG:
        draw_frog(img, rng, FROG[item_id])
    elif item_id == "egg_moth_fly":
        draw_insect(img, rng)
    elif item_id == "deluxe_fried_eggs":
        draw_fried(img, rng)
    elif item_id == "caviar_egg_sandwich":
        draw_sandwich(img, rng, caviar=True)
    elif item_id == "deluxe_fried_egg_sandwich":
        draw_sandwich(img, rng, caviar=False)
    else:
        raise KeyError(item_id)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (item_id, alphas)
    return img


def main():
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = render(item_id, rng)
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
                       "preview_item07.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item07.png {sheet.size}")


if __name__ == "__main__":
    main()
