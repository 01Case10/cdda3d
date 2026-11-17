#!/usr/bin/env python3
"""ITEM81: robofac exo armor, textile crafts, protein foods, serums and
an ammo tail -- CDDA Voxel coverage audit.

Families (49 ids):

  Hub 01 exoskeleton pieces (6) -- dark qt_steel motionframes drawn as
    the body-part frame they are: ARM (shoulder mount, upper-arm bars,
    elbow joint, forearm bars, gauntlet), BACK (shoulder yoke, spine
    rail, power unit, hip belt), LEG (hip mount, thigh bars, knee,
    shin bars, foot plate).  The _on variants light cyan power seams
    and status LEDs along the same frame; off pieces stay dark.
  Textile crafts (6) -- wooden frame loom with warp threads and a
    half-woven striped cloth, red sewing machine in profile, flat
    wooden shed stick, fanned yellow paper tailoring patterns with a
    dashed garment outline, and tanning hide / fur pelt in progress
    (irregular skin outline, wet sheen; the pelt gets fur strokes).
  Protein foods (6) -- gable-top fortified milk carton with a plus
    badge, two wrapped protein bars (milk/whey: cream wrapper, blue
    band, drop motif; peanut: orange-brown wrapper, peanut motif),
    shaker bottles of protein shake (fortified adds a gold cap band
    and plus badge), and a domed-lid protein smoothie with straw.
  Serums (6) -- little sealed glass ampoules; liquid colour is the
    colour named in each serum's game entry (learning blue, night
    operations black, regen white, resilience gray, speed yellow,
    super soldier red), with a white label carrying a motif glyph
    (book, moon, cross, shield, lightning bolt, star).
  Ammo tail (25) -- one upright cartridge per id, drawn per-row so
    bullet profile and case finish read at a glance.  Sized by
    caliber: .380 ACP small, .38 Special longer rimmed case, .45 ACP
    short and fat, .45-70 a long rifle case with a wide flat lead
    nose, .50 BMG a huge bottleneck case.  Bullet by load: FMJ round
    nose, JHP flat with a dark cavity, +P loads add a red case band,
    .45-70 LFN flat lead, SP exposed-lead tip, penetrator pointed,
    .50 M33 ball plain copper, M17 tracer red tip, Mk 211 green tip
    with white ring, Match sleek polished ogive, M2 AP black tip.
    Finish by prefix: factory bright brass, bp_ soot-smudged dull
    brass, matchhead_ polished with an amber mouth band, reloaded_
    scuffed two-tone brass.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "integrated_bio_robofac_xmarm",
    "integrated_bio_robofac_xmarm_on",
    "integrated_bio_robofac_xmback",
    "integrated_bio_robofac_xmback_on",
    "integrated_bio_robofac_xmleg",
    "integrated_bio_robofac_xmleg_on",
    "loom_frame",
    "sewing_machine",
    "shed_stick",
    "tailoring_pattern_set",
    "tanning_hide_active",
    "tanning_pelt_active",
    "milk_fortified",
    "protein_bar_milk",
    "protein_bar_peanut",
    "protein_shake",
    "protein_shake_fortified",
    "protein_smoothie",
    "serum_learning",
    "serum_night_operations",
    "serum_regen",
    "serum_resilience",
    "serum_speed",
    "serum_super_soldier",
    "380_JHP",
    "380_p",
    "bp_380_FMJ",
    "matchhead_380_FMJ",
    "reloaded_380_FMJ",
    "38_fmj",
    "38_special_p",
    "bp_38_fmj",
    "matchhead_38_fmj",
    "reloaded_38_fmj",
    "4570_low",
    "reloaded_4570_bp",
    "reloaded_4570_low",
    "reloaded_4570_pen",
    "reloaded_4570_sp",
    "45_acp",
    "45_super",
    "bp_45_acp",
    "matchhead_45_acp",
    "reloaded_45_acp",
    "50_incendiary",
    "50_mk211",
    "50bmg",
    "50match",
    "50ss",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 46, 52, 255)
STEEL_DARK = (52, 56, 62, 255)
STEEL = (88, 94, 102, 255)
STEEL_LIGHT = (152, 158, 166, 255)
GLOW = (66, 216, 196, 255)
GLOW_CORE = (196, 255, 246, 255)
WOOD = (150, 110, 64, 255)
WOOD_DARK = (108, 76, 40, 255)
WOOD_LIGHT = (188, 150, 98, 255)
PAPER = (240, 230, 196, 255)
RED = (178, 44, 44, 255)
RED_DARK = (138, 30, 30, 255)
RED_BAND = (192, 50, 44, 255)
BRASS = (200, 164, 78, 255)
BRASS_LIGHT = (236, 202, 112, 255)
BRASS_DARK = (148, 116, 50, 255)
COPPER = (198, 122, 74, 255)
LEAD = (150, 152, 158, 255)
LEAD_DARK = (116, 118, 124, 255)
SOOT = (62, 60, 56, 255)
AMBER = (228, 152, 42, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- exoskeleton

def exo_arm(d, on):
    # shoulder mount
    d.rectangle([6, 5, 14, 10], fill=STEEL, outline=STEEL_DARK)
    d.rectangle([8, 7, 11, 8], fill=STEEL_DARK)
    # upper-arm parallel bars
    d.line([8, 10, 9, 17], fill=STEEL, width=2)
    d.line([13, 10, 13, 17], fill=STEEL, width=2)
    d.line([10, 12, 10, 16], fill=STEEL_LIGHT, width=1)  # piston
    # elbow joint
    d.ellipse([8, 16, 15, 22], fill=STEEL_DARK)
    d.point((11, 19), fill=STEEL_LIGHT)
    # forearm bars
    d.line([9, 21, 8, 26], fill=STEEL, width=2)
    d.line([13, 21, 13, 26], fill=STEEL, width=2)
    # gauntlet with finger notches
    d.rectangle([7, 26, 14, 30], fill=STEEL, outline=STEEL_DARK)
    for fx in (9, 11, 13):
        d.line([fx, 28, fx, 30], fill=STEEL_DARK, width=1)
    if on:
        d.line([10, 11, 11, 16], fill=GLOW, width=1)
        d.line([11, 22, 11, 26], fill=GLOW, width=1)
        d.line([8, 28, 13, 28], fill=GLOW, width=1)
        for px, py in ((10, 7), (11, 19), (8, 27)):
            d.point((px, py), fill=GLOW_CORE)
            d.point((px + 1, py), fill=GLOW)


def exo_back(d, on):
    # shoulder yoke
    d.rectangle([6, 7, 25, 10], fill=STEEL, outline=STEEL_DARK)
    d.rectangle([6, 10, 9, 13], fill=STEEL)
    d.rectangle([22, 10, 25, 13], fill=STEEL)
    # spine rail
    d.rectangle([14, 10, 17, 26], fill=STEEL_DARK)
    # power unit box with vents and core
    d.rectangle([10, 13, 21, 21], fill=STEEL, outline=STEEL_DARK)
    for vx in (12, 19):
        d.line([vx, 15, vx, 19], fill=STEEL_DARK, width=1)
    d.ellipse([14, 15, 17, 18], outline=STEEL_LIGHT)
    # side rails to hip belt
    d.line([10, 21, 9, 25], fill=STEEL, width=2)
    d.line([21, 21, 22, 25], fill=STEEL, width=2)
    d.rectangle([7, 25, 24, 28], fill=STEEL, outline=STEEL_DARK)
    d.rectangle([14, 26, 17, 27], fill=STEEL_LIGHT)  # buckle
    if on:
        d.line([15, 11, 15, 25], fill=GLOW, width=1)
        d.ellipse([14, 15, 17, 18], outline=GLOW)
        d.point((15, 16), fill=GLOW_CORE)
        d.line([8, 8, 23, 8], fill=GLOW, width=1)
        for px, py in ((7, 11), (24, 11), (15, 27)):
            d.point((px, py), fill=GLOW_CORE)


def exo_leg(d, on):
    # hip mount
    d.rectangle([11, 4, 20, 8], fill=STEEL, outline=STEEL_DARK)
    d.rectangle([14, 6, 17, 7], fill=STEEL_DARK)
    # thigh bars
    d.line([13, 8, 13, 17], fill=STEEL, width=2)
    d.line([18, 8, 18, 17], fill=STEEL, width=2)
    d.line([16, 10, 16, 16], fill=STEEL_LIGHT, width=1)  # piston
    # knee joint
    d.ellipse([12, 17, 19, 22], fill=STEEL_DARK)
    d.point((15, 19), fill=STEEL_LIGHT)
    # shin bars
    d.line([13, 21, 13, 27], fill=STEEL, width=2)
    d.line([18, 21, 18, 27], fill=STEEL, width=2)
    # foot plate
    d.polygon([(11, 27), (19, 27), (23, 30), (10, 30)], fill=STEEL,
              outline=STEEL_DARK)
    if on:
        d.line([15, 9, 15, 16], fill=GLOW, width=1)
        d.line([15, 22, 15, 27], fill=GLOW, width=1)
        d.line([12, 29, 21, 29], fill=GLOW, width=1)
        for px, py in ((15, 6), (15, 19), (19, 27)):
            d.point((px, py), fill=GLOW_CORE)
            d.point((px + 1, py), fill=GLOW)


# ---------------------------------------------------------- textile crafts

def loom(d, rng):
    # posts and beams
    d.rectangle([5, 6, 8, 28], fill=WOOD)
    d.rectangle([23, 6, 26, 28], fill=WOOD)
    d.rectangle([5, 6, 26, 9], fill=WOOD)
    d.rectangle([5, 25, 26, 28], fill=WOOD)
    d.line([5, 9, 26, 9], fill=WOOD_DARK, width=1)
    # warp threads
    for x in range(10, 22, 2):
        d.line([x, 9, x, 25], fill=(226, 216, 188, 255), width=1)
    # half-woven cloth over the lower warp
    stripes = [(168, 62, 48, 255), (226, 216, 188, 255), (66, 84, 130, 255),
               (226, 216, 188, 255), (168, 62, 48, 255), (120, 96, 52, 255)]
    for i, col in enumerate(stripes):
        d.line([9, 19 + i, 22, 19 + i], fill=col, width=1)
    # heddle bar + shuttle
    d.line([9, 14, 22, 14], fill=WOOD_DARK, width=1)
    d.polygon([(12, 16), (19, 16), (17, 18), (14, 18)], fill=WOOD_LIGHT)


def sewing_machine(d, rng):
    # base, column, arm, head (profile facing left)
    d.rectangle([5, 25, 27, 28], fill=RED_DARK)
    d.rectangle([21, 10, 27, 25], fill=RED)
    d.rectangle([7, 10, 27, 14], fill=RED)
    d.rectangle([5, 10, 10, 21], fill=RED)
    d.rectangle([5, 14, 27, 15], fill=RED_DARK)  # arm shadow line
    # needle bar and needle
    d.line([7, 21, 7, 25], fill=STEEL_LIGHT, width=1)
    d.point((7, 26), fill=INK)
    d.rectangle([12, 23, 17, 24], fill=STEEL_LIGHT)  # stitch plate
    # handwheel
    d.ellipse([24, 16, 30, 22], outline=INK, width=1)
    d.point((27, 19), fill=INK)
    # spool pin with thread cone
    d.line([17, 7, 17, 10], fill=INK, width=1)
    d.polygon([(15, 7), (19, 7), (18, 9), (16, 9)], fill=(226, 216, 188, 255))
    # tension disc + stitch dial
    d.ellipse([11, 16, 13, 18], outline=STEEL_LIGHT)
    d.point((24, 12), fill=(250, 220, 120, 255))


def shed_stick(d, rng):
    # short thin flat wooden stick, diagonal
    d.line([6, 21, 25, 10], fill=WOOD_DARK, width=4)
    d.line([6, 21, 25, 10], fill=WOOD_LIGHT, width=2)
    d.point((7, 20), fill=WOOD_DARK)
    d.point((24, 11), fill=WOOD_DARK)


def pattern_set(d, rng):
    # three fanned paper sheets
    d.polygon([(8, 8), (24, 5), (26, 22), (10, 25)], fill=shade(PAPER, 0.88))
    d.polygon([(6, 10), (22, 7), (24, 24), (8, 27)], fill=shade(PAPER, 0.94))
    d.polygon([(5, 7), (20, 6), (21, 23), (6, 24)], fill=PAPER)
    d.polygon([(5, 7), (20, 6), (21, 23), (6, 24)], outline=INK)
    # dashed garment (bodice) outline on the top sheet
    pts = [(9, 10), (12, 9), (13, 11), (13, 15), (15, 19), (11, 20),
           (9, 16), (9, 12)]
    for i in range(len(pts)):
        x0, y0 = pts[i]
        x1, y1 = pts[(i + 1) % len(pts)]
        mx, my = (x0 + x1) // 2, (y0 + y1) // 2
        d.line([x0, y0, mx, my], fill=(120, 60, 50, 255), width=1)
    for px, py in ((18, 10), (18, 14), (17, 19)):  # pin heads
        d.point((px, py), fill=RED_BAND)


HIDE_PTS = [(10, 6), (20, 5), (24, 9), (23, 14), (26, 18), (22, 20),
            (23, 26), (17, 24), (12, 27), (9, 22), (6, 24), (7, 18),
            (4, 14), (8, 12), (7, 7), (12, 9)]


def tanning_hide(d, rng):
    d.polygon(HIDE_PTS, fill=(128, 90, 58, 255), outline=WOOD_DARK)
    # darker wet patches
    for cx, cy, r in ((13, 12, 3), (19, 18, 3), (12, 20, 2)):
        d.ellipse([cx - r, cy - r // 2, cx + r, cy + r // 2],
                  fill=(102, 68, 40, 255))
    # wet sheen streaks
    for x0, y0, x1, y1 in ((10, 10, 15, 9), (16, 15, 21, 14), (11, 17, 14, 16)):
        d.line([x0, y0, x1, y1], fill=(198, 162, 122, 255), width=1)


def tanning_pelt(d, rng):
    pts = [(11, 7), (20, 6), (23, 10), (22, 15), (24, 19), (20, 21),
           (21, 25), (16, 23), (12, 25), (10, 21), (7, 22), (8, 17),
           (6, 13), (9, 11), (9, 8), (13, 10)]
    d.polygon(pts, fill=(122, 102, 84, 255), outline=(84, 68, 54, 255))
    # fur strokes: short light ticks inside and fringing the edge
    for _ in range(46):
        x = rng.randint(9, 21)
        y = rng.randint(9, 23)
        d.line([x, y, x + rng.choice((-1, 0, 1)), y + 1],
               fill=(164, 144, 118, 255), width=1)
    for x, y in ((8, 12), (7, 17), (10, 23), (16, 24), (21, 22), (23, 18),
                 (22, 10), (14, 7)):
        d.line([x, y, x + (1 if x < 15 else -1), y + 1],
               fill=(164, 144, 118, 255), width=1)
    # pale flesh edge along one side
    d.line([9, 10, 8, 16], fill=(196, 150, 128, 255), width=1)


# ----------------------------------------------------------- protein foods

def milk_carton(d, rng):
    d.polygon([(11, 11), (16, 5), (21, 11)], fill=(232, 232, 236, 255))
    d.polygon([(11, 11), (16, 5), (21, 11)], outline=INK)
    d.rectangle([11, 11, 21, 28], fill=(246, 246, 248, 255), outline=INK)
    d.line([16, 5, 16, 11], fill=(200, 200, 206, 255), width=1)
    # blue fortified band + plus badge
    d.rectangle([11, 14, 21, 17], fill=(66, 110, 190, 255))
    d.rectangle([14, 20, 18, 24], outline=(66, 110, 190, 255))
    d.line([16, 21, 16, 23], fill=(66, 110, 190, 255), width=1)
    d.line([15, 22, 17, 22], fill=(66, 110, 190, 255), width=1)
    d.line([13, 26, 19, 26], fill=(178, 182, 190, 255), width=1)


def protein_bar(d, rng, peanut):
    wrap = (240, 234, 220, 255) if not peanut else (216, 142, 62, 255)
    band = (66, 110, 190, 255) if not peanut else (122, 76, 34, 255)
    d.rectangle([5, 13, 27, 20], fill=wrap, outline=INK)
    # crimped ends
    for x in (5, 6, 25, 26):
        d.line([x, 13, x, 20], fill=shade(wrap, 0.82), width=1)
    d.rectangle([13, 13, 19, 20], fill=band)
    if peanut:
        # peanut motif: two-lobed tan peanut
        d.ellipse([14, 14, 18, 17], fill=(232, 198, 148, 255))
        d.ellipse([14, 16, 18, 19], fill=(232, 198, 148, 255))
        d.point((16, 16), fill=(196, 156, 104, 255))
    else:
        # milk drop motif
        d.polygon([(16, 14), (18, 17), (17, 19), (15, 19), (14, 17)],
                  fill=(246, 246, 248, 255))
    # exposed bar tip at the right end
    d.rectangle([24, 15, 26, 18], fill=(198, 162, 110, 255))


def shaker(d, rng, fortified):
    fill = (232, 150, 160, 255) if not fortified else (206, 118, 150, 255)
    cap = (70, 72, 78, 255) if not fortified else (52, 54, 60, 255)
    d.rectangle([13, 4, 19, 7], fill=cap)               # spout cap
    d.rectangle([11, 7, 21, 10], fill=cap)              # screw lid
    if fortified:
        d.line([11, 9, 21, 9], fill=AMBER, width=1)     # gold band
    d.rectangle([11, 10, 21, 28], fill=(214, 220, 224, 255), outline=INK)
    d.rectangle([12, 14, 20, 27], fill=fill)            # shake inside
    d.line([13, 15, 13, 26], fill=(250, 250, 250, 255), width=1)
    for ty in (13, 17, 21, 25):                         # measure ticks
        d.point((20, ty), fill=INK)
    if fortified:                                       # plus badge
        d.ellipse([17, 22, 21, 26], fill=(246, 246, 248, 255), outline=INK)
        d.line([19, 23, 19, 25], fill=(62, 148, 74, 255), width=1)
        d.line([18, 24, 20, 24], fill=(62, 148, 74, 255), width=1)


def smoothie(d, rng):
    # cup with domed lid and straw
    d.polygon([(10, 10), (22, 10), (20, 28), (12, 28)], fill=(238, 240, 240, 255),
              outline=INK)
    d.polygon([(11, 14), (21, 14), (20, 27), (12, 27)], fill=(240, 182, 172, 255))
    d.arc([10, 5, 22, 13], start=180, end=360, fill=INK, width=1)
    d.line([10, 10, 22, 10], fill=INK, width=1)
    d.line([19, 3, 16, 9], fill=RED_BAND, width=2)      # straw
    # strawberry motif
    d.polygon([(14, 19), (18, 19), (16, 23)], fill=RED_BAND)
    d.line([15, 18, 17, 18], fill=(70, 132, 62, 255), width=1)
    d.point((16, 20), fill=(250, 220, 200, 255))


# ----------------------------------------------------------------- serums

SERUMS = {
    "serum_learning": ((70, 120, 220, 255), "book"),
    "serum_night_operations": ((38, 38, 46, 255), "moon"),
    "serum_regen": ((228, 238, 240, 255), "cross"),
    "serum_resilience": ((196, 200, 204, 255), "shield"),
    "serum_speed": ((240, 200, 60, 255), "bolt"),
    "serum_super_soldier": ((210, 60, 50, 255), "star"),
}


def serum_motif(d, key, cx, cy):
    if key == "book":
        d.rectangle([cx - 2, cy - 2, cx, cy + 2], outline=INK)
        d.rectangle([cx, cy - 2, cx + 2, cy + 2], outline=INK)
    elif key == "moon":
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=INK)
        d.ellipse([cx - 1, cy - 3, cx + 3, cy + 1], fill=(246, 246, 248, 255))
        d.point((cx + 2, cy + 2), fill=INK)
    elif key == "cross":
        d.line([cx, cy - 2, cx, cy + 2], fill=INK, width=1)
        d.line([cx - 2, cy, cx + 2, cy], fill=INK, width=1)
    elif key == "shield":
        d.polygon([(cx - 2, cy - 2), (cx + 2, cy - 2), (cx + 2, cy),
                   (cx, cy + 3), (cx - 2, cy)], outline=INK)
    elif key == "bolt":
        d.polygon([(cx + 1, cy - 3), (cx - 2, cy + 1), (cx, cy + 1),
                   (cx - 1, cy + 3), (cx + 2, cy - 1), (cx, cy - 1)],
                  fill=INK)
    elif key == "star":
        d.polygon([(cx, cy - 3), (cx + 1, cy - 1), (cx + 3, cy - 1),
                   (cx + 1, cy), (cx + 2, cy + 2), (cx, cy + 1),
                   (cx - 2, cy + 2), (cx - 1, cy), (cx - 3, cy - 1),
                   (cx - 1, cy - 1)], fill=INK)


def serum(d, rng, sid):
    liquid, key = SERUMS[sid]
    # flame-sealed tip and neck
    d.polygon([(15, 8), (17, 8), (16, 3)], fill=(206, 216, 222, 255),
              outline=(150, 160, 168, 255))
    d.rectangle([15, 8, 17, 12], fill=(206, 216, 222, 255),
                outline=(150, 160, 168, 255))
    # glass body
    d.rectangle([12, 12, 20, 28], fill=(214, 222, 226, 255),
                outline=(150, 160, 168, 255))
    # liquid fill
    d.rectangle([13, 16, 19, 27], fill=liquid)
    d.line([13, 16, 19, 16], fill=shade(liquid, 1.18), width=1)
    # glass highlight
    d.line([13, 13, 13, 27], fill=(250, 252, 252, 255), width=1)
    # white label with motif
    d.rectangle([12, 19, 20, 25], fill=(246, 246, 248, 255))
    d.line([12, 19, 20, 19], fill=(150, 160, 168, 255), width=1)
    d.line([12, 25, 20, 25], fill=(150, 160, 168, 255), width=1)
    serum_motif(d, key, 16, 22)


# ------------------------------------------------------------------- ammo

def bullet_factor(profile, t):
    if profile == "fmj":
        return 1.0 if t < 0.5 else max(0.14, (1 - ((t - 0.5) / 0.5) ** 2) ** 0.5)
    if profile == "jhp":
        return 1.0 if t < 0.78 else 0.82
    if profile == "lfn":
        return 0.96
    if profile == "sp":
        return 1.0 if t < 0.45 else 1.0 - 0.52 * ((t - 0.45) / 0.55) ** 1.4
    if profile == "ap":  # pointed penetrator
        return 1.0 if t < 0.38 else 1.0 - 0.86 * ((t - 0.38) / 0.62) ** 1.25
    if profile == "match":
        return 1.0 if t < 0.3 else max(0.12, (1 - ((t - 0.3) / 0.7) ** 2) ** 0.6)
    return 1.0


def cartridge(d, rng, case_w, case_h, bullet_h, profile, finish, col_fn,
              bottleneck=False, red_band=False):
    cx, base = 16, 29
    top = base - case_h
    brass = {"factory": BRASS, "bp": (172, 138, 70, 255),
             "matchhead": (216, 180, 94, 255),
             "reloaded": (188, 150, 72, 255)}[finish]
    x0, x1 = cx - case_w // 2, cx + case_w // 2
    if bottleneck:
        neck_w = max(4, int(case_w * 0.54))
        for i in range(case_h):
            y = base - 1 - i
            t = i / case_h
            if t < 0.52:
                w = case_w
            elif t < 0.78:
                w = case_w - (case_w - neck_w) * (t - 0.52) / 0.26
            else:
                w = neck_w
            hw = w / 2
            d.line([cx - hw, y, cx + hw, y], fill=brass, width=1)
            d.point((int(cx - hw) + 1, y), fill=BRASS_LIGHT)
        d.line([x0 - 1, base - 1, x1 + 1, base - 1], fill=BRASS_DARK, width=1)
        d.line([x0 - 1, base - 2, x1 + 1, base - 2], fill=brass, width=1)
    else:
        d.rectangle([x0, top, x1, base], fill=brass)
        d.line([x0, base - 3, x1, base - 3], fill=BRASS_DARK, width=1)
        d.rectangle([x0 - 1, base - 2, x1 + 1, base], fill=shade(brass, 0.94))
        if finish in ("factory", "matchhead"):
            d.line([x0 + 1, top + 1, x0 + 1, base - 4],
                   fill=BRASS_LIGHT, width=1)
        if finish == "reloaded":
            d.rectangle([x0, (top + base) // 2, x1, base - 3],
                        fill=shade(brass, 0.9))
            for _ in range(4):
                sx = rng.randint(x0 + 1, x1 - 2)
                sy = rng.randint(top + 2, base - 5)
                d.line([sx, sy, sx + 1, sy + 1], fill=(216, 192, 138, 255),
                       width=1)
    if finish == "bp":
        for _ in range(16):
            sx = rng.randint(x0, x1)
            sy = rng.randint(top + 1, base - 2)
            d.point((sx, sy), fill=SOOT)
        d.line([x0, top, x1, top], fill=SOOT, width=1)
    if finish == "matchhead":
        d.rectangle([x0, top, x1, top + 2], fill=AMBER)
    if red_band:
        d.line([x0, base - 6, x1, base - 6], fill=RED_BAND, width=1)
    # bullet, row by row from the case mouth up
    half_w = max(2, case_w // 2 - 1)
    for i in range(bullet_h):
        y = top - i
        t = i / (bullet_h - 1) if bullet_h > 1 else 1.0
        hw = max(1, round(half_w * bullet_factor(profile, t)))
        d.line([cx - hw, y, cx + hw, y], fill=col_fn(t), width=1)
    if profile == "jhp":  # hollow-point cavity
        d.point((cx, top - bullet_h + 1), fill=INK)
        d.point((cx, top - bullet_h + 2), fill=LEAD_DARK)


def col_copper(t):
    return COPPER


def col_jhp(t):
    return COPPER if t < 0.85 else LEAD


def col_lead(t):
    return LEAD


def col_sp(t):
    return COPPER if t < 0.45 else LEAD


def col_pen(t):
    return COPPER if t < 0.7 else (120, 124, 130, 255)


def col_tracer(t):
    return COPPER if t < 0.7 else RED_BAND


def col_mk211(t):
    if t > 0.68:
        return (62, 148, 74, 255)
    if t > 0.56:
        return (240, 240, 240, 255)
    return COPPER


def col_match(t):
    return (216, 140, 84, 255)


def col_ap(t):
    return COPPER if t < 0.74 else (30, 30, 34, 255)


AMMO = {
    # .380 ACP: small case
    "380_JHP": (8, 11, 8, "jhp", "factory", col_jhp, False, False),
    "380_p": (8, 11, 8, "jhp", "factory", col_jhp, False, True),
    "bp_380_FMJ": (8, 11, 8, "fmj", "bp", col_copper, False, False),
    "matchhead_380_FMJ": (8, 11, 8, "fmj", "matchhead", col_copper, False, False),
    "reloaded_380_FMJ": (8, 11, 8, "fmj", "reloaded", col_copper, False, False),
    # .38 Special: longer rimmed case
    "38_fmj": (8, 13, 8, "fmj", "factory", col_copper, False, False),
    "38_special_p": (8, 13, 8, "jhp", "factory", col_jhp, False, True),
    "bp_38_fmj": (8, 13, 8, "fmj", "bp", col_copper, False, False),
    "matchhead_38_fmj": (8, 13, 8, "fmj", "matchhead", col_copper, False, False),
    "reloaded_38_fmj": (8, 13, 8, "fmj", "reloaded", col_copper, False, False),
    # .45-70: long case, wide flat lead nose
    "4570_low": (10, 15, 8, "lfn", "factory", col_lead, False, False),
    "reloaded_4570_bp": (10, 15, 8, "lfn", "bp", col_lead, False, False),
    "reloaded_4570_low": (10, 15, 8, "lfn", "reloaded", col_lead, False, False),
    "reloaded_4570_pen": (10, 15, 9, "ap", "reloaded", col_pen, False, False),
    "reloaded_4570_sp": (10, 15, 8, "sp", "reloaded", col_sp, False, False),
    # .45 ACP: short fat case
    "45_acp": (9, 11, 8, "fmj", "factory", col_copper, False, False),
    "45_super": (9, 11, 8, "jhp", "factory", col_jhp, False, True),
    "bp_45_acp": (9, 11, 8, "fmj", "bp", col_copper, False, False),
    "matchhead_45_acp": (9, 11, 8, "fmj", "matchhead", col_copper, False, False),
    "reloaded_45_acp": (9, 11, 8, "fmj", "reloaded", col_copper, False, False),
    # .50 BMG: huge bottleneck case
    "50_incendiary": (12, 16, 11, "fmj", "factory", col_tracer, True, False),
    "50_mk211": (12, 16, 11, "fmj", "factory", col_mk211, True, False),
    "50bmg": (12, 16, 11, "fmj", "factory", col_copper, True, False),
    "50match": (12, 16, 12, "match", "matchhead", col_match, True, False),
    "50ss": (12, 16, 11, "ap", "factory", col_ap, True, False),
}


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "integrated_bio_robofac_xmarm":
        exo_arm(d, False)
    elif sid == "integrated_bio_robofac_xmarm_on":
        exo_arm(d, True)
    elif sid == "integrated_bio_robofac_xmback":
        exo_back(d, False)
    elif sid == "integrated_bio_robofac_xmback_on":
        exo_back(d, True)
    elif sid == "integrated_bio_robofac_xmleg":
        exo_leg(d, False)
    elif sid == "integrated_bio_robofac_xmleg_on":
        exo_leg(d, True)
    elif sid == "loom_frame":
        loom(d, rng)
    elif sid == "sewing_machine":
        sewing_machine(d, rng)
    elif sid == "shed_stick":
        shed_stick(d, rng)
    elif sid == "tailoring_pattern_set":
        pattern_set(d, rng)
    elif sid == "tanning_hide_active":
        tanning_hide(d, rng)
    elif sid == "tanning_pelt_active":
        tanning_pelt(d, rng)
    elif sid == "milk_fortified":
        milk_carton(d, rng)
    elif sid == "protein_bar_milk":
        protein_bar(d, rng, False)
    elif sid == "protein_bar_peanut":
        protein_bar(d, rng, True)
    elif sid == "protein_shake":
        shaker(d, rng, False)
    elif sid == "protein_shake_fortified":
        shaker(d, rng, True)
    elif sid == "protein_smoothie":
        smoothie(d, rng)
    elif sid in SERUMS:
        serum(d, rng, sid)
    elif sid in AMMO:
        cw, ch, bh, prof, fin, cf, bn, rb = AMMO[sid]
        cartridge(d, rng, cw, ch, bh, prof, fin, cf, bottleneck=bn,
                  red_band=rb)
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
    # contact-sheet preview: 7 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item81.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item81.png {prev.size}")


if __name__ == "__main__":
    main()
