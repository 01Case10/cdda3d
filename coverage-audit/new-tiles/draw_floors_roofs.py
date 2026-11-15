#!/usr/bin/env python3
"""
Draw the missing floor / roof / z-level-transition terrain sprites from
their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/
terrain-floors-indoor.json, terrain-roofs.json,
terrain-zlevel-transitions.json, CDDA 6a3e59e0) -- every entry in those
files whose id is in the shipped-universe missing list
(coverage-audit/missing.json) and has no tile yet: 22 ids.

  t_brick_floor               "A relatively flat area made of tightly
                               placed bricks."  symbol ., light_red
  t_floor_warped              "A rough stone floor.  Its patterns
                               constantly shift..."  symbol ., magenta
  t_floor_warped_nl_glow      same floor, light_emitted 90, pale-green
                               light_color [220,255,200] -- glowing veins
  t_floor_warped_nl_n / _s    same floor (looks_like t_floor_warped)
  t_floor_warped_light        same floor, light_emitted 50 -- lit veins
  t_broken_metal_floor        "Something has cratered this metal
                               flooring."  symbol ., light_cyan
  t_rad_platform              "radiation platform... hoisting the
                               radioactive material stored within"
                               symbol 0, light_blue (looks_like t_blue_floor)
  t_greenhouse                "A greenhouse that allows planting year
                               round."  symbol #, green, PLOWABLE
  t_greenhouse_tilled         "...prepared for planting crops inside."
                               tilled soil mounds, PLANTABLE
  t_warped_roof               "flat, stone-like rooftop... grooves and
                               lines that constantly shift"  magenta
  t_canvas_roof               "A section of canvas rooftop."  blue
  t_treetop                   "The upper branches of a tree.  There's
                               just enough room to perch here."  symbol #,
                               seasonal colors light_green/green/yellow/brown
  t_treetop_evergreen         same perch, evergreen -- green only
  t_treetop_dead              "The bare upper branches of a dead tree."
                               brown
  t_treetop_giant_fern        "The top of some unearthly gigantic fern."
                               red (alien-meadow palette)
  t_earth_ramp_down_high/low  "The upper/lower end of an earth ramp
                               leading down."  symbol >, brown
  t_earth_ramp_up_high/low    "The upper/lower end of an earth ramp
                               leading up."  symbol <, brown
  t_low_stairs_begin          "A flight of stairs leading up." (wood --
                               bash yields 2x4s/nails/splinters)
                               dark_gray_white
  t_low_stairs_end            same flight, end section (RAMP_END)
                               black_white

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet.
Floors and roofs are full-tile surface textures (fg with bg "").
Ramps and stairs read by their shading: a light-to-dark gradient runs
in the travel direction, high ends sit in the bright half of the ramp
gradient and low ends in the dark half, and a chevron states the
direction (down chevrons on the down ramps, up chevrons on the up
ramps); stair treads brighten as the flight rises, and the end section
tops out in a pale landing. Deterministic: every random choice comes
from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json + credits),
plus this batch's own contact sheet, preview_floors_roofs.png.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def full(color):
    return Image.new("RGBA", (S, S), color)

def flecks(d, rng, n, cols, box=(0, 0, S - 1, S - 1)):
    for _ in range(n):
        x = rng.uniform(box[0], box[2]); y = rng.uniform(box[1], box[3])
        d.point((x, y), fill=cols[rng.randrange(len(cols))])

# ---------------------------------------------------------------- brick floor
def draw_brick_floor():
    rng = random.Random(3301)
    img = full((120, 102, 90, 255))  # mortar
    d = ImageDraw.Draw(img)
    cols = [(186, 96, 78, 255), (196, 104, 82, 255), (176, 88, 70, 255),
            (202, 112, 88, 255)]
    course_h, brick_w = 8, 16
    for row in range(S // course_h):
        y0 = row * course_h
        off = (brick_w // 2) if row % 2 else 0
        for bx in range(-1, S // brick_w + 1):
            x0 = bx * brick_w + off
            col = cols[rng.randrange(len(cols))]
            d.rectangle([x0 + 1, y0 + 1, x0 + brick_w - 1, y0 + course_h - 1],
                        fill=col)
            d.line([x0 + 1, y0 + 1, x0 + brick_w - 1, y0 + 1],
                   fill=(col[0] + 18, col[1] + 14, col[2] + 12, 255))
    flecks(d, rng, 30, [(160, 84, 66, 255), (214, 128, 102, 255)])
    return img

# ---------------------------------------------------------------- warped stone
def _warped(rng, base, slab_cols, vein_col, fleck_cols, glow_pts=0):
    img = full(base)
    d = ImageDraw.Draw(img, "RGBA")
    # rough stone slabs: broad jittered patches in alternate tones
    for _ in range(5):
        x = rng.uniform(-4, S - 10); y = rng.uniform(-4, S - 10)
        w = rng.uniform(10, 18); h = rng.uniform(8, 14)
        d.ellipse([x, y, x + w, y + h],
                  fill=slab_cols[rng.randrange(len(slab_cols))])
    # patterns that constantly shift: sine-warped wandering veins
    for v in range(6):
        ph = rng.uniform(0, math.tau)
        x = rng.uniform(-2, 6); y = rng.uniform(0, S)
        pts = []
        for i in range(9):
            pts.append((x + i * 4, y + 3.2 * math.sin(i * 1.1 + ph)
                        + rng.uniform(-1, 1)))
        d.line(pts, fill=vein_col, width=1)
    flecks(d, rng, 46, fleck_cols)
    for _ in range(glow_pts):  # emitted light: bright specks in the veins
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        d.point((x, y), fill=(232, 255, 214, 255))
        d.point((x + 1, y), fill=(190, 235, 170, 255))
    return img

def draw_floor_warped(seed=3302):
    rng = random.Random(seed)
    return _warped(rng, (72, 56, 84, 255),
                   [(84, 64, 98, 255), (62, 48, 74, 255), (78, 60, 92, 255)],
                   (196, 64, 180, 255),
                   [(150, 60, 140, 255), (48, 38, 58, 255), (104, 82, 118, 255)])

def draw_floor_warped_glow():
    rng = random.Random(3303)
    # light_emitted 90 in pale green [220,255,200]: the veins themselves glow
    return _warped(rng, (84, 76, 84, 255),
                   [(96, 88, 96, 255), (74, 66, 76, 255), (90, 84, 90, 255)],
                   (196, 240, 178, 255),
                   [(150, 190, 140, 255), (56, 50, 56, 255), (120, 140, 116, 255)],
                   glow_pts=16)

def draw_floor_warped_light():
    rng = random.Random(3306)
    # light_emitted 50: the shifting magenta veins burn brighter
    return _warped(rng, (78, 60, 90, 255),
                   [(90, 68, 104, 255), (66, 50, 78, 255), (84, 64, 98, 255)],
                   (242, 110, 224, 255),
                   [(170, 76, 158, 255), (50, 40, 60, 255), (112, 88, 126, 255)],
                   glow_pts=6)

def draw_warped_roof():
    rng = random.Random(3311)
    img = full((88, 66, 100, 255))
    d = ImageDraw.Draw(img)
    # flat rooftop: a groove grid whose lines will not sit still
    for gy in range(0, S + 1, 8):
        pts = [(x, gy + 1.6 * math.sin(x / 4.0 + gy)) for x in range(S)]
        d.line(pts, fill=(58, 42, 70, 255), width=1)
    for gx in range(0, S + 1, 8):
        pts = [(gx + 1.6 * math.sin(y / 4.0 + gx), y) for y in range(S)]
        d.line(pts, fill=(58, 42, 70, 255), width=1)
    for v in range(4):  # shifting magenta line accents across the grooves
        ph = rng.uniform(0, math.tau)
        y = rng.uniform(2, S - 2)
        pts = [(x, y + 2.4 * math.sin(x / 5.0 + ph)) for x in range(0, S, 2)]
        d.line(pts, fill=(186, 76, 168, 255), width=1)
    flecks(d, rng, 36, [(112, 88, 126, 255), (66, 50, 78, 255)])
    return img

# ---------------------------------------------------------------- metal floors
def draw_broken_metal_floor():
    rng = random.Random(3307)
    img = full((126, 146, 150, 255))  # light_cyan steel
    d = ImageDraw.Draw(img)
    # plate seams + rivets
    for g in (0, 16):
        d.line([g, 0, g, S - 1], fill=(88, 106, 110, 255))
        d.line([0, g, S - 1, g], fill=(88, 106, 110, 255))
    for (x, y) in [(2, 2), (29, 2), (2, 29), (29, 29), (18, 2), (2, 18),
                   (29, 18), (18, 29)]:
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(70, 86, 90, 255))
    flecks(d, rng, 22, [(146, 166, 170, 255), (108, 128, 132, 255)])
    # the crater: a jagged torn depression, center-low
    cx, cy = 16, 17
    rim = []
    for i in range(12):
        a = i / 12 * math.tau
        r = rng.uniform(6.0, 9.5)
        rim.append((cx + r * math.cos(a), cy + r * 0.82 * math.sin(a)))
    d.polygon(rim, fill=(52, 54, 58, 255))
    inner = [(cx + (x - cx) * 0.55, cy + (y - cy) * 0.55) for x, y in rim]
    d.polygon(inner, fill=(28, 28, 32, 255))
    # torn bright edges + cracks radiating out
    for i in range(0, 12, 2):
        x, y = rim[i]
        d.line([x, y, x + (x - cx) * 0.16, y + (y - cy) * 0.16],
               fill=(204, 224, 228, 255), width=1)
    for _ in range(5):
        a = rng.uniform(0, math.tau)
        x0, y0 = cx + 8 * math.cos(a), cy + 6.5 * math.sin(a)
        d.line([x0, y0, x0 + rng.uniform(3, 6) * math.cos(a),
                y0 + rng.uniform(3, 6) * math.sin(a)],
               fill=(70, 78, 82, 255), width=1)
    flecks(d, rng, 14, [(40, 40, 44, 255), (180, 196, 200, 255)],
           box=(6, 8, 26, 26))
    return img

def draw_rad_platform():
    rng = random.Random(3308)
    img = full((116, 146, 178, 255))  # light_blue steel (looks_like t_blue_floor)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, S - 1, S - 1], outline=(84, 110, 138, 255), width=1)
    flecks(d, rng, 20, [(134, 164, 194, 255), (100, 128, 158, 255)])
    # the platform: a heavy ring (symbol 0) with the hoist disc inside
    d.ellipse([4, 4, 28, 28], fill=(140, 172, 202, 255),
              outline=(56, 78, 102, 255), width=3)
    d.ellipse([10, 10, 22, 22], fill=(96, 128, 160, 255),
              outline=(56, 78, 102, 255), width=1)
    d.ellipse([14, 14, 18, 18], fill=(70, 98, 128, 255))
    for i in range(8):  # bolts around the ring
        a = i / 8 * math.tau
        x, y = 16 + 12 * math.cos(a), 16 + 12 * math.sin(a)
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(52, 72, 96, 255))
    return img

# ---------------------------------------------------------------- greenhouse
def draw_greenhouse():
    rng = random.Random(3309)
    img = full((92, 68, 44, 255))  # planting soil
    d = ImageDraw.Draw(img, "RGBA")
    for y in range(3, S, 6):  # furrows
        d.line([0, y, S - 1, y], fill=(70, 50, 32, 255), width=1)
        d.line([0, y + 2, S - 1, y + 2], fill=(112, 86, 54, 255), width=1)
    # rows of young green growth, year-round
    for y in range(5, S, 6):
        for x in range(3, S, 6):
            jx, jy = x + rng.uniform(-1, 1), y + rng.uniform(-0.6, 0.6)
            col = (110, 190, 80, 255) if rng.random() < 0.5 else (70, 150, 60, 255)
            d.line([jx - 1.6, jy + 1, jx, jy - 1.4], fill=col, width=1)
            d.line([jx + 1.6, jy + 1, jx, jy - 1.4], fill=col, width=1)
    # glasshouse frame at the edges + a pane sheen (sheen on an overlay,
    # composited "over" so the tile stays fully opaque)
    d.rectangle([0, 0, S - 1, S - 1], outline=(168, 178, 168, 255), width=1)
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(ov).polygon([(3, 26), (14, 8), (18, 8), (8, 26)],
                               fill=(210, 230, 220, 42))
    return Image.alpha_composite(img, ov)

def draw_greenhouse_tilled():
    rng = random.Random(3310)
    img = full((84, 60, 38, 255))  # prepared soil
    d = ImageDraw.Draw(img)
    for y in range(0, S, 5):  # tilled mounds: shadow, ridge light, shadow
        d.line([0, y, S - 1, y], fill=(54, 38, 22, 255), width=1)
        d.line([0, y + 2, S - 1, y + 2], fill=(124, 94, 58, 255), width=2)
        d.line([0, y + 4, S - 1, y + 4], fill=(64, 46, 28, 255), width=1)
    for _ in range(26):  # loose clods on the ridges
        x = rng.uniform(0, S - 2); y = rng.uniform(0, S - 2)
        d.ellipse([x, y, x + rng.uniform(1.5, 3), y + rng.uniform(1, 2)],
                  fill=(104, 76, 46, 255))
    flecks(d, rng, 16, [(140, 108, 68, 255), (48, 34, 20, 255)])
    return img

# ---------------------------------------------------------------- canvas roof
def draw_canvas_roof():
    rng = random.Random(3312)
    img = full((62, 84, 150, 255))  # blue canvas
    # gentle sag bands across the sheet (overlay, composited "over" so
    # the tile stays fully opaque)
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for y in range(0, S, 8):
        od.rectangle([0, y + 5, S - 1, y + 7], fill=(48, 66, 120, 70))
        od.rectangle([0, y, S - 1, y + 1], fill=(140, 160, 210, 40))
    img = Image.alpha_composite(img, ov)
    d = ImageDraw.Draw(img)
    # seams with stitching
    for x in range(0, S + 1, 8):
        d.line([x, 0, x, S - 1], fill=(44, 60, 110, 255), width=1)
        for y in range(2, S, 4):
            d.line([x - 1, y, x + 1, y], fill=(150, 168, 214, 255), width=1)
    flecks(d, rng, 120, [(72, 94, 160, 255), (54, 74, 136, 255),
                         (80, 102, 168, 255)])
    return img

# ---------------------------------------------------------------- treetops
def _canopy_blobs(d, rng, cols, n=210, rmax=3.4):
    for _ in range(n):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        r = rng.uniform(1.4, rmax)
        d.ellipse([x - r, y - r * 0.8, x + r, y + r * 0.8],
                  fill=cols[rng.randrange(len(cols))])

def draw_treetop():
    rng = random.Random(3313)
    img = full((30, 72, 26, 255))
    d = ImageDraw.Draw(img)
    # branch hints under the leaves
    for (x0, y0, x1, y1) in [(16, 31, 8, 14), (16, 31, 24, 12), (16, 31, 16, 8)]:
        d.line([x0, y0, x1, y1], fill=(96, 68, 40, 255), width=2)
    # seasonal canopy: greens with yellow and brown turning through it
    _canopy_blobs(d, rng, [(96, 160, 60, 255), (56, 128, 44, 255),
                           (74, 144, 50, 255), (196, 170, 60, 255),
                           (120, 84, 44, 255), (44, 104, 36, 255)])
    # sunlit clusters toward the top
    _canopy_blobs(d, rng, [(126, 184, 76, 255), (210, 184, 74, 255)], n=36,
                  rmax=2.6)
    return img

def draw_treetop_evergreen():
    rng = random.Random(3314)
    img = full((20, 56, 30, 255))
    d = ImageDraw.Draw(img)
    cols = [(40, 110, 52, 255), (58, 140, 70, 255), (30, 88, 44, 255),
            (70, 156, 82, 255)]
    # needle clusters: short strokes radiating in stars
    for _ in range(46):
        cx = rng.uniform(1, S - 2); cy = rng.uniform(1, S - 2)
        col = cols[rng.randrange(len(cols))]
        for _ in range(5):
            a = rng.uniform(0, math.tau); ln = rng.uniform(1.6, 3.2)
            d.line([cx, cy, cx + ln * math.cos(a), cy + ln * math.sin(a)],
                   fill=col, width=1)
    flecks(d, rng, 20, [(16, 44, 24, 255), (86, 168, 96, 255)])
    return img

def draw_treetop_dead():
    rng = random.Random(3315)
    img = full((72, 60, 52, 255))  # shadowed dead-canopy backdrop
    d = ImageDraw.Draw(img)
    cols = [(110, 92, 76, 255), (88, 72, 58, 255)]
    hi = (142, 122, 100, 255)
    def branch(x, y, ang, ln, w, depth):
        ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
        col = cols[depth % 2]
        d.line([x, y, ex, ey], fill=col, width=w)
        if w >= 2:
            d.line([x - 0.7, y, ex - 0.7, ey], fill=hi, width=1)
        if depth <= 0:
            return
        branch(ex, ey, ang + rng.uniform(-0.55, -0.2), ln * 0.68,
               max(1, w - 1), depth - 1)
        branch(ex, ey, ang + rng.uniform(0.2, 0.55), ln * 0.68,
               max(1, w - 1), depth - 1)
        if rng.random() < 0.4:
            branch(ex, ey, ang + rng.uniform(-0.15, 0.15), ln * 0.55,
                   max(1, w - 1), depth - 1)
    for bx in (9, 16, 23):  # bare limbs reaching up, nothing on them
        branch(bx, S, rng.uniform(-0.2, 0.2), 9, 3, 3)
    flecks(d, rng, 14, [(58, 48, 42, 255), (96, 82, 68, 255)])
    return img

def draw_treetop_giant_fern():
    rng = random.Random(3316)
    img = full((38, 14, 16, 255))  # dark red shadow under the fronds
    d = ImageDraw.Draw(img)
    cols = [(150, 40, 44, 255), (120, 30, 36, 255), (186, 60, 54, 255)]
    def frond(x, y, ang, ln, col):
        ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
        pts = []
        for i in range(9):
            t = i / 8
            px = x + (ex - x) * t
            py = y + (ey - y) * t + 3.4 * math.sin(t * math.pi) * 0.4
            pts.append((px, py))
        d.line(pts, fill=col, width=2)
        for i in range(1, 8):  # pinnae
            px, py = pts[i]
            pl = ln * 0.26 * (1 - i / 10)
            for sgn in (-1, 1):
                a = ang + sgn * 1.05
                d.line([px, py, px + math.sin(a) * pl, py - math.cos(a) * pl],
                       fill=col, width=1)
    for a in (-1.15, -0.7, -0.3, 0.05, 0.45, 0.85, 1.2):
        frond(16, 30, a + rng.uniform(-0.06, 0.06), rng.uniform(19, 25),
              cols[rng.randrange(len(cols))])
    return img

# ---------------------------------------------------------------- earth ramps
def _ramp_gradient(d, lo, hi):
    """Per-row earth gradient: t=0 at the top row uses hi, t=1 lo... the
    caller passes (top_color, bottom_color)."""
    for y in range(S):
        t = y / (S - 1)
        col = tuple(int(hi[c] + (lo[c] - hi[c]) * t) for c in range(3)) + (255,)
        d.line([0, y, S - 1, y], fill=col)

def _chevrons(d, up, col):
    for cy in (10, 21):
        if up:
            d.line([10, cy + 3, 16, cy - 3], fill=col, width=2)
            d.line([16, cy - 3, 22, cy + 3], fill=col, width=2)
        else:
            d.line([10, cy - 3, 16, cy + 3], fill=col, width=2)
            d.line([16, cy + 3, 22, cy - 3], fill=col, width=2)

def draw_earth_ramp(seed, high_end, up):
    rng = random.Random(seed)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 255))
    d = ImageDraw.Draw(img)
    # translucent marks (striations, chevrons) go on an overlay that is
    # composited "over" at the end, so the tile stays fully opaque
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    light, dark = (156, 120, 80), (94, 68, 42)
    if high_end:  # the bright half of the slope
        top = tuple(int(dark[c] + (light[c] - dark[c]) * 0.85) for c in range(3))
        bot = tuple(int(dark[c] + (light[c] - dark[c]) * 0.45) for c in range(3))
    else:         # the dark half
        top = tuple(int(dark[c] + (light[c] - dark[c]) * 0.5) for c in range(3))
        bot = dark
    # ramps up rise toward the top of the tile; ramps down fall away
    if up:
        _ramp_gradient(d, bot, top)
    else:
        _ramp_gradient(d, top, bot)
    # packed-earth striations across the slope
    for y in range(2, S, 4):
        pts = [(x, y + rng.uniform(-0.7, 0.7)) for x in range(0, S, 4)]
        od.line(pts, fill=(0, 0, 0, 46), width=1)
    flecks(d, rng, 40, [(170, 134, 92, 255), (80, 58, 36, 255),
                        (128, 98, 62, 255)])
    for _ in range(6):  # embedded stones
        x = rng.uniform(2, S - 4); y = rng.uniform(2, S - 4)
        d.ellipse([x, y, x + rng.uniform(1.6, 3), y + rng.uniform(1.2, 2.2)],
                  fill=(138, 128, 116, 255))
    _chevrons(od, up, (52, 36, 20, 160))
    return Image.alpha_composite(img, ov)

# ---------------------------------------------------------------- low stairs
def draw_low_stairs(seed, end):
    rng = random.Random(seed)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 255))
    d = ImageDraw.Draw(img)
    # translucent marks (edge light, seams, grain) go on an overlay that
    # is composited "over" at the end, so the tile stays fully opaque
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    # wooden treads: bands brighten as the flight rises (toward the top)
    if end:   # top section: brighter overall, pale landing at the top
        lo, hi = (138, 130, 120), (196, 190, 180)
    else:     # beginning section: the flight climbs out of shadow
        lo, hi = (94, 86, 78), (152, 144, 132)
    bands = 5
    bh = S // bands
    for b in range(bands):
        t = b / (bands - 1)  # b=0 is the bottom band
        col = tuple(int(lo[c] + (hi[c] - lo[c]) * t) for c in range(3)) + (255,)
        y1 = S - 1 - b * bh
        y0 = y1 - bh + 1
        d.rectangle([0, y0, S - 1, y1], fill=col)
        # riser shadow under each tread's front edge
        d.line([0, y1, S - 1, y1], fill=(40, 36, 32, 255), width=1)
        od.line([0, y0, S - 1, y0], fill=(255, 255, 255, 46), width=1)
        # plank seams + grain + nail heads
        od.line([10, y0 + 1, 10, y1 - 1], fill=(0, 0, 0, 40), width=1)
        od.line([22, y0 + 1, 22, y1 - 1], fill=(0, 0, 0, 40), width=1)
        for _ in range(3):
            gx = rng.uniform(1, S - 8); gy = rng.uniform(y0 + 1, y1 - 1)
            od.line([gx, gy, gx + rng.uniform(3, 7), gy], fill=(0, 0, 0, 52),
                    width=1)
        d.point((3, y0 + 2), fill=(52, 50, 48, 255))
        d.point((28, y0 + 2), fill=(52, 50, 48, 255))
    img = Image.alpha_composite(img, ov)
    if end:  # RAMP_END: the landing the flight arrives at
        d2 = ImageDraw.Draw(img)
        d2.rectangle([0, 0, S - 1, 3], fill=(206, 200, 190, 255))
        d2.line([0, 4, S - 1, 4], fill=(60, 56, 50, 255), width=1)
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg) -- floors, roofs and transitions are all
# full-tile surfaces in this batch: fg is the tile itself, bg "".
JOBS = [
    ("t_brick_floor", draw_brick_floor(), "t_brick_floor", ""),
    ("t_floor_warped", draw_floor_warped(), "t_floor_warped", ""),
    ("t_floor_warped_nl_glow", draw_floor_warped_glow(),
     "t_floor_warped_nl_glow", ""),
    ("t_floor_warped_nl_n", draw_floor_warped(3304), "t_floor_warped_nl_n", ""),
    ("t_floor_warped_nl_s", draw_floor_warped(3305), "t_floor_warped_nl_s", ""),
    ("t_floor_warped_light", draw_floor_warped_light(), "t_floor_warped_light", ""),
    ("t_broken_metal_floor", draw_broken_metal_floor(), "t_broken_metal_floor", ""),
    ("t_rad_platform", draw_rad_platform(), "t_rad_platform", ""),
    ("t_greenhouse", draw_greenhouse(), "t_greenhouse", ""),
    ("t_greenhouse_tilled", draw_greenhouse_tilled(), "t_greenhouse_tilled", ""),
    ("t_warped_roof", draw_warped_roof(), "t_warped_roof", ""),
    ("t_canvas_roof", draw_canvas_roof(), "t_canvas_roof", ""),
    ("t_treetop", draw_treetop(), "t_treetop", ""),
    ("t_treetop_evergreen", draw_treetop_evergreen(), "t_treetop_evergreen", ""),
    ("t_treetop_dead", draw_treetop_dead(), "t_treetop_dead", ""),
    ("t_treetop_giant_fern", draw_treetop_giant_fern(), "t_treetop_giant_fern", ""),
    ("t_earth_ramp_down_high", draw_earth_ramp(3317, True, False),
     "t_earth_ramp_down_high", ""),
    ("t_earth_ramp_down_low", draw_earth_ramp(3318, False, False),
     "t_earth_ramp_down_low", ""),
    ("t_earth_ramp_up_high", draw_earth_ramp(3319, True, True),
     "t_earth_ramp_up_high", ""),
    ("t_earth_ramp_up_low", draw_earth_ramp(3320, False, True),
     "t_earth_ramp_up_low", ""),
    ("t_low_stairs_begin", draw_low_stairs(3321, False), "t_low_stairs_begin", ""),
    ("t_low_stairs_end", draw_low_stairs(3322, True), "t_low_stairs_end", ""),
]

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img, fg, bg in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": fg, "bg": bg}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

CELL = 64
COLS = 6
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img, _fg, _bg) in enumerate(JOBS):
    t = tile_img(tid).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_floors_roofs.png"))
print("wrote preview_floors_roofs.png", sheet.size)
