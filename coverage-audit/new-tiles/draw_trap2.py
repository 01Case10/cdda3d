#!/usr/bin/env python3
"""Batch TRAP2 (Phase 2g.4 scope extension): 40 trap ids.

Families, from the game data:
  * tr_LIXA_unfold_entry_10..16 -- copy-from tr_LIXA_unfold_entry_1, i.e. the
    same invisible green "#" portal pads as TRAP1's unfold family. Rendered
    with TRAP1's pad renderer copied verbatim (self-contained, no import):
    green portal pad, nested-squares unfold glyph, channel number as lit
    pips in a 2x8 grid beneath the pad.
  * Visible devices drawn as devices: spinning blade (tr_blade), spinning
    blade engine (tr_engine), exposed high-energy conduit (tr_dissector),
    crossbow trap, buried bear trap, booby trap, alien tree roots,
    birchbark funnel, down mattress, bubble wrap, glass shards, monster
    teeth caltrops, burrowed dermatik (burrow mound), dormant corpse
    (a corpse -- drawn as one; it is not dormant), strange open door,
    swirling vortex exit portal, fluid void.
  * Invisible mapgen triggers drawn as subtle floor-level markers:
    overhead lights (tr_light_on_*: pale light pool + lamp dot, variant
    told by edge tick + pip count), lab/res door triggers ("+" door
    glyph with corner/pip variant ticks), drain (grate), glow (pale
    light pool), hum (sound arcs), imperial road go (road chevron),
    glimmering floor (faint gold sparkles + "#" hatch).

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is strictly
{0, 255}: glow/translucency faked with pale opaque tones. Writes ONLY its
own 40 folders + preview_trap2.png. No other files touched.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "traps")
CREDITS = ("Drawn from the trap's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "tr_LIXA_unfold_entry_10", "tr_LIXA_unfold_entry_11",
    "tr_LIXA_unfold_entry_12", "tr_LIXA_unfold_entry_13",
    "tr_LIXA_unfold_entry_14", "tr_LIXA_unfold_entry_15",
    "tr_LIXA_unfold_entry_16",
    "tr_alien_tree_trap_roots", "tr_beartrap_buried", "tr_birchbark_funnel",
    "tr_blade", "tr_boobytrap", "tr_bubblewrap", "tr_caltrops_teeth",
    "tr_concrete_wall_door_lab_res_8_NE",
    "tr_concrete_wall_door_lab_res_8_NW",
    "tr_crossbow", "tr_dermatik", "tr_dissector", "tr_dormant_corpse",
    "tr_down_mattress", "tr_drain", "tr_engine", "tr_glass",
    "tr_glimmer_floor", "tr_glow", "tr_hum", "tr_imperial_road_go",
    "tr_inner_cabins_door_o", "tr_inner_cabins_exit_portal",
    "tr_inner_cabins_void",
    "tr_lab_security_corridor_a", "tr_lab_security_corridor_b",
    "tr_light_on_MUT_SE_2",
    "tr_light_on_lab_security_z0N_A", "tr_light_on_lab_security_z0N_B",
    "tr_light_on_lab_security_z0S_A", "tr_light_on_lab_security_z0S_B",
    "tr_light_on_res_8_NE", "tr_light_on_res_8_NW",
]

# --------------------------------------------------------------------------
# LIXA pad renderer: copied from draw_trap1.py (pattern + palette verbatim)
# so the unfold_entry_10..16 pads match entries 1..9 exactly.
# --------------------------------------------------------------------------
RIM_DARK = (24, 30, 28, 255)
RIM_EDGE = (58, 71, 66, 255)
FACE = (30, 40, 36, 255)
FACE_SPECK = (38, 50, 44, 255)
HATCH = (25, 34, 30, 255)
GROOVE = (13, 38, 22, 255)
CORE_DARK = (10, 30, 17, 255)
GLYPH = (234, 255, 238, 255)
PIP_OFF = (20, 44, 28, 255)

RING_SETS = {
    # (outer pale glow tone, mid band, inner field tone)
    "forward": ((196, 246, 158), (110, 199, 74), (24, 84, 34)),
    "back": ((158, 240, 205), (56, 184, 143), (16, 74, 55)),
    "unfold": ((168, 248, 178), (66, 196, 96), (18, 82, 38)),
}


def draw_pad(family, n, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx, cy, r = 15.5, 13.5, 12.5

    # Pad disc: rim, face.
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=RIM_DARK)
    d.ellipse([cx - r + 1, cy - r + 1, cx + r - 1, cy + r - 1], fill=RIM_EDGE)
    d.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2], fill=FACE)

    # Seeded speckle on the face (deterministic per sprite).
    for _ in range(46):
        a = rng.uniform(0, 2 * math.pi)
        rr = rng.uniform(2.0, r - 3.0)
        x, y = cx + rr * math.cos(a), cy + rr * math.sin(a)
        d.point((x, y), fill=FACE_SPECK)

    # Faint "#" cross-hatch clipped to the face (data symbol is "#").
    hatch = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    hd = ImageDraw.Draw(hatch)
    for gx in range(5, 28, 4):
        hd.line([(gx, 2), (gx, 25)], fill=HATCH)
    for gy in range(4, 26, 4):
        hd.line([(3, gy), (28, gy)], fill=HATCH)
    mask = Image.new("L", (32, 32), 0)
    md = ImageDraw.Draw(mask)
    md.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2], fill=255)
    img.paste(hatch, (0, 0), mask)
    d = ImageDraw.Draw(img)

    # Portal rings: pale glow tone, mid band, groove, glowing core field.
    pale, band, field = RING_SETS[family]
    d.ellipse([cx - 9.5, cy - 9.5, cx + 9.5, cy + 9.5], fill=pale + (255,))
    # Seeded dash gap in the pale ring so siblings are not pixel-clones.
    gap_a = rng.uniform(0, 360)
    d.arc([cx - 9.5, cy - 9.5, cx + 9.5, cy + 9.5],
          start=gap_a, end=gap_a + 26, fill=band + (255,), width=2)
    d.ellipse([cx - 8.0, cy - 8.0, cx + 8.0, cy + 8.0], fill=band + (255,))
    d.ellipse([cx - 6.4, cy - 6.4, cx + 6.4, cy + 6.4], fill=GROOVE)
    d.ellipse([cx - 5.2, cy - 5.2, cx + 5.2, cy + 5.2], fill=field + (255,))
    d.ellipse([cx - 3.4, cy - 3.4, cx + 3.4, cy + 3.4], fill=CORE_DARK)

    # Family glyph in the core.
    if family == "forward":
        for dy in (2.6, -1.4):
            d.polygon([(cx - 3.4, cy + dy + 1.6), (cx, cy + dy - 2.0),
                       (cx + 3.4, cy + dy + 1.6), (cx + 3.4, cy + dy + 3.1),
                       (cx, cy + dy - 0.5), (cx - 3.4, cy + dy + 3.1)],
                      fill=GLYPH)
    elif family == "back":
        for dy in (-2.6, 1.4):
            d.polygon([(cx - 3.4, cy + dy - 1.6), (cx, cy + dy + 2.0),
                       (cx + 3.4, cy + dy - 1.6), (cx + 3.4, cy + dy - 3.1),
                       (cx, cy + dy + 0.5), (cx - 3.4, cy + dy - 3.1)],
                      fill=GLYPH)
    else:  # unfold: nested squares opening outward + entry dot
        d.rectangle([cx - 4.4, cy - 4.4, cx + 4.4, cy + 4.4],
                    outline=GLYPH, width=1)
        for sx, sy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
            d.line([(cx + sx * 4.4, cy + sy * 4.4),
                    (cx + sx * 2.2, cy + sy * 4.4)], fill=GLYPH)
            d.line([(cx + sx * 4.4, cy + sy * 4.4),
                    (cx + sx * 4.4, cy + sy * 2.2)], fill=GLYPH)
        d.rectangle([cx - 2.0, cy - 2.0, cx + 2.0, cy + 2.0],
                    outline=GLYPH, width=1)
        d.ellipse([cx - 0.7, cy - 0.7, cx + 0.7, cy + 0.7], fill=GLYPH)

    # Channel index pips: 2 rows x 8 under the pad, first N lit.
    lit = pale + (255,)
    for i in range(16):
        col, row = i % 8, i // 8
        x0 = 4 + col * 3
        y0 = 27 + row * 3
        d.rectangle([x0, y0, x0 + 1, y0 + 1],
                    fill=lit if i < n else PIP_OFF)
    return img


# --------------------------------------------------------------------------
# Helpers for the device / marker renderers.
# --------------------------------------------------------------------------
def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def sparkle(d, cx, cy, s, color):
    """Four-point star sparkle."""
    d.polygon([(cx, cy - s), (cx + s * 0.28, cy - s * 0.28),
               (cx + s, cy), (cx + s * 0.28, cy + s * 0.28),
               (cx, cy + s), (cx - s * 0.28, cy + s * 0.28),
               (cx - s, cy), (cx - s * 0.28, cy - s * 0.28)], fill=color)


def taper(d, x0, y0, w0, x1, y1, w1, fill):
    """Tapered limb/root from (x0,y0) width w0 to (x1,y1) width w1."""
    dx, dy = x1 - x0, y1 - y0
    ln = math.hypot(dx, dy) or 1.0
    px, py = -dy / ln, dx / ln
    d.polygon([(x0 + px * w0 / 2, y0 + py * w0 / 2),
               (x1 + px * w1 / 2, y1 + py * w1 / 2),
               (x1 - px * w1 / 2, y1 - py * w1 / 2),
               (x0 - px * w0 / 2, y0 - py * w0 / 2)], fill=fill)


def saw_disc(d, cx, cy, r, body, tooth, hub):
    """Toothed circular blade centered at (cx, cy)."""
    for k in range(12):
        a = math.radians(k * 30)
        a1 = math.radians(k * 30 - 8)
        a2 = math.radians(k * 30 + 8)
        d.polygon([(cx + r * math.cos(a1), cy + r * math.sin(a1)),
                   (cx + (r + 2.2) * math.cos(a), cy + (r + 2.2) * math.sin(a)),
                   (cx + r * math.cos(a2), cy + r * math.sin(a2))], fill=tooth)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=body)
    d.ellipse([cx - r * 0.38, cy - r * 0.38, cx + r * 0.38, cy + r * 0.38],
              fill=hub)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(52, 54, 58, 255))


# --------------------------------------------------------------------------
# Visible devices.
# --------------------------------------------------------------------------
def r_alien_roots(rng):
    img, d = canvas()
    dark = (46, 46, 52, 255)
    body = (76, 76, 84, 255)
    lite = (104, 104, 112, 255)
    ends = [(2, 25), (8, 30), (15, 28), (22, 30), (30, 24), (28, 15)]
    for ex, ey in ends:
        taper(d, 16, 1, 4.6, ex, ey, 1.4, dark)
        taper(d, 16, 1, 3.2, ex, ey, 0.9, body)
    # Rootlets branching off the mains.
    for (x0, y0, x1, y1) in [(11, 12, 5, 17), (20, 13, 26, 10),
                             (14, 19, 9, 24), (19, 20, 25, 25)]:
        taper(d, x0, y0, 1.6, x1, y1, 0.6, body)
    # Pale alien nodules at the joints.
    for (nx, ny) in [(13, 9), (19, 16), (11, 21)]:
        d.ellipse([nx - 1.6, ny - 1.6, nx + 1.6, ny + 1.6],
                  fill=(150, 148, 138, 255))
        d.point((nx, ny), fill=lite)
    return img


def r_beartrap_buried(rng):
    img, d = canvas()
    dirt = (110, 84, 58, 255)
    dirt_d = (88, 64, 42, 255)
    dirt_l = (134, 106, 74, 255)
    steel = (108, 128, 152, 255)
    steel_l = (178, 192, 208, 255)
    steel_d = (70, 86, 106, 255)
    # Dirt mound the trap is buried in.
    d.ellipse([1, 13, 31, 30], fill=dirt)
    for _ in range(26):
        x = rng.uniform(3, 29)
        y = rng.uniform(15, 29)
        d.point((x, y), fill=dirt_d if rng.random() < 0.5 else dirt_l)
    # Base plate ring.
    d.ellipse([6, 8, 26, 24], fill=steel_d)
    d.ellipse([7, 9, 25, 23], fill=steel)
    # Open jaws: toothed arcs top and bottom.
    for k in range(7):
        a = math.radians(200 + k * 23)
        x, y = 16 + 8.2 * math.cos(a), 16 + 6.4 * math.sin(a)
        d.polygon([(x - 1.4, y + 1.6), (x + 1.4, y + 1.6), (x, y - 1.8)],
                  fill=steel_l)
        a = math.radians(20 + k * 23)
        x, y = 16 + 8.2 * math.cos(a), 16 + 6.4 * math.sin(a)
        d.polygon([(x - 1.4, y - 1.6), (x + 1.4, y - 1.6), (x, y + 1.8)],
                  fill=steel_l)
    # Trigger plate + spring housings.
    d.rectangle([13, 13, 19, 19], fill=steel_l)
    d.ellipse([14.8, 14.8, 17.2, 17.2], fill=steel_d)
    d.rectangle([5, 14, 8, 18], fill=steel_d)
    d.rectangle([24, 14, 27, 18], fill=steel_d)
    # Dirt clumps slumped over the rim.
    for (x, y, r) in [(8, 21, 2.6), (23, 22, 2.2), (16, 23, 1.8)]:
        d.ellipse([x - r, y - r, x + r, y + r], fill=dirt_d)
    return img


def r_birchbark_funnel(rng):
    img, d = canvas()
    bark = (233, 229, 219, 255)
    bark_sh = (213, 208, 198, 255)
    band = (188, 183, 172, 255)
    lent = (34, 30, 26, 255)
    d.ellipse([6, 25, 26, 30], fill=(42, 40, 36, 255))  # ground shadow
    # Cone body.
    d.polygon([(6, 8), (26, 8), (18, 26), (14, 26)], fill=bark)
    d.polygon([(16, 8), (26, 8), (18, 26), (16, 26)], fill=bark_sh)
    for y in (13, 18, 23):
        d.line([(7 + (y - 8) * 0.35, y), (25 - (y - 8) * 0.35, y)], fill=band)
    # Birch lenticel dashes.
    for _ in range(9):
        x = rng.uniform(9, 22)
        y = rng.uniform(11, 23)
        w = rng.uniform(1.5, 3.2)
        d.line([(x, y), (x + w, y)], fill=lent)
    # Rim + dark throat.
    d.ellipse([6, 5, 26, 11], fill=(203, 197, 186, 255))
    d.ellipse([9.5, 6.3, 22.5, 9.7], fill=(88, 78, 66, 255))
    return img


def r_blade(rng):
    img, d = canvas()
    d.rectangle([11, 23, 21, 29], fill=(58, 60, 66, 255))  # mount base
    d.rectangle([11, 23, 21, 25], fill=(84, 88, 96, 255))
    saw_disc(d, 16, 13, 9.5, (186, 196, 206, 255), (148, 160, 172, 255),
             (112, 120, 128, 255))
    d.ellipse([9, 6, 23, 20], outline=(206, 216, 224, 255))
    # Cyan sheen + spin motion arcs (data color cyan).
    d.arc([7, 4, 25, 22], start=195, end=315, fill=(96, 208, 218, 255),
          width=2)
    d.arc([3, 1, 29, 27], start=295, end=345, fill=(168, 232, 238, 255),
          width=2)
    d.arc([3, 1, 29, 27], start=115, end=160, fill=(168, 232, 238, 255),
          width=2)
    return img


def r_engine(rng):
    img, d = canvas()
    d.ellipse([5, 25, 27, 30], fill=(40, 40, 44, 255))  # shadow
    # Blade disc up top, red accent arc (data color light_red).
    saw_disc(d, 16, 10, 7.6, (180, 190, 198, 255), (142, 152, 162, 255),
             (104, 112, 120, 255))
    d.arc([8, 2, 24, 18], start=180, end=300, fill=(228, 108, 98, 255),
          width=2)
    # Motor block with cooling fins.
    d.rectangle([9, 15, 23, 28], fill=(62, 64, 70, 255))
    d.rectangle([9, 15, 23, 17], fill=(78, 80, 88, 255))
    for x in (12, 15, 18, 21):
        d.line([(x, 19), (x, 27)], fill=(88, 91, 99, 255))
    d.rectangle([9, 21, 23, 23], fill=(206, 88, 80, 255))  # red stripe
    # Pull cord + handle.
    d.arc([19, 17, 30, 29], start=270, end=60, fill=(198, 198, 192, 255))
    d.ellipse([26.5, 25.5, 29.5, 28.5], fill=(150, 60, 52, 255))
    return img


def r_dissector(rng):
    img, d = canvas()
    conduit = (70, 74, 82, 255)
    conduit_e = (102, 108, 118, 255)
    clamp = (52, 55, 61, 255)
    # Two conduit segments with a broken gap in the middle.
    d.line([(2, 25), (12, 18)], fill=conduit, width=6)
    d.line([(20, 13), (30, 6)], fill=conduit, width=6)
    d.line([(2, 23), (12, 16)], fill=conduit_e, width=1)
    d.line([(20, 11), (30, 4)], fill=conduit_e, width=1)
    # Clamps.
    d.line([(5, 26), (8, 20)], fill=clamp, width=3)
    d.line([(24, 12), (27, 6)], fill=clamp, width=3)
    # Exposed glowing core in the gap.
    d.line([(12, 18), (20, 13)], fill=(110, 235, 245, 255), width=3)
    d.line([(12, 18), (20, 13)], fill=(224, 255, 255, 255), width=1)
    # Arcing sparks (zigzag echoes the "7" symbol).
    arc_c = (172, 242, 248, 255)
    d.line([(15, 14), (11, 10), (14, 8), (10, 3)], fill=arc_c)
    d.line([(18, 16), (22, 12), (20, 9), (24, 5)], fill=arc_c)
    d.line([(16, 17), (15, 22), (18, 24), (16, 28)], fill=arc_c)
    for _ in range(7):
        x = rng.uniform(8, 24)
        y = rng.uniform(2, 28)
        d.point((x, y), fill=(240, 255, 255, 255))
    return img


def r_crossbow(rng):
    img, d = canvas()
    wood = (122, 86, 50, 255)
    wood_d = (94, 64, 36, 255)
    # Bow limbs across the top.
    d.arc([2, 1, 30, 17], start=180, end=360, fill=wood_d, width=4)
    d.arc([2, 1, 30, 17], start=180, end=360, fill=wood, width=2)
    # String to the notch.
    d.line([(3.5, 9), (16, 11), (28.5, 9)], fill=(212, 206, 190, 255))
    # Stock.
    d.polygon([(14, 8), (18, 8), (17.2, 27), (14.8, 27)], fill=wood)
    d.line([(16, 9), (16, 26)], fill=wood_d)
    # Green cord wraps (data color green).
    d.rectangle([14, 14, 18, 16], fill=(66, 148, 66, 255))
    d.rectangle([14, 20, 18, 22], fill=(66, 148, 66, 255))
    # Loaded steel bolt.
    d.line([(16, 3), (16, 12)], fill=(172, 177, 182, 255), width=2)
    d.polygon([(14.4, 5), (17.6, 5), (16, 1.6)], fill=(205, 210, 215, 255))
    # Tripwire trailing off.
    d.line([(16, 27), (27, 31)], fill=(198, 198, 190, 255))
    d.point((27, 31), fill=(150, 150, 145, 255))
    return img


def r_boobytrap(rng):
    img, d = canvas()
    wood = (130, 95, 60, 255)
    wood_d = (102, 72, 44, 255)
    # Base board.
    d.polygon([(4, 24), (28, 24), (26, 29), (6, 29)], fill=wood)
    d.line([(6, 26.5), (26, 26.5)], fill=wood_d)
    # Pipe clamp holding the shell.
    d.rectangle([13, 10, 19, 24], fill=(88, 88, 94, 255))
    d.rectangle([13, 16, 19, 18], fill=(64, 64, 70, 255))
    # Shotgun shell: brass base, red body.
    d.rectangle([13.5, 9, 18.5, 12], fill=(206, 176, 86, 255))
    d.rectangle([13.5, 3, 18.5, 9], fill=(186, 46, 38, 255))
    d.rectangle([13.5, 3, 18.5, 4.5], fill=(160, 36, 30, 255))
    # Tripwire strung both ways at ankle height.
    wire = (204, 204, 194, 255)
    d.line([(14, 21), (2, 7)], fill=wire)
    d.line([(18, 21), (30, 9)], fill=wire)
    d.point((2, 7), fill=(150, 150, 145, 255))
    d.point((30, 9), fill=(150, 150, 145, 255))
    return img


def r_bubblewrap(rng):
    img, d = canvas()
    sheet = (166, 218, 226, 255)
    sheet_d = (146, 202, 212, 255)
    ring = (232, 248, 250, 255)
    popped = (138, 190, 200, 255)
    d.rounded_rectangle([3, 7, 29, 26], radius=3, fill=sheet)
    d.rounded_rectangle([3, 21, 29, 26], radius=3, fill=sheet_d)
    d.rounded_rectangle([3, 7, 29, 26], radius=3, outline=ring)
    pops = {(1, 0), (3, 2), (4, 1)}
    for row in range(3):
        for col in range(5):
            cx, cy = 7 + col * 4.6, 11 + row * 4.6
            if (col, row) in pops:
                d.ellipse([cx - 1.8, cy - 1.8, cx + 1.8, cy + 1.8], fill=popped)
            else:
                d.ellipse([cx - 1.9, cy - 1.9, cx + 1.9, cy + 1.9],
                          outline=ring)
                d.point((cx - 0.7, cy - 0.7), fill=(255, 255, 255, 255))
    # Lifted fold corner.
    d.polygon([(29, 7), (24, 7), (29, 12)], fill=(198, 238, 242, 255))
    return img


def r_caltrops_teeth(rng):
    img, d = canvas()
    ivory = (228, 222, 198, 255)
    ivory_d = (168, 160, 132, 255)
    base = (96, 94, 90, 255)
    for _ in range(10):
        x = rng.uniform(4, 28)
        y = rng.uniform(6, 28)
        a = rng.uniform(0, 2 * math.pi)
        s = rng.uniform(2.6, 4.0)
        tip = (x + s * math.cos(a), y + s * math.sin(a))
        px, py = -math.sin(a), math.cos(a)
        b1 = (x + 1.6 * px, y + 1.6 * py)
        b2 = (x - 1.6 * px, y - 1.6 * py)
        d.polygon([b1, tip, b2], fill=ivory)
        d.line([b1, b2], fill=base, width=1)
        d.point((x, y), fill=ivory_d)
    return img


def r_glass(rng):
    img, d = canvas()
    fill_c = (168, 226, 234, 255)
    edge = (104, 186, 198, 255)
    hi = (240, 255, 255, 255)
    for _ in range(7):
        x = rng.uniform(5, 25)
        y = rng.uniform(6, 26)
        pts = []
        for k in range(3):
            a = rng.uniform(0, 2 * math.pi)
            rr = rng.uniform(2.0, 4.2)
            pts.append((x + rr * math.cos(a), y + rr * math.sin(a)))
        d.polygon(pts, fill=fill_c, outline=edge)
        d.line([pts[0], pts[1]], fill=hi)
    for _ in range(5):
        d.point((rng.uniform(3, 29), rng.uniform(4, 29)), fill=hi)
    return img


def r_dermatik(rng):
    img, d = canvas()
    mound = (104, 87, 65, 255)
    mound_d = (82, 67, 48, 255)
    d.ellipse([5, 15, 27, 28], fill=mound)
    for _ in range(18):
        x = rng.uniform(7, 25)
        y = rng.uniform(17, 27)
        d.point((x, y), fill=mound_d)
    d.arc([5, 13, 27, 24], start=180, end=330, fill=(142, 122, 92, 255),
          width=2)
    # Burrow hole.
    d.ellipse([11, 14, 21, 22], fill=(16, 13, 10, 255))
    d.ellipse([13, 16, 19, 20], fill=(8, 6, 5, 255))
    # The burrowed wasp barely shows: amber/black banded abdomen tip.
    for i, c in enumerate([(206, 164, 44, 255), (38, 33, 28, 255),
                           (206, 164, 44, 255)]):
        d.rectangle([14.6 + i * 1.1, 15.2, 15.5 + i * 1.1, 17.4], fill=c)
    # Faint green speckle (data color green).
    for _ in range(5):
        d.point((rng.uniform(7, 25), rng.uniform(18, 27)),
                fill=(92, 142, 82, 255))
    return img


def r_dormant_corpse(rng):
    img, d = canvas()
    skin = (152, 216, 142, 255)      # light_green pallor
    skin_d = (128, 190, 118, 255)
    shirt = (94, 110, 104, 255)
    pants = (68, 70, 79, 255)
    d.ellipse([1, 9, 31, 25], fill=(36, 38, 36, 255))  # shadow
    # Legs stretched to the right, shoes at the ends.
    d.polygon([(15, 14), (29, 12.6), (29, 16), (15, 17)], fill=pants)
    d.polygon([(15, 17.4), (29, 16.6), (29, 20), (15, 20.6)], fill=pants)
    d.rectangle([28.4, 12.2, 31, 16], fill=(28, 28, 30, 255))
    d.rectangle([28.4, 16.4, 31, 20.2], fill=(28, 28, 30, 255))
    # Torso in a torn shirt.
    d.polygon([(6, 12.6), (16, 11.8), (16, 20.8), (6, 21.4)], fill=shirt)
    d.polygon([(10, 15), (13.4, 14.4), (12, 18.6)], fill=skin_d)  # tear
    # Sprawled arms.
    taper(d, 8, 12.6, 3.0, 2.6, 6.4, 1.8, skin)
    taper(d, 12, 20.6, 2.8, 17.5, 25.6, 1.7, skin)
    # Head: hair cap, shut eye, slack mouth.
    d.ellipse([1.6, 11.8, 9.4, 20.2], fill=skin)
    d.arc([1.6, 10.6, 9.4, 18], start=90, end=250, fill=(58, 52, 42, 255),
          width=3)
    d.line([(5.4, 15.4), (7.6, 15.4)], fill=(40, 48, 38, 255))
    d.line([(5.8, 18.2), (7.4, 18.5)], fill=(96, 120, 88, 255))
    return img


def r_down_mattress(rng):
    img, d = canvas()
    d.rounded_rectangle([5, 6, 30, 28], radius=4, fill=(148, 146, 138, 255))
    d.rounded_rectangle([3, 4, 28, 26], radius=4, fill=(236, 233, 222, 255))
    # Quilt stitching clipped to the mattress top.
    layer = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    for k in range(-3, 6):
        ld.line([(3 + k * 6, 4), (3 + k * 6 + 11, 26)],
                fill=(203, 199, 188, 255))
        ld.line([(3 + k * 6, 26), (3 + k * 6 + 11, 4)],
                fill=(203, 199, 188, 255))
    mask = Image.new("L", (32, 32), 0)
    ImageDraw.Draw(mask).rounded_rectangle([3, 4, 28, 26], radius=4,
                                           fill=255)
    img.paste(layer, (0, 0), mask)
    d = ImageDraw.Draw(img)
    # Side band + piping + tuft buttons.
    d.rounded_rectangle([3, 21, 28, 26], radius=4, fill=(216, 212, 200, 255))
    d.rounded_rectangle([3, 4, 28, 26], radius=4,
                        outline=(206, 202, 191, 255))
    for bx in (9, 15.5, 22):
        for by in (9.5, 15):
            d.ellipse([bx - 1, by - 1, bx + 1, by + 1],
                      fill=(190, 186, 174, 255))
    return img


def r_inner_cabins_door(rng):
    img, d = canvas()
    frame = (96, 66, 38, 255)
    wood = (142, 96, 55, 255)
    wood_d = (112, 74, 40, 255)
    # The dark opening, with an uncanny magenta sheen.
    d.rectangle([10, 4, 22, 27], fill=(11, 9, 15, 255))
    d.line([(11, 24), (21, 8)], fill=(110, 55, 100, 255))
    d.rectangle([8, 2, 24, 27], outline=frame, width=2)
    # Door panel swung open toward the viewer.
    d.polygon([(10, 4), (20.5, 8), (20.5, 28.5), (10, 24.5)], fill=wood)
    for gx in (13.5, 17):
        d.line([(gx, 6.6 + (gx - 10) * 0.38), (gx, 26.6 + (gx - 10) * 0.38)],
               fill=wood_d)
    d.line([(10, 14.5), (20.5, 18.5)], fill=wood_d)
    d.ellipse([17.8, 17.4, 19.8, 19.4], fill=(212, 176, 92, 255))  # knob
    return img


def r_inner_cabins_portal(rng):
    img, d = canvas()
    mag = (216, 70, 200, 255)
    mag_l = (238, 130, 224, 255)
    pur = (198, 92, 228, 255)
    d.ellipse([3.5, 3.5, 28.5, 28.5], outline=mag, width=3)
    d.arc([7, 7, 25, 25], start=20, end=250, fill=mag_l, width=2)
    d.arc([7, 7, 25, 25], start=200, end=430, fill=pur, width=2)
    d.arc([10, 10, 22, 22], start=140, end=380, fill=mag_l, width=2)
    d.arc([10, 10, 22, 22], start=320, end=480, fill=mag, width=2)
    d.ellipse([12.6, 12.6, 19.4, 19.4], fill=(250, 222, 250, 255))
    d.ellipse([14.8, 14.8, 17.2, 17.2], fill=(255, 255, 255, 255))
    for (x, y) in [(5, 10), (26, 20), (9, 26)]:
        d.point((x, y), fill=mag_l)
    return img


def r_inner_cabins_void(rng):
    img, d = canvas()
    void = (11, 11, 16, 255)
    rim = (26, 26, 36, 255)
    d.ellipse([3, 8, 29, 26], fill=rim)
    d.ellipse([8, 5, 21, 15], fill=rim)
    d.ellipse([4, 9, 28, 25], fill=void)
    d.ellipse([9, 6, 20, 14], fill=void)
    # Fluid "~" wave lines (data symbol).
    for wy, col in [(13, (74, 79, 99, 255)), (17.5, (96, 102, 124, 255)),
                    (22, (74, 79, 99, 255))]:
        pts = [(6 + t, wy + 1.4 * math.sin(t / 2.6)) for t in range(21)]
        d.line(pts, fill=col)
    d.arc([7, 8, 17, 14], start=180, end=280, fill=(126, 132, 150, 255))
    return img


# --------------------------------------------------------------------------
# Subtle floor-level markers for invisible mapgen triggers.
# --------------------------------------------------------------------------
def door_marker(rng, corner=None, pips=0, security=False):
    """Faint '+' door glyph (data symbol "+") with variant ticks."""
    img, d = canvas()
    jamb = (168, 168, 171, 255)
    cross = (150, 150, 153, 255)
    tick = (188, 188, 190, 255)
    pip_c = (150, 150, 153, 255)
    for _ in range(9):  # concrete speckle
        d.point((rng.uniform(5, 27), rng.uniform(5, 24)),
                fill=(132, 132, 135, 255))
    d.line([(6, 4), (6, 24)], fill=jamb, width=2)
    d.line([(26, 4), (26, 24)], fill=jamb, width=2)
    d.line([(6, 4), (26, 4)], fill=jamb, width=2)
    d.line([(5, 25), (27, 25)], fill=cross)
    d.line([(16, 10), (16, 19)], fill=cross, width=2)
    d.line([(11.5, 14.5), (20.5, 14.5)], fill=cross, width=2)
    if corner == "NE":
        d.rectangle([24, 1.5, 29, 6.5], fill=tick)
    elif corner == "NW":
        d.rectangle([3, 1.5, 8, 6.5], fill=tick)
    if security:
        d.ellipse([14.8, 5.2, 17.2, 7.6], fill=(198, 82, 72, 255))
    for i in range(pips):
        x = 13 + i * 5 - (pips - 1) * 2.5
        d.rectangle([x, 27, x + 3, 30], fill=pip_c)
    return img


def light_marker(tick_pos, pips):
    """Overhead-light trigger: pale pool of light, lamp dot, variant ticks."""
    img, d = canvas()
    d.ellipse([5, 7, 27, 25], fill=(224, 219, 184, 255))
    d.ellipse([5, 7, 27, 25], outline=(203, 198, 163, 255))
    d.ellipse([9, 10, 23, 22], fill=(240, 236, 205, 255))
    d.ellipse([14, 14, 18, 18], fill=(252, 250, 236, 255))
    d.point((16, 16), fill=(255, 255, 255, 255))
    tick = (124, 124, 117, 255)
    boxes = {"N": [14, 0.8, 18, 3.8], "S": [14, 28.2, 18, 31.2],
             "NE": [26.2, 0.8, 30.2, 4.8], "NW": [1.8, 0.8, 5.8, 4.8],
             "SE": [26.2, 27.2, 30.2, 31.2]}
    d.rectangle(boxes[tick_pos], fill=tick)
    # Pips sit just inside the ticked edge.
    for i in range(pips):
        if tick_pos == "N":
            x = 12.5 + i * 4.5 - (pips - 1) * 2.2
            d.rectangle([x, 5, x + 2, 7], fill=tick)
        elif tick_pos == "S":
            x = 12.5 + i * 4.5 - (pips - 1) * 2.2
            d.rectangle([x, 25, x + 2, 27], fill=tick)
        elif tick_pos == "SE":
            x = 21.5 + i * 3.4
            d.rectangle([x, 27.2, x + 2, 29.2], fill=tick)
    return img


def r_drain(rng):
    img, d = canvas()
    ring = (128, 130, 134, 255)
    dark = (50, 52, 56, 255)
    bar = (118, 120, 124, 255)
    d.ellipse([6, 6, 26, 26], fill=ring)
    d.ellipse([8.4, 8.4, 23.6, 23.6], fill=dark)
    layer = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    for x in (12, 16, 20):
        ld.line([(x, 8), (x, 24)], fill=bar, width=2)
    mask = Image.new("L", (32, 32), 0)
    ImageDraw.Draw(mask).ellipse([8.4, 8.4, 23.6, 23.6], fill=255)
    img.paste(layer, (0, 0), mask)
    d = ImageDraw.Draw(img)
    d.ellipse([14.8, 14.8, 17.2, 17.2], fill=(148, 150, 154, 255))
    return img


def r_glow(rng):
    img, d = canvas()
    d.ellipse([6.5, 6.5, 25.5, 25.5], fill=(231, 231, 213, 255))
    d.ellipse([9.5, 9.5, 22.5, 22.5], fill=(243, 243, 229, 255))
    d.ellipse([12.5, 12.5, 19.5, 19.5], fill=(252, 252, 245, 255))
    d.ellipse([14.8, 14.8, 17.2, 17.2], fill=(255, 255, 255, 255))
    sparkle(d, 8, 9, 2.2, (240, 240, 220, 255))
    sparkle(d, 24, 22, 1.8, (240, 240, 220, 255))
    return img


def r_hum(rng):
    img, d = canvas()
    c1 = (216, 216, 211, 255)
    c2 = (192, 192, 188, 255)
    for r, col in [(13.5, c2), (9.5, c1), (5.8, c2)]:
        d.arc([16 - r, 19 - r, 16 + r, 19 + r], start=185, end=355, fill=col,
              width=2)
    d.ellipse([14, 17, 18, 21], fill=(230, 230, 226, 255))
    d.line([(9, 22), (7, 25)], fill=c2)
    d.line([(23, 22), (25, 25)], fill=c2)
    return img


def r_glimmer(rng):
    img, d = canvas()
    hatch_c = (196, 176, 92, 255)
    for gx in range(6, 28, 6):  # faint "#" hatch (data symbol)
        d.line([(gx, 5), (gx, 27)], fill=hatch_c)
    for gy in range(6, 28, 6):
        d.line([(5, gy), (27, gy)], fill=hatch_c)
    gold = (242, 216, 112, 255)
    lite = (255, 242, 176, 255)
    for (x, y, s) in [(10, 10, 3.0), (21, 8, 2.2), (16, 17, 3.6),
                      (25, 20, 2.4), (9, 23, 2.0), (22, 26, 1.8)]:
        sparkle(d, x, y, s, gold)
        d.point((x, y), fill=lite)
    return img


def r_imperial_road(rng):
    img, d = canvas()
    road = (52, 52, 57, 255)
    edge = (92, 92, 97, 255)
    d.rectangle([0, 10, 32, 22], fill=road)
    d.line([(0, 10), (32, 10)], fill=edge)
    d.line([(0, 22), (32, 22)], fill=edge)
    for _ in range(16):
        d.point((rng.uniform(1, 31), rng.uniform(11, 21)),
                fill=(70, 70, 76, 255))
    # Pale "go" chevron pointing along the road.
    chev = (226, 226, 216, 255)
    d.polygon([(10, 11), (19, 16), (10, 21), (13, 21), (22, 16), (13, 11)],
              fill=chev)
    return img


RENDERERS = {
    "tr_alien_tree_trap_roots": r_alien_roots,
    "tr_beartrap_buried": r_beartrap_buried,
    "tr_birchbark_funnel": r_birchbark_funnel,
    "tr_blade": r_blade,
    "tr_boobytrap": r_boobytrap,
    "tr_bubblewrap": r_bubblewrap,
    "tr_caltrops_teeth": r_caltrops_teeth,
    "tr_concrete_wall_door_lab_res_8_NE":
        lambda rng: door_marker(rng, corner="NE"),
    "tr_concrete_wall_door_lab_res_8_NW":
        lambda rng: door_marker(rng, corner="NW"),
    "tr_crossbow": r_crossbow,
    "tr_dermatik": r_dermatik,
    "tr_dissector": r_dissector,
    "tr_dormant_corpse": r_dormant_corpse,
    "tr_down_mattress": r_down_mattress,
    "tr_drain": r_drain,
    "tr_engine": r_engine,
    "tr_glass": r_glass,
    "tr_glimmer_floor": r_glimmer,
    "tr_glow": r_glow,
    "tr_hum": r_hum,
    "tr_imperial_road_go": r_imperial_road,
    "tr_inner_cabins_door_o": r_inner_cabins_door,
    "tr_inner_cabins_exit_portal": r_inner_cabins_portal,
    "tr_inner_cabins_void": r_inner_cabins_void,
    "tr_lab_security_corridor_a":
        lambda rng: door_marker(rng, pips=1, security=True),
    "tr_lab_security_corridor_b":
        lambda rng: door_marker(rng, pips=2, security=True),
    "tr_light_on_MUT_SE_2": lambda rng: light_marker("SE", 2),
    "tr_light_on_lab_security_z0N_A": lambda rng: light_marker("N", 1),
    "tr_light_on_lab_security_z0N_B": lambda rng: light_marker("N", 2),
    "tr_light_on_lab_security_z0S_A": lambda rng: light_marker("S", 1),
    "tr_light_on_lab_security_z0S_B": lambda rng: light_marker("S", 2),
    "tr_light_on_res_8_NE": lambda rng: light_marker("NE", 0),
    "tr_light_on_res_8_NW": lambda rng: light_marker("NW", 0),
}


def render(tid, rng):
    if tid.startswith("tr_LIXA_unfold_entry_"):
        n = int(tid.rsplit("_", 1)[1])
        return draw_pad("unfold", n, rng)
    return RENDERERS[tid](rng)


def main():
    assert len(IDS) == 40, len(IDS)
    for tid in IDS:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = render(tid, rng)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (tid, alphas)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", tid)

    # Contact sheet: 8 cols x 5 rows in assignment order, sprites at 2x.
    cell, gap, cols = 64, 3, 8
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, tid in enumerate(IDS):
        img = Image.open(os.path.join(BASE, tid, tid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_trap2.png")
    sheet.save(out)
    print("wrote preview_trap2.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
