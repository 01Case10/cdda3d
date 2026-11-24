#!/usr/bin/env python3
"""
Draw the missing vehicle part sprites (batch VEH4, the final vehicle
batch) from their game descriptions.

Sources (fork-src/cdda/data/json/, CDDA 6a3e59e0) -- the 48 ids below
are vehicle parts missing from UltiCa in the shipped universe
(coverage-audit/missing.json) with no folder yet under new-tiles/.
Each is drawn as the part itself, an object on a transparent
background, in flat UltiCa style at native 32x32 (vehicle parts
present at tile size; no sheet-size rule for this category):

  stowed_* (14)               workshop machines and appliances tied
                              down with rope lashings for transport:
                              fridges/freezers, hydraulic press,
                              jointer, mitre saw, oven, oxygen
                              concentrator, planer, router, table
                              saw, high-pressure compressor unit
  tank / tank_medium /
  tank_small / tank_little    vehicle liquid tanks graded by
                              capacity: 60 L steel, 20 L steel,
                              10 L plastic, 2 L steel bottle
  tank_barrel                 wooden stave barrel
  tank_30gal_drum             dark steel drum; the two aluminum
  tank_30gal_drum_aluminum    drums in light silver (30 gal, and
  tank_55gal_drum_aluminum    the taller 55 gal)
  xl_pressure_tank            large horizontal capsule gas tank
  towel_hanger                bar with a draped towel
  veh_binoculars              binoculars secured near the seat
  veh_*_weather_station (3)   vehicle-mounted counterparts of the
                              VEH1 ap_* stations (same family
                              language, roof-rack mount bar added)
  veh_tools_workshop          workbench station with drawers
  vehicle_alarm               security module + key fob, red LED
  vehicle_clock               makeshift analog clock
  water_wheel / xl_water_wheel
                              wooden paddle wheels (VEH1 family)
  wheel_caster_large          swivel caster: plate, fork, wheel
  wheel_skateboard            a set of four small wheels
  wheel_mount_* (10)          hub/axle mounts, distinct from wheels:
                              plates with holes (light, wood), hub
                              assemblies with bolt circles (medium,
                              heavy, concrete-mix pair), skateboard
                              trucks; steerable variants get a
                              kingpin knuckle and steering arm
  xedra_genset                spartan prototype genset, DARPA stamp
  yoke_harness /
  yoke_harness_heavyduty      wooden yoke + straps; heavy steel
                              beams + straps

Bodies are fully opaque (alpha only {0, 255}); translucency is
faked with pale opaque colors.  Deterministic: per-sprite
random.Random(crc32(id)).
"""

import json
import math
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
def R(d, box, fill, outline=None, w=1, o=None):
    d.rectangle(box, fill=fill, outline=o if o is not None else outline,
                width=w)


def RR(d, box, r, fill, outline=None, w=1):
    d.rounded_rectangle(box, radius=r, fill=fill, outline=outline, width=w)


def E(d, box, fill, outline=None, w=1, o=None):
    d.ellipse(box, fill=fill, outline=o if o is not None else outline,
              width=w)


def P(d, pts, fill, outline=None, o=None):
    d.polygon(pts, fill=fill, outline=o if o is not None else outline)


def L(d, pts, fill, w=1):
    d.line(pts, fill=fill, width=w)


def speckle(d, rng, box, color, n):
    x0, y0, x1, y1 = box
    for _ in range(n):
        x = rng.randint(x0, x1)
        y = rng.randint(y0, y1)
        d.point((x, y), fill=color)


STEEL_L = (188, 193, 198, 255)
STEEL = (148, 152, 158, 255)
STEEL_D = (92, 96, 102, 255)
STEEL_XD = (52, 54, 60, 255)
ALU_L = (216, 220, 226, 255)
ALU = (178, 183, 190, 255)
ALU_D = (138, 142, 150, 255)
IRON = (58, 60, 64, 255)
IRON_L = (84, 87, 92, 255)
RUST = (146, 84, 50, 255)
BRASS = (186, 152, 82, 255)
COPPER = (176, 96, 44, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (110, 76, 42, 255)
WOOD_L = (178, 138, 88, 255)
ROPE = (198, 172, 120, 255)
ROPE_D = (164, 138, 88, 255)
YELLOW = (216, 167, 34, 255)
YELLOW_L = (240, 198, 84, 255)
YELLOW_D = (172, 128, 22, 255)
BROWN = (122, 84, 52, 255)
BROWN_D = (92, 62, 38, 255)
LBLUE = (168, 200, 230, 255)      # data color light_blue
LBLUE_D = (110, 148, 188, 255)
GLASS = (196, 228, 238, 255)      # pale opaque "glass"
GLASS_D = (140, 180, 196, 255)
WHITE = (232, 232, 230, 255)
OFFWHITE = (214, 212, 204, 255)
RED = (196, 62, 52, 255)
RED_L = (226, 106, 96, 255)
MAGENTA = (196, 84, 168, 255)
CYAN = (120, 216, 222, 255)
CYAN_D = (70, 150, 158, 255)
GREEN_L = (140, 200, 110, 255)
BLACK = (34, 34, 38, 255)


def lash(d, box):
    """Rope lashings over a stowed item: an X across the front and a
    horizontal band -- 'secured for transportation'."""
    x0, y0, x1, y1 = box
    L(d, [(x0 + 1, y0 + 1), (x1 - 1, y1 - 1)], ROPE, 2)
    L(d, [(x1 - 1, y0 + 1), (x0 + 1, y1 - 1)], ROPE, 2)
    ym = (y0 + y1) // 2
    L(d, [(x0, ym), (x1, ym)], ROPE_D, 1)
    d.point(((x0 + x1) // 2, ym), fill=ROPE_D)


# ---------------------------------------------------------------- stowed
def stowed_glass_fridge_double(d, rng):
    R(d, (6, 4, 26, 29), LBLUE_D)                    # cabinet
    R(d, (7, 5, 15, 27), GLASS, LBLUE_D)             # left glass door
    R(d, (17, 5, 25, 27), GLASS, LBLUE_D)            # right glass door
    L(d, [(9, 9), (13, 24)], GLASS_D, 1)             # glass sheen
    L(d, [(19, 9), (23, 24)], GLASS_D, 1)
    for y in (12, 19):                               # shelf lines
        L(d, [(8, y), (14, y)], LBLUE, 1)
        L(d, [(18, y), (24, y)], LBLUE, 1)
    L(d, [(15, 8), (15, 24)], STEEL_D, 2)            # center handles
    L(d, [(17, 8), (17, 24)], STEEL_D, 2)
    lash(d, (6, 4, 26, 29))


def stowed_hd_compressor_unit(d, rng):
    R(d, (5, 17, 27, 25), STEEL, STEEL_XD)           # horizontal tank
    E(d, (3, 17, 8, 25), STEEL_D)
    E(d, (24, 17, 29, 25), STEEL_D)
    R(d, (11, 9, 21, 17), IRON, STEEL_XD)            # compressor block
    R(d, (13, 6, 19, 9), STEEL_D)                    # motor on top
    E(d, (22, 8, 27, 13), WHITE, STEEL_D)            # pressure gauge
    d.point((24, 10), fill=RED)
    L(d, [(21, 13), (25, 17)], COPPER, 2)            # discharge pipe
    R(d, (8, 25, 11, 28), STEEL_XD)                  # feet
    R(d, (21, 25, 24, 28), STEEL_XD)
    speckle(d, rng, (6, 19, 26, 24), STEEL_D, 6)
    lash(d, (5, 6, 27, 28))


def stowed_heavy_duty_freezer(d, rng):
    R(d, (5, 6, 27, 29), LBLUE, LBLUE_D)             # big upright body
    R(d, (5, 6, 27, 11), WHITE, LBLUE_D)             # thick lid band
    L(d, [(9, 13), (13, 13)], STEEL_D, 2)            # handle
    R(d, (8, 22, 24, 27), LBLUE_D)                   # vent grille
    for x in range(10, 24, 3):
        L(d, [(x, 23), (x, 26)], LBLUE, 1)
    d.point((24, 8), fill=(90, 220, 90, 255))        # power light
    lash(d, (5, 6, 27, 29))


def stowed_heavy_duty_fridge(d, rng):
    R(d, (7, 3, 25, 29), LBLUE, LBLUE_D)             # tall upright body
    L(d, [(7, 10), (25, 10)], LBLUE_D, 1)            # freezer seam
    L(d, [(22, 12), (22, 22)], STEEL_D, 2)           # long door handle
    R(d, (9, 24, 23, 27), LBLUE_D)                   # bottom vent
    for x in range(11, 23, 3):
        L(d, [(x, 25), (x, 26)], LBLUE, 1)
    d.point((9, 5), fill=(90, 220, 90, 255))
    lash(d, (7, 3, 25, 29))


def stowed_hydraulic_press(d, rng):
    R(d, (5, 5, 8, 28), BLACK, STEEL_XD)             # columns
    R(d, (24, 5, 27, 28), BLACK, STEEL_XD)
    R(d, (5, 3, 27, 8), BLACK, STEEL_XD)             # top crossbeam
    R(d, (13, 8, 19, 14), RED, STEEL_XD)             # hydraulic cylinder
    R(d, (15, 14, 17, 19), STEEL_L)                  # ram
    R(d, (11, 19, 21, 21), STEEL_D)                  # press plate
    R(d, (5, 25, 27, 28), BLACK, STEEL_XD)           # base bed
    E(d, (7, 10, 11, 14), WHITE, STEEL_D)            # gauge
    d.point((9, 12), fill=RED)
    lash(d, (5, 3, 27, 28))


def stowed_jointer(d, rng):
    R(d, (8, 20, 12, 28), STEEL_XD)                  # legs
    R(d, (20, 20, 24, 28), STEEL_XD)
    R(d, (4, 14, 28, 20), STEEL_L, STEEL_D)          # long flat beds
    R(d, (14, 15, 18, 19), STEEL_XD)                 # cutter head gap
    L(d, [(4, 17), (28, 17)], STEEL, 1)
    R(d, (6, 8, 26, 12), YELLOW, YELLOW_D)           # rear fence rail
    R(d, (6, 12, 26, 14), YELLOW_D)
    d.point((24, 16), fill=MAGENTA)                  # accent knobs
    d.point((8, 16), fill=MAGENTA)
    E(d, (14, 22, 18, 26), MAGENTA, STEEL_XD)        # handwheel accent
    lash(d, (4, 8, 28, 28))


def stowed_minifreezer(d, rng):
    R(d, (8, 9, 24, 27), LBLUE, LBLUE_D)             # chest body
    R(d, (8, 9, 24, 13), WHITE, LBLUE_D)             # lid
    L(d, [(13, 15), (19, 15)], STEEL_D, 2)           # recessed handle
    speckle(d, rng, (10, 18, 22, 25), WHITE, 5)      # frost
    lash(d, (8, 9, 24, 27))


def stowed_minifridge(d, rng):
    R(d, (8, 7, 24, 28), LBLUE, LBLUE_D)             # upright cube
    L(d, [(8, 12), (24, 12)], LBLUE_D, 1)            # door seam
    L(d, [(21, 14), (21, 20)], STEEL_D, 2)           # handle
    d.point((10, 9), fill=(90, 220, 90, 255))
    lash(d, (8, 7, 24, 28))


def stowed_mitresaw(d, rng):
    R(d, (5, 24, 27, 28), YELLOW_D, STEEL_XD)        # base table
    L(d, [(8, 24), (24, 24)], STEEL_XD, 1)           # fence line
    P(d, [(16, 24), (16, 10), (20, 10), (21, 24)], YELLOW, YELLOW_D)
    E(d, (11, 8, 23, 20), CYAN_D, STEEL_XD)          # blade guard disc
    E(d, (14, 11, 20, 17), STEEL_L, STEEL_D)         # blade
    d.point((17, 14), fill=STEEL_XD)
    R(d, (19, 5, 26, 10), YELLOW, YELLOW_D)          # motor housing
    lash(d, (5, 5, 27, 28))


def stowed_oven(d, rng):
    R(d, (6, 8, 26, 28), STEEL_D, STEEL_XD)          # dark gray body
    R(d, (6, 5, 26, 9), WHITE, STEEL_D)              # white hob top
    E(d, (8, 5, 12, 9), STEEL_XD)                    # burner rings
    E(d, (20, 5, 24, 9), STEEL_XD)
    R(d, (9, 14, 23, 24), BLACK, STEEL_L)            # glass door
    L(d, [(11, 16), (15, 22)], GLASS_D, 1)
    L(d, [(8, 12), (24, 12)], STEEL_L, 2)            # handle bar
    for x in (10, 14, 18, 22):                       # knobs
        d.point((x, 26), fill=WHITE)
    lash(d, (6, 5, 26, 28))


def stowed_oxygen_concentrator(d, rng):
    RR(d, (9, 6, 23, 28), 2, OFFWHITE, STEEL_D)      # cabinet
    R(d, (12, 4, 20, 6), STEEL_D)                    # carry handle
    R(d, (12, 9, 20, 13), BLACK)                     # display
    d.point((14, 11), fill=(90, 220, 90, 255))
    d.point((17, 11), fill=CYAN)
    for y in (17, 20, 23):                           # vent slots
        L(d, [(12, y), (20, y)], STEEL_D, 1)
    L(d, [(20, 15), (25, 18), (25, 24)], CYAN_D, 1)  # coiled tube stub
    lash(d, (9, 4, 25, 28))


def stowed_planer(d, rng):
    P(d, [(11, 4), (21, 4), (23, 9), (9, 9)], WHITE, STEEL_D)  # top arch
    R(d, (7, 9, 25, 22), YELLOW, YELLOW_D)           # body
    R(d, (7, 15, 25, 20), STEEL_XD)                  # feed slot
    P(d, [(12, 17), (28, 19), (28, 22), (12, 20)], WOOD_L, WOOD_D)
    L(d, [(14, 18), (26, 20)], WOOD, 1)              # board grain
    R(d, (9, 22, 12, 26), STEEL_XD)                  # feet
    R(d, (20, 22, 23, 26), STEEL_XD)
    d.point((16, 12), fill=STEEL_XD)
    lash(d, (7, 4, 28, 26))


def stowed_router(d, rng):
    R(d, (12, 6, 20, 18), YELLOW, YELLOW_D)          # motor body
    R(d, (12, 6, 20, 9), YELLOW_D)
    E(d, (5, 12, 10, 16), GREEN_L, STEEL_XD)         # knob handles
    E(d, (22, 12, 27, 16), GREEN_L, STEEL_XD)
    L(d, [(10, 14), (12, 14)], STEEL_D, 2)
    L(d, [(20, 14), (22, 14)], STEEL_D, 2)
    R(d, (8, 18, 24, 21), GREEN_L, STEEL_XD)         # base plate
    R(d, (15, 21, 17, 25), STEEL_L, STEEL_D)         # bit
    L(d, [(20, 8), (26, 4)], BLACK, 1)               # cord
    lash(d, (5, 4, 27, 25))


def stowed_tablesaw(d, rng):
    R(d, (8, 16, 24, 28), YELLOW, YELLOW_D)          # cabinet
    R(d, (4, 11, 28, 16), STEEL_L, STEEL_D)          # table top
    E(d, (13, 6, 19, 13), STEEL_L, STEEL_D)          # blade poking up
    L(d, [(16, 7), (16, 12)], STEEL_D, 1)
    L(d, [(5, 13), (27, 13)], RED, 2)                # red fence rail
    d.point((11, 22), fill=RED)                      # switch
    d.point((21, 22), fill=STEEL_XD)
    lash(d, (4, 6, 28, 28))


# ---------------------------------------------------------------- tanks
def tank_vessel(d, rng, x0, y0, x1, y1, body, dark, light, ribs=2):
    RR(d, (x0, y0, x1, y1), 3, body, dark)
    L(d, [(x0 + 3, y0 + 2), (x0 + 3, y1 - 2)], light, 1)   # highlight
    for i in range(ribs):
        y = y0 + (y1 - y0) * (i + 1) // (ribs + 1)
        L(d, [(x0 + 1, y), (x1 - 1, y)], dark, 1)
    cx = (x0 + x1) // 2
    R(d, (cx - 2, y0 - 3, cx + 2, y0), dark)               # filler neck
    R(d, (cx - 3, y0 - 4, cx + 3, y0 - 2), dark)           # cap


def draw_tank(d, rng):          # 60 L steel
    tank_vessel(d, rng, 8, 8, 24, 28, STEEL, STEEL_D, STEEL_L, ribs=2)
    speckle(d, rng, (10, 12, 22, 26), STEEL_D, 6)


def draw_tank_medium(d, rng):   # 20 L steel
    tank_vessel(d, rng, 10, 11, 22, 28, STEEL, STEEL_D, STEEL_L, ribs=1)
    speckle(d, rng, (12, 15, 20, 26), STEEL_D, 4)


def draw_tank_small(d, rng):    # 10 L plastic
    RR(d, (11, 13, 21, 28), 2, LBLUE, LBLUE_D)   # plastic body
    L(d, [(13, 15), (13, 26)], WHITE, 1)
    R(d, (13, 10, 19, 13), LBLUE_D)              # molded handle/neck
    R(d, (14, 8, 18, 10), STEEL_D)               # cap
    L(d, [(12, 20), (20, 20)], LBLUE_D, 1)


def draw_tank_little(d, rng):   # 2 L steel bottle
    RR(d, (13, 17, 19, 28), 2, STEEL, STEEL_D)
    L(d, [(14, 19), (14, 26)], STEEL_L, 1)
    R(d, (14, 14, 18, 17), STEEL_D)              # neck
    R(d, (14, 12, 18, 14), STEEL_XD)             # cap


def draw_tank_barrel(d, rng):
    P(d, [(11, 6), (21, 6), (24, 16), (21, 26), (11, 26), (8, 16)],
      WOOD, WOOD_D)                                       # bulged staves
    for x in (13, 16, 19):                                # stave seams
        L(d, [(x, 7), (x - (1 if x < 16 else -1 if x > 16 else 0), 25)],
          WOOD_D, 1)
    L(d, [(9, 11), (23, 11)], IRON, 2)                    # hoops
    L(d, [(9, 21), (23, 21)], IRON, 2)
    E(d, (13, 4, 19, 8), WOOD_L, WOOD_D)                  # top head
    speckle(d, rng, (11, 13, 21, 24), WOOD_D, 5)


def drum(d, rng, x0, y0, x1, y1, body, dark, light, ribs):
    R(d, (x0, y0, x1, y1), body, dark)
    E(d, (x0, y0 - 2, x1, y0 + 3), light, dark)           # rim top
    for i in range(ribs):
        y = y0 + (y1 - y0) * (i + 1) // (ribs + 1)
        L(d, [(x0, y), (x1, y)], dark, 2)                 # rolling ribs
        L(d, [(x0, y + 2), (x1, y + 2)], light, 1)
    L(d, [(x0 + 3, y0 + 3), (x0 + 3, y1 - 2)], light, 1)
    d.point((x1 - 4, y0 + 1), fill=dark)                  # bung


def draw_tank_30gal_drum(d, rng):
    drum(d, rng, 9, 8, 23, 28, STEEL_D, STEEL_XD, STEEL, ribs=2)
    speckle(d, rng, (11, 12, 21, 26), STEEL_XD, 6)


def draw_tank_30gal_drum_aluminum(d, rng):
    drum(d, rng, 9, 8, 23, 28, ALU, ALU_D, ALU_L, ribs=2)
    speckle(d, rng, (11, 12, 21, 26), ALU_D, 4)


def draw_tank_55gal_drum_aluminum(d, rng):
    drum(d, rng, 7, 5, 25, 29, ALU, ALU_D, ALU_L, ribs=3)
    speckle(d, rng, (9, 9, 23, 27), ALU_D, 5)


def draw_xl_pressure_tank(d, rng):
    R(d, (8, 22, 12, 27), STEEL_XD)                  # saddle feet
    R(d, (20, 22, 24, 27), STEEL_XD)
    RR(d, (3, 10, 29, 23), 6, STEEL, STEEL_D)        # horizontal capsule
    E(d, (3, 10, 10, 23), STEEL_L, STEEL_D)          # end cap shading
    L(d, [(11, 12), (11, 21)], STEEL_D, 1)           # weld seam
    L(d, [(6, 13), (26, 13)], STEEL_L, 1)            # highlight
    R(d, (15, 6, 18, 10), STEEL_D)                   # valve neck
    E(d, (19, 4, 24, 9), WHITE, STEEL_D)             # gauge
    d.point((21, 6), fill=RED)
    L(d, [(18, 8), (21, 8)], BRASS, 1)


# ---------------------------------------------------------------- misc parts
def draw_towel_hanger(d, rng):
    R(d, (5, 7, 8, 11), STEEL_D)                     # mounts
    R(d, (24, 7, 27, 11), STEEL_D)
    L(d, [(6, 9), (26, 9)], STEEL_L, 2)              # bar
    P(d, [(10, 9), (22, 9), (22, 24), (19, 26), (13, 26), (10, 24)],
      OFFWHITE, STEEL_D)                             # draped towel front
    P(d, [(10, 9), (22, 9), (22, 13), (10, 13)], WHITE, None)
    L(d, [(13, 14), (13, 24)], STEEL_L, 1)           # fold lines
    L(d, [(19, 14), (19, 24)], STEEL_L, 1)
    L(d, [(11, 21), (21, 21)], CYAN_D, 1)            # towel stripe
    L(d, [(11, 23), (21, 23)], CYAN_D, 1)


def draw_veh_binoculars(d, rng):
    L(d, [(8, 4), (16, 8), (24, 4)], BLACK, 1)       # strap
    RR(d, (6, 9, 14, 24), 2, IRON, STEEL_XD)         # barrels
    RR(d, (18, 9, 26, 24), 2, IRON, STEEL_XD)
    R(d, (14, 13, 18, 18), STEEL_D)                  # bridge
    E(d, (15, 14, 17, 16), STEEL_L)                  # focus wheel
    E(d, (7, 19, 13, 25), GLASS_D, STEEL_XD)         # objective lenses
    E(d, (19, 19, 25, 25), GLASS_D, STEEL_XD)
    d.point((9, 21), fill=GLASS)
    d.point((21, 21), fill=GLASS)
    R(d, (8, 6, 12, 9), STEEL_D)                     # eyepieces
    R(d, (20, 6, 24, 9), STEEL_D)


def weather_station(d, rng, style="std"):
    """VEH1 family treatment, plus a roof-rack mount bar (vehicle
    mounted counterpart of the ap_* stations)."""
    if style == "std":
        base_c, acc = YELLOW, STEEL_D
    elif style == "robofac":
        base_c, acc = BROWN, BROWN_D
    else:  # xedra
        base_c, acc = (40, 42, 48, 255), CYAN
    R(d, (10, 20, 22, 27), base_c)                   # base box
    R(d, (10, 20, 22, 22), acc, o=None)
    d.point((13, 24), fill=(90, 220, 90, 255))
    d.point((19, 24), fill=(230, 90, 70, 255))
    R(d, (15, 6, 17, 20), STEEL)                     # telescoping mast
    L(d, [(15, 11), (17, 11)], STEEL_D)
    L(d, [(15, 16), (17, 16)], STEEL_D)
    E(d, (14, 3, 18, 7), STEEL_D)                    # anemometer hub
    for a in (270, 30, 150):                         # cups
        r = math.radians(a)
        cx, cy = 16 + 5 * math.cos(r), 5 + 4 * math.sin(r)
        E(d, (cx - 2, cy - 1, cx + 2, cy + 2), STEEL_L)
    L(d, [(17, 9), (24, 9)], STEEL_D, 1)             # wind vane
    P(d, [(24, 7), (28, 9), (24, 11)], acc, o=None)
    R(d, (11, 12, 15, 16), STEEL_L)                  # sensor box
    if style == "robofac":
        R(d, (18, 12, 22, 16), STEEL_L)              # second pod
        E(d, (12, 15, 20, 20), STEEL_D)              # dome on base
    if style == "xedra":
        L(d, [(16, 20), (16, 24)], CYAN, 1)
        R(d, (12, 13, 14, 15), BLACK, o=CYAN)
    R(d, (7, 27, 25, 29), STEEL_XD)                  # roof-rack mount bar
    L(d, [(10, 29), (10, 30)], STEEL_D, 2)
    L(d, [(22, 29), (22, 30)], STEEL_D, 2)


def draw_veh_tools_workshop(d, rng):
    R(d, (5, 5, 27, 13), STEEL_D)                    # pegboard back
    for x in range(8, 26, 4):                        # tool silhouettes
        L(d, [(x, 7), (x, 11)], CYAN, 1)
        d.point((x, 6), fill=STEEL_L)
    R(d, (4, 13, 28, 16), WOOD, WOOD_D)              # bench top
    R(d, (5, 16, 16, 28), STEEL, STEEL_D)            # drawer bank
    for y in (19, 23):                               # drawers
        R(d, (7, y, 14, y + 2), STEEL_L, STEEL_D)
        d.point((10, y + 1), fill=STEEL_XD)
    R(d, (18, 16, 27, 28), STEEL_D)                  # cabinet + vise
    R(d, (20, 12, 26, 16), IRON, STEEL_XD)
    d.point((23, 18), fill=CYAN)                     # power connector
    d.point((23, 21), fill=YELLOW_L)


def draw_vehicle_alarm(d, rng):
    RR(d, (6, 10, 19, 22), 2, IRON, STEEL_XD)        # control module
    d.point((9, 13), fill=RED_L)                     # armed LED
    L(d, [(15, 12), (15, 19)], STEEL_D, 1)
    L(d, [(8, 20), (24, 5)], STEEL_D, 1)             # antenna wire
    RR(d, (21, 15, 27, 26), 2, BLACK, STEEL_D)       # key fob
    d.point((24, 18), fill=RED_L)
    d.point((24, 21), fill=STEEL_L)
    d.point((24, 24), fill=STEEL_L)
    E(d, (22, 12, 25, 15), None, o=STEEL_L, w=1)     # fob ring


def draw_vehicle_clock(d, rng):
    E(d, (7, 5, 25, 23), STEEL_D, STEEL_XD)          # rim
    E(d, (9, 7, 23, 21), WHITE, STEEL_D)             # face
    for a in range(0, 360, 30):                      # ticks
        r = math.radians(a)
        x = 16 + 6 * math.cos(r)
        y = 14 + 6 * math.sin(r)
        d.point((int(x), int(y)), fill=STEEL_XD)
    L(d, [(16, 14), (16, 9)], BLACK, 1)              # hands ~10:09
    L(d, [(16, 14), (20, 16)], BLACK, 1)
    d.point((16, 14), fill=RED)
    L(d, [(10, 23), (7, 28)], COPPER, 1)             # makeshift wires
    L(d, [(22, 23), (25, 28)], COPPER, 1)
    R(d, (20, 4, 25, 7), ROPE, ROPE_D)               # tape patch


def water_wheel(d, rng, xl=False):
    """Same construction as the VEH1 ap_* water wheels."""
    cx, cy = 16, 15
    r = 13 if xl else 11
    L(d, [(cx - r + 2, 29), (cx, cy), (cx + r - 2, 29)], WOOD_D, 3)
    E(d, (cx - r, cy - r, cx + r, cy + r), None, o=WOOD_D, w=2)
    ri = r - 4
    E(d, (cx - ri, cy - ri, cx + ri, cy + ri), None, o=WOOD, w=1)
    n = 10 if xl else 8
    for i in range(n):
        a = 2 * math.pi * i / n
        x2, y2 = cx + r * math.cos(a), cy + r * math.sin(a)
        L(d, [(cx, cy), (x2, y2)], WOOD, 1)
        px, py = cx + (r + 1) * math.cos(a), cy + (r + 1) * math.sin(a)
        R(d, (px - 2, py - 1, px + 2, py + 1), WOOD_D, o=None)
    E(d, (cx - 3, cy - 3, cx + 3, cy + 3), STEEL_D)
    d.point((cx, cy), fill=ALU_L)
    if xl:                                            # support posts
        R(d, (3, 18, 5, 29), WOOD_D)
        R(d, (27, 18, 29, 29), WOOD_D)


# ---------------------------------------------------------------- wheels & mounts
def draw_wheel_caster_large(d, rng):
    R(d, (9, 4, 23, 7), STEEL, STEEL_D)              # swivel plate
    E(d, (13, 6, 19, 11), STEEL_D)                   # swivel bearing
    P(d, [(11, 10), (14, 10), (14, 20), (11, 20)], STEEL, STEEL_D)
    P(d, [(21, 10), (18, 10), (18, 20), (21, 20)], STEEL, STEEL_D)
    E(d, (10, 17, 22, 29), IRON, STEEL_XD)           # wheel
    E(d, (13, 20, 19, 26), STEEL_D)
    d.point((16, 23), fill=ALU_L)                    # axle bolt
    for x in (10, 15, 20):                           # plate bolts
        d.point((x, 5), fill=STEEL_XD)


def draw_wheel_skateboard(d, rng):
    for (x, y) in ((8, 9), (20, 9), (8, 21), (20, 21)):   # set of four
        E(d, (x - 4, y - 4, x + 4, y + 4), OFFWHITE, STEEL_D)
        E(d, (x - 1, y - 1, x + 1, y + 1), STEEL_D)
        d.point((x - 2, y - 2), fill=WHITE)


def hub_mount(d, rng, kind, steerable):
    """Hub/axle mount -- the fixture a wheel bolts to, not a wheel."""
    if kind == "light":          # pierced plate for a bike wheel
        RR(d, (11, 6, 21, 26), 2, STEEL, STEEL_D)
        E(d, (14, 9, 18, 13), BLACK, STEEL_D)        # axle hole
        d.point((16, 18), fill=BLACK)
        d.point((16, 22), fill=BLACK)
        L(d, [(13, 8), (13, 24)], STEEL_L, 1)
    elif kind == "wood":         # pierced wooden plate
        RR(d, (11, 6, 21, 26), 2, WOOD, WOOD_D)
        E(d, (14, 9, 18, 13), BLACK, WOOD_D)
        d.point((16, 18), fill=BLACK)
        d.point((16, 22), fill=BLACK)
        L(d, [(13, 8), (13, 24)], WOOD_L, 1)
        speckle(d, rng, (12, 14, 20, 25), WOOD_D, 5)
    elif kind == "skate":        # skateboard truck
        for y0 in (7, 19):                           # a pair of trucks
            R(d, (8, y0, 24, y0 + 2), STEEL, STEEL_D)      # baseplate
            P(d, [(13, y0 + 2), (19, y0 + 2), (17, y0 + 6),
                  (15, y0 + 6)], STEEL_D)                  # hanger neck
            L(d, [(6, y0 + 7), (26, y0 + 7)], STEEL_L, 2)  # axle
            d.point((16, y0 + 4), fill=STEEL_XD)           # kingpin
        return
    else:                        # hub assembly: flange + bolt circle
        big = kind == "heavy"
        cx, cy = (14, 16) if steerable else (16, 16)
        r = 9 if big else 7
        E(d, (cx - r, cy - r, cx + r, cy + r), STEEL, STEEL_D)
        E(d, (cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2),
          STEEL_D, STEEL)
        n = 6 if big else 5
        for i in range(n):                           # lug bolts
            a = 2 * math.pi * i / n - math.pi / 2
            bx, by = cx + (r - 3) * math.cos(a), cy + (r - 3) * math.sin(a)
            E(d, (bx - 1, by - 1, bx + 1, by + 1), STEEL_L, STEEL_XD)
        E(d, (cx - 2, cy - 2, cx + 2, cy + 2), STEEL_XD)   # axle bore
        d.point((cx, cy), fill=BLACK)
        if kind == "mix":                            # paired small mounts
            E(d, (22, 20, 29, 27), STEEL, STEEL_D)
            E(d, (24, 22, 27, 25), STEEL_D)
            d.point((25, 23), fill=STEEL_L)
    if steerable:                                    # kingpin knuckle + arm
        R(d, (24, 8, 28, 24), STEEL_D, STEEL_XD)     # kingpin housing
        L(d, [(26, 6), (26, 26)], STEEL_L, 1)
        P(d, [(24, 20), (18, 26), (21, 28), (26, 24)], STEEL_D)
        d.point((26, 10), fill=ALU_L)
        d.point((26, 22), fill=ALU_L)


def draw_xedra_genset(d, rng):
    R(d, (6, 8, 26, 27), STEEL_D, STEEL_XD)          # spartan casing
    R(d, (6, 8, 26, 11), CYAN_D)                     # xedra cyan band
    for y in (14, 17, 20):                           # vents
        L(d, [(9, y), (16, y)], STEEL_XD, 1)
    R(d, (19, 14, 24, 20), BLACK, CYAN_D)            # outlet panel
    d.point((21, 16), fill=WHITE)                    # the ':' outlets
    d.point((21, 18), fill=WHITE)
    R(d, (9, 23, 14, 25), BLACK)                     # DARPA stamp plate
    d.point((10, 24), fill=CYAN)
    d.point((12, 24), fill=CYAN)
    E(d, (21, 22, 25, 26), STEEL_XD)                 # fuel cap
    L(d, [(8, 8), (8, 27)], STEEL_L, 1)
    speckle(d, rng, (7, 12, 25, 26), STEEL_XD, 5)


def draw_yoke_harness(d, rng, heavy=False):
    if heavy:
        bar, dark, strap = STEEL, STEEL_D, IRON_L
        R(d, (4, 9, 28, 13), bar, dark)              # twin steel beams
        R(d, (4, 15, 28, 18), bar, dark)
        for x in (8, 15, 22):                        # cross bolts
            d.point((x, 11), fill=STEEL_XD)
            d.point((x, 16), fill=STEEL_XD)
        top = 18
    else:
        bar, dark, strap = WOOD, WOOD_D, BROWN
        P(d, [(4, 12), (10, 9), (22, 9), (28, 12), (28, 15),
              (22, 12), (10, 12), (4, 15)], bar, dark)   # curved yoke
        speckle(d, rng, (6, 10, 26, 14), WOOD_D, 6)
        top = 15
    for x in (10, 22):                               # hanging straps
        P(d, [(x - 3, top), (x + 3, top), (x + 2, 26), (x, 28),
              (x - 2, 26)], strap, STEEL_XD)
        E(d, (x - 1, 21, x + 1, 24), None, o=BRASS, w=1)  # rings
    L(d, [(13, top), (19, top)], ROPE, 2)            # padding wrap


RENDERERS = [
    ("stowed_glass_fridge_double", stowed_glass_fridge_double),
    ("stowed_hd_compressor_unit", stowed_hd_compressor_unit),
    ("stowed_heavy_duty_freezer", stowed_heavy_duty_freezer),
    ("stowed_heavy_duty_fridge", stowed_heavy_duty_fridge),
    ("stowed_hydraulic_press", stowed_hydraulic_press),
    ("stowed_jointer", stowed_jointer),
    ("stowed_minifreezer", stowed_minifreezer),
    ("stowed_minifridge", stowed_minifridge),
    ("stowed_mitresaw", stowed_mitresaw),
    ("stowed_oven", stowed_oven),
    ("stowed_oxygen_concentrator", stowed_oxygen_concentrator),
    ("stowed_planer", stowed_planer),
    ("stowed_router", stowed_router),
    ("stowed_tablesaw", stowed_tablesaw),
    ("tank", draw_tank),
    ("tank_30gal_drum", draw_tank_30gal_drum),
    ("tank_30gal_drum_aluminum", draw_tank_30gal_drum_aluminum),
    ("tank_55gal_drum_aluminum", draw_tank_55gal_drum_aluminum),
    ("tank_barrel", draw_tank_barrel),
    ("tank_little", draw_tank_little),
    ("tank_medium", draw_tank_medium),
    ("tank_small", draw_tank_small),
    ("towel_hanger", draw_towel_hanger),
    ("veh_binoculars", draw_veh_binoculars),
    ("veh_mobile_weather_station",
     lambda d, r: weather_station(d, r, "std")),
    ("veh_robofac_mobile_weather_station",
     lambda d, r: weather_station(d, r, "robofac")),
    ("veh_tools_workshop", draw_veh_tools_workshop),
    ("veh_xedra_mobile_weather_station",
     lambda d, r: weather_station(d, r, "xedra")),
    ("vehicle_alarm", draw_vehicle_alarm),
    ("vehicle_clock", draw_vehicle_clock),
    ("water_wheel", lambda d, r: water_wheel(d, r, False)),
    ("wheel_caster_large", draw_wheel_caster_large),
    ("wheel_mount_concrete_mix",
     lambda d, r: hub_mount(d, r, "mix", False)),
    ("wheel_mount_heavy", lambda d, r: hub_mount(d, r, "heavy", False)),
    ("wheel_mount_heavy_steerable",
     lambda d, r: hub_mount(d, r, "heavy", True)),
    ("wheel_mount_light", lambda d, r: hub_mount(d, r, "light", False)),
    ("wheel_mount_light_steerable",
     lambda d, r: hub_mount(d, r, "light", True)),
    ("wheel_mount_medium", lambda d, r: hub_mount(d, r, "med", False)),
    ("wheel_mount_medium_steerable",
     lambda d, r: hub_mount(d, r, "med", True)),
    ("wheel_mount_skateboard",
     lambda d, r: hub_mount(d, r, "skate", False)),
    ("wheel_mount_wood", lambda d, r: hub_mount(d, r, "wood", False)),
    ("wheel_mount_wood_steerable",
     lambda d, r: hub_mount(d, r, "wood", True)),
    ("wheel_skateboard", draw_wheel_skateboard),
    ("xedra_genset", draw_xedra_genset),
    ("xl_pressure_tank", draw_xl_pressure_tank),
    ("xl_water_wheel", lambda d, r: water_wheel(d, r, True)),
    ("yoke_harness", lambda d, r: draw_yoke_harness(d, r, False)),
    ("yoke_harness_heavyduty", lambda d, r: draw_yoke_harness(d, r, True)),
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
    COLS, ROWS, CELL = 8, 6, 96
    sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL),
                      (24, 24, 28, 255))
    for i, (pid, img) in enumerate(imgs):
        big = img.resize((CELL, CELL), Image.NEAREST)
        sheet.alpha_composite(big, ((i % COLS) * CELL,
                                    (i // COLS) * CELL))
    sheet.save(os.path.join(BASE, "preview_veh4.png"))
    print("wrote preview_veh4.png", sheet.size)


if __name__ == "__main__":
    main()
