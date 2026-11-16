#!/usr/bin/env python3
"""Batch ITEM13 (Phase 2g.4 scope extension): tool-armor,
final set.

23 ids from data/json/items/tool_armor.json (authoritative
list embedded below; source list:
~/workspace/item-batches/13.txt). Copy-from chains resolved
against the data before drawing (smart_watch_adv <-
smart_watch, thermal_*_on <- their base garments,
welding_papr_helmet_raised <- welding_papr_helmet, ...).
Parameterized family renderers driven by each entry's data:

- Breathing gear: the XS rebreather as a small dark-gray
  mouth mask with an aluminum filter canister (the "on"
  variant adds rising bubbles and a lit indicator); the
  SCUBA regulator as a mouthpiece with rubber hoses looping
  away (the "on" variant is fed air -- bubbles, open valve).
- Injectors: the RX11 stimulant delivery system as a dark
  superalloy body-worn pack with vials and a red activator;
  the wearable RX12 as a blue ceramic jet injector with a
  hazard-striped warning label.
- Smart watches: touchscreen wrist watch on a strap -- app
  grid for the plain state, a music note and sound arcs for
  the music states; the advanced model gets a chunkier
  shock-resistant aluminum case and extra sensor dots.
- KÁ expedition harness (string-dimension phase immersion
  suit): a full body suit of segmented plates coated in
  gold, with collar clasps, tube fittings and the stamped
  electronic control box.
- Thermal electric clothing (light-gray cotton): gloves,
  balaclava, socks, suit and full outfit; the "on" variants
  show orange heating-element coils glowing along the seams.
- Cameras on neck straps: the blue point-and-shoot wearable
  camera, and the professional SLR with prism hump and a
  large zoom lens.
- Welding masks: the crude shaped-steel mask down over the
  face; its raised variant flipped up; the plastic welding
  mask raised; and the PAPR helmet raised -- helmet dome,
  clear full-face shield under the flipped-up welding
  shield, and the respirator hose down the back.

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 23 folders + preview_item13.png.
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
    "rebreather_xs",
    "rebreather_xs_on",
    "rx11_stimpack",
    "scuba_regulator",
    "scuba_regulator_scba_on",
    "smart_watch_adv",
    "smart_watch_adv_music",
    "smart_watch_music",
    "string_dimension_phase_immersion_suit",
    "thermal_gloves",
    "thermal_gloves_on",
    "thermal_mask_on",
    "thermal_outfit",
    "thermal_outfit_on",
    "thermal_socks_on",
    "thermal_suit_on",
    "wearable_camera",
    "wearable_camera_pro",
    "wearable_rx12",
    "welding_mask_crude",
    "welding_mask_crude_raised",
    "welding_mask_raised",
    "welding_papr_helmet_raised",
]

OUTLINE = (28, 28, 34, 255)
DGRAY = (98, 98, 108, 255)        # data color dark_gray
DGRAY_D = (66, 66, 74, 255)
DGRAY_L = (134, 134, 144, 255)
LGRAY = (178, 178, 184, 255)      # data color light_gray
LGRAY_D = (138, 138, 146, 255)
LGRAY_L = (208, 208, 214, 255)
ALU = (172, 178, 188, 255)
ALU_D = (128, 134, 144, 255)
STEEL = (118, 124, 134, 255)
STEEL_D = (84, 90, 100, 255)
STEEL_L = (152, 158, 168, 255)
RUBBER = (42, 42, 48, 255)
VISOR = (16, 18, 26, 255)
GOLD = (228, 188, 62, 255)        # data color yellow (gold coat)
GOLD_D = (176, 138, 38, 255)
GOLD_L = (250, 222, 122, 255)
BLUE = (58, 92, 198, 255)         # data color blue
BLUE_D = (40, 64, 148, 255)
BLUE_L = (104, 138, 226, 255)
GLOW = (255, 132, 38, 255)        # heating-element orange
GLOW_L = (255, 186, 92, 255)
BUBBLE = (196, 228, 240, 255)
LED_ON = (84, 230, 110, 255)
RED = (198, 52, 44, 255)


def canvas():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def speckle(rng, d, box, colors, n):
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=rng.choice(colors))


# ------------------------------------------------------------------
# Breathing gear
# ------------------------------------------------------------------

def rebreather(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    # head strap behind the mask
    d.rectangle([3, 12, 29, 15], fill=RUBBER)
    # mask body (XS: shrunk to fit a smaller face)
    d.rounded_rectangle([8, 9, 24, 26], 6, fill=DGRAY, outline=OUTLINE)
    d.rounded_rectangle([9, 10, 23, 16], 5, fill=DGRAY_L)
    d.rounded_rectangle([9, 20, 23, 25], 5, fill=DGRAY_D)
    # aluminum filter canister, center bottom
    d.ellipse([11, 17, 21, 27], fill=ALU, outline=OUTLINE)
    d.ellipse([13, 19, 19, 25], outline=ALU_D)
    for px, py in ((16, 20), (14, 22), (18, 22), (16, 24)):
        d.point((px, py), fill=DGRAY_D)
    # valve cap on top
    d.rectangle([14, 6, 18, 9], fill=ALU_D, outline=OUTLINE)
    if on:
        d.ellipse([21, 10, 23, 12], fill=LED_ON)
        for bx, by, r in ((26, 7, 1), (28, 4, 2), (24, 3, 1)):
            d.ellipse([bx - r, by - r, bx + r, by + r],
                      fill=BUBBLE, outline=ALU_D)
    return img


def scuba_regulator(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    # hoses looping up and away to the tank (off-frame)
    d.line([(13, 16), (8, 10), (5, 4), (4, 1)], fill=RUBBER, width=3)
    d.line([(19, 16), (25, 11), (28, 6), (29, 2)], fill=RUBBER, width=3)
    d.line([(13, 16), (8, 10), (5, 4), (4, 1)], fill=DGRAY_D, width=1)
    # coiled exhaust hose at the bottom
    d.arc([8, 20, 24, 31], 0, 180, fill=RUBBER, width=3)
    d.arc([11, 22, 21, 30], 0, 180, fill=DGRAY_D, width=1)
    # regulator second stage: round housing
    d.ellipse([10, 13, 22, 25], fill=DGRAY, outline=OUTLINE)
    d.ellipse([12, 15, 20, 23], fill=DGRAY_D)
    d.ellipse([14, 17, 18, 21], fill=ALU, outline=OUTLINE)
    # mouthpiece bar on top of the housing
    d.rounded_rectangle([11, 10, 21, 14], 2, fill=RUBBER, outline=OUTLINE)
    d.rectangle([15, 8, 17, 10], fill=RUBBER)
    if on:
        d.ellipse([19, 14, 21, 16], fill=LED_ON)
        for bx, by, r in ((6, 22, 1), (4, 18, 2), (26, 24, 1), (28, 20, 1)):
            d.ellipse([bx - r, by - r, bx + r, by + r],
                      fill=BUBBLE, outline=ALU_D)
    return img


# ------------------------------------------------------------------
# Injectors
# ------------------------------------------------------------------

def rx11_stimpack(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # kevlar body straps out to the sides
    d.polygon([(2, 13), (9, 15), (9, 19), (2, 20)], fill=(104, 100, 66, 255))
    d.polygon([(30, 13), (23, 15), (23, 19), (30, 20)], fill=(104, 100, 66, 255))
    # superalloy pack body
    d.rounded_rectangle([8, 6, 24, 27], 3, fill=(92, 97, 107, 255),
                        outline=OUTLINE)
    d.rounded_rectangle([9, 7, 23, 11], 2, fill=(122, 128, 140, 255))
    # red activator button on top
    d.ellipse([13, 3, 19, 8], fill=RED, outline=OUTLINE)
    d.point((15, 5), fill=(240, 140, 130, 255))
    # two stimulant vials in their slots
    for x in (11, 18):
        d.rounded_rectangle([x, 14, x + 3, 24], 1, fill=(38, 40, 48, 255))
        d.rectangle([x + 1, 16, x + 2, 22], fill=(232, 168, 60, 255))
    # injector nozzle plate at the base (needle-free jet)
    d.rectangle([12, 25, 20, 27], fill=ALU_D, outline=OUTLINE)
    speckle(rng, d, (9, 12, 23, 24), [DGRAY_D, (122, 128, 140, 255)], 6)
    return img


def wearable_rx12(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # neck/body strap loop
    d.arc([9, 0, 23, 12], 180, 360, fill=BLUE_D, width=2)
    # blue ceramic injector body
    d.rounded_rectangle([10, 7, 22, 24], 3, fill=BLUE, outline=OUTLINE)
    d.rounded_rectangle([11, 8, 16, 23], 2, fill=BLUE_L)
    # dose window with two dose marks (label: max two per hour)
    d.rectangle([17, 10, 21, 17], fill=(225, 232, 240, 255), outline=OUTLINE)
    d.rectangle([18, 12, 20, 13], fill=BLUE_D)
    d.rectangle([18, 15, 20, 16], fill=BLUE_D)
    # hazard warning stripe
    for i in range(3):
        d.polygon([(11 + i * 4, 21), (13 + i * 4, 21), (12 + i * 4, 23),
                   (10 + i * 4, 23)], fill=(240, 200, 50, 255))
    # jet nozzle at the base
    d.polygon([(13, 24), (19, 24), (17, 28), (15, 28)], fill=ALU,
              outline=OUTLINE)
    return img


# ------------------------------------------------------------------
# Smart watches
# ------------------------------------------------------------------

def _music_note(d, cx, cy, color):
    d.ellipse([cx - 3, cy + 2, cx, cy + 5], fill=color)
    d.rectangle([cx, cy - 4, cx + 1, cy + 3], fill=color)
    d.polygon([(cx, cy - 4), (cx + 4, cy - 3), (cx + 4, cy - 1),
               (cx, cy - 2)], fill=color)
    d.ellipse([cx + 3, cy + 1, cx + 6, cy + 4], fill=color)
    d.rectangle([cx + 5, cy - 3, cx + 6, cy + 2], fill=color)


def smart_watch(rng, adv, music):
    img = canvas()
    d = ImageDraw.Draw(img)
    # strap
    d.rectangle([13, 1, 19, 31], fill=(64, 68, 84, 255))
    d.rectangle([14, 1, 18, 31], fill=(80, 85, 102, 255))
    for y in (4, 27):
        d.ellipse([15, y, 17, y + 2], fill=RUBBER)
    # case: advanced model is chunkier shock-resistant aluminum
    if adv:
        d.rounded_rectangle([8, 8, 24, 24], 4, fill=ALU, outline=OUTLINE)
        for cx, cy in ((10, 10), (22, 10), (10, 22), (22, 22)):
            d.point((cx, cy), fill=ALU_D)
        d.rectangle([24, 14, 26, 18], fill=ALU_D, outline=OUTLINE)
    else:
        d.rounded_rectangle([9, 9, 23, 23], 4, fill=LGRAY, outline=OUTLINE)
        d.rectangle([23, 14, 25, 17], fill=LGRAY_D, outline=OUTLINE)
    # touchscreen
    d.rounded_rectangle([11, 11, 21, 21], 2, fill=(22, 32, 56, 255))
    if music:
        _music_note(d, 14, 13, (96, 220, 235, 255))
        for r in (4, 7):
            d.arc([22 - r // 2, 12 - r // 2, 22 + r, 12 + r], -60, 60,
                  fill=(96, 220, 235, 255))
    else:
        cols = [(235, 120, 90, 255), (110, 220, 140, 255), (110, 160, 240, 255),
                (240, 210, 90, 255), (200, 130, 235, 255), (120, 220, 220, 255),
                (240, 150, 170, 255), (170, 200, 110, 255), (150, 170, 200, 255)]
        for i, c in enumerate(cols):
            x = 12 + (i % 3) * 3
            y = 12 + (i // 3) * 3
            d.rectangle([x, y, x + 1, y + 1], fill=c)
    if adv:
        # extra sensor dots under the case
        for x in (13, 16, 19):
            d.ellipse([x, 26, x + 1, 27], fill=(90, 200, 120, 255))
    return img


# ------------------------------------------------------------------
# KÁ expedition harness (phase immersion suit)
# ------------------------------------------------------------------

def phase_suit(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # hood/head
    d.ellipse([12, 1, 20, 8], fill=GOLD, outline=OUTLINE)
    d.rectangle([13, 5, 19, 7], fill=VISOR)
    # collar bar with clasps and tube fittings
    d.rectangle([10, 8, 22, 11], fill=GOLD_D, outline=OUTLINE)
    for x in (11, 20):
        d.ellipse([x, 8, x + 2, 10], fill=ALU, outline=OUTLINE)
    d.line([(12, 11), (9, 16)], fill=ALU_D, width=1)
    d.line([(20, 11), (23, 16)], fill=ALU_D, width=1)
    # torso of segmented gold plates
    d.polygon([(10, 11), (22, 11), (24, 24), (8, 24)], fill=GOLD,
              outline=OUTLINE)
    for y in (15, 19):
        d.line([(9, y), (23, y)], fill=GOLD_D)
    d.line([(16, 11), (16, 24)], fill=GOLD_D)
    # electronic control box stamped "KA" (abstracted glyph marks)
    d.rectangle([13, 16, 19, 21], fill=(38, 40, 48, 255), outline=GOLD_L)
    d.line([(14, 18), (16, 17), (18, 18)], fill=GOLD_L)
    d.point((16, 19), fill=GOLD_L)
    # harness straps with spring clips
    d.line([(11, 12), (21, 23)], fill=GOLD_D, width=2)
    d.rectangle([18, 20, 21, 23], fill=ALU, outline=OUTLINE)
    # arms
    d.polygon([(8, 12), (10, 12), (9, 24), (6, 24)], fill=GOLD,
              outline=OUTLINE)
    d.polygon([(24, 12), (22, 12), (23, 24), (26, 24)], fill=GOLD,
              outline=OUTLINE)
    d.rectangle([5, 24, 9, 27], fill=GOLD_D, outline=OUTLINE)
    d.rectangle([23, 24, 27, 27], fill=GOLD_D, outline=OUTLINE)
    # legs and feet
    d.polygon([(9, 24), (15, 24), (14, 30), (10, 30)], fill=GOLD,
              outline=OUTLINE)
    d.polygon([(17, 24), (23, 24), (22, 30), (18, 30)], fill=GOLD,
              outline=OUTLINE)
    d.rectangle([8, 30, 14, 31], fill=GOLD_D)
    d.rectangle([18, 30, 24, 31], fill=GOLD_D)
    speckle(rng, d, (9, 12, 23, 29), [GOLD_L], 8)
    return img


# ------------------------------------------------------------------
# Thermal electric clothing
# ------------------------------------------------------------------

def _heat(d, pts):
    d.line(pts, fill=GLOW, width=1)
    d.line([(x, y - 1) for x, y in pts[::2]], fill=GLOW_L, width=1)


def thermal_gloves(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    for x0, flip in ((5, 0), (18, 1)):
        # glove body: palm + fingers + thumb
        d.rounded_rectangle([x0, 8, x0 + 9, 22], 3, fill=LGRAY,
                            outline=OUTLINE)
        for fx in range(3):
            d.rounded_rectangle([x0 + 1 + fx * 3, 3, x0 + 3 + fx * 3, 9], 1,
                                fill=LGRAY, outline=OUTLINE)
        tx = x0 + 9 if not flip else x0 - 2
        d.rounded_rectangle([tx, 13, tx + 2, 18], 1, fill=LGRAY,
                            outline=OUTLINE)
        # ribbed cuff + battery pack
        d.rectangle([x0, 22, x0 + 9, 26], fill=LGRAY_D, outline=OUTLINE)
        d.rectangle([x0 + 3, 23, x0 + 6, 25], fill=(52, 54, 62, 255))
        if on:
            _heat(d, [(x0 + 2, 18), (x0 + 4, 12), (x0 + 6, 18),
                      (x0 + 7, 12)])
            d.point((x0 + 4, 24), fill=LED_ON)
    return img


def thermal_mask_on(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # balaclava: snug head cover
    d.ellipse([8, 2, 24, 20], fill=LGRAY, outline=OUTLINE)
    d.rectangle([8, 14, 24, 24], fill=LGRAY)
    d.line([(8, 18), (8, 24)], fill=OUTLINE)
    d.line([(24, 18), (24, 24)], fill=OUTLINE)
    d.line([(8, 24), (24, 24)], fill=OUTLINE)
    # eye opening and mouth opening
    d.rounded_rectangle([11, 9, 21, 13], 2, fill=(40, 34, 30, 255),
                        outline=OUTLINE)
    d.ellipse([13, 10, 15, 12], fill=(24, 24, 30, 255))
    d.ellipse([17, 10, 19, 12], fill=(24, 24, 30, 255))
    d.rounded_rectangle([13, 16, 19, 19], 1, fill=(40, 34, 30, 255),
                        outline=OUTLINE)
    # heating coils glow around the face opening (it is on)
    _heat(d, [(10, 6), (16, 4), (22, 6)])
    _heat(d, [(10, 21), (16, 23), (22, 21)])
    d.rectangle([14, 24, 18, 27], fill=(52, 54, 62, 255), outline=OUTLINE)
    d.point((16, 25), fill=LED_ON)
    return img


def thermal_socks_on(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    for x0 in (6, 18):
        # sock: leg tube, heel, foot
        d.polygon([(x0, 3), (x0 + 7, 3), (x0 + 7, 16), (x0 + 12, 21),
                   (x0 + 12, 26), (x0 + 1, 26), (x0, 21)],
                  fill=LGRAY, outline=OUTLINE)
        d.rectangle([x0, 3, x0 + 7, 6], fill=LGRAY_D)
        _heat(d, [(x0 + 2, 9), (x0 + 5, 12), (x0 + 2, 15), (x0 + 5, 18)])
        d.point((x0 + 3, 4), fill=LED_ON)
    return img


def _thermal_body(rng, on, full):
    img = canvas()
    d = ImageDraw.Draw(img)
    if full:
        # hood for the head-to-toe outfit
        d.ellipse([12, 1, 20, 7], fill=LGRAY, outline=OUTLINE)
        d.rectangle([13, 4, 19, 6], fill=(40, 34, 30, 255))
        top = 7
    else:
        top = 4
        d.rectangle([13, 1, 19, 4], fill=LGRAY_D, outline=OUTLINE)
    # torso
    d.polygon([(10, top), (22, top), (23, 18), (9, 18)], fill=LGRAY,
              outline=OUTLINE)
    # arms
    d.polygon([(10, top + 1), (7, top + 3), (6, 17), (9, 17)], fill=LGRAY,
              outline=OUTLINE)
    d.polygon([(22, top + 1), (25, top + 3), (26, 17), (23, 17)], fill=LGRAY,
              outline=OUTLINE)
    # legs (outfit covers the feet too: closed bootie ends)
    d.polygon([(10, 18), (15, 18), (14, 29), (10, 29)], fill=LGRAY,
              outline=OUTLINE)
    d.polygon([(17, 18), (22, 18), (22, 29), (18, 29)], fill=LGRAY,
              outline=OUTLINE)
    if full:
        d.rectangle([9, 29, 15, 31], fill=LGRAY, outline=OUTLINE)
        d.rectangle([17, 29, 23, 31], fill=LGRAY, outline=OUTLINE)
    # waist battery pack
    d.rectangle([14, 15, 18, 18], fill=(52, 54, 62, 255), outline=OUTLINE)
    if on:
        _heat(d, [(12, top + 3), (16, top + 6), (20, top + 3)])
        _heat(d, [(8, top + 5), (7, top + 10), (8, top + 14)])
        _heat(d, [(24, top + 5), (25, top + 10), (24, top + 14)])
        _heat(d, [(12, 21), (11, 26)])
        _heat(d, [(20, 21), (21, 26)])
        d.point((16, 16), fill=LED_ON)
    else:
        for y in (top + 4, top + 8):
            d.line([(11, y), (21, y)], fill=LGRAY_D)
    return img


# ------------------------------------------------------------------
# Cameras
# ------------------------------------------------------------------

def _strap(d):
    d.line([(8, 13), (5, 2)], fill=RUBBER, width=2)
    d.line([(24, 13), (27, 2)], fill=RUBBER, width=2)


def wearable_camera(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    _strap(d)
    # blue point-and-shoot body
    d.rounded_rectangle([7, 12, 25, 25], 2, fill=BLUE, outline=OUTLINE)
    d.rectangle([7, 12, 25, 15], fill=BLUE_D)
    # lens
    d.ellipse([12, 15, 20, 23], fill=ALU_D, outline=OUTLINE)
    d.ellipse([13, 16, 19, 22], fill=(30, 44, 74, 255))
    d.ellipse([14, 17, 17, 20], fill=(96, 150, 220, 255))
    d.point((15, 17), fill=(230, 242, 252, 255))
    # flash + viewfinder + screen edge
    d.rectangle([21, 16, 24, 19], fill=(240, 240, 210, 255), outline=OUTLINE)
    d.rectangle([8, 16, 11, 19], fill=(24, 28, 40, 255), outline=OUTLINE)
    d.rectangle([9, 21, 11, 22], fill=RED)
    return img


def wearable_camera_pro(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    _strap(d)
    # SLR body with prism hump and grip
    d.rounded_rectangle([4, 13, 28, 26], 2, fill=(40, 40, 47, 255),
                        outline=OUTLINE)
    d.polygon([(12, 13), (14, 8), (19, 8), (21, 13)], fill=(40, 40, 47, 255),
              outline=OUTLINE)
    d.rectangle([25, 15, 28, 26], fill=(30, 30, 36, 255))
    for y in (18, 21, 24):
        d.line([(26, y), (27, y)], fill=DGRAY_D)
    # big zoom lens: concentric barrels
    d.ellipse([9, 12, 23, 27], fill=(24, 24, 30, 255), outline=ALU_D)
    d.ellipse([11, 14, 21, 25], fill=(52, 52, 60, 255))
    d.ellipse([12, 15, 20, 24], fill=(26, 40, 70, 255))
    d.ellipse([14, 17, 18, 21], fill=(92, 144, 214, 255))
    d.point((15, 17), fill=(235, 244, 252, 255))
    # focus ring ticks
    for a in range(0, 360, 45):
        import math
        x = 16 + int(6.5 * math.cos(math.radians(a)))
        y = 19 + int(6.5 * math.sin(math.radians(a)))
        d.point((x, y), fill=ALU_D)
    # flash + viewfinder window
    d.rectangle([5, 14, 8, 17], fill=(240, 240, 210, 255), outline=OUTLINE)
    d.rectangle([22, 9, 25, 12], fill=(24, 28, 40, 255), outline=OUTLINE)
    return img


# ------------------------------------------------------------------
# Welding masks
# ------------------------------------------------------------------

def welding_crude(rng, raised):
    img = canvas()
    d = ImageDraw.Draw(img)
    if not raised:
        # shaped-steel plate down over the face
        d.polygon([(9, 5), (23, 5), (25, 12), (24, 27), (8, 27), (7, 12)],
                  fill=STEEL, outline=OUTLINE)
        d.polygon([(10, 6), (22, 6), (23, 12), (10, 12)], fill=STEEL_L)
        # very dark viewing slit
        d.rectangle([11, 13, 21, 17], fill=VISOR, outline=OUTLINE)
        d.line([(12, 14), (20, 14)], fill=(70, 90, 120, 255))
        # hammered dents and scratches of the crude shaping
        speckle(rng, d, (9, 7, 23, 26), [STEEL_D, STEEL_L], 22)
        for x in (10, 22):
            d.point((x, 24), fill=STEEL_D)
        return img
    # raised: plate flipped up above the forehead
    d.polygon([(8, 1), (24, 1), (26, 9), (6, 9)], fill=STEEL, outline=OUTLINE)
    d.rectangle([11, 3, 21, 6], fill=VISOR, outline=OUTLINE)
    speckle(rng, d, (7, 1, 25, 8), [STEEL_D, STEEL_L], 12)
    # head strap and the uncovered face below (shadowed)
    d.rectangle([8, 10, 24, 12], fill=RUBBER)
    d.ellipse([10, 12, 22, 26], fill=(56, 48, 44, 255), outline=OUTLINE)
    d.line([(12, 19), (20, 19)], fill=(36, 32, 30, 255))
    return img


def welding_mask_raised(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # plastic mask shell flipped up on top of the head
    d.polygon([(7, 2), (25, 2), (27, 11), (5, 11)], fill=DGRAY,
              outline=OUTLINE)
    d.rectangle([10, 4, 22, 8], fill=VISOR, outline=OUTLINE)
    d.line([(11, 5), (21, 5)], fill=(70, 90, 120, 255))
    d.rectangle([5, 11, 27, 13], fill=DGRAY_D)
    # head strap + uncovered face below
    d.rectangle([8, 13, 24, 15], fill=RUBBER)
    d.ellipse([10, 15, 22, 29], fill=(56, 48, 44, 255), outline=OUTLINE)
    d.line([(12, 22), (20, 22)], fill=(36, 32, 30, 255))
    return img


def welding_papr_raised(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # respirator hose from the back of the helmet, down the side
    d.line([(24, 10), (29, 16), (28, 26), (29, 31)], fill=RUBBER, width=3)
    for y in (14, 18, 22, 26):
        d.line([(27, y), (30, y)], fill=DGRAY_D)
    # flipped-up welding shield
    d.polygon([(8, 1), (24, 1), (26, 8), (6, 8)], fill=DGRAY, outline=OUTLINE)
    d.rectangle([11, 2, 21, 5], fill=VISOR, outline=OUTLINE)
    # helmet dome
    d.pieslice([7, 5, 25, 19], 180, 360, fill=DGRAY, outline=OUTLINE)
    d.rectangle([6, 12, 26, 14], fill=DGRAY_D, outline=OUTLINE)
    # transparent full-face shield under the raised mask
    d.rectangle([9, 14, 23, 28], fill=(188, 214, 224, 255), outline=OUTLINE)
    d.line([(11, 16), (15, 26)], fill=(226, 240, 244, 255), width=2)
    # respirator mouthpiece behind the shield
    d.rounded_rectangle([13, 21, 19, 25], 2, fill=DGRAY_D, outline=OUTLINE)
    d.point((16, 23), fill=RUBBER)
    return img


# ------------------------------------------------------------------

RENDER = {
    "rebreather_xs": lambda rng: rebreather(rng, False),
    "rebreather_xs_on": lambda rng: rebreather(rng, True),
    "rx11_stimpack": rx11_stimpack,
    "scuba_regulator": lambda rng: scuba_regulator(rng, False),
    "scuba_regulator_scba_on": lambda rng: scuba_regulator(rng, True),
    "smart_watch_adv": lambda rng: smart_watch(rng, True, False),
    "smart_watch_adv_music": lambda rng: smart_watch(rng, True, True),
    "smart_watch_music": lambda rng: smart_watch(rng, False, True),
    "string_dimension_phase_immersion_suit": phase_suit,
    "thermal_gloves": lambda rng: thermal_gloves(rng, False),
    "thermal_gloves_on": lambda rng: thermal_gloves(rng, True),
    "thermal_mask_on": thermal_mask_on,
    "thermal_outfit": lambda rng: _thermal_body(rng, False, True),
    "thermal_outfit_on": lambda rng: _thermal_body(rng, True, True),
    "thermal_socks_on": thermal_socks_on,
    "thermal_suit_on": lambda rng: _thermal_body(rng, True, False),
    "wearable_camera": wearable_camera,
    "wearable_camera_pro": wearable_camera_pro,
    "wearable_rx12": wearable_rx12,
    "welding_mask_crude": lambda rng: welding_crude(rng, False),
    "welding_mask_crude_raised": lambda rng: welding_crude(rng, True),
    "welding_mask_raised": welding_mask_raised,
    "welding_papr_helmet_raised": welding_papr_raised,
}


def main():
    assert set(RENDER) == set(IDS), set(IDS) ^ set(RENDER)
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = RENDER[item_id](rng)
        assert img.size == (32, 32), item_id
        assert set(img.getchannel("A").getdata()) <= {0, 255}, item_id
        imgs[item_id] = img
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f,
                      indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols, cell = 6, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item13.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item13.png {sheet.size}")


if __name__ == "__main__":
    main()
