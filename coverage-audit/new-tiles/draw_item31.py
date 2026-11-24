#!/usr/bin/env python3
"""Batch ITEM31 (Phase 2g.4 scope extension): junk food.

47 ids (authoritative list embedded below; source list:
~/workspace/item-batches/31.txt). All 47 are defined in
data/json/items/comestibles/junkfood.json; entries were
read first and copy-from chains resolved (candy3gator <-
candy3, choc_pancakes <- pancakes, cookies_egg <-
cookies, cow_candy <- candy2, debug_calories <-
jihelucake, grahmcrackers_roe <- grahmcrackers,
toasterpastryfrozen <- toastem).

Treatment: each snack drawn as itself or in the
packaging its entry names -- the two pan pastas in a
skillet (amatriciana red sauce with bacon and pecorino;
carbonara cream-yellow with bacon and pepper), fast-food
cartons for buffalo wings / fried chicken / fries,
poutine as a gravy-and-curd fry pile, freeze-dried meals
as sealed red pouches with a label motif vs. their
hydrated twins as opened brown pouches with the food
mounded at the opening, a frosty divided TV-dinner tray,
small candy bags with the candies themselves carrying
the identity (peanut butter cups, coated chocolates,
chews, gator-shaped chews, gummy bears, paler vegan
gummy worms, mint patties, Necco wafer roll), pretzel
twists plain vs. chocolate-covered, chocolate bars
wrapped vs. the thick plain military bar, syrups in
glass bottles, toaster pastries frosted red -- the
frozen one paler and frost-speckled -- and the unholy
debug angel cake as a white ring cake under a halo.

Glass/plastic translucency is faked with pale opaque
colors (this Pillow build replaces rather than blends);
all sprite alpha is in {0, 255}, asserted per sprite.

Writes ONLY its own 47 folders + preview_item31.png.
PIL only; per-sprite random.Random(crc32(id)).
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "amatriciana_pasta",
    "buffalo_wings",
    "buttercream",
    "candy",
    "candy2",
    "candy3",
    "candy3gator",
    "candy4",
    "candy5",
    "candy_coated_peanuts",
    "candycigarette",
    "caramel_apple",
    "choc_pancakes",
    "choc_waffles",
    "chocolate",
    "chocolate_military",
    "chocpretzels",
    "cookies_egg",
    "cow_candy",
    "debug_calories",
    "fchicken",
    "freeze_dried_meal",
    "freeze_dried_meal_hydrated",
    "freeze_dried_stroganoff",
    "freeze_dried_stroganoff_hydrated",
    "freeze_dried_veggypasta",
    "freeze_dried_veggypasta_hydrated",
    "fries",
    "frozen_dinner",
    "gelatin_dessert_powder",
    "grahmcrackers",
    "grahmcrackers_roe",
    "maltballs",
    "marshmallow_fluff",
    "mintpatties",
    "neccowafers",
    "onion_rings",
    "pasta_carbonara",
    "porkstick",
    "poutine",
    "pretzels",
    "smores",
    "sugar_fried",
    "syrup_corn",
    "syrup_simple",
    "toastem",
    "toasterpastryfrozen",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.\n")

INK = (26, 24, 22, 255)
# pasta / pan
PAN = (52, 52, 58, 255)
PAN_D = (30, 30, 34, 255)
PAN_L = (96, 98, 106, 255)
NOODLE = (232, 198, 110, 255)
NOODLE_D = (206, 168, 82, 255)
SAUCE_RED = (188, 52, 34, 255)
SAUCE_RED_D = (150, 36, 24, 255)
CREAM = (240, 220, 150, 255)
CREAM_D = (222, 192, 108, 255)
BACON = (146, 52, 38, 255)
BACON_L = (196, 110, 84, 255)
PECORINO = (244, 240, 224, 255)
PEPPER = (40, 36, 32, 255)
# fast food
CARTON = (198, 52, 44, 255)
CARTON_D = (158, 36, 32, 255)
CARTON_IN = (238, 226, 205, 255)
FRY = (240, 198, 92, 255)
FRY_D = (214, 162, 58, 255)
CRISP = (196, 128, 44, 255)
CRISP_D = (156, 94, 28, 255)
CRISP_L = (228, 172, 88, 255)
WING = (206, 92, 36, 255)
WING_D = (168, 62, 24, 255)
WING_L = (236, 138, 70, 255)
BONE = (240, 236, 224, 255)
GRAVY = (128, 82, 44, 255)
GRAVY_D = (98, 60, 30, 255)
CURD = (246, 242, 228, 255)
RING = (216, 156, 66, 255)
RING_D = (178, 116, 40, 255)
# packaging
BAG = (198, 214, 220, 255)
BAG_D = (138, 162, 172, 255)
BAG_L = (232, 242, 244, 255)
POUCH_R = (172, 62, 48, 255)
POUCH_R_D = (132, 44, 36, 255)
POUCH_B = (148, 104, 66, 255)
POUCH_B_D = (110, 74, 44, 255)
LABEL = (240, 232, 208, 255)
LABEL_D = (206, 192, 158, 255)
BOXW = (238, 236, 230, 255)
BOXW_D = (196, 192, 182, 255)
GLASS = (208, 232, 220, 255)
GLASS_D = (150, 188, 166, 255)
# sweets
CHOC = (94, 56, 34, 255)
CHOC_D = (62, 34, 20, 255)
CHOC_L = (128, 82, 50, 255)
MILKCHOC = (140, 88, 52, 255)
CARAMEL = (206, 138, 58, 255)
CARAMEL_D = (168, 102, 36, 255)
CARAMEL_L = (238, 182, 100, 255)
GRAHAM = (206, 164, 100, 255)
GRAHAM_D = (172, 128, 66, 255)
COOKIE = (196, 148, 88, 255)
COOKIE_D = (158, 112, 58, 255)
WHITE = (246, 246, 242, 255)
FROST_RED = (216, 84, 92, 255)
FROST_RED_D = (186, 58, 68, 255)
PASTRY = (220, 182, 116, 255)
PASTRY_D = (186, 144, 80, 255)
SYRUP = (146, 84, 30, 255)
BUTTER = (248, 220, 110, 255)
APPLE = (198, 44, 38, 255)
APPLE_D = (158, 30, 28, 255)
STICK = (170, 128, 76, 255)
CANDYcols = [(208, 52, 44, 255), (52, 108, 196, 255),
             (66, 160, 66, 255), (240, 196, 52, 255),
             (140, 88, 52, 255), (232, 128, 40, 255)]
PASTEL = [(244, 196, 120, 255), (246, 226, 150, 255),
          (178, 226, 150, 255), (246, 190, 200, 255),
          (240, 240, 236, 255), (190, 160, 130, 255),
          (246, 210, 170, 255), (200, 220, 240, 255)]


def chunk(d, cx, cy, r, fill, rng, squash=0.75):
    """An irregular rounded blob."""
    pts = []
    n = 7
    for i in range(n):
        ang = 6.2832 * i / n
        rr = r * rng.uniform(0.78, 1.12)
        pts.append((cx + rr * math.cos(ang),
                    cy + rr * squash * math.sin(ang)))
    d.polygon(pts, fill=fill)


def small_bag(d, contents):
    """A small translucent-look plastic bag; contents is a
    callback drawn inside before the outline/sheen."""
    d.rounded_rectangle([8, 9, 24, 29], radius=4, fill=BAG)
    contents(d)
    # crimped top seal
    d.rectangle([8, 6, 24, 10], fill=BAG_D)
    for x in range(9, 24, 2):
        d.point((x, 7), fill=BAG_L)
        d.point((x, 9), fill=BAG_L)
    d.rounded_rectangle([8, 9, 24, 29], radius=4, outline=BAG_D)
    d.line([(11, 12), (11, 25)], fill=BAG_L, width=1)


def pan(d):
    d.ellipse([3, 6, 27, 29], fill=PAN)
    d.ellipse([3, 6, 27, 29], outline=PAN_D)
    d.polygon([(24, 15), (31, 12), (31, 17), (25, 19)], fill=PAN_D)
    d.ellipse([5, 8, 25, 27], fill=PAN_L)


def noodles(d, rng, sauce, sauce_d, base, base_d, cx=15, cy=17):
    d.ellipse([6, 9, 24, 26], fill=sauce)
    for _ in range(26):
        x = rng.randrange(8, 21)
        y = rng.randrange(11, 23)
        c = base if rng.random() < 0.6 else base_d
        d.arc([x, y, x + 5, y + 4], rng.randrange(0, 360),
              rng.randrange(180, 330), fill=c, width=1)
    for _ in range(10):
        x = rng.randrange(8, 22)
        y = rng.randrange(11, 24)
        d.point((x, y), fill=sauce_d)


def bacon_bits(d, rng, n=7):
    for _ in range(n):
        x = rng.randrange(8, 21)
        y = rng.randrange(11, 23)
        d.rectangle([x, y, x + 2, y + 1], fill=BACON)
        d.point((x, y), fill=BACON_L)


def carton(d, color=CARTON, color_d=CARTON_D):
    """Open fast-food carton; food is drawn after, heaped
    above the rim."""
    d.polygon([(7, 16), (25, 16), (23, 29), (9, 29)], fill=color)
    d.polygon([(7, 16), (25, 16), (23, 29), (9, 29)], outline=color_d)
    d.rectangle([6, 14, 26, 17], fill=color)
    d.rectangle([6, 14, 26, 15], fill=CARTON_IN)
    d.rectangle([13, 20, 19, 25], fill=CARTON_IN)
    d.rectangle([13, 20, 19, 25], outline=color_d)


def pouch(d, body, body_d, opened, rng):
    """Freeze-dried meal pouch, sealed or opened."""
    d.rounded_rectangle([8, 8, 24, 29], radius=3, fill=body)
    if opened:
        d.ellipse([9, 6, 23, 12], fill=(40, 32, 26, 255))
        d.rectangle([8, 8, 24, 10], fill=body_d)
    else:
        d.rectangle([8, 5, 24, 9], fill=body_d)
        for x in range(9, 24, 2):
            d.point((x, 6), fill=body)
            d.point((x, 8), fill=body)
    d.rounded_rectangle([8, 8, 24, 29], radius=3, outline=body_d)
    d.line([(11, 12), (11, 26)], fill=BAG_L, width=1)


def label_band(d, y0=16, y1=24):
    d.rectangle([9, y0, 23, y1], fill=LABEL)
    d.rectangle([9, y0, 23, y1], outline=LABEL_D)


def draw_amatriciana(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pan(d)
    noodles(d, rng, SAUCE_RED, SAUCE_RED_D, NOODLE, NOODLE_D)
    bacon_bits(d, rng, 8)
    for _ in range(9):  # pecorino shavings
        x = rng.randrange(8, 22)
        y = rng.randrange(11, 24)
        d.point((x, y), fill=PECORINO)
    return img


def draw_carbonara(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pan(d)
    noodles(d, rng, CREAM, CREAM_D, NOODLE, NOODLE_D)
    bacon_bits(d, rng, 8)
    for _ in range(12):  # cracked pepper
        x = rng.randrange(8, 22)
        y = rng.randrange(11, 24)
        d.point((x, y), fill=PEPPER)
    return img


def draw_buffalo_wings(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # wings heaped above the rim first
    for (x, y, a) in [(11, 11, -20), (17, 9, 10), (21, 12, 30),
                      (14, 14, -35), (19, 15, 15)]:
        d.ellipse([x - 4, y - 3, x + 4, y + 3], fill=WING)
        d.ellipse([x - 3, y - 2, x + 1, y + 1], fill=WING_L)
        d.point((x + 4, y), fill=BONE)
        d.point((x + 4, y - 1), fill=BONE)
    carton(d)
    return img


def draw_fchicken(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # drumstick leaning out
    d.line([(20, 6), (17, 12)], fill=BONE, width=2)
    d.point((21, 5), fill=BONE)
    d.point((19, 5), fill=BONE)
    chunk(d, 16, 13, 5.2, CRISP, rng)
    chunk(d, 16, 12, 3.4, CRISP_L, rng)
    chunk(d, 11, 14, 4.4, CRISP, rng)
    chunk(d, 22, 15, 4.0, CRISP_D, rng)
    for _ in range(14):
        x = rng.randrange(7, 25)
        y = rng.randrange(9, 18)
        d.point((x, y), fill=CRISP_D)
    carton(d)
    return img


def draw_fries(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for (x, y, dx) in [(10, 5, -1), (13, 4, 0), (16, 5, 1),
                       (19, 4, 0), (22, 6, 1), (12, 8, -1),
                       (18, 8, 1), (15, 7, 0), (21, 9, 0)]:
        d.line([(x, y), (x + dx, y + 10)], fill=FRY, width=3)
        d.line([(x, y), (x + dx, y + 10)], fill=FRY_D, width=1)
    for _ in range(6):  # salt
        d.point((rng.randrange(9, 24), rng.randrange(5, 13)),
                fill=WHITE)
    carton(d)
    return img


def draw_poutine(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # fry base, a loose mound
    for (x, y, dx) in [(8, 20, -2), (11, 17, -1), (14, 16, 0),
                       (17, 16, 1), (20, 17, 2), (23, 20, 2),
                       (10, 23, -1), (15, 22, 0), (20, 23, 1),
                       (12, 25, 0), (18, 25, 0)]:
        d.line([(x, y), (x + dx, y + 5)], fill=FRY, width=3)
    # gravy poured over
    chunk(d, 13, 18, 5.0, GRAVY, rng)
    chunk(d, 19, 19, 5.4, GRAVY, rng)
    chunk(d, 16, 22, 5.0, GRAVY_D, rng)
    d.line([(10, 20), (10, 25)], fill=GRAVY, width=2)
    d.line([(22, 21), (22, 26)], fill=GRAVY, width=2)
    # cheese curds on top
    for (x, y) in [(12, 16), (17, 15), (21, 18), (15, 20),
                   (20, 22), (11, 21)]:
        chunk(d, x, y, 2.2, CURD, rng, squash=0.8)
    return img


def draw_onion_rings(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for (x, y, w) in [(16, 24, 11), (15, 19, 10), (17, 14, 9),
                      (16, 9, 8)]:
        d.ellipse([x - w // 2, y - 3, x + w // 2, y + 3],
                  outline=RING, width=3)
        d.ellipse([x - w // 2, y - 3, x + w // 2, y + 3],
                  outline=RING_D, width=1)
    for _ in range(8):
        d.point((rng.randrange(9, 24), rng.randrange(6, 27)),
                fill=RING_D)
    return img


def _fd_sealed(rng, motif):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pouch(d, POUCH_R, POUCH_R_D, False, rng)
    label_band(d)
    motif(d)
    return img


def _fd_open(rng, food):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    food(d, rng)  # mound behind the pouch rim
    pouch(d, POUCH_B, POUCH_B_D, True, rng)
    return img


def draw_fd_meal(rng):
    def motif(d):
        chunk(d, 13, 20, 2.8, (128, 82, 48, 255), rng, squash=0.8)
        d.ellipse([17, 18, 21, 22], fill=(240, 196, 64, 255))
        d.point((19, 19), fill=(250, 226, 140, 255))
    return _fd_sealed(rng, motif)


def draw_fd_meal_hyd(rng):
    def food(d, rng):
        chunk(d, 13, 7, 4.2, (128, 82, 48, 255), rng, squash=0.7)
        chunk(d, 19, 7, 4.4, (146, 96, 56, 255), rng, squash=0.7)
        chunk(d, 16, 6, 3.6, (240, 196, 64, 255), rng, squash=0.7)
        d.point((14, 6), fill=(250, 226, 140, 255))
        d.point((19, 5), fill=(250, 226, 140, 255))
    return _fd_open(rng, food)


def draw_fd_stroganoff(rng):
    def motif(d):
        d.arc([11, 17, 17, 23], 20, 300, fill=CREAM_D, width=2)
        d.arc([15, 17, 21, 23], 200, 120, fill=NOODLE_D, width=1)
        d.rectangle([18, 18, 21, 20], fill=(128, 82, 48, 255))
    return _fd_sealed(rng, motif)


def draw_fd_stroganoff_hyd(rng):
    def food(d, rng):
        chunk(d, 13, 7, 4.4, CREAM, rng, squash=0.7)
        chunk(d, 19, 7, 4.2, CREAM_D, rng, squash=0.7)
        chunk(d, 16, 6, 3.8, CREAM, rng, squash=0.7)
        d.line([(11, 6), (15, 8)], fill=(128, 82, 48, 255),
               width=2)
        d.line([(18, 5), (21, 7)], fill=(110, 70, 40, 255),
               width=2)
    return _fd_open(rng, food)


def draw_fd_veggypasta(rng):
    def motif(d):
        d.arc([11, 17, 16, 22], 0, 300, fill=NOODLE_D, width=1)
        for (x, y, c) in [(18, 18, (84, 150, 60, 255)),
                          (20, 21, (226, 126, 36, 255)),
                          (13, 21, (84, 150, 60, 255))]:
            d.point((x, y), fill=c)
            d.point((x + 1, y), fill=c)
    return _fd_sealed(rng, motif)


def draw_fd_veggypasta_hyd(rng):
    def food(d, rng):
        chunk(d, 13, 7, 4.2, CREAM, rng, squash=0.7)
        chunk(d, 19, 7, 4.4, CREAM_D, rng, squash=0.7)
        for (x, y, c) in [(12, 6, (84, 150, 60, 255)),
                          (17, 5, (226, 126, 36, 255)),
                          (20, 7, (84, 150, 60, 255)),
                          (15, 8, (206, 52, 40, 255))]:
            d.point((x, y), fill=c)
            d.point((x + 1, y), fill=c)
    return _fd_open(rng, food)


def draw_frozen_dinner(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    TRAY = (70, 66, 72, 255)
    TRAY_D = (44, 42, 46, 255)
    FROST = (214, 230, 238, 255)
    d.rounded_rectangle([4, 9, 28, 26], radius=3, fill=TRAY)
    d.rounded_rectangle([4, 9, 28, 26], radius=3, outline=TRAY_D)
    # meat compartment
    d.rectangle([6, 12, 17, 23], fill=(104, 66, 42, 255))
    d.rectangle([7, 13, 12, 16], fill=(128, 86, 54, 255))
    # carb compartment
    d.rectangle([19, 12, 26, 23], fill=(216, 188, 130, 255))
    d.rectangle([20, 13, 24, 17], fill=(232, 206, 156, 255))
    d.line([(6, 12), (26, 12)], fill=TRAY_D)
    # uncooked frost over everything
    for _ in range(26):
        d.point((rng.randrange(5, 28), rng.randrange(10, 26)),
                fill=FROST)
    d.line([(6, 10), (14, 10)], fill=FROST, width=1)
    return img


def draw_candy(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        for (x, y) in [(12, 15), (18, 14), (15, 20), (20, 21),
                       (12, 24)]:
            dd.ellipse([x - 3, y - 3, x + 3, y + 3], fill=CHOC)
            dd.ellipse([x - 2, y - 2, x + 2, y + 1], fill=MILKCHOC)
    small_bag(d, contents)
    # three peanut butter cups in front of the bag
    for (x, y) in [(10, 27), (17, 28), (23, 26)]:
        for i in range(4):
            d.line([(x - 3 + i * 2, y + 2), (x - 3 + i * 2, y + 4)],
                   fill=CHOC_D)
        d.ellipse([x - 4, y - 2, x + 4, y + 3], fill=CHOC)
        d.ellipse([x - 3, y - 2, x + 3, y + 1], fill=MILKCHOC)
    return img


def _candy_pile(rng, draw_one, bag_spots, front_spots):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        for i, (x, y) in enumerate(bag_spots):
            draw_one(dd, x, y, CANDYcols[i % len(CANDYcols)], rng)
    small_bag(d, contents)
    for i, (x, y) in enumerate(front_spots):
        draw_one(d, x, y, CANDYcols[(i + 2) % len(CANDYcols)],
                 rng)
    return img

BAG_SPOTS = [(12, 15), (19, 14), (15, 20), (20, 22)]


def _oval_candy(dd, x, y, c, rng):
    dd.ellipse([x - 3, y - 2, x + 3, y + 2], fill=c)
    dd.point((x - 1, y - 1), fill=WHITE)


def draw_candy2(rng):
    front = [(9, 27), (16, 28), (23, 27), (12, 24), (20, 24)]
    return _candy_pile(rng, _oval_candy, BAG_SPOTS, front)


def _chew(dd, x, y, c, rng):
    dd.rounded_rectangle([x - 3, y - 2, x + 3, y + 2], radius=2,
                         fill=c)
    dd.line([(x - 2, y - 1), (x + 1, y - 1)], fill=WHITE)


def draw_candy3(rng):
    front = [(9, 27), (16, 28), (23, 27), (12, 24), (20, 24)]
    return _candy_pile(rng, _chew, BAG_SPOTS, front)


def _gator(dd, x, y, c, rng):
    # chunky little alligator: tapered tail, long snout
    body = [(x - 9, y + 1), (x - 5, y - 1), (x - 1, y - 2),
            (x + 4, y - 2), (x + 8, y - 1), (x + 9, y),
            (x + 8, y + 2), (x + 3, y + 2), (x - 2, y + 3),
            (x - 6, y + 3)]
    dd.polygon(body, fill=c)
    dd.point((x + 6, y - 1), fill=INK)          # eye
    dd.point((x + 8, y + 1), fill=INK)          # nostril
    for lx in (-5, 3):                          # legs
        dd.rectangle([x + lx, y + 3, x + lx + 1, y + 4], fill=c)
    for sx in (-3, 0, 3):                       # back scutes
        dd.point((x + sx, y - 2), fill=INK)


def draw_candy3gator(rng):
    front = [(10, 26), (21, 27), (16, 22)]
    return _candy_pile(rng, _gator, BAG_SPOTS, front)


def _gummy_bear(dd, x, y, c, rng):
    dd.ellipse([x - 3, y - 6, x - 1, y - 4], fill=c)   # ears
    dd.ellipse([x + 1, y - 6, x + 3, y - 4], fill=c)
    dd.ellipse([x - 3, y - 5, x + 3, y], fill=c)       # head
    dd.ellipse([x - 3, y, x + 3, y + 6], fill=c)       # body
    dd.ellipse([x - 5, y + 1, x - 3, y + 4], fill=c)   # arms
    dd.ellipse([x + 3, y + 1, x + 5, y + 4], fill=c)
    dd.ellipse([x - 1, y + 1, x + 1, y + 4], fill=WHITE)
    dd.point((x, y - 3), fill=INK)


def draw_candy4(rng):
    front = [(9, 24), (17, 25), (24, 23)]
    return _candy_pile(rng, _gummy_bear, BAG_SPOTS, front)


def _gummy_worm(dd, x, y, c, rng):
    pale = tuple(min(255, v + 46) for v in c[:3]) + (255,)
    for i in range(4):
        xx = x - 4 + i * 3
        yy = y + (1 if i % 2 else -1)
        dd.ellipse([xx - 2, yy - 2, xx + 2, yy + 2], fill=pale)
    dd.point((x - 3, y - 2), fill=WHITE)


def draw_candy5(rng):
    front = [(9, 27), (16, 28), (23, 27), (12, 24), (20, 24)]
    return _candy_pile(rng, _gummy_worm, BAG_SPOTS, front)


def draw_candy_coated_peanuts(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # snack box
    d.polygon([(8, 14), (24, 14), (22, 27), (10, 27)],
              fill=(188, 84, 44, 255))
    d.polygon([(8, 14), (24, 14), (22, 27), (10, 27)],
              outline=(140, 56, 28, 255))
    d.rectangle([7, 12, 25, 15], fill=(214, 120, 66, 255))
    # coated peanuts mounded and spilling
    spots = [(11, 11), (16, 9), (21, 11), (13, 14), (19, 14),
             (9, 26), (24, 25), (16, 28)]
    shell = [(208, 52, 44, 255), (240, 196, 52, 255),
             (140, 88, 52, 255), (240, 240, 236, 255),
             (52, 108, 196, 255)]
    for i, (x, y) in enumerate(spots):
        c = shell[i % len(shell)]
        d.ellipse([x - 3, y - 2, x + 3, y + 2], fill=c)
        d.point((x - 1, y - 1), fill=WHITE)
    return img


def draw_candycigarette(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # sticks poking out of the pack
    for x in (12, 16, 20):
        d.rectangle([x, 5, x + 2, 13], fill=WHITE)
        d.rectangle([x, 5, x + 2, 7], fill=(222, 196, 150, 255))
    # cigarette pack
    d.rectangle([9, 12, 23, 29], fill=BOXW)
    d.rectangle([9, 12, 23, 29], outline=BOXW_D)
    d.rectangle([9, 12, 23, 16], fill=(198, 52, 44, 255))
    d.rectangle([9, 21, 23, 23], fill=(198, 52, 44, 255))
    d.ellipse([14, 17, 18, 20], outline=BOXW_D)
    return img


def draw_caramel_apple(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.line([(16, 3), (17, 10)], fill=STICK, width=2)
    d.ellipse([8, 10, 24, 29], fill=APPLE)
    d.ellipse([8, 10, 24, 29], outline=APPLE_D)
    # caramel coating over the top, dripping down
    d.ellipse([8, 10, 24, 24], fill=CARAMEL)
    for (x, ln) in [(10, 5), (14, 8), (19, 6), (22, 4)]:
        d.line([(x, 20), (x, 20 + ln)], fill=CARAMEL, width=2)
    d.ellipse([10, 11, 17, 15], fill=CARAMEL_L)
    d.point((13, 12), fill=WHITE)
    return img


def draw_choc_pancakes(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for (y, w) in [(22, 24), (17, 22), (12, 20)]:
        d.ellipse([16 - w // 2, y, 16 + w // 2, y + 6], fill=CHOC_L)
        d.ellipse([16 - w // 2, y, 16 + w // 2, y + 6],
                  outline=CHOC)
    # syrup drizzle + butter
    d.line([(10, 13), (22, 13)], fill=SYRUP, width=2)
    d.line([(12, 16), (12, 20)], fill=SYRUP, width=2)
    d.line([(20, 15), (20, 19)], fill=SYRUP, width=2)
    d.rectangle([14, 8, 19, 11], fill=BUTTER)
    d.rectangle([14, 8, 19, 11], outline=(214, 178, 70, 255))
    return img


def draw_choc_waffles(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for (ox, oy) in [(7, 15), (6, 8)]:
        d.rectangle([ox, oy, ox + 19, oy + 12], fill=CHOC_L)
        d.rectangle([ox, oy, ox + 19, oy + 12], outline=CHOC)
        for gx in range(ox + 4, ox + 18, 5):
            d.line([(gx, oy + 1), (gx, oy + 11)], fill=CHOC,
                   width=1)
        for gy in range(oy + 3, oy + 11, 4):
            d.line([(ox + 1, gy), (ox + 18, gy)], fill=CHOC,
                   width=1)
    d.line([(8, 10), (24, 10)], fill=SYRUP, width=2)
    d.rectangle([14, 5, 19, 8], fill=BUTTER)
    d.rectangle([14, 5, 19, 8], outline=(214, 178, 70, 255))
    return img


def draw_chocolate(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # bar, top half bare segments
    d.rectangle([9, 6, 23, 20], fill=CHOC)
    d.rectangle([9, 6, 23, 20], outline=CHOC_D)
    for gx in (13, 18):
        d.line([(gx, 7), (gx, 19)], fill=CHOC_D)
    d.line([(10, 13), (22, 13)], fill=CHOC_D)
    d.rectangle([10, 7, 12, 9], fill=CHOC_L)
    # wrapper over the bottom half
    d.rectangle([8, 18, 24, 28], fill=(198, 52, 44, 255))
    d.rectangle([8, 18, 24, 28], outline=(140, 32, 28, 255))
    d.rectangle([8, 18, 24, 20], fill=(216, 220, 224, 255))
    d.rectangle([11, 22, 21, 25], fill=WHITE)
    return img


def draw_chocolate_military(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # thick dense slab, two segments, no wrapper
    d.rectangle([8, 9, 24, 24], fill=CHOC_D)
    d.rectangle([8, 9, 24, 24], outline=(40, 22, 12, 255))
    d.rectangle([9, 10, 15, 17], fill=CHOC)
    d.rectangle([17, 10, 23, 17], fill=CHOC)
    d.rectangle([9, 18, 15, 23], fill=CHOC)
    d.rectangle([17, 18, 23, 23], fill=CHOC)
    d.line([(8, 24), (24, 24)], fill=(40, 22, 12, 255), width=2)
    return img


def _pretzel(dd, cx, cy, fill, dark, salt):
    dd.ellipse([cx - 6, cy - 6, cx + 6, cy + 5], outline=fill,
               width=3)
    dd.line([(cx - 4, cy - 2), (cx + 2, cy + 5)], fill=fill,
            width=3)
    dd.line([(cx + 4, cy - 2), (cx - 2, cy + 5)], fill=fill,
            width=3)
    if salt:
        for (x, y) in [(-3, -5), (2, -4), (-1, 1), (4, 2),
                       (-5, 0)]:
            dd.point((cx + x, cy + y), fill=WHITE)


def draw_pretzels(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        _pretzel(dd, 16, 19, (186, 138, 70, 255), GRAHAM_D, True)
    small_bag(d, contents)
    _pretzel(d, 11, 25, (196, 148, 78, 255), GRAHAM_D, True)
    _pretzel(d, 22, 24, (186, 138, 70, 255), GRAHAM_D, True)
    return img


def draw_chocpretzels(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        _pretzel(dd, 16, 19, CHOC, CHOC_D, False)
    small_bag(d, contents)
    _pretzel(d, 11, 25, CHOC_L, CHOC_D, False)
    _pretzel(d, 22, 24, CHOC, CHOC_D, False)
    return img


def draw_cookies_egg(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([9, 6, 23, 27], fill=COOKIE)
    d.ellipse([9, 6, 23, 27], outline=COOKIE_D)
    # pastel icing cap with a zigzag edge
    d.pieslice([9, 6, 23, 20], 0, 180, fill=(246, 196, 208, 255))
    for x in range(10, 22, 3):
        d.point((x, 16), fill=(246, 196, 208, 255))
        d.point((x, 17), fill=(246, 196, 208, 255))
    for (x, y, c) in [(12, 10, (52, 108, 196, 255)),
                      (17, 9, (240, 196, 52, 255)),
                      (20, 12, (66, 160, 66, 255)),
                      (14, 13, (232, 128, 40, 255))]:
        d.point((x, y), fill=c)
        d.point((x + 1, y), fill=c)
    d.point((12, 22), fill=COOKIE_D)
    d.point((19, 24), fill=COOKIE_D)
    return img


def _cow(dd, x, y, c, rng):
    dd.ellipse([x - 6, y - 3, x + 4, y + 3], fill=c)      # body
    dd.ellipse([x + 3, y - 4, x + 8, y + 1], fill=c)      # head
    dd.point((x + 7, y - 2), fill=INK)
    for lx in (-5, -2, 2):
        dd.rectangle([x + lx, y + 3, x + lx + 1, y + 6],
                     fill=c)
    dd.line([(x - 6, y - 2), (x - 8, y + 2)], fill=c, width=1)
    dd.ellipse([x - 3, y - 2, x, y + 1], fill=CHOC_D)    # patch


def draw_cow_candy(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        _cow(dd, 16, 19, MILKCHOC, rng)
    small_bag(d, contents)
    _cow(d, 12, 24, MILKCHOC, rng)
    _cow(d, 23, 23, CHOC_L, rng)
    return img


def draw_debug_calories(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # halo first (behind the cake top)
    d.ellipse([11, 3, 21, 7], outline=(240, 200, 72, 255),
              width=2)
    # angel food ring cake
    d.ellipse([7, 12, 25, 28], fill=WHITE)
    d.ellipse([7, 12, 25, 28], outline=(206, 192, 168, 255))
    d.ellipse([7, 12, 25, 19], fill=(252, 250, 244, 255))
    d.ellipse([13, 14, 19, 20], fill=(226, 214, 192, 255))
    d.ellipse([13, 14, 19, 20], outline=(206, 192, 168, 255))
    for (x, y) in [(9, 22), (22, 23), (16, 26), (11, 26)]:
        d.point((x, y), fill=(232, 220, 196, 255))
    for (x, y) in [(6, 9), (26, 10), (27, 18)]:
        d.point((x, y), fill=(240, 200, 72, 255))
    return img


def draw_buttercream(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        dd.ellipse([10, 13, 22, 28], fill=(244, 238, 226, 255))
        dd.ellipse([12, 16, 18, 22], fill=WHITE)
    small_bag(d, contents)
    # a piped swirl dollop on top of the bag
    d.ellipse([11, 4, 21, 9], fill=(244, 238, 226, 255))
    d.ellipse([13, 2, 19, 6], fill=WHITE)
    d.point((16, 1), fill=(244, 238, 226, 255))
    return img


def draw_gelatin_powder(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rectangle([9, 8, 23, 26], fill=BOXW)
    d.rectangle([9, 8, 23, 26], outline=BOXW_D)
    d.rectangle([9, 8, 23, 12], fill=FROST_RED)
    d.ellipse([12, 15, 20, 23], fill=FROST_RED)
    d.ellipse([14, 17, 18, 21], fill=(240, 150, 140, 255))
    # a little pile of the powder spilled in front
    chunk(d, 16, 28, 4.6, (240, 170, 160, 255), rng, squash=0.45)
    return img


def draw_grahmcrackers(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for (ox, oy) in [(13, 9), (6, 13)]:
        d.rectangle([ox, oy, ox + 13, oy + 15], fill=GRAHAM)
        d.rectangle([ox, oy, ox + 13, oy + 15], outline=GRAHAM_D)
        d.line([(ox + 6, oy + 1), (ox + 6, oy + 14)],
               fill=GRAHAM_D)
        d.line([(ox + 1, oy + 7), (ox + 12, oy + 7)],
               fill=GRAHAM_D)
        for gx in (ox + 3, ox + 9):
            for gy in (oy + 3, oy + 11):
                d.point((gx, gy), fill=GRAHAM_D)
    return img


def draw_grahmcrackers_roe(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.polygon([(8, 13), (24, 13), (22, 26), (10, 26)],
              fill=(188, 84, 44, 255))
    d.polygon([(8, 13), (24, 13), (22, 26), (10, 26)],
              outline=(140, 56, 28, 255))
    for _ in range(11):
        x = rng.randrange(9, 23)
        y = rng.randrange(10, 25)
        r = rng.uniform(1.8, 3.0)
        chunk(d, x, y, r, GRAHAM, rng, squash=0.8)
        d.point((x, y), fill=GRAHAM_D)
    return img


def draw_maltballs(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.polygon([(8, 15), (24, 15), (22, 27), (10, 27)],
              fill=(188, 84, 44, 255))
    d.polygon([(8, 15), (24, 15), (22, 27), (10, 27)],
              outline=(140, 56, 28, 255))
    for (x, y) in [(11, 12), (16, 10), (21, 12), (13, 15),
                   (19, 15), (9, 25), (24, 24), (16, 27)]:
        d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=CHOC)
        d.point((x - 1, y - 2), fill=CHOC_L)
        d.point((x - 1, y - 1), fill=WHITE)
    return img


def draw_marshmallow_fluff(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rectangle([10, 4, 22, 8], fill=(198, 52, 44, 255))
    for x in range(11, 22, 2):
        d.line([(x, 4), (x, 8)], fill=(158, 36, 32, 255))
    d.rounded_rectangle([9, 8, 23, 29], radius=3, fill=GLASS)
    d.rounded_rectangle([11, 12, 21, 27], radius=2,
                        fill=(248, 248, 246, 255))
    d.ellipse([12, 13, 19, 18], fill=WHITE)
    d.arc([12, 20, 20, 26], 300, 140, fill=(232, 230, 224, 255),
          width=2)
    d.rounded_rectangle([9, 8, 23, 29], radius=3, outline=GLASS_D)
    d.line([(11, 10), (11, 25)], fill=(242, 250, 244, 255))
    return img


def draw_mintpatties(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    def contents(dd):
        for (x, y) in [(13, 16), (19, 15), (16, 21)]:
            dd.ellipse([x - 4, y - 3, x + 4, y + 3], fill=CHOC)
    small_bag(d, contents)
    # patties in front, one showing its white mint layer
    for (x, y) in [(10, 26), (17, 27), (23, 25)]:
        d.ellipse([x - 4, y - 3, x + 4, y + 3], fill=CHOC)
        d.ellipse([x - 3, y - 2, x + 2, y], fill=CHOC_L)
    d.rectangle([14, 26, 20, 27], fill=WHITE)
    d.rectangle([14, 25, 20, 26], fill=CHOC)
    return img


def draw_neccowafers(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # roll of stacked wafers
    for i in range(7):
        y = 24 - i * 3
        c = PASTEL[i % len(PASTEL)]
        d.ellipse([10, y - 2, 20, y + 3], fill=c)
        d.ellipse([10, y - 2, 20, y + 3],
                  outline=(180, 168, 150, 255))
    # two loose wafers leaning at the base
    d.ellipse([19, 21, 27, 28], fill=PASTEL[3])
    d.ellipse([19, 21, 27, 28], outline=(180, 168, 150, 255))
    d.ellipse([4, 22, 11, 29], fill=PASTEL[1])
    d.ellipse([4, 22, 11, 29], outline=(180, 168, 150, 255))
    return img


def draw_porkstick(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # wrapper sleeve at the lower end
    d.polygon([(8, 24), (14, 18), (19, 23), (13, 29)],
              fill=(216, 220, 224, 255))
    d.polygon([(8, 24), (14, 18), (19, 23), (13, 29)],
              outline=BAG_D)
    # the dried pork stick, diagonal
    d.line([(13, 25), (24, 8)], fill=(150, 74, 44, 255), width=5)
    d.line([(13, 25), (24, 8)], fill=(178, 102, 62, 255),
           width=2)
    for (x, y) in [(20, 13), (17, 18), (22, 10)]:
        d.point((x, y), fill=(120, 54, 32, 255))
    d.ellipse([22, 5, 27, 10], fill=(178, 102, 62, 255))
    return img


def draw_smores(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # bottom graham
    d.rectangle([7, 21, 25, 26], fill=GRAHAM)
    d.rectangle([7, 21, 25, 26], outline=GRAHAM_D)
    # chocolate layer
    d.rectangle([8, 18, 24, 21], fill=CHOC)
    # marshmallow bulging out
    d.rounded_rectangle([9, 12, 23, 19], radius=3, fill=WHITE)
    d.point((12, 14), fill=(240, 226, 196, 255))
    d.point((19, 13), fill=(240, 226, 196, 255))
    # top graham, tilted
    d.polygon([(7, 12), (24, 8), (25, 13), (8, 17)],
              fill=GRAHAM)
    d.polygon([(7, 12), (24, 8), (25, 13), (8, 17)],
              outline=GRAHAM_D)
    for (x, y) in [(12, 12), (18, 10)]:
        d.point((x, y), fill=GRAHAM_D)
    return img


def draw_sugar_fried(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # glossy caramel mound with drips
    chunk(d, 16, 20, 8.0, CARAMEL, rng, squash=0.62)
    chunk(d, 12, 22, 5.0, CARAMEL_D, rng, squash=0.55)
    chunk(d, 20, 22, 5.0, CARAMEL, rng, squash=0.55)
    d.line([(9, 24), (9, 27)], fill=CARAMEL_D, width=2)
    d.line([(23, 24), (23, 26)], fill=CARAMEL, width=2)
    d.arc([11, 14, 20, 21], 180, 320, fill=CARAMEL_L, width=2)
    d.point((13, 16), fill=WHITE)
    d.point((14, 15), fill=WHITE)
    return img


def _syrup_bottle(rng, liquid, liquid_d):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rectangle([14, 3, 18, 6], fill=(120, 84, 44, 255))
    d.polygon([(13, 6), (19, 6), (21, 11), (21, 27), (11, 27),
               (11, 11)], fill=GLASS)
    d.polygon([(12, 12), (20, 12), (20, 26), (12, 26)],
              fill=liquid)
    d.line([(13, 13), (13, 25)], fill=liquid_d, width=1)
    d.polygon([(13, 6), (19, 6), (21, 11), (21, 27), (11, 27),
               (11, 11)], outline=GLASS_D)
    d.line([(18, 8), (19, 12)], fill=(242, 250, 244, 255))
    d.line([(19, 14), (19, 24)], fill=(242, 250, 244, 255))
    return img


def draw_syrup_corn(rng):
    return _syrup_bottle(rng, (238, 196, 84, 255),
                         (214, 164, 52, 255))


def draw_syrup_simple(rng):
    return _syrup_bottle(rng, (216, 224, 226, 255),
                         (188, 200, 204, 255))


def _toaster_pastry(rng, frozen):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    crust = PASTRY if not frozen else (226, 204, 160, 255)
    frost = FROST_RED if not frozen else (226, 130, 140, 255)
    d.rounded_rectangle([7, 9, 25, 25], radius=3, fill=crust)
    d.rounded_rectangle([7, 9, 25, 25], radius=3,
                        outline=PASTRY_D)
    d.rounded_rectangle([9, 11, 23, 23], radius=2, fill=frost)
    sprinkles = CANDYcols if not frozen else PASTEL
    for i in range(8):
        x = rng.randrange(11, 22)
        y = rng.randrange(13, 22)
        d.point((x, y), fill=sprinkles[i % len(sprinkles)])
        d.point((x + 1, y), fill=sprinkles[i % len(sprinkles)])
    if frozen:
        for _ in range(16):
            d.point((rng.randrange(8, 25), rng.randrange(10, 25)),
                    fill=(236, 244, 248, 255))
    return img


def draw_toastem(rng):
    return _toaster_pastry(rng, False)


def draw_toasterpastryfrozen(rng):
    return _toaster_pastry(rng, True)


RENDER = {
    "amatriciana_pasta": draw_amatriciana,
    "buffalo_wings": draw_buffalo_wings,
    "buttercream": draw_buttercream,
    "candy": draw_candy,
    "candy2": draw_candy2,
    "candy3": draw_candy3,
    "candy3gator": draw_candy3gator,
    "candy4": draw_candy4,
    "candy5": draw_candy5,
    "candy_coated_peanuts": draw_candy_coated_peanuts,
    "candycigarette": draw_candycigarette,
    "caramel_apple": draw_caramel_apple,
    "choc_pancakes": draw_choc_pancakes,
    "choc_waffles": draw_choc_waffles,
    "chocolate": draw_chocolate,
    "chocolate_military": draw_chocolate_military,
    "chocpretzels": draw_chocpretzels,
    "cookies_egg": draw_cookies_egg,
    "cow_candy": draw_cow_candy,
    "debug_calories": draw_debug_calories,
    "fchicken": draw_fchicken,
    "freeze_dried_meal": draw_fd_meal,
    "freeze_dried_meal_hydrated": draw_fd_meal_hyd,
    "freeze_dried_stroganoff": draw_fd_stroganoff,
    "freeze_dried_stroganoff_hydrated": draw_fd_stroganoff_hyd,
    "freeze_dried_veggypasta": draw_fd_veggypasta,
    "freeze_dried_veggypasta_hydrated": draw_fd_veggypasta_hyd,
    "fries": draw_fries,
    "frozen_dinner": draw_frozen_dinner,
    "gelatin_dessert_powder": draw_gelatin_powder,
    "grahmcrackers": draw_grahmcrackers,
    "grahmcrackers_roe": draw_grahmcrackers_roe,
    "maltballs": draw_maltballs,
    "marshmallow_fluff": draw_marshmallow_fluff,
    "mintpatties": draw_mintpatties,
    "neccowafers": draw_neccowafers,
    "onion_rings": draw_onion_rings,
    "pasta_carbonara": draw_carbonara,
    "porkstick": draw_porkstick,
    "poutine": draw_poutine,
    "pretzels": draw_pretzels,
    "smores": draw_smores,
    "sugar_fried": draw_sugar_fried,
    "syrup_corn": draw_syrup_corn,
    "syrup_simple": draw_syrup_simple,
    "toastem": draw_toastem,
    "toasterpastryfrozen": draw_toasterpastryfrozen,
}


def main():
    assert set(RENDER) == set(IDS), (
        set(IDS) ^ set(RENDER))
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = RENDER[item_id](rng)
        assert img.size == (32, 32), item_id
        assert set(img.getchannel("A").getdata()) <= {0, 255}, \
            item_id
        imgs[item_id] = img
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"),
                  "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}],
                      f, indent=2)
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
                       "preview_item31.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item31.png "
          f"{sheet.size}")


if __name__ == "__main__":
    main()
