#!/usr/bin/env python3
"""ITEM55: paper documents / notes / posters -- CDDA Voxel coverage
audit.  All 27 ids come from items/newspaper.json.

Each item is drawn in its own paper format:

  Manila folders (lab reports): bio_file (top-secret test data,
    folder damaged/stained, red stamp), psych_file (folder of
    psychotherapy transcripts, one passage highlighted),
    security_file (atom-and-rifle insignia, shipping-log rows,
    one out-of-place page).
  Lab protocol sheets (atom symbol header + topic motif):
    lab_file_bio (atom-and-flask), lab_file_portal (atom-and-
    flask + alien geometry), lab_file_psych (atom-and-cross),
    lab_file_sec (atom-and-rifle), lab_file_mutagen_guide
    (Certificate of Analysis: test/result rows, seal, checkmarks
    that fade down the page), lab_file_mutation_testing
    (subject rows whose silhouettes mutate), lab_file_serum_
    testing (serum vials + subject test rows).
  Post-it notes (small sticky square, curled corner, scribble
    motif; colour from the item name): bio yellow (flask),
    blob gray (amoeba), migo pink (mi-go glyph), portal orange
    (swirl), tech red (circuit).
  Missing posters: header band, photo box with a silhouette,
    tear-off phone tabs along the bottom.  adult + child are
    official/printed; cat + dog are handmade (wobbly border,
    crayon accents).
  movie_poster: dark poster, starburst, hero silhouette, title
    glyph band.  newspaper_memorial: newsprint columns, portrait
    box, laurel.  childnote: messy wobbly handwriting + a crayon
    drawing.  note_mutant_alpha_boss ("child's drawing"): torn
    scrap with a faded crayon mutant.  toxic_waste_labnote:
    grimy note with hazard-stripe borders.  valentine_card:
    creased card with a big heart.  wallet_photo: small photo
    with a white border.  xedra_shack_checklist: two-column
    checklist, 7 boxes ticked, 5 left empty (per its text).

Self-contained: the id list is embedded below.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "bio_file",
    "childnote",
    "lab_file_bio",
    "lab_file_mutagen_guide",
    "lab_file_mutation_testing",
    "lab_file_portal",
    "lab_file_psych",
    "lab_file_sec",
    "lab_file_serum_testing",
    "lab_postit_bio",
    "lab_postit_blob",
    "lab_postit_migo",
    "lab_postit_portal",
    "lab_postit_tech",
    "missing_poster_adult",
    "missing_poster_cat",
    "missing_poster_child",
    "missing_poster_dog",
    "movie_poster",
    "newspaper_memorial",
    "note_mutant_alpha_boss",
    "psych_file",
    "security_file",
    "toxic_waste_labnote",
    "valentine_card",
    "wallet_photo",
    "xedra_shack_checklist",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

PAPER = (240, 236, 226, 255)
PAPER_DARK = (206, 200, 184, 255)
INK = (52, 52, 58, 255)
SQUIG = (148, 148, 152, 255)
SQUIG_DARK = (110, 110, 116, 255)
MANILA = (214, 182, 118, 255)
MANILA_DARK = (178, 146, 88, 255)
RED = (196, 48, 40, 255)
GREEN = (62, 148, 74, 255)
TEAL = (44, 140, 132, 255)
PURPLE = (128, 84, 178, 255)
ORANGE = (226, 110, 36, 255)
BROWN = (122, 88, 54, 255)
GRAY_PHOTO = (168, 172, 180, 255)
SILHOUETTE = (70, 70, 78, 255)

SX0, SX1, SY0, SY1 = 5, 26, 2, 29


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


def sheet(d, paper=PAPER, x0=SX0, x1=SX1, y0=SY0, y1=SY1):
    d.rectangle([x0 + 1, y0 + 1, x1 + 1, y1 + 1], fill=PAPER_DARK)
    d.rectangle([x0, y0, x1, y1], fill=paper)
    d.rectangle([x0, y0, x1, y1], outline=INK, width=1)


def squiggles(d, rng, x0, x1, y0, y1, col=SQUIG, step=2):
    for y in range(y0, y1 + 1, step):
        x = x0
        while x < x1 - 2:
            w = rng.randint(2, 5)
            c = col if rng.random() < 0.6 else SQUIG_DARK
            d.line([x, y, min(x + w, x1 - 1), y], fill=c, width=1)
            x += w + 2


def glyph_band(d, x0, x1, y, code, col=INK):
    """Title/title-code as dark glyph blocks (per-char bit patterns)."""
    x = x0
    for ch in code:
        if x > x1 - 2:
            break
        if ch != " ":
            o = ord(ch)
            for col_i in range(2):
                bits = (o >> (col_i * 2)) & 0b111
                for row in range(3):
                    if bits & (1 << row):
                        d.point((x + col_i, y + row), fill=col)
                if bits == 0:
                    d.point((x + col_i, y + 1), fill=col)
        x += 3


def atom(d, cx, cy, r=4, col=INK):
    """Atom symbol: nucleus + three rotated orbit ellipses (as lines)."""
    for k in range(3):
        a0 = k * math.pi / 3.0
        pts = []
        for t in range(13):
            th = t * 2 * math.pi / 12
            ex, ey = r * math.cos(th), r * 0.38 * math.sin(th)
            ca, sa = math.cos(a0), math.sin(a0)
            pts.append((cx + ex * ca - ey * sa, cy + ex * sa + ey * ca))
        d.line(pts, fill=col, width=1)
    d.point((cx, cy), fill=col)
    d.point((cx, cy - 0), fill=col)


def flask(d, cx, cy, col=TEAL):
    d.rectangle([cx - 1, cy - 4, cx + 1, cy - 1], outline=INK, width=1)
    d.polygon([(cx - 1, cy - 1), (cx + 1, cy - 1), (cx + 4, cy + 4),
               (cx - 4, cy + 4)], outline=INK, width=1)
    d.line([cx - 2, cy + 2, cx + 2, cy + 2], fill=col, width=2)


def rifle(d, x0, y, col=INK):
    """Simple rifle profile pointing right."""
    d.line([x0, y + 2, x0 + 9, y], fill=col, width=1)          # barrel/top
    d.line([x0 + 2, y + 3, x0 + 9, y + 2], fill=col, width=1)  # body
    d.polygon([(x0, y + 2), (x0 + 3, y + 3), (x0 + 1, y + 5)],
              fill=col)                                        # stock
    d.line([x0 + 6, y + 2, x0 + 6, y + 4], fill=col, width=1)  # magazine
    d.point((x0 + 10, y), fill=col)


def cross(d, cx, cy, col=RED):
    d.rectangle([cx - 1, cy - 4, cx + 1, cy + 4], fill=col)
    d.rectangle([cx - 3, cy - 1, cx + 3, cy + 1], fill=col)


# ------------------------------------------------------------------ folders

def folder_base(d, rng):
    """Manila folder with papers sticking out of the top."""
    d.rectangle([8, 3, 24, 9], fill=PAPER)                     # papers
    d.rectangle([8, 3, 24, 9], outline=SQUIG_DARK, width=1)
    d.line([10, 5, 18, 5], fill=SQUIG, width=1)
    d.line([10, 7, 20, 7], fill=SQUIG, width=1)
    d.polygon([(4, 8), (14, 8), (16, 11), (27, 11), (27, 29), (4, 29)],
              fill=MANILA)                                     # tab + body
    d.polygon([(4, 8), (14, 8), (16, 11), (27, 11), (27, 29), (4, 29)],
              outline=MANILA_DARK)
    d.line([4, 11, 27, 11], fill=MANILA_DARK, width=1)


def draw_bio_file(d, rng):
    folder_base(d, rng)
    flask(d, 15, 19, col=TEAL)
    d.rectangle([8, 24, 23, 27], outline=RED, width=1)         # stamp box
    glyph_band(d, 10, 22, 25, "SECRET", col=RED)
    # damage from the chaos: torn corner + grime stains
    d.polygon([(24, 29), (27, 29), (27, 26)], fill=(24, 24, 28, 255))
    for sx, sy in ((7, 14), (22, 13), (9, 21)):
        d.point((sx, sy), fill=MANILA_DARK)
        d.point((sx + 1, sy), fill=MANILA_DARK)


def draw_psych_file(d, rng):
    folder_base(d, rng)
    # a transcript page pulled out over the folder front
    d.rectangle([9, 14, 23, 28], fill=PAPER)
    d.rectangle([9, 14, 23, 28], outline=SQUIG_DARK, width=1)
    squiggles(d, rng, 11, 22, 16, 26)
    d.rectangle([11, 20, 21, 22], fill=(246, 226, 130, 255))   # highlight
    squiggles(d, rng, 11, 21, 21, 21)
    d.ellipse([12, 8 + 6, 16, 12 + 6], outline=INK, width=1)   # head profile
    d.point((14, 16), fill=INK)


def draw_security_file(d, rng):
    folder_base(d, rng)
    atom(d, 11, 17, r=3)
    rifle(d, 15, 16)
    # shipping-log / rotation rows
    for y in (22, 24, 26):
        d.line([8, y, 24, y], fill=shade(MANILA_DARK, 0.9), width=1)
        d.point((8 + rng.randint(2, 5), y - 1), fill=INK)
    # the one out-of-place page, slipped at an angle at the corner
    d.polygon([(20, 25), (26, 23), (27, 28), (21, 29)], fill=PAPER)
    d.polygon([(20, 25), (26, 23), (27, 28), (21, 29)],
              outline=RED)
    d.point((24, 26), fill=RED)


# ------------------------------------------------------------- lab protocols

def lab_sheet(d, rng, sid):
    sheet(d)
    atom(d, 9, 5, r=3)
    d.line([14, 3, 25, 3], fill=INK, width=1)
    d.line([14, 5, 22, 5], fill=SQUIG_DARK, width=1)
    d.line([SX0 + 1, 8, SX1 - 1, 8], fill=SQUIG_DARK, width=1)


def draw_lab_bio(d, rng):
    lab_sheet(d, rng, "lab_file_bio")
    flask(d, 15, 15, col=GREEN)
    squiggles(d, rng, SX0 + 2, SX1 - 2, 22, 27)


def draw_lab_portal(d, rng):
    lab_sheet(d, rng, "lab_file_portal")
    flask(d, 10, 14, col=PURPLE)
    # alien geometry: nested non-euclidean triangles + warped grid
    d.polygon([(16, 11), (23, 18), (17, 19)], outline=PURPLE, width=1)
    d.polygon([(18, 13), (21, 17), (18, 17)], outline=TEAL, width=1)
    d.point((19, 16), fill=PURPLE)
    for gx in range(8, 25, 4):
        d.line([gx, 24, gx + 2, 27], fill=SQUIG, width=1)
    squiggles(d, rng, SX0 + 2, 13, 22, 22)


def draw_lab_psych(d, rng):
    lab_sheet(d, rng, "lab_file_psych")
    cross(d, 15, 14)
    d.ellipse([10, 9, 20, 19], outline=INK, width=1)
    squiggles(d, rng, SX0 + 2, SX1 - 2, 22, 27)


def draw_lab_sec(d, rng):
    lab_sheet(d, rng, "lab_file_sec")
    rifle(d, 9, 13)
    atom(d, 21, 20, r=3)
    squiggles(d, rng, SX0 + 2, 15, 23, 27)


def draw_lab_mutagen_guide(d, rng):
    sheet(d)
    d.rectangle([SX0 + 1, SY0 + 1, SX1 - 1, SY0 + 4],
                fill=(226, 230, 236, 255))
    glyph_band(d, SX0 + 3, SX1 - 3, SY0 + 1, "CERT")
    # Certificate of Analysis rows: method squiggle, result, signature
    for i, y in enumerate(range(9, 24, 3)):
        d.line([7, y, 13, y], fill=SQUIG_DARK, width=1)
        d.line([15, y, 18, y], fill=INK, width=1)
        d.line([20, y, 24, y + 1], fill=(90, 90, 160, 255), width=1)
    # checkmarks that slowly appear and fade down the page
    fade = [(52, 52, 58), (110, 110, 116), (160, 160, 164),
            (196, 196, 200), (222, 222, 224), (238, 238, 238)]
    for i, y in enumerate(range(9, 24, 3)):
        c = fade[min(i, len(fade) - 1)] + (255,)
        d.line([24, y - 1, 25, y], fill=c, width=1)
        d.line([25, y, 26, y - 2], fill=c, width=1)
    # seal rosette
    d.ellipse([8, 25, 13, 29], outline=RED, width=1)
    d.point((10, 27), fill=RED)
    d.line([9, 29, 8, 30 - 1], fill=RED, width=1)
    d.line([12, 29, 13, 29], fill=RED, width=1)
    d.line([19, 27, 25, 27], fill=SQUIG_DARK, width=1)


def draw_lab_testing(d, rng, serum=False):
    sheet(d)
    glyph_band(d, SX0 + 3, SX1 - 3, SY0 + 2, "TEST")
    d.line([SX0 + 1, 7, SX1 - 1, 7], fill=SQUIG_DARK, width=1)
    if serum:
        # serum vials along the top of the test table
        for i, vx in enumerate((8, 13, 18, 23)):
            col = (TEAL, GREEN, ORANGE, PURPLE)[i]
            d.rectangle([vx - 1, 9, vx + 1, 13], outline=INK, width=1)
            d.line([vx, 11, vx, 13], fill=col, width=2)
            d.point((vx, 8), fill=INK)
    # subject rows: silhouette + result squiggles; mutation grows rightward
    for i, y in enumerate((16, 20, 24) if serum else (13, 18, 23)):
        d.point((8, y - 2), fill=SILHOUETTE)                    # head
        d.rectangle([7, y - 1, 9, y + 2], fill=SILHOUETTE)      # body
        if not serum:
            # mutations accumulate per row: extra limb / horn / bulk
            if i >= 1:
                d.line([9, y, 12, y - 1], fill=SILHOUETTE, width=1)
            if i >= 2:
                d.line([7, y - 1, 5, y - 3], fill=SILHOUETTE, width=1)
                d.rectangle([6, y - 1, 10, y + 2], fill=SILHOUETTE)
        d.line([14, y, 19, y], fill=SQUIG_DARK, width=1)
        d.line([14, y + 2, 17, y + 2], fill=SQUIG, width=1)
        ok = rng.random() < 0.7
        c = GREEN if ok else RED
        if ok:
            d.line([22, y + 1, 23, y + 2], fill=c, width=1)
            d.line([23, y + 2, 25, y - 1], fill=c, width=1)
        else:
            d.line([22, y - 1, 25, y + 2], fill=c, width=1)
            d.line([25, y - 1, 22, y + 2], fill=c, width=1)


# ------------------------------------------------------------------ post-its

POSTIT = {
    "lab_postit_bio": ((250, 224, 92, 255), "flask"),
    "lab_postit_blob": ((196, 198, 204, 255), "blob"),
    "lab_postit_migo": ((246, 178, 208, 255), "migo"),
    "lab_postit_portal": ((250, 178, 92, 255), "portal"),
    "lab_postit_tech": ((240, 110, 100, 255), "tech"),
}


def draw_postit(d, rng, sid):
    col, key = POSTIT[sid]
    x0, x1, y0, y1 = 7, 24, 7, 24
    d.rectangle([x0 + 1, y0 + 1, x1 + 1, y1 + 1], fill=PAPER_DARK)
    d.rectangle([x0, y0, x1, y1], fill=col)
    d.polygon([(x1 - 4, y1), (x1, y1 - 4), (x1, y1)], fill=shade(col, 0.78))
    d.line([x0 + 2, y0 + 2, x1 - 3, y0 + 2], fill=shade(col, 0.85), width=1)
    cx, cy = 15, 14
    if key == "flask":
        flask(d, cx, cy - 1, col=TEAL)
    elif key == "blob":
        d.ellipse([cx - 5, cy - 4, cx + 4, cy + 3], fill=(120, 124, 132, 255))
        d.ellipse([cx + 2, cy + 1, cx + 7, cy + 5], fill=(120, 124, 132, 255))
        d.point((cx - 2, cy - 1), fill=(60, 62, 68, 255))
        d.point((cx + 1, cy), fill=(60, 62, 68, 255))
    elif key == "migo":
        d.ellipse([cx - 3, cy - 3, cx + 3, cy + 2], fill=(196, 120, 150, 255))
        for lx in (-5, -2, 2, 5):  # jointed legs
            d.line([cx + lx // 2, cy + 1, cx + lx, cy + 5],
                   fill=INK, width=1)
        d.line([cx - 2, cy - 3, cx - 5, cy - 6], fill=INK, width=1)
        d.line([cx + 2, cy - 3, cx + 5, cy - 6], fill=INK, width=1)
        d.point((cx - 1, cy - 1), fill=INK)
        d.point((cx + 1, cy - 1), fill=INK)
    elif key == "portal":
        for r in (5, 3, 1):
            d.ellipse([cx - r, cy - 5 + r - 1, cx + r, cy + 3],
                      outline=PURPLE, width=1)
        d.point((cx, cy - 1), fill=PURPLE)
    elif key == "tech":
        d.rectangle([cx - 4, cy - 4, cx + 1, cy + 1], outline=INK, width=1)
        for px in (-3, -1):
            d.line([cx + px, cy - 4, cx + px, cy - 6], fill=INK, width=1)
            d.line([cx + px, cy + 1, cx + px, cy + 3], fill=INK, width=1)
        d.line([cx + 1, cy - 2, cx + 5, cy - 2], fill=INK, width=1)
        d.point((cx + 5, cy - 2), fill=INK)
        d.line([cx + 1, cy, cx + 4, cy], fill=INK, width=1)
        d.point((cx - 2, cy - 2), fill=INK)
    # two scribble lines under the motif
    for y in (21, 23):
        x = x0 + 3
        while x < x1 - 4:
            w = rng.randint(2, 4)
            d.line([x, y, x + w, y], fill=shade(col, 0.55), width=1)
            x += w + 2


# ------------------------------------------------------------ missing posters

def silhouette_person(d, cx, y0, scale=1.0, col=SILHOUETTE):
    r = max(1, int(2 * scale))
    d.ellipse([cx - r, y0, cx + r, y0 + 2 * r], fill=col)
    w = max(1, int(3 * scale))
    h = int(6 * scale)
    d.polygon([(cx - w, y0 + 2 * r + h), (cx - w + 1, y0 + 2 * r),
               (cx + w - 1, y0 + 2 * r), (cx + w, y0 + 2 * r + h)],
              fill=col)


def poster_base(d, rng, handmade=False):
    sheet(d)
    d.rectangle([SX0 + 1, SY0 + 1, SX1 - 1, SY0 + 5],
                fill=(52, 52, 58, 255) if not handmade else PAPER)
    glyph_band(d, SX0 + 3, SX1 - 3, SY0 + 2, "MISSING",
               col=(240, 240, 240, 255) if not handmade else RED)
    # photo box
    d.rectangle([10, 8, 21, 19], fill=GRAY_PHOTO)
    d.rectangle([10, 8, 21, 19], outline=INK, width=1)
    # tear-off phone tabs along the bottom
    for tx in range(SX0 + 2, SX1 - 1, 3):
        d.line([tx, 26, tx, 29], fill=SQUIG_DARK, width=1)
        d.point((tx + 1, 27), fill=SQUIG_DARK)
    if handmade:  # wobbly crayon border accents
        for px, py in ((SX0, SY0), (SX1, SY0), (SX0, SY1), (SX1, SY1)):
            d.point((px, py), fill=ORANGE)


def draw_missing_adult(d, rng):
    poster_base(d, rng)
    silhouette_person(d, 15, 10, scale=1.1)
    squiggles(d, rng, SX0 + 2, SX1 - 2, 21, 24)


def draw_missing_child(d, rng):
    poster_base(d, rng)
    silhouette_person(d, 15, 12, scale=0.65)
    squiggles(d, rng, SX0 + 2, SX1 - 2, 21, 24)


def draw_missing_cat(d, rng):
    poster_base(d, rng, handmade=True)
    cx, cy = 15, 15
    d.ellipse([cx - 5, cy - 1, cx + 3, cy + 3], fill=SILHOUETTE)  # body
    d.ellipse([cx + 2, cy - 4, cx + 6, cy], fill=SILHOUETTE)      # head
    d.polygon([(cx + 2, cy - 4), (cx + 3, cy - 6), (cx + 4, cy - 4)],
              fill=SILHOUETTE)                                   # ears
    d.polygon([(cx + 5, cy - 4), (cx + 6, cy - 6), (cx + 7, cy - 4)],
              fill=SILHOUETTE)
    d.arc([cx - 8, cy - 5, cx - 3, cy + 2], start=90, end=270,
          fill=SILHOUETTE, width=2)                              # tail
    for lx in (-3, 1):
        d.line([cx + lx, cy + 3, cx + lx, cy + 5], fill=SILHOUETTE,
               width=1)
    d.point((cx + 3, cy - 2), fill=(120, 220, 120, 255))         # eyes
    d.point((cx + 5, cy - 2), fill=(120, 220, 120, 255))
    squiggles(d, rng, SX0 + 2, SX1 - 2, 21, 24, col=(200, 140, 90, 255))


def draw_missing_dog(d, rng):
    poster_base(d, rng, handmade=True)
    cx, cy = 15, 14
    d.rectangle([cx - 5, cy, cx + 3, cy + 3], fill=SILHOUETTE)    # body
    d.rectangle([cx + 3, cy - 2, cx + 6, cy + 2], fill=SILHOUETTE)  # head
    d.polygon([(cx + 3, cy - 2), (cx + 5, cy - 2), (cx + 4, cy + 1)],
              fill=SILHOUETTE)                                   # ear
    d.point((cx + 5, cy), fill=(240, 240, 240, 255))             # eye
    d.line([cx - 5, cy, cx - 7, cy - 2], fill=SILHOUETTE, width=2)  # tail
    for lx in (-4, 2):
        d.line([cx + lx, cy + 3, cx + lx, cy + 6], fill=SILHOUETTE,
               width=1)
    d.line([cx + 3, cy + 3, cx + 6, cy + 3], fill=RED, width=1)  # collar
    squiggles(d, rng, SX0 + 2, SX1 - 2, 21, 24, col=(200, 140, 90, 255))


# ------------------------------------------------------------------- others

def draw_movie_poster(d, rng):
    d.rectangle([4, 1, 27, 30], fill=(30, 28, 44, 255))
    d.rectangle([4, 1, 27, 30], outline=INK, width=1)
    # starburst behind the hero
    cx, cy = 15, 13
    for k in range(8):
        a = k * math.pi / 4
        d.line([cx, cy, cx + 9 * math.cos(a), cy + 8 * math.sin(a)],
               fill=(120, 90, 160, 255), width=1)
    d.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], fill=(226, 160, 60, 255))
    silhouette_person(d, cx, cy - 4, scale=1.2, col=(24, 22, 34, 255))
    # title glyph band + credits block
    d.rectangle([6, 22, 25, 26], fill=(16, 14, 26, 255))
    glyph_band(d, 8, 24, 23, "MOVIE", col=(250, 210, 90, 255))
    for y in (28, 29):
        d.line([8, y, 23, y], fill=(150, 150, 170, 255), width=1)


def draw_newspaper_memorial(d, rng):
    sheet(d, paper=(236, 232, 222, 255))
    d.rectangle([SX0 + 1, SY0 + 1, SX1 - 1, SY0 + 4],
                fill=(210, 206, 196, 255))
    glyph_band(d, SX0 + 3, SX1 - 3, SY0 + 1, "NEWS")
    d.line([SX0 + 1, SY0 + 5, SX1 - 1, SY0 + 5], fill=INK, width=1)
    # portrait of the enshrined, centre top
    d.rectangle([12, 8, 19, 16], fill=GRAY_PHOTO)
    d.rectangle([12, 8, 19, 16], outline=INK, width=1)
    silhouette_person(d, 15, 9, scale=0.75)
    # laurel dots flanking the portrait
    for ly in range(9, 16, 2):
        d.point((10, ly), fill=GREEN)
        d.point((21, ly), fill=GREEN)
    # two columns of body text
    squiggles(d, rng, SX0 + 2, 14, 18, 27)
    squiggles(d, rng, 17, SX1 - 2, 18, 27)


def draw_childnote(d, rng):
    sheet(d)
    # messy handwriting: wobbly jittered lines
    for y in range(6, 17, 3):
        x = SX0 + 3
        while x < SX1 - 4:
            w = rng.randint(2, 4)
            jy = rng.randint(-1, 1)
            d.line([x, y + jy, x + w, y + rng.randint(-1, 1)],
                   fill=(90, 90, 140, 255), width=1)
            x += w + 2
    # crayon drawing: house + sun
    d.rectangle([10, 21, 17, 27], outline=ORANGE, width=1)
    d.polygon([(9, 21), (13, 18), (18, 21)], outline=RED, width=1)
    d.point((13, 24), fill=BROWN)
    d.ellipse([21, 19, 24, 22], outline=(230, 180, 40, 255), width=1)
    for dx, dy in ((-1, -1), (4, -1), (-1, 3), (4, 3)):
        d.point((22 + dx, 20 + dy), fill=(230, 180, 40, 255))


def draw_mutant_note(d, rng):
    # faded child's drawing on a torn scrap
    d.polygon([(6, 5), (25, 4), (27, 24), (20, 28), (7, 26), (4, 14)],
              fill=(232, 228, 216, 255))
    d.polygon([(6, 5), (25, 4), (27, 24), (20, 28), (7, 26), (4, 14)],
              outline=SQUIG_DARK)
    faded = (178, 150, 168, 255)   # washed-out crayon purple
    faded2 = (160, 178, 150, 255)
    cx, cy = 15, 12
    d.ellipse([cx - 4, cy - 4, cx + 4, cy + 3], fill=faded)      # bulky body
    d.ellipse([cx - 2, cy - 7, cx + 2, cy - 4], fill=faded)      # head
    d.point((cx - 1, cy - 6), fill=(120, 90, 110, 255))
    d.point((cx + 1, cy - 6), fill=(120, 90, 110, 255))
    for sx in (-1, 1):  # oversized claw arms (the "boss")
        d.line([cx + sx * 4, cy - 1, cx + sx * 7, cy + 3],
               fill=faded, width=2)
        for fx in range(3):
            d.line([cx + sx * 7, cy + 3,
                    cx + sx * (6 + fx), cy + 6], fill=faded, width=1)
        d.line([cx + sx * 2, cy + 3, cx + sx * 3, cy + 8],
               fill=faded, width=2)                              # legs
    d.line([8, 23, 22, 23], fill=faded2, width=1)                # ground
    for gx in (9, 13, 19, 23):
        d.point((gx, 22), fill=faded2)


def draw_toxic_labnote(d, rng):
    sheet(d, paper=(226, 220, 196, 255))
    # hazard stripes top and bottom
    for x in range(SX0, SX1, 4):
        d.polygon([(x, SY0), (x + 2, SY0), (x + 3, SY0 + 2),
                   (x + 1, SY0 + 2)], fill=(240, 200, 40, 255))
        d.polygon([(x + 2, SY0), (x + 4, SY0), (x + 3, SY0 + 2),
                   (x + 1, SY0 + 2)], fill=INK)
        d.polygon([(x, SY1 - 2), (x + 2, SY1 - 2), (x + 3, SY1),
                   (x + 1, SY1)], fill=(240, 200, 40, 255))
        d.polygon([(x + 2, SY1 - 2), (x + 4, SY1 - 2), (x + 3, SY1),
                   (x + 1, SY1)], fill=INK)
    # toxic barrel motif
    d.rectangle([12, 9, 19, 16], fill=(240, 200, 40, 255))
    d.rectangle([12, 9, 19, 16], outline=INK, width=1)
    d.line([12, 11, 19, 11], fill=INK, width=1)
    d.line([12, 14, 19, 14], fill=INK, width=1)
    d.point((15, 12), fill=INK)
    d.point((14, 13), fill=INK)
    d.point((16, 13), fill=INK)
    squiggles(d, rng, SX0 + 2, SX1 - 2, 19, 25, col=(130, 126, 100, 255))
    # grime
    for gx, gy in ((7, 6), (23, 20), (9, 24), (20, 7)):
        d.point((gx, gy), fill=(150, 138, 96, 255))
        d.point((gx + 1, gy), fill=(150, 138, 96, 255))


def draw_valentine(d, rng):
    d.rectangle([7, 5, 24, 27], fill=(248, 214, 224, 255))
    d.rectangle([7, 5, 24, 27], outline=(196, 90, 120, 255), width=1)
    d.line([15, 5, 15, 27], fill=(232, 178, 196, 255), width=1)  # crease
    # big heart
    cx, cy = 15, 13
    d.ellipse([cx - 6, cy - 4, cx, cy + 2], fill=RED)
    d.ellipse([cx, cy - 4, cx + 6, cy + 2], fill=RED)
    d.polygon([(cx - 6, cy), (cx + 6, cy), (cx, cy + 8)], fill=RED)
    d.point((cx - 3, cy - 2), fill=(250, 170, 170, 255))
    # small hearts + squiggle message
    for hx, hy in ((9, 22), (21, 22)):
        d.ellipse([hx - 1, hy - 1, hx, hy + 1], fill=(226, 110, 140, 255))
        d.ellipse([hx, hy - 1, hx + 1, hy + 1], fill=(226, 110, 140, 255))
        d.point((hx, hy + 2), fill=(226, 110, 140, 255))
    d.line([11, 24, 20, 24], fill=(196, 90, 120, 255), width=1)


def draw_wallet_photo(d, rng):
    # small photo with a white border, slightly askew feel via shadow
    d.rectangle([7, 9, 26, 24], fill=PAPER_DARK)
    d.rectangle([6, 8, 25, 23], fill=(250, 250, 248, 255))
    d.rectangle([6, 8, 25, 23], outline=SQUIG_DARK, width=1)
    # scene inside: sky, sun, ground, a person
    d.rectangle([8, 10, 23, 17], fill=(150, 190, 226, 255))
    d.rectangle([8, 17, 23, 21], fill=(120, 170, 96, 255))
    d.ellipse([20, 11, 22, 13], fill=(250, 220, 90, 255))
    silhouette_person(d, 13, 12, scale=0.62, col=(60, 60, 90, 255))
    silhouette_person(d, 17, 13, scale=0.5, col=(120, 60, 70, 255))


def draw_xedra_checklist(d, rng):
    sheet(d)
    d.rectangle([SX0 + 1, SY0 + 1, SX1 - 1, SY0 + 4],
                fill=(226, 230, 236, 255))
    glyph_band(d, SX0 + 3, SX1 - 3, SY0 + 1, "TODO")
    # the list from its text: 7 done, 5 not; two columns of six
    done = [True, True, True, False, True, False,
            True, True, True, False, False, False]
    for i, is_done in enumerate(done):
        col_i, row_i = divmod(i, 6)
        x = SX0 + 2 + col_i * 11
        y = 9 + row_i * 3
        d.rectangle([x, y, x + 2, y + 2], outline=INK, width=1)
        if is_done:
            d.line([x, y, x + 2, y + 2], fill=GREEN, width=1)
            d.line([x + 2, y, x, y + 2], fill=GREEN, width=1)
        d.line([x + 4, y + 1, x + 8, y + 1], fill=SQUIG_DARK, width=1)
    d.line([SX0 + 2, 28, SX1 - 6, 28], fill=SQUIG, width=1)


# -------------------------------------------------------------------- main

DRAW = {
    "bio_file": draw_bio_file,
    "psych_file": draw_psych_file,
    "security_file": draw_security_file,
    "lab_file_bio": draw_lab_bio,
    "lab_file_portal": draw_lab_portal,
    "lab_file_psych": draw_lab_psych,
    "lab_file_sec": draw_lab_sec,
    "lab_file_mutagen_guide": draw_lab_mutagen_guide,
    "missing_poster_adult": draw_missing_adult,
    "missing_poster_child": draw_missing_child,
    "missing_poster_cat": draw_missing_cat,
    "missing_poster_dog": draw_missing_dog,
    "movie_poster": draw_movie_poster,
    "newspaper_memorial": draw_newspaper_memorial,
    "childnote": draw_childnote,
    "note_mutant_alpha_boss": draw_mutant_note,
    "toxic_waste_labnote": draw_toxic_labnote,
    "valentine_card": draw_valentine,
    "wallet_photo": draw_wallet_photo,
    "xedra_shack_checklist": draw_xedra_checklist,
}


def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    if sid in DRAW:
        DRAW[sid](d, rng)
    elif sid == "lab_file_mutation_testing":
        draw_lab_testing(d, rng, serum=False)
    elif sid == "lab_file_serum_testing":
        draw_lab_testing(d, rng, serum=True)
    elif sid in POSTIT:
        draw_postit(d, rng, sid)
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
    # contact-sheet preview: 6 cols x 5 rows, 2x scale, dark backdrop
    cols, rows, cell = 6, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1,
                                (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item55.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item55.png {prev.size}")


if __name__ == "__main__":
    main()
