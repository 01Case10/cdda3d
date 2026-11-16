#!/usr/bin/env python3
"""ITEM41: science tools (34 tiles).

Self-contained: the id list is embedded below. Each sprite is drawn from
its entry in data/json/items/tool/science.json (name / description /
color / material), native 32x32, transparent background, fully opaque
bodies (alpha only {0,255}); glass and glow are faked with pale opaque
colors because this Pillow build's ImageDraw replaces rather than blends.

Run: python3 draw_item41.py   (writes only its own 34 folders + preview)
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "ammonia_machine_pipework",
    "ammonia_machine_reactor",
    "burette",
    "canister_goo",
    "closed_loop_extractor_large",
    "closed_loop_extractor_small",
    "cuvettes",
    "electrolysis_kit",
    "electrolyzer_makeshift",
    "engineering_engine_kit",
    "engineering_makerspace_kit",
    "engineering_robotics_kit",
    "glass_prism",
    "glass_thermometer",
    "glass_tube_small",
    "hygrometer",
    "hygroscope",
    "ketene_lamp",
    "lux_meter",
    "melting_point",
    "microscope_slide",
    "microscope_slide_soil",
    "microscope_slide_water_murky",
    "migo_bio_tech",
    "nitrogen_generator",
    "platinum_grille",
    "portal",
    "recovery_pump",
    "rotovap",
    "sextant",
    "still_lab",
    "teleporter",
    "vortex",
    "weather_glass",
]

HERE = os.path.dirname(os.path.abspath(__file__))
BASE = os.path.join(HERE, "pngs_normal_32x32", "items")
CREDITS = (
    "Drawn from the item's in-game description for the CDDA Voxel "
    "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset."
)

# ---- palette -------------------------------------------------------------
STEEL = (178, 182, 190)
STEEL_D = (112, 116, 126)
STEEL_DD = (66, 69, 78)
DARK = (44, 46, 54)
GLASS = (203, 230, 242)
GLASS_PALE = (228, 242, 249)
GLASS_D = (130, 172, 196)
WATER = (96, 172, 224)
CREAM = (242, 232, 204)


def B(d, box, fill, outline=None, w=1):
    d.rectangle(box, fill=fill, outline=outline, width=w)


def E(d, box, fill, outline=None, w=1):
    d.ellipse(box, fill=fill, outline=outline, width=w)


def L(d, pts, fill, w=1):
    d.line(pts, fill=fill, width=w)


def P(d, pts, fill, outline=None):
    d.polygon(pts, fill=fill, outline=outline)


def rng_for(item_id):
    return random.Random(zlib.crc32(item_id.encode()))


# ---- individual sprites ---------------------------------------------------

def draw_reactor(d, r):
    # huge steel Haber-Bosch appliance: cabinet, top pipes, round vessel door
    B(d, (7, 2, 11, 7), STEEL_D, STEEL_DD)
    B(d, (20, 2, 24, 7), STEEL_D, STEEL_DD)
    B(d, (3, 7, 28, 29), STEEL, STEEL_DD)
    B(d, (3, 7, 28, 10), STEEL_D, STEEL_DD)
    E(d, (8, 12, 22, 27), STEEL_D, STEEL_DD, 2)          # vessel door rim
    E(d, (10, 14, 20, 25), (196, 200, 208), STEEL_DD)
    E(d, (14, 18, 16, 21), STEEL_DD)                       # hub
    for x, y in ((9, 13), (21, 13), (9, 26), (21, 26)):
        E(d, (x, y, x + 1, y + 1), STEEL_DD)               # bolts
    E(d, (23, 12, 28, 17), CREAM, STEEL_DD)                # gauge
    L(d, [(25, 14), (27, 13)], (200, 60, 50), 1)
    for i in range(4):                                     # vent slots
        L(d, [(23, 21 + i * 2), (27, 21 + i * 2)], STEEL_DD, 1)
    B(d, (4, 27, 12, 28), (216, 176, 60))                  # hazard stripe


def draw_pipework(d, r):
    # thick-walled pipes, compressor block, valve wheel, wiring
    B(d, (12, 17, 21, 27), STEEL_D, STEEL_DD)              # compressor
    for i in range(3):
        L(d, [(13, 20 + i * 3), (20, 20 + i * 3)], STEEL_DD, 1)
    B(d, (2, 8, 29, 11), STEEL, STEEL_DD)                  # main horizontal
    B(d, (5, 8, 8, 26), STEEL, STEEL_DD)                   # left riser
    B(d, (23, 8, 26, 21), STEEL, STEEL_DD)                 # right riser
    B(d, (5, 24, 12, 27), STEEL, STEEL_DD)                 # bottom run
    for box in ((4, 7, 9, 12), (22, 7, 27, 12), (22, 19, 27, 23)):
        B(d, box, STEEL_D, STEEL_DD)                       # elbow blocks
    E(d, (12, 1, 18, 7), None, (178, 60, 50), 2)           # valve wheel
    L(d, [(15, 2), (15, 6)], (178, 60, 50), 1)
    L(d, [(13, 4), (17, 4)], (178, 60, 50), 1)
    L(d, [(15, 7), (15, 8)], STEEL_DD, 2)                  # valve stem
    d.arc((16, 14, 30, 26), start=300, end=80, fill=DARK, width=2)  # cable
    E(d, (27, 22, 29, 24), STEEL_D, STEEL_DD)              # junction box


def draw_burette(d, r):
    B(d, (13, 1, 18, 3), GLASS_PALE, GLASS_D)              # rim
    B(d, (14, 3, 17, 23), GLASS_PALE, GLASS_D)             # column
    B(d, (15, 10, 16, 22), WATER)                          # liquid
    for y in range(5, 22, 3):                              # graduations
        L(d, [(18, y), (19 if y % 6 == 5 else 18, y)], STEEL_DD, 1)
    B(d, (12, 23, 19, 25), STEEL_D, STEEL_DD)              # stopcock
    B(d, (17, 25, 20, 26), STEEL_DD)                       # handle
    P(d, [(15, 25), (16, 25), (16, 30)], GLASS_D)         # tip


def draw_canister_goo(d, r):
    E(d, (11, 3, 20, 8), (104, 196, 74))                   # goo bulging out
    E(d, (13, 2, 17, 5), (140, 220, 100))
    B(d, (8, 6, 23, 9), (128, 132, 142), DARK)             # lid
    B(d, (9, 9, 22, 28), (88, 90, 98), DARK)               # body
    L(d, [(9, 12), (22, 12)], DARK, 1)
    B(d, (10, 15, 21, 23), (216, 200, 90), DARK)           # warning label
    P(d, [(15, 17), (18, 21), (12, 21)], DARK)            # hazard triangle
    E(d, (15, 19, 16, 20), (216, 200, 90))
    B(d, (20, 10, 22, 15), (104, 196, 74))                 # goo drip
    E(d, (20, 15, 22, 17), (104, 196, 74))


def draw_extractor_large(d, r):
    for x in (7, 14, 21):                                  # three tubes
        B(d, (x, 5, x + 3, 14), STEEL, STEEL_DD)
        P(d, [(x - 1, 5), (x + 4, 5), (x + 3, 2), (x, 2)], STEEL_D)
    B(d, (4, 14, 27, 24), (104, 107, 115), DARK)           # horizontal drum
    for x in (9, 15, 21):
        L(d, [(x, 14), (x, 24)], STEEL_DD, 1)
    E(d, (13, 17, 18, 22), STEEL_DD)                       # port
    d.arc((22, 16, 31, 27), start=270, end=120, fill=DARK, width=2)  # hose
    B(d, (6, 24, 8, 28), STEEL_DD)
    B(d, (23, 24, 25, 28), STEEL_DD)


def draw_extractor_small(d, r):
    B(d, (13, 3, 18, 16), STEEL, STEEL_DD)                 # column tube
    B(d, (13, 6, 18, 7), STEEL_DD)                         # clamps
    B(d, (13, 12, 18, 13), STEEL_DD)
    E(d, (9, 15, 22, 19), STEEL_D, STEEL_DD)               # honey pot rim
    B(d, (9, 17, 22, 27), (104, 107, 115), DARK)           # honey pot
    B(d, (12, 21, 19, 25), (216, 176, 60))                 # extract visible
    d.arc((16, 8, 28, 20), start=270, end=90, fill=DARK, width=2)    # hose
    B(d, (10, 27, 21, 28), STEEL_DD)


def draw_cuvettes(d, r):
    for i, x in enumerate((7, 11, 15, 19, 23)):            # cuvettes in box
        B(d, (x, 6, x + 3, 15), GLASS_PALE, GLASS_D)
        if i % 2 == 0:
            B(d, (x + 1, 11, x + 2, 14), WATER)
    B(d, (5, 13, 27, 16), (96, 100, 110), DARK)            # box opening
    B(d, (5, 16, 27, 28), (202, 204, 210), STEEL_D)        # box body
    L(d, [(8, 21), (24, 21)], STEEL_D, 1)                  # label line
    L(d, [(8, 24), (19, 24)], STEEL_D, 1)


def draw_electrolysis_kit(d, r):
    d.arc((3, 3, 13, 13), start=0, end=360, fill=(200, 60, 50), width=2)
    d.arc((5, 5, 11, 11), start=0, end=360, fill=(200, 60, 50), width=2)
    d.arc((18, 3, 28, 13), start=0, end=360, fill=DARK, width=2)
    d.arc((20, 5, 26, 11), start=0, end=360, fill=DARK, width=2)
    L(d, [(8, 13), (11, 18)], (200, 60, 50), 2)           # leads down
    L(d, [(23, 13), (20, 18)], DARK, 2)
    B(d, (9, 18, 13, 29), STEEL_D, STEEL_DD)               # electrode plates
    B(d, (18, 18, 22, 29), STEEL_D, STEEL_DD)
    for y in (21, 24, 27):
        E(d, (11, y, 12, y + 1), STEEL_DD)
        E(d, (20, y, 21, y + 1), STEEL_DD)
    B(d, (9, 16, 13, 18), (200, 60, 50))                   # clips
    B(d, (18, 16, 22, 18), DARK)


def draw_electrolyzer(d, r):
    B(d, (4, 12, 28, 28), STEEL, STEEL_DD)                 # tank
    B(d, (5, 17, 27, 27), (158, 208, 236))                 # solution
    for x in (8, 20):                                      # inverted tubes
        B(d, (x, 5, x + 4, 20), GLASS_PALE, GLASS_D)
        B(d, (x + 1, 6, x + 3, 10), (240, 248, 250))       # gas space
        B(d, (x + 1, 17, x + 3, 19), STEEL_DD)             # electrode
    for _ in range(7):                                     # bubbles
        x, y = r.randint(6, 26), r.randint(19, 26)
        E(d, (x, y, x + 1, y + 1), (240, 250, 252))
    L(d, [(10, 5), (10, 2)], DARK, 2)                      # wires
    L(d, [(22, 5), (22, 2)], DARK, 2)
    L(d, [(10, 2), (22, 2)], DARK, 2)


def draw_stirling(d, r):
    B(d, (3, 24, 29, 27), (104, 164, 100), (52, 96, 52))  # base
    B(d, (6, 12, 11, 24), GLASS_PALE, GLASS_D)             # glass cylinder
    B(d, (6, 17, 11, 20), STEEL_D, STEEL_DD)               # piston
    P(d, [(7, 24), (10, 24), (8, 21)], (240, 140, 40))    # burner flame
    E(d, (14, 7, 29, 22), None, STEEL_DD, 2)               # flywheel
    for dx, dy in ((0, -6), (0, 6), (-6, 0), (6, 0), (-4, -4), (4, 4)):
        L(d, [(21, 14), (21 + dx, 14 + dy)], STEEL_D, 1)
    E(d, (20, 13, 23, 16), STEEL_DD)
    L(d, [(11, 18), (20, 14)], STEEL_DD, 2)                # connecting rod


def _kit_case(d):
    B(d, (4, 13, 28, 28), (70, 72, 80), DARK)              # case
    B(d, (4, 7, 28, 13), (86, 88, 96), DARK)               # open lid
    L(d, [(4, 13), (28, 13)], (150, 152, 160), 1)


def draw_makerspace(d, r):
    _kit_case(d)
    E(d, (7, 17, 13, 23), None, (240, 150, 50), 2)         # gear
    E(d, (9, 19, 11, 21), (240, 150, 50))
    B(d, (15, 23, 27, 25), (240, 210, 80), (150, 120, 30))  # perforated beam
    for x in range(17, 27, 3):
        E(d, (x, 23, x + 1, 24), DARK)
    B(d, (17, 16, 25, 18), (80, 190, 220), (40, 110, 140))  # hydraulic syringe
    B(d, (25, 15, 27, 19), (200, 200, 206), STEEL_DD)      # plunger
    E(d, (21, 19, 26, 24), (210, 70, 60))                  # wheel


def draw_robotics(d, r):
    B(d, (3, 17, 29, 28), (70, 72, 80), DARK)              # case behind
    L(d, [(16, 1), (16, 4)], STEEL_DD, 1)                  # antenna
    E(d, (15, 0, 17, 2), (230, 80, 70))
    B(d, (12, 4, 20, 10), (196, 200, 208), STEEL_DD)       # head
    E(d, (13, 6, 15, 8), (70, 200, 220))                   # eyes
    E(d, (17, 6, 19, 8), (70, 200, 220))
    B(d, (11, 10, 21, 19), (170, 174, 182), STEEL_DD)      # body
    B(d, (14, 13, 18, 16), (80, 190, 220), STEEL_DD)       # chest panel
    B(d, (8, 11, 11, 16), STEEL_D, STEEL_DD)               # arms
    B(d, (21, 11, 24, 16), STEEL_D, STEEL_DD)
    E(d, (9, 19, 14, 24), DARK, STEEL_DD)                  # wheels
    E(d, (18, 19, 23, 24), DARK, STEEL_DD)


def draw_prism(d, r):
    for (y2, c) in zip((9, 13, 17, 21, 25),
                       ((230, 60, 50), (240, 140, 40), (240, 210, 60),
                        (80, 190, 90), (80, 130, 230))):
        L(d, [(19, 15), (31, y2)], c, 2)                   # rainbow fan
    P(d, [(10, 26), (22, 26), (16, 7)], GLASS_PALE, GLASS_D)  # prism
    L(d, [(16, 7), (16, 26)], GLASS_D, 1)
    L(d, [(0, 10), (12, 14)], (250, 250, 250), 2)          # incoming beam
    E(d, (11, 13, 13, 15), (250, 250, 250))


def draw_glass_thermometer(d, r):
    B(d, (14, 3, 17, 24), GLASS_PALE, GLASS_D)             # tube
    E(d, (12, 22, 19, 29), GLASS_PALE, GLASS_D)            # bulb
    E(d, (13, 23, 18, 28), (216, 66, 56))                  # mercury bulb
    B(d, (15, 10, 16, 24), (216, 66, 56))                  # mercury column
    for y in range(6, 21, 3):
        L(d, [(18, y), (20, y)], STEEL_DD, 1)               # ticks


def draw_glass_tube(d, r):
    B(d, (11, 6, 20, 8), GLASS_PALE, GLASS_D)              # rim
    B(d, (12, 8, 19, 20), GLASS_PALE)
    d.arc((12, 15, 19, 24), start=0, end=180, fill=GLASS_D, width=1)
    L(d, [(12, 8), (12, 19)], GLASS_D, 1)
    L(d, [(19, 8), (19, 19)], GLASS_D, 1)
    B(d, (13, 10, 14, 21), (250, 253, 255))                # highlight


def draw_hygrometer(d, r):
    B(d, (10, 4, 22, 28), (168, 214, 240), (88, 138, 178))  # body
    B(d, (12, 8, 20, 16), (226, 240, 236), (88, 138, 178))  # screen
    P(d, [(16, 9), (18, 13), (14, 13)], (70, 150, 220))    # droplet
    E(d, (14, 12, 18, 15), (70, 150, 220))
    B(d, (12, 19, 15, 21), (88, 138, 178))                 # buttons
    B(d, (17, 19, 20, 21), (88, 138, 178))
    E(d, (15, 24, 17, 26), (88, 138, 178))


def draw_hygroscope(d, r):
    B(d, (5, 8, 27, 26), (150, 105, 64), (98, 66, 36))     # wooden box
    for y in (11, 23):
        L(d, [(6, y), (26, y)], (128, 88, 52), 1)           # grain
    E(d, (9, 10, 23, 24), CREAM, DARK, 2)                  # dial
    for a in range(0, 360, 45):
        import math
        x = 16 + int(5.2 * math.cos(a * 3.14159 / 180))
        y = 17 + int(5.2 * math.sin(a * 3.14159 / 180))
        E(d, (x, y, x, y), DARK)
    L(d, [(16, 17), (20, 13)], (190, 50, 45), 2)           # needle
    E(d, (15, 16, 17, 18), DARK)


def draw_ketene_lamp(d, r):
    B(d, (2, 13, 30, 17), GLASS_PALE, GLASS_D)             # glass pipe
    B(d, (2, 13, 5, 17), STEEL_D, STEEL_DD)                # end fittings
    B(d, (27, 13, 30, 17), STEEL_D, STEEL_DD)
    E(d, (11, 8, 21, 22), GLASS_PALE, GLASS_D)              # central bulb
    pts = []
    for i, x in enumerate(range(12, 21)):
        pts.append((x, 13 if i % 2 == 0 else 17))          # heating element
    L(d, pts, (240, 130, 40), 2)
    L(d, [(16, 22), (16, 27)], STEEL_DD, 2)                # stand
    B(d, (11, 27, 21, 28), DARK)


def draw_lux_meter(d, r):
    E(d, (11, 2, 21, 9), (246, 246, 246), STEEL_D)         # sensor dome
    B(d, (10, 9, 22, 28), (190, 192, 198), STEEL_DD)       # body
    B(d, (12, 13, 20, 19), (216, 226, 206), STEEL_DD)      # display
    E(d, (15, 14, 17, 16), (240, 200, 60))                 # sun glyph
    for dx, dy in ((-2, 0), (2, 0), (0, -2), (0, 2)):
        E(d, (16 + dx, 15 + dy, 16 + dx, 15 + dy), (240, 200, 60))
    E(d, (15, 22, 17, 24), STEEL_DD)                       # button
    B(d, (13, 26, 19, 27), STEEL_D)


def draw_melting_point(d, r):
    B(d, (6, 6, 18, 20), STEEL_D, STEEL_DD)                # housing
    E(d, (8, 9, 14, 15), (172, 220, 240), CREAM, 2)        # viewport
    E(d, (10, 11, 12, 13), (240, 140, 40))                 # hot sample
    B(d, (16, 8, 17, 18), DARK)                            # capillary slot
    L(d, [(16, 8), (16, 12)], GLASS_PALE, 1)
    B(d, (4, 20, 28, 28), (176, 179, 186), STEEL_DD)       # hot plate base
    E(d, (23, 22, 27, 26), DARK)                           # knob
    L(d, [(24, 23), (26, 25)], CREAM, 1)
    B(d, (6, 23, 12, 25), (200, 60, 50))                   # heat indicator


def _slide_base(d):
    B(d, (3, 13, 29, 19), (206, 240, 248), GLASS_D)
    B(d, (3, 13, 8, 19), (242, 249, 251), GLASS_D)          # frosted end
    L(d, [(9, 14), (28, 14)], (250, 253, 255), 1)


def draw_slide(d, r):
    _slide_base(d)


def draw_slide_soil(d, r):
    _slide_base(d)
    E(d, (13, 14, 21, 18), (122, 86, 50))
    E(d, (15, 13, 19, 16), (140, 102, 62))
    for _ in range(6):
        x, y = r.randint(13, 20), r.randint(14, 17)
        E(d, (x, y, x + 1, y + 1), (84, 58, 32))


def draw_slide_murky(d, r):
    _slide_base(d)
    E(d, (13, 14, 21, 18), (112, 132, 82))
    E(d, (15, 13, 19, 16), (128, 148, 94))
    for _ in range(6):
        x, y = r.randint(13, 20), r.randint(14, 17)
        E(d, (x, y, x, y), (70, 88, 48))


def draw_migo(d, r):
    E(d, (7, 10, 25, 24), (226, 122, 112))                 # resin body
    E(d, (4, 13, 10, 19), (214, 100, 100))                 # nodules
    E(d, (22, 9, 28, 15), (214, 100, 100))
    E(d, (17, 20, 24, 26), (200, 88, 92))
    L(d, [(10, 23), (7, 28), (10, 29)], (200, 88, 92), 3)  # tendrils
    L(d, [(20, 24), (23, 29)], (200, 88, 92), 3)
    E(d, (13, 14, 19, 20), (158, 48, 58))                  # core
    E(d, (15, 16, 17, 18), (240, 170, 150))
    d.arc((9, 11, 17, 17), start=180, end=290, fill=(248, 200, 190), width=2)


def draw_nitrogen_generator(d, r):
    B(d, (19, 1, 24, 6), STEEL_D, STEEL_DD)                # top canister
    B(d, (4, 6, 26, 29), STEEL, STEEL_DD)                  # cabinet
    L(d, [(4, 10), (26, 10)], STEEL_DD, 1)
    E(d, (7, 13, 12, 18), CREAM, STEEL_DD)                 # gauge
    L(d, [(9, 15), (11, 14)], (200, 60, 50), 1)
    B(d, (19, 12, 24, 25), STEEL_D, STEEL_DD)              # membrane tube
    for y in (16, 20, 24):
        L(d, [(19, y), (24, y)], STEEL_DD, 1)
    for i in range(4):                                     # filter vents
        L(d, [(6, 21 + i * 2), (12, 21 + i * 2)], STEEL_DD, 1)
    B(d, (14, 14, 17, 18), (96, 172, 224), STEEL_DD)       # filter window


def draw_platinum_grille(d, r):
    B(d, (5, 7, 27, 25), (212, 214, 220), (118, 120, 128))  # frame
    for x in range(8, 27, 3):
        L(d, [(x, 8), (x, 24)], (150, 152, 160), 1)         # mesh
    for y in range(10, 25, 3):
        L(d, [(6, y), (26, y)], (150, 152, 160), 1)
    L(d, [(8, 22), (17, 9)], (238, 239, 242), 3)            # sheen
    B(d, (5, 7, 27, 25), None, (90, 92, 100), 2)


def draw_portal(d, r):
    E(d, (9, 2, 23, 19), (52, 26, 74), (232, 62, 202), 3)  # portal ring
    d.arc((11, 5, 21, 16), start=40, end=250, fill=(246, 150, 230), width=2)
    for x, y in ((9, 4), (22, 4), (7, 10), (24, 10), (10, 17), (21, 17)):
        E(d, (x, y, x + 1, y + 1), (246, 180, 238))         # alien markings
    B(d, (6, 20, 26, 28), (74, 62, 96), DARK)               # base unit
    B(d, (8, 22, 12, 24), (232, 62, 202))                   # glow panel
    E(d, (20, 22, 23, 25), (150, 220, 120))


def draw_recovery_pump(d, r):
    L(d, [(12, 7), (12, 4), (20, 4), (20, 7)], DARK, 2)     # handle
    B(d, (6, 8, 15, 13), STEEL_D, STEEL_DD)                # cylinder head
    for x in range(8, 15, 2):
        L(d, [(x, 8), (x, 13)], STEEL_DD, 1)               # cooling fins
    B(d, (6, 13, 26, 26), (96, 98, 106), DARK)             # motor housing
    L(d, [(6, 17), (26, 17)], DARK, 1)
    E(d, (22, 19, 26, 23), STEEL_DD)                       # hose port
    E(d, (23, 20, 25, 22), DARK)
    B(d, (9, 20, 12, 22), (200, 60, 50))                   # switch
    B(d, (7, 26, 11, 28), DARK)                            # feet
    B(d, (21, 26, 25, 28), DARK)


def draw_rotovap(d, r):
    L(d, [(22, 4), (22, 28)], STEEL_DD, 2)                 # stand rod
    B(d, (19, 28, 29, 29), DARK)                           # base
    B(d, (13, 5, 21, 11), (122, 126, 134), STEEL_DD)       # motor unit
    E(d, (4, 20, 13, 28), STEEL_D, STEEL_DD)               # heating bath
    B(d, (5, 20, 12, 22), (240, 170, 90))                  # hot liquid rim
    E(d, (5, 12, 13, 21), GLASS_PALE, GLASS_D)             # rotating flask
    L(d, [(12, 13), (15, 9)], GLASS_D, 3)                  # flask neck
    L(d, [(19, 11), (24, 8)], GLASS_D, 3)                  # vapor duct
    B(d, (23, 8, 27, 23), GLASS_PALE, GLASS_D)             # condenser
    pts = [(24, 10), (26, 13), (24, 16), (26, 19), (24, 22)]
    L(d, pts, (110, 180, 225), 1)                          # coil
    E(d, (22, 23, 28, 29), GLASS_PALE, GLASS_D)            # receiving flask
    B(d, (23, 26, 27, 28), (170, 215, 235))


def draw_sextant(d, r):
    d.arc((-4, -16, 36, 24), start=38, end=142, fill=STEEL_DD, width=3)
    L(d, [(16, 4), (4, 19)], STEEL_DD, 3)                  # arms
    L(d, [(16, 4), (28, 19)], STEEL_DD, 3)
    L(d, [(9, 12), (23, 12)], STEEL_D, 2)                   # cross strut
    L(d, [(16, 4), (10, 20)], (90, 92, 100), 2)            # index arm
    B(d, (14, 2, 18, 6), STEEL_D, STEEL_DD)                # mirror at apex
    P(d, [(20, 7), (27, 10), (26, 13), (19, 10)], (150, 152, 160))  # telescope
    for x in range(6, 27, 4):                              # scale ticks
        E(d, (x, 21, x, 21), STEEL_DD)
    E(d, (15, 3, 17, 5), DARK)                             # pivot


def draw_still(d, r):
    P(d, [(7, 29), (10, 29), (8, 26)], (240, 140, 40))    # flame
    E(d, (4, 19, 13, 28), GLASS_PALE, GLASS_D)             # boiler flask
    B(d, (5, 24, 12, 27), (232, 178, 92))                  # liquid
    B(d, (8, 6, 11, 20), GLASS_PALE, GLASS_D)              # fraction column
    for y in (9, 12, 15, 18):
        L(d, [(8, y), (11, y)], GLASS_D, 1)                # bubble plates
    L(d, [(9, 6), (9, 3)], STEEL_DD, 2)                    # thermometer
    L(d, [(11, 8), (23, 15)], GLASS_D, 4)                  # condenser
    L(d, [(11, 8), (23, 15)], (170, 214, 236), 2)
    L(d, [(18, 17), (20, 20)], GLASS_D, 2)                 # drip tip
    E(d, (21, 19, 28, 27), GLASS_PALE, GLASS_D)            # receiver
    B(d, (22, 24, 27, 26), (170, 215, 235))


def draw_teleporter(d, r):
    E(d, (12, 2, 20, 8), (202, 204, 210), STEEL_D)         # emitter dish
    L(d, [(16, 8), (16, 10)], STEEL_DD, 2)
    B(d, (11, 10, 21, 27), (202, 62, 172), (128, 28, 108))  # body
    E(d, (13, 12, 19, 18), (250, 175, 240), (128, 28, 108))  # glowing core
    E(d, (15, 14, 17, 16), (120, 20, 100))
    E(d, (13, 21, 15, 23), (250, 220, 120))                # buttons
    E(d, (17, 21, 19, 23), (120, 220, 160))
    for y in (24, 26):
        L(d, [(12, y), (20, y)], (150, 40, 128), 1)        # grip lines


def draw_vortex(d, r):
    B(d, (14, 2, 18, 9), GLASS_PALE, GLASS_D)              # test tube
    B(d, (14, 6, 18, 9), (212, 82, 72))                    # sample
    B(d, (12, 8, 20, 12), (50, 50, 56), DARK)              # rubber receptacle
    P(d, [(11, 12), (21, 12), (25, 28), (7, 28)], (152, 192, 222))  # body
    P(d, [(11, 12), (21, 12), (25, 28), (7, 28)], None)
    d.line([(11, 12), (21, 12), (25, 28), (7, 28), (11, 12)], fill=(70, 110, 150), width=1)
    E(d, (12, 19, 18, 25), CREAM, STEEL_DD)                # speed dial
    L(d, [(15, 22), (17, 20)], (200, 60, 50), 1)
    B(d, (20, 20, 23, 23), DARK)                           # switch


def draw_weather_glass(d, r):
    B(d, (6, 28, 26, 29), (110, 78, 48))                   # base
    E(d, (7, 12, 21, 28), GLASS_PALE, GLASS_D)             # sealed body
    E(d, (8, 20, 20, 27), WATER)                          # water in body
    B(d, (12, 6, 16, 13), GLASS_PALE, GLASS_D)             # neck
    B(d, (22, 9, 24, 25), GLASS_PALE, GLASS_D)             # open spout
    B(d, (22, 15, 24, 24), WATER)                          # spout level (higher)
    L(d, [(21, 24), (22, 22)], GLASS_D, 2)                 # spout join


DRAW = {
    "ammonia_machine_pipework": draw_pipework,
    "ammonia_machine_reactor": draw_reactor,
    "burette": draw_burette,
    "canister_goo": draw_canister_goo,
    "closed_loop_extractor_large": draw_extractor_large,
    "closed_loop_extractor_small": draw_extractor_small,
    "cuvettes": draw_cuvettes,
    "electrolysis_kit": draw_electrolysis_kit,
    "electrolyzer_makeshift": draw_electrolyzer,
    "engineering_engine_kit": draw_stirling,
    "engineering_makerspace_kit": draw_makerspace,
    "engineering_robotics_kit": draw_robotics,
    "glass_prism": draw_prism,
    "glass_thermometer": draw_glass_thermometer,
    "glass_tube_small": draw_glass_tube,
    "hygrometer": draw_hygrometer,
    "hygroscope": draw_hygroscope,
    "ketene_lamp": draw_ketene_lamp,
    "lux_meter": draw_lux_meter,
    "melting_point": draw_melting_point,
    "microscope_slide": draw_slide,
    "microscope_slide_soil": draw_slide_soil,
    "microscope_slide_water_murky": draw_slide_murky,
    "migo_bio_tech": draw_migo,
    "nitrogen_generator": draw_nitrogen_generator,
    "platinum_grille": draw_platinum_grille,
    "portal": draw_portal,
    "recovery_pump": draw_recovery_pump,
    "rotovap": draw_rotovap,
    "sextant": draw_sextant,
    "still_lab": draw_still,
    "teleporter": draw_teleporter,
    "vortex": draw_vortex,
    "weather_glass": draw_weather_glass,
}


def main():
    assert set(DRAW) == set(IDS), set(IDS) ^ set(DRAW)
    for item_id in IDS:
        img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        DRAW[item_id](d, rng_for(item_id))
        assert img.size == (32, 32)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (item_id, alphas)
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            f.write(json.dumps([{"id": item_id, "fg": item_id, "bg": ""}],
                               indent=2) + "\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS + "\n")
    # contact-sheet preview: 6 columns, 2x nearest-neighbour on dark ground
    cols, cell = 6, 66
    rows = (len(IDS) + cols - 1) // cols
    prev = Image.new("RGBA", (cols * cell, rows * cell), (38, 38, 44, 255))
    for i, item_id in enumerate(IDS):
        img = Image.open(os.path.join(BASE, item_id, item_id + ".png"))
        img = img.resize((64, 64), Image.NEAREST)
        prev.alpha_composite(img, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    prev.convert("RGB").save(os.path.join(HERE, "preview_item41.png"))
    print("wrote %d sprites + preview_item41.png (%d, %d)"
          % (len(IDS), prev.width, prev.height))


if __name__ == "__main__":
    main()
