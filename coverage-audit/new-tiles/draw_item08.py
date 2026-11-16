#!/usr/bin/env python3
"""Batch ITEM08 (Phase 2g.4 scope extension): carnivore comestibles,
first set. 52 ids from data/json/items/comestibles/carnivore.json
(authoritative list embedded below; source file:
~/workspace/item-batches/08.txt).

Parameterized family renderers driven by each entry's data
(copy-from chains resolved by hand against carnivore.json):

- Meat chunks: irregular marbled blob; raw / cooked (sear
  speckle) / fried (dark crispy edge) / fatty (white fat
  patches) / tainted (sickly green mottle) variants.
- Hearts: parametric heart silhouette + aorta stub; raw,
  cooked, mutant (green tint + nodules), the black_red
  golem heart (fist-sized, dimpled grooves), jabberwock
  heart (head-sized, ribbed grooves) and its desiccated
  (shriveled, wrinkled) form.
- Stomachs: J-shaped sac; normal / large / boiled (pale).
- Bacon: uncut slab (smoked, fat-striped) vs uncooked
  strips (wavy pink, white fat lines).
- Fats: tallow = smooth white block; tainted fat =
  watery yellow glob with a wrong green hint.
- Jerky: thin dark dried strips, salt speckle.
- Tainted limbs (arm_zed / leg_zed): rotted flesh, bone
  end, shred slashes; leg thicker than arm.
- Alien plant: meat_frond (membranous teal leaves, purple
  veins), meat_bark (dry pale-amber shards, glisten),
  leech_flower (indigo fleshy petals, glossy).
- bone_mi-go: deep green springy curved rod, lighter bands.
- Seafood: fish scrap (silvery sliver) raw/cooked, smoked
  fish (brown), lobster tail (segmented, red-orange;
  smoked browner), mollusk (soft beige blob), gastropods
  (handful of small snails) purged/cooked, frog legs pair
  raw/cooked/smoked.
- Mutant organs: cooked brain (gyri squiggles), cooked
  bug organs (dark glossy blobs + chitin shards), kidney
  (bean w/ notch) raw/cooked, liver (flat dark lobe),
  lung (spongy lobes) raw/cooked.

Items are objects on transparency, native 32x32, flat
UltiCa style, bodies fully opaque (alpha in {0,255});
translucency faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item08.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageChops, ImageDraw, ImageFilter

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")
IDS = [
    "arm_zed", "bacon_uncooked", "bacon_uncut", "bone_mi-go",
    "demihuman_flesh", "demihuman_heart", "demihuman_stomach",
    "demihuman_stomach_large", "fat_tainted_human", "fish_scrap",
    "fish_scrap_cooked", "fish_smoked", "flesh_golem_heart",
    "froglegs", "froglegs_cooked", "froglegs_smoked",
    "gastropod_cooked", "gastropod_purged", "heart", "heart_cooked",
    "hstomach", "hstomach_boiled", "hstomach_large", "human_cooked",
    "human_flesh", "human_heart", "human_tallow", "jabberwock_heart",
    "jabberwock_heart_desiccated", "jerky", "leech_flower", "leg_zed",
    "lobster_cooked", "lobster_smoked", "meat_bark",
    "meat_fatty_cooked", "meat_fried", "meat_frond", "meat_mi-go",
    "meat_tainted_human", "mollusk_cooked", "mollusk_smoked",
    "mutant_brain_cooked", "mutant_bug_organs_cooked", "mutant_heart",
    "mutant_heart_cooked", "mutant_human_heart", "mutant_kidney",
    "mutant_kidney_cooked", "mutant_liver", "mutant_lung",
    "mutant_lung_cooked",
]
assert len(IDS) == 52, len(IDS)


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3])


def mix(c1, c2, t):
    return tuple(int(a + (b - a) * t) for a, b in zip(c1, c2))


def blob_pts(rng, cx, cy, rx, ry, wob=0.20, n=16, rot=0.0):
    pts = []
    cr, sr = math.cos(rot), math.sin(rot)
    for i in range(n):
        t = 2 * math.pi * i / n
        r = 1 + rng.uniform(-wob, wob)
        x, y = rx * r * math.cos(t), ry * r * math.sin(t)
        pts.append((cx + x * cr - y * sr, cy + x * sr + y * cr))
    return pts


def mask_from(draw_fn):
    m = Image.new("L", (32, 32), 0)
    d = ImageDraw.Draw(m)
    draw_fn(d)
    return m


def texture(mask, base, rng, mottle=None, mottle_n=46, hi=None, hi_n=14,
            edge_f=0.55):
    """Fill mask with base color + mottling + dark outline."""
    layer = Image.new("RGBA", (32, 32), base + (255,))
    d = ImageDraw.Draw(layer)
    if mottle:
        for _ in range(mottle_n):
            x, y = rng.uniform(1, 30), rng.uniform(1, 30)
            r = rng.uniform(0.8, 2.4)
            d.ellipse([x - r, y - r, x + r, y + r],
                      fill=shade(mottle, rng.uniform(0.85, 1.1)) + (255,))
    if hi:
        for _ in range(hi_n):
            x, y = rng.uniform(2, 29), rng.uniform(2, 29)
            r = rng.uniform(0.6, 1.6)
            d.ellipse([x - r, y - r, x + r, y + r], fill=hi + (255,))
    img = Image.composite(layer, Image.new("RGBA", (32, 32), (0, 0, 0, 0)),
                          mask)
    er = mask.filter(ImageFilter.MinFilter(3))
    edge = ImageChops.subtract(mask, er)
    dark = Image.new("RGBA", (32, 32), shade(base, edge_f) + (255,))
    return Image.composite(dark, img, edge)


def clipped(img, mask, draw_fn):
    """Draw opaque details on a layer, clip to mask, composite."""
    layer = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    draw_fn(d)
    layer = Image.composite(layer, Image.new("RGBA", (32, 32), (0, 0, 0, 0)),
                            mask)
    img.alpha_composite(layer)
    return img


def speckles(rng, n, color, box=(3, 3, 29, 29), rmax=1.4):
    def fn(d):
        for _ in range(n):
            x = rng.uniform(box[0], box[2])
            y = rng.uniform(box[1], box[3])
            r = rng.uniform(0.5, rmax)
            d.ellipse([x - r, y - r, x + r, y + r], fill=color + (255,))
    return fn


# ---------------------------------------------------------------- meat chunk
def meat_chunk(rng, base, cooked=False, fried=False, fatty=False,
               tainted=False, alien=False):
    pts = blob_pts(rng, 16, 17, 11.5, 8.5, wob=0.24)
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    mottle = shade(base, 0.8)
    img = texture(mask, base, rng, mottle=mottle,
                  hi=shade(base, 1.25), hi_n=10)
    light = mix(base, (245, 235, 220), 0.55)

    def veins(d):
        for _ in range(3 if fatty else 2):
            y0 = rng.uniform(10, 24)
            p = [(4 + i * 3, y0 + rng.uniform(-2.5, 2.5)) for i in range(9)]
            d.line(p, fill=light + (255,), width=2 if fatty else 1)
    img = clipped(img, mask, veins)
    if fatty:
        img = clipped(img, mask, speckles(rng, 7, (238, 228, 205),
                                          rmax=2.6))
    if cooked or fried:
        img = clipped(img, mask, speckles(rng, 26, shade(base, 0.55),
                                          rmax=1.2))
        img = clipped(img, mask, speckles(rng, 8, (60, 35, 20),
                                          rmax=0.9))
    if tainted:
        img = clipped(img, mask, speckles(rng, 16, (105, 118, 66),
                                          rmax=2.2))
    if alien:  # mi-go: strange pale bumps
        img = clipped(img, mask, speckles(rng, 12, (245, 200, 210),
                                          rmax=1.8))
    return img


# -------------------------------------------------------------------- heart
def heart_pts(cx, cy, w, h):
    raw = []
    for i in range(44):
        t = 2 * math.pi * i / 44
        x = 16 * math.sin(t) ** 3
        y = (13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t)
             - math.cos(4 * t))
        raw.append((x, y))
    xs = [p[0] for p in raw]
    ys = [p[1] for p in raw]
    sx = w / (max(xs) - min(xs))
    sy = h / (max(ys) - min(ys))
    ox = (max(xs) + min(xs)) / 2
    oy = (max(ys) + min(ys)) / 2
    return [(cx + (x - ox) * sx, cy - (y - oy) * sy) for x, y in raw]


def heart(rng, base, w=19, h=19, grooves=None, desiccated=False,
          mutant=False, cooked=False):
    if mutant:
        base = mix(base, (96, 128, 60), 0.22)
    cx, cy = 16, 17
    pts = heart_pts(cx, cy, w, h)
    if desiccated:
        pts = [(cx + (x - cx) * 0.88 + rng.uniform(-0.7, 0.7),
                cy + (y - cy) * 0.92 + rng.uniform(-0.7, 0.7))
               for x, y in pts]
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    img = texture(mask, base, rng, mottle=shade(base, 0.78),
                  hi=shade(base, 1.3), hi_n=9)
    # aorta / vessel stubs at the top
    def vessels(d):
        d.rectangle([cx - 3, cy - h // 2 - 2, cx + 1, cy - h // 2 + 3],
                    fill=shade(base, 0.8) + (255,))
        d.ellipse([cx - 3, cy - h // 2 - 3, cx + 1, cy - h // 2 - 1],
                  fill=shade(base, 0.5) + (255,))
        d.rectangle([cx + 3, cy - h // 2, cx + 6, cy - h // 2 + 3],
                    fill=shade(base, 0.85) + (255,))
    vessel_mask = mask_from(lambda d: (
        d.rectangle([cx - 3, cy - h // 2 - 2, cx + 1, cy - h // 2 + 3],
                    fill=255),
        d.rectangle([cx + 3, cy - h // 2, cx + 6, cy - h // 2 + 3],
                    fill=255)))
    img = clipped(img, vessel_mask, vessels)
    if grooves == "dimple":
        img = clipped(img, mask, speckles(rng, 22, shade(base, 0.5),
                                          rmax=1.5))
    if grooves == "ribbed":
        def ribs(d):
            for k in range(5):
                y = cy - h // 2 + 4 + k * 3.4
                d.arc([cx - w // 2 + 2, y - 4, cx + w // 2 - 2, y + 5],
                      20, 160, fill=shade(base, 0.5) + (255,), width=1)
        img = clipped(img, mask, ribs)
    if desiccated:
        def wrinkles(d):
            for _ in range(9):
                x0, y0 = rng.uniform(8, 22), rng.uniform(9, 25)
                d.line([(x0, y0), (x0 + rng.uniform(-3, 3),
                                   y0 + rng.uniform(1, 4))],
                       fill=shade(base, 0.45) + (255,), width=1)
        img = clipped(img, mask, wrinkles)
    if mutant:
        img = clipped(img, mask, speckles(rng, 6, (150, 170, 90),
                                          rmax=1.7))
    if cooked:
        img = clipped(img, mask, speckles(rng, 18, shade(base, 0.6),
                                          rmax=1.1))
    return img


# ------------------------------------------------------------------ stomach
def stomach(rng, base, scale=1.0, boiled=False):
    cx, cy = 15, 17
    rx, ry = 8.5 * scale, 10 * scale

    def shape(d):
        d.ellipse([cx - rx, cy - ry + 3, cx + rx, cy + ry], fill=255)
        d.ellipse([cx + 1, cy - ry - 3 * scale, cx + rx + 2,
                   cy - ry + 6], fill=255)  # esophagus tube
        d.ellipse([cx + rx - 3, cy + ry - 4, cx + rx + 3,
                   cy + ry + 1], fill=255)  # pylorus nub
    mask = mask_from(shape)
    if boiled:
        base = mix(base, (232, 205, 190), 0.55)
    img = texture(mask, base, rng, mottle=shade(base, 0.85),
                  hi=shade(base, 1.2), hi_n=10)

    def fold(d):  # rugae fold shading
        d.arc([cx - rx + 2, cy - 2, cx + rx - 3, cy + ry - 1], 300, 80,
              fill=shade(base, 0.75) + (255,), width=1)
    img = clipped(img, mask, fold)
    return img


# --------------------------------------------------------------------- fats
def fat_glob(rng):
    pts = blob_pts(rng, 16, 18, 11, 7.5, wob=0.3)
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    base = (212, 182, 88)
    img = texture(mask, base, rng, mottle=(190, 160, 80),
                  hi=(245, 230, 170), hi_n=16)
    img = clipped(img, mask, speckles(rng, 7, (120, 128, 70), rmax=1.6))
    return img


def tallow_block(rng):
    mask = mask_from(lambda d: d.rounded_rectangle(
        [6, 9, 26, 24], radius=4, fill=255))
    base = (240, 237, 226)
    img = texture(mask, base, rng, mottle=(228, 224, 210), mottle_n=18,
                  hi=(252, 250, 244), hi_n=8, edge_f=0.82)
    return img


# -------------------------------------------------------------------- bacon
def bacon_slab(rng):
    mask = mask_from(lambda d: d.rounded_rectangle(
        [4, 8, 28, 25], radius=3, fill=255))
    base = (148, 68, 52)
    img = texture(mask, base, rng, mottle=(128, 55, 42),
                  hi=(175, 95, 75), hi_n=8)

    def stripes(d):
        fat = (238, 228, 208)
        d.line([(5, 13), (27, 12)], fill=fat + (255,), width=3)
        d.line([(5, 20), (27, 21)], fill=fat + (255,), width=2)
        d.line([(5, 13), (27, 12)], fill=(200, 120, 95, 255), width=1)
    img = clipped(img, mask, stripes)
    return img


def bacon_strips(rng):
    base = (214, 128, 128)
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    for k, x0 in enumerate((8, 15, 22)):
        pts_l, pts_r = [], []
        for i in range(9):
            y = 6 + i * 2.6
            off = math.sin(i * 1.1 + k * 2.0) * 1.6
            pts_l.append((x0 - 2.2 + off, y))
            pts_r.append((x0 + 2.2 + off, y))
        pts = pts_l + pts_r[::-1]
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        strip = texture(mask, base, rng, mottle=(196, 108, 108),
                        hi=(232, 160, 155), hi_n=5)
        def center(d, x0=x0, k=k):
            p = [(x0 + math.sin(i * 1.1 + k * 2.0) * 1.6, 6 + i * 2.6)
                 for i in range(9)]
            d.line(p, fill=(240, 232, 216, 255), width=1)
        strip = clipped(strip, mask, center)
        img.alpha_composite(strip)
    return img


# -------------------------------------------------------------------- jerky
def jerky(rng):
    base = (104, 60, 34)
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    for k, rot in enumerate((-0.18, 0.06, 0.28)):
        cx, cy = 9.5 + k * 6.5, 16 + (k % 2) * 1.5 - 0.75
        pts = blob_pts(rng, cx, cy, 2.7, 11.0, wob=0.3, n=12, rot=rot)
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        strip = texture(mask, base, rng, mottle=(88, 48, 26),
                        hi=(140, 92, 52), hi_n=5)
        strip = clipped(strip, mask, speckles(rng, 5, (240, 238, 230),
                                               rmax=0.7))
        img.alpha_composite(strip)
    return img


# --------------------------------------------------------------------- limb
def limb(rng, thick=4.5):
    # horizontal rotted limb, bone end at left
    spine = [(7, 19), (12, 16.5), (18, 16), (24, 17.5), (27, 19)]
    up, dn = [], []
    for i, (x, y) in enumerate(spine):
        w = thick * (1 - 0.35 * i / (len(spine) - 1)) + rng.uniform(-0.5, 0.5)
        up.append((x, y - w))
        dn.append((x, y + w))
    pts = up + dn[::-1]
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    base = (138, 70, 54)
    img = texture(mask, base, rng, mottle=(110, 110, 66), mottle_n=30,
                  hi=(170, 105, 85), hi_n=8)

    def shreds(d):
        for _ in range(6):
            x, y = rng.uniform(10, 26), rng.uniform(13, 21)
            d.line([(x, y), (x + rng.uniform(1, 3), y + rng.uniform(-1, 1))],
                   fill=(52, 30, 24, 255), width=1)
    img = clipped(img, mask, shreds)
    bone = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    bd = ImageDraw.Draw(bone)
    bd.ellipse([3, 15.5, 9, 21.5], fill=(226, 216, 196, 255))
    bd.ellipse([4.5, 17, 7.5, 20], fill=(200, 188, 165, 255))
    img.alpha_composite(bone)
    return img


# --------------------------------------------------------------- alien flora
def frond(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    base_c = (88, 148, 128)
    for k, (dx, rot) in enumerate(((-6, -0.5), (0, 0.0), (6, 0.5))):
        pts = blob_pts(rng, 16 + dx, 15, 3.4, 10.5, wob=0.18, n=12,
                       rot=rot)
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        leaf = texture(mask, base_c, rng, mottle=(70, 125, 110),
                       hi=(120, 180, 158), hi_n=7)

        def vein(d, dx=dx, rot=rot):
            cr, sr = math.cos(rot), math.sin(rot)
            p = []
            for i in range(9):
                lx, ly = 0, -9 + i * 2.2
                p.append((16 + dx + lx * cr - ly * sr,
                          15 + lx * sr + ly * cr))
            d.line(p, fill=(128, 74, 138, 255), width=1)
        leaf = clipped(leaf, mask, vein)
        img.alpha_composite(leaf)
    stem = ImageDraw.Draw(img)
    stem.line([(16, 30), (16, 24)], fill=(96, 84, 60, 255), width=2)
    return img


def bark(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    base_c = (172, 142, 100)
    for k in range(3):
        cx, cy = 10 + k * 6.5, 13 + (k % 2) * 7
        pts = blob_pts(rng, cx, cy, 5.5, 3.2, wob=0.35, n=10,
                       rot=rng.uniform(-0.5, 0.5))
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        shard = texture(mask, base_c, rng, mottle=(150, 120, 82),
                        hi=(205, 182, 138), hi_n=6)
        shard = clipped(shard, mask, speckles(rng, 4, (245, 240, 210),
                                              rmax=0.8))
        img.alpha_composite(shard)
    return img


def leech_flower(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    cx, cy = 16, 16
    petal = (84, 82, 184)
    for k in range(6):
        t = 2 * math.pi * k / 6
        px, py = cx + 6.2 * math.cos(t), cy + 6.2 * math.sin(t)
        pts = blob_pts(rng, px, py, 5.2, 3.4, wob=0.15, n=12, rot=t)
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        p = texture(mask, mix(petal, (130, 120, 215), 0.25 * (k % 2)),
                    rng, mottle=(66, 62, 150), hi=(150, 145, 230), hi_n=5)
        img.alpha_composite(p)
    d = ImageDraw.Draw(img)
    d.ellipse([cx - 3.4, cy - 3.4, cx + 3.4, cy + 3.4],
              fill=(42, 36, 92, 255))
    d.ellipse([cx - 1.4, cy - 1.8, cx + 0.6, cy - 0.2],
              fill=(120, 60, 90, 255))  # fleshy throat glisten
    return img


def bone_migo(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    base_c = (38, 106, 58)
    spine = [(8, 24), (12, 17), (17, 12), (23, 9)]
    up, dn = [], []
    for i, (x, y) in enumerate(spine):
        w = 2.6 - 0.4 * i + rng.uniform(-0.3, 0.3)
        up.append((x - w * 0.6, y - w))
        dn.append((x + w * 0.6, y + w))
    pts = up + dn[::-1]
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    img = texture(mask, base_c, rng, mottle=(30, 88, 48),
                  hi=(80, 160, 96), hi_n=8)

    def bands(d):
        for (x, y) in spine[1:]:
            d.line([(x - 2.4, y + 1.5), (x + 2.4, y - 1.5)],
                   fill=(96, 176, 104, 255), width=1)
    img = clipped(img, mask, bands)
    return img


# ------------------------------------------------------------------ seafood
def fish_piece(rng, base, cooked=False, smoked=False):
    pts = [(6, 17), (11, 11), (20, 10), (25, 14), (29, 11), (28, 17),
           (29, 22), (25, 19), (20, 22), (11, 22), (6, 17)]
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    img = texture(mask, base, rng, mottle=shade(base, 0.85),
                  hi=(225, 228, 232) if not (cooked or smoked)
                  else shade(base, 1.25), hi_n=10)

    def details(d):
        d.ellipse([9.5, 14.5, 11.5, 16.5], fill=(30, 30, 34, 255))
        d.arc([15, 11, 23, 22], 270, 90, fill=shade(base, 0.7) + (255,))
        if cooked:
            d.line([(13, 13), (24, 13)], fill=(120, 78, 40, 255), width=1)
    img = clipped(img, mask, details)
    return img


def lobster(rng, smoked=False):
    shell = (150, 84, 48) if smoked else (206, 82, 40)
    meat = (238, 200, 168)
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    # segmented tail curving down-right, fan at bottom
    centers = [(13, 8), (14, 12), (15.5, 16), (17, 20), (18.5, 24)]
    full = Image.new("L", (32, 32), 0)
    fd = ImageDraw.Draw(full)
    for i, (x, y) in enumerate(centers):
        w = 5.2 - i * 0.35
        fd.ellipse([x - w, y - 3.2, x + w, y + 3.2], fill=255)
    fd.polygon([(14, 26), (23, 26), (26, 31), (11, 31)], fill=255)  # fan
    seg = texture(full, shell, rng, mottle=shade(shell, 0.82),
                  hi=shade(shell, 1.3), hi_n=10)

    def lines(d):
        for (x, y) in centers[1:]:
            d.arc([x - 5, y - 4.5, x + 5, y + 2], 15, 165,
                  fill=shade(shell, 0.6) + (255,), width=1)
        for k in range(3):  # fan rays
            d.line([(18.5, 27), (13 + k * 5.5, 30.5)],
                   fill=shade(shell, 0.6) + (255,), width=1)
        d.ellipse([9.5, 5.5, 16.5, 10.5], fill=meat + (255,))  # meat top
        d.ellipse([11, 6.5, 15, 9.5], fill=shade(meat, 0.92) + (255,))
    seg = clipped(seg, full, lines)
    img.alpha_composite(seg)
    return img


def mollusk(rng, smoked=False):
    base = (148, 108, 68) if smoked else (214, 188, 158)
    pts = blob_pts(rng, 16, 17, 10.5, 7.5, wob=0.34, n=18)
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    img = texture(mask, base, rng, mottle=shade(base, 0.85),
                  hi=shade(base, 1.15), hi_n=10)

    def ruffle(d):
        d.arc([8, 11, 24, 23], 20, 160, fill=shade(base, 0.72) + (255,),
              width=1)
    img = clipped(img, mask, ruffle)
    return img


def gastropods(rng, cooked=False):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    body_c = (158, 122, 80) if cooked else (148, 152, 128)
    shell_c = (140, 112, 74) if cooked else (118, 108, 88)
    for (cx, cy, s) in ((11, 12, 1.0), (21, 15, 0.85), (14, 23, 0.9)):
        d = ImageDraw.Draw(img)
        d.ellipse([cx - 5 * s, cy + 1 * s, cx + 6 * s, cy + 4.5 * s],
                  fill=body_c + (255,))  # foot
        d.ellipse([cx - 4 * s, cy - 4.5 * s, cx + 3.6 * s, cy + 2.6 * s],
                  fill=shell_c + (255,))  # shell
        d.arc([cx - 2.8 * s, cy - 3.2 * s, cx + 2.2 * s, cy + 1.4 * s],
              0, 300, fill=shade(shell_c, 0.62) + (255,), width=1)
        d.arc([cx - 1.4 * s, cy - 1.8 * s, cx + 1.2 * s, cy + 0.8 * s],
              0, 300, fill=shade(shell_c, 0.62) + (255,), width=1)
        d.line([(cx + 4 * s, cy), (cx + 6.4 * s, cy - 2.4 * s)],
               fill=body_c + (255,), width=1)  # eyestalk
    return img


def froglegs(rng, state="raw"):
    base = {"raw": (192, 202, 162), "cooked": (216, 170, 92),
            "smoked": (142, 96, 54)}[state]
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    for sgn in (-1, 1):
        cx = 16 + sgn * 4.5
        pts = blob_pts(rng, cx, 12, 3.6, 6.5, wob=0.16, n=12,
                       rot=sgn * 0.22)
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        leg = texture(mask, base, rng, mottle=shade(base, 0.86),
                      hi=shade(base, 1.15), hi_n=6)
        img.alpha_composite(leg)
        d = ImageDraw.Draw(img)
        sx = cx + sgn * 2.2
        d.line([(sx, 17), (sx + sgn * 1.6, 25)],
               fill=shade(base, 0.9) + (255,), width=2)  # shin
        for t in (-1, 0, 1):  # toes
            d.line([(sx + sgn * 1.6, 25),
                    (sx + sgn * (2.6 + t * 0.4), 28 + abs(t))],
                   fill=shade(base, 0.9) + (255,), width=1)
    if state != "raw":
        img = clipped(img, img.getchannel("A"),
                      speckles(rng, 12, shade(base, 0.6), rmax=1.0))
    return img


# ------------------------------------------------------------------- organs
def brain_cooked(rng):
    pts = blob_pts(rng, 16, 16, 10.5, 9.5, wob=0.14, n=16)
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    base = (186, 150, 134)
    img = texture(mask, base, rng, mottle=(168, 132, 118),
                  hi=(205, 175, 158), hi_n=8)

    def gyri(d):
        for _ in range(11):
            x0, y0 = rng.uniform(8, 22), rng.uniform(8, 22)
            p = [(x0 + i * 1.6, y0 + math.sin(i * 1.7 + x0) * 1.6)
                 for i in range(4)]
            d.line(p, fill=(138, 102, 92, 255), width=1)
        d.line([(16, 7), (16, 25)], fill=(138, 102, 92, 255), width=1)
    img = clipped(img, mask, gyri)
    img = clipped(img, mask, speckles(rng, 10, (120, 80, 55), rmax=0.9))
    return img


def bug_organs_cooked(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    base = (74, 56, 44)
    for (cx, cy, rx, ry) in ((12, 13, 6.5, 5), (21, 17, 7, 5.5),
                             (13, 22, 5.5, 4.2)):
        pts = blob_pts(rng, cx, cy, rx, ry, wob=0.25)
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        blob = texture(mask, base, rng, mottle=(58, 42, 34),
                       hi=(150, 118, 70), hi_n=7)
        img.alpha_composite(blob)
    d = ImageDraw.Draw(img)  # chitin shards
    for (x, y, r) in ((19, 9, 0.4), (9, 19, 2.2), (24, 24, 1.1)):
        d.polygon([(x, y), (x + 5, y + 1.5), (x + 1.5, y + 4.5)],
                  fill=(150, 100, 46, 255))
    return img


def kidney(rng, cooked=False):
    base = (118, 66, 46) if cooked else (122, 40, 56)
    full = mask_from(lambda d: d.ellipse([7, 6, 25, 27], fill=255))
    notch = mask_from(lambda d: d.ellipse([19, 12, 29, 21], fill=255))
    mask = ImageChops.subtract(full, notch)
    img = texture(mask, base, rng, mottle=shade(base, 0.82),
                  hi=shade(base, 1.3), hi_n=8)

    def hilum(d):
        d.ellipse([19.5, 14.5, 23, 18.5], fill=(196, 160, 130, 255))
    img = clipped(img, mask, hilum)
    if cooked:
        img = clipped(img, mask, speckles(rng, 14, shade(base, 0.6),
                                          rmax=1.0))
    return img


def liver(rng):
    pts = [(5, 14), (10, 9), (20, 8), (27, 12), (26, 17), (18, 21),
           (9, 21), (5, 18)]
    mask = mask_from(lambda d: d.polygon(pts, fill=255))
    base = (110, 44, 40)
    img = texture(mask, base, rng, mottle=(96, 36, 34),
                  hi=(150, 78, 66), hi_n=7)

    def sheen(d):
        d.arc([7, 9, 25, 19], 180, 320, fill=(168, 96, 82, 255), width=2)
    img = clipped(img, mask, sheen)
    img = clipped(img, mask, speckles(rng, 4, (150, 170, 90), rmax=1.5))
    return img


def lung(rng, cooked=False):
    base = (150, 94, 74) if cooked else (192, 110, 104)
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    for (cx, rx) in ((12.5, 5.5), (20.5, 6.2)):
        pts = blob_pts(rng, cx, 17, rx, 10.5, wob=0.16, n=14)
        mask = mask_from(lambda d, p=pts: d.polygon(p, fill=255))
        lobe = texture(mask, base, rng, mottle=shade(base, 0.88),
                       hi=shade(base, 1.18), hi_n=9)
        lobe = clipped(lobe, mask, speckles(rng, 22, shade(base, 0.8),
                                            rmax=1.1))  # spongy holes
        img.alpha_composite(lobe)
    d = ImageDraw.Draw(img)
    d.line([(16, 3), (16, 8)], fill=(196, 170, 140, 255), width=2)
    if cooked:
        img = clipped(img, img.getchannel("A"),
                      speckles(rng, 12, (96, 58, 40), rmax=1.0))
    return img


# -------------------------------------------------------------------- render
RAW_MEAT = (168, 52, 48)


def render(item_id, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    if item_id == "demihuman_flesh":
        img = meat_chunk(rng, (160, 56, 50))
    elif item_id == "human_flesh":
        img = meat_chunk(rng, (172, 60, 52))
    elif item_id == "human_cooked":
        img = meat_chunk(rng, (146, 84, 52), cooked=True)
    elif item_id == "meat_tainted_human":
        img = meat_chunk(rng, (138, 88, 62), tainted=True)
    elif item_id == "meat_mi-go":
        img = meat_chunk(rng, (224, 138, 158), alien=True)
    elif item_id == "meat_fatty_cooked":
        img = meat_chunk(rng, (150, 86, 52), cooked=True, fatty=True)
    elif item_id == "meat_fried":
        img = meat_chunk(rng, (112, 62, 36), cooked=True, fried=True)
    elif item_id == "heart":
        img = heart(rng, (150, 30, 38))
    elif item_id == "heart_cooked":
        img = heart(rng, (122, 62, 46), cooked=True)
    elif item_id == "demihuman_heart":
        img = heart(rng, (146, 34, 40))
    elif item_id == "human_heart":
        img = heart(rng, (152, 32, 36))
    elif item_id == "mutant_heart":
        img = heart(rng, (148, 34, 40), mutant=True)
    elif item_id == "mutant_heart_cooked":
        img = heart(rng, (120, 62, 46), mutant=True, cooked=True)
    elif item_id == "mutant_human_heart":
        img = heart(rng, (150, 32, 36), mutant=True)
    elif item_id == "flesh_golem_heart":
        img = heart(rng, (92, 20, 28), w=17, h=17, grooves="dimple")
    elif item_id == "jabberwock_heart":
        img = heart(rng, (88, 18, 26), w=23, h=23, grooves="ribbed")
    elif item_id == "jabberwock_heart_desiccated":
        img = heart(rng, (70, 22, 24), w=21, h=21, grooves="ribbed",
                    desiccated=True)
    elif item_id == "demihuman_stomach":
        img = stomach(rng, (204, 128, 118))
    elif item_id == "demihuman_stomach_large":
        img = stomach(rng, (204, 128, 118), scale=1.18)
    elif item_id == "hstomach":
        img = stomach(rng, (208, 132, 120))
    elif item_id == "hstomach_large":
        img = stomach(rng, (208, 132, 120), scale=1.18)
    elif item_id == "hstomach_boiled":
        img = stomach(rng, (208, 132, 120), scale=1.12, boiled=True)
    elif item_id == "bacon_uncut":
        img = bacon_slab(rng)
    elif item_id == "bacon_uncooked":
        img = bacon_strips(rng)
    elif item_id == "fat_tainted_human":
        img = fat_glob(rng)
    elif item_id == "human_tallow":
        img = tallow_block(rng)
    elif item_id == "jerky":
        img = jerky(rng)
    elif item_id == "arm_zed":
        img = limb(rng, thick=3.4)
    elif item_id == "leg_zed":
        img = limb(rng, thick=5.2)
    elif item_id == "meat_frond":
        img = frond(rng)
    elif item_id == "meat_bark":
        img = bark(rng)
    elif item_id == "leech_flower":
        img = leech_flower(rng)
    elif item_id == "bone_mi-go":
        img = bone_migo(rng)
    elif item_id == "fish_scrap":
        img = fish_piece(rng, (198, 158, 148))
    elif item_id == "fish_scrap_cooked":
        img = fish_piece(rng, (202, 152, 92), cooked=True)
    elif item_id == "fish_smoked":
        img = fish_piece(rng, (142, 96, 54), smoked=True)
    elif item_id == "lobster_cooked":
        img = lobster(rng)
    elif item_id == "lobster_smoked":
        img = lobster(rng, smoked=True)
    elif item_id == "mollusk_cooked":
        img = mollusk(rng)
    elif item_id == "mollusk_smoked":
        img = mollusk(rng, smoked=True)
    elif item_id == "gastropod_purged":
        img = gastropods(rng)
    elif item_id == "gastropod_cooked":
        img = gastropods(rng, cooked=True)
    elif item_id == "froglegs":
        img = froglegs(rng, "raw")
    elif item_id == "froglegs_cooked":
        img = froglegs(rng, "cooked")
    elif item_id == "froglegs_smoked":
        img = froglegs(rng, "smoked")
    elif item_id == "mutant_brain_cooked":
        img = brain_cooked(rng)
    elif item_id == "mutant_bug_organs_cooked":
        img = bug_organs_cooked(rng)
    elif item_id == "mutant_kidney":
        img = kidney(rng)
    elif item_id == "mutant_kidney_cooked":
        img = kidney(rng, cooked=True)
    elif item_id == "mutant_liver":
        img = liver(rng)
    elif item_id == "mutant_lung":
        img = lung(rng)
    elif item_id == "mutant_lung_cooked":
        img = lung(rng, cooked=True)
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
                       "preview_item08.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item08.png {sheet.size}")


if __name__ == "__main__":
    main()
