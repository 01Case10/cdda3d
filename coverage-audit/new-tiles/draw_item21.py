#!/usr/bin/env python3
"""Batch ITEM21 (Phase 2g.4 scope extension): metal
resources, final set -- steel bar stock and structural
sections.

10 ids from data/json/items/resources/metal.json
(authoritative list embedded below; source list:
~/workspace/item-batches/21.txt). Copy-from chains
resolved against the data before drawing: the lc_/mc_
small bars inherit description/symbol/color/volume from
small_steel_bar_flat / small_steel_bar_round and differ
only in material (lc_steel / mc_steel).

Every piece is drawn horizontally on the 32x32 canvas
and rotated -45 degrees with NEAREST resampling, so the
finished sprite lies on the diagonal, matching the
sections' '/' symbol and the sibling melee batches'
diagonal convention. Sizes follow the data: the 5 cm
small flat bars are short slim prisms, the 10 cm flat
bar is chunkier, the small round bar is a thin 30 cm
rod (212 ml) while steel_bar_round is a thick 30 cm
bar (2470 ml), and the sections span the tile (100 cm
long / 50 cm short).

The set, from the entries' names/descriptions/materials:

- small_steel_bar_flat (steel, light_gray, 5 cm):
  "A small rectangular bar of steel.  Ready to be
  milled on a milling machine." -- short flat prism,
  bright clean machined faces, end face showing the
  rectangular cross-section.
- small_lc_steel_bar_flat (lc_steel): same form, low
  carbon steel drawn a touch duller and warmer with
  faint mill-scale speckle.
- small_mc_steel_bar_flat (mc_steel): same form,
  medium carbon steel a touch darker and bluer, with
  a faint temper tint at the ends.
- small_steel_bar_round (steel, light_gray, 30 cm):
  "A small round bar of steel.  Ready to be turned on
  the lathe." -- thin cylinder with fine turning lines
  and a circular end cap.
- small_lc_steel_bar_round / small_mc_steel_bar_round:
  the same rod in the lc / mc finishes above.
- steel_bar_flat (steel, 10 cm, 250 ml): the full-size
  flat bar -- longer and thicker prism.
- steel_bar_round (steel, 30 cm, 2470 ml): the
  full-size round bar -- a thick cylinder, same lathe
  finish, much heavier in frame.
- steel_section_long (mc_steel, dark_gray, 100 cm):
  "A long section of carbon structural steel." -- an
  I-section beam spanning the tile, its I profile
  (two flanges, recessed web) drawn on the end face.
- steel_section_short (mc_steel, dark_gray, 50 cm):
  the same beam at half length.

Writes ONLY its own 10 folders + preview_item21.png.
PIL only; per-sprite random.Random(crc32(id)).
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
    "small_lc_steel_bar_flat",
    "small_lc_steel_bar_round",
    "small_mc_steel_bar_flat",
    "small_mc_steel_bar_round",
    "small_steel_bar_flat",
    "small_steel_bar_round",
    "steel_bar_flat",
    "steel_bar_round",
    "steel_section_long",
    "steel_section_short",
]

# grade palettes: (base, light, dark, edge)
STEEL = ((178, 182, 188, 255), (233, 236, 241, 255),
         (118, 122, 130, 255), (84, 88, 96, 255))
LC_STEEL = ((166, 163, 156, 255), (214, 211, 202, 255),
            (108, 105, 98, 255), (76, 73, 66, 255))
MC_STEEL = ((156, 163, 175, 255), (207, 215, 226, 255),
            (98, 105, 119, 255), (66, 72, 86, 255))
SECTION = ((94, 96, 100, 255), (142, 144, 148, 255),
           (56, 58, 62, 255), (34, 35, 38, 255))
TEMPER = (104, 122, 168, 255)   # faint blue temper tint
SCALE = (88, 84, 76, 255)       # mill-scale speckle on lc steel


def finish(img):
    return img.rotate(-45, resample=Image.NEAREST)


def flat_bar(rng, length, height, pal, grade):
    """Rectangular bar stock as a flat-shaded prism:
    light top face, mid front face, dark end face with
    the rectangular cross-section picked out."""
    base, light, dark, edge = pal
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    x0 = (32 - length) // 2 - 1
    x1 = x0 + length
    cy = 16
    y0, y1 = cy - height // 2, cy + (height - height // 2)
    dx, dy = 3, -3  # depth offset of the top face
    # top face
    d.polygon([(x0, y0), (x0 + dx, y0 + dy), (x1 + dx, y0 + dy),
               (x1, y0)], fill=light)
    # front face: light band under the top edge, base, dark hem
    d.rectangle([x0, y0, x1, y1], fill=base)
    d.line([(x0, y0), (x1, y0)], fill=light)
    d.line([(x0, y0 + 1), (x1, y0 + 1)], fill=light)
    d.line([(x0, y1 - 1), (x1, y1 - 1)], fill=dark)
    d.line([(x0, y1), (x1, y1)], fill=edge)
    # right end face (the milled cross-section)
    d.polygon([(x1, y0), (x1 + dx, y0 + dy), (x1 + dx, y1 + dy),
               (x1, y1)], fill=dark)
    d.line([(x1 + 1, y0 - 1), (x1 + 1, y1 - 1)], fill=base)
    if grade == "lc":
        for _ in range(7):  # mill-scale speckle
            sx = rng.randrange(x0 + 1, x1 - 1)
            sy = rng.randrange(y0 + 1, y1)
            d.point((sx, sy), fill=SCALE)
    if grade == "mc":  # faint temper tint near both ends
        for x in (x0 + 1, x1 - 2):
            d.line([(x, y0 + 2), (x, y1 - 2)], fill=TEMPER)
    return finish(img)


def round_bar(rng, length, height, pal, grade):
    """Round bar stock as a cylinder: shaded shaft with
    fine lathe-turning lines and a circular end cap at
    the upper (right) end."""
    base, light, dark, edge = pal
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    x0 = (32 - length) // 2 - 2
    x1 = x0 + length
    cy = 16
    r = height // 2
    y0, y1 = cy - r, cy + r
    # shaft with a cylindrical shade: dark crown, bright
    # band above centre, base, dark underside
    d.rectangle([x0, y0, x1, y1], fill=base)
    d.line([(x0, y0), (x1, y0)], fill=dark)
    d.line([(x0, y0 + 1), (x1, y0 + 1)], fill=base)
    d.line([(x0, cy - 1), (x1, cy - 1)], fill=light)
    d.line([(x0, cy), (x1, cy)], fill=light)
    d.line([(x0, y1 - 1), (x1, y1 - 1)], fill=dark)
    d.line([(x0, y1), (x1, y1)], fill=edge)
    # lathe-turning lines along the shaft (fine, low
    # contrast, so the rod does not read as threaded)
    mid = tuple((a + b) // 2 for a, b in zip(base, dark))
    for x in range(x0 + 4, x1 - 3, 4):
        d.line([(x, y0 + 2), (x, y1 - 2)], fill=mid)
    # left end: sawn face
    d.line([(x0, y0), (x0, y1)], fill=edge)
    d.line([(x0 + 1, y0 + 1), (x0 + 1, y1 - 1)], fill=dark)
    # right end cap: the circular cross-section
    d.ellipse([x1 - 2, y0, x1 + 2, y1], fill=dark)
    d.ellipse([x1 - 1, y0 + 1, x1 + 1, y1 - 1], fill=base)
    d.point((x1, cy), fill=light)
    if grade == "lc":
        soft = tuple((a + b) // 2 for a, b in zip(base, SCALE))
        for _ in range(3):
            sx = rng.randrange(x0 + 2, x1 - 3)
            sy = rng.randrange(y0 + 1, y1)
            d.point((sx, sy), fill=soft)
    if grade == "mc":
        d.line([(x0 + 2, y0 + 2), (x0 + 2, y1 - 2)], fill=TEMPER)
    return finish(img)


def section(rng, length, pal):
    """Structural steel section: an I-beam. Top and
    bottom flanges with a recessed darker web between
    them; the end face draws the I profile."""
    base, light, dark, edge = pal
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    x0 = (32 - length) // 2 - 1
    x1 = x0 + length
    cy = 16
    y0, y1 = cy - 6, cy + 5        # overall 12 px tall
    wy0, wy1 = cy - 2, cy + 1      # web band
    # web (recessed, darkest)
    d.rectangle([x0, wy0, x1, wy1], fill=dark)
    # flanges
    d.rectangle([x0, y0, x1, wy0 - 1], fill=base)
    d.rectangle([x0, wy1 + 1, x1, y1], fill=base)
    d.line([(x0, y0), (x1, y0)], fill=light)
    d.line([(x0, y0 + 1), (x1, y0 + 1)], fill=light)
    d.line([(x0, wy0 - 1), (x1, wy0 - 1)], fill=edge)
    d.line([(x0, wy1 + 1), (x1, wy1 + 1)], fill=edge)
    d.line([(x0, y1), (x1, y1)], fill=dark)
    # left end: cut face
    d.line([(x0, y0), (x0, y1)], fill=edge)
    # right end face: the I profile picked out in light
    ex = x1
    d.rectangle([ex, y0, ex + 2, y1], fill=dark)
    d.rectangle([ex, y0, ex + 2, wy0 - 1], fill=light)
    d.rectangle([ex, wy1 + 1, ex + 2, y1], fill=light)
    d.rectangle([ex + 1, wy0, ex + 1, wy1], fill=light)
    # faint scuffs along the top flange
    for _ in range(5):
        sx = rng.randrange(x0 + 2, x1 - 2)
        d.point((sx, y0 + 2), fill=dark)
    return finish(img)


def render(item_id, rng):
    if item_id == "small_steel_bar_flat":
        return flat_bar(rng, 16, 5, STEEL, "steel")
    if item_id == "small_lc_steel_bar_flat":
        return flat_bar(rng, 16, 5, LC_STEEL, "lc")
    if item_id == "small_mc_steel_bar_flat":
        return flat_bar(rng, 16, 5, MC_STEEL, "mc")
    if item_id == "steel_bar_flat":
        return flat_bar(rng, 22, 7, STEEL, "steel")
    if item_id == "small_steel_bar_round":
        return round_bar(rng, 24, 5, STEEL, "steel")
    if item_id == "small_lc_steel_bar_round":
        return round_bar(rng, 24, 5, LC_STEEL, "lc")
    if item_id == "small_mc_steel_bar_round":
        return round_bar(rng, 24, 5, MC_STEEL, "mc")
    if item_id == "steel_bar_round":
        return round_bar(rng, 24, 11, STEEL, "steel")
    if item_id == "steel_section_long":
        return section(rng, 27, SECTION)
    if item_id == "steel_section_short":
        return section(rng, 19, SECTION)
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
    cols, cell = 5, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item21.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item21.png {sheet.size}")


if __name__ == "__main__":
    main()
