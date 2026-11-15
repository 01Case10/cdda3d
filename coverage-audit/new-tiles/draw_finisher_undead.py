#!/usr/bin/env python3
"""
Draw the finisher-wave undead & lab-tail monster sprites from their game
descriptions. Batch 24 of the Phase 2g.4 coverage drive (42 ids).

Sources (fork-src/cdda/data/json/monsters/*.json, CDDA 6a3e59e0):
zed_skeletal.json, zed_command.json, zanimal_upgrade.json, zed_burned.json,
zed_nomex.json, zed_soldiers.json, zed_survivor.json, zed_children.json,
scarlet_sea.json, cabins_inner.json, string_dimension.json, cyborgs.json,
nether_animal.json, ghouls.json, zed-pupating.json, misc.json.

Design: flat UltiCa style -- dark outlines, flat fills, one highlight
pass. Bodies are fully opaque (alpha only {0,255}); translucency
(incorporeal / antimemetic / void / smoke) is faked with pale or dark
opaque tones. Sheet placement follows the standing volume rule, volume
resolved through each entry's copy-from chain:
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

    def speckles(self, cx, cy, rx, ry, count, colors, rmin=0.006, rmax=0.016):
        for _ in range(count):
            a = self.rng.uniform(0, 2 * math.pi)
            rr = math.sqrt(self.rng.uniform(0, 1))
            x, y = cx + math.cos(a) * rx * rr, cy + math.sin(a) * ry * rr
            r = self.rng.uniform(rmin, rmax)
            self.ell(x, y, r, r, self.rng.choice(colors))


# ---------------------------------------------------------------- palettes
BONE = (233, 228, 211, 255)
BONE_L = (246, 242, 228, 255)
BONE_D = (196, 188, 164, 255)
ZED = (146, 166, 112, 255)          # generic zombie skin
ZED_D = (110, 128, 84, 255)
ZED_P = (178, 196, 142, 255)
CHAR = (40, 32, 28, 255)            # charred shell
CHAR_D = (26, 21, 19, 255)
CHAR_L = (66, 52, 42, 255)
MUSCLE = (128, 46, 36, 255)         # burnt muscle showing in cracks
LEATHER = (112, 78, 50, 255)
METAL = (122, 126, 134, 255)
METAL_D = (84, 88, 96, 255)
METAL_L = (168, 172, 180, 255)
FROST = (212, 228, 238, 255)
BLOOD = (150, 50, 44, 255)


# ---------------------------------------------------------------- humanoid
def humanoid(c, skin=ZED, cloth=None, bulk=1.0, arm_len=0.60, arm_w=0.055,
             leg_w=0.075, head_r=(0.085, 0.10), head_cy=0.17, stance=0.0):
    """Front-facing humanoid base. Returns dict of anchor points.
    stance shifts the whole figure sideways lean (-1..1)."""
    tw = 0.135 * bulk                     # torso half width
    sh_y, hip_y = 0.315, 0.615
    lx = stance * 0.02
    # legs
    for sgn in (-1, 1):
        hx = 0.5 + lx + sgn * tw * 0.52
        fx = 0.5 + lx + sgn * (tw * 0.62 + 0.012)
        c.taper([(hx, hip_y), (hx + sgn * 0.008, 0.80), (fx, 0.955)],
                leg_w, leg_w * 0.8, cloth or skin, DARK, 0.01)
        c.ell(fx + sgn * 0.012, 0.962, 0.030, 0.012, cloth or skin, DARK, 0.008)
    # torso
    c.poly([(0.5 + lx - tw, sh_y), (0.5 + lx + tw, sh_y),
            (0.5 + lx + tw * 0.82, hip_y + 0.03), (0.5 + lx - tw * 0.82, hip_y + 0.03)],
           cloth or skin, DARK, 0.011)
    # arms
    hands = {}
    for sgn in (-1, 1):
        sx = 0.5 + lx + sgn * tw
        ex = sx + sgn * 0.035
        hx = ex + sgn * 0.018
        c.taper([(sx, sh_y + 0.01), (ex, (sh_y + arm_len) / 2), (hx, arm_len)],
                arm_w, arm_w * 0.78, skin, DARK, 0.01)
        c.ell(hx, arm_len + 0.012, arm_w * 0.52, arm_w * 0.62, skin, DARK, 0.008)
        hands[sgn] = (hx, arm_len + 0.01)
    # head
    c.ell(0.5 + lx, head_cy, head_r[0], head_r[1], skin, DARK, 0.011)
    return {"tw": tw, "sh_y": sh_y, "hip_y": hip_y, "hands": hands,
            "head": (0.5 + lx, head_cy), "lx": lx}


def zed_face(c, hx, hy, r=(0.085, 0.10), eye=(20, 18, 16, 255), mouth=DARK):
    """Sunken dark eyes + slack mouth on an existing head."""
    c.ell(hx - r[0] * 0.38, hy - r[1] * 0.08, r[0] * 0.20, r[1] * 0.16, eye)
    c.ell(hx + r[0] * 0.38, hy - r[1] * 0.08, r[0] * 0.20, r[1] * 0.16, eye)
    c.line([(hx - r[0] * 0.30, hy + r[1] * 0.52), (hx + r[0] * 0.26, hy + r[1] * 0.58)],
           mouth, 0.016)


def skull_face(c, hx, hy, r=(0.088, 0.102), goo=False):
    """Bare skull: dark sockets, nose hole, teeth ticks. Head already drawn pale."""
    c.ell(hx - r[0] * 0.38, hy - r[1] * 0.10, r[0] * 0.24, r[1] * 0.20, (18, 16, 14, 255))
    c.ell(hx + r[0] * 0.38, hy - r[1] * 0.10, r[0] * 0.24, r[1] * 0.20, (18, 16, 14, 255))
    c.poly([(hx, hy + r[1] * 0.16), (hx - r[0] * 0.13, hy + r[1] * 0.42),
            (hx + r[0] * 0.13, hy + r[1] * 0.42)], (18, 16, 14, 255))
    if goo:  # black goo oozing from the sockets
        for sgn in (-1, 1):
            c.taper([(hx + sgn * r[0] * 0.38, hy + r[1] * 0.05),
                     (hx + sgn * r[0] * 0.42, hy + r[1] * 0.45),
                     (hx + sgn * r[0] * 0.38, hy + r[1] * 0.78)],
                    0.020, 0.007, (14, 12, 12, 255))
    # jaw + teeth
    c.line([(hx - r[0] * 0.42, hy + r[1] * 0.66), (hx + r[0] * 0.42, hy + r[1] * 0.66)],
           (120, 112, 92, 255), 0.014)
    for i in range(4):
        x = hx - r[0] * 0.30 + i * r[0] * 0.20
        c.line([(x, hy + r[1] * 0.66), (x, hy + r[1] * 0.82)], (120, 112, 92, 255), 0.010)


def cracks(c, cx, cy, rx, ry, count, col=MUSCLE, wunit=0.013):
    """Jagged crack lines with muscle showing through (scorched shell)."""
    for _ in range(count):
        x = cx + c.rng.uniform(-rx, rx) * 0.8
        y = cy + c.rng.uniform(-ry, ry) * 0.8
        pts = [(x, y)]
        for _ in range(3):
            x += c.rng.uniform(-0.035, 0.035)
            y += c.rng.uniform(0.01, 0.045)
            pts.append((x, y))
        c.line(pts, col, wunit)


# ---------------------------------------------------------------- quadruped
def quadruped(c, coat, dark=None, bulk=1.0, body_c=(0.47, 0.44), body_r=(0.285, 0.155),
              leg_top=0.54, leg_bot=0.93, head_at=(0.775, 0.335), neck=True):
    """Side-view quadruped facing right. Returns anchors."""
    dark = dark or tuple(max(0, v - 34) for v in coat[:3]) + (255,)
    bx, by = body_c
    rx, ry = body_r[0] * bulk, body_r[1] * bulk
    # far legs
    for fx in (bx - rx * 0.62, bx + rx * 0.58):
        c.taper([(fx, leg_top), (fx + 0.008, (leg_top + leg_bot) / 2), (fx + 0.004, leg_bot)],
                0.052 * bulk, 0.040 * bulk, dark, DARK, 0.009)
    # tail base (drawn by callers), body
    c.blob(bx, by, rx, ry, coat, DARK, n=12, jit=0.07)
    # near legs
    for fx in (bx - rx * 0.78, bx + rx * 0.76):
        c.taper([(fx, leg_top), (fx - 0.004, (leg_top + leg_bot) / 2), (fx, leg_bot)],
                0.058 * bulk, 0.044 * bulk, coat, DARK, 0.009)
        c.ell(fx + 0.016, leg_bot + 0.008, 0.026, 0.011, coat, DARK, 0.007)
    if neck:
        hx, hy = head_at
        c.taper([(bx + rx * 0.72, by - ry * 0.45), (hx - 0.055, hy + 0.045), (hx - 0.01, hy + 0.02)],
                0.115 * bulk, 0.085 * bulk, coat, DARK, 0.009)
    return {"body": (bx, by), "r": (rx, ry), "head": head_at, "dark": dark}


def dog_head(c, hx, hy, coat, dark=None, snout=0.085, ear_col=None):
    """Dog head with snout + ear, facing right."""
    dark = dark or DARK
    c.ell(hx, hy, 0.072, 0.066, coat, DARK, 0.010)
    c.poly([(hx + 0.03, hy + 0.005), (hx + 0.03 + snout, hy + 0.028),
            (hx + 0.028, hy + 0.052)], coat, DARK, 0.009)  # snout
    c.ell(hx + 0.03 + snout - 0.006, hy + 0.026, 0.013, 0.011, (30, 26, 28, 255))  # nose
    c.ell(hx + 0.018, hy - 0.016, 0.011, 0.013, (22, 20, 18, 255))  # eye
    c.poly([(hx - 0.045, hy - 0.045), (hx - 0.062, hy - 0.105), (hx - 0.008, hy - 0.062)],
           ear_col or coat, DARK, 0.009)  # ear


def tail_curve(c, x, y, col, wunit=0.045, up=True):
    sgn = -1 if up else 1
    pts = c.curve((x, y), (x - 0.085, y + sgn * 0.05), (x - 0.105, y + sgn * 0.16), 8)
    c.limb(pts, wunit, col)



# ================================================================ skeletons
def bone_plates(c, a):
    """Bone-armor seams + rib arcs over a humanoid torso."""
    tw, sy = a["tw"], a["sh_y"]
    for i in range(3):
        y = sy + 0.09 + i * 0.075
        c.arc(0.5 + a["lx"], y, tw * 0.78, 0.045, 15, 165, BONE_D, 0.013)
    c.line([(0.5 + a["lx"], sy + 0.02), (0.5 + a["lx"], a["hip_y"])], BONE_D, 0.013)


def skel_executioner(c):
    a = humanoid(c, skin=BONE, cloth=BONE_D, bulk=1.12, arm_len=0.47)
    bone_plates(c, a)
    # forearm sword blades
    for sgn in (-1, 1):
        ex, ey = a["hands"][sgn]
        ex = 0.5 + a["lx"] + sgn * (a["tw"] + 0.05)
        c.poly([(ex - sgn * 0.018, 0.44), (ex + sgn * 0.020, 0.44),
                (ex + sgn * 0.055, 0.80), (ex + sgn * 0.012, 0.845)],
               BONE_L, DARK, 0.010)
        c.line([(ex + sgn * 0.030, 0.47), (ex + sgn * 0.052, 0.79)], (255, 252, 240, 255), 0.008)
    # bone mask fused to jaw: pale face plate with sinister dark gaps
    hx, hy = a["head"]
    c.ell(hx, hy, 0.086, 0.101, BONE_L, DARK, 0.011)
    c.ell(hx - 0.032, hy - 0.012, 0.020, 0.017, (16, 14, 12, 255))
    c.ell(hx + 0.032, hy - 0.012, 0.020, 0.017, (16, 14, 12, 255))
    for gx in (-0.045, 0.0, 0.045):  # fused-jaw gaps
        c.line([(hx + gx, hy + 0.052), (hx + gx, hy + 0.088)], (16, 14, 12, 255), 0.013)


def skel_necro(c):
    robe = (52, 52, 58, 255)
    a = humanoid(c, skin=BONE, cloth=robe, bulk=0.95)
    # robe skirt over legs
    c.poly([(0.5 - 0.10, 0.58), (0.5 + 0.10, 0.58), (0.5 + 0.135, 0.965), (0.5 - 0.135, 0.965)],
           robe, DARK, 0.010)
    bone_plates(c, a)  # decked in bone head to toe
    hx, hy = a["head"]
    c.ell(hx, hy, 0.086, 0.101, BONE, DARK, 0.011)
    skull_face(c, hx, hy)
    # cold, calculating eyes: pale glints inside the sockets
    for sgn in (-1, 1):
        c.ell(hx + sgn * 0.032, hy - 0.010, 0.007, 0.007, (206, 232, 240, 255))
    # bone crown spikes
    for dx in (-0.05, -0.017, 0.017, 0.05):
        c.poly([(hx + dx - 0.010, hy - 0.088), (hx + dx, hy - 0.128), (hx + dx + 0.010, hy - 0.088)],
               BONE, DARK, 0.007)


def skel_reaper(c):
    a = humanoid(c, skin=BONE, cloth=BONE_D, bulk=1.22, arm_len=0.46)
    bone_plates(c, a)
    # scythe blades in place of hands
    for sgn in (-1, 1):
        hx = 0.5 + a["lx"] + sgn * (a["tw"] + 0.055)
        c.arc(hx + sgn * 0.055, 0.52, 0.115, 0.125, 250 if sgn > 0 else 200,
              60 if sgn > 0 else 20, BONE_L, 0.030)
        c.arc(hx + sgn * 0.055, 0.52, 0.115, 0.125, 250 if sgn > 0 else 200,
              60 if sgn > 0 else 20, (255, 252, 240, 255), 0.010)
    # thick ossified mask, eyes oozing black goo
    hx, hy = a["head"]
    c.ell(hx, hy, 0.092, 0.106, BONE_L, DARK, 0.012)
    skull_face(c, hx, hy, r=(0.092, 0.106), goo=True)


def skel_slasher(c):
    a = humanoid(c, skin=BONE, cloth=BONE_D, bulk=1.0, arm_len=0.585)
    bone_plates(c, a)
    # finger blades: three knife triangles per hand
    for sgn in (-1, 1):
        hx, hy = a["hands"][sgn]
        for i in range(3):
            dx = (i - 1) * 0.020
            c.poly([(hx + dx - 0.008, hy), (hx + dx + 0.008, hy),
                    (hx + dx + sgn * 0.006, hy + 0.085)], BONE_L, DARK, 0.007)
    # thickened exposed skull
    hx, hy = a["head"]
    c.ell(hx, hy, 0.094, 0.108, BONE_L, DARK, 0.012)
    skull_face(c, hx, hy, r=(0.094, 0.108))


# ================================================================ zombie horses
def horse_base(c, coat, dark=None):
    a = quadruped(c, coat, dark=dark, bulk=1.06, body_c=(0.455, 0.415),
                  body_r=(0.295, 0.165), leg_top=0.52, leg_bot=0.935,
                  head_at=(0.80, 0.275))
    return a


def horse_head(c, hx, hy, coat, mane_col, skull=False):
    # long equine head angled down-forward
    c.poly([(hx - 0.055, hy - 0.045), (hx + 0.045, hy - 0.028),
            (hx + 0.088, hy + 0.075), (hx + 0.052, hy + 0.115),
            (hx - 0.028, hy + 0.082)], coat, DARK, 0.010)
    c.poly([(hx - 0.038, hy - 0.052), (hx - 0.052, hy - 0.115), (hx - 0.008, hy - 0.062)],
           coat, DARK, 0.009)  # ear
    c.ell(hx + 0.062, hy + 0.082, 0.013, 0.010, (26, 22, 22, 255))  # nostril
    if skull:
        c.ell(hx + 0.012, hy + 0.005, 0.016, 0.019, (16, 14, 12, 255))  # socket
    else:
        c.ell(hx + 0.012, hy + 0.005, 0.010, 0.012, (20, 18, 16, 255))
    # mane along neck
    for i in range(4):
        x = hx - 0.085 - i * 0.035
        y = hy - 0.045 + i * 0.038
        c.poly([(x, y), (x - 0.030, y + 0.045), (x + 0.012, y + 0.052)], mane_col, DARK, 0.007)


def horse_tail(c, col):
    pts = c.curve((0.165, 0.36), (0.095, 0.48), (0.115, 0.68), 8)
    c.limb(pts, 0.038, col)


def horse_black(c):
    coat = (32, 30, 32, 255)
    a = horse_base(c, coat)
    horse_tail(c, (22, 20, 22, 255))
    horse_head(c, 0.80, 0.275, coat, (20, 19, 20, 255))
    c.ell(0.812, 0.280, 0.011, 0.012, (216, 230, 244, 255))  # blue-white eye glint
    # gashes and holes along the underside
    for (gx, gy) in [(0.34, 0.545), (0.45, 0.565), (0.56, 0.55), (0.40, 0.52)]:
        c.ell(gx, gy, 0.026, 0.016, (10, 9, 10, 255), (70, 60, 58, 255), 0.008)
    # pillars of choking black smoke pouring from the wounds
    for (sx, sy, sr) in [(0.36, 0.30, 0.055), (0.44, 0.20, 0.070), (0.52, 0.115, 0.082),
                         (0.60, 0.26, 0.048), (0.47, 0.335, 0.042)]:
        c.ell(sx, sy, sr, sr * 0.82, (52, 50, 54, 255))
    for (sx, sy, sr) in [(0.45, 0.185, 0.045), (0.53, 0.10, 0.052)]:
        c.ell(sx, sy, sr, sr * 0.8, (74, 72, 78, 255))


def horse_pale(c):
    coat = (206, 200, 186, 255)
    a = horse_base(c, coat, dark=(172, 165, 148, 255))
    horse_tail(c, (188, 182, 168, 255))
    # overlapping asymmetrical bone plates over back and flank
    plates = [(0.30, 0.315, 0.075, 0.045), (0.42, 0.285, 0.088, 0.050),
              (0.545, 0.30, 0.080, 0.046), (0.365, 0.40, 0.070, 0.052),
              (0.50, 0.415, 0.084, 0.055), (0.615, 0.395, 0.062, 0.048),
              (0.27, 0.43, 0.058, 0.046), (0.575, 0.475, 0.055, 0.040)]
    for (px, py, prx, pry) in plates:
        c.blob(px, py, prx, pry, BONE, DARK, n=8, jit=0.10)
    # overgrown skull head, flattened battered forehead
    hx, hy = 0.80, 0.275
    c.poly([(hx - 0.055, hy - 0.045), (hx + 0.038, hy - 0.034),
            (hx + 0.082, hy + 0.068), (hx + 0.046, hy + 0.118),
            (hx - 0.028, hy + 0.082)], BONE, DARK, 0.010)
    c.line([(hx + 0.030, hy - 0.030), (hx + 0.052, hy + 0.030)], BONE_D, 0.012)  # flat brow
    c.line([(hx + 0.040, hy - 0.018), (hx + 0.028, hy + 0.012)], (140, 132, 112, 255), 0.008)
    c.ell(hx + 0.012, hy + 0.005, 0.016, 0.019, (16, 14, 12, 255))
    c.ell(hx + 0.062, hy + 0.082, 0.013, 0.010, (26, 22, 22, 255))
    c.poly([(hx - 0.038, hy - 0.052), (hx - 0.052, hy - 0.115), (hx - 0.008, hy - 0.062)],
           BONE, DARK, 0.009)


def horse_red(c):
    coat = (158, 52, 44, 255)
    a = horse_base(c, coat, dark=(128, 40, 36, 255))
    horse_tail(c, (110, 34, 30, 255))
    horse_head(c, 0.80, 0.275, coat, (96, 30, 28, 255))
    c.ell(0.812, 0.280, 0.011, 0.011, (240, 236, 228, 255))  # bare white eye
    # exposed muscle striations
    for i in range(5):
        x = 0.27 + i * 0.085
        c.arc(x, 0.43, 0.030, 0.105, 80, 260, (118, 34, 30, 255), 0.011)
    for i in range(3):
        c.line([(0.30 + i * 0.13, 0.315), (0.315 + i * 0.13, 0.40)], (118, 34, 30, 255), 0.010)
    # torn muscle strands flailing loose from the flank
    for (p0, p1, p2) in [((0.40, 0.52), (0.335, 0.60), (0.375, 0.685)),
                         ((0.55, 0.535), (0.615, 0.615), (0.575, 0.70))]:
        c.limb(c.curve(p0, p1, p2, 8), 0.020, (168, 60, 50, 255))


def horse_white(c):
    coat = (208, 202, 170, 255)
    a = quadruped(c, coat, dark=(178, 170, 138, 255), bulk=1.06, body_c=(0.445, 0.40),
                  body_r=(0.29, 0.155), leg_top=0.52, leg_bot=0.935,
                  head_at=(0.815, 0.30), neck=False)
    # longer, thicker neck
    c.taper([(0.655, 0.335), (0.755, 0.315), (0.80, 0.315)], 0.150, 0.105, coat, DARK, 0.010)
    horse_tail(c, (190, 184, 152, 255))
    horse_head(c, 0.815, 0.30, coat, (196, 190, 158, 255))
    # rotund belly: near-see-through skin over a caustic bile reservoir
    c.ell(0.45, 0.565, 0.185, 0.105, (226, 220, 182, 255), DARK, 0.010)
    c.blob(0.45, 0.575, 0.145, 0.078, (148, 168, 58, 255), None, n=10, jit=0.10)
    c.blob(0.415, 0.555, 0.070, 0.040, (172, 190, 76, 255), None, n=8, jit=0.12)
    for (bx, by, br) in [(0.40, 0.545, 0.013), (0.475, 0.585, 0.010), (0.44, 0.60, 0.008)]:
        c.ell(bx, by, br, br, (196, 208, 110, 255))  # bile bubbles


# ================================================================ scorched zeds
def scorched_base(c, bulk=1.0, arm_len=0.60, cloth=CHAR):
    a = humanoid(c, skin=CHAR, cloth=cloth, bulk=bulk, arm_len=arm_len)
    # leathery patches mended over the char
    c.speckles(0.5, 0.45, 0.11 * bulk, 0.16, 7, [LEATHER, CHAR_L], 0.014, 0.034)
    cracks(c, 0.5, 0.44, 0.11 * bulk, 0.17, 7)
    cracks(c, 0.5, 0.17, 0.07, 0.08, 3)
    zed_face(c, *a["head"], eye=(12, 10, 8, 255))
    return a


def metal_shards(c, spots):
    for (sx, sy, sa) in spots:
        c.poly([(sx, sy), (sx + 0.030, sy + sa), (sx + 0.006, sy + 0.048)],
               METAL, DARK, 0.007)
        c.line([(sx + 0.006, sy + 0.004), (sx + 0.024, sy + sa + 0.002)], METAL_L, 0.006)


def scorched_brute(c):
    scorched_base(c, bulk=1.30)


def scorched_lasher(c):
    # unnaturally long arms dragging behind / on the ground
    a = humanoid(c, skin=CHAR, cloth=CHAR, bulk=0.98, arm_len=0.60)
    for sgn in (-1, 1):
        sx = 0.5 + sgn * (a["tw"] + 0.01)
        pts = c.curve((sx, 0.33), (sx + sgn * 0.075, 0.62), (sx + sgn * 0.105, 0.925), 8)
        c.limb(pts, 0.052, CHAR)
        c.ell(sx + sgn * 0.105, 0.935, 0.030, 0.014, CHAR, DARK, 0.008)  # knuckle fist
    c.speckles(0.5, 0.45, 0.10, 0.16, 6, [LEATHER, CHAR_L], 0.014, 0.032)
    cracks(c, 0.5, 0.44, 0.10, 0.17, 6)
    zed_face(c, *a["head"], eye=(12, 10, 8, 255))


def scorched_rust(c):
    scorched_base(c, bulk=1.02)
    metal_shards(c, [(0.415, 0.30, 0.010), (0.565, 0.335, -0.006), (0.475, 0.47, 0.014),
                     (0.60, 0.52, 0.004)])


def fiend_rust(c):
    scorched_base(c, bulk=1.30)
    metal_shards(c, [(0.40, 0.29, 0.012), (0.575, 0.32, -0.008), (0.46, 0.44, 0.016),
                     (0.615, 0.50, 0.002), (0.375, 0.55, 0.008)])


# ================================================================ fireproof zeds
NOMEX = (172, 140, 76, 255)       # off-yellow turnout fabric flesh
NOMEX_D = (128, 100, 52, 255)
NOMEX_L = (206, 178, 108, 255)


def nomex_fibers(c, cx, cy, rx, ry, count=8):
    for _ in range(count):
        x = cx + c.rng.uniform(-rx, rx)
        y = cy + c.rng.uniform(-ry, ry)
        c.line([(x, y), (x + c.rng.uniform(-0.02, 0.02), y + c.rng.uniform(0.03, 0.07))],
               NOMEX_D, 0.011)


def fireproof(c):
    # prowling on all fours, turnout gear clinging like a molted carapace
    coat = NOMEX
    a = quadruped(c, coat, dark=NOMEX_D, bulk=0.98, body_c=(0.47, 0.47),
                  body_r=(0.265, 0.135), leg_top=0.56, leg_bot=0.925,
                  head_at=(0.775, 0.415), neck=False)
    c.taper([(0.665, 0.42), (0.735, 0.415), (0.765, 0.42)], 0.095, 0.075, coat, DARK, 0.009)
    # humanoid zed head, low and prowling
    c.ell(0.795, 0.415, 0.068, 0.076, ZED, DARK, 0.010)
    zed_face(c, 0.795, 0.415, r=(0.068, 0.076))
    # wilted turnout panels on the back
    for (px, py, prx) in [(0.36, 0.375, 0.075), (0.50, 0.36, 0.082), (0.60, 0.395, 0.06)]:
        c.blob(px, py, prx, 0.042, NOMEX_L, DARK, n=8, jit=0.12)
    c.line([(0.30, 0.415), (0.63, 0.415)], (240, 226, 150, 255), 0.016)  # reflective stripe remnant
    nomex_fibers(c, 0.47, 0.48, 0.20, 0.10, 7)
    tail_curve(c, 0.20, 0.42, NOMEX_D, 0.030)


def fireproof_brute(c):
    a = humanoid(c, skin=(186, 118, 58, 255), cloth=(172, 108, 52, 255), bulk=1.30, arm_len=0.565)
    nomex_fibers(c, 0.5, 0.45, 0.16, 0.17, 10)
    # hands congealed into a pair of clubs
    for sgn in (-1, 1):
        hx, hy = a["hands"][sgn]
        c.ell(hx, hy + 0.045, 0.052, 0.068, (172, 108, 52, 255), DARK, 0.011)
        c.ell(hx - 0.014, hy + 0.028, 0.020, 0.026, NOMEX_L)
    zed_face(c, *a["head"])


def fireproof_grappler(c):
    a = humanoid(c, skin=(122, 104, 56, 255), cloth=(110, 94, 48, 255), bulk=1.10, arm_len=0.60)
    # swollen torso and back with knotted Nomex growths
    c.blob(0.5, 0.44, 0.155, 0.175, (110, 94, 48, 255), DARK, n=11, jit=0.09)
    c.speckles(0.5, 0.44, 0.12, 0.14, 9, [NOMEX_D, (86, 72, 36, 255)], 0.012, 0.026)
    # lanky arms dragging on the floor
    for sgn in (-1, 1):
        sx = 0.5 + sgn * (a["tw"] + 0.015)
        pts = c.curve((sx, 0.34), (sx + sgn * 0.06, 0.63), (sx + sgn * 0.085, 0.915), 8)
        c.limb(pts, 0.046, (122, 104, 56, 255))
        c.ell(sx + sgn * 0.085, 0.925, 0.027, 0.013, (122, 104, 56, 255), DARK, 0.008)
    zed_face(c, *a["head"])


# ================================================================ no-weakpoints pair
def soldier_nw(c):
    a = humanoid(c, skin=ZED, cloth=(96, 112, 70, 255), bulk=1.05)
    # helmet
    hx, hy = a["head"]
    c.arc(hx, hy - 0.01, 0.098, 0.095, 180, 360, (80, 94, 58, 255), 0.030)
    c.line([(hx - 0.105, hy - 0.008), (hx + 0.105, hy - 0.008)], (80, 94, 58, 255), 0.014)
    # vest + pouches
    c.rect(0.435, 0.36, 0.565, 0.52, (74, 86, 52, 255), DARK, 0.009)
    c.rect(0.452, 0.545, 0.492, 0.585, (74, 86, 52, 255), DARK, 0.007)
    c.rect(0.508, 0.545, 0.548, 0.585, (74, 86, 52, 255), DARK, 0.007)
    zed_face(c, hx, hy + 0.012)


def survivor_nw(c):
    a = humanoid(c, skin=ZED, cloth=(148, 82, 68, 255), bulk=1.0)
    # diagonal pack strap + torn shirt patch
    c.line([(0.415, 0.33), (0.575, 0.55)], (84, 62, 48, 255), 0.026)
    c.poly([(0.44, 0.47), (0.485, 0.465), (0.478, 0.515), (0.438, 0.512)], ZED_D, DARK, 0.007)
    zed_face(c, *a["head"])


# ================================================================ singles (humanoid)
def zombie_child(c):
    a = humanoid(c, skin=(158, 186, 124, 255), cloth=(120, 130, 148, 255), bulk=0.78,
                 arm_len=0.565, head_r=(0.112, 0.122), head_cy=0.205)
    hx, hy = a["head"]
    # hungry look: wide eyes, small open mouth
    for sgn in (-1, 1):
        c.ell(hx + sgn * 0.042, hy - 0.008, 0.020, 0.024, (24, 22, 18, 255))
        c.ell(hx + sgn * 0.042, hy - 0.014, 0.006, 0.007, (240, 244, 230, 255))
    c.ell(hx, hy + 0.062, 0.018, 0.023, (40, 30, 26, 255))


def xedra_scarlet(c):
    suit = (216, 222, 226, 255)
    a = humanoid(c, skin=suit, cloth=(198, 206, 212, 255), bulk=1.08)
    # interlocking plate seams
    for y in (0.385, 0.455, 0.525):
        c.line([(0.5 - a["tw"] * 0.9, y), (0.5 + a["tw"] * 0.9, y)], METAL_D, 0.009)
    c.line([(0.5, 0.32), (0.5, 0.60)], METAL_D, 0.009)
    # helmet with cyan visor
    hx, hy = a["head"]
    c.ell(hx, hy, 0.104, 0.112, suit, DARK, 0.012)
    c.ell(hx, hy + 0.004, 0.072, 0.058, (120, 205, 220, 255), DARK, 0.009)
    c.line([(hx - 0.05, hy - 0.018), (hx + 0.028, hy - 0.018)], (226, 246, 250, 255), 0.012)
    # yellowish-red staining, strongest at the joints
    stain = (198, 128, 84, 255)
    for (jx, jy) in [(0.5 - a["tw"] - 0.01, 0.335), (0.5 + a["tw"] + 0.01, 0.335),
                     (0.455, 0.79), (0.545, 0.79), (0.5, 0.615)]:
        c.blob(jx, jy, 0.026, 0.020, stain, None, n=7, jit=0.25)
    c.speckles(0.5, 0.45, 0.10, 0.15, 5, [stain], 0.006, 0.013)


def grue(c):
    # massive, lanky spectre bathed in shadow; swollen gray eyes, stark white maw
    body = (30, 28, 36, 255)
    edge = (74, 70, 84, 255)
    # stretched legs
    for sgn in (-1, 1):
        hx = 0.5 + sgn * 0.055
        c.taper([(hx, 0.60), (hx + sgn * 0.012, 0.80), (hx + sgn * 0.02, 0.955)],
                0.046, 0.034, body, None)
    # hunched narrow torso
    c.poly([(0.415, 0.235), (0.585, 0.235), (0.565, 0.615), (0.435, 0.615)], body, None)
    c.line([(0.418, 0.25), (0.438, 0.60)], edge, 0.010)  # faint rim light
    # enormously long arms with stretched digits
    for sgn in (-1, 1):
        sx = 0.5 + sgn * 0.088
        pts = c.curve((sx, 0.25), (sx + sgn * 0.055, 0.55), (sx + sgn * 0.075, 0.885), 8)
        c.limb(pts, 0.036, body)
        fx, fy = sx + sgn * 0.075, 0.895
        for fi in range(4):  # long fingers
            dx = (fi - 1.5) * 0.016
            c.line([(fx + dx * 0.4, fy), (fx + dx, fy + 0.055)], body, 0.009)
    # head: swollen gray eyes + slack white maw
    c.ell(0.5, 0.155, 0.068, 0.082, body, None)
    for sgn in (-1, 1):
        c.ell(0.5 + sgn * 0.030, 0.128, 0.021, 0.026, (168, 168, 174, 255))
        c.ell(0.5 + sgn * 0.030, 0.122, 0.007, 0.009, (226, 226, 230, 255))
    c.poly([(0.468, 0.185), (0.532, 0.185), (0.524, 0.243), (0.476, 0.243)],
           (238, 238, 240, 255), None)  # the stark white hanging maw
    c.line([(0.476, 0.243), (0.524, 0.243)], (150, 148, 152, 255), 0.008)


def brute_string(c):
    skin = (178, 150, 110, 255)  # preserved, almost living
    a = humanoid(c, skin=skin, cloth=(104, 96, 84, 255), bulk=1.28, arm_len=0.585)
    # corded muscle striations on arms and chest
    for i in range(3):
        c.arc(0.5, 0.42 + i * 0.055, 0.085, 0.030, 20, 160, (146, 118, 82, 255), 0.010)
    # ceremonial metal armor hanging loose, black-stained
    armor = (70, 66, 72, 255)
    c.poly([(0.408, 0.345), (0.487, 0.335), (0.478, 0.50), (0.415, 0.515)], armor, DARK, 0.010)
    c.poly([(0.515, 0.338), (0.592, 0.352), (0.585, 0.518), (0.522, 0.502)], armor, DARK, 0.010)
    c.speckles(0.45, 0.42, 0.035, 0.07, 5, [(16, 14, 14, 255)], 0.008, 0.018)
    c.speckles(0.555, 0.43, 0.035, 0.07, 5, [(16, 14, 14, 255)], 0.008, 0.018)
    c.line([(0.415, 0.335), (0.40, 0.30)], METAL_D, 0.012)   # loose straps
    c.line([(0.588, 0.345), (0.602, 0.305)], METAL_D, 0.012)
    # bulging eyes, drawn visage
    hx, hy = a["head"]
    for sgn in (-1, 1):
        c.ell(hx + sgn * 0.033, hy - 0.008, 0.017, 0.019, (242, 240, 232, 255), DARK, 0.006)
        c.ell(hx + sgn * 0.033, hy - 0.006, 0.007, 0.008, (20, 16, 14, 255))
    c.line([(hx - 0.026, hy + 0.055), (hx + 0.024, hy + 0.058)], (96, 74, 52, 255), 0.013)
    c.line([(hx - 0.045, hy + 0.028), (hx - 0.028, hy + 0.036)], (146, 118, 82, 255), 0.009)
    c.line([(hx + 0.045, hy + 0.028), (hx + 0.028, hy + 0.036)], (146, 118, 82, 255), 0.009)


# ================================================================ zomborgs
def zomborg_boss(c):
    flesh = (32, 30, 32, 255)  # black frostbitten flesh
    # mismatched legs: one metal, one flesh
    for sgn, col in ((-1, METAL_D), (1, flesh)):
        hx = 0.5 + sgn * 0.075
        c.taper([(hx, 0.615), (hx + sgn * 0.01, 0.80), (hx + sgn * 0.018, 0.955)],
                0.078, 0.062, col, DARK, 0.010)
    # armored robotic torso, shoulders uneven (awkward pull)
    c.poly([(0.365, 0.315), (0.635, 0.295), (0.615, 0.63), (0.385, 0.63)], METAL, DARK, 0.012)
    c.line([(0.375, 0.40), (0.625, 0.385)], METAL_D, 0.011)
    c.line([(0.38, 0.50), (0.62, 0.49)], METAL_D, 0.011)
    for (rx, ry) in [(0.41, 0.345), (0.59, 0.33), (0.42, 0.585), (0.585, 0.575)]:
        c.ell(rx, ry, 0.008, 0.008, METAL_D)
    c.ell(0.50, 0.445, 0.020, 0.020, (196, 60, 48, 255), DARK, 0.007)  # core light
    # one segmented robot arm, one black flesh arm
    c.taper([(0.365, 0.325), (0.325, 0.47), (0.335, 0.60)], 0.062, 0.05, METAL, DARK, 0.010)
    c.line([(0.338, 0.47), (0.352, 0.47)], METAL_D, 0.012)
    c.taper([(0.635, 0.305), (0.678, 0.46), (0.668, 0.615)], 0.058, 0.046, flesh, DARK, 0.010)
    # bands of black flesh wrapped over the metal, tugging at angles
    for (bx, by, ang) in [(0.44, 0.36, 0.03), (0.545, 0.465, -0.025), (0.465, 0.555, 0.02)]:
        c.line([(bx - 0.055, by - ang), (bx + 0.055, by + ang)], flesh, 0.024)
    # head: half metal plate, half black flesh; frost patches
    c.ell(0.5, 0.17, 0.086, 0.101, flesh, DARK, 0.011)
    c.poly([(0.5, 0.072), (0.585, 0.10), (0.586, 0.24), (0.5, 0.268)], METAL, DARK, 0.009)
    c.ell(0.462, 0.162, 0.013, 0.015, (14, 12, 12, 255))
    c.ell(0.545, 0.162, 0.011, 0.013, (220, 70, 56, 255))  # optic
    c.blob(0.415, 0.115, 0.030, 0.020, FROST, None, n=7, jit=0.2)
    c.blob(0.60, 0.315, 0.026, 0.018, FROST, None, n=7, jit=0.2)


def zomborg_devourer(c, frost=False):
    # colossus: bloated fused mass with heads and limbs sticking out at angles
    mass = (172, 154, 142, 255)
    mass_d = (140, 122, 112, 255)
    c.blob(0.5, 0.565, 0.295, 0.315, mass, DARK, n=13, jit=0.10)
    c.speckles(0.5, 0.56, 0.20, 0.22, 10, [mass_d, (188, 168, 152, 255)], 0.012, 0.030)
    # stub legs at the base
    for sgn in (-1, 1):
        c.taper([(0.5 + sgn * 0.13, 0.82), (0.5 + sgn * 0.15, 0.955)], 0.075, 0.06, mass_d, DARK, 0.010)
    # a robotic limb sticking out, upper left: segmented arm + claw
    c.taper([(0.30, 0.42), (0.205, 0.345), (0.165, 0.26)], 0.052, 0.04, METAL, DARK, 0.010)
    for dx in (-0.016, 0.014):
        c.line([(0.165 + dx, 0.255), (0.155 + dx * 1.6, 0.205)], METAL_D, 0.011)
    # a flesh arm reaching out, right side
    c.taper([(0.755, 0.50), (0.845, 0.44), (0.875, 0.345)], 0.055, 0.042, mass, DARK, 0.010)
    c.ell(0.878, 0.335, 0.026, 0.030, mass, DARK, 0.008)
    # a leg jutting upward at an angle
    c.taper([(0.665, 0.335), (0.72, 0.235), (0.755, 0.155)], 0.055, 0.042, mass_d, DARK, 0.010)
    c.ell(0.762, 0.148, 0.030, 0.014, mass_d, DARK, 0.008)
    # human head sticking out of the mass, upper center-left
    c.ell(0.415, 0.315, 0.052, 0.060, ZED_P, DARK, 0.010)
    zed_face(c, 0.415, 0.315, r=(0.052, 0.060))
    # robot head embedded right of center
    c.rect(0.545, 0.335, 0.635, 0.425, METAL, DARK, 0.010)
    c.ell(0.575, 0.372, 0.011, 0.011, (220, 70, 56, 255))
    c.ell(0.608, 0.372, 0.011, 0.011, (220, 70, 56, 255))
    c.line([(0.555, 0.405), (0.625, 0.405)], METAL_D, 0.009)
    # main head on top: half flesh, half plate, screaming
    c.ell(0.5, 0.175, 0.068, 0.078, ZED, DARK, 0.011)
    c.poly([(0.5, 0.10), (0.566, 0.125), (0.568, 0.225), (0.5, 0.248)], METAL, DARK, 0.009)
    c.ell(0.468, 0.168, 0.011, 0.013, (16, 14, 12, 255))
    c.ell(0.535, 0.168, 0.009, 0.011, (220, 70, 56, 255))
    c.ell(0.478, 0.222, 0.016, 0.020, (30, 20, 18, 255))  # open mouth
    # panel + wires on the mass
    c.rect(0.315, 0.565, 0.385, 0.635, METAL_D, DARK, 0.009)
    c.line(c.curve((0.60, 0.62), (0.66, 0.68), (0.63, 0.76), 7), (60, 58, 60, 255), 0.011)
    if frost:
        for (fx, fy, fr) in [(0.335, 0.335, 0.034), (0.63, 0.235, 0.028), (0.545, 0.70, 0.038),
                             (0.415, 0.115, 0.024)]:
            c.blob(fx, fy, fr, fr * 0.62, FROST, None, n=7, jit=0.22)


def zomborg_devourer_string(c):
    zomborg_devourer(c, frost=True)


# ================================================================ weird dogs
def dog_base(c, coat, dark=None, bulk=1.0):
    a = quadruped(c, coat, dark=dark, bulk=bulk, body_c=(0.465, 0.455),
                  body_r=(0.265, 0.14), leg_top=0.55, leg_bot=0.915,
                  head_at=(0.755, 0.365))
    return a


def dog_antimemetic(c):
    coat = (178, 178, 182, 255)
    dog_base(c, coat)
    tail_curve(c, 0.205, 0.41, coat)
    # the head cannot be retained: a static-smudge where it should be
    c.blob(0.775, 0.36, 0.105, 0.088, (198, 198, 202, 255), None, n=11, jit=0.22)
    c.speckles(0.775, 0.36, 0.095, 0.078, 42,
               [(150, 150, 156, 255), (226, 226, 230, 255), (120, 120, 128, 255),
                (188, 188, 192, 255)], 0.005, 0.013)
    for i in range(3):  # erasure streaks
        y = 0.315 + i * 0.045
        c.line([(0.70, y), (0.85, y + c.rng.uniform(-0.008, 0.008))], (210, 210, 214, 255), 0.010)


def dog_cuspated(c):
    coat = (216, 198, 190, 255)  # white_red: pale with raw red patches
    a = dog_base(c, coat, dark=(192, 168, 158, 255))
    tail_curve(c, 0.205, 0.41, coat)
    c.speckles(0.44, 0.46, 0.17, 0.09, 6, [(176, 84, 70, 255)], 0.012, 0.026)
    # head slung low; snout and jaw fused into one jagged bony point
    c.taper([(0.665, 0.40), (0.725, 0.475), (0.755, 0.525)], 0.105, 0.08, coat, DARK, 0.009)
    c.ell(0.762, 0.53, 0.066, 0.060, coat, DARK, 0.010)
    c.poly([(0.775, 0.545), (0.905, 0.635), (0.872, 0.672), (0.845, 0.640),
            (0.822, 0.678), (0.775, 0.60)], BONE, DARK, 0.010)  # the 'beak'
    c.line([(0.79, 0.565), (0.875, 0.638)], BONE_D, 0.009)
    c.ell(0.745, 0.505, 0.011, 0.012, (24, 20, 18, 255))  # eye
    c.poly([(0.715, 0.485), (0.70, 0.425), (0.745, 0.465)], coat, DARK, 0.008)  # ear


def dog_howling(c):
    coat = (172, 84, 140, 255)  # magenta, decomposing
    a = dog_base(c, coat, dark=(140, 62, 112, 255))
    tail_curve(c, 0.205, 0.41, coat)
    c.speckles(0.45, 0.47, 0.18, 0.09, 7, [(96, 110, 62, 255), (128, 56, 104, 255)],
               0.012, 0.028)  # rot spots
    # arrhythmic breathing underbelly: ribbed bulge
    c.ell(0.46, 0.575, 0.155, 0.062, (186, 100, 154, 255), DARK, 0.009)
    for i in range(4):
        c.arc(0.385 + i * 0.05, 0.575, 0.020, 0.048, 300, 60, (140, 62, 112, 255), 0.009)
    # oversized snout and throat
    c.taper([(0.66, 0.40), (0.72, 0.42), (0.75, 0.45)], 0.115, 0.095, coat, DARK, 0.009)
    c.ell(0.775, 0.435, 0.075, 0.070, coat, DARK, 0.010)
    c.ell(0.775, 0.505, 0.052, 0.038, (186, 100, 154, 255), DARK, 0.008)  # swollen throat
    c.poly([(0.795, 0.455), (0.895, 0.478), (0.80, 0.512)], coat, DARK, 0.009)  # long snout
    c.ell(0.878, 0.472, 0.012, 0.010, (28, 22, 26, 255))
    # analytical eyes: bright, alert, wrong for a corpse
    c.ell(0.755, 0.408, 0.014, 0.016, (240, 240, 234, 255), DARK, 0.006)
    c.ell(0.756, 0.409, 0.007, 0.008, (18, 16, 20, 255))
    c.poly([(0.715, 0.385), (0.698, 0.318), (0.752, 0.362)], coat, DARK, 0.008)  # ear


def dog_incorporeal(c):
    coat = (226, 231, 238, 255)   # pale opaque ghost tones
    line_col = (168, 178, 196, 255)
    # faint echo silhouette behind (offset, still opaque)
    a = quadruped(c, (212, 220, 230, 255), dark=(196, 206, 218, 255), bulk=1.0,
                  body_c=(0.445, 0.435), body_r=(0.265, 0.14), leg_top=0.53,
                  leg_bot=0.895, head_at=(0.735, 0.345))
    a = dog_base(c, coat, dark=(200, 208, 220, 255))
    tail_curve(c, 0.205, 0.41, coat, 0.040)
    dog_head(c, 0.755, 0.365, coat, dark=line_col)
    # see-through hint: faint inner outline of the far side
    c.arc(0.47, 0.455, 0.20, 0.095, 200, 340, (196, 206, 218, 255), 0.008)


def dog_scorched(c):
    coat = (56, 44, 36, 255)
    a = dog_base(c, coat, dark=(38, 30, 25, 255))
    tail_curve(c, 0.205, 0.41, (44, 35, 29, 255), 0.040)
    dog_head(c, 0.755, 0.365, coat)
    # fused patches of burnt hide and flesh
    c.speckles(0.46, 0.45, 0.19, 0.095, 9, [CHAR_D, LEATHER, CHAR_L], 0.014, 0.032)
    cracks(c, 0.47, 0.44, 0.16, 0.08, 4)


def dog_scorched_hulk(c):
    coat = (52, 41, 33, 255)
    a = quadruped(c, coat, dark=(36, 28, 23, 255), bulk=1.22, body_c=(0.465, 0.44),
                  body_r=(0.275, 0.165), leg_top=0.545, leg_bot=0.935,
                  head_at=(0.775, 0.35))
    tail_curve(c, 0.185, 0.395, (40, 31, 26, 255), 0.052)
    dog_head(c, 0.775, 0.35, coat, snout=0.10)
    # entire body under a fused, melted hide shell: plates + cracks
    c.speckles(0.465, 0.44, 0.21, 0.12, 8, [CHAR_L, LEATHER], 0.016, 0.036)
    cracks(c, 0.465, 0.44, 0.20, 0.11, 8)
    cracks(c, 0.775, 0.35, 0.05, 0.05, 2)


def dog_telekenetic(c):
    coat = (150, 128, 104, 255)  # a rather average dog
    a = dog_base(c, coat, dark=(122, 102, 80, 255))
    tail_curve(c, 0.205, 0.41, coat)
    dog_head(c, 0.755, 0.365, coat)
    # the third eye, center forehead, under a thick eyelid
    c.ell(0.748, 0.318, 0.020, 0.023, (240, 238, 230, 255), DARK, 0.007)
    c.ell(0.748, 0.322, 0.009, 0.011, (96, 60, 120, 255))
    c.arc(0.748, 0.312, 0.021, 0.020, 180, 360, coat, 0.016)  # heavy lid
    # dust and grit rising, swirling around it
    for i in range(14):
        ang = i * 2.4
        rr = 0.30 + 0.05 * math.sin(i * 1.7)
        x = 0.47 + math.cos(ang) * rr
        y = 0.60 + math.sin(ang) * rr * 0.42 - i * 0.006
        r = 0.006 + (i % 3) * 0.003
        c.ell(x, y, r, r, (206, 190, 160, 255))
    for (px, py) in [(0.30, 0.70), (0.63, 0.66), (0.52, 0.755)]:
        c.ell(px, py, 0.010, 0.008, (178, 160, 132, 255))  # levitating pebbles


# ================================================================ weird cats
def cat_base(c, coat, dark=None):
    a = quadruped(c, coat, dark=dark, bulk=0.92, body_c=(0.475, 0.545),
                  body_r=(0.235, 0.105), leg_top=0.615, leg_bot=0.875,
                  head_at=(0.715, 0.475), neck=False)
    c.taper([(0.63, 0.52), (0.685, 0.49), (0.71, 0.48)], 0.07, 0.055, coat, DARK, 0.008)
    return a


def cat_head(c, hx, hy, coat, eye_col=(30, 40, 26, 255)):
    c.ell(hx, hy, 0.080, 0.074, coat, DARK, 0.010)
    for sgn in (-1, 1):  # triangle ears
        ex = hx + sgn * 0.048
        c.poly([(ex - 0.020, hy - 0.052), (ex + 0.004, hy - 0.115), (ex + 0.024, hy - 0.048)],
               coat, DARK, 0.008)
    c.ell(hx + 0.030, hy - 0.006, 0.010, 0.014, eye_col)  # eye
    c.ell(hx + 0.068, hy + 0.026, 0.009, 0.007, (120, 70, 76, 255))  # nose
    for i in range(2):  # whiskers
        c.line([(hx + 0.055, hy + 0.018 + i * 0.014), (hx + 0.115, hy + 0.008 + i * 0.02)],
               (200, 200, 200, 255), 0.005)


def cat_liquid(c):
    coat = (186, 186, 190, 255)
    # body gone wavy and sagging, spine twisted like a poured ribbon
    pts_top = [(0.225, 0.525), (0.33, 0.445), (0.45, 0.525), (0.565, 0.445), (0.685, 0.505)]
    pts_bot = [(0.685, 0.645), (0.565, 0.575), (0.45, 0.665), (0.33, 0.585), (0.225, 0.655)]
    c.poly(pts_top + pts_bot, coat, DARK, 0.010)
    for fx in (0.28, 0.60):  # legs, one draped at a wrong angle
        c.taper([(fx, 0.615), (fx + 0.006, 0.745), (fx + 0.002, 0.875)], 0.048, 0.036, coat, DARK, 0.008)
    c.taper([(0.42, 0.635), (0.455, 0.73), (0.505, 0.795)], 0.046, 0.034, coat, DARK, 0.008)
    # tail pouring over an invisible edge
    c.limb(c.curve((0.225, 0.555), (0.15, 0.565), (0.14, 0.705), 8), 0.040, coat)
    # head low, twisted sideways
    c.taper([(0.655, 0.52), (0.70, 0.545), (0.725, 0.565)], 0.065, 0.05, coat, DARK, 0.008)
    cat_head(c, 0.745, 0.575, coat)
    # quiver marks
    for (qx, qy) in [(0.36, 0.415), (0.52, 0.405), (0.615, 0.42)]:
        c.arc(qx, qy, 0.020, 0.014, 200, 340, (140, 140, 146, 255), 0.007)


def cat_void(c):
    # aura silhouette first: a slightly larger darkness behind the cat
    a = quadruped(c, (26, 24, 34, 255), dark=(26, 24, 34, 255), bulk=0.96,
                  body_c=(0.475, 0.54), body_r=(0.245, 0.112), leg_top=0.61,
                  leg_bot=0.88, head_at=(0.715, 0.47), neck=False)
    coat = (13, 12, 17, 255)  # unnaturally dark: absorbs the light
    a = cat_base(c, coat, dark=(13, 12, 17, 255))
    c.line(c.curve((0.245, 0.52), (0.165, 0.40), (0.225, 0.315), 8), coat, 0.036)  # tail up
    cat_head(c, 0.715, 0.475, coat, eye_col=(206, 216, 226, 255))
    # the only glint: cold pale eyes
    c.ell(0.745, 0.469, 0.013, 0.016, (216, 226, 236, 255))


# ================================================================ lab ghouls
def ghoul_base(c, coat, dark=None, glisten=False):
    """Knuckle-walking quadruped stance: long arms to the floor, low muzzle."""
    dark = dark or tuple(max(0, v - 30) for v in coat[:3]) + (255,)
    # hind legs, bent
    for fx in (0.285, 0.375):
        c.taper([(fx, 0.52), (fx - 0.02, 0.70), (fx + 0.012, 0.90)], 0.058, 0.042, dark, DARK, 0.009)
        c.ell(fx + 0.026, 0.905, 0.028, 0.011, dark, DARK, 0.007)
    # torso, sloped down toward the front
    c.blob(0.475, 0.445, 0.255, 0.125, coat, DARK, n=12, jit=0.07)
    # unnaturally long front arms reaching the floor
    for (sx, fx) in [(0.655, 0.685), (0.715, 0.755)]:
        c.taper([(sx, 0.50), (fx - 0.01, 0.70), (fx, 0.885)], 0.052, 0.040, coat, DARK, 0.009)
        for fi in range(3):  # long claws clicking on the floor
            dx = (fi - 1) * 0.020
            c.poly([(fx + dx - 0.006, 0.885), (fx + dx + 0.006, 0.885),
                    (fx + dx + 0.004, 0.935)], (226, 222, 210, 255), DARK, 0.006)
    if glisten:
        c.speckles(0.475, 0.44, 0.19, 0.085, 12, [(246, 240, 238, 255)], 0.004, 0.009)


def ghoul_head(c, hx, hy, coat, mouth_open=0.030, drool=False):
    c.ell(hx, hy, 0.082, 0.070, coat, DARK, 0.010)
    # extended muzzle
    c.poly([(hx + 0.035, hy + 0.012), (hx + 0.125, hy + 0.032),
            (hx + 0.115, hy + 0.068), (hx + 0.03, hy + 0.062)], coat, DARK, 0.009)
    # flat black eyes
    c.ell(hx + 0.018, hy - 0.018, 0.016, 0.013, (8, 8, 10, 255))
    # fanged mouth
    c.ell(hx + 0.078, hy + 0.052, 0.034, mouth_open, (26, 16, 16, 255))
    for i in range(3):
        fx = hx + 0.055 + i * 0.020
        c.poly([(fx, hy + 0.040), (fx + 0.011, hy + 0.040), (fx + 0.005, hy + 0.062)],
               (238, 236, 228, 255))
    c.ell(hx + 0.118, hy + 0.030, 0.010, 0.008, (24, 20, 20, 255))  # nostril
    if drool:
        c.line([(hx + 0.085, hy + 0.075), (hx + 0.088, hy + 0.135)], (226, 232, 234, 255), 0.009)
        c.ell(hx + 0.088, hy + 0.148, 0.008, 0.011, (226, 232, 234, 255))


def ghoul_adult(c):
    coat = (196, 192, 184, 255)  # pallid, leathery
    ghoul_base(c, coat)
    c.speckles(0.475, 0.445, 0.18, 0.08, 6, [(172, 166, 156, 255)], 0.012, 0.026)
    ghoul_head(c, 0.755, 0.415, coat, mouth_open=0.026)


def ghoul_baby(c):
    coat = (206, 168, 160, 255)  # grey-pink and glistening
    ghoul_base(c, coat, glisten=True)
    # thick neck swinging the head low
    c.taper([(0.665, 0.44), (0.715, 0.455), (0.745, 0.465)], 0.085, 0.07, coat, DARK, 0.009)
    ghoul_head(c, 0.76, 0.465, coat, mouth_open=0.034, drool=True)
    # shiver ticks: still getting the hang of its body
    for (tx, ty, ta) in [(0.30, 0.315, -14), (0.47, 0.295, 6), (0.615, 0.325, 14),
                         (0.255, 0.47, -20)]:
        c.arc(tx, ty, 0.026, 0.020, 200 + ta, 320 + ta, (120, 96, 92, 255), 0.008)


def ghoul_juvenile(c):
    coat = (190, 186, 178, 255)
    ghoul_base(c, coat)
    ghoul_head(c, 0.755, 0.42, coat, mouth_open=0.042)  # mouth hangs gaping
    # one claw cracked
    c.line([(0.759, 0.888), (0.763, 0.918)], (120, 114, 104, 255), 0.006)


# ================================================================ pupae
def pupa(c, head_col, bulk=1.0, tint=None):
    # head + neck bare; everything below wrapped in sticky black fibers
    wrap = (38, 34, 36, 255)
    cx = 0.5
    top, bot = 0.335, 0.955
    hw = 0.155 * bulk
    c.poly([(cx - hw * 0.72, top), (cx + hw * 0.72, top),
            (cx + hw, bot - 0.06), (cx + hw * 0.8, bot),
            (cx - hw * 0.8, bot), (cx - hw, bot - 0.06)], wrap, DARK, 0.011)
    # fiber strands wound around the cocoon
    for i in range(7):
        y = top + 0.055 + i * (bot - top - 0.09) / 6.5
        wob = c.rng.uniform(-0.008, 0.008)
        c.line([(cx - hw * (0.94 - i * 0.012), y), (cx + hw * (0.94 - i * 0.012), y + wob)],
               (74, 66, 70, 255), 0.012)
    # rhythmic movement beneath: soft bulges pressing out
    for (bx, by, br) in [(cx - 0.05 * bulk, 0.52, 0.052), (cx + 0.045 * bulk, 0.66, 0.058),
                         (cx - 0.03 * bulk, 0.79, 0.048)]:
        c.ell(bx, by, br * bulk, br * 0.78 * bulk, (56, 49, 52, 255))
    if tint:
        c.speckles(cx, 0.62, hw * 0.7, 0.24, 7, [tint], 0.010, 0.024)
    # shoulders + head above the wrap
    c.ell(cx, top + 0.005, hw * 0.78, 0.045, head_col, DARK, 0.010)
    c.ell(cx, 0.205, 0.082, 0.095, head_col, DARK, 0.011)
    zed_face(c, cx, 0.205, r=(0.082, 0.095), eye=(16, 13, 11, 255))


def brute_pupa(c):
    pupa(c, (158, 88, 68, 255), bulk=1.06)  # muscular red-toned corpse


def brute_pupa_decoy(c):
    pupa(c, (158, 88, 68, 255), bulk=1.06)  # identical twin by design (seeded jitter only)


def hulk_pupa(c):
    pupa(c, (204, 172, 192, 255), bulk=1.5, tint=(150, 96, 132, 255))  # white_magenta


def hulk_pupa_decoy(c):
    pupa(c, (204, 172, 192, 255), bulk=1.5, tint=(150, 96, 132, 255))


# ================================================================ pseudo test monsters
def pseudo_sponge(c):
    # debug bullet sponge: a shooting-range target dummy riddled with holes
    body = (216, 216, 220, 255)
    a = humanoid(c, skin=body, cloth=(198, 198, 204, 255), bulk=1.05)
    # bullseye on the chest
    for r, col in [(0.062, (206, 60, 52, 255)), (0.044, (240, 240, 242, 255)),
                   (0.026, (206, 60, 52, 255))]:
        c.ell(0.5, 0.455, r, r, col)
    # its symbol is X: bold cross over the torso
    c.line([(0.415, 0.335), (0.585, 0.575)], (40, 38, 44, 255), 0.020)
    c.line([(0.585, 0.335), (0.415, 0.575)], (40, 38, 44, 255), 0.020)
    # bullet holes, clustered where testers aim
    c.speckles(0.5, 0.45, 0.085, 0.13, 16, [(30, 28, 32, 255)], 0.005, 0.010)
    c.speckles(0.5, 0.17, 0.05, 0.06, 5, [(30, 28, 32, 255)], 0.004, 0.008)
    zed_face(c, *a["head"], eye=(40, 38, 44, 255), mouth=(40, 38, 44, 255))


def pseudo_weakpoint(c):
    # weakpoint test dummy: plain body, glowing marked weak points
    body = (196, 198, 204, 255)
    a = humanoid(c, skin=body, cloth=(172, 174, 182, 255), bulk=1.05)
    zed_face(c, *a["head"], eye=(40, 38, 44, 255), mouth=(40, 38, 44, 255))
    mark = (240, 214, 64, 255)
    for (mx, my, mr) in [(0.5, 0.17, 0.030), (0.5, 0.44, 0.036), (0.455, 0.79, 0.024),
                         (0.615, 0.47, 0.022)]:
        c.ell(mx, my, mr, mr, mark, DARK, 0.008)
        c.ell(mx, my, mr * 0.42, mr * 0.42, (250, 120, 60, 255))
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)]:  # crosshair ticks
            c.line([(mx + dx * mr * 1.25, my + dy * mr * 1.25),
                    (mx + dx * mr * 1.7, my + dy * mr * 1.7)], (40, 38, 44, 255), 0.009)
    # its symbol is X, stencilled on the belly
    c.line([(0.472, 0.545), (0.528, 0.60)], (40, 38, 44, 255), 0.013)
    c.line([(0.528, 0.545), (0.472, 0.60)], (40, 38, 44, 255), 0.013)


# ================================================================ table + main
# (id, sheet, size, renderer) -- volume resolved through copy-from chains:
# 32x32 (<100 L): necro 62.5, slasher 62.5, scorched lasher/rust 62.5,
#   fireproof 62.5, fireproof grappler 62.5, soldier/survivor nw 62.5,
#   child base 62.5, xedra 62.5, zomborg boss 62.5, dogs 30, cats 4,
#   ghouls 42.5-48.6, pseudo pair 62.5
# 64x64 (100-400 L): executioner 108.5, reaper 120, scorched brute 100,
#   fiend rust 100, fireproof brute 100, mummified brute 100,
#   zomborg devourers 120, brute pupae 100
# 96x96 (>400 L): zombie horses 550, grue 435, scorched dog hulk 440,
#   hive hulk pupae 875
ENTRIES = [
    ("mon_skeleton_necro", "pngs_normal_32x32", 32, skel_necro),
    ("mon_skeleton_slasher", "pngs_normal_32x32", 32, skel_slasher),
    ("mon_zombie_scorched_lasher", "pngs_normal_32x32", 32, scorched_lasher),
    ("mon_zombie_scorched_rust", "pngs_normal_32x32", 32, scorched_rust),
    ("mon_zombie_fireproof", "pngs_normal_32x32", 32, fireproof),
    ("mon_zombie_fireproof_grappler", "pngs_normal_32x32", 32, fireproof_grappler),
    ("mon_zombie_soldier_no_weakpoints", "pngs_normal_32x32", 32, soldier_nw),
    ("mon_zombie_survivor_no_weakpoints", "pngs_normal_32x32", 32, survivor_nw),
    ("mon_zombie_child_base", "pngs_normal_32x32", 32, zombie_child),
    ("mon_zombie_xedra_scarlet_sea", "pngs_normal_32x32", 32, xedra_scarlet),
    ("mon_zomborg_boss_string_dimension", "pngs_normal_32x32", 32, zomborg_boss),
    ("mon_dog_antimemetic", "pngs_normal_32x32", 32, dog_antimemetic),
    ("mon_dog_cuspated", "pngs_normal_32x32", 32, dog_cuspated),
    ("mon_dog_howling", "pngs_normal_32x32", 32, dog_howling),
    ("mon_dog_incorporeal", "pngs_normal_32x32", 32, dog_incorporeal),
    ("mon_dog_scorched", "pngs_normal_32x32", 32, dog_scorched),
    ("mon_dog_telekenetic", "pngs_normal_32x32", 32, dog_telekenetic),
    ("mon_cat_liquid", "pngs_normal_32x32", 32, cat_liquid),
    ("mon_cat_void", "pngs_normal_32x32", 32, cat_void),
    ("mon_ghoul_lab_adult", "pngs_normal_32x32", 32, ghoul_adult),
    ("mon_ghoul_lab_baby", "pngs_normal_32x32", 32, ghoul_baby),
    ("mon_ghoul_lab_juvenile", "pngs_normal_32x32", 32, ghoul_juvenile),
    ("pseudo_debug_bullet_sponge", "pngs_normal_32x32", 32, pseudo_sponge),
    ("pseudo_weakpoint_mon", "pngs_normal_32x32", 32, pseudo_weakpoint),
    ("mon_skeleton_executioner", "pngs_large_64x64", 64, skel_executioner),
    ("mon_skeleton_reaper", "pngs_large_64x64", 64, skel_reaper),
    ("mon_zombie_scorched_brute", "pngs_large_64x64", 64, scorched_brute),
    ("mon_zombie_fiend_rust", "pngs_large_64x64", 64, fiend_rust),
    ("mon_zombie_fireproof_brute", "pngs_large_64x64", 64, fireproof_brute),
    ("mon_zombie_brute_string_dimension", "pngs_large_64x64", 64, brute_string),
    ("mon_zomborg_devourer", "pngs_large_64x64", 64, zomborg_devourer),
    ("mon_zomborg_devourer_string_dimension", "pngs_large_64x64", 64, zomborg_devourer_string),
    ("mon_brute_pupa", "pngs_large_64x64", 64, brute_pupa),
    ("mon_brute_pupa_decoy", "pngs_large_64x64", 64, brute_pupa_decoy),
    ("mon_zombie_horse_black", "pngs_giant_96x96", 96, horse_black),
    ("mon_zombie_horse_pale", "pngs_giant_96x96", 96, horse_pale),
    ("mon_zombie_horse_red", "pngs_giant_96x96", 96, horse_red),
    ("mon_zombie_horse_white", "pngs_giant_96x96", 96, horse_white),
    ("mon_zombie_inner_cabins_grue", "pngs_giant_96x96", 96, grue),
    ("mon_dog_scorched_hulk", "pngs_giant_96x96", 96, dog_scorched_hulk),
    ("mon_hulk_pupa", "pngs_giant_96x96", 96, hulk_pupa),
    ("mon_hulk_pupa_decoy", "pngs_giant_96x96", 96, hulk_pupa_decoy),
]


def render(mid, size, fn):
    c = Ctx(size, mid)
    fn(c)
    alphas = set(c.img.getchannel("A").getdata())
    assert alphas <= {0, 255}, f"{mid}: non-binary alpha {sorted(alphas)[:8]}"
    return c.img


def main():
    imgs = {}
    for mid, sheet, size, fn in ENTRIES:
        img = render(mid, size, fn)
        imgs[mid] = img
        folder = os.path.join(BASE, sheet, "monsters", mid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, mid + ".png"))
        with open(os.path.join(folder, mid + ".json"), "w") as f:
            f.write(json.dumps([{"id": mid, "fg": mid, "bg": ""}], indent=2) + "\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print(f"wrote {sheet}/monsters/{mid}/{mid}.png ({size}, {size})")
    # contact sheet: 7 x 6 grid of 96px cells
    cell, cols = 96, 7
    rows = (len(ENTRIES) + cols - 1) // cols
    sheet_img = Image.new("RGBA", (cols * cell, rows * cell), (34, 36, 44, 255))
    sd = ImageDraw.Draw(sheet_img)
    for i, (mid, _sheet, size, _fn) in enumerate(ENTRIES):
        cx, cy = (i % cols) * cell, (i // cols) * cell
        sd.rectangle([cx, cy, cx + cell - 1, cy + cell - 1], outline=(58, 60, 70, 255))
        im = imgs[mid]
        if size != cell:
            im = im.resize((cell, cell), Image.NEAREST)
        sheet_img.alpha_composite(im, (cx, cy))
    sheet_img.save(os.path.join(BASE, "preview_finisher_undead.png"))
    print(f"wrote preview_finisher_undead.png ({sheet_img.size[0]}, {sheet_img.size[1]})")


if __name__ == "__main__":
    main()
