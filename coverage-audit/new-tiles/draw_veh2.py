#!/usr/bin/env python3
"""
Draw the missing vehicle part sprites (batch VEH2) from their game
descriptions.

Sources (fork-src/cdda/data/json/vehicleparts/ + teleportation.json,
CDDA 6a3e59e0) -- the 49 ids below are vehicle parts missing from
UltiCa in the shipped universe (coverage-audit/missing.json) with no
folder yet under new-tiles/.  Each is drawn as the part itself, an
object on a transparent background, in flat UltiCa style at native
32x32 (vehicle parts present at tile size; no sheet-size rule for
this category):

  diesel_engine_multifuel_i6  military multifuel inline-6 diesel
  diesel_engine_v12 / v6 / v8 diesel blocks, sized by cylinder count
  engine_1cyl_*               single-cylinder engines, finned barrel
  engine_aero_light           opposed-4 aircraft engine + prop hub
  engine_inline4              car inline-4 block
  engine_electric_*           motor cans, sized tiny -> train
  engine_steam_*              boiler + stack + flywheel
  engine_turbine_*            intake ring with fan blades
  dimensional_perambulator    pram under a violet vortex disc
  door_lock / door_motor      light-blue lock plate; tiny green motor
  drive_by_wire_controls      light-red remote control console
  drum_concrete_mix           light-blue 100 L mixer drum
  frame_concrete_mix          the drum's A-frame cradle stand
  external_tank_aluminum      silver 200 L drum
  folding_frame / folding_wooden_frame / foldxlframe
                              scissor / pipe foldable frames
  frame_wood_handle / frame_wood_light / frame_wood_light_handle
                              wooden handles and a rope-lashed frame
  fridgetank / fridgetank_large / heattank
                              refrigerated tanks (heat: warm accent)
  fuel_bunker                 brown solid-fuel bin with coal heap
  hand_controls               twin rods with trigger grips
  handheld_battery_mount      plate with two docked tool batteries
  horn_big / horn_car         twin truck trumpets; round disc horn
  improvised_frame            crudely welded heavy steel frame
  inboard_mirror              small pale-cyan mirror on a stalk
  inflatable_airbag           green inflatable cushion with seams
  integrated_cooler           light-cyan mounted cooler box
  integrated_heater_small     red serpentine heater tubes
  jumper_cable_debug          DEBUG: yellow cables + checkerboard tag
  large_pressure_tank         horizontal capsule gas tank

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
COPPER = (176, 96, 44, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (110, 76, 42, 255)
WOOD_L = (178, 138, 88, 255)
ROPE = (198, 172, 120, 255)


# ---------------------------------------------------------------- engines
def engine_block(d, rng, x0, y0, x1, y1, base, dark, light, cyls,
                 vbank=False):
    """Side-view engine block: sump, block, valve cover(s), pulley,
    exhaust pipe along the bottom, intake stub on top."""
    # sump
    R(d, (x0 + 2, y1 - 3, x1 - 2, y1), dark)
    # main block
    R(d, (x0, y0 + 5, x1, y1 - 3), base, dark)
    speckle(d, rng, (x0 + 2, y0 + 8, x1 - 2, y1 - 6), dark, 10)
    # valve cover
    if vbank:
        R(d, (x0, y0 + 1, (x0 + x1) // 2 - 1, y0 + 6), light, dark)
        R(d, ((x0 + x1) // 2 + 1, y0 + 1, x1, y0 + 6), light, dark)
    else:
        R(d, (x0 + 1, y0 + 1, x1 - 1, y0 + 6), light, dark)
    # cylinder bumps across the cover
    span = (x1 - x0 - 4) / max(cyls, 1)
    for i in range(cyls):
        cx = x0 + 3 + int(span * i + span / 2)
        R(d, (cx - 1, y0 - 1, cx + 1, y0 + 2), dark)
    # intake stub
    R(d, ((x0 + x1) // 2 - 2, y0 - 3, (x0 + x1) // 2 + 2, y0), dark)
    # front pulley (left)
    E(d, (x0 - 2, (y0 + y1) // 2 - 3, x0 + 3, (y0 + y1) // 2 + 3),
      dark, light)
    # exhaust manifold pipe
    L(d, [(x0 + 3, y1 - 5), (x1 - 4, y1 - 5), (x1 - 1, y1 - 2)],
      dark, 2)


def draw_diesel_i6(d, rng):
    # long olive military block, six cylinders
    engine_block(d, rng, 3, 9, 29, 27, (88, 94, 58, 255),
                 (52, 58, 34, 255), (112, 120, 76, 255), 6)
    # stencilled lifting eyes + rugged ribs
    L(d, [(6, 12), (6, 24)], (52, 58, 34, 255), 1)
    L(d, [(26, 12), (26, 24)], (52, 58, 34, 255), 1)


def draw_diesel_v12(d, rng):
    engine_block(d, rng, 2, 8, 30, 28, (66, 70, 78, 255),
                 (38, 40, 46, 255), (96, 102, 112, 255), 6, vbank=True)


def draw_diesel_v8(d, rng):
    engine_block(d, rng, 3, 9, 29, 27, (72, 76, 84, 255),
                 (42, 45, 52, 255), (100, 106, 116, 255), 4, vbank=True)


def draw_diesel_v6(d, rng):
    engine_block(d, rng, 5, 10, 27, 26, (76, 80, 88, 255),
                 (46, 49, 56, 255), (104, 110, 120, 255), 3, vbank=True)


def draw_inline4(d, rng):
    engine_block(d, rng, 5, 10, 27, 26, (110, 116, 124, 255),
                 (66, 71, 79, 255), (146, 84, 60, 255), 4)


def one_cyl(d, rng, scale, diesel=False):
    """Single finned cylinder upright on a small crankcase."""
    w = scale
    cx = 16
    x0, x1 = cx - w // 2, cx + w // 2
    base = (74, 78, 84, 255) if diesel else (104, 108, 114, 255)
    dark = (44, 47, 52, 255) if diesel else (62, 66, 72, 255)
    # crankcase
    RR(d, (x0 - 2, 21, x1 + 2, 28), 2, base, dark)
    # flywheel disc at left of case
    E(d, (x0 - 5, 20, x0 + 1, 27), dark, STEEL_L)
    # finned barrel
    R(d, (x0, 8, x1, 21), base, dark)
    for y in range(10, 21, 2):
        L(d, [(x0 - 1, y), (x1 + 1, y)], dark, 1)
    # head + cap
    R(d, (x0 - 1, 5, x1 + 1, 8), dark)
    R(d, (cx - 2, 3, cx + 2, 5), RUST if diesel else COPPER)
    # exhaust stub
    L(d, [(x1, 12), (x1 + 3, 10)], dark, 2)
    speckle(d, rng, (x0, 9, x1, 20), dark, 4)


def draw_1cyl_diesel_large(d, rng):
    one_cyl(d, rng, 10, diesel=True)


def draw_1cyl_large(d, rng):
    one_cyl(d, rng, 10)


def draw_1cyl_med(d, rng):
    one_cyl(d, rng, 8)


def draw_1cyl_small(d, rng):
    # tiny 25 cc: pull-start housing + mini tank
    RR(d, (10, 14, 22, 27), 3, (120, 124, 128, 255), STEEL_D)
    E(d, (12, 17, 20, 25), STEEL_D, STEEL_L)          # recoil housing
    R(d, (13, 8, 19, 14), (88, 92, 96, 255), IRON)    # little cylinder
    for y in (10, 12):
        L(d, [(12, y), (20, y)], IRON, 1)
    R(d, (21, 9, 25, 14), RUST)                        # fuel tank
    R(d, (22, 7, 24, 9), IRON)                         # cap


def draw_aero(d, rng):
    # crankcase center, two finned jugs each side, prop hub front
    RR(d, (12, 11, 20, 22), 2, (96, 100, 106, 255), STEEL_D)
    for sx in (3, 22):  # cylinder pairs L / R
        for i, yy in enumerate((10, 17)):
            R(d, (sx, yy, sx + 7, yy + 5), (80, 84, 90, 255), IRON)
            for fx in range(sx + 1, sx + 7, 2):
                L(d, [(fx, yy), (fx, yy + 5)], IRON, 1)
    E(d, (13, 14, 19, 20), IRON, STEEL_L)              # prop hub
    E(d, (15, 16, 17, 18), STEEL_L)
    L(d, [(16, 4), (16, 13)], STEEL_D, 2)              # prop blade stub up
    L(d, [(16, 21), (16, 29)], STEEL_D, 2)             # and down
    speckle(d, rng, (12, 12, 20, 21), STEEL_D, 5)


def motor(d, rng, x0, y0, x1, y1, base, dark, light, cable=True):
    """Horizontal electric motor can: fins, end bell, shaft, terminal
    box, power cable."""
    RR(d, (x0, y0, x1, y1), 3, base, dark)
    # cooling fins (vertical stripes)
    for x in range(x0 + 3, x1 - 2, 3):
        L(d, [(x, y0 + 2), (x, y1 - 2)], dark, 1)
    # end bell
    RR(d, (x1 - 4, y0 + 1, x1 + 2, y1 - 1), 2, light, dark)
    # shaft
    R(d, (x0 - 3, (y0 + y1) // 2 - 1, x0, (y0 + y1) // 2 + 1), STEEL_L)
    # terminal box
    R(d, ((x0 + x1) // 2 - 3, y0 - 3, (x0 + x1) // 2 + 3, y0), dark)
    if cable:
        d.arc((x1 - 2, y1 - 4, x1 + 6, y1 + 6), 90, 200, fill=dark,
              width=2)
    speckle(d, rng, (x0 + 2, y0 + 2, x1 - 5, y1 - 2), dark, 6)


def draw_electric_tiny(d, rng):
    motor(d, rng, 11, 13, 21, 20, (140, 144, 148, 255), STEEL_D,
          STEEL_L, cable=False)
    R(d, (14, 22, 18, 24), STEEL_D)  # little mount foot


def draw_electric_small(d, rng):
    motor(d, rng, 8, 12, 24, 21, (126, 130, 136, 255), STEEL_D, STEEL_L)


def draw_electric_enhanced(d, rng):
    # sporty performance motor: orange can, black end bell
    motor(d, rng, 6, 10, 26, 23, (206, 96, 42, 255), (120, 48, 20, 255),
          (52, 54, 58, 255))
    L(d, [(9, 13), (21, 13)], (255, 178, 92, 255), 1)  # highlight stripe


def draw_electric_super(d, rng):
    motor(d, rng, 4, 9, 28, 25, (70, 84, 110, 255), (38, 48, 66, 255),
          (104, 122, 148, 255))
    # heavy twin cables
    d.arc((22, 20, 31, 30), 90, 190, fill=(30, 34, 40, 255), width=3)


def draw_electric_train(d, rng):
    # huge boxy traction motor: ribbed housing, big terminals
    RR(d, (3, 8, 29, 26), 2, (84, 92, 84, 255), (44, 50, 44, 255))
    for x in range(6, 28, 4):
        L(d, [(x, 10), (x, 24)], (44, 50, 44, 255), 2)
    R(d, (6, 4, 12, 8), COPPER, IRON)     # terminal posts
    R(d, (20, 4, 26, 8), COPPER, IRON)
    E(d, (13, 13, 19, 19), (52, 58, 52, 255), STEEL_L)  # axle cap
    speckle(d, rng, (4, 9, 28, 25), (44, 50, 44, 255), 8)


def steam(d, rng, x0, y0, x1, y1, boiler, dark, trim, stack_dx=0,
          rough=False):
    """Boiler on a frame, smokestack, steam dome, flywheel."""
    # frame legs
    L(d, [(x0 + 2, y1 - 4), (x0 + 2, y1)], dark, 2)
    L(d, [(x1 - 2, y1 - 4), (x1 - 2, y1)], dark, 2)
    # boiler body
    RR(d, (x0, y0 + 4, x1 - 6, y1 - 4), 4, boiler, dark)
    for bx in range(x0 + 4, x1 - 8, 5):   # boiler bands
        L(d, [(bx, y0 + 5), (bx, y1 - 5)], dark, 1)
    # firebox door
    E(d, (x0 + 2, y1 - 10, x0 + 7, y1 - 5), dark, trim)
    # smokestack
    sx = x0 + 5 + stack_dx
    P(d, [(sx, y0 + 4), (sx + 4, y0 + 4), (sx + 3, y0 - 2),
          (sx + 1, y0 - 2)], dark)
    R(d, (sx - 1, y0 - 4, sx + 5, y0 - 2), dark)  # stack cap flare
    # steam dome
    d.pieslice((x1 - 13, y0, x1 - 7, y0 + 7), 180, 360, fill=trim,
               outline=dark)
    # flywheel at right
    E(d, (x1 - 7, y1 - 13, x1 + 1, y1 - 5), dark, trim)
    E(d, (x1 - 5, y1 - 11, x1 - 1, y1 - 7), boiler)
    if rough:
        # patch plates + crooked seams
        R(d, (x0 + 8, y0 + 8, x0 + 13, y0 + 13), RUST, dark)
        R(d, (x0 + 15, y1 - 11, x0 + 19, y1 - 7), RUST, dark)
        speckle(d, rng, (x0 + 1, y0 + 5, x1 - 8, y1 - 5), RUST, 12)


def draw_steam_makeshift(d, rng):
    steam(d, rng, 5, 10, 27, 28, (122, 96, 70, 255), (66, 50, 34, 255),
          (150, 118, 84, 255), stack_dx=1, rough=True)


def draw_steam_small(d, rng):
    steam(d, rng, 7, 11, 26, 27, (70, 72, 78, 255), (36, 38, 42, 255),
          BRASS)


def draw_steam_medium(d, rng):
    steam(d, rng, 4, 9, 29, 28, (64, 66, 72, 255), (32, 34, 38, 255),
          BRASS)


def turbine(d, rng, cx, cy, r, ring, dark, blade):
    """Front view of a gas turbine: intake ring, fan blades, hub;
    exhaust cone peeking at lower right."""
    E(d, (cx + r - 4, cy + r - 6, cx + r + 3, cy + r + 1), dark)  # exhaust
    E(d, (cx - r, cy - r, cx + r, cy + r), ring, dark)
    E(d, (cx - r + 3, cy - r + 3, cx + r - 3, cy + r - 3),
      (30, 32, 36, 255))
    import math
    for i in range(8):
        a = i * math.pi / 4 + 0.2
        bx = cx + int((r - 4) * math.cos(a))
        by = cy + int((r - 4) * math.sin(a))
        L(d, [(cx, cy), (bx, by)], blade, 2)
    E(d, (cx - 3, cy - 3, cx + 3, cy + 3), ring, dark)
    speckle(d, rng, (cx - r + 1, cy - r + 1, cx + r - 1, cy - r + 3),
            dark, 3)


def draw_turbine_small(d, rng):
    turbine(d, rng, 15, 16, 9, STEEL, STEEL_D, STEEL_L)


def draw_turbine_medium(d, rng):
    turbine(d, rng, 15, 16, 12, (132, 138, 144, 255), STEEL_D, STEEL_L)


def draw_turbine_large(d, rng):
    turbine(d, rng, 15, 16, 14, (120, 126, 132, 255), (64, 68, 74, 255),
            STEEL_L)


# ---------------------------------------------------------------- others
def draw_perambulator(d, rng):
    # pram body on spoked wheels, violet vortex disc floating above
    E(d, (7, 18, 25, 27), (44, 40, 62, 255), (24, 22, 36, 255))  # basket
    R(d, (6, 17, 26, 19), (58, 52, 80, 255), (24, 22, 36, 255))
    L(d, [(25, 18), (29, 14)], (24, 22, 36, 255), 2)             # handle
    for wx in (10, 22):
        E(d, (wx - 3, 25, wx + 3, 31), (24, 22, 36, 255),
          (120, 110, 160, 255))
        d.point((wx, 28), fill=(180, 170, 220, 255))
    # vortex: concentric broken rings of pale violet (opaque)
    for rr_, col in ((7, (150, 120, 220, 255)), (5, (178, 152, 240, 255)),
                     (3, (208, 190, 255, 255))):
        d.arc((16 - rr_, 9 - rr_, 16 + rr_, 9 + rr_), 20, 300,
              fill=col, width=2)
        d.arc((16 - rr_, 9 - rr_, 16 + rr_, 9 + rr_), 200, 480,
              fill=col, width=2)
    d.point((16, 9), fill=(240, 235, 255, 255))
    speckle(d, rng, (8, 3, 24, 15), (160, 140, 230, 255), 6)


def draw_door_lock(d, rng):
    # light-blue mounting plate, lock cylinder with keyhole, latch bolt
    RR(d, (9, 6, 20, 26), 2, (150, 180, 205, 255), (84, 110, 140, 255))
    E(d, (11, 9, 18, 16), STEEL_L, (84, 110, 140, 255))
    E(d, (13, 11, 15, 13), IRON)                       # keyhole
    L(d, [(14, 13), (14, 15)], IRON, 1)
    R(d, (20, 13, 27, 18), STEEL, STEEL_D)             # latch bolt
    for sy in (8, 23):
        E(d, (10, sy, 13, sy + 3), STEEL_D)            # screws
    speckle(d, rng, (10, 18, 19, 24), (120, 155, 185, 255), 4)


def draw_door_motor(d, rng):
    # tiny light-green motor + gear + wire
    RR(d, (9, 12, 20, 21), 2, (140, 190, 140, 255), (70, 110, 70, 255))
    for x in range(12, 19, 2):
        L(d, [(x, 14), (x, 19)], (70, 110, 70, 255), 1)
    E(d, (19, 13, 27, 21), BRASS, (110, 84, 40, 255))  # gear
    for gx, gy in ((23, 12), (27, 17), (23, 22), (19, 17)):
        d.point((gx, gy), fill=(110, 84, 40, 255))
    E(d, (21, 15, 25, 19), (110, 84, 40, 255))
    d.arc((6, 18, 14, 27), 180, 330, fill=IRON, width=1)  # wire
    speckle(d, rng, (10, 13, 18, 20), (100, 150, 100, 255), 3)


def draw_drive_by_wire(d, rng):
    # light-red accented remote console: box, two sticks, buttons, antenna
    RR(d, (6, 12, 26, 27), 3, (70, 72, 78, 255), (36, 38, 42, 255))
    R(d, (6, 12, 26, 16), (178, 70, 60, 255), (36, 38, 42, 255))
    L(d, [(23, 12), (27, 5)], STEEL_L, 1)              # antenna
    d.point((27, 5), fill=(255, 90, 80, 255))
    for sx in (11, 20):                                # sticks
        L(d, [(sx, 23), (sx, 18)], STEEL_L, 2)
        E(d, (sx - 2, 15, sx + 2, 19), IRON, STEEL_L)
    for i, col in enumerate(((230, 70, 60, 255), (90, 200, 90, 255),
                             (240, 200, 80, 255))):
        E(d, (13 + i * 4, 24, 15 + i * 4, 26), col)
    speckle(d, rng, (8, 18, 24, 25), (50, 52, 58, 255), 4)


def draw_drum_mix(d, rng):
    # light-blue plastic drum, ribbed, dark mouth on top
    P(d, [(9, 8), (23, 8), (25, 26), (7, 26)], (130, 175, 215, 255))
    d.polygon([(9, 8), (23, 8), (25, 26), (7, 26)],
              outline=(70, 105, 140, 255))
    for y in (13, 18, 23):
        L(d, [(8, y), (24, y)], (95, 140, 180, 255), 2)
    E(d, (11, 5, 21, 10), (52, 60, 68, 255), (70, 105, 140, 255))
    speckle(d, rng, (10, 10, 22, 24), (160, 200, 230, 255), 8)


def draw_frame_mix(d, rng):
    # A-frame cradle stand for the mixer drum
    L(d, [(8, 27), (13, 10)], STEEL_D, 3)
    L(d, [(24, 27), (19, 10)], STEEL_D, 3)
    L(d, [(6, 27), (26, 27)], STEEL_D, 3)              # base bar
    L(d, [(11, 18), (21, 18)], STEEL_D, 2)             # cross bar
    d.arc((11, 6, 21, 14), 0, 180, fill=STEEL, width=3)  # cradle arc
    E(d, (14, 8, 18, 12), STEEL_D)                     # axle boss
    speckle(d, rng, (8, 12, 24, 26), STEEL_D, 3)


def drum(d, rng, x0, y0, x1, y1, base, dark, light, cap=True):
    RR(d, (x0, y0, x1, y1), 3, base, dark)
    for fy in (0.3, 0.62):
        y = int(y0 + (y1 - y0) * fy)
        L(d, [(x0 + 1, y), (x1 - 1, y)], dark, 2)
    L(d, [(x0 + 3, y0 + 2), (x0 + 3, y1 - 2)], light, 2)  # highlight
    if cap:
        R(d, ((x0 + x1) // 2 - 2, y0 - 2, (x0 + x1) // 2 + 2, y0), dark)
    speckle(d, rng, (x0 + 2, y0 + 3, x1 - 2, y1 - 3), dark, 6)


def draw_external_tank(d, rng):
    drum(d, rng, 8, 6, 24, 27, (196, 202, 208, 255), (120, 128, 136, 255),
         (232, 238, 242, 255))


def scissor_frame(d, rng, bar, dark, thick=2):
    # two X crossings with hinge bolts
    for x0 in (7, 17):
        L(d, [(x0, 8), (x0 + 8, 24)], bar, thick)
        L(d, [(x0 + 8, 8), (x0, 24)], bar, thick)
        E(d, (x0 + 3, 15, x0 + 5, 17), dark)
    L(d, [(7, 8), (25, 8)], bar, thick)
    L(d, [(7, 24), (25, 24)], bar, thick)


def draw_folding_frame(d, rng):
    scissor_frame(d, rng, STEEL, STEEL_D)
    speckle(d, rng, (8, 9, 24, 23), STEEL_D, 4)


def draw_folding_wooden_frame(d, rng):
    scissor_frame(d, rng, WOOD, WOOD_D, thick=3)
    speckle(d, rng, (8, 9, 24, 23), WOOD_D, 5)


def draw_foldxlframe(d, rng):
    # extra-light pipework: thin double rectangle
    RR(d, (8, 8, 24, 24), 2, None, STEEL_L, 2)
    RR(d, (12, 12, 20, 20), 1, None, STEEL, 1)
    for cx, cy in ((8, 8), (24, 8), (8, 24), (24, 24)):
        E(d, (cx - 2, cy - 2, cx + 2, cy + 2), STEEL_D)
    L(d, [(12, 16), (8, 16)], STEEL, 1)
    L(d, [(20, 16), (24, 16)], STEEL, 1)


def handles(d, rng, wood, dark, light_wood=False):
    # a pair of horizontal handles on two mounting brackets
    for y in (11, 20):
        R(d, (5, y, 27, y + 3), wood, dark)
        L(d, [(6, y + 1), (26, y + 1)],
          WOOD_L if light_wood else dark, 1)
        for bx in (8, 22):
            R(d, (bx, y + 3, bx + 2, y + 5), STEEL_D)
    speckle(d, rng, (6, 11, 26, 23), dark, 5)


def draw_frame_wood_handle(d, rng):
    handles(d, rng, WOOD, WOOD_D)


def draw_frame_wood_light_handle(d, rng):
    handles(d, rng, WOOD_L, WOOD, light_wood=True)


def draw_frame_wood_light(d, rng):
    # few pieces of wood held by rope: two posts, one crossbar, lashings
    R(d, (8, 6, 11, 26), WOOD, WOOD_D)
    R(d, (21, 6, 24, 26), WOOD, WOOD_D)
    R(d, (6, 12, 26, 15), WOOD, WOOD_D)
    L(d, [(7, 13), (25, 13)], WOOD_L, 1)
    for lx, ly in ((8, 12), (21, 12)):
        for i in range(3):
            L(d, [(lx, ly + i), (lx + 3, ly + i + 1)], ROPE, 1)
    R(d, (14, 20, 18, 23), WOOD_D)  # peg
    speckle(d, rng, (8, 7, 24, 25), WOOD_D, 6)


def fridge_tank(d, rng, x0, y0, x1, y1, warm=False):
    body = (214, 218, 220, 255)
    dark = (110, 116, 120, 255)
    RR(d, (x0, y0 + 4, x1, y1), 3, body, dark)
    # cooling/heating unit on top
    R(d, (x0 + 3, y0, x1 - 3, y0 + 5), STEEL, dark)
    if warm:
        R(d, (x0 + 2, y0 + 8, x1 - 2, y0 + 11), (226, 120, 60, 255))
        for wx in (x0 + 6, (x0 + x1) // 2, x1 - 6):  # heat shimmer
            d.arc((wx - 2, y1 - 12, wx + 2, y1 - 4), 270, 450,
                  fill=(230, 130, 60, 255), width=1)
    else:
        R(d, (x0 + 2, y0 + 8, x1 - 2, y0 + 11), (110, 180, 225, 255))
        for gx in range(x0 + 5, x1 - 4, 3):          # grille
            L(d, [(gx, y0 + 1), (gx, y0 + 4)], dark, 1)
    L(d, [(x0 + 2, (y0 + y1) // 2), (x1 - 2, (y0 + y1) // 2)], dark, 1)
    for fx in (x0 + 3, x1 - 5):
        R(d, (fx, y1, fx + 2, y1 + 2), dark)
    speckle(d, rng, (x0 + 2, y0 + 13, x1 - 2, y1 - 3), dark, 5)


def draw_fridgetank(d, rng):
    fridge_tank(d, rng, 8, 7, 24, 25)


def draw_fridgetank_large(d, rng):
    fridge_tank(d, rng, 5, 5, 27, 27)


def draw_heattank(d, rng):
    fridge_tank(d, rng, 8, 7, 24, 25, warm=True)


def draw_fuel_bunker(d, rng):
    # brown open bin, coal heaped above the rim
    for cx, cy, r in ((11, 8, 3), (16, 6, 4), (21, 8, 3), (14, 9, 3),
                      (19, 9, 3)):
        E(d, (cx - r, cy - r, cx + r, cy + r), (40, 40, 44, 255),
          (16, 16, 18, 255))
    speckle(d, rng, (8, 4, 24, 11), (90, 90, 96, 255), 6)
    P(d, [(6, 11), (26, 11), (24, 27), (8, 27)], (128, 92, 58, 255))
    d.polygon([(6, 11), (26, 11), (24, 27), (8, 27)],
              outline=(84, 58, 34, 255))
    R(d, (5, 10, 27, 13), WOOD_D)
    for x in (10, 16, 22):
        L(d, [(x, 14), (x - 1, 26)], (84, 58, 34, 255), 1)


def draw_hand_controls(d, rng):
    # clamp bar with two adjustable rods ending in trigger grips
    R(d, (4, 24, 28, 27), STEEL_D, IRON)
    for x0 in (9, 20):
        L(d, [(x0, 24), (x0 + 3, 12)], STEEL, 3)
        RR(d, (x0 + 1, 6, x0 + 6, 12), 2, IRON, STEEL_D)   # grip
        L(d, [(x0 + 4, 12), (x0 + 4, 15)], (220, 70, 60, 255), 2)
        E(d, (x0 + 1, 17, x0 + 4, 20), STEEL_D)            # pivot
    speckle(d, rng, (5, 24, 27, 26), IRON, 3)


def draw_battery_mount(d, rng):
    # back plate, converter box, two docked tool batteries, cable
    RR(d, (6, 6, 26, 26), 2, (88, 92, 96, 255), IRON)
    for bx in (9, 18):
        RR(d, (bx, 10, bx + 6, 21), 1, (52, 54, 58, 255), IRON)
        R(d, (bx + 1, 12, bx + 5, 14), (90, 205, 90, 255))  # charge bar
        R(d, (bx + 2, 8, bx + 4, 10), STEEL_D)
    R(d, (12, 22, 20, 25), COPPER, IRON)                     # converter
    d.arc((20, 20, 29, 29), 0, 160, fill=IRON, width=2)      # cable
    speckle(d, rng, (7, 7, 25, 25), IRON, 5)


def draw_horn_big(d, rng):
    # twin long air-horn trumpets
    for y0 in (9, 19):
        P(d, [(6, y0 + 2), (20, y0), (20, y0 + 6), (6, y0 + 4)],
          (196, 200, 204, 255))
        d.polygon([(6, y0 + 2), (20, y0), (20, y0 + 6), (6, y0 + 4)],
                  outline=STEEL_D)
        E(d, (19, y0 - 1, 27, y0 + 7), STEEL_L, STEEL_D)     # bell
        E(d, (22, y0 + 1, 25, y0 + 5), (120, 40, 36, 255))
        R(d, (4, y0 + 1, 7, y0 + 5), STEEL_D)
    L(d, [(5, 15), (5, 22)], STEEL_D, 2)                      # bracket


def draw_horn_car(d, rng):
    E(d, (7, 7, 25, 25), (66, 68, 72, 255), STEEL_D)
    E(d, (10, 10, 22, 22), STEEL, STEEL_D)
    E(d, (13, 13, 19, 19), STEEL_D, STEEL_L)
    E(d, (15, 15, 17, 17), STEEL_L)
    R(d, (14, 25, 18, 28), STEEL_D)                          # mount tab
    speckle(d, rng, (9, 9, 23, 23), STEEL_D, 5)


def draw_improvised_frame(d, rng):
    # heavy crude frame: thick border, crossbar, weld blobs
    R(d, (6, 6, 26, 26), None, (52, 54, 58, 255), 4)
    L(d, [(6, 16), (26, 16)], (52, 54, 58, 255), 3)
    L(d, [(16, 6), (16, 26)], (52, 54, 58, 255), 2)
    for cx, cy in ((6, 6), (26, 6), (6, 26), (26, 26), (16, 16)):
        E(d, (cx - 2, cy - 2, cx + 2, cy + 2), (140, 144, 148, 255))
    speckle(d, rng, (7, 7, 25, 25), RUST, 8)
    speckle(d, rng, (7, 7, 25, 25), (30, 30, 34, 255), 6)


def draw_inboard_mirror(d, rng):
    # small mirror on a stalk with ball joint
    L(d, [(16, 27), (16, 20)], STEEL_D, 2)
    R(d, (12, 26, 20, 29), STEEL_D)
    RR(d, (8, 6, 24, 20), 3, (196, 232, 240, 255), STEEL_D)
    RR(d, (10, 8, 22, 18), 2, (214, 242, 246, 255))
    L(d, [(12, 17), (18, 9)], (255, 255, 255, 255), 2)  # glare streak
    L(d, [(17, 17), (21, 12)], (255, 255, 255, 255), 1)
    E(d, (14, 18, 18, 21), STEEL_D)                     # ball joint


def draw_airbag(d, rng):
    # green inflatable cushion: stacked lobes, seams, valve
    E(d, (6, 10, 26, 25), (74, 160, 84, 255), (40, 96, 48, 255))
    E(d, (9, 6, 23, 16), (88, 176, 98, 255), (40, 96, 48, 255))
    d.arc((8, 9, 24, 24), 300, 120, fill=(40, 96, 48, 255), width=1)
    d.arc((10, 7, 22, 15), 300, 120, fill=(40, 96, 48, 255), width=1)
    R(d, (23, 13, 27, 16), (52, 120, 62, 255), (40, 96, 48, 255))
    speckle(d, rng, (8, 8, 24, 23), (120, 200, 128, 255), 7)


def draw_cooler(d, rng):
    # light-cyan mounted cooler box: lid, latch, vents, feet
    RR(d, (7, 10, 25, 25), 2, (206, 238, 242, 255), (96, 140, 148, 255))
    R(d, (7, 10, 25, 14), (170, 216, 224, 255), (96, 140, 148, 255))
    R(d, (14, 14, 18, 17), STEEL_D)                       # latch
    for y in (19, 22):
        L(d, [(20, y), (24, y)], (96, 140, 148, 255), 1)  # vents
    L(d, [(9, 17), (13, 17)], (150, 200, 208, 255), 1)
    for fx in (9, 21):
        R(d, (fx, 25, fx + 2, 27), (96, 140, 148, 255))
    speckle(d, rng, (8, 15, 24, 24), (170, 216, 224, 255), 4)


def draw_heater_small(d, rng):
    # serpentine red tube bundle with steel clamps
    pipe = (196, 74, 58, 255)
    dark = (110, 40, 32, 255)
    for i, y in enumerate((9, 15, 21)):
        L(d, [(8, y), (24, y)], pipe, 3)
        if i < 2:
            d.arc((21, y, 27, y + 6), 270, 90, fill=pipe, width=3)
    L(d, [(8, 9), (8, 6)], pipe, 3)      # inlet
    L(d, [(24, 21), (24, 25)], pipe, 3)  # outlet
    for x in (12, 20):
        R(d, (x, 7, x + 2, 23), STEEL_D)  # clamps
    speckle(d, rng, (9, 8, 23, 22), dark, 4)


def draw_jumper_debug(d, rng):
    # DEBUG artifact: coiled yellow cable, red/black clamps, checker tag
    d.arc((7, 6, 25, 24), 0, 360, fill=(240, 205, 40, 255), width=3)
    d.arc((11, 10, 21, 20), 0, 360, fill=(240, 205, 40, 255), width=3)
    # red clamp top-right
    P(d, [(23, 6), (29, 3), (30, 6), (25, 8)], (216, 60, 52, 255))
    P(d, [(25, 8), (30, 8), (29, 11), (24, 10)], (160, 40, 36, 255))
    # black clamp bottom-left
    P(d, [(3, 22), (8, 21), (9, 25), (4, 26)], (52, 52, 56, 255))
    P(d, [(8, 24), (12, 26), (10, 29), (6, 27)], (30, 30, 34, 255))
    # checkerboard debug tag
    for i in range(3):
        for j in range(3):
            col = (245, 245, 245, 255) if (i + j) % 2 == 0 \
                else (20, 20, 20, 255)
            R(d, (21 + i * 3, 21 + j * 3, 23 + i * 3, 23 + j * 3), col)
    R(d, (21, 21, 29, 29), None, (240, 205, 40, 255), 1)


def draw_pressure_tank(d, rng):
    # horizontal capsule tank, top valve, saddle feet
    RR(d, (4, 11, 28, 22), 8, (206, 210, 214, 255), (122, 128, 134, 255))
    L(d, [(8, 13), (8, 20)], (150, 156, 162, 255), 2)   # weld seam
    L(d, [(24, 13), (24, 20)], (150, 156, 162, 255), 2)
    R(d, (14, 8, 18, 11), STEEL_D)                       # valve stem
    L(d, [(12, 8), (20, 8)], STEEL_D, 2)                 # valve handle
    E(d, (14, 5, 18, 9), BRASS, STEEL_D)
    for fx in (8, 22):
        P(d, [(fx, 22), (fx + 3, 22), (fx + 2, 26), (fx + 1, 26)],
          STEEL_D)
    speckle(d, rng, (6, 13, 26, 20), (160, 166, 172, 255), 5)


RENDERERS = [
    ("diesel_engine_multifuel_i6", draw_diesel_i6),
    ("diesel_engine_v12", draw_diesel_v12),
    ("diesel_engine_v6", draw_diesel_v6),
    ("diesel_engine_v8", draw_diesel_v8),
    ("dimensional_perambulator", draw_perambulator),
    ("door_lock", draw_door_lock),
    ("door_motor", draw_door_motor),
    ("drive_by_wire_controls", draw_drive_by_wire),
    ("drum_concrete_mix", draw_drum_mix),
    ("engine_1cyl_diesel_large", lambda d, r: one_cyl(d, r, 10, True)),
    ("engine_1cyl_large", lambda d, r: one_cyl(d, r, 10)),
    ("engine_1cyl_med", lambda d, r: one_cyl(d, r, 8)),
    ("engine_1cyl_small", draw_1cyl_small),
    ("engine_aero_light", draw_aero),
    ("engine_electric_enhanced", draw_electric_enhanced),
    ("engine_electric_small", draw_electric_small),
    ("engine_electric_super", draw_electric_super),
    ("engine_electric_tiny", draw_electric_tiny),
    ("engine_electric_train", draw_electric_train),
    ("engine_inline4", draw_inline4),
    ("engine_steam_makeshift", draw_steam_makeshift),
    ("engine_steam_medium", draw_steam_medium),
    ("engine_steam_small", draw_steam_small),
    ("engine_turbine_large", draw_turbine_large),
    ("engine_turbine_medium", draw_turbine_medium),
    ("engine_turbine_small", draw_turbine_small),
    ("external_tank_aluminum", draw_external_tank),
    ("folding_frame", draw_folding_frame),
    ("folding_wooden_frame", draw_folding_wooden_frame),
    ("foldxlframe", draw_foldxlframe),
    ("frame_concrete_mix", draw_frame_mix),
    ("frame_wood_handle", draw_frame_wood_handle),
    ("frame_wood_light", draw_frame_wood_light),
    ("frame_wood_light_handle", draw_frame_wood_light_handle),
    ("fridgetank", draw_fridgetank),
    ("fridgetank_large", draw_fridgetank_large),
    ("fuel_bunker", draw_fuel_bunker),
    ("hand_controls", draw_hand_controls),
    ("handheld_battery_mount", draw_battery_mount),
    ("heattank", draw_heattank),
    ("horn_big", draw_horn_big),
    ("horn_car", draw_horn_car),
    ("improvised_frame", draw_improvised_frame),
    ("inboard_mirror", draw_inboard_mirror),
    ("inflatable_airbag", draw_airbag),
    ("integrated_cooler", draw_cooler),
    ("integrated_heater_small", draw_heater_small),
    ("jumper_cable_debug", draw_jumper_debug),
    ("large_pressure_tank", draw_pressure_tank),
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
    sheet.save(os.path.join(BASE, "preview_veh2.png"))
    print("wrote preview_veh2.png", sheet.size)


if __name__ == "__main__":
    main()
