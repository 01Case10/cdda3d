#!/usr/bin/env python3
"""
Draw the missing vehicle part sprites (batch VEH3) from their game
descriptions.

Sources (fork-src/cdda/data/json/vehicleparts/ + appliances.json +
obsoletion_and_migration_0.J/stowed_parts.json, CDDA 6a3e59e0) -- the
49 ids below are vehicle parts missing from UltiCa in the shipped
universe (coverage-audit/missing.json) with no folder yet under
new-tiles/.  Each is drawn as the part itself, an object on a
transparent background, in flat UltiCa style at native 32x32
(vehicle parts present at tile size; no sheet-size rule for this
category):

  large/medium_storage_battery   yellow battery boxes, size-graded
  *_removable                    same on a quick-release frame
  storage_battery_mount          empty battery case + harness + chip
  medium/small_pressure_tank     propane gas tanks, capsule + bottle
  metal_bench                    red steel bench with restraints
  metal_tray                     flat sheet-metal cargo tray
  motorcycle_headlight           round headlight, white
  motorcycle_kickstand           angled stand leg with spring
  mountable_cooler               light-cyan roof cooler unit
  nose_plate                     L-shaped ally plate, straps, skids
  null                           "null part" placeholder -> DEBUG tile
  pedal_extenders                rubber pedal pads on clamps
  pedal_extenders_makeshift      wood blocks taped to the pedals
  plating_chitin                 yellow segmented chitin plate
  plating_acidchitin             same plate, silica sheen (acid-proof)
  plugged_3in_ordnance_rifle     display cannon, barrel concreted
  power_cord                     yellow cord coil + plug
  programmable_autopilot         light-blue computer, route on screen
  rail_wheel / _steerable        flanged steel rail wheels
  rail_wheel_small_pair          two small flanged wheels on an axle
  rebar_plate                    welded rebar grid armor
  roof_wood                      wooden plank roof panel
  sail                           mast, boom and cream sail
  small_civilian_rotors          four light-blue helicopter blades
  smart_engine_controller        electronics module, chip + LEDs
  solar_reflector                faceted cooking dish, yellow
  spring_plate                   scrap plate on coil springs
  stowed_*                       the machine itself, tied down with
                                 ratchet straps: air compressor
                                 (black/yellow), arcade machine (red),
                                 bandsaw (yellow/cyan), drill press
                                 (yellow/red), fridges/freezers told
                                 apart by form (upright / chest /
                                 display glass / glass door, sized
                                 mini -> household -> food-truck), the
                                 golden monolith (whole and broken),
                                 and the broken Yrax quadraphract and
                                 its snapped leg

Bodies are fully opaque (alpha only {0, 255}); translucency is
faked with pale opaque colors.  Deterministic: per-sprite
random.Random(crc32(id)).
"""

import json
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.dirname(os.path.abspath(__file__))
SHEET = os.path.join(BASE, "pngs_normal_32x32", "vehicle")
S = 32

CREDITS = (
    "Drawn from the vehicle part's in-game description for the CDDA "
    "Voxel coverage audit (2026-11-15). License: CC BY-SA 3.0, as "
    "the tileset."
)


def canvas():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))


# ---------------------------------------------------------------- helpers
def R(d, box, fill, outline=None, w=1):
    d.rectangle(box, fill=fill, outline=outline, width=w)


def RR(d, box, r, fill, outline=None, w=1):
    d.rounded_rectangle(box, radius=r, fill=fill, outline=outline, width=w)


def E(d, box, fill, outline=None, w=1):
    d.ellipse(box, fill=fill, outline=outline, width=w)


def P(d, pts, fill, outline=None):
    d.polygon(pts, fill=fill, outline=outline)


def L(d, pts, fill, w=1):
    d.line(pts, fill=fill, width=w)


def speckle(d, rng, box, color, n):
    x0, y0, x1, y1 = box
    for _ in range(n):
        x = rng.randint(x0, x1)
        y = rng.randint(y0, y1)
        d.point((x, y), fill=color)


STEEL = (148, 152, 158, 255)
STEEL_D = (92, 96, 102, 255)
STEEL_L = (188, 193, 198, 255)
IRON = (58, 60, 64, 255)
IRON_L = (84, 87, 92, 255)
RUST = (146, 84, 50, 255)
BRASS = (186, 152, 82, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (110, 76, 42, 255)
WOOD_L = (178, 138, 88, 255)
BATT_Y = (206, 172, 44, 255)      # battery yellow
BATT_YD = (158, 128, 30, 255)
BATT_YL = (232, 200, 92, 255)
STRAP = (198, 112, 42, 255)       # ratchet-strap webbing
STRAP_D = (150, 82, 28, 255)
APPL = (196, 208, 216, 255)       # appliance enamel, pale blue-white
APPL_D = (140, 156, 168, 255)
APPL_L = (226, 236, 242, 255)
GLASS = (168, 208, 224, 255)      # pale opaque "glass"
GLASS_D = (120, 164, 186, 255)


def straps(d, x0, y0, x1, y1):
    """Two vertical ratchet straps over a stowed machine's front."""
    for fx in (0.27, 0.70):
        sx = int(x0 + (x1 - x0) * fx)
        R(d, (sx - 1, y0, sx + 1, y1), STRAP, STRAP_D)
        R(d, (sx - 2, (y0 + y1) // 2 - 2, sx + 2, (y0 + y1) // 2 + 2),
          IRON, IRON_L)  # buckle


# ---------------------------------------------------------------- batteries
def battery_body(d, rng, x0, y0, x1, y1):
    R(d, (x0, y0 + 3, x1, y1), BATT_Y, BATT_YD)
    R(d, (x0, y0 + 3, x1, y0 + 6), BATT_YL)               # top rim
    R(d, (x0 + 2, y0, x0 + 5, y0 + 3), IRON)              # - terminal
    R(d, (x1 - 5, y0, x1 - 2, y0 + 3), (178, 52, 40, 255))  # + terminal
    R(d, (x0 + 3, y0 + 10, x1 - 3, y0 + 14), BATT_YD)     # label band
    R(d, (x0 + 5, y0 + 11, x1 - 5, y0 + 12), BATT_YL)
    speckle(d, rng, (x0 + 2, y1 - 4, x1 - 2, y1 - 1), BATT_YD, 5)


def draw_large_storage_battery(d, rng):
    battery_body(d, rng, 3, 7, 28, 27)


def draw_medium_storage_battery(d, rng):
    battery_body(d, rng, 6, 10, 25, 25)


def removable_frame(d, x0, y0, x1, y1):
    # quick-release rails + side clamps + red release lever
    R(d, (x0 - 1, y1, x1 + 1, y1 + 2), STEEL_D, IRON)
    R(d, (x0 - 2, y0 + 8, x0, y1), STEEL, STEEL_D)
    R(d, (x1, y0 + 8, x1 + 2, y1), STEEL, STEEL_D)
    L(d, [(x1 + 2, y0 + 10), (x1 + 4, y0 + 6)], (178, 52, 40, 255), 2)


def draw_large_storage_battery_removable(d, rng):
    battery_body(d, rng, 3, 5, 28, 25)
    removable_frame(d, 3, 5, 28, 25)


def draw_storage_battery_removable(d, rng):
    battery_body(d, rng, 5, 8, 26, 24)
    removable_frame(d, 5, 8, 26, 24)


def draw_storage_battery_mount(d, rng):
    # empty case: dark interior, rim, harness straps, controller + wires
    R(d, (5, 9, 26, 27), IRON, STEEL_D)
    R(d, (7, 11, 24, 25), (30, 30, 34, 255))
    R(d, (5, 9, 26, 11), STEEL, STEEL_D)
    L(d, [(8, 12), (8, 24)], STRAP_D, 2)                  # harness straps
    L(d, [(23, 12), (23, 24)], STRAP_D, 2)
    R(d, (12, 4, 19, 9), (52, 110, 62, 255), IRON)        # controller chip
    R(d, (14, 5, 17, 7), IRON)
    L(d, [(13, 9), (11, 12)], (178, 52, 40, 255), 1)      # wires
    L(d, [(18, 9), (21, 12)], BATT_Y, 1)
    R(d, (4, 27, 27, 29), STEEL_D, IRON)                  # base rails


# ---------------------------------------------------------------- gas tanks
def draw_medium_pressure_tank(d, rng):
    # horizontal propane capsule
    RR(d, (3, 11, 28, 22), 5, STEEL_L, STEEL_D)
    R(d, (8, 11, 10, 22), STEEL, STEEL_D)                  # weld bands
    R(d, (21, 11, 23, 22), STEEL, STEEL_D)
    R(d, (14, 7, 17, 11), STEEL_D, IRON)                  # valve neck
    E(d, (13, 4, 18, 8), BRASS, STEEL_D)                  # valve wheel
    R(d, (6, 22, 9, 25), STEEL_D)                         # feet
    R(d, (22, 22, 25, 25), STEEL_D)
    speckle(d, rng, (5, 18, 26, 21), STEEL, 6)


def draw_small_pressure_tank(d, rng):
    # small upright propane bottle
    RR(d, (10, 10, 21, 27), 4, STEEL_L, STEEL_D)
    R(d, (10, 17, 21, 19), STEEL, STEEL_D)                 # band
    R(d, (14, 6, 17, 10), STEEL_D, IRON)
    E(d, (13, 3, 18, 7), BRASS, STEEL_D)
    L(d, [(11, 6), (20, 6)], STEEL_D, 1)                  # guard collar
    speckle(d, rng, (12, 22, 19, 26), STEEL, 4)


# ---------------------------------------------------------------- parts
def draw_metal_bench(d, rng):
    RED = (168, 52, 44, 255)
    RED_D = (116, 34, 30, 255)
    # backrest slats
    for i in range(3):
        R(d, (5, 3 + i * 4, 26, 5 + i * 4), RED, RED_D)
    R(d, (5, 3, 7, 15), RED_D)
    R(d, (24, 3, 26, 15), RED_D)
    # seat slats
    for i in range(3):
        R(d, (4, 16 + i * 3, 27, 17 + i * 3), RED, RED_D)
    R(d, (5, 24, 7, 29), STEEL_D)                         # legs
    R(d, (24, 24, 26, 29), STEEL_D)
    # wrist / ankle restraints: dark straps with buckles
    for x in (8, 21):
        R(d, (x, 12, x + 3, 19), IRON, IRON_L)
        R(d, (x, 14, x + 3, 16), STEEL_L)
    for x in (9, 20):
        L(d, [(x, 24), (x, 28)], IRON, 2)


def draw_metal_tray(d, rng):
    P(d, [(5, 12), (27, 12), (29, 24), (3, 24)], STEEL, STEEL_D)
    P(d, [(7, 14), (25, 14), (26, 22), (6, 22)], STEEL_L, STEEL)
    L(d, [(8, 17), (24, 17)], STEEL, 1)
    L(d, [(7, 20), (25, 20)], STEEL, 1)
    R(d, (3, 10, 29, 12), STEEL_D, IRON)                  # back rim
    speckle(d, rng, (8, 15, 24, 21), STEEL, 7)


def draw_motorcycle_headlight(d, rng):
    E(d, (8, 6, 24, 22), IRON, STEEL_D)                   # housing
    E(d, (10, 8, 22, 20), (238, 240, 220, 255), STEEL)    # lens
    E(d, (13, 10, 18, 15), (255, 255, 255, 255))          # hotspot
    L(d, [(12, 18), (20, 11)], GLASS_D, 1)                # reflector facet
    R(d, (14, 22, 18, 25), STEEL_D)                       # bracket
    R(d, (6, 12, 8, 16), STEEL_D)
    R(d, (24, 12, 26, 16), STEEL_D)
    # pale beam hints (opaque, faked translucency)
    P(d, [(23, 10), (29, 8), (29, 12)], (226, 228, 196, 255))
    P(d, [(23, 16), (29, 16), (29, 20)], (226, 228, 196, 255))


def draw_motorcycle_kickstand(d, rng):
    R(d, (13, 3, 19, 7), IRON, STEEL_D)                   # pivot mount
    E(d, (14, 4, 17, 7), STEEL_L)
    L(d, [(15, 7), (10, 24)], STEEL, 3)                   # leg
    L(d, [(15, 7), (10, 24)], STEEL_L, 1)
    for i in range(4):                                    # spring coils
        y = 9 + i * 3
        L(d, [(16, y), (19, y + 2)], STEEL_D, 1)
    P(d, [(6, 24), (14, 24), (15, 27), (5, 27)], STEEL_D, IRON)  # foot


def draw_mountable_cooler(d, rng):
    CY = (126, 196, 208, 255)
    CY_D = (84, 148, 162, 255)
    RR(d, (6, 8, 26, 26), 2, CY, CY_D)
    R(d, (6, 8, 26, 11), CY_D)                            # top band
    E(d, (11, 14, 21, 24), CY_D, (52, 96, 108, 255))      # fan grill
    E(d, (14, 17, 18, 21), (52, 96, 108, 255))
    for a in (0, 60, 120):
        pass
    L(d, [(16, 15), (16, 23)], (52, 96, 108, 255), 1)
    L(d, [(12, 19), (20, 19)], (52, 96, 108, 255), 1)
    R(d, (22, 13, 25, 16), BATT_YD)                       # control knob
    R(d, (7, 26, 10, 28), STEEL_D)                        # feet
    R(d, (22, 26, 25, 28), STEEL_D)


def draw_nose_plate(d, rng):
    AL = (178, 182, 186, 255)
    AL_D = (128, 132, 138, 255)
    R(d, (7, 4, 11, 22), AL, AL_D)                        # vertical plate
    P(d, [(7, 22), (26, 22), (28, 26), (7, 26)], AL, AL_D)  # skid base
    L(d, [(10, 24), (26, 24)], AL_D, 1)                   # skid runners
    L(d, [(10, 25), (27, 25)], AL_D, 1)
    for y in (8, 15):                                     # straps on plate
        R(d, (6, y, 12, y + 3), STRAP, STRAP_D)
        R(d, (8, y, 10, y + 3), IRON)
    speckle(d, rng, (8, 5, 10, 21), AL_D, 4)


FONT = {
    "N": ("1001", "1101", "1011", "1001", "1001"),
    "U": ("1001", "1001", "1001", "1001", "0110"),
    "L": ("1000", "1000", "1000", "1000", "1111"),
}
QMASK = ("01110", "10001", "00001", "00010",
         "00100", "00000", "00100")


def draw_null(d, rng):
    # "null part": hidden placeholder part -> debug tile treatment
    R(d, (3, 3, 28, 28), (12, 12, 14, 255), (200, 40, 36, 255))
    RED = (216, 52, 44, 255)
    for r, row in enumerate(QMASK):                       # big "?"
        for c, bit in enumerate(row):
            if bit == "1":
                R(d, (11 + c * 2, 5 + r * 2, 12 + c * 2, 6 + r * 2), RED)
    x = 7                                                 # tiny "NULL"
    for ch in "NULL":
        for r, row in enumerate(FONT[ch]):
            for c, bit in enumerate(row):
                if bit == "1":
                    d.point((x + c, 22 + r), fill=RED)
        x += 5


def pedal_pair(d, rng, pad_fn):
    for i, x in enumerate((7, 19)):
        L(d, [(x + 3, 4), (x + 3, 12)], STEEL_D, 2)       # pedal arm
        pad_fn(d, x, 12)


def draw_pedal_extenders(d, rng):
    def pad(d, x, y):
        RR(d, (x - 2, y, x + 8, y + 10), 2, (40, 40, 44, 255), IRON)
        for gy in range(3):                               # rubber tread
            L(d, [(x, y + 2 + gy * 3), (x + 6, y + 2 + gy * 3)],
              IRON_L, 1)
        R(d, (x - 2, y + 4, x + 8, y + 6), STEEL, STEEL_D)  # clamp band
    pedal_pair(d, rng, pad)


def draw_pedal_extenders_makeshift(d, rng):
    def pad(d, x, y):
        R(d, (x - 2, y, x + 8, y + 10), WOOD, WOOD_D)      # wood block
        L(d, [(x - 1, y + 3), (x + 7, y + 3)], WOOD_D, 1)
        for tx in (x - 2, x + 3):                         # tape wraps
            R(d, (tx, y - 1, tx + 2, y + 11), (196, 196, 200, 255),
              STEEL_D)
    pedal_pair(d, rng, pad)


def chitin_plate(d, rng, acid):
    CH = (196, 158, 52, 255)
    CH_D = (146, 112, 34, 255)
    CH_L = (224, 190, 96, 255)
    RR(d, (5, 6, 27, 26), 3, CH, CH_D)
    # overlapping segment arcs
    for row in range(3):
        y = 9 + row * 6
        for col in range(3):
            x = 7 + col * 7
            d.arc((x, y, x + 8, y + 8), 0, 180, fill=CH_D, width=1)
            d.point((x + 3, y + 2), fill=CH_L)
    if acid:
        # biosilicified sheen: pale glossy patches + acid-green tint
        for (x, y) in ((9, 10), (19, 15), (12, 21), (22, 9)):
            E(d, (x, y, x + 4, y + 2), (238, 240, 214, 255))
        speckle(d, rng, (7, 8, 25, 24), (150, 186, 92, 255), 10)
    speckle(d, rng, (7, 8, 25, 24), CH_D, 6)


def draw_plating_chitin(d, rng):
    chitin_plate(d, rng, acid=False)


def draw_plating_acidchitin(d, rng):
    chitin_plate(d, rng, acid=True)


def draw_plugged_3in_ordnance_rifle(d, rng):
    # barrel, breech block and carriage cheek; muzzle concreted shut
    L(d, [(4, 13), (24, 13)], IRON, 6)
    L(d, [(4, 13), (24, 13)], IRON_L, 2)
    R(d, (2, 9, 7, 18), IRON, STEEL_D)                    # breech
    E(d, (3, 11, 6, 15), STEEL_D)
    R(d, (20, 10, 24, 16), STEEL_D, IRON)                 # muzzle swell
    R(d, (24, 10, 28, 16), (168, 164, 156, 255), STEEL_D)  # concrete plug
    speckle(d, rng, (25, 11, 27, 15), STEEL_D, 5)
    P(d, [(8, 16), (16, 16), (19, 24), (6, 24)], (96, 74, 48, 255),
      WOOD_D)                                             # carriage
    E(d, (9, 22, 15, 28), WOOD_D, IRON)                   # wheel hint
    E(d, (11, 24, 13, 26), IRON)


def draw_power_cord(d, rng):
    # coiled yellow cord + two-prong plug + appliance stub
    R(d, (12, 2, 20, 6), STEEL_D, IRON)
    for i, r in enumerate((9, 7, 5)):
        box = (16 - r, 8 + i * 2, 16 + r, 20 + i * 2)
        E(d, box, None, outline=BATT_Y)
        E(d, (box[0] + 1, box[1], box[2] - 1, box[3]), None,
          outline=BATT_YD)
    R(d, (13, 24, 19, 28), (40, 40, 44, 255), IRON)       # plug body
    R(d, (14, 28, 15, 30), STEEL_L)                       # prongs
    R(d, (17, 28, 18, 30), STEEL_L)
    L(d, [(16, 20), (16, 24)], BATT_YD, 2)


def draw_programmable_autopilot(d, rng):
    BL = (120, 168, 216, 255)
    BL_D = (76, 116, 158, 255)
    RR(d, (5, 7, 27, 25), 2, BL, BL_D)
    R(d, (8, 10, 20, 19), (24, 34, 44, 255), BL_D)        # screen
    L(d, [(10, 17), (13, 12), (16, 15), (19, 11)],
      (96, 220, 140, 255), 1)                             # route polyline
    d.point((10, 17), fill=(255, 255, 255, 255))
    d.point((19, 11), fill=(255, 120, 100, 255))
    for i, c in enumerate(((220, 80, 70, 255), (240, 200, 80, 255),
                           (110, 220, 130, 255))):
        E(d, (22, 10 + i * 4, 24, 12 + i * 4), c, BL_D)   # LEDs
    R(d, (8, 21, 20, 23), BL_D)                           # button row
    L(d, [(16, 25), (16, 29)], IRON, 2)                   # cable


def rail_wheel(d, cx, cy, r):
    E(d, (cx - r, cy - r, cx + r, cy + r), STEEL, STEEL_D)
    E(d, (cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2),
      STEEL_L, STEEL)
    for dx, dy in ((0, -r + 3), (0, r - 3), (-r + 3, 0), (r - 3, 0)):
        d.point((cx + dx, cy + dy), fill=STEEL_D)
    E(d, (cx - 2, cy - 2, cx + 2, cy + 2), STEEL_D, IRON)  # hub


def draw_rail_wheel(d, rng):
    # flange: larger thin ring offset behind the tread
    E(d, (5, 7, 25, 27), STEEL_D, IRON)
    rail_wheel(d, 16, 16, 9)
    speckle(d, rng, (9, 9, 23, 23), STEEL_D, 5)


def draw_rail_wheel_steerable(d, rng):
    E(d, (7, 9, 27, 29), STEEL_D, IRON)
    rail_wheel(d, 18, 18, 9)
    L(d, [(18, 9), (18, 4)], STEEL_D, 2)                  # steering arm
    L(d, [(8, 4), (26, 4)], STEEL, 2)                     # tie rod
    E(d, (7, 3, 9, 5), IRON)
    E(d, (25, 3, 27, 5), IRON)


def draw_rail_wheel_small_pair(d, rng):
    L(d, [(6, 16), (26, 16)], STEEL_D, 3)                  # axle
    for cx in (10, 22):
        E(d, (cx - 6, 10, cx + 6, 22), STEEL_D, IRON)     # flange
        rail_wheel(d, cx, 16, 5)
    R(d, (4, 24, 28, 26), (120, 96, 70, 255), WOOD_D)      # rail hint


def draw_rebar_plate(d, rng):
    for x in range(6, 27, 4):                             # vertical rods
        L(d, [(x, 5), (x, 27)], STEEL_D, 2)
        for y in range(7, 27, 5):
            d.point((x - 1, y), fill=STEEL)
    for y in range(6, 27, 5):                             # horizontal rods
        L(d, [(5, y), (27, y)], STEEL_D, 2)
    for (x, y) in ((9, 10), (17, 16), (25, 10), (13, 22), (21, 26)):
        R(d, (x, y, x + 2, y + 1), IRON)                  # weld blobs
    speckle(d, rng, (6, 6, 26, 26), RUST, 7)


def draw_roof_wood(d, rng):
    R(d, (4, 8, 28, 24), WOOD, WOOD_D)
    for i in range(3):
        y = 8 + i * 5
        L(d, [(4, y), (28, y)], WOOD_D, 1)
        L(d, [(6, y + 2), (20, y + 2)], WOOD_L, 1)
        L(d, [(14, y + 3), (26, y + 3)], WOOD_D, 1)
    for (x, y) in ((6, 10), (26, 10), (6, 21), (26, 21)):
        d.point((x, y), fill=IRON)                        # nails
    speckle(d, rng, (6, 9, 26, 23), WOOD_D, 6)


def draw_sail(d, rng):
    L(d, [(10, 3), (10, 27)], WOOD_D, 2)                   # mast
    L(d, [(10, 24), (26, 24)], WOOD_D, 2)                  # boom
    P(d, [(12, 5), (12, 22), (25, 22)], (238, 230, 208, 255),
      (198, 186, 158, 255))                               # sail cloth
    L(d, [(12, 12), (19, 12)], (198, 186, 158, 255), 1)   # seam
    L(d, [(12, 18), (22, 18)], (198, 186, 158, 255), 1)
    L(d, [(10, 3), (25, 22)], (120, 110, 96, 255), 1)      # rigging
    R(d, (4, 27, 28, 29), WOOD_D)                         # hull hint


def draw_small_civilian_rotors(d, rng):
    LB = (168, 200, 224, 255)
    LB_D = (110, 148, 178, 255)
    for ang_pts in (((16, 3), (19, 14), (13, 14)),        # blade up
                    ((16, 29), (13, 18), (19, 18)),       # blade down
                    ((3, 16), (14, 13), (14, 19)),        # blade left
                    ((29, 16), (18, 19), (18, 13))):      # blade right
        P(d, list(ang_pts), LB, LB_D)
    E(d, (12, 12, 20, 20), STEEL, STEEL_D)                 # hub
    E(d, (14, 14, 18, 18), STEEL_D, IRON)
    speckle(d, rng, (13, 13, 19, 19), STEEL_D, 3)


def draw_smart_engine_controller(d, rng):
    RR(d, (6, 9, 26, 24), 2, IRON, STEEL_D)               # module case
    R(d, (9, 12, 18, 19), (52, 110, 62, 255), IRON)       # PCB window
    R(d, (11, 13, 16, 18), (30, 30, 34, 255), STEEL_D)    # main chip
    for i in range(3):
        d.point((12 + i * 2, 14), fill=STEEL_L)
    for i, c in enumerate(((110, 220, 130, 255), (240, 200, 80, 255),
                           (220, 80, 70, 255))):
        E(d, (20, 12 + i * 4, 23, 14 + i * 4), c, IRON)   # status LEDs
    L(d, [(10, 24), (9, 28)], (178, 52, 40, 255), 1)      # wire stubs
    L(d, [(14, 24), (14, 28)], BATT_Y, 1)
    L(d, [(18, 24), (19, 28)], STEEL_L, 1)


def draw_solar_reflector(d, rng):
    YL = (232, 204, 92, 255)
    YL_D = (186, 156, 58, 255)
    # faceted parabolic dish
    P(d, [(4, 6), (28, 6), (23, 16), (9, 16)], YL, YL_D)
    for x in (10, 16, 22):
        L(d, [(x, 6), (x - 1, 16)], YL_D, 1)
    L(d, [(4, 11), (28, 11)], YL_D, 1)
    for (x, y) in ((8, 8), (20, 8), (14, 13)):
        d.point((x, y), fill=(250, 240, 200, 255))        # glints
    L(d, [(16, 16), (16, 20)], STEEL_D, 2)                # pot arm
    R(d, (12, 19, 20, 23), IRON, STEEL_D)                 # cooking pot
    L(d, [(10, 23), (7, 28)], STEEL_D, 2)                 # tripod legs
    L(d, [(22, 23), (25, 28)], STEEL_D, 2)
    L(d, [(16, 23), (16, 28)], STEEL_D, 2)


def draw_spring_plate(d, rng):
    R(d, (5, 6, 27, 11), STEEL, STEEL_D)                   # top scrap plate
    speckle(d, rng, (6, 7, 26, 10), RUST, 6)
    for cx in (9, 16, 23):                                # coil springs
        for i in range(4):
            y = 12 + i * 3
            L(d, [(cx - 3, y), (cx + 3, y + 1)], STEEL_D, 1)
            L(d, [(cx + 3, y + 1), (cx - 3, y + 2)], STEEL, 1)
    R(d, (5, 24, 27, 28), STEEL_D, IRON)                  # base plate
    for x in (7, 24):
        E(d, (x, 25, x + 2, 27), IRON)                    # bolts


# ---------------------------------------------------------------- stowed
def fridge_upright(d, rng, x0, y0, x1, y1, split_y=None, glass=False,
                   shelves=False):
    """Upright fridge/freezer body; optional freezer-door split or a
    full glass door with shelf lines inside."""
    RR(d, (x0, y0, x1, y1), 2, APPL, APPL_D)
    if glass:
        R(d, (x0 + 3, y0 + 3, x1 - 3, y1 - 3), GLASS, GLASS_D)
        for i in range(3):                                # shelf lines
            y = y0 + 9 + i * 6
            L(d, [(x0 + 4, y), (x1 - 4, y)], GLASS_D, 1)
            R(d, (x0 + 5, y - 3, x0 + 8, y - 1), (226, 150, 90, 255))
            R(d, (x1 - 9, y - 3, x1 - 6, y - 1), (140, 190, 120, 255))
        L(d, [(x0 + 5, y0 + 5), (x1 - 8, y0 + 5)],
          (240, 250, 252, 255), 1)                        # glass glint
    else:
        if split_y is not None:
            L(d, [(x0, split_y), (x1, split_y)], APPL_D, 1)
            R(d, (x1 - 4, split_y + 3, x1 - 3, split_y + 8), STEEL_D)
            R(d, (x1 - 4, y0 + 3, x1 - 3, y0 + 7), STEEL_D)
        else:
            R(d, (x1 - 4, y0 + 4, x1 - 3, y0 + 10), STEEL_D)
        R(d, (x0 + 2, y1 - 4, x1 - 2, y1 - 2), APPL_D)    # vent grill
        for gx in range(x0 + 4, x1 - 3, 3):
            d.point((gx, y1 - 3), fill=APPL_D)
    speckle(d, rng, (x0 + 2, y0 + 2, x1 - 2, y1 - 2), APPL_D, 3)


def chest_freezer(d, rng, x0, y0, x1, y1, glass_top=False):
    """Chest freezer: low wide box, top lid (plain or glass sliders)."""
    RR(d, (x0, y0, x1, y1), 2, APPL, APPL_D)
    if glass_top:
        R(d, (x0 + 2, y0 + 2, x1 - 2, y0 + 8), GLASS, GLASS_D)
        L(d, [(16, y0 + 2), (16, y0 + 8)], GLASS_D, 1)     # slider split
        L(d, [(x0 + 4, y0 + 4), (x1 - 6, y0 + 4)],
          (240, 250, 252, 255), 1)
        for fx in (x0 + 6, x1 - 10):                      # frozen goods
            R(d, (fx, y0 + 5, fx + 3, y0 + 7), (200, 220, 235, 255))
    else:
        L(d, [(x0, y0 + 5), (x1, y0 + 5)], APPL_D, 1)      # lid seam
        R(d, (14, y0 + 2, 18, y0 + 4), APPL_D)             # handle
    R(d, (x0 + 3, y1 - 4, x0 + 9, y1 - 2), APPL_D)         # vent
    speckle(d, rng, (x0 + 2, y0 + 10, x1 - 2, y1 - 2), APPL_D, 3)


def draw_stowed_fridge(d, rng):
    fridge_upright(d, rng, 8, 3, 24, 29, split_y=11)
    straps(d, 8, 3, 24, 29)


def draw_stowed_freezer(d, rng):
    fridge_upright(d, rng, 8, 3, 24, 29, split_y=None)
    straps(d, 8, 3, 24, 29)


def draw_stowed_big_portable_fridge(d, rng):
    fridge_upright(d, rng, 5, 2, 27, 30, split_y=10)
    straps(d, 5, 2, 27, 30)


def draw_stowed_big_portable_freezer(d, rng):
    chest_freezer(d, rng, 3, 9, 29, 27)
    straps(d, 3, 9, 29, 27)


def draw_stowed_chest_freezer(d, rng):
    chest_freezer(d, rng, 5, 10, 27, 27)
    straps(d, 5, 10, 27, 27)


def draw_stowed_chest_minifreezer(d, rng):
    chest_freezer(d, rng, 8, 13, 24, 26)
    straps(d, 8, 13, 24, 26)


def draw_stowed_display_freezer(d, rng):
    chest_freezer(d, rng, 5, 10, 27, 27, glass_top=True)
    straps(d, 5, 10, 27, 27)


def draw_stowed_glass_freezer(d, rng):
    chest_freezer(d, rng, 5, 10, 27, 27, glass_top=True)
    R(d, (7, 12, 25, 14), GLASS_D)                        # deeper glass hint
    straps(d, 5, 10, 27, 27)


def draw_stowed_display_fridge(d, rng):
    fridge_upright(d, rng, 8, 3, 24, 29, glass=True, shelves=True)
    straps(d, 8, 3, 24, 29)


def draw_stowed_glass_fridge(d, rng):
    fridge_upright(d, rng, 8, 3, 24, 29, glass=True, shelves=True)
    R(d, (11, 6, 21, 8), (240, 250, 252, 255))            # bright interior
    straps(d, 8, 3, 24, 29)


def draw_stowed_air_compressor(d, rng):
    # horizontal receiver tank (black) + yellow pump/motor block
    RR(d, (5, 14, 27, 23), 4, IRON, STEEL_D)
    R(d, (5, 17, 27, 18), IRON_L)
    R(d, (11, 6, 20, 14), BATT_Y, BATT_YD)                # pump housing
    E(d, (13, 8, 18, 13), IRON, BATT_YD)                  # motor fan cowl
    L(d, [(20, 8), (24, 12)], STEEL_D, 2)                  # handle pipe
    R(d, (22, 12, 24, 14), STEEL_D)
    R(d, (8, 23, 10, 26), STEEL_D)                        # feet
    R(d, (22, 23, 24, 26), STEEL_D)
    E(d, (24, 15, 26, 17), BRASS, STEEL_D)                 # gauge
    straps(d, 5, 6, 27, 23)


def draw_stowed_arcade_machine(d, rng):
    RED = (178, 48, 44, 255)
    RED_D = (124, 32, 30, 255)
    P(d, [(9, 3), (23, 3), (23, 20), (26, 24), (26, 29), (9, 29)],
      RED, RED_D)                                         # cabinet profile
    R(d, (11, 5, 21, 8), (250, 220, 120, 255), RED_D)      # marquee
    R(d, (11, 10, 20, 17), (30, 40, 54, 255), IRON)        # screen
    for (x, y, c) in ((13, 12, (110, 220, 130, 255)),
                      (16, 14, (240, 200, 80, 255)),
                      (18, 11, (120, 170, 240, 255))):
        R(d, (x, y, x + 1, y + 1), c)                     # game pixels
    P(d, [(10, 19), (22, 17), (25, 21), (11, 23)],
      RED_D, IRON)                                        # control deck
    E(d, (14, 19, 16, 21), (240, 90, 80, 255), IRON)      # joystick ball
    E(d, (19, 18, 21, 20), BATT_Y, IRON)                  # button
    R(d, (12, 25, 20, 28), IRON, RED_D)                    # coin door
    straps(d, 9, 3, 26, 29)


def draw_stowed_bandsaw(d, rng):
    CY = (110, 190, 205, 255)
    CY_D = (72, 138, 154, 255)
    d.arc((8, 4, 24, 20), 180, 360, fill=CY_D, width=4)    # upper wheel arc
    R(d, (8, 11, 24, 14), CY, CY_D)                       # blade guard band
    L(d, [(16, 14), (16, 22)], STEEL_L, 1)                # blade
    R(d, (6, 22, 26, 24), STEEL, STEEL_D)                 # table
    R(d, (10, 24, 13, 29), CY_D, IRON)                    # legs
    R(d, (19, 24, 22, 29), CY_D, IRON)
    E(d, (20, 15, 26, 21), BATT_Y, BATT_YD)               # motor (yellow)
    R(d, (9, 6, 13, 9), CY_D)                             # throat detail
    straps(d, 6, 4, 26, 29)


def draw_stowed_drill_press(d, rng):
    YL = (206, 172, 44, 255)
    YL_D = (158, 128, 30, 255)
    RED = (178, 52, 40, 255)
    R(d, (7, 27, 25, 29), STEEL_D, IRON)                  # base
    R(d, (9, 8, 12, 27), YL, YL_D)                        # column
    R(d, (8, 3, 24, 10), RED, (124, 34, 28, 255))         # head (red)
    R(d, (19, 4, 23, 7), IRON)                            # motor cap
    L(d, [(14, 10), (14, 15)], STEEL_D, 2)                # quill
    P(d, [(12, 15), (16, 15), (14, 19)], STEEL_L, STEEL_D)  # chuck/bit
    R(d, (13, 20, 24, 22), STEEL, STEEL_D)                # table
    L(d, [(24, 6), (27, 4)], STEEL_D, 2)                  # feed handle
    E(d, (26, 3, 28, 5), IRON)
    straps(d, 7, 3, 25, 29)


def monolith(d, rng, broken):
    AU = (216, 176, 72, 255)                              # gold
    AU_D = (164, 128, 44, 255)
    AU_L = (244, 208, 118, 255)
    if not broken:
        R(d, (11, 3, 21, 28), AU, AU_D)
        R(d, (13, 5, 19, 26), AU_L, AU_D)
        E(d, (14, 9, 18, 13), (110, 220, 220, 255), AU_D)  # alien eye
        d.point((16, 11), fill=(240, 255, 255, 255))
        L(d, [(14, 18), (18, 18)], AU_D, 1)               # glyph lines
        L(d, [(14, 21), (17, 21)], AU_D, 1)
        R(d, (9, 28, 23, 30), STEEL_D, IRON)              # skid base
    else:
        P(d, [(10, 8), (17, 5), (18, 16), (11, 18)], AU, AU_D)
        P(d, [(19, 14), (23, 18), (21, 28), (15, 24)], AU, AU_D)
        L(d, [(17, 5), (15, 12), (18, 16)], IRON, 1)      # crack lines
        L(d, [(19, 14), (18, 20), (21, 28)], IRON, 1)
        R(d, (13, 20, 16, 23), AU_L, AU_D)                # loose chunk
        speckle(d, rng, (11, 6, 22, 27), AU_D, 8)
        R(d, (8, 28, 24, 30), STEEL_D, IRON)


def draw_stowed_bot_golden_monolith(d, rng):
    monolith(d, rng, broken=False)
    straps(d, 9, 3, 23, 30)


def draw_stowed_broken_golden_monolith(d, rng):
    monolith(d, rng, broken=True)
    straps(d, 8, 5, 24, 30)


def draw_stowed_broken_yrax_quadraphract(d, rng):
    # bulky alien war-machine torso, four legs, one snapped
    BODY = (96, 116, 118, 255)
    BODY_D = (60, 76, 78, 255)
    AU = (216, 176, 72, 255)
    RR(d, (8, 8, 25, 19), 3, BODY, BODY_D)                # hull pod
    P(d, [(12, 8), (21, 8), (18, 3), (14, 3)], BODY_D, IRON)  # dorsal plate
    E(d, (13, 11, 19, 16), (150, 60, 50, 255), IRON)      # dead sensor eye
    L(d, [(10, 17), (23, 17)], AU, 1)                     # gold trim
    for (x0, y0, x1, y1) in ((6, 18, 9, 25), (23, 18, 26, 25),
                             (10, 19, 11, 26), (20, 19, 22, 26)):
        L(d, [(x0 + 1, y0), (x1, y1)], BODY_D, 3)         # legs
        R(d, (x1 - 1, y1, x1 + 2, y1 + 2), IRON)          # feet
    L(d, [(24, 19), (28, 22)], BODY_D, 2)                 # snapped leg stub
    P(d, [(27, 21), (29, 23), (27, 24)], IRON)
    speckle(d, rng, (9, 9, 24, 18), (40, 40, 42, 255), 8)  # scorch
    straps(d, 8, 3, 26, 26)


def draw_stowed_broken_yrax_quadraphract_leg(d, rng):
    BODY = (96, 116, 118, 255)
    BODY_D = (60, 76, 78, 255)
    AU = (216, 176, 72, 255)
    # upper segment, joint, lower segment snapped at the foot
    P(d, [(12, 3), (18, 3), (17, 12), (12, 12)], BODY, BODY_D)
    E(d, (11, 11, 19, 17), BODY_D, IRON)                  # knee joint
    E(d, (13, 12, 17, 16), AU, BODY_D)                    # joint core
    P(d, [(13, 17), (17, 17), (19, 25), (14, 26)], BODY, BODY_D)
    P(d, [(14, 26), (19, 25), (20, 28), (13, 29)], IRON, STEEL_D)
    L(d, [(15, 26), (16, 29)], STEEL_L, 1)                # torn wires
    L(d, [(17, 26), (19, 29)], (178, 52, 40, 255), 1)
    speckle(d, rng, (12, 4, 18, 25), BODY_D, 6)
    straps(d, 11, 3, 20, 29)


RENDERERS = [
    ("large_storage_battery", draw_large_storage_battery),
    ("large_storage_battery_removable",
     draw_large_storage_battery_removable),
    ("medium_pressure_tank", draw_medium_pressure_tank),
    ("medium_storage_battery", draw_medium_storage_battery),
    ("metal_bench", draw_metal_bench),
    ("metal_tray", draw_metal_tray),
    ("motorcycle_headlight", draw_motorcycle_headlight),
    ("motorcycle_kickstand", draw_motorcycle_kickstand),
    ("mountable_cooler", draw_mountable_cooler),
    ("nose_plate", draw_nose_plate),
    ("null", draw_null),
    ("pedal_extenders", draw_pedal_extenders),
    ("pedal_extenders_makeshift", draw_pedal_extenders_makeshift),
    ("plating_acidchitin", draw_plating_acidchitin),
    ("plating_chitin", draw_plating_chitin),
    ("plugged_3in_ordnance_rifle", draw_plugged_3in_ordnance_rifle),
    ("power_cord", draw_power_cord),
    ("programmable_autopilot", draw_programmable_autopilot),
    ("rail_wheel", draw_rail_wheel),
    ("rail_wheel_small_pair", draw_rail_wheel_small_pair),
    ("rail_wheel_steerable", draw_rail_wheel_steerable),
    ("rebar_plate", draw_rebar_plate),
    ("roof_wood", draw_roof_wood),
    ("sail", draw_sail),
    ("small_civilian_rotors", draw_small_civilian_rotors),
    ("small_pressure_tank", draw_small_pressure_tank),
    ("smart_engine_controller", draw_smart_engine_controller),
    ("solar_reflector", draw_solar_reflector),
    ("spring_plate", draw_spring_plate),
    ("storage_battery_mount", draw_storage_battery_mount),
    ("storage_battery_removable", draw_storage_battery_removable),
    ("stowed_air_compressor", draw_stowed_air_compressor),
    ("stowed_arcade_machine", draw_stowed_arcade_machine),
    ("stowed_bandsaw", draw_stowed_bandsaw),
    ("stowed_big_portable_freezer", draw_stowed_big_portable_freezer),
    ("stowed_big_portable_fridge", draw_stowed_big_portable_fridge),
    ("stowed_bot_golden_monolith", draw_stowed_bot_golden_monolith),
    ("stowed_broken_golden_monolith",
     draw_stowed_broken_golden_monolith),
    ("stowed_broken_yrax_quadraphract",
     draw_stowed_broken_yrax_quadraphract),
    ("stowed_broken_yrax_quadraphract_leg",
     draw_stowed_broken_yrax_quadraphract_leg),
    ("stowed_chest_freezer", draw_stowed_chest_freezer),
    ("stowed_chest_minifreezer", draw_stowed_chest_minifreezer),
    ("stowed_display_freezer", draw_stowed_display_freezer),
    ("stowed_display_fridge", draw_stowed_display_fridge),
    ("stowed_drill_press", draw_stowed_drill_press),
    ("stowed_freezer", draw_stowed_freezer),
    ("stowed_fridge", draw_stowed_fridge),
    ("stowed_glass_freezer", draw_stowed_glass_freezer),
    ("stowed_glass_fridge", draw_stowed_glass_fridge),
]


def main():
    os.makedirs(SHEET, exist_ok=True)
    imgs = []
    for pid, fn in RENDERERS:
        rng = random.Random(zlib.crc32(pid.encode()))
        img = canvas()
        d = ImageDraw.Draw(img)
        fn(d, rng)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (pid, sorted(alphas))
        folder = os.path.join(SHEET, pid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, pid + ".png"))
        entry = [{"id": pid, "fg": pid, "bg": ""}]
        with open(os.path.join(folder, pid + ".json"), "w") as f:
            json.dump(entry, f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS + "\n")
        imgs.append((pid, img))
        print("wrote", pid)
    # ------------------------------------------------------------ preview
    COLS, ROWS, CELL = 7, 7, 96
    sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL),
                      (24, 24, 28, 255))
    for i, (pid, img) in enumerate(imgs):
        big = img.resize((CELL, CELL), Image.NEAREST)
        sheet.alpha_composite(big, ((i % COLS) * CELL,
                                    (i // COLS) * CELL))
    sheet.save(os.path.join(BASE, "preview_veh3.png"))
    print("wrote preview_veh3.png", sheet.size)


if __name__ == "__main__":
    main()
