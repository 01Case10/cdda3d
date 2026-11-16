#!/usr/bin/env python3
"""ITEM46: firearm retool kits + a grenade sling -- CDDA Voxel
coverage audit.

Draws each retool (caliber-conversion) kit as a barrel + bolt/upper
part set laid out on a dark foam tool-roll backing, with a caliber
colour band on the barrel.  Families differ by silhouette, barrel
lengths by suffix, calibers by band colour:

  band colours:  .22 gray (150,150,154)   .223 green (74,160,74)
    300BLK black/red (band black, red centre stripe)
    .450 orange (232,126,40)   .50 Beowulf gold (216,178,60)
    7.62 tan (196,164,110)     .300WM blue (64,110,200)
    .308 olive (120,130,62)    .338 purple (140,80,180)
    .357 silver (200,204,210)  .44 red (206,52,44)
    .50 gold (216,178,60)      9mm white (240,240,236)
    .40 yellow (236,200,60)    .45 brown (140,100,64)

  lengths: pistol < short < medium < standard < extended.

staff_sling_grenade_cradle is unrelated to the kits: a woven sling
with a pouch cradling round grenades.  Self-contained: the id list
is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "retool_ar15_22",
    "retool_ar15_223rem",
    "retool_ar15_223rem_extended",
    "retool_ar15_223rem_medium",
    "retool_ar15_223rem_pistol",
    "retool_ar15_223rem_short",
    "retool_ar15_300blk",
    "retool_ar15_300blk_medium",
    "retool_ar15_300blk_short",
    "retool_ar15_450",
    "retool_ar15_450_medium",
    "retool_ar15_450_short",
    "retool_ar15_50beowulf",
    "retool_ar15_50beowulf_medium",
    "retool_ar15_50beowulf_short",
    "retool_ar15_762",
    "retool_axmc_300win",
    "retool_axmc_308win",
    "retool_axmc_338lapua",
    "retool_deagle_357",
    "retool_deagle_44",
    "retool_deagle_50",
    "retool_mdrx_223rem",
    "retool_mdrx_223rem_medium",
    "retool_mdrx_223rem_short",
    "retool_mdrx_300blk",
    "retool_mdrx_308win",
    "retool_mdrx_308win_medium",
    "retool_ump_40",
    "retool_ump_45",
    "retool_ump_9",
    "staff_sling_grenade_cradle",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

STEEL = (96, 100, 108, 255)
STEEL_DARK = (58, 60, 66, 255)
STEEL_LIGHT = (150, 154, 162, 255)
BRASS = (202, 168, 82, 255)
FOAM = (46, 46, 54, 255)

BANDS = {
    "22": (150, 150, 154, 255),
    "223rem": (74, 160, 74, 255),
    "300blk": (24, 24, 28, 255),  # black band, red centre stripe (below)
    "450": (232, 126, 40, 255),
    "50beowulf": (216, 178, 60, 255),
    "762": (196, 164, 110, 255),
    "300win": (64, 110, 200, 255),
    "308win": (120, 130, 62, 255),
    "338lapua": (140, 80, 180, 255),
    "357": (200, 204, 210, 255),
    "44": (206, 52, 44, 255),
    "50": (216, 178, 60, 255),
    "9": (240, 240, 236, 255),
    "40": (236, 200, 60, 255),
    "45": (140, 100, 64, 255),
}
STRIPE_300BLK = (196, 40, 36, 255)

# barrel lengths in px, by family then length suffix
LENGTHS = {
    "ar15": {"pistol": 11, "short": 13, "medium": 16, "standard": 19,
             "extended": 22},
    "axmc": {"standard": 21},
    "deagle": {"standard": 10},
    "mdrx": {"short": 12, "medium": 14, "standard": 15},
    "ump": {"standard": 11},
}
THICK = {"ar15": 3, "axmc": 4, "deagle": 4, "mdrx": 3, "ump": 3}


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def parse(sid):
    parts = sid[len("retool_"):].split("_")
    return parts[0], parts[1], (parts[2] if len(parts) > 2 else "standard")


# ------------------------------------------------------------------ pieces

def foam(d, rng, band_col):
    """Dark foam tool-roll backing with stitching and a caliber tag."""
    d.rectangle([3, 7, 30, 26], fill=shade(FOAM, 0.55))
    d.rectangle([2, 6, 29, 25], fill=FOAM)
    d.rectangle([3, 7, 28, 24], fill=shade(FOAM, 1.18))
    for x in range(5, 28, 4):  # stitch dashes, top and bottom
        d.line([x, 7, x + 2, 7], fill=shade(FOAM, 1.7), width=1)
        d.line([x, 24, x + 2, 24], fill=shade(FOAM, 1.7), width=1)
    for _ in range(3):  # worn scratches in the foam
        x, y = rng.uniform(5, 24), rng.uniform(19, 23)
        d.line([x, y, x + rng.uniform(2, 4), y], fill=shade(FOAM, 1.45),
               width=1)
    d.rectangle([25, 7, 28, 10], fill=band_col)  # caliber tag, top corner
    d.point((26, 8), fill=shade(band_col, 0.6))


def barrel(d, x0, x1, y, thick):
    """A horizontal barrel: dark underside, steel body, bright bore line."""
    d.rectangle([x0, y, x1, y + thick - 1], fill=STEEL)
    d.line([x0, y + thick - 1, x1, y + thick - 1], fill=STEEL_DARK, width=1)
    d.line([x0, y, x1, y], fill=STEEL_LIGHT, width=1)


def band(d, x, y, thick, caliber):
    """The caliber colour band wrapped around the barrel at x."""
    col = BANDS[caliber]
    d.rectangle([x, y - 1, x + 2, y + thick], fill=col)
    if caliber == "300blk":  # red centre stripe on the black band
        d.line([x + 1, y - 1, x + 1, y + thick], fill=STRIPE_300BLK, width=1)


def cartridges(d, rng, caliber):
    """A couple of loose rounds standing at the foam's lower right."""
    n = rng.choice([2, 2, 3])
    for i in range(n):
        x = 21 + i * 3
        d.rectangle([x, 20, x + 1, 24], fill=BRASS)
        d.polygon([(x, 20), (x + 1, 20), (x, 18)], fill=BANDS[caliber])


def small_part(d, rng, kind):
    """One loose small part on the lower foam row (bolt, spring, rod)."""
    if kind == 0:  # bolt carrier: block with ring grooves
        d.rectangle([5, 20, 13, 23], fill=STEEL)
        for x in (7, 10):
            d.line([x, 20, x, 23], fill=STEEL_DARK, width=1)
        d.line([5, 20, 13, 20], fill=STEEL_LIGHT, width=1)
    elif kind == 1:  # recoil spring: zigzag
        pts = []
        for i in range(6):
            pts.append((5 + i * 2, 21 if i % 2 == 0 else 23))
        d.line(pts, fill=STEEL_LIGHT, width=1)
    else:  # cleaning rod: one long thin line with a loop end
        d.line([5, 22, 15, 22], fill=STEEL_LIGHT, width=1)
        d.ellipse([15, 21, 18, 24], outline=STEEL_LIGHT, width=1)


# -------------------------------------------------------------------- kits

def draw_kit(d, rng, family, caliber, length):
    foam(d, rng, BANDS[caliber])
    thick = THICK[family]
    blen = LENGTHS[family][length]
    part = rng.randrange(3)
    if family == "ar15":
        # slim upper receiver at the left, barrel running right
        d.rectangle([3, 10, 7, 16], fill=STEEL)
        d.rectangle([4, 12, 6, 14], fill=STEEL_DARK)  # ejection port
        d.line([3, 9, 6, 9], fill=STEEL_LIGHT, width=1)  # charging handle
        x0 = 7
        barrel(d, x0, x0 + blen, 11, thick)
        band(d, x0 + 2, 11, thick, caliber)
        if length in ("standard", "medium", "extended"):
            hx = x0 + blen - (10 if length == "extended" else 6)
            d.rectangle([hx, 10, x0 + blen - 2, 14], fill=STEEL_DARK)
            for vx in range(hx + 2, x0 + blen - 3, 3):  # handguard vents
                d.point((vx, 12), fill=STEEL)
        if length in ("standard", "extended"):  # birdcage muzzle
            mx = x0 + blen
            d.rectangle([mx - 2, 10, mx, 14], fill=STEEL)
            d.line([mx - 1, 10, mx - 1, 14], fill=STEEL_DARK, width=1)
        if length == "pistol":  # brace hint below the receiver
            d.line([5, 16, 3, 21], fill=STEEL, width=2)
            d.rectangle([2, 21, 6, 23], fill=STEEL_DARK)
        small_part(d, rng, part)
    elif family == "axmc":
        # heavy precision barrel with a slotted muzzle brake, bolt below
        d.rectangle([3, 10, 7, 15], fill=STEEL)
        d.line([3, 10, 7, 10], fill=STEEL_LIGHT, width=1)
        x0 = 7
        barrel(d, x0, x0 + blen, 11, thick)
        d.line([x0 + 3, 12, x0 + blen - 5, 12], fill=STEEL_DARK, width=1)
        band(d, x0 + 2, 11, thick, caliber)
        mx = x0 + blen  # muzzle brake: taller block, three slots
        d.rectangle([mx - 4, 10, mx, 15], fill=STEEL)
        for sx in (mx - 3, mx - 2, mx - 1):
            d.line([sx, 11, sx, 14], fill=STEEL_DARK, width=1)
        d.rectangle([5, 20, 13, 22], fill=STEEL)  # bolt body
        d.line([13, 21, 17, 24], fill=STEEL, width=2)  # bolt handle
        d.ellipse([16, 23, 19, 26], fill=STEEL_DARK)  # handle knob
    elif family == "deagle":
        # short chunky pistol barrel over a wide slide
        barrel(d, 5, 5 + blen, 10, thick)
        d.rectangle([4, 10, 6, 14], fill=STEEL_DARK)  # muzzle crown
        band(d, 8, 10, thick, caliber)
        d.rectangle([6, 14, 21, 18], fill=STEEL)  # slide
        d.line([6, 14, 21, 14], fill=STEEL_LIGHT, width=1)
        for sx in (18, 20):  # rear slide serrations
            d.line([sx, 15, sx, 17], fill=STEEL_DARK, width=1)
        d.line([8, 19, 20, 19], fill=STEEL_DARK, width=1)  # frame rail
        d.polygon([(17, 19), (21, 19), (20, 24), (16, 24)],
                  fill=STEEL_DARK)  # grip hint
        d.rectangle([8, 20, 12, 24], fill=STEEL)  # loose magazine
        d.line([8, 22, 12, 22], fill=STEEL_DARK, width=1)
    elif family == "mdrx":
        # bullpup: barrel from the left, long action block at the rear
        x0 = 3
        barrel(d, x0, x0 + blen, 11, thick)
        rx = x0 + blen
        d.rectangle([rx, 10, rx + 8, 17], fill=STEEL)  # action/receiver
        d.line([rx, 10, rx + 8, 10], fill=STEEL_LIGHT, width=1)
        d.rectangle([rx + 2, 12, rx + 5, 14], fill=STEEL_DARK)  # port
        band(d, rx - 4, 11, thick, caliber)  # band near the chamber
        d.polygon([(rx + 1, 17), (rx + 5, 17), (rx + 4, 23), (rx, 23)],
                  fill=STEEL_DARK)  # curved magazine below the action
        small_part(d, rng, part)
    elif family == "ump":
        # short SMG barrel ending in a fat suppressor tube
        d.rectangle([3, 10, 8, 16], fill=STEEL)
        d.rectangle([4, 12, 6, 14], fill=STEEL_DARK)
        x0 = 8
        barrel(d, x0, x0 + blen, 11, thick)
        band(d, x0 + 2, 11, thick, caliber)
        sx = x0 + blen  # suppressor: taller tube with ring grooves
        d.rectangle([sx, 9, sx + 5, 16], fill=STEEL_DARK)
        d.line([sx, 9, sx + 5, 9], fill=STEEL_LIGHT, width=1)
        for gx in (sx + 1, sx + 3):
            d.line([gx, 10, gx, 15], fill=STEEL, width=1)
        d.polygon([(4, 16), (7, 16), (8, 23), (5, 23)],
                  fill=STEEL_DARK)  # stick magazine below the receiver
        d.rectangle([10, 20, 16, 23], fill=STEEL)  # loose bolt group
        d.line([12, 20, 12, 23], fill=STEEL_DARK, width=1)
    cartridges(d, rng, caliber)


# ------------------------------------------------------------------- sling

def draw_sling(d, rng):
    """Woven sling strap with a pouch cradling three round grenades."""
    web = (96, 104, 66, 255)
    canvas = (150, 130, 90, 255)
    # diagonal strap: dark edge under a webbing band
    d.line([6, 1, 24, 30], fill=shade(web, 0.55), width=9)
    d.line([6, 1, 24, 30], fill=web, width=7)
    for t in range(1, 9):  # woven cross-ticks along the strap
        x, y = 6 + t * 2.1, 1 + t * 3.4
        d.line([x - 3, y, x + 3, y + 2], fill=shade(web, 0.7), width=1)
    d.rectangle([3, 3, 10, 8], outline=(170, 172, 178, 255),
                width=2)  # buckle over the strap's top end
    # pouch: canvas cradle with a flap line and stitch dashes
    d.rectangle([12, 18, 28, 27], fill=shade(canvas, 0.6))
    d.rectangle([11, 17, 27, 26], fill=canvas)
    d.line([11, 21, 27, 21], fill=shade(canvas, 0.65), width=1)
    for x in range(13, 26, 4):
        d.line([x, 25, x + 2, 25], fill=shade(canvas, 0.7), width=1)
    # three grenades poking out of the pouch top
    for cx, cy in ((14, 16), (19, 14), (24, 16)):
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 4], fill=(70, 84, 52, 255))
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 4],
                  outline=(44, 54, 32, 255), width=1)
        d.point((cx - 1, cy - 1), fill=(150, 164, 110, 255))
        d.rectangle([cx - 1, cy - 6, cx + 1, cy - 3],
                    fill=(120, 122, 128, 255))  # fuse head
    d.line([19, 8, 22, 11], fill=(120, 122, 128, 255),
           width=2)  # spoon/lever on the middle grenade


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "staff_sling_grenade_cradle":
        draw_sling(d, rng)
    elif sid.startswith("retool_"):
        family, caliber, length = parse(sid)
        draw_kit(d, rng, family, caliber, length)
    else:
        raise KeyError(sid)
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
    # contact-sheet preview: 6 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item46.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item46.png {prev.size}")


if __name__ == "__main__":
    main()
