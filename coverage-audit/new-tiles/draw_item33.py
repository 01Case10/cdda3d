#!/usr/bin/env python3
"""Batch ITEM33 (Phase 2g.4 scope extension): tailoring resources.

44 ids (authoritative list embedded below; source list:
~/workspace/item-batches/33.txt). 41 resolve to ITEM entries
in data/json/items/resources/tailoring.json; fur, faux_fur
and nomex also have material definitions in materials.json,
but their ITEM entries live in tailoring.json too (fur patch,
brown; faux fur patch, copy-from fur, pink; Nomex patch,
light_gray) and those are what is drawn. Copy-from chains
were resolved against the data before drawing.

Families, parameterized from the entries' data:

- Patches (6-inch squares): canvas_patch (cream coarse
  weave), denim_patch (blue twill), kevlar_patch (golden-
  olive loose weave), nomex (light gray fine weave), fur
  (brown pelt on a leather backing edge), faux_fur (garish
  pink fluff).
- Sheets (12x24 inch, drawn wide): plain sheets get a woven
  body, hem stitch and a soft fold shadow -- canvas (waxed,
  slight sheen), denim, kevlar, leather (smooth hide, darker
  edge), lycra (violet-gray with a stretch sheen band),
  neoprene (matte black rubber), nomex, nylon (gray sheen),
  faux fur (magenta fluff). Patchwork variants are the same
  sheet cut into a 3x2 grid of shade variants with dark
  seams and stitch dashes; faux-fur patchwork uses clashing
  brights ("diseased Muppet"), cotton patchwork uses muted
  clothing colors, neoprene patchwork gets pale seam tape
  (waterproofed seams), leather patchwork is folded.
- Scraps: a handful of small odd-shaped bits of each
  material. scrap_carpet is one big misshapen chunky section
  with a mottled pilled top and dark backing.
- Singles: rigid_kevlar_plate (green epoxy panel, bevel,
  rivets), sheet_kevlar_layered (ballistic panel with a
  layered edge showing its 16 plies), gambeson_batting (a
  heap of shredded fabric strips, straw and hair),
  strap_cotton (long cotton strap with a steel adjustment
  buckle in the center), wool_staple (a fluffy cluster of
  off-white wool fibers with curl strokes), tainted_kevlar
  and tainted_nomex (fabric lumps fused with rotten flesh
  blobs and rot spots).
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "canvas_patch",
    "denim_patch",
    "faux_fur",
    "fur",
    "gambeson_batting",
    "kevlar_patch",
    "nomex",
    "rigid_kevlar_plate",
    "scrap_canvas",
    "scrap_carpet",
    "scrap_denim",
    "scrap_faux_fur",
    "scrap_fur",
    "scrap_kevlar",
    "scrap_leather",
    "scrap_lycra",
    "scrap_neoprene",
    "scrap_nomex",
    "scrap_nylon",
    "sheet_canvas",
    "sheet_canvas_patchwork",
    "sheet_cotton_patchwork",
    "sheet_denim",
    "sheet_denim_patchwork",
    "sheet_faux_fur",
    "sheet_faux_fur_patchwork",
    "sheet_fur_patchwork",
    "sheet_kevlar",
    "sheet_kevlar_layered",
    "sheet_kevlar_patchwork",
    "sheet_leather",
    "sheet_leather_patchwork",
    "sheet_lycra",
    "sheet_lycra_patchwork",
    "sheet_neoprene",
    "sheet_neoprene_patchwork",
    "sheet_nomex",
    "sheet_nomex_patchwork",
    "sheet_nylon",
    "sheet_nylon_patchwork",
    "strap_cotton",
    "tainted_kevlar",
    "tainted_nomex",
    "wool_staple",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")


def C(rgb):
    return (rgb[0], rgb[1], rgb[2], 255)


def shade(rgb, f):
    return C(tuple(max(0, min(255, int(v * f))) for v in rgb))


def new_sprite():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


# --- palettes (base, dark, light) -------------------------------------------
CANVAS = ((196, 186, 158), (166, 154, 124), (218, 210, 184))
CANVAS_WAX = ((184, 172, 140), (156, 144, 112), (206, 196, 164))
DENIM = ((66, 92, 140), (52, 74, 118), (86, 112, 160))
KEVLAR = ((168, 152, 84), (138, 122, 60), (194, 178, 108))
KEVLAR_GREEN = ((96, 110, 64), (76, 90, 50), (120, 134, 84))
LEATHER = ((128, 88, 54), (98, 62, 34), (150, 108, 68))
LYCRA = ((90, 86, 106), (68, 64, 84), (122, 118, 142))
NEOPRENE = ((46, 46, 50), (32, 32, 36), (70, 70, 76))
NOMEX = ((196, 198, 200), (166, 168, 172), (220, 222, 224))
NYLON = ((140, 144, 150), (112, 116, 124), (172, 178, 186))
FUR = ((122, 90, 60), (94, 66, 40), (152, 118, 82))
FAUXFUR = ((226, 120, 180), (196, 86, 150), (246, 172, 212))
COTTON_PW = [(176, 92, 82), (98, 122, 168), (216, 210, 196),
             (122, 152, 112), (168, 132, 170), (96, 96, 104)]
FAUXFUR_PW = [(226, 120, 180), (90, 200, 190), (160, 110, 220),
              (240, 150, 70), (120, 210, 110), (230, 90, 110)]
NYLON_PW = [(140, 144, 150), (110, 118, 96), (88, 100, 130),
            (150, 116, 116), (124, 130, 140), (104, 110, 118)]


def variants(pal, n=6):
    base = pal[0]
    fs = [1.0, 0.88, 1.1, 0.94, 1.16, 0.82]
    return [shade(base, fs[i % len(fs)]) for i in range(n)]


# --- texture helpers ---------------------------------------------------------

def weave(d, box, pal, rng, step=2, twill=False):
    x0, y0, x1, y1 = box
    for y in range(y0 + step, y1, step):
        d.line([x0, y, x1, y], fill=pal[1])
    for x in range(x0 + step, x1, step):
        d.line([x, y0, x, y1], fill=pal[2])
    if twill:
        for i in range(-(y1 - y0), x1 - x0, 4):
            d.line([x0 + max(i, 0), y0 + max(-i, 0),
                    x0 + min(i + (y1 - y0), x1 - x0), y0 + min((y1 - y0), (y1 - y0) + i)],
                   fill=pal[1])


def stitch_border(d, box, color, inset=2):
    x0, y0, x1, y1 = (box[0] + inset, box[1] + inset,
                      box[2] - inset, box[3] - inset)
    for x in range(x0, x1, 3):
        d.line([x, y0, min(x + 1, x1), y0], fill=color)
        d.line([x, y1, min(x + 1, x1), y1], fill=color)
    for y in range(y0, y1, 3):
        d.line([x0, y, x0, min(y + 1, y1)], fill=color)
        d.line([x1, y, x1, min(y + 1, y1)], fill=color)


def fur_strokes(d, box, pal, rng, n=90, length=3):
    x0, y0, x1, y1 = box
    cols = [pal[0], pal[1], pal[2]]
    for _ in range(n):
        x = rng.randint(x0 + 1, x1 - 2)
        y = rng.randint(y0 + 1, y1 - 2)
        c = cols[rng.randrange(3)]
        d.line([x, y, x + rng.randint(-1, 1), y + length], fill=c)


def mottle(d, box, pal, rng, n=40):
    x0, y0, x1, y1 = box
    for _ in range(n):
        x = rng.randint(x0 + 1, x1 - 2)
        y = rng.randint(y0 + 1, y1 - 2)
        d.point((x, y), fill=pal[rng.randrange(3)])


# --- shape renderers ---------------------------------------------------------

def draw_patch(rng, pal, texture="weave", twill=False):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    box = (8, 8, 23, 23)
    d.rectangle(box, fill=C(pal[0]))
    if texture == "weave":
        weave(d, box, pal, rng, step=2, twill=twill)
    elif texture == "fur":
        fur_strokes(d, box, pal, rng, n=110)
        # leather backing peeking at the bottom edge
        d.line([8, 23, 23, 23], fill=C((104, 72, 44)))
    elif texture == "fluff":
        fur_strokes(d, box, pal, rng, n=130, length=2)
        for _ in range(14):  # fluffy halo ticks on the edge
            x = rng.randint(7, 24)
            y = rng.choice([7, 24])
            d.point((x, y), fill=C(pal[2]))
    stitch_border(d, box, shade(pal[1], 0.8))
    return img


def draw_sheet(rng, pal, texture="weave", twill=False, sheen=False):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    box = (3, 9, 28, 22)
    d.rectangle(box, fill=C(pal[0]))
    if texture == "weave":
        weave(d, box, pal, rng, step=2, twill=twill)
    elif texture == "fur":
        fur_strokes(d, box, pal, rng, n=150)
    elif texture == "fluff":
        fur_strokes(d, box, pal, rng, n=170, length=2)
    elif texture == "leather":
        mottle(d, box, pal, rng, n=50)
        d.line([3, 22, 28, 22], fill=C(pal[1]))
    elif texture == "matte":
        mottle(d, box, pal, rng, n=18)
    if sheen:
        d.polygon([(8, 9), (14, 9), (10, 22), (4, 22)], fill=C(pal[2]))
    # soft fold shadow along the bottom, hem stitch
    d.line([3, 21, 28, 21], fill=shade(pal[1], 0.92))
    stitch_border(d, box, shade(pal[1], 0.85), inset=1)
    return img


def draw_patchwork(rng, pal, texture="weave", colors=None, seam_tape=False,
                   folded=False):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    box = (3, 9, 28, 22)
    cols = colors if colors else variants(pal)
    cw = (box[2] - box[0]) // 3
    ch = (box[3] - box[1]) // 2
    cells = []
    for r in range(2):
        for cidx in range(3):
            cx0 = box[0] + cidx * cw
            cy0 = box[1] + r * ch
            cx1 = box[2] if cidx == 2 else cx0 + cw
            cy1 = box[3] if r == 1 else cy0 + ch
            cells.append((cx0, cy0, cx1, cy1))
            d.rectangle([cx0, cy0, cx1, cy1], fill=cols[(r * 3 + cidx) % len(cols)])
    for cell in cells:
        if texture == "weave":
            weave(d, cell, (tuple(cols[0][:3]), pal[1], pal[2]), rng, step=3)
        elif texture in ("fur", "fluff"):
            fur_strokes(d, cell, pal, rng, n=26, length=2)
        elif texture == "leather":
            mottle(d, cell, pal, rng, n=8)
    seam = C((238, 234, 220)) if seam_tape else shade(pal[1], 0.75)
    for cidx in (1, 2):
        x = box[0] + cidx * cw
        d.line([x, box[1], x, box[3]], fill=seam)
    y = box[1] + ch
    d.line([box[0], y, box[2], y], fill=seam)
    # stitch dashes along the seams
    st = shade(pal[1], 0.6) if not seam_tape else shade(pal[0], 0.8)
    for cidx in (1, 2):
        x = box[0] + cidx * cw
        for yy in range(box[1] + 1, box[3], 3):
            d.point((x - 1, yy), fill=st)
    for xx in range(box[0] + 1, box[2], 3):
        d.point((xx, y - 1), fill=st)
    d.rectangle(box, outline=shade(pal[1], 0.8))
    if folded:
        d.line([15, 9, 15, 22], fill=C(pal[2]))
        d.line([16, 9, 16, 22], fill=shade(pal[1], 0.9))
    return img


def draw_scraps(rng, pal, texture="weave", n=4):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    centers = [(10, 11), (21, 10), (11, 21), (22, 20), (16, 15)][:n]
    for (cx, cy) in centers:
        r = rng.randint(4, 6)
        pts = []
        for k in range(5):
            ang = k * 1.2566 + rng.uniform(-0.3, 0.3)
            rr = r * rng.uniform(0.6, 1.1)
            pts.append((cx + int(rr * math.cos(ang)),
                        cy + int(rr * math.sin(ang))))
        d.polygon(pts, fill=C(pal[0]))
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        cell = (min(xs), min(ys), max(xs), max(ys))
        if texture == "weave":
            for yy in range(cell[1] + 1, cell[3], 2):
                d.line([cell[0], yy, cell[2], yy], fill=pal[1])
        elif texture in ("fur", "fluff"):
            fur_strokes(d, cell, pal, rng, n=12, length=2)
        elif texture == "leather":
            mottle(d, cell, pal, rng, n=5)
        d.line([pts[0], pts[1]], fill=shade(pal[1], 0.85))
    return img


def draw_carpet(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = ((122, 106, 90), (88, 74, 60), (146, 128, 106))
    # thick dark backing, offset down
    d.polygon([(6, 12), (25, 10), (27, 20), (8, 23)], fill=C(pal[1]))
    d.polygon([(6, 10), (25, 8), (27, 18), (8, 21)], fill=C(pal[0]))
    box = (7, 9, 26, 20)
    for _ in range(70):  # pilling
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=[C(pal[1]), C(pal[2]),
                              C((158, 112, 92)), C((104, 96, 84))][rng.randrange(4)])
    return img


def draw_batting(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.ellipse([5, 13, 27, 25], fill=C((96, 88, 72)))
    cols = [(206, 180, 110), (150, 148, 140), (210, 204, 190),
            (120, 96, 70), (168, 110, 100), (110, 120, 150),
            (188, 172, 140), (140, 128, 96)]
    for _ in range(80):
        x = rng.randint(7, 24)
        y = rng.randint(11, 23)
        ln = rng.randint(3, 6)
        dx = rng.randint(-1, 1)
        d.line([x, y, x + dx * ln // 2, y + rng.randint(-1, 2)],
               fill=C(cols[rng.randrange(len(cols))]))
    for _ in range(16):  # stray fibers sticking out
        x = rng.randint(6, 25)
        d.line([x, 12, x + rng.randint(-1, 1), 10],
               fill=C(cols[rng.randrange(len(cols))]))
    return img


def draw_rigid_plate(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = KEVLAR_GREEN
    d.rounded_rectangle([7, 5, 24, 26], radius=3, fill=C(pal[0]))
    d.rounded_rectangle([9, 7, 22, 24], radius=2, outline=shade(pal[1], 0.9))
    # bevel: light top/left, dark bottom/right
    d.line([9, 5, 22, 5], fill=C(pal[2]))
    d.line([7, 7, 7, 24], fill=C(pal[2]))
    d.line([9, 26, 22, 26], fill=C(pal[1]))
    d.line([24, 7, 24, 24], fill=C(pal[1]))
    d.polygon([(10, 7), (15, 7), (11, 24), (9, 24)], fill=shade(pal[0], 1.12))
    for (x, y) in [(10, 8), (21, 8), (10, 23), (21, 23)]:  # rivets
        d.point((x, y), fill=C((52, 58, 38)))
        d.point((x, y - 1), fill=C(pal[2]))
    return img


def draw_layered_panel(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = KEVLAR_GREEN
    d.rectangle([5, 7, 26, 21], fill=C(pal[0]))
    weave(d, (5, 7, 26, 21), pal, rng, step=1)
    d.line([5, 13, 26, 13], fill=shade(pal[1], 0.95))
    # layered edge: the 16 plies as stripes along the bottom
    for i, yy in enumerate(range(22, 26)):
        d.line([5, yy, 26, yy], fill=C(pal[2]) if i % 2 == 0 else C(pal[1]))
    d.rectangle([5, 7, 26, 21], outline=shade(pal[1], 0.85))
    return img


def draw_strap(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = ((186, 180, 168), (158, 150, 138), (206, 200, 188))
    d.rectangle([1, 13, 30, 19], fill=C(pal[0]))
    for yy in (15, 17):
        d.line([1, yy, 30, yy], fill=pal[1])
    stitch_border(d, (1, 13, 30, 19), shade(pal[1], 0.85), inset=1)
    # steel adjustment buckle in the center
    steel = C((158, 162, 168))
    dark = C((52, 52, 58))
    d.rectangle([12, 11, 19, 21], fill=dark)
    d.rectangle([12, 11, 19, 21], outline=steel)
    d.rectangle([13, 12, 18, 20], outline=steel)
    d.line([15, 11, 15, 21], fill=steel)  # center bar
    d.point((17, 16), fill=C((210, 214, 220)))  # prong tip
    return img


def draw_tainted(rng, pal):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.polygon([(9, 10), (22, 9), (25, 16), (21, 23), (11, 22), (7, 16)],
              fill=shade(pal[0], 0.92))
    weave(d, (9, 10, 24, 22), (pal[0], pal[1], pal[2]), rng, step=3)
    flesh = [(186, 110, 104), (160, 80, 76), (204, 138, 124)]
    for (cx, cy, r) in [(13, 14, 3), (20, 18, 4), (17, 11, 2)]:
        d.ellipse([cx - r, cy - r, cx + r, cy + r],
                  fill=C(flesh[rng.randrange(3)]))
        d.point((cx, cy), fill=C((140, 62, 58)))
    for _ in range(10):  # rot spots and sickly speckle
        x = rng.randint(9, 24)
        y = rng.randint(10, 22)
        d.point((x, y), fill=C((70, 60, 40)) if rng.random() < 0.5
                else C((110, 130, 80)))
    return img


def draw_wool(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    pal = ((232, 226, 212), (212, 204, 186), (246, 242, 230))
    for (cx, cy, r) in [(11, 15, 5), (16, 13, 6), (21, 15, 5),
                        (13, 19, 5), (19, 19, 5), (16, 17, 6)]:
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=C(pal[0]))
    for _ in range(70):  # curl strokes
        x = rng.randint(7, 24)
        y = rng.randint(9, 23)
        c = C(pal[rng.randrange(3)])
        d.line([x, y, x + 2, y + 1], fill=c)
        d.point((x + 2, y), fill=c)
    for _ in range(8):  # loose fibers
        x = rng.randint(6, 25)
        y = rng.choice([7, 8, 24, 25])
        d.point((x, y), fill=C(pal[1]))
    return img


# --- dispatch ----------------------------------------------------------------

def _patch(pal, texture="weave", twill=False):
    return lambda rng: draw_patch(rng, pal, texture, twill)


def _sheet(pal, texture="weave", twill=False, sheen=False):
    return lambda rng: draw_sheet(rng, pal, texture, twill, sheen)


def _pw(pal, texture="weave", colors=None, seam_tape=False, folded=False):
    return lambda rng: draw_patchwork(rng, pal, texture, colors, seam_tape,
                                      folded)


def _scraps(pal, texture="weave"):
    return lambda rng: draw_scraps(rng, pal, texture)


BUILDERS = {
    "canvas_patch": _patch(CANVAS),
    "denim_patch": _patch(DENIM, twill=True),
    "faux_fur": _patch(FAUXFUR, "fluff"),
    "fur": _patch(FUR, "fur"),
    "gambeson_batting": draw_batting,
    "kevlar_patch": _patch(KEVLAR),
    "nomex": _patch(NOMEX),
    "rigid_kevlar_plate": draw_rigid_plate,
    "scrap_canvas": _scraps(CANVAS),
    "scrap_carpet": draw_carpet,
    "scrap_denim": _scraps(DENIM),
    "scrap_faux_fur": _scraps(FAUXFUR, "fluff"),
    "scrap_fur": _scraps(FUR, "fur"),
    "scrap_kevlar": _scraps(KEVLAR),
    "scrap_leather": _scraps(LEATHER, "leather"),
    "scrap_lycra": _scraps(LYCRA),
    "scrap_neoprene": _scraps(NEOPRENE, "matte"),
    "scrap_nomex": _scraps(NOMEX),
    "scrap_nylon": _scraps(NYLON),
    "sheet_canvas": _sheet(CANVAS_WAX, sheen=True),
    "sheet_canvas_patchwork": _pw(CANVAS),
    "sheet_cotton_patchwork": _pw(NOMEX, colors=[C(c) for c in COTTON_PW]),
    "sheet_denim": _sheet(DENIM, twill=True),
    "sheet_denim_patchwork": _pw(DENIM),
    "sheet_faux_fur": _sheet(FAUXFUR, "fluff"),
    "sheet_faux_fur_patchwork": _pw(FAUXFUR, "fluff",
                                    colors=[C(c) for c in FAUXFUR_PW]),
    "sheet_fur_patchwork": _pw(FUR, "fur"),
    "sheet_kevlar": _sheet(KEVLAR),
    "sheet_kevlar_layered": draw_layered_panel,
    "sheet_kevlar_patchwork": _pw(KEVLAR),
    "sheet_leather": _sheet(LEATHER, "leather"),
    "sheet_leather_patchwork": _pw(LEATHER, "leather", folded=True),
    "sheet_lycra": _sheet(LYCRA, sheen=True),
    "sheet_lycra_patchwork": _pw(LYCRA),
    "sheet_neoprene": _sheet(NEOPRENE, "matte"),
    "sheet_neoprene_patchwork": _pw(NEOPRENE, "matte", seam_tape=True),
    "sheet_nomex": _sheet(NOMEX),
    "sheet_nomex_patchwork": _pw(NOMEX),
    "sheet_nylon": _sheet(NYLON, sheen=True),
    "sheet_nylon_patchwork": _pw(NYLON, colors=[C(c) for c in NYLON_PW]),
    "strap_cotton": draw_strap,
    "tainted_kevlar": lambda rng: draw_tainted(rng, KEVLAR),
    "tainted_nomex": lambda rng: draw_tainted(rng, NOMEX),
    "wool_staple": draw_wool,
}


def build(item_id):
    rng = random.Random(zlib.crc32(item_id.encode()))
    img = BUILDERS[item_id](rng)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (item_id, alphas)
    return img


def main():
    assert len(IDS) == 44, len(IDS)
    assert set(IDS) == set(BUILDERS), set(IDS) ^ set(BUILDERS)
    for item_id in IDS:
        img = build(item_id)
        assert img.size == (32, 32)
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols = 8
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * 36, rows * 36), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        img = build(item_id)
        sheet.alpha_composite(img, ((i % cols) * 36 + 2, (i // cols) * 36 + 2))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item33.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item33.png {sheet.size}")


if __name__ == "__main__":
    main()
