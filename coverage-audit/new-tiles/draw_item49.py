#!/usr/bin/env python3
"""ITEM49: electronics -- CDDA Voxel coverage audit.

All 29 items come from items/tool/electronics.json.  Each device is
drawn as itself -- body, screen/display, buttons and LEDs -- with
shared silhouettes inside each on/off pair:

  UPS_OFF / UPS_ON -- chunky light-gray unified power supply with a
    display window and battery bars; ON lights the display, fills
    the bars and glows the power LED.
  e_handcuffs -- two cuffs linked by a chain with a white electronics
    pod, its siren LED lit (the description says the siren is
    continuous).
  eink_tablet_pc_on -- blue tablet, pale e-ink page of text on screen.
  emf_detector / _on -- yellow handheld with a sensor wand and an
    LED bar; ON lights the screen and the bar (green->yellow->red).
  exodii_portable_tv -- drab 1970s travel TV: bulbous CRT with faint
    static, oversized handle, chunky knobs, camera lens above screen.
  exodii_usb_stick / _full -- scrap-metal-shielded Exodii storage
    with an otherworldly connector; FULL adds a glowing data LED and
    a full data bar, empty leaves both dark.
  exodii_wrist_computer / _on -- strapped wrist module; OFF has dim
    alien glyphs on the analog screen, ON lights them and adds
    two-way-radio waves.
  gas_charger / _on -- compact thermoelectric generator with a
    propane cylinder; ON glows the charge LED and bolt icon.
  geiger_off / geiger_on -- green radiation counter with a trefoil
    badge; ON lights the reading screen and click LED.
  laptop_screen_lit -- open laptop, screen bright with content.
  memory_card_* -- notched cards with gold contacts: unread has a
    blank white label, older is yellowed and scuffed, science
    (XEDRA) carries a glyph-code label and number string.
  methanol_fuel_cell -- UPS-like box plumbed to a methanol tank,
    with a fuel-level window and droplet badge.
  mp3_on -- dark player, lit screen with a music note and progress
    bar, earbud cable curling out.
  noise_emitter / _on -- yellow speaker box with amplifier knobs;
    ON adds crackle zigzags, sound arcs and a red LED.
  noise_emitter_super / _on -- the same idea bigger: stacked
    amplifier boxes and a steel frame; ON is louder on every axis.
  nre_recorder -- red apparatus: glass-domed microphone, thermal
    printer with a printed sheet, a perpetually blank screen save
    its battery indicator, and the single red PRINT DATA button.
  portable_hard_drive -- white external drive brick with a cable.
  resonance_measurement_device -- green laminated XEDRA box, poorly
    backlit screen, and its wound spool of cloth netting (mesh).
  usb_drive_older -- a decade-old thumb drive: bulky, yellowed,
    scuffed, cap off beside it.

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "UPS_OFF",
    "UPS_ON",
    "e_handcuffs",
    "eink_tablet_pc_on",
    "emf_detector",
    "emf_detector_on",
    "exodii_portable_tv",
    "exodii_usb_stick",
    "exodii_usb_stick_full",
    "exodii_wrist_computer",
    "exodii_wrist_computer_on",
    "gas_charger",
    "gas_charger_on",
    "geiger_off",
    "geiger_on",
    "laptop_screen_lit",
    "memory_card_older",
    "memory_card_science",
    "memory_card_unread",
    "methanol_fuel_cell",
    "mp3_on",
    "noise_emitter",
    "noise_emitter_on",
    "noise_emitter_super",
    "noise_emitter_super_on",
    "nre_recorder",
    "portable_hard_drive",
    "resonance_measurement_device",
    "usb_drive_older",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

GRAY = (168, 172, 178, 255)        # UPS / charger aluminum-plastic
GRAY_DARK = (104, 108, 114, 255)
GRAY_LIGHT = (206, 210, 214, 255)
PLASTIC_DARK = (52, 54, 60, 255)   # dark devices
YELLOW = (226, 182, 52, 255)       # EMF / noise emitters
YELLOW_DARK = (168, 132, 34, 255)
GREEN_DEV = (84, 138, 76, 255)     # geiger / resonance device
DRAB = (128, 142, 104, 255)        # Exodii TV drab shell
TEAL = (52, 178, 168, 255)
SCREEN_OFF = (34, 40, 44, 255)
SCREEN_DIM = (86, 96, 96, 255)
SCREEN_LIT = (168, 226, 210, 255)
SCREEN_BLUE = (150, 196, 240, 255)
AMBER = (240, 186, 84, 255)
RED = (206, 52, 44, 255)
LED_ON = (92, 232, 96, 255)
LED_OFF = (52, 66, 54, 255)
INK = (30, 30, 34, 255)
WHITE = (238, 238, 236, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------ pieces

def body(d, x0, y0, x1, y1, fill, outline=INK):
    d.rectangle([x0 + 1, y0 + 1, x1 + 1, y1 + 1], fill=shade(fill, 0.55))
    d.rectangle([x0, y0, x1, y1], fill=fill)
    d.rectangle([x0, y0, x1, y1], outline=outline, width=1)
    d.line([x0 + 1, y0 + 1, x1 - 1, y0 + 1], fill=shade(fill, 1.25), width=1)


def screen(d, x0, y0, x1, y1, lit, col=SCREEN_LIT):
    d.rectangle([x0, y0, x1, y1], fill=col if lit else SCREEN_OFF)
    d.rectangle([x0, y0, x1, y1], outline=INK, width=1)
    if not lit:  # faint glass reflection on a dark screen
        d.line([x0 + 1, y1 - 1, x0 + 4, y0 + 1], fill=(70, 80, 84, 255),
               width=1)


def led(d, x, y, on, col=LED_ON):
    d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=col if on else LED_OFF)
    if on:
        d.point((x, y - 2), fill=shade(col, 1.0))
        d.point((x, y + 2), fill=shade(col, 1.0))
        d.point((x - 2, y), fill=shade(col, 1.0))
        d.point((x + 2, y), fill=shade(col, 1.0))


def knob(d, cx, cy, r=2, col=GRAY_DARK):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=col)
    d.point((cx, cy - r), fill=GRAY_LIGHT)


def grille(d, x0, y0, x1, y1, col):
    for x in range(x0, x1 + 1, 2):
        for y in range(y0, y1 + 1, 2):
            d.point((x, y), fill=col)


def waves(d, cx, cy, col, n=2, r0=5):
    for i in range(n):
        r = r0 + i * 3
        d.arc([cx - r, cy - r, cx + r, cy + r], start=-50, end=50,
              fill=col, width=1)
        d.arc([cx - r, cy - r, cx + r, cy + r], start=130, end=230,
              fill=col, width=1)


def crackles(d, rng, col=AMBER):
    for _ in range(4):
        x, y = rng.randint(2, 26), rng.randint(2, 10)
        d.line([x, y, x + 2, y + 2, x + 1, y + 4], fill=col, width=1)


def glyphs(d, rng, x0, y0, x1, y1, col):
    """Alien glyph marks: short bars/dots in rows."""
    for y in range(y0, y1 + 1, 3):
        x = x0
        while x < x1:
            w = rng.randint(1, 3)
            d.line([x, y, min(x + w, x1), y], fill=col, width=1)
            if rng.random() < 0.4:
                d.point((x, y + 1), fill=col)
            x += w + 2


def trefoil(d, cx, cy, col=INK):
    for dx, dy in ((0, -2), (-2, 1), (2, 1)):
        d.ellipse([cx + dx - 1, cy + dy - 1, cx + dx + 1, cy + dy + 1],
                  fill=col)
    d.point((cx, cy), fill=col)


# ------------------------------------------------------------------ devices

def draw_ups(d, rng, on):
    body(d, 5, 7, 27, 26, GRAY)
    d.rectangle([5, 7, 27, 10], fill=GRAY_DARK)          # top vent band
    for x in range(8, 26, 3):
        d.point((x, 8), fill=GRAY)
    screen(d, 8, 13, 19, 19, on, SCREEN_LIT)
    if on:  # lightning bolt on the lit display
        d.line([13, 14, 11, 17, 13, 17, 12, 19], fill=(30, 110, 84, 255),
               width=1)
    # battery bars: filled when on, hollow when off
    for i in range(3):
        x = 8 + i * 4
        d.rectangle([x, 22, x + 2, 24], outline=GRAY_DARK, width=1)
        if on:
            d.rectangle([x, 22, x + 2, 24], fill=LED_ON)
    knob(d, 24, 16, 2)
    led(d, 24, 22, on)
    d.rectangle([21, 12, 26, 13], fill=GRAY_DARK)        # outlet hint


def draw_handcuffs(d, rng):
    steel = (176, 180, 186, 255)
    for cx in (9, 23):  # two cuff rings, open at the top
        d.ellipse([cx - 6, 12, cx + 6, 27], outline=steel, width=3)
        d.ellipse([cx - 6, 12, cx + 6, 27], outline=INK, width=1)
        d.rectangle([cx - 2, 10, cx + 2, 14], fill=WHITE)  # hinge pod
    for i in range(3):  # chain links between the cuffs
        x = 13 + i * 3
        d.ellipse([x, 6, x + 3, 9], outline=steel, width=1)
    # electronics pod on the right cuff: siren LED lit, speaker dots
    d.rectangle([20, 4, 28, 11], fill=WHITE)
    d.rectangle([20, 4, 28, 11], outline=INK, width=1)
    led(d, 23, 7, True, RED)
    grille(d, 26, 6, 27, 9, GRAY_DARK)
    d.arc([18, 0, 30, 8], start=200, end=340, fill=RED, width=1)


def draw_eink(d, rng):
    body(d, 7, 3, 25, 29, (72, 110, 178, 255))
    # e-ink page: pale even when on, with a header and text lines
    d.rectangle([9, 5, 23, 26], fill=(226, 228, 220, 255))
    d.rectangle([9, 5, 23, 26], outline=INK, width=1)
    d.rectangle([9, 5, 23, 8], fill=(190, 196, 188, 255))
    d.line([11, 6, 17, 6], fill=INK, width=1)
    for y in range(11, 25, 2):
        w = rng.randint(8, 12)
        d.line([11, y, 11 + w, y], fill=(120, 122, 118, 255), width=1)
    led(d, 16, 28, True, AMBER)


def draw_emf(d, rng, on):
    d.rectangle([14, 1, 17, 7], fill=GRAY_DARK)          # sensor wand
    d.ellipse([13, 0, 18, 3], fill=GRAY_LIGHT)
    body(d, 8, 7, 24, 29, YELLOW)
    screen(d, 11, 10, 21, 16, on, (170, 240, 170, 255))
    if on:  # needle gauge swept up
        d.line([13, 15, 19, 11], fill=(30, 110, 60, 255), width=1)
    else:
        d.line([13, 15, 14, 12], fill=(120, 128, 120, 255), width=1)
    cols = [LED_ON, (240, 220, 80, 255), RED]            # LED proximity bar
    for i, col in enumerate(cols):
        led(d, 12 + i * 4, 20, on, col)
    knob(d, 12, 25, 2, YELLOW_DARK)
    knob(d, 20, 25, 2, YELLOW_DARK)
    if on:
        waves(d, 16, 4, YELLOW_DARK, n=1, r0=4)


def draw_exodii_tv(d, rng):
    d.arc([9, 0, 23, 6], start=180, end=360, fill=GRAY_DARK, width=2)
    d.line([9, 3, 23, 3], fill=GRAY_DARK, width=2)       # carrying handle
    body(d, 3, 5, 29, 27, DRAB)
    d.ellipse([7, 9, 21, 23], fill=(150, 168, 148, 255))  # bulbous CRT
    d.ellipse([7, 9, 21, 23], outline=INK, width=1)
    for _ in range(26):                                   # static snow
        x, y = rng.randint(9, 19), rng.randint(11, 21)
        if (x - 14) ** 2 + (y - 16) ** 2 < 30:
            d.point((x, y), fill=(196, 210, 190, 255))
    d.ellipse([10, 12, 17, 19], outline=(96, 116, 92, 255), width=1)
    d.ellipse([13, 6, 16, 9], fill=PLASTIC_DARK)          # camera lens
    d.point((14, 7), fill=SCREEN_BLUE)
    knob(d, 25, 12, 2)                                    # chunky tuners
    knob(d, 25, 18, 2)
    grille(d, 24, 22, 27, 25, shade(DRAB, 0.6))
    d.rectangle([5, 25, 12, 26], fill=GRAY_DARK)


def draw_exodii_usb(d, rng, full):
    # scrap-metal shielded bundle: riveted plates over circuit green
    d.rectangle([9, 8, 22, 24], fill=(70, 110, 66, 255))
    d.rectangle([8, 7, 23, 15], fill=(150, 154, 158, 255))
    d.rectangle([8, 17, 23, 25], fill=(128, 132, 136, 255))
    d.rectangle([8, 7, 23, 25], outline=INK, width=1)
    for x, y in ((9, 8), (22, 8), (9, 24), (22, 24)):
        d.point((x, y), fill=GRAY_DARK)                   # rivets
    for x in range(11, 21, 3):                            # exposed traces
        d.line([x, 16, x, 20], fill=(196, 178, 90, 255), width=1)
    # otherworldly connector: three unequal prongs, top
    for i, h in enumerate((4, 6, 3)):
        x = 12 + i * 4
        d.rectangle([x, 7 - h, x + 1, 7], fill=(196, 178, 90, 255))
    # data state: LED + fill bar
    led(d, 12, 11, full, TEAL)
    d.rectangle([15, 10, 21, 12], outline=INK, width=1)
    if full:
        d.rectangle([15, 10, 21, 12], fill=TEAL)
        glyphs(d, rng, 10, 20, 20, 23, (220, 240, 220, 255))


def draw_wrist(d, rng, on):
    d.rectangle([11, 0, 21, 6], fill=PLASTIC_DARK)        # strap, top
    d.rectangle([11, 26, 21, 31], fill=PLASTIC_DARK)      # strap, bottom
    for y in (2, 28):
        d.line([13, y, 19, y], fill=GRAY_DARK, width=1)
    body(d, 6, 6, 26, 26, (84, 88, 94, 255))
    d.rectangle([8, 8, 24, 22], fill=SCREEN_LIT if on else SCREEN_DIM)
    d.rectangle([8, 8, 24, 22], outline=INK, width=1)
    glyphs(d, rng, 10, 11, 22, 19,
           (240, 220, 150, 255) if on else (120, 128, 122, 255))
    led(d, 10, 24, on, RED)
    knob(d, 23, 24, 1, GRAY_DARK)
    if on:
        waves(d, 27, 10, TEAL, n=2, r0=3)


def draw_gas_charger(d, rng, on):
    d.rectangle([22, 10, 28, 24], fill=(178, 84, 60, 255))  # propane tank
    d.rectangle([22, 10, 28, 24], outline=INK, width=1)
    d.rectangle([24, 7, 26, 10], fill=GRAY_DARK)
    d.line([24, 9, 19, 12], fill=GRAY_DARK, width=1)         # feed hose
    body(d, 3, 9, 20, 26, GRAY)
    for x in range(5, 19, 2):                                # cooling fins
        d.line([x, 12, x, 16], fill=GRAY_DARK, width=1)
    screen(d, 5, 19, 12, 23, on, SCREEN_LIT)
    if on:
        d.line([8, 20, 7, 22, 9, 22], fill=(30, 110, 84, 255), width=1)
    led(d, 16, 21, on)
    knob(d, 16, 13, 2)


def draw_geiger(d, rng, on):
    body(d, 7, 6, 25, 29, GREEN_DEV)
    d.rectangle([13, 1, 19, 6], fill=GRAY_DARK)           # probe head
    d.line([16, 6, 16, 8], fill=GRAY_DARK, width=1)
    screen(d, 10, 9, 22, 16, on, (190, 240, 170, 255))
    if on:  # reading bars
        for i, h in enumerate((2, 4, 3, 5)):
            d.line([12 + i * 3, 15, 12 + i * 3, 15 - h],
                   fill=(30, 110, 60, 255), width=1)
    d.rectangle([10, 18, 16, 23], fill=(240, 200, 60, 255))  # warning badge
    d.rectangle([10, 18, 16, 23], outline=INK, width=1)
    trefoil(d, 13, 20)
    grille(d, 19, 19, 22, 23, shade(GREEN_DEV, 0.6))
    led(d, 21, 26, on, RED)
    knob(d, 11, 26, 2, shade(GREEN_DEV, 0.7))
    if on:
        for x, y in ((4, 4), (27, 8), (5, 14)):
            d.point((x, y), fill=RED)                     # click sparks


def draw_laptop(d, rng):
    # upright lit screen
    d.rectangle([6, 3, 26, 20], fill=PLASTIC_DARK)
    d.rectangle([7, 4, 25, 19], fill=SCREEN_BLUE)
    d.rectangle([9, 6, 17, 12], fill=(226, 240, 252, 255))  # window
    for y in range(8, 12, 2):
        d.line([10, y, 15, y], fill=(110, 140, 180, 255), width=1)
    d.rectangle([19, 6, 23, 9], fill=(190, 220, 250, 255))
    d.line([9, 15, 23, 15], fill=(110, 140, 180, 255), width=1)
    d.line([9, 17, 19, 17], fill=(110, 140, 180, 255), width=1)
    # keyboard deck in perspective
    d.polygon([(4, 21), (28, 21), (30, 28), (2, 28)], fill=(70, 72, 78, 255))
    d.polygon([(4, 21), (28, 21), (30, 28), (2, 28)], outline=INK)
    for x in range(7, 26, 3):
        d.point((x, 24), fill=GRAY_DARK)
    d.rectangle([13, 25, 19, 27], outline=GRAY_DARK, width=1)  # touchpad


def draw_memory_card(d, rng, kind):
    x0, y0, x1, y1 = 9, 5, 23, 27
    base = (214, 210, 196, 255) if kind == "older" else WHITE
    d.polygon([(x0, y0 + 3), (x0 + 3, y0), (x1, y0), (x1, y1), (x0, y1)],
              fill=base)
    d.polygon([(x0, y0 + 3), (x0 + 3, y0), (x1, y0), (x1, y1), (x0, y1)],
              outline=INK)
    # gold contacts along the bottom edge
    for x in range(x0 + 2, x1 - 1, 3):
        d.rectangle([x, y1 - 3, x + 1, y1 - 1], fill=(202, 168, 82, 255))
    if kind == "unread":  # blank label
        d.rectangle([x0 + 2, y0 + 3, x1 - 2, y0 + 12], fill=(232, 232, 228, 255))
        d.rectangle([x0 + 2, y0 + 3, x1 - 2, y0 + 12], outline=GRAY_DARK,
                    width=1)
    elif kind == "older":  # yellowed, scuffed label
        d.rectangle([x0 + 2, y0 + 3, x1 - 2, y0 + 12],
                    fill=(226, 204, 148, 255))
        d.rectangle([x0 + 2, y0 + 3, x1 - 2, y0 + 12], outline=GRAY_DARK,
                    width=1)
        d.line([x0 + 4, y0 + 6, x0 + 9, y0 + 6], fill=(150, 128, 84, 255),
               width=1)
        for _ in range(5):
            d.point((rng.randint(x0 + 2, x1 - 2), rng.randint(y0 + 2, y1 - 5)),
                    fill=(170, 164, 140, 255))
    else:  # science / XEDRA: glyph-code label + number string
        d.rectangle([x0 + 2, y0 + 3, x1 - 2, y0 + 12],
                    fill=(52, 54, 60, 255))
        glyphs(d, rng, x0 + 3, y0 + 5, x1 - 3, y0 + 7, (240, 220, 120, 255))
        for i in range(6):
            d.point((x0 + 3 + i * 2, y0 + 10), fill=WHITE)
        d.line([x0 + 2, y0 + 15, x1 - 2, y0 + 15], fill=RED, width=1)


def draw_fuel_cell(d, rng):
    d.rectangle([21, 8, 28, 25], fill=(196, 204, 208, 255))  # methanol tank
    d.rectangle([21, 8, 28, 25], outline=INK, width=1)
    d.rectangle([21, 14, 28, 25], fill=(120, 190, 200, 255))  # fuel level
    d.rectangle([23, 5, 26, 8], fill=GRAY_DARK)
    d.line([21, 12, 18, 12], fill=GRAY_DARK, width=2)          # feed tube
    body(d, 3, 8, 19, 26, GRAY)
    screen(d, 5, 11, 12, 16, True, SCREEN_LIT)
    d.line([8, 12, 7, 14, 9, 14], fill=(30, 110, 84, 255), width=1)
    # droplet badge
    d.polygon([(15, 19), (17, 22), (15, 24), (13, 22)], fill=TEAL)
    for i in range(3):
        x = 5 + i * 4
        d.rectangle([x, 21, x + 2, 23], fill=LED_ON)
    led(d, 16, 12, True)


def draw_mp3(d, rng):
    body(d, 10, 4, 23, 26, PLASTIC_DARK)
    screen(d, 12, 6, 21, 14, True, SCREEN_BLUE)
    # music note on the lit screen
    d.ellipse([14, 11, 16, 13], fill=INK)
    d.line([16, 12, 16, 7], fill=INK, width=1)
    d.line([16, 7, 19, 8], fill=INK, width=1)
    d.ellipse([18, 12, 20, 14], fill=INK)
    d.rectangle([12, 17, 21, 18], fill=GRAY_DARK)          # progress bar
    d.rectangle([12, 17, 17, 18], fill=SCREEN_BLUE)
    d.ellipse([14, 20, 19, 25], outline=GRAY_LIGHT, width=1)  # click wheel
    d.point((16, 22), fill=GRAY_LIGHT)
    # earbud cable curling out to two buds
    d.arc([2, 16, 12, 29], start=90, end=300, fill=GRAY_LIGHT, width=1)
    d.ellipse([3, 14, 6, 17], fill=WHITE)
    d.ellipse([6, 24, 9, 27], fill=WHITE)


def draw_noise(d, rng, on, super_):
    if super_:  # stacked amplifier boxes in a steel frame
        d.rectangle([4, 4, 28, 28], outline=GRAY_DARK, width=2)
        body(d, 6, 6, 26, 17, YELLOW)
        body(d, 6, 19, 26, 27, shade(YELLOW, 0.9))
        cy = 11
    else:
        body(d, 6, 7, 26, 27, YELLOW)
        cy = 15
    # speaker cone
    d.ellipse([8, cy - 5, 18, cy + 5], fill=GRAY_DARK)
    d.ellipse([10, cy - 3, 16, cy + 3], fill=PLASTIC_DARK)
    d.point((13, cy), fill=GRAY_LIGHT)
    knob(d, 22, cy - 3, 2, YELLOW_DARK)
    led(d, 22, cy + 3, on, RED)
    if super_:
        knob(d, 11, 23, 2, YELLOW_DARK)
        knob(d, 17, 23, 2, YELLOW_DARK)
        led(d, 23, 23, on, RED)
        grille(d, 20, 21, 25, 25, YELLOW_DARK)
    else:
        d.line([8, 23, 17, 23], fill=YELLOW_DARK, width=1)  # amp wire trace
    if on:
        waves(d, 13, cy, RED if super_ else YELLOW_DARK,
              n=3 if super_ else 2, r0=8)
        crackles(d, rng, RED if super_ else AMBER)
        if super_:
            crackles(d, rng, RED)


def draw_nre(d, rng):
    body(d, 4, 8, 28, 28, (178, 60, 52, 255))
    # glass-domed microphone, left
    d.arc([5, 10, 15, 22], start=180, end=360, fill=(200, 226, 232, 255),
          width=2)
    d.rectangle([8, 13, 11, 20], fill=PLASTIC_DARK)
    grille(d, 9, 14, 10, 17, GRAY_LIGHT)
    d.line([5, 22, 15, 22], fill=GRAY_DARK, width=1)
    # thermal printer, right: sheet emerging from the slot
    d.rectangle([18, 16, 27, 21], fill=(226, 224, 216, 255))
    d.rectangle([18, 16, 27, 21], outline=INK, width=1)
    for y in (18, 20):
        d.line([20, y, 25, y], fill=(150, 148, 142, 255), width=1)
    d.rectangle([17, 22, 28, 24], fill=GRAY_DARK)
    # blank screen with only a battery indicator, upper right
    screen(d, 17, 9, 27, 14, False)
    d.rectangle([24, 10, 26, 12], outline=LED_ON, width=1)
    d.rectangle([24, 10, 25, 12], fill=LED_ON)
    # the single red PRINT DATA button, left side
    d.ellipse([5, 24, 9, 27], fill=RED)
    d.ellipse([5, 24, 9, 27], outline=INK, width=1)
    d.point((7, 25), fill=WHITE)


def draw_hard_drive(d, rng):
    body(d, 6, 9, 26, 24, WHITE)
    d.rectangle([6, 9, 26, 13], fill=(196, 200, 204, 255))  # label band
    d.line([8, 11, 16, 11], fill=GRAY_DARK, width=1)
    d.ellipse([11, 16, 21, 22], outline=(196, 200, 204, 255), width=1)
    led(d, 23, 20, True, TEAL)
    # USB cable curling from the top
    d.arc([18, 0, 30, 12], start=90, end=270, fill=GRAY_DARK, width=1)
    d.rectangle([24, 1, 27, 4], fill=GRAY_LIGHT)
    d.rectangle([24, 1, 27, 4], outline=INK, width=1)


def draw_resonance(d, rng):
    body(d, 4, 6, 21, 27, GREEN_DEV)
    # XEDRA glyph letters down the side
    glyphs(d, rng, 6, 8, 9, 24, (240, 240, 220, 255))
    # poorly backlit screen: dim pale glow
    d.rectangle([11, 9, 19, 17], fill=(150, 168, 138, 255))
    d.rectangle([11, 9, 19, 17], outline=INK, width=1)
    d.line([12, 13, 18, 13], fill=(96, 110, 84, 255), width=1)
    d.point((13, 11), fill=(96, 110, 84, 255))
    d.rectangle([11, 20, 19, 24], fill=GRAY_DARK)          # battery bulge
    led(d, 12, 26, True, AMBER)
    # wound spool of cloth netting beside the box
    d.ellipse([22, 10, 30, 24], fill=(206, 200, 178, 255))
    d.ellipse([22, 10, 30, 24], outline=(150, 142, 118, 255), width=1)
    for x in range(23, 30, 2):                              # net mesh wraps
        d.line([x, 11, x, 23], fill=(150, 142, 118, 255), width=1)
    for y in range(12, 24, 2):
        d.line([23, y, 29, y], fill=(150, 142, 118, 255), width=1)
    d.ellipse([24, 15, 28, 19], outline=(150, 142, 118, 255), width=1)


def draw_usb_older(d, rng):
    # decade-old thumb drive: bulky yellowed body, metal connector
    d.rectangle([12, 2, 19, 9], fill=(188, 190, 194, 255))
    d.rectangle([12, 2, 19, 9], outline=INK, width=1)
    d.rectangle([14, 4, 17, 7], fill=SCREEN_OFF)
    body(d, 9, 9, 22, 27, (224, 216, 182, 255))
    d.rectangle([11, 12, 20, 17], fill=(206, 196, 158, 255))  # faded label
    d.line([12, 14, 18, 14], fill=(150, 138, 100, 255), width=1)
    for _ in range(6):                                          # scuffs
        d.point((rng.randint(10, 21), rng.randint(10, 26)),
                fill=(170, 160, 124, 255))
    led(d, 19, 24, False, RED)
    # detached cap beside it
    d.rectangle([24, 16, 29, 26], fill=(214, 204, 168, 255))
    d.rectangle([24, 16, 29, 26], outline=INK, width=1)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid == "UPS_OFF":
        draw_ups(d, rng, False)
    elif sid == "UPS_ON":
        draw_ups(d, rng, True)
    elif sid == "e_handcuffs":
        draw_handcuffs(d, rng)
    elif sid == "eink_tablet_pc_on":
        draw_eink(d, rng)
    elif sid == "emf_detector":
        draw_emf(d, rng, False)
    elif sid == "emf_detector_on":
        draw_emf(d, rng, True)
    elif sid == "exodii_portable_tv":
        draw_exodii_tv(d, rng)
    elif sid == "exodii_usb_stick":
        draw_exodii_usb(d, rng, False)
    elif sid == "exodii_usb_stick_full":
        draw_exodii_usb(d, rng, True)
    elif sid == "exodii_wrist_computer":
        draw_wrist(d, rng, False)
    elif sid == "exodii_wrist_computer_on":
        draw_wrist(d, rng, True)
    elif sid == "gas_charger":
        draw_gas_charger(d, rng, False)
    elif sid == "gas_charger_on":
        draw_gas_charger(d, rng, True)
    elif sid == "geiger_off":
        draw_geiger(d, rng, False)
    elif sid == "geiger_on":
        draw_geiger(d, rng, True)
    elif sid == "laptop_screen_lit":
        draw_laptop(d, rng)
    elif sid == "memory_card_older":
        draw_memory_card(d, rng, "older")
    elif sid == "memory_card_science":
        draw_memory_card(d, rng, "science")
    elif sid == "memory_card_unread":
        draw_memory_card(d, rng, "unread")
    elif sid == "methanol_fuel_cell":
        draw_fuel_cell(d, rng)
    elif sid == "mp3_on":
        draw_mp3(d, rng)
    elif sid == "noise_emitter":
        draw_noise(d, rng, False, False)
    elif sid == "noise_emitter_on":
        draw_noise(d, rng, True, False)
    elif sid == "noise_emitter_super":
        draw_noise(d, rng, False, True)
    elif sid == "noise_emitter_super_on":
        draw_noise(d, rng, True, True)
    elif sid == "nre_recorder":
        draw_nre(d, rng)
    elif sid == "portable_hard_drive":
        draw_hard_drive(d, rng)
    elif sid == "resonance_measurement_device":
        draw_resonance(d, rng)
    elif sid == "usb_drive_older":
        draw_usb_older(d, rng)
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
                       "preview_item49.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item49.png {prev.size}")


if __name__ == "__main__":
    main()
