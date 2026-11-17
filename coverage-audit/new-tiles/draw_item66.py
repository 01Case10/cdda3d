#!/usr/bin/env python3
"""ITEM66: frozen treats + ID badges + Hub 01 robofac armor -- CDDA
Voxel coverage audit.  Mixed tail batch, three families:

  Frozen treats (16, items/comestibles/frozen.json) -- each drawn in
    its own serving form: scoop cups told apart by scoop colour and
    inclusions (chocolate, custard gold, candy/fruit bits, sorbet,
    gelato with a wafer), frozen yogurt as a soft-serve swirl,
    milkshakes as tall straw glasses (deluxe ones get whipped cream
    and a cherry; the fast-food one is a lidded paper cup), snow
    cones as cyan shaved-ice mounds in paper cones (the alcoholic
    one gets a cherry and a cocktail umbrella), a split half-and-
    half sundae cup, and a slushy frozen lemonade with a lemon wedge.
  ID cards (16, items/id_cards.json) -- portrait employee badges
    with a zone-colour band, photo box and a role motif (wrench for
    maintenance, cross for the doctor, flask for researchers,
    shield for security, star for the slate-zone supervisor, truck
    for transport freight).  The three non-badges are drawn as
    themselves: the LIXA military police badge as a shield badge,
    the Exodii string-dimension key as a thick yellow plastic key,
    and the warehouse rebooter as a small blue plug-in device.
  Hub 01 armor (16, items/armor/robofac_armor.json) -- helmet,
    mantle and segmented skirt forms in five material languages:
    prototype (pale printed polymer, print lines), kinetic (dark
    rubber, hexagon bumps / scale skirt), turnout (tan Nomex with a
    silver reflective stripe), ballistic (olive Kevlar weave,
    layered bands), soldier (dark sleek UHMWPE with a metal core
    band).  The MPET poncho liner is a folded quilted blanket with
    a silver space-blanket sheen.

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "frozen_lemonade",
    "icecream_artificial",
    "icecream_candy",
    "icecream_choc",
    "icecream_custard",
    "icecream_fruit",
    "icecream_gelato",
    "icecream_sorbet",
    "icecream_yogurt",
    "milkshake_choc",
    "milkshake_deluxe",
    "milkshake_deluxe_choc",
    "milkshake_fastfood",
    "snowcone",
    "snowcone_alcoholic",
    "sundae_cup",
    "id_LIXA_mil",
    "id_anduril_ar",
    "id_exodii_string_dimension",
    "id_science_maintenance_blue",
    "id_science_maintenance_green",
    "id_science_maintenance_yellow",
    "id_science_medical_red",
    "id_science_mu",
    "id_science_mutagen_cyan",
    "id_science_mutagen_green",
    "id_science_mutagen_pink",
    "id_science_security_black",
    "id_science_security_magenta",
    "id_science_security_yellow",
    "id_science_transport_1",
    "warehouse_rebooter",
    "robofac_basic_helmet",
    "robofac_basic_mantle",
    "robofac_basic_skirt",
    "robofac_kevlar_helmet",
    "robofac_kevlar_mantle",
    "robofac_kevlar_skirt",
    "robofac_military_helmet",
    "robofac_military_mantle",
    "robofac_military_skirt",
    "robofac_nomex_helmet",
    "robofac_nomex_mantle",
    "robofac_nomex_skirt",
    "robofac_poncho_liner",
    "robofac_rubber_helmet",
    "robofac_rubber_mantle",
    "robofac_rubber_skirt",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

WHITE = (246, 244, 238, 255)
CREAM = (244, 230, 200, 255)
INK = (52, 52, 58, 255)
GRAY = (148, 148, 152, 255)
CHOC = (110, 66, 38, 255)
CHOC_DARK = (82, 48, 28, 255)
RED = (200, 44, 40, 255)
CHERRY = (178, 26, 34, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ---------------------------------------------------------- frozen treats

def cup(d, col=(226, 230, 234, 255)):
    """Plastic serving cup under a scoop: rim + tapered body."""
    d.rectangle([9, 17, 22, 19], fill=shade(col, 1.04))
    d.rectangle([9, 17, 22, 19], outline=INK, width=1)
    d.polygon([(10, 20), (21, 20), (20, 28), (11, 28)], fill=col)
    d.polygon([(10, 20), (21, 20), (20, 28), (11, 28)], outline=INK)
    d.line([11, 24, 20, 24], fill=shade(col, 0.88), width=1)


def scoop(d, rng, col, speckles=(), sparkle=False, flat=False):
    """Dome scoop centred ~(15,11); speckles=(colour, count, size)."""
    if flat:
        d.rectangle([8, 9, 23, 17], fill=col)
        d.ellipse([8, 6, 23, 12], fill=col)
        d.rectangle([8, 9, 23, 17], outline=INK, width=1)
        top = [(x, y) for x in range(10, 22) for y in range(8, 12)]
    else:
        d.ellipse([8, 5, 23, 18], fill=col)
        d.ellipse([8, 5, 23, 18], outline=INK, width=1)
        d.line([10, 17, 21, 17], fill=shade(col, 0.9), width=1)
        top = None
    # drip bumps along the scoop's lower edge
    for bx in (10, 15, 20):
        d.point((bx, 18), fill=col)
    for scol, n, big in speckles:
        for _ in range(n):
            for _try in range(12):
                x = rng.randint(9, 22)
                y = rng.randint(6, 15)
                if ((x - 15.5) / 7.6) ** 2 + ((y - 11.5) / 6.6) ** 2 <= 1:
                    if big:
                        d.rectangle([x, y, x + 1, y + 1], fill=scol)
                    else:
                        d.point((x, y), fill=scol)
                    break
    if sparkle:  # icy crystals catching the light
        for x, y in ((11, 8), (17, 7), (20, 11), (13, 12), (19, 14)):
            d.point((x, y), fill=(255, 255, 255, 255))
            d.point((x, y - 1), fill=(230, 248, 252, 255))
    d.point((12, 8), fill=shade(col, 1.1))  # sheen
    d.point((13, 7), fill=shade(col, 1.1))


def draw_treat(d, rng, sid):
    if sid == "frozen_lemonade":
        # slush in a clear cup, straw, lemon wedge on the rim
        d.polygon([(10, 12), (21, 12), (20, 28), (11, 28)],
                  fill=(238, 240, 232, 255))
        d.polygon([(10, 12), (21, 12), (20, 28), (11, 28)], outline=INK)
        d.rectangle([11, 14, 20, 26], fill=(240, 226, 130, 255))
        for x, y in ((13, 17), (17, 20), (14, 23), (18, 25)):
            d.point((x, y), fill=(252, 244, 190, 255))  # ice crystals
        d.line([18, 3, 15, 13], fill=(240, 200, 40, 255), width=2)  # straw
        d.ellipse([19, 8, 25, 14], fill=(248, 224, 84, 255))  # lemon wedge
        d.ellipse([19, 8, 25, 14], outline=INK, width=1)
        d.point((22, 11), fill=(252, 240, 170, 255))
        d.line([22, 9, 22, 13], fill=(226, 198, 60, 255), width=1)
    elif sid == "icecream_artificial":
        # plain government tub of beige "dairy dessert", barely domed
        d.rectangle([9, 13, 22, 27], fill=(232, 228, 218, 255))
        d.rectangle([9, 13, 22, 27], outline=INK, width=1)
        d.rectangle([8, 11, 23, 14], fill=(206, 200, 186, 255))
        d.rectangle([8, 11, 23, 14], outline=INK, width=1)
        d.ellipse([10, 8, 21, 13], fill=(232, 220, 192, 255))
        d.rectangle([11, 18, 20, 22], fill=(196, 190, 174, 255))  # label
        d.line([12, 20, 19, 20], fill=GRAY, width=1)
    elif sid == "icecream_candy":
        scoop(d, rng, (246, 240, 228, 255),
              speckles=((CHOC_DARK, 7, True), ((196, 130, 62, 255), 5, True),
                        (RED, 3, False)))
        cup(d)
    elif sid == "icecream_choc":
        scoop(d, rng, CHOC, speckles=((CHOC_DARK, 5, False),))
        cup(d)
    elif sid == "icecream_custard":
        scoop(d, rng, (246, 216, 130, 255))  # smooth golden custard
        cup(d, col=(234, 226, 206, 255))
    elif sid == "icecream_fruit":
        scoop(d, rng, (248, 226, 226, 255),
              speckles=((RED, 6, True), ((128, 60, 140, 255), 4, True),
                        ((240, 150, 60, 255), 3, True)))
        cup(d)
    elif sid == "icecream_gelato":
        scoop(d, rng, (226, 196, 158, 255), flat=True)  # dense, flat top
        cup(d, col=(214, 200, 178, 255))
        d.polygon([(20, 2), (23, 3), (19, 10), (17, 9)],
                  fill=(222, 186, 128, 255))  # wafer stick
        d.polygon([(20, 2), (23, 3), (19, 10), (17, 9)], outline=INK)
    elif sid == "icecream_sorbet":
        scoop(d, rng, (226, 62, 92, 255), sparkle=True)  # raspberry sorbet
        cup(d)
    elif sid == "icecream_yogurt":
        # soft-serve swirl: stacked tapering bands with a curled peak
        for i, (w, y) in enumerate(((15, 20), (13, 16), (10, 12), (7, 9))):
            x0 = 16 - w // 2
            d.ellipse([x0, y, x0 + w, y + 5], fill=WHITE)
            d.ellipse([x0, y, x0 + w, y + 5], outline=INK, width=1)
        d.polygon([(14, 9), (17, 4), (18, 8)], fill=WHITE)
        d.polygon([(14, 9), (17, 4), (18, 8)], outline=INK)
        d.polygon([(11, 24), (21, 24), (19, 29), (13, 29)],
                  fill=(206, 178, 138, 255))  # cup base
        d.polygon([(11, 24), (21, 24), (19, 29), (13, 29)], outline=INK)
    elif sid.startswith("milkshake"):
        choc = sid in ("milkshake_choc", "milkshake_deluxe_choc")
        deluxe = "deluxe" in sid
        body = CHOC if choc else (246, 226, 214, 255)
        if sid == "milkshake_fastfood":
            # lidded paper fast-food cup, red band, straw through the lid
            d.polygon([(9, 10), (22, 10), (20, 28), (11, 28)],
                      fill=(240, 238, 234, 255))
            d.polygon([(9, 10), (22, 10), (20, 28), (11, 28)], outline=INK)
            d.rectangle([9, 16, 21, 20], fill=RED)  # branding band
            d.ellipse([13, 17, 18, 20], fill=WHITE)
            d.rectangle([8, 8, 23, 10], fill=(214, 212, 208, 255))
            d.rectangle([8, 8, 23, 10], outline=INK, width=1)
            d.line([18, 2, 16, 8], fill=RED, width=2)  # straw
            return
        # tall glass, slightly tapered
        d.polygon([(10, 9), (21, 9), (20, 28), (11, 28)],
                  fill=shade(body, 1.02))
        d.polygon([(10, 9), (21, 9), (20, 28), (11, 28)], outline=INK)
        d.line([12, 12, 11, 26], fill=shade(body, 0.86), width=1)
        d.line([19, 12, 19, 26], fill=shade(body, 1.1), width=1)
        d.line([17, 3, 15, 9], fill=RED, width=2)  # straw
        if deluxe:
            for bx in (11, 14, 17, 20):  # whipped cream bumps
                d.ellipse([bx - 2, 5, bx + 2, 10], fill=WHITE)
            d.ellipse([14, 2, 18, 6], fill=CHERRY)  # cherry on top
            d.point((15, 3), fill=(240, 120, 120, 255))
            if not choc:  # candy sprinkles on the cream
                for x, y, c in ((11, 7, RED), (19, 7, (60, 140, 220, 255)),
                                (15, 8, (240, 180, 40, 255))):
                    d.point((x, y), fill=c)
        else:
            d.rectangle([10, 7, 21, 9], fill=shade(body, 1.12))
    elif sid.startswith("snowcone"):
        # paper cone with a mound of shaved ice
        d.ellipse([7, 3, 24, 14], fill=(120, 216, 236, 255))
        d.ellipse([7, 3, 24, 14], outline=INK, width=1)
        for x, y in ((11, 6), (16, 5), (20, 8), (13, 10), (19, 12)):
            d.point((x, y), fill=(214, 246, 252, 255))
        d.polygon([(9, 12), (22, 12), (16, 29)], fill=(240, 236, 226, 255))
        d.polygon([(9, 12), (22, 12), (16, 29)], outline=INK)
        d.line([11, 17, 20, 17], fill=(206, 200, 184, 255), width=1)
        if sid == "snowcone_alcoholic":
            d.line([16, 12, 16, 29], fill=(226, 150, 60, 255),
                   width=1)  # booze soaking down the cone
            d.ellipse([20, 1, 24, 5], fill=CHERRY)  # cherry
            d.point((21, 2), fill=(240, 120, 120, 255))
            d.line([9, 8, 9, 2], fill=INK, width=1)  # umbrella stick
            d.polygon([(4, 3), (14, 3), (9, 0)], fill=(240, 130, 40, 255))
    elif sid == "sundae_cup":
        # short cup, dome split half vanilla / half chocolate
        d.ellipse([8, 6, 15, 16], fill=WHITE)
        d.ellipse([16, 6, 24, 16], fill=CHOC)
        d.rectangle([8, 10, 24, 16], fill=WHITE)
        d.rectangle([16, 10, 24, 16], fill=CHOC)
        d.rectangle([8, 6, 24, 16], outline=INK, width=1)
        d.line([16, 6, 16, 16], fill=INK, width=1)
        d.polygon([(9, 17), (23, 17), (21, 28), (11, 28)],
                  fill=(214, 200, 178, 255))
        d.polygon([(9, 17), (23, 17), (21, 28), (11, 28)], outline=INK)
        d.line([21, 4, 25, 1], fill=GRAY, width=2)  # spoon handle
    else:
        raise KeyError(sid)


# -------------------------------------------------------------- ID badges

# sid -> (zone band colour, role motif)
BADGES = {
    "id_science_maintenance_blue": ((52, 96, 204, 255), "wrench"),
    "id_science_maintenance_green": ((58, 158, 68, 255), "wrench"),
    "id_science_maintenance_yellow": ((204, 164, 44, 255), "wrench"),
    "id_science_medical_red": ((204, 48, 44, 255), "cross"),
    "id_science_mu": ((112, 122, 138, 255), "star"),
    "id_science_mutagen_cyan": ((36, 188, 204, 255), "flask"),
    "id_science_mutagen_green": ((138, 220, 118, 255), "flask"),
    "id_science_mutagen_pink": ((242, 138, 182, 255), "flask"),
    "id_science_security_black": ((30, 30, 36, 255), "shield"),
    "id_science_security_magenta": ((204, 40, 162, 255), "shield"),
    "id_science_security_yellow": ((238, 202, 40, 255), "shield"),
    "id_science_transport_1": ((204, 122, 42, 255), "truck"),
}


def role_icon(d, key, cx, cy, col):
    if key == "wrench":
        d.line([cx - 2, cy + 2, cx + 1, cy - 1], fill=col, width=2)
        d.ellipse([cx, cy - 3, cx + 3, cy], outline=col, width=1)
        d.point((cx + 2, cy - 2), fill=WHITE)
    elif key == "cross":
        d.rectangle([cx, cy - 3, cx + 2, cy + 3], fill=RED)
        d.rectangle([cx - 2, cy - 1, cx + 4, cy + 1], fill=RED)
    elif key == "flask":
        d.polygon([(cx, cy - 3), (cx + 2, cy - 3), (cx + 4, cy + 3),
                   (cx - 2, cy + 3)], outline=col, width=1)
        d.line([cx - 1, cy + 1, cx + 3, cy + 1], fill=col, width=1)
    elif key == "shield":
        d.polygon([(cx - 2, cy - 3), (cx + 3, cy - 3), (cx + 3, cy),
                   (cx, cy + 3), (cx - 2, cy)], fill=col)
        d.point((cx, cy - 1), fill=WHITE)
    elif key == "star":
        for dx, dy in ((0, -3), (0, 3), (-3, 0), (3, 0), (-2, -2), (2, -2),
                       (-2, 2), (2, 2)):
            d.point((cx + dx // 1, cy + dy), fill=col)
        d.rectangle([cx - 1, cy - 1, cx + 1, cy + 1], fill=col)
    elif key == "truck":
        d.rectangle([cx - 3, cy - 2, cx + 1, cy + 1], fill=col)
        d.polygon([(cx + 1, cy - 1), (cx + 3, cy - 1), (cx + 3, cy + 1),
                   (cx + 1, cy + 1)], fill=col)
        d.point((cx - 2, cy + 2), fill=INK)
        d.point((cx + 2, cy + 2), fill=INK)


def draw_badge(d, rng, sid):
    band, role = BADGES[sid]
    # portrait card with a clip slot
    d.rectangle([10, 5, 21, 27], fill=(238, 238, 240, 255))
    d.rectangle([10, 5, 21, 27], outline=INK, width=1)
    d.rectangle([14, 6, 17, 7], fill=(200, 200, 204, 255))  # clip slot
    d.rectangle([10, 9, 21, 12], fill=band)  # zone band
    d.line([11, 10, 15, 10], fill=WHITE, width=1)
    # photo box with a head silhouette
    d.rectangle([12, 14, 16, 19], fill=(206, 212, 220, 255))
    d.rectangle([12, 14, 16, 19], outline=GRAY, width=1)
    d.ellipse([13, 15, 15, 17], fill=(120, 128, 140, 255))
    d.rectangle([13, 17, 15, 19], fill=(120, 128, 140, 255))
    # text lines
    for i, w in enumerate((3, 4, 2)):
        d.line([18, 15 + i * 2, 18 + w, 15 + i * 2], fill=GRAY, width=1)
    d.line([12, 22, 20, 22], fill=GRAY, width=1)
    d.line([12, 24, 18, 24], fill=GRAY, width=1)
    role_icon(d, role, 18, 24, band if band != (238, 202, 40, 255) else
              (160, 128, 20, 255))


def draw_card_special(d, rng, sid):
    if sid == "id_LIXA_mil":
        # shield-shaped military police badge, brass rim, olive field
        d.polygon([(10, 5), (22, 5), (22, 16), (16, 27), (10, 16)],
                  fill=(104, 110, 62, 255))
        d.polygon([(10, 5), (22, 5), (22, 16), (16, 27), (10, 16)],
                  outline=(196, 164, 84, 255), width=2)
        for dx, dy in ((0, -3), (0, 3), (-3, 0), (3, 0), (-2, -2), (2, -2),
                       (-2, 2), (2, 2), (0, 0)):  # star
            d.point((16 + dx, 12 + dy), fill=(240, 236, 220, 255))
        d.rectangle([12, 18, 20, 19], fill=(240, 236, 220, 255))  # MP bar
        d.rectangle([12, 21, 20, 22], fill=(196, 164, 84, 255))
    elif sid == "id_anduril_ar":
        # blue corporate ID card, landscape, circuit/antenna motif
        d.rectangle([5, 9, 26, 23], fill=(52, 92, 178, 255))
        d.rectangle([5, 9, 26, 23], outline=INK, width=1)
        d.rectangle([7, 11, 11, 16], fill=(206, 216, 232, 255))  # photo
        d.ellipse([8, 12, 10, 14], fill=(90, 110, 150, 255))
        for i, w in enumerate((8, 6, 7)):
            d.line([13, 12 + i * 3, 13 + w, 12 + i * 3],
                   fill=(220, 230, 246, 255), width=1)
        d.point((23, 19), fill=(240, 244, 252, 255))  # antenna node
        for r in (2, 4):  # radiating arcs
            d.arc([23 - r, 19 - r, 23 + r, 19 + r], start=300, end=60,
                  fill=(240, 244, 252, 255), width=1)
        d.rectangle([6, 20, 10, 22], fill=(196, 164, 84, 255))  # chip
    elif sid == "id_exodii_string_dimension":
        # thick yellow plastic key with an archaic angled bit, diagonal
        yel = (240, 196, 44, 255)
        d.rectangle([6, 5, 15, 13], fill=yel)  # chunky bow
        d.rectangle([6, 5, 15, 13], outline=INK, width=1)
        d.rectangle([9, 8, 12, 10], fill=(206, 160, 30, 255))  # grip hole
        d.polygon([(13, 11), (24, 22), (21, 25), (10, 14)], fill=yel)
        d.polygon([(13, 11), (24, 22), (21, 25), (10, 14)], outline=INK)
        # angled bit teeth
        d.polygon([(21, 19), (26, 20), (24, 24)], fill=yel)
        d.polygon([(21, 19), (26, 20), (24, 24)], outline=INK)
        d.polygon([(18, 22), (22, 26), (19, 27)], fill=yel)
        d.line([12, 9, 20, 17], fill=(206, 160, 30, 255), width=1)
    elif sid == "warehouse_rebooter":
        # small blue plug-in device, steel prongs, LED, qt_steel trim
        d.rectangle([9, 8, 22, 21], fill=(52, 96, 190, 255))
        d.rectangle([9, 8, 22, 21], outline=INK, width=1)
        d.rectangle([9, 8, 22, 10], fill=(140, 148, 156, 255))  # steel trim
        d.rectangle([12, 13, 19, 17], fill=(24, 44, 84, 255))  # dark screen
        d.line([13, 15, 18, 15], fill=(90, 220, 160, 255), width=1)
        d.point((20, 12), fill=(90, 230, 110, 255))  # LED
        d.point((20, 19), fill=(240, 200, 60, 255))
        for px in (12, 15, 18):  # plug prongs
            d.rectangle([px, 22, px + 1, 26], fill=(168, 174, 182, 255))
            d.rectangle([px, 22, px + 1, 26], outline=INK, width=1)
    else:
        raise KeyError(sid)


# ------------------------------------------------------------ robofac armor

# material -> (body, dark, accent, finish)
MATS = {
    "basic": ((198, 202, 208, 255), (168, 174, 182, 255),
              (176, 182, 190, 255), "print"),
    "rubber": ((56, 56, 64, 255), (38, 38, 44, 255),
               (84, 84, 94, 255), "hex"),
    "nomex": ((198, 166, 94, 255), (164, 132, 66, 255),
              (216, 220, 224, 255), "stripe"),
    "kevlar": ((98, 106, 64, 255), (72, 80, 46, 255),
               (128, 136, 92, 255), "weave"),
    "military": ((68, 76, 72, 255), (48, 54, 50, 255),
                 (150, 156, 152, 255), "core"),
}


def finish_marks(d, rng, mat, x0, x1, y0, y1):
    """Surface language of the material over a region."""
    body, dark, accent, fin = MATS[mat]
    if fin == "print":  # 3D-print layer lines
        for y in range(y0 + 2, y1, 3):
            d.line([x0 + 1, y, x1 - 1, y], fill=dark, width=1)
    elif fin == "hex":  # rubber hexagon bumps
        for y in range(y0 + 2, y1 - 1, 4):
            for x in range(x0 + 2 + (2 if (y // 4) % 2 else 0), x1 - 1, 4):
                d.point((x, y), fill=accent)
                d.point((x + 1, y), fill=accent)
                d.point((x, y + 1), fill=dark)
    elif fin == "stripe":  # silver reflective band
        ym = (y0 + y1) // 2
        d.rectangle([x0 + 1, ym, x1 - 1, ym + 1], fill=accent)
    elif fin == "weave":  # aramid cross-hatch
        for y in range(y0 + 1, y1, 2):
            for x in range(x0 + 1 + ((y // 2) % 2), x1, 2):
                d.point((x, y), fill=dark)
    elif fin == "core":  # sleek: one polished metal band
        ym = y0 + 3
        d.line([x0 + 2, ym, x1 - 2, ym], fill=accent, width=1)


def draw_helmet(d, rng, mat):
    body, dark, accent, fin = MATS[mat]
    d.ellipse([7, 5, 24, 21], fill=body)  # dome
    d.ellipse([7, 5, 24, 21], outline=INK, width=1)
    d.rectangle([7, 13, 24, 21], fill=body)
    d.rectangle([7, 20, 24, 22], fill=dark)  # rim band
    d.rectangle([7, 20, 24, 22], outline=INK, width=1)
    # face opening
    d.rectangle([11, 14, 20, 20], fill=(30, 30, 36, 255))
    d.rectangle([11, 14, 20, 20], outline=INK, width=1)
    d.line([12, 16, 19, 16], fill=(120, 190, 220, 255), width=1)  # visor
    d.rectangle([14, 22, 17, 24], fill=dark)  # chin clip
    d.point((9, 18), fill=accent)  # side clip points
    d.point((22, 18), fill=accent)
    finish_marks(d, rng, mat, 7, 24, 5, 13)
    if fin == "stripe":
        d.rectangle([8, 11, 23, 12], fill=accent)


def draw_mantle(d, rng, mat):
    body, dark, accent, fin = MATS[mat]
    # shoulder pauldrons
    d.ellipse([5, 7, 13, 15], fill=body)
    d.ellipse([5, 7, 13, 15], outline=INK, width=1)
    d.ellipse([18, 7, 26, 15], fill=body)
    d.ellipse([18, 7, 26, 15], outline=INK, width=1)
    # chest plate
    d.polygon([(9, 9), (22, 9), (21, 27), (10, 27)], fill=body)
    d.polygon([(9, 9), (22, 9), (21, 27), (10, 27)], outline=INK)
    d.arc([12, 6, 19, 12], start=0, end=180, fill=INK, width=1)  # neck cut
    d.line([15, 12, 15, 26], fill=dark, width=1)  # centre seam
    for bx in (10, 20):  # strap buckles
        d.rectangle([bx, 22, bx + 2, 24], outline=accent, width=1)
    d.point((11, 13), fill=accent)  # clip points
    d.point((20, 13), fill=accent)
    finish_marks(d, rng, mat, 9, 22, 14, 26)
    if fin == "stripe":
        d.rectangle([10, 18, 21, 19], fill=accent)


def draw_skirt(d, rng, mat):
    body, dark, accent, fin = MATS[mat]
    d.rectangle([7, 7, 24, 11], fill=dark)  # belt
    d.rectangle([7, 7, 24, 11], outline=INK, width=1)
    d.rectangle([14, 8, 17, 10], outline=accent, width=1)  # buckle
    if mat == "nomex":  # tight leggings under the plates
        d.rectangle([9, 12, 14, 28], fill=shade(body, 0.8))
        d.rectangle([17, 12, 22, 28], fill=shade(body, 0.8))
    if mat == "kevlar":  # leg sleeves under the plates
        d.rectangle([9, 24, 14, 28], fill=dark)
        d.rectangle([17, 24, 22, 28], fill=dark)
    # segmented plates: two leg columns, three rows
    for cx0 in (8, 17):
        for r in range(3):
            y0 = 12 + r * 5
            d.polygon([(cx0, y0), (cx0 + 6, y0), (cx0 + 5, y0 + 4),
                       (cx0 + 1, y0 + 4)], fill=body)
            d.polygon([(cx0, y0), (cx0 + 6, y0), (cx0 + 5, y0 + 4),
                       (cx0 + 1, y0 + 4)], outline=INK)
            if fin == "hex":  # rubber skirt reads as scale armor
                d.arc([cx0 + 1, y0 + 1, cx0 + 5, y0 + 5], start=0, end=180,
                      fill=accent, width=1)
            elif fin == "weave":
                d.point((cx0 + 2, y0 + 2), fill=dark)
                d.point((cx0 + 4, y0 + 2), fill=dark)
            elif fin == "print":
                d.line([cx0 + 1, y0 + 2, cx0 + 5, y0 + 2], fill=dark,
                       width=1)
            elif fin == "core":
                d.point((cx0 + 3, y0 + 2), fill=accent)  # metal core rivet
            elif fin == "stripe" and r == 1:
                d.line([cx0 + 1, y0 + 2, cx0 + 5, y0 + 2], fill=accent,
                       width=1)


def draw_poncho_liner(d, rng):
    body = (86, 88, 94, 255)  # dark gray MPET
    d.rectangle([7, 9, 24, 25], fill=body)
    d.rectangle([7, 9, 24, 25], outline=INK, width=1)
    for i in range(-2, 6):  # diamond quilting
        d.line([7 + i * 4, 9, 7 + i * 4 + 8, 17], fill=(64, 66, 72, 255),
               width=1)
        d.line([7 + i * 4, 25, 7 + i * 4 + 8, 17], fill=(64, 66, 72, 255),
               width=1)
    # silver space-blanket sheen patches
    d.polygon([(8, 10), (14, 10), (10, 16)], fill=(206, 212, 218, 255))
    d.polygon([(18, 19), (23, 19), (20, 24)], fill=(190, 196, 202, 255))
    d.line([7, 23, 24, 23], fill=(64, 66, 72, 255), width=1)  # fold edge
    d.line([7, 24, 24, 24], fill=(150, 154, 160, 255), width=1)
    for gx in (9, 15, 21):  # tie tapes
        d.point((gx, 9), fill=(196, 164, 84, 255))


def draw_armor(d, rng, sid):
    if sid == "robofac_poncho_liner":
        draw_poncho_liner(d, rng)
        return
    mat = sid.split("_")[1]
    part = sid.split("_")[2]
    if part == "helmet":
        draw_helmet(d, rng, mat)
    elif part == "mantle":
        draw_mantle(d, rng, mat)
    else:
        draw_skirt(d, rng, mat)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in BADGES:
        draw_badge(d, rng, sid)
    elif sid.startswith("id_") or sid == "warehouse_rebooter":
        draw_card_special(d, rng, sid)
    elif sid.startswith("robofac_"):
        draw_armor(d, rng, sid)
    else:
        draw_treat(d, rng, sid)
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
    # contact-sheet preview: 8 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 8, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item66.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item66.png {prev.size}")


if __name__ == "__main__":
    main()
