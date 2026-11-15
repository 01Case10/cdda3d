#!/usr/bin/env python3
"""Micro-batch (Phase 2g.4 scope extension): the null overmap terrain.

The game data defines an overmap_terrain with id "" — name "nothing",
sym '%', color white (overmap_terrain.json). It is the null terrain
that fills unset map space, and the coverage audit counts it missing
in shipped UltiCa. The decomposed tileset format can represent it:
a folder whose entry JSON declares "id": "" (verified against the
voxelizer — the material resolves exact with id "").

Treatment: a full-tile opaque void cell — near-black with a faint
white '%' glyph (the data symbol/color) and sparse dim specks.
PIL only, seeded random.Random(crc32 of the sprite name).
Writes ONLY its own folder + preview_nothing.png.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "overmap", "nothing")
CREDITS = ("Drawn from the overmap terrain's in-game description for "
           "the CDDA Voxel coverage audit (2026-11-15). License: "
           "CC BY-SA 3.0, as the tileset.\n")

VOID = (8, 8, 10, 255)
SPECK = (38, 38, 44, 255)
GLYPH = (210, 210, 215, 255)
GLYPH_DIM = (120, 120, 128, 255)


def draw_nothing(rng):
    img = Image.new("RGBA", (32, 32), VOID)
    d = ImageDraw.Draw(img)
    for _ in range(26):
        x = rng.randrange(32)
        y = rng.randrange(32)
        d.point((x, y), fill=SPECK)
    # '%' glyph, pixel-drawn: two rings and a diagonal slash.
    def ring(cx, cy):
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], outline=GLYPH)
        d.point((cx, cy), fill=VOID)
    ring(11, 11)
    ring(21, 21)
    for i in range(9):
        d.point((21 - i, 10 + i), fill=GLYPH)
        d.point((22 - i, 10 + i), fill=GLYPH_DIM)
    alphas = set(img.getchannel("A").getdata())
    assert alphas == {255}, alphas
    return img


def main():
    rng = random.Random(zlib.crc32(b"nothing"))
    img = draw_nothing(rng)
    os.makedirs(BASE, exist_ok=True)
    img.save(os.path.join(BASE, "nothing.png"))
    with open(os.path.join(BASE, "nothing.json"), "w") as f:
        json.dump([{"id": "", "fg": "nothing", "bg": ""}], f, indent=2)
        f.write("\n")
    with open(os.path.join(BASE, "credits.txt"), "w") as f:
        f.write(CREDITS)
    sheet = Image.new("RGBA", (136, 136), (30, 32, 36, 255))
    sheet.alpha_composite(img.resize((128, 128), Image.NEAREST), (4, 4))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_nothing.png")
    sheet.save(out)
    print("wrote the null-terrain tile + preview_nothing.png")


if __name__ == "__main__":
    main()
