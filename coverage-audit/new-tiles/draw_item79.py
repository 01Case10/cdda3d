#!/usr/bin/env python3
"""ITEM79: .50 Beowulf ammo, nether stones/orbs, cookware, bow
accessories, radios, fossils, brewing, dead-Yrax remains -- CDDA
Voxel coverage audit.

Families (art grounded in each id's game entry):

  .50 Beowulf (6) -- one big fat straight-wall cartridge, brass case.
    Penetrator: solid-copper bullet, blunt fluted nose.  XTP:
    copper jacket with a dark hollow-point cavity.  bp_ variants are
    black-powder loads: soot-darkened, dulled case.  reloaded_ are
    handloads: mixed brass tones, scratches, uneven seating line.
  Nether (6) -- abyss orb: black glass sphere with endless-depth
    swirl; default-dimension orb: dull brown, cracked dried-earth
    matte; escape stone ("portable portal"): yielding magenta blob
    with a portal swirl; beckoning stone: red blob, horror tendrils
    crawling from its base; calming stone: soft red blob with calm
    concentric ripples; unstable rift: a jagged glowing tear.
  Cookware (6) -- aluminum frying pan (round pan + handle), aluminum
    pot (lidded, side handles), cast-iron dutch oven (black, bail
    handle, feet), improvised oven (steel pot with a cut door),
    pizza cutter (wheel + handle), sippy cup (plastic, spout lid).
  Bow accessories (6) -- wooden arrow rest (small prong shelf),
    belt clip (slim steel clip), bow dampening kit (dark rubber
    pads), primitive kit (fur strips), bow stabilizer (single rod +
    counterweight), stabilizer system (three rods + dampeners).
  Comms (6) -- Balthazar's modified radio (speaker grille, dial
    removed, wire mess sprouting from the back), radio (on) with lit
    display and sound arcs, PAT earpiece off (no light) vs on
    (green LED + signal arcs), radio activation mod (small dark
    device, wires), remote vehicle controller (yellow RC handset,
    twin sticks, antenna).
  Fossils (6) -- bone fossil: cream bone embedded in a dark stone
    slab; large: bigger slab and bone.  Plaster replicas: clean
    white bone, no stone.  Flint: knapped dark chunk, sharp edge.
    Rock wool batt: foil-wrapped insulation slab, fibrous edge.
  Brewing (6) -- unfermented soy sauce (crock of pale brine, bean
    and koji flecks), unstrained soy sauce (dark jar, particles
    floating), malted grain (sack of golden kernels), malting grain
    active (kernels sprouting rootlets), soaked dandelion (jar of
    yellow flower heads in water), soaking dandelion (wilted heads
    in a bowl, mid-process).
  Dead Yrax (6) -- golden monolith (tall slab, asymmetric carved
    edges), inert delta (broadhead-shaped alloy), inert quadraphract
    (slumped arachnid of gold and crystal), its sickle leg, triakis
    remains (cracked steel shell, nested quartz tetrahedra inside),
    trifacet remains (split facet, milk-white quartz primitives).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "50beowulf_penetrator",
    "50beowulf_xtp",
    "bp_50beowulf_penetrator",
    "bp_50beowulf_xtp",
    "reloaded_50beowulf_penetrator",
    "reloaded_50beowulf_xtp",
    "abyss_teleport_orb",
    "beckoning_stone",
    "calming_stone",
    "default_dimension_teleport_orb",
    "escape_stone",
    "unstable_rift",
    "aluminum_pan",
    "dutch_oven",
    "improvised_oven",
    "pizza_cutter",
    "pot_aluminum",
    "sippy_cup",
    "arrowrest_wood",
    "belt_clip",
    "bow_silencer",
    "bow_stabilizer",
    "bow_stabilizer_set",
    "primitive_bow_silencer",
    "balthazar_radio",
    "pat_earpiece",
    "pat_earpiece_on",
    "radio_mod",
    "radio_on",
    "remotevehcontrol",
    "bone_fossil",
    "bone_plaster",
    "flint",
    "large_bone_fossil",
    "large_bone_plaster",
    "rock_wool_bat",
    "brew_soy_sauce",
    "malted_grain",
    "malting_grain_active",
    "soaked_dandelion",
    "soaking_dandelion_active",
    "soy_sauce_unstrained",
    "broken_golden_monolith",
    "broken_yrax_delta",
    "broken_yrax_quadraphract",
    "broken_yrax_quadraphract_leg",
    "broken_yrax_triakis",
    "broken_yrax_trifacet",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (46, 44, 48, 255)
BRASS = (196, 158, 74, 255)
BRASS_DARK = (150, 116, 52, 255)
BRASS_PALE = (224, 190, 110, 255)
COPPER = (184, 106, 62, 255)
COPPER_DARK = (140, 76, 42, 255)
SOOT = (70, 62, 58, 255)
STEEL = (150, 154, 160, 255)
STEEL_DARK = (96, 100, 106, 255)
ALU = (196, 200, 204, 255)
ALU_DARK = (140, 144, 150, 255)
IRON = (52, 52, 56, 255)
IRON_LIT = (88, 88, 94, 255)
WOOD = (110, 78, 44, 255)
WOOD_LIT = (150, 112, 64, 255)
RUBBER = (58, 54, 54, 255)
FUR = (168, 138, 96, 255)
FUR_DARK = (120, 94, 58, 255)
GOLD = (216, 172, 60, 255)
GOLD_DARK = (164, 126, 36, 255)
QUARTZ = (238, 236, 228, 255)
QUARTZ_SHADE = (198, 196, 188, 255)
BONE = (232, 224, 200, 255)
BONE_SHADE = (198, 188, 160, 255)
STONE = (104, 94, 84, 255)
STONE_DARK = (74, 66, 58, 255)
PLASTER = (242, 240, 236, 255)
GREEN = (62, 168, 78, 255)
GREEN_DARK = (38, 110, 52, 255)
YELLOW = (240, 200, 60, 255)
FLOWER = (246, 208, 70, 255)
MAGENTA = (206, 62, 168, 255)
MAGENTA_DARK = (150, 36, 122, 255)
RED = (196, 52, 44, 255)
RED_SOFT = (216, 110, 100, 255)
PURPLE = (110, 62, 168, 255)
BROWN = (128, 92, 56, 255)
EARTH = (138, 106, 72, 255)
EARTH_DARK = (104, 78, 50, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------- ammo

def draw_beowulf(d, rng, bullet, finish):
    """One fat straight-wall .50 Beowulf cartridge, upright, centred."""
    cx = 16
    # case: x 11..20, y 14..28; rim at the bottom
    case_col = BRASS
    if finish == "bp":
        case_col = shade(BRASS, 0.72)
    d.rectangle([cx - 5, 14, cx + 4, 27], fill=case_col)
    d.rectangle([cx - 6, 27, cx + 5, 29], fill=BRASS_DARK)  # rim
    d.line([cx - 5, 14, cx - 5, 27], fill=BRASS_DARK, width=1)
    d.line([cx + 4, 14, cx + 4, 27], fill=BRASS_DARK, width=1)
    d.line([cx - 3, 16, cx - 3, 26], fill=BRASS_PALE, width=1)  # sheen
    if finish == "bp":  # soot smudges from black powder
        for _ in range(7):
            sx = rng.randint(cx - 5, cx + 3)
            sy = rng.randint(15, 26)
            d.point((sx, sy), fill=SOOT)
            d.point((sx + 1, sy), fill=SOOT)
        d.line([cx - 5, 14, cx + 4, 14], fill=SOOT, width=1)
    if finish == "reloaded":  # mixed brass, scratches, uneven seat
        d.rectangle([cx + 1, 18, cx + 4, 27], fill=shade(BRASS, 0.85))
        for sx, sy in ((cx - 2, 20), (cx + 2, 23), (cx - 4, 25)):
            d.point((sx, sy), fill=BRASS_DARK)
    # bullet seated on top of the case mouth
    seat = 14 if finish != "reloaded" else 13
    if bullet == "penetrator":
        # solid copper, blunt nose with flutes
        d.polygon([(cx - 4, seat), (cx - 4, 7), (cx - 3, 5), (cx + 3, 5),
                   (cx + 4, 7), (cx + 4, seat)], fill=COPPER)
        d.rectangle([cx - 3, 5, cx + 3, 6], fill=COPPER_DARK)  # flat nose
        for fx in (cx - 2, cx, cx + 2):  # flute grooves
            d.line([fx, 6, fx, 10], fill=COPPER_DARK, width=1)
        d.line([cx - 4, 11, cx + 4, 11], fill=COPPER_DARK, width=1)
    else:  # XTP: copper jacket, hollow-point cavity
        d.polygon([(cx - 4, seat), (cx - 4, 8), (cx - 3, 6), (cx + 3, 6),
                   (cx + 4, 8), (cx + 4, seat)], fill=COPPER)
        d.ellipse([cx - 2, 5, cx + 2, 8], fill=(40, 30, 26, 255))  # cavity
        d.point((cx, 6), fill=(16, 12, 10, 255))
        for fx in (cx - 3, cx + 3):  # jacket skives
            d.line([fx, 8, fx, 11], fill=COPPER_DARK, width=1)
        d.line([cx - 2, 12, cx - 2, seat - 1], fill=BRASS_PALE,
               width=1)  # polymer-coat sheen


# ----------------------------------------------------------------- nether

def blob(d, rng, cx, cy, r, col, dark):
    pts = []
    for i in range(14):
        a = 2 * math.pi * i / 14
        rr = r + rng.randint(-1, 1)
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a) * 0.88))
    d.polygon(pts, fill=col)
    d.polygon([(x, y + r * 0.45) for x, y in pts[3:9]], fill=dark)


def draw_nether(d, rng, sid):
    cx, cy = 16, 16
    if sid == "abyss_teleport_orb":
        d.ellipse([cx - 9, cy - 9, cx + 9, cy + 9], fill=(18, 16, 26, 255))
        for r in (6, 4, 2):  # endless-depth swirl
            d.arc([cx - r, cy - r, cx + r, cy + r], start=40 + r * 30,
                  end=300 + r * 30, fill=PURPLE if r % 4 else
                  (70, 90, 160, 255), width=1)
        d.point((cx, cy), fill=(8, 6, 14, 255))
        d.point((cx - 4, cy - 5), fill=(150, 150, 180, 255))  # glint
    elif sid == "default_dimension_teleport_orb":
        d.ellipse([cx - 9, cy - 9, cx + 9, cy + 9], fill=EARTH)
        d.ellipse([cx - 9, cy + 2, cx + 9, cy + 9], fill=EARTH_DARK)
        cracks = [((cx - 6, cy - 4), (cx - 2, cy - 1), (cx - 3, cy + 3)),
                  ((cx - 2, cy - 1), (cx + 3, cy - 3), (cx + 6, cy - 1)),
                  ((cx - 2, cy - 1), (cx + 1, cy + 4), (cx, cy + 7))]
        for seg in cracks:  # dried-earth cracks
            d.line(list(seg), fill=EARTH_DARK, width=1)
    elif sid == "escape_stone":
        blob(d, rng, cx, cy, 9, MAGENTA, MAGENTA_DARK)
        for r in (5, 3):  # portal swirl on the yielding surface
            d.arc([cx - r, cy - r, cx + r, cy + r], start=20, end=290,
                  fill=(250, 190, 240, 255), width=1)
        d.point((cx, cy), fill=(250, 230, 250, 255))
    elif sid == "beckoning_stone":
        blob(d, rng, cx, cy - 1, 8, RED, shade(RED, 0.7))
        for tx in range(cx - 8, cx + 8, 4):  # horrors crawling out
            d.line([tx, cy + 6, tx + 1, cy + 9], fill=(60, 20, 24, 255),
                   width=1)
            d.line([tx + 1, cy + 9, tx + 3, cy + 10], fill=(60, 20, 24, 255),
                   width=1)
        d.point((cx - 2, cy - 3), fill=(250, 170, 160, 255))
        d.point((cx + 3, cy), fill=(120, 24, 20, 255))
    elif sid == "calming_stone":
        blob(d, rng, cx, cy, 9, RED_SOFT, shade(RED_SOFT, 0.78))
        for r in (6, 4, 2):  # calm concentric ripples
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      outline=(250, 214, 208, 255), width=1)
        d.point((cx, cy), fill=(250, 230, 226, 255))
    elif sid == "unstable_rift":
        # a jagged vertical tear with a bright core
        tear = [(cx - 2, 4), (cx + 2, 9), (cx - 1, 13), (cx + 3, 18),
                (cx - 1, 23), (cx + 1, 28), (cx - 2, 24), (cx - 4, 19),
                (cx - 1, 14), (cx - 4, 9)]
        d.polygon(tear, fill=RED)
        core = [(cx, 7), (cx + 1, 12), (cx - 1, 17), (cx + 1, 22),
                (cx, 26)]
        d.line(core, fill=(255, 214, 170, 255), width=1)
        for sx, sy in ((cx - 7, 8), (cx + 6, 12), (cx - 6, 21),
                       (cx + 7, 24)):  # sparking fragments
            d.point((sx, sy), fill=RED)
            d.point((sx, sy - 1), fill=(255, 170, 140, 255))


# --------------------------------------------------------------- cookware

def draw_cookware(d, rng, sid):
    if sid == "aluminum_pan":
        d.ellipse([6, 10, 24, 26], fill=ALU)
        d.ellipse([6, 10, 24, 26], outline=ALU_DARK, width=1)
        d.ellipse([9, 13, 21, 23], fill=ALU_DARK)
        d.ellipse([10, 14, 20, 22], fill=shade(ALU, 0.88))
        d.polygon([(23, 15), (30, 12), (30, 15), (23, 19)],
                  fill=STEEL_DARK)  # handle
        d.point((28, 13), fill=(30, 30, 34, 255))  # hanging hole
    elif sid == "pot_aluminum":
        d.rectangle([8, 13, 24, 27], fill=ALU)
        d.rectangle([8, 13, 24, 27], outline=ALU_DARK, width=1)
        d.ellipse([8, 9, 24, 14], fill=ALU_DARK)  # lid
        d.ellipse([13, 7, 19, 11], outline=STEEL_DARK, width=1)  # knob arc
        d.rectangle([5, 16, 8, 19], fill=ALU_DARK)  # side handles
        d.rectangle([24, 16, 27, 19], fill=ALU_DARK)
        d.line([10, 17, 10, 25], fill=(230, 232, 234, 255), width=1)
    elif sid == "dutch_oven":
        d.rectangle([7, 14, 25, 26], fill=IRON)
        d.rectangle([7, 14, 25, 26], outline=(24, 24, 28, 255), width=1)
        d.ellipse([7, 10, 25, 15], fill=IRON_LIT)  # heavy lid
        d.arc([11, 3, 21, 12], start=180, end=360, fill=STEEL_DARK,
              width=2)  # bail handle
        d.point((16, 11), fill=STEEL_DARK)
        for fx in (10, 16, 22):  # three feet
            d.rectangle([fx - 1, 26, fx + 1, 28], fill=IRON)
        d.line([9, 18, 9, 24], fill=IRON_LIT, width=1)
    elif sid == "improvised_oven":
        d.rectangle([8, 12, 24, 27], fill=STEEL)
        d.rectangle([8, 12, 24, 27], outline=STEEL_DARK, width=1)
        d.ellipse([8, 8, 24, 13], fill=STEEL_DARK)  # pot lid on top
        d.rectangle([12, 17, 20, 24], fill=(70, 66, 62, 255))  # cut door
        d.rectangle([12, 17, 20, 24], outline=INK, width=1)
        d.line([13, 20, 19, 20], fill=(120, 116, 110, 255),
               width=1)  # rack line inside the door
        d.point((19, 18), fill=INK)  # door knob
        for sx in (10, 22):  # rivets from the conversion
            d.point((sx, 14), fill=STEEL_DARK)
    elif sid == "pizza_cutter":
        d.ellipse([5, 6, 19, 20], fill=STEEL)  # wheel
        d.ellipse([5, 6, 19, 20], outline=STEEL_DARK, width=1)
        d.ellipse([10, 11, 14, 15], fill=ALU_DARK)  # hub
        d.point((12, 13), fill=INK)
        d.polygon([(16, 15), (27, 24), (24, 27), (13, 18)],
                  fill=WOOD)  # handle
        d.line([18, 18, 25, 25], fill=WOOD_LIT, width=1)
    elif sid == "sippy_cup":
        d.polygon([(10, 12), (22, 12), (21, 27), (11, 27)],
                  fill=(96, 178, 214, 255))  # cup body
        d.rectangle([9, 9, 23, 12], fill=(70, 140, 190, 255))  # lid
        d.polygon([(14, 9), (15, 6), (18, 6), (19, 9)],
                  fill=(70, 140, 190, 255))  # spout
        d.point((17, 7), fill=(40, 90, 130, 255))  # spout hole
        for hx in (8, 24):  # side grips
            d.rectangle([hx - 1, 14, hx + 1, 20], fill=(70, 140, 190, 255))
        d.line([12, 15, 12, 25], fill=(170, 220, 240, 255), width=1)


# ---------------------------------------------------------- bow accessories

def draw_bowacc(d, rng, sid):
    if sid == "arrowrest_wood":
        d.rectangle([8, 20, 24, 23], fill=WOOD)  # mounting block
        d.polygon([(18, 20), (18, 12), (21, 12), (21, 20)],
                  fill=WOOD_LIT)  # upright prong
        d.polygon([(18, 14), (26, 10), (26, 13), (19, 16)],
                  fill=WOOD_LIT)  # arrow shelf arm
        d.line([9, 21, 23, 21], fill=WOOD_LIT, width=1)
        d.point((11, 21), fill=INK)  # screw
    elif sid == "belt_clip":
        d.polygon([(12, 5), (16, 5), (17, 22), (20, 25), (20, 28),
                   (13, 28), (11, 24)], fill=STEEL)  # sprung clip
        d.line([14, 7, 15, 22], fill=ALU, width=1)
        for sy in (8, 12):  # screw holes
            d.point((14, sy), fill=INK)
            d.point((15, sy), fill=INK)
    elif sid in ("bow_silencer", "primitive_bow_silencer"):
        fur = sid.startswith("primitive")
        pad = FUR if fur else RUBBER
        dark = FUR_DARK if fur else (34, 32, 32, 255)
        for x0 in (7, 19):  # two limb pads
            d.rectangle([x0, 10, x0 + 6, 20], fill=pad)
            d.rectangle([x0, 10, x0 + 6, 20], outline=dark, width=1)
        # string silencers: fuzzy puffs on a string line
        d.line([16, 4, 16, 28], fill=INK, width=1)
        for cy in (9, 22):
            for _ in range(9):
                px = 16 + rng.randint(-3, 3)
                py = cy + rng.randint(-2, 2)
                d.point((px, py), fill=pad)
            d.point((16, cy), fill=dark)
    elif sid == "bow_stabilizer":
        d.rectangle([6, 14, 10, 18], fill=INK)  # mount
        d.rectangle([10, 15, 24, 17], fill=STEEL_DARK)  # rod
        d.rectangle([24, 12, 28, 20], fill=RUBBER)  # counterweight
        d.line([25, 13, 25, 19], fill=STEEL, width=1)
        d.line([11, 15, 23, 15], fill=ALU, width=1)
    elif sid == "bow_stabilizer_set":
        d.rectangle([5, 13, 9, 19], fill=INK)  # adjustable mount
        for y, x1 in ((9, 25), (16, 29), (23, 25)):  # three rods
            d.line([9, 16, x1, y], fill=STEEL_DARK, width=2)
            d.ellipse([x1 - 1, y - 2, x1 + 3, y + 2], fill=RUBBER)
            d.point((x1, y), fill=STEEL)
        d.point((7, 16), fill=STEEL)  # pivot bolt


# ------------------------------------------------------------------ comms

def radio_body(d, lit):
    d.rectangle([9, 8, 24, 28], fill=(176, 178, 182, 255))
    d.rectangle([9, 8, 24, 28], outline=INK, width=1)
    for gy in range(18, 27, 2):  # speaker grille
        d.line([11, gy, 17, gy], fill=STEEL_DARK, width=1)
    d.line([23, 8, 26, 3], fill=INK, width=1)  # antenna
    d.point((26, 3), fill=STEEL_DARK)
    if lit:
        d.rectangle([19, 11, 23, 15], fill=(140, 230, 160, 255))
        d.point((20, 12), fill=GREEN_DARK)
        d.point((21, 13), fill=GREEN_DARK)
        for r in (3, 5):  # sound arcs
            d.arc([24 - r, 20 - r, 24 + r, 20 + r], start=300, end=60,
                  fill=GREEN, width=1)
    else:
        d.ellipse([19, 11, 23, 15], outline=STEEL_DARK, width=1)
        d.point((21, 13), fill=STEEL_DARK)


def draw_comms(d, rng, sid):
    if sid == "balthazar_radio":
        radio_body(d, lit=False)
        # dial removed: blank screwed plate where it was
        d.rectangle([19, 11, 23, 15], fill=ALU_DARK)
        d.point((20, 12), fill=INK)
        d.point((22, 14), fill=INK)
        # a mess of wires sprouting from the back/top
        for i, (ex, ey) in enumerate(((6, 6), (4, 12), (6, 20), (27, 8))):
            sx, sy = (10, 9) if i < 3 else (23, 9)
            mx = (sx + ex) // 2 + rng.randint(-2, 2)
            my = (sy + ey) // 2 + rng.randint(-2, 2)
            col = (COPPER if i % 2 else STEEL_DARK)
            d.line([sx, sy, mx, my, ex, ey], fill=col, width=1)
            d.point((ex, ey), fill=col)
        d.rectangle([12, 5, 15, 8], fill=COPPER_DARK)  # soldered lump
    elif sid == "radio_on":
        radio_body(d, lit=True)
    elif sid in ("pat_earpiece", "pat_earpiece_on"):
        on = sid.endswith("_on")
        # cable looping down to the inline transponder box
        d.line([12, 8, 10, 16, 13, 22], fill=INK, width=1)
        d.line([15, 9, 18, 15, 16, 21], fill=INK, width=1)
        d.ellipse([10, 4, 17, 10], fill=GREEN)  # earpiece bud
        d.ellipse([10, 4, 17, 10], outline=GREEN_DARK, width=1)
        d.point((13, 7), fill=(30, 60, 36, 255))  # speaker hole
        d.rectangle([13, 21, 20, 27], fill=GREEN_DARK)  # transponder
        d.rectangle([13, 21, 20, 27], outline=INK, width=1)
        if on:
            d.point((15, 23), fill=(170, 255, 180, 255))  # LED
            for r in (3, 5):  # signal arcs
                d.arc([20 - r, 8 - r, 20 + r, 8 + r], start=300, end=60,
                      fill=GREEN, width=1)
        else:
            d.point((15, 23), fill=(24, 40, 28, 255))  # dark LED
    elif sid == "radio_mod":
        d.rectangle([10, 11, 22, 22], fill=STEEL_DARK)
        d.rectangle([10, 11, 22, 22], outline=INK, width=1)
        d.ellipse([13, 14, 17, 18], outline=ALU, width=1)  # relay coil
        d.point((20, 13), fill=RED)  # status LED (unlit red lens)
        d.line([22, 14, 27, 10], fill=COPPER, width=1)  # leads
        d.line([22, 19, 27, 22], fill=COPPER, width=1)
        d.line([12, 22, 9, 27], fill=INK, width=1)
        d.point((14, 20), fill=ALU)
        d.point((18, 20), fill=ALU)
    elif sid == "remotevehcontrol":
        d.rectangle([7, 10, 26, 26], fill=YELLOW)
        d.rectangle([7, 10, 26, 26], outline=INK, width=1)
        d.line([9, 10, 6, 4], fill=INK, width=1)  # antenna
        d.point((6, 4), fill=RED)
        d.rectangle([13, 12, 20, 16], fill=(120, 200, 220, 255))  # screen
        d.point((15, 14), fill=(40, 90, 110, 255))
        for cx in (11, 22):  # twin sticks
            d.ellipse([cx - 2, 19, cx + 2, 24], outline=INK, width=1)
            d.point((cx, 21), fill=INK)
        d.point((24, 12), fill=GREEN)  # power LED


# ---------------------------------------------------------------- fossils

def bone_shape(d, cx, cy, length, col, shade_col, angle=0.0):
    """A long bone: shaft with knobby ends, rotated by angle (radians)."""
    pts = []
    half = length // 2
    for t in range(-half, half + 1):
        w = 2 if abs(t) < half - 2 else 3
        pts.append((t, w))
    for t, w in pts:
        x = cx + int(t * math.cos(angle) - 0 * math.sin(angle))
        y = cy + int(t * math.sin(angle))
        d.rectangle([x - 1, y - w // 2, x + 1, y + w // 2], fill=col)
    for end in (-1, 1):  # knobby epiphyses
        ex = cx + int(end * half * math.cos(angle))
        ey = cy + int(end * half * math.sin(angle))
        d.ellipse([ex - 3, ey - 3, ex + 2, ey + 2], fill=col)
        d.ellipse([ex - 2, ey + 1, ex + 3, ey + 4], fill=shade_col)


def draw_fossil(d, rng, sid):
    if sid in ("bone_fossil", "large_bone_fossil"):
        large = sid.startswith("large")
        d.polygon([(4, 8), (27, 6), (29, 24), (6, 27), (3, 18)],
                  fill=STONE)  # rock slab
        d.polygon([(4, 8), (27, 6), (29, 24), (6, 27), (3, 18)],
                  outline=STONE_DARK)
        for _ in range(10):  # stone speckle
            d.point((rng.randint(5, 27), rng.randint(8, 25)),
                    fill=STONE_DARK)
        bone_shape(d, 16, 16, 20 if large else 15, BONE, BONE_SHADE,
                   angle=-0.35)
    elif sid in ("bone_plaster", "large_bone_plaster"):
        large = sid.startswith("large")
        bone_shape(d, 16, 16, 24 if large else 18, PLASTER,
                   QUARTZ_SHADE, angle=-0.35)
        d.point((10, 12), fill=QUARTZ_SHADE)  # casting seam flecks
        d.point((21, 20), fill=QUARTZ_SHADE)
    elif sid == "flint":
        d.polygon([(9, 22), (8, 12), (14, 6), (23, 8), (26, 16),
                   (21, 24), (13, 25)], fill=(86, 84, 88, 255))
        d.polygon([(14, 6), (23, 8), (20, 14), (13, 13)], fill=
                  (112, 110, 114, 255))  # flake scar
        d.line([9, 22, 13, 25, 21, 24], fill=(210, 208, 200, 255),
               width=1)  # sharp worked edge
        d.line([14, 6, 13, 13], fill=(64, 62, 66, 255), width=1)
    elif sid == "rock_wool_bat":
        d.rectangle([6, 9, 26, 24], fill=(206, 208, 212, 255))  # foil
        d.rectangle([6, 9, 26, 24], outline=ALU_DARK, width=1)
        d.line([7, 11, 25, 11], fill=(232, 233, 235, 255), width=1)
        # torn corner shows the fibrous brown batt inside
        d.polygon([(20, 18), (26, 18), (26, 24), (19, 24)], fill=BROWN)
        for _ in range(8):
            d.point((rng.randint(20, 25), rng.randint(19, 23)),
                    fill=EARTH_DARK)
        d.line([6, 16, 19, 16], fill=ALU_DARK, width=1)  # foil seam


# ---------------------------------------------------------------- brewing

def draw_brewing(d, rng, sid):
    if sid == "brew_soy_sauce":
        d.rectangle([9, 12, 23, 27], fill=(168, 138, 100, 255))  # crock
        d.rectangle([9, 12, 23, 27], outline=WOOD, width=1)
        d.ellipse([9, 9, 23, 14], fill=(226, 214, 178, 255))  # pale brine
        for _ in range(8):  # mashed soybean + koji flecks
            d.point((rng.randint(11, 21), rng.randint(10, 12)),
                    fill=(BROWN if rng.random() < 0.5 else
                          (190, 200, 150, 255)))
        d.line([10, 18, 22, 18], fill=WOOD, width=1)  # crock band
    elif sid == "soy_sauce_unstrained":
        d.rectangle([11, 10, 21, 27], fill=(70, 44, 26, 255))  # dark jar
        d.rectangle([11, 10, 21, 27], outline=(40, 26, 16, 255), width=1)
        d.rectangle([13, 6, 19, 10], fill=(70, 44, 26, 255))  # neck
        for _ in range(7):  # unstrained solid particles floating
            d.point((rng.randint(12, 20), rng.randint(12, 25)),
                    fill=(150, 110, 70, 255))
        d.line([12, 13, 12, 25], fill=(120, 84, 50, 255), width=1)
    elif sid in ("malted_grain", "malting_grain_active"):
        active = sid.endswith("active")
        d.polygon([(9, 14), (23, 14), (25, 27), (7, 27)],
                  fill=(178, 148, 100, 255))  # sack
        d.polygon([(9, 14), (23, 14), (25, 27), (7, 27)], outline=WOOD)
        d.ellipse([9, 11, 23, 16], fill=(216, 178, 96, 255))  # grain top
        for _ in range(12):  # kernels
            kx, ky = rng.randint(10, 22), rng.randint(12, 15)
            d.point((kx, ky), fill=(186, 146, 66, 255))
            if active:  # sprouting rootlets
                d.point((kx, ky - 1), fill=(120, 190, 90, 255))
        if active:
            for sx in (12, 17, 21):  # pale shoots above the sack rim
                d.line([sx, 11, sx, 8], fill=(150, 210, 120, 255), width=1)
        d.line([8, 21, 24, 21], fill=WOOD, width=1)  # sack tie band
    elif sid in ("soaked_dandelion", "soaking_dandelion_active"):
        active = sid.endswith("active")
        if active:  # open bowl, wilted brown-yellow heads
            d.pieslice([7, 12, 25, 26], start=0, end=180,
                       fill=(150, 116, 74, 255))
            d.ellipse([7, 10, 25, 15], fill=(120, 96, 54, 255))
            heads = (176, 148, 60, 255)
        else:  # glass jar of flowers in water
            d.rectangle([10, 8, 22, 27], fill=(168, 206, 214, 255))
            d.rectangle([10, 8, 22, 27], outline=ALU_DARK, width=1)
            d.rectangle([12, 5, 20, 8], fill=ALU_DARK)  # jar rim
            heads = FLOWER
        for hx, hy in ((12, 13), (17, 11), (20, 14), (14, 16)):
            d.ellipse([hx - 2, hy - 2, hx + 2, hy + 2], fill=heads)
            d.point((hx, hy), fill=(250, 240, 180, 255))
            d.line([hx, hy + 2, hx, hy + 5], fill=GREEN_DARK,
                   width=1)  # stem


# ------------------------------------------------------------------- yrax

def draw_yrax(d, rng, sid):
    if sid == "broken_golden_monolith":
        d.polygon([(11, 4), (20, 3), (22, 10), (21, 27), (12, 28),
                   (10, 14)], fill=GOLD)  # asymmetric slab
        d.polygon([(11, 4), (20, 3), (22, 10), (21, 27), (12, 28),
                   (10, 14)], outline=GOLD_DARK)
        d.line([13, 6, 14, 25], fill=GOLD_DARK, width=1)  # carved edge
        d.line([19, 6, 19, 20], fill=(240, 206, 110, 255), width=1)
        for qx, qy in ((15, 10), (17, 17), (14, 22)):  # quartz inlay
            d.point((qx, qy), fill=QUARTZ)
    elif sid == "broken_yrax_delta":
        d.polygon([(16, 5), (24, 20), (19, 18), (20, 26), (16, 22),
                   (12, 26), (13, 18), (8, 20)], fill=GOLD)  # broadhead
        d.polygon([(16, 5), (24, 20), (19, 18), (20, 26), (16, 22),
                   (12, 26), (13, 18), (8, 20)], outline=GOLD_DARK)
        d.line([16, 8, 16, 20], fill=GOLD_DARK, width=1)  # centre ridge
        d.point((16, 12), fill=QUARTZ)
    elif sid == "broken_yrax_quadraphract":
        d.ellipse([10, 10, 22, 20], fill=GOLD)  # slumped body
        d.ellipse([10, 10, 22, 20], outline=GOLD_DARK, width=1)
        d.ellipse([13, 12, 19, 17], fill=QUARTZ)  # crystal core showing
        d.ellipse([13, 12, 19, 17], outline=QUARTZ_SHADE, width=1)
        for i, (lx, ly) in enumerate(((7, 12), (6, 18), (24, 12),
                                      (25, 18))):  # folded legs
            d.line([16 + (lx - 16), 15, lx, ly], fill=GOLD_DARK, width=2)
            d.line([lx, ly, lx + (-2 if lx < 16 else 2), ly + 5],
                   fill=GOLD, width=2)
        d.point((14, 11), fill=GOLD_DARK)
    elif sid == "broken_yrax_quadraphract_leg":
        pts = [(8, 6), (12, 5), (17, 8), (21, 13), (23, 19), (22, 25),
               (20, 22), (18, 15), (14, 10), (10, 9)]  # sickle curve
        d.polygon(pts, fill=GOLD)
        d.polygon(pts, outline=GOLD_DARK)
        for jx, jy in ((13, 8), (18, 13)):  # locked joints
            d.ellipse([jx - 2, jy - 2, jx + 2, jy + 2], outline=GOLD_DARK,
                      width=1)
            d.point((jx, jy), fill=QUARTZ)
    elif sid == "broken_yrax_triakis":
        d.polygon([(7, 9), (16, 5), (25, 9), (23, 22), (10, 24)],
                  fill=STEEL_DARK)  # broken shell
        d.polygon([(7, 9), (16, 5), (25, 9), (23, 22), (10, 24)],
                  outline=INK)
        # nested quartz tetrahedra inside the crack
        d.polygon([(16, 10), (21, 19), (11, 19)], outline=QUARTZ, width=1)
        d.polygon([(16, 13), (19, 18), (13, 18)], outline=QUARTZ_SHADE,
                  width=1)
        d.point((16, 16), fill=QUARTZ)
        d.line([7, 9, 12, 13], fill=INK, width=1)  # crack lines
        d.line([25, 9, 20, 13], fill=INK, width=1)
    elif sid == "broken_yrax_trifacet":
        d.polygon([(6, 10), (15, 6), (15, 24), (8, 22)],
                  fill=STEEL_DARK)  # left facet half
        d.polygon([(26, 10), (17, 6), (17, 24), (24, 22)],
                  fill=STEEL)  # right facet half
        for gx, gy, s in ((13, 12, 2), (19, 15, 3), (14, 19, 2),
                          (20, 10, 1)):  # milk-white quartz primitives
            d.rectangle([gx - s, gy - s, gx + s, gy + s], fill=QUARTZ)
            d.rectangle([gx - s, gy - s, gx + s, gy + s],
                        outline=QUARTZ_SHADE, width=1)
        d.point((16, 8), fill=QUARTZ)


# -------------------------------------------------------------------- main

BEOWULF = {
    "50beowulf_penetrator": ("penetrator", "factory"),
    "50beowulf_xtp": ("xtp", "factory"),
    "bp_50beowulf_penetrator": ("penetrator", "bp"),
    "bp_50beowulf_xtp": ("xtp", "bp"),
    "reloaded_50beowulf_penetrator": ("penetrator", "reloaded"),
    "reloaded_50beowulf_xtp": ("xtp", "reloaded"),
}
NETHER = {"abyss_teleport_orb", "beckoning_stone", "calming_stone",
          "default_dimension_teleport_orb", "escape_stone", "unstable_rift"}
COOKWARE = {"aluminum_pan", "dutch_oven", "improvised_oven", "pizza_cutter",
            "pot_aluminum", "sippy_cup"}
BOWACC = {"arrowrest_wood", "belt_clip", "bow_silencer", "bow_stabilizer",
          "bow_stabilizer_set", "primitive_bow_silencer"}
COMMS = {"balthazar_radio", "pat_earpiece", "pat_earpiece_on", "radio_mod",
         "radio_on", "remotevehcontrol"}
FOSSIL = {"bone_fossil", "bone_plaster", "flint", "large_bone_fossil",
          "large_bone_plaster", "rock_wool_bat"}
BREWING = {"brew_soy_sauce", "malted_grain", "malting_grain_active",
           "soaked_dandelion", "soaking_dandelion_active",
           "soy_sauce_unstrained"}
YRAX = {"broken_golden_monolith", "broken_yrax_delta",
        "broken_yrax_quadraphract", "broken_yrax_quadraphract_leg",
        "broken_yrax_triakis", "broken_yrax_trifacet"}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in BEOWULF:
        bullet, finish = BEOWULF[sid]
        draw_beowulf(d, rng, bullet, finish)
    elif sid in NETHER:
        draw_nether(d, rng, sid)
    elif sid in COOKWARE:
        draw_cookware(d, rng, sid)
    elif sid in BOWACC:
        draw_bowacc(d, rng, sid)
    elif sid in COMMS:
        draw_comms(d, rng, sid)
    elif sid in FOSSIL:
        draw_fossil(d, rng, sid)
    elif sid in BREWING:
        draw_brewing(d, rng, sid)
    elif sid in YRAX:
        draw_yrax(d, rng, sid)
    else:
        raise KeyError(sid)
    alphas = {px[3] for px in img.getdata()}
    assert alphas <= {0, 255}, (sid, alphas)
    assert img.size == (32, 32)
    return img


def main():
    # the embedded list must equal the authoritative batch file verbatim
    batch = os.path.expanduser("~/workspace/item-batches/79.txt")
    with open(batch) as f:
        file_ids = [line.strip() for line in f if line.strip()]
    assert IDS == file_ids, "embedded IDS differ from 79.txt"
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
                       "preview_item79.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item79.png {prev.size}")


if __name__ == "__main__":
    main()
