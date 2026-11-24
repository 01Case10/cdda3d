#!/usr/bin/env python3
"""Batch ITEM22 (Phase 2g.4 scope extension): fake items,
first set -- workshop machine stand-ins, bionic stand-ins,
and pseudo items.

52 ids from data/json/items/fake.json (authoritative list
embedded below; source list: ~/workspace/item-batches/22.txt).
All 52 entries were read in fake.json; copy-from chains
resolved (most of these are bare ITEM stand-ins whose only
data is a name -- they exist so furniture/workstations can
lend their tool qualities to crafting). Each sprite draws
the object the entry names, as itself:

- Workshop machines (the *_tool and fake_* machine
  stand-ins): air compressor (tank + motor + gauge), band
  saw (C-frame, two wheels, table), drill press, lathe,
  vertical mill, jointer, planer, mitre saw, power hammer,
  hydraulic press, industrial oil press, Beverly shear,
  arc furnace (electrodes + glowing door), catalytic
  cracking reactor (vessel + pipes), concrete mixer
  (tilted drum), deep fryer (baskets up), tire changer
  (tire on turntable), oven, glassblower's crucible
  (brown, sym 'o', molten glow in the mouth).
- Smithing: classic anvil, chunkier dark heavy anvil,
  and the boulder anvil (a rounded rock with a flat top).
- Bionic stand-ins follow the ITEM01-03 CBM canister
  language where the entry is an implant: firestarter
  (flame pictogram), radio (grille + antenna + waves);
  the rest draw as the body-worn thing itself --
  surgical scalpel cluster on articulated arms, goggles,
  a straight razor, AR lenses (plastic, thin, HUD dot),
  the dimensional anchor as a ring module, the psyshield
  as a shielded skull, Pat's Infolink pieces as a pinned
  chip (handset pictogram; transponder dim off / lit
  green on), and the WearMan pair as skull-speaker
  modules (music notes vs. radio waves).
- Singles: 50 lb dynamite bundle (red, paper band, fuse),
  teeth-and-claws burrowing set over dirt, semi-ground
  grains as a coarse meal heap in a shallow bowl, sharp
  wreckage shards, smoldering embers with smoke wisps,
  and the generic fake crafting tool as crossed
  wrench/hammer with the small '?' badge the project
  uses for practice-only items.
- debug_item_search is a debug dummy ("You should never
  see it outside of debug", sym '?'): the project's
  debug-tile treatment -- black tile, red border, '?'
  and BUG band (f_clear / metabolism precedent).
- Appliance/exercise pseudo items: compressor unit
  (fan grille box), oxygen concentrator (vents +
  display), archery targets (yellow straw bale with
  painted rings, brown box target), mechanical
  ergometer (flywheel + seat rail), exercise machine
  (pulley frame + weight stack, sym 'T').

Items on transparent background, native 32x32, bodies
fully opaque (alpha only {0,255}); translucency faked
with pale opaque colors. Writes ONLY its own 52 folders
+ preview_item22.png. PIL only; per-sprite
random.Random(crc32(id)).
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "air_compressor_tool",
    "bandsaw_tool",
    "bio_scalpel",
    "boulder_anvil",
    "catalytic_cracking_reactor_tool",
    "concrete_mix_tool",
    "debug_item_search",
    "drill_press_tool",
    "fake_anvil",
    "fake_anvil_heavy",
    "fake_arc_furnace",
    "fake_beverly_shear",
    "fake_burrowing",
    "fake_crafting_tool",
    "fake_deep_fryer",
    "fake_dynamite_50lbs",
    "fake_firestarter",
    "fake_goggles",
    "fake_lathe",
    "fake_lift_heavy",
    "fake_lift_light",
    "fake_lift_medium",
    "fake_mill_vertical",
    "fake_milling_item",
    "fake_oil_press_industrial",
    "fake_power_tool",
    "fake_radio",
    "fake_razor",
    "fake_sharp_metal_wreckage",
    "fake_smoke_plume",
    "fake_tire_changer",
    "glassblowers_crucible_tool",
    "hydraulic_press_tool",
    "integrated_ar",
    "integrated_nl_dimensional_anchor",
    "integrated_nl_psyshield",
    "integrated_pat_radio",
    "integrated_pat_transponder",
    "integrated_pat_transponder_on",
    "internal_mp3_music",
    "internal_mp3_radio",
    "jointer_tool",
    "mitresaw_tool",
    "oven_tool",
    "planer_tool",
    "power_hammer_tool",
    "pseudo_app_hd_compressor_unit",
    "pseudo_app_oxygen_concentrator",
    "pseudo_archery_target_bale",
    "pseudo_archery_target_box",
    "pseudo_ergometer_mechanical",
    "pseudo_exercise_machine",
]

# Palette (flat UltiCa-style fills, dark outlines).
INK = (38, 40, 48, 255)
STEEL = (168, 174, 184, 255)
STEEL_L = (205, 211, 219, 255)
STEEL_D = (104, 110, 122, 255)
IRON = (74, 78, 88, 255)
IRON_D = (52, 55, 63, 255)
TEAL = (64, 128, 132, 255)
TEAL_D = (44, 92, 96, 255)
RED = (178, 52, 44, 255)
RED_D = (128, 36, 32, 255)
ORANGE = (226, 122, 40, 255)
GOLD = (216, 164, 60, 255)
WOOD = (146, 102, 62, 255)
WOOD_D = (104, 70, 40, 255)
CREAM = (232, 224, 200, 255)
PAPER = (226, 220, 198, 255)
BROWN = (122, 82, 52, 255)
GREEN = (84, 148, 74, 255)
CYAN_PALE = (168, 226, 228, 255)
GLOW_TEAL = (64, 208, 196, 255)
GLOW_GREEN = (96, 220, 110, 255)


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def box(d, xy, fill, outline=INK, w=1):
    d.rectangle(xy, fill=fill, outline=outline, width=w)


def rbox(d, xy, fill, outline=INK):
    d.rounded_rectangle(xy, radius=2, fill=fill, outline=outline)


def ell(d, xy, fill, outline=INK):
    d.ellipse(xy, fill=fill, outline=outline)


def poly(d, pts, fill, outline=INK):
    d.polygon(pts, fill=fill, outline=outline)


def line(d, pts, fill=INK, w=1):
    d.line(pts, fill=fill, width=w)


FONT35 = {
    "?": ["111", "001", "011", "000", "010"],
    "B": ["110", "101", "110", "101", "110"],
    "U": ["101", "101", "101", "101", "111"],
    "G": ["111", "100", "101", "101", "111"],
}


def glyph(d, ch, x, y, scale, color):
    for r, row in enumerate(FONT35[ch]):
        for c, bit in enumerate(row):
            if bit == "1":
                d.rectangle([x + c * scale, y + r * scale,
                             x + c * scale + scale - 1,
                             y + r * scale + scale - 1], fill=color)


def speckle(d, rng, box_xy, color, n):
    x0, y0, x1, y1 = box_xy
    for _ in range(n):
        x = rng.randint(x0, x1)
        y = rng.randint(y0, y1)
        d.point((x, y), fill=color)


# ---------------------------------------------------------------- machines

def air_compressor_tool(rng):
    img, d = canvas()
    # motor + pump on top of a horizontal tank
    box(d, [11, 7, 20, 13], STEEL_D)
    box(d, [21, 9, 25, 13], STEEL)
    ell(d, [23, 3, 28, 8], CREAM)  # gauge
    d.point((25, 5), fill=RED)
    rbox(d, [3, 14, 29, 24], RED)
    box(d, [3, 17, 29, 19], RED_D)
    line(d, [(7, 24), (7, 27)], INK, 2)
    line(d, [(25, 24), (25, 27)], INK, 2)
    line(d, [(5, 27), (10, 27)], INK, 2)
    line(d, [(22, 27), (28, 27)], INK, 2)
    d.arc([26, 18, 33, 27], start=90, end=300, fill=INK, width=1)
    return img


def bandsaw_tool(rng):
    img, d = canvas()
    box(d, [3, 3, 8, 28], TEAL_D)                      # column
    box(d, [3, 3, 27, 7], TEAL)                        # top arm
    ell(d, [19, 5, 28, 14], TEAL_D)                    # upper wheel housing
    ell(d, [21, 7, 26, 12], STEEL_L)
    box(d, [3, 20, 27, 28], TEAL_D)                    # lower body
    ell(d, [19, 20, 28, 29], TEAL)
    box(d, [11, 15, 29, 18], STEEL)                    # table
    line(d, [(22, 14), (22, 15)], INK, 1)              # blade above table
    line(d, [(22, 18), (22, 21)], INK, 1)              # blade below table
    box(d, [20, 11, 24, 14], STEEL_D)                  # blade guard
    return img


def drill_press_tool(rng):
    img, d = canvas()
    box(d, [4, 26, 28, 29], IRON)                      # base
    box(d, [6, 5, 10, 26], STEEL_D)                    # column
    box(d, [6, 3, 25, 9], TEAL)                        # head
    box(d, [19, 9, 23, 13], STEEL_D)                   # chuck housing
    line(d, [(21, 13), (21, 18)], STEEL_L, 1)          # bit
    box(d, [12, 18, 28, 21], STEEL)                    # table
    for dx in (0, 3, -3):                              # feed spokes
        line(d, [(25, 6), (25 + dx, 2 if dx == 0 else 4)], INK, 1)
    ell(d, [24, 1, 27, 4], STEEL_L)
    return img


def fake_lathe(rng):
    img, d = canvas()
    box(d, [3, 24, 29, 27], IRON)                      # stand
    line(d, [(6, 27), (6, 29)], INK, 2)
    line(d, [(26, 27), (26, 29)], INK, 2)
    box(d, [3, 19, 29, 24], STEEL_D)                   # bed
    box(d, [4, 9, 11, 19], TEAL)                       # headstock
    ell(d, [11, 11, 17, 17], STEEL_L)                  # chuck
    d.point((14, 14), fill=INK)
    line(d, [(17, 14), (24, 14)], STEEL_L, 1)          # workpiece
    box(d, [24, 11, 29, 19], STEEL)                    # tailstock
    box(d, [17, 16, 21, 19], IRON)                     # tool post
    return img


def fake_mill_vertical(rng):
    img, d = canvas()
    box(d, [21, 4, 28, 29], STEEL_D)                   # column
    box(d, [7, 3, 24, 9], TEAL)                        # head arm
    box(d, [11, 9, 16, 14], STEEL)                     # spindle head
    line(d, [(13, 14), (13, 17)], STEEL_L, 1)          # bit
    box(d, [4, 18, 25, 21], STEEL_L)                   # table
    box(d, [17, 21, 26, 26], STEEL_D)                  # knee
    box(d, [4, 27, 28, 29], IRON)                      # base
    ell(d, [22, 12, 26, 16], STEEL_L)                  # handwheel
    return img


def jointer_tool(rng):
    img, d = canvas()
    box(d, [5, 18, 27, 25], TEAL_D)                    # body
    box(d, [2, 14, 30, 18], STEEL_L)                   # bed
    box(d, [5, 7, 9, 14], STEEL)                       # fence
    line(d, [(5, 7), (9, 14)], INK, 1)
    box(d, [14, 11, 20, 14], ORANGE)                   # cutter guard
    line(d, [(8, 25), (8, 29)], INK, 2)
    line(d, [(24, 25), (24, 29)], INK, 2)
    return img


def planer_tool(rng):
    img, d = canvas()
    # board passing through the machine
    box(d, [1, 16, 31, 20], WOOD)
    line(d, [(1, 18), (31, 18)], WOOD_D, 1)
    box(d, [7, 9, 25, 23], TEAL)                       # body over the board
    box(d, [7, 9, 25, 12], TEAL_D)
    d.arc([12, 4, 20, 11], start=180, end=360, fill=INK, width=2)  # handle
    ell(d, [25, 14, 29, 18], STEEL_L)                  # crank
    box(d, [10, 20, 22, 23], IRON)
    return img


def mitresaw_tool(rng):
    img, d = canvas()
    box(d, [4, 24, 28, 28], IRON)                      # base
    box(d, [23, 10, 27, 24], STEEL_D)                  # rear column/arm
    ell(d, [8, 5, 24, 21], STEEL_L)                    # blade
    ell(d, [13, 10, 19, 16], STEEL_D)
    d.arc([8, 5, 24, 21], start=180, end=330, fill=ORANGE, width=3)  # guard
    line(d, [(21, 7), (27, 4)], INK, 2)                # handle
    box(d, [10, 21, 22, 24], STEEL)                    # fence/table slot
    return img


def power_hammer_tool(rng):
    img, d = canvas()
    box(d, [22, 3, 28, 29], IRON)                      # column
    box(d, [7, 3, 28, 8], IRON)                        # top arm
    box(d, [8, 8, 15, 14], STEEL_D)                    # hammer head
    line(d, [(11, 14), (11, 18)], STEEL_L, 2)          # ram
    box(d, [7, 18, 17, 22], STEEL)                     # anvil block
    box(d, [4, 25, 29, 29], IRON_D)                    # base
    ell(d, [21, 11, 27, 17], STEEL)                    # belt wheel
    return img


def hydraulic_press_tool(rng):
    img, d = canvas()
    box(d, [5, 4, 9, 28], IRON)                        # columns
    box(d, [23, 4, 27, 28], IRON)
    box(d, [5, 4, 27, 9], IRON_D)                      # top beam
    box(d, [12, 9, 20, 15], STEEL_D)                   # cylinder
    box(d, [10, 17, 22, 19], STEEL_L)                  # press plate
    box(d, [7, 23, 25, 25], STEEL)                     # work table
    box(d, [4, 27, 28, 29], IRON_D)                    # base
    ell(d, [24, 11, 28, 15], STEEL_L)                  # pump handle wheel
    return img


def fake_oil_press_industrial(rng):
    img, d = canvas()
    box(d, [6, 6, 10, 27], IRON)
    box(d, [22, 6, 26, 27], IRON)
    box(d, [6, 6, 26, 10], IRON_D)
    ell(d, [11, 1, 21, 7], STEEL_L)                    # screw handwheel
    line(d, [(16, 7), (16, 12)], STEEL_D, 2)           # screw
    box(d, [11, 12, 21, 15], STEEL_D)                  # press plate
    box(d, [11, 16, 21, 24], WOOD_D)                   # press barrel
    line(d, [(11, 19), (21, 19)], WOOD, 1)
    line(d, [(11, 22), (21, 22)], WOOD, 1)
    box(d, [5, 25, 27, 28], IRON_D)                    # drip tray/base
    d.point((16, 25), fill=GOLD)
    return img


def fake_beverly_shear(rng):
    img, d = canvas()
    # throatless shear: heavy curved frame, pivot blade, long lever
    poly(d, [(6, 12), (14, 10), (16, 16), (12, 22), (7, 22)], IRON)
    box(d, [4, 22, 18, 26], IRON_D)                    # base
    line(d, [(14, 11), (28, 3)], STEEL_D, 3)           # lever handle
    ell(d, [26, 1, 30, 5], WOOD)                       # grip
    poly(d, [(13, 11), (21, 13), (14, 17)], STEEL_L)   # blade
    ell(d, [12, 10, 16, 14], STEEL)
    box(d, [16, 17, 24, 19], STEEL)                    # work piece
    return img


def fake_arc_furnace(rng):
    img, d = canvas()
    for x in (10, 16, 22):                             # electrodes
        line(d, [(x, 1), (x, 9)], IRON_D, 2)
    box(d, [5, 9, 27, 28], IRON)                       # furnace body
    box(d, [5, 9, 27, 12], IRON_D)
    box(d, [11, 16, 21, 25], IRON_D)                   # door
    box(d, [12, 19, 20, 21], ORANGE)                   # glowing slit
    box(d, [13, 20, 19, 21], GOLD)
    line(d, [(8, 28), (8, 30)], INK, 2)
    line(d, [(24, 28), (24, 30)], INK, 2)
    return img


def catalytic_cracking_reactor_tool(rng):
    img, d = canvas()
    ell(d, [11, 3, 21, 9], STEEL)                      # dome
    box(d, [11, 6, 21, 27], STEEL)                     # vessel
    line(d, [(11, 12), (21, 12)], STEEL_D, 1)
    line(d, [(11, 20), (21, 20)], STEEL_D, 1)
    line(d, [(21, 10), (26, 10)], STEEL_D, 2)          # side pipe
    line(d, [(26, 10), (26, 22)], STEEL_D, 2)
    ell(d, [23, 22, 29, 28], STEEL)                    # valve body
    ell(d, [4, 13, 9, 18], CREAM)                      # gauge
    d.point((6, 15), fill=RED)
    line(d, [(11, 15), (4, 15)], STEEL_D, 1)
    box(d, [8, 27, 24, 29], IRON)
    return img


def concrete_mix_tool(rng):
    img, d = canvas()
    # tilted drum
    poly(d, [(8, 6), (20, 3), (26, 10), (22, 18), (11, 19), (6, 12)], ORANGE)
    ell(d, [6, 5, 13, 12], IRON_D)                     # drum mouth
    line(d, [(12, 19), (8, 26)], IRON, 2)              # frame legs
    line(d, [(21, 18), (25, 26)], IRON, 2)
    line(d, [(8, 22), (25, 22)], IRON, 1)
    ell(d, [5, 24, 11, 30], IRON_D)                    # wheels
    ell(d, [22, 24, 28, 30], IRON_D)
    ell(d, [7, 26, 9, 28], STEEL_L)
    ell(d, [24, 26, 26, 28], STEEL_L)
    line(d, [(24, 8), (29, 5)], STEEL_D, 2)            # handle
    return img


def fake_deep_fryer(rng):
    img, d = canvas()
    box(d, [4, 11, 28, 27], STEEL)                     # body
    box(d, [4, 11, 28, 14], STEEL_D)                   # rim/control strip
    ell(d, [23, 11, 27, 15], IRON)
    d.point((25, 12), fill=RED)
    box(d, [6, 14, 26, 17], GOLD)                      # oil surface
    # two raised wire baskets
    for x0 in (7, 18):
        box(d, [x0, 5, x0 + 8, 12], None)
        for gx in range(x0 + 1, x0 + 8, 2):
            line(d, [(gx, 6), (gx, 11)], STEEL_D, 1)
        line(d, [(x0, 8), (x0 + 8, 8)], STEEL_D, 1)
        line(d, [(x0 + 3, 5), (x0 + 3, 2)], INK, 1)
    line(d, [(8, 27), (8, 29)], INK, 2)
    line(d, [(24, 27), (24, 29)], INK, 2)
    return img


def fake_tire_changer(rng):
    img, d = canvas()
    box(d, [4, 24, 28, 28], IRON)                      # base cabinet
    box(d, [22, 8, 27, 24], STEEL_D)                   # column
    line(d, [(22, 10), (15, 10)], STEEL_D, 2)          # swing arm
    line(d, [(15, 10), (15, 14)], STEEL_D, 2)          # mount head
    ell(d, [5, 19, 21, 25], STEEL_D)                   # turntable
    ell(d, [8, 12, 20, 22], IRON_D)                    # tire
    ell(d, [11, 15, 17, 19], STEEL_L)                  # rim
    return img


def oven_tool(rng):
    img, d = canvas()
    box(d, [5, 5, 27, 28], STEEL_L)                    # body
    ell(d, [8, 5, 13, 8], IRON_D)                      # burners
    ell(d, [19, 5, 24, 8], IRON_D)
    for x in (10, 16, 22):                             # knobs
        ell(d, [x - 1, 9, x + 1, 11], IRON)
    line(d, [(8, 13), (24, 13)], INK, 2)               # handle
    box(d, [9, 15, 23, 24], IRON_D)                    # door window
    box(d, [11, 17, 21, 22], (70, 60, 66, 255))
    return img


def glassblowers_crucible_tool(rng):
    img, d = canvas()
    poly(d, [(8, 9), (24, 9), (22, 20), (18, 26), (14, 26), (10, 20)], BROWN)
    box(d, [7, 8, 25, 11], WOOD_D)                     # rim band
    ell(d, [11, 9, 21, 13], IRON_D)                    # mouth
    ell(d, [13, 10, 19, 12], ORANGE)                   # molten glass glow
    speckle(d, rng, (11, 15, 21, 24), WOOD_D, 8)
    return img


# ---------------------------------------------------------------- smithing

def anvil(rng, heavy=False):
    img, d = canvas()
    body = IRON if heavy else STEEL_D
    top = STEEL if heavy else STEEL_L
    y0 = 7 if heavy else 9
    poly(d, [(4, y0), (22, y0), (30, y0 + 3), (30, y0 + 5), (22, y0 + 6),
             (4, y0 + 6)], top)                        # face + horn
    waist = 3 if heavy else 5
    box(d, [13 - (2 if heavy else 0), y0 + 6, 19 + (2 if heavy else 0),
            y0 + 6 + waist + 4], body)                 # waist
    box(d, [7, y0 + 11 + waist, 25, y0 + 15 + waist], body)  # base
    box(d, [5, y0 + 15 + waist, 27, y0 + 17 + waist], INK if heavy else IRON_D)
    return img


def fake_anvil(rng):
    return anvil(rng, heavy=False)


def fake_anvil_heavy(rng):
    return anvil(rng, heavy=True)


def boulder_anvil(rng):
    img, d = canvas()
    poly(d, [(7, 11), (11, 8), (21, 8), (26, 12), (27, 19), (23, 25),
             (10, 25), (5, 19)], (128, 130, 136, 255))
    box(d, [10, 7, 22, 10], STEEL)                     # flattened top
    speckle(d, rng, (8, 12, 25, 24), STEEL_D, 14)
    speckle(d, rng, (8, 12, 25, 24), STEEL_L, 6)
    return img


# ---------------------------------------------------------------- hoists

def hoist(rng, thick, tone, base_tone):
    img, d = canvas()
    line(d, [(6, 28), (13, 5)], tone, thick)           # legs
    line(d, [(26, 28), (19, 5)], tone, thick)
    line(d, [(10, 4), (22, 4)], tone, thick + 1)       # top beam
    line(d, [(8, 20), (24, 20)], tone, max(1, thick - 1))  # cross brace
    line(d, [(16, 5), (16, 13)], IRON_D, 1)            # chain
    for y in range(6, 13, 2):
        d.point((15, y), fill=STEEL_L)
        d.point((17, y), fill=STEEL_L)
    d.arc([13, 13, 19, 20], start=0, end=270, fill=base_tone, width=2)  # hook
    box(d, [3, 27, 10, 29], base_tone)                 # feet
    box(d, [22, 27, 29, 29], base_tone)
    return img


def fake_lift_light(rng):
    return hoist(rng, 1, STEEL, STEEL_D)


def fake_lift_medium(rng):
    return hoist(rng, 2, STEEL_D, IRON)


def fake_lift_heavy(rng):
    return hoist(rng, 3, IRON, IRON_D)


# ---------------------------------------------------------------- bionics

def cbm_canister(rng):
    img, d = canvas()
    rbox(d, [10, 6, 22, 27], STEEL)
    box(d, [10, 9, 22, 12], STEEL_D)                   # accent band
    box(d, [12, 15, 20, 24], IRON_D)                   # inset panel
    ell(d, [19, 7, 21, 9], GLOW_GREEN)
    return img, d


def fake_firestarter(rng):
    img, d = cbm_canister(rng)
    poly(d, [(16, 16), (19, 20), (17, 21), (18, 24), (16, 25), (14, 24),
             (15, 21), (13, 20)], ORANGE)             # flame pictogram
    ell(d, [15, 21, 17, 24], GOLD)
    return img


def fake_radio(rng):
    img, d = cbm_canister(rng)
    for gy in range(16, 24, 2):                        # speaker grille
        for gx in range(13, 20, 2):
            d.point((gx, gy), fill=STEEL_L)
    line(d, [(21, 7), (27, 1)], STEEL_L, 1)            # antenna
    d.arc([22, 12, 30, 20], start=270, end=60, fill=GLOW_TEAL, width=1)
    return img


def bio_scalpel(rng):
    img, d = canvas()
    box(d, [5, 23, 27, 28], STEEL_D)                   # base unit
    box(d, [5, 23, 27, 25], STEEL)
    for bx, tx, ty in ((9, 6, 7), (16, 16, 4), (23, 26, 7)):
        line(d, [(bx, 23), (tx, ty + 5)], STEEL_D, 2)  # articulated arm
        poly(d, [(tx - 1, ty), (tx + 2, ty + 1), (tx, ty + 6), (tx - 2, ty + 4)],
             STEEL_L)                                  # scalpel blade
    ell(d, [14, 24, 18, 27], GLOW_TEAL)
    return img


def fake_goggles(rng):
    img, d = canvas()
    line(d, [(1, 11), (5, 12)], IRON, 2)               # straps
    line(d, [(31, 11), (27, 12)], IRON, 2)
    ell(d, [4, 8, 15, 19], IRON)                       # lens frames
    ell(d, [17, 8, 28, 19], IRON)
    ell(d, [6, 10, 13, 17], TEAL_D)
    ell(d, [19, 10, 26, 17], TEAL_D)
    d.point((8, 11), fill=CYAN_PALE)
    d.point((21, 11), fill=CYAN_PALE)
    line(d, [(15, 12), (17, 12)], IRON, 2)             # bridge
    return img


def fake_razor(rng):
    img, d = canvas()
    # open straight razor, diagonal
    poly(d, [(6, 20), (20, 6), (24, 5), (23, 8), (10, 21)], STEEL_L)
    line(d, [(6, 20), (10, 21)], STEEL_D, 1)
    poly(d, [(6, 20), (3, 25), (6, 27), (11, 22)], WOOD_D)  # handle
    ell(d, [7, 22, 9, 24], GOLD)
    return img


def integrated_ar(rng):
    img, d = canvas()
    line(d, [(2, 10), (6, 12)], IRON, 1)               # temples
    line(d, [(30, 10), (26, 12)], IRON, 1)
    rbox(d, [5, 10, 15, 18], CYAN_PALE)                # thin lenses
    rbox(d, [17, 10, 27, 18], CYAN_PALE)
    line(d, [(15, 12), (17, 12)], IRON, 1)
    d.rectangle([20, 12, 24, 15], outline=TEAL)
    for gx in range(21, 24):
        for gy in range(13, 15):
            d.point((gx, gy), fill=TEAL)
    d.point((8, 12), fill=TEAL)
    return img


def integrated_nl_dimensional_anchor(rng):
    img, d = canvas()
    rbox(d, [9, 7, 23, 26], IRON)
    box(d, [9, 10, 23, 12], IRON_D)
    ell(d, [12, 14, 20, 24], None)
    d.ellipse([12, 14, 20, 24], outline=GLOW_TEAL, width=2)  # anchor ring
    line(d, [(16, 15), (16, 23)], GLOW_TEAL, 1)
    line(d, [(13, 19), (19, 19)], GLOW_TEAL, 1)
    ell(d, [20, 8, 22, 10], GLOW_TEAL)
    return img


def integrated_nl_psyshield(rng):
    img, d = canvas()
    # skull profile with a shield sheen over the cranium
    ell(d, [9, 6, 23, 20], (196, 200, 208, 255))
    box(d, [13, 18, 20, 24], (196, 200, 208, 255))
    ell(d, [17, 12, 20, 15], IRON_D)                   # eye socket
    d.arc([6, 3, 26, 17], start=180, end=330, fill=CYAN_PALE, width=2)
    d.arc([8, 5, 24, 18], start=185, end=325, fill=GLOW_TEAL, width=1)
    poly(d, [(24, 20), (27, 22), (24, 26), (21, 22)], CYAN_PALE)  # shield fleck
    return img


def pat_chip(rng, mode):
    img, d = canvas()
    rbox(d, [10, 7, 22, 25], IRON)
    for gx in range(12, 21, 2):                        # pins
        line(d, [(gx, 25), (gx, 28)], GOLD, 1)
    box(d, [12, 10, 20, 20], IRON_D)
    if mode == "radio":
        # handset pictogram
        d.arc([13, 12, 19, 18], start=200, end=340, fill=GLOW_TEAL, width=2)
        ell(d, [12, 15, 14, 18], GLOW_TEAL)
        ell(d, [18, 15, 20, 18], GLOW_TEAL)
        d.arc([13, 9, 21, 15], start=270, end=350, fill=GLOW_TEAL, width=1)
    elif mode == "off":
        ell(d, [14, 13, 18, 17], STEEL_D)              # dim transponder eye
        d.point((16, 15), fill=IRON_D)
        d.point((13, 22), fill=RED)                    # off indicator
    else:
        ell(d, [14, 13, 18, 17], GLOW_GREEN)
        d.arc([12, 10, 20, 18], start=270, end=60, fill=GLOW_TEAL, width=1)
        d.arc([11, 8, 22, 20], start=270, end=60, fill=GLOW_TEAL, width=1)
        d.point((13, 22), fill=GREEN)
    return img


def integrated_pat_radio(rng):
    return pat_chip(rng, "radio")


def integrated_pat_transponder(rng):
    return pat_chip(rng, "off")


def integrated_pat_transponder_on(rng):
    return pat_chip(rng, "on")


def weaman(rng, music=True):
    img, d = canvas()
    rbox(d, [8, 8, 24, 25], IRON)                      # skull speaker module
    box(d, [8, 11, 24, 13], IRON_D)
    ell(d, [12, 15, 20, 23], STEEL_D)                  # speaker cone
    ell(d, [14, 17, 18, 21], IRON_D)
    d.point((16, 19), fill=STEEL_L)
    if music:
        for nx, ny in ((26, 10), (27, 17)):
            ell(d, [nx - 2, ny + 3, nx + 1, ny + 6], GLOW_TEAL)
            line(d, [(nx + 1, ny + 4), (nx + 1, ny)], GLOW_TEAL, 1)
            line(d, [(nx + 1, ny), (nx + 3, ny + 1)], GLOW_TEAL, 1)
    else:
        line(d, [(23, 9), (28, 3)], STEEL_L, 1)        # antenna
        d.arc([22, 12, 30, 20], start=270, end=60, fill=GLOW_TEAL, width=1)
        d.arc([21, 9, 31, 21], start=270, end=60, fill=GLOW_TEAL, width=1)
    return img


def internal_mp3_music(rng):
    return weaman(rng, music=True)


def internal_mp3_radio(rng):
    return weaman(rng, music=False)


# ---------------------------------------------------------------- singles

def fake_dynamite_50lbs(rng):
    img, d = canvas()
    for i, x in enumerate((6, 11, 16, 21)):
        rbox(d, [x, 8, x + 5, 26], RED)
        line(d, [(x + 2, 9), (x + 2, 25)], RED_D, 1)
    box(d, [5, 15, 27, 19], PAPER)                     # paper band
    line(d, [(16, 8), (16, 4)], WOOD_D, 1)             # fuse
    d.arc([16, 0, 24, 8], start=180, end=300, fill=WOOD_D, width=1)
    ell(d, [23, 1, 25, 3], GOLD)                       # spark
    return img


def fake_burrowing(rng):
    img, d = canvas()
    # dirt mound
    poly(d, [(3, 26), (8, 21), (14, 24), (20, 20), (26, 23), (29, 26)], BROWN)
    speckle(d, rng, (6, 22, 27, 26), WOOD_D, 10)
    # claws
    for cx in (8, 14, 20):
        d.arc([cx - 3, 6, cx + 4, 15], start=300, end=140, fill=CREAM, width=2)
    # fangs
    poly(d, [(24, 7), (27, 7), (25, 13)], CREAM)
    poly(d, [(28, 9), (30, 10), (27, 14)], CREAM)
    return img


def fake_milling_item(rng):
    img, d = canvas()
    # shallow bowl of coarse grain paste
    d.arc([5, 12, 27, 30], start=0, end=180, fill=WOOD_D, width=3)
    ell(d, [6, 13, 26, 19], (206, 178, 128, 255))      # meal surface
    speckle(d, rng, (8, 14, 24, 18), WOOD, 16)
    speckle(d, rng, (8, 14, 24, 18), CREAM, 8)
    poly(d, [(12, 12), (17, 9), (21, 12)], (216, 190, 140, 255))  # heap
    speckle(d, rng, (13, 10, 20, 12), WOOD, 5)
    return img


def fake_sharp_metal_wreckage(rng):
    img, d = canvas()
    poly(d, [(5, 22), (12, 6), (15, 10), (11, 16), (14, 22)], STEEL_D)
    poly(d, [(14, 25), (20, 10), (24, 13), (21, 25)], STEEL)
    poly(d, [(21, 24), (27, 16), (29, 20), (26, 26)], IRON)
    line(d, [(8, 18), (12, 12)], STEEL_L, 1)
    line(d, [(18, 20), (21, 14)], STEEL_L, 1)
    return img


def fake_smoke_plume(rng):
    img, d = canvas()
    # smoke wisps (pale opaque gray)
    for x, ph in ((11, 0), (16, 2), (21, 1)):
        pts = [(x + (2 if (y // 3 + ph) % 2 else -1), y) for y in range(3, 17, 2)]
        line(d, pts, (196, 198, 204, 255), 2)
    # ember pile
    poly(d, [(6, 26), (10, 21), (16, 23), (21, 20), (26, 26)], IRON_D)
    for ex, ey in ((11, 23), (16, 24), (20, 22), (14, 25)):
        ell(d, [ex - 1, ey - 1, ex + 1, ey + 1], ORANGE)
    d.point((16, 22), fill=GOLD)
    return img


def fake_crafting_tool(rng):
    img, d = canvas()
    # crossed wrench and hammer
    line(d, [(8, 24), (20, 10)], STEEL_D, 3)
    d.arc([17, 4, 25, 12], start=300, end=170, fill=STEEL_D, width=3)
    line(d, [(24, 24), (13, 11)], WOOD_D, 3)
    box(d, [8, 6, 16, 11], STEEL)
    box(d, [8, 6, 16, 8], STEEL_L)
    ell(d, [22, 21, 30, 29], GOLD)                     # '?' badge
    glyph(d, "?", 24, 22, 1, IRON_D)
    return img


def fake_power_tool(rng):
    img, d = canvas()
    box(d, [5, 10, 21, 16], TEAL)                      # drill body
    box(d, [5, 10, 21, 12], TEAL_D)
    poly(d, [(21, 11), (27, 12), (27, 14), (21, 15)], STEEL_D)  # chuck
    line(d, [(27, 13), (29, 13)], STEEL_L, 1)
    box(d, [9, 16, 14, 24], IRON)                      # handle
    box(d, [7, 24, 17, 28], IRON_D)                    # battery pack
    box(d, [9, 25, 15, 27], GLOW_TEAL)
    d.point((15, 17), fill=RED)                        # trigger
    return img


def debug_item_search(rng):
    img, d = canvas()
    box(d, [0, 0, 31, 31], (8, 8, 10, 255))
    d.rectangle([0, 0, 31, 31], outline=RED, width=2)
    glyph(d, "?", 11, 4, 3, (235, 235, 240, 255))
    box(d, [4, 24, 27, 29], RED)
    x = 6
    for ch in "BUG":
        glyph(d, ch, x, 25, 1, (255, 255, 255, 255))
        x += 7
    return img


# ---------------------------------------------------------------- pseudo

def pseudo_app_hd_compressor_unit(rng):
    img, d = canvas()
    box(d, [5, 7, 27, 26], STEEL_L)                    # cabinet
    box(d, [5, 7, 27, 10], STEEL_D)
    ell(d, [9, 11, 23, 25], STEEL_D)                   # fan
    for ang in (0, 60, 120):
        import math
        cx, cy = 16, 18
        for s in (1, -1):
            x2 = cx + int(6 * math.cos(math.radians(ang)) * s)
            y2 = cy + int(6 * math.sin(math.radians(ang)) * s)
            line(d, [(cx, cy), (x2, y2)], IRON, 1)
    ell(d, [14, 16, 18, 20], IRON_D)
    line(d, [(24, 22), (29, 22)], STEEL_D, 2)          # pipe stubs
    line(d, [(24, 25), (29, 25)], STEEL_D, 2)
    return img


def pseudo_app_oxygen_concentrator(rng):
    img, d = canvas()
    rbox(d, [8, 4, 24, 28], (226, 228, 232, 255))
    box(d, [11, 7, 21, 12], IRON_D)                    # display
    box(d, [12, 8, 17, 10], GLOW_TEAL)
    d.point((20, 9), fill=GREEN)
    for y in range(16, 24, 2):                         # vents
        line(d, [(11, y), (21, y)], STEEL_D, 1)
    ell(d, [14, 24, 18, 27], STEEL)                    # tube port
    return img


def pseudo_archery_target_bale(rng):
    img, d = canvas()
    ell(d, [4, 4, 28, 28], (216, 186, 92, 255))        # straw bale
    for r in (9, 6):
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=(186, 152, 66, 255))
    for y in (9, 15, 21, 26):                          # straw banding
        d.arc([4, y - 6, 28, y + 6], start=20, end=160, fill=(196, 164, 74, 255))
    ell(d, [12, 12, 20, 20], CREAM)                    # painted rings
    ell(d, [14, 14, 18, 18], RED)
    d.point((16, 16), fill=GOLD)
    return img


def pseudo_archery_target_box(rng):
    img, d = canvas()
    box(d, [5, 5, 27, 27], BROWN)                      # box frame
    box(d, [8, 8, 24, 24], (196, 170, 128, 255))       # foam face
    ell(d, [11, 11, 21, 21], CREAM)
    ell(d, [13, 13, 19, 19], RED)
    d.point((16, 16), fill=GOLD)
    line(d, [(5, 27), (3, 29)], WOOD_D, 2)             # stand feet
    line(d, [(27, 27), (29, 29)], WOOD_D, 2)
    return img


def pseudo_ergometer_mechanical(rng):
    img, d = canvas()
    ell(d, [4, 13, 18, 27], IRON)                      # flywheel
    ell(d, [6, 15, 16, 25], STEEL_D)
    for pts in (((11, 14), (11, 26)), ((5, 20), (17, 20)), ((7, 16), (15, 24)),
                ((15, 16), (7, 24))):
        line(d, list(pts), IRON_D, 1)
    d.point((11, 20), fill=STEEL_L)
    line(d, [(15, 16), (25, 9)], IRON, 2)              # rail to seat
    box(d, [22, 6, 29, 10], IRON_D)                    # seat
    line(d, [(25, 10), (25, 14)], IRON, 1)
    line(d, [(4, 28), (28, 28)], INK, 2)               # floor line
    return img


def pseudo_exercise_machine(rng):
    img, d = canvas()
    box(d, [6, 4, 10, 28], IRON)                       # upright
    box(d, [6, 4, 25, 7], IRON)                        # top arm
    ell(d, [22, 7, 26, 11], STEEL_L)                   # pulley
    line(d, [(24, 11), (24, 15)], STEEL_L, 1)          # cable
    box(d, [20, 15, 27, 27], IRON_D)                   # weight stack
    for y in range(17, 27, 3):
        line(d, [(20, y), (27, y)], STEEL_D, 1)
    box(d, [10, 17, 19, 20], RED_D)                    # seat
    line(d, [(14, 20), (14, 27)], IRON, 2)
    box(d, [4, 27, 28, 29], IRON_D)                    # base
    line(d, [(24, 15), (27, 13)], STEEL_L, 1)          # handle
    return img


DRAW = {
    "air_compressor_tool": air_compressor_tool,
    "bandsaw_tool": bandsaw_tool,
    "bio_scalpel": bio_scalpel,
    "boulder_anvil": boulder_anvil,
    "catalytic_cracking_reactor_tool": catalytic_cracking_reactor_tool,
    "concrete_mix_tool": concrete_mix_tool,
    "debug_item_search": debug_item_search,
    "drill_press_tool": drill_press_tool,
    "fake_anvil": fake_anvil,
    "fake_anvil_heavy": fake_anvil_heavy,
    "fake_arc_furnace": fake_arc_furnace,
    "fake_beverly_shear": fake_beverly_shear,
    "fake_burrowing": fake_burrowing,
    "fake_crafting_tool": fake_crafting_tool,
    "fake_deep_fryer": fake_deep_fryer,
    "fake_dynamite_50lbs": fake_dynamite_50lbs,
    "fake_firestarter": fake_firestarter,
    "fake_goggles": fake_goggles,
    "fake_lathe": fake_lathe,
    "fake_lift_heavy": fake_lift_heavy,
    "fake_lift_light": fake_lift_light,
    "fake_lift_medium": fake_lift_medium,
    "fake_mill_vertical": fake_mill_vertical,
    "fake_milling_item": fake_milling_item,
    "fake_oil_press_industrial": fake_oil_press_industrial,
    "fake_power_tool": fake_power_tool,
    "fake_radio": fake_radio,
    "fake_razor": fake_razor,
    "fake_sharp_metal_wreckage": fake_sharp_metal_wreckage,
    "fake_smoke_plume": fake_smoke_plume,
    "fake_tire_changer": fake_tire_changer,
    "glassblowers_crucible_tool": glassblowers_crucible_tool,
    "hydraulic_press_tool": hydraulic_press_tool,
    "integrated_ar": integrated_ar,
    "integrated_nl_dimensional_anchor": integrated_nl_dimensional_anchor,
    "integrated_nl_psyshield": integrated_nl_psyshield,
    "integrated_pat_radio": integrated_pat_radio,
    "integrated_pat_transponder": integrated_pat_transponder,
    "integrated_pat_transponder_on": integrated_pat_transponder_on,
    "internal_mp3_music": internal_mp3_music,
    "internal_mp3_radio": internal_mp3_radio,
    "jointer_tool": jointer_tool,
    "mitresaw_tool": mitresaw_tool,
    "oven_tool": oven_tool,
    "planer_tool": planer_tool,
    "power_hammer_tool": power_hammer_tool,
    "pseudo_app_hd_compressor_unit": pseudo_app_hd_compressor_unit,
    "pseudo_app_oxygen_concentrator": pseudo_app_oxygen_concentrator,
    "pseudo_archery_target_bale": pseudo_archery_target_bale,
    "pseudo_archery_target_box": pseudo_archery_target_box,
    "pseudo_ergometer_mechanical": pseudo_ergometer_mechanical,
    "pseudo_exercise_machine": pseudo_exercise_machine,
}


def main():
    assert set(DRAW) == set(IDS), set(IDS) ^ set(DRAW)
    imgs = {}
    for iid in IDS:
        rng = random.Random(zlib.crc32(iid.encode()))
        img = DRAW[iid](rng)
        assert img.size == (32, 32)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (iid, alphas)
        imgs[iid] = img
        folder = os.path.join(BASE, iid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, iid + ".png"))
        with open(os.path.join(folder, iid + ".json"), "w") as f:
            json.dump([{"id": iid, "fg": iid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # contact-sheet preview: 8 cols x 7 rows, 66px cells, sprite x2
    cols, rows, cell = 8, 7, 66
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (34, 36, 42, 255))
    for n, iid in enumerate(IDS):
        x = (n % cols) * cell + 1
        y = (n // cols) * cell + 1
        sheet.alpha_composite(imgs[iid].resize((64, 64), Image.NEAREST), (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item22.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item22.png {sheet.size}")


if __name__ == "__main__":
    main()
