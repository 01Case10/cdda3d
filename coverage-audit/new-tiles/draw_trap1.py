#!/usr/bin/env python3
"""Batch TRAP1 (Phase 2g.4 scope extension): the LIXA portal trap family.

41 ids: tr_LIXA_back_1..16, tr_LIXA_forward_1..16, tr_LIXA_unfold_entry_1..9.

Game-data semantics (effects_on_condition/nether_eocs/LIXA_EOCs_spells_traps.json):
every entry is color "green", symbol "#", name "", always_invisible -- invisible
trigger pads in-game. The number is a channel index into the teleport chain:
  forward_N  -> spell "LIXA Forward Teleport" (teleports the victim forwards),
                sets u_LIXA_tp_forward = N
  back_N     -> spell "LIXA Back Teleport" (teleports the victim backwards),
                sets u_LIXA_tp_back = N
  unfold_entry_N -> sets u_LIXA_entry = N; the unfold EOC teleports to
                LIXA_forward_tp_N and opens the "Infinite Corridor".

So one parameterized renderer: a green portal pad (the shipped tr_portal is a
glowing ring on transparency -- same family treatment), differentiated by
  * glyph: forward = double chevron up, back = double chevron down,
           unfold = nested-squares "unfolding" glyph with entry dot
  * a slight ring hue lean per family (yellow-green / teal-green / pure green)
  * the channel index N as a 2x8 pip grid under the pad (reading order
    left-to-right, top row 1-8, bottom row 9-16), first N pips lit
  * a faint "#" cross-hatch on the pad face, honoring the data symbol

PIL only, per-sprite seeded random.Random (crc32 of id) for speckle/dash
variation. Alpha is strictly {0, 255}: glow is faked with pale opaque tones.
Writes ONLY its own 41 folders + preview_trap1.png. No other files touched.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "traps")
CREDITS = ("Drawn from the trap's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

FAMILIES = [
    ("back", "tr_LIXA_back_", 16),
    ("forward", "tr_LIXA_forward_", 16),
    ("unfold", "tr_LIXA_unfold_entry_", 9),
]

# Palette (all greens per the data color; family hue lean applied to rings).
RIM_DARK = (24, 30, 28, 255)
RIM_EDGE = (58, 71, 66, 255)
FACE = (30, 40, 36, 255)
FACE_SPECK = (38, 50, 44, 255)
HATCH = (25, 34, 30, 255)
GROOVE = (13, 38, 22, 255)
CORE_DARK = (10, 30, 17, 255)
GLYPH = (234, 255, 238, 255)
PIP_OFF = (20, 44, 28, 255)

RING_SETS = {
    # (outer pale glow tone, mid band, inner field tone)
    "forward": ((196, 246, 158), (110, 199, 74), (24, 84, 34)),
    "back": ((158, 240, 205), (56, 184, 143), (16, 74, 55)),
    "unfold": ((168, 248, 178), (66, 196, 96), (18, 82, 38)),
}


def draw_pad(family, n, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx, cy, r = 15.5, 13.5, 12.5

    # Pad disc: rim, face.
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=RIM_DARK)
    d.ellipse([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1], fill=RIM_EDGE)
    d.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2], fill=FACE)

    # Seeded speckle on the face (deterministic per sprite).
    for _ in range(46):
        a = rng.uniform(0, 2 * math.pi)
        rr = rng.uniform(2.0, r - 3.0)
        x, y = cx + rr * math.cos(a), cy + rr * math.sin(a)
        d.point((x, y), fill=FACE_SPECK)

    # Faint "#" cross-hatch clipped to the face (data symbol is "#").
    hatch = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    hd = ImageDraw.Draw(hatch)
    for gx in range(5, 28, 4):
        hd.line([(gx, 2), (gx, 25)], fill=HATCH)
    for gy in range(4, 26, 4):
        hd.line([(3, gy), (28, gy)], fill=HATCH)
    mask = Image.new("L", (32, 32), 0)
    md = ImageDraw.Draw(mask)
    md.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2], fill=255)
    img.paste(hatch, (0, 0), mask)
    d = ImageDraw.Draw(img)

    # Portal rings: pale glow tone, mid band, groove, glowing core field.
    pale, band, field = RING_SETS[family]
    d.ellipse([cx - 9.5, cy - 9.5, cx + 9.5, cy + 9.5], fill=pale + (255,))
    # Seeded dash gap in the pale ring so siblings are not pixel-clones.
    gap_a = rng.uniform(0, 360)
    d.arc([cx - 9.5, cy - 9.5, cx + 9.5, cy + 9.5],
          start=gap_a, end=gap_a + 26, fill=band + (255,), width=2)
    d.ellipse([cx - 8.0, cy - 8.0, cx + 8.0, cy + 8.0], fill=band + (255,))
    d.ellipse([cx - 6.4, cy - 6.4, cx + 6.4, cy + 6.4], fill=GROOVE)
    d.ellipse([cx - 5.2, cy - 5.2, cx + 5.2, cy + 5.2], fill=field + (255,))
    d.ellipse([cx - 3.4, cy - 3.4, cx + 3.4, cy + 3.4], fill=CORE_DARK)

    # Family glyph in the core.
    if family == "forward":
        for dy in (2.6, -1.4):
            d.polygon([(cx - 3.4, cy + dy + 1.6), (cx, cy + dy - 2.0),
                       (cx + 3.4, cy + dy + 1.6), (cx + 3.4, cy + dy + 3.1),
                       (cx, cy + dy - 0.5), (cx - 3.4, cy + dy + 3.1)],
                      fill=GLYPH)
    elif family == "back":
        for dy in (-2.6, 1.4):
            d.polygon([(cx - 3.4, cy + dy - 1.6), (cx, cy + dy + 2.0),
                       (cx + 3.4, cy + dy - 1.6), (cx + 3.4, cy + dy - 3.1),
                       (cx, cy + dy + 0.5), (cx - 3.4, cy + dy - 3.1)],
                      fill=GLYPH)
    else:  # unfold: nested squares opening outward + entry dot
        d.rectangle([cx - 4.4, cy - 4.4, cx + 4.4, cy + 4.4],
                    outline=GLYPH, width=1)
        for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
            d.line([(cx + sx * 4.4, cy + sy * 4.4),
                    (cx + sx * 2.2, cy + sy * 4.4)], fill=GLYPH)
            d.line([(cx + sx * 4.4, cy + sy * 4.4),
                    (cx + sx * 4.4, cy + sy * 2.2)], fill=GLYPH)
        d.rectangle([cx - 2.0, cy - 2.0, cx + 2.0, cy + 2.0],
                    outline=GLYPH, width=1)
        d.ellipse([cx - 0.7, cy - 0.7, cx + 0.7, cy + 0.7], fill=GLYPH)

    # Channel index pips: 2 rows x 8 under the pad, first N lit.
    lit = pale + (255,)
    for i in range(16):
        col, row = i % 8, i // 8
        x0 = 4 + col * 3
        y0 = 27 + row * 3
        d.rectangle([x0, y0, x0 + 1, y0 + 1],
                    fill=lit if i < n else PIP_OFF)

    # Opacity discipline: alpha strictly {0, 255}.
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (family, n, alphas)
    return img


def main():
    ids = []
    for family, prefix, count in FAMILIES:
        for n in range(1, count + 1):
            tid = f"{prefix}{n}"
            ids.append(tid)
            rng = random.Random(zlib.crc32(tid.encode()))
            img = draw_pad(family, n, rng)
            folder = os.path.join(BASE, tid)
            os.makedirs(folder, exist_ok=True)
            img.save(os.path.join(folder, tid + ".png"))
            with open(os.path.join(folder, tid + ".json"), "w") as f:
                json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
                f.write("\n")
            with open(os.path.join(folder, "credits.txt"), "w") as f:
                f.write(CREDITS)
            print("wrote", tid)

    # Contact sheet: rows = families, sprites at 2x NEAREST.
    cell, gap = 64, 3
    cols = 16
    rows = 3
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    pos = {"back": 0, "forward": 1, "unfold": 2}
    for family, prefix, count in FAMILIES:
        for n in range(1, count + 1):
            tid = f"{prefix}{n}"
            img = Image.open(os.path.join(BASE, tid, tid + ".png"))
            img = img.resize((cell, cell), Image.NEAREST)
            x = gap + (n - 1) * (cell + gap)
            y = gap + pos[family] * (cell + gap)
            sheet.alpha_composite(img, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_trap1.png")
    sheet.save(out)
    print("wrote preview_trap1.png", sheet.size, "| total ids:", len(ids))


if __name__ == "__main__":
    main()
