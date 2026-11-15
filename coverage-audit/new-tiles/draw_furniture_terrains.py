#!/usr/bin/env python3
"""
Draw the missing furniture-terrains sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/
furniture-terrains.json, CDDA 6a3e59e0) -- 51 furniture ids in that file
are missing from UltiCa in the shipped universe
(coverage-audit/missing.json, category "furniture"):

Cardboard set (all color brown):
  f_cardboard_fort   "A fort built by tipping a cardboard box on its
                      side, lining it with blankets, and partly
                      weather-sealing it with a plastic sheet."  sym ^
  f_cardboard_wall   "A pile of cardboard boxes that have been filled
                      with rags and junk and stacked together like
                      bricks to form a wall."  sym #
  f_cardboard_floor  "A flat floor made of sheets of cardboard
                      duct-taped together, covering a plastic sheet
                      on the ground."  sym _
  f_cardboard_roof   "A sloped roof made of cardboard boxes... Sheets
                      of plastic are carefully arranged on top to
                      shed water."  sym _
  f_cardboard_door   the box wall, largest box cut and folded into a
                      makeshift door, closed.  sym +
  f_cardboard_door_o same, door open.  sym .

Mound family (symbol #, heaped material; short = coverage 40 "heaped,
not easily traversable", mid = coverage 60 "heaped up to your
shoulders", tall = coverage 100 "heaped well above your head"):
  f_sand_mound_*        sand, color yellow
  f_gravel_mound_*      gravel, color light_gray
  f_pebble_mound_*      pebbles, color light_gray
  f_clay_mound_*        clay, color brown
  f_glass_mound_*       broken glass shards, color light_cyan, SHARP
  f_rock_mound_*        rocks, color dark_gray
  f_rock_large_mound_*  large rocks, color dark_gray

Stack family (symbol #, same short/mid/tall height language):
  f_log_stack_*         logs, brown -- "A stack of logs as high as
                        your shoulders" (mid)
  f_plank_stack_*       planks (2x4s), brown
  f_stick_stack_*       sticks, brown
  f_stick_long_stack_*  long sticks, brown
  f_brick_stack_*       bricks, brown (red-brown brick)
  f_adobe_stack_*       adobe bricks, brown (tan earthen blocks)
  f_rebar_stack_*       rebar, light_gray
  f_pipe_stack_*        pipes, light_gray

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet. Furniture draws over the terrain beneath it, so every entry is
fg with bg "". Piles sit on the tile's ground line (bottom row);
height reads from the profile: short tops out around y=20 (low heap),
mid around y=11 (shoulder height), tall around y=3 (over the head).
Mounds use one parameterized heap renderer (silhouette mask +
per-material texture field); stacks use one parameterized row
renderer per unit type (log ends, planks, brick courses, adobe
blocks, rebar rods, pipe ends, stick bundles). Flat fills, dark
outlines, one highlight pass -- UltiCa style. Solid surfaces are
fully opaque; translucent-looking marks (glints, sheen) are opaque
light tones or composited overlays, never raw alpha. Deterministic:
every random choice comes from a per-sprite seeded Random.
Output: pngs_normal_32x32/furniture/<id>/ (png + tile-entry json +
credits).
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32
GROUND = 31  # bottom row of the tile: piles sit on this line

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3)) + (255,)

# ================================================================ heaps
# height name -> (top y of the heap silhouette)
HEAP_TOP = {"short": 20, "mid": 11, "tall": 3}

def heap_mask(top, rng=None, jag=0.0, base_hw=15.5):
    """Silhouette of a heap: rounded cone from the ground line to top."""
    mask = Image.new("L", (S, S), 0)
    d = ImageDraw.Draw(mask)
    pts = []
    n = 16
    for i in range(n + 1):  # left side, base -> peak
        v = i / n
        hw = base_hw * (1 - v) ** 0.72
        y = GROUND - v * (GROUND - top)
        if jag and v > 0.55 and rng is not None:
            y += rng.uniform(-jag, jag) * (v - 0.55) * 2
        pts.append((16 - hw, y))
    for i in range(n, -1, -1):  # right side, peak -> base
        v = i / n
        hw = base_hw * (1 - v) ** 0.72
        y = GROUND - v * (GROUND - top)
        if jag and v > 0.55 and rng is not None:
            y += rng.uniform(-jag, jag) * (v - 0.55) * 2
        pts.append((16 + hw, y))
    d.polygon(pts, fill=255)
    return mask, pts

def heap_base(rng, top_col, bot_col, top):
    """Full-tile vertical gradient field the heap texture draws over."""
    img = Image.new("RGBA", (S, S), top_col)
    d = ImageDraw.Draw(img)
    for y in range(S):
        t = max(0.0, min(1.0, (y - top) / max(1, GROUND - top)))
        d.line([(0, y), (S - 1, y)], fill=lerp(top_col, bot_col, t))
    return img

def finish_heap(mask, pts, tex, outline=(38, 32, 28, 255)):
    out = new_img()
    out.paste(tex, (0, 0), mask)
    d = ImageDraw.Draw(out)
    d.line(pts + [pts[0]], fill=outline, width=1)
    return out

def speckle(d, rng, n, cols, y0=0, y1=GROUND, x0=0, x1=S - 1):
    for _ in range(n):
        x = rng.uniform(x0, x1); y = rng.uniform(y0, y1)
        d.point((x, y), fill=cols[rng.randrange(len(cols))])

# ---------------------------------------------------------------- sand
def tex_sand(rng):
    img = heap_base(rng, (224, 186, 108, 255), (188, 148, 76, 255), 0)
    d = ImageDraw.Draw(img)
    speckle(d, rng, 190, [(236, 204, 132, 255), (204, 164, 90, 255),
                          (176, 136, 66, 255)])
    for _ in range(7):  # wind ripples
        y = rng.uniform(4, 28); x = rng.uniform(2, 18)
        d.arc([x, y, x + rng.uniform(8, 13), y + 3], start=200, end=330,
              fill=(198, 158, 84, 255), width=1)
    return img

# ---------------------------------------------------------------- gravel
def tex_gravel(rng):
    img = heap_base(rng, (158, 158, 162, 255), (116, 116, 122, 255), 0)
    d = ImageDraw.Draw(img)
    cols = [(132, 132, 138, 255), (172, 172, 176, 255), (108, 108, 114, 255),
            (186, 184, 178, 255), (146, 140, 132, 255)]
    for _ in range(150):  # small angular stones
        x = rng.uniform(0, S - 3); y = rng.uniform(0, S - 3)
        w = rng.uniform(1.6, 3.2); h = w * rng.uniform(0.6, 0.9)
        c = cols[rng.randrange(len(cols))]
        d.polygon([(x, y + h * 0.5), (x + w * 0.4, y), (x + w, y + h * 0.4),
                   (x + w * 0.6, y + h)], fill=c)
    speckle(d, rng, 60, [(196, 196, 200, 255), (96, 96, 102, 255)])
    return img

# ---------------------------------------------------------------- pebble
def tex_pebble(rng):
    img = heap_base(rng, (110, 108, 104, 255), (84, 82, 80, 255), 0)
    d = ImageDraw.Draw(img)
    cols = [(168, 166, 160, 255), (150, 146, 138, 255), (182, 176, 166, 255),
            (138, 132, 124, 255), (160, 148, 128, 255)]
    for _ in range(95):  # distinct rounded stones
        x = rng.uniform(0, S - 4); y = rng.uniform(0, S - 3)
        w = rng.uniform(2.6, 4.4); h = w * rng.uniform(0.62, 0.8)
        c = cols[rng.randrange(len(cols))]
        d.ellipse([x, y, x + w, y + h], fill=c, outline=(70, 68, 64, 255))
        d.arc([x + 0.6, y + 0.4, x + w * 0.62, y + h * 0.6], start=180,
              end=300, fill=(206, 202, 192, 255), width=1)
    return img

# ---------------------------------------------------------------- clay
def tex_clay(rng):
    img = heap_base(rng, (164, 110, 70, 255), (128, 82, 48, 255), 0)
    d = ImageDraw.Draw(img)
    for _ in range(46):  # soft lumps
        x = rng.uniform(0, S - 5); y = rng.uniform(0, S - 4)
        w = rng.uniform(3.5, 6.5); h = w * rng.uniform(0.45, 0.65)
        c = [(148, 96, 58, 255), (178, 122, 80, 255),
             (136, 88, 52, 255)][rng.randrange(3)]
        d.ellipse([x, y, x + w, y + h], fill=c)
    for _ in range(6):  # drying cracks
        x = rng.uniform(3, 26); y = rng.uniform(3, 24)
        pts = [(x, y)]
        for _ in range(3):
            x += rng.uniform(-3, 3); y += rng.uniform(1.5, 3.5)
            pts.append((x, y))
        d.line(pts, fill=(96, 60, 34, 255), width=1)
    speckle(d, rng, 50, [(186, 132, 88, 255), (112, 72, 42, 255)])
    return img

# ---------------------------------------------------------------- glass
def tex_glass(rng):
    img = heap_base(rng, (172, 216, 218, 255), (128, 178, 184, 255), 0)
    d = ImageDraw.Draw(img)
    cols = [(150, 202, 208, 255), (192, 230, 232, 255), (128, 182, 190, 255),
            (206, 238, 238, 255)]
    for _ in range(90):  # angular shards
        x = rng.uniform(0, S - 4); y = rng.uniform(0, S - 4)
        w = rng.uniform(2.5, 5); h = rng.uniform(2, 4)
        c = cols[rng.randrange(len(cols))]
        d.polygon([(x, y + h), (x + w * 0.5, y), (x + w, y + h * 0.7)],
                  fill=c, outline=(104, 152, 160, 255))
    return img

def add_glints(img, mask, rng, top, n=10):
    d = ImageDraw.Draw(img)
    tries = 0
    while n > 0 and tries < 300:
        tries += 1
        x = rng.uniform(2, S - 3)
        y = rng.uniform(top, min(S - 2, top + 14))
        if mask.getpixel((int(x), int(y))) == 0:
            continue
        n -= 1
        d.line([(x - 1.6, y), (x + 1.6, y)], fill=(250, 254, 254, 255))
        d.line([(x, y - 1.6), (x, y + 1.6)], fill=(250, 254, 254, 255))
        d.point((x, y), fill=(255, 255, 255, 255))

# ---------------------------------------------------------------- rock
def _rock_field(rng, n, wmin, wmax, base_top, base_bot):
    img = heap_base(rng, base_top, base_bot, 0)
    d = ImageDraw.Draw(img)
    cols = [(122, 122, 130, 255), (104, 104, 112, 255), (140, 138, 144, 255),
            (92, 92, 100, 255)]
    for _ in range(n):
        x = rng.uniform(0, S - wmax); y = rng.uniform(0, S - wmax * 0.7)
        w = rng.uniform(wmin, wmax); h = w * rng.uniform(0.6, 0.8)
        c = cols[rng.randrange(len(cols))]
        pts = [(x + w * 0.15, y + h), (x, y + h * 0.45), (x + w * 0.3, y),
               (x + w * 0.75, y + h * 0.12), (x + w, y + h * 0.55),
               (x + w * 0.8, y + h)]
        d.polygon(pts, fill=c, outline=(58, 58, 64, 255))
        d.line([(x + w * 0.3, y + 1), (x + w * 0.55, y + h * 0.5)],
               fill=(168, 166, 172, 255), width=1)  # facet highlight
    return img

def tex_rock(rng):
    return _rock_field(rng, 42, 3.5, 6.5, (116, 116, 124, 255),
                       (84, 84, 92, 255))

def tex_rock_large(rng):
    return _rock_field(rng, 15, 7, 11, (112, 112, 120, 255),
                       (80, 80, 88, 255))

MOUND_TEX = {
    "sand": tex_sand, "gravel": tex_gravel, "pebble": tex_pebble,
    "clay": tex_clay, "glass": tex_glass, "rock": tex_rock,
    "rock_large": tex_rock_large,
}

def draw_mound(mat, height, seed):
    rng = random.Random(seed)
    top = HEAP_TOP[height]
    jag = 1.6 if mat == "glass" else 0.0
    mask, pts = heap_mask(top, rng, jag=jag)
    tex = MOUND_TEX[mat](rng)
    out = finish_heap(mask, pts, tex,
                      outline=(52, 44, 36, 255) if mat != "glass"
                      else (70, 110, 116, 255))
    if mat == "glass":
        add_glints(out, mask, rng, top)
    return out

# ================================================================ stacks
def rows_for(height, short, mid, tall):
    return {"short": short, "mid": mid, "tall": tall}[height]

# ---------------------------------------------------------------- logs
def draw_log_stack(height, seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    counts = rows_for(height, [3, 2], [4, 3, 2], [4, 3, 3, 2])
    dia, pitch = 8, 7
    yb = GROUND
    for row, n in enumerate(counts):
        cy = yb - dia / 2 - row * pitch + row  # slight nestling
        x0 = 16 - (n * dia) / 2 + (row % 2) * 1.5
        for i in range(n):
            cx = x0 + dia / 2 + i * dia + rng.uniform(-0.4, 0.4)
            d.ellipse([cx - dia / 2, cy - dia / 2, cx + dia / 2, cy + dia / 2],
                      fill=(104, 72, 44, 255), outline=(62, 40, 24, 255))
            r = dia / 2 - 1.3
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      fill=(198, 160, 110, 255))
            for rr in (r * 0.62, r * 0.32):  # growth rings
                d.ellipse([cx - rr, cy - rr, cx + rr, cy + rr],
                          outline=(172, 130, 82, 255))
            d.point((cx, cy), fill=(140, 102, 60, 255))
    return img

# ---------------------------------------------------------------- pipes
def draw_pipe_stack(height, seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    counts = rows_for(height, [3, 2], [3, 3, 2], [3, 3, 2, 2])
    dia, pitch = 8, 7
    for row, n in enumerate(counts):
        cy = GROUND - dia / 2 - row * pitch + row
        x0 = 16 - (n * dia) / 2 + (row % 2) * 1.5
        for i in range(n):
            cx = x0 + dia / 2 + i * dia + rng.uniform(-0.3, 0.3)
            d.ellipse([cx - dia / 2, cy - dia / 2, cx + dia / 2, cy + dia / 2],
                      fill=(138, 144, 152, 255), outline=(70, 74, 82, 255))
            d.arc([cx - dia / 2 + 0.7, cy - dia / 2 + 0.7,
                   cx + dia / 2 - 0.7, cy + dia / 2 - 0.7],
                  start=180, end=290, fill=(186, 192, 200, 255), width=1)
            r = dia / 2 - 1.8  # the hollow bore
            d.ellipse([cx - r, cy - r, cx + r, cy + r],
                      fill=(36, 38, 44, 255), outline=(96, 102, 110, 255))
    return img

# ---------------------------------------------------------------- planks
def draw_plank_stack(height, seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    n = rows_for(height, 3, 6, 8)
    bh, pitch = 4, 3.6
    for i in range(n):
        y1 = GROUND - i * pitch
        y0 = y1 - bh
        w = rng.uniform(25, 29)
        x0 = 16 - w / 2 + rng.uniform(-0.8, 0.8)
        base = [(172, 132, 84, 255), (182, 142, 92, 255),
                (164, 124, 78, 255)][rng.randrange(3)]
        d.rectangle([x0, y0, x0 + w, y1], fill=base,
                    outline=(96, 68, 38, 255))
        for _ in range(3):  # grain
            gy = rng.uniform(y0 + 0.8, y1 - 0.8)
            d.line([(x0 + 1.5, gy), (x0 + w - 1.5, gy)],
                   fill=(148, 110, 66, 255), width=1)
        d.line([(x0 + 1, y0 + 0.7), (x0 + w * 0.4, y0 + 0.7)],
               fill=(204, 168, 116, 255), width=1)  # top highlight
    return img

# ---------------------------------------------------------------- bricks
def _courses(height, course_h, short, mid, tall):
    return rows_for(height, short, mid, tall), course_h

def draw_brick_stack(height, seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    n, ch = _courses(height, 4, 3, 5, 7)
    mortar = (198, 182, 162, 255)
    cols = [(160, 76, 54, 255), (172, 86, 60, 255), (148, 66, 46, 255),
            (166, 80, 62, 255)]
    bw = 9
    for c in range(n):
        y1 = GROUND - c * ch
        y0 = y1 - ch
        inset = min(3.0, c * 0.55)
        x = 1.5 + inset + (bw / 2 if c % 2 else 0)
        d.rectangle([1 + inset, y0, S - 2 - inset, y1], fill=mortar)
        while x + bw <= S - 1 - inset:
            col = cols[rng.randrange(len(cols))]
            d.rectangle([x, y0 + 0.6, x + bw - 1, y1 - 0.4], fill=col)
            d.line([(x + 0.8, y0 + 1.1), (x + bw * 0.5, y0 + 1.1)],
                   fill=(196, 116, 92, 255), width=1)
            x += bw
    return img

# ---------------------------------------------------------------- adobe
def draw_adobe_stack(height, seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    n = rows_for(height, 2, 4, 6)
    ch, bw = 5, 12
    mortar = (208, 180, 134, 255)
    for c in range(n):
        y1 = GROUND - c * ch
        y0 = y1 - ch
        inset = min(2.5, c * 0.5)
        d.rectangle([2 + inset, y0, S - 3 - inset, y1], fill=mortar)
        x = 2.5 + inset + (bw / 2 if c % 2 else 0)
        while x + bw <= S - 2 - inset:
            base = [(190, 148, 98, 255), (182, 140, 90, 255),
                    (198, 156, 106, 255)][rng.randrange(3)]
            d.rectangle([x, y0 + 0.6, x + bw - 1, y1 - 0.4], fill=base)
            for _ in range(4):  # straw flecks + grit
                fx = rng.uniform(x + 1, x + bw - 2)
                fy = rng.uniform(y0 + 1, y1 - 1)
                d.point((fx, fy), fill=(226, 200, 124, 255) if rng.random() < 0.5
                        else (150, 114, 68, 255))
            x += bw
    return img

# ---------------------------------------------------------------- rebar
def draw_rebar_stack(height, seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    n = rows_for(height, 3, 6, 9)
    pitch = 3
    for i in range(n):
        y1 = GROUND - i * pitch
        y0 = y1 - 2.4
        ln = rng.uniform(25, 29)
        x0 = 16 - ln / 2 + rng.uniform(-0.7, 0.7)
        d.rectangle([x0, y0, x0 + ln, y1], fill=(148, 148, 154, 255),
                    outline=(84, 84, 90, 255))
        x = x0 + 2  # rib ticks
        while x < x0 + ln - 1:
            d.line([(x, y0 + 0.3), (x, y1 - 0.3)], fill=(110, 110, 116, 255),
                   width=1)
            x += 3
        if rng.random() < 0.55:  # rust patch
            rx = rng.uniform(x0 + 2, x0 + ln - 6)
            d.line([(rx, y0 + 0.5), (rx + rng.uniform(2.5, 5), y0 + 0.5)],
                   fill=(156, 100, 62, 255), width=1)
    return img

# ---------------------------------------------------------------- sticks
def draw_stick_stack(height, seed, long_sticks=False):
    """A bundle of individual sticks laid across each other: each stick
    is one long line spanning most of the pile's width at its layer,
    with shadowed gaps between layers and a few diagonal crossers."""
    rng = random.Random(seed)
    top = {"short": 21, "mid": 13, "tall": 5}[height]
    thw = {"short": 9, "mid": 11, "tall": 13}[height]
    bhw = {"short": 13.5, "mid": 15, "tall": 15.8}[height]
    layers = {"short": 4, "mid": 6, "tall": 9}[height]
    img = new_img(); d = ImageDraw.Draw(img)
    poly = [(16 - bhw, GROUND), (16 - thw, top), (16 + thw, top),
            (16 + bhw, GROUND)]
    d.polygon(poly, fill=(66, 48, 30, 255))  # shadowed gaps between sticks
    cols = [(130, 94, 54, 255), (150, 110, 64, 255), (112, 80, 44, 255),
            (164, 124, 76, 255)]
    def hw_at(y):
        t = (GROUND - y) / max(1, GROUND - top)
        return bhw + (thw - bhw) * t
    for i in range(layers):
        y = GROUND - 1.5 - i * ((GROUND - top) / layers)
        hw = hw_at(y)
        # one or two sticks per layer, ends jittered, slight tilt
        for _ in range(2 if rng.random() < 0.4 else 1):
            tilt = rng.uniform(-1.2, 1.2)
            x0 = 16 - hw + rng.uniform(-0.5, 1.5)
            x1 = 16 + hw + rng.uniform(-1.5, 0.5)
            if long_sticks:  # ends protrude past the pile
                x0 -= rng.uniform(0, 1.6); x1 += rng.uniform(0, 1.6)
            col = cols[rng.randrange(len(cols))]
            d.line([(x0, y + tilt), (x1, y - tilt)], fill=col, width=2)
            d.point((x0, y + tilt), fill=(198, 162, 116, 255))  # cut end
            d.point((x1, y - tilt), fill=(198, 162, 116, 255))
    # diagonal crossers lashed over the top
    for _ in range(3 if not long_sticks else 2):
        x0 = rng.uniform(5, 12); x1 = rng.uniform(20, 27)
        y0 = rng.uniform(top + 4, top + 9); y1 = y0 + rng.uniform(5, 9)
        d.line([(x0, y0), (x1, min(y1, GROUND - 2))],
               fill=cols[rng.randrange(len(cols))], width=2)
    d.line(poly + [poly[0]], fill=(52, 38, 22, 255), width=1)
    return img

STACK_DRAW = {
    "log": draw_log_stack, "pipe": draw_pipe_stack,
    "plank": draw_plank_stack, "brick": draw_brick_stack,
    "adobe": draw_adobe_stack, "rebar": draw_rebar_stack,
}

# ================================================================ cardboard
CB_BASE = (178, 140, 94, 255)
CB_DARK = (140, 106, 66, 255)
CB_LIGHT = (200, 164, 114, 255)
CB_EDGE = (96, 72, 44, 255)
TAPE = (156, 156, 158, 255)

def _box(d, x0, y0, x1, y1, rng, base=CB_BASE):
    d.rectangle([x0, y0, x1, y1], fill=base, outline=CB_EDGE)
    d.line([(x0 + 1, (y0 + y1) / 2), (x1 - 1, (y0 + y1) / 2)],
           fill=CB_DARK, width=1)  # flap seam
    if rng.random() < 0.6:  # tape strip
        tx = rng.uniform(x0 + 2, x1 - 4)
        d.rectangle([tx, y0 + 0.6, tx + 2.2, y1 - 0.6], fill=TAPE)

def draw_cardboard_floor(seed):
    rng = random.Random(seed)
    img = Image.new("RGBA", (S, S), CB_BASE)
    d = ImageDraw.Draw(img)
    for _ in range(90):  # cardboard fibre speckle
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=CB_LIGHT if rng.random() < 0.5 else CB_DARK)
    # sheet seams: a 2x2 patchwork of flattened sheets
    d.line([(16, 0), (16, S - 1)], fill=CB_DARK, width=1)
    d.line([(0, 15), (S - 1, 15)], fill=CB_DARK, width=1)
    # duct tape over the seams
    d.rectangle([14.6, 0, 17.4, S - 1], fill=TAPE)
    d.rectangle([0, 13.6, S - 1, 16.4], fill=TAPE)
    d.line([(16, 0), (16, S - 1)], fill=(178, 178, 180, 255), width=1)
    d.line([(0, 15), (S - 1, 15)], fill=(178, 178, 180, 255), width=1)
    # plastic sheet underneath peeking at one corner
    d.polygon([(0, 26), (6, 31), (0, 31)], fill=(206, 202, 190, 255))
    d.rectangle([0, 0, S - 1, S - 1], outline=CB_EDGE, width=1)
    return img

def draw_cardboard_roof(seed):
    rng = random.Random(seed)
    img = Image.new("RGBA", (S, S), CB_DARK)
    d = ImageDraw.Draw(img)
    # sloped, shingled layers of stuffed boxes, top (ridge) to bottom (eave)
    rows = 6
    for i in range(rows):
        y0 = 1 + i * 5.2
        y1 = y0 + 5.6
        base = lerp(CB_LIGHT, CB_BASE, i / rows)
        d.rectangle([0, y0, S - 1, y1], fill=base)
        d.line([(0, y1 - 0.6), (S - 1, y1 - 0.6)], fill=CB_EDGE, width=1)
        x = rng.uniform(6, 12)  # box joints
        while x < S - 2:
            d.line([(x, y0 + 0.6), (x, y1 - 1)], fill=CB_DARK, width=1)
            x += rng.uniform(8, 13)
    # plastic sheets arranged on top to shed water: pale sheen bands
    sheen = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    sd = ImageDraw.Draw(sheen)
    for i in range(3):
        y0 = 2 + i * 10.5
        sd.polygon([(0, y0 + 4), (S - 1, y0), (S - 1, y0 + 2.6),
                    (0, y0 + 6.6)], fill=(214, 212, 200, 110))
    img.alpha_composite(sheen)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, S - 1, S - 1], outline=CB_EDGE, width=1)
    return img

def _cardboard_wall_base(rng):
    img = Image.new("RGBA", (S, S), CB_DARK)
    d = ImageDraw.Draw(img)
    # boxes stacked like bricks, three courses
    courses = [(26, 31.5), (17, 26), (8, 17), (0.5, 8)]
    for (y0, y1) in courses:
        x = -rng.uniform(0, 4)
        while x < S - 3:
            w = rng.uniform(9, 14)
            _box(d, x, y0, min(x + w, S - 0.5), y1, rng)
            x += w + 0.6
    # rags and junk peeking from the gaps
    for _ in range(6):
        x = rng.uniform(1, 28); y = rng.uniform(2, 28)
        c = [(150, 84, 84, 255), (96, 110, 150, 255),
             (168, 158, 120, 255)][rng.randrange(3)]
        d.rectangle([x, y, x + rng.uniform(2, 3.6), y + rng.uniform(1.6, 3)],
                    fill=c)
    return img

def draw_cardboard_wall(seed):
    rng = random.Random(seed)
    return _cardboard_wall_base(rng)

def draw_cardboard_door_closed(seed):
    rng = random.Random(seed)
    img = _cardboard_wall_base(rng)
    d = ImageDraw.Draw(img)
    # the largest box, cut and folded into a door: one tall panel, centre
    d.rectangle([10, 3, 22, 31], fill=CB_LIGHT, outline=CB_EDGE)
    d.line([(10.8, 17), (21.2, 17)], fill=CB_DARK, width=1)  # fold line
    d.line([(16, 3.8), (16, 30)], fill=CB_DARK, width=1)     # cut seam
    d.rectangle([19.6, 15.4, 21, 16.8], fill=TAPE)           # tape hinge/tab
    return img

def draw_cardboard_door_open(seed):
    rng = random.Random(seed)
    img = _cardboard_wall_base(rng)
    d = ImageDraw.Draw(img)
    # dark opening where the door was
    d.rectangle([10, 3, 22, 31], fill=(34, 26, 20, 255))
    d.rectangle([11.5, 5, 20.5, 31], fill=(48, 38, 28, 255))
    # the door panel folded back against the left-hand boxes
    d.polygon([(10, 3), (4.5, 6), (4.5, 29), (10, 31)], fill=CB_LIGHT,
              outline=CB_EDGE)
    d.line([(4.9, 17), (9.8, 17)], fill=CB_DARK, width=1)
    return img

def draw_cardboard_fort(seed):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    # a big box tipped on its side: body of the box
    d.rectangle([2, 9, 30, 31], fill=CB_BASE, outline=CB_EDGE)
    d.line([(2, 15), (9, 15)], fill=CB_DARK, width=1)   # side flap seams
    d.line([(23, 15), (30, 15)], fill=CB_DARK, width=1)
    d.rectangle([3.4, 10.4, 6, 13], fill=TAPE)          # tape patch
    # open front: dark interior
    d.rectangle([10, 13, 23, 31], fill=(52, 40, 30, 255))
    d.rectangle([11.5, 15, 21.5, 31], fill=(66, 52, 38, 255))
    # blankets lining the floor of the fort
    d.rectangle([11, 26, 22, 31], fill=(96, 110, 150, 255))
    d.line([(11, 28.4), (22, 28.4)], fill=(150, 84, 84, 255), width=2)
    # pillows
    for (x, y) in [(12.5, 22.5), (18.5, 23.5)]:
        d.ellipse([x, y, x + 4.6, y + 3], fill=(234, 232, 226, 255),
                  outline=(188, 184, 174, 255))
    # plastic sheet draped over the top for weather-sealing
    sheen = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    sd = ImageDraw.Draw(sheen)
    sd.polygon([(2, 9), (30, 9), (30, 13.5), (2, 15.5)],
               fill=(216, 214, 202, 120))
    sd.line([(5, 13.8), (27, 11.4)], fill=(238, 236, 226, 130), width=1)
    img.alpha_composite(sheen)
    return img

# ================================================================ jobs
JOBS = []  # (id, image)

def _add(tid, img):
    JOBS.append((tid, img))

_seed = [6100]
def next_seed():
    _seed[0] += 7
    return _seed[0]

_add("f_cardboard_fort", draw_cardboard_fort(next_seed()))
_add("f_cardboard_wall", draw_cardboard_wall(next_seed()))
_add("f_cardboard_floor", draw_cardboard_floor(next_seed()))
_add("f_cardboard_roof", draw_cardboard_roof(next_seed()))
_add("f_cardboard_door_o", draw_cardboard_door_open(next_seed()))
_add("f_cardboard_door", draw_cardboard_door_closed(next_seed()))

for mat in ("sand", "gravel", "pebble", "clay", "glass", "rock",
            "rock_large"):
    for h in ("short", "mid", "tall"):
        _add(f"f_{mat}_mound_{h}", draw_mound(mat, h, next_seed()))

for kind in ("log", "plank", "brick", "adobe", "rebar", "pipe"):
    for h in ("short", "mid", "tall"):
        _add(f"f_{kind}_stack_{h}", STACK_DRAW[kind](h, next_seed()))
for h in ("short", "mid", "tall"):
    _add(f"f_stick_stack_{h}", draw_stick_stack(h, next_seed(), False))
for h in ("short", "mid", "tall"):
    _add(f"f_stick_long_stack_{h}",
         draw_stick_stack(h, next_seed(), True))

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": tid, "bg": ""}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
CELL = 48
COLS = 9
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (96, 148, 60, 255))
for i, (tid, _img) in enumerate(JOBS):
    p = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid,
                     tid + ".png")
    t = Image.open(p).convert("RGBA").resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_furniture_terrains.png"))
print("wrote preview_furniture_terrains.png", sheet.size)
