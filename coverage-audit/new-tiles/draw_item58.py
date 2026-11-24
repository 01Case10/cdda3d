#!/usr/bin/env python3
"""ITEM58: engines + combat exoskeleton armor -- CDDA Voxel
coverage audit.

Two clusters, 24 ids each:

  Engines (engine.json) -- each drawn as itself:
    * 1-cylinder combustion (small 25 cc / medium 150 cc / large
      200 cc) and diesel singles (i1, large 1-cyl diesel): one
      finned cylinder on a crankcase, size graded.
    * Inline diesels: i4 (car), i6 (industrial straight-six),
      i6 multifuel (military, olive, multi-fuel caps) -- cylinder
      head bumps and exhaust manifold counted along the block.
    * V engines: v8 diesel (semi truck), v12 diesel, v12
      combustion (sports car, polished with red valve covers) --
      two angled banks of head bumps.
    * Engine blocks tiny -> small -> medium -> large -> massive:
      bare iron blocks, cylinder bores on top, no heads/manifold.
    * aero_engine_light: radial -- cylinders radiating around a
      central prop hub.
    * Turbines small / medium / large: intake fan in a round
      cowling, casing, exhaust cone; size graded.
    * Steam: watts small is makeshift (rusty boiler, flywheel,
      stack); triple-expansion small / medium have three vertical
      cylinders of growing size, brass pipes and a flywheel.
    * xedra_genset: spartan prototype engine-generator -- small
      engine, finned generator drum, frame rails, warning stripe.

  Combat exoskeleton armor (combat_exoskeleton_armor.json) --
  arm / head / leg / torso pieces in light (2 mm) / medium (4 mm)
  / heavy (6 mm) qt-steel plate over kevlar, on a powered exo
  frame (side struts + joint discs).  Bulk scales with tier.
  Reinforced variants are reinforced with chainmail filling the
  gaps (per their descriptions): chain-link mesh in the joints
  and under plate edges, plus an extra rivet row and trim band.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "1cyl_combustion_large",
    "1cyl_combustion_med",
    "1cyl_combustion_small",
    "aero_engine_light",
    "engine_1cyl_diesel_large",
    "engine_block_large",
    "engine_block_massive",
    "engine_block_medium",
    "engine_block_small",
    "engine_block_tiny",
    "i1_diesel",
    "i4_diesel",
    "i6_diesel",
    "i6_diesel_multifuel",
    "large_turbine_engine",
    "medium_turbine_engine",
    "small_turbine_engine",
    "steam_triple_medium",
    "steam_triple_small",
    "steam_watts_small",
    "v12_combustion",
    "v12_diesel",
    "v8_diesel",
    "xedra_genset",
    "combat_exoskeleton_armor_arm_heavy",
    "combat_exoskeleton_armor_arm_heavy_reinforced",
    "combat_exoskeleton_armor_arm_light",
    "combat_exoskeleton_armor_arm_light_reinforced",
    "combat_exoskeleton_armor_arm_medium",
    "combat_exoskeleton_armor_arm_medium_reinforced",
    "combat_exoskeleton_armor_head_heavy",
    "combat_exoskeleton_armor_head_heavy_reinforced",
    "combat_exoskeleton_armor_head_light",
    "combat_exoskeleton_armor_head_light_reinforced",
    "combat_exoskeleton_armor_head_medium",
    "combat_exoskeleton_armor_head_medium_reinforced",
    "combat_exoskeleton_armor_leg_heavy",
    "combat_exoskeleton_armor_leg_heavy_reinforced",
    "combat_exoskeleton_armor_leg_light",
    "combat_exoskeleton_armor_leg_light_reinforced",
    "combat_exoskeleton_armor_leg_medium",
    "combat_exoskeleton_armor_leg_medium_reinforced",
    "combat_exoskeleton_armor_torso_heavy",
    "combat_exoskeleton_armor_torso_heavy_reinforced",
    "combat_exoskeleton_armor_torso_light",
    "combat_exoskeleton_armor_torso_light_reinforced",
    "combat_exoskeleton_armor_torso_medium",
    "combat_exoskeleton_armor_torso_medium_reinforced",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

# engine metals
IRON = (88, 90, 96, 255)
IRON_DARK = (54, 56, 62, 255)
IRON_LIGHT = (140, 144, 152, 255)
STEEL = (112, 120, 132, 255)
STEEL_DARK = (66, 72, 84, 255)
STEEL_LIGHT = (168, 178, 192, 255)
QT_STEEL = (128, 142, 164, 255)
DIESEL = (74, 76, 72, 255)       # grimy diesel cast iron
OLIVE = (96, 104, 66, 255)       # military multifuel paint
RED_PAINT = (168, 44, 38, 255)   # v12 combustion valve covers
BRASS = (186, 148, 74, 255)
COPPER = (172, 106, 62, 255)
RUST = (142, 84, 52, 255)
COAL = (30, 30, 34, 255)
RUBBER = (38, 38, 42, 255)

# armor materials
PLATE = (122, 134, 154, 255)     # qt steel plate
PLATE_DARK = (78, 88, 104, 255)
PLATE_LIGHT = (178, 190, 206, 255)
KEVLAR = (52, 54, 48, 255)       # dark kevlar underlayer
FRAME = (44, 46, 52, 255)        # exo frame struts
CHAIN = (158, 166, 178, 255)     # chainmail links
VISOR = (28, 40, 48, 255)
VISOR_GLINT = (72, 190, 180, 255)
TRIM = (206, 170, 84, 255)       # reinforced trim band


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def bolts(d, rng, x0, x1, y, col=STEEL_LIGHT):
    for x in range(x0, x1 + 1, 4):
        d.point((x, y), fill=col)


# ------------------------------------------------------------------ engines

def engine_base(d, rng, cx, w, y_top, y_bot, body=IRON, dark=IRON_DARK,
                light=IRON_LIGHT):
    """Crankcase block: shaded box with a top deck and bolt row."""
    x0, x1 = cx - w // 2, cx + w // 2 - (0 if w % 2 else 1)
    d.rectangle([x0, y_top, x1, y_bot], fill=body)
    d.rectangle([x0, y_top, x1, y_top + 1], fill=light)
    d.rectangle([x0, y_bot - 1, x1, y_bot], fill=dark)
    d.line([x0, y_top, x0, y_bot], fill=dark, width=1)
    d.line([x1, y_top, x1, y_bot], fill=dark, width=1)
    bolts(d, rng, x0 + 2, x1 - 2, y_bot - 3)
    # crank snout + pulley at the front (bottom centre)
    d.rectangle([cx - 1, y_bot, cx + 1, y_bot + 1], fill=dark)
    d.point((cx, y_bot + 1), fill=light)
    return x0, x1


def finned_cylinder(d, x, y_top, w, h, body=STEEL, dark=STEEL_DARK):
    """One air-cooled cylinder: cooling fins + head cap."""
    for i in range(h // 2):
        y = y_top + i * 2
        d.line([x - 1, y, x + w, y], fill=dark, width=1)
    d.rectangle([x, y_top, x + w - 1, y_top + h - 1], fill=body)
    for i in range(h // 2):
        y = y_top + i * 2
        d.line([x - 1, y, x + w, y], fill=dark, width=1)
    d.rectangle([x, y_top, x + w - 1, y_top + 1], fill=shade(body, 1.3))
    d.point((x + w // 2, y_top), fill=(30, 30, 34, 255))  # plug


def draw_single(d, rng, scale, diesel=False):
    """Single-cylinder engine; scale = (block w, cyl w, cyl h)."""
    bw, cw, ch = scale
    cx = 16
    y_bot = 27
    body = DIESEL if diesel else IRON
    x0, x1 = engine_base(d, rng, cx, bw, 19, y_bot, body=body,
                         dark=shade(body, 0.6), light=shade(body, 1.5))
    finned_cylinder(d, cx - cw // 2, 19 - ch, cw, ch,
                    body=STEEL if not diesel else STEEL_DARK)
    # exhaust pipe curling off the cylinder's right side
    d.line([cx + cw // 2 + 1, 19 - ch + 3, cx + cw // 2 + 3,
            19 - ch + 3], fill=IRON_DARK, width=2)
    d.line([cx + cw // 2 + 3, 19 - ch + 3, cx + cw // 2 + 3, 22],
           fill=IRON_DARK, width=2)
    if diesel:  # injector + fuel filter canister on the case
        d.point((cx - 1, 19 - ch - 1), fill=BRASS)
        d.rectangle([x0 + 1, 22, x0 + 3, 25], fill=BRASS)
    else:  # pull-start housing disc on the case front
        d.ellipse([cx - 3, 21, cx + 3, 26], outline=IRON_LIGHT, width=1)
        d.point((cx, 23), fill=IRON_DARK)


def draw_inline(d, rng, n_cyl, w, military=False):
    """Inline diesel: head bumps + manifold counted along the block."""
    cx = 16
    body = OLIVE if military else DIESEL
    x0, x1 = engine_base(d, rng, cx, w, 17, 27, body=body,
                         dark=shade(body, 0.6), light=shade(body, 1.45))
    # valve cover with one bump per cylinder
    seg = w // n_cyl
    for i in range(n_cyl):
        bx = x0 + 1 + i * seg
        d.rectangle([bx, 13, bx + seg - 2, 16], fill=shade(body, 1.25))
        d.point((bx + (seg - 2) // 2, 14), fill=STEEL_DARK)  # injector
    d.line([x0, 12, x1, 12], fill=shade(body, 1.5), width=1)
    # exhaust manifold: a runner swooping from each cylinder
    for i in range(n_cyl):
        bx = x0 + 2 + i * seg
        d.line([bx, 17, bx, 19], fill=IRON_DARK, width=1)
    d.line([x0 + 1, 19, x1 - 1, 19], fill=IRON_DARK, width=2)
    if military:  # multifuel: three fuel caps in different colours
        for i, cap in enumerate(((196, 60, 44, 255), (240, 200, 60, 255),
                                 (90, 160, 80, 255))):
            d.point((x0 + 3 + i * 3, 10), fill=cap)
            d.point((x0 + 3 + i * 3, 11), fill=cap)


def draw_v(d, rng, n_per_bank, w, sport=False):
    """V engine: two angled banks of head bumps over a deep block."""
    cx = 16
    body = IRON
    x0, x1 = engine_base(d, rng, cx, w, 18, 28, body=body,
                         dark=IRON_DARK, light=IRON_LIGHT)
    cover = RED_PAINT if sport else STEEL_DARK
    seg = (w // 2) // n_per_bank
    for side in (-1, 1):
        for i in range(n_per_bank):
            bx = cx + side * (2 + i * seg) - (seg // 2 if side > 0
                                              else seg // 2)
            by = 15 - i // 3
            d.rectangle([bx, by, bx + seg - 2, by + 3],
                        fill=cover if sport or i % 2 == 0 else STEEL)
            d.line([bx, by, bx + seg - 2, by], fill=STEEL_LIGHT, width=1)
    # intake plenum in the valley
    d.rectangle([cx - 2, 12, cx + 2, 16], fill=STEEL)
    d.line([cx - 2, 12, cx + 2, 12], fill=STEEL_LIGHT, width=1)
    if sport:  # polished intake trumpets
        for dx in (-1, 1):
            d.point((cx + dx, 10), fill=STEEL_LIGHT)
            d.point((cx + dx, 11), fill=STEEL_LIGHT)


def draw_block(d, rng, w, h):
    """Bare engine block: bores on top, no heads, no manifold."""
    cx = 16
    y_bot = 26 if h < 20 else 28
    y_top = y_bot - h
    x0, x1 = cx - w // 2, cx + w // 2 - (0 if w % 2 else 1)
    d.rectangle([x0, y_top, x1, y_bot], fill=IRON)
    d.rectangle([x0, y_bot - 2, x1, y_bot], fill=IRON_DARK)
    d.line([x0, y_top, x0, y_bot], fill=IRON_DARK, width=1)
    d.line([x1, y_top, x1, y_bot], fill=IRON_DARK, width=1)
    # cylinder bores: dark circles in a row on the deck
    n = max(1, w // 6)
    for i in range(n):
        bx = x0 + 3 + i * (w - 4) // max(1, n) if n > 1 else cx
        if n == 1:
            bx = cx
        r = 2 if w >= 14 else 1
        d.ellipse([bx - r, y_top - r + 1, bx + r, y_top + r + 1],
                  fill=(24, 24, 28, 255))
        d.ellipse([bx - r, y_top - r + 1, bx + r, y_top + r + 1],
                  outline=IRON_LIGHT, width=1)
    # casting ribs + main bearing caps along the side
    for x in range(x0 + 3, x1 - 1, 5):
        d.line([x, y_top + 4, x, y_bot - 4], fill=IRON_DARK, width=1)
    bolts(d, rng, x0 + 2, x1 - 2, y_bot - 4, col=IRON_DARK)
    # oil pan hint below
    d.rectangle([x0 + 2, y_bot, x1 - 2, y_bot + 1], fill=IRON_DARK)


def draw_radial(d, rng):
    """Aero radial: cylinders radiating around a central prop hub."""
    cx, cy = 16, 16
    for k in range(7):
        a = -math.pi / 2 + k * (2 * math.pi / 7)
        x2 = cx + 9 * math.cos(a)
        y2 = cy + 9 * math.sin(a)
        d.line([cx, cy, x2, y2], fill=STEEL_DARK, width=3)
        d.line([cx, cy, x2, y2], fill=STEEL, width=1)
        d.point((int(x2), int(y2)), fill=STEEL_LIGHT)
    d.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=IRON)
    d.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], outline=IRON_LIGHT,
              width=1)
    # prop hub: spinner cone + two blade roots
    d.line([cx - 2, cy, cx - 12, cy - 2], fill=IRON_DARK, width=2)
    d.line([cx + 2, cy, cx + 12, cy - 2], fill=IRON_DARK, width=2)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_LIGHT)
    d.point((cx, cy), fill=IRON_DARK)


def draw_turbine(d, rng, r):
    """Gas turbine: round intake with fan blades, casing, exhaust."""
    cx, cy = 13, 16
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=QT_STEEL)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=STEEL_DARK,
              width=1)
    ir = r - 3
    d.ellipse([cx - ir, cy - ir, cx + ir, cy + ir], fill=(30, 34, 40, 255))
    for k in range(8):  # fan blades
        a = k * math.pi / 4 + 0.3
        d.line([cx, cy, cx + ir * math.cos(a), cy + ir * math.sin(a)],
               fill=STEEL_LIGHT, width=1)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=STEEL_LIGHT)
    # casing running back to an exhaust cone at the right
    ex = cx + r
    d.rectangle([ex - 2, cy - r + 3, 29, cy + r - 3], fill=STEEL)
    d.line([ex - 2, cy - r + 3, 29, cy - r + 3], fill=STEEL_LIGHT, width=1)
    d.line([ex - 2, cy + r - 3, 29, cy + r - 3], fill=STEEL_DARK, width=1)
    d.polygon([(29, cy - r + 5), (31, cy), (29, cy + r - 5)],
              fill=STEEL_DARK)
    for x in range(ex + 1, 29, 3):  # casing bands
        d.line([x, cy - r + 4, x, cy + r - 4], fill=STEEL_DARK, width=1)


def steam_boiler(d, rng, x0, x1, y0, y1, body=IRON):
    d.rectangle([x0, y0, x1, y1], fill=body)
    d.rectangle([x0, y0, x1, y0 + 1], fill=shade(body, 1.4))
    for x in range(x0 + 2, x1 - 1, 4):  # rivet row
        d.point((x, y1 - 2), fill=shade(body, 1.5))
    d.rectangle([x0 + 2, y1 - 5, x0 + 6, y1 - 1], fill=COAL)  # firebox door
    d.point((x0 + 4, y1 - 3), fill=(240, 140, 50, 255))       # fire glow


def flywheel(d, cx, cy, r, col=IRON_DARK):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=col, width=2)
    d.line([cx - r, cy, cx + r, cy], fill=col, width=1)
    d.line([cx, cy - r, cx, cy + r], fill=col, width=1)
    d.point((cx, cy), fill=STEEL_LIGHT)


def draw_steam_watts(d, rng):
    """Makeshift Watts steam engine: rusty boiler, stack, flywheel."""
    steam_boiler(d, rng, 4, 22, 14, 27, body=RUST)
    d.rectangle([6, 6, 8, 14], fill=IRON_DARK)               # smokestack
    d.rectangle([5, 5, 9, 6], fill=IRON_DARK)
    for px, py in ((6, 3), (7, 2)):                          # smoke puffs
        d.point((px, py), fill=(170, 170, 174, 255))
    d.rectangle([12, 9, 15, 14], fill=BRASS)                 # cylinder
    d.line([13, 9, 24, 18], fill=STEEL_DARK, width=1)        # rod
    flywheel(d, 25, 21, 5)
    d.line([4, 28, 28, 28], fill=(96, 66, 38, 255), width=1)  # skid base
    d.point((18, 24), fill=COPPER)                           # bodged pipe


def draw_steam_triple(d, rng, scale):
    """Triple-expansion: 3 growing cylinders, brass pipes, flywheel."""
    w = scale  # boiler half-width grade
    steam_boiler(d, rng, 16 - w, 16 + w - 2, 20, 28, body=IRON)
    # three vertical cylinders of increasing bore, left to right
    cyl_w = (4, 5, 7) if scale > 11 else (3, 4, 6)
    x = 16 - w + 1
    for i, cw in enumerate(cyl_w):
        top = 11 - i
        d.rectangle([x, top, x + cw - 1, 20], fill=STEEL)
        d.rectangle([x, top, x + cw - 1, top + 1], fill=BRASS)
        d.line([x, top + 3, x + cw - 1, top + 3], fill=STEEL_DARK,
               width=1)
        d.line([x + cw // 2, 20, x + cw // 2, 23], fill=STEEL_DARK,
               width=1)  # piston rod
        x += cw + 2
    # brass steam pipe arching over the cylinders
    d.line([16 - w + 2, 9, 16 + w - 3, 9], fill=BRASS, width=1)
    d.line([16 - w + 2, 9, 16 - w + 2, 12], fill=BRASS, width=1)
    d.rectangle([16 + w - 1, 12, 16 + w, 20], fill=COPPER)  # condenser
    flywheel(d, 16 + w + 3 if scale <= 11 else 29, 24, 4, col=STEEL_DARK)


def draw_genset(d, rng):
    """XEDRA genset: engine + finned generator on frame rails."""
    # frame rails
    d.line([3, 28, 29, 28], fill=FRAME, width=1)
    d.line([3, 26, 3, 28], fill=FRAME, width=1)
    d.line([29, 26, 29, 28], fill=FRAME, width=1)
    # engine (left): small finned single
    engine_base(d, rng, 10, 12, 18, 26, body=IRON, dark=IRON_DARK,
                light=IRON_LIGHT)
    finned_cylinder(d, 8, 11, 5, 7)
    # generator drum (right): horizontal cylinder with cooling fins
    d.rectangle([17, 14, 28, 25], fill=STEEL)
    for x in range(18, 28, 2):
        d.line([x, 14, x, 25], fill=STEEL_DARK, width=1)
    d.rectangle([17, 14, 28, 15], fill=STEEL_LIGHT)
    d.ellipse([26, 17, 29, 22], fill=STEEL_DARK)            # end bell
    d.point((27, 19), fill=STEEL_LIGHT)
    # control box + spartan warning stripe
    d.rectangle([18, 8, 24, 12], fill=(60, 62, 66, 255))
    d.point((20, 10), fill=(240, 60, 50, 255))
    d.point((22, 10), fill=(80, 220, 90, 255))
    for x in range(4, 12, 2):  # hazard stripe on the rail
        d.point((x, 27), fill=(236, 200, 60, 255))


ENGINES = {
    "1cyl_combustion_small": ("single", (10, 4, 5), False),
    "1cyl_combustion_med": ("single", (13, 5, 7), False),
    "1cyl_combustion_large": ("single", (16, 6, 8), False),
    "engine_1cyl_diesel_large": ("single", (16, 6, 8), True),
    "i1_diesel": ("single", (14, 5, 7), True),
    "i4_diesel": ("inline", 4, 20),
    "i6_diesel": ("inline", 6, 26),
    "i6_diesel_multifuel": ("inline_mil", 6, 26),
    "v8_diesel": ("v", 4, 22),
    "v12_diesel": ("v", 6, 27),
    "v12_combustion": ("v_sport", 6, 27),
    "engine_block_tiny": ("block", 10, 9),
    "engine_block_small": ("block", 14, 12),
    "engine_block_medium": ("block", 19, 16),
    "engine_block_large": ("block", 24, 20),
    "engine_block_massive": ("block", 28, 23),
    "aero_engine_light": ("radial",),
    "small_turbine_engine": ("turbine", 8),
    "medium_turbine_engine": ("turbine", 10),
    "large_turbine_engine": ("turbine", 12),
    "steam_watts_small": ("watts",),
    "steam_triple_small": ("triple", 11),
    "steam_triple_medium": ("triple", 13),
    "xedra_genset": ("genset",),
}


def draw_engine(d, rng, sid):
    spec = ENGINES[sid]
    kind = spec[0]
    if kind == "single":
        draw_single(d, rng, spec[1], diesel=spec[2])
    elif kind == "inline":
        draw_inline(d, rng, spec[1], spec[2])
    elif kind == "inline_mil":
        draw_inline(d, rng, spec[1], spec[2], military=True)
    elif kind == "v":
        draw_v(d, rng, spec[1], spec[2])
    elif kind == "v_sport":
        draw_v(d, rng, spec[1], spec[2], sport=True)
    elif kind == "block":
        draw_block(d, rng, spec[1], spec[2])
    elif kind == "radial":
        draw_radial(d, rng)
    elif kind == "turbine":
        draw_turbine(d, rng, spec[1])
    elif kind == "watts":
        draw_steam_watts(d, rng)
    elif kind == "triple":
        draw_steam_triple(d, rng, spec[1])
    elif kind == "genset":
        draw_genset(d, rng)
    else:
        raise KeyError(sid)


# ------------------------------------------------------------------- armor

# tier -> (half width of the piece, plate inset, extra bulk rows)
TIERS = {"light": (7, 1, 0), "medium": (9, 2, 1), "heavy": (11, 2, 2)}


def chainmail(d, x0, x1, y0, y1):
    """Interlinked-ring mesh: alternating dots in a tight grid."""
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if (x + y) % 2 == 0:
                d.point((x, y), fill=CHAIN)
            elif x % 2 == 0:
                d.point((x, y), fill=shade(CHAIN, 0.7))


def rivets(d, x0, x1, y0, y1):
    for x, y in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)):
        d.point((x, y), fill=PLATE_LIGHT)


def plate(d, x0, x1, y0, y1, bulk):
    """A qt-steel plate with shaded rim; bulk thickens the rim."""
    d.rectangle([x0, y0, x1, y1], fill=PLATE)
    rim = min(bulk, (x1 - x0) // 2, (y1 - y0) // 2)
    for i in range(rim + 1):
        d.rectangle([x0 + i, y0 + i, x1 - i, y1 - i], outline=PLATE_DARK,
                    width=1)
    if y0 + rim + 1 <= y1 and x0 + rim + 1 <= x1 - rim - 1:
        d.line([x0 + rim + 1, y0 + rim + 1, x1 - rim - 1, y0 + rim + 1],
               fill=PLATE_LIGHT, width=1)
    rivets(d, x0 + 1, x1 - 1, y0 + 1, y1 - 1)


def exo_struts(d, x0, x1, y0, y1):
    """Powered frame struts down both sides, with joint discs."""
    for x in (x0, x1):
        d.line([x, y0, x, y1], fill=FRAME, width=2)
        d.point((x, (y0 + y1) // 2), fill=STEEL_LIGHT)


def reinforced_trim(d, x0, x1, y):
    d.line([x0, y, x1, y], fill=TRIM, width=1)
    for x in range(x0 + 1, x1, 3):
        d.point((x, y - 1), fill=PLATE_LIGHT)  # extra rivet row


def draw_armor(d, rng, part, tier, reinforced):
    hw, inset, bulk = TIERS[tier]
    cx = 16
    x0, x1 = cx - hw, cx + hw - 1
    if part == "head":
        # helmet: dome + brow plate + visor slit + cheek/neck guards
        top = 6
        d.rectangle([x0 + 2, top + 4, x1 - 2, 26], fill=KEVLAR)
        d.pieslice([x0, top, x1, top + 14], start=180, end=360, fill=PLATE)
        d.pieslice([x0, top, x1, top + 14], start=180, end=360,
                   outline=PLATE_DARK, width=bulk + 1)
        plate(d, x0 + 1, x1 - 1, top + 6, top + 9, bulk)      # brow band
        d.rectangle([x0 + 2, top + 11, x1 - 2, top + 13], fill=VISOR)
        d.point((cx - 2, top + 12), fill=VISOR_GLINT)
        d.point((cx + 3, top + 12), fill=VISOR_GLINT)
        plate(d, x0 + 1, x0 + 4, top + 14, 24, bulk)          # cheek L
        plate(d, x1 - 4, x1 - 1, top + 14, 24, bulk)          # cheek R
        if reinforced:
            chainmail(d, x0 + 6, x1 - 6, top + 14, 21)        # face gap
            chainmail(d, x0 + 2, x1 - 2, 25, 27)              # neck gap
            reinforced_trim(d, x0 + 1, x1 - 1, top + 5)
        else:
            d.rectangle([x0 + 6, top + 14, x1 - 6, 20], fill=KEVLAR)
            d.line([cx, top + 14, cx, 20], fill=PLATE_DARK, width=1)
        d.rectangle([x0 + 3, 25, x1 - 3, 27], fill=PLATE_DARK)  # neck guard
    elif part == "torso":
        # cuirass: chest plate, ab rows, shoulder straps, spine strut
        top = 4 if tier == "heavy" else 6
        d.rectangle([x0 + 1, top + 3, x1 - 1, 28], fill=KEVLAR)
        plate(d, x0, x1, top + 2, top + 11, bulk)             # chest
        d.line([cx, top + 3, cx, top + 10], fill=PLATE_DARK, width=1)
        d.point((cx, top + 6), fill=PLATE_LIGHT)              # chest light
        for i in range(2 + bulk):                             # ab plates
            y = top + 13 + i * 4
            plate(d, x0 + 1 + i, x1 - 1 - i, y, y + 3, 0)
        plate(d, x0, x0 + 3, top, top + 3, 0)                 # strap L
        plate(d, x1 - 3, x1, top, top + 3, 0)                 # strap R
        exo_struts(d, x0 - 1, x1 + 1, top + 4, 27)
        if reinforced:
            chainmail(d, x0 + 2, x0 + 4, top + 12, 26)        # side gaps
            chainmail(d, x1 - 4, x1 - 2, top + 12, 26)
            chainmail(d, cx - 1, cx + 1, top + 12, 13)
            reinforced_trim(d, x0, x1, top + 1)
    elif part == "arm":
        # arm guard: upper plate, elbow cop, forearm plate, gauntlet
        top = 3 if tier == "heavy" else 5
        d.rectangle([x0 + 2, top, x1 - 2, 29], fill=KEVLAR)
        plate(d, x0, x1, top, top + 8, bulk)                  # upper arm
        exo_struts(d, x0 - 1, x1 + 1, top, 28)
        ey = top + 11                                         # elbow cop
        d.ellipse([cx - 3 - bulk, ey - 3 - bulk, cx + 3 + bulk,
                   ey + 3 + bulk], fill=PLATE)
        d.ellipse([cx - 3 - bulk, ey - 3 - bulk, cx + 3 + bulk,
                   ey + 3 + bulk], outline=PLATE_DARK, width=1)
        d.point((cx, ey), fill=PLATE_LIGHT)
        if reinforced:
            chainmail(d, x0 + 2, x1 - 2, top + 9, ey - 4)     # inner elbow
        plate(d, x0 + 1, x1 - 1, ey + 4, ey + 11, bulk)       # forearm
        if reinforced:
            chainmail(d, x0 + 2, x1 - 2, ey + 12, ey + 14)    # wrist gap
            reinforced_trim(d, x0, x1, top - 1 if top > 0 else top)
        # gauntlet: back-of-hand plate + finger segments
        gy = ey + 13 if not reinforced else ey + 15
        gy = min(gy, 25)
        plate(d, x0 + 2, x1 - 2, gy, gy + 2, 0)
        for fx in range(x0 + 3, x1 - 2, 3):
            d.line([fx, gy + 3, fx, min(gy + 5, 30)], fill=PLATE_DARK,
                   width=1)
    elif part == "leg":
        # leg guard: thigh plate, knee cop, shin plate, foot plate
        top = 3 if tier == "heavy" else 5
        d.rectangle([x0 + 2, top, x1 - 2, 29], fill=KEVLAR)
        plate(d, x0, x1, top, top + 8, bulk)                  # thigh
        exo_struts(d, x0 - 1, x1 + 1, top, 28)
        ky = top + 11                                         # knee cop
        d.ellipse([cx - 3 - bulk, ky - 3 - bulk, cx + 3 + bulk,
                   ky + 3 + bulk], fill=PLATE)
        d.ellipse([cx - 3 - bulk, ky - 3 - bulk, cx + 3 + bulk,
                   ky + 3 + bulk], outline=PLATE_DARK, width=1)
        d.point((cx, ky), fill=PLATE_LIGHT)
        if reinforced:
            chainmail(d, x0 + 2, x1 - 2, top + 9, ky - 4)     # back of knee
        plate(d, x0 + 1, x1 - 1, ky + 4, ky + 12, bulk)       # shin
        if reinforced:
            chainmail(d, x0 + 2, x1 - 2, ky + 13, ky + 14)    # ankle gap
            reinforced_trim(d, x0, x1, top - 1 if top > 0 else top)
        fy = min(ky + 14 if not reinforced else ky + 16, 28)
        d.rectangle([x0 + 1, fy, x1 + 1, fy + 1], fill=PLATE_DARK)  # foot
        d.line([x0 + 1, fy, x1 + 1, fy], fill=PLATE_LIGHT, width=1)
    # a few scratches on the plate
    for _ in range(2):
        sx = rng.randint(x0 + 2, x1 - 3)
        sy = rng.randint(6, 24)
        d.point((sx, sy), fill=PLATE_DARK)


def parse_armor(sid):
    rest = sid[len("combat_exoskeleton_armor_"):]
    reinforced = rest.endswith("_reinforced")
    if reinforced:
        rest = rest[:-len("_reinforced")]
    part, tier = rest.rsplit("_", 1)
    return part, tier, reinforced


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in ENGINES:
        draw_engine(d, rng, sid)
    elif sid.startswith("combat_exoskeleton_armor_"):
        part, tier, reinforced = parse_armor(sid)
        draw_armor(d, rng, part, tier, reinforced)
    else:
        raise KeyError(sid)
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
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item58.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item58.png {prev.size}")


if __name__ == "__main__":
    main()
