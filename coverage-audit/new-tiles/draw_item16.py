#!/usr/bin/env python3
"""Batch ITEM16 (Phase 2g.4 scope extension): head
attachments, first set.

52 ids from data/json/items/armor/head_attachments.json
(authoritative list embedded below; source list:
~/workspace/item-batches/16.txt). Copy-from chains were
resolved against the data before drawing. Parameterized
family renderers driven by each entry's data:

- Night vision devices (dark gray plastic/steel/glass on a
  helmet mount): the military double-tube NVG raised when
  off, its single-tube form glowing green when on; the
  four-tube panoramic GPNVG, raised / on; the enhanced
  thermal NVG with its central thermal sensor, on-variants
  showing amber thermal glow.
- Attachable ear muffs (yellow): cups flipped up in the
  standby state, cups lowered over the ears when on.
- Jaw protection: the dark ballistic mandible guard on its
  rail mounts, the light-blue plastic mandible guard, and
  the small plastic chin cup on straps.
- Face shields on their mounting brackets: thermoplastic
  and cheaper plastic shields in pale blue, the thick
  layered EOD shield in gray, the short shield visor, and
  the opaque sun shield brim.
- Nape protectors hanging from a band: cyan cotton cloth,
  yellow mesh, dark Kevlar weave.
- The green nylon counterweight pouch with flap and MOLLE
  straps; the vision-correction insert as a spectacle frame
  of prescription lenses on its clip; the three gas-mask
  outsert lenses (clear, dark sunlight tint, amber breach
  lens with reinforced rim).
- The chainmail family in five steel grades (mild lc,
  medium mc, high hc, hardened ch, tempered qt) plus the
  xl_ oversize variants: aventails as wide ring-mail drapes
  to the shoulders, partial aventails as shorter drapes
  with a face opening, and solid steel facemasks -- crude
  and dull in mild steel, lightly ornamented in medium,
  mustached in high steel, gold-inlaid in hardened, and
  gold-inlaid with a gem setting in tempered steel.

Items are objects on transparency, native 32x32, flat
UltiCa style, bodies fully opaque (alpha in {0,255});
translucency faked with pale opaque colors.
PIL only, per-sprite seeded random.Random (crc32 of id).
Writes ONLY its own 52 folders + preview_item16.png.
"""
import json
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
    "advanced_gpnvg",
    "advanced_gpnvg_on",
    "attachable_ear_muffs",
    "attachable_ear_muffs_on",
    "ballistic_mandible_guard",
    "ch_chainmail_aventail",
    "ch_chainmail_face",
    "ch_steel_facemask",
    "counterweight_pouch",
    "enhanced_nvg",
    "enhanced_nvg_on",
    "face_shield",
    "face_shield_eod",
    "face_shield_plastic",
    "hc_chainmail_aventail",
    "hc_chainmail_face",
    "hc_steel_facemask",
    "insert_vision_correction",
    "kevlar_nape_protector",
    "lc_chainmail_aventail",
    "lc_chainmail_face",
    "lc_steel_facemask",
    "mc_chainmail_aventail",
    "mc_chainmail_face",
    "mc_steel_facemask",
    "mesh_nape_protector",
    "military_nvg",
    "military_nvg_on",
    "nape_protector",
    "outsert_breach",
    "outsert_clear",
    "outsert_sunlight",
    "plastic_chin_guard",
    "plastic_mandible_guard",
    "qt_chainmail_aventail",
    "qt_chainmail_face",
    "qt_steel_facemask",
    "shield_visor",
    "sun_shield",
    "xl_ch_chainmail_aventail",
    "xl_ch_chainmail_face",
    "xl_ch_steel_facemask",
    "xl_hc_chainmail_aventail",
    "xl_hc_chainmail_face",
    "xl_hc_steel_facemask",
    "xl_lc_chainmail_aventail",
    "xl_lc_chainmail_face",
    "xl_lc_steel_facemask",
    "xl_mc_chainmail_aventail",
    "xl_mc_chainmail_face",
    "xl_mc_steel_facemask",
    "xl_qt_chainmail_aventail",
]

OUTLINE = (28, 28, 34, 255)
DGRAY = (98, 98, 108, 255)        # data color dark_gray
DGRAY_D = (66, 66, 74, 255)
DGRAY_L = (134, 134, 144, 255)
LGRAY = (178, 178, 184, 255)      # data color light_gray
LGRAY_D = (138, 138, 146, 255)
LGRAY_L = (208, 208, 214, 255)
STEEL = (118, 124, 134, 255)
STEEL_D = (84, 90, 100, 255)
RUBBER = (42, 42, 48, 255)
LENS_OFF = (38, 52, 66, 255)      # unlit glass
NV_GLOW = (110, 240, 130, 255)    # phosphor green
NV_GLOW_D = (52, 160, 74, 255)
THERMAL = (255, 150, 52, 255)     # thermal amber
THERMAL_L = (255, 205, 110, 255)
YELLOW = (238, 200, 52, 255)      # data color yellow
YELLOW_D = (196, 156, 32, 255)
LBLUE = (168, 208, 232, 255)      # data color light_blue (pale shield)
LBLUE_D = (120, 168, 198, 255)
LBLUE_L = (214, 238, 250, 255)
CYAN = (72, 196, 206, 255)        # data color cyan
CYAN_D = (44, 148, 158, 255)
GREEN = (86, 138, 62, 255)        # data color green (nylon pouch)
GREEN_D = (58, 96, 42, 255)
GREEN_L = (118, 172, 88, 255)
GOLD = (228, 188, 62, 255)
GOLD_D = (176, 138, 38, 255)
GEM = (206, 52, 64, 255)


def canvas():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def speckle(rng, d, box, colors, n):
    for _ in range(n):
        x = rng.randint(box[0], box[2])
        y = rng.randint(box[1], box[3])
        d.point((x, y), fill=rng.choice(colors))


def _mount_bracket(d, y=4):
    # helmet mount: central post + dovetail base
    d.rectangle([13, y, 19, y + 3], fill=STEEL_D, outline=OUTLINE)
    d.rectangle([11, y + 3, 21, y + 5], fill=DGRAY_D, outline=OUTLINE)
    d.point((16, y + 1), fill=STEEL)


# ------------------------------------------------------------------
# Night vision devices
# ------------------------------------------------------------------

def _tube(d, cx, cy, r, on, glow=NV_GLOW, glow_d=NV_GLOW_D):
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=DGRAY, outline=OUTLINE)
    d.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2],
              fill=DGRAY_D)
    if on:
        d.ellipse([cx - r + 3, cy - r + 3, cx + r - 3, cy + r - 3],
                  fill=glow_d)
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=glow)
        d.point((cx - 1, cy - 1), fill=(235, 255, 240, 255))
    else:
        d.ellipse([cx - r + 3, cy - r + 3, cx + r - 3, cy + r - 3],
                  fill=LENS_OFF)
        d.point((cx - 2, cy - 2), fill=(120, 150, 175, 255))


def military_nvg(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    _mount_bracket(d, 3)
    if not on:
        # raised up and out of the way: body hinged high, tubes
        # tilted up, bridge arm visible below
        d.rectangle([15, 8, 17, 13], fill=STEEL_D)
        d.rounded_rectangle([6, 12, 26, 20], 3, fill=DGRAY,
                            outline=OUTLINE)
        _tube(d, 11, 16, 4, False)
        _tube(d, 21, 16, 4, False)
        d.rectangle([14, 14, 18, 18], fill=DGRAY_D)
    else:
        # on: single central tube lowered over the eye line
        d.rectangle([15, 8, 17, 11], fill=STEEL_D)
        d.rounded_rectangle([10, 11, 22, 23], 3, fill=DGRAY,
                            outline=OUTLINE)
        _tube(d, 16, 17, 6, True)
        d.ellipse([20, 12, 22, 14], fill=NV_GLOW)
    return img


def gpnvg(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    _mount_bracket(d, 3)
    top = 13 if on else 11
    if not on:
        d.rectangle([15, 8, 17, 12], fill=STEEL_D)
    # wide panoramic housing: four tubes, outer pair angled out
    d.rounded_rectangle([3, top, 29, top + 9], 3, fill=DGRAY,
                        outline=OUTLINE)
    _tube(d, 8, top + 4, 3, on)
    _tube(d, 14, top + 5, 4, on)
    _tube(d, 19, top + 5, 4, on)
    _tube(d, 25, top + 4, 3, on)
    if on:
        d.ellipse([15, top - 2, 17, top], fill=NV_GLOW)
    return img


def enhanced_nvg(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    _mount_bracket(d, 3)
    top = 12 if on else 10
    if not on:
        d.rectangle([15, 8, 17, 11], fill=STEEL_D)
    d.rounded_rectangle([6, top, 26, top + 10], 3, fill=DGRAY,
                        outline=OUTLINE)
    # central thermal sensor block between the tubes
    d.rectangle([13, top + 1, 19, top + 6], fill=DGRAY_D, outline=OUTLINE)
    if on:
        d.rectangle([14, top + 2, 18, top + 5], fill=THERMAL)
        d.point((16, top + 3), fill=THERMAL_L)
    else:
        d.rectangle([14, top + 2, 18, top + 5], fill=LENS_OFF)
    _tube(d, 10, top + 5, 4, on, THERMAL, (196, 96, 30, 255))
    _tube(d, 22, top + 5, 4, on, THERMAL, (196, 96, 30, 255))
    if on:
        # detected heat signatures at the frame edge
        for px, py in ((4, 26), (27, 24), (24, 28)):
            d.point((px, py), fill=(255, 90, 60, 255))
    return img


# ------------------------------------------------------------------
# Ear muffs
# ------------------------------------------------------------------

def ear_muffs(rng, on):
    img = canvas()
    d = ImageDraw.Draw(img)
    # headband arc with helmet clips at its ends
    d.arc([5, 2, 27, 18], 180, 360, fill=YELLOW_D, width=3)
    d.arc([5, 2, 27, 18], 180, 360, fill=YELLOW, width=1)
    for x in (4, 26):
        d.rectangle([x, 9, x + 3, 13], fill=DGRAY_D, outline=OUTLINE)
    if not on:
        # standby: cups flipped up, resting above the band ends
        for cx in (6, 26):
            d.ellipse([cx - 4, 2, cx + 4, 12], fill=YELLOW, outline=OUTLINE)
            d.ellipse([cx - 2, 4, cx + 2, 10], fill=YELLOW_D)
    else:
        # working position: cups lowered over the ears, pads in
        for cx in (6, 26):
            d.rectangle([cx - 1, 10, cx + 1, 13], fill=STEEL_D)
            d.ellipse([cx - 5, 13, cx + 5, 25], fill=YELLOW,
                      outline=OUTLINE)
            d.ellipse([cx - 3, 15, cx + 3, 23], fill=(90, 84, 60, 255))
            d.ellipse([cx - 1, 17, cx + 1, 21], fill=YELLOW_D)
    return img


# ------------------------------------------------------------------
# Jaw protection
# ------------------------------------------------------------------

def mandible_guard(rng, ballistic):
    img = canvas()
    d = ImageDraw.Draw(img)
    if ballistic:
        body, dark, light = DGRAY, DGRAY_D, DGRAY_L
    else:
        body, dark, light = (140, 190, 224, 255), LBLUE_D, LBLUE_L
    # side arms rising to the helmet rail mounts
    for x in (5, 24):
        d.rectangle([x, 4, x + 3, 16], fill=dark, outline=OUTLINE)
        d.rectangle([x, 2, x + 3, 5], fill=body, outline=OUTLINE)
        for yy in (7, 10, 13):
            d.point((x + 1, yy), fill=OUTLINE)
    # curved jaw plate across the bottom
    d.pieslice([4, 8, 28, 30], 0, 180, fill=body, outline=OUTLINE)
    d.pieslice([7, 11, 25, 29], 0, 180, fill=dark)
    d.pieslice([8, 12, 24, 28], 0, 180, fill=body)
    # ventilation slots
    for x in (11, 15, 19):
        d.rectangle([x, 21, x + 2, 24], fill=dark)
    d.line([(10, 17), (22, 17)], fill=light)
    if ballistic:
        speckle(rng, d, (6, 14, 26, 26), [dark], 8)
    return img


def chin_guard(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # straps rising diagonally to the hard-hat straps
    d.line([(12, 18), (6, 4)], fill=LGRAY_D, width=2)
    d.line([(20, 18), (26, 4)], fill=LGRAY_D, width=2)
    d.line([(12, 18), (7, 5)], fill=LGRAY_L, width=1)
    # the chin cup itself
    d.pieslice([9, 14, 23, 28], 0, 180, fill=LGRAY, outline=OUTLINE)
    d.pieslice([11, 16, 21, 26], 0, 180, fill=LGRAY_D)
    d.pieslice([12, 17, 20, 25], 0, 180, fill=LGRAY_L)
    for x in (14, 17):
        d.point((x, 21), fill=LGRAY_D)
    return img


# ------------------------------------------------------------------
# Face shields and visors
# ------------------------------------------------------------------

def face_shield(rng, kind):
    img = canvas()
    d = ImageDraw.Draw(img)
    # mounting bracket band across the top
    d.rectangle([5, 4, 27, 8], fill=DGRAY_D, outline=OUTLINE)
    d.rectangle([5, 4, 27, 5], fill=DGRAY_L)
    for x in (8, 15, 22):
        d.rectangle([x, 8, x + 2, 10], fill=STEEL_D)
    if kind == "eod":
        # thick layered polycarbonate: heavy frame, double edge
        d.rounded_rectangle([6, 10, 26, 29], 4, fill=LGRAY_D,
                            outline=OUTLINE)
        d.rounded_rectangle([8, 12, 24, 27], 3, fill=(196, 200, 204, 255),
                            outline=LGRAY_D)
        d.rounded_rectangle([10, 14, 22, 25], 2,
                            fill=(216, 220, 224, 255))
        d.line([(12, 15), (16, 24)], fill=(238, 240, 242, 255), width=2)
        return img
    tint = LBLUE if kind in ("thermo", "plastic") else LBLUE
    edge = LBLUE_D if kind == "thermo" else (150, 196, 226, 255)
    bottom = 29 if kind != "visor" else 21
    d.rounded_rectangle([6, 10, 26, bottom], 4, fill=tint,
                        outline=OUTLINE)
    d.rounded_rectangle([8, 12, 24, bottom - 2], 3, fill=LBLUE_L)
    # sheen streak
    d.line([(11, 13), (15, bottom - 4)], fill=(240, 250, 253, 255),
           width=2)
    d.line([(19, 12), (21, 18)], fill=(226, 244, 250, 255), width=1)
    if kind == "plastic":
        # cheaper plastic: slight warp marks, thinner edge
        speckle(rng, d, (8, 12, 24, 27), [edge], 5)
    return img


def sun_shield(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # band that slips over the hard hat
    d.rectangle([8, 5, 24, 9], fill=LGRAY_D, outline=OUTLINE)
    d.rectangle([8, 5, 24, 6], fill=LGRAY_L)
    # opaque brim jutting forward: drawn as a wide shade plate
    d.polygon([(3, 12), (29, 12), (27, 21), (5, 21)], fill=LGRAY,
              outline=OUTLINE)
    d.polygon([(5, 13), (27, 13), (26, 16), (6, 16)], fill=LGRAY_L)
    d.line([(6, 19), (26, 19)], fill=LGRAY_D)
    speckle(rng, d, (6, 13, 26, 20), [LGRAY_D], 6)
    return img


# ------------------------------------------------------------------
# Nape protectors
# ------------------------------------------------------------------

def nape_protector(rng, kind):
    img = canvas()
    d = ImageDraw.Draw(img)
    # attachment band for the helmet rim
    d.rectangle([7, 4, 25, 8], fill=DGRAY_D, outline=OUTLINE)
    d.rectangle([7, 4, 25, 5], fill=DGRAY_L)
    if kind == "cyan":
        body, dark, light = CYAN, CYAN_D, (140, 226, 232, 255)
    elif kind == "mesh":
        body, dark, light = YELLOW, YELLOW_D, (250, 226, 120, 255)
    else:  # kevlar
        body, dark, light = DGRAY, DGRAY_D, DGRAY_L
    # hanging cloth flap, wider at the bottom
    d.polygon([(8, 8), (24, 8), (26, 27), (6, 27)], fill=body,
              outline=OUTLINE)
    d.line([(16, 8), (16, 27)], fill=dark)
    if kind == "mesh":
        # open mesh: grid of holes
        for yy in range(11, 26, 3):
            for xx in range(9, 25, 3):
                d.point((xx, yy), fill=(120, 96, 20, 255))
                d.point((xx + 1, yy + 1), fill=light)
    elif kind == "kevlar":
        # woven crosshatch
        for yy in range(10, 27, 2):
            d.line([(8, yy), (25, yy)], fill=dark)
        for xx in range(10, 25, 3):
            d.line([(xx, 9), (xx - 1, 26)], fill=light)
    else:
        # cloth folds
        for xx in (11, 21):
            d.line([(xx, 9), (xx - 1, 26)], fill=dark)
        d.line([(7, 24), (25, 24)], fill=light)
    return img


# ------------------------------------------------------------------
# Small mounted items
# ------------------------------------------------------------------

def counterweight_pouch(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # MOLLE straps rising from the back of the pouch
    for x in (11, 18):
        d.rectangle([x, 2, x + 3, 9], fill=GREEN_D, outline=OUTLINE)
    # pouch body
    d.rounded_rectangle([7, 8, 25, 27], 3, fill=GREEN, outline=OUTLINE)
    # flap with buckle
    d.rounded_rectangle([7, 8, 25, 16], 3, fill=GREEN_L, outline=OUTLINE)
    d.rectangle([14, 13, 18, 17], fill=DGRAY_D, outline=OUTLINE)
    d.rectangle([15, 14, 17, 16], fill=DGRAY_L)
    # stitching + a battery end peeking from the side pocket
    for xx in range(9, 24, 3):
        d.point((xx, 19), fill=GREEN_D)
    d.rectangle([22, 20, 25, 24], fill=(196, 60, 52, 255), outline=OUTLINE)
    d.rectangle([23, 19, 24, 20], fill=STEEL)
    speckle(rng, d, (8, 20, 21, 26), [GREEN_D], 6)
    return img


def vision_insert(rng):
    img = canvas()
    d = ImageDraw.Draw(img)
    # clip mount that seats behind the mask visor
    d.rectangle([13, 3, 19, 7], fill=STEEL_D, outline=OUTLINE)
    d.rectangle([15, 7, 17, 9], fill=STEEL)
    # spectacle frame: two round prescription lenses + bridge
    for cx in (10, 22):
        d.ellipse([cx - 6, 9, cx + 6, 23], fill=(206, 226, 234, 255),
                  outline=OUTLINE)
        d.ellipse([cx - 4, 11, cx + 4, 21], fill=(226, 240, 246, 255))
        d.line([(cx - 3, 12), (cx, 19)], fill=(246, 252, 254, 255),
               width=1)
        d.point((cx + 2, 13), fill=(255, 255, 255, 255))
    d.arc([13, 11, 19, 17], 180, 360, fill=OUTLINE, width=1)
    # temple stubs out to the sides
    d.line([(4, 13), (1, 11)], fill=STEEL_D, width=2)
    d.line([(28, 13), (31, 11)], fill=STEEL_D, width=2)
    return img


def outsert(rng, kind):
    img = canvas()
    d = ImageDraw.Draw(img)
    # panoramic gas-mask eyepiece: one wide lens, clip tabs at
    # the corners
    for x, y in ((5, 8), (24, 8), (5, 21), (24, 21)):
        d.rectangle([x, y, x + 3, y + 3], fill=DGRAY_D, outline=OUTLINE)
    if kind == "clear":
        fill, hi = (208, 222, 228, 255), (240, 246, 248, 255)
        rim = LGRAY_D
    elif kind == "sunlight":
        fill, hi = (74, 70, 66, 255), (120, 114, 104, 255)
        rim = DGRAY_D
    else:  # breach: dark amber, reinforced rim for torch work
        fill, hi = (148, 96, 40, 255), (206, 148, 74, 255)
        rim = (96, 60, 26, 255)
    d.rounded_rectangle([6, 9, 26, 22], 6, fill=rim, outline=OUTLINE)
    d.rounded_rectangle([8, 11, 24, 20], 5, fill=fill)
    d.line([(10, 13), (15, 19)], fill=hi, width=2)
    if kind == "breach":
        for x in (9, 15, 21):
            d.point((x, 10), fill=GOLD_D)
            d.point((x, 21), fill=GOLD_D)
    return img


# ------------------------------------------------------------------
# Chainmail family
# ------------------------------------------------------------------

# grade -> (base, dark, light, trim) ; trim: None | "gold" | "gold+gem"
GRADES = {
    "lc": ((146, 148, 154, 255), (102, 104, 110, 255),
           (176, 178, 184, 255), None),
    "mc": ((160, 163, 170, 255), (114, 117, 124, 255),
           (190, 193, 200, 255), "etch"),
    "hc": ((178, 181, 188, 255), (130, 133, 140, 255),
           (208, 211, 218, 255), "mustache"),
    "ch": ((190, 193, 200, 255), (140, 143, 150, 255),
           (220, 223, 230, 255), "gold"),
    "qt": ((164, 176, 198, 255), (116, 128, 150, 255),
           (198, 210, 230, 255), "gold+gem"),
}


def _mail_rings(d, x0, y0, x1, y1, width_at, dark, light):
    # staggered ring dots suggesting woven mail
    row = 0
    for yy in range(y0, y1, 3):
        off = 2 if row % 2 else 0
        w = width_at(yy)
        for xx in range(16 - w + off, 16 + w, 4):
            if x0 <= xx <= x1:
                d.point((xx, yy), fill=dark)
                d.point((xx + 1, yy), fill=dark)
                d.point((xx, yy + 1), fill=light)
        row += 1


def chainmail(rng, grade, kind, xl):
    base, dark, light, trim = GRADES[grade]
    img = canvas()
    d = ImageDraw.Draw(img)
    m = 2 if xl else 0  # xl pieces are drawn larger
    # cap-helmet attachment band at the top
    d.rectangle([9 - m, 3, 23 + m, 6], fill=dark, outline=OUTLINE)
    d.rectangle([9 - m, 3, 23 + m, 4], fill=light)
    if kind in ("aventail", "face"):
        bottom = 29 if kind == "aventail" else 24
        topw = 7 + m          # half width at the band
        botw = 12 + m if kind == "aventail" else 11 + m

        def width_at(yy, _t=7, _b=bottom, _tw=topw, _bw=botw):
            f = (yy - _t) / max(1, _b - _t)
            return int(_tw + (_bw - _tw) * f)

        d.polygon([(16 - topw, 6), (16 + topw, 6), (16 + botw, bottom),
                   (16 - botw, bottom)], fill=base, outline=OUTLINE)
        _mail_rings(d, 16 - botw, 8, 16 + botw, bottom - 1, width_at,
                    dark, light)
        # scalloped lower edge
        for xx in range(16 - botw + 1, 16 + botw - 1, 3):
            d.point((xx, bottom), fill=dark)
        if kind == "face":
            # face opening punched at the top center
            px = img.load()
            for yy in range(5, 14):
                for xx in range(10, 23):
                    ex = (xx - 16) / 5.5
                    ey = (yy - 9) / 4.5
                    if ex * ex + ey * ey <= 1.0:
                        px[xx, yy] = (0, 0, 0, 0)
            d.arc([10, 5, 22, 14], 0, 360, fill=dark)
        speckle(rng, d, (16 - botw, 8, 16 + botw, bottom),
                [dark, light], 10)
        return img
    # steel facemask: solid plate over the lower face
    x0, x1 = 8 - m, 24 + m
    d.polygon([(x0, 9), (x1, 9), (x1 - 1, 16), (x1 - 3, 25),
               (16, 28), (x0 + 3, 25), (x0 + 1, 16)],
              fill=base, outline=OUTLINE)
    d.polygon([(x0 + 1, 10), (x1 - 1, 10), (x1 - 2, 15), (x0 + 2, 15)],
              fill=light)
    # nose dip at the top center
    d.polygon([(14, 9), (18, 9), (16, 12)], fill=dark)
    # mouth slit + breath holes
    d.rectangle([13, 20, 19, 21], fill=dark)
    for xx in (11, 14, 17, 20):
        for yy in (23, 25):
            d.point((xx, yy), fill=dark)
    if trim == "etch":
        d.arc([11, 12, 21, 20], 200, 340, fill=dark)
    elif trim == "mustache":
        # sculpted mustache above the mouth slit
        d.arc([10, 14, 16, 20], 300, 160, fill=dark, width=2)
        d.arc([16, 14, 22, 20], 20, 240, fill=dark, width=2)
        d.arc([10, 13, 16, 19], 300, 160, fill=light, width=1)
        d.arc([16, 13, 22, 19], 20, 240, fill=light, width=1)
    elif trim in ("gold", "gold+gem"):
        # gold inlay swirls on the cheeks + gold edge trim
        d.arc([9, 13, 15, 19], 90, 330, fill=GOLD, width=1)
        d.arc([17, 13, 23, 19], 210, 90, fill=GOLD, width=1)
        d.line([(x0 + 2, 25), (16, 27), (x1 - 2, 25)], fill=GOLD_D)
        if trim == "gold+gem":
            # gemstone setting at the brow center
            d.ellipse([14, 5, 18, 9], fill=GOLD, outline=OUTLINE)
            d.point((16, 7), fill=GEM)
            d.point((15, 6), fill=(255, 160, 160, 255))
    speckle(rng, d, (x0, 10, x1, 26), [dark, light], 8)
    return img


# ------------------------------------------------------------------

def _chainmail_for(item_id):
    xl = item_id.startswith("xl_")
    rest = item_id[3:] if xl else item_id
    grade = rest.split("_", 1)[0]
    if rest.endswith("chainmail_aventail"):
        kind = "aventail"
    elif rest.endswith("chainmail_face"):
        kind = "face"
    else:
        kind = "facemask"
    return lambda rng: chainmail(rng, grade, kind, xl)


RENDER = {
    "advanced_gpnvg": lambda rng: gpnvg(rng, False),
    "advanced_gpnvg_on": lambda rng: gpnvg(rng, True),
    "attachable_ear_muffs": lambda rng: ear_muffs(rng, False),
    "attachable_ear_muffs_on": lambda rng: ear_muffs(rng, True),
    "ballistic_mandible_guard": lambda rng: mandible_guard(rng, True),
    "counterweight_pouch": counterweight_pouch,
    "enhanced_nvg": lambda rng: enhanced_nvg(rng, False),
    "enhanced_nvg_on": lambda rng: enhanced_nvg(rng, True),
    "face_shield": lambda rng: face_shield(rng, "thermo"),
    "face_shield_eod": lambda rng: face_shield(rng, "eod"),
    "face_shield_plastic": lambda rng: face_shield(rng, "plastic"),
    "insert_vision_correction": vision_insert,
    "kevlar_nape_protector": lambda rng: nape_protector(rng, "kevlar"),
    "mesh_nape_protector": lambda rng: nape_protector(rng, "mesh"),
    "military_nvg": lambda rng: military_nvg(rng, False),
    "military_nvg_on": lambda rng: military_nvg(rng, True),
    "nape_protector": lambda rng: nape_protector(rng, "cyan"),
    "outsert_breach": lambda rng: outsert(rng, "breach"),
    "outsert_clear": lambda rng: outsert(rng, "clear"),
    "outsert_sunlight": lambda rng: outsert(rng, "sunlight"),
    "plastic_chin_guard": chin_guard,
    "plastic_mandible_guard": lambda rng: mandible_guard(rng, False),
    "shield_visor": lambda rng: face_shield(rng, "visor"),
    "sun_shield": sun_shield,
}
for _iid in IDS:
    if "chainmail" in _iid or "steel_facemask" in _iid:
        RENDER[_iid] = _chainmail_for(_iid)


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
    cols, cell = 6, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item16.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item16.png {sheet.size}")


if __name__ == "__main__":
    main()
