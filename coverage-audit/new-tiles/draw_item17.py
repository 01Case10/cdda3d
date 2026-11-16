#!/usr/bin/env python3
"""Batch ITEM17 (Phase 2g.4 scope extension): head
attachments, final set.

17 ids from data/json/items/armor/head_attachments.json
(authoritative list embedded below; source list:
~/workspace/item-batches/17.txt). Copy-from chains resolved
against the data before drawing (each size variant inherits
its base item's name/description/materials). The set is one
family in three shapes across five steel grades, in XS size
(15 ids) plus two XL tempered-steel pieces:

- Chainmail aventail (xs_{ch,hc,lc,mc,qt}): "a wide chainmail
  draping designed to attach to a cap helmet, hanging down
  to extend protection to the face, neck and shoulders" --
  a flaring mail curtain under a riveted attachment band,
  with a shallow face notch at the top.
- Chainmail partial aventail / "face" (xs_{...} + xl_qt):
  "a wide chainmail ring designed to attach to a cap
  helmet, hanging down to extend protection to the face
  and neck" -- an oval ring of mail framing a face opening,
  with attachment tabs at the top.
- Armored face mask (xs_{...} + xl_qt): "a metal face mask
  designed to attach to a cap helmet to protect the lower
  half" -- a solid plate with nose notch, mouth slit and
  breathing holes, ornamented per its description: the mild
  steel one is a crude, dull facsimile (dented, untrimmed);
  the medium steel has at least some ornamentation (an
  etched line); the high steel has "a fancy mustache and
  everything" (a raised mustache over the mouth slit); the
  hardened steel has fancy gold inlays; the tempered steel
  has gold inlays and a gemstone setting (gold bezel with a
  red gem on the brow band) -- "apocalypse fancy".

Steel grades shade the metal: mild dull and dark, medium
mid-gray, high bright polished, hardened darker blue-gray,
tempered with a straw-tinted sheen. XS pieces sit smaller
in the frame; the two XL pieces fill it.

Items are objects on transparency, native 32x32, flat
UltiCa style, bodies fully opaque (alpha in {0,255});
translucency faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 17 folders + preview_item17.png.
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
    "xl_qt_chainmail_face",
    "xl_qt_steel_facemask",
    "xs_ch_chainmail_aventail",
    "xs_ch_chainmail_face",
    "xs_ch_steel_facemask",
    "xs_hc_chainmail_aventail",
    "xs_hc_chainmail_face",
    "xs_hc_steel_facemask",
    "xs_lc_chainmail_aventail",
    "xs_lc_chainmail_face",
    "xs_lc_steel_facemask",
    "xs_mc_chainmail_aventail",
    "xs_mc_chainmail_face",
    "xs_mc_steel_facemask",
    "xs_qt_chainmail_aventail",
    "xs_qt_chainmail_face",
    "xs_qt_steel_facemask",
]

# grade -> (base, dark, light, ring) steel palette
GRADES = {
    "lc": ((148, 148, 152, 255), (100, 100, 105, 255),
           (178, 178, 182, 255), (196, 196, 200, 255)),
    "mc": ((166, 168, 173, 255), (116, 118, 124, 255),
           (196, 198, 203, 255), (214, 216, 220, 255)),
    "hc": ((186, 189, 196, 255), (132, 136, 144, 255),
           (216, 219, 226, 255), (234, 236, 240, 255)),
    "ch": ((138, 146, 159, 255), (92, 100, 113, 255),
           (174, 182, 195, 255), (192, 200, 212, 255)),
    "qt": ((156, 156, 166, 255), (106, 108, 120, 255),
           (198, 186, 148, 255), (216, 214, 222, 255)),
}
GOLD = (212, 175, 55, 255)
GOLD_DARK = (160, 128, 32, 255)
GEM = (196, 30, 42, 255)
GEM_LIGHT = (240, 120, 130, 255)
LEATHER = (110, 82, 52, 255)
LEATHER_DARK = (84, 62, 38, 255)


def parse(item_id):
    size, rest = item_id.split("_", 1)
    grade, shape = rest.split("_", 1)
    return size, grade, shape


def mail_texture(rng, grade, rough):
    """Full-canvas chainmail: dark ground, offset rows of rings."""
    base, dark, light, ring = GRADES[grade]
    img = Image.new("RGBA", (32, 32), dark)
    d = ImageDraw.Draw(img)
    for row, y in enumerate(range(0, 33, 3)):
        off = 2 if row % 2 else 0
        for x in range(-2 + off, 33, 4):
            jx = rng.choice((-1, 0, 0, 1)) if rough else 0
            jy = rng.choice((-1, 0, 0)) if rough else 0
            d.ellipse([x + jx, y + jy, x + 3 + jx, y + 2 + jy],
                      outline=ring)
            d.point((x + 1 + jx, y + 1 + jy), fill=base)
    return img


def apply_mask(tex, mask):
    out = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    out.paste(tex, (0, 0), mask)
    return out


def shade_bottom(img, rows, color):
    """Darken the lowest rows of the opaque region (hang shading)."""
    px = img.load()
    for y in rows:
        for x in range(32):
            if px[x, y][3] == 255:
                px[x, y] = color


def aventail(rng, grade, size):
    inset = 1 if size == "xl" else 3
    top, bot = 5, 29 - (0 if size == "xl" else 1)
    half_top = 8 - inset // 2
    half_bot = 15 - inset
    mask = Image.new("L", (32, 32), 0)
    md = ImageDraw.Draw(mask)
    for y in range(top, bot + 1):
        t = (y - top) / max(1, (bot - top))
        half = half_top + (half_bot - half_top) * t
        md.line([16 - half, y, 16 + half, y], fill=255)
    # shallow face notch: clear a U at the top center
    md.ellipse([11, top - 2, 21, top + 6], fill=0)
    tex = mail_texture(rng, grade, rough=(grade == "lc"))
    img = apply_mask(tex, mask)
    d = ImageDraw.Draw(img)
    base, dark, light, ring = GRADES[grade]
    # attachment band across the top with rivets (the mounting feature)
    d.rectangle([16 - half_top - 1, top - 2, 16 + half_top + 1, top + 1],
                fill=LEATHER)
    d.line([16 - half_top - 1, top + 1, 16 + half_top + 1, top + 1],
           fill=LEATHER_DARK)
    for x in range(16 - half_top, 16 + half_top + 1, 4):
        if not (11 <= x <= 21):  # keep rivets out of the face notch
            d.point((x, top - 1), fill=ring)
    # scalloped bottom row of half-rings
    for x in range(16 - half_bot + 1, 16 + half_bot, 4):
        d.arc([x, bot - 1, x + 3, bot + 2], 0, 180, fill=ring)
    shade_bottom(img, range(bot - 2, bot + 1), dark)
    return img


def chainmail_face(rng, grade, size):
    grow = 2 if size == "xl" else 0
    cx, cy = 16, 17
    ow, oh = 13 + grow, 12 + grow          # outer oval half-extents
    iw, ih = 7 + grow // 2, 6 + grow // 2  # face opening
    mask = Image.new("L", (32, 32), 0)
    md = ImageDraw.Draw(mask)
    md.ellipse([cx - ow, cy - oh, cx + ow, cy + oh], fill=255)
    md.ellipse([cx - iw, cy - ih - 1, cx + iw, cy + ih - 1], fill=0)
    tex = mail_texture(rng, grade, rough=(grade == "lc"))
    img = apply_mask(tex, mask)
    d = ImageDraw.Draw(img)
    base, dark, light, ring = GRADES[grade]
    # rim highlight on the outer edge, shadow inside the opening
    d.ellipse([cx - ow, cy - oh, cx + ow, cy + oh], outline=light)
    d.ellipse([cx - iw, cy - ih - 1, cx + iw, cy + ih - 1], outline=dark)
    # attachment tabs at the top (mounting to the cap helmet)
    for tx in (cx - 6, cx + 4):
        d.rectangle([tx, cy - oh - 2, tx + 2, cy - oh + 1], fill=LEATHER)
        d.point((tx + 1, cy - oh - 1), fill=ring)
    return img


def facemask(rng, grade, size):
    grow = 2 if size == "xl" else 0
    base, dark, light, ring = GRADES[grade]
    cx = 16
    top, bot = 6 - grow // 2, 27 + grow // 2
    hw_top, hw_mid, hw_bot = 9 + grow, 11 + grow, 6 + grow
    mask = Image.new("L", (32, 32), 0)
    md = ImageDraw.Draw(mask)
    for y in range(top, bot + 1):
        t = (y - top) / max(1, (bot - top))
        if t < 0.45:
            half = hw_top + (hw_mid - hw_top) * (t / 0.45)
        else:
            half = hw_mid + (hw_bot - hw_mid) * ((t - 0.45) / 0.55)
        md.line([cx - half, y, cx + half, y], fill=255)
    # nose notch at the top center
    md.polygon([(cx - 2, top), (cx + 2, top), (cx, top + 3)], fill=0)
    img = apply_mask(Image.new("RGBA", (32, 32), base), mask)
    d = ImageDraw.Draw(img)
    # brushed shading: light down the left-center, dark at the chin
    for y in range(top, bot + 1):
        for x in range(32):
            if img.getpixel((x, y))[3] == 255:
                if x <= cx - 4:
                    img.putpixel((x, y), light)
                elif x >= cx + 5:
                    img.putpixel((x, y), dark)
    d = ImageDraw.Draw(img)
    # brow band (mounting strip) + corner loops
    d.rectangle([cx - hw_top, top, cx + hw_top, top + 2], fill=dark)
    d.rectangle([cx - hw_top - 1, top, cx - hw_top + 1, top + 3],
                outline=dark)
    d.rectangle([cx + hw_top - 1, top, cx + hw_top + 1, top + 3],
                outline=dark)
    # mouth slit + breathing holes
    my = top + (bot - top) * 2 // 3
    d.line([cx - 4, my, cx + 4, my], fill=dark)
    for hx in range(cx - 3, cx + 4, 2):
        for hy in (my + 3, my + 5):
            d.point((hx, hy), fill=dark)
    # grade ornamentation, per the descriptions
    if grade == "lc":
        # crude and dull: hammer dents, no trim
        for _ in range(9):
            x = rng.randrange(cx - hw_mid + 1, cx + hw_mid - 1)
            y = rng.randrange(top + 4, bot - 1)
            if img.getpixel((x, y))[3] == 255:
                d.point((x, y), fill=dark)
    elif grade == "mc":
        # at least some ornamentation: one etched line
        ey = top + 5
        d.line([cx - hw_top + 1, ey, cx + hw_top - 1, ey], fill=light)
    elif grade == "hc":
        # a fancy mustache and everything: raised mustache
        # over the mouth slit
        for sgn in (-1, 1):
            for i in range(5):
                x = cx + sgn * (1 + i)
                y = my - 3 + (i * i) // 6
                d.line([x, y, x, y + 1], fill=dark)
                d.point((x, y - 1), fill=light)
        d.point((cx, my - 3), fill=dark)
    elif grade in ("ch", "qt"):
        # fancy gold inlays: gold edge trim + brow accents
        d.ellipse([cx - hw_mid, top + 1, cx + hw_mid, bot], outline=GOLD)
        d.line([cx - hw_top + 1, top + 1, cx + hw_top - 1, top + 1],
               fill=GOLD)
        if grade == "qt":
            # gemstone setting on the brow band
            d.ellipse([cx - 2, top - 1, cx + 2, top + 3], outline=GOLD)
            d.point((cx, top), fill=GEM)
            d.point((cx, top + 1), fill=GEM)
            d.point((cx - 1, top + 1), fill=GEM_LIGHT)
    return img


def render(item_id, rng):
    size, grade, shape = parse(item_id)
    if shape == "chainmail_aventail":
        return aventail(rng, grade, size)
    if shape == "chainmail_face":
        return chainmail_face(rng, grade, size)
    if shape == "steel_facemask":
        return facemask(rng, grade, size)
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
    cols, cell = 6, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item17.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item17.png {sheet.size}")


if __name__ == "__main__":
    main()
