#!/usr/bin/env python3
"""ITEM75: .410 + .50 BMG ammo variants, gun internals, bones/skulls,
pseudo furniture, muzzle devices -- CDDA Voxel coverage audit.

Families:

  .410 shotshells (8) -- one upright shell: red plastic hull over a
    brass base, the load showing at the crimp: birdshot = closed star
    crimp with pellet dots, scrap = open mouth of jagged metal
    chunks, slug = smooth lead dome.  Variants by finish: factory =
    clean bright brass and neat crimp; bp_ (black powder) = darker
    hull, soot smudging at the mouth, dull brass; reloaded_ =
    scuffed hull, two-tone mixed brass, crooked crimp.
  .50 BMG (8) -- one big bottleneck cartridge: tracer (the
    "incendiary" ids) = copper bullet with a red-orange tip; M33
    ball/FMJ = plain copper point; Match = longer, slimmer polished
    bullet; AP ("ss") = steel-gray bullet with a black tip band.
    bp_/reloaded_ finishes as above (reloaded_50bmg is the Match
    load, per its copy-from).
  Gun internals (8) -- small steel parts drawn as themselves:
    balanced recoil system (rod, sliding counterweight, spring),
    pristine condenser (finned can, shiny), drop-in auto sear
    (U bracket + T flapper on a pin), Glock sear plate (back plate
    with switch nub), lightning link (flat paddle), M240 / M249
    select-fire trigger groups (bulky vs slim housing), match
    trigger (slim precision blade with adjustment screw).
  Bones and skulls (8) -- long bones with knobby ends (demihuman
    slim, large huge), tainted bone meal as a gray-green speckled
    mound, a thin bone skewer; skulls: dambreaker (huge beaver skull
    with long orange incisors), tainted human (stained, warped,
    cracked), rabbit (small, long snout), skunk (small, flat, broad).
  Pseudo furniture (8) -- crude furniture stand-ins: brick oven
    (arched mouth), butchery tree (post, crossbar, hooks), butter
    churn (barrel + dasher plunger), fake butcher rack (frame with
    hooks), fake solar cooker (reflective panel), fake stove (box
    stove with pipe), large / medium surface (work slabs on legs,
    sized to name).
  Muzzle devices (8) -- horizontal tube profiles, thread stub at
    left: choke (short, knurled), compensator (top vent slots),
    homemade suppressor (welded pipe, tape band), 'solvent trap'
    filter suppressor (fat fluted oil-filter can), MP5SD integral
    (long fat smooth tube), muzzle brake (chunky, big side ports),
    shotgun suppressor (very large boxy tube), pistol suppressor
    (medium slim tube).

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "410shot_bird",
    "410shot_scrap",
    "410shot_slug",
    "bp_410shot_bird",
    "bp_410shot_scrap",
    "bp_410shot_slug",
    "reloaded_410shot_bird",
    "reloaded_410shot_slug",
    "bars",
    "condenser_pristine",
    "dias",
    "glocksear",
    "llink",
    "m240semi",
    "m249semi",
    "match_trigger",
    "bone_demihuman",
    "large_bone",
    "meal_bone_tainted",
    "skewer_bone",
    "skull_beaver_huge",
    "skull_human_tainted",
    "skull_rabbit",
    "skull_skunk",
    "bp_50_incendiary",
    "bp_50bmgfmj",
    "bp_50match",
    "bp_50ss",
    "reloaded_50_incendiary",
    "reloaded_50bmg",
    "reloaded_50bmgfmj",
    "reloaded_50ss",
    "brick_oven_pseudo",
    "butchery_tree_pseudo",
    "butter_churn",
    "fake_butcher_rack",
    "fake_solar_cooker",
    "fake_stove",
    "large_surface_pseudo",
    "medium_surface_pseudo",
    "choke",
    "compensator",
    "crafted_suppressor",
    "filter_suppressor",
    "mp5sd_suppressor",
    "muzzle_brake",
    "shot_suppressor",
    "suppressor_compact",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 44, 255)
STEEL = (150, 154, 160, 255)
STEEL_DARK = (96, 100, 106, 255)
STEEL_LIGHT = (196, 200, 206, 255)
BRASS = (198, 162, 70, 255)
BRASS_DULL = (158, 126, 56, 255)
BRASS_DARK = (122, 96, 40, 255)
COPPER = (184, 108, 62, 255)
HULL_RED = (192, 44, 38, 255)
HULL_RED_DARK = (138, 38, 34, 255)
HULL_RED_SCUF = (172, 52, 44, 255)
LEAD = (128, 130, 138, 255)
SOOT = (52, 48, 46, 255)
RED_TIP = (226, 60, 34, 255)
BONE = (232, 226, 204, 255)
BONE_SHADE = (198, 190, 162, 255)
BONE_DARK = (164, 156, 128, 255)
TAINT = (126, 140, 108, 255)
TAINT_DARK = (96, 110, 80, 255)
ORANGE_TEETH = (226, 140, 44, 255)
WOOD = (120, 86, 50, 255)
WOOD_DARK = (88, 60, 34, 255)
BRICK = (156, 84, 62, 255)
BRICK_DARK = (116, 60, 44, 255)
DARK = (30, 28, 30, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- .410 shells

def draw_shell(d, rng, load, variant):
    """Upright .410 shotshell, hull x12..19, brass base at bottom."""
    if variant == "factory":
        hull, brass = HULL_RED, BRASS
    elif variant == "bp":
        hull, brass = HULL_RED_DARK, BRASS_DULL
    else:  # reloaded
        hull, brass = HULL_RED_SCUF, BRASS
    hx0, hx1 = 12, 19
    # load at the mouth
    if load == "slug":  # smooth lead dome above the hull
        d.pieslice([hx0, 1, hx1, 9], start=180, end=360, fill=LEAD)
        d.rectangle([hx0, 5, hx1, 7], fill=LEAD)
        d.line([hx0 + 2, 4, hx0 + 2, 6], fill=shade(LEAD, 0.75), width=1)
        d.line([hx1 - 2, 4, hx1 - 2, 6], fill=shade(LEAD, 0.75), width=1)
        top = 7
    elif load == "scrap":  # open mouth full of jagged chunks
        d.rectangle([hx0, 4, hx1, 7], fill=DARK)
        for cx, ch in ((13, 3), (15, 4), (17, 3), (18, 2)):
            d.polygon([(cx, 7), (cx + 1, 7 - ch), (cx + 2, 7)],
                      fill=STEEL if (cx % 2) else STEEL_DARK)
        top = 7
    else:  # bird: closed star crimp, pellet dots around the centre
        d.rectangle([hx0, 4, hx1, 7], fill=shade(hull, 1.12))
        cx, cy = 15, 5
        for dx, dy in ((-2, 0), (2, 0), (-1, -1), (1, -1), (-1, 1), (1, 1)):
            d.point((cx + dx, cy + dy), fill=shade(hull, 0.7))
        d.point((cx, cy), fill=shade(hull, 0.55))
        if variant == "reloaded":  # crooked crimp: one flap lifted
            d.line([hx0, 4, hx0 + 3, 3], fill=shade(hull, 1.12), width=1)
        top = 7
    # hull body
    d.rectangle([hx0, top, hx1, 21], fill=hull)
    d.line([hx0 + 1, top + 1, hx0 + 1, 21], fill=shade(hull, 1.15), width=1)
    d.line([hx1 - 1, top + 1, hx1 - 1, 21], fill=shade(hull, 0.8), width=1)
    if variant == "bp":  # soot smudging down from the mouth
        for sx, sy in ((13, 9), (16, 11), (18, 8), (14, 13), (17, 14)):
            d.point((sx, sy), fill=SOOT)
            d.point((sx, sy + 1), fill=shade(hull, 0.6))
    if variant == "reloaded":  # scuffs
        for sx, sy in ((13, 12), (17, 16), (14, 18)):
            d.line([sx, sy, sx + 1, sy + 1], fill=shade(hull, 0.65), width=1)
    # brass base: two-tone for reloaded (mixed brass)
    d.rectangle([hx0, 22, hx1, 26], fill=brass)
    if variant == "reloaded":
        d.rectangle([hx0, 24, hx1, 26], fill=BRASS_DULL)
        d.line([hx0, 23, hx1, 23], fill=BRASS_DARK, width=1)
    else:
        d.line([hx0, 23, hx1, 23], fill=shade(brass, 0.8), width=1)
    d.line([hx0 + 1, 22, hx0 + 1, 26], fill=shade(brass, 1.15), width=1)
    # rim
    d.rectangle([hx0 - 1, 27, hx1 + 1, 28], fill=shade(brass, 0.85))
    d.point((15, 27), fill=BRASS_DARK)  # primer hint
    d.point((16, 27), fill=BRASS_DARK)


# ---------------------------------------------------------------- .50 BMG

def draw_bmg(d, rng, bullet, variant):
    """Big bottleneck .50 cartridge, upright, bullet at top."""
    case = BRASS if variant != "bp" else BRASS_DULL
    # bullet: y3..11, width by type
    if bullet == "match":  # long, slim, polished
        d.polygon([(14, 11), (15, 3), (16, 2), (17, 3), (18, 11)],
                  fill=shade(COPPER, 1.12))
        d.line([15, 4, 15, 10], fill=shade(COPPER, 1.3), width=1)
        d.point((16, 2), fill=INK)  # tiny meplat notch
    elif bullet == "tracer":
        d.polygon([(13, 11), (14, 5), (15, 3), (17, 3), (18, 5), (19, 11)],
                  fill=COPPER)
        d.polygon([(14, 5), (15, 3), (17, 3), (18, 5)], fill=RED_TIP)
        d.line([14, 6, 14, 10], fill=shade(COPPER, 1.25), width=1)
    elif bullet == "ss":  # AP: steel-gray with black tip band
        d.polygon([(13, 11), (14, 5), (15, 3), (17, 3), (18, 5), (19, 11)],
                  fill=STEEL)
        d.polygon([(14, 5), (15, 3), (17, 3), (18, 5)], fill=INK)
        d.line([14, 6, 14, 10], fill=STEEL_LIGHT, width=1)
    else:  # fmj ball: plain copper point
        d.polygon([(13, 11), (14, 5), (15, 3), (17, 3), (18, 5), (19, 11)],
                  fill=COPPER)
        d.line([14, 5, 14, 10], fill=shade(COPPER, 1.25), width=1)
        d.line([18, 5, 18, 10], fill=shade(COPPER, 0.8), width=1)
    # neck + shoulder
    d.rectangle([13, 11, 19, 13], fill=case)
    d.polygon([(13, 13), (19, 13), (20, 17), (12, 17)], fill=case)
    # body
    d.rectangle([12, 17, 20, 27], fill=case)
    d.line([13, 14, 13, 27], fill=shade(case, 1.15), width=1)
    d.line([19, 17, 19, 27], fill=shade(case, 0.8), width=1)
    if variant == "bp":  # soot at the mouth, dull streaks
        d.rectangle([13, 11, 19, 12], fill=SOOT)
        for sx, sy in ((14, 19), (17, 22), (15, 24)):
            d.point((sx, sy), fill=shade(case, 0.65))
    if variant == "reloaded":  # mixed brass tones + scratches
        d.rectangle([12, 21, 20, 23], fill=BRASS_DULL)
        d.line([14, 18, 15, 20], fill=BRASS_DARK, width=1)
        d.line([18, 24, 17, 26], fill=BRASS_DARK, width=1)
    # extractor groove + rim + primer
    d.line([12, 27, 20, 27], fill=BRASS_DARK, width=1)
    d.rectangle([12, 28, 20, 29], fill=shade(case, 0.85))
    d.point((15, 28), fill=BRASS_DARK)
    d.point((16, 28), fill=BRASS_DARK)


# ------------------------------------------------------------ gun internals

def draw_bars(d, rng):
    # reciprocating counterweight on a rod, spring behind
    d.line([4, 15, 27, 15], fill=STEEL_DARK, width=2)  # guide rod
    for i in range(5):  # coil spring zigzag, left
        x = 4 + i * 2
        d.line([x, 12, x + 1, 18], fill=STEEL, width=1)
        d.line([x + 1, 18, x + 2, 12], fill=STEEL, width=1)
    d.rectangle([15, 10, 24, 20], fill=STEEL)  # counterweight block
    d.rectangle([15, 10, 24, 20], outline=INK, width=1)
    d.line([17, 12, 17, 18], fill=STEEL_DARK, width=1)
    d.line([21, 12, 21, 18], fill=STEEL_DARK, width=1)
    d.rectangle([25, 13, 28, 17], fill=STEEL_DARK)  # buffer cap


def draw_condenser(d, rng):
    # finned cylindrical can, pristine = bright with a strong highlight
    d.rectangle([10, 8, 21, 24], fill=STEEL_LIGHT)
    d.rectangle([10, 8, 21, 24], outline=STEEL_DARK, width=1)
    for y in range(11, 24, 3):  # cooling fins
        d.line([9, y, 22, y], fill=STEEL, width=1)
    d.line([12, 9, 12, 23], fill=(238, 240, 244, 255), width=2)
    d.rectangle([13, 5, 15, 8], fill=STEEL)  # terminals
    d.rectangle([17, 5, 19, 8], fill=STEEL)
    d.point((14, 4), fill=INK)
    d.point((18, 4), fill=INK)
    d.rectangle([12, 24, 19, 26], fill=STEEL)  # base plate


def draw_dias(d, rng):
    # U-shaped bracket with a T flapper on a pin
    d.line([9, 8, 9, 22], fill=STEEL, width=2)
    d.line([22, 8, 22, 22], fill=STEEL, width=2)
    d.line([9, 22, 22, 22], fill=STEEL, width=2)
    d.rectangle([13, 13, 18, 20], fill=STEEL_DARK)  # flapper paddle
    d.line([15, 10, 15, 13], fill=STEEL_DARK, width=2)  # T stem
    d.line([12, 10, 18, 10], fill=STEEL_DARK, width=2)  # T bar
    d.ellipse([14, 15, 17, 18], fill=INK)  # pin


def draw_glocksear(d, rng):
    # slide back plate with a selector switch nub and sear tab
    d.rectangle([10, 9, 21, 20], fill=STEEL)
    d.rectangle([10, 9, 21, 20], outline=INK, width=1)
    d.line([12, 11, 19, 11], fill=STEEL_DARK, width=1)  # plate grooves
    d.line([12, 13, 19, 13], fill=STEEL_DARK, width=1)
    d.rectangle([14, 15, 17, 18], fill=INK)  # switch button
    d.point((15, 16), fill=STEEL_LIGHT)
    d.rectangle([14, 20, 17, 24], fill=STEEL_DARK)  # protruding tab


def draw_llink(d, rng):
    # flat lightning link: paddle with a stem, two prong tips
    d.polygon([(8, 20), (8, 12), (13, 9), (18, 12), (18, 20)],
              fill=STEEL)  # paddle head
    d.polygon([(8, 20), (8, 12), (13, 9), (18, 12), (18, 20)],
              outline=INK)
    d.rectangle([12, 20, 14, 27], fill=STEEL)  # stem
    d.line([12, 27, 12, 29], fill=STEEL, width=1)  # split prongs
    d.line([14, 27, 14, 29], fill=STEEL, width=1)
    d.point((13, 15), fill=STEEL_DARK)  # pivot hole


def draw_trigger_group(d, rng, bulky):
    # trigger housing block + curved blade + guard arc
    if bulky:  # M240: wide housing, two pins
        d.rectangle([8, 9, 24, 16], fill=STEEL_DARK)
        d.point((11, 12), fill=STEEL_LIGHT)
        d.point((21, 12), fill=STEEL_LIGHT)
        gy = 24
    else:  # M249: slimmer housing, one pin + safety lever
        d.rectangle([10, 9, 22, 15], fill=STEEL_DARK)
        d.point((13, 12), fill=STEEL_LIGHT)
        d.line([19, 10, 23, 7], fill=STEEL, width=2)  # safety lever
        gy = 23
    d.arc([9, 14, 23, 28], start=0, end=180, fill=STEEL_DARK, width=2)
    # curved trigger blade hanging from the housing
    d.line([15, 16, 14, 20], fill=STEEL, width=2)
    d.line([14, 20, 15, gy - 1], fill=STEEL, width=2)


def draw_match_trigger(d, rng):
    # slim precision trigger: housing nub, elegant blade, screw dot
    d.rectangle([12, 8, 20, 12], fill=STEEL)
    d.rectangle([12, 8, 20, 12], outline=INK, width=1)
    pts = [(16, 12), (14, 17), (14, 21), (16, 25)]
    d.line(pts, fill=STEEL_LIGHT, width=2)
    d.ellipse([15, 19, 18, 22], fill=BRASS)  # adjustment screw
    d.point((16, 20), fill=INK)
    d.arc([10, 15, 22, 27], start=0, end=180, fill=STEEL_DARK, width=1)


# ------------------------------------------------------------------- bones

def long_bone(d, thick, slim_curve=False):
    cx = 16
    w = thick // 2
    # knobby ends: two bumps top and bottom
    for cy, sgn in ((6, -1), (26, 1)):
        d.ellipse([cx - w - 2, cy - 3, cx - 1, cy + 3], fill=BONE)
        d.ellipse([cx + 1, cy - 3, cx + w + 2, cy + 3], fill=BONE)
        d.ellipse([cx - w - 2, cy - 3, cx - 1, cy + 3], outline=BONE_SHADE)
        d.ellipse([cx + 1, cy - 3, cx + w + 2, cy + 3], outline=BONE_SHADE)
    d.rectangle([cx - w, 6, cx + w, 26], fill=BONE)
    d.line([cx - w, 6, cx - w, 26], fill=BONE_SHADE, width=1)
    d.line([cx + w, 6, cx + w, 26], fill=BONE_SHADE, width=1)
    if slim_curve:  # demihuman: faint odd curve shading
        d.line([cx - 1, 10, cx - 2, 16], fill=BONE_SHADE, width=1)
        d.line([cx - 2, 16, cx - 1, 22], fill=BONE_SHADE, width=1)


def draw_meal(d, rng):
    # speckled mound of gray-green bone meal in a rough heap
    d.polygon([(6, 26), (10, 16), (16, 12), (22, 16), (26, 26)], fill=TAINT)
    d.polygon([(10, 16), (16, 12), (20, 17), (13, 20)], fill=shade(TAINT, 1.12))
    for _ in range(26):
        x = rng.randint(8, 24)
        y = rng.randint(15, 25)
        if y > 14 + abs(x - 16) * 0.8:
            d.point((x, y), fill=TAINT_DARK if rng.random() < 0.5 else BONE_SHADE)
    d.line([5, 27, 27, 27], fill=BONE_DARK, width=1)  # ground line


def draw_skewer(d, rng):
    # thin bone skewer on a diagonal, pointed at both ends
    d.polygon([(6, 25), (24, 7), (26, 6), (8, 26)], fill=BONE)
    d.line([8, 24, (24), 8], fill=BONE_SHADE, width=1)
    d.point((25, 5), fill=BONE_SHADE)  # tip
    d.point((5, 26), fill=BONE_SHADE)


def skull(d, cx, top, w, tone=BONE, teeth=None, sockets=2, snout=0,
          stain=False, crack=False, rng=None):
    """Front-view skull: cranium, sockets, nose, jaw with teeth."""
    h = int(w * 0.92)
    x0, x1 = cx - w // 2, cx + w // 2
    d.ellipse([x0, top, x1, top + h], fill=tone)
    d.ellipse([x0, top, x1, top + h], outline=BONE_SHADE, width=1)
    jaw_y = top + h - 2
    d.rectangle([cx - w // 4, jaw_y, cx + w // 4, jaw_y + 4 + snout],
                fill=tone)
    d.rectangle([cx - w // 4, jaw_y, cx + w // 4, jaw_y + 4 + snout],
                outline=BONE_SHADE, width=1)
    ey = top + h // 2
    for sgn in (-1, 1):  # eye sockets
        ex = cx + sgn * (w // 4)
        d.ellipse([ex - 2, ey - 2, ex + 2, ey + 2], fill=DARK)
    d.polygon([(cx, ey + 2), (cx - 1, ey + 5), (cx + 1, ey + 5)],
              fill=DARK)  # nasal cavity
    if teeth == "beaver":  # two huge orange incisors
        d.rectangle([cx - 3, jaw_y + 2, cx - 1, jaw_y + 10], fill=ORANGE_TEETH)
        d.rectangle([cx + 1, jaw_y + 2, cx + 3, jaw_y + 10], fill=ORANGE_TEETH)
        d.line([cx - 2, jaw_y + 9, cx - 2, jaw_y + 10], fill=shade(ORANGE_TEETH, 0.7), width=1)
        d.line([cx + 2, jaw_y + 9, cx + 2, jaw_y + 10], fill=shade(ORANGE_TEETH, 0.7), width=1)
    elif teeth == "small":
        for tx in range(cx - 3, cx + 3, 2):
            d.line([tx, jaw_y + 2, tx, jaw_y + 4], fill=BONE_SHADE, width=1)
    else:  # standard tooth row
        for tx in range(cx - w // 4 + 1, cx + w // 4, 2):
            d.line([tx, jaw_y + 1, tx, jaw_y + 3], fill=BONE_SHADE, width=1)
    if stain and rng is not None:  # tainted blotches + warped socket
        for _ in range(8):
            x = rng.randint(x0 + 1, x1 - 1)
            y = rng.randint(top + 1, jaw_y)
            d.point((x, y), fill=TAINT)
        d.point((cx + w // 4 + 1, ey), fill=DARK)  # warped extra hollow
    if crack:
        d.line([cx - 3, top + 1, cx - 1, top + 4], fill=BONE_DARK, width=1)
        d.line([cx - 1, top + 4, cx - 3, top + 7], fill=BONE_DARK, width=1)


# ---------------------------------------------------------- pseudo furniture

def draw_brick_oven(d, rng):
    d.rectangle([6, 14, 26, 28], fill=BRICK)
    for y in range(15, 28, 4):  # brick courses
        d.line([6, y, 26, y], fill=BRICK_DARK, width=1)
        for x in range(9 + (y % 8) // 4 * 3, 26, 6):
            d.point((x, y + 1), fill=BRICK_DARK)
    d.pieslice([11, 10, 21, 22], start=180, end=360, fill=BRICK)  # dome top
    d.arc([11, 16, 21, 28], start=180, end=360, fill=DARK, width=3)
    d.pieslice([12, 18, 20, 27], start=180, end=360, fill=DARK)  # mouth
    d.point((15, 24), fill=(240, 130, 40, 255))  # ember glow
    d.point((17, 25), fill=(240, 130, 40, 255))
    d.rectangle([14, 4, 18, 9], fill=BRICK_DARK)  # chimney stub


def draw_butchery_tree(d, rng):
    d.line([16, 4, 16, 28], fill=WOOD_DARK, width=3)  # trunk/post
    d.line([16, 8, 7, 5], fill=WOOD_DARK, width=2)  # branches
    d.line([16, 8, 25, 5], fill=WOOD_DARK, width=2)
    d.line([7, 12, 25, 12], fill=WOOD, width=2)  # crossbar
    for hx in (10, 16, 22):  # hooks
        d.arc([hx - 1, 12, hx + 2, 17], start=0, end=180, fill=STEEL, width=1)
    d.polygon([(14, 17), (19, 17), (18, 23), (15, 23)],
              fill=(168, 74, 66, 255))  # hanging meat chunk
    d.line([5, 28, 27, 28], fill=WOOD_DARK, width=1)


def draw_butter_churn(d, rng):
    d.polygon([(11, 12), (21, 12), (22, 27), (10, 27)], fill=WOOD)  # barrel
    d.polygon([(11, 12), (21, 12), (22, 27), (10, 27)], outline=WOOD_DARK)
    for y in (16, 21, 25):  # stave bands
        d.line([10, y, 22, y], fill=WOOD_DARK, width=1)
    d.rectangle([10, 10, 22, 12], fill=WOOD_DARK)  # lid
    d.line([16, 2, 16, 10], fill=WOOD_DARK, width=2)  # dasher staff
    d.line([12, 2, 20, 2], fill=WOOD_DARK, width=2)  # handle bar


def draw_butcher_rack(d, rng):
    for x in (7, 24):  # uprights
        d.line([x, 6, x, 27], fill=STEEL_DARK, width=2)
    d.line([7, 7, 24, 7], fill=STEEL, width=2)  # top rail
    d.line([7, 15, 24, 15], fill=STEEL, width=1)  # mid rail
    for hx in (10, 14, 18, 22):  # S hooks
        d.arc([hx - 1, 7, hx + 2, 12], start=0, end=180, fill=STEEL_LIGHT,
              width=1)
    d.line([7, 27, 24, 27], fill=STEEL_DARK, width=2)  # feet


def draw_solar_cooker(d, rng):
    # box cooker with a propped reflective lid panel
    d.polygon([(9, 20), (23, 20), (25, 27), (7, 27)], fill=STEEL_DARK)
    d.polygon([(9, 20), (23, 20), (25, 27), (7, 27)], outline=INK)
    d.rectangle([13, 22, 19, 25], fill=DARK)  # black pot inside
    d.polygon([(9, 20), (6, 8), (12, 6), (14, 18)], fill=STEEL_LIGHT)
    d.polygon([(9, 20), (6, 8), (12, 6), (14, 18)], outline=STEEL)
    for i in range(3):  # panel fold lines
        d.line([8 + i, 17 - i * 3, 13 + i, 15 - i * 3], fill=STEEL, width=1)
    d.point((24, 5), fill=(250, 214, 90, 255))  # sun glint
    d.point((25, 6), fill=(250, 214, 90, 255))
    d.point((23, 6), fill=(250, 214, 90, 255))


def draw_fake_stove(d, rng):
    d.rectangle([9, 12, 23, 27], fill=STEEL_DARK)  # stove box
    d.rectangle([9, 12, 23, 27], outline=INK, width=1)
    d.rectangle([12, 17, 20, 24], fill=DARK)  # fire door
    d.rectangle([12, 17, 20, 24], outline=STEEL, width=1)
    d.point((18, 20), fill=(240, 130, 40, 255))  # fire glow slit
    d.point((17, 20), fill=(240, 130, 40, 255))
    d.rectangle([19, 5, 22, 12], fill=STEEL_DARK)  # chimney pipe
    d.line([9, 27, 7, 29], fill=INK, width=2)  # legs
    d.line([23, 27, 25, 29], fill=INK, width=2)
    d.ellipse([13, 13, 19, 15], outline=STEEL, width=1)  # top plate ring


def draw_surface(d, rng, large):
    if large:
        x0, x1, y, leg = 3, 29, 12, 26
    else:
        x0, x1, y, leg = 7, 25, 14, 25
    d.rectangle([x0, y, x1, y + 3], fill=WOOD)  # slab top
    d.line([x0, y + 1, x1, y + 1], fill=shade(WOOD, 1.2), width=1)
    for lx in (x0 + 2, x1 - 3):  # legs
        d.line([lx, y + 3, lx, leg], fill=WOOD_DARK, width=2)
    d.line([x0 + 2, leg - 3, x1 - 3, leg - 3], fill=WOOD_DARK, width=1)
    # a couple of items resting on top
    d.rectangle([x0 + 4, y - 3, x0 + 8, y - 1], fill=STEEL)
    d.point((x1 - 6, y - 2), fill=BRICK)


# ------------------------------------------------------------ muzzle devices
# Horizontal profiles: thread stub at left (x3..6), body to the right,
# vertically centred around y16.

def muzzle_base(d, x_end, y0, y1, fill=STEEL_DARK):
    d.rectangle([3, 14, 6, 18], fill=STEEL)  # thread stub
    d.rectangle([6, y0, x_end, y1], fill=fill)
    d.line([6, y0, x_end, y0], fill=shade(fill, 1.25), width=1)


def draw_choke(d, rng):
    muzzle_base(d, 20, 13, 19)
    d.polygon([(20, 13), (23, 14), (23, 18), (20, 19)], fill=STEEL_DARK)
    for x in (9, 12, 15, 18):  # knurl rings
        d.line([x, 13, x, 19], fill=INK, width=1)


def draw_compensator(d, rng):
    muzzle_base(d, 25, 12, 20, fill=STEEL)
    for x in range(9, 23, 3):  # top vent slots
        d.rectangle([x, 12, x + 1, 14], fill=DARK)
    d.rectangle([23, 13, 26, 19], fill=STEEL_DARK)  # muzzle face
    d.point((24, 16), fill=DARK)


def draw_crafted_supp(d, rng):
    muzzle_base(d, 27, 12, 20, fill=STEEL_DARK)
    for x in (11, 19):  # weld beads
        for y in range(12, 21, 2):
            d.point((x, y), fill=STEEL_LIGHT)
    d.rectangle([14, 11, 17, 21], fill=(70, 66, 60, 255))  # tape wrap band
    d.line([6, 12, 27, 12], fill=STEEL, width=1)
    d.rectangle([27, 13, 28, 19], fill=INK)  # end cap
    d.point((27, 16), fill=DARK)


def draw_filter_supp(d, rng):
    d.rectangle([3, 14, 6, 18], fill=STEEL)  # thread stub
    d.rectangle([6, 13, 9, 19], fill=BRASS_DARK)  # adapter nut
    d.rectangle([9, 10, 24, 22], fill=STEEL_DARK)  # oil-filter canister
    d.line([9, 11, 24, 11], fill=STEEL, width=1)
    for x in range(19, 24, 2):  # fluted end
        d.line([x, 10, x, 22], fill=INK, width=1)
    d.rectangle([24, 12, 25, 20], fill=STEEL)
    d.point((24, 16), fill=DARK)  # bored-through hole


def draw_mp5sd_supp(d, rng):
    muzzle_base(d, 29, 11, 21, fill=STEEL_DARK)
    for x in (12, 20):  # subtle expansion-chamber rings
        d.line([x, 11, x, 21], fill=shade(STEEL_DARK, 0.8), width=1)
    d.line([6, 12, 29, 12], fill=STEEL, width=1)
    d.rectangle([29, 13, 30, 19], fill=INK)
    d.point((29, 16), fill=DARK)


def draw_muzzle_brake(d, rng):
    muzzle_base(d, 19, 12, 20, fill=STEEL)
    for x in (9, 14):  # big side ports
        d.rectangle([x, 14, x + 3, 18], fill=DARK)
    for x in (9, 12, 15):  # top slots
        d.point((x, 12), fill=DARK)
        d.point((x + 1, 12), fill=DARK)
    d.rectangle([19, 13, 21, 19], fill=STEEL_DARK)
    d.point((20, 16), fill=DARK)


def draw_shot_supp(d, rng):
    # very large boxy shotgun suppressor with panel segments
    d.rectangle([3, 14, 6, 18], fill=STEEL)
    d.rectangle([6, 9, 29, 23], fill=STEEL_DARK)
    d.line([6, 10, 29, 10], fill=STEEL, width=1)
    for x in (13, 20, 26):  # segment seams
        d.line([x, 9, x, 23], fill=INK, width=1)
    d.rectangle([27, 12, 29, 20], fill=shade(STEEL_DARK, 0.8))
    d.point((28, 16), fill=DARK)


def draw_supp_compact(d, rng):
    muzzle_base(d, 24, 13, 19, fill=STEEL_DARK)
    d.line([6, 14, 24, 14], fill=STEEL, width=1)
    d.rectangle([22, 12, 24, 20], fill=shade(STEEL_DARK, 0.85))  # end cap
    d.point((23, 16), fill=DARK)


# -------------------------------------------------------------------- main

SHELL_410 = {
    "410shot_bird": ("bird", "factory"),
    "410shot_scrap": ("scrap", "reloaded"),
    "410shot_slug": ("slug", "factory"),
    "bp_410shot_bird": ("bird", "bp"),
    "bp_410shot_scrap": ("scrap", "bp"),
    "bp_410shot_slug": ("slug", "bp"),
    "reloaded_410shot_bird": ("bird", "reloaded"),
    "reloaded_410shot_slug": ("slug", "reloaded"),
}

BMG_50 = {
    "bp_50_incendiary": ("tracer", "bp"),
    "bp_50bmgfmj": ("fmj", "bp"),
    "bp_50match": ("match", "bp"),
    "bp_50ss": ("ss", "bp"),
    "reloaded_50_incendiary": ("tracer", "reloaded"),
    "reloaded_50bmg": ("match", "reloaded"),
    "reloaded_50bmgfmj": ("fmj", "reloaded"),
    "reloaded_50ss": ("ss", "reloaded"),
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in SHELL_410:
        load, variant = SHELL_410[sid]
        draw_shell(d, rng, load, variant)
    elif sid in BMG_50:
        bullet, variant = BMG_50[sid]
        draw_bmg(d, rng, bullet, variant)
    elif sid == "bars":
        draw_bars(d, rng)
    elif sid == "condenser_pristine":
        draw_condenser(d, rng)
    elif sid == "dias":
        draw_dias(d, rng)
    elif sid == "glocksear":
        draw_glocksear(d, rng)
    elif sid == "llink":
        draw_llink(d, rng)
    elif sid == "m240semi":
        draw_trigger_group(d, rng, bulky=True)
    elif sid == "m249semi":
        draw_trigger_group(d, rng, bulky=False)
    elif sid == "match_trigger":
        draw_match_trigger(d, rng)
    elif sid == "bone_demihuman":
        long_bone(d, 2, slim_curve=True)
    elif sid == "large_bone":
        long_bone(d, 5)
    elif sid == "meal_bone_tainted":
        draw_meal(d, rng)
    elif sid == "skewer_bone":
        draw_skewer(d, rng)
    elif sid == "skull_beaver_huge":
        skull(d, 16, 4, 21, teeth="beaver", rng=rng)
    elif sid == "skull_human_tainted":
        skull(d, 16, 6, 16, tone=shade(BONE, 0.94), stain=True, crack=True,
              rng=rng)
    elif sid == "skull_rabbit":
        skull(d, 16, 9, 11, teeth="small", snout=2, rng=rng)
    elif sid == "skull_skunk":
        skull(d, 16, 10, 13, teeth="small", rng=rng)
    elif sid == "brick_oven_pseudo":
        draw_brick_oven(d, rng)
    elif sid == "butchery_tree_pseudo":
        draw_butchery_tree(d, rng)
    elif sid == "butter_churn":
        draw_butter_churn(d, rng)
    elif sid == "fake_butcher_rack":
        draw_butcher_rack(d, rng)
    elif sid == "fake_solar_cooker":
        draw_solar_cooker(d, rng)
    elif sid == "fake_stove":
        draw_fake_stove(d, rng)
    elif sid == "large_surface_pseudo":
        draw_surface(d, rng, large=True)
    elif sid == "medium_surface_pseudo":
        draw_surface(d, rng, large=False)
    elif sid == "choke":
        draw_choke(d, rng)
    elif sid == "compensator":
        draw_compensator(d, rng)
    elif sid == "crafted_suppressor":
        draw_crafted_supp(d, rng)
    elif sid == "filter_suppressor":
        draw_filter_supp(d, rng)
    elif sid == "mp5sd_suppressor":
        draw_mp5sd_supp(d, rng)
    elif sid == "muzzle_brake":
        draw_muzzle_brake(d, rng)
    elif sid == "shot_suppressor":
        draw_shot_supp(d, rng)
    elif sid == "suppressor_compact":
        draw_supp_compact(d, rng)
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
                       "preview_item75.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item75.png {prev.size}")


if __name__ == "__main__":
    main()
