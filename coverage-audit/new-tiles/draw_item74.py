#!/usr/bin/env python3
"""ITEM74: seats, costume pieces, vehicle controls/lights, basecamp
props -- CDDA Voxel coverage audit.

Four families, each drawn as itself, front view:

  Seats (items/furniture/seating.json) -- the furniture piece itself:
    upholstered green armchair, steel barstool with leather seat,
    slumped light-blue bean bag, plywood chair on a steel tube frame,
    the plain wooden chair and its black- and white-painted twins,
    a bark-sided log stool, a backless wooden stool.
  Costume / personal (items/armor/misc.json) -- long blue Boston-Chan
    wig, red clown nose, pink clown afro, steel dog tags on a ball
    chain, tefillin (black leather boxes: hand pair mirrored by strap
    coil direction, head piece with straps hanging both sides), a
    green necktie and a skinny black-and-white checkered tie.
  Vehicle controls + lights (items/vehicle/) -- monitor bank,
    drive-by-wire unit, gauge panels, hand-control rods, pedal
    extenders (rubber pads vs taped wooden blocks), robot and turret
    control units, dashboard; headlights by form (car, wide,
    motorcycle bucket, cage-reinforced variants), floodlights,
    red/blue emergency light bars.
  Basecamp "fake" furniture (items/basecamp.json) -- kilns, smoker,
    digester tank, drop hammer, fireplace, forge, woodstove and the
    parkour gym.  These are prop facades, so they are drawn FLAT:
    muted colours, painted-on flames, and a plywood backing board
    peeking out behind each silhouette.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "armchair",
    "barstool",
    "beanbag",
    "chair_plywood",
    "chair_wood",
    "chair_wood_black",
    "chair_wood_white",
    "stool_log",
    "stool_wood",
    "bostonchan_wig",
    "clown_nose",
    "clown_wig",
    "dog_tag",
    "tefillin_hand_l",
    "tefillin_hand_r",
    "tefillin_head",
    "tie_necktie",
    "tie_skinny",
    "camera_control",
    "drive_by_wire_controls",
    "electronics_controls",
    "hand_controls",
    "pedal_extenders",
    "pedal_extenders_makeshift",
    "robot_controls",
    "turret_controls",
    "vehicle_dashboard",
    "car_headlight",
    "car_wide_headlight",
    "directed_floodlight",
    "floodlight",
    "headlight_reinforced",
    "light_emergency_blue",
    "light_emergency_red",
    "motorcycle_headlight",
    "wide_headlight_reinforced",
    "fake_char_kiln",
    "fake_char_smoker",
    "fake_clay_kiln",
    "fake_digester_tank",
    "fake_drop_hammer",
    "fake_fireplace",
    "fake_forge",
    "fake_woodstove",
    "parkour_practice",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 38, 36, 255)
WOOD = (146, 102, 58, 255)
WOOD_DARK = (104, 70, 36, 255)
WOOD_LIGHT = (196, 152, 96, 255)
PLY = (206, 172, 118, 255)          # plywood / pale timber
STEEL = (168, 172, 178, 255)
STEEL_DARK = (96, 100, 108, 255)
CHROME = (216, 220, 226, 255)
HOUSING = (58, 60, 66, 255)         # dark plastic housing
LENS = (240, 244, 224, 255)         # lit lens pale
LENS_DIM = (206, 214, 200, 255)
SCREEN = (150, 205, 225, 255)
BACKING = (188, 152, 102, 255)      # prop-facade plywood backing


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def rrect(d, box, fill, outline=None):
    d.rectangle(box, fill=fill, outline=outline, width=1)


# ------------------------------------------------------------------- seats

def draw_armchair(d, rng):
    green = (74, 128, 66, 255)
    dark = shade(green, 0.72)
    light = shade(green, 1.25)
    # back cushion
    rrect(d, [7, 4, 24, 16], green, INK)
    rrect(d, [10, 6, 21, 13], light)
    d.line([15, 6, 15, 13], fill=dark, width=1)          # cushion seam
    # armrests
    rrect(d, [4, 12, 8, 23], dark, INK)
    rrect(d, [23, 12, 27, 23], dark, INK)
    rrect(d, [4, 12, 8, 14], green)
    rrect(d, [23, 12, 27, 14], green)
    # seat cushion + skirt
    rrect(d, [8, 16, 23, 21], light, INK)
    rrect(d, [6, 21, 25, 26], dark, INK)
    # short wooden feet
    rrect(d, [7, 26, 9, 28], WOOD_DARK)
    rrect(d, [22, 26, 24, 28], WOOD_DARK)


def draw_barstool(d, rng):
    leather = (122, 76, 48, 255)
    # base disc + column + footring
    d.ellipse([7, 24, 24, 28], fill=STEEL_DARK, outline=INK)
    rrect(d, [14, 12, 17, 25], STEEL)
    d.line([14, 12, 14, 25], fill=STEEL_DARK, width=1)
    d.ellipse([9, 17, 22, 20], outline=STEEL_DARK, width=1)
    # round leather seat
    d.ellipse([7, 6, 24, 13], fill=leather, outline=INK)
    d.ellipse([9, 7, 22, 10], fill=shade(leather, 1.3))
    d.line([10, 12, 21, 12], fill=shade(leather, 0.7), width=1)


def draw_beanbag(d, rng):
    blue = (130, 178, 216, 255)
    # slumped sack: wide bottom blob with a gathered top dimple
    d.ellipse([4, 11, 27, 29], fill=blue, outline=INK)
    d.ellipse([6, 13, 25, 29], fill=blue)
    d.ellipse([11, 8, 20, 15], fill=shade(blue, 0.82), outline=INK)
    d.point((15, 11), fill=shade(blue, 0.6))
    # seam + fabric shading
    d.arc([6, 12, 25, 28], start=300, end=60, fill=shade(blue, 0.8),
          width=1)
    for _ in range(10):
        x, y = rng.randint(7, 24), rng.randint(16, 27)
        d.point((x, y), fill=shade(blue, 0.9))
    d.ellipse([8, 20, 13, 25], fill=shade(blue, 1.12))


def draw_chair_plywood(d, rng):
    # bent plywood back + seat on a thin steel tube frame
    d.line([9, 15, 8, 28], fill=STEEL_DARK, width=1)    # legs
    d.line([22, 15, 23, 28], fill=STEEL_DARK, width=1)
    d.line([10, 20, 9, 28], fill=STEEL, width=1)
    d.line([21, 20, 22, 28], fill=STEEL, width=1)
    rrect(d, [9, 4, 22, 14], PLY, INK)                  # back panel
    for gy in (7, 10, 13):                              # grain
        d.line([10, gy, 21, gy], fill=shade(PLY, 0.86), width=1)
    rrect(d, [8, 16, 23, 19], PLY, INK)                 # seat
    d.line([8, 19, 23, 19], fill=WOOD_DARK, width=1)


def wooden_chair(d, body, dark, light):
    # posts, two back slats, seat, four legs -- palette varies
    rrect(d, [9, 5, 10, 20], dark)
    rrect(d, [21, 5, 22, 20], dark)
    rrect(d, [9, 7, 22, 9], body)
    rrect(d, [9, 12, 22, 14], body)
    d.line([9, 7, 22, 7], fill=light, width=1)
    rrect(d, [8, 18, 23, 21], body, dark)
    d.line([8, 18, 23, 18], fill=light, width=1)
    rrect(d, [9, 21, 10, 29], dark)
    rrect(d, [21, 21, 22, 29], dark)
    rrect(d, [12, 21, 13, 27], shade(dark, 1.15))
    rrect(d, [18, 21, 19, 27], shade(dark, 1.15))
    d.line([10, 25, 21, 25], fill=dark, width=1)        # stretcher


def draw_chair_wood(d, rng):
    wooden_chair(d, WOOD, WOOD_DARK, WOOD_LIGHT)


def draw_chair_wood_black(d, rng):
    wooden_chair(d, (52, 50, 52, 255), (30, 29, 31, 255),
                 (92, 90, 94, 255))


def draw_chair_wood_white(d, rng):
    wooden_chair(d, (238, 236, 230, 255), (176, 172, 164, 255),
                 (255, 255, 252, 255))


def draw_stool_log(d, rng):
    # trunk section: ringed top, bark sides
    rrect(d, [7, 9, 24, 26], (110, 78, 46, 255))
    for x in range(8, 24, 3):                           # bark streaks
        d.line([x, 11, x - 1, 25], fill=(84, 58, 32, 255), width=1)
    d.ellipse([7, 5, 24, 12], fill=PLY, outline=INK)
    d.ellipse([10, 6, 21, 11], outline=WOOD, width=1)   # growth rings
    d.ellipse([13, 7, 18, 10], outline=WOOD_LIGHT, width=1)
    d.ellipse([7, 23, 24, 27], fill=(96, 66, 38, 255))
    d.rectangle([7, 9, 24, 24], fill=None)


def draw_stool_wood(d, rng):
    # round seat, splayed legs, ring stretcher -- no back
    d.line([10, 14, 7, 28], fill=WOOD_DARK, width=2)
    d.line([21, 14, 24, 28], fill=WOOD_DARK, width=2)
    d.line([13, 14, 12, 27], fill=WOOD, width=2)
    d.line([18, 14, 19, 27], fill=WOOD, width=2)
    d.line([8, 23, 23, 23], fill=WOOD_DARK, width=1)
    d.ellipse([7, 8, 24, 15], fill=WOOD, outline=INK)
    d.ellipse([9, 9, 22, 12], fill=WOOD_LIGHT)


# ------------------------------------------------------- costume / personal

def draw_bostonchan_wig(d, rng):
    blue = (66, 102, 190, 255)
    light = (116, 148, 226, 255)
    # long straight hair: top mass, blunt fringe, two long side locks
    rrect(d, [7, 3, 24, 10], blue, INK)
    for x in range(8, 24, 3):                           # fringe strands
        d.line([x, 9, x, 11], fill=blue, width=1)
    rrect(d, [5, 8, 10, 27], blue, INK)                 # left lock
    rrect(d, [21, 8, 26, 27], blue, INK)                # right lock
    d.line([7, 10, 7, 25], fill=light, width=1)
    d.line([23, 10, 23, 25], fill=light, width=1)
    d.line([12, 4, 19, 4], fill=light, width=1)
    d.line([5, 27, 10, 27], fill=shade(blue, 0.75), width=1)
    d.line([21, 27, 26, 27], fill=shade(blue, 0.75), width=1)


def draw_clown_nose(d, rng):
    red = (216, 44, 40, 255)
    # elastic strap running off both sides
    d.line([2, 13, 10, 14], fill=(210, 205, 195, 255), width=1)
    d.line([21, 14, 29, 13], fill=(210, 205, 195, 255), width=1)
    d.ellipse([9, 9, 22, 22], fill=red, outline=INK)
    d.ellipse([11, 11, 15, 15], fill=shade(red, 1.35))  # highlight
    d.point((12, 12), fill=(255, 235, 230, 255))
    d.arc([10, 10, 21, 21], start=60, end=160,
          fill=shade(red, 0.7), width=1)


def draw_clown_wig(d, rng):
    pink = (240, 130, 190, 255)
    puffs = [(9, 8), (15, 6), (21, 8), (7, 13), (24, 13), (11, 11),
             (20, 11), (15, 10)]
    accents = [(250, 205, 60, 255), (80, 160, 235, 255),
               (240, 110, 50, 255), (120, 205, 90, 255)]
    for i, (x, y) in enumerate(puffs):
        col = accents[i % len(accents)] if i % 3 == 2 else pink
        d.ellipse([x - 4, y - 4, x + 4, y + 4], fill=col, outline=INK)
        d.point((x - 1, y - 1), fill=shade(col, 1.25))
    # bald forehead gap between the side puffs
    d.rectangle([12, 13, 19, 17], fill=(0, 0, 0, 0))


def draw_dog_tag(d, rng):
    # ball chain arc (dotted) across the top
    for t in range(13):
        x = 5 + t * 2
        y = 4 + abs(t - 6) * 1
        d.point((x, y), fill=STEEL)
        d.point((x, y + 1), fill=STEEL_DARK)
    # two overlapping oval steel tags
    for (x0, y0) in ((7, 14), (15, 18)):
        d.ellipse([x0, y0, x0 + 11, y0 + 8], fill=STEEL, outline=INK)
        d.point((x0 + 5, y0 + 1), fill=HOUSING)         # chain hole
        for ly in (y0 + 3, y0 + 5):                     # stamped lines
            d.line([x0 + 2, ly, x0 + 8, ly], fill=STEEL_DARK, width=1)


def tefillin_hand(d, mirror):
    black = (26, 24, 26, 255)
    strap = (52, 40, 32, 255)
    # the box (bayit)
    rrect(d, [11, 5, 20, 13], black, INK)
    d.line([11, 5, 20, 5], fill=(86, 80, 78, 255), width=1)
    d.line([15, 5, 15, 13], fill=(60, 56, 54, 255), width=1)
    rrect(d, [10, 13, 21, 15], shade(black, 1.5))       # base plate
    # strap coiled below the box; coil leans to the wearing side
    lean = -1 if mirror else 1
    for i, y in enumerate(range(16, 26, 2)):
        x0 = 9 + (i * lean if i < 3 else 2 * lean)
        rrect(d, [x0, y, x0 + 13, y + 1], strap)
    # hanging strap end with the hand loop
    ex = 22 if mirror else 9
    d.line([ex, 18, ex + (2 if mirror else -2), 27], fill=strap, width=2)
    d.ellipse([ex - 2 if mirror else ex - 1, 25,
               ex + 3 if mirror else ex + 2, 29], outline=strap, width=1)


def draw_tefillin_hand_l(d, rng):
    tefillin_hand(d, mirror=False)


def draw_tefillin_hand_r(d, rng):
    tefillin_hand(d, mirror=True)


def draw_tefillin_head(d, rng):
    black = (26, 24, 26, 255)
    strap = (52, 40, 32, 255)
    # straps hang down both sides from the box
    for x in (8, 22):
        d.line([x, 10, x - 1, 26], fill=strap, width=2)
        d.line([x + 1, 10, x + 2, 26], fill=strap, width=1)
        rrect(d, [x - 2, 26, x + 3, 28], strap)         # strap ends
    # head box centred on top, knot behind suggested by side bumps
    rrect(d, [11, 4, 20, 12], black, INK)
    d.line([11, 4, 20, 4], fill=(86, 80, 78, 255), width=1)
    d.line([13, 4, 13, 12], fill=(60, 56, 54, 255), width=1)
    d.line([18, 4, 18, 12], fill=(60, 56, 54, 255), width=1)
    rrect(d, [10, 12, 21, 14], shade(black, 1.5))


def tie_blade(d, col, knot_col, skinny=False, checker=False):
    cx = 15
    # knot
    if skinny:
        rrect(d, [13, 4, 18, 8], knot_col, INK)
        top_w, bot_w, y1 = 1, 2, 27
    else:
        d.polygon([(12, 4), (19, 4), (18, 9), (13, 9)], fill=knot_col,
                  outline=INK)
        top_w, bot_w, y1 = 2, 4, 29
    # blade drawn row by row so the checker variant can pattern it
    for y in range(9 if not skinny else 8, y1):
        t = (y - 8) / (y1 - 8)
        w = top_w + (bot_w - top_w) * min(1.0, t * 1.6)
        if y > y1 - 4:                                  # taper to point
            w = max(0, bot_w * (y1 - y) / 4)
        x0, x1 = int(cx - w), int(cx + w)
        for x in range(x0, x1 + 1):
            if checker and ((x // 2 + y // 2) % 2 == 0):
                d.point((x, y), fill=(28, 28, 30, 255))
            else:
                d.point((x, y), fill=col)
    if not checker:
        d.line([cx - 1, 12, cx - 1, 24], fill=shade(col, 0.8), width=1)


def draw_tie_necktie(d, rng):
    green = (52, 120, 62, 255)
    tie_blade(d, green, shade(green, 0.7))


def draw_tie_skinny(d, rng):
    tie_blade(d, (232, 232, 232, 255), (40, 40, 44, 255),
              skinny=True, checker=True)


# --------------------------------------------------------- vehicle controls

def gauge(d, cx, cy, r, needle_ang=0.6):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(232, 234, 230, 255),
              outline=INK)
    import math
    nx = cx + int((r - 2) * math.cos(needle_ang))
    ny = cy + int((r - 2) * math.sin(needle_ang))
    d.line([cx, cy, nx, ny], fill=(200, 50, 40, 255), width=1)
    d.point((cx, cy), fill=INK)


def draw_camera_control(d, rng):
    rrect(d, [4, 9, 27, 25], HOUSING, INK)
    # 2x2 monitor bank with tiny camera views
    for i, (x0, y0) in enumerate(((6, 11), (16, 11), (6, 18), (16, 18))):
        rrect(d, [x0, y0, x0 + 8, y0 + 5], SCREEN, INK)
        if i == 0:      # doorway view
            d.rectangle([x0 + 3, y0 + 1, x0 + 5, y0 + 4],
                        outline=shade(SCREEN, 0.55))
        elif i == 1:    # corridor lines
            d.line([x0 + 1, y0 + 4, x0 + 3, y0 + 1],
                   fill=shade(SCREEN, 0.55), width=1)
            d.line([x0 + 7, y0 + 4, x0 + 5, y0 + 1],
                   fill=shade(SCREEN, 0.55), width=1)
        elif i == 2:    # figure dot
            d.point((x0 + 4, y0 + 2), fill=(200, 50, 40, 255))
            d.point((x0 + 4, y0 + 3), fill=shade(SCREEN, 0.5))
        else:           # static bars
            d.line([x0 + 1, y0 + 2, x0 + 7, y0 + 2],
                   fill=shade(SCREEN, 0.6), width=1)
    d.point((25, 23), fill=(80, 220, 90, 255))          # power LED


def draw_drive_by_wire(d, rng):
    rrect(d, [6, 12, 25, 24], (44, 44, 50, 255), INK)
    d.line([6, 12, 25, 12], fill=(200, 60, 50, 255), width=2)   # red trim
    rrect(d, [8, 15, 15, 21], SCREEN, INK)              # screen
    d.line([9, 19, 14, 16], fill=shade(SCREEN, 0.55), width=1)
    # joystick with red ball top
    d.line([21, 21, 22, 16], fill=STEEL, width=2)
    d.ellipse([20, 13, 24, 17], fill=(208, 52, 44, 255), outline=INK)
    # antenna + signal arcs
    d.line([8, 12, 6, 5], fill=STEEL_DARK, width=1)
    for r in (3, 5):
        d.arc([6 - r, 5 - r, 6 + r, 5 + r], start=270, end=350,
              fill=(200, 60, 50, 255), width=1)


def draw_electronics_controls(d, rng):
    rrect(d, [4, 9, 27, 23], (104, 108, 116, 255), INK)
    gauge(d, 10, 14, 4, 0.7)
    gauge(d, 17, 14, 4, 2.4)
    gauge(d, 24, 14, 3, 1.2)
    for i, x in enumerate(range(7, 26, 4)):             # toggle row
        d.line([x, 21, x, 19], fill=INK, width=1)
        d.point((x, 18), fill=CHROME)
    d.line([5, 10, 26, 10], fill=shade((104, 108, 116, 255), 1.3),
           width=1)


def draw_hand_controls(d, rng):
    # pair of adjustable steel rods, clamps low-left, trigger grips high
    for off in (0, 4):
        d.line([6 + off, 25, 20 + off, 9], fill=STEEL, width=2)
        d.line([6 + off, 25, 20 + off, 9], fill=STEEL_DARK, width=1)
        for t in (0.35, 0.55, 0.75):                    # adjustment holes
            x = int(6 + off + (14) * t)
            y = int(25 - 16 * t)
            d.point((x, y), fill=HOUSING)
    rrect(d, [4, 23, 9, 27], HOUSING, INK)              # pedal clamps
    # trigger + grip on the lead rod
    d.line([22, 8, 26, 6], fill=INK, width=2)
    d.line([23, 10, 26, 12], fill=(200, 50, 40, 255), width=1)
    rrect(d, [20, 5, 24, 8], HOUSING, INK)


def pedal_pair(d, block_fn):
    for x0 in (5, 17):
        block_fn(d, x0)


def draw_pedal_extenders(d, rng):
    def block(d, x0):
        rrect(d, [x0, 15, x0 + 9, 24], (52, 52, 56, 255), INK)
        for y in (17, 20, 23):                          # ribbed rubber top
            d.line([x0 + 1, y, x0 + 8, y], fill=(86, 86, 92, 255),
                   width=1)
        d.line([x0, 19, x0 + 9, 19], fill=STEEL_DARK, width=2)  # clamp
        d.point((x0 + 4, 19), fill=CHROME)
    pedal_pair(d, block)


def draw_pedal_extenders_makeshift(d, rng):
    def block(d, x0):
        rrect(d, [x0, 15, x0 + 9, 24], WOOD, INK)
        d.line([x0 + 1, 18, x0 + 8, 18], fill=WOOD_DARK, width=1)
        for y in (16, 22):                              # duct-tape bands
            d.line([x0, y, x0 + 9, y], fill=(178, 180, 184, 255),
                   width=2)
        d.line([x0 + 1, 16, x0 + 8, 23], fill=(150, 152, 156, 255),
               width=1)                                 # tape X
    pedal_pair(d, block)


def draw_robot_controls(d, rng):
    rrect(d, [7, 11, 24, 24], (226, 228, 230, 255), INK)
    # servo horns on top
    for x in (11, 19):
        d.ellipse([x - 2, 6, x + 2, 10], fill=STEEL_DARK, outline=INK)
        d.line([x - 3, 7, x + 3, 7], fill=STEEL, width=1)
    # exposed circuit board + wiring
    rrect(d, [9, 14, 16, 20], (46, 128, 66, 255), INK)
    for p in ((11, 16), (14, 16), (11, 18), (14, 18)):
        d.point(p, fill=(230, 210, 90, 255))
    d.line([16, 16, 21, 15], fill=(200, 60, 50, 255), width=1)
    d.line([16, 18, 21, 19], fill=(60, 110, 210, 255), width=1)
    d.line([16, 20, 21, 22], fill=(230, 180, 60, 255), width=1)
    d.point((22, 13), fill=(80, 220, 90, 255))
    d.line([24, 11, 27, 5], fill=STEEL_DARK, width=1)   # antenna
    d.point((27, 4), fill=(200, 50, 40, 255))


def draw_turret_controls(d, rng):
    rrect(d, [6, 11, 25, 24], (214, 216, 220, 255), INK)
    # camera eye
    d.ellipse([8, 14, 16, 22], fill=HOUSING, outline=INK)
    d.ellipse([10, 16, 14, 20], fill=(40, 80, 140, 255))
    d.point((11, 17), fill=(190, 225, 255, 255))
    # targeting screen with reticle
    rrect(d, [18, 13, 24, 20], (30, 34, 30, 255), INK)
    d.line([21, 14, 21, 19], fill=(220, 60, 50, 255), width=1)
    d.line([19, 16, 23, 16], fill=(220, 60, 50, 255), width=1)
    # motor drums below
    for x in (9, 19):
        rrect(d, [x, 22, x + 5, 24], STEEL_DARK, INK)


def draw_vehicle_dashboard(d, rng):
    rrect(d, [3, 10, 28, 23], (66, 68, 76, 255), INK)
    d.line([3, 10, 28, 10], fill=(96, 100, 110, 255), width=2)
    gauge(d, 10, 16, 5, 0.5)                            # speedometer
    gauge(d, 19, 15, 3, 2.2)                            # fuel
    for i, x in enumerate(range(23, 28)):               # cyan tell-tales
        d.point((x, 14 + (i % 2) * 3), fill=(80, 220, 235, 255))
    for x in (6, 14, 24):                               # switches
        d.line([x, 22, x, 20], fill=INK, width=1)
        d.point((x, 19), fill=CHROME)
    d.line([4, 21, 12, 21], fill=(80, 220, 235, 255), width=1)


# ------------------------------------------------------------ vehicle lights

def headlight(d, x0, x1, y0=10, y1=22, cage=0):
    rrect(d, [x0, y0, x1, y1], HOUSING, INK)
    rrect(d, [x0 + 2, y0 + 2, x1 - 2, y1 - 2], LENS, INK)
    cx = (x0 + x1) // 2
    d.ellipse([cx - 3, (y0 + y1) // 2 - 3, cx + 3, (y0 + y1) // 2 + 3],
              outline=LENS_DIM, width=1)
    for x in range(x0 + 4, x1 - 3, 3):                  # reflector ribs
        d.line([x, y0 + 3, x, y1 - 3], fill=LENS_DIM, width=1)
    if cage:
        for i in range(cage):
            x = x0 + 3 + i * ((x1 - x0 - 6) // max(1, cage - 1) or 1)
            d.line([x, y0, x, y1], fill=STEEL_DARK, width=2)
        d.line([x0, (y0 + y1) // 2, x1, (y0 + y1) // 2],
               fill=STEEL_DARK, width=2)
        for px in (x0 + 1, x1 - 1):
            for py in (y0 + 1, y1 - 1):
                d.point((px, py), fill=CHROME)


def draw_car_headlight(d, rng):
    headlight(d, 6, 25)


def draw_car_wide_headlight(d, rng):
    headlight(d, 3, 28, y0=11, y1=20)
    d.line([10, 11, 10, 20], fill=HOUSING, width=2)     # segment splits
    d.line([21, 11, 21, 20], fill=HOUSING, width=2)


def draw_headlight_reinforced(d, rng):
    headlight(d, 6, 25, cage=3)


def draw_wide_headlight_reinforced(d, rng):
    headlight(d, 3, 28, y0=11, y1=20, cage=4)


def draw_motorcycle_headlight(d, rng):
    # chrome bucket, round lens, side mounting ears
    rrect(d, [4, 14, 6, 17], STEEL_DARK)
    rrect(d, [25, 14, 27, 17], STEEL_DARK)
    d.ellipse([6, 6, 25, 25], fill=CHROME, outline=INK)
    d.ellipse([8, 8, 23, 23], fill=STEEL, outline=INK)
    d.ellipse([10, 10, 21, 21], fill=LENS, outline=INK)
    d.ellipse([12, 12, 19, 19], outline=LENS_DIM, width=1)
    d.point((13, 13), fill=(255, 255, 255, 255))
    d.point((14, 13), fill=(255, 255, 255, 255))
    rrect(d, [14, 25, 17, 28], STEEL_DARK)              # bracket


def flood_mount(d):
    d.line([10, 21, 10, 27], fill=STEEL_DARK, width=2)  # yoke arms
    d.line([21, 21, 21, 27], fill=STEEL_DARK, width=2)
    rrect(d, [8, 27, 23, 29], HOUSING, INK)             # base


def draw_floodlight(d, rng):
    flood_mount(d)
    d.ellipse([7, 5, 24, 22], fill=HOUSING, outline=INK)    # round can
    d.ellipse([9, 7, 22, 20], fill=LENS, outline=INK)
    d.ellipse([12, 10, 19, 17], outline=LENS_DIM, width=1)
    for x in (9, 12, 15, 18, 21):                       # cooling fins
        d.point((x, 5), fill=STEEL_DARK)


def draw_directed_floodlight(d, rng):
    flood_mount(d)
    rrect(d, [8, 6, 23, 21], HOUSING, INK)              # rectangular can
    rrect(d, [10, 8, 21, 19], LENS, INK)
    d.line([12, 9, 12, 18], fill=LENS_DIM, width=1)
    d.line([19, 9, 19, 18], fill=LENS_DIM, width=1)
    # barn-door flaps angled at both sides
    d.polygon([(8, 7), (4, 5), (4, 20), (8, 19)], fill=STEEL_DARK,
              outline=INK)
    d.polygon([(23, 7), (27, 5), (27, 20), (23, 19)],
              fill=STEEL_DARK, outline=INK)
    d.polygon([(10, 6), (12, 3), (19, 3), (21, 6)], fill=STEEL_DARK,
              outline=INK)


def emergency_bar(d, col):
    rrect(d, [5, 15, 26, 19], HOUSING, INK)             # base
    rrect(d, [5, 16, 8, 19], STEEL_DARK)                # feet/mounts
    rrect(d, [23, 16, 26, 19], STEEL_DARK)
    d.rectangle([6, 9, 25, 15], fill=col)               # dome
    d.ellipse([6, 7, 25, 11], fill=col)
    d.rectangle([6, 9, 25, 15], outline=INK)
    d.line([15, 8, 15, 15], fill=shade(col, 0.65), width=2)  # segments
    d.line([7, 10, 20, 10], fill=shade(col, 1.45), width=2)  # highlight
    d.line([6, 15, 25, 15], fill=shade(col, 0.7), width=1)


def draw_light_emergency_blue(d, rng):
    emergency_bar(d, (50, 100, 235, 255))


def draw_light_emergency_red(d, rng):
    emergency_bar(d, (225, 45, 40, 255))


# ------------------------------------------------------------- basecamp fake
# Prop facades: a plywood backing board peeks out behind the flat,
# muted silhouette; flames are painted shapes, never glow.

def backing(d, x0, y0, x1, y1):
    rrect(d, [x0 + 2, y0 + 1, x1 + 2, y1 + 1], BACKING, INK)


def painted_flame(d, cx, ybase, s=1):
    d.polygon([(cx - 2 * s, ybase), (cx, ybase - 5 * s),
               (cx + 2 * s, ybase)], fill=(228, 110, 36, 255))
    d.polygon([(cx - 1 * s, ybase), (cx, ybase - 3 * s),
               (cx + 1 * s, ybase)], fill=(244, 190, 60, 255))


def draw_fake_char_kiln(d, rng):
    backing(d, 8, 6, 23, 27)
    flat = (122, 126, 132, 255)
    d.ellipse([8, 6, 23, 16], fill=flat, outline=INK)   # domed drum top
    rrect(d, [8, 11, 23, 27], flat, INK)
    d.rectangle([8, 11, 23, 27], fill=flat)
    d.line([8, 16, 23, 16], fill=shade(flat, 0.8), width=1)  # drum seam
    rrect(d, [13, 19, 18, 26], shade(flat, 0.55), INK)  # door
    rrect(d, [14, 8, 17, 11], shade(flat, 0.8), INK)    # chimney stub
    for x in (10, 20):                                  # painted rust
        d.line([x, 22, x, 26], fill=(150, 96, 60, 255), width=1)


def draw_fake_char_smoker(d, rng):
    backing(d, 10, 4, 21, 28)
    flat = (116, 104, 94, 255)
    rrect(d, [17, 4, 20, 9], shade(flat, 0.8), INK)     # chimney
    d.ellipse([10, 6, 21, 12], fill=flat, outline=INK)  # lid dome
    rrect(d, [10, 9, 21, 26], flat, INK)
    d.rectangle([10, 9, 21, 26], fill=flat)
    d.line([10, 13, 21, 13], fill=shade(flat, 0.78), width=1)
    rrect(d, [12, 16, 19, 24], shade(flat, 0.6), INK)   # door
    d.point((18, 20), fill=INK)                         # latch
    d.line([11, 26, 11, 28], fill=INK, width=2)         # legs
    d.line([20, 26, 20, 28], fill=INK, width=2)
    d.ellipse([13, 14, 16, 16], outline=shade(flat, 0.6))  # dial


def draw_fake_clay_kiln(d, rng):
    backing(d, 7, 7, 24, 27)
    terra = (178, 116, 82, 255)
    d.ellipse([7, 7, 24, 20], fill=terra, outline=INK)  # beehive dome
    rrect(d, [7, 14, 24, 27], terra, INK)
    d.rectangle([7, 14, 24, 27], fill=terra)
    d.arc([8, 9, 23, 20], start=180, end=360,
          fill=shade(terra, 0.8), width=1)              # clay banding
    # arched mouth with painted flames
    d.polygon([(12, 27), (12, 19), (15, 16), (18, 19), (18, 27)],
              fill=(52, 40, 34, 255), outline=INK)
    painted_flame(d, 15, 26)
    rrect(d, [14, 8, 17, 11], shade(terra, 0.8), INK)   # top vent


def draw_fake_digester_tank(d, rng):
    backing(d, 5, 11, 26, 24)
    flat = (110, 128, 112, 255)
    rrect(d, [5, 12, 26, 23], flat, INK)                # horizontal tank
    d.ellipse([5, 12, 10, 23], fill=shade(flat, 0.85), outline=INK)
    d.ellipse([21, 12, 26, 23], fill=shade(flat, 1.1), outline=INK)
    d.line([12, 12, 12, 23], fill=shade(flat, 0.8), width=1)
    d.line([18, 12, 18, 23], fill=shade(flat, 0.8), width=1)
    rrect(d, [9, 8, 12, 12], shade(flat, 0.75), INK)    # inlet pipe
    d.ellipse([19, 6, 24, 11], outline=INK, width=1)    # valve wheel
    d.line([21, 6, 21, 11], fill=INK, width=1)
    d.line([19, 8, 24, 8], fill=INK, width=1)
    rrect(d, [20, 11, 22, 12], shade(flat, 0.75))
    d.line([8, 23, 8, 27], fill=INK, width=2)           # legs
    d.line([23, 23, 23, 27], fill=INK, width=2)


def draw_fake_drop_hammer(d, rng):
    backing(d, 7, 4, 24, 28)
    frame = (110, 114, 120, 255)
    rrect(d, [7, 4, 24, 7], frame, INK)                 # top beam
    rrect(d, [8, 7, 11, 26], frame, INK)                # uprights
    rrect(d, [20, 7, 23, 26], frame, INK)
    d.ellipse([13, 4, 18, 8], fill=STEEL_DARK, outline=INK)  # pulley
    d.line([14, 7, 14, 11], fill=INK, width=1)          # belt
    d.line([17, 7, 17, 11], fill=INK, width=1)
    rrect(d, [12, 11, 19, 17], shade(frame, 0.62), INK)  # hammer block
    rrect(d, [9, 23, 22, 28], WOOD_DARK, INK)           # base + anvil
    rrect(d, [13, 20, 18, 23], STEEL_DARK, INK)
    d.line([11, 17, 11, 23], fill=shade(frame, 0.8), width=1)  # guides
    d.line([20, 17, 20, 23], fill=shade(frame, 0.8), width=1)


def draw_fake_fireplace(d, rng):
    backing(d, 6, 5, 25, 28)
    brick = (158, 96, 78, 255)
    rrect(d, [6, 5, 25, 28], brick, INK)
    for y in range(8, 28, 3):                           # flat brick rows
        d.line([6, y, 25, y], fill=shade(brick, 0.78), width=1)
        off = 4 if (y // 3) % 2 else 0
        for x in range(8 + off, 25, 7):
            d.line([x, y, x, min(y + 3, 28)], fill=shade(brick, 0.78),
                   width=1)
    rrect(d, [5, 5, 26, 8], WOOD_DARK, INK)             # mantel shelf
    # firebox with painted flames
    d.polygon([(11, 28), (11, 18), (15, 15), (20, 18), (20, 28)],
              fill=(44, 36, 32, 255), outline=INK)
    painted_flame(d, 14, 27)
    painted_flame(d, 18, 27, s=1)
    d.line([12, 26, 19, 26], fill=(30, 26, 24, 255), width=2)  # log


def draw_fake_forge(d, rng):
    backing(d, 7, 3, 24, 28)
    brick = (140, 102, 84, 255)
    # chimney hood tapering up
    d.polygon([(11, 12), (20, 12), (17, 3), (14, 3)], fill=brick,
              outline=INK)
    rrect(d, [7, 16, 24, 28], brick, INK)               # forge body
    for y in (20, 24):
        d.line([7, y, 24, y], fill=shade(brick, 0.78), width=1)
    rrect(d, [7, 12, 24, 16], shade(brick, 0.85), INK)  # hood lip
    # coal bed with painted embers
    rrect(d, [10, 18, 21, 23], (40, 34, 30, 255), INK)
    for x in range(11, 21, 2):
        d.point((x, 20), fill=(228, 110, 36, 255))
        d.point((x + 1, 21), fill=(196, 70, 40, 255))
    painted_flame(d, 15, 20)
    # bellows wedge at the side
    d.polygon([(24, 19), (28, 21), (24, 24)], fill=WOOD_DARK, outline=INK)
    d.line([26, 21, 24, 21], fill=INK, width=1)


def draw_fake_woodstove(d, rng):
    backing(d, 9, 11, 22, 26)
    flat = (74, 74, 80, 255)
    rrect(d, [14, 3, 17, 12], shade(flat, 0.8), INK)    # stovepipe
    rrect(d, [8, 11, 23, 12], shade(flat, 1.15), INK)   # top plate
    rrect(d, [9, 12, 22, 26], flat, INK)                # body
    rrect(d, [12, 15, 19, 23], shade(flat, 0.6), INK)   # door
    d.ellipse([13, 16, 18, 21], fill=(36, 32, 30, 255),
              outline=INK)                              # round window
    painted_flame(d, 15, 21)
    d.point((20, 19), fill=CHROME)                      # handle
    d.line([10, 26, 10, 29], fill=INK, width=2)         # legs
    d.line([21, 26, 21, 29], fill=INK, width=2)
    d.line([9, 24, 22, 24], fill=shade(flat, 0.75), width=1)  # ash lip


def draw_parkour_practice(d, rng):
    # vault box (left) + precision rail (right) + step block
    rrect(d, [4, 17, 17, 27], PLY, INK)                 # vault box
    d.line([4, 21, 17, 21], fill=WOOD_DARK, width=1)    # panel seams
    d.line([10, 17, 10, 27], fill=WOOD_DARK, width=1)
    d.line([4, 17, 17, 17], fill=WOOD_LIGHT, width=2)   # padded top edge
    # rail: bar on two posts
    d.line([20, 10, 29, 10], fill=STEEL, width=2)
    d.line([21, 10, 21, 27], fill=STEEL_DARK, width=2)
    d.line([28, 10, 28, 27], fill=STEEL_DARK, width=2)
    d.line([19, 27, 23, 27], fill=STEEL_DARK, width=2)  # feet
    d.line([26, 27, 30, 27], fill=STEEL_DARK, width=2)
    rrect(d, [18, 22, 24, 27], WOOD, INK)               # step block
    d.line([18, 24, 24, 24], fill=WOOD_DARK, width=1)


DRAW = {
    "armchair": draw_armchair,
    "barstool": draw_barstool,
    "beanbag": draw_beanbag,
    "chair_plywood": draw_chair_plywood,
    "chair_wood": draw_chair_wood,
    "chair_wood_black": draw_chair_wood_black,
    "chair_wood_white": draw_chair_wood_white,
    "stool_log": draw_stool_log,
    "stool_wood": draw_stool_wood,
    "bostonchan_wig": draw_bostonchan_wig,
    "clown_nose": draw_clown_nose,
    "clown_wig": draw_clown_wig,
    "dog_tag": draw_dog_tag,
    "tefillin_hand_l": draw_tefillin_hand_l,
    "tefillin_hand_r": draw_tefillin_hand_r,
    "tefillin_head": draw_tefillin_head,
    "tie_necktie": draw_tie_necktie,
    "tie_skinny": draw_tie_skinny,
    "camera_control": draw_camera_control,
    "drive_by_wire_controls": draw_drive_by_wire,
    "electronics_controls": draw_electronics_controls,
    "hand_controls": draw_hand_controls,
    "pedal_extenders": draw_pedal_extenders,
    "pedal_extenders_makeshift": draw_pedal_extenders_makeshift,
    "robot_controls": draw_robot_controls,
    "turret_controls": draw_turret_controls,
    "vehicle_dashboard": draw_vehicle_dashboard,
    "car_headlight": draw_car_headlight,
    "car_wide_headlight": draw_car_wide_headlight,
    "directed_floodlight": draw_directed_floodlight,
    "floodlight": draw_floodlight,
    "headlight_reinforced": draw_headlight_reinforced,
    "light_emergency_blue": draw_light_emergency_blue,
    "light_emergency_red": draw_light_emergency_red,
    "motorcycle_headlight": draw_motorcycle_headlight,
    "wide_headlight_reinforced": draw_wide_headlight_reinforced,
    "fake_char_kiln": draw_fake_char_kiln,
    "fake_char_smoker": draw_fake_char_smoker,
    "fake_clay_kiln": draw_fake_clay_kiln,
    "fake_digester_tank": draw_fake_digester_tank,
    "fake_drop_hammer": draw_fake_drop_hammer,
    "fake_fireplace": draw_fake_fireplace,
    "fake_forge": draw_fake_forge,
    "fake_woodstove": draw_fake_woodstove,
    "parkour_practice": draw_parkour_practice,
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
        prev.alpha_composite(big, ((i % cols) * cell + 1,
                                   (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item74.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item74.png {prev.size}")


if __name__ == "__main__":
    main()
