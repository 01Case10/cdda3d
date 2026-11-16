#!/usr/bin/env python3
"""ITEM30: chemicals and resources (50 ids).

Self-contained: the id list is embedded below. Draws each item as the
substance in the container/form its game entry describes, in the data
color. PIL only, per-sprite random.Random(crc32(id)), alpha in {0,255}.
Writes ONLY its own 50 folders + preview_item30.png.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA "
           "Voxel coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")

IDS = [
    "acetylene", "ammonia_hydroxide", "blood_acid", "blood_acid_plant",
    "blood_acid_underground", "butternut", "butternut_husk", "chem_agar",
    "chem_anfo", "chem_antimony_trichloride", "chem_apex", "chem_benzene",
    "chem_caco3", "chem_carbide", "chem_compositionb",
    "chem_ferric_chloride", "chem_hexamine", "chem_nickel_powder",
    "chem_potassium_chloride", "chem_potassium_hydroxide", "chem_rdx",
    "chem_slaked_lime", "chem_toluene", "cobalt_60", "conc_paralytic",
    "conc_venom", "exodii_ceramic_patch", "fermentable_fish_mixture_active",
    "fermentable_liquid_mixture", "fermented_fertilizer_liquid",
    "fert_supplement", "fertilizer_commercial", "flux_ash", "glue_strong",
    "glue_strong_metal", "glue_weak", "iodine_crystal", "liquid_soap",
    "lye_potassium", "material_niter", "material_styrofoam_chunk",
    "material_styrofoam_sheet", "peanut", "red_phosphorous_small",
    "rocuronium", "rosin", "silicone", "steel_grille", "uranium",
    "white_phosphorous",
]


def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)


def C(r, g, b):
    return (r, g, b, 255)


# ---------------------------------------------------------------- helpers

def powder_pile(d, rng, color, style="smooth"):
    """A heap of loose powder centered low in the tile."""
    base = shade(color, 0.82)
    d.polygon([(5, 27), (10, 18), (16, 12), (22, 18), (27, 27)], fill=base)
    d.polygon([(10, 18), (16, 12), (22, 18), (20, 24), (12, 24)], fill=color)
    d.line([(12, 20), (16, 14), (20, 20)], fill=shade(color, 1.12), width=1)
    for _ in range(26):
        x = rng.randint(8, 24)
        y = rng.randint(17, 26)
        if style == "sparkle" and rng.random() < 0.4:
            d.point((x, y), fill=C(255, 255, 255))
            if rng.random() < 0.5:
                d.point((x, y - 1), fill=shade(color, 1.15))
        elif style == "flake":
            d.line([(x, y), (x + 2, y)], fill=shade(color, 0.9))
        elif style == "chunk":
            d.rectangle([x, y, x + 1, y + 1], fill=shade(color, 0.62))
        else:
            d.point((x, y), fill=shade(color, rng.choice([0.88, 1.08])))


def plastic_bag(d, rng, fill_color, sparkle=False):
    """Small plastic bag, tied at the top, contents mounded at the bottom."""
    bag = C(206, 216, 222)
    edge = C(150, 165, 175)
    d.polygon([(13, 3), (19, 3), (20, 7), (12, 7)], fill=edge)  # knot
    d.polygon([(12, 7), (20, 7), (24, 16), (25, 27), (7, 27), (8, 16)],
              fill=bag)
    d.polygon([(9, 19), (23, 19), (25, 27), (7, 27)], fill=fill_color)
    for _ in range(14):
        x = rng.randint(10, 22)
        y = rng.randint(20, 26)
        if sparkle and rng.random() < 0.35:
            d.point((x, y), fill=C(255, 255, 255))
        else:
            d.point((x, y), fill=shade(fill_color, rng.choice([0.85, 1.1])))
    d.line([(10, 10), (9, 18)], fill=C(235, 242, 246))  # sheen


def canvas_sack(d, rng, fill_color, granules=True):
    """Open canvas sack with contents mounded above the rim."""
    sack = C(178, 148, 102)
    d.polygon([(8, 12), (24, 12), (26, 28), (6, 28)], fill=sack)
    d.polygon([(8, 12), (24, 12), (25, 20), (7, 20)], fill=shade(sack, 1.1))
    d.rectangle([7, 12, 25, 14], fill=shade(sack, 0.8))  # rim band
    d.ellipse([9, 6, 23, 14], fill=fill_color)
    if granules:
        for _ in range(22):
            x = rng.randint(10, 22)
            y = rng.randint(8, 13)
            d.point((x, y), fill=shade(fill_color, rng.choice([0.8, 1.12])))
    d.line([(10, 22), (22, 22)], fill=shade(sack, 0.75))


def glass_bottle(d, rng, liquid, cap="cork", sediment=False):
    """Glass bottle with colored liquid."""
    glass = C(168, 186, 178)
    d.rectangle([13, 4, 19, 9], fill=glass)                    # neck
    d.polygon([(13, 9), (19, 9), (23, 14), (23, 28), (9, 28), (9, 14)],
              fill=glass)
    d.polygon([(11, 15), (21, 15), (21, 27), (11, 27)], fill=liquid)
    d.line([(11, 15), (21, 15)], fill=shade(liquid, 1.2))
    if sediment:
        for _ in range(10):
            x = rng.randint(12, 20)
            d.point((x, rng.randint(25, 27)), fill=shade(liquid, 0.6))
    if cap == "cork":
        d.rectangle([13, 2, 19, 5], fill=C(178, 138, 92))
    else:
        d.rectangle([12, 2, 20, 5], fill=C(40, 40, 44))
    d.line([(10, 16), (10, 26)], fill=C(225, 238, 230))        # highlight


def small_plastic_bottle(d, rng, content, cap_color):
    """Small translucent plastic bottle, contents in the lower half."""
    body = C(214, 222, 226)
    d.rectangle([12, 8, 20, 10], fill=body)
    d.polygon([(12, 10), (20, 10), (22, 15), (22, 28), (10, 28), (10, 15)],
              fill=body)
    d.polygon([(11, 19), (21, 19), (21, 27), (11, 27)], fill=content)
    for _ in range(8):
        x = rng.randint(12, 20)
        d.point((x, rng.randint(20, 26)), fill=shade(content, 1.12))
    d.rectangle([11, 4, 21, 8], fill=cap_color)
    d.line([(11, 14), (11, 26)], fill=C(240, 246, 248))


def jug(d, rng, liquid, speck=None):
    """Plastic jug with handle and cap."""
    body = C(216, 220, 208)
    d.rectangle([10, 10, 24, 28], fill=body)
    d.rectangle([20, 5, 24, 10], fill=body)                    # neck
    d.rectangle([19, 3, 25, 6], fill=C(90, 110, 130))          # cap
    d.rectangle([12, 12, 16, 19], fill=C(120, 128, 118))       # handle hole
    d.rectangle([10, 15, 24, 27], fill=liquid)
    if speck:
        for _ in range(12):
            x = rng.randint(12, 23)
            y = rng.randint(17, 26)
            d.point((x, y), fill=speck)
    d.line([(11, 16), (11, 26)], fill=C(238, 242, 232))


def test_tube(d, rng, liquid, ticks=False):
    """Glass test tube with liquid."""
    glass = C(196, 208, 212)
    d.rectangle([12, 5, 20, 7], fill=glass)                    # rim
    d.rectangle([13, 7, 19, 24], fill=glass)
    d.ellipse([13, 22, 19, 28], fill=glass)
    d.rectangle([14, 14, 18, 24], fill=liquid)
    d.ellipse([14, 22, 18, 27], fill=liquid)
    d.line([(14, 14), (18, 14)], fill=shade(liquid, 1.18))
    if ticks:
        for y in (10, 13, 16):
            d.line([(17, y), (19, y)], fill=C(120, 130, 135))
    d.line([(14, 9), (14, 22)], fill=C(232, 240, 242))


def cardboard_box(d, rng, contents_fn=None, band=None):
    """Small open cardboard box."""
    box = C(172, 134, 86)
    d.polygon([(7, 14), (25, 14), (25, 28), (7, 28)], fill=box)
    d.polygon([(7, 14), (25, 14), (23, 18), (9, 18)], fill=shade(box, 0.8))
    d.polygon([(7, 14), (4, 10), (12, 10), (9, 14)], fill=shade(box, 1.12))
    d.polygon([(25, 14), (28, 10), (20, 10), (23, 14)], fill=shade(box, 1.12))
    if band:
        d.rectangle([7, 22, 25, 25], fill=band)
    if contents_fn:
        contents_fn(d, rng)


def trefoil(d, cx, cy, color, r=3):
    """Radiation trefoil."""
    import math
    for k in range(3):
        a = math.radians(-90 + 120 * k)
        x = int(cx + r * 1.6 * math.cos(a))
        y = int(cy + r * 1.6 * math.sin(a))
        d.pieslice([x - r, y - r, x + r, y + r],
                   start=math.degrees(a) - 30, end=math.degrees(a) + 30,
                   fill=color)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=color)


# ---------------------------------------------------------------- sprites

def s_acetylene(d, rng):
    red = C(178, 36, 30)
    d.rectangle([14, 3, 18, 6], fill=C(150, 150, 155))          # valve
    d.rectangle([12, 5, 20, 7], fill=C(120, 120, 125))          # guard ring
    d.ellipse([10, 6, 22, 13], fill=red)
    d.rectangle([10, 9, 22, 27], fill=red)
    d.ellipse([10, 24, 22, 29], fill=shade(red, 0.8))
    d.line([(12, 11), (12, 25)], fill=shade(red, 1.25))
    d.polygon([(16, 15), (19, 18), (16, 21), (13, 18)],        # hazard diamond
              fill=C(240, 200, 60))
    d.point((16, 18), fill=C(200, 60, 20))


def s_ammonia_hydroxide(d, rng):
    body = C(208, 224, 230)
    d.rectangle([13, 6, 19, 10], fill=body)
    d.polygon([(13, 10), (19, 10), (23, 15), (23, 28), (9, 28), (9, 15)],
              fill=body)
    d.polygon([(11, 17), (21, 17), (21, 27), (11, 27)], fill=C(196, 222, 232))
    d.rectangle([12, 3, 20, 6], fill=C(46, 92, 164))            # blue cap
    d.rectangle([11, 19, 21, 25], fill=C(240, 246, 248))        # label
    d.rectangle([11, 20, 21, 22], fill=C(90, 140, 205))
    d.line([(10, 16), (10, 26)], fill=C(238, 248, 250))


def s_blood_acid(d, rng):
    glass_bottle(d, rng, C(172, 30, 24), cap="cork")


def s_blood_acid_plant(d, rng):
    glass_bottle(d, rng, C(58, 138, 48), cap="cork", sediment=True)


def s_blood_acid_underground(d, rng):
    glass_bottle(d, rng, C(138, 24, 20), cap="cork", sediment=True)


def s_butternut(d, rng):
    nut = C(204, 172, 84)
    for (x, y, w, h) in [(8, 17, 10, 9), (17, 15, 10, 10), (12, 21, 11, 8),
                         (19, 22, 9, 7), (7, 23, 8, 6)]:
        d.ellipse([x, y, x + w, y + h], fill=nut)
        d.ellipse([x + 2, y + 2, x + w - 3, y + h - 3], fill=shade(nut, 1.1))
        d.point((x + w // 2, y), fill=shade(nut, 0.7))
        for _ in range(4):
            d.point((rng.randint(x + 1, x + w - 2), rng.randint(y + 1, y + h - 2)),
                    fill=shade(nut, 0.85))


def s_butternut_husk(d, rng):
    husk = C(168, 150, 70)
    inner = C(96, 82, 44)
    for (x, y) in [(7, 16), (16, 20), (12, 11)]:
        d.pieslice([x, y, x + 12, y + 11], start=300, end=200, fill=husk)
        d.pieslice([x + 2, y + 2, x + 10, y + 9], start=300, end=200, fill=inner)
    d.line([(9, 26), (24, 26)], fill=shade(husk, 0.8))


def s_chem_agar(d, rng):
    small_plastic_bottle(d, rng, C(226, 208, 130), C(70, 110, 170))
    for _ in range(6):  # flakes
        x = rng.randint(12, 19)
        y = rng.randint(15, 19)
        d.line([(x, y), (x + 2, y + 1)], fill=C(238, 226, 160))


def s_chem_anfo(d, rng):
    canvas_sack(d, rng, C(146, 98, 58))
    for (x, y) in [(11, 26), (22, 27), (19, 25)]:               # spilled pellets
        d.ellipse([x, y, x + 2, y + 2], fill=C(146, 98, 58))


def s_chem_antimony_trichloride(d, rng):
    plastic_bag(d, rng, C(240, 240, 236))


def s_chem_apex(d, rng):
    plastic_bag(d, rng, C(242, 242, 240), sparkle=True)


def s_chem_benzene(d, rng):
    glass_bottle(d, rng, C(198, 228, 234), cap="cork")


def s_chem_caco3(d, rng):
    powder_pile(d, rng, C(238, 236, 228), style="smooth")


def s_chem_carbide(d, rng):
    powder_pile(d, rng, C(226, 226, 218), style="chunk")


def s_chem_compositionb(d, rng):
    blk = C(142, 96, 54)
    d.polygon([(8, 16), (24, 16), (26, 27), (6, 27)], fill=blk)
    d.polygon([(8, 16), (24, 16), (22, 12), (10, 12)], fill=shade(blk, 1.18))
    for _ in range(16):
        x = rng.randint(9, 23)
        y = rng.randint(17, 26)
        d.point((x, y), fill=shade(blk, rng.choice([0.82, 1.1])))


def s_chem_ferric_chloride(d, rng):
    plastic_bag(d, rng, C(34, 92, 44), sparkle=True)


def s_chem_hexamine(d, rng):
    def tablets(dd, rr):
        for (x, y) in [(11, 9), (16, 8), (14, 11), (19, 10)]:
            dd.ellipse([x, y, x + 5, y + 4], fill=C(236, 226, 198))
            dd.ellipse([x + 1, y + 1, x + 4, y + 3], fill=C(246, 238, 214))
    cardboard_box(d, rng, contents_fn=tablets, band=C(178, 44, 36))


def s_chem_nickel_powder(d, rng):
    plastic_bag(d, rng, C(198, 200, 206), sparkle=True)


def s_chem_potassium_chloride(d, rng):
    powder_pile(d, rng, C(240, 240, 238), style="sparkle")


def s_chem_potassium_hydroxide(d, rng):
    powder_pile(d, rng, C(236, 236, 230), style="flake")


def s_chem_rdx(d, rng):
    powder_pile(d, rng, C(240, 242, 246), style="sparkle")


def s_chem_slaked_lime(d, rng):
    powder_pile(d, rng, C(242, 242, 238), style="smooth")


def s_chem_toluene(d, rng):
    glass_bottle(d, rng, C(202, 230, 236), cap="black")


def s_cobalt_60(d, rng):
    d.rectangle([12, 12, 20, 24], fill=C(148, 150, 156))
    d.ellipse([12, 9, 20, 14], fill=C(172, 174, 180))
    d.ellipse([12, 21, 20, 26], fill=C(120, 122, 128))
    trefoil(d, 16, 17, C(70, 205, 85), r=2)
    for (x, y) in [(8, 10), (24, 13), (9, 22), (23, 24)]:
        d.point((x, y), fill=C(90, 220, 105))


def s_conc_paralytic(d, rng):
    test_tube(d, rng, C(186, 186, 188))


def s_conc_venom(d, rng):
    test_tube(d, rng, C(152, 222, 140))


def s_exodii_ceramic_patch(d, rng):
    plate = C(78, 78, 84)
    d.polygon([(9, 9), (23, 9), (25, 12), (25, 24), (23, 27), (9, 27),
               (7, 24), (7, 12)], fill=plate)
    d.polygon([(10, 10), (22, 10), (23, 12), (23, 23), (22, 25), (10, 25),
               (9, 23), (9, 12)], fill=shade(plate, 1.12))
    d.line([(16, 10), (16, 25)], fill=shade(plate, 0.75))
    d.line([(9, 17), (23, 17)], fill=shade(plate, 0.75))
    for _ in range(18):
        x = rng.randint(10, 22)
        y = rng.randint(11, 24)
        d.point((x, y), fill=C(150, 155, 165))


def s_fermentable_fish(d, rng):
    jug(d, rng, C(214, 178, 58), speck=C(230, 200, 110))
    fish = C(110, 130, 150)
    d.ellipse([14, 19, 21, 23], fill=fish)                      # fish in the mix
    d.polygon([(20, 20), (24, 18), (24, 24), (20, 22)], fill=fish)
    d.point((16, 20), fill=C(30, 34, 40))


def s_fermentable_liquid(d, rng):
    jug(d, rng, C(220, 186, 68), speck=C(160, 130, 60))


def s_fermented_fertilizer(d, rng):
    jug(d, rng, C(198, 158, 52), speck=C(120, 96, 40))


def s_fert_supplement(d, rng):
    body = C(238, 238, 238)
    d.rectangle([11, 9, 21, 28], fill=body)
    d.rectangle([11, 9, 21, 13], fill=C(240, 148, 182))         # pink cap band
    d.rectangle([11, 16, 21, 23], fill=C(246, 196, 214))        # pink label
    for (x, y) in [(8, 26), (23, 27), (13, 27)]:
        d.ellipse([x, y, x + 3, y + 2], fill=C(240, 148, 182))  # pills


def s_fertilizer_commercial(d, rng):
    canvas_sack(d, rng, C(226, 190, 70))


def s_flux_ash(d, rng):
    powder_pile(d, rng, C(96, 96, 98), style="chunk")
    for _ in range(6):
        d.point((rng.randint(9, 23), rng.randint(20, 26)), fill=C(196, 170, 120))


def _glue_bottle(d, rng, body, nozzle, h=28):
    d.polygon([(13, 8), (19, 8), (21, 13), (21, h), (11, h), (11, 13)],
              fill=body)
    d.polygon([(14, 3), (18, 3), (19, 8), (13, 8)], fill=nozzle)
    d.point((16, 2), fill=nozzle)
    d.line([(12, 15), (12, h - 2)], fill=shade(body, 1.15))


def s_glue_strong(d, rng):
    _glue_bottle(d, rng, C(238, 238, 238), C(178, 44, 36))
    d.rectangle([11, 19, 21, 23], fill=C(120, 130, 150))


def s_glue_strong_metal(d, rng):
    body = C(168, 170, 176)
    d.polygon([(13, 9), (19, 9), (20, 14), (20, 28), (12, 28), (12, 14)],
              fill=body)
    d.polygon([(14, 4), (18, 4), (19, 9), (13, 9)], fill=C(90, 92, 98))
    for _ in range(10):  # steel flecks
        d.point((rng.randint(13, 19), rng.randint(15, 26)), fill=C(210, 212, 218))
    d.rectangle([12, 20, 20, 23], fill=C(100, 104, 112))


def s_glue_weak(d, rng):
    _glue_bottle(d, rng, C(240, 240, 240), C(232, 128, 40), h=25)
    d.rectangle([11, 16, 21, 20], fill=C(140, 180, 220))


def s_iodine_crystal(d, rng):
    def crystals(dd, rr):
        for (x, y, s) in [(10, 8, 5), (16, 7, 6), (20, 9, 4), (13, 11, 4)]:
            dd.polygon([(x, y + s), (x + s // 2, y), (x + s, y + s)],
                       fill=C(96, 84, 118))
            dd.line([(x + s // 2, y), (x + s // 2, y + s)], fill=C(150, 140, 175))
    cardboard_box(d, rng, contents_fn=crystals)


def s_liquid_soap(d, rng):
    body = C(186, 224, 232)
    d.rectangle([11, 11, 21, 28], fill=body)
    d.rectangle([11, 11, 21, 15], fill=shade(body, 0.92))
    d.rectangle([14, 7, 18, 11], fill=C(230, 234, 236))         # pump stem
    d.rectangle([14, 5, 23, 8], fill=C(230, 234, 236))          # pump head
    d.rectangle([21, 7, 24, 9], fill=C(230, 234, 236))          # nozzle
    d.line([(12, 17), (12, 26)], fill=C(228, 246, 250))
    for _ in range(5):  # bubbles
        d.point((rng.randint(13, 19), rng.randint(18, 25)), fill=C(240, 252, 254))


def s_lye_potassium(d, rng):
    glass_bottle(d, rng, C(232, 236, 236), cap="black")
    d.polygon([(16, 19), (19, 22), (16, 25), (13, 22)], fill=C(240, 200, 60))
    d.point((16, 22), fill=C(30, 30, 30))


def s_material_niter(d, rng):
    crys = C(240, 240, 246)
    for (x, y, w, h) in [(13, 6, 6, 20), (8, 12, 5, 14), (19, 11, 5, 15)]:
        d.polygon([(x, y + 4), (x + w // 2, y), (x + w, y + 4), (x + w, y + h),
                   (x, y + h)], fill=crys)
        d.line([(x + w // 2, y), (x + w // 2, y + h)], fill=C(196, 198, 210))
        d.line([(x + 1, y + 6), (x + 1, y + h - 2)], fill=C(255, 255, 255))
    d.rectangle([7, 25, 25, 27], fill=C(206, 208, 216))


def s_styrofoam_chunk(d, rng):
    foam = C(244, 244, 246)
    d.polygon([(9, 12), (20, 9), (25, 15), (23, 25), (12, 27), (7, 20)],
              fill=foam)
    for _ in range(30):
        x = rng.randint(9, 23)
        y = rng.randint(11, 25)
        d.point((x, y), fill=C(214, 214, 220))


def s_styrofoam_sheet(d, rng):
    foam = C(244, 244, 246)
    d.polygon([(4, 11), (27, 8), (28, 22), (5, 25)], fill=foam)
    d.polygon([(5, 25), (28, 22), (28, 24), (5, 27)], fill=C(206, 206, 212))
    for _ in range(36):
        x = rng.randint(6, 26)
        y = rng.randint(10, 22)
        d.point((x, y), fill=C(216, 216, 222))


def s_peanut(d, rng):
    pod = C(192, 152, 96)
    for (x, y) in [(8, 14), (16, 19), (11, 22)]:
        d.ellipse([x, y, x + 9, y + 6], fill=pod)
        d.ellipse([x + 3, y - 1, x + 7, y + 7], fill=pod)
        for _ in range(8):
            d.point((rng.randint(x + 1, x + 8), rng.randint(y + 1, y + 5)),
                    fill=shade(pod, 0.82))


def s_red_phosphorous_small(d, rng):
    small_plastic_bottle(d, rng, C(186, 44, 34), C(60, 60, 64))


def s_rocuronium(d, rng):
    test_tube(d, rng, C(190, 190, 192), ticks=True)


def s_rosin(d, rng):
    amber = C(226, 178, 58)
    d.polygon([(10, 12), (21, 10), (25, 17), (22, 26), (11, 27), (7, 19)],
              fill=amber)
    d.polygon([(12, 14), (20, 12), (22, 17), (13, 18)], fill=shade(amber, 1.15))
    d.line([(11, 22), (20, 24)], fill=shade(amber, 0.8))
    d.line([(13, 15), (16, 13)], fill=C(250, 226, 150))


def s_silicone(d, rng):
    body = C(172, 174, 178)
    d.polygon([(12, 12), (20, 12), (20, 27), (12, 27)], fill=body)
    d.polygon([(14, 12), (18, 12), (16, 5)], fill=C(200, 202, 206))  # nozzle
    d.rectangle([12, 25, 20, 28], fill=C(120, 122, 126))        # crimped end
    d.rectangle([12, 16, 20, 20], fill=C(140, 170, 200))        # label band
    d.ellipse([14, 3, 18, 6], fill=C(190, 192, 196))            # paste blob


def s_steel_grille(d, rng):
    frame = C(150, 152, 158)
    gap = C(38, 40, 44)
    d.rectangle([6, 6, 26, 26], fill=gap)
    for x in range(6, 27, 5):
        d.rectangle([x, 6, x + 2, 26], fill=frame)
    for y in range(6, 27, 5):
        d.rectangle([6, y, 26, y + 2], fill=frame)
    d.rectangle([6, 6, 26, 8], fill=shade(frame, 1.15))


def s_uranium(d, rng):
    rock = C(112, 112, 118)
    d.polygon([(9, 13), (20, 10), (25, 17), (22, 26), (11, 27), (7, 20)],
              fill=rock)
    d.polygon([(12, 14), (19, 12), (21, 17), (13, 19)], fill=shade(rock, 1.15))
    trefoil(d, 16, 20, C(84, 205, 95), r=2)
    for (x, y) in [(8, 9), (24, 11), (26, 22)]:
        d.point((x, y), fill=C(96, 220, 108))


def s_white_phosphorous(d, rng):
    powder_pile(d, rng, C(242, 240, 230), style="smooth")
    # faint smoke wisps (pale opaque gray curls)
    d.arc([13, 2, 19, 9], start=180, end=350, fill=C(178, 178, 182))
    d.arc([15, 5, 21, 12], start=190, end=340, fill=C(196, 196, 200))


DRAW = {
    "acetylene": s_acetylene,
    "ammonia_hydroxide": s_ammonia_hydroxide,
    "blood_acid": s_blood_acid,
    "blood_acid_plant": s_blood_acid_plant,
    "blood_acid_underground": s_blood_acid_underground,
    "butternut": s_butternut,
    "butternut_husk": s_butternut_husk,
    "chem_agar": s_chem_agar,
    "chem_anfo": s_chem_anfo,
    "chem_antimony_trichloride": s_chem_antimony_trichloride,
    "chem_apex": s_chem_apex,
    "chem_benzene": s_chem_benzene,
    "chem_caco3": s_chem_caco3,
    "chem_carbide": s_chem_carbide,
    "chem_compositionb": s_chem_compositionb,
    "chem_ferric_chloride": s_chem_ferric_chloride,
    "chem_hexamine": s_chem_hexamine,
    "chem_nickel_powder": s_chem_nickel_powder,
    "chem_potassium_chloride": s_chem_potassium_chloride,
    "chem_potassium_hydroxide": s_chem_potassium_hydroxide,
    "chem_rdx": s_chem_rdx,
    "chem_slaked_lime": s_chem_slaked_lime,
    "chem_toluene": s_chem_toluene,
    "cobalt_60": s_cobalt_60,
    "conc_paralytic": s_conc_paralytic,
    "conc_venom": s_conc_venom,
    "exodii_ceramic_patch": s_exodii_ceramic_patch,
    "fermentable_fish_mixture_active": s_fermentable_fish,
    "fermentable_liquid_mixture": s_fermentable_liquid,
    "fermented_fertilizer_liquid": s_fermented_fertilizer,
    "fert_supplement": s_fert_supplement,
    "fertilizer_commercial": s_fertilizer_commercial,
    "flux_ash": s_flux_ash,
    "glue_strong": s_glue_strong,
    "glue_strong_metal": s_glue_strong_metal,
    "glue_weak": s_glue_weak,
    "iodine_crystal": s_iodine_crystal,
    "liquid_soap": s_liquid_soap,
    "lye_potassium": s_lye_potassium,
    "material_niter": s_material_niter,
    "material_styrofoam_chunk": s_styrofoam_chunk,
    "material_styrofoam_sheet": s_styrofoam_sheet,
    "peanut": s_peanut,
    "red_phosphorous_small": s_red_phosphorous_small,
    "rocuronium": s_rocuronium,
    "rosin": s_rosin,
    "silicone": s_silicone,
    "steel_grille": s_steel_grille,
    "uranium": s_uranium,
    "white_phosphorous": s_white_phosphorous,
}


def main():
    assert sorted(DRAW) == sorted(IDS), "DRAW/IDS mismatch"
    sprites = {}
    for eid in IDS:
        rng = random.Random(zlib.crc32(eid.encode()))
        img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        DRAW[eid](d, rng)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (eid, alphas)
        assert img.size == (32, 32)
        sprites[eid] = img
        folder = os.path.join(BASE, eid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, eid + ".png"))
        with open(os.path.join(folder, eid + ".json"), "w") as f:
            json.dump([{"id": eid, "fg": eid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # contact-sheet preview
    cols, cell = 8, 36
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (38, 40, 46, 255))
    for i, eid in enumerate(IDS):
        x = (i % cols) * cell + 2
        y = (i // cols) * cell + 2
        sheet.alpha_composite(sprites[eid], (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item30.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item30.png {sheet.size}")


if __name__ == "__main__":
    main()
