#!/usr/bin/env python3
"""ITEM45: paper goods (pamphlets, brochures, flyers, maps, notes) --
CDDA Voxel coverage audit.

Draws each paper id in the form its description names: folded leaflets
with a distinct cover colour and motif per topic, unfolded map sheets
with fold creases and a themed route network, and handwritten note
scraps (the _used variants crumpled, stained, scribbled out and torn).
Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "buylocalpamphlet",
    "flyer_evac",
    "furniturebrochure",
    "getactivepamphlet",
    "godco_flier",
    "grocerybrochure",
    "gunnutpamphlet",
    "historybrochure",
    "inner_cabins_depths_note",
    "inner_cabins_depths_note_2",
    "inner_cabins_depths_note_2_used",
    "inner_cabins_depths_note_3",
    "inner_cabins_depths_note_3_used",
    "inner_cabins_depths_note_used",
    "labmap",
    "mallbrochure",
    "militarymap",
    "nightlifepamphlet",
    "petpamphlet",
    "restaurantbrochure",
    "restaurantmap",
    "roadmap",
    "satellitemap",
    "sewermap",
    "subwaymap",
    "subwaystationmap",
    "survivormap",
    "technologybrochure",
    "touristmap",
    "trailmap",
    "urbexmap",
    "xedra_field_notes_used",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

PAPER = (240, 234, 214, 255)
PAPER_AGED = (232, 218, 182, 255)
INK = (52, 50, 58, 255)
WHITE = (248, 248, 244, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ---------------------------------------------------------------- leaflets

def leaflet_base(d, cover):
    """An upright folded leaflet: cover, title band, trifold shading."""
    d.rectangle([8, 3, 24, 29], fill=shade(cover, 0.55))
    d.rectangle([7, 2, 23, 28], fill=cover)
    d.rectangle([7, 2, 23, 8], fill=shade(cover, 1.22))
    d.line([7, 8, 23, 8], fill=shade(cover, 0.6), width=1)
    # trifold creases + a darker folded edge on the right
    for x in (12, 18):
        d.line([x, 9, x, 28], fill=shade(cover, 0.82), width=1)
    d.rectangle([21, 9, 23, 28], fill=shade(cover, 0.88))
    # title ticks in the band
    d.line([10, 5, 20, 5], fill=WHITE, width=2)
    d.line([12, 7, 18, 7], fill=shade(cover, 0.65), width=1)


def motif(d, kind, ink):
    """A simple cover motif centred near (15, 18)."""
    if kind == "bag":  # shopping bag
        d.polygon([(11, 14), (20, 14), (21, 23), (10, 23)], fill=ink)
        d.arc([12, 10, 19, 17], 180, 360, fill=ink, width=2)
        d.point((15, 18), fill=shade(ink, 0.6))
    elif kind == "evac":  # exit door + arrow out
        d.rectangle([9, 12, 15, 24], outline=ink, width=2)
        d.polygon([(16, 16), (21, 16), (21, 14), (25, 18), (21, 22),
                   (21, 20), (16, 20)], fill=ink)
    elif kind == "chair":  # chair side view
        d.line([12, 11, 12, 20], fill=ink, width=2)
        d.line([12, 18, 20, 18], fill=ink, width=2)
        d.line([12, 25, 12, 18], fill=ink, width=2)
        d.line([20, 25, 20, 18], fill=ink, width=2)
        d.line([12, 14, 17, 14], fill=ink, width=1)
    elif kind == "active":  # running figure
        d.ellipse([17, 10, 20, 13], fill=ink)
        d.line([18, 13, 15, 18], fill=ink, width=2)
        d.line([15, 18, 11, 21], fill=ink, width=2)
        d.line([15, 18, 19, 22], fill=ink, width=2)
        d.line([18, 14, 22, 16], fill=ink, width=2)
        d.line([17, 15, 12, 17], fill=ink, width=2)
    elif kind == "godco":  # radiant sun over a horizon
        d.ellipse([12, 12, 19, 19], fill=ink)
        for a in range(0, 360, 45):
            r = math.radians(a)
            d.line([15.5 + 5 * math.cos(r), 15.5 + 5 * math.sin(r),
                    15.5 + 7 * math.cos(r), 15.5 + 7 * math.sin(r)],
                   fill=ink, width=1)
        d.line([9, 23, 22, 23], fill=ink, width=2)
    elif kind == "grocery":  # apple
        d.ellipse([11, 14, 20, 24], fill=ink)
        d.line([15, 14, 16, 11], fill=ink, width=2)
        d.polygon([(16, 11), (20, 10), (18, 13)], fill=ink)
        d.point((13, 17), fill=WHITE)
    elif kind == "gun":  # target over a rifle line
        for r in (6, 4, 2):
            d.ellipse([15 - r, 15 - r, 15 + r, 15 + r], outline=ink, width=1)
        d.point((15, 15), fill=ink)
        d.line([9, 25, 22, 22], fill=ink, width=2)
        d.line([19, 23, 22, 25], fill=ink, width=2)
    elif kind == "history":  # classical column
        d.rectangle([10, 11, 21, 13], fill=ink)
        for x in (11, 14, 18):
            d.rectangle([x, 14, x + 1, 21], fill=ink)
        d.rectangle([10, 22, 21, 24], fill=ink)
    elif kind == "mall":  # storefront awning + bag
        d.polygon([(9, 13), (22, 13), (20, 17), (11, 17)], fill=ink)
        for x in (12, 15, 18):
            d.line([x, 13, x - 0.5, 17], fill=WHITE, width=1)
        d.rectangle([11, 19, 20, 25], outline=ink, width=2)
    elif kind == "night":  # crescent moon + star
        d.ellipse([10, 11, 20, 21], fill=ink)
        d.ellipse([14, 9, 23, 19], fill=(38, 32, 64, 255))
        d.polygon([(21, 19), (22, 22), (23, 19), (26, 20), (23, 21),
                   (24, 24), (22, 22), (20, 24), (21, 21), (18, 20)],
                  fill=ink)
    elif kind == "pet":  # paw print
        d.ellipse([12, 17, 19, 23], fill=ink)
        for x, y in ((10, 13), (14, 11), (18, 12), (21, 15)):
            d.ellipse([x, y, x + 3, y + 4], fill=ink)
    elif kind == "restaurant":  # serving cloche
        d.arc([10, 13, 21, 23], 180, 360, fill=ink, width=2)
        d.line([10, 18, 10, 20], fill=ink, width=1)
        d.line([21, 18, 21, 20], fill=ink, width=1)
        d.line([9, 23, 22, 23], fill=ink, width=2)
        d.ellipse([14, 11, 17, 14], fill=ink)
    elif kind == "tech":  # microchip
        d.rectangle([11, 13, 20, 22], outline=ink, width=2)
        d.rectangle([14, 16, 17, 19], fill=ink)
        for t in (12, 15, 18):
            d.line([t, 10, t, 13], fill=ink, width=1)
            d.line([t, 22, t, 25], fill=ink, width=1)
            d.line([8, t + 2, 11, t + 2], fill=ink, width=1)
            d.line([20, t + 2, 23, t + 2], fill=ink, width=1)


# leaflet profiles: (cover colour, motif ink, motif kind)
LEAFLETS = {
    "buylocalpamphlet": ((74, 148, 84, 255), WHITE, "bag"),
    "flyer_evac": ((206, 62, 44, 255), WHITE, "evac"),
    "furniturebrochure": ((146, 106, 66, 255), WHITE, "chair"),
    "getactivepamphlet": ((232, 126, 40, 255), WHITE, "active"),
    "godco_flier": ((238, 232, 216, 255), (196, 156, 44, 255), "godco"),
    "grocerybrochure": ((140, 186, 92, 255), (178, 44, 40, 255), "grocery"),
    "gunnutpamphlet": ((84, 92, 62, 255), WHITE, "gun"),
    "historybrochure": ((178, 152, 108, 255), (84, 62, 40, 255), "history"),
    "mallbrochure": ((196, 92, 152, 255), WHITE, "mall"),
    "nightlifepamphlet": ((38, 32, 64, 255), (240, 214, 110, 255), "night"),
    "petpamphlet": ((72, 156, 160, 255), WHITE, "pet"),
    "restaurantbrochure": ((128, 44, 56, 255), WHITE, "restaurant"),
    "technologybrochure": ((52, 96, 168, 255), WHITE, "tech"),
}


# -------------------------------------------------------------------- maps

def map_base(d, bg):
    """An unfolded map sheet: fold creases and a shaded fold edge."""
    d.rectangle([4, 6, 30, 28], fill=shade(bg, 0.6))
    d.rectangle([3, 5, 29, 27], fill=bg)
    for x in (11, 20):  # vertical fold creases
        d.line([x, 5, x, 27], fill=shade(bg, 0.8), width=1)
    d.line([3, 16, 29, 16], fill=shade(bg, 0.8), width=1)  # horizontal fold
    d.rectangle([3, 5, 29, 6], fill=shade(bg, 1.08))


def road(d, pts, col, w=1):
    d.line(pts, fill=col, width=w, joint="curve")


def draw_map(d, rng, sid):
    grey = (120, 120, 124, 255)
    if sid == "roadmap":
        map_base(d, (238, 234, 206, 255))
        for x in (7, 15, 24):  # road grid
            road(d, [x, 6, x + rng.choice([-1, 1]), 26], grey)
        for y in (10, 20, 25):
            road(d, [4, y, 28, y + rng.choice([-1, 1])], grey)
        road(d, [5, 24, 12, 18, 18, 19, 27, 9], (206, 52, 44, 255), 2)
        road(d, [4, 8, 12, 12, 20, 11, 28, 14], (84, 140, 200, 255))
        d.ellipse([5, 22, 8, 25], fill=(52, 52, 56, 255))
        d.ellipse([25, 7, 28, 10], outline=(206, 52, 44, 255), width=2)
    elif sid == "militarymap":
        map_base(d, (196, 198, 164, 255))
        for x in range(7, 29, 4):  # survey grid
            d.line([x, 6, x, 26], fill=(150, 152, 118, 255), width=1)
        for y in range(9, 27, 4):
            d.line([4, y, 28, y], fill=(150, 152, 118, 255), width=1)
        for r in (3, 6, 9):  # contour rings around a hill
            d.ellipse([18 - r, 15 - r * 0.7, 18 + r, 15 + r * 0.7],
                      outline=(140, 116, 72, 255), width=1)
        d.line([7, 8, 12, 13], fill=(190, 44, 36, 255), width=2)
        d.line([7, 13, 12, 8], fill=(190, 44, 36, 255), width=2)
        road(d, [4, 24, 14, 20, 28, 22], (96, 96, 92, 255))
    elif sid == "satellitemap":
        map_base(d, (104, 122, 76, 255))
        for _ in range(9):  # terrain blobs: fields, woods, scrub
            x, y = rng.uniform(5, 24), rng.uniform(7, 23)
            c = rng.choice([(84, 104, 58, 255), (140, 124, 82, 255),
                            (116, 138, 84, 255), (92, 84, 60, 255)])
            d.ellipse([x, y, x + rng.uniform(4, 8), y + rng.uniform(3, 6)],
                      fill=c)
        road(d, [4, 20, 12, 16, 20, 18, 28, 10], (92, 140, 196, 255), 2)
        for x in (9, 17, 25):  # photo grid ticks
            d.line([x, 6, x, 8], fill=WHITE, width=1)
    elif sid == "sewermap":
        map_base(d, (168, 178, 182, 255))
        road(d, [5, 9, 15, 9, 15, 20, 27, 20], (52, 84, 130, 255), 3)
        road(d, [8, 26, 8, 14, 22, 14, 22, 7], (70, 110, 150, 255), 3)
        road(d, [5, 23, 18, 23, 24, 26], (96, 96, 100, 255), 2)
        for x, y in ((8, 14), (15, 9), (22, 14), (15, 20)):
            d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(36, 40, 46, 255))
    elif sid in ("subwaymap", "subwaystationmap"):
        map_base(d, (242, 242, 238, 255))
        lines = [((206, 52, 44, 255), [4, 22, 12, 14, 20, 14, 28, 8]),
                 ((52, 96, 196, 255), [4, 8, 14, 12, 22, 10, 28, 16]),
                 ((62, 160, 84, 255), [6, 26, 10, 18, 18, 20, 26, 24]),
                 ((232, 176, 44, 255), [10, 6, 16, 14, 24, 20, 28, 26])]
        if sid == "subwaystationmap":  # one station: platforms + concourse
            d.rectangle([8, 11, 24, 15], fill=(206, 52, 44, 255))
            d.rectangle([8, 19, 24, 23], fill=(52, 96, 196, 255))
            d.rectangle([13, 15, 19, 19], fill=(200, 200, 204, 255),
                        outline=grey)
            d.line([16, 6, 16, 11], fill=grey, width=2)
            d.line([16, 23, 16, 26], fill=grey, width=2)
        else:
            for col, pts in lines:
                road(d, pts, col, 2)
                for i in range(0, len(pts), 2):
                    d.ellipse([pts[i] - 1, pts[i + 1] - 1, pts[i] + 1,
                               pts[i + 1] + 1], fill=WHITE, outline=col)
    elif sid == "trailmap":
        map_base(d, (214, 228, 196, 255))
        for r in (4, 8):  # faint contours
            d.ellipse([20 - r, 12 - r * 0.6, 20 + r, 12 + r * 0.6],
                      outline=(168, 178, 140, 255), width=1)
        pts = [(5, 25), (10, 20), (9, 15), (15, 12), (20, 14), (25, 8)]
        for i in range(len(pts) - 1):  # dashed trail
            x1, y1 = pts[i]
            x2, y2 = pts[i + 1]
            for t in (0.0, 0.35, 0.7):
                d.line([x1 + (x2 - x1) * t, y1 + (y2 - y1) * t,
                        x1 + (x2 - x1) * (t + 0.2), y1 + (y2 - y1) * (t + 0.2)],
                       fill=(150, 84, 44, 255), width=2)
        for x, y in ((7, 10), (13, 22), (24, 20)):  # pine triangles
            d.polygon([(x, y - 3), (x - 2, y + 2), (x + 2, y + 2)],
                      fill=(72, 120, 64, 255))
    elif sid == "touristmap":
        map_base(d, (240, 236, 214, 255))
        road(d, [4, 18, 28, 18], grey)
        road(d, [14, 6, 14, 26], grey)
        road(d, [4, 10, 28, 24], (170, 170, 174, 255))
        d.polygon([(20, 6), (21, 9), (24, 9), (22, 11), (23, 14), (20, 12),
                   (17, 14), (18, 11), (16, 9), (19, 9)],
                  fill=(232, 176, 44, 255))  # star landmark
        d.rectangle([6, 20, 11, 25], fill=(150, 110, 80, 255))
        d.rectangle([20, 20, 26, 24], fill=(110, 140, 170, 255))
        d.ellipse([6, 7, 11, 12], fill=(110, 170, 110, 255))
    elif sid == "survivormap":
        map_base(d, PAPER_AGED)
        pts = [(5, 24), (11, 19), (10, 13), (17, 11), (23, 14)]
        for i in range(len(pts) - 1):  # hand-drawn dashed route in pencil
            x1, y1 = pts[i]
            x2, y2 = pts[i + 1]
            for t in (0.0, 0.4):
                d.line([x1 + (x2 - x1) * t, y1 + (y2 - y1) * t,
                        x1 + (x2 - x1) * (t + 0.3), y1 + (y2 - y1) * (t + 0.3)],
                       fill=INK, width=1)
        d.line([22, 11, 27, 16], fill=(178, 44, 36, 255), width=2)
        d.line([22, 16, 27, 11], fill=(178, 44, 36, 255), width=2)
        for x, y in ((7, 9), (19, 22)):  # sketched pines
            d.polygon([(x, y - 3), (x - 2, y + 2), (x + 2, y + 2)],
                      outline=INK)
        d.arc([4, 20, 12, 28], 270, 360, fill=(84, 140, 200, 255), width=2)
    elif sid == "labmap":
        map_base(d, (226, 232, 236, 255))
        # floor-plan rooms: walls, a door gap, hazard room hatched red
        d.rectangle([6, 8, 16, 16], outline=INK, width=2)
        d.rectangle([16, 8, 27, 16], outline=INK, width=2)
        d.rectangle([6, 16, 20, 25], outline=INK, width=2)
        d.rectangle([20, 16, 27, 25], outline=(178, 44, 36, 255), width=2)
        for t in range(4):
            d.line([21 + t * 2, 24, 24 + t * 2, 17], fill=(178, 44, 36, 255),
                   width=1)
        d.line([11, 16, 13, 16], fill=(226, 232, 236, 255), width=2)
        d.rectangle([8, 10, 12, 12], fill=(150, 170, 180, 255))
        d.rectangle([19, 10, 24, 12], fill=(150, 170, 180, 255))
    elif sid == "restaurantmap":
        map_base(d, (240, 234, 210, 255))
        road(d, [4, 14, 28, 14], grey, 2)
        road(d, [16, 6, 16, 26], grey, 2)
        for x, y in ((8, 9), (22, 9), (9, 20), (23, 21)):  # dining blocks
            d.rectangle([x - 2, y - 2, x + 2, y + 2], fill=(178, 120, 70, 255))
            d.point((x, y), fill=WHITE)
        d.polygon([(16, 4), (17, 7), (20, 7), (18, 9), (19, 12), (16, 10),
                   (13, 12), (14, 9), (12, 7), (15, 7)],
                  fill=(206, 52, 44, 255))  # "you are here" star
    elif sid == "urbexmap":
        map_base(d, (204, 204, 200, 255))
        for box in ((5, 7, 13, 14), (15, 7, 27, 13), (5, 17, 16, 25),
                    (19, 16, 27, 25)):  # derelict building outlines
            d.rectangle(list(box), outline=INK, width=2)
        d.line([5, 17, 16, 25], fill=(150, 60, 44, 255), width=1)
        d.line([5, 25, 16, 17], fill=(150, 60, 44, 255), width=1)
        road(d, [6, 24, 12, 18, 20, 19, 26, 9], (178, 44, 36, 255))
        d.line([21, 8, 25, 12], fill=INK, width=2)
        d.line([21, 12, 25, 8], fill=INK, width=2)


MAPS = ["labmap", "militarymap", "restaurantmap", "roadmap",
        "satellitemap", "sewermap", "subwaymap", "subwaystationmap",
        "survivormap", "touristmap", "trailmap", "urbexmap"]


# ------------------------------------------------------------------- notes

def torn_corner(img, w=7):
    """Cut the top-right corner away to transparency (a torn scrap)."""
    px = img.load()
    for y in range(0, w):
        for x in range(32 - w + y, 32):
            if 0 <= x < 32:
                px[x, y] = (0, 0, 0, 0)


def apply_used(img, rng):
    """Wear for a _used note: stains, crumple creases, scribble, torn corner."""
    d = ImageDraw.Draw(img)
    for _ in range(3):  # opaque tea/blood-brown stains
        x, y = rng.uniform(8, 22), rng.uniform(8, 24)
        c = rng.choice([(186, 158, 110, 255), (168, 138, 92, 255),
                        (150, 118, 78, 255)])
        d.ellipse([x, y, x + rng.uniform(4, 7), y + rng.uniform(3, 5)], fill=c)
    for _ in range(4):  # crumple creases
        x1, y1 = rng.uniform(5, 26), rng.uniform(3, 28)
        d.line([x1, y1, x1 + rng.uniform(-8, 8), y1 + rng.uniform(-8, 8)],
               fill=(196, 186, 160, 255), width=1)
    for _ in range(3):  # lines scribbled out hard in dark ink
        y = rng.uniform(11, 24)
        d.line([8, y, 24, y + rng.uniform(-1, 1)], fill=(30, 28, 34, 255),
               width=2)
    torn_corner(img)


def draw_note(img, rng, num, used):
    """One inner-cabins depths note: aged scrap, cabin sketch, depth waves.

    The series shares one paper style; `num` (1/2/3) sets the tally
    marks and the number of depth-wave lines under the cabin sketch.
    """
    d = ImageDraw.Draw(img)
    d.rectangle([7, 5, 27, 29], fill=shade(PAPER_AGED, 0.6))
    d.rectangle([6, 4, 26, 28], fill=PAPER_AGED)
    d.line([9, 4, 9, 28], fill=(196, 110, 100, 255), width=1)  # margin
    for y in range(9, 27, 4):  # ruled lines
        d.line([7, y, 25, y], fill=(150, 160, 180, 255), width=1)
    # cabin sketch: roof triangle over a square, a door notch
    d.polygon([(12, 8), (20, 8), (16, 5)], outline=INK)
    d.rectangle([13, 8, 19, 12], outline=INK)
    d.rectangle([15, 10, 17, 12], fill=INK)
    # depth waves below the cabin: one more wave per note in the series
    for k in range(num):
        y = 15 + k * 3
        pts = []
        for x in range(11, 22):
            pts.append((x, y + (1 if (x + k) % 4 < 2 else -1)))
        d.line(pts, fill=(52, 84, 130, 255), width=1)
    # tally marks for the note number, top right
    for i in range(num):
        d.line([21 + i * 2, 5, 21 + i * 2, 8], fill=INK, width=1)
    # handwritten lines: short wavy ink strokes on the lower ruled lines
    for row in range(3):
        y = 20 + row * 3 if num == 1 else 23 + row * 2
        if y > 26:
            break
        x = 11
        while x < rng.uniform(19, 24):
            ln = rng.uniform(2, 4)
            d.line([x, y + rng.uniform(-0.5, 0.5), x + ln, y],
                   fill=INK, width=1)
            x += ln + 1.5
    if used:
        apply_used(img, rng)


def draw_xedra(img, rng):
    """XEDRA field notes, used: a stained notebook page, stamped + torn."""
    d = ImageDraw.Draw(img)
    d.rectangle([8, 4, 26, 30], fill=shade((70, 92, 84, 255), 0.6))
    d.rectangle([7, 3, 25, 29], fill=(70, 92, 84, 255))  # notebook cover
    for y in range(6, 28, 4):  # spiral holes down the left edge
        d.ellipse([8, y, 10, y + 2], fill=(30, 38, 34, 255))
    d.rectangle([12, 7, 23, 15], fill=PAPER)  # pasted label
    for i, y in enumerate((9, 11, 13)):  # handwriting on the label
        d.line([13, y, 22 - i * 2, y], fill=INK, width=1)
    d.rectangle([12, 18, 23, 24], outline=(178, 44, 36, 255), width=1)
    d.line([13, 21, 22, 21], fill=(178, 44, 36, 255), width=2)  # stamp bar
    for _ in range(4):  # field scribbles below the stamp
        y = rng.uniform(25, 28)
        d.line([12, y, rng.uniform(18, 23), y], fill=(220, 220, 210, 255),
               width=1)
    apply_used(img, rng)


NOTES = {
    "inner_cabins_depths_note": (1, False),
    "inner_cabins_depths_note_2": (2, False),
    "inner_cabins_depths_note_2_used": (2, True),
    "inner_cabins_depths_note_3": (3, False),
    "inner_cabins_depths_note_3_used": (3, True),
    "inner_cabins_depths_note_used": (1, True),
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in LEAFLETS:
        cover, ink, kind = LEAFLETS[sid]
        if sid == "godco_flier":  # a flyer: one flat sheet, pin at the top
            d.rectangle([8, 4, 25, 30], fill=shade(cover, 0.6))
            d.rectangle([7, 3, 24, 29], fill=cover)
            d.line([9, 9, 22, 9], fill=shade(cover, 0.8), width=1)
            motif(d, kind, ink)
            d.ellipse([14, 1, 18, 5], fill=(196, 52, 44, 255))
            d.point((16, 3), fill=WHITE)
            # caption ticks under the motif
            d.line([10, 26, 21, 26], fill=shade(cover, 0.7), width=1)
        else:
            leaflet_base(d, cover)
            motif(d, kind, ink)
    elif sid in MAPS:
        draw_map(d, rng, sid)
    elif sid in NOTES:
        num, used = NOTES[sid]
        draw_note(img, rng, num, used)
    elif sid == "xedra_field_notes_used":
        draw_xedra(img, rng)
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
    # contact-sheet preview: 6 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item45.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item45.png {prev.size}")


if __name__ == "__main__":
    main()
