#!/usr/bin/env python3
"""ITEM52: workshop tools and equipment -- CDDA Voxel coverage audit.

All 28 items come from workshop.json (plus robofac_gun_kit).  Each
tool is drawn as itself, in its own material:

  steel / iron tools in grays (bench vise, claw bar, demolition
    hammer, acetylene machine, wire draw machine);
  bronze tools in bronze-brown metal (bronze fileset, bronze nail
    header, bronze nail puller -- the steel nail header is the
    same shape in steel gray, for contrast);
  wooden tools in browns (block and tackle with its two pulleys,
    carding paddles with toothy faces, distaff and spindle);
  powered tools with their cords (yellow-black power drill,
    demolition hammer), the Exodii kiln as a fridge-sized dark
    box with a glowing ceramic door and copper trim;
  vacuum molds twice: the proper aluminum/white vacuum molder
    and the makeshift homemade version in wood and patchwork;
  welding blankets twice: the neat asbestos mat and the
    makeshift one stitched together from oven insulation batts
    (patchwork panels, dashed stitching);
  small hand tools as their descriptions say: the monkey tool
    is a cylindrical steel rivet former with a rod through its
    hole, the pin reamer a tapered fluted bit with a grip, the
    thread cutting set a round die in a grip plus a tap.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "acetylene_machine",
    "adobe_pallet_full_active",
    "bench_vise",
    "block_and_tackle_wood",
    "bronze_file",
    "carding_paddles",
    "claw_bar",
    "corded_demolition_hammer",
    "corded_powerdrill",
    "distaff_spindle",
    "exodii_kiln",
    "fanning_mill",
    "hammer_peen",
    "makeshift_vacuum_mold",
    "makeshift_welding_blanket",
    "monkey_tool",
    "nail_header",
    "nail_header_bronze",
    "nail_puller_bronze",
    "pin_reamer",
    "propane_torch",
    "robofac_gun_kit",
    "soldering_iron_portable",
    "thread_cutting_set",
    "vac_mold",
    "welding_blanket",
    "welding_kit",
    "wire_draw_machine",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

STEEL = (150, 154, 162, 255)
STEEL_DARK = (92, 96, 104, 255)
STEEL_LIGHT = (198, 202, 210, 255)
IRON = (110, 112, 118, 255)
IRON_DARK = (64, 66, 72, 255)
BRONZE = (176, 124, 60, 255)
BRONZE_DARK = (122, 82, 38, 255)
BRONZE_LIGHT = (216, 168, 96, 255)
WOOD = (150, 108, 62, 255)
WOOD_DARK = (104, 70, 38, 255)
WOOD_LIGHT = (182, 140, 90, 255)
BLACK = (36, 36, 40, 255)
PLASTIC = (52, 52, 58, 255)
YELLOW = (232, 190, 52, 255)
RED = (186, 44, 38, 255)
RED_DARK = (130, 28, 26, 255)
WHITE = (236, 236, 234, 255)
GRAY_MAT = (178, 178, 174, 255)
MUD = (128, 94, 62, 255)
MUD_DARK = (96, 68, 42, 255)
MUD_LIGHT = (158, 120, 82, 255)
COPPER = (184, 110, 66, 255)
TEAL = (52, 160, 150, 255)
ORANGE = (238, 128, 40, 255)
BLUE_FLAME = (90, 150, 240, 255)
ROPE = (196, 172, 120, 255)
FIBER = (226, 218, 198, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def grain(d, rng, box, base, n=14):
    """Speckle texture inside a box, from the seeded rng."""
    x0, y0, x1, y1 = box
    for _ in range(n):
        x = rng.randint(x0, x1)
        y = rng.randint(y0, y1)
        d.point((x, y), fill=shade(base, rng.uniform(0.82, 1.14)))


def cord(d, x, y, droop=5, col=BLACK):
    """A power cord trailing down from (x, y) in a loose curl."""
    pts = []
    for t in range(0, 11):
        pts.append((x + t * 0.9, y + droop * math.sin(t * 0.55)
                    + t * 0.55))
    d.line(pts, fill=col, width=1)
    pts2 = [(px, py + 1) for px, py in pts]
    d.line(pts2, fill=col, width=1)


# ------------------------------------------------------------------ tools

def draw_acetylene(d, rng):
    # bulky steel machine: body, carbide hopper lid, gauge, hose coil
    d.rectangle([5, 12, 24, 28], fill=STEEL)
    d.rectangle([5, 12, 24, 28], outline=STEEL_DARK, width=1)
    d.rectangle([5, 12, 24, 16], fill=STEEL_LIGHT)
    d.polygon([(8, 12), (10, 7), (19, 7), (21, 12)], fill=STEEL_DARK)
    d.ellipse([11, 2 + 6, 17, 14], fill=WHITE, outline=IRON_DARK)
    d.line([14, 11, 16, 9], fill=RED, width=1)  # gauge needle
    d.point((14, 11), fill=BLACK)
    for gx in (8, 12, 16, 20):  # front panel vents
        d.line([gx, 20, gx, 25], fill=STEEL_DARK, width=1)
    d.rectangle([3, 26, 26, 29], fill=IRON_DARK)  # base skid
    # hose coiled on the right side, torch tip at its end
    d.ellipse([22, 15, 30, 24], outline=BLACK, width=2)
    d.ellipse([24, 17, 28, 22], outline=BLACK, width=1)
    d.line([28, 23, 30, 27], fill=COPPER, width=2)
    grain(d, rng, (6, 17, 23, 27), STEEL)


def draw_adobe_pallet(d, rng):
    # stacked wet mud bricks on a wooden pallet
    rows = [(4, 6, 27, 11), (4, 12, 27, 17), (4, 18, 27, 23)]
    for (x0, y0, x1, y1) in rows:
        off = 0 if y0 != 12 else 4
        for bx in range(x0 + off, x1 - 3, 8):
            d.rectangle([bx, y0, min(bx + 6, x1), y1], fill=MUD)
            d.line([bx, y0, min(bx + 6, x1), y0], fill=MUD_LIGHT, width=1)
            d.line([bx, y1, min(bx + 6, x1), y1], fill=MUD_DARK, width=1)
        grain(d, rng, (x0, y0, x1, y1), MUD, n=10)
    d.point((9, 8), fill=MUD_LIGHT)  # wet sheen spots
    d.point((20, 14), fill=MUD_LIGHT)
    # pallet: slats + blocks + bottom board
    d.rectangle([3, 24, 28, 26], fill=WOOD)
    d.line([3, 24, 28, 24], fill=WOOD_LIGHT, width=1)
    for bx in (4, 14, 24):
        d.rectangle([bx, 27, bx + 3, 28], fill=WOOD_DARK)
    d.rectangle([3, 29, 28, 30], fill=WOOD_DARK)


def draw_bench_vise(d, rng):
    # heavy iron vise: base, fixed + sliding jaws, screw handle
    d.rectangle([6, 25, 26, 28], fill=IRON_DARK)
    d.rectangle([6, 25, 26, 25], fill=IRON)
    d.rectangle([13, 18, 19, 25], fill=IRON)  # body
    d.rectangle([7, 8, 11, 20], fill=IRON)  # fixed jaw
    d.rectangle([7, 8, 11, 11], fill=STEEL)  # jaw face
    for gy in (9, 10):
        d.line([8, gy, 10, gy], fill=STEEL_DARK, width=1)
    d.rectangle([21, 8, 25, 20], fill=IRON)  # sliding jaw
    d.rectangle([21, 8, 25, 11], fill=STEEL)
    for gy in (9, 10):
        d.line([22, gy, 24, gy], fill=STEEL_DARK, width=1)
    d.rectangle([11, 12, 21, 15], fill=STEEL_LIGHT)  # flat anvil surface
    d.rectangle([19, 16, 23, 19], fill=IRON_DARK)  # screw housing
    d.line([16, 21, 28, 21], fill=STEEL, width=2)  # handle bar
    d.ellipse([14, 19, 17, 23], fill=WOOD_DARK)  # handle knobs
    d.ellipse([26, 19, 29, 23], fill=WOOD_DARK)
    grain(d, rng, (7, 16, 25, 24), IRON)


def draw_block_and_tackle(d, rng):
    # wooden block housing, two pulley wheels, rope and hook
    d.line([16, 1, 16, 4], fill=STEEL_DARK, width=2)  # top hook shank
    d.arc([13, 0, 19, 6], start=270, end=180, fill=STEEL_DARK, width=2)
    d.rectangle([9, 5, 23, 24], fill=WOOD)
    d.rectangle([9, 5, 23, 24], outline=WOOD_DARK, width=1)
    d.line([11, 5, 11, 24], fill=WOOD_DARK, width=1)  # side cheek seam
    d.line([21, 5, 21, 24], fill=WOOD_DARK, width=1)
    for cy in (11, 19):  # the pair of pulleys
        d.ellipse([13, cy - 4, 19, cy + 4], fill=WOOD_LIGHT,
                  outline=WOOD_DARK, width=1)
        d.ellipse([15, cy - 2, 17, cy + 2], fill=IRON_DARK)
        d.point((16, cy), fill=STEEL_LIGHT)
    d.line([13, 7, 13, 22], fill=ROPE, width=1)  # rope over the wheels
    d.line([19, 7, 19, 15], fill=ROPE, width=1)
    d.line([19, 15, 24, 28], fill=ROPE, width=1)  # hauling rope down
    d.line([13, 22, 10, 28], fill=ROPE, width=1)
    grain(d, rng, (12, 6, 20, 23), WOOD, n=8)


def file_blade(d, x, y, ln, col, rng, tilt=0):
    """One flat file: tapered blade with crosshatch teeth + tang."""
    d.polygon([(x, y), (x + ln, y + tilt), (x + ln - 2, y + 3 + tilt),
               (x, y + 3)], fill=col)
    for tx in range(x + 2, x + ln - 3, 2):  # teeth
        d.line([tx, y + 1, tx + 1, y + 2], fill=shade(col, 0.7), width=1)
    d.line([x, y, x + ln, y + tilt], fill=shade(col, 1.25), width=1)


def draw_bronze_file(d, rng):
    # a fanned set of three bronze files with wooden handles
    for i, (y, ln) in enumerate(((8, 19), (15, 21), (22, 18))):
        hx = 4 + i
        d.rectangle([hx, y + 1, hx + 5, y + 4], fill=WOOD_DARK)
        d.rectangle([hx, y + 1, hx + 5, y + 4], outline=shade(WOOD_DARK, 0.7))
        file_blade(d, hx + 6, y, ln, BRONZE, rng, tilt=(i - 1))
        d.point((hx + 6 + ln - 1, y + 1 + (i - 1)), fill=BRONZE_LIGHT)


def draw_carding_paddles(d, rng):
    # pair of toothy wooden paddles, handles down
    for x0 in (5, 17):
        d.rectangle([x0, 4, x0 + 9, 17], fill=WOOD)
        d.rectangle([x0, 4, x0 + 9, 17], outline=WOOD_DARK, width=1)
        for ty in range(6, 16, 2):  # carding teeth: dotted rows
            for tx in range(x0 + 2, x0 + 8, 2):
                d.point((tx, ty), fill=STEEL_DARK)
        d.rectangle([x0 + 3, 18, x0 + 6, 27], fill=WOOD_DARK)  # handle
        d.line([x0 + 4, 19, x0 + 4, 26], fill=WOOD, width=1)
        grain(d, rng, (x0 + 1, 5, x0 + 8, 16), WOOD, n=6)
    # loose cleaned fibers caught between them
    for fx, fy in ((15, 9), (16, 13), (15, 20)):
        d.point((fx, fy), fill=FIBER)


def draw_claw_bar(d, rng):
    # steel pry bar, diagonal: forked claw bend at top, chisel at base
    d.line([10, 26, 22, 10], fill=STEEL, width=3)
    d.line([10, 26, 22, 10], fill=STEEL_LIGHT, width=1)
    d.polygon([(8, 28), (12, 28), (11, 24), (9, 24)], fill=STEEL_DARK)
    # claw: bent fork with a V notch at the top end
    d.line([22, 10, 26, 6], fill=STEEL, width=3)
    d.polygon([(25, 3), (28, 5), (26, 8), (24, 6)], fill=STEEL)
    d.polygon([(28, 5), (30, 8), (27, 9), (26, 8)], fill=STEEL)
    d.point((27, 6), fill=(0, 0, 0, 0))
    d.line([26, 6, 27, 7], fill=STEEL_DARK, width=1)


def draw_demolition_hammer(d, rng):
    # handheld jackhammer: T handle, motor body, long chisel, cord
    d.line([8, 5, 24, 5], fill=BLACK, width=3)  # top handle bar
    d.rectangle([10, 3, 13, 7], fill=PLASTIC)
    d.rectangle([19, 3, 22, 7], fill=PLASTIC)
    d.rectangle([12, 6, 20, 16], fill=STEEL)  # motor body
    d.rectangle([12, 6, 20, 9], fill=STEEL_DARK)
    for vx in (14, 17):  # cooling vents
        d.line([vx, 11, vx, 14], fill=STEEL_DARK, width=1)
    d.rectangle([14, 16, 18, 20], fill=IRON_DARK)  # chuck
    d.polygon([(15, 20), (17, 20), (16, 29)], fill=STEEL_LIGHT)  # bit
    d.point((16, 29), fill=STEEL_DARK)
    cord(d, 21, 7, droop=4)
    grain(d, rng, (13, 10, 19, 15), STEEL, n=8)


def draw_powerdrill(d, rng):
    # black/yellow corded drill in profile, bit to the right, cord below
    d.rectangle([6, 9, 20, 16], fill=PLASTIC)  # body
    d.rectangle([6, 9, 20, 11], fill=YELLOW)  # yellow top band
    d.rectangle([6, 12, 9, 15], fill=YELLOW)
    d.rectangle([20, 11, 23, 14], fill=STEEL_DARK)  # chuck
    d.line([23, 12, 29, 12], fill=STEEL_LIGHT, width=1)  # bit
    d.point((29, 12), fill=STEEL_DARK)
    d.polygon([(9, 16), (14, 16), (15, 25), (10, 25)], fill=PLASTIC)
    d.rectangle([10, 18, 12, 20], fill=RED)  # trigger
    cord(d, 12, 25, droop=3)
    # a couple of spare bits laid below
    d.line([18, 24, 24, 24], fill=STEEL, width=1)
    d.line([19, 27, 23, 27], fill=STEEL, width=1)
    grain(d, rng, (7, 12, 19, 15), PLASTIC, n=6)


def draw_distaff_spindle(d, rng):
    # distaff: tall rod with a fiber bundle; spindle: whorl + wound yarn
    d.line([10, 4, 10, 28], fill=WOOD, width=2)
    for fx in range(6, 15):  # fluff of unspun fiber at the top
        for fy in range(2, 9):
            if (fx - 10) ** 2 + (fy - 5) ** 2 <= 10:
                d.point((fx, fy), fill=FIBER)
    d.point((8, 4), fill=shade(FIBER, 0.85))
    d.point((12, 7), fill=shade(FIBER, 0.85))
    d.line([10, 9, 21, 22], fill=shade(FIBER, 0.9), width=1)  # drafted thread
    d.line([21, 6, 21, 27], fill=WOOD_DARK, width=2)  # spindle rod
    d.ellipse([17, 13, 25, 17], fill=WOOD)  # whorl disc
    d.ellipse([17, 13, 25, 17], outline=WOOD_DARK, width=1)
    for wy in (19, 21, 23):  # yarn wound below the whorl
        d.line([20, wy, 22, wy], fill=(206, 120, 110, 255), width=1)
    d.polygon([(20, 27), (22, 27), (21, 29)], fill=WOOD_DARK)


def draw_exodii_kiln(d, rng):
    # fridge-sized dark kiln: ceramic door, round window, copper trim
    d.rectangle([7, 3, 25, 29], fill=IRON)
    d.rectangle([7, 3, 25, 29], outline=IRON_DARK, width=1)
    d.rectangle([9, 6, 23, 24], fill=(168, 158, 146, 255))  # ceramic door
    d.rectangle([9, 6, 23, 24], outline=COPPER, width=1)
    d.ellipse([12, 10, 20, 18], fill=IRON_DARK)  # viewing window
    d.ellipse([13, 11, 19, 17], fill=ORANGE)  # hot glow inside
    d.ellipse([14, 12, 18, 16], fill=(250, 196, 92, 255))
    d.line([21, 8, 21, 22], fill=COPPER, width=1)  # door hinges/trim
    for hx in (10, 13, 16, 19, 22):
        d.point((hx, 4), fill=COPPER)
    d.rectangle([9, 26, 14, 28], fill=BLACK)  # control panel
    d.point((10, 27), fill=TEAL)
    d.point((12, 27), fill=RED)
    d.line([18, 27, 23, 27], fill=TEAL, width=1)  # display line
    grain(d, rng, (8, 4, 24, 28), IRON, n=10)


def draw_fanning_mill(d, rng):
    # wooden winnowing machine: hopper, sloped sieve body, crank, legs
    d.polygon([(9, 3), (23, 3), (20, 9), (12, 9)], fill=WOOD_DARK)
    d.line([10, 5, 22, 5], fill=MUD_LIGHT, width=1)  # grain in the hopper
    d.rectangle([8, 9, 24, 20], fill=WOOD)
    d.rectangle([8, 9, 24, 20], outline=WOOD_DARK, width=1)
    for sy in (12, 15, 18):  # sieve slats
        d.line([10, sy, 22, sy], fill=WOOD_DARK, width=1)
    d.ellipse([24, 12, 29, 17], outline=IRON_DARK, width=1)  # crank wheel
    d.line([26, 14, 26, 10], fill=IRON_DARK, width=1)
    d.point((26, 10), fill=WOOD_DARK)
    d.line([10, 20, 8, 28], fill=WOOD_DARK, width=2)  # legs
    d.line([22, 20, 24, 28], fill=WOOD_DARK, width=2)
    d.line([7, 26, 25, 26], fill=WOOD_DARK, width=1)
    for cx, cy in ((12, 23), (16, 24), (20, 23)):  # chaff drifting out
        d.point((cx, cy), fill=ROPE)
    grain(d, rng, (9, 10, 23, 19), WOOD, n=8)


def draw_hammer_peen(d, rng):
    # ball-peen hammer on the diagonal: flat face one end, ball the other
    d.line([11, 27, 20, 12], fill=WOOD, width=3)
    d.line([11, 27, 20, 12], fill=WOOD_LIGHT, width=1)
    d.rectangle([15, 5, 24, 11], fill=STEEL)  # head block
    d.rectangle([15, 5, 24, 11], outline=STEEL_DARK, width=1)
    d.rectangle([12, 6, 15, 10], fill=STEEL_DARK)  # flat striking face
    d.ellipse([23, 5, 28, 11], fill=STEEL)  # rounded peen
    d.ellipse([23, 5, 28, 11], outline=STEEL_DARK, width=1)
    d.point((25, 7), fill=STEEL_LIGHT)
    d.rectangle([18, 11, 21, 13], fill=WOOD_DARK)  # eye/wedge


def blanket_base(d, rng, makeshift):
    """A folded welding blanket; makeshift = patchwork insulation batts."""
    d.rectangle([5, 9, 27, 24], fill=GRAY_MAT)
    d.rectangle([5, 9, 27, 24], outline=shade(GRAY_MAT, 0.6), width=1)
    d.rectangle([5, 21, 27, 24], fill=shade(GRAY_MAT, 0.88))  # fold band
    if makeshift:
        panels = [((5, 9, 15, 15), (186, 180, 164, 255)),
                  ((16, 9, 27, 15), (168, 166, 158, 255)),
                  ((5, 16, 15, 20), (172, 170, 160, 255)),
                  ((16, 16, 27, 20), (190, 184, 168, 255))]
        for (box, col) in panels:
            d.rectangle(list(box), fill=col)
        for sx in range(7, 26, 3):  # dashed stitching
            d.point((sx, 15), fill=RED_DARK)
            d.point((sx, 20), fill=RED_DARK)
        for sy in range(10, 20, 3):
            d.point((15, sy), fill=RED_DARK)
        d.line([5, 24, 8, 26], fill=shade(GRAY_MAT, 0.8), width=1)  # fray
        d.line([24, 24, 26, 26], fill=shade(GRAY_MAT, 0.8), width=1)
    else:
        for sx in range(7, 26, 4):  # neat stitched border
            d.point((sx, 11), fill=shade(GRAY_MAT, 0.62))
            d.point((sx, 22), fill=shade(GRAY_MAT, 0.62))
        d.ellipse([23, 10, 26, 13], outline=STEEL_DARK, width=1)  # grommet
        d.line([7, 17, 25, 17], fill=shade(GRAY_MAT, 0.92), width=1)
    grain(d, rng, (6, 10, 26, 23), GRAY_MAT, n=10)


def vac_former(d, rng, makeshift):
    """Vacuum-forming machine; makeshift = wood frame, clamps, rough."""
    frame = WOOD if makeshift else STEEL_LIGHT
    body = WOOD_DARK if makeshift else (120, 124, 130, 255)
    d.rectangle([5, 16, 27, 27], fill=body)  # base box
    d.rectangle([5, 16, 27, 27], outline=shade(body, 0.6), width=1)
    d.rectangle([6, 10, 26, 16], fill=frame)  # forming frame
    d.rectangle([8, 11, 24, 15], fill=WHITE)  # plastic sheet
    d.polygon([(12, 12), (20, 12), (18, 15), (14, 15)],
              fill=shade(WHITE, 0.88))  # sheet drawn down over a mold bump
    d.rectangle([22, 19, 26, 24], fill=BLACK if makeshift else IRON_DARK)
    d.line([24, 19, 24, 12], fill=BLACK, width=1)  # pump hose
    if makeshift:  # clamps + patch screws on the frame
        for cx in (7, 15, 25):
            d.rectangle([cx, 9, cx + 2, 12], fill=STEEL_DARK)
        d.point((8, 25), fill=STEEL_LIGHT)
        d.point((12, 22), fill=STEEL_LIGHT)
    else:  # gauge + switch on the proper machine
        d.ellipse([8, 19, 13, 24], fill=WHITE, outline=IRON_DARK)
        d.line([10, 22, 12, 20], fill=RED, width=1)
        d.rectangle([15, 20, 18, 23], fill=TEAL)
    grain(d, rng, (6, 17, 21, 26), body, n=8)


def draw_monkey_tool(d, rng):
    # cylindrical steel rivet former: chunk with a hole, rod, mushroom head
    d.rectangle([11, 12, 21, 24], fill=STEEL)  # the steel chunk
    d.ellipse([11, 9, 21, 14], fill=STEEL_LIGHT)  # top face of cylinder
    d.ellipse([11, 9, 21, 14], outline=STEEL_DARK, width=1)
    d.ellipse([14, 10, 18, 13], fill=IRON_DARK)  # the central hole
    d.rectangle([15, 3, 17, 11], fill=STEEL_DARK)  # rod through the hole
    d.ellipse([13, 1, 19, 5], fill=STEEL_LIGHT)  # mushroom rivet head
    d.ellipse([13, 1, 19, 5], outline=STEEL_DARK, width=1)
    d.line([11, 18, 21, 18], fill=STEEL_DARK, width=1)  # knurl band
    for kx in range(12, 21, 2):
        d.point((kx, 20), fill=STEEL_DARK)
    d.rectangle([11, 23, 21, 24], fill=STEEL_DARK)
    grain(d, rng, (12, 15, 20, 22), STEEL, n=6)


def nail_header_shape(d, col):
    """Flat-ended handle with a square hole; a nail fed through it."""
    d.line([6, 22, 20, 12], fill=col, width=3)  # handle
    d.line([6, 22, 20, 12], fill=shade(col, 1.25), width=1)
    d.polygon([(19, 8), (27, 10), (26, 15), (18, 13)], fill=col)  # flat end
    d.polygon([(19, 8), (27, 10), (26, 15), (18, 13)],
              outline=shade(col, 0.65))
    d.rectangle([21, 10, 23, 12], fill=BLACK)  # the square hole
    d.line([22, 10, 22, 4], fill=STEEL_LIGHT, width=1)  # nail shank up
    d.rectangle([20, 2, 24, 4], fill=STEEL)  # nail head being formed
    d.line([22, 12, 22, 16], fill=STEEL_DARK, width=1)  # shank below


def draw_pin_reamer(d, rng):
    # grip on top, tapered fluted bit below
    d.rectangle([12, 3, 20, 10], fill=WOOD_DARK)  # handle
    for kx in range(13, 20, 2):
        d.line([kx, 4, kx, 9], fill=shade(WOOD_DARK, 0.7), width=1)
    d.rectangle([14, 10, 18, 13], fill=STEEL_DARK)  # shank collar
    d.polygon([(14, 13), (18, 13), (16, 28)], fill=STEEL)  # tapered bit
    for t in range(4):  # spiral flutes
        y0 = 15 + t * 3
        d.line([14 + (t % 2), y0, 18 - (t % 2), y0 + 2],
               fill=STEEL_DARK, width=1)
    d.line([15, 14, 16, 27], fill=STEEL_LIGHT, width=1)
    d.point((16, 28), fill=STEEL_DARK)


def draw_propane_torch(d, rng):
    # red propane cylinder, brass valve, angled torch head, blue flame
    d.rectangle([11, 13, 21, 28], fill=RED)  # cylinder
    d.ellipse([11, 11, 21, 15], fill=RED_DARK)
    d.rectangle([11, 20, 21, 23], fill=WHITE)  # label band
    d.line([13, 21, 19, 21], fill=RED, width=1)
    d.line([13, 22, 17, 22], fill=STEEL_DARK, width=1)
    d.rectangle([14, 7, 18, 11], fill=(202, 168, 82, 255))  # brass valve
    d.line([17, 8, 24, 4], fill=STEEL_DARK, width=2)  # torch neck
    d.polygon([(23, 2), (27, 3), (25, 6), (22, 5)], fill=IRON_DARK)
    d.polygon([(27, 1), (30, 3), (27, 4)], fill=BLUE_FLAME)  # flame
    d.point((28, 3), fill=(200, 225, 255, 255))
    d.rectangle([9, 28, 23, 29], fill=IRON_DARK)  # tote base edge
    grain(d, rng, (12, 14, 20, 27), RED, n=6)


def draw_robofac_kit(d, rng):
    # heavy-duty Hub 01 bag, open: barrels and tools inside, logo patch
    for i, bx in enumerate((10, 15, 20)):  # barrel tubes poking out
        d.rectangle([bx, 3 + (i % 2), bx + 2, 11], fill=STEEL_DARK)
        d.point((bx + 1, 3 + (i % 2)), fill=BLACK)
    d.line([22, 5, 26, 10], fill=STEEL, width=2)  # a tool handle
    d.rectangle([5, 10, 27, 28], fill=(84, 88, 96, 255))  # nylon bag body
    d.rectangle([5, 10, 27, 28], outline=BLACK, width=1)
    d.rectangle([5, 10, 27, 14], fill=(64, 68, 76, 255))  # top flap band
    for zx in range(7, 26, 3):  # zipper teeth
        d.point((zx, 12), fill=STEEL_LIGHT)
    d.ellipse([12, 17, 20, 25], outline=WHITE, width=1)  # Hub 01 logo ring
    d.line([14, 21, 18, 21], fill=WHITE, width=1)
    d.line([16, 19, 16, 23], fill=WHITE, width=1)
    d.rectangle([22, 18, 26, 22], fill=BLACK)  # side pocket
    d.line([6, 28, 26, 28], fill=BLACK, width=1)
    grain(d, rng, (6, 15, 26, 27), (84, 88, 96, 255), n=10)


def draw_soldering_iron(d, rng):
    # portable iron on the diagonal: battery-slot handle, hot tip
    d.line([8, 24, 15, 15], fill=PLASTIC, width=4)  # handle
    d.rectangle([6, 25, 10, 28], fill=BLACK)  # battery pack at the butt
    d.point((8, 26), fill=TEAL)  # battery pip
    d.rectangle([11, 18, 14, 21], fill=STEEL_DARK)  # slot line / collar
    d.line([15, 15, 23, 7], fill=STEEL, width=2)  # shaft
    d.polygon([(23, 7), (27, 3), (25, 8)], fill=STEEL_LIGHT)  # tip
    d.point((27, 3), fill=ORANGE)  # hot point
    d.point((26, 4), fill=ORANGE)
    d.point((24, 6), fill=(250, 196, 92, 255))  # heat shimmer hint (opaque)
    d.line([9, 23, 13, 19], fill=shade(PLASTIC, 1.5), width=1)


def draw_thread_cutting_set(d, rng):
    # round die held in a two-handled grip + a tap bit beside it
    d.line([3, 16, 12, 16], fill=STEEL_DARK, width=2)  # grip bars
    d.line([20, 16, 29, 16], fill=STEEL_DARK, width=2)
    d.ellipse([4, 14, 6, 18], fill=WOOD_DARK)
    d.ellipse([26, 14, 28, 18], fill=WOOD_DARK)
    d.ellipse([11, 9, 21, 23], fill=STEEL)  # the round die
    d.ellipse([11, 9, 21, 23], outline=STEEL_DARK, width=1)
    d.ellipse([14, 12, 18, 20], fill=IRON_DARK)  # die hole
    for tx in (15, 17):  # thread teeth in the hole
        d.line([tx, 13, tx, 19], fill=STEEL_LIGHT, width=1)
    for sx, sy in ((12, 11), (20, 11), (12, 21), (20, 21)):  # die screws
        d.point((sx, sy), fill=STEEL_DARK)
    # the tap: small threaded rod below
    d.rectangle([10, 26, 22, 28], fill=STEEL)
    for tx in range(11, 22, 2):
        d.line([tx, 26, tx, 28], fill=STEEL_DARK, width=1)
    d.rectangle([21, 25, 24, 29], fill=STEEL_DARK)  # tap square drive


def draw_welding_kit(d, rng):
    # red welder unit with dial + cables, fireproof blanket draped in front
    d.rectangle([6, 5, 22, 20], fill=RED)  # welder body
    d.rectangle([6, 5, 22, 20], outline=RED_DARK, width=1)
    d.rectangle([6, 5, 22, 8], fill=RED_DARK)  # top handle band
    d.ellipse([9, 10, 14, 15], fill=WHITE, outline=IRON_DARK)  # dial
    d.line([11, 13, 13, 11], fill=RED, width=1)
    d.point((18, 11), fill=BLACK)  # switch / socket
    d.point((18, 14), fill=(250, 196, 92, 255))
    d.line([8, 17, 20, 17], fill=RED_DARK, width=1)  # vent slots
    d.line([8, 19, 20, 19], fill=RED_DARK, width=1)
    cord(d, 20, 18, droop=3)  # electrode cable
    d.line([24, 22, 27, 25], fill=STEEL_DARK, width=2)  # electrode holder
    # folded blanket draped across the bottom
    d.rectangle([4, 21, 24, 29], fill=GRAY_MAT)
    d.rectangle([4, 21, 24, 29], outline=shade(GRAY_MAT, 0.6), width=1)
    d.line([4, 25, 24, 25], fill=shade(GRAY_MAT, 0.8), width=1)
    for sx in range(6, 23, 4):
        d.point((sx, 23), fill=shade(GRAY_MAT, 0.62))
    grain(d, rng, (7, 9, 21, 19), RED, n=6)


def draw_wire_draw_machine(d, rng):
    # steel frame machine: rod in, draw plate, crank, coil of wire out
    d.rectangle([4, 12, 20, 24], fill=STEEL)  # machine body
    d.rectangle([4, 12, 20, 24], outline=STEEL_DARK, width=1)
    d.rectangle([4, 12, 20, 15], fill=STEEL_LIGHT)
    d.rectangle([8, 17, 13, 22], fill=IRON_DARK)  # draw plate
    for hx, hy in ((9, 18), (11, 18), (10, 20), (12, 21)):  # die holes
        d.point((hx, hy), fill=STEEL_LIGHT)
    d.line([1, 18, 8, 19], fill=STEEL_LIGHT, width=2)  # rod stock in
    d.ellipse([15, 17, 19, 21], outline=STEEL_DARK, width=1)  # drive wheel
    d.line([17, 19, 21, 14], fill=IRON_DARK, width=1)  # crank arm
    d.point((21, 14), fill=WOOD_DARK)
    d.rectangle([3, 24, 21, 26], fill=IRON_DARK)  # base
    # finished wire: neat coil at the right
    for r in (5, 3):
        d.ellipse([22, 14, 22 + 2 * r, 14 + 2 * r], outline=STEEL_LIGHT,
                  width=1)
    d.line([20, 19, 24, 19], fill=STEEL_LIGHT, width=1)  # wire feed to coil
    grain(d, rng, (5, 16, 19, 23), STEEL, n=8)


DRAW = {
    "acetylene_machine": draw_acetylene,
    "adobe_pallet_full_active": draw_adobe_pallet,
    "bench_vise": draw_bench_vise,
    "block_and_tackle_wood": draw_block_and_tackle,
    "bronze_file": draw_bronze_file,
    "carding_paddles": draw_carding_paddles,
    "claw_bar": draw_claw_bar,
    "corded_demolition_hammer": draw_demolition_hammer,
    "corded_powerdrill": draw_powerdrill,
    "distaff_spindle": draw_distaff_spindle,
    "exodii_kiln": draw_exodii_kiln,
    "fanning_mill": draw_fanning_mill,
    "hammer_peen": draw_hammer_peen,
    "makeshift_vacuum_mold": lambda d, r: vac_former(d, r, True),
    "makeshift_welding_blanket": lambda d, r: blanket_base(d, r, True),
    "monkey_tool": draw_monkey_tool,
    "nail_header": lambda d, r: nail_header_shape(d, STEEL),
    "nail_header_bronze": lambda d, r: nail_header_shape(d, BRONZE),
    "nail_puller_bronze": None,  # filled below (needs rng signature)
    "pin_reamer": draw_pin_reamer,
    "propane_torch": draw_propane_torch,
    "robofac_gun_kit": draw_robofac_kit,
    "soldering_iron_portable": draw_soldering_iron,
    "thread_cutting_set": draw_thread_cutting_set,
    "vac_mold": lambda d, r: vac_former(d, r, False),
    "welding_blanket": lambda d, r: blanket_base(d, r, False),
    "welding_kit": draw_welding_kit,
    "wire_draw_machine": draw_wire_draw_machine,
}


def draw_nail_puller_bronze(d, rng):
    # crude bronze puller: shaft with a forked claw, rough cast edges
    d.line([9, 25, 21, 11], fill=BRONZE, width=3)
    d.line([9, 25, 21, 11], fill=BRONZE_LIGHT, width=1)
    d.polygon([(7, 27), (11, 27), (10, 23), (8, 23)], fill=BRONZE_DARK)
    d.line([21, 11, 25, 6], fill=BRONZE, width=3)  # bent claw neck
    d.polygon([(24, 3), (28, 4), (26, 8), (23, 6)], fill=BRONZE)  # fork tine
    d.polygon([(28, 4), (30, 7), (27, 9), (26, 8)], fill=BRONZE)
    for px, py in ((13, 20), (17, 15), (20, 12)):  # casting pits (crude)
        d.point((px, py), fill=BRONZE_DARK)
    grain(d, rng, (10, 12, 24, 24), BRONZE, n=5)


DRAW["nail_puller_bronze"] = draw_nail_puller_bronze


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    fn = DRAW.get(sid)
    if fn is None:
        raise KeyError(sid)
    fn(d, rng)
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
                       "preview_item52.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item52.png {prev.size}")


if __name__ == "__main__":
    main()
