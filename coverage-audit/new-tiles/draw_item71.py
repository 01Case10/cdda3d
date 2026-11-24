#!/usr/bin/env python3
"""ITEM71: farrier tools + vehicle storage + misc resources + offal
dishes -- CDDA Voxel coverage audit.

Four families, each drawn as itself:

  Farrier tools (items/farrier.json, all mc_steel but the plastic
    hoof pick) -- steel hand tools in profile: clinch block with its
    angled edge, hatchet-like clinch cutter, pliers/clinchers,
    hoof nippers, rasp, hoof stand, two-pronged hoof tester,
    horseshoe pullers, nail puller, pritchel punch, and the
    horseshoe itself: a U band with nail holes.

  Vehicle storage (items/vehicle/cargo.json) -- wire baskets
    (shopping-cart, bike, foldable), bike rack of pipes and straps,
    floor trunk panel, cargo lock set, cargo and livestock carrier
    frames, a mounted spare tire, and the aluminum nose plate.

  Misc resources (items/resources/misc.json) -- a destroyed
    ryusei kabuto in two cracked halves with metallic tubes in the
    split, bee and fungal-fighter stingers, batteries discharged
    and soaking, a fuse coil, incendiary powder, rubber tire
    chunk/strip/sole, and the M320 grenade-launcher buttstock.

  Offal dishes (items/comestibles/offal_dishes.json) -- canned
    heart, deviled kidney, fried brain, deep-fried tripe, homemade
    hot dogs, lung provence, natural and artificial sausage
    casings, scrambled eggs and brain, stuffed heart, tottchen.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "clinch_block",
    "clinch_cutter",
    "clinch_pliers",
    "hoof_nippers",
    "hoof_pick",
    "hoof_rasp",
    "hoof_stand",
    "hoof_tester",
    "horseshoe",
    "horseshoe_pliers",
    "nail_puller",
    "pritchel",
    "animal_locker",
    "basket",
    "bike_basket",
    "bike_rack",
    "cargo_aisle",
    "cargo_lock",
    "cargo_rack",
    "folding_basket",
    "livestock_carrier",
    "mounted_spare_tire",
    "nose_plate",
    "armor_star_samurai_helmet_destroyed",
    "bee_sting",
    "discharged_storage_battery",
    "discharging_battery",
    "fighter_sting",
    "fuse",
    "incendiary",
    "rubber_tire_chunk",
    "rubber_tire_strip",
    "sole_rubber",
    "ugl_buttstock",
    "canned_heart",
    "deviled_kidney",
    "fried_brain",
    "fried_tripe",
    "homemade_hotdogs",
    "lung_provence",
    "sausage_casings",
    "sausage_casings_artificial",
    "scrambledeggsandbrain",
    "stuffed_heart",
    "tÖttchen",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

# steel grades (ITEM47 language): mc = mid gray, lc = light matte
MC = (148, 150, 156, 255)
MC_DARK = (86, 88, 94, 255)
MC_LITE = (198, 200, 206, 255)
LC = (178, 180, 186, 255)
LC_DARK = (110, 112, 118, 255)
WOOD = (122, 86, 48, 255)
WOOD_DARK = (88, 60, 32, 255)
PLASTIC_BLUE = (52, 92, 168, 255)
PLASTIC_DARK = (44, 46, 52, 255)
RUBBER = (34, 34, 38, 255)
RUBBER_LITE = (72, 72, 78, 255)
ALU = (196, 200, 204, 255)
ALU_DARK = (128, 132, 138, 255)
ROCK = (74, 70, 78, 255)
ROCK_LITE = (104, 100, 110, 255)
AMBER = (196, 132, 44, 255)
AMBER_DARK = (128, 80, 24, 255)
PLATE = (216, 216, 220, 255)
PLATE_DARK = (160, 160, 166, 255)
MEAT = (168, 52, 48, 255)
MEAT_DARK = (116, 34, 34, 255)
SAUCE = (120, 40, 32, 255)
GOLDEN = (206, 156, 84, 255)
GOLDEN_DARK = (158, 110, 52, 255)
BRAIN = (216, 168, 156, 255)
BRAIN_DARK = (178, 124, 112, 255)
EGG = (240, 196, 84, 255)
EGG_DARK = (206, 158, 52, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def bar(d, pts, w, col, dark):
    """A steel bar along pts: dark underlay, then the face colour."""
    d.line(pts, fill=dark, width=w + 2)
    d.line(pts, fill=col, width=w)


def pivot(d, cx, cy):
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=MC_DARK)
    d.point((cx, cy), fill=MC_LITE)


# ------------------------------------------------------------- farrier tools

def d_clinch_block(d, rng):
    # metal block, angled top edge for clinching nails against
    d.polygon([(9, 15), (15, 9), (23, 9), (23, 25), (9, 25)], fill=MC)
    d.polygon([(9, 15), (15, 9), (23, 9), (23, 25), (9, 25)], outline=MC_DARK)
    d.line([9, 15, 15, 9], fill=MC_LITE, width=2)          # angled face
    d.line([10, 19, 22, 19], fill=MC_DARK, width=1)
    for _ in range(4):                                     # wear scratches
        x, y = rng.randint(11, 21), rng.randint(20, 24)
        d.point((x, y), fill=MC_DARK)


def d_clinch_cutter(d, rng):
    # small hatchet: wooden haft, wedge head
    bar(d, [(11, 25), (18, 14)], 3, WOOD, WOOD_DARK)
    d.polygon([(16, 5), (27, 7), (27, 13), (17, 13), (15, 9)], fill=MC)
    d.polygon([(16, 5), (27, 7), (27, 13), (17, 13), (15, 9)],
              outline=MC_DARK)
    d.line([26, 8, 26, 12], fill=MC_LITE, width=1)         # cutting edge


def _pliers(d, rng, jaw):
    # shared plier body: two handles below a pivot; jaw(d) draws the head
    bar(d, [(11, 27), (14, 16)], 3, MC, MC_DARK)
    bar(d, [(21, 27), (18, 16)], 3, MC, MC_DARK)
    jaw(d)
    pivot(d, 16, 15)


def d_clinch_pliers(d, rng):
    def jaw(dd):  # short flat jaws, tips bent down to flatten the nail
        dd.polygon([(13, 15), (13, 7), (16, 6), (16, 15)], fill=MC)
        dd.polygon([(19, 15), (19, 6), (16, 6), (16, 15)], fill=MC)
        dd.line([13, 7, 19, 7], fill=MC_DARK, width=1)
    _pliers(d, rng, jaw)


def d_hoof_nippers(d, rng):
    def jaw(dd):  # broad opposing cutting blades
        dd.polygon([(12, 15), (11, 5), (16, 4), (16, 15)], fill=MC)
        dd.polygon([(20, 15), (21, 5), (16, 4), (16, 15)], fill=MC)
        dd.line([11, 5, 21, 5], fill=MC_LITE, width=1)
    _pliers(d, rng, jaw)


def d_horseshoe_pliers(d, rng):
    def jaw(dd):  # wide flat gripping jaws of a puller
        dd.rectangle([12, 5, 15, 15], fill=MC)
        dd.rectangle([17, 5, 20, 15], fill=MC)
        dd.line([12, 5, 20, 5], fill=MC_DARK, width=1)
        for x in (13, 18):
            dd.line([x, 7, x, 13], fill=MC_DARK, width=1)  # grip teeth
    _pliers(d, rng, jaw)


def d_hoof_tester(d, rng):
    # two long prongs on scissor arms, squeezed to press the sole
    bar(d, [(10, 27), (15, 12)], 3, MC, MC_DARK)
    bar(d, [(22, 27), (17, 12)], 3, MC, MC_DARK)
    bar(d, [(15, 12), (13, 5)], 2, MC, MC_DARK)            # prong tips
    bar(d, [(17, 12), (19, 5)], 2, MC, MC_DARK)
    d.point((13, 4), fill=MC_DARK)
    d.point((19, 4), fill=MC_DARK)
    pivot(d, 16, 14)


def d_hoof_pick(d, rng):
    # brush block + plastic handle + steel hook pick
    d.rectangle([4, 13, 10, 19], fill=WOOD)
    for x in range(4, 11):                                 # bristles
        d.line([x, 19, x, 22], fill=(196, 178, 130, 255), width=1)
    d.rectangle([10, 14, 21, 18], fill=PLASTIC_BLUE)
    d.rectangle([10, 14, 21, 18], outline=shade(PLASTIC_BLUE, 0.6))
    bar(d, [(21, 16), (26, 16), (27, 20)], 2, MC, MC_DARK)  # hook


def d_hoof_rasp(d, rng):
    # big flat file with crosshatch teeth, tang into a wooden handle
    d.rectangle([13, 3, 19, 20], fill=MC)
    d.rectangle([13, 3, 19, 20], outline=MC_DARK)
    for y in range(5, 19, 3):
        d.line([14, y, 18, y + 2], fill=MC_DARK, width=1)
        d.line([18, y, 14, y + 2], fill=MC_LITE, width=1)
    d.rectangle([14, 20, 18, 22], fill=MC_DARK)            # tang
    d.rectangle([12, 22, 20, 29], fill=WOOD)
    d.rectangle([12, 22, 20, 29], outline=WOOD_DARK)


def d_hoof_stand(d, rng):
    # post on a round base, cradle fork on top for the hoof
    d.ellipse([8, 25, 24, 29], fill=MC_DARK)
    d.ellipse([9, 24, 23, 28], fill=MC)
    d.rectangle([15, 12, 17, 25], fill=MC)
    d.line([15, 12, 17, 12], fill=MC_DARK, width=1)
    bar(d, [(15, 12), (11, 7)], 2, MC, MC_DARK)            # fork prongs
    bar(d, [(17, 12), (21, 7)], 2, MC, MC_DARK)
    d.arc([11, 6, 21, 14], start=20, end=160, fill=MC_DARK, width=2)


def d_horseshoe(d, rng):
    # U band, opening down, nail holes punched along it
    d.arc([6, 4, 26, 27], start=125, end=55, fill=MC_DARK, width=7)
    d.arc([6, 4, 26, 27], start=125, end=55, fill=MC, width=5)
    cx, cy, rx, ry = 16, 15.5, 10, 11.5
    for deg in (150, 185, 220, 255, 290, 325, 355, 25):
        a = math.radians(deg)
        x, y = cx + rx * math.cos(a), cy + ry * math.sin(a)
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=MC_DARK)  # nail holes
    d.polygon([(14, 3), (18, 3), (16, 6)], fill=MC_DARK)   # toe clip


def d_nail_puller(d, rng):
    # pry bar with a claw notch at the working end
    bar(d, [(8, 25), (19, 12)], 3, MC, MC_DARK)
    d.polygon([(19, 12), (24, 6), (27, 8), (22, 14)], fill=MC)
    d.polygon([(19, 12), (24, 6), (27, 8), (22, 14)], outline=MC_DARK)
    d.polygon([(23, 9), (25, 7), (24, 11)], fill=(0, 0, 0, 0))
    d.line([23, 10, 25, 7], fill=MC_DARK, width=1)         # the V notch
    d.line([7, 26, 10, 23], fill=MC_LITE, width=2)         # flat tail


def d_pritchel(d, rng):
    # tapered punch: broad striking face, point for nail holes
    d.polygon([(13, 4), (19, 4), (17, 18), (16, 27), (15, 18)], fill=MC)
    d.polygon([(13, 4), (19, 4), (17, 18), (16, 27), (15, 18)],
              outline=MC_DARK)
    d.rectangle([13, 4, 19, 6], fill=MC_LITE)              # struck face
    for y in (10, 14):                                     # grip rings
        d.line([14, y, 18, y], fill=MC_DARK, width=1)


# ----------------------------------------------------------- vehicle storage

def _wire_basket(d, x0, x1, y0, y1, fold=False):
    bx0, bx1 = x0 + 3, x1 - 3                               # narrower bottom
    d.polygon([(x0, y0), (x1, y0), (bx1, y1), (bx0, y1)], outline=LC_DARK)
    for x in range(x0 + 2, x1 - 1, 3):                      # vertical wires
        t = (x - x0) / (x1 - x0)
        d.line([x, y0, bx0 + t * (bx1 - bx0), y1], fill=LC, width=1)
    for y in range(y0 + 3, y1, 3):                          # horizontal wires
        t = (y - y0) / (y1 - y0)
        d.line([x0 + t * 3, y, x1 - t * 3, y], fill=LC, width=1)
    d.rectangle([x0 - 1, y0 - 1, x1 + 1, y0 + 1], fill=LC)  # rim
    if fold:  # hinge knuckles + X fold braces on the faces
        for x in (x0 + 2, x1 - 2):
            d.ellipse([x - 1, y0 + 6, x + 1, y0 + 8], fill=LC_DARK)
        d.line([x0 + 2, y0 + 2, bx1 - 2, y1 - 1], fill=LC_DARK, width=1)
        d.line([x1 - 2, y0 + 2, bx0 + 2, y1 - 1], fill=LC_DARK, width=1)


def d_basket(d, rng):
    _wire_basket(d, 4, 28, 10, 25)
    d.line([4, 8, 28, 8], fill=LC_DARK, width=1)            # cart handle hint


def d_folding_basket(d, rng):
    _wire_basket(d, 5, 27, 11, 24, fold=True)


def d_bike_basket(d, rng):
    _wire_basket(d, 8, 24, 11, 22)
    d.rectangle([14, 7, 18, 11], fill=LC_DARK)              # handlebar clamp


def d_animal_locker(d, rng):
    d.rectangle([6, 7, 26, 27], fill=LC)
    d.rectangle([6, 7, 26, 27], outline=LC_DARK)
    d.rectangle([9, 10, 23, 24], outline=LC_DARK)           # barred door
    for x in range(11, 23, 3):
        d.line([x, 11, x, 23], fill=LC_DARK, width=1)
    d.line([9, 17, 23, 17], fill=LC_DARK, width=1)
    d.rectangle([21, 15, 23, 19], fill=MC_DARK)             # latch
    for x in range(10, 23, 4):                              # vent slots
        d.point((x, 8), fill=LC_DARK)


def d_bike_rack(d, rng):
    # pipes, wheel trays, cams and straps
    bar(d, [(6, 22), (26, 22)], 2, LC, LC_DARK)            # base rail
    bar(d, [(9, 22), (9, 10), (13, 8)], 2, LC, LC_DARK)    # upright + hook
    bar(d, [(23, 22), (23, 10), (19, 8)], 2, LC, LC_DARK)
    d.line([6, 26, 26, 26], fill=LC_DARK, width=2)          # wheel tray
    for x in (8, 15, 22):
        d.point((x, 25), fill=LC)
    d.line([11, 12, 20, 20], fill=PLASTIC_DARK, width=2)    # strap
    d.rectangle([15, 15, 18, 18], fill=MC_DARK)             # cam buckle


def d_cargo_aisle(d, rng):
    # floor panel with a hinged access door
    d.rectangle([4, 12, 28, 23], fill=LC)
    d.rectangle([4, 12, 28, 23], outline=LC_DARK)
    d.rectangle([8, 14, 24, 21], outline=LC_DARK)           # door seam
    d.rectangle([14, 16, 18, 18], fill=LC_DARK)             # recessed handle
    for x, y in ((5, 13), (27, 13), (5, 22), (27, 22)):
        d.point((x, y), fill=MC_DARK)                       # corner bolts


def d_cargo_lock(d, rng):
    # a padlock, a second smaller lock, and a hasp plate
    d.rectangle([7, 14, 16, 24], fill=GOLDEN)
    d.rectangle([7, 14, 16, 24], outline=GOLDEN_DARK)
    d.arc([8, 6, 15, 16], start=180, end=360, fill=MC_DARK, width=2)
    d.ellipse([10, 17, 12, 20], fill=GOLDEN_DARK)           # keyhole
    d.rectangle([19, 17, 25, 24], fill=MC)
    d.rectangle([19, 17, 25, 24], outline=MC_DARK)
    d.arc([20, 12, 24, 19], start=180, end=360, fill=MC_DARK, width=2)
    d.rectangle([19, 26, 26, 28], fill=LC)                  # hasp plate
    d.point((21, 27), fill=MC_DARK)
    d.point((24, 27), fill=MC_DARK)


def _carrier_frame(d, rails=False):
    d.rectangle([5, 12, 27, 25], outline=LC_DARK)
    d.rectangle([5, 12, 27, 25], outline=LC)
    for x in (12, 20):                                      # crossbars
        d.line([x, 12, x, 25], fill=LC, width=1)
    d.line([5, 19, 27, 19], fill=LC, width=1)
    for x, y in ((5, 12), (27, 12), (5, 25), (27, 25)):     # tie-down eyes
        d.ellipse([x - 1, y - 1, x + 1, y + 1], outline=MC_DARK)
    if rails:  # livestock railings stand above the frame
        for x in range(7, 27, 4):
            d.line([x, 12, x, 6], fill=LC, width=1)
        d.line([5, 6, 27, 6], fill=LC, width=2)


def d_cargo_rack(d, rng):
    _carrier_frame(d)


def d_livestock_carrier(d, rng):
    _carrier_frame(d, rails=True)


def d_mounted_spare_tire(d, rng):
    # tire on its carrier post
    d.rectangle([15, 22, 17, 29], fill=MC_DARK)             # carrier post
    d.rectangle([10, 27, 22, 29], fill=MC_DARK)             # base arm
    d.ellipse([6, 4, 26, 24], fill=RUBBER)
    for deg in range(0, 360, 30):                           # tread blocks
        a = math.radians(deg)
        x, y = 16 + 9 * math.cos(a), 14 + 9 * math.sin(a)
        d.point((int(x), int(y)), fill=RUBBER_LITE)
    d.ellipse([11, 9, 21, 19], fill=LC)                     # rim
    d.ellipse([14, 12, 18, 16], fill=LC_DARK)               # hub
    for deg in range(0, 360, 72):
        a = math.radians(deg)
        d.point((int(16 + 3 * math.cos(a)), int(14 + 3 * math.sin(a))),
                fill=MC_DARK)


def d_nose_plate(d, rng):
    # L-shaped aluminum plate, straps, skid runners (side profile)
    d.rectangle([9, 5, 13, 22], fill=ALU)                   # upright leg
    d.rectangle([9, 5, 13, 22], outline=ALU_DARK)
    d.rectangle([9, 20, 27, 24], fill=ALU)                  # foot leg
    d.rectangle([9, 20, 27, 24], outline=ALU_DARK)
    for y in (9, 15):                                       # straps
        d.rectangle([9, y, 13, y + 2], fill=PLASTIC_DARK)
    d.line([14, 22, 26, 22], fill=ALU_DARK, width=1)
    for x in (12, 20, 26):                                  # skid runners
        d.rectangle([x, 25, x + 3, 26], fill=MC_DARK)


# ---------------------------------------------------------------- miscellany

def d_destroyed_kabuto(d, rng):
    # conical rock slab split in two, metallic tubes in the crack
    d.polygon([(5, 24), (7, 12), (13, 5), (15, 9), (13, 14), (15, 18),
               (13, 24)], fill=ROCK)
    d.polygon([(5, 24), (7, 12), (13, 5), (15, 9), (13, 14), (15, 18),
               (13, 24)], outline=shade(ROCK, 0.6))
    d.polygon([(19, 10), (21, 5), (26, 13), (27, 24), (18, 24), (19, 18),
               (17, 14)], fill=ROCK)
    d.polygon([(19, 10), (21, 5), (26, 13), (27, 24), (18, 24), (19, 18),
               (17, 14)], outline=shade(ROCK, 0.6))
    for x, y in ((9, 10), (22, 9), (8, 19), (24, 18)):      # rock facets
        d.point((x, y), fill=ROCK_LITE)
    for y in (11, 16, 21):                                  # exposed tubes
        d.rectangle([15, y, 18, y + 2], fill=MC_LITE)
        d.point((18, y), fill=(84, 160, 150, 255))
    d.line([4, 26, 28, 26], fill=shade(ROCK, 0.6), width=1)  # brim shards


def _sting(d, rng, base_col, tip_col, barb_col, long_=True):
    # venom-sac bulb at top-left, tapered sting running down-right
    d.ellipse([5, 5, 12, 12], fill=base_col)
    d.ellipse([5, 5, 12, 12], outline=shade(base_col, 0.6))
    tip = (27, 27) if long_ else (22, 22)
    d.polygon([(9, 10), (12, 8), tip, (8, 13)], fill=tip_col)
    d.polygon([(9, 10), (12, 8), tip, (8, 13)], outline=shade(tip_col, 0.6))
    for t in (0.3, 0.5, 0.7):                               # barbs
        x = 10 + (tip[0] - 10) * t
        y = 10 + (tip[1] - 10) * t
        d.line([x, y, x - 3, y + 1], fill=barb_col, width=1)


def d_bee_sting(d, rng):
    _sting(d, rng, AMBER, AMBER_DARK, AMBER)


def d_fighter_sting(d, rng):
    # short vegetal dart: green, segmented, leaf-finned
    _sting(d, rng, (96, 140, 62, 255), (62, 102, 40, 255),
           (120, 164, 84, 255), long_=False)
    d.polygon([(6, 14), (3, 18), (8, 17)], fill=(96, 140, 62, 255))


def _battery(d, rng):
    d.rectangle([8, 11, 24, 23], fill=(96, 98, 104, 255))
    d.rectangle([8, 11, 24, 23], outline=PLASTIC_DARK)
    d.rectangle([8, 11, 24, 14], fill=PLASTIC_DARK)         # cap band
    d.rectangle([11, 8, 14, 11], fill=MC_DARK)              # terminals
    d.rectangle([18, 8, 21, 11], fill=MC_DARK)
    d.rectangle([11, 17, 21, 20], fill=(150, 150, 154, 255))  # label


def d_discharged_battery(d, rng):
    _battery(d, rng)
    for _ in range(10):                                     # salt crust
        d.point((rng.randint(9, 23), rng.randint(12, 22)),
                fill=(232, 232, 228, 255))
    d.line([11, 18, 21, 18], fill=(120, 120, 124, 255), width=1)


def d_discharging_battery(d, rng):
    # battery soaking tilted in a tub of saltwater
    d.polygon([(5, 17), (27, 17), (25, 28), (7, 28)], fill=(70, 110, 150, 255))
    d.polygon([(5, 17), (27, 17), (25, 28), (7, 28)], outline=(44, 74, 108, 255))
    d.rectangle([10, 6, 21, 18], fill=(96, 98, 104, 255))
    d.rectangle([10, 6, 21, 18], outline=PLASTIC_DARK)
    d.rectangle([10, 6, 21, 9], fill=PLASTIC_DARK)
    d.rectangle([12, 3, 15, 6], fill=MC_DARK)
    d.rectangle([17, 3, 20, 6], fill=MC_DARK)
    d.line([6, 19, 26, 19], fill=(150, 190, 220, 255), width=1)  # waterline
    for _ in range(7):                                      # bubbles
        d.point((rng.randint(8, 24), rng.randint(20, 26)),
                fill=(190, 220, 240, 255))


def d_fuse(d, rng):
    # coiled length of cotton fuse cord with a loose end
    for r in (9, 6, 3):
        d.ellipse([16 - r, 15 - r, 16 + r, 15 + r],
                  outline=(178, 158, 120, 255), width=2)
    d.line([23, 20, 28, 26], fill=(178, 158, 120, 255), width=2)
    d.point((28, 27), fill=(120, 104, 72, 255))             # cut end
    for deg in range(20, 340, 60):                          # cord flecks
        a = math.radians(deg)
        d.point((int(16 + 9 * math.cos(a)), int(15 + 9 * math.sin(a))),
                fill=(120, 104, 72, 255))


def d_incendiary(d, rng):
    # heap of incendiary powder beside a split brass casing
    d.polygon([(6, 26), (10, 17), (15, 14), (20, 17), (23, 26)],
              fill=(214, 92, 40, 255))
    for _ in range(16):
        x = rng.randint(8, 21)
        y = rng.randint(19, 25)
        d.point((x, y), fill=(240, 150, 60, 255) if rng.random() < 0.5
                else (168, 56, 28, 255))
    d.polygon([(24, 26), (26, 19), (29, 20), (28, 26)], fill=GOLDEN)
    d.line([26, 19, 28, 26], fill=GOLDEN_DARK, width=1)     # split seam
    for x, y in ((12, 10), (19, 8), (24, 12)):              # spark flecks
        d.point((x, y), fill=(250, 200, 90, 255))


def d_rubber_chunk(d, rng):
    d.polygon([(8, 10), (20, 7), (25, 13), (23, 23), (12, 25), (7, 18)],
              fill=RUBBER)
    d.polygon([(8, 10), (20, 7), (25, 13), (23, 23), (12, 25), (7, 18)],
              outline=RUBBER_LITE)
    for x in range(10, 22, 4):                              # tread grooves
        d.line([x, 9, x + 2, 13], fill=RUBBER_LITE, width=1)


def d_rubber_strip(d, rng):
    # long curled strip: S-curve with a rolled end
    pts = [(6 + i, 10 + int(6 * math.sin(i * 0.32))) for i in range(20)]
    d.line(pts, fill=RUBBER_LITE, width=5)
    d.line(pts, fill=RUBBER, width=3)
    d.arc([22, 14, 30, 24], start=300, end=200, fill=RUBBER, width=3)


def d_sole_rubber(d, rng):
    # boot sole, toe up: heel, waist, broad toe
    d.polygon([(13, 27), (13, 18), (11, 12), (12, 6), (15, 3), (19, 3),
               (21, 6), (20, 12), (18, 18), (18, 27)], fill=RUBBER)
    d.polygon([(13, 27), (13, 18), (11, 12), (12, 6), (15, 3), (19, 3),
               (21, 6), (20, 12), (18, 18), (18, 27)], outline=RUBBER_LITE)
    for y in (6, 10, 22, 25):                               # tread bars
        d.line([13, y, 19, y], fill=RUBBER_LITE, width=1)
    d.line([12, 15, 19, 15], fill=RUBBER_LITE, width=1)     # arch line


def d_ugl_buttstock(d, rng):
    # collapsible stock: buttplate, body, sliding rail underneath
    d.rectangle([24, 8, 27, 24], fill=PLASTIC_DARK)         # buttplate
    d.polygon([(24, 10), (12, 13), (10, 18), (24, 19)], fill=(58, 60, 66, 255))
    d.polygon([(24, 10), (12, 13), (10, 18), (24, 19)], outline=PLASTIC_DARK)
    d.rectangle([6, 14, 11, 17], fill=MC_DARK)              # receiver stub
    for x in (14, 18, 22):                                  # adjustment notches
        d.line([x, 20, x, 23], fill=MC_DARK, width=1)
    d.line([12, 21, 24, 21], fill=MC, width=1)              # slide rail


# ------------------------------------------------------------------ offal

def _plate(d, cx=16, cy=19, rx=11, ry=7):
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], fill=PLATE)
    d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], outline=PLATE_DARK)
    d.ellipse([cx - rx + 3, cy - ry + 2, cx + rx - 3, cy + ry - 2],
              outline=PLATE_DARK)


def d_canned_heart(d, rng):
    # open tin of heart chunks
    d.rectangle([9, 12, 23, 27], fill=MC)
    d.rectangle([9, 12, 23, 27], outline=MC_DARK)
    d.rectangle([9, 19, 23, 24], fill=(168, 74, 60, 255))   # label band
    d.ellipse([9, 8, 23, 14], fill=MC_LITE)                 # open rim
    d.ellipse([10, 9, 22, 13], fill=MEAT_DARK)              # contents
    for x, y in ((12, 10), (16, 11), (20, 10)):
        d.ellipse([x - 2, y - 1, x + 2, y + 2], fill=MEAT)  # heart chunks
    d.polygon([(18, 8), (24, 3), (26, 5), (21, 9)], fill=MC_LITE)  # peel lid


def d_deviled_kidney(d, rng):
    _plate(d)
    d.ellipse([6, 14, 26, 25], fill=SAUCE)                  # sauce pool
    for cx in (12, 20):                                     # kidney halves
        d.ellipse([cx - 4, 14, cx + 4, 22], fill=MEAT_DARK)
        d.ellipse([cx - 2, 16, cx + 1, 20], fill=MEAT)      # inner curve
    for _ in range(6):                                      # garnish
        d.point((rng.randint(8, 24), rng.randint(14, 23)),
                fill=(96, 150, 70, 255))


def _brain_mass(d, cx, cy, scale=1.0):
    pts = [(cx - 6, cy + 2), (cx - 7, cy - 2), (cx - 4, cy - 5),
           (cx, cy - 6), (cx + 4, cy - 5), (cx + 7, cy - 2),
           (cx + 6, cy + 2), (cx + 2, cy + 4), (cx - 2, cy + 4)]
    pts = [(cx + (x - cx) * scale, cy + (y - cy) * scale) for x, y in pts]
    d.polygon(pts, fill=BRAIN)
    d.polygon(pts, outline=BRAIN_DARK)
    return pts


def d_fried_brain(d, rng):
    _plate(d)
    _brain_mass(d, 16, 18)
    for _ in range(9):                                      # crispy flecks
        d.point((rng.randint(10, 22), rng.randint(13, 21)), fill=GOLDEN)
    d.arc([12, 13, 20, 20], start=200, end=320, fill=BRAIN_DARK, width=1)


def d_fried_tripe(d, rng):
    _plate(d)
    for i, (x, y) in enumerate(((10, 14), (16, 13), (13, 18), (19, 18))):
        d.polygon([(x, y), (x + 6, y + 1), (x + 5, y + 4), (x + 1, y + 4)],
                  fill=GOLDEN)
        d.polygon([(x, y), (x + 6, y + 1), (x + 5, y + 4), (x + 1, y + 4)],
                  outline=GOLDEN_DARK)
        for hx in range(x + 1, x + 5, 2):                   # honeycomb pits
            d.point((hx, y + 2), fill=GOLDEN_DARK)


def d_homemade_hotdogs(d, rng):
    # two dogs in buns on a paper tray
    d.polygon([(5, 20), (27, 20), (25, 27), (7, 27)], fill=(228, 224, 214, 255))
    d.polygon([(5, 20), (27, 20), (25, 27), (7, 27)], outline=PLATE_DARK)
    for y in (13, 19):
        d.ellipse([6, y, 26, y + 5], fill=(222, 178, 116, 255))  # bun
        d.line([8, y + 2, 24, y + 2], fill=(172, 84, 52, 255), width=2)
        d.line([8, y + 1, 24, y + 1], fill=(196, 108, 66, 255), width=1)


def d_lung_provence(d, rng):
    _plate(d)
    d.ellipse([6, 13, 26, 25], fill=(140, 52, 44, 255))     # wine sauce
    for cx, cy in ((12, 17), (19, 16), (16, 21)):           # lung slices
        d.ellipse([cx - 4, cy - 3, cx + 4, cy + 3], fill=(196, 138, 120, 255))
        for _ in range(4):                                  # spongy pores
            d.point((cx + rng.randint(-3, 3), cy + rng.randint(-2, 2)),
                    fill=(160, 100, 88, 255))
    for x, y in ((9, 14), (23, 20), (14, 23)):              # onion/tomato
        d.point((x, y), fill=(240, 230, 210, 255))
        d.point((x + 1, y), fill=(214, 84, 60, 255))


def _casing_coil(d, col, dark, neat):
    for r in ((8, 5) if neat else (8,)):
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=dark, width=4)
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=col, width=2)
    if not neat:  # loose inner loop, off-centre
        d.ellipse([11, 12, 19, 20], outline=dark, width=4)
        d.ellipse([11, 12, 19, 20], outline=col, width=2)


def d_sausage_casings(d, rng):
    _casing_coil(d, (226, 178, 160, 255), (188, 138, 122, 255), neat=False)
    d.line([22, 22, 27, 27], fill=(188, 138, 122, 255), width=2)  # loose end


def d_sausage_casings_artificial(d, rng):
    _casing_coil(d, (214, 226, 232, 255), (150, 168, 178, 255), neat=True)
    for deg in (40, 160, 280):                              # plastic sheen
        a = math.radians(deg)
        d.point((int(16 + 8 * math.cos(a)), int(16 + 8 * math.sin(a))),
                fill=(250, 252, 254, 255))
    d.line([23, 9, 27, 6], fill=(200, 60, 50, 255), width=1)  # tied string


def d_scrambled_eggs_brain(d, rng):
    _plate(d)
    for _ in range(10):                                     # egg curds
        x, y = rng.randint(8, 22), rng.randint(13, 22)
        d.ellipse([x, y, x + 4, y + 3], fill=EGG)
        d.point((x + 1, y + 1), fill=EGG_DARK)
    for cx, cy in ((12, 16), (19, 19)):                     # brain pieces
        d.ellipse([cx - 3, cy - 2, cx + 3, cy + 2], fill=BRAIN)
        d.arc([cx - 2, cy - 2, cx + 2, cy + 2], start=180, end=330,
              fill=BRAIN_DARK, width=1)


def d_stuffed_heart(d, rng):
    # whole heart, cut top showing the vegetable stuffing
    d.polygon([(16, 27), (8, 18), (6, 11), (9, 6), (13, 6), (16, 9),
               (19, 6), (23, 6), (26, 11), (24, 18)], fill=MEAT)
    d.polygon([(16, 27), (8, 18), (6, 11), (9, 6), (13, 6), (16, 9),
               (19, 6), (23, 6), (26, 11), (24, 18)], outline=MEAT_DARK)
    d.ellipse([11, 5, 21, 9], fill=MEAT_DARK)               # cut face
    for x, y in ((13, 6), (16, 7), (19, 6), (15, 5)):       # stuffing bits
        d.point((x, y), fill=(110, 156, 70, 255))
        d.point((x, y + 1), fill=(226, 130, 50, 255))
    d.line([10, 12, 12, 18], fill=(226, 120, 110, 255), width=1)  # sheen


def d_tottchen(d, rng):
    # bowl of chunky organ stew in a buttery wine sauce
    d.polygon([(6, 13), (26, 13), (23, 25), (9, 25)], fill=(70, 62, 58, 255))
    d.polygon([(6, 13), (26, 13), (23, 25), (9, 25)], outline=PLASTIC_DARK)
    d.ellipse([6, 10, 26, 16], fill=(196, 138, 74, 255))    # sauce surface
    for _ in range(9):                                      # meat/organ chunks
        x, y = rng.randint(8, 23), rng.randint(11, 14)
        d.ellipse([x, y, x + 3, y + 2],
                  fill=MEAT if rng.random() < 0.6 else MEAT_DARK)
    for x in (11, 17, 22):                                  # butter gloss
        d.point((x, 12), fill=(246, 214, 140, 255))
    d.rectangle([13, 26, 19, 28], fill=PLASTIC_DARK)        # bowl foot


DRAW = {
    "clinch_block": d_clinch_block,
    "clinch_cutter": d_clinch_cutter,
    "clinch_pliers": d_clinch_pliers,
    "hoof_nippers": d_hoof_nippers,
    "hoof_pick": d_hoof_pick,
    "hoof_rasp": d_hoof_rasp,
    "hoof_stand": d_hoof_stand,
    "hoof_tester": d_hoof_tester,
    "horseshoe": d_horseshoe,
    "horseshoe_pliers": d_horseshoe_pliers,
    "nail_puller": d_nail_puller,
    "pritchel": d_pritchel,
    "animal_locker": d_animal_locker,
    "basket": d_basket,
    "bike_basket": d_bike_basket,
    "bike_rack": d_bike_rack,
    "cargo_aisle": d_cargo_aisle,
    "cargo_lock": d_cargo_lock,
    "cargo_rack": d_cargo_rack,
    "folding_basket": d_folding_basket,
    "livestock_carrier": d_livestock_carrier,
    "mounted_spare_tire": d_mounted_spare_tire,
    "nose_plate": d_nose_plate,
    "armor_star_samurai_helmet_destroyed": d_destroyed_kabuto,
    "bee_sting": d_bee_sting,
    "discharged_storage_battery": d_discharged_battery,
    "discharging_battery": d_discharging_battery,
    "fighter_sting": d_fighter_sting,
    "fuse": d_fuse,
    "incendiary": d_incendiary,
    "rubber_tire_chunk": d_rubber_chunk,
    "rubber_tire_strip": d_rubber_strip,
    "sole_rubber": d_sole_rubber,
    "ugl_buttstock": d_ugl_buttstock,
    "canned_heart": d_canned_heart,
    "deviled_kidney": d_deviled_kidney,
    "fried_brain": d_fried_brain,
    "fried_tripe": d_fried_tripe,
    "homemade_hotdogs": d_homemade_hotdogs,
    "lung_provence": d_lung_provence,
    "sausage_casings": d_sausage_casings,
    "sausage_casings_artificial": d_sausage_casings_artificial,
    "scrambledeggsandbrain": d_scrambled_eggs_brain,
    "stuffed_heart": d_stuffed_heart,
    "tÖttchen": d_tottchen,
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    DRAW[sid](d, rng)
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
    # contact-sheet preview: 8 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item71.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item71.png {prev.size}")


if __name__ == "__main__":
    main()
