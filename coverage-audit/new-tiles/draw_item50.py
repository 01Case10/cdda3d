#!/usr/bin/env python3
"""ITEM50: containers -- CDDA Voxel coverage audit.

All 29 items come from items/containers/containers.json.  Each
container is drawn as itself, sized by its stated capacity (bigger
volume = bigger in frame) and coloured by its named material:

  aquariums (glass, light cyan): empty glass tanks with dark
    silicone seams and fresh silicone beads at the corners -- they
    are the *drying* variants, so no water, no fish.
    small 30 gal / medium 50 gal / large 100 gal.
  bottles: glass 330 ml (cyan glass, crown cap), plastic 355 ml
    (ridged water bottle, blue cap), glue bottles 500 ml / 250 ml
    (white plastic, pointed orange nozzle, glue label band).
  can_aluminum_250ml: silver soda can with a light-blue band.
  cartons (paper laminate, threaded cap): gable-top cartons,
    half pint 284 ml small, quart 946 ml large; carton_egg_6 is a
    flat brown cardboard box with 3x2 egg domes on the lid.
  clay: clay_cup a small terracotta cup; clay_watercont_no_lid a
    bulky wide-bellied terracotta pot with an open dark mouth
    (no lid).
  jug_plastic_1892ml: half-gallon milk jug, handle + cap.
  biogas tanks (steel, propane-tank form: capsule body, collar,
    valve, base ring): small 5 L and medium 23 L green, large
    80 L and xl 390 L light gray, growing to fill the frame.
  pillboxes (white plastic): flat weekly organizer, 7 day cells;
    pillbox_large has two weeks = 2 rows of 7.
  pressurized_canister: small green steel gas canister, valve top.
  six_pack_rings: 3x2 grid of thin light-cyan plastic rings.
  small_squeeze_tube: tiny white glue tube, crimped tail.
  spools (dark gray plastic flanges): tiny (thread), small (wire),
    welding (large, copper-coloured welding wire wound on).
  tube_welding_rod: long clear canister, welding rods visible
    inside, threaded cap.
  wrapper_bar: flat paper snack wrapper, crimped ends.
  wrapper_foil_roll: cardboard tube with a silver foil sheet
    unrolling from it.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "aquarium_large_drying_active",
    "aquarium_medium_drying_active",
    "aquarium_small_drying_active",
    "bottle_glass_330ml",
    "bottle_plastic_355ml",
    "bottle_plastic_glue",
    "bottle_plastic_small_glue",
    "can_aluminum_250ml",
    "carton_284ml",
    "carton_946ml",
    "carton_egg_6",
    "clay_cup",
    "clay_watercont_no_lid",
    "jug_plastic_1892ml",
    "large_biogas_tank",
    "medium_biogas_tank",
    "pillbox",
    "pillbox_large",
    "pressurized_canister",
    "six_pack_rings",
    "small_biogas_tank",
    "small_squeeze_tube",
    "spool_small",
    "spool_tiny",
    "spool_welding",
    "tube_welding_rod",
    "wrapper_bar",
    "wrapper_foil_roll",
    "xl_biogas_tank",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

GLASS = (178, 224, 230, 255)
GLASS_DARK = (96, 148, 158, 255)
GLASS_PALE = (214, 240, 242, 255)
SILICONE = (52, 54, 58, 255)
PLASTIC_CYAN = (168, 216, 226, 255)
PLASTIC_WHITE = (238, 238, 236, 255)
PLASTIC_DARK = (88, 90, 96, 255)
CAP_BLUE = (52, 96, 188, 255)
CAP_ORANGE = (232, 116, 36, 255)
ALUMINUM = (196, 200, 206, 255)
ALUMINUM_DARK = (128, 132, 140, 255)
CARTON = (226, 232, 236, 255)
CARTON_SHADE = (178, 192, 200, 255)
CARDBOARD = (168, 128, 82, 255)
CARDBOARD_DARK = (122, 90, 56, 255)
CLAY = (178, 102, 66, 255)
CLAY_DARK = (128, 68, 42, 255)
CLAY_LIGHT = (208, 138, 96, 255)
GREEN_TANK = (74, 148, 84, 255)
GREEN_DARK = (46, 100, 56, 255)
GRAY_TANK = (172, 176, 182, 255)
GRAY_DARK = (110, 114, 120, 255)
STEEL = (150, 154, 162, 255)
COPPER = (196, 124, 74, 255)
THREAD = (226, 220, 200, 255)
FOIL = (208, 212, 218, 255)
INK = (40, 40, 44, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def speckle(d, rng, box, col, n=6):
    x0, y0, x1, y1 = box
    for _ in range(n):
        d.point((rng.randint(x0, x1), rng.randint(y0, y1)), fill=col)


# ---------------------------------------------------------------- aquarium

def aquarium(d, rng, w, h):
    """Empty glass tank, silicone seams, fresh beads at the corners."""
    x0, y1 = 16 - w // 2, 27
    x1, y0 = x0 + w - 1, y1 - h + 1
    d.rectangle([x0, y0, x1, y1], fill=GLASS_PALE)
    d.rectangle([x0, y0, x1, y1], outline=SILICONE, width=1)
    d.line([x0, y1, x1, y1], fill=SILICONE, width=2)  # bottom seam
    d.line([x0 + w // 3, y0, x0 + w // 3, y1], fill=GLASS, width=1)
    d.line([x0 + 2 * w // 3, y0, x0 + 2 * w // 3, y1], fill=GLASS, width=1)
    # diagonal glint on the front pane
    d.line([x0 + 3, y1 - 3, x0 + 7, y0 + 3], fill=(255, 255, 255, 255),
           width=1)
    for cx, cy in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)):
        d.point((cx, cy), fill=(240, 240, 240, 255))  # wet silicone bead
    speckle(d, rng, (x0 + 1, y0 + 1, x1 - 1, y1 - 1), GLASS, 5)


# ------------------------------------------------------------------ bottles

def bottle_glass(d, rng):
    # 330 ml glass bottle: body, shoulder, neck, crown cap
    d.rectangle([12, 12, 19, 27], fill=GLASS)
    d.rectangle([12, 12, 19, 27], outline=GLASS_DARK, width=1)
    d.polygon([(12, 12), (19, 12), (17, 7), (14, 7)], fill=GLASS)
    d.rectangle([14, 4, 17, 7], fill=GLASS)
    d.rectangle([13, 3, 18, 4], fill=(196, 164, 60, 255))  # crown cap
    d.line([14, 14, 14, 25], fill=GLASS_PALE, width=1)  # glint
    d.rectangle([12, 19, 19, 23], fill=shade(GLASS, 0.88))  # label zone


def bottle_plastic(d, rng):
    # 355 ml ridged water bottle, blue cap
    d.rectangle([12, 11, 19, 27], fill=PLASTIC_CYAN)
    d.polygon([(12, 11), (19, 11), (17, 6), (14, 6)], fill=PLASTIC_CYAN)
    d.rectangle([14, 3, 17, 6], fill=CAP_BLUE)
    for y in (15, 19, 23):  # grip ridges
        d.line([12, y, 19, y], fill=shade(PLASTIC_CYAN, 0.78), width=1)
    d.line([13, 12, 13, 26], fill=(240, 250, 252, 255), width=1)
    d.rectangle([12, 11, 19, 27], outline=shade(PLASTIC_CYAN, 0.7), width=1)


def glue_bottle(d, rng, scale):
    """Plastic glue bottle: pointed nozzle cap, glue label band."""
    h = int(22 * scale)
    w = int(10 * scale)
    x0, y1 = 16 - w // 2, 27
    y0 = y1 - h + 1
    d.rectangle([x0, y0 + 5, x0 + w - 1, y1], fill=PLASTIC_WHITE)
    d.rectangle([x0, y0 + 5, x0 + w - 1, y1],
                outline=(170, 170, 168, 255), width=1)
    d.polygon([(x0 + 1, y0 + 5), (x0 + w - 2, y0 + 5),
               (16, y0 + 1)], fill=CAP_ORANGE)  # nozzle cone
    d.point((16, y0), fill=CAP_ORANGE)
    band_y = y0 + h // 2
    d.rectangle([x0, band_y, x0 + w - 1, band_y + 3],
                fill=(84, 140, 200, 255))  # glue label
    d.line([x0 + 1, band_y + 1, x0 + w - 2, band_y + 1],
           fill=PLASTIC_WHITE, width=1)
    speckle(d, rng, (x0 + 1, y1 - 3, x0 + w - 2, y1 - 1),
            (210, 210, 208, 255), 3)


# ---------------------------------------------------------------------- can

def aluminum_can(d, rng):
    d.rectangle([11, 11, 20, 27], fill=ALUMINUM)
    d.rectangle([11, 11, 20, 27], outline=ALUMINUM_DARK, width=1)
    d.ellipse([11, 9, 20, 13], fill=shade(ALUMINUM, 1.08))
    d.ellipse([11, 9, 20, 13], outline=ALUMINUM_DARK, width=1)
    d.ellipse([14, 10, 17, 12], outline=ALUMINUM_DARK, width=1)  # tab ring
    d.rectangle([11, 17, 20, 22], fill=(120, 178, 216, 255))  # blue band
    d.line([13, 12, 13, 26], fill=(240, 242, 246, 255), width=1)


# ------------------------------------------------------------------ cartons

def carton(d, rng, w, h):
    """Gable-top laminate carton with a threaded cap."""
    x0, y1 = 16 - w // 2, 27
    x1 = x0 + w - 1
    gable = 4
    y0 = y1 - h + 1
    d.rectangle([x0, y0 + gable, x1, y1], fill=CARTON)
    d.rectangle([x0, y0 + gable, x1, y1], outline=CARTON_SHADE, width=1)
    d.polygon([(x0, y0 + gable), (16, y0), (x1, y0 + gable)],
              fill=shade(CARTON, 0.94))
    d.line([x0, y0 + gable, x1, y0 + gable], fill=CARTON_SHADE, width=1)
    d.line([16, y0, 16, y0 + gable], fill=CARTON_SHADE, width=1)
    cx = x0 + w // 4  # threaded cap on one gable slope
    d.ellipse([cx - 1, y0 + 1, cx + 1, y0 + 3], fill=CAP_BLUE)
    d.rectangle([x0 + 2, y0 + gable + 4, x1 - 2, y0 + gable + 7],
                fill=(120, 178, 216, 255))  # printed band
    speckle(d, rng, (x0 + 1, y1 - 4, x1 - 1, y1 - 1), CARTON_SHADE, 3)


def egg_carton(d, rng):
    # flat cardboard box, 3x2 egg domes showing on the lid
    d.rectangle([5, 17, 26, 26], fill=CARDBOARD)
    d.rectangle([5, 17, 26, 26], outline=CARDBOARD_DARK, width=1)
    d.rectangle([5, 14, 26, 17], fill=shade(CARDBOARD, 1.08))
    d.line([5, 17, 26, 17], fill=CARDBOARD_DARK, width=1)
    for ex in (9, 15, 21):
        for ey in (15,):
            d.ellipse([ex - 2, ey - 2, ex + 2, ey + 2],
                      fill=(240, 234, 220, 255))  # egg tops through lid
    for ex in (9, 15, 21):  # lower row bumps on the box face
        d.arc([ex - 2, 19, ex + 2, 24], start=0, end=180,
              fill=CARDBOARD_DARK, width=1)
    d.point((16, 22), fill=CARDBOARD_DARK)  # latch


# ---------------------------------------------------------------------- clay

def clay_cup(d, rng):
    d.polygon([(11, 9), (21, 9), (19, 19), (13, 19)], fill=CLAY)
    d.ellipse([11, 7, 21, 11], fill=CLAY_LIGHT)
    d.ellipse([12, 8, 20, 10], fill=CLAY_DARK)  # hollow mouth
    d.line([12, 13, 20, 13], fill=shade(CLAY, 0.88), width=1)
    d.line([13, 19, 19, 19], fill=CLAY_DARK, width=1)


def clay_pot(d, rng):
    # bulky wide-bellied water pot, open mouth, no lid
    d.ellipse([6, 12, 26, 28], fill=CLAY)
    d.rectangle([11, 6, 21, 13], fill=CLAY)
    d.ellipse([10, 4, 22, 9], fill=CLAY_LIGHT)  # rim
    d.ellipse([11, 5, 21, 8], fill=(30, 20, 16, 255))  # open dark mouth
    d.arc([8, 12, 24, 27], start=90, end=200, fill=CLAY_LIGHT, width=2)
    d.line([8, 17, 24, 17], fill=shade(CLAY, 0.85), width=1)  # thrown ridge
    speckle(d, rng, (9, 20, 23, 26), CLAY_DARK, 5)


# ----------------------------------------------------------------------- jug

def jug(d, rng):
    d.rectangle([9, 13, 21, 27], fill=PLASTIC_WHITE)
    d.rectangle([9, 13, 21, 27], outline=(176, 196, 202, 255), width=1)
    d.polygon([(9, 13), (21, 13), (19, 9), (12, 9)], fill=PLASTIC_WHITE)
    d.rectangle([12, 6, 16, 9], fill=PLASTIC_CYAN)  # neck
    d.rectangle([12, 4, 16, 6], fill=CAP_BLUE)  # cap
    d.arc([18, 8, 25, 16], start=270, end=90, fill=(176, 196, 202, 255),
          width=2)  # handle
    d.rectangle([11, 17, 19, 22], fill=(196, 226, 236, 255))  # label
    d.line([11, 19, 17, 19], fill=(120, 160, 180, 255), width=1)


# ---------------------------------------------------------------- biogas etc

def biogas_tank(d, rng, w, h, body, dark):
    """Propane-form tank: capsule body, collar, valve, base ring."""
    x0, y1 = 16 - w // 2, 28
    x1 = x0 + w - 1
    y0 = y1 - h + 1
    d.rectangle([x0, y0 + 3, x1, y1 - 2], fill=body)
    d.ellipse([x0, y0, x1, y0 + 6], fill=body)  # domed top
    d.rectangle([x0, y1 - 2, x1, y1], fill=dark)  # base ring
    d.line([x0 + 2, y0 + 5, x0 + 2, y1 - 3], fill=shade(body, 1.18),
           width=1)  # highlight
    d.line([x1 - 2, y0 + 5, x1 - 2, y1 - 3], fill=dark, width=1)
    d.rectangle([16 - 2, y0 - 2, 16 + 1, y0 + 1], fill=STEEL)  # valve stem
    d.line([16 - 3, y0 - 2, 16 + 2, y0 - 2], fill=INK, width=1)  # handwheel
    # collar guard posts
    d.line([x0 + 3, y0 + 1, x0 + 3, y0 - 1], fill=dark, width=1)
    d.line([x1 - 3, y0 + 1, x1 - 3, y0 - 1], fill=dark, width=1)
    # flame mark on the body
    fy = (y0 + y1) // 2
    d.polygon([(16, fy - 3), (18, fy + 1), (16, fy + 3), (14, fy + 1)],
              fill=(240, 170, 60, 255))
    speckle(d, rng, (x0 + 2, y1 - 6, x1 - 2, y1 - 3), dark, 3)


def pillbox(d, rng, rows):
    """Flat weekly organizer: rows x 7 day cells."""
    w, h = 22, 6 + rows * 5
    x0, y0 = 16 - w // 2, 16 - h // 2
    x1, y1 = x0 + w - 1, y0 + h - 1
    d.rectangle([x0, y0, x1, y1], fill=PLASTIC_WHITE)
    d.rectangle([x0, y0, x1, y1], outline=(168, 168, 166, 255), width=1)
    cell_w = w // 7
    for r in range(rows):
        for c in range(7):
            cx0 = x0 + c * cell_w
            cy0 = y0 + r * (h // rows)
            d.rectangle([cx0, cy0, cx0 + cell_w, cy0 + h // rows],
                        outline=(190, 190, 188, 255), width=1)
            if (c + r) % 3 == 0:  # a pill sitting in some cells
                d.point((cx0 + cell_w // 2, cy0 + (h // rows) // 2),
                        fill=(232, 150, 150, 255))
    d.line([x0 + 1, y0 + 1, x1 - 1, y0 + 1], fill=(200, 220, 240, 255),
           width=1)  # translucent lid sheen


def canister(d, rng):
    d.rectangle([11, 12, 20, 27], fill=GREEN_TANK)
    d.ellipse([11, 9, 20, 14], fill=shade(GREEN_TANK, 1.1))
    d.rectangle([11, 12, 20, 27], outline=GREEN_DARK, width=1)
    d.rectangle([14, 5, 17, 9], fill=STEEL)  # valve
    d.point((15, 4), fill=INK)
    d.rectangle([11, 18, 20, 21], fill=(240, 200, 60, 255))  # hazard band
    d.line([12, 19, 19, 19], fill=INK, width=1)
    d.line([13, 13, 13, 26], fill=shade(GREEN_TANK, 1.2), width=1)


def six_pack_rings(d, rng):
    col = PLASTIC_CYAN
    for r in range(2):
        for c in range(3):
            cx, cy = 9 + c * 7, 11 + r * 9
            d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], outline=col,
                      width=2)
    for c in range(2):  # connectors between rings
        d.line([12 + c * 7, 11, 13 + c * 7, 11], fill=col, width=1)
        d.line([12 + c * 7, 20, 13 + c * 7, 20], fill=col, width=1)
    d.line([9, 14, 9, 17], fill=col, width=1)
    d.line([23, 14, 23, 17], fill=col, width=1)


def squeeze_tube(d, rng):
    d.polygon([(13, 12), (19, 12), (18, 24), (14, 24)],
              fill=PLASTIC_WHITE)
    d.polygon([(13, 12), (19, 12), (18, 24), (14, 24)],
              outline=(170, 170, 168, 255))
    d.rectangle([14, 24, 18, 26], fill=(190, 190, 188, 255))  # crimp tail
    for x in (15, 17):
        d.point((x, 25), fill=(150, 150, 148, 255))
    d.rectangle([15, 8, 17, 12], fill=PLASTIC_WHITE)  # neck
    d.rectangle([15, 6, 17, 8], fill=CAP_ORANGE)  # nozzle cap
    d.rectangle([14, 16, 18, 19], fill=(84, 140, 200, 255))  # glue label


def spool(d, rng, size, wind_col, turns):
    """Plastic spool: two flanges + core wound with material."""
    w = size
    x0, y0 = 16 - w // 2, 16 - w // 2
    x1, y1 = x0 + w - 1, y0 + w - 1
    core_w = max(2, w // 3)
    cx0, cx1 = 16 - core_w // 2, 16 + core_w // 2 - 1
    # winding between the flanges
    d.rectangle([x0 + 1, y0 + 2, x1 - 1, y1 - 2], fill=wind_col)
    for i in range(turns):
        yy = y0 + 3 + i * max(1, (w - 6) // max(1, turns))
        if yy < y1 - 2:
            d.line([x0 + 1, yy, x1 - 1, yy], fill=shade(wind_col, 0.8),
                   width=1)
    d.rectangle([cx0, y0 + 1, cx1, y1 - 1], fill=shade(wind_col, 0.9))
    # flanges
    d.rectangle([x0, y0, x1, y0 + 1], fill=PLASTIC_DARK)
    d.rectangle([x0, y1 - 1, x1, y1], fill=PLASTIC_DARK)
    d.point((16, y0), fill=(30, 30, 32, 255))  # axle hole hint
    d.point((16, y1 - 1), fill=(30, 30, 32, 255))


def welding_rod_tube(d, rng):
    # long clear canister, rods inside, threaded cap on top
    d.rectangle([12, 8, 19, 28], fill=GLASS_PALE)
    d.rectangle([12, 8, 19, 28], outline=GLASS_DARK, width=1)
    for x in (14, 16, 18):  # welding rods
        d.line([x, 10, x, 27], fill=(140, 110, 80, 255), width=1)
    d.rectangle([11, 4, 20, 8], fill=CAP_BLUE)  # threaded cap
    for x in (13, 16, 19):
        d.line([x, 4, x, 8], fill=shade(CAP_BLUE, 0.75), width=1)
    d.line([13, 10, 13, 27], fill=(255, 255, 255, 255), width=1)


def bar_wrapper(d, rng):
    d.rectangle([6, 13, 25, 20], fill=(232, 230, 226, 255))
    d.rectangle([6, 13, 25, 20], outline=(168, 166, 162, 255), width=1)
    for x in (6, 7, 24, 25):  # crimped ends
        d.line([x, 13, x, 20], fill=(190, 188, 184, 255), width=1)
    d.rectangle([6, 15, 25, 17], fill=(196, 84, 60, 255))  # printed stripe
    d.line([9, 16, 15, 16], fill=PLASTIC_WHITE, width=1)
    speckle(d, rng, (8, 18, 23, 19), (200, 198, 194, 255), 3)


def foil_roll(d, rng):
    # horizontal cardboard tube + silver foil sheet unrolling right
    d.rectangle([4, 13, 22, 19], fill=CARDBOARD)
    d.ellipse([4, 13, 8, 19], fill=CARDBOARD_DARK)
    d.ellipse([5, 14, 7, 18], fill=(60, 44, 30, 255))  # hollow core
    d.line([9, 14, 21, 14], fill=shade(CARDBOARD, 1.15), width=1)
    d.polygon([(22, 14), (28, 12), (28, 21), (22, 19)], fill=FOIL)
    d.line([22, 14, 28, 12], fill=ALUMINUM_DARK, width=1)
    d.line([23, 17, 27, 16], fill=ALUMINUM_DARK, width=1)  # foil crease
    for x in range(24, 28, 2):  # serrated tear edge
        d.point((x, 21), fill=ALUMINUM_DARK)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "aquarium_small_drying_active":
        aquarium(d, rng, 20, 13)   # 30 gal / 133 L
    elif sid == "aquarium_medium_drying_active":
        aquarium(d, rng, 24, 16)   # 50 gal / 222 L
    elif sid == "aquarium_large_drying_active":
        aquarium(d, rng, 28, 20)   # 100 gal / 400 L
    elif sid == "bottle_glass_330ml":
        bottle_glass(d, rng)
    elif sid == "bottle_plastic_355ml":
        bottle_plastic(d, rng)
    elif sid == "bottle_plastic_glue":
        glue_bottle(d, rng, 1.0)   # 500 ml glue
    elif sid == "bottle_plastic_small_glue":
        glue_bottle(d, rng, 0.72)  # 250 ml glue
    elif sid == "can_aluminum_250ml":
        aluminum_can(d, rng)
    elif sid == "carton_284ml":
        carton(d, rng, 12, 16)     # half pint
    elif sid == "carton_946ml":
        carton(d, rng, 15, 22)     # quart
    elif sid == "carton_egg_6":
        egg_carton(d, rng)
    elif sid == "clay_cup":
        clay_cup(d, rng)
    elif sid == "clay_watercont_no_lid":
        clay_pot(d, rng)
    elif sid == "jug_plastic_1892ml":
        jug(d, rng)
    elif sid == "small_biogas_tank":
        biogas_tank(d, rng, 12, 14, GREEN_TANK, GREEN_DARK)   # 5 L
    elif sid == "medium_biogas_tank":
        biogas_tank(d, rng, 16, 19, GREEN_TANK, GREEN_DARK)   # 23 L
    elif sid == "large_biogas_tank":
        biogas_tank(d, rng, 20, 24, GRAY_TANK, GRAY_DARK)     # 80 L
    elif sid == "xl_biogas_tank":
        biogas_tank(d, rng, 25, 26, GRAY_TANK, GRAY_DARK)     # 390 L
    elif sid == "pillbox":
        pillbox(d, rng, 1)         # one week
    elif sid == "pillbox_large":
        pillbox(d, rng, 2)         # two weeks
    elif sid == "pressurized_canister":
        canister(d, rng)
    elif sid == "six_pack_rings":
        six_pack_rings(d, rng)
    elif sid == "small_squeeze_tube":
        squeeze_tube(d, rng)
    elif sid == "spool_tiny":
        spool(d, rng, 9, THREAD, 3)
    elif sid == "spool_small":
        spool(d, rng, 13, STEEL, 4)
    elif sid == "spool_welding":
        spool(d, rng, 21, COPPER, 6)
    elif sid == "tube_welding_rod":
        welding_rod_tube(d, rng)
    elif sid == "wrapper_bar":
        bar_wrapper(d, rng)
    elif sid == "wrapper_foil_roll":
        foil_roll(d, rng)
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
    # contact-sheet preview: 6 cols x 5 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item50.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item50.png {prev.size}")


if __name__ == "__main__":
    main()
