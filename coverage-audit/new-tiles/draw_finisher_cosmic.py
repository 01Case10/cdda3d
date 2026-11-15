#!/usr/bin/env python3
"""
Draw the finisher-wave cosmic & misc monster sprites from their game
descriptions. Batch 25 of the Phase 2g.4 coverage drive (42 ids).

Sources (fork-src/cdda/data/json/monsters/*.json, CDDA 6a3e59e0):
yraq.json, zed_fusion.json, kraken_spawn.json, zed-animal.json,
mollusk.json, reverberation.json, triffid.json, string_dimension.json,
singularities.json, defense_bot.json, balthazar_fac_robots.json,
robofac_robots.json, radiosphere.json, reverberation_cabin.json,
cabins_inner.json, rodentkin.json, slimes.json, mutant.json,
misc.json, fungus.json.

Design: flat UltiCa style -- dark outlines, flat fills, one highlight
pass. Bodies are fully opaque (alpha only {0,255}); translucency is
faked with pale opaque tones. Sheet placement follows the standing
volume rule, volume resolved through each entry's copy-from chain:
  < 100 L -> pngs_normal_32x32, 100-400 L -> pngs_large_64x64,
  > 400 L -> pngs_giant_96x96.
Renderers are parameterized in unit space and drawn natively at each
sheet's resolution. Output is UltiCa decomposed layout (png +
tile-entry json + credits per folder). This script only ever creates
or overwrites its own 42 folders and its own preview sheet.
"""
from PIL import Image, ImageDraw
import json, math, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))
CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")
DARK = (24, 20, 28, 255)


class Ctx:
    def __init__(self, size, mid):
        self.size = size
        self.img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        self.d = ImageDraw.Draw(self.img)
        self.rng = random.Random(zlib.crc32(mid.encode()))

    def X(self, pts):
        return [(x * self.size, y * self.size) for x, y in pts]

    def w(self, unit):
        return max(1, int(unit * self.size))

    def ell(self, cx, cy, rx, ry, fill, outline=None, ow=0.012):
        s = self.size
        self.d.ellipse([(cx - rx) * s, (cy - ry) * s, (cx + rx) * s, (cy + ry) * s],
                       fill=fill, outline=outline, width=self.w(ow) if outline else 1)

    def poly(self, pts, fill, outline=None, ow=0.012):
        self.d.polygon(self.X(pts), fill=fill, outline=outline,
                       width=self.w(ow) if outline else 1)

    def line(self, pts, fill, wunit=0.022):
        self.d.line(self.X(pts), fill=fill, width=self.w(wunit))

    def rect(self, x0, y0, x1, y1, fill, outline=None, ow=0.012):
        s = self.size
        self.d.rectangle([x0 * s, y0 * s, x1 * s, y1 * s], fill=fill,
                         outline=outline, width=self.w(ow) if outline else 1)

    def arc(self, cx, cy, rx, ry, a0, a1, fill, wunit=0.02):
        s = self.size
        self.d.arc([(cx - rx) * s, (cy - ry) * s, (cx + rx) * s, (cy + ry) * s],
                   start=a0, end=a1, fill=fill, width=self.w(wunit))

    def blob(self, cx, cy, rx, ry, fill, outline=None, n=10, jit=0.16, ow=0.012):
        pts = []
        for i in range(n):
            a = 2 * math.pi * i / n
            r = 1 + self.rng.uniform(-jit, jit)
            pts.append((cx + math.cos(a) * rx * r, cy + math.sin(a) * ry * r))
        self.poly(pts, fill, outline, ow)

    def curve(self, p0, p1, p2, n=8):
        pts = []
        for i in range(n + 1):
            t = i / n
            x = (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t ** 2 * p2[0]
            y = (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t ** 2 * p2[1]
            pts.append((x, y))
        return pts

    def taper(self, pts, w0, w1, fill, outline=None, ow=0.01):
        n = len(pts)
        left, right = [], []
        for i, (x, y) in enumerate(pts):
            if i == 0:
                dx, dy = pts[1][0] - x, pts[1][1] - y
            elif i == n - 1:
                dx, dy = x - pts[i - 1][0], y - pts[i - 1][1]
            else:
                dx, dy = pts[i + 1][0] - pts[i - 1][0], pts[i + 1][1] - pts[i - 1][1]
            ln = math.hypot(dx, dy) or 1.0
            nx, ny = -dy / ln, dx / ln
            wdt = (w0 + (w1 - w0) * i / (n - 1)) / 2
            left.append((x + nx * wdt, y + ny * wdt))
            right.append((x - nx * wdt, y - ny * wdt))
        self.poly(left + right[::-1], fill, outline, ow)

    def limb(self, pts, wunit, fill):
        self.line(pts, fill, wunit)
        for (x, y) in pts:
            self.ell(x, y, wunit / 2, wunit / 2, fill)

    def teeth(self, cx, cy, r, count, fill, in_frac=0.45):
        """Ring of triangular teeth around a dark opening."""
        for i in range(count):
            a = 2 * math.pi * i / count
            a2 = 2 * math.pi * (i + 0.5) / count
            tip = (cx + math.cos(a2) * r * (1 - in_frac), cy + math.sin(a2) * r * (1 - in_frac))
            b1 = (cx + math.cos(a) * r, cy + math.sin(a) * r)
            b2 = (cx + math.cos(a + 2 * math.pi / count) * r, cy + math.sin(a + 2 * math.pi / count) * r)
            self.poly([b1, tip, b2], fill)


# ---------------------------------------------------------------- palettes
GOLD = (216, 164, 52, 255)
GOLD_L = (246, 208, 96, 255)
GOLD_P = (252, 232, 160, 255)
GOLD_D = (158, 108, 26, 255)
GOLD_X = (110, 74, 14, 255)
FLESH = (148, 128, 92, 255)
FLESH_L = (182, 160, 114, 255)
FLESH_D = (102, 86, 58, 255)
FLESH_P = (198, 132, 120, 255)
BONE = (238, 232, 214, 255)
BLOOD = (150, 50, 44, 255)
UNI_BLUE = (64, 84, 140, 255)
UNI_BLUE_D = (40, 54, 96, 255)
OCTO = (232, 222, 232, 255)
OCTO_P = (206, 120, 180, 255)
OCTO_D = (158, 70, 130, 255)
KRAK = (226, 120, 104, 255)
KRAK_D = (176, 72, 62, 255)
KRAK_Q = (204, 64, 54, 255)
KRAK_QD = (148, 36, 34, 255)
SUCKER = (248, 216, 198, 255)


# ---------------------------------------------------------------- yrax
def yrax_apeirogon(c):
    cx, cy, r = 0.5, 0.52, 0.36
    n = 15
    verts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        rr = r * (1 + c.rng.uniform(-0.10, 0.10))
        verts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr))
    shades = [GOLD, GOLD_L, GOLD_D, GOLD_P]
    for i in range(n):
        c.poly([verts[i], verts[(i + 1) % n], (cx, cy)], shades[i % 4])
    c.poly(verts, None, GOLD_X, 0.014)
    for i in range(0, n, 2):
        c.line([(cx, cy), verts[i]], GOLD_X, 0.008)
    for _ in range(6):
        x, y = cx + c.rng.uniform(-0.2, 0.2), cy + c.rng.uniform(-0.2, 0.2)
        c.poly([(x, y - 0.03), (x + 0.018, y), (x, y + 0.03), (x - 0.018, y)], GOLD_P)


def yrax_delta(c):
    for i, dx in enumerate((-0.30, -0.20, -0.10)):
        c.line([(0.28 + dx * 0.4, 0.30 + i * 0.14), (0.44 + dx * 0.4, 0.30 + i * 0.14)], GOLD_P, 0.014)
    dart = [(0.50, 0.05), (0.82, 0.42), (0.63, 0.50), (0.72, 0.90),
            (0.50, 0.74), (0.28, 0.90), (0.37, 0.50), (0.18, 0.42)]
    c.poly(dart, GOLD, DARK, 0.014)
    c.poly([(0.50, 0.05), (0.63, 0.50), (0.50, 0.74), (0.37, 0.50)], GOLD_L)
    c.line([(0.50, 0.05), (0.50, 0.74)], GOLD_X, 0.010)
    c.ell(0.50, 0.47, 0.035, 0.035, GOLD_P)


def yrax_triakis(c):
    # short snakelike limbs first (behind body)
    for (sx, sy, ex, ey) in [(0.24, 0.66, 0.06, 0.86), (0.76, 0.66, 0.94, 0.86),
                             (0.32, 0.74, 0.16, 0.94), (0.68, 0.74, 0.84, 0.94)]:
        spine = c.curve((sx, sy), ((sx + ex) / 2, sy + 0.02), (ex, ey), 7)
        c.taper(spine, 0.075, 0.02, GOLD_D, GOLD_X)
    # aggressively faceted low body: rows of triangles
    body = [(0.14, 0.72), (0.24, 0.44), (0.42, 0.30), (0.62, 0.30), (0.78, 0.46), (0.86, 0.72)]
    c.poly(body, GOLD, DARK, 0.014)
    tris = [([(0.24, 0.44), (0.42, 0.30), (0.36, 0.58)], GOLD_L),
            ([(0.42, 0.30), (0.62, 0.30), (0.52, 0.56)], GOLD_P),
            ([(0.62, 0.30), (0.78, 0.46), (0.66, 0.60)], GOLD_L),
            ([(0.14, 0.72), (0.24, 0.44), (0.36, 0.58), (0.30, 0.72)], GOLD_D),
            ([(0.36, 0.58), (0.52, 0.56), (0.50, 0.72), (0.30, 0.72)], GOLD),
            ([(0.52, 0.56), (0.66, 0.60), (0.72, 0.72), (0.50, 0.72)], GOLD_D),
            ([(0.66, 0.60), (0.78, 0.46), (0.86, 0.72), (0.72, 0.72)], GOLD)]
    for pts, col in tris:
        c.poly(pts, col)
    c.line([(0.14, 0.72), (0.86, 0.72)], GOLD_X, 0.012)


def yrax_trifacet(c):
    # three dual leg modules at the pyramid's lower vertices
    for bx in (0.26, 0.50, 0.74):
        for splay in (-0.055, 0.055):
            c.limb([(bx, 0.56), (bx + splay * 0.6, 0.74), (bx + splay, 0.90)], 0.030, GOLD_D)
            c.ell(bx + splay, 0.905, 0.028, 0.014, GOLD_X)
    pyr = [(0.50, 0.10), (0.82, 0.58), (0.18, 0.58)]
    c.poly(pyr, GOLD, DARK, 0.014)
    c.poly([(0.50, 0.10), (0.82, 0.58), (0.50, 0.58)], GOLD_L)
    c.poly([(0.50, 0.10), (0.50, 0.58), (0.18, 0.58)], GOLD_D)
    c.line([(0.50, 0.10), (0.50, 0.58)], GOLD_X, 0.010)
    c.poly([(0.50, 0.30), (0.60, 0.50), (0.50, 0.56), (0.40, 0.50)], GOLD_P)


def yrax_quadraphract(c):
    # four smooth faceted legs, jointless tapered ribbons
    for (tx, fx) in [(0.30, 0.16), (0.44, 0.36), (0.58, 0.64), (0.72, 0.86)]:
        spine = c.curve((tx, 0.44), ((tx + fx) / 2 + 0.02, 0.66), (fx, 0.90), 8)
        c.taper(spine, 0.085, 0.045, GOLD, GOLD_X)
        c.ell(fx, 0.915, 0.045, 0.016, GOLD_D)
    # gem-like body
    gem = [(0.50, 0.16), (0.70, 0.24), (0.78, 0.40), (0.64, 0.52),
           (0.38, 0.52), (0.24, 0.40), (0.31, 0.24)]
    c.poly(gem, GOLD, DARK, 0.012)
    c.poly([(0.50, 0.16), (0.70, 0.24), (0.50, 0.38)], GOLD_P)
    c.poly([(0.50, 0.16), (0.50, 0.38), (0.31, 0.24)], GOLD_L)
    c.poly([(0.70, 0.24), (0.78, 0.40), (0.50, 0.38)], GOLD_L)
    c.poly([(0.31, 0.24), (0.50, 0.38), (0.24, 0.40)], GOLD_D)
    c.poly([(0.50, 0.38), (0.78, 0.40), (0.64, 0.52)], GOLD_D)
    c.poly([(0.50, 0.38), (0.64, 0.52), (0.38, 0.52), (0.24, 0.40)], GOLD)
    # goblet-shaped crown on the peak, slowly swivelling
    c.rect(0.475, 0.115, 0.525, 0.165, GOLD_D)
    c.poly([(0.40, 0.02), (0.60, 0.02), (0.545, 0.115), (0.455, 0.115)], GOLD_L, DARK, 0.010)
    c.ell(0.50, 0.022, 0.10, 0.020, GOLD_X)
    c.ell(0.50, 0.020, 0.075, 0.013, (60, 30, 60, 255))


def yrax_quadraphract_leg(c):
    # a morphing lune: one great crescent of burnished gold
    outer, inner = [], []
    for i in range(25):
        a = math.radians(-115 + 230 * i / 24)
        outer.append((0.52 + 0.44 * math.cos(a), 0.52 + 0.44 * math.sin(a)))
    for i in range(25):
        a = math.radians(115 - 230 * i / 24)
        inner.append((0.66 + 0.335 * math.cos(a), 0.52 + 0.335 * math.sin(a)))
    lune = outer + inner
    c.poly(lune, GOLD, DARK, 0.012)
    for f in (0.25, 0.45, 0.65):
        x0 = 0.52 - 0.44 * f * 0.4
        c.arc(0.52, 0.52, 0.44 - 0.10 * f * 2, 0.44 - 0.10 * f * 2, -100, 100, GOLD_X, 0.008)
    mid = outer[12]
    c.poly([(mid[0], mid[1] - 0.05), (mid[0] + 0.05, mid[1]), (mid[0], mid[1] + 0.05),
            (mid[0] - 0.05, mid[1])], GOLD_P)
    tip = outer[0]
    c.poly([(tip[0], tip[1]), (tip[0] + 0.10, tip[1] + 0.05), (tip[0] + 0.02, tip[1] + 0.10)], GOLD_L)


def yrax_sphenocorona(c):
    # manipulator arrays protracting from asymmetric facets, floating
    for (sx, sy, ex, ey) in [(0.30, 0.44, 0.08, 0.30), (0.70, 0.44, 0.93, 0.28),
                             (0.36, 0.60, 0.16, 0.78), (0.64, 0.60, 0.86, 0.80),
                             (0.50, 0.64, 0.50, 0.86)]:
        spine = c.curve((sx, sy), ((sx + ex) / 2, (sy + ey) / 2 - 0.03), (ex, ey), 7)
        c.taper(spine, 0.045, 0.012, GOLD_D, GOLD_X)
        c.ell(ex, ey, 0.016, 0.016, GOLD_P)
    crown = [(0.16, 0.52), (0.22, 0.30), (0.34, 0.44), (0.42, 0.22), (0.52, 0.40),
             (0.60, 0.26), (0.70, 0.42), (0.80, 0.30), (0.86, 0.52), (0.72, 0.62),
             (0.52, 0.56), (0.30, 0.64)]
    c.poly(crown, GOLD, DARK, 0.012)
    c.poly([(0.22, 0.30), (0.34, 0.44), (0.30, 0.64), (0.16, 0.52)], GOLD_D)
    c.poly([(0.42, 0.22), (0.52, 0.40), (0.52, 0.56), (0.34, 0.44)], GOLD_L)
    c.poly([(0.60, 0.26), (0.70, 0.42), (0.72, 0.62), (0.52, 0.56), (0.52, 0.40)], GOLD_P)
    c.poly([(0.70, 0.42), (0.80, 0.30), (0.86, 0.52), (0.72, 0.62)], GOLD_D)
    for _ in range(7):
        x, y = c.rng.uniform(0.08, 0.92), c.rng.uniform(0.06, 0.30)
        c.ell(x, y, 0.010, 0.010, GOLD_P)


def golden_monolith(c):
    slab = [(0.38, 0.05), (0.60, 0.03), (0.635, 0.30), (0.615, 0.34), (0.65, 0.62),
            (0.625, 0.93), (0.365, 0.95), (0.345, 0.60), (0.375, 0.56), (0.35, 0.28)]
    c.poly(slab, GOLD, DARK, 0.012)
    c.poly([(0.38, 0.05), (0.50, 0.04), (0.50, 0.94), (0.365, 0.95), (0.345, 0.60),
            (0.375, 0.56), (0.35, 0.28)], GOLD_L)
    c.poly([(0.50, 0.04), (0.60, 0.03), (0.635, 0.30), (0.615, 0.34), (0.65, 0.62),
            (0.625, 0.93), (0.50, 0.94)], GOLD_D)
    for y in (0.22, 0.48, 0.74):
        c.line([(0.40, y), (0.58, y - 0.015)], GOLD_X, 0.009)
    c.line([(0.44, 0.10), (0.43, 0.86)], GOLD_P, 0.008)
    c.rect(0.30, 0.945, 0.68, 0.965, GOLD_X)


# ---------------------------------------------------------------- devourers
def _flesh_mass(c, cx, cy, rx, ry, base=FLESH):
    c.blob(cx, cy, rx, ry, base, DARK, n=13, jit=0.13)
    c.blob(cx - rx * 0.25, cy - ry * 0.3, rx * 0.45, ry * 0.4, FLESH_L, None, n=9, jit=0.2)


def devourer_daitya_clone(c):
    _flesh_mass(c, 0.5, 0.56, 0.34, 0.32)
    for (ox, oy, r) in [(0.36, 0.44, 0.085), (0.62, 0.52, 0.105), (0.44, 0.70, 0.07)]:
        c.ell(ox, oy, r, r * 0.85, (30, 16, 14, 255))
        c.teeth(ox, oy, r * 0.92, 8, BONE)
    c.limb([(0.18, 0.62), (0.10, 0.72)], 0.05, FLESH_D)
    c.limb([(0.82, 0.60), (0.90, 0.70)], 0.05, FLESH_D)


def devourer_daitya(c):
    _flesh_mass(c, 0.48, 0.55, 0.38, 0.36)
    # small clones straining to birth themselves from the bulk
    for (hx, hy, r) in [(0.30, 0.26, 0.085), (0.55, 0.18, 0.10), (0.76, 0.34, 0.08)]:
        c.ell(hx, hy, r, r * 1.1, FLESH_P, DARK, 0.010)
        c.ell(hx - r * 0.3, hy - r * 0.15, 0.014, 0.018, DARK)
        c.ell(hx + r * 0.3, hy - r * 0.15, 0.014, 0.018, DARK)
        for k in range(3):
            tx = hx - r * 0.5 + k * r * 0.5
            c.poly([(tx, hy + r * 0.55), (tx + r * 0.22, hy + r * 0.55), (tx + r * 0.11, hy + r * 0.85)], BONE)
    # limbs pushing the straining masses back inside
    for (sx, sy, ex, ey) in [(0.14, 0.44, 0.26, 0.30), (0.86, 0.50, 0.78, 0.38)]:
        c.limb([(sx, sy), ((sx + ex) / 2, (sy + ey) / 2 - 0.03), (ex, ey)], 0.055, FLESH)
        c.ell(ex, ey, 0.035, 0.028, FLESH_L)
    # one escapee at lower right, snatched by a long arm
    c.ell(0.90, 0.80, 0.045, 0.055, FLESH_P, DARK, 0.010)
    c.limb([(0.88, 0.85), (0.86, 0.92)], 0.03, FLESH_P)
    c.limb([(0.72, 0.66), (0.82, 0.74), (0.885, 0.79)], 0.04, FLESH)
    c.ell(0.50, 0.60, 0.09, 0.07, (30, 16, 14, 255))
    c.teeth(0.50, 0.60, 0.085, 9, BONE)


def devourer_hekatonkheire(c):
    _flesh_mass(c, 0.5, 0.58, 0.40, 0.34)
    # dozens of arms radiating
    arms = [(0.16, 0.42, 0.03, 0.30), (0.12, 0.58, 0.02, 0.62), (0.20, 0.74, 0.06, 0.90),
            (0.84, 0.42, 0.97, 0.30), (0.88, 0.58, 0.98, 0.62), (0.80, 0.74, 0.94, 0.90),
            (0.34, 0.30, 0.24, 0.12), (0.66, 0.30, 0.76, 0.12), (0.28, 0.84, 0.20, 0.97),
            (0.72, 0.84, 0.80, 0.97)]
    for (sx, sy, ex, ey) in arms:
        c.limb([(sx, sy), ((sx + ex) / 2, (sy + ey) / 2), (ex, ey)], 0.048, FLESH)
        for f in (-0.02, 0.0, 0.02):
            c.line([(ex, ey), (ex + f * 2.2, ey + (0.035 if ey > sy else -0.035))], FLESH_L, 0.012)
    # a multitude of heads around the crown
    for (hx, hy, r) in [(0.32, 0.24, 0.062), (0.50, 0.17, 0.072), (0.68, 0.24, 0.062),
                        (0.20, 0.34, 0.052), (0.80, 0.34, 0.052), (0.41, 0.28, 0.05), (0.59, 0.28, 0.05)]:
        c.ell(hx, hy, r, r * 1.12, FLESH_P, DARK, 0.009)
        c.ell(hx - r * 0.32, hy - r * 0.1, 0.011, 0.014, DARK)
        c.ell(hx + r * 0.32, hy - r * 0.1, 0.011, 0.014, DARK)
        c.ell(hx, hy + r * 0.45, r * 0.3, r * 0.18, (30, 16, 14, 255))
    c.ell(0.50, 0.62, 0.10, 0.075, (30, 16, 14, 255))
    c.teeth(0.50, 0.62, 0.095, 10, BONE)


def devourer_lab_sec(c):
    # tangle of fused security personnel: blue gear mass with flesh
    c.blob(0.5, 0.55, 0.40, 0.37, UNI_BLUE, DARK, n=13, jit=0.13)
    c.blob(0.36, 0.38, 0.20, 0.16, UNI_BLUE_D, None, n=9, jit=0.2)
    c.blob(0.66, 0.66, 0.20, 0.15, UNI_BLUE_D, None, n=9, jit=0.2)
    # flesh patches and warped limbs woven through
    for (lx, ly, ex, ey) in [(0.22, 0.34, 0.12, 0.20), (0.80, 0.30, 0.90, 0.16),
                             (0.18, 0.70, 0.08, 0.84), (0.84, 0.74, 0.94, 0.86),
                             (0.40, 0.86, 0.36, 0.97)]:
        c.limb([(lx, ly), ((lx + ex) / 2, (ly + ey) / 2), (ex, ey)], 0.05, FLESH)
    c.blob(0.70, 0.40, 0.10, 0.09, FLESH, None, n=8, jit=0.2)
    c.blob(0.30, 0.62, 0.09, 0.08, FLESH_P, None, n=8, jit=0.2)
    # helmets and vests: recognizable gear shapes
    for (hx, hy) in [(0.30, 0.26), (0.62, 0.20), (0.78, 0.52)]:
        c.arc(hx, hy + 0.03, 0.055, 0.05, 180, 360, (30, 34, 44, 255), 0.045)
        c.rect(hx - 0.06, hy + 0.025, hx + 0.06, hy + 0.04, (30, 34, 44, 255))
    c.rect(0.24, 0.48, 0.36, 0.62, (34, 40, 54, 255), DARK, 0.008)
    c.rect(0.66, 0.56, 0.78, 0.70, (34, 40, 54, 255), DARK, 0.008)
    # a rifle shape embedded diagonally, flesh-covered
    c.limb([(0.30, 0.78), (0.52, 0.60)], 0.028, (26, 26, 30, 255))
    c.rect(0.50, 0.545, 0.60, 0.585, (26, 26, 30, 255))
    c.blob(0.42, 0.68, 0.05, 0.035, FLESH, None, n=7, jit=0.25)
    # the pitch-black figure at its center, mirroring the mass
    c.ell(0.50, 0.36, 0.045, 0.052, (6, 6, 8, 255))
    c.poly([(0.455, 0.41), (0.545, 0.41), (0.56, 0.60), (0.44, 0.60)], (6, 6, 8, 255))
    c.limb([(0.45, 0.44), (0.36, 0.54)], 0.028, (6, 6, 8, 255))
    c.limb([(0.55, 0.44), (0.64, 0.54)], 0.028, (6, 6, 8, 255))
    c.limb([(0.47, 0.60), (0.44, 0.72)], 0.028, (6, 6, 8, 255))
    c.limb([(0.53, 0.60), (0.56, 0.72)], 0.028, (6, 6, 8, 255))


# ---------------------------------------------------------------- cephalopods & sea
def _octo_arms(c, cx, base_y, spread, count, w0, col, dark):
    for i in range(count):
        f = i / (count - 1) if count > 1 else 0.5
        ax = cx + (f - 0.5) * spread
        curl = (f - 0.5) * 0.22
        spine = c.curve((cx + (f - 0.5) * spread * 0.4, base_y - 0.06),
                        (ax, base_y + 0.08), (ax + curl, base_y + 0.16 - abs(f - 0.5) * 0.08), 7)
        c.taper(spine, w0, w0 * 0.35, col, dark, 0.008)


def octupus_stalker(c):
    _octo_arms(c, 0.5, 0.66, 0.62, 7, 0.075, OCTO, OCTO_D)
    c.ell(0.5, 0.38, 0.27, 0.30, OCTO, DARK, 0.012)
    # camouflage mottling: magenta patches mimicking surroundings
    for (px, py, pr) in [(0.38, 0.24, 0.075), (0.60, 0.30, 0.09), (0.46, 0.46, 0.07),
                         (0.66, 0.48, 0.055), (0.33, 0.44, 0.05)]:
        c.blob(px, py, pr, pr * 0.8, OCTO_P, None, n=7, jit=0.25)
    c.ell(0.40, 0.40, 0.030, 0.038, DARK)
    c.ell(0.60, 0.40, 0.030, 0.038, DARK)
    c.ell(0.5, 0.20, 0.09, 0.045, (246, 240, 246, 255))


def octupus_stalker_spawn(c):
    _octo_arms(c, 0.5, 0.62, 0.5, 6, 0.06, OCTO, OCTO_D)
    c.ell(0.5, 0.40, 0.24, 0.26, OCTO, DARK, 0.014)
    c.blob(0.42, 0.26, 0.07, 0.05, OCTO_P, None, n=7, jit=0.25)
    c.blob(0.60, 0.34, 0.055, 0.045, OCTO_P, None, n=7, jit=0.25)
    c.ell(0.41, 0.42, 0.038, 0.046, DARK)
    c.ell(0.59, 0.42, 0.038, 0.046, DARK)
    c.ell(0.395, 0.405, 0.013, 0.016, (255, 255, 255, 255))
    c.ell(0.575, 0.405, 0.013, 0.016, (255, 255, 255, 255))


def kraken_guard(c):
    # thick tentacles spreading along the bottom, suckers on the inner faces
    for i, (bx, tx) in enumerate([(0.16, 0.04), (0.32, 0.24), (0.50, 0.50), (0.68, 0.76), (0.84, 0.96)]):
        spine = c.curve((0.5 + (bx - 0.5) * 0.5, 0.62), (bx, 0.78), (tx, 0.90 - abs(i - 2) * 0.02), 8)
        c.taper(spine, 0.11, 0.05, KRAK, DARK, 0.009)
        for t in (0.45, 0.65, 0.85):
            sx = spine[int(t * 8)][0] + 0.02
            sy = spine[int(t * 8)][1]
            c.ell(sx, sy, 0.020, 0.016, SUCKER)
    c.ell(0.5, 0.36, 0.31, 0.30, KRAK, DARK, 0.012)
    for _ in range(9):
        x, y = c.rng.uniform(0.28, 0.72), c.rng.uniform(0.14, 0.42)
        c.ell(x, y, 0.020, 0.015, KRAK_D)
    c.ell(0.5, 0.18, 0.11, 0.05, (240, 156, 140, 255))
    # fierce eyes under angled brows
    c.limb([(0.34, 0.36), (0.44, 0.41)], 0.022, DARK)
    c.limb([(0.66, 0.36), (0.56, 0.41)], 0.022, DARK)
    c.ell(0.40, 0.45, 0.030, 0.036, DARK)
    c.ell(0.60, 0.45, 0.030, 0.036, DARK)


def kraken_queen(c):
    # a great bulk of tentacles breaching; beak rises above the mass
    for i in range(7):
        f = i / 6
        bx = 0.14 + f * 0.72
        sway = (f - 0.5) * 0.30
        spine = c.curve((bx, 0.86), (bx + sway * 0.4, 0.55), (bx + sway, 0.30 + abs(f - 0.5) * 0.22), 8)
        c.taper(spine, 0.105, 0.045, KRAK_Q, DARK, 0.009)
        for t in (0.4, 0.6, 0.8):
            p = spine[int(t * 8)]
            c.ell(p[0] - 0.018, p[1], 0.016, 0.013, SUCKER)
    c.ell(0.5, 0.60, 0.20, 0.20, KRAK_Q, DARK, 0.011)
    c.blob(0.42, 0.46, 0.08, 0.06, KRAK_QD, None, n=8, jit=0.2)
    # the beak, open in a shriek
    c.poly([(0.435, 0.30), (0.565, 0.30), (0.50, 0.46)], (40, 26, 22, 255), DARK, 0.008)
    c.poly([(0.45, 0.30), (0.55, 0.30), (0.50, 0.20)], (52, 34, 28, 255), DARK, 0.008)
    for r in (0.13, 0.19):
        c.arc(0.5, 0.26, r, r * 0.8, 200, 340, SUCKER, 0.010)
    # water froth along the base
    for i in range(16):
        x = 0.05 + i * 0.058 + c.rng.uniform(-0.012, 0.012)
        c.ell(x, 0.90 + c.rng.uniform(-0.012, 0.012), c.rng.uniform(0.022, 0.042),
              c.rng.uniform(0.014, 0.026), (244, 248, 250, 255))
    c.rect(0.02, 0.925, 0.98, 0.945, (210, 232, 240, 255))


def ocean_zhark(c):
    body = [(0.06, 0.48), (0.16, 0.36), (0.38, 0.28), (0.62, 0.27), (0.80, 0.33),
            (0.90, 0.24), (0.86, 0.44), (0.95, 0.60), (0.88, 0.66), (0.64, 0.66),
            (0.38, 0.68), (0.16, 0.62), (0.07, 0.55)]
    c.poly(body, (204, 184, 202, 255), DARK, 0.012)
    belly = [(0.10, 0.55), (0.20, 0.63), (0.42, 0.675), (0.66, 0.655), (0.84, 0.63),
             (0.80, 0.50), (0.60, 0.55), (0.34, 0.56), (0.16, 0.53)]
    c.poly(belly, (240, 238, 240, 255))
    c.poly([(0.44, 0.28), (0.56, 0.10), (0.62, 0.28)], (204, 184, 202, 255), DARK, 0.010)
    c.poly([(0.52, 0.62), (0.62, 0.80), (0.68, 0.62)], (190, 168, 188, 255), DARK, 0.010)
    for gx in (0.30, 0.345, 0.39):
        c.arc(gx, 0.47, 0.030, 0.075, 300, 120, (120, 100, 118, 255), 0.012)
    # dead milky eye and jagged undead grin
    c.ell(0.155, 0.44, 0.024, 0.024, (236, 240, 236, 255), DARK, 0.008)
    mouth = [(0.065, 0.545), (0.14, 0.575), (0.22, 0.565), (0.28, 0.585)]
    c.line(mouth, (90, 40, 44, 255), 0.014)
    for i, (tx, ty) in enumerate([(0.10, 0.565), (0.16, 0.585), (0.22, 0.575)]):
        c.poly([(tx, ty), (tx + 0.022, ty), (tx + 0.011, ty + 0.028)], BONE)
    # rot wounds on the flank
    for (wx, wy) in [(0.55, 0.42), (0.68, 0.50)]:
        c.blob(wx, wy, 0.045, 0.028, (168, 58, 58, 255), None, n=7, jit=0.3)
        c.blob(wx, wy, 0.022, 0.013, (120, 36, 40, 255), None, n=7, jit=0.3)


# ---------------------------------------------------------------- snails
def _snail(c, scale_body, shell_r, acid):
    # foot/body along the bottom, head to the left
    c.poly([(0.10, 0.80), (0.30, 0.70), (0.78, 0.70), (0.92, 0.80), (0.90, 0.88),
            (0.14, 0.88)], (150, 116, 78, 255), DARK, 0.012)
    c.ell(0.16, 0.72, 0.075, 0.09, (150, 116, 78, 255), DARK, 0.010)
    # eyestalks
    for (ex, ey) in [(0.115, 0.52), (0.185, 0.50)]:
        c.limb([(0.155, 0.68), (ex, ey)], 0.020, (150, 116, 78, 255))
        c.ell(ex, ey, 0.020, 0.020, DARK)
    # shell with spiral
    sx, sy = 0.58, 0.46
    c.ell(sx, sy, shell_r, shell_r * 0.92, (128, 92, 58, 255), DARK, 0.012)
    for f in (0.78, 0.56, 0.36):
        c.arc(sx, sy, shell_r * f, shell_r * f * 0.92, 300, 200, (92, 64, 36, 255), 0.016)
    c.ell(sx - shell_r * 0.2, sy - shell_r * 0.25, shell_r * 0.22, shell_r * 0.14, (172, 134, 92, 255))
    if acid:
        c.ell(0.22, 0.915, 0.16, 0.028, (168, 214, 120, 255))
        c.ell(0.55, 0.925, 0.22, 0.024, (150, 200, 105, 255))
        c.ell(0.13, 0.80, 0.018, 0.030, (168, 214, 120, 255))


def snail_giant(c):
    _snail(c, 1.0, 0.335, True)


def snail_small(c):
    _snail(c, 1.0, 0.29, True)


def snail_river(c):
    # a tiny shelled snail, small in the frame
    c.poly([(0.34, 0.80), (0.46, 0.74), (0.66, 0.74), (0.72, 0.80), (0.70, 0.85),
            (0.37, 0.85)], (150, 116, 78, 255), DARK, 0.016)
    c.ell(0.385, 0.755, 0.035, 0.045, (150, 116, 78, 255), DARK, 0.012)
    for (ex, ey) in [(0.36, 0.66), (0.405, 0.65)]:
        c.limb([(0.385, 0.73), (ex, ey)], 0.012, (150, 116, 78, 255))
        c.ell(ex, ey, 0.011, 0.011, DARK)
    c.ell(0.56, 0.665, 0.105, 0.10, (128, 92, 58, 255), DARK, 0.014)
    c.arc(0.56, 0.665, 0.06, 0.055, 300, 200, (92, 64, 36, 255), 0.014)
    c.arc(0.56, 0.665, 0.032, 0.03, 300, 200, (92, 64, 36, 255), 0.012)


# ---------------------------------------------------------------- solar beetles
def _beetle(c, starving):
    shell = (96, 164, 224, 255)
    shell_d = (64, 120, 180, 255)
    panel = (28, 62, 128, 255) if not starving else (52, 78, 118, 255)
    panel_l = (96, 164, 232, 255) if not starving else (110, 130, 160, 255)
    tilt = 0.03 if starving else 0.0
    # legs
    for (lx, ly, ex, ey) in [(0.34, 0.62, 0.20, 0.78), (0.36, 0.72, 0.24, 0.88),
                             (0.64, 0.62, 0.80, 0.78), (0.62, 0.72, 0.76, 0.88),
                             (0.30, 0.52, 0.16, 0.58), (0.70, 0.52, 0.84, 0.58)]:
        c.limb([(lx, ly), (ex, ey)], 0.026, (40, 50, 70, 255))
    # open elytra covered with curved solar panels
    droop = 0.10 if starving else 0.0
    left = [(0.47, 0.30), (0.16, 0.16 + droop), (0.13, 0.44 + droop), (0.44, 0.52)]
    right = [(0.53, 0.30), (0.84, 0.16), (0.87, 0.44), (0.56, 0.52)]
    for wing in (left, right):
        wing = [(x, y + tilt if x > 0.5 else y) for (x, y) in wing]
        c.poly(wing, panel, DARK, 0.012)
    for wing, sgn in ((left, -1), (right, 1)):
        cx0 = 0.47 if sgn < 0 else 0.53
        for k in range(3):
            f = 0.25 + k * 0.22
            x = cx0 + sgn * f * 0.62
            c.line([(x, 0.30 - f * 0.10 + (droop if sgn < 0 else 0)),
                    (x + sgn * 0.02, 0.50 - f * 0.06 + (droop if sgn < 0 else 0))], panel_l, 0.010)
        c.line([(cx0 + sgn * 0.06, 0.375 + (droop if sgn < 0 else 0)),
                (cx0 + sgn * 0.30, 0.30 + (droop if sgn < 0 else 0))], panel_l, 0.010)
    # body and head
    c.ell(0.5, 0.52, 0.13, 0.22, shell, DARK, 0.012)
    c.ell(0.5, 0.76, 0.085, 0.075, shell_d, DARK, 0.012)
    c.limb([(0.45, 0.81), (0.36, 0.90)], 0.014, (40, 50, 70, 255))
    c.limb([(0.55, 0.81), (0.64, 0.90)], 0.014, (40, 50, 70, 255))
    c.ell(0.465, 0.745, 0.016, 0.016, DARK)
    c.ell(0.535, 0.745, 0.016, 0.016, DARK)
    if starving:
        for _ in range(7):
            x, y = c.rng.uniform(0.2, 0.8), c.rng.uniform(0.18, 0.5)
            c.line([(x, y), (x + c.rng.uniform(0.03, 0.09), y + c.rng.uniform(-0.01, 0.03))],
                   (210, 226, 240, 255), 0.008)
    else:
        for (x, y) in [(0.40, 0.42), (0.58, 0.36), (0.50, 0.55)]:
            c.poly([(x, y - 0.022), (x + 0.013, y), (x, y + 0.022), (x - 0.013, y)],
                   (240, 250, 255, 255))


def beetle_solar(c):
    _beetle(c, False)


def beetle_solar_starving(c):
    _beetle(c, True)


# ---------------------------------------------------------------- biollante
def _biollante(c, tall, buds):
    stalk = (86, 150, 74, 255)
    stalk_d = (58, 110, 50, 255)
    leaf = (64, 128, 58, 255)
    top_y = 0.30 if tall else 0.52
    base_y = 0.90
    w0 = 0.055 if tall else 0.075
    spine = c.curve((0.5, base_y), (0.47, (top_y + base_y) / 2), (0.5, top_y), 7)
    c.taper(spine, w0 * 2, w0 * 1.1, stalk, stalk_d, 0.010)
    # broad leaves at the base
    for (lx, ly, sgn) in [(0.30, 0.82, -1), (0.70, 0.82, 1), (0.38, 0.88, -1), (0.62, 0.88, 1)]:
        leaf_pts = [(0.5, ly + 0.03), (lx + sgn * 0.10, ly - 0.06), (lx - sgn * 0.06, ly + 0.015),
                    (lx + sgn * 0.05, ly + 0.06)]
        c.poly(leaf_pts, leaf, stalk_d, 0.010)
    bud_cols = [(216, 90, 180, 255), (168, 84, 204, 255), (226, 120, 200, 255)]
    for i in range(buds):
        a = -0.9 + i * (1.8 / max(1, buds - 1)) if buds > 1 else 0
        bx = 0.5 + math.sin(a) * 0.075
        by = top_y - 0.03 - abs(math.cos(a)) * 0.015 + (0.01 if abs(a) > 0.6 else -0.015)
        r = 0.036 if tall else 0.028
        c.ell(bx, by, r, r * 1.2, bud_cols[i % 3], stalk_d, 0.009)
        c.ell(bx - r * 0.3, by - r * 0.35, r * 0.3, r * 0.3, (246, 190, 235, 255))


def biollante_sprig(c):
    _biollante(c, False, 3)


def biollante_sprout(c):
    _biollante(c, True, 5)


# ---------------------------------------------------------------- linelanders
def _linelander(c, core, pale):
    # a vibration: a bright filament that always faces you, space oscillating
    for k, (off, amp, col, wdt) in enumerate([(-0.13, 0.030, pale, 0.012),
                                              (0.13, 0.030, pale, 0.012),
                                              (-0.065, 0.045, pale, 0.018),
                                              (0.065, 0.045, pale, 0.018),
                                              (0.0, 0.060, core, 0.030)]):
        pts = []
        for i in range(25):
            t = i / 24
            y = 0.13 + t * 0.74
            x = 0.5 + off + math.sin(t * 12.0 + k * 1.7) * amp * (0.4 + 0.6 * math.sin(t * math.pi))
            pts.append((x, y))
        c.line(pts, col, wdt)
    c.ell(0.5, 0.5, 0.030, 0.045, core)
    for _ in range(14):
        x = 0.5 + c.rng.uniform(-0.34, 0.34)
        y = c.rng.uniform(0.16, 0.86)
        ln = c.rng.uniform(0.03, 0.08)
        dy = c.rng.uniform(-0.02, 0.02)
        c.line([(x, y), (x + ln, y + dy)], pale, 0.010)


def linelander_cyan(c):
    _linelander(c, (110, 235, 245, 255), (170, 200, 205, 255))


def linelander_magenta(c):
    _linelander(c, (244, 110, 210, 255), (205, 165, 195, 255))


def linelander_yellow(c):
    _linelander(c, (250, 214, 60, 255), (208, 190, 140, 255))


# ---------------------------------------------------------------- singularity reclaimers
def _reclaimer(c, big):
    base = (178, 70, 58, 255)
    dark = (126, 42, 38, 255)
    light = (212, 112, 94, 255)
    rx = 0.36 if big else 0.32
    ry = 0.40 if big else 0.35
    cy = 0.56 if big else 0.58
    # appendages reaching toward the ground (and its corpses) first
    reaches = [(-0.30, 0.93), (-0.12, 0.97), (0.14, 0.97), (0.31, 0.92)] if big else \
              [(-0.26, 0.94), (0.0, 0.97), (0.27, 0.93)]
    for (ex, ey) in reaches:
        sx = 0.5 + ex * 0.55
        spine = c.curve((sx, cy + ry * 0.45), ((sx + 0.5 + ex) / 2, ey - 0.10), (0.5 + ex, ey), 7)
        c.taper(spine, 0.075 if big else 0.06, 0.018, base, DARK, 0.008)
    c.blob(0.5, cy, rx, ry, base, DARK, n=15, jit=0.12)
    # writhing surface bumps
    for _ in range(10 if big else 7):
        a = c.rng.uniform(0, 2 * math.pi)
        bx = 0.5 + math.cos(a) * rx * 0.78
        by = cy + math.sin(a) * ry * 0.78
        c.ell(bx, by, c.rng.uniform(0.035, 0.065), c.rng.uniform(0.03, 0.055), light, None)
    # vertical stretch streaks, as if torn apart by invisible hands
    for k in range(6 if big else 4):
        x = 0.5 + (k - (5 if big else 3) / 2) * 0.085 + c.rng.uniform(-0.015, 0.015)
        pts = [(x + math.sin(t * 9 + k) * 0.012, cy - ry * 0.8 + t * ry * 1.5) for t in
               (i / 10 for i in range(11))]
        c.line(pts, dark, 0.014)
    c.blob(0.5, cy - ry * 0.35, rx * 0.4, ry * 0.3, light, None, n=9, jit=0.2)


def singularity_reclaimer(c):
    _reclaimer(c, True)


def singularity_reclaimer_spawn(c):
    _reclaimer(c, False)


# ---------------------------------------------------------------- robots
def _talon(c, body_col, dark_col, flag):
    track = (38, 40, 46, 255)
    gun = (28, 28, 32, 255)
    # tracks
    for tx in (0.20, 0.62):
        c.rect(tx, 0.70, tx + 0.20, 0.86, track, DARK, 0.010)
        for k in range(3):
            c.ell(tx + 0.035 + k * 0.065, 0.78, 0.024, 0.024, (86, 90, 98, 255))
    # chassis
    c.rect(0.24, 0.52, 0.78, 0.72, body_col, DARK, 0.011)
    c.rect(0.28, 0.56, 0.46, 0.68, dark_col)
    if flag:
        # American flag decal: stripes and a blue canton
        c.rect(0.56, 0.565, 0.74, 0.665, (238, 238, 238, 255), DARK, 0.006)
        for k in range(3):
            c.rect(0.56, 0.578 + k * 0.028, 0.74, 0.592 + k * 0.028, (196, 48, 52, 255))
        c.rect(0.56, 0.565, 0.635, 0.615, (44, 56, 128, 255))
        for (dx, dy) in [(0.575, 0.578), (0.60, 0.578), (0.575, 0.598), (0.60, 0.598), (0.618, 0.588)]:
            c.ell(dx, dy, 0.006, 0.006, (255, 255, 255, 255))
    # weapon mount with sensor array and an M16A4
    c.rect(0.42, 0.40, 0.58, 0.52, dark_col, DARK, 0.009)
    c.rect(0.30, 0.335, 0.80, 0.375, gun)                    # barrel + handguard
    c.rect(0.60, 0.375, 0.70, 0.44, gun)                     # receiver
    c.rect(0.635, 0.44, 0.665, 0.50, gun)                    # magazine
    c.rect(0.315, 0.315, 0.345, 0.335, gun)                  # front sight
    c.rect(0.44, 0.30, 0.56, 0.345, (52, 56, 64, 255), DARK, 0.007)  # sensor box
    c.ell(0.475, 0.322, 0.013, 0.013, (96, 205, 235, 255))
    c.ell(0.525, 0.322, 0.013, 0.013, (96, 205, 235, 255))
    c.limb([(0.70, 0.44), (0.78, 0.30)], 0.010, gun)         # antenna


def secubot_america(c):
    _talon(c, (168, 172, 178, 255), (120, 124, 132, 255), True)


def secubot_balthazar(c):
    _talon(c, (86, 90, 98, 255), (56, 60, 68, 255), False)
    c.ell(0.50, 0.322, 0.011, 0.011, (235, 120, 96, 255))    # third sensor eye


def turret_rifle_balthazar(c):
    green = (86, 120, 70, 255)
    green_d = (58, 86, 46, 255)
    gun = (28, 28, 32, 255)
    c.poly([(0.30, 0.92), (0.70, 0.92), (0.60, 0.62), (0.40, 0.62)], green_d, DARK, 0.012)
    c.rect(0.36, 0.52, 0.64, 0.64, green, DARK, 0.011)
    # M249 on the CROWS mount
    c.rect(0.16, 0.40, 0.86, 0.445, gun)                     # barrel + receiver line
    c.rect(0.52, 0.445, 0.72, 0.52, gun)                     # receiver
    c.rect(0.56, 0.52, 0.68, 0.62, (40, 40, 44, 255), DARK, 0.008)   # ammo box
    c.rect(0.20, 0.445, 0.26, 0.52, gun)                     # bipod hint
    for k in range(4):
        c.ell(0.34 + k * 0.045, 0.422, 0.007, 0.007, (90, 90, 96, 255))
    c.rect(0.40, 0.30, 0.58, 0.40, green, DARK, 0.010)       # sensor head
    c.ell(0.455, 0.35, 0.016, 0.016, (96, 205, 235, 255))
    c.ell(0.535, 0.35, 0.016, 0.016, (250, 220, 120, 255))
    c.limb([(0.64, 0.52), (0.74, 0.36)], 0.010, gun)


def robofac_hound(c):
    metal = (150, 200, 150, 255)
    metal_d = (96, 132, 96, 255)
    joint = (70, 78, 74, 255)
    # legs with knee joints
    for (hx, kx, fx) in [(0.30, 0.24, 0.26), (0.42, 0.40, 0.38), (0.66, 0.72, 0.70), (0.78, 0.86, 0.88)]:
        c.limb([(hx, 0.52), (kx, 0.70), (fx, 0.88)], 0.038, metal_d)
        c.ell(kx, 0.70, 0.026, 0.026, joint)
        c.ell(fx, 0.89, 0.030, 0.013, joint)
    # angular body
    c.poly([(0.22, 0.30), (0.62, 0.24), (0.80, 0.34), (0.78, 0.52), (0.30, 0.56), (0.20, 0.44)],
           metal, DARK, 0.011)
    c.poly([(0.30, 0.33), (0.58, 0.28), (0.70, 0.36), (0.34, 0.44)], (178, 222, 178, 255))
    # speaker grille: the noise generator
    c.ell(0.46, 0.44, 0.055, 0.05, metal_d, DARK, 0.008)
    for (dx, dy) in [(-0.025, -0.02), (0.025, -0.02), (0, 0), (-0.025, 0.02), (0.025, 0.02)]:
        c.ell(0.46 + dx, 0.44 + dy, 0.008, 0.008, DARK)
    # head with sensor eye and taser prongs
    c.poly([(0.76, 0.30), (0.94, 0.36), (0.93, 0.48), (0.78, 0.50)], metal, DARK, 0.010)
    c.ell(0.865, 0.395, 0.017, 0.017, (96, 225, 245, 255), DARK, 0.006)
    c.line([(0.935, 0.42), (0.975, 0.40)], joint, 0.012)
    c.line([(0.935, 0.46), (0.975, 0.45)], joint, 0.012)
    c.poly([(0.978, 0.375), (0.965, 0.415), (0.985, 0.42), (0.972, 0.455)], (250, 230, 90, 255))
    c.limb([(0.24, 0.32), (0.13, 0.24)], 0.024, metal_d)    # tail/antenna stub


# ---------------------------------------------------------------- singles
def flailer(c):
    cloth = (238, 236, 230, 255)
    seam = (168, 166, 158, 255)
    # a cloth facsimile of a bird, wings askew mid-flail
    c.poly([(0.50, 0.42), (0.16, 0.18), (0.24, 0.50), (0.46, 0.56)], cloth, seam, 0.012)
    c.poly([(0.54, 0.42), (0.86, 0.62), (0.72, 0.72), (0.54, 0.56)], cloth, seam, 0.012)
    c.ell(0.5, 0.52, 0.17, 0.13, cloth, seam, 0.012)
    c.ell(0.63, 0.40, 0.085, 0.075, cloth, seam, 0.012)
    c.poly([(0.70, 0.40), (0.80, 0.435), (0.70, 0.465)], (206, 188, 160, 255), seam, 0.009)
    c.ell(0.645, 0.385, 0.013, 0.013, DARK)
    c.poly([(0.36, 0.58), (0.24, 0.74), (0.34, 0.76), (0.42, 0.64), (0.48, 0.78),
            (0.54, 0.64), (0.60, 0.74), (0.66, 0.60)], cloth, seam, 0.010)
    # stitch dashes and crosses
    for t in range(5):
        x = 0.40 + t * 0.05
        c.line([(x, 0.475), (x + 0.022, 0.475)], seam, 0.010)
    for (x, y) in [(0.30, 0.32), (0.70, 0.62), (0.48, 0.60)]:
        c.line([(x - 0.016, y - 0.016), (x + 0.016, y + 0.016)], seam, 0.009)
        c.line([(x - 0.016, y + 0.016), (x + 0.016, y - 0.016)], seam, 0.009)
    c.limb([(0.47, 0.64), (0.45, 0.76)], 0.010, seam)
    c.limb([(0.55, 0.64), (0.58, 0.76)], 0.010, seam)


def cabin_tunnel_shadow(c):
    pale = (208, 210, 216, 255)
    mid = (172, 174, 184, 255)
    edge = (128, 130, 142, 255)
    # an arm reaching forward on the ground
    c.limb([(0.34, 0.72), (0.20, 0.82), (0.10, 0.86)], 0.055, mid)
    for f in (-0.025, 0.0, 0.025):
        c.line([(0.10, 0.86), (0.055, 0.885 + f)], pale, 0.011)
    # the long cloaked body, crawling low
    c.blob(0.52, 0.68, 0.36, 0.20, pale, edge, n=12, jit=0.08)
    c.blob(0.60, 0.62, 0.26, 0.13, (222, 224, 230, 255), None, n=10, jit=0.1)
    # hood with a featureless blank face
    c.ell(0.26, 0.55, 0.115, 0.125, mid, edge, 0.012)
    c.ell(0.245, 0.56, 0.072, 0.082, (232, 233, 238, 255))
    # fold streaks suggesting translucency
    for k in range(4):
        x = 0.42 + k * 0.11
        c.line([(x, 0.56), (x + 0.05, 0.82)], (226, 228, 234, 255), 0.012)
    c.ell(0.86, 0.80, 0.05, 0.035, mid)


def inner_cabins_horror(c):
    skin = (224, 196, 178, 255)
    skin_d = (188, 148, 132, 255)
    wash = (198, 96, 84, 255)
    # contorted limbs at wrong angles, on all fours
    c.limb([(0.36, 0.52), (0.22, 0.44), (0.16, 0.66), (0.20, 0.86)], 0.052, skin_d)
    c.limb([(0.44, 0.58), (0.36, 0.76), (0.42, 0.90)], 0.052, skin)
    c.limb([(0.62, 0.52), (0.78, 0.46), (0.84, 0.68), (0.78, 0.88)], 0.052, skin_d)
    c.limb([(0.58, 0.58), (0.66, 0.78), (0.58, 0.92)], 0.052, skin)
    for (fx, fy) in [(0.20, 0.86), (0.42, 0.90), (0.78, 0.88), (0.58, 0.92)]:
        for f in (-0.03, 0.0, 0.03):
            c.line([(fx, fy), (fx + f * 1.4, fy + 0.035)], skin_d, 0.010)
    # torso twisted diagonally
    c.poly([(0.34, 0.38), (0.62, 0.30), (0.70, 0.50), (0.44, 0.62), (0.30, 0.54)], skin, DARK, 0.012)
    c.blob(0.55, 0.40, 0.09, 0.06, wash, None, n=8, jit=0.25)
    # head wrenched around, pale darting eyes
    c.ell(0.30, 0.30, 0.095, 0.085, skin, DARK, 0.012)
    c.ell(0.265, 0.285, 0.020, 0.024, (246, 246, 240, 255))
    c.ell(0.335, 0.285, 0.020, 0.024, (246, 246, 240, 255))
    c.ell(0.272, 0.29, 0.008, 0.010, DARK)
    c.ell(0.328, 0.29, 0.008, 0.010, DARK)
    c.ell(0.30, 0.345, 0.026, 0.012, (90, 40, 38, 255))


def black_ratkin(c):
    fur = (50, 50, 57, 255)
    fur_l = (84, 84, 92, 255)
    # long rough tail, segmented
    tail = c.curve((0.72, 0.66), (0.90, 0.60), (0.94, 0.38), 9)
    c.taper(tail, 0.035, 0.012, (148, 108, 108, 255), None)
    for t in (0.3, 0.55, 0.8):
        p = tail[int(t * 9)]
        c.ell(p[0], p[1], 0.020, 0.012, (120, 86, 86, 255))
    c.ell(0.46, 0.64, 0.27, 0.135, fur, DARK, 0.013)
    c.ell(0.40, 0.585, 0.17, 0.06, fur_l)
    # head and snout
    c.poly([(0.24, 0.56), (0.10, 0.63), (0.13, 0.70), (0.27, 0.71)], fur, DARK, 0.011)
    c.ell(0.115, 0.665, 0.016, 0.013, (196, 130, 130, 255))
    c.ell(0.235, 0.585, 0.020, 0.020, (240, 240, 240, 255))
    c.ell(0.238, 0.588, 0.010, 0.010, DARK)
    for (ex, ey) in [(0.27, 0.52), (0.33, 0.535)]:
        c.ell(ex, ey, 0.030, 0.034, fur, DARK, 0.009)
        c.ell(ex, ey, 0.016, 0.019, (150, 110, 112, 255))
    for wy in (0.635, 0.66, 0.685):
        c.line([(0.115, wy), (0.03, wy - 0.012)], (200, 200, 200, 255), 0.007)
    for fx in (0.30, 0.58):
        c.ell(fx, 0.775, 0.035, 0.013, (120, 92, 92, 255))


def blob_sewer(c):
    glob = (34, 52, 34, 255)
    glob_l = (88, 128, 78, 255)
    c.blob(0.5, 0.62, 0.36, 0.27, glob, DARK, n=13, jit=0.15)
    c.blob(0.38, 0.47, 0.15, 0.09, glob_l, None, n=9, jit=0.2)
    c.blob(0.63, 0.55, 0.09, 0.06, (60, 88, 56, 255), None, n=8, jit=0.2)
    # incorporated sewer contents
    c.poly([(0.55, 0.68), (0.63, 0.65), (0.65, 0.72), (0.57, 0.75)], (196, 190, 170, 255))
    c.rect(0.30, 0.60, 0.375, 0.665, (130, 134, 138, 255), DARK, 0.007)
    c.ell(0.44, 0.76, 0.035, 0.02, (104, 76, 44, 255))
    # a drip tendril reaching forward, and stench wisps
    spine = c.curve((0.24, 0.78), (0.14, 0.84), (0.07, 0.80), 6)
    c.taper(spine, 0.05, 0.015, glob, None)
    for (wx, wy) in [(0.36, 0.22), (0.58, 0.16)]:
        c.arc(wx, wy, 0.045, 0.06, 200, 480, (140, 180, 120, 255), 0.012)


def mutant_camouflaged(c):
    fur_a = (110, 130, 170, 255)
    fur_b = (142, 142, 140, 255)
    fur_c = (98, 112, 92, 255)
    skin = (196, 168, 150, 255)
    # long thin legs mid-stride, ending in talons
    legs = [((0.30, 0.52), (0.20, 0.68), (0.13, 0.86)), ((0.40, 0.56), (0.44, 0.72), (0.50, 0.88)),
            ((0.66, 0.52), (0.76, 0.66), (0.86, 0.82)), ((0.74, 0.56), (0.70, 0.74), (0.64, 0.90))]
    for pts in legs:
        c.limb(list(pts), 0.034, skin)
        fx, fy = pts[-1]
        for f in (-0.028, 0.0, 0.028):
            c.poly([(fx, fy), (fx + f * 1.6, fy + 0.012), (fx + f * 0.8, fy + 0.045)], BONE)
    # lean body in camouflage patches
    c.poly([(0.24, 0.36), (0.52, 0.28), (0.78, 0.34), (0.80, 0.50), (0.52, 0.58), (0.28, 0.54)],
           fur_b, DARK, 0.011)
    c.poly([(0.24, 0.36), (0.44, 0.30), (0.46, 0.50), (0.28, 0.54)], fur_a)
    c.poly([(0.60, 0.30), (0.78, 0.34), (0.80, 0.50), (0.62, 0.54)], fur_c)
    c.poly([(0.46, 0.38), (0.60, 0.36), (0.62, 0.50), (0.48, 0.52)], fur_a)
    # neck and head, mouth held open by knife-sized fangs
    c.poly([(0.76, 0.36), (0.90, 0.30), (0.95, 0.40), (0.82, 0.50), (0.74, 0.46)], skin, DARK, 0.010)
    c.poly([(0.86, 0.415), (0.945, 0.405), (0.90, 0.475), (0.845, 0.465)], (60, 26, 26, 255))
    for i, tx in enumerate((0.865, 0.895, 0.925)):
        c.poly([(tx, 0.412), (tx + 0.014, 0.412), (tx + 0.007, 0.455)], BONE)
        c.poly([(tx, 0.472), (tx + 0.014, 0.472), (tx + 0.007, 0.435)], BONE)
    c.ell(0.815, 0.365, 0.015, 0.015, DARK)
    c.limb([(0.26, 0.40), (0.14, 0.34), (0.06, 0.38)], 0.020, fur_c)


def poly_special_test(c):
    red = (196, 52, 40, 255)
    red_d = (138, 30, 26, 255)
    memb = (172, 62, 50, 255)
    belly = (234, 182, 122, 255)
    # tail curling at the base
    spine = c.curve((0.55, 0.80), (0.80, 0.92), (0.95, 0.72), 8)
    c.taper(spine, 0.085, 0.02, red_d, DARK, 0.009)
    c.poly([(0.95, 0.72), (0.90, 0.60), (0.99, 0.62)], red_d, DARK, 0.008)
    # far wing
    c.poly([(0.55, 0.30), (0.92, 0.08), (0.88, 0.38), (0.66, 0.40)], memb, DARK, 0.010)
    # body
    c.ell(0.46, 0.62, 0.20, 0.24, red, DARK, 0.011)
    for k in range(4):
        c.arc(0.46, 0.52 + k * 0.075, 0.115 - k * 0.012, 0.075, 20, 160, belly, 0.016)
    for _ in range(12):
        x, y = c.rng.uniform(0.32, 0.60), c.rng.uniform(0.44, 0.72)
        c.arc(x, y, 0.026, 0.02, 180, 360, red_d, 0.010)
    # legs with claws
    for (lx, fx) in [(0.36, 0.30), (0.56, 0.62)]:
        c.limb([(lx, 0.78), (fx, 0.92)], 0.055, red_d)
        for f in (-0.03, 0.0, 0.03):
            c.poly([(fx, 0.93), (fx + f * 1.5, 0.945), (fx + f * 0.7, 0.975)], BONE)
    # neck rearing up to the head
    spine = c.curve((0.42, 0.48), (0.30, 0.34), (0.26, 0.22), 7)
    c.taper(spine, 0.115, 0.075, red, DARK, 0.009)
    # head: curving horns, glowing maw in a hateful grimace
    c.poly([(0.265, 0.13), (0.13, 0.05), (0.155, 0.155), (0.22, 0.185)], BONE, DARK, 0.008)
    c.poly([(0.315, 0.125), (0.235, 0.03), (0.285, 0.16), (0.30, 0.185)], BONE, DARK, 0.008)
    c.ell(0.27, 0.185, 0.085, 0.070, red, DARK, 0.010)
    c.poly([(0.20, 0.215), (0.335, 0.205), (0.315, 0.275), (0.225, 0.285)], (255, 158, 40, 255), DARK, 0.008)
    c.poly([(0.225, 0.25), (0.31, 0.245), (0.295, 0.275), (0.24, 0.278)], (255, 196, 70, 255))
    for i, tx in enumerate((0.225, 0.255, 0.285, 0.308)):
        c.poly([(tx, 0.212), (tx + 0.014, 0.212), (tx + 0.007, 0.242)], BONE)
    c.ell(0.245, 0.175, 0.014, 0.014, (255, 224, 90, 255), DARK, 0.005)
    c.ell(0.30, 0.172, 0.014, 0.014, (255, 224, 90, 255), DARK, 0.005)
    c.limb([(0.225, 0.16), (0.255, 0.168)], 0.012, red_d)
    c.limb([(0.295, 0.158), (0.325, 0.162)], 0.012, red_d)
    # near wing, with a small checkerboard calibration patch at its tip
    c.poly([(0.44, 0.42), (0.16, 0.14), (0.10, 0.44), (0.30, 0.50)], memb, DARK, 0.010)
    c.line([(0.44, 0.42), (0.16, 0.14)], red_d, 0.012)
    px, py, cell = 0.145, 0.175, 0.021
    for i in range(3):
        for j in range(3):
            col = (240, 240, 240, 255) if (i + j) % 2 == 0 else (30, 30, 34, 255)
            c.rect(px + i * cell, py + j * cell, px + (i + 1) * cell, py + (j + 1) * cell, col)


def wasp_queen_fungus(c):
    yellow = (240, 200, 60, 255)
    mold = (156, 158, 152, 255)
    mold_d = (122, 124, 118, 255)
    # wings first (pale, opaque)
    c.ell(0.40, 0.30, 0.16, 0.065, (222, 228, 232, 255), mold_d, 0.009)
    c.ell(0.62, 0.28, 0.15, 0.06, (222, 228, 232, 255), mold_d, 0.009)
    # dangling legs
    for (lx, ex) in [(0.42, 0.34), (0.50, 0.48), (0.58, 0.64)]:
        c.limb([(lx, 0.58), (ex, 0.78)], 0.016, DARK)
    # banded abdomen, largely consumed by gray mold
    c.ell(0.50, 0.66, 0.155, 0.19, yellow, DARK, 0.012)
    for k, yy in enumerate((0.575, 0.65, 0.725)):
        c.arc(0.50, yy, 0.145 - k * 0.018, 0.045, 15, 165, DARK, 0.020)
    c.poly([(0.465, 0.83), (0.535, 0.83), (0.50, 0.92)], DARK)
    # thorax and head
    c.ell(0.50, 0.44, 0.105, 0.09, yellow, DARK, 0.011)
    c.ell(0.50, 0.315, 0.075, 0.06, yellow, DARK, 0.011)
    c.ell(0.472, 0.305, 0.020, 0.024, DARK)
    c.ell(0.528, 0.305, 0.020, 0.024, DARK)
    c.limb([(0.465, 0.265), (0.42, 0.20)], 0.010, DARK)
    c.limb([(0.535, 0.265), (0.58, 0.20)], 0.010, DARK)
    # mandibles
    c.poly([(0.475, 0.365), (0.462, 0.395), (0.488, 0.385)], DARK)
    c.poly([(0.525, 0.365), (0.538, 0.395), (0.512, 0.385)], DARK)
    # the sickly gray mold consuming her
    for (mx, my, mr) in [(0.42, 0.60, 0.075), (0.60, 0.70, 0.085), (0.47, 0.78, 0.06),
                         (0.55, 0.42, 0.055), (0.44, 0.33, 0.045), (0.63, 0.55, 0.05)]:
        c.blob(mx, my, mr, mr * 0.8, mold, mold_d, n=8, jit=0.3)
    # spore mist drifting from the mandibles
    for _ in range(9):
        x, y = 0.5 + c.rng.uniform(-0.10, 0.10), c.rng.uniform(0.38, 0.52)
        c.ell(x, y, 0.011, 0.011, (206, 208, 200, 255))


def whisper_fungus(c):
    yel = (236, 196, 84, 255)
    yel_d = (198, 156, 56, 255)
    hole = (118, 86, 28, 255)
    c.ell(0.5, 0.90, 0.30, 0.055, yel_d, DARK, 0.010)
    # coral-like branching spires riddled with holes
    spires = [(0.30, 0.88, 0.26, 0.34), (0.50, 0.90, 0.52, 0.18), (0.70, 0.88, 0.74, 0.30),
              (0.40, 0.88, 0.38, 0.26), (0.62, 0.90, 0.60, 0.24)]
    for (bx, by, tx, ty) in spires:
        spine = c.curve((bx, by), ((bx + tx) / 2 + 0.02, (by + ty) / 2), (tx, ty), 7)
        c.taper(spine, 0.075, 0.035, yel, yel_d, 0.009)
        for (dx, dy, ex, ey) in [(-0.07, -0.10, -0.13, -0.20), (0.07, -0.12, 0.13, -0.22)]:
            bspine = c.curve((tx, ty + 0.10), (tx + dx, ty + dy), (tx + ex, ty + ey), 5)
            c.taper(bspine, 0.035, 0.012, yel, yel_d, 0.008)
        for t in (0.35, 0.6, 0.85):
            p = spine[int(t * 7)]
            c.ell(p[0], p[1], 0.013, 0.013, hole)
        c.ell(tx, ty, 0.020, 0.020, hole)
    # spores pulsing out on a rhythm
    for _ in range(12):
        x, y = c.rng.uniform(0.16, 0.84), c.rng.uniform(0.04, 0.22)
        c.ell(x, y, 0.010, 0.010, (246, 230, 170, 255))


# ---------------------------------------------------------------- table
# (id, sheet, resolved volume L, renderer) -- volumes resolved through
# copy-from chains in fork-src/cdda/data/json/monsters/*.json.
SPRITES = [
    ("mon_yrax_apeirogon", 32, 50.0, yrax_apeirogon),
    ("mon_yrax_delta", 32, 0.5, yrax_delta),
    ("mon_yrax_triakis", 32, 50.0, yrax_triakis),
    ("mon_yrax_trifacet", 32, 50.0, yrax_trifacet),
    ("mon_yrax_quadraphract", 96, 930.0, yrax_quadraphract),
    ("mon_yrax_quadraphract_leg", 96, 930.0, yrax_quadraphract_leg),
    ("mon_yrax_sphenocorona", 96, 930.0, yrax_sphenocorona),
    ("mon_golden_monolith", 96, 930.0, golden_monolith),
    ("mon_devourer_daitya", 64, 187.5, devourer_daitya),
    ("mon_devourer_daitya_clone", 32, 62.5, devourer_daitya_clone),
    ("mon_devourer_hekatonkheire", 64, 287.5, devourer_hekatonkheire),
    ("mon_devourer_lab_sec", 96, 483.0, devourer_lab_sec),
    ("mon_kraken_guard", 96, 450.0, kraken_guard),
    ("mon_kraken_queen", 96, 950.0, kraken_queen),
    ("mon_octupus_stalker", 64, 108.5, octupus_stalker),
    ("mon_octupus_stalker_spawn", 32, 45.0, octupus_stalker_spawn),
    ("mon_ocean_zhark", 64, 200.0, ocean_zhark),
    ("mon_snail_giant", 96, 875.0, snail_giant),
    ("mon_snail_river", 32, 0.3, snail_river),
    ("mon_snail_small", 32, 62.5, snail_small),
    ("mon_beetle_solar_reverberation", 32, 62.5, beetle_solar),
    ("mon_beetle_solar_starving_reverberation", 32, 62.5, beetle_solar_starving),
    ("mon_biollante_sprig", 32, 30.0, biollante_sprig),
    ("mon_biollante_sprout", 32, 62.5, biollante_sprout),
    ("mon_linelander_cyan", 64, 122.5, linelander_cyan),
    ("mon_linelander_magenta", 64, 122.5, linelander_magenta),
    ("mon_linelander_yellow", 64, 122.5, linelander_yellow),
    ("mon_singularity_reclaimer", 96, 950.0, singularity_reclaimer),
    ("mon_singularity_reclaimer_spawn", 64, 187.5, singularity_reclaimer_spawn),
    ("mon_secubot_america_refurbished", 64, 150.0, secubot_america),
    ("mon_secubot_balthazar", 64, 150.0, secubot_balthazar),
    ("mon_turret_rifle_balthazar", 32, 60.0, turret_rifle_balthazar),
    ("mon_robofac_hound", 64, 108.5, robofac_hound),
    ("mon_flailer_radiosphere", 32, 0.15, flailer),
    ("mon_cabin_tunnel_shadow", 64, 120.0, cabin_tunnel_shadow),
    ("mon_inner_cabins_interstice_horror", 32, 62.5, inner_cabins_horror),
    ("mon_black_ratkin", 32, 0.2, black_ratkin),
    ("mon_blob_sewer", 32, 62.5, blob_sewer),
    ("mon_mutant_camouflaged", 64, 200.0, mutant_camouflaged),
    ("mon_poly_special_test", 96, 875.0, poly_special_test),
    ("mon_wasp_queen_fungus", 32, 10.0, wasp_queen_fungus),
    ("mon_whisper_fungus", 32, 20.0, whisper_fungus),
]

SHEET_DIR = {32: "pngs_normal_32x32", 64: "pngs_large_64x64", 96: "pngs_giant_96x96"}


def main():
    assert len(SPRITES) == 42, len(SPRITES)
    rendered = {}
    for mid, size, _vol, fn in SPRITES:
        ctx = Ctx(size, mid)
        fn(ctx)
        alphas = set(ctx.img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (mid, sorted(alphas)[:8])
        folder = os.path.join(BASE, SHEET_DIR[size], "monsters", mid)
        os.makedirs(folder, exist_ok=True)
        png_path = os.path.join(folder, mid + ".png")
        ctx.img.save(png_path)
        with open(os.path.join(folder, mid + ".json"), "w") as f:
            json.dump([{"id": mid, "fg": mid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        rendered[mid] = ctx.img
        print("wrote", os.path.relpath(png_path, BASE), ctx.img.size)
    # contact sheet: 7 x 6 cells of 96px, sprites fitted with NEAREST
    cell, cols = 96, 7
    rows = (len(SPRITES) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (38, 40, 50, 255))
    d = ImageDraw.Draw(sheet)
    for i, (mid, size, _vol, _fn) in enumerate(SPRITES):
        img = rendered[mid]
        if img.size != (cell, cell):
            img = img.resize((cell, cell), Image.NEAREST)
        x, y = (i % cols) * cell, (i // cols) * cell
        sheet.alpha_composite(img, (x, y))
        d.rectangle([x, y, x + cell - 1, y + cell - 1], outline=(90, 94, 110, 255))
    out = os.path.join(BASE, "preview_finisher_cosmic.png")
    sheet.save(out)
    print("wrote", os.path.relpath(out, BASE), sheet.size)


if __name__ == "__main__":
    main()
