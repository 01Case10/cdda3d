#!/usr/bin/env python3
"""Batch ITEM39 (Phase 2g.4 scope extension): alien
resources.

35 ids from data/json/items/resources/alien.json
(authoritative list embedded below; source list:
~/workspace/item-batches/39.txt). No copy-from chains in
this file -- every entry is drawn from its own
name/description/color/material.

The set, from the entries:

- resin: resin_chunk is "a large piece of sea glass,
  frosted and gritty, edges rounded, somewhat warm" --
  a pale frosted shard; alien_pod_resin is a keg-sized
  light-blue pod with a broken stalk that squirts gooey
  resin.
- cyborg_matrix: tanks and tubes housing a brain/spine
  life-support rig -- vertical glass tanks, fluid lines,
  ports.
- ephemeral_gem: a black gem you perceive as a pure
  void, surfaces "beautifully etched" -- a faceted void
  shape with etched facet lines.
- Exodii ingots (aluminum, lc_steel, copper, lead,
  plastic, tin, zinc): "enormous" 800 L blocks -- one
  tile-filling trapezoid ingot per metal in its data
  color; copper carries its blue-green patina.
- Exodii chassis / drone chassis: roughly hexagonal
  one-piece qt_steel frames with fitting holes and wear;
  the drone's is smaller.
- Exodii computers: heavy dark unfamiliar electronic
  boxes (10 L and palm-sized micro) -- skewed housings,
  alien ports and glyph lights.
- exodii_module / micro_module: concentric coppery
  rings / nuts wired to a central axis.
- exodii_motor: copper coils and a protruding piston in
  a blue housing. exodii_portalizer: a perfectly smooth
  dull blue-gray oblong, riverstone-like, with intricate
  patterning visible in its reflection. exodii_sensor:
  a dinner-plate glassy eye. exodii_scanner_dish: an
  engraved parabolic dish. exodii_wire_kit /
  exodii_ac_kit: soldered wires, plugs, pipes and a
  central micro controller.
- Nether huntsman set: bone-like material --
  armorpiece_big is a coral-perforated plate with
  barbed hooks on the inner side, armorpiece_small a
  crumbling fragment, spike a flexible tapered rod,
  statuette a small fantastical creature carved from a
  rough unnatural dark material.
- string_dimension_computer: an extremely heavy
  rounded cube of dense metal, swing door open on
  neatly packaged unknown electronics.
- Yrax set (gold / ochre / white): computer is a heavy
  gold sphere etched with a complex pattern;
  electronics are woven ochre metallic strips;
  metal is a slightly curved white plate that collects
  static; robotics are clockwork-like tight plates and
  gears; scrap is tiny strips, spheres and geometric
  primitives; sensor is a carved white sphere studded
  with small black glass screens; trinket is a tiny
  golden plate etched with detailed artwork.
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "alien_pod_resin",
    "cyborg_matrix",
    "ephemeral_gem",
    "exodii_ac_kit",
    "exodii_chassis",
    "exodii_computer",
    "exodii_drone_chassis",
    "exodii_ingot_aluminum",
    "exodii_ingot_copper",
    "exodii_ingot_lc_steel",
    "exodii_ingot_lead",
    "exodii_ingot_plastic",
    "exodii_ingot_tin",
    "exodii_ingot_zinc",
    "exodii_micro_computer",
    "exodii_micro_module",
    "exodii_module",
    "exodii_motor",
    "exodii_portalizer",
    "exodii_scanner_dish",
    "exodii_sensor",
    "exodii_wire_kit",
    "nether_huntsman_armorpiece_big",
    "nether_huntsman_armorpiece_small",
    "nether_huntsman_spike",
    "nether_huntsman_statuette",
    "resin_chunk",
    "string_dimension_computer",
    "yrax_computer",
    "yrax_electronics",
    "yrax_metal",
    "yrax_robotics",
    "yrax_scrap",
    "yrax_sensor",
    "yrax_trinket",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.")


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- resin

def draw_resin_chunk(d, rng):
    # frosted sea-glass shard, rounded edges, gritty speckle, warm tint
    body = (196, 200, 196, 255)
    pts = [(9, 8), (20, 5), (26, 12), (24, 23), (14, 27), (7, 19)]
    d.polygon(pts, fill=body)
    d.polygon([(9, 8), (20, 5), (19, 14), (11, 15)], fill=shade(body, 1.1))
    d.polygon([(11, 15), (19, 14), (24, 23), (14, 27)], fill=shade(body, 0.92))
    d.line(pts + [pts[0]], fill=shade(body, 0.75), width=1)
    for _ in range(46):  # grit
        x = rng.randint(8, 25)
        y = rng.randint(6, 26)
        d.point((x, y), fill=shade(body, rng.choice([0.8, 1.12])))
    for _ in range(7):  # warm amber flecks
        x = rng.randint(10, 23)
        y = rng.randint(8, 24)
        d.point((x, y), fill=(216, 178, 120, 255))


def draw_alien_pod_resin(d, rng):
    pod = (128, 178, 224, 255)  # light_blue
    d.ellipse([6, 8, 26, 29], fill=pod)
    d.ellipse([9, 10, 19, 20], fill=shade(pod, 1.13))
    d.ellipse([8, 21, 24, 28], fill=shade(pod, 0.88))
    # surface mottling
    for _ in range(26):
        x = rng.randint(8, 24)
        y = rng.randint(10, 27)
        d.point((x, y), fill=shade(pod, rng.choice([0.9, 1.08])))
    # broken stalk at top, resin squirting out
    d.line([(15, 9), (17, 4)], fill=shade(pod, 0.8), width=3)
    d.line([(17, 4), (19, 3)], fill=shade(pod, 0.7), width=2)
    goo = (168, 208, 240, 255)
    d.line([(19, 3), (23, 2)], fill=goo, width=1)
    d.ellipse([23, 1, 26, 4], fill=goo)
    d.ellipse([20, 5, 22, 7], fill=goo)


# ---------------------------------------------------------------- matrix / gem

def draw_cyborg_matrix(d, rng):
    frame = (168, 172, 178, 255)
    glass = (150, 196, 232, 255)
    fluid = (96, 160, 214, 255)
    d.rectangle([3, 5, 29, 28], fill=shade(frame, 0.9))
    d.rectangle([3, 5, 29, 7], fill=frame)
    d.rectangle([3, 26, 29, 28], fill=frame)
    # two life-support tanks with fluid and a pale spine-like form
    for x0 in (6, 18):
        d.rectangle([x0, 9, x0 + 8, 25], fill=glass)
        d.rectangle([x0 + 1, 13, x0 + 7, 24], fill=fluid)
        d.line([(x0 + 4, 14), (x0 + 4, 23)], fill=(226, 232, 222, 255), width=2)
        for yy in (16, 19, 22):
            d.point((x0 + 3, yy), fill=(240, 244, 238, 255))
            d.point((x0 + 5, yy), fill=(240, 244, 238, 255))
    # tubes across the top, ports along the bottom
    d.line([(8, 8), (8, 4), (24, 4), (24, 8)], fill=(120, 128, 138, 255), width=1)
    d.line([(10, 26), (10, 30)], fill=(90, 96, 104, 255), width=2)
    d.line([(16, 26), (16, 30)], fill=(90, 96, 104, 255), width=2)
    d.line([(22, 26), (22, 30)], fill=(90, 96, 104, 255), width=2)
    d.point((16, 6), fill=(240, 120, 90, 255))


def draw_ephemeral_gem(d, rng):
    # a void perceived as a faceted black gem; etched facet lines
    void = (6, 6, 10, 255)
    etch = (86, 90, 104, 255)
    top = [(9, 7), (23, 7), (27, 13), (16, 27), (5, 13)]
    d.polygon(top, fill=void)
    d.polygon([(9, 7), (23, 7), (21, 13), (11, 13)], fill=(14, 14, 20, 255))
    d.polygon([(11, 13), (21, 13), (16, 27)], fill=(10, 10, 15, 255))
    for a, b in [((9, 7), (11, 13)), ((23, 7), (21, 13)),
                 ((5, 13), (11, 13)), ((27, 13), (21, 13)),
                 ((11, 13), (16, 27)), ((21, 13), (16, 27)),
                 ((5, 13), (16, 27)), ((27, 13), (16, 27))]:
        d.line([a, b], fill=etch, width=1)
    d.line(top + [top[0]], fill=(120, 126, 142, 255), width=1)
    # fine etched tracery on the table facet
    d.line([(12, 9), (20, 9)], fill=etch, width=1)
    d.line([(13, 11), (19, 11)], fill=etch, width=1)
    d.point((16, 10), fill=(150, 156, 172, 255))


# ---------------------------------------------------------------- ingots

INGOTS = {
    # id: (base color, patina or None)
    "exodii_ingot_aluminum": ((198, 202, 208), None),
    "exodii_ingot_lc_steel": ((172, 178, 188), None),
    "exodii_ingot_copper": ((172, 106, 66), (96, 168, 148)),
    "exodii_ingot_lead": ((104, 106, 114), None),
    "exodii_ingot_plastic": ((232, 232, 230), None),
    "exodii_ingot_tin": ((216, 218, 220), None),
    "exodii_ingot_zinc": ((224, 230, 234), None),
}


def draw_ingot(d, rng, item_id):
    base, patina = INGOTS[item_id]
    base = base + (255,)
    # enormous block: trapezoid, top face + front face
    d.polygon([(7, 9), (25, 9), (29, 15), (3, 15)], fill=shade(base, 1.12))
    d.polygon([(3, 15), (29, 15), (29, 26), (3, 26)], fill=base)
    d.polygon([(3, 22), (29, 22), (29, 26), (3, 26)], fill=shade(base, 0.86))
    d.line([(3, 15), (29, 15)], fill=shade(base, 0.7), width=1)
    # stamped alien glyph marks on the front
    gx = rng.randint(7, 10)
    for k in range(3):
        x = gx + k * 6
        d.line([(x, 18), (x + 3, 18)], fill=shade(base, 0.72), width=1)
        d.line([(x + 1, 18), (x + 1, 20)], fill=shade(base, 0.72), width=1)
    for _ in range(10):  # wear
        x = rng.randint(4, 28)
        y = rng.randint(16, 25)
        d.point((x, y), fill=shade(base, rng.choice([0.9, 1.06])))
    if patina:
        patina = patina + (255,)
        for _ in range(16):
            x = rng.randint(4, 27)
            y = rng.randint(10, 25)
            d.ellipse([x, y, x + rng.randint(1, 3), y + rng.randint(1, 2)],
                      fill=patina)


# ---------------------------------------------------------------- exodii frames / devices

def hex_frame(d, cx, cy, r, wall, inner_dark=True):
    pts = [(cx + r * math.cos(math.radians(60 * k)),
            cy + r * math.sin(math.radians(60 * k))) for k in range(6)]
    pts = [(int(x), int(y)) for x, y in pts]
    d.polygon(pts, fill=wall)
    r2 = r - 4
    pts2 = [(int(cx + r2 * math.cos(math.radians(60 * k))),
             int(cy + r2 * math.sin(math.radians(60 * k)))) for k in range(6)]
    if inner_dark:
        d.polygon(pts2, fill=(38, 40, 46, 255))
    return pts


def draw_chassis(d, rng, drone=False):
    qt = (178, 182, 190, 255)
    r = 9 if drone else 13
    cx, cy = 16, 16
    pts = hex_frame(d, cx, cy, r, qt)
    # inner bodywork panel
    r2 = r - 5
    pts2 = [(int(cx + r2 * math.cos(math.radians(60 * k))),
             int(cy + r2 * math.sin(math.radians(60 * k)))) for k in range(6)]
    d.polygon(pts2, fill=shade(qt, 0.82))
    d.polygon([(cx - 2, cy - 2), (cx + 2, cy - 2), (cx + 2, cy + 2),
               (cx - 2, cy + 2)], fill=shade(qt, 0.66))
    # fitting holes at the vertices
    for (x, y) in pts:
        hx = int(cx + (x - cx) * 0.82)
        hy = int(cy + (y - cy) * 0.82)
        d.ellipse([hx - 1, hy - 1, hx + 1, hy + 1], fill=(30, 32, 38, 255))
    # wear scratches and one repair weld
    for _ in range(6):
        x = rng.randint(cx - r + 2, cx + r - 4)
        y = rng.randint(cy - r + 2, cy + r - 2)
        d.line([(x, y), (x + rng.randint(1, 3), y)], fill=shade(qt, 1.1), width=1)
    d.line([(cx - r + 3, cy + r - 4), (cx - r + 7, cy + r - 6)],
           fill=(140, 144, 152, 255), width=1)
    if drone:
        d.point((cx, cy - r + 2), fill=(110, 160, 220, 255))


def draw_computer(d, rng, micro=False):
    body = (92, 94, 102, 255)
    if micro:
        box = [9, 10, 23, 23]
    else:
        box = [5, 7, 27, 26]
    x0, y0, x1, y1 = box
    d.polygon([(x0 + 2, y0), (x1, y0 + 2), (x1, y1 - 2), (x0, y1)],
              fill=body)
    d.polygon([(x0 + 2, y0), (x1, y0 + 2), (x1, y0 + 5), (x0, y0 + 4)],
              fill=shade(body, 1.18))
    # unfamiliar port cluster: no right angles where you expect them
    d.ellipse([x0 + 3, y0 + 6, x0 + 7, y0 + 10], fill=(30, 32, 38, 255))
    d.polygon([(x1 - 8, y0 + 6), (x1 - 4, y0 + 6), (x1 - 6, y0 + 10)],
              fill=(30, 32, 38, 255))
    # glyph lights
    for k in range(3):
        d.point((x0 + 4 + k * 3, y1 - 4), fill=(110, 200, 230, 255))
    d.line([(x0 + 3, y1 - 7), (x1 - 4, y1 - 7)], fill=shade(body, 0.7), width=1)
    # wiring stub
    d.line([(x1, y0 + 8), (x1 + (2 if micro else 3), y0 + 11)],
           fill=(150, 100, 60, 255), width=1)


def draw_rings(d, rng, nuts=False):
    copper = (176, 116, 70, 255)
    cx, cy = 16, 16
    radii = [11, 8, 5] if not nuts else [10, 7, 4]
    for i, r in enumerate(radii):
        col = shade(copper, 1.0 - i * 0.12)
        if nuts:
            pts = [(int(cx + r * math.cos(math.radians(60 * k + 30))),
                    int(cy + r * math.sin(math.radians(60 * k + 30)) * 0.82))
                   for k in range(6)]
            d.polygon(pts, outline=col, width=2)
        else:
            d.ellipse([cx - r, cy - int(r * 0.82), cx + r, cy + int(r * 0.82)],
                      outline=col, width=2)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(210, 190, 150, 255))
    # wires from each ring to the axis
    for ang in (200, 320, 90):
        x = int(cx + 11 * math.cos(math.radians(ang)))
        y = int(cy + 9 * math.sin(math.radians(ang)))
        d.line([(x, y), (cx, cy)], fill=(120, 80, 46, 255), width=1)


def draw_motor(d, rng):
    housing = (84, 110, 178, 255)  # blue qt_steel
    copper = (178, 116, 70, 255)
    d.rectangle([8, 9, 22, 24], fill=housing)
    d.rectangle([8, 9, 22, 12], fill=shade(housing, 1.2))
    # exposed copper coils
    for x in (11, 15, 19):
        d.rectangle([x, 14, x + 2, 22], fill=copper)
        d.line([(x, 14), (x, 22)], fill=shade(copper, 1.25), width=1)
    # protruding piston
    d.rectangle([14, 4, 17, 9], fill=(170, 174, 182, 255))
    d.rectangle([13, 3, 18, 5], fill=(140, 144, 152, 255))
    d.rectangle([10, 24, 20, 26], fill=shade(housing, 0.8))


def draw_portalizer(d, rng):
    stone = (110, 124, 142, 255)  # dull blue-gray
    d.ellipse([5, 10, 27, 23], fill=stone)
    d.ellipse([8, 12, 20, 18], fill=shade(stone, 1.14))
    d.ellipse([6, 18, 26, 22], fill=shade(stone, 0.92))
    # intricate pattern visible only as a shift in reflection
    for k in range(4):
        y = 13 + k * 2
        d.arc([7, y - 3, 25, y + 5], start=200, end=330,
              fill=shade(stone, 1.22), width=1)
    for x in (11, 16, 21):
        d.line([(x, 12), (x - 1, 21)], fill=shade(stone, 0.84), width=1)


def draw_scanner_dish(d, rng):
    steel = (96, 122, 178, 255)
    d.ellipse([4, 8, 28, 24], fill=steel)
    d.ellipse([7, 11, 25, 21], fill=shade(steel, 1.18))
    d.ellipse([11, 13, 21, 19], fill=shade(steel, 1.3))
    # engraved symbol pattern along arcs
    for ang in range(0, 360, 30):
        x = int(16 + 8 * math.cos(math.radians(ang)))
        y = int(16 + 5 * math.sin(math.radians(ang)))
        d.point((x, y), fill=shade(steel, 0.7))
        d.point((x + 1, y), fill=shade(steel, 0.7))
    # feed arm and receiver
    d.line([(16, 16), (16, 6)], fill=shade(steel, 0.75), width=2)
    d.ellipse([14, 3, 18, 7], fill=(60, 66, 76, 255))
    d.rectangle([14, 24, 18, 28], fill=shade(steel, 0.7))


def draw_sensor(d, rng):
    housing = (96, 122, 178, 255)
    d.ellipse([5, 5, 27, 27], fill=housing)
    d.ellipse([7, 7, 25, 25], fill=shade(housing, 0.85))
    # the glassy dinner-plate eye
    d.ellipse([9, 9, 23, 23], fill=(28, 52, 96, 255))
    d.ellipse([11, 11, 21, 21], fill=(44, 92, 150, 255))
    d.ellipse([13, 13, 19, 19], fill=(16, 26, 48, 255))
    d.ellipse([11, 10, 14, 13], fill=(190, 220, 245, 255))
    d.point((19, 19), fill=(140, 190, 230, 255))
    for ang in (45, 135, 225, 315):
        x = int(16 + 10 * math.cos(math.radians(ang)))
        y = int(16 + 10 * math.sin(math.radians(ang)))
        d.point((x, y), fill=(210, 214, 220, 255))


def draw_wire_kit(d, rng, ac=False):
    box = (150, 154, 160, 255)
    # central micro controller
    d.rectangle([12, 12, 21, 20], fill=box)
    d.rectangle([12, 12, 21, 14], fill=shade(box, 1.15))
    d.point((14, 17), fill=(110, 200, 230, 255))
    d.point((17, 17), fill=(240, 150, 90, 255))
    cols = [(176, 116, 70), (110, 160, 220), (200, 90, 80)]
    # soldered wires arcing out to plugs
    for i, c in enumerate(cols):
        c = c + (255,)
        y0 = 13 + i * 3
        d.line([(12, y0), (7, y0 - 2), (4, y0 + 1)], fill=c, width=1)
        d.line([(21, y0), (25, y0 - 2), (28, y0 + 1)], fill=c, width=1)
        d.rectangle([2, y0, 4, y0 + 2], fill=(70, 72, 78, 255))
        d.rectangle([28, y0, 30, y0 + 2], fill=(70, 72, 78, 255))
    if ac:
        # mismatched pipes of the climate kit fanning below
        for i, x in enumerate((9, 14, 19)):
            d.line([(x + 2, 20), (x, 27)], fill=shade(box, 0.85 - i * 0.06), width=2)
        d.arc([6, 22, 14, 30], start=0, end=180, fill=shade(box, 0.8), width=1)


def draw_string_computer(d, rng):
    metal = (88, 90, 98, 255)
    d.rounded_rectangle([5, 5, 27, 27], radius=4, fill=metal)
    d.rounded_rectangle([5, 5, 27, 9], radius=3, fill=shade(metal, 1.16))
    # swing-open door, ajar: inner face at left, electronics inside
    d.polygon([(5, 8), (9, 11), (9, 24), (5, 26)], fill=shade(metal, 1.22))
    d.rectangle([11, 11, 25, 24], fill=(24, 26, 32, 255))
    # neatly packaged unknown electronics
    blocks = [(70, 130, 170), (150, 110, 60), (96, 160, 110), (170, 80, 90)]
    for i, c in enumerate(blocks):
        x = 12 + (i % 2) * 7
        y = 12 + (i // 2) * 6
        d.rectangle([x, y, x + 5, y + 4], fill=c + (255,))
    d.line([(13, 22), (24, 22)], fill=(200, 180, 90, 255), width=1)
    d.point((23, 13), fill=(240, 120, 90, 255))


# ---------------------------------------------------------------- nether bone

BONE = (196, 194, 186, 255)


def draw_armor_big(d, rng):
    d.polygon([(6, 8), (26, 8), (28, 20), (22, 26), (9, 25), (4, 17)],
              fill=BONE)
    d.polygon([(6, 8), (26, 8), (27, 13), (5, 13)], fill=shade(BONE, 1.08))
    # coral-like perforations
    for _ in range(52):
        x = rng.randint(6, 26)
        y = rng.randint(9, 24)
        d.point((x, y), fill=(88, 84, 76, 255))
    # barbed hooks growing on the inner (lower) side
    for x in (9, 14, 19, 24):
        d.line([(x, 25), (x - 1, 29)], fill=shade(BONE, 0.9), width=2)
        d.line([(x - 1, 29), (x + 1, 27)], fill=shade(BONE, 0.9), width=1)


def draw_armor_small(d, rng):
    d.polygon([(10, 11), (22, 9), (25, 17), (19, 23), (11, 20), (8, 15)],
              fill=shade(BONE, 0.97))
    for _ in range(16):
        x = rng.randint(10, 23)
        y = rng.randint(10, 21)
        d.point((x, y), fill=(88, 84, 76, 255))
    # crumbling: cracks and a chipped corner
    d.line([(12, 12), (16, 16), (15, 20)], fill=shade(BONE, 0.72), width=1)
    d.line([(21, 11), (19, 15)], fill=shade(BONE, 0.72), width=1)
    d.polygon([(22, 9), (25, 12), (23, 14)], fill=(0, 0, 0, 0))


def draw_spike(d, rng):
    # flexible tapered rod, diagonal, slight curve
    pts_top = [(5, 24), (11, 18), (18, 12), (26, 5)]
    pts_bot = [(8, 26), (14, 20), (20, 14), (27, 7)]
    d.polygon(pts_top + pts_bot[::-1], fill=BONE)
    d.line(pts_top, fill=shade(BONE, 1.1), width=1)
    d.line([(7, 25), (13, 19), (19, 13), (26, 6)], fill=shade(BONE, 0.85), width=1)
    # growth rings
    for t in (0.3, 0.55, 0.78):
        x = int(5 + (26 - 5) * t)
        y = int(24 + (5 - 24) * t)
        d.point((x, y + 1), fill=shade(BONE, 0.8))
        d.point((x + 1, y), fill=shade(BONE, 0.8))


def draw_statuette(d, rng):
    mat = (96, 96, 104, 255)  # rough unnatural dark gray
    # plinth
    d.rectangle([9, 25, 23, 28], fill=shade(mat, 0.8))
    # fantastical creature: horned quadruped with a curled tail
    d.ellipse([10, 15, 22, 23], fill=mat)                    # body
    d.ellipse([19, 10, 26, 17], fill=mat)                    # head
    d.line([(24, 11), (27, 6)], fill=shade(mat, 1.15), width=1)  # horn
    d.line([(21, 11), (21, 6)], fill=shade(mat, 1.15), width=1)  # horn
    d.point((23, 13), fill=(190, 200, 210, 255))             # eye
    for x in (12, 15, 19):
        d.rectangle([x, 22, x + 2, 26], fill=shade(mat, 0.92))  # legs
    d.arc([4, 12, 13, 22], start=90, end=300, fill=shade(mat, 1.1), width=2)  # tail
    for _ in range(18):  # rough stone grain
        x = rng.randint(9, 25)
        y = rng.randint(9, 24)
        d.point((x, y), fill=shade(mat, rng.choice([0.85, 1.18])))


# ---------------------------------------------------------------- yrax

GOLD = (216, 178, 52, 255)


def draw_yrax_computer(d, rng):
    d.ellipse([5, 5, 27, 27], fill=GOLD)
    d.ellipse([8, 7, 19, 16], fill=shade(GOLD, 1.16))
    d.ellipse([6, 19, 26, 26], fill=shade(GOLD, 0.88))
    # complex etched pattern: rings, radial ticks, glyph dots
    for r in (4, 7, 10):
        d.ellipse([16 - r, 16 - r, 16 + r, 16 + r], outline=shade(GOLD, 0.74), width=1)
    for ang in range(0, 360, 24):
        x = int(16 + 9 * math.cos(math.radians(ang)))
        y = int(16 + 9 * math.sin(math.radians(ang)))
        d.point((x, y), fill=shade(GOLD, 0.7))
    d.point((16, 16), fill=shade(GOLD, 0.66))


def draw_yrax_electronics(d, rng):
    ochre = (196, 150, 60, 255)
    dark = shade(ochre, 0.78)
    light = shade(ochre, 1.14)
    d.rectangle([5, 8, 27, 24], fill=ochre)
    # woven strips: alternating over/under bands
    for i, x in enumerate(range(5, 27, 4)):
        d.rectangle([x, 8, x + 2, 24], fill=light if i % 2 == 0 else dark)
    for i, y in enumerate(range(8, 24, 4)):
        for x in range(5, 27, 4):
            if (x // 4 + i) % 2 == 0:
                d.rectangle([x, y, x + 3, y + 2], fill=dark if i % 2 else light)
    d.rectangle([5, 8, 27, 9], fill=shade(ochre, 1.1))
    # frayed strip ends
    for x in (7, 13, 20, 25):
        d.line([(x, 24), (x, 27)], fill=dark, width=1)


def draw_yrax_metal(d, rng):
    plate = (228, 230, 232, 255)
    # slightly curved plate: shallow arc band with lit crown
    d.polygon([(5, 12), (27, 12), (27, 21), (5, 21)], fill=plate)
    d.polygon([(5, 12), (27, 12), (27, 15), (5, 15)], fill=shade(plate, 1.05))
    d.polygon([(5, 18), (27, 18), (27, 21), (5, 21)], fill=shade(plate, 0.9))
    d.line([(5, 12), (27, 12)], fill=shade(plate, 0.78), width=1)
    # static charge flecks snapping off the surface
    for (x, y) in ((9, 8), (16, 6), (23, 9), (12, 25), (20, 26)):
        d.point((x, y), fill=(150, 200, 245, 255))
        d.point((x + 1, y + 1), fill=(150, 200, 245, 255))


def draw_yrax_robotics(d, rng):
    steel = (196, 198, 202, 255)
    d.rectangle([6, 7, 26, 25], fill=shade(steel, 0.94))
    d.rectangle([6, 7, 26, 10], fill=steel)
    # clockwork gears
    def gear(cx, cy, r, col):
        for ang in range(0, 360, 45):
            x = int(cx + r * math.cos(math.radians(ang)))
            y = int(cy + r * math.sin(math.radians(ang)))
            d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=col)
        d.ellipse([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1], fill=col)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(40, 42, 48, 255))
    gear(12, 15, 5, GOLD)
    gear(21, 19, 4, shade(GOLD, 0.85))
    gear(22, 11, 3, steel)
    # tight plate seams
    d.line([(6, 21), (26, 21)], fill=shade(steel, 0.75), width=1)
    d.line([(17, 7), (17, 25)], fill=shade(steel, 0.75), width=1)


def draw_yrax_scrap(d, rng):
    pieces = [(216, 178, 52), (206, 208, 212), (188, 152, 44), (220, 222, 226)]
    for _ in range(7):  # tiny strips
        x = rng.randint(5, 24)
        y = rng.randint(6, 24)
        c = rng.choice(pieces) + (255,)
        d.line([(x, y), (x + rng.randint(2, 4), y + rng.randint(-1, 1))],
               fill=c, width=2)
    for _ in range(5):  # small spheres
        x = rng.randint(5, 24)
        y = rng.randint(6, 24)
        c = rng.choice(pieces) + (255,)
        d.ellipse([x, y, x + 3, y + 3], fill=c)
        d.point((x + 1, y + 1), fill=shade(c, 1.25))
    for _ in range(4):  # geometric primitives
        x = rng.randint(5, 23)
        y = rng.randint(6, 23)
        c = rng.choice(pieces) + (255,)
        if rng.random() < 0.5:
            d.polygon([(x, y + 3), (x + 2, y), (x + 4, y + 3)], fill=c)
        else:
            d.rectangle([x, y, x + 3, y + 3], fill=c)


def draw_yrax_sensor(d, rng):
    body = (224, 226, 228, 255)
    d.ellipse([5, 5, 27, 27], fill=body)
    d.ellipse([8, 7, 18, 15], fill=shade(body, 1.05))
    d.ellipse([7, 20, 25, 26], fill=shade(body, 0.9))
    # carved grooves
    d.arc([7, 7, 25, 25], start=30, end=150, fill=shade(body, 0.8), width=1)
    d.arc([7, 7, 25, 25], start=210, end=330, fill=shade(body, 0.8), width=1)
    # small black glass screens embedded in the sphere
    for (x, y) in ((11, 11), (19, 9), (15, 16), (9, 19), (21, 19), (14, 23)):
        d.rectangle([x, y, x + 3, y + 2], fill=(18, 20, 26, 255))
        d.point((x, y), fill=(120, 160, 190, 255))


def draw_yrax_trinket(d, rng):
    # tiny golden plate, drawn large to stay legible; etched artwork
    d.rectangle([7, 10, 25, 22], fill=GOLD)
    d.rectangle([7, 10, 25, 12], fill=shade(GOLD, 1.14))
    d.rectangle([7, 20, 25, 22], fill=shade(GOLD, 0.88))
    d.rectangle([8, 11, 24, 21], outline=shade(GOLD, 0.72), width=1)
    # barely-there detailed etching: a tiny diagram of circles and lines
    d.ellipse([11, 13, 16, 18], outline=shade(GOLD, 0.68), width=1)
    d.line([(16, 15), (21, 15)], fill=shade(GOLD, 0.68), width=1)
    d.line([(18, 13), (18, 19)], fill=shade(GOLD, 0.68), width=1)
    d.point((20, 18), fill=shade(GOLD, 0.62))
    d.point((12, 19), fill=shade(GOLD, 0.62))


# ---------------------------------------------------------------- dispatch

def render(item_id, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if item_id == "resin_chunk":
        draw_resin_chunk(d, rng)
    elif item_id == "alien_pod_resin":
        draw_alien_pod_resin(d, rng)
    elif item_id == "cyborg_matrix":
        draw_cyborg_matrix(d, rng)
    elif item_id == "ephemeral_gem":
        draw_ephemeral_gem(d, rng)
    elif item_id in INGOTS:
        draw_ingot(d, rng, item_id)
    elif item_id == "exodii_chassis":
        draw_chassis(d, rng, drone=False)
    elif item_id == "exodii_drone_chassis":
        draw_chassis(d, rng, drone=True)
    elif item_id == "exodii_computer":
        draw_computer(d, rng, micro=False)
    elif item_id == "exodii_micro_computer":
        draw_computer(d, rng, micro=True)
    elif item_id == "exodii_module":
        draw_rings(d, rng, nuts=False)
    elif item_id == "exodii_micro_module":
        draw_rings(d, rng, nuts=True)
    elif item_id == "exodii_motor":
        draw_motor(d, rng)
    elif item_id == "exodii_portalizer":
        draw_portalizer(d, rng)
    elif item_id == "exodii_scanner_dish":
        draw_scanner_dish(d, rng)
    elif item_id == "exodii_sensor":
        draw_sensor(d, rng)
    elif item_id == "exodii_wire_kit":
        draw_wire_kit(d, rng, ac=False)
    elif item_id == "exodii_ac_kit":
        draw_wire_kit(d, rng, ac=True)
    elif item_id == "nether_huntsman_armorpiece_big":
        draw_armor_big(d, rng)
    elif item_id == "nether_huntsman_armorpiece_small":
        draw_armor_small(d, rng)
    elif item_id == "nether_huntsman_spike":
        draw_spike(d, rng)
    elif item_id == "nether_huntsman_statuette":
        draw_statuette(d, rng)
    elif item_id == "string_dimension_computer":
        draw_string_computer(d, rng)
    elif item_id == "yrax_computer":
        draw_yrax_computer(d, rng)
    elif item_id == "yrax_electronics":
        draw_yrax_electronics(d, rng)
    elif item_id == "yrax_metal":
        draw_yrax_metal(d, rng)
    elif item_id == "yrax_robotics":
        draw_yrax_robotics(d, rng)
    elif item_id == "yrax_scrap":
        draw_yrax_scrap(d, rng)
    elif item_id == "yrax_sensor":
        draw_yrax_sensor(d, rng)
    elif item_id == "yrax_trinket":
        draw_yrax_trinket(d, rng)
    else:
        raise ValueError(item_id)
    return img


def main():
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = render(item_id, rng)
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
    cols, cell = 8, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet_out = Image.new("RGBA", (cols * cell, rows * cell),
                          (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet_out.alpha_composite(big, ((i % cols) * cell + 1,
                                        (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item39.png")
    sheet_out.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item39.png {sheet_out.size}")


if __name__ == "__main__":
    main()
