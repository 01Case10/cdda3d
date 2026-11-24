#!/usr/bin/env python3
"""Batch ITEM42 (Phase 2g.4 scope extension): inactive bots.

34 ids from data/json/items/corpses/inactive_bots.json
(authoritative list embedded below; source list:
~/workspace/item-batches/42.txt). The file has no
copy-from chains -- every entry is drawn from its own
name/description/color/material.

All of these are deactivated machines carried as items,
so everything is drawn powered DOWN: dark lenses, unlit
status pips, folded or slumped forms, no glow.

The set, from the entries:

- Manhacks (fist-sized flying robots, aluminum/plastic):
  bot_manhack is "covered with whirring blades";
  bot_EMP_hack (cyan) carries an EMP grenade;
  bot_c4_hack (light gray) a C-4 charge; bot_flashbang_hack
  and bot_gasbomb_hack (dark gray) a flashbang / tear gas
  canister; bot_tazer_hack (cyan) a taser with prongs.
- Lab security drones (fist-sized flyers with "a scanner,
  lens and speaker"): six variants told apart only by
  their data color -- GR/GM light green, YM yellow,
  BM blue, BM2 cyan, BS dark gray.
- CROWS / turrets on folded mounts: bot_antimateriel
  (M2HB .50 BMG, red), bot_rifleturret (M249, green),
  bot_crows_m240 (M240, green), bot_turret (9mm SMG,
  light green), bot_turret_riot (40mm sponge gun, red,
  the bulkiest at 60 L).
- TALON UGVs (tracked, steel): bot_secubot with an
  M16A4; bot_talon_m202a1 with the 4-tube M202A1 rocket
  launcher; bot_secubot_liberty with firework tubes
  welded above the original gun.
- Humanoid service bots (steel, ~62.5 L): bot_nursebot
  (light blue, surgery assistant), bot_grocerybot
  (white), bot_grocerybot_busted (same bot, wrecked),
  bot_hazmatbot (cleaner bot, light green).
- Cyborgs (steel + flesh): bot_prototype_cyborg --
  "glimpses of humanity can still be seen in its empty
  eyes"; bot_broken_cyborg -- "the last shreds of its
  humanity still rotting away", a mass of rotting flesh
  and jagged metal.
- Northrop Dispatch (huge quadruped manhack deployers):
  bot_dispatch guard model (yellow), bot_dispatch_military
  (green, slashing arms).
- bot_turret_searchlight: milspec automated searchlight,
  lamp dark. bot_turret_speaker: automated loudspeaker.
- Yrax bots (gold / yrax_quartz): bot_yrax_trifacet is
  "a featureless quartz pyramid"; bot_yrax_triakis an
  "artfully carved piece of gold hued ceramics";
  bot_yrax_delta "a small piece of inscrutable alloy
  shaped like an arrow's broadhead"; bot_golden_monolith
  "a large golden monolith" with "carefully carved
  asymmetric edges".
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "bot_EMP_hack",
    "bot_antimateriel",
    "bot_broken_cyborg",
    "bot_c4_hack",
    "bot_crows_m240",
    "bot_dispatch",
    "bot_dispatch_military",
    "bot_flashbang_hack",
    "bot_gasbomb_hack",
    "bot_golden_monolith",
    "bot_grocerybot",
    "bot_grocerybot_busted",
    "bot_hazmatbot",
    "bot_lab_security_drone_BM",
    "bot_lab_security_drone_BM2",
    "bot_lab_security_drone_BS",
    "bot_lab_security_drone_GM",
    "bot_lab_security_drone_GR",
    "bot_lab_security_drone_YM",
    "bot_manhack",
    "bot_nursebot",
    "bot_prototype_cyborg",
    "bot_rifleturret",
    "bot_secubot",
    "bot_secubot_liberty",
    "bot_talon_m202a1",
    "bot_tazer_hack",
    "bot_turret",
    "bot_turret_riot",
    "bot_turret_searchlight",
    "bot_turret_speaker",
    "bot_yrax_delta",
    "bot_yrax_triakis",
    "bot_yrax_trifacet",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.")

STEEL = (148, 152, 158, 255)
DARK = (26, 28, 34, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)


def lens(d, cx, cy, r):
    """A powered-down optical lens: near-black, one dim
    catchlight, no glow."""
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(16, 18, 24, 255))
    d.ellipse([cx - r, cy - r, cx + r, cy + r],
              outline=(90, 98, 110, 255), width=1)
    d.point((cx - max(1, r // 2), cy - max(1, r // 2)),
            fill=(110, 130, 150, 255))


def pip(d, x, y):
    """An unlit status pip."""
    d.rectangle([x, y, x + 1, y + 1], fill=(52, 54, 58, 255))


def rivets(d, pts, col=(70, 74, 82, 255)):
    for (x, y) in pts:
        d.point((x, y), fill=col)


def rotor(d, cx, cy, r=3):
    """A folded/stopped rotor disc at a corner."""
    d.ellipse([cx - r, cy - r, cx + r, cy + r],
              fill=(58, 62, 70, 255))
    d.line([(cx - r, cy), (cx + r, cy)], fill=(96, 102, 112, 255),
           width=1)
    d.line([(cx, cy - r), (cx, cy + r)], fill=(96, 102, 112, 255),
           width=1)
    d.point((cx, cy), fill=(140, 146, 156, 255))


# ---------------------------------------------------------------- manhacks

def draw_hack(d, rng, body, payload):
    """Fist-sized flying robot, dormant: rounded hull,
    four stopped corner rotors, dark lens, payload on top."""
    # corner rotors (behind hull)
    for (x, y) in ((7, 8), (25, 8), (7, 24), (25, 24)):
        rotor(d, x, y)
    # hull
    d.ellipse([8, 9, 24, 25], fill=body)
    d.ellipse([10, 10, 20, 18], fill=shade(body, 1.14))
    d.ellipse([9, 19, 23, 24], fill=shade(body, 0.82))
    d.arc([8, 9, 24, 25], start=20, end=160, fill=shade(body, 0.7),
          width=1)
    lens(d, 16, 18, 3)
    pip(d, 21, 12)
    if payload == "blades":
        # whirring blades folded around the hull rim
        blade = (176, 182, 190, 255)
        for (pts) in (
                [(4, 12), (8, 13), (5, 17)],
                [(28, 12), (24, 13), (27, 17)],
                [(4, 20), (8, 20), (5, 16)],
                [(28, 20), (24, 20), (27, 16)],
                [(13, 5), (15, 9), (18, 5)],
                [(13, 28), (15, 24), (18, 28)]):
            d.polygon(pts, fill=blade)
            d.line(pts + [pts[0]], fill=shade(blade, 0.7), width=1)
    elif payload == "emp":
        # EMP grenade: coil-wrapped canister on top
        d.rectangle([12, 2, 20, 9], fill=(120, 128, 138, 255))
        for yy in (3, 5, 7):
            d.line([(12, yy), (20, yy)], fill=(64, 150, 170, 255),
                   width=1)
        d.rectangle([14, 1, 18, 2], fill=(90, 96, 106, 255))
    elif payload == "c4":
        # C-4 demolition charge: tan block strapped on top
        d.rectangle([11, 2, 21, 9], fill=(196, 178, 138, 255))
        d.rectangle([11, 2, 21, 4], fill=(178, 158, 118, 255))
        d.line([(16, 2), (16, 9)], fill=(90, 84, 70, 255), width=1)
        d.point((18, 6), fill=(150, 60, 50, 255))  # dead detonator cap
    elif payload == "flash":
        # flashbang: silver cylinder on top
        d.rectangle([13, 1, 19, 9], fill=(188, 192, 198, 255))
        d.rectangle([13, 1, 19, 3], fill=(150, 154, 162, 255))
        d.line([(13, 6), (19, 6)], fill=(120, 124, 132, 255), width=1)
        d.rectangle([15, 0, 17, 1], fill=(110, 114, 122, 255))
    elif payload == "gas":
        # tear gas canister with nozzle and vent holes
        d.rectangle([12, 2, 20, 9], fill=(104, 110, 118, 255))
        d.rectangle([14, 0, 18, 2], fill=(140, 146, 154, 255))
        for xx in (13, 16, 19):
            d.point((xx, 4), fill=(40, 42, 48, 255))
        d.line([(12, 8), (20, 8)], fill=(70, 76, 84, 255), width=1)
    elif payload == "tazer":
        # taser: two prongs at the front, coil marks
        prong = (200, 206, 214, 255)
        d.line([(11, 26), (10, 30)], fill=prong, width=1)
        d.line([(21, 26), (22, 30)], fill=prong, width=1)
        d.line([(13, 11), (19, 11)], fill=shade(body, 0.65), width=1)
        d.line([(13, 13), (19, 13)], fill=shade(body, 0.65), width=1)


# ---------------------------------------------------------------- drones

def draw_drone(d, rng, body, accent):
    """Lab security drone: hull with scanner bar, big dark
    lens, speaker grille; four stopped rotors."""
    for (x, y) in ((6, 7), (26, 7), (6, 25), (26, 25)):
        rotor(d, x, y)
    d.rectangle([8, 8, 24, 24], fill=body)
    d.rectangle([8, 8, 24, 11], fill=shade(body, 1.15))
    d.rectangle([8, 21, 24, 24], fill=shade(body, 0.8))
    d.rectangle([8, 8, 24, 24], outline=shade(body, 0.65), width=1)
    # scanner bar (dark, unlit)
    d.rectangle([11, 9, 21, 10], fill=(44, 48, 54, 255))
    lens(d, 16, 15, 4)
    # speaker grille: dot grid
    for yy in (20, 22):
        for xx in (12, 14, 16, 18, 20):
            d.point((xx, yy), fill=shade(body, 0.55))
    d.rectangle([22, 12, 23, 13], fill=accent)  # variant stripe
    pip(d, 9, 12)


# ---------------------------------------------------------------- turrets

def draw_turret_base(d, rng, body):
    """Folded tripod mount + receiver box, powered down."""
    leg = shade(body, 0.6)
    d.line([(10, 22), (5, 29)], fill=leg, width=2)
    d.line([(22, 22), (27, 29)], fill=leg, width=2)
    d.line([(16, 23), (16, 29)], fill=leg, width=2)
    d.rectangle([8, 12, 24, 22], fill=body)
    d.rectangle([8, 12, 24, 14], fill=shade(body, 1.13))
    d.rectangle([8, 20, 24, 22], fill=shade(body, 0.82))
    d.rectangle([8, 12, 24, 22], outline=shade(body, 0.62), width=1)
    lens(d, 12, 17, 2)
    pip(d, 21, 13)


def draw_turret(d, rng, body, kind):
    barrel = (52, 54, 60, 255)
    if kind == "riot":
        # bulkiest hull, short wide sponge-gun muzzle
        d.line([(9, 23), (4, 29)], fill=shade(body, 0.6), width=2)
        d.line([(23, 23), (28, 29)], fill=shade(body, 0.6), width=2)
        d.rectangle([6, 11, 26, 23], fill=body)
        d.rectangle([6, 11, 26, 13], fill=shade(body, 1.13))
        d.rectangle([6, 21, 26, 23], fill=shade(body, 0.82))
        d.rectangle([6, 11, 26, 23], outline=shade(body, 0.62),
                    width=1)
        d.rectangle([13, 5, 20, 11], fill=shade(body, 0.9))
        d.ellipse([13, 2, 20, 7], fill=(30, 30, 34, 255))
        d.ellipse([15, 3, 18, 6], fill=(52, 52, 56, 255))
        lens(d, 10, 17, 2)
        pip(d, 23, 12)
        return
    draw_turret_base(d, rng, body)
    if kind == "smg":
        d.rectangle([14, 7, 18, 12], fill=shade(body, 0.92))
        d.line([(16, 7), (16, 3)], fill=barrel, width=2)
        d.ellipse([19, 14, 24, 20], fill=shade(body, 0.75))  # drum
        d.ellipse([20, 15, 23, 19], outline=shade(body, 0.55),
                  width=1)
    elif kind == "antimateriel":
        # long heavy .50 barrel with muzzle brake, big ammo box
        d.line([(16, 12), (16, 2)], fill=barrel, width=3)
        d.rectangle([14, 2, 18, 5], fill=(40, 42, 46, 255))
        d.rectangle([3, 14, 8, 21], fill=shade(body, 0.78))
        d.rectangle([3, 14, 8, 16], fill=shade(body, 0.95))
        d.line([(18, 10), (22, 8)], fill=barrel, width=1)  # charging handle
    elif kind == "rifle":
        # M249: medium barrel, box magazine beneath
        d.line([(16, 12), (16, 4)], fill=barrel, width=2)
        d.rectangle([15, 3, 17, 5], fill=(40, 42, 46, 255))
        d.rectangle([13, 18, 20, 22], fill=shade(body, 0.72))
        d.line([(20, 12), (23, 9)], fill=barrel, width=1)  # bipod folded
    elif kind == "m240":
        # M240: medium barrel, ammo belt draped into the side
        d.line([(16, 12), (16, 4)], fill=barrel, width=2)
        d.rectangle([14, 6, 18, 8], fill=shade(body, 0.9))  # gas block
        belt = (168, 148, 92, 255)
        for i in range(4):
            d.rectangle([4 + i, 13 + i * 2, 6 + i, 14 + i * 2],
                        fill=belt)
        d.rectangle([3, 19, 8, 22], fill=shade(body, 0.72))


# ---------------------------------------------------------------- TALONs

def draw_talon(d, rng, body, kind):
    """Tracked UGV: two track units, hull, sensor mast with
    dark ball, weapon mounted on top. Powered down."""
    track = (44, 46, 52, 255)
    for x0 in (4, 20):
        d.rectangle([x0, 22, x0 + 8, 28], fill=track)
        d.rectangle([x0, 22, x0 + 8, 28],
                    outline=(70, 74, 82, 255), width=1)
        for xx in (x0 + 2, x0 + 5):
            d.ellipse([xx - 1, 24, xx + 1, 27],
                      fill=(88, 94, 104, 255))
    d.rectangle([6, 16, 26, 23], fill=body)
    d.rectangle([6, 16, 26, 18], fill=shade(body, 1.13))
    d.rectangle([6, 16, 26, 23], outline=shade(body, 0.62), width=1)
    # sensor mast + dark sensor ball
    d.line([(23, 16), (23, 10)], fill=shade(body, 0.7), width=2)
    d.ellipse([20, 6, 26, 12], fill=shade(body, 0.9))
    lens(d, 23, 9, 2)
    pip(d, 8, 17)
    barrel = (52, 54, 60, 255)
    if kind == "rifle":
        d.line([(8, 16), (10, 6)], fill=barrel, width=2)
        d.rectangle([9, 11, 12, 15], fill=shade(body, 0.75))
        d.point((7, 5), fill=barrel)
    elif kind == "rocket":
        # M202A1: 2x2 tube cluster
        d.rectangle([6, 5, 17, 14], fill=shade(body, 0.85))
        d.rectangle([6, 5, 17, 14], outline=shade(body, 0.6), width=1)
        for (x, y) in ((8, 7), (13, 7), (8, 11), (13, 11)):
            d.ellipse([x, y, x + 3, y + 3], fill=(24, 24, 28, 255))
            d.ellipse([x, y, x + 3, y + 3],
                      outline=(100, 106, 116, 255), width=1)
    elif kind == "liberty":
        # original rifle, with firework tubes welded above it
        d.line([(8, 16), (10, 8)], fill=barrel, width=2)
        tube_cols = [(178, 60, 52), (226, 226, 230), (70, 90, 170)]
        for i, c in enumerate(tube_cols):
            x = 11 + i * 3
            d.rectangle([x, 2, x + 2, 9], fill=c + (255,))
            d.rectangle([x, 2, x + 2, 3],
                        fill=shade(c + (255,), 1.2))
            d.line([(x + 1, 9), (x + 1, 11)],
                   fill=(120, 110, 90, 255), width=1)  # weld strut


# ---------------------------------------------------------------- humanoids

def draw_humanoid(d, rng, kind):
    """Slumped powered-down humanoid: head tipped forward,
    dark faceplate, hanging arms, column legs."""
    if kind == "nurse":
        skin = (196, 216, 232, 255)   # light blue-white shell
        accent = (198, 52, 48, 255)   # red cross
        trim = (120, 160, 190, 255)
    elif kind in ("grocery", "grocery_busted"):
        skin = (228, 228, 230, 255)   # white shell
        accent = (90, 140, 90, 255)
        trim = (150, 152, 156, 255)
    elif kind == "hazmat":
        skin = (168, 206, 150, 255)   # light green
        accent = (240, 200, 70, 255)  # caution yellow
        trim = (110, 150, 96, 255)
    busted = kind == "grocery_busted"
    tilt = 2 if busted else 1
    # legs / base column
    d.rectangle([12, 24, 15, 30], fill=shade(skin, 0.78))
    d.rectangle([18, 24, 21, 30], fill=shade(skin, 0.78))
    d.rectangle([10, 30, 23, 31], fill=shade(skin, 0.6))
    # torso
    d.rectangle([10, 12, 23, 25], fill=skin)
    d.rectangle([10, 12, 23, 14], fill=shade(skin, 1.06))
    d.rectangle([10, 12, 23, 25], outline=shade(skin, 0.7), width=1)
    # arms hanging
    d.line([(9, 13), (7, 22)], fill=shade(skin, 0.85), width=3)
    if busted:
        d.line([(24, 13), (27, 20)], fill=shade(skin, 0.8), width=3)
        d.line([(27, 20), (26, 24)], fill=shade(skin, 0.65), width=2)
    else:
        d.line([(24, 13), (26, 22)], fill=shade(skin, 0.85), width=3)
    # head, tipped forward
    hx = 12 + tilt
    d.rectangle([hx, 3, hx + 9, 11], fill=skin)
    d.rectangle([hx, 3, hx + 9, 11], outline=shade(skin, 0.7),
                width=1)
    d.rectangle([hx + 1, 6, hx + 8, 9], fill=DARK)  # dark faceplate
    d.point((hx + 2, 7), fill=(110, 130, 150, 255))
    pip(d, hx + 7, 4)
    if kind == "nurse":
        # nurse cap band + red cross on chest
        d.rectangle([hx, 3, hx + 9, 4], fill=(232, 232, 236, 255))
        d.rectangle([15, 16, 18, 21], fill=accent)
        d.rectangle([14, 17, 19, 19], fill=accent)
        d.rectangle([11, 22, 22, 23], fill=trim)  # tray ledge
    elif kind in ("grocery", "grocery_busted"):
        # basket grid on the torso front
        for xx in (13, 16, 19):
            d.line([(xx, 15), (xx, 22)], fill=trim, width=1)
        for yy in (17, 20):
            d.line([(11, yy), (22, yy)], fill=trim, width=1)
        d.rectangle([11, 14, 22, 15], fill=accent)
    elif kind == "hazmat":
        # scrub brush bar across the feet + caution stripe
        d.rectangle([10, 18, 23, 19], fill=accent)
        for xx in range(11, 23, 2):
            d.line([(xx, 27), (xx, 30)], fill=(120, 104, 70, 255),
                   width=1)
        d.rectangle([10, 26, 23, 27], fill=shade(skin, 0.7))
    if busted:
        # cracks, dents, scuffs
        crack = (70, 70, 74, 255)
        d.line([(13, 13), (15, 17), (14, 20)], fill=crack, width=1)
        d.line([(20, 14), (19, 18)], fill=crack, width=1)
        d.line([(hx + 2, 3), (hx + 4, 6)], fill=crack, width=1)
        for _ in range(14):
            x = rng.randint(8, 25)
            y = rng.randint(12, 29)
            d.point((x, y), fill=(120, 116, 108, 255))
        d.rectangle([17, 24, 20, 25], fill=(96, 92, 86, 255))  # dent


def draw_cyborg(d, rng, broken):
    """Humanoid of flesh and steel, slumped and dark-eyed.
    The prototype is clean chrome and pale flesh; the
    broken one is rotting flesh and jagged metal."""
    if broken:
        flesh = (146, 124, 96, 255)
        rot = (104, 116, 66, 255)
        metal = (128, 132, 138, 255)
    else:
        flesh = (206, 172, 148, 255)
        rot = None
        metal = (186, 192, 200, 255)
    # legs: one flesh, one metal (prototype: both plated shins)
    d.rectangle([12, 24, 15, 30], fill=shade(flesh, 0.9))
    d.rectangle([18, 24, 21, 30], fill=metal)
    d.rectangle([10, 30, 23, 31], fill=shade(metal, 0.65))
    # torso: flesh with metal plating
    d.rectangle([10, 12, 23, 25], fill=flesh)
    d.polygon([(10, 12), (17, 12), (14, 25), (10, 25)],
              fill=metal)  # chest plate
    d.line([(17, 12), (14, 25)], fill=shade(metal, 0.7), width=1)
    d.rectangle([19, 16, 23, 20], fill=shade(flesh, 0.85))
    rivets(d, [(11, 13), (16, 13), (12, 23)],
           col=shade(metal, 0.6))
    # arms: left flesh hanging, right bare metal struts
    d.line([(9, 13), (7, 22)], fill=shade(flesh, 0.9), width=3)
    d.line([(24, 13), (26, 18)], fill=metal, width=2)
    d.line([(26, 18), (25, 23)], fill=shade(metal, 0.8), width=1)
    d.point((25, 24), fill=shade(metal, 0.7))
    # head: half flesh, half plate; empty dark eyes
    d.rectangle([12, 3, 21, 11], fill=flesh)
    d.polygon([(17, 3), (21, 3), (21, 11), (18, 11)], fill=metal)
    d.rectangle([12, 3, 21, 11], outline=shade(flesh, 0.7), width=1)
    d.rectangle([13, 6, 15, 8], fill=DARK)   # empty eye socket
    d.rectangle([18, 6, 20, 8], fill=DARK)   # dark optic
    d.line([(14, 10), (19, 10)], fill=shade(flesh, 0.65), width=1)
    if broken:
        # rotting patches, jagged tears, exposed ribs
        for (x, y, w, h) in ((19, 17, 3, 2), (11, 20, 2, 2),
                             (20, 22, 3, 2), (13, 26, 2, 2)):
            d.rectangle([x, y, x + w - 1, y + h - 1], fill=rot)
        for yy in (17, 19, 21):
            d.line([(20, yy), (23, yy)], fill=(210, 204, 190, 255),
                   width=1)  # ribs
        d.polygon([(10, 12), (13, 12), (10, 16)],
                  fill=shade(metal, 0.75))  # jagged shard
        d.line([(13, 4), (15, 7)], fill=(70, 66, 60, 255), width=1)
        for _ in range(10):
            x = rng.randint(10, 23)
            y = rng.randint(12, 29)
            d.point((x, y), fill=rot)
    else:
        # clean seams, a hairline of real hair
        d.rectangle([12, 3, 21, 4], fill=(120, 92, 70, 255))
        d.line([(12, 9), (21, 9)], fill=shade(flesh, 0.8), width=1)
        pip(d, 20, 4)


# ---------------------------------------------------------------- dispatch

def draw_dispatch(d, rng, body, military):
    """Northrop Dispatch: huge quadruped assembler, folded
    legs, top hatch, manipulator arms at the front."""
    leg = shade(body, 0.62)
    # folded legs
    for (x0, x1) in ((6, 3), (12, 10), (20, 22), (26, 29)):
        d.line([(x0, 22), (x1, 29)], fill=leg, width=3)
        d.line([(x1 - 1, 29), (x1 + 2, 29)], fill=leg, width=1)
    # bulky hull
    d.rectangle([5, 9, 27, 23], fill=body)
    d.rectangle([5, 9, 27, 12], fill=shade(body, 1.13))
    d.rectangle([5, 20, 27, 23], fill=shade(body, 0.82))
    d.rectangle([5, 9, 27, 23], outline=shade(body, 0.6), width=1)
    # top hatch (where manhacks deploy from)
    d.rectangle([12, 5, 20, 9], fill=shade(body, 0.9))
    d.rectangle([12, 5, 20, 9], outline=shade(body, 0.62), width=1)
    d.line([(16, 5), (16, 9)], fill=shade(body, 0.62), width=1)
    # sensor cluster, dark
    lens(d, 9, 16, 2)
    lens(d, 23, 16, 2)
    pip(d, 16, 13)
    # panel seams
    d.line([(5, 16), (27, 16)], fill=shade(body, 0.7), width=1)
    if military:
        # slashing blade arms folded at the front
        blade = (178, 184, 192, 255)
        d.polygon([(6, 23), (3, 28), (8, 27)], fill=blade)
        d.polygon([(26, 23), (29, 28), (24, 27)], fill=blade)
        d.rectangle([13, 18, 19, 20], fill=shade(body, 0.72))
        rivets(d, [(7, 11), (25, 11), (7, 21), (25, 21)],
               col=shade(body, 0.55))
    else:
        # guard model's blocky manipulator stubs
        d.rectangle([6, 23, 9, 27], fill=shade(body, 0.78))
        d.rectangle([23, 23, 26, 27], fill=shade(body, 0.78))


# ---------------------------------------------------------------- fixtures

def draw_searchlight(d, rng):
    body = (150, 178, 138, 255)  # light green steel
    d.rectangle([9, 24, 23, 29], fill=shade(body, 0.85))
    d.rectangle([9, 24, 23, 29], outline=shade(body, 0.6), width=1)
    d.line([(11, 24), (8, 30)], fill=shade(body, 0.6), width=2)
    d.line([(21, 24), (24, 30)], fill=shade(body, 0.6), width=2)
    # yoke
    d.line([(10, 12), (10, 24)], fill=shade(body, 0.75), width=2)
    d.line([(22, 12), (22, 24)], fill=shade(body, 0.75), width=2)
    # the big lamp, OFF: dark glass in a steel rim
    d.ellipse([7, 4, 25, 22], fill=shade(body, 0.95))
    d.ellipse([7, 4, 25, 22], outline=shade(body, 0.6), width=1)
    d.ellipse([9, 6, 23, 20], fill=(30, 34, 40, 255))
    d.ellipse([11, 8, 17, 13], fill=(52, 60, 70, 255))
    d.point((12, 9), fill=(130, 150, 165, 255))
    # rear housing + cable
    d.rectangle([14, 20, 18, 24], fill=shade(body, 0.8))
    d.arc([18, 22, 30, 32], start=90, end=200,
          fill=(60, 62, 66, 255), width=1)
    pip(d, 20, 26)


def draw_speaker(d, rng):
    body = (150, 178, 138, 255)
    d.rectangle([8, 23, 24, 29], fill=shade(body, 0.85))
    d.rectangle([8, 23, 24, 29], outline=shade(body, 0.6), width=1)
    d.line([(10, 23), (7, 30)], fill=shade(body, 0.6), width=2)
    d.line([(22, 23), (25, 30)], fill=shade(body, 0.6), width=2)
    d.rectangle([15, 14, 17, 23], fill=shade(body, 0.75))
    # two horn loudspeakers, mouths toward the viewer
    horn = shade(body, 0.95)
    for (x0, x1) in ((4, 14), (18, 28)):
        d.polygon([(x0 + 3, 12), (x1 - 3, 12), (x1, 4), (x0, 4)],
                  fill=horn)
        d.rectangle([x0, 3, x1, 5], fill=(40, 42, 48, 255))
        d.ellipse([x0 + 1, 3, x1 - 1, 5], fill=(58, 62, 68, 255))
        d.line([(x0 + 3, 12), (x1 - 3, 12)],
               fill=shade(body, 0.65), width=1)
    # grille dots on the base + dead control dial
    for xx in (11, 14, 17, 20):
        d.point((xx, 26), fill=shade(body, 0.55))
    d.ellipse([20, 24, 23, 27], outline=shade(body, 0.55), width=1)
    pip(d, 9, 24)


# ---------------------------------------------------------------- yrax

GOLD = (206, 168, 74, 255)


def draw_trifacet(d, rng):
    # featureless quartz pyramid, pale cyan crystal
    quartz = (196, 226, 228, 255)
    d.polygon([(16, 4), (28, 26), (16, 26)], fill=shade(quartz, 0.92))
    d.polygon([(16, 4), (4, 26), (16, 26)], fill=quartz)
    d.line([(16, 4), (16, 26)], fill=shade(quartz, 0.78), width=1)
    d.line([(4, 26), (28, 26)], fill=shade(quartz, 0.7), width=1)
    d.line([(16, 4), (9, 18)], fill=shade(quartz, 1.06), width=1)
    d.rectangle([3, 26, 29, 27], fill=shade(quartz, 0.66))


def draw_triakis(d, rng):
    # gold-hued ceramic polyhedron: a core with triangular
    # pyramids on its faces, artfully carved
    cx, cy = 16, 16
    spikes = []
    for i in range(8):
        ang = math.radians(i * 45 + 22)
        x = int(cx + 12 * math.cos(ang))
        y = int(cy + 12 * math.sin(ang))
        spikes.append((x, y))
    for i in range(8):
        x1, y1 = spikes[i]
        x2, y2 = spikes[(i + 1) % 8]
        mid = (int(cx + 6 * math.cos(math.radians(i * 45 + 45))),
               int(cy + 6 * math.sin(math.radians(i * 45 + 45))))
        tone = 1.08 if i % 2 == 0 else 0.9
        d.polygon([(cx, cy), (x1, y1), mid], fill=shade(GOLD, tone))
        d.polygon([(cx, cy), mid, (x2, y2)], fill=shade(GOLD, tone * 0.94))
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=shade(GOLD, 1.12))
    # carved groove arcs
    d.arc([cx - 8, cy - 8, cx + 8, cy + 8], start=40, end=140,
          fill=shade(GOLD, 0.72), width=1)
    d.arc([cx - 8, cy - 8, cx + 8, cy + 8], start=220, end=320,
          fill=shade(GOLD, 0.72), width=1)


def draw_delta(d, rng):
    # broadhead arrow shape of gold alloy, like a robot
    # boomerang: wide swept triangle with a basal notch
    d.polygon([(16, 3), (28, 22), (20, 20), (16, 25), (12, 20),
               (4, 22)], fill=GOLD)
    d.polygon([(16, 3), (28, 22), (20, 20), (16, 25)],
              fill=shade(GOLD, 0.9))
    d.line([(16, 5), (16, 23)], fill=shade(GOLD, 0.74), width=1)
    d.line([(16, 8), (24, 19)], fill=shade(GOLD, 1.14), width=1)
    d.line([(16, 8), (8, 19)], fill=shade(GOLD, 1.14), width=1)
    d.point((16, 14), fill=shade(GOLD, 0.7))


def draw_monolith(d, rng):
    # a large golden monolith, tile-filling slab with
    # carefully carved asymmetric edges
    d.polygon([(9, 2), (24, 2), (24, 8), (22, 10), (22, 20),
               (24, 22), (24, 30), (8, 30), (8, 14), (10, 12),
               (10, 6), (8, 4)], fill=GOLD)
    # carved asymmetric step notches, shadowed
    d.line([(24, 8), (22, 10), (22, 20), (24, 22)],
           fill=shade(GOLD, 0.7), width=1)
    d.line([(8, 4), (10, 6), (10, 12), (8, 14)],
           fill=shade(GOLD, 0.78), width=1)
    # face shading: lit left, darker right
    d.polygon([(10, 4), (16, 4), (16, 28), (10, 28)],
              fill=shade(GOLD, 1.1))
    d.polygon([(19, 4), (22, 4), (22, 28), (19, 28)],
              fill=shade(GOLD, 0.88))
    # faint carved glyph lines
    for yy in (9, 15, 21):
        d.line([(12, yy), (20, yy)], fill=shade(GOLD, 0.8), width=1)
    d.point((16, 12), fill=shade(GOLD, 0.72))
    d.point((14, 18), fill=shade(GOLD, 0.72))
    d.point((18, 24), fill=shade(GOLD, 0.72))


# ---------------------------------------------------------------- dispatch

DRONES = {
    "bot_lab_security_drone_GR": ((146, 196, 128, 255), (240, 200, 70, 255)),
    "bot_lab_security_drone_GM": ((128, 186, 140, 255), (70, 160, 220, 255)),
    "bot_lab_security_drone_YM": ((226, 196, 84, 255), (70, 160, 220, 255)),
    "bot_lab_security_drone_BM": ((96, 132, 196, 255), (240, 200, 70, 255)),
    "bot_lab_security_drone_BM2": ((96, 196, 208, 255), (226, 120, 60, 255)),
    "bot_lab_security_drone_BS": ((96, 100, 108, 255), (200, 70, 60, 255)),
}

HACKS = {
    "bot_manhack": ((146, 196, 128, 255), "blades"),
    "bot_EMP_hack": ((96, 196, 208, 255), "emp"),
    "bot_c4_hack": ((178, 182, 188, 255), "c4"),
    "bot_flashbang_hack": ((96, 100, 108, 255), "flash"),
    "bot_gasbomb_hack": ((96, 100, 108, 255), "gas"),
    "bot_tazer_hack": ((96, 196, 208, 255), "tazer"),
}

TURRETS = {
    "bot_turret": ((146, 196, 128, 255), "smg"),
    "bot_antimateriel": ((188, 84, 76, 255), "antimateriel"),
    "bot_rifleturret": ((110, 160, 96, 255), "rifle"),
    "bot_crows_m240": ((110, 160, 96, 255), "m240"),
    "bot_turret_riot": ((188, 84, 76, 255), "riot"),
}

TALONS = {
    "bot_secubot": ((146, 178, 138, 255), "rifle"),
    "bot_talon_m202a1": ((146, 178, 138, 255), "rocket"),
    "bot_secubot_liberty": ((178, 182, 188, 255), "liberty"),
}


def render(item_id, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if item_id in HACKS:
        body, payload = HACKS[item_id]
        draw_hack(d, rng, body, payload)
    elif item_id in DRONES:
        body, accent = DRONES[item_id]
        draw_drone(d, rng, body, accent)
    elif item_id in TURRETS:
        body, kind = TURRETS[item_id]
        draw_turret(d, rng, body, kind)
    elif item_id in TALONS:
        body, kind = TALONS[item_id]
        draw_talon(d, rng, body, kind)
    elif item_id == "bot_nursebot":
        draw_humanoid(d, rng, "nurse")
    elif item_id == "bot_grocerybot":
        draw_humanoid(d, rng, "grocery")
    elif item_id == "bot_grocerybot_busted":
        draw_humanoid(d, rng, "grocery_busted")
    elif item_id == "bot_hazmatbot":
        draw_humanoid(d, rng, "hazmat")
    elif item_id == "bot_prototype_cyborg":
        draw_cyborg(d, rng, broken=False)
    elif item_id == "bot_broken_cyborg":
        draw_cyborg(d, rng, broken=True)
    elif item_id == "bot_dispatch":
        draw_dispatch(d, rng, (216, 182, 84, 255), military=False)
    elif item_id == "bot_dispatch_military":
        draw_dispatch(d, rng, (110, 150, 92, 255), military=True)
    elif item_id == "bot_turret_searchlight":
        draw_searchlight(d, rng)
    elif item_id == "bot_turret_speaker":
        draw_speaker(d, rng)
    elif item_id == "bot_yrax_trifacet":
        draw_trifacet(d, rng)
    elif item_id == "bot_yrax_triakis":
        draw_triakis(d, rng)
    elif item_id == "bot_yrax_delta":
        draw_delta(d, rng)
    elif item_id == "bot_golden_monolith":
        draw_monolith(d, rng)
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
                       "preview_item42.png")
    sheet_out.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item42.png {sheet_out.size}")


if __name__ == "__main__":
    main()
