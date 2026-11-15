#!/usr/bin/env python3
"""Batch FIELD1 (Phase 2g.4 scope extension): field types, first half.

45 ids. Two families, per the game data:

* Zone fields (31 ids, data/json/zone_field_type.json): organizational
  area markers from the zone manager -- loot zones by category, work
  zones, auto eat/drink, farm plot, fishing spot, firewood source,
  chop trees. Not physical fields, so ONE shared treatment: corner
  brackets in a per-category accent color + a category pictogram, so
  the set reads as a coherent marker family. Filthy variants get
  seeded dirt splotches over the clean pictogram.

* Physical fields (14 ids, data/json/field_type.json): drawn as
  themselves -- bee swarm (cloud of bees), biolum (cyan/blue glowing
  motes), clairvoyant (eye), darkness (full-tile near-black),
  construction site (scaffold + boards), air-conditioner radiosphere
  (warm-air rings), and cold_air/hot_air 1-4 as intensity-graded
  wisp treatments, one parameterized renderer per family (data:
  convection mods grow ~ -2..-160 / +2..+160 across the grades).

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is
strictly {0, 255}: translucency is faked with pale opaque tones.
Writes ONLY its own 45 folders + preview_field1.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "fields")
CREDITS = ("Drawn from the field's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

INK = (238, 238, 228, 255)       # pictogram light tone
SHADE = (28, 30, 34, 255)        # pictogram dark detail


# ---------------------------------------------------------------- zones

def brackets(d, color, inset=2, length=6, width=2):
    for x, y, dx, dy in ((inset, inset, 1, 1), (31 - inset, inset, -1, 1),
                         (inset, 31 - inset, 1, -1), (31 - inset, 31 - inset, -1, -1)):
        d.line([(x, y), (x + dx * length, y)], fill=color, width=width)
        d.line([(x, y), (x, y + dy * length)], fill=color, width=width)


def p_bullet(d, c):
    d.polygon([(14, 8), (18, 8), (17, 12), (15, 12)], fill=(212, 175, 105, 255))
    d.rectangle([14, 12, 18, 22], fill=c)
    d.rectangle([14, 20, 18, 22], fill=(150, 110, 60, 255))


def p_armor(d, c):
    d.polygon([(11, 9), (15, 9), (16, 11), (20, 11), (21, 9), (24, 10),
               (23, 16), (21, 15), (21, 23), (11, 23), (11, 15), (9, 16),
               (8, 10)], fill=c)
    d.rectangle([15, 13, 17, 19], fill=SHADE)


def p_artifacts(d, c):
    d.polygon([(16, 7), (18, 13), (24, 15), (18, 17), (16, 23), (14, 17),
               (8, 15), (14, 13)], fill=c)
    d.ellipse([14, 13, 18, 17], fill=INK)


def p_cup(d, c):
    d.polygon([(11, 11), (21, 11), (19, 23), (13, 23)], fill=c)
    d.line([(18, 5), (16, 11)], fill=INK, width=2)
    d.line([(12, 15), (20, 15)], fill=INK, width=1)


def p_cloche(d, c):
    d.arc([9, 11, 23, 25], start=180, end=360, fill=c, width=3)
    d.line([(8, 22), (24, 22)], fill=c, width=2)
    d.ellipse([15, 9, 17, 11], fill=c)


def p_sack(d, c):
    d.polygon([(13, 10), (19, 10), (22, 16), (21, 23), (11, 23), (10, 16)],
              fill=c)
    d.line([(13, 10), (19, 10)], fill=(150, 110, 60, 255), width=2)
    d.ellipse([13, 15, 19, 20], fill=INK)


def p_crate(d, c):
    d.rectangle([9, 10, 23, 23], outline=c, width=2)
    d.line([(9, 10), (23, 23)], fill=c, width=1)
    d.line([(23, 10), (9, 23)], fill=c, width=1)


def p_chip(d, c):
    d.rectangle([11, 11, 21, 21], outline=c, width=2)
    d.rectangle([14, 14, 18, 18], fill=c)
    for i in range(3):
        p = 12 + i * 4
        d.line([(p, 8), (p, 11)], fill=c, width=1)
        d.line([(p, 21), (p, 24)], fill=c, width=1)
        d.line([(8, p), (11, p)], fill=c, width=1)
        d.line([(21, p), (24, p)], fill=c, width=1)


def p_book(d, c):
    d.polygon([(16, 10), (11, 8), (9, 8), (9, 22), (11, 22), (16, 24)],
              fill=c)
    d.polygon([(16, 10), (21, 8), (23, 8), (23, 22), (21, 22), (16, 24)],
              fill=INK)
    d.line([(16, 10), (16, 24)], fill=SHADE, width=1)


def p_flask(d, c):
    d.polygon([(14, 8), (18, 8), (18, 13), (22, 21), (22, 23), (10, 23),
               (10, 21), (14, 13)], fill=c)
    d.line([(12, 19), (20, 19)], fill=(120, 230, 140, 255), width=2)


def p_tree(d, c):
    d.polygon([(16, 6), (21, 14), (18, 14), (23, 20), (9, 20), (14, 14),
               (11, 14)], fill=c)
    d.rectangle([15, 20, 17, 24], fill=(140, 100, 60, 255))


def p_shirt(d, c):
    d.polygon([(12, 9), (15, 9), (16, 11), (17, 11), (20, 9), (24, 12),
               (21, 15), (21, 23), (11, 23), (11, 15), (8, 12)], fill=c)


def p_container(d, c):
    d.rectangle([9, 12, 23, 23], fill=c)
    d.rectangle([8, 9, 24, 12], fill=INK)
    d.rectangle([14, 10, 18, 11], fill=SHADE)


def p_corpse(d, c):
    d.ellipse([8, 13, 13, 18], fill=c)
    d.rounded_rectangle([13, 14, 24, 18], radius=2, fill=c)
    d.line([(18, 18), (21, 22)], fill=c, width=2)
    d.line([(15, 18), (14, 22)], fill=c, width=2)


def p_coin(d, c):
    d.ellipse([9, 9, 23, 23], fill=c)
    d.ellipse([12, 12, 20, 20], outline=(150, 110, 30, 255), width=1)
    d.line([(16, 12), (16, 20)], fill=(150, 110, 30, 255), width=2)


def p_question(d, c):
    for x, y in ((13, 9), (19, 9), (11, 11), (21, 11), (21, 13), (17, 15),
                 (16, 17)):
        d.rectangle([x, y, x + 2, y + 2], fill=c)
    d.rectangle([15, 21, 17, 23], fill=c)


def p_exclaim(d, c):
    d.rectangle([15, 8, 17, 18], fill=c)
    d.rectangle([15, 21, 17, 23], fill=c)


def p_wrench(d, c):
    d.arc([13, 7, 21, 15], start=300, end=200, fill=c, width=3)
    d.line([(15, 13), (21, 21)], fill=c, width=3)
    d.ellipse([19, 19, 23, 23], outline=c, width=2)


def p_scalpel(d, c):
    d.polygon([(9, 20), (17, 12), (20, 12), (13, 21)], fill=INK)
    d.line([(18, 11), (23, 6)], fill=c, width=2)
    d.line([(17, 12), (22, 7)], fill=c, width=1)


def p_bottle(d, c):
    d.rectangle([14, 7, 18, 10], fill=c)
    d.polygon([(14, 10), (18, 10), (20, 14), (20, 23), (12, 23), (12, 14)],
              fill=c)
    d.rectangle([13, 16, 19, 20], fill=INK)


def p_capsule(d, c):
    d.rounded_rectangle([9, 12, 23, 19], radius=3, fill=INK)
    d.rounded_rectangle([16, 12, 23, 19], radius=3, fill=c)
    d.line([(10, 20), (22, 11)], fill=(0, 0, 0, 0))


def p_farm(d, c):
    for i, y in enumerate((12, 17, 22)):
        d.line([(8, y), (24, y)], fill=(140, 100, 60, 255), width=2)
    for x in (11, 16, 21):
        d.line([(x, 12), (x, 8)], fill=c, width=1)
        d.ellipse([x - 2, 6, x, 9], fill=c)
        d.ellipse([x, 6, x + 2, 9], fill=c)


def p_logs(d, c):
    d.ellipse([9, 15, 16, 22], fill=(190, 150, 100, 255))
    d.ellipse([11, 17, 14, 20], outline=(140, 100, 60, 255))
    d.ellipse([16, 15, 23, 22], fill=(190, 150, 100, 255))
    d.ellipse([18, 17, 21, 20], outline=(140, 100, 60, 255))
    d.rectangle([10, 9, 22, 13], fill=c)
    d.ellipse([20, 9, 23, 13], fill=(190, 150, 100, 255))


def p_fish(d, c):
    d.polygon([(8, 16), (13, 11), (20, 11), (24, 16), (20, 21), (13, 21)],
              fill=c)
    d.polygon([(8, 16), (5, 12), (5, 20)], fill=c)
    d.ellipse([19, 14, 21, 16], fill=SHADE)


def p_apple(d, c):
    d.ellipse([10, 12, 22, 24], fill=c)
    d.line([(16, 12), (17, 8)], fill=(140, 100, 60, 255), width=2)
    d.ellipse([17, 7, 22, 10], fill=(90, 190, 90, 255))


def p_drop(d, c):
    d.polygon([(16, 7), (21, 16), (22, 19), (20, 23), (12, 23), (10, 19),
               (11, 16)], fill=c)
    d.ellipse([13, 17, 16, 21], fill=INK)


def p_gun(d, c):
    d.polygon([(7, 12), (23, 12), (23, 15), (15, 15), (14, 17), (16, 17),
               (15, 23), (11, 23), (12, 16), (7, 16)], fill=c)


def p_star_slash(d, c):
    d.polygon([(16, 8), (18, 13), (23, 13), (19, 16), (20, 22), (16, 19),
               (12, 22), (13, 16), (9, 13), (14, 13)], outline=c, width=2)
    d.line([(9, 23), (23, 9)], fill=(230, 80, 70, 255), width=2)


def dirt(d, rng, n=6):
    for _ in range(n):
        x = rng.uniform(8, 22)
        y = rng.uniform(8, 22)
        r = rng.uniform(1.2, 3.0)
        col = rng.choice([(101, 78, 48, 255), (84, 64, 38, 255),
                          (120, 95, 55, 255)])
        d.ellipse([x - r, y - r, x + r, y + r], fill=col)


# id -> (accent RGB, pictogram fn, filthy?)
ZONES = {
    "fd_ammo_loot_zone": ((240, 150, 60), p_bullet, False),
    "fd_armor_loot_zone": ((130, 170, 220), p_armor, False),
    "fd_artifacts_loot_zone": ((200, 120, 240), p_artifacts, False),
    "fd_auto_drink_zone": ((110, 200, 240), p_cup, False),
    "fd_auto_eat_zone": ((245, 180, 90), p_cloche, False),
    "fd_basecamp_food_zone": ((140, 210, 110), p_sack, False),
    "fd_basecamp_storage_zone": ((200, 160, 110), p_crate, False),
    "fd_bionics_loot_zone": ((90, 230, 210), p_chip, False),
    "fd_books_loot_zone": ((225, 120, 100), p_book, False),
    "fd_chemical_loot_zone": ((120, 230, 140), p_flask, False),
    "fd_chop_trees_zone": ((100, 200, 100), p_tree, False),
    "fd_clothing_loot_zone": ((240, 170, 200), p_shirt, False),
    "fd_containers_loot_zone": ((190, 190, 180), p_container, False),
    "fd_corpses_loot_zone": ((170, 190, 160), p_corpse, False),
    "fd_currency_loot_zone": ((245, 210, 80), p_coin, False),
    "fd_custom_loot_zone": ((235, 235, 235), p_question, False),
    "fd_default_loot_zone": ((200, 200, 200), p_exclaim, False),
    "fd_disassembly_work_zone": ((245, 160, 70), p_wrench, False),
    "fd_dissection_work_zone": ((240, 120, 140), p_scalpel, False),
    "fd_drink_loot_zone": ((100, 170, 245), p_bottle, False),
    "fd_drugs_loot_zone": ((240, 130, 190), p_capsule, False),
    "fd_farm_plot_zone": ((120, 210, 90), p_farm, False),
    "fd_filthy_armor_loot_zone": ((130, 170, 220), p_armor, True),
    "fd_filthy_clothing_loot_zone": ((240, 170, 200), p_shirt, True),
    "fd_firewood_source_zone": ((215, 140, 80), p_logs, False),
    "fd_fishing_spot_zone": ((110, 190, 245), p_fish, False),
    "fd_food_loot_zone": ((240, 90, 80), p_apple, False),
    "fd_fuel_loot_zone": ((240, 120, 60), p_drop, False),
    "fd_guns_loot_zone": ((180, 185, 195), p_gun, False),
    "fd_ignore_favorites_loot_zone": ((245, 215, 90), p_star_slash, False),
}


def draw_zone(tid, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if tid == "fd_construction_blueprint_zone":
        d.rectangle([5, 5, 26, 26], fill=(38, 84, 168, 255))
        for g in range(10, 26, 5):
            d.line([(g, 5), (g, 26)], fill=(90, 140, 220, 255))
            d.line([(5, g), (26, g)], fill=(90, 140, 220, 255))
        d.rectangle([10, 10, 21, 18], outline=INK, width=1)
        d.line([(10, 14), (21, 14)], fill=INK)
        d.line([(15, 14), (15, 21)], fill=INK)
        d.rectangle([15, 14, 21, 21], outline=INK, width=1)
        brackets(d, INK)
        return img
    accent, fn, filthy = ZONES[tid]
    accent = accent + (255,)
    brackets(d, accent)
    fn(d, accent)
    if filthy:
        dirt(d, rng)
    return img


# -------------------------------------------------------------- physical

def draw_bees(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for _ in range(28):
        x = rng.uniform(3, 27)
        y = rng.uniform(3, 27)
        d.ellipse([x - 2, y - 1.4, x + 2, y + 1.4], fill=(240, 205, 60, 255))
        d.line([(x, y - 1.4), (x, y + 1.4)], fill=(40, 36, 20, 255), width=1)
        d.ellipse([x - 1.6, y - 3.2, x - 0.2, y - 1.6],
                  fill=(225, 240, 245, 255))
        d.ellipse([x + 0.2, y - 3.2, x + 1.6, y - 1.6],
                  fill=(225, 240, 245, 255))
    for _ in range(10):
        x = rng.uniform(2, 29)
        y = rng.uniform(2, 29)
        d.point((x, y), fill=(120, 105, 40, 255))
    return img


def draw_biolum(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for _ in range(42):
        x = rng.uniform(2, 29)
        y = rng.uniform(2, 29)
        r = rng.choice([0.7, 0.7, 1.1, 1.6])
        col = rng.choice([(120, 240, 255, 255), (110, 170, 255, 255),
                          (170, 250, 255, 255)])
        d.ellipse([x - r, y - r, x + r, y + r], fill=col)
        if r > 1.2:
            d.point((x, y), fill=(240, 255, 255, 255))
    return img


def draw_clairvoyant(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    violet = (208, 188, 255, 255)
    d.arc([5, 8, 27, 24], start=200, end=340, fill=violet, width=2)
    d.arc([5, 8, 27, 24], start=20, end=160, fill=violet, width=2)
    d.ellipse([11, 11, 21, 21], fill=(110, 200, 240, 255))
    d.ellipse([14, 14, 18, 18], fill=(16, 18, 30, 255))
    d.point((15, 13), fill=(255, 255, 255, 255))
    for x, y in ((5, 5), (26, 5), (4, 26), (27, 25)):
        d.line([(x - 1, y), (x + 1, y)], fill=violet)
        d.line([(x, y - 1), (x, y + 1)], fill=violet)
    return img


def draw_darkness(rng):
    img = Image.new("RGBA", (32, 32), (4, 4, 7, 255))
    d = ImageDraw.Draw(img)
    for _ in range(24):
        x = rng.uniform(0, 31)
        y = rng.uniform(0, 31)
        d.point((x, y), fill=(11, 11, 16, 255))
    return img


def draw_construction(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    wood = (172, 132, 82, 255)
    wood_d = (118, 86, 48, 255)
    pole = (105, 108, 112, 255)
    d.line([(7, 4), (7, 27)], fill=pole, width=2)
    d.line([(24, 4), (24, 27)], fill=pole, width=2)
    d.line([(7, 6), (24, 18)], fill=pole, width=1)
    for y in (9, 19):
        d.rectangle([5, y, 27, y + 2], fill=wood)
        d.line([(5, y + 2), (27, y + 2)], fill=wood_d)
    d.line([(16, 11), (16, 15)], fill=SHADE, width=1)
    d.polygon([(14, 15), (18, 15), (16, 18)], fill=(230, 130, 50, 255))
    for i, (x, y) in enumerate(((17, 25), (21, 25), (19, 22))):
        d.rectangle([x, y, x + 4, y + 2], fill=(165, 84, 62, 255))
    d.rectangle([5, 24, 11, 27], fill=wood_d)
    return img


def wisp_layer(size, vertical, palette, count, rng, amp=1.6):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for i in range(count):
        col = palette[i % len(palette)]
        if vertical:
            x0 = rng.uniform(3, size - 4)
            y0 = rng.uniform(2, size - 14)
            ln = rng.uniform(8, 13)
            pts = [(x0 + amp * math.sin((y - y0) / 3.0), y)
                   for y in range(int(y0), int(y0 + ln))]
        else:
            y0 = rng.uniform(3, size - 4)
            x0 = rng.uniform(2, size - 14)
            ln = rng.uniform(8, 13)
            pts = [(x, y0 + amp * math.sin((x - x0) / 3.0))
                   for x in range(int(x0), int(x0 + ln))]
        d.line(pts, fill=col, width=2)
    return img


COLD_PALS = {
    1: [(203, 230, 247, 255)],
    2: [(168, 214, 243, 255), (196, 228, 248, 255)],
    3: [(120, 183, 238, 255), (168, 214, 243, 255)],
    4: [(84, 200, 238, 255), (110, 170, 240, 255), (170, 235, 250, 255)],
}
HOT_PALS = {
    1: [(247, 228, 158, 255)],
    2: [(245, 198, 112, 255), (247, 228, 158, 255)],
    3: [(242, 152, 74, 255), (245, 198, 112, 255)],
    4: [(238, 102, 62, 255), (242, 152, 74, 255), (250, 210, 130, 255)],
}


def draw_air(level, hot, rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    pal = (HOT_PALS if hot else COLD_PALS)[level]
    layer = wisp_layer(32, vertical=hot, palette=pal,
                       count=2 + level * 2, rng=rng)
    img.alpha_composite(layer)
    d = ImageDraw.Draw(img)
    if not hot:
        for _ in range(level * 3):
            x = rng.uniform(2, 29)
            y = rng.uniform(2, 29)
            d.point((x, y), fill=(235, 248, 255, 255))
            if level >= 3:
                d.line([(x - 1, y), (x + 1, y)], fill=(200, 235, 250, 255))
                d.line([(x, y - 1), (x, y + 1)], fill=(200, 235, 250, 255))
    else:
        for _ in range(level * 2):
            x = rng.uniform(2, 29)
            y = rng.uniform(2, 29)
            d.point((x, y), fill=(255, 190, 110, 255))
    return img


def draw_radiosphere(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cols = [(242, 152, 74, 255), (245, 198, 112, 255), (247, 228, 158, 255)]
    for i, r in enumerate((13, 9.5, 6)):
        col = cols[i]
        pts = []
        for a in range(0, 361, 10):
            rr = r + 0.8 * math.sin(math.radians(a * 3) + i)
            pts.append((16 + rr * math.cos(math.radians(a)),
                        16 + rr * math.sin(math.radians(a))))
        d.line(pts, fill=col, width=2)
    d.rectangle([13, 13, 19, 19], fill=(120, 124, 128, 255))
    for y in (15, 17):
        d.line([(14, y), (18, y)], fill=(70, 72, 76, 255))
    return img


PHYSICAL = {
    "fd_bee_swarm": draw_bees,
    "fd_biolum": draw_biolum,
    "fd_clairvoyant": draw_clairvoyant,
    "fd_darkness": draw_darkness,
    "fd_construction_site": draw_construction,
    "fd_air_conditioner_radiosphere": draw_radiosphere,
}

IDS = [
    "fd_air_conditioner_radiosphere", "fd_ammo_loot_zone",
    "fd_armor_loot_zone", "fd_artifacts_loot_zone", "fd_auto_drink_zone",
    "fd_auto_eat_zone", "fd_basecamp_food_zone",
    "fd_basecamp_storage_zone", "fd_bee_swarm", "fd_biolum",
    "fd_bionics_loot_zone", "fd_books_loot_zone", "fd_chemical_loot_zone",
    "fd_chop_trees_zone", "fd_clairvoyant", "fd_clothing_loot_zone",
    "fd_cold_air1",
    "fd_cold_air2", "fd_cold_air3", "fd_cold_air4",
    "fd_construction_blueprint_zone", "fd_construction_site",
    "fd_containers_loot_zone", "fd_corpses_loot_zone",
    "fd_currency_loot_zone", "fd_custom_loot_zone", "fd_darkness",
    "fd_default_loot_zone", "fd_disassembly_work_zone",
    "fd_dissection_work_zone", "fd_drink_loot_zone",
    "fd_drugs_loot_zone", "fd_farm_plot_zone",
    "fd_filthy_armor_loot_zone", "fd_filthy_clothing_loot_zone",
    "fd_firewood_source_zone", "fd_fishing_spot_zone",
    "fd_food_loot_zone", "fd_fuel_loot_zone", "fd_guns_loot_zone",
    "fd_hot_air1", "fd_hot_air2", "fd_hot_air3", "fd_hot_air4",
    "fd_ignore_favorites_loot_zone",
]


def draw_one(tid, rng):
    if tid in ZONES or tid == "fd_construction_blueprint_zone":
        img = draw_zone(tid, rng)
    elif tid.startswith("fd_cold_air"):
        img = draw_air(int(tid[-1]), hot=False, rng=rng)
    elif tid.startswith("fd_hot_air"):
        img = draw_air(int(tid[-1]), hot=True, rng=rng)
    else:
        img = PHYSICAL[tid](rng)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (tid, alphas)
    return img


def main():
    for tid in IDS:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = draw_one(tid, rng)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", tid)

    cell, gap, cols = 64, 3, 9
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
                       "preview_field1.png")
    sheet.save(out)
    print("wrote preview_field1.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
