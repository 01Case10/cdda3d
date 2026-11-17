#!/usr/bin/env python3
"""ITEM57: metalworking tools + underbarrel gun mods -- CDDA Voxel
coverage audit.

Two clusters:

  Metalworking (25, tool/metalworking.json):
    anvils graded by form/finish -- crude (small, rough, matte),
      heavy (enormous, polished face, hardy + pritchel holes),
      scavenged (smoothed machinery block, bolt circles, asymmetric),
      stump (steel plates bolted onto a heavy log);
    belt grinder (wheels + abrasive belt loop + motor box);
    sandcasting molds in a wood frame with a dark cavity impression
      (standard: round + bar; large: big ingot trapezoid; long: a
      long bar; small: a small round cavity in a small frame);
    chainmail link sheets by steel grade, in the ITEM47 colour
      language (lc light matte, mc mid, hc bright, ch dark
      blue-gray, qt heat-tinted rows);
    char kiln (steel drum, lid, chimney, soot), bronze chisel,
    draw plate (graded drilled holes), punch & drift set (tapered
    pins), gas forge (chamber with a glowing mouth, burner + gas
    bottle), bronze flatjaw tongs, pedal grindstone (stone wheel in
    a wood frame with a treadle), steel scraper (drawknife form,
    wood handles), sandleather (leather patch + sand), polishing
    stone (flat natural slab), tempering blocks (two steel blocks
    sandwiching a hot strip).

  Underbarrel gun mods (25, gunmod/underbarrel.json): each drawn
    mounted-profile style under a minimal host barrel line, so the
    attachment reads as itself:
    wire monopod collapsed (folded along the barrel) vs deployed
      (wire leg down); bipod (legs splayed); bipod hand guard with
      legs folded into the guard vs deployed; combination-gun
      shotgun (second barrel tube); integrated bayonet out (blade
      forward) vs folded (blade back along the barrel); laser sight
      (box + red beam dot); M203 and modified M203 (custom band,
      rail and grip tape on the mod); M26-MASS shotgun (bolt +
      magazine); M320 and modified M320 (compact fat tube, grip);
      masterkey and modified masterkey (pump shotgun); high-end
      hand guard (ventilated rail guard, no device); mounted
      flashlight off (dark lens) vs on (bright lens + solid beam
      wedge); muzzle weight (block under the muzzle); pistol
      bayonet (short blade under a pistol slide); REE-33 heavy
      tripod (three massive legs); underslung shotgun (two short
      barrels).  Deployed/folded and on/off pairs read differently
      by construction.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "anvil_crude",
    "anvil_heavy",
    "anvil_scavenged",
    "anvil_stump",
    "belt_grinder",
    "casting_mold",
    "casting_mold_large",
    "casting_mold_long",
    "casting_mold_small",
    "ch_link_sheet",
    "char_kiln",
    "chisel_bronze",
    "draw_plate",
    "drift",
    "gas_forge",
    "hc_link_sheet",
    "lc_link_sheet",
    "leather_filing",
    "mc_link_sheet",
    "metalworking_tongs_bronze",
    "pedal_grindstone",
    "qt_link_sheet",
    "scraper_steel",
    "stone_polishing",
    "temper_block",
    "arisaka_monopod",
    "arisaka_monopod_deployed",
    "bipod",
    "bipod_handguard",
    "bipod_handguard_deployed",
    "combination_gun_shotgun",
    "inter_bayonet",
    "inter_bayonet_folded",
    "laser_sight",
    "m203",
    "m203_mod",
    "m26_mass",
    "m320_mod",
    "m320_mod_mod",
    "masterkey",
    "masterkey_mod",
    "modern_handguard",
    "mounted_flashlight",
    "mounted_flashlight_on",
    "muzzle_weight",
    "pistol_bayonet",
    "ree_33_tripod",
    "scout_bipod",
    "scout_bipod_deployed",
    "u_shotgun",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

STEEL = (110, 114, 122, 255)
STEEL_DARK = (62, 64, 70, 255)
STEEL_LIGHT = (168, 172, 180, 255)
IRON_DARK = (44, 44, 50, 255)
WOOD = (122, 88, 54, 255)
WOOD_DARK = (84, 58, 34, 255)
SAND = (196, 170, 128, 255)
SAND_DARK = (150, 124, 88, 255)
BRONZE = (176, 128, 62, 255)
BRONZE_LIGHT = (216, 170, 96, 255)
BRONZE_DARK = (128, 90, 40, 255)
LEATHER = (140, 100, 66, 255)
STONE = (146, 142, 136, 255)
STONE_DARK = (104, 100, 94, 255)
POLYMER = (52, 52, 58, 255)
ALU = (158, 162, 168, 255)
TAN = (168, 148, 106, 255)
RED = (206, 52, 44, 255)
GLOW = (255, 128, 36, 255)
GLOW_CORE = (255, 196, 92, 255)
BEAM = (252, 238, 170, 255)
BEAM_CORE = (255, 255, 244, 255)

# chainmail grades, ITEM47 colour language
GRADES = {
    "lc": (168, 172, 178, 255),   # mild: light matte
    "mc": (140, 146, 154, 255),   # medium: mid gray
    "hc": (196, 202, 210, 255),   # high: bright polished
    "ch": (104, 112, 128, 255),   # hardened: dark blue-gray
    "qt": (158, 138, 120, 255),   # tempered: warm heat tint
}
QT_BANDS = [(196, 164, 92, 255), (140, 120, 180, 255), (96, 130, 190, 255)]


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- metalworking

def anvil_profile(d, x0, x1, top, waist, base, col, face):
    """Classic anvil: horn to the left, flat face, waist, foot."""
    horn_y = top + 2
    d.polygon([(x0, horn_y), (x0 + 5, top), (x1, top), (x1, top + 3),
               (x0 + 9, top + 3)], fill=col)               # horn + face
    d.rectangle([x0 + 5, top, x1, top + 1], fill=face)      # polished face
    d.polygon([(x0 + 7, top + 3), (x1 - 3, top + 3), (x1 - 5, waist),
               (x0 + 9, waist)], fill=shade(col, 0.85))    # neck
    d.rectangle([x0 + 8, waist, x1 - 4, base - 3], fill=col)  # body
    d.rectangle([x0 + 4, base - 3, x1, base], fill=shade(col, 0.8))  # foot


def draw_anvil(d, rng, sid):
    if sid == "anvil_crude":
        anvil_profile(d, 6, 26, 10, 17, 25, shade(STEEL, 0.8),
                      shade(STEEL_LIGHT, 0.85))
        for _ in range(5):  # rough pitted finish
            d.point((rng.randint(8, 25), rng.randint(12, 23)),
                    fill=STEEL_DARK)
    elif sid == "anvil_heavy":
        anvil_profile(d, 2, 30, 7, 15, 27, STEEL, STEEL_LIGHT)
        d.point((26, 8), fill=IRON_DARK)   # hardy hole (square-ish)
        d.point((27, 8), fill=IRON_DARK)
        d.point((22, 8), fill=IRON_DARK)   # pritchel hole
        d.line([4, 26, 28, 26], fill=STEEL_LIGHT, width=1)
    elif sid == "anvil_scavenged":
        # smoothed machinery block: asymmetric, gear stub + bolt circles
        d.rectangle([7, 9, 27, 12], fill=STEEL_LIGHT)      # smoothed top
        d.rectangle([8, 12, 26, 24], fill=STEEL)
        d.rectangle([8, 21, 28, 24], fill=shade(STEEL, 0.8))
        for bx, by in ((10, 14), (24, 14), (10, 20), (24, 20)):
            d.ellipse([bx - 1, by - 1, bx + 1, by + 1],
                      outline=STEEL_DARK, width=1)          # bolt circles
        for gx in range(26, 30):                            # gear teeth stub
            d.point((gx, 13 + (gx % 2) * 2), fill=STEEL_DARK)
        d.line([9, 10, 25, 10], fill=(210, 214, 220, 255), width=1)
    elif sid == "anvil_stump":
        # heavy log with steel plates bolted on top
        d.rectangle([8, 11, 24, 27], fill=WOOD)
        for gx in (11, 16, 21):
            d.line([gx, 12, gx, 26], fill=WOOD_DARK, width=1)
        d.ellipse([8, 25, 24, 28], outline=WOOD_DARK, width=1)
        d.rectangle([6, 8, 26, 11], fill=STEEL)             # top plate
        d.rectangle([6, 8, 26, 9], fill=STEEL_LIGHT)
        d.rectangle([13, 5, 19, 8], fill=STEEL)             # raised bit
        for bx in (8, 15, 24):
            d.point((bx, 10), fill=IRON_DARK)               # bolts


def draw_belt_grinder(d, rng):
    d.rectangle([5, 22, 27, 25], fill=STEEL_DARK)           # base
    d.rectangle([5, 22, 27, 22], fill=STEEL)
    d.ellipse([6, 8, 14, 16], outline=STEEL, width=2)       # drive wheel
    d.ellipse([20, 6, 26, 12], outline=STEEL, width=2)      # idler wheel
    d.line([10, 8, 23, 6], fill=TAN, width=2)               # abrasive belt
    d.line([10, 16, 23, 12], fill=shade(TAN, 0.8), width=2)
    d.point((10, 12), fill=IRON_DARK)
    d.point((23, 9), fill=IRON_DARK)
    d.rectangle([16, 16, 26, 22], fill=(70, 96, 130, 255))  # motor box
    d.line([17, 18, 25, 18], fill=shade((70, 96, 130, 255), 1.4), width=1)
    d.rectangle([18, 13, 22, 16], fill=STEEL)               # work rest


def draw_mold(d, rng, sid):
    frames = {"casting_mold": (5, 6, 26, 25),
              "casting_mold_large": (3, 4, 28, 27),
              "casting_mold_long": (2, 9, 29, 22),
              "casting_mold_small": (9, 10, 22, 21)}
    x0, y0, x1, y1 = frames[sid]
    d.rectangle([x0, y0, x1, y1], fill=WOOD)                # wood frame
    d.rectangle([x0 + 2, y0 + 2, x1 - 2, y1 - 2], fill=SAND)  # sand bed
    for _ in range(14):                                     # sand grain
        d.point((rng.randint(x0 + 3, x1 - 3), rng.randint(y0 + 3, y1 - 3)),
                fill=SAND_DARK)
    cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
    if sid == "casting_mold":
        d.ellipse([cx - 6, cy - 4, cx - 1, cy + 1], fill=SAND_DARK)
        d.rectangle([cx + 2, cy - 2, cx + 7, cy + 1], fill=SAND_DARK)
    elif sid == "casting_mold_large":
        d.polygon([(cx - 7, cy - 5), (cx + 7, cy - 5), (cx + 5, cy + 5),
                   (cx - 5, cy + 5)], fill=SAND_DARK)       # ingot cavity
        d.line([cx - 5, cy - 3, cx + 5, cy - 3], fill=shade(SAND_DARK, 0.7),
               width=1)
    elif sid == "casting_mold_long":
        d.rectangle([x0 + 4, cy - 2, x1 - 4, cy + 2], fill=SAND_DARK)
        d.point((cx, cy), fill=shade(SAND_DARK, 0.7))      # sprue
    else:
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=SAND_DARK)
    d.line([cx, y0 + 2, cx, y0 + 4], fill=SAND_DARK, width=1)  # pour channel


def draw_link_sheet(d, rng, grade):
    col = GRADES[grade]
    d.rectangle([6, 5, 26, 26], fill=shade(col, 0.55))      # sheet backing
    for ry, y in enumerate(range(7, 26, 4)):                # ring rows
        for rx, x in enumerate(range(7, 26, 4)):
            c = col
            if grade == "qt":                               # heat-tint rows
                c = QT_BANDS[ry % len(QT_BANDS)] if rx % 3 == 0 else col
            off = 2 if ry % 2 else 0
            d.ellipse([x + off - 2, y - 2, x + off + 2, y + 2],
                      outline=c, width=1)
            if grade == "hc":                               # polished glint
                d.point((x + off - 1, y - 2), fill=(240, 244, 248, 255))
    d.line([6, 5, 26, 5], fill=shade(col, 1.15), width=1)


def draw_kiln(d, rng):
    d.rectangle([10, 7, 23, 26], fill=STEEL)                # drum
    d.ellipse([10, 5, 23, 9], fill=STEEL_LIGHT)             # lid
    d.rectangle([15, 2, 18, 6], fill=STEEL_DARK)            # chimney stub
    for y in (13, 20):                                      # drum ribs
        d.line([10, y, 23, y], fill=STEEL_DARK, width=1)
    d.rectangle([14, 16, 19, 22], fill=IRON_DARK)           # fire door
    d.point((18, 19), fill=STEEL_LIGHT)
    for _ in range(4):                                      # soot streaks
        x = rng.randint(11, 22)
        d.line([x, 8, x, rng.randint(10, 13)], fill=shade(STEEL, 0.6),
               width=1)
    d.line([11, 26, 9, 29], fill=STEEL_DARK, width=2)       # legs
    d.line([22, 26, 24, 29], fill=STEEL_DARK, width=2)


def draw_chisel(d, rng):
    # short stout bronze chisel, diagonal
    d.polygon([(9, 22), (20, 9), (23, 11), (13, 25)], fill=BRONZE)
    d.polygon([(20, 9), (23, 11), (25, 8), (22, 6)], fill=BRONZE_LIGHT)
    d.line([11, 21, 21, 10], fill=BRONZE_DARK, width=1)
    d.rectangle([8, 22, 12, 26], fill=BRONZE_DARK)          # struck head
    d.line([22, 6, 25, 8], fill=(240, 210, 150, 255), width=1)  # edge


def draw_draw_plate(d, rng):
    d.rectangle([6, 6, 25, 25], fill=STEEL)
    d.rectangle([6, 6, 25, 25], outline=STEEL_DARK, width=1)
    d.line([7, 7, 24, 7], fill=STEEL_LIGHT, width=1)
    sizes = [(1, 1), (2, 2), (2, 2), (3, 3), (3, 3), (4, 4)]
    i = 0
    for y in (11, 16, 21):
        for x in (10, 15, 20):
            w, h = sizes[i % len(sizes)]
            d.ellipse([x - w // 2, y - h // 2, x + w // 2 + (w % 2),
                       y + h // 2 + (h % 2)], fill=IRON_DARK)
            i += 1


def draw_drift(d, rng):
    d.rectangle([4, 6, 28, 26], fill=(40, 40, 46, 255))     # roll backing
    d.rectangle([4, 6, 28, 26], outline=STEEL_DARK, width=1)
    for i, x in enumerate((8, 12, 16, 20, 24)):             # tapered pins
        d.polygon([(x - 1, 9), (x + 1, 9), (x, 20)], fill=STEEL_LIGHT)
        d.rectangle([x - 2, 20, x + 2, 23], fill=STEEL)     # heads
        if i == 1:                                          # square drift
            d.rectangle([x - 1, 10, x + 1, 19], fill=STEEL_LIGHT)
        if i == 3:                                          # oval drift
            d.ellipse([x - 1, 10, x + 1, 19], fill=STEEL_LIGHT)


def draw_gas_forge(d, rng):
    d.rectangle([6, 10, 22, 24], fill=STEEL)               # chamber body
    d.rectangle([6, 10, 22, 12], fill=STEEL_LIGHT)
    d.ellipse([10, 14, 18, 22], fill=GLOW)                  # glowing mouth
    d.ellipse([12, 16, 16, 20], fill=GLOW_CORE)
    d.rectangle([12, 6, 16, 10], fill=STEEL_DARK)           # burner tube
    d.point((14, 5), fill=(84, 120, 190, 255))              # pilot tint
    d.line([16, 8, 24, 12], fill=IRON_DARK, width=1)        # gas hose
    d.rectangle([23, 13, 28, 24], fill=(70, 110, 60, 255))  # gas bottle
    d.rectangle([24, 11, 27, 13], fill=STEEL_DARK)
    d.line([7, 24, 5, 28], fill=STEEL_DARK, width=2)        # legs
    d.line([21, 24, 23, 28], fill=STEEL_DARK, width=2)


def draw_tongs(d, rng):
    # flatjaw bronze tongs: jaws top, pivot rivet, ring handles bottom
    d.polygon([(13, 3), (15, 3), (15, 10), (13, 10)], fill=BRONZE)
    d.polygon([(17, 3), (19, 3), (19, 10), (17, 10)], fill=BRONZE)
    d.line([14, 10, 12, 24], fill=BRONZE, width=2)          # arms crossing
    d.line([18, 10, 20, 24], fill=BRONZE, width=2)
    d.line([12, 24, 20, 12], fill=BRONZE_DARK, width=1)
    d.ellipse([14, 11, 18, 15], fill=BRONZE_DARK)           # pivot rivet
    d.point((16, 13), fill=BRONZE_LIGHT)
    d.ellipse([9, 24, 14, 28], outline=BRONZE, width=1)     # handle rings
    d.ellipse([18, 24, 23, 28], outline=BRONZE, width=1)


def draw_grindstone(d, rng):
    d.ellipse([8, 4, 24, 20], fill=STONE)                   # wheel
    d.ellipse([8, 4, 24, 20], outline=STONE_DARK, width=1)
    for _ in range(10):                                     # stone speckle
        d.point((rng.randint(10, 22), rng.randint(6, 18)), fill=STONE_DARK)
    d.ellipse([14, 10, 18, 14], fill=STEEL_DARK)            # hub
    d.line([6, 20, 26, 20], fill=WOOD, width=2)             # frame beam
    d.line([9, 20, 8, 28], fill=WOOD_DARK, width=2)         # legs
    d.line([23, 20, 24, 28], fill=WOOD_DARK, width=2)
    d.line([10, 26, 22, 26], fill=STEEL, width=2)           # treadle pedal
    d.line([16, 20, 16, 26], fill=STEEL_DARK, width=1)      # pedal link


def draw_scraper(d, rng):
    d.rectangle([7, 13, 25, 16], fill=STEEL)                # blade
    d.line([7, 16, 25, 16], fill=STEEL_LIGHT, width=1)      # cutting edge
    d.line([7, 13, 25, 13], fill=shade(STEEL, 0.75), width=1)
    d.line([7, 14, 3, 9], fill=WOOD, width=2)               # handles
    d.line([25, 14, 29, 9], fill=WOOD, width=2)
    d.ellipse([2, 7, 5, 10], fill=WOOD_DARK)
    d.ellipse([27, 7, 30, 10], fill=WOOD_DARK)


def draw_leather_filing(d, rng):
    d.rounded_rectangle([6, 9, 24, 24], radius=3, fill=LEATHER)
    d.rounded_rectangle([6, 9, 24, 24], radius=3, outline=shade(LEATHER, 0.6),
                        width=1)
    for x in range(9, 23, 3):                               # stitch border
        d.point((x, 11), fill=(232, 210, 170, 255))
        d.point((x, 22), fill=(232, 210, 170, 255))
    for _ in range(16):                                     # sand handful
        d.point((rng.randint(17, 26), rng.randint(6, 12)), fill=SAND)
    d.ellipse([18, 5, 27, 10], fill=SAND_DARK)
    d.ellipse([19, 5, 26, 9], fill=SAND)


def draw_polishing_stone(d, rng):
    d.polygon([(5, 12), (10, 8), (22, 7), (27, 11), (26, 20), (20, 24),
               (9, 23), (5, 18)], fill=STONE)               # natural slab
    d.polygon([(5, 12), (10, 8), (22, 7), (27, 11), (26, 20), (20, 24),
               (9, 23), (5, 18)], outline=STONE_DARK)
    d.ellipse([11, 10, 21, 17], fill=shade(STONE, 1.12))    # polished flat
    for _ in range(8):
        d.point((rng.randint(7, 25), rng.randint(9, 21)), fill=STONE_DARK)


def draw_temper_block(d, rng):
    d.rectangle([5, 12, 15, 22], fill=GRADES["mc"])         # block one
    d.rectangle([5, 12, 15, 22], outline=STEEL_DARK, width=1)
    d.rectangle([17, 12, 27, 22], fill=GRADES["mc"])        # block two
    d.rectangle([17, 12, 27, 22], outline=STEEL_DARK, width=1)
    d.line([5, 13, 15, 13], fill=STEEL_LIGHT, width=1)
    d.line([17, 13, 27, 13], fill=STEEL_LIGHT, width=1)
    d.rectangle([14, 15, 18, 18], fill=GLOW)                # hot strip between
    d.rectangle([15, 16, 17, 17], fill=GLOW_CORE)
    d.line([10, 9, 22, 9], fill=STEEL_DARK, width=1)        # laid tool hint
    d.point((16, 9), fill=STEEL)


METAL = {
    "anvil_crude": draw_anvil, "anvil_heavy": draw_anvil,
    "anvil_scavenged": draw_anvil, "anvil_stump": draw_anvil,
}


def draw_metal(d, rng, sid):
    if sid in METAL:
        draw_anvil(d, rng, sid)
    elif sid == "belt_grinder":
        draw_belt_grinder(d, rng)
    elif sid.startswith("casting_mold"):
        draw_mold(d, rng, sid)
    elif sid.endswith("_link_sheet"):
        draw_link_sheet(d, rng, sid[:2])
    elif sid == "char_kiln":
        draw_kiln(d, rng)
    elif sid == "chisel_bronze":
        draw_chisel(d, rng)
    elif sid == "draw_plate":
        draw_draw_plate(d, rng)
    elif sid == "drift":
        draw_drift(d, rng)
    elif sid == "gas_forge":
        draw_gas_forge(d, rng)
    elif sid == "metalworking_tongs_bronze":
        draw_tongs(d, rng)
    elif sid == "pedal_grindstone":
        draw_grindstone(d, rng)
    elif sid == "scraper_steel":
        draw_scraper(d, rng)
    elif sid == "leather_filing":
        draw_leather_filing(d, rng)
    elif sid == "stone_polishing":
        draw_polishing_stone(d, rng)
    elif sid == "temper_block":
        draw_temper_block(d, rng)
    else:
        raise KeyError(sid)


# --------------------------------------------------------------- underbarrel

def host_barrel(d, x1=30):
    """Minimal host: barrel line + receiver block + handguard hint."""
    d.rectangle([2, 6, 9, 10], fill=POLYMER)                # receiver
    d.rectangle([9, 7, x1, 9], fill=IRON_DARK)              # barrel
    d.line([9, 7, x1, 7], fill=STEEL, width=1)
    d.rectangle([11, 9, 19, 11], fill=shade(POLYMER, 1.25))  # handguard hint
    d.point((x1 - 1, 8), fill=STEEL_DARK)                   # muzzle


def mount(d, x=14):
    d.rectangle([x, 10, x + 3, 13], fill=STEEL_DARK)        # mount block


def ground(d, y=29):
    d.line([3, y, 29, y], fill=(70, 70, 76, 255), width=1)


def draw_monopod(d, rng, deployed):
    host_barrel(d)
    if not deployed:  # wire folded along the barrel band
        d.rectangle([18, 11, 27, 14], outline=STEEL, width=1)
        d.line([19, 12, 26, 12], fill=STEEL_DARK, width=1)
        mount(d, 20)
    else:             # wire leg down: inverted-U wire
        mount(d, 21)
        d.line([21, 13, 20, 28], fill=STEEL, width=1)
        d.line([24, 13, 25, 28], fill=STEEL, width=1)
        d.line([20, 28, 25, 28], fill=STEEL, width=1)
        ground(d)


def bipod_legs(d, col=STEEL):
    mount(d, 16)
    d.line([16, 13, 10, 28], fill=col, width=2)             # splayed legs
    d.line([19, 13, 25, 28], fill=col, width=2)
    d.line([8, 28, 12, 28], fill=STEEL_DARK, width=2)       # feet
    d.line([23, 28, 27, 28], fill=STEEL_DARK, width=2)
    ground(d)


def draw_bipod(d, rng, sid):
    host_barrel(d)
    if sid == "bipod":
        bipod_legs(d)
        d.point((17, 12), fill=STEEL_LIGHT)                 # pivot
    elif sid == "bipod_handguard":
        # legs folded forward into a chunky aluminum hand guard
        d.rectangle([10, 9, 26, 14], fill=ALU)
        for x in range(12, 25, 3):                          # guard slots
            d.point((x, 11), fill=STEEL_DARK)
        d.line([12, 15, 25, 15], fill=STEEL, width=2)       # folded legs
        d.line([12, 17, 24, 17], fill=shade(STEEL, 0.8), width=1)
    elif sid == "bipod_handguard_deployed":
        d.rectangle([10, 9, 26, 13], fill=ALU)
        for x in range(12, 25, 3):
            d.point((x, 11), fill=STEEL_DARK)
        bipod_legs(d, ALU)
    elif sid == "scout_bipod":
        # polymer legs flush in the housing: grooved hand guard only
        d.rectangle([10, 9, 27, 14], fill=POLYMER)
        d.line([12, 12, 25, 12], fill=shade(POLYMER, 1.5), width=1)
        d.line([12, 14, 25, 14], fill=shade(POLYMER, 1.5), width=1)
        d.point((26, 11), fill=STEEL_DARK)
    elif sid == "scout_bipod_deployed":
        d.rectangle([10, 9, 27, 13], fill=POLYMER)
        d.line([12, 12, 25, 12], fill=shade(POLYMER, 1.5), width=1)
        bipod_legs(d, POLYMER)


def draw_combo_shotgun(d, rng):
    host_barrel(d)
    d.rectangle([10, 11, 29, 14], fill=STEEL)               # second barrel
    d.line([10, 11, 29, 11], fill=STEEL_LIGHT, width=1)
    d.ellipse([27, 11, 30, 14], fill=IRON_DARK)             # muzzle bore
    d.point((28, 12), fill=(12, 12, 14, 255))
    d.rectangle([12, 14, 17, 16], fill=WOOD_DARK)           # fore-end wood


def draw_bayonet(d, rng, folded, pistol=False):
    if pistol:
        d.rectangle([4, 7, 16, 12], fill=POLYMER)           # pistol slide
        d.line([4, 8, 16, 8], fill=STEEL, width=1)
        d.rectangle([5, 12, 8, 17], fill=POLYMER)           # grip hint
        bx = 12
    else:
        host_barrel(d)
        bx = 24
    if not folded:  # blade forward from under the muzzle
        mount(d, bx - 2)
        d.polygon([(bx, 11), (bx + (6 if pistol else 8), 12),
                   (bx, 14)], fill=STEEL_LIGHT)
        d.line([bx, 12, bx + (5 if pistol else 7), 12],
               fill=(230, 234, 238, 255), width=1)          # edge
    else:           # blade folded back along the barrel underside
        d.polygon([(bx + 4, 11), (bx - 8, 12), (bx + 4, 14)],
                  fill=shade(STEEL_LIGHT, 0.85))
        d.line([bx - 7, 12, bx + 3, 12], fill=STEEL_DARK, width=1)
        mount(d, bx + 2)


def draw_laser(d, rng):
    host_barrel(d)
    mount(d, 17)
    d.rectangle([15, 13, 23, 18], fill=POLYMER)             # sight body
    d.rectangle([15, 13, 23, 14], fill=shade(POLYMER, 1.4))
    d.ellipse([21, 14, 24, 17], fill=IRON_DARK)             # lens housing
    d.point((22, 15), fill=RED)                            # emitter
    d.line([24, 15, 30, 15], fill=RED, width=1)             # beam stub
    d.point((30, 15), fill=RED)
    d.point((30, 14), fill=shade(RED, 1.3))


def launcher_tube(d, x0, x1, y0, y1, col=STEEL):
    d.rectangle([x0, y0, x1, y1], fill=col)
    d.line([x0, y0, x1, y0], fill=STEEL_LIGHT, width=1)
    d.ellipse([x1 - 2, y0, x1 + 1, y1], fill=IRON_DARK)     # muzzle bore
    for x in range(x0 + 4, x1 - 3, 4):                      # ribbing
        d.line([x, y0 + 1, x, y1 - 1], fill=STEEL_DARK, width=1)


def draw_m203(d, rng, mod):
    host_barrel(d)
    mount(d, 12)
    launcher_tube(d, 10, 29, 12, 17)
    d.rectangle([13, 17, 16, 20], fill=POLYMER)             # trigger/grip
    d.line([18, 10, 20, 7], fill=STEEL_DARK, width=1)       # leaf sight strut
    if mod:  # customized: tan band, top rail, grip tape
        d.rectangle([20, 11, 23, 18], fill=TAN)
        d.line([11, 10, 28, 10], fill=STEEL_DARK, width=1)
        for x in range(14, 17):
            d.point((x, 19), fill=TAN)


def draw_m26(d, rng):
    host_barrel(d)
    mount(d, 12)
    launcher_tube(d, 11, 28, 12, 16)
    d.rectangle([14, 16, 19, 19], fill=STEEL_DARK)          # pump
    d.line([24, 13, 26, 11], fill=STEEL_LIGHT, width=2)     # bolt handle
    d.rectangle([19, 17, 22, 21], fill=POLYMER)             # box magazine hint


def draw_m320(d, rng, mod):
    host_barrel(d)
    mount(d, 13)
    launcher_tube(d, 12, 26, 12, 18)                        # short fat tube
    d.rectangle([15, 18, 18, 23], fill=POLYMER)             # pistol grip
    d.line([15, 20, 18, 20], fill=shade(POLYMER, 1.4), width=1)
    d.point((13, 14), fill=STEEL_LIGHT)                     # hinge pin
    if mod:  # customized: tan band + side rail
        d.rectangle([21, 11, 24, 19], fill=TAN)
        d.line([13, 19, 25, 19], fill=STEEL_DARK, width=1)


def draw_masterkey(d, rng, mod):
    host_barrel(d)
    mount(d, 12)
    d.rectangle([10, 12, 29, 15], fill=STEEL)               # barrel + tube
    d.line([10, 15, 29, 15], fill=STEEL_DARK, width=1)
    d.ellipse([27, 12, 30, 15], fill=IRON_DARK)
    d.rectangle([17, 15, 23, 19], fill=WOOD_DARK if not mod else TAN)
    for x in range(18, 23, 2):                              # pump ribs
        d.line([x, 16, x, 18], fill=STEEL_DARK, width=1)
    if mod:  # customized: muzzle device + band
        d.rectangle([26, 11, 30, 16], fill=STEEL_DARK)
        d.line([27, 12, 27, 15], fill=STEEL, width=1)
        d.rectangle([11, 11, 13, 16], fill=TAN)


def draw_handguard(d, rng):
    # high-end guard replaces the factory one: long, ventilated, railed
    d.rectangle([2, 6, 9, 10], fill=POLYMER)
    d.rectangle([9, 7, 30, 9], fill=IRON_DARK)
    d.rectangle([10, 9, 29, 15], fill=ALU)
    for x in range(12, 28, 3):                              # vent holes
        d.ellipse([x, 11, x + 1, 13], fill=STEEL_DARK)
    d.line([11, 9, 28, 9], fill=STEEL_DARK, width=1)        # top rail
    for x in (14, 20, 26):                                  # accessory mounts
        d.rectangle([x, 15, x + 2, 17], fill=STEEL_DARK)


def draw_flashlight(d, rng, on):
    host_barrel(d)
    mount(d, 18)
    d.rectangle([15, 13, 24, 18], fill=POLYMER)             # body
    d.rectangle([22, 12, 26, 19], fill=STEEL_DARK)          # bezel
    if not on:
        d.ellipse([24, 14, 27, 17], fill=(60, 66, 72, 255))  # dark lens
        d.point((25, 15), fill=(96, 104, 112, 255))
    else:
        d.polygon([(26, 13), (31, 10), (31, 21), (26, 18)],
                  fill=BEAM)                                 # beam wedge
        d.polygon([(26, 14), (31, 13), (31, 18), (26, 17)],
                  fill=BEAM_CORE)
        d.ellipse([24, 14, 27, 17], fill=BEAM_CORE)          # bright lens
        d.point((25, 15), fill=(255, 255, 255, 255))


def draw_muzzle_weight(d, rng):
    host_barrel(d)
    d.rectangle([24, 10, 30, 17], fill=ALU)                 # weight block
    d.rectangle([24, 10, 30, 17], outline=STEEL_DARK, width=1)
    d.line([25, 13, 29, 13], fill=STEEL_DARK, width=1)      # split line
    d.point((27, 15), fill=STEEL_DARK)                      # screw


def draw_tripod(d, rng):
    host_barrel(d)
    mount(d, 15)
    d.rectangle([13, 12, 21, 15], fill=STEEL_DARK)           # massive head
    d.line([14, 15, 6, 28], fill=STEEL, width=2)            # three thick legs
    d.line([17, 15, 17, 28], fill=STEEL, width=2)
    d.line([20, 15, 28, 28], fill=STEEL, width=2)
    for fx in (4, 15, 26):
        d.line([fx, 28, fx + 4, 28], fill=IRON_DARK, width=2)  # feet pads
    ground(d)


def draw_u_shotgun(d, rng):
    host_barrel(d)
    mount(d, 12)
    d.rectangle([11, 12, 27, 15], fill=STEEL)               # twin tubes
    d.ellipse([25, 12, 28, 13], fill=IRON_DARK)             # two bores
    d.ellipse([25, 14, 28, 15], fill=IRON_DARK)
    d.line([11, 13, 25, 13], fill=STEEL_LIGHT, width=1)
    d.rectangle([13, 15, 18, 18], fill=WOOD_DARK)           # fore-end
    d.point((12, 13), fill=STEEL_DARK)                      # hinge


GUN = {
    "combination_gun_shotgun": draw_combo_shotgun,
    "m26_mass": draw_m26,
    "modern_handguard": draw_handguard,
    "muzzle_weight": draw_muzzle_weight,
    "ree_33_tripod": draw_tripod,
    "u_shotgun": draw_u_shotgun,
    "laser_sight": draw_laser,
}


def draw_gun(d, rng, sid):
    if sid.startswith("arisaka_monopod"):
        draw_monopod(d, rng, sid.endswith("deployed"))
    elif "bipod" in sid:
        draw_bipod(d, rng, sid)
    elif sid == "inter_bayonet":
        draw_bayonet(d, rng, folded=False)
    elif sid == "inter_bayonet_folded":
        draw_bayonet(d, rng, folded=True)
    elif sid == "pistol_bayonet":
        draw_bayonet(d, rng, folded=False, pistol=True)
    elif sid in ("m203", "m203_mod"):
        draw_m203(d, rng, sid.endswith("mod"))
    elif sid in ("m320_mod", "m320_mod_mod"):
        draw_m320(d, rng, sid == "m320_mod_mod")
    elif sid in ("masterkey", "masterkey_mod"):
        draw_masterkey(d, rng, sid.endswith("mod"))
    elif sid in ("mounted_flashlight", "mounted_flashlight_on"):
        draw_flashlight(d, rng, sid.endswith("on"))
    elif sid in GUN:
        GUN[sid](d, rng)
    else:
        raise KeyError(sid)


# -------------------------------------------------------------------- main

METALWORKING = set(IDS[:25])


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in METALWORKING:
        draw_metal(d, rng, sid)
    else:
        draw_gun(d, rng, sid)
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
    # contact-sheet preview: 7 cols x 8 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 8, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item57.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item57.png {prev.size}")


if __name__ == "__main__":
    main()
