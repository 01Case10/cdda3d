#!/usr/bin/env python3
"""ITEM64: software/data media + spears and polearms -- CDDA Voxel
coverage audit.

Two clusters, each drawn as itself:

  Software / data (software.json, 18) -- data storage media:
    abstract_software: a generic external drive (platter window,
      blank gray label) -- the copy-from base for all software.
    e-files (7): SD-style memory cards with contact pins and a
      content icon on the label -- junk (trash/X), lore (eye),
      map (folded map + route), photos (picture glyph), recipes
      (book), recipes_medical (book + red cross), recipes_science
      (book + flask).
    named software (9): CD discs, each with a distinct label
      colour and icon -- archived data (archive box, tan), blood
      data (red droplet), electronics reference (IC chip, green),
      encryption codes (padlock, dark slate), hacking (terminal
      >_ on black/green), lab data (flask, teal), math (pi glyph,
      blue), medical (red cross, white), useless (blank gray).
    train_data: a dataset cartridge card -- laboratory shipping
      records on subterranean train routes: rail tracks + train
      front on the label.

  Spears and polearms (spears_and_polearms.json, 18) -- long
  weapons drawn diagonally, butt at lower-left, head upper-right:
    tepoztopili (Aztec spears): an oar-like oval wooden head lined
      with blades of the head material -- glass (dark green,
      translucent-look highlights), scrap (rusty jagged), stone
      (chipped gray), and the five steel grades in ITEM47 colours
      (lc/mc/hc/ch/qt) for the redesigned metal-bladed versions.
    crude halberd: spear shaft with an axe head lashed below the
      point.  Javelins: slim throwing shafts -- wooden with a
      fire-hardened point, iron with a small iron leaf tip.
    naginata machete: a machete blade mounted as a polearm head
      (mc steel; the tempered variant in qt steel with a heat
      sheen band).  Pointed rebar: ribbed dark rod, crudely
      hammered irregular point; sharp rebar: same rod, cleaner
      cut point.  Sharpened pipe: hollow steel pipe, one end cut
      at an angle.  Simple wood spear: all wood, whittled point.
    string-dimension partigiana: an alien shortspear of a single
      continuous piece of tempered steel -- steel shaft and all,
      broad cutting head with a small hook on either side.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "abstract_software",
    "efile_junk",
    "efile_lore",
    "efile_map",
    "efile_photos",
    "efile_recipes",
    "efile_recipes_medical",
    "efile_recipes_science",
    "software_archived_data",
    "software_blood_data",
    "software_electronics_reference",
    "software_encryption_codes",
    "software_hacking",
    "software_lab_data",
    "software_math",
    "software_medical",
    "software_useless",
    "train_data",
    "aztec_spear_glass",
    "aztec_spear_scrap",
    "aztec_spear_stone",
    "ch_aztec_spear",
    "halberd_crude",
    "hc_aztec_spear",
    "javelin",
    "javelin_iron",
    "lc_aztec_spear",
    "mc_aztec_spear",
    "naginata_machete",
    "naginata_machete_temp",
    "pointed_rebar",
    "qt_aztec_spear",
    "sharp_rebar",
    "sharpened_pipe",
    "spear_wood_simple",
    "string_dimension_partigiana_ixwa",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (44, 44, 50, 255)
WOOD = (128, 94, 56, 255)
WOOD_DARK = (92, 66, 38, 255)
WOOD_LIGHT = (168, 130, 84, 255)
PAPER = (238, 234, 224, 255)
RED = (198, 44, 38, 255)
TEAL = (44, 140, 132, 255)

# ITEM47 steel grades: (base, dark, light)
STEELS = {
    "lc": ((168, 172, 176, 255), (124, 128, 132, 255), (196, 200, 204, 255)),
    "mc": ((128, 132, 138, 255), (84, 88, 96, 255), (170, 174, 180, 255)),
    "hc": ((196, 202, 212, 255), (108, 116, 128, 255), (248, 250, 252, 255)),
    "ch": ((74, 86, 106, 255), (42, 52, 70, 255), (118, 134, 158, 255)),
    "qt": ((138, 134, 148, 255), (88, 84, 102, 255), (186, 180, 196, 255)),
}
QT_SHEEN = (196, 150, 84, 255)  # straw heat-tint band on tempered steel


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- media icons
# Icons are drawn inside a label box roughly x 10..21, y 11..20.

def icon(d, key, x0=10, y0=11):
    cx, cy = x0 + 5, y0 + 4
    if key == "junk":  # trash can with an X lid
        d.polygon([(cx - 3, y0 + 2), (cx + 3, y0 + 2), (cx + 2, y0 + 8),
                   (cx - 2, y0 + 8)], outline=INK, width=1)
        d.line([cx - 4, y0 + 1, cx + 4, y0 + 1], fill=INK, width=1)
        d.line([cx - 1, y0 + 4, cx + 1, y0 + 6], fill=RED, width=1)
        d.line([cx + 1, y0 + 4, cx - 1, y0 + 6], fill=RED, width=1)
    elif key == "lore":  # an eye -- something to take a look at
        d.ellipse([cx - 5, y0 + 1, cx + 5, y0 + 7], outline=INK, width=1)
        d.ellipse([cx - 2, y0 + 2, cx + 2, y0 + 6], fill=INK)
        d.point((cx, y0 + 3), fill=PAPER)
        d.line([cx - 3, y0 + 9, cx + 3, y0 + 9], fill=INK, width=1)
    elif key == "map":  # folded map: fold lines + a route
        d.rectangle([x0, y0, x0 + 11, y0 + 9], fill=(196, 214, 170, 255))
        d.rectangle([x0, y0, x0 + 11, y0 + 9], outline=INK, width=1)
        for fx in (x0 + 4, x0 + 8):
            d.line([fx, y0, fx, y0 + 9], fill=(140, 160, 110, 255), width=1)
        d.line([x0 + 1, y0 + 7, x0 + 5, y0 + 4, x0 + 10, y0 + 5],
               fill=RED, width=1)
        d.point((x0 + 1, y0 + 7), fill=INK)
        d.point((x0 + 10, y0 + 5), fill=INK)
    elif key == "photos":  # picture glyph: frame, sun, mountains
        d.rectangle([x0, y0, x0 + 11, y0 + 9], fill=(206, 226, 240, 255))
        d.rectangle([x0, y0, x0 + 11, y0 + 9], outline=INK, width=1)
        d.ellipse([x0 + 7, y0 + 1, x0 + 9, y0 + 3], fill=(244, 196, 64, 255))
        d.polygon([(x0 + 1, y0 + 8), (x0 + 4, y0 + 4), (x0 + 7, y0 + 8)],
                  fill=(90, 140, 80, 255))
        d.polygon([(x0 + 5, y0 + 8), (x0 + 8, y0 + 5), (x0 + 10, y0 + 8)],
                  fill=(70, 118, 66, 255))
    elif key in ("book", "book_med", "book_sci"):
        d.rectangle([cx - 4, y0, cx + 4, y0 + 9], fill=(150, 96, 60, 255)
                    if key == "book" else (226, 224, 218, 255))
        d.rectangle([cx - 4, y0, cx + 4, y0 + 9], outline=INK, width=1)
        d.line([cx, y0, cx, y0 + 9], fill=INK, width=1)  # spine split
        if key == "book":
            for gy in (y0 + 2, y0 + 4, y0 + 6):
                d.line([cx - 3, gy, cx - 1, gy], fill=PAPER, width=1)
                d.line([cx + 1, gy, cx + 3, gy], fill=PAPER, width=1)
        elif key == "book_med":  # red cross on the cover
            d.rectangle([cx - 1, y0 + 2, cx + 1, y0 + 7], fill=RED)
            d.rectangle([cx - 2, y0 + 3, cx + 2, y0 + 5], fill=RED)
        else:  # flask badge on the cover
            d.polygon([(cx, y0 + 2), (cx + 1, y0 + 2), (cx + 3, y0 + 7),
                       (cx - 3, y0 + 7), (cx - 1, y0 + 2)], fill=TEAL)
    elif key == "drop":  # blood droplet
        d.polygon([(cx, y0), (cx + 3, y0 + 5), (cx + 2, y0 + 8),
                   (cx - 2, y0 + 8), (cx - 3, y0 + 5)], fill=RED)
        d.point((cx - 1, y0 + 5), fill=(240, 130, 120, 255))
    elif key == "chip":  # integrated circuit with pins
        d.rectangle([cx - 3, y0 + 2, cx + 3, y0 + 7], fill=INK)
        d.rectangle([cx - 1, y0 + 3, cx + 1, y0 + 5],
                    fill=(120, 220, 130, 255))
        for px in (cx - 3, cx, cx + 3):
            d.line([px, y0, px, y0 + 2], fill=INK, width=1)
            d.line([px, y0 + 7, px, y0 + 9], fill=INK, width=1)
        for py in (y0 + 3, y0 + 6):
            d.line([cx - 5, py, cx - 3, py], fill=INK, width=1)
            d.line([cx + 3, py, cx + 5, py], fill=INK, width=1)
    elif key == "lock":  # padlock
        d.rectangle([cx - 3, y0 + 4, cx + 3, y0 + 9], fill=(216, 178, 60, 255))
        d.rectangle([cx - 3, y0 + 4, cx + 3, y0 + 9], outline=INK, width=1)
        d.arc([cx - 2, y0, cx + 2, y0 + 6], start=180, end=360, fill=INK,
              width=2)
        d.point((cx, y0 + 6), fill=INK)
        d.line([cx, y0 + 6, cx, y0 + 7], fill=INK, width=1)
    elif key == "term":  # terminal prompt >_
        d.line([x0 + 1, y0 + 2, x0 + 4, y0 + 4], fill=(90, 230, 110, 255),
               width=1)
        d.line([x0 + 4, y0 + 4, x0 + 1, y0 + 6], fill=(90, 230, 110, 255),
               width=1)
        d.line([x0 + 6, y0 + 7, x0 + 10, y0 + 7], fill=(90, 230, 110, 255),
               width=1)
    elif key == "flask":
        d.polygon([(cx - 1, y0 + 2), (cx + 1, y0 + 2), (cx + 4, y0 + 8),
                   (cx - 4, y0 + 8)], outline=INK, width=1)
        d.line([cx - 2, y0 + 6, cx + 2, y0 + 6], fill=TEAL, width=2)
        d.rectangle([cx - 1, y0, cx + 1, y0 + 2], outline=INK, width=1)
        d.point((cx + 4, y0 + 1), fill=TEAL)
        d.point((cx - 4, y0 + 3), fill=TEAL)
    elif key == "math":  # pi-ish glyph + equals bars
        d.line([x0 + 1, y0 + 2, x0 + 6, y0 + 2], fill=INK, width=1)
        d.line([x0 + 2, y0 + 2, x0 + 2, y0 + 7], fill=INK, width=1)
        d.line([x0 + 5, y0 + 2, x0 + 5, y0 + 7], fill=INK, width=1)
        d.line([x0 + 8, y0 + 4, x0 + 10, y0 + 4], fill=INK, width=1)
        d.line([x0 + 8, y0 + 6, x0 + 10, y0 + 6], fill=INK, width=1)
    elif key == "cross":  # big red cross
        d.rectangle([cx - 1, y0, cx + 1, y0 + 9], fill=RED)
        d.rectangle([cx - 4, y0 + 3, cx + 4, y0 + 6], fill=RED)
    elif key == "box":  # archive box with a lid line and slot
        d.rectangle([x0 + 1, y0 + 2, x0 + 10, y0 + 8],
                    fill=(196, 164, 110, 255))
        d.rectangle([x0 + 1, y0 + 2, x0 + 10, y0 + 8], outline=INK, width=1)
        d.line([x0, y0 + 1, x0 + 11, y0 + 1], fill=(160, 128, 78, 255),
               width=1)
        d.line([cx - 2, y0 + 5, cx + 2, y0 + 5], fill=INK, width=1)
    elif key == "blank":  # useless: one faint dot on an empty label
        d.point((cx, cy), fill=(168, 168, 172, 255))


# ------------------------------------------------------------------- media

def draw_drive(d, rng):
    """abstract_software: a generic external drive."""
    body = (148, 150, 156, 255)
    d.rectangle([5, 9, 27, 23], fill=shade(body, 0.6))
    d.rectangle([4, 8, 26, 22], fill=body)
    d.rectangle([4, 8, 26, 22], outline=INK, width=1)
    d.ellipse([7, 10, 17, 20], fill=shade(body, 0.75))  # platter window
    d.ellipse([7, 10, 17, 20], outline=INK, width=1)
    d.ellipse([10, 13, 14, 17], fill=shade(body, 1.2))
    d.point((12, 15), fill=INK)
    d.rectangle([19, 11, 25, 16], fill=PAPER)  # blank label
    d.line([20, 13, 24, 13], fill=(170, 170, 174, 255), width=1)
    d.line([20, 15, 23, 15], fill=(170, 170, 174, 255), width=1)
    d.point((24, 20), fill=(90, 200, 100, 255))  # LED
    d.line([6, 24, 12, 27], fill=INK, width=2)  # cable stub
    d.line([12, 27, 15, 27], fill=INK, width=1)


# e-file id -> (label colour, icon key)
EFILES = {
    "efile_junk": ((150, 148, 142, 255), "junk"),
    "efile_lore": ((146, 110, 178, 255), "lore"),
    "efile_map": ((110, 164, 92, 255), "map"),
    "efile_photos": ((96, 150, 200, 255), "photos"),
    "efile_recipes": ((206, 138, 74, 255), "book"),
    "efile_recipes_medical": ((216, 220, 224, 255), "book_med"),
    "efile_recipes_science": ((84, 168, 160, 255), "book_sci"),
}


def draw_efile(d, rng, sid):
    """An SD-style memory card: clipped corner, label + icon, pins."""
    label, key = EFILES[sid]
    card = (52, 56, 66, 255)
    # body with the clipped top-right corner
    d.polygon([(8, 4), (20, 4), (24, 8), (24, 27), (8, 27)], fill=card)
    d.polygon([(8, 4), (20, 4), (24, 8), (24, 27), (8, 27)], outline=INK)
    d.rectangle([10, 6, 22, 21], fill=label)  # label
    d.rectangle([10, 6, 22, 21], outline=INK, width=1)
    icon(d, key, x0=10, y0=10)
    for px in range(11, 23, 3):  # gold contact pins along the bottom
        d.rectangle([px, 24, px + 1, 26], fill=(216, 178, 60, 255))
    d.line([8, 8, 8, 12], fill=(216, 178, 60, 255), width=1)  # lock tab


# software id -> (label colour, icon key, icon on dark?)
SOFTWARE = {
    "software_archived_data": ((216, 190, 130, 255), "box"),
    "software_blood_data": ((120, 26, 30, 255), "drop"),
    "software_electronics_reference": ((52, 120, 66, 255), "chip"),
    "software_encryption_codes": ((70, 76, 96, 255), "lock"),
    "software_hacking": ((18, 22, 20, 255), "term"),
    "software_lab_data": ((120, 190, 186, 255), "flask"),
    "software_math": ((84, 110, 190, 255), "math"),
    "software_medical": ((240, 240, 242, 255), "cross"),
    "software_useless": ((176, 176, 180, 255), "blank"),
}


def draw_disc(d, rng, sid):
    """A CD: silver disc, coloured label ring, icon at the centre top."""
    label, key = SOFTWARE[sid]
    cx, cy, r = 16, 16, 13
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(206, 208, 214, 255))
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=INK, width=1)
    # rainbow sheen arcs on the silver
    for rr, col in ((11, (190, 210, 235, 255)), (9, (215, 195, 225, 255))):
        d.arc([cx - rr, cy - rr, cx + rr, cy + rr], start=200, end=320,
              fill=col, width=1)
    d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], fill=label)  # label area
    d.ellipse([cx - 8, cy - 8, cx + 8, cy + 8], outline=INK, width=1)
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=(206, 208, 214, 255))
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], outline=INK, width=1)
    # icon sits in the label's upper half, hub stays clear below it:
    # draw a mini glyph band above the hub
    if key == "drop":
        d.polygon([(cx, 6), (cx + 2, 9), (cx + 1, 11), (cx - 1, 11),
                   (cx - 2, 9)], fill=(240, 120, 110, 255))
    elif key == "box":
        d.rectangle([cx - 3, 8, cx + 3, 11], fill=(150, 116, 62, 255))
        d.line([cx - 4, 7, cx + 4, 7], fill=(120, 90, 44, 255), width=1)
    elif key == "chip":
        d.rectangle([cx - 2, 7, cx + 2, 11], fill=INK)
        d.point((cx, 9), fill=(120, 220, 130, 255))
        d.line([cx - 4, 8, cx - 2, 8], fill=INK, width=1)
        d.line([cx + 2, 8, cx + 4, 8], fill=INK, width=1)
    elif key == "lock":
        d.rectangle([cx - 2, 8, cx + 2, 11], fill=(216, 178, 60, 255))
        d.arc([cx - 1, 5, cx + 1, 9], start=180, end=360,
              fill=(230, 230, 234, 255), width=1)
    elif key == "term":
        d.line([cx - 4, 7, cx - 2, 9], fill=(90, 230, 110, 255), width=1)
        d.line([cx - 2, 9, cx - 4, 11], fill=(90, 230, 110, 255), width=1)
        d.line([cx + 1, 10, cx + 4, 10], fill=(90, 230, 110, 255), width=1)
    elif key == "flask":
        d.polygon([(cx, 6), (cx + 2, 10), (cx - 2, 10)],
                  fill=(230, 244, 242, 255))
        d.point((cx, 9), fill=TEAL)
    elif key == "math":
        d.line([cx - 4, 7, cx + 1, 7], fill=(240, 240, 250, 255), width=1)
        d.line([cx - 3, 7, cx - 3, 11], fill=(240, 240, 250, 255), width=1)
        d.line([cx, 7, cx, 11], fill=(240, 240, 250, 255), width=1)
    elif key == "cross":
        d.rectangle([cx - 1, 6, cx + 1, 11], fill=RED)
        d.rectangle([cx - 2, 7, cx + 2, 9], fill=RED)
    elif key == "blank":
        d.point((cx, 8), fill=(140, 140, 144, 255))
    # catalogue lines on the label's lower arc
    d.line([cx - 5, cy + 5, cx - 2, cy + 6], fill=shade(label, 0.7), width=1)
    d.line([cx + 2, cy + 6, cx + 5, cy + 5], fill=shade(label, 0.7), width=1)


def draw_train_data(d, rng):
    """A dataset cartridge: chunky card, rail + train label."""
    body = (84, 74, 96, 255)
    d.rectangle([5, 7, 27, 24], fill=shade(body, 0.6))
    d.rectangle([4, 6, 26, 23], fill=body)
    d.rectangle([4, 6, 26, 23], outline=INK, width=1)
    d.rectangle([6, 8, 24, 19], fill=PAPER)
    d.rectangle([6, 8, 24, 19], outline=INK, width=1)
    # train front: window band, light, grille -- over rail tracks
    d.rectangle([12, 9, 18, 14], fill=(96, 130, 170, 255))
    d.rectangle([12, 9, 18, 14], outline=INK, width=1)
    d.line([15, 9, 15, 14], fill=INK, width=1)
    d.point((15, 15), fill=(250, 210, 90, 255))  # headlight
    for gx in (13, 17):
        d.line([gx, 16, gx, 17], fill=INK, width=1)
    # rails converging below the train, with sleepers
    d.line([9, 18, 13, 15], fill=INK, width=1)
    d.line([21, 18, 17, 15], fill=INK, width=1)
    d.line([8, 18, 22, 18], fill=WOOD_DARK, width=1)
    # dataset bars down the right side of the label
    for i, w in enumerate((3, 4, 2)):
        d.line([20, 9 + i * 3, 20 + w, 9 + i * 3],
               fill=(120, 120, 128, 255), width=1)
    for px in range(8, 24, 4):  # connector teeth
        d.rectangle([px, 21, px + 2, 23], fill=(216, 178, 60, 255))


# ----------------------------------------------------------------- weapons
# Diagonal axis: butt (5, 26) -> head base (21, 10); unit along (1,-1).

def shaft(d, x0=4, y0=27, x1=21, y1=10, col=WOOD, dark=WOOD_DARK, w=2):
    d.line([x0, y0, x1, y1], fill=dark, width=w + 1)
    d.line([x0, y0, x1, y1], fill=col, width=w)
    d.line([x0 + 1, y0 - 2, x1 - 2, y1 + 1], fill=WOOD_LIGHT, width=1)


def blades_around(d, cx, cy, col, dark, light, rng, jagged=False,
                  glassy=False):
    """Small blade teeth around the tepoztopili head's oval rim."""
    rim = [(cx - 5, cy + 2), (cx - 5, cy - 1), (cx - 4, cy - 4),
           (cx - 2, cy - 6), (cx + 1, cy - 6), (cx + 4, cy - 5),
           (cx + 6, cy - 3), (cx + 6, cy), (cx + 5, cy + 3),
           (cx + 3, cy + 5), (cx, cy + 6), (cx - 3, cy + 5)]
    for i, (x, y) in enumerate(rim):
        c = col if i % 3 else light
        if jagged and rng.random() < 0.3:
            continue  # a tooth snapped off
        d.point((x, y), fill=c)
        d.point((x + (1 if x >= cx else 0), y), fill=dark if i % 2 else c)
    if glassy:  # translucent-look flecks inside the rim
        for x, y in ((cx - 2, cy - 2), (cx + 2, cy + 1), (cx, cy + 3)):
            d.point((x, y), fill=(150, 210, 160, 255))


def draw_tepoztopili(d, rng, sid):
    shaft(d)
    cx, cy = 23, 9
    # oar-like oval wooden head, long axis along the shaft diagonal
    d.ellipse([cx - 6, cy - 6, cx + 7, cy + 7], fill=WOOD)
    d.ellipse([cx - 6, cy - 6, cx + 7, cy + 7], outline=WOOD_DARK, width=1)
    d.ellipse([cx - 4, cy - 4, cx + 5, cy + 5], fill=WOOD_LIGHT)
    if sid == "aztec_spear_glass":
        col, dark, light = ((30, 92, 56, 255), (16, 52, 32, 255),
                            (92, 168, 110, 255))
        blades_around(d, cx, cy, col, dark, light, rng, glassy=True)
    elif sid == "aztec_spear_scrap":
        col, dark, light = ((146, 104, 66, 255), (92, 62, 40, 255),
                            (178, 150, 118, 255))
        blades_around(d, cx, cy, col, dark, light, rng, jagged=True)
    elif sid == "aztec_spear_stone":
        col, dark, light = ((128, 126, 118, 255), (84, 82, 76, 255),
                            (172, 170, 162, 255))
        blades_around(d, cx, cy, col, dark, light, rng, jagged=True)
    else:
        grade = sid.split("_")[0]
        col, dark, light = STEELS[grade]
        blades_around(d, cx, cy, col, dark, light, rng)
        if grade == "qt":  # temper sheen flecks on the blades
            for x, y in ((cx - 5, cy - 1), (cx + 4, cy - 5), (cx, cy + 6)):
                d.point((x, y), fill=QT_SHEEN)
        if grade == "hc":  # polished glint
            d.point((cx + 6, cy - 3), fill=(255, 255, 255, 255))


def draw_halberd(d, rng):
    shaft(d, x1=23, y1=8)
    steel, dark, light = STEELS["lc"]
    # spear point above the axe head
    d.polygon([(23, 8), (26, 3), (27, 9), (25, 10)], fill=steel)
    d.polygon([(23, 8), (26, 3), (27, 9), (25, 10)], outline=dark)
    # axe head lashed to one side of the shaft
    d.polygon([(20, 12), (26, 10), (28, 13), (27, 18), (21, 16)],
              fill=steel)
    d.polygon([(20, 12), (26, 10), (28, 13), (27, 18), (21, 16)],
              outline=dark)
    d.line([26, 10, 28, 13], fill=light, width=1)  # cutting edge
    d.line([27, 18, 28, 13], fill=light, width=1)
    for lx, ly in ((20, 13), (21, 15)):  # lashings
        d.line([lx - 1, ly + 1, lx + 2, ly - 2], fill=WOOD_DARK, width=1)


def draw_javelin(d, rng, iron):
    # slimmer, shorter throwing shaft
    d.line([6, 26, 22, 10], fill=WOOD_DARK, width=2)
    d.line([6, 26, 22, 10], fill=WOOD, width=1)
    if iron:
        steel, dark, light = STEELS["lc"]
        d.polygon([(22, 10), (27, 4), (28, 5), (25, 12), (23, 12)],
                  fill=steel)
        d.polygon([(22, 10), (27, 4), (28, 5), (25, 12), (23, 12)],
                  outline=dark)
        d.line([27, 4, 25, 11], fill=light, width=1)
    else:
        # fire-hardened point: wood darkening to a charred tip
        d.polygon([(22, 10), (27, 4), (28, 6), (24, 12)], fill=WOOD_DARK)
        d.polygon([(25, 7), (27, 4), (28, 6), (26, 8)],
                  fill=(40, 30, 22, 255))
        d.point((27, 5), fill=(16, 12, 10, 255))


def draw_naginata(d, rng, tempered):
    shaft(d, x1=19, y1=12)
    steel, dark, light = STEELS["qt" if tempered else "mc"]
    # long curved machete blade: wide, single-edged, clipped tip
    d.polygon([(19, 12), (24, 5), (29, 2), (30, 4), (27, 10), (22, 14)],
              fill=steel)
    d.polygon([(19, 12), (24, 5), (29, 2), (30, 4), (27, 10), (22, 14)],
              outline=dark)
    d.line([24, 5, 29, 2], fill=light, width=1)  # spine highlight
    d.line([27, 10, (30, 4)[0], 4], fill=light, width=1)
    d.line([22, 14, 27, 10], fill=dark, width=1)  # cutting edge shade
    if tempered:  # heat-tint band along the blade
        d.line([23, 8, 28, 4], fill=QT_SHEEN, width=1)
        d.line([24, 11, 28, 6], fill=(150, 110, 190, 255), width=1)
    for lx, ly in ((19, 12), (20, 13)):  # mount lashings
        d.line([lx - 1, ly + 1, lx + 2, ly - 2], fill=WOOD_DARK, width=1)


def draw_rebar(d, rng, sharp):
    # ribbed dark steel rod as the whole weapon
    dark = (52, 52, 56, 255)
    base = (92, 92, 98, 255)
    d.line([5, 26, 21, 10], fill=dark, width=3)
    d.line([5, 26, 21, 10], fill=base, width=2)
    for t in range(1, 8):  # rib ticks across the rod
        x, y = 5 + t * 2, 26 - t * 2
        d.line([x - 1, y, x + 1, y + 1], fill=dark, width=1)
    if sharp:
        # cleanly cut point: symmetric, bright fresh-cut faces
        d.polygon([(21, 10), (27, 3), (28, 6), (24, 12)], fill=base)
        d.polygon([(21, 10), (27, 3), (28, 6), (24, 12)], outline=dark)
        d.line([27, 3, 24, 11], fill=(196, 198, 204, 255), width=1)
    else:
        # crudely hammered point: irregular, bent facets
        d.polygon([(21, 10), (25, 4), (28, 5), (26, 8), (28, 9),
                   (24, 12)], fill=base)
        d.polygon([(21, 10), (25, 4), (28, 5), (26, 8), (28, 9),
                   (24, 12)], outline=dark)
        d.line([25, 4, 26, 8], fill=dark, width=1)
        d.point((27, 7), fill=(150, 150, 156, 255))


def draw_pipe(d, rng):
    steel, dark, light = STEELS["lc"]
    # hollow pipe shaft: two walls with a bright bore line between
    d.line([5, 27, 20, 12], fill=dark, width=4)
    d.line([5, 27, 20, 12], fill=steel, width=3)
    d.line([6, 25, 19, 12], fill=light, width=1)
    # open butt end: dark hollow ellipse
    d.ellipse([3, 25, 8, 30], fill=dark)
    d.ellipse([4, 26, 7, 29], fill=(20, 20, 24, 255))
    # angled cut tip: a long bevel, bright cut face
    d.polygon([(20, 12), (28, 3), (29, 5), (23, 14)], fill=steel)
    d.polygon([(20, 12), (28, 3), (29, 5), (23, 14)], outline=dark)
    d.line([28, 3, 23, 13], fill=(238, 240, 244, 255), width=1)


def draw_wood_spear(d, rng):
    shaft(d, x1=23, y1=8, w=3)
    # whittled point: the shaft itself tapers, lighter fresh wood
    d.polygon([(23, 8), (28, 2), (29, 4), (25, 11)], fill=WOOD_LIGHT)
    d.polygon([(23, 8), (28, 2), (29, 4), (25, 11)], outline=WOOD_DARK)
    d.line([28, 2, 25, 10], fill=WOOD, width=1)  # whittle facet
    d.point((26, 6), fill=WOOD_DARK)  # knot


def draw_partigiana(d, rng):
    steel, dark, light = STEELS["qt"]
    # one continuous piece of tempered steel: slim steel shaft
    d.line([5, 26, 20, 11], fill=dark, width=3)
    d.line([5, 26, 20, 11], fill=steel, width=2)
    d.line([6, 24, 19, 11], fill=light, width=1)
    d.point((5, 27), fill=light)  # rounded butt cap, same piece
    # broad cutting-stabbing head
    d.polygon([(20, 11), (25, 4), (29, 1), (30, 3), (28, 9), (24, 13),
               (21, 13)], fill=steel)
    d.polygon([(20, 11), (25, 4), (29, 1), (30, 3), (28, 9), (24, 13),
               (21, 13)], outline=dark)
    d.line([25, 4, 28, 9], fill=light, width=1)  # central ridge
    d.line([23, 8, 27, 5], fill=QT_SHEEN, width=1)  # temper sheen
    # small hooks on either side of the head base
    d.line([21, 12, 18, 13], fill=dark, width=2)  # rear hook
    d.point((18, 12), fill=light)
    d.line([24, 13, 24, 16], fill=dark, width=2)  # lower hook
    d.point((25, 16), fill=light)


# -------------------------------------------------------------------- main

TEPOZT = {"aztec_spear_glass", "aztec_spear_scrap", "aztec_spear_stone",
          "ch_aztec_spear", "hc_aztec_spear", "lc_aztec_spear",
          "mc_aztec_spear", "qt_aztec_spear"}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "abstract_software":
        draw_drive(d, rng)
    elif sid in EFILES:
        draw_efile(d, rng, sid)
    elif sid in SOFTWARE:
        draw_disc(d, rng, sid)
    elif sid == "train_data":
        draw_train_data(d, rng)
    elif sid in TEPOZT:
        draw_tepoztopili(d, rng, sid)
    elif sid == "halberd_crude":
        draw_halberd(d, rng)
    elif sid == "javelin":
        draw_javelin(d, rng, iron=False)
    elif sid == "javelin_iron":
        draw_javelin(d, rng, iron=True)
    elif sid == "naginata_machete":
        draw_naginata(d, rng, tempered=False)
    elif sid == "naginata_machete_temp":
        draw_naginata(d, rng, tempered=True)
    elif sid == "pointed_rebar":
        draw_rebar(d, rng, sharp=False)
    elif sid == "sharp_rebar":
        draw_rebar(d, rng, sharp=True)
    elif sid == "sharpened_pipe":
        draw_pipe(d, rng)
    elif sid == "spear_wood_simple":
        draw_wood_spear(d, rng)
    elif sid == "string_dimension_partigiana_ixwa":
        draw_partigiana(d, rng)
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
    # contact-sheet preview: 6 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item64.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item64.png {prev.size}")


if __name__ == "__main__":
    main()
