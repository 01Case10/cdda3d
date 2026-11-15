#!/usr/bin/env python3
"""
Draw the missing alien-meadow terrain sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/terrain_alien_meadow.json,
CDDA 6a3e59e0) -- all 16 ids in that file are missing from UltiCa in every
form (no tile-entry JSON, no orphan PNG, no looks_like):

  t_alien_meadow_grass        "light reddish color and the stalks are much
                               broader.  Some kind of waxy residue..."
                               symbol ., color light_red, FLAT
  t_alien_meadow_grass_long   "Dark red in color and almost waist-high, each
                               stalk splits into two to four other stalks...
                               ending in fine fern-like fronds."  color red
  t_alien_meadow_grass_tall   "Head high and such a dark red as to be nearly
                               black, the ends of the long stalks are a burst
                               of fine fronds... forming a canopy."  color red
  t_alien_meadow_shrub_1      "dark blue-green... leaves are all interwoven,
                               forming a sphere except for spots where the
                               leaves have been eaten away... It glows faintly
                               in the dark."  color blue, light_emitted 5
  t_alien_meadow_shrub_2      "looks very similar to a terrestrial bush
                               despite the dark purple leaves.  There are
                               even clusters of white berries."  color magenta
  t_alien_meadow_shrub_2_harvested  same bush, berries gone (HARVESTED)
  t_alien_meadow_shrub_3      "branches split and split again into very fine
                               tendrils with bulbs on the end that float
                               upward, holding its yellow leaves(?) up"
                               color yellow
  t_alien_meadow_tree_1       "a group of individual dark-colored trunks that
                               intertwine... branches have nodes of dark red
                               leaves interspersed"  color red
  t_alien_meadow_tree_2       "vaguely like a willow with yellow leaves...
                               branches that touch the ground have put down
                               roots and are forming secondary trunks"
  t_alien_meadow_tree_3       "trunk lifted off the ground on a cluster of
                               roots.  Giant dark-red globes the size of
                               watermelons hang from its branches amidst
                               clusters of dark-red leaves."  color red
  t_alien_meadow_tree_3_harvested  same tree, globes gone: "clusters of long,
                               dark-red leaves" on a root tangle
  t_alien_meadow_tree_trap    "a group of thick dark-colored trunks half-fused
                               together.  The drooping branches have nodes of
                               dark red leaves" (mature tree_1 form)
  t_alien_meadow_tree_trap_roots_long_grass / _tall_grass
                              game data: "Looks exactly the same so it's not
                              obvious unless the player spots the trap" --
                              their entries therefore reference the SAME fg
                              sprite as the plain long/tall grass. No own PNG.
  t_alien_meadow_tree_trap_roots_obvious
                              "A cluster of dark roots from an alien tree are
                               visible here.  Did one of them just move?"
  t_alien_meadow_pond_goo     "A pool of some very thick, nearly-opaque
                               liquid.  The surface is almost perfectly
                               smooth."  symbol ~, color green

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet
(the standard terrain size). Flat ground tiles (grass, pond goo) are
full-tile fg with bg ""; plant/tree tiles are transparent fg over
bg "t_alien_meadow_grass" -- the t_shrub-over-t_grass pattern. Flat
fills, dark outlines, one highlight pass -- UltiCa style. Deterministic:
every random choice comes from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json + credits).
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

# ---------------------------------------------------------------- grass ground
def draw_grass_ground():
    rng = random.Random(1101)
    img = Image.new("RGBA", (S, S), (198, 112, 88, 255))  # light reddish base
    d = ImageDraw.Draw(img)
    # broad stalk blades lying over the whole tile
    blade_cols = [(214, 128, 98, 255), (186, 98, 76, 255), (224, 142, 108, 255),
                  (176, 90, 70, 255)]
    for _ in range(46):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        wdt = rng.uniform(1.6, 3.0); ln = rng.uniform(4, 8)
        ang = rng.uniform(-0.5, 0.5)
        col = blade_cols[rng.randrange(len(blade_cols))]
        d.polygon([(x, y), (x + wdt, y + 1),
                   (x + wdt * 0.55 + ang * ln, y - ln), (x + ang * ln, y - ln + 1)],
                  fill=col)
    # waxy residue: pale glossy streaks
    for _ in range(9):
        x = rng.uniform(2, S - 4); y = rng.uniform(2, S - 4)
        d.line([x, y, x + rng.uniform(2, 5), y - rng.uniform(0, 2)],
               fill=(238, 186, 158, 160), width=1)
    # soil flecks in the gaps
    for _ in range(26):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, S - 1)
        d.point((x, y), fill=(160, 84, 64, 255))
    return img

# ---------------------------------------------------------------- fern grasses
def frond(d, x, y, ang, ln, col, rng):
    """A fine fern-like frond: a short stem with pinnae lines."""
    ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
    d.line([x, y, ex, ey], fill=col, width=1)
    steps = 4
    for i in range(1, steps + 1):
        t = i / (steps + 1)
        px, py = x + (ex - x) * t, y + (ey - y) * t
        pl = ln * 0.30 * (1 - t * 0.55)
        for sgn in (-1, 1):
            a = ang + sgn * 1.05
            d.line([px, py, px + math.sin(a) * pl, py - math.cos(a) * pl],
                   fill=col, width=1)

def split_stalk(d, x, y, ang, ln, depth, col, hi, rng):
    """Stalk that splits into 2-4 stalks, recursively, ending in fronds."""
    ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
    d.line([x, y, ex, ey], fill=col, width=1)
    if depth <= 0:
        frond(d, ex, ey, ang + rng.uniform(-0.3, 0.3), ln * 0.55, hi, rng)
        return
    n = rng.randint(2, 3) if depth == 1 else rng.randint(2, 4)
    for i in range(n):
        na = ang + (i - (n - 1) / 2.0) * 0.42 + rng.uniform(-0.08, 0.08)
        split_stalk(d, ex, ey, na, ln * 0.62, depth - 1, col, hi, rng)

def draw_grass_long():
    rng = random.Random(1102)
    img = new_img(); d = ImageDraw.Draw(img)
    dark, hi = (128, 24, 30, 255), (172, 44, 44, 255)  # dark red
    for _ in range(7):  # waist-high: stalks fill lower 2/3, fronds above
        x = rng.uniform(2, S - 3)
        split_stalk(d, x, S, rng.uniform(-0.14, 0.14), rng.uniform(9, 12),
                    1, dark, hi, rng)
    return img

def draw_grass_tall():
    rng = random.Random(1103)
    img = new_img(); d = ImageDraw.Draw(img)
    dark, hi = (84, 12, 18, 255), (116, 22, 28, 255)  # nearly black dark red
    for _ in range(8):  # head-high stalks
        x = rng.uniform(1, S - 2)
        split_stalk(d, x, S, rng.uniform(-0.12, 0.12), rng.uniform(13, 16),
                    1, dark, hi, rng)
    # tangled canopy: dense frond bursts overlapping across the top
    for _ in range(15):
        x = rng.uniform(0, S - 1); y = rng.uniform(0, 9)
        frond(d, x, y + 5, rng.uniform(-0.7, 0.7), rng.uniform(4, 6),
              hi if rng.random() < 0.4 else dark, rng)
    return img

# ---------------------------------------------------------------- roots
def draw_roots_obvious():
    rng = random.Random(1104)
    img = new_img(); d = ImageDraw.Draw(img)
    dark, mid, hi = (56, 26, 22, 255), (84, 38, 30, 255), (122, 52, 40, 255)
    # thick roots snaking across the ground (arcs of ellipses)
    for (y0, amp, ph) in [(20, 3.0, 0.0), (25, 2.4, 1.7), (15, 2.0, 3.4)]:
        pts = [(x, y0 + amp * math.sin(x / 5.0 + ph)) for x in range(0, S, 2)]
        d.line(pts, fill=dark, width=3)
        d.line([(x, y - 1) for x, y in pts], fill=mid, width=1)
    # rootlets branching off
    for _ in range(7):
        x = rng.uniform(2, S - 4); y = rng.uniform(12, S - 4)
        d.line([x, y, x + rng.uniform(-4, 4), y + rng.uniform(2, 5)],
               fill=mid, width=1)
    # one root tip raised, mid-motion ("did one of them just move?")
    d.line([22, 24, 25, 18, 27, 15], fill=dark, width=2)
    d.line([22, 23, 25, 17.4, 27, 14.6], fill=hi, width=1)
    d.ellipse([25.6, 12.8, 28.6, 15.8], fill=mid)
    return img

# ---------------------------------------------------------------- shrubs
def leaf_blob(d, cx, cy, r, cols, rng):
    for _ in range(int(r * r * 1.15)):
        a = rng.uniform(0, math.tau); rr = r * math.sqrt(rng.random()) * 0.92
        x, y = cx + math.cos(a) * rr, cy + math.sin(a) * rr * 0.88
        lr = rng.uniform(1.2, 2.6)
        d.ellipse([x - lr, y - lr * 0.6, x + lr, y + lr * 0.6],
                  fill=cols[rng.randrange(len(cols))])

def blob_silhouette(d, cx, cy, r, base, rng):
    """Irregular bush silhouette: a union of jittered overlapping blobs."""
    for _ in range(9):
        bx = cx + rng.uniform(-r * 0.38, r * 0.38)
        by = cy + rng.uniform(-r * 0.34, r * 0.34)
        br = r * rng.uniform(0.42, 0.62)
        d.ellipse([bx - br, by - br * 0.88, bx + br, by + br * 0.88], fill=base)

def draw_shrub_1():
    rng = random.Random(1105)
    img = new_img(); d = ImageDraw.Draw(img)
    # interwoven sphere of dark blue-green leaves
    blob_silhouette(d, 16, 18, 12.5, (18, 66, 60, 255), rng)
    leaf_blob(d, 16, 18, 11.5,
              [(26, 92, 84, 255), (38, 138, 120, 255), (30, 112, 98, 255),
               (48, 158, 132, 255)], rng)
    # woven arcs over the surface
    for i in range(6):
        y = 9 + i * 3.6
        d.arc([5, y - 3, 27, y + 5], start=200, end=340,
              fill=(22, 80, 72, 255), width=1)
    # eaten-away holes: dark pits showing through
    for (x, y, r) in [(10, 13, 2.2), (21, 21, 2.6), (17, 10, 1.6), (24, 14, 1.4)]:
        d.ellipse([x - r, y - r, x + r, y + r], fill=(10, 34, 30, 255))
    # faint glow (light_emitted 5): dim, sparse -- a shimmer, not spots
    for _ in range(9):
        x = rng.uniform(6, 26); y = rng.uniform(8, 28)
        d.point((x, y), fill=(120, 214, 188, 255))
    return img

def draw_shrub_2(harvested):
    rng = random.Random(1106)
    img = new_img(); d = ImageDraw.Draw(img)
    blob_silhouette(d, 16, 19, 13, (66, 28, 88, 255), rng)
    leaf_blob(d, 16, 19, 12,
              [(96, 44, 120, 255), (128, 64, 150, 255), (110, 52, 134, 255),
               (84, 36, 104, 255)], rng)
    if not harvested:  # clusters of white berries
        for (cx, cy) in [(10, 14), (20, 11), (24, 20), (13, 23), (18, 17)]:
            for _ in range(5):
                x = cx + rng.uniform(-2.4, 2.4); y = cy + rng.uniform(-2.0, 2.0)
                d.ellipse([x - 1.3, y - 1.3, x + 1.3, y + 1.3],
                          fill=(246, 246, 250, 255))
                d.point((x - 0.4, y - 0.4), fill=(198, 198, 210, 255))
    else:  # bare berry stems, slightly duller canopy
        for (cx, cy) in [(10, 14), (20, 11), (24, 20), (13, 23)]:
            d.line([cx, cy + 2, cx + rng.uniform(-1, 1), cy - 1],
                   fill=(150, 120, 160, 255), width=1)
    return img

def draw_shrub_3():
    rng = random.Random(1107)
    img = new_img(); d = ImageDraw.Draw(img)
    stem, bulb, leaf = (104, 88, 40, 255), (240, 214, 96, 255), (232, 196, 64, 255)
    def tendril(x, y, ang, ln, depth):
        ex, ey = x + math.sin(ang) * ln, y - math.cos(ang) * ln
        d.line([x, y, ex, ey], fill=stem, width=1)
        if depth <= 0:
            # bulb on the end, floating upward, holding a yellow leaf
            d.line([ex, ey, ex, ey - 2.4], fill=stem, width=1)
            d.ellipse([ex - 1.6, ey - 5.4, ex + 1.6, ey - 2.2], fill=bulb)
            d.ellipse([ex - 2.6, ey - 4.6, ex + 0.2, ey - 2.8], fill=leaf)
            return
        for na in (ang - 0.45, ang + 0.45):
            tendril(ex, ey, na + rng.uniform(-0.07, 0.07), ln * 0.66, depth - 1)
        if rng.random() < 0.7:  # a yellow leaf along the tendril
            d.ellipse([ex - 2.2, ey - 1.2, ex + 0.6, ey + 0.6], fill=leaf)
    # swaying base stems
    for bx in (10, 16, 22):
        tendril(bx, S, rng.uniform(-0.16, 0.16), 8.5, 2)
    return img

# ---------------------------------------------------------------- trees
def leaf_nodes(d, pts, rng, cols=((122, 26, 32, 255), (150, 34, 38, 255),
                                  (100, 20, 26, 255))):
    for (x, y) in pts:  # nodes of dark red leaves along the branches
        for _ in range(5):
            lx = x + rng.uniform(-2.6, 2.6); ly = y + rng.uniform(-2.2, 2.2)
            d.ellipse([lx - 1.7, ly - 1.1, lx + 1.7, ly + 1.1],
                      fill=cols[rng.randrange(len(cols))])

def draw_tree_1():
    rng = random.Random(1108)
    img = new_img(); d = ImageDraw.Draw(img)
    trunk, trunk_hi = (38, 28, 24, 255), (62, 46, 38, 255)
    # intertwining dark trunks
    for (x0, x1, ph) in [(11, 14, 0.0), (16, 15, 2.1), (21, 18, 4.2)]:
        pts = [(x0 + (x1 - x0) * t + 1.6 * math.sin(t * 5 + ph),
                S - 1 - t * 17) for t in [i / 12 for i in range(13)]]
        d.line(pts, fill=trunk, width=3)
        d.line([(x - 1, y) for x, y in pts], fill=trunk_hi, width=1)
    # branches with dark-red leaf nodes interspersed
    nodes = []
    for (x, y, ex, ey) in [(14, 15, 5, 7), (15, 14, 16, 5), (17, 15, 27, 8),
                           (13, 17, 7, 12), (19, 16, 26, 13)]:
        d.line([x, y, ex, ey], fill=trunk, width=2)
        nodes += [(x + (ex - x) * t, y + (ey - y) * t) for t in (0.45, 0.8, 1.0)]
    leaf_nodes(d, nodes, rng)
    return img

def draw_tree_2():
    rng = random.Random(1109)
    img = new_img(); d = ImageDraw.Draw(img)
    trunk, trunk_hi = (52, 40, 26, 255), (80, 62, 40, 255)
    yel = [(226, 190, 70, 255), (240, 208, 92, 255), (206, 168, 52, 255)]
    d.line([16, 31, 15, 13], fill=trunk, width=3)
    d.line([15, 31, 15, 13], fill=trunk_hi, width=1)
    # willow: branches arc out and droop to the ground
    for (ex, bend) in [(3, 6), (8, 9), (24, 9), (29, 6), (16, 11)]:
        pts = []
        for i in range(13):
            t = i / 12
            x = 15.5 + (ex - 15.5) * t
            y = 12 - 6 * math.sin(t * math.pi * 0.62) + bend * t * t + 6 * t
            pts.append((x, min(y, 29)))
        d.line(pts, fill=trunk, width=1)
        # hanging yellow leaves along the droop
        for (x, y) in pts[3:]:
            if rng.random() < 0.75:
                d.ellipse([x - 1.4, y - 0.8, x + 1.4, y + 1.6],
                          fill=yel[rng.randrange(3)])
        # grounded branch tips put down roots: small root flare + mini trunk
        gx, gy = pts[-1]
        d.line([gx, gy, gx - 1.6, gy + 2], fill=trunk, width=1)
        d.line([gx, gy, gx + 1.6, gy + 2], fill=trunk, width=1)
    return img

def draw_tree_3(harvested):
    rng = random.Random(1110)
    img = new_img(); d = ImageDraw.Draw(img)
    trunk, trunk_hi = (44, 30, 24, 255), (70, 50, 38, 255)
    # trunk lifted off the ground on a cluster of roots
    for rx in (10, 13.5, 18.5, 22):
        d.line([16 + (rx - 16) * 0.35, 17, rx, 31], fill=trunk, width=2)
        d.line([16 + (rx - 16) * 0.35, 17, rx + rng.uniform(-2.5, 2.5), 31],
               fill=trunk, width=1)
    d.line([16, 18, 16, 8], fill=trunk, width=3)  # raised trunk
    d.line([15, 18, 15, 8], fill=trunk_hi, width=1)
    nodes = []
    for (ex, ey) in [(5, 4), (12, 2), (21, 3), (27, 6), (8, 9), (25, 11)]:
        d.line([16, 9, ex, ey], fill=trunk, width=2)
        nodes.append((ex, ey))
    if harvested:  # long dark-red leaves only
        for (x, y) in nodes:
            for _ in range(6):
                lx = x + rng.uniform(-3, 3); ly = y + rng.uniform(-1, 3)
                d.line([x, y, lx, ly + 2.4], fill=(132, 28, 34, 255), width=1)
    else:  # giant dark-red globes (watermelon-sized) amid leaf clusters
        leaf_nodes(d, nodes, rng)
        for (x, y, r) in [(8, 7, 4.2), (22, 6, 4.6), (16, 12, 3.6), (27, 12, 3.2)]:
            d.ellipse([x - r, y - r, x + r, y + r], fill=(148, 28, 34, 255))
            d.ellipse([x - r * 0.55, y - r * 0.62, x + r * 0.1, y - r * 0.05],
                      fill=(186, 52, 54, 255))
            d.line([x, y - r - 1.6, x, y - r + 0.4], fill=trunk, width=1)
    return img

def draw_tree_trap():
    rng = random.Random(1111)
    img = new_img(); d = ImageDraw.Draw(img)
    trunk, trunk_hi = (34, 24, 20, 255), (58, 42, 34, 255)
    # thick trunks, half-fused: overlapping wide columns merging at the base
    for x in (11.5, 16, 20.5):
        d.line([x, 31, x + rng.uniform(-1, 1), 13], fill=trunk, width=5)
    for x in (10.2, 14.7, 19.2):
        d.line([x, 31, x, 14], fill=trunk_hi, width=1)
    d.rectangle([8, 27, 24, 31], fill=trunk)  # fused base mass
    # drooping branches with dark-red leaf nodes
    nodes = []
    for (x, ex, ey) in [(12, 4, 6), (16, 15, 3), (20, 28, 7), (13, 6, 11),
                        (19, 26, 12)]:
        pts = [(x + (ex - x) * t, 13 - 3 * math.sin(t * 2.2) + (ey - 10) * t)
               for t in [i / 8 for i in range(9)]]
        d.line(pts, fill=trunk, width=2)
        nodes += [pts[4], pts[-1]]
    leaf_nodes(d, nodes, rng)
    return img

# ---------------------------------------------------------------- pond goo
def draw_pond_goo():
    rng = random.Random(1112)
    img = Image.new("RGBA", (S, S), (30, 102, 48, 255))  # thick green
    d = ImageDraw.Draw(img)
    # nearly-opaque: faint slow swirls under a smooth surface
    for _ in range(7):
        x = rng.uniform(4, 26); y = rng.uniform(4, 26); r = rng.uniform(3, 7)
        d.arc([x - r, y - r * 0.55, x + r, y + r * 0.55],
              start=rng.uniform(0, 180), end=rng.uniform(200, 340),
              fill=(44, 128, 62, 255), width=1)
    for _ in range(10):  # suspended motes
        x = rng.uniform(1, 30); y = rng.uniform(1, 30)
        d.point((x, y), fill=(66, 152, 82, 255))
    # almost perfectly smooth: one broad specular sheen
    d.polygon([(4, 20), (15, 12), (19, 12), (9, 20)], fill=(96, 178, 104, 90))
    d.polygon([(6, 19), (15, 13.4), (17, 13.4), (9.5, 19)], fill=(140, 210, 140, 80))
    # darker meniscus rim
    d.rectangle([0, 0, S - 1, S - 1], outline=(20, 74, 34, 255), width=1)
    return img

# ---------------------------------------------------------------- jobs
# (id, image or None, fg sprite, bg) -- image None = entry reuses another
# sprite (the two hidden trap-root grasses: identical by game design).
JOBS = [
    ("t_alien_meadow_grass", draw_grass_ground(), "t_alien_meadow_grass", ""),
    ("t_alien_meadow_grass_long", draw_grass_long(),
     "t_alien_meadow_grass_long", "t_alien_meadow_grass"),
    ("t_alien_meadow_grass_tall", draw_grass_tall(),
     "t_alien_meadow_grass_tall", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_trap_roots_long_grass", None,
     "t_alien_meadow_grass_long", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_trap_roots_tall_grass", None,
     "t_alien_meadow_grass_tall", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_trap_roots_obvious", draw_roots_obvious(),
     "t_alien_meadow_tree_trap_roots_obvious", "t_alien_meadow_grass"),
    ("t_alien_meadow_shrub_1", draw_shrub_1(),
     "t_alien_meadow_shrub_1", "t_alien_meadow_grass"),
    ("t_alien_meadow_shrub_2", draw_shrub_2(False),
     "t_alien_meadow_shrub_2", "t_alien_meadow_grass"),
    ("t_alien_meadow_shrub_2_harvested", draw_shrub_2(True),
     "t_alien_meadow_shrub_2_harvested", "t_alien_meadow_grass"),
    ("t_alien_meadow_shrub_3", draw_shrub_3(),
     "t_alien_meadow_shrub_3", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_1", draw_tree_1(),
     "t_alien_meadow_tree_1", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_2", draw_tree_2(),
     "t_alien_meadow_tree_2", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_3", draw_tree_3(False),
     "t_alien_meadow_tree_3", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_3_harvested", draw_tree_3(True),
     "t_alien_meadow_tree_3_harvested", "t_alien_meadow_grass"),
    ("t_alien_meadow_tree_trap", draw_tree_trap(),
     "t_alien_meadow_tree_trap", "t_alien_meadow_grass"),
    ("t_alien_meadow_pond_goo", draw_pond_goo(), "t_alien_meadow_pond_goo", ""),
]

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img, fg, bg in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid)
    os.makedirs(folder, exist_ok=True)
    if img is not None:
        img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": fg, "bg": bg}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid, "png" if img is not None else "entry-only (shared fg)")

# ---------------------------------------------------------------- previews
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

grass = tile_img("t_alien_meadow_grass")

def composited(tid):
    """The tile as the game shows it: fg over its bg ground."""
    if tid in ("t_alien_meadow_grass", "t_alien_meadow_pond_goo"):
        return tile_img(tid)
    if tid == "t_alien_meadow_tree_trap_roots_long_grass":
        fg = tile_img("t_alien_meadow_grass_long")
    elif tid == "t_alien_meadow_tree_trap_roots_tall_grass":
        fg = tile_img("t_alien_meadow_grass_tall")
    else:
        fg = tile_img(tid)
    out = grass.copy()
    out.alpha_composite(fg)
    return out

CELL = 64
sheet = Image.new("RGBA", (4 * CELL, 4 * CELL), (24, 24, 28, 255))
for i, (tid, *_rest) in enumerate(JOBS):
    t = composited(tid).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % 4) * CELL, (i // 4) * CELL))
sheet.save(os.path.join(BASE, "preview_alien_meadow.png"))
print("wrote preview_alien_meadow.png", sheet.size)

# combined preview: previous monster batches (native size, grass green,
# contact shadow -- the established preview style) above the terrain grid
monsters = [
    "pngs_normal_32x32/monsters/mon_butterfly/mon_butterfly.png",
    "pngs_normal_32x32/monsters/mon_moth/mon_moth.png",
    "pngs_large_64x64/monsters/mon_butterfly_giant/mon_butterfly_giant.png",
    "pngs_large_64x64/monsters/mon_moth_giant/mon_moth_giant.png",
    "pngs_large_64x64/monsters/mon_caterpillar_butterfly/mon_caterpillar_butterfly.png",
    "pngs_large_64x64/monsters/mon_caterpillar_moth/mon_caterpillar_moth.png",
    "pngs_large_64x64/monsters/mon_butterfly_cocoon/mon_butterfly_cocoon.png",
    "pngs_large_64x64/monsters/mon_moth_cocoon/mon_moth_cocoon.png",
    "pngs_giant_96x96/monsters/mon_butterfly_titan/mon_butterfly_titan.png",
]
W = 4 * CELL
mons = Image.new("RGBA", (W, 224), (96, 148, 60, 255))
x = y = 0; rowh = 0
for rel in monsters:
    im = Image.open(os.path.join(BASE, rel)).convert("RGBA")
    if x + im.width > W:
        x = 0; y += rowh; rowh = 0
    sh = Image.new("RGBA", im.size, (0, 0, 0, 90))
    mons.alpha_composite(sh, (x + 3, y + 5))
    mons.alpha_composite(im, (x, y))
    x += im.width; rowh = max(rowh, im.height)
combined = Image.new("RGBA", (W, sheet.height + mons.height), (24, 24, 28, 255))
combined.alpha_composite(sheet, (0, 0))
combined.alpha_composite(mons, (0, sheet.height))
combined.save(os.path.join(BASE, "preview.png"))
print("wrote preview.png", combined.size)
