#!/usr/bin/env python3
"""Batch ITEM28 (Phase 2g.4 scope extension): ammunition
components and ammo-typed resources -- steel bullet cores,
gunpowders, primers, chain links, lamellae, gas charges and
assorted Rivtech/medical/chemical ammo items.

52 ids (authoritative list embedded below; source list:
~/workspace/item-batches/28.txt). 49 are defined in
data/json/items/ammo.json; hydrogen, nitrogen and
uran_cell also have material entries in materials.json
(the ITEM entries in ammo.json are the ones drawn).

Entries read first, copy-from chains resolved:
- Steel cores (123/3006/50/545/9x18): lathe-machined
  stainless bullet cores, dark_gray steel, symbol ';'.
  Drawn as three standing spitzer cores, sized by the
  entry's weight/volume (9x18 1 g smallest, .50 BMG
  54 g by far the largest), with lathe turning rings.
- Gunpowders (artillery/large_rifle/magnum_pistol/
  pistol/rifle/shotgun): all copy-from `gunpowder`
  (dark_gray smokeless powder). Drawn as a powder heap
  on a steel tray, differentiated by grain form: pistol
  = fine grains, magnum = coarser blacker grains, rifle
  = extruded sticks, large rifle = longer sticks,
  shotgun = flat flakes, artillery = coarse chunks and
  the biggest heap.
- Primers (lg/sm pistol, lg/sm rifle, shotgun; plus a
  spent version of each): small steel primer cups,
  dark_gray. Fresh cups are bright with a clean center;
  spent cups are dull with a dark firing-pin dent.
  Large > small; rifle slightly taller than pistol;
  the shotgun primer is the tall battery cup with a
  base flange.
- Chain links (lc/mc/hc/ch/qt): copy-from lc_chain_link,
  "a small metal ring, suitable for constructing
  chainmail." Drawn as three interlocked rings, shaded
  per steel grade (mild dull, medium mid, high bright,
  hardened dark blue-steel, tempered blue-tinted with a
  straw glint).
- Lamellae (lc/ch): piles of 3x6 cm steel rectangles
  with drilled hole patterns for lamellar armor. Drawn
  as a fanned stack of three holed plates, mild dull
  vs. hardened dark blue-steel.
- RTG_coffee: a 3 kg lead/steel radioisotope generator
  chunk, green. Drawn as a squat finned canister with a
  green radiation trefoil.
- betavoltaic: a D-cell-sized cell with folded
  radioactive layers, green. Drawn as a D-cell with a
  green label band and trefoil.
- uran_cell: hard ceramic UO2 pellets, green,
  looks_like cobalt_60. Drawn as three stacked dark
  ceramic pellets with green rim light.
- ampoule: RA21E medical ampoule -- tiny glass vial of
  cyan fast-healing chemicals for a jet injector.
- stimpack_ammo: RA10K stimulant module -- pressurized
  multi-chamber glass vial, cyan. Drawn segmented with
  a crimp cap.
- co2: propellant charge -- drawn as a classic silver
  CO2 powerlet cartridge.
- hydrogen / nitrogen: compressed gases, white. Drawn
  as small gas bottles: hydrogen slim with a red
  shoulder band, nitrogen squat with a black shoulder
  band (real cylinder shoulder colors).
- nitrox: diver's gas mix, 250 ml -- a small dive tank
  with the green/yellow nitrox band.
- money: "cent" -- a copper penny (two stacked coins).
- copper_wire: a coil of raw copper wire.
- candle_wax: a lump of white candle wax.
- extinguishing_agent: white dry-chemical powder heap.
- flare_nitrate: white pyrotechnic crystals with
  orange spark flecks.
- glowstick_juice: blue dye mix -- a slim tube of
  glowing blue liquid.
- gasfilter_m: activated carbon -- black granules in a
  shallow cup. gasfilter_s: a sealed round filter mask
  cartridge with a pleated dark face.
- liq_bandage: a tiny bottle with a brush cap.
- moisturizer: a small squeeze tube with a cream
  dollop (ammo_type soap).
- notch: a small brown wooden fire-drill notch board
  with a V cut and charred ember spot.
- rebreather_filler_ko2: potassium superoxide -- pale
  yellow granules in a tin (KO2 is yellow).
  rebreather_filler_soda_lime: white/pink scrubber
  granules in a tin.
- rubber_slug: a fat dark rubber beanbag slug.
- teargas_spray: a small spray canister with a pale
  irritant puff.

Translucency is faked with pale opaque colors (this
Pillow build replaces rather than blends); all sprite
alpha is in {0, 255}, asserted per sprite.

Writes ONLY its own 52 folders + preview_item28.png.
PIL only; per-sprite random.Random(crc32(id)).
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "123_steel_core",
    "3006_steel_core",
    "50_steel_core",
    "545_steel_core",
    "9x18_steel_core",
    "RTG_coffee",
    "ampoule",
    "betavoltaic",
    "candle_wax",
    "ch_chain_link",
    "co2",
    "copper_wire",
    "extinguishing_agent",
    "flare_nitrate",
    "gasfilter_m",
    "gasfilter_s",
    "glowstick_juice",
    "gunpowder_artillery",
    "gunpowder_large_rifle",
    "gunpowder_magnum_pistol",
    "gunpowder_pistol",
    "gunpowder_rifle",
    "gunpowder_shotgun",
    "hc_chain_link",
    "hydrogen",
    "lamellae_ch",
    "lamellae_lc",
    "lc_chain_link",
    "lgpistol_primer",
    "lgrifle_primer",
    "liq_bandage",
    "mc_chain_link",
    "moisturizer",
    "money",
    "nitrogen",
    "nitrox",
    "notch",
    "qt_chain_link",
    "rebreather_filler_ko2",
    "rebreather_filler_soda_lime",
    "rubber_slug",
    "shotgun_primer",
    "smpistol_primer",
    "smrifle_primer",
    "spent_lgpistol_primer",
    "spent_lgrifle_primer",
    "spent_shotgun_primer",
    "spent_smpistol_primer",
    "spent_smrifle_primer",
    "stimpack_ammo",
    "teargas_spray",
    "uran_cell",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA "
           "3.0, as the tileset.\n")

INK = (24, 26, 24, 255)
# steels
STEEL = (150, 154, 158, 255)
STEEL_D = (96, 100, 105, 255)
STEEL_L = (198, 202, 208, 255)
CORE = (128, 132, 138, 255)
CORE_D = (84, 88, 94, 255)
CORE_L = (182, 186, 192, 255)
TRAY = (140, 144, 148, 255)
TRAY_D = (92, 96, 100, 255)
# glass / liquids
GLASS = (206, 232, 230, 255)
GLASS_D = (140, 178, 176, 255)
CYAN = (56, 196, 216, 255)
CYAN_D = (30, 140, 168, 255)
# radiation greens
RAD = (64, 168, 74, 255)
RAD_D = (36, 110, 44, 255)
LEAD = (96, 98, 104, 255)
LEAD_D = (62, 64, 70, 255)
# copper / brass
COPPER = (188, 112, 62, 255)
COPPER_D = (138, 76, 40, 255)
COPPER_L = (226, 156, 96, 255)
# whites / powders
WHITE = (238, 238, 234, 255)
WHITE_D = (198, 198, 192, 255)
POWDER = (58, 58, 60, 255)
POWDER_D = (38, 38, 40, 255)
POWDER_L = (96, 96, 98, 255)
WOOD = (146, 104, 62, 255)
WOOD_D = (104, 70, 40, 255)
CHAR = (36, 30, 24, 255)


def blank():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


def trefoil(d, cx, cy, r, color):
    """Radiation trefoil: three 60-degree blades + hub."""
    import math
    for k in range(3):
        a0 = math.radians(-90 + k * 120 - 30)
        a1 = math.radians(-90 + k * 120 + 30)
        pts = [(cx, cy)]
        steps = 6
        for s in range(steps + 1):
            a = a0 + (a1 - a0) * s / steps
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
        d.polygon(pts, fill=color)
    d.ellipse([cx - r * 0.22, cy - r * 0.22, cx + r * 0.22,
               cy + r * 0.22], fill=color)


# ----------------------------------------------------------------
# steel cores
# ----------------------------------------------------------------
def draw_core(rng, half_w, height):
    img = blank()
    d = ImageDraw.Draw(img)
    base = 28

    def core(cx, hw, h, dx=0):
        tip = base - h
        shoulder = base - int(h * 0.45)
        pts = [(cx - hw, base), (cx - hw, shoulder),
               (cx - hw // 2, tip + 2), (cx, tip),
               (cx + hw // 2, tip + 2), (cx + hw, shoulder),
               (cx + hw, base)]
        d.polygon(pts, fill=CORE)
        d.line([(cx - hw, base), (cx - hw, shoulder),
                (cx - hw // 2, tip + 2), (cx, tip)], fill=INK)
        d.line([(cx + hw, base), (cx + hw, shoulder),
                (cx + hw // 2, tip + 2), (cx, tip)], fill=INK)
        d.line([(cx - hw, base), (cx + hw, base)], fill=INK)
        # lathe turning rings
        for yy in (base - 3, base - 6):
            if yy > shoulder:
                d.line([(cx - hw, yy), (cx + hw, yy)], fill=CORE_D)
        # highlight
        d.line([(cx - hw + 1, base - 2), (cx - hw + 1, shoulder + 1)],
               fill=CORE_L)

    core(16, half_w, height)
    core(16 - half_w - 4, max(1, half_w - 1), height - 3)
    core(16 + half_w + 4, max(1, half_w - 1), height - 3)
    return img


# ----------------------------------------------------------------
# powders on a tray
# ----------------------------------------------------------------
def tray(d):
    d.polygon([(5, 24), (27, 24), (25, 29), (7, 29)], fill=TRAY)
    d.line([(5, 24), (27, 24)], fill=TRAY_D)
    d.line([(7, 29), (25, 29)], fill=INK)


def heap(d, cy, rx, ry, fill):
    d.ellipse([16 - rx, cy - ry, 16 + rx, cy + ry], fill=fill)
    d.rectangle([16 - rx, cy, 16 + rx, 25], fill=fill)


def grains_dots(d, rng, n, col_a, col_b, r=9):
    for _ in range(n):
        x = 16 + rng.randint(-r, r)
        y = 20 + rng.randint(-2, 3)
        d.point((x, y), fill=col_a if rng.random() < 0.6 else col_b)


def grains_sticks(d, rng, n, length, col):
    for _ in range(n):
        x = 16 + rng.randint(-8, 8)
        y = 20 + rng.randint(-2, 3)
        dx = rng.choice([-1, 0, 1])
        for s in range(length):
            d.point((x + dx * s, y + (s % 2)), fill=col)


def grains_flakes(d, rng, n, col_a, col_b):
    for _ in range(n):
        x = 16 + rng.randint(-8, 8)
        y = 20 + rng.randint(-2, 3)
        c = col_a if rng.random() < 0.5 else col_b
        d.line([(x, y), (x + 2, y)], fill=c)


def grains_chunks(d, rng, n, col_a, col_b):
    for _ in range(n):
        x = 16 + rng.randint(-8, 8)
        y = 20 + rng.randint(-2, 3)
        c = col_a if rng.random() < 0.5 else col_b
        d.rectangle([x, y, x + 1, y + 1], fill=c)


def draw_powder(rng, kind):
    img = blank()
    d = ImageDraw.Draw(img)
    tray(d)
    if kind == "artillery":
        heap(d, 21, 10, 6, POWDER_D)
        grains_chunks(d, rng, 26, POWDER_L, (120, 118, 110, 255))
    elif kind == "shotgun":
        heap(d, 21, 9, 5, (64, 68, 60, 255))
        grains_flakes(d, rng, 22, (150, 156, 138, 255),
                      (110, 116, 100, 255))
    elif kind == "rifle":
        heap(d, 21, 9, 5, (56, 60, 54, 255))
        grains_sticks(d, rng, 16, 3, (128, 134, 118, 255))
    elif kind == "large_rifle":
        heap(d, 21, 9, 5, (54, 58, 52, 255))
        grains_sticks(d, rng, 14, 4, (134, 140, 124, 255))
    elif kind == "magnum":
        heap(d, 21, 9, 5, POWDER_D)
        grains_dots(d, rng, 30, POWDER_L, (140, 138, 130, 255))
    else:  # pistol: fine grains
        heap(d, 21, 9, 5, POWDER)
        grains_dots(d, rng, 34, POWDER_L, (120, 120, 118, 255))
    return img


# ----------------------------------------------------------------
# primers
# ----------------------------------------------------------------
def draw_primer(rng, w, h, spent, shotgun=False):
    img = blank()
    d = ImageDraw.Draw(img)
    body = (104, 106, 108, 255) if spent else STEEL
    top = (128, 130, 132, 255) if spent else STEEL_L
    shade = (72, 74, 76, 255) if spent else STEEL_D

    def cup(cx, cy):
        x0, x1 = cx - w // 2, cx + w // 2
        y0, y1 = cy - h // 2, cy + h // 2
        if shotgun:
            d.rectangle([x0 - 2, y1 - 2, x1 + 2, y1], fill=shade)
        d.rectangle([x0, y0 + 1, x1, y1], fill=body)
        d.ellipse([x0, y0, x1, y0 + 3], fill=top)
        d.ellipse([x0, y0, x1, y0 + 3], outline=INK)
        d.line([(x0, y0 + 2), (x0, y1)], fill=INK)
        d.line([(x1, y0 + 2), (x1, y1)], fill=INK)
        d.line([(x0, y1), (x1, y1)], fill=INK)
        if spent:
            d.ellipse([cx - 1, y0, cx + 1, y0 + 2],
                      fill=(34, 34, 36, 255))
            d.point((cx + 2, y0 + 2), fill=(52, 52, 54, 255))
        else:
            d.ellipse([cx - 1, y0, cx + 1, y0 + 2], fill=shade)

    cup(16, 15)
    cup(10, 22)
    cup(22, 22)
    return img


# ----------------------------------------------------------------
# chain links & lamellae
# ----------------------------------------------------------------
# grade: (ring, ring_dark, glint or None)
GRADES = {
    "lc": ((148, 150, 154, 255), (94, 96, 100, 255), None),
    "mc": ((170, 173, 178, 255), (110, 113, 118, 255), None),
    "hc": ((198, 202, 208, 255), (128, 132, 138, 255), None),
    "ch": ((106, 116, 136, 255), (64, 72, 92, 255), None),
    "qt": ((148, 158, 188, 255), (92, 102, 140, 255),
           (216, 178, 96, 255)),
}


def draw_chain(rng, grade):
    img = blank()
    d = ImageDraw.Draw(img)
    col, dark, glint = GRADES[grade]

    def ring(cx, cy, tall):
        box = [cx - 5, cy - 7, cx + 5, cy + 7] if tall else \
              [cx - 7, cy - 5, cx + 7, cy + 5]
        d.ellipse(box, outline=dark, width=3)
        d.ellipse(box, outline=col, width=1)

    ring(10, 12, False)
    ring(17, 18, True)
    ring(24, 12, False)
    ring(13, 24, True)
    ring(22, 24, False)
    if glint:
        d.arc([5, 7, 15, 17], start=180, end=270, fill=glint)
        d.arc([19, 19, 29, 29], start=0, end=90, fill=glint)
    return img


def draw_lamella(rng, grade):
    img = blank()
    d = ImageDraw.Draw(img)
    col, dark, _ = GRADES[grade]
    fill = (158, 160, 164, 255) if grade == "lc" else \
        (112, 122, 142, 255)

    def plate(x, y):
        d.rectangle([x, y, x + 15, y + 8], fill=fill)
        d.rectangle([x, y, x + 15, y + 8], outline=INK)
        d.line([(x, y + 1), (x + 15, y + 1)], fill=col)
        for hx in (x + 3, x + 8, x + 12):
            for hy in (y + 2, y + 6):
                d.point((hx, hy), fill=dark)
                d.point((hx + 1, hy), fill=dark)

    plate(6, 8)
    plate(10, 15)
    plate(7, 22)
    return img


# ----------------------------------------------------------------
# singles
# ----------------------------------------------------------------
def draw_rtg(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # squat finned canister
    d.rectangle([9, 10, 23, 28], fill=LEAD)
    d.ellipse([9, 7, 23, 12], fill=STEEL)
    d.ellipse([9, 7, 23, 12], outline=INK)
    for x in (12, 16, 20):
        d.line([(x, 12), (x, 27)], fill=LEAD_D)
    d.rectangle([9, 10, 23, 28], outline=INK)
    d.rectangle([14, 4, 18, 8], fill=STEEL_D)  # lifting lug
    trefoil(d, 16, 20, 5, RAD)
    return img


def draw_betavoltaic(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # D-cell
    d.rectangle([11, 9, 21, 28], fill=RAD_D)
    d.rectangle([11, 13, 21, 24], fill=RAD)
    for y in (16, 20):
        d.line([(11, y), (21, y)], fill=RAD_D)
    d.rectangle([11, 9, 21, 28], outline=INK)
    d.ellipse([11, 7, 21, 11], fill=STEEL_L)
    d.ellipse([11, 7, 21, 11], outline=INK)
    d.rectangle([14, 4, 18, 7], fill=STEEL)  # positive nub
    trefoil(d, 16, 19, 3, (20, 60, 24, 255))
    return img


def draw_uran(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    pellet = (54, 56, 52, 255)
    rim = (96, 170, 92, 255)
    for i, y in enumerate((10, 17, 24)):
        d.rectangle([10, y, 22, y + 5], fill=pellet)
        d.ellipse([10, y - 2, 22, y + 2], fill=(74, 76, 70, 255))
        d.ellipse([10, y - 2, 22, y + 2], outline=INK)
        d.line([(10, y), (10, y + 5)], fill=INK)
        d.line([(22, y), (22, y + 5)], fill=INK)
        d.line([(11, y + 5), (21, y + 5)], fill=rim)
    d.point((25, 14), fill=rim)
    d.point((7, 21), fill=rim)
    return img


def draw_ampoule(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # sealed glass ampoule: tip, neck, body
    d.polygon([(15, 5), (17, 5), (17, 10), (15, 10)], fill=GLASS)
    d.polygon([(14, 10), (18, 10), (19, 14), (13, 14)], fill=GLASS)
    d.rectangle([12, 14, 20, 28], fill=GLASS)
    d.rectangle([13, 20, 19, 27], fill=CYAN)
    d.line([(13, 20), (19, 20)], fill=CYAN_D)
    d.line([(14, 15), (14, 27)], fill=(255, 255, 255, 255))
    d.polygon([(15, 5), (17, 5), (17, 10), (18, 10), (19, 14),
               (20, 14), (20, 28), (12, 28), (12, 14), (13, 14),
               (14, 10), (15, 10)], outline=INK)
    d.line([(12, 28), (20, 28)], fill=INK)
    return img


def draw_stimpack(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # multi-chamber pressurized vial with crimp cap
    d.rectangle([13, 4, 19, 8], fill=STEEL_D)  # crimp cap
    d.rectangle([11, 8, 21, 28], fill=GLASS)
    d.rectangle([12, 10, 20, 27], fill=CYAN)
    for y in (15, 21):  # chamber dividers
        d.line([(11, y), (21, y)], fill=GLASS_D)
        d.line([(12, y), (20, y)], fill=CYAN_D)
    d.line([(13, 9), (13, 27)], fill=(255, 255, 255, 255))
    d.rectangle([11, 8, 21, 28], outline=INK)
    return img


def draw_co2(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # powerlet cartridge
    d.polygon([(14, 6), (18, 6), (19, 11), (19, 24), (18, 27),
               (14, 27), (13, 24), (13, 11)], fill=STEEL_L)
    d.polygon([(14, 6), (18, 6), (19, 11), (19, 24), (18, 27),
               (14, 27), (13, 24), (13, 11)], outline=INK)
    d.line([(15, 12), (15, 24)], fill=(255, 255, 255, 255))
    d.line([(18, 12), (18, 24)], fill=STEEL_D)
    d.line([(13, 11), (19, 11)], fill=STEEL_D)
    return img


def draw_gas_bottle(rng, slim, band, motif):
    img = blank()
    d = ImageDraw.Draw(img)
    x0, x1 = (13, 19) if slim else (11, 21)
    y0 = 12
    d.rectangle([x0, y0, x1, 28], fill=(208, 218, 226, 255))
    d.ellipse([x0, y0 - 3, x1, y0 + 3], fill=(226, 234, 240, 255))
    d.ellipse([x0, y0 - 3, x1, y0 + 3], outline=INK)
    d.rectangle([x0, y0 + 2, x1, y0 + 5], fill=band)  # shoulder band
    d.line([(x0, 16), (x0, 28)], fill=INK)
    d.line([(x1, 16), (x1, 28)], fill=INK)
    d.line([(x0, 28), (x1, 28)], fill=INK)
    d.rectangle([15, 5, 17, 9], fill=STEEL_D)  # valve stem
    d.line([(13, 6), (19, 6)], fill=STEEL_D)   # valve wheel
    d.point((16, 6), fill=INK)
    cx = (x0 + x1) // 2
    if motif == "h2":
        d.ellipse([cx - 3, 19, cx, 22], outline=(120, 170, 210, 255))
        d.ellipse([cx, 19, cx + 3, 22], outline=(120, 170, 210, 255))
    else:  # n2: three-dot triangle
        for dx, dy in ((-2, 0), (2, 0), (0, 3)):
            d.point((cx + dx, 20 + dy), fill=(110, 120, 135, 255))
            d.point((cx + dx + 1, 20 + dy), fill=(110, 120, 135, 255))
    return img


def draw_nitrox(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    d.rectangle([12, 11, 20, 28], fill=(168, 194, 204, 255))
    d.ellipse([12, 8, 20, 14], fill=(190, 212, 220, 255))
    d.ellipse([12, 8, 20, 14], outline=INK)
    d.rectangle([12, 17, 20, 21], fill=(240, 200, 60, 255))
    d.rectangle([12, 19, 20, 21], fill=RAD)
    d.line([(12, 14), (12, 28)], fill=INK)
    d.line([(20, 14), (20, 28)], fill=INK)
    d.line([(12, 28), (20, 28)], fill=INK)
    d.rectangle([15, 4, 17, 8], fill=STEEL_D)
    d.line([(13, 5), (19, 5)], fill=STEEL_D)
    return img


def draw_money(rng):
    img = blank()
    d = ImageDraw.Draw(img)

    def penny(cx, cy, r):
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=COPPER)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=COPPER_D)
        d.ellipse([cx - r + 2, cy - r + 2, cx + r - 2, cy + r - 2],
                  outline=COPPER_L)
        d.line([(cx, cy - 3), (cx, cy + 3)], fill=COPPER_D)
        d.point((cx - 1, cy - 3), fill=COPPER_D)

    penny(20, 13, 7)
    penny(13, 20, 8)
    return img


def draw_copper_wire(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    for r, y in ((11, 13), (9, 17), (11, 21)):
        d.ellipse([16 - r, y - 4, 16 + r, y + 4], outline=COPPER_D,
                  width=2)
        d.ellipse([16 - r, y - 4, 16 + r, y + 4], outline=COPPER,
                  width=1)
    # loose end curling out
    d.arc([20, 20, 30, 29], start=270, end=120, fill=COPPER, width=2)
    d.point((29, 24), fill=COPPER_L)
    return img


def draw_wax(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    d.polygon([(10, 14), (22, 14), (23, 26), (21, 28), (11, 28),
               (9, 26)], fill=WHITE)
    # melted wavy top
    d.polygon([(10, 14), (13, 11), (16, 13), (19, 11), (22, 14)],
              fill=WHITE)
    d.line([(10, 14), (13, 11), (16, 13), (19, 11), (22, 14)],
           fill=WHITE_D)
    d.polygon([(10, 14), (22, 14), (23, 26), (21, 28), (11, 28),
               (9, 26)], outline=INK)
    # drip down the front
    d.rectangle([18, 14, 20, 21], fill=WHITE)
    d.ellipse([18, 19, 20, 23], fill=WHITE)
    d.line([(13, 18), (13, 25)], fill=WHITE_D)
    return img


def draw_white_heap(rng, sparks):
    img = blank()
    d = ImageDraw.Draw(img)
    tray(d)
    heap(d, 21, 9, 5, WHITE)
    for _ in range(20):
        x = 16 + rng.randint(-8, 8)
        y = 20 + rng.randint(-2, 3)
        d.point((x, y), fill=WHITE_D)
    if sparks:
        for col in ((232, 122, 48, 255), (204, 62, 40, 255),
                    (244, 178, 60, 255)):
            for _ in range(3):
                x = 16 + rng.randint(-7, 7)
                y = 19 + rng.randint(-2, 2)
                d.point((x, y), fill=col)
    return img


def draw_glowstick(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    d.rectangle([14, 8, 18, 27], fill=(64, 148, 255, 255))
    d.rectangle([14, 5, 18, 8], fill=GLASS_D)  # cap
    d.rectangle([14, 5, 18, 27], outline=INK)
    d.line([(15, 9), (15, 26)], fill=(170, 215, 255, 255))
    for x, y in ((11, 12), (21, 16), (12, 22), (20, 25)):
        d.point((x, y), fill=(140, 200, 255, 255))
    return img


def draw_gasfilter_m(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # shallow cup of black activated-carbon granules
    d.polygon([(8, 18), (24, 18), (22, 27), (10, 27)],
              fill=(88, 90, 94, 255))
    d.polygon([(8, 18), (24, 18), (22, 27), (10, 27)], outline=INK)
    d.ellipse([8, 15, 24, 20], fill=(34, 34, 36, 255))
    for _ in range(26):
        x = rng.randint(10, 22)
        y = rng.randint(16, 19)
        d.point((x, y), fill=(86, 86, 90, 255))
    d.ellipse([8, 15, 24, 20], outline=INK)
    return img


def draw_gasfilter_s(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    d.rectangle([9, 15, 23, 27], fill=(172, 175, 178, 255))
    d.rectangle([9, 15, 23, 27], outline=INK)
    d.ellipse([9, 11, 23, 18], fill=(58, 60, 63, 255))
    d.ellipse([9, 11, 23, 18], outline=INK)
    # pleats radiating on the filter face
    for x in (12, 15, 18, 21):
        d.line([(x, 12), (x + (16 - x) // 4, 17)], fill=(96, 98, 102, 255))
    d.ellipse([14, 13, 18, 16], outline=(120, 122, 126, 255))
    d.line([(10, 22), (22, 22)], fill=STEEL_D)
    return img


def draw_liq_bandage(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    d.rectangle([12, 8, 20, 14], fill=(52, 54, 58, 255))  # brush cap
    d.rectangle([12, 8, 20, 14], outline=INK)
    d.rectangle([11, 14, 21, 28], fill=(228, 230, 224, 255))
    d.rectangle([11, 14, 21, 28], outline=INK)
    d.rectangle([12, 19, 20, 25], fill=(188, 214, 208, 255))
    d.line([(13, 15), (13, 27)], fill=(255, 255, 255, 255))
    return img


def draw_moisturizer(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # squeeze tube, crimped top, cap at base
    d.polygon([(12, 10), (20, 10), (19, 24), (13, 24)],
              fill=(240, 238, 232, 255))
    d.polygon([(12, 10), (20, 10), (19, 24), (13, 24)], outline=INK)
    d.line([(12, 10), (20, 10)], fill=WHITE_D)
    for x in (14, 17, 19):
        d.point((x, 10), fill=WHITE_D)
    d.rectangle([13, 24, 19, 28], fill=(168, 188, 200, 255))
    d.rectangle([13, 24, 19, 28], outline=INK)
    # cream dollop at the nozzle side
    d.ellipse([21, 22, 27, 27], fill=(248, 246, 240, 255))
    d.ellipse([23, 20, 26, 24], fill=(248, 246, 240, 255))
    return img


def draw_notch(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    # flat wooden board with a V notch, charred at the cut
    d.rectangle([6, 17, 26, 24], fill=WOOD)
    d.rectangle([6, 17, 26, 24], outline=INK)
    d.polygon([(13, 17), (19, 17), (16, 21)], fill=(0, 0, 0, 0))
    # redraw notch as transparent: punch via destination-out
    notch_img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    nd = ImageDraw.Draw(notch_img)
    nd.polygon([(13, 17), (19, 17), (16, 21)], fill=(0, 0, 0, 255))
    from PIL import ImageChops
    img = ImageChops.subtract(img, notch_img)
    d = ImageDraw.Draw(img)
    d.line([(13, 17), (16, 21), (19, 17)], fill=CHAR)
    d.ellipse([14, 19, 18, 22], fill=CHAR)
    d.point((16, 20), fill=(120, 60, 30, 255))  # ember glint
    for y in (19, 22):
        d.line([(7, y), (12, y)], fill=WOOD_D)
        d.line([(20, y), (25, y)], fill=WOOD_D)
    return img


def draw_filler(rng, granule, speckle):
    img = blank()
    d = ImageDraw.Draw(img)
    d.polygon([(10, 16), (22, 16), (21, 27), (11, 27)], fill=TRAY)
    d.polygon([(10, 16), (22, 16), (21, 27), (11, 27)], outline=INK)
    d.ellipse([10, 13, 22, 18], fill=granule)
    for _ in range(24):
        x = rng.randint(11, 21)
        y = rng.randint(14, 17)
        d.point((x, y), fill=speckle)
    d.ellipse([10, 13, 22, 18], outline=INK)
    return img


def draw_rubber_slug(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    slug = (72, 74, 76, 255)
    d.rectangle([11, 14, 21, 27], fill=slug)
    d.ellipse([11, 9, 21, 16], fill=(88, 90, 92, 255))
    d.ellipse([11, 9, 21, 16], outline=INK)
    d.line([(11, 14), (11, 27)], fill=INK)
    d.line([(21, 14), (21, 27)], fill=INK)
    d.line([(11, 27), (21, 27)], fill=INK)
    d.line([(11, 20), (21, 20)], fill=(52, 54, 56, 255))  # groove
    d.line([(13, 15), (13, 26)], fill=(104, 106, 108, 255))
    return img


def draw_teargas(rng):
    img = blank()
    d = ImageDraw.Draw(img)
    d.rectangle([11, 12, 19, 28], fill=(208, 210, 202, 255))
    d.rectangle([11, 12, 19, 28], outline=INK)
    d.rectangle([11, 18, 19, 21], fill=(178, 74, 56, 255))
    d.rectangle([12, 8, 18, 12], fill=(70, 72, 74, 255))  # actuator
    d.rectangle([17, 9, 21, 11], fill=(70, 72, 74, 255))  # nozzle
    # irritant puff
    puff = (206, 220, 200, 255)
    for x, y in ((23, 8), (25, 10), (22, 12), (26, 13), (24, 6),
                 (28, 10)):
        d.point((x, y), fill=puff)
        d.point((x + 1, y), fill=puff)
    return img


RENDER = {
    "123_steel_core": lambda r: draw_core(r, 3, 15),
    "3006_steel_core": lambda r: draw_core(r, 3, 18),
    "50_steel_core": lambda r: draw_core(r, 4, 20),
    "545_steel_core": lambda r: draw_core(r, 2, 14),
    "9x18_steel_core": lambda r: draw_core(r, 2, 11),
    "RTG_coffee": draw_rtg,
    "ampoule": draw_ampoule,
    "betavoltaic": draw_betavoltaic,
    "candle_wax": draw_wax,
    "ch_chain_link": lambda r: draw_chain(r, "ch"),
    "co2": draw_co2,
    "copper_wire": draw_copper_wire,
    "extinguishing_agent": lambda r: draw_white_heap(r, False),
    "flare_nitrate": lambda r: draw_white_heap(r, True),
    "gasfilter_m": draw_gasfilter_m,
    "gasfilter_s": draw_gasfilter_s,
    "glowstick_juice": draw_glowstick,
    "gunpowder_artillery": lambda r: draw_powder(r, "artillery"),
    "gunpowder_large_rifle": lambda r: draw_powder(r, "large_rifle"),
    "gunpowder_magnum_pistol": lambda r: draw_powder(r, "magnum"),
    "gunpowder_pistol": lambda r: draw_powder(r, "pistol"),
    "gunpowder_rifle": lambda r: draw_powder(r, "rifle"),
    "gunpowder_shotgun": lambda r: draw_powder(r, "shotgun"),
    "hc_chain_link": lambda r: draw_chain(r, "hc"),
    "hydrogen": lambda r: draw_gas_bottle(r, True,
                                          (168, 62, 52, 255), "h2"),
    "lamellae_ch": lambda r: draw_lamella(r, "ch"),
    "lamellae_lc": lambda r: draw_lamella(r, "lc"),
    "lc_chain_link": lambda r: draw_chain(r, "lc"),
    "lgpistol_primer": lambda r: draw_primer(r, 7, 7, False),
    "lgrifle_primer": lambda r: draw_primer(r, 7, 8, False),
    "liq_bandage": draw_liq_bandage,
    "mc_chain_link": lambda r: draw_chain(r, "mc"),
    "moisturizer": draw_moisturizer,
    "money": draw_money,
    "nitrogen": lambda r: draw_gas_bottle(r, False,
                                          (40, 40, 44, 255), "n2"),
    "nitrox": draw_nitrox,
    "notch": draw_notch,
    "qt_chain_link": lambda r: draw_chain(r, "qt"),
    "rebreather_filler_ko2": lambda r: draw_filler(
        r, (228, 202, 96, 255), (196, 164, 60, 255)),
    "rebreather_filler_soda_lime": lambda r: draw_filler(
        r, (236, 228, 222, 255), (219, 178, 178, 255)),
    "rubber_slug": draw_rubber_slug,
    "shotgun_primer": lambda r: draw_primer(r, 9, 9, False, True),
    "smpistol_primer": lambda r: draw_primer(r, 6, 6, False),
    "smrifle_primer": lambda r: draw_primer(r, 6, 7, False),
    "spent_lgpistol_primer": lambda r: draw_primer(r, 7, 7, True),
    "spent_lgrifle_primer": lambda r: draw_primer(r, 7, 8, True),
    "spent_shotgun_primer": lambda r: draw_primer(r, 9, 9, True, True),
    "spent_smpistol_primer": lambda r: draw_primer(r, 6, 6, True),
    "spent_smrifle_primer": lambda r: draw_primer(r, 6, 7, True),
    "stimpack_ammo": draw_stimpack,
    "teargas_spray": draw_teargas,
    "uran_cell": draw_uran,
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
    sheet = Image.new("RGBA", (cols * cell, rows * cell),
                      (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item28.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item28.png {sheet.size}")


if __name__ == "__main__":
    main()
