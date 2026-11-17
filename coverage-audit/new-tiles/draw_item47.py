#!/usr/bin/env python3
"""ITEM47: brigandine gloves + shoulder guards -- CDDA Voxel
coverage audit.

Draws each item as a pair on transparent background.  Two types:

  *_brigandine_hands -- a pair of canvas gloves (tough fabric base
    with stitch lines), steel plates of varied size/shape riveted
    over the back of the hand and over the finger segments.
  *_shoulder_guards  -- a pair of curved/dome steel pauldrons:
    layered lame bands following the dome curve, rivets, and a
    leather/canvas attachment strap below.

Steel grades by prefix (replace_materials on an lc_steel base):
  lc mild     -- light matte gray, low contrast, no glint
  mc medium   -- mid gray, moderate shading
  hc high     -- bright polished, strong white highlight
  ch hardened -- dark blue-gray
  qt tempered -- mid steel with a heat-tinted sheen band
                 (straw / blue / purple) across the steel

Sizes: plain = standard, _xs = undersize (0.75x, noticeably
smaller in frame), xl_ prefix = oversize (1.125x, noticeably
larger in frame).  Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "ch_brigandine_hands",
    "ch_brigandine_hands_xs",
    "ch_shoulder_guards",
    "ch_shoulder_guards_xs",
    "hc_brigandine_hands",
    "hc_brigandine_hands_xs",
    "hc_shoulder_guards",
    "hc_shoulder_guards_xs",
    "lc_brigandine_hands",
    "lc_brigandine_hands_xs",
    "lc_shoulder_guards",
    "lc_shoulder_guards_xs",
    "mc_brigandine_hands",
    "mc_brigandine_hands_xs",
    "mc_shoulder_guards",
    "mc_shoulder_guards_xs",
    "qt_brigandine_hands",
    "qt_brigandine_hands_xs",
    "qt_shoulder_guards",
    "qt_shoulder_guards_xs",
    "xl_ch_brigandine_hands",
    "xl_ch_shoulder_guards",
    "xl_hc_brigandine_hands",
    "xl_hc_shoulder_guards",
    "xl_lc_brigandine_hands",
    "xl_lc_shoulder_guards",
    "xl_mc_brigandine_hands",
    "xl_mc_shoulder_guards",
    "xl_qt_brigandine_hands",
    "xl_qt_shoulder_guards",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

# steel finishes: (base, dark, light)
STEELS = {
    "lc": ((168, 172, 176, 255), (124, 128, 132, 255), (196, 200, 204, 255)),
    "mc": ((128, 132, 138, 255), (84, 88, 96, 255), (170, 174, 180, 255)),
    "hc": ((196, 202, 212, 255), (108, 116, 128, 255), (248, 250, 252, 255)),
    "ch": ((74, 86, 106, 255), (42, 52, 70, 255), (118, 134, 158, 255)),
    "qt": ((138, 134, 148, 255), (88, 84, 102, 255), (186, 180, 196, 255)),
}
# heat-tint sheen colours for tempered steel
TINT_STRAW = (216, 178, 92, 255)
TINT_BLUE = (84, 110, 190, 255)
TINT_PURPLE = (150, 92, 180, 255)

CANVAS = (140, 124, 84, 255)
CANVAS_DARK = (100, 88, 58, 255)
CANVAS_LIGHT = (174, 160, 116, 255)
STITCH = (84, 72, 44, 255)
LEATHER = (110, 78, 52, 255)
LEATHER_DARK = (76, 52, 34, 255)
RIVET_DARK = (38, 38, 42, 255)
RIVET_LIGHT = (232, 234, 238, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def parse(sid):
    rest = sid
    size = "standard"
    if rest.startswith("xl_"):
        size = "xl"
        rest = rest[3:]
    if rest.endswith("_xs"):
        size = "xs"
        rest = rest[:-3]
    grade = rest.split("_")[0]
    kind = "hands" if "brigandine_hands" in rest else "shoulders"
    return grade, kind, size


def rivet(d, x, y, grade):
    """One rivet: dark dot with a tiny light catch on top-left."""
    d.point((x, y), fill=RIVET_DARK)
    if grade == "hc":
        d.point((x, y), fill=RIVET_LIGHT)


def tint_band(d, x0, x1, y):
    """Heat-tinted sheen band across steel: straw -> purple -> blue."""
    w = x1 - x0 + 1
    third = max(1, w // 3)
    d.line([x0, y, x0 + third - 1, y], fill=TINT_STRAW, width=1)
    d.line([x0 + third, y, x0 + 2 * third - 1, y], fill=TINT_PURPLE, width=1)
    d.line([x0 + 2 * third, y, x1, y], fill=TINT_BLUE, width=1)
    d.line([x0, y + 1, x1, y + 1], fill=TINT_BLUE, width=1)


# ------------------------------------------------------------------ gloves

def single_glove(d, rng, cx, top, gw, gh, grade, flip):
    steel, steel_dark, steel_light = STEELS[grade]
    x0 = cx - gw // 2
    x1 = x0 + gw - 1
    finger_h = max(3, gh // 4)
    palm_top = top + finger_h
    bottom = top + gh - 1
    # fingers: 4 canvas fingers, middle ones longer
    fw = max(2, gw // 4)
    finger_tops = []
    for i in range(4):
        fx0 = x0 + i * fw
        fx1 = fx0 + fw - 1 if i < 3 else x1
        extra = 1 if i in (1, 2) else 0
        ftop = top if extra else top + 1
        finger_tops.append((fx0, fx1, ftop))
        d.rectangle([fx0, ftop, fx1, palm_top], fill=CANVAS)
        d.line([fx0, ftop, fx0, palm_top], fill=CANVAS_DARK, width=1)
        # finger steel segment plate (varied height by rng)
        ph = 2 if rng.random() < 0.5 else 3
        d.rectangle([fx0, ftop + 1, fx1, min(fx1 and ftop + ph, palm_top - 1)],
                    fill=steel)
        d.line([fx0, ftop + 1, fx1, ftop + 1], fill=steel_light, width=1)
        rivet(d, fx0, ftop + 1, grade)
    # thumb: small canvas wedge on the inner side
    if flip:  # right glove: thumb on its left
        d.rectangle([x0 - 2, palm_top + 3, x0 - 1, palm_top + 6],
                    fill=CANVAS)
        d.rectangle([x0 - 2, palm_top + 3, x0 - 1, palm_top + 4],
                    fill=steel)
    else:  # left glove: thumb on its right
        d.rectangle([x1 + 1, palm_top + 3, x1 + 2, palm_top + 6],
                    fill=CANVAS)
        d.rectangle([x1 + 1, palm_top + 3, x1 + 2, palm_top + 4],
                    fill=steel)
    # palm / back-of-hand canvas body
    d.rectangle([x0, palm_top, x1, bottom], fill=CANVAS)
    d.line([x0, palm_top, x0, bottom], fill=CANVAS_DARK, width=1)
    d.line([x1, palm_top, x1, bottom], fill=CANVAS_DARK, width=1)
    d.line([x0, bottom, x1, bottom], fill=CANVAS_DARK, width=1)
    # cuff shading at the wrist
    d.line([x0, bottom - 2, x1, bottom - 2], fill=CANVAS_DARK, width=1)
    d.line([x0, bottom - 1, x1, bottom - 1], fill=CANVAS_LIGHT, width=1)
    # stitch lines down both sides of the palm
    for y in range(palm_top + 2, bottom - 2, 3):
        d.point((x0 + 1, y), fill=STITCH)
        d.point((x1 - 1, y), fill=STITCH)
    # main back plate: large riveted plate, shape varies by rng
    inset = 1
    px0, px1 = x0 + inset, x1 - inset
    py0 = palm_top + 1
    py1 = palm_top + max(4, (bottom - palm_top) // 2)
    # a notch / split variant on some gloves
    split = rng.random() < 0.4
    d.rectangle([px0, py0, px1, py1], fill=steel)
    d.line([px0, py0, px1, py0], fill=steel_light, width=1)
    d.line([px0, py1, px1, py1], fill=steel_dark, width=1)
    if split:
        mid = (px0 + px1) // 2
        d.line([mid, py0, mid, py1], fill=steel_dark, width=1)
    if grade == "hc":  # polished glint down the plate
        d.line([px0 + 1, py0 + 1, px0 + 1, py1 - 1],
               fill=(255, 255, 255, 255), width=1)
    if grade == "qt":  # heat-tinted sheen across the plate
        tint_band(d, px0, px1, py0 + 2)
    # rivets at the main plate corners
    for rx, ry in ((px0, py0), (px1, py0), (px0, py1), (px1, py1)):
        rivet(d, rx, ry, grade)
    # smaller knuckle plate below the main plate
    ky = py1 + 2
    if ky + 2 <= bottom - 3:
        d.rectangle([px0 + 1, ky, px1 - 1, ky + 2], fill=steel)
        d.line([px0 + 1, ky, px1 - 1, ky], fill=steel_light, width=1)
        rivet(d, px0 + 1, ky, grade)
        rivet(d, px1 - 1, ky, grade)


def draw_gloves(d, rng, grade, size):
    if size == "xs":
        gw, gh, spread = 7, 15, 5
    elif size == "xl":
        gw, gh, spread = 11, 23, 8
    else:
        gw, gh, spread = 10, 20, 7
    top = 16 - gh // 2
    single_glove(d, rng, 16 - spread, top, gw, gh, grade, flip=False)
    single_glove(d, rng, 16 + spread, top, gw, gh, grade, flip=True)


# -------------------------------------------------------------- pauldrons

def single_pauldron(d, rng, cx, top, dw, dh, grade):
    steel, steel_dark, steel_light = STEELS[grade]
    x0 = cx - dw // 2
    x1 = x0 + dw - 1
    dome_h = max(5, dh * 2 // 3)
    # dome: filled ellipse top, squared off below its centre line
    d.ellipse([x0, top, x1, top + dome_h + 3], fill=steel)
    d.rectangle([x0, top + dome_h - 1, x1, top + dome_h + 1], fill=steel)
    # dome shading: dark rim, light crown
    d.arc([x0, top, x1, top + dome_h + 3], start=0, end=180,
          fill=steel_dark, width=1)
    if grade == "hc":
        d.ellipse([cx - 3, top + 1, cx, top + 3],
                  fill=(255, 255, 255, 255))
        d.line([x0 + 2, top + 2, cx + 2, top + 1], fill=steel_light,
               width=1)
    elif grade != "lc":  # lc is matte: no glint at all
        d.line([x0 + 2, top + 2, cx + 1, top + 1], fill=steel_light,
               width=1)
    if grade == "qt":
        tint_band(d, x0 + 1, x1 - 1, top + dome_h // 2)
    # rivets around the dome rim
    for rx in (x0 + 1, cx, x1 - 1):
        rivet(d, rx, top + dome_h - 1, grade)
    # lame bands: 3 layered bands following below the dome
    n_lames = 3
    band_h = max(2, (dh - dome_h) // n_lames)
    y = top + dome_h + 1
    for i in range(n_lames):
        # bands bulge slightly then taper, like real lames
        bulge = 1 if i == 0 else 0
        bx0, bx1 = x0 - bulge, x1 + bulge
        by1 = min(y + band_h - 1, 29)
        d.rectangle([bx0, y, bx1, by1], fill=shade(steel, 1.0 - i * 0.12))
        d.line([bx0, y, bx1, y], fill=steel_dark, width=1)
        d.line([bx0, by1, bx1, by1], fill=steel_dark, width=1)
        d.line([bx0, y + 1, bx1, y + 1], fill=steel_light, width=1)
        if grade == "qt" and i == 1:
            tint_band(d, bx0 + 1, bx1 - 1, y + 1)
        rivet(d, bx0 + 1, y + 1, grade)
        rivet(d, bx1 - 1, y + 1, grade)
        y = by1 + 1
    return y  # bottom edge, for the strap


def draw_shoulder_guards(d, rng, grade, size):
    if size == "xs":
        dw, dh, spread = 10, 13, 6
    elif size == "xl":
        dw, dh, spread = 15, 20, 8
    else:
        dw, dh, spread = 13, 17, 7
    top = 15 - dh // 2
    bottom = single_pauldron(d, rng, 16 - spread, top, dw, dh, grade)
    single_pauldron(d, rng, 16 + spread, top, dw, dh, grade)
    # attachment straps: leather tabs hanging below each pauldron,
    # joined by a horizontal canvas/leather strap
    strap_y = min(bottom + 1, 30)
    for cx in (16 - spread, 16 + spread):
        d.rectangle([cx - 1, bottom, cx + 1, strap_y], fill=LEATHER)
        d.line([cx - 1, bottom, cx - 1, strap_y], fill=LEATHER_DARK,
               width=1)
        d.point((cx, strap_y), fill=RIVET_LIGHT)
    d.line([16 - spread, strap_y, 16 + spread, strap_y], fill=LEATHER,
           width=1)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    grade, kind, size = parse(sid)
    if kind == "hands":
        draw_gloves(d, rng, grade, size)
    else:
        draw_shoulder_guards(d, rng, grade, size)
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
                       "preview_item47.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item47.png {prev.size}")


if __name__ == "__main__":
    main()
