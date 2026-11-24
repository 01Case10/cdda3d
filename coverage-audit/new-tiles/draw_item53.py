#!/usr/bin/env python3
"""ITEM53: martial-arts manuals -- CDDA Voxel coverage audit.

All 28 items are books from martial.json: one training manual per
fighting style.  Shared base form: a closed book seen from the
front -- spine strip down the left, page block showing at the
right and bottom edges, a title band across the top carrying the
style's short name as dark glyph blocks, and a framed emblem panel
in the centre of the cover.  Variants differ by cover colour
scheme and emblem motif, taken from each book's in-game title:

  aikido (The Spirit of Aikido): indigo, enso circle
  barbaran (Las Nueve Reglas de la Montante): oxblood, greatsword
  bojutsu (The Matsumoto System): walnut, diagonal bo staff
  boxing (Practical Pugilism): red, boxing glove
  capoeira (Capoeira 100): green/gold, berimbau bow
  crane (The Red Crane): white, red crane
  dragon (The Jade Dragon): jade, coiling dragon
  eskrima (Practical Eskrima): rattan tan, crossed sticks
  fencing (The Modern Swordsman): silver-white, foil
  judo (Kodokan Judo): white, gi lapels over a red disc
  karate (Shotokan Handbook): white, black belt band + fist
  kickboxing (Ultimate Guide): orange, kicking leg arc
  krav maga (Complete Krav Maga): olive drab, shield + fist
  leopard (The Deaf Leopard): tawny, spotted paw
  medievalpole (Fior Di Battaglia): parchment blue, halberd + flower
  muay thai (Ultimate Muay Thai): royal blue, mongkol + knee arc
  ninjutsu (Essence of Ninjutsu): black, shuriken
  niten (The Book of Five Rings): navy/gold, two crossed swords
  pankration (The Modern Pankratiast): aegean blue, laurel wreath
  silat (The Indonesian Warrior): deep teal, wavy kris
  snake (The Black Snake): black, green coiled snake
  sojutsu (The Way of the Spear): vermilion, spear
  swordsmanship (Historic European Swordfighting): slate, longsword
  taekwondo (Official Training Manual): white, red/blue disc + kick
  tai chi (Becoming One with the Tao): charcoal, yin-yang
  tiger (The White Tiger): white, tiger-striped paw
  wingchun (Wing Chun Compendium): dark red, wooden dummy
  zui quan (Zui Quan and You): wine, tilted drunken cup

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "manual_aikido",
    "manual_barbaran",
    "manual_bojutsu",
    "manual_boxing",
    "manual_capoeira",
    "manual_crane",
    "manual_dragon",
    "manual_eskrima",
    "manual_fencing",
    "manual_judo",
    "manual_karate",
    "manual_kickboxing",
    "manual_krav_maga",
    "manual_leopard",
    "manual_medievalpole",
    "manual_muay_thai",
    "manual_ninjutsu",
    "manual_niten",
    "manual_pankration",
    "manual_silat",
    "manual_snake",
    "manual_sojutsu",
    "manual_swordsmanship",
    "manual_taekwondo",
    "manual_tai_chi",
    "manual_tiger",
    "manual_wingchun",
    "manual_zui_quan",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 38, 36, 255)
PAGES = (238, 232, 214, 255)
PAGES_DARK = (198, 190, 168, 255)
WHITE = (244, 242, 236, 255)
BLACK = (30, 28, 30, 255)
GOLD = (216, 178, 80, 255)
SILVER = (188, 192, 198, 255)
RED = (188, 44, 38, 255)

# sid -> (cover, spine/accent, emblem key, title code)
STYLES = {
    "manual_aikido": ((52, 58, 110, 255), (34, 38, 78, 255), "enso",
                      "AIKIDO"),
    "manual_barbaran": ((110, 44, 36, 255), (76, 28, 24, 255), "greatsword",
                        "BARBARAN"),
    "manual_bojutsu": ((122, 88, 54, 255), (84, 58, 32, 255), "staff",
                       "BOJUTSU"),
    "manual_boxing": ((168, 40, 36, 255), (116, 26, 24, 255), "glove",
                      "BOXING"),
    "manual_capoeira": ((58, 132, 66, 255), (38, 92, 46, 255), "berimbau",
                        "CAPOEIRA"),
    "manual_crane": ((238, 234, 224, 255), (196, 190, 176, 255), "crane",
                     "CRANE"),
    "manual_dragon": ((36, 118, 92, 255), (22, 80, 62, 255), "dragon",
                      "DRAGON"),
    "manual_eskrima": ((176, 132, 74, 255), (128, 92, 48, 255), "sticks",
                       "ESKRIMA"),
    "manual_fencing": ((214, 218, 224, 255), (150, 156, 164, 255), "foil",
                       "FENCING"),
    "manual_judo": ((240, 238, 232, 255), (198, 194, 186, 255), "gi",
                    "JUDO"),
    "manual_karate": ((232, 228, 218, 255), (186, 180, 166, 255), "belt",
                      "KARATE"),
    "manual_kickboxing": ((224, 116, 36, 255), (160, 78, 22, 255), "kick",
                          "KICKBOX"),
    "manual_krav_maga": ((104, 108, 66, 255), (70, 74, 42, 255), "shield",
                         "KRAV MAGA"),
    "manual_leopard": ((206, 164, 92, 255), (156, 118, 62, 255), "paw_spots",
                       "LEOPARD"),
    "manual_medievalpole": ((84, 96, 128, 255), (56, 66, 92, 255),
                            "halberd", "FIOR"),
    "manual_muay_thai": ((44, 74, 148, 255), (28, 48, 102, 255), "mongkol",
                         "MUAY THAI"),
    "manual_ninjutsu": ((36, 34, 40, 255), (18, 17, 22, 255), "shuriken",
                        "NINJUTSU"),
    "manual_niten": ((38, 46, 84, 255), (24, 30, 58, 255), "two_swords",
                     "NITEN"),
    "manual_pankration": ((62, 110, 164, 255), (40, 76, 118, 255), "laurel",
                          "PANKRATION"),
    "manual_silat": ((32, 104, 102, 255), (20, 70, 70, 255), "kris",
                     "SILAT"),
    "manual_snake": ((34, 34, 36, 255), (16, 16, 18, 255), "snake",
                     "SNAKE"),
    "manual_sojutsu": ((178, 62, 40, 255), (126, 40, 26, 255), "spear",
                       "SOJUTSU"),
    "manual_swordsmanship": ((96, 102, 112, 255), (62, 68, 78, 255),
                             "longsword", "SWORDS"),
    "manual_taekwondo": ((238, 236, 230, 255), (192, 190, 182, 255),
                         "taeguk", "TAEKWONDO"),
    "manual_tai_chi": ((58, 58, 62, 255), (36, 36, 40, 255), "yinyang",
                       "TAI CHI"),
    "manual_tiger": ((240, 236, 226, 255), (198, 192, 178, 255),
                     "paw_stripes", "TIGER"),
    "manual_wingchun": ((128, 38, 34, 255), (88, 24, 22, 255), "dummy",
                        "WING CHUN"),
    "manual_zui_quan": ((116, 40, 66, 255), (80, 26, 44, 255), "cup",
                        "ZUI QUAN"),
}

# book geometry
BX0, BX1, BY0, BY1 = 5, 26, 3, 28
# emblem panel
EX0, EX1, EY0, EY1 = 11, 24, 11, 24
ECX, ECY = 17, 17


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def title_band(d, code, accent):
    """Title band with the style name as light glyph blocks."""
    d.rectangle([BX0 + 4, BY0 + 1, BX1 - 1, BY0 + 5], fill=accent)
    avail = (BX1 - 2) - (BX0 + 5)
    step = 2 if len(code) * 2 - 1 <= avail else 1
    x = BX0 + 5
    for ch in code:
        o = ord(ch)
        if ch != " ":
            for col in range(2 if step == 2 else 1):
                bits = (o >> (col * 2)) & 0b111
                for row in range(3):
                    if bits & (1 << row):
                        d.point((x + col, BY0 + 2 + row), fill=WHITE)
                if bits == 0:
                    d.point((x + col, BY0 + 3), fill=WHITE)
        x += step + (1 if step == 2 else 0)
        if x > BX1 - 3:
            break


def book(d, rng, cover, accent):
    """Closed book: pages at right/bottom, spine left, cover on top."""
    # page block peeking out right and bottom
    d.rectangle([BX0 + 2, BY0 + 2, BX1 + 1, BY1 + 1], fill=PAGES_DARK)
    d.rectangle([BX0 + 2, BY0 + 1, BX1, BY1], fill=PAGES)
    for y in range(BY0 + 4, BY1, 3):  # page lines on the right edge
        d.point((BX1, y), fill=PAGES_DARK)
    d.line([BX0 + 3, BY1, BX1 - 1, BY1], fill=PAGES_DARK, width=1)
    # cover
    d.rectangle([BX0, BY0, BX1 - 1, BY1 - 2], fill=cover)
    d.rectangle([BX0, BY0, BX1 - 1, BY1 - 2], outline=shade(cover, 0.55),
                width=1)
    # spine strip + hinge line
    d.rectangle([BX0, BY0, BX0 + 3, BY1 - 2], fill=accent)
    d.line([BX0 + 4, BY0, BX0 + 4, BY1 - 2], fill=shade(cover, 0.7), width=1)
    for y in (BY0 + 7, BY1 - 6):  # spine bands
        d.line([BX0, y, BX0 + 3, y], fill=shade(accent, 1.35), width=1)
    # emblem panel
    d.rectangle([EX0, EY0, EX1, EY1], fill=shade(cover, 1.12))
    d.rectangle([EX0, EY0, EX1, EY1], outline=shade(cover, 0.6), width=1)
    # a couple of faint wear scratches, seeded per book
    for _ in range(2):
        x = rng.randint(BX0 + 5, BX1 - 5)
        y = rng.randint(BY1 - 5, BY1 - 3)
        d.line([x, y, x + rng.randint(1, 3), y], fill=shade(cover, 1.25),
               width=1)


# ------------------------------------------------------------------ emblems
# Emblem panel: x 11..24, y 11..24, centre (17, 17).

def emblem(d, rng, key):
    if key == "enso":  # open circle brush ring
        d.arc([ECX - 5, ECY - 5, ECX + 5, ECY + 5], start=40, end=330,
              fill=WHITE, width=2)
        d.arc([ECX - 5, ECY - 5, ECX + 5, ECY + 5], start=330, end=380,
              fill=shade(WHITE, 0.75), width=1)
        d.point((ECX, ECY), fill=RED)
    elif key == "greatsword":  # montante: long blade, wide crossguard
        d.polygon([(ECX, EY0 + 1), (ECX + 1, EY0 + 3), (ECX + 1, EY1 - 5),
                   (ECX - 1, EY1 - 5), (ECX - 1, EY0 + 3)], fill=SILVER)
        d.line([ECX - 4, EY1 - 5, ECX + 4, EY1 - 5], fill=GOLD, width=2)
        d.line([ECX, EY1 - 4, ECX, EY1 - 2], fill=(96, 66, 38, 255), width=2)
        d.point((ECX, EY1 - 1), fill=GOLD)
    elif key == "staff":  # bo staff diagonal with grip wraps
        d.line([EX0 + 2, EY1 - 2, EX1 - 2, EY0 + 2],
               fill=(168, 124, 78, 255), width=3)
        d.line([EX0 + 2, EY1 - 2, EX1 - 2, EY0 + 2],
               fill=(206, 164, 110, 255), width=1)
        for t in (0.38, 0.5, 0.62):  # grip wraps across the staff
            x = EX0 + 2 + (EX1 - EX0 - 4) * t
            y = EY1 - 2 - (EY1 - EY0 - 4) * t
            d.line([x - 1, y + 1, x + 1, y - 1], fill=(70, 50, 30, 255),
                   width=1)
    elif key == "glove":  # boxing glove with cuff and laces
        d.ellipse([ECX - 4, EY0 + 2, ECX + 4, EY0 + 10], fill=RED)
        d.ellipse([ECX - 4, EY0 + 2, ECX + 4, EY0 + 10],
                  outline=shade(RED, 0.65), width=1)
        d.ellipse([ECX + 2, EY0 + 5, ECX + 6, EY0 + 9],
                  fill=shade(RED, 0.85))  # thumb
        d.point((ECX - 2, EY0 + 4), fill=shade(RED, 1.35))
        d.rectangle([ECX - 3, EY0 + 10, ECX + 3, EY0 + 12], fill=WHITE)
        for lx in (ECX - 1, ECX + 1):  # laces
            d.line([lx, EY0 + 10, lx, EY0 + 12], fill=INK, width=1)
    elif key == "berimbau":  # bow arc, string, gourd circle
        d.arc([EX0 + 1, EY0 - 2, EX1 + 4, EY1 + 2], start=90, end=215,
              fill=(96, 66, 38, 255), width=2)
        d.line([EX0 + 3, EY0 + 2, EX0 + 3, EY1 - 3], fill=WHITE, width=1)
        d.ellipse([EX0 + 1, EY1 - 6, EX0 + 6, EY1 - 1],
                  outline=GOLD, width=1)
        d.line([EX1 - 4, EY1 - 4, EX1 - 1, EY1 - 7], fill=INK, width=2)
    elif key == "crane":  # red crane: S neck, wing, one standing leg
        d.ellipse([ECX - 4, ECY - 1, ECX + 3, ECY + 3],
                  fill=RED)  # body
        d.arc([ECX + 1, ECY - 6, ECX + 6, ECY + 1], start=90, end=270,
              fill=RED, width=2)  # neck
        d.point((ECX + 4, ECY - 6), fill=RED)  # head
        d.line([ECX + 4, ECY - 6, ECX + 6, ECY - 5], fill=GOLD, width=1)
        d.polygon([(ECX - 4, ECY), (ECX - 7, ECY - 3), (ECX - 2, ECY + 1)],
                  fill=shade(RED, 0.7))  # wing/tail
        d.line([ECX, ECY + 3, ECX, EY1 - 1], fill=INK, width=1)  # leg
        d.line([ECX + 2, ECY + 3, ECX + 3, ECY + 5], fill=INK, width=1)
        d.line([ECX + 3, ECY + 5, ECX + 3, EY1 - 3], fill=INK, width=1)
    elif key == "dragon":  # S-curved dragon with horned head
        pts = [(EX0 + 3 + t * 1.05, ECY + 4 * math.sin(t * 0.85))
               for t in range(0, 11)]
        d.line(pts, fill=(120, 220, 170, 255), width=3)
        d.line(pts, fill=(52, 150, 110, 255), width=1)
        hx, hy = pts[-1]
        d.rectangle([hx - 1, hy - 3, hx + 2, hy], fill=(120, 220, 170, 255))
        d.line([hx + 2, hy - 3, hx + 4, hy - 4], fill=GOLD, width=1)
        d.point((hx + 1, hy - 2), fill=RED)  # eye
        d.line([hx + 2, hy - 1, hx + 4, hy - 1], fill=RED, width=1)
        for t in (2, 5, 8):  # belly ticks
            x, y = pts[t]
            d.point((int(x), int(y) + 2), fill=GOLD)
    elif key == "sticks":  # crossed eskrima sticks
        d.line([EX0 + 2, EY0 + 2, EX1 - 2, EY1 - 2],
               fill=(150, 104, 58, 255), width=3)
        d.line([EX0 + 2, EY1 - 2, EX1 - 2, EY0 + 2],
               fill=(178, 128, 72, 255), width=3)
        d.line([EX0 + 2, EY0 + 2, EX1 - 2, EY1 - 2],
               fill=(206, 164, 110, 255), width=1)
        d.line([EX0 + 2, EY1 - 2, EX1 - 2, EY0 + 2],
               fill=(120, 82, 44, 255), width=1)
        for x, y in ((EX0 + 3, EY0 + 3), (EX1 - 3, EY0 + 3),
                     (EX0 + 3, EY1 - 3), (EX1 - 3, EY1 - 3)):
            d.point((x, y), fill=(70, 50, 30, 255))  # grip ends
    elif key == "foil":  # fencing foil diagonal, bell guard, mask dot grid
        d.line([EX0 + 2, EY1 - 2, EX1 - 4, EY0 + 4], fill=SILVER, width=1)
        d.line([EX1 - 4, EY0 + 4, EX1 - 2, EY0 + 2], fill=SILVER, width=1)
        d.arc([EX0 + 1, EY1 - 8, EX0 + 7, EY1 - 2], start=270, end=90,
              fill=GOLD, width=2)  # bell guard
        d.line([EX0 + 2, EY1 - 3, EX0 + 1, EY1 - 1],
               fill=(96, 66, 38, 255), width=2)  # grip
        for gx in range(EX1 - 5, EX1 - 1, 2):  # mask mesh hint, top right
            for gy in range(EY0 + 1, EY0 + 4, 1):
                d.point((gx, gy + (gx % 2)), fill=(90, 96, 104, 255))
    elif key == "gi":  # judogi lapels over the Kodokan red disc
        d.ellipse([ECX - 4, ECY - 4, ECX + 4, ECY + 4], fill=RED)
        d.polygon([(EX0 + 2, EY0 + 1), (ECX, ECY + 1), (EX0 + 2, EY1 - 2)],
                  fill=WHITE)  # left lapel
        d.polygon([(EX1 - 2, EY0 + 1), (ECX, ECY + 1), (EX1 - 2, EY1 - 2)],
                  fill=shade(WHITE, 0.88))  # right lapel
        d.line([EX0 + 2, EY1 - 3, EX1 - 2, EY1 - 3], fill=BLACK, width=2)
    elif key == "belt":  # fist over a black belt band
        d.rectangle([ECX - 3, EY0 + 2, ECX + 3, EY0 + 7],
                    fill=(222, 186, 150, 255))  # fist
        d.rectangle([ECX - 3, EY0 + 2, ECX + 3, EY0 + 7],
                    outline=(150, 116, 84, 255), width=1)
        for lx in (ECX - 1, ECX + 1):  # knuckle seams
            d.line([lx, EY0 + 2, lx, EY0 + 4], fill=(150, 116, 84, 255),
                   width=1)
        d.rectangle([EX0 + 1, EY1 - 5, EX1 - 1, EY1 - 3], fill=BLACK)
        d.line([ECX + 2, EY1 - 3, ECX + 4, EY1 - 1], fill=BLACK, width=2)
        d.line([ECX + 3, EY1 - 3, ECX + 5, EY1 - 1], fill=BLACK, width=1)
    elif key == "kick":  # kicking leg: thigh, shin, foot on an arc
        for t in range(0, 9):  # motion arc
            a = -0.4 - t * 0.16
            d.point((ECX + 1 + 6 * math.cos(a), ECY + 6 * math.sin(a)),
                    fill=shade(WHITE, 0.8))
        d.line([EX0 + 3, EY1 - 3, ECX - 1, ECY + 1],
               fill=(52, 52, 58, 255), width=3)  # thigh (trouser leg)
        d.line([ECX - 1, ECY + 1, EX1 - 3, EY0 + 3],
               fill=(222, 186, 150, 255), width=3)  # bare shin
        d.polygon([(EX1 - 4, EY0 + 2), (EX1 - 1, EY0 + 2), (EX1 - 2, EY0 + 5),
                   (EX1 - 4, EY0 + 5)], fill=(222, 186, 150, 255))  # foot
    elif key == "shield":  # shield outline with a fist
        d.polygon([(ECX, EY0 + 1), (EX1 - 2, EY0 + 3), (EX1 - 2, ECY),
                   (ECX, EY1 - 1), (EX0 + 2, ECY), (EX0 + 2, EY0 + 3)],
                  outline=GOLD, width=1)
        d.rectangle([ECX - 2, ECY - 2, ECX + 2, ECY + 2],
                    fill=(222, 186, 150, 255))
        for lx in (ECX - 1, ECX + 1):
            d.line([lx, ECY - 2, lx, ECY], fill=(150, 116, 84, 255),
                   width=1)
        d.line([ECX - 2, ECY + 3, ECX + 2, ECY + 3], fill=GOLD, width=1)
    elif key in ("paw_spots", "paw_stripes"):  # paw print, spotted/striped
        pad = (120, 84, 44, 255) if key == "paw_spots" else (60, 56, 54, 255)
        d.ellipse([ECX - 3, ECY + 1, ECX + 3, ECY + 6], fill=pad)  # main pad
        for dx, dy in ((-5, -2), (-2, -4), (2, -4), (5, -2)):  # toes
            d.ellipse([ECX + dx - 1, ECY + dy - 1, ECX + dx + 1,
                       ECY + dy + 2], fill=pad)
            d.line([ECX + dx, ECY + dy - 3, ECX + dx, ECY + dy - 1],
                   fill=pad, width=1)  # claw tick
        if key == "paw_spots":  # leopard rosettes around the paw
            for x, y in ((EX0 + 2, EY0 + 3), (EX1 - 3, EY0 + 5),
                         (EX0 + 3, EY1 - 4), (EX1 - 4, EY1 - 3),
                         (EX0 + 2, ECY), (EX1 - 2, ECY - 2)):
                d.arc([x - 1, y - 1, x + 1, y + 1], start=30, end=300,
                      fill=(120, 84, 44, 255), width=1)
        else:  # tiger stripes radiating in
            for x, y, dx in ((EX0 + 1, EY0 + 3, 2), (EX1 - 2, EY0 + 4, -2),
                             (EX0 + 1, EY1 - 4, 2), (EX1 - 2, EY1 - 3, -2),
                             (EX0 + 1, ECY, 3), (EX1 - 2, ECY, -3)):
                d.line([x, y, x + dx, y + (1 if y < ECY else -1)],
                       fill=(40, 38, 36, 255), width=1)
    elif key == "halberd":  # polearm shaft, axe blade, fior flower
        d.line([ECX - 2, EY0 + 1, ECX - 2, EY1 - 1],
               fill=(96, 66, 38, 255), width=2)
        d.polygon([(ECX - 2, EY0 + 1), (ECX + 3, EY0 + 3), (ECX + 3, EY0 + 7),
                   (ECX - 1, EY0 + 5)], fill=SILVER)  # axe blade
        d.point((ECX - 2, EY0), fill=SILVER)  # spike
        fx, fy = EX1 - 4, EY1 - 5  # the "flower of battle"
        for dx, dy in ((0, -2), (2, 0), (0, 2), (-2, 0)):
            d.ellipse([fx + dx - 1, fy + dy - 1, fx + dx + 1, fy + dy + 1],
                      fill=(232, 150, 170, 255))
        d.point((fx, fy), fill=GOLD)
    elif key == "mongkol":  # headband arc with tails + knee strike
        d.arc([ECX - 5, ECY - 6, ECX + 5, ECY + 2], start=180, end=360,
              fill=WHITE, width=2)
        d.line([ECX + 4, ECY - 3, ECX + 6, ECY + 1], fill=WHITE, width=1)
        d.line([ECX + 5, ECY - 3, ECX + 7, ECY], fill=RED, width=1)
        d.line([EX0 + 3, EY1 - 2, ECX, ECY + 3], fill=(222, 186, 150, 255),
               width=3)  # thigh up
        d.line([ECX, ECY + 3, ECX + 2, EY1 - 3], fill=(222, 186, 150, 255),
               width=3)  # knee/shin down
        d.point((ECX + 1, ECY + 3), fill=shade((222, 186, 150, 255), 0.8))
    elif key == "shuriken":  # four-point throwing star
        cx, cy = ECX, ECY
        d.polygon([(cx, cy - 6), (cx + 1, cy - 1), (cx + 6, cy),
                   (cx + 1, cy + 1), (cx, cy + 6), (cx - 1, cy + 1),
                   (cx - 6, cy), (cx - 1, cy - 1)], fill=SILVER)
        d.polygon([(cx, cy - 6), (cx + 1, cy - 1), (cx + 6, cy),
                   (cx + 1, cy + 1), (cx, cy + 6), (cx - 1, cy + 1),
                   (cx - 6, cy), (cx - 1, cy - 1)],
                  outline=(90, 94, 100, 255), width=1)
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=BLACK)
        d.line([EX0 + 2, EY1 - 3, EX0 + 5, EY1 - 3], fill=RED, width=1)
    elif key == "two_swords":  # crossed katana, five-ring dots below
        d.line([EX0 + 3, EY0 + 2, EX1 - 4, EY1 - 4], fill=SILVER, width=1)
        d.line([EX1 - 3, EY0 + 2, EX0 + 4, EY1 - 4], fill=SILVER, width=1)
        d.line([EX1 - 4, EY1 - 4, EX1 - 2, EY1 - 2], fill=INK, width=2)
        d.line([EX0 + 4, EY1 - 4, EX0 + 2, EY1 - 2], fill=INK, width=2)
        d.point((EX1 - 3, EY1 - 3), fill=GOLD)
        d.point((EX0 + 3, EY1 - 3), fill=GOLD)
        for i, dx in enumerate((-4, -2, 0, 2, 4)):  # five rings
            d.ellipse([ECX + dx - 1, EY1 - 2, ECX + dx + 1, EY1],
                      outline=GOLD, width=1)
    elif key == "laurel":  # laurel wreath open at the top
        for t in range(0, 13):
            a = 0.6 + t * 0.38
            for s in (1, -1):
                x = ECX + s * 5 * math.sin(a)
                y = ECY + 5 * math.cos(a) - 1
                d.ellipse([x - 1, y - 1, x + 1, y + 1],
                          fill=(120, 190, 110, 255))
        d.line([ECX - 2, EY1 - 2, ECX + 2, EY1 - 2], fill=GOLD, width=1)
        d.point((ECX, ECY - 1), fill=WHITE)  # grappler's knot
        d.point((ECX - 1, ECY + 1), fill=WHITE)
        d.point((ECX + 1, ECY + 1), fill=WHITE)
    elif key == "kris":  # wavy-bladed dagger
        pts = []
        for t in range(0, 9):
            pts.append((ECX + 2 * math.sin(t * 1.1), EY0 + 2 + t * 1.3))
        d.line(pts, fill=SILVER, width=2)
        d.line([ECX - 3, EY0 + 13, ECX + 3, EY0 + 13], fill=GOLD, width=2)
        d.line([ECX, EY0 + 14, ECX, EY0 + 16], fill=(70, 48, 28, 255),
               width=2)
    elif key == "snake":  # coiled S snake, head raised, forked tongue
        pts = [(ECX + 5 * math.sin(t * 0.62), EY1 - 2 - t * 1.15)
               for t in range(0, 11)]
        d.line(pts, fill=(74, 168, 84, 255), width=3)
        d.line(pts, fill=(44, 110, 54, 255), width=1)
        hx, hy = pts[-1]
        d.ellipse([hx - 2, hy - 2, hx + 2, hy + 1], fill=(74, 168, 84, 255))
        d.point((hx + 1, hy - 1), fill=(250, 220, 80, 255))  # eye
        d.line([hx + 2, hy, hx + 4, hy - 1], fill=RED, width=1)
        d.line([hx + 2, hy, hx + 4, hy + 1], fill=RED, width=1)
    elif key == "spear":  # yari: leaf blade on a long shaft
        d.line([ECX, EY0 + 4, ECX, EY1 - 1], fill=(150, 104, 58, 255),
               width=2)
        d.polygon([(ECX, EY0), (ECX + 2, EY0 + 3), (ECX + 1, EY0 + 5),
                   (ECX - 1, EY0 + 5), (ECX - 2, EY0 + 3)], fill=SILVER)
        d.line([ECX - 2, EY0 + 5, ECX + 2, EY0 + 5], fill=RED, width=1)
        d.line([ECX - 1, EY1 - 4, ECX + 1, EY1 - 4],
               fill=(70, 50, 30, 255), width=1)
    elif key == "longsword":  # vertical longsword over a shield outline
        d.polygon([(ECX - 4, EY0 + 2), (ECX + 4, EY0 + 2), (ECX + 4, ECY + 3),
                   (ECX, EY1 - 1), (ECX - 4, ECY + 3)],
                  outline=shade(SILVER, 0.8), width=1)
        d.polygon([(ECX, EY0 + 3), (ECX + 1, EY0 + 5), (ECX + 1, EY1 - 6),
                   (ECX - 1, EY1 - 6), (ECX - 1, EY0 + 5)], fill=SILVER)
        d.line([ECX - 3, EY1 - 6, ECX + 3, EY1 - 6], fill=GOLD, width=2)
        d.line([ECX, EY1 - 5, ECX, EY1 - 3], fill=(96, 66, 38, 255),
               width=2)
        d.point((ECX, EY1 - 2), fill=GOLD)
    elif key == "taeguk":  # red/blue S disc + high-kick tick
        d.ellipse([ECX - 5, ECY - 5, ECX + 3, ECY + 3], fill=RED)
        d.pieslice([ECX - 5, ECY - 5, ECX + 3, ECY + 3], start=90, end=270,
                   fill=(52, 92, 178, 255))
        d.ellipse([ECX - 5, ECY - 5, ECX + 3, ECY + 3],
                  outline=INK, width=1)
        d.ellipse([ECX - 3, ECY - 5, ECX + 1, ECY - 1],
                  fill=(52, 92, 178, 255))
        d.ellipse([ECX - 3, ECY - 1, ECX + 1, ECY + 3], fill=RED)
        d.line([EX1 - 4, EY1 - 2, EX1 - 2, EY0 + 4],
               fill=(52, 52, 58, 255), width=2)  # kicking leg
        d.line([EX1 - 2, EY0 + 4, EX1 - 1, EY0 + 4],
               fill=(222, 186, 150, 255), width=2)  # foot
    elif key == "yinyang":
        d.ellipse([ECX - 6, ECY - 6, ECX + 6, ECY + 6], fill=WHITE)
        d.pieslice([ECX - 6, ECY - 6, ECX + 6, ECY + 6], start=270, end=90,
                   fill=BLACK)
        d.ellipse([ECX - 6, ECY - 6, ECX + 6, ECY + 6], outline=INK, width=1)
        d.ellipse([ECX - 3, ECY - 6, ECX + 3, ECY], fill=BLACK)
        d.ellipse([ECX - 3, ECY, ECX + 3, ECY + 6], fill=WHITE)
        d.point((ECX, ECY - 3), fill=WHITE)
        d.point((ECX, ECY + 3), fill=BLACK)
    elif key == "dummy":  # wooden dummy: post, three arms, one leg
        wood = (150, 104, 58, 255)
        wood_d = (104, 70, 36, 255)
        d.rectangle([ECX - 2, EY0 + 1, ECX + 2, EY1 - 3], fill=wood)
        d.line([ECX - 2, EY0 + 1, ECX - 2, EY1 - 3], fill=wood_d, width=1)
        d.line([ECX - 2, EY0 + 5, EX0 + 2, EY0 + 3], fill=wood, width=2)
        d.line([ECX + 2, EY0 + 5, EX1 - 2, EY0 + 3], fill=wood, width=2)
        d.line([ECX, EY0 + 9, EX0 + 3, EY0 + 10], fill=wood, width=2)
        d.line([ECX + 1, ECY + 3, ECX + 4, EY1 - 4], fill=wood, width=2)
        d.line([ECX - 4, EY1 - 2, ECX + 4, EY1 - 2], fill=wood_d, width=1)
        d.line([ECX - 3, EY1 - 1, ECX + 3, EY1 - 1], fill=wood_d, width=1)
    elif key == "cup":  # tilted wine cup, spill drops, wavy drunk lines
        for i, yy in enumerate((EY0 + 2, EY0 + 5)):  # wavy motion lines
            pts = [(x, yy + 1.2 * math.sin((x - EX0) * 0.8 + i * 2))
                   for x in range(EX0 + 2, EX1 - 1)]
            d.line(pts, fill=shade(WHITE, 0.75), width=1)
        d.polygon([(ECX - 4, ECY - 1), (ECX + 3, ECY - 4), (ECX + 4, ECY),
                   (ECX - 3, ECY + 3)], fill=GOLD)  # tilted cup bowl
        d.polygon([(ECX - 4, ECY - 1), (ECX + 3, ECY - 4), (ECX + 4, ECY),
                   (ECX - 3, ECY + 3)], outline=shade(GOLD, 0.65), width=1)
        d.line([ECX - 1, ECY + 3, ECX - 2, ECY + 6], fill=GOLD, width=2)
        d.line([ECX - 4, ECY + 6, ECX + 1, ECY + 6], fill=GOLD, width=1)
        for dx, dy in ((5, 0), (6, 3), (4, 5)):  # spilled drops
            d.point((ECX + dx, ECY + dy), fill=(150, 190, 230, 255))
    else:
        raise KeyError(key)


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cover, accent, key, code = STYLES[sid]
    book(d, rng, cover, accent)
    title_band(d, code, accent)
    emblem(d, rng, key)
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
                       "preview_item53.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item53.png {prev.size}")


if __name__ == "__main__":
    main()
