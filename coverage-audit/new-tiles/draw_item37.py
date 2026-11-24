#!/usr/bin/env python3
"""ITEM37: other comestibles (39 tiles) — CDDA Voxel coverage audit.

Draws each id from data/json/items/comestibles/other.json as the food
itself or in its container, parameterized by the entry. Self-contained:
the id list is embedded. Per-sprite random.Random(crc32(id)).
Opacity rule: alpha only {0,255}; translucency faked with pale colors.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.abspath(__file__))
ITEMS_DIR = os.path.join(ROOT, "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

IDS = [
    "acid_soaked_hide", "blood_rice_pudding", "camp_meal_large",
    "camp_meal_medium", "camp_meal_small", "catfood_raw", "cocoa_powder",
    "cocoa_powder_milk", "coffee_raw_kentucky", "dogfood_raw", "dry_soybean",
    "dry_tofu", "earthworm_boiled", "earthworm_dried", "earthworm_fried",
    "foodplace_food", "foodplace_snack_bar", "gelatin_extracted", "grass",
    "gravy_prepared", "honey_glassed", "honeycomb", "human_brain_embalmed",
    "marloss_berry", "marloss_gel", "napkin", "nectar", "paste_nut",
    "paste_seed", "press_cake", "raw_edamame", "royal_jelly", "small_plant",
    "soybean", "swarm_bees", "tofu", "tofu_fried", "underbrush",
    "yogurt_starter_culture",
]
assert len(IDS) == 39


def rng_for(i):
    return random.Random(zlib.crc32(i.encode()))


def new_img():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def bowl(d, x0, y0, x1, y1, body, rim):
    d.ellipse([x0, y0, x1, y0 + 8], fill=rim)
    d.pieslice([x0, y0 - 2, x1, y1 + 6], 0, 180, fill=body)
    d.ellipse([x0 + 1, y0 + 1, x1 - 1, y0 + 7], fill=rim)


def beans(d, r, base, dark, light, n=14, hard=False):
    for _ in range(n):
        x = r.randint(7, 22)
        y = r.randint(17, 26)
        w = r.randint(4, 6)
        d.ellipse([x, y, x + w, y + 3], fill=base, outline=dark)
        d.point((x + 1, y + 1), fill=light)
        if hard:
            d.point((x + w - 1, y + 1), fill=light)


def powder_bag(d, r, powder, label):
    # small plastic bag (bag_plastic_small), pale opaque "plastic"
    d.polygon([(9, 6), (23, 6), (25, 28), (7, 28)], fill=(205, 205, 195))
    d.polygon([(10, 13), (22, 13), (23, 27), (9, 27)], fill=powder)
    d.rectangle([9, 6, 23, 9], fill=(170, 170, 160))  # sealed top
    d.rectangle([12, 16, 20, 22], fill=label)
    for _ in range(10):
        d.point((r.randint(10, 22), r.randint(24, 27)), fill=(255, 255, 250))


def meat_mound(d, r, w, fleck=None):
    # minced meat in an open steel tin
    cx0, cx1 = 16 - w // 2, 16 + w // 2
    d.rectangle([cx0, 16, cx1, 27], fill=(150, 150, 155))
    d.rectangle([cx0, 16, cx1, 18], fill=(185, 185, 190))
    d.ellipse([cx0 - 1, 9, cx1 + 1, 19], fill=(175, 70, 60))
    for _ in range(26):
        x = r.randint(cx0, cx1 - 1)
        y = r.randint(11, 17)
        d.point((x, y), fill=r.choice([(150, 50, 45), (195, 90, 75), (130, 45, 40)]))
    if fleck:
        for _ in range(8):
            d.point((r.randint(cx0, cx1 - 1), r.randint(11, 17)), fill=fleck)


def worm_pile(d, r, seg, dark):
    for k in range(6):
        x = 8 + (k % 3) * 6
        y = 13 + (k // 3) * 7
        d.arc([x, y, x + 10, y + 8], 200, 520, fill=seg, width=3)
        d.arc([x + 1, y + 1, x + 9, y + 7], 200, 520, fill=dark, width=1)


def tofu_block(d, r, fillc, edge, fried=False):
    d.polygon([(7, 13), (25, 13), (27, 17), (9, 17)], fill=edge)  # top
    d.rectangle([9, 17, 27, 27], fill=fillc)
    d.polygon([(25, 13), (27, 17), (27, 27), (25, 27)], fill=edge)
    if fried:
        for _ in range(14):
            d.point((r.randint(10, 26), r.randint(18, 26)), fill=(170, 110, 35))
    else:
        d.line([10, 20, 26, 20], fill=edge)


def hex_chunk(d, r, wax, fillc):
    pts = [(16, 5), (26, 10), (26, 22), (16, 27), (6, 22), (6, 10)]
    d.polygon(pts, fill=wax)
    for (hx, hy) in [(11, 11), (19, 11), (15, 17), (11, 21), (20, 20)]:
        cell = [(hx, hy - 3), (hx + 3, hy - 1), (hx + 3, hy + 2),
                (hx, hy + 4), (hx - 3, hy + 2), (hx - 3, hy - 1)]
        d.polygon(cell, fill=fillc, outline=wax)


def jar(d, r, liquid, lid=(120, 120, 125)):
    d.rectangle([9, 8, 23, 10], fill=lid)
    d.rectangle([10, 10, 22, 28], fill=(215, 220, 215))  # glass, pale opaque
    d.rectangle([11, 14, 21, 27], fill=liquid)


def paste_bowl(d, r, paste, fleck=None):
    bowl(d, 5, 15, 27, 27, (140, 140, 148), (175, 175, 182))
    d.ellipse([6, 12, 26, 20], fill=paste)
    for _ in range(12):
        d.point((r.randint(8, 24), r.randint(14, 18)),
                fill=fleck or paste)


def draw(i):
    r = rng_for(i)
    img = new_img()
    d = ImageDraw.Draw(img)

    if i == "acid_soaked_hide":
        d.ellipse([4, 14, 28, 29], fill=(130, 130, 135))       # basin
        d.ellipse([5, 13, 27, 22], fill=(240, 230, 140))       # acid
        d.polygon([(9, 12), (20, 10), (24, 16), (14, 20), (8, 17)],
                  fill=(176, 141, 87))                          # hide
        d.polygon([(11, 13), (18, 12), (20, 15), (13, 17)],
                  fill=(195, 160, 105))
        for _ in range(6):
            bx, by = r.randint(7, 23), r.randint(14, 19)
            d.ellipse([bx, by, bx + 2, by + 2], outline=(255, 250, 200))
    elif i == "blood_rice_pudding":
        d.ellipse([5, 18, 27, 28], fill=(200, 200, 205))       # plate
        d.pieslice([7, 8, 25, 26], 180, 360, fill=(110, 32, 30))
        d.pieslice([7, 8, 25, 26], 0, 180, fill=(110, 32, 30))
        for _ in range(12):                                     # rice grains
            d.point((r.randint(9, 23), r.randint(11, 17)), fill=(235, 230, 215))
        for _ in range(5):                                      # peanut bits
            d.point((r.randint(9, 23), r.randint(11, 16)), fill=(215, 180, 120))
    elif i.startswith("camp_meal"):
        s = {"camp_meal_small": 0, "camp_meal_medium": 1, "camp_meal_large": 2}[i]
        w = 14 + s * 5
        x0, x1 = 16 - w // 2, 16 + w // 2
        d.rectangle([x0, 9, x1, 27], fill=(130, 95, 60))       # brown pouch/tray
        d.rectangle([x0, 9, x1, 12], fill=(105, 75, 45))
        d.rectangle([x0 + 2, 15, x1 - 2, 24], fill=(160, 125, 85))
        if s == 2:                                              # compartments
            d.line([16, 15, 16, 24], fill=(105, 75, 45), width=2)
            d.line([x0 + 2, 19, x1 - 2, 19], fill=(105, 75, 45))
        if s >= 1:
            d.rectangle([x0 + 3, 17, x0 + 7, 21], fill=(120, 110, 60))
    elif i == "catfood_raw":
        meat_mound(d, r, 16)
    elif i == "dogfood_raw":
        meat_mound(d, r, 22, fleck=(80, 150, 60))
    elif i == "cocoa_powder":
        powder_bag(d, r, (90, 55, 30), (240, 240, 235))
    elif i == "cocoa_powder_milk":
        powder_bag(d, r, (155, 108, 72), (240, 240, 235))
    elif i == "yogurt_starter_culture":
        powder_bag(d, r, (245, 243, 235), (170, 200, 220))
    elif i == "coffee_raw_kentucky":
        d.polygon([(8, 12), (24, 12), (26, 28), (6, 28)], fill=(170, 140, 95))
        d.ellipse([7, 8, 25, 15], fill=(60, 40, 25))           # grounds mound
        for _ in range(20):
            d.point((r.randint(9, 23), r.randint(9, 13)),
                    fill=r.choice([(45, 30, 18), (80, 55, 35), (100, 70, 45)]))
    elif i == "dry_soybean":
        d.ellipse([5, 20, 27, 28], fill=(150, 120, 80))
        beans(d, r, (198, 168, 108), (150, 125, 75), (235, 215, 165), hard=True)
    elif i == "soybean":
        d.ellipse([5, 20, 27, 28], fill=(150, 120, 80))
        beans(d, r, (215, 190, 130), (165, 140, 90), (245, 230, 185))
    elif i == "dry_tofu":
        tofu_block(d, r, (232, 226, 208), (200, 192, 168))
        d.line([9, 23, 27, 23], fill=(200, 192, 168))          # seal wrap line
    elif i == "tofu":
        tofu_block(d, r, (246, 246, 241), (215, 215, 205))
    elif i == "tofu_fried":
        tofu_block(d, r, (218, 165, 65), (185, 130, 45), fried=True)
    elif i == "earthworm_boiled":
        worm_pile(d, r, (208, 165, 145), (170, 125, 105))
    elif i == "earthworm_fried":
        worm_pile(d, r, (185, 125, 55), (145, 90, 35))
        for _ in range(8):                                      # salt
            d.point((r.randint(7, 25), r.randint(12, 25)), fill=(250, 250, 245))
    elif i == "earthworm_dried":
        d.polygon([(8, 27), (13, 16), (20, 16), (25, 27)], fill=(120, 85, 55))
        for _ in range(22):
            d.point((r.randint(10, 23), r.randint(18, 26)),
                    fill=r.choice([(95, 65, 40), (145, 108, 72), (110, 78, 50)]))
    elif i == "foodplace_food":
        d.rectangle([7, 10, 25, 27], fill=(200, 40, 160))      # magenta box
        d.rectangle([7, 10, 25, 14], fill=(225, 80, 185))
        d.ellipse([11, 17, 21, 24], fill=(235, 200, 120))      # foodstuff blob
        d.rectangle([13, 19, 19, 21], fill=(250, 240, 200))
    elif i == "foodplace_snack_bar":
        d.polygon([(6, 13), (26, 13), (26, 20), (6, 20)], fill=(200, 40, 160))
        for x in (5, 26):                                       # crimped ends
            d.line([x, 13, x, 20], fill=(160, 25, 125), width=2)
        d.rectangle([11, 15, 21, 18], fill=(240, 220, 240))
    elif i == "gelatin_extracted":
        jar(d, r, (240, 226, 140))
        d.pieslice([12, 16, 20, 24], 180, 360, fill=(246, 236, 165))
        for _ in range(5):
            d.point((r.randint(12, 20), r.randint(18, 25)), fill=(250, 244, 190))
    elif i == "grass":
        d.rectangle([6, 26, 26, 28], fill=(100, 75, 45))       # soil
        for k in range(9):
            x = 7 + k * 2.4
            h = r.randint(11, 18)
            d.line([x, 27, x + r.choice([-2, 0, 2]), 27 - h],
                   fill=r.choice([(55, 150, 55), (70, 175, 65), (45, 130, 50)]), width=2)
    elif i == "gravy_prepared":
        d.ellipse([6, 14, 24, 24], fill=(150, 150, 155))       # gravy boat
        d.polygon([(22, 16), (28, 18), (22, 21)], fill=(150, 150, 155))
        d.rectangle([10, 24, 20, 27], fill=(130, 130, 135))
        d.ellipse([7, 12, 23, 18], fill=(122, 78, 42))
        d.ellipse([10, 13, 17, 16], fill=(150, 105, 62))       # sheen
    elif i == "honey_glassed":
        jar(d, r, (238, 175, 35), lid=(150, 60, 40))
        d.rectangle([11, 14, 21, 17], fill=(245, 195, 70))     # thick top layer
    elif i == "honeycomb":
        hex_chunk(d, r, (238, 190, 70), (250, 205, 90))
        d.polygon([(13, 15), (18, 15), (17, 20), (14, 20)], fill=(245, 180, 40))
    elif i == "royal_jelly":
        hex_chunk(d, r, (238, 228, 195), (250, 248, 236))
    elif i == "human_brain_embalmed":
        jar(d, r, (222, 226, 205))
        d.ellipse([12, 15, 20, 22], fill=(236, 152, 162))      # brain
        d.arc([13, 16, 17, 21], 90, 330, fill=(205, 110, 125))
        d.arc([16, 16, 20, 21], 200, 480, fill=(205, 110, 125))
        d.line([16, 15, 16, 22], fill=(205, 110, 125))
    elif i == "marloss_berry":
        d.ellipse([7, 8, 25, 27], fill=(240, 130, 170))        # fist-sized berry
        d.ellipse([10, 11, 16, 17], fill=(250, 175, 200))
        d.line([16, 8, 17, 5], fill=(90, 140, 60), width=2)
        for _ in range(8):
            d.point((r.randint(10, 23), r.randint(12, 24)), fill=(225, 105, 150))
    elif i == "marloss_gel":
        d.pieslice([6, 10, 26, 28], 180, 360, fill=(245, 230, 110))
        d.rectangle([6, 19, 26, 26], fill=(245, 230, 110))
        d.ellipse([9, 12, 17, 17], fill=(250, 242, 165))       # wobble sheen
        d.ellipse([5, 25, 27, 28], fill=(200, 200, 205))       # plate
    elif i == "napkin":
        d.polygon([(10, 7), (24, 10), (22, 26), (8, 23)], fill=(246, 246, 246))
        d.line([12, 9, 10, 23], fill=(205, 205, 205))
        d.line([18, 10, 16, 25], fill=(215, 215, 215))
    elif i == "nectar":
        d.polygon([(10, 22), (22, 22), (19, 12), (13, 12)], fill=(210, 90, 140))
        d.ellipse([11, 10, 21, 15], fill=(225, 120, 165))      # flower cup
        for (x, y) in [(13, 17), (17, 19), (15, 14)]:          # nectar drops
            d.ellipse([x, y, x + 3, y + 4], fill=(185, 205, 85))
            d.point((x + 1, y + 1), fill=(230, 240, 170))
    elif i == "paste_nut":
        paste_bowl(d, r, (112, 76, 44), (90, 58, 32))
    elif i == "paste_seed":
        paste_bowl(d, r, (172, 142, 96), (140, 112, 70))
    elif i == "press_cake":
        d.ellipse([6, 12, 26, 26], fill=(222, 186, 102))       # dry cake disc
        d.ellipse([6, 10, 26, 20], fill=(235, 205, 130))
        for _ in range(14):                                     # crumbly edge
            d.point((r.randint(7, 25), r.randint(11, 19)), fill=(200, 160, 80))
        d.point((9, 24), fill=(200, 160, 80)); d.point((22, 25), fill=(200, 160, 80))
    elif i == "raw_edamame":
        for (x, y, a) in [(8, 10, -25), (14, 8, 0), (20, 10, 25)]:
            rad = math.radians(a)
            ex, ey = x + int(9 * math.sin(rad)), y + int(13 * math.cos(rad))
            d.line([x, y, ex, ey], fill=(90, 170, 70), width=5)
            d.line([x, y, ex, ey], fill=(115, 195, 90), width=2)
            for t in (0.3, 0.55, 0.8):                          # bean bumps
                bx, by = int(x + (ex - x) * t), int(y + (ey - y) * t)
                d.ellipse([bx - 2, by - 2, bx + 2, by + 2], fill=(105, 185, 80))
    elif i == "small_plant":
        d.rectangle([7, 25, 25, 28], fill=(100, 75, 45))
        d.line([16, 26, 16, 13], fill=(60, 150, 60), width=2)
        d.ellipse([8, 14, 15, 19], fill=(70, 170, 65))
        d.ellipse([17, 12, 24, 17], fill=(80, 180, 70))
        d.ellipse([14, 7, 18, 11], fill=(245, 150, 190))       # flower
        d.point((16, 9), fill=(250, 220, 90))
    elif i == "swarm_bees":
        for _ in range(11):
            x = r.randint(6, 22)
            y = r.randint(7, 22)
            d.ellipse([x - 2, y - 3, x + 4, y + 1], fill=(215, 225, 230))  # wing
            d.ellipse([x, y, x + 5, y + 4], fill=(240, 195, 40))
            d.line([x + 2, y, x + 2, y + 3], fill=(40, 35, 30), width=1)
            d.line([x + 4, y, x + 4, y + 3], fill=(40, 35, 30), width=1)
    elif i == "underbrush":
        for k in range(6):                                      # twigs
            x = 9 + k * 2.6
            d.line([x, 27, x + r.choice([-3, 3]), 9], fill=(120, 90, 55), width=2)
        for _ in range(12):                                     # leaves
            x, y = r.randint(7, 23), r.randint(9, 22)
            d.ellipse([x, y, x + 4, y + 3], fill=r.choice([(60, 140, 55), (80, 160, 65), (100, 120, 50)]))
        d.rectangle([7, 22, 25, 25], fill=(150, 120, 70))       # tie band
    else:
        raise ValueError(i)

    assert img.size == (32, 32)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (i, alphas)
    return img


def main():
    for i in IDS:
        img = draw(i)
        folder = os.path.join(ITEMS_DIR, i)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, i + ".png"))
        with open(os.path.join(folder, i + ".json"), "w") as f:
            json.dump([{"id": i, "fg": i, "bg": ""}], f)
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # preview contact sheet
    cols, rows, cell = 8, 5, 66
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (30, 30, 30, 255))
    for n, i in enumerate(IDS):
        img = draw(i).resize((64, 64), Image.NEAREST)
        x = (n % cols) * cell + 1
        y = (n // cols) * cell + 1
        sheet.alpha_composite(img, (x, y))
    sheet.convert("RGB").save(os.path.join(ROOT, "preview_item37.png"))
    print(f"wrote {len(IDS)} sprites + preview_item37.png {sheet.size}")


if __name__ == "__main__":
    main()
