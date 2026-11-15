#!/usr/bin/env python3
"""
Draw the missing triffid, reverberations, and ice terrain sprites from
their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/, CDDA 6a3e59e0) --
the ids below are exactly the terrain entries of terrain-triffid.json,
terrain-reverberations.json, and terrain-ice.json that appear in
coverage-audit/missing.json (missing_shipped, category terrain) with no
pre-existing folder under pngs_normal_32x32/terrain/:

  terrain-triffid.json:
  t_bark_wall_chipped   "A solid wall of bark enclosing the trunk of a
                         massive tree.  Some of it has been broken away."
                         symbol #, color brown, WALL
  t_bark_wall_broken    "A formerly solid wall of bark, now almost
                         completely destroyed and waiting to collapse."
                         symbol &, color brown
  t_barkfloor           "A solid piece of bark covers the floor here."
                         symbol ., color brown, FLAT
  t_barkfloor_noroof    identical name/description to t_barkfloor (the
                         no-roof variant) -- drawn with the same design
  t_triffid_slope_down  "A downward-facing slope of root-covered dirt."
                         symbol >, color brown, GOES_DOWN
  t_triffid_slope_up    "An upward-facing slope of root-covered dirt."
                         symbol <, color brown, GOES_UP
  t_bramble_door_c      "A bramble of roots covering a large hole.  They
                         could easily be pulled apart."  symbol +, DOOR
  t_bramble_door_o      "A bramble of roots, pulled apart to reveal a
                         hole."  symbol '

  terrain-reverberations.json:
  t_inner_cabins_void   "sprawling plane of black... black as the depths
                         of the earth."  color black, FLAT
  t_inner_cabins_void_trans
                        "fluid void... a pool of dark water, slowly
                         undulating in an unnaturally slow fashion.  The
                         air around it is frigid cold, stale, and softens
                         ambient light."  symbol ~, color black
  t_inner_cabins_void_trans_retracting
                        the same fluid void, "slowly retracting toward
                         the center of the pool.  The grass growing at
                         its edges wobbles... constantly bending toward
                         the blackness."  color light_gray
  t_inner_cabins_exit_portal
                        "unstable portal... A swirling tear in reality.
                         Beyond it, you see a field of ever-expanding
                         buildings.  Something else is moving inside,
                         beyond your ability to resolve."  symbol O,
                         color magenta
  t_inner_cabins_nether_glass_floor
                        "black glass... An endless expanse of black
                         glass, illuminated by dim, otherworldly light."
                         light_emitted 5, warm amber light_color
  t_radiosphere_tree    "power tree... A power pole with a crown of
                         signal antennas on top.  You can hear
                         intermittent buzzing"  color blue,
                         light_emitted 50 in cyan (0, 200, 200)
  t_radiosphere_bush    "wire bush... A giant tangle of copper wiring
                         and steel pipes.  The moss covering the tangle
                         gives it a plant-like appearance."  symbol #

  terrain-ice.json:
  t_ice_sh_thin         "thin ice... A thin layer of ice covering
                         shallow water.  It may not support much weight."
                         symbol /, color light_cyan, THIN_ICE
  t_ice_sh_thick        "thick ice... A solid sheet of thick ice
                         covering shallow water."  color cyan, THICK_ICE
  t_ice_dp_thin         "thin ice... A thin layer of ice covering deep
                         water.  It may break under weight."  light_cyan
  t_ice_dp_thick        "thick ice... A thick sheet of ice covering
                         deep water... danger remains below."  cyan
  t_pseudo_phase        "pseudo phase... Placeholder terrain
                         representing the original tile for PHASE_BACK
                         behavior.  Should not be seen in-game."
                         symbol ., color light_gray, PSEUDO

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet.
Full-tile sprites (bark walls and floors, slopes, bramble doors with
their hole baked in, the void planes, black glass, all ice, the pseudo
placeholder) are fg with bg "" and are FULLY OPAQUE: translucent-looking
marks (ice sheen, glass glow, shading gradients) are painted on a
separate overlay and composited with Image.alpha_composite, never
drawn as semi-transparent pixels directly. The portal is a transparent
fg over bg "t_inner_cabins_void" (the black plane it stands on, the
t_string-over-void pattern); the two radiosphere features are
transparent fg with bg "". Flat fills, dark outlines, one highlight
pass -- UltiCa style. Deterministic: every random choice comes from a
per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json +
credits), plus preview_triffid_ice.png.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def full(color):
    return Image.new("RGBA", (S, S), color)

def with_overlay(img, draw_fn):
    """Paint translucent marks on an overlay and composite them over
    the (opaque) base, so the result stays fully opaque."""
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    draw_fn(d)
    img.alpha_composite(ov)
    return img

# ---------------------------------------------------------------- bark
BARK_DARK = (74, 50, 28, 255)
BARK = (116, 80, 46, 255)
BARK_MID = (140, 100, 58, 255)
BARK_HI = (170, 126, 78, 255)
BARK_SHADOW = (40, 26, 14, 255)
FRESH_WOOD = (198, 162, 108, 255)

def bark_plates(d, rng, x0=0, x1=S, y0=0, y1=S):
    """Vertical bark plates: wavy dark seams, grain flecks, one
    highlight line along each plate's left edge."""
    seams = []
    x = x0 + rng.uniform(3, 6)
    while x < x1 - 3:
        seams.append(x)
        x += rng.uniform(5, 8)
    for sx in seams:
        pts = [(sx + rng.uniform(-0.8, 0.8), y) for y in range(y0, y1 + 1, 4)]
        d.line(pts, fill=BARK_SHADOW, width=2)
        d.line([(px + 1.6, py) for px, py in pts], fill=BARK_HI, width=1)
    for _ in range(40):  # grain flecks
        x = rng.uniform(x0 + 1, x1 - 2); y = rng.uniform(y0 + 1, y1 - 2)
        ln = rng.uniform(1.5, 4)
        col = BARK_DARK if rng.random() < 0.55 else BARK_HI
        d.line([x, y, x + rng.uniform(-0.5, 0.5), y + ln], fill=col, width=1)
    for _ in range(3):  # knots
        x = rng.uniform(x0 + 4, x1 - 5); y = rng.uniform(y0 + 4, y1 - 5)
        d.ellipse([x - 2, y - 2.6, x + 2, y + 2.6], outline=BARK_DARK, width=1)
        d.point((x, y), fill=BARK_SHADOW)

def draw_bark_wall_chipped():
    rng = random.Random(3301)
    img = full(BARK)
    d = ImageDraw.Draw(img)
    bark_plates(d, rng)
    # the broken-away patch: jagged hole of pale fresh wood, lower right
    cx, cy = 22, 16
    pts = []
    for i in range(9):
        a = i / 9 * math.tau
        r = rng.uniform(5.5, 8.5)
        pts.append((cx + math.cos(a) * r, cy + math.sin(a) * r * 0.85))
    d.polygon(pts, fill=FRESH_WOOD)
    d.line(pts + [pts[0]], fill=BARK_SHADOW, width=1)
    for _ in range(7):  # splinter cracks radiating in the fresh wood
        a = rng.uniform(0, math.tau)
        x0, y0 = cx + math.cos(a) * 1.5, cy + math.sin(a) * 1.2
        x1, y1 = cx + math.cos(a) * rng.uniform(3, 6), cy + math.sin(a) * rng.uniform(2.5, 5)
        d.line([x0, y0, x1, y1], fill=(150, 112, 66, 255), width=1)
    for _ in range(4):  # loose splinters hanging at the patch's rim
        x = cx + rng.uniform(-7, 7); y = cy + rng.uniform(4, 8)
        d.polygon([(x, y), (x + 1.6, y + 0.6), (x + 0.4, y + 3.2)],
                  fill=BARK_MID)
    return img

def draw_bark_wall_broken():
    rng = random.Random(3302)
    img = full((30, 19, 10, 255))  # the hollow dark behind the wall
    d = ImageDraw.Draw(img)
    # remaining slabs: a top band, a left column, one diagonal shard --
    # each bark-textured, the rest is gone
    d.rectangle([0, 0, S - 1, 6], fill=BARK)
    d.rectangle([0, 0, 5, 20], fill=BARK)
    d.polygon([(18, 10), (30, 4), (31, 9), (21, 15)], fill=BARK)
    bark_plates(d, rng, 0, S, 0, 7)
    bark_plates(d, rng, 0, 6, 0, 20)
    for (x, y, w, h) in [(18, 10, 13, 6), (19, 11, 11, 4)]:
        for _ in range(6):
            gx = rng.uniform(x, x + w); gy = rng.uniform(y, y + h)
            d.line([gx, gy, gx, gy + rng.uniform(1.5, 3)], fill=BARK_DARK, width=1)
    # hanging splinters from the top band
    for _ in range(6):
        x = rng.uniform(2, 29)
        ln = rng.uniform(2.5, 5.5)
        d.polygon([(x, 6), (x + 1.8, 6), (x + 0.7, 6 + ln)], fill=BARK_MID)
    # rubble piled at the bottom
    for _ in range(9):
        x = rng.uniform(0, S - 5); y = rng.uniform(25, 29)
        w = rng.uniform(3.5, 6.5); h = rng.uniform(2, 3.6)
        col = BARK if rng.random() < 0.6 else BARK_MID
        d.polygon([(x, y + h), (x + w * 0.3, y), (x + w, y + h * 0.7),
                   (x + w * 0.8, y + h)], fill=col)
        d.line([x + 0.6, y + h - 0.8, x + w - 0.6, y + h - 1.2],
               fill=BARK_HI, width=1)
    return img

def draw_barkfloor():
    rng = random.Random(3303)
    img = full(BARK_MID)
    d = ImageDraw.Draw(img)
    # horizontal bark slabs with staggered joints
    rows = [(0, 8), (8, 16), (16, 24), (24, 32)]
    for ri, (y0, y1) in enumerate(rows):
        d.line([0, y0, S - 1, y0], fill=BARK_SHADOW, width=2)
        d.line([0, y0 + 2, S - 1, y0 + 2], fill=BARK_HI, width=1)
        jx = (ri % 2) * 9 + rng.uniform(6, 10)
        while jx < S - 2:
            d.line([jx, y0 + 1, jx + rng.uniform(-0.6, 0.6), y1 - 1],
                   fill=BARK_SHADOW, width=1)
            jx += rng.uniform(11, 15)
        for _ in range(9):  # wavy grain along the slab
            x = rng.uniform(0, S - 5); y = rng.uniform(y0 + 2, y1 - 2)
            pts = [(x + t, y + 0.7 * math.sin(t / 2.2)) for t in range(5)]
            col = BARK_DARK if rng.random() < 0.5 else BARK_HI
            d.line(pts, fill=col, width=1)
    for _ in range(3):  # knots
        x = rng.uniform(4, 27); y = rng.uniform(4, 27)
        d.ellipse([x - 2, y - 1.6, x + 2, y + 1.6], outline=BARK_DARK, width=1)
        d.point((x, y), fill=BARK_SHADOW)
    return img

# ---------------------------------------------------------------- slopes
ROOT = (88, 60, 34, 255)
ROOT_HI = (126, 92, 54, 255)

def root_snakes(d, rng, n=6):
    for _ in range(n):
        x = rng.uniform(-4, S); y = rng.uniform(0, S - 6)
        pts = [(x, y)]
        for _ in range(4):
            x += rng.uniform(3, 7); y += rng.uniform(-2.5, 3.5)
            pts.append((x, y))
        d.line(pts, fill=ROOT, width=2)
        d.line([(px, py - 0.8) for px, py in pts], fill=ROOT_HI, width=1)

def draw_slope(down):
    rng = random.Random(3304 if down else 3305)
    img = full((106, 80, 52, 255))  # root-covered dirt
    d = ImageDraw.Draw(img)
    for _ in range(46):  # dirt mottling and pebbles
        x = rng.uniform(0, S - 2); y = rng.uniform(0, S - 2)
        col = (84, 60, 38, 255) if rng.random() < 0.5 else (130, 102, 66, 255)
        d.ellipse([x, y, x + rng.uniform(1, 2.6), y + rng.uniform(1, 2)], fill=col)
    root_snakes(d, rng)
    # direction chevron formed by two thick roots
    if down:  # "v": descends away from the viewer
        d.line([8, 11, 16, 20], fill=ROOT, width=3)
        d.line([24, 11, 16, 20], fill=ROOT, width=3)
        d.line([8, 10, 16, 19], fill=ROOT_HI, width=1)
        d.line([24, 10, 16, 19], fill=ROOT_HI, width=1)
    else:     # "^": rises away from the viewer
        d.line([8, 21, 16, 12], fill=ROOT, width=3)
        d.line([24, 21, 16, 12], fill=ROOT, width=3)
        d.line([8, 20, 16, 11], fill=ROOT_HI, width=1)
        d.line([24, 20, 16, 11], fill=ROOT_HI, width=1)
    # shading gradient via overlay: down-slope sinks into shadow at the
    # bottom; up-slope climbs into the light at the top
    def shade(dd):
        for i in range(14):
            y = S - 1 - i if down else i
            a = int(88 * (1 - i / 14))
            col = (10, 6, 2, a) if down else (255, 240, 200, a)
            dd.line([0, y, S - 1, y], fill=col)
    with_overlay(img, shade)
    return img

# ---------------------------------------------------------------- bramble doors
def bramble_strands(d, rng, region, n, thick=2):
    x0, y0, x1, y1 = region
    for _ in range(n):
        # crossing arcs woven in both diagonal directions
        if rng.random() < 0.5:
            pts = [(x0 + (x1 - x0) * t,
                    rng.uniform(y0, y1) + (rng.uniform(-4, 4)) * math.sin(t * math.pi))
                   for t in [i / 8 for i in range(9)]]
        else:
            pts = [(rng.uniform(x0, x1) + rng.uniform(-4, 4) * math.sin(t * math.pi),
                    y0 + (y1 - y0) * t)
                   for t in [i / 8 for i in range(9)]]
        col = ROOT if rng.random() < 0.7 else BARK_DARK
        d.line(pts, fill=col, width=thick)
        d.line([(px, py - 0.7) for px, py in pts], fill=ROOT_HI, width=1)

def draw_bramble_door(closed):
    rng = random.Random(3306 if closed else 3307)
    img = full((18, 11, 6, 255))  # the large hole behind the roots
    d = ImageDraw.Draw(img)
    # hole shading: darkest at the center
    for r, col in [(13, (26, 17, 9, 255)), (9, (20, 12, 6, 255)), (5, (12, 7, 3, 255))]:
        d.ellipse([16 - r, 16 - r * 0.85, 16 + r, 16 + r * 0.85], fill=col)
    if closed:
        # roots woven densely across the whole hole
        bramble_strands(d, rng, (1, 1, 30, 30), 15, thick=3)
        bramble_strands(d, rng, (2, 2, 29, 29), 10, thick=2)
    else:
        # pulled apart: root masses cling to the sides and top corners,
        # a few strands hang from the top; the hole gapes open
        bramble_strands(d, rng, (0, 0, 7, 31), 9, thick=3)
        bramble_strands(d, rng, (24, 0, 31, 31), 9, thick=3)
        bramble_strands(d, rng, (0, 0, 31, 5), 6, thick=2)
        for _ in range(4):  # hanging strands at the top of the opening
            x = rng.uniform(11, 21)
            ln = rng.uniform(3, 6)
            d.line([x, 3, x + rng.uniform(-1.5, 1.5), 3 + ln], fill=ROOT, width=2)
    return img

# ---------------------------------------------------------------- void planes
def draw_void_plane():
    rng = random.Random(3308)
    img = full((7, 7, 9, 255))  # black as the depths of the earth
    d = ImageDraw.Draw(img)
    for _ in range(12):  # barely-there depth mottling
        x = rng.uniform(-4, S - 4); y = rng.uniform(-4, S - 4)
        w = rng.uniform(5, 11); h = rng.uniform(3, 7)
        col = (13, 13, 17, 255) if rng.random() < 0.6 else (3, 3, 5, 255)
        d.ellipse([x, y, x + w, y + h], fill=col)
    return img

def draw_fluid_void(retracting):
    rng = random.Random(3310 if retracting else 3309)
    img = full((5, 7, 11, 255))  # a pool of dark water
    d = ImageDraw.Draw(img)
    if retracting:
        # grassy rim: desaturated gray-green fringe the void is pulling
        # away from
        d.rectangle([0, 0, S - 1, S - 1], outline=(86, 96, 78, 255), width=4)
        for _ in range(60):
            x = rng.uniform(0, S - 1); y = rng.choice([rng.uniform(0, 3),
                                                       rng.uniform(28, 31)])
            if rng.random() < 0.4:
                x = rng.choice([rng.uniform(0, 3), rng.uniform(28, 31)])
                y = rng.uniform(0, S - 1)
            d.point((x, y), fill=(104, 116, 94, 255) if rng.random() < 0.5
                    else (70, 80, 64, 255))
    # unnaturally slow undulation: broad concentric bands, tightening
    # toward the center on the retracting patch
    cx, cy = 16, 16
    radii = [13.5, 10.5, 7.5, 4.5] if retracting else [14, 10, 6]
    for i, r in enumerate(radii):
        col = (15, 21, 31, 255) if i % 2 == 0 else (9, 12, 20, 255)
        d.ellipse([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8], outline=col, width=2)
    def sheen(dd):
        # frigid sheen: softens ambient light -- faint cold arcs
        for _ in range(5):
            x = rng.uniform(3, 22); y = rng.uniform(4, 26)
            r = rng.uniform(3, 6)
            dd.arc([x, y, x + r * 2, y + r], start=rng.uniform(180, 260),
                   end=rng.uniform(280, 350), fill=(110, 140, 170, 56), width=1)
    with_overlay(img, sheen)
    if retracting:
        d = ImageDraw.Draw(img)
        # grass blades at the edges, all bending toward the blackness
        for _ in range(14):
            side = rng.randrange(4)
            if side == 0: x, y, dx, dy = rng.uniform(2, 29), 2.5, 0, 1
            elif side == 1: x, y, dx, dy = rng.uniform(2, 29), 29.5, 0, -1
            elif side == 2: x, y, dx, dy = 2.5, rng.uniform(2, 29), 1, 0
            else: x, y, dx, dy = 29.5, rng.uniform(2, 29), -1, 0
            ln = rng.uniform(2.5, 4.5)
            d.line([x, y, x + dx * ln, y + dy * ln], fill=(112, 126, 98, 255), width=1)
    return img

def draw_exit_portal():
    rng = random.Random(3311)
    img = new_img()
    # interior: a field of ever-expanding buildings, and something
    # moving inside beyond resolution -- built on its own layer, then
    # masked to the tear's ellipse
    inner = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    di = ImageDraw.Draw(inner)
    di.rectangle([0, 0, S - 1, S - 1], fill=(24, 6, 34, 255))
    for _ in range(9):  # building silhouettes with lit windows
        x = rng.uniform(6, 24); w = rng.uniform(2.5, 4.5)
        h = rng.uniform(5, 13); y1 = 27
        di.rectangle([x, y1 - h, x + w, y1], fill=(104, 112, 148, 255))
        for _ in range(3):
            wx = rng.uniform(x + 0.5, x + w - 1); wy = rng.uniform(y1 - h + 1, y1 - 2)
            di.point((wx, wy), fill=(255, 220, 150, 255))
    # the unresolved moving thing: a smeared dark shape
    for i in range(5):
        di.line([11 + i * 0.8, 12 + rng.uniform(-0.5, 0.5),
                 19 + i * 0.8, 12 + rng.uniform(-0.5, 0.5)],
                fill=(8, 2, 12, 255), width=2)
    mask = Image.new("L", (S, S), 0)
    dm = ImageDraw.Draw(mask)
    dm.ellipse([7, 3, 25, 29], fill=255)
    img.paste(inner, (0, 0), mask)
    d = ImageDraw.Draw(img)
    # the swirling tear: spiral arcs of magenta around the rim
    cols = [(255, 0, 200, 255), (224, 44, 224, 255), (168, 64, 255, 255),
            (255, 110, 230, 255)]
    for i in range(14):
        r = 9.5 + (i % 3) * 1.6
        start = rng.uniform(0, 360)
        d.arc([16 - r, 16 - r * 1.35, 16 + r, 16 + r * 1.35],
              start=start, end=start + rng.uniform(50, 130),
              fill=cols[i % len(cols)], width=2)
    d.arc([7, 3, 25, 29], start=300, end=30, fill=(255, 200, 245, 255), width=1)
    def glow(dd):
        for _ in range(8):
            x = rng.uniform(4, 24); y = rng.uniform(1, 27)
            dd.ellipse([x, y, x + 3, y + 3], fill=(255, 0, 200, 40))
    with_overlay(img, glow)
    return img

def draw_nether_glass_floor():
    rng = random.Random(3312)
    img = full((9, 9, 11, 255))  # black glass
    d = ImageDraw.Draw(img)
    # faint panel seams in the endless expanse
    d.line([0, 10, S - 1, 10], fill=(26, 26, 32, 255), width=1)
    d.line([0, 22, S - 1, 22], fill=(26, 26, 32, 255), width=1)
    d.line([11, 0, 11, S - 1], fill=(26, 26, 32, 255), width=1)
    d.line([23, 0, 23, S - 1], fill=(26, 26, 32, 255), width=1)
    def glow(dd):
        # dim otherworldly light: warm amber reflections (its emitted
        # light_color) and one cool sheen sweeping the glass
        for _ in range(4):
            x = rng.uniform(2, 24); y = rng.uniform(2, 24)
            w = rng.uniform(5, 10); h = rng.uniform(2.5, 5)
            dd.ellipse([x, y, x + w, y + h], fill=(150, 75, 0, 44))
            dd.ellipse([x + w * 0.25, y + h * 0.25, x + w * 0.75, y + h * 0.75],
                       fill=(170, 95, 20, 40))
        dd.polygon([(2, 26), (14, 8), (19, 8), (8, 26)], fill=(140, 160, 200, 30))
        for _ in range(7):  # warm flecks caught in the glass
            x = rng.uniform(1, 30); y = rng.uniform(1, 30)
            dd.point((x, y), fill=(205, 120, 40, 150))
    with_overlay(img, glow)
    return img

# ---------------------------------------------------------------- radiosphere
def draw_radiosphere_tree():
    rng = random.Random(3313)
    img = new_img()
    d = ImageDraw.Draw(img)
    steel = (116, 122, 130, 255)
    steel_dark = (70, 76, 84, 255)
    steel_hi = (172, 180, 190, 255)
    # footing and pole
    d.polygon([(11, 31), (21, 31), (18, 26), (14, 26)], fill=(98, 102, 108, 255),
              outline=steel_dark)
    d.rectangle([14, 6, 17, 27], fill=steel)
    d.line([14, 6, 14, 27], fill=steel_hi, width=1)
    d.line([17, 6, 17, 27], fill=steel_dark, width=1)
    # crossarm and a sagging cable
    d.rectangle([6, 8, 26, 10], fill=steel)
    d.line([6, 8, 26, 8], fill=steel_hi, width=1)
    pts = [(6 + t * 2, 10 + 2.6 * math.sin(t / 9 * math.pi)) for t in range(10)]
    d.line(pts, fill=steel_dark, width=1)
    # junction box on the pole
    d.rectangle([12, 15, 20, 20], fill=(88, 94, 102, 255), outline=steel_dark)
    d.point((16, 17), fill=(0, 230, 230, 255))
    # the crown of signal antennas: rods of varying heights, a dish
    for (x, top) in [(8, 1), (12, 0), (16, 2), (20, 0), (24, 3)]:
        d.line([x, 8, x, top], fill=steel_dark, width=1)
        d.point((x, top), fill=(0, 230, 230, 255))
        d.point((x, top + 2), fill=(0, 180, 180, 255))
    d.ellipse([19, 4, 26, 8], fill=steel, outline=steel_dark)
    d.line([21, 8, 22, 6], fill=steel_dark, width=1)
    def glow(dd):
        # its cyan emission (light 50, 0/200/200): soft halos at the tips
        for (x, y) in [(8, 1), (12, 0), (16, 2), (20, 0), (24, 3), (16, 17)]:
            dd.ellipse([x - 2, y - 2, x + 2, y + 2], fill=(0, 200, 200, 60))
    with_overlay(img, glow)
    return img

def draw_radiosphere_bush():
    rng = random.Random(3314)
    img = new_img()
    d = ImageDraw.Draw(img)
    copper = (186, 110, 62, 255)
    copper_dark = (146, 80, 42, 255)
    steel = (128, 134, 142, 255)
    steel_dark = (94, 100, 108, 255)
    wire_cols = [copper, copper_dark, steel, steel_dark]
    # a giant tangle: overlapping loops and arcs of wire and pipe
    for _ in range(26):
        cx = rng.uniform(6, 26); cy = rng.uniform(7, 26)
        r = rng.uniform(3, 9)
        col = wire_cols[rng.randrange(len(wire_cols))]
        start = rng.uniform(0, 360)
        d.arc([cx - r, cy - r * 0.8, cx + r, cy + r * 0.8], start=start,
              end=start + rng.uniform(120, 320), fill=col,
              width=2 if col in (steel, steel_dark) else 1)
    # two straight pipe runs with joint collars
    d.line([3, 24, 20, 12], fill=steel, width=3)
    d.line([3, 23, 20, 11], fill=(168, 174, 182, 255), width=1)
    d.ellipse([10, 16, 14, 20], outline=steel_dark, width=1)
    d.line([14, 28, 29, 18], fill=copper_dark, width=2)
    # moss creeping over the tangle, giving it a plant-like look
    moss = [(84, 124, 60, 255), (104, 146, 72, 255), (68, 106, 48, 255)]
    for (cx, cy, r) in [(10, 10, 5.5), (23, 22, 6), (17, 27, 4.5), (26, 9, 3.6)]:
        for _ in range(int(r * r)):
            a = rng.uniform(0, math.tau); rr = r * math.sqrt(rng.random())
            x, y = cx + math.cos(a) * rr, cy + math.sin(a) * rr * 0.8
            d.ellipse([x - 1.4, y - 1, x + 1.4, y + 1], fill=moss[rng.randrange(3)])
    return img

# ---------------------------------------------------------------- ice
def draw_ice(shallow, thick, seed):
    rng = random.Random(seed)
    if shallow:  # pale sandy shallows under the ice
        img = full((126, 178, 182, 255))
        d = ImageDraw.Draw(img)
        for _ in range(16):
            x = rng.uniform(-3, S - 3); y = rng.uniform(-3, S - 3)
            w = rng.uniform(4, 9); h = rng.uniform(3, 6)
            col = (168, 198, 180, 255) if rng.random() < 0.5 else (102, 154, 162, 255)
            d.ellipse([x, y, x + w, y + h], fill=col)
        for _ in range(5):  # soft weed shadows in the shallows
            x = rng.uniform(2, 26); y = rng.uniform(2, 26)
            d.arc([x, y, x + 6, y + 5], start=200, end=330,
                  fill=(84, 134, 128, 255), width=1)
    else:        # deep water: dark blue depths
        img = full((14, 44, 92, 255))
        d = ImageDraw.Draw(img)
        for _ in range(14):
            x = rng.uniform(-3, S - 3); y = rng.uniform(-3, S - 3)
            w = rng.uniform(4, 10); h = rng.uniform(3, 6)
            col = (22, 58, 110, 255) if rng.random() < 0.5 else (9, 32, 70, 255)
            d.ellipse([x, y, x + w, y + h], fill=col)
    # the ice film itself, composited so the sheet stays opaque: thin
    # ice is a translucent veil, thick ice nearly solid
    def film(dd):
        dd.rectangle([0, 0, S - 1, S - 1],
                     fill=(202, 236, 242, 235 if thick else 118))
        if thick:  # frost patches blooming on a solid sheet
            for _ in range(9):
                x = rng.uniform(-2, S - 4); y = rng.uniform(-2, S - 4)
                w = rng.uniform(4, 9); h = rng.uniform(3, 6)
                dd.ellipse([x, y, x + w, y + h], fill=(244, 250, 252, 120))
    with_overlay(img, film)
    d = ImageDraw.Draw(img)
    # cracks: white fracture lines with a shadowed twin; thin ice is
    # webbed with them, thick ice has only a few deep ones
    crack_col = (246, 252, 254, 255)
    shadow_col = (92, 132, 152, 255) if shallow else (6, 22, 48, 255)
    n_cracks = 5 if thick else 11
    for _ in range(n_cracks):
        x = rng.uniform(1, S - 6); y = rng.uniform(1, S - 6)
        pts = [(x, y)]
        for _ in range(rng.randint(2, 4)):
            x += rng.uniform(-5, 6); y += rng.uniform(-4, 5)
            pts.append((x, y))
        d.line([(px, py + 1) for px, py in pts], fill=shadow_col, width=1)
        d.line(pts, fill=crack_col, width=1)
        if not thick and rng.random() < 0.6:  # a branch fracture
            bx, by = pts[len(pts) // 2]
            d.line([bx, by, bx + rng.uniform(-4, 4), by + rng.uniform(-4, 4)],
                   fill=crack_col, width=1)
    if not thick:  # open holes where the film has given way
        for _ in range(2):
            x = rng.uniform(4, 24); y = rng.uniform(4, 24)
            w = rng.uniform(3.5, 6); h = rng.uniform(2.5, 4)
            col = (140, 192, 196, 255) if shallow else (10, 36, 80, 255)
            d.ellipse([x, y, x + w, y + h], fill=col)
            d.ellipse([x, y, x + w, y + h], outline=crack_col, width=1)
    for _ in range(12):  # sparkle
        x = rng.uniform(1, 30); y = rng.uniform(1, 30)
        d.point((x, y), fill=(255, 255, 255, 255))
        if rng.random() < 0.4:
            d.line([x - 1, y, x + 1, y], fill=(255, 255, 255, 255))
            d.line([x, y - 1, x, y + 1], fill=(255, 255, 255, 255))
    return img

def draw_pseudo_phase():
    rng = random.Random(3319)
    img = full((168, 168, 168, 255))  # light_gray placeholder
    d = ImageDraw.Draw(img)
    # a faint two-tone checker: unmistakably a stand-in tile
    for cy in range(4):
        for cx in range(4):
            if (cx + cy) % 2 == 0:
                d.rectangle([cx * 8, cy * 8, cx * 8 + 7, cy * 8 + 7],
                            fill=(178, 178, 178, 255))
    d.rectangle([0, 0, S - 1, S - 1], outline=(138, 138, 138, 255), width=1)
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg)
JOBS = [
    ("t_bark_wall_chipped", draw_bark_wall_chipped(), "t_bark_wall_chipped", ""),
    ("t_bark_wall_broken", draw_bark_wall_broken(), "t_bark_wall_broken", ""),
    ("t_barkfloor", draw_barkfloor(), "t_barkfloor", ""),
    ("t_barkfloor_noroof", draw_barkfloor(), "t_barkfloor_noroof", ""),
    ("t_triffid_slope_down", draw_slope(True), "t_triffid_slope_down", ""),
    ("t_triffid_slope_up", draw_slope(False), "t_triffid_slope_up", ""),
    ("t_bramble_door_c", draw_bramble_door(True), "t_bramble_door_c", ""),
    ("t_bramble_door_o", draw_bramble_door(False), "t_bramble_door_o", ""),
    ("t_inner_cabins_void", draw_void_plane(), "t_inner_cabins_void", ""),
    ("t_inner_cabins_void_trans", draw_fluid_void(False),
     "t_inner_cabins_void_trans", ""),
    ("t_inner_cabins_void_trans_retracting", draw_fluid_void(True),
     "t_inner_cabins_void_trans_retracting", ""),
    ("t_inner_cabins_exit_portal", draw_exit_portal(),
     "t_inner_cabins_exit_portal", "t_inner_cabins_void"),
    ("t_inner_cabins_nether_glass_floor", draw_nether_glass_floor(),
     "t_inner_cabins_nether_glass_floor", ""),
    ("t_radiosphere_tree", draw_radiosphere_tree(), "t_radiosphere_tree", ""),
    ("t_radiosphere_bush", draw_radiosphere_bush(), "t_radiosphere_bush", ""),
    ("t_ice_sh_thin", draw_ice(True, False, 3315), "t_ice_sh_thin", ""),
    ("t_ice_sh_thick", draw_ice(True, True, 3316), "t_ice_sh_thick", ""),
    ("t_ice_dp_thin", draw_ice(False, False, 3317), "t_ice_dp_thin", ""),
    ("t_ice_dp_thick", draw_ice(False, True, 3318), "t_ice_dp_thick", ""),
    ("t_pseudo_phase", draw_pseudo_phase(), "t_pseudo_phase", ""),
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

void = tile_img("t_inner_cabins_void")

def composited(tid, bg):
    """The tile as the game shows it: fg over its bg (or the sheet's
    dark backdrop for the free-standing radiosphere features)."""
    fg = tile_img(tid)
    if bg:
        out = void.copy()
        out.alpha_composite(fg)
        return out
    return fg

CELL = 64
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img, _fg, bg) in enumerate(JOBS):
    t = composited(tid, bg).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_triffid_ice.png"))
print("wrote preview_triffid_ice.png", sheet.size)
