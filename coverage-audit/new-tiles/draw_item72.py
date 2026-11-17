#!/usr/bin/env python3
"""ITEM72: ropes/cordage, cow + horse barding, migo plates and
artifact oddities -- CDDA Voxel coverage audit.

Families:

  Ropes and cordage (11) -- coils sized by length (6 in lace is a
    small hank, 30 ft rope a big coil).  Natural cordage/rope is tan
    dry-plant fibre; leather lace is brown; makeshift rope is rougher
    and grayer; the seat belt is a flat gray nylon webbing roll with
    a steel buckle; the fibrous stalk is a green stick, not a coil;
    vines are green coils with leaves.

  Animal barding (20) -- a draped torso panel: cow barding is broad
    and short, horse barding narrower and taller with a raised neck
    (criniere) at the front.  Ten materials, each with its finish:
    biosilicified (acid) chitin = glossy green-teal shell segments,
    chitin = dark red-brown segments, bronze/iron/steel = plate rows
    with rivets over leather, chainmail = ring mesh, kevlar peto =
    quilted mattress cloth, leather = boiled-leather caparison,
    leatherbone = leather with white bone trophies, rubber = a thin
    glossy rain sheet.

  Migo plates (8) -- ballistic plates forced from otherworldly
    carapace: iridescent sheen with a scarred glyph; naked = scarred
    flesh, overgrown = bulky with sharp jutting shards, undergrown =
    small and pale.  Side plates are the same language, smaller.

  Misc (12) -- the sealed samurai mask (dark monolith, horn-like
    filters, spiral-inscribed lenses), a feeding kit (syringes and
    an aerosol sprayer in a case), and ten one-of-a-kind artifacts
    drawn as themselves (Borromean rings, athame, duocylinder,
    black-wood harp, irrational wire cube, Klein flask, Penrose
    triangle, amber ring with spiral worm, blue-sun spear,
    tesseract).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "cordage_36",
    "cordage_36_leather",
    "cordage_6",
    "cordage_6_leather",
    "rope_30",
    "rope_makeshift_30",
    "rope_makeshift_6",
    "seatbelt",
    "stick_fiber",
    "vine_30",
    "vine_6",
    "acidchitin_armor_cow",
    "bronze_armor_cow",
    "chainmail_armor_cow",
    "chitin_armor_cow",
    "iron_armor_cow",
    "kevlar_armor_cow",
    "leather_armor_cow",
    "leatherbone_armor_cow",
    "rubber_armor_cow",
    "steel_armor_cow",
    "acidchitin_armor_horse",
    "bronze_armor_horse",
    "chainmail_armor_horse",
    "chitin_armor_horse",
    "iron_armor_horse",
    "kevlar_armor_horse",
    "leather_armor_horse",
    "leatherbone_armor_horse",
    "rubber_armor_horse",
    "steel_armor_horse",
    "armor_star_samurai_mask",
    "feeding_kit",
    "migo_plate",
    "migo_plate_naked",
    "migo_plate_overgrown",
    "migo_plate_undergrown",
    "migo_sideplate",
    "migo_sideplate_naked",
    "migo_sideplate_overgrown",
    "migo_sideplate_undergrown",
    "art_bomorrean",
    "art_dagger",
    "art_duocylinder",
    "art_harp",
    "art_irrational",
    "art_klein",
    "art_penrose",
    "art_ring",
    "art_spear",
    "art_tesseract",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

FIBER = (178, 148, 96, 255)        # dry-plant tan
FIBER_DARK = (140, 112, 70, 255)
ROPE = (196, 164, 104, 255)
ROUGH = (158, 138, 104, 255)       # makeshift: grayer, rougher
ROUGH_DARK = (118, 102, 74, 255)
LEATHER = (122, 84, 50, 255)
LEATHER_DARK = (92, 62, 36, 255)
LEATHER_LIGHT = (156, 112, 70, 255)
NYLON = (128, 132, 140, 255)
NYLON_DARK = (94, 98, 108, 255)
STEEL = (168, 174, 182, 255)
STEEL_DARK = (110, 116, 126, 255)
IRON = (104, 106, 114, 255)
IRON_DARK = (70, 72, 80, 255)
BRONZE = (176, 124, 62, 255)
BRONZE_DARK = (128, 88, 42, 255)
VINE = (74, 128, 58, 255)
VINE_DARK = (52, 94, 42, 255)
LEAF = (92, 156, 70, 255)
CHITIN = (116, 62, 44, 255)
CHITIN_DARK = (82, 42, 30, 255)
ACIDCHITIN = (84, 168, 128, 255)
ACID_DARK = (50, 118, 88, 255)
KEVLAR = (172, 160, 110, 255)
KEVLAR_DARK = (134, 124, 84, 255)
RUBBER = (62, 130, 178, 255)
RUBBER_DARK = (42, 96, 136, 255)
BONE = (228, 220, 198, 255)
MONOLITH = (44, 44, 52, 255)
MONO_LIGHT = (84, 84, 96, 255)
FLESH = (196, 110, 104, 255)
FLESH_DARK = (156, 76, 74, 255)
CARAPACE = (122, 96, 168, 255)
GOLD = (216, 168, 60, 255)
AMBER = (232, 148, 44, 255)
GLASS = (196, 226, 230, 255)
WOOD = (110, 78, 44, 255)
WOOD_DARK = (76, 52, 28, 255)
BLACKWOOD = (40, 32, 30, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------------- ropes

def coil(d, rng, cx, cy, r, col, dark, rough=False, leaves=False):
    """Concentric rope coil; r = outer radius."""
    for rr in range(r, 1, -2):
        c = col if (rr // 2) % 2 == 0 else dark
        d.ellipse([cx - rr, cy - rr * 3 // 4, cx + rr, cy + rr * 3 // 4],
                  outline=c, width=2)
    # strand ticks around the outer turns
    steps = 26 if not rough else 18
    for i in range(steps):
        a = i * (2 * math.pi / steps) + rng.random() * 0.1
        for rr in (r, r - 2):
            x = cx + rr * math.cos(a)
            y = cy + rr * 3 // 4 * math.sin(a)
            d.point((int(x), int(y)), fill=dark)
    d.ellipse([cx - 2, cy - 1, cx + 2, cy + 1], outline=dark, width=1)
    if leaves:
        for lx, ly in ((cx - r - 1, cy - 3), (cx + r, cy - 1),
                       (cx - 2, cy - r * 3 // 4 - 1), (cx + r - 2, cy + 4)):
            d.polygon([(lx, ly), (lx + 3, ly - 2), (lx + 4, ly + 1),
                       (lx + 1, ly + 2)], fill=LEAF)
            d.point((lx + 1, ly), fill=VINE_DARK)


def draw_rope(d, rng, sid):
    if sid == "stick_fiber":
        # a green fibrous stalk, leaning, with fraying fibre strands
        d.line([10, 27, 21, 5], fill=VINE, width=3)
        d.line([10, 27, 21, 5], fill=VINE_DARK, width=1)
        for t in range(5):
            x = 12 + t * 2
            y = 23 - t * 4
            d.line([x, y, x - 2, y - 2], fill=LEAF, width=1)
            d.line([x + 1, y + 1, x + 3, y - 1], fill=LEAF, width=1)
        d.polygon([(20, 4), (24, 6), (21, 9)], fill=LEAF)
        return
    if sid == "seatbelt":
        # rolled flat webbing: concentric rounded bands + buckle
        for i, rr in enumerate((11, 9, 7, 5)):
            c = NYLON if i % 2 == 0 else NYLON_DARK
            d.rounded_rectangle([16 - rr, 16 - rr * 3 // 4, 16 + rr,
                                 16 + rr * 3 // 4], radius=3, outline=c,
                                width=2)
        d.rectangle([13, 13, 19, 19], fill=STEEL_DARK)
        d.rectangle([14, 14, 18, 18], fill=STEEL)
        d.rectangle([15, 15, 17, 17], fill=NYLON_DARK)
        # loose end with the tongue plate
        d.rectangle([24, 20, 29, 23], fill=NYLON)
        d.rectangle([28, 20, 30, 23], fill=STEEL)
        return
    if sid.startswith("vine"):
        r = 11 if sid == "vine_30" else 7
        coil(d, rng, 16, 16, r, VINE, VINE_DARK, leaves=True)
        return
    if sid.startswith("cordage"):
        leather = sid.endswith("leather")
        r = 5 if sid.startswith("cordage_6") else 8
        col = LEATHER if leather else FIBER
        dark = LEATHER_DARK if leather else FIBER_DARK
        coil(d, rng, 16, 16, r, col, dark)
        if leather and r == 8:  # lace pieces tied together: knot bumps
            for kx, ky in ((9, 12), (23, 13), (16, 23)):
                d.rectangle([kx - 1, ky - 1, kx + 1, ky + 1],
                            fill=LEATHER_DARK)
        return
    # ropes: makeshift rough vs proper
    rough = "makeshift" in sid
    r = 11 if sid.endswith("_30") else 8
    if rough:
        coil(d, rng, 16, 16, r, ROUGH, ROUGH_DARK, rough=True)
    else:
        coil(d, rng, 16, 16, r, ROPE, FIBER_DARK)


# ----------------------------------------------------------------- barding
# Panel outline per animal: (x0, x1, y_top, y_bot, neck_rise)
BARD = {
    "cow": (3, 28, 11, 27, 0),
    "horse": (6, 26, 9, 27, 5),
}

MATS = ("acidchitin", "bronze", "chainmail", "chitin", "iron", "kevlar",
        "leather", "leatherbone", "rubber", "steel")


def barding_panel(d, animal):
    x0, x1, yt, yb, neck = BARD[animal]
    pts = [(x0, yt + 3), (x0 + 4, yt), (x1 - 6, yt)]
    if neck:  # criniere rises at the front (right side)
        pts += [(x1 - 2, yt - neck), (x1, yt - neck + 2)]
    pts += [(x1, yt + 3), (x1 - 1, yb), (x0 + 1, yb)]
    return pts, (x0, x1, yt, yb)


def texture_plates(d, box, base, dark, rivet=True):
    x0, x1, yt, yb = box
    rows = 3
    rh = (yb - yt - 2) // rows
    for r in range(rows):
        y0 = yt + 2 + r * rh
        off = 5 if r % 2 else 0
        for x in range(x0 + 2 + off, x1 - 4, 9):
            d.rectangle([x, y0, x + 7, y0 + rh - 1], outline=dark, width=1)
            d.line([x + 1, y0 + 1, x + 6, y0 + 1], fill=shade(base, 1.18),
                   width=1)
            if rivet:
                d.point((x + 1, y0 + rh - 2), fill=dark)
                d.point((x + 6, y0 + rh - 2), fill=dark)


def texture_segments(d, box, base, dark):
    x0, x1, yt, yb = box
    for i, x in enumerate(range(x0 + 2, x1 - 3, 6)):
        c = base if i % 2 == 0 else shade(base, 0.88)
        d.arc([x - 2, yt + 2, x + 6, yb + 4], start=270, end=90, fill=c,
              width=2)
        d.line([x + 2, yt + 3, x + 2, yb - 2], fill=dark, width=1)
    for x in range(x0 + 3, x1 - 2, 6):  # glossy shell highlights
        d.point((x, yt + 4), fill=shade(base, 1.3))


def texture_chainmail(d, box):
    x0, x1, yt, yb = box
    for y in range(yt + 3, yb - 1, 3):
        for x in range(x0 + 2 + (3 if (y // 3) % 2 else 0), x1 - 2, 4):
            d.ellipse([x, y, x + 2, y + 2], outline=STEEL, width=1)


def texture_quilt(d, box):
    x0, x1, yt, yb = box
    for i in range(-4, 8):
        d.line([x0 + 2 + i * 4, yt + 2, x0 + 2 + i * 4 + 8, yb - 1],
               fill=KEVLAR_DARK, width=1)
        d.line([x0 + 2 + i * 4 + 8, yt + 2, x0 + 2 + i * 4, yb - 1],
               fill=KEVLAR_DARK, width=1)
    for y in range(yt + 4, yb - 1, 5):
        for x in range(x0 + 4, x1 - 2, 8):
            d.point((x, y), fill=shade(KEVLAR, 0.8))


def draw_barding(d, rng, sid):
    animal = "cow" if sid.endswith("_cow") else "horse"
    mat = sid[:-len("_armor_cow")] if animal == "cow" \
        else sid[:-len("_armor_horse")]
    pts, box = barding_panel(d, animal)
    x0, x1, yt, yb = box
    base_fill = {
        "acidchitin": ACIDCHITIN, "bronze": BRONZE, "chainmail": (74, 76, 84, 255),
        "chitin": CHITIN, "iron": IRON, "kevlar": KEVLAR,
        "leather": LEATHER, "leatherbone": LEATHER, "rubber": RUBBER,
        "steel": STEEL,
    }[mat]
    d.polygon(pts, fill=base_fill)
    d.line(pts + [pts[0]], fill=shade(base_fill, 0.55), width=1)
    if mat == "acidchitin":
        texture_segments(d, box, ACIDCHITIN, ACID_DARK)
    elif mat == "chitin":
        texture_segments(d, box, CHITIN, CHITIN_DARK)
    elif mat == "bronze":
        texture_plates(d, box, BRONZE, BRONZE_DARK)
    elif mat == "iron":
        texture_plates(d, box, IRON, IRON_DARK)
    elif mat == "steel":
        texture_plates(d, box, STEEL, STEEL_DARK)
    elif mat == "chainmail":
        texture_chainmail(d, box)
    elif mat == "kevlar":
        texture_quilt(d, box)
    elif mat == "rubber":
        # thin glossy sheet: sheen diagonal + drip beads
        d.line([x0 + 4, yb - 3, x1 - 8, yt + 3], fill=shade(RUBBER, 1.35),
               width=2)
        for bx in range(x0 + 4, x1 - 3, 6):
            d.point((bx, yb - 2), fill=shade(RUBBER, 1.3))
    if mat in ("leather", "leatherbone"):
        # cloth undercovering peeking below the leather
        d.line([x0 + 2, yb - 1, x1 - 2, yb - 1], fill=(196, 186, 168, 255),
               width=1)
    if mat == "leather":
        # caparison: tiny dragon-vs-griffin battle motif at the centre
        cx, cy = (x0 + x1) // 2, (yt + yb) // 2 + 1
        d.polygon([(cx - 5, cy + 2), (cx - 2, cy - 3), (cx - 1, cy + 1),
                   (cx - 3, cy + 3)], fill=(70, 110, 62, 255))  # dragon
        d.polygon([(cx + 5, cy + 2), (cx + 2, cy - 3), (cx + 1, cy + 1),
                   (cx + 3, cy + 3)], fill=(206, 190, 150, 255))  # griffin
        d.point((cx - 3, cy - 2), fill=(196, 48, 40, 255))
        d.point((cx + 3, cy - 2), fill=GOLD)
    if mat == "leatherbone":
        # decorative white bones strapped across the panel
        for bx, by, ang in ((x0 + 5, yt + 6, 1), (x1 - 8, yt + 7, -1),
                            ((x0 + x1) // 2, yb - 6, 1)):
            d.line([bx, by, bx + 4 * ang, by + 3], fill=BONE, width=2)
            d.point((bx - 1, by - 1), fill=BONE)
            d.point((bx + 4 * ang + 1, by + 4), fill=BONE)
    if mat in ("bronze", "iron", "steel", "chainmail"):
        # leather straps + girth band
        d.line([x0 + 3, yt + 1, x0 + 3, yb - 1], fill=LEATHER_DARK, width=1)
        d.line([x1 - 4, yt + 1, x1 - 4, yb - 1], fill=LEATHER_DARK, width=1)
    d.line([x0 + 1, yb - 3, x1 - 1, yb - 3], fill=shade(base_fill, 0.6),
           width=1)  # girth shadow


# -------------------------------------------------------------- migo plates

def plate_shape(d, box, fill):
    x0, x1, yt, yb = box
    pts = [(x0 + 2, yt), (x1 - 2, yt), (x1, yt + 4), (x1 - 2, yb),
           (x0 + 2, yb), (x0, yt + 4)]
    d.polygon(pts, fill=fill)
    d.line(pts + [pts[0]], fill=shade(fill, 0.5), width=1)
    # concave profile: lit centre band, dark edges
    d.line([x0 + 3, yt + 3, x0 + 3, yb - 3], fill=shade(fill, 0.7), width=1)
    d.line([x1 - 3, yt + 3, x1 - 3, yb - 3], fill=shade(fill, 0.7), width=1)
    return pts


def scar_glyph(d, cx, cy, col):
    d.line([cx - 3, cy - 4, cx, cy - 1, cx - 2, cy + 2, cx + 2, cy + 4],
           fill=col, width=1)
    d.line([cx + 3, cy - 3, cx + 1, cy], fill=col, width=1)
    d.point((cx, cy - 1), fill=col)


def draw_migo(d, rng, sid):
    side = "sideplate" in sid
    kind = sid.split("_")[-1] if "_" in sid.replace("migo_", "") else "plate"
    if sid in ("migo_plate", "migo_sideplate"):
        kind = "base"
    box = (9, 22, 7, 25) if side else (6, 25, 4, 28)
    if kind == "undergrown":
        box = (box[0] + 2, box[1] - 2, box[2] + 2, box[3] - 2)
    if kind == "naked":
        plate_shape(d, box, FLESH)
        x0, x1, yt, yb = box
        for y in range(yt + 4, yb - 2, 4):  # scarred seams + veins
            d.line([x0 + 3, y, x1 - 4, y + rng.randint(-1, 1)],
                   fill=FLESH_DARK, width=1)
        scar_glyph(d, (x0 + x1) // 2, (yt + yb) // 2, FLESH_DARK)
        return
    base = shade(CARAPACE, 1.12) if kind == "undergrown" else CARAPACE
    plate_shape(d, box, base)
    x0, x1, yt, yb = box
    # iridescent diagonal sheen bands
    bands = [(122, 96, 168, 255), (84, 168, 150, 255), (196, 120, 180, 255),
             (90, 130, 190, 255)]
    for i in range(4):
        bx = x0 + 3 + i * 5
        d.line([bx, yb - 3, bx + 6, yt + 3], fill=bands[i], width=2)
    d.line([x0 + 3, yt + 2, x1 - 3, yt + 2], fill=shade(base, 1.35), width=1)
    scar_glyph(d, (x0 + x1) // 2, (yt + yb) // 2, shade(base, 0.45))
    if kind == "overgrown":
        # sharp carapace shards jutting haphazardly off the edges
        shards = [((x0 + 1, yt + 2), (x0 - 3, yt - 2), (x0 + 4, yt)),
                  ((x1 - 1, yt + 3), (x1 + 3, yt - 1), (x1 - 4, yt + 1)),
                  ((x0, yb - 4), (x0 - 3, yb + 1), (x0 + 4, yb - 1)),
                  ((x1, yb - 5), (x1 + 3, yb), (x1 - 4, yb - 2)),
                  (((x0 + x1) // 2, yt), ((x0 + x1) // 2 + 2, yt - 3),
                   ((x0 + x1) // 2 + 4, yt + 1))]
        for s in shards:
            d.polygon(list(s), fill=shade(base, 0.8))
            d.line(list(s) + [s[0]], fill=shade(base, 0.45), width=1)
        for gx, gy in ((x0 + 6, yt + 8), (x1 - 7, yb - 8)):
            d.point((gx, gy), fill=(84, 168, 150, 255))


# -------------------------------------------------------------------- misc

def draw_mask(d, rng):
    # sealed monolith mask: rounded face shell, horn filters, spiral lenses
    d.rounded_rectangle([8, 6, 23, 27], radius=5, fill=MONOLITH)
    d.rounded_rectangle([8, 6, 23, 27], radius=5, outline=MONO_LIGHT,
                        width=1)
    # filter components turned up like horns at both cheeks
    for hx, flip in ((7, -1), (24, 1)):
        d.polygon([(hx, 14), (hx + 3 * flip, 8), (hx + 4 * flip, 5),
                   (hx + 1, 5), (hx - 1, 10)], fill=MONO_LIGHT)
        d.point((hx + 3 * flip, 6), fill=STEEL_DARK)
        d.rectangle([hx - 1, 15, hx + 1, 19], fill=STEEL_DARK)
    # lenses with spiral inscriptions
    for cx in (12, 19):
        d.ellipse([cx - 2, 10, cx + 2, 14], fill=(30, 34, 40, 255))
        d.arc([cx - 2, 10, cx + 2, 14], start=30, end=330,
              fill=(150, 170, 190, 255), width=1)
        d.point((cx, 12), fill=(150, 170, 190, 255))
    # central filter grille
    d.rectangle([13, 19, 18, 24], fill=(30, 30, 36, 255))
    for gy in (20, 22):
        d.line([14, gy, 17, gy], fill=MONO_LIGHT, width=1)
    d.line([10, 8, 21, 8], fill=MONO_LIGHT, width=1)


def draw_feeding_kit(d, rng):
    # open case with syringes and an aerosol sprayer
    d.rectangle([4, 14, 27, 27], fill=(58, 62, 70, 255))
    d.rectangle([4, 14, 27, 27], outline=STEEL_DARK, width=1)
    d.rectangle([5, 15, 26, 18], fill=(40, 43, 50, 255))  # lid shadow band
    # syringes: barrel, plunger, nozzle
    for sx, col in ((8, (120, 200, 220, 255)), (13, (220, 160, 90, 255))):
        d.rectangle([sx, 19, sx + 2, 25], fill=GLASS)
        d.rectangle([sx, 22, sx + 2, 25], fill=col)
        d.line([sx + 1, 25, sx + 1, 27], fill=STEEL, width=1)
        d.rectangle([sx - 1, 18, sx + 3, 19], fill=STEEL_DARK)
    # aerosol sprayer bottle with trigger nozzle
    d.rectangle([20, 20, 24, 26], fill=(210, 214, 218, 255))
    d.rectangle([20, 20, 24, 22], fill=(196, 90, 60, 255))
    d.polygon([(22, 20), (27, 18), (27, 20), (23, 21)], fill=STEEL_DARK)
    for px, py in ((28, 17), (29, 19), (28, 15)):
        d.point((px, py), fill=(170, 220, 235, 255))  # spray mist


def draw_artifact(d, rng, sid):
    if sid == "art_bomorrean":
        # three interlocked rings, none separable
        rings = [((16, 10), (196, 80, 70, 255)), ((11, 19), (80, 160, 90, 255)),
                 ((21, 19), (80, 110, 190, 255))]
        for (cx, cy), col in rings:
            d.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], outline=col, width=3)
        # redraw small arcs to fake over-under interlock
        d.arc([10, 4, 22, 16], start=200, end=290, fill=rings[1][1], width=3)
        d.arc([15, 13, 27, 25], start=20, end=110, fill=rings[0][1], width=3)
        d.arc([5, 13, 17, 25], start=250, end=340, fill=rings[2][1], width=3)
    elif sid == "art_dagger":
        # athame: steel blade, rune-carved wooden handle, diagonal
        d.polygon([(6, 26), (20, 12), (23, 9), (21, 14), (10, 25)],
                  fill=STEEL)
        d.line([8, 24, (20), (12)], fill=STEEL_DARK, width=1)
        d.polygon([(20, 12), (26, 6), (28, 8), (23, 15)], fill=WOOD)
        for rx, ry in ((23, 11), (25, 9), (24, 13)):  # runes
            d.point((rx, ry), fill=WOOD_DARK)
            d.point((rx + 1, ry), fill=WOOD_DARK)
        d.ellipse([19, 13, 22, 16], fill=BRONZE)  # guard/pommel ring
    elif sid == "art_duocylinder":
        # chunky ring folding onto itself: torus with a twist band
        d.ellipse([6, 7, 26, 25], outline=(150, 130, 170, 255), width=5)
        d.ellipse([6, 7, 26, 25], outline=shade((150, 130, 170, 255), 0.6),
                  width=1)
        d.ellipse([11, 12, 21, 20], outline=shade((150, 130, 170, 255), 0.6),
                  width=1)
        d.polygon([(9, 9), (14, 13), (11, 16), (7, 12)],
                  fill=(178, 158, 200, 255))  # the fold crossing over
    elif sid == "art_harp":
        # black-wood harp: curved pillar, neck, soundbox, steel strings
        d.arc([4, 4, 30, 30], start=270, end=360, fill=BLACKWOOD, width=3)
        d.line([7, 6, 7, 26], fill=BLACKWOOD, width=3)
        d.line([7, 26, 26, 26], fill=BLACKWOOD, width=2)
        d.polygon([(24, 12), (28, 26), (24, 26)], fill=BLACKWOOD)
        for i in range(6):
            x = 10 + i * 2
            d.line([x, 8 + i, x, 25], fill=(200, 204, 210, 255), width=1)
        d.point((5, 10), fill=GOLD)
        d.point((5, 20), fill=GOLD)
    elif sid == "art_irrational":
        # wire-frame cube whose inner wires cross over the outer ones
        wire = (210, 200, 170, 255)
        d.rectangle([7, 10, 21, 24], outline=wire, width=1)
        d.rectangle([13, 5, 27, 19], outline=wire, width=1)
        for a, b in (((7, 10), (13, 5)), ((21, 10), (27, 5)),
                     ((7, 24), (13, 19)), ((21, 24), (27, 19))):
            d.line([a, b], fill=wire, width=1)
        # impossible interior: diagonals drawn OVER the frame
        d.line([7, 24, 27, 5], fill=(240, 120, 90, 255), width=1)
        d.line([21, 10, 13, 19], fill=(240, 120, 90, 255), width=1)
        d.line([14, 17, 20, 12], fill=(240, 120, 90, 255), width=1)
    elif sid == "art_klein":
        # Klein flask: neck curves over and enters the body to the bottom
        d.ellipse([10, 16, 24, 28], fill=GLASS)
        d.ellipse([10, 16, 24, 28], outline=(120, 160, 168, 255), width=1)
        d.arc([8, 2, 26, 20], start=180, end=360, fill=(120, 160, 168, 255),
              width=2)  # neck arching over
        d.line([10, 10, 10, 18], fill=(120, 160, 168, 255), width=2)
        d.line([24, 10, 21, 17], fill=(120, 160, 168, 255), width=2)
        d.line([21, 17, 18, 24], fill=(150, 190, 198, 255), width=2)
        d.ellipse([11, 3, 15, 6], outline=(120, 160, 168, 255), width=1)
        d.line([12, 20, 20, 20], fill=(150, 190, 198, 255), width=1)
        d.point((14, 24), fill=(255, 255, 255, 255))
    elif sid == "art_penrose":
        # impossible triangle from three thick bars, shaded per bar
        bar = 4
        d.polygon([(6, 25), (26, 25), (26, 25 - bar), (6, 25 - bar)],
                  fill=(196, 170, 90, 255))  # bottom bar
        d.polygon([(22, 25), (16, 6), (19, 5), (26, 25)],
                  fill=(150, 128, 66, 255))  # right bar rising
        d.polygon([(6, 25), (16, 6), (19, 9), (10, 25)],
                  fill=(228, 200, 116, 255))  # left bar
        d.polygon([(13, 10), (19, 9), (18, 12), (14, 13)],
                  fill=(196, 170, 90, 255))  # impossible top joint
        d.point((16, 7), fill=(240, 220, 150, 255))
    elif sid == "art_ring":
        # gold band, amber stone with a spiral worm sealed inside
        d.ellipse([8, 12, 24, 27], outline=GOLD, width=3)
        d.ellipse([11, 6, 21, 15], fill=AMBER)
        d.ellipse([11, 6, 21, 15], outline=shade(AMBER, 0.7), width=1)
        pts = []
        for t in range(16):
            a = t * 0.7
            r = 3.2 - t * 0.16
            pts.append((16 + r * math.cos(a), 10.5 + r * math.sin(a) * 0.8))
        d.line(pts, fill=(120, 70, 30, 255), width=1)
        d.point((13, 8), fill=(255, 230, 170, 255))
    elif sid == "art_spear":
        # wooden pole, steel spike, blue sun painted on the handle
        d.line([8, 24, 22, 10], fill=WOOD, width=3)
        d.polygon([(21, 11), (28, 4), (24, 13)], fill=STEEL)
        d.line([22, 10, (26), (6)], fill=STEEL_DARK, width=1)
        cx, cy = 13, 19  # blue sun symbol
        d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=(70, 110, 210, 255))
        for dx, dy in ((-3, 0), (3, 0), (0, -3), (0, 3), (-2, -2), (2, 2)):
            d.point((cx + dx, cy + dy), fill=(70, 110, 210, 255))
    elif sid == "art_tesseract":
        # cube within a cube, vertices linked, mid-fold
        wire = (170, 200, 235, 255)
        d.rectangle([5, 7, 23, 25], outline=wire, width=1)
        d.rectangle([12, 12, 20, 20], outline=(240, 180, 90, 255), width=1)
        for a, b in (((5, 7), (12, 12)), ((23, 7), (20, 12)),
                     ((5, 25), (12, 20)), ((23, 25), (20, 20))):
            d.line([a, b], fill=wire, width=1)
        d.line([12, 12, 23, 25], fill=(240, 120, 160, 255), width=1)
        d.line([20, 20, 5, 7], fill=(240, 120, 160, 255), width=1)


# -------------------------------------------------------------------- main

ROPE_IDS = {"cordage_36", "cordage_36_leather", "cordage_6",
            "cordage_6_leather", "rope_30", "rope_makeshift_30",
            "rope_makeshift_6", "seatbelt", "stick_fiber", "vine_30",
            "vine_6"}
MIGO_IDS = {s for s in IDS if s.startswith("migo_")}
ART_IDS = {s for s in IDS if s.startswith("art_")}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in ROPE_IDS:
        draw_rope(d, rng, sid)
    elif sid.endswith("_armor_cow") or sid.endswith("_armor_horse"):
        draw_barding(d, rng, sid)
    elif sid in MIGO_IDS:
        draw_migo(d, rng, sid)
    elif sid == "armor_star_samurai_mask":
        draw_mask(d, rng)
    elif sid == "feeding_kit":
        draw_feeding_kit(d, rng)
    elif sid in ART_IDS:
        draw_artifact(d, rng, sid)
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
    # contact-sheet preview: 8 cols x 7 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 7, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item72.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item72.png {prev.size}")


if __name__ == "__main__":
    main()
