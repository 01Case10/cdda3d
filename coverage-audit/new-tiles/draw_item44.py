#!/usr/bin/env python3
"""ITEM44: raw grains (items/comestibles/raw_grain.json) -- CDDA Voxel
coverage audit.

Draws each raw-grain id in the form its description names: loose piles
of grain, threshed piles with the chaff still mixed in, tied sheaves of
stalks (fresh and dried), corn cobs, and a heap of chaff. Self-contained:
the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "barley_stalks",
    "buckwheat",
    "buckwheat_stalks",
    "canola_stalks",
    "chaff",
    "dry_barley_stalks",
    "dry_buckwheat_stalks",
    "dry_canola_stalks",
    "dry_lentil_stalks",
    "dry_oat_stalks",
    "dry_wheat_stalks",
    "dry_wild_rice_stalks",
    "dry_winter_wheat_stalks",
    "empty_corn_cob",
    "kernels",
    "oat_stalks",
    "oats",
    "popcorn3",
    "popcorn_cob",
    "popcorn_raw",
    "red_winter_wheat_stalks",
    "threshed_barley",
    "threshed_buckwheat",
    "threshed_canola",
    "threshed_lentils",
    "threshed_oats",
    "threshed_wheat",
    "threshed_winter_wheat",
    "wheat_stalks",
    "wild_rice",
    "wild_rice_stalks",
    "winter_wheat",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

CHAFF_COLS = [(216, 196, 140, 255), (198, 176, 116, 255), (228, 208, 154, 255)]


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
    """One opaque grain: darker outline, base fill, optional marking."""
    edge = shade(base, 0.55)
    d.polygon(rot_ellipse_pts(cx, cy, rx + 0.8, ry + 0.8, ang), fill=edge)
    d.polygon(rot_ellipse_pts(cx, cy, rx, ry, ang), fill=base)
    hx, hy = cx - rx * 0.3, cy - ry * 0.35
    d.polygon(rot_ellipse_pts(hx, hy, rx * 0.45, ry * 0.4, ang),
              fill=shade(base, 1.25))
    if mark == "groove":  # crease along the long axis
        ca, sa = math.cos(ang), math.sin(ang)
        x1, y1 = cx - ca * rx * 0.8, cy - sa * rx * 0.8
        x2, y2 = cx + ca * rx * 0.8, cy + sa * rx * 0.8
        d.line([x1, y1, x2, y2], fill=shade(base, 0.6), width=1)


def draw_tri(d, cx, cy, r, ang, base):
    """One buckwheat groat: a plump triangular seed."""
    pts = []
    for k in range(3):
        a = ang + k * 2.094
        pts.append((cx + r * 1.18 * math.cos(a), cy + r * 1.18 * math.sin(a)))
    d.polygon([(cx + (x - cx) * 1.22, cy + (y - cy) * 1.22) for x, y in pts],
              fill=shade(base, 0.55))
    d.polygon(pts, fill=base)
    d.point((cx - 0.5, cy - 0.5), fill=shade(base, 1.35))


def draw_disc(d, cx, cy, r, ang, base):
    """One lentil: a flat lens-shaped disc seen face-on."""
    d.polygon(rot_ellipse_pts(cx, cy, r + 0.7, r * 0.88 + 0.7, ang),
              fill=shade(base, 0.55))
    d.polygon(rot_ellipse_pts(cx, cy, r, r * 0.88, ang), fill=base)
    d.polygon(rot_ellipse_pts(cx - r * 0.25, cy - r * 0.3, r * 0.45, r * 0.38,
                              ang), fill=shade(base, 1.2))


def grain_pile(img, rng, shape, rx, ry, count, colors, mark=None, chaff=0):
    """A loose mound of grain; optional pale chaff flecks mixed in."""
    d = ImageDraw.Draw(img)
    spots = []
    for _ in range(count):
        a = rng.uniform(0, 2 * math.pi)
        r = math.sqrt(rng.uniform(0, 1))
        x = 16 + math.cos(a) * r * 11
        y = 18 + math.sin(a) * r * 6.5 - (1 - r) * 1.5
        spots.append((y, x))
    spots.sort()
    for y, x in spots:
        ang = rng.uniform(0, math.pi)
        base = rng.choice(colors)
        jr = 1 + rng.uniform(-0.15, 0.15)
        if shape == "tri":
            draw_tri(d, x, y, rx * jr, ang, base)
        elif shape == "disc":
            draw_disc(d, x, y, rx * jr, ang, base)
        else:
            draw_seed(d, x, y, rx * jr, ry * jr, ang, base, rng, mark)
    for _ in range(chaff):  # short straw flecks scattered over the mound
        a = rng.uniform(0, 2 * math.pi)
        r = math.sqrt(rng.uniform(0, 1))
        x = 16 + math.cos(a) * r * 11.5
        y = 18 + math.sin(a) * r * 7
        ang = rng.uniform(0, math.pi)
        ln = rng.uniform(2.5, 5.5)
        d.line([x, y, x + ln * math.cos(ang), y + ln * math.sin(ang)],
               fill=rng.choice(CHAFF_COLS), width=1)


def chaff_heap(img, rng):
    """Loose dried straw and seed casings, a few seed fragments mixed in."""
    d = ImageDraw.Draw(img)
    for _ in range(52):
        a = rng.uniform(0, 2 * math.pi)
        r = math.sqrt(rng.uniform(0, 1))
        x = 16 + math.cos(a) * r * 12
        y = 18 + math.sin(a) * r * 7.5 - (1 - r) * 1.5
        ang = rng.uniform(0, math.pi)
        ln = rng.uniform(3, 7)
        c = rng.choice(CHAFF_COLS + [(186, 160, 100, 255)])
        d.line([x, y, x + ln * math.cos(ang), y + ln * math.sin(ang)],
               fill=c, width=rng.choice([1, 1, 2]))
    for _ in range(9):  # broken seed fragments
        x = rng.uniform(6, 26)
        y = rng.uniform(13, 25)
        draw_seed(d, x, y, 1.6, 1.1, rng.uniform(0, 3), (150, 118, 72, 255),
                  rng)


def sheaf(img, rng, stem_cols, head_cols, kind, dry=False):
    """A tied sheaf: stems fan from a twine band, heads at the top."""
    d = ImageDraw.Draw(img)
    base = (16, 30.5)
    tops = []
    n = 9
    top_lo, top_hi = (6.0, 10.0) if kind == "barley" else (3.0, 7.0)
    for i in range(n):
        tx = 5 + i * 3.2 + rng.uniform(-1, 1)
        ty = rng.uniform(top_lo, top_hi)
        tops.append((tx, ty))
    for tx, ty in tops:  # stems (slight mid bow), edge pass then color pass
        mx = (base[0] + tx) / 2 + rng.uniform(-1.5, 1.5)
        my = (base[1] + ty) / 2
        c = rng.choice(stem_cols)
        d.line([base, (mx, my), (tx, ty)], fill=shade(c, 0.6), width=3)
        d.line([base, (mx, my), (tx, ty)], fill=c, width=2)
    if kind == "buckwheat":  # leafy non-grass: broad leaves along the stems
        for tx, ty in tops[::2]:
            for t in (0.45, 0.7):
                lx = base[0] + (tx - base[0]) * t
                ly = base[1] + (ty - base[1]) * t
                s = rng.choice([-1, 1])
                leaf = rng.choice(stem_cols)
                d.polygon([(lx, ly), (lx + 5 * s, ly - 2), (lx + 6 * s, ly + 2),
                           (lx + 2 * s, ly + 3)], fill=leaf)
    # twine band around the bundle waist
    band = (140, 104, 64, 255) if not dry else (156, 128, 84, 255)
    d.rectangle([11, 23, 21, 26], fill=band)
    d.line([11, 24.5, 21, 24.5], fill=shade(band, 0.65), width=1)
    d.line([13.5, 23, 13.5, 26], fill=shade(band, 0.75), width=1)
    d.line([18.5, 23, 18.5, 26], fill=shade(band, 0.75), width=1)
    # heads
    for tx, ty in tops:
        hc = rng.choice(head_cols)
        if kind in ("wheat", "barley"):
            awn = 8 if kind == "barley" else 5
            for k in range(5):  # grains alternate up a short rachis
                gy = ty + 4.5 - k * 1.9
                gx = tx + (1.3 if k % 2 else -1.3)
                draw_seed(d, gx, gy, 1.5, 2.4, (0.5 if k % 2 else -0.5), hc,
                          rng)
            for da in (-0.35, 0.0, 0.35):  # awns
                d.line([tx, ty - 4, tx + da * awn, ty - 4 - awn],
                       fill=shade(hc, 1.15), width=1)
        elif kind in ("oat", "wildrice"):
            for da in (-3.2, 0.0, 3.2):  # drooping panicle branches
                bx, by = tx + da, ty + 3.2
                d.line([tx, ty, bx, by], fill=shade(hc, 0.7), width=1)
                if kind == "oat":  # bell spikelet hanging at the branch tip
                    d.polygon(rot_ellipse_pts(bx, by + 1.8, 1.5, 2.3, 0.2),
                              fill=hc)
                    d.point((bx, by + 0.5), fill=shade(hc, 0.6))
                else:  # long thin dark grain angled down the branch
                    draw_seed(d, bx, by + 1.6, 2.6, 0.9, 1.2 + da * 0.1, hc,
                              rng, "groove")
        elif kind == "buckwheat":
            for _ in range(4):  # seed cluster + pale blossom dots at the tip
                gx = tx + rng.uniform(-2, 2)
                gy = ty + rng.uniform(-1, 2.5)
                draw_tri(d, gx, gy, 1.25, rng.uniform(0, 3),
                         rng.choice([(70, 52, 38, 255), (88, 64, 46, 255)]))
            for _ in range(3):
                d.point((tx + rng.uniform(-2.5, 2.5), ty + rng.uniform(-2, 1)),
                        fill=(240, 226, 220, 255))
        elif kind == "canola":
            for k in range(3):  # slender pods angle up from the upper stem
                t = 0.55 + k * 0.16
                px = base[0] + (tx - base[0]) * t
                py = base[1] + (ty - base[1]) * t
                s = 1 if k % 2 else -1
                pc = rng.choice(head_cols)
                d.line([px, py, px + 3.5 * s, py - 5.5], fill=shade(pc, 0.6),
                       width=3)
                d.line([px, py, px + 3.5 * s, py - 5.5], fill=pc, width=2)
            if not dry:  # a couple of yellow blossoms at the tip
                d.point((tx, ty - 1), fill=(238, 206, 74, 255))
                d.point((tx + 1, ty), fill=(238, 206, 74, 255))


def lentil_heap(img, rng):
    """Short dried stalks in a low pile, pods with lentils rattling in them."""
    d = ImageDraw.Draw(img)
    stem = [(188, 164, 108, 255), (172, 148, 94, 255)]
    for _ in range(16):
        x = rng.uniform(5, 24)
        y = rng.uniform(14, 26)
        ang = rng.uniform(-0.5, 0.5)
        ln = rng.uniform(6, 10)
        d.line([x, y, x + ln * math.cos(ang), y + ln * math.sin(ang)],
               fill=rng.choice(stem), width=2)
    for _ in range(8):  # small oval pods, a lentil dot showing in each
        x = rng.uniform(7, 25)
        y = rng.uniform(13, 25)
        pod = rng.choice([(206, 182, 124, 255), (192, 166, 108, 255)])
        d.polygon(rot_ellipse_pts(x, y, 2.6, 1.5, rng.uniform(0, 3)),
                  fill=shade(pod, 0.6))
        d.polygon(rot_ellipse_pts(x, y, 2.2, 1.2, rng.uniform(0, 3)),
                  fill=pod)
        d.point((x, y), fill=(150, 100, 58, 255))


def corn_cob(img, rng, kernel_cols, husk_cols, empty=False):
    """A vertical corn cob in kernel rows, husk leaves at the base."""
    d = ImageDraw.Draw(img)
    if husk_cols:  # husk leaves sweep out from the base
        for s, hc in ((-1, husk_cols[0]), (1, husk_cols[-1])):
            d.polygon([(16, 26), (16 + 9 * s, 31), (16 + 3 * s, 24),
                       (16 + 1 * s, 28)], fill=shade(hc, 0.7))
            d.polygon([(16, 26), (16 + 8 * s, 30), (16 + 3 * s, 24)],
                      fill=hc)
    # pith body, slightly tapered toward the tip
    d.polygon([(11.5, 27), (12.5, 6), (14, 4), (18, 4), (19.5, 6),
               (20.5, 27)], fill=(233, 221, 182, 255),
              outline=(186, 168, 120, 255))
    rows = 8
    for r in range(rows):
        y = 7.5 + r * 2.5
        halfw = 4.3 - (r / rows) * 0.7 if r < 2 else 4.3 - ((rows - r) / rows) * 0.4
        xs = [16 - halfw + 1.6, 16 - halfw / 2 + 0.8, 16 + 0.8,
              16 + halfw / 2 + 0.8, 16 + halfw - 1.6]
        if r % 2:
            xs = [x + 0.8 for x in xs]
        for x in xs:
            if empty:  # bare socket: pale diamond with a darker pit
                d.polygon([(x, y - 1.7), (x + 1.7, y), (x, y + 1.7),
                           (x - 1.7, y)], fill=(206, 188, 142, 255))
                d.ellipse([x - 0.8, y - 0.8, x + 0.8, y + 0.8],
                          fill=(150, 128, 84, 255))
            else:
                kc = rng.choice(kernel_cols)
                d.polygon(rot_ellipse_pts(x, y, 1.55, 1.45, 0), fill=shade(kc, 0.6))
                d.polygon(rot_ellipse_pts(x, y, 1.3, 1.2, 0), fill=kc)
                d.point((x - 0.4, y - 0.4), fill=shade(kc, 1.3))


def popped_pile(img, rng):
    """Buttered popcorn: fluffy white puffs, butter tint, hull flecks."""
    d = ImageDraw.Draw(img)
    spots = []
    for _ in range(10):
        a = rng.uniform(0, 2 * math.pi)
        r = math.sqrt(rng.uniform(0, 1))
        spots.append((18 + math.sin(a) * r * 6, 16 + math.cos(a) * r * 9.5))
    spots.sort()
    for y, x in spots:
        puff = (246, 242, 230, 255)
        for dx, dy, rr in ((-1.8, 0.4, 2.4), (1.8, 0.2, 2.5), (0, -1.8, 2.6)):
            d.ellipse([x + dx - rr, y + dy - rr, x + dx + rr, y + dy + rr],
                      fill=puff)
        d.ellipse([x - 1.2, y - 2.6, x + 1.4, y - 0.6],
                  fill=(244, 208, 110, 255))  # butter sheen
        d.ellipse([x - 0.9, y + 0.8, x + 0.9, y + 2.0],
                  fill=(176, 120, 50, 255))  # hull at the heart
        d.point((x - 1.5, y - 1), fill=(232, 222, 196, 255))


# loose-grain profiles: (shape, rx, ry, count, colors, mark, chaff)
P = {
    "buckwheat": ("tri", 2.5, 0, 16, [(96, 70, 48, 255), (80, 58, 40, 255)], None, 0),
    "oats": ("ellipse", 3.8, 1.4, 15, [(198, 172, 120, 255), (180, 152, 98, 255)], "groove", 0),
    "wild_rice": ("ellipse", 4.4, 1.3, 14, [(104, 80, 54, 255), (88, 66, 42, 255)], "groove", 0),
    "winter_wheat": ("ellipse", 3.3, 1.8, 16, [(190, 140, 76, 255), (172, 124, 64, 255)], "groove", 0),
    "kernels": ("ellipse", 2.8, 2.1, 19, [(236, 172, 60, 255), (214, 146, 44, 255)], None, 0),
    "threshed_barley": ("ellipse", 3.6, 1.5, 14, [(208, 180, 130, 255), (190, 160, 106, 255)], "groove", 18),
    "threshed_buckwheat": ("tri", 2.5, 0, 14, [(96, 70, 48, 255), (80, 58, 40, 255)], None, 18),
    "threshed_canola": ("ellipse", 1.5, 1.5, 30, [(36, 34, 36, 255), (54, 52, 54, 255)], None, 24),
    "threshed_lentils": ("disc", 2.1, 0, 16, [(178, 120, 66, 255), (140, 128, 74, 255)], None, 16),
    "threshed_oats": ("ellipse", 3.8, 1.4, 14, [(198, 172, 120, 255), (180, 152, 98, 255)], "groove", 18),
    "threshed_wheat": ("ellipse", 3.3, 1.8, 15, [(196, 150, 84, 255), (178, 132, 68, 255)], "groove", 18),
    "threshed_winter_wheat": ("ellipse", 3.3, 1.8, 15, [(176, 118, 62, 255), (158, 102, 50, 255)], "groove", 18),
}

FRESH_STEM = [(150, 160, 84, 255), (138, 150, 76, 255)]
DRY_STEM = [(198, 172, 110, 255), (184, 156, 96, 255)]

# sheaf profiles: (stem colors, head colors, head kind, dry)
S = {
    "wheat_stalks": (FRESH_STEM, [(196, 168, 96, 255), (182, 154, 84, 255)], "wheat", False),
    "barley_stalks": (FRESH_STEM, [(208, 186, 122, 255), (194, 170, 104, 255)], "barley", False),
    "oat_stalks": (FRESH_STEM, [(204, 182, 128, 255), (190, 166, 108, 255)], "oat", False),
    "red_winter_wheat_stalks": (FRESH_STEM, [(186, 128, 70, 255), (170, 114, 60, 255)], "wheat", False),
    "buckwheat_stalks": ([(110, 142, 66, 255), (96, 128, 58, 255)], [(70, 52, 38, 255)], "buckwheat", False),
    "canola_stalks": (FRESH_STEM, [(128, 158, 84, 255), (114, 144, 74, 255)], "canola", False),
    "wild_rice_stalks": ([(140, 152, 88, 255), (128, 140, 78, 255)], [(92, 70, 46, 255), (78, 58, 38, 255)], "wildrice", False),
    "dry_wheat_stalks": (DRY_STEM, [(210, 184, 118, 255), (196, 168, 102, 255)], "wheat", True),
    "dry_barley_stalks": (DRY_STEM, [(218, 196, 136, 255), (204, 180, 118, 255)], "barley", True),
    "dry_oat_stalks": (DRY_STEM, [(214, 192, 140, 255), (200, 176, 120, 255)], "oat", True),
    "dry_winter_wheat_stalks": (DRY_STEM, [(196, 142, 80, 255), (182, 128, 70, 255)], "wheat", True),
    "dry_canola_stalks": (DRY_STEM, [(186, 164, 106, 255), (172, 150, 92, 255)], "canola", True),
    "dry_buckwheat_stalks": ([(168, 138, 84, 255), (154, 124, 72, 255)], [(70, 52, 38, 255)], "buckwheat", True),
    "dry_wild_rice_stalks": (DRY_STEM, [(92, 70, 46, 255), (78, 58, 38, 255)], "wildrice", True),
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    if sid in P:
        shape, rx, ry, count, colors, mark, chaff = P[sid]
        grain_pile(img, rng, shape, rx, ry, count, colors, mark, chaff)
    elif sid in S:
        stem_cols, head_cols, kind, dry = S[sid]
        sheaf(img, rng, stem_cols, head_cols, kind, dry)
    elif sid == "chaff":
        chaff_heap(img, rng)
    elif sid == "dry_lentil_stalks":
        lentil_heap(img, rng)
    elif sid == "popcorn_cob":
        corn_cob(img, rng, [(232, 176, 66, 255), (216, 152, 48, 255),
                            (240, 196, 92, 255)],
                 [(168, 142, 84, 255), (150, 124, 70, 255)])
    elif sid == "popcorn_raw":
        corn_cob(img, rng, [(238, 228, 190, 255), (226, 214, 168, 255)],
                 [(140, 178, 92, 255), (120, 160, 78, 255)])
    elif sid == "empty_corn_cob":
        corn_cob(img, rng, [], [(196, 172, 110, 255)], empty=True)
    elif sid == "popcorn3":
        popped_pile(img, rng)
    else:
        raise KeyError(sid)
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
                       "preview_item44.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item44.png {prev.size}")


if __name__ == "__main__":
    main()
