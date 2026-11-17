#!/usr/bin/env python3
"""ITEM59: raw fruit + knuckle weapons -- CDDA Voxel coverage audit.

Two clusters:

  Raw fruit (23) -- berries and fruit drawn as themselves, colour
    and form per species: berry handfuls as piles of individual
    berries (blueberries with pale crown stars, near-black
    chokeberries, red cranberries, drupelet-cluster raspberries,
    stem-clustered hobble/nanny/tupelo berries), apples and hips
    with stems, a netted whole cantaloupe vs. its orange chunks,
    a watermelon wedge, halved passionfruit and lulo showing their
    flesh, white coconut flesh in a shell half, stringy juice pulp,
    green-red cholla cactus buds, and wintergreen berries on leaves.

  Knuckle weapons (21) -- cestus and knuckle dusters in standard,
    xl_ (oversize) and xs_ (undersize) sizes, drawn as the weapon
    itself: a top band with finger loops over a palm grip bar.
    Feature by type: brass = classic gold duster; steel_forged =
    clean light-steel duster; steel (scrap) = crude dark duster
    with folded rag wraps under it; impact = thick plate with
    blunt studs and hand straps; nail = two wood squares with
    short nails sticking through; skewer = brace bar with one
    long spike; cestus = leather hand/arm wrap with steel plates
    over the knuckles.  Size scales xs < standard < xl.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "blueberries",
    "chokeberries",
    "cholla_bud",
    "coconut_flesh",
    "crabapple",
    "cranberries",
    "guayaba",
    "hobbleberries",
    "huckleberries",
    "juice_pulp",
    "lulo",
    "m_stellatum_berries",
    "maracuya",
    "melon",
    "melon_chunks",
    "nannyberries",
    "pineapple_chunks",
    "raspberries",
    "rose_hips",
    "rose_hips_cut",
    "tupelo_fruit",
    "watermelon_wedge",
    "wintergreen_berry",
    "cestus",
    "knuckle_brass",
    "knuckle_impact",
    "knuckle_nail",
    "knuckle_skewer",
    "knuckle_steel",
    "knuckle_steel_forged",
    "xl_cestus",
    "xl_knuckle_brass",
    "xl_knuckle_impact",
    "xl_knuckle_nail",
    "xl_knuckle_skewer",
    "xl_knuckle_steel",
    "xl_knuckle_steel_forged",
    "xs_cestus",
    "xs_knuckle_brass",
    "xs_knuckle_impact",
    "xs_knuckle_nail",
    "xs_knuckle_skewer",
    "xs_knuckle_steel",
    "xs_knuckle_steel_forged",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


LEAF = (62, 122, 52, 255)
LEAF_DARK = (40, 88, 38, 255)
STEM = (96, 74, 44, 255)

# ------------------------------------------------------------------- fruit

def berry(d, cx, cy, r, col, crown=None, gloss=True):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=col)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=shade(col, 0.62),
              width=1)
    if gloss:
        d.point((cx - max(1, r // 2), cy - max(1, r // 2)),
                fill=shade(col, 1.55) if max(col[:3]) < 200 else (255, 255, 255, 255))
    if crown:  # pale star scar, e.g. blueberry crown
        d.point((cx, cy), fill=crown)
        d.point((cx - 1, cy), fill=crown)
        d.point((cx + 1, cy), fill=crown)
        d.point((cx, cy - 1), fill=crown)
        d.point((cx, cy + 1), fill=crown)


def berry_pile(d, rng, col, r=3, n=7, crown=None, oval=False, stems=False):
    """A handful: berries heaped in a rough mound, back row first."""
    layout = [(16, 10), (10, 14), (22, 14), (13, 19), (19, 19),
              (16, 23), (9, 22), (23, 22), (16, 16)]
    if stems:  # stem lines fanning from the top before berries go on
        for sx, sy in layout[:3]:
            d.line([16, 6, sx, sy], fill=STEM, width=1)
        d.line([13, 6, 19, 6], fill=STEM, width=1)
    for i, (cx, cy) in enumerate(layout[:n]):
        cx += rng.randint(-1, 1)
        cy += rng.randint(-1, 1)
        if oval:
            d.ellipse([cx - r + 1, cy - r - 1, cx + r - 1, cy + r + 1],
                      fill=col)
            d.ellipse([cx - r + 1, cy - r - 1, cx + r - 1, cy + r + 1],
                      outline=shade(col, 0.62), width=1)
            d.point((cx - 1, cy - 2), fill=shade(col, 1.5))
        else:
            berry(d, cx, cy, r, shade(col, rng.uniform(0.9, 1.08)),
                  crown=crown)


def draw_fruit(d, rng, sid):
    if sid == "blueberries":
        berry_pile(d, rng, (58, 82, 168, 255), r=3, n=8,
                   crown=(168, 182, 216, 255))
    elif sid == "chokeberries":  # black, big and glossy
        berry_pile(d, rng, (38, 34, 44, 255), r=4, n=6)
    elif sid == "hobbleberries":  # black, sweet, on a stem cluster
        berry_pile(d, rng, (44, 40, 52, 255), r=3, n=7, stems=True)
    elif sid == "huckleberries":  # blue like blueberries, smaller/darker
        berry_pile(d, rng, (66, 62, 140, 255), r=2, n=9,
                   crown=(150, 148, 190, 255))
    elif sid == "nannyberries":  # dark blue ovals, prune-like
        berry_pile(d, rng, (52, 58, 104, 255), r=3, n=7, oval=True,
                   stems=True)
    elif sid == "tupelo_fruit":  # dark blue tart drupes
        berry_pile(d, rng, (48, 62, 112, 255), r=3, n=7, oval=True,
                   stems=True)
    elif sid == "cranberries":  # bright red, bitter
        berry_pile(d, rng, (196, 40, 44, 255), r=3, n=8)
    elif sid == "m_stellatum_berries":  # deep red, starry bunch
        berry_pile(d, rng, (158, 28, 40, 255), r=3, n=7, stems=True,
                   crown=(240, 200, 120, 255))
    elif sid == "wintergreen_berry":
        # brilliant red berries nestled in glossy green leaves
        for lx, ly, a in ((8, 20, -0.5), (24, 20, 0.5), (16, 24, 0.0),
                          (11, 13, -0.9), (21, 13, 0.9)):
            pts = [(lx + 6 * math.cos(a + t), ly + 3 * math.sin(a + t))
                   for t in (0, 1.6, 3.1, 4.7)]
            d.polygon(pts, fill=LEAF)
            d.line([lx - 4, ly, lx + 5, ly], fill=LEAF_DARK, width=1)
        berry_pile(d, rng, (226, 36, 40, 255), r=3, n=5)
    elif sid == "raspberries":
        # aggregate drupelets with a hollow core, three berries
        for cx, cy in ((11, 14), (21, 14), (16, 22)):
            for dx, dy in ((-3, -2), (0, -3), (3, -2), (-3, 1), (3, 1),
                           (-1, 3), (2, 3)):
                berry(d, cx + dx, cy + dy, 2, (206, 52, 74, 255),
                      gloss=True)
            d.point((cx, cy), fill=(120, 26, 38, 255))  # hollow core
            d.point((cx, cy - 5), fill=LEAF)  # sepal tuft
    elif sid == "cholla_bud":
        # cactus buds: green-red segmented buds, spines removed
        for cx, cy, h in ((11, 13, 10), (17, 11, 12), (23, 14, 9)):
            col = (104, 148, 62, 255)
            d.polygon([(cx - 3, cy + h), (cx - 3, cy + 2), (cx, cy - 2),
                       (cx + 3, cy + 2), (cx + 3, cy + h)], fill=col)
            d.polygon([(cx - 3, cy + h), (cx - 3, cy + 2), (cx, cy - 2),
                       (cx + 3, cy + 2), (cx + 3, cy + h)],
                      outline=shade(col, 0.65))
            for gy in range(cy + 2, cy + h, 3):  # segment bands
                d.line([cx - 3, gy, cx + 3, gy], fill=shade(col, 0.75),
                       width=1)
            d.ellipse([cx - 2, cy - 3, cx + 2, cy + 1],
                      fill=(196, 74, 54, 255))  # red bud tip
            for px in (cx - 2, cx, cx + 2):  # bare spine scars
                d.point((px, cy + 4), fill=(220, 210, 170, 255))
        d.line([6, 26, 26, 26], fill=STEM, width=1)
    elif sid == "coconut_flesh":
        # brown shell half holding white flesh chunks
        d.polygon([(5, 14), (27, 14), (24, 25), (8, 25)], fill=(104, 72, 44, 255))
        d.polygon([(5, 14), (27, 14), (24, 25), (8, 25)],
                  outline=(70, 48, 28, 255))
        d.line([5, 14, 27, 14], fill=(140, 104, 66, 255), width=2)
        for cx, cy, w, h in ((11, 9, 5, 5), (17, 7, 6, 6), (23, 10, 5, 5),
                             (14, 13, 5, 4), (20, 13, 5, 4)):
            d.polygon([(cx - w // 2, cy + h // 2), (cx, cy - h // 2),
                       (cx + w // 2, cy + h // 2)], fill=(246, 244, 238, 255))
            d.polygon([(cx - w // 2, cy + h // 2), (cx, cy - h // 2),
                       (cx + w // 2, cy + h // 2)],
                      outline=(206, 200, 188, 255))
    elif sid == "crabapple":
        # small sour red apples with stems and a leaf
        for cx, cy, r in ((11, 15, 6), (21, 15, 6), (16, 23, 6)):
            d.ellipse([cx - r, cy - r + 1, cx + r, cy + r + 1],
                      fill=(188, 44, 38, 255))
            d.ellipse([cx - r, cy - r + 1, cx + r, cy + r + 1],
                      outline=(128, 26, 24, 255), width=1)
            d.line([cx, cy - r + 1, cx + 1, cy - r - 2], fill=STEM,
                   width=1)
            d.point((cx - r + 2, cy - 2), fill=(240, 150, 130, 255))
        d.polygon([(17, 7), (23, 5), (22, 9)], fill=LEAF)
    elif sid == "guayaba":
        # guava: yellow-green whole fruit + a halved one, pink flesh
        d.ellipse([5, 10, 17, 24], fill=(206, 196, 84, 255))
        d.ellipse([5, 10, 17, 24], outline=(150, 142, 52, 255), width=1)
        d.point((8, 13), fill=(232, 226, 140, 255))
        d.ellipse([16, 9, 28, 25], fill=(150, 142, 52, 255))
        d.ellipse([17, 10, 27, 24], fill=(240, 168, 178, 255))
        for px, py in ((20, 14), (24, 14), (22, 17), (20, 20), (24, 20),
                       (22, 12)):
            d.point((px, py), fill=(250, 240, 210, 255))  # seeds
    elif sid == "juice_pulp":
        # stringy mash left over from juicing
        base = (240, 148, 96, 255)
        d.ellipse([6, 14, 26, 26], fill=shade(base, 0.92))
        for _ in range(26):
            x, y = rng.uniform(8, 24), rng.uniform(13, 24)
            ln = rng.uniform(2, 5)
            d.line([x, y, x + ln, y + rng.uniform(-1, 1)],
                   fill=shade(base, rng.uniform(0.8, 1.25)), width=1)
        for _ in range(6):
            x, y = rng.uniform(9, 23), rng.uniform(14, 22)
            d.point((int(x), int(y)), fill=(250, 214, 160, 255))
        d.line([10, 12, 14, 10], fill=shade(base, 1.1), width=2)
        d.line([17, 11, 21, 10], fill=shade(base, 1.1), width=2)
    elif sid == "lulo":
        # hairy orange whole + halved, green-yellow juicy flesh
        d.ellipse([4, 11, 16, 25], fill=(232, 128, 40, 255))
        d.ellipse([4, 11, 16, 25], outline=(176, 88, 22, 255), width=1)
        for _ in range(10):  # hairy fuzz dots
            x, y = rng.uniform(5, 15), rng.uniform(12, 24)
            d.point((int(x), int(y)), fill=(250, 190, 110, 255))
        d.ellipse([16, 10, 28, 26], fill=(176, 88, 22, 255))
        d.ellipse([17, 11, 27, 25], fill=(206, 196, 84, 255))
        d.ellipse([19, 14, 25, 22], fill=(168, 196, 74, 255))
        for px, py in ((20, 16), (24, 16), (22, 19)):
            d.point((px, py), fill=(250, 246, 200, 255))
    elif sid == "maracuya":
        # passionfruit halved: dark rind, golden seedy pulp
        for cx in (11, 21):
            d.ellipse([cx - 6, 10, cx + 6, 26], fill=(110, 64, 66, 255))
            d.ellipse([cx - 6, 10, cx + 6, 26],
                      outline=(74, 40, 44, 255), width=1)
            d.ellipse([cx - 5, 11, cx + 5, 25], fill=(246, 240, 230, 255))
            d.ellipse([cx - 4, 12, cx + 4, 24], fill=(246, 186, 52, 255))
            for _ in range(7):
                x, y = rng.uniform(cx - 3, cx + 3), rng.uniform(14, 22)
                d.point((int(x), int(y)), fill=(90, 70, 30, 255))
        d.line([16, 8, 16, 11], fill=STEM, width=1)
    elif sid == "melon":
        # whole cantaloupe: netted rind
        d.ellipse([4, 8, 28, 26], fill=(206, 186, 130, 255))
        d.ellipse([4, 8, 28, 26], outline=(158, 138, 88, 255), width=1)
        for gx in range(7, 28, 4):  # rind netting
            d.arc([gx - 5, 9, gx + 5, 25], start=80, end=280,
                  fill=(232, 216, 170, 255), width=1)
        for gy in range(11, 26, 4):
            d.arc([5, gy - 5, 27, gy + 5], start=350, end=190,
                  fill=(232, 216, 170, 255), width=1)
        d.rectangle([15, 6, 17, 8], fill=STEM)
    elif sid == "melon_chunks":
        # orange cantaloupe chunks, one with a rind sliver
        for cx, cy, w, h in ((11, 12, 8, 7), (21, 12, 8, 7), (16, 20, 9, 7),
                             (9, 21, 6, 5), (24, 21, 6, 5)):
            d.polygon([(cx - w // 2, cy + h // 2), (cx - w // 2 + 1, cy - h // 2),
                       (cx + w // 2, cy - h // 2 + 1), (cx + w // 2 - 1, cy + h // 2)],
                      fill=(246, 156, 74, 255))
            d.polygon([(cx - w // 2, cy + h // 2), (cx - w // 2 + 1, cy - h // 2),
                       (cx + w // 2, cy - h // 2 + 1), (cx + w // 2 - 1, cy + h // 2)],
                      outline=(206, 108, 40, 255))
            d.point((cx - 2, cy - 2), fill=(252, 196, 130, 255))
        d.arc([6, 22, 16, 30], start=0, end=180, fill=(140, 168, 84, 255),
              width=2)  # rind sliver
    elif sid == "pineapple_chunks":
        # golden chunks with fibrous striations
        for cx, cy, w, h in ((11, 12, 8, 7), (21, 12, 8, 7), (16, 20, 9, 7),
                             (9, 21, 6, 5), (24, 21, 6, 5)):
            d.rectangle([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
                        fill=(246, 200, 64, 255))
            d.rectangle([cx - w // 2, cy - h // 2, cx + w // 2, cy + h // 2],
                        outline=(196, 146, 30, 255), width=1)
            for fx in range(cx - w // 2 + 2, cx + w // 2, 2):
                d.line([fx, cy - h // 2 + 1, fx, cy + h // 2 - 1],
                       fill=(252, 224, 120, 255), width=1)
    elif sid == "rose_hips":
        # red oval hips with dark sepal crowns
        for cx, cy in ((11, 13), (21, 13), (16, 21), (9, 21), (23, 21)):
            d.ellipse([cx - 3, cy - 5, cx + 3, cy + 5],
                      fill=(214, 58, 40, 255))
            d.ellipse([cx - 3, cy - 5, cx + 3, cy + 5],
                      outline=(150, 32, 24, 255), width=1)
            d.point((cx, cy - 6), fill=(60, 44, 30, 255))
            d.point((cx - 1, cy - 6), fill=(60, 44, 30, 255))
            d.point((cx + 1, cy - 6), fill=(60, 44, 30, 255))
            d.point((cx - 1, cy - 2), fill=(248, 140, 110, 255))
    elif sid == "rose_hips_cut":
        # cut hips: halves with seeds and spines removed
        for cx, cy in ((11, 14), (21, 14), (16, 23)):
            d.polygon([(cx - 5, cy - 3), (cx + 5, cy - 3), (cx + 3, cy + 4),
                       (cx - 3, cy + 4)], fill=(214, 58, 40, 255))
            d.polygon([(cx - 5, cy - 3), (cx + 5, cy - 3), (cx + 3, cy + 4),
                       (cx - 3, cy + 4)], outline=(150, 32, 24, 255))
            d.polygon([(cx - 3, cy - 2), (cx + 3, cy - 2), (cx + 2, cy + 3),
                       (cx - 2, cy + 3)], fill=(246, 148, 74, 255))
            d.line([cx - 3, cy - 2, cx + 3, cy - 2],
                   fill=(252, 200, 140, 255), width=1)
    elif sid == "watermelon_wedge":
        # wedge: green rind arc, white inner rind, red flesh, seeds
        d.polygon([(6, 10), (26, 10), (16, 27)], fill=(228, 40, 52, 255))
        d.arc([3, 4, 29, 22], start=15, end=165, fill=(52, 140, 52, 255),
              width=3)
        d.arc([4, 5, 28, 21], start=15, end=165, fill=(238, 244, 226, 255),
              width=1)
        for px, py in ((13, 14), (19, 14), (16, 18), (12, 20), (20, 20),
                       (16, 23)):
            d.point((px, py), fill=(40, 32, 28, 255))
        d.point((10, 12), fill=(248, 130, 130, 255))
    else:
        raise KeyError(sid)


# ---------------------------------------------------------------- knuckles
# sid -> (kind, size scale)
def parse_knuckle(sid):
    size = 1.0
    name = sid
    if sid.startswith("xl_"):
        size, name = 1.22, sid[3:]
    elif sid.startswith("xs_"):
        size, name = 0.78, sid[3:]
    if name == "cestus":
        return "cestus", size
    kind = name[len("knuckle_"):]
    return kind, size


MATS = {  # kind -> (body, dark, light)
    "brass": ((198, 158, 52, 255), (140, 108, 30, 255), (246, 214, 110, 255)),
    "steel_forged": ((168, 172, 180, 255), (104, 108, 116, 255),
                     (224, 228, 234, 255)),
    "steel": ((104, 106, 112, 255), (62, 64, 70, 255), (150, 152, 160, 255)),
    "impact": ((158, 162, 170, 255), (96, 100, 108, 255),
               (216, 220, 226, 255)),
    "skewer": ((158, 162, 170, 255), (96, 100, 108, 255),
               (216, 220, 226, 255)),
    "nail": ((120, 122, 128, 255), (78, 80, 86, 255), (180, 182, 188, 255)),
}
WOOD = (146, 104, 58, 255)
WOOD_DARK = (100, 70, 36, 255)
LEATHER = (128, 88, 52, 255)
LEATHER_DARK = (88, 58, 32, 255)
RAG = (186, 172, 148, 255)


def duster(d, cx, top, w, body, dark, light, crude=False):
    """Classic knuckle duster: top band, 4 finger loops, palm grip."""
    hw = w // 2
    x0, x1 = cx - hw, cx + hw
    band_h = 4
    d.rectangle([x0, top, x1, top + band_h - 1], fill=body)
    d.line([x0, top, x1, top], fill=light, width=1)
    d.line([x0, top + band_h - 1, x1, top + band_h - 1], fill=dark, width=1)
    loop_h = max(4, int(6 * (w / 22)))
    # 5 pillars leave 4 finger holes between them
    pw = max(2, w // 8)
    for i in range(5):
        px = x0 + i * (w - pw) // 4
        d.rectangle([px, top + band_h, px + pw - 1, top + band_h + loop_h],
                    fill=body)
        d.line([px, top + band_h, px, top + band_h + loop_h], fill=dark,
               width=1)
    gy = top + band_h + loop_h + 1
    grip_h = max(3, int(4 * (w / 22)))
    d.rectangle([x0 + 1, gy, x1 - 1, gy + grip_h - 1], fill=body)
    d.line([x0 + 1, gy + grip_h - 1, x1 - 1, gy + grip_h - 1], fill=dark,
           width=1)
    d.line([x0 + 1, gy, x1 - 1, gy], fill=light, width=1)
    if crude:  # scrap: hammer dents and an uneven edge
        for _ in range(5):
            pass
    return gy + grip_h


def draw_knuckle(d, rng, sid):
    kind, size = parse_knuckle(sid)
    w = int(round(22 * size))
    cx = 16
    top = 16 - int(round(7 * size))
    if kind == "cestus":
        body, dark, light = MATS["steel_forged"]
        # leather wrap: a wrapped mitt with lacing and an arm tail
        hw = w // 2
        x0, x1 = cx - hw, cx + hw
        y0, y1 = top + 2, top + 2 + int(round(13 * size))
        d.polygon([(x0 + 2, y1 + 6), (x0, y0 + 3), (x0 + 2, y0),
                   (x1 - 2, y0), (x1, y0 + 3), (x1 - 1, y1)],
                  fill=LEATHER)
        d.polygon([(x0 + 2, y1 + 6), (x0, y0 + 3), (x0 + 2, y0),
                   (x1 - 2, y0), (x1, y0 + 3), (x1 - 1, y1)],
                  outline=LEATHER_DARK)
        for wy in range(y0 + 3, y1, 3):  # wrap bands
            d.line([x0 + 1, wy, x1 - 1, wy - 1], fill=LEATHER_DARK, width=1)
        # steel plates over the knuckles
        pw = max(3, w // 6)
        for i in range(4):
            px = x0 + 2 + i * (w - 4 - pw) // 3
            d.rectangle([px, y0 + 1, px + pw - 1, y0 + 3], fill=body)
            d.line([px, y0 + 1, px + pw - 1, y0 + 1], fill=light, width=1)
        # lace X below the plates
        d.line([x0 + 3, y1 - 3, x1 - 3, y1 - 6], fill=RAG, width=1)
        d.line([x0 + 3, y1 - 6, x1 - 3, y1 - 3], fill=RAG, width=1)
        return
    if kind == "nail":
        metal, mdark, mlight = MATS["nail"]
        # two small wood squares with nails sticking through, pointing up
        bw = int(round(10 * size))
        gap = 2
        for bx0 in (cx - bw - gap // 2, cx + gap // 2):
            by = top + int(round(7 * size))
            bh = int(round(8 * size))
            d.rectangle([bx0, by, bx0 + bw - 1, by + bh - 1], fill=WOOD)
            d.rectangle([bx0, by, bx0 + bw - 1, by + bh - 1],
                        outline=WOOD_DARK, width=1)
            d.line([bx0 + 1, by + bh // 2, bx0 + bw - 2, by + bh // 2],
                   fill=WOOD_DARK, width=1)
            for nx in (bx0 + 2, bx0 + bw // 2, bx0 + bw - 3):
                # nail: shaft up from the square, flat head on top
                d.line([nx, by - 4, nx, by], fill=metal, width=1)
                d.line([nx - 1, by - 4, nx + 1, by - 4], fill=mlight,
                       width=1)
                d.point((nx, by + 2), fill=mdark)  # nail point shadow in wood
        return
    if kind == "skewer":
        body, dark, light = MATS["skewer"]
        # one long sturdy spike on a brace bar, grip below
        spike_top = max(2, top - int(round(7 * size)))
        d.polygon([(cx - 1, top + 2), (cx, spike_top), (cx + 1, top + 2)],
                  fill=body)
        d.line([cx, spike_top, cx, top + 2], fill=light, width=1)
        d.rectangle([cx - 2, top + 1, cx + 2, top + 3], fill=dark)
        bottom = duster(d, cx, top + 3, w, body, dark, light)
        # brace straps hanging from the grip
        d.line([cx - w // 4, bottom, cx - w // 4, bottom + 2], fill=dark,
               width=2)
        d.line([cx + w // 4, bottom, cx + w // 4, bottom + 2], fill=dark,
               width=2)
        return
    if kind == "impact":
        body, dark, light = MATS["impact"]
        hw = w // 2
        x0, x1 = cx - hw, cx + hw
        # blunt studs on top of a thick plate
        sw = max(3, w // 6)
        for i in range(4):
            px = x0 + 1 + i * (w - 2 - sw) // 3
            d.rectangle([px, top, px + sw - 1, top + 2], fill=body)
            d.line([px, top, px + sw - 1, top], fill=light, width=1)
        py = top + 3
        ph = max(5, int(round(6 * size)))
        d.rectangle([x0, py, x1, py + ph - 1], fill=body)
        d.rectangle([x0, py, x1, py + ph - 1], outline=dark, width=1)
        d.line([x0, py, x1, py], fill=light, width=1)
        for i in range(4):  # plate seams between knuckle sections
            px = x0 + 2 + i * (w - 4) // 3
            d.line([px, py + 1, px, py + ph - 2], fill=dark, width=1)
        # metal brace straps below the plate
        for sx in (x0 + 3, cx - 1, x1 - 4):
            d.rectangle([sx, py + ph, sx + 2, py + ph + int(round(4 * size))],
                        fill=dark)
        return
    # brass / steel_forged / steel (scrap): classic dusters
    body, dark, light = MATS[kind]
    bottom = duster(d, cx, top + 1, w, body, dark, light,
                    crude=(kind == "steel"))
    if kind == "steel":
        # scrap: folded rags under the grip, hammered dents on the band
        for rx in range(cx - w // 2 + 2, cx + w // 2 - 1, 4):
            d.line([rx, bottom, rx + 2, bottom + 2], fill=RAG, width=1)
            d.line([rx, bottom + 2, rx + 2, bottom], fill=shade(RAG, 0.8),
                   width=1)
        for dx in (cx - w // 3, cx, cx + w // 3):
            d.point((dx, top + 2), fill=dark)
    if kind == "steel_forged":  # clean forged sheen line
        d.line([cx - w // 2 + 2, top + 2, cx + w // 2 - 2, top + 2],
               fill=light, width=1)


# -------------------------------------------------------------------- main

FRUIT = set(IDS[:23])


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in FRUIT:
        draw_fruit(d, rng, sid)
    else:
        draw_knuckle(d, rng, sid)
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
    # contact-sheet preview: 6 cols x 8 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 8, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item59.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item59.png {prev.size}")


if __name__ == "__main__":
    main()
