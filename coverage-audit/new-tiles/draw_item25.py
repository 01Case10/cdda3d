#!/usr/bin/env python3
"""Batch ITEM25 (Phase 2g.4 scope extension): mutagens
and effect items, final set -- the seven remaining
mutagen flavors, the purifier, and the purifier
smart shot.

9 ids (authoritative list embedded below; source
list: ~/workspace/item-batches/25.txt). All nine are
defined in data/json/items/comestibles/mutagen.json;
no effects.json pseudo-items fell in this batch.

Copy-from chains resolved against the data before
drawing: the seven mutagen flavors and the purifier
all copy-from the `mutagen_flavor` abstract -- a
250 ml liquid in a glass flask (`flask_glass`), base
color light_green, "A rare substance of uncertain
origins. Causes you to mutate." So the whole family
is drawn as one glass Erlenmeyer flask, the liquid
tinted per line and a paper label on the body bearing
a small ink pictogram of the mutation line:

- mutagen_raptor: yellow-green liquid; three raptor
  claw slashes on the label ("An extremely rare
  mutagen cocktail").
- mutagen_rat: grayish green liquid; a rat (round
  body, ear, long curling tail).
- mutagen_slime: saturated green goo; a dripping
  slime blob.
- mutagen_snail: bright lime liquid (its description:
  "looks awfully like the slime from an old children's
  network"); a snail -- spiral shell, slug body, two
  eye stalks.
- mutagen_spider: deep green liquid; a spider (two
  body spheres, eight bent legs).
- mutagen_troglobite: pale milky mint liquid; an
  eyeless cave fish (the missing eye is the point).
- mutagen_ursine: amber-green liquid; a bear paw
  print (heel pad and four toes).
- purifier: copy-from mutagen_flavor but its own
  color is pink -- "This thick, flesh-hued liquid
  helps revert the mutated to baseline humanity":
  the same flask with thick pink liquid and a plain
  human figure on the label.
- purifier_smart_shot: standalone entry, NOT the
  flask -- "An experimental stem cell treatment...
  The liquid sloshes strangely inside of this
  syringe", color magenta, symbol "!": a diagonal
  syringe, barrel of magenta liquid whose surface
  sits in a strange wavy slosh, with odd bubbles.

Glass is faked with pale opaque colors (this Pillow
build replaces rather than blends); all sprite alpha
is in {0, 255}, asserted per sprite.

Writes ONLY its own 9 folders + preview_item25.png.
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
    "mutagen_raptor",
    "mutagen_rat",
    "mutagen_slime",
    "mutagen_snail",
    "mutagen_spider",
    "mutagen_troglobite",
    "mutagen_ursine",
    "purifier",
    "purifier_smart_shot",
]

INK = (38, 34, 32, 255)
GLASS = (198, 219, 216, 255)      # pale opaque glass
GLASS_D = (148, 172, 170, 255)    # glass shading / outline side
LABEL = (238, 228, 196, 255)      # cream paper label
LABEL_D = (203, 190, 150, 255)


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def flask(d, liquid, liquid_d, rng):
    """Erlenmeyer flask outline; liquid fills the cone."""
    # body silhouette: rim, neck, sloping shoulders, flat base
    body = [(13, 3), (18, 3), (18, 9), (24, 25), (24, 27),
            (7, 27), (7, 25), (13, 9)]
    d.polygon(body, fill=GLASS, outline=INK)
    # liquid: surface at y=15, following the inner cone walls
    # inner half-width grows from ~2.2 at the neck to ~7.6 at base
    def hw(y):
        return 2.2 + (y - 9) * (7.6 - 2.2) / 18.0
    surf = 15
    liq = [(16 - hw(surf), surf), (16 + hw(surf), surf),
           (16 + hw(26), 26), (16 - hw(26), 26)]
    d.polygon(liq, fill=liquid)
    d.line([(16 - hw(surf), surf), (16 + hw(surf), surf)],
           fill=liquid_d, width=1)
    # bubbles rising in the liquid
    for _ in range(7):
        bx = rng.randint(10, 21)
        by = rng.randint(17, 25)
        if abs(bx - 16) < hw(by) - 1:
            d.ellipse([bx, by, bx + 1, by + 1], fill=liquid_d)
    # glass shading: right wall + a highlight streak on the left
    d.line([(18, 9), (24, 25)], fill=GLASS_D, width=1)
    d.line([(12, 11), (9, 22)], fill=(232, 244, 242, 255), width=1)
    # rim
    d.rectangle([12, 2, 19, 4], fill=GLASS_D, outline=INK)
    # paper label on the body
    d.rectangle([11, 17, 20, 25], fill=LABEL, outline=LABEL_D)


# --- label pictograms, drawn in the 11..20 x 17..25 label box ---

def pic_claw(d):
    for i, x in enumerate((12, 15, 18)):
        d.line([(x, 19), (x + 1, 23)], fill=INK, width=1)
        d.point((x - 1, 24), fill=INK)


def pic_rat(d):
    d.ellipse([12, 20, 16, 23], fill=INK)            # body
    d.point((13, 19), fill=INK)                      # ear, clear of the body
    d.point((17, 21), fill=INK)                      # nose
    d.line([(12, 23), (14, 24), (17, 24), (19, 23)], fill=INK)  # tail curl


def pic_slime(d):
    d.pieslice([12, 18, 19, 24], start=180, end=360, fill=INK)
    d.rectangle([12, 21, 19, 23], fill=INK)
    d.point((13, 23), fill=INK)                      # drip
    d.point((18, 23), fill=INK)                      # drip
    d.point((14, 21), fill=LABEL)                    # eye shine
    d.point((17, 21), fill=LABEL)


def pic_snail(d):
    d.rectangle([11, 23, 20, 24], fill=INK)          # slug foot
    d.ellipse([12, 18, 18, 24], outline=INK, width=1)  # shell
    d.ellipse([14, 20, 16, 22], outline=INK, width=1)  # spiral inner
    d.line([(19, 23), (20, 19)], fill=INK)           # eye stalk
    d.point((20, 18), fill=INK)
    d.line([(18, 23), (18, 20)], fill=INK)           # eye stalk


def pic_spider(d):
    d.ellipse([14, 20, 18, 24], fill=INK)            # abdomen
    d.ellipse([15, 18, 17, 20], fill=INK)            # cephalothorax
    for sx, sy, ex, ey in ((14, 19, 11, 17), (14, 21, 11, 21),
                           (14, 22, 11, 25), (18, 19, 20, 17),
                           (18, 21, 20, 21), (18, 22, 20, 25)):
        d.line([(sx, sy), (ex, ey)], fill=INK)


def pic_cavefish(d):
    # eyeless pale fish: body in label-cream cut out of an ink silhouette
    d.polygon([(11, 21), (15, 18), (19, 20), (20, 19), (20, 23),
               (19, 22), (15, 24), (11, 21)], fill=INK)
    d.polygon([(12, 21), (15, 19), (18, 21), (15, 23), (12, 21)],
              fill=LABEL_D)
    # deliberately NO eye dot -- troglobites are eyeless


def pic_paw(d):
    d.ellipse([13, 21, 18, 24], fill=INK)            # heel pad
    for x, y in ((11, 19), (13, 17), (16, 17), (18, 19)):
        d.ellipse([x, y, x + 1, y + 1], fill=INK)    # toes, spread clear


def pic_human(d):
    d.ellipse([15, 17, 16, 18], fill=INK)            # head
    d.line([(15, 19), (15, 22)], fill=INK)           # torso
    d.line([(13, 20), (18, 20)], fill=INK)           # arms
    d.line([(15, 22), (13, 25)], fill=INK)           # leg
    d.line([(15, 22), (17, 25)], fill=INK)           # leg


FLAVORS = {
    # id: (liquid, liquid_dark, pictogram)
    "mutagen_raptor": ((176, 214, 92, 255), (140, 178, 58, 255), pic_claw),
    "mutagen_rat": ((158, 196, 130, 255), (120, 158, 96, 255), pic_rat),
    "mutagen_slime": ((96, 200, 84, 255), (62, 164, 54, 255), pic_slime),
    "mutagen_snail": ((150, 226, 84, 255), (110, 190, 52, 255), pic_snail),
    "mutagen_spider": ((110, 168, 96, 255), (76, 130, 66, 255), pic_spider),
    "mutagen_troglobite": ((196, 226, 196, 255), (158, 196, 164, 255),
                           pic_cavefish),
    "mutagen_ursine": ((178, 200, 104, 255), (142, 164, 70, 255), pic_paw),
    "purifier": ((232, 158, 152, 255), (198, 116, 112, 255), pic_human),
}


def draw_flask_flavor(item_id, rng):
    img, d = canvas()
    liquid, liquid_d, pic = FLAVORS[item_id]
    flask(d, liquid, liquid_d, rng)
    pic(d)
    return img


def draw_smart_shot(rng):
    img, d = canvas()
    magenta = (216, 64, 168, 255)
    magenta_d = (172, 40, 132, 255)
    metal = (168, 172, 178, 255)
    # diagonal axis from needle tip (3,29) to plunger end (29,3)
    # needle
    d.line([(3, 29), (9, 23)], fill=metal, width=1)
    d.point((3, 29), fill=INK)
    # hub
    d.polygon([(8, 24), (11, 21), (13, 23), (10, 26)], fill=metal,
              outline=INK)
    # barrel: a thick diagonal bar from (11,22) to (23,10)
    barrel = [(10, 20), (13, 23), (25, 11), (22, 8)]
    d.polygon(barrel, fill=GLASS, outline=INK)
    # magenta liquid fills most of the barrel; its top surface is a
    # strange wavy slosh (two humps) instead of a level line
    liq = [(11, 20), (13, 22), (24, 11), (22, 10), (20, 12), (19, 10),
           (17, 13), (16, 11), (14, 15), (13, 13)]
    d.polygon(liq, fill=magenta)
    # odd bubbles in the liquid
    for bx, by, s in ((15, 17, 2), (19, 13, 1), (17, 19, 1)):
        d.ellipse([bx, by, bx + s, by + s], fill=magenta_d)
    # measurement ticks along the barrel's upper edge
    for t in range(4):
        x = 15 + t * 2
        y = 17 - t * 2
        d.line([(x, y), (x + 1, y - 1)], fill=INK)
    # finger flange at the top end of the barrel
    d.line([(21, 7), (26, 12)], fill=metal, width=2)
    # plunger rod + thumb rest
    d.line([(24, 9), (28, 5)], fill=metal, width=1)
    d.line([(26, 3), (30, 7)], fill=metal, width=2)
    d.line([(26, 3), (30, 7)], fill=INK, width=1)
    return img


def render(item_id, rng):
    if item_id == "purifier_smart_shot":
        return draw_smart_shot(rng)
    return draw_flask_flavor(item_id, rng)


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
                       "preview_item25.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item25.png {sheet.size}")


if __name__ == "__main__":
    main()
