#!/usr/bin/env python3
"""
Draw the missing vitrified-terrain sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/
terrain_vitrified.json, CDDA 6a3e59e0) -- the 16 ids below are the
entries in that file listed as missing in the shipped universe
(coverage-audit/missing.json); the file's other entries resolve via
looks_like (t_vitrified_door, t_vitrified_tree, ...) and stay out:

  t_vitrified_dirt            "black glass ground... roughly textured and
                               entirely solid.  The little of it you can
                               see through reveals larger, darker pebbles
                               underneath."  color black, FLAT
  t_vitrified_grass           "black grass... thick, luxuriant midnight
                               black grass.  Every blade is a razor."
                               color dark_gray, SHARP
  t_vitrified_floor           "black glass floor, textured with the knots
                               and whorls of old and well-loved wood
                               boards."  color black, FLAT
  t_vitrified_tile            "black glass floor formed from tessellated
                               octagonal tiles."  color black, FLAT
  t_vitrified_wall            "black glass wall... heavy wall, textured
                               with the knots and whorls of old and
                               well-loved wood boards."  color dark_gray
  t_vitrified_edge            "edge of a vastness... an endless expanse of
                               smooth black."  color black, WALL
  t_vitrified_fence_barbed    "spined glasswire fence... rolling coils of
                               thin black glass covered in delicate barbs.
                               It's reminiscent of snowflakes."
                               color dark_gray, symbol $
  t_vitrified_wall_glass      "translucent black glass wall... translucent
                               enough to allow murky shapes to be seen
                               through it."  color light_gray, WALL
  t_vitrified_sand            "black glass dust... a pile of glass ground
                               so finely that the slightest breeze bears
                               it aloft into mesmerizing clouds."
                               color light_gray, symbol ~
  t_vitrified_curtains        "black glass curtains... the shape of a
                               window and trim, covered in drapings of
                               black glass... now the opaque curtains are
                               solid and immobile."  color dark_gray
  t_vitrified_window_frame    "jagged glass aperture... a ragged hole
                               smashed through a thin portion of black
                               glass.  The biggest shards could probably
                               be snapped off."  color dark_gray, symbol 0
  t_vitrified_bars            "black glass bars... thick spars of black
                               glass extending from floor to ceiling."
                               color light_gray
  t_vitrified_column          "black glass column... a tremendous spar of
                               black glass."  color light_gray, symbol 1
  t_vitrified_bush            "black glass bush... a small shrub grown
                               wild, each of its branches twisting into
                               delicate glass spirals."  color dark_gray
  t_vitrified_stairs_down     "black glass stairs... smooth glass stairs,
                               with countless years of bare footprints worn
                               into them."  color light_gray, GOES_DOWN
  t_vitrified_stairs_up       same stairs, GOES_UP

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet. The whole set is black glass fused by impossible heat: near-
black blue-teal glass, pale teal/gray glassy highlights and specular
streaks, the light_gray ids drawn paler and more translucent-looking.
Full-tile tiles (dirt, floor, tile, wall, edge, sand, wall_glass) are
fg with bg ""; features (grass, bush, bars, column, curtains,
fence_barbed, stairs, window_frame) are transparent fg over
bg "t_vitrified_floor" -- the t_shrub-over-t_grass pattern.
Deterministic: every random choice comes from a per-sprite seeded
Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json +
credits) and preview_vitrified_terrain.png.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def full(color):
    return Image.new("RGBA", (S, S), color)

# black-glass palette: scorched near-black backing, teal/pale glass light
GLASS_DEEP = (14, 19, 22, 255)
GLASS_DARK = (24, 32, 36, 255)
GLASS_MID = (44, 60, 64, 255)
GLASS_PALE = (96, 128, 128, 255)
GLASS_HI = (168, 204, 198, 255)
GLASS_SHEEN = (210, 238, 230, 255)

def glints(d, rng, n, box=(0, 0, S - 1, S - 1), col=GLASS_HI):
    """Sparse pale specular flecks -- the glassy catch-light."""
    for _ in range(n):
        x = rng.uniform(box[0], box[2]); y = rng.uniform(box[1], box[3])
        d.point((x, y), fill=col)

def sheen(d, rng, n=2):
    """One or two broad diagonal specular streaks."""
    for _ in range(n):
        x = rng.uniform(2, 20); y = rng.uniform(12, 28)
        d.polygon([(x, y), (x + 9, y - 8), (x + 12, y - 8), (x + 3, y)],
                  fill=(190, 224, 214, 46))

# ---------------------------------------------------------------- dirt
def draw_dirt():
    rng = random.Random(3301)
    img = full(GLASS_DEEP)
    d = ImageDraw.Draw(img, "RGBA")
    # larger, darker pebbles seen dimly through the glass
    for _ in range(11):
        x = rng.uniform(0, S - 6); y = rng.uniform(0, S - 5)
        w = rng.uniform(4, 8); h = rng.uniform(3, 5)
        d.ellipse([x, y, x + w, y + h], fill=(30, 42, 46, 255))
        d.arc([x, y, x + w, y + h], start=180, end=330, fill=(70, 94, 94, 255))
    # rough texture: dense short strokes and pits over the whole tile
    for _ in range(90):
        x = rng.uniform(0, S - 2); y = rng.uniform(0, S - 1)
        col = GLASS_DARK if rng.random() < 0.6 else (8, 11, 13, 255)
        d.line([x, y, x + rng.uniform(1, 3), y], fill=col, width=1)
    sheen(d, rng, 1)
    glints(d, rng, 14)
    return img

# ---------------------------------------------------------------- wood-glass floors/wall
def _wood_glass(d, rng, vertical=False, heavy=False):
    """Board seams + grain whorls under a glass glaze."""
    seam = (8, 11, 13, 255)
    grain = (52, 66, 62, 255)
    boards = [0, 8, 16, 24, 32]
    for b in boards:
        if vertical:
            d.line([b, 0, b, S - 1], fill=seam, width=1)
        else:
            d.line([0, b, S - 1, b], fill=seam, width=1)
    # long grain streaks along the boards
    for _ in range(26 if not heavy else 34):
        if vertical:
            x = rng.uniform(1, S - 2); y = rng.uniform(0, S - 8)
            d.line([x, y, x + rng.uniform(-1, 1), y + rng.uniform(4, 8)],
                   fill=grain, width=1)
        else:
            x = rng.uniform(0, S - 8); y = rng.uniform(1, S - 2)
            d.line([x, y, x + rng.uniform(4, 8), y + rng.uniform(-1, 1)],
                   fill=grain, width=1)
    # knots: concentric whorl rings, the "well-loved wood" showing through
    knots = [(9, 10, 3.4), (23, 21, 4.0), (17, 6, 2.4)] if not heavy else \
            [(10, 12, 4.4), (24, 24, 5.0), (20, 7, 3.0), (7, 26, 3.2)]
    for (x, y, r) in knots:
        for k in range(3, 0, -1):
            rr = r * k / 3
            d.ellipse([x - rr, y - rr * 0.72, x + rr, y + rr * 0.72],
                      outline=(66, 84, 78, 255), width=1)
        d.point((x, y), fill=(90, 110, 100, 255))

def draw_floor():
    rng = random.Random(3302)
    img = full(GLASS_DARK)
    d = ImageDraw.Draw(img, "RGBA")
    _wood_glass(d, rng)
    sheen(d, rng, 2)
    glints(d, rng, 10)
    return img

def draw_wall():
    rng = random.Random(3303)
    img = full((18, 24, 27, 255))
    d = ImageDraw.Draw(img, "RGBA")
    _wood_glass(d, rng, vertical=True, heavy=True)
    # heavy wall: pale top edge where it catches light, deep base shadow
    d.line([0, 0, S - 1, 0], fill=(120, 150, 144, 255), width=1)
    d.line([0, 1, S - 1, 1], fill=(70, 92, 88, 255), width=1)
    d.rectangle([0, S - 4, S - 1, S - 1], fill=(8, 10, 12, 255))
    sheen(d, rng, 1)
    glints(d, rng, 8)
    return img

# ---------------------------------------------------------------- tile
def _octagon(d, cx, cy, r, fill, edge):
    pts = []
    for k in range(8):
        a = math.pi / 8 + k * math.pi / 4
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    d.polygon(pts, fill=fill, outline=edge)

def draw_tile():
    rng = random.Random(3304)
    img = full((10, 13, 15, 255))  # grout: near-black
    d = ImageDraw.Draw(img, "RGBA")
    tones = [(26, 35, 39, 255), (31, 41, 45, 255), (22, 30, 34, 255),
             (36, 47, 50, 255)]
    # tessellated octagons: full tiles + half tiles wrapping the edges
    centers = [(8, 8), (24, 8), (8, 24), (24, 24),
               (0, 0), (16, 0), (32, 0), (0, 16), (32, 16),
               (0, 32), (16, 32), (32, 32), (16, 16)]
    for (x, y) in centers:
        _octagon(d, x, y, 7.2, tones[rng.randrange(len(tones))],
                 (88, 114, 108, 255))
    glints(d, rng, 12)
    return img

# ---------------------------------------------------------------- edge
def draw_edge():
    rng = random.Random(3305)
    img = full((6, 8, 10, 255))
    d = ImageDraw.Draw(img, "RGBA")
    # endless smooth black: one slow gradient and a single distant sheen
    for y in range(S):
        t = y / (S - 1)
        col = (int(6 + 8 * t), int(8 + 10 * t), int(10 + 12 * t), 255)
        d.line([0, y, S - 1, y], fill=col)
    d.line([0, 9, S - 1, 7], fill=(52, 70, 70, 120), width=1)
    glints(d, rng, 3, col=(120, 150, 144, 255))
    return img

# ---------------------------------------------------------------- sand
def draw_sand():
    rng = random.Random(3306)
    img = full(GLASS_DARK)
    d = ImageDraw.Draw(img, "RGBA")
    # a soft mound of glass dust: paler and finer toward the crest
    for _ in range(240):
        x = rng.uniform(0, S - 1)
        # mound profile: crest around y=19
        y = rng.gauss(19, 6.5)
        if y < 4 or y > S - 1:
            continue
        lift = max(0.0, 1.0 - abs(y - 19) / 15.0)
        base = [int(52 + 74 * lift), int(66 + 84 * lift), int(68 + 82 * lift)]
        d.point((x, y), fill=tuple(base) + (255,))
    # the breeze bearing it aloft: wispy clouds curling off the crest
    for (x0, y0, r) in [(8, 9, 5.5), (19, 6, 7.0), (26, 11, 4.5)]:
        d.arc([x0 - r, y0 - r * 0.5, x0 + r, y0 + r * 0.5],
              start=200, end=520, fill=(150, 182, 174, 150), width=1)
    glints(d, rng, 18, col=GLASS_SHEEN)
    return img

# ---------------------------------------------------------------- wall glass
def draw_wall_glass():
    rng = random.Random(3307)
    img = full((52, 68, 70, 255))
    d = ImageDraw.Draw(img, "RGBA")
    # murky shapes behind the translucent pane: soft dark/light blobs
    for _ in range(7):
        x = rng.uniform(2, S - 9); y = rng.uniform(2, S - 8)
        w = rng.uniform(5, 9); h = rng.uniform(4, 7)
        col = (30, 38, 40, 110) if rng.random() < 0.55 else (96, 118, 112, 90)
        d.ellipse([x, y, x + w, y + h], fill=col)
    # bubbles frozen in the glass
    for _ in range(16):
        x = rng.uniform(1, S - 2); y = rng.uniform(1, S - 2)
        r = rng.uniform(0.7, 1.6)
        d.ellipse([x - r, y - r, x + r, y + r], outline=(140, 172, 164, 200))
    # translucent wash + frame-dark edges (it is still a wall)
    d.rectangle([0, 0, S - 1, S - 1], outline=(16, 21, 24, 255), width=2)
    sheen(d, rng, 2)
    glints(d, rng, 10, col=GLASS_SHEEN)
    return img

# ---------------------------------------------------------------- grass
def draw_grass():
    rng = random.Random(3308)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    # midnight-black razor blades: thin triangles with pale cutting edges
    for _ in range(30):
        x = rng.uniform(0, S - 1)
        h = rng.uniform(9, 21)
        lean = rng.uniform(-3.5, 3.5)
        wdt = rng.uniform(0.8, 1.6)
        body = (10, 13, 15, 255) if rng.random() < 0.5 else (20, 27, 30, 255)
        d.polygon([(x - wdt, S), (x + wdt, S), (x + lean, S - h)], fill=body)
        d.line([x + lean, S - h, x + wdt * 0.4, S - h * 0.45],
               fill=(128, 164, 156, 255), width=1)
    # a low dark tuft mass along the base
    d.rectangle([0, S - 3, S - 1, S - 1], fill=(8, 10, 12, 255))
    return img

# ---------------------------------------------------------------- bush
def draw_bush():
    rng = random.Random(3309)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    stem, hi = (16, 22, 25, 255), (110, 146, 138, 255)
    def spiral(cx, cy, r, turns, phase):
        pts = []
        steps = 26
        for i in range(steps + 1):
            t = i / steps
            a = phase + t * turns * math.tau
            rr = r * (1 - t * 0.82)
            pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a) * 0.9))
        d.line(pts, fill=stem, width=1)
        d.point(pts[-1], fill=hi)
    def branch(x, y, ang, ln, depth):
        ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
        d.line([x, y, ex, ey], fill=stem, width=1)
        if depth <= 0:
            spiral(ex, ey, rng.uniform(2.2, 3.4), rng.uniform(1.4, 2.0),
                   rng.uniform(0, math.tau))
            return
        for na in (ang - 0.5, ang + 0.42):
            branch(ex, ey, na + rng.uniform(-0.08, 0.08), ln * 0.68, depth - 1)
    for bx in (9, 16, 23):
        branch(bx, S - 1, rng.uniform(-0.2, 0.2), 9.0, 2)
    # glass highlight ticks along the stems
    glints(d, rng, 8, box=(4, 6, S - 5, S - 6))
    return img

# ---------------------------------------------------------------- bars / column
def draw_bars():
    rng = random.Random(3310)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    # thick spars, floor to ceiling
    for x in (3, 10, 17, 24):
        w = rng.uniform(2.6, 3.4)
        d.rectangle([x, 0, x + w, S - 1], fill=(22, 30, 33, 255))
        d.line([x + 0.6, 0, x + 0.6, S - 1], fill=(150, 186, 176, 255), width=1)
        d.line([x + w - 0.6, 0, x + w - 0.6, S - 1], fill=(52, 68, 68, 255),
               width=1)
    glints(d, rng, 6)
    return img

def draw_column():
    rng = random.Random(3311)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    # one tremendous faceted spar, slightly tapered, with a capital flare
    d.polygon([(11, 31), (12.5, 4), (19.5, 4), (21, 31)], fill=(26, 35, 38, 255))
    d.polygon([(15, 31), (15.8, 4), (19.5, 4), (21, 31)], fill=(38, 50, 52, 255))
    d.line([13.4, 30, (13.4 + 12.5) / 2 + 1.6, 5], fill=(168, 204, 196, 255),
           width=1)
    d.rectangle([10, 1.5, 22, 4.5], fill=(32, 43, 46, 255))
    d.rectangle([10, 29, 22, 31.5], fill=(18, 24, 27, 255))
    glints(d, rng, 5, box=(12, 6, 20, 28))
    return img

# ---------------------------------------------------------------- curtains
def draw_curtains():
    rng = random.Random(3312)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    # the shape of a window and trim, filled with immobile glass drapings
    d.rectangle([5, 3, 27, 29], outline=(120, 152, 144, 255), width=1)
    d.rectangle([4, 2, 28, 30], outline=(40, 52, 54, 255), width=1)
    # heavy vertical drape folds: wavy bands hanging from the top rail
    fold_cols = [(16, 22, 25, 255), (30, 40, 43, 255), (22, 29, 32, 255),
                 (38, 50, 52, 255)]
    for i, fx in enumerate(range(6, 27, 3)):
        col = fold_cols[i % len(fold_cols)]
        pts = [(fx + 1.2 * math.sin(y / 4.5 + i * 1.7), y) for y in range(4, 29)]
        d.line(pts, fill=col, width=2)
    # highlights riding the fold crests + a pooled hem at the bottom
    for i, fx in enumerate(range(7, 27, 6)):
        pts = [(fx + 1.2 * math.sin(y / 4.5 + i * 1.7), y) for y in range(4, 29)]
        d.line(pts, fill=(104, 136, 128, 255), width=1)
    d.rectangle([5, 27, 27, 29], fill=(12, 16, 18, 255))
    return img

# ---------------------------------------------------------------- fence barbed
def draw_fence_barbed():
    rng = random.Random(3313)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    wire, barb = (88, 116, 110, 255), (178, 214, 204, 255)
    # rolling coils: rows of overlapping loops, like razor wire
    for row_y in (9, 17, 25):
        for cx in range(2, S, 7):
            r = 4.4
            d.ellipse([cx - r, row_y - r * 0.72, cx + r, row_y + r * 0.72],
                      outline=wire, width=1)
        # delicate barbs along the row, snowflake-fine
        for bx in range(1, S, 4):
            y = row_y + rng.uniform(-3.4, 3.4)
            d.line([bx, y, bx + 1.6, y - 1.8], fill=barb, width=1)
            d.line([bx, y, bx - 1.6, y - 1.8], fill=barb, width=1)
    # sagging strands between the coil rows
    for y in (13, 21):
        pts = [(x, y + 1.1 * math.sin(x / 6.0 + y)) for x in range(0, S, 3)]
        d.line(pts, fill=wire, width=1)
    return img

# ---------------------------------------------------------------- stairs
def _footprints(d, rng, ys):
    # smooth worn ovals: countless years of bare footprints
    for y in ys:
        for _ in range(2):
            x = rng.uniform(7, 22)
            d.ellipse([x, y, x + 4.6, y + 2.4], fill=(120, 152, 146, 200))
            d.ellipse([x + 1.2, y + 0.6, x + 3.4, y + 1.8],
                      fill=(168, 198, 188, 200))

def draw_stairs_down():
    rng = random.Random(3314)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    # steps receding downward: treads narrow and darken toward the bottom
    rows = [(2, 9, (86, 112, 106, 255)), (9, 15, (62, 82, 80, 255)),
            (15, 20, (42, 56, 56, 255)), (20, 24, (28, 38, 40, 255)),
            (24, 27, (18, 25, 28, 255)), (27, 30, (10, 14, 16, 255))]
    for (y0, y1, col) in rows:
        inset = (y0 - 2) * 0.28
        d.polygon([(3 + inset, y1), (8 + inset, y0), (24 - inset, y0),
                   (29 - inset, y1)], fill=col)
        d.line([8 + inset, y0, 24 - inset, y0], fill=(196, 226, 216, 255),
               width=1)
    _footprints(d, rng, [4, 11, 17])
    return img

def draw_stairs_up():
    rng = random.Random(3315)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    # steps rising: treads stack up toward a pale top landing
    rows = [(26, 30, (34, 46, 46, 255)), (21, 26, (48, 64, 62, 255)),
            (16, 21, (64, 84, 80, 255)), (11, 16, (82, 106, 100, 255)),
            (6, 11, (102, 130, 122, 255)), (2, 6, (126, 156, 146, 255))]
    for (y0, y1, col) in rows:
        inset = (26 - y0) * 0.22
        d.polygon([(3 + inset, y1), (8 + inset, y0), (24 - inset, y0),
                   (29 - inset, y1)], fill=col)
        d.line([8 + inset, y0, 24 - inset, y0], fill=(206, 234, 224, 255),
               width=1)
    _footprints(d, rng, [23, 13, 4])
    return img

# ---------------------------------------------------------------- window frame
def draw_window_frame():
    rng = random.Random(3316)
    img = new_img(); d = ImageDraw.Draw(img, "RGBA")
    glass, edge = (20, 27, 30, 255), (140, 176, 166, 255)
    # the surviving glass border of the smashed pane
    d.rectangle([0, 0, S - 1, 4], fill=glass)
    d.rectangle([0, S - 5, S - 1, S - 1], fill=glass)
    d.rectangle([0, 0, 4, S - 1], fill=glass)
    d.rectangle([S - 5, 0, S - 1, S - 1], fill=glass)
    # jagged shard teeth jutting into the ragged hole
    shards = [  # (base1, base2, tip) along each side
        ((4, 4), (11, 4), (8, 11)), ((13, 4), (20, 4), (15, 9)),
        ((22, 4), (28, 4), (26, 12)), ((28, 10), (28, 18), (21, 14)),
        ((28, 20), (28, 27), (22, 24)), ((18, 28), (26, 28), (21, 21)),
        ((9, 28), (16, 28), (13, 22)), ((4, 19), (4, 27), (10, 23)),
        ((4, 9), (4, 16), (9, 13)),
    ]
    for (b1, b2, tip) in shards:
        d.polygon([b1, b2, tip], fill=glass, outline=edge)
    # the biggest shard, snapped halfway off, hanging at the top
    d.polygon([(14, 4), (19, 4), (17, 14)], fill=(30, 40, 43, 255),
              outline=(190, 222, 212, 255))
    glints(d, rng, 6, box=(1, 1, S - 2, S - 2))
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg) -- full-tile glass is fg with bg "";
# features are transparent fg over the black glass floor.
JOBS = [
    ("t_vitrified_dirt", draw_dirt(), "t_vitrified_dirt", ""),
    ("t_vitrified_floor", draw_floor(), "t_vitrified_floor", ""),
    ("t_vitrified_tile", draw_tile(), "t_vitrified_tile", ""),
    ("t_vitrified_wall", draw_wall(), "t_vitrified_wall", ""),
    ("t_vitrified_edge", draw_edge(), "t_vitrified_edge", ""),
    ("t_vitrified_sand", draw_sand(), "t_vitrified_sand", ""),
    ("t_vitrified_wall_glass", draw_wall_glass(), "t_vitrified_wall_glass", ""),
    ("t_vitrified_grass", draw_grass(), "t_vitrified_grass", "t_vitrified_floor"),
    ("t_vitrified_bush", draw_bush(), "t_vitrified_bush", "t_vitrified_floor"),
    ("t_vitrified_bars", draw_bars(), "t_vitrified_bars", "t_vitrified_floor"),
    ("t_vitrified_column", draw_column(), "t_vitrified_column", "t_vitrified_floor"),
    ("t_vitrified_curtains", draw_curtains(), "t_vitrified_curtains", "t_vitrified_floor"),
    ("t_vitrified_fence_barbed", draw_fence_barbed(), "t_vitrified_fence_barbed", "t_vitrified_floor"),
    ("t_vitrified_stairs_down", draw_stairs_down(), "t_vitrified_stairs_down", "t_vitrified_floor"),
    ("t_vitrified_stairs_up", draw_stairs_up(), "t_vitrified_stairs_up", "t_vitrified_floor"),
    ("t_vitrified_window_frame", draw_window_frame(), "t_vitrified_window_frame", "t_vitrified_floor"),
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

floor = tile_img("t_vitrified_floor")

def composited(tid, bg):
    """The tile as the game shows it: fg over its bg."""
    fg = tile_img(tid)
    if not bg:
        return fg
    out = floor.copy()
    out.alpha_composite(fg)
    return out

CELL = 64
COLS = 4
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img, _fg, bg) in enumerate(JOBS):
    t = composited(tid, bg).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_vitrified_terrain.png"))
print("wrote preview_vitrified_terrain.png", sheet.size)
