#!/usr/bin/env python3
"""ITEM76: fire starters, debug tools, vehicle frames, highland
incense, integrated bionics and labyrinth kipple -- CDDA Voxel
coverage audit.

Families, drawn from each item's in-game description:

  Fire starters (items/tool/fire.json) -- each drawn as itself:
    the crude electric firestarter is a rough steel box with two
    prongs and a spark; the electric lighter a slim blue body with
    an arc top; the pump fire drill a shaft, pump bar, cord and
    flywheel over a notched plank; the fire piston a wooden
    cylinder with its piston rod raised; the crude lens a glass
    disc focusing sunlight onto a glowing point; the refillable
    butane lighter a large trigger lighter with a butane can in
    its base; the lit refillable lighter a flip-top with the lid
    open and a flame; the lit ember carrier a steel tinderbox with
    punched holes and a smoldering ember glowing inside.
  Debug tools (items/tool/debug_tools.json) -- light-gray plastic
    devices, each carrying a black/white checker tag marking it as
    a debug tool and a screen glyph for its function: fault X,
    dimension swirl, cracked item (simple: one crack, plain;
    advanced: bigger, gold trim, more cracks), sun for the lux
    meter, explosion burst for the mortar, clock face for the
    timer, cloud and sun for the weather reporter.
  Vehicle frames (items/vehicle/frames.json) -- rectangular
    chassis frame outlines, material and bulk by name: thin
    hinged steel for the foldable light frame, brown planks for
    the foldable wooden frame, the thinnest pipework for the
    foldable extra-light frame, rails plus a tilted mixing drum
    for the concrete mixer chassis, a few rope-lashed boards for
    the light wooden frame, the thickest dark-green riveted rails
    for the heavy-duty frame, round crudely-welded tubes for the
    improvised tubular frame, slim aluminium pipework for the
    extra-light unibody frame.
  Highland incense (items/relics/highland_censer.json) -- the
    loose substance itself in a small heap, per its description:
    black fine powder, blue gel with pale dead grubs, golden
    mouldy dried flower remnants, glossy green resin pearls with
    leaves and chips, magenta crystal shards, twigs with dried
    red flowers, milk-white spiral shells, yellow resin stones.
  Integrated bionics (items/tool/integrated.json) -- superalloy
    and copper fingertip tools (soldering iron set, multimeter,
    welder, multitool) and magenta plastic internal housings
    (battery compartment with cells, ethanol / gasoline / oil
    tanks told apart by size and a contents glyph).
  Labyrinth kipple (items/netherum/labyrinth_kipple.json) -- a
    strange monolith-grey object, a crinkled foil-like wrapper
    with glyph text, and six odd devices of unknown purpose, each
    shaped in its own material: aluminium, copper, nanotubes,
    plastic, steel, unobtanium.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "crude_firestarter",
    "electric_lighter",
    "fire_drill_large",
    "fire_piston",
    "lens_crude",
    "ref_lighter_butane",
    "ref_lighter_on",
    "tinderbox_on",
    "debug_apply_fault",
    "debug_dimension_traveller",
    "debug_item_damager",
    "debug_item_damager_adv",
    "debug_lux_meter",
    "debug_mortar",
    "debug_timer",
    "debug_weather_check",
    "foldframe",
    "foldwoodframe",
    "foldxlframe",
    "frame_concrete_mix",
    "frame_wood_light",
    "hdframe",
    "improvised_frame",
    "xlframe",
    "highland_black_incense",
    "highland_blue_incense",
    "highland_golden_incense",
    "highland_green_incense",
    "highland_purple_incense",
    "highland_red_incense",
    "highland_white_incense",
    "highland_yellow_incense",
    "integrated_electrokit",
    "integrated_multimeter",
    "integrated_welder",
    "internal_battery_compartment",
    "internal_ethanol_tank",
    "internal_gasoline_tank",
    "internal_oil_tank",
    "toolset",
    "kipple_object",
    "kipple_wrapper",
    "nl_kipple_scrap_alu",
    "nl_kipple_scrap_copper",
    "nl_kipple_scrap_nanotubes",
    "nl_kipple_scrap_plastic",
    "nl_kipple_scrap_steel",
    "nl_kipple_scrap_unob",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 46, 255)
STEEL = (168, 172, 178, 255)
STEEL_DARK = (104, 108, 114, 255)
STEEL_LIGHT = (206, 210, 216, 255)
WOOD = (122, 86, 48, 255)
WOOD_DARK = (88, 60, 32, 255)
WOOD_LIGHT = (164, 124, 78, 255)
PLASTIC_GRAY = (196, 198, 202, 255)
SCREEN = (36, 52, 60, 255)
GLYPH = (140, 230, 200, 255)
CHECKER_D = (30, 30, 34, 255)
CHECKER_L = (238, 238, 240, 255)
GOLD = (214, 168, 60, 255)
COPPER = (184, 108, 62, 255)
SUPERALLOY = (188, 182, 196, 255)
MAGENTA = (196, 62, 148, 255)
MAGENTA_DARK = (150, 40, 112, 255)
FLAME = (246, 150, 40, 255)
FLAME_CORE = (252, 214, 110, 255)
EMBER = (232, 84, 30, 255)
SPARK = (150, 130, 250, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- fire tools

def draw_crude_firestarter(d, rng):
    # rough steel box, two prongs on top, spark arcing between them
    d.rectangle([9, 14, 22, 27], fill=STEEL_DARK)
    d.rectangle([9, 14, 22, 27], outline=INK, width=1)
    d.rectangle([11, 17, 15, 21], fill=STEEL)  # crude weld plate
    d.point((19, 24), fill=INK)
    d.point((20, 18), fill=INK)
    d.line([12, 14, 12, 8], fill=STEEL_LIGHT, width=2)
    d.line([19, 14, 19, 8], fill=STEEL_LIGHT, width=2)
    d.line([12, 8, 15, 6], fill=SPARK, width=1)  # spark zigzag
    d.line([15, 6, 17, 9], fill=SPARK, width=1)
    d.line([17, 9, 19, 8], fill=SPARK, width=1)
    d.point((15, 10), fill=(220, 210, 255, 255))


def draw_electric_lighter(d, rng):
    blue = (52, 92, 190, 255)
    d.rectangle([12, 10, 20, 28], fill=blue)
    d.rectangle([12, 10, 20, 28], outline=INK, width=1)
    d.rectangle([12, 6, 20, 10], fill=STEEL_DARK)  # zinc top
    d.line([14, 6, 14, 4], fill=STEEL_LIGHT, width=1)
    d.line([18, 6, 18, 4], fill=STEEL_LIGHT, width=1)
    d.line([14, 4, 16, 3], fill=SPARK, width=1)  # plasma arc
    d.line([16, 3, 18, 4], fill=SPARK, width=1)
    d.rectangle([14, 14, 18, 17], fill=shade(blue, 1.25))  # button
    d.rectangle([15, 25, 17, 27], outline=INK, width=1)  # charge port


def draw_fire_drill(d, rng):
    # notched plank base, vertical shaft, flywheel, pump bar + cords
    d.rectangle([5, 26, 27, 29], fill=WOOD)
    d.rectangle([5, 26, 27, 29], outline=WOOD_DARK, width=1)
    d.point((16, 27), fill=INK)  # notch ember spot
    d.point((15, 28), fill=EMBER)
    d.line([16, 26, 16, 5], fill=WOOD_LIGHT, width=2)  # shaft
    d.ellipse([10, 19, 22, 24], fill=WOOD_DARK)  # flywheel
    d.ellipse([10, 19, 22, 24], outline=INK, width=1)
    d.point((16, 21), fill=WOOD_LIGHT)
    d.rectangle([8, 8, 24, 10], fill=WOOD)  # pump bar
    d.rectangle([8, 8, 24, 10], outline=WOOD_DARK, width=1)
    d.line([9, 10, 16, 19], fill=(210, 196, 160, 255), width=1)  # cords
    d.line([23, 10, 16, 19], fill=(210, 196, 160, 255), width=1)
    d.rectangle([14, 3, 18, 5], fill=WOOD_DARK)  # top bearing


def draw_fire_piston(d, rng):
    # hollow wooden cylinder, piston rod raised out of the top
    d.rectangle([12, 13, 20, 29], fill=WOOD)
    d.rectangle([12, 13, 20, 29], outline=WOOD_DARK, width=1)
    d.line([14, 15, 14, 27], fill=WOOD_DARK, width=1)
    d.line([18, 15, 18, 27], fill=WOOD_LIGHT, width=1)
    d.rectangle([11, 11, 21, 13], fill=WOOD_DARK)  # open rim
    d.line([16, 11, 16, 5], fill=WOOD_LIGHT, width=2)  # piston rod
    d.rectangle([13, 3, 19, 5], fill=WOOD)  # piston handle cap
    d.line([13, 9, 19, 9], fill=(196, 60, 40, 255), width=1)  # seal ring


def draw_lens(d, rng):
    glass = (168, 226, 232, 255)
    d.ellipse([8, 4, 24, 20], fill=glass)
    d.ellipse([8, 4, 24, 20], outline=(90, 140, 148, 255), width=1)
    d.ellipse([11, 7, 17, 12], outline=(230, 250, 252, 255), width=1)
    sun = (250, 214, 90, 255)
    d.point((4, 3), fill=sun)
    d.point((28, 3), fill=sun)
    d.line([10, 19, 15, 25], fill=(250, 230, 150, 255), width=1)  # rays
    d.line([22, 19, 17, 25], fill=(250, 230, 150, 255), width=1)
    d.point((16, 26), fill=FLAME_CORE)  # focal point ember
    d.point((16, 25), fill=FLAME)
    d.point((15, 27), fill=EMBER)
    d.point((17, 27), fill=EMBER)


def draw_ref_lighter_butane(d, rng):
    # big trigger lighter: barrel nozzle, grip, butane can in base
    d.rectangle([6, 8, 22, 12], fill=STEEL_LIGHT)  # barrel
    d.rectangle([6, 8, 22, 12], outline=INK, width=1)
    d.rectangle([4, 9, 6, 11], fill=STEEL_DARK)  # nozzle tip
    d.rectangle([18, 12, 22, 24], fill=(238, 238, 240, 255))  # body
    d.rectangle([18, 12, 22, 24], outline=INK, width=1)
    d.polygon([(17, 12), (19, 12), (18, 16), (16, 16)], fill=INK)  # trigger
    d.point((20, 14), fill=(196, 60, 40, 255))  # safety dot
    d.rectangle([18, 24, 22, 29], fill=(90, 150, 210, 255))  # butane can
    d.line([18, 26, 22, 26], fill=INK, width=1)


def draw_ref_lighter_on(d, rng):
    # flip-top lighter, lid open, flame above the wick
    d.rectangle([12, 14, 21, 29], fill=STEEL)
    d.rectangle([12, 14, 21, 29], outline=INK, width=1)
    d.line([12, 18, 21, 18], fill=STEEL_DARK, width=1)
    d.polygon([(12, 14), (12, 7), (15, 7), (15, 14)], fill=STEEL_LIGHT)
    d.polygon([(12, 14), (12, 7), (15, 7), (15, 14)], outline=INK)
    d.rectangle([16, 11, 20, 14], fill=STEEL_DARK)  # chimney
    for hx in (17, 19):
        d.point((hx, 12), fill=INK)
    d.polygon([(16, 11), (18, 5), (20, 11)], fill=FLAME)  # flame
    d.polygon([(17, 11), (18, 8), (19, 11)], fill=FLAME_CORE)
    d.point((24, 20), fill=STEEL_DARK)  # flint wheel hint
    d.point((24, 21), fill=STEEL_DARK)


def draw_tinderbox_on(d, rng):
    # steel ember carrier: box with punched air holes, ember glow
    d.rectangle([8, 13, 24, 27], fill=STEEL)
    d.rectangle([8, 13, 24, 27], outline=INK, width=1)
    d.rectangle([8, 13, 24, 16], fill=STEEL_DARK)  # lid band
    d.line([16, 10, 16, 13], fill=STEEL_DARK, width=2)  # handle stub
    for hx in range(11, 23, 3):  # punched holes, ember glowing through
        for hy in (20, 24):
            d.point((hx, hy), fill=EMBER)
            d.point((hx + 1, hy), fill=FLAME_CORE)
    d.line([11, 9, 13, 6], fill=(170, 170, 176, 255), width=1)  # smoke
    d.line([13, 6, 12, 4], fill=(170, 170, 176, 255), width=1)
    d.line([20, 8, 21, 5], fill=(170, 170, 176, 255), width=1)


# ------------------------------------------------------------ debug tools

def checker_tag(d, x, y):
    """2x2 black/white checker tag: the debug-tool motif."""
    d.rectangle([x, y, x + 1, y + 1], fill=CHECKER_D)
    d.rectangle([x + 2, y, x + 3, y + 1], fill=CHECKER_L)
    d.rectangle([x, y + 2, x + 1, y + 3], fill=CHECKER_L)
    d.rectangle([x + 2, y + 2, x + 3, y + 3], fill=CHECKER_D)


def debug_device(d, rng, adv=False):
    """Shared light-gray plastic handheld with a dark screen."""
    body = PLASTIC_GRAY if not adv else shade(PLASTIC_GRAY, 0.94)
    d.rectangle([9, 6, 23, 27], fill=body)
    d.rectangle([9, 6, 23, 27], outline=INK, width=1)
    if adv:
        d.rectangle([9, 6, 23, 8], fill=GOLD)  # advanced trim
    d.rectangle([11, 10, 21, 20], fill=SCREEN)
    d.rectangle([11, 10, 21, 20], outline=INK, width=1)
    d.point((12, 24), fill=INK)  # buttons
    d.point((16, 24), fill=INK)
    d.point((20, 24), fill=(196, 60, 40, 255))
    checker_tag(d, 18, 2) if adv else checker_tag(d, 11, 2)
    return (11, 10, 21, 20)


def draw_debug_fault(d, rng):
    box = debug_device(d, rng)
    x0, y0, x1, y1 = box
    red = (235, 70, 60, 255)  # fault X on screen
    d.line([x0 + 2, y0 + 2, x1 - 2, y1 - 2], fill=red, width=1)
    d.line([x0 + 2, y1 - 2, x1 - 2, y0 + 2], fill=red, width=1)


def draw_debug_traveller(d, rng):
    box = debug_device(d, rng)
    cx, cy = 16, 15  # dimension swirl
    for r in (4, 2):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=GLYPH, width=1)
    d.point((cx, cy), fill=GLYPH)
    d.line([23, 8, 26, 4], fill=STEEL_DARK, width=1)  # antenna
    d.point((26, 4), fill=GLYPH)


def _damager(d, rng, adv):
    box = debug_device(d, rng, adv=adv)
    x0, y0, x1, y1 = box
    # cracked item glyph: a little box with crack lines
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], outline=GLYPH, width=1)
    d.line([x0 + 4, y0 + 2, x0 + 5, y0 + 5], fill=GLYPH, width=1)
    d.line([x0 + 5, y0 + 5, x0 + 4, y1 - 2], fill=GLYPH, width=1)
    if adv:
        d.line([x1 - 4, y0 + 2, x1 - 5, y0 + 4], fill=GLYPH, width=1)
        d.line([x1 - 5, y0 + 4, x1 - 4, y1 - 2], fill=GLYPH, width=1)
        d.line([x0 + 5, y0 + 6, x1 - 5, y0 + 6], fill=GLYPH, width=1)


def draw_debug_damager(d, rng):
    _damager(d, rng, adv=False)


def draw_debug_damager_adv(d, rng):
    _damager(d, rng, adv=True)


def draw_debug_lux(d, rng):
    box = debug_device(d, rng)
    d.ellipse([13, 1, 19, 5], fill=(240, 240, 244, 255))  # sensor dome
    d.ellipse([13, 1, 19, 5], outline=INK, width=1)
    cx, cy = 16, 15  # sun glyph
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=(250, 214, 90, 255))
    for dx, dy in ((-4, 0), (4, 0), (0, -4), (0, 4), (-3, -3), (3, -3),
                   (-3, 3), (3, 3)):
        d.point((cx + dx, cy + dy), fill=(250, 214, 90, 255))


def draw_debug_mortar(d, rng):
    box = debug_device(d, rng)
    cx, cy = 16, 15  # explosion burst
    for a in range(0, 360, 45):
        ex = cx + int(4 * math.cos(math.radians(a)))
        ey = cy + int(4 * math.sin(math.radians(a)))
        d.line([cx, cy, ex, ey], fill=(250, 170, 60, 255), width=1)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(250, 220, 130, 255))
    d.rectangle([14, 27, 18, 30], fill=STEEL_DARK)  # mortar base hint


def draw_debug_timer(d, rng):
    # stopwatch: round body, crown, clock glyph on the face
    d.ellipse([9, 9, 23, 27], fill=PLASTIC_GRAY)
    d.ellipse([9, 9, 23, 27], outline=INK, width=1)
    d.rectangle([14, 5, 18, 9], fill=STEEL_DARK)  # crown stem
    d.rectangle([13, 3, 19, 5], fill=STEEL_DARK)  # crown
    d.ellipse([11, 11, 21, 25], fill=SCREEN)
    cx, cy = 16, 18
    d.line([cx, cy, cx, cy - 4], fill=GLYPH, width=1)  # hands
    d.line([cx, cy, cx + 3, cy + 1], fill=GLYPH, width=1)
    d.point((cx, cy), fill=GLYPH)
    checker_tag(d, 24, 3)


def draw_debug_weather(d, rng):
    box = debug_device(d, rng)
    # cloud + sun glyph
    d.ellipse([12, 14, 18, 18], fill=(190, 205, 220, 255))
    d.ellipse([14, 12, 20, 17], fill=(190, 205, 220, 255))
    d.point((13, 20), fill=(90, 140, 220, 255))  # rain ticks
    d.point((16, 21), fill=(90, 140, 220, 255))
    d.point((19, 20), fill=(90, 140, 220, 255))
    d.point((20, 11), fill=(250, 214, 90, 255))
    d.line([23, 8, 27, 3], fill=STEEL_DARK, width=1)  # antenna
    d.point((27, 3), fill=(250, 214, 90, 255))


# ---------------------------------------------------------- vehicle frames

def frame_outline(d, x0, y0, x1, y1, rail, thick, col, cross=True):
    """Rectangular chassis frame: two rails + end bars (+crossmembers)."""
    for i in range(thick):
        d.rectangle([x0 + i, y0 + i, x1 - i, y1 - i], outline=col, width=1)
    if cross:
        for fx in (x0 + (x1 - x0) // 3, x0 + 2 * (x1 - x0) // 3):
            d.line([fx, y0 + thick, fx, y1 - thick], fill=col, width=1)


def draw_foldframe(d, rng):
    # light steel frame, hinge marks halfway along each rail
    frame_outline(d, 5, 9, 27, 23, 2, 2, STEEL)
    for hx in (15, 16):
        d.line([hx, 9, hx, 12], fill=INK, width=1)  # hinges
        d.line([hx, 20, hx, 23], fill=INK, width=1)
    d.point((7, 11), fill=STEEL_DARK)
    d.point((25, 21), fill=STEEL_DARK)


def draw_foldwoodframe(d, rng):
    frame_outline(d, 5, 10, 27, 22, 2, 2, WOOD)
    for hx in (15, 16):
        d.line([hx, 10, hx, 12], fill=WOOD_DARK, width=1)
        d.line([hx, 20, hx, 22], fill=WOOD_DARK, width=1)
    d.line([6, 14, 26, 14], fill=WOOD_LIGHT, width=1)  # grain
    d.line([6, 18, 26, 18], fill=WOOD_DARK, width=1)


def draw_foldxlframe(d, rng):
    # smallest, thinnest pipework, folding hinges
    frame_outline(d, 8, 11, 24, 21, 1, 1, STEEL_LIGHT)
    d.point((16, 11), fill=INK)
    d.point((16, 21), fill=INK)
    d.point((8, 16), fill=STEEL_DARK)
    d.point((24, 16), fill=STEEL_DARK)


def draw_frame_concrete_mix(d, rng):
    # chassis rails with a tilted mixing drum mounted on them
    frame_outline(d, 4, 20, 28, 27, 2, 2, STEEL_DARK, cross=False)
    d.line([10, 20, 10, 27], fill=STEEL_DARK, width=1)
    d.line([22, 20, 22, 27], fill=STEEL_DARK, width=1)
    d.polygon([(10, 20), (13, 6), (21, 6), (24, 20)], fill=(178, 182, 188, 255))
    d.polygon([(10, 20), (13, 6), (21, 6), (24, 20)], outline=INK)
    d.line([12, 14, 22, 14], fill=STEEL_DARK, width=1)  # drum band
    d.ellipse([14, 4, 20, 7], fill=STEEL_DARK)  # drum mouth
    d.line([24, 10, 27, 8], fill=INK, width=1)  # chute


def draw_frame_wood_light(d, rng):
    # a few boards lashed together with rope at the corners
    frame_outline(d, 7, 11, 25, 21, 2, 2, WOOD_LIGHT, cross=False)
    d.line([16, 11, 16, 21], fill=WOOD, width=1)
    rope = (216, 198, 158, 255)
    for cx, cy in ((7, 11), (25, 11), (7, 21), (25, 21)):
        d.line([cx - 1, cy - 1, cx + 1, cy + 1], fill=rope, width=1)
        d.line([cx - 1, cy + 1, cx + 1, cy - 1], fill=rope, width=1)


def draw_hdframe(d, rng):
    # heavy-duty: thickest dark-green military rails, rivets, largest
    green = (72, 96, 62, 255)
    frame_outline(d, 3, 7, 29, 25, 3, 3, green)
    for rx in range(5, 29, 4):
        d.point((rx, 8), fill=shade(green, 0.6))
        d.point((rx, 24), fill=shade(green, 0.6))
    d.line([16, 10, 16, 22], fill=shade(green, 0.8), width=2)


def draw_improvised_frame(d, rng):
    # tubular steel: round tube rails, crude weld blobs at the joints
    tube = (96, 168, 186, 255)
    frame_outline(d, 5, 9, 27, 23, 2, 2, tube, cross=False)
    d.line([16, 9, 16, 23], fill=tube, width=2)
    for cx, cy in ((5, 9), (27, 9), (5, 23), (27, 23), (16, 9), (16, 23)):
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=STEEL_LIGHT)
    d.point((11, 16), fill=STEEL_DARK)  # rust/weld spatter
    d.point((21, 13), fill=STEEL_DARK)


def draw_xlframe(d, rng):
    # extra-light unibody: slim aluminium pipework, integrated look
    frame_outline(d, 7, 10, 25, 22, 1, 1, (216, 220, 226, 255), cross=False)
    d.line([12, 10, 12, 22], fill=(216, 220, 226, 255), width=1)
    d.line([20, 10, 20, 22], fill=(216, 220, 226, 255), width=1)
    d.line([7, 16, 25, 16], fill=shade(STEEL, 0.9), width=1)
    for cx, cy in ((7, 10), (25, 10), (7, 22), (25, 22)):
        d.point((cx, cy), fill=STEEL_DARK)


# ------------------------------------------------------- highland incense
# Loose substances, per description, heaped in a small shallow dish.

def incense_dish(d):
    d.polygon([(7, 24), (25, 24), (22, 28), (10, 28)],
              fill=(120, 74, 44, 255))
    d.polygon([(7, 24), (25, 24), (22, 28), (10, 28)], outline=INK)
    d.line([8, 25, 24, 25], fill=(90, 54, 32, 255), width=1)


def draw_incense_black(d, rng):
    incense_dish(d)
    d.polygon([(10, 24), (13, 17), (19, 17), (22, 24)],
              fill=(34, 32, 36, 255))  # fine black powder heap
    d.point((15, 19), fill=(70, 66, 74, 255))
    d.point((17, 21), fill=(70, 66, 74, 255))
    d.point((12, 27), fill=(34, 32, 36, 255))  # spilled grains


def draw_incense_blue(d, rng):
    incense_dish(d)
    gel = (52, 92, 200, 255)
    d.ellipse([9, 17, 23, 25], fill=gel)  # foul blue gel blob
    d.ellipse([9, 17, 23, 25], outline=shade(gel, 0.7), width=1)
    grub = (226, 222, 196, 255)
    for gx, gy in ((12, 20), (17, 19), (15, 22)):  # dead grubs
        d.line([gx, gy, gx + 2, gy + 1], fill=grub, width=1)
        d.point((gx + 3, gy + 1), fill=grub)
        d.point((gx + 1, gy + 2), fill=grub)  # too many legs
    d.point((11, 18), fill=(140, 180, 240, 255))  # gel sheen


def draw_incense_golden(d, rng):
    incense_dish(d)
    gold = (216, 168, 66, 255)
    for bx, by in ((11, 21), (15, 19), (19, 21), (13, 23), (18, 23)):
        d.ellipse([bx, by, bx + 3, by + 2], fill=gold)  # dried flower bits
        d.point((bx + 1, by), fill=(240, 206, 110, 255))  # golden mould
    d.line([12, 18, 14, 16], fill=(150, 116, 44, 255), width=1)  # stem
    d.line([19, 18, 18, 16], fill=(150, 116, 44, 255), width=1)


def draw_incense_green(d, rng):
    incense_dish(d)
    resin = (62, 168, 84, 255)
    for bx, by in ((10, 21), (14, 19), (18, 20), (12, 23), (17, 23),
                   (20, 22)):
        d.ellipse([bx, by, bx + 2, by + 2], fill=resin)  # resin pearls
        d.point((bx, by), fill=(150, 230, 170, 255))  # gloss
    leaf = (44, 120, 56, 255)
    d.polygon([(9, 19), (12, 17), (12, 20)], fill=leaf)  # leaves
    d.polygon([(21, 18), (24, 19), (21, 21)], fill=leaf)
    d.line([15, 24, 17, 22], fill=WOOD_LIGHT, width=1)  # wood chip


def draw_incense_purple(d, rng):
    incense_dish(d)
    crys = (206, 72, 190, 255)
    shards = [((10, 24), (12, 18), (14, 24)),
              ((14, 24), (16, 16), (18, 24)),
              ((18, 24), (20, 19), (22, 24)),
              ((12, 24), (15, 20), (17, 24))]
    for s in shards:  # crystal shards
        d.polygon(list(s), fill=crys)
        d.polygon(list(s), outline=shade(crys, 0.65))
    d.point((16, 19), fill=(246, 180, 240, 255))  # facet glint
    d.point((12, 21), fill=(246, 180, 240, 255))


def draw_incense_red(d, rng):
    incense_dish(d)
    d.line([10, 24, 13, 17], fill=WOOD_DARK, width=2)  # small branches
    d.line([21, 24, 19, 16], fill=WOOD_DARK, width=2)
    d.line([15, 24, 16, 19], fill=WOOD, width=1)
    red = (186, 44, 40, 255)
    for fx, fy in ((12, 18), (19, 17), (16, 20), (14, 22), (20, 21)):
        d.ellipse([fx, fy, fx + 2, fy + 2], fill=red)  # dried red flowers
        d.point((fx + 1, fy + 1), fill=(120, 60, 30, 255))


def draw_incense_white(d, rng):
    incense_dish(d)
    shell = (240, 238, 230, 255)
    for cx, cy, r in ((12, 21, 3), (17, 19, 3), (20, 22, 2), (14, 23, 2)):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=shell)
        d.ellipse([cx - r, cy - r, cx + r, cy + r],
                  outline=(180, 176, 168, 255), width=1)
        d.arc([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1],
              start=20, end=300, fill=(180, 176, 168, 255), width=1)
    d.point((24, 26), fill=shell)  # stray shell


def draw_incense_yellow(d, rng):
    incense_dish(d)
    resin = (232, 186, 52, 255)
    for bx, by in ((10, 21), (14, 19), (18, 20), (12, 23), (17, 23),
                   (20, 22), (15, 21)):  # fragrant resin stones
        d.ellipse([bx, by, bx + 3, by + 2], fill=resin)
        d.ellipse([bx, by, bx + 3, by + 2], outline=(170, 130, 34, 255),
                  width=1)
        d.point((bx + 1, by), fill=(250, 224, 140, 255))


# ----------------------------------------------------- integrated bionics

def draw_electrokit(d, rng):
    # soldering iron pen + screwdriver, superalloy with copper tips
    d.line([8, 24, 19, 13], fill=SUPERALLOY, width=3)  # iron body
    d.line([19, 13, 24, 8], fill=COPPER, width=1)  # hot tip
    d.point((25, 7), fill=FLAME_CORE)
    d.point((8, 24), fill=INK)
    d.line([10, 28, 21, 20], fill=SUPERALLOY, width=2)  # screwdriver
    d.line([21, 20, 25, 17], fill=STEEL_LIGHT, width=1)  # bit
    d.point((24, 16), fill=COPPER)
    d.point((13, 15), fill=COPPER)  # copper accent rings
    d.point((14, 14), fill=COPPER)


def draw_multimeter(d, rng):
    d.rectangle([10, 7, 22, 26], fill=SUPERALLOY)
    d.rectangle([10, 7, 22, 26], outline=INK, width=1)
    d.rectangle([12, 9, 20, 15], fill=SCREEN)  # display
    d.line([13, 12, 19, 12], fill=GLYPH, width=1)  # reading
    d.point((14, 10), fill=GLYPH)
    d.ellipse([13, 18, 19, 24], outline=COPPER, width=1)  # dial
    d.line([16, 21, 18, 19], fill=COPPER, width=1)
    d.line([12, 26, 8, 30], fill=(196, 60, 40, 255), width=1)  # probes
    d.line([20, 26, 24, 30], fill=INK, width=1)


def draw_welder(d, rng):
    # mini welding torch: angled neck, nozzle, arc spark at the tip
    d.rectangle([9, 18, 17, 24], fill=SUPERALLOY)
    d.rectangle([9, 18, 17, 24], outline=INK, width=1)
    d.line([17, 19, 22, 13], fill=COPPER, width=2)  # neck
    d.line([22, 13, 24, 11], fill=STEEL_DARK, width=1)  # nozzle
    for dx, dy in ((0, -2), (2, -1), (2, 2), (-1, 2), (-2, 0)):
        d.point((25 + dx, 9 + dy), fill=(170, 220, 255, 255))  # arc sparks
    d.point((25, 9), fill=(240, 250, 255, 255))
    d.point((12, 21), fill=COPPER)
    d.line([10, 24, 8, 28], fill=INK, width=1)  # cable


def draw_toolset(d, rng):
    # fold-out multitool cluster: pliers, blade, driver, file
    d.rectangle([13, 12, 19, 27], fill=SUPERALLOY)
    d.rectangle([13, 12, 19, 27], outline=INK, width=1)
    d.polygon([(13, 12), (9, 5), (12, 4), (15, 10)], fill=STEEL_LIGHT)
    d.polygon([(19, 12), (23, 5), (20, 4), (17, 10)], fill=STEEL_LIGHT)
    d.line([14, 12, 8, 8], fill=COPPER, width=1)  # driver bit arm
    d.line([18, 12, 25, 9], fill=STEEL_DARK, width=1)  # file arm
    d.point((16, 16), fill=COPPER)  # pivot accents
    d.point((16, 22), fill=COPPER)
    d.line([15, 27, 15, 29], fill=INK, width=1)


def _tank(d, size, glyph):
    """Magenta internal tank housing; size = (x0, y0, x1, y1)."""
    x0, y0, x1, y1 = size
    d.rectangle([x0, y0, x1, y1], fill=MAGENTA)
    d.rectangle([x0, y0, x1, y1], outline=MAGENTA_DARK, width=1)
    d.rectangle([x0 + 2, y0 - 2, x0 + 6, y0], fill=MAGENTA_DARK)  # cap neck
    cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
    glyph(d, cx, cy)
    d.line([x0 + 2, y1 - 3, x1 - 2, y1 - 3], fill=MAGENTA_DARK, width=1)


def _drop(col):
    def g(d, cx, cy):
        d.polygon([(cx, cy - 4), (cx + 3, cy + 1), (cx + 3, cy + 3),
                   (cx - 3, cy + 3), (cx - 3, cy + 1)], fill=col)
        d.point((cx - 1, cy + 1), fill=(240, 240, 244, 255))
    return g


def draw_internal_ethanol(d, rng):
    _tank(d, (11, 11, 21, 27), _drop((120, 200, 240, 255)))


def draw_internal_gasoline(d, rng):
    _tank(d, (10, 9, 22, 28), _drop((240, 170, 60, 255)))


def draw_internal_oil(d, rng):
    _tank(d, (9, 8, 23, 29), _drop((52, 44, 40, 255)))


def draw_internal_battery(d, rng):
    d.rectangle([9, 12, 23, 28], fill=MAGENTA)
    d.rectangle([9, 12, 23, 28], outline=MAGENTA_DARK, width=1)
    for bx in (11, 16):  # two battery cells visible in the open top
        d.rectangle([bx, 6, bx + 4, 14], fill=(70, 130, 200, 255))
        d.rectangle([bx, 6, bx + 4, 14], outline=INK, width=1)
        d.rectangle([bx + 1, 4, bx + 3, 6], fill=STEEL_LIGHT)  # terminal
        d.point((bx + 2, 10), fill=(250, 220, 90, 255))  # charge glyph
    d.line([11, 22, 21, 22], fill=MAGENTA_DARK, width=1)
    d.point((12, 18), fill=(240, 240, 244, 255))
    d.point((20, 18), fill=(240, 240, 244, 255))


# ------------------------------------------------------------ kipple

def draw_kipple_object(d, rng):
    # strange object: an irregular monolith-grey polyhedron, odd marking
    mono = (176, 178, 184, 255)
    d.polygon([(9, 22), (11, 9), (18, 5), (24, 12), (22, 24), (14, 27)],
              fill=mono)
    d.polygon([(9, 22), (11, 9), (18, 5), (24, 12), (22, 24), (14, 27)],
              outline=INK)
    d.line([11, 9, 16, 16], fill=shade(mono, 0.8), width=1)  # facets
    d.line([18, 5, 16, 16], fill=shade(mono, 0.8), width=1)
    d.line([16, 16, 22, 24], fill=shade(mono, 0.8), width=1)
    d.ellipse([14, 13, 19, 18], outline=(90, 60, 140, 255), width=1)
    d.point((16, 15), fill=(90, 60, 140, 255))  # eye-like marking


def draw_kipple_wrapper(d, rng):
    # crinkled foil-grass wrapper with glyph text lines
    foil = (186, 196, 170, 255)
    d.polygon([(8, 9), (24, 7), (25, 23), (9, 25)], fill=foil)
    d.polygon([(8, 9), (24, 7), (25, 23), (9, 25)], outline=INK)
    for i, y in enumerate((11, 15, 19)):  # crinkles
        d.line([9, y + 1, 24, y], fill=shade(foil, 0.8), width=1)
    d.rectangle([11, 12, 17, 14], fill=(150, 60, 150, 255))  # colour band
    x = 19  # made-up glyph text
    for ch in "KIP":
        o = ord(ch)
        for col in range(2):
            bits = (o >> (col * 2)) & 0b111
            for row in range(3):
                if bits & (1 << row):
                    d.point((x + col, 12 + row), fill=INK)
        x += 3
    d.line([10, 21, 16, 21], fill=INK, width=1)


def _odd_device(d, rng, body, dark, light, form):
    """An odd device of unknown purpose; form picks the silhouette."""
    if form == "alu":  # bulky box, dial + lever
        d.rectangle([7, 10, 25, 26], fill=body)
        d.rectangle([7, 10, 25, 26], outline=dark, width=1)
        d.ellipse([10, 13, 16, 19], outline=dark, width=1)
        d.line([13, 16, 15, 14], fill=dark, width=1)
        d.line([21, 12, 24, 7], fill=dark, width=2)  # lever
        d.point((24, 7), fill=light)
        d.rectangle([18, 21, 23, 24], fill=dark)
    elif form == "copper":  # small coil-wound gadget
        d.rectangle([10, 12, 22, 25], fill=body)
        d.rectangle([10, 12, 22, 25], outline=dark, width=1)
        for y in range(14, 24, 2):  # coil windings
            d.line([11, y, 21, y], fill=dark, width=1)
        d.line([16, 12, 16, 7], fill=dark, width=1)
        d.ellipse([14, 4, 18, 7], outline=light, width=1)  # loop terminal
    elif form == "nanotubes":  # tiny intricate dark-blue lattice gadget
        d.rectangle([11, 11, 21, 22], fill=body)
        d.rectangle([11, 11, 21, 22], outline=dark, width=1)
        for gx in range(13, 21, 2):
            d.line([gx, 12, gx, 21], fill=light, width=1)
        for gy in range(13, 22, 2):
            d.line([12, gy, 20, gy], fill=light, width=1)
        d.point((16, 16), fill=(240, 250, 255, 255))
        d.line([21, 13, 25, 9], fill=dark, width=1)
        d.point((25, 9), fill=light)
    elif form == "plastic":  # big light-blue blob with knobs and a fin
        d.ellipse([6, 12, 26, 27], fill=body)
        d.ellipse([6, 12, 26, 27], outline=dark, width=1)
        d.polygon([(14, 12), (18, 5), (20, 12)], fill=body)
        d.polygon([(14, 12), (18, 5), (20, 12)], outline=dark)
        for kx, ky in ((10, 18), (20, 20), (15, 23)):
            d.ellipse([kx, ky, kx + 3, ky + 3], fill=light)
            d.ellipse([kx, ky, kx + 3, ky + 3], outline=dark, width=1)
    elif form == "steel":  # mid-size steel housing, vents + antenna
        d.rectangle([8, 11, 24, 26], fill=body)
        d.rectangle([8, 11, 24, 26], outline=dark, width=1)
        for y in (20, 22, 24):
            d.line([10, y, 17, y], fill=dark, width=1)  # vents
        d.ellipse([18, 13, 23, 18], outline=dark, width=1)
        d.point((20, 15), fill=light)
        d.line([12, 11, 10, 5], fill=dark, width=1)
        d.point((10, 5), fill=(196, 60, 40, 255))
    elif form == "unob":  # small white device, faint otherworldly glow
        for r in (8, 6):  # pale glow halo rings (opaque; alpha stays 0/255)
            d.ellipse([16 - r, 16 - r, 16 + r, 16 + r],
                      outline=(214, 220, 244, 255), width=1)
        d.polygon([(11, 21), (13, 10), (20, 9), (23, 16), (20, 23),
                   (14, 24)], fill=(246, 248, 252, 255))
        d.polygon([(11, 21), (13, 10), (20, 9), (23, 16), (20, 23),
                   (14, 24)], outline=(170, 176, 200, 255))
        d.point((17, 15), fill=(150, 120, 250, 255))
        d.point((15, 19), fill=(150, 120, 250, 255))


# -------------------------------------------------------------------- main

DRAW = {
    "crude_firestarter": draw_crude_firestarter,
    "electric_lighter": draw_electric_lighter,
    "fire_drill_large": draw_fire_drill,
    "fire_piston": draw_fire_piston,
    "lens_crude": draw_lens,
    "ref_lighter_butane": draw_ref_lighter_butane,
    "ref_lighter_on": draw_ref_lighter_on,
    "tinderbox_on": draw_tinderbox_on,
    "debug_apply_fault": draw_debug_fault,
    "debug_dimension_traveller": draw_debug_traveller,
    "debug_item_damager": draw_debug_damager,
    "debug_item_damager_adv": draw_debug_damager_adv,
    "debug_lux_meter": draw_debug_lux,
    "debug_mortar": draw_debug_mortar,
    "debug_timer": draw_debug_timer,
    "debug_weather_check": draw_debug_weather,
    "foldframe": draw_foldframe,
    "foldwoodframe": draw_foldwoodframe,
    "foldxlframe": draw_foldxlframe,
    "frame_concrete_mix": draw_frame_concrete_mix,
    "frame_wood_light": draw_frame_wood_light,
    "hdframe": draw_hdframe,
    "improvised_frame": draw_improvised_frame,
    "xlframe": draw_xlframe,
    "highland_black_incense": draw_incense_black,
    "highland_blue_incense": draw_incense_blue,
    "highland_golden_incense": draw_incense_golden,
    "highland_green_incense": draw_incense_green,
    "highland_purple_incense": draw_incense_purple,
    "highland_red_incense": draw_incense_red,
    "highland_white_incense": draw_incense_white,
    "highland_yellow_incense": draw_incense_yellow,
    "integrated_electrokit": draw_electrokit,
    "integrated_multimeter": draw_multimeter,
    "integrated_welder": draw_welder,
    "internal_battery_compartment": draw_internal_battery,
    "internal_ethanol_tank": draw_internal_ethanol,
    "internal_gasoline_tank": draw_internal_gasoline,
    "internal_oil_tank": draw_internal_oil,
    "toolset": draw_toolset,
    "kipple_object": draw_kipple_object,
    "kipple_wrapper": draw_kipple_wrapper,
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in DRAW:
        DRAW[sid](d, rng)
    elif sid.startswith("nl_kipple_scrap_"):
        forms = {
            "nl_kipple_scrap_alu": ((178, 182, 188, 255), STEEL_DARK,
                                    STEEL_LIGHT, "alu"),
            "nl_kipple_scrap_copper": (COPPER, (120, 62, 32, 255),
                                       (240, 180, 130, 255), "copper"),
            "nl_kipple_scrap_nanotubes": ((40, 60, 140, 255),
                                          (20, 30, 80, 255),
                                          (120, 160, 240, 255), "nanotubes"),
            "nl_kipple_scrap_plastic": ((150, 200, 240, 255),
                                        (80, 130, 180, 255),
                                        (220, 240, 252, 255), "plastic"),
            "nl_kipple_scrap_steel": (STEEL, STEEL_DARK, STEEL_LIGHT,
                                      "steel"),
            "nl_kipple_scrap_unob": ((246, 248, 252, 255),
                                     (170, 176, 200, 255),
                                     (240, 244, 255, 255), "unob"),
        }
        body, dark, light, form = forms[sid]
        _odd_device(d, rng, body, dark, light, form)
    else:
        raise KeyError(sid)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    # the batch file is authoritative: the embedded list must match it
    batch = os.path.expanduser("~/workspace/item-batches/76.txt")
    with open(batch) as f:
        file_ids = [ln.strip() for ln in f if ln.strip()]
    assert file_ids == IDS, "embedded IDS differ from 76.txt"
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
                       "preview_item76.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item76.png {prev.size}")


if __name__ == "__main__":
    main()
