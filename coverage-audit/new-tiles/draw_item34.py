#!/usr/bin/env python3
"""Batch ITEM34 (Phase 2g.4 scope extension): shell casings.

43 ids (authoritative list embedded below; source list:
~/workspace/item-batches/34.txt). All 43 are defined in
data/json/items/generic/casing.json. Entries read first,
copy-from chains resolved: nearly everything copy-froms the
`casing` abstract (brass, yellow, symbol '='); the shotgun
hulls copy-from `shot_hull` (red plastic tube, brass head);
atgm_spent is steel/dark_gray; caotel_cell_spent is
qt_steel.

Treatment -- one parameterized family renderer. Each brass
casing is a small pile: a hero case in profile (proportions
from the real cartridge: case length and base diameter in
mm, bottleneck / straight / rimmed / belted profile),
a smaller case behind, one case lying in front, and one
case seen base-on so the primer state reads: fired casings
show a dented primer and slight soot at the mouth on duller
brass; the `unprimed_*` (primerless) variants show an empty
dark primer pocket on brighter, cleaner brass. Caliber
must read differently: a squat 9mm pile is not a towering
.50 BMG case, a red 12-gauge hull is not a 20-gauge hull.

Singles drawn as themselves:
- 273x110 / 273x44 Diyua: giant all-brass alien shotshells,
  wide straight tubes; the base disc carries a ring of
  illegible alien glyph ticks around the primer (the
  headstamp from the description). 110 mm is huge, 44 mm
  merely large.
- 33_naval_casing: 100 ml, "could be used as a cup" -- one
  big brass case, wide soot-streaked mouth, heat-tinted
  band below the rim, burns and soot per the description.
- atgm_spent: a 152 mm ATGM launch tube, 7.5 L of empty
  steel pipe -- one big dark-gray tube diagonal across the
  tile, hollow muzzle ring, reinforcing bands.
- caotel_cell_spent: a small metallic cube of inscrutable
  construction (qt_steel) -- a blue-gray cube with panel
  seams, a circular port, and one dim indicator (spent =
  slightly lighter, light out).

Writes ONLY its own 43 folders + preview_item34.png.
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")

CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.\n")

IDS = [
    "20shot_hull",
    "223_casing",
    "270win_casing",
    "273x110casing",
    "273x44casing",
    "3006_casing",
    "300_casing",
    "300blk_casing",
    "3030_casing",
    "303_casing",
    "308_casing",
    "30carbine_casing",
    "338lapua_casing",
    "33_naval_casing",
    "450_casing",
    "458wm_casing",
    "50_casing",
    "50beowulf_casing",
    "77arisaka_casing",
    "atgm_spent",
    "caotel_cell_spent",
    "unprimed_123ln_casing",
    "unprimed_20shot_hull",
    "unprimed_223_casing",
    "unprimed_270win_casing",
    "unprimed_3006_casing",
    "unprimed_300_casing",
    "unprimed_300blk_casing",
    "unprimed_3030_casing",
    "unprimed_303_casing",
    "unprimed_308_casing",
    "unprimed_30carbine_casing",
    "unprimed_338lapua_casing",
    "unprimed_450_casing",
    "unprimed_458wm_casing",
    "unprimed_50_casing",
    "unprimed_50beowulf_casing",
    "unprimed_762R_casing",
    "unprimed_762_51_casing",
    "unprimed_762_casing",
    "unprimed_77arisaka_casing",
    "unprimed_9mm_casing",
    "unprimed_shot_hull",
]

# Brass palettes: (outline, dark, mid, light, highlight).
BRASS_FIRED = ((94, 66, 20, 255), (146, 102, 32, 255),
               (192, 146, 56, 255), (226, 182, 88, 255),
               (242, 206, 122, 255))
BRASS_CLEAN = ((100, 70, 22, 255), (160, 116, 38, 255),
               (214, 168, 68, 255), (240, 202, 108, 255),
               (252, 226, 148, 255))
MOUTH = (36, 26, 14, 255)
SOOT = (34, 27, 18, 255)
PRIMER = (206, 204, 208, 255)
PRIMER_DARK = (120, 118, 122, 255)
DENT = (38, 30, 22, 255)
POCKET = (28, 20, 12, 255)

# Red shotgun-hull plastic: (outline, dark, mid, light).
RED = ((108, 26, 22, 255), (156, 40, 34, 255),
       (198, 58, 46, 255), (230, 96, 74, 255))


def shade(d, x0, y0, x1, y1, pal):
    """Vertical 4-band shading across [x0, x1): light core,
    mid body, dark right edge, outline on both edges."""
    w = x1 - x0
    for x in range(x0, x1):
        t = (x - x0) / max(1, w - 1)
        if x == x0 or x == x1 - 1:
            c = pal[0]
        elif t < 0.30:
            c = pal[3]
        elif t < 0.62:
            c = pal[2]
        else:
            c = pal[1]
        d.line([(x, y0), (x, y1)], fill=c)


def base_disc(d, cx, cy, r, fired, pal, glyphs=False):
    """A case seen base-on: brass disc; center shows a
    dented primer (fired) or an empty pocket (primerless).
    Diyua discs get a ring of alien glyph ticks."""
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=pal[0])
    d.ellipse([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1],
              fill=pal[2])
    if glyphs:
        n = 7
        for i in range(n):
            a = 2 * math.pi * i / n
            gx = cx + int(round((r - 1.6) * math.cos(a)))
            gy = cy + int(round((r - 1.6) * math.sin(a)))
            d.point((gx, gy), fill=pal[0])
    pr = max(1, r // 2)
    if fired:
        d.ellipse([cx - pr, cy - pr, cx + pr, cy + pr], fill=PRIMER)
        d.ellipse([cx - pr + 1, cy - pr + 1, cx + pr - 1, cy + pr - 1],
                  fill=PRIMER_DARK)
        d.point((cx, cy), fill=DENT)
    else:
        d.ellipse([cx - pr, cy - pr, cx + pr, cy + pr], fill=POCKET)
        d.point((cx, cy), fill=(12, 8, 4, 255))


def case_profile(d, cx, base_y, h, w, style, pal, fired):
    """One upright case in profile, base at base_y, centered
    on cx. style: 'bneck', 'straight', 'rim', 'belt'."""
    top = base_y - h
    half = w / 2.0
    x0 = int(round(cx - half))
    x1 = int(round(cx + half))
    neck_half = half * (0.46 if style == 'bneck' else 0.88)
    nx0 = int(round(cx - neck_half))
    nx1 = int(round(cx + neck_half))
    # Mouth: dark open top with a brass rim line.
    d.rectangle([nx0, top, nx1 - 1, top + 1], fill=pal[0])
    d.rectangle([nx0 + 1, top + 1, nx1 - 2, top + 1], fill=MOUTH)
    if style == 'bneck':
        shoulder_y = top + max(3, int(h * 0.30))
        # Neck.
        shade(d, nx0, top + 2, nx1, shoulder_y, pal)
        # Shoulder: taper rows from neck width to body width.
        steps = max(2, int(h * 0.16))
        for i in range(steps):
            t = i / steps
            sx0 = int(round(nx0 - (nx0 - x0) * t))
            sx1 = int(round(nx1 + (x1 - nx1) * t))
            shade(d, sx0, shoulder_y + i, sx1, shoulder_y + i, pal)
        body_top = shoulder_y + steps
    else:
        # Straight wall with a whisper of taper.
        steps = h - 5
        for i in range(steps):
            t = i / max(1, steps)
            sx0 = int(round(nx0 - (nx0 - x0) * t))
            sx1 = int(round(nx1 + (x1 - nx1) * t))
            shade(d, sx0, top + 2 + i, sx1, top + 2 + i, pal)
        body_top = base_y - 4
    # Body down to the extractor groove.
    shade(d, x0, body_top, x1, base_y - 3, pal)
    # Extractor groove + rim/head.
    d.rectangle([x0 + 1, base_y - 3, x1 - 2, base_y - 3], fill=pal[1])
    if style == 'rim':
        d.rectangle([x0 - 1, base_y - 2, x1, base_y - 1], fill=pal[0])
        shade(d, x0 - 1, base_y - 2, x1, base_y - 1, pal)
    elif style == 'belt':
        d.rectangle([x0 - 1, base_y - 5, x1, base_y - 4], fill=pal[0])
        shade(d, x0 - 1, base_y - 5, x1, base_y - 4, pal)
        shade(d, x0, base_y - 2, x1, base_y - 1, pal)
    else:
        shade(d, x0, base_y - 2, x1, base_y - 1, pal)
    d.rectangle([x0, base_y - 1, x1 - 1, base_y - 1], fill=pal[0])
    if fired:
        # Soot smudge just below the mouth.
        sy = top + 2
        d.line([(nx0 + 1, sy), (nx1 - 2, sy)], fill=SOOT)


def lying_case(d, x, y, length, w, style, pal):
    """One case lying on its side, base at left, mouth at
    right; y is the centerline."""
    lh = max(3, w - 2)
    y0, y1 = y - lh // 2, y + (lh - lh // 2)
    # Base cap + rim.
    d.rectangle([x, y0 - 1, x + 1, y1], fill=pal[0])
    d.rectangle([x + 1, y0, x + 2, y1 - 1], fill=pal[1])
    body_x1 = x + length - (4 if style == 'bneck' else 1)
    d.rectangle([x + 2, y0, body_x1, y1 - 1], fill=pal[2])
    d.line([(x + 2, y0), (body_x1, y0)], fill=pal[3])
    d.line([(x + 2, y1 - 1), (body_x1, y1 - 1)], fill=pal[1])
    if style == 'bneck':
        d.rectangle([body_x1, y0 + 1, x + length - 1, y1 - 2],
                    fill=pal[2])
    d.rectangle([x + length - 1, y0 + 1, x + length - 1, y1 - 2],
                fill=MOUTH)


def brass_pile(rng, length_mm, dia_mm, style, fired):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pal = BRASS_FIRED if fired else BRASS_CLEAN
    h = int(round(9 + length_mm * 0.19))
    w = max(6, int(round(3.5 + dia_mm * 0.5)))
    base_y = 29
    jx = rng.choice([-1, 0, 0, 1])
    if h >= 26:  # .50 BMG: the hero case fills the tile.
        case_profile(d, 14 + jx, base_y, h, w, style, pal, fired)
        lying_case(d, 3, 29, 15, w - 3, style, pal)
        base_disc(d, 26, 24, 4, fired, pal)
    elif h <= 15:  # 9mm / .30 Carbine: a scatter of small brass.
        case_profile(d, 20 + jx, base_y, h, w, style, pal, fired)
        case_profile(d, 11, base_y, h - 1, w - 1, style, pal, fired)
        case_profile(d, 15 - jx, base_y - 4, h - 2, w - 1, style,
                     pal, fired)
        lying_case(d, 4, 29, 13, w, style, pal)
        base_disc(d, 27, 26, 3, fired, pal)
    else:
        case_profile(d, 12, base_y, int(h * 0.74), w - 1, style,
                     pal, fired)
        case_profile(d, 21 + jx, base_y, h, w, style, pal, fired)
        lying_case(d, 3, 29, 16, w - 1, style, pal)
        base_disc(d, 27, 25, max(3, w // 2 - 1), fired, pal)
    return img


# (length mm, base dia mm, profile style) per caliber; the
# fired and primerless entries share the caliber's dims.
CALIBERS = {
    "223": (44.7, 9.6, 'bneck'),
    "270win": (64.5, 12.0, 'bneck'),
    "3006": (63.5, 12.0, 'bneck'),
    "300": (66.5, 13.5, 'belt'),
    "300blk": (36.8, 9.6, 'bneck'),
    "3030": (51.8, 12.7, 'rim'),
    "303": (56.4, 13.5, 'rim'),
    "308": (51.2, 12.0, 'bneck'),
    "30carbine": (32.8, 9.1, 'straight'),
    "338lapua": (69.2, 14.9, 'bneck'),
    "450": (43.2, 12.7, 'straight'),
    "458wm": (63.5, 13.5, 'belt'),
    "50": (99.1, 20.4, 'bneck'),
    "50beowulf": (42.0, 13.7, 'straight'),
    "77arisaka": (58.0, 12.1, 'bneck'),
    "123ln": (56.0, 13.0, 'bneck'),
    "762R": (53.6, 13.5, 'rim'),
    "762_51": (51.2, 12.0, 'bneck'),
    "762": (38.7, 11.4, 'bneck'),
    "9mm": (19.0, 9.9, 'straight'),
}


def caliber_of(item_id):
    key = item_id
    if key.startswith("unprimed_"):
        key = key[len("unprimed_"):]
    return key[:-len("_casing")]


def draw_brass(item_id):
    def go(rng):
        L, D, style = CALIBERS[caliber_of(item_id)]
        return brass_pile(rng, L, D, style,
                          fired=not item_id.startswith("unprimed_"))
    return go


def hull(rng, dia_px, h, fired):
    """One upright shotgun hull: red plastic tube, brass
    head, open mouth; base disc in front for primer state."""
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pal = BRASS_FIRED if fired else BRASS_CLEAN
    base_y = 29

    def one(cx, hh, w):
        top = base_y - hh
        x0, x1 = cx - w // 2, cx + w // 2
        head_h = max(4, hh // 4)
        # Flared open mouth + dark interior.
        d.rectangle([x0 - 1, top, x1, top + 1], fill=RED[0])
        d.rectangle([x0, top + 1, x1 - 1, top + 1], fill=MOUTH)
        shade(d, x0, top + 2, x1, base_y - head_h, RED)
        # Brass head.
        shade(d, x0, base_y - head_h, x1, base_y, pal)
        d.rectangle([x0, base_y - head_h, x1 - 1, base_y - head_h],
                    fill=pal[0])
        d.rectangle([x0, base_y, x1 - 1, base_y], fill=pal[0])

    one(11, int(h * 0.8), dia_px - 1)
    one(20, h, dia_px)
    base_disc(d, 27, 25, max(3, dia_px // 2 - 2), fired, pal)
    return img


def draw_diyua(length_mm):
    def go(rng):
        img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        pal = BRASS_FIRED
        h = int(round(9 + length_mm * 0.155))
        w = 15
        base_y = 29
        if length_mm < 60:
            case_profile(d, 11, base_y, h - 3, w - 2, 'straight',
                         pal, True)
        # Hero: a giant straight brass shotshell -- thick
        # rimmed head band carrying glyph ticks.
        case_profile(d, 19, base_y, h, w, 'rim', pal, True)
        top = base_y - h
        head_top = base_y - 6
        for gx in range(19 - w // 2 + 2, 19 + w // 2 - 1, 3):
            d.line([(gx, head_top + 1), (gx, head_top + 3)],
                   fill=pal[0])
        d.rectangle([19 - w // 2 + 1, top + 1, 19 + w // 2 - 2,
                     top + 1], fill=MOUTH)
        base_disc(d, 27 if length_mm < 60 else 6, 10, 5, True,
                  pal, glyphs=True)
        return img
    return go


def draw_naval(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pal = BRASS_FIRED
    # One cup-sized case, soot and heat tint at the mouth.
    case_profile(d, 15, 29, 23, 16, 'straight', pal, True)
    top = 29 - 23
    d.rectangle([8, top + 2, 21, top + 4], fill=(96, 74, 58, 255))
    d.rectangle([9, top + 1, 20, top + 1], fill=MOUTH)
    for sx, ln in ((10, 6), (14, 8), (18, 5), (12, 4), (20, 7)):
        d.line([(sx, top + 3), (sx, top + 3 + ln)], fill=SOOT)
    base_disc(d, 26, 25, 4, True, pal)
    return img


def draw_atgm(rng):
    layer = Image.new("RGBA", (44, 22), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    outl = (38, 40, 44, 255)
    dark = (82, 86, 92, 255)
    mid = (118, 124, 132, 255)
    light = (166, 172, 180, 255)
    # Tube body, muzzle at right.
    d.rectangle([3, 5, 36, 16], fill=mid)
    d.rectangle([3, 5, 36, 7], fill=light)
    d.rectangle([3, 14, 36, 16], fill=dark)
    d.rectangle([3, 5, 4, 16], fill=outl)          # rear cap
    d.rectangle([12, 5, 13, 16], fill=dark)        # band
    d.rectangle([24, 5, 25, 16], fill=dark)        # band
    # Hollow muzzle: ring + black bore.
    d.ellipse([33, 3, 43, 18], fill=outl)
    d.ellipse([34, 4, 42, 17], fill=light)
    d.ellipse([36, 6, 40, 15], fill=(14, 14, 16, 255))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    img.alpha_composite(layer.rotate(-22, resample=Image.NEAREST,
                                     expand=True), (-4, 3))
    return img


def draw_caotel(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    outl = (50, 62, 76, 255)
    dark = (94, 114, 136, 255)
    mid = (130, 150, 170, 255)
    light = (178, 198, 216, 255)
    # Isometric cube: top, front, right faces.
    d.polygon([(9, 13), (14, 7), (27, 7), (22, 13)], fill=light)
    d.polygon([(22, 13), (27, 7), (27, 21), (22, 27)], fill=dark)
    d.rectangle([9, 13, 22, 27], fill=mid)
    d.line([(9, 13), (22, 13)], fill=outl)
    d.line([(9, 27), (22, 27)], fill=outl)
    d.line([(22, 13), (22, 27)], fill=outl)
    # Panel seams + a circular port on the front face.
    d.line([(9, 20), (22, 20)], fill=dark)
    d.ellipse([13, 15, 19, 21], fill=outl)
    d.ellipse([14, 16, 18, 20], fill=(22, 28, 36, 255))
    d.point((16, 18), fill=light)
    # Spent: the indicator is dark, one seam scuffed.
    d.rectangle([20, 24, 21, 25], fill=(70, 52, 22, 255))
    d.line([(10, 25), (13, 23)], fill=dark)
    return img


RENDER = {}
for _id in IDS:
    if _id in ("20shot_hull", "unprimed_20shot_hull"):
        RENDER[_id] = (lambda f: (lambda rng: hull(rng, 11, 19, f)))(
            not _id.startswith("unprimed_"))
    elif _id == "unprimed_shot_hull":
        RENDER[_id] = lambda rng: hull(rng, 13, 20, False)
    elif _id == "273x110casing":
        RENDER[_id] = draw_diyua(110)
    elif _id == "273x44casing":
        RENDER[_id] = draw_diyua(44)
    elif _id == "33_naval_casing":
        RENDER[_id] = draw_naval
    elif _id == "atgm_spent":
        RENDER[_id] = draw_atgm
    elif _id == "caotel_cell_spent":
        RENDER[_id] = draw_caotel
    else:
        RENDER[_id] = draw_brass(_id)


def main():
    assert set(RENDER) == set(IDS), set(IDS) ^ set(RENDER)
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = RENDER[item_id](rng)
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
    sheet = Image.new("RGBA", (cols * cell, rows * cell),
                      (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item34.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item34.png {sheet.size}")


if __name__ == "__main__":
    main()
