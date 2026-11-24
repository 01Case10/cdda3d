#!/usr/bin/env python3
"""Batch ITEM12 (Phase 2g.4 scope extension): tool-armor,
first set.

52 ids from data/json/items/tool_armor.json (authoritative
list embedded below; source list:
~/workspace/item-batches/12.txt). Parameterized family
renderers driven by each entry's data:

- Ear plugs (attached industrial / makeshift wax): two plugs
  on a string -- dangling around the neck when off, seated in
  ears when on.
- Powered hearing gear (blue): earmuffs and in-ear plugs;
  the _on variants gain an LED and sound-wave arcs.
- Goggles: IR (red lenses) and NV (green lenses), off/on;
  on = bright lenses, glow halo, lit LED.
- EOD helmet: bulky olive blast helmet with headlamp and
  camera; _on lights the lamp (pale beam) and camera LED.
- Riot helmet: dark helmet with face shield down, or the
  shield flipped up for the raised-visor variant.
- Survivor gas masks (treated leather): plain, Nomex-faced
  firemask, steel-plated, half mask, and the dark armored
  divemask (on = bubbles + valve light); each in regular,
  XS and XL sizes.
- Firefighter respirator: full-face mask with regulator;
  SCBA-on adds the corrugated hose and status light.
- Rebreather: mouth mask with side filters and hose loop;
  on = bubbles; XL drawn larger.
- Singles: binoculars, white combo cover, fedora, fitness
  band with pulse display, wrist flashlight mount, electric
  blanket (on: glowing coils + cord), holo cloak with hex
  sheets, ski mask / ski collar, military AR visor (on =
  cyan HUD), radiation biomonitor with trefoil display.

Items are objects on transparency, native 32x32, flat UltiCa
style, bodies fully opaque (alpha in {0,255}); translucency
and glow faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item12.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "attached_ear_plugs_off",
    "attached_ear_plugs_on",
    "binoculars",
    "combo_cover",
    "electric_blanket_on",
    "fedora",
    "fitness_band",
    "flashlight_mount_wrist",
    "goggles_ir",
    "goggles_ir_on",
    "goggles_nv",
    "goggles_nv_on",
    "helmet_eod",
    "helmet_eod_on",
    "helmet_riot",
    "helmet_riot_raised",
    "holo_cloak",
    "makeshift_ear_plugs_off",
    "makeshift_ear_plugs_on",
    "mask_bunker",
    "mask_bunker_scba_on",
    "mask_fsurvivor",
    "mask_fsurvivorxl",
    "mask_fsurvivorxs",
    "mask_h20survivor",
    "mask_h20survivor_on",
    "mask_h20survivorxl",
    "mask_h20survivorxl_on",
    "mask_h20survivorxs",
    "mask_h20survivorxs_on",
    "mask_hsurvivor",
    "mask_hsurvivorxl",
    "mask_hsurvivorxs",
    "mask_lsurvivor",
    "mask_lsurvivorxl",
    "mask_lsurvivorxs",
    "mask_ski",
    "mask_ski_loose",
    "mask_survivor",
    "mask_survivorxl",
    "mask_survivorxs",
    "military_vis_aug_system",
    "military_vis_aug_system_on",
    "powered_earmuffs",
    "powered_earmuffs_on",
    "powered_earplugs",
    "powered_earplugs_on",
    "rad_monitor",
    "rebreather",
    "rebreather_on",
    "rebreather_xl",
    "rebreather_xl_on",
]

OUTLINE = (28, 26, 30, 255)
GRAY_L = (168, 172, 178, 255)
GRAY_M = (108, 112, 118, 255)
GRAY_D = (58, 60, 66, 255)
STEEL = (150, 156, 164, 255)
LEATHER = (122, 82, 50, 255)
LEATHER_D = (86, 56, 32, 255)
NOMEX = (182, 156, 106, 255)
LENS = (168, 205, 215, 255)
LENS_D = (52, 84, 94, 255)
BLUE = (52, 90, 180, 255)
BLUE_D = (36, 62, 128, 255)
SKIN = (216, 172, 138, 255)


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def rrect(d, box, r, **kw):
    d.rounded_rectangle(box, radius=r, **kw)


# ---------------------------------------------------------------- ear plugs
def earplugs(plug_fill, plug_edge, wax=False):
    def render(rng):
        img, d = canvas()
        # string: U-curve hanging between the plugs
        pts = []
        for t in range(21):
            x = 8 + t * 0.8
            y = 14 + 9 * math.sin(math.pi * t / 20)
            pts.append((x, y))
        d.line(pts, fill=(210, 208, 200, 255) if wax else (90, 90, 96, 255),
               width=1)
        for cx in (8, 24):
            if wax:
                d.ellipse([cx - 3, 9, cx + 3, 16], fill=plug_fill,
                          outline=plug_edge)
                d.ellipse([cx - 2, 11, cx + 1, 14], fill=plug_edge)
            else:
                d.rectangle([cx - 1, 6, cx + 1, 9], fill=plug_edge)
                for i, w in enumerate((3, 2, 3)):
                    y = 9 + i * 2
                    d.ellipse([cx - w, y, cx + w, y + 2], fill=plug_fill,
                              outline=plug_edge)
                d.rectangle([cx - 1, 15, cx + 1, 17], fill=plug_fill)
        return img
    return render


def earplugs_worn(plug_fill, plug_edge, wax=False):
    def render(rng):
        img, d = canvas()
        for cx in (9, 23):
            # ear: C-shaped shell of skin tone
            d.ellipse([cx - 5, 8, cx + 5, 20], fill=SKIN, outline=plug_edge)
            d.ellipse([cx - 2, 11, cx + 2, 17], fill=(190, 140, 104, 255))
            if wax:
                d.ellipse([cx - 2, 11, cx + 3, 16], fill=plug_fill,
                          outline=plug_edge)
            else:
                d.ellipse([cx - 3, 12, cx + 3, 15], fill=plug_fill,
                          outline=plug_edge)
                d.point((cx, 13), fill=(240, 240, 240, 255))
        # string draped below between the ears
        pts = []
        for t in range(17):
            x = 9 + t * 0.875
            y = 21 + 5 * math.sin(math.pi * t / 16)
            pts.append((x, y))
        d.line(pts, fill=(210, 208, 200, 255) if wax else (90, 90, 96, 255),
               width=1)
        return img
    return render


# ------------------------------------------------------------------- singles
def binoculars(rng):
    img, d = canvas()
    # neck strap
    d.arc([2, 2, 30, 22], 200, 340, fill=GRAY_D, width=2)
    for x0 in (6, 19):
        rrect(d, [x0, 9, x0 + 7, 25], 2, fill=GRAY_D, outline=OUTLINE)
        d.ellipse([x0 + 1, 21, x0 + 6, 26], fill=LENS_D, outline=STEEL)
        d.point((x0 + 2, 22), fill=(220, 240, 245, 255))
        d.ellipse([x0 + 2, 8, x0 + 5, 11], fill=(30, 30, 34, 255),
                  outline=GRAY_M)
    d.rectangle([13, 12, 19, 16], fill=GRAY_M, outline=OUTLINE)
    d.ellipse([14, 9, 18, 13], fill=GRAY_M, outline=OUTLINE)
    return img


def combo_cover(rng):
    img, d = canvas()
    d.pieslice([5, 6, 27, 24], 180, 360, fill=(238, 238, 240, 255),
               outline=OUTLINE)
    d.rectangle([5, 14, 27, 18], fill=(238, 238, 240, 255))
    d.rectangle([5, 16, 27, 19], fill=(40, 44, 60, 255), outline=OUTLINE)
    d.ellipse([3, 18, 29, 23], fill=(52, 56, 72, 255), outline=OUTLINE)
    d.ellipse([14, 5, 18, 8], fill=(200, 200, 205, 255), outline=OUTLINE)
    d.point((16, 17), fill=(220, 190, 90, 255))
    return img


def fedora(rng):
    img, d = canvas()
    d.ellipse([2, 17, 30, 24], fill=(106, 70, 40, 255), outline=LEATHER_D)
    d.pieslice([8, 4, 24, 22], 180, 360, fill=LEATHER, outline=LEATHER_D)
    d.rectangle([8, 13, 24, 18], fill=LEATHER)
    d.line([(11, 6), (13, 12)], fill=LEATHER_D, width=1)
    d.line([(21, 6), (19, 12)], fill=LEATHER_D, width=1)
    d.rectangle([8, 15, 24, 18], fill=(52, 36, 24, 255))
    return img


def electric_blanket_on(rng):
    img, d = canvas()
    rrect(d, [5, 5, 27, 25], 3, fill=(128, 88, 58, 255), outline=(84, 56, 34, 255))
    # warm glow patches (pale opaque orange)
    d.ellipse([8, 8, 16, 15], fill=(196, 128, 74, 255))
    d.ellipse([17, 15, 25, 23], fill=(196, 128, 74, 255))
    # heating coil zigzag
    pts = []
    for row, y in enumerate((10, 15, 20)):
        xs = range(7, 26) if row % 2 == 0 else range(25, 6, -1)
        for x in xs:
            pts.append((x, y + (1 if (x % 4 == 0) else -1)))
    d.line(pts, fill=(240, 176, 96, 255), width=1)
    # power cord + plug
    d.arc([20, 22, 31, 31], 270, 90, fill=(40, 40, 44, 255), width=1)
    d.rectangle([27, 26, 30, 29], fill=GRAY_D, outline=OUTLINE)
    return img


def fitness_band(rng):
    img, d = canvas()
    rrect(d, [11, 3, 21, 29], 5, fill=(70, 72, 78, 255), outline=OUTLINE)
    rrect(d, [9, 11, 23, 21], 2, fill=GRAY_L, outline=OUTLINE)
    # pulse trace on the little screen
    d.line([(11, 17), (14, 17), (15, 13), (17, 20), (18, 16), (21, 16)],
           fill=(200, 40, 40, 255), width=1)
    d.point((20, 12), fill=(200, 40, 40, 255))
    for y in (6, 25):
        d.point((16, y), fill=GRAY_M)
    return img


def flashlight_mount_wrist(rng):
    img, d = canvas()
    # wrist strap
    rrect(d, [4, 13, 28, 20], 3, fill=(52, 54, 58, 255), outline=OUTLINE)
    d.rectangle([14, 13, 18, 20], fill=GRAY_M)
    # empty flashlight cradle ring on top
    d.ellipse([10, 3, 22, 13], fill=(88, 90, 96, 255), outline=OUTLINE)
    d.ellipse([12, 5, 20, 11], fill=(16, 16, 20, 255))
    d.rectangle([14, 11, 18, 14], fill=(88, 90, 96, 255), outline=OUTLINE)
    return img


def holo_cloak(rng):
    img, d = canvas()
    # hood
    d.pieslice([9, 2, 23, 15], 180, 360, fill=GRAY_L, outline=GRAY_M)
    d.ellipse([12, 6, 20, 13], fill=(40, 44, 52, 255))
    # cloak body
    d.polygon([(9, 10), (23, 10), (27, 28), (22, 26), (17, 29), (11, 26),
               (5, 28)], fill=(178, 182, 188, 255), outline=GRAY_M)
    # hexagonal carbide sheets
    for (hx, hy) in [(12, 14), (18, 14), (15, 19), (11, 22), (20, 22),
                     (16, 25), (21, 17)]:
        d.polygon([(hx, hy - 2), (hx + 2, hy - 1), (hx + 2, hy + 1),
                   (hx, hy + 2), (hx - 2, hy + 1), (hx - 2, hy - 1)],
                  outline=(96, 130, 150, 255))
    d.line([(10, 12), (7, 26)], fill=(230, 235, 240, 255), width=1)
    return img


def rad_monitor(rng):
    img, d = canvas()
    rrect(d, [12, 2, 20, 30], 4, fill=(52, 54, 58, 255), outline=OUTLINE)
    rrect(d, [8, 9, 24, 23], 2, fill=GRAY_D, outline=OUTLINE)
    rrect(d, [10, 11, 22, 19], 1, fill=(36, 44, 40, 255))
    # trefoil on screen
    cx, cy = 16, 15
    for ang in (90, 210, 330):
        x = cx + int(2.6 * math.cos(math.radians(ang)))
        y = cy - int(2.6 * math.sin(math.radians(ang)))
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(240, 200, 60, 255))
    d.point((cx, cy), fill=(240, 200, 60, 255))
    d.line([(11, 21), (21, 21)], fill=(90, 200, 110, 255), width=1)
    d.point((22, 12), fill=(220, 60, 50, 255))
    return img


# ------------------------------------------------------------------ goggles
def goggles(kind, on):
    def render(rng):
        img, d = canvas()
        # strap
        d.rectangle([1, 11, 5, 16], fill=GRAY_D)
        d.rectangle([27, 11, 31, 16], fill=GRAY_D)
        # frame
        rrect(d, [4, 8, 28, 21], 4, fill=(44, 46, 52, 255), outline=OUTLINE)
        if kind == "ir":
            off_c, on_c = (140, 38, 32, 255), (235, 72, 52, 255)
            glow = (250, 170, 140, 255)
            # sensor box on top
            d.rectangle([13, 5, 19, 8], fill=GRAY_M, outline=OUTLINE)
        else:
            off_c, on_c = (36, 100, 56, 255), (86, 228, 118, 255)
            glow = (196, 250, 204, 255)
            # IR emitter nub on the side
            d.rectangle([26, 6, 29, 9], fill=GRAY_M, outline=OUTLINE)
        lens_c = on_c if on else off_c
        for x0 in (6, 18):
            rrect(d, [x0, 11, x0 + 8, 18], 2, fill=lens_c, outline=OUTLINE)
            d.point((x0 + 2, 12), fill=(245, 248, 248, 255))
        if on:
            for (gx, gy) in [(5, 10), (27, 10), (5, 19), (27, 19), (16, 7)]:
                d.point((gx, gy), fill=glow)
            d.point((28, 7) if kind == "nv" else (16, 6),
                    fill=(255, 240, 150, 255))
        return img
    return render


# ------------------------------------------------------------------ helmets
def helmet_eod(on):
    def render(rng):
        img, d = canvas()
        olive = (124, 164, 82, 255)
        olive_d = (92, 124, 58, 255)
        d.pieslice([4, 3, 28, 26], 180, 360, fill=olive, outline=olive_d)
        d.rectangle([4, 14, 28, 20], fill=olive)
        d.rectangle([4, 19, 28, 22], fill=olive_d, outline=olive_d)
        # visor
        rrect(d, [9, 13, 23, 19], 2, fill=LENS_D, outline=OUTLINE)
        d.line([(11, 15), (16, 15)], fill=(210, 235, 240, 255), width=1)
        # headlamp
        d.rectangle([13, 4, 19, 8], fill=GRAY_D, outline=OUTLINE)
        lamp = (255, 238, 160, 255) if on else (150, 150, 140, 255)
        d.ellipse([14, 5, 18, 8], fill=lamp)
        # camera bump + comms nub
        d.rectangle([24, 8, 28, 12], fill=olive_d, outline=OUTLINE)
        if on:
            d.point((26, 9), fill=(240, 60, 50, 255))
            # lamp beam: pale wedge in front
            beam = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
            bd = ImageDraw.Draw(beam)
            bd.polygon([(14, 9), (18, 9), (22, 17), (10, 17)],
                       fill=(250, 240, 170, 90))
            img.alpha_composite(beam)
            d = ImageDraw.Draw(img)
            d.ellipse([14, 5, 18, 8], fill=lamp)
        d.rectangle([6, 22, 26, 25], fill=olive_d)
        return img
    return render


def helmet_riot(raised):
    def render(rng):
        img, d = canvas()
        d.pieslice([5, 3, 27, 24], 180, 360, fill=(66, 68, 74, 255),
                   outline=OUTLINE)
        d.rectangle([5, 13, 27, 17], fill=(66, 68, 74, 255))
        d.rectangle([4, 16, 28, 19], fill=(48, 50, 56, 255), outline=OUTLINE)
        if raised:
            # shield flipped up above the brim
            rrect(d, [8, 1, 24, 7], 2, fill=(140, 170, 185, 255),
                  outline=OUTLINE)
            d.line([(10, 3), (20, 3)], fill=(215, 235, 242, 255), width=1)
            # open face
            d.ellipse([11, 19, 21, 27], fill=(34, 32, 34, 255))
        else:
            # shield down over the face
            rrect(d, [8, 17, 24, 29], 3, fill=(140, 170, 185, 255),
                  outline=OUTLINE)
            d.line([(11, 19), (14, 26)], fill=(220, 240, 246, 255), width=2)
            d.line([(18, 19), (20, 24)], fill=(200, 225, 235, 255), width=1)
        return img
    return render


# --------------------------------------------------------------------- masks
def mask_bunker(on):
    def render(rng):
        img, d = canvas()
        rrect(d, [6, 4, 26, 28], 6, fill=GRAY_L, outline=GRAY_M)
        # big single visor
        rrect(d, [9, 7, 23, 16], 3, fill=LENS, outline=GRAY_D)
        d.line([(11, 9), (16, 9)], fill=(240, 250, 252, 255), width=1)
        # regulator
        d.ellipse([12, 19, 20, 27], fill=GRAY_D, outline=OUTLINE)
        d.ellipse([14, 21, 18, 25], fill=GRAY_M)
        d.point((16, 23), fill=(30, 30, 34, 255))
        # side straps / hose port
        d.rectangle([4, 12, 6, 17], fill=GRAY_M)
        if on:
            # corrugated hose curving away from the regulator
            for i in range(5):
                x = 10 - i
                y = 25 + i
                if y < 32:
                    d.ellipse([x - 1, y - 1, x + 1, y + 1], outline=GRAY_D)
            d.point((23, 20), fill=(90, 220, 110, 255))
        return img
    return render


def survivor_mask(variant, size, on=False):
    """size: half-width scale factor (xs .8, normal .95, xl 1.08)."""
    def render(rng):
        img, d = canvas()
        cx = 16
        w = 11 * size
        top, bot = 4, 28
        body = LEATHER if variant != "dive" else (56, 58, 64, 255)
        edge = LEATHER_D if variant != "dive" else OUTLINE
        rrect(d, [cx - w, top, cx + w, bot], 5, fill=body, outline=edge)
        # side straps
        d.rectangle([cx - w - 2, 12, cx - w, 17], fill=edge)
        d.rectangle([cx + w, 12, cx + w + 2, 17], fill=edge)
        if variant == "dive":
            # single wide visor
            rrect(d, [cx - w + 2, 8, cx + w - 2, 16], 3, fill=LENS,
                  outline=OUTLINE)
            d.line([(cx - w + 4, 10), (cx - 2, 10)],
                   fill=(240, 250, 252, 255), width=1)
            # rebreather chin box
            rrect(d, [cx - 4, 20, cx + 4, 27], 2, fill=GRAY_D,
                  outline=OUTLINE)
            for gx in range(-2, 3):
                d.line([(cx + gx, 22), (cx + gx, 25)], fill=GRAY_M, width=1)
            if on:
                for (bx, by, br) in [(26, 6, 1), (28, 3, 1), (25, 2, 0)]:
                    d.ellipse([bx - br, by - br, bx + br, by + br],
                              outline=(210, 240, 248, 255))
                d.point((cx + 5, 23), fill=(90, 220, 110, 255))
            return img
        if variant == "half":
            # covers nose/mouth only: redraw as a lower-face piece
            img2, d2 = canvas()
            rrect(d2, [cx - w + 1, 13, cx + w - 1, 28], 5, fill=body,
                  outline=edge)
            d2.line([(cx - w - 1, 10), (cx - w + 2, 15)], fill=edge, width=2)
            d2.line([(cx + w + 1, 10), (cx + w - 2, 15)], fill=edge, width=2)
            d2.ellipse([cx - 4, 18, cx + 4, 26], fill=(70, 62, 54, 255),
                       outline=OUTLINE)
            d2.ellipse([cx - 2, 20, cx + 2, 24], fill=(44, 40, 36, 255))
            return img2
        # two round eye lenses
        for ex in (cx - 5, cx + 5):
            d.ellipse([ex - 3, 9, ex + 3, 15], fill=LENS_D, outline=OUTLINE)
            d.ellipse([ex - 2, 10, ex + 2, 14], fill=LENS)
            d.point((ex - 1, 11), fill=(245, 252, 253, 255))
        if variant == "fire":
            # Nomex face panel + stitching
            d.polygon([(cx - w + 1, 5), (cx + w - 1, 5), (cx + w - 3, 12),
                       (cx - w + 3, 12)], fill=NOMEX)
            for ex in (cx - 5, cx + 5):
                d.ellipse([ex - 3, 9, ex + 3, 15], fill=LENS_D,
                          outline=OUTLINE)
                d.ellipse([ex - 2, 10, ex + 2, 14], fill=LENS)
            for sx in range(int(cx - w + 3), int(cx + w - 2), 3):
                d.point((sx, 17), fill=LEATHER_D)
        if variant == "plate":
            # steel faceplate over the upper face
            rrect(d, [cx - w + 1, 5, cx + w - 1, 13], 3, fill=STEEL,
                  outline=GRAY_D)
            for ex in (cx - 5, cx + 5):
                d.ellipse([ex - 3, 8, ex + 3, 14], fill=LENS_D,
                          outline=OUTLINE)
                d.ellipse([ex - 2, 9, ex + 2, 13], fill=LENS)
            for rx in (cx - w + 3, cx + w - 3):
                d.point((rx, 6), fill=GRAY_D)
                d.point((rx, 12), fill=GRAY_D)
        # round filter canister at the mouth
        d.ellipse([cx - 4, 19, cx + 4, 27], fill=(70, 62, 54, 255),
                  outline=OUTLINE)
        for a in range(0, 360, 45):
            x = cx + int(2.6 * math.cos(math.radians(a)))
            y = 23 + int(2.6 * math.sin(math.radians(a)))
            d.point((x, y), fill=(40, 36, 32, 255))
        d.point((cx, 23), fill=(30, 28, 26, 255))
        return img
    return render


def mask_ski(loose):
    def render(rng):
        img, d = canvas()
        wool = (58, 58, 64, 255)
        if loose:
            # rolled collar around the throat
            rrect(d, [6, 14, 26, 24], 5, fill=wool, outline=OUTLINE)
            for y in (17, 20, 23):
                d.line([(8, y), (24, y)], fill=(40, 40, 46, 255), width=1)
            d.rectangle([13, 24, 19, 28], fill=wool, outline=OUTLINE)
            return img
        d.pieslice([7, 3, 25, 20], 180, 360, fill=wool, outline=OUTLINE)
        d.rectangle([7, 12, 25, 26], fill=wool)
        d.rectangle([7, 24, 25, 27], fill=wool, outline=OUTLINE)
        # eye slit
        rrect(d, [10, 12, 22, 16], 2, fill=SKIN)
        d.point((13, 14), fill=(20, 16, 14, 255))
        d.point((19, 14), fill=(20, 16, 14, 255))
        # mouth holes
        for mx in (14, 16, 18):
            d.point((mx, 21), fill=(24, 24, 28, 255))
        for y in (6, 9):
            d.arc([8, y - 2, 24, y + 5], 190, 350, fill=(42, 42, 48, 255))
        return img
    return render


def mil_vis(on):
    def render(rng):
        img, d = canvas()
        # head strap
        d.arc([3, 2, 29, 18], 180, 360, fill=GRAY_D, width=2)
        # camera array bumps on top
        for bx in (9, 15, 21):
            d.rectangle([bx - 2, 5, bx + 2, 9], fill=GRAY_M, outline=OUTLINE)
            d.point((bx, 7), fill=(30, 40, 60, 255))
        # broad eyepiece
        rrect(d, [5, 9, 27, 18], 3, fill=(36, 38, 44, 255), outline=OUTLINE)
        if on:
            rrect(d, [7, 11, 25, 16], 2, fill=(64, 190, 205, 255))
            d.rectangle([9, 12, 13, 15], outline=(220, 250, 252, 255))
            d.line([(17, 14), (23, 14)], fill=(220, 250, 252, 255), width=1)
            d.point((24, 12), fill=(255, 120, 90, 255))
        else:
            rrect(d, [7, 11, 25, 16], 2, fill=(26, 44, 54, 255))
            d.line([(9, 12), (14, 12)], fill=(120, 150, 160, 255), width=1)
        # computer puck on the side with cable
        d.rectangle([25, 18, 30, 24], fill=GRAY_D, outline=OUTLINE)
        d.point((27, 20), fill=(90, 200, 110, 255) if on else GRAY_M)
        d.arc([18, 18, 30, 30], 270, 360, fill=GRAY_D, width=1)
        return img
    return render


# -------------------------------------------------------------- hearing gear
def earmuffs(on):
    def render(rng):
        img, d = canvas()
        d.arc([6, 3, 26, 20], 180, 360, fill=BLUE_D, width=3)
        for x0 in (3, 21):
            rrect(d, [x0, 12, x0 + 8, 24], 3, fill=BLUE, outline=BLUE_D)
            rrect(d, [x0 + 2, 15, x0 + 6, 21], 2, fill=BLUE_D)
        if on:
            d.point((7, 13), fill=(120, 255, 140, 255))
            for cx, flip in ((2, -1), (30, 1)):
                d.arc([cx - 3, 13, cx + 3, 23], 300 if flip < 0 else 120,
                      60 if flip < 0 else 240, fill=(150, 200, 255, 255))
        return img
    return render


def earplugs_powered(on):
    def render(rng):
        img, d = canvas()
        # connecting wire
        pts = []
        for t in range(17):
            x = 9 + t * 0.875
            y = 20 + 5 * math.sin(math.pi * t / 16)
            pts.append((x, y))
        d.line(pts, fill=BLUE_D, width=1)
        for cx in (9, 23):
            d.ellipse([cx - 3, 9, cx + 3, 16], fill=BLUE, outline=BLUE_D)
            d.rectangle([cx - 1, 6, cx + 1, 10], fill=BLUE_D)
            if on:
                d.point((cx, 11), fill=(120, 255, 140, 255))
        if on:
            for cx in (3, 29):
                d.arc([cx - 2, 9, cx + 2, 17], 270, 90, fill=(150, 200, 255, 255))
        return img
    return render


# ---------------------------------------------------------------- rebreather
def rebreather(xl, on):
    def render(rng):
        img, d = canvas()
        w = 13 if xl else 11
        cx = 16
        top = 8 if xl else 10
        # hose loop below
        d.arc([cx - 8, 16, cx + 8, 30], 0, 180, fill=GRAY_M, width=2)
        d.arc([cx - 8, 14, cx + 8, 28], 0, 180, fill=GRAY_D, width=1)
        # mask body over the mouth
        rrect(d, [cx - w, top, cx + w, top + 11], 4, fill=(64, 66, 72, 255),
              outline=OUTLINE)
        # side filter cylinders
        for fx in (cx - w - 2, cx + w - 2):
            d.rectangle([fx, top + 3, fx + 4, top + 9], fill=GRAY_M,
                        outline=OUTLINE)
        # center valve
        d.ellipse([cx - 3, top + 3, cx + 3, top + 9], fill=GRAY_L,
                  outline=OUTLINE)
        d.point((cx, top + 6), fill=GRAY_D)
        # straps
        d.line([(cx - w, top + 2), (cx - w - 3, top - 2)], fill=GRAY_D,
               width=2)
        d.line([(cx + w, top + 2), (cx + w + 3, top - 2)], fill=GRAY_D,
               width=2)
        if on:
            for (bx, by, br) in [(27, 6, 1), (29, 3, 1), (25, 2, 0),
                                 (5, 5, 1)]:
                d.ellipse([bx - br, by - br, bx + br, by + br],
                          outline=(210, 240, 248, 255))
        return img
    return render


RENDER = {
    "attached_ear_plugs_off": earplugs(GRAY_L, GRAY_M),
    "attached_ear_plugs_on": earplugs_worn(GRAY_L, GRAY_M),
    "binoculars": binoculars,
    "combo_cover": combo_cover,
    "electric_blanket_on": electric_blanket_on,
    "fedora": fedora,
    "fitness_band": fitness_band,
    "flashlight_mount_wrist": flashlight_mount_wrist,
    "goggles_ir": goggles("ir", False),
    "goggles_ir_on": goggles("ir", True),
    "goggles_nv": goggles("nv", False),
    "goggles_nv_on": goggles("nv", True),
    "helmet_eod": helmet_eod(False),
    "helmet_eod_on": helmet_eod(True),
    "helmet_riot": helmet_riot(False),
    "helmet_riot_raised": helmet_riot(True),
    "holo_cloak": holo_cloak,
    "makeshift_ear_plugs_off": earplugs((206, 178, 120, 255),
                                        (170, 140, 84, 255), wax=True),
    "makeshift_ear_plugs_on": earplugs_worn((206, 178, 120, 255),
                                            (170, 140, 84, 255), wax=True),
    "mask_bunker": mask_bunker(False),
    "mask_bunker_scba_on": mask_bunker(True),
    "mask_fsurvivor": survivor_mask("fire", 0.95),
    "mask_fsurvivorxl": survivor_mask("fire", 1.08),
    "mask_fsurvivorxs": survivor_mask("fire", 0.8),
    "mask_h20survivor": survivor_mask("dive", 0.95),
    "mask_h20survivor_on": survivor_mask("dive", 0.95, on=True),
    "mask_h20survivorxl": survivor_mask("dive", 1.08),
    "mask_h20survivorxl_on": survivor_mask("dive", 1.08, on=True),
    "mask_h20survivorxs": survivor_mask("dive", 0.8),
    "mask_h20survivorxs_on": survivor_mask("dive", 0.8, on=True),
    "mask_hsurvivor": survivor_mask("plate", 0.95),
    "mask_hsurvivorxl": survivor_mask("plate", 1.08),
    "mask_hsurvivorxs": survivor_mask("plate", 0.8),
    "mask_lsurvivor": survivor_mask("half", 0.95),
    "mask_lsurvivorxl": survivor_mask("half", 1.08),
    "mask_lsurvivorxs": survivor_mask("half", 0.8),
    "mask_ski": mask_ski(False),
    "mask_ski_loose": mask_ski(True),
    "mask_survivor": survivor_mask("plain", 0.95),
    "mask_survivorxl": survivor_mask("plain", 1.08),
    "mask_survivorxs": survivor_mask("plain", 0.8),
    "military_vis_aug_system": mil_vis(False),
    "military_vis_aug_system_on": mil_vis(True),
    "powered_earmuffs": earmuffs(False),
    "powered_earmuffs_on": earmuffs(True),
    "powered_earplugs": earplugs_powered(False),
    "powered_earplugs_on": earplugs_powered(True),
    "rad_monitor": rad_monitor,
    "rebreather": rebreather(False, False),
    "rebreather_on": rebreather(False, True),
    "rebreather_xl": rebreather(True, False),
    "rebreather_xl_on": rebreather(True, True),
}


def main():
    assert set(RENDER) == set(IDS), set(IDS) ^ set(RENDER)
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = RENDER[item_id](rng)
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
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item12.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item12.png {sheet.size}")


if __name__ == "__main__":
    main()
