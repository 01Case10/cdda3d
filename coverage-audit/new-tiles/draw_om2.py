#!/usr/bin/env python3
"""Batch OM2 (Phase 2g.4 scope extension): overmap terrain, second batch.

48 ids. Overmap treatment (same as OM1): map cells, so every sprite is
FULL-TILE OPAQUE 32x32 -- a ground/base fill in the location's data
color with a centered iconic pictogram, bold at map zoom. *_roof ids
are the roof plan of their base building (same footprint/palette,
roof texture). Families:

* cs_* shops: storefront pictograms; roofs as parapet slabs.
* debug_* : black tile, red border/glyph (project debug treatment).
* exo_/exodii: stone barns graded 0->2 (teal alien-tech seams grow
  with grade; _2 = roof plan), strange warehouses, rocky butte,
  scrap encampment, ground-level entrance.
* faction_base_*: one camp system -- palisade compound + function
  pictogram in the entry's data color; _2_0 = upgraded tier (stone/
  metal palisade, same icon). Mansion c/e/t components on lawn with
  hedge border and the room's pictogram (t3 is literally "mansion
  base???" in the data -- mansion + "?").

Game data (name/sym/color, copy-from resolved):
  cs_car_showroom_roof 'c' i_cyan; cs_gardening_allotment (city_
  building; roof 'a' brown_green); cs_internet_cafe (city_building);
  cs_market_small (city_building; roof 'm' i_light_gray);
  cs_tire_shop (city_building; roof 'O' white);
  debug_itemgroup_test 'T' light_red; debug_ramps '%' white;
  derelict_property_roof '.' i_brown; dirt_road_farm_parking '|'
  brown; dirtroad1_aban1 'forest' 'F' green; drug_lab_roof '^' brown;
  exo safehouses 'unusual stone barn' '^' brown; exo_warehouse '#'
  light_gray; exodii butte '%' light_gray, scrap encampment,
  entrance '?' white; faction surveys '+' in per-function colors;
  mansion 'M' in per-room colors.

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is
255 everywhere (asserted). Writes ONLY its own 48 folders +
preview_om2.png.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "overmap")
CREDITS = ("Drawn from the overmap terrain's in-game description for the "
           "CDDA Voxel coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")

INK = (238, 238, 228, 255)
SHADE = (26, 28, 32, 255)


# ---------------------------------------------------------------- helpers

def base_img(rng, fill, speckles, n=110):
    img = Image.new("RGBA", (32, 32), fill)
    d = ImageDraw.Draw(img)
    for _ in range(n):
        x, y = rng.randrange(32), rng.randrange(32)
        d.point((x, y), fill=rng.choice(speckles))
    return img, d


def speckle(d, rng, colors, n, box=(0, 0, 31, 31)):
    for _ in range(n):
        x = rng.randrange(box[0], box[2] + 1)
        y = rng.randrange(box[1], box[3] + 1)
        d.point((x, y), fill=rng.choice(colors))


def roof_slab(d, rng, slab, edge, line, broken=False):
    """Parapet roof plan: edge border, slab, texture lines, vent."""
    d.rectangle([0, 0, 31, 31], fill=slab)
    speckle(d, rng, [line], 60)
    for y in range(5, 31, 6):
        d.line([(2, y), (29, y)], fill=line)
    d.rectangle([0, 0, 31, 31], outline=edge, width=2)
    d.rectangle([22, 5, 27, 10], fill=edge)          # vent box
    d.rectangle([23, 6, 26, 9], fill=slab)
    if broken:
        d.polygon([(3, 20), (10, 18), (12, 26), (5, 29), (2, 25)],
                  fill=(24, 18, 14, 255))
        d.polygon([(18, 24), (24, 23), (25, 29), (19, 29)],
                  fill=(24, 18, 14, 255))
        speckle(d, rng, [(90, 110, 50, 255)], 26)    # moss


def palisade(d, rng, tier):
    """Compound border: tier 0 = wood posts, tier 2 = stone/metal."""
    if tier == 0:
        posts = [(122, 88, 50, 255), (140, 104, 62, 255), (104, 72, 40, 255)]
        for x in range(0, 32, 3):
            c = rng.choice(posts)
            d.rectangle([x, 0, x + 1, 3], fill=c)
            d.rectangle([x, 28, x + 1, 31], fill=c)
        for y in range(0, 32, 3):
            c = rng.choice(posts)
            d.rectangle([0, y, 3, y + 1], fill=c)
            d.rectangle([28, y, 31, y + 1], fill=c)
    else:
        d.rectangle([0, 0, 31, 31], outline=(128, 132, 138, 255), width=3)
        d.rectangle([1, 1, 30, 30], outline=(168, 172, 178, 255), width=1)
        for x in range(2, 31, 7):
            d.rectangle([x, 0, x + 1, 2], fill=(96, 100, 106, 255))
            d.rectangle([x, 29, x + 1, 31], fill=(96, 100, 106, 255))


def survey_base(rng, tier):
    img, d = base_img(rng, (146, 124, 84, 255),
                      [(158, 138, 96, 255), (132, 110, 72, 255),
                       (120, 128, 70, 255)], 90)
    palisade(d, rng, tier)
    return img, d


# ---------------------------------------------------------------- shops

def d_showroom_roof(rng):
    img, d = base_img(rng, (168, 172, 176, 255), [(158, 162, 166, 255)], 40)
    roof_slab(d, rng, (168, 172, 176, 255), (110, 116, 122, 255),
              (150, 154, 158, 255))
    # cyan glass skylight band (data: i_cyan, sym 'c')
    d.rectangle([5, 13, 26, 19], fill=(24, 140, 158, 255))
    d.rectangle([6, 14, 25, 18], fill=(64, 205, 226, 255))
    d.line([(6, 16), (25, 16)], fill=(24, 140, 158, 255))
    return img


def d_allotment(rng):
    img, d = base_img(rng, (104, 122, 58, 255),
                      [(94, 110, 50, 255), (116, 134, 66, 255)], 60)
    # 3x2 soil plots with crop rows
    for r in range(2):
        for c in range(3):
            x0, y0 = 3 + c * 9, 4 + r * 13
            d.rectangle([x0, y0, x0 + 7, y0 + 9], fill=(96, 68, 40, 255))
            for yy in range(y0 + 2, y0 + 9, 3):
                d.line([(x0 + 1, yy), (x0 + 6, yy)], fill=(70, 160, 60, 255))
    return img


def d_allotment_roof(rng):
    # greenhouse roof: brown-green frame, pale glass panels
    img, d = base_img(rng, (107, 122, 60, 255), [(96, 110, 52, 255)], 30)
    d.rectangle([0, 0, 31, 31], outline=(84, 96, 44, 255), width=2)
    for r in range(3):
        for c in range(3):
            x0, y0 = 3 + c * 9, 3 + r * 9
            d.rectangle([x0, y0, x0 + 7, y0 + 7], fill=(196, 224, 178, 255))
            d.line([(x0, y0 + 7), (x0 + 7, y0)], fill=(226, 240, 210, 255))
    return img


def d_internet_cafe(rng):
    img, d = base_img(rng, (128, 128, 132, 255),
                      [(118, 118, 122, 255), (138, 138, 142, 255)], 50)
    d.rectangle([5, 7, 26, 25], fill=(168, 164, 156, 255))     # building
    d.rectangle([5, 7, 26, 11], fill=(70, 130, 170, 255))      # sign band
    # monitor pictogram
    d.rectangle([11, 14, 20, 21], fill=SHADE)
    d.rectangle([12, 15, 19, 20], fill=(120, 220, 235, 255))
    d.rectangle([14, 22, 17, 23], fill=SHADE)
    d.line([(10, 24), (21, 24)], fill=SHADE)
    # wifi arcs
    for r in (3, 5):
        d.arc([16 - r, 4 - r + 6, 16 + r, 4 + r + 6], start=200, end=340,
              fill=INK)
    return img


def d_market(rng):
    img, d = base_img(rng, (122, 124, 118, 255),
                      [(112, 114, 108, 255), (134, 136, 130, 255)], 50)
    d.rectangle([5, 9, 26, 26], fill=(188, 182, 168, 255))     # storefront
    for i in range(5):                                          # awning
        c = (196, 60, 50, 255) if i % 2 == 0 else INK
        d.rectangle([5 + i * 4 + (1 if i else 0), 9, 8 + i * 4 + (1 if i < 4 else 0), 12], fill=c)
    d.rectangle([8, 16, 13, 21], fill=(90, 140, 190, 255))     # window
    d.rectangle([19, 16, 23, 26], fill=(96, 68, 40, 255))      # door
    d.rectangle([14, 22, 18, 26], fill=(150, 110, 60, 255))    # crate
    d.line([(14, 24), (18, 24)], fill=(120, 84, 44, 255))
    return img


def d_market_roof(rng):
    img, d = base_img(rng, (176, 178, 180, 255), [(166, 168, 170, 255)], 30)
    roof_slab(d, rng, (176, 178, 180, 255), (120, 122, 126, 255),
              (158, 160, 164, 255))
    d.rectangle([5, 20, 12, 26], fill=(120, 122, 126, 255))    # hatch
    d.rectangle([6, 21, 11, 25], fill=(150, 152, 156, 255))
    return img


def d_tire_shop(rng):
    img, d = base_img(rng, (120, 120, 124, 255),
                      [(110, 110, 114, 255), (132, 132, 136, 255)], 50)
    d.rectangle([4, 5, 27, 12], fill=(150, 146, 140, 255))     # shop band
    d.rectangle([6, 7, 12, 10], fill=(70, 130, 170, 255))
    # big tire (data sym 'O')
    d.ellipse([8, 13, 23, 28], fill=(30, 30, 34, 255))
    d.ellipse([11, 16, 20, 25], fill=(120, 120, 124, 255))
    d.ellipse([13, 18, 18, 23], fill=(70, 70, 76, 255))
    # small stacked tires
    d.ellipse([24, 20, 29, 24], fill=(30, 30, 34, 255))
    d.ellipse([24, 24, 29, 28], fill=(38, 38, 42, 255))
    return img


def d_tire_roof(rng):
    img, d = base_img(rng, (206, 206, 208, 255), [(196, 196, 198, 255)], 30)
    roof_slab(d, rng, (206, 206, 208, 255), (150, 150, 154, 255),
              (188, 188, 190, 255))
    d.rectangle([5, 5, 13, 12], fill=(170, 200, 215, 255))     # skylight
    d.rectangle([5, 5, 13, 12], outline=(120, 140, 155, 255))
    return img


# ---------------------------------------------------------------- misc

def d_debug(rng, kind):
    img = Image.new("RGBA", (32, 32), (8, 8, 10, 255))
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, 31, 31], outline=(220, 40, 40, 255), width=2)
    red = (235, 60, 60, 255)
    if kind == "T":
        d.rectangle([10, 9, 21, 12], fill=red)
        d.rectangle([14, 9, 17, 23], fill=red)
    else:  # ramps: two ramp triangles + % dots
        d.polygon([(5, 23), (14, 23), (14, 14)], outline=red, width=2)
        d.polygon([(18, 23), (27, 23), (18, 14)], outline=red, width=2)
        d.ellipse([13, 6, 16, 9], outline=red)
        d.ellipse([17, 10, 20, 13], outline=red)
    return img


def d_derelict_roof(rng):
    img, d = base_img(rng, (122, 90, 56, 255), [(110, 80, 48, 255)], 20)
    roof_slab(d, rng, (122, 90, 56, 255), (84, 60, 36, 255),
              (104, 76, 46, 255), broken=True)
    d.line([(4, 4), (14, 14)], fill=(70, 50, 30, 255), width=2)  # crack
    return img


def d_farm_parking(rng):
    img, d = base_img(rng, (96, 130, 58, 255),
                      [(86, 118, 50, 255), (108, 142, 66, 255)], 80)
    d.rectangle([13, 0, 19, 31], fill=(146, 110, 66, 255))     # dirt road |
    d.line([(14, 0), (14, 31)], fill=(120, 88, 50, 255))
    d.line([(18, 0), (18, 31)], fill=(120, 88, 50, 255))
    d.rectangle([19, 10, 29, 21], fill=(152, 116, 70, 255))    # parking lot
    # parked pickup
    d.rectangle([21, 12, 28, 16], fill=(150, 60, 50, 255))
    d.rectangle([25, 13, 27, 15], fill=(150, 200, 220, 255))
    d.rectangle([21, 17, 26, 20], fill=(90, 110, 130, 255))
    return img


def d_dirtroad_aban(rng):
    img, d = base_img(rng, (46, 96, 40, 255),
                      [(38, 84, 34, 255), (56, 110, 48, 255),
                       (66, 122, 54, 255)], 190)
    # overgrown dirt track, diagonal, broken by regrowth
    for t in range(0, 30, 2):
        if t in (10, 11, 20):
            continue
        x = 4 + t // 2
        d.rectangle([x, 29 - t, x + 3, 30 - t], fill=(140, 112, 70, 255))
    speckle(d, rng, [(56, 110, 48, 255)], 20)
    return img


def d_drug_lab_roof(rng):
    img, d = base_img(rng, (130, 98, 60, 255), [(118, 88, 52, 255)], 20)
    roof_slab(d, rng, (130, 98, 60, 255), (92, 68, 40, 255),
              (112, 84, 50, 255))
    for cx, cy in ((9, 9), (14, 22)):                            # vent stacks
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 3], fill=(105, 108, 112, 255))
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(60, 62, 66, 255))
    d.rectangle([20, 18, 27, 25], fill=(96, 130, 90, 255))     # chem tank
    d.rectangle([20, 18, 27, 20], fill=(120, 155, 112, 255))
    return img


# ---------------------------------------------------------------- exo

def stone_barn(rng, tech, roof, emblem):
    if roof:
        img, d = base_img(rng, (128, 116, 100, 255),
                          [(118, 106, 90, 255)], 30)
        d.rectangle([0, 0, 31, 31], fill=(128, 116, 100, 255))
        speckle(d, rng, [(112, 100, 84, 255)], 50)
        for y in range(6, 31, 7):
            d.line([(2, y), (29, y)], fill=(104, 92, 74, 255))
        d.line([(15, 1), (15, 30)], fill=(88, 76, 60, 255), width=2)  # ridge
        d.rectangle([0, 0, 31, 31], outline=(92, 80, 64, 255), width=2)
    else:
        img, d = base_img(rng, (110, 96, 74, 255),
                          [(100, 86, 64, 255), (122, 108, 84, 255)], 70)
        # stone gable barn silhouette
        d.polygon([(7, 13), (15, 5), (24, 13), (24, 26), (7, 26)],
                  fill=(142, 132, 116, 255))
        d.polygon([(6, 13), (15, 4), (25, 13), (22, 13), (15, 7), (9, 13)],
                  fill=(96, 70, 44, 255))
        for y in range(16, 26, 3):                              # stone courses
            d.line([(8, y), (23, y)], fill=(122, 112, 96, 255))
        d.rectangle([13, 19, 18, 26], fill=(70, 52, 34, 255))  # door
        d.line([(13, 19), (18, 26)], fill=(88, 66, 42, 255))
        d.line([(18, 19), (13, 26)], fill=(88, 66, 42, 255))
    if tech >= 1:  # alien-tech seams / glow nodes grow with grade
        teal = (64, 230, 208, 255)
        pts = [(4, 4), (27, 4), (4, 27), (27, 27), (15, 15)][:2 + tech * 2]
        for x, y in pts:
            d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=teal)
        if tech >= 2 and not roof:
            d.line([(15, 4), (15, 1)], fill=teal, width=1)     # antenna
            d.point((15, 0), fill=teal)
    if emblem:
        d.polygon([(15, 8), (18, 11), (15, 14), (12, 11)],
                  outline=(64, 230, 208, 255))
    return img


def d_warehouse(rng, upper):
    img, d = base_img(rng, (150, 150, 154, 255),
                      [(140, 140, 144, 255)], 40)
    d.rectangle([3, 4, 28, 27], fill=(172, 172, 176, 255))
    d.rectangle([3, 4, 28, 27], outline=(104, 104, 108, 255), width=2)
    if upper:  # roof level: vent rows + hatch
        for x in (8, 15, 22):
            d.rectangle([x - 2, 8, x + 2, 12], fill=(120, 122, 126, 255))
        d.rectangle([11, 18, 20, 25], outline=(104, 104, 108, 255), width=2)
        d.line([(11, 21), (20, 21)], fill=(104, 104, 108, 255))
    else:      # ground level: '#' grid paneling + door
        for x in range(7, 28, 5):
            d.line([(x, 6), (x, 25)], fill=(140, 140, 144, 255))
        for y in range(9, 27, 5):
            d.line([(5, y), (26, y)], fill=(140, 140, 144, 255))
        d.rectangle([13, 19, 18, 27], fill=(88, 88, 92, 255))
    # alien tinge: faint teal corner nodes
    for x, y in ((3, 4), (28, 4), (3, 27), (28, 27)):
        d.point((x, y), fill=(64, 200, 190, 255))
    return img


def d_butte(rng):
    img, d = base_img(rng, (168, 148, 116, 255),
                      [(158, 136, 104, 255), (178, 160, 128, 255)], 70)
    d.polygon([(8, 26), (11, 10), (21, 10), (24, 26)], fill=(150, 148, 150, 255))
    d.polygon([(11, 10), (21, 10), (22, 14), (10, 14)], fill=(172, 170, 172, 255))
    for y in (17, 21, 24):
        d.line([(9 + (y - 14) // 6, y), (23 - (y - 14) // 6, y)],
               fill=(128, 126, 128, 255))
    speckle(d, rng, [(120, 118, 120, 255)], 14, (8, 10, 24, 26))
    return img


def d_scrap_camp(rng):
    img, d = base_img(rng, (138, 122, 96, 255),
                      [(128, 112, 86, 255), (148, 134, 108, 255)], 80)
    # scrap piles
    for cx, cy in ((8, 9), (24, 7), (7, 24)):
        d.polygon([(cx - 4, cy + 3), (cx, cy - 3), (cx + 4, cy + 3)],
                  fill=(150, 150, 154, 255))
        d.line([(cx - 2, cy + 1), (cx + 2, cy + 1)], fill=(110, 110, 114, 255))
    # tents
    for cx, cy in ((17, 17), (25, 23)):
        d.polygon([(cx - 5, cy + 4), (cx, cy - 4), (cx + 5, cy + 4)],
                  fill=(140, 110, 70, 255))
        d.polygon([(cx - 1, cy + 4), (cx, cy), (cx + 1, cy + 4)],
                  fill=(80, 60, 38, 255))
    d.ellipse([13, 24, 17, 27], fill=(70, 62, 54, 255))       # fire pit
    d.point((15, 25), fill=(240, 140, 50, 255))
    return img


def d_entrance(rng):
    img, d = base_img(rng, (146, 144, 146, 255),
                      [(134, 132, 134, 255), (158, 156, 158, 255)], 60)
    for y in (6, 13, 20, 27):                                   # strata
        d.line([(0, y), (31, y)], fill=(122, 120, 122, 255))
    d.rectangle([11, 12, 20, 27], fill=(30, 28, 30, 255))      # doorway
    d.rectangle([11, 12, 20, 27], outline=(90, 88, 90, 255), width=1)
    # white '?' marker (data sym '?', color white)
    d.rectangle([14, 3, 17, 4], fill=INK)
    d.rectangle([17, 4, 18, 6], fill=INK)
    d.rectangle([15, 6, 16, 8], fill=INK)
    d.point((15, 10), fill=INK)
    return img


# ---------------------------------------------------------------- faction

def p_anvil(d, c):
    d.rectangle([10, 12, 22, 15], fill=c)
    d.polygon([(10, 12), (5, 14), (10, 15)], fill=c)
    d.rectangle([14, 15, 18, 19], fill=c)
    d.rectangle([11, 19, 21, 22], fill=c)


def p_tent(d, c):
    d.polygon([(6, 23), (16, 9), (26, 23)], fill=c)
    d.polygon([(13, 23), (16, 15), (19, 23)], fill=SHADE)
    d.line([(16, 9), (16, 6)], fill=c)


def p_canteen(d, c):
    d.rectangle([8, 19, 24, 22], fill=c)                        # tray
    d.pieslice([10, 11, 22, 21], start=180, end=360, fill=c)    # cloche
    d.rectangle([15, 9, 17, 11], fill=c)
    d.line([(6, 24), (26, 24)], fill=c)


def p_farm(d, c):
    for i, x in enumerate((8, 13, 18, 23)):
        d.rectangle([x, 9, x + 2, 23], fill=(96, 68, 40, 255))
        for y in range(10, 23, 4):
            d.ellipse([x, y, x + 2, y + 2], fill=c)


def p_garage(d, c, helipad=False):
    if helipad:
        d.ellipse([7, 6, 25, 24], outline=(228, 60, 50, 255), width=2)
        d.rectangle([13, 10, 15, 20], fill=INK)
        d.rectangle([17, 10, 19, 20], fill=INK)
        d.rectangle([13, 14, 19, 16], fill=INK)
    d.rectangle([8, 21, 24, 26], fill=c)                        # garage block
    d.rectangle([11, 22, 21, 26], fill=SHADE)
    for y in (23, 24, 25):
        d.line([(11, y), (21, y)], fill=c)


def p_pot(d, c):
    d.rectangle([10, 13, 22, 21], fill=c)
    d.rectangle([8, 12, 24, 14], fill=c)
    d.rectangle([14, 9, 18, 11], fill=c)
    d.line([(10, 17), (6, 15)], fill=c)
    d.line([(22, 17), (26, 15)], fill=c)
    for x in (12, 16, 20):                                      # steam
        d.arc([x - 2, 3, x + 2, 8], start=270, end=90, fill=INK)


def p_factory(d, c):
    d.rectangle([7, 16, 25, 24], fill=c)
    d.polygon([(7, 16), (13, 12), (19, 16), (25, 12), (25, 16)], fill=c)
    d.rectangle([20, 6, 23, 16], fill=c)                        # chimney
    d.rectangle([10, 19, 13, 22], fill=SHADE)
    d.rectangle([16, 19, 19, 22], fill=SHADE)


def p_livestock(d, c):
    for x in range(7, 26, 5):                                   # pen fence
        d.rectangle([x, 9, x + 1, 23], fill=(150, 116, 74, 255))
    d.line([(6, 13), (26, 13)], fill=(150, 116, 74, 255))
    d.line([(6, 19), (26, 19)], fill=(150, 116, 74, 255))
    d.rectangle([11, 13, 20, 19], fill=c)                       # animal body
    d.rectangle([19, 11, 23, 15], fill=c)                       # head
    d.point((21, 12), fill=SHADE)


def survey(rng, tier, pictogram, accent):
    img, d = survey_base(rng, tier)
    pictogram(d, accent)
    return img


def mansion(rng, accent, kind):
    img, d = base_img(rng, (86, 128, 58, 255),
                      [(76, 116, 50, 255), (98, 140, 66, 255)], 70)
    d.rectangle([0, 0, 31, 31], outline=(44, 84, 36, 255), width=3)  # hedge
    d.rectangle([6, 6, 25, 25], fill=(196, 188, 172, 255))     # walls
    d.rectangle([6, 6, 25, 25], outline=(120, 110, 96, 255), width=1)
    if kind == "pool":
        d.rectangle([9, 10, 22, 21], fill=(52, 110, 205, 255))
        for y in (13, 17):
            d.arc([10, y, 15, y + 3], start=0, end=180, fill=(140, 190, 240, 255))
            d.arc([16, y, 21, y + 3], start=180, end=360, fill=(140, 190, 240, 255))
    elif kind == "bar":
        d.rectangle([11, 9, 14, 19], fill=(150, 40, 130, 255))  # bottle
        d.rectangle([12, 7, 13, 9], fill=(150, 40, 130, 255))
        d.polygon([(17, 12), (22, 12), (19, 17)], outline=accent)  # glass
        d.line([(19, 17), (19, 21)], fill=accent)
        d.line([(16, 21), (22, 21)], fill=accent)
    elif kind == "living":
        d.rectangle([9, 15, 22, 21], fill=accent)               # sofa
        d.rectangle([9, 12, 12, 18], fill=accent)
        d.rectangle([19, 12, 22, 18], fill=accent)
        d.line([(12, 18), (19, 18)], fill=(60, 130, 50, 255))
    elif kind == "bed":
        d.rectangle([9, 10, 22, 21], fill=(120, 100, 80, 255))  # frame
        d.rectangle([10, 11, 21, 15], fill=INK)                 # pillow zone
        d.rectangle([10, 15, 21, 20], fill=accent)              # blanket
    elif kind == "beds":
        for x0 in (8, 17):
            d.rectangle([x0, 10, x0 + 6, 21], fill=(120, 100, 80, 255))
            d.rectangle([x0 + 1, 11, x0 + 5, 14], fill=INK)
            d.rectangle([x0 + 1, 14, x0 + 5, 20], fill=accent)
    elif kind == "kitchen":
        d.rectangle([9, 13, 22, 21], fill=(150, 150, 154, 255))  # stove block
        d.ellipse([11, 15, 15, 19], outline=SHADE)
        d.ellipse([17, 15, 21, 19], outline=SHADE)
        d.rectangle([13, 8, 19, 12], fill=accent)               # pot
    elif kind == "entrance":
        for x in (9, 14, 20):                                   # columns
            d.rectangle([x, 9, x + 2, 21], fill=INK)
        d.rectangle([7, 7, 24, 9], fill=accent)                 # pediment
        d.rectangle([13, 15, 18, 21], fill=(96, 68, 40, 255))  # door
    elif kind == "unknown":
        d.rectangle([13, 8, 18, 10], fill=accent)
        d.rectangle([17, 10, 19, 14], fill=accent)
        d.rectangle([14, 14, 17, 17], fill=accent)
        d.point((15, 20), fill=accent)
        d.rectangle([15, 20, 16, 21], fill=accent)
    return img


# ---------------------------------------------------------------- dispatch

def build(eid):
    rng = random.Random(zlib.crc32(eid.encode()))
    dark_gray = (92, 92, 96, 255)
    green_ac = (84, 190, 84, 255)
    brown_ac = (168, 118, 66, 255)
    lgray_ac = (186, 186, 190, 255)
    pink_ac = (238, 158, 182, 255)
    cyan_ac = (70, 205, 215, 255)
    table = {
        "cs_car_showroom_roof": lambda: d_showroom_roof(rng),
        "cs_gardening_allotment": lambda: d_allotment(rng),
        "cs_gardening_allotment_roof": lambda: d_allotment_roof(rng),
        "cs_internet_cafe": lambda: d_internet_cafe(rng),
        "cs_market_small": lambda: d_market(rng),
        "cs_market_small_roof": lambda: d_market_roof(rng),
        "cs_tire_shop": lambda: d_tire_shop(rng),
        "cs_tire_shop_roof": lambda: d_tire_roof(rng),
        "debug_itemgroup_test": lambda: d_debug(rng, "T"),
        "debug_ramps": lambda: d_debug(rng, "ramps"),
        "derelict_property_roof": lambda: d_derelict_roof(rng),
        "dirt_road_farm_parking": lambda: d_farm_parking(rng),
        "dirtroad1_aban1": lambda: d_dirtroad_aban(rng),
        "drug_lab_roof": lambda: d_drug_lab_roof(rng),
        "exo_base_safehouse_stone_0": lambda: stone_barn(rng, 0, False, True),
        "exo_base_safehouse_stone_1": lambda: stone_barn(rng, 1, False, True),
        "exo_base_safehouse_stone_2": lambda: stone_barn(rng, 1, True, True),
        "exo_safehouse_stone_0": lambda: stone_barn(rng, 0, False, False),
        "exo_safehouse_stone_1": lambda: stone_barn(rng, 1, False, False),
        "exo_safehouse_stone_2": lambda: stone_barn(rng, 1, True, False),
        "exo_warehouse_1": lambda: d_warehouse(rng, False),
        "exo_warehouse_2": lambda: d_warehouse(rng, True),
        "exodii_base_x0y0z0": lambda: d_butte(rng),
        "exodii_base_x0y0z1": lambda: d_scrap_camp(rng),
        "exodii_base_x0y3z0": lambda: d_entrance(rng),
        "faction_base_blacksmith_0": lambda: survey(rng, 0, p_anvil, dark_gray),
        "faction_base_camp_0": lambda: survey(rng, 0, p_tent, INK),
        "faction_base_canteen_0": lambda: survey(rng, 0, p_canteen, green_ac),
        "faction_base_canteen_2_0": lambda: survey(rng, 2, p_canteen, green_ac),
        "faction_base_farm_0": lambda: survey(rng, 0, p_farm, green_ac),
        "faction_base_farm_2_0": lambda: survey(rng, 2, p_farm, green_ac),
        "faction_base_garage_0": lambda: survey(rng, 0, p_garage, lgray_ac),
        "faction_base_garage_2_0": lambda: survey(rng, 2, p_garage, lgray_ac),
        "faction_base_helipad_garage_0": lambda: survey(
            rng, 0, lambda d, c: p_garage(d, c, helipad=True), lgray_ac),
        "faction_base_kitchen_0": lambda: survey(rng, 0, p_pot, green_ac),
        "faction_base_lightindustry_workshop_0": lambda: survey(
            rng, 0, p_factory, cyan_ac),
        "faction_base_livestock_0": lambda: survey(rng, 0, p_livestock, pink_ac),
        "faction_base_livestock_2_0": lambda: survey(rng, 2, p_livestock, pink_ac),
        "faction_base_mansion_c1": lambda: mansion(rng, (52, 110, 205, 255), "pool"),
        "faction_base_mansion_c2": lambda: mansion(rng, (215, 70, 195, 255), "bar"),
        "faction_base_mansion_c3": lambda: mansion(rng, (70, 165, 70, 255), "living"),
        "faction_base_mansion_c4": lambda: mansion(rng, (140, 225, 140, 255), "bed"),
        "faction_base_mansion_c5": lambda: mansion(rng, (238, 158, 182, 255), "kitchen"),
        "faction_base_mansion_e1": lambda: mansion(rng, INK, "entrance"),
        "faction_base_mansion_e2": lambda: mansion(rng, INK, "entrance"),
        "faction_base_mansion_t1": lambda: mansion(rng, (52, 110, 205, 255), "pool"),
        "faction_base_mansion_t2": lambda: mansion(rng, (140, 225, 140, 255), "beds"),
        "faction_base_mansion_t3": lambda: mansion(rng, INK, "unknown"),
    }
    img = table[eid]()
    alphas = set(img.getchannel("A").getdata())
    assert alphas == {255}, (eid, alphas)
    return img


IDS = [
    "cs_car_showroom_roof", "cs_gardening_allotment",
    "cs_gardening_allotment_roof", "cs_internet_cafe", "cs_market_small",
    "cs_market_small_roof", "cs_tire_shop", "cs_tire_shop_roof",
    "debug_itemgroup_test", "debug_ramps", "derelict_property_roof",
    "dirt_road_farm_parking", "dirtroad1_aban1", "drug_lab_roof",
    "exo_base_safehouse_stone_0", "exo_base_safehouse_stone_1",
    "exo_base_safehouse_stone_2", "exo_safehouse_stone_0",
    "exo_safehouse_stone_1", "exo_safehouse_stone_2", "exo_warehouse_1",
    "exo_warehouse_2", "exodii_base_x0y0z0", "exodii_base_x0y0z1",
    "exodii_base_x0y3z0", "faction_base_blacksmith_0",
    "faction_base_camp_0", "faction_base_canteen_0",
    "faction_base_canteen_2_0", "faction_base_farm_0",
    "faction_base_farm_2_0", "faction_base_garage_0",
    "faction_base_garage_2_0", "faction_base_helipad_garage_0",
    "faction_base_kitchen_0", "faction_base_lightindustry_workshop_0",
    "faction_base_livestock_0", "faction_base_livestock_2_0",
    "faction_base_mansion_c1", "faction_base_mansion_c2",
    "faction_base_mansion_c3", "faction_base_mansion_c4",
    "faction_base_mansion_c5", "faction_base_mansion_e1",
    "faction_base_mansion_e2", "faction_base_mansion_t1",
    "faction_base_mansion_t2", "faction_base_mansion_t3",
]


def main():
    for eid in IDS:
        img = build(eid)
        folder = os.path.join(BASE, eid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, eid + ".png"))
        with open(os.path.join(folder, eid + ".json"), "w") as f:
            json.dump([{"id": eid, "fg": eid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", eid)

    # Contact sheet: 8 cols x 6 rows at 2x NEAREST.
    cell, gap = 64, 3
    cols, rows = 8, 6
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, eid in enumerate(IDS):
        img = Image.open(os.path.join(BASE, eid, eid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_om2.png")
    sheet.save(out)
    print("wrote preview_om2.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
