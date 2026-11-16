#!/usr/bin/env python3
"""ITEM40: MRE entrees/sides/dessert (35 tiles).

All 35 ids live in data/json/items/comestibles/mre.json. The 31 entrees
copy-from the `mre_entree` abstract (brown, 250 ml retort pouch), the four
sides (potato, fried_rice, zapplesauce, blackbeans) copy-from `mre_side`,
and mre_chocolate_cake is an MRE toaster pastry (copy-from toastem, with
chocolate/strawberry variants) drawn as the frosted pastry in foil.

Treatment: the classic tan MRE retort pouch, top torn open on its silver
foil layer with the actual dish heaped in the opening, plus a printed
label band whose accent color and white pictogram carry the entree
identity. Dish contents are parameterized per entry (sauce base, chunk /
strand / disc / grain toppings).
"""
import json
import math
import os
import zlib
from PIL import Image, ImageDraw

IDS = [
    "mre_asianbeef", "mre_bbqbeef", "mre_beef", "mre_beefstew",
    "mre_beeftaco", "mre_blackbeans", "mre_cheesepizza", "mre_cheesetort",
    "mre_chicken", "mre_chickenburritobowl", "mre_chickennoodle",
    "mre_chickenpesto", "mre_chilibeans", "mre_chilimac",
    "mre_chocolate_cake", "mre_fried_rice", "mre_hashbrownbacon",
    "mre_hotdog", "mre_lemontuna", "mre_lobstermedley",
    "mre_macaronimarinara", "mre_maplesausage", "mre_meatball",
    "mre_mexicanchickenstew", "mre_mushroomfettuccine",
    "mre_pepperjackbeef", "mre_pizza", "mre_potato", "mre_ratatouille",
    "mre_ravioli", "mre_sfettuccine", "mre_southwestbeef",
    "mre_spaghetti", "mre_veggy", "mre_zapplesauce",
]

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.")

# pouch palette
BODY = (138, 122, 82)
BODY_D = (109, 96, 62)
OUTLINE = (84, 74, 47)
SEAL = (96, 84, 54)
FOIL = (198, 198, 194)
FOIL_D = (158, 158, 152)

BEEF = (110, 66, 40)
BEEF_D = (86, 50, 30)
CHICK = (226, 198, 150)
RICE = (238, 228, 196)
BEAN_R = (122, 52, 34)
BEAN_B = (38, 32, 40)
TOMATO = (178, 58, 36)
CHEESE = (240, 178, 62)
PASTA = (238, 206, 130)
CREAM = (240, 228, 196)
GREEN = (86, 138, 62)
CARROT = (232, 122, 44)
PEPPER_G = (62, 140, 66)
PEPPER_R = (196, 52, 40)

# per-dish: (base food color, toppings [(kind, color, n)], band color, glyph)
DISH = {
    "mre_asianbeef": ((122, 78, 44),
                      [("slice", BEEF_D, 4), ("slice", CARROT, 3),
                       ("slice", PEPPER_G, 3)], (150, 84, 40), "beef"),
    "mre_bbqbeef": ((104, 44, 30),
                    [("chunk", BEEF_D, 6)], (140, 52, 34), "beef"),
    "mre_beef": ((116, 72, 42),
                 [("slice", BEEF, 5), ("slice", BEEF_D, 3)],
                 (120, 70, 40), "beef"),
    "mre_beefstew": ((128, 84, 46),
                     [("chunk", BEEF_D, 4), ("chunk", CARROT, 3),
                      ("chunk", (226, 204, 158), 3)], (140, 96, 48), "stew"),
    "mre_beeftaco": ((196, 158, 92),
                     [("chunk", BEEF_D, 5), ("grain", GREEN, 6),
                      ("grain", TOMATO, 4)], (196, 120, 44), "taco"),
    "mre_blackbeans": ((52, 44, 48),
                       [("disc", BEAN_B, 9), ("grain", (120, 90, 60), 5)],
                       (70, 60, 74), "bean"),
    "mre_cheesepizza": (CHEESE,
                        [("grain", (250, 214, 120), 8)], (226, 160, 52),
                        "pizza"),
    "mre_cheesetort": (TOMATO,
                       [("disc", PASTA, 7), ("grain", CHEESE, 5)],
                       (190, 84, 44), "pasta"),
    "mre_chicken": (CHICK,
                    [("chunk", (240, 216, 170), 6)], (216, 178, 96),
                    "drumstick"),
    "mre_chickenburritobowl": (RICE,
                               [("chunk", CHICK, 4), ("disc", BEAN_B, 5),
                                ("grain", (246, 200, 70), 6)],
                               (188, 110, 52), "bowl"),
    "mre_chickennoodle": ((232, 196, 110),
                          [("strand", PASTA, 5), ("chunk", CHICK, 4),
                           ("grain", CARROT, 4)], (226, 178, 84), "noodle"),
    "mre_chickenpesto": ((110, 156, 76),
                         [("strand", (140, 180, 96), 5),
                          ("chunk", CHICK, 4)], (96, 150, 70), "pasta"),
    "mre_chilibeans": ((150, 56, 34),
                       [("disc", BEAN_R, 7), ("chunk", BEEF_D, 3)],
                       (160, 60, 38), "bean"),
    "mre_chilimac": ((164, 66, 38),
                     [("strand", PASTA, 6), ("disc", BEAN_R, 3)],
                     (172, 76, 44), "noodle"),
    "mre_fried_rice": ((226, 196, 138),
                       [("grain", RICE, 10), ("grain", (120, 180, 80), 5),
                        ("grain", CARROT, 5)], (214, 178, 96), "rice"),
    "mre_hashbrownbacon": ((232, 184, 96),
                           [("strand", (240, 202, 116), 7),
                            ("grain", (150, 62, 44), 7)],
                           (222, 160, 70), "potato"),
    "mre_hotdog": ((196, 150, 92),
                   [("capsule", (196, 92, 66), 4), ("disc", (168, 116, 66), 5)],
                   (186, 96, 58), "sausage"),
    "mre_lemontuna": ((216, 178, 158),
                      [("grain", (232, 198, 178), 8),
                       ("wedge", (246, 216, 84), 2)], (232, 196, 92), "fish"),
    "mre_lobstermedley": ((240, 226, 204),
                          [("chunk", (222, 84, 56), 5),
                           ("chunk", (250, 240, 224), 4)], (216, 92, 64),
                          "fish"),
    "mre_macaronimarinara": (TOMATO,
                             [("strand", PASTA, 6)], (186, 70, 42), "pasta"),
    "mre_maplesausage": ((148, 88, 50),
                         [("capsule", (132, 74, 42), 3),
                          ("grain", (196, 130, 62), 6)], (176, 112, 54),
                         "sausage"),
    "mre_meatball": (TOMATO,
                     [("disc", BEEF, 5), ("disc", BEEF_D, 2)],
                     (172, 62, 40), "ball"),
    "mre_mexicanchickenstew": ((168, 84, 44),
                               [("chunk", CHICK, 4), ("disc", BEAN_B, 4),
                                ("grain", PEPPER_R, 4), ("grain", CARROT, 3)],
                               (178, 92, 46), "stew"),
    "mre_mushroomfettuccine": (CREAM,
                               [("strand", PASTA, 6),
                                ("slice", (168, 140, 108), 5)],
                               (196, 172, 128), "mushroom"),
    "mre_pepperjackbeef": (BEEF,
                           [("slice", (244, 196, 96), 4),
                            ("grain", PEPPER_R, 4), ("grain", PEPPER_G, 3)],
                           (206, 140, 62), "cheese"),
    "mre_pizza": (CHEESE,
                  [("disc", (188, 52, 40), 4), ("grain", (250, 214, 120), 6)],
                  (196, 84, 48), "pizza"),
    "mre_potato": ((240, 196, 96),
                   [("slice", (248, 216, 130), 6), ("grain", CHEESE, 6)],
                   (228, 178, 84), "potato"),
    "mre_ratatouille": ((150, 66, 44),
                        [("chunk", (104, 66, 128), 3), ("chunk", PEPPER_G, 3),
                         ("chunk", (120, 170, 84), 3), ("chunk", TOMATO, 3)],
                        (128, 110, 70), "leaf"),
    "mre_ravioli": ((196, 96, 52),
                    [("chunk", PASTA, 6)], (198, 104, 56), "pasta"),
    "mre_sfettuccine": ((126, 168, 88),
                        [("strand", (150, 190, 106), 7),
                         ("grain", CREAM, 5)], (110, 160, 84), "pasta"),
    "mre_southwestbeef": ((140, 66, 38),
                          [("chunk", BEEF_D, 4), ("disc", BEAN_B, 6),
                           ("grain", PEPPER_R, 3)], (152, 74, 42), "bean"),
    "mre_spaghetti": (TOMATO,
                      [("strand", PASTA, 8)], (184, 66, 40), "noodle"),
    "mre_veggy": ((186, 148, 88),
                  [("chunk", (120, 170, 84), 4), ("disc", BEAN_B, 4),
                   ("grain", TOMATO, 5), ("grain", (246, 200, 70), 4)],
                  (110, 158, 76), "taco"),
    "mre_zapplesauce": ((238, 186, 84),
                        [("grain", (248, 208, 116), 8)], (232, 180, 80),
                        "apple"),
}


def draw_glyph(d, kind, cx, cy, col):
    if kind == "pizza":
        d.polygon([(cx - 4, cy - 4), (cx + 4, cy - 4), (cx, cy + 5)],
                  fill=col)
        d.point((cx - 1, cy - 1), fill=(0, 0, 0, 0))
    elif kind == "fish":
        d.ellipse([cx - 5, cy - 2, cx + 2, cy + 3], fill=col)
        d.polygon([(cx + 2, cy), (cx + 5, cy - 3), (cx + 5, cy + 3)],
                  fill=col)
    elif kind == "leaf":
        d.ellipse([cx - 4, cy - 4, cx + 3, cy + 3], fill=col)
        d.line([cx - 3, cy + 3, cx + 4, cy - 4], fill=col)
    elif kind == "bean":
        d.ellipse([cx - 3, cy - 4, cx + 3, cy + 4], fill=col)
        d.ellipse([cx + 1, cy - 4, cx + 5, cy + 4], fill=col)
    elif kind == "noodle":
        for i in range(3):
            y = cy - 3 + i * 3
            d.arc([cx - 5, y - 2, cx + 5, y + 3], 0, 180, fill=col)
    elif kind == "drumstick":
        d.ellipse([cx - 4, cy - 4, cx + 2, cy + 2], fill=col)
        d.line([cx + 1, cy + 1, cx + 4, cy + 4], fill=col, width=2)
        d.ellipse([cx + 3, cy + 3, cx + 5, cy + 5], fill=col)
    elif kind == "sausage":
        d.rounded_rectangle([cx - 5, cy - 2, cx + 5, cy + 2], 2, fill=col)
    elif kind == "ball":
        d.ellipse([cx - 4, cy - 4, cx + 4, cy + 4], fill=col)
    elif kind == "taco":
        d.pieslice([cx - 5, cy - 5, cx + 5, cy + 5], 180, 360, fill=col)
    elif kind == "bowl":
        d.pieslice([cx - 5, cy - 6, cx + 5, cy + 4], 0, 180, fill=col)
        d.line([cx - 5, cy - 1, cx + 5, cy - 1], fill=col)
    elif kind == "stew":
        d.rectangle([cx - 4, cy - 1, cx + 4, cy + 4], fill=col)
        d.line([cx - 5, cy - 3, cx + 5, cy - 3], fill=col, width=2)
    elif kind == "pasta":
        d.rectangle([cx - 4, cy - 3, cx + 4, cy + 3], outline=col)
        d.line([cx - 4, cy, cx + 4, cy], fill=col)
    elif kind == "rice":
        for dx, dy in ((-3, -2), (0, 1), (3, -2), (-1, -3), (2, 3)):
            d.ellipse([cx + dx - 1, cy + dy - 1, cx + dx + 1, cy + dy + 1],
                      fill=col)
    elif kind == "potato":
        d.ellipse([cx - 4, cy - 3, cx + 4, cy + 3], fill=col)
    elif kind == "apple":
        d.ellipse([cx - 4, cy - 2, cx + 4, cy + 4], fill=col)
        d.line([cx, cy - 2, cx + 1, cy - 5], fill=col, width=1)
    elif kind == "mushroom":
        d.pieslice([cx - 5, cy - 5, cx + 5, cy + 3], 180, 360, fill=col)
        d.rectangle([cx - 1, cy, cx + 1, cy + 4], fill=col)
    elif kind == "cheese":
        d.polygon([(cx - 5, cy + 3), (cx + 5, cy + 3), (cx + 5, cy - 3)],
                  fill=col)
    elif kind == "beef":
        d.rectangle([cx - 4, cy - 3, cx + 4, cy + 3], fill=col)
        d.rectangle([cx - 2, cy - 1, cx + 2, cy + 1],
                    fill=(255, 255, 255, 255))


def draw_topping(d, rng, kind, color, cx, cy):
    if kind == "chunk":
        w = rng.randint(3, 5)
        d.rounded_rectangle([cx - w // 2, cy - 2, cx + w // 2, cy + 2], 1,
                            fill=color + (255,))
    elif kind == "slice":
        w = rng.randint(4, 6)
        d.rectangle([cx - w // 2, cy - 1, cx + w // 2, cy + 1],
                    fill=color + (255,))
    elif kind == "disc":
        r = rng.randint(2, 3)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=color + (255,))
    elif kind == "grain":
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=color + (255,))
    elif kind == "strand":
        pts = []
        for i in range(7):
            x = cx - 4 + i
            y = cy + int(round(1.6 * math.sin(i * 1.4 + rng.random())))
            pts.append((x, y))
        d.line(pts, fill=color + (255,), width=1)
    elif kind == "capsule":
        d.rounded_rectangle([cx - 2, cy - 4, cx + 2, cy + 4], 2,
                            fill=color + (255,))
    elif kind == "wedge":
        d.polygon([(cx - 3, cy + 2), (cx + 3, cy + 2), (cx, cy - 3)],
                  fill=color + (255,))


def draw_pouch_entree(item_id):
    import random
    rng = random.Random(zlib.crc32(item_id.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    base, toppings, band, glyph = DISH[item_id]

    # food heap in the torn opening (drawn first, pouch overlaps bottom)
    d.ellipse([5, 4, 27, 15], fill=base + (255,))
    d.ellipse([8, 3, 24, 12], fill=base + (255,))
    for kind, color, n in toppings:
        for _ in range(n):
            cx = rng.randint(8, 24)
            cy = rng.randint(5, 11)
            draw_topping(d, rng, kind, color, cx, cy)

    # pouch body
    d.rounded_rectangle([5, 12, 27, 30], 2, fill=BODY + (255,),
                        outline=OUTLINE + (255,))
    d.rectangle([5, 26, 27, 30], fill=BODY_D + (255,))
    d.rounded_rectangle([5, 12, 27, 30], 2, outline=OUTLINE + (255,))
    # bottom crimp seal
    for x in range(7, 27, 3):
        d.line([x, 27, x, 29], fill=SEAL + (255,))
    # torn foil edge across the top of the pouch front
    pts = [(5, 13)]
    x = 5
    while x < 27:
        x += rng.randint(2, 4)
        pts.append((min(x, 27), rng.randint(12, 14)))
    d.line(pts, fill=FOIL + (255,), width=2)
    d.line([(px, py + 1) for px, py in pts], fill=FOIL_D + (255,))
    # label band with accent color + white glyph
    d.rectangle([5, 19, 27, 25], fill=band + (255,))
    d.rectangle([5, 19, 27, 19], fill=(255, 255, 255, 255))
    draw_glyph(d, glyph, 16, 22, (255, 255, 255, 255))
    return img


def draw_toaster_pastry():
    import random
    rng = random.Random(zlib.crc32(b"mre_chocolate_cake"))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # silver foil pouch behind, open at top
    d.rounded_rectangle([7, 14, 25, 30], 2, fill=FOIL + (255,),
                        outline=FOIL_D + (255,))
    for x in range(9, 25, 3):
        d.line([x, 27, x, 29], fill=FOIL_D + (255,))
    d.rectangle([7, 21, 25, 25], fill=(150, 62, 44, 255))
    draw_glyph(d, "ball", 16, 23, (255, 255, 255, 255))
    # the pastry itself, leaning out of the pouch
    d.rounded_rectangle([8, 4, 24, 17], 3, fill=(216, 176, 116, 255),
                        outline=(168, 128, 76, 255))
    d.rounded_rectangle([10, 6, 22, 15], 2, fill=(106, 62, 38, 255))
    for _ in range(9):  # sprinkles
        x = rng.randint(11, 21)
        y = rng.randint(7, 14)
        col = [(240, 120, 150), (250, 220, 90), (140, 210, 140),
               (140, 180, 250)][rng.randint(0, 3)]
        d.point((x, y), fill=col + (255,))
    return img


def main():
    os.makedirs(ROOT, exist_ok=True)
    tiles = []
    for item_id in IDS:
        if item_id == "mre_chocolate_cake":
            img = draw_toaster_pastry()
        else:
            img = draw_pouch_entree(item_id)
        assert img.size == (32, 32)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (item_id, alphas)
        folder = os.path.join(ROOT, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f)
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        tiles.append((item_id, img))

    # contact-sheet preview, 8 cols
    cols = 8
    rows = (len(tiles) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * 36 + 4, rows * 36 + 4), (24, 24, 28, 255))
    for i, (_, img) in enumerate(tiles):
        x = 4 + (i % cols) * 36
        y = 4 + (i // cols) * 36
        sheet.alpha_composite(img, (x + 2, y + 2))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item40.png")
    sheet.convert("RGB").save(out)
    print("wrote %d sprites + preview_item40.png %s" % (len(tiles), sheet.size))


if __name__ == "__main__":
    main()
