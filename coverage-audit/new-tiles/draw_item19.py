#!/usr/bin/env python3
"""Batch ITEM19 (Phase 2g.4 scope extension): swords and
blades, final set.

12 ids from data/json/items/melee/swords_and_blades.json
(authoritative list embedded below; source list:
~/workspace/item-batches/19.txt). No copy-from parents
outside the set except tanto_fake / tanto_inferior <-
tanto and wakizashi* <- mc_wakizashi (a covered sibling);
chains resolved against the data before drawing.

Every blade is drawn vertically on the 32x32 canvas (tip
up, hilt down) and rotated -45 degrees with NEAREST
resampling, so the finished sprite lies on the diagonal
with the hilt low-left and the tip high-right, matching
the entries' '/' symbol. Lengths scale with the data's
longest_side (100 cm -> 30 px, 50 cm -> 22 px).

The set, from the entries' names/descriptions/materials:

- sword_bayonet (steel, 56 cm): "a long, knife bladed
  bayonet ... long enough to be used as a short sword" --
  slim single-edged blade with a clipped point, a small
  crossguard with the muzzle ring beside it, ribbed dark
  grip.
- sword_bronze (bronze, 50 cm): "an imitation of a sword
  made from bronze" -- a plain straight arming-sword shape
  in dull bronze, a nick or two on the edge.
- sword_xiphos (bronze, 50 cm): "a bronze sword of
  ancient Greek origin" -- the real thing: leaf-shaped
  blade (swells, then tapers), minimal guard, short grip,
  brighter polished bronze than the imitation.
- sword_crude (wood + budget steel, 100 cm): "bits of
  thin scrap metal crudely beat into the semblance of an
  edge over a wooden sword" -- a plank core with jagged
  scrap-steel teeth beaten along one edge, patches and
  nail dots, an uneven scrap guard strip.
- sword_metal (budget steel, 100 cm): "an imitation of a
  true sword ... made from whatever scrap metal was on
  hand" -- a proper sword silhouette, but patchy matte
  steel with a weld line across the blade.
- sword_wood (wood, 100 cm): the "2-by-sword", "a plank
  with a handle and a whittled-down point" -- a wide
  grained plank, whittled tip, narrower handle, no metal.
- tanto (steel/leather, 53 cm): the samurai's backup
  knife, "a more-modern remake" -- single-edged blade
  with a gentle curve and clipped tip, dark polished
  steel, small oval tsuba, neat diamond-wrapped grip.
- tanto_fake: "a dull, cheaply-made replica" -- same
  silhouette, flat matte gray blade, blunt flat tip,
  plain wrap.
- tanto_inferior: "doesn't feel well-balanced" -- same
  knife with the proportions off: stubbier thicker blade
  on an over-long grip, mid-gray steel.
- wakizashi (mc steel/leather, 70 cm): "smaller and
  lighter than a katana" -- the tanto's bigger sibling:
  longer gently curved single-edged blade, bright
  polished steel, tsuba and diamond wrap.
- wakizashi_fake: "a dull, cheap replica" -- flat matte
  blade, blunt tip, plain wrap.
- wakizashi_inferior: "something not quite right" --
  the blade is dead straight (a wakizashi should curve)
  and the tsuba sits a pixel off-center; mid steel.

Items are objects on transparency, native 32x32, flat
UltiCa style, bodies fully opaque (alpha in {0,255}).
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 12 folders + preview_item19.png.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "sword_bayonet",
    "sword_bronze",
    "sword_crude",
    "sword_metal",
    "sword_wood",
    "sword_xiphos",
    "tanto",
    "tanto_fake",
    "tanto_inferior",
    "wakizashi",
    "wakizashi_fake",
    "wakizashi_inferior",
]

CX = 16  # vertical axis of the un-rotated sword

# palettes: (base, light/edge, dark/spine-or-fuller)
STEEL = ((188, 192, 198, 255), (236, 239, 243, 255), (138, 142, 149, 255))
STEEL_DARK = ((106, 110, 117, 255), (198, 202, 208, 255), (66, 69, 75, 255))
STEEL_MID = ((140, 144, 150, 255), (206, 210, 216, 255), (98, 102, 108, 255))
STEEL_FLAT = ((150, 150, 150, 255), (158, 158, 158, 255), (138, 138, 138, 255))
BUDGET = ((156, 158, 156, 255), (196, 198, 194, 255), (116, 118, 116, 255))
BUDGET_PATCH = ((136, 138, 136, 255), (172, 174, 170, 255), (104, 106, 104, 255))
BRONZE_DULL = ((164, 118, 50, 255), (204, 156, 72, 255), (116, 80, 32, 255))
BRONZE = ((194, 144, 58, 255), (234, 188, 94, 255), (136, 94, 36, 255))
SCRAP = ((148, 150, 148, 255), (188, 190, 186, 255), (102, 104, 102, 255))
WOOD = ((146, 102, 58, 255), (176, 130, 80, 255), (112, 76, 38, 255))
WOOD_DARK = ((110, 74, 40, 255), (140, 100, 56, 255), (82, 54, 28, 255))
GRIP_BROWN = ((88, 60, 38, 255), (142, 104, 64, 255), (58, 38, 24, 255))
GRIP_BLACK = ((34, 34, 40, 255), (96, 96, 106, 255), (18, 18, 22, 255))
IRON_DARK = ((52, 52, 58, 255), (92, 92, 100, 255), (30, 30, 34, 255))
RIB = ((30, 30, 34, 255), (56, 56, 62, 255), (16, 16, 18, 255))
NAIL = (40, 40, 42, 255)


def rows_fill(d, cx, y_top, widths, color_fn, anchor="center"):
    """Fill horizontal spans; widths[i] is the span on row y_top+i.
    color_fn(x, i) -> rgba. anchor 'center' centers on cx, 'left'
    starts at cx (for clipped points anchored to the spine)."""
    for i, w in enumerate(widths):
        if w <= 0:
            continue
        if anchor == "center":
            x0 = cx - w // 2
        else:
            x0 = cx
        for x in range(x0, x0 + w):
            d.point((x, y_top + i), fill=color_fn(x, i))


def straight_widths(n, w=4):
    """Double-edged straight blade: full width, then a taper."""
    out = []
    for i in range(n):  # i = 0 at guard, n-1 at tip
        frac = i / max(1, n - 1)
        if frac < 0.58:
            out.append(w)
        else:
            k = (frac - 0.58) / 0.42
            out.append(max(1, round(w * (1 - k))))
    return out[::-1]  # return tip-first for rows_fill from y_top


def leaf_widths(n):
    """Xiphos leaf blade: narrow at guard, swells, long taper."""
    out = []
    for i in range(n):
        frac = i / max(1, n - 1)
        if frac < 0.45:
            out.append(3 + round(2 * frac / 0.45))
        else:
            k = (frac - 0.45) / 0.55
            out.append(max(1, round(5 * (1 - k))))
    return out[::-1]


def european(rng, total, blade_n, grip_n, guard_n, pommel_n,
             widths, pal, guard_w, guard_pal, grip_pal,
             patch_pal=None, weld=False, nicks=False):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    y_bot = 16 + total // 2
    # pommel (bottom)
    y = y_bot - pommel_n
    if pommel_n:
        rows_fill(d, CX, y, [3, 4][:pommel_n], lambda x, i: pal[2])
    # grip
    y -= grip_n
    for i in range(grip_n):
        for x in range(CX - 1, CX + 2):
            c = grip_pal[0]
            if (x - (y + i)) % 3 == 0:
                c = grip_pal[1]
            d.point((x, y + i), fill=c)
    # guard
    y -= guard_n
    for i in range(guard_n):
        rows_fill(d, CX, y + i, [guard_w], lambda x, j: guard_pal[0])
        d.point((CX - guard_w // 2, y + i), fill=guard_pal[2])
        d.point((CX + guard_w // 2 - (1 if guard_w % 2 == 0 else 0),
                 y + i), fill=guard_pal[2])
    # blade
    y -= blade_n
    base, light, dark = pal

    def blade_color(x, i):
        row_from_guard = blade_n - 1 - i
        w = widths[i]
        x0 = CX - w // 2
        if patch_pal and rng.random() < 0.16:
            return patch_pal[0]
        if weld and row_from_guard == blade_n // 2:
            return dark
        if x == x0 or x == x0 + w - 1:
            return light
        if w >= 4 and x == CX - 1 and row_from_guard < blade_n * 0.6:
            return dark  # fuller
        return base

    rows_fill(d, CX, y, widths, blade_color)
    if nicks:  # dark nicks on the right edge of the blade
        for _ in range(2):
            j = rng.randrange(0, blade_n // 2)  # tip-first row index
            w = widths[j]
            x_edge = CX - w // 2 + w - 1
            d.point((x_edge, y + j), fill=pal[2])
    return img


def japanese(rng, total, blade_n, grip_n, pal, wrap=True,
             curve=True, blunt=False, thick=False, tsuba_off=0):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    y_bot = 16 + total // 2
    # kashira (pommel cap), 1 row
    y = y_bot - 1
    rows_fill(d, CX, y, [3], lambda x, i: IRON_DARK[1])
    # grip (tsuka) with diamond wrap
    y -= grip_n
    for i in range(grip_n):
        for x in range(CX - 1, CX + 2):
            c = GRIP_BLACK[0]
            if wrap and (x + i) % 3 == 1:
                c = GRIP_BLACK[1]
            d.point((x, y + i), fill=c)
    # tsuba (guard), 2 rows, small oval
    y -= 2
    rows_fill(d, CX + tsuba_off, y, [5], lambda x, i: IRON_DARK[0])
    rows_fill(d, CX + tsuba_off, y + 1, [7], lambda x, i: IRON_DARK[0])
    d.point((CX + tsuba_off - 3, y + 1), fill=IRON_DARK[1])
    d.point((CX + tsuba_off + 3, y + 1), fill=IRON_DARK[1])
    # blade: single-edged, spine at left, edge at right
    y -= blade_n
    base, light, dark = pal
    for i in range(blade_n):  # i = 0 at tip row (y), n-1 at guard
        from_guard = blade_n - 1 - i
        shift = 1 if (curve and from_guard > blade_n // 2) else 0
        spine = CX - 1 + shift
        edge = CX + 1 + shift + (1 if thick else 0)
        if i == 0:  # tip row
            if blunt:
                pass  # full span: a flat, dull end
            else:
                edge = spine + 1  # clipped kissaki point
        elif i == 1 and not blunt:
            edge -= 1
        for x in range(spine, edge + 1):
            if x == spine:
                c = dark
            elif x == edge:
                c = light
            else:
                c = base
            d.point((x, y + i), fill=c)
    return img


def plank(rng, total, blade_n, grip_n, pal, crude=False):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    y_bot = 16 + total // 2
    # handle: narrower, darker, two wrap grooves
    y = y_bot - grip_n
    for i in range(grip_n):
        for x in range(CX - 2, CX + 2):
            c = WOOD_DARK[0]
            if i in (1, grip_n - 2):
                c = WOOD_DARK[2]
            d.point((x, y + i), fill=c)
    # guard strip (crude only): uneven scrap band
    if crude:
        y -= 2
        rows_fill(d, CX, y, [8], lambda x, i: SCRAP[2])
        rows_fill(d, CX - 1, y + 1, [7], lambda x, i: SCRAP[0])
    # blade: wide plank, whittled tip
    y -= blade_n
    base, light, dark = pal
    widths = []
    for i in range(blade_n):  # tip-first
        from_tip = i
        if from_tip == 0:
            widths.append(1)
        elif from_tip == 1:
            widths.append(2)
        elif from_tip == 2:
            widths.append(4)
        elif from_tip == 3:
            widths.append(5 if crude else 6)
        else:
            widths.append(5 if crude else 6)

    def plank_color(x, i):
        w = widths[i]
        x0 = CX - w // 2
        if x == x0:
            return light
        if x == x0 + w - 1:
            return dark
        if x in (CX - 2, CX + 1) and rng.random() < 0.7:
            return dark  # grain streaks
        return base

    rows_fill(d, CX, y, widths, plank_color)
    if crude:
        # scrap steel beaten along the right edge: jagged teeth,
        # patches and nail dots
        for i in range(blade_n):
            w = widths[i]
            x_edge = CX + w // 2 - (0 if w % 2 else 1)
            teeth = rng.choice([0, 1, 1, 2])
            for t in range(teeth + 1):
                d.point((x_edge + t, y + i),
                        fill=SCRAP[1] if t == teeth else SCRAP[0])
            if rng.random() < 0.30:  # a patch plate over the flat
                for dx in (-1, 0):
                    d.point((x_edge + dx, y + i), fill=SCRAP[0])
                if rng.random() < 0.5:
                    d.point((x_edge, y + i), fill=NAIL)
    return img


def bayonet(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    total, blade_n, grip_n = 23, 14, 6
    y_bot = 16 + total // 2
    # steel pommel cap
    y = y_bot - 1
    rows_fill(d, CX, y, [3], lambda x, i: STEEL[2])
    # ribbed black grip
    y -= grip_n
    for i in range(grip_n):
        c = RIB[0] if i % 2 == 0 else RIB[1]
        for x in range(CX - 1, CX + 2):
            d.point((x, y + i), fill=c)
    # guard bar + muzzle ring below it on the spine side
    y -= 2
    rows_fill(d, CX, y, [7], lambda x, i: STEEL[2])
    rows_fill(d, CX, y + 1, [7], lambda x, i: STEEL[0])
    for yy in (y + 2, y + 3):
        d.point((CX - 3, yy), fill=STEEL[2])
    d.point((CX - 3, y + 4), fill=STEEL[2])
    d.point((CX - 2, y + 4), fill=STEEL[2])
    # blade: slim single edge, spine left, clipped point
    y -= blade_n
    for i in range(blade_n):  # tip-first
        spine = CX - 1
        edge = CX + 1
        if i == 0:
            edge = spine + 1
        elif i == 1:
            edge = spine + 1
        elif i == 2:
            edge = CX
        for x in range(spine, edge + 1):
            c = STEEL[2] if x == spine else (
                STEEL[1] if x == edge else STEEL[0])
            d.point((x, y + i), fill=c)
    return img


def finish(img):
    out = img.rotate(-45, resample=Image.NEAREST)
    assert out.size == (32, 32)
    assert set(out.getchannel("A").getdata()) <= {0, 255}
    return out


def render(item_id, rng):
    if item_id == "sword_bayonet":
        return finish(bayonet(rng))
    if item_id == "sword_bronze":
        return finish(european(
            rng, total=22, blade_n=13, grip_n=5, guard_n=2, pommel_n=2,
            widths=straight_widths(13), pal=BRONZE_DULL, guard_w=9,
            guard_pal=BRONZE_DULL, grip_pal=GRIP_BROWN, nicks=True))
    if item_id == "sword_xiphos":
        return finish(european(
            rng, total=22, blade_n=15, grip_n=4, guard_n=1, pommel_n=2,
            widths=leaf_widths(15), pal=BRONZE, guard_w=6,
            guard_pal=BRONZE, grip_pal=GRIP_BROWN))
    if item_id == "sword_metal":
        return finish(european(
            rng, total=30, blade_n=20, grip_n=6, guard_n=2, pommel_n=2,
            widths=straight_widths(20), pal=BUDGET, guard_w=10,
            guard_pal=BUDGET, grip_pal=GRIP_BROWN,
            patch_pal=BUDGET_PATCH, weld=True))
    if item_id == "sword_wood":
        return finish(plank(rng, total=30, blade_n=23, grip_n=7,
                            pal=WOOD))
    if item_id == "sword_crude":
        return finish(plank(rng, total=30, blade_n=22, grip_n=6,
                            pal=WOOD, crude=True))
    if item_id == "tanto":
        return finish(japanese(rng, total=22, blade_n=12, grip_n=7,
                               pal=STEEL_DARK))
    if item_id == "tanto_fake":
        return finish(japanese(rng, total=22, blade_n=12, grip_n=7,
                               pal=STEEL_FLAT, wrap=False, blunt=True))
    if item_id == "tanto_inferior":
        return finish(japanese(rng, total=22, blade_n=10, grip_n=9,
                               pal=STEEL_MID, thick=True))
    if item_id == "wakizashi":
        return finish(japanese(rng, total=25, blade_n=16, grip_n=6,
                               pal=STEEL))
    if item_id == "wakizashi_fake":
        return finish(japanese(rng, total=25, blade_n=16, grip_n=6,
                               pal=STEEL_FLAT, wrap=False, blunt=True))
    if item_id == "wakizashi_inferior":
        return finish(japanese(rng, total=25, blade_n=15, grip_n=7,
                               pal=STEEL_MID, curve=False, tsuba_off=1))
    raise ValueError(item_id)


def main():
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = render(item_id, rng)
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
    cols, cell = 4, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item19.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item19.png {sheet.size}")


if __name__ == "__main__":
    main()
