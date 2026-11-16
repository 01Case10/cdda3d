#!/usr/bin/env python3
"""Batch ITEM24 (Phase 2g.4 scope extension): mutagens
and mutagen-effect items -- the IV primer line, the
oral mutagen line, purifier, chelator, jabberblood,
and the mutated arm/leg.

52 ids (authoritative list embedded below; source
list: ~/workspace/item-batches/24.txt). All 52 ids
resolve to ITEM entries in
data/json/items/comestibles/mutagen.json (the
mutagen_* ids also exist as effect_type entries in
effects.json, but the item entries are the ones this
tileset category covers). Copy-from chains resolved
against the data before drawing:

- iv_* entries copy-from iv_mutagen_flavor (itself
  from mutagen_flavor): container test_tube, tool
  syringe, symbol '!', base color green. Drawn as a
  syringe-injector whose barrel carries the line's
  liquid color (from each entry's color field /
  description) and a small motif glyph for the line.
- mutagen_* entries copy-from mutagen_flavor:
  container flask_glass, symbol '~', 250 ml. Drawn
  as a glass flask with the line's liquid and motif.
  Line colors follow the matching IV primer's
  color story where the oral entry gives none.
- iv_mutagen ("mutagenic catalyst"): the base
  primer, green, helix glyph.
- iv_purifier ("purifier primer", pink): "chunky,
  flesh-hued injection" -- a thicker injector with
  a return-to-human circular-arrow glyph.
- mutagen_chelator (white, standalone entry): a
  flask of milky liquid with a chelate ring glyph
  (a claw ring binding a dot).
- mutagen_interstice ("perfectly clear water"): a
  flask whose liquid is nearly invisible -- faint
  meniscus and motion ripples only.
- mutagen_jabberblood (copy-from the base mutagen
  slurry, red): thick soupy red liquid, bubbling,
  one bubble watching back with an eye ("seems to
  bubble with an intelligence of its own").
- arm / leg (ITEM, hflesh, brown, symbol '%'):
  "mutated arm" / "mutated leg" -- misshapen human
  limbs with an extra digit and a growth lump.

Writes ONLY its own 52 folders + preview_item24.png.
PIL only; per-sprite random.Random(crc32(id)).
This Pillow build's ImageDraw replaces rather than
blends, so all colors are fully opaque and shading
is done with flat tones.
"""
import json
import math
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
    "arm",
    "iv_mutagen",
    "iv_mutagen_alpha",
    "iv_mutagen_batrachian",
    "iv_mutagen_beast",
    "iv_mutagen_bird",
    "iv_mutagen_cattle",
    "iv_mutagen_cephalopod",
    "iv_mutagen_chimera",
    "iv_mutagen_chiropteran",
    "iv_mutagen_crustacean",
    "iv_mutagen_elfa",
    "iv_mutagen_feline",
    "iv_mutagen_fish",
    "iv_mutagen_insect",
    "iv_mutagen_lizard",
    "iv_mutagen_lupine",
    "iv_mutagen_medical",
    "iv_mutagen_mouse",
    "iv_mutagen_plant",
    "iv_mutagen_rabbit",
    "iv_mutagen_raptor",
    "iv_mutagen_rat",
    "iv_mutagen_slime",
    "iv_mutagen_snail",
    "iv_mutagen_spider",
    "iv_mutagen_troglobite",
    "iv_mutagen_ursine",
    "iv_purifier",
    "leg",
    "mutagen_alpha",
    "mutagen_batrachian",
    "mutagen_beast",
    "mutagen_bird",
    "mutagen_cattle",
    "mutagen_cephalopod",
    "mutagen_chelator",
    "mutagen_chimera",
    "mutagen_chiropteran",
    "mutagen_crustacean",
    "mutagen_elfa",
    "mutagen_feline",
    "mutagen_fish",
    "mutagen_insect",
    "mutagen_interstice",
    "mutagen_jabberblood",
    "mutagen_lizard",
    "mutagen_lupine",
    "mutagen_medical",
    "mutagen_mouse",
    "mutagen_plant",
    "mutagen_rabbit",
]

INK = (38, 34, 32, 255)
GLASS = (198, 222, 228, 255)
GLASS_D = (110, 138, 148, 255)
WHITE = (246, 246, 246, 255)

# line -> (liquid color, motif kind)
LINES = {
    "alpha":       ((196, 36, 40, 255), "helix"),
    "batrachian":  ((148, 158, 148, 255), "frog"),
    "beast":       ((200, 40, 36, 255), "claw"),
    "bird":        ((64, 196, 214, 255), "feather"),
    "cattle":      ((128, 216, 104, 255), "horns"),
    "cephalopod":  ((52, 52, 60, 255), "tentacle"),
    "chimera":     ((196, 44, 44, 255), "triskel"),
    "chiropteran": ((44, 44, 52, 255), "bat"),
    "crustacean":  ((206, 52, 40, 255), "pincer"),
    "elfa":        ((96, 196, 92, 255), "leaf"),
    "feline":      ((242, 202, 58, 255), "cateye"),
    "fish":        ((104, 178, 232, 255), "fish"),
    "insect":      ((226, 164, 52, 255), "hex"),
    "lizard":      ((110, 204, 96, 255), "scales"),
    "lupine":      ((236, 236, 238, 255), "moon"),
    "medical":     ((230, 108, 100, 255), "cross"),
    "mouse":       ((170, 172, 176, 255), "mouse"),
    "plant":       ((74, 178, 66, 255), "sprout"),
    "rabbit":      ((238, 140, 50, 255), "ears"),
    "raptor":      ((84, 186, 80, 255), "talon"),
    "rat":         ((150, 118, 84, 255), "tail"),
    "slime":       ((58, 58, 64, 255), "blob"),
    "snail":       ((172, 176, 170, 255), "spiral"),
    "spider":      ((176, 178, 176, 255), "spider"),
    "troglobite":  ((50, 50, 58, 255), "noeye"),
    "ursine":      ((238, 178, 64, 255), "paw"),
}


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def motif_ink(liquid):
    lum = 0.3 * liquid[0] + 0.59 * liquid[1] + 0.11 * liquid[2]
    return (34, 32, 36, 255) if lum > 130 else WHITE


def motif(d, kind, cx, cy, col, bg):
    """Tiny glyphs, drawn within roughly +/-4 px of (cx, cy)."""
    if kind == "helix":
        for t in range(-4, 5):
            x = int(round(2.2 * math.sin(t * 0.95)))
            d.point((cx + x, cy + t), fill=col)
            d.point((cx - x, cy + t), fill=col)
        for t in (-3, 0, 3):
            d.line([(cx - 2, cy + t), (cx + 2, cy + t)], fill=col)
    elif kind == "frog":
        d.ellipse([cx - 4, cy - 4, cx - 1, cy - 1], fill=col)
        d.ellipse([cx + 1, cy - 4, cx + 4, cy - 1], fill=col)
        d.arc([cx - 4, cy - 2, cx + 4, cy + 4], start=20, end=160, fill=col)
    elif kind == "claw":
        for i in range(3):
            x = cx - 3 + i * 3
            d.line([(x, cy - 4), (x - 1, cy + 3)], fill=col)
    elif kind == "feather":
        d.line([(cx - 3, cy + 4), (cx + 3, cy - 4)], fill=col)
        for i in range(3):
            t = -2 + i * 2
            d.line([(cx + t // 2, cy - t // 2), (cx + t // 2 + 3, cy - t // 2 - 1)], fill=col)
            d.line([(cx + t // 2, cy - t // 2), (cx + t // 2 - 1, cy - t // 2 + 3)], fill=col)
    elif kind == "horns":
        d.arc([cx - 4, cy - 3, cx + 4, cy + 3], start=180, end=360, fill=col)
        d.polygon([(cx - 4, cy - 2), (cx - 5, cy - 5), (cx - 2, cy - 3)], fill=col)
        d.polygon([(cx + 4, cy - 2), (cx + 5, cy - 5), (cx + 2, cy - 3)], fill=col)
    elif kind == "tentacle":
        pts = [(cx + int(round(2 * math.sin(t * 1.1))), cy + t) for t in range(-4, 5)]
        d.line(pts, fill=col)
        for t in (-2, 1, 3):
            d.point((cx + int(round(2 * math.sin(t * 1.1))) + 1, cy + t), fill=col)
    elif kind == "triskel":
        for a in (90, 210, 330):
            x = cx + int(round(2.6 * math.cos(math.radians(a))))
            y = cy + int(round(2.6 * math.sin(math.radians(a))))
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=col)
        d.point((cx, cy), fill=col)
    elif kind == "bat":
        d.polygon([(cx, cy - 2), (cx - 4, cy - 4), (cx - 3, cy), (cx - 1, cy - 1),
                   (cx, cy + 1), (cx + 1, cy - 1), (cx + 3, cy), (cx + 4, cy - 4)],
                  fill=col)
    elif kind == "pincer":
        d.arc([cx - 5, cy - 4, cx, cy + 3], start=270, end=120, fill=col)
        d.arc([cx, cy - 4, cx + 5, cy + 3], start=60, end=270, fill=col)
        d.point((cx, cy + 3), fill=col)
    elif kind == "leaf":
        d.polygon([(cx, cy + 4), (cx - 3, cy), (cx, cy - 4), (cx + 3, cy)], fill=col)
        d.line([(cx, cy + 4), (cx, cy - 3)], fill=bg)
    elif kind == "cateye":
        d.polygon([(cx - 4, cy), (cx, cy - 3), (cx + 4, cy), (cx, cy + 3)], outline=col)
        d.line([(cx, cy - 2), (cx, cy + 2)], fill=col)
    elif kind == "fish":
        d.polygon([(cx - 4, cy), (cx + 1, cy - 3), (cx + 1, cy + 3)], fill=col)
        d.polygon([(cx + 1, cy), (cx + 4, cy - 3), (cx + 4, cy + 3)], fill=col)
        d.point((cx - 2, cy - 1), fill=bg)
    elif kind == "hex":
        pts = [(cx + int(round(3.4 * math.cos(math.radians(a)))),
                cy + int(round(3.4 * math.sin(math.radians(a)))))
               for a in range(0, 360, 60)]
        d.polygon(pts, outline=col)
        d.line([(cx - 1, cy - 3), (cx - 2, cy - 5)], fill=col)
        d.line([(cx + 1, cy - 3), (cx + 2, cy - 5)], fill=col)
    elif kind == "scales":
        for row, dy in enumerate((-3, 0, 3)):
            off = 2 if row % 2 else 0
            for dx in (-3 + off, 0 + off):
                d.arc([cx + dx - 2, cy + dy - 2, cx + dx + 2, cy + dy + 2],
                      start=0, end=180, fill=col)
    elif kind == "moon":
        d.ellipse([cx - 3, cy - 4, cx + 3, cy + 4], fill=col)
        d.ellipse([cx - 1, cy - 4, cx + 5, cy + 4], fill=bg)
    elif kind == "cross":
        d.rectangle([cx - 1, cy - 4, cx + 1, cy + 4], fill=col)
        d.rectangle([cx - 4, cy - 1, cx + 4, cy + 1], fill=col)
    elif kind == "mouse":
        d.ellipse([cx - 4, cy - 4, cx - 1, cy - 1], outline=col)
        d.ellipse([cx + 1, cy - 4, cx + 4, cy - 1], outline=col)
        d.ellipse([cx - 3, cy - 1, cx + 3, cy + 4], outline=col)
    elif kind == "sprout":
        d.line([(cx, cy + 4), (cx, cy - 1)], fill=col)
        d.polygon([(cx, cy - 1), (cx - 4, cy - 3), (cx - 1, cy + 1)], fill=col)
        d.polygon([(cx, cy - 1), (cx + 4, cy - 4), (cx + 1, cy + 1)], fill=col)
    elif kind == "ears":
        d.ellipse([cx - 4, cy - 5, cx - 1, cy + 1], outline=col)
        d.ellipse([cx + 1, cy - 5, cx + 4, cy + 1], outline=col)
        d.line([(cx - 2, cy + 2), (cx + 2, cy + 2)], fill=col)
    elif kind == "talon":
        for i in range(3):
            x = cx - 3 + i * 3
            d.arc([x - 2, cy - 4, x + 2, cy + 2], start=300, end=160, fill=col)
    elif kind == "tail":
        pts = [(cx - 4 + t, cy + int(round(2.4 * math.sin(t * 0.9))))
               for t in range(9)]
        d.line(pts, fill=col)
    elif kind == "blob":
        d.ellipse([cx - 4, cy - 3, cx + 4, cy + 2], fill=col)
        d.rectangle([cx - 2, cy + 2, cx - 1, cy + 4], fill=col)
        d.rectangle([cx + 2, cy + 2, cx + 3, cy + 3], fill=col)
    elif kind == "spiral":
        for t in range(26):
            r = 4.2 * t / 26.0
            a = t * 0.62
            d.point((cx + int(round(r * math.cos(a))),
                     cy + int(round(r * math.sin(a)))), fill=col)
    elif kind == "spider":
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 2], fill=col)
        for sx in (-1, 1):
            for i, dy in enumerate((-3, -1, 1, 3)):
                d.line([(cx + sx, cy), (cx + sx * 4, cy + dy)], fill=col)
    elif kind == "noeye":
        d.polygon([(cx - 4, cy), (cx, cy - 3), (cx + 4, cy), (cx, cy + 3)], outline=col)
        d.line([(cx - 3, cy + 3), (cx + 3, cy - 3)], fill=col)
    elif kind == "paw":
        d.ellipse([cx - 2, cy, cx + 2, cy + 4], fill=col)
        for dx in (-3, 0, 3):
            d.point((cx + dx, cy - 2), fill=col)
            d.point((cx + dx, cy - 3), fill=col)
    elif kind == "chelate":
        d.arc([cx - 4, cy - 4, cx + 4, cy + 4], start=50, end=310, fill=col)
        d.point((cx + 4, cy - 2), fill=col)
        d.point((cx + 4, cy + 2), fill=col)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=col)
    elif kind == "return":
        d.arc([cx - 4, cy - 4, cx + 4, cy + 4], start=300, end=200, fill=col)
        d.polygon([(cx + 3, cy - 4), (cx + 5, cy - 1), (cx + 1, cy - 2)], fill=col)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=col)
    elif kind == "ripple":
        for dy in (-2, 1):
            pts = [(cx - 4 + t, cy + dy + (1 if t % 4 == 2 else 0)) for t in range(9)]
            d.line(pts, fill=col)
    elif kind == "eye":
        d.ellipse([cx - 4, cy - 2, cx + 4, cy + 2], fill=WHITE)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=INK)
    else:
        raise ValueError(kind)


def flecks(d, rng, kind, box):
    """In-liquid details for special liquids."""
    x0, y0, x1, y1 = box
    if kind == "iridescent":
        for col in ((240, 120, 220, 255), (90, 220, 230, 255), (250, 230, 110, 255)):
            for _ in range(3):
                x = rng.randrange(x0, x1)
                y = rng.randrange(y0, y1)
                d.point((x, y), fill=col)
    elif kind == "foam":
        for x in range(x0, x1):
            if rng.random() < 0.75:
                d.point((x, y0), fill=WHITE)
                if rng.random() < 0.4:
                    d.point((x, y0 + 1), fill=WHITE)
    elif kind == "filaments":
        for _ in range(4):
            x = rng.randrange(x0 + 1, x1 - 1)
            pts = [(x + (1 if t % 2 else -1), y0 + t) for t in range(y1 - y0)]
            d.line(pts, fill=(232, 234, 230, 255))
    elif kind == "tadpoles":
        for _ in range(3):
            x = rng.randrange(x0 + 1, x1 - 2)
            y = rng.randrange(y0 + 1, y1 - 1)
            d.point((x, y), fill=(30, 34, 30, 255))
            d.line([(x, y), (x + 2, y + 1)], fill=(30, 34, 30, 255))
    elif kind == "bubbles":
        for _ in range(5):
            x = rng.randrange(x0 + 1, x1 - 1)
            y = rng.randrange(y0, y1)
            d.ellipse([x, y, x + 1, y + 1], outline=(246, 170, 160, 255))
    elif kind == "puree":
        for _ in range(8):
            x = rng.randrange(x0, x1)
            y = rng.randrange(y0, y1)
            d.point((x, y), fill=(120, 190, 100, 255))
    elif kind == "metallic":
        for _ in range(4):
            x = rng.randrange(x0, x1 - 3)
            y = rng.randrange(y0, y1)
            d.line([(x, y), (x + 3, y)], fill=(228, 230, 234, 255))
    elif kind == "sheen":
        for col in ((120, 200, 220, 255), (200, 140, 220, 255)):
            for _ in range(2):
                x = rng.randrange(x0, x1)
                y = rng.randrange(y0, y1)
                d.point((x, y), fill=col)


# liquid-detail key per item id (beyond the base color)
IV_DETAIL = {"iv_mutagen_chimera": "iridescent", "iv_mutagen_fish": "foam",
             "iv_mutagen_spider": "filaments", "iv_mutagen_mouse": "metallic",
             "iv_mutagen_slime": "sheen", "iv_mutagen_feline": "metallic"}
ORAL_DETAIL = {"mutagen_chimera": "iridescent", "mutagen_fish": "foam",
               "mutagen_batrachian": "tadpoles", "mutagen_jabberblood": "bubbles",
               "mutagen_plant": "puree", "mutagen_lizard": "puree"}


def syringe(rng, item_id, liquid, kind, chunky=False):
    """Syringe-injector drawn horizontally, then rotated."""
    layer = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    ink = motif_ink(liquid)
    if chunky:
        by0, by1 = 10, 22
        bx0, bx1 = 10, 24
    else:
        by0, by1 = 12, 20
        bx0, bx1 = 10, 25
    # needle + hub
    d.line([(1, 16), (7, 16)], fill=(170, 174, 178, 255))
    d.polygon([(7, 14), (10, by0), (10, by1), (7, 18)], fill=GLASS_D)
    # barrel: glass shell, liquid fill, air gap at the plunger end
    d.rectangle([bx0, by0, bx1, by1], fill=GLASS, outline=INK)
    fill_end = bx0 + int((bx1 - bx0) * 0.68)
    d.rectangle([bx0 + 1, by0 + 1, fill_end, by1 - 1], fill=liquid)
    detail = IV_DETAIL.get(item_id)
    if detail:
        flecks(d, rng, detail, (bx0 + 2, by0 + 2, fill_end - 1, by1 - 2))
    # measurement ticks on the glass
    for x in range(bx0 + 3, bx1 - 1, 4):
        d.line([(x, by0 + 1), (x, by0 + 2)], fill=GLASS_D)
    # motif on the barrel
    motif(d, kind, (bx0 + fill_end) // 2, 16, ink, liquid)
    # flange + plunger
    d.rectangle([bx1, by0 - 2, bx1 + 1, by1 + 2], fill=GLASS_D, outline=INK)
    d.rectangle([bx1 + 2, 15, 29, 17], fill=(120, 124, 128, 255))
    d.rounded_rectangle([29, by0 - 1, 31, by1 + 1], radius=2,
                        fill=(120, 124, 128, 255), outline=INK)
    img = layer.rotate(-32, resample=Image.NEAREST, center=(16, 16))
    out, _ = canvas()
    out.alpha_composite(img)
    return out


def flask(rng, item_id, liquid, kind, detail=None):
    img, d = canvas()
    ink = motif_ink(liquid)
    # neck + lip + cork
    d.rectangle([13, 5, 19, 12], fill=GLASS, outline=INK)
    d.rectangle([12, 3, 20, 5], fill=GLASS, outline=INK)
    d.rectangle([14, 1, 18, 3], fill=(178, 142, 96, 255), outline=INK)
    # round body
    d.ellipse([6, 11, 26, 30], fill=GLASS, outline=INK)
    # liquid: lower segment of the body (surface chord at y=17)
    surf = 17
    pts = []
    for x in range(8, 25):
        dx = (x - 16) / 9.5
        if abs(dx) < 1:
            yb = 20.5 + 9.5 * math.sqrt(1 - dx * dx)
            pts.append((x, int(yb)))
    seg = [(9, surf), (23, surf)] + pts[::-1]
    d.polygon(seg, fill=liquid)
    d.line([(9, surf), (23, surf)], fill=GLASS_D if item_id == "mutagen_interstice"
           else liquid)
    if detail:
        flecks(d, rng, detail, (10, surf + 2, 22, 27))
    # glass shine
    d.line([(9, 16), (9, 24)], fill=WHITE)
    d.line([(10, 15), (10, 17)], fill=WHITE)
    if item_id == "mutagen_interstice":
        # nearly invisible: only a meniscus glint and ripples in motion
        motif(d, "ripple", 16, 22, (140, 180, 190, 255), liquid)
    elif item_id == "mutagen_jabberblood":
        motif(d, "eye", 16, 21, ink, liquid)
        d.ellipse([11, 18, 13, 20], outline=(246, 170, 160, 255))
        d.ellipse([20, 24, 21, 25], outline=(246, 170, 160, 255))
    else:
        motif(d, kind, 16, 22, ink, liquid)
    return img


SKIN = (172, 128, 92, 255)
SKIN_D = (138, 98, 64, 255)
SKIN_L = (198, 158, 118, 255)


def arm(rng):
    img, d = canvas()
    # upper arm: shoulder stub (top-left) to elbow; forearm down to hand
    d.line([(9, 4), (17, 12)], fill=SKIN, width=8)
    d.line([(17, 12), (14, 22)], fill=SKIN, width=7)
    d.line([(11, 6), (18, 13)], fill=SKIN_D, width=2)
    # growth lump on the forearm
    d.ellipse([15, 15, 21, 20], fill=SKIN, outline=SKIN_D)
    d.arc([16, 16, 20, 19], start=300, end=120, fill=SKIN_D)
    # torn skin at the shoulder stub
    d.polygon([(5, 1), (12, 2), (9, 6), (6, 5)], fill=SKIN_L)
    d.point((7, 3), fill=(150, 60, 52, 255))
    # hand: palm + six fingers (one extra, one bent wrong)
    d.ellipse([10, 21, 17, 27], fill=SKIN, outline=SKIN_D)
    fingers = [(10, 26, 8, 30), (12, 27, 11, 31), (14, 27, 14, 31),
               (16, 26, 17, 30), (17, 24, 20, 26), (11, 25, 7, 27)]
    for x0, y0, x1, y1 in fingers:
        d.line([(x0, y0), (x1, y1)], fill=SKIN, width=2)
    for _ in range(4):  # wart flecks
        x = rng.randrange(9, 19)
        y = rng.randrange(5, 20)
        d.point((x, y), fill=SKIN_D)
    return img


def leg(rng):
    img, d = canvas()
    # thigh to knee, shin to ankle (slightly twisted), foot with six toes
    d.line([(12, 3), (16, 12)], fill=SKIN, width=9)
    d.line([(16, 12), (14, 23)], fill=SKIN, width=6)
    d.line([(10, 5), (14, 12)], fill=SKIN_D, width=2)
    # bulging growth on the thigh
    d.ellipse([7, 6, 13, 12], fill=SKIN, outline=SKIN_D)
    d.point((10, 9), fill=SKIN_D)
    # bruise patch on the shin
    d.ellipse([13, 17, 17, 20], fill=(140, 104, 96, 255))
    # severed top: torn flesh + bone nub
    d.polygon([(8, 1), (16, 1), (15, 4), (9, 4)], fill=SKIN_L)
    d.ellipse([10, 1, 13, 3], fill=(232, 226, 210, 255))
    # foot pointing right, six toe bumps
    d.polygon([(13, 23), (20, 23), (24, 26), (24, 28), (13, 28)], fill=SKIN,
              outline=SKIN_D)
    for i in range(6):
        x = 19 + i
        d.point((x, 29), fill=SKIN)
    d.line([(14, 25), (22, 25)], fill=SKIN_D)
    for _ in range(4):
        x = rng.randrange(9, 18)
        y = rng.randrange(5, 22)
        d.point((x, y), fill=SKIN_D)
    return img


def render(item_id, rng):
    if item_id == "arm":
        return arm(rng)
    if item_id == "leg":
        return leg(rng)
    if item_id == "iv_purifier":
        return syringe(rng, item_id, (240, 150, 180, 255), "return",
                       chunky=True)
    if item_id.startswith("iv_"):
        if item_id == "iv_mutagen":
            return syringe(rng, item_id, (84, 176, 92, 255), "helix")
        line = item_id[len("iv_mutagen_"):]
        liquid, kind = LINES[line]
        return syringe(rng, item_id, liquid, kind)
    # oral line
    if item_id == "mutagen_chelator":
        return flask(rng, item_id, (240, 240, 236, 255), "chelate")
    if item_id == "mutagen_interstice":
        return flask(rng, item_id, (226, 240, 242, 255), "ripple")
    if item_id == "mutagen_jabberblood":
        return flask(rng, item_id, (188, 32, 32, 255), "eye",
                     detail="bubbles")
    line = item_id[len("mutagen_"):]
    liquid, kind = LINES[line]
    if line == "rabbit":
        liquid = (238, 240, 238, 255)  # "white, clear colored"
    if line == "plant":
        liquid = (70, 140, 58, 255)  # pureed spinach
    if line == "batrachian":
        liquid = (96, 110, 96, 255)  # shadowy pond water
    if line == "chiropteran":
        liquid = (56, 38, 42, 255)  # murky black, smelling of blood
    return flask(rng, item_id, liquid, kind, detail=ORAL_DETAIL.get(item_id))


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
    cols, cell = 8, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item24.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item24.png {sheet.size}")


if __name__ == "__main__":
    main()
