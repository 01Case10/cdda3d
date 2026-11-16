#!/usr/bin/env python3
"""Batch ITEM29 (Phase 2g.4 scope extension): integrated
armor -- the mutation-grown body coverings and features
from data/json/items/armor/integrated.json.

52 ids (authoritative list embedded below; source list:
~/workspace/item-batches/29.txt). All 52 are concrete
entries in integrated.json; none uses copy-from.

These are not garments: they are the survivor's own
mutated body -- fur, scales, chitin, bark, shells, horns,
claws, fangs -- plus three oddities (bionic eye lenses,
an implanted air filter, a fragment of reality). Each is
drawn as the feature itself, front-on:

- Skin coverings (fur / scales / chitin / bark / patchwork
  / plantskin / feathers) are drawn as a body panel: a
  rounded swatch of the covering with its texture worked
  in -- fur strokes, overlapping scale rows, plate seams,
  bark fissures -- in the entry's own color/material.
  Molted ("fresh") variants are paler and softer; summer
  coats are thinner; patchwork grades gain bone and
  alien-carapace plates as they advance.
- Head/hand/foot features (antlers, horns, claws, talons,
  fangs, teeth, mandibles) are drawn as the feature on a
  small mount of the body part it grows from.
- Shells are domed back shells, growing larger and
  pearlier across shell / shell2 / shell3.
- armor_bio_eyes: two dark ballistic lenses on a strap.
- internal_bio_purifier: the tracheal filter implant --
  a mesh cylinder with tubes.
- integrated_interstice_aura: flesh panel wrapped in a
  dark penumbral halo with void flecks.
- item_portal_storm_dimension_returner: a dark shard of
  reality with glowing fracture lines.

All sprite alpha is in {0, 255}, asserted per sprite
(this Pillow build replaces rather than blends, so
sheens are pale opaque colors).

Writes ONLY its own 52 folders + preview_item29.png.
PIL only; per-sprite random.Random(crc32(id)).
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "armor_bio_eyes",
    "integrated_antlers",
    "integrated_bark",
    "integrated_bark2_a",
    "integrated_bark2_b",
    "integrated_bark2_c",
    "integrated_carapace",
    "integrated_carapace_molted",
    "integrated_chitin",
    "integrated_chitin2",
    "integrated_chitin2_molted",
    "integrated_chitin3",
    "integrated_chitin3_claws",
    "integrated_chitin3_molted",
    "integrated_claws",
    "integrated_claws_rat",
    "integrated_claws_st",
    "integrated_fangs",
    "integrated_fangs_spider",
    "integrated_feathers",
    "integrated_feline_fur",
    "integrated_fishscales",
    "integrated_fur",
    "integrated_horns_curled",
    "integrated_horns_pointed",
    "integrated_horns_small",
    "integrated_interstice_aura",
    "integrated_lightfur",
    "integrated_lupine_fur",
    "integrated_lupine_fur_summer",
    "integrated_lynx_fur",
    "integrated_mandibles",
    "integrated_patchskin1",
    "integrated_patchskin2",
    "integrated_patchskin3",
    "integrated_plantskin",
    "integrated_rabbitfur",
    "integrated_reptilescales",
    "integrated_saber_teeth",
    "integrated_scales",
    "integrated_shark_teeth",
    "integrated_shell",
    "integrated_shell2",
    "integrated_shell3",
    "integrated_sparse_scales",
    "integrated_talons",
    "integrated_toe_talons",
    "integrated_ursine_fur",
    "integrated_ursine_fur_summer",
    "integrated_vampire_fangs",
    "internal_bio_purifier",
    "item_portal_storm_dimension_returner",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.\n")

# shared tones
SKIN = (216, 170, 138, 255)
SKIN_D = (188, 140, 106, 255)
BONE = (204, 194, 168, 255)
BONE_D = (158, 146, 118, 255)
BONE_L = (232, 224, 200, 255)
HORN = (148, 146, 138, 255)
HORN_D = (104, 102, 96, 255)
MOUTH = (96, 34, 36, 255)
MOUTH_D = (62, 20, 24, 255)


def panel(d, fill, outline, box=(4, 5, 28, 28), radius=6):
    d.rounded_rectangle(list(box), radius=radius, fill=fill,
                        outline=outline, width=1)


def fur_strokes(d, rng, tones, box, count, length=(2, 4), width=1):
    x0, y0, x1, y1 = box
    for _ in range(count):
        x = rng.randrange(x0, x1)
        y = rng.randrange(y0, y1)
        ln = rng.randint(*length)
        c = tones[rng.randrange(len(tones))]
        d.line([(x, y), (x + rng.randint(-1, 1), y + ln)], fill=c,
               width=width)


def fur_panel(rng, base, dark, light, count=210, length=(2, 4),
              patches=None):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, base, dark)
    if patches:  # bare skin showing through (patchy coats)
        for (x, y, r) in patches:
            d.ellipse([x - r, y - r, x + r, y + r], fill=SKIN)
    fur_strokes(d, rng, [dark, base, light], (7, 8, 26, 26), count,
                length)
    return img


def scale_rows(d, rng, tones, outline, box=(6, 8, 27, 27), sw=7, sh=6,
               sparse=False):
    x0, y0, x1, y1 = box
    row = 0
    y = y0
    while y < y1:
        off = (sw // 2) if row % 2 else 0
        x = x0 - off
        while x < x1:
            if not (sparse and rng.random() < 0.55):
                c = tones[rng.randrange(len(tones))]
                d.ellipse([x, y, x + sw, y + sh + 3], fill=c,
                          outline=outline)
                d.arc([x + 1, y + 1, x + sw - 1, y + sh + 2], 20, 160,
                      fill=tones[-1])
            x += sw
        y += sh
        row += 1


def scale_panel(rng, base, tones, outline, sparse=False):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, SKIN if sparse else base, outline)
    scale_rows(d, rng, tones, outline, sparse=sparse)
    return img


def plate_panel(rng, base, dark, light, bands=3, soft=False,
                rivets=True):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, base, dark)
    # horizontal plate bands with a center seam (elytra-like)
    y0, y1 = 6, 27
    step = (y1 - y0) // bands
    for i in range(1, bands):
        y = y0 + i * step
        d.line([(6, y), (26, y)], fill=dark, width=1)
        if not soft:
            d.line([(6, y + 1), (26, y + 1)], fill=light, width=1)
    d.line([(16, 6), (16, 27)], fill=dark, width=1)
    if not soft:
        d.line([(17, 6), (17, 27)], fill=light, width=1)
    if rivets:
        for (x, y) in [(8, 9), (24, 9), (8, 24), (24, 24), (16, 16)]:
            d.point((x, y), fill=dark)
            d.point((x + 1, y), fill=dark)
    # glossy sheen
    if not soft:
        d.line([(8, 8), (13, 8)], fill=light, width=2)
    return img


# ----------------------------------------------------------------
# skin coverings

def draw_fur(rng):
    return fur_panel(rng, (168, 170, 174, 255), (120, 122, 128, 255),
                     (206, 208, 212, 255), count=240)


def draw_lightfur(rng):
    return fur_panel(rng, (232, 230, 226, 255), (196, 192, 186, 255),
                     (250, 248, 244, 255), count=90, length=(2, 3),
                     patches=[(11, 13, 3), (21, 19, 4), (14, 24, 3)])


def draw_rabbitfur(rng):
    return fur_panel(rng, (238, 236, 232, 255), (198, 194, 188, 255),
                     (252, 250, 248, 255), count=300, length=(1, 3))


def draw_feline_fur(rng):
    img = fur_panel(rng, (148, 106, 66, 255), (106, 72, 40, 255),
                    (186, 146, 96, 255), count=230, length=(2, 3))
    d = ImageDraw.Draw(img)
    # faint sleek stripes
    for x in (10, 16, 22):
        d.line([(x, 8), (x - 2, 26)], fill=(106, 72, 40, 255))
    return img


def draw_lupine_fur(rng):
    return fur_panel(rng, (110, 110, 116, 255), (72, 72, 78, 255),
                     (152, 152, 158, 255), count=250)


def draw_lupine_summer(rng):
    return fur_panel(rng, (128, 128, 132, 255), (88, 88, 92, 255),
                     (168, 168, 170, 255), count=120, length=(2, 3),
                     patches=[(20, 22, 3)])


def draw_ursine_fur(rng):
    return fur_panel(rng, (104, 102, 106, 255), (64, 62, 66, 255),
                     (148, 146, 150, 255), count=280, length=(4, 6))


def draw_ursine_summer(rng):
    return fur_panel(rng, (118, 116, 118, 255), (78, 76, 78, 255),
                     (158, 156, 158, 255), count=170, length=(3, 4))


def draw_lynx_fur(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # a thick ruff collar around a neck opening
    d.ellipse([3, 6, 29, 29], fill=(148, 106, 66, 255))
    d.ellipse([3, 6, 29, 29], outline=(106, 72, 40, 255))
    fur_strokes(d, rng, [(106, 72, 40, 255), (148, 106, 66, 255),
                         (190, 150, 100, 255)], (6, 9, 27, 27), 220,
                (3, 5))
    d.ellipse([11, 13, 21, 23], fill=SKIN)
    d.ellipse([11, 13, 21, 23], outline=SKIN_D)
    # ear-tuft tips at the top of the ruff
    d.polygon([(7, 8), (5, 2), (10, 6)], fill=(106, 72, 40, 255))
    d.polygon([(25, 8), (27, 2), (22, 6)], fill=(106, 72, 40, 255))
    return img


def draw_feathers(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, (226, 224, 220, 255), (168, 164, 158, 255))
    # layered feather teardrops, rows top to bottom
    iri = [(168, 220, 208, 255), (198, 182, 232, 255),
           (240, 246, 244, 255)]
    for row, y in enumerate((8, 13, 18, 23)):
        off = 3 if row % 2 else 0
        for x in range(5 + off, 26, 6):
            d.polygon([(x, y), (x + 4, y), (x + 2, y + 6)],
                      fill=(238, 236, 232, 255),
                      outline=(178, 174, 168, 255))
            d.line([(x + 2, y + 1), (x + 2, y + 5)],
                   fill=(198, 194, 188, 255))
            if rng.random() < 0.5:
                d.point((x + 2, y + 3), fill=iri[rng.randrange(3)])
                d.point((x + 3, y + 3), fill=iri[rng.randrange(3)])
    return img


def draw_fishscales(rng):
    return scale_panel(rng, (216, 138, 120, 255),
                       [(226, 156, 138, 255), (206, 124, 108, 255),
                        (238, 178, 158, 255)], (172, 92, 82, 255))


def draw_scales(rng):
    return scale_panel(rng, (206, 118, 104, 255),
                       [(216, 136, 120, 255), (192, 102, 90, 255),
                        (230, 160, 142, 255)], (150, 70, 62, 255))


def draw_reptilescales(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, (186, 108, 94, 255), (128, 64, 56, 255))
    # heavy lamellar rows: bigger, thicker scales with bone studs
    scale_rows(d, rng, [(198, 122, 106, 255), (176, 96, 84, 255),
                        (214, 146, 126, 255)], (128, 64, 56, 255),
               sw=9, sh=7)
    for (x, y) in [(11, 12), (20, 18), (13, 24)]:
        d.point((x, y), fill=BONE)
        d.point((x + 1, y), fill=BONE_D)
    return img


def draw_sparse_scales(rng):
    return scale_panel(rng, SKIN, [(214, 140, 122, 255),
                                    (198, 116, 102, 255),
                                    (228, 164, 144, 255)],
                       (168, 96, 84, 255), sparse=True)


def draw_plantskin(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, (150, 198, 110, 255), (96, 148, 62, 255))
    # living veins and small leaflets
    d.line([(16, 7), (16, 27)], fill=(110, 168, 74, 255), width=2)
    for y in (11, 16, 21, 25):
        d.line([(16, y), (9, y + 3)], fill=(110, 168, 74, 255))
        d.line([(16, y), (23, y + 3)], fill=(110, 168, 74, 255))
    for (x, y) in [(10, 10), (22, 12), (12, 20), (21, 23), (16, 14)]:
        d.polygon([(x, y), (x + 4, y + 1), (x + 1, y + 4)],
                  fill=(172, 216, 128, 255))
    d.line([(7, 8), (12, 8)], fill=(196, 232, 156, 255), width=2)
    return img


def bark_marks(d, rng, box, count=7):
    x0, y0, x1, y1 = box
    for _ in range(count):
        if x1 - 2 <= x0 or y1 - y0 < 3:
            return
        x = rng.randrange(x0, x1 - 2)
        y = rng.randrange(y0, max(y0 + 1, y1 - 6))
        ln = rng.randint(3, max(3, min(8, y1 - y)))
        # dark fissure with a lighter ridge beside it
        d.line([(x, y), (x + rng.randint(-1, 1), y + ln)],
               fill=(70, 48, 30, 255), width=2)
        d.line([(x + 2, y), (x + 2, y + ln)], fill=(168, 128, 88, 255))


def draw_bark(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, (128, 94, 58, 255), (84, 60, 36, 255))
    bark_marks(d, rng, (7, 8, 26, 24), count=9)
    d.rectangle([14, 5, 19, 28], fill=(140, 104, 64, 255))
    bark_marks(d, rng, (15, 8, 19, 24), count=2)
    return img


def draw_bark2_a(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, SKIN, SKIN_D)
    # bark spread evenly in patches across the body
    for (x, y, w, h) in [(6, 7, 8, 7), (18, 7, 8, 7), (11, 14, 9, 7),
                         (6, 21, 8, 6), (19, 21, 8, 6), (21, 14, 6, 6)]:
        d.rounded_rectangle([x, y, x + w, y + h], radius=2,
                            fill=(128, 94, 58, 255),
                            outline=(84, 60, 36, 255))
        bark_marks(d, rng, (x + 1, y + 1, x + w - 1, y + h - 2),
                   count=2)
    return img


def draw_bark2_b(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, SKIN, SKIN_D)
    # bark pulled together into one thick torso shield
    d.rounded_rectangle([9, 7, 24, 27], radius=4,
                        fill=(128, 94, 58, 255),
                        outline=(84, 60, 36, 255))
    d.rounded_rectangle([12, 10, 21, 24], radius=3,
                        fill=(140, 104, 64, 255))
    bark_marks(d, rng, (11, 9, 23, 24), count=7)
    return img


def draw_bark2_c(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, SKIN, SKIN_D)
    # latticed bark strips in a diagonal crosshatch frame
    for off in (-12, -4, 4, 12):
        d.line([(6 + off, 27), (20 + off, 6)], fill=(128, 94, 58, 255),
               width=3)
        d.line([(26 - off, 27), (12 - off, 6)],
               fill=(140, 104, 64, 255), width=3)
    d.rounded_rectangle([4, 5, 28, 28], radius=6,
                        outline=(84, 60, 36, 255), width=2)
    for (x, y) in [(10, 11), (21, 11), (16, 17), (10, 23), (21, 23)]:
        d.point((x, y), fill=(70, 48, 30, 255))
    return img


def draw_carapace(rng):
    return plate_panel(rng, (140, 102, 62, 255), (96, 66, 36, 255),
                       (186, 148, 96, 255), bands=3)


def draw_carapace_molted(rng):
    return plate_panel(rng, (196, 168, 132, 255), (150, 124, 92, 255),
                       (214, 192, 158, 255), bands=3, soft=True,
                       rivets=False)


def draw_chitin(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, (110, 148, 74, 255), (70, 102, 44, 255))
    # smooth epicuticle with fine segment creases
    for y in (12, 18, 24):
        d.arc([6, y - 4, 26, y + 4], 20, 160, fill=(84, 120, 54, 255))
    d.line([(8, 8), (14, 8)], fill=(160, 196, 116, 255), width=2)
    return img


def draw_chitin2(rng):
    return plate_panel(rng, (104, 106, 110, 255), (62, 64, 68, 255),
                       (150, 152, 156, 255), bands=4)


def draw_chitin2_molted(rng):
    return plate_panel(rng, (168, 172, 158, 255), (124, 128, 114, 255),
                       (196, 200, 186, 255), bands=4, soft=True,
                       rivets=False)


def draw_chitin3(rng):
    return plate_panel(rng, (128, 90, 52, 255), (84, 56, 30, 255),
                       (178, 138, 88, 255), bands=4)


def draw_chitin3_molted(rng):
    return plate_panel(rng, (188, 156, 118, 255), (142, 112, 80, 255),
                       (208, 182, 144, 255), bands=4, soft=True,
                       rivets=False)


def patchskin(rng, grade):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    panel(d, (216, 150, 128, 255), (176, 108, 88, 255))
    # splattered patches of fur, hide and chitin
    patch_cols = [(150, 110, 70, 255), (110, 120, 60, 255),
                  (196, 170, 140, 255), (226, 224, 218, 255)]
    n = 7 + grade * 2
    for _ in range(n):
        x = rng.randrange(7, 24)
        y = rng.randrange(8, 25)
        r = rng.randint(2, 4)
        c = patch_cols[rng.randrange(len(patch_cols))]
        d.ellipse([x - r, y - r, x + r, y + r], fill=c)
        if c == patch_cols[0]:
            fur_strokes(d, rng, [c], (x - 2, y - 2, x + 2, y + 2), 5,
                        (1, 2))
    if grade >= 2:  # bone plates knitted in
        for (x, y) in [(11, 12), (20, 20)]:
            d.polygon([(x, y), (x + 5, y + 1), (x + 4, y + 5),
                       (x, y + 4)], fill=BONE, outline=BONE_D)
    if grade >= 3:  # alien carapace plate, dark and glossy
        d.polygon([(14, 9), (23, 11), (22, 19), (14, 17)],
                  fill=(70, 52, 84, 255), outline=(40, 30, 52, 255))
        d.line([(16, 11), (20, 12)], fill=(140, 120, 170, 255),
               width=2)
        d.polygon([(8, 20), (14, 19), (15, 25), (9, 26)],
                  fill=BONE, outline=BONE_D)
    return img


def draw_patchskin1(rng):
    return patchskin(rng, 1)


def draw_patchskin2(rng):
    return patchskin(rng, 2)


def draw_patchskin3(rng):
    return patchskin(rng, 3)


def draw_interstice_aura(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # penumbral halo: dark violet shadow wrapped around the flesh
    d.rounded_rectangle([2, 3, 30, 30], radius=8,
                        fill=(52, 38, 74, 255))
    d.rounded_rectangle([4, 5, 28, 28], radius=7,
                        fill=(84, 58, 108, 255))
    panel(d, (224, 158, 138, 255), (188, 116, 100, 255),
          box=(7, 8, 25, 26), radius=5)
    fur_strokes(d, rng, [(196, 128, 110, 255)], (9, 10, 24, 24), 24,
                (1, 2))
    # void flecks and wrong-colored sparks in the halo
    for (x, y, c) in [(5, 7, (240, 240, 250, 255)),
                      (26, 9, (170, 220, 255, 255)),
                      (6, 25, (255, 190, 220, 255)),
                      (25, 24, (240, 240, 250, 255)),
                      (16, 4, (170, 220, 255, 255))]:
        d.point((x, y), fill=c)
        d.point((x + 1, y), fill=c)
    return img


# ----------------------------------------------------------------
# head features: antlers, horns

def draw_antlers(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # scalp mount
    d.pieslice([9, 24, 23, 34], 180, 360, fill=SKIN, outline=SKIN_D)
    for flip in (1, -1):
        cx = 16 + flip * 5
        # main beam sweeping up and out, with tines
        beam = [(cx, 26), (cx + flip * 3, 18), (cx + flip * 6, 10),
                (cx + flip * 7, 4)]
        d.line(beam, fill=HORN, width=3)
        d.line(beam, fill=HORN_D, width=1)
        for (tx, ty, tl) in [(2, 20, 4), (4, 13, 5), (6, 8, 4)]:
            bx = cx + flip * tx
            d.line([(bx, ty + 3), (bx + flip * 2, ty + 3 - tl)],
                   fill=HORN, width=2)
        d.point((cx + flip * 7, 3), fill=BONE)
    return img


def draw_horns_curled(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.pieslice([9, 24, 23, 34], 180, 360, fill=SKIN, outline=SKIN_D)
    for flip in (1, -1):
        cx = 16 + flip * 8
        # ram curl: concentric arcs tightening inward
        for (box, st, en) in [((cx - 7, 8, cx + 7, 22), 300, 160),
                              ((cx - 5, 11, cx + 5, 20), 280, 140),
                              ((cx - 3, 13, cx + 3, 19), 260, 120)]:
            d.arc(list(box), st if flip > 0 else 180 - en,
                  en if flip > 0 else 180 - st, fill=HORN, width=3)
        d.point((cx, 16), fill=HORN_D)
    return img


def draw_horns_pointed(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.pieslice([9, 24, 23, 34], 180, 360, fill=SKIN, outline=SKIN_D)
    for flip in (1, -1):
        bx = 16 + flip * 4
        tip = (16 + flip * 10, 2)
        d.polygon([(bx - 2, 26), (bx + 2, 26), tip], fill=HORN)
        d.line([(bx, 26), tip], fill=HORN_D, width=1)
        d.point(tip, fill=BONE_L)
        # ring ridges
        for t in (0.35, 0.6):
            x = bx + (tip[0] - bx) * t
            y = 26 + (tip[1] - 26) * t
            d.point((int(x), int(y)), fill=HORN_D)
    return img


def draw_horns_small(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.pieslice([9, 22, 23, 32], 180, 360, fill=SKIN, outline=SKIN_D)
    for flip in (1, -1):
        bx = 16 + flip * 5
        d.polygon([(bx - 2, 24), (bx + 2, 24), (bx, 15)],
                  fill=(178, 176, 170, 255))
        d.point((bx, 15), fill=BONE_L)
    return img


# ----------------------------------------------------------------
# claws and talons

def claw_hand(rng, claw_col, claw_dark, style, fingers=4):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # palm
    d.rounded_rectangle([8, 18, 25, 30], radius=4, fill=SKIN,
                        outline=SKIN_D)
    xs = [10, 14, 18, 22][:fingers]
    for i, x in enumerate(xs):
        fl = 8 - (i % 2) * 2
        d.rectangle([x, 18 - fl + 6, x + 2, 20], fill=SKIN)
        # claw: curved tapered blade from the fingertip
        ty = 18 - fl + 6
        if style == "gnarled":
            d.polygon([(x, ty), (x + 2, ty), (x + 4, ty - 3),
                       (x + 1, ty - 6), (x - 1, ty - 3)],
                      fill=claw_col, outline=claw_dark)
        elif style == "steel":
            d.polygon([(x, ty), (x + 2, ty), (x + 1, ty - 9)],
                      fill=(216, 220, 226, 255), outline=claw_dark)
            d.line([(x + 1, ty - 2), (x + 1, ty - 8)],
                   fill=(250, 252, 255, 255))
        else:
            d.polygon([(x, ty), (x + 2, ty), (x + 3, ty - 4),
                       (x + 1, ty - 8)], fill=claw_col,
                      outline=claw_dark)
    return img


def draw_claws(rng):
    return claw_hand(rng, BONE, BONE_D, "long")


def draw_claws_rat(rng):
    return claw_hand(rng, (188, 178, 158, 255), BONE_D, "gnarled")


def draw_claws_st(rng):
    return claw_hand(rng, (200, 204, 210, 255), (120, 124, 130, 255),
                     "steel")


def draw_chitin3_claws(rng):
    return claw_hand(rng, (150, 112, 70, 255), (96, 66, 36, 255),
                     "long")


def draw_talons(rng):
    img = claw_hand(rng, BONE, BONE_D, "long")
    d = ImageDraw.Draw(img)
    # the two index-finger talons are far longer -- wicked daggers
    for x in (10, 22):
        d.polygon([(x, 16), (x + 3, 16), (x + 4, 6), (x + 2, 2),
                   (x, 8)], fill=BONE, outline=BONE_D)
    return img


def draw_toe_talons(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # foot: heel at bottom, toes fanning at top
    d.rounded_rectangle([10, 14, 22, 30], radius=5, fill=SKIN,
                        outline=SKIN_D)
    d.ellipse([8, 24, 24, 31], fill=SKIN, outline=SKIN_D)
    for x in (10, 14, 18, 21):
        d.ellipse([x, 10, x + 3, 15], fill=SKIN, outline=SKIN_D)
        d.polygon([(x, 11), (x + 3, 11), (x + 3, 5), (x + 1, 2),
                   (x - 1, 6)], fill=BONE, outline=BONE_D)
    return img


# ----------------------------------------------------------------
# teeth and mouthparts

def fang_pair(rng, length, width=3, curve=0, col=BONE, dark=BONE_D):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # upper jaw band
    d.rectangle([5, 4, 27, 10], fill=MOUTH)
    d.rectangle([5, 4, 27, 6], fill=MOUTH_D)
    d.line([(5, 10), (27, 10)], fill=(150, 60, 60, 255), width=1)
    for cx in (11, 21):
        tip = (cx + curve, 10 + length)
        d.polygon([(cx - width // 2, 10), (cx + width // 2 + 1, 10),
                   tip], fill=col, outline=dark)
        d.point(tip, fill=(250, 250, 246, 255))
    # lesser teeth between and beside
    for x in (7, 15, 17, 25):
        d.polygon([(x, 10), (x + 2, 10), (x + 1, 13)], fill=col)
    return img


def draw_fangs(rng):
    return fang_pair(rng, 11)


def draw_vampire_fangs(rng):
    return fang_pair(rng, 15, width=2, curve=1)


def draw_saber_teeth(rng):
    return fang_pair(rng, 20, width=4, curve=-1)


def draw_shark_teeth(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rectangle([4, 6, 28, 12], fill=MOUTH)
    d.rectangle([4, 6, 28, 8], fill=MOUTH_D)
    # a row of serrated triangular teeth, with a second row behind
    for x in range(6, 26, 4):
        d.polygon([(x, 16), (x + 3, 16), (x + 1, 20)], fill=BONE_D)
    for x in range(5, 27, 4):
        d.polygon([(x, 12), (x + 4, 12), (x + 2, 18)], fill=BONE,
                  outline=BONE_D)
        d.point((x + 1, 15), fill=BONE_D)
        d.point((x + 3, 15), fill=BONE_D)
    return img


def draw_fangs_spider(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # two sclerotin chelicerae with folding fangs angled inward
    for flip in (1, -1):
        cx = 16 + flip * 7
        d.rounded_rectangle([cx - 4, 6, cx + 4, 15], radius=3,
                            fill=(140, 102, 62, 255),
                            outline=(96, 66, 36, 255))
        d.polygon([(cx - 2, 14), (cx + 2, 14),
                   (cx - flip * 4, 24), (cx - flip * 5, 20)],
                  fill=(120, 84, 48, 255), outline=(84, 56, 30, 255))
    d.rectangle([12, 4, 20, 8], fill=(110, 78, 46, 255))
    return img


def draw_mandibles(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # jaw base
    d.rounded_rectangle([11, 20, 21, 28], radius=3,
                        fill=(90, 92, 96, 255), outline=(58, 60, 64, 255))
    for flip in (1, -1):
        # great curved pincer sweeping in from the side
        pts = []
        for t in range(11):
            a = math.pi * (0.15 + 0.75 * t / 10)
            x = 16 + flip * 13 * math.sin(a)
            y = 24 - 13 * math.cos(a) + 4
            pts.append((x, y))
        d.line(pts, fill=(104, 106, 110, 255), width=3)
        d.line(pts, fill=(66, 68, 72, 255), width=1)
        tip = pts[-1]
        d.point(tip, fill=BONE_L)
        # teeth notches on the inner edge
        for t in (3, 5, 7):
            x, y = pts[t]
            d.point((int(x - flip * 2), int(y)), fill=(66, 68, 72, 255))
    return img


# ----------------------------------------------------------------
# shells

def draw_shell_base(rng, size, nacre=False):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx, cy = 16, 19
    rx, ry = size, int(size * 0.82)
    body = (206, 106, 168, 255)
    dark = (156, 66, 126, 255)
    light = (232, 148, 198, 255)
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=body,
              outline=dark)
    # growth ridges
    for f in (0.72, 0.45):
        d.ellipse([cx - rx * f, cy - ry * f, cx + rx * f, cy + ry * f],
                  outline=dark)
    d.arc([cx - rx, cy - ry, cx + rx, cy + ry], 180, 360, fill=light,
          width=2)
    # rim opening at the bottom
    d.pieslice([cx - rx // 2, cy + ry // 3, cx + rx // 2,
                cy + ry + 3], 0, 180, fill=(120, 48, 96, 255))
    if nacre:
        # pearly sheen streaks
        for (x, y, c) in [(cx - 5, cy - 6, (250, 230, 240, 255)),
                          (cx + 2, cy - 8, (210, 240, 240, 255)),
                          (cx + 6, cy - 3, (250, 230, 240, 255)),
                          (cx - 8, cy - 2, (226, 214, 250, 255))]:
            d.line([(x, y), (x + 3, y + 1)], fill=c, width=2)
    return img


def draw_shell(rng):
    return draw_shell_base(rng, 11)


def draw_shell2(rng):
    return draw_shell_base(rng, 13)


def draw_shell3(rng):
    return draw_shell_base(rng, 14, nacre=True)


# ----------------------------------------------------------------
# singles

def draw_bio_eyes(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # strap across the top
    d.rectangle([3, 10, 29, 14], fill=(70, 70, 76, 255))
    d.rectangle([3, 10, 29, 11], fill=(96, 96, 102, 255))
    for cx in (10, 22):
        # sealed lens housings
        d.ellipse([cx - 6, 9, cx + 6, 23], fill=(88, 88, 94, 255),
                  outline=(52, 52, 58, 255))
        d.ellipse([cx - 4, 11, cx + 4, 21], fill=(44, 48, 54, 255))
        # ballistic-glass sheen
        d.line([(cx - 3, 13), (cx, 13)], fill=(190, 210, 220, 255),
               width=2)
        d.point((cx + 2, 18), fill=(120, 140, 150, 255))
    # tear-duct tube routing down to the mouth
    d.line([(16, 21), (16, 26)], fill=(88, 88, 94, 255), width=2)
    d.point((16, 27), fill=(52, 52, 58, 255))
    return img


def draw_bio_purifier(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # intake tube curving in at the top
    d.arc([10, 0, 26, 12], 180, 330, fill=(150, 152, 156, 255),
          width=3)
    # filter body: nanotube mesh cylinder
    d.rounded_rectangle([10, 9, 22, 27], radius=3,
                        fill=(168, 170, 174, 255),
                        outline=(110, 112, 116, 255))
    for y in range(13, 26, 3):
        for x in range(12, 21, 2):
            d.point((x, y), fill=(110, 112, 116, 255))
    # light_red status ring (its data color)
    d.rectangle([10, 17, 22, 19], fill=(216, 120, 108, 255))
    d.point((16, 18), fill=(250, 220, 210, 255))
    # trachea connector below
    d.rectangle([14, 27, 18, 30], fill=(110, 112, 116, 255))
    return img


def draw_dimension_returner(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # a jagged fragment of reality, dark as bone-shadow
    shard = [(13, 3), (21, 6), (24, 14), (20, 24), (14, 29),
             (8, 22), (7, 11)]
    d.polygon(shard, fill=(96, 94, 90, 255), outline=(56, 54, 52, 255))
    d.polygon([(13, 3), (21, 6), (18, 14), (12, 12)], fill=(122, 120, 114, 255))
    # glowing fracture lines -- the way back
    frac = [(200, 240, 255, 255), (216, 190, 255, 255)]
    d.line([(13, 6), (16, 13), (14, 20), (16, 26)], fill=frac[0],
           width=1)
    d.line([(16, 13), (21, 16)], fill=frac[1], width=1)
    d.line([(14, 20), (10, 22)], fill=frac[0], width=1)
    # small splinters drifting off the fragment
    for (x, y, c) in [(26, 9, frac[0]), (4, 16, frac[1]),
                      (24, 22, frac[1]), (6, 6, frac[0])]:
        d.point((x, y), fill=c)
        d.point((x + 1, y + 1), fill=c)
    return img


RENDER = {
    "armor_bio_eyes": draw_bio_eyes,
    "integrated_antlers": draw_antlers,
    "integrated_bark": draw_bark,
    "integrated_bark2_a": draw_bark2_a,
    "integrated_bark2_b": draw_bark2_b,
    "integrated_bark2_c": draw_bark2_c,
    "integrated_carapace": draw_carapace,
    "integrated_carapace_molted": draw_carapace_molted,
    "integrated_chitin": draw_chitin,
    "integrated_chitin2": draw_chitin2,
    "integrated_chitin2_molted": draw_chitin2_molted,
    "integrated_chitin3": draw_chitin3,
    "integrated_chitin3_claws": draw_chitin3_claws,
    "integrated_chitin3_molted": draw_chitin3_molted,
    "integrated_claws": draw_claws,
    "integrated_claws_rat": draw_claws_rat,
    "integrated_claws_st": draw_claws_st,
    "integrated_fangs": draw_fangs,
    "integrated_fangs_spider": draw_fangs_spider,
    "integrated_feathers": draw_feathers,
    "integrated_feline_fur": draw_feline_fur,
    "integrated_fishscales": draw_fishscales,
    "integrated_fur": draw_fur,
    "integrated_horns_curled": draw_horns_curled,
    "integrated_horns_pointed": draw_horns_pointed,
    "integrated_horns_small": draw_horns_small,
    "integrated_interstice_aura": draw_interstice_aura,
    "integrated_lightfur": draw_lightfur,
    "integrated_lupine_fur": draw_lupine_fur,
    "integrated_lupine_fur_summer": draw_lupine_summer,
    "integrated_lynx_fur": draw_lynx_fur,
    "integrated_mandibles": draw_mandibles,
    "integrated_patchskin1": draw_patchskin1,
    "integrated_patchskin2": draw_patchskin2,
    "integrated_patchskin3": draw_patchskin3,
    "integrated_plantskin": draw_plantskin,
    "integrated_rabbitfur": draw_rabbitfur,
    "integrated_reptilescales": draw_reptilescales,
    "integrated_saber_teeth": draw_saber_teeth,
    "integrated_scales": draw_scales,
    "integrated_shark_teeth": draw_shark_teeth,
    "integrated_shell": draw_shell,
    "integrated_shell2": draw_shell2,
    "integrated_shell3": draw_shell3,
    "integrated_sparse_scales": draw_sparse_scales,
    "integrated_talons": draw_talons,
    "integrated_toe_talons": draw_toe_talons,
    "integrated_ursine_fur": draw_ursine_fur,
    "integrated_ursine_fur_summer": draw_ursine_summer,
    "integrated_vampire_fangs": draw_vampire_fangs,
    "internal_bio_purifier": draw_bio_purifier,
    "item_portal_storm_dimension_returner": draw_dimension_returner,
}


def main():
    assert set(RENDER) == set(IDS) and len(IDS) == 52
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
                       "preview_item29.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item29.png {sheet.size}")


if __name__ == "__main__":
    main()
