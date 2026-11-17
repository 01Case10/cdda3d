#!/usr/bin/env python3
"""ITEM48: HEW printouts + HEW/MWS weather datasheets -- CDDA Voxel
coverage audit.

All 29 items are paper.  Three layouts:

  HEW printouts (24) -- a short sheet of paper, most of it coded in
    complex equations and data.  Shared base form: a header band
    carrying the printed title code as dark glyph blocks (code
    characters encoded as glyph-bar patterns), dense gray
    equation/data squiggle rows, one small subject motif icon in a
    framed box, and a footer status strip.  Variants:
      HEW_printout_data (no code): green footer ("CONDITIONS
        NORMAL"), plain checkmark / flat-baseline motif.
      _portal: green footer;  _portal_storm: orange/red footer;
      _inner_cabins_warped_cabin: the whole sheet is error
        messages ("VALUE OUTSIDE EXPECTED") -- red error lines
        throughout and a red footer.
  HEW_weather_data -- a datasheet, chart-forward: a chart curve,
    a small radiation trefoil, and a data-table grid; no title
    header band.
  MWS datasheets (4) -- complex weather recordings:
      mws_weather_data: cloud + temperature/pressure curves;
      mws_weather_data_incomplete: a small curled/torn scrap of
        unspooled thermal paper, a few partial faded lines;
      mws_portal_storm_weather_data: chart interrupted by red
        error rows;
      mws_monster_corpse_weather_data: alien "atmospheric" sheet
        from inside a giant dead creature -- organic curves, rib
        arcs, faint flesh-pink paper.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "HEW_printout_data",
    "HEW_printout_data_amigara",
    "HEW_printout_data_barricaded_church",
    "HEW_printout_data_cabin_reverb",
    "HEW_printout_data_cabin_reverb_dimension",
    "HEW_printout_data_exodii",
    "HEW_printout_data_highlands",
    "HEW_printout_data_inner_cabins_home_cabin",
    "HEW_printout_data_inner_cabins_open_field",
    "HEW_printout_data_inner_cabins_warped_cabin",
    "HEW_printout_data_labyrinth",
    "HEW_printout_data_lixa",
    "HEW_printout_data_monster_corpse",
    "HEW_printout_data_morgantown",
    "HEW_printout_data_physics_lab",
    "HEW_printout_data_portal",
    "HEW_printout_data_portal_storm",
    "HEW_printout_data_portal_storm_dungeon",
    "HEW_printout_data_radiosphere",
    "HEW_printout_data_spiral_mine",
    "HEW_printout_data_strange_temple",
    "HEW_printout_data_string_dimension",
    "HEW_printout_data_vitrified",
    "HEW_printout_data_void_spider_lair",
    "HEW_weather_data",
    "mws_monster_corpse_weather_data",
    "mws_portal_storm_weather_data",
    "mws_weather_data",
    "mws_weather_data_incomplete",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

PAPER = (240, 236, 226, 255)
PAPER_DARK = (206, 200, 184, 255)
PAPER_PINK = (246, 226, 218, 255)
INK = (52, 52, 58, 255)
SQUIG = (148, 148, 152, 255)
SQUIG_DARK = (110, 110, 116, 255)
HEADER_BG = (196, 198, 204, 255)
FOOTER = (168, 170, 176, 255)
GREEN = (62, 148, 74, 255)
RED = (196, 48, 40, 255)
RED_DARK = (148, 30, 26, 255)
ORANGE = (226, 110, 36, 255)
PURPLE = (128, 84, 178, 255)
TEAL = (44, 140, 132, 255)
BROWN = (122, 88, 54, 255)
WOOD = (96, 66, 38, 255)
LEAF = (70, 132, 62, 255)

# sid -> (title code, motif key, footer colour or None for default)
HEW = {
    "HEW_printout_data": ("", "normal", GREEN),
    "HEW_printout_data_amigara": ("01ITO", "amigara", None),
    "HEW_printout_data_barricaded_church": ("ESFX", "church", None),
    "HEW_printout_data_cabin_reverb": ("CITW01", "cabin", None),
    "HEW_printout_data_cabin_reverb_dimension": ("CITW02", "cabin_dim",
                                                 None),
    "HEW_printout_data_exodii": ("IAE20", "exodii", None),
    "HEW_printout_data_highlands": ("TCOB1", "highlands", None),
    "HEW_printout_data_inner_cabins_home_cabin": ("CITW-BFγ", "cabin_home",
                                                  None),
    "HEW_printout_data_inner_cabins_open_field": ("CITW-BFα", "cabin_field",
                                                  None),
    "HEW_printout_data_inner_cabins_warped_cabin": ("CITW-BFβ", "error",
                                                    RED),
    "HEW_printout_data_labyrinth": ("DB86", "labyrinth", None),
    "HEW_printout_data_lixa": ("DP72063", "lixa", None),
    "HEW_printout_data_monster_corpse": ("LKG1", "corpse", None),
    "HEW_printout_data_morgantown": ("01Z90", "town", None),
    "HEW_printout_data_physics_lab": ("22BMBS", "flask", None),
    "HEW_printout_data_portal": ("STABLE PORTAL", "portal", GREEN),
    "HEW_printout_data_portal_storm": ("UNSTABLE PORTAL", "portal_storm",
                                       ORANGE),
    "HEW_printout_data_portal_storm_dungeon": ("MROTS-S", "portal_dungeon",
                                               None),
    "HEW_printout_data_radiosphere": ("VKTRS", "radiosphere", None),
    "HEW_printout_data_spiral_mine": ("UJI98", "spiral_mine", None),
    "HEW_printout_data_strange_temple": ("WKG1", "temple", None),
    "HEW_printout_data_string_dimension": ("IHS", "strings", None),
    "HEW_printout_data_vitrified": ("CC1044", "vitrified", None),
    "HEW_printout_data_void_spider_lair": ("NLL8", "spider", None),
}

# sheet geometry (printouts + full datasheets)
SX0, SX1, SY0, SY1 = 5, 26, 2, 29


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def sheet(d, paper=PAPER):
    """Paper sheet with a drop edge and border."""
    d.rectangle([SX0 + 1, SY0 + 1, SX1 + 1, SY1 + 1], fill=PAPER_DARK)
    d.rectangle([SX0, SY0, SX1, SY1], fill=paper)
    d.rectangle([SX0, SY0, SX1, SY1], outline=INK, width=1)


def squiggles(d, rng, y0, y1, skip=None):
    """Dense gray equation/data rows; skip=(x0,x1) leaves a gap."""
    for y in range(y0, y1 + 1, 2):
        x = SX0 + 2
        while x < SX1 - 3:
            w = rng.randint(2, 6)
            if skip and skip[0] - 1 <= x <= skip[1]:
                x = skip[1] + 1
                continue
            col = SQUIG if rng.random() < 0.6 else SQUIG_DARK
            d.line([x, y, min(x + w, SX1 - 2), y], fill=col, width=1)
            if rng.random() < 0.25:  # a taller glyph in the row
                d.point((x + 1, y - 1), fill=SQUIG_DARK)
            x += w + 2


def header(d, code):
    """Header band with the title code as dark glyph blocks."""
    d.rectangle([SX0 + 1, SY0 + 1, SX1 - 1, SY0 + 5], fill=HEADER_BG)
    d.line([SX0 + 1, SY0 + 5, SX1 - 1, SY0 + 5], fill=INK, width=1)
    if not code:
        d.line([SX0 + 3, SY0 + 3, SX0 + 8, SY0 + 3], fill=INK, width=1)
        return
    avail = (SX1 - 2) - (SX0 + 2)
    step = 2 if len(code) * 2 - 1 <= avail else 1
    x = SX0 + 2
    for ch in code:
        o = ord(ch)
        if ch != " ":
            for col in range(2 if step == 2 else 1):
                bits = (o >> (col * 2)) & 0b111
                for row in range(3):
                    if bits & (1 << row):
                        d.point((x + col, SY0 + 2 + row), fill=INK)
                if bits == 0:
                    d.point((x + col, SY0 + 3), fill=INK)
        x += step + (1 if step == 2 else 0)
        if x > SX1 - 3:
            break


def footer(d, colour):
    d.rectangle([SX0 + 1, SY1 - 3, SX1 - 1, SY1 - 1],
                fill=colour if colour else FOOTER)
    d.line([SX0 + 3, SY1 - 2, SX0 + 9, SY1 - 2],
           fill=(240, 240, 240, 255) if colour else INK, width=1)


# ------------------------------------------------------------------ motifs
# Motif box: x 9..22, y 12..21 (drawn over a framed paper inset).

def cabin_shape(d, cx, cy, warm=False, small=False):
    w, h = (6, 4) if small else (9, 6)
    x0, x1 = cx - w // 2, cx + w // 2
    y1 = cy + h // 2
    y0 = y1 - h
    d.polygon([(x0 - 1, y0 + 2), (cx, y0 - 2), (x1 + 1, y0 + 2)], fill=WOOD)
    d.rectangle([x0, y0 + 2, x1, y1], fill=BROWN)
    d.rectangle([cx - 1, y0 + 3, cx, y1], fill=WOOD)  # door
    win = (250, 190, 80, 255) if warm else (210, 220, 228, 255)
    d.point((x0 + 1, y0 + 3), fill=win)
    d.point((x1 - 1, y0 + 3), fill=win)
    if warm:  # chimney with a smoke curl
        d.rectangle([x1 - 2, y0 - 3, x1 - 1, y0 - 1], fill=WOOD)
        d.point((x1 - 1, y0 - 4), fill=SQUIG)


def motif(d, rng, key):
    x0, x1, y0, y1 = 9, 22, 12, 21
    cx, cy = 15, 16
    d.rectangle([x0, y0, x1, y1], fill=shade(PAPER, 0.96))
    d.rectangle([x0, y0, x1, y1], outline=SQUIG_DARK, width=1)
    if key == "normal":
        d.line([x0 + 2, y1 - 2, x1 - 2, y1 - 2], fill=SQUIG_DARK, width=1)
        d.line([x0 + 2, y0 + 5, x0 + 5, y0 + 8], fill=GREEN, width=1)
        d.line([x0 + 5, y0 + 8, x1 - 3, y0 + 2], fill=GREEN, width=1)
    elif key == "amigara":
        d.polygon([(x0 + 1, y1 - 1), (x0 + 3, y0 + 2), (cx, y0 + 4),
                   (x1 - 3, y0 + 1), (x1 - 1, y1 - 1)], fill=(96, 96, 102, 255))
        # human-shaped hole in the slope: paper-coloured figure
        d.point((cx, cy - 1), fill=PAPER)
        d.rectangle([cx, cy, cx, cy + 3], fill=PAPER)
        d.line([cx - 1, cy + 1, cx + 1, cy + 1], fill=PAPER, width=1)
    elif key == "church":
        d.rectangle([cx - 3, y0 + 4, cx + 3, y1 - 1], fill=(188, 182, 168, 255))
        d.polygon([(cx - 4, y0 + 4), (cx, y0 + 1), (cx + 4, y0 + 4)],
                  fill=WOOD)
        d.line([cx, y0 - 1 + 1, cx, y0 + 2], fill=INK, width=1)
        d.line([cx - 1, y0 + 1, cx + 1, y0 + 1], fill=INK, width=1)
        for wx in (cx - 2, cx + 1):  # boarded windows: X of planks
            d.line([wx, y0 + 6, wx + 1, y0 + 8], fill=BROWN, width=1)
            d.line([wx + 1, y0 + 6, wx, y0 + 8], fill=BROWN, width=1)
    elif key in ("cabin", "cabin_dim", "cabin_home", "cabin_field"):
        if key == "cabin_field":
            d.line([x0 + 1, y1 - 2, x1 - 1, y1 - 2], fill=LEAF, width=1)
            for gx in range(x0 + 2, x1 - 1, 3):
                d.point((gx, y1 - 3), fill=LEAF)
            cabin_shape(d, cx, cy, small=True)
        else:
            for tx in (x0 + 1, x1 - 2):  # pine trees flanking
                d.polygon([(tx, y0 + 5), (tx + 1, y0 + 1), (tx + 2, y0 + 5)],
                          fill=LEAF)
                d.point((tx + 1, y1 - 3), fill=WOOD)
            if key == "cabin_dim":  # warped/mirrored ghost, purple accents
                d.rectangle([cx - 5, y0 + 4, cx + 4, y1 - 2],
                            outline=PURPLE, width=1)
                d.line([cx - 5, y0 + 4, cx + 5, y1 - 2], fill=PURPLE,
                       width=1)
            cabin_shape(d, cx, cy, warm=(key == "cabin_home"))
            if key == "cabin_dim":
                for px, py in ((cx - 4, y0 + 3), (cx + 4, y0 + 6),
                               (cx + 2, y1 - 2)):
                    d.point((px, py), fill=PURPLE)
    elif key == "exodii":
        d.rectangle([cx - 2, y0 + 2, cx + 2, y0 + 5], fill=(90, 98, 108, 255))
        d.point((cx - 1, y0 + 3), fill=TEAL)
        d.point((cx + 1, y0 + 3), fill=TEAL)
        d.line([cx, y0 + 2, cx, y0 + 0], fill=INK, width=1)
        d.point((cx, y0), fill=RED)
        d.polygon([(cx - 3, y1 - 1), (cx - 1, y0 + 6), (cx + 1, y0 + 6),
                   (cx + 3, y1 - 1)], fill=(90, 98, 108, 255))
        d.line([cx - 3, y0 + 7, cx - 5, y0 + 10], fill=INK, width=1)
        d.line([cx + 3, y0 + 7, cx + 5, y0 + 10], fill=INK, width=1)
    elif key == "highlands":
        d.arc([x0 + 1, y0 + 3, cx + 3, y1 + 3], start=180, end=360,
              fill=LEAF, width=2)
        d.arc([cx - 2, y0 + 5, x1 - 1, y1 + 4], start=180, end=360,
              fill=(96, 150, 84, 255), width=2)
        d.line([x0 + 1, y1 - 1, x1 - 1, y1 - 1], fill=LEAF, width=1)
    elif key == "labyrinth":
        for gx in range(x0 + 2, x1 - 1, 3):
            d.line([gx, y0 + 1, gx, y1 - 1], fill=INK, width=1)
        gaps = [(x0 + 2, y0 + 4), (x0 + 5, y0 + 7), (x0 + 8, y0 + 3),
                (x0 + 11, y0 + 6)]
        for gx, gy in gaps:
            d.point((gx, gy), fill=shade(PAPER, 0.96))
            d.point((gx, gy + 1), fill=shade(PAPER, 0.96))
        for gy in range(y0 + 3, y1 - 1, 3):
            d.line([x0 + 1, gy, x1 - 1, gy], fill=INK, width=1)
            d.point((x0 + 1 + (gy % 3), gy), fill=shade(PAPER, 0.96))
    elif key == "lixa":
        d.arc([cx - 4, y0 + 2, cx + 4, y1 + 4], start=180, end=360,
              fill=PURPLE, width=2)
        d.line([cx - 4, y0 + 6, cx - 4, y1 - 1], fill=PURPLE, width=1)
        d.line([cx + 4, y0 + 6, cx + 4, y1 - 1], fill=PURPLE, width=1)
        d.ellipse([cx - 2, y0 + 4, cx + 2, y0 + 8], outline=TEAL, width=1)
        d.point((cx, y0 + 6), fill=TEAL)
    elif key == "corpse":
        for r in range(1, 5):  # ribcage arcs
            d.arc([cx - 2 - r, y0 + 1, cx + 2 + r, y0 + 4 + 2 * r],
                  start=20, end=160, fill=(176, 164, 140, 255), width=1)
        d.line([cx, y0 + 2, cx, y1 - 1], fill=(140, 128, 104, 255),
               width=1)
    elif key == "town":
        heights = [5, 8, 6, 9, 4]
        bx = x0 + 1
        for h in heights:
            d.rectangle([bx, y1 - 1 - h, bx + 2, y1 - 1],
                        fill=(104, 108, 116, 255))
            d.point((bx + 1, y1 - 3), fill=(250, 220, 120, 255))
            bx += 3
    elif key == "flask":
        d.polygon([(cx - 1, y0 + 4), (cx + 1, y0 + 4), (cx + 4, y1 - 2),
                   (cx - 4, y1 - 2)], outline=INK, width=1)
        d.line([cx - 2, y1 - 4, cx + 2, y1 - 4], fill=TEAL, width=2)
        d.rectangle([cx - 1, y0 + 1, cx + 1, y0 + 4], outline=INK, width=1)
        for bx, by in ((cx + 5, y0 + 3), (cx - 5, y0 + 5), (cx + 4, y0 + 7)):
            d.point((bx, by), fill=TEAL)  # atom/bubble sparks
    elif key in ("portal", "portal_storm", "portal_dungeon"):
        if key == "portal_dungeon":  # dungeon bricks under the swirl
            for by in (y1 - 3, y1 - 1):
                d.line([x0 + 1, by, x1 - 1, by], fill=(120, 96, 84, 255),
                       width=1)
            for bx in range(x0 + 3, x1 - 1, 4):
                d.point((bx, y1 - 2), fill=(120, 96, 84, 255))
        col = TEAL if key == "portal" else ORANGE
        for r in range(4, 0, -1):
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      outline=col if r % 2 else SQUIG_DARK, width=1)
        if key == "portal_storm":  # jagged bolts off the swirl
            for bx, by in ((cx - 6, y0 + 2), (cx + 5, y0 + 4),
                           (cx - 5, y1 - 3)):
                d.line([bx, by, bx + 1, by + 1], fill=RED, width=1)
                d.line([bx + 1, by + 1, bx, by + 2], fill=RED, width=1)
        d.point((cx, cy), fill=col)
    elif key == "radiosphere":
        for r in (5, 3, 1):
            d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=TEAL,
                      width=1)
        d.ellipse([cx - 6, cy - 2, cx + 6, cy + 2], outline=PURPLE,
                  width=1)
        d.point((cx, cy), fill=RED)
    elif key == "spiral_mine":
        pts = []
        for t in range(0, 26):
            a = t * 0.55
            r = 5.0 - t * 0.16
            pts.append((cx + r * math.cos(a), cy - 3 + t * 0.32
                        + r * math.sin(a) * 0.4))
        d.line(pts, fill=INK, width=1)
        d.line([cx - 4, y1 - 1, cx + 4, y1 - 1], fill=WOOD, width=1)
        d.line([cx - 3, y1 - 3, cx - 3, y1 - 1], fill=WOOD, width=1)
        d.line([cx + 3, y1 - 3, cx + 3, y1 - 1], fill=WOOD, width=1)
    elif key == "temple":
        d.polygon([(x0 + 1, y0 + 3), (cx, y0), (x1 - 1, y0 + 3)],
                  fill=(176, 164, 140, 255))
        d.line([x0 + 1, y0 + 4, x1 - 1, y0 + 4], fill=(140, 128, 104, 255),
               width=1)
        for px in (x0 + 3, cx - 1, x1 - 4):
            d.rectangle([px, y0 + 5, px + 1, y1 - 3],
                        fill=(188, 176, 150, 255))
        d.line([x0 + 1, y1 - 2, x1 - 1, y1 - 2], fill=(140, 128, 104, 255),
               width=1)
    elif key == "strings":
        for i, amp in enumerate((1, 2, 1, 3)):
            yy = y0 + 2 + i * 2
            pts = [(x, yy + amp * math.sin((x - x0) * 0.9 + i))
                   for x in range(x0 + 1, x1)]
            d.line(pts, fill=PURPLE if i == 3 else INK, width=1)
    elif key == "vitrified":
        d.rectangle([x0 + 1, y0 + 1, x1 - 1, y1 - 1],
                    fill=(196, 208, 212, 255))
        cracks = [((x0 + 3, y0 + 2), (x0 + 5, y0 + 5), (x0 + 4, y0 + 8)),
                  ((x0 + 5, y0 + 5), (x0 + 8, y0 + 6), (x1 - 2, y0 + 5)),
                  ((x0 + 5, y0 + 5), (x0 + 7, y0 + 8), (x0 + 6, y1 - 1)),
                  ((x0 + 8, y0 + 6), (x1 - 3, y0 + 8))]
        for seg in cracks:
            d.line(list(seg), fill=INK, width=1)
    elif key == "spider":
        for dx, dy in ((-5, -3), (0, -4), (5, -3), (-5, 3), (0, 4), (5, 3),
                       (-6, 0), (6, 0)):  # web radials
            d.line([cx, cy, cx + dx, cy + dy], fill=SQUIG_DARK, width=1)
        d.ellipse([cx - 4, cy - 3, cx + 4, cy + 3], outline=SQUIG_DARK,
                  width=1)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=INK)  # spider
        for lx, ly in ((-3, -2), (3, -2), (-3, 2), (3, 2)):
            d.line([cx, cy, cx + lx, cy + ly], fill=INK, width=1)
        d.point((cx - 1, cy - 1), fill=RED)
        d.point((cx + 1, cy - 1), fill=RED)


def draw_printout(d, rng, sid):
    code, key, foot = HEW[sid]
    sheet(d)
    if key == "error":
        # the whole document is error messages: red lines throughout
        header(d, code)
        for y in range(SY0 + 7, SY1 - 4, 2):
            x = SX0 + 2
            d.point((SX0 + 2, y), fill=RED)
            while x < SX1 - 3:
                w = rng.randint(2, 5)
                d.line([x + 1, y, min(x + w, SX1 - 2), y],
                       fill=RED if rng.random() < 0.7 else RED_DARK,
                       width=1)
                x += w + 2
        footer(d, RED)
        return
    header(d, code)
    squiggles(d, rng, SY0 + 7, SY0 + 10)
    motif(d, rng, key)
    squiggles(d, rng, 23, SY1 - 5)
    footer(d, foot)


# --------------------------------------------------------------- datasheets

def trefoil(d, cx, cy):
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=(240, 200, 60, 255))
    d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], outline=INK, width=1)
    for dx, dy in ((0, -2), (-2, 1), (2, 1)):
        d.point((cx + dx, cy + dy), fill=INK)
    d.point((cx, cy), fill=INK)


def chart(d, rng, x0, x1, y0, y1, curves):
    d.line([x0, y0, x0, y1], fill=INK, width=1)
    d.line([x0, y1, x1, y1], fill=INK, width=1)
    for gx in range(x0 + 3, x1, 3):  # grid
        d.line([gx, y0, gx, y1], fill=(214, 212, 206, 255), width=1)
    for col, amp, phase in curves:
        pts = [(x, y1 - 2 - amp * (0.5 + 0.5 * math.sin(
            (x - x0) * 0.55 + phase)) - rng.random() * 0.6)
            for x in range(x0 + 1, x1)]
        d.line(pts, fill=col, width=1)


def draw_hew_weather(d, rng):
    sheet(d)
    d.line([SX0 + 2, SY0 + 2, SX0 + 9, SY0 + 2], fill=INK, width=2)
    d.line([SX0 + 2, SY0 + 4, SX0 + 6, SY0 + 4], fill=SQUIG_DARK, width=1)
    chart(d, rng, SX0 + 2, SX1 - 7, SY0 + 6, SY0 + 15,
          [(TEAL, 5.0, 0.0), (SQUIG_DARK, 3.0, 1.4)])
    trefoil(d, SX1 - 4, SY0 + 10)
    # data-table grid below the chart
    tx0, tx1, ty0, ty1 = SX0 + 2, SX1 - 2, SY0 + 17, SY1 - 2
    d.rectangle([tx0, ty0, tx1, ty1], outline=INK, width=1)
    for gy in range(ty0 + 3, ty1, 3):
        d.line([tx0, gy, tx1, gy], fill=SQUIG, width=1)
    for gx in range(tx0 + 5, tx1, 5):
        d.line([gx, ty0, gx, ty1], fill=SQUIG, width=1)
    for gy in range(ty0 + 1, ty1, 3):  # cell ticks
        for gx in range(tx0 + 1, tx1 - 1, 5):
            if rng.random() < 0.7:
                d.point((gx, gy), fill=SQUIG_DARK)


def draw_mws_weather(d, rng):
    sheet(d)
    # cloud icon, top left
    d.ellipse([SX0 + 2, SY0 + 2, SX0 + 9, SY0 + 6], fill=(188, 200, 214, 255))
    d.ellipse([SX0 + 4, SY0 + 1, SX0 + 11, SY0 + 6],
              fill=(188, 200, 214, 255))
    d.line([SX0 + 3, SY0 + 8, SX0 + 5, SY0 + 10], fill=(84, 120, 190, 255),
           width=1)
    d.line([SX0 + 7, SY0 + 8, SX0 + 9, SY0 + 10], fill=(84, 120, 190, 255),
           width=1)
    d.line([SX0 + 13, SY0 + 3, SX1 - 2, SY0 + 3], fill=INK, width=1)
    d.line([SX0 + 13, SY0 + 5, SX1 - 4, SY0 + 5], fill=SQUIG_DARK, width=1)
    chart(d, rng, SX0 + 2, SX1 - 2, SY0 + 12, SY1 - 6,
          [(RED, 6.0, 0.4), ((84, 120, 190, 255), 4.0, 2.0)])
    squiggles(d, rng, SY1 - 4, SY1 - 4)
    footer(d, None)


def draw_mws_incomplete(d, rng):
    # small curled/torn scrap of thermal paper, centred in frame
    d.polygon([(10, 10), (23, 9), (24, 21), (18, 23), (11, 22)],
              fill=(228, 224, 214, 255))
    d.polygon([(10, 10), (23, 9), (24, 21), (18, 23), (11, 22)],
              outline=SQUIG_DARK)
    d.polygon([(21, 20), (24, 21), (18, 23), (20, 22)],
              fill=PAPER_DARK)  # curled corner
    for i, y in enumerate((12, 15, 18)):  # a few partial faded lines
        w = (9, 6, 4)[i]
        d.line([12, y, 12 + w, y], fill=(178, 176, 170, 255), width=1)
    d.point((13, 20), fill=(178, 176, 170, 255))


def draw_mws_portal_storm(d, rng):
    sheet(d)
    d.line([SX0 + 2, SY0 + 2, SX0 + 10, SY0 + 2], fill=INK, width=1)
    chart(d, rng, SX0 + 2, SX1 - 2, SY0 + 5, SY0 + 13,
          [((84, 120, 190, 255), 4.0, 0.8)])
    # red error rows interrupting the sheet
    for y in (SY0 + 15, SY0 + 18, SY0 + 21):
        d.rectangle([SX0 + 1, y, SX1 - 1, y + 1], fill=(248, 226, 222, 255))
        x = SX0 + 2
        while x < SX1 - 3:
            w = rng.randint(2, 5)
            d.line([x, y, min(x + w, SX1 - 2), y], fill=RED, width=1)
            x += w + 2
    chart(d, rng, SX0 + 2, SX1 - 2, SY0 + 23, SY1 - 2,
          [(RED, 2.5, 2.6)])


def draw_mws_corpse(d, rng):
    sheet(d, paper=PAPER_PINK)
    d.line([SX0 + 2, SY0 + 2, SX0 + 9, SY0 + 2],
           fill=(150, 92, 92, 255), width=1)
    # rib arcs across the top half
    for r in range(1, 5):
        d.arc([16 - 2 - r, SY0 + 3, 16 + 2 + r, SY0 + 6 + 2 * r],
              start=20, end=160, fill=(176, 130, 118, 255), width=1)
    # organic curves instead of a ruled chart
    for i, (amp, ph) in enumerate(((3.0, 0.0), (2.0, 1.2), (4.0, 2.4))):
        col = (178, 96, 96, 255) if i == 0 else (150, 110, 100, 255)
        pts = [(x, SY0 + 19 - amp * (0.5 + 0.5 * math.sin(
            (x - SX0) * 0.4 + ph))) for x in range(SX0 + 2, SX1 - 1)]
        d.line(pts, fill=col, width=1)
    for gx in range(SX0 + 3, SX1 - 2, 4):  # faint organic cell dots
        d.point((gx, SY1 - 3), fill=(196, 150, 140, 255))


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in HEW:
        draw_printout(d, rng, sid)
    elif sid == "HEW_weather_data":
        draw_hew_weather(d, rng)
    elif sid == "mws_weather_data":
        draw_mws_weather(d, rng)
    elif sid == "mws_weather_data_incomplete":
        draw_mws_incomplete(d, rng)
    elif sid == "mws_portal_storm_weather_data":
        draw_mws_portal_storm(d, rng)
    elif sid == "mws_monster_corpse_weather_data":
        draw_mws_corpse(d, rng)
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
    # contact-sheet preview: 6 cols x 5 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item48.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item48.png {prev.size}")


if __name__ == "__main__":
    main()
