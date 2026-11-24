#!/usr/bin/env python3
"""ITEM60: scarves/headwraps + throat guards, and Hub 01 (RoboFac)
firearms, magazines and parts -- CDDA Voxel coverage audit.

Cluster 1 (armor/scarfs.json, 21):
  Scarves in two worn forms -- a scarf drawn as a coiled neck loop
  with two hanging tails, a headwrap drawn as a wrapped head form
  (cloth dome, wrap bands, mouth band, side tail).  Materials:
    knit cotton (dark gray, ribbed knit lines), patchwork cloth
    (light sewn squares in muted colours), wool (brown, plain
    weave), fur (brown, shaggy jagged edges, light tips), faux fur
    (tan-brown, smoother synthetic sheen).  _long = longer tails,
    _loose = wider, slack drape.
  nun_wimple: white veil -- cloth dome with a face opening draping
    to the shoulders.
  tallit_gadol: a large prayer shawl -- cream drape with blue
    stripes and knotted fringes at the corners.
  Throat guards: rigid curved collar pieces (gorget arc + side
    straps), coloured by material: canvas light gray stitched,
    cut-resistant Kevlar dark woven, mercenary = taller ballistic
    collar panel over a nylon guard with a buckle, nylon light
    blue smooth, plastic dark glossy shell (HEMA gorget).

Cluster 2 (gun/robofac_gun.json, 21): Hub 01 manufacture -- clean
gunmetal-green steel with white accents; XEDRA prototypes darker
and plainer.
  Magazines, sized by capacity: robofac10 (10rd shotgun, dark,
  wide), robofac40 (40rd 5.56, curved box), robofac60 (60rd 9mm
  helical tube with spiral grooves), robofacmultimag (50rd 5.7,
  long slim box), FSP 40mm mags (3rd short fat box, 5rd taller,
  10rd huge coffin profile tapering from 4 columns to 1).
  HWP gun configurations in profile on a shared platform
  (receiver, stock, grip, rail), barrels per the description:
    57: short barrel + integrated silencer;  smg: short barrel +
    silencer, curved 9mm mag;  ar: long barrel + muzzle brake;
    dmr: long barrel + flash hider + scope;  exodii: heavy barrel
    + bipod + threaded muzzle;  shotgun: long thick barrel +
    choke + pump grip;  breacher: the same, barrel shortened.
  XEDRA: xedra10 / xedra40 prototype magazines (dark), xedra_gun_ar
    (long rifled barrel + muzzle brake), xedra_gun_shotgun (long
    unrifled barrel + pump) -- darker, no white accents.
  Parts as themselves: robofac_handguard (integrated front grip),
    robofac_laser_sight (designator module, red lens + beam dot),
    robofac_stock (bullpup adjustable stock with notches).

Self-contained: the id list is embedded below.
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "knit_scarf_headwrap",
    "long_knit_scarf_headwrap",
    "long_patchwork_scarf_headwrap",
    "nun_wimple",
    "patchwork_scarf_headwrap",
    "scarf_faux_fur",
    "scarf_faux_fur_headwrap",
    "scarf_faux_fur_long",
    "scarf_faux_fur_long_headwrap",
    "scarf_faux_fur_long_loose",
    "scarf_faux_fur_loose",
    "scarf_fur_headwrap",
    "scarf_fur_long_headwrap",
    "scarf_headwrap",
    "scarf_long_headwrap",
    "tallit_gadol",
    "throat_guard_canvas",
    "throat_guard_cut_resistant",
    "throat_guard_mercenary",
    "throat_guard_nylon",
    "throat_guard_plastic",
    "robofac10",
    "robofac40",
    "robofac60",
    "robofac_gun_40mm_10rd",
    "robofac_gun_40mm_3rd",
    "robofac_gun_40mm_5rd",
    "robofac_gun_57",
    "robofac_gun_ar",
    "robofac_gun_dmr",
    "robofac_gun_exodii",
    "robofac_gun_shotgun",
    "robofac_gun_shotgun_breach",
    "robofac_gun_smg",
    "robofac_handguard",
    "robofac_laser_sight",
    "robofac_stock",
    "robofacmultimag",
    "xedra10",
    "xedra40",
    "xedra_gun_ar",
    "xedra_gun_shotgun",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.")

INK = (40, 40, 44, 255)
SKIN = (222, 184, 148, 255)
WHITE_ACC = (238, 240, 242, 255)
RED = (206, 44, 36, 255)


def shade(c, f):
    return tuple(max(0, min(255, int(round(v * f)))) for v in c[:3]) + (255,)


# ------------------------------------------------------------- materials
# key -> (base colour, texture kind)
MATS = {
    "knit": ((94, 94, 100, 255), "knit"),
    "patchwork": ((188, 188, 182, 255), "patch"),
    "wool": ((106, 80, 54, 255), "weave"),
    "fur": ((110, 76, 44, 255), "fur"),
    "fauxfur": ((152, 112, 72, 255), "faux"),
    "white": ((238, 236, 230, 255), "weave"),
}
PATCH_COLS = [(168, 158, 140, 255), (140, 152, 168, 255),
              (168, 130, 122, 255), (188, 188, 182, 255),
              (146, 160, 138, 255)]


def texture(d, rng, box, mat):
    """Fill texture inside box (clipped by caller via points only)."""
    base, kind = MATS[mat]
    x0, y0, x1, y1 = box
    if kind == "knit":  # vertical rib columns with dash stitches
        for x in range(x0, x1 + 1, 2):
            d.line([x, y0, x, y1], fill=shade(base, 0.82), width=1)
        for y in range(y0 + 1, y1, 3):
            for x in range(x0 + 1, x1, 3):
                d.point((x, y), fill=shade(base, 1.18))
    elif kind == "weave":
        for y in range(y0, y1 + 1, 2):
            d.line([x0, y, x1, y], fill=shade(base, 0.9), width=1)
    elif kind == "fur":  # shaggy strokes, lighter tips at the bottom
        for _ in range(46):
            x = rng.randint(x0, x1)
            y = rng.randint(y0, y1 - 2)
            d.line([x, y, x + rng.choice((-1, 0, 1)), y + 2],
                   fill=shade(base, rng.uniform(0.75, 1.25)), width=1)
    elif kind == "faux":  # smoother, even sheen bands
        for y in range(y0, y1 + 1, 3):
            d.line([x0, y, x1, y], fill=shade(base, 1.08), width=1)


def fringe(d, rng, x0, x1, y, col, n=5):
    for x in range(x0, x1 + 1, max(1, (x1 - x0) // n)):
        d.line([x, y, x, y + 2], fill=col, width=1)


# --------------------------------------------------------------- scarves

def draw_scarf(d, rng, mat, long=False, loose=False):
    """Coiled neck loop with two hanging tails."""
    base, _ = MATS[mat]
    cx = 16
    spread = 3 if loose else 0
    # tails behind the coil
    t_top, t_bot = 13, 27 if long else 24
    tw = 5 + spread
    for tx in (cx - 6 - spread, cx + 1):
        d.rectangle([tx, t_top, tx + tw - 1, t_bot], fill=base)
        d.rectangle([tx, t_top, tx + tw - 1, t_bot], outline=shade(base, 0.6))
        texture(d, rng, (tx + 1, t_top + 1, tx + tw - 2, t_bot - 1), mat)
        fringe(d, rng, tx + 1, tx + tw - 2, t_bot, shade(base, 0.7))
    if mat == "patchwork":
        for tx in (cx - 6, cx + 1):
            for py in range(t_top + 2, t_bot - 2, 5):
                d.line([tx, py, tx + 4, py], fill=shade(base, 0.7), width=1)
                d.rectangle([tx + 1, py + 1, tx + 3, py + 3],
                            fill=rng.choice(PATCH_COLS))
    # neck coil: stacked ellipse bands
    for i, dy in enumerate((0, 3, 6)):
        col = shade(base, 1.0 if i != 1 else 0.88)
        d.ellipse([cx - 10 - spread, 4 + dy, cx + 10 + spread, 12 + dy],
                  fill=col)
        d.ellipse([cx - 10 - spread, 4 + dy, cx + 10 + spread, 12 + dy],
                  outline=shade(base, 0.6))
    texture(d, rng, (cx - 8, 6, cx + 8, 13), mat)
    if mat == "patchwork":
        for px in range(cx - 8, cx + 8, 5):
            d.rectangle([px, 6 + (px % 3), px + 3, 9 + (px % 3)],
                        fill=rng.choice(PATCH_COLS))
    if mat in ("fur", "fauxfur"):  # shaggy coil edge
        for x in range(cx - 10, cx + 11, 2):
            d.point((x, 3), fill=shade(base, 1.15))
            d.point((x, 18), fill=shade(base, 0.85))


def draw_headwrap(d, rng, mat, long=False):
    """Wrapped head form: cloth dome, wrap bands, mouth band, tail."""
    base, _ = MATS[mat]
    cx = 15
    # side tail hanging from the wrap
    tx = cx + 8
    t_bot = 28 if long else 24
    d.rectangle([tx, 12, tx + 4, t_bot], fill=base)
    d.rectangle([tx, 12, tx + 4, t_bot], outline=shade(base, 0.6))
    texture(d, rng, (tx + 1, 13, tx + 3, t_bot - 1), mat)
    fringe(d, rng, tx + 1, tx + 3, t_bot, shade(base, 0.7), n=3)
    # head dome
    d.ellipse([cx - 9, 3, cx + 9, 21], fill=base)
    d.rectangle([cx - 9, 12, cx + 9, 21], fill=base)
    # wrap bands across the dome
    for dy in (6, 9, 12):
        d.arc([cx - 9, dy - 3, cx + 9, dy + 5], start=0, end=180,
              fill=shade(base, 0.78), width=1)
        d.arc([cx - 9, dy - 2, cx + 9, dy + 6], start=0, end=180,
              fill=shade(base, 1.12), width=1)
    # face gap: eyes slit, then the mouth band pulled up
    d.rectangle([cx - 6, 14, cx + 5, 15], fill=shade(SKIN, 0.92))
    d.point((cx - 3, 14), fill=INK)
    d.point((cx + 2, 14), fill=INK)
    d.rectangle([cx - 9, 16, cx + 9, 22], fill=shade(base, 0.94))
    d.line([cx - 9, 16, cx + 9, 16], fill=shade(base, 0.65), width=1)
    texture(d, rng, (cx - 8, 17, cx + 8, 21), mat)
    # knot on top
    d.ellipse([cx - 2, 1, cx + 3, 5], fill=shade(base, 0.9))
    d.ellipse([cx - 2, 1, cx + 3, 5], outline=shade(base, 0.6))
    if mat == "patchwork":
        for px, py in ((cx - 7, 5), (cx + 2, 8), (cx - 4, 18), (cx + 4, 4)):
            d.rectangle([px, py, px + 3, py + 3], fill=rng.choice(PATCH_COLS))
    if mat in ("fur", "fauxfur"):
        for x in range(cx - 9, cx + 10, 2):
            d.point((x, 2), fill=shade(base, 1.15))


def draw_wimple(d, rng):
    base = MATS["white"][0]
    # veil drape to the shoulders
    d.polygon([(16, 2), (25, 8), (27, 26), (5, 26), (7, 8)], fill=base)
    d.polygon([(16, 2), (25, 8), (27, 26), (5, 26), (7, 8)],
              outline=shade(base, 0.72))
    for dx in (-6, 6):  # drape fold lines
        d.line([16 + dx // 2, 10, 16 + dx, 25], fill=shade(base, 0.86),
               width=1)
    # face opening
    d.ellipse([11, 6, 21, 17], fill=SKIN)
    d.ellipse([11, 6, 21, 17], outline=shade(base, 0.72))
    d.point((14, 11), fill=INK)
    d.point((18, 11), fill=INK)
    d.arc([13, 12, 19, 16], start=20, end=160, fill=(170, 110, 96, 255),
          width=1)
    # cloth band across the forehead
    d.arc([11, 5, 21, 12], start=180, end=360, fill=shade(base, 0.9),
          width=2)


def draw_tallit(d, rng):
    base = (240, 234, 218, 255)
    blue = (52, 74, 150, 255)
    # shawl drape: wide rectangle over a shoulder form
    d.polygon([(6, 6), (26, 6), (28, 24), (4, 24)], fill=base)
    d.polygon([(6, 6), (26, 6), (28, 24), (4, 24)], outline=shade(base, 0.7))
    d.line([16, 6, 16, 24], fill=shade(base, 0.88), width=1)  # centre fold
    # blue stripes near both short edges (worn over the head: top band too)
    for x in (7, 24):
        d.line([x, 7, x - 1, 23], fill=blue, width=2)
        d.line([x + 3, 7, x + 2, 23], fill=blue, width=1)
    d.line([8, 8, 24, 8], fill=blue, width=1)
    # tzitzit fringes: knotted strings at the four corners
    for cx, cy in ((5, 24), (27, 24), (6, 6), (26, 6)):
        d.point((cx, cy), fill=INK)
        for s in (-1, 0, 1):
            y2 = cy + 3 if cy > 10 else cy - 3
            if 0 <= y2 < 32:
                d.line([cx, cy, cx + s, y2], fill=shade(base, 0.75), width=1)


# ----------------------------------------------------------- throat guard

def draw_throat_guard(d, rng, kind):
    styles = {
        # kind: (base, top band / detail, tall?)
        "canvas": ((178, 178, 170, 255), "stitch", False),
        "cut_resistant": ((72, 72, 78, 255), "weave", False),
        "mercenary": ((58, 58, 64, 255), "ballistic", True),
        "nylon": ((132, 172, 212, 255), "sheen", False),
        "plastic": ((66, 68, 74, 255), "gloss", True),
    }
    base, detail, tall = styles[kind]
    y0 = 8 if tall else 11
    # collar: a curved gorget arc (front view)
    d.polygon([(5, y0), (27, y0), (25, 20), (20, 24), (12, 24), (7, 20)],
              fill=base)
    d.polygon([(5, y0), (27, y0), (25, 20), (20, 24), (12, 24), (7, 20)],
              outline=shade(base, 0.55))
    d.arc([8, y0 - 2, 24, y0 + 7], start=180, end=360,
          fill=shade(base, 1.12), width=2)  # rolled top rim
    # side straps
    for sx in (2, 27):
        d.rectangle([sx, y0 + 3, sx + 3, y0 + 7], fill=shade(base, 0.8))
    d.rectangle([26, y0 + 4, 29, y0 + 6], fill=(150, 152, 158, 255))  # buckle
    if detail == "stitch":
        for x in range(7, 26, 4):
            d.line([x, y0 + 2, x + 2, y0 + 2], fill=shade(base, 0.7),
                   width=1)
            d.line([x, 21, x + 2, 21], fill=shade(base, 0.7), width=1)
    elif detail == "weave":
        for x in range(7, 26, 3):
            d.line([x, y0 + 2, x, 22], fill=shade(base, 1.25), width=1)
        for y in range(y0 + 3, 23, 3):
            d.line([8, y, 24, y], fill=shade(base, 0.75), width=1)
    elif detail == "ballistic":
        # raised ballistic panel over the nylon guard
        d.rectangle([10, y0 + 3, 22, 19], fill=shade(base, 1.22))
        d.rectangle([10, y0 + 3, 22, 19], outline=shade(base, 0.6))
        for x in range(12, 22, 3):
            d.line([x, y0 + 4, x, 18], fill=shade(base, 0.85), width=1)
        d.line([10, 21, 22, 21], fill=(90, 120, 160, 255), width=1)  # nylon
    elif detail == "sheen":
        d.line([9, y0 + 4, 22, y0 + 4], fill=shade(base, 1.18), width=2)
        d.line([9, 18, 23, 18], fill=shade(base, 0.85), width=1)
    elif detail == "gloss":
        d.line([9, y0 + 3, 20, y0 + 3], fill=(196, 200, 208, 255), width=2)
        d.arc([10, y0 + 4, 22, 20], start=30, end=150,
              fill=shade(base, 1.3), width=1)
        d.line([8, 22, 24, 22], fill=(90, 120, 160, 255), width=1)  # strap


# ------------------------------------------------------------------ guns
GUNMETAL = (92, 106, 90, 255)      # Hub 01 green steel
GUN_DARK = (54, 56, 60, 255)       # XEDRA charcoal
POLY = (196, 198, 202, 255)        # robofac light polymer
POLY_DARK = (84, 86, 92, 255)


def magazine(d, rng, kind):
    """Magazines sized by capacity, upright, feed lips at top."""
    if kind == "helical60":
        # compact horizontal helical tube with spiral grooves
        d.rectangle([4, 12, 27, 20], fill=POLY)
        d.rectangle([4, 12, 27, 20], outline=shade(POLY, 0.55))
        for x in range(6, 27, 3):  # spiral groove ticks, alternating slant
            d.line([x, 13, x + 1, 19], fill=shade(POLY, 0.62), width=1)
        d.line([4, 13, 27, 13], fill=shade(POLY, 1.12), width=1)
        d.rectangle([13, 8, 18, 12], fill=POLY_DARK)  # feed tower
        d.rectangle([14, 6, 17, 8], fill=(176, 140, 60, 255))  # top round
        d.rectangle([6, 21, 10, 23], fill=WHITE_ACC)  # white accent block
        return
    shapes = {
        # kind: (top width, bottom width, top y, bottom y, curve, colour)
        "shot10": (11, 10, 7, 27, 0, POLY_DARK),
        "rifle40": (10, 9, 6, 28, 2, POLY),
        "mag57_50": (8, 8, 5, 29, 0, POLY),
        "fsp3": (13, 12, 10, 24, 0, POLY),
        "fsp5": (13, 12, 8, 27, 0, POLY),
        "fsp10": (15, 7, 5, 29, 0, POLY),   # coffin: wide top, narrow foot
        "xedra10": (11, 10, 7, 27, 0, GUN_DARK),
        "xedra40": (10, 9, 6, 28, 2, GUN_DARK),
    }
    tw, bw, ty, by, curve, col = shapes[kind]
    cx = 16
    d.polygon([(cx - tw // 2, ty), (cx + tw // 2, ty),
               (cx + bw // 2 + curve, by), (cx - bw // 2 + curve, by)],
              fill=col)
    d.polygon([(cx - tw // 2, ty), (cx + tw // 2, ty),
               (cx + bw // 2 + curve, by), (cx - bw // 2 + curve, by)],
              outline=shade(col, 0.5))
    d.rectangle([cx - tw // 2 + 1, ty - 2, cx + tw // 2 - 1, ty],
                fill=shade(col, 0.7))  # feed lips
    if kind in ("shot10", "xedra10"):  # fat shotgun shell at the top
        d.rectangle([cx - 3, ty - 4, cx + 3, ty - 2],
                    fill=(176, 44, 40, 255))
        d.line([cx - 3, ty - 2, cx + 3, ty - 2], fill=(196, 164, 84, 255),
               width=1)
    else:
        d.point((cx, ty - 3), fill=(196, 164, 84, 255))
    # witness holes / round-count dots down the body
    n = (by - ty) // 4
    for i in range(n):
        d.point((cx + curve * i // max(1, n), ty + 4 + i * 4),
                fill=shade(col, 0.55))
    d.rectangle([cx - bw // 2 + curve, by - 2, cx + bw // 2 + curve, by],
                fill=shade(col, 0.72))  # baseplate
    if kind.startswith("xedra"):  # prototype stencil band
        d.rectangle([cx - tw // 2 + 1, ty + 6, cx + tw // 2 - 1, ty + 8],
                    fill=(206, 110, 40, 255))
    elif col == POLY:  # robofac white accent stripe
        d.line([cx - tw // 2 + 1, ty + 3, cx + tw // 2 - 1, ty + 3],
               fill=WHITE_ACC, width=1)


def gun_profile(d, rng, barrel, muzzle, mag="box", accent=True,
                scope=False, bipod=False, pump=False, col=GUNMETAL):
    """Side profile: stock left, barrel right, on a shared platform."""
    dark = shade(col, 0.62)
    light = shade(col, 1.18)
    # stock + receiver
    d.polygon([(2, 13), (9, 12), (9, 19), (3, 20)], fill=dark)  # stock
    d.rectangle([3, 19, 8, 21], fill=dark)  # buttpad foot
    d.rectangle([9, 11, 21, 19], fill=col)  # receiver
    d.rectangle([9, 11, 21, 19], outline=dark)
    d.line([9, 11, 21, 11], fill=light, width=1)  # top rail line
    for rx in range(11, 21, 3):
        d.point((rx, 10), fill=dark)  # rail teeth
    d.rectangle([12, 13, 16, 16], fill=dark)  # ejection / panel
    if accent:
        d.rectangle([17, 13, 20, 15], fill=WHITE_ACC)  # Hub white panel
    # grip + trigger
    d.polygon([(11, 19), (14, 19), (13, 24), (10, 24)], fill=dark)
    d.arc([15, 18, 19, 23], start=270, end=90, fill=dark, width=1)
    # magazine under the receiver
    if mag == "box":
        d.polygon([(16, 19), (20, 19), (21, 26), (17, 26)], fill=dark)
    elif mag == "curved":
        d.polygon([(16, 19), (20, 19), (22, 25), (18, 27), (16, 24)],
                  fill=dark)
    elif mag == "tube":  # shotgun tube under the barrel instead
        d.line([21, 17, 29, 17], fill=dark, width=2)
    # barrel from x=21
    thick = 3 if barrel != "heavy" else 4
    blen = {"short": 4, "long": 8, "heavy": 7}[barrel]
    y = 13
    d.rectangle([21, y, 21 + blen, y + thick - 1], fill=col)
    d.line([21, y, 21 + blen, y], fill=light, width=1)
    mx = 21 + blen
    if pump:
        d.rectangle([22, y + thick, 27, y + thick + 2], fill=dark)
        for px in range(23, 27, 2):
            d.point((px, y + thick + 1), fill=light)
    if muzzle == "silencer":  # fat integrated tube
        d.rectangle([mx - 1, y - 1, mx + 4, y + thick], fill=dark)
        d.line([mx - 1, y - 1, mx + 4, y - 1], fill=light, width=1)
    elif muzzle == "brake":
        d.rectangle([mx - 1, y - 1, mx + 2, y + thick], fill=dark)
        for sx in (mx, mx + 1):
            d.line([sx, y, sx, y + thick - 1], fill=col, width=1)
    elif muzzle == "hider":  # pronged flash hider
        for py in (y - 1, y + 1, y + thick):
            d.line([mx, py, mx + 3, py], fill=dark, width=1)
    elif muzzle == "choke":
        d.polygon([(mx - 1, y), (mx + 2, y - 1), (mx + 2, y + thick),
                   (mx - 1, y + thick - 1)], fill=dark)
    elif muzzle == "threaded":
        for tx in range(mx, mx + 3):
            d.line([tx, y, tx, y + thick - 1], fill=dark, width=1)
    if scope:
        d.rectangle([12, 7, 19, 9], fill=dark)
        d.line([14, 9, 14, 11], fill=dark, width=1)
        d.line([17, 9, 17, 11], fill=dark, width=1)
        d.point((18, 8), fill=(120, 180, 220, 255))  # lens glint
    if bipod:
        d.line([mx - 2, y + thick, mx - 5, 27], fill=dark, width=1)
        d.line([mx - 1, y + thick, mx + 1, 27], fill=dark, width=1)
        d.line([mx - 6, 27, mx - 4, 27], fill=dark, width=1)
        d.line([mx, 27, mx + 2, 27], fill=dark, width=1)


def draw_part(d, rng, kind):
    if kind == "handguard":
        # integrated front grip: rail block + angled finger grip
        d.rectangle([8, 8, 24, 12], fill=GUNMETAL)
        d.rectangle([8, 8, 24, 12], outline=shade(GUNMETAL, 0.6))
        for rx in range(10, 24, 3):
            d.point((rx, 7), fill=shade(GUNMETAL, 0.6))
        d.rectangle([10, 9, 14, 11], fill=WHITE_ACC)
        d.polygon([(13, 12), (18, 12), (16, 23), (11, 23)], fill=shade(
            GUNMETAL, 0.7))
        for gy in (15, 18, 21):  # finger grooves
            d.line([12, gy, 16, gy], fill=shade(GUNMETAL, 0.5), width=1)
    elif kind == "laser":
        # designator module on a rail mount, red lens + beam
        d.rectangle([9, 11, 22, 17], fill=GUN_DARK)
        d.rectangle([9, 11, 22, 17], outline=INK)
        d.rectangle([20, 12, 23, 16], fill=shade(GUN_DARK, 0.7))
        d.ellipse([20, 13, 23, 16], fill=RED)  # lens
        d.line([23, 14, 30, 14], fill=(240, 120, 110, 255), width=1)
        d.point((30, 14), fill=RED)
        d.point((29, 13), fill=RED)
        d.rectangle([12, 17, 19, 19], fill=shade(GUN_DARK, 0.7))  # mount
        for rx in range(12, 20, 2):
            d.point((rx, 20), fill=INK)
        d.rectangle([11, 12, 15, 14], fill=WHITE_ACC)  # Hub accent
    elif kind == "stock":
        # bullpup adjustable stock: body, cheek riser, notches, buttpad
        d.polygon([(6, 11), (24, 11), (26, 14), (26, 21), (8, 21), (6, 17)],
                  fill=GUNMETAL)
        d.polygon([(6, 11), (24, 11), (26, 14), (26, 21), (8, 21), (6, 17)],
                  outline=shade(GUNMETAL, 0.6))
        d.rectangle([8, 8, 20, 11], fill=shade(GUNMETAL, 0.78))  # cheek riser
        for nx in range(10, 23, 3):  # adjustment notches
            d.line([nx, 18, nx, 20], fill=shade(GUNMETAL, 0.5), width=1)
        d.rectangle([24, 10, 27, 22], fill=shade(GUNMETAL, 0.55))  # buttpad
        d.rectangle([10, 13, 15, 16], fill=WHITE_ACC)


# -------------------------------------------------------------------- main

def draw_one(sid):
    rng = random.Random(zlib.crc32(sid.encode()))
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # cluster 1
    if sid == "knit_scarf_headwrap":
        draw_headwrap(d, rng, "knit")
    elif sid == "long_knit_scarf_headwrap":
        draw_headwrap(d, rng, "knit", long=True)
    elif sid == "patchwork_scarf_headwrap":
        draw_headwrap(d, rng, "patchwork")
    elif sid == "long_patchwork_scarf_headwrap":
        draw_headwrap(d, rng, "patchwork", long=True)
    elif sid == "scarf_headwrap":
        draw_headwrap(d, rng, "wool")
    elif sid == "scarf_long_headwrap":
        draw_headwrap(d, rng, "wool", long=True)
    elif sid == "scarf_fur_headwrap":
        draw_headwrap(d, rng, "fur")
    elif sid == "scarf_fur_long_headwrap":
        draw_headwrap(d, rng, "fur", long=True)
    elif sid == "scarf_faux_fur_headwrap":
        draw_headwrap(d, rng, "fauxfur")
    elif sid == "scarf_faux_fur_long_headwrap":
        draw_headwrap(d, rng, "fauxfur", long=True)
    elif sid == "scarf_faux_fur":
        draw_scarf(d, rng, "fauxfur")
    elif sid == "scarf_faux_fur_long":
        draw_scarf(d, rng, "fauxfur", long=True)
    elif sid == "scarf_faux_fur_loose":
        draw_scarf(d, rng, "fauxfur", loose=True)
    elif sid == "scarf_faux_fur_long_loose":
        draw_scarf(d, rng, "fauxfur", long=True, loose=True)
    elif sid == "nun_wimple":
        draw_wimple(d, rng)
    elif sid == "tallit_gadol":
        draw_tallit(d, rng)
    elif sid.startswith("throat_guard_"):
        draw_throat_guard(d, rng, sid[len("throat_guard_"):])
    # cluster 2: magazines
    elif sid == "robofac10":
        magazine(d, rng, "shot10")
    elif sid == "robofac40":
        magazine(d, rng, "rifle40")
    elif sid == "robofac60":
        magazine(d, rng, "helical60")
    elif sid == "robofacmultimag":
        magazine(d, rng, "mag57_50")
    elif sid == "robofac_gun_40mm_3rd":
        magazine(d, rng, "fsp3")
    elif sid == "robofac_gun_40mm_5rd":
        magazine(d, rng, "fsp5")
    elif sid == "robofac_gun_40mm_10rd":
        magazine(d, rng, "fsp10")
    elif sid == "xedra10":
        magazine(d, rng, "xedra10")
    elif sid == "xedra40":
        magazine(d, rng, "xedra40")
    # cluster 2: guns
    elif sid == "robofac_gun_57":
        gun_profile(d, rng, "short", "silencer")
    elif sid == "robofac_gun_smg":
        gun_profile(d, rng, "short", "silencer", mag="curved")
    elif sid == "robofac_gun_ar":
        gun_profile(d, rng, "long", "brake")
    elif sid == "robofac_gun_dmr":
        gun_profile(d, rng, "long", "hider", scope=True)
    elif sid == "robofac_gun_exodii":
        gun_profile(d, rng, "heavy", "threaded", bipod=True)
    elif sid == "robofac_gun_shotgun":
        gun_profile(d, rng, "long", "choke", mag="tube", pump=True)
    elif sid == "robofac_gun_shotgun_breach":
        gun_profile(d, rng, "short", "choke", mag="tube", pump=True)
    elif sid == "xedra_gun_ar":
        gun_profile(d, rng, "long", "brake", accent=False, col=GUN_DARK)
    elif sid == "xedra_gun_shotgun":
        gun_profile(d, rng, "long", "choke", mag="tube", pump=True,
                    accent=False, col=GUN_DARK)
    # cluster 2: parts
    elif sid == "robofac_handguard":
        draw_part(d, rng, "handguard")
    elif sid == "robofac_laser_sight":
        draw_part(d, rng, "laser")
    elif sid == "robofac_stock":
        draw_part(d, rng, "stock")
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
    # contact-sheet preview: 7 cols x 6 rows, 2x scale, dark backdrop
    cols, rows, cell = 7, 6, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, sid in enumerate(IDS):
        big = imgs[sid].resize((64, 64), Image.NEAREST)
        prev.alpha_composite(big, ((i % cols) * cell + 1, (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item60.png")
    prev.convert("RGB").save(out)
    print(f"wrote {len(IDS)} sprites + preview_item60.png {prev.size}")


if __name__ == "__main__":
    main()
