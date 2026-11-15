#!/usr/bin/env python3
"""
Draw the scatter remainder: the last 26 missing terrain sprites, from
their game descriptions.  Sources (fork-src/cdda/data/json/, CDDA
6a3e59e0); each id below is missing from UltiCa in every form (no
tile-entry JSON, no orphan PNG, no usable looks_like):

  terrain-flora.json:
  t_beehive_natural     "A hive of bees located in a cavity inside a
                         tree, swarming in and out at a feverish pace.
                         There's probably honey inside."  color yellow
  t_grass_grazed        "extremely short green stubble protruding from
                         the dirt" (copy-from t_grass_dead)
  t_highland_holy_grass "Soft mats of fine golden grass shimmer and
                         sway like sea waves, casting an ethereal
                         glow"  name "golden grass", color yellow
  t_fungus_colony       "A large pile of organic matter which has
                         molded over with fungus.  The caps of many
                         mundane mushrooms spring from the top"
                         color dark_gray, coverage 80
  terrain-floors-outdoors.json:
  t_gelatinous_groundcover "otherworldly substrate... like walking on
                         hardened gelatin"  color pink, FLAT
  LIXA_furniture_and_terrain.json:
  t_folded_wall         "seems to bend and curve in on itself...
                         a repeating pattern of solid concrete
                         hexagons"  color yellow_cyan, WALL
  t_unfolded_wall       "This wall tessellates as you view it."
                         color cyan_red, WALL
  t_infinite_shaft_no_roof / t_infinite_shaft_rooved
                        "open air in... a shaft extending into the
                         depths of the earth... its bottom, if there
                         is one, seems to be far past where light
                         penetrates"  color i_cyan, GOES_DOWN
  terrain-nether.json:
  t_nether_basalt       "Hexagonal columns of basalt spring from the
                         dark floor, each an exact copy of the next"
                         color dark_gray, WALL
  t_nether_glass_floor / t_nether_glass_floor_clairvoyance /
  t_nether_glass_wall   "An endless expanse of black glass.  It
                         perfectly reflects the darkness around it"
                         color black (the clairvoyance variant emits
                         emit_clairvoyant)
  terrain-scarlet-sea.json:
  t_scrap_scaffolding_scarlet "A mass of jagged metal, climbing into
                         the sky.  It looks sharp, but also sturdy
                         enough to climb"  color dark_gray, GOES_UP
  t_water_sh_scarlet    "scarlet water... difficult to tell if the
                         water is tinted, or if it's reflecting the
                         dim, scarlet sky"  shallow
  t_water_sh_scarlet_wave  the same, with waves (emit_scarlet_sea_wave)
  terrain-liquids.json:
  t_saltwater_cube      "You are submerged below the surface of the
                         water"  color blue, DEEP_WATER / WATER_CUBE
  t_water_cube_hot      "submerged below the surface of warm water"
                         color blue, heat_radiation 1
  terrain-shore.json:
  t_shore_boulder       "A medium rocky outcropping, smoothed by the
                         waves and weather"  color dark_gray
  t_shore_boulder_oyster "An oyster-encrusted boulder"  color white
  special_use/isherwood_spawnpoints.json:
  t_dirt_isherwood_player_spawnpoint "It's dirt.  Looks like some
                         fine soil for tillage" (copy-from t_dirt)
  terrain-fungal.json:
  t_fungus_wall_transformed "The fungus here has grown thickly and
                         tightly enough to form a solid wall... very
                         stiff"  color dark_gray, WALL
  terrain_nether_monster.json:
  t_nm_floor_bone       "An impossibly hard, impregnable bone
                         structure covers the internals of the beast"
                         color magenta_white, FLAT floor
  terrain-fences-gates.json:
  t_portcullis          "An immense gateway cobbled together from
                         various bits of steel and metal scraps"
                         color cyan, TRANSPARENT / PERMEABLE
  mapgen/reverberations/radiosphere.json:
  t_radiosphere_glowing_floor "vanishing floor... The sheer fact you
                         can stand on this dumbfounds you"  color
                         cyan, light_emitted 50
  terrain-windows.json:
  t_window_boarded_shutters_closed "A glass window with a heavy
                         makeshift shutter bolted into the window
                         frame, with a wooden bar to close it"

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet.  Full-tile tiles are fg with bg ""; feature tiles are
transparent fg (the scaffolding rises out of the scarlet shallows:
bg "t_water_sh_scarlet", in this batch).  Every full-tile sprite is
fully opaque: translucent-looking sheens are drawn on a separate
overlay and alpha-composited onto the opaque base, never drawn as
semi-transparent pixels.  Deterministic: every random choice comes
from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json +
credits) and preview_scatter_terrain.png.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def full(color):
    return Image.new("RGBA", (S, S), color)

def transparent():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def sheen(img, draw_fn):
    """Draw translucent marks on an overlay, composite onto img."""
    ov = transparent()
    d = ImageDraw.Draw(ov, "RGBA")
    draw_fn(d)
    img.alpha_composite(ov)
    return img

# ---------------------------------------------------------------- flora
def draw_beehive():
    rng = random.Random(2601)
    img = full((94, 66, 40, 255))  # bark of the host tree fills the tile
    d = ImageDraw.Draw(img)
    # vertical bark furrows
    for _ in range(30):
        x = rng.uniform(0, S - 1)
        col = (74, 50, 30, 255) if rng.random() < 0.55 else (116, 86, 52, 255)
        d.line([x, 0, x + rng.uniform(-1.5, 1.5), S - 1], fill=col, width=1)
    for _ in range(8):  # knots and cracks
        x = rng.uniform(2, S - 4); y = rng.uniform(2, S - 4)
        d.ellipse([x, y, x + 2.4, y + 3.4], outline=(64, 42, 24, 255))
    # the cavity: a dark hollow in the trunk
    d.ellipse([9, 7, 25, 27], fill=(26, 16, 10, 255))
    d.ellipse([10.5, 8.5, 23.5, 25.5], fill=(38, 24, 14, 255))
    # the hive inside: stacked golden comb masses hanging from the top
    for (x, y, w, h, col) in [
            (13, 9, 9, 7, (216, 168, 58, 255)),
            (12, 13, 11, 8, (228, 182, 70, 255)),
            (13.5, 18, 8, 6, (204, 154, 48, 255))]:
        d.ellipse([x, y, x + w, y + h], fill=col)
    for _ in range(16):  # comb cells
        x = rng.uniform(13, 21); y = rng.uniform(10, 22)
        d.point((x, y), fill=(178, 128, 36, 255))
    d.point((16, 12), fill=(246, 214, 120, 255))  # honey glint
    # bees swarming in and out at a feverish pace
    for _ in range(15):
        x = rng.uniform(3, S - 4); y = rng.uniform(3, S - 4)
        if 10 < x < 24 and 8 < y < 26:
            continue
        d.ellipse([x, y, x + 1.8, y + 1.2], fill=(30, 24, 12, 255))
        d.point((x + 0.4, y - 0.6), fill=(230, 230, 235, 255))  # wing
    return img

def draw_grass_grazed():
    rng = random.Random(2602)
    img = full((110, 88, 56, 255))  # dirt showing through
    d = ImageDraw.Draw(img)
    for _ in range(36):
        x = rng.uniform(0, S - 3); y = rng.uniform(0, S - 3)
        col = (96, 74, 46, 255) if rng.random() < 0.5 else (126, 102, 66, 255)
        d.ellipse([x, y, x + rng.uniform(2, 3.4), y + rng.uniform(1.4, 2.4)],
                  fill=col)
    # extremely short green stubble: dense 1-2 px ticks, nothing taller
    for _ in range(210):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        col = (88, 132, 54, 255) if rng.random() < 0.6 else (104, 150, 64, 255)
        d.line([x, y, x, y - rng.uniform(1, 2)], fill=col, width=1)
    for _ in range(6):  # bare grazed patches
        x = rng.uniform(2, S - 6); y = rng.uniform(2, S - 6)
        d.ellipse([x, y, x + rng.uniform(3, 5), y + rng.uniform(2, 3.4)],
                  fill=(118, 94, 60, 255))
    return img

def draw_holy_grass():
    rng = random.Random(2603)
    img = full((198, 166, 66, 255))  # golden mat
    d = ImageDraw.Draw(img)
    # sway like sea waves: horizontal sine bands, alternating golds
    for row in range(S):
        amp = 1.4
        pts = [(x, row + amp * math.sin(x / 6.0 + row * 0.8)) for x in range(S)]
        col = (214, 184, 84, 255) if (row // 3) % 2 == 0 else (182, 150, 54, 255)
        d.line(pts, fill=col, width=1)
    # fine grass strokes riding the waves
    for _ in range(90):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.line([x, y, x + rng.uniform(-1, 1), y - rng.uniform(2, 3.4)],
               fill=(232, 204, 110, 255), width=1)
    # ethereal glow: soft bright pools, composited
    def glow(dd):
        for _ in range(7):
            x = rng.uniform(2, S - 8); y = rng.uniform(2, S - 8)
            dd.ellipse([x, y, x + rng.uniform(5, 8), y + rng.uniform(3, 5)],
                       fill=(255, 240, 170, 46))
    sheen(img, glow)
    d = ImageDraw.Draw(img)
    for _ in range(12):  # sparkles in the glow
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        d.point((x, y), fill=(252, 244, 196, 255))
    return img

def draw_gelatinous():
    rng = random.Random(2604)
    img = full((216, 138, 168, 255))  # pink hardened gelatin
    d = ImageDraw.Draw(img)
    # blobby cell boundaries under the surface
    for _ in range(11):
        x = rng.uniform(-3, S); y = rng.uniform(-3, S)
        r = rng.uniform(4, 8)
        d.ellipse([x, y, x + r, y + r * 0.75], outline=(198, 112, 148, 255))
    def blobs(dd):
        for _ in range(9):  # deeper pink masses
            x = rng.uniform(0, S - 7); y = rng.uniform(0, S - 6)
            dd.ellipse([x, y, x + rng.uniform(5, 8), y + rng.uniform(4, 6)],
                       fill=(198, 104, 142, 60))
        for _ in range(8):  # pale suspended lumps
            x = rng.uniform(0, S - 5); y = rng.uniform(0, S - 4)
            dd.ellipse([x, y, x + rng.uniform(3, 5), y + rng.uniform(2, 4)],
                       fill=(240, 186, 204, 70))
    sheen(img, blobs)
    d = ImageDraw.Draw(img)
    for _ in range(7):  # glossy highlights on the gel surface
        x = rng.uniform(2, S - 5); y = rng.uniform(2, S - 4)
        d.ellipse([x, y, x + 2.2, y + 1.2], fill=(250, 224, 234, 255))
    return img

def draw_fungus_colony():
    rng = random.Random(2605)
    img = transparent()
    d = ImageDraw.Draw(img)
    # ground shadow of the pile
    d.ellipse([3, 24, 29, 30], fill=(20, 18, 12, 110))
    # the molded-over mound of organic matter
    for (x, y, w, h, col) in [
            (4, 16, 24, 13, (84, 80, 66, 255)),
            (6, 12, 19, 12, (94, 90, 72, 255)),
            (9, 9, 13, 10, (104, 100, 80, 255))]:
        d.ellipse([x, y, x + w, y + h], fill=col)
    # mold patches: grey-green fungal felt
    for _ in range(42):
        x = rng.uniform(6, S - 8); y = rng.uniform(10, S - 8)
        col = (116, 122, 98, 255) if rng.random() < 0.5 else (100, 106, 84, 255)
        d.ellipse([x, y, x + rng.uniform(1.6, 3), y + rng.uniform(1.2, 2.2)],
                  fill=col)
    # mundane mushroom caps springing from the top
    for _ in range(9):
        x = rng.uniform(7, S - 8); y = rng.uniform(9, 20)
        sw = rng.uniform(1.6, 2.6)
        d.line([x, y + 3, x, y], fill=(216, 204, 180, 255), width=1)
        cap = (156, 114, 76, 255) if rng.random() < 0.5 else (172, 130, 88, 255)
        d.pieslice([x - sw, y - 2.2, x + sw, y + 1.6], start=180, end=360,
                   fill=cap)
        d.point((x - 0.5, y - 1.6), fill=(196, 158, 116, 255))
    return img

# ---------------------------------------------------------------- LIXA
def _hex_points(cx, cy, r):
    return [(cx + r * math.cos(math.radians(60 * i - 30)),
             cy + r * math.sin(math.radians(60 * i - 30))) for i in range(6)]

def draw_folded_wall():
    rng = random.Random(2606)
    img = full((104, 106, 100, 255))  # grout
    d = ImageDraw.Draw(img)
    # repeating pattern of solid concrete hexagons
    r = 4.6
    dx, dy = r * math.sqrt(3), r * 1.5
    row = 0; cy = 0.0
    while cy < S + r:
        cx = (row % 2) * dx / 2
        while cx < S + r:
            tint = rng.random()
            if tint < 0.22:
                col = (142, 162, 158, 255)   # cyan sheen (yellow_cyan)
            elif tint < 0.44:
                col = (170, 166, 134, 255)   # pale yellow sheen
            else:
                col = (152, 154, 148, 255)   # concrete
            d.polygon(_hex_points(cx, cy, r - 0.7), fill=col)
            cx += dx
        cy += dy; row += 1
    # the wall bends in on itself: a curved shadow/light fold, composited
    def fold(dd):
        dd.polygon([(0, 13), (S, 9), (S, 15), (0, 20)], fill=(30, 32, 30, 74))
        dd.polygon([(0, 11), (S, 7), (S, 9), (0, 13)], fill=(235, 240, 220, 52))
    sheen(img, fold)
    return img

def draw_unfolded_wall():
    rng = random.Random(2607)
    img = full((88, 90, 96, 255))
    d = ImageDraw.Draw(img)
    # tessellation: interlocking diamonds in alternating stone shades
    step = 8
    for gy in range(-1, S // step + 1):
        for gx in range(-1, S // step + 1):
            cx, cy = gx * step, gy * step
            col = (150, 152, 156, 255) if (gx + gy) % 2 == 0 else (128, 130, 138, 255)
            d.polygon([(cx + step / 2, cy), (cx + step, cy + step / 2),
                       (cx + step / 2, cy + step), (cx, cy + step / 2)], fill=col)
            d.polygon([(cx + step / 2, cy + 1.4), (cx + step - 1.4, cy + step / 2),
                       (cx + step / 2, cy + step - 1.4), (cx + 1.4, cy + step / 2)],
                      outline=(104, 106, 112, 255))
    # the tessellation shifts as you view it: cyan / red accent edges
    for _ in range(9):
        gx = rng.randrange(0, 4) * step; gy = rng.randrange(0, 4) * step
        col = (0, 178, 188, 255) if rng.random() < 0.5 else (196, 62, 52, 255)
        d.line([gx, gy + step / 2, gx + step / 2, gy + step], fill=col, width=1)
    return img

def draw_shaft(rooved):
    rng = random.Random(2609 if rooved else 2608)
    img = full((14, 16, 22, 255))
    d = ImageDraw.Draw(img)
    # receding shaft cross-sections: the bottom is past where light goes
    for (m, col) in [(1, (52, 58, 70, 255)), (5, (36, 40, 50, 255)),
                     (9, (24, 27, 34, 255)), (13, (14, 16, 20, 255))]:
        d.rectangle([m, m, S - 1 - m, S - 1 - m], outline=col, width=1)
    d.rectangle([15, 15, 17, 17], fill=(2, 2, 4, 255))
    # a cable / rung line fading down one side
    for i in range(6):
        y = 3 + i * 4.6
        fade = max(10, 66 - i * 11)
        d.line([4, y, 6.4, y], fill=(fade, fade + 4, fade + 8, 255), width=1)
    if rooved:
        # roofed over: a shadow band across the top, everything dimmer
        def shade(dd):
            dd.rectangle([0, 0, S - 1, 7], fill=(0, 0, 0, 90))
            dd.rectangle([0, 0, S - 1, S - 1], fill=(0, 0, 0, 36))
        sheen(img, shade)
        d = ImageDraw.Draw(img)
        d.line([0, 7.5, S - 1, 7.5], fill=(70, 76, 88, 255), width=1)  # roof edge
    return img

# ---------------------------------------------------------------- nether
def draw_basalt():
    rng = random.Random(2610)
    img = full((40, 40, 44, 255))
    d = ImageDraw.Draw(img)
    # hexagonal columns, each an exact copy of the next: one stamp, tiled
    r = 4.4
    dx, dy = r * math.sqrt(3), r * 1.5
    row = 0; cy = 1.0
    while cy < S + r:
        cx = 1.0 + (row % 2) * dx / 2
        while cx < S + r:
            pts = _hex_points(cx, cy, r - 0.6)
            d.polygon(pts, fill=(76, 76, 82, 255))
            # identical shading on every column: lit top-left, dark base
            d.line([pts[3], pts[4], pts[5]], fill=(106, 106, 114, 255), width=1)
            d.line([pts[0], pts[1], pts[2]], fill=(48, 48, 54, 255), width=1)
            cx += dx
        cy += dy; row += 1
    return img

def _black_glass(sheen_pts, extra=None):
    img = full((8, 8, 10, 255))
    def gl(dd):
        for (x, y, w, h, a) in sheen_pts:
            dd.ellipse([x, y, x + w, y + h], fill=(210, 220, 235, a))
        if extra:
            extra(dd)
    sheen(img, gl)
    return img

def draw_glass_floor():
    # perfectly reflects the darkness: only a whisper of sheen
    return _black_glass([(4, 6, 15, 3.4, 26), (18, 21, 10, 2.4, 18)])

def draw_glass_floor_clairvoyance():
    def glint(dd):
        rng = random.Random(2612)
        for _ in range(8):  # faint violet clairvoyant shimmer
            x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
            dd.point((x, y), fill=(150, 120, 220, 90))
    return _black_glass([(16, 4, 12, 3, 22), (5, 19, 13, 2.6, 16)], extra=glint)

def draw_glass_wall():
    img = full((9, 9, 11, 255))
    def gl(dd):
        dd.rectangle([6, 0, 11, S - 1], fill=(205, 216, 232, 30))  # vertical sheen
        dd.rectangle([20, 0, 22, S - 1], fill=(205, 216, 232, 16))
    sheen(img, gl)
    d = ImageDraw.Draw(img)
    d.line([0, 1, S - 1, 1], fill=(38, 40, 46, 255), width=1)   # top edge
    d.line([0, S - 2, S - 1, S - 2], fill=(4, 4, 6, 255), width=1)  # base
    return img

# ---------------------------------------------------------------- scarlet sea
def draw_scarlet_water(wave):
    rng = random.Random(2614 if wave else 2613)
    img = full((54, 17, 15, 255))  # dark water under a scarlet sky
    d = ImageDraw.Draw(img)
    for _ in range(10):  # depth patches
        x = rng.uniform(0, S - 6); y = rng.uniform(0, S - 5)
        d.ellipse([x, y, x + rng.uniform(4, 7), y + rng.uniform(3, 5)],
                  fill=(38, 11, 11, 255))
    # scarlet reflections: short wavy ripple strokes
    for _ in range(26 if wave else 17):
        x = rng.uniform(0, S - 6); y = rng.uniform(0, S - 1)
        ln = rng.uniform(3, 6)
        col = (156, 46, 38, 255) if rng.random() < 0.6 else (196, 72, 52, 255)
        pts = [(x + t, y + 0.7 * math.sin(t / 1.4)) for t in range(int(ln))]
        d.line(pts, fill=col, width=1)
    if wave:  # foam-crested wave arcs rolling through
        d = ImageDraw.Draw(img)
        for (wy, off) in [(7, 0), (16, 5), (25, 2)]:
            pts = [(x, wy + 2.2 * math.sin(x / 4.5 + off)) for x in range(S)]
            d.line(pts, fill=(214, 96, 74, 255), width=1)
            crest = [(x, y - 1.6) for (x, y) in pts if int(x + off) % 7 < 2]
            for (x, y) in crest:
                d.point((x, y), fill=(232, 158, 134, 255))
    return img

def draw_scaffolding():
    rng = random.Random(2615)
    img = transparent()
    d = ImageDraw.Draw(img)
    dark, mid, hi = (52, 54, 58, 255), (112, 114, 118, 255), (158, 160, 164, 255)
    def pipe(x0, y0, x1, y1, w=3):
        d.line([x0, y0, x1, y1], fill=dark, width=w + 2)
        d.line([x0, y0, x1, y1], fill=mid, width=w)
        d.line([x0 - 0.7, y0, x1 - 0.7, y1], fill=hi, width=1)
    # a jagged mass of climbable metal: uprights, braces, crossbeams
    pipe(6, 31, 8, 5); pipe(16, 31, 15, 2); pipe(26, 31, 24, 7)
    pipe(4, 12, 28, 9); pipe(5, 22, 27, 24)
    pipe(8, 5, 24, 7, 2); pipe(15, 2, 26, 31, 2); pipe(24, 7, 6, 31, 2)
    pipe(8, 18, 24, 18, 2)
    # jagged broken ends and sharp shards
    for (x, y, a) in [(8, 5, -0.5), (15, 2, 0.2), (24, 7, 0.6), (28, 9, 1.2)]:
        ex, ey = x + 3.4 * math.sin(a), y - 3.4 * math.cos(a)
        d.polygon([(x - 1.4, y), (x + 1.4, y), (ex, ey)], fill=mid)
    for _ in range(7):  # bolts and fittings
        x = rng.uniform(5, S - 6); y = rng.uniform(4, S - 5)
        d.ellipse([x, y, x + 1.8, y + 1.8], fill=hi)
    return img

# ---------------------------------------------------------------- liquids
def draw_saltwater_cube():
    rng = random.Random(2616)
    img = full((22, 70, 138, 255))  # submerged deep blue
    d = ImageDraw.Draw(img)
    for i in range(4):  # darker with depth
        d.rectangle([0, 22 + i * 3, S - 1, 24 + i * 3], fill=(18, 58, 116, 255))
    def rays(dd):  # light filtering down
        for (x, w, a) in [(5, 3.4, 40), (14, 2.4, 30), (23, 4, 36)]:
            dd.polygon([(x, 0), (x + w, 0), (x + w - 2, 20), (x - 2, 20)],
                       fill=(170, 216, 250, a))
    sheen(img, rays)
    d = ImageDraw.Draw(img)
    for _ in range(14):  # caustic wiggles
        x = rng.uniform(0, S - 5); y = rng.uniform(0, S - 1)
        pts = [(x + t, y + 0.8 * math.sin(t / 1.3)) for t in range(4)]
        d.line(pts, fill=(74, 134, 204, 255), width=1)
    for _ in range(7):  # rising bubbles
        x = rng.uniform(2, S - 3); y = rng.uniform(4, S - 3)
        r = rng.uniform(0.8, 1.6)
        d.ellipse([x - r, y - r, x + r, y + r], outline=(186, 224, 252, 255))
    return img

def draw_water_cube_hot():
    rng = random.Random(2617)
    img = full((28, 108, 138, 255))  # warm teal-blue
    d = ImageDraw.Draw(img)
    def warmth(dd):  # the heat in the water
        dd.ellipse([6, 14, 30, 32], fill=(255, 178, 92, 26))
        for _ in range(6):  # rising heat-shimmer columns
            x = rng.uniform(3, S - 4)
            pts = [(x + 1.1 * math.sin(y / 3.2 + x), y) for y in range(4, S - 2, 2)]
            dd.line(pts, fill=(190, 238, 236, 52), width=1)
    sheen(img, warmth)
    d = ImageDraw.Draw(img)
    for _ in range(16):  # busy small bubbles, warmed water churns
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        r = rng.uniform(0.5, 1.1)
        d.ellipse([x - r, y - r, x + r, y + r], outline=(200, 240, 240, 255))
    for _ in range(8):
        x = rng.uniform(0, S - 5); y = rng.uniform(0, S - 1)
        pts = [(x + t, y + 0.7 * math.sin(t / 1.2)) for t in range(4)]
        d.line(pts, fill=(64, 152, 168, 255), width=1)
    return img

# ---------------------------------------------------------------- shore / dirt
def _boulder_base(d, rng):
    d.ellipse([2, 25, 30, 31], fill=(24, 24, 26, 100))  # ground shadow
    # smooth wave-worn rock: one rounded mass, lit from the top-left
    d.ellipse([5, 7, 29, 29], fill=(118, 120, 124, 255))
    d.ellipse([7, 9, 26, 24], fill=(134, 136, 140, 255))
    d.ellipse([9, 10, 20, 17], fill=(152, 154, 156, 255))
    d.ellipse([6, 22, 29, 29], fill=(102, 104, 108, 255))
    for _ in range(5):  # weather smoothing streaks, no sharp cracks
        x = rng.uniform(8, S - 10); y = rng.uniform(12, S - 8)
        d.line([x, y, x + rng.uniform(2, 4), y + rng.uniform(0, 1.4)],
               fill=(142, 144, 146, 255), width=1)

def draw_shore_boulder():
    rng = random.Random(2618)
    img = transparent()
    _boulder_base(ImageDraw.Draw(img, "RGBA"), rng)
    return img

def draw_shore_boulder_oyster():
    rng = random.Random(2619)
    img = transparent()
    d = ImageDraw.Draw(img, "RGBA")
    _boulder_base(d, rng)
    # oyster crust: clustered ridged shells around the waterline and base
    for _ in range(26):
        a = rng.uniform(0, math.tau)
        rr = rng.uniform(0.55, 1.0)
        x = 17 + 12.5 * rr * math.cos(a); y = 20 + 8.5 * abs(math.sin(a)) * rng.uniform(0.4, 1)
        if y < 14:
            y = 14 + rng.uniform(0, 3)
        w = rng.uniform(1.8, 3.2); h = w * 0.72
        col = (228, 224, 214, 255) if rng.random() < 0.55 else (176, 172, 162, 255)
        d.ellipse([x - w / 2, y - h / 2, x + w / 2, y + h / 2], fill=col)
        d.arc([x - w / 2 + 0.4, y - h / 2 + 0.3, x + w / 2 - 0.4, y + h / 2 - 0.3],
              start=200, end=340, fill=(120, 116, 106, 255), width=1)
    return img

def draw_dirt_spawnpoint():
    rng = random.Random(2620)
    img = full((122, 92, 58, 255))  # fine tillage soil
    d = ImageDraw.Draw(img)
    for _ in range(90):
        x = rng.uniform(0, S - 2); y = rng.uniform(0, S - 2)
        col = (106, 78, 48, 255) if rng.random() < 0.5 else (140, 108, 70, 255)
        d.ellipse([x, y, x + rng.uniform(1, 2.2), y + rng.uniform(0.8, 1.6)],
                  fill=col)
    for _ in range(5):  # tiny stones
        x = rng.uniform(1, S - 3); y = rng.uniform(1, S - 3)
        d.ellipse([x, y, x + 1.8, y + 1.4], fill=(168, 158, 142, 255))
    for row in range(4, S, 8):  # faint tillage furrows
        d.line([0, row, S - 1, row + rng.uniform(-0.6, 0.6)],
               fill=(110, 82, 50, 255), width=1)
    return img

# ---------------------------------------------------------------- fungal / bone
def draw_fungus_wall():
    rng = random.Random(2621)
    img = full((70, 72, 62, 255))
    d = ImageDraw.Draw(img)
    # fungus grown thick and tight: packed bulbous masses, seam shadows
    for _ in range(46):
        x = rng.uniform(-2, S - 2); y = rng.uniform(-2, S - 2)
        r = rng.uniform(2.6, 5)
        col = [(88, 90, 78, 255), (98, 100, 86, 255), (80, 82, 70, 255)][rng.randrange(3)]
        d.ellipse([x, y, x + r, y + r * 0.85], fill=col)
    for _ in range(30):  # stiff ridges and highlights on the bumps
        x = rng.uniform(0, S - 2); y = rng.uniform(0, S - 2)
        d.arc([x, y, x + rng.uniform(2, 3.6), y + rng.uniform(1.6, 3)],
              start=190, end=320, fill=(122, 124, 108, 255), width=1)
    for _ in range(12):  # spore dust in the seams
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=(150, 148, 128, 255))
    return img

def draw_bone_floor():
    rng = random.Random(2622)
    img = full((228, 220, 202, 255))  # bone ivory (magenta_white)
    d = ImageDraw.Draw(img)
    def tint(dd):  # the magenta cast of the beast's bone
        for _ in range(6):
            x = rng.uniform(0, S - 9); y = rng.uniform(0, S - 8)
            dd.ellipse([x, y, x + rng.uniform(6, 10), y + rng.uniform(5, 8)],
                       fill=(228, 168, 196, 42))
    sheen(img, tint)
    d = ImageDraw.Draw(img)
    # impregnable bone plates with dark seams
    plates = [(0, 0, 13, 11), (13, 0, 32, 9), (0, 11, 10, 22), (10, 11, 24, 20),
              (24, 9, 32, 21), (0, 22, 15, 32), (15, 20, 32, 32)]
    for (x0, y0, x1, y1) in plates:
        d.rounded_rectangle([x0 + 0.7, y0 + 0.7, x1 - 0.7, y1 - 0.7], radius=2.4,
                            outline=(156, 144, 122, 255), width=1)
    for _ in range(40):  # bone pores and grain
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        d.point((x, y), fill=(206, 196, 172, 255))
    for _ in range(6):  # pale sheen on the hard surface
        x = rng.uniform(2, S - 6); y = rng.uniform(2, S - 5)
        d.line([x, y, x + rng.uniform(2, 4), y - rng.uniform(0, 1)],
               fill=(244, 240, 228, 255), width=1)
    return img

# ---------------------------------------------------------------- gates / windows / glow
def draw_portcullis():
    rng = random.Random(2623)
    img = transparent()
    d = ImageDraw.Draw(img)
    dark, mid, hi = (44, 46, 50, 255), (118, 122, 128, 255), (164, 168, 174, 255)
    def bar_v(x, w=2.6):
        d.rectangle([x, 3, x + w, 28], fill=mid)
        d.line([x, 3, x, 28], fill=dark, width=1)
        d.polygon([(x, 28), (x + w, 28), (x + w / 2, 31)], fill=mid)  # spike tip
    def bar_h(y, w=2.6):
        d.rectangle([2, y, 30, y + w], fill=mid)
        d.line([2, y + w, 30, y + w], fill=dark, width=1)
    # cobbled frame
    d.rectangle([1, 1, 5, 31], fill=mid); d.rectangle([27, 1, 31, 31], fill=mid)
    d.rectangle([1, 1, 31, 4.4], fill=mid)
    d.line([1, 4.4, 31, 4.4], fill=dark, width=1)
    for x in (8.4, 13.8, 19.2, 23.8):
        bar_v(x)
    bar_h(11); bar_h(20.4)
    # scrap patches bolted over the grid: mismatched metal, rivets
    for (x, y, w, h, col) in [(12, 9, 7, 5.4, (146, 104, 70, 255)),
                              (20, 18.6, 8, 5.4, (102, 106, 112, 255)),
                              (6.6, 19, 5.4, 6.4, (128, 118, 96, 255))]:
        d.rectangle([x, y, x + w, y + h], fill=col, outline=dark)
        for (rx, ry) in [(x + 1, y + 1), (x + w - 1, y + 1),
                         (x + 1, y + h - 1), (x + w - 1, y + h - 1)]:
            d.ellipse([rx - 0.7, ry - 0.7, rx + 0.7, ry + 0.7], fill=hi)
    return img

def draw_radiosphere_floor():
    rng = random.Random(2624)
    img = full((36, 196, 218, 255))  # glowing cyan plane (light_emitted 50)
    def glow(dd):
        dd.ellipse([4, 6, 28, 26], fill=(160, 244, 252, 66))  # radiant core
        for _ in range(5):  # slow radiant rings
            x = rng.uniform(4, S - 12); y = rng.uniform(4, S - 10)
            dd.ellipse([x, y, x + rng.uniform(7, 11), y + rng.uniform(5, 8)],
                       outline=(220, 252, 255, 60), width=1)
    sheen(img, glow)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, S - 1, S - 1], outline=(22, 156, 188, 255), width=1)
    for _ in range(16):  # sparkles standing on nothing
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        d.point((x, y), fill=(236, 254, 255, 255))
    return img

def draw_window_shutters():
    rng = random.Random(2625)
    img = full((88, 66, 42, 255))  # window frame
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, S - 1, S - 1], outline=(58, 42, 26, 255), width=2)
    d.rectangle([3, 3, 28, 28], outline=(66, 48, 30, 255), width=1)
    # heavy makeshift shutter: horizontal planks bolted over the glass
    for i, y in enumerate(range(5, 27, 5)):
        col = (148, 114, 72, 255) if i % 2 == 0 else (138, 106, 66, 255)
        d.rectangle([5, y, 27, y + 4.4], fill=col)
        for _ in range(4):  # wood grain
            gx = rng.uniform(6, 22); 
            d.line([gx, y + rng.uniform(0.8, 3.6), gx + rng.uniform(3, 6),
                    y + rng.uniform(0.8, 3.6)], fill=(120, 90, 54, 255), width=1)
        d.line([5, y + 4.4, 27, y + 4.4], fill=(70, 50, 30, 255), width=1)
    # bolts driven into the frame at the plank ends
    for y in range(5, 27, 5):
        for x in (6.4, 25.6):
            d.ellipse([x - 1, y + 1.2, x + 1, y + 3.2], fill=(52, 50, 52, 255))
            d.point((x - 0.3, y + 1.7), fill=(150, 150, 156, 255))
    # the wooden closing bar across the middle, on brackets
    d.rectangle([4, 14.6, 28, 18], fill=(128, 96, 56, 255))
    d.line([4, 14.6, 28, 14.6], fill=(160, 126, 78, 255), width=1)
    d.line([4, 18, 28, 18], fill=(84, 60, 34, 255), width=1)
    for x in (4.6, 25.4):
        d.rectangle([x, 13.8, x + 2, 18.8], fill=(52, 50, 52, 255))
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg) -- full tiles bg ""; the scaffolding is a
# transparent fg rising out of the scarlet shallows (in-batch bg).
JOBS = [
    ("t_beehive_natural", draw_beehive(), "t_beehive_natural", ""),
    ("t_grass_grazed", draw_grass_grazed(), "t_grass_grazed", ""),
    ("t_highland_holy_grass", draw_holy_grass(), "t_highland_holy_grass", ""),
    ("t_fungus_colony", draw_fungus_colony(), "t_fungus_colony", ""),
    ("t_gelatinous_groundcover", draw_gelatinous(), "t_gelatinous_groundcover", ""),
    ("t_folded_wall", draw_folded_wall(), "t_folded_wall", ""),
    ("t_unfolded_wall", draw_unfolded_wall(), "t_unfolded_wall", ""),
    ("t_infinite_shaft_no_roof", draw_shaft(False), "t_infinite_shaft_no_roof", ""),
    ("t_infinite_shaft_rooved", draw_shaft(True), "t_infinite_shaft_rooved", ""),
    ("t_nether_basalt", draw_basalt(), "t_nether_basalt", ""),
    ("t_nether_glass_floor", draw_glass_floor(), "t_nether_glass_floor", ""),
    ("t_nether_glass_floor_clairvoyance", draw_glass_floor_clairvoyance(),
     "t_nether_glass_floor_clairvoyance", ""),
    ("t_nether_glass_wall", draw_glass_wall(), "t_nether_glass_wall", ""),
    ("t_scrap_scaffolding_scarlet", draw_scaffolding(),
     "t_scrap_scaffolding_scarlet", "t_water_sh_scarlet"),
    ("t_water_sh_scarlet", draw_scarlet_water(False), "t_water_sh_scarlet", ""),
    ("t_water_sh_scarlet_wave", draw_scarlet_water(True),
     "t_water_sh_scarlet_wave", ""),
    ("t_saltwater_cube", draw_saltwater_cube(), "t_saltwater_cube", ""),
    ("t_water_cube_hot", draw_water_cube_hot(), "t_water_cube_hot", ""),
    ("t_shore_boulder", draw_shore_boulder(), "t_shore_boulder", ""),
    ("t_shore_boulder_oyster", draw_shore_boulder_oyster(),
     "t_shore_boulder_oyster", ""),
    ("t_dirt_isherwood_player_spawnpoint", draw_dirt_spawnpoint(),
     "t_dirt_isherwood_player_spawnpoint", ""),
    ("t_fungus_wall_transformed", draw_fungus_wall(), "t_fungus_wall_transformed", ""),
    ("t_nm_floor_bone", draw_bone_floor(), "t_nm_floor_bone", ""),
    ("t_portcullis", draw_portcullis(), "t_portcullis", ""),
    ("t_radiosphere_glowing_floor", draw_radiosphere_floor(),
     "t_radiosphere_glowing_floor", ""),
    ("t_window_boarded_shutters_closed", draw_window_shutters(),
     "t_window_boarded_shutters_closed", ""),
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

BACKDROP = (24, 24, 28, 255)

def composited(tid, bg):
    """The tile as the game shows it: fg over its bg, else the backdrop."""
    fg = tile_img(tid)
    if bg:
        out = tile_img(bg)
        out.alpha_composite(fg)
        return out
    if fg.getchannel("A").getextrema()[0] == 255:
        return fg
    out = Image.new("RGBA", (S, S), BACKDROP)
    out.alpha_composite(fg)
    return out

CELL = 64
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), BACKDROP)
for i, (tid, _img, _fg, bg) in enumerate(JOBS):
    t = composited(tid, bg).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_scatter_terrain.png"))
print("wrote preview_scatter_terrain.png", sheet.size)
